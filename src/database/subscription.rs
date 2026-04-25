use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper,
    associations::HasTable,
};
use diesel_async::RunQueryDsl;

use crate::{
    DbConnection,
    database::{DbError, channel::create_or_update_channel},
    models::{self, Channel, Subscription},
    schema::{channel::dsl::channel, subscription::dsl::*},
};

pub async fn get_subscriptions_by_account_id(
    conn: &mut DbConnection,
    account_id_: &str,
) -> Result<Vec<models::Channel>, DbError> {
    subscription
        .filter(account_id.eq(account_id_.to_string()))
        .inner_join(channel::table())
        .select(models::Channel::as_select())
        .load::<models::Channel>(conn)
        .await
}

/// Get the [Channel], if the user subscribed to it, otherwise [None].
pub async fn get_subscription_channel_by_account_id(
    conn: &mut DbConnection,
    account_id_: &str,
    channel_id_: &str,
) -> Result<Option<models::Channel>, DbError> {
    subscription
        .filter(
            account_id
                .eq(account_id_.to_string())
                .and(channel_id.eq(channel_id_.to_string())),
        )
        .inner_join(channel::table())
        .select(models::Channel::as_select())
        .first::<models::Channel>(conn)
        .await
        .optional()
}

pub async fn add_subscription_by_account_id(
    conn: &mut DbConnection,
    channel_: &Channel,
    account_id_: &str,
) -> Result<(), DbError> {
    create_or_update_channel(conn, channel_).await?;

    let new_subscription = Subscription {
        account_id: account_id_.to_string(),
        channel_id: channel_.id.clone(),
    };
    diesel::insert_into(subscription)
        .values(&new_subscription)
        .on_conflict_do_nothing()
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn remove_subscription_by_account_id(
    conn: &mut DbConnection,
    channel_id_: &str,
    account_id_: &str,
) -> Result<(), DbError> {
    diesel::delete(
        subscription.filter(
            account_id
                .eq(account_id_.to_string())
                .and(channel_id.eq(channel_id_.to_string())),
        ),
    )
    .execute(conn)
    .await?;

    Ok(())
}
