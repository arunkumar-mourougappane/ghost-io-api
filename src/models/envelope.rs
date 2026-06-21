//! Generic response envelopes for Ghost API endpoints.
//!
//! Ghost wraps every response in a resource-keyed JSON object. Browse endpoints
//! include a `meta` block for pagination; read, create, and update endpoints
//! return a one-element array without `meta`. These two envelope shapes are
//! modelled by [`BrowseEnvelope`] and [`SingleEnvelope`] respectively.
//!
//! Both types use a custom `Deserialize` implementation that locates the
//! resource array by scanning the JSON object for the first array-valued field
//! (skipping `meta`). This means a single generic type works for every Ghost
//! resource — `posts`, `pages`, `tags`, `authors`, and any future resource —
//! without requiring per-resource newtype wrappers.
//!
//! # Examples
//!
//! ```
//! use ghost_io_api::models::envelope::{BrowseEnvelope, SingleEnvelope};
//! use serde::Deserialize;
//! use serde_json::json;
//!
//! #[derive(Debug, Deserialize)]
//! struct Widget { id: String, name: String }
//!
//! // Browse: items + pagination
//! let browse_json = json!({
//!     "widgets": [
//!         {"id": "w1", "name": "Alpha"},
//!         {"id": "w2", "name": "Beta"}
//!     ],
//!     "meta": {
//!         "pagination": {
//!             "page": 1, "limit": 15, "pages": 1, "total": 2
//!         }
//!     }
//! });
//! let env: BrowseEnvelope<Widget> = serde_json::from_value(browse_json).unwrap();
//! assert_eq!(env.items.len(), 2);
//! assert_eq!(env.meta.pagination.total, 2);
//!
//! // Single: one item in the array, no meta
//! let read_json = json!({ "widgets": [{"id": "w1", "name": "Alpha"}] });
//! let env: SingleEnvelope<Widget> = serde_json::from_value(read_json).unwrap();
//! assert_eq!(env.item.id, "w1");
//! ```

use crate::models::pagination::Meta;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

// ── BrowseEnvelope ────────────────────────────────────────────────────────────

/// Response envelope for Ghost browse endpoints.
///
/// Browse endpoints return `{"<resource>": [...], "meta": {...}}`. The resource
/// key differs by type (`"posts"`, `"tags"`, `"authors"`, …), but the shape is
/// always the same: an array of items plus a `"meta"` block containing
/// pagination details.
///
/// `BrowseEnvelope<T>` deserializes any such object by scanning for the first
/// array-valued field that is not `"meta"`. You never need to specify the
/// resource key; the same type works for every Ghost resource.
///
/// # Fields
///
/// * `items` — the deserialized resource objects from the current page.
/// * `meta`  — pagination metadata (`page`, `limit`, `pages`, `total`, …).
///
/// # Example
///
/// ```
/// use ghost_io_api::models::envelope::BrowseEnvelope;
/// use serde::Deserialize;
/// use serde_json::json;
///
/// #[derive(Debug, Deserialize)]
/// struct Post { id: String, title: String }
///
/// let json = json!({
///     "posts": [{"id": "abc", "title": "Hello"}],
///     "meta": {
///         "pagination": {"page": 1, "limit": 15, "pages": 1, "total": 1}
///     }
/// });
///
/// let env: BrowseEnvelope<Post> = serde_json::from_value(json).unwrap();
/// assert_eq!(env.items[0].id, "abc");
/// assert_eq!(env.meta.pagination.page, 1);
/// assert_eq!(env.meta.pagination.total, 1);
/// ```
#[derive(Debug, Clone)]
pub struct BrowseEnvelope<T> {
    /// The resource items returned by this page.
    pub items: Vec<T>,
    /// Pagination metadata.
    pub meta: Meta,
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for BrowseEnvelope<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;

        let mut map: serde_json::Map<String, Value> = serde_json::Map::deserialize(deserializer)?;

        // Extract `meta` before scanning for items so it is not mistaken for
        // an array-valued resource field (it is an object, but just in case).
        let meta_val = map.remove("meta").unwrap_or(Value::Null);
        let meta = serde_json::from_value::<Meta>(meta_val).map_err(Error::custom)?;

        // The resource array is the first (and typically only) remaining field
        // that holds a JSON array.
        let items_val = map
            .into_values()
            .find(|v| v.is_array())
            .ok_or_else(|| Error::custom("no resource array found in browse envelope"))?;

        let items = serde_json::from_value::<Vec<T>>(items_val).map_err(Error::custom)?;

        Ok(Self { items, meta })
    }
}

// ── SingleEnvelope ────────────────────────────────────────────────────────────

/// Response envelope for Ghost read, create, and update endpoints.
///
/// These endpoints return `{"<resource>": [item]}` — a one-element array
/// wrapped in a resource-keyed object, with no `"meta"` block. `SingleEnvelope`
/// deserializes any such object, extracts the first element of the array, and
/// stores it as `item`.
///
/// As with [`BrowseEnvelope`], the resource key (`"posts"`, `"pages"`, …) is
/// detected automatically; no per-resource specialisation is required.
///
/// # Fields
///
/// * `item` — the single deserialized resource object.
///
/// # Errors
///
/// Deserialization fails if the JSON object contains no array-valued field, or
/// if that array is empty.
///
/// # Example
///
/// ```
/// use ghost_io_api::models::envelope::SingleEnvelope;
/// use serde::Deserialize;
/// use serde_json::json;
///
/// #[derive(Debug, Deserialize)]
/// struct Post { id: String, title: String }
///
/// // Read by ID
/// let json = json!({"posts": [{"id": "abc", "title": "Hello"}]});
/// let env: SingleEnvelope<Post> = serde_json::from_value(json).unwrap();
/// assert_eq!(env.item.id, "abc");
/// assert_eq!(env.item.title, "Hello");
/// ```
#[derive(Debug, Clone)]
pub struct SingleEnvelope<T> {
    /// The single resource item extracted from the array.
    pub item: T,
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for SingleEnvelope<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;

        let map: serde_json::Map<String, Value> = serde_json::Map::deserialize(deserializer)?;

        // Skip `meta` if present (some endpoints include it even for single
        // reads), then locate the resource array.
        let items_val = map
            .into_iter()
            .filter(|(k, _)| k != "meta")
            .find_map(|(_, v)| v.is_array().then_some(v))
            .ok_or_else(|| Error::custom("no resource array found in single-item envelope"))?;

        let mut items = serde_json::from_value::<Vec<T>>(items_val).map_err(Error::custom)?;

        let item = items
            .drain(..)
            .next()
            .ok_or_else(|| Error::custom("resource array is empty in single-item envelope"))?;

        Ok(Self { item })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    // ── Fixture types ─────────────────────────────────────────────────────────

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    struct Post {
        id: String,
        title: String,
        #[serde(default)]
        status: String,
    }

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    struct Tag {
        id: String,
        name: String,
        slug: String,
    }

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    struct Author {
        id: String,
        email: String,
    }

    fn pagination(page: u32, total: u32) -> serde_json::Value {
        json!({
            "pagination": {
                "page": page, "limit": 15,
                "pages": 1, "total": total,
                "next": null, "prev": null
            }
        })
    }

    // ── BrowseEnvelope — happy path ───────────────────────────────────────────

    #[test]
    fn test_browse_envelope_posts() {
        let v = json!({
            "posts": [
                {"id": "1", "title": "First", "status": "published"},
                {"id": "2", "title": "Second", "status": "draft"}
            ],
            "meta": pagination(1, 2)
        });
        let env: BrowseEnvelope<Post> = serde_json::from_value(v).unwrap();
        assert_eq!(env.items.len(), 2);
        assert_eq!(env.items[0].id, "1");
        assert_eq!(env.items[1].title, "Second");
        assert_eq!(env.meta.pagination.total, 2);
        assert_eq!(env.meta.pagination.page, 1);
    }

    #[test]
    fn test_browse_envelope_tags() {
        let v = json!({
            "tags": [{"id": "t1", "name": "Rust", "slug": "rust"}],
            "meta": pagination(1, 1)
        });
        let env: BrowseEnvelope<Tag> = serde_json::from_value(v).unwrap();
        assert_eq!(env.items.len(), 1);
        assert_eq!(env.items[0].slug, "rust");
        assert_eq!(env.meta.pagination.total, 1);
    }

    #[test]
    fn test_browse_envelope_authors() {
        let v = json!({
            "authors": [{"id": "a1", "email": "jane@example.com"}],
            "meta": pagination(1, 1)
        });
        let env: BrowseEnvelope<Author> = serde_json::from_value(v).unwrap();
        assert_eq!(env.items[0].email, "jane@example.com");
    }

    #[test]
    fn test_browse_envelope_empty_list() {
        let v = json!({
            "posts": [],
            "meta": pagination(1, 0)
        });
        let env: BrowseEnvelope<Post> = serde_json::from_value(v).unwrap();
        assert!(env.items.is_empty());
        assert_eq!(env.meta.pagination.total, 0);
    }

    #[test]
    fn test_browse_envelope_multiple_pages() {
        let v = json!({
            "posts": [{"id": "p1", "title": "Page 2 post"}],
            "meta": {
                "pagination": {
                    "page": 2, "limit": 1,
                    "pages": 3, "total": 3,
                    "next": 3, "prev": 1
                }
            }
        });
        let env: BrowseEnvelope<Post> = serde_json::from_value(v).unwrap();
        assert_eq!(env.meta.pagination.page, 2);
        assert_eq!(env.meta.pagination.pages, 3);
        assert_eq!(env.meta.pagination.next, Some(3));
        assert_eq!(env.meta.pagination.prev, Some(1));
    }

    #[test]
    fn test_browse_envelope_extra_fields_ignored() {
        // Ghost may add extra top-level keys in future — ensure they are ignored
        let v = json!({
            "posts": [{"id": "x", "title": "X"}],
            "meta": pagination(1, 1),
            "some_future_key": "value"
        });
        let env: BrowseEnvelope<Post> = serde_json::from_value(v).unwrap();
        assert_eq!(env.items.len(), 1);
    }

    #[test]
    fn test_browse_envelope_many_items() {
        let posts: Vec<serde_json::Value> = (1..=20)
            .map(|i| json!({"id": i.to_string(), "title": format!("Post {i}")}))
            .collect();
        let v = json!({"posts": posts, "meta": pagination(1, 20)});
        let env: BrowseEnvelope<Post> = serde_json::from_value(v).unwrap();
        assert_eq!(env.items.len(), 20);
        assert_eq!(env.items[19].title, "Post 20");
    }

    // ── BrowseEnvelope — error cases ──────────────────────────────────────────

    #[test]
    fn test_browse_envelope_missing_array_errors() {
        let v = json!({"meta": pagination(1, 0)});
        let err = serde_json::from_value::<BrowseEnvelope<Post>>(v).unwrap_err();
        assert!(err.to_string().contains("no resource array"));
    }

    #[test]
    fn test_browse_envelope_not_an_object_errors() {
        let v = json!([1, 2, 3]);
        assert!(serde_json::from_value::<BrowseEnvelope<Post>>(v).is_err());
    }

    #[test]
    fn test_browse_envelope_null_errors() {
        assert!(serde_json::from_value::<BrowseEnvelope<Post>>(json!(null)).is_err());
    }

    #[test]
    fn test_browse_envelope_bad_item_type_errors() {
        // `status` should be a string, but we pass a number — deserialization of
        // Post should fail and propagate through BrowseEnvelope.
        let v = json!({
            "posts": [{"id": 99, "title": "Bad", "status": 0}],
            "meta": pagination(1, 1)
        });
        // id is a String field, so 99 as a number should fail
        assert!(serde_json::from_value::<BrowseEnvelope<Post>>(v).is_err());
    }

    // ── SingleEnvelope — happy path ───────────────────────────────────────────

    #[test]
    fn test_single_envelope_posts() {
        let v = json!({"posts": [{"id": "abc", "title": "Hello", "status": "published"}]});
        let env: SingleEnvelope<Post> = serde_json::from_value(v).unwrap();
        assert_eq!(env.item.id, "abc");
        assert_eq!(env.item.title, "Hello");
        assert_eq!(env.item.status, "published");
    }

    #[test]
    fn test_single_envelope_tags() {
        let v = json!({"tags": [{"id": "t1", "name": "Rust", "slug": "rust"}]});
        let env: SingleEnvelope<Tag> = serde_json::from_value(v).unwrap();
        assert_eq!(env.item.name, "Rust");
        assert_eq!(env.item.slug, "rust");
    }

    #[test]
    fn test_single_envelope_authors() {
        let v = json!({"authors": [{"id": "a1", "email": "jane@example.com"}]});
        let env: SingleEnvelope<Author> = serde_json::from_value(v).unwrap();
        assert_eq!(env.item.email, "jane@example.com");
    }

    #[test]
    fn test_single_envelope_ignores_meta_if_present() {
        // Some Ghost endpoints include meta even on single-item reads
        let v = json!({
            "posts": [{"id": "x", "title": "With meta"}],
            "meta": pagination(1, 1)
        });
        let env: SingleEnvelope<Post> = serde_json::from_value(v).unwrap();
        assert_eq!(env.item.id, "x");
    }

    #[test]
    fn test_single_envelope_extra_fields_ignored() {
        let v = json!({
            "posts": [{"id": "y", "title": "Extra"}],
            "some_key": "ignored"
        });
        let env: SingleEnvelope<Post> = serde_json::from_value(v).unwrap();
        assert_eq!(env.item.id, "y");
    }

    #[test]
    fn test_single_envelope_picks_first_item_when_multiple() {
        // Ghost always returns one item, but we verify we pick the first
        let v = json!({"posts": [
            {"id": "first", "title": "First"},
            {"id": "second", "title": "Second"}
        ]});
        let env: SingleEnvelope<Post> = serde_json::from_value(v).unwrap();
        assert_eq!(env.item.id, "first");
    }

    // ── SingleEnvelope — error cases ──────────────────────────────────────────

    #[test]
    fn test_single_envelope_empty_array_errors() {
        let v = json!({"posts": []});
        let err = serde_json::from_value::<SingleEnvelope<Post>>(v).unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn test_single_envelope_missing_array_errors() {
        let v = json!({"not_an_array": "hello"});
        let err = serde_json::from_value::<SingleEnvelope<Post>>(v).unwrap_err();
        assert!(err.to_string().contains("no resource array"));
    }

    #[test]
    fn test_single_envelope_not_an_object_errors() {
        assert!(serde_json::from_value::<SingleEnvelope<Post>>(json!(42)).is_err());
    }

    #[test]
    fn test_single_envelope_null_errors() {
        assert!(serde_json::from_value::<SingleEnvelope<Post>>(json!(null)).is_err());
    }

    // ── Clone & Debug ─────────────────────────────────────────────────────────

    #[test]
    fn test_browse_envelope_clone() {
        let v = json!({
            "posts": [{"id": "1", "title": "Clone me"}],
            "meta": pagination(1, 1)
        });
        let env: BrowseEnvelope<Post> = serde_json::from_value(v).unwrap();
        let cloned = env.clone();
        assert_eq!(cloned.items[0].id, env.items[0].id);
        assert_eq!(cloned.meta.pagination.total, env.meta.pagination.total);
    }

    #[test]
    fn test_browse_envelope_debug() {
        let v = json!({
            "posts": [{"id": "d1", "title": "Debug"}],
            "meta": pagination(1, 1)
        });
        let env: BrowseEnvelope<Post> = serde_json::from_value(v).unwrap();
        let debug = format!("{env:?}");
        assert!(debug.contains("BrowseEnvelope"));
    }

    #[test]
    fn test_single_envelope_clone() {
        let v = json!({"posts": [{"id": "c1", "title": "Clone"}]});
        let env: SingleEnvelope<Post> = serde_json::from_value(v).unwrap();
        let cloned = env.clone();
        assert_eq!(cloned.item.id, env.item.id);
    }

    #[test]
    fn test_single_envelope_debug() {
        let v = json!({"posts": [{"id": "d2", "title": "Debug"}]});
        let env: SingleEnvelope<Post> = serde_json::from_value(v).unwrap();
        let debug = format!("{env:?}");
        assert!(debug.contains("SingleEnvelope"));
    }

    // ── Real Ghost model round-trips ──────────────────────────────────────────

    #[test]
    fn test_browse_envelope_with_real_post_model() {
        use crate::models::post::Post;
        let v = json!({
            "posts": [{
                "id": "5ddc9141c35e7700383b2937",
                "title": "Welcome",
                "slug": "welcome",
                "status": "published"
            }],
            "meta": pagination(1, 1)
        });
        let env: BrowseEnvelope<Post> = serde_json::from_value(v).unwrap();
        assert_eq!(env.items[0].id, "5ddc9141c35e7700383b2937");
        assert_eq!(env.items[0].title, "Welcome");
    }

    #[test]
    fn test_single_envelope_with_real_post_model() {
        use crate::models::post::Post;
        let v = json!({
            "posts": [{
                "id": "abc123",
                "title": "Created Post",
                "slug": "created-post",
                "status": "draft"
            }]
        });
        let env: SingleEnvelope<Post> = serde_json::from_value(v).unwrap();
        assert_eq!(env.item.id, "abc123");
        assert_eq!(env.item.title, "Created Post");
    }
}
