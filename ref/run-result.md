Plan: ~/.claude/plans/lucky-giggling-meerkat.md

Migration of `src/nvim/ex_cmds2.c` is complete. All 8 phases executed successfully:

- **53 `#[no_mangle]` exports** across 9 Rust modules (script_host, autowrite_impl, bufcheck, check_changed_any, compiler, dialog, drop, listdo_impl, + lib.rs constants)
- **26 C functions** replaced with thin `rs_*()` wrappers
- **~100+ C accessor functions** (`nvim_ex2_*`) for FFI access to struct fields and globals
- **`_Static_assert` guards** for every hardcoded constant
- **3626 unit tests** passing, all clippy/fmt/build/smoke checks green

Commits:
1. `1204c2e` — refactor(ex_cmds2): migrate phases 1-5 to Rust
2. `48b1d31` — refactor(ex_cmds2): migrate phases 6-8 to Rust
