use crate::settings;
use async_trait::async_trait;
use futures::{stream, Stream};
use karakeep_client::BookmarkCreate;
use serde::Deserialize;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct YouTubeLiked {}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct VideoListResponse {
    items: Option<Vec<VideoItem>>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VideoItem {
    id: String,
    snippet: VideoSnippet,
}

#[derive(Debug, Deserialize)]
struct VideoSnippet {
    title: String,
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
}

async fn refresh_access_token(client_id: &str, client_secret: &str, refresh_token: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::new();

    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await?;

    if !resp.status().is_success() {
        let error_text = resp.text().await?;
        anyhow::bail!("Failed to refresh token: {}", error_text);
    }

    let token_resp: TokenResponse = resp.json().await?;
    Ok(token_resp.access_token)
}

#[async_trait]
impl super::Plugin for YouTubeLiked {
    fn list_name(&self) -> &'static str {
        "YouTube Liked"
    }

    async fn to_bookmark_stream(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Vec<BookmarkCreate>> + Send>>> {
        let settings = settings::get_settings();
        let client_id = settings
            .youtube
            .clientid
            .as_ref()
            .expect("YouTube client ID must be set")
            .clone();
        let client_secret = settings
            .youtube
            .clientsecret
            .as_ref()
            .expect("YouTube client secret must be set")
            .clone();
        let refresh_token = settings
            .youtube
            .refreshtoken
            .as_ref()
            .expect("YouTube refresh token must be set")
            .clone();

        // Get initial access token
        let access_token = refresh_access_token(&client_id, &client_secret, &refresh_token).await?;
        let access_token = Arc::new(Mutex::new(access_token));

        let stream = stream::unfold(
            Some("".to_string()),
            move |page_token| {
                let access_token = Arc::clone(&access_token);
                let client_id = client_id.clone();
                let client_secret = client_secret.clone();
                let refresh_token = refresh_token.clone();

                async move {
                    let page_token = page_token?;

                    tracing::info!("fetching YouTube liked videos, page_token: {:?}",
                        if page_token.is_empty() { "first page" } else { &page_token });

                    let client = reqwest::Client::new();

                    let mut url = "https://www.googleapis.com/youtube/v3/videos?part=snippet&myRating=like&maxResults=50".to_string();
                    if !page_token.is_empty() {
                        url.push_str(&format!("&pageToken={}", page_token));
                    }

                    let token = access_token.lock().await.clone();
                    let resp = client
                        .get(&url)
                        .header("Authorization", format!("Bearer {}", token))
                        .send()
                        .await
                        .ok()?;

                    // Handle token expiration
                    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
                        tracing::info!("Access token expired, refreshing...");
                        let new_token = refresh_access_token(&client_id, &client_secret, &refresh_token)
                            .await
                            .ok()?;
                        *access_token.lock().await = new_token.clone();

                        // Retry with new token
                        let resp = client
                            .get(&url)
                            .header("Authorization", format!("Bearer {}", new_token))
                            .send()
                            .await
                            .ok()?;

                        let video_resp: VideoListResponse = resp.json().await.ok()?;
                        return process_response(video_resp);
                    }

                    if !resp.status().is_success() {
                        tracing::error!("YouTube API error: {}", resp.status());
                        return None;
                    }

                    let video_resp: VideoListResponse = resp.json().await.ok()?;
                    process_response(video_resp)
                }
            },
        );

        Ok(Box::pin(stream))
    }

    fn is_activated(&self) -> bool {
        let settings = settings::get_settings();
        settings.youtube.clientid.is_some()
            && settings.youtube.clientsecret.is_some()
            && settings.youtube.refreshtoken.is_some()
            && !settings.youtube.clientid.as_ref().unwrap().is_empty()
            && !settings.youtube.clientsecret.as_ref().unwrap().is_empty()
            && !settings.youtube.refreshtoken.as_ref().unwrap().is_empty()
    }

    fn recurring_schedule(&self) -> String {
        let settings = settings::get_settings();
        settings.youtube.schedule.clone()
    }
}

fn process_response(video_resp: VideoListResponse) -> Option<(Vec<BookmarkCreate>, Option<String>)> {
    let items = video_resp.items.unwrap_or_default();

    let bookmarks: Vec<BookmarkCreate> = items
        .into_iter()
        .map(|item| {
            let url = format!("https://www.youtube.com/watch?v={}", item.id);
            BookmarkCreate {
                url,
                title: item.snippet.title,
                created_at: item.snippet.published_at,
            }
        })
        .collect();

    let next_page = video_resp.next_page_token;

    // Return None to stop iteration if no more pages
    if bookmarks.is_empty() && next_page.is_none() {
        return None;
    }

    Some((bookmarks, next_page))
}

#[cfg(test)]
mod test {
    #[test]
    fn test_youtube_url_format() {
        let video_id = "dQw4w9WgXcQ";
        let url = format!("https://www.youtube.com/watch?v={}", video_id);
        assert_eq!(url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    }
}
