Plan: ~/.claude/plans/purrfect-dancing-hinton.md

All 4 phases of the getchar.c migration plan are complete. Here's a summary:

**Phase 1** (previously done): 7 self-contained helpers migrated to Rust
**Phase 2** (previously done): Buffer ownership transferred from C to Rust statics
**Phase 3** (this session): Redo + stuff operations migrated
- `AppendToRedobuffLit`, `AppendToRedobuffSpec`, `stuffReadbuffSpec`, `stuffescaped` → Rust
- `read_redo`, `copy_redo`, `start_redo`, `start_redo_ins` → Rust
- Added `nvim_set_visual_from_cursor` C accessor
- Fixed `termcap2key`/`key2termcap0`/`key2termcap1` formulas to match C macros

**Phase 4** (this session): Recording/gotchars migrated
- `gotchars`, `ungetchars`, `gotchars_ignore` → Rust
- `GotcharsState::add_byte()` method added matching C `gotchars_add_byte`
- Added C accessors: `nvim_on_key_buf_process`, `nvim_call_updatescript`, `nvim_set_debug_did_msg`
- Added `nvim-getchar` to justfile test list

**Results:**
- `getchar.c`: 3841 → 3310 lines (~14% reduction)
- Tests: 3942 → 3977 (35 new tests from getchar crate)
- All checks green: build, smoke-test, fmt, clippy, 3977 tests passing
