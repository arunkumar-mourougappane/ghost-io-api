# ghost-io-api

A strongly-typed, async Rust client for the [Ghost CMS](https://ghost.org/) API.

> ⚠️ **Early development** — the public API is not yet stable.

## Overview

`ghost-io-api` provides an ergonomic Rust interface to the Ghost Content API and Admin API. It is designed to be:

- **Async-first** — built on top of `tokio` and `reqwest`
- **Strongly typed** — all request/response types are modelled as Rust structs
- **Easy to use** — minimal boilerplate to get posts, pages, tags, authors, and more

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
ghost-io-api = "0.1"
```

## Usage

> Full usage examples will be added as the API stabilises.

```rust
// Placeholder — implementation in progress
fn main() {
    println!("ghost-io-api");
}
```

## Features

Planned support for:

| API | Status |
|-----|--------|
| Content API — Posts | 🚧 In progress |
| Content API — Pages | 🚧 In progress |
| Content API — Tags | 🚧 In progress |
| Content API — Authors | 🚧 In progress |
| Admin API — Posts (create/update/delete) | 🔜 Planned |
| Admin API — Members | 🔜 Planned |

## License

MIT © 2026 [Arunkumar Mourougappane](https://github.com/arunkumar-mourougappane)
