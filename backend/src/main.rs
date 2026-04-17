mod auth;
mod config;
mod dto;
mod error;
mod routes;
mod state;
mod ws;

use std::{collections::HashMap, sync::Arc};

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use config::Config;
use sqlx::sqlite::SqlitePoolOptions;
use state::AppState;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::auth::register,
        routes::auth::login,
        routes::auth::refresh,
        routes::auth::logout,
        routes::auth::me,
        routes::friends::send_request,
        routes::friends::pending_requests,
        routes::friends::accept_request,
        routes::friends::reject_request,
        routes::friends::list_friends,
        routes::friends::delete_friend,
        routes::dm::list_threads,
        routes::dm::create_or_get_thread,
        routes::dm::list_messages,
        routes::dm::send_message,
        routes::dm::edit_message,
        routes::dm::delete_message,
        routes::users::update_me,
        routes::users::delete_me,
        routes::users::change_password,
        routes::users::upload_avatar,
        routes::users::get_avatar,
        routes::servers::create_server,
        routes::servers::list_user_servers,
        routes::servers::update_server,
        routes::servers::delete_server,
        routes::servers::list_public_servers,
        routes::servers::join_public_server,
        routes::servers::create_invite,
        routes::servers::join_by_invite,
        routes::servers::leave_server,
        routes::servers::update_member_role,
        routes::servers::remove_member,
        routes::servers::create_channel,
        routes::servers::list_channels,
        routes::servers::update_channel,
        routes::servers::delete_channel,
        routes::channels::send_message,
        routes::channels::list_messages,
        routes::channels::edit_message,
        routes::channels::delete_message,
    ),
    components(
        schemas(
            dto::RegisterRequest,
            dto::LoginRequest,
            dto::RefreshRequest,
            dto::AuthResponse,
            dto::UserDto,
            dto::CreateFriendRequest,
            dto::FriendRequestDto,
            dto::FriendDto,
            dto::CreateServerRequest,
            dto::UpdateServerRequest,
            dto::ServerDto,
            dto::CreateChannelRequest,
            dto::UpdateChannelRequest,
            dto::ChannelDto,
            dto::SendMessageRequest,
            dto::MessageDto,
            dto::OkResponse,
            dto::CreateDmThreadRequest,
            dto::DmThreadDto,
            dto::SendDmMessageRequest,
            dto::EditDmMessageRequest,
            dto::DmMessageDto,
            routes::auth::LogoutRequest,
            routes::users::UpdateUsernameRequest,
            routes::users::ChangePasswordRequest,
            routes::servers::JoinInviteRequest,
            routes::servers::RoleUpdateRequest,
            routes::channels::EditMessageRequest,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "poseidon", description = "Poseidon chat API")
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Arc::new(Config::from_env());

    let db = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let (ws_tx, _) = broadcast::channel(1024);

    let app_state = AppState {
        db,
        config: config.clone(),
        ws_tx,
        presence_connections: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/refresh", post(routes::auth::refresh))
        .route("/auth/logout", post(routes::auth::logout))
        .route("/auth/me", get(routes::auth::me))
        .route("/users/me", patch(routes::users::update_me).delete(routes::users::delete_me))
        .route("/users/me/password", post(routes::users::change_password))
        .route("/users/me/avatar", post(routes::users::upload_avatar))
        .route("/users/{user_id}/avatar", get(routes::users::get_avatar))
        .route("/friends", get(routes::friends::list_friends))
        .route("/friends/requests", post(routes::friends::send_request).get(routes::friends::pending_requests))
        .route("/friends/requests/{request_id}/accept", post(routes::friends::accept_request))
        .route("/friends/requests/{request_id}", delete(routes::friends::reject_request))
        .route("/friends/{friend_user_id}", delete(routes::friends::delete_friend))
        .route("/dm/threads", get(routes::dm::list_threads).post(routes::dm::create_or_get_thread))
        .route("/dm/threads/{thread_id}/messages", get(routes::dm::list_messages).post(routes::dm::send_message))
        .route("/dm/messages/{message_id}", patch(routes::dm::edit_message).delete(routes::dm::delete_message))
        .route(
            "/servers",
            post(routes::servers::create_server).get(routes::servers::list_user_servers),
        )
        .route(
            "/servers/{server_id}",
            patch(routes::servers::update_server).delete(routes::servers::delete_server),
        )
        .route("/servers/public", get(routes::servers::list_public_servers))
        .route("/servers/join/invite", post(routes::servers::join_by_invite))
        .route("/servers/{server_id}/join", post(routes::servers::join_public_server))
        .route("/servers/{server_id}/invite", post(routes::servers::create_invite))
        .route("/servers/{server_id}/leave", post(routes::servers::leave_server))
        .route(
            "/servers/{server_id}/members/{member_id}",
            delete(routes::servers::remove_member),
        )
        .route(
            "/servers/{server_id}/members/{member_id}/role",
            post(routes::servers::update_member_role),
        )
        .route(
            "/servers/{server_id}/channels",
            post(routes::servers::create_channel).get(routes::servers::list_channels),
        )
        .route(
            "/channels/{channel_id}",
            patch(routes::servers::update_channel).delete(routes::servers::delete_channel),
        )
        .route(
            "/channels/{channel_id}/messages",
            post(routes::channels::send_message).get(routes::channels::list_messages),
        )
        .route("/messages/{message_id}", patch(routes::channels::edit_message).delete(routes::channels::delete_message))
        .route("/ws", get(ws::ws_handler));

    let app = Router::new()
        .nest("/api/v1", api)
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
        .with_state(app_state)
        .layer(cors);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("poseidon backend listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}
