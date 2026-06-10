#include <inttypes.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/edit.h"
#include "nvim/eval/typval.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/indent_c.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/mark_defs.h"
#include "nvim/math.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/search.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

extern const char *rs_skip_to_option_part(const char *p);
extern bool cin_iscase(const char *s, bool strict);

/// C accessor for p_paste global option.
int nvim_get_p_paste(void) { return p_paste; }

/// C accessor for curbuf->b_p_cin (cindent option).
int nvim_curbuf_get_p_cin(void) { return curbuf->b_p_cin; }

/// C accessor for whether curbuf->b_p_inde is non-empty.
int nvim_curbuf_get_inde_nonempty(void) { return *curbuf->b_p_inde != NUL; }

/// C accessor for curbuf->b_p_si (smartindent option).
int nvim_curbuf_get_p_si(void) { return curbuf->b_p_si; }

/// C accessor for curbuf->b_ind_hash_comment (# comment indentation).
int nvim_curbuf_get_ind_hash_comment(void) { return curbuf->b_ind_hash_comment; }

/// C accessor for curbuf->b_p_lisp (lisp option).
int nvim_curbuf_get_p_lisp(void) { return curbuf->b_p_lisp; }

/// C accessor for curbuf->b_p_inde (indentexpr) as pointer.
const char *nvim_curbuf_get_inde_ptr(void) { return curbuf->b_p_inde; }

/// C accessor for curbuf->b_p_lop (lispoptions).
const char *nvim_curbuf_get_p_lop(void) { return curbuf->b_p_lop; }

/// C accessor for curbuf->b_p_lw (lispwords local).
const char *nvim_curbuf_get_p_lw(void) { return curbuf->b_p_lw; }

/// C accessor for global p_lispwords.
const char *nvim_get_p_lispwords(void) { return p_lispwords; }

/// C accessor for in_cinkeys function (for Rust FFI).
bool nvim_in_cinkeys(int keytyped, int when, bool line_is_empty) { return in_cinkeys(keytyped, when, line_is_empty); }

/// C accessor for curbuf->b_p_indk (indentkeys option).
const char *nvim_cindent_curbuf_get_indk(void) { return curbuf->b_p_indk; }

/// C accessor for curbuf->b_p_cink (cinkeys option).
const char *nvim_cindent_curbuf_get_cink(void) { return curbuf->b_p_cink; }

/// C accessor for get_cursor_pos_ptr() (pointer to cursor position in line).
const char *nvim_cindent_get_cursor_pos_ptr(void) { return get_cursor_pos_ptr(); }

/// C accessor for get_special_key_code(look) -- returns keycode for named key.
int nvim_cindent_get_special_key_code(const char *look) { return get_special_key_code(look); }

/// C accessor for getwhitecols_curline().
int nvim_cindent_getwhitecols_curline(void) { return (int)getwhitecols_curline(); }

/// C accessor for vim_iswordp(p).
int nvim_cindent_vim_iswordp(const char *p) { return vim_iswordp(p); }

/// C accessor for mb_prevptr(line, p) -- previous multibyte char pointer.
const char *nvim_cindent_mb_prevptr(const char *line, const char *p) { return mb_prevptr((char *)line, (char *)p); }

/// C accessor for get_cursor_line_ptr() as mutable (for temporary write in ':' case).
char *nvim_cindent_get_cursor_line_ptr_mut(void) { return (char *)get_cursor_line_ptr(); }

// Phase 2 C accessors
/// C accessor for curwin->w_cursor.lnum.
int nvim_cindent_curwin_get_cursor_lnum(void) { return curwin->w_cursor.lnum; }

/// C accessor for curbuf->b_ind_maxparen.
int nvim_cindent_curbuf_get_ind_maxparen(void) { return curbuf->b_ind_maxparen; }

/// C accessor for curbuf->b_p_cinw (cinwords option).
const char *nvim_cindent_curbuf_get_cinw(void) { return curbuf->b_p_cinw; }

/// C accessor for ml_get(lnum).
const char *nvim_cindent_ml_get(int lnum) { return ml_get(lnum); }

/// C accessor for get_indent_lnum(lnum).
int nvim_cindent_get_indent_lnum(int lnum) { return get_indent_lnum(lnum); }

extern void rs_parse_cino(const char *cino, int sw, CindentOptions *opts);

// Phase 2 bulk accessors for parse_cino / get_c_indent migration.

/// Bulk setter: copy all CindentOptions fields into buf->b_ind_* fields.
void nvim_cindent_buf_set_ind_fields(void *buf_, const CindentOptions *opts)
{
  buf_T *buf = buf_;
  buf->b_ind_level = opts->ind_level;
  buf->b_ind_open_imag = opts->ind_open_imag;
  buf->b_ind_no_brace = opts->ind_no_brace;
  buf->b_ind_first_open = opts->ind_first_open;
  buf->b_ind_open_extra = opts->ind_open_extra;
  buf->b_ind_close_extra = opts->ind_close_extra;
  buf->b_ind_open_left_imag = opts->ind_open_left_imag;
  buf->b_ind_jump_label = opts->ind_jump_label;
  buf->b_ind_case = opts->ind_case;
  buf->b_ind_case_code = opts->ind_case_code;
  buf->b_ind_case_break = opts->ind_case_break;
  buf->b_ind_scopedecl = opts->ind_scopedecl;
  buf->b_ind_scopedecl_code = opts->ind_scopedecl_code;
  buf->b_ind_param = opts->ind_param;
  buf->b_ind_func_type = opts->ind_func_type;
  buf->b_ind_cpp_baseclass = opts->ind_cpp_baseclass;
  buf->b_ind_continuation = opts->ind_continuation;
  buf->b_ind_unclosed = opts->ind_unclosed;
  buf->b_ind_unclosed2 = opts->ind_unclosed2;
  buf->b_ind_unclosed_noignore = opts->ind_unclosed_noignore;
  buf->b_ind_unclosed_wrapped = opts->ind_unclosed_wrapped;
  buf->b_ind_unclosed_whiteok = opts->ind_unclosed_whiteok;
  buf->b_ind_matching_paren = opts->ind_matching_paren;
  buf->b_ind_paren_prev = opts->ind_paren_prev;
  buf->b_ind_comment = opts->ind_comment;
  buf->b_ind_in_comment = opts->ind_in_comment;
  buf->b_ind_in_comment2 = opts->ind_in_comment2;
  buf->b_ind_maxparen = opts->ind_maxparen;
  buf->b_ind_maxcomment = opts->ind_maxcomment;
  buf->b_ind_java = opts->ind_java;
  buf->b_ind_js = opts->ind_js;
  buf->b_ind_keep_case_label = opts->ind_keep_case_label;
  buf->b_ind_cpp_namespace = opts->ind_cpp_namespace;
  buf->b_ind_if_for_while = opts->ind_if_for_while;
  buf->b_ind_hash_comment = opts->ind_hash_comment;
  buf->b_ind_cpp_extern_c = opts->ind_cpp_extern_c;
  buf->b_ind_pragma = opts->ind_pragma;
}

/// Bulk getter: copy all curbuf->b_ind_* fields into opts.
void nvim_cindent_curbuf_get_ind_opts(CindentOptions *opts)
{
  opts->ind_level = curbuf->b_ind_level;
  opts->ind_open_imag = curbuf->b_ind_open_imag;
  opts->ind_no_brace = curbuf->b_ind_no_brace;
  opts->ind_first_open = curbuf->b_ind_first_open;
  opts->ind_open_extra = curbuf->b_ind_open_extra;
  opts->ind_close_extra = curbuf->b_ind_close_extra;
  opts->ind_open_left_imag = curbuf->b_ind_open_left_imag;
  opts->ind_jump_label = curbuf->b_ind_jump_label;
  opts->ind_case = curbuf->b_ind_case;
  opts->ind_case_code = curbuf->b_ind_case_code;
  opts->ind_case_break = curbuf->b_ind_case_break;
  opts->ind_scopedecl = curbuf->b_ind_scopedecl;
  opts->ind_scopedecl_code = curbuf->b_ind_scopedecl_code;
  opts->ind_param = curbuf->b_ind_param;
  opts->ind_func_type = curbuf->b_ind_func_type;
  opts->ind_cpp_baseclass = curbuf->b_ind_cpp_baseclass;
  opts->ind_continuation = curbuf->b_ind_continuation;
  opts->ind_unclosed = curbuf->b_ind_unclosed;
  opts->ind_unclosed2 = curbuf->b_ind_unclosed2;
  opts->ind_unclosed_noignore = curbuf->b_ind_unclosed_noignore;
  opts->ind_unclosed_wrapped = curbuf->b_ind_unclosed_wrapped;
  opts->ind_unclosed_whiteok = curbuf->b_ind_unclosed_whiteok;
  opts->ind_matching_paren = curbuf->b_ind_matching_paren;
  opts->ind_paren_prev = curbuf->b_ind_paren_prev;
  opts->ind_comment = curbuf->b_ind_comment;
  opts->ind_in_comment = curbuf->b_ind_in_comment;
  opts->ind_in_comment2 = curbuf->b_ind_in_comment2;
  opts->ind_maxparen = curbuf->b_ind_maxparen;
  opts->ind_maxcomment = curbuf->b_ind_maxcomment;
  opts->ind_java = curbuf->b_ind_java;
  opts->ind_js = curbuf->b_ind_js;
  opts->ind_keep_case_label = curbuf->b_ind_keep_case_label;
  opts->ind_cpp_namespace = curbuf->b_ind_cpp_namespace;
  opts->ind_if_for_while = curbuf->b_ind_if_for_while;
  opts->ind_hash_comment = curbuf->b_ind_hash_comment;
  opts->ind_cpp_extern_c = curbuf->b_ind_cpp_extern_c;
  opts->ind_pragma = curbuf->b_ind_pragma;
}

/// Get buf->b_p_cino (cinoptions string) for the given buffer.
const char *nvim_cindent_buf_get_p_cino(void *buf_) { return ((buf_T *)buf_)->b_p_cino; }

/// Get get_sw_value(buf) -- shiftwidth for the given buffer.
int nvim_cindent_buf_get_sw_value(void *buf_) { return get_sw_value((buf_T *)buf_); }

/// C accessor for curbuf->b_p_cinsd (cinscopedecls option).
const char *nvim_cindent_curbuf_get_cinsd(void) { return curbuf->b_p_cinsd; }

// Phase 3 C accessors

/// C accessor for findmatchlimit, returning a copyable result.
FindMatchResult nvim_cindent_findmatchlimit(int what, int flags, int64_t maxtravel)
{
  FindMatchResult result = { false, 0, 0 };
  pos_T *pos = findmatchlimit(NULL, what, flags, maxtravel);
  if (pos != NULL) {
    result.found = true;
    result.lnum = pos->lnum;
    result.col = pos->col;
  }
  return result;
}

/// C accessor for ml_get_pos: get string at (lnum, col) and return pointer offset by col.
const char *nvim_cindent_ml_get_pos_lnum_col(int lnum, int col) { return ml_get(lnum) + col; }

/// C accessor for getvcol.
int nvim_cindent_getvcol(int lnum, int col)
{
  pos_T fp;
  colnr_T vcol;
  fp.lnum = lnum;
  fp.col = col;
  getvcol(curwin, &fp, &vcol, NULL, NULL);
  return (int)vcol;
}

/// C accessor for curwin->w_cursor.col.
int nvim_cindent_curwin_get_cursor_col(void) { return curwin->w_cursor.col; }

/// C accessor for curwin->w_cursor.coladd (read).
int nvim_cindent_curwin_get_cursor_coladd(void) { return curwin->w_cursor.coladd; }

/// C accessor to set curwin->w_cursor.coladd.
void nvim_cindent_curwin_set_cursor_coladd(int v) { curwin->w_cursor.coladd = v; }

/// C accessor to set curwin->w_cursor.
void nvim_cindent_curwin_set_cursor(int lnum, int col)
{
  curwin->w_cursor.lnum = lnum;
  curwin->w_cursor.col = col;
}

/// C accessor for curbuf->b_ind_maxcomment.
int nvim_cindent_curbuf_get_ind_maxcomment(void) { return curbuf->b_ind_maxcomment; }

/// C accessor for curbuf->b_ml.ml_line_count.
int nvim_cindent_curbuf_get_ml_line_count(void) { return curbuf->b_ml.ml_line_count; }

/// C accessor for curbuf->b_ind_cpp_baseclass.
int nvim_cindent_curbuf_get_ind_cpp_baseclass(void) { return curbuf->b_ind_cpp_baseclass; }

/// C accessor for get_indent().
int nvim_cindent_get_indent(void) { return get_indent(); }

/// C accessor for get_cursor_line_ptr().
const char *nvim_cindent_get_cursor_line_ptr(void) { return get_cursor_line_ptr(); }

extern FindMatchResult rs_find_start_comment(int ind_maxcomment);
extern bool cin_isscopedecl(const char *p);
extern bool cin_islabel(void);
extern int rs_get_c_indent(const CindentOptions *opts);

#include "indent_c.c.generated.h"

pos_T *find_start_comment(int ind_maxcomment)  // XXX
{
  static pos_T pos_copy;
  FindMatchResult result = rs_find_start_comment(ind_maxcomment);
  if (result.found) {
    pos_copy.lnum = result.lnum;
    pos_copy.col = result.col;
    return &pos_copy;
  }
  return NULL;
}

// Functions for C-indenting.
// Most of this originally comes from Eric Fischer.

// Below "XXX" means that this function may unlock the current line.

// parse_cino and get_c_indent are implemented in Rust
// (src/nvim-rs/indent_c/src/lib.rs) via #[unsafe(export_name)].
// The C bodies have been removed.

// in_cinkeys is implemented in Rust (src/nvim-rs/indent_c/src/lib.rs) via
// #[unsafe(export_name = "in_cinkeys")]. The C body has been removed.

// Do C or expression indenting on the current line.
void do_c_expr_indent(void)
{
  if (*curbuf->b_p_inde != NUL) {
    fixthisline(get_expr_indent);
  } else {
    fixthisline(get_c_indent);
  }
}

// f_cindent is implemented in Rust (src/nvim-rs/indent_c/src/lib.rs).
