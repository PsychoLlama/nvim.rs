Plan: ~/.claude/plans/sparkling-tumbling-hummingbird.md

All 4 phases complete. Here's the summary:

## Migration Complete

**Plan:** `~/.claude/plans/sparkling-tumbling-hummingbird.md`

### Commits
| Phase | Commit | Function | C Lines Deleted |
|-------|--------|----------|-----------------|
| 1 | `b8f0146` | `fex_format()` | ~30 |
| 2 | `4ff906b` | `format_lines()` | ~220 |
| 3 | `890c82f` | `internal_format()` | ~389 |
| 4 | `be6efd9` | run-result update | — |

### Results
- **textformat.c**: 1,187 → 753 lines (639 lines of C logic deleted, 205 lines of new accessor functions added)
- **3 new Rust modules** created: `fex.rs`, `format_lines.rs`, `internal.rs`
- **Intra-crate direct calls**: `ops.rs` and `auto.rs` now call `format_lines_impl()` and `fex_format_impl()` directly in Rust, eliminating C round-trips
- **All checks pass**: `just build`, `just smoke-test`, `just rust-fmt-check`, `just rust-clippy`, `just rust-test` (4155 tests)
