use axum::{extract::Path, extract::State, Json};
use sqlx::Row;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    dto::{CreateFriendRequest, FriendDto, FriendRequestDto, OkResponse, UserDto},
    error::AppError,
    state::AppState,
    ws,
};

#[utoipa::path(
    post,
    path = "/api/v1/friends/requests",
    request_body = CreateFriendRequest,
    responses((status = 200, body = OkResponse)),
    security(("bearer_auth" = []))
)]
pub async fn send_request(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateFriendRequest>,
) -> Result<Json<OkResponse>, AppError> {
    let to = sqlx::query("SELECT id FROM users WHERE username = ?")
        .bind(payload.username.trim())
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let to_user_id: String = to.get("id");
    if to_user_id == user.user_id {
        return Err(AppError::BadRequest("cannot friend yourself".to_string()));
    }

    let id = Uuid::new_v4().to_string();
    let now = OffsetDateTime::now_utc().unix_timestamp();

    let ins = sqlx::query(
        "INSERT INTO friend_requests(id, from_user_id, to_user_id, created_at) VALUES(?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&user.user_id)
    .bind(&to_user_id)
    .bind(now)
    .execute(&state.db)
    .await;

    if ins.is_err() {
        return Err(AppError::Conflict("request already exists".to_string()));
    }

    ws::emit(
        &state,
        "friend.request.created",
        serde_json::json!({"request_id": id, "from_user_id": user.user_id, "to_user_id": to_user_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    get,
    path = "/api/v1/friends/requests",
    responses((status = 200, body = Vec<FriendRequestDto>)),
    security(("bearer_auth" = []))
)]
pub async fn pending_requests(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<FriendRequestDto>>, AppError> {
    let rows = sqlx::query(
        "SELECT fr.id as request_id, fr.created_at as created_at,
                uf.id as from_id, uf.username as from_username, uf.avatar_mime as from_avatar, uf.created_at as from_created,
                ut.id as to_id, ut.username as to_username, ut.avatar_mime as to_avatar, ut.created_at as to_created
         FROM friend_requests fr
         JOIN users uf ON uf.id = fr.from_user_id
         JOIN users ut ON ut.id = fr.to_user_id
         WHERE fr.to_user_id = ?",
    )
    .bind(user.user_id)
    .fetch_all(&state.db)
    .await?;

    let data = rows
        .into_iter()
        .map(|r| FriendRequestDto {
            id: r.get("request_id"),
            from_user: UserDto {
                id: r.get("from_id"),
                username: r.get("from_username"),
                avatar_mime: r.try_get("from_avatar").ok(),
                created_at: r.get("from_created"),
            },
            to_user: UserDto {
                id: r.get("to_id"),
                username: r.get("to_username"),
                avatar_mime: r.try_get("to_avatar").ok(),
                created_at: r.get("to_created"),
            },
            created_at: r.get("created_at"),
        })
        .collect::<Vec<_>>();

    Ok(Json(data))
}

#[utoipa::path(
    post,
    path = "/api/v1/friends/requests/{request_id}/accept",
    responses((status = 200, body = OkResponse)),
    params(("request_id" = String, Path, description = "Friend request id")),
    security(("bearer_auth" = []))
)]
pub async fn accept_request(
    State(state): State<AppState>,
    user: AuthUser,
    Path(request_id): Path<String>,
) -> Result<Json<OkResponse>, AppError> {
    let row = sqlx::query("SELECT from_user_id, to_user_id FROM friend_requests WHERE id = ?")
        .bind(&request_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let from_user_id: String = row.get("from_user_id");
    let to_user_id: String = row.get("to_user_id");

    if to_user_id != user.user_id {
        return Err(AppError::Forbidden);
    }

    let (a, b) = if from_user_id < to_user_id {
        (from_user_id.clone(), to_user_id.clone())
    } else {
        (to_user_id.clone(), from_user_id.clone())
    };

    sqlx::query("INSERT OR IGNORE INTO friendships(user_a, user_b, created_at) VALUES(?, ?, ?)")
        .bind(a)
        .bind(b)
        .bind(OffsetDateTime::now_utc().unix_timestamp())
        .execute(&state.db)
        .await?;

    sqlx::query("DELETE FROM friend_requests WHERE id = ?")
        .bind(request_id)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "friend.request.accepted",
        serde_json::json!({"from_user_id": from_user_id, "to_user_id": to_user_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/friends/requests/{request_id}",
    responses((status = 200, body = OkResponse)),
    params(("request_id" = String, Path, description = "Friend request id")),
    security(("bearer_auth" = []))
)]
pub async fn reject_request(
    State(state): State<AppState>,
    user: AuthUser,
    Path(request_id): Path<String>,
) -> Result<Json<OkResponse>, AppError> {
    let row = sqlx::query("SELECT from_user_id, to_user_id FROM friend_requests WHERE id = ?")
        .bind(&request_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let from_user_id: String = row.get("from_user_id");
    let to_user_id: String = row.get("to_user_id");

    if to_user_id != user.user_id {
        return Err(AppError::Forbidden);
    }

    sqlx::query("DELETE FROM friend_requests WHERE id = ?")
        .bind(request_id)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "friend.request.rejected",
        serde_json::json!({"from_user_id": from_user_id, "to_user_id": to_user_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    get,
    path = "/api/v1/friends",
    responses((status = 200, body = Vec<FriendDto>)),
    security(("bearer_auth" = []))
)]
pub async fn list_friends(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<FriendDto>>, AppError> {
    let rows = sqlx::query(
        "SELECT u.id, u.username, u.avatar_mime, u.created_at
         FROM friendships f
         JOIN users u ON u.id = CASE WHEN f.user_a = ? THEN f.user_b ELSE f.user_a END
         WHERE f.user_a = ? OR f.user_b = ?",
    )
    .bind(&user.user_id)
    .bind(&user.user_id)
    .bind(&user.user_id)
    .fetch_all(&state.db)
    .await?;

    let friends = rows
        .into_iter()
        .map(|r| FriendDto {
            user: UserDto {
                id: r.get("id"),
                username: r.get("username"),
                avatar_mime: r.try_get("avatar_mime").ok(),
                created_at: r.get("created_at"),
            },
        })
        .collect();

    Ok(Json(friends))
}

#[utoipa::path(
    delete,
    path = "/api/v1/friends/{friend_user_id}",
    responses((status = 200, body = OkResponse)),
    params(("friend_user_id" = String, Path, description = "Friend user id")),
    security(("bearer_auth" = []))
)]
pub async fn delete_friend(
    State(state): State<AppState>,
    user: AuthUser,
    Path(friend_user_id): Path<String>,
) -> Result<Json<OkResponse>, AppError> {
    let (a, b) = if user.user_id < friend_user_id {
        (user.user_id.clone(), friend_user_id.clone())
    } else {
        (friend_user_id.clone(), user.user_id.clone())
    };

    sqlx::query("DELETE FROM friendships WHERE user_a = ? AND user_b = ?")
        .bind(&a)
        .bind(&b)
        .execute(&state.db)
        .await?;

    sqlx::query("DELETE FROM dm_threads WHERE user_a = ? AND user_b = ?")
        .bind(&a)
        .bind(&b)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "friend.deleted",
        serde_json::json!({"user_a": a, "user_b": b}),
    );

    Ok(Json(OkResponse { ok: true }))
}
