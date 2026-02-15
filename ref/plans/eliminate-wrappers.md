# Plan: Eliminate Thin Wrapper C Files

## Goal
Delete C files that are 100% thin wrappers forwarding to Rust `rs_*` functions.
Target: **~3000+ lines of C deleted** across 15+ files.

## Approach

For each wrapper C file:

1. **Rename Rust functions**: Change `#[no_mangle] pub extern "C" fn rs_foo(...)` to `#[export_name = "foo"] pub extern "C" fn rs_foo(...)` — this exports the symbol as `foo` instead of `rs_foo`, making the C wrapper unnecessary.

2. **Update the `.h` header**: The build system generates `foo.h.generated.h` by scanning the C file. When we delete the C file, those prototypes vanish. Move the function declarations into the manual `foo.h` header (remove the `#include "foo.h.generated.h"` if it becomes empty, or keep it).

3. **Delete the `.c` file**: CMake uses `file(GLOB *.c)` so no CMake changes needed.

4. **Update cbindgen if needed**: If the function appears in `src/nvim-rs/cbindgen.toml`'s export list, update accordingly.

5. **Build and test after each batch**.

## Phases

### Phase 1: Tiny Pure Wrappers (~170 lines)
Files that are 100% wrappers with zero logic:

| File | Lines | Functions |
|------|-------|-----------|
| `base64.c` | 36 | `base64_encode`, `base64_decode` — Rust in `encoding/src/base64.rs` |
| `linematch.c` | 44 | `fastforward_buf_to_lnum`, `linematch_nbuffers` — Rust in `linematch/src/lib.rs` |
| `arabic.c` | 46 | `arabic_maycombine`, `arabic_combine`, `arabic_shape` — Rust in `arabic/src/lib.rs` |
| `input.c` | 44 | `ask_yesno`, `get_keystroke`, `prompt_for_input` — Rust in `input/src/lib.rs` |
| `ugrid.c` | 52 | `ugrid_init/free/resize/clear/clear_chunk/goto/scroll` — Rust in `ugrid/src/lib.rs` |

### Phase 2: Small Wrappers (~460 lines)

| File | Lines | Functions |
|------|-------|-----------|
| `math.c` | 86 | `xfpclassify`, `xisinf`, `xisnan`, `xctz`, `xpopcount`, `vim_append_digit_int`, `trim_to_int` — Rust in `math/src/lib.rs`. **Note**: `math.c` has an `rs_fp_to_libc` mapping function — move this logic into Rust. |
| `map.c` | 118 | `mh_realloc`, `mh_clear`, `pmap_del2` — Rust in `collections/src/` |
| `garray.c` | 128 | `ga_clear/init/grow/...` (9 functions) — Rust in `collections/src/` |
| `sha256.c` | 134 | `sha256_start/update/finish`, `sha256_self_test` — Rust in `encoding/src/` |

### Phase 3: Medium Wrappers (~1100 lines)

| File | Lines | Functions |
|------|-------|-----------|
| `clipboard.c` | 232 | 11 funcs — Rust in `clipboard/src/lib.rs` |
| `hashtab.c` | 237 | 9 funcs — Rust in `collections/src/` |
| `context.c` | 301 | 16 funcs — Rust in `context/src/lib.rs` |
| `extmark.c` | 313 | 21 funcs — Rust in `extmark/src/lib.rs` |

### Phase 4: Larger Wrappers (~1300 lines)

| File | Lines | Functions |
|------|-------|-----------|
| `highlight.c` | 321 | 57 func wrappers — Rust in `highlight/src/lib.rs` |
| `keycodes.c` | 348 | 17 func wrappers — Rust in `keycodes/src/lib.rs` |
| `cursor_shape.c` | 300 | 31 func wrappers — Rust in `cursor_shape/src/lib.rs` |
| `fuzzy.c` | 413 | 11 funcs — Rust in `fuzzy/src/lib.rs` |

## Important Details

- The `input.c` file has `_Static_assert` checks — move these to a test or keep a minimal C file with just the asserts if needed.
- The `math.c` file has `rs_fp_to_libc()` conversion logic — this MUST be moved into Rust before deleting.
- The `.h` files may include `foo.h.generated.h` — when the C file is gone, the generated header will be empty. Add manual `extern` declarations to the `.h` file instead.
- Some `.h` files use `FUNC_ATTR_PURE`, `FUNC_ATTR_CONST`, etc. These can be omitted in the manual declarations (they're for static analysis only).
- Run `just check` (build + test + lint) after each phase.
- Commit after each successful phase.

## Success Criteria
- All targeted C files deleted
- `just check` passes (build, smoke tests, formatting, clippy)
- Net C line reduction >= 2500 lines
