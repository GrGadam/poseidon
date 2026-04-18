use axum::{
    body::Bytes,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use rand::{distr::Alphanumeric, Rng};
use serde::Deserialize;
use sqlx::Row;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    dto::{
        ChannelDto, CreateChannelRequest, CreateServerRequest, OkResponse, ServerDto,
        UpdateChannelRequest, UpdateServerRequest,
    },
    error::AppError,
    state::AppState,
    ws,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct ServerSearchQuery {
    pub query: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct JoinInviteRequest {
    pub code: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RoleUpdateRequest {
    pub role: String,
}

fn is_moderator_or_owner(role: &str) -> bool {
    role == "owner" || role == "moderator"
}

async fn get_role(state: &AppState, user_id: &str, server_id: &str) -> Result<String, AppError> {
    let role = sqlx::query("SELECT role FROM server_members WHERE server_id = ? AND user_id = ?")
        .bind(server_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::Forbidden)?
        .get::<String, _>("role");

    Ok(role)
}

#[utoipa::path(
    post,
    path = "/api/v1/servers",
    request_body = CreateServerRequest,
    responses((status = 200, body = ServerDto)),
    security(("bearer_auth" = []))
)]
pub async fn create_server(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateServerRequest>,
) -> Result<Json<ServerDto>, AppError> {
    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("name cannot be empty".to_string()));
    }

    let id = Uuid::new_v4().to_string();
    let now = OffsetDateTime::now_utc().unix_timestamp();

    sqlx::query(
        "INSERT INTO servers(id, name, description, owner_id, is_public, created_at) VALUES(?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(payload.name.trim())
    .bind(payload.description.clone())
    .bind(&user.user_id)
    .bind(payload.is_public)
    .bind(now)
    .execute(&state.db)
    .await?;

    sqlx::query("INSERT INTO server_members(server_id, user_id, role, joined_at) VALUES(?, ?, 'owner', ?)")
        .bind(&id)
        .bind(&user.user_id)
        .bind(now)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "server.created",
        serde_json::json!({"server_id": id, "owner_id": user.user_id}),
    );

    Ok(Json(ServerDto {
        id,
        name: payload.name,
        description: payload.description,
        owner_id: user.user_id,
        is_public: payload.is_public,
        created_at: now,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/servers",
    responses((status = 200, body = Vec<ServerDto>)),
    security(("bearer_auth" = []))
)]
pub async fn list_user_servers(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<ServerDto>>, AppError> {
    let rows = sqlx::query(
        "SELECT s.id, s.name, s.description, s.owner_id, s.is_public, s.created_at
         FROM servers s
         JOIN server_members m ON m.server_id = s.id
         WHERE m.user_id = ?",
    )
    .bind(user.user_id)
    .fetch_all(&state.db)
    .await?;

    let data = rows
        .into_iter()
        .map(|r| ServerDto {
            id: r.get("id"),
            name: r.get("name"),
            description: r.try_get("description").ok(),
            owner_id: r.get("owner_id"),
            is_public: r.get("is_public"),
            created_at: r.get("created_at"),
        })
        .collect();

    Ok(Json(data))
}

#[utoipa::path(
    patch,
    path = "/api/v1/servers/{server_id}",
    request_body = UpdateServerRequest,
    responses((status = 200, body = OkResponse)),
    params(("server_id" = String, Path, description = "Server id")),
    security(("bearer_auth" = []))
)]
pub async fn update_server(
    State(state): State<AppState>,
    user: AuthUser,
    Path(server_id): Path<String>,
    Json(payload): Json<UpdateServerRequest>,
) -> Result<Json<OkResponse>, AppError> {
    let role = get_role(&state, &user.user_id, &server_id).await?;
    if role != "owner" {
        return Err(AppError::Forbidden);
    }

    let current = sqlx::query("SELECT name, description, is_public FROM servers WHERE id = ?")
        .bind(&server_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let name = payload
        .name
        .unwrap_or_else(|| current.get::<String, _>("name"));
    let description = payload.description.or_else(|| current.try_get("description").ok());
    let is_public = payload
        .is_public
        .unwrap_or_else(|| current.get::<bool, _>("is_public"));

    sqlx::query("UPDATE servers SET name = ?, description = ?, is_public = ? WHERE id = ?")
        .bind(name)
        .bind(description)
        .bind(is_public)
        .bind(&server_id)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "server.updated",
        serde_json::json!({"server_id": server_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/servers/{server_id}",
    responses((status = 200, body = OkResponse)),
    params(("server_id" = String, Path, description = "Server id")),
    security(("bearer_auth" = []))
)]
pub async fn delete_server(
    State(state): State<AppState>,
    user: AuthUser,
    Path(server_id): Path<String>,
) -> Result<Json<OkResponse>, AppError> {
    let role = get_role(&state, &user.user_id, &server_id).await?;
    if role != "owner" {
        return Err(AppError::Forbidden);
    }

    sqlx::query("DELETE FROM servers WHERE id = ?")
        .bind(&server_id)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "server.deleted",
        serde_json::json!({"server_id": server_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    get,
    path = "/api/v1/servers/public",
    params(("query" = Option<String>, Query, description = "Search text")),
    responses((status = 200, body = Vec<ServerDto>)),
    security(("bearer_auth" = []))
)]
pub async fn list_public_servers(
    State(state): State<AppState>,
    Query(q): Query<ServerSearchQuery>,
    _user: AuthUser,
) -> Result<Json<Vec<ServerDto>>, AppError> {
    let search = format!("%{}%", q.query.unwrap_or_default());
    let rows = sqlx::query(
        "SELECT id, name, description, owner_id, is_public, created_at
         FROM servers
         WHERE is_public = 1 AND name LIKE ?",
    )
    .bind(search)
    .fetch_all(&state.db)
    .await?;

    let data = rows
        .into_iter()
        .map(|r| ServerDto {
            id: r.get("id"),
            name: r.get("name"),
            description: r.try_get("description").ok(),
            owner_id: r.get("owner_id"),
            is_public: r.get("is_public"),
            created_at: r.get("created_at"),
        })
        .collect();

    Ok(Json(data))
}

#[utoipa::path(
    post,
    path = "/api/v1/servers/{server_id}/join",
    responses((status = 200, body = OkResponse)),
    params(("server_id" = String, Path, description = "Server id")),
    security(("bearer_auth" = []))
)]
pub async fn join_public_server(
    State(state): State<AppState>,
    user: AuthUser,
    Path(server_id): Path<String>,
) -> Result<Json<OkResponse>, AppError> {
    let row = sqlx::query("SELECT is_public FROM servers WHERE id = ?")
        .bind(&server_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let is_public: bool = row.get("is_public");
    if !is_public {
        return Err(AppError::Forbidden);
    }

    sqlx::query("INSERT OR IGNORE INTO server_members(server_id, user_id, role, joined_at) VALUES(?, ?, 'user', ?)")
        .bind(&server_id)
        .bind(&user.user_id)
        .bind(OffsetDateTime::now_utc().unix_timestamp())
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "server.joined",
        serde_json::json!({"server_id": server_id, "user_id": user.user_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    post,
    path = "/api/v1/servers/{server_id}/invite",
    responses((status = 200, body = String)),
    params(("server_id" = String, Path, description = "Server id")),
    security(("bearer_auth" = []))
)]
pub async fn create_invite(
    State(state): State<AppState>,
    user: AuthUser,
    Path(server_id): Path<String>,
) -> Result<Json<String>, AppError> {
    let role = get_role(&state, &user.user_id, &server_id).await?;
    if !is_moderator_or_owner(&role) {
        return Err(AppError::Forbidden);
    }

    let code: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    sqlx::query("INSERT INTO server_invites(code, server_id, created_by, created_at) VALUES(?, ?, ?, ?)")
        .bind(&code)
        .bind(&server_id)
        .bind(user.user_id)
        .bind(OffsetDateTime::now_utc().unix_timestamp())
        .execute(&state.db)
        .await?;

    Ok(Json(code))
}

#[utoipa::path(
    post,
    path = "/api/v1/servers/join/invite",
    request_body = JoinInviteRequest,
    responses((status = 200, body = OkResponse)),
    security(("bearer_auth" = []))
)]
pub async fn join_by_invite(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<JoinInviteRequest>,
) -> Result<Json<OkResponse>, AppError> {
    let row = sqlx::query(
        "SELECT i.server_id FROM server_invites i
         JOIN servers s ON s.id = i.server_id
         WHERE i.code = ? AND s.is_public = 0",
    )
    .bind(payload.code)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let server_id: String = row.get("server_id");

    sqlx::query("INSERT OR IGNORE INTO server_members(server_id, user_id, role, joined_at) VALUES(?, ?, 'user', ?)")
        .bind(&server_id)
        .bind(&user.user_id)
        .bind(OffsetDateTime::now_utc().unix_timestamp())
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "server.joined",
        serde_json::json!({"server_id": server_id, "user_id": user.user_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    post,
    path = "/api/v1/servers/{server_id}/leave",
    responses((status = 200, body = OkResponse)),
    params(("server_id" = String, Path, description = "Server id")),
    security(("bearer_auth" = []))
)]
pub async fn leave_server(
    State(state): State<AppState>,
    user: AuthUser,
    Path(server_id): Path<String>,
) -> Result<Json<OkResponse>, AppError> {
    sqlx::query("DELETE FROM server_members WHERE server_id = ? AND user_id = ?")
        .bind(&server_id)
        .bind(&user.user_id)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "server.left",
        serde_json::json!({"server_id": server_id, "user_id": user.user_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    post,
    path = "/api/v1/servers/{server_id}/members/{member_id}/role",
    request_body = RoleUpdateRequest,
    responses((status = 200, body = OkResponse)),
    params(("server_id" = String, Path, description = "Server id"), ("member_id" = String, Path, description = "Member user id")),
    security(("bearer_auth" = []))
)]
pub async fn update_member_role(
    State(state): State<AppState>,
    user: AuthUser,
    Path((server_id, member_id)): Path<(String, String)>,
    Json(payload): Json<RoleUpdateRequest>,
) -> Result<Json<OkResponse>, AppError> {
    let role = get_role(&state, &user.user_id, &server_id).await?;
    if role != "owner" {
        return Err(AppError::Forbidden);
    }
    if payload.role != "user" && payload.role != "moderator" {
        return Err(AppError::BadRequest("invalid role".to_string()));
    }

    sqlx::query("UPDATE server_members SET role = ? WHERE server_id = ? AND user_id = ?")
        .bind(payload.role.clone())
        .bind(&server_id)
        .bind(&member_id)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "member.role.updated",
        serde_json::json!({"server_id": server_id, "user_id": member_id, "role": payload.role}),
    );

    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/servers/{server_id}/members/{member_id}",
    responses((status = 200, body = OkResponse)),
    params(("server_id" = String, Path, description = "Server id"), ("member_id" = String, Path, description = "Member user id")),
    security(("bearer_auth" = []))
)]
pub async fn remove_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path((server_id, member_id)): Path<(String, String)>,
) -> Result<Json<OkResponse>, AppError> {
    let role = get_role(&state, &user.user_id, &server_id).await?;
    if !is_moderator_or_owner(&role) {
        return Err(AppError::Forbidden);
    }

    sqlx::query("DELETE FROM server_members WHERE server_id = ? AND user_id = ?")
        .bind(&server_id)
        .bind(&member_id)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "member.removed",
        serde_json::json!({"server_id": server_id, "user_id": member_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    post,
    path = "/api/v1/servers/{server_id}/channels",
    request_body = CreateChannelRequest,
    responses((status = 200, body = ChannelDto)),
    params(("server_id" = String, Path, description = "Server id")),
    security(("bearer_auth" = []))
)]
pub async fn create_channel(
    State(state): State<AppState>,
    user: AuthUser,
    Path(server_id): Path<String>,
    Json(payload): Json<CreateChannelRequest>,
) -> Result<Json<ChannelDto>, AppError> {
    let role = get_role(&state, &user.user_id, &server_id).await?;
    if !is_moderator_or_owner(&role) {
        return Err(AppError::Forbidden);
    }

    let id = Uuid::new_v4().to_string();
    let now = OffsetDateTime::now_utc().unix_timestamp();

    sqlx::query("INSERT INTO channels(id, server_id, name, emoji, created_at) VALUES(?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(&server_id)
        .bind(payload.name.trim())
        .bind(payload.emoji.clone())
        .bind(now)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "channel.created",
        serde_json::json!({"server_id": server_id, "channel_id": id}),
    );

    Ok(Json(ChannelDto {
        id,
        server_id,
        name: payload.name,
        emoji: payload.emoji,
        created_at: now,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/servers/{server_id}/channels",
    responses((status = 200, body = Vec<ChannelDto>)),
    params(("server_id" = String, Path, description = "Server id")),
    security(("bearer_auth" = []))
)]
pub async fn list_channels(
    State(state): State<AppState>,
    user: AuthUser,
    Path(server_id): Path<String>,
) -> Result<Json<Vec<ChannelDto>>, AppError> {
    let _ = get_role(&state, &user.user_id, &server_id).await?;

    let rows = sqlx::query("SELECT id, server_id, name, emoji, created_at FROM channels WHERE server_id = ?")
        .bind(&server_id)
        .fetch_all(&state.db)
        .await?;

    let channels = rows
        .into_iter()
        .map(|r| ChannelDto {
            id: r.get("id"),
            server_id: r.get("server_id"),
            name: r.get("name"),
            emoji: r.get("emoji"),
            created_at: r.get("created_at"),
        })
        .collect();

    Ok(Json(channels))
}

#[utoipa::path(
    post,
    path = "/api/v1/servers/{server_id}/avatar",
    responses((status = 200, body = OkResponse)),
    params(("server_id" = String, Path, description = "Server id")),
    security(("bearer_auth" = []))
)]
pub async fn upload_server_avatar(
    State(state): State<AppState>,
    user: AuthUser,
    Path(server_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<OkResponse>, AppError> {
    let role = get_role(&state, &user.user_id, &server_id).await?;
    if role != "owner" {
        return Err(AppError::Forbidden);
    }

    let mut avatar: Option<(Vec<u8>, String)> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::BadRequest("invalid multipart body".to_string()))?
    {
        if field.name() != Some("avatar") {
            continue;
        }

        let mime = field
            .content_type()
            .map(ToString::to_string)
            .ok_or(AppError::BadRequest("missing content type".to_string()))?;

        if mime != "image/png" && mime != "image/jpeg" {
            return Err(AppError::BadRequest("only png/jpeg is allowed".to_string()));
        }

        let data = field
            .bytes()
            .await
            .map_err(|_| AppError::BadRequest("invalid file bytes".to_string()))?;

        if data.is_empty() || data.len() > 5 * 1024 * 1024 {
            return Err(AppError::BadRequest("file size must be 1B..5MB".to_string()));
        }

        avatar = Some((data.to_vec(), mime));
        break;
    }

    let (blob, mime) = avatar.ok_or(AppError::BadRequest("avatar field is required".to_string()))?;

    sqlx::query("UPDATE servers SET avatar_blob = ?, avatar_mime = ? WHERE id = ?")
        .bind(blob)
        .bind(mime)
        .bind(&server_id)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "server.updated",
        serde_json::json!({"server_id": server_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    get,
    path = "/api/v1/servers/{server_id}/avatar",
    responses((status = 200, description = "Raw server avatar bytes")),
    params(("server_id" = String, Path, description = "Server id")),
    security(("bearer_auth" = []))
)]
pub async fn get_server_avatar(
    State(state): State<AppState>,
    user: AuthUser,
    Path(server_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let _ = get_role(&state, &user.user_id, &server_id).await?;

    let row = sqlx::query("SELECT avatar_blob, avatar_mime FROM servers WHERE id = ?")
        .bind(&server_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let mime: Option<String> = row.try_get("avatar_mime").ok();
    let blob: Option<Vec<u8>> = row.try_get("avatar_blob").ok();

    let mime = mime.ok_or(AppError::NotFound)?;
    let blob = blob.ok_or(AppError::NotFound)?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, mime)],
        Bytes::from(blob),
    ))
}

#[utoipa::path(
    patch,
    path = "/api/v1/channels/{channel_id}",
    request_body = UpdateChannelRequest,
    responses((status = 200, body = OkResponse)),
    params(("channel_id" = String, Path, description = "Channel id")),
    security(("bearer_auth" = []))
)]
pub async fn update_channel(
    State(state): State<AppState>,
    user: AuthUser,
    Path(channel_id): Path<String>,
    Json(payload): Json<UpdateChannelRequest>,
) -> Result<Json<OkResponse>, AppError> {
    let row = sqlx::query("SELECT server_id, name, emoji FROM channels WHERE id = ?")
        .bind(&channel_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let server_id: String = row.get("server_id");
    let role = get_role(&state, &user.user_id, &server_id).await?;
    if !is_moderator_or_owner(&role) {
        return Err(AppError::Forbidden);
    }

    let name = payload.name.unwrap_or_else(|| row.get("name"));
    let emoji = payload.emoji.unwrap_or_else(|| row.get("emoji"));

    sqlx::query("UPDATE channels SET name = ?, emoji = ? WHERE id = ?")
        .bind(name)
        .bind(emoji)
        .bind(&channel_id)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "channel.updated",
        serde_json::json!({"server_id": server_id, "channel_id": channel_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/channels/{channel_id}",
    responses((status = 200, body = OkResponse)),
    params(("channel_id" = String, Path, description = "Channel id")),
    security(("bearer_auth" = []))
)]
pub async fn delete_channel(
    State(state): State<AppState>,
    user: AuthUser,
    Path(channel_id): Path<String>,
) -> Result<Json<OkResponse>, AppError> {
    let row = sqlx::query("SELECT server_id FROM channels WHERE id = ?")
        .bind(&channel_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let server_id: String = row.get("server_id");
    let role = get_role(&state, &user.user_id, &server_id).await?;
    if !is_moderator_or_owner(&role) {
        return Err(AppError::Forbidden);
    }

    sqlx::query("DELETE FROM channels WHERE id = ?")
        .bind(&channel_id)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "channel.deleted",
        serde_json::json!({"server_id": server_id, "channel_id": channel_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}
