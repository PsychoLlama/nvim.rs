# Plan: Eliminate Wrapper C Files — Phases 2-4

Phase 1 is complete (base64.c, linematch.c, arabic.c, input.c, ugrid.c deleted = 222 lines).

## Pattern (same as Phase 1)

For each function `foo()` in the C file that just calls `rs_foo()`:
1. Add `#[export_name = "foo"]` to the Rust function (replacing `#[no_mangle]`)
2. Add `extern` declaration to the `.h` header file
3. Delete the `.c` file

Look at the Phase 1 commit (HEAD) for the exact pattern used.

## Phase 2: Pure Wrapper Files (~214 lines)

### `garray.c` (128 lines) — 100% pure wrappers
Every function just calls `rs_*`. Rust code is in `collections/src/`.
Functions: `ga_clear`, `ga_clear_strings`, `ga_init`, `ga_set_growsize`, `ga_grow`, 
`ga_remove_duplicate_strings`, `ga_concat_strings`, `ga_concat`, `ga_concat_len`, 
`ga_append`, `ga_append_via_ptr`

### `math.c` (86 lines) — wrappers + one conversion function
Most functions are pure wrappers. **BUT** `xfpclassify` has a `rs_fp_to_libc()` helper 
that maps Rust FP constants to C libc FP constants (FP_NAN, FP_INFINITE, etc.).
**FIX**: Modify the Rust `rs_xfpclassify` to return the libc constant directly 
(use libc crate or define the constants). Then all functions become pure wrappers.
Rust code is in `math/src/lib.rs`.
Functions: `xfpclassify`, `xisinf`, `xisnan`, `xctz`, `xpopcount`, `vim_append_digit_int`, `trim_to_int`

## Phase 3: Mostly Wrapper Files (~237 lines)

### `hashtab.c` (237 lines) — mostly wrappers
Most functions are pure wrappers. Non-wrapper items:
- `hash_removed` — a global `char` variable. Move to Rust or keep a tiny C file.
- `hash_add()` — calls `hash_hash()` then `hash_lookup()` then `hash_add_item()`, 
  plus error message. Migrate this to Rust.  
- `hash_debug_results()` — debug-only function (inside `#ifdef HT_DEBUG`). Can be 
  removed or migrated.
- `HASH_CYCLE_BODY` macro — unused after wrapper removal.
Rust code is in `collections/src/`.

## Phase 4: Larger Wrapper-Heavy Files

### `context.c` (301 lines)
Check if all functions are wrappers. If so, apply the same pattern.

### `extmark.c` (313 lines) 
Check if all functions are wrappers. If so, apply the same pattern.

### `highlight.c` (321 lines)
Has 57 function definitions with 56 wrapper calls — nearly all wrappers. Apply pattern.

### `keycodes.c` (348 lines)
17 functions, 30 wrapper calls — apply pattern.

### `cursor_shape.c` (300 lines)
31 functions, 12 wrapper calls — check if remaining are migrateable.

## Success Criteria
- All targeted C files from Phases 2-3 deleted (garray.c, math.c, hashtab.c)
- As many Phase 4 files as possible deleted
- `just check` passes after each phase
- Commit after each successful phase
- Target: >= 800 lines C deleted in Phases 2-3, more from Phase 4
