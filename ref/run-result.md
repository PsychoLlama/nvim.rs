Plan: ~/.claude/plans/glowing-soaring-cookie.md

All plan steps are complete. Here's a summary:

## Migration Complete: `arglist.c` → Rust

**Commits:** 10 phased commits (`a5798b6d98` through `182d3bc019`)

### What was done:
- **41 functions** migrated from `arglist.c` to the `nvim-arglist` Rust crate across 10 phases
- C file now contains only thin `rs_*` wrappers + C accessor functions
- ~100+ C accessor functions (`nvim_al_*`) for struct fields and globals
- `_Static_assert` guards for all hardcoded constants
- Workspace + justfile registration fixed

### Verification:
- `just build` — passes
- `just smoke-test` — passes (headless start + 29/29 regexp tests)
- `just rust-fmt-check` — clean
- `just rust-clippy` — clean
- `just rust-test` — 3739 tests pass
