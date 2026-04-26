pub mod account;
pub mod channel;
pub mod playlist;
pub mod playlist_bookmark;
pub mod public_playlist;
pub mod subscription;
pub mod video;

type DbError = diesel::result::Error;
