Plan: ~/.claude/plans/parallel-dreaming-key.md

All 4 phases of the BT regexp execution infrastructure migration are complete.

## Migration Complete: BT Execution Support Functions

### Commits (4 total):
1. **39093bf8** — Phase 1: reg_save/reg_restore/reg_save_equal/save_se_multi/save_se_one
2. **eba7bf31** — Phase 2: save_subexpr/restore_subexpr
3. **7154d781** — Phase 3: regrepeat (~360 lines, largest target)
4. **77cc725c** — Phase 4: regtry (sets up match, calls regmatch, packages results)

### Key metrics:
- **68** `#[no_mangle]` exports in regexp crate (was 59)
- **~480 lines** of C replaced with thin wrappers
- 18 new C accessor functions added
- C functions reduced to thin wrappers:
  - `reg_save` → `rs_reg_save(save, gap->ga_len)`
  - `reg_restore` → `rs_reg_restore(save, &gap->ga_len)`
  - `reg_save_equal` → `rs_reg_save_equal(save) != 0`
  - `save_se_multi` → `rs_save_se_multi(savep, posp)`
  - `save_se_one` → `rs_save_se_one(savep, pp)`
  - `save_subexpr` → `rs_save_subexpr(bp)`
  - `restore_subexpr` → `rs_restore_subexpr(bp)`
  - `regrepeat` → `rs_regrepeat(p, maxcount)`
  - `regtry` → `rs_regtry(prog, col, tm, timed_out)`
- All checks pass: build, smoke-test, fmt, clippy, 3491 Rust tests, 628 regexp corpus entries
