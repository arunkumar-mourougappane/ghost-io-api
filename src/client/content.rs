//! Ghost Content API client.
//!
//! Provides read-only access to published Ghost content: posts, pages, tags,
//! authors, tiers, and site settings.
//!
//! # Example
//!
//! ```no_run
//! use ghost_io_api::auth::content::ContentApiKey;
//! use ghost_io_api::client::content::{GhostContentClient, BrowsePostsParams};
//!
//! # async fn example() -> ghost_io_api::error::Result<()> {
//! let key = ContentApiKey::new("22444f78447824223cefc48062")?;
//! let client = GhostContentClient::new("https://demo.ghost.io", key)?;
//!
//! let posts = client.browse_posts(BrowsePostsParams::default()).await?;
//! println!("Found {} posts", posts.posts.len());
//! # Ok(())
//! # }
//! ```

use crate::auth::content::ContentApiKey;
use crate::error::{GhostError, Result};
use crate::models::author::Author;
use crate::models::page::Page;
use crate::models::pagination::Meta;
use crate::models::post::Post;
use crate::models::settings::Settings;
use crate::models::tag::Tag;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

const GHOST_API_VERSION: &str = "v5.0";
const CONTENT_API_PATH: &str = "/ghost/api/content";

// ── Query parameter helpers ──────────────────────────────────────────────────

/// Parameters for browsing posts.
///
/// All fields are optional. When not specified, Ghost applies its defaults
/// (page 1, limit 15, published posts only).
#[derive(Debug, Clone, Default)]
pub struct BrowsePostsParams {
    /// Page number to retrieve (1-indexed). Defaults to 1.
    pub page: Option<u32>,
    /// Number of posts per page. Defaults to 15.
    pub limit: Option<u32>,
    /// Relations to include, e.g. `"authors,tags"`.
    pub include: Option<String>,
    /// Fields to return, e.g. `"id,title,slug"`.
    pub fields: Option<String>,
    /// Filter expression, e.g. `"featured:true"`.
    pub filter: Option<String>,
    /// Sort order, e.g. `"published_at DESC"`.
    pub order: Option<String>,
}

impl BrowsePostsParams {
    fn to_query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        if let Some(page) = self.page {
            pairs.push(("page", page.to_string()));
        }
        if let Some(limit) = self.limit {
            pairs.push(("limit", limit.to_string()));
        }
        if let Some(ref include) = self.include {
            pairs.push(("include", include.clone()));
        }
        if let Some(ref fields) = self.fields {
            pairs.push(("fields", fields.clone()));
        }
        if let Some(ref filter) = self.filter {
            pairs.push(("filter", filter.clone()));
        }
        if let Some(ref order) = self.order {
            pairs.push(("order", order.clone()));
        }
        pairs
    }
}

/// Parameters for browsing a collection (pages, tags, authors, tiers).
///
/// Equivalent to [`BrowsePostsParams`] but used for non-post endpoints.
pub type BrowsePagesParams = BrowsePostsParams;
/// Parameters for browsing tags.
pub type BrowseTagsParams = BrowsePostsParams;
/// Parameters for browsing authors.
pub type BrowseAuthorsParams = BrowsePostsParams;
/// Parameters for browsing tiers.
pub type BrowseTiersParams = BrowsePostsParams;

// ── Response envelopes ───────────────────────────────────────────────────────

/// Response envelope for browse-posts requests.
#[derive(Debug, Deserialize)]
pub struct PostsResponse {
    /// Posts returned by the API.
    pub posts: Vec<Post>,
    /// Pagination metadata.
    pub meta: Meta,
}

/// Response envelope for single-post read requests.
#[derive(Debug, Deserialize)]
pub struct PostResponse {
    /// Single-element vec containing the post.
    pub posts: Vec<Post>,
}

/// Response envelope for browse-pages requests.
#[derive(Debug, Deserialize)]
pub struct PagesResponse {
    /// Pages returned by the API.
    pub pages: Vec<Page>,
    /// Pagination metadata.
    pub meta: Meta,
}

/// Response envelope for single-page read requests.
#[derive(Debug, Deserialize)]
pub struct PageResponse {
    /// Single-element vec containing the page.
    pub pages: Vec<Page>,
}

/// Response envelope for browse-tags requests.
#[derive(Debug, Deserialize)]
pub struct TagsResponse {
    /// Tags returned by the API.
    pub tags: Vec<Tag>,
    /// Pagination metadata.
    pub meta: Meta,
}

/// Response envelope for single-tag read requests.
#[derive(Debug, Deserialize)]
pub struct TagResponse {
    /// Single-element vec containing the tag.
    pub tags: Vec<Tag>,
}

/// Response envelope for browse-authors requests.
#[derive(Debug, Deserialize)]
pub struct AuthorsResponse {
    /// Authors returned by the API.
    pub authors: Vec<Author>,
    /// Pagination metadata.
    pub meta: Meta,
}

/// Response envelope for single-author read requests.
#[derive(Debug, Deserialize)]
pub struct AuthorResponse {
    /// Single-element vec containing the author.
    pub authors: Vec<Author>,
}

/// A Ghost membership tier (formerly "product").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Tier {
    /// Ghost object ID.
    pub id: String,
    /// Display name.
    pub name: String,
    /// URL slug.
    pub slug: String,
    /// Whether the tier is active.
    #[serde(default)]
    pub active: bool,
    /// Tier type: `"free"` or `"paid"`.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub tier_type: Option<String>,
    /// Welcome page URL shown after subscription.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub welcome_page_url: Option<String>,
    /// Short description of the tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// List of benefit strings shown on the pricing page.
    #[serde(default)]
    pub benefits: Vec<String>,
    /// Monthly price in the smallest currency unit (e.g. cents).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_price: Option<u64>,
    /// Yearly price in the smallest currency unit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yearly_price: Option<u64>,
    /// ISO 4217 currency code, e.g. `"usd"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// ISO 8601 creation timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// ISO 8601 last-updated timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Response envelope for browse-tiers requests.
#[derive(Debug, Deserialize)]
pub struct TiersResponse {
    /// Tiers returned by the API.
    pub tiers: Vec<Tier>,
    /// Pagination metadata.
    pub meta: Meta,
}

/// Response envelope for the settings endpoint.
#[derive(Debug, Deserialize)]
pub struct SettingsResponse {
    /// Site settings.
    pub settings: Settings,
}

// ── Internal error shape ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct GhostApiErrors {
    errors: Vec<GhostApiError>,
}

#[derive(Debug, Deserialize)]
struct GhostApiError {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
    context: Option<String>,
}

// ── Client ───────────────────────────────────────────────────────────────────

/// Read-only client for the Ghost Content API.
///
/// Communicates with `/ghost/api/content/` using a [`ContentApiKey`] passed as
/// a query parameter. All methods are `async` and require a Tokio runtime.
///
/// The client sets `Accept-Version: v5.0` on every request.
///
/// # Example
///
/// ```no_run
/// use ghost_io_api::auth::content::ContentApiKey;
/// use ghost_io_api::client::content::{GhostContentClient, BrowsePostsParams};
///
/// # async fn example() -> ghost_io_api::error::Result<()> {
/// let key = ContentApiKey::new("22444f78447824223cefc48062")?;
/// let client = GhostContentClient::new("https://demo.ghost.io", key)?;
///
/// // Browse posts
/// let posts = client.browse_posts(BrowsePostsParams::default()).await?;
///
/// // Read by ID
/// let post = client.read_post_by_id("5ddc9141c35e7700383b2937", None).await?;
///
/// // Read by slug
/// let post = client.read_post_by_slug("welcome", None).await?;
/// # Ok(())
/// # }
/// ```
pub struct GhostContentClient {
    base_url: String,
    api_key: ContentApiKey,
    http: Client,
}

impl GhostContentClient {
    /// Creates a new `GhostContentClient`.
    ///
    /// Trailing slashes on `base_url` are stripped automatically.
    ///
    /// # Errors
    ///
    /// Returns `GhostError::Http` if the underlying `reqwest` client cannot be built.
    pub fn new(base_url: impl Into<String>, api_key: ContentApiKey) -> Result<Self> {
        let mut base_url = base_url.into();
        while base_url.ends_with('/') {
            base_url.pop();
        }

        let mut default_headers = header::HeaderMap::new();
        default_headers.insert(
            "Accept-Version",
            header::HeaderValue::from_static(GHOST_API_VERSION),
        );

        let http = Client::builder()
            .default_headers(default_headers)
            .build()
            .map_err(GhostError::Http)?;

        Ok(Self {
            base_url,
            api_key,
            http,
        })
    }

    // ── Posts ────────────────────────────────────────────────────────────────

    /// Browses published posts with optional filtering, sorting, and pagination.
    ///
    /// # Errors
    ///
    /// Returns [`GhostError`] on network failure, a non-2xx HTTP
    /// response, or a JSON decoding error.
    pub async fn browse_posts(&self, params: BrowsePostsParams) -> Result<PostsResponse> {
        let url = format!("{}{}/posts/", self.base_url, CONTENT_API_PATH);
        let mut query = self.base_query();
        query.extend(params.to_query_pairs());
        let response = self.http.get(&url).query(&query).send().await?;
        self.parse_response::<PostsResponse>(response).await
    }

    /// Reads a single post by its Ghost ID.
    ///
    /// `include` is an optional comma-separated list of relations to embed, e.g. `"authors,tags"`.
    ///
    /// # Errors
    ///
    /// Returns [`GhostError::Api`] with `"NotFoundError"` if no
    /// post matches the ID, or a network / decoding error otherwise.
    pub async fn read_post_by_id(&self, id: &str, include: Option<&str>) -> Result<Post> {
        let url = format!("{}{}/posts/{}/", self.base_url, CONTENT_API_PATH, id);
        self.read_single_item::<PostResponse, Post>(&url, include, |r| r.posts)
            .await
            .and_then(|opt| {
                opt.ok_or_else(|| GhostError::api("Post not found", "NotFoundError", None))
            })
    }

    /// Reads a single post by its slug.
    ///
    /// `include` is an optional comma-separated list of relations to embed, e.g. `"authors,tags"`.
    ///
    /// # Errors
    ///
    /// Returns [`GhostError::Api`] with `"NotFoundError"` if no
    /// post matches the slug, or a network / decoding error otherwise.
    pub async fn read_post_by_slug(&self, slug: &str, include: Option<&str>) -> Result<Post> {
        let url = format!("{}{}/posts/slug/{}/", self.base_url, CONTENT_API_PATH, slug);
        self.read_single_item::<PostResponse, Post>(&url, include, |r| r.posts)
            .await
            .and_then(|opt| {
                opt.ok_or_else(|| GhostError::api("Post not found", "NotFoundError", None))
            })
    }

    // ── Pages ────────────────────────────────────────────────────────────────

    /// Browses published pages with optional filtering, sorting, and pagination.
    ///
    /// # Errors
    ///
    /// Returns [`GhostError`] on network failure, a non-2xx HTTP
    /// response, or a JSON decoding error.
    pub async fn browse_pages(&self, params: BrowsePagesParams) -> Result<PagesResponse> {
        let url = format!("{}{}/pages/", self.base_url, CONTENT_API_PATH);
        let mut query = self.base_query();
        query.extend(params.to_query_pairs());
        let response = self.http.get(&url).query(&query).send().await?;
        self.parse_response::<PagesResponse>(response).await
    }

    /// Reads a single page by its Ghost ID.
    ///
    /// `include` is an optional comma-separated list of relations to embed, e.g. `"authors"`.
    ///
    /// # Errors
    ///
    /// Returns [`GhostError::Api`] with `"NotFoundError"` if no
    /// page matches the ID, or a network / decoding error otherwise.
    pub async fn read_page_by_id(&self, id: &str, include: Option<&str>) -> Result<Page> {
        let url = format!("{}{}/pages/{}/", self.base_url, CONTENT_API_PATH, id);
        self.read_single_item::<PageResponse, Page>(&url, include, |r| r.pages)
            .await
            .and_then(|opt| {
                opt.ok_or_else(|| GhostError::api("Page not found", "NotFoundError", None))
            })
    }

    /// Reads a single page by its slug.
    ///
    /// `include` is an optional comma-separated list of relations to embed, e.g. `"authors"`.
    ///
    /// # Errors
    ///
    /// Returns [`GhostError::Api`] with `"NotFoundError"` if no
    /// page matches the slug, or a network / decoding error otherwise.
    pub async fn read_page_by_slug(&self, slug: &str, include: Option<&str>) -> Result<Page> {
        let url = format!("{}{}/pages/slug/{}/", self.base_url, CONTENT_API_PATH, slug);
        self.read_single_item::<PageResponse, Page>(&url, include, |r| r.pages)
            .await
            .and_then(|opt| {
                opt.ok_or_else(|| GhostError::api("Page not found", "NotFoundError", None))
            })
    }

    // ── Tags ─────────────────────────────────────────────────────────────────

    /// Browses tags with optional filtering, sorting, and pagination.
    ///
    /// # Errors
    ///
    /// Returns [`GhostError`] on network failure, a non-2xx HTTP
    /// response, or a JSON decoding error.
    pub async fn browse_tags(&self, params: BrowseTagsParams) -> Result<TagsResponse> {
        let url = format!("{}{}/tags/", self.base_url, CONTENT_API_PATH);
        let mut query = self.base_query();
        query.extend(params.to_query_pairs());
        let response = self.http.get(&url).query(&query).send().await?;
        self.parse_response::<TagsResponse>(response).await
    }

    /// Reads a single tag by its Ghost ID.
    ///
    /// `include` accepts `"count.posts"` to embed the post count for this tag.
    ///
    /// # Errors
    ///
    /// Returns [`GhostError::Api`] with `"NotFoundError"` if no
    /// tag matches the ID, or a network / decoding error otherwise.
    pub async fn read_tag_by_id(&self, id: &str, include: Option<&str>) -> Result<Tag> {
        let url = format!("{}{}/tags/{}/", self.base_url, CONTENT_API_PATH, id);
        self.read_single_item::<TagResponse, Tag>(&url, include, |r| r.tags)
            .await
            .and_then(|opt| {
                opt.ok_or_else(|| GhostError::api("Tag not found", "NotFoundError", None))
            })
    }

    /// Reads a single tag by its slug.
    ///
    /// `include` accepts `"count.posts"` to embed the post count for this tag.
    ///
    /// # Errors
    ///
    /// Returns [`GhostError::Api`] with `"NotFoundError"` if no
    /// tag matches the slug, or a network / decoding error otherwise.
    pub async fn read_tag_by_slug(&self, slug: &str, include: Option<&str>) -> Result<Tag> {
        let url = format!("{}{}/tags/slug/{}/", self.base_url, CONTENT_API_PATH, slug);
        self.read_single_item::<TagResponse, Tag>(&url, include, |r| r.tags)
            .await
            .and_then(|opt| {
                opt.ok_or_else(|| GhostError::api("Tag not found", "NotFoundError", None))
            })
    }

    // ── Authors ──────────────────────────────────────────────────────────────

    /// Browses authors with optional filtering, sorting, and pagination.
    ///
    /// # Errors
    ///
    /// Returns [`GhostError`] on network failure, a non-2xx HTTP
    /// response, or a JSON decoding error.
    pub async fn browse_authors(&self, params: BrowseAuthorsParams) -> Result<AuthorsResponse> {
        let url = format!("{}{}/authors/", self.base_url, CONTENT_API_PATH);
        let mut query = self.base_query();
        query.extend(params.to_query_pairs());
        let response = self.http.get(&url).query(&query).send().await?;
        self.parse_response::<AuthorsResponse>(response).await
    }

    /// Reads a single author by their Ghost ID.
    ///
    /// `include` accepts `"count.posts"` to embed the post count for this author.
    ///
    /// # Errors
    ///
    /// Returns [`GhostError::Api`] with `"NotFoundError"` if no
    /// author matches the ID, or a network / decoding error otherwise.
    pub async fn read_author_by_id(&self, id: &str, include: Option<&str>) -> Result<Author> {
        let url = format!("{}{}/authors/{}/", self.base_url, CONTENT_API_PATH, id);
        self.read_single_item::<AuthorResponse, Author>(&url, include, |r| r.authors)
            .await
            .and_then(|opt| {
                opt.ok_or_else(|| GhostError::api("Author not found", "NotFoundError", None))
            })
    }

    /// Reads a single author by their slug.
    ///
    /// `include` accepts `"count.posts"` to embed the post count for this author.
    ///
    /// # Errors
    ///
    /// Returns [`GhostError::Api`] with `"NotFoundError"` if no
    /// author matches the slug, or a network / decoding error otherwise.
    pub async fn read_author_by_slug(&self, slug: &str, include: Option<&str>) -> Result<Author> {
        let url = format!(
            "{}{}/authors/slug/{}/",
            self.base_url, CONTENT_API_PATH, slug
        );
        self.read_single_item::<AuthorResponse, Author>(&url, include, |r| r.authors)
            .await
            .and_then(|opt| {
                opt.ok_or_else(|| GhostError::api("Author not found", "NotFoundError", None))
            })
    }

    // ── Tiers ────────────────────────────────────────────────────────────────

    /// Browses membership tiers (free and paid).
    ///
    /// # Errors
    ///
    /// Returns [`GhostError`] on network failure, a non-2xx HTTP
    /// response, or a JSON decoding error.
    pub async fn browse_tiers(&self, params: BrowseTiersParams) -> Result<TiersResponse> {
        let url = format!("{}{}/tiers/", self.base_url, CONTENT_API_PATH);
        let mut query = self.base_query();
        query.extend(params.to_query_pairs());
        let response = self.http.get(&url).query(&query).send().await?;
        self.parse_response::<TiersResponse>(response).await
    }

    // ── Settings ─────────────────────────────────────────────────────────────

    /// Fetches site-wide settings (title, logo, navigation, social links, etc.).
    ///
    /// # Errors
    ///
    /// Returns [`GhostError`] on network failure, a non-2xx HTTP
    /// response, or a JSON decoding error.
    pub async fn get_settings(&self) -> Result<Settings> {
        let url = format!("{}{}/settings/", self.base_url, CONTENT_API_PATH);
        let query = self.base_query();
        let response = self.http.get(&url).query(&query).send().await?;
        self.parse_response::<SettingsResponse>(response)
            .await
            .map(|r| r.settings)
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn base_query(&self) -> Vec<(&'static str, String)> {
        vec![("key", self.api_key.as_str().to_string())]
    }

    async fn read_single_item<E, T>(
        &self,
        url: &str,
        include: Option<&str>,
        extract: impl FnOnce(E) -> Vec<T>,
    ) -> Result<Option<T>>
    where
        E: for<'de> Deserialize<'de>,
    {
        let mut query = self.base_query();
        if let Some(inc) = include {
            query.push(("include", inc.to_string()));
        }
        let response = self.http.get(url).query(&query).send().await?;
        let envelope = self.parse_response::<E>(response).await?;
        Ok(extract(envelope).into_iter().next())
    }

    async fn parse_response<T: for<'de> Deserialize<'de>>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        let status = response.status();
        if status.is_success() {
            Ok(response.json::<T>().await?)
        } else {
            let api_errors: GhostApiErrors = response.json().await?;
            let first = api_errors
                .errors
                .into_iter()
                .next()
                .unwrap_or(GhostApiError {
                    message: "Unknown API error".to_string(),
                    error_type: "UnknownError".to_string(),
                    context: None,
                });
            Err(GhostError::api(
                first.message,
                first.error_type,
                first.context,
            ))
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_KEY: &str = "22444f78447824223cefc48062";

    fn make_key() -> ContentApiKey {
        ContentApiKey::new(VALID_KEY).unwrap()
    }

    fn make_client() -> GhostContentClient {
        GhostContentClient::new("https://demo.ghost.io", make_key()).unwrap()
    }

    // ── Construction ──────────────────────────────────────────────────────────

    #[test]
    fn test_client_creation() {
        assert!(GhostContentClient::new("https://demo.ghost.io", make_key()).is_ok());
    }

    #[test]
    fn test_client_strips_trailing_slash() {
        let client = GhostContentClient::new("https://demo.ghost.io/", make_key()).unwrap();
        assert_eq!(client.base_url, "https://demo.ghost.io");
    }

    #[test]
    fn test_client_strips_multiple_trailing_slashes() {
        let client = GhostContentClient::new("https://demo.ghost.io///", make_key()).unwrap();
        assert_eq!(client.base_url, "https://demo.ghost.io");
    }

    // ── base_query ────────────────────────────────────────────────────────────

    #[test]
    fn test_base_query_contains_key() {
        let client = make_client();
        let query = client.base_query();
        assert_eq!(query.len(), 1);
        assert_eq!(query[0], ("key", VALID_KEY.to_string()));
    }

    // ── BrowsePostsParams ─────────────────────────────────────────────────────

    #[test]
    fn test_browse_params_default_empty() {
        let params = BrowsePostsParams::default();
        assert!(params.to_query_pairs().is_empty());
    }

    #[test]
    fn test_browse_params_all_fields() {
        let params = BrowsePostsParams {
            page: Some(2),
            limit: Some(10),
            include: Some("authors,tags".to_string()),
            fields: Some("id,title".to_string()),
            filter: Some("featured:true".to_string()),
            order: Some("published_at DESC".to_string()),
        };
        let pairs = params.to_query_pairs();
        assert_eq!(pairs.len(), 6);

        let map: std::collections::HashMap<_, _> = pairs.into_iter().collect();
        assert_eq!(map["page"], "2");
        assert_eq!(map["limit"], "10");
        assert_eq!(map["include"], "authors,tags");
        assert_eq!(map["fields"], "id,title");
        assert_eq!(map["filter"], "featured:true");
        assert_eq!(map["order"], "published_at DESC");
    }

    #[test]
    fn test_browse_params_partial() {
        let params = BrowsePostsParams {
            page: Some(1),
            include: Some("authors".to_string()),
            ..Default::default()
        };
        assert_eq!(params.to_query_pairs().len(), 2);
    }

    // ── Tier model ────────────────────────────────────────────────────────────

    #[test]
    fn test_tier_deserialization() {
        let json = serde_json::json!({
            "id": "tier1",
            "name": "Free",
            "slug": "free",
            "active": true,
            "type": "free"
        });
        let tier: Tier = serde_json::from_value(json).unwrap();
        assert_eq!(tier.id, "tier1");
        assert!(tier.active);
        assert_eq!(tier.tier_type.as_deref(), Some("free"));
    }

    #[test]
    fn test_tier_default() {
        let tier = Tier::default();
        assert!(tier.id.is_empty());
        assert!(!tier.active);
        assert!(tier.benefits.is_empty());
    }

    #[test]
    fn test_tier_with_pricing() {
        let json = serde_json::json!({
            "id": "tier2",
            "name": "Supporter",
            "slug": "supporter",
            "active": true,
            "type": "paid",
            "monthly_price": 500,
            "yearly_price": 5000,
            "currency": "usd",
            "benefits": ["No ads", "Newsletter"]
        });
        let tier: Tier = serde_json::from_value(json).unwrap();
        assert_eq!(tier.monthly_price, Some(500));
        assert_eq!(tier.yearly_price, Some(5000));
        assert_eq!(tier.currency.as_deref(), Some("usd"));
        assert_eq!(tier.benefits.len(), 2);
    }

    #[test]
    fn test_tier_clone_eq() {
        let tier = Tier {
            id: "1".to_string(),
            name: "Free".to_string(),
            slug: "free".to_string(),
            ..Default::default()
        };
        assert_eq!(tier, tier.clone());
    }

    // ── Response envelope deserialization ─────────────────────────────────────

    #[test]
    fn test_settings_response_deserialization() {
        let json = serde_json::json!({
            "settings": {
                "title": "Test Blog",
                "lang": "en"
            }
        });
        let resp: SettingsResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.settings.title.as_deref(), Some("Test Blog"));
    }

    #[test]
    fn test_pages_response_deserialization() {
        let json = serde_json::json!({
            "pages": [
                { "id": "1", "title": "About", "slug": "about" }
            ],
            "meta": { "pagination": { "page": 1, "limit": 15, "pages": 1, "total": 1 } }
        });
        let resp: PagesResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.pages.len(), 1);
        assert_eq!(resp.pages[0].slug, "about");
    }

    #[test]
    fn test_tags_response_deserialization() {
        let json = serde_json::json!({
            "tags": [
                { "id": "t1", "name": "Rust", "slug": "rust" }
            ],
            "meta": { "pagination": { "page": 1, "limit": 15, "pages": 1, "total": 1 } }
        });
        let resp: TagsResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.tags.len(), 1);
        assert_eq!(resp.tags[0].name, "Rust");
    }

    #[test]
    fn test_authors_response_deserialization() {
        let json = serde_json::json!({
            "authors": [
                { "id": "a1", "name": "Jane", "slug": "jane" }
            ],
            "meta": { "pagination": { "page": 1, "limit": 15, "pages": 1, "total": 1 } }
        });
        let resp: AuthorsResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.authors.len(), 1);
        assert_eq!(resp.authors[0].name, "Jane");
    }

    #[test]
    fn test_tiers_response_deserialization() {
        let json = serde_json::json!({
            "tiers": [
                { "id": "t1", "name": "Free", "slug": "free", "active": true }
            ],
            "meta": { "pagination": { "page": 1, "limit": 15, "pages": 1, "total": 1 } }
        });
        let resp: TiersResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.tiers.len(), 1);
        assert!(resp.tiers[0].active);
    }
}

// ── Integration tests (requires --features integration-tests) ─────────────────

#[cfg(test)]
#[cfg(feature = "integration-tests")]
mod integration_tests {
    use super::*;

    const DEMO_URL: &str = "https://demo.ghost.io";
    const DEMO_KEY: &str = "22444f78447824223cefc48062";

    fn make_client() -> GhostContentClient {
        let key = ContentApiKey::new(DEMO_KEY).unwrap();
        GhostContentClient::new(DEMO_URL, key).unwrap()
    }

    #[tokio::test]
    async fn test_browse_posts_integration() {
        let client = make_client();
        let result = client.browse_posts(BrowsePostsParams::default()).await;
        assert!(result.is_ok(), "browse_posts failed: {:?}", result);
        let response = result.unwrap();
        assert!(!response.posts.is_empty());
        assert!(response.meta.pagination.page >= 1);
    }

    #[tokio::test]
    async fn test_browse_posts_with_limit() {
        let client = make_client();
        let params = BrowsePostsParams {
            limit: Some(2),
            ..Default::default()
        };
        let result = client.browse_posts(params).await.unwrap();
        assert!(result.posts.len() <= 2);
    }

    #[tokio::test]
    async fn test_read_post_by_slug_integration() {
        let client = make_client();
        let browse = client
            .browse_posts(BrowsePostsParams {
                limit: Some(1),
                ..Default::default()
            })
            .await
            .unwrap();
        let slug = &browse.posts[0].slug;
        let post = client.read_post_by_slug(slug, None).await.unwrap();
        assert_eq!(&post.slug, slug);
    }

    #[tokio::test]
    async fn test_read_post_by_id_integration() {
        let client = make_client();
        let browse = client
            .browse_posts(BrowsePostsParams {
                limit: Some(1),
                ..Default::default()
            })
            .await
            .unwrap();
        let id = &browse.posts[0].id;
        let post = client.read_post_by_id(id, None).await.unwrap();
        assert_eq!(&post.id, id);
    }

    #[tokio::test]
    async fn test_read_post_not_found() {
        let client = make_client();
        let err = client
            .read_post_by_id("000000000000000000000000", None)
            .await
            .unwrap_err();
        assert!(err.is_api_error());
    }

    #[tokio::test]
    async fn test_browse_pages_integration() {
        let client = make_client();
        let result = client.browse_pages(BrowsePagesParams::default()).await;
        assert!(result.is_ok(), "browse_pages failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_browse_tags_integration() {
        let client = make_client();
        let result = client.browse_tags(BrowseTagsParams::default()).await;
        assert!(result.is_ok(), "browse_tags failed: {:?}", result);
        assert!(!result.unwrap().tags.is_empty());
    }

    #[tokio::test]
    async fn test_read_tag_by_slug_integration() {
        let client = make_client();
        let tags = client
            .browse_tags(BrowseTagsParams {
                limit: Some(1),
                ..Default::default()
            })
            .await
            .unwrap();
        let slug = &tags.tags[0].slug;
        let tag = client.read_tag_by_slug(slug, None).await.unwrap();
        assert_eq!(&tag.slug, slug);
    }

    #[tokio::test]
    async fn test_browse_authors_integration() {
        let client = make_client();
        let result = client.browse_authors(BrowseAuthorsParams::default()).await;
        assert!(result.is_ok(), "browse_authors failed: {:?}", result);
        assert!(!result.unwrap().authors.is_empty());
    }

    #[tokio::test]
    async fn test_get_settings_integration() {
        let client = make_client();
        let result = client.get_settings().await;
        assert!(result.is_ok(), "get_settings failed: {:?}", result);
        let settings = result.unwrap();
        assert!(settings.title.is_some());
    }
}
