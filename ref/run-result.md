Plan: ~/.claude/plans/whimsical-knitting-lark.md

All 10 phases of the `sign.c` migration plan are complete:

| Phase | Description | Commit |
|-------|-------------|--------|
| 1 | Core types & constants | `033c18f160` |
| 2 | Sign text utilities | `479e8679f6` |
| 3 | Sign definitions | `60e7fef022` |
| 4 | Sign placement | `7d76af33e4` |
| 5 | Sign removal | `8e67a7fafa` |
| 6 | High-level operations | `f97a57134d` |
| 7 | Display/listing | `715847a18e` |
| 8 | Ex command handling | `267866fe48` |
| 9 | Command completion | `cdf85eba10` |
| 10 | VimL f_sign_* functions | `f02d86c287` |

All checks pass: build, smoke-test, rust-fmt-check, rust-clippy, and rust-test (4106 tests).
