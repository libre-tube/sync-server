use actix_web::{
    HttpRequest, HttpResponse, Responder, delete, error, get, middleware::from_fn, patch, post, web,
};
use itertools::Itertools;
use utoipa_actix_web::scope;

use crate::{
    DbConnection, WebData,
    database::{
        channel::create_or_update_channel,
        playlist::{
            add_video_to_playlist, create_new_playlist, delete_playlist_by_id, get_playlist_by_id,
            get_playlist_by_id_with_videos, get_playlist_video_count, get_playlists_by_account_id,
            remove_video_from_playlist, update_existing_playlist,
        },
    },
    dto::{CreatePlaylist, CreateVideo, ExtendedPlaylist, PlaylistResponse},
    get_db_conn,
    handlers::{ScopedHandler, get_account, user::auth_middleware},
    models::Playlist,
    validation::validate_video_information_if_changed,
};

pub struct PlaylistsHandler {}
impl ScopedHandler for PlaylistsHandler {
    fn get_service() -> utoipa_actix_web::scope::Scope<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
            Config = (),
            InitError = (),
            Error = actix_web::Error,
        >,
    > {
        scope("/playlists")
            .wrap(from_fn(auth_middleware))
            .service(get_playlists)
            .service(get_playlist)
            .service(create_playlist)
            .service(update_playlist)
            .service(delete_playlist)
            .service(add_to_playlist)
            .service(remove_from_playlist)
    }
}

#[utoipa::path(responses((status = OK, body = PlaylistResponse)))]
#[get("/{playlist_id}")]
async fn get_playlist(
    pool: WebData,
    req: HttpRequest,
    playlist_id: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);
    let account_id = get_account(&req).id;

    let Some((playlist, videos)) = get_playlist_by_id_with_videos(&mut conn, &playlist_id)
        .await
        .map_err(error::ErrorInternalServerError)?
    else {
        return Err(error::ErrorNotFound("playlist does not exist"));
    };
    if playlist.account_id != account_id {
        return Err(error::ErrorForbidden("not the owner of the playlist"));
    }

    let videos: Vec<_> = videos
        .iter()
        .map(|(video, channel)| CreateVideo::from((video, channel)))
        .collect();

    let playlist = ExtendedPlaylist::from_playlist(&playlist, videos.len() as u64);

    let playlist_response = PlaylistResponse { playlist, videos };
    Ok(HttpResponse::Ok().json(playlist_response))
}

#[utoipa::path(responses((status = OK, body = Vec<ExtendedPlaylist>)))]
#[get("/")]
async fn get_playlists(req: HttpRequest, pool: WebData) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);
    let account_id = get_account(&req).id;

    let playlists = get_playlists_by_account_id(&mut conn, &account_id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let mut extended_playlists: Vec<ExtendedPlaylist> = vec![];
    for playlist in &playlists {
        let video_count = get_playlist_video_count(&mut conn, &playlist.id)
            .await
            .unwrap_or(-1);
        let extended_playlist = ExtendedPlaylist::from_playlist(playlist, video_count as u64);
        extended_playlists.push(extended_playlist);
    }

    Ok(HttpResponse::Ok().json(extended_playlists))
}

#[utoipa::path(responses((status = CREATED, body = Playlist)))]
#[post("/")]
async fn create_playlist(
    req: HttpRequest,
    pool: WebData,
    playlist_data: web::Json<CreatePlaylist>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);
    let account = get_account(&req);

    let playlist = Playlist {
        id: String::new(),
        account_id: account.id.clone(),
        title: playlist_data.title.clone(),
        description: playlist_data.description.clone(),
        thumbnail_url: playlist_data.thumbnail_url.clone(),
    };

    match create_new_playlist(&mut conn, &playlist).await {
        Ok(playlist) => Ok(HttpResponse::Created().json(playlist)),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

/// Get the playlist if it exists. Only succeeds if the user is the owner of the playlist, i.e. `account_id` matches.
async fn get_owned_playlist_or_error(
    conn: &mut DbConnection,
    playlist_id: &str,
    account_id: &str,
) -> actix_web::Result<Playlist> {
    let playlist = get_playlist_by_id(conn, playlist_id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let playlist = playlist.ok_or_else(|| error::ErrorNotFound("playlist doesn't exist"))?;
    if playlist.account_id != account_id {
        return Err(error::ErrorForbidden("not the owner of the playlist"));
    }

    Ok(playlist)
}

#[utoipa::path(responses((status = OK, body = Playlist)))]
#[patch("/{playlist_id}")]
async fn update_playlist(
    req: HttpRequest,
    pool: WebData,
    playlist_id: web::Path<String>,
    playlist_data: web::Json<CreatePlaylist>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);
    let account = get_account(&req);

    get_owned_playlist_or_error(&mut conn, &playlist_id, &account.id).await?;

    let playlist = Playlist {
        id: playlist_id.clone(),
        account_id: account.id.clone(),
        title: playlist_data.title.clone(),
        description: playlist_data.description.clone(),
        thumbnail_url: playlist_data.thumbnail_url.clone(),
    };

    match update_existing_playlist(&mut conn, &playlist).await {
        Ok(playlist) => Ok(HttpResponse::Ok().json(playlist)),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[utoipa::path(responses((status = OK)))]
#[delete("/{playlist_id}")]
async fn delete_playlist(
    req: HttpRequest,
    pool: WebData,
    playlist_id: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);
    let account = get_account(&req);

    get_owned_playlist_or_error(&mut conn, &playlist_id, &account.id).await?;

    match delete_playlist_by_id(&mut conn, &playlist_id).await {
        Ok(()) => Ok(HttpResponse::Ok().json(())),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[utoipa::path(responses((status = CREATED)))]
#[post("/{playlist_id}/videos")]
async fn add_to_playlist(
    req: HttpRequest,
    pool: WebData,
    playlist_id: web::Path<String>,
    video_datas: web::Json<Vec<CreateVideo>>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);
    let account_id = get_account(&req).id;

    get_owned_playlist_or_error(&mut conn, &playlist_id, &account_id).await?;

    let video_datas = video_datas.into_inner();
    let videos_grouped_by_uploader = video_datas
        .iter()
        .sorted_by(|a, b| Ord::cmp(&a.uploader.id, &b.uploader.id))
        .chunk_by(|video| video.uploader.clone());

    for (channel, videos) in &videos_grouped_by_uploader {
        let mut videos: Vec<_> = videos.cloned().collect();

        validate_video_information_if_changed(&mut conn, &mut videos).await?;

        // store channel information first before storing video to ensure data integrity
        create_or_update_channel(&mut conn, &channel)
            .await
            .map_err(error::ErrorInternalServerError)?;

        for video in videos {
            add_video_to_playlist(&mut conn, &playlist_id, &(&video).into())
                .await
                .map_err(error::ErrorInternalServerError)?;
        }
    }

    Ok(HttpResponse::Created())
}

#[utoipa::path(responses((status = OK)))]
#[delete("/{playlist_id}/videos/{video_id}")]
async fn remove_from_playlist(
    req: HttpRequest,
    pool: WebData,
    path: web::Path<(String, String)>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);
    let account_id = get_account(&req).id;

    let (playlist_id, video_id) = path.into_inner();

    get_owned_playlist_or_error(&mut conn, &playlist_id, &account_id).await?;

    match remove_video_from_playlist(&mut conn, &playlist_id, &video_id).await {
        Ok(()) => Ok(HttpResponse::Ok()),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}
