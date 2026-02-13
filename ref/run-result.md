Plan: ~/.claude/plans/dapper-scribbling-planet.md

All 8 phases of the `mapping.c` Rust migration are complete and committed. Here's the summary:

**Phase 8** (just committed as `47ac10b9e0`):
- `rs_get_maptype` — determines maptype (MAP/UNMAP/NOREMAP) from ex command character
- `rs_find_matching_abbr` — searches buffer-local then global abbreviation lists for matching abbreviations, handling K_SPECIAL unescaping

**Full migration across all phases — 40 `rs_` functions in the mapping crate:**
- Phase 1: langmap subsystem
- Phase 2: get_map_mode
- Phase 3: argument parsing (replace_termcodes, special modifiers)
- Phase 4: mapping lookup & translation (check_map, map_to_exists)
- Phase 5: mutation primitives (map_clear, map_add, mapblock_free)
- Phase 6: buf_do_map + do_map (core mapping logic)
- Phase 7: completion + serialization (ExpandMappings, makemap helpers, put_escstr)
- Phase 8: ex commands + abbreviation logic

All checks pass: build, smoke-test, fmt, clippy, and 4,155 unit tests.
