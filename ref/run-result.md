Plan: ~/.claude/plans/prancy-frolicking-waffle.md

All 5 phases of the plan are complete. Here's a summary:

## Completed: option.c Rust Migration Plan

### Phase 1 — Wire 6 already-correct callbacks (`0eb7ab162c`)
Wired `did_set_hlsearch`, `did_set_ignorecase`, `did_set_title_icon`, `did_set_iminsert`, `did_set_eof_eol_fixeol_bomb`, and `did_set_title`. Fixed all `UpdateType` enum values that were wrong and added `_Static_assert` guards.

### Phase 2 — Complete and wire 10 stub callbacks (`bd99f380f8`)
Completed `langnoremap/langremap` (toggle paired option), `foldlevel` (call `newFoldLevel`), `showtabline`, `modifiable`, `pumblend` (full pum_grid blending logic), `textwidth` (FOR_ALL_TAB_WINDOWS iteration), `smoothscroll`, `winblend` (clamp + check_blending), `updatecount`.

### Phase 3 — Wire 10 argument-taking callbacks (`5df91b154b`)
Wired `binary`, `diff`, `equalalways`, `foldminlines`, `foldnestmax`, `helpheight`, `swapfile`, `numberwidth`, `number_relativenumber`, `titlelen` — extracting `win_T*`/`buf_T*`/old_value from `optset_T` in C thin wrappers.

### Phase 4 — Fix 5 incorrect implementations (`2ac6922aeb`)
Fixed `lisp` (was setting modeline, should call `buf_init_chartab`), `wildchar` (wrong validation), `window` (missing lower-bound clamp), `scrollbind` (wrong arg + missing `w_scbind_pos`), `autochdir` (had incorrect guard).

### Phase 5 — Migrate self-contained helpers (`28022e427e`)
Wired `optval_free`, `optval_copy`, `optval_equal` to Rust. Fixed a critical bug: Rust's `extern static` for a C `char[]` array reads the array contents as a pointer instead of its address, causing `free(): invalid pointer` crashes. Used the existing `nvim_get_empty_string_option()` accessor instead.

**Total: ~31 C functions converted to thin wrappers calling Rust.** All checks pass: build, smoke-test, fmt, clippy, 4028 unit tests.
