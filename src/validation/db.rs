use async_trait::async_trait;

use crate::{
    database::{
        DbPool, channel::get_channel_by_id, public_playlist::get_public_playlist_by_id,
        video::get_video_by_id,
    },
    dto::{CreateVideo, ExtendedPlaylist, ExtendedPublicPlaylist},
    get_db_conn,
    models::{Channel, Video},
    validation::{ValidationResult, Validator},
};

pub struct DbValidator(DbPool);

impl DbValidator {
    pub fn new(pool: DbPool) -> Self {
        Self(pool)
    }
}

#[async_trait]
impl Validator for DbValidator {
    async fn is_channel_information_valid(&self, channel: &Channel) -> ValidationResult<Channel> {
        let mut conn = get_db_conn!(self.0);

        if let Some(existing_channel) = get_channel_by_id(&mut conn, &channel.id)
            .await
            .ok()
            .flatten()
            && *channel == existing_channel
        {
            ValidationResult::Valid
        } else {
            ValidationResult::UnableToValidate
        }
    }

    async fn is_video_information_valid(
        &self,
        video_data: &CreateVideo,
    ) -> ValidationResult<CreateVideo> {
        let mut conn = get_db_conn!(self.0);

        let existing_video = get_video_by_id(&mut conn, &video_data.id)
            .await
            .ok()
            .flatten();

        if let Some(existing_video) = existing_video
            && std::convert::Into::<Video>::into(&*video_data) == existing_video
        {
            ValidationResult::Valid
        } else {
            ValidationResult::UnableToValidate
        }
    }

    async fn is_playlist_information_valid(
        &self,
        playlist: &ExtendedPublicPlaylist,
    ) -> ValidationResult<ExtendedPublicPlaylist> {
        let mut conn = get_db_conn!(self.0);

        if let Some(existing_playlist) = get_public_playlist_by_id(&mut conn, &playlist.playlist.id)
            .await
            .ok()
            .flatten()
            && playlist.playlist == ExtendedPlaylist::from_public_playlist(&existing_playlist)
        {
            ValidationResult::Valid
        } else {
            ValidationResult::UnableToValidate
        }
    }
}
