//! Example: list posts from a Ghost site using the Content API.
//!
//! Run with:
//!
//! ```sh
//! GHOST_URL=https://demo.ghost.io \
//! GHOST_CONTENT_KEY=22444f78447824223cefc48062 \
//! cargo run --example list_posts
//! ```
//!
//! The example pages through all published posts (up to `MAX_PAGES`) and
//! prints each post's title, slug, and publication date.

use ghost_io_api::auth::content::ContentApiKey;
use ghost_io_api::client::content::{BrowsePostsParams, GhostContentClient};
use ghost_io_api::error::Result;
use ghost_io_api::params::browse::BrowseParams;

const MAX_PAGES: u32 = 5;
const PAGE_SIZE: u32 = 10;

#[tokio::main]
async fn main() -> Result<()> {
    let url = std::env::var("GHOST_URL").unwrap_or_else(|_| "https://demo.ghost.io".to_string());
    let raw_key = std::env::var("GHOST_CONTENT_KEY")
        .unwrap_or_else(|_| "22444f78447824223cefc48062".to_string());

    let key = ContentApiKey::new(raw_key)?;
    let client = GhostContentClient::new(&url, key)?;

    println!("Fetching posts from {url}");
    println!("{:-<60}", "");

    let mut total_printed = 0u32;
    let mut current_page = 1u32;

    loop {
        // Use BrowseParams (the fluent builder from issue #6) to build params,
        // then copy into BrowsePostsParams for the client.
        let browse = BrowseParams::new()
            .page(current_page)
            .limit(PAGE_SIZE)
            .order("published_at DESC");

        let params = BrowsePostsParams {
            page: browse.get_page(),
            limit: browse.get_limit(),
            order: browse.get_order().map(|s| s.to_string()),
            ..Default::default()
        };

        let response = client.browse_posts(params).await?;
        let pagination = &response.meta.pagination;

        if response.posts.is_empty() {
            println!("(no posts found)");
            break;
        }

        for post in &response.posts {
            let published = post.published_at.as_deref().unwrap_or("(unpublished)");
            println!(
                "[{}] {} — {}",
                &published[..10.min(published.len())],
                post.title,
                post.slug
            );
            total_printed += 1;
        }

        println!(
            "\n  Page {}/{} — showing {}-{} of {} total posts",
            pagination.page,
            pagination.pages,
            pagination.start_index() + 1,
            pagination.end_index(),
            pagination.total
        );

        if !pagination.has_next() || current_page >= MAX_PAGES {
            break;
        }

        current_page += 1;
        println!();
    }

    println!("{:-<60}", "");
    println!("Listed {total_printed} post(s).");

    Ok(())
}
