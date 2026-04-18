use axum::{extract::Path, extract::State, Json};
use sqlx::Row;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    dto::{MessageDto, OkResponse, SendMessageRequest},
    error::AppError,
    state::AppState,
    ws,
};

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct EditMessageRequest {
    pub content: String,
}

async fn assert_channel_membership(
    state: &AppState,
    user_id: &str,
    channel_id: &str,
) -> Result<(), AppError> {
    let row = sqlx::query(
        "SELECT m.user_id
         FROM channels c
         JOIN server_members m ON m.server_id = c.server_id
         WHERE c.id = ? AND m.user_id = ?",
    )
    .bind(channel_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    if row.is_none() {
        return Err(AppError::Forbidden);
    }

    Ok(())
}

#[utoipa::path(
    post,
    path = "/api/v1/channels/{channel_id}/messages",
    request_body = SendMessageRequest,
    responses((status = 200, body = MessageDto)),
    params(("channel_id" = String, Path, description = "Channel id")),
    security(("bearer_auth" = []))
)]
pub async fn send_message(
    State(state): State<AppState>,
    user: AuthUser,
    Path(channel_id): Path<String>,
    Json(payload): Json<SendMessageRequest>,
) -> Result<Json<MessageDto>, AppError> {
    if payload.content.trim().is_empty() {
        return Err(AppError::BadRequest("content cannot be empty".to_string()));
    }

    assert_channel_membership(&state, &user.user_id, &channel_id).await?;

    let channel = sqlx::query("SELECT server_id FROM channels WHERE id = ?")
        .bind(&channel_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;
    let server_id: String = channel.get("server_id");

    let id = Uuid::new_v4().to_string();
    let now = OffsetDateTime::now_utc().unix_timestamp();

    sqlx::query(
        "INSERT INTO channel_messages(id, channel_id, user_id, content, created_at) VALUES(?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&channel_id)
    .bind(&user.user_id)
    .bind(payload.content.clone())
    .bind(now)
    .execute(&state.db)
    .await?;

    ws::emit(
        &state,
        "channel.message.created",
        serde_json::json!({"id": id, "server_id": server_id, "channel_id": channel_id, "user_id": user.user_id}),
    );

    Ok(Json(MessageDto {
        id,
        channel_id,
        user_id: user.user_id,
        content: payload.content,
        created_at: now,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/channels/{channel_id}/messages",
    responses((status = 200, body = Vec<MessageDto>)),
    params(("channel_id" = String, Path, description = "Channel id")),
    security(("bearer_auth" = []))
)]
pub async fn list_messages(
    State(state): State<AppState>,
    user: AuthUser,
    Path(channel_id): Path<String>,
) -> Result<Json<Vec<MessageDto>>, AppError> {
    assert_channel_membership(&state, &user.user_id, &channel_id).await?;

    let rows = sqlx::query(
        "SELECT id, channel_id, user_id, content, created_at
         FROM channel_messages
         WHERE channel_id = ?
         ORDER BY created_at DESC
         LIMIT 100",
    )
    .bind(&channel_id)
    .fetch_all(&state.db)
    .await?;

    let messages = rows
        .into_iter()
        .map(|r| MessageDto {
            id: r.get("id"),
            channel_id: r.get("channel_id"),
            user_id: r.get("user_id"),
            content: r.get("content"),
            created_at: r.get("created_at"),
        })
        .collect();

    Ok(Json(messages))
}

#[utoipa::path(
    patch,
    path = "/api/v1/messages/{message_id}",
    request_body = EditMessageRequest,
    responses((status = 200, body = OkResponse)),
    params(("message_id" = String, Path, description = "Message id")),
    security(("bearer_auth" = []))
)]
pub async fn edit_message(
    State(state): State<AppState>,
    user: AuthUser,
    Path(message_id): Path<String>,
    Json(payload): Json<EditMessageRequest>,
) -> Result<Json<OkResponse>, AppError> {
    let exists = sqlx::query("SELECT user_id, channel_id FROM channel_messages WHERE id = ?")
        .bind(&message_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let author_id: String = exists.get("user_id");
    let channel_id: String = exists.get("channel_id");

    assert_channel_membership(&state, &user.user_id, &channel_id).await?;
    if author_id != user.user_id {
        return Err(AppError::Forbidden);
    }

    sqlx::query("UPDATE channel_messages SET content = ? WHERE id = ?")
        .bind(payload.content.clone())
        .bind(&message_id)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "channel.message.updated",
        serde_json::json!({"id": message_id, "channel_id": channel_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/messages/{message_id}",
    responses((status = 200, body = OkResponse)),
    params(("message_id" = String, Path, description = "Message id")),
    security(("bearer_auth" = []))
)]
pub async fn delete_message(
    State(state): State<AppState>,
    user: AuthUser,
    Path(message_id): Path<String>,
) -> Result<Json<OkResponse>, AppError> {
    let exists = sqlx::query("SELECT user_id, channel_id FROM channel_messages WHERE id = ?")
        .bind(&message_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let author_id: String = exists.get("user_id");
    let channel_id: String = exists.get("channel_id");

    assert_channel_membership(&state, &user.user_id, &channel_id).await?;

    if author_id != user.user_id {
        return Err(AppError::Forbidden);
    }

    sqlx::query("DELETE FROM channel_messages WHERE id = ?")
        .bind(&message_id)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "channel.message.deleted",
        serde_json::json!({"id": message_id, "channel_id": channel_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}
