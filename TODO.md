# LendWise Recovery Rename & Budget Docs - Implementation TODO

## Approved Plan Steps (Breakdown)

### Phase 1: File Edits (AI will do these now)
- [ ] ✅ Create this TODO.md
- [ ] Edit Cargo.toml (package name)
- [ ] Edit README.md (title, URLs, tags, refs)
- [ ] Edit PROJECT_PROPOSAL.md (title/refs)
- [ ] Edit METHODOLOGY_AND_ARCHITECTURE.md (title)
- [ ] Edit docs/OBJECTIVES_TRACEABILITY.md (title)
- [ ] Edit fly.toml (app name)
- [ ] Edit CONTAINERIZATION.md (binary/image)
- [ ] Edit DEVELOPMENT_CHALLENGES.md (title/binary)
- [ ] Create docs/BUDGET.md

### Phase 2: Shell Commands (User runs after edits confirm)
```
git add .
git commit -m 'Rename project to LendWise Recovery + add budget docs'

# Rename directory
cd ..
git mv smart-loan-recovery lendwise-recovery
cd lendwise-recovery

# Rebuild binary (new name)
cargo build --release

# Fly.io (if keeping same app; else fly apps create lendwise-recovery)
fly deploy
fly certs create  # if needed for new domain

# Test
cargo run
# Visit new URL if changed
```

### Phase 3: Verification
- [ ] Binary runs as `lendwise-recovery`
- [ ] All docs updated
- [ ] Budget.md in docs/
- [ ] Fly app live (update DNS if custom domain)

### Phase 4: Completion
- Push to git
- Update repo name if GitHub
- Archive old backups (smart-loan-recovery-fly.dev if needed)

Track progress by checking off. Reply when Phase 1 complete to confirm before Phase 2.
