use config::Config;
use serde::Deserialize;
use std::sync::OnceLock;

use crate::settings;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GitHubSettings {
    pub token: Option<String>,
    pub schedule: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct HNSettings {
    pub auth: Option<String>,
    pub schedule: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct KarakeepSettings {
    pub auth: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RedditSettings {
    pub clientid: Option<String>,
    pub clientsecret: Option<String>,
    pub refreshtoken: Option<String>,
    pub schedule: String,
}

/// Reddit JSON feed settings - supports multiple feed types from old.reddit.com/prefs/feeds
/// Note: Field names use no underscores to work with config crate's env var parsing
/// Env vars: KS_REDDITFEED_SAVEDURL, KS_REDDITFEED_UPVOTEDURL, etc.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RedditFeedSettings {
    /// JSON feed URL for saved links (env: KS_REDDITFEED_SAVEDURL)
    pub savedurl: Option<String>,
    /// JSON feed URL for upvoted links (env: KS_REDDITFEED_UPVOTEDURL)
    pub upvotedurl: Option<String>,
    /// Only sync posts created after this Unix timestamp (env: KS_REDDITFEED_SINCETIMESTAMP)
    pub sincetimestamp: Option<i64>,
    /// Exclude NSFW posts (env: KS_REDDITFEED_EXCLUDENSFW)
    #[serde(default)]
    pub excludensfw: bool,
    /// Sync schedule (defaults to @daily)
    pub schedule: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PinboardSettings {
    pub token: Option<String>,
    pub schedule: String,
}

/// YouTube Data API v3 settings for syncing liked videos
/// Requires OAuth2 credentials from Google Cloud Console
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct YouTubeSettings {
    /// OAuth2 client ID from Google Cloud Console
    pub clientid: Option<String>,
    /// OAuth2 client secret from Google Cloud Console
    pub clientsecret: Option<String>,
    /// OAuth2 refresh token (obtained via one-time auth flow)
    pub refreshtoken: Option<String>,
    /// Comma-separated list of category IDs to exclude (e.g., "10" for Music)
    /// See: https://developers.google.com/youtube/v3/docs/videoCategories/list
    pub excludecategories: Option<String>,
    /// Force full sync of all pages (ignores "5 consecutive existing" optimization)
    /// Set to "true" for initial import of all historical likes
    #[serde(default)]
    pub fullsync: bool,
    /// Sync schedule (defaults to @daily)
    pub schedule: String,
}

/// Readwise Reader API settings for syncing documents
/// Get your token from https://readwise.io/access_token
/// Uses Reader API v3 which supports location filtering
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ReadwiseSettings {
    /// Readwise API access token (same token works for Reader API)
    pub token: Option<String>,
    /// Document location filter - only sync documents in this location
    /// Valid values: "new", "later", "shortlist", "archive", "feed"
    /// Default: "archive" (only sync archived items)
    /// Set to empty string to sync ALL locations (not recommended)
    pub location: Option<String>,
    /// Comma-separated list of categories to include (e.g., "article,tweet,video")
    /// Valid: article, email, rss, highlight, note, pdf, epub, tweet, video
    /// If empty, syncs all categories
    pub category: Option<String>,
    /// Force full sync of all pages (ignores "5 consecutive existing" optimization)
    /// Set to "true" for initial import of all historical items
    #[serde(default)]
    pub fullsync: bool,
    /// Sync schedule (defaults to @daily)
    pub schedule: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Settings {
    pub hn: HNSettings,
    pub karakeep: KarakeepSettings,
    pub reddit: RedditSettings,
    /// Reddit JSON feeds (no OAuth required) - from old.reddit.com/prefs/feeds
    /// Env var prefix: KS_REDDITFEED_*
    pub redditfeed: RedditFeedSettings,
    pub github: GitHubSettings,
    pub pinboard: PinboardSettings,
    /// YouTube liked videos (requires OAuth2)
    /// Env var prefix: KS_YOUTUBE_*
    pub youtube: YouTubeSettings,
    /// Readwise highlights and articles
    /// Env var prefix: KS_READWISE_*
    pub readwise: ReadwiseSettings,
}

impl Settings {
    pub fn new() -> Self {
        dotenvy::dotenv().ok();

        let config = Config::builder()
            .add_source(config::Environment::with_prefix("KS").separator("_"))
            .set_override("hn.schedule", "@daily")
            .unwrap()
            .set_override("reddit.schedule", "@daily")
            .unwrap()
            .set_override("redditfeed.schedule", "@daily")
            .unwrap()
            .set_override("github.schedule", "@daily")
            .unwrap()
            .set_override("pinboard.schedule", "@daily")
            .unwrap()
            .set_override("youtube.schedule", "@daily")
            .unwrap()
            .set_override("readwise.schedule", "@daily")
            .unwrap()
            .build()
            .unwrap();

        config
            .try_deserialize::<settings::Settings>()
            .expect("Failed to deserialize settings")
    }
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();
pub fn get_settings() -> &'static Settings {
    SETTINGS.get_or_init(Settings::new)
}
