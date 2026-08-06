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
- Don't implement changes yourself; delegate work to subagents (model=opus, not your default).
- Don't try to multi-task subagents - one at a time, all on the current branch.

## Scratchpad

- Carry learnings from one subagent to the next using a scratchpad.
- Each slice gets its own learnings section under a common directory.
- Provide an index file linking to each learnings file, maintained by each serial agent.
- Instruct each subagent to compact the index file at the end of its work to keep it concise and relevant.
- Declare a maximum size for the index file. Have agents keep it below that size. Otherwise, it risks serious context bloat.

## Unplanned Work

- For major concerns with a clear fix, prioritize it within your session and delegate a fix. (It can be bundled with other delegated changes.)

## Final Report

When finished with all work:

- Include the learnings scratchpad file path.
