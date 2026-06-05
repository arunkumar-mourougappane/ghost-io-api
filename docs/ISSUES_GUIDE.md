# GitHub Issues Guide for ghost-io-api

## Overview

This guide provides templates and instructions for creating all 58 GitHub issues for the project.

## Quick Start

### Option 1: Use the CSV File
Import `docs/issues.csv` into your GitHub project management tool or use it as reference for manual creation.

### Option 2: Follow the Implementation Plan
Use `docs/IMPLEMENTATION_PLAN.md` which provides detailed issue templates for each feature.

### Option 3: Use the SQL Database
Query the session database for complete feature/dependency information:

```bash
sqlite3 ~/.copilot/session-state/*/session.db
SELECT * FROM features;
SELECT * FROM feature_deps;
```

## Issue Naming Convention

```
[Component] Feature Name - Brief Description

Examples:
- Error Types - GhostError enum
- Content Client - GhostContentClient
- Example: list-posts
```

## Label Strategy

### By Component
- `foundation` - Core types and error handling
- `models` - Data model structs
- `client` - API client implementations
- `auth` - Authentication (Content API key, JWT)
- `params` - Query parameter builders
- `example` - Example programs
- `media` - Media upload functionality
- `markdown` - Markdown processing
- `security` - Encryption and credentials
- `lexical` - Lexical content format
- `export` - Export/backup functionality
- `cli` - CLI tool components

### By Version
- `v0.1.0` - Foundation & Content API (alpha)
- `v0.2.0` - Admin API - Core CRUD (alpha)
- `v0.3.0` - Media Upload & Images (alpha)
- `v0.4.0` - Markdown Pipeline (beta)
- `v0.5.0` - Credential Security (beta)
- `v0.6.0` - Rich Content & Tags (beta)
- `v0.7.0` - Advanced Features (RC)
- `v1.0.0` - CLI Tool (stable)

## Issue Template Structure

Each issue should include:

### 1. Title
Clear, descriptive title following naming convention

### 2. Description
Brief overview of what needs to be implemented

### 3. Tasks Checklist
```markdown
- [ ] Task 1
- [ ] Task 2
- [ ] Add tests
- [ ] Add documentation
```

### 4. Module
Path to the file: `src/client/content.rs`

### 5. Dependencies
List of issue numbers this depends on:
```markdown
**Depends on:**
- #1 (Error Types)
- #3 (Post Model)
```

### 6. Example Usage
```rust
// Code example showing how to use the feature
```

### 7. References
Links to relevant documentation:
- `docs/design/rust-client-architecture.md`
- `docs/api-reference/ghost-api-overview.md`

### 8. Acceptance Criteria
```markdown
**Acceptance Criteria:**
- [ ] All public APIs documented
- [ ] Tests pass with >80% coverage
- [ ] Clippy warnings resolved
- [ ] Integration test included (where applicable)
```

## Sample Issue: Error Types

```markdown
### Error Types - GhostError enum

**Labels:** `enhancement`, `foundation`, `v0.1.0`  
**Milestone:** v0.1.0 - Foundation & Content API

#### Description
Implement the core error handling type for the crate using `thiserror`.

#### Tasks
- [ ] Create `GhostError` enum with variants: `Api`, `Http`, `Json`, `Auth`
- [ ] Use `#[error(...)]` attributes for descriptive error messages
- [ ] Implement `From` traits for `reqwest::Error` and `serde_json::Error`
- [ ] Add unit tests for error conversions
- [ ] Add documentation comments with examples

#### Module
`src/error.rs`

#### Example Usage
```rust
pub type Result<T> = std::result::Result<T, GhostError>;

#[derive(thiserror::Error, Debug)]
pub enum GhostError {
    #[error("Ghost API error ({error_type}): {message}")]
    Api {
        message: String,
        error_type: String,
        context: Option<String>,
    },
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("JWT signing error: {0}")]
    Auth(String),
}
```

#### References
- `docs/design/rust-client-architecture.md` (Error Handling section)

#### Acceptance Criteria
- [ ] All error variants defined and documented
- [ ] Error messages are descriptive
- [ ] Tests for error conversions pass
- [ ] Documentation includes usage examples
```

## Dependency Graph

### Milestone 1 (v0.1.0)
```
#1 (Error Types) ──┐
                   ├──→ #5 (Content Client) ──→ #8 (Extended Client)
#3 (Post Model) ───┤                            ↓
                   │                          #9 (list-posts Example)
#4 (Content Auth) ─┘                            ↑
                                                 │
#6 (Browse Params) ─────────────────────────────┘

#2 (Pagination Models) ──→ (used by all browse methods)
#7 (Basic Models) ──────→ #8 (Extended Client)
```

### Milestone 2 (v0.2.0)
```
#10 (JWT Generation) ──→ #11 (Admin Client Core) ──→ #14 (Admin Posts CRUD)
                                                       ↑
#12 (PostCreate) ──────────────────────────────────┤
#13 (PostUpdate) ──────────────────────────────────┘
                                                       ↓
                                                  #16 (publish-post Example)

#15 (Response Envelope) ──→ (used by all Admin API methods)
```

### Milestone 3 (v0.3.0)
```
#11 (Admin Client) ──→ #17 (Image Upload) ──→ #22 (upload-image Example)
                                   ↓
                            #18 (Multipart)
                            #19 (Upload Response)
                                   ↓
#20 (Progress Events) ──→ #21 (Progress Stream)
```

## Creating Issues in Bulk

### Method 1: GitHub Web UI
1. Go to Issues tab
2. Click "New Issue"
3. Copy/paste from `IMPLEMENTATION_PLAN.md`
4. Set labels and milestone
5. Submit

### Method 2: GitHub CLI (if milestones were supported)
```bash
gh issue create \
  --title "Error Types - GhostError enum" \
  --body-file issue-template.md \
  --label "enhancement,foundation,v0.1.0"
```

### Method 3: GitHub API
Use the REST API with a script to create all issues programmatically.

## Issue Progression

For each issue:

1. **Create** - Add issue with proper metadata
2. **Assign** - Assign to developer
3. **Branch** - Create feature branch: `git checkout -b feature/issue-N-name`
4. **Implement** - Write code with tests
5. **Format** - Run `cargo fmt` and `cargo clippy`
6. **PR** - Create pull request: "Closes #N"
7. **Review** - Code review + CI checks
8. **Merge** - Merge to main after approval
9. **Close** - Issue auto-closes on PR merge

## Cross-Milestone Dependencies

Some features depend on features from earlier milestones:

- **M3 → M2:** Image Upload (#17) depends on Admin Client (#11)
- **M4 → M3:** Batch Media Upload (#24) depends on Image Upload (#17)
- **M4 → M4:** Publish Pipeline (#27) depends on Front Matter, Media Upload, Lexical Conversion
- **M6 → M4:** Card JSON Schemas (#38) depends on Lexical Base Structures (#25)
- **M7 → M4:** bulk-publish (#45) depends on Publish Pipeline (#27)
- **M8 → M1-7:** CLI subcommands depend on corresponding library features

## Version Release Checklist

Before releasing each version:

- [ ] All milestone issues closed
- [ ] All tests passing
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Git tag created: `git tag -s v0.1.0 -m "Release v0.1.0"`
- [ ] Published to crates.io: `cargo publish`
- [ ] GitHub Release created with notes

## Statistics

- **Total Issues:** 58
- **Milestones:** 8
- **Dependencies:** 35 tracked relationships
- **Estimated Time:** 19 weeks (based on 3-week milestone cycles)

## Files Generated

1. **IMPLEMENTATION_PLAN.md** - Detailed issue templates (first 2 milestones)
2. **issues.csv** - CSV format for bulk import (first 3 milestones)
3. **milestone-report.md** - Complete milestone breakdown
4. **version-roadmap.md** - Version release details
5. **ISSUES_GUIDE.md** - This file

## Next Steps

1. Review all documentation files in `docs/`
2. Create GitHub issues using templates
3. Set up project board with milestones
4. Begin implementation with Milestone 1
5. Follow the dependency graph for proper order

---

*For questions or clarifications, refer to the research documentation in `docs/` directory.*
