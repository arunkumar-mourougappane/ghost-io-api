# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] — 2026-06-21

### Added

#### Admin API authentication (`src/auth/admin.rs`) — #17

- `AdminApiKey` — type-safe wrapper for Ghost Admin API key pairs (`{id}:{hex-secret}`)
  - Parses and validates the colon-separated `kid:hex-secret` format at construction time
  - `generate_jwt()` — produces a short-lived (5-minute) HS256 JWT signed with the hex
    secret; freshly minted on every request so tokens never expire mid-flight
  - `sign_jwt(iat: u64)` — deterministic variant for testing with a fixed timestamp
  - JWT header: `{"alg":"HS256","kid":"<id>","typ":"JWT"}` (alphabetical field order)
  - JWT payload: `{"exp":<iat+300>,"iat":<iat>}`
  - Pure RustCrypto stack: `hmac 0.12` + `sha2 0.10` + `base64 0.22` (no OpenSSL)
  - Masked `Display` impl — `AdminApiKey(kid:***)` keeps secrets out of logs
  - `key_id()`, `is_valid()`, `AsRef<str>` accessors

#### Admin API client (`src/client/admin.rs`) — #18, #21

- `GhostAdminClient` — write-capable async HTTP client for the Ghost Admin API
  - `new(base_url, api_key)` — strips trailing slashes; sets `Accept-Version: v5.0`
  - Injects `Authorization: Ghost <jwt>` on every request with a freshly generated token
  - Parses Ghost's `{"errors":[{"message","type","context"}]}` error envelope into
    `GhostError::Api`
- **Posts CRUD** (6 public methods):
  - `browse_posts(AdminBrowsePostsParams)` → `AdminPostsResponse` — paginated list;
    returns all statuses (draft, scheduled, published) unlike the Content API
  - `read_post_by_id(id, include?)` → `Post`
  - `read_post_by_slug(slug, include?)` → `Post`
  - `create_post(PostCreate)` → `Post`
  - `update_post(id, PostUpdate)` → `Post`
  - `delete_post(id)` → `()`
- `AdminBrowsePostsParams` — query parameter struct: `page`, `limit`, `filter` (NQL),
  `order`, `include`, `fields`; all optional, omitted fields use Ghost's defaults
- `AdminPostsResponse` — public browse response carrying `posts: Vec<Post>` + `meta: Meta`

#### Post write models (`src/models/post.rs`) — #19, #20

- `PostCreate` — input model for `POST /ghost/api/admin/posts/`
  - `title` is the only required field; all others are `Option<_>` and skipped when `None`
  - Fields: content (`lexical`, `html`, `custom_excerpt`), identifiers (`slug`),
    status & visibility, scheduling (`published_at`), media (`feature_image`, alt, caption),
    relationships (`tags: Vec<TagRef>`, `authors: Vec<AuthorRef>`, `newsletter`),
    SEO, Open Graph, Twitter Cards, code injection, custom template
  - `PostCreate::new(title)` constructor
  - `PostCreateEnvelope::new(post)` — wraps in `{"posts":[…]}` as the API requires
- `PostUpdate` — input model for `PUT /ghost/api/admin/posts/{id}/`
  - `updated_at: String` is **required** — Ghost's optimistic-concurrency token; the
    server rejects stale values with `ConflictError` to prevent silent overwrites
  - All other fields are `Option<_>`, identical set to `PostCreate` minus `title`
  - `PostUpdate::new(updated_at)` constructor
  - `PostUpdateEnvelope::new(post)` — wraps in `{"posts":[…]}`
- `TagRef` — lightweight tag reference for create/update: `by_slug()` or `by_name()`
- `AuthorRef` — author reference: `by_id()` or `by_email()`

#### Generic response envelopes (`src/models/envelope.rs`) — #22

- `BrowseEnvelope<T>` — generic browse response envelope
  - Deserializes `{"<resource>": [...], "meta": {...}}` into `items: Vec<T>` + `meta: Meta`
  - Custom `Deserialize` impl locates the resource array by scanning for the first
    array-valued field (skipping `"meta"`); works for any resource key
- `SingleEnvelope<T>` — generic single-item response envelope
  - Deserializes `{"<resource>": [item]}` into `item: T`
  - Same key-detection strategy; tolerates an optional `"meta"` field

#### Examples (`examples/`) — #23

- `publish_post` — full Admin API authoring workflow:
  1. Create a timestamped draft post with a custom excerpt and tag
  2. Publish it by calling `update_post` with `status: PostStatus::Published` and the
     `updated_at` token from the create response
  3. Delete the post (cleanup so the example is safe to run repeatedly)
  - Configurable via `GHOST_URL` and `GHOST_ADMIN_KEY` environment variables (required)

#### Documentation — #82

- `RELEASE_NOTES.md` — replaced with v0.2.0 release notes
- `docs/release_notes/v0.1.0.md` — archived v0.1.0 release notes
- `README.md` — added Admin API section: key format, client construction, all six CRUD
  methods, optimistic concurrency explanation, `publish_post` example invocation, and
  updated capability matrix
- `src/lib.rs` — updated crate-level docs: Admin API quick-start example, module
  description for `auth::admin` and `client::admin`

### Dependencies

- `hmac = "0.12"` — HMAC-SHA256 for JWT signing
- `sha2 = "0.10"` — SHA-256 digest
- `base64 = "0.22"` — base64url encoding for JWT segments
- `wiremock = "0.6"` (dev) — HTTP mock server for client unit tests

---

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
- **Tiers**: `browse_tiers()` with inline `Tier` model (free / paid membership tiers)
- **Settings**: `get_settings()`
- `BrowsePostsParams` struct (and type aliases for each resource) for optional
  filtering, sorting, pagination, field selection, and relation inclusion

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

#### CI / CD (`.github/workflows/`)

- `docs.yml` — builds and deploys API documentation to GitHub Pages on every
  push to `main`; strict lint flags (`-D warnings`, `-D missing_docs`,
  `-D rustdoc::redundant_explicit_links`)

### Fixed

- `Post::status` field: added `#[serde(default)]` so posts where the Ghost API
  omits the `status` key deserialise correctly (defaulting to `PostStatus::Draft`)
  instead of returning a JSON decoding error.

### Documentation

- `lib.rs`: replaced placeholder crate-level docs with accurate module
  cross-links and a working `no_run` quick-start example
- `README.md`: rewrote with real usage examples, feature table, running-tests
  instructions, and correct badge set

### Dependencies

- `reqwest 0.12` with `json` feature
- `serde 1` with `derive` feature
- `serde_json 1`
- `thiserror 1.0`
- `tokio 1` (dev/test only, with `rt-multi-thread` and `macros`)

[Unreleased]: https://github.com/arunkumar-mourougappane/ghost-io-api/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/arunkumar-mourougappane/ghost-io-api/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/arunkumar-mourougappane/ghost-io-api/releases/tag/v0.1.0
