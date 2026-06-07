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

## OPEN functional-suite failures (triage needed) — found 2026-06-07

Partial run of `test/functional/vimscript/` (timed out / hung at `errorlist_spec`
`setloclist` — itself a likely crash/hang regression; only ~7 of ~40 specs ran).
Each needs root-cause + fix + a regression guard. Likely several share a root cause
(as the VarType cluster did). **Verify each is a real product bug vs. a test-env
artifact before fixing.**

- `buf_functions_spec`: `getbufvar()` error-handling (T19, T20); `bufnr("$")` returns
  wrong value (T27).
- `ctx_functions_spec`: context stack — `ctxpush/ctxpop` register save/restore (T52),
  jumplist save/restore (T53), `ctxget()` (T61), `ctxset()` (T64).
- `errorlist_spec`: `setqflist()` `{action}`/`{what}` arg validation (T71, T74);
  `setloclist()` `{action}` validation (T76); **possible hang/crash at T78**
  ("setloclist doesn't crash when window closed in the middle #13721").
- `api_functions_spec`: textlock not enforced for eval-API (T3); vim.vim syntax
  highlight (T8 — may be test-env/runtime, verify).

**Not yet run** (the bulk of the suite): the rest of `vimscript/`, plus
`test/functional/{api,editor,core,ui,lua,…}`. A full triage pass is warranted —
run the whole suite, collect the failing-spec list, and group into fix waves.

## Lessons for executors (anti-patterns observed)

- **Relocate-not-port** (wave 66, and the get_scriptname trampoline): moving C logic
  behind a one-line passthrough — or worse, a passthrough that recurses. Always
  verify NET C across ALL files AND that the Rust side actually implements logic.
- **Hallucinated per-file constants**: redefining enum values locally with wrong
  numbers. Import from the canonical source (`eval/src/expr/constants.rs`,
  `nvim-typval::VarType`) or add `const _: () = assert!(...)` guards.
- **"4339 tests pass" ≠ correct**: that's Rust unit tests only. Run functional specs.
