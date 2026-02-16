//! Reddit JSON feed plugins - uses private feeds from old.reddit.com/prefs/feeds
//! No OAuth required - just the feed URL with embedded auth token.

use async_trait::async_trait;
use futures::Stream;
use karakeep_client::BookmarkCreate;
use serde::Deserialize;
use std::{pin::Pin, sync::Arc};

use crate::settings;

/// Reddit JSON feed response structure
#[derive(Debug, Deserialize)]
struct RedditListing {
    data: RedditListingData,
}

#[derive(Debug, Deserialize)]
struct RedditListingData {
    children: Vec<RedditChild>,
    after: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RedditChild {
    data: RedditPost,
}

#[derive(Debug, Deserialize)]
struct RedditPost {
    title: Option<String>,
    url: Option<String>,
    permalink: String,
    /// Unix timestamp when the post was created
    created_utc: Option<f64>,
    /// Whether the post is marked NSFW (over 18)
    #[serde(default)]
    over_18: bool,
}

/// Create a bookmark stream from a Reddit JSON feed URL
/// If `since_timestamp` is provided, only posts created after that time will be included
/// If `exclude_nsfw` is true, posts marked as over_18 will be skipped
async fn stream_from_feed(
    feed_url: String,
    since_timestamp: Option<i64>,
    exclude_nsfw: bool,
) -> anyhow::Result<Pin<Box<dyn Stream<Item = Vec<BookmarkCreate>> + Send>>> {
    let client = reqwest::Client::builder()
        .user_agent("karakeep-sync/1.0")
        .build()?;
    let client = Arc::new(client);

    if let Some(ts) = since_timestamp {
        tracing::info!("filtering Reddit posts to only those created after timestamp {}", ts);
    }
    if exclude_nsfw {
        tracing::info!("excluding NSFW posts (over_18 = true)");
    }

    enum StreamState {
        Init,
        Next(Option<String>),
        /// Stop pagination - we've hit posts older than our cutoff
        Done,
    }

    let stream = futures::stream::unfold(StreamState::Init, move |state| {
        let client = client.clone();
        let base_url = feed_url.clone();

        async move {
            let after = match &state {
                StreamState::Init => None,
                StreamState::Next(after) => after.clone(),
                StreamState::Done => return None,
            };
            if after.is_none() && matches!(state, StreamState::Next(_)) {
                return None;
            }

            // Build URL with pagination
            let url = match &after {
                Some(after_token) => {
                    if base_url.contains('?') {
                        format!("{}&after={}", base_url, after_token)
                    } else {
                        format!("{}?after={}", base_url, after_token)
                    }
                }
                None => base_url.clone(),
            };

            tracing::debug!("fetching Reddit JSON feed: {}", url);

            let resp = match client.get(&url).send().await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("failed to fetch Reddit feed: {}", e);
                    return None;
                }
            };

            if !resp.status().is_success() {
                tracing::error!("Reddit feed returned status: {}", resp.status());
                return None;
            }

            let listing: RedditListing = match resp.json().await {
                Ok(l) => l,
                Err(e) => {
                    tracing::error!("failed to parse Reddit JSON: {}", e);
                    return None;
                }
            };

            let mut hit_old_post = false;
            let items: Vec<BookmarkCreate> = listing
                .data
                .children
                .into_iter()
                .filter_map(|child| {
                    let post = child.data;

                    // Filter NSFW posts if configured
                    if exclude_nsfw && post.over_18 {
                        tracing::debug!(
                            "skipping NSFW post '{}'",
                            post.title.as_deref().unwrap_or("Untitled")
                        );
                        return None;
                    }

                    // Filter by timestamp if configured
                    if let Some(cutoff) = since_timestamp {
                        if let Some(created) = post.created_utc {
                            if (created as i64) < cutoff {
                                tracing::debug!(
                                    "skipping post '{}' - created at {} which is before cutoff {}",
                                    post.title.as_deref().unwrap_or("Untitled"),
                                    created as i64,
                                    cutoff
                                );
                                hit_old_post = true;
                                return None;
                            }
                        }
                    }

                    // Use the post URL if available, otherwise construct from permalink
                    let url = post
                        .url
                        .or_else(|| Some(format!("https://www.reddit.com{}", post.permalink)))?;

                    Some(BookmarkCreate {
                        title: post.title.unwrap_or_else(|| "Untitled".to_string()),
                        url,
                        created_at: None,
                    })
                })
                .collect();

            tracing::debug!(
                "fetched {} posts from Reddit feed, after: {:?}",
                items.len(),
                listing.data.after
            );

            // If we hit an old post, stop pagination - Reddit returns posts in reverse chronological order
            // so all subsequent pages will also be old
            let next_state = if hit_old_post {
                tracing::info!("reached posts older than cutoff timestamp, stopping pagination");
                StreamState::Done
            } else {
                StreamState::Next(listing.data.after)
            };

            Some((items, next_state))
        }
    });

    Ok(Box::pin(stream))
}

// ============================================================================
// Reddit Saved Feed Plugin
// ============================================================================

#[derive(Debug, Clone)]
pub struct RedditSavedFeed {}

#[async_trait]
impl super::Plugin for RedditSavedFeed {
    fn list_name(&self) -> &'static str {
        "Reddit Saved"
    }

    async fn to_bookmark_stream(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Vec<BookmarkCreate>> + Send>>> {
        let settings = settings::get_settings();
        let feed_url = settings
            .redditfeed
            .savedurl
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Reddit saved feed URL not configured"))?
            .clone();
        let since_timestamp = settings.redditfeed.sincetimestamp;
        let exclude_nsfw = settings.redditfeed.excludensfw;

        tracing::info!("using Reddit saved JSON feed (no OAuth)");
        stream_from_feed(feed_url, since_timestamp, exclude_nsfw).await
    }

    fn is_activated(&self) -> bool {
        let settings = settings::get_settings();
        settings.redditfeed.savedurl.is_some()
    }

    fn recurring_schedule(&self) -> String {
        let settings = settings::get_settings();
        settings.redditfeed.schedule.clone()
    }
}

// ============================================================================
// Reddit Upvoted Feed Plugin
// ============================================================================

#[derive(Debug, Clone)]
pub struct RedditUpvotedFeed {}

#[async_trait]
impl super::Plugin for RedditUpvotedFeed {
    fn list_name(&self) -> &'static str {
        "Reddit Upvoted"
    }

    async fn to_bookmark_stream(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Vec<BookmarkCreate>> + Send>>> {
        let settings = settings::get_settings();
        let feed_url = settings
            .redditfeed
            .upvotedurl
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Reddit upvoted feed URL not configured"))?
            .clone();
        let since_timestamp = settings.redditfeed.sincetimestamp;
        let exclude_nsfw = settings.redditfeed.excludensfw;

        tracing::info!("using Reddit upvoted JSON feed (no OAuth)");
        stream_from_feed(feed_url, since_timestamp, exclude_nsfw).await
    }

    fn is_activated(&self) -> bool {
        let settings = settings::get_settings();
        settings.redditfeed.upvotedurl.is_some()
    }

    fn recurring_schedule(&self) -> String {
        let settings = settings::get_settings();
        settings.redditfeed.schedule.clone()
    }
}
