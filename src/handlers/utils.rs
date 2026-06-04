pub fn thumbnail_url_from_id(video_id: &str) -> String {
    format!("https://i.ytimg.com/vi/{}/hq720.jpg", video_id)
}
