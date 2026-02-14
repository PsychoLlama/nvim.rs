Plan: ~/.claude/plans/iterative-stirring-puzzle.md

All phases of the highlight.c migration plan are complete. Here's a summary:

## Results

**highlight.c**: 728 → 321 lines (**-56%**, ~407 lines deleted)

### Commits
| Commit | Phase | Description |
|--------|-------|-------------|
| `24c7ac5788` | 1 | Delete dead static functions |
| `7269d7f85d` | 2 | Migrate coordination functions (clear_hl_tables, hl_invalidate_blends, highlight_use_hlstate) |
| `3cd625c1a3` | 3 | Migrate arena management (get_attr_entry, ui_send_all_hls) |
| `ceb0cbaac0` | 4 | Migrate dict2hlattrs (~155 lines of dict parsing) |
| `55ed43c36e` | 5 | Migrate ns_get_hl middle section (Lua callback orchestration) |
| `a5f168d7ca` | 7 | Final cleanup (remove orphaned declarations, unused includes, reorganize) |

### What remains in highlight.c
- `_Static_assert` checks (6 lines)
- Consolidated `extern rs_*` declarations (30 lines)
- C accessor functions for globals/enums (45 lines)
- C callback wrappers for UI dispatch, emsg, Lua bridge (55 lines)
- Thin 1-line wrappers delegating to Rust (160 lines)
- `dict2hlattrs` bridge (needs C `Dict(highlight)` type conversion)
- `ns_hl_def` branch (ns_id==0 needs C `set_hl_group`)

### All checks pass
- `just build` - clean
- `just smoke-test` - passes
- `just rust-fmt-check` - clean
- `just rust-clippy` - clean
- `just rust-test` - 4296 tests pass
