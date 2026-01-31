Plan: ~/.claude/plans/witty-roaming-galaxy.md

I've completed phases 5.1 through 5.4 of the plan. Here's a summary:

## Completed Work (Phases 5.1-5.4)

### Phase 5.1: Integration Wiring
- Wired `rs_nfa_process_state()` to be called before the C switch statement
- Deleted C cases for: NFA_MATCH, anchors (BOL, EOL, BOW, EOW, BOF, EOF), NFA_NEWL, NFA_ANY, NFA_ANY_COMPOSING, character classes (NFA_IDENT through NFA_NUPPER_IC), literal character matching
- ~250 C lines deleted

### Phase 5.2: NFA_SKIP
- Implemented `process_skip()` in Rust for backreference skip handling
- ~15 C lines deleted

### Phase 5.3: Position Matching (partial)
- Implemented `process_position()` for NFA_LNUM, NFA_COL, NFA_CURSOR
- Added FFI accessors for reg_firstlnum, cursor position
- ~35 C lines deleted

### Phase 5.4: Backreferences  
- Implemented `process_backref()` for NFA_BACKREF1-9, NFA_ZREF1-9
- Added FFI wrappers for match_backref and match_zref
- ~50 C lines deleted

**Total: ~350 C lines deleted, ~170 Rust lines added**

## Remaining Work (Phases 5.5-5.10)

The remaining phases require more complex infrastructure:
- **Phase 5.5**: POSIX character classes (need check_char_class integration)
- **Phase 5.6**: NFA_COMPOSING (complex Unicode handling)
- **Phase 5.7**: Character collections (NFA_START_COLL with embedded composing)
- **Phase 5.8**: Invisible/Lookaround (recursive_regmatch infrastructure)
- **Phase 5.9**: NFA_START_PATTERN (pattern recursion)
- **Phase 5.10**: Cleanup (empty cases)

These phases involve recursive matching infrastructure and complex Unicode handling that require significant additional FFI work.
