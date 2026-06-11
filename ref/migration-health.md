# Migration Health & Known Regressions

This file tracks correctness regressions discovered in the Rust-migration branch
and the process gaps that let them accumulate. It is a roadmap for fix waves.
Keep it updated as regressions are fixed (strike through / remove) or found.

## CRITICAL PROCESS GAP — `just check` does not run the functional suite

`just check` = `build + smoke-test + rust-check`. It runs the Rust *unit* tests
(`cargo nextest` on pure crates) and three `.vim` smoke files — but it does **NOT**
run Neovim's C/Lua **functional test suite** (`functionaltest`). The "4339 tests
pass" that executors cite are Rust-only unit tests. As a result, broken VimL
builtins and crashes in core paths have been merged repeatedly without detection.

**The functional suite DOES work here and is fast per-spec.** Run a targeted spec:

```bash
TEST_FILE=test/functional/vimscript/writefile_spec.lua \
  timeout -s 9 580 cmake --build build --target functionaltest
```

`TEST_FILE` may be a single spec or a directory. Also supports `TEST_FILTER`,
`TEST_TAG`, `TEST_FILTER_OUT` (see `cmake/RunTests.cmake`).

**Recommendation:** every migration/fix wave that touches a builtin or eval path
should run the relevant functional spec(s), not just `just check`. Consider adding
a `just functionaltest-vimscript` recipe and running it before claiming a wave green.

## Regression-guard smoke tests added (partial mitigation)

Wired into `just smoke-test` (so `just check` now covers them):
- `test/regexp_smoke.vim` — regexp (29 cases)
- `test/throw_smoke.vim` — exception throw from inside functions (4 cases) [Wave 70]
- `test/type_smoke.vim` — VarType-constant-sensitive builtins (22 cases) [Wave 71]

## Fixed regressions

- **[Wave 70] P0 SIGSEGV: throw from inside a function.** 3-way infinite recursion
  `get_scriptname` (runtime.c) → `rs_get_scriptname` (never-implemented trampoline)
  → `nvim_rt_get_scriptname` (runtime_ffi.c) → … Introduced by commit `7f4f9ecf9a`
  (a relocate-not-port that created a cycle). Fixed by genuinely porting the SID
  switch into Rust + deleting the trampoline. Commits `d477d29d64`, `61fe2e1241`.
- **[Wave 71] 13 wrong VarType constants across 10 files.** Per-file local copies of
  the VarType enum with wrong literals (e.g. fs.rs `VAR_STRING=6`, `VAR_LIST=5`).
  Broke `chdir`, `writefile([list])`, and others. Fixed by importing canonical
  constants / adding compile-time guards. Commits `795f4403ef`, `a8ffac30c8`,
  `e4d6b49f15`.

## OPEN functional-suite failures — AUTHORITATIVE TRIAGE (2026-06-08)

Two independent full triage passes (vimscript + api fully per-spec; core mostly;
other dirs sampled) CORROBORATE the clusters below. The migration has pervasive,
severe functional regressions invisible to `just check`. Fix-wave order is by
leverage (a single root cause often clears a whole spec). Per-spec logs were saved
under `/tmp/triage/` during the run (may be gone after reboot — re-run as needed).

**Reproduce any spec:** `TEST_FILE=test/functional/<path>_spec.lua timeout -s 9 560 cmake --build build --target functionaltest 2>&1 | tail -n 80`

### Cluster A — `b:` / buffer-var → E908 "Using an invalid value as a String" [HIGH, clean]
`getbufvar()`/`setbufvar()` and `b:changedtick` access wrongly raise E908 on valid
input. Clears TWO specs. Likely wrong VarType/typval string-coercion in the migrated
`vars` crate. Source leads: `src/nvim-rs/vars/src/viml_funcs.rs` (getbufvar/setbufvar),
b:changedtick dict population.
- `vimscript/buf_functions_spec.lua` (5 FAIL + 6 ERR), `vimscript/changedtick_spec.lua` (8 ERR).

### Cluster B — quickfix arg-type validation emits wrong error code [HIGH, clean]
`setqflist()`/`setloclist()` raise E731/E928 where E928/E715 expected (and vice-versa).
Source confirmed: `src/nvim-rs/quickfix/src/api.rs:~2003`.
- `vimscript/errorlist_spec.lua` (also HANGS, see D), `api/vim_spec.lua` nvim_call_function.

### Cluster C — JSON/msgpack codec CRASHES nvim ("EOF received") [CRITICAL]
`json_decode(list)` and `msgpackparse(systemlist(...))` hard-crash nvim; `json_encode`
returns '' / wrong. One crash poisons 60+ tests via EOF cascade. Source:
`src/nvim-rs/eval_codec/{decode,encode,json}.rs`.
- `vimscript/json_functions_spec.lua` (10F + 62E), `vimscript/msgpack_functions_spec.lua` (3F + 11E).

### Cluster D — HANGS / deadlocks (cmdwin · textlock · insert-completion · feedkeys) [CRITICAL]
A spec that hangs loses ALL its remaining tests and blocks whole-directory runs.
Strong shared sub-theme: **cmdwin + textlock guards**, and key-feeding that waits for
input that never arrives. Confirmed hangs: `errorlist`(setloclist window-close #13721),
`map_functions`(mapset replace_keycodes), `null`(complete()), `setpos`(at startup),
`system`(mid-suite), core/`fileio`(symlink backup #11349), core/`path`(gf multibyte #20517),
core/`remote`; api/`tabpage`(set_win when textlocked/cmdwin), api/`window`(set_buf in cmdwin),
api/`buffer`(E315), api/`extmark`(undo #25147), api/`keymap`(nowait), api/`vim`(nvim_paste insert),
editor/`completion`(v:completed_item), editor/`count`(v:count in cmdwin). Whole dirs that
hang on an early spec: editor, options, lua, ui, autocmd, ex_cmds, shada, treesitter,
plugin, terminal, legacy, provider. **Fixing the cmdwin/textlock deadlock likely clears several at once.**

### Cluster E — keymap dict serialization round-trip mismatch [MEDIUM, single root]
`nvim_get_keymap`/`nvim_set_keymap` round-trip returns a structurally different dict
(~23 "Expected objects to be the same" in `api/keymap_spec.lua`). One fix → ~20 tests.

### Cluster F — cursor/pos API rejects valid [row,col] [MEDIUM, clean]
`Argument "pos" must be a [row, col] array` on valid input. `api/buffer_spec.lua` (6E),
`vimscript/api_functions_spec.lua` eval-API.

### Cluster G — Lua function marshalling [MEDIUM]
`nvim_eval`/`nvim_call_function` return a Lua function as a string, not callable.
`api/vim_spec.lua` ("can return Lua function to Lua code").

### Cluster H — misc eval/ex-command (likely VarType family, shares A) [MEDIUM]
`let_spec`(:let listing curly/subscript vars CRASH), `match_functions`(setmatches CRASH),
`ctx_functions`(ctxpush/pop/get/set round-trip + buffer-list CRASH), editor/`ctrl_c`(:global E323),
editor/`fold`(:fold filter E493), api/`autocmd`(lambda E117 / augroup delete CRASH).

### Cluster I — startup / Ex-mode / stdin-tty [MEDIUM]
`core/startup`(10F+19E: stdin/pipe, ttyin/ttyout, -es/-Es, exrc), `core/main`(-s, Ex-mode),
`core/exit`(:cquit redir, v:exiting try-catch).

### Likely test-env artifacts (verify, de-prioritize)
Screen/redraw diff failures: `input_spec`(17F highlight), `screenchar_spec`(floating),
`timer`, `execute`, api/`ui`, editor/`defaults` popupmenu — noisy, terminal/timing-sensitive.
`system_spec` shell tests (SHELL=sh) and `environ`($HOME E108) partly env-dependent (but the
system_spec HANG is real). `script/` dir PASSES 98/98 → the test runner itself is healthy.

### Recommended fix order
A (E908) → C (codec crash) → D cmdwin/textlock sub-theme → B (quickfix codes) → E/F/G → H/I.

### STATUS (updated 2026-06-08, fix waves in progress)
- **Cluster A — FIXED** (commit 916bdf7d3b): root cause was `TV_SIZE=24` vs real 16 in the
  `vars` crate (argvar stride corruption) + a null-`first` deref in `:let b:` + a msg_row
  clamp. buf_functions 30/30, changedtick 10/10. Guard: test/buf_smoke.vim.
- **Cluster C — FIXED** (commit 2fadb04b26): `tv_list_equal/tv_dict_equal/tv_blob_equal`
  declared `-> c_int` but defined `-> bool` (x86-64 partial-register garbage). json 77/77,
  msgpack 71/71.
- **Cluster B — FIXED** (commit d596ba7ba8): wrong VarType magic numbers (5 vs VAR_STRING=2)
  + NULL deref in setqflist/setloclist. All arg-validation tests green (the T9 hang is D).
- **FFI Class A/B/C signature mismatches — FIXED** (commit 8ac4c7fe1d, 82 files): 59 bool/c_int
  return-type symbols corrected + arena_alloc + operators.rs param. Also fixed **let_spec
  CRASH → 9/9**. See ref/ffi-audit.md. REMAINING: Class D/E param mismatches (~170 sites, MEDIUM).
- **Cluster D — insert/feedkeys sub-theme FIXED** (commit 5e7aed2dc0): `VimState` struct in
  `edit/src/dispatch.rs` had `execute`/`check` fields in the WRONG ORDER vs `state_defs.h` /
  `state/src/lib.rs`. `state_enter` reads by offset, so the check handler was called as the
  execute handler — always returned "continue", spinning at 100% CPU in insert mode and never
  consuming ESC → hang. One-line field-order swap. Unblocks every spec using `insert()` in setup.
  setpos_spec now completes (5 tests, was a 200s hang). Guard: test/insert_smoke.vim.
- **Cluster D — ROOT-CAUSED + FIX IN FLIGHT (2026-06-10).** CONFIRMED via a real core-dump
  backtrace of the hung process (method below): the migrated `inchar`
  (src/nvim-rs/getchar/src/typebuf.rs, fn @1848) passes `std::ptr::null_mut()` as the `events`
  (MultiQueue*) arg to `input_get` at BOTH call sites (~:1902 got_int drain, ~:1916 main read);
  UPSTREAM getchar.c `inchar` passes `main_loop.events`. With events=NULL, `inbuf_poll`
  (os/input.c:487) does `LOOP_PROCESS_EVENTS_UNTIL(&main_loop, NULL, -1, os_input_ready(events)|...)`
  — `os_input_ready(NULL)` ignores the event queue and `loop_poll_events` drains only `fast_events`,
  so a deferred RPC request on `main_loop.events` is read off the socket but NEVER executed, and
  `vgetc` never returns K_EVENT to dispatch it → 0% CPU deadlock in ANY modal vgetc wait
  (command-line `:`, cmdwin `q:`, complete(), getchar()). The top-level normal loop survives via a
  separate event path (input.rs:823), which is why only modal waits hang. THIS ONE OMISSION likely
  explains a broad swath of Cluster D hangs. Captured stack: epoll_pwait <- uv_run <-
  loop_poll_events <- inbuf_poll(events=0x0) <- input_get(events=0x0) <- inchar <- vgetc <-
  safe_vgetc <- state_enter <- rs_command_line_enter <- do_cmdline <- rs_nv_colon. Fix (executor
  ad030dca, in flight): pass main_loop.events (rs_loop_get_events(main_loop)) at both sites.
  - **REAL-STACK METHOD (this finally worked — record for reuse):** gdb is NOT installed; live
    attach is blocked by yama ptrace_scope=1 ANYWAY. BUT `coredumpctl` extracts symbolized stacks
    with no ptrace. Repro the hang in isolation (TEST_FILTER to the one test), find the stable 0%-CPU
    repo build/bin/nvim child (WCHAN=do_epoll_wait), `kill -ABRT <pid>` (nvim's signal handler still
    leaves the pre-signal frames visible), then `coredumpctl info <pid>` for the stack, or
    `coredumpctl dump <pid> --output=/tmp/x.core && nix shell nixpkgs#gdb --command gdb build/bin/nvim
    /tmp/x.core -batch -ex 'thread apply all bt'` for all-thread + source lines.
- **SECOND HANG — insert-mode completion busy-loop — FIXED (commit c0516d97b3).** Real root
  cause: edit/dispatch.rs passed the WRONG pointer to the completion window check
  (nvim_curwin_get_cursor_ptr instead of nvim_get_curwin), so the window-walk in
  rs_ins_compl_next_buf compared NEXT_BUF_WP against a garbage curwin -> == curwin never matched
  -> infinite 99.7%-CPU spin. Fixed alongside a cluster of WRONG HLF highlight constants (verified
  vs highlight_defs.h: HLF_MSG 5->63, HLF_8 8->1, HLF_M 6->10, HLF_R_DICT 6->18) + menuone flag
  + redraw_mode bool. VERIFIED: just check 4339/4339, completion_spec.lua now COMPLETES in 97s
  (was hanging) with 32 fails / 3 errors remaining (ordinary functional bugs, NOT hangs — backlog).
  The "WinPtr vs WinHandle type mismatch" theory was REFUTED (ABI-identical pointers). LATENT
  (low priority, untouched): buffer.rs:168 still has a '&& focusable' term not in upstream's break
  condition (missed scans in multi-window completion; not a hang).
- **completion_spec off-by-one FIXED (commit d507160551, 2026-06-10).** rs_get_normal_compl_info
  (insexpand/pattern.rs:163) used `while startcol > 0 { startcol -= 1 }` (never reaches -1) vs
  upstream `while (--startcol >= 0)`; compl_col ended one column too high at a word in column 0.
  REAL fix, but LOW leverage: ground-truth spec run cleared only 1 error (20->21 pass, 25 fail,
  4->3 err) — NOT the ~24 a triage agent predicted (its "leading-space" grouping was inflated;
  those tests already passed). LESSON: trust ground-truth spec runs over subagent failure-groupings.
- **completion_spec E785 FIXED (commit 5cec6c06c1, 2026-06-10).** insexpand/funcexpand.rs:61
  declared `const MODE_INSERT = 0x80` (that's MODE_TERMINAL); real value is 0x10 (state_defs.h:25).
  The complete() insert-mode guard tested `State & 0x80` == 0 in real insert mode → every valid
  complete() call wrongly raised "E785: complete() can only be used in Insert mode". Fix 0x80->0x10
  + compile-time guard. VERIFIED: completion_spec 25F/4E -> 18F/3E (cleared 7F+1E; E785 gone).
  Another recurring wrong-constant-class instance, now guarded.
  REMAINING completion_spec backlog (18F/3E, NOT yet targeted): lua getcompletion() EOF CRASH
  @931/@943 (severity: crash); nvim__complete_set "Key not found: err_msg" @1348; v:completed_item
  E742-vs-E46 @73; ~15 independent completion-render snapshot diffs (diffuse, lower per-fix leverage);
  fuzzy+autocomplete has its OWN start-column bug distinct from rs_get_normal_compl_info.
  - **REFUTED theory** (a diagnostic agent's wrong guess — do NOT act on it): "WinPtr=*mut u8 vs
    WinHandle type mismatch corrupts the NEXT_BUF_WP==curwin comparison." False: nvim_get_curwin/
    firstwin (globals.rs:140/201) return WinHandle wrapping the raw global pointer; nvim_win_get_w_next
    (insexpand_shim.c:384) returns raw win_T*; WinHandle is repr(transparent) over the pointer — all
    ABI-identical bits, comparison is valid. nvim_win_get_focusable (insexpand_shim.c:385) returns 0/1
    correctly. Changing those types is cosmetic.
  - **CONFIRMED divergence:** buffer.rs:168 breaks on `NEXT_BUF_WP==curwin || (!scanned && focusable)`;
    upstream ins_compl_next_buf breaks on `wp==curwin || !wp->w_buffer->b_scanned` (NO focusable term).
    The added `&& focusable` is a port error. Likely additional cause: a b_scanned read/write or
    BufStruct-offset issue so the outer ins_compl_get_exp loop never reaches the done state.
- **NEW BUGS found in passing via the cores (separate from the deadlock, NOT yet fixed):**
  1. **Exit-path heap corruption:** SIGABRT during shutdown shows `preserve_exit -> ml_close_all ->
     free -> munmap_chunk/malloc_printerr (abort)` — a double-free / bad-free in memline close on exit.
  2. **jobstart/terminal NULL deref — FIXED (commit 79426aa484).** `nvim_proc_get_exepath`
     (event/proc.c, C accessor for Rust) was a bare `return proc->exepath`, dropping the upstream
     `argv[0]` fallback; in the normal jobstart/:terminal path exepath is NULL → xstrdup(NULL) →
     SIGSEGV. Fixed to `proc->exepath ? proc->exepath : proc->argv[0]`. Verified: termxx no longer
     instant-crashes, channels_spec 14/14, just check 4339/4339.
- **THIRD HANG — :terminal-open busy-loop — FIXED (commit 2889ac1a9a).** (Was mislabeled
  "TermClose deadlock"; the real stack showed a 99.8%-CPU spin on terminal OPEN, never reaching
  TermClose.) Root cause: Rust VTermScreenCell.schar was SChar=u64 but C schar_T=uint32_t (4B),
  misaligning `width` (Rust offset 8 vs C offset 4) so rs_fetch_row read width=0 -> col never
  advanced -> infinite loop. Fix: SChar u64->u32 + sentinel u64::MAX->u32::MAX + size guards
  (Rust assert + C _Static_assert == 24). Verified: terminal specs complete (~60s) instead of
  hanging; just check 4339/4339. 3rd LAYOUT-class bug this session (VarType sizes, HLF, now schar)
  — recurring gap is MISSING SIZE ASSERTS on repr(C) mirrors; a vterm/repr(C) assert sweep is warranted.
- **LAYOUT-BUG CLASS — AUDITED + CLOSED in vterm (commit 025f26faf6).** Proactive read-only audit
  of all 50+ repr(C) structs in the vterm crate found it otherwise CLEAN with ONE real mismatch:
  VTermGlyphInfo mirrored C's packed bitfields (protected_cell:1/dwl:1/dhl:2 in one 4-byte word)
  as 3 separate u8 fields -> dwl/dhl read as 0 (latent; putglyph doesn't read them yet). Fixed:
  flags -> single u32 + accessors; added compile-time guards (size + offset_of) on VTermGlyphInfo
  and C _Static_asserts on ScreenPen(16)/ScreenCell(20)/VTermStateFields(24)/VTermKeyEncodingFlags(1)/
  VTermKeyEncodingStack(17). These guards are the systemic fix — they would have caught all 3 layout
  bugs at compile time. FUTURE: extend the same repr(C) size/offset-assert discipline to other
  C-filled/Rust-read structs in other crates.
- **RE-BASELINE (2026-06-10, after both hang fixes):** big leverage confirmed — ~400-600 tests
  that previously hung now execute (autocmd ~158+, editor ~57+, ui ~338+). Regression spot-check
  green (buf_functions 30/30, json 77/77). The 3 dirs still don't fully COMPLETE within 560s:
  autocmd blocked by termxx jobstart crash (above); editor slow / candidate ctrl_c_spec; ui large
  but no specific hang identified (may just need more time). Next after jobstart: re-measure dirs.
- **(historical localization note)** Cluster D reproduces deterministically as
  `test/functional/autocmd/tabnewentered_spec.lua` **T57** ("cmdline-win prevents tab switch via
  g<Tab>"): `feed('q:')` enters the command-line window, then an RPC `eval('win_getid()')` NEVER
  returns. The nvim child sits at **0% CPU, state S** (blocked in the libuv event loop, eventpoll
  + io_uring fds open) — a BLOCK, not a busy-spin. The cmdwin's nested input loop is not
  pumping/servicing the multiqueue, so the blocking client API request is never executed →
  both sides wait forever. This single hang takes down the whole autocmd/editor/ui DIRECTORY runs.
  - **RULED OUT:** `stuff_empty` FFI mismatch — all decls are already `-> bool` (the prior FFI
    sweep fixed it). NOT the cause.
  - **Suspect area (unconfirmed):** the cmdwin nested loop's event pumping (`open_cmdwin` and the
    normal-mode state loop it runs) — does it drain the multiqueue between keystrokes? Compare the
    migrated path against upstream `ex_getln.c` open_cmdwin + input_get/inbuf_poll/loop_poll_events.
  - **To get a real stack next time:** live gdb-attach is blocked (`yama ptrace_scope=1`, the test
    nvim is a sibling, not a descendant of gdb). Options: (a) launch the child nvim UNDER gdb via a
    minimal standalone RPC repro (start nvim --embed as gdb's child, send nvim_input("q:") then
    nvim_eval), or (b) force a core with a signal nvim does NOT catch (SIGTRAP/SIGXCPU — SIGABRT/
    SIGQUIT/SIGSEGV are handled) and retrieve via `coredumpctl gdb <pid>` (core_pattern pipes to
    systemd-coredump). Then `thread apply all bt`.
  - Other specs in this cluster to re-check once fixed: window (set_buf in cmdwin), tabpage
    (set_win when textlocked), null T58 (complete() w/ NULL list), map_functions T13 (mapset
    replace_keycodes). Note: errorlist T9 (setloclist window-closed) is NOW GREEN (9/9) — unblocked.
- **Cluster H — partially open**: let_spec FIXED; ctx_functions still 5 FAIL (register/jumplist/
  buflist round-trip), match_functions 1 FAIL (matchaddpos zero-length) + setmatches was CRASH
  (recheck), editor/ctrl_c/fold, api/autocmd lambda.

## FRESH BASELINE (2026-06-09, after the insert-spin fix 5e7aed2dc0)

A one-time re-baseline was run now that the insert-mode 100% CPU spin is fixed. Results:

**Fixed clusters held GREEN** (regression-checked individually):
- buf_functions 30/30, changedtick 10/10, json 77/77, msgpack 71/71, let 9/9.
- `errorlist` now 9/9 — it was a Cluster D HANG (setloclist window-close, T9) in the prior
  triage, so the insert-spin fix (or a related change) UNBLOCKED it.
- `setpos` runs fully (50/52; was a 200s hang). The 2 remaining failures are the pre-existing
  setpos() return-value bug (T1/T2 inverted -1 vs 0), NOT the old hang.

**Whole-directory HANGS PERSIST (distinct root cause — NOT the insert-spin):**
- `test/functional/editor` — DIR HANGS (560s timeout)
- `test/functional/ui` — DIR HANGS
- `test/functional/autocmd` — DIR HANGS
- `test/functional/options` — now COMPLETES 62/114 (failures: UI-render attr 18 in fillchars
  statusline tests + a disk-full artifact in defaults_spec; not hangs).

**Interpretation:** the insert-spin fix did NOT clear editor/ui/autocmd. These hang on a
SEPARATE deadlock (open Cluster D cmdwin/textlock sub-theme, or an event-loop/libuv wait).
A diagnostic bisect+gdb-backtrace pass is in flight to localize it. This blocks the autocmd
C-deletion plan (ref/plans/e1b12171-…, Phase 2 gates on 7 autocmd specs that live in the
hanging dir) — fix the deadlock BEFORE that port.

## Lessons for executors (anti-patterns observed)

- **Relocate-not-port** (wave 66, and the get_scriptname trampoline): moving C logic
  behind a one-line passthrough — or worse, a passthrough that recurses. Always
  verify NET C across ALL files AND that the Rust side actually implements logic.
- **Hallucinated per-file constants**: redefining enum values locally with wrong
  numbers. Import from the canonical source (`eval/src/expr/constants.rs`,
  `nvim-typval::VarType`) or add `const _: () = assert!(...)` guards.
- **"4339 tests pass" ≠ correct**: that's Rust unit tests only. Run functional specs.
