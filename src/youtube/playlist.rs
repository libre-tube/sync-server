use crate::youtube::{
    FeedRss, YOUTUBE_BASE_URL, YouTubeError, YouTubeResult, feed_parser::FeedParser,
};

pub struct PlaylistFetcher {}

impl PlaylistFetcher {
    pub async fn get_playlist_rss(playlist_id: &str) -> YouTubeResult<FeedRss> {
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

        FeedParser::parse_feed_rss(&response_body).await
    }
}
