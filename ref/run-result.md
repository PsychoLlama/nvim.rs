Plan: ~/.claude/plans/optimized-drifting-puffin.md

All 8 phases of the plan are complete. Here's a summary of what was done:

**4 commits:**

1. **`facbf953`** - Core migration: Created `nvim-regexp` crate with `rs_skip_regexp` and `rs_skip_regexp_ex`, added C accessor wrappers for static helpers (`get_char_class`, `get_equi_class`, `get_coll_element`), wired `skip_regexp_ex` in regexp.c to delegate to Rust, and registered the crate in all build system touchpoints.

2. **`9e07dc85`** - Test harness: Updated `compare_regexp.c` with FFI stubs for standalone testing and 24 comparison tests validating Rust matches C behavior.

3. **`313eea1c`** - Caller updates: Changed `ex_docmd`, `search`, and `tag` crates to call `rs_skip_regexp`/`rs_skip_regexp_ex` directly. Fixed tag crate's `magic: bool` → `magic: c_int`.

4. **`f6b9e741`** - Test corpus: Added skip_regexp-focused patterns (collections with delimiters, POSIX classes, magic mode switches, edge cases) and regenerated corpus (545 entries).
