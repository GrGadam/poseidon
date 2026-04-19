use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserDto,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct UserDto {
    pub id: String,
    pub username: String,
    pub avatar_mime: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFriendRequest {
    pub username: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FriendRequestDto {
    pub id: String,
    pub from_user: UserDto,
    pub to_user: UserDto,
    pub created_at: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FriendDto {
    pub user: UserDto,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateServerRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateServerRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ServerDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub is_public: bool,
    pub created_at: i64,
    pub member_count: Option<i64>,
    pub member_role: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateChannelRequest {
    pub name: String,
    pub emoji: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateChannelRequest {
    pub name: Option<String>,
    pub emoji: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChannelDto {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub emoji: String,
    pub created_at: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MessageDto {
    pub id: String,
    pub channel_id: String,
    pub user_id: String,
    pub username: Option<String>,
    pub avatar_mime: Option<String>,
    pub content: String,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OkResponse {
    pub ok: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDmThreadRequest {
    pub user_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DmThreadDto {
    pub id: String,
    pub peer_user: UserDto,
    pub created_at: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SendDmMessageRequest {
    pub content: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EditDmMessageRequest {
    pub content: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DmMessageDto {
    pub id: String,
    pub thread_id: String,
    pub user_id: String,
    pub content: String,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}
