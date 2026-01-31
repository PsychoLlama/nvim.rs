Plan: ~/.claude/plans/encapsulated-puzzling-bird.md

The Phase 7 migration is complete. Here's a summary of what was accomplished:

## Summary: Phase 7 - Quickfix Directory Stack Migration

### Files Modified/Created:
1. **`src/nvim-rs/quickfix/src/dirstack.rs`** (new) - 302 lines
   - `DirStackNode` struct mirroring C's `dir_stack_T`
   - `push_dir_raw()` - Push directory onto stack with relative path resolution
   - `pop_dir_raw()` - Pop directory from stack
   - `clean_dir_stack_raw()` - Free all stack entries
   - `guess_filepath_raw()` - Search stack for directory containing a file

2. **`src/nvim-rs/quickfix/src/lib.rs`** - Updated
   - Added `pub mod dirstack` module declaration
   - Updated `rs_qf_push_dir`, `rs_qf_pop_dir`, `rs_qf_clean_dir_stack`, `rs_qf_guess_filepath` to use Rust implementations
   - Updated internal `rs_qf_free_items` to use Rust stack cleanup
   - Removed FFI declarations for deleted C functions

3. **`src/nvim/quickfix.c`** - 194 lines deleted
   - Deleted static C functions: `qf_push_dir`, `qf_pop_dir`, `qf_clean_dir_stack`, `qf_guess_filepath`
   - Deleted C wrappers: `nvim_qf_push_dir`, `nvim_qf_pop_dir`, `nvim_qf_clean_dir_stack`, `nvim_qf_guess_filepath`
   - Deleted forward declarations
   - Added extern declarations for Rust functions
   - Updated callers to use Rust `rs_qf_*` functions

### Verification:
- `just build` - ✅ Passes
- `just rust-fmt-check` - ✅ Passes
- `just rust-clippy` - ✅ Passes
- `just rust-test` - ✅ 3717 tests pass

### Impact:
- **194 lines of C deleted** from quickfix.c
- Directory stack functionality (used for `:make` with directory-changing builds) now fully implemented in Rust with proper memory safety
