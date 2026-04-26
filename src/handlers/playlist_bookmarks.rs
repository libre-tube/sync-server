use actix_web::{HttpResponse, Responder, delete, error, get, middleware::from_fn, post, web};
use utoipa_actix_web::scope;

use crate::{
    WebData,
    database::{
        channel::create_or_update_channel,
        playlist_bookmark::{
            create_playlist_bookmark_by_playlist_id, delete_playlist_bookmark_by_playlist_id,
            get_playlist_bookmark_by_id, get_playlist_bookmarks_by_account_id,
        },
        public_playlist::create_or_update_public_playlist,
    },
    dto::ExtendedPublicPlaylist,
    get_db_conn,
    handlers::{ScopedHandler, user::auth_middleware},
    models::Account,
    validation::validate_public_playlist_information_if_changed,
};

pub struct PlaylistBookmarksHandler {}
impl ScopedHandler for PlaylistBookmarksHandler {
    fn get_service() -> utoipa_actix_web::scope::Scope<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
            Config = (),
            InitError = (),
            Error = actix_web::Error,
        >,
    > {
        scope("/playlist_bookmarks")
            .wrap(from_fn(auth_middleware))
            .service(get_playlist_bookmarks)
            .service(get_playlist_bookmark)
            .service(create_playlist_bookmark)
            .service(delete_playlist_bookmark)
    }
}

#[utoipa::path(responses((status = OK, body = Vec<ExtendedPublicPlaylist>)), security(("api_jwt_token" = [])))]
#[get("/")]
async fn get_playlist_bookmarks(
    account: Account,
    pool: WebData,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let playlists = get_playlist_bookmarks_by_account_id(&mut conn, &account.id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let playlists: Vec<_> = playlists
        .iter()
        .map(|(playlist, channel)| ExtendedPublicPlaylist::from_public_playlist(playlist, channel))
        .collect();

    Ok(HttpResponse::Ok().json(playlists))
}

#[utoipa::path(responses((status = OK, body = ExtendedPublicPlaylist)), security(("api_jwt_token" = [])))]
#[get("/{public_playlist_id}")]
async fn get_playlist_bookmark(
    account: Account,
    pool: WebData,
    playlist_id: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let Some((playlist, channel)) =
        get_playlist_bookmark_by_id(&mut conn, &playlist_id, &account.id)
            .await
            .map_err(error::ErrorInternalServerError)?
    else {
        return Err(error::ErrorNotFound("bookmark doesn't exist"));
    };

    let extended_playlist = ExtendedPublicPlaylist::from_public_playlist(&playlist, &channel);
    Ok(HttpResponse::Ok().json(extended_playlist))
}

#[utoipa::path(responses((status = CREATED, body = ExtendedPublicPlaylist)), security(("api_jwt_token" = [])))]
#[post("/")]
async fn create_playlist_bookmark(
    account: Account,
    pool: WebData,
    playlist: web::Json<ExtendedPublicPlaylist>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let playlist =
        validate_public_playlist_information_if_changed(&mut conn, playlist.into_inner()).await?;

    create_or_update_channel(&mut conn, &playlist.uploader)
        .await
        .map_err(error::ErrorInternalServerError)?;

    create_or_update_public_playlist(
        &mut conn,
        &playlist
            .playlist
            .clone()
            .into_public_playlist(&playlist.uploader.id),
    )
    .await
    .map_err(error::ErrorInternalServerError)?;
    create_playlist_bookmark_by_playlist_id(&mut conn, &playlist.playlist.id, &account.id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(playlist))
}

#[utoipa::path(responses((status = OK)), security(("api_jwt_token" = [])))]
#[delete("/{public_playlist_id}")]
async fn delete_playlist_bookmark(
    account: Account,
    pool: WebData,
    playlist_id: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    delete_playlist_bookmark_by_playlist_id(&mut conn, &playlist_id, &account.id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok())
}
