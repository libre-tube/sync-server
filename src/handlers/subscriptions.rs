use actix_web::{
    HttpResponse, Responder, delete, error, get, middleware::from_fn, patch, post, put, web,
};
use utoipa_actix_web::scope;

use crate::{
    DbConnection, WebData,
    database::{
        channel::create_or_update_channel,
        subscription::{
            add_subscription_by_account_id, get_subscription_channel_by_account_id,
            get_subscriptions_by_account_id, remove_subscription_by_account_id,
        },
        subscription_groups::{
            add_channel_to_subscription_group, create_new_subscription_group,
            delete_subscription_group_by_id, get_subscription_group_by_id,
            get_subscription_group_channels_by_id, get_subscription_groups_by_account_id,
            remove_channel_from_all_subscription_groups, remove_channel_from_subscription_group,
            update_existing_subscription_group,
        },
    },
    dto::ExtendedSubscriptionGroup,
    get_db_conn,
    handlers::{ScopedHandler, user::auth_middleware},
    models::{Account, Channel, SubscriptionGroup},
    validation::validate_channel_information_if_changed,
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
            .service(
                scope::scope("/groups")
                    .service(get_subscription_groups)
                    .service(get_subscription_group)
                    .service(create_subscription_group)
                    .service(update_subscription_group)
                    .service(delete_subscription_group)
                    .service(add_to_subscription_group)
                    .service(remove_from_subscription_group),
            )
            .service(get_subscriptions)
            .service(get_subscription)
            .service(subscribe)
            .service(unsubscribe)
    }
}

#[utoipa::path(responses((status = OK, body = Vec<Channel>)), security(("api_jwt_token" = [])))]
#[get("/")]
async fn get_subscriptions(account: Account, pool: WebData) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let subscriptions = get_subscriptions_by_account_id(&mut conn, &account.id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(subscriptions))
}

#[utoipa::path(responses((status = OK, body = Channel)), security(("api_jwt_token" = [])))]
#[get("/{channel_id}")]
async fn get_subscription(
    account: Account,
    pool: WebData,
    channel_id: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    match get_subscription_channel_by_account_id(&mut conn, &account.id, &channel_id).await {
        Ok(channel) => match channel {
            Some(channel) => Ok(HttpResponse::Ok().json(channel)),
            None => Err(error::ErrorNotFound("not subscribed to this channel")),
        },
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[utoipa::path(responses((status = CREATED)), security(("api_jwt_token" = [])))]
#[put("/")]
async fn subscribe(
    account: Account,
    pool: WebData,
    channel: web::Json<Channel>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    // verify that the provided information is valid
    validate_channel_information_if_changed(&mut conn, &channel).await?;

    match add_subscription_by_account_id(&mut conn, &channel, &account.id).await {
        Ok(_) => Ok(HttpResponse::Ok()),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[utoipa::path(responses((status = OK)), security(("api_jwt_token" = [])))]
#[delete("/{channel_id}")]
async fn unsubscribe(
    account: Account,
    pool: WebData,
    channel_id: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    remove_subscription_by_account_id(&mut conn, &channel_id, &account.id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    // now that the user no longer subscribed to the channel, the channel may also no
    // longer be part of any subscription groups, so we auto-wipe it from all groups
    remove_channel_from_all_subscription_groups(&mut conn, &channel_id, &account.id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok())
}

/* Routes under the /groups prefix */
#[utoipa::path(responses((status = OK, body = Vec<ExtendedSubscriptionGroup>)), security(("api_jwt_token" = [])))]
#[get("/")]
async fn get_subscription_groups(
    account: Account,
    pool: WebData,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    match get_subscription_groups_by_account_id(&mut conn, &account.id).await {
        Ok(groups) => {
            let groups: Vec<_> = groups
                .iter()
                .map(|(group, channels)| ExtendedSubscriptionGroup {
                    group: group.clone(),
                    channels: channels.clone(),
                })
                .collect();

            Ok(HttpResponse::Ok().json(groups))
        }
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[utoipa::path(responses((status = OK, body = ExtendedSubscriptionGroup)), security(("api_jwt_token" = [])))]
#[get("/{subscription_group_id}")]
async fn get_subscription_group(
    account: Account,
    pool: WebData,
    subscription_group_id: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let Some(group) = get_subscription_group_by_id(&mut conn, &subscription_group_id, &account.id)
        .await
        .map_err(error::ErrorInternalServerError)?
    else {
        return Err(error::ErrorNotFound(
            "subscription group doesn't exist or doesn't belong to this account",
        ));
    };

    let channels = get_subscription_group_channels_by_id(&mut conn, &subscription_group_id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let extended_subscription_group = ExtendedSubscriptionGroup { group, channels };
    Ok(HttpResponse::Ok().json(extended_subscription_group))
}

#[utoipa::path(responses((status = CREATED, body = ExtendedSubscriptionGroup)), security(("api_jwt_token" = [])))]
#[post("/")]
async fn create_subscription_group(
    account: Account,
    pool: WebData,
    subscription_group: web::Json<SubscriptionGroup>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let mut subscription_group = subscription_group.into_inner();
    subscription_group.id = uuid::Uuid::now_v7().to_string();
    subscription_group.account_id = account.id;

    match create_new_subscription_group(&mut conn, subscription_group).await {
        Ok(group) => Ok(HttpResponse::Ok().json(group)),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[utoipa::path(responses((status = OK, body = SubscriptionGroup)), security(("api_jwt_token" = [])))]
#[patch("/{subscription_group_id}")]
async fn update_subscription_group(
    account: Account,
    pool: WebData,
    subscription_group_id: web::Path<String>,
    subscription_group: web::Json<SubscriptionGroup>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let mut subscription_group = subscription_group.into_inner();
    subscription_group.id = subscription_group_id.into_inner();
    subscription_group.account_id = account.id;

    match update_existing_subscription_group(&mut conn, subscription_group).await {
        Ok(group) => Ok(HttpResponse::Ok().json(group)),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[utoipa::path(responses((status = OK)), security(("api_jwt_token" = [])))]
#[delete("/{subscription_group_id}")]
async fn delete_subscription_group(
    account: Account,
    pool: WebData,
    subscription_group_id: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    match delete_subscription_group_by_id(&mut conn, &subscription_group_id, &account.id).await {
        Ok(_) => Ok(HttpResponse::Ok()),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

async fn verify_is_subscription_group_owner(
    conn: &mut DbConnection,
    subscription_group_id: &str,
    account_id: &str,
) -> actix_web::Result<()> {
    if get_subscription_group_by_id(conn, subscription_group_id, account_id)
        .await
        .map_err(error::ErrorInternalServerError)?
        .is_none()
    {
        return Err(error::ErrorNotFound(
            "either the subscription group doesn't exist or you don't own it",
        ));
    }

    Ok(())
}

#[utoipa::path(responses((status = OK)), security(("api_jwt_token" = [])))]
#[put("/{subscription_group_id}/channels")]
async fn add_to_subscription_group(
    account: Account,
    pool: WebData,
    subscription_group_id: web::Path<String>,
    channel: web::Json<Channel>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    verify_is_subscription_group_owner(&mut conn, &subscription_group_id, &account.id).await?;

    let subscription = get_subscription_channel_by_account_id(&mut conn, &account.id, &channel.id)
        .await
        .map_err(error::ErrorInternalServerError)?;
    if subscription.is_none() {
        return Err(error::ErrorBadRequest(
            "channel has to be subscribed to before it can be added to a channel group",
        ));
    }

    // we don't have to update the channel information in the database because we can assume that it's already
    // up to date, given that the user already subscribed to that channel

    add_channel_to_subscription_group(&mut conn, &subscription_group_id, &channel.id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok())
}

#[utoipa::path(responses((status = OK)), security(("api_jwt_token" = [])))]
#[delete("/{subscription_group_id}/channels/{channel_id}")]
async fn remove_from_subscription_group(
    account: Account,
    pool: WebData,
    path: web::Path<(String, String)>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);
    let (subscription_group_id, channel_id) = path.into_inner();

    verify_is_subscription_group_owner(&mut conn, &subscription_group_id, &account.id).await?;

    match remove_channel_from_subscription_group(&mut conn, &subscription_group_id, &channel_id)
        .await
    {
        Ok(_) => Ok(HttpResponse::Ok()),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}
