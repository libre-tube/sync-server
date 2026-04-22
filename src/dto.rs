use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::{Channel, Playlist, Video};

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct RegisterUser {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct LoginUser {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    pub jwt: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct DeleteUser {
    pub password: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct UnsubscribeChannel {
    pub channel_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct CreatePlaylist {
    pub title: String,
    pub description: String,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlaylistResponse {
    pub playlist: Playlist,
    pub videos: Vec<CreateVideo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVideo {
    pub id: String,
    pub title: String,
    pub upload_date: i64,
    pub uploader: Channel,
    pub thumbnail_url: String,
    pub duration: i32,
}
impl From<(&Video, &Channel)> for CreateVideo {
    fn from((video, channel): (&Video, &Channel)) -> Self {
        CreateVideo {
            id: video.id.clone(),
            title: video.title.clone(),
            upload_date: video.upload_date,
            thumbnail_url: video.thumbnail_url.clone(),
            duration: video.duration,
            uploader: channel.clone(),
        }
    }
}

/// Claims to store inside the JWT Token
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// User ID.
    pub sub: String,
    pub exp: usize,
}
