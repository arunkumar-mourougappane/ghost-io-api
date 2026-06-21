//! Ghost Admin API client.
//!
//! Provides write-capable access to Ghost via the Admin API. Every request
//! carries a freshly-signed HS256 JWT in the `Authorization: Ghost <token>`
//! header so that tokens never expire mid-flight.
//!
//! # Example
//!
//! ```no_run
//! use ghost_io_api::auth::admin::AdminApiKey;
//! use ghost_io_api::client::admin::{GhostAdminClient, AdminBrowsePostsParams};
//! use ghost_io_api::models::post::PostCreate;
//!
//! # async fn example() -> ghost_io_api::error::Result<()> {
//! let key = AdminApiKey::new(
//!     "6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1"
//! )?;
//! let client = GhostAdminClient::new("https://example.ghost.io", key)?;
//!
//! // Browse posts
//! let response = client.browse_posts(AdminBrowsePostsParams::default()).await?;
//! println!("Found {} posts", response.posts.len());
//!
//! // Create a post
//! let post = client.create_post(PostCreate::new("Hello, Admin!")).await?;
//! println!("Created: {}", post.id);
//! # Ok(())
//! # }
//! ```

use crate::auth::admin::AdminApiKey;
use crate::error::{GhostError, Result};
use crate::models::pagination::Meta;
use crate::models::post::{Post, PostCreate, PostCreateEnvelope, PostUpdate, PostUpdateEnvelope};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

const GHOST_API_VERSION: &str = "v5.0";
const ADMIN_API_PATH: &str = "/ghost/api/admin";

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

// ── Internal response envelopes ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct AdminPostEnvelope {
    posts: Vec<Post>,
}

// ── Public types ──────────────────────────────────────────────────────────────

/// Query parameters for browsing admin posts.
///
/// All fields are optional; omitted fields cause Ghost to apply its defaults
/// (page 1, limit 15, most-recently-updated order).
///
/// # Example
///
/// ```
/// use ghost_io_api::client::admin::AdminBrowsePostsParams;
///
/// let params = AdminBrowsePostsParams {
///     limit: Some(5),
///     filter: Some("status:draft".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct AdminBrowsePostsParams {
    /// Page number (1-indexed).
    pub page: Option<u32>,
    /// Number of posts per page.
    pub limit: Option<u32>,
    /// NQL filter expression, e.g. `"status:draft"` or `"featured:true"`.
    pub filter: Option<String>,
    /// Sort order, e.g. `"published_at DESC"`.
    pub order: Option<String>,
    /// Comma-separated relations to embed, e.g. `"authors,tags"`.
    pub include: Option<String>,
    /// Comma-separated field names to return.
    pub fields: Option<String>,
}

impl AdminBrowsePostsParams {
    fn to_query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs: Vec<(&'static str, String)> = Vec::new();
        if let Some(v) = self.page {
            pairs.push(("page", v.to_string()));
        }
        if let Some(v) = self.limit {
            pairs.push(("limit", v.to_string()));
        }
        if let Some(ref v) = self.filter {
            pairs.push(("filter", v.clone()));
        }
        if let Some(ref v) = self.order {
            pairs.push(("order", v.clone()));
        }
        if let Some(ref v) = self.include {
            pairs.push(("include", v.clone()));
        }
        if let Some(ref v) = self.fields {
            pairs.push(("fields", v.clone()));
        }
        pairs
    }
}

/// Response from [`GhostAdminClient::browse_posts`].
#[derive(Debug, Deserialize)]
pub struct AdminPostsResponse {
    /// Posts returned by this page.
    pub posts: Vec<Post>,
    /// Pagination metadata.
    pub meta: Meta,
}

// ── Client ───────────────────────────────────────────────────────────────────

/// Write-capable client for the Ghost Admin API.
///
/// Communicates with `/ghost/api/admin/` using a [`AdminApiKey`] that is
/// converted to a short-lived HS256 JWT on **every request** so tokens never
/// sit long enough to expire. The `Accept-Version: v5.0` header is sent
/// automatically.
///
/// # Security
///
/// Keep the [`AdminApiKey`] secret — it grants write access to your Ghost
/// installation.
///
/// # Example
///
/// ```no_run
/// use ghost_io_api::auth::admin::AdminApiKey;
/// use ghost_io_api::client::admin::GhostAdminClient;
///
/// # async fn example() -> ghost_io_api::error::Result<()> {
/// let key = AdminApiKey::new(
///     "6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1"
/// )?;
/// let client = GhostAdminClient::new("https://example.ghost.io", key)?;
/// # Ok(())
/// # }
/// ```
pub struct GhostAdminClient {
    base_url: String,
    api_key: AdminApiKey,
    http: Client,
}

impl GhostAdminClient {
    /// Creates a new `GhostAdminClient`.
    ///
    /// Trailing slashes on `base_url` are stripped automatically.
    ///
    /// # Errors
    ///
    /// Returns [`GhostError::Http`] if the underlying `reqwest` client cannot
    /// be built.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ghost_io_api::auth::admin::AdminApiKey;
    /// use ghost_io_api::client::admin::GhostAdminClient;
    ///
    /// let key = AdminApiKey::new(
    ///     "6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1"
    /// ).unwrap();
    /// let client = GhostAdminClient::new("https://example.ghost.io", key).unwrap();
    /// ```
    pub fn new(base_url: impl Into<String>, api_key: AdminApiKey) -> Result<Self> {
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

    /// Returns the normalised base URL (no trailing slash).
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns a reference to the stored [`AdminApiKey`].
    pub fn api_key(&self) -> &AdminApiKey {
        &self.api_key
    }

    // ── Posts CRUD ────────────────────────────────────────────────────────────

    /// Browses posts with optional filtering, sorting, and pagination.
    ///
    /// Unlike the Content API, the Admin API returns all posts regardless of
    /// status (drafts, scheduled, published).
    ///
    /// # Errors
    ///
    /// Returns [`GhostError`] on network failure, a non-2xx HTTP response, or
    /// a JSON decoding error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ghost_io_api::client::admin::{GhostAdminClient, AdminBrowsePostsParams};
    /// # use ghost_io_api::auth::admin::AdminApiKey;
    /// # async fn example() -> ghost_io_api::error::Result<()> {
    /// # let key = AdminApiKey::new("6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1")?;
    /// # let client = GhostAdminClient::new("https://example.ghost.io", key)?;
    /// let response = client.browse_posts(AdminBrowsePostsParams {
    ///     limit: Some(10),
    ///     filter: Some("status:draft".to_string()),
    ///     ..Default::default()
    /// }).await?;
    /// println!("{} drafts found", response.posts.len());
    /// # Ok(()) }
    /// ```
    pub async fn browse_posts(&self, params: AdminBrowsePostsParams) -> Result<AdminPostsResponse> {
        let query = params.to_query_pairs();
        self.get("/posts/", &query).await
    }

    /// Reads a single post by its Ghost ID.
    ///
    /// `include` is an optional comma-separated list of relations to embed,
    /// e.g. `"authors,tags"`.
    ///
    /// # Errors
    ///
    /// Returns [`GhostError::Api`] with type `"NotFoundError"` when no post
    /// matches `id`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ghost_io_api::auth::admin::AdminApiKey;
    /// # use ghost_io_api::client::admin::GhostAdminClient;
    /// # async fn example() -> ghost_io_api::error::Result<()> {
    /// # let key = AdminApiKey::new("6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1")?;
    /// # let client = GhostAdminClient::new("https://example.ghost.io", key)?;
    /// let post = client.read_post_by_id("5ddc9141c35e7700383b2937", None).await?;
    /// println!("{}", post.title);
    /// # Ok(()) }
    /// ```
    pub async fn read_post_by_id(&self, id: &str, include: Option<&str>) -> Result<Post> {
        let path = format!("/posts/{id}/");
        let query = include
            .map(|i| vec![("include", i.to_string())])
            .unwrap_or_default();
        let envelope: AdminPostEnvelope = self.get(&path, &query).await?;
        envelope
            .posts
            .into_iter()
            .next()
            .ok_or_else(|| GhostError::api("Post not found", "NotFoundError", None))
    }

    /// Reads a single post by its slug.
    ///
    /// `include` is an optional comma-separated list of relations to embed,
    /// e.g. `"authors,tags"`.
    ///
    /// # Errors
    ///
    /// Returns [`GhostError::Api`] with type `"NotFoundError"` when no post
    /// matches `slug`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ghost_io_api::auth::admin::AdminApiKey;
    /// # use ghost_io_api::client::admin::GhostAdminClient;
    /// # async fn example() -> ghost_io_api::error::Result<()> {
    /// # let key = AdminApiKey::new("6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1")?;
    /// # let client = GhostAdminClient::new("https://example.ghost.io", key)?;
    /// let post = client.read_post_by_slug("welcome", Some("authors")).await?;
    /// println!("{}", post.title);
    /// # Ok(()) }
    /// ```
    pub async fn read_post_by_slug(&self, slug: &str, include: Option<&str>) -> Result<Post> {
        let path = format!("/posts/slug/{slug}/");
        let query = include
            .map(|i| vec![("include", i.to_string())])
            .unwrap_or_default();
        let envelope: AdminPostEnvelope = self.get(&path, &query).await?;
        envelope
            .posts
            .into_iter()
            .next()
            .ok_or_else(|| GhostError::api("Post not found", "NotFoundError", None))
    }

    /// Creates a new post.
    ///
    /// Returns the created post as stored by Ghost (with server-assigned `id`,
    /// `slug`, `created_at`, `updated_at`, etc.).
    ///
    /// # Errors
    ///
    /// Returns [`GhostError::Api`] on validation failures (e.g. empty title).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ghost_io_api::auth::admin::AdminApiKey;
    /// # use ghost_io_api::client::admin::GhostAdminClient;
    /// use ghost_io_api::models::post::{PostCreate, PostStatus};
    /// # async fn example() -> ghost_io_api::error::Result<()> {
    /// # let key = AdminApiKey::new("6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1")?;
    /// # let client = GhostAdminClient::new("https://example.ghost.io", key)?;
    /// let post = client.create_post(PostCreate {
    ///     title: "My New Post".to_string(),
    ///     status: Some(PostStatus::Draft),
    ///     ..Default::default()
    /// }).await?;
    /// println!("Created post: {}", post.id);
    /// # Ok(()) }
    /// ```
    pub async fn create_post(&self, post: PostCreate) -> Result<Post> {
        let envelope = PostCreateEnvelope::new(post);
        let response: AdminPostEnvelope = self.post("/posts/", &envelope).await?;
        response
            .posts
            .into_iter()
            .next()
            .ok_or_else(|| GhostError::api("No post in create response", "ServerError", None))
    }

    /// Updates an existing post.
    ///
    /// The [`PostUpdate`] must carry the post's current `updated_at` timestamp
    /// (obtained from a prior read). Ghost rejects the request if the value
    /// does not match, protecting against concurrent-edit overwrites.
    ///
    /// Returns the updated post as stored by Ghost.
    ///
    /// # Errors
    ///
    /// Returns [`GhostError::Api`] with type `"NotFoundError"` if `id` is
    /// unknown, or `"ConflictError"` if `updated_at` is stale.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ghost_io_api::auth::admin::AdminApiKey;
    /// # use ghost_io_api::client::admin::GhostAdminClient;
    /// use ghost_io_api::models::post::{PostUpdate, PostStatus};
    /// # async fn example() -> ghost_io_api::error::Result<()> {
    /// # let key = AdminApiKey::new("6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1")?;
    /// # let client = GhostAdminClient::new("https://example.ghost.io", key)?;
    /// let post = client.read_post_by_id("abc123", None).await?;
    /// let updated = client.update_post("abc123", PostUpdate {
    ///     updated_at: post.updated_at.unwrap(),
    ///     status: Some(PostStatus::Published),
    ///     ..Default::default()
    /// }).await?;
    /// println!("Now published: {}", updated.id);
    /// # Ok(()) }
    /// ```
    pub async fn update_post(&self, id: &str, post: PostUpdate) -> Result<Post> {
        let path = format!("/posts/{id}/");
        let envelope = PostUpdateEnvelope::new(post);
        let response: AdminPostEnvelope = self.put(&path, &envelope).await?;
        response
            .posts
            .into_iter()
            .next()
            .ok_or_else(|| GhostError::api("No post in update response", "ServerError", None))
    }

    /// Deletes a post by its Ghost ID.
    ///
    /// Ghost returns `204 No Content` on success. The post is permanently
    /// removed and cannot be recovered.
    ///
    /// # Errors
    ///
    /// Returns [`GhostError::Api`] with type `"NotFoundError"` if `id` is
    /// unknown.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ghost_io_api::auth::admin::AdminApiKey;
    /// # use ghost_io_api::client::admin::GhostAdminClient;
    /// # async fn example() -> ghost_io_api::error::Result<()> {
    /// # let key = AdminApiKey::new("6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1")?;
    /// # let client = GhostAdminClient::new("https://example.ghost.io", key)?;
    /// client.delete_post("abc123").await?;
    /// # Ok(()) }
    /// ```
    pub async fn delete_post(&self, id: &str) -> Result<()> {
        let path = format!("/posts/{id}/");
        self.delete(&path).await
    }

    // ── Internal plumbing ─────────────────────────────────────────────────────

    pub(crate) fn admin_url(&self, path: &str) -> String {
        format!("{}{}{}", self.base_url, ADMIN_API_PATH, path)
    }

    pub(crate) fn auth_header_value(&self) -> Result<header::HeaderValue> {
        let token = self.api_key.generate_jwt()?;
        let value = format!("Ghost {token}");
        header::HeaderValue::from_str(&value)
            .map_err(|e| GhostError::auth(format!("Invalid Authorization header value: {e}")))
    }

    pub(crate) async fn get<T>(&self, path: &str, query: &[(&str, String)]) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = self.admin_url(path);
        let auth = self.auth_header_value()?;
        let response = self
            .http
            .get(&url)
            .header(header::AUTHORIZATION, auth)
            .query(query)
            .send()
            .await?;
        self.parse_response(response).await
    }

    pub(crate) async fn post<T, B>(&self, path: &str, body: &B) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
        B: Serialize,
    {
        let url = self.admin_url(path);
        let auth = self.auth_header_value()?;
        let response = self
            .http
            .post(&url)
            .header(header::AUTHORIZATION, auth)
            .json(body)
            .send()
            .await?;
        self.parse_response(response).await
    }

    pub(crate) async fn put<T, B>(&self, path: &str, body: &B) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
        B: Serialize,
    {
        let url = self.admin_url(path);
        let auth = self.auth_header_value()?;
        let response = self
            .http
            .put(&url)
            .header(header::AUTHORIZATION, auth)
            .json(body)
            .send()
            .await?;
        self.parse_response(response).await
    }

    pub(crate) async fn delete(&self, path: &str) -> Result<()> {
        let url = self.admin_url(path);
        let auth = self.auth_header_value()?;
        let response = self
            .http
            .delete(&url)
            .header(header::AUTHORIZATION, auth)
            .send()
            .await?;
        if response.status().is_success() {
            Ok(())
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

    async fn parse_response<T>(&self, response: reqwest::Response) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
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
    use wiremock::matchers::{body_json, header_exists, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const VALID_KEY: &str =
        "6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1";

    fn make_key() -> AdminApiKey {
        AdminApiKey::new(VALID_KEY).unwrap()
    }

    fn make_client(uri: &str) -> GhostAdminClient {
        GhostAdminClient::new(uri, make_key()).unwrap()
    }

    fn post_json(id: &str, title: &str, status: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "title": title,
            "slug": title.to_lowercase().replace(' ', "-"),
            "status": status,
            "updated_at": "2026-01-01T00:00:00.000Z"
        })
    }

    fn pagination_meta(page: u32, total: u32) -> serde_json::Value {
        serde_json::json!({
            "pagination": {
                "page": page, "limit": 15,
                "pages": 1, "total": total,
                "next": null, "prev": null
            }
        })
    }

    // ── Construction (carried over) ───────────────────────────────────────────

    #[test]
    fn test_client_creation() {
        assert!(GhostAdminClient::new("https://example.ghost.io", make_key()).is_ok());
    }

    #[test]
    fn test_client_strips_trailing_slash() {
        let c = GhostAdminClient::new("https://example.ghost.io/", make_key()).unwrap();
        assert_eq!(c.base_url(), "https://example.ghost.io");
    }

    // ── AdminBrowsePostsParams ────────────────────────────────────────────────

    #[test]
    fn test_browse_params_default_empty() {
        assert!(AdminBrowsePostsParams::default()
            .to_query_pairs()
            .is_empty());
    }

    #[test]
    fn test_browse_params_all_fields() {
        let p = AdminBrowsePostsParams {
            page: Some(2),
            limit: Some(5),
            filter: Some("status:draft".to_string()),
            order: Some("published_at DESC".to_string()),
            include: Some("authors,tags".to_string()),
            fields: Some("id,title".to_string()),
        };
        let pairs = p.to_query_pairs();
        let map: std::collections::HashMap<_, _> = pairs.into_iter().collect();
        assert_eq!(map["page"], "2");
        assert_eq!(map["limit"], "5");
        assert_eq!(map["filter"], "status:draft");
        assert_eq!(map["order"], "published_at DESC");
        assert_eq!(map["include"], "authors,tags");
        assert_eq!(map["fields"], "id,title");
    }

    #[test]
    fn test_browse_params_partial() {
        let p = AdminBrowsePostsParams {
            limit: Some(10),
            ..Default::default()
        };
        let pairs = p.to_query_pairs();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("limit", "10".to_string()));
    }

    // ── browse_posts ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_browse_posts_returns_posts_and_meta() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ghost/api/admin/posts/"))
            .and(header_exists("Authorization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "posts": [post_json("id1", "First Post", "draft")],
                "meta": pagination_meta(1, 1)
            })))
            .mount(&server)
            .await;

        let response = make_client(&server.uri())
            .browse_posts(AdminBrowsePostsParams::default())
            .await
            .unwrap();

        assert_eq!(response.posts.len(), 1);
        assert_eq!(response.posts[0].id, "id1");
        assert_eq!(response.posts[0].title, "First Post");
        assert_eq!(response.meta.pagination.total, 1);
    }

    #[tokio::test]
    async fn test_browse_posts_forwards_query_params() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ghost/api/admin/posts/"))
            .and(query_param("limit", "5"))
            .and(query_param("filter", "status:draft"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "posts": [],
                "meta": pagination_meta(1, 0)
            })))
            .expect(1)
            .mount(&server)
            .await;

        make_client(&server.uri())
            .browse_posts(AdminBrowsePostsParams {
                limit: Some(5),
                filter: Some("status:draft".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();

        server.verify().await;
    }

    #[tokio::test]
    async fn test_browse_posts_empty_result() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ghost/api/admin/posts/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "posts": [],
                "meta": pagination_meta(1, 0)
            })))
            .mount(&server)
            .await;

        let response = make_client(&server.uri())
            .browse_posts(AdminBrowsePostsParams::default())
            .await
            .unwrap();

        assert!(response.posts.is_empty());
        assert_eq!(response.meta.pagination.total, 0);
    }

    // ── read_post_by_id ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_read_post_by_id_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ghost/api/admin/posts/abc123/"))
            .and(header_exists("Authorization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "posts": [post_json("abc123", "Test Post", "published")]
            })))
            .mount(&server)
            .await;

        let post = make_client(&server.uri())
            .read_post_by_id("abc123", None)
            .await
            .unwrap();

        assert_eq!(post.id, "abc123");
        assert_eq!(post.title, "Test Post");
    }

    #[tokio::test]
    async fn test_read_post_by_id_with_include() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ghost/api/admin/posts/abc123/"))
            .and(query_param("include", "authors,tags"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "posts": [post_json("abc123", "Test Post", "draft")]
            })))
            .expect(1)
            .mount(&server)
            .await;

        make_client(&server.uri())
            .read_post_by_id("abc123", Some("authors,tags"))
            .await
            .unwrap();

        server.verify().await;
    }

    #[tokio::test]
    async fn test_read_post_by_id_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ghost/api/admin/posts/nope/"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "errors": [{"message": "Post not found", "type": "NotFoundError", "context": null}]
            })))
            .mount(&server)
            .await;

        let err = make_client(&server.uri())
            .read_post_by_id("nope", None)
            .await
            .unwrap_err();

        assert!(err.is_api_error());
        assert_eq!(err.api_error_type(), Some("NotFoundError"));
    }

    // ── read_post_by_slug ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_read_post_by_slug_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ghost/api/admin/posts/slug/my-post/"))
            .and(header_exists("Authorization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "posts": [post_json("abc123", "My Post", "published")]
            })))
            .mount(&server)
            .await;

        let post = make_client(&server.uri())
            .read_post_by_slug("my-post", None)
            .await
            .unwrap();

        assert_eq!(post.id, "abc123");
        assert_eq!(post.slug, "my-post");
    }

    #[tokio::test]
    async fn test_read_post_by_slug_with_include() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ghost/api/admin/posts/slug/hello/"))
            .and(query_param("include", "authors"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "posts": [post_json("id1", "Hello", "draft")]
            })))
            .expect(1)
            .mount(&server)
            .await;

        make_client(&server.uri())
            .read_post_by_slug("hello", Some("authors"))
            .await
            .unwrap();

        server.verify().await;
    }

    #[tokio::test]
    async fn test_read_post_by_slug_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ghost/api/admin/posts/slug/nope/"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "errors": [{"message": "Post not found", "type": "NotFoundError", "context": null}]
            })))
            .mount(&server)
            .await;

        let err = make_client(&server.uri())
            .read_post_by_slug("nope", None)
            .await
            .unwrap_err();

        assert_eq!(err.api_error_type(), Some("NotFoundError"));
    }

    // ── create_post ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_post_sends_envelope() {
        let server = MockServer::start().await;

        let expected_body = serde_json::json!({
            "posts": [{"title": "New Post"}]
        });

        Mock::given(method("POST"))
            .and(path("/ghost/api/admin/posts/"))
            .and(header_exists("Authorization"))
            .and(body_json(&expected_body))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "posts": [post_json("new-id", "New Post", "draft")]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let post = make_client(&server.uri())
            .create_post(PostCreate::new("New Post"))
            .await
            .unwrap();

        assert_eq!(post.id, "new-id");
        assert_eq!(post.title, "New Post");
        server.verify().await;
    }

    #[tokio::test]
    async fn test_create_post_with_status() {
        let server = MockServer::start().await;

        let expected_body = serde_json::json!({
            "posts": [{"title": "Published", "status": "published"}]
        });

        Mock::given(method("POST"))
            .and(path("/ghost/api/admin/posts/"))
            .and(body_json(&expected_body))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "posts": [post_json("pub-id", "Published", "published")]
            })))
            .mount(&server)
            .await;

        use crate::models::post::PostStatus;
        let post = make_client(&server.uri())
            .create_post(PostCreate {
                title: "Published".to_string(),
                status: Some(PostStatus::Published),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(post.id, "pub-id");
    }

    #[tokio::test]
    async fn test_create_post_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/ghost/api/admin/posts/"))
            .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
                "errors": [{"message": "Validation failed", "type": "ValidationError", "context": "Title is required"}]
            })))
            .mount(&server)
            .await;

        let err = make_client(&server.uri())
            .create_post(PostCreate::new(""))
            .await
            .unwrap_err();

        assert!(err.is_api_error());
        assert_eq!(err.api_error_type(), Some("ValidationError"));
        assert_eq!(err.api_message(), Some("Validation failed"));
    }

    // ── update_post ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_update_post_sends_envelope_with_updated_at() {
        let server = MockServer::start().await;

        let expected_body = serde_json::json!({
            "posts": [{"updated_at": "2026-01-01T00:00:00.000Z"}]
        });

        Mock::given(method("PUT"))
            .and(path("/ghost/api/admin/posts/abc123/"))
            .and(header_exists("Authorization"))
            .and(body_json(&expected_body))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "posts": [post_json("abc123", "Unchanged", "draft")]
            })))
            .expect(1)
            .mount(&server)
            .await;

        make_client(&server.uri())
            .update_post("abc123", PostUpdate::new("2026-01-01T00:00:00.000Z"))
            .await
            .unwrap();

        server.verify().await;
    }

    #[tokio::test]
    async fn test_update_post_returns_updated_post() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/ghost/api/admin/posts/abc123/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "posts": [post_json("abc123", "Updated Title", "published")]
            })))
            .mount(&server)
            .await;

        use crate::models::post::PostStatus;
        let post = make_client(&server.uri())
            .update_post(
                "abc123",
                PostUpdate {
                    updated_at: "2026-01-01T00:00:00.000Z".to_string(),
                    title: Some("Updated Title".to_string()),
                    status: Some(PostStatus::Published),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(post.title, "Updated Title");
    }

    #[tokio::test]
    async fn test_update_post_conflict_error() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/ghost/api/admin/posts/abc123/"))
            .respond_with(ResponseTemplate::new(409).set_body_json(serde_json::json!({
                "errors": [{"message": "Conflict", "type": "ConflictError", "context": "updated_at is stale"}]
            })))
            .mount(&server)
            .await;

        let err = make_client(&server.uri())
            .update_post("abc123", PostUpdate::new("2020-01-01T00:00:00.000Z"))
            .await
            .unwrap_err();

        assert_eq!(err.api_error_type(), Some("ConflictError"));
    }

    #[tokio::test]
    async fn test_update_post_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/ghost/api/admin/posts/nope/"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "errors": [{"message": "Not found", "type": "NotFoundError", "context": null}]
            })))
            .mount(&server)
            .await;

        let err = make_client(&server.uri())
            .update_post("nope", PostUpdate::new("2026-01-01T00:00:00.000Z"))
            .await
            .unwrap_err();

        assert_eq!(err.api_error_type(), Some("NotFoundError"));
    }

    // ── delete_post ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_delete_post_success() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/ghost/api/admin/posts/abc123/"))
            .and(header_exists("Authorization"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;

        make_client(&server.uri())
            .delete_post("abc123")
            .await
            .unwrap();

        server.verify().await;
    }

    #[tokio::test]
    async fn test_delete_post_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/ghost/api/admin/posts/nope/"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "errors": [{"message": "Post not found", "type": "NotFoundError", "context": null}]
            })))
            .mount(&server)
            .await;

        let err = make_client(&server.uri())
            .delete_post("nope")
            .await
            .unwrap_err();

        assert_eq!(err.api_error_type(), Some("NotFoundError"));
    }

    // ── auth header (regression) ──────────────────────────────────────────────

    #[test]
    fn test_auth_header_starts_with_ghost() {
        let client = GhostAdminClient::new("https://example.ghost.io", make_key()).unwrap();
        let hv = client.auth_header_value().unwrap();
        assert!(hv.to_str().unwrap().starts_with("Ghost "));
    }

    #[test]
    fn test_admin_url_construction() {
        let client = GhostAdminClient::new("https://example.ghost.io", make_key()).unwrap();
        assert_eq!(
            client.admin_url("/posts/"),
            "https://example.ghost.io/ghost/api/admin/posts/"
        );
    }
}
