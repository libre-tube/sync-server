use diesel::{
    ExpressionMethods, OptionalExtension, SelectableHelper,
    query_dsl::methods::{FilterDsl, SelectDsl},
};
use diesel_async::RunQueryDsl;

use crate::{
    DbConnection, database::DbError, models::PublicPlaylist, schema::public_playlist::dsl::*,
};

pub async fn create_or_update_public_playlist(
    conn: &mut DbConnection,
    playlist_: &PublicPlaylist,
) -> Result<PublicPlaylist, DbError> {
    diesel::insert_into(public_playlist)
        .values(playlist_)
        .on_conflict(id)
        .do_update()
        .set(playlist_)
        .returning(PublicPlaylist::as_returning())
        .get_result(conn)
        .await
}

pub async fn get_public_playlist_by_id(
    conn: &mut DbConnection,
    playlist_id_: &str,
) -> Result<Option<PublicPlaylist>, DbError> {
    public_playlist
        .filter(id.eq(playlist_id_.to_string()))
        .select(PublicPlaylist::as_select())
        .first(conn)
        .await
        .optional()
}
