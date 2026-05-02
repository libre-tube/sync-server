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
    playlist_bookmark (account_id, public_playlist_id) {
        account_id -> Text,
        public_playlist_id -> Text,
    }
}

diesel::table! {
    playlist_video_member (playlist_id, video_id) {
        playlist_id -> Text,
        video_id -> Text,
    }
}

diesel::table! {
    public_playlist (id) {
        id -> Text,
        title -> Text,
        description -> Text,
        thumbnail_url -> Nullable<Text>,
        uploader_id -> Text,
        video_count -> Nullable<Integer>,
    }
}

diesel::table! {
    subscription (account_id, channel_id) {
        account_id -> Text,
        channel_id -> Text,
    }
}

diesel::table! {
    subscription_group (id) {
        id -> Text,
        account_id -> Text,
        title -> Text,
    }
}

diesel::table! {
    subscription_group_member (subscription_group_id, channel_id) {
        subscription_group_id -> Text,
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

diesel::table! {
    watch_history (video_id, account_id) {
        video_id -> Text,
        account_id -> Text,
        added_date -> BigInt,
        watched_state -> crate::models::WatchedStateMapping,
        position_millis -> Nullable<Integer>,
    }
}

diesel::joinable!(playlist -> account (account_id));
diesel::joinable!(playlist_bookmark -> account (account_id));
diesel::joinable!(playlist_bookmark -> public_playlist (public_playlist_id));
diesel::joinable!(playlist_video_member -> playlist (playlist_id));
diesel::joinable!(playlist_video_member -> video (video_id));
diesel::joinable!(public_playlist -> channel (uploader_id));
diesel::joinable!(subscription -> account (account_id));
diesel::joinable!(subscription -> channel (channel_id));
diesel::joinable!(subscription_group -> account (account_id));
diesel::joinable!(subscription_group_member -> channel (channel_id));
diesel::joinable!(subscription_group_member -> subscription_group (subscription_group_id));
diesel::joinable!(video -> channel (uploader_id));
diesel::joinable!(watch_history -> account (account_id));
diesel::joinable!(watch_history -> video (video_id));

diesel::allow_tables_to_appear_in_same_query!(
    account,
    channel,
    playlist,
    playlist_bookmark,
    playlist_video_member,
    public_playlist,
    subscription,
    subscription_group,
    subscription_group_member,
    video,
    watch_history,
);
