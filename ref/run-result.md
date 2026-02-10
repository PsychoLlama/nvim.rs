Plan: ~/.claude/plans/glimmering-tumbling-wombat.md

All 8 phases of the ShaDa migration plan are complete. Here's the summary:

## ShaDa Migration — Complete

| Phase | Description | Commit |
|-------|-------------|--------|
| 0 | Add `nvim-shada` to justfile `rust-test` | (previous session) |
| 1 | Pure stateless helpers (`marklist_insert`, `compare_file_marks`, `replace_numbered_mark`) | `449976dd6a` |
| 2 | Buffer/path filtering (`shada_removable`, `ignore_buf`, `find_removable_bufs`) | `c66d98d7ae` |
| 3 | Data collection (`shada_hist_iter`, `add_search_pattern`, `shada_initialize_registers`, `shada_get_buflist`, `shada_init_jumps`) | `1f8fffb736` |
| 4 | Entry free consolidation (`shada_free_shada_entry` → Rust) | `016de875e8` |
| 5 | File I/O wrappers (`close_file`, `shada_get_default_file`, `shada_read_file`) | `14dc93ad80` |
| 6 | Public API wrappers (`shada_read_marks`, `shada_read_everything`, `check_marks_read`) | `e6c7a573bc` |
| 7 | History merger deduplication (`hmll_*`/`hms_*` → thin wrappers) | `cda61c27ce` |

### Final Statistics
- **Rust `#[no_mangle]` exports:** 92 (up from 77)
- **Rust lib.rs:** 4,309 lines (up from ~3,459)
- **C shada.c:** 3,890 lines (down from ~3,899, net small decrease since new accessor functions offset deleted logic)
- All checks pass: `build`, `smoke-test`, `rust-fmt-check`, `rust-clippy`, `rust-test` (4,012 tests)
