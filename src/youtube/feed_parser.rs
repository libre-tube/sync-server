use chrono::DateTime;

use crate::youtube::{
    FeedRss, VideoRss, YouTubeError, YouTubeResult,
    xml_helpers::{get_child_by_name, get_child_text_by_name, get_children_by_name},
};

pub struct FeedParser {}

impl FeedParser {
    pub async fn parse_feed_rss(raw_feed: &str) -> YouTubeResult<FeedRss> {
        let doc = roxmltree::Document::parse(raw_feed)
            .map_err(|err| YouTubeError::SyntaxError(Box::new(err)))?;

        let videos = get_children_by_name(&doc.root_element(), "entry")
            .map(|entry| -> YouTubeResult<VideoRss> {
                let media_group = get_child_by_name(&entry, "group")?;
                Ok(VideoRss {
                    id: get_child_text_by_name(&entry, "videoId")?.to_string(),
                    title: get_child_text_by_name(&entry, "title")?.to_string(),
                    published_date: get_child_text_by_name(&entry, "published").and_then(
                        |date_string| {
                            DateTime::parse_from_rfc3339(date_string).map_err(|_err| {
                                YouTubeError::ParserError("invalid date format".to_string())
                            })
                        },
                    )?,

                    description: get_child_text_by_name(&media_group, "description")?.to_string(),
                    thumbnail: get_child_by_name(&media_group, "thumbnail")?
                        .attribute("url")
                        .unwrap_or_default()
                        .to_string(),
                })
            })
            .filter_map(|v| v.ok())
            .collect::<Vec<_>>();

        Ok(FeedRss {
            channel_id: get_child_text_by_name(&doc.root_element(), "channelId")?.to_string(),
            channel_name: get_child_by_name(&doc.root_element(), "author")
                .and_then(|el| get_child_text_by_name(&el, "name").map(|t| t.to_string()))?,
            playlist_id: get_child_text_by_name(&doc.root_element(), "playlistId")
                .ok()
                .map(|t| t.to_string()),
            title: get_child_text_by_name(&doc.root_element(), "title")?.to_string(),
            videos,
        })
    }
}
