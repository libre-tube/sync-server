use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::schema::*;

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset, ToSchema,
)]
#[diesel(table_name = user)]
pub struct User {
    pub id: String,
    pub name_hash: String,
    pub password_hash: String,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset, ToSchema,
)]
#[diesel(table_name = channel)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub avatar: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, ToSchema)]
#[diesel(primary_key(user_id, channel_id))]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Channel))]
#[diesel(table_name = subscription)]
pub struct Subscription {
    pub user_id: String,
    pub channel_id: String,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset, ToSchema,
)]
#[diesel(belongs_to(User))]
#[diesel(table_name = playlist)]
pub struct Playlist {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub description: String,
    pub thumbnail_url: Option<String>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset, ToSchema,
)]
#[diesel(belongs_to(Channel, foreign_key = uploader_id))]
#[diesel(table_name = video)]
pub struct Video {
    pub id: String,
    pub title: String,
    pub upload_date: i64,
    /// ID of the uploader.
    pub uploader_id: String,
    pub thumbnail_url: String,
    /// Duration in seconds.
    pub duration: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, ToSchema)]
#[diesel(primary_key(playlist_id, video_id))]
#[diesel(belongs_to(Playlist))]
#[diesel(belongs_to(Video))]
#[diesel(table_name = playlist_video_member)]
pub struct PlaylistVideoMember {
    pub playlist_id: String,
    pub video_id: String,
}
