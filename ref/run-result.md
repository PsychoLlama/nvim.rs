Plan: ~/.claude/plans/streamed-prancing-wall.md

All 7 phases of the regmatch migration plan are complete. Here's a summary:

## Results

**Phases completed**: 1-7 (Phase 8 deferred per plan)

**Commits created**:
1. `feat(regexp): add regmatch infrastructure — accessors, types, constants` (Phase 1)
2. `feat(regexp): add regmatch core loops + simple opcodes (Phase 2)` 
3. `feat(regexp): add character classes, string matching, word boundaries (Phase 3)`
4. `feat(regexp): add groups, branches, backrefs to regmatch (Phase 4)`
5. `feat(regexp): add quantifiers to regmatch (Phase 5)`
6. `feat(regexp): add lookaround + special position opcodes (Phase 6)`
7. `feat(regexp): activate Rust rs_regmatch, delete C regmatch (Phase 7)`

**Lines changed**:
- **~1493 lines of C deleted** (regmatch + 9 helper functions)
- **~1100 lines of Rust added** (rs_regmatch_impl with all opcodes + backtracking handlers)
- **~125 lines of C accessors added** (for Rust FFI)

**All opcodes implemented**: BOL, EOL, RE_BOF, RE_EOF, ANY, NOTHING, NEWL, BHPOS, RE_COMPOSING, BACK, END, 26 character classes, EXACTLY, ANYOF, ANYBUT, MULTIBYTECODE, BOW, EOW, MOPEN/MCLOSE, NOPEN/NCLOSE, ZOPEN/ZCLOSE, BACKREF, ZREF, BRANCH, BRACE_LIMITS, BRACE_COMPLEX, STAR, PLUS, BRACE_SIMPLE, NOMATCH, MATCH, SUBPAT, BEHIND, NOBEHIND, CURSOR, RE_MARK, RE_VISUAL, RE_LNUM, RE_COL, RE_VCOL

**All backtracking handlers**: RS_NOPEN, RS_MOPEN, RS_MCLOSE, RS_ZOPEN, RS_ZCLOSE, RS_BRANCH, RS_NOMATCH, RS_BEHIND1, RS_BEHIND2, RS_BRCPLX_MORE, RS_BRCPLX_LONG, RS_BRCPLX_SHORT, RS_STAR_LONG, RS_STAR_SHORT

**Validation**: All 3491 unit tests pass, regexp baseline (628 entries) unchanged, smoke test clean, clippy clean, formatting clean.
