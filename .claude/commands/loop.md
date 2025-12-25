---
description: "Start infinite loop until completion promise is met"
argument-hint: 'PROMPT --promise "TEXT"'
---

!`"${PWD}/.claude/scripts/setup-loop.sh" $ARGUMENTS`

Work on the task above. Your previous work is preserved in files and git history.

CRITICAL RULE: To exit, output `<promise>COMPLETION_PROMISE</promise>` where
COMPLETION_PROMISE matches what was set. You may ONLY output this when the
statement is completely and unequivocally TRUE. Do not lie to escape.
