use crate::settings;
use async_trait::async_trait;
use futures::{stream, Stream};
use karakeep_client::BookmarkCreate;
use serde::Deserialize;
use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ReadwiseHighlights {}

/// Reader API v3 response structure
/// Docs: https://readwise.io/reader_api
#[derive(Debug, Deserialize)]
struct ReaderListResponse {
    results: Vec<ReaderDocument>,
    /// Pagination cursor for next page (integer or null)
    #[serde(rename = "nextPageCursor")]
    next_page_cursor: Option<i64>,
}

/// Individual document from Reader API
#[derive(Debug, Deserialize)]
struct ReaderDocument {
    /// Unique document ID
    id: String,
    /// The URL to open in Reader
    url: String,
    /// Original source URL (for articles/tweets/etc)
    source_url: Option<String>,
    /// Document title
    title: Option<String>,
    /// Author name
    author: Option<String>,
    /// Category: article, email, rss, highlight, note, pdf, epub, tweet, video
    category: String,
    /// Location: new, later, shortlist, archive, feed
    location: String,
    /// When the document was saved
    created_at: Option<String>,
}

#[async_trait]
impl super::Plugin for ReadwiseHighlights {
    fn list_name(&self) -> &'static str {
        "Readwise"
    }

    async fn to_bookmark_stream(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Vec<BookmarkCreate>> + Send>>> {
        let settings = settings::get_settings();
        let token = settings
            .readwise
            .token
            .as_ref()
            .expect("Readwise token must be set")
            .clone();

        // Location filter - defaults to "archive" (only sync archived items)
        let location = settings
            .readwise
            .location
            .clone()
            .unwrap_or_else(|| "archive".to_string());

        // Parse category filter (filter out empty strings to handle KS_READWISE_CATEGORY="")
        let categories: HashSet<String> = settings
            .readwise
            .category
            .as_ref()
            .map(|s| {
                s.split(',')
                    .map(|c| c.trim().to_lowercase())
                    .filter(|c| !c.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        // Track seen URLs to avoid duplicates
        let seen_urls = Arc::new(Mutex::new(HashSet::<String>::new()));

        let stream = stream::unfold(
            Some(0i64),  // 0 means first page
            move |page_cursor| {
                let token = token.clone();
                let location = location.clone();
                let categories = categories.clone();
                let seen_urls = Arc::clone(&seen_urls);

                async move {
                    let page_cursor = page_cursor?;

                    tracing::info!(
                        "fetching Reader documents (location={}), cursor: {:?}",
                        location,
                        if page_cursor == 0 {
                            "first page".to_string()
                        } else {
                            page_cursor.to_string()
                        }
                    );

                    let client = reqwest::Client::new();

                    // Build Reader API v3 URL with location filter
                    // Docs: https://readwise.io/reader_api
                    let mut url = format!(
                        "https://readwise.io/api/v3/list/?location={}",
                        location
                    );
                    if page_cursor > 0 {
                        url.push_str(&format!("&pageCursor={}", page_cursor));
                    }

                    let resp = client
                        .get(&url)
                        .header("Authorization", format!("Token {}", token))
                        .send()
                        .await
                        .ok()?;

                    if !resp.status().is_success() {
                        tracing::error!("Reader API error: {}", resp.status());
                        return None;
                    }

                    let reader_resp: ReaderListResponse = resp.json().await.ok()?;
                    let next_cursor = reader_resp.next_page_cursor;

                    // Filter and deduplicate
                    let mut seen = seen_urls.lock().await;
                    let bookmarks: Vec<BookmarkCreate> = reader_resp
                        .results
                        .into_iter()
                        .filter(|doc| {
                            // Get the best URL (prefer source_url, fall back to reader url)
                            let url = doc.source_url.as_ref().unwrap_or(&doc.url);

                            // Skip if already seen
                            if seen.contains(url) {
                                return false;
                            }

                            // Apply category filter if specified
                            if !categories.is_empty() {
                                if !categories.contains(&doc.category.to_lowercase()) {
                                    tracing::debug!(
                                        "skipping {:?} in category {}: {}",
                                        doc.title,
                                        doc.category,
                                        url
                                    );
                                    return false;
                                }
                            }

                            // Mark as seen
                            seen.insert(url.clone());
                            true
                        })
                        .map(|doc| {
                            // Prefer source_url for bookmarking (original article URL)
                            let url = doc.source_url.unwrap_or(doc.url);
                            let title = match (doc.title, doc.author) {
                                (Some(t), Some(a)) => format!("{} - {}", t, a),
                                (Some(t), None) => t,
                                (None, _) => url.clone(),
                            };
                            BookmarkCreate {
                                url,
                                title,
                                created_at: doc.created_at,
                            }
                        })
                        .collect();

                    tracing::info!(
                        "processed {} documents from Reader (location={})",
                        bookmarks.len(),
                        location
                    );

                    // Return None to stop iteration if no more pages
                    if bookmarks.is_empty() && next_cursor.is_none() {
                        return None;
                    }

                    Some((bookmarks, next_cursor))
                }
            },
        );

        Ok(Box::pin(stream))
    }

    fn is_activated(&self) -> bool {
        let settings = settings::get_settings();
        let has_token = settings.readwise.token.is_some()
            && !settings.readwise.token.as_ref().unwrap().is_empty();
        tracing::info!(
            "Readwise plugin activation check: has_token={}, location={:?}",
            has_token,
            settings.readwise.location
        );
        has_token
    }

    fn recurring_schedule(&self) -> String {
        let settings = settings::get_settings();
        settings.readwise.schedule.clone()
    }

    fn force_full_sync(&self) -> bool {
        let settings = settings::get_settings();
        settings.readwise.fullsync
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_readwise_category_filter() {
        // Reader API categories: article, email, rss, highlight, note, pdf, epub, tweet, video
        let category = "article,tweet,video";
        let parsed: Vec<&str> = category.split(',').map(|c| c.trim()).collect();
        assert_eq!(parsed, vec!["article", "tweet", "video"]);
    }

    #[test]
    fn test_empty_category_filter() {
        // Empty string should result in no filter (sync all categories)
        let category = "";
        let parsed: Vec<&str> = category
            .split(',')
            .map(|c| c.trim())
            .filter(|c| !c.is_empty())
            .collect();
        assert!(parsed.is_empty());
    }
}
