//! Validates user-provided data to be valid (to some extent, as it only has limited info due to using YouTube's RSS feeds)

use std::{collections::HashSet, str::FromStr};

use actix_web::{error, http::Uri};

use crate::{
    CONFIG, DbConnection,
    database::{channel::get_channel_by_id, video::get_video_by_id},
    dto::CreateVideo,
    models::{Channel, Video},
    youtube::{FeedRss, channel::ChannelFetcher},
};

const ALLOWED_THUMBNAIL_DOMAINS: [&str; 5] = [
    "youtube.com",
    "googlevideo.com",
    "ytimg.com",
    "ggpht.com",
    "googleusercontent.com",
];

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
) -> actix_web::Result<Option<FeedRss>> {
    if !CONFIG.validate_submitted_metadata {
        return Ok(None);
    }

    // verification is only required if the channel doesn't exist yet or has changed since then
    if let Some(existing_channel) = get_channel_by_id(conn, &channel.id).await.ok().flatten()
        && *channel == existing_channel
    {
        return Ok(None);
    }

    validate_channel_information(channel)
        .await
        .map_err(error::ErrorBadRequest)
}

/// Validate if the provided channel information is valid.
/// If yes, the method returns an `Ok` result. If not, the method returns an `Err`.
///
/// The return value inside the `Option<ChannelRss>` doesn't say anything about
/// whether the verification was succesfull, it's only returned in case the caller
/// wants to re-use the RSS feed info.
async fn validate_channel_information(channel: &Channel) -> Result<Option<FeedRss>, String> {
    if !verify_image_url(&channel.avatar) {
        return Err("invalid channel avatar provided".to_string());
    }

    let channel_info = ChannelFetcher::get_channel_rss(&channel.id)
        .await
        .map_err(|err| err.to_string())?;

    if !channel_info
        .channel_name
        .trim()
        .eq_ignore_ascii_case(channel.name.trim())
    {
        return Err("invalid channel information provided".to_string());
    }

    Ok(Some(channel_info))
}

/// Requirement: all videos must be from the same channel!
pub async fn validate_video_information_if_changed(
    conn: &mut DbConnection,
    video_datas: &mut [CreateVideo],
) -> actix_web::Result<()> {
    if !CONFIG.validate_submitted_metadata {
        return Ok(());
    }

    let channel = video_datas
        .iter()
        .map(|video_data| video_data.uploader.clone())
        .collect::<HashSet<Channel>>();
    if channel.len() != 1 {
        return Err(error::ErrorInternalServerError(
            "can only process videos from the same channel",
        ));
    }
    let channel = channel.iter().next().unwrap();

    let channel_rss = validate_channel_information_if_changed(conn, channel).await?;
    let channel_rss = match channel_rss {
        Some(channel_rss) => channel_rss,
        None => {
            // RSS not loaded yet, so we have to load it now
            ChannelFetcher::get_channel_rss(&channel.id)
                .await
                .map_err(|err| error::ErrorInternalServerError(err.to_string()))?
        }
    };

    for video_data in video_datas.iter_mut() {
        // verification is only required if the channel doesn't exist yet or has changed since then
        if let Some(existing_video) = get_video_by_id(conn, &video_data.id).await.ok().flatten()
            && std::convert::Into::<Video>::into(&*video_data) == existing_video
        {
            continue;
        }

        (*video_data) = validate_video_information(video_data.clone(), &channel_rss)
            .map_err(error::ErrorBadRequest)?;
    }

    Ok(())
}

/// Validates the video exists and returns updated meta information from the RSS feed.
///
/// You should use the resulting [CreateVideo] for doing any further actions with the video,
/// because its metadata is more accurate.
fn validate_video_information(
    video_data: CreateVideo,
    channel_rss: &FeedRss,
) -> Result<CreateVideo, String> {
    // validate thumbnail URL
    if !verify_image_url(&video_data.thumbnail_url) {
        return Err("invalid channel information provided".to_string());
    }

    // RSS feed doesn't contain videos, so we can't validate anything
    if channel_rss.videos.is_empty() {
        return Ok(video_data);
    }
    let oldest_date = channel_rss
        .videos
        .last()
        .map(|vid| vid.published_date)
        .unwrap();

    // Video is older than the videos in the feed
    if oldest_date.timestamp_millis() > video_data.upload_date {
        return Ok(video_data);
    }

    // look if video exists in RSS feed
    for video_rss in &channel_rss.videos {
        if video_rss.id == video_data.id {
            // update video information to the one from the RSS feed
            let mut video_data = video_data;
            video_data.title = video_rss.title.clone();
            video_data.upload_date = video_rss.published_date.timestamp_millis();
            video_data.thumbnail_url = video_rss.thumbnail.clone();

            return Ok(video_data);
        }
    }

    Ok(video_data)
}

#[cfg(test)]
mod test {
    use crate::{
        dto::CreateVideo,
        models::Channel,
        validation::{validate_channel_information, validate_video_information, verify_image_url},
        youtube::channel::ChannelFetcher,
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

    #[actix_rt::test]
    async fn test_video_validator() {
        let video = CreateVideo {
            id: "kMO1L5J1cn8".to_string(),
            title: "Minecraft Livestream [FaceCam] | Kotti".to_string(),
            upload_date: 1549036231000, /* 2019-02-01T16:50:31+00:00 */
            thumbnail_url: "https://i4.ytimg.com/vi/kMO1L5J1cn8/hqdefault.jpg".to_string(),
            duration: 4352,
            uploader: Channel {
                id: "UCWnQYRWgTbsLTDOAVc3uzRg".to_string(),
                name: "KottiXD".to_string(),
                avatar: "https://yt3.googleusercontent.com/ytc/AIdro_lBXTw2HqumabqUMrMcWlB5BVUa-bDCP1YQ0Jwf89C6RMY=s160-c-k-c0x00ffffff-no-rj".to_string(),
                verified: false,
            },
        };

        let channel_rss = ChannelFetcher::get_channel_rss(&video.uploader.id)
            .await
            .unwrap();
        assert!(validate_video_information(video, &channel_rss).is_ok());
    }
}
