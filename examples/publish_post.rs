//! Example: create a draft post and then publish it via the Ghost Admin API.
//!
//! The example walks through three steps that mirror a real authoring workflow:
//!
//! 1. **Create** a post in `draft` status.
//! 2. **Publish** it by updating its `status` to `published` — using the
//!    `updated_at` token returned from step 1 for optimistic concurrency.
//! 3. **Clean up** by deleting the post so the example can be run repeatedly
//!    without cluttering the Ghost installation.
//!
//! # Running the example
//!
//! ```sh
//! GHOST_URL=https://your-site.ghost.io \
//! GHOST_ADMIN_KEY=6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1 \
//! cargo run --example publish_post
//! ```
//!
//! Both variables are **required** — the Admin API key must come from
//! *Ghost Admin → Settings → Integrations → Add custom integration*.
//!
//! # What the output looks like
//!
//! ```text
//! ── Ghost Admin API: publish_post example ──────────────────────────────────
//! Site: https://your-site.ghost.io
//!
//! [1/3] Creating draft post …
//!       id    : 6748592f4b9b7700010f6564
//!       title : publish_post example — 2026-06-21T14:30:00Z
//!       slug  : publish-post-example-2026-06-21t14-30-00z
//!       status: draft
//!
//! [2/3] Publishing the draft …
//!       status: published
//!       url   : https://your-site.ghost.io/publish-post-example-2026-06-21t14-30-00z/
//!
//! [3/3] Cleaning up — deleting post 6748592f4b9b7700010f6564 …
//!       Done.
//!
//! ── Finished ───────────────────────────────────────────────────────────────
//! ```

use ghost_io_api::auth::admin::AdminApiKey;
use ghost_io_api::client::admin::GhostAdminClient;
use ghost_io_api::error::Result;
use ghost_io_api::models::post::{PostCreate, PostStatus, PostUpdate, TagRef};

#[tokio::main]
async fn main() -> Result<()> {
    // ── Configuration ─────────────────────────────────────────────────────────

    let url = require_env("GHOST_URL");
    let raw_key = require_env("GHOST_ADMIN_KEY");

    let key = AdminApiKey::new(raw_key)?;
    let client = GhostAdminClient::new(&url, key)?;

    println!(
        "── Ghost Admin API: publish_post example {}",
        "─".repeat(30)
    );
    println!("Site: {url}");

    // Use a timestamped title so repeated runs don't collide on the slug.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let title = format!("publish_post example — {now}");

    // ── Step 1: Create a draft ────────────────────────────────────────────────

    println!("\n[1/3] Creating draft post …");

    let draft = client
        .create_post(PostCreate {
            title: title.clone(),
            status: Some(PostStatus::Draft),
            custom_excerpt: Some("Created by the ghost-io-api publish_post example.".to_string()),
            tags: Some(vec![TagRef::by_name("ghost-io-api-example")]),
            ..Default::default()
        })
        .await?;

    println!("      id    : {}", draft.id);
    println!("      title : {}", draft.title);
    println!("      slug  : {}", draft.slug);
    println!(
        "      status: {}",
        status_label(draft.status == PostStatus::Published)
    );

    // `updated_at` is required by Ghost for every update request — it acts as
    // an optimistic concurrency token so concurrent edits are never silently
    // overwritten.
    let updated_at = draft.updated_at.clone().unwrap_or_default();

    // ── Step 2: Publish the draft ─────────────────────────────────────────────

    println!("\n[2/3] Publishing the draft …");

    let published = client
        .update_post(
            &draft.id,
            PostUpdate {
                updated_at,
                status: Some(PostStatus::Published),
                ..Default::default()
            },
        )
        .await?;

    println!(
        "      status: {}",
        status_label(published.status == PostStatus::Published)
    );
    if let Some(url) = &published.url {
        println!("      url   : {url}");
    }

    // ── Step 3: Delete the post (cleanup) ─────────────────────────────────────

    println!("\n[3/3] Cleaning up — deleting post {} …", published.id);

    client.delete_post(&published.id).await?;

    println!("      Done.");

    println!("\n{}", "─".repeat(60));
    println!("── Finished {}", "─".repeat(49));

    Ok(())
}

fn require_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| {
        eprintln!("ERROR: environment variable {name} is required.");
        eprintln!("       Set it before running this example.");
        std::process::exit(1);
    })
}

fn status_label(is_published: bool) -> &'static str {
    if is_published {
        "published"
    } else {
        "draft"
    }
}
