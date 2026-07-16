use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    DbConnection,
    database::{DbError, video::create_or_update_video},
    models::{Channel, Playlist, PlaylistVideoMember, Video},
    schema::{
        channel,
        playlist::dsl::{account_id as playlist_account_id, *},
        playlist_video_member::dsl::{account_id as playlist_video_member_account_id, *},
        video,
    },
};

pub async fn create_new_playlist(
    conn: &mut DbConnection,
    playlist_: &Playlist,
) -> Result<Playlist, DbError> {
    let mut playlist_ = playlist_.clone();

    if playlist_.id.is_empty() {
        playlist_.id = Uuid::now_v7().to_string();
    }

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
    account_id_: &str,
) -> Result<Playlist, DbError> {
    let updated_playlist = diesel::update(
        playlist.filter(
            id.eq(playlist_.id.clone())
                .and(playlist_account_id.eq(account_id_)),
        ),
    )
    .set(playlist_)
    .returning(Playlist::as_returning())
    .get_result(conn)
    .await?;

    Ok(updated_playlist)
}

pub async fn delete_playlist_by_id(
    conn: &mut DbConnection,
    playlist_id_: &str,
    account_id_: &str,
) -> Result<(), DbError> {
    // delete linked videos first to ensure database integrity
    // TODO: use ON DELETE CASCADE
    diesel::delete(
        playlist_video_member.filter(
            playlist_id
                .eq(playlist_id_.to_string())
                .and(playlist_video_member_account_id.eq(account_id_)),
        ),
    )
    .execute(conn)
    .await?;

    diesel::delete(playlist.filter(id.eq(playlist_id_.to_string())))
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn add_video_to_playlist(
    conn: &mut DbConnection,
    playlist_id_: &str,
    account_id_: &str,
    video_: &Video,
) -> Result<(), DbError> {
    create_or_update_video(conn, video_).await?;

    let new_playlist_video_member = PlaylistVideoMember {
        account_id: account_id_.to_string(),
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
    account_id_: &str,
    video_id_: &str,
) -> Result<(), DbError> {
    diesel::delete(
        playlist_video_member.filter(
            playlist_id
                .eq(playlist_id_)
                .and(playlist_video_member_account_id.eq(account_id_))
                .and(video_id.eq(video_id_)),
        ),
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub async fn get_playlist_by_id(
    conn: &mut DbConnection,
    playlist_id_: &str,
    account_id_: &str,
) -> Result<Option<Playlist>, DbError> {
    let mut playlist_: Option<Playlist> = playlist
        .filter(id.eq(playlist_id_).and(playlist_account_id.eq(account_id_)))
        .first(conn)
        .await
        .optional()?;

    if let Some(ref mut playlist_) = playlist_ {
        assign_thumbnail_if_missing(conn, playlist_).await?;
    }

    Ok(playlist_)
}

pub async fn get_playlist_by_id_with_videos(
    conn: &mut DbConnection,
    playlist_id_: &str,
    account_id_: &str,
) -> Result<Option<(Playlist, Vec<(Video, Channel)>)>, DbError> {
    let Some(playlist_) = get_playlist_by_id(conn, playlist_id_, account_id_).await? else {
        return Ok(None);
    };

    let videos = playlist_video_member
        .filter(
            playlist_id
                .eq(playlist_id_)
                .and(playlist_video_member_account_id.eq(account_id_)),
        )
        .inner_join(video::table.inner_join(channel::table))
        .select((Video::as_select(), Channel::as_select()))
        .load(conn)
        .await?;

    Ok(Some((playlist_, videos)))
}

pub async fn get_playlist_first_video(
    conn: &mut DbConnection,
    playlist_id_: &str,
    account_id_: &str,
) -> Result<Option<Video>, DbError> {
    playlist_video_member
        .filter(
            playlist_id
                .eq(playlist_id_)
                .and(playlist_video_member_account_id.eq(account_id_)),
        )
        .inner_join(video::table)
        .select(Video::as_select())
        .first(conn)
        .await
        .optional()
}

pub async fn assign_thumbnail_if_missing(
    conn: &mut DbConnection,
    playlist_: &mut Playlist,
) -> Result<(), DbError> {
    if playlist_.thumbnail_url.is_none()
        && let Some(first_video) =
            get_playlist_first_video(conn, &playlist_.id, &playlist_.account_id).await?
    {
        playlist_.thumbnail_url = Some(first_video.thumbnail_url);
    }

    Ok(())
}

pub async fn get_playlist_video_count(
    conn: &mut DbConnection,
    playlist_id_: &str,
    account_id_: &str,
) -> Result<i64, DbError> {
    playlist_video_member
        .filter(
            playlist_id
                .eq(playlist_id_)
                .and(playlist_video_member_account_id.eq(account_id_)),
        )
        .inner_join(video::table.inner_join(channel::table))
        .count()
        .get_result(conn)
        .await
}

pub async fn get_playlists_by_account_id(
    conn: &mut DbConnection,
    account_id_: &str,
) -> Result<Vec<Playlist>, DbError> {
    let mut playlists = playlist
        .filter(playlist_account_id.eq(account_id_.to_string()))
        .select(Playlist::as_select())
        .load(conn)
        .await?;

    for playlist_ in &mut playlists {
        assign_thumbnail_if_missing(conn, playlist_).await?;
    }

    Ok(playlists)
}
