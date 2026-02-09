Plan: ~/.claude/plans/polymorphic-humming-metcalfe.md

All steps are complete. Here's a summary:

## Migration Complete: `memfile.c` → Rust

**What was done:**
- Migrated all 25 functions (17 public + 8 static) from `src/nvim/memfile.c` to the `nvim-memfile` Rust crate
- C file now contains only thin wrappers + accessor functions (~660 lines of wrappers vs 787 lines of logic before)
- Rust crate grew from 370 lines of utility types to 1194 lines with full function implementations

**Key decisions:**
- `mf_close_file` and `mf_release_all` use a split approach: C keeps `buf_T`/memline iteration, Rust handles per-memfile logic (avoids circular FFI)
- Map iteration uses index-based `nvim_mf_hash_size`/`nvim_mf_hash_value_at` instead of `map_foreach_value` macro
- Error messages are `c"..."` CStr literals in Rust; C wrappers apply `_()` for localization

**Verification (all pass):**
- `just build` — links successfully
- `just smoke-test` — nvim starts, regexp smoke tests pass
- `just rust-fmt-check` — clean
- `just rust-clippy` — clean
- `just rust-test` — 3577 tests pass (including memfile tests)
