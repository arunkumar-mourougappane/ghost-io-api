//! Settings model for Ghost API.
//!
//! The Ghost Content API exposes a read-only subset of site settings via the
//! `/ghost/api/content/settings/` endpoint.
//!
//! # Example
//!
//! ```
//! use ghost_io_api::models::settings::Settings;
//!
//! let settings = Settings {
//!     title: Some("My Blog".to_string()),
//!     description: Some("A blog about things".to_string()),
//!     ..Default::default()
//! };
//!
//! assert_eq!(settings.title.as_deref(), Some("My Blog"));
//! ```

use serde::{Deserialize, Serialize};

/// A navigation item in the site menu.
///
/// # Example
///
/// ```
/// use ghost_io_api::models::settings::NavItem;
/// use serde_json::json;
///
/// let json = json!({ "label": "Home", "url": "/" });
/// let item: NavItem = serde_json::from_value(json).unwrap();
/// assert_eq!(item.label, "Home");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NavItem {
    /// Display label.
    pub label: String,
    /// Link URL.
    pub url: String,
}

/// Site-wide settings returned by the Ghost Content API.
///
/// All fields are optional because the API may omit fields that have not been
/// configured by the site owner.
///
/// # Example
///
/// ```
/// use ghost_io_api::models::settings::Settings;
/// use serde_json::json;
///
/// let json = json!({
///     "title": "Ghost",
///     "description": "The professional publishing platform",
///     "logo": "https://example.com/logo.png",
///     "lang": "en",
///     "timezone": "Etc/UTC",
///     "navigation": [
///         { "label": "Home", "url": "/" },
///         { "label": "About", "url": "/about/" }
///     ]
/// });
///
/// let settings: Settings = serde_json::from_value(json).unwrap();
/// assert_eq!(settings.title.as_deref(), Some("Ghost"));
/// assert_eq!(settings.navigation.len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Settings {
    /// Site title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Site description / tagline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Logo image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    /// Site icon URL (favicon).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Accent colour hex code (e.g. `"#ff0095"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<String>,
    /// Cover image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_image: Option<String>,
    /// Facebook page URL or username.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facebook: Option<String>,
    /// Twitter handle (without `@`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter: Option<String>,
    /// Site locale/language code, e.g. `"en"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// Site timezone, e.g. `"Etc/UTC"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    /// Code injected into `<head>` site-wide.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codeinjection_head: Option<String>,
    /// Code injected at the end of `<body>` site-wide.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codeinjection_foot: Option<String>,
    /// Primary navigation items.
    #[serde(default)]
    pub navigation: Vec<NavItem>,
    /// Secondary navigation items.
    #[serde(default)]
    pub secondary_navigation: Vec<NavItem>,
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
    /// Members support email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members_support_address: Option<String>,
    /// Canonical site URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl Settings {
    /// Returns `true` if the site has a logo configured.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::settings::Settings;
    ///
    /// let s = Settings { logo: Some("https://example.com/logo.png".to_string()), ..Default::default() };
    /// assert!(s.has_logo());
    ///
    /// let empty = Settings::default();
    /// assert!(!empty.has_logo());
    /// ```
    pub fn has_logo(&self) -> bool {
        self.logo.is_some()
    }

    /// Returns the number of primary navigation items.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::settings::{Settings, NavItem};
    ///
    /// let s = Settings {
    ///     navigation: vec![
    ///         NavItem { label: "Home".to_string(), url: "/".to_string() },
    ///     ],
    ///     ..Default::default()
    /// };
    /// assert_eq!(s.nav_count(), 1);
    /// ```
    pub fn nav_count(&self) -> usize {
        self.navigation.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_nav_item_deserialization() {
        let json = json!({ "label": "Home", "url": "/" });
        let item: NavItem = serde_json::from_value(json).unwrap();
        assert_eq!(item.label, "Home");
        assert_eq!(item.url, "/");
    }

    #[test]
    fn test_settings_minimal_deserialization() {
        let json = json!({ "title": "My Blog" });
        let settings: Settings = serde_json::from_value(json).unwrap();
        assert_eq!(settings.title, Some("My Blog".to_string()));
        assert!(settings.navigation.is_empty());
    }

    #[test]
    fn test_settings_full_deserialization() {
        let json = json!({
            "title": "Ghost",
            "description": "Platform",
            "logo": "https://example.com/logo.png",
            "icon": "https://example.com/icon.png",
            "accent_color": "#ff0095",
            "cover_image": "https://example.com/cover.jpg",
            "facebook": "ghostorg",
            "twitter": "@Ghost",
            "lang": "en",
            "timezone": "Etc/UTC",
            "navigation": [
                { "label": "Home", "url": "/" },
                { "label": "About", "url": "/about/" }
            ],
            "secondary_navigation": [],
            "url": "https://example.com/"
        });
        let settings: Settings = serde_json::from_value(json).unwrap();
        assert_eq!(settings.title.as_deref(), Some("Ghost"));
        assert_eq!(settings.lang.as_deref(), Some("en"));
        assert_eq!(settings.navigation.len(), 2);
        assert_eq!(settings.navigation[0].label, "Home");
        assert!(settings.has_logo());
        assert_eq!(settings.nav_count(), 2);
    }

    #[test]
    fn test_settings_empty_navigation_default() {
        let settings = Settings::default();
        assert!(settings.navigation.is_empty());
        assert_eq!(settings.nav_count(), 0);
    }

    #[test]
    fn test_has_logo_true() {
        let s = Settings {
            logo: Some("https://example.com/logo.png".to_string()),
            ..Default::default()
        };
        assert!(s.has_logo());
    }

    #[test]
    fn test_has_logo_false() {
        assert!(!Settings::default().has_logo());
    }

    #[test]
    fn test_settings_serialization_skips_none() {
        let s = Settings {
            title: Some("T".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_value(&s).unwrap();
        let obj = json.as_object().unwrap();
        assert!(!obj.contains_key("logo"));
        assert!(!obj.contains_key("twitter"));
    }

    #[test]
    fn test_settings_clone_eq() {
        let s = Settings {
            title: Some("Blog".to_string()),
            lang: Some("en".to_string()),
            ..Default::default()
        };
        assert_eq!(s, s.clone());
    }

    #[test]
    fn test_nav_item_clone_eq() {
        let item = NavItem {
            label: "Home".to_string(),
            url: "/".to_string(),
        };
        assert_eq!(item, item.clone());
    }

    #[test]
    fn test_nav_item_default() {
        let item = NavItem::default();
        assert!(item.label.is_empty());
        assert!(item.url.is_empty());
    }
}
