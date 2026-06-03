# Markdown Publish Pipeline

> Research notes for implementing a markdown-to-Ghost publishing pipeline in `ghost-io-api`.
> Covers: front matter parsing, markdown AST walking, media detection + upload, Lexical conversion,
> progress callbacks, and the public API design.

---

## Table of Contents

1. [Overview](#overview)
2. [Markdown Document Format](#markdown-document-format)
3. [Crate Selection](#crate-selection)
4. [Pipeline Stages](#pipeline-stages)
5. [Stage 1 — Front Matter Parsing](#stage-1--front-matter-parsing)
6. [Stage 2 — Markdown Parsing and Media Discovery](#stage-2--markdown-parsing-and-media-discovery)
7. [Stage 3 — Media Upload with Progress](#stage-3--media-upload-with-progress)
8. [Stage 4 — Markdown to Lexical Conversion](#stage-4--markdown-to-lexical-conversion)
9. [Stage 5 — Post Creation](#stage-5--post-creation)
10. [Progress Event Design](#progress-event-design)
11. [Upload Progress Tracking](#upload-progress-tracking)
12. [Public API Design](#public-api-design)
13. [Module Structure](#module-structure)
14. [Dependency Summary](#dependency-summary)

---

## Overview

The goal is a pipeline that takes a single Markdown string (from a file or string literal),
and publishes it to Ghost with all images/videos uploaded to Ghost's storage automatically.
The pipeline is fully async, and emits typed progress events throughout.

```
Markdown string
      │
      ▼
┌─────────────────────┐
│ 1. Parse front matter│  → PostFrontMatter (title, tags, status, …)
└─────────────────────┘
      │  markdown body
      ▼
┌─────────────────────┐
│ 2. Walk Markdown AST │  → List of MediaRef (local path or remote URL)
└─────────────────────┘         + intermediate representation (IR)
      │
      ▼
┌──────────────────────────┐
│ 3. Upload local media    │  → path → Ghost hosted URL map
│    (with progress events)│     + progress: MediaUploadStarted/Progress/Complete
└──────────────────────────┘
      │
      ▼
┌─────────────────────┐
│ 4. Build Lexical JSON │  → `lexical` string (Ghost post body)
└─────────────────────┘
      │
      ▼
┌─────────────────────┐
│ 5. POST /admin/posts/│  → CreatedPost (id, url, status)
│    or PATCH to update│     + progress: PostCreating / PostCreated
└─────────────────────┘
```

---

## Markdown Document Format

A markdown file for this pipeline uses **YAML front matter** between `---` delimiters,
followed by the post body in standard CommonMark markdown.

```markdown
---
title: "Building a Ghost API Client in Rust"
slug: ghost-api-client-rust
status: published
tags:
  - Rust
  - Ghost CMS
  - "#internal-draft"
authors:
  - author@example.com
feature_image: ./images/hero.jpg
feature_image_alt: "A Rust crab wearing a ghost hat"
excerpt: "Learn how to build a type-safe Ghost CMS API client in Rust."
---

## Introduction

Here is the opening paragraph of the post.

![An inline image](./images/inline-image.jpg)

A paragraph with a [link](https://example.com).

```rust
fn main() {
    println!("Hello, Ghost!");
}
```

> A blockquote with important info.
```

### Supported front matter fields

| Field | Type | Notes |
|-------|------|-------|
| `title` | string | Required |
| `slug` | string | Optional — Ghost generates from title if absent |
| `status` | `"draft"` \| `"published"` \| `"scheduled"` | Default: `"draft"` |
| `tags` | `Vec<String>` | Name strings or `#`-prefixed internal tags |
| `authors` | `Vec<String>` | Email addresses |
| `feature_image` | path or URL | Local path is uploaded automatically |
| `feature_image_alt` | string | |
| `feature_image_caption` | string | |
| `excerpt` | string | |
| `publish_at` | RFC 3339 datetime | Used when `status = "scheduled"` |
| `custom_excerpt` | string | |
| `canonical_url` | URL string | |

---

## Crate Selection

### Markdown parsing: `pulldown-cmark`

**Chosen over `comrak`** for the following reasons:

- Event-based (pull parser) — no need to allocate an arena-based tree. Iterate events
  in a single pass while accumulating state.
- Lower allocations. `CowStr` avoids copying most string data.
- Native support for YAML metadata blocks via `Options::ENABLE_YAML_STYLE_METADATA_BLOCKS`,
  surfaced as `Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle))` + `Event::Text(content)`.
- `OffsetIter` gives source byte ranges per event (useful for diagnostics).

```toml
pulldown-cmark = { version = "0.13", features = [] }
```

**Key types:**

| Type | Description |
|------|-------------|
| `Parser` | Iterator over `Event<'a>` |
| `Event::Start(Tag::Image { dest_url, title, .. })` | Opening of an image — inner Text events are the alt text |
| `Event::End(TagEnd::Image)` | Close of image |
| `Event::Start(Tag::Heading { level, .. })` | `h1`–`h6` |
| `Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang)))` | Fenced code block |
| `Event::Start(Tag::HtmlBlock)` | Raw HTML block |
| `Event::Rule` | Horizontal rule (`---`) |
| `Event::Start(Tag::BlockQuote(_))` | Blockquote |
| `Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle))` | YAML front matter |
| `Event::Text(s)` | Text content inside any open tag |

### Front matter parsing: `gray_matter`

Parses YAML/TOML/JSON front matter delimited by `---`. Strips the front matter block
and returns both the parsed data (deserialised to a custom struct via `serde`) and the
remaining markdown body.

```toml
gray_matter = { version = "0.3", features = ["yaml"] }
```

Usage:

```rust
use gray_matter::{Matter, ParsedEntity};
use gray_matter::engine::YAML;
use serde::Deserialize;

#[derive(Deserialize)]
struct PostFrontMatter {
    title: String,
    #[serde(default = "default_status")]
    status: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    authors: Vec<String>,
    slug: Option<String>,
    feature_image: Option<String>,
    feature_image_alt: Option<String>,
    excerpt: Option<String>,
}

fn parse(input: &str) -> (PostFrontMatter, String) {
    let matter = Matter::<YAML>::new();
    let parsed = matter.parse::<PostFrontMatter>(input).unwrap();
    (parsed.data.unwrap(), parsed.content)
}
```

### Upload progress streaming: `tokio-util` + `futures-util`

`tokio_util::io::ReaderStream` converts a `tokio::fs::File` (implements `AsyncRead`)
into a `Stream<Item = Result<Bytes, io::Error>>`. This stream can be wrapped in a custom
byte-counting adapter that sends progress events as chunks are consumed, then passed to
`reqwest::Body::wrap_stream(...)`.

```toml
tokio-util = { version = "0.7", features = ["io"] }
futures-util = "0.3"
bytes = "1"
```

### Progress channels: `tokio::sync::mpsc`

Use `tokio::sync::mpsc::unbounded_channel()`. The pipeline holds an
`UnboundedSender<ProgressEvent>` and the caller holds the `UnboundedReceiver`.

- `UnboundedSender::send()` never blocks or returns an error worth aborting for (just a
  dropped receiver), so it's safe to fire-and-forget progress events without await.
- The sender is `Clone + Send + 'static` — it can be moved into closures and tasks.

---

## Pipeline Stages

### Stage 1 — Front Matter Parsing

```
markdown string
    │
    ▼ gray_matter::Matter::<YAML>::new().parse::<PostFrontMatter>(input)
    │
    ├─ PostFrontMatter { title, status, tags, authors, feature_image, ... }
    └─ markdown body string (everything after the closing ---)
```

The `feature_image` field may be a relative local path (e.g., `./images/hero.jpg`)
or an already-hosted URL. Local paths are queued for upload in Stage 3.

---

### Stage 2 — Markdown Parsing and Media Discovery

Walk the `pulldown-cmark` event stream to:

1. Detect all image/media references and classify them as `Local(PathBuf)` or `Remote(Url)`.
2. Build an **intermediate representation (IR)** — a `Vec<IrNode>` that mirrors the
   Lexical tree structure but still has unresolved local paths.

#### Media reference detection

```rust
let mut parser_options = Options::empty();
// (No metadata block feature needed — gray_matter already stripped front matter)
let parser = Parser::new_ext(&body, parser_options);

let mut media_refs: IndexMap<String, MediaRef> = IndexMap::new();

for event in parser {
    match event {
        Event::Start(Tag::Image { dest_url, .. }) => {
            let src = dest_url.to_string();
            if !src.starts_with("http://") && !src.starts_with("https://") {
                media_refs.insert(src.clone(), MediaRef::Local(base_dir.join(&src)));
            } else {
                media_refs.insert(src.clone(), MediaRef::Remote(src.clone()));
            }
        }
        _ => {}
    }
}
```

`IndexMap` (from `indexmap` crate) preserves insertion order so uploads happen in
document order, which matters for `MediaUploadStarted { index }` progress reporting.

#### Media classification

```rust
pub enum MediaRef {
    Local(PathBuf),     // Needs uploading — path relative to the markdown file
    Remote(String),     // Already a URL — use as-is in the card
}
```

**Video detection by extension**: if a local/remote path ends in `.mp4`, `.webm`, `.mov`,
`.avi`, `.mkv` → emit a `video` card instead of an `image` card. Audio: `.mp3`, `.wav`,
`.ogg`, `.flac`, `.m4a` → `audio` card.

---

### Stage 3 — Media Upload with Progress

For each `MediaRef::Local` in the collected set:

1. Open the file: `tokio::fs::File::open(&path).await?`
2. Get size: `file.metadata().await?.len()`
3. Emit `ProgressEvent::MediaUploadStarted { file, index, total, size_bytes }`
4. Wrap file in `ReaderStream` then a `ProgressStream` counting-wrapper
5. Build `reqwest::multipart::Form` with the stream as the file part
6. POST to `/admin/images/upload/`
7. On success: emit `ProgressEvent::MediaUploadComplete { file, hosted_url }`
8. Store `original_src → hosted_url` in an `upload_map: HashMap<String, String>`

Remote URLs skip upload entirely. The `upload_map` is used in Stage 4 to rewrite
`dest_url` values before building Lexical cards.

---

### Stage 4 — Markdown to Lexical Conversion

Re-walk (or replay the stored IR) with the `upload_map` available, and produce a
`LexicalDocument` (the Rust types from `src/markdown/lexical.rs`). Finally call
`serde_json::to_string(&document)` to get the `lexical` field string.

#### Mapping table

| Markdown element | Lexical node type | Notes |
|-----------------|-------------------|-------|
| Paragraph | `paragraph` | All child inlines preserved |
| `# H1` – `###### H6` | `heading` with `tag: "h1"` – `"h6"` | |
| `![alt](local.jpg)` | `image` card | `src` = hosted URL from upload_map |
| `![alt](remote.jpg)` | `image` card | `src` = remote URL as-is |
| `![alt](video.mp4)` | `video` card | Detected by extension |
| `![alt](audio.mp3)` | `audio` card | Detected by extension |
| `` ```lang ... ``` `` | `codeblock` card | `language` = lang string |
| `<div>...</div>` HTML block | `html` card | Raw HTML preserved |
| `---` rule | `horizontalrule` | |
| `> blockquote` | `blockquote` | |
| `| table |` | `html` card | Rendered to HTML via pulldown-cmark's html module |
| `**strong**` | `format: 1` text node (bold flag) | Inline inside paragraph |
| `*em*` | `format: 2` text node (italic flag) | Inline inside paragraph |
| `~~strike~~` | `format: 8` text node | Requires `ENABLE_STRIKETHROUGH` |
| `` `code` `` | `code` inline node | |
| `[text](url)` | link-wrapped text node | |

#### Text format flags (Ghost Lexical)

Ghost Lexical text nodes use a bitmask for inline formatting:

| Flag | Value |
|------|-------|
| Bold | `1` |
| Italic | `2` |
| Strikethrough | `4` |
| Underline | `8` |
| Code | `16` |
| Subscript | `32` |
| Superscript | `64` |
| Highlight | `128` |

Multiple flags combine with bitwise OR: bold+italic = `3`.

#### Lexical IR types for the converter

```rust
pub struct LexicalDocument {
    pub root: LexicalRoot,
}

pub struct LexicalRoot {
    pub children: Vec<LexicalNode>,
    pub direction: &'static str,  // "ltr"
    pub format: &'static str,     // ""
    pub indent: u8,               // 0
    pub version: u8,              // 1
}

pub enum LexicalNode {
    Paragraph(ParagraphNode),
    Heading(HeadingNode),
    // --- cards ---
    Image(ImageCard),
    Video(VideoCard),
    Audio(AudioCard),
    CodeBlock(CodeBlockCard),
    Html(HtmlCard),
    HorizontalRule,
    Blockquote(BlockquoteNode),
}
```

Serialise via `serde` with `#[serde(tag = "type", rename_all = "lowercase")]` on the enum
variants — but note the mismatch: `"horizontalrule"` is all lowercase with no separator,
so use `#[serde(rename = "horizontalrule")]` on that variant explicitly.

---

### Stage 5 — Post Creation

Emit `ProgressEvent::PostCreating`, then POST to `/admin/posts/`:

```json
{
  "posts": [{
    "title": "<front_matter.title>",
    "slug": "<front_matter.slug | undefined>",
    "status": "<front_matter.status>",
    "lexical": "<serde_json::to_string(&lexical_document)>",
    "tags": ["<tag_name>", ...],
    "authors": ["<email>", ...],
    "feature_image": "<hosted URL after upload>",
    "feature_image_alt": "<front_matter.feature_image_alt>",
    "custom_excerpt": "<front_matter.excerpt>"
  }]
}
```

On success, emit `ProgressEvent::PostCreated { post_id, post_url, status }`.

---

## Progress Event Design

```rust
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    // --- Parse phase ---

    /// Front matter parsed successfully.
    FrontMatterParsed {
        title: String,
        status: String,
    },

    /// Markdown scanned; reports how many local media files were found.
    MediaDiscovered {
        local_count: usize,   // files that need uploading
        remote_count: usize,  // URLs used as-is
    },

    // --- Upload phase (one set per file) ---

    /// About to start uploading a media file.
    MediaUploadStarted {
        file: PathBuf,
        index: usize,     // 0-based, for "uploading 1 of 5" UI
        total: usize,     // total local files to upload
        size_bytes: u64,
    },

    /// Periodic byte-level progress during upload.
    MediaUploadProgress {
        file: PathBuf,
        bytes_sent: u64,
        total_bytes: u64,
    },

    /// Upload completed; `hosted_url` is the Ghost CDN URL.
    MediaUploadComplete {
        file: PathBuf,
        hosted_url: String,
    },

    /// Upload failed (non-fatal: pipeline returns an error after all uploads attempted).
    MediaUploadFailed {
        file: PathBuf,
        reason: String,
    },

    // --- Post creation phase ---

    /// Lexical document built; about to POST to Ghost.
    PostCreating,

    /// Post created or updated successfully.
    PostCreated {
        post_id: String,
        post_url: String,
        status: String,   // "draft" | "published" | "scheduled"
    },
}
```

### Channel usage

```rust
// Caller creates the channel
let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ProgressEvent>();

// Caller drives the receiver (e.g. in a separate task for UI updates)
tokio::spawn(async move {
    while let Some(event) = rx.recv().await {
        match event {
            ProgressEvent::MediaUploadProgress { file, bytes_sent, total_bytes } => {
                let pct = bytes_sent * 100 / total_bytes;
                println!("[{}%] uploading {}", pct, file.display());
            }
            ProgressEvent::PostCreated { post_url, .. } => {
                println!("Published: {}", post_url);
            }
            _ => {}
        }
    }
});

// Caller calls the pipeline
let post = pipeline.publish(&markdown, tx).await?;
```

---

## Upload Progress Tracking

`reqwest` does not expose upload progress natively. The technique is to wrap the file
`Stream` in a custom adapter that sends progress events as each chunk is polled out.

```rust
use std::pin::Pin;
use std::task::{Context, Poll};
use std::path::PathBuf;
use bytes::Bytes;
use futures_util::Stream;
use tokio::sync::mpsc::UnboundedSender;

struct ProgressStream<S> {
    inner: S,
    sender: UnboundedSender<ProgressEvent>,
    file: PathBuf,
    total_bytes: u64,
    bytes_sent: u64,
}

impl<S> Stream for ProgressStream<S>
where
    S: Stream<Item = Result<Bytes, std::io::Error>> + Unpin,
{
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                self.bytes_sent += chunk.len() as u64;
                // fire-and-forget; ignore dropped receiver
                let _ = self.sender.send(ProgressEvent::MediaUploadProgress {
                    file: self.file.clone(),
                    bytes_sent: self.bytes_sent,
                    total_bytes: self.total_bytes,
                });
                Poll::Ready(Some(Ok(chunk)))
            }
            other => other,
        }
    }
}
```

Then wrap into a `reqwest::Body`:

```rust
use tokio::fs::File;
use tokio_util::io::ReaderStream;

let file = File::open(&path).await?;
let total_bytes = file.metadata().await?.len();
let file_stream = ReaderStream::new(file);
let progress_stream = ProgressStream {
    inner: file_stream,
    sender: tx.clone(),
    file: path.clone(),
    total_bytes,
    bytes_sent: 0,
};

let body = reqwest::Body::wrap_stream(progress_stream);

let part = reqwest::multipart::Part::stream_with_length(body, total_bytes)
    .file_name(filename.clone())
    .mime_str("image/jpeg")?;

let form = reqwest::multipart::Form::new()
    .part("file", part)
    .text("purpose", "image");
```

> `Part::stream_with_length` sends a `Content-Length` header which helps servers show
> accurate upload progress on their end too.

---

## Public API Design

```rust
// src/markdown/mod.rs

pub struct PublishPipeline {
    /// Admin client to upload images and create posts.
    admin_client: GhostAdminClient,
    /// Directory used to resolve relative media paths in the markdown.
    /// Typically the directory containing the markdown file.
    base_dir: PathBuf,
}

impl PublishPipeline {
    pub fn new(admin_client: GhostAdminClient, base_dir: impl Into<PathBuf>) -> Self;

    /// Parse, upload media, and publish the post (or draft).
    /// Status from front matter is respected.
    /// Progress events are sent to `progress` as they happen.
    pub async fn publish(
        &self,
        markdown: &str,
        progress: tokio::sync::mpsc::UnboundedSender<ProgressEvent>,
    ) -> Result<CreatedPost, GhostError>;

    /// Like `publish`, but forces `status = "draft"` regardless of front matter.
    pub async fn draft(
        &self,
        markdown: &str,
        progress: tokio::sync::mpsc::UnboundedSender<ProgressEvent>,
    ) -> Result<CreatedPost, GhostError>;

    /// Parse and upload media only — no post creation.
    /// Returns the mapping of original src strings → hosted URLs.
    /// Useful for previewing or testing uploads independently.
    pub async fn upload_media(
        &self,
        markdown: &str,
        progress: tokio::sync::mpsc::UnboundedSender<ProgressEvent>,
    ) -> Result<HashMap<String, String>, GhostError>;
}

/// Returned after successful post creation.
pub struct CreatedPost {
    pub id: String,
    pub url: String,
    pub slug: String,
    pub status: String,
    pub title: String,
}
```

### Convenience constructor for no-progress callers

For callers that don't care about progress, expose a sink helper:

```rust
/// Returns an UnboundedSender that discards all events.
pub fn no_progress() -> tokio::sync::mpsc::UnboundedSender<ProgressEvent> {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    tx  // _rx is dropped immediately; sender.send() will silently fail — that's fine
}

// Usage:
pipeline.publish(&markdown, no_progress()).await?;
```

---

## Module Structure

```
src/
  markdown/
    mod.rs           PublishPipeline struct; publish(), draft(), upload_media()
    frontmatter.rs   PostFrontMatter struct; parse_frontmatter(input) -> (PostFrontMatter, String)
    scanner.rs       scan_media(body, base_dir) -> (Vec<IrNode>, IndexMap<String, MediaRef>)
    uploader.rs      upload_all(refs, admin_client, tx) -> HashMap<String, String>
                     ProgressStream<S> wrapper
    converter.rs     to_lexical(ir, upload_map) -> LexicalDocument
    progress.rs      ProgressEvent enum; no_progress() helper
```

---

## Dependency Summary

Add these to `Cargo.toml` for the markdown pipeline:

```toml
[dependencies]
# Markdown parsing (event-based, CommonMark + GFM)
pulldown-cmark = { version = "0.13", features = [] }

# Front matter (YAML/TOML/JSON stripping + deserialization)
gray_matter = { version = "0.3", features = ["yaml"] }

# Ordered map for preserving media discovery order
indexmap = { version = "2", features = ["serde"] }

# Async file streaming to reqwest body (for upload progress)
tokio-util = { version = "0.7", features = ["io"] }
futures-util = "0.3"
bytes = "1"

# Already planned:
# reqwest = { version = "0.12", features = ["json", "multipart", "stream"] }
# tokio = { version = "1", features = ["full"] }
# serde = { version = "1", features = ["derive"] }
# serde_json = "1"
# thiserror = "1"
```

> Note: `reqwest` must have the `stream` feature enabled for `Body::wrap_stream` to be available.
