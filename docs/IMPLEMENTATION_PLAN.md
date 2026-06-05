# Implementation Plan - GitHub Issues

## Overview

This document maps the 58 features across 8 milestones (versions) with detailed implementation guidance.

## How to Use This Plan

1. Features are organized by milestone/version
2. Each feature has dependencies tracked
3. Create GitHub issues using the templates below
4. Link dependencies using "Depends on #N" in issue descriptions

---

## Milestone 1: v0.1.0 - Foundation & Content API

**Target:** Week 3 | **Alpha Release** | **9 Features**

### Issue 1: Error Types - GhostError enum
**Labels:** `enhancement`, `foundation`, `v0.1.0`

**Description:**
Implement the core error handling type for the crate using `thiserror`.

**Tasks:**
- [ ] Create `GhostError` enum with variants: `Api`, `Http`, `Json`, `Auth`
- [ ] Use `#[error(...)]` attributes for descriptive messages
- [ ] Implement `From` traits for reqwest and serde_json errors
- [ ] Add tests for error conversions
- [ ] Document with examples

**Module:** `src/error.rs`

**Example:**
```rust
pub type Result<T> = std::result::Result<T, GhostError>;
```

**References:**
- `docs/design/rust-client-architecture.md` (Error Handling section)

---

### Issue 2: Pagination Models
**Labels:** `enhancement`, `models`, `v0.1.0`

**Description:**
Implement pagination support structures for browse endpoints.

**Tasks:**
- [ ] Create `Meta` struct with `pagination` field
- [ ] Create `Pagination` struct with page/limit/pages/total/next/prev
- [ ] Add serde derives
- [ ] Test deserialization from Ghost API responses
- [ ] Document fields

**Module:** `src/models/pagination.rs`

**Example:**
```rust
meta.pagination.next // Some(2) or None
```

**References:**
- `docs/api-reference/ghost-api-overview.md` (Response Format)

---

### Issue 3: Post Model
**Labels:** `enhancement`, `models`, `v0.1.0`

**Description:**
Implement the core Post struct with all Ghost API fields.

**Tasks:**
- [ ] Create `Post` struct with all fields (id, title, slug, html, lexical, status, etc.)
- [ ] Create `PostStatus` enum (Draft, Published, Scheduled, Sent)
- [ ] Add serde derives with proper field renaming
- [ ] Test deserialization
- [ ] Document all fields

**Module:** `src/models/post.rs`

**Example:**
```rust
Post { id, title, slug, status: PostStatus::Published, ... }
```

**References:**
- `docs/api-reference/admin-api.md` (Post Object Schema)

---

### Issue 4: Content Auth - ContentApiKey
**Labels:** `enhancement`, `auth`, `v0.1.0`

**Description:**
Implement authentication wrapper for Content API key.

**Tasks:**
- [ ] Create `ContentApiKey` struct wrapping API key string
- [ ] Implement query parameter injection (?key=...)
- [ ] Add key format validation
- [ ] Add tests

**Module:** `src/auth/content.rs`

**Example:**
```rust
ContentApiKey::new("22444f78447824223cefc48062")
```

---

### Issue 5: Content Client - GhostContentClient
**Labels:** `enhancement`, `client`, `v0.1.0`
**Depends on:** #1 (Error Types), #3 (Post Model), #4 (Content Auth)

**Description:**
Implement the read-only Content API client with reqwest.

**Tasks:**
- [ ] Create `GhostContentClient` struct with base_url, api_key, reqwest client
- [ ] Implement `browse_posts(params)` → `Result<Vec<Post>>`
- [ ] Implement `read_post_by_id(id)` → `Result<Post>`
- [ ] Implement `read_post_by_slug(slug)` → `Result<Post>`
- [ ] Add `Accept-Version: v6.0` header
- [ ] Parse response envelope
- [ ] Add integration tests (demo.ghost.io or mock)

**Module:** `src/client/content.rs`

**Example:**
```rust
let client = GhostContentClient::new("https://demo.ghost.io", api_key)?;
let posts = client.browse_posts(params).await?;
```

**References:**
- `docs/api-reference/ghost-api-overview.md` (Content API)

---

### Issue 6: Browse Params Builder
**Labels:** `enhancement`, `params`, `v0.1.0`

**Description:**
Implement fluent builder for browse endpoint query parameters.

**Tasks:**
- [ ] Create `BrowseParams` struct
- [ ] Implement builder methods: filter, limit, page, order, include, fields, formats
- [ ] Serialize to query string
- [ ] Add tests for all parameters
- [ ] Document each method

**Module:** `src/params/browse.rs`

**Example:**
```rust
BrowseParams::new()
  .limit(10)
  .filter("featured:true")
  .include(&["tags", "authors"])
```

---

### Issue 7: Basic Models
**Labels:** `enhancement`, `models`, `v0.1.0`

**Description:**
Implement Page, Tag, Author, Settings structs with serde.

**Tasks:**
- [ ] Create `Page` struct (similar to Post)
- [ ] Create `Tag` struct (id, name, slug, description, etc.)
- [ ] Create `Author` struct (id, name, slug, profile_image, etc.)
- [ ] Create `Settings` struct (title, description, logo, etc.)
- [ ] Add serde derives
- [ ] Test deserialization
- [ ] Document fields

**Modules:** `src/models/page.rs`, `src/models/tag.rs`, `src/models/author.rs`, `src/models/settings.rs`

**Example:**
```rust
Tag { id, name, slug, description, ... }
```

---

### Issue 8: Extended Content Client
**Labels:** `enhancement`, `client`, `v0.1.0`
**Depends on:** #5 (Content Client), #7 (Basic Models)

**Description:**
Add all remaining Content API endpoints to GhostContentClient.

**Tasks:**
- [ ] Implement browse_pages, read_page_by_id, read_page_by_slug
- [ ] Implement browse_tags, read_tag_by_id, read_tag_by_slug
- [ ] Implement browse_authors, read_author_by_id, read_author_by_slug
- [ ] Implement browse_tiers
- [ ] Implement get_settings
- [ ] Add tests for each endpoint

**Module:** `src/client/content.rs`

**Example:**
```rust
client.browse_tags(params).await?
```

---

### Issue 9: list-posts Example
**Labels:** `example`, `v0.1.0`
**Depends on:** #5 (Content Client), #6 (Browse Params Builder)

**Description:**
Create working example demonstrating Content API + pagination.

**Tasks:**
- [ ] Create `examples/list_posts.rs`
- [ ] Parse CLI args (url, key, tag, limit, page)
- [ ] Call browse_posts with params
- [ ] Format output with post titles, dates, tags
- [ ] Show pagination info
- [ ] Add README example usage

**Module:** `examples/list_posts.rs`

**Example:**
```bash
cargo run --example list_posts -- --url https://demo.ghost.io --key YOUR_KEY
```

---

## Milestone 2: v0.2.0 - Admin API - Core CRUD

**Target:** Week 6 | **Alpha Release** | **7 Features**

### Issue 10: JWT Generation
**Labels:** `enhancement`, `auth`, `v0.2.0`

**Description:**
Implement hand-rolled HS256 JWT signing for Admin API.

**Tasks:**
- [ ] Parse Admin API key format `{key_id}:{hex_secret}`
- [ ] Decode hex secret to bytes
- [ ] Generate JWT header with `alg`, `typ`, `kid`
- [ ] Generate JWT payload with `iat`, `exp`, `aud`
- [ ] HMAC-SHA256 signature
- [ ] Base64url encode (no padding)
- [ ] Concatenate to form JWT
- [ ] Handle 5-minute expiry
- [ ] Add tests

**Module:** `src/auth/admin.rs`

**Example:**
```rust
let jwt = admin_key.generate_jwt()?;
// Authorization: Ghost {jwt}
```

**References:**
- `docs/design/rust-client-architecture.md` (JWT Generation section)
- `docs/api-reference/ghost-api-overview.md` (Token Generation)

---

### Issue 11: Admin Client Core
**Labels:** `enhancement`, `client`, `v0.2.0`
**Depends on:** #10 (JWT Generation)

**Description:**
Implement GhostAdminClient with JWT auth header injection.

**Tasks:**
- [ ] Create `GhostAdminClient` struct
- [ ] Store base_url, AdminApiKey, reqwest client
- [ ] Generate fresh JWT on each request
- [ ] Inject `Authorization: Ghost {jwt}` header
- [ ] Inject `Accept-Version: v6.0` header
- [ ] Inject `Content-Type: application/json` for POST/PUT
- [ ] Add tests

**Module:** `src/client/admin.rs`

**Example:**
```rust
GhostAdminClient::new("https://myblog.ghost.io", admin_key)?
```

---

### Issue 12: PostCreate Model
**Labels:** `enhancement`, `models`, `v0.2.0`

**Description:**
Create PostCreate struct for creating new posts.

**Tasks:**
- [ ] Define `PostCreate` struct with optional fields
- [ ] Include: title, slug, lexical, html, status, tags, authors, feature_image, etc.
- [ ] Add serde derives
- [ ] Add builder pattern or impl Default
- [ ] Document fields
- [ ] Add tests

**Module:** `src/models/post.rs`

**Example:**
```rust
PostCreate { 
  title: "Hello".into(), 
  status: PostStatus::Draft,
  lexical: Some(doc),
  ..Default::default()
}
```

---

### Issue 13: PostUpdate Model
**Labels:** `enhancement`, `models`, `v0.2.0`

**Description:**
Create PostUpdate struct with required updated_at for collision detection.

**Tasks:**
- [ ] Define `PostUpdate` struct
- [ ] Make `updated_at` a REQUIRED field (String, ISO 8601)
- [ ] All other fields optional (Option<T>)
- [ ] Add serde derives
- [ ] Document collision detection
- [ ] Add tests

**Module:** `src/models/post.rs`

**Example:**
```rust
PostUpdate {
  updated_at: post.updated_at.clone(), // from prior GET
  title: Some("New Title".into()),
  ..Default::default()
}
```

**References:**
- `docs/design/rust-client-architecture.md` (Collision Detection section)

---

### Issue 14: Admin Posts CRUD
**Labels:** `enhancement`, `client`, `v0.2.0`
**Depends on:** #11 (Admin Client Core), #12 (PostCreate Model), #13 (PostUpdate Model)

**Description:**
Implement full CRUD operations for posts in Admin API.

**Tasks:**
- [ ] Implement `browse_posts(params)` → `Result<Vec<Post>>`
- [ ] Implement `read_post_by_id(id)` → `Result<Post>`
- [ ] Implement `create_post(post)` → `Result<Post>`
- [ ] Implement `update_post(id, post)` → `Result<Post>`
- [ ] Implement `delete_post(id)` → `Result<()>`
- [ ] Handle response envelope parsing
- [ ] Add integration tests
- [ ] Document each method

**Module:** `src/client/admin.rs`

**Example:**
```rust
client.create_post(post).await?
```

---

### Issue 15: Response Envelope
**Labels:** `enhancement`, `models`, `v0.2.0`

**Description:**
Generic envelope parsing for Ghost API responses.

**Tasks:**
- [ ] Create `PostsResponse` struct: `{ posts: Vec<Post>, meta: Option<Meta> }`
- [ ] Create similar structs for other resources
- [ ] Add serde derives
- [ ] Test deserialization
- [ ] Document envelope format

**Module:** `src/models/envelope.rs`

**Example:**
```rust
PostsResponse { posts, meta }
```

---

### Issue 16: publish-post Example
**Labels:** `example`, `v0.2.0`
**Depends on:** #14 (Admin Posts CRUD)

**Description:**
Create working example for creating/publishing posts via Admin API.

**Tasks:**
- [ ] Create `examples/publish_post.rs`
- [ ] Parse CLI args (title, body, tags, draft/publish/schedule)
- [ ] Convert plain text body to minimal Lexical
- [ ] Call create_post
- [ ] Display created post ID, URL, status
- [ ] Add README example usage

**Module:** `examples/publish_post.rs`

**Example:**
```bash
cargo run --example publish_post -- --title "Test" --publish
```

---

## Summary Statistics

- **Total Milestones:** 8
- **Total Features:** 58
- **Total Issues to Create:** 58
- **Dependencies Tracked:** 35

## Implementation Order

1. Complete Milestone 1 (v0.1.0) first — all subsequent work depends on it
2. Complete Milestone 2 (v0.2.0) next — enables write operations
3. Then Milestone 3 (v0.3.0) — needed for Milestone 4
4. Then Milestone 4 (v0.4.0) — high-value markdown pipeline
5. Milestones 5 & 6 can be done in parallel
6. Finish with Milestones 7 & 8

## GitHub Workflow

For each issue:

1. Create issue with proper labels and milestone
2. Assign to yourself
3. Create feature branch: `git checkout -b feature/issue-N-brief-name`
4. Implement with tests
5. Run `cargo fmt` and `cargo clippy`
6. Create PR linking issue: "Closes #N"
7. Merge after review/CI passes

---

*This plan covers the first 2 milestones in detail. Continue this pattern for all 8 milestones.*
*See `milestone-report.md` and `version-roadmap.md` for complete feature lists.*

