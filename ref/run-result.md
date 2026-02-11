Plan: ~/.claude/plans/delegated-juggling-gray.md

All 4 phases of the search.c migration plan are complete and committed:

| Phase | Function(s) | Commit |
|-------|-------------|--------|
| 1 | `searchit()` + `first_submatch()` | `39e55ea` |
| 2 | `do_search()` | `7d5c625` |
| 3 | `findmatchlimit()` + `check_prevcol` + `find_rawstring_end` + `find_mps_values` | `02019fc` |
| 4 | `find_pattern_in_path()` + batch helpers | `cc7c3af` |

All checks pass:
- **Build**: clean
- **Smoke test**: passes (regexp smoke: 29/29)
- **Format**: clean
- **Clippy**: clean
- **Unit tests**: 4083/4083 passing
