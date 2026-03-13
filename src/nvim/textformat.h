#pragma once

#include "nvim/normal_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep

// Forward declarations for Rust-implemented functions (exported under C names via #[export_name])
bool has_format_option(int x);
void internal_format(int textwidth, int second_indent, int flags, bool format_only, int c);
void auto_format(bool trailblank, bool prev_line);
void check_auto_format(bool end_insert);
int comp_textwidth(bool ff);
void op_format(oparg_T *oap, bool keep_cursor);
void op_formatexpr(oparg_T *oap);
int fex_format(linenr_T lnum, long count, int c);
void format_lines(linenr_T line_count, bool avoid_fex);

#include "textformat.h.generated.h"
