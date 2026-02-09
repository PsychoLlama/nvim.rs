Plan: ~/.claude/plans/elegant-wiggling-wombat.md

All 8 phases of the `ex_session.c` migration are complete. Here's the final summary:

**Migration Complete: `src/nvim/ex_session.c` → Rust**

All 21 C functions from `ex_session.c` have been ported to Rust across 8 phases:

| Phase | Functions | Commit |
|-------|-----------|--------|
| 1 | `put_eol`, `put_line` + infrastructure | previous session |
| 2 | `ses_do_win`, `ses_do_frame`, `ses_skipframe` | previous session |
| 3 | `ses_get_fname`, `ses_escape_fname`, `ses_put_fname`, `ses_fname` | previous session |
| 4 | `put_view_curpos`, `ses_winsizes`, `ses_arglist`, `ses_win_rec` | previous session |
| 5 | `store_session_globals`, `get_view_file` | previous session |
| 6 | `put_view` | previous session |
| 7 | `makeopens` | `c524cfff8b` |
| 8 | `ex_mkrc`, `ex_loadview` | `01fad0abab` |

**Verification:**
- `just build` — links successfully
- `just smoke-test` — nvim starts, regexp smoke passes
- `just rust-fmt-check` — clean
- `just rust-clippy` — clean
- `just rust-test` — 3717 tests pass
- Manual: `:mksession`, `:mkview`, `:mkexrc`, `:mkvimrc`, `:loadview` all produce valid output

**Totals:**
- ~80+ C accessor functions (`nvim_ses_*`)
- 21 Rust `#[no_mangle]` exports (`rs_*`) for ex_session.c functions
- ~28 `_Static_assert` guards
- No C function bodies remain beyond accessors and thin wrappers
