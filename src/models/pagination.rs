//! Pagination types for Ghost API browse endpoints.
//!
//! Ghost API browse endpoints return paginated results with metadata about
//! the current page, total pages, and navigation links.
//!
//! # Example
//!
//! ```
//! use ghost_io_api::models::pagination::{Meta, Pagination};
//!
//! // Typically you won't construct these manually, they come from API responses
//! let pagination = Pagination {
//!     page: 1,
//!     limit: 15,
//!     pages: 5,
//!     total: 73,
//!     next: Some(2),
//!     prev: None,
//! };
//!
//! // Check if there are more pages
//! assert!(pagination.has_next());
//! assert!(!pagination.has_prev());
//! ```

use serde::{Deserialize, Serialize};

/// Metadata wrapper returned by Ghost API browse endpoints.
///
/// Browse endpoints return paginated results with a `meta` object containing
/// pagination information.
///
/// # Example
///
/// ```
/// use ghost_io_api::models::pagination::Meta;
/// use serde_json::json;
///
/// let json = json!({
///     "pagination": {
///         "page": 1,
///         "limit": 15,
///         "pages": 5,
///         "total": 73,
///         "next": 2,
///         "prev": null
///     }
/// });
///
/// let meta: Meta = serde_json::from_value(json).unwrap();
/// assert_eq!(meta.pagination.page, 1);
/// assert_eq!(meta.pagination.total, 73);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meta {
    /// Pagination information for the current request.
    pub pagination: Pagination,
}

/// Pagination information for browse endpoints.
///
/// Contains information about the current page, total number of pages,
/// and links to the next/previous pages.
///
/// # Fields
///
/// * `page` - Current page number (1-indexed)
/// * `limit` - Number of items per page
/// * `pages` - Total number of pages available
/// * `total` - Total number of items across all pages
/// * `next` - Page number of the next page, if available
/// * `prev` - Page number of the previous page, if available
///
/// # Example
///
/// ```
/// use ghost_io_api::models::pagination::Pagination;
/// use serde_json::json;
///
/// let json = json!({
///     "page": 2,
///     "limit": 10,
///     "pages": 5,
///     "total": 47,
///     "next": 3,
///     "prev": 1
/// });
///
/// let pagination: Pagination = serde_json::from_value(json).unwrap();
/// assert_eq!(pagination.page, 2);
/// assert!(pagination.has_next());
/// assert!(pagination.has_prev());
/// assert!(!pagination.is_first_page());
/// assert!(!pagination.is_last_page());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pagination {
    /// Current page number (1-indexed).
    pub page: u32,

    /// Number of items per page.
    pub limit: u32,

    /// Total number of pages available.
    pub pages: u32,

    /// Total number of items across all pages.
    pub total: u32,

    /// Page number of the next page, if available.
    ///
    /// `None` if this is the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<u32>,

    /// Page number of the previous page, if available.
    ///
    /// `None` if this is the first page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<u32>,
}

impl Pagination {
    /// Returns `true` if there is a next page available.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::pagination::Pagination;
    ///
    /// let pagination = Pagination {
    ///     page: 1,
    ///     limit: 15,
    ///     pages: 5,
    ///     total: 73,
    ///     next: Some(2),
    ///     prev: None,
    /// };
    ///
    /// assert!(pagination.has_next());
    /// ```
    pub fn has_next(&self) -> bool {
        self.next.is_some()
    }

    /// Returns `true` if there is a previous page available.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::pagination::Pagination;
    ///
    /// let pagination = Pagination {
    ///     page: 2,
    ///     limit: 15,
    ///     pages: 5,
    ///     total: 73,
    ///     next: Some(3),
    ///     prev: Some(1),
    /// };
    ///
    /// assert!(pagination.has_prev());
    /// ```
    pub fn has_prev(&self) -> bool {
        self.prev.is_some()
    }

    /// Returns `true` if this is the first page.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::pagination::Pagination;
    ///
    /// let pagination = Pagination {
    ///     page: 1,
    ///     limit: 15,
    ///     pages: 5,
    ///     total: 73,
    ///     next: Some(2),
    ///     prev: None,
    /// };
    ///
    /// assert!(pagination.is_first_page());
    /// ```
    pub fn is_first_page(&self) -> bool {
        self.page == 1
    }

    /// Returns `true` if this is the last page.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::pagination::Pagination;
    ///
    /// let pagination = Pagination {
    ///     page: 5,
    ///     limit: 15,
    ///     pages: 5,
    ///     total: 73,
    ///     next: None,
    ///     prev: Some(4),
    /// };
    ///
    /// assert!(pagination.is_last_page());
    /// ```
    pub fn is_last_page(&self) -> bool {
        self.page == self.pages
    }

    /// Returns the number of items on the current page.
    ///
    /// For all pages except potentially the last one, this will equal `limit`.
    /// On the last page, it may be less than `limit`.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::pagination::Pagination;
    ///
    /// let pagination = Pagination {
    ///     page: 5,
    ///     limit: 15,
    ///     pages: 5,
    ///     total: 73,
    ///     next: None,
    ///     prev: Some(4),
    /// };
    ///
    /// // Last page: 73 total - (4 * 15) = 13 items on this page
    /// assert_eq!(pagination.items_on_page(), 13);
    /// ```
    pub fn items_on_page(&self) -> u32 {
        if self.is_last_page() {
            // Calculate remaining items on last page
            let items_before_last_page = (self.pages - 1) * self.limit;
            self.total.saturating_sub(items_before_last_page)
        } else {
            self.limit
        }
    }

    /// Returns the starting item index for the current page (0-indexed).
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::pagination::Pagination;
    ///
    /// let pagination = Pagination {
    ///     page: 2,
    ///     limit: 15,
    ///     pages: 5,
    ///     total: 73,
    ///     next: Some(3),
    ///     prev: Some(1),
    /// };
    ///
    /// // Page 2, limit 15: starts at index 15
    /// assert_eq!(pagination.start_index(), 15);
    /// ```
    pub fn start_index(&self) -> u32 {
        (self.page - 1) * self.limit
    }

    /// Returns the ending item index for the current page (0-indexed, exclusive).
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::models::pagination::Pagination;
    ///
    /// let pagination = Pagination {
    ///     page: 2,
    ///     limit: 15,
    ///     pages: 5,
    ///     total: 73,
    ///     next: Some(3),
    ///     prev: Some(1),
    /// };
    ///
    /// // Page 2, limit 15: ends at index 30 (exclusive)
    /// assert_eq!(pagination.end_index(), 30);
    /// ```
    pub fn end_index(&self) -> u32 {
        self.start_index() + self.items_on_page()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_pagination_deserialization() {
        let json = json!({
            "page": 1,
            "limit": 15,
            "pages": 5,
            "total": 73,
            "next": 2,
            "prev": null
        });

        let pagination: Pagination = serde_json::from_value(json).unwrap();
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.limit, 15);
        assert_eq!(pagination.pages, 5);
        assert_eq!(pagination.total, 73);
        assert_eq!(pagination.next, Some(2));
        assert_eq!(pagination.prev, None);
    }

    #[test]
    fn test_pagination_middle_page() {
        let json = json!({
            "page": 3,
            "limit": 10,
            "pages": 5,
            "total": 47,
            "next": 4,
            "prev": 2
        });

        let pagination: Pagination = serde_json::from_value(json).unwrap();
        assert_eq!(pagination.page, 3);
        assert_eq!(pagination.next, Some(4));
        assert_eq!(pagination.prev, Some(2));
    }

    #[test]
    fn test_pagination_last_page() {
        let json = json!({
            "page": 5,
            "limit": 15,
            "pages": 5,
            "total": 73,
            "next": null,
            "prev": 4
        });

        let pagination: Pagination = serde_json::from_value(json).unwrap();
        assert_eq!(pagination.page, 5);
        assert_eq!(pagination.next, None);
        assert_eq!(pagination.prev, Some(4));
    }

    #[test]
    fn test_meta_deserialization() {
        let json = json!({
            "pagination": {
                "page": 1,
                "limit": 15,
                "pages": 5,
                "total": 73,
                "next": 2,
                "prev": null
            }
        });

        let meta: Meta = serde_json::from_value(json).unwrap();
        assert_eq!(meta.pagination.page, 1);
        assert_eq!(meta.pagination.total, 73);
    }

    #[test]
    fn test_has_next() {
        let pagination = Pagination {
            page: 1,
            limit: 15,
            pages: 5,
            total: 73,
            next: Some(2),
            prev: None,
        };
        assert!(pagination.has_next());

        let last_page = Pagination {
            page: 5,
            limit: 15,
            pages: 5,
            total: 73,
            next: None,
            prev: Some(4),
        };
        assert!(!last_page.has_next());
    }

    #[test]
    fn test_has_prev() {
        let pagination = Pagination {
            page: 2,
            limit: 15,
            pages: 5,
            total: 73,
            next: Some(3),
            prev: Some(1),
        };
        assert!(pagination.has_prev());

        let first_page = Pagination {
            page: 1,
            limit: 15,
            pages: 5,
            total: 73,
            next: Some(2),
            prev: None,
        };
        assert!(!first_page.has_prev());
    }

    #[test]
    fn test_is_first_page() {
        let pagination = Pagination {
            page: 1,
            limit: 15,
            pages: 5,
            total: 73,
            next: Some(2),
            prev: None,
        };
        assert!(pagination.is_first_page());

        let not_first = Pagination {
            page: 2,
            limit: 15,
            pages: 5,
            total: 73,
            next: Some(3),
            prev: Some(1),
        };
        assert!(!not_first.is_first_page());
    }

    #[test]
    fn test_is_last_page() {
        let pagination = Pagination {
            page: 5,
            limit: 15,
            pages: 5,
            total: 73,
            next: None,
            prev: Some(4),
        };
        assert!(pagination.is_last_page());

        let not_last = Pagination {
            page: 4,
            limit: 15,
            pages: 5,
            total: 73,
            next: Some(5),
            prev: Some(3),
        };
        assert!(!not_last.is_last_page());
    }

    #[test]
    fn test_items_on_page() {
        // Full page
        let full_page = Pagination {
            page: 1,
            limit: 15,
            pages: 5,
            total: 73,
            next: Some(2),
            prev: None,
        };
        assert_eq!(full_page.items_on_page(), 15);

        // Last page with partial items: 73 - (4 * 15) = 13
        let last_page = Pagination {
            page: 5,
            limit: 15,
            pages: 5,
            total: 73,
            next: None,
            prev: Some(4),
        };
        assert_eq!(last_page.items_on_page(), 13);
    }

    #[test]
    fn test_start_index() {
        let page_1 = Pagination {
            page: 1,
            limit: 15,
            pages: 5,
            total: 73,
            next: Some(2),
            prev: None,
        };
        assert_eq!(page_1.start_index(), 0);

        let page_2 = Pagination {
            page: 2,
            limit: 15,
            pages: 5,
            total: 73,
            next: Some(3),
            prev: Some(1),
        };
        assert_eq!(page_2.start_index(), 15);

        let page_5 = Pagination {
            page: 5,
            limit: 15,
            pages: 5,
            total: 73,
            next: None,
            prev: Some(4),
        };
        assert_eq!(page_5.start_index(), 60);
    }

    #[test]
    fn test_end_index() {
        let page_1 = Pagination {
            page: 1,
            limit: 15,
            pages: 5,
            total: 73,
            next: Some(2),
            prev: None,
        };
        assert_eq!(page_1.end_index(), 15);

        let page_2 = Pagination {
            page: 2,
            limit: 15,
            pages: 5,
            total: 73,
            next: Some(3),
            prev: Some(1),
        };
        assert_eq!(page_2.end_index(), 30);

        // Last page: starts at 60, has 13 items
        let page_5 = Pagination {
            page: 5,
            limit: 15,
            pages: 5,
            total: 73,
            next: None,
            prev: Some(4),
        };
        assert_eq!(page_5.end_index(), 73);
    }

    #[test]
    fn test_pagination_serialization() {
        let pagination = Pagination {
            page: 2,
            limit: 10,
            pages: 5,
            total: 47,
            next: Some(3),
            prev: Some(1),
        };

        let json = serde_json::to_value(&pagination).unwrap();
        assert_eq!(json["page"], 2);
        assert_eq!(json["limit"], 10);
        assert_eq!(json["pages"], 5);
        assert_eq!(json["total"], 47);
        assert_eq!(json["next"], 3);
        assert_eq!(json["prev"], 1);
    }

    #[test]
    fn test_pagination_serialization_with_nulls() {
        let pagination = Pagination {
            page: 1,
            limit: 15,
            pages: 1,
            total: 10,
            next: None,
            prev: None,
        };

        let json = serde_json::to_value(&pagination).unwrap();
        // Fields with None should be omitted due to skip_serializing_if
        assert!(!json.as_object().unwrap().contains_key("next"));
        assert!(!json.as_object().unwrap().contains_key("prev"));
    }

    #[test]
    fn test_single_page() {
        let pagination = Pagination {
            page: 1,
            limit: 15,
            pages: 1,
            total: 10,
            next: None,
            prev: None,
        };

        assert!(pagination.is_first_page());
        assert!(pagination.is_last_page());
        assert!(!pagination.has_next());
        assert!(!pagination.has_prev());
        assert_eq!(pagination.items_on_page(), 10);
        assert_eq!(pagination.start_index(), 0);
        assert_eq!(pagination.end_index(), 10);
    }

    #[test]
    fn test_meta_serialization() {
        let meta = Meta {
            pagination: Pagination {
                page: 1,
                limit: 15,
                pages: 5,
                total: 73,
                next: Some(2),
                prev: None,
            },
        };

        let json = serde_json::to_value(&meta).unwrap();
        assert_eq!(json["pagination"]["page"], 1);
        assert_eq!(json["pagination"]["total"], 73);
    }

    #[test]
    fn test_pagination_clone_and_eq() {
        let pagination = Pagination {
            page: 1,
            limit: 15,
            pages: 5,
            total: 73,
            next: Some(2),
            prev: None,
        };

        let cloned = pagination.clone();
        assert_eq!(pagination, cloned);
    }

    #[test]
    fn test_meta_clone_and_eq() {
        let meta = Meta {
            pagination: Pagination {
                page: 1,
                limit: 15,
                pages: 5,
                total: 73,
                next: Some(2),
                prev: None,
            },
        };

        let cloned = meta.clone();
        assert_eq!(meta, cloned);
    }
}
