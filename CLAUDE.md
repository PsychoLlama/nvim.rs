## Guidelines

- `rust-migration` is your main branch.
- Every time you begin working on a new migration phase, create a new `phases/<id>-<slug>` branch.
- When you finish the phase, merge it back to `rust-migration`.
- Commit your work regularly.
- Run tests and static analysis before committing.
- Use `just` to run builds, tests, and static analysis.
- Keep the `justfile` up to date.
