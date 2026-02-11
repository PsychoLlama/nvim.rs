Plan: ~/.claude/plans/spicy-leaping-perlis.md

All 5 phases of the syntax.c migration plan are complete:

| Phase | Functions | Status |
|-------|-----------|--------|
| 1 | `find_endpos` + helpers (~530 lines) | Committed |
| 2 | `syn_current_attr` + `syn_finish_line` (~500 lines) | Committed |
| 3 | `syn_sync` (~222 lines) | Committed |
| 4 | `get_syn_options` + `get_id_list` (~290 lines) | Committed |
| 5 | `syn_cmd_region` + `get_syn_pattern` (~300 lines) | Committed |

**Summary of commits:**
- 5 commits on `rust-migration` branch
- ~1,842 C lines migrated to Rust across 5 new/modified modules (`region.rs`, `current_attr.rs`, `sync.rs`, `opt_parse.rs`, `cmd_region.rs`)
- All checks pass: build, smoke test (29 tests), clippy, fmt, unit tests (4075 tests)
- C functions replaced with thin wrappers calling `rs_*` Rust implementations
- Complex data structure operations (synpat_T field writes, garray management) encapsulated in high-level C wrappers
