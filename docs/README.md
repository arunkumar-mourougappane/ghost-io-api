# ghost-io-api — Research & Design Documentation

Pre-implementation research for a fully-typed, async Rust client for the Ghost CMS API.
All documents below were written before any Rust code was committed.

---

## api-reference/

Official Ghost API behaviour — sourced directly from Ghost's developer docs.

| Document | Description |
|----------|-------------|
| [ghost-api-overview.md](api-reference/ghost-api-overview.md) | Both APIs at a glance: base URLs, authentication methods, all endpoints, query parameters (filter/limit/page/order/include/fields/formats), response envelope, pagination, and JWT generation examples in Bash, Node, and Python. |
| [admin-api.md](api-reference/admin-api.md) | Deep Admin API reference: full post/page object schema, CRUD for posts/pages/tags/members, publish/schedule/email flows, collision detection (`updated_at` requirement on PUT), member schema with subscriptions and newsletters. |

---

## design/

Architecture decisions and security design for the `ghost-io-api` crate itself.

| Document | Description |
|----------|-------------|
| [rust-client-architecture.md](design/rust-client-architecture.md) | Overall crate design: two client types (`GhostContentClient` / `GhostAdminClient`), recommended dependencies, module structure, JWT generation (no external JWT crate), typed BrowseParams builder, error enum, and implementation order. |
| [credential-storage.md](design/credential-storage.md) | Cross-platform encrypted credential storage: why not OS keychains, XChaCha20-Poly1305 AEAD + Argon2id key derivation, binary file format (`[16B salt][24B nonce][ciphertext]`), platform config paths via `dirs`, and the `secrecy`/`zeroize` in-memory safety strategy. |

---

## features/

Deep-dives into specific feature areas with Rust implementation notes.

| Document | Description |
|----------|-------------|
| [rich-content-creation.md](features/rich-content-creation.md) | Ghost's Lexical content format: all 23 card types with full JSON schemas (image, video, audio, gallery, embed, bookmark, file, HTML, codeblock, callout, paywall, …), image upload API, tags/authors syntax, and how to model Lexical as a Rust enum with `serde`. |
| [markdown-publish-pipeline.md](features/markdown-publish-pipeline.md) | End-to-end Markdown → Ghost pipeline: YAML front matter format, crate selection (`pulldown-cmark`, `gray_matter`, `indexmap`, `tokio-util`), 5 pipeline stages, typed `ProgressEvent` enum, upload progress streaming via `ProgressStream<S>`, and the public `PublishPipeline` API. |

---

## Implementation Roadmap

Work should proceed roughly in this order, cross-referencing the documents above:

1. **`Cargo.toml`** — add all planned dependencies (see `design/rust-client-architecture.md` + `features/markdown-publish-pipeline.md`)
2. **`src/error.rs`** — `GhostError` enum with `thiserror`
3. **`src/models/pagination.rs`** — `Meta`, `Pagination`
4. **`src/auth/`** — `ContentApiKey`, `AdminApiKey`, JWT generation
5. **`src/models/`** — `Post`, `Page`, `Tag`, `Author`, `Member`, `Settings`
6. **`src/client/content.rs`** — `GhostContentClient`, read-only endpoints
7. **`src/params/browse.rs`** — `BrowseParams` builder
8. **`src/client/admin.rs`** — `GhostAdminClient`, full CRUD + image upload
9. **`src/credentials/`** — encrypted file storage (`crypto.rs`, `store.rs`)
10. **`src/markdown/`** — `PublishPipeline` with front matter → Lexical → post creation
