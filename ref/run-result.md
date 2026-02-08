Plan: ~/.claude/plans/woolly-giggling-flurry.md

All 8 phases of the `regatom` migration are complete. Here's a summary:

## Migration Complete: `regatom` → `rs_regatom`

The BT regexp recursive descent parser chain is now fully in Rust:
`rs_reg → rs_regbranch → rs_regconcat → rs_regpiece → rs_regatom`

### Commits (8 total):
1. **d80ce005** — Phase 1: C accessor functions and error helpers
2. **482cccef** — Phase 2: Rust constants, extern decls, and helpers (`use_multibytecode`, `coll_get_char`, `seen_endbrace`)
3. **0a1959d8** — Phase 3: Position assertions (`^$<>`), char class shortcuts (`.iIkK...`), `\n`
4. **72dba9ca** — Phase 4: Grouping `\(`, backreferences `\1-\9`, `\z`, `\~`, error cases
5. **70c04ecc** — Phase 5: Percent operators `\%^$#VCd/o/x/u/U`, `\%[abc]`, `\%l/c/v`, `\%'m`
6. **15a08696** — Phase 6: Collection parsing `[...]` with POSIX classes, ranges, equivalence classes
7. **dbd7e3dc** — Phase 7: Default/literal EXACTLY case + wiring (55 lines Rust added, 822 lines C deleted)
8. **ed408eea** — Phase 8: Test patterns + corpus regeneration (581 entries)

### Key metrics:
- **59** `#[no_mangle]` exports in regexp crate (was 58)
- **~800 lines** of C deleted from `regatom`
- C `regatom` reduced to: `return rs_regatom(flagp);`
- `seen_endbrace` and `coll_get_char` kept in C (shared with NFA engine)
- All checks pass: build, smoke-test, fmt, clippy, 3486 Rust tests
