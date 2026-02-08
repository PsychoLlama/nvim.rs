Plan: ~/.claude/plans/eager-spinning-wozniak.md

All 4 phases are complete. Here's a summary of what was done:

**4 commits, 10 functions migrated to Rust:**

| Commit | Functions | Description |
|--------|-----------|-------------|
| 1 | `rs_re_put_uint32`, `rs_regc`, `rs_regmbc` | Big-endian uint32 write + byte/multibyte code emission. Added 7 C accessor functions for compilation globals (regcode, regsize, reg_toolong, JUST_CALC_SIZE). |
| 2 | `rs_regnode`, `rs_regnext` | 3-byte node emission + bidirectional chain navigation via 16-bit offsets. |
| 3 | `rs_regtail`, `rs_regoptail` | Chain linking: walk to end of node chain, compute and write 16-bit next-pointer. Conditional variant for BRANCH/BRACE_COMPLEX opcodes. |
| 4 | `rs_reginsert`, `rs_reginsert_nr`, `rs_reginsert_limits` | Byte-shifting insertion operations using `ptr::copy` (memmove semantics) for 3/7/11-byte operator nodes. |

**All checks pass for every commit:**
- `just build` — links successfully
- `just smoke-test` — nvim starts without crashing
- `just rust-fmt-check` — formatting clean
- `just rust-clippy` — no warnings
- `just rust-test` — 3486 tests pass
