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
}
