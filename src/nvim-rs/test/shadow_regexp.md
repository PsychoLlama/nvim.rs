# Shadow Mode Strategy for Regexp Migration

## Overview

Shadow mode runs both C and Rust regexp implementations in parallel, asserting
equivalence on every real operation. This turns all of Neovim's normal usage
(opening files, syntax highlighting, search, substitution) into a continuous
integration test. A divergence triggers an assertion immediately, not 50
commits later.

## How It Works

During migration, instead of deleting C functions when Rust replacements are
written, **keep both and compare**:

```c
// In regexp.c during migration:
bool vim_regexec_shadow(regmatch_T *rmp, const char *line, colnr_T col)
{
    bool c_result = vim_regexec_c(rmp, line, col);   // original C
    bool rs_result = rs_vim_regexec(rmp, line, col);  // new Rust

    if (c_result != rs_result) {
        // Log the divergence with full context for debugging
        fprintf(stderr, "REGEXP SHADOW DIVERGENCE: pattern=%s line=%s col=%d "
                "c=%d rust=%d\n",
                rmp->regprog ? "..." : "NULL", line, col,
                c_result, rs_result);
        assert(c_result == rs_result);  // crash in debug builds
    }

    return c_result;  // always return C result during validation
}
```

## Implementation Steps

### Phase 1: Rename and Wrap

1. Rename the original C function: `vim_regexec` → `vim_regexec_c`
2. Create a shadow wrapper: `vim_regexec` that calls both
3. All callers continue to call `vim_regexec` - no changes needed

### Phase 2: Add Rust Implementation

1. Write `rs_vim_regexec` in the Rust regexp crate
2. The shadow wrapper now calls both C and Rust
3. Run the full test suite, open files, do searches - every operation is tested

### Phase 3: Compare Results Deeply

For match operations, compare more than just the boolean result:
- Match position (`startp`, `endp`)
- All submatches (`startp[1..9]`, `endp[1..9]`)
- The `rm_matchcol` field

```c
bool vim_regexec_shadow(regmatch_T *rmp, const char *line, colnr_T col)
{
    // Make copies for both engines
    regmatch_T c_rmp = *rmp;
    regmatch_T rs_rmp = *rmp;

    bool c_result = vim_regexec_c(&c_rmp, line, col);
    bool rs_result = rs_vim_regexec(&rs_rmp, line, col);

    assert(c_result == rs_result);

    if (c_result) {
        // Compare all submatch positions
        for (int i = 0; i < NSUBEXP; i++) {
            assert(c_rmp.startp[i] == rs_rmp.startp[i]);
            assert(c_rmp.endp[i] == rs_rmp.endp[i]);
        }
    }

    // Return C result with C's match state
    *rmp = c_rmp;
    return c_result;
}
```

### Phase 4: Monitor and Switch

1. Run with shadow mode through the full test suite
2. Run with shadow mode during manual testing sessions
3. After sustained zero-divergence (aim for weeks of testing):
   - Delete the `_c` suffix functions
   - Remove the shadow wrapper
   - Have `vim_regexec` call `rs_vim_regexec` directly

## Functions to Shadow

Apply shadow mode to each function as it's migrated:

| Function | Purpose | Priority |
|----------|---------|----------|
| `vim_regcomp` | Compile a regexp pattern | High |
| `vim_regexec` | Match against a single line | High |
| `vim_regexec_multi` | Multi-line match | High |
| `vim_regexec_nl` | Match with newline handling | High |
| `vim_regsub` | Substitute matched text | Medium |
| `vim_regsub_multi` | Multi-line substitute | Medium |
| `skip_regexp` | Skip past a regexp pattern | Low (stateless) |
| `skip_regexp_ex` | Extended skip with magic detection | Low (stateless) |

## Compile-Time Control

Use a preprocessor flag to enable/disable shadow mode:

```c
#ifdef REGEXP_SHADOW_MODE
#define vim_regexec vim_regexec_shadow
#endif
```

This allows:
- **Debug builds**: Shadow mode ON (catches divergence immediately)
- **Release builds**: Shadow mode OFF (no performance cost)
- **CI**: Shadow mode ON (every test run validates equivalence)

## Performance Considerations

Shadow mode doubles the regexp work. In practice:
- Regexp is fast; doubling it is barely noticeable for interactive use
- CI runs are the primary validation environment
- For performance-sensitive benchmarks, disable shadow mode

## Logging Divergences

Instead of crashing immediately on divergence, an alternative is to log
divergences to a file for batch analysis:

```c
static FILE *shadow_log = NULL;

void shadow_log_divergence(const char *func, const char *pattern,
                           const char *input, int c_result, int rs_result)
{
    if (!shadow_log) {
        shadow_log = fopen("regexp_shadow.log", "a");
    }
    if (shadow_log) {
        fprintf(shadow_log, "%s: pattern=\"%s\" input=\"%s\" c=%d rust=%d\n",
                func, pattern, input, c_result, rs_result);
        fflush(shadow_log);
    }
}
```

This is useful during early development when divergences are expected and you
want to see the full picture before fixing them one by one.

## Relationship to Other Testing

Shadow mode complements (does not replace) other testing:

- **`regexp_corpus.json`**: Pre-computed expected results for offline testing
- **`compare_regexp.c`**: FFI comparison for stateless utility functions
- **`cargo fuzz`**: Randomized input testing for crashes and panics
- **Shadow mode**: Live validation during actual Neovim usage

Together these form a defense-in-depth strategy: corpus tests catch known
regressions, fuzzing finds unknown edge cases, and shadow mode catches
behavioral divergence in real-world usage patterns that no test suite can
fully anticipate.
