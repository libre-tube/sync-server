use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use itertools::Itertools;

use crate::{
    DbConnection,
    database::DbError,
    models::{Channel, SubscriptionGroup, SubscriptionGroupMember},
    schema::{channel, subscription_group::dsl::*, subscription_group_member::dsl::*},
};

pub async fn get_subscription_groups(
    conn: &mut DbConnection,
    account_id_: &str,
) -> Result<Vec<(SubscriptionGroup, Vec<Channel>)>, DbError> {
    let results = subscription_group
        .inner_join(subscription_group_member.inner_join(channel::table))
        .filter(account_id.eq(account_id_))
        .select((SubscriptionGroup::as_select(), Channel::as_select()))
        .order_by(subscription_group_id)
        .load(conn)
        .await?;

    let chunked = results.iter().chunk_by(|(group, _)| group);

    let grouped = chunked.into_iter().map(|(group, subscribed)| {
        (
            group.clone(),
            subscribed
                .map(|(_group, channel)| channel.clone())
                .collect(),
        )
    });

    Ok(grouped.collect())
}

pub async fn get_channels_in_group(
    conn: &mut DbConnection,
    subscription_group_id_: &str,
) -> Result<Vec<Channel>, DbError> {
    subscription_group_member
        .filter(subscription_group_id.eq(subscription_group_id_))
        .inner_join(channel::table)
        .select(Channel::as_select())
        .load(conn)
        .await
}

pub async fn create_new_subscription_group(
    conn: &mut DbConnection,
    subscription_group_: SubscriptionGroup,
) -> Result<SubscriptionGroup, DbError> {
    diesel::insert_into(subscription_group)
        .values(subscription_group_)
        .returning(SubscriptionGroup::as_returning())
        .get_result(conn)
        .await
}

pub async fn update_existing_subscription_group(
    conn: &mut DbConnection,
    subscription_group_: SubscriptionGroup,
) -> Result<SubscriptionGroup, DbError> {
    diesel::update(subscription_group)
        .filter(id.eq(subscription_group_.id.clone()))
        .set(subscription_group_)
        .returning(SubscriptionGroup::as_returning())
        .get_result(conn)
        .await
}

pub async fn delete_subscription_group_by_id(
    conn: &mut DbConnection,
    subscription_group_id_: &str,
) -> Result<(), DbError> {
    diesel::delete(subscription_group.filter(id.eq(subscription_group_id_)))
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn add_channel_to_subscription_group(
    conn: &mut DbConnection,
    subscription_group_id_: &str,
    channel_id_: &str,
) -> Result<(), DbError> {
    let subscription_group_member_ = SubscriptionGroupMember {
        subscription_group_id: subscription_group_id_.to_string(),
        channel_id: channel_id_.to_string(),
    };

    diesel::insert_into(subscription_group_member)
        .values(subscription_group_member_)
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn remove_from_subscription_group(
    conn: &mut DbConnection,
    subscription_group_id_: &str,
    channel_id_: &str,
) -> Result<(), DbError> {
    diesel::delete(subscription_group_member)
        .filter(
            subscription_group_id
                .eq(subscription_group_id_)
                .and(channel_id.eq(channel_id_)),
        )
        .execute(conn)
        .await?;

    Ok(())
}
