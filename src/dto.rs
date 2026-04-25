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
pub struct CreatePlaylist {
    pub title: String,
    pub description: String,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Eq, PartialEq)]
pub struct ExtendedPlaylist {
    pub id: String,
    pub account_id: String,
    pub title: String,
    pub description: String,
    pub thumbnail_url: Option<String>,
    // only difference from playlist is this video count field:
    // ugly workaround because of https://github.com/diesel-rs/diesel/issues/860
    pub video_count: u64,
}
impl ExtendedPlaylist {
    pub fn from_playlist(playlist: &Playlist, video_count: u64) -> Self {
        ExtendedPlaylist {
            id: playlist.id.clone(),
            account_id: playlist.account_id.clone(),
            title: playlist.title.clone(),
            description: playlist.description.clone(),
            thumbnail_url: playlist.thumbnail_url.clone(),
            video_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlaylistResponse {
    pub playlist: ExtendedPlaylist,
    pub videos: Vec<CreateVideo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVideo {
    pub id: String,
    pub title: String,
    /// Upload date as UNIX timestamp (millis).
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
impl Into<Video> for &CreateVideo {
    fn into(self) -> Video {
        Video {
            id: self.id.clone(),
            title: self.title.clone(),
            upload_date: self.upload_date,
            uploader_id: self.uploader.id.clone(),
            thumbnail_url: self.thumbnail_url.clone(),
            duration: self.duration,
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
