//! Ghost Content API client.
//!
//! Provides read-only access to published Ghost content (posts, pages, tags, authors).
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
use crate::models::pagination::Meta;
use crate::models::post::Post;
use reqwest::{Client, header};
use serde::Deserialize;

const GHOST_API_VERSION: &str = "v5.0";
const CONTENT_API_PATH: &str = "/ghost/api/content";

/// Parameters for browsing posts.
///
/// All fields are optional. When not specified, Ghost applies its defaults
/// (page 1, limit 15, published posts only).
#[derive(Debug, Clone, Default)]
pub struct BrowsePostsParams {
    /// Page number to retrieve (1-indexed). Defaults to 1.
    pub page: Option<u32>,
    /// Number of posts per page. Defaults to 15. Use `None` to let Ghost decide.
    pub limit: Option<u32>,
    /// Fields to include in the response, e.g. `"authors,tags"`.
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

/// Response envelope for browse-posts requests.
#[derive(Debug, Deserialize)]
pub struct PostsResponse {
    /// The list of posts returned by the API.
    pub posts: Vec<Post>,
    /// Pagination metadata.
    pub meta: Meta,
}

/// Response envelope for single-post requests.
#[derive(Debug, Deserialize)]
pub struct PostResponse {
    /// Single-element array containing the requested post.
    pub posts: Vec<Post>,
}

/// Ghost API error shape returned by the server.
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

/// Read-only client for the Ghost Content API.
///
/// Communicates with the Ghost Content API (`/ghost/api/content/`) using a
/// `ContentApiKey` for authentication. All methods are async and require a
/// Tokio runtime.
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
/// let result = client.browse_posts(BrowsePostsParams::default()).await?;
///
/// // Read a specific post by ID
/// let post = client.read_post_by_id("5ddc9141c35e7700383b2937", None).await?;
///
/// // Read a specific post by slug
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
    /// # Arguments
    ///
    /// * `base_url` - The Ghost site URL, e.g. `"https://demo.ghost.io"`. Trailing slashes are stripped.
    /// * `api_key` - A validated [`ContentApiKey`].
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

    /// Browses published posts with optional filtering, sorting, and pagination.
    ///
    /// # Errors
    ///
    /// Returns `GhostError` on network failure, non-2xx HTTP status, or JSON
    /// parsing errors.
    pub async fn browse_posts(&self, params: BrowsePostsParams) -> Result<PostsResponse> {
        let url = format!("{}{}/posts/", self.base_url, CONTENT_API_PATH);
        let mut query = vec![("key", self.api_key.as_str().to_string())];
        query.extend(params.to_query_pairs());

        let response = self.http.get(&url).query(&query).send().await?;
        self.parse_response::<PostsResponse>(response).await
    }

    /// Reads a single post by its Ghost ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The Ghost post ID (e.g. `"5ddc9141c35e7700383b2937"`).
    /// * `include` - Optional comma-separated list of relations to include, e.g. `Some("authors,tags")`.
    ///
    /// # Errors
    ///
    /// Returns `GhostError::Api` with `"NotFoundError"` if no post matches the ID.
    pub async fn read_post_by_id(&self, id: &str, include: Option<&str>) -> Result<Post> {
        let url = format!("{}{}/posts/{}/", self.base_url, CONTENT_API_PATH, id);
        self.read_post(&url, include).await
    }

    /// Reads a single post by its slug.
    ///
    /// # Arguments
    ///
    /// * `slug` - The post slug (e.g. `"welcome"`).
    /// * `include` - Optional comma-separated list of relations to include, e.g. `Some("authors,tags")`.
    ///
    /// # Errors
    ///
    /// Returns `GhostError::Api` with `"NotFoundError"` if no post matches the slug.
    pub async fn read_post_by_slug(&self, slug: &str, include: Option<&str>) -> Result<Post> {
        let url = format!(
            "{}{}/posts/slug/{}/",
            self.base_url, CONTENT_API_PATH, slug
        );
        self.read_post(&url, include).await
    }

    async fn read_post(&self, url: &str, include: Option<&str>) -> Result<Post> {
        let mut query = vec![("key", self.api_key.as_str().to_string())];
        if let Some(inc) = include {
            query.push(("include", inc.to_string()));
        }

        let response = self.http.get(url).query(&query).send().await?;
        let envelope = self.parse_response::<PostResponse>(response).await?;
        envelope
            .posts
            .into_iter()
            .next()
            .ok_or_else(|| GhostError::api("Post not found", "NotFoundError", None))
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
            let first = api_errors.errors.into_iter().next().unwrap_or(GhostApiError {
                message: "Unknown API error".to_string(),
                error_type: "UnknownError".to_string(),
                context: None,
            });
            Err(GhostError::api(first.message, first.error_type, first.context))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_KEY: &str = "22444f78447824223cefc48062";

    fn make_key() -> ContentApiKey {
        ContentApiKey::new(VALID_KEY).unwrap()
    }

    #[test]
    fn test_client_creation() {
        let client = GhostContentClient::new("https://demo.ghost.io", make_key());
        assert!(client.is_ok());
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

    #[test]
    fn test_browse_params_default() {
        let params = BrowsePostsParams::default();
        let pairs = params.to_query_pairs();
        assert!(pairs.is_empty());
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
            limit: None,
            include: Some("authors".to_string()),
            ..Default::default()
        };
        let pairs = params.to_query_pairs();
        assert_eq!(pairs.len(), 2);
    }
}

#[cfg(test)]
#[cfg(feature = "integration-tests")]
mod integration_tests {
    use super::*;

    const DEMO_URL: &str = "https://demo.ghost.io";
    // Public demo key from Ghost documentation
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
        assert!(!response.posts.is_empty(), "Expected at least one post");
        assert!(response.meta.pagination.page >= 1);
    }

    #[tokio::test]
    async fn test_browse_posts_with_limit() {
        let client = make_client();
        let params = BrowsePostsParams {
            limit: Some(2),
            ..Default::default()
        };
        let result = client.browse_posts(params).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.posts.len() <= 2);
    }

    #[tokio::test]
    async fn test_read_post_by_slug_integration() {
        let client = make_client();
        // Fetch first post slug from browse, then read it
        let browse = client.browse_posts(BrowsePostsParams { limit: Some(1), ..Default::default() }).await.unwrap();
        let slug = &browse.posts[0].slug;
        let post = client.read_post_by_slug(slug, None).await;
        assert!(post.is_ok(), "read_post_by_slug failed: {:?}", post);
        assert_eq!(&post.unwrap().slug, slug);
    }

    #[tokio::test]
    async fn test_read_post_by_id_integration() {
        let client = make_client();
        let browse = client.browse_posts(BrowsePostsParams { limit: Some(1), ..Default::default() }).await.unwrap();
        let id = &browse.posts[0].id;
        let post = client.read_post_by_id(id, None).await;
        assert!(post.is_ok(), "read_post_by_id failed: {:?}", post);
        assert_eq!(&post.unwrap().id, id);
    }

    #[tokio::test]
    async fn test_read_post_not_found() {
        let client = make_client();
        let result = client.read_post_by_id("000000000000000000000000", None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.is_api_error());
    }
}
