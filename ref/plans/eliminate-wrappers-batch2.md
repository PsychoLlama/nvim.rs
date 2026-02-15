# Plan: Eliminate Wrapper Functions — Batch 2

## Goal
Delete wrapper functions from C files that forward to Rust `rs_*` functions.
Previous batch deleted 7 C files and reduced 3 more. This batch targets the 
next tier of files with significant wrapper content.

## Pattern (same as before — see recent commits)

For each wrapper function `foo()` that just calls `rs_foo()`:
1. In Rust: Replace `#[no_mangle]` with `#[export_name = "foo"]` on the `rs_foo` function
2. In the `.h` header: Add `extern` declaration for `foo()`
3. Delete the wrapper function from the `.c` file
4. If the `.c` file becomes empty (or just has includes/static_asserts), delete it entirely

Build with `just build` after each file. Run `just check` after each commit.

## Target Files

### Tier 1: Heavy wrapper files (~1600 lines total)

**cursor.c (568 lines)**
- 18 wrapper functions call rs_*: getviscol, getviscol2, coladvance_force, coladvance, 
  getvpos, get_cursor_rel_lnum, check_pos, check_cursor_lnum, check_cursor_col, 
  check_cursor, check_visual_pos, adjust_cursor_col, set_leftcol, gchar_cursor, 
  char_before_cursor, pchar_cursor, get_cursor_line_len, get_cursor_pos_len
- Keep: coladvance2 (static, real logic), inc_cursor, dec_cursor (call inc/dec from memline)
- Keep: ~28 nvim_* accessor functions (needed by Rust)
- Expected: ~200 lines of wrappers deletable

**change.c (604 lines)**
- ~25 wrapper functions: change_warning, changed, changed_internal, 
  changed_lines_invalidate_buf, changed_bytes, inserted_bytes, appended_lines_buf,
  appended_lines, appended_lines_mark, deleted_lines_buf, deleted_lines, 
  deleted_lines_mark, changed_lines_redraw_buf, changed_lines, unchanged, 
  save_file_ff, file_ff_differs, ins_bytes, ins_bytes_len, ins_char, ins_char_bytes,
  ins_str, del_char, del_chars, del_bytes, open_line, truncate_line, del_lines,
  get_leader_len, get_last_leader_offset
- Keep: changed_common (real logic, ~180 lines), changed_lines_invalidate_win (static, ~55 lines)
- Expected: ~350 lines of wrappers deletable

**move.c (826 lines)**  
- ~54 rs_ calls, many wrapper functions
- Check each function before eliminating
- Expected: ~300+ lines of wrappers deletable

**indent.c (581 lines)**
- ~32 rs_ calls, wrapper functions
- Expected: ~200 lines of wrappers deletable

### Tier 2: Medium wrapper files (~800 lines total)

**help.c (325 lines)** — 14 wrapper calls
**cmdhist.c (555 lines)** — 13 wrapper calls  
**charset.c (929 lines)** — 43 wrapper calls

## Important Notes
- Only delete functions that are PURE wrappers (body is just `return rs_foo(...)` or `rs_foo(...)`)
- Do NOT delete accessor functions (nvim_* pattern) — Rust needs these
- Do NOT delete functions with real logic (even one line beyond the rs_ call)
- If a function has `FUNC_ATTR_*` annotations, those can be dropped in the header declaration
- Commit after completing each file or pair of files
- Run `just check` to validate after each commit

## Success Criteria
- All wrapper functions eliminated from target files
- `just check` passes (build, smoke tests, formatting, clippy, tests)
- Target: >= 1000 net lines of C deleted
