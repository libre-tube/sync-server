use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;

use crate::{
    DbConnection,
    database::DbError,
    models::{self, PlaylistBookmark},
    schema::{channel, playlist_bookmark::dsl::*, public_playlist::dsl::*},
};

pub async fn get_playlist_bookmark_by_id(
    conn: &mut DbConnection,
    playlist_id_: &str,
    account_id_: &str,
) -> Result<Option<(models::PublicPlaylist, models::Channel)>, DbError> {
    let item = playlist_bookmark
        .filter(
            public_playlist_id
                .eq(playlist_id_.to_string())
                .and(account_id.eq(account_id_.to_string())),
        )
        .inner_join(public_playlist.inner_join(channel::table))
        .select((
            models::PublicPlaylist::as_select(),
            models::Channel::as_select(),
        ))
        .first::<(models::PublicPlaylist, models::Channel)>(conn)
        .await
        .optional()?;

    Ok(item)
}

/// This assumes that the public playlist already exists!
pub async fn create_playlist_bookmark_by_playlist_id(
    conn: &mut DbConnection,
    playlist_id_: &str,
    account_id_: &str,
) -> Result<(), DbError> {
    let bookmark = PlaylistBookmark {
        account_id: account_id_.to_string(),
        public_playlist_id: playlist_id_.to_string(),
    };
    diesel::insert_into(playlist_bookmark)
        .values(bookmark)
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn get_playlist_bookmarks_by_account_id(
    conn: &mut DbConnection,
    account_id_: &str,
) -> Result<Vec<(models::PublicPlaylist, models::Channel)>, DbError> {
    playlist_bookmark
        .filter(account_id.eq(account_id_.to_string()))
        .inner_join(public_playlist.inner_join(channel::table))
        .select((
            models::PublicPlaylist::as_select(),
            models::Channel::as_select(),
        ))
        .load::<(models::PublicPlaylist, models::Channel)>(conn)
        .await
}

pub async fn delete_playlist_bookmark_by_playlist_id(
    conn: &mut DbConnection,
    playlist_id_: &str,
    account_id_: &str,
) -> Result<(), DbError> {
    diesel::delete(
        playlist_bookmark.filter(
            public_playlist_id
                .eq(playlist_id_.to_string())
                .and(account_id.eq(account_id_.to_string())),
        ),
    )
    .execute(conn)
    .await?;

    Ok(())
}
