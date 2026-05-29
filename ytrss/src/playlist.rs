use serde::Deserialize;

use crate::{
    RssChannel, YOUTUBE_BASE_URL, YouTubeError, YouTubeResult, channel::RssChannelAuthor,
    video::RssVideo,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RssPlaylist {
    author: RssChannelAuthor,
    title: String,
    #[serde(rename = "entry")]
    videos: Vec<RssVideo>,
}

impl RssPlaylist {
    pub async fn fetch_from_playlist_id(playlist_id: &str) -> YouTubeResult<Self> {
        let feed_url = format!(
            "{}/feeds/videos.xml?playlist_id={}",
            YOUTUBE_BASE_URL, playlist_id
        );
        let response_body = reqwest::get(feed_url)
            .await
            .map_err(|_err| YouTubeError::ConnectionError)?
            .text()
            .await
            .map_err(|_err| YouTubeError::ConnectionError)?;

        serde_roxmltree::from_str(&response_body).map_err(YouTubeError::ParserError)
    }

    pub fn video_count(&self) -> usize {
        self.videos.len()
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    /// Create an `RssChannel` view for the uploader of this playlist
    pub fn to_channel(&self) -> RssChannel {
        RssChannel {
            author: self.author.clone(),
            videos: self.videos.clone(),
        }
    }
}
