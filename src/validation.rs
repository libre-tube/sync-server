//! Validates user-provided data to be valid (to some extent, as it only has limited info due to using YouTube's RSS feeds)

use std::str::FromStr;

use actix_web::{error, http::Uri};

use crate::{
    DbConnection,
    database::{channel::get_channel_by_id, video::get_video_by_id},
    dto::CreateVideo,
    models::{Channel, Video},
    youtube::channel::ChannelFetcher,
};

const ALLOWED_THUMBNAIL_DOMAINS: [&str; 4] =
    ["youtube.com", "googlevideo.com", "ytimg.com", "ggpht.com"];

fn verify_image_url(image_url: &str) -> bool {
    // TODO: don't rely on Actix for this, bad separation of concerns
    let Ok(uri) = Uri::from_str(image_url) else {
        return false;
    };

    let Some(host) = uri.host() else {
        return false;
    };

    for thumbnail_domain in ALLOWED_THUMBNAIL_DOMAINS {
        if host.ends_with(thumbnail_domain) {
            return true;
        }
    }

    false
}

pub async fn validate_channel_information_if_changed(
    conn: &mut DbConnection,
    channel: &Channel,
) -> actix_web::Result<()> {
    // verification is only required if the channel doesn't exist yet or has changed since then
    if let Some(existing_channel) = get_channel_by_id(conn, &channel.id).await.ok().flatten()
        && *channel == existing_channel
    {
        return Ok(());
    }

    validate_channel_information(channel)
        .await
        .map_err(error::ErrorBadRequest)?;

    Ok(())
}

async fn validate_channel_information(channel: &Channel) -> Result<(), String> {
    if !verify_image_url(&channel.avatar) {
        return Err("invalid channel information provided".to_string());
    }

    let channel_info = ChannelFetcher::get_channel_rss(&channel.id)
        .await
        .map_err(|err| err.to_string())?;

    if channel_info
        .name
        .trim()
        .eq_ignore_ascii_case(channel.name.trim())
    {
        return Err("invalid channel information provided".to_string());
    }

    Ok(())
}
pub async fn validate_video_information_if_changed(
    conn: &mut DbConnection,
    video_data: &mut CreateVideo,
) -> actix_web::Result<()> {
    // TODO: don't fetch same channel info twice!
    validate_channel_information_if_changed(conn, &video_data.uploader).await?;

    // verification is only required if the channel doesn't exist yet or has changed since then
    if let Some(existing_video) = get_video_by_id(conn, &video_data.id).await.ok().flatten()
        && std::convert::Into::<Video>::into(&*video_data) == existing_video
    {
        return Ok(());
    }

    validate_video_information(video_data)
        .await
        .map_err(error::ErrorBadRequest)?;

    Err(error::ErrorBadRequest("video doesn't exist"))
}

async fn validate_video_information(video_data: &mut CreateVideo) -> Result<(), String> {
    // validate thumbnail URL
    if !verify_image_url(&video_data.thumbnail_url) {
        return Err("invalid channel information provided".to_string());
    }

    let channel_info = ChannelFetcher::get_channel_rss(&video_data.uploader.id)
        .await
        .map_err(|err| err.to_string())?;

    // RSS feed doesn't contain videos, so we can't validate anything
    if channel_info.videos.is_empty() {
        return Ok(());
    }
    let oldest_date = channel_info
        .videos
        .last()
        .map(|vid| vid.published_date)
        .unwrap();

    // Video is older than the videos in the feed
    if oldest_date.timestamp_millis() > video_data.upload_date {
        return Ok(());
    }

    // look if video exists in RSS feed
    for video_rss in channel_info.videos {
        if video_rss.id == video_data.id {
            // update video information to the one from the RSS feed
            video_data.title = video_rss.title;
            video_data.upload_date = video_rss.published_date.timestamp_millis();
            video_data.thumbnail_url = video_rss.thumbnail;

            return Ok(());
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{
        models::Channel,
        validation::{validate_channel_information, verify_image_url},
    };

    #[test]
    fn test_image_url_validator() {
        assert!(verify_image_url(
            "https://i1.ytimg.com/vi/hTC6Xa5TrRc/hqdefault.jpg"
        ));
        assert!(verify_image_url(
            "https://ytimg.com/vi/hTC6Xa5TrRc/hqdefault.jpg"
        ));
        assert!(!verify_image_url(
            "https://mydomain.com/vi/hTC6Xa5TrRc/hqdefault.jpg"
        ));
    }

    #[actix_rt::test]
    async fn test_channel_validator() {
        assert!(
            validate_channel_information(&Channel {
                id: "UC8-Th83bH_thdKZDJCrn88g".to_string(),
                name: "The Tonight Show Starring Jimmy Fallon".to_string(),
                avatar: "https://i1.ytimg.com/vi/hTC6Xa5TrRc/hqdefault.jpg".to_string(),
                verified: true,
            })
            .await
            .is_ok()
        );

        assert!(
            validate_channel_information(&Channel {
                id: "UC8-Th83bH_thdKZDJCrn88g".to_string(),
                name: "The Tonight Show Starring Jimmy Fallon".to_string(),
                avatar: "https://i1.example.com/vi/hTC6Xa5TrRc/hqdefault.jpg".to_string(),
                verified: true,
            })
            .await
            .is_err()
        );

        assert!(
            validate_channel_information(&Channel {
                id: "UC8-Th83bH_thdKZDJCrn88g".to_string(),
                name: "Wrong channel name".to_string(),
                avatar: "https://i1.example.com/vi/hTC6Xa5TrRc/hqdefault.jpg".to_string(),
                verified: true,
            })
            .await
            .is_err()
        );
    }
}
