Plan: ~/.claude/plans/fizzy-finding-steele.md

All steps complete. Summary:

**Deleted 172 lines of dead C code across 3 files:**
- `src/nvim/map_glyph_cache.c` — deleted entirely (109 lines, 4 functions with zero callers)
- `src/nvim/map_defs.h` — removed `MH_DECLS(glyph, char, String)` type declaration
- `src/nvim/grid.c` — removed 62 lines: `glyph_cache` variable + comment, 2 dead accessor functions (`nvim_glyph_cache_get`, `nvim_glyph_cache_n_keys`), `schar_idx` macro, 3 dead static functions (`schar_get_first_byte`, `schar_in_arabic_block`, `schar_get_first_two_codepoints`), and `set_destroy` cleanup call

All checks pass: build, smoke-test, rust-fmt-check, rust-clippy, and rust-test (3519/3519).
