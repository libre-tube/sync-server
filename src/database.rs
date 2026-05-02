pub mod account;
pub mod channel;
pub mod playlist;
pub mod playlist_bookmark;
pub mod public_playlist;
pub mod subscription;
pub mod subscription_groups;
pub mod video;
pub mod watch_history;

type DbError = diesel::result::Error;
