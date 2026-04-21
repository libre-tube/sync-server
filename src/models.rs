use serde::{Deserialize, Serialize};

use super::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = user)]
pub struct User {
    pub id: String,
    pub name_hash: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NewUser {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = channel)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub avatar: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(primary_key(user_id, channel_id))]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Channel))]
#[diesel(table_name = subscription)]
pub struct Subscription {
    pub user_id: String,
    pub channel_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(belongs_to(User))]
#[diesel(table_name = playlist)]
pub struct Playlist {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub description: String,
    pub thumbnail_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(belongs_to(Channel, foreign_key = uploader))]
#[diesel(table_name = playlist_video)]
pub struct PlaylistVideo {
    pub id: String,
    pub title: String,
    pub upload_date: String,
    pub uploader: String,
    pub thumbnail_url: String,
    pub duration: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(primary_key(playlist_id, video_id))]
#[diesel(belongs_to(Playlist))]
#[diesel(belongs_to(PlaylistVideo, foreign_key = video_id))]
#[diesel(table_name = playlist_video_member)]
pub struct PlaylistVideoMember {
    pub playlist_id: String,
    pub video_id: String,
}
