//! Author model for Ghost API.
//!
//! Authors are the people who write posts and pages in Ghost.
//!
//! # Example
//!
//! ```
//! use ghost_io_api::models::author::Author;
//!
//! let author = Author {
//!     id: "1".to_string(),
//!     name: "Jane Doe".to_string(),
//!     slug: "jane-doe".to_string(),
//!     ..Default::default()
//! };
//!
//! assert_eq!(author.name, "Jane Doe");
//! ```

use serde::{Deserialize, Serialize};

use crate::models::tag::PostCount;

/// A Ghost author resource.
///
/// # Example
///
/// ```
/// use ghost_io_api::models::author::Author;
/// use serde_json::json;
///
/// let json = json!({
///     "id": "1",
///     "name": "Jane Doe",
///     "slug": "jane-doe",
///     "profile_image": "https://example.com/avatar.jpg"
/// });
/// let author: Author = serde_json::from_value(json).unwrap();
/// assert_eq!(author.name, "Jane Doe");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Author {
    /// Ghost object ID.
    pub id: String,
    /// Display name.
    pub name: String,
    /// URL slug.
    pub slug: String,
    /// Profile image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_image: Option<String>,
    /// Cover image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_image: Option<String>,
    /// Author bio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    /// Personal website URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    /// Location string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Facebook profile URL or username.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facebook: Option<String>,
    /// Twitter handle (without `@`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter: Option<String>,
    /// Meta title override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_title: Option<String>,
    /// Meta description override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_description: Option<String>,
    /// Public URL of the author's archive page.
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

impl Author {
    /// Returns `true` if the author has a profile image set.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::author::Author;
    ///
    /// let author = Author {
    ///     profile_image: Some("https://example.com/img.jpg".to_string()),
    ///     ..Default::default()
    /// };
    /// assert!(author.has_profile_image());
    /// ```
    pub fn has_profile_image(&self) -> bool {
        self.profile_image.is_some()
    }

    /// Returns the number of published posts by this author, if available.
    ///
    /// Returns `None` unless `include=count.posts` was in the request.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::author::Author;
    /// use ghost_io_api::models::tag::PostCount;
    ///
    /// let author = Author {
    ///     count: Some(PostCount { posts: 5 }),
    ///     ..Default::default()
    /// };
    /// assert_eq!(author.post_count(), Some(5));
    /// ```
    pub fn post_count(&self) -> Option<u32> {
        self.count.as_ref().map(|c| c.posts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_author_minimal_deserialization() {
        let json = json!({ "id": "1", "name": "Jane Doe", "slug": "jane-doe" });
        let author: Author = serde_json::from_value(json).unwrap();
        assert_eq!(author.id, "1");
        assert_eq!(author.name, "Jane Doe");
        assert_eq!(author.slug, "jane-doe");
    }

    #[test]
    fn test_author_full_deserialization() {
        let json = json!({
            "id": "1",
            "name": "Jane Doe",
            "slug": "jane-doe",
            "profile_image": "https://example.com/avatar.jpg",
            "cover_image": "https://example.com/cover.jpg",
            "bio": "Writer",
            "website": "https://janedoe.com",
            "location": "New York",
            "facebook": "janedoe",
            "twitter": "@janedoe",
            "meta_title": "Jane Doe - Writer",
            "meta_description": "Posts by Jane",
            "url": "https://example.com/author/jane-doe/",
            "created_at": "2021-01-01T00:00:00.000Z",
            "updated_at": "2021-06-01T00:00:00.000Z",
            "count": { "posts": 12 }
        });
        let author: Author = serde_json::from_value(json).unwrap();
        assert_eq!(author.bio, Some("Writer".to_string()));
        assert!(author.has_profile_image());
        assert_eq!(author.post_count(), Some(12));
    }

    #[test]
    fn test_has_profile_image_true() {
        let author = Author {
            profile_image: Some("https://example.com/img.jpg".to_string()),
            ..Default::default()
        };
        assert!(author.has_profile_image());
    }

    #[test]
    fn test_has_profile_image_false() {
        let author = Author::default();
        assert!(!author.has_profile_image());
    }

    #[test]
    fn test_post_count_some() {
        let author = Author {
            count: Some(PostCount { posts: 7 }),
            ..Default::default()
        };
        assert_eq!(author.post_count(), Some(7));
    }

    #[test]
    fn test_post_count_none() {
        let author = Author::default();
        assert_eq!(author.post_count(), None);
    }

    #[test]
    fn test_author_serialization_skips_none() {
        let author = Author {
            id: "1".to_string(),
            name: "J".to_string(),
            slug: "j".to_string(),
            ..Default::default()
        };
        let json = serde_json::to_value(&author).unwrap();
        let obj = json.as_object().unwrap();
        assert!(!obj.contains_key("bio"));
        assert!(!obj.contains_key("profile_image"));
        assert!(!obj.contains_key("count"));
    }

    #[test]
    fn test_author_default() {
        let author = Author::default();
        assert!(author.id.is_empty());
        assert!(author.count.is_none());
        assert!(!author.has_profile_image());
    }

    #[test]
    fn test_author_clone_eq() {
        let author = Author {
            id: "1".to_string(),
            name: "J".to_string(),
            slug: "j".to_string(),
            ..Default::default()
        };
        assert_eq!(author, author.clone());
    }
}
