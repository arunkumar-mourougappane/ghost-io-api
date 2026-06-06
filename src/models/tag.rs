//! Tag model for Ghost API.
//!
//! Tags are used to categorise posts and pages. Each tag has a name, slug,
//! and optional metadata fields.
//!
//! # Example
//!
//! ```
//! use ghost_io_api::models::tag::Tag;
//!
//! let tag = Tag {
//!     id: "5e6f7a8b9c0d1e2f3a4b5c6e".to_string(),
//!     name: "Getting Started".to_string(),
//!     slug: "getting-started".to_string(),
//!     ..Default::default()
//! };
//!
//! assert_eq!(tag.name, "Getting Started");
//! assert!(tag.is_public());
//! ```

use serde::{Deserialize, Serialize};

/// Count of posts associated with a tag or author.
///
/// Populated when `include=count.posts` is added to the request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PostCount {
    /// Number of published posts with this tag / by this author.
    pub posts: u32,
}

/// A Ghost tag resource.
///
/// # Example
///
/// ```
/// use ghost_io_api::models::tag::Tag;
/// use serde_json::json;
///
/// let json = json!({
///     "id": "abc",
///     "name": "Rust",
///     "slug": "rust",
///     "visibility": "public"
/// });
/// let tag: Tag = serde_json::from_value(json).unwrap();
/// assert!(tag.is_public());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Tag {
    /// Ghost object ID.
    pub id: String,
    /// Display name.
    pub name: String,
    /// URL slug.
    pub slug: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Feature image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_image: Option<String>,
    /// Visibility: `"public"` or `"internal"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    /// OG image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_image: Option<String>,
    /// OG title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_title: Option<String>,
    /// OG description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_description: Option<String>,
    /// Twitter card image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_image: Option<String>,
    /// Twitter card title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_title: Option<String>,
    /// Twitter card description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_description: Option<String>,
    /// Meta title override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_title: Option<String>,
    /// Meta description override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_description: Option<String>,
    /// Code injected into `<head>` on tag pages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codeinjection_head: Option<String>,
    /// Code injected at the end of `<body>` on tag pages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codeinjection_foot: Option<String>,
    /// Canonical URL override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_url: Option<String>,
    /// Public URL of the tag archive page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// ISO 8601 creation timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// ISO 8601 last-updated timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// Post count (populated when `include=count.posts`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<PostCount>,
}

impl Tag {
    /// Returns `true` if the tag's visibility is `"public"` or unset.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::tag::Tag;
    ///
    /// let public_tag = Tag { visibility: Some("public".to_string()), ..Default::default() };
    /// assert!(public_tag.is_public());
    ///
    /// let internal_tag = Tag { visibility: Some("internal".to_string()), ..Default::default() };
    /// assert!(!internal_tag.is_public());
    /// ```
    pub fn is_public(&self) -> bool {
        matches!(self.visibility.as_deref(), None | Some("public"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tag_minimal_deserialization() {
        let json = json!({ "id": "1", "name": "Rust", "slug": "rust" });
        let tag: Tag = serde_json::from_value(json).unwrap();
        assert_eq!(tag.id, "1");
        assert_eq!(tag.name, "Rust");
        assert_eq!(tag.slug, "rust");
    }

    #[test]
    fn test_tag_full_deserialization() {
        let json = json!({
            "id": "1",
            "name": "Rust",
            "slug": "rust",
            "description": "Rust programming",
            "feature_image": "https://example.com/img.jpg",
            "visibility": "public",
            "og_image": null,
            "meta_title": "Rust Tag",
            "url": "https://example.com/tag/rust/",
            "created_at": "2021-01-01T00:00:00.000Z",
            "updated_at": "2021-01-01T00:00:00.000Z",
            "count": { "posts": 42 }
        });
        let tag: Tag = serde_json::from_value(json).unwrap();
        assert_eq!(tag.description, Some("Rust programming".to_string()));
        assert_eq!(tag.count, Some(PostCount { posts: 42 }));
        assert!(tag.is_public());
    }

    #[test]
    fn test_tag_is_public_with_public_visibility() {
        let tag = Tag {
            visibility: Some("public".to_string()),
            ..Default::default()
        };
        assert!(tag.is_public());
    }

    #[test]
    fn test_tag_is_public_with_no_visibility() {
        let tag = Tag::default();
        assert!(tag.is_public());
    }

    #[test]
    fn test_tag_is_not_public_when_internal() {
        let tag = Tag {
            visibility: Some("internal".to_string()),
            ..Default::default()
        };
        assert!(!tag.is_public());
    }

    #[test]
    fn test_tag_serialization_skips_none() {
        let tag = Tag {
            id: "1".to_string(),
            name: "T".to_string(),
            slug: "t".to_string(),
            ..Default::default()
        };
        let json = serde_json::to_value(&tag).unwrap();
        let obj = json.as_object().unwrap();
        assert!(!obj.contains_key("description"));
        assert!(!obj.contains_key("feature_image"));
        assert!(!obj.contains_key("count"));
    }

    #[test]
    fn test_post_count_default() {
        let count = PostCount::default();
        assert_eq!(count.posts, 0);
    }

    #[test]
    fn test_post_count_serialization() {
        let count = PostCount { posts: 10 };
        let json = serde_json::to_value(&count).unwrap();
        assert_eq!(json["posts"], 10);
    }

    #[test]
    fn test_tag_clone_eq() {
        let tag = Tag {
            id: "1".to_string(),
            name: "T".to_string(),
            slug: "t".to_string(),
            ..Default::default()
        };
        assert_eq!(tag, tag.clone());
    }

    #[test]
    fn test_tag_default() {
        let tag = Tag::default();
        assert!(tag.id.is_empty());
        assert!(tag.name.is_empty());
        assert!(tag.count.is_none());
    }
}
