# Release Notes — v0.1.0

**Released:** 2026-06-06

## Overview

`ghost-io-api` v0.1.0 delivers the first stable release of the Ghost Content API
client for Rust. The crate provides complete read-only access to all Ghost Content
API endpoints with strong typing, async ergonomics, and integration-tested
correctness.

## What's Included

### Ghost Content API — full endpoint coverage

Every Content API endpoint is implemented and integration-tested against
[demo.ghost.io](https://demo.ghost.io):

| Resource | Browse | Read by ID | Read by Slug |
|---|:---:|:---:|:---:|
| Posts | ✅ | ✅ | ✅ |
| Pages | ✅ | ✅ | ✅ |
| Tags | ✅ | ✅ | ✅ |
| Authors | ✅ | ✅ | ✅ |
| Tiers | ✅ | — | — |
| Settings | ✅ | — | — |

### Strongly typed models

All Ghost resources are modelled as Rust structs with serde derives:

- `Post` + `PostStatus` (Draft / Published / Scheduled / Sent)
- `Page` + `PageStatus` (Draft / Published / Scheduled)
- `Tag` with optional `PostCount`
- `Author` with optional `PostCount`
- `Settings` with `NavItem` navigation
- `Pagination` / `Meta` for browse responses
- `Tier` for membership tiers

### Fluent `BrowseParams` builder

```rust
let params = BrowseParams::new()
    .limit(10)
    .filter("featured:true")
    .order("published_at DESC")
    .include("authors,tags");
```

### Type-safe authentication

`ContentApiKey` validates the 26-character hex key at construction time,
normalises casing, and injects the key as a query parameter automatically.

### Structured error handling

`GhostError` covers all failure modes — Ghost API errors (with `error_type` and
`context`), HTTP / network errors, and JSON decoding errors — making it easy to
handle each case precisely.

### Working example

`examples/list_posts` demonstrates paginated browsing with the fluent builder.
Run it against any Ghost site:

```sh
GHOST_URL=https://your-site.ghost.io \
GHOST_CONTENT_KEY=your-key \
cargo run --example list_posts
```

### API documentation published to GitHub Pages

Documentation is built and deployed automatically on every push to `main`
and is available at:

**<https://arunkumar-mourougappane.github.io/ghost-io-api/ghost_io_api/>**

The documentation workflow runs under strict lint flags (`-D missing_docs`,
`-D warnings`, `-D rustdoc::redundant_explicit_links`), ensuring every public
item is documented and all intra-doc links resolve correctly.

## Bug Fixes

- **`Post::status` deserialization** — added `#[serde(default)]` so posts where
  Ghost omits the `status` field in the API response (observed on `demo.ghost.io`)
  deserialise correctly, defaulting to `PostStatus::Draft` instead of returning
  a JSON decoding error.

## Testing

- **149 unit tests** across all modules
- **67 doc-tests** — every public API example in the documentation is compiled
  and executed as part of `cargo test`
- **10 integration tests** run against the live Ghost demo site
  (`cargo test --features integration-tests`)

All tests pass clean. Clippy (`-D warnings`) and `rustfmt` checks are enforced
in CI on Ubuntu, macOS, and Windows.

## Upgrade Guide

This is the initial release — no migration is required.

## What's Next (v0.2.0)

- JWT-based Admin API authentication
- Admin API: create, update, and delete posts
- `GhostAdminClient` with full CRUD

See the [open issues](https://github.com/arunkumar-mourougappane/ghost-io-api/issues)
for the full roadmap.
