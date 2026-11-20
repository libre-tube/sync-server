use diesel_async::pooled_connection::bb8::Pool;

pub mod account;
pub mod channel;
pub mod playlist;
pub mod playlist_bookmark;
pub mod public_playlist;
pub mod subscription;
pub mod subscription_groups;
pub mod video;
pub mod watch_history;

#[cfg(feature = "sqlite")]
pub type DbConnection =
    diesel_async::sync_connection_wrapper::SyncConnectionWrapper<diesel::SqliteConnection>;
#[cfg(feature = "postgres")]
pub type DbConnection = diesel_async::AsyncPgConnection;
pub type DbPool = Pool<DbConnection>;

type DbError = diesel::result::Error;
