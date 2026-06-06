//! # Ghost API Client for Rust
//!
//! A strongly-typed, async Rust client for the Ghost Content and Admin APIs.
//!
//! This crate provides two main clients:
//! - `GhostContentClient` - For read-only access to published content (posts, pages, tags, etc.)
//! - `GhostAdminClient` - For full CRUD operations on Ghost resources
//!
//! ## Example
//!
//! ```ignore
//! use ghost_io_api::error::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Your code here
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod client;
pub mod error;
pub mod models;
