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
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RedditFeedSettings {
    /// JSON feed URL for saved links
    pub saved_url: Option<String>,
    /// JSON feed URL for upvoted links
    pub upvoted_url: Option<String>,
    /// Only sync posts created after this Unix timestamp (e.g., 1708041600 for 2024-02-16)
    /// Set this to "now" when first deploying to skip historical items
    pub since_timestamp: Option<i64>,
    /// Exclude NSFW posts (over_18 = true)
    #[serde(default)]
    pub exclude_nsfw: bool,
    /// Sync schedule (defaults to @daily)
    pub schedule: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PinboardSettings {
    pub token: Option<String>,
    pub schedule: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Settings {
    pub hn: HNSettings,
    pub karakeep: KarakeepSettings,
    pub reddit: RedditSettings,
    /// Reddit JSON feeds (no OAuth required) - from old.reddit.com/prefs/feeds
    pub reddit_feed: RedditFeedSettings,
    pub github: GitHubSettings,
    pub pinboard: PinboardSettings,
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
            .set_override("reddit_feed.schedule", "@daily")
            .unwrap()
            .set_override("github.schedule", "@daily")
            .unwrap()
            .set_override("pinboard.schedule", "@daily")
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
