Plan: ~/.claude/plans/fuzz-testing-regexp.md

All plan steps are complete. Here's a summary:

**Commit**: `48c98cb4b0` — `feat(regexp): add VimL-based regexp fuzz testing`

**What was done:**

1. **`test/regexp_fuzz.vim`** — New VimL fuzz test script that:
   - Tests 18 adversarial patterns (catastrophic backtracking, unclosed brackets, deep nesting)
   - Generates 10,000 random regexp patterns from metacharacters
   - Tests each pattern against 5 engine variants: default, `\v`, `\V`, BT (`\%#=1`), NFA (`\%#=2`)
   - 50,018 total regexp operations per run
   - Uses `silent!` inside `try/catch` to properly suppress non-exception regexp error messages

2. **`justfile`** — Replaced the cargo-fuzz `regexp-fuzz` target with a VimL-based one that runs inside nvim with a 30-second timeout

3. **`fuzz/` stubs** — Updated `regexp_compile.rs`, `regexp_match.rs`, and `fuzz/Cargo.toml` to document the FFI limitation and point to `just regexp-fuzz`

**All checks passed:** build, regexp-fuzz, smoke-test, rust-fmt-check, rust-clippy, rust-test (3518 tests)

**Notable finding:** VimL regexp errors (like `E866: Misplaced +`) are not proper exceptions — they bypass `try/catch` when running from a sourced script, causing the script to hang waiting for user input. The fix is `try | silent! call matchstrpos(...) | catch | endtry`.
