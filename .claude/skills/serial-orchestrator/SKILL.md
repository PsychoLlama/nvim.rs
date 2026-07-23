---
description: Plan work, then execute each phase in serial with a subagent.
disable-model-invocation: true
user-invocable: true
arguments:
  - migration_plan
---

## Preflight

- Stop immediately and ask if the phase you're responsible for wasn't specified.

## Steps

- Read $migration_plan to understand your slice of work.
- Don't implement changes yourself; delegate work to subagents (model=fable).
- Don't try to multi-task subagents - one at a time, all on the current branch.
- Carry learnings from one subagent to the next using a scratchpad.

## Unplanned Work

- For major concerns with a clear fix, prioritize it within your session and delegate a fix. (It can be bundled with other delegated changes.)

## Final Report

When finished with all work:

- Include the learnings scratchpad file path.
