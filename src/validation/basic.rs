use async_trait::async_trait;

use crate::{
    CONFIG,
    dto::{CreateVideo, ExtendedPublicPlaylist},
    models::Channel,
    validation::{ValidationError, ValidationResult, Validator},
};

const ALLOWED_THUMBNAIL_DOMAINS: [&str; 5] = [
    "youtube.com",
    "googlevideo.com",
    "ytimg.com",
    "ggpht.com",
    "googleusercontent.com",
];

const MAX_IMAGE_URL_LENGTH: usize = 150;

pub fn verify_image_url(image_url: &str) -> bool {
    if image_url.is_empty() {
        return true;
    }

    if image_url.chars().count() > MAX_IMAGE_URL_LENGTH {
        return false;
    }

    let Ok(url) = url::Url::parse(image_url) else {
        return false;
    };

    let Some(host) = url.host_str() else {
        return false;
    };

    for thumbnail_domain in ALLOWED_THUMBNAIL_DOMAINS {
        if host.ends_with(thumbnail_domain) {
            return true;
        }
    }

    false
}

pub struct BasicValidator;

#[async_trait]
impl Validator for BasicValidator {
    async fn is_channel_information_valid(&self, channel: &Channel) -> ValidationResult<Channel> {
        if !CONFIG.validate_submitted_metadata {
            return ValidationResult::Valid;
        }

        if channel
            .avatar
            .as_ref()
            .is_some_and(|avatar_url| !verify_image_url(avatar_url))
        {
            ValidationError::InvalidWithContext("channel avatar url".to_string()).into()
        } else {
            ValidationResult::Skip
        }
    }

    async fn is_video_information_valid(
        &self,
        video_data: &CreateVideo,
    ) -> ValidationResult<CreateVideo> {
        if !CONFIG.validate_submitted_metadata {
            return ValidationResult::Valid;
        }

        if let ValidationResult::Invalid(err) = self
            .is_channel_information_valid(&video_data.uploader)
            .await
        {
            return ValidationResult::Invalid(err);
        }

        if !verify_image_url(&video_data.thumbnail_url) {
            ValidationError::InvalidWithContext("video thumbnail url".to_string()).into()
        } else {
            ValidationResult::Skip
        }
    }

    async fn is_playlist_information_valid(
        &self,
        playlist: &ExtendedPublicPlaylist,
    ) -> ValidationResult<ExtendedPublicPlaylist> {
        if !CONFIG.validate_submitted_metadata {
            return ValidationResult::Valid;
        }

        if let ValidationResult::Invalid(err) =
            self.is_channel_information_valid(&playlist.uploader).await
        {
            return ValidationResult::Invalid(err);
        }

        if playlist
            .playlist
            .thumbnail_url
            .as_ref()
            .is_some_and(|thumbnail_url| !verify_image_url(thumbnail_url))
        {
            ValidationError::InvalidWithContext("playlist thumbnail".to_string()).into()
        } else {
            ValidationResult::Skip
        }
    }
}
