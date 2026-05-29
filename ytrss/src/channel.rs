use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{YOUTUBE_BASE_URL, YouTubeError, YouTubeResult, video::RssVideo};

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RssChannel {
    pub(crate) author: RssChannelAuthor,
    #[serde(rename = "entry")]
    pub(crate) videos: Vec<RssVideo>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub(crate) struct RssChannelAuthor {
    pub(crate) name: String,
}

impl RssChannel {
    pub async fn fetch_from_channel_id(channel_id: &str) -> YouTubeResult<Self> {
        let feed_url = format!(
            "{}/feeds/videos.xml?channel_id={}",
            YOUTUBE_BASE_URL, channel_id
        );
        let response_body = reqwest::get(feed_url)
            .await
            .map_err(|_err| YouTubeError::ConnectionError)?
            .text()
            .await
            .map_err(|_err| YouTubeError::ConnectionError)?;

        serde_roxmltree::from_str(&response_body).map_err(YouTubeError::ParserError)
    }

    pub fn name(&self) -> &str {
        &self.author.name
    }

    pub fn oldest_video_date(&self) -> Option<DateTime<Utc>> {
        self.videos.last().map(|vid| vid.published)
    }

    pub fn find_video(&self, id: &str) -> Option<&RssVideo> {
        self.videos.iter().find(|vid| vid.id == id)
    }
}
