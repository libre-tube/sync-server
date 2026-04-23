use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    DbConnection,
    database::{DbError, video::create_or_update_video},
    models::{Channel, Playlist, PlaylistVideoMember, Video},
    schema::{channel, playlist::dsl::*, playlist_video_member::dsl::*, video},
};

pub async fn create_new_playlist(
    conn: &mut DbConnection,
    playlist_: &Playlist,
) -> Result<Playlist, DbError> {
    let mut playlist_ = playlist_.clone();
    playlist_.id = Uuid::now_v7().to_string();

    let created_playlist = diesel::insert_into(playlist)
        .values(playlist_)
        .on_conflict_do_nothing()
        .returning(Playlist::as_returning())
        .get_result(conn)
        .await?;

    Ok(created_playlist)
}

pub async fn update_existing_playlist(
    conn: &mut DbConnection,
    playlist_: &Playlist,
) -> Result<Playlist, DbError> {
    let updated_playlist = diesel::update(playlist.filter(id.eq(playlist_.id.to_string())))
        .set(playlist_)
        .returning(Playlist::as_returning())
        .get_result(conn)
        .await?;

    Ok(updated_playlist)
}

pub async fn delete_playlist_by_id(
    conn: &mut DbConnection,
    playlist_id_: &str,
) -> Result<(), DbError> {
    diesel::delete(playlist.filter(id.eq(playlist_id_.to_string())))
        .execute(conn)
        .await?;

    diesel::delete(playlist_video_member.filter(playlist_id.eq(playlist_id_.to_string())))
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn add_video_to_playlist(
    conn: &mut DbConnection,
    playlist_id_: &str,
    video_: &Video,
) -> Result<(), DbError> {
    create_or_update_video(conn, video_).await?;

    // TODO: support adding the same video to a playlist multiple times
    let new_playlist_video_member = PlaylistVideoMember {
        playlist_id: playlist_id_.to_string(),
        video_id: video_.id.clone(),
    };
    diesel::insert_into(playlist_video_member)
        .values(new_playlist_video_member)
        .on_conflict_do_nothing()
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn remove_video_from_playlist(
    conn: &mut DbConnection,
    playlist_id_: &str,
    video_id_: &str,
) -> Result<(), DbError> {
    diesel::delete(
        playlist_video_member.filter(
            playlist_id
                .eq(playlist_id_.to_string())
                .and(video_id.eq(video_id_.to_string())),
        ),
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub async fn get_playlist_by_id(
    conn: &mut DbConnection,
    playlist_id_: &str,
) -> Result<Playlist, DbError> {
    let playlist_ = playlist
        .filter(id.eq(playlist_id_.to_string()))
        .first(conn)
        .await?;

    Ok(playlist_)
}

pub async fn get_playlist_by_id_with_videos(
    conn: &mut DbConnection,
    playlist_id_: &str,
) -> Result<(Playlist, Vec<(Video, Channel)>), DbError> {
    let playlist_ = get_playlist_by_id(conn, playlist_id_).await?;

    let videos = playlist_video_member
        .filter(playlist_id.eq(playlist_id_.to_string()))
        .inner_join(video::table.inner_join(channel::table))
        .select((Video::as_select(), Channel::as_select()))
        .load(conn)
        .await?;

    Ok((playlist_, videos))
}

pub async fn get_playlists_by_account_id(
    conn: &mut DbConnection,
    account_id_: &str,
) -> Result<Vec<Playlist>, DbError> {
    let playlists = playlist
        .filter(account_id.eq(account_id_.to_string()))
        .select(Playlist::as_select())
        .load(conn)
        .await?;

    Ok(playlists)
}
