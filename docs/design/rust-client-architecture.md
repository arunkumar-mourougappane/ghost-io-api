# Rust Client Architecture

> Research and design notes for implementing `ghost-io-api` as a strongly-typed async Rust crate.

---

## Two Top-Level Clients

Ghost exposes two distinct APIs, each needing its own client:

| Client | API | Auth | Access |
|--------|-----|------|--------|
| `GhostContentClient` | Content API | API key as `?key=` query param | Read-only |
| `GhostAdminClient` | Admin API | Short-lived JWT (HS256) | Read + Write |

---

## Recommended Dependencies

```toml
[dependencies]
reqwest    = { version = "0.12", features = ["json"] }
tokio      = { version = "1",    features = ["full"] }
serde      = { version = "1",    features = ["derive"] }
serde_json = "1"
hmac       = "0.12"   # HS256 JWT signing (Admin API)
sha2       = "0.10"
base64     = "0.22"   # base64url encoding for JWT
hex        = "0.4"    # decode hex secret from Admin API key
time       = { version = "0.3", features = ["std"] }
thiserror  = "1"      # ergonomic error types
```

> No full JWT crate needed — Ghost JWTs are simple enough to hand-roll with `hmac` + `sha2`.

---

## Module Structure

```
src/
├── lib.rs
├── error.rs              # GhostError enum
├── auth/
│   ├── mod.rs
│   ├── content.rs        # ContentApiKey (wraps key string)
│   └── admin.rs          # AdminApiKey, JWT generation
├── client/
│   ├── mod.rs
│   ├── content.rs        # GhostContentClient
│   └── admin.rs          # GhostAdminClient
├── models/
│   ├── mod.rs
│   ├── post.rs           # Post, PostCreate, PostUpdate
│   ├── page.rs
│   ├── tag.rs
│   ├── author.rs
│   ├── member.rs
│   ├── pagination.rs     # Meta, Pagination structs
│   └── settings.rs
└── params/
    ├── mod.rs
    └── browse.rs         # BrowseParams builder (filter/limit/page/order/include/fields)
```

---

## Key Design Decisions

### 1. JWT Generation (Admin Auth)

Ghost Admin API keys follow the format `{key_id}:{hex_secret}`.

A JWT must be generated server-side on every request:

- **Header:** `{"alg":"HS256","typ":"JWT","kid":"{key_id}"}`
- **Payload:** `{"iat": now_unix_secs, "exp": now_unix_secs + 300, "aud": "/admin/"}`
- **Signature:** `HMAC-SHA256(base64url(header) + "." + base64url(payload), hex_decode(secret))`

Steps in Rust:
1. Split key on `:` → `id` (str) and `hex_secret` (str)
2. `hex::decode(hex_secret)` → raw bytes for the HMAC key
3. Encode header + payload as base64url (no padding, `-` and `_` instead of `+` and `/`)
4. HMAC-SHA256 the `header_b64.payload_b64` string with the raw key bytes
5. base64url-encode the signature
6. Concatenate: `{header_b64}.{payload_b64}.{sig_b64}`
7. Set `Authorization: Ghost {token}` on the request

Timestamps are Unix epoch **seconds** (not milliseconds). `exp` must be at most 5 minutes after `iat`.

### 2. Typed Envelope Pattern

All Ghost responses wrap resources in a named array. Model this with a generic wrapper:

```rust
#[derive(serde::Deserialize)]
struct Envelope<T> {
    // field name varies per resource: "posts", "tags", etc.
    // Use a resource-specific type alias or a named struct per resource.
}

// Per-resource response structs
#[derive(serde::Deserialize)]
struct PostsResponse {
    posts: Vec<Post>,
    meta: Option<Meta>,
}
```

Single-resource endpoints (`/settings/`, `/site/`) return a flat object, not an array.

### 3. BrowseParams Builder

All browse (list) endpoints share the same query parameters. A single builder serialises to query params:

```rust
BrowseParams::new()
    .limit(10)
    .filter("featured:true")
    .include(&["tags", "authors"])
    .order("published_at desc")
    .page(2)
    .fields(&["id", "title", "url"])
```

Parameters:

| Param | Type | Notes |
|-------|------|-------|
| `filter` | NQL string | e.g. `featured:true`, `tag:news` |
| `limit` | u32 or `"all"` | Default 15, max 100 |
| `page` | u32 | 1-based pagination |
| `order` | string | SQL-style e.g. `published_at desc` |
| `include` | comma-separated | e.g. `authors,tags` |
| `fields` | comma-separated | Sparse fieldsets |
| `formats` | comma-separated | `html`, `plaintext` (posts/pages only) |

### 4. Collision Detection on Update (Admin)

`PUT` to posts and pages **requires** sending `updated_at` matching the server's current value. This prevents overwriting concurrent edits.

The `PostUpdate` struct must include `updated_at` as a **required** field:

```rust
pub struct PostUpdate {
    pub updated_at: String, // ISO 8601, required — get from a prior GET
    pub title: Option<String>,
    pub status: Option<PostStatus>,
    // ...
}
```

> Tag and author arrays are **replaced, not merged** on update. Always GET the post first, modify the arrays, then PUT the full modified arrays back.

### 5. Error Handling

Ghost returns structured JSON errors:

```json
{
  "errors": [
    {
      "message": "Resource not found.",
      "type": "NotFoundError",
      "context": null
    }
  ]
}
```

Map to a typed error enum:

```rust
#[derive(thiserror::Error, Debug)]
pub enum GhostError {
    #[error("Ghost API error ({error_type}): {message}")]
    Api {
        message: String,
        error_type: String,
        context: Option<String>,
    },
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("JWT signing error: {0}")]
    Auth(String),
}
```

---

## Content API Flow

```
GhostContentClient::new(base_url, content_api_key)
  └── appends ?key=... to every request

  .posts().browse(params).await       → Vec<Post>
  .posts().read_by_id(id).await       → Post
  .posts().read_by_slug(slug).await   → Post
  .pages().browse(params).await       → Vec<Page>
  .tags().browse(params).await        → Vec<Tag>
  .tags().read_by_slug(slug).await    → Tag
  .authors().browse(params).await     → Vec<Author>
  .tiers().browse(params).await       → Vec<Tier>
  .settings().get().await             → Settings
```

Required headers:
```
Accept-Version: v6.0
```

## Admin API Flow

```
GhostAdminClient::new(base_url, admin_api_key)
  └── generates fresh JWT on each request (5-min expiry)

  .posts().browse(params).await              → Vec<Post>
  .posts().read_by_id(id).await              → Post
  .posts().create(PostCreate).await          → Post
  .posts().update(id, PostUpdate).await      → Post   // requires updated_at
  .posts().delete(id).await                  → ()
  .pages()  ...  (same pattern as posts)
  .tags().create(TagCreate).await            → Tag
  .tags().update(id, TagUpdate).await        → Tag
  .tags().delete(id).await                   → ()
  .members().browse(params).await            → Vec<Member>
  .members().create(MemberCreate).await      → Member
  .members().update(id, MemberUpdate).await  → Member
```

Required headers:
```
Authorization: Ghost {jwt_token}
Accept-Version: v6.0
Content-Type: application/json   (POST / PUT only)
```

---

## Post Status Lifecycle

```
draft  ──→  published  ──→  (sends email newsletter if newsletter set)
  │                │
  └──→  scheduled ─┘   (published_at in the future)
                   │
              email_only → sent
```

Status values: `draft`, `published`, `scheduled`, `sent`

---

## Suggested Implementation Order

| Step | Scope |
|------|-------|
| 1 | `error.rs` + `models/pagination.rs` |
| 2 | `auth/content.rs` + `client/content.rs` + `models/post.rs` → Content posts end-to-end |
| 3 | `params/browse.rs` builder |
| 4 | Remaining Content models: `page`, `tag`, `author`, `settings` |
| 5 | `auth/admin.rs` JWT signing |
| 6 | `client/admin.rs` + Admin CRUD for posts |
| 7 | Admin tags, members |
| 8 | Integration tests against a live or mocked Ghost instance |
