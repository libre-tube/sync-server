use diesel_async::RunQueryDsl as _;

use crate::{DbConnection, database::DbError, models::Video, schema::video::dsl::*};

pub async fn create_or_update_video(
    conn: &mut DbConnection,
    video_: &Video,
) -> Result<(), DbError> {
    // TODO: update existing entries if the video already exists
    diesel::insert_or_ignore_into(video)
        .values(video_)
        .execute(conn)
        .await?;

    Ok(())
}
