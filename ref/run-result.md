Plan: ~/.claude/plans/sparkling-tumbling-hummingbird.md

All 3 phases of textformat.c migration complete:

| Phase | Commit | Function | Lines Deleted |
|-------|--------|----------|---------------|
| 1 | `b8f0146` | fex_format() | ~30 |
| 2 | `4ff906b` | format_lines() | ~220 |
| 3 | `890c82f` | internal_format() | ~389 |

textformat.c reduced from 1,187 → 753 lines (~639 lines of C logic deleted and rewritten in Rust, ~205 lines of new accessor functions added). All checks pass: build, smoke-test, rust-fmt-check, rust-clippy, rust-test.
