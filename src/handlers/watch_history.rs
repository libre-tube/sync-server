use actix_web::{HttpResponse, Responder, delete, get, middleware::from_fn, patch, put, web};
use serde::Deserialize;
use utoipa_actix_web::scope;

use crate::{
    WebData,
    database::{
        channel::create_or_update_channel,
        video::create_or_update_video,
        watch_history::{
            add_or_update_video_to_watch_history, clear_watch_history_by_account_id,
            get_watch_history_by_account_id, get_watch_history_entry,
            remove_video_from_watch_history,
        },
    },
    dto::{CreateVideo, ExtendedWatchHistoryItem, WatchedState},
    get_db_conn,
    handlers::{HandlerError, HandlerResult, ScopedHandler, user::auth_middleware},
    models::{Account, WatchHistoryItem},
    validation::validate_video_information_if_changed_single,
};

pub struct WatchHistoryHandler;
impl ScopedHandler for WatchHistoryHandler {
    fn get_service() -> utoipa_actix_web::scope::Scope<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
            Config = (),
            InitError = (),
            Error = actix_web::Error,
        >,
    > {
        scope::scope("/watch_history")
            .wrap(from_fn(auth_middleware))
            .service(get_watch_history)
            .service(get_from_watch_history)
            .service(add_to_watch_history)
            .service(update_watch_history_video_state)
            .service(remove_from_watch_history)
            .service(clear_watch_history)
    }
}

#[derive(Deserialize, Eq, PartialEq, PartialOrd, Ord)]
enum WatchHistoryOrder {
    #[serde(rename = "added_date_asc")]
    AddedDateAscending,
    #[serde(rename = "added_date_desc")]
    AddedDateDescending,
}
#[derive(Deserialize)]
struct WatchHistoryPaginationRequest {
    page: u32,
    state: Option<WatchedState>,
    order: Option<WatchHistoryOrder>,
}

#[utoipa::path(responses((status = OK, body = Vec<ExtendedWatchHistoryItem>)), params(("page" = u32, Query)), security(("api_jwt_token" = [])))]
#[get("/")]
async fn get_watch_history(
    account: Account,
    pool: WebData,
    params: web::Query<WatchHistoryPaginationRequest>,
) -> HandlerResult<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let watched_state = params
        .state
        .clone()
        .map(|s| serde_json::ser::to_string(&s).unwrap());
    dbg!(&watched_state);

    match get_watch_history_by_account_id(
        &mut conn,
        &account.id,
        params.page,
        &watched_state,
        params.order == Some(WatchHistoryOrder::AddedDateAscending),
    )
    .await
    {
        Ok(history) => {
            let history = history
                .iter()
                .map(|(metadata, video, channel)| ExtendedWatchHistoryItem {
                    video: CreateVideo::from((video, channel)),
                    metadata: metadata.clone(),
                })
                .collect::<Vec<_>>();
            Ok(HttpResponse::Ok().json(history))
        }
        Err(err) => Err(HandlerError::InternalDatabaseErrorWithContext(
            err.to_string(),
        )),
    }
}

#[utoipa::path(responses((status = OK, body = ExtendedWatchHistoryItem)), security(("api_jwt_token" = [])))]
#[get("/{video_id}")]
async fn get_from_watch_history(
    account: Account,
    pool: WebData,
    video_id: web::Path<String>,
) -> HandlerResult<impl Responder> {
    let mut conn = get_db_conn!(pool);

    match get_watch_history_entry(&mut conn, &account.id, &video_id)
        .await
        .map_err(|_| HandlerError::InternalDatabaseError)?
    {
        Some((metadata, video, channel)) => Ok(HttpResponse::Ok().json(ExtendedWatchHistoryItem {
            video: CreateVideo::from((&video, &channel)),
            metadata: metadata.clone(),
        })),
        None => Err(HandlerError::NotInWatchHistory),
    }
}

#[utoipa::path(responses((status = CREATED, body = ExtendedWatchHistoryItem)), security(("api_jwt_token" = [])))]
#[put("/")]
async fn add_to_watch_history(
    account: Account,
    pool: WebData,
    watch_history_item: web::Json<ExtendedWatchHistoryItem>,
) -> HandlerResult<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let mut watch_history_item = watch_history_item.into_inner();
    watch_history_item.metadata.account_id = account.id;
    watch_history_item.metadata.video_id = watch_history_item.video.id.clone();

    validate_video_information_if_changed_single(&mut conn, &mut watch_history_item.video).await?;

    // store video metadata in database
    create_or_update_channel(&mut conn, &watch_history_item.video.uploader)
        .await
        .map_err(|_| HandlerError::InternalDatabaseError)?;
    create_or_update_video(&mut conn, &(&watch_history_item.video).into())
        .await
        .map_err(|_| HandlerError::InternalDatabaseError)?;

    // create actual watch history entry
    add_or_update_video_to_watch_history(&mut conn, &watch_history_item.metadata)
        .await
        .map_err(|_| HandlerError::InternalDatabaseError)?;

    Ok(HttpResponse::Ok().json(watch_history_item))
}

#[utoipa::path(responses((status = OK)), security(("api_jwt_token" = [])))]
#[patch("/{video_id}")]
async fn update_watch_history_video_state(
    account: Account,
    pool: WebData,
    watch_history_item: web::Json<WatchHistoryItem>,
    video_id: web::Path<String>,
) -> HandlerResult<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let mut watch_history_item = watch_history_item.into_inner();
    watch_history_item.video_id = video_id.into_inner();
    watch_history_item.account_id = account.id;

    add_or_update_video_to_watch_history(&mut conn, &watch_history_item)
        .await
        .map_err(|_| HandlerError::InternalDatabaseError)?;

    Ok(HttpResponse::Ok().json(watch_history_item))
}

#[utoipa::path(responses((status = OK)), security(("api_jwt_token" = [])))]
#[delete("/")]
async fn clear_watch_history(account: Account, pool: WebData) -> HandlerResult<impl Responder> {
    let mut conn = get_db_conn!(pool);

    match clear_watch_history_by_account_id(&mut conn, &account.id).await {
        Ok(()) => Ok(HttpResponse::Ok()),
        Err(err) => Err(HandlerError::InternalDatabaseErrorWithContext(
            err.to_string(),
        )),
    }
}

#[utoipa::path(responses((status = OK)), security(("api_jwt_token" = [])))]
#[delete("/{video_id}")]
async fn remove_from_watch_history(
    account: Account,
    pool: WebData,
    video_id: web::Path<String>,
) -> HandlerResult<impl Responder> {
    let mut conn = get_db_conn!(pool);

    match remove_video_from_watch_history(&mut conn, &account.id, &video_id).await {
        Ok(()) => Ok(HttpResponse::Ok()),
        Err(err) => Err(HandlerError::InternalDatabaseErrorWithContext(
            err.to_string(),
        )),
    }
}
