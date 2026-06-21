//! # ghost-io-api
//!
//! A strongly-typed, async Rust client for both the
//! [Ghost CMS](https://ghost.org/) **Content API** and **Admin API**.
//!
//! ## Modules
//!
//! | Module | Purpose |
//! |---|---|
//! | [`auth::content`] | [`ContentApiKey`](auth::content::ContentApiKey) — 26-char hex key for the read-only Content API |
//! | [`auth::admin`] | [`AdminApiKey`](auth::admin::AdminApiKey) — `id:hex-secret` key pair; mints HS256 JWTs for the Admin API |
//! | [`client::content`] | [`GhostContentClient`](client::content::GhostContentClient) — browse and read posts, pages, tags, authors, tiers, settings |
//! | [`client::admin`] | [`GhostAdminClient`](client::admin::GhostAdminClient) — create, update, delete, and browse posts |
//! | [`models`] | Serde-annotated structs for every Ghost resource |
//! | [`models::envelope`] | [`BrowseEnvelope<T>`](models::envelope::BrowseEnvelope) / [`SingleEnvelope<T>`](models::envelope::SingleEnvelope) — generic JSON envelope parsing |
//! | [`error`] | [`GhostError`](error::GhostError) enum and [`Result`](error::Result) alias |
//! | [`params`] | Fluent [`BrowseParams`](params::browse::BrowseParams) builder for query parameters |
//!
//! ## Content API quick-start
//!
//! ```no_run
//! use ghost_io_api::auth::content::ContentApiKey;
//! use ghost_io_api::client::content::{BrowsePostsParams, GhostContentClient};
//! use ghost_io_api::error::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let key = ContentApiKey::new("22444f78447824223cefc48062")?;
//!     let client = GhostContentClient::new("https://demo.ghost.io", key)?;
//!
//!     let response = client.browse_posts(BrowsePostsParams::default()).await?;
//!     for post in &response.posts {
//!         println!("{} — {}", post.title, post.slug);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Admin API quick-start
//!
//! ```no_run
//! use ghost_io_api::auth::admin::AdminApiKey;
//! use ghost_io_api::client::admin::{AdminBrowsePostsParams, GhostAdminClient};
//! use ghost_io_api::error::Result;
//! use ghost_io_api::models::post::{PostCreate, PostStatus, PostUpdate};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let key = AdminApiKey::new(
//!         "6748592f4b9b7700010f6564:\
//!          b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1"
//!     )?;
//!     let client = GhostAdminClient::new("https://your-site.ghost.io", key)?;
//!
//!     // Create a draft
//!     let draft = client.create_post(PostCreate::new("Hello, Admin!")).await?;
//!     println!("Draft id: {}", draft.id);
//!
//!     // Publish it — updated_at is Ghost's optimistic-concurrency token
//!     let published = client.update_post(&draft.id, PostUpdate {
//!         updated_at: draft.updated_at.unwrap(),
//!         status: Some(PostStatus::Published),
//!         ..Default::default()
//!     }).await?;
//!     println!("Published: {}", published.url.unwrap_or_default());
//!
//!     // Delete it
//!     client.delete_post(&published.id).await?;
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]

pub mod auth;
pub mod client;
pub mod error;
pub mod models;
pub mod params;
