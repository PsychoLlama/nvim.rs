Plan: ~/.claude/plans/playful-swimming-twilight.md

All plan steps are complete. Here's a summary:

## Migration Complete: `map.c` → Rust

**Commit:** `f719e40462` on `rust-migration`

### What was migrated
- **93 exported functions** from `map.c` into `src/nvim-rs/collections/src/map.rs`
  - 45 key-type ops (9 types × 5 functions: find_bucket, get, rehash, put, delete)
  - 45 map-value ops (15 pairs × 3 functions: ref, put_ref, del)
  - 3 explicit: `mh_realloc`, `mh_clear`, `pmap_del2`
- C files (`map.c`, `map_key_impl.c.h`, `map_value_impl.c.h`) are now thin `rs_` wrappers
- Public API (`map_defs.h` macros) unchanged — no caller changes needed

### Verification
- `just build` — pass
- `just smoke-test` — pass (headless + 29/29 regexp)
- `just rust-fmt-check` — clean
- `just rust-clippy` — clean
- `just rust-test` — 3538 tests pass
- Cross-language `_Static_assert` checks for `MapHash` and `ColorKey` struct sizes
