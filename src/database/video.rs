use diesel_async::RunQueryDsl as _;

use crate::{DbConnection, database::DbError, models::Video, schema::video::dsl::*};

pub async fn create_or_update_video(
    conn: &mut DbConnection,
    video_: &Video,
) -> Result<(), DbError> {
    diesel::insert_into(video)
        .values(video_)
        .on_conflict(id)
        .do_update()
        .set(video_)
        .execute(conn)
        .await?;

    Ok(())
}
