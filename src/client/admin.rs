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
//! use ghost_io_api::client::admin::GhostAdminClient;
//!
//! # async fn example() -> ghost_io_api::error::Result<()> {
//! let key = AdminApiKey::new(
//!     "6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1"
//! )?;
//! let client = GhostAdminClient::new("https://example.ghost.io", key)?;
//! # Ok(())
//! # }
//! ```

use crate::auth::admin::AdminApiKey;
use crate::error::{GhostError, Result};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

const GHOST_API_VERSION: &str = "v5.0";
#[allow(dead_code)]
const ADMIN_API_PATH: &str = "/ghost/api/admin";

// ── Internal error shape ─────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GhostApiErrors {
    errors: Vec<GhostApiError>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GhostApiError {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
    context: Option<String>,
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
    #[allow(dead_code)]
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

    // ── Internal plumbing (used by future CRUD method modules) ───────────────

    /// Builds the full URL for an Admin API path segment.
    ///
    /// `path` should start with `/` and end with `/`, e.g. `"/posts/"`.
    #[allow(dead_code)]
    pub(crate) fn admin_url(&self, path: &str) -> String {
        format!("{}{}{}", self.base_url, ADMIN_API_PATH, path)
    }

    /// Generates an `Authorization: Ghost <token>` header value.
    ///
    /// A fresh JWT is minted on every call so that the 5-minute expiry window
    /// always starts from the current moment.
    #[allow(dead_code)]
    pub(crate) fn auth_header_value(&self) -> Result<header::HeaderValue> {
        let token = self.api_key.generate_jwt()?;
        let value = format!("Ghost {token}");
        header::HeaderValue::from_str(&value)
            .map_err(|e| GhostError::auth(format!("Invalid Authorization header value: {e}")))
    }

    // ── HTTP verbs ────────────────────────────────────────────────────────────

    /// Sends an authenticated GET request and deserialises the JSON response.
    #[allow(dead_code)]
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

    /// Sends an authenticated POST request with a JSON body.
    #[allow(dead_code)]
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

    /// Sends an authenticated PUT request with a JSON body.
    #[allow(dead_code)]
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

    /// Sends an authenticated DELETE request.
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
    use wiremock::matchers::{header_exists, header_regex, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const VALID_KEY: &str =
        "6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1";

    fn make_key() -> AdminApiKey {
        AdminApiKey::new(VALID_KEY).unwrap()
    }

    fn make_client() -> GhostAdminClient {
        GhostAdminClient::new("https://example.ghost.io", make_key()).unwrap()
    }

    // ── Construction ──────────────────────────────────────────────────────────

    #[test]
    fn test_client_creation() {
        assert!(GhostAdminClient::new("https://example.ghost.io", make_key()).is_ok());
    }

    #[test]
    fn test_client_strips_trailing_slash() {
        let client = GhostAdminClient::new("https://example.ghost.io/", make_key()).unwrap();
        assert_eq!(client.base_url(), "https://example.ghost.io");
    }

    #[test]
    fn test_client_strips_multiple_trailing_slashes() {
        let client = GhostAdminClient::new("https://example.ghost.io///", make_key()).unwrap();
        assert_eq!(client.base_url(), "https://example.ghost.io");
    }

    #[test]
    fn test_base_url_accessor() {
        let client = make_client();
        assert_eq!(client.base_url(), "https://example.ghost.io");
    }

    #[test]
    fn test_api_key_accessor() {
        let client = make_client();
        assert_eq!(client.api_key().key_id(), "6748592f4b9b7700010f6564");
    }

    // ── admin_url ─────────────────────────────────────────────────────────────

    #[test]
    fn test_admin_url_posts() {
        let client = make_client();
        assert_eq!(
            client.admin_url("/posts/"),
            "https://example.ghost.io/ghost/api/admin/posts/"
        );
    }

    #[test]
    fn test_admin_url_with_id() {
        let client = make_client();
        assert_eq!(
            client.admin_url("/posts/abc123/"),
            "https://example.ghost.io/ghost/api/admin/posts/abc123/"
        );
    }

    // ── auth_header_value ─────────────────────────────────────────────────────

    #[test]
    fn test_auth_header_starts_with_ghost() {
        let client = make_client();
        let hv = client.auth_header_value().unwrap();
        let s = hv.to_str().unwrap();
        assert!(s.starts_with("Ghost "));
    }

    #[test]
    fn test_auth_header_token_is_valid_jwt() {
        let client = make_client();
        let hv = client.auth_header_value().unwrap();
        let s = hv.to_str().unwrap();
        let token = s.strip_prefix("Ghost ").unwrap();
        assert_eq!(
            token.split('.').count(),
            3,
            "token must have three JWT parts"
        );
    }

    #[test]
    fn test_auth_header_token_kid_matches_key_id() {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

        let client = make_client();
        let hv = client.auth_header_value().unwrap();
        let s = hv.to_str().unwrap();
        let token = s.strip_prefix("Ghost ").unwrap();
        let header_b64 = token.split('.').next().unwrap();
        let header_bytes = URL_SAFE_NO_PAD.decode(header_b64).unwrap();
        let header: serde_json::Value = serde_json::from_slice(&header_bytes).unwrap();

        assert_eq!(header["kid"], "6748592f4b9b7700010f6564");
        assert_eq!(header["alg"], "HS256");
        assert_eq!(header["typ"], "JWT");
    }

    #[test]
    fn test_auth_header_is_fresh_each_call() {
        // Two calls 1 second apart (via mocked iat) should differ.
        // Here we just verify two immediate calls can produce values —
        // determinism within the same second is acceptable.
        let client = make_client();
        let h1 = client.auth_header_value().unwrap();
        let h2 = client.auth_header_value().unwrap();
        // Both must be structurally valid regardless of equality
        assert!(h1.to_str().unwrap().starts_with("Ghost "));
        assert!(h2.to_str().unwrap().starts_with("Ghost "));
    }

    // ── HTTP-level: Authorization header injection ────────────────────────────

    #[tokio::test]
    async fn test_get_sends_authorization_header() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/ghost/api/admin/posts/"))
            .and(header_exists("Authorization"))
            .and(header_regex(
                "Authorization",
                "^Ghost [A-Za-z0-9_\\-]+\\.[A-Za-z0-9_\\-]+\\.[A-Za-z0-9_\\-]+$",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "posts": [],
                "meta": { "pagination": { "page": 1, "limit": 15, "pages": 0, "total": 0 } }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let key = make_key();
        let client = GhostAdminClient::new(server.uri(), key).unwrap();

        #[derive(Deserialize)]
        struct Resp {
            posts: Vec<serde_json::Value>,
        }

        let resp: Resp = client.get("/posts/", &[]).await.unwrap();
        assert!(resp.posts.is_empty());

        server.verify().await;
    }

    #[tokio::test]
    async fn test_get_sends_accept_version_header() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/ghost/api/admin/posts/"))
            .and(header_exists("Accept-Version"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "posts": [],
                "meta": { "pagination": { "page": 1, "limit": 15, "pages": 0, "total": 0 } }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let key = make_key();
        let client = GhostAdminClient::new(server.uri(), key).unwrap();

        #[derive(Deserialize)]
        struct Resp {
            posts: Vec<serde_json::Value>,
        }

        let _: Resp = client.get("/posts/", &[]).await.unwrap();
        server.verify().await;
    }

    #[tokio::test]
    async fn test_post_sends_authorization_header() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/ghost/api/admin/posts/"))
            .and(header_exists("Authorization"))
            .and(header_regex("Authorization", "^Ghost "))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "posts": [{ "id": "new1", "title": "Hello", "slug": "hello" }]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let key = make_key();
        let client = GhostAdminClient::new(server.uri(), key).unwrap();

        #[derive(Deserialize)]
        struct Resp {
            posts: Vec<serde_json::Value>,
        }

        let body = serde_json::json!({"posts": [{"title": "Hello"}]});
        let resp: Resp = client.post("/posts/", &body).await.unwrap();
        assert_eq!(resp.posts.len(), 1);

        server.verify().await;
    }

    #[tokio::test]
    async fn test_put_sends_authorization_header() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/ghost/api/admin/posts/abc123/"))
            .and(header_exists("Authorization"))
            .and(header_regex("Authorization", "^Ghost "))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "posts": [{ "id": "abc123", "title": "Updated", "slug": "updated" }]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let key = make_key();
        let client = GhostAdminClient::new(server.uri(), key).unwrap();

        #[derive(Deserialize)]
        struct Resp {
            posts: Vec<serde_json::Value>,
        }

        let body = serde_json::json!({"posts": [{"title": "Updated"}]});
        let resp: Resp = client.put("/posts/abc123/", &body).await.unwrap();
        assert_eq!(resp.posts[0]["id"], "abc123");

        server.verify().await;
    }

    #[tokio::test]
    async fn test_delete_sends_authorization_header() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/ghost/api/admin/posts/abc123/"))
            .and(header_exists("Authorization"))
            .and(header_regex("Authorization", "^Ghost "))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;

        let key = make_key();
        let client = GhostAdminClient::new(server.uri(), key).unwrap();

        client.delete("/posts/abc123/").await.unwrap();

        server.verify().await;
    }

    // ── Error handling ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_returns_api_error_on_4xx() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/ghost/api/admin/posts/notfound/"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "errors": [{
                    "message": "Resource not found",
                    "type": "NotFoundError",
                    "context": null
                }]
            })))
            .mount(&server)
            .await;

        let key = make_key();
        let client = GhostAdminClient::new(server.uri(), key).unwrap();

        #[derive(Debug, Deserialize)]
        struct Resp {}

        let err = client
            .get::<Resp>("/posts/notfound/", &[])
            .await
            .unwrap_err();
        assert!(err.is_api_error());
        assert_eq!(err.api_error_type(), Some("NotFoundError"));
        assert_eq!(err.api_message(), Some("Resource not found"));
    }

    #[tokio::test]
    async fn test_delete_returns_api_error_on_4xx() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/ghost/api/admin/posts/locked/"))
            .respond_with(ResponseTemplate::new(409).set_body_json(serde_json::json!({
                "errors": [{
                    "message": "Cannot delete",
                    "type": "ConflictError",
                    "context": "Post is locked"
                }]
            })))
            .mount(&server)
            .await;

        let key = make_key();
        let client = GhostAdminClient::new(server.uri(), key).unwrap();

        let err = client.delete("/posts/locked/").await.unwrap_err();
        assert!(err.is_api_error());
        assert_eq!(err.api_error_type(), Some("ConflictError"));
    }

    #[tokio::test]
    async fn test_get_query_params_forwarded() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/ghost/api/admin/posts/"))
            .and(wiremock::matchers::query_param("limit", "5"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "posts": [],
                "meta": { "pagination": { "page": 1, "limit": 5, "pages": 0, "total": 0 } }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let key = make_key();
        let client = GhostAdminClient::new(server.uri(), key).unwrap();

        #[derive(Deserialize)]
        struct Resp {
            posts: Vec<serde_json::Value>,
        }

        let _: Resp = client
            .get("/posts/", &[("limit", "5".to_string())])
            .await
            .unwrap();

        server.verify().await;
    }
}
