# Quick Start - Issue Creation

This guide helps you quickly create all 58 GitHub issues for the ghost-io-api project.

## Prerequisites

✓ GitHub labels created (component + version labels)  
✓ Documentation reviewed (`IMPLEMENTATION_PLAN.md`, `ISSUES_GUIDE.md`)  
✓ Milestone structure understood (8 versions: v0.1.0 → v1.0.0)

## Fast Track: Create First 9 Issues (Milestone 1)

These are the foundation issues with no dependencies. Start here:

### 1. Error Types - GhostError enum
```bash
gh issue create \
  --title "Error Types - GhostError enum" \
  --label "enhancement,foundation,v0.1.0" \
  --body "See IMPLEMENTATION_PLAN.md Issue 1"
```

### 2. Pagination Models
```bash
gh issue create \
  --title "Pagination Models" \
  --label "enhancement,models,v0.1.0" \
  --body "See IMPLEMENTATION_PLAN.md Issue 2"
```

### 3. Post Model
```bash
gh issue create \
  --title "Post Model" \
  --label "enhancement,models,v0.1.0" \
  --body "See IMPLEMENTATION_PLAN.md Issue 3"
```

### 4. Content Auth - ContentApiKey
```bash
gh issue create \
  --title "Content Auth - ContentApiKey" \
  --label "enhancement,auth,v0.1.0" \
  --body "See IMPLEMENTATION_PLAN.md Issue 4"
```

### 5. Content Client - GhostContentClient
```bash
gh issue create \
  --title "Content Client - GhostContentClient" \
  --label "enhancement,client,v0.1.0" \
  --body "Depends on #1, #3, #4. See IMPLEMENTATION_PLAN.md Issue 5"
```

### 6. Browse Params Builder
```bash
gh issue create \
  --title "Browse Params Builder" \
  --label "enhancement,params,v0.1.0" \
  --body "See IMPLEMENTATION_PLAN.md Issue 6"
```

### 7. Basic Models
```bash
gh issue create \
  --title "Basic Models (Page, Tag, Author, Settings)" \
  --label "enhancement,models,v0.1.0" \
  --body "See IMPLEMENTATION_PLAN.md Issue 7"
```

### 8. Extended Content Client
```bash
gh issue create \
  --title "Extended Content Client" \
  --label "enhancement,client,v0.1.0" \
  --body "Depends on #5, #7. See IMPLEMENTATION_PLAN.md Issue 8"
```

### 9. list-posts Example
```bash
gh issue create \
  --title "Example: list-posts" \
  --label "example,v0.1.0" \
  --body "Depends on #5, #6. See IMPLEMENTATION_PLAN.md Issue 9"
```

## Recommended Workflow

1. **Week 1:** Create issues #1-9 (Milestone 1)
2. **Week 2:** Create issues #10-16 (Milestone 2)
3. **Week 3:** Create issues #17-22 (Milestone 3)
4. Continue for remaining milestones

## Bulk Creation Script

For power users, create a script:

```bash
#!/bin/bash
# create-milestone-1-issues.sh

ISSUES=(
  "Error Types - GhostError enum|enhancement,foundation,v0.1.0"
  "Pagination Models|enhancement,models,v0.1.0"
  "Post Model|enhancement,models,v0.1.0"
  # ... add all issues
)

for issue in "${ISSUES[@]}"; do
  IFS='|' read -r title labels <<< "$issue"
  gh issue create --title "$title" --label "$labels" \
    --body "See IMPLEMENTATION_PLAN.md for details"
  sleep 1  # Rate limiting
done
```

## Tracking Progress

After creating issues:

1. **Set up Project Board**
   - Create columns: Backlog, In Progress, Review, Done
   - Organize issues by milestone

2. **Monitor Dependencies**
   - Use issue references: "Depends on #N"
   - GitHub will auto-link issues

3. **Track Completion**
   ```bash
   # Check milestone progress
   gh issue list --milestone "v0.1.0"
   ```

## Reference Files

- **Full Templates:** `docs/IMPLEMENTATION_PLAN.md`
- **Creation Guide:** `docs/ISSUES_GUIDE.md`
- **Milestone Details:** `docs/milestone-report.md`
- **Version Info:** `docs/version-roadmap.md`
- **CSV Export:** `docs/issues.csv`

## Success Metrics

- ✓ All 58 issues created
- ✓ Labels applied correctly
- ✓ Dependencies documented
- ✓ Modules identified
- ✓ Acceptance criteria listed

## Support

For questions or clarifications, refer to:
- Research docs in `docs/design/`
- API references in `docs/api-reference/`
- Feature specs in `docs/features/`

---

**Ready to start?** Begin with Milestone 1, Issues #1-9!
