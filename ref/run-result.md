Plan: ~/.claude/plans/zazzy-orbiting-flurry.md

All four phases of the plan at `/home/overlord/.claude/plans/zazzy-orbiting-flurry.md` are now complete:

**Phase 1** - `win_split_ins` (516 lines) - committed as `0e61261945`
**Phase 2** - `win_close_othertab` (128 lines) - committed as `04b8f790ff`
**Phase 3** - `win_close` (269 lines) - committed as `996b3f4a72`
**Phase 4** - `do_window` (535 lines) - committed as `f4d112b720`

**Summary of Phase 4:**
- Created `dispatch.rs` with `rs_do_window()` — the Rust dispatcher for all CTRL-W commands
- The C `do_window()` body was replaced with a single call to `rs_do_window(nchar, Prenum, xchar)`
- Complex cases (file goto, tag preview, 'g' sub-switch, window traversal, alternate buffer) remain as C helper functions called from Rust
- Simple cases (navigation, resize, rotate, exchange, close/quit) are dispatched directly to existing C functions
- Fixed incorrect WSP_* flag values in `commands.rs` (WSP_VERT was 0x01, corrected to 0x02)
- Added `_Static_assert` for key code constants (K_UP, K_DOWN, etc.)
- All 5 validation checks pass: build, smoke-test (29 tests), rust-fmt, clippy, rust-test (4058 tests)
