Plan: ~/.claude/plans/breezy-wibbling-alpaca.md

The regex engine is working correctly with the migrated Rust helpers.

Based on my analysis, the regexp migration plan has been partially completed:

**Completed:**
- Phase 1: Helper functions (`cstrchr`, `cstrncmp`) migrated to Rust
- Phase 2: BT engine support functions (`reg_breakcheck`, `reg_iswordc`, `reg_prev_class`, `reg_nextline`) migrated
- Phase 3: NFA engine now uses shared Rust helpers
- Phase 4: Key compilation helpers are wired (`rs_bt_find_regmust`, `rs_bt_get_regstart`, `rs_re_put_uint32`, `rs_read_limits`, scanner functions)

**Pending (requires significant work):**
- Phase 5: Public API migration (`vim_regcomp`, `vim_regexec` - these would require migrating the ~1000-line `reg()` parser and ~1400-line `regmatch()` functions)
- Phase 6: Cleanup and optimization

The remaining work to fully migrate the regexp engine would require:
1. Converting global state (`regcode`, `regsize`, `regparse`) to Rust-owned structures
2. Migrating the recursive descent parser `reg()` (~1000 lines)
3. Migrating `regmatch()` (~1400 lines) and `nfa_regmatch()` (~1400 lines)
4. Migrating the public API entry points

This is substantial work that was beyond what could be completed in this session. The incremental work done establishes the foundation (FFI bindings, accessors, helper functions) that future phases would build upon.
