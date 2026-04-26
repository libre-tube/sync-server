use std::fmt::Display;

use chrono::{DateTime, FixedOffset};

// TODO: move into different crate
pub mod channel;
mod feed_parser;
pub mod playlist;
mod xml_helpers;

const YOUTUBE_BASE_URL: &str = "https://www.youtube.com";

#[derive(Debug, Default, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct FeedRss {
    pub channel_id: String,
    pub channel_name: String,
    /// only available for playlist feeds
    pub playlist_id: Option<String>,
    pub title: String,
    pub videos: Vec<VideoRss>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct VideoRss {
    pub id: String,
    pub title: String,
    pub description: String,
    pub published_date: DateTime<FixedOffset>,
    pub thumbnail: String,
}

type InnerError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum YouTubeError {
    ConnectionError,
    ParserError(String),
    SyntaxError(InnerError),
}

impl Display for YouTubeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            YouTubeError::ConnectionError => write!(f, "failed to connect to youtube"),
            YouTubeError::SyntaxError(reason) => write!(f, "malformed RSS: {reason}"),
            YouTubeError::ParserError(reason) => write!(f, "failed to parse: {reason}"),
        }
    }
}
impl std::error::Error for YouTubeError {}

type YouTubeResult<T> = Result<T, YouTubeError>;
