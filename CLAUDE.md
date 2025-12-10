## Guidelines

- `rust-migration` is your main branch.
- Every time you begin working on a new migration phase, create a new `phases/<id>-<slug>` branch.
- When you finish the phase, merge it back to `rust-migration`.
- Commit your work regularly.
- Run tests and static analysis before committing.
- Use `just` to run builds, tests, and static analysis.
- Keep the `justfile` up to date.

## Migration Plan

The migration plan lives in `plans/migration.md`. This document is critical for continuity across sessions.

### What to Keep in migration.md

1. **Current Status**: Update the function count and latest phase at the top. Run `grep -rh "^#\[no_mangle\]" src/nvim-rs --include="*.rs" | wc -l` to get the accurate count.

2. **Crate Overview Table**: One row per crate with its purpose and 2-3 key functions. Add new crates here when created.

3. **Architecture & Patterns**: The opaque handle pattern, conditional compilation pattern, and build system notes. These are essential for writing new migrations correctly.

4. **Phase Summary Table**: Keep completed phases as single-row summaries. Only expand detail for the current in-progress phase.

5. **Quick Commands**: Keep the grep/find commands that help discover what's been migrated.

### What NOT to Keep

- Exhaustive function lists (grep can find these)
- Detailed per-function notes from completed phases
- Duplicate or conflicting phase numbers
- Speculative future work beyond the next 2-3 phases

### When to Update

- **After completing a phase**: Update status, add crate to table if new, collapse phase detail
- **After hitting a blocker**: Document it clearly in the "In Progress" section
- **When document exceeds ~200 lines**: Audit and compact

### Discovery Commands

```bash
# Functions in a crate
grep -n "pub.*extern.*fn rs_" src/nvim-rs/<crate>/src/lib.rs

# All USE_RUST flags
grep "USE_RUST_" src/nvim/CMakeLists.txt

# Find C accessor functions
grep -rn "nvim_get_" src/nvim/*.c --include="*.c" | grep -v "^Binary"
```
