# ghost-io-api

A strongly-typed, async Rust client for the [Ghost CMS](https://ghost.org/) API.

[![Crates.io](https://img.shields.io/crates/v/ghost-io-api.svg)](https://crates.io/crates/ghost-io-api)
[![Docs.rs](https://docs.rs/ghost-io-api/badge.svg)](https://docs.rs/ghost-io-api)
[![CI](https://github.com/arunkumar-mourougappane/ghost-io-api/actions/workflows/ci.yml/badge.svg)](https://github.com/arunkumar-mourougappane/ghost-io-api/actions/workflows/ci.yml)

> ⚠️ **v0.1.0 — Content API is stable. Admin API is planned for v0.2.0.**

## Overview

`ghost-io-api` is an ergonomic Rust interface to the Ghost Content API. It is designed to be:

- **Async-first** — built on `tokio` and `reqwest`
- **Strongly typed** — every request and response is a Rust struct with serde derives
- **Fluent** — a builder API for query parameters keeps call sites readable
- **Correct** — integration-tested against `demo.ghost.io`

## Installation

```toml
[dependencies]
ghost-io-api = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

```rust
use ghost_io_api::auth::content::ContentApiKey;
use ghost_io_api::client::content::{GhostContentClient, BrowsePostsParams};
use ghost_io_api::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let key = ContentApiKey::new("your-content-api-key")?;
    let client = GhostContentClient::new("https://your-site.ghost.io", key)?;

    // Browse posts
    let response = client.browse_posts(BrowsePostsParams::default()).await?;
    for post in &response.posts {
        println!("{} — {}", post.title, post.slug);
    }

    println!(
        "Page {}/{}, {} total",
        response.meta.pagination.page,
        response.meta.pagination.pages,
        response.meta.pagination.total,
    );

    Ok(())
}
```

## Features

### Content API (v0.1.0)

| Endpoint | Browse | Read by ID | Read by Slug |
|---|:---:|:---:|:---:|
| Posts | ✅ | ✅ | ✅ |
| Pages | ✅ | ✅ | ✅ |
| Tags | ✅ | ✅ | ✅ |
| Authors | ✅ | ✅ | ✅ |
| Tiers | ✅ | — | — |
| Settings | ✅ | — | — |

### Admin API

| Feature | Status |
|---|---|
| JWT authentication | 🔜 v0.2.0 |
| Posts CRUD | 🔜 v0.2.0 |
| Members | 🔜 v0.6.0 |

## Usage

### Authentication

Content API keys are 26-character hexadecimal strings found under **Ghost Admin → Integrations → Content API**.

```rust
use ghost_io_api::auth::content::ContentApiKey;

let key = ContentApiKey::new("22444f78447824223cefc48062")?;
```

### Creating a client

```rust
use ghost_io_api::client::content::GhostContentClient;

let client = GhostContentClient::new("https://demo.ghost.io", key)?;
```

Trailing slashes on the URL are stripped automatically. The client sets `Accept-Version: v5.0` on every request.

### Browsing posts

```rust
use ghost_io_api::client::content::BrowsePostsParams;

let response = client.browse_posts(BrowsePostsParams {
    limit: Some(10),
    filter: Some("featured:true".to_string()),
    order: Some("published_at DESC".to_string()),
    include: Some("authors,tags".to_string()),
    ..Default::default()
}).await?;
```

### Fluent query builder

Use [`BrowseParams`](src/params/browse.rs) for a chainable builder interface:

```rust
use ghost_io_api::params::browse::BrowseParams;
use ghost_io_api::client::content::BrowsePostsParams;

let browse = BrowseParams::new()
    .limit(10)
    .page(2)
    .filter("featured:true")
    .order("published_at DESC")
    .include("authors,tags");

let params = BrowsePostsParams {
    limit: browse.get_limit(),
    page: browse.get_page(),
    filter: browse.get_filter().map(str::to_string),
    order: browse.get_order().map(str::to_string),
    include: browse.get_include().map(str::to_string),
    ..Default::default()
};

let response = client.browse_posts(params).await?;
```

### Reading a single resource

```rust
// By ID
let post = client.read_post_by_id("5ddc9141c35e7700383b2937", None).await?;

// By slug, with relations
let post = client.read_post_by_slug("welcome", Some("authors,tags")).await?;

// Pages, tags, authors follow the same pattern
let page   = client.read_page_by_slug("about", None).await?;
let tag    = client.read_tag_by_slug("getting-started", None).await?;
let author = client.read_author_by_slug("ghost", Some("count.posts")).await?;
```

### Pagination

```rust
let pagination = &response.meta.pagination;

println!("Page {}/{}", pagination.page, pagination.pages);
println!(
    "Showing items {}-{} of {}",
    pagination.start_index() + 1,
    pagination.end_index(),
    pagination.total,
);

if pagination.has_next() {
    // fetch next page …
}
```

### Site settings

```rust
let settings = client.get_settings().await?;
println!("Site: {}", settings.title.unwrap_or_default());
println!("Nav items: {}", settings.nav_count());
```

### Error handling

```rust
use ghost_io_api::error::GhostError;

match client.read_post_by_id("bad-id", None).await {
    Ok(post) => println!("{}", post.title),
    Err(GhostError::Api { message, error_type, .. }) => {
        eprintln!("API error [{error_type}]: {message}");
    }
    Err(GhostError::Http(e)) => eprintln!("Network error: {e}"),
    Err(e) => eprintln!("Other error: {e}"),
}
```

## Examples

### list_posts

Pages through all published posts and prints title, slug, and publication date.

```sh
# Uses demo.ghost.io by default
cargo run --example list_posts

# Point at your own site
GHOST_URL=https://your-site.ghost.io \
GHOST_CONTENT_KEY=your-content-api-key \
cargo run --example list_posts
```

## Running Tests

```sh
# Unit + doc tests (no network required)
cargo test

# Integration tests (hits demo.ghost.io)
cargo test --features integration-tests
```

## Module Structure

```
src/
├── auth/
│   └── content.rs      # ContentApiKey — validation and query param injection
├── client/
│   └── content.rs      # GhostContentClient — all Content API endpoints
├── error.rs            # GhostError enum (API / HTTP / JSON / Auth)
├── models/
│   ├── author.rs       # Author struct
│   ├── page.rs         # Page struct + PageStatus enum
│   ├── pagination.rs   # Meta + Pagination structs
│   ├── post.rs         # Post struct + PostStatus enum
│   ├── settings.rs     # Settings struct + NavItem
│   └── tag.rs          # Tag struct + PostCount
└── params/
    └── browse.rs       # BrowseParams fluent builder
```

## License

MIT © 2026 [Arunkumar Mourougappane](https://github.com/arunkumar-mourougappane)
