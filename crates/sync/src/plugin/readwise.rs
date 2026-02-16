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

#[derive(Debug, Deserialize)]
struct ExportResponse {
    results: Vec<ExportResult>,
    #[serde(rename = "nextPageCursor")]
    next_page_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExportResult {
    /// Source URL of the article/book/etc
    source_url: Option<String>,
    /// Title of the source
    title: String,
    /// Category: articles, books, tweets, supplementals, podcasts
    category: String,
    /// Author name
    author: Option<String>,
    /// When the source was last highlighted
    updated: Option<String>,
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

        // Parse category filter
        let categories: HashSet<String> = settings
            .readwise
            .categories
            .as_ref()
            .map(|s| s.split(',').map(|c| c.trim().to_lowercase()).collect())
            .unwrap_or_default();

        // Track seen URLs to avoid duplicates (Readwise exports per-highlight, not per-source)
        let seen_urls = Arc::new(Mutex::new(HashSet::<String>::new()));

        let stream = stream::unfold(
            Some("".to_string()),
            move |page_cursor| {
                let token = token.clone();
                let categories = categories.clone();
                let seen_urls = Arc::clone(&seen_urls);

                async move {
                    let page_cursor = page_cursor?;

                    tracing::info!(
                        "fetching Readwise highlights, cursor: {:?}",
                        if page_cursor.is_empty() {
                            "first page"
                        } else {
                            &page_cursor
                        }
                    );

                    let client = reqwest::Client::new();

                    let mut url = "https://readwise.io/api/v2/export/".to_string();
                    if !page_cursor.is_empty() {
                        url.push_str(&format!("?pageCursor={}", page_cursor));
                    }

                    let resp = client
                        .get(&url)
                        .header("Authorization", format!("Token {}", token))
                        .send()
                        .await
                        .ok()?;

                    if !resp.status().is_success() {
                        tracing::error!("Readwise API error: {}", resp.status());
                        return None;
                    }

                    let export_resp: ExportResponse = resp.json().await.ok()?;
                    let next_cursor = export_resp.next_page_cursor.clone();

                    // Filter and deduplicate
                    let mut seen = seen_urls.lock().await;
                    let bookmarks: Vec<BookmarkCreate> = export_resp
                        .results
                        .into_iter()
                        .filter(|item| {
                            // Must have a source URL
                            if item.source_url.is_none() {
                                return false;
                            }
                            let url = item.source_url.as_ref().unwrap();

                            // Skip if already seen
                            if seen.contains(url) {
                                return false;
                            }

                            // Apply category filter if specified
                            if !categories.is_empty() {
                                if !categories.contains(&item.category.to_lowercase()) {
                                    tracing::debug!(
                                        "skipping {} in category {}: {}",
                                        item.title,
                                        item.category,
                                        url
                                    );
                                    return false;
                                }
                            }

                            // Mark as seen
                            seen.insert(url.clone());
                            true
                        })
                        .map(|item| {
                            let url = item.source_url.unwrap();
                            let title = if let Some(author) = item.author {
                                format!("{} - {}", item.title, author)
                            } else {
                                item.title
                            };
                            BookmarkCreate {
                                url,
                                title,
                                created_at: item.updated,
                            }
                        })
                        .collect();

                    tracing::info!("processed {} unique sources from Readwise", bookmarks.len());

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
        settings.readwise.token.is_some()
            && !settings.readwise.token.as_ref().unwrap().is_empty()
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
    fn test_readwise_categories() {
        let categories = "articles,books,tweets";
        let parsed: Vec<&str> = categories.split(',').map(|c| c.trim()).collect();
        assert_eq!(parsed, vec!["articles", "books", "tweets"]);
    }
}
