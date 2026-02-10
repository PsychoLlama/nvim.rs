Plan: ~/.claude/plans/binary-gliding-floyd.md

All 8 phases of the bufwrite.c migration are complete. Here's a summary:

**Phase 0**: Created bufwrite crate scaffold
**Phase 1**: Migrated line ending / BOM / encoding helpers
**Phase 2**: Migrated `ucs2bytes` and `make_bom`
**Phase 3**: Migrated file info functions (`check_mtime`, `get_fileinfo_os`, `get_fileinfo`)
**Phase 4**: Migrated conversion pipeline (`buf_write_bytes`, `buf_write_convert`, iconv)
**Phase 5**: Migrated backup management (`buf_write_make_backup`)
**Phase 6**: Migrated autocommand wrappers (`buf_write_do_autocmds`, `buf_write_do_post_autocmds`)
**Phase 7**: Migrated the main `buf_write()` orchestrator (~880 lines) - the largest and most complex function

The C `buf_write()` function is now a thin wrapper calling `rs_buf_write()` in Rust. All verification passes:
- `just build` - links successfully
- `just smoke-test` - nvim starts, 29 regexp smoke tests pass
- `just rust-fmt-check` - clean
- `just rust-clippy` - clean
- `just rust-test` - 3779/3779 tests pass
