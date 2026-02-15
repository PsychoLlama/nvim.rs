/// indent_ffi.c: C accessor wrappers for the Rust indent crate (nvim-indent).
///
/// These thin wrappers provide a stable C ABI for Rust code to call into
/// Neovim's C internals for indentation operations.

#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/extmark.h"
#include "nvim/extmark_defs.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/message.h"
#include "nvim/memory.h"
#include "nvim/move.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/regexp.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/textformat.h"
#include "nvim/types_defs.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"

#include "indent_ffi.c.generated.h"

// =============================================================================
// Static assertions for constants used in Rust code
// =============================================================================

_Static_assert(SIN_CHANGED == 1, "SIN_CHANGED must be 1");
_Static_assert(SIN_INSERT == 2, "SIN_INSERT must be 2");
_Static_assert(SIN_UNDO == 4, "SIN_UNDO must be 4");
_Static_assert(SIN_NOMARK == 8, "SIN_NOMARK must be 8");
_Static_assert(VREPLACE_FLAG == 0x200, "VREPLACE_FLAG must be 0x200");
_Static_assert(INDENT_SET == 1, "INDENT_SET must be 1");
_Static_assert(BL_SOL == 2, "BL_SOL must be 2");
_Static_assert(BL_FIX == 4, "BL_FIX must be 4");
_Static_assert(UPD_INVERTED == 20, "UPD_INVERTED must be 20");
_Static_assert(CMOD_LOCKMARKS == 0x0800, "CMOD_LOCKMARKS must be 0x0800");

// =============================================================================
// Phase 1: set_indent() accessors
// =============================================================================

bool nvim_curbuf_get_p_et(void) { return curbuf->b_p_et; }

int nvim_u_savesub_curline(void) { return u_savesub(curwin->w_cursor.lnum); }

linenr_T nvim_get_saved_cursor_lnum(void) { return saved_cursor.lnum; }
colnr_T nvim_get_saved_cursor_col(void) { return saved_cursor.col; }
void nvim_set_saved_cursor_col(colnr_T val) { saved_cursor.col = val; }

// =============================================================================
// Phase 3: get_breakindent_win() accessors
// =============================================================================

// These accessors already exist in other files (window.c, move.c, message.c, option.c):
//   nvim_win_get_view_width, nvim_win_get_buffer, nvim_win_get_p_list,
//   nvim_win_get_lcs_tab1, nvim_win_get_briopt_sbr, nvim_win_get_briopt_list,
//   nvim_win_col_off, nvim_win_col_off2, nvim_vim_strsize

int nvim_win_get_briopt_shift(win_T *wp) { return wp->w_briopt_shift; }
int nvim_win_get_briopt_min(win_T *wp) { return wp->w_briopt_min; }
int nvim_win_get_briopt_vcol(win_T *wp) { return wp->w_briopt_vcol; }

int nvim_buf_get_b_fnum(buf_T *buf) { return buf->b_fnum; }
int64_t nvim_indent_buf_get_changedtick(buf_T *buf) { return buf_get_changedtick(buf); }
const char *nvim_get_flp_value(buf_T *buf) { return get_flp_value(buf); }

unsigned nvim_get_dy_flags_uhex(void) { return dy_flags & kOptDyFlagUhex; }

const char *nvim_indent_get_showbreak_value(win_T *wp) { return get_showbreak_value(wp); }

/// Higher-level regex helper for breakindent: match formatlistpat against line.
///
/// Returns 1 if match found, 0 if no match. On match with list < 0, computes
/// the width of the matched text and stores it in *out_width.
/// On match with list > 0, *out_width is not modified.
int nvim_breakindent_flp_match(win_T *wp, const char *pat, const char *line,
                               int briopt_list, int *out_width)
{
  regmatch_T regmatch = {
    .regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING + RE_AUTO + RE_STRICT),
  };
  if (regmatch.regprog == NULL) {
    return 0;
  }
  regmatch.rm_ic = false;
  int matched = 0;
  if (vim_regexec(&regmatch, line, 0)) {
    matched = 1;
    if (briopt_list < 0) {
      // Compute width of matched text
      char *ptr = *regmatch.startp;
      char *end_ptr = *regmatch.endp;
      int indent = 0;
      while (ptr < end_ptr) {
        indent += win_chartabsize(wp, ptr, indent);
        MB_PTR_ADV(ptr);
      }
      *out_width = indent;
    }
  }
  vim_regfree(regmatch.regprog);
  return matched;
}

// =============================================================================
// Phase 4: ins_try_si() accessors
// =============================================================================

// These accessors already exist in other files:
//   nvim_get_did_si, nvim_get_can_si, nvim_get_can_si_back (change_ffi.c)
//   nvim_get_ai_col, nvim_set_ai_col (change_ffi.c)
//   nvim_get_State (window.c)
//   nvim_ml_get, nvim_skipwhite (change_ffi.c, fold.c)
//   nvim_findmatch, nvim_change_get_curwin_cursor, nvim_set_curwin_cursor (change_ffi.c)
//   nvim_get_curwin_cursor_col, nvim_get_curwin_cursor_lnum (change_ffi.c, insexpand.c)
//   nvim_set_curwin_cursor_lnum (change_ffi.c)

void nvim_shift_line(bool left, bool round, int amount, int call_changed_bytes)
{
  shift_line(left, round, amount, call_changed_bytes);
}

int nvim_get_old_indent(void) { return old_indent; }
void nvim_set_old_indent(int val) { old_indent = val; }

void nvim_change_indent(int type, int amount, int round, bool call_changed_bytes)
{
  change_indent(type, amount, round, call_changed_bytes);
}

// =============================================================================
// Phase 5: op_reindent() accessors
// =============================================================================

// Existing accessors from other files:
//   nvim_get_curwin_cursor_lnum, nvim_set_curwin_cursor_lnum (change_ffi.c/insexpand.c)
//   nvim_set_curwin_cursor_col (change_ffi.c)
//   nvim_skipwhite (fold.c)
//   nvim_get_cursor_line_ptr (change_ffi.c)
//   nvim_beginline (normal.c)
//   nvim_redraw_curbuf_later (fold.c)
//   nvim_emsg_modifiable (undo.c)
//   nvim_get_got_int (ex_eval.c)
//   nvim_oap_get_line_count (normal.c)

bool nvim_curbuf_is_modifiable(void) { return MODIFIABLE(curbuf); }

int nvim_u_savecommon_range(linenr_T start, linenr_T count)
{
  return u_savecommon(curbuf, start - 1, start + count, start + count, false);
}

void nvim_smsg_lines_to_indent(int64_t i)
{
  smsg(0, _("%" PRId64 " lines to indent... "), i);
}

void nvim_smsg_lines_indented(int64_t count)
{
  smsg(0, NGETTEXT("%" PRId64 " line indented ",
                    "%" PRId64 " lines indented ", (int)count), count);
}

int64_t nvim_get_p_report(void) { return p_report; }
bool nvim_get_cmdmod_lockmarks(void) { return (cmdmod.cmod_flags & CMOD_LOCKMARKS) != 0; }

void nvim_indent_changed_lines(linenr_T first, linenr_T last, linenr_T xtra)
{
  changed_lines(curbuf, first, 0, last, xtra, true);
}

// oparg_T field accessors
bool nvim_oap_is_visual(oparg_T *oap) { return oap->is_VIsual; }
void nvim_oap_set_marks(oparg_T *oap)
{
  curbuf->b_op_start = oap->start;
  curbuf->b_op_end = oap->end;
}

/// Compare a function pointer to get_lisp_indent.
bool nvim_is_lisp_indent(Indenter how) { return how == get_lisp_indent; }

// =============================================================================
// Phase 6: change_indent() accessors
// =============================================================================

_Static_assert(MODE_INSERT == 0x10, "MODE_INSERT must be 0x10");
_Static_assert(REPLACE_FLAG == 0x100, "REPLACE_FLAG must be 0x100");
_Static_assert(MAXCOL == 0x7fffffff, "MAXCOL must be 0x7fffffff");
_Static_assert(BL_WHITE == 1, "BL_WHITE must be 1");
_Static_assert(INDENT_DEC == 3, "INDENT_DEC must be 3");

// Insstart col-only setter (lnum+col setter already in edit.c)
void nvim_set_Insstart_col(colnr_T val) { Insstart.col = val; }

// curwin->w_p_list setter
void nvim_curwin_set_w_p_list(int val) { curwin->w_p_list = val; }

// curwin->w_set_curswant setter
void nvim_curwin_set_w_set_curswant(bool val) { curwin->w_set_curswant = val; }

// curwin->w_virtcol setter
void nvim_curwin_set_w_virtcol(colnr_T val) { curwin->w_virtcol = val; }

// ml_replace for curwin cursor line
int nvim_ml_replace_curline(char *line, bool copy)
{
  return ml_replace(curwin->w_cursor.lnum, line, copy);
}

// ins_str wrapper: inserts `len` bytes from `ptr` at cursor
void nvim_ins_str(char *ptr, size_t len)
{
  ins_str(ptr, len);
}

/// Advance cursor in `line` until reaching `target_vcol`.
/// Returns the byte offset in line, and writes final vcol to *out_vcol.
int nvim_advance_to_vcol(char *line, int target_vcol, int *out_vcol)
{
  int vcol = 0;
  int new_cursor_col = 0;
  if (*line != NUL) {
    CharsizeArg csarg;
    CSType cstype = init_charsize_arg(&csarg, curwin, 0, line);
    StrCharInfo ci = utf_ptr2StrCharInfo(line);
    while (true) {
      int next_vcol = vcol + win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &csarg).width;
      if (next_vcol > target_vcol) {
        break;
      }
      vcol = next_vcol;
      ci = utfc_next(ci);
      if (*ci.ptr == NUL) {
        break;
      }
    }
    new_cursor_col = (int)(ci.ptr - line);
  }
  *out_vcol = vcol;
  return new_cursor_col;
}

// Get curbuf handle for extmark_splice_cols
buf_T *nvim_indent_get_curbuf(void) { return curbuf; }

// =============================================================================
// Phase 7: ex_retab() accessors
// =============================================================================

_Static_assert(UPD_NOT_VALID == 40, "UPD_NOT_VALID must be 40");

bool nvim_eap_get_forceit(const exarg_T *eap) { return eap->forceit; }

int nvim_u_save(linenr_T top, linenr_T bot)
{
  return u_save(top, bot);
}

colnr_T nvim_indent_win_chartabsize(const char *ptr, colnr_T vcol)
{
  return win_chartabsize(curwin, ptr, vcol);
}

colnr_T nvim_curwin_get_w_curswant(void) { return (colnr_T)curwin->w_curswant; }

OptInt nvim_retab_curbuf_get_p_ts(void) { return curbuf->b_p_ts; }
int *nvim_retab_curbuf_get_p_vts_array(void) { return curbuf->b_p_vts_array; }

void nvim_retab_curbuf_set_p_ts(OptInt val) { curbuf->b_p_ts = val; }
void nvim_retab_curbuf_set_p_vts_array(int *val) { curbuf->b_p_vts_array = val; }

void nvim_set_option_direct_vts(const char *str)
{
  set_option_direct(kOptVartabstop, CSTR_AS_OPTVAL(str), OPT_LOCAL, 0);
}

void nvim_emsg_interr(void)
{
  emsg(_(e_interr));
}

// =============================================================================
// Phase 8: get_lisp_indent() accessors
// =============================================================================

/// Compute vcol of the character at byte offset `col` in `line`, using
/// the charsize machinery for window `wp` at line `lnum`.
/// Also returns the pointer to the character at that column via *out_ptr.
int nvim_lisp_vcol_at_col(linenr_T lnum, char *line, colnr_T col, char **out_ptr)
{
  CharsizeArg csarg;
  CSType cstype = init_charsize_arg(&csarg, curwin, lnum, line);
  StrCharInfo sci = utf_ptr2StrCharInfo(line);
  int amount = 0;
  colnr_T c = col;
  while (*sci.ptr != NUL && c > 0) {
    amount += win_charsize(cstype, amount, sci.ptr, sci.chr.value, &csarg).width;
    sci = utfc_next(sci);
    c--;
  }
  *out_ptr = sci.ptr;
  return amount;
}

/// Compute the width of whitespace characters starting at `ptr` in the
/// current charsize context (same line as the previous nvim_lisp_vcol_at_col
/// call). Returns the total vcol after advancing past whitespace, and
/// updates *out_ptr to point past the whitespace.
///
/// This is a higher-level helper that takes the current vcol `amount`,
/// the line `line` and start position `lnum`, and advances past ASCII
/// whitespace at `ptr`.
int nvim_lisp_skip_whitespace(linenr_T lnum, char *line, int amount, char *ptr, char **out_ptr)
{
  CharsizeArg csarg;
  CSType cstype = init_charsize_arg(&csarg, curwin, lnum, line);
  int vcol = amount;
  char *p = ptr;
  while (ascii_iswhite(*p)) {
    vcol += win_charsize(cstype, vcol, p, (uint8_t)(*p), &csarg).width;
    p++;
  }
  *out_ptr = p;
  return vcol;
}

/// Walk a "word" in lisp (non-whitespace with paren/quote tracking).
/// Starting at `ptr` with given `amount` vcol, advance past the word.
/// Returns the final vcol. Updates *out_ptr to point past the word.
int nvim_lisp_skip_word(linenr_T lnum, char *line, int amount, char *ptr, char **out_ptr)
{
  CharsizeArg csarg;
  CSType cstype = init_charsize_arg(&csarg, curwin, lnum, line);
  int vcol = amount;
  char *that = ptr;

  CharInfo ci = utf_ptr2CharInfo(that);
  int quotecount = 0;
  int parencount = 0;
  while (*that && (!ascii_iswhite(ci.value) || quotecount || parencount)) {
    if (ci.value == '"') {
      quotecount = !quotecount;
    }
    if (((ci.value == '(') || (ci.value == '[')) && !quotecount) {
      parencount++;
    }
    if (((ci.value == ')') || (ci.value == ']')) && !quotecount) {
      parencount--;
    }
    if ((ci.value == '\\') && (*(that + 1) != NUL)) {
      vcol += win_charsize(cstype, vcol, that, ci.value, &csarg).width;
      StrCharInfo next_sci = utfc_next((StrCharInfo){ that, ci });
      that = next_sci.ptr;
      ci = next_sci.chr;
    }

    vcol += win_charsize(cstype, vcol, that, ci.value, &csarg).width;
    StrCharInfo next_sci = utfc_next((StrCharInfo){ that, ci });
    that = next_sci.ptr;
    ci = next_sci.chr;
  }
  *out_ptr = that;
  return vcol;
}

/// Get the character value at ptr (for checking against '"', '\'', '#', digits).
int nvim_utf_ptr2char_value(const char *ptr) { return utf_ptr2CharInfo(ptr).value; }
