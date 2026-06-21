//! Post model for Ghost API.
//!
//! Represents a post/article in Ghost with all its associated metadata,
//! content, and relationships.
//!
//! # Example
//!
//! ```
//! use ghost_io_api::models::post::{Post, PostStatus};
//!
//! // Typically posts come from API responses
//! let post = Post {
//!     id: "5ddc9141c35e7700383b2937".to_string(),
//!     uuid: Some("a5aa9bd8-ea31-415c-b452-3040dae1e730".to_string()),
//!     title: "Welcome".to_string(),
//!     slug: "welcome-short".to_string(),
//!     status: PostStatus::Published,
//!     visibility: Some("public".to_string()),
//!     created_at: Some("2019-11-26T02:43:13.000Z".to_string()),
//!     updated_at: Some("2019-11-26T02:44:17.000Z".to_string()),
//!     published_at: Some("2019-11-26T02:44:17.000Z".to_string()),
//!     // ... other fields
//!     ..Default::default()
//! };
//!
//! assert_eq!(post.status, PostStatus::Published);
//! assert!(post.is_published());
//! ```

use serde::{Deserialize, Serialize};

/// Status of a Ghost post.
///
/// Posts can be in one of four states throughout their lifecycle.
///
/// # Example
///
/// ```
/// use ghost_io_api::models::post::PostStatus;
/// use serde_json;
///
/// let json = serde_json::json!("published");
/// let status: PostStatus = serde_json::from_value(json).unwrap();
/// assert_eq!(status, PostStatus::Published);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PostStatus {
    /// Post is a draft, not visible publicly.
    #[default]
    Draft,
    /// Post is published and visible.
    Published,
    /// Post is scheduled for future publication.
    Scheduled,
    /// Post was sent as an email newsletter.
    Sent,
}

impl PostStatus {
    /// Returns `true` if the post is published.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::PostStatus;
    ///
    /// assert!(PostStatus::Published.is_published());
    /// assert!(!PostStatus::Draft.is_published());
    /// ```
    pub fn is_published(&self) -> bool {
        matches!(self, PostStatus::Published)
    }

    /// Returns `true` if the post is a draft.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::PostStatus;
    ///
    /// assert!(PostStatus::Draft.is_draft());
    /// assert!(!PostStatus::Published.is_draft());
    /// ```
    pub fn is_draft(&self) -> bool {
        matches!(self, PostStatus::Draft)
    }

    /// Returns `true` if the post is scheduled.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::PostStatus;
    ///
    /// assert!(PostStatus::Scheduled.is_scheduled());
    /// assert!(!PostStatus::Draft.is_scheduled());
    /// ```
    pub fn is_scheduled(&self) -> bool {
        matches!(self, PostStatus::Scheduled)
    }

    /// Returns `true` if the post was sent as email.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::PostStatus;
    ///
    /// assert!(PostStatus::Sent.is_sent());
    /// assert!(!PostStatus::Draft.is_sent());
    /// ```
    pub fn is_sent(&self) -> bool {
        matches!(self, PostStatus::Sent)
    }
}

/// A Ghost post/article.
///
/// Represents a post with all its metadata, content, and relationships.
/// Posts can contain Lexical JSON content or HTML, along with extensive
/// metadata for SEO, social sharing, and organization.
///
/// # Required Fields
///
/// Only `title` is required when creating a new post. The Ghost API will
/// generate IDs, slugs, and timestamps automatically.
///
/// # Field Details
///
/// * **Identifiers:** `id`, `uuid`, `slug`, `comment_id`
/// * **Content:** `title`, `lexical`, `html`, `excerpt`, `custom_excerpt`
/// * **Status:** `status`, `visibility`, `email_only`
/// * **Timestamps:** `created_at`, `updated_at`, `published_at`
/// * **Media:** `feature_image`, `feature_image_alt`, `feature_image_caption`
/// * **Flags:** `featured`
/// * **Relationships:** `tags`, `authors`, `primary_author`, `primary_tag`, `newsletter`
/// * **SEO:** `meta_title`, `meta_description`, `canonical_url`, `url`
/// * **Social:** `og_*` (Open Graph), `twitter_*` (Twitter Cards)
/// * **Code Injection:** `codeinjection_head`, `codeinjection_foot`, `custom_template`
///
/// # Example
///
/// ```
/// use ghost_io_api::models::post::{Post, PostStatus};
/// use serde_json::json;
///
/// let json = json!({
///     "id": "5ddc9141c35e7700383b2937",
///     "title": "Welcome",
///     "slug": "welcome-short",
///     "status": "published",
///     "visibility": "public",
///     "created_at": "2019-11-26T02:43:13.000Z",
///     "published_at": "2019-11-26T02:44:17.000Z"
/// });
///
/// let post: Post = serde_json::from_value(json).unwrap();
/// assert_eq!(post.title, "Welcome");
/// assert_eq!(post.status, PostStatus::Published);
/// assert!(post.is_published());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Post {
    // === Identifiers ===
    /// Unique post ID (24-character hex string).
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub id: String,

    /// UUID of the post (RFC 4122).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,

    /// URL-friendly identifier (unique, auto-generated from title).
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub slug: String,

    /// ID used for Disqus/comment systems (typically same as `id`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_id: Option<String>,

    // === Content ===
    /// Post title (**required** on create).
    pub title: String,

    /// Rich text content in Lexical JSON format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lexical: Option<String>,

    /// Rendered HTML content (read-only, use `?formats=html`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,

    /// Auto-generated excerpt from content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,

    /// Custom excerpt/summary (overrides auto-generated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_excerpt: Option<String>,

    // === Status & Visibility ===
    /// Publication status: draft, published, scheduled, or sent.
    #[serde(default)]
    pub status: PostStatus,

    /// Who can see the post: "public", "members", "paid", "tiers".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,

    /// If `true`, post is sent as email only (no web post).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_only: Option<bool>,

    // === Timestamps ===
    /// When the post was created (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// When the post was last updated (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,

    /// When the post was/will be published (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,

    // === Media ===
    /// URL to the feature image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_image: Option<String>,

    /// Alt text for the feature image (accessibility).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_image_alt: Option<String>,

    /// Caption for the feature image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_image_caption: Option<String>,

    // === Flags ===
    /// Whether this post is featured/highlighted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featured: Option<bool>,

    // === Relationships (simplified) ===
    // Note: Full tag/author/newsletter objects would require separate types.
    // For now, we use serde_json::Value to accept any structure.
    /// Associated tags (array of tag objects or strings).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<serde_json::Value>,

    /// Post authors (array of author objects or email strings).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<serde_json::Value>,

    /// Primary author object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_author: Option<serde_json::Value>,

    /// Primary tag object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_tag: Option<serde_json::Value>,

    /// Newsletter this post belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newsletter: Option<serde_json::Value>,

    /// Email send metadata (for posts sent as emails).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<serde_json::Value>,

    // === URLs ===
    /// Public URL of the post (read-only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Override canonical URL for SEO.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_url: Option<String>,

    // === SEO ===
    /// Custom meta title for `<title>` tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_title: Option<String>,

    /// Custom meta description for `<meta name="description">`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_description: Option<String>,

    // === Open Graph (Facebook) ===
    /// Custom Open Graph image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_image: Option<String>,

    /// Custom Open Graph title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_title: Option<String>,

    /// Custom Open Graph description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_description: Option<String>,

    // === Twitter Cards ===
    /// Custom Twitter Card image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_image: Option<String>,

    /// Custom Twitter Card title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_title: Option<String>,

    /// Custom Twitter Card description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_description: Option<String>,

    // === Code Injection ===
    /// Custom HTML injected into `<head>` (Ghost Admin only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codeinjection_head: Option<String>,

    /// Custom HTML injected before `</body>` (Ghost Admin only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codeinjection_foot: Option<String>,

    /// Custom theme template name to use for rendering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_template: Option<String>,
}

impl Post {
    /// Returns `true` if the post is published.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::{Post, PostStatus};
    ///
    /// let mut post = Post::default();
    /// post.status = PostStatus::Published;
    /// assert!(post.is_published());
    /// ```
    pub fn is_published(&self) -> bool {
        self.status.is_published()
    }

    /// Returns `true` if the post is a draft.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::{Post, PostStatus};
    ///
    /// let mut post = Post::default();
    /// post.status = PostStatus::Draft;
    /// assert!(post.is_draft());
    /// ```
    pub fn is_draft(&self) -> bool {
        self.status.is_draft()
    }

    /// Returns `true` if the post is scheduled for future publication.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::{Post, PostStatus};
    ///
    /// let mut post = Post::default();
    /// post.status = PostStatus::Scheduled;
    /// assert!(post.is_scheduled());
    /// ```
    pub fn is_scheduled(&self) -> bool {
        self.status.is_scheduled()
    }

    /// Returns `true` if the post was sent as an email newsletter.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::{Post, PostStatus};
    ///
    /// let mut post = Post::default();
    /// post.status = PostStatus::Sent;
    /// assert!(post.is_sent());
    /// ```
    pub fn is_sent(&self) -> bool {
        self.status.is_sent()
    }

    /// Returns `true` if the post is featured.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::Post;
    ///
    /// let mut post = Post::default();
    /// post.featured = Some(true);
    /// assert!(post.is_featured());
    /// ```
    pub fn is_featured(&self) -> bool {
        self.featured.unwrap_or(false)
    }

    /// Returns `true` if the post is email-only (no web post).
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::Post;
    ///
    /// let mut post = Post::default();
    /// post.email_only = Some(true);
    /// assert!(post.is_email_only());
    /// ```
    pub fn is_email_only(&self) -> bool {
        self.email_only.unwrap_or(false)
    }
}

// ── PostCreate supporting types ───────────────────────────────────────────────

/// Reference to a tag by name or slug, used when creating/updating posts.
///
/// At least one of `name` or `slug` must be set. Ghost resolves the tag by
/// slug first; if not found it creates a new tag with the given name.
///
/// # Example
///
/// ```
/// use ghost_io_api::models::post::TagRef;
///
/// // Reference an existing tag by slug
/// let existing = TagRef { slug: Some("rust".to_string()), ..Default::default() };
///
/// // Create a new tag on the fly
/// let new_tag = TagRef { name: Some("WebAssembly".to_string()), ..Default::default() };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TagRef {
    /// Tag name (used to create the tag if it does not exist yet).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Tag slug (used to look up an existing tag).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
}

impl TagRef {
    /// Creates a `TagRef` that looks up an existing tag by slug.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::TagRef;
    ///
    /// let t = TagRef::by_slug("rust");
    /// assert_eq!(t.slug.as_deref(), Some("rust"));
    /// ```
    pub fn by_slug(slug: impl Into<String>) -> Self {
        Self {
            slug: Some(slug.into()),
            name: None,
        }
    }

    /// Creates a `TagRef` that creates a new tag with the given name.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::TagRef;
    ///
    /// let t = TagRef::by_name("WebAssembly");
    /// assert_eq!(t.name.as_deref(), Some("WebAssembly"));
    /// ```
    pub fn by_name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            slug: None,
        }
    }
}

/// Reference to an author by id or email, used when creating/updating posts.
///
/// # Example
///
/// ```
/// use ghost_io_api::models::post::AuthorRef;
///
/// let author = AuthorRef { email: Some("jane@example.com".to_string()), ..Default::default() };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AuthorRef {
    /// Ghost user ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Ghost user email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

impl AuthorRef {
    /// Creates an `AuthorRef` that identifies an author by their Ghost ID.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::AuthorRef;
    ///
    /// let a = AuthorRef::by_id("6748592f4b9b7700010f6564");
    /// assert_eq!(a.id.as_deref(), Some("6748592f4b9b7700010f6564"));
    /// ```
    pub fn by_id(id: impl Into<String>) -> Self {
        Self {
            id: Some(id.into()),
            email: None,
        }
    }

    /// Creates an `AuthorRef` that identifies an author by their email.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::AuthorRef;
    ///
    /// let a = AuthorRef::by_email("jane@example.com");
    /// assert_eq!(a.email.as_deref(), Some("jane@example.com"));
    /// ```
    pub fn by_email(email: impl Into<String>) -> Self {
        Self {
            email: Some(email.into()),
            id: None,
        }
    }
}

// ── PostCreate ────────────────────────────────────────────────────────────────

/// Input model for creating a new Ghost post via `POST /ghost/api/admin/posts/`.
///
/// Only `title` is required by Ghost; every other field is optional and omitted
/// from the JSON payload when not set (via `#[serde(skip_serializing_if)]`).
///
/// Wrap this in a [`PostCreateEnvelope`] before sending to the API.
///
/// # Example
///
/// ```
/// use ghost_io_api::models::post::{PostCreate, PostStatus, TagRef};
///
/// let post = PostCreate {
///     title: "Hello, Ghost!".to_string(),
///     status: Some(PostStatus::Published),
///     tags: Some(vec![TagRef::by_slug("rust")]),
///     ..Default::default()
/// };
///
/// assert_eq!(post.title, "Hello, Ghost!");
/// assert_eq!(post.status, Some(PostStatus::Published));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PostCreate {
    // === Required ===
    /// Post title (**required**).
    pub title: String,

    // === Content ===
    /// Rich text content in Lexical JSON format (Ghost 6+).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lexical: Option<String>,

    /// HTML content (alternative to `lexical` for simpler content).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,

    /// Custom excerpt shown in post listings (overrides auto-generated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_excerpt: Option<String>,

    // === Identifiers ===
    /// URL slug (auto-generated from `title` if not set).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,

    // === Status & Visibility ===
    /// Publication status. Defaults to `Draft` when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<PostStatus>,

    /// Visibility of the post: `"public"`, `"members"`, `"paid"`, or `"tiers"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,

    /// If `true`, the post is delivered as email only (no web post).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_only: Option<bool>,

    /// If `true`, the post is highlighted in the portal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featured: Option<bool>,

    // === Scheduling ===
    /// ISO 8601 publication timestamp. Required when `status` is `"scheduled"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,

    // === Media ===
    /// URL to the feature image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_image: Option<String>,

    /// Alt text for the feature image (accessibility).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_image_alt: Option<String>,

    /// Caption displayed beneath the feature image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_image_caption: Option<String>,

    // === Relationships ===
    /// Tags to associate with the post. Ghost resolves by slug or creates by name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<TagRef>>,

    /// Authors to attribute the post to. Resolves by id or email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<AuthorRef>>,

    /// Slug of the newsletter to associate this post with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newsletter: Option<String>,

    // === SEO ===
    /// Custom `<title>` tag content for SEO.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_title: Option<String>,

    /// Custom `<meta name="description">` content for SEO.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_description: Option<String>,

    /// Canonical URL override for SEO (prevents duplicate-content penalties).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_url: Option<String>,

    // === Open Graph ===
    /// Custom Open Graph image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_image: Option<String>,

    /// Custom Open Graph title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_title: Option<String>,

    /// Custom Open Graph description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_description: Option<String>,

    // === Twitter Cards ===
    /// Custom Twitter Card image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_image: Option<String>,

    /// Custom Twitter Card title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_title: Option<String>,

    /// Custom Twitter Card description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_description: Option<String>,

    // === Code Injection ===
    /// Custom HTML injected into `<head>` for this post.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codeinjection_head: Option<String>,

    /// Custom HTML injected before `</body>` for this post.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codeinjection_foot: Option<String>,

    /// Custom theme template name to use when rendering this post.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_template: Option<String>,
}

impl PostCreate {
    /// Creates a minimal `PostCreate` with only the required `title`.
    ///
    /// All other fields default to `None` and will be omitted from the
    /// serialised JSON, letting Ghost apply its own defaults.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::PostCreate;
    ///
    /// let post = PostCreate::new("Hello, World!");
    /// assert_eq!(post.title, "Hello, World!");
    /// assert!(post.status.is_none());
    /// assert!(post.tags.is_none());
    /// ```
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }
}

/// Request envelope for `POST /ghost/api/admin/posts/`.
///
/// The Ghost Admin API requires the post payload to be wrapped in a
/// top-level `posts` array even when creating a single post.
///
/// # Example
///
/// ```
/// use ghost_io_api::models::post::{PostCreate, PostCreateEnvelope};
/// use serde_json;
///
/// let envelope = PostCreateEnvelope::new(PostCreate::new("My Post"));
/// let json = serde_json::to_value(&envelope).unwrap();
/// assert!(json["posts"].is_array());
/// assert_eq!(json["posts"][0]["title"], "My Post");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCreateEnvelope {
    /// Single-element array containing the post to create.
    pub posts: Vec<PostCreate>,
}

impl PostCreateEnvelope {
    /// Wraps a single `PostCreate` in the envelope Ghost expects.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::{PostCreate, PostCreateEnvelope};
    ///
    /// let envelope = PostCreateEnvelope::new(PostCreate::new("Draft"));
    /// assert_eq!(envelope.posts.len(), 1);
    /// assert_eq!(envelope.posts[0].title, "Draft");
    /// ```
    pub fn new(post: PostCreate) -> Self {
        Self { posts: vec![post] }
    }
}

// ── PostUpdate ────────────────────────────────────────────────────────────────

/// Input model for updating an existing Ghost post via
/// `PUT /ghost/api/admin/posts/{id}/`.
///
/// `updated_at` is **required** by Ghost as an optimistic-concurrency token:
/// if the value you send does not match the server's current `updated_at`,
/// Ghost rejects the request with a conflict error, preventing silent
/// overwrite of concurrent edits.
///
/// Every other field is optional; only the fields you set will be sent in the
/// JSON payload.  Wrap this in a [`PostUpdateEnvelope`] before sending.
///
/// # Example
///
/// ```
/// use ghost_io_api::models::post::{PostUpdate, PostStatus, TagRef};
///
/// let update = PostUpdate {
///     updated_at: "2026-01-01T00:00:00.000Z".to_string(),
///     status: Some(PostStatus::Published),
///     tags: Some(vec![TagRef::by_slug("rust")]),
///     ..Default::default()
/// };
///
/// assert_eq!(update.updated_at, "2026-01-01T00:00:00.000Z");
/// assert_eq!(update.status, Some(PostStatus::Published));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PostUpdate {
    // === Collision-detection (required) ===
    /// ISO 8601 timestamp of the post's last update.
    ///
    /// Ghost rejects the request if this does not match the server's
    /// current value, preventing silent overwrites of concurrent edits.
    pub updated_at: String,

    // === Content ===
    /// Updated post title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Updated rich text content in Lexical JSON format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lexical: Option<String>,

    /// Updated HTML content (alternative to `lexical`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,

    /// Updated custom excerpt shown in post listings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_excerpt: Option<String>,

    // === Identifiers ===
    /// Updated URL slug.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,

    // === Status & Visibility ===
    /// Updated publication status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<PostStatus>,

    /// Updated visibility: `"public"`, `"members"`, `"paid"`, or `"tiers"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,

    /// Updated email-only flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_only: Option<bool>,

    /// Updated featured flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featured: Option<bool>,

    // === Scheduling ===
    /// Updated publication timestamp (ISO 8601). Required when setting
    /// `status` to `"scheduled"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,

    // === Media ===
    /// Updated feature image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_image: Option<String>,

    /// Updated feature image alt text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_image_alt: Option<String>,

    /// Updated feature image caption.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_image_caption: Option<String>,

    // === Relationships ===
    /// Replacement tag list. Resolves by slug or creates by name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<TagRef>>,

    /// Replacement author list. Resolves by id or email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<AuthorRef>>,

    /// Updated newsletter slug.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newsletter: Option<String>,

    // === SEO ===
    /// Updated meta title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_title: Option<String>,

    /// Updated meta description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_description: Option<String>,

    /// Updated canonical URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_url: Option<String>,

    // === Open Graph ===
    /// Updated Open Graph image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_image: Option<String>,

    /// Updated Open Graph title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_title: Option<String>,

    /// Updated Open Graph description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_description: Option<String>,

    // === Twitter Cards ===
    /// Updated Twitter Card image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_image: Option<String>,

    /// Updated Twitter Card title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_title: Option<String>,

    /// Updated Twitter Card description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_description: Option<String>,

    // === Code Injection ===
    /// Updated `<head>` code injection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codeinjection_head: Option<String>,

    /// Updated `</body>` code injection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codeinjection_foot: Option<String>,

    /// Updated custom theme template name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_template: Option<String>,
}

impl PostUpdate {
    /// Creates a minimal `PostUpdate` with only the required `updated_at`.
    ///
    /// All other fields default to `None` and are omitted from the JSON
    /// payload, so only the fields you explicitly set will be sent.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::PostUpdate;
    ///
    /// let u = PostUpdate::new("2026-01-01T12:00:00.000Z");
    /// assert_eq!(u.updated_at, "2026-01-01T12:00:00.000Z");
    /// assert!(u.title.is_none());
    /// assert!(u.status.is_none());
    /// ```
    pub fn new(updated_at: impl Into<String>) -> Self {
        Self {
            updated_at: updated_at.into(),
            ..Default::default()
        }
    }
}

/// Request envelope for `PUT /ghost/api/admin/posts/{id}/`.
///
/// The Ghost Admin API requires the update payload to be wrapped in a
/// top-level `posts` array even when updating a single post.
///
/// # Example
///
/// ```
/// use ghost_io_api::models::post::{PostUpdate, PostUpdateEnvelope};
/// use serde_json;
///
/// let envelope = PostUpdateEnvelope::new(PostUpdate::new("2026-01-01T12:00:00.000Z"));
/// let json = serde_json::to_value(&envelope).unwrap();
/// assert!(json["posts"].is_array());
/// assert_eq!(json["posts"][0]["updated_at"], "2026-01-01T12:00:00.000Z");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostUpdateEnvelope {
    /// Single-element array containing the post update payload.
    pub posts: Vec<PostUpdate>,
}

impl PostUpdateEnvelope {
    /// Wraps a single [`PostUpdate`] in the envelope Ghost expects.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::post::{PostUpdate, PostUpdateEnvelope};
    ///
    /// let envelope = PostUpdateEnvelope::new(PostUpdate::new("2026-01-01T12:00:00.000Z"));
    /// assert_eq!(envelope.posts.len(), 1);
    /// assert_eq!(envelope.posts[0].updated_at, "2026-01-01T12:00:00.000Z");
    /// ```
    pub fn new(post: PostUpdate) -> Self {
        Self { posts: vec![post] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_post_status_deserialization() {
        assert_eq!(
            serde_json::from_str::<PostStatus>("\"draft\"").unwrap(),
            PostStatus::Draft
        );
        assert_eq!(
            serde_json::from_str::<PostStatus>("\"published\"").unwrap(),
            PostStatus::Published
        );
        assert_eq!(
            serde_json::from_str::<PostStatus>("\"scheduled\"").unwrap(),
            PostStatus::Scheduled
        );
        assert_eq!(
            serde_json::from_str::<PostStatus>("\"sent\"").unwrap(),
            PostStatus::Sent
        );
    }

    #[test]
    fn test_post_status_serialization() {
        assert_eq!(
            serde_json::to_string(&PostStatus::Draft).unwrap(),
            "\"draft\""
        );
        assert_eq!(
            serde_json::to_string(&PostStatus::Published).unwrap(),
            "\"published\""
        );
        assert_eq!(
            serde_json::to_string(&PostStatus::Scheduled).unwrap(),
            "\"scheduled\""
        );
        assert_eq!(
            serde_json::to_string(&PostStatus::Sent).unwrap(),
            "\"sent\""
        );
    }

    #[test]
    fn test_post_status_methods() {
        assert!(PostStatus::Draft.is_draft());
        assert!(!PostStatus::Draft.is_published());
        assert!(!PostStatus::Draft.is_scheduled());
        assert!(!PostStatus::Draft.is_sent());

        assert!(!PostStatus::Published.is_draft());
        assert!(PostStatus::Published.is_published());
        assert!(!PostStatus::Published.is_scheduled());
        assert!(!PostStatus::Published.is_sent());

        assert!(!PostStatus::Scheduled.is_draft());
        assert!(!PostStatus::Scheduled.is_published());
        assert!(PostStatus::Scheduled.is_scheduled());
        assert!(!PostStatus::Scheduled.is_sent());

        assert!(!PostStatus::Sent.is_draft());
        assert!(!PostStatus::Sent.is_published());
        assert!(!PostStatus::Sent.is_scheduled());
        assert!(PostStatus::Sent.is_sent());
    }

    #[test]
    fn test_post_minimal_deserialization() {
        let json = json!({
            "id": "5ddc9141c35e7700383b2937",
            "title": "Welcome",
            "slug": "welcome-short",
            "status": "published"
        });

        let post: Post = serde_json::from_value(json).unwrap();
        assert_eq!(post.id, "5ddc9141c35e7700383b2937");
        assert_eq!(post.title, "Welcome");
        assert_eq!(post.slug, "welcome-short");
        assert_eq!(post.status, PostStatus::Published);
    }

    #[test]
    fn test_post_full_deserialization() {
        let json = json!({
            "id": "5ddc9141c35e7700383b2937",
            "uuid": "a5aa9bd8-ea31-415c-b452-3040dae1e730",
            "title": "Welcome",
            "slug": "welcome-short",
            "status": "published",
            "visibility": "public",
            "created_at": "2019-11-26T02:43:13.000Z",
            "updated_at": "2019-11-26T02:44:17.000Z",
            "published_at": "2019-11-26T02:44:17.000Z",
            "feature_image": "https://example.com/image.png",
            "featured": true,
            "excerpt": "Welcome excerpt",
            "custom_excerpt": "Custom excerpt",
            "meta_title": "Welcome | My Site",
            "meta_description": "A welcoming post",
            "og_title": "Welcome",
            "og_description": "OG description",
            "twitter_title": "Welcome on Twitter",
            "email_only": false,
            "canonical_url": "https://example.com/welcome"
        });

        let post: Post = serde_json::from_value(json).unwrap();
        assert_eq!(post.id, "5ddc9141c35e7700383b2937");
        assert_eq!(
            post.uuid,
            Some("a5aa9bd8-ea31-415c-b452-3040dae1e730".to_string())
        );
        assert_eq!(post.title, "Welcome");
        assert_eq!(post.status, PostStatus::Published);
        assert_eq!(post.visibility, Some("public".to_string()));
        assert_eq!(post.featured, Some(true));
        assert_eq!(post.email_only, Some(false));
        assert_eq!(post.meta_title, Some("Welcome | My Site".to_string()));
    }

    #[test]
    fn test_post_with_relationships() {
        let json = json!({
            "id": "123",
            "title": "Post with tags",
            "slug": "post-with-tags",
            "status": "draft",
            "tags": [
                {"id": "tag1", "name": "Tech"},
                {"id": "tag2", "name": "Rust"}
            ],
            "authors": [
                {"id": "author1", "name": "John Doe", "email": "john@example.com"}
            ]
        });

        let post: Post = serde_json::from_value(json).unwrap();
        assert_eq!(post.title, "Post with tags");
        assert!(post.tags.is_some());
        assert!(post.authors.is_some());
    }

    #[test]
    fn test_post_serialization() {
        let post = Post {
            id: "123".to_string(),
            title: "Test Post".to_string(),
            slug: "test-post".to_string(),
            status: PostStatus::Published,
            visibility: Some("public".to_string()),
            featured: Some(true),
            ..Default::default()
        };

        let json = serde_json::to_value(&post).unwrap();
        assert_eq!(json["id"], "123");
        assert_eq!(json["title"], "Test Post");
        assert_eq!(json["slug"], "test-post");
        assert_eq!(json["status"], "published");
        assert_eq!(json["visibility"], "public");
        assert_eq!(json["featured"], true);
    }

    #[test]
    fn test_post_serialization_skips_none() {
        let post = Post {
            id: "123".to_string(),
            title: "Minimal Post".to_string(),
            slug: "minimal".to_string(),
            status: PostStatus::Draft,
            ..Default::default()
        };

        let json = serde_json::to_value(&post).unwrap();
        assert!(!json.as_object().unwrap().contains_key("uuid"));
        assert!(!json.as_object().unwrap().contains_key("feature_image"));
        assert!(!json.as_object().unwrap().contains_key("tags"));
    }

    #[test]
    fn test_post_default() {
        let post = Post::default();
        assert_eq!(post.id, "");
        assert_eq!(post.title, "");
        assert_eq!(post.slug, "");
        assert_eq!(post.status, PostStatus::Draft);
        assert_eq!(post.uuid, None);
        assert_eq!(post.featured, None);
    }

    #[test]
    fn test_post_is_published() {
        let post = Post {
            status: PostStatus::Published,
            ..Default::default()
        };
        assert!(post.is_published());
        assert!(!post.is_draft());
    }

    #[test]
    fn test_post_is_draft() {
        let post = Post {
            status: PostStatus::Draft,
            ..Default::default()
        };
        assert!(post.is_draft());
        assert!(!post.is_published());
    }

    #[test]
    fn test_post_is_scheduled() {
        let post = Post {
            status: PostStatus::Scheduled,
            ..Default::default()
        };
        assert!(post.is_scheduled());
        assert!(!post.is_published());
    }

    #[test]
    fn test_post_is_sent() {
        let post = Post {
            status: PostStatus::Sent,
            ..Default::default()
        };
        assert!(post.is_sent());
        assert!(!post.is_published());
    }

    #[test]
    fn test_post_is_featured() {
        let mut post = Post::default();
        assert!(!post.is_featured());

        post.featured = Some(true);
        assert!(post.is_featured());

        post.featured = Some(false);
        assert!(!post.is_featured());
    }

    #[test]
    fn test_post_is_email_only() {
        let mut post = Post::default();
        assert!(!post.is_email_only());

        post.email_only = Some(true);
        assert!(post.is_email_only());

        post.email_only = Some(false);
        assert!(!post.is_email_only());
    }

    #[test]
    fn test_post_clone() {
        let post = Post {
            id: "123".to_string(),
            title: "Clone Test".to_string(),
            slug: "clone-test".to_string(),
            status: PostStatus::Published,
            ..Default::default()
        };

        let cloned = post.clone();
        assert_eq!(post, cloned);
    }

    #[test]
    fn test_post_status_clone() {
        let status = PostStatus::Published;
        let cloned = status;
        assert_eq!(status, cloned);
    }

    // ── TagRef ────────────────────────────────────────────────────────────────

    #[test]
    fn test_tag_ref_by_slug() {
        let t = TagRef::by_slug("rust");
        assert_eq!(t.slug.as_deref(), Some("rust"));
        assert!(t.name.is_none());
    }

    #[test]
    fn test_tag_ref_by_name() {
        let t = TagRef::by_name("WebAssembly");
        assert_eq!(t.name.as_deref(), Some("WebAssembly"));
        assert!(t.slug.is_none());
    }

    #[test]
    fn test_tag_ref_default() {
        let t = TagRef::default();
        assert!(t.name.is_none());
        assert!(t.slug.is_none());
    }

    #[test]
    fn test_tag_ref_serialization_skips_none() {
        let t = TagRef::by_slug("rust");
        let json = serde_json::to_value(&t).unwrap();
        assert_eq!(json["slug"], "rust");
        assert!(!json.as_object().unwrap().contains_key("name"));
    }

    #[test]
    fn test_tag_ref_deserialization() {
        let json = json!({"name": "Rust", "slug": "rust"});
        let t: TagRef = serde_json::from_value(json).unwrap();
        assert_eq!(t.name.as_deref(), Some("Rust"));
        assert_eq!(t.slug.as_deref(), Some("rust"));
    }

    #[test]
    fn test_tag_ref_clone_eq() {
        let t = TagRef::by_slug("rust");
        assert_eq!(t.clone(), t);
    }

    // ── AuthorRef ─────────────────────────────────────────────────────────────

    #[test]
    fn test_author_ref_by_id() {
        let a = AuthorRef::by_id("abc123");
        assert_eq!(a.id.as_deref(), Some("abc123"));
        assert!(a.email.is_none());
    }

    #[test]
    fn test_author_ref_by_email() {
        let a = AuthorRef::by_email("jane@example.com");
        assert_eq!(a.email.as_deref(), Some("jane@example.com"));
        assert!(a.id.is_none());
    }

    #[test]
    fn test_author_ref_default() {
        let a = AuthorRef::default();
        assert!(a.id.is_none());
        assert!(a.email.is_none());
    }

    #[test]
    fn test_author_ref_serialization_skips_none() {
        let a = AuthorRef::by_email("jane@example.com");
        let json = serde_json::to_value(&a).unwrap();
        assert_eq!(json["email"], "jane@example.com");
        assert!(!json.as_object().unwrap().contains_key("id"));
    }

    #[test]
    fn test_author_ref_deserialization() {
        let json = json!({"id": "abc123", "email": "jane@example.com"});
        let a: AuthorRef = serde_json::from_value(json).unwrap();
        assert_eq!(a.id.as_deref(), Some("abc123"));
        assert_eq!(a.email.as_deref(), Some("jane@example.com"));
    }

    #[test]
    fn test_author_ref_clone_eq() {
        let a = AuthorRef::by_email("jane@example.com");
        assert_eq!(a.clone(), a);
    }

    // ── PostCreate ────────────────────────────────────────────────────────────

    #[test]
    fn test_post_create_new_title_only() {
        let p = PostCreate::new("Hello, Ghost!");
        assert_eq!(p.title, "Hello, Ghost!");
        assert!(p.status.is_none());
        assert!(p.slug.is_none());
        assert!(p.lexical.is_none());
        assert!(p.tags.is_none());
        assert!(p.authors.is_none());
    }

    #[test]
    fn test_post_create_default_all_none() {
        let p = PostCreate::default();
        assert_eq!(p.title, "");
        assert!(p.status.is_none());
        assert!(p.visibility.is_none());
        assert!(p.featured.is_none());
        assert!(p.email_only.is_none());
        assert!(p.published_at.is_none());
        assert!(p.lexical.is_none());
        assert!(p.html.is_none());
        assert!(p.custom_excerpt.is_none());
        assert!(p.slug.is_none());
        assert!(p.feature_image.is_none());
        assert!(p.tags.is_none());
        assert!(p.authors.is_none());
        assert!(p.newsletter.is_none());
        assert!(p.meta_title.is_none());
        assert!(p.meta_description.is_none());
        assert!(p.canonical_url.is_none());
        assert!(p.og_image.is_none());
        assert!(p.og_title.is_none());
        assert!(p.og_description.is_none());
        assert!(p.twitter_image.is_none());
        assert!(p.twitter_title.is_none());
        assert!(p.twitter_description.is_none());
        assert!(p.codeinjection_head.is_none());
        assert!(p.codeinjection_foot.is_none());
        assert!(p.custom_template.is_none());
    }

    #[test]
    fn test_post_create_serializes_title_only() {
        let p = PostCreate::new("Minimal");
        let json = serde_json::to_value(&p).unwrap();
        let obj = json.as_object().unwrap();
        assert_eq!(json["title"], "Minimal");
        // All optional fields must be absent
        for key in &[
            "status",
            "slug",
            "lexical",
            "html",
            "custom_excerpt",
            "visibility",
            "featured",
            "email_only",
            "published_at",
            "feature_image",
            "feature_image_alt",
            "feature_image_caption",
            "tags",
            "authors",
            "newsletter",
            "meta_title",
            "meta_description",
            "canonical_url",
            "og_image",
            "og_title",
            "og_description",
            "twitter_image",
            "twitter_title",
            "twitter_description",
            "codeinjection_head",
            "codeinjection_foot",
            "custom_template",
        ] {
            assert!(!obj.contains_key(*key), "unexpected key: {key}");
        }
    }

    #[test]
    fn test_post_create_with_status() {
        let p = PostCreate {
            title: "Draft Post".to_string(),
            status: Some(PostStatus::Draft),
            ..Default::default()
        };
        let json = serde_json::to_value(&p).unwrap();
        assert_eq!(json["status"], "draft");
    }

    #[test]
    fn test_post_create_with_all_fields() {
        let p = PostCreate {
            title: "Full Post".to_string(),
            status: Some(PostStatus::Published),
            slug: Some("full-post".to_string()),
            visibility: Some("members".to_string()),
            featured: Some(true),
            email_only: Some(false),
            published_at: Some("2026-01-01T00:00:00.000Z".to_string()),
            lexical: Some("{\"root\":{}}".to_string()),
            html: Some("<p>Hello</p>".to_string()),
            custom_excerpt: Some("Summary".to_string()),
            feature_image: Some("https://example.com/img.jpg".to_string()),
            feature_image_alt: Some("Alt text".to_string()),
            feature_image_caption: Some("Caption".to_string()),
            tags: Some(vec![TagRef::by_slug("rust"), TagRef::by_name("New Tag")]),
            authors: Some(vec![AuthorRef::by_email("jane@example.com")]),
            newsletter: Some("weekly".to_string()),
            meta_title: Some("SEO Title".to_string()),
            meta_description: Some("SEO Desc".to_string()),
            canonical_url: Some("https://example.com/canonical".to_string()),
            og_image: Some("https://example.com/og.jpg".to_string()),
            og_title: Some("OG Title".to_string()),
            og_description: Some("OG Desc".to_string()),
            twitter_image: Some("https://example.com/tw.jpg".to_string()),
            twitter_title: Some("TW Title".to_string()),
            twitter_description: Some("TW Desc".to_string()),
            codeinjection_head: Some("<meta name='test' content='1'>".to_string()),
            codeinjection_foot: Some("<script>console.log('ok')</script>".to_string()),
            custom_template: Some("custom".to_string()),
        };

        let json = serde_json::to_value(&p).unwrap();
        assert_eq!(json["title"], "Full Post");
        assert_eq!(json["status"], "published");
        assert_eq!(json["slug"], "full-post");
        assert_eq!(json["visibility"], "members");
        assert_eq!(json["featured"], true);
        assert_eq!(json["email_only"], false);
        assert_eq!(json["published_at"], "2026-01-01T00:00:00.000Z");
        assert_eq!(json["custom_excerpt"], "Summary");
        assert_eq!(json["feature_image"], "https://example.com/img.jpg");
        assert_eq!(json["feature_image_alt"], "Alt text");
        assert_eq!(json["feature_image_caption"], "Caption");
        assert_eq!(json["tags"][0]["slug"], "rust");
        assert_eq!(json["tags"][1]["name"], "New Tag");
        assert_eq!(json["authors"][0]["email"], "jane@example.com");
        assert_eq!(json["newsletter"], "weekly");
        assert_eq!(json["meta_title"], "SEO Title");
        assert_eq!(json["og_title"], "OG Title");
        assert_eq!(json["twitter_title"], "TW Title");
        assert_eq!(json["codeinjection_head"], "<meta name='test' content='1'>");
        assert_eq!(json["custom_template"], "custom");
    }

    #[test]
    fn test_post_create_deserialization() {
        let json = json!({
            "title": "Deserialized Post",
            "status": "draft",
            "slug": "deserialized-post",
            "tags": [{"slug": "rust"}]
        });
        let p: PostCreate = serde_json::from_value(json).unwrap();
        assert_eq!(p.title, "Deserialized Post");
        assert_eq!(p.status, Some(PostStatus::Draft));
        assert_eq!(p.slug.as_deref(), Some("deserialized-post"));
        assert_eq!(p.tags.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_post_create_scheduled_status() {
        let p = PostCreate {
            title: "Future Post".to_string(),
            status: Some(PostStatus::Scheduled),
            published_at: Some("2027-06-01T09:00:00.000Z".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_value(&p).unwrap();
        assert_eq!(json["status"], "scheduled");
        assert_eq!(json["published_at"], "2027-06-01T09:00:00.000Z");
    }

    #[test]
    fn test_post_create_clone_eq() {
        let p = PostCreate::new("Clone Me");
        assert_eq!(p.clone(), p);
    }

    // ── PostCreateEnvelope ────────────────────────────────────────────────────

    #[test]
    fn test_post_create_envelope_new() {
        let env = PostCreateEnvelope::new(PostCreate::new("Wrapped"));
        assert_eq!(env.posts.len(), 1);
        assert_eq!(env.posts[0].title, "Wrapped");
    }

    #[test]
    fn test_post_create_envelope_serialization() {
        let env = PostCreateEnvelope::new(PostCreate::new("API Post"));
        let json = serde_json::to_value(&env).unwrap();
        assert!(json["posts"].is_array());
        assert_eq!(json["posts"].as_array().unwrap().len(), 1);
        assert_eq!(json["posts"][0]["title"], "API Post");
    }

    #[test]
    fn test_post_create_envelope_omits_none_fields() {
        let env = PostCreateEnvelope::new(PostCreate::new("Minimal"));
        let json = serde_json::to_value(&env).unwrap();
        let post_obj = json["posts"][0].as_object().unwrap();
        assert_eq!(post_obj.len(), 1, "only 'title' key should be present");
        assert!(post_obj.contains_key("title"));
    }

    #[test]
    fn test_post_create_envelope_deserialization() {
        let json = json!({
            "posts": [{"title": "From API", "status": "published"}]
        });
        let env: PostCreateEnvelope = serde_json::from_value(json).unwrap();
        assert_eq!(env.posts.len(), 1);
        assert_eq!(env.posts[0].title, "From API");
        assert_eq!(env.posts[0].status, Some(PostStatus::Published));
    }

    #[test]
    fn test_post_create_envelope_clone() {
        let env = PostCreateEnvelope::new(PostCreate::new("Clone"));
        let cloned = env.clone();
        assert_eq!(cloned.posts[0].title, "Clone");
    }

    // ── PostUpdate ────────────────────────────────────────────────────────────

    const UPDATED_AT: &str = "2026-01-15T10:30:00.000Z";

    #[test]
    fn test_post_update_new_required_only() {
        let u = PostUpdate::new(UPDATED_AT);
        assert_eq!(u.updated_at, UPDATED_AT);
        assert!(u.title.is_none());
        assert!(u.status.is_none());
        assert!(u.slug.is_none());
        assert!(u.lexical.is_none());
        assert!(u.html.is_none());
        assert!(u.custom_excerpt.is_none());
        assert!(u.visibility.is_none());
        assert!(u.featured.is_none());
        assert!(u.email_only.is_none());
        assert!(u.published_at.is_none());
        assert!(u.feature_image.is_none());
        assert!(u.feature_image_alt.is_none());
        assert!(u.feature_image_caption.is_none());
        assert!(u.tags.is_none());
        assert!(u.authors.is_none());
        assert!(u.newsletter.is_none());
        assert!(u.meta_title.is_none());
        assert!(u.meta_description.is_none());
        assert!(u.canonical_url.is_none());
        assert!(u.og_image.is_none());
        assert!(u.og_title.is_none());
        assert!(u.og_description.is_none());
        assert!(u.twitter_image.is_none());
        assert!(u.twitter_title.is_none());
        assert!(u.twitter_description.is_none());
        assert!(u.codeinjection_head.is_none());
        assert!(u.codeinjection_foot.is_none());
        assert!(u.custom_template.is_none());
    }

    #[test]
    fn test_post_update_default_updated_at_empty() {
        let u = PostUpdate::default();
        assert_eq!(u.updated_at, "");
    }

    #[test]
    fn test_post_update_serializes_updated_at_always() {
        // updated_at must ALWAYS be present — it has no skip_serializing_if
        let u = PostUpdate::new(UPDATED_AT);
        let json = serde_json::to_value(&u).unwrap();
        assert_eq!(json["updated_at"], UPDATED_AT);
    }

    #[test]
    fn test_post_update_minimal_has_only_updated_at() {
        let u = PostUpdate::new(UPDATED_AT);
        let json = serde_json::to_value(&u).unwrap();
        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 1, "only 'updated_at' should be serialised");
        assert!(obj.contains_key("updated_at"));
    }

    #[test]
    fn test_post_update_with_title() {
        let u = PostUpdate {
            updated_at: UPDATED_AT.to_string(),
            title: Some("New Title".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_value(&u).unwrap();
        assert_eq!(json["updated_at"], UPDATED_AT);
        assert_eq!(json["title"], "New Title");
        // No other keys should be present
        assert_eq!(json.as_object().unwrap().len(), 2);
    }

    #[test]
    fn test_post_update_publish() {
        let u = PostUpdate {
            updated_at: UPDATED_AT.to_string(),
            status: Some(PostStatus::Published),
            ..Default::default()
        };
        let json = serde_json::to_value(&u).unwrap();
        assert_eq!(json["status"], "published");
    }

    #[test]
    fn test_post_update_unpublish_to_draft() {
        let u = PostUpdate {
            updated_at: UPDATED_AT.to_string(),
            status: Some(PostStatus::Draft),
            ..Default::default()
        };
        let json = serde_json::to_value(&u).unwrap();
        assert_eq!(json["status"], "draft");
    }

    #[test]
    fn test_post_update_schedule() {
        let u = PostUpdate {
            updated_at: UPDATED_AT.to_string(),
            status: Some(PostStatus::Scheduled),
            published_at: Some("2027-03-01T09:00:00.000Z".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_value(&u).unwrap();
        assert_eq!(json["status"], "scheduled");
        assert_eq!(json["published_at"], "2027-03-01T09:00:00.000Z");
    }

    #[test]
    fn test_post_update_with_all_fields() {
        let u = PostUpdate {
            updated_at: UPDATED_AT.to_string(),
            title: Some("Updated Title".to_string()),
            lexical: Some("{\"root\":{}}".to_string()),
            html: Some("<p>Updated</p>".to_string()),
            custom_excerpt: Some("Updated excerpt".to_string()),
            slug: Some("updated-slug".to_string()),
            status: Some(PostStatus::Published),
            visibility: Some("members".to_string()),
            email_only: Some(false),
            featured: Some(true),
            published_at: Some("2026-06-01T09:00:00.000Z".to_string()),
            feature_image: Some("https://example.com/new.jpg".to_string()),
            feature_image_alt: Some("New alt".to_string()),
            feature_image_caption: Some("New caption".to_string()),
            tags: Some(vec![TagRef::by_slug("rust")]),
            authors: Some(vec![AuthorRef::by_email("bob@example.com")]),
            newsletter: Some("weekly".to_string()),
            meta_title: Some("New SEO Title".to_string()),
            meta_description: Some("New SEO Desc".to_string()),
            canonical_url: Some("https://example.com/new-canon".to_string()),
            og_image: Some("https://example.com/og-new.jpg".to_string()),
            og_title: Some("New OG Title".to_string()),
            og_description: Some("New OG Desc".to_string()),
            twitter_image: Some("https://example.com/tw-new.jpg".to_string()),
            twitter_title: Some("New TW Title".to_string()),
            twitter_description: Some("New TW Desc".to_string()),
            codeinjection_head: Some("<meta name='v' content='2'>".to_string()),
            codeinjection_foot: Some("<script>console.log('v2')</script>".to_string()),
            custom_template: Some("custom-v2".to_string()),
        };

        let json = serde_json::to_value(&u).unwrap();
        assert_eq!(json["updated_at"], UPDATED_AT);
        assert_eq!(json["title"], "Updated Title");
        assert_eq!(json["status"], "published");
        assert_eq!(json["slug"], "updated-slug");
        assert_eq!(json["visibility"], "members");
        assert_eq!(json["featured"], true);
        assert_eq!(json["email_only"], false);
        assert_eq!(json["tags"][0]["slug"], "rust");
        assert_eq!(json["authors"][0]["email"], "bob@example.com");
        assert_eq!(json["newsletter"], "weekly");
        assert_eq!(json["meta_title"], "New SEO Title");
        assert_eq!(json["og_title"], "New OG Title");
        assert_eq!(json["twitter_title"], "New TW Title");
        assert_eq!(json["codeinjection_head"], "<meta name='v' content='2'>");
        assert_eq!(json["custom_template"], "custom-v2");
    }

    #[test]
    fn test_post_update_deserialization() {
        let json = json!({
            "updated_at": UPDATED_AT,
            "title": "Round-tripped",
            "status": "draft"
        });
        let u: PostUpdate = serde_json::from_value(json).unwrap();
        assert_eq!(u.updated_at, UPDATED_AT);
        assert_eq!(u.title.as_deref(), Some("Round-tripped"));
        assert_eq!(u.status, Some(PostStatus::Draft));
        assert!(u.tags.is_none());
    }

    #[test]
    fn test_post_update_deserialization_missing_optional_fields() {
        // Only updated_at present — all options should deserialise to None
        let json = json!({"updated_at": UPDATED_AT});
        let u: PostUpdate = serde_json::from_value(json).unwrap();
        assert_eq!(u.updated_at, UPDATED_AT);
        assert!(u.title.is_none());
        assert!(u.status.is_none());
    }

    #[test]
    fn test_post_update_clone_eq() {
        let u = PostUpdate::new(UPDATED_AT);
        assert_eq!(u.clone(), u);
    }

    #[test]
    fn test_post_update_differs_on_updated_at() {
        let u1 = PostUpdate::new("2026-01-01T00:00:00.000Z");
        let u2 = PostUpdate::new("2026-01-02T00:00:00.000Z");
        assert_ne!(u1, u2);
    }

    // ── PostUpdateEnvelope ────────────────────────────────────────────────────

    #[test]
    fn test_post_update_envelope_new() {
        let env = PostUpdateEnvelope::new(PostUpdate::new(UPDATED_AT));
        assert_eq!(env.posts.len(), 1);
        assert_eq!(env.posts[0].updated_at, UPDATED_AT);
    }

    #[test]
    fn test_post_update_envelope_serialization() {
        let env = PostUpdateEnvelope::new(PostUpdate::new(UPDATED_AT));
        let json = serde_json::to_value(&env).unwrap();
        assert!(json["posts"].is_array());
        assert_eq!(json["posts"].as_array().unwrap().len(), 1);
        assert_eq!(json["posts"][0]["updated_at"], UPDATED_AT);
    }

    #[test]
    fn test_post_update_envelope_minimal_has_only_updated_at_inside() {
        let env = PostUpdateEnvelope::new(PostUpdate::new(UPDATED_AT));
        let json = serde_json::to_value(&env).unwrap();
        let post_obj = json["posts"][0].as_object().unwrap();
        assert_eq!(
            post_obj.len(),
            1,
            "only 'updated_at' should be in the posts[0] object"
        );
    }

    #[test]
    fn test_post_update_envelope_deserialization() {
        let json = json!({
            "posts": [{"updated_at": UPDATED_AT, "status": "published"}]
        });
        let env: PostUpdateEnvelope = serde_json::from_value(json).unwrap();
        assert_eq!(env.posts.len(), 1);
        assert_eq!(env.posts[0].updated_at, UPDATED_AT);
        assert_eq!(env.posts[0].status, Some(PostStatus::Published));
    }

    #[test]
    fn test_post_update_envelope_clone() {
        let env = PostUpdateEnvelope::new(PostUpdate::new(UPDATED_AT));
        let cloned = env.clone();
        assert_eq!(cloned.posts[0].updated_at, UPDATED_AT);
    }
}
