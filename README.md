# Ghost.io API

A strongly-typed, async Rust client for the [Ghost CMS](https://ghost.org/) API.

[![Crates.io](https://img.shields.io/crates/v/ghost-io-api.svg)](https://crates.io/crates/ghost-io-api)
[![Docs.rs](https://docs.rs/ghost-io-api/badge.svg)](https://docs.rs/ghost-io-api)
[![CI](https://github.com/arunkumar-mourougappane/ghost-io-api/actions/workflows/ci.yml/badge.svg)](https://github.com/arunkumar-mourougappane/ghost-io-api/actions/workflows/ci.yml)
[![Docs](https://github.com/arunkumar-mourougappane/ghost-io-api/actions/workflows/docs.yml/badge.svg)](https://github.com/arunkumar-mourougappane/ghost-io-api/actions/workflows/docs.yml)

> **v0.2.0** — Content API (stable) + Admin API Posts CRUD (new in v0.2.0)

## Overview

`ghost-io-api` is an ergonomic Rust interface to both the Ghost Content API
and the Ghost Admin API. It is designed to be:

- **Async-first** — built on `tokio` and `reqwest`
- **Strongly typed** — every request and response is a Rust struct with serde derives
- **Secure** — Admin API requests carry freshly-minted HS256 JWTs; no long-lived tokens
- **Fluent** — a builder API for query parameters keeps call sites readable
- **Correct** — integration-tested against `demo.ghost.io`

## Installation

```toml
[dependencies]
ghost-io-api = "0.2"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

---

## Content API

### Quick Start

```rust
use ghost_io_api::auth::content::ContentApiKey;
use ghost_io_api::client::content::{GhostContentClient, BrowsePostsParams};
use ghost_io_api::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let key = ContentApiKey::new("your-content-api-key")?;
    let client = GhostContentClient::new("https://your-site.ghost.io", key)?;

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

Content API keys are 26-character hex strings found under
**Ghost Admin → Settings → Integrations → Content API**.

### Endpoint Coverage

| Resource | Browse | Read by ID | Read by Slug |
|---|:---:|:---:|:---:|
| Posts | ✅ | ✅ | ✅ |
| Pages | ✅ | ✅ | ✅ |
| Tags | ✅ | ✅ | ✅ |
| Authors | ✅ | ✅ | ✅ |
| Tiers | ✅ | — | — |
| Settings | ✅ | — | — |

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
```

### Reading a single resource

```rust
// By ID
let post = client.read_post_by_id("5ddc9141c35e7700383b2937", None).await?;

// By slug, with relations embedded
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

---

## Admin API *(new in v0.2.0)*

The Admin API provides write access to your Ghost installation. Every request
carries a freshly-signed HS256 JWT so tokens never sit long enough to expire.

### Authentication

Admin API keys take the form `{id}:{hex-secret}`. Obtain one under
**Ghost Admin → Settings → Integrations → Add custom integration**.

```rust
use ghost_io_api::auth::admin::AdminApiKey;

let key = AdminApiKey::new(
    "6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1"
)?;
```

### Creating a client

```rust
use ghost_io_api::client::admin::GhostAdminClient;

let client = GhostAdminClient::new("https://your-site.ghost.io", key)?;
```

### Browsing posts

The Admin API returns posts of **all statuses** (draft, scheduled, published),
unlike the Content API which only returns published posts.

```rust
use ghost_io_api::client::admin::AdminBrowsePostsParams;

let response = client.browse_posts(AdminBrowsePostsParams {
    limit: Some(10),
    filter: Some("status:draft".to_string()),
    order: Some("updated_at DESC".to_string()),
    ..Default::default()
}).await?;

for post in &response.posts {
    println!("[{}] {}", post.status, post.title);
}
```

### Reading a single post

```rust
// By Ghost ID
let post = client.read_post_by_id("5ddc9141c35e7700383b2937", None).await?;

// By slug, with authors and tags embedded
let post = client.read_post_by_slug("my-draft", Some("authors,tags")).await?;
```

### Creating a post

```rust
use ghost_io_api::models::post::{PostCreate, PostStatus, TagRef, AuthorRef};

let post = client.create_post(PostCreate {
    title: "Hello, Ghost!".to_string(),
    status: Some(PostStatus::Draft),
    custom_excerpt: Some("A brief summary.".to_string()),
    tags: Some(vec![
        TagRef::by_slug("rust"),       // resolves existing tag by slug
        TagRef::by_name("New Topic"),  // creates tag if it doesn't exist
    ]),
    authors: Some(vec![AuthorRef::by_email("you@example.com")]),
    ..Default::default()
}).await?;

println!("Created post: {} ({})", post.title, post.id);
```

### Updating a post

Ghost uses **optimistic concurrency**: every update must include the
`updated_at` timestamp from the most recent read. If the value is stale
(someone else edited the post), Ghost returns a `409 ConflictError`.

```rust
use ghost_io_api::models::post::{PostUpdate, PostStatus};

// Read first to get the current updated_at token
let existing = client.read_post_by_id("5ddc9141c35e7700383b2937", None).await?;

let updated = client.update_post(&existing.id, PostUpdate {
    updated_at: existing.updated_at.unwrap(),
    status: Some(PostStatus::Published),
    title: Some("My Published Post".to_string()),
    ..Default::default()
}).await?;

println!("Published: {}", updated.url.unwrap_or_default());
```

### Deleting a post

```rust
client.delete_post("5ddc9141c35e7700383b2937").await?;
```

### Posts CRUD reference

| Method | Description |
|---|---|
| `browse_posts(params)` | List posts — all statuses, optional NQL filter |
| `read_post_by_id(id, include?)` | Read one post by Ghost ID |
| `read_post_by_slug(slug, include?)` | Read one post by slug |
| `create_post(post)` | Create a new post |
| `update_post(id, update)` | Update an existing post |
| `delete_post(id)` | Permanently delete a post |

---

## Error Handling

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

---

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

### publish_post *(new in v0.2.0)*

Demonstrates the full Admin API authoring workflow: create a draft, publish it,
then delete it (so the example is safe to run repeatedly).

```sh
GHOST_URL=https://your-site.ghost.io \
GHOST_ADMIN_KEY=<id>:<hex-secret> \
cargo run --example publish_post
```

---

## Running Tests

```sh
# Unit + doc tests (no network required)
cargo test

# Integration tests (hits demo.ghost.io)
cargo test --features integration-tests
```

---

## License

MIT © 2026 [Arunkumar Mourougappane](https://github.com/arunkumar-mourougappane)
