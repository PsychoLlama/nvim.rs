Plan: ~/.claude/plans/compressed-puzzling-dream.md

The migration is complete. Here's a summary of what was accomplished:

## Summary

Successfully migrated the core logging logic from `log.c` to Rust in the `nvim-log` crate:

### Changes Made

1. **Added C Accessor Functions** (`src/nvim/log.c`):
   - `nvim_log_get_file_path()` / `nvim_log_set_file_path()` - log file path access
   - `nvim_log_is_initialized()` / `nvim_log_set_initialized()` - initialization state
   - `nvim_log_get_ui_client_channel_id()` - UI client detection
   - `nvim_log_get_servername()` / `nvim_log_get_parent_nvim()` - instance naming
   - `nvim_log_get_localtime()` / `nvim_log_get_millis()` - timestamp access
   - Various utility functions for path/env operations

2. **Created Rust Core Module** (`src/nvim-rs/log/src/core.rs`):
   - `rs_log_path_init()` - log file path initialization (replaces `log_path_init()`)
   - `rs_do_log_to_file()` - log message formatting and output (replaces `v_do_log_to_file()`)
   - Instance name generation with caching
   - Thread-local state for recursion detection

3. **Updated C Wrappers** (`src/nvim/log.c`):
   - `log_init()` now calls `rs_log_path_init()`
   - `logmsg()` formats the message then calls `rs_do_log_to_file()`
   - `do_log_to_file()` calls `rs_do_log_to_file()`
   - Removed the now-unused `v_do_log_to_file()` function

4. **Kept in C**:
   - Backtrace functionality (`log_callstack`, `log_callstack_to_file`) - depends on libuv/execinfo
   - `log_uv_handles()` - prints libuv handles directly
   - Variadic formatting (handled via `vsnprintf` before calling Rust)
   - Mutex locking (`log_lock`/`log_unlock`)

### Verification
- `just build` ✓
- `just rust-test` ✓ (3717 tests passed)
- `just rust-fmt-check` ✓
- `just rust-clippy` ✓
