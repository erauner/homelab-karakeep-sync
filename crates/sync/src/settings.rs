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
