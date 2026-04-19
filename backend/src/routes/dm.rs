use axum::{extract::Path, extract::State, Json};
use sqlx::Row;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    dto::{
        CreateDmThreadRequest, DmMessageDto, DmThreadDto, EditDmMessageRequest, OkResponse,
        SendDmMessageRequest, UserDto,
    },
    error::AppError,
    state::AppState,
    ws,
};

fn sort_pair(a: &str, b: &str) -> (String, String) {
    if a < b {
        (a.to_string(), b.to_string())
    } else {
        (b.to_string(), a.to_string())
    }
}

async fn assert_thread_membership(
    state: &AppState,
    user_id: &str,
    thread_id: &str,
) -> Result<(String, String), AppError> {
    let row = sqlx::query("SELECT user_a, user_b FROM dm_threads WHERE id = ?")
        .bind(thread_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let user_a: String = row.get("user_a");
    let user_b: String = row.get("user_b");

    if user_id != user_a && user_id != user_b {
        return Err(AppError::Forbidden);
    }

    Ok((user_a, user_b))
}

#[utoipa::path(
    get,
    path = "/api/v1/dm/threads",
    responses((status = 200, body = Vec<DmThreadDto>)),
    security(("bearer_auth" = []))
)]
pub async fn list_threads(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<DmThreadDto>>, AppError> {
    let rows = sqlx::query(
        "SELECT t.id as thread_id, t.created_at as thread_created,
                u.id as peer_id, u.username as peer_username, u.avatar_mime as peer_avatar, u.created_at as peer_created
         FROM dm_threads t
         JOIN users u ON u.id = CASE WHEN t.user_a = ? THEN t.user_b ELSE t.user_a END
         WHERE t.user_a = ? OR t.user_b = ?
         ORDER BY t.created_at DESC",
    )
    .bind(&user.user_id)
    .bind(&user.user_id)
    .bind(&user.user_id)
    .fetch_all(&state.db)
    .await?;

    let threads = rows
        .into_iter()
        .map(|r| DmThreadDto {
            id: r.get("thread_id"),
            peer_user: UserDto {
                id: r.get("peer_id"),
                username: r.get("peer_username"),
                avatar_mime: r.try_get("peer_avatar").ok(),
                created_at: r.get("peer_created"),
            },
            created_at: r.get("thread_created"),
        })
        .collect::<Vec<_>>();

    Ok(Json(threads))
}

#[utoipa::path(
    post,
    path = "/api/v1/dm/threads",
    request_body = CreateDmThreadRequest,
    responses((status = 200, body = DmThreadDto)),
    security(("bearer_auth" = []))
)]
pub async fn create_or_get_thread(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateDmThreadRequest>,
) -> Result<Json<DmThreadDto>, AppError> {
    if payload.user_id == user.user_id {
        return Err(AppError::BadRequest("cannot create dm with yourself".to_string()));
    }

    let (user_a, user_b) = sort_pair(&user.user_id, &payload.user_id);

    let friend = sqlx::query("SELECT 1 FROM friendships WHERE user_a = ? AND user_b = ?")
        .bind(&user_a)
        .bind(&user_b)
        .fetch_optional(&state.db)
        .await?;

    if friend.is_none() {
        return Err(AppError::Forbidden);
    }

    let existing = sqlx::query("SELECT id, created_at FROM dm_threads WHERE user_a = ? AND user_b = ?")
        .bind(&user_a)
        .bind(&user_b)
        .fetch_optional(&state.db)
        .await?;

    let (thread_id, created_at) = if let Some(row) = existing {
        (row.get::<String, _>("id"), row.get::<i64, _>("created_at"))
    } else {
        let id = Uuid::new_v4().to_string();
        let now = OffsetDateTime::now_utc().unix_timestamp();
        sqlx::query("INSERT INTO dm_threads(id, user_a, user_b, created_at) VALUES(?, ?, ?, ?)")
            .bind(&id)
            .bind(&user_a)
            .bind(&user_b)
            .bind(now)
            .execute(&state.db)
            .await?;

        ws::emit(
            &state,
            "dm.thread.created",
            serde_json::json!({"id": id, "user_a": user_a, "user_b": user_b}),
        );

        (id, now)
    };

    let peer = sqlx::query("SELECT id, username, avatar_mime, created_at FROM users WHERE id = ?")
        .bind(&payload.user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(DmThreadDto {
        id: thread_id,
        peer_user: UserDto {
            id: peer.get("id"),
            username: peer.get("username"),
            avatar_mime: peer.try_get("avatar_mime").ok(),
            created_at: peer.get("created_at"),
        },
        created_at,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/dm/threads/{thread_id}/messages",
    responses((status = 200, body = Vec<DmMessageDto>)),
    params(("thread_id" = String, Path, description = "DM thread id")),
    security(("bearer_auth" = []))
)]
pub async fn list_messages(
    State(state): State<AppState>,
    user: AuthUser,
    Path(thread_id): Path<String>,
) -> Result<Json<Vec<DmMessageDto>>, AppError> {
    assert_thread_membership(&state, &user.user_id, &thread_id).await?;

    let rows = sqlx::query(
        "SELECT id, thread_id, user_id, content, created_at, updated_at
         FROM dm_messages
         WHERE thread_id = ?
         ORDER BY created_at DESC
         LIMIT 100",
    )
    .bind(&thread_id)
    .fetch_all(&state.db)
    .await?;

    let messages = rows
        .into_iter()
        .map(|r| DmMessageDto {
            id: r.get("id"),
            thread_id: r.get("thread_id"),
            user_id: r.get("user_id"),
            content: r.get("content"),
            created_at: r.get("created_at"),
            updated_at: r.try_get("updated_at").ok(),
        })
        .collect::<Vec<_>>();

    Ok(Json(messages))
}

#[utoipa::path(
    post,
    path = "/api/v1/dm/threads/{thread_id}/messages",
    request_body = SendDmMessageRequest,
    responses((status = 200, body = DmMessageDto)),
    params(("thread_id" = String, Path, description = "DM thread id")),
    security(("bearer_auth" = []))
)]
pub async fn send_message(
    State(state): State<AppState>,
    user: AuthUser,
    Path(thread_id): Path<String>,
    Json(payload): Json<SendDmMessageRequest>,
) -> Result<Json<DmMessageDto>, AppError> {
    if payload.content.trim().is_empty() {
        return Err(AppError::BadRequest("content cannot be empty".to_string()));
    }

    assert_thread_membership(&state, &user.user_id, &thread_id).await?;

    let id = Uuid::new_v4().to_string();
    let now = OffsetDateTime::now_utc().unix_timestamp();

    sqlx::query("INSERT INTO dm_messages(id, thread_id, user_id, content, created_at) VALUES(?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(&thread_id)
        .bind(&user.user_id)
        .bind(payload.content.clone())
        .bind(now)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "dm.message.created",
        serde_json::json!({"id": id, "thread_id": thread_id, "user_id": user.user_id}),
    );

    Ok(Json(DmMessageDto {
        id,
        thread_id,
        user_id: user.user_id,
        content: payload.content,
        created_at: now,
        updated_at: None,
    }))
}

#[utoipa::path(
    patch,
    path = "/api/v1/dm/messages/{message_id}",
    request_body = EditDmMessageRequest,
    responses((status = 200, body = OkResponse)),
    params(("message_id" = String, Path, description = "DM message id")),
    security(("bearer_auth" = []))
)]
pub async fn edit_message(
    State(state): State<AppState>,
    user: AuthUser,
    Path(message_id): Path<String>,
    Json(payload): Json<EditDmMessageRequest>,
) -> Result<Json<OkResponse>, AppError> {
    if payload.content.trim().is_empty() {
        return Err(AppError::BadRequest("content cannot be empty".to_string()));
    }

    let row = sqlx::query("SELECT thread_id, user_id FROM dm_messages WHERE id = ?")
        .bind(&message_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let thread_id: String = row.get("thread_id");
    let author_id: String = row.get("user_id");

    assert_thread_membership(&state, &user.user_id, &thread_id).await?;
    if author_id != user.user_id {
        return Err(AppError::Forbidden);
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();

    sqlx::query("UPDATE dm_messages SET content = ?, updated_at = ? WHERE id = ?")
        .bind(payload.content)
        .bind(now)
        .bind(&message_id)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "dm.message.updated",
        serde_json::json!({"id": message_id, "thread_id": thread_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/dm/messages/{message_id}",
    responses((status = 200, body = OkResponse)),
    params(("message_id" = String, Path, description = "DM message id")),
    security(("bearer_auth" = []))
)]
pub async fn delete_message(
    State(state): State<AppState>,
    user: AuthUser,
    Path(message_id): Path<String>,
) -> Result<Json<OkResponse>, AppError> {
    let row = sqlx::query("SELECT thread_id, user_id FROM dm_messages WHERE id = ?")
        .bind(&message_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let thread_id: String = row.get("thread_id");
    let author_id: String = row.get("user_id");

    assert_thread_membership(&state, &user.user_id, &thread_id).await?;
    if author_id != user.user_id {
        return Err(AppError::Forbidden);
    }

    sqlx::query("DELETE FROM dm_messages WHERE id = ?")
        .bind(&message_id)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "dm.message.deleted",
        serde_json::json!({"id": message_id, "thread_id": thread_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}
