# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] — 2026-06-06

Initial release. Full read-only access to the Ghost Content API.

### Added

#### Authentication (`src/auth/`)

- `ContentApiKey` — type-safe wrapper for 26-character hex Content API keys
  - Validation at construction time (length, hex-only characters)
  - Auto-normalises input: trims whitespace, lowercases uppercase hex
  - `as_str()`, `as_query_param()`, `is_valid()` accessors
  - Masked `Display` impl (shows first 8 and last 4 characters)

#### Error types (`src/error.rs`)

- `GhostError` enum backed by `thiserror`:
  - `Api { message, error_type, context }` — structured Ghost API errors
  - `Http(reqwest::Error)` — network and transport errors
  - `Json(serde_json::Error)` — serialisation / deserialisation errors
  - `Auth(String)` — authentication failures
- `Result<T>` type alias
- Constructor helpers: `GhostError::api()`, `GhostError::auth()`
- Predicate methods: `is_api_error()`, `is_http_error()`, `is_json_error()`, `is_auth_error()`

#### Models (`src/models/`)

- **`Post`** — full Ghost post resource with all API fields
  - `PostStatus` enum: `Draft`, `Published`, `Scheduled`, `Sent`
  - Helper methods: `is_published()`, `is_draft()`, `is_scheduled()`, `is_sent()`, `is_featured()`, `is_email_only()`
  - All optional fields use `#[serde(skip_serializing_if = "Option::is_none")]`
- **`Page`** — static page resource (mirrors Post schema)
  - `PageStatus` enum: `Draft`, `Published`, `Scheduled`
  - Helper methods: `is_published()`, `is_draft()`, `is_scheduled()`
- **`Tag`** — tag resource with SEO and code injection fields
  - `PostCount` struct (populated via `include=count.posts`)
  - `is_public()` helper
- **`Author`** — author resource with social and bio fields
  - Re-uses `PostCount` from the tag module
  - `has_profile_image()`, `post_count()` helpers
- **`Settings`** — read-only site settings (title, logo, nav, social, SEO)
  - `NavItem` struct for primary and secondary navigation
  - `has_logo()`, `nav_count()` helpers
- **`Pagination`** / **`Meta`** — browse endpoint pagination envelope
  - Helpers: `has_next()`, `has_prev()`, `is_first_page()`, `is_last_page()`, `items_on_page()`, `start_index()`, `end_index()`

#### Content API client (`src/client/content.rs`)

- `GhostContentClient` — async HTTP client using `reqwest`
  - Sets `Accept-Version: v5.0` on every request
  - Strips trailing slashes from the base URL
  - Parses Ghost's JSON error envelope (`{ "errors": [...] }`) into `GhostError::Api`
- **Posts**: `browse_posts()`, `read_post_by_id()`, `read_post_by_slug()`
- **Pages**: `browse_pages()`, `read_page_by_id()`, `read_page_by_slug()`
- **Tags**: `browse_tags()`, `read_tag_by_id()`, `read_tag_by_slug()`
- **Authors**: `browse_authors()`, `read_author_by_id()`, `read_author_by_slug()`
- **Tiers**: `browse_tiers()` with `Tier` model (free / paid membership tiers)
- **Settings**: `get_settings()`
- `BrowsePostsParams` struct (and type aliases for each resource) for optional filtering, sorting, pagination, field selection, and relation inclusion

#### Query parameter builder (`src/params/browse.rs`)

- `BrowseParams` — fluent builder for Ghost browse query parameters
  - Chainable setters: `limit()`, `page()`, `filter()`, `order()`, `include()`, `fields()`, `formats()`
  - Getter accessors for each field
  - `to_query_pairs()` → `Vec<(&'static str, String)>` for `reqwest`
  - `to_query_string()` with RFC 3986 percent-encoding

#### Examples (`examples/`)

- `list_posts` — pages through published posts, printing title, slug, and date
  - Configurable via `GHOST_URL` and `GHOST_CONTENT_KEY` environment variables
  - Falls back to `demo.ghost.io` when env vars are absent

### Fixed

- `Post::status` field now has `#[serde(default)]` so posts without an explicit
  `status` in the API response (which Ghost occasionally omits) deserialise
  correctly instead of returning a decoding error.

### Dependencies

- `reqwest 0.12` with `json` feature
- `serde 1` with `derive` feature
- `serde_json 1`
- `thiserror 1.0`
- `tokio 1` (dev/test only, with `rt-multi-thread` and `macros`)

[Unreleased]: https://github.com/arunkumar-mourougappane/ghost-io-api/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/arunkumar-mourougappane/ghost-io-api/releases/tag/v0.1.0
