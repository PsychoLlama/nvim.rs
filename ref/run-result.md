Plan: ~/.claude/plans/delightful-giggling-fern.md

The plan indicates that **no further migration work is needed for `ex_cmds2.c`**. The analysis found that:

1. All original function bodies from the C file have already been migrated to Rust in `src/nvim-rs/ex_cmds2/` (2,286 lines across 11 modules).
2. The remaining ~1,117 lines in the C file are **FFI glue only** — accessor functions, thin wrappers calling `rs_*` functions, and static assertions. This is the expected end-state for a migrated file.
3. The one function with real logic (`nvim_ex2_listdo_restore_syntax`) was intentionally kept in C due to deep coupling with autocmd internals and stack-allocated structs.

**All steps are complete — there is nothing to implement.** If you'd like to continue migration work, I can help identify a new C file candidate to migrate.
