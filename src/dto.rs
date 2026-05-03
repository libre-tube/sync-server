use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::{
    Channel, Playlist, PublicPlaylist, SubscriptionGroup, Video, WatchHistoryItem,
};

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

/// Public (API) view of a playlist owned by a user.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Eq, PartialEq)]
pub struct ExtendedPlaylist {
    pub id: String,
    pub title: String,
    pub description: String,
    pub thumbnail_url: Option<String>,
    // only difference from playlist is this video count field:
    // ugly workaround because of https://github.com/diesel-rs/diesel/issues/860
    pub video_count: Option<u64>,
}
impl ExtendedPlaylist {
    pub fn from_playlist(playlist: &Playlist, video_count: u64) -> Self {
        ExtendedPlaylist {
            id: playlist.id.clone(),
            title: playlist.title.clone(),
            description: playlist.description.clone(),
            thumbnail_url: playlist.thumbnail_url.clone(),
            video_count: Some(video_count),
        }
    }

    pub fn from_public_playlist(playlist: &PublicPlaylist) -> Self {
        ExtendedPlaylist {
            id: playlist.id.clone(),
            title: playlist.title.clone(),
            description: playlist.description.clone(),
            thumbnail_url: playlist.thumbnail_url.clone(),
            video_count: playlist.video_count.map(|count| count as u64),
        }
    }
}
impl ExtendedPlaylist {
    pub fn into_public_playlist(self, uploader_id: &str) -> PublicPlaylist {
        PublicPlaylist {
            id: self.id,
            title: self.title,
            description: self.description,
            thumbnail_url: self.thumbnail_url,
            video_count: self.video_count.map(|count| count as i32),
            uploader_id: uploader_id.to_string(),
        }
    }
}

/// Public (API) view of a read-only playlist (e.g. from YouTube).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Eq, PartialEq)]
pub struct ExtendedPublicPlaylist {
    pub playlist: ExtendedPlaylist,
    pub uploader: Channel,
}
impl ExtendedPublicPlaylist {
    pub fn from_public_playlist(playlist: &PublicPlaylist, channel: &Channel) -> Self {
        ExtendedPublicPlaylist {
            playlist: ExtendedPlaylist {
                id: playlist.id.clone(),
                title: playlist.title.clone(),
                description: playlist.description.clone(),
                thumbnail_url: playlist.thumbnail_url.clone(),
                video_count: playlist.video_count.map(|c| c as u64),
            },
            uploader: channel.clone(),
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
impl From<&CreateVideo> for Video {
    fn from(val: &CreateVideo) -> Self {
        Video {
            id: val.id.clone(),
            title: val.title.clone(),
            upload_date: val.upload_date,
            uploader_id: val.uploader.id.clone(),
            thumbnail_url: val.thumbnail_url.clone(),
            duration: val.duration,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExtendedWatchHistoryItem {
    pub video: CreateVideo,
    pub metadata: WatchHistoryItem,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExtendedSubscriptionGroup {
    pub group: SubscriptionGroup,
    pub channels: Vec<Channel>,
}

/// Claims to store inside the JWT Token
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// User ID.
    pub sub: String,
    pub exp: usize,
}
