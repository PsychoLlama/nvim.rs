Plan: ~/.claude/plans/swift-orbiting-lake.md

All 4 phases of the extmark migration plan are complete:

**Phase 1** (committed earlier): Wired up 12 existing Rust implementations as thin `rs_` wrappers in `extmark.c`.

**Phase 2** (committed earlier): Implemented `extmark_set` in Rust, handling namespace tracking, in-place mark revision, deletion+recreation, and decoration registration.

**Phase 3**: Migrated `extmark_get` and `push_mark` to Rust:
- Fixed a **MTPair layout mismatch** bug — the extmark crate had `{ MTKey start; MTKey end }` (80 bytes) instead of the correct C layout `{ MTKey start; MTPos end_pos; bool end_right_gravity }` (~52 bytes)
- Added 6 new C accessor wrappers for overlap iteration, type flags, and array operations
- Implemented `push_mark` (namespace + type filtering) and `rs_extmark_get` (overlap + regular iteration paths)

**Phase 4**: Cleaned up dead code:
- Removed 12 dead `rs_flags_*`/`rs_pos_*` extern declarations
- Trimmed 6 unused includes
- Added `nvim-extmark` to justfile (4254 tests total, up from 4239)
- Fixed 2 incorrect test assertions

The `extmark.c` file now contains only thin wrapper functions calling Rust implementations, plus C accessor functions for the opaque handle pattern.
