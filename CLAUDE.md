## Guidelines

- `rust-migration` is your main branch.
- Every time you begin working on a new migration phase, create a new `phases/<id>-<slug>` branch.
- When you finish the phase, merge it back to `rust-migration`.
- Commit your work regularly.
- Run tests and static analysis before committing.
- Use `just` to run builds, tests, and static analysis.
- Keep the `justfile` up to date.

## Migration Plan

- The migration plan is stored in `plans/migration.md`.
- **Update the checklist in `plans/migration.md` as you complete items.** Mark items with `[x]` when done.
- If you encounter fundamental issues, research alternatives and create a new plan for following phases.
- **Compact the migration file regularly** to keep it manageable:
  - Keep the "Current Status" section updated with the latest phase
  - Collapse completed phases into summary bullet points when they become too detailed
  - Preserve key patterns, decisions, and blockers that inform future work
  - Keep the "Next Steps" section focused on actionable items
