use axum::{extract::State, Json};
use sqlx::Row;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    auth::{generate_access_token, generate_refresh_token, hash_password, verify_password, AuthUser},
    dto::{AuthResponse, LoginRequest, OkResponse, RefreshRequest, RegisterRequest, UserDto},
    error::AppError,
    state::AppState,
    ws,
};

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, body = AuthResponse),
        (status = 409, body = crate::dto::OkResponse)
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let username = payload.username.trim();
    let email = payload.email.trim().to_lowercase();
    if username.is_empty() || payload.password.len() < 6 {
        return Err(AppError::BadRequest("invalid username or password".to_string()));
    }

    if !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
        return Err(AppError::BadRequest("invalid email".to_string()));
    }

    let user_id = Uuid::new_v4().to_string();
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let pwd_hash = hash_password(&payload.password)?;

    let insert = sqlx::query(
        "INSERT INTO users(id, username, email, password_hash, created_at) VALUES(?, ?, ?, ?, ?)",
    )
    .bind(&user_id)
    .bind(username)
    .bind(&email)
    .bind(pwd_hash)
    .bind(now)
    .execute(&state.db)
    .await;

    if insert.is_err() {
        return Err(AppError::Conflict("username or email already exists".to_string()));
    }

    let access_token = generate_access_token(&user_id, &state.config)?;
    let refresh_token = generate_refresh_token();
    let exp = (OffsetDateTime::now_utc() + Duration::days(state.config.refresh_token_days)).unix_timestamp();

    sqlx::query("INSERT INTO refresh_tokens(token, user_id, expires_at) VALUES(?, ?, ?)")
        .bind(&refresh_token)
        .bind(&user_id)
        .bind(exp)
        .execute(&state.db)
        .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user: UserDto {
            id: user_id,
            username: username.to_string(),
            avatar_mime: None,
            created_at: now,
        },
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses((status = 200, body = AuthResponse))
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let login_value = payload.username.trim();
    let login_email = login_value.to_lowercase();

    let row = sqlx::query("SELECT id, username, password_hash, avatar_mime, created_at FROM users WHERE username = ? OR email = ?")
        .bind(login_value)
        .bind(login_email)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let hash: String = row.get("password_hash");
    if !verify_password(&payload.password, &hash)? {
        return Err(AppError::Unauthorized);
    }

    let user_id: String = row.get("id");
    let access_token = generate_access_token(&user_id, &state.config)?;
    let refresh_token = generate_refresh_token();
    let exp = (OffsetDateTime::now_utc() + Duration::days(state.config.refresh_token_days)).unix_timestamp();

    sqlx::query("INSERT INTO refresh_tokens(token, user_id, expires_at) VALUES(?, ?, ?)")
        .bind(&refresh_token)
        .bind(&user_id)
        .bind(exp)
        .execute(&state.db)
        .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user: UserDto {
            id: user_id,
            username: row.get("username"),
            avatar_mime: row.try_get("avatar_mime").ok(),
            created_at: row.get("created_at"),
        },
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = RefreshRequest,
    responses((status = 200, body = AuthResponse))
)]
pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let provided_refresh = payload.refresh_token.clone();
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let row = sqlx::query(
        "SELECT u.id, u.username, u.avatar_mime, u.created_at
         FROM refresh_tokens t
         JOIN users u ON u.id = t.user_id
         WHERE t.token = ? AND t.expires_at > ?",
    )
    .bind(provided_refresh.clone())
    .bind(now)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let user_id: String = row.get("id");
    let access_token = generate_access_token(&user_id, &state.config)?;
    let new_refresh = generate_refresh_token();
    let exp = (OffsetDateTime::now_utc() + Duration::days(state.config.refresh_token_days)).unix_timestamp();

    sqlx::query("DELETE FROM refresh_tokens WHERE token = ?")
        .bind(provided_refresh)
        .execute(&state.db)
        .await?;

    sqlx::query("INSERT INTO refresh_tokens(token, user_id, expires_at) VALUES(?, ?, ?)")
        .bind(&new_refresh)
        .bind(&user_id)
        .bind(exp)
        .execute(&state.db)
        .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token: new_refresh,
        user: UserDto {
            id: user_id,
            username: row.get("username"),
            avatar_mime: row.try_get("avatar_mime").ok(),
            created_at: row.get("created_at"),
        },
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses((status = 200, body = OkResponse)),
    security(("bearer_auth" = []))
)]
pub async fn logout(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<OkResponse>, AppError> {
    ws::emit(
        &state,
        "presence.updated",
        serde_json::json!({"user_id": user.user_id, "online": false}),
    );

    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    responses((status = 200, body = UserDto)),
    security(("bearer_auth" = []))
)]
pub async fn me(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<UserDto>, AppError> {
    let row = sqlx::query("SELECT id, username, avatar_mime, created_at FROM users WHERE id = ?")
        .bind(user.user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    Ok(Json(UserDto {
        id: row.get("id"),
        username: row.get("username"),
        avatar_mime: row.try_get("avatar_mime").ok(),
        created_at: row.get("created_at"),
    }))
}
