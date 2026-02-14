Plan: ~/.claude/plans/misty-exploring-feather.md

All 8 phases of the fold.c migration plan are complete. Here's a summary:

**Commits (6 total):**
1. `a9dcfd12` - Phase 1: foldMoveRange + truncate_fold (~104 C lines)
2. `a403074c` - Phase 2: foldMarkAdjustRecurse thin wrapper (~83 C lines)
3. `2b18fd6e` - Phase 3: foldLevel + checkupdate + fold_info + lineFolded + foldCheckClose (~80 C lines)
4. `17deacb4` - Phase 4a: expr/syntax level getter accessors (prep)
5. `fc14d15c` - Phase 4b-4d: Unified IEMS for all fold methods (marker/expr/syntax support + dispatcher)
6. `dc3bf6bb` - Phase 5: Dead code cleanup (644 lines deleted)

**Results:**
- `fold.c` went from ~3,216 lines to **2,436 lines** (~780 lines deleted across all phases)
- All fold methods (indent, diff, marker, expr, syntax) now route through the unified Rust IEMS implementation
- All checks pass: `build`, `smoke-test`, `rust-fmt-check`, `rust-clippy`, `rust-test` (4155 tests)
