use crate::{
    dto::{CreateVideo, ExtendedPlaylist, ExtendedPublicPlaylist},
    models::Channel,
    validation::{ValidationError, ValidationResult, Validator},
};
use async_trait::async_trait;
use innertube::{Innertube, InnertubeError};

pub struct InnertubeValidator {
    innertube: &'static Innertube,
}

impl InnertubeValidator {
    pub fn new(innertube: &'static Innertube) -> Self {
        Self { innertube }
    }
}

macro_rules! unwrap {
    ($fn:expr) => {
        match $fn {
            Ok(resp) => resp,
            Err(err) => {
                return if matches!(err, InnertubeError::NotFoundInJSON(_)) {
                    ValidationError::Invalid.into()
                } else {
                    ValidationResult::UnableToValidate
                };
            }
        }
    };
}

#[async_trait]
impl Validator for InnertubeValidator {
    async fn is_channel_information_valid(&self, channel: &Channel) -> ValidationResult<Channel> {
        let it_channel = unwrap!(self.innertube.channel_metadata(&channel.id).await);

        if channel.name != it_channel.title {
            return ValidationError::InvalidWithContext(
                "invalid channel name provided".to_string(),
            )
            .into();
        }

        ValidationResult::ValidAfterChanges(Channel {
            id: channel.id.clone(),
            name: it_channel.title,
            avatar: Some(it_channel.avatar.thumbnails[0].url.clone()),
            verified: channel.verified,
        })
    }

    async fn is_video_information_valid(
        &self,
        video_data: &CreateVideo,
    ) -> ValidationResult<CreateVideo> {
        let mut video_data_uploader = video_data.uploader.clone();
        let it_video = unwrap!(self.innertube.video_metadata(&video_data.id).await);

        if video_data.title != it_video.title {
            return ValidationError::InvalidWithContext("invalid video title provided".to_string())
                .into();
        }

        if video_data.uploader.id == it_video.channel_id
            && self
                .is_channel_information_valid(&video_data.uploader)
                .await
                .assign_changes(&mut video_data_uploader)
                .is_valid()
        {
            ValidationResult::ValidAfterChanges(CreateVideo {
                title: it_video.title,
                uploader: video_data_uploader,
                thumbnail_url: it_video.thumbnail.thumbnails[0].url.clone(),
                duration: it_video.length_seconds as i32,
                ..video_data.to_owned()
            })
        } else {
            ValidationError::Invalid.into()
        }
    }

    async fn is_playlist_information_valid(
        &self,
        playlist: &ExtendedPublicPlaylist,
    ) -> ValidationResult<ExtendedPublicPlaylist> {
        let it_playlist = unwrap!(
            self.innertube
                .playlist_metadata(&playlist.playlist.id)
                .await
        );

        if playlist.playlist.title != it_playlist.title {
            return ValidationError::InvalidWithContext(
                "invalid playlist title provided".to_string(),
            )
            .into();
        }

        let mut uploader = playlist.uploader.clone();

        if playlist
            .playlist
            .video_count
            .is_some_and(|video_count| video_count != it_playlist.video_count as u64)
        {
            return ValidationError::InvalidWithContext(
                "video count is not equal to the actual amount of videos".to_string(),
            )
            .into();
        };

        match self
            .is_channel_information_valid(&playlist.uploader)
            .await
            .assign_changes(&mut uploader)
        {
            ValidationResult::Valid => (),
            ValidationResult::ValidAfterChanges(changes) => uploader = changes,
            ValidationResult::UnableToValidate => return ValidationResult::UnableToValidate,
            ValidationResult::Invalid(err) => return ValidationResult::Invalid(err),
        };

        ValidationResult::ValidAfterChanges(ExtendedPublicPlaylist {
            playlist: ExtendedPlaylist {
                id: playlist.playlist.id.clone(),
                title: it_playlist.title,
                description: it_playlist.description.unwrap_or_default(),
                thumbnail_url: Some(it_playlist.thumbnail.max_resolution().url),
                video_count: Some(it_playlist.video_count as u64),
            },
            uploader,
        })
    }
}
