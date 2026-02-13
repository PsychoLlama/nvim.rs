Plan: ~/.claude/plans/sorted-skipping-noodle.md

All steps complete. Here's the summary:

**3 commits on `rust-migration`:**

| Commit | Description |
|--------|-------------|
| `7f18af3` | Phase 1: Convert 7 C functions to thin `rs_*` wrappers; fix `gchar_cursor` circular FFI |
| `cef7648` | Phase 2: Migrate `coladvance_force` to Rust with `nvim_coladvance2_addspaces` and `nvim_coladvance_force` C accessors |
| `d2ececa` | Phase 3: Update run-result metadata |

**Key changes:**
- Fixed the `gchar_cursor` → `rs_gchar_cursor` → `nvim_gchar_cursor` → `gchar_cursor` circular FFI chain
- All 7 remaining C functions with duplicated logic converted to thin wrappers
- `coladvance_force` logic now lives in Rust as `rs_coladvance_force`
- Change crate's `nvim_coladvance_force` FFI now resolves at link time

**Validation:** `just build`, `just smoke-test`, `just rust-fmt-check`, `just rust-clippy`, `just rust-test` (4106/4106 passed) — all green.
