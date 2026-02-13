Plan: ~/.claude/plans/starry-dancing-naur.md

The plan indicates that the session crate migration is already complete. The C file `src/nvim/ex_session.c` is already a pure FFI bridge layer, which is the expected end-state for a fully migrated crate.

There's nothing to execute here — no migration work is needed. The plan recommends either:

1. **Picking a different C file** to migrate next
2. **Reducing the FFI surface** of the session crate (optimization, not migration)

All steps are effectively "complete" since the analysis confirmed no work is required. Would you like me to identify the next C file that has substantial unmigrated logic to work on instead?
