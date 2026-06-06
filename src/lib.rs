//! # ghost-io-api
//!
//! A strongly-typed, async Rust client for the [Ghost CMS](https://ghost.org/) Content API.
//!
//! ## Modules
//!
//! - [`auth`] — API key types and validation
//! - [`client`] — HTTP clients for each Ghost API
//! - [`error`] — [`GhostError`](error::GhostError) enum and [`Result`](error::Result) alias
//! - [`models`] — serde-annotated structs for every Ghost resource
//! - [`params`] — fluent query-parameter builders
//!
//! ## Quick start
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

#![warn(missing_docs)]

pub mod auth;
pub mod client;
pub mod error;
pub mod models;
pub mod params;
