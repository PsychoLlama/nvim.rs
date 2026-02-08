Plan: ~/.claude/plans/lucky-splashing-music.md

All 3 phases are complete. Here's a summary:

## Summary

Successfully migrated 6 regexp utility functions from C to Rust in 3 commits:

### Commit 1: Pure arithmetic functions
- `rs_no_magic` — strip Magic encoding from a character
- `rs_toggle_magic` — toggle Magic encoding on/off  
- `rs_re_multi_type` — classify multi-match operators
- `rs_backslash_trans` — translate backslash escape sequences

### Commit 2: Class table initialization
- `rs_init_class_tab` — compile-time const 256-entry class lookup table (replacing runtime initialization)

### Commit 3: Opaque handle function
- `rs_re_multiline` — check if regexp can match newlines (uses C accessor `nvim_regexp_get_regflags` for opaque struct access)

### Test results
- 3451 Rust unit tests pass (17 new tests for the regexp crate)
- 57 FFI comparison tests pass (9 new C-vs-Rust comparisons)
- All build, smoke-test, formatting, and lint checks pass
