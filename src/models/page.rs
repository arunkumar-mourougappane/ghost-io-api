//! Page model for Ghost API.
//!
//! A Ghost page is a static content item (About, Contact, etc.) as opposed to
//! a chronological post. The schema mirrors [`super::post::Post`] closely.
//!
//! # Example
//!
//! ```
//! use ghost_io_api::models::page::{Page, PageStatus};
//!
//! let page = Page {
//!     id: "5e6f7a8b9c0d1e2f3a4b5c6d".to_string(),
//!     title: "About".to_string(),
//!     slug: "about".to_string(),
//!     status: PageStatus::Published,
//!     ..Default::default()
//! };
//!
//! assert!(page.is_published());
//! ```

use serde::{Deserialize, Serialize};

/// Publication status of a Ghost page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PageStatus {
    /// Draft — not publicly visible.
    #[default]
    Draft,
    /// Published and publicly accessible.
    Published,
    /// Scheduled for future publication.
    Scheduled,
}

impl PageStatus {
    /// Returns `true` if the page is published.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::page::PageStatus;
    ///
    /// assert!(PageStatus::Published.is_published());
    /// assert!(!PageStatus::Draft.is_published());
    /// ```
    pub fn is_published(&self) -> bool {
        matches!(self, PageStatus::Published)
    }

    /// Returns `true` if the page is a draft.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::page::PageStatus;
    ///
    /// assert!(PageStatus::Draft.is_draft());
    /// ```
    pub fn is_draft(&self) -> bool {
        matches!(self, PageStatus::Draft)
    }

    /// Returns `true` if the page is scheduled.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::page::PageStatus;
    ///
    /// assert!(PageStatus::Scheduled.is_scheduled());
    /// ```
    pub fn is_scheduled(&self) -> bool {
        matches!(self, PageStatus::Scheduled)
    }
}

/// A Ghost page resource.
///
/// Pages are static content items that are not part of the chronological
/// post feed (e.g. "About", "Contact").
///
/// # Example
///
/// ```
/// use ghost_io_api::models::page::{Page, PageStatus};
///
/// let page = Page {
///     id: "5e6f7a8b9c0d1e2f3a4b5c6d".to_string(),
///     title: "About".to_string(),
///     slug: "about".to_string(),
///     status: PageStatus::Published,
///     ..Default::default()
/// };
///
/// assert_eq!(page.slug, "about");
/// assert!(page.is_published());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Page {
    /// Ghost object ID.
    pub id: String,
    /// UUID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    /// Page title.
    pub title: String,
    /// URL slug.
    pub slug: String,
    /// Rendered HTML content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    /// Plaintext content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plaintext: Option<String>,
    /// Feature image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_image: Option<String>,
    /// Feature image alt text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_image_alt: Option<String>,
    /// Feature image caption.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_image_caption: Option<String>,
    /// Whether the page is featured.
    #[serde(default)]
    pub featured: bool,
    /// Publication status.
    #[serde(default)]
    pub status: PageStatus,
    /// Visibility: `"public"`, `"members"`, `"paid"`, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    /// ISO 8601 creation timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// ISO 8601 last-updated timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// ISO 8601 publication timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    /// Custom excerpt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_excerpt: Option<String>,
    /// Auto-generated excerpt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    /// Estimated reading time in minutes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reading_time: Option<u32>,
    /// Whether the page requires a paid membership to access.
    #[serde(default)]
    pub access: bool,
    /// Canonical URL override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_url: Option<String>,
    /// Public URL of the page on the site.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
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
    /// Custom template name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_template: Option<String>,
}

impl Page {
    /// Returns `true` if the page is published.
    pub fn is_published(&self) -> bool {
        self.status.is_published()
    }

    /// Returns `true` if the page is a draft.
    pub fn is_draft(&self) -> bool {
        self.status.is_draft()
    }

    /// Returns `true` if the page is scheduled.
    pub fn is_scheduled(&self) -> bool {
        self.status.is_scheduled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_page_status_default() {
        assert_eq!(PageStatus::default(), PageStatus::Draft);
    }

    #[test]
    fn test_page_status_is_published() {
        assert!(PageStatus::Published.is_published());
        assert!(!PageStatus::Draft.is_published());
        assert!(!PageStatus::Scheduled.is_published());
    }

    #[test]
    fn test_page_status_is_draft() {
        assert!(PageStatus::Draft.is_draft());
        assert!(!PageStatus::Published.is_draft());
    }

    #[test]
    fn test_page_status_is_scheduled() {
        assert!(PageStatus::Scheduled.is_scheduled());
        assert!(!PageStatus::Published.is_scheduled());
    }

    #[test]
    fn test_page_status_serialization() {
        assert_eq!(
            serde_json::to_string(&PageStatus::Published).unwrap(),
            "\"published\""
        );
        assert_eq!(
            serde_json::to_string(&PageStatus::Draft).unwrap(),
            "\"draft\""
        );
        assert_eq!(
            serde_json::to_string(&PageStatus::Scheduled).unwrap(),
            "\"scheduled\""
        );
    }

    #[test]
    fn test_page_status_deserialization() {
        let s: PageStatus = serde_json::from_str("\"published\"").unwrap();
        assert_eq!(s, PageStatus::Published);
        let d: PageStatus = serde_json::from_str("\"draft\"").unwrap();
        assert_eq!(d, PageStatus::Draft);
    }

    #[test]
    fn test_page_minimal_deserialization() {
        let json = json!({
            "id": "abc123",
            "title": "About",
            "slug": "about"
        });
        let page: Page = serde_json::from_value(json).unwrap();
        assert_eq!(page.id, "abc123");
        assert_eq!(page.title, "About");
        assert_eq!(page.slug, "about");
        assert_eq!(page.status, PageStatus::Draft);
        assert!(!page.featured);
    }

    #[test]
    fn test_page_full_deserialization() {
        let json = json!({
            "id": "abc123",
            "uuid": "a-b-c-d",
            "title": "About",
            "slug": "about",
            "html": "<p>Hello</p>",
            "plaintext": "Hello",
            "feature_image": "https://example.com/img.jpg",
            "featured": false,
            "status": "published",
            "visibility": "public",
            "created_at": "2021-01-01T00:00:00.000Z",
            "updated_at": "2021-01-02T00:00:00.000Z",
            "published_at": "2021-01-02T00:00:00.000Z",
            "custom_excerpt": "Custom",
            "excerpt": "Auto",
            "reading_time": 2,
            "access": true,
            "url": "https://example.com/about/"
        });
        let page: Page = serde_json::from_value(json).unwrap();
        assert_eq!(page.status, PageStatus::Published);
        assert!(page.is_published());
        assert_eq!(page.reading_time, Some(2));
        assert!(page.access);
    }

    #[test]
    fn test_page_default() {
        let page = Page::default();
        assert_eq!(page.status, PageStatus::Draft);
        assert!(page.id.is_empty());
        assert!(!page.featured);
        assert!(!page.access);
    }

    #[test]
    fn test_page_is_methods() {
        let published = Page {
            status: PageStatus::Published,
            ..Default::default()
        };
        assert!(published.is_published());
        assert!(!published.is_draft());
        assert!(!published.is_scheduled());

        let draft = Page {
            status: PageStatus::Draft,
            ..Default::default()
        };
        assert!(draft.is_draft());

        let scheduled = Page {
            status: PageStatus::Scheduled,
            ..Default::default()
        };
        assert!(scheduled.is_scheduled());
    }

    #[test]
    fn test_page_serialization_skips_none() {
        let page = Page {
            id: "1".to_string(),
            title: "T".to_string(),
            slug: "t".to_string(),
            ..Default::default()
        };
        let json = serde_json::to_value(&page).unwrap();
        let obj = json.as_object().unwrap();
        assert!(!obj.contains_key("html"));
        assert!(!obj.contains_key("feature_image"));
        assert!(!obj.contains_key("og_image"));
    }

    #[test]
    fn test_page_clone_eq() {
        let page = Page {
            id: "1".to_string(),
            title: "T".to_string(),
            slug: "t".to_string(),
            status: PageStatus::Published,
            ..Default::default()
        };
        assert_eq!(page, page.clone());
    }
}
