// @generated automatically by Diesel CLI.

diesel::table! {
    account (id) {
        id -> Text,
        name_hash -> Text,
        password_hash -> Text,
    }
}

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
        account_id -> Text,
        title -> Text,
        description -> Text,
        thumbnail_url -> Nullable<Text>,
    }
}

diesel::table! {
    playlist_video_member (playlist_id, video_id) {
        playlist_id -> Text,
        video_id -> Text,
    }
}

diesel::table! {
    subscription (account_id, channel_id) {
        account_id -> Text,
        channel_id -> Text,
    }
}

diesel::table! {
    video (id) {
        id -> Text,
        title -> Text,
        upload_date -> BigInt,
        uploader_id -> Text,
        thumbnail_url -> Text,
        duration -> Integer,
    }
}

diesel::joinable!(playlist -> account (account_id));
diesel::joinable!(playlist_video_member -> playlist (playlist_id));
diesel::joinable!(playlist_video_member -> video (video_id));
diesel::joinable!(subscription -> account (account_id));
diesel::joinable!(subscription -> channel (channel_id));
diesel::joinable!(video -> channel (uploader_id));

diesel::allow_tables_to_appear_in_same_query!(
    account,
    channel,
    playlist,
    playlist_video_member,
    subscription,
    video,
);
