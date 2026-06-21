# Release Notes — v0.2.0

**Released:** 2026-06-21

## Overview

`ghost-io-api` v0.2.0 adds complete **Ghost Admin API** support for posts.
The crate now covers the full post lifecycle — create a draft, update its
content, publish it, and delete it — all from strongly-typed async Rust code,
authenticated with freshly-minted HS256 JWTs on every request.

For the full history see [`CHANGELOG.md`](CHANGELOG.md).  
Previous release notes are archived in [`docs/release_notes/`](docs/release_notes/).

---

## What's New

### Admin API authentication — `AdminApiKey` (#17)

Ghost Admin API keys take the form `{id}:{hex-secret}`. `AdminApiKey` parses,
validates, and stores this key pair. On every request the client calls
`generate_jwt()` which produces a short-lived (5-minute) HS256 JWT signed with
the hex secret and sent as `Authorization: Ghost <token>`.

```rust
use ghost_io_api::auth::admin::AdminApiKey;

let key = AdminApiKey::new(
    "6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1"
)?;
```

Keys are found under **Ghost Admin → Settings → Integrations → Add custom
integration**. The `Display` impl masks the secret (`AdminApiKey(kid:***)`).

**Implementation:** pure RustCrypto stack — `hmac 0.12`, `sha2 0.10`,
`base64 0.22`. No OpenSSL dependency.

---

### `GhostAdminClient` — write-capable HTTP client (#18)

```rust
use ghost_io_api::client::admin::GhostAdminClient;

let client = GhostAdminClient::new("https://your-site.ghost.io", key)?;
```

The client injects `Accept-Version: v5.0` on every request, strips trailing
slashes from the base URL, and generates a fresh JWT for each call so tokens
never sit long enough to expire.

---

### Post write models (#19, #20)

**`PostCreate`** — input model for `POST /ghost/api/admin/posts/`:

```rust
use ghost_io_api::models::post::{PostCreate, PostStatus, TagRef};

let post = PostCreate {
    title: "Hello, Ghost!".to_string(),
    status: Some(PostStatus::Published),
    tags: Some(vec![TagRef::by_slug("rust"), TagRef::by_name("New Tag")]),
    ..Default::default()
};
```

**`PostUpdate`** — input model for `PUT /ghost/api/admin/posts/{id}/`.
The `updated_at` field is **required** as Ghost's optimistic-concurrency token:
the server rejects any update where the value is stale, protecting against
silent overwrite of concurrent edits.

```rust
use ghost_io_api::models::post::PostUpdate;

let update = PostUpdate {
    updated_at: existing_post.updated_at.unwrap(),
    title: Some("Updated Title".to_string()),
    ..Default::default()
};
```

Both types serialise only their set fields (`skip_serializing_if = "Option::is_none"`),
keeping request payloads minimal.

---

### Admin Posts CRUD (#21)

Six public methods on `GhostAdminClient`:

| Method | HTTP | Description |
|---|---|---|
| `browse_posts(params)` | `GET /posts/` | Paginated list, all statuses |
| `read_post_by_id(id, include?)` | `GET /posts/{id}/` | Read by Ghost ID |
| `read_post_by_slug(slug, include?)` | `GET /posts/slug/{slug}/` | Read by slug |
| `create_post(post)` | `POST /posts/` | Create new post |
| `update_post(id, update)` | `PUT /posts/{id}/` | Update existing post |
| `delete_post(id)` | `DELETE /posts/{id}/` | Permanently remove post |

`AdminBrowsePostsParams` supports `page`, `limit`, `filter` (NQL), `order`,
`include`, and `fields`. The response type `AdminPostsResponse` carries
`posts: Vec<Post>` and `meta: Meta` for pagination.

---

### Generic response envelopes (#22)

Two reusable types in `models::envelope` eliminate the boilerplate of writing
per-resource envelope structs:

**`BrowseEnvelope<T>`** — deserializes `{"<resource>": [...], "meta": {...}}`.
Works for posts, pages, tags, authors — any resource key is detected
automatically by scanning for the first array-valued field.

**`SingleEnvelope<T>`** — deserializes `{"<resource>": [item]}`. Used
internally by read, create, and update methods to extract the single item.

```rust
use ghost_io_api::models::envelope::BrowseEnvelope;
use ghost_io_api::models::post::Post;

// Deserializes {"posts": [...], "meta": {...}} regardless of the "posts" key
let env: BrowseEnvelope<Post> = serde_json::from_str(&json_body)?;
println!("{} posts, page {}", env.items.len(), env.meta.pagination.page);
```

---

### `publish_post` example (#23)

`examples/publish_post.rs` demonstrates the full create-then-publish workflow:

```sh
GHOST_URL=https://your-site.ghost.io \
GHOST_ADMIN_KEY=<id>:<hex-secret> \
cargo run --example publish_post
```

The example creates a draft, publishes it by updating its status, then deletes
the post so it can be run repeatedly without cluttering the site.

---

## Testing

- **270 unit tests** across all modules (up from 149 in v0.1.0)
- **99 doc-tests** — every public API example in the documentation is compiled
  and executed (up from 67)
- All tests pass clean. Clippy (`-D warnings`) and `rustfmt` checks enforced in CI.

---

## Upgrade Guide

### From v0.1.0

No breaking changes. All v0.1.0 Content API types and methods are unchanged.

Add the crate to your `Cargo.toml`:

```toml
ghost-io-api = "0.2"
```

To use the Admin API, create an `AdminApiKey` from your Ghost integration key
and pass it to `GhostAdminClient::new()`. See the quick-start in the README.

---

## What's Next (v0.3.0)

- Media upload — image and file attachment to posts
- Ghost's `/ghost/api/admin/images/upload/` endpoint

See the [open issues](https://github.com/arunkumar-mourougappane/ghost-io-api/issues)
for the full roadmap.
