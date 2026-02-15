# Wrapper Elimination Session (2026-02-15)

## Results
- **14 commits** eliminating thin C wrapper functions
- **3,728 deletions / 974 additions = ~2,754 net lines of C deleted**
- C codebase: 238,904 → **235,626 lines** (1.4% reduction)
- All 4,296 Rust tests pass, build clean, smoke tests pass

## Files Fully Deleted (9)
base64.c, linematch.c, arabic.c, input.c, ugrid.c, garray.c, math.c, hashtab.c, keycodes.c

## Files Substantially Reduced
- charset.c: 929→364 (-565 lines)
- move.c: 826→372 (-454 lines)  
- indent.c: 581→247 (-334 lines)
- change.c: 604→300 (-304 lines)
- cursor.c: 568→398 (-170 lines)
- highlight.c: 321→169 (-152 lines)
- extmark.c: 313→178 (-135 lines)
- help.c: 325→196 (-129 lines)
- cmdhist.c: 555→454 (-101 lines)
- context.c: 301→222 (-79 lines)
- cursor_shape.c: 300→272 (-28 lines)

## Pattern Used
`#[export_name = "foo"]` on Rust functions → delete C wrapper → add extern to .h header

## Next Targets
- More wrapper elimination in: arglist.c, state.c, digraph.c, testing.c, memfile.c, ui_compositor.c, ex_session.c
- Full file migration for nearly-done files (sha256.c, fuzzy.c)
- Attack big files: regexp.c (9316), quickfix.c (9108), normal.c (7908), window.c (7886), ex_docmd.c (7697)
