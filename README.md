# Mastodon WSJ Bot

A Rust-based bot that automatically fetches RSS feeds from The Wall Street Journal (WSJ) and posts new articles to a Mastodon instance.

## Features

- **Multi-feed Monitoring**: Tracks multiple WSJ RSS feeds including:
  - Opinion
  - World News
  - U.S. Business
  - Market News
  - Technology (What's News)
  - Lifestyle
- **Deduplication**: Uses MongoDB to track posted articles and ensure no duplicates are sent.
- **Async Execution**: Built with `tokio` for efficient, non-blocking I/O.
- **Docker Ready**: Includes a `Dockerfile` and `docker-compose.yaml` for easy deployment.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (if running locally)
- [MongoDB](https://www.mongodb.com/) (for deduplication storage)
- A Mastodon account and an [Access Token](https://docs.joinmastodon.org/methods/apps/) with `write:statuses` scope.

## Configuration

1. Copy the `sample.env` to `.env`:
   ```bash
   cp sample.env .env
   ```
2. Fill in the required environment variables:
   - `MSTDN_ACCESS_TOKEN`: Your Mastodon API access token.
   - `MONGODB_URI`: Your MongoDB connection string (e.g., `mongodb://localhost:27017`).
   - `MONGODB_DBNAME`: The database name (Note: current implementation may use `scratchpad`).

> **Note**: The Mastodon instance URL is currently hardcoded in `src/main.rs`. You may need to modify the `toot` function to point to your specific instance.

## Usage

### Running Locally

```bash
cargo run --release
```

### Running with Docker

1. Build the image:
   ```bash
   docker build -t mastodon-wsj-bot .
   ```
2. Run with Docker Compose:
   ```bash
   docker-compose up -d
   ```

## Database Maintenance

To ensure the MongoDB collection doesn't grow indefinitely, it is recommended to create a TTL (Time To Live) index. You can run the following command in your MongoDB shell:

```javascript
db.scratchpad.createIndex(
    { "created_at": 1 },
    { 
        name: "mastodon-wsj-bot-clean-3mon",
        expireAfterSeconds: 60*60*24*93, // 3 months
        partialFilterExpression: {
            '_app_' : { $eq: "mastodon-wsj-bot" }
        },
    }
)
```

## License

This project is licensed under the terms of the license found in the repository (if applicable) or follows standard open-source conventions.
