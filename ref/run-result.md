Plan: ~/.claude/plans/glimmering-hatching-waterfall.md

All steps complete. Here's the summary:

## Regexp Utility Functions Migration — Batch 2 Complete

**7 commits**, all passing `build`, `smoke-test`, `rust-fmt-check`, `rust-clippy`, and `rust-test`:

| Commit | Functions | Tests Added |
|--------|-----------|-------------|
| `f54114d` | Delete dead C `mb_decompose`, migrate `get_cpo_flags` | 4 (mb_decompose) |
| `29ce130` | `make_extmatch`, `ref_extmatch`, `unref_extmatch` | 3 (layout) |
| `f73818b` | `re_mult_next` | 2 (logic) |
| `bec9176` | `cleanup_subexpr`, `cleanup_zsubexpr` | 0 (orchestration) |
| `5f9d470` | `reg_prev_class` | 0 (FFI only) |
| `e033b72` | `reg_nextline` | 0 (FFI only) |
| `6ff569b` | `skip_regexp_err` | 0 (FFI only) |

**Totals:** 11 functions migrated + 1 dead C function deleted, 32 `#[no_mangle]` functions in regexp crate (up from 20), 3472 unit tests passing (up from 3467), ~25 new C accessor/helper functions.

Also fixed a symbol conflict: removed duplicate `rs_ref_extmatch`/`rs_unref_extmatch` wrappers from the syntax crate that conflicted with the new regexp crate implementations.
