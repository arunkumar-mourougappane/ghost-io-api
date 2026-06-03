# Creating Rich Content — Posts, Pages, Tags, and Media

> Research notes for implementing rich content creation in `ghost-io-api`.
> Sources: Ghost Admin API docs, Ghost publishing docs, Koenig `kg-default-nodes` source.

---

## Table of Contents

1. [Content Format Overview](#content-format-overview)
2. [Lexical Document Structure](#lexical-document-structure)
3. [All Card (Node) Types](#all-card-node-types)
4. [Card JSON Schemas](#card-json-schemas)
5. [Image Upload (before using in posts)](#image-upload)
6. [Creating a Post or Page](#creating-a-post-or-page)
7. [Tags and Authors](#tags-and-authors)
8. [Rust Modelling Strategy](#rust-modelling-strategy)

---

## Content Format Overview

Ghost stores all post/page body content as **Lexical** — a JSON document format used by Ghost's Koenig editor.

| Format | Used for | API field |
|--------|----------|-----------|
| `lexical` | Default — structured JSON document | `lexical` |
| `html` | Fallback — Ghost converts to Lexical (lossy) | `html` (with `?source=html`) |
| `plaintext` | Read-only rendered output | returned with `?formats=plaintext` |

> When creating/updating posts via the API, you **write** either `lexical` or `html` (with `?source=html`).
> For lossless HTML, wrap it in an HTML card inside Lexical (see below).

---

## Lexical Document Structure

A Lexical document is a JSON string stored in the `lexical` field of a post. The root structure is:

```json
{
  "root": {
    "children": [ ...nodes... ],
    "direction": "ltr",
    "format": "",
    "indent": 0,
    "type": "root",
    "version": 1
  }
}
```

Each child is either an **inline node** (text, link) or a **block/card node**. Card nodes are wrapped in a `decorated-link` or directly in the root children array.

### Minimal paragraph example

```json
{
  "root": {
    "children": [
      {
        "children": [
          {
            "detail": 0,
            "format": 0,
            "mode": "normal",
            "style": "",
            "text": "Hello, world!",
            "type": "extended-text",
            "version": 1
          }
        ],
        "direction": "ltr",
        "format": "",
        "indent": 0,
        "type": "paragraph",
        "version": 1
      }
    ],
    "direction": "ltr",
    "format": "",
    "indent": 0,
    "type": "root",
    "version": 1
  }
}
```

---

## All Card (Node) Types

The following card types are defined in Ghost's Koenig editor (`kg-default-nodes`):

| Card type | `type` string | Description |
|-----------|---------------|-------------|
| Image | `image` | Single image with optional link, alt, caption, width |
| Video | `video` | Uploaded video with thumbnail, loop, dimensions |
| Audio | `audio` | Audio file with title, thumbnail |
| Gallery | `gallery` | Multiple images in a grid |
| Embed | `embed` | oEmbed/iframe embed (YouTube, Twitter, Vimeo, etc.) |
| Bookmark | `bookmark` | URL bookmark card with title, description, icon, thumbnail |
| File | `file` | Downloadable file attachment |
| HTML | `html` | Raw HTML block (lossless passthrough) |
| Markdown | `markdown` | Markdown block |
| Code block | `codeblock` | Syntax-highlighted code |
| Callout | `callout` | Highlighted callout/notice box |
| Toggle | `toggle` | Collapsible section |
| Button | `button` | CTA button |
| Header | `header` | Hero/header card with title, subtitle, background image |
| Product | `product` | Product card with image, title, description, button |
| Call to action | `call-to-action` | Signup/conversion CTA |
| Email CTA | `email-cta` | Email-only CTA (visible in newsletters only) |
| Signup | `signup` | Email signup form card |
| Paywall | `paywall` | Members-only content divider |
| Horizontal rule | `horizontalrule` | `<hr>` divider |
| Aside | `aside` | Pull-quote/aside block |
| Transistor | `transistor` | Transistor.fm podcast embed |
| At-link | `at-link` | Internal Ghost content link |

---

## Card JSON Schemas

Each card node sits in the Lexical `children` array as an object with `"type": "<card_type>"`.

### Image Card

```json
{
  "type": "image",
  "version": 1,
  "src": "https://example.com/image.jpg",
  "width": 1200,
  "height": 800,
  "alt": "A descriptive alt text",
  "title": "",
  "caption": "<em>Optional HTML caption</em>",
  "cardWidth": "regular",
  "href": "https://example.com/link"
}
```

| Field | Type | Notes |
|-------|------|-------|
| `src` | URL string | Image URL — upload first via `POST /admin/images/upload/` |
| `width` / `height` | int or null | Pixel dimensions |
| `alt` | string | Alt text |
| `title` | string | Title attribute |
| `caption` | HTML string | Rendered below the image |
| `cardWidth` | `"regular"` \| `"wide"` \| `"full"` | Layout width |
| `href` | URL string | Wrap image in a link |

---

### Video Card

```json
{
  "type": "video",
  "version": 1,
  "src": "https://example.com/video.mp4",
  "fileName": "video.mp4",
  "mimeType": "video/mp4",
  "width": 1920,
  "height": 1080,
  "duration": 125,
  "thumbnailSrc": "https://example.com/thumb.jpg",
  "customThumbnailSrc": "",
  "thumbnailWidth": 1920,
  "thumbnailHeight": 1080,
  "caption": "Optional caption",
  "cardWidth": "regular",
  "loop": false
}
```

| Field | Notes |
|-------|-------|
| `src` | Video URL — host externally or use Ghost's storage |
| `duration` | Duration in seconds (integer) |
| `thumbnailSrc` | Auto-generated thumbnail URL |
| `customThumbnailSrc` | Override thumbnail URL |
| `loop` | Boolean — loop the video |
| `cardWidth` | `"regular"` \| `"wide"` \| `"full"` |

---

### Audio Card

```json
{
  "type": "audio",
  "version": 1,
  "src": "https://example.com/episode.mp3",
  "title": "Episode 42 — The Meaning",
  "mimeType": "audio/mpeg",
  "duration": 3661,
  "thumbnailSrc": "https://example.com/cover.jpg"
}
```

---

### Gallery Card

```json
{
  "type": "gallery",
  "version": 1,
  "images": [
    {
      "src": "https://example.com/img1.jpg",
      "width": 1200,
      "height": 900,
      "alt": "",
      "caption": "",
      "title": "",
      "fileName": "img1.jpg",
      "row": 0
    },
    {
      "src": "https://example.com/img2.jpg",
      "width": 1200,
      "height": 900,
      "alt": "",
      "caption": "",
      "title": "",
      "fileName": "img2.jpg",
      "row": 0
    }
  ],
  "caption": "Gallery caption"
}
```

Upload each image first via `POST /admin/images/upload/`, then use the returned URLs.

---

### Embed Card

Use for oEmbed content: YouTube, Vimeo, Twitter/X, Instagram, Spotify, SoundCloud, Giphy, Codepen, etc.

```json
{
  "type": "embed",
  "version": 1,
  "url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
  "embedType": "video",
  "html": "<iframe ...></iframe>",
  "metadata": {
    "title": "Rick Astley — Never Gonna Give You Up",
    "description": "...",
    "author": "Rick Astley",
    "thumbnail_url": "https://i.ytimg.com/vi/dQw4w9WgXcQ/maxresdefault.jpg"
  },
  "caption": "Optional caption"
}
```

| Field | Notes |
|-------|-------|
| `url` | Source URL — Ghost's editor resolves the oEmbed automatically |
| `embedType` | Ghost-assigned type string (e.g. `"video"`, `"rich"`) |
| `html` | The raw embed HTML. Provide this if you already have it; otherwise Ghost resolves from `url` |
| `metadata` | oEmbed metadata (title, description, thumbnail_url, author) |

---

### Bookmark Card

Rich link preview card (like Slack/Notion unfurls).

```json
{
  "type": "bookmark",
  "version": 1,
  "url": "https://ghost.org",
  "metadata": {
    "icon": "https://ghost.org/favicon.ico",
    "title": "Ghost: The Creator Economy Platform",
    "description": "The world's most popular open source headless Node.js CMS",
    "author": "Ghost",
    "publisher": "Ghost",
    "thumbnail": "https://ghost.org/og-image.jpg"
  },
  "caption": ""
}
```

---

### File Card

Downloadable file attachment.

```json
{
  "type": "file",
  "version": 1,
  "src": "https://example.com/content/files/report.pdf",
  "fileName": "report.pdf",
  "fileTitle": "Annual Report 2026",
  "fileCaption": "Download the full report",
  "fileSize": 2048576
}
```

| Field | Notes |
|-------|-------|
| `src` | File URL — upload via Ghost's storage or host externally |
| `fileSize` | Size in bytes |

---

### HTML Card (lossless HTML passthrough)

```json
{
  "type": "html",
  "version": 1,
  "html": "<table><tr><td>Any HTML here</td></tr></table>"
}
```

Equivalent to wrapping content in `<!--kg-card-begin: html-->...<!--kg-card-end: html-->` in the `?source=html` mode.

---

### Code Block Card

```json
{
  "type": "codeblock",
  "version": 1,
  "code": "fn main() {\n    println!(\"Hello, world!\");\n}",
  "language": "rust",
  "caption": ""
}
```

---

### Callout Card

```json
{
  "type": "callout",
  "version": 1,
  "calloutText": "⚠️ This is an important notice.",
  "calloutEmoji": "⚠️",
  "backgroundColor": "blue"
}
```

---

### Horizontal Rule

```json
{
  "type": "horizontalrule",
  "version": 1
}
```

---

### Paywall Card

Divides content into free-preview and members-only sections.

```json
{
  "type": "paywall",
  "version": 1
}
```

Everything below this node in the Lexical document is gated behind membership.

---

## Image Upload

Before using an image in a post, upload it to Ghost's storage. The API returns the hosted URL to embed in the card.

```
POST /admin/images/upload/
Content-Type: multipart/form-data

Fields:
  file     — binary image data (WEBP, JPEG, GIF, PNG, SVG)
  purpose  — "image" | "profile_image" | "icon"  (default: "image")
  ref      — optional original filename (returned as-is in response)
```

Response:

```json
{
  "images": [
    {
      "url": "https://demo.ghost.io/content/images/2026/06/my-photo.jpg",
      "ref": "my-photo.jpg"
    }
  ]
}
```

Use the returned `url` as the `src` in image, gallery, video thumbnail, or audio thumbnail cards.

In Rust, this requires `reqwest` with `multipart` feature enabled:

```toml
reqwest = { version = "0.12", features = ["json", "multipart"] }
```

---

## Creating a Post or Page

### With Lexical content

```
POST /admin/posts/
Content-Type: application/json
Authorization: Ghost {jwt}
Accept-Version: v6.0
```

```json
{
  "posts": [
    {
      "title": "My Rich Post",
      "status": "published",
      "lexical": "{\"root\":{\"children\":[{\"type\":\"image\",\"version\":1,\"src\":\"https://example.com/img.jpg\",\"cardWidth\":\"regular\",\"alt\":\"\",\"caption\":\"\"}],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"root\",\"version\":1}}"
    }
  ]
}
```

> The `lexical` value is a **JSON-encoded string** (a string containing JSON), not an inline object.

### With HTML source (converted to Lexical by Ghost)

```
POST /admin/posts/?source=html
```

```json
{
  "posts": [
    {
      "title": "My HTML Post",
      "html": "<p>Hello</p><img src=\"https://example.com/img.jpg\" alt=\"test\">",
      "status": "published"
    }
  ]
}
```

> Conversion is **lossy**. Use HTML cards (`<!--kg-card-begin: html-->`) for lossless passthrough.

### Feature image

```json
{
  "posts": [{
    "title": "My Post",
    "feature_image": "https://example.com/hero.jpg",
    "feature_image_alt": "Hero image description",
    "feature_image_caption": "Photo by Foo Bar"
  }]
}
```

---

## Tags and Authors

### Short form (most common)

```json
{
  "posts": [{
    "title": "My Post",
    "tags": ["Technology", "Rust", "#internal-flag"],
    "authors": ["author@example.com"]
  }]
}
```

- Tags by **name** — created automatically if they don't exist
- Authors by **email address**
- Tags prefixed with `#` are **internal** (invisible to readers)
- `primary_tag` = the first tag in the array
- `primary_author` = the first author in the array

### Long form (with metadata or IDs)

```json
{
  "posts": [{
    "title": "My Post",
    "tags": [
      { "name": "Rust", "description": "The Rust programming language", "accent_color": "#f74c00" },
      { "name": "#internal" }
    ],
    "authors": [
      { "id": "5c739b7c8a59a6c8ddc164a1" }
    ]
  }]
}
```

### Tag visibility

| Tag name prefix | Visibility | Effect |
|----------------|------------|--------|
| No prefix | `public` | Generates tag archive page, appears in RSS |
| `#` prefix | `internal` | Never shown to readers, used for theme logic |

---

## Rust Modelling Strategy

### Lexical as a builder type

The `lexical` field is a `String` (JSON-encoded). In Rust, model the document as a structured type that serialises to that string:

```rust
pub struct LexicalDocument {
    pub root: LexicalRoot,
}

pub struct LexicalRoot {
    pub children: Vec<LexicalNode>,
    // direction, format, indent, version fields
}

// An enum covering all card types
pub enum LexicalNode {
    Paragraph(ParagraphNode),
    Heading(HeadingNode),
    Image(ImageCard),
    Video(VideoCard),
    Audio(AudioCard),
    Gallery(GalleryCard),
    Embed(EmbedCard),
    Bookmark(BookmarkCard),
    File(FileCard),
    Html(HtmlCard),
    CodeBlock(CodeBlockCard),
    HorizontalRule,
    Paywall,
    // ...
}
```

Serialise with `serde_json::to_string(&document)` to produce the `lexical` field value.

### Serde tagging

Use `#[serde(tag = "type", rename_all = "lowercase")]` for the enum so each variant serialises with the correct `"type"` discriminant field automatically.

### Image upload helper

The `GhostAdminClient` should expose an `images().upload(path, purpose)` method that:
1. Reads the file from disk
2. Sends a `multipart/form-data` POST to `/admin/images/upload/`
3. Returns the hosted `url` string

That URL is then passed directly into `ImageCard { src: url, .. }`.
