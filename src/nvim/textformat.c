// textformat.c: text formatting functions

#include <stdbool.h>
#include <stdint.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/eval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/getchar.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/indent_c.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/pos_defs.h"
#include "nvim/search.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/textformat.h"
#include "nvim/textobject.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

#include "textformat.c.generated.h"

// Rust implementations
extern int rs_has_format_option(int x);
extern int rs_ends_in_white(linenr_T lnum);
extern int rs_fmt_check_par(linenr_T lnum, int *leader_len, char **leader_flags, int do_comments);
extern int rs_same_leader(linenr_T lnum, int leader1_len, char *leader1_flags,
                          int leader2_len, char *leader2_flags);
extern int rs_paragraph_start(linenr_T lnum);
extern int rs_comp_textwidth(int ff);
extern void rs_op_format(oparg_T *oap, int keep_cursor);
extern void rs_op_formatexpr(oparg_T *oap);
extern void rs_auto_format(int trailblank, int prev_line);
extern void rs_check_auto_format(int end_insert);
extern int rs_fex_format(linenr_T lnum, long count, int c);
extern void rs_format_lines(linenr_T line_count, int avoid_fex);

/// C accessor for curbuf->b_p_fo (formatoptions).
char *nvim_get_curbuf_b_p_fo(void)
{
  return curbuf->b_p_fo;
}

// =============================================================================
// C Accessor Functions for Rust
// =============================================================================

/// Get line content at lnum (accessor for Rust).
char *nvim_textfmt_ml_get(linenr_T lnum)
{
  return ml_get(lnum);
}

/// Get line length at lnum (accessor for Rust).
colnr_T nvim_textfmt_ml_get_len(linenr_T lnum)
{
  return ml_get_len(lnum);
}

/// Get comment leader length (accessor for Rust).
int nvim_textfmt_get_leader_len(char *line, char **flags, bool backward, bool include_space)
{
  return get_leader_len(line, flags, backward, include_space);
}

/// Check if line starts a paragraph/section (accessor for Rust).
bool nvim_textfmt_startPS(linenr_T lnum, int para, bool both)
{
  return startPS(lnum, para, both);
}

/// Get number indent for a line (accessor for Rust).
int nvim_textfmt_get_number_indent(linenr_T lnum)
{
  return get_number_indent(lnum);
}

/// Get curbuf->b_p_tw (textwidth option, accessor for Rust).
int nvim_textfmt_get_curbuf_b_p_tw(void)
{
  return (int)curbuf->b_p_tw;
}

/// Get curbuf->b_p_wm (wrapmargin option, accessor for Rust).
int nvim_textfmt_get_curbuf_b_p_wm(void)
{
  return (int)curbuf->b_p_wm;
}

/// Get curwin->w_view_width (accessor for Rust).
int nvim_textfmt_get_curwin_w_view_width(void)
{
  return curwin->w_view_width;
}

/// Get curbuf pointer (accessor for Rust).
void *nvim_textfmt_get_curbuf(void)
{
  return curbuf;
}

/// Get cmdwin_buf pointer (accessor for Rust).
void *nvim_textfmt_get_cmdwin_buf(void)
{
  return cmdwin_buf;
}

/// Get curwin pointer (accessor for Rust).
void *nvim_textfmt_get_curwin(void)
{
  return curwin;
}

/// Get fold column count for window (accessor for Rust).
int nvim_textfmt_win_fdccol_count(void *win)
{
  return win_fdccol_count((win_T *)win);
}

/// Get curwin->w_scwidth (sign column width, accessor for Rust).
int nvim_textfmt_get_curwin_w_scwidth(void)
{
  return curwin->w_scwidth;
}

/// Get curwin->w_p_nu (number option, accessor for Rust).
bool nvim_textfmt_get_curwin_w_p_nu(void)
{
  return curwin->w_p_nu;
}

/// Get curwin->w_p_rnu (relativenumber option, accessor for Rust).
bool nvim_textfmt_get_curwin_w_p_rnu(void)
{
  return curwin->w_p_rnu;
}

// =============================================================================
// C Accessor Functions for Format Operators (Phase T3)
// =============================================================================

/// Get oap->cursor_start.lnum (accessor for Rust).
linenr_T nvim_textfmt_oap_get_cursor_start_lnum(oparg_T *oap)
{
  return oap->cursor_start.lnum;
}

/// Get oap->cursor_start.col (accessor for Rust).
colnr_T nvim_textfmt_oap_get_cursor_start_col(oparg_T *oap)
{
  return oap->cursor_start.col;
}

/// Get oap->start.lnum (accessor for Rust).
linenr_T nvim_textfmt_oap_get_start_lnum(oparg_T *oap)
{
  return oap->start.lnum;
}

/// Get oap->start.col (accessor for Rust).
colnr_T nvim_textfmt_oap_get_start_col(oparg_T *oap)
{
  return oap->start.col;
}

/// Get oap->end.lnum (accessor for Rust).
linenr_T nvim_textfmt_oap_get_end_lnum(oparg_T *oap)
{
  return oap->end.lnum;
}

/// Get oap->line_count (accessor for Rust).
linenr_T nvim_textfmt_oap_get_line_count(oparg_T *oap)
{
  return oap->line_count;
}

/// Get oap->is_VIsual (accessor for Rust).
bool nvim_textfmt_oap_get_is_VIsual(oparg_T *oap)
{
  return oap->is_VIsual;
}

/// Get oap->end_adjusted (accessor for Rust).
bool nvim_textfmt_oap_get_end_adjusted(oparg_T *oap)
{
  return oap->end_adjusted;
}

/// Set curwin->w_cursor (accessor for Rust).
void nvim_textfmt_set_curwin_cursor(linenr_T lnum, colnr_T col)
{
  curwin->w_cursor.lnum = lnum;
  curwin->w_cursor.col = col;
}

/// Get curwin->w_cursor.lnum (accessor for Rust).
linenr_T nvim_textfmt_get_curwin_cursor_lnum(void)
{
  return curwin->w_cursor.lnum;
}

/// Call u_save (accessor for Rust).
int nvim_textfmt_u_save(linenr_T top, linenr_T bot)
{
  return u_save(top, bot);
}

/// Call redraw_curbuf_later (accessor for Rust).
void nvim_textfmt_redraw_curbuf_later(int type)
{
  redraw_curbuf_later(type);
}

/// Get cmdmod lockmarks flag (accessor for Rust).
bool nvim_textfmt_get_cmdmod_lockmarks(void)
{
  return (cmdmod.cmod_flags & CMOD_LOCKMARKS) != 0;
}

/// Set curbuf->b_op_start (accessor for Rust).
void nvim_textfmt_set_curbuf_op_start(linenr_T lnum, colnr_T col)
{
  curbuf->b_op_start.lnum = lnum;
  curbuf->b_op_start.col = col;
}

/// Set curbuf->b_op_end (accessor for Rust).
void nvim_textfmt_set_curbuf_op_end(linenr_T lnum, colnr_T col)
{
  curbuf->b_op_end.lnum = lnum;
  curbuf->b_op_end.col = col;
}

/// Set saved_cursor (accessor for Rust).
void nvim_textfmt_set_saved_cursor(linenr_T lnum, colnr_T col)
{
  saved_cursor.lnum = lnum;
  saved_cursor.col = col;
}

/// Get saved_cursor.lnum (accessor for Rust).
linenr_T nvim_textfmt_get_saved_cursor_lnum(void)
{
  return saved_cursor.lnum;
}

/// Clear saved_cursor (accessor for Rust).
void nvim_textfmt_clear_saved_cursor(void)
{
  saved_cursor.lnum = 0;
}

/// Call beginline (accessor for Rust).
void nvim_textfmt_beginline(int flags)
{
  beginline(flags);
}

/// Call check_cursor (accessor for Rust).
void nvim_textfmt_check_cursor(void *win)
{
  check_cursor((win_T *)win);
}

/// Get curbuf->b_ml.ml_line_count (accessor for Rust).
linenr_T nvim_textfmt_get_ml_line_count(void)
{
  return curbuf->b_ml.ml_line_count;
}

/// Call msgmore (accessor for Rust).
void nvim_textfmt_msgmore(linenr_T n)
{
  msgmore(n);
}

/// Adjust visual windows after line count change (accessor for Rust).
void nvim_textfmt_adjust_visual_windows(linenr_T old_line_count)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_old_cursor_lnum != 0) {
      if (wp->w_old_cursor_lnum > wp->w_old_visual_lnum) {
        wp->w_old_cursor_lnum += old_line_count;
      } else {
        wp->w_old_visual_lnum += old_line_count;
      }
    }
  }
}

// =============================================================================
// C Accessor Functions for fex_format (Phase fex)
// =============================================================================

/// Check if 'formatexpr' was set insecurely (accessor for Rust).
bool nvim_textfmt_fex_was_set_insecurely(void)
{
  return was_set_insecurely(curwin, kOptFormatexpr, OPT_LOCAL);
}

/// Set v:lnum (accessor for Rust).
void nvim_textfmt_set_vv_lnum(int64_t val)
{
  set_vim_var_nr(VV_LNUM, (varnumber_T)val);
}

/// Set v:count (accessor for Rust).
void nvim_textfmt_set_vv_count(int64_t val)
{
  set_vim_var_nr(VV_COUNT, (varnumber_T)val);
}

/// Set v:char (accessor for Rust).
void nvim_textfmt_set_vv_char(int c)
{
  set_vim_var_char(c);
}

/// Clear v:char (accessor for Rust).
void nvim_textfmt_clear_vv_char(void)
{
  set_vim_var_string(VV_CHAR, NULL, -1);
}

/// Get curbuf->b_p_fex (accessor for Rust).
char *nvim_textfmt_get_curbuf_b_p_fex(void)
{
  return curbuf->b_p_fex;
}

static sctx_T fex_saved_sctx;

/// Save current_sctx and set to curbuf's formatexpr script context (accessor for Rust).
void nvim_textfmt_fex_save_sctx(void)
{
  fex_saved_sctx = current_sctx;
  current_sctx = curbuf->b_p_script_ctx[kBufOptFormatexpr];
}

/// Restore current_sctx after fex evaluation (accessor for Rust).
void nvim_textfmt_fex_restore_sctx(void)
{
  current_sctx = fex_saved_sctx;
}

/// Increment sandbox counter (accessor for Rust).
void nvim_textfmt_sandbox_inc(void)
{
  sandbox++;
}

/// Decrement sandbox counter (accessor for Rust).
void nvim_textfmt_sandbox_dec(void)
{
  sandbox--;
}

/// Evaluate expression and return number (accessor for Rust).
int nvim_textfmt_eval_to_number(char *expr)
{
  return (int)eval_to_number(expr, true);
}

// =============================================================================
// C Accessor Functions for format_lines (Phase 2)
// =============================================================================

_Static_assert(INSCHAR_FORMAT == 1, "");
_Static_assert(INSCHAR_DO_COM == 2, "");
_Static_assert(INSCHAR_NO_FEX == 8, "");
_Static_assert(INSCHAR_COM_LIST == 16, "");
_Static_assert(MODE_NORMAL == 0x01, "");
_Static_assert(MODE_INSERT == 0x10, "");
_Static_assert(SIN_CHANGED == 1, "");

/// Delete bytes at cursor (accessor for Rust).
void nvim_textfmt_del_bytes(int count, bool fixpos, bool use_delcombine)
{
  del_bytes(count, fixpos, use_delcombine);
}

/// Call do_join (accessor for Rust).
int nvim_textfmt_do_join(int count, bool insert_space, bool save_undo, bool use_fo, bool setmark)
{
  return do_join((size_t)count, insert_space, save_undo, use_fo, setmark);
}

/// Check if line is empty (accessor for Rust).
bool nvim_textfmt_lineempty(linenr_T lnum)
{
  return *ml_get(lnum) == NUL;
}

/// Get p_smd (accessor for Rust).
int nvim_textfmt_get_p_smd(void)
{
  return p_smd;
}

/// Set p_smd (accessor for Rust).
void nvim_textfmt_set_p_smd(int val)
{
  p_smd = val;
}

/// Get get_c_indent (accessor for Rust).
int nvim_textfmt_get_c_indent(void)
{
  return get_c_indent();
}

/// Get get_expr_indent (accessor for Rust).
int nvim_textfmt_get_expr_indent(void)
{
  return get_expr_indent();
}

// =============================================================================
// C Accessor Functions for Auto-format (Phase T4)
// =============================================================================

/// Get curwin->w_cursor.col (accessor for Rust).
colnr_T nvim_textfmt_get_curwin_cursor_col(void)
{
  return curwin->w_cursor.col;
}

/// Call dec_cursor (accessor for Rust).
void nvim_textfmt_dec_cursor(void)
{
  dec_cursor();
}

/// Call inc_cursor (accessor for Rust).
void nvim_textfmt_inc_cursor(void)
{
  inc_cursor();
}

/// Call gchar_cursor (accessor for Rust).
int nvim_textfmt_gchar_cursor(void)
{
  return gchar_cursor();
}

/// Get pointer to cursor line content (accessor for Rust).
char *nvim_textfmt_get_cursor_line_ptr(void)
{
  return get_cursor_line_ptr();
}

/// Get length of cursor line (accessor for Rust).
colnr_T nvim_textfmt_get_cursor_line_len(void)
{
  return get_cursor_line_len();
}

/// Check WHITECHAR condition (accessor for Rust).
/// This handles the UTF-8 composing character check.
bool nvim_textfmt_whitechar(int cc)
{
  return ascii_iswhite(cc)
         && !utf_iscomposing_first(utf_ptr2char((char *)get_cursor_pos_ptr() + 1));
}

/// Get leader length without flags output (accessor for Rust).
int nvim_textfmt_get_leader_len_simple(char *line)
{
  return get_leader_len(line, NULL, false, true);
}

/// Call u_save_cursor (accessor for Rust).
int nvim_textfmt_u_save_cursor(void)
{
  return u_save_cursor();
}

/// Get saved_cursor.col (accessor for Rust).
colnr_T nvim_textfmt_get_saved_cursor_col(void)
{
  return saved_cursor.col;
}

/// Call coladvance (accessor for Rust).
void nvim_textfmt_coladvance(void *win, colnr_T col)
{
  coladvance((win_T *)win, col);
}

/// Call check_cursor_col (accessor for Rust).
void nvim_textfmt_check_cursor_col(void *win)
{
  check_cursor_col((win_T *)win);
}

/// Replace line with added trailing space (accessor for Rust).
void nvim_textfmt_ml_replace_with_space(linenr_T lnum)
{
  char *linep = get_cursor_line_ptr();
  colnr_T len = get_cursor_line_len();
  char *plinep = xstrnsave(linep, (size_t)len + 2);
  plinep[len] = ' ';
  plinep[len + 1] = NUL;
  ml_replace(lnum, plinep, false);
}

/// Call del_char (accessor for Rust).
int nvim_textfmt_del_char(bool fixpos)
{
  return del_char(fixpos);
}

static bool did_add_space = false;  ///< auto_format() added an extra space
                                    ///< under the cursor

/// Get did_add_space state (accessor for Rust).
bool nvim_textfmt_get_did_add_space(void)
{
  return did_add_space;
}

/// Set did_add_space state (accessor for Rust).
void nvim_textfmt_set_did_add_space(bool val)
{
  did_add_space = val;
}

#define WHITECHAR(cc) (ascii_iswhite(cc) \
                       && !utf_iscomposing_first(utf_ptr2char((char *)get_cursor_pos_ptr() + 1)))

/// Return true if format option 'x' is in effect.
/// Take care of no formatting when 'paste' is set.
bool has_format_option(int x)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_has_format_option(x) != 0;
}

/// Format text at the current insert position.
///
/// If the INSCHAR_COM_LIST flag is present, then the value of second_indent
/// will be the comment leader length sent to open_line().
///
/// @param c  character to be inserted (can be NUL)
void internal_format(int textwidth, int second_indent, int flags, bool format_only, int c)
{
  int cc;
  char save_char = NUL;
  bool haveto_redraw = false;
  const bool fo_ins_blank = has_format_option(FO_INS_BLANK);
  const bool fo_multibyte = has_format_option(FO_MBYTE_BREAK);
  const bool fo_rigor_tw = has_format_option(FO_RIGOROUS_TW);
  const bool fo_white_par = has_format_option(FO_WHITE_PAR);
  bool first_line = true;
  colnr_T leader_len;
  bool no_leader = false;
  bool do_comments = (flags & INSCHAR_DO_COM);
  int has_lbr = curwin->w_p_lbr;

  // make sure win_charsize() counts correctly
  curwin->w_p_lbr = false;

  // When 'ai' is off we don't want a space under the cursor to be
  // deleted.  Replace it with an 'x' temporarily.
  if (!curbuf->b_p_ai && !(State & VREPLACE_FLAG)) {
    cc = gchar_cursor();
    if (ascii_iswhite(cc)) {
      save_char = (char)cc;
      pchar_cursor('x');
    }
  }

  // Repeat breaking lines, until the current line is not too long.
  while (!got_int) {
    int startcol;                       // Cursor column at entry
    int wantcol;                        // column at textwidth border
    int foundcol;                       // column for start of spaces
    int end_foundcol = 0;               // column for start of word
    int orig_col = 0;
    char *saved_text = NULL;
    colnr_T col;
    bool did_do_comment = false;

    colnr_T virtcol = get_nolist_virtcol() + char2cells(c != NUL ? c : gchar_cursor());
    if (virtcol <= (colnr_T)textwidth) {
      break;
    }

    if (no_leader) {
      do_comments = false;
    } else if (!(flags & INSCHAR_FORMAT)
               && has_format_option(FO_WRAP_COMS)) {
      do_comments = true;
    }

    // Don't break until after the comment leader
    if (do_comments) {
      char *line = get_cursor_line_ptr();
      leader_len = get_leader_len(line, NULL, false, true);
      if (leader_len == 0 && curbuf->b_p_cin) {
        // Check for a line comment after code.
        int comment_start = check_linecomment(line);
        if (comment_start != MAXCOL) {
          leader_len = get_leader_len(line + comment_start, NULL, false, true);
          if (leader_len != 0) {
            leader_len += comment_start;
          }
        }
      }
    } else {
      leader_len = 0;
    }

    // If the line doesn't start with a comment leader, then don't
    // start one in a following broken line.  Avoids that a %word
    // moved to the start of the next line causes all following lines
    // to start with %.
    if (leader_len == 0) {
      no_leader = true;
    }
    if (!(flags & INSCHAR_FORMAT)
        && leader_len == 0
        && !has_format_option(FO_WRAP)) {
      break;
    }
    if ((startcol = curwin->w_cursor.col) == 0) {
      break;
    }

    // find column of textwidth border
    coladvance(curwin, (colnr_T)textwidth);
    wantcol = curwin->w_cursor.col;

    curwin->w_cursor.col = startcol;
    foundcol = 0;
    int skip_pos = 0;

    // Find position to break at.
    // Stop at first entered white when 'formatoptions' has 'v'
    while ((!fo_ins_blank && !has_format_option(FO_INS_VI))
           || (flags & INSCHAR_FORMAT)
           || curwin->w_cursor.lnum != Insstart.lnum
           || curwin->w_cursor.col >= Insstart.col) {
      if (curwin->w_cursor.col == startcol && c != NUL) {
        cc = c;
      } else {
        cc = gchar_cursor();
      }
      if (WHITECHAR(cc)) {
        // remember position of blank just before text
        colnr_T end_col = curwin->w_cursor.col;

        // find start of sequence of blanks
        int wcc = 0;  // counter for whitespace chars
        while (curwin->w_cursor.col > 0 && WHITECHAR(cc)) {
          dec_cursor();
          cc = gchar_cursor();

          // Increment count of how many whitespace chars in this
          // group; we only need to know if it's more than one.
          if (wcc < 2) {
            wcc++;
          }
        }
        if (curwin->w_cursor.col == 0 && WHITECHAR(cc)) {
          break;                        // only spaces in front of text
        }

        // Don't break after a period when 'formatoptions' has 'p' and
        // there are less than two spaces.
        if (has_format_option(FO_PERIOD_ABBR) && cc == '.' && wcc < 2) {
          continue;
        }

        // Don't break until after the comment leader
        if (curwin->w_cursor.col < leader_len) {
          break;
        }

        if (has_format_option(FO_ONE_LETTER)) {
          // do not break after one-letter words
          if (curwin->w_cursor.col == 0) {
            break;              // one-letter word at begin
          }
          // do not break "#a b" when 'tw' is 2
          if (curwin->w_cursor.col <= leader_len) {
            break;
          }
          col = curwin->w_cursor.col;
          dec_cursor();
          cc = gchar_cursor();

          if (WHITECHAR(cc)) {
            continue;                   // one-letter, continue
          }
          curwin->w_cursor.col = col;
        }

        inc_cursor();

        end_foundcol = end_col + 1;
        foundcol = curwin->w_cursor.col;
        if (curwin->w_cursor.col <= (colnr_T)wantcol) {
          break;
        }
      } else if ((cc >= 0x100 || !utf_allow_break_before(cc)) && fo_multibyte) {
        int ncc;
        bool allow_break;

        // Break after or before a multi-byte character.
        if (curwin->w_cursor.col != startcol) {
          // Don't break until after the comment leader
          if (curwin->w_cursor.col < leader_len) {
            break;
          }
          col = curwin->w_cursor.col;
          inc_cursor();
          ncc = gchar_cursor();
          allow_break = utf_allow_break(cc, ncc);

          // If we have already checked this position, skip!
          if (curwin->w_cursor.col != skip_pos && allow_break) {
            foundcol = curwin->w_cursor.col;
            end_foundcol = foundcol;
            if (curwin->w_cursor.col <= (colnr_T)wantcol) {
              break;
            }
          }
          curwin->w_cursor.col = col;
        }

        if (curwin->w_cursor.col == 0) {
          break;
        }

        ncc = cc;
        col = curwin->w_cursor.col;

        dec_cursor();
        cc = gchar_cursor();

        if (WHITECHAR(cc)) {
          continue;                     // break with space
        }
        // Don't break until after the comment leader.
        if (curwin->w_cursor.col < leader_len) {
          break;
        }

        curwin->w_cursor.col = col;
        skip_pos = curwin->w_cursor.col;

        allow_break = utf_allow_break(cc, ncc);

        // Must handle this to respect line break prohibition.
        if (allow_break) {
          foundcol = curwin->w_cursor.col;
          end_foundcol = foundcol;
        }
        if (curwin->w_cursor.col <= (colnr_T)wantcol) {
          const bool ncc_allow_break = utf_allow_break_before(ncc);

          if (allow_break) {
            break;
          }
          if (!ncc_allow_break && !fo_rigor_tw) {
            // Enable at most 1 punct hang outside of textwidth.
            if (curwin->w_cursor.col == startcol) {
              // We are inserting a non-breakable char, postpone
              // line break check to next insert.
              end_foundcol = foundcol = 0;
              break;
            }

            // Neither cc nor ncc is NUL if we are here, so
            // it's safe to inc_cursor.
            col = curwin->w_cursor.col;

            inc_cursor();
            cc = ncc;
            ncc = gchar_cursor();
            // handle insert
            ncc = (ncc != NUL) ? ncc : c;

            allow_break = utf_allow_break(cc, ncc);

            if (allow_break) {
              // Break only when we are not at end of line.
              end_foundcol = foundcol = ncc == NUL ? 0 : curwin->w_cursor.col;
              break;
            }
            curwin->w_cursor.col = col;
          }
        }
      }
      if (curwin->w_cursor.col == 0) {
        break;
      }
      dec_cursor();
    }

    if (foundcol == 0) {                // no spaces, cannot break line
      curwin->w_cursor.col = startcol;
      break;
    }

    // Going to break the line, remove any "$" now.
    undisplay_dollar();

    // Offset between cursor position and line break is used by replace
    // stack functions.  MODE_VREPLACE does not use this, and backspaces
    // over the text instead.
    if (State & VREPLACE_FLAG) {
      orig_col = startcol;              // Will start backspacing from here
    } else {
      replace_offset = startcol - end_foundcol;
    }

    // adjust startcol for spaces that will be deleted and
    // characters that will remain on top line
    curwin->w_cursor.col = foundcol;
    while ((cc = gchar_cursor(), WHITECHAR(cc))
           && (!fo_white_par || curwin->w_cursor.col < startcol)) {
      inc_cursor();
    }
    startcol -= curwin->w_cursor.col;
    startcol = MAX(startcol, 0);

    if (State & VREPLACE_FLAG) {
      // In MODE_VREPLACE state, we will backspace over the text to be
      // wrapped, so save a copy now to put on the next line.
      saved_text = xstrnsave(get_cursor_pos_ptr(), (size_t)get_cursor_pos_len());
      curwin->w_cursor.col = orig_col;
      saved_text[startcol] = NUL;

      // Backspace over characters that will move to the next line
      if (!fo_white_par) {
        backspace_until_column(foundcol);
      }
    } else {
      // put cursor after pos. to break line
      if (!fo_white_par) {
        curwin->w_cursor.col = foundcol;
      }
    }

    // Split the line just before the margin.
    // Only insert/delete lines, but don't really redraw the window.
    open_line(FORWARD, OPENLINE_DELSPACES + OPENLINE_MARKFIX
              + (fo_white_par ? OPENLINE_KEEPTRAIL : 0)
              + (do_comments ? OPENLINE_DO_COM : 0)
              + OPENLINE_FORMAT
              + ((flags & INSCHAR_COM_LIST) ? OPENLINE_COM_LIST : 0),
              ((flags & INSCHAR_COM_LIST) ? second_indent : old_indent),
              &did_do_comment);
    if (!(flags & INSCHAR_COM_LIST)) {
      old_indent = 0;
    }

    // If a comment leader was inserted, may also do this on a following
    // line.
    if (did_do_comment) {
      no_leader = false;
    }

    replace_offset = 0;
    if (first_line) {
      if (!(flags & INSCHAR_COM_LIST)) {
        // This section is for auto-wrap of numeric lists.  When not
        // in insert mode (i.e. format_lines()), the INSCHAR_COM_LIST
        // flag will be set and open_line() will handle it (as seen
        // above).  The code here (and in get_number_indent()) will
        // recognize comments if needed...
        if (second_indent < 0 && has_format_option(FO_Q_NUMBER)) {
          second_indent = get_number_indent(curwin->w_cursor.lnum - 1);
        }
        if (second_indent >= 0) {
          if (State & VREPLACE_FLAG) {
            change_indent(INDENT_SET, second_indent, false, true);
          } else if (leader_len > 0 && second_indent - leader_len > 0) {
            int padding = second_indent - leader_len;

            // We started at the first_line of a numbered list
            // that has a comment.  the open_line() function has
            // inserted the proper comment leader and positioned
            // the cursor at the end of the split line.  Now we
            // add the additional whitespace needed after the
            // comment leader for the numbered list.
            for (int i = 0; i < padding; i++) {
              ins_str(S_LEN(" "));
            }
          } else {
            set_indent(second_indent, SIN_CHANGED);
          }
        }
      }
      first_line = false;
    }

    if (State & VREPLACE_FLAG) {
      // In MODE_VREPLACE state we have backspaced over the text to be
      // moved, now we re-insert it into the new line.
      ins_bytes(saved_text);
      xfree(saved_text);
    } else {
      // Check if cursor is not past the NUL off the line, cindent
      // may have added or removed indent.
      curwin->w_cursor.col += startcol;
      colnr_T len = get_cursor_line_len();
      curwin->w_cursor.col = MIN(curwin->w_cursor.col, len);
    }

    haveto_redraw = true;
    set_can_cindent(true);
    // moved the cursor, don't autoindent or cindent now
    did_ai = false;
    did_si = false;
    can_si = false;
    can_si_back = false;
    line_breakcheck();
  }

  if (save_char != NUL) {               // put back space after cursor
    pchar_cursor(save_char);
  }

  curwin->w_p_lbr = has_lbr;

  if (!format_only && haveto_redraw) {
    update_topline(curwin);
    redraw_curbuf_later(UPD_VALID);
  }
}

/// Blank lines, and lines containing only the comment leader, are left
/// untouched by the formatting.  The function returns true in this
/// case.  It also returns true when a line starts with the end of a comment
/// ('e' in comment flags), so that this line is skipped, and not joined to the
/// previous line.  A new paragraph starts after a blank line, or when the
/// comment leader changes.
static int fmt_check_par(linenr_T lnum, int *leader_len, char **leader_flags, bool do_comments)
{
  return rs_fmt_check_par(lnum, leader_len, leader_flags, do_comments);
}

/// @return  true if line "lnum" ends in a white character.
static bool ends_in_white(linenr_T lnum)
{
  return rs_ends_in_white(lnum) != 0;
}

/// @return  true if the two comment leaders given are the same.
///
/// @param lnum  The first line.  White-space is ignored.
///
/// @note the whole of 'leader1' must match 'leader2_len' characters from 'leader2'.
static bool same_leader(linenr_T lnum, int leader1_len, char *leader1_flags, int leader2_len,
                        char *leader2_flags)
{
  return rs_same_leader(lnum, leader1_len, leader1_flags, leader2_len, leader2_flags) != 0;
}

/// Used for auto-formatting.
///
/// @return  true when a paragraph starts in line "lnum".
///          false when the previous line is in the same paragraph.
static bool paragraph_start(linenr_T lnum)
{
  return rs_paragraph_start(lnum) != 0;
}

/// Called after inserting or deleting text: When 'formatoptions' includes the
/// 'a' flag format from the current line until the end of the paragraph.
/// Keep the cursor at the same position relative to the text.
/// The caller must have saved the cursor line for undo, following ones will be
/// saved here.
///
/// @param trailblank  when true also format with trailing blank
/// @param prev_line   may start in previous line
void auto_format(bool trailblank, bool prev_line)
{
  rs_auto_format(trailblank, prev_line);
}

/// When an extra space was added to continue a paragraph for auto-formatting,
/// delete it now.  The space must be under the cursor, just after the insert
/// position.
///
/// @param end_insert  true when ending Insert mode
void check_auto_format(bool end_insert)
{
  rs_check_auto_format(end_insert);
}

/// Find out textwidth to be used for formatting:
///      if 'textwidth' option is set, use it
///      else if 'wrapmargin' option is set, use curwin->w_view_width-'wrapmargin'
///      if invalid value, use 0.
///      Set default to window width (maximum 79) for "gq" operator.
///
/// @param ff  force formatting (for "gq" command)
int comp_textwidth(bool ff)
{
  return rs_comp_textwidth(ff);
}

/// Implementation of the format operator 'gq'.
///
/// @param keep_cursor  keep cursor on same text char
void op_format(oparg_T *oap, bool keep_cursor)
{
  rs_op_format(oap, keep_cursor);
}

/// Implementation of the format operator 'gq' for when using 'formatexpr'.
void op_formatexpr(oparg_T *oap)
{
  rs_op_formatexpr(oap);
}

/// @param c  character to be inserted
int fex_format(linenr_T lnum, long count, int c)
{
  return rs_fex_format(lnum, count, c);
}

/// @param line_count  number of lines to format, starting at the cursor position.
///                    when negative, format until the end of the paragraph.
///
/// Lines after the cursor line are saved for undo, caller must have saved the
/// first line.
///
/// @param avoid_fex  don't use 'formatexpr'
void format_lines(linenr_T line_count, bool avoid_fex)
{
  rs_format_lines(line_count, avoid_fex);
}
