// @generated automatically by Diesel CLI.

diesel::table! {
    channel (id) {
        id -> Text,
        name -> Text,
        avatar -> Text,
        verified -> Bool,
    }
}

diesel::table! {
    playlist (id) {
        id -> Text,
        user_id -> Text,
        title -> Text,
        description -> Text,
        thumbnail_url -> Text,
    }
}

diesel::table! {
    playlist_video (id) {
        id -> Text,
        title -> Text,
        upload_date -> Text,
        uploader -> Text,
        thumbnail_url -> Text,
        duration -> Integer,
    }
}

diesel::table! {
    playlist_video_member (playlist_id, video_id) {
        playlist_id -> Text,
        video_id -> Text,
    }
}

diesel::table! {
    subscription (user_id, channel_id) {
        user_id -> Text,
        channel_id -> Text,
    }
}

diesel::table! {
    user (id) {
        id -> Text,
        name_hash -> Text,
        password_hash -> Text,
    }
}

diesel::joinable!(playlist -> user (user_id));
diesel::joinable!(playlist_video -> channel (uploader));
diesel::joinable!(playlist_video_member -> playlist (playlist_id));
diesel::joinable!(playlist_video_member -> playlist_video (video_id));
diesel::joinable!(subscription -> channel (channel_id));
diesel::joinable!(subscription -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    channel,
    playlist,
    playlist_video,
    playlist_video_member,
    subscription,
    user,
);
