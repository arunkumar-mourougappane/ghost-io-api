# Example Applications

> Design document for the `ghost-io-api` example programs.
> Each example lives under `examples/` in the crate root and can be run with
> `cargo run --example <name> -- [args]`.

---

## Table of Contents

1. [Overview](#overview)
2. [Directory Structure](#directory-structure)
3. [Cargo.toml Registration](#cargotoml-registration)
4. [Example 1 — `list-posts`](#example-1--list-posts)
5. [Example 2 — `site-stats`](#example-2--site-stats)
6. [Example 3 — `publish-post`](#example-3--publish-post)
7. [Example 4 — `publish-markdown`](#example-4--publish-markdown)
8. [Example 5 — `bulk-publish`](#example-5--bulk-publish)
9. [Example 6 — `upload-image`](#example-6--upload-image)
10. [Example 7 — `ghost-backup`](#example-7--ghost-backup)
11. [Example 8 — `ghost-cli`](#example-8--ghost-cli)
12. [Shared Conventions](#shared-conventions)

---

## Overview

| # | Name | API Used | Auth | Key Modules Demonstrated |
|---|------|----------|------|--------------------------|
| 1 | `list-posts` | Content | API key | `GhostContentClient`, `BrowseParams`, pagination |
| 2 | `site-stats` | Content | API key | Multi-endpoint fetch, `Settings`, aggregation |
| 3 | `publish-post` | Admin | JWT | `GhostAdminClient`, `PostCreate`, draft/publish |
| 4 | `publish-markdown` | Admin | JWT | `PublishPipeline`, `ProgressEvent`, image upload |
| 5 | `bulk-publish` | Admin | JWT | `PublishPipeline`, `tokio::spawn`, concurrency |
| 6 | `upload-image` | Admin | JWT | `GhostAdminClient::upload_image`, multipart |
| 7 | `ghost-backup` | Admin + Content | JWT + API key | Full post export, markdown serialization |
| 8 | `ghost-cli` | Admin + Content | JWT + API key | `clap`, credential storage, all major ops |

---

## Directory Structure

```
examples/
├── list_posts.rs          # single-file examples auto-discovered by Cargo
├── site_stats.rs
├── publish_post.rs
├── publish_markdown.rs
├── bulk_publish.rs
├── upload_image.rs
├── ghost_backup.rs
└── ghost_cli/             # multi-file example — needs [[example]] in Cargo.toml
    ├── main.rs
    └── cmd/
        ├── mod.rs
        ├── list.rs
        ├── publish.rs
        ├── upload.rs
        └── backup.rs
```

Cargo auto-discovers every `examples/*.rs` file. Multi-file examples that use
sub-modules must be registered explicitly (see next section).

---

## Cargo.toml Registration

Single-file examples in `examples/*.rs` require no registration; Cargo picks them
up automatically. The only entry needed is for `ghost-cli` which is multi-file:

```toml
[[example]]
name    = "ghost-cli"
path    = "examples/ghost_cli/main.rs"

[dev-dependencies]
# Shared across all examples
tokio     = { version = "1", features = ["full"] }
clap      = { version = "4", features = ["derive"] }   # ghost-cli only
anyhow    = "1"                                         # ergonomic error handling in examples
dotenvy   = "0.15"                                      # load .env files for test credentials
```

> Examples use `anyhow` and `dotenvy` as dev-dependencies so production library
> code stays lean. The library itself uses `thiserror` for typed errors.

---

## Example 1 — `list-posts`

### Purpose

Demonstrates the **Content API** read path. Fetches published posts from a Ghost
blog and prints them to stdout with a compact, human-readable format. Covers
pagination so very large blogs (hundreds of posts) work without memory pressure.

### Invocation

```bash
# Minimum — uses env vars GHOST_URL and GHOST_CONTENT_KEY
cargo run --example list_posts

# Override individual params
cargo run --example list_posts -- \
  --url   https://myblog.ghost.io \
  --key   <content-api-key>       \
  --tag   rust                    \
  --limit 5                       \
  --page  2
```

### CLI Args (parsed with `std::env::args` or `clap`)

| Argument | Default | Description |
|----------|---------|-------------|
| `--url` | `$GHOST_URL` | Blog base URL |
| `--key` | `$GHOST_CONTENT_KEY` | Content API key |
| `--tag` | _(none)_ | Filter to one tag slug |
| `--author` | _(none)_ | Filter to one author slug |
| `--limit` | `15` | Posts per page |
| `--page` | `1` | Page number |
| `--fields` | title,slug,published\_at | Comma-separated fields to include |

### Sample Output

```
Ghost Post Listing — myblog.ghost.io  (page 1 / 3, 32 total)
──────────────────────────────────────────────────────────────
 1  Building a Rust CLI Tool                      2026-05-10  #rust #cli
 2  Understanding Async Streams in Tokio           2026-04-28  #rust #async
 3  Ghost CMS Review for Developers               2026-04-01  #ghost #cms
 4  Zero-Copy Parsing with nom                    2026-03-15  #rust #parsing
 5  Deploying Ghost on Fly.io                     2026-02-20  #ghost #devops
──────────────────────────────────────────────────────────────
Next page: cargo run --example list_posts -- --page 2
```

### Key Code Sketch

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let client = GhostContentClient::new(&args.url, &args.key)?;

    let params = BrowseParams::new()
        .limit(args.limit)
        .page(args.page)
        .fields(["title", "slug", "published_at", "tags", "authors"])
        .filter_opt(args.tag.as_deref().map(|t| format!("tags:[{t}]")))
        .order("published_at desc");

    let response = client.browse_posts(params).await?;

    let meta = &response.meta.pagination;
    println!("page {} / {}, {} total", meta.page, meta.pages, meta.total);

    for post in &response.posts {
        println!("{:40}  {}  {}", post.title, post.published_at, tag_list(post));
    }
    Ok(())
}
```

### Library Modules Used

- `ghost_io_api::client::content::GhostContentClient`
- `ghost_io_api::params::browse::BrowseParams`
- `ghost_io_api::models::{Post, Pagination}`

---

## Example 2 — `site-stats`

### Purpose

Demonstrates **multi-endpoint Content API usage**. Fetches posts, pages, tags,
authors, and settings in parallel and prints a concise summary of the publication.
Useful as a health-check or onboarding tool to inspect an unfamiliar Ghost site.

### Invocation

```bash
cargo run --example site_stats
# or
cargo run --example site_stats -- --url https://demo.ghost.io --key <key>
```

### Sample Output

```
═══════════════════════════════════════════
  myblog.ghost.io — Site Stats
═══════════════════════════════════════════
  Title        : My Developer Blog
  Description  : Thoughts on Rust and systems programming
  Ghost version: 5.87.2

  Posts   : 32 published, 4 drafts
  Pages   : 7
  Tags    : 18  (top: #rust ×14, #async ×9, #ghost ×7)
  Authors : 2   (primary: Arunkumar Mourougappane)

  Paid tiers  : 1 (Standard — $9/mo)
  Free members: estimated (members API requires Admin key)
═══════════════════════════════════════════
```

### Key Code Sketch

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = GhostContentClient::new(&url, &key)?;

    // Fan out all requests in parallel
    let (posts, pages, tags, authors, settings) = tokio::try_join!(
        client.browse_posts(BrowseParams::new().limit(LimitAll)),
        client.browse_pages(BrowseParams::new().limit(LimitAll)),
        client.browse_tags(BrowseParams::new().limit(LimitAll)),
        client.browse_authors(BrowseParams::new().limit(LimitAll)),
        client.settings(),
    )?;

    let draft_count = posts.iter().filter(|p| p.status == PostStatus::Draft).count();

    // sort tags by post count descending
    let mut sorted_tags = tags.clone();
    sorted_tags.sort_by(|a, b| b.count.posts.cmp(&a.count.posts));

    print_report(&settings, &posts, draft_count, &pages, &sorted_tags, &authors);
    Ok(())
}
```

### Library Modules Used

- `ghost_io_api::client::content::GhostContentClient`
- `ghost_io_api::models::{Post, Page, Tag, Author, Settings, PostStatus}`
- `ghost_io_api::params::browse::BrowseParams`

---

## Example 3 — `publish-post`

### Purpose

Demonstrates the **Admin API write path** at its simplest. Creates a single blog
post from command-line arguments — either as a draft or immediately published.
No markdown, no media — pure API call to `POST /admin/posts/`.

### Invocation

```bash
# Create a draft
cargo run --example publish_post -- \
  --title "Hello from Rust" \
  --tags  "rust,ghost-io-api" \
  --draft

# Publish immediately
cargo run --example publish_post -- \
  --title "Hello from Rust" \
  --body  "This post was published via the Ghost Admin API from a Rust program." \
  --tags  "rust,ghost-io-api" \
  --publish

# Schedule for a future date (ISO 8601)
cargo run --example publish_post -- \
  --title          "Future post" \
  --publish-at     "2026-07-01T09:00:00.000Z"
```

### CLI Args

| Argument | Description |
|----------|-------------|
| `--title` | Post title (required) |
| `--body` | Post body as plain text (converted to minimal Lexical paragraph node) |
| `--tags` | Comma-separated tag slugs |
| `--draft` | Save as draft (default if neither flag given) |
| `--publish` | Publish immediately |
| `--publish-at` | Schedule for an ISO 8601 UTC datetime |
| `--feature-image` | URL of a hosted image to set as feature image |

### Sample Output

```
✓ Post created!
  ID     : 64a7c2f9b3e4a10001234abc
  Title  : Hello from Rust
  Status : draft
  URL    : https://myblog.ghost.io/p/64a7c2f9b3e4a10001234abc/
  Edit   : https://myblog.ghost.io/ghost/#/editor/post/64a7c2f9b3e4a10001234abc
```

### Key Code Sketch

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let client = GhostAdminClient::new(&url, &admin_key)?;

    let status = if args.publish {
        PostStatus::Published
    } else if args.publish_at.is_some() {
        PostStatus::Scheduled
    } else {
        PostStatus::Draft
    };

    let lexical = args.body.as_deref()
        .map(plain_text_to_lexical)
        .transpose()?;

    let post = PostCreate {
        title:      args.title.clone(),
        status,
        tags:       parse_tags(&args.tags),
        lexical,
        published_at: args.publish_at,
        feature_image: args.feature_image,
        ..Default::default()
    };

    let created = client.create_post(post).await?;
    println!("✓ Post created!  ID: {}  URL: {}", created.id, created.url.unwrap_or_default());
    Ok(())
}
```

### Notes

- `plain_text_to_lexical(text)` is a helper in the example that wraps each
  paragraph in a minimal `{"type":"paragraph","children":[{"text":"..."}]}` node
  and encodes the root document. It is not part of the library itself.
- To pass a full Lexical JSON body, the caller constructs it via
  `ghost_io_api::models::lexical::Document` and serialises with `serde_json`.

### Library Modules Used

- `ghost_io_api::client::admin::GhostAdminClient`
- `ghost_io_api::models::{PostCreate, PostStatus, TagRef}`

---

## Example 4 — `publish-markdown`

### Purpose

The primary **end-to-end pipeline** demo. Reads a single Markdown file from disk,
parses the YAML front matter, discovers local media references, uploads them to
Ghost's storage, converts the body to Lexical JSON, and creates the post. All
progress events are printed to stdout as they arrive.

This is the central showcase for `src/markdown/`.

### Invocation

```bash
cargo run --example publish_markdown -- \
  path/to/my-post.md

# Override front matter status from the CLI
cargo run --example publish_markdown -- \
  path/to/my-post.md --publish

cargo run --example publish_markdown -- \
  path/to/my-post.md --draft

# Dry-run: parse, scan, upload media, but stop before creating the post
cargo run --example publish_markdown -- \
  path/to/my-post.md --dry-run
```

### Markdown File Format

```markdown
---
title: "Building a Ghost API Client in Rust"
slug: ghost-api-client-rust
status: draft
tags:
  - Rust
  - Ghost CMS
authors:
  - arun
feature_image: ./assets/hero.png
excerpt: "A deep dive into building a fully typed async Ghost client in Rust."
---

# Introduction

Some intro text.

![Architecture Diagram](./assets/arch-diagram.png)

More content here…
```

For the full front matter schema see
[`features/markdown-publish-pipeline.md`](../features/markdown-publish-pipeline.md).

### Sample Output

```
[1/5] Front matter parsed
      title  : Building a Ghost API Client in Rust
      slug   : ghost-api-client-rust
      status : draft
      tags   : Rust, Ghost CMS

[2/5] Media discovered: 2 local file(s)
      ./assets/hero.png        (142 KB)
      ./assets/arch-diagram.png (88 KB)

[3/5] Uploading media…
      hero.png          ████████████████████  100%  142 KB
      arch-diagram.png  ████████████████████  100%   88 KB
      All media uploaded (2/2)

[4/5] Building Lexical document…
      Nodes: 1 heading, 3 paragraphs, 2 image cards, 1 code block

[5/5] Creating post via Admin API…

✓ Done!
  ID     : 64a7c2f9b3e4a10001234abc
  Status : draft
  URL    : https://myblog.ghost.io/p/ghost-api-client-rust/
```

### Key Code Sketch

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let markdown = tokio::fs::read_to_string(&args.file).await?;

    let admin_client = GhostAdminClient::new(&url, &admin_key)?;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ProgressEvent>();

    // Print events as they arrive in a background task
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            print_progress(&event);
        }
    });

    let override_status = if args.publish {
        Some(PostStatus::Published)
    } else if args.draft {
        Some(PostStatus::Draft)
    } else {
        None
    };

    let result = PublishPipeline::new(admin_client)
        .with_progress(tx)
        .publish_markdown(&markdown, override_status)
        .await?;

    println!("✓ Done!  ID: {}  URL: {}", result.id, result.url.unwrap_or_default());
    Ok(())
}
```

### Library Modules Used

- `ghost_io_api::markdown::PublishPipeline`
- `ghost_io_api::markdown::progress::ProgressEvent`
- `ghost_io_api::client::admin::GhostAdminClient`
- `ghost_io_api::models::PostStatus`

### Additional Dev-Dependencies

```toml
indicatif = "0.17"   # progress bars (used only in this example)
```

---

## Example 5 — `bulk-publish`

### Purpose

Demonstrates **concurrent publishing** of an entire directory of Markdown files.
Each `.md` file is processed in a separate `tokio::spawn` task. A per-file
progress line (with file name) is printed to stderr; final success/failure
counts go to stdout. Useful for initial content migrations.

### Invocation

```bash
# Publish all .md files in a directory
cargo run --example bulk_publish -- ./posts/

# Limit concurrency (default: 4 concurrent uploads at once)
cargo run --example bulk_publish -- ./posts/ --concurrency 2

# Dry-run: parse + upload media, skip post creation
cargo run --example bulk_publish -- ./posts/ --dry-run
```

### Sample Output

```
Scanning ./posts/ → 12 markdown file(s) found

[ghost-api-client-rust   ]  ✓ published  (3 media, 1.4s)
[async-streams-tokio     ]  ✓ published  (1 media, 0.8s)
[zero-copy-nom           ]  ✓ published  (0 media, 0.3s)
[deploying-ghost-fly     ]  ✗ failed: HTTP 422 — slug already exists
[understanding-lexical   ]  ✓ published  (2 media, 1.1s)
…

────────────────────────────────
  Total  : 12
  OK     : 11
  Failed :  1  (see above)
```

### Key Code Sketch

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let paths = collect_markdown_files(&args.dir)?;

    let semaphore = Arc::new(tokio::sync::Semaphore::new(args.concurrency));
    let mut handles = Vec::new();

    for path in paths {
        let sem   = semaphore.clone();
        let client = GhostAdminClient::new(&url, &admin_key)?;

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await?;
            let markdown = tokio::fs::read_to_string(&path).await?;
            let slug = path.file_stem().and_then(|s| s.to_str()).unwrap_or("?");

            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            tokio::spawn(async move {
                while let Some(ev) = rx.recv().await {
                    eprintln!("[{:<25}]  {}", slug, format_event(&ev));
                }
            });

            PublishPipeline::new(client)
                .with_progress(tx)
                .publish_markdown(&markdown, None)
                .await
                .map(|post| (slug.to_owned(), post))
        });

        handles.push(handle);
    }

    let results = futures_util::future::join_all(handles).await;
    print_summary(&results);
    Ok(())
}
```

### Library Modules Used

- `ghost_io_api::markdown::PublishPipeline`
- `ghost_io_api::markdown::progress::ProgressEvent`
- `ghost_io_api::client::admin::GhostAdminClient`

### Additional Dev-Dependencies

```toml
futures-util = "0.3"
```

---

## Example 6 — `upload-image`

### Purpose

A focused demo of the **Admin API image upload** endpoint (`POST /admin/images/upload/`).
Uploads one or more local image files and prints the Ghost-hosted URL for each.
Useful for testing media storage configuration or generating hosted image URLs
for use in other posts.

### Invocation

```bash
# Upload a single file
cargo run --example upload_image -- ./photo.jpg

# Upload multiple files
cargo run --example upload_image -- ./images/*.png

# Set a custom alt text on the uploaded image
cargo run --example upload_image -- ./hero.jpg --alt "Hero image for the post"

# Specify a target subdirectory (Ghost organises by year/month by default)
cargo run --example upload_image -- ./photo.jpg --ref "2026/06/photo"
```

### Sample Output

```
Uploading 3 file(s)…

  photo.jpg       → https://myblog.ghost.io/content/images/2026/06/photo.jpg
  hero.png        → https://myblog.ghost.io/content/images/2026/06/hero.png
  thumbnail.webp  → https://myblog.ghost.io/content/images/2026/06/thumbnail.webp

Done. 3/3 uploaded successfully.
```

### Key Code Sketch

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let client = GhostAdminClient::new(&url, &admin_key)?;

    for path in &args.files {
        let file   = tokio::fs::File::open(path).await?;
        let size   = file.metadata().await?.len();
        let stream = tokio_util::io::ReaderStream::new(file);
        let body   = reqwest::Body::wrap_stream(stream);

        let part = reqwest::multipart::Part::stream_with_length(body, size)
            .file_name(path.file_name().unwrap().to_str().unwrap().to_owned())
            .mime_str(mime_guess::from_path(path).first_or_octet_stream().as_ref())?;

        let upload_ref = args.upload_ref.clone()
            .unwrap_or_else(|| path.file_stem().unwrap().to_str().unwrap().to_owned());

        let hosted_url = client
            .upload_image(part, &upload_ref, args.alt.as_deref())
            .await?;

        println!("{:<20}  →  {}", path.display(), hosted_url);
    }
    Ok(())
}
```

### Error Cases Shown

- File not found → `GhostError::Io`
- Unsupported MIME type rejected by Ghost → `GhostError::Api { code: 422, .. }`
- Network timeout → `GhostError::Request`

### Library Modules Used

- `ghost_io_api::client::admin::GhostAdminClient`
- `ghost_io_api::error::GhostError`

### Additional Dev-Dependencies

```toml
mime_guess = "2"
```

---

## Example 7 — `ghost-backup`

### Purpose

Demonstrates **full-read via the Admin API** and **serialising posts back to
Markdown files**. Fetches all posts (published + drafts + scheduled) including
their tags, authors, and Lexical body, converts each post back to a Markdown file
with YAML front matter, and writes them to a local output directory.

This is the natural companion to `publish-markdown` — a round-trip export/import
workflow for content migration or offline editing.

### Invocation

```bash
# Export all posts to ./backup/
cargo run --example ghost_backup -- --out ./backup/

# Export only published posts
cargo run --example ghost_backup -- --out ./backup/ --status published

# Export including member-only (paid) posts
cargo run --example ghost_backup -- --out ./backup/ --visibility all
```

### Output File Naming

Each post is written as `{slug}.md`. If two posts share a slug (e.g. a draft and
a published revision), the draft is written as `{slug}.draft.md`.

### Sample Output

```
Fetching posts from myblog.ghost.io…
  32 published, 4 drafts, 1 scheduled — 37 total

Writing to ./backup/…
  ghost-api-client-rust.md               ✓
  async-streams-tokio.md                 ✓
  zero-copy-nom.md                       ✓
  …
  my-unfinished-post.draft.md            ✓

Backup complete. 37/37 files written to ./backup/
```

### Key Code Sketch

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let admin_client = GhostAdminClient::new(&url, &admin_key)?;

    // Fetch all posts (walk pages until exhausted)
    let mut posts: Vec<Post> = Vec::new();
    let mut page = 1u32;
    loop {
        let resp = admin_client.browse_posts(
            BrowseParams::new()
                .limit(100)
                .page(page)
                .include(["tags", "authors"])
        ).await?;
        let total_pages = resp.meta.pagination.pages;
        posts.extend(resp.posts);
        if page >= total_pages { break; }
        page += 1;
    }

    tokio::fs::create_dir_all(&args.out).await?;

    for post in &posts {
        let markdown = post_to_markdown(post)?;   // converts Lexical → Markdown + front matter
        let filename = if post.status == PostStatus::Draft {
            format!("{}.draft.md", post.slug)
        } else {
            format!("{}.md", post.slug)
        };
        tokio::fs::write(args.out.join(&filename), markdown).await?;
        println!("  {:<50}  ✓", filename);
    }
    Ok(())
}
```

### `post_to_markdown` Internals

This helper (in the example file, not the library) performs:
1. Serialize `post.tags` and `post.authors` into YAML front matter lists
2. Walk the Lexical document tree and emit CommonMark text node-by-node:
   - `paragraph` → plain text with trailing newline
   - `heading` → `#` through `######` prefixes
   - `image` card → `![alt](url)` (uses the Ghost-hosted URL already stored in the card)
   - `codeblock` card → fenced code block with language hint
   - `html` card → raw HTML fence
   - `divider` → `---`
   - Other card types → `<!-- unsupported: {type} -->` comment

### Library Modules Used

- `ghost_io_api::client::admin::GhostAdminClient`
- `ghost_io_api::models::{Post, PostStatus, Tag, Author}`
- `ghost_io_api::params::browse::BrowseParams`

---

## Example 8 — `ghost-cli`

### Purpose

A full **command-line interface** wrapping all major library operations. Targets
developers and power users who want to manage Ghost from their terminal without
writing custom code. Demonstrates `clap`-based subcommand routing, the credential
storage module, and every public-facing API of the library.

This is the largest example and lives in `examples/ghost_cli/` as a multi-file
Cargo example.

### Invocation

```bash
# First-time setup — store credentials (prompts interactively)
cargo run --example ghost-cli -- init

# List posts
cargo run --example ghost-cli -- list posts --tag rust --limit 10

# Publish a single markdown file
cargo run --example ghost-cli -- publish ./my-post.md

# Publish and go live immediately
cargo run --example ghost-cli -- publish ./my-post.md --status published

# Bulk publish a directory
cargo run --example ghost-cli -- publish-dir ./content/ --concurrency 4

# Upload an image and print its URL
cargo run --example ghost-cli -- upload image ./hero.jpg

# Backup all posts to disk
cargo run --example ghost-cli -- backup --out ./backup/

# Show site stats
cargo run --example ghost-cli -- stats
```

### Subcommand Tree

```
ghost-cli
├── init                    Store credentials (URL, content key, admin key) securely
│   └── --profile <name>    Named profile (default: "default")
│
├── list <resource>         Read-only browse via Content API
│   ├── posts               List posts
│   │   ├── --tag <slug>    Filter by tag
│   │   ├── --author <slug> Filter by author
│   │   ├── --limit <n>     Results per page (default 15)
│   │   └── --page <n>      Page number
│   ├── pages               List pages
│   ├── tags                List tags (sorted by post count)
│   └── authors             List authors
│
├── stats                   Print site-wide statistics (wraps site-stats example)
│
├── publish <file.md>       Publish one markdown file via the pipeline
│   ├── --status <s>        Override status: draft | published | scheduled
│   ├── --publish-at <dt>   ISO 8601 datetime for scheduled posts
│   └── --dry-run           Parse + upload media only; skip post creation
│
├── publish-dir <dir/>      Bulk publish a directory of .md files
│   ├── --concurrency <n>   Max parallel uploads (default 4)
│   ├── --status <s>        Override status for every file
│   └── --dry-run
│
├── upload <type> <file>    Upload media
│   └── image <file>        Upload an image; print hosted URL
│       ├── --alt <text>    Alt text
│       └── --ref <ref>     Storage ref hint
│
├── backup                  Export posts to markdown files
│   ├── --out <dir>         Output directory (default ./ghost-backup/)
│   └── --status <s>        Filter by status: all | published | draft
│
└── profiles                Manage stored credential profiles
    ├── list                List all profiles
    ├── show <name>         Print profile details (keys redacted)
    └── delete <name>       Delete a profile
```

### Source File Structure

```
examples/ghost_cli/
├── main.rs          # Entry point: parse top-level subcommand, dispatch
└── cmd/
    ├── mod.rs       # Re-exports all command modules
    ├── init.rs      # `init` — interactive credential setup
    ├── list.rs      # `list posts|pages|tags|authors`
    ├── stats.rs     # `stats`
    ├── publish.rs   # `publish <file>` and `publish-dir`
    ├── upload.rs    # `upload image`
    ├── backup.rs    # `backup`
    └── profiles.rs  # `profiles list|show|delete`
```

### Credential Flow (`init` command)

```rust
// cmd/init.rs
pub async fn run(profile: &str) -> anyhow::Result<()> {
    let url       = prompt("Ghost URL (e.g. https://myblog.ghost.io): ")?;
    let content   = prompt("Content API key: ")?;
    let admin     = prompt("Admin API key (id:hex_secret): ")?;
    let password  = rpassword::prompt_password("Encryption passphrase: ")?;

    let creds = Credentials { url, content_key: content, admin_key: admin };
    CredentialStore::new(profile).save(&creds, password.as_bytes()).await?;
    println!("✓ Profile '{}' saved.", profile);
    Ok(())
}
```

The `CredentialStore` uses XChaCha20-Poly1305 + Argon2id to encrypt credentials
at rest. See [`design/credential-storage.md`](../design/credential-storage.md).

### `publish` Command Flow

```rust
// cmd/publish.rs
pub async fn run(args: &PublishArgs) -> anyhow::Result<()> {
    let creds  = load_credentials(&args.profile).await?;
    let client = GhostAdminClient::new(&creds.url, &creds.admin_key)?;

    let markdown = tokio::fs::read_to_string(&args.file).await?;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        while let Some(ev) = rx.recv().await { print_event(&ev); }
    });

    let result = PublishPipeline::new(client)
        .with_progress(tx)
        .publish_markdown(&markdown, args.status_override)
        .await?;

    println!("✓  {}  →  {}", result.slug, result.url.unwrap_or_default());
    Ok(())
}
```

### Additional Dev-Dependencies

```toml
clap      = { version = "4", features = ["derive"] }
rpassword = "7"       # secure password prompt (no echo)
indicatif = "0.17"    # progress bars
```

### Library Modules Used

- `ghost_io_api::client::content::GhostContentClient`
- `ghost_io_api::client::admin::GhostAdminClient`
- `ghost_io_api::credentials::{CredentialStore, Credentials}`
- `ghost_io_api::markdown::{PublishPipeline, ProgressEvent}`
- `ghost_io_api::models::{Post, Page, Tag, Author, PostStatus}`
- `ghost_io_api::params::browse::BrowseParams`

---

## Shared Conventions

All examples follow these conventions for consistency and ease of review.

### Credential Resolution Order

Each example resolves Ghost credentials in this priority order:

1. **CLI flags** (`--url`, `--key`, `--admin-key`)
2. **Environment variables** (`GHOST_URL`, `GHOST_CONTENT_KEY`, `GHOST_ADMIN_KEY`)
3. **`.env` file** in the current working directory (loaded via `dotenvy`)
4. **Credential store** (only `ghost-cli`, which manages named profiles)

### Error Reporting

Examples use `anyhow::Result<()>` as the return type for `main()` so that `?`
propagates errors naturally. On failure the error chain is printed by the
`anyhow` runtime without a custom handler:

```
Error: failed to upload image
Caused by:
    0: HTTP 413 — Request Entity Too Large
    1: POST https://myblog.ghost.io/ghost/api/admin/images/upload/ → 413
```

The library's own `GhostError` implements `std::error::Error`, so it integrates
cleanly with `anyhow` via `.context()`.

### Environment Variable Reference

| Variable | Used By | Description |
|----------|---------|-------------|
| `GHOST_URL` | all | Blog base URL |
| `GHOST_CONTENT_KEY` | Content API examples | `?key=` value |
| `GHOST_ADMIN_KEY` | Admin API examples | `{id}:{hex_secret}` |
| `GHOST_PROFILE` | `ghost-cli` | Named credential profile (default: `"default"`) |

### `.env` File Example

```dotenv
GHOST_URL=https://myblog.ghost.io
GHOST_CONTENT_KEY=22444f78447824223cefc48062
GHOST_ADMIN_KEY=64a7c2f9b3e4a10001234abc:aabbccddeeff001122334455aabbccddeeff001122334455aabbccddeeff0011
```

### Progress Event Printing

The `print_event(ev: &ProgressEvent)` helper used across examples maps each
variant to a single-line console message:

| ProgressEvent variant | Printed as |
|-----------------------|-----------|
| `FrontMatterParsed { title, .. }` | `[1/5] Front matter parsed — "{title}"` |
| `MediaDiscovered { count, .. }` | `[2/5] Media discovered: {count} file(s)` |
| `MediaUploadStarted { path, size }` | `  → uploading {path}  ({size} bytes)` |
| `MediaUploadProgress { path, bytes_sent, total }` | progress bar update |
| `MediaUploadComplete { path, hosted_url }` | `  ✓ {path}  →  {hosted_url}` |
| `MediaUploadFailed { path, error }` | `  ✗ {path}  FAILED: {error}` |
| `PostCreating { title }` | `[5/5] Creating post "{title}"…` |
| `PostCreated { id, url }` | `✓ Done!  {id}  {url}` |
