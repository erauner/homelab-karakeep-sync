# karakeep-sync

A tool to sync links from various services to [Karakeep](https://github.com/hoarder-app/hoarder) to keep all your interesting links in one place.

## Overview

When looking up something interesting you found in the past, you probably check multiple places - Karakeep, HN upvotes, Reddit bookmarks, etc. This tool syncs all those links to Karakeep automatically, organizing them under lists for easy future access.

## Supported Services

- ✅ Hacker News upvotes
- ✅ Reddit saved posts
- ✅ Github stars
- ✅ Pinboard bookmarks
- ✅ YouTube liked videos
- 🚧 X bookmarks (planned)
- 🚧 Bluesky bookmarks (planned)

## Environment Variables

Configure these environment variables in your `docker-compose.yml`:

| Variable           | Required | Description                                                       |
| ------------------ | -------- | ----------------------------------------------------------------- |
| `KS_KARAKEEP_AUTH` | ✅       | Your Karakeep API token                                           |
| `KS_KARAKEEP_URL`  | ✅       | Your Karakeep instance URL (e.g., `https://karakeep.example.com`) |

### For Hacker News

| Variable         | Required | Description                                      |
| ---------------- | -------- | ------------------------------------------------ |
| `KS_HN_AUTH`     | ❌       | Your Hacker News authentication cookie value     |
| `KS_HN_SCHEDULE` | ❌       | Sync schedule in cron format (default: `@daily`) |

Hacker news auth cookie can be obtained by logging into your HN account and inspecting the cookies in your browser. Look for the `user` cookie.

Hacker News upvotes will be synced to a list named `HN Upvoted` in your Karakeep instance.

Hacker News sync will be skipped if `KS_HN_AUTH` is not set.

### For Reddit

| Variable                 | Required | Description                                      |
| ------------------------ | -------- | ------------------------------------------------ |
| `KS_REDDIT_CLIENTID`     | ❌       | Your Reddit app client ID                        |
| `KS_REDDIT_CLIENTSECRET` | ❌       | Your Reddit app client secret                    |
| `KS_REDDIT_REFRESHTOKEN` | ❌       | Your Reddit app refresh token                    |
| `KS_REDDIT_SCHEDULE`     | ❌       | Sync schedule in cron format (default: `@daily`) |

To obtain a refresh token, you can follow these steps:

1. Create a Reddit app [here](https://www.reddit.com/prefs/apps) (choose "script" as the app type).
2. You can use a tool like [this](https://github.com/not-an-aardvark/reddit-oauth-helper) to generate a refresh token using your app's client ID and client secret. Make sure that the redirect URI matches the one provided from reddit-oauth-helper.
3. Make sure to give the app `history` scope access.
4. Make sure to tick the "permanent" option to get a refresh token.

If you don't want to trust a third party tool, you can also implement the OAuth2 flow yourself using the [Reddit API docs](https://www.reddit.com/dev/api/).

Reddit saves will be synced to a list named `Reddit Saved` in your Karakeep instance.

Reddit sync will be skipped if any of `KS_REDDIT_CLIENTID`, `KS_REDDIT_CLIENTSECRET` or `KS_REDDIT_REFRESHTOKEN` is not set.

### GitHub Stars

| Variable             | Required | Description                                      |
| -------------------- | -------- | ------------------------------------------------ |
| `KS_GITHUB_TOKEN`    | ❌       | Your GitHub personal access token                |
| `KS_GITHUB_SCHEDULE` | ❌       | Sync schedule in cron format (default: `@daily`) |

To obtain a GitHub personal access token, you can visit [this link](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens#creating-a-fine-grained-personal-access-token) and create a new token with `Starring` user permission (read).

GitHub stars will be synced to a list named `GitHub Starred` in your Karakeep instance.

GitHub sync will be skipped if `KS_GITHUB_TOKEN` is not set.

### Pinboard Bookmarks

| Variable               | Required | Description                                      |
| ---------------------- | -------- | ------------------------------------------------ |
| `KS_PINBOARD_TOKEN`    | ❌       | Your Pinboard API token                          |
| `KS_PINBOARD_SCHEDULE` | ❌       | Sync schedule in cron format (default: `@daily`) |

To obtain your Pinboard API token, visit your [Pinboard password page](https://pinboard.in/settings/password) and scroll down to the "API Token" section. The token will be in the format `username:TOKEN`.

Pinboard bookmarks will be synced to a list named `Pinboard` in your Karakeep instance.

Pinboard sync will be skipped if `KS_PINBOARD_TOKEN` is not set.

### YouTube Liked Videos

| Variable                     | Required | Description                                      |
| ---------------------------- | -------- | ------------------------------------------------ |
| `KS_YOUTUBE_CLIENTID`        | ❌       | Your Google Cloud OAuth2 client ID               |
| `KS_YOUTUBE_CLIENTSECRET`    | ❌       | Your Google Cloud OAuth2 client secret           |
| `KS_YOUTUBE_REFRESHTOKEN`    | ❌       | OAuth2 refresh token (from one-time auth flow)   |
| `KS_YOUTUBE_EXCLUDECATEGORIES` | ❌     | Comma-separated category IDs to exclude (e.g., `10` for Music) |
| `KS_YOUTUBE_SCHEDULE`        | ❌       | Sync schedule in cron format (default: `@daily`) |

**Common YouTube Category IDs:**
- `10` - Music
- `1` - Film & Animation
- `20` - Gaming
- `22` - People & Blogs
- `24` - Entertainment
- `25` - News & Politics
- `27` - Education
- `28` - Science & Technology

See [YouTube Video Categories API](https://developers.google.com/youtube/v3/docs/videoCategories/list) for the full list.

To set up YouTube sync:

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project (or use an existing one)
3. Enable the **YouTube Data API v3** in [APIs & Services > Library](https://console.cloud.google.com/apis/library/youtube.googleapis.com)
4. Go to [APIs & Services > Credentials](https://console.cloud.google.com/apis/credentials)
5. Click "Create Credentials" > "OAuth client ID"
6. If prompted, configure the OAuth consent screen:
   - User Type: External (or Internal if using Google Workspace)
   - Add yourself as a test user
   - Add scope: `https://www.googleapis.com/auth/youtube.readonly`
7. For Application type, choose "Desktop app"
8. Note your **Client ID** and **Client Secret**

To obtain a refresh token, run this one-time auth flow:

```bash
# Replace with your client_id and client_secret
CLIENT_ID="your-client-id.apps.googleusercontent.com"
CLIENT_SECRET="your-client-secret"

# Step 1: Open this URL in your browser and authorize
echo "Open this URL in your browser:"
echo "https://accounts.google.com/o/oauth2/v2/auth?client_id=${CLIENT_ID}&redirect_uri=http://localhost:8080&response_type=code&scope=https://www.googleapis.com/auth/youtube.readonly&access_type=offline&prompt=consent"

# Step 2: After authorizing, you'll be redirected to localhost with a code parameter
# Copy the code from the URL (e.g., http://localhost:8080?code=4/0ABC...&scope=...)
# The page will fail to load (that's expected) - just copy the code

# Step 3: Exchange the code for tokens
AUTH_CODE="paste-your-auth-code-here"
curl -s -X POST https://oauth2.googleapis.com/token \
  -d "client_id=${CLIENT_ID}" \
  -d "client_secret=${CLIENT_SECRET}" \
  -d "code=${AUTH_CODE}" \
  -d "grant_type=authorization_code" \
  -d "redirect_uri=http://localhost:8080" | jq .

# The response will contain your refresh_token - save it!
```

YouTube liked videos will be synced to a list named `YouTube Liked` in your Karakeep instance.

YouTube sync will be skipped if any of `KS_YOUTUBE_CLIENTID`, `KS_YOUTUBE_CLIENTSECRET`, or `KS_YOUTUBE_REFRESHTOKEN` is not set.

## Deployment

Create a `docker-compose.yml` file with the following content:

```yaml
services:
  karakeep-sync:
    image: ghcr.io/sidoshi/karakeep-sync:latest
    container_name: karakeep-sync
    restart: unless-stopped
    environment:
      - KS_KARAKEEP_AUTH=<your_karakeep_auth_cookie> # required
      - KS_KARAKEEP_URL=<your_karakeep_instance_url> # required

      - KS_HN_AUTH=<your_hn_auth_cookie> # optional
      - KS_HN_SCHEDULE=@daily # optional Cron format, e.g., "@hourly", "@daily", "0 0 * * *" default is "@daily"

      - KS_REDDIT_CLIENTID=<your_reddit_client_id> # optional
      - KS_REDDIT_CLIENTSECRET=<your_reddit_client_secret> # optional
      - KS_REDDIT_REFRESHTOKEN=<your_reddit_refresh_token> # optional
      - KS_REDDIT_SCHEDULE=@daily # optional Cron format, e.g., "@hourly", "@daily", "0 0 * * *" default is "@daily"

      - KS_GITHUB_TOKEN=<your_github_personal_access_token> # optional
      - KS_GITHUB_SCHEDULE=@daily # optional Cron format, e.g., "@hourly", "@daily", "0 0 * * *" default is "@daily"

      - KS_PINBOARD_TOKEN=<your_pinboard_api_token> # optional
      - KS_PINBOARD_SCHEDULE=@daily # optional Cron format, e.g., "@hourly", "@daily", "0 0 * * *" default is "@daily"

      - KS_YOUTUBE_CLIENTID=<your_google_oauth_client_id> # optional
      - KS_YOUTUBE_CLIENTSECRET=<your_google_oauth_client_secret> # optional
      - KS_YOUTUBE_REFRESHTOKEN=<your_youtube_refresh_token> # optional
      - KS_YOUTUBE_EXCLUDECATEGORIES=10 # optional - exclude Music (category 10)
      - KS_YOUTUBE_SCHEDULE=@daily # optional Cron format, e.g., "@hourly", "@daily", "0 0 * * *" default is "@daily"
```

Then run:

```bash
docker-compose up -d
```

You can also add this service definition alongside your existing Hoarder/Karakeep services.

## Contributing

Contributions are welcome! Please open issues or pull requests for any features, bug fixes, or improvements.

To add support for more services, implement the `Plugin` trait in a new module under `crates/sync/src/plugin/`. You can refer to the existing `hn_upvotes` and `reddit_saves` modules as examples. All plugins must be registered in `crates/sync/src/plugin.rs`. Make sure to add appropriate configuration options in `crates/sync/src/settings.rs`. Finally, update the documentation in this README to include the new service.

See this PR for adding GitHub stars support as an example: [#2](https://github.com/sidoshi/karakeep-sync/pull/2)

## License

MIT License. See `LICENSE` file for details.
