use axum::{
    body::Bytes,
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use sqlx::Row;
use utoipa::ToSchema;

use crate::{
    auth::AuthUser,
    dto::{OkResponse, UserDto},
    error::AppError,
    state::AppState,
    ws,
};

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct UpdateUsernameRequest {
    pub username: String,
}

#[utoipa::path(
    patch,
    path = "/api/v1/users/me",
    request_body = UpdateUsernameRequest,
    responses((status = 200, body = UserDto)),
    security(("bearer_auth" = []))
)]
pub async fn update_me(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<UpdateUsernameRequest>,
) -> Result<Json<UserDto>, AppError> {
    if payload.username.trim().is_empty() {
        return Err(AppError::BadRequest("username cannot be empty".to_string()));
    }

    let result = sqlx::query("UPDATE users SET username = ? WHERE id = ?")
        .bind(payload.username.trim())
        .bind(&user.user_id)
        .execute(&state.db)
        .await;

    if result.is_err() {
        return Err(AppError::Conflict("username already exists".to_string()));
    }

    let row = sqlx::query("SELECT id, username, avatar_mime, created_at FROM users WHERE id = ?")
        .bind(&user.user_id)
        .fetch_one(&state.db)
        .await?;

    ws::emit(
        &state,
        "user.updated",
        serde_json::json!({"user_id": user.user_id}),
    );

    Ok(Json(UserDto {
        id: row.get("id"),
        username: row.get("username"),
        avatar_mime: row.try_get("avatar_mime").ok(),
        created_at: row.get("created_at"),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/me",
    responses((status = 200, body = OkResponse)),
    security(("bearer_auth" = []))
)]
pub async fn delete_me(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<OkResponse>, AppError> {
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(&user.user_id)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "user.deleted",
        serde_json::json!({"user_id": user.user_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    post,
    path = "/api/v1/users/me/avatar",
    responses((status = 200, body = OkResponse)),
    security(("bearer_auth" = []))
)]
pub async fn upload_avatar(
    State(state): State<AppState>,
    user: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<OkResponse>, AppError> {
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

    sqlx::query("UPDATE users SET avatar_blob = ?, avatar_mime = ? WHERE id = ?")
        .bind(blob)
        .bind(mime)
        .bind(&user.user_id)
        .execute(&state.db)
        .await?;

    ws::emit(
        &state,
        "user.avatar.updated",
        serde_json::json!({"user_id": user.user_id}),
    );

    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}/avatar",
    responses((status = 200, description = "Raw avatar bytes")),
    params(("user_id" = String, Path, description = "User id")),
    security(("bearer_auth" = []))
)]
pub async fn get_avatar(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query("SELECT avatar_blob, avatar_mime FROM users WHERE id = ?")
        .bind(user_id)
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
