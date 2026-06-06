//! Fluent builder for Ghost API browse endpoint query parameters.
//!
//! [`BrowseParams`] covers every standard query parameter accepted by Ghost
//! browse endpoints (`/posts/`, `/pages/`, `/tags/`, `/authors/`, etc.).
//!
//! # Example
//!
//! ```
//! use ghost_io_api::params::browse::BrowseParams;
//!
//! let params = BrowseParams::new()
//!     .limit(10)
//!     .page(2)
//!     .filter("featured:true")
//!     .order("published_at DESC")
//!     .include("authors,tags")
//!     .fields("id,title,slug")
//!     .formats("html,plaintext");
//!
//! let pairs = params.to_query_pairs();
//! assert_eq!(pairs.len(), 7);
//! ```

/// Fluent builder for Ghost API browse query parameters.
///
/// All setter methods consume and return `self` so calls can be chained.
/// Fields left unset are omitted from the query string entirely, letting
/// Ghost apply its own defaults.
///
/// # Example
///
/// ```
/// use ghost_io_api::params::browse::BrowseParams;
///
/// let params = BrowseParams::new()
///     .limit(5)
///     .filter("featured:true")
///     .order("published_at DESC");
///
/// let pairs = params.to_query_pairs();
/// let map: std::collections::HashMap<_, _> = pairs.into_iter().collect();
/// assert_eq!(map["limit"], "5");
/// assert_eq!(map["filter"], "featured:true");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BrowseParams {
    /// Maximum number of results to return per page.
    limit: Option<u32>,
    /// Page number to retrieve (1-indexed).
    page: Option<u32>,
    /// Ghost filter expression, e.g. `"featured:true+status:published"`.
    filter: Option<String>,
    /// Sort order, e.g. `"published_at DESC,title ASC"`.
    order: Option<String>,
    /// Comma-separated list of relations to include, e.g. `"authors,tags"`.
    include: Option<String>,
    /// Comma-separated list of fields to return, e.g. `"id,title,slug"`.
    fields: Option<String>,
    /// Comma-separated list of content formats to return, e.g. `"html,plaintext"`.
    formats: Option<String>,
}

impl BrowseParams {
    /// Creates a new `BrowseParams` with no fields set.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::params::browse::BrowseParams;
    ///
    /// let params = BrowseParams::new();
    /// assert!(params.to_query_pairs().is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of results per page.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::params::browse::BrowseParams;
    ///
    /// let params = BrowseParams::new().limit(15);
    /// let pairs = params.to_query_pairs();
    /// assert_eq!(pairs[0], ("limit", "15".to_string()));
    /// ```
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the page number to retrieve (1-indexed).
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::params::browse::BrowseParams;
    ///
    /// let params = BrowseParams::new().page(3);
    /// let pairs = params.to_query_pairs();
    /// assert_eq!(pairs[0], ("page", "3".to_string()));
    /// ```
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the Ghost filter expression.
    ///
    /// See the [Ghost filter docs](https://ghost.org/docs/content-api/#filtering)
    /// for the full syntax. Examples: `"featured:true"`, `"tag:getting-started"`,
    /// `"published_at:>2020-01-01"`.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::params::browse::BrowseParams;
    ///
    /// let params = BrowseParams::new().filter("featured:true");
    /// let pairs = params.to_query_pairs();
    /// assert_eq!(pairs[0], ("filter", "featured:true".to_string()));
    /// ```
    pub fn filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }

    /// Sets the sort order.
    ///
    /// Accepts a comma-separated list of `<field> <direction>` clauses,
    /// e.g. `"published_at DESC,title ASC"`.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::params::browse::BrowseParams;
    ///
    /// let params = BrowseParams::new().order("published_at DESC");
    /// let pairs = params.to_query_pairs();
    /// assert_eq!(pairs[0], ("order", "published_at DESC".to_string()));
    /// ```
    pub fn order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }

    /// Sets the relations to include in the response.
    ///
    /// Accepts a comma-separated list of relation names supported by the
    /// endpoint, e.g. `"authors,tags"`.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::params::browse::BrowseParams;
    ///
    /// let params = BrowseParams::new().include("authors,tags");
    /// let pairs = params.to_query_pairs();
    /// assert_eq!(pairs[0], ("include", "authors,tags".to_string()));
    /// ```
    pub fn include(mut self, include: impl Into<String>) -> Self {
        self.include = Some(include.into());
        self
    }

    /// Sets the fields to return in each result object.
    ///
    /// Accepts a comma-separated list of field names, e.g. `"id,title,slug"`.
    /// Unlisted fields are omitted from the response.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::params::browse::BrowseParams;
    ///
    /// let params = BrowseParams::new().fields("id,title,slug");
    /// let pairs = params.to_query_pairs();
    /// assert_eq!(pairs[0], ("fields", "id,title,slug".to_string()));
    /// ```
    pub fn fields(mut self, fields: impl Into<String>) -> Self {
        self.fields = Some(fields.into());
        self
    }

    /// Sets the content formats to return.
    ///
    /// Accepts a comma-separated list of format names. Valid values are
    /// `"html"`, `"mobiledoc"`, `"lexical"`, and `"plaintext"`.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::params::browse::BrowseParams;
    ///
    /// let params = BrowseParams::new().formats("html,plaintext");
    /// let pairs = params.to_query_pairs();
    /// assert_eq!(pairs[0], ("formats", "html,plaintext".to_string()));
    /// ```
    pub fn formats(mut self, formats: impl Into<String>) -> Self {
        self.formats = Some(formats.into());
        self
    }

    /// Returns the current `limit` value, if set.
    pub fn get_limit(&self) -> Option<u32> {
        self.limit
    }

    /// Returns the current `page` value, if set.
    pub fn get_page(&self) -> Option<u32> {
        self.page
    }

    /// Returns the current `filter` value, if set.
    pub fn get_filter(&self) -> Option<&str> {
        self.filter.as_deref()
    }

    /// Returns the current `order` value, if set.
    pub fn get_order(&self) -> Option<&str> {
        self.order.as_deref()
    }

    /// Returns the current `include` value, if set.
    pub fn get_include(&self) -> Option<&str> {
        self.include.as_deref()
    }

    /// Returns the current `fields` value, if set.
    pub fn get_fields(&self) -> Option<&str> {
        self.fields.as_deref()
    }

    /// Returns the current `formats` value, if set.
    pub fn get_formats(&self) -> Option<&str> {
        self.formats.as_deref()
    }

    /// Serialises the parameters as a `Vec` of `(name, value)` string pairs.
    ///
    /// Only fields that have been set are included. The order is stable:
    /// `limit`, `page`, `filter`, `order`, `include`, `fields`, `formats`.
    ///
    /// Suitable for passing directly to `reqwest`'s `.query()` method.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::params::browse::BrowseParams;
    ///
    /// let pairs = BrowseParams::new()
    ///     .limit(5)
    ///     .page(1)
    ///     .to_query_pairs();
    ///
    /// assert_eq!(pairs.len(), 2);
    /// assert_eq!(pairs[0], ("limit", "5".to_string()));
    /// assert_eq!(pairs[1], ("page", "1".to_string()));
    /// ```
    pub fn to_query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        if let Some(limit) = self.limit {
            pairs.push(("limit", limit.to_string()));
        }
        if let Some(page) = self.page {
            pairs.push(("page", page.to_string()));
        }
        if let Some(ref filter) = self.filter {
            pairs.push(("filter", filter.clone()));
        }
        if let Some(ref order) = self.order {
            pairs.push(("order", order.clone()));
        }
        if let Some(ref include) = self.include {
            pairs.push(("include", include.clone()));
        }
        if let Some(ref fields) = self.fields {
            pairs.push(("fields", fields.clone()));
        }
        if let Some(ref formats) = self.formats {
            pairs.push(("formats", formats.clone()));
        }
        pairs
    }

    /// Serialises the parameters as a URL query string (without leading `?`).
    ///
    /// Fields are percent-encoded. Returns an empty string when no fields
    /// are set.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::params::browse::BrowseParams;
    ///
    /// let qs = BrowseParams::new()
    ///     .limit(10)
    ///     .filter("featured:true")
    ///     .to_query_string();
    ///
    /// assert!(qs.contains("limit=10"));
    /// assert!(qs.contains("filter=featured%3Atrue"));
    /// ```
    pub fn to_query_string(&self) -> String {
        self.to_query_pairs()
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, percent_encode(&v)))
            .collect::<Vec<_>>()
            .join("&")
    }
}

/// Percent-encodes characters that are not unreserved URI characters.
fn percent_encode(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
                vec![c]
            } else {
                let mut buf = [0u8; 4];
                let bytes = c.encode_utf8(&mut buf);
                bytes
                    .bytes()
                    .flat_map(|b| {
                        let hi = char::from_digit((b >> 4) as u32, 16)
                            .unwrap()
                            .to_ascii_uppercase();
                        let lo = char::from_digit((b & 0xf) as u32, 16)
                            .unwrap()
                            .to_ascii_uppercase();
                        vec!['%', hi, lo]
                    })
                    .collect::<Vec<_>>()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── construction ──────────────────────────────────────────────────────────

    #[test]
    fn test_new_produces_empty_params() {
        let params = BrowseParams::new();
        assert!(params.to_query_pairs().is_empty());
    }

    #[test]
    fn test_default_produces_empty_params() {
        let params = BrowseParams::default();
        assert!(params.to_query_pairs().is_empty());
    }

    // ── individual setters ────────────────────────────────────────────────────

    #[test]
    fn test_limit() {
        let params = BrowseParams::new().limit(15);
        assert_eq!(params.get_limit(), Some(15));
        let pairs = params.to_query_pairs();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("limit", "15".to_string()));
    }

    #[test]
    fn test_page() {
        let params = BrowseParams::new().page(3);
        assert_eq!(params.get_page(), Some(3));
        let pairs = params.to_query_pairs();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("page", "3".to_string()));
    }

    #[test]
    fn test_filter() {
        let params = BrowseParams::new().filter("featured:true");
        assert_eq!(params.get_filter(), Some("featured:true"));
        let pairs = params.to_query_pairs();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("filter", "featured:true".to_string()));
    }

    #[test]
    fn test_order() {
        let params = BrowseParams::new().order("published_at DESC");
        assert_eq!(params.get_order(), Some("published_at DESC"));
        let pairs = params.to_query_pairs();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("order", "published_at DESC".to_string()));
    }

    #[test]
    fn test_include() {
        let params = BrowseParams::new().include("authors,tags");
        assert_eq!(params.get_include(), Some("authors,tags"));
        let pairs = params.to_query_pairs();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("include", "authors,tags".to_string()));
    }

    #[test]
    fn test_fields() {
        let params = BrowseParams::new().fields("id,title,slug");
        assert_eq!(params.get_fields(), Some("id,title,slug"));
        let pairs = params.to_query_pairs();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("fields", "id,title,slug".to_string()));
    }

    #[test]
    fn test_formats() {
        let params = BrowseParams::new().formats("html,plaintext");
        assert_eq!(params.get_formats(), Some("html,plaintext"));
        let pairs = params.to_query_pairs();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("formats", "html,plaintext".to_string()));
    }

    // ── chaining ──────────────────────────────────────────────────────────────

    #[test]
    fn test_all_fields_chained() {
        let params = BrowseParams::new()
            .limit(10)
            .page(2)
            .filter("featured:true")
            .order("published_at DESC")
            .include("authors,tags")
            .fields("id,title,slug")
            .formats("html,plaintext");

        let pairs = params.to_query_pairs();
        assert_eq!(pairs.len(), 7);

        let map: std::collections::HashMap<_, _> = pairs.into_iter().collect();
        assert_eq!(map["limit"], "10");
        assert_eq!(map["page"], "2");
        assert_eq!(map["filter"], "featured:true");
        assert_eq!(map["order"], "published_at DESC");
        assert_eq!(map["include"], "authors,tags");
        assert_eq!(map["fields"], "id,title,slug");
        assert_eq!(map["formats"], "html,plaintext");
    }

    #[test]
    fn test_partial_chain() {
        let params = BrowseParams::new().limit(5).filter("tag:news");
        let pairs = params.to_query_pairs();
        assert_eq!(pairs.len(), 2);
    }

    // ── stable ordering ───────────────────────────────────────────────────────

    #[test]
    fn test_query_pairs_order() {
        let params = BrowseParams::new()
            .formats("html")
            .fields("id")
            .include("tags")
            .order("title ASC")
            .filter("featured:false")
            .page(1)
            .limit(20);

        let keys: Vec<_> = params
            .to_query_pairs()
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        assert_eq!(
            keys,
            ["limit", "page", "filter", "order", "include", "fields", "formats"]
        );
    }

    // ── overwrite / last-write-wins ───────────────────────────────────────────

    #[test]
    fn test_setter_overwrites_previous_value() {
        let params = BrowseParams::new().limit(5).limit(20);
        assert_eq!(params.get_limit(), Some(20));
        let pairs = params.to_query_pairs();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].1, "20");
    }

    // ── query string serialisation ────────────────────────────────────────────

    #[test]
    fn test_to_query_string_empty() {
        assert_eq!(BrowseParams::new().to_query_string(), "");
    }

    #[test]
    fn test_to_query_string_simple() {
        let qs = BrowseParams::new().limit(10).page(1).to_query_string();
        assert_eq!(qs, "limit=10&page=1");
    }

    #[test]
    fn test_to_query_string_encodes_special_chars() {
        let qs = BrowseParams::new()
            .filter("featured:true")
            .to_query_string();
        assert!(qs.contains("filter=featured%3Atrue"));
    }

    #[test]
    fn test_to_query_string_encodes_spaces() {
        let qs = BrowseParams::new()
            .order("published_at DESC")
            .to_query_string();
        assert!(qs.contains("order=published_at%20DESC"));
    }

    #[test]
    fn test_to_query_string_encodes_plus_sign() {
        let qs = BrowseParams::new()
            .filter("featured:true+status:published")
            .to_query_string();
        assert!(qs.contains("%2B"));
    }

    // ── getters when unset ────────────────────────────────────────────────────

    #[test]
    fn test_getters_return_none_when_unset() {
        let params = BrowseParams::new();
        assert_eq!(params.get_limit(), None);
        assert_eq!(params.get_page(), None);
        assert_eq!(params.get_filter(), None);
        assert_eq!(params.get_order(), None);
        assert_eq!(params.get_include(), None);
        assert_eq!(params.get_fields(), None);
        assert_eq!(params.get_formats(), None);
    }

    // ── string-like inputs ────────────────────────────────────────────────────

    #[test]
    fn test_filter_accepts_string_and_str() {
        let owned = "featured:true".to_string();
        let p1 = BrowseParams::new().filter("featured:true");
        let p2 = BrowseParams::new().filter(owned);
        assert_eq!(p1, p2);
    }

    // ── clone & eq ────────────────────────────────────────────────────────────

    #[test]
    fn test_clone_and_eq() {
        let params = BrowseParams::new().limit(5).filter("featured:true");
        let cloned = params.clone();
        assert_eq!(params, cloned);
    }

    #[test]
    fn test_ne() {
        let a = BrowseParams::new().limit(5);
        let b = BrowseParams::new().limit(10);
        assert_ne!(a, b);
    }

    // ── edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn test_limit_zero() {
        let params = BrowseParams::new().limit(0);
        assert_eq!(params.get_limit(), Some(0));
        let pairs = params.to_query_pairs();
        assert_eq!(pairs[0], ("limit", "0".to_string()));
    }

    #[test]
    fn test_page_zero() {
        let params = BrowseParams::new().page(0);
        assert_eq!(params.get_page(), Some(0));
    }

    #[test]
    fn test_empty_string_filter() {
        let params = BrowseParams::new().filter("");
        assert_eq!(params.get_filter(), Some(""));
        assert_eq!(params.to_query_pairs().len(), 1);
    }

    #[test]
    fn test_percent_encode_unreserved_chars_unchanged() {
        let qs = BrowseParams::new()
            .filter("abc-def_ghi.jkl~mno")
            .to_query_string();
        assert_eq!(qs, "filter=abc-def_ghi.jkl~mno");
    }

    #[test]
    fn test_percent_encode_alphanumeric_unchanged() {
        let qs = BrowseParams::new().filter("abc123").to_query_string();
        assert_eq!(qs, "filter=abc123");
    }
}
