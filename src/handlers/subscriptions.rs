use actix_web::{HttpRequest, HttpResponse, Responder, error, get, middleware::from_fn, post, web};
use utoipa_actix_web::scope;

use crate::{
    WebData,
    database::subscription::{
        add_subscription_by_account_id, get_subscriptions_by_account_id,
        remove_subscription_by_account_id,
    },
    dto::UnsubscribeChannel,
    get_db_conn,
    handlers::{ScopedHandler, get_account, user::auth_middleware},
    models::Channel,
};

pub struct SubscriptionsHandler {}
impl ScopedHandler for SubscriptionsHandler {
    fn get_service() -> utoipa_actix_web::scope::Scope<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
            Config = (),
            InitError = (),
            Error = actix_web::Error,
        >,
    > {
        scope("/subscriptions")
            .wrap(from_fn(auth_middleware))
            .service(get_subscriptions)
            .service(subscribe)
            .service(unsubscribe)
    }
}

#[utoipa::path(responses((status = OK, body = Vec<Channel>)))]
#[get("/")]
async fn get_subscriptions(req: HttpRequest, pool: WebData) -> actix_web::Result<impl Responder> {
    let account = get_account(&req);
    let mut conn = get_db_conn!(pool);

    let subscriptions = get_subscriptions_by_account_id(&mut conn, &account.id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(subscriptions))
}

#[utoipa::path(responses((status = CREATED)))]
#[post("/subscribe")]
async fn subscribe(
    req: HttpRequest,
    pool: WebData,
    channel: web::Json<Channel>,
) -> actix_web::Result<impl Responder> {
    let account = get_account(&req);
    let mut conn = get_db_conn!(pool);

    match add_subscription_by_account_id(&mut conn, &channel, &account.id).await {
        Ok(_) => Ok(HttpResponse::Ok()),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[utoipa::path(responses((status = CREATED)))]
#[post("/unsubscribe")]
async fn unsubscribe(
    req: HttpRequest,
    pool: WebData,
    channel: web::Json<UnsubscribeChannel>,
) -> actix_web::Result<impl Responder> {
    let account = get_account(&req);
    let mut conn = get_db_conn!(pool);

    match remove_subscription_by_account_id(&mut conn, &channel.channel_id, &account.id).await {
        Ok(_) => Ok(HttpResponse::Ok()),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}
