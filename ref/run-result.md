Plan: ~/.claude/plans/tranquil-noodling-pelican.md

All 7 phases of the profile migration plan are now complete. Here's a summary:

**Commits created (Phases 1-7):**

| Phase | Commit | Functions |
|-------|--------|-----------|
| 1 | `68f7cf118c` | timing wrappers + types foundation (6 fns) |
| 2 | `e0512a522c` | input/wait profiling (3 fns) |
| 3 | `70b6e65de3` | function line profiling (4 fns) |
| 4 | `9a305a73f4` | script line profiling (5 fns) |
| 5 | `998a7742e3` | child profiling (2 fns) |
| 6 | `15fbc5750b` | startup timing (7 fns) |
| 7 | `6300ad318e` | command handling, state mgmt, dump (5 fns + accessors) |

**What moved to Rust:**
- All profiling statics: `prof_wait_time`, `wait_time`, `g_start_time`, `g_prev_time`, `profile_fname`, `pause_time`, `pexpand_what`, `pexpand_cmds`
- ~27 `rs_*` functions across 7 Rust modules (`timing`, `input`, `func_line`, `script_line`, `child`, `startup`, `command`)

**What remains in C:**
- `profile_init` (directly manipulates `scriptitem_T` fields via `ga_init`)
- `func_dump_profile`, `script_dump_profile`, `prof_func_line`, `prof_sort_list`, `prof_total_cmp`, `prof_self_cmp` (deep hashtable/garray iteration + file I/O)
- All C accessor functions (thin wrappers exposing struct fields to Rust)

All checks pass: `build`, `smoke-test`, `rust-fmt-check`, `rust-clippy`, `rust-test` (4102 tests).
