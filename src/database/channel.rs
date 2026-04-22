use diesel_async::RunQueryDsl;

use crate::{DbConnection, database::DbError, models::Channel, schema::channel::dsl::*};

pub async fn create_or_update_channel(
    conn: &mut DbConnection,
    channel_: &Channel,
) -> Result<(), DbError> {
    diesel::insert_into(channel)
        .values(channel_)
        .on_conflict(id)
        .do_update()
        .set(channel_)
        .execute(conn)
        .await?;

    Ok(())
}
