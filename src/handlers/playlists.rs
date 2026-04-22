use actix_web::{
    HttpRequest, HttpResponse, Responder, delete, error, get, middleware::from_fn, patch, post, web,
};
use utoipa_actix_web::scope;

use crate::{
    WebData,
    database::{
        channel::create_or_update_channel,
        playlist::{
            add_video_to_playlist, create_new_playlist, delete_playlist_by_id, get_playlist_by_id,
            get_playlist_by_id_with_videos, get_playlists_by_user_id, remove_video_from_playlist,
            update_existing_playlist,
        },
    },
    dto::{CreatePlaylist, CreateVideo, PlaylistResponse},
    get_db_conn,
    handlers::{ScopedHandler, get_user, user::auth_middleware},
    models::{Playlist, Video},
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
    let user_id = get_user(&req).id;

    let (playlist, videos) = get_playlist_by_id_with_videos(&mut conn, &playlist_id)
        .await
        .map_err(error::ErrorInternalServerError)?;
    if playlist.user_id != user_id {
        return Err(error::ErrorForbidden("not the owner of the playlist"));
    }

    let videos = videos
        .iter()
        .map(|(video, channel)| CreateVideo::from((video, channel)))
        .collect();

    let playlist_response = PlaylistResponse { playlist, videos };
    Ok(HttpResponse::Ok().json(playlist_response))
}

#[utoipa::path(responses((status = OK, body = Vec<Playlist>)))]
#[get("/")]
async fn get_playlists(req: HttpRequest, pool: WebData) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);
    let user_id = get_user(&req).id;

    match get_playlists_by_user_id(&mut conn, &user_id).await {
        Ok(playlists) => Ok(HttpResponse::Ok().json(playlists)),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[utoipa::path(responses((status = CREATED, body = Playlist)))]
#[post("/")]
async fn create_playlist(
    req: HttpRequest,
    pool: WebData,
    playlist_data: web::Json<CreatePlaylist>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);
    let user = get_user(&req);

    let playlist = Playlist {
        id: String::new(),
        user_id: user.id.clone(),
        title: playlist_data.title.clone(),
        description: playlist_data.description.clone(),
        thumbnail_url: playlist_data.thumbnail_url.clone(),
    };

    match create_new_playlist(&mut conn, &playlist).await {
        Ok(playlist) => Ok(HttpResponse::Created().json(playlist)),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
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
    let user = get_user(&req);

    let playlist = get_playlist_by_id(&mut conn, &playlist_id)
        .await
        .map_err(error::ErrorInternalServerError)?;
    if playlist.user_id != user.id {
        return Err(error::ErrorForbidden("not the owner of the playlist"));
    }

    let playlist = Playlist {
        id: playlist_id.clone(),
        user_id: user.id.clone(),
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
    let user = get_user(&req);

    let playlist = get_playlist_by_id(&mut conn, &playlist_id)
        .await
        .map_err(error::ErrorInternalServerError)?;
    if playlist.user_id != user.id {
        return Err(error::ErrorForbidden("not the owner of the playlist"));
    }

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
    video_data: web::Json<CreateVideo>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);
    let user_id = get_user(&req).id;

    let playlist = get_playlist_by_id(&mut conn, &playlist_id)
        .await
        .map_err(error::ErrorInternalServerError)?;
    if playlist.user_id != user_id {
        return Err(error::ErrorForbidden("not the owner of the playlist"));
    }

    // store channel information first before storing video to ensure data integrity
    create_or_update_channel(&mut conn, &video_data.uploader)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let video = Video {
        id: video_data.id.clone(),
        title: video_data.title.clone(),
        upload_date: video_data.upload_date,
        uploader_id: video_data.uploader.id.clone(),
        thumbnail_url: video_data.thumbnail_url.clone(),
        duration: video_data.duration,
    };

    match add_video_to_playlist(&mut conn, &playlist_id, &video).await {
        Ok(()) => Ok(HttpResponse::Created()),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[utoipa::path(responses((status = OK)))]
#[delete("/{playlist_id}/videos/{video_id}")]
async fn remove_from_playlist(
    req: HttpRequest,
    pool: WebData,
    path: web::Path<(String, String)>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);
    let user_id = get_user(&req).id;

    let (playlist_id, video_id) = path.into_inner();

    let playlist = get_playlist_by_id(&mut conn, &playlist_id)
        .await
        .map_err(error::ErrorInternalServerError)?;
    if playlist.user_id != user_id {
        return Err(error::ErrorForbidden("not the owner of the playlist"));
    }

    match remove_video_from_playlist(&mut conn, &playlist_id, &video_id).await {
        Ok(()) => Ok(HttpResponse::Ok()),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}
