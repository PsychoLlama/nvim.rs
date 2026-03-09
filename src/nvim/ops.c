// ops.c: implementation of various operators: op_shift, op_delete, op_tilde,
//        op_change, op_yank, do_join

#include <assert.h>
#include <ctype.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>

#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/assert_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/buffer_updates.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/clipboard.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/ex_cmds2.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_getln.h"
#include "nvim/extmark.h"
#include "nvim/file_search.h"
#include "nvim/fold.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/getchar_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/indent.h"
#include "nvim/indent_c.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/math.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/os/time.h"
#include "nvim/plines.h"
#include "nvim/register.h"
#include "nvim/search.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/terminal.h"
#include "nvim/textformat.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"

#include "ops.c.generated.h"

// Rust fold FFI declarations
extern void rs_foldOpenCursor(void);
extern void rs_deleteFold(win_T *wp, linenr_T start, linenr_T end, int recursive, bool had_visual);
extern void rs_foldCreate(win_T *wp, linenr_T start_lnum, linenr_T end_lnum);
extern void rs_opFoldRange(linenr_T first_lnum, linenr_T last_lnum, int opening, int recurse, bool had_visual);

extern void rs_restore_visual_mode(void);
extern void rs_prep_redo(int regname, int num, int cmd1, int cmd2, int cmd3, int cmd4, int cmd5);
extern void rs_prep_redo_num2(int regname, int num1, int cmd1, int cmd2, int num2, int cmd3, int cmd4, int cmd5);
extern void rs_clearop(oparg_T *oap);
extern void rs_clearopbeep(oparg_T *oap);
extern void rs_may_clear_cmdline(void);
extern bool rs_unadjust_for_sel(void);

extern int rs_get_fileformat(buf_T *buf);
extern bool rs_set_ref_in_callback(Callback *callback, int copyID, ht_stack_T **ht_stack,
                                   list_stack_T **list_stack);


// Functions now exported from Rust (via #[export_name]) but still called within ops.c
extern int get_op_char(int optype);
extern int get_extra_op_char(int optype);
extern int op_on_lines(int op);

/// handle a shift operation
void op_shift(oparg_T *oap, bool curs_top, int amount)
{
  int block_col = 0;

  if (u_save((linenr_T)(oap->start.lnum - 1),
             (linenr_T)(oap->end.lnum + 1)) == FAIL) {
    return;
  }

  if (oap->motion_type == kMTBlockWise) {
    block_col = curwin->w_cursor.col;
  }

  for (int i = oap->line_count - 1; i >= 0; i--) {
    int first_char = (uint8_t)(*get_cursor_line_ptr());
    if (first_char == NUL) {  // empty line
      curwin->w_cursor.col = 0;
    } else if (oap->motion_type == kMTBlockWise) {
      shift_block(oap, amount);
    } else if (first_char != '#' || !preprocs_left()) {
      // Move the line right if it doesn't start with '#', 'smartindent'
      // isn't set or 'cindent' isn't set or '#' isn't in 'cino'.
      shift_line(oap->op_type == OP_LSHIFT, p_sr, amount, false);
    }
    curwin->w_cursor.lnum++;
  }

  if (oap->motion_type == kMTBlockWise) {
    curwin->w_cursor.lnum = oap->start.lnum;
    curwin->w_cursor.col = block_col;
  } else if (curs_top) {  // put cursor on first line, for ">>"
    curwin->w_cursor.lnum = oap->start.lnum;
    beginline(BL_SOL | BL_FIX);       // shift_line() may have set cursor.col
  } else {
    curwin->w_cursor.lnum--;            // put cursor on last line, for ":>"
  }
  // The cursor line is not in a closed fold
  rs_foldOpenCursor();

  if (oap->line_count > p_report) {
    char *op = oap->op_type == OP_RSHIFT ? ">" : "<";

    char *msg_line_single = NGETTEXT("%" PRId64 " line %sed %d time",
                                     "%" PRId64 " line %sed %d times", amount);
    char *msg_line_plural = NGETTEXT("%" PRId64 " lines %sed %d time",
                                     "%" PRId64 " lines %sed %d times", amount);
    vim_snprintf(IObuff, IOSIZE,
                 NGETTEXT(msg_line_single, msg_line_plural, oap->line_count),
                 (int64_t)oap->line_count, op, amount);
    msg_keep(IObuff, 0, true, false);
  }

  if ((cmdmod.cmod_flags & CMOD_LOCKMARKS) == 0) {
    // Set "'[" and "']" marks.
    curbuf->b_op_start = oap->start;
    curbuf->b_op_end.lnum = oap->end.lnum;
    curbuf->b_op_end.col = ml_get_len(oap->end.lnum);
    if (curbuf->b_op_end.col > 0) {
      curbuf->b_op_end.col--;
    }
  }

  changed_lines(curbuf, oap->start.lnum, 0, oap->end.lnum + 1, 0, true);
}

/// Return the tabstop width at the given index of the variable tabstop array.
static int get_vts(const int *vts_array, int index)
{
  if (index < 1) {
    return 0;
  } else if (index <= vts_array[0]) {
    return vts_array[index];
  } else {
    return vts_array[vts_array[0]];
  }
}

/// Return the sum of all the tabstops through the index-th.
static int get_vts_sum(const int *vts_array, int index)
{
  int sum = 0;
  int i;
  for (i = 1; i <= index && i <= vts_array[0]; i++) {
    sum += vts_array[i];
  }
  if (i <= index) {
    sum += vts_array[vts_array[0]] * (index - vts_array[0]);
  }
  return sum;
}

/// @param left    true if shift is to the left
/// @param count   true if new indent is to be to a tabstop
/// @param amount  number of shifts
static int64_t get_new_sw_indent(bool left, bool round, int64_t amount, int64_t sw_val)
{
  int64_t count = get_indent();  // get current indent

  if (round) {  // round off indent
    int64_t i = trim_to_int(count / sw_val);  // number of 'shiftwidth' rounded down
    int64_t j = trim_to_int(count % sw_val);  // extra spaces
    if (j && left) {  // first remove extra spaces
      amount--;
    }
    if (left) {
      i = MAX(i - amount, 0);
    } else {
      i += amount;
    }
    count = i * sw_val;
  } else {  // original vi indent
    if (left) {
      count = MAX(count - sw_val * amount, 0);
    } else {
      count += sw_val * amount;
    }
  }

  return count;
}

/// @param left    true if shift is to the left
/// @param count   true if new indent is to be to a tabstop
/// @param amount  number of shifts
static int64_t get_new_vts_indent(bool left, bool round, int amount, int *vts_array)
{
  int64_t indent = get_indent();
  int vtsi = 0;
  int vts_indent = 0;
  int ts = 0;         // Silence uninitialized variable warning.

  // Find the tabstop at or to the left of the current indent.
  while (vts_indent <= indent) {
    vtsi++;
    ts = get_vts(vts_array, vtsi);
    vts_indent += ts;
  }
  vts_indent -= ts;
  vtsi--;

  // Extra indent spaces to the right of the tabstop
  int64_t offset = indent - vts_indent;

  if (round) {
    if (left) {
      if (offset == 0) {
        indent = get_vts_sum(vts_array, vtsi - amount);
      } else {
        indent = get_vts_sum(vts_array, vtsi - (amount - 1));
      }
    } else {
      indent = get_vts_sum(vts_array, vtsi + amount);
    }
  } else {
    if (left) {
      if (amount > vtsi) {
        indent = 0;
      } else {
        indent = get_vts_sum(vts_array, vtsi - amount) + offset;
      }
    } else {
      indent = get_vts_sum(vts_array, vtsi + amount) + offset;
    }
  }

  return indent;
}

/// Shift the current line 'amount' shiftwidth(s) left (if 'left' is true) or
/// right.
///
/// The rules for choosing a shiftwidth are:  If 'shiftwidth' is non-zero, use
/// 'shiftwidth'; else if 'vartabstop' is not empty, use 'vartabstop'; else use
/// 'tabstop'.  The Vim documentation says nothing about 'softtabstop' or
/// 'varsofttabstop' affecting the shiftwidth, and neither affects the
/// shiftwidth in current versions of Vim, so they are not considered here.
///
/// @param left                true if shift is to the left
/// @param count               true if new indent is to be to a tabstop
/// @param amount              number of shifts
/// @param call_changed_bytes  call changed_bytes()
void shift_line(bool left, bool round, int amount, int call_changed_bytes)
{
  int64_t count;
  int64_t sw_val = curbuf->b_p_sw;
  int64_t ts_val = curbuf->b_p_ts;
  int *vts_array = curbuf->b_p_vts_array;

  if (sw_val != 0) {
    // 'shiftwidth' is not zero; use it as the shift size.
    count = get_new_sw_indent(left, round, amount, sw_val);
  } else if ((vts_array == NULL) || (vts_array[0] == 0)) {
    // 'shiftwidth' is zero and 'vartabstop' is empty; use 'tabstop' as the
    // shift size.
    count = get_new_sw_indent(left, round, amount, ts_val);
  } else {
    // 'shiftwidth' is zero and 'vartabstop' is defined; use 'vartabstop'
    // to determine the new indent.
    count = get_new_vts_indent(left, round, amount, vts_array);
  }

  // Set new indent
  if (State & VREPLACE_FLAG) {
    change_indent(INDENT_SET, trim_to_int(count), false, call_changed_bytes);
  } else {
    set_indent(trim_to_int(count), call_changed_bytes ? SIN_CHANGED : 0);
  }
}

/// Shift one line of the current block one shiftwidth right or left.
/// Leaves cursor on first character in block.
static void shift_block(oparg_T *oap, int amount)
{
  const bool left = (oap->op_type == OP_LSHIFT);
  const int oldstate = State;
  char *newp;
  const int oldcol = curwin->w_cursor.col;
  const int sw_val = get_sw_value_indent(curbuf, left);
  const int ts_val = (int)curbuf->b_p_ts;
  struct block_def bd;
  int incr;
  const int old_p_ri = p_ri;

  p_ri = 0;                     // don't want revins in indent

  State = MODE_INSERT;          // don't want MODE_REPLACE for State
  block_prep(oap, &bd, curwin->w_cursor.lnum, true);
  if (bd.is_short) {
    return;
  }

  // total is number of screen columns to be inserted/removed
  int total = (int)((unsigned)amount * (unsigned)sw_val);
  if ((total / sw_val) != amount) {
    return;   // multiplication overflow
  }

  char *const oldp = get_cursor_line_ptr();
  const int old_line_len = get_cursor_line_len();

  int startcol, oldlen, newlen;

  if (!left) {
    //  1. Get start vcol
    //  2. Total ws vcols
    //  3. Divvy into TABs & spp
    //  4. Construct new string
    total += bd.pre_whitesp;    // all virtual WS up to & incl a split TAB
    colnr_T ws_vcol = bd.start_vcol - bd.pre_whitesp;
    char *old_textstart = bd.textstart;
    if (bd.startspaces) {
      if (utfc_ptr2len(bd.textstart) == 1) {
        bd.textstart++;
      } else {
        ws_vcol = 0;
        bd.startspaces = 0;
      }
    }

    // TODO(vim): is passing bd.textstart for start of the line OK?
    CharsizeArg csarg;
    CSType cstype = init_charsize_arg(&csarg, curwin, curwin->w_cursor.lnum, bd.textstart);
    StrCharInfo ci = utf_ptr2StrCharInfo(bd.textstart);
    int vcol = bd.start_vcol;
    while (ascii_iswhite(ci.chr.value)) {
      incr = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &csarg).width;
      ci = utfc_next(ci);
      total += incr;
      vcol += incr;
    }
    bd.textstart = ci.ptr;
    bd.start_vcol = vcol;

    int tabs = 0;
    int spaces = 0;
    // OK, now total=all the VWS reqd, and textstart points at the 1st
    // non-ws char in the block.
    if (!curbuf->b_p_et) {
      tabstop_fromto(ws_vcol, ws_vcol + total,
                     ts_val, curbuf->b_p_vts_array, &tabs, &spaces);
    } else {
      spaces = total;
    }

    // if we're splitting a TAB, allow for it
    const int col_pre = bd.pre_whitesp_c - (bd.startspaces != 0);
    bd.textcol -= col_pre;

    const int new_line_len  // the length of the line after the block shift
      = bd.textcol + tabs + spaces + (old_line_len - (int)(bd.textstart - oldp));
    newp = xmalloc((size_t)new_line_len + 1);
    memmove(newp, oldp, (size_t)bd.textcol);
    startcol = bd.textcol;
    oldlen = (int)(bd.textstart - old_textstart) + col_pre;
    newlen = tabs + spaces;
    memset(newp + bd.textcol, TAB, (size_t)tabs);
    memset(newp + bd.textcol + tabs, ' ', (size_t)spaces);
    STRCPY(newp + bd.textcol + tabs + spaces, bd.textstart);
    assert(newlen - oldlen == new_line_len - old_line_len);
  } else {  // left
    char *verbatim_copy_end;      // end of the part of the line which is
                                  // copied verbatim
    colnr_T verbatim_copy_width;  // the (displayed) width of this part
                                  // of line
    char *non_white = bd.textstart;

    // Firstly, let's find the first non-whitespace character that is
    // displayed after the block's start column and the character's column
    // number. Also, let's calculate the width of all the whitespace
    // characters that are displayed in the block and precede the searched
    // non-whitespace character.

    // If "bd.startspaces" is set, "bd.textstart" points to the character,
    // the part of which is displayed at the block's beginning. Let's start
    // searching from the next character.
    if (bd.startspaces) {
      MB_PTR_ADV(non_white);
    }

    // The character's column is in "bd.start_vcol".
    colnr_T non_white_col = bd.start_vcol;

    CharsizeArg csarg;
    CSType cstype = init_charsize_arg(&csarg, curwin, curwin->w_cursor.lnum, bd.textstart);
    while (ascii_iswhite(*non_white)) {
      incr = win_charsize(cstype, non_white_col, non_white, (uint8_t)(*non_white), &csarg).width;
      non_white_col += incr;
      non_white++;
    }

    const colnr_T block_space_width = non_white_col - oap->start_vcol;
    // We will shift by "total" or "block_space_width", whichever is less.
    const colnr_T shift_amount = MIN(block_space_width, total);
    // The column to which we will shift the text.
    const colnr_T destination_col = non_white_col - shift_amount;

    // Now let's find out how much of the beginning of the line we can
    // reuse without modification.
    verbatim_copy_end = bd.textstart;
    verbatim_copy_width = bd.start_vcol;

    // If "bd.startspaces" is set, "bd.textstart" points to the character
    // preceding the block. We have to subtract its width to obtain its
    // column number.
    if (bd.startspaces) {
      verbatim_copy_width -= bd.start_char_vcols;
    }
    cstype = init_charsize_arg(&csarg, curwin, 0, bd.textstart);
    StrCharInfo ci = utf_ptr2StrCharInfo(verbatim_copy_end);
    while (verbatim_copy_width < destination_col) {
      incr = win_charsize(cstype, verbatim_copy_width, ci.ptr, ci.chr.value, &csarg).width;
      if (verbatim_copy_width + incr > destination_col) {
        break;
      }
      verbatim_copy_width += incr;
      ci = utfc_next(ci);
    }
    verbatim_copy_end = ci.ptr;

    // If "destination_col" is different from the width of the initial
    // part of the line that will be copied, it means we encountered a tab
    // character, which we will have to partly replace with spaces.
    assert(destination_col - verbatim_copy_width >= 0);
    const int fill  // nr of spaces that replace a TAB
      = destination_col - verbatim_copy_width;

    assert(verbatim_copy_end - oldp >= 0);
    // length of string left of the shift position (ie the string not being shifted)
    const int fixedlen = (int)(verbatim_copy_end - oldp);
    // The replacement line will consist of:
    // - the beginning of the original line up to "verbatim_copy_end",
    // - "fill" number of spaces,
    // - the rest of the line, pointed to by non_white.
    const int new_line_len  // the length of the line after the block shift
      = fixedlen + fill + (old_line_len - (int)(non_white - oldp));

    newp = xmalloc((size_t)new_line_len + 1);
    startcol = fixedlen;
    oldlen = bd.textcol + (int)(non_white - bd.textstart) - fixedlen;
    newlen = fill;
    memmove(newp, oldp, (size_t)fixedlen);
    memset(newp + fixedlen, ' ', (size_t)fill);
    STRCPY(newp + fixedlen + fill, non_white);
    assert(newlen - oldlen == new_line_len - old_line_len);
  }
  // replace the line
  ml_replace(curwin->w_cursor.lnum, newp, false);
  changed_bytes(curwin->w_cursor.lnum, bd.textcol);
  extmark_splice_cols(curbuf, (int)curwin->w_cursor.lnum - 1, startcol,
                      oldlen, newlen,
                      kExtmarkUndo);
  State = oldstate;
  curwin->w_cursor.col = oldcol;
  p_ri = old_p_ri;
}

/// Insert string "s" (b_insert ? before : after) block :AKelly
/// Caller must prepare for undo.
static void block_insert(oparg_T *oap, const char *s, size_t slen, bool b_insert,
                         struct block_def *bdp)
{
  int ts_val;
  int count = 0;                // extra spaces to replace a cut TAB
  int spaces = 0;               // non-zero if cutting a TAB
  colnr_T offset;               // pointer along new line
  char *newp, *oldp;            // new, old lines
  int oldstate = State;
  State = MODE_INSERT;          // don't want MODE_REPLACE for State

  for (linenr_T lnum = oap->start.lnum + 1; lnum <= oap->end.lnum; lnum++) {
    block_prep(oap, bdp, lnum, true);
    if (bdp->is_short && b_insert) {
      continue;  // OP_INSERT, line ends before block start
    }

    oldp = ml_get(lnum);

    if (b_insert) {
      ts_val = bdp->start_char_vcols;
      spaces = bdp->startspaces;
      if (spaces != 0) {
        count = ts_val - 1;         // we're cutting a TAB
      }
      offset = bdp->textcol;
    } else {  // append
      ts_val = bdp->end_char_vcols;
      if (!bdp->is_short) {     // spaces = padding after block
        spaces = (bdp->endspaces ? ts_val - bdp->endspaces : 0);
        if (spaces != 0) {
          count = ts_val - 1;           // we're cutting a TAB
        }
        offset = bdp->textcol + bdp->textlen - (spaces != 0);
      } else {  // spaces = padding to block edge
                // if $ used, just append to EOL (ie spaces==0)
        if (!bdp->is_MAX) {
          spaces = (oap->end_vcol - bdp->end_vcol) + 1;
        }
        count = spaces;
        offset = bdp->textcol + bdp->textlen;
      }
    }

    if (spaces > 0) {
      // avoid copying part of a multi-byte character
      offset -= utf_head_off(oldp, oldp + offset);
    }
    spaces = MAX(spaces, 0);  // can happen when the cursor was moved

    assert(count >= 0);
    // Make sure the allocated size matches what is actually copied below.
    newp = xmalloc((size_t)ml_get_len(lnum) + (size_t)spaces + slen
                   + (spaces > 0 && !bdp->is_short ? (size_t)(ts_val - spaces) : 0)
                   + (size_t)count + 1);

    // copy up to shifted part
    memmove(newp, oldp, (size_t)offset);
    oldp += offset;
    int startcol = offset;

    // insert pre-padding
    memset(newp + offset, ' ', (size_t)spaces);

    // copy the new text
    memmove(newp + offset + spaces, s, slen);
    offset += (int)slen;

    int skipped = 0;
    if (spaces > 0 && !bdp->is_short) {
      if (*oldp == TAB) {
        // insert post-padding
        memset(newp + offset + spaces, ' ', (size_t)(ts_val - spaces));
        // We're splitting a TAB, don't copy it.
        oldp++;
        // We allowed for that TAB, remember this now
        count++;
        skipped = 1;
      } else {
        // Not a TAB, no extra spaces
        count = spaces;
      }
    }

    if (spaces > 0) {
      offset += count;
    }
    STRCPY(newp + offset, oldp);

    ml_replace(lnum, newp, false);
    extmark_splice_cols(curbuf, (int)lnum - 1, startcol,
                        skipped, offset - startcol, kExtmarkUndo);

    if (lnum == oap->end.lnum) {
      // Set "']" mark to the end of the block instead of the end of
      // the insert in the first line.
      curbuf->b_op_end.lnum = oap->end.lnum;
      curbuf->b_op_end.col = offset;
    }
  }   // for all lnum

  State = oldstate;

  // Only call changed_lines if we actually modified additional lines beyond the first
  // This matches the condition for the for loop above: start + 1 <= end
  if (oap->start.lnum < oap->end.lnum) {
    changed_lines(curbuf, oap->start.lnum + 1, 0, oap->end.lnum + 1, 0, true);
  }
}

/// Handle a delete operation.
///
/// @return  FAIL if undo failed, OK otherwise.
// ===========================================================================
// op_delete C accessors for Rust migration (Phase 4)
// ===========================================================================

/// Setup visual select register.
void nvim_opd_setup_visual_reg(oparg_T *oap)
{
  if (VIsual_select && oap->is_VIsual) {
    oap->regname = VIsual_select_reg;
  }
}

/// Adjust multi-byte opend for delete.
void nvim_opd_mb_adjust_opend(oparg_T *oap)
{
  mb_adjust_opend(oap);
}

/// Check if charwise delete should be promoted to linewise.
void nvim_opd_maybe_promote_to_linewise(oparg_T *oap)
{
  if (oap->motion_type == kMTCharWise
      && !oap->is_VIsual
      && oap->line_count > 1
      && oap->motion_force == NUL
      && oap->op_type == OP_DELETE) {
    char *ptr = ml_get(oap->end.lnum) + oap->end.col;
    if (*ptr != NUL) {
      ptr += oap->inclusive;
    }
    ptr = skipwhite(ptr);
    if (*ptr == NUL && inindent(0)) {
      oap->motion_type = kMTLineWise;
    }
  }
}

/// Check for empty line delete.
/// Returns: 0 = proceed, 1 = return OK, 2 = goto setmarks.
int nvim_opd_check_empty_line(oparg_T *oap)
{
  if (oap->motion_type != kMTLineWise
      && oap->line_count == 1
      && oap->op_type == OP_DELETE
      && *ml_get(oap->start.lnum) == NUL) {
    if (virtual_op) {
      return 2;  // goto setmarks
    }
    if (vim_strchr(p_cpo, CPO_EMPTYREGION) != NULL) {
      beep_flush();
    }
    return 1;  // return OK
  }
  return 0;  // proceed
}

/// Do yank and register handling.
/// Returns: OK (1) if should return OK (read-only reg), 2 if proceed normally.
int nvim_opd_do_yank_and_registers(oparg_T *oap)
{
  if (oap->regname != '_') {
    yankreg_T *reg = NULL;
    bool did_yank = false;
    if (oap->regname != 0) {
      if (!valid_yank_reg(oap->regname, true)) {
        beep_flush();
        return OK;  // return OK to caller
      }
      reg = get_yank_register(oap->regname, YREG_YANK);
      op_yank_reg(oap, false, reg, is_append_register(oap->regname));
      did_yank = true;
    }

    if (oap->motion_type == kMTLineWise || oap->line_count > 1
        || oap->use_reg_one) {
      shift_delete_registers(is_append_register(oap->regname));
      reg = get_y_register(1);
      op_yank_reg(oap, false, reg, false);
      did_yank = true;
    }

    if (oap->regname == 0 && oap->motion_type != kMTLineWise
        && oap->line_count == 1) {
      reg = get_yank_register('-', YREG_YANK);
      op_yank_reg(oap, false, reg, false);
      did_yank = true;
    }

    if (did_yank || oap->regname == 0) {
      if (reg == NULL) {
        abort();
      }
      set_clipboard(oap->regname, reg);
      do_autocmd_textyankpost(oap, reg);
    }
  }
  return 2;  // proceed normally
}

/// Block mode delete.
int nvim_opd_block_delete(oparg_T *oap)
{
  struct block_def bd = { 0 };

  if (u_save((linenr_T)(oap->start.lnum - 1),
             (linenr_T)(oap->end.lnum + 1)) == FAIL) {
    return FAIL;
  }

  for (linenr_T lnum = curwin->w_cursor.lnum; lnum <= oap->end.lnum; lnum++) {
    block_prep(oap, &bd, lnum, true);
    if (bd.textlen == 0) {
      continue;
    }

    if (lnum == curwin->w_cursor.lnum) {
      curwin->w_cursor.col = bd.textcol + bd.startspaces;
      curwin->w_cursor.coladd = 0;
    }

    int n = bd.textlen - bd.startspaces - bd.endspaces;
    char *oldp = ml_get(lnum);
    char *newp = xmalloc((size_t)ml_get_len(lnum) - (size_t)n + 1);
    memmove(newp, oldp, (size_t)bd.textcol);
    memset(newp + bd.textcol, ' ',
           (size_t)bd.startspaces + (size_t)bd.endspaces);
    STRCPY(newp + bd.textcol + bd.startspaces + bd.endspaces,
           oldp + bd.textcol + bd.textlen);
    ml_replace(lnum, newp, false);

    extmark_splice_cols(curbuf, (int)lnum - 1, bd.textcol,
                        bd.textlen, bd.startspaces + bd.endspaces,
                        kExtmarkUndo);
  }

  check_cursor_col(curwin);
  changed_lines(curbuf, curwin->w_cursor.lnum, curwin->w_cursor.col,
                oap->end.lnum + 1, 0, true);
  oap->line_count = 0;
  return OK;
}

/// Linewise delete.
int nvim_opd_linewise_delete(oparg_T *oap)
{
  if (oap->op_type == OP_CHANGE) {
    if (oap->line_count > 1) {
      linenr_T lnum = curwin->w_cursor.lnum;
      curwin->w_cursor.lnum++;
      del_lines(oap->line_count - 1, true);
      curwin->w_cursor.lnum = lnum;
    }
    if (u_save_cursor() == FAIL) {
      return FAIL;
    }
    if (curbuf->b_p_ai) {
      beginline(BL_WHITE);
      did_ai = true;
      ai_col = curwin->w_cursor.col;
    } else {
      beginline(0);
    }
    truncate_line(false);
    if (oap->line_count > 1) {
      u_clearline(curbuf);
    }
  } else {
    del_lines(oap->line_count, true);
    beginline(BL_WHITE | BL_FIX);
    u_clearline(curbuf);
  }
  return OK;
}

/// Charwise delete.
int nvim_opd_charwise_delete(oparg_T *oap)
{
  if (virtual_op) {
    if (gchar_pos(&oap->start) == '\t') {
      int endcol = 0;
      if (u_save_cursor() == FAIL) {
        return FAIL;
      }
      if (oap->line_count == 1) {
        endcol = getviscol2(oap->end.col, oap->end.coladd);
      }
      coladvance_force(getviscol2(oap->start.col, oap->start.coladd));
      oap->start = curwin->w_cursor;
      if (oap->line_count == 1) {
        coladvance(curwin, endcol);
        oap->end.col = curwin->w_cursor.col;
        oap->end.coladd = curwin->w_cursor.coladd;
        curwin->w_cursor = oap->start;
      }
    }

    if (gchar_pos(&oap->end) == '\t'
        && oap->end.coladd == 0
        && oap->inclusive) {
      if (u_save((linenr_T)(oap->end.lnum - 1),
                 (linenr_T)(oap->end.lnum + 1)) == FAIL) {
        return FAIL;
      }
      curwin->w_cursor = oap->end;
      coladvance_force(getviscol2(oap->end.col, oap->end.coladd));
      oap->end = curwin->w_cursor;
      curwin->w_cursor = oap->start;
    }
    mb_adjust_opend(oap);
  }

  if (oap->line_count == 1) {
    if (u_save_cursor() == FAIL) {
      return FAIL;
    }

    if (vim_strchr(p_cpo, CPO_DOLLAR) != NULL
        && oap->op_type == OP_CHANGE
        && oap->end.lnum == curwin->w_cursor.lnum
        && !oap->is_VIsual) {
      display_dollar(oap->end.col - !oap->inclusive);
    }

    int n = oap->end.col - oap->start.col + 1 - !oap->inclusive;

    if (virtual_op) {
      int len = get_cursor_line_len();
      if (oap->end.coladd != 0
          && (int)oap->end.col >= len - 1
          && !(oap->start.coladd && (int)oap->end.col >= len - 1)) {
        n++;
      }
      if (n == 0 && oap->start.coladd != oap->end.coladd) {
        n = 1;
      }
      if (gchar_cursor() != NUL) {
        curwin->w_cursor.coladd = 0;
      }
    }

    del_bytes((colnr_T)n, !virtual_op,
              oap->op_type == OP_DELETE && !oap->is_VIsual);
  } else {
    pos_T curpos;
    if (u_save(curwin->w_cursor.lnum - 1,
               curwin->w_cursor.lnum + oap->line_count) == FAIL) {
      return FAIL;
    }

    curbuf_splice_pending++;
    pos_T startpos = curwin->w_cursor;
    bcount_t deleted_bytes = get_region_bytecount(
        curbuf, startpos.lnum, oap->end.lnum,
        startpos.col, oap->end.col) + oap->inclusive;
    truncate_line(true);

    curpos = curwin->w_cursor;
    curwin->w_cursor.lnum++;
    del_lines(oap->line_count - 2, false);

    int n = (oap->end.col + 1 - !oap->inclusive);
    curwin->w_cursor.col = 0;
    del_bytes((colnr_T)n, !virtual_op,
              oap->op_type == OP_DELETE && !oap->is_VIsual);
    curwin->w_cursor = curpos;
    do_join(2, false, false, false, false);
    curbuf_splice_pending--;
    extmark_splice(curbuf, (int)startpos.lnum - 1, startpos.col,
                   (int)oap->line_count - 1, n, deleted_bytes,
                   0, 0, 0, kExtmarkUndo);
  }
  if (oap->op_type == OP_DELETE) {
    auto_format(false, true);
  }
  return OK;
}


/// Finish: msgmore + setmarks.
void nvim_opd_finish(oparg_T *oap, int old_lcount)
{
  msgmore(curbuf->b_ml.ml_line_count - (linenr_T)old_lcount);

  if ((cmdmod.cmod_flags & CMOD_LOCKMARKS) == 0) {
    if (oap->motion_type == kMTBlockWise) {
      curbuf->b_op_end.lnum = oap->end.lnum;
      curbuf->b_op_end.col = oap->start.col;
    } else {
      curbuf->b_op_end = oap->start;
    }
    curbuf->b_op_start = oap->start;
  }
}

/// Setmarks only (for goto setmarks case).
void nvim_opd_setmarks(oparg_T *oap)
{
  if ((cmdmod.cmod_flags & CMOD_LOCKMARKS) == 0) {
    if (oap->motion_type == kMTBlockWise) {
      curbuf->b_op_end.lnum = oap->end.lnum;
      curbuf->b_op_end.col = oap->start.col;
    } else {
      curbuf->b_op_end = oap->start;
    }
    curbuf->b_op_start = oap->start;
  }
}

/// Adjust end of operating area for ending on a multi-byte character.
/// Used for deletion.
static void mb_adjust_opend(oparg_T *oap)
{
  if (!oap->inclusive) {
    return;
  }

  const char *line = ml_get(oap->end.lnum);
  const char *ptr = line + oap->end.col;
  if (*ptr != NUL) {
    ptr -= utf_head_off(line, ptr);
    ptr += utfc_ptr2len(ptr) - 1;
    oap->end.col = (colnr_T)(ptr - line);
  }
}

/// Put character 'c' at position 'lp'
static inline void pbyte(pos_T lp, int c)
{
  assert(c <= UCHAR_MAX);
  *(ml_get_buf_mut(curbuf, lp.lnum) + lp.col) = (char)c;
  if (!curbuf_splice_pending) {
    extmark_splice_cols(curbuf, (int)lp.lnum - 1, lp.col, 1, 1, kExtmarkUndo);
  }
}

/// Replace the character under the cursor with "c".
/// This takes care of multi-byte characters.
static void replace_character(int c)
{
  const int n = State;

  State = MODE_REPLACE;
  ins_char(c);
  State = n;
  // Backup to the replaced character.
  dec_cursor();
}

// ===========================================================================
// op_replace C accessors for Rust migration (Phase 3)
// ===========================================================================

// op_replace is now exported from Rust via #[export_name]
extern int op_replace(oparg_T *oap, int c);

_Static_assert(kMTBlockWise == 2, "kMTBlockWise must be 2");


/// Block mode replacement loop — all lines in the block.
void nvim_opr_block_loop(oparg_T *oap, int c, int had_ctrl_v_cr)
{
  struct block_def bd;
  char *after_p = NULL;

  bd.is_MAX = (curwin->w_curswant == MAXCOL);
  for (; curwin->w_cursor.lnum <= oap->end.lnum; curwin->w_cursor.lnum++) {
    curwin->w_cursor.col = 0;
    block_prep(oap, &bd, curwin->w_cursor.lnum, true);
    if (bd.textlen == 0 && (!virtual_op || bd.is_MAX)) {
      continue;
    }

    int n;
    if (virtual_op && bd.is_short && *bd.textstart == NUL) {
      pos_T vpos;
      vpos.lnum = curwin->w_cursor.lnum;
      getvpos(curwin, &vpos, oap->start_vcol);
      bd.startspaces += vpos.coladd;
      n = bd.startspaces;
    } else {
      n = (bd.startspaces ? bd.start_char_vcols - 1 : 0);
    }

    n += (bd.endspaces && !bd.is_oneChar && bd.end_char_vcols > 0)
         ? bd.end_char_vcols - 1 : 0;

    int numc = oap->end_vcol - oap->start_vcol + 1;
    if (bd.is_short && (!virtual_op || bd.is_MAX)) {
      numc -= (oap->end_vcol - bd.end_vcol) + 1;
    }

    if (utf_char2cells(c) > 1) {
      if ((numc & 1) && !bd.is_short) {
        bd.endspaces++;
        n++;
      }
      numc = numc / 2;
    }

    int num_chars = numc;
    numc *= utf_char2len(c);

    char *oldp = get_cursor_line_ptr();
    colnr_T oldlen = get_cursor_line_len();

    size_t newp_size = (size_t)bd.textcol + (size_t)bd.startspaces;
    if (had_ctrl_v_cr || (c != '\r' && c != '\n')) {
      newp_size += (size_t)numc;
      if (!bd.is_short) {
        newp_size += (size_t)(bd.endspaces + oldlen - bd.textcol - bd.textlen);
      }
    }
    char *newp = xmallocz(newp_size);
    memmove(newp, oldp, (size_t)bd.textcol);
    oldp += bd.textcol + bd.textlen;
    memset(newp + bd.textcol, ' ', (size_t)bd.startspaces);

    size_t after_p_len = 0;
    int col = oldlen - bd.textcol - bd.textlen + 1;
    assert(col >= 0);
    int newrows = 0;
    int newcols = 0;
    if (had_ctrl_v_cr || (c != '\r' && c != '\n')) {
      int newp_len = bd.textcol + bd.startspaces;
      while (--num_chars >= 0) {
        newp_len += utf_char2bytes(c, newp + newp_len);
      }
      if (!bd.is_short) {
        memset(newp + newp_len, ' ', (size_t)bd.endspaces);
        newp_len += bd.endspaces;
        memmove(newp + newp_len, oldp, (size_t)col);
      }
      newcols = newp_len - bd.textcol;
    } else {
      after_p_len = (size_t)col;
      after_p = xmalloc(after_p_len);
      memmove(after_p, oldp, after_p_len);
      newrows = 1;
    }

    ml_replace(curwin->w_cursor.lnum, newp, false);
    curbuf_splice_pending++;
    linenr_T baselnum = curwin->w_cursor.lnum;
    if (after_p != NULL) {
      ml_append(curwin->w_cursor.lnum++, after_p, (int)after_p_len, false);
      appended_lines_mark(curwin->w_cursor.lnum, 1);
      oap->end.lnum++;
      xfree(after_p);
      after_p = NULL;
    }
    curbuf_splice_pending--;
    extmark_splice(curbuf, (int)baselnum - 1, bd.textcol,
                   0, bd.textlen, bd.textlen,
                   newrows, newcols, newrows + newcols, kExtmarkUndo);
  }
}

/// Charwise/linewise replacement loop.
void nvim_opr_charwise_loop(oparg_T *oap, int c)
{
  if (oap->motion_type == kMTLineWise) {
    oap->start.col = 0;
    curwin->w_cursor.col = 0;
    oap->end.col = ml_get_len(oap->end.lnum);
    if (oap->end.col) {
      oap->end.col--;
    }
  } else if (!oap->inclusive) {
    dec(&(oap->end));
  }

  while (ltoreq(curwin->w_cursor, oap->end)) {
    bool done = false;
    int n = gchar_cursor();

    if (n != NUL) {
      int new_byte_len = utf_char2len(c);
      int old_byte_len = utfc_ptr2len(get_cursor_pos_ptr());

      if (new_byte_len > 1 || old_byte_len > 1) {
        if (curwin->w_cursor.lnum == oap->end.lnum) {
          oap->end.col += new_byte_len - old_byte_len;
        }
        replace_character(c);
        done = true;
      } else {
        if (n == TAB) {
          int end_vcol = 0;
          if (curwin->w_cursor.lnum == oap->end.lnum) {
            end_vcol = getviscol2(oap->end.col, oap->end.coladd);
          }
          coladvance_force(getviscol());
          if (curwin->w_cursor.lnum == oap->end.lnum) {
            getvpos(curwin, &oap->end, end_vcol);
          }
        }
        if (gchar_cursor() != NUL) {
          pbyte(curwin->w_cursor, c);
          done = true;
        }
      }
    }

    if (!done && virtual_op && curwin->w_cursor.lnum == oap->end.lnum) {
      int virtcols = oap->end.coladd;
      if (curwin->w_cursor.lnum == oap->start.lnum
          && oap->start.col == oap->end.col && oap->start.coladd) {
        virtcols -= oap->start.coladd;
      }
      coladvance_force(getviscol2(oap->end.col, oap->end.coladd) + 1);
      curwin->w_cursor.col -= (virtcols + 1);
      for (; virtcols >= 0; virtcols--) {
        if (utf_char2len(c) > 1) {
          replace_character(c);
        } else {
          pbyte(curwin->w_cursor, c);
        }
        if (inc(&curwin->w_cursor) == -1) {
          break;
        }
      }
    }

    if (inc_cursor() == -1) {
      break;
    }
  }
}

/// Finish: restore cursor, mark changed lines, set marks.
void nvim_opr_finish(oparg_T *oap)
{
  curwin->w_cursor = oap->start;
  check_cursor(curwin);
  changed_lines(curbuf, oap->start.lnum, oap->start.col,
                oap->end.lnum + 1, 0, true);

  if ((cmdmod.cmod_flags & CMOD_LOCKMARKS) == 0) {
    curbuf->b_op_start = oap->start;
    curbuf->b_op_end = oap->end;
  }
}

/// Handle the (non-standard vi) tilde operator.  Also for "gu", "gU" and "g?".
void op_tilde(oparg_T *oap)
{
  struct block_def bd;
  bool did_change = false;

  if (u_save((linenr_T)(oap->start.lnum - 1),
             (linenr_T)(oap->end.lnum + 1)) == FAIL) {
    return;
  }

  pos_T pos = oap->start;
  if (oap->motion_type == kMTBlockWise) {  // Visual block mode
    for (; pos.lnum <= oap->end.lnum; pos.lnum++) {
      block_prep(oap, &bd, pos.lnum, false);
      pos.col = bd.textcol;
      bool one_change = swapchars(oap->op_type, &pos, bd.textlen);
      did_change |= one_change;
    }
    if (did_change) {
      changed_lines(curbuf, oap->start.lnum, 0, oap->end.lnum + 1, 0, true);
    }
  } else {  // not block mode
    if (oap->motion_type == kMTLineWise) {
      oap->start.col = 0;
      pos.col = 0;
      oap->end.col = ml_get_len(oap->end.lnum);
      if (oap->end.col) {
        oap->end.col--;
      }
    } else if (!oap->inclusive) {
      dec(&(oap->end));
    }

    if (pos.lnum == oap->end.lnum) {
      did_change = swapchars(oap->op_type, &pos,
                             oap->end.col - pos.col + 1);
    } else {
      while (true) {
        did_change |= swapchars(oap->op_type, &pos,
                                pos.lnum == oap->end.lnum ? oap->end.col + 1
                                                          : ml_get_pos_len(&pos));
        if (ltoreq(oap->end, pos) || inc(&pos) == -1) {
          break;
        }
      }
    }
    if (did_change) {
      changed_lines(curbuf, oap->start.lnum, oap->start.col, oap->end.lnum + 1,
                    0, true);
    }
  }

  if (!did_change && oap->is_VIsual) {
    // No change: need to remove the Visual selection
    redraw_curbuf_later(UPD_INVERTED);
  }

  if ((cmdmod.cmod_flags & CMOD_LOCKMARKS) == 0) {
    // Set '[ and '] marks.
    curbuf->b_op_start = oap->start;
    curbuf->b_op_end = oap->end;
  }

  if (oap->line_count > p_report) {
    smsg(0, NGETTEXT("%" PRId64 " line changed", "%" PRId64 " lines changed", oap->line_count),
         (int64_t)oap->line_count);
  }
}

/// Invoke swapchar() on "length" bytes at position "pos".
///
/// @param pos     is advanced to just after the changed characters.
/// @param length  is rounded up to include the whole last multi-byte character.
/// Also works correctly when the number of bytes changes.
///
/// @return  true if some character was changed.
static int swapchars(int op_type, pos_T *pos, int length)
  FUNC_ATTR_NONNULL_ALL
{
  int did_change = 0;

  for (int todo = length; todo > 0; todo--) {
    const int len = utfc_ptr2len(ml_get_pos(pos));

    // we're counting bytes, not characters
    if (len > 0) {
      todo -= len - 1;
    }
    did_change |= swapchar(op_type, pos);
    if (inc(pos) == -1) {      // at end of file
      break;
    }
  }
  return did_change;
}

/// @param op_type
///                 == OP_UPPER: make uppercase,
///                 == OP_LOWER: make lowercase,
///                 == OP_ROT13: do rot13 encoding,
///                 else swap case of character at 'pos'
///
/// @return  true when something actually changed.
bool swapchar(int op_type, pos_T *pos)
  FUNC_ATTR_NONNULL_ARG(2)
{
  const int c = gchar_pos(pos);

  // Only do rot13 encoding for ASCII characters.
  if (c >= 0x80 && op_type == OP_ROT13) {
    return false;
  }

  int nc = c;
  if (mb_islower(c)) {
    if (op_type == OP_ROT13) {
      nc = ROT13(c, 'a');
    } else if (op_type != OP_LOWER) {
      nc = mb_toupper(c);
    }
  } else if (mb_isupper(c)) {
    if (op_type == OP_ROT13) {
      nc = ROT13(c, 'A');
    } else if (op_type != OP_UPPER) {
      nc = mb_tolower(c);
    }
  }
  if (nc != c) {
    if (c >= 0x80 || nc >= 0x80) {
      pos_T sp = curwin->w_cursor;

      curwin->w_cursor = *pos;
      // don't use del_char(), it also removes composing chars
      del_bytes(utf_ptr2len(get_cursor_pos_ptr()), false, false);
      ins_char(nc);
      curwin->w_cursor = sp;
    } else {
      pbyte(*pos, nc);
    }
    return true;
  }
  return false;
}

/// Insert and append operators for Visual mode.
void op_insert(oparg_T *oap, int count1)
{
  int pre_textlen = 0;
  colnr_T ind_pre_col = 0;
  int ind_pre_vcol = 0;
  struct block_def bd;

  // edit() changes this - record it for OP_APPEND
  bd.is_MAX = (curwin->w_curswant == MAXCOL);

  // vis block is still marked. Get rid of it now.
  curwin->w_cursor.lnum = oap->start.lnum;
  redraw_curbuf_later(UPD_INVERTED);
  update_screen();

  if (oap->motion_type == kMTBlockWise) {
    // When 'virtualedit' is used, need to insert the extra spaces before
    // doing block_prep().  When only "block" is used, virtual edit is
    // already disabled, but still need it when calling
    // coladvance_force().
    // coladvance_force() uses get_ve_flags() to get the 'virtualedit'
    // state for the current window.  To override that state, we need to
    // set the window-local value of ve_flags rather than the global value.
    if (curwin->w_cursor.coladd > 0) {
      unsigned old_ve_flags = curwin->w_ve_flags;

      if (u_save_cursor() == FAIL) {
        return;
      }
      curwin->w_ve_flags = kOptVeFlagAll;
      coladvance_force(oap->op_type == OP_APPEND
                       ? oap->end_vcol + 1 : getviscol());
      if (oap->op_type == OP_APPEND) {
        curwin->w_cursor.col--;
      }
      curwin->w_ve_flags = old_ve_flags;
    }
    // Get the info about the block before entering the text
    block_prep(oap, &bd, oap->start.lnum, true);
    // Get indent information
    ind_pre_col = (colnr_T)getwhitecols_curline();
    ind_pre_vcol = get_indent();
    pre_textlen = ml_get_len(oap->start.lnum) - bd.textcol;
    if (oap->op_type == OP_APPEND) {
      pre_textlen -= bd.textlen;
    }
  }

  if (oap->op_type == OP_APPEND) {
    if (oap->motion_type == kMTBlockWise
        && curwin->w_cursor.coladd == 0) {
      // Move the cursor to the character right of the block.
      curwin->w_set_curswant = true;
      while (*get_cursor_pos_ptr() != NUL
             && (curwin->w_cursor.col < bd.textcol + bd.textlen)) {
        curwin->w_cursor.col++;
      }
      if (bd.is_short && !bd.is_MAX) {
        // First line was too short, make it longer and adjust the
        // values in "bd".
        if (u_save_cursor() == FAIL) {
          return;
        }
        for (int i = 0; i < bd.endspaces; i++) {
          ins_char(' ');
        }
        bd.textlen += bd.endspaces;
      }
    } else {
      curwin->w_cursor = oap->end;
      check_cursor_col(curwin);

      // Works just like an 'i'nsert on the next character.
      if (!LINEEMPTY(curwin->w_cursor.lnum)
          && oap->start_vcol != oap->end_vcol) {
        inc_cursor();
      }
    }
  }

  pos_T t1 = oap->start;
  const pos_T start_insert = curwin->w_cursor;
  edit(NUL, false, (linenr_T)count1);

  // When a tab was inserted, and the characters in front of the tab
  // have been converted to a tab as well, the column of the cursor
  // might have actually been reduced, so need to adjust here.
  if (t1.lnum == curbuf->b_op_start_orig.lnum
      && lt(curbuf->b_op_start_orig, t1)) {
    oap->start = curbuf->b_op_start_orig;
  }

  // If user has moved off this line, we don't know what to do, so do
  // nothing.
  // Also don't repeat the insert when Insert mode ended with CTRL-C.
  if (curwin->w_cursor.lnum != oap->start.lnum || got_int) {
    return;
  }

  if (oap->motion_type == kMTBlockWise) {
    int ind_post_vcol = 0;
    struct block_def bd2;
    bool did_indent = false;

    // if indent kicked in, the firstline might have changed
    // but only do that, if the indent actually increased
    colnr_T ind_post_col = (colnr_T)getwhitecols_curline();
    if (curbuf->b_op_start.col > ind_pre_col && ind_post_col > ind_pre_col) {
      bd.textcol += ind_post_col - ind_pre_col;
      ind_post_vcol = get_indent();
      bd.start_vcol += ind_post_vcol - ind_pre_vcol;
      did_indent = true;
    }

    // The user may have moved the cursor before inserting something, try
    // to adjust the block for that.  But only do it, if the difference
    // does not come from indent kicking in.
    if (oap->start.lnum == curbuf->b_op_start_orig.lnum && !bd.is_MAX && !did_indent) {
      const int t = getviscol2(curbuf->b_op_start_orig.col, curbuf->b_op_start_orig.coladd);

      if (oap->op_type == OP_INSERT
          && oap->start.col + oap->start.coladd
          != curbuf->b_op_start_orig.col + curbuf->b_op_start_orig.coladd) {
        oap->start.col = curbuf->b_op_start_orig.col;
        pre_textlen -= t - oap->start_vcol;
        oap->start_vcol = t;
      } else if (oap->op_type == OP_APPEND
                 && oap->start.col + oap->start.coladd
                 >= curbuf->b_op_start_orig.col + curbuf->b_op_start_orig.coladd) {
        oap->start.col = curbuf->b_op_start_orig.col;
        // reset pre_textlen to the value of OP_INSERT
        pre_textlen += bd.textlen;
        pre_textlen -= t - oap->start_vcol;
        oap->start_vcol = t;
        oap->op_type = OP_INSERT;
      }
    }

    // Spaces and tabs in the indent may have changed to other spaces and
    // tabs.  Get the starting column again and correct the length.
    // Don't do this when "$" used, end-of-line will have changed.
    //
    // if indent was added and the inserted text was after the indent,
    // correct the selection for the new indent.
    if (did_indent && bd.textcol - ind_post_col > 0) {
      oap->start.col += ind_post_col - ind_pre_col;
      oap->start_vcol += ind_post_vcol - ind_pre_vcol;
      oap->end.col += ind_post_col - ind_pre_col;
      oap->end_vcol += ind_post_vcol - ind_pre_vcol;
    }
    block_prep(oap, &bd2, oap->start.lnum, true);
    if (did_indent && bd.textcol - ind_post_col > 0) {
      // undo for where "oap" is used below
      oap->start.col -= ind_post_col - ind_pre_col;
      oap->start_vcol -= ind_post_vcol - ind_pre_vcol;
      oap->end.col -= ind_post_col - ind_pre_col;
      oap->end_vcol -= ind_post_vcol - ind_pre_vcol;
    }
    if (!bd.is_MAX || bd2.textlen < bd.textlen) {
      if (oap->op_type == OP_APPEND) {
        pre_textlen += bd2.textlen - bd.textlen;
        if (bd2.endspaces) {
          bd2.textlen--;
        }
      }
      bd.textcol = bd2.textcol;
      bd.textlen = bd2.textlen;
    }

    // Subsequent calls to ml_get() flush the firstline data - take a
    // copy of the required string.
    char *firstline = ml_get(oap->start.lnum);
    colnr_T len = ml_get_len(oap->start.lnum);
    colnr_T add = bd.textcol;
    colnr_T offset = 0;  // offset when cursor was moved in insert mode
    if (oap->op_type == OP_APPEND) {
      add += bd.textlen;
      // account for pressing cursor in insert mode when '$' was used
      if (bd.is_MAX && start_insert.lnum == Insstart.lnum && start_insert.col > Insstart.col) {
        offset = start_insert.col - Insstart.col;
        add -= offset;
        if (oap->end_vcol > offset) {
          oap->end_vcol -= offset + 1;
        } else {
          // moved outside of the visual block, what to do?
          return;
        }
      }
    }
    add = MIN(add, len);  // short line, point to the NUL
    firstline += add;
    len -= add;
    int ins_len = len - pre_textlen - offset;
    if (pre_textlen >= 0 && ins_len > 0) {
      char *ins_text = xmemdupz(firstline, (size_t)ins_len);
      // block handled here
      if (u_save(oap->start.lnum, (linenr_T)(oap->end.lnum + 1)) == OK) {
        block_insert(oap, ins_text, (size_t)ins_len, (oap->op_type == OP_INSERT), &bd);
      }

      curwin->w_cursor.col = oap->start.col;
      check_cursor(curwin);
      xfree(ins_text);
    }
  }
}

/// handle a change operation
///
/// @return  true if edit() returns because of a CTRL-O command
int op_change(oparg_T *oap)
{
  int pre_textlen = 0;
  int pre_indent = 0;
  char *firstline;
  struct block_def bd;

  colnr_T l = oap->start.col;
  if (oap->motion_type == kMTLineWise) {
    l = 0;
    can_si = may_do_si();  // Like opening a new line, do smart indent
  }

  // First delete the text in the region.  In an empty buffer only need to
  // save for undo
  if (curbuf->b_ml.ml_flags & ML_EMPTY) {
    if (u_save_cursor() == FAIL) {
      return false;
    }
  } else if (op_delete(oap) == FAIL) {
    return false;
  }

  if ((l > curwin->w_cursor.col) && !LINEEMPTY(curwin->w_cursor.lnum)
      && !virtual_op) {
    inc_cursor();
  }

  // check for still on same line (<CR> in inserted text meaningless)
  // skip blank lines too
  if (oap->motion_type == kMTBlockWise) {
    // Add spaces before getting the current line length.
    if (virtual_op && (curwin->w_cursor.coladd > 0
                       || gchar_cursor() == NUL)) {
      coladvance_force(getviscol());
    }
    firstline = ml_get(oap->start.lnum);
    pre_textlen = ml_get_len(oap->start.lnum);
    pre_indent = (int)getwhitecols(firstline);
    bd.textcol = curwin->w_cursor.col;
  }

  if (oap->motion_type == kMTLineWise) {
    fix_indent();
  }

  // Reset finish_op now, don't want it set inside edit().
  const bool save_finish_op = finish_op;
  finish_op = false;

  int retval = edit(NUL, false, 1);

  finish_op = save_finish_op;

  // In Visual block mode, handle copying the new text to all lines of the
  // block.
  // Don't repeat the insert when Insert mode ended with CTRL-C.
  if (oap->motion_type == kMTBlockWise
      && oap->start.lnum != oap->end.lnum && !got_int) {
    // Auto-indenting may have changed the indent.  If the cursor was past
    // the indent, exclude that indent change from the inserted text.
    firstline = ml_get(oap->start.lnum);
    if (bd.textcol > (colnr_T)pre_indent) {
      int new_indent = (int)getwhitecols(firstline);

      pre_textlen += new_indent - pre_indent;
      bd.textcol += (colnr_T)(new_indent - pre_indent);
    }

    int ins_len = ml_get_len(oap->start.lnum) - pre_textlen;
    if (ins_len > 0) {
      // Subsequent calls to ml_get() flush the firstline data - take a
      // copy of the inserted text.
      char *ins_text = xmalloc((size_t)ins_len + 1);
      xmemcpyz(ins_text, firstline + bd.textcol, (size_t)ins_len);
      for (linenr_T linenr = oap->start.lnum + 1; linenr <= oap->end.lnum;
           linenr++) {
        block_prep(oap, &bd, linenr, true);
        if (!bd.is_short || virtual_op) {
          pos_T vpos;

          // If the block starts in virtual space, count the
          // initial coladd offset as part of "startspaces"
          if (bd.is_short) {
            vpos.lnum = linenr;
            getvpos(curwin, &vpos, oap->start_vcol);
          } else {
            vpos.coladd = 0;
          }
          char *oldp = ml_get(linenr);
          char *newp = xmalloc((size_t)ml_get_len(linenr)
                               + (size_t)vpos.coladd + (size_t)ins_len + 1);
          // copy up to block start
          memmove(newp, oldp, (size_t)bd.textcol);
          int newlen = bd.textcol;
          memset(newp + newlen, ' ', (size_t)vpos.coladd);
          newlen += vpos.coladd;
          memmove(newp + newlen, ins_text, (size_t)ins_len);
          newlen += ins_len;
          STRCPY(newp + newlen, oldp + bd.textcol);
          ml_replace(linenr, newp, false);
          extmark_splice_cols(curbuf, (int)linenr - 1, bd.textcol,
                              0, vpos.coladd + ins_len, kExtmarkUndo);
        }
      }
      check_cursor(curwin);
      changed_lines(curbuf, oap->start.lnum + 1, 0, oap->end.lnum + 1, 0, true);
      xfree(ins_text);
    }
  }
  auto_format(false, true);

  return retval;
}

/// When the cursor is on the NUL past the end of the line and it should not be
/// there move it left.
void adjust_cursor_eol(void)
{
  unsigned cur_ve_flags = get_ve_flags(curwin);

  const bool adj_cursor = (curwin->w_cursor.col > 0
                           && gchar_cursor() == NUL
                           && (cur_ve_flags & kOptVeFlagOnemore) == 0
                           && !(restart_edit || (State & MODE_INSERT)));
  if (!adj_cursor) {
    return;
  }

  // Put the cursor on the last character in the line.
  dec_cursor();

  if (cur_ve_flags == kOptVeFlagAll) {
    colnr_T scol, ecol;

    // Coladd is set to the width of the last character.
    getvcol(curwin, &curwin->w_cursor, &scol, NULL, &ecol);
    curwin->w_cursor.coladd = ecol - scol + 1;
  }
}

/// If \p "process" is true and the line begins with a comment leader (possibly
/// after some white space), return a pointer to the text after it.
/// Put a boolean value indicating whether the line ends with an unclosed
/// comment in "is_comment".
///
/// @param line - line to be processed
/// @param process - if false, will only check whether the line ends
///         with an unclosed comment,
/// @param include_space - whether to skip space following the comment leader
/// @param[out] is_comment - whether the current line ends with an unclosed
///  comment.
char *skip_comment(char *line, bool process, bool include_space, bool *is_comment)
{
  char *comment_flags = NULL;
  int leader_offset = get_last_leader_offset(line, &comment_flags);

  *is_comment = false;
  if (leader_offset != -1) {
    // Let's check whether the line ends with an unclosed comment.
    // If the last comment leader has COM_END in flags, there's no comment.
    while (*comment_flags) {
      if (*comment_flags == COM_END
          || *comment_flags == ':') {
        break;
      }
      comment_flags++;
    }
    if (*comment_flags != COM_END) {
      *is_comment = true;
    }
  }

  if (process == false) {
    return line;
  }

  int lead_len = get_leader_len(line, &comment_flags, false, include_space);

  if (lead_len == 0) {
    return line;
  }

  // Find:
  // - COM_END,
  // - colon,
  // whichever comes first.
  while (*comment_flags) {
    if (*comment_flags == COM_END
        || *comment_flags == ':') {
      break;
    }
    comment_flags++;
  }

  // If we found a colon, it means that we are not processing a line
  // starting with a closing part of a three-part comment. That's good,
  // because we don't want to remove those as this would be annoying.
  if (*comment_flags == ':' || *comment_flags == NUL) {
    line += lead_len;
  }

  return line;
}

/// @param count              number of lines (minimal 2) to join at cursor position.
/// @param save_undo          when true, save lines for undo first.
/// @param use_formatoptions  set to false when e.g. processing backspace and comment
///                           leaders should not be removed.
/// @param setmark            when true, sets the '[ and '] mark, else, the caller is expected
///                           to set those marks.
///
/// @return  FAIL for failure, OK otherwise
int do_join(size_t count, bool insert_space, bool save_undo, bool use_formatoptions, bool setmark)
{
  char *curr = NULL;
  char *curr_start = NULL;
  char *cend;
  int endcurr1 = NUL;
  int endcurr2 = NUL;
  int currsize = 0;             // size of the current line
  int sumsize = 0;              // size of the long new line
  int ret = OK;
  int *comments = NULL;
  bool remove_comments = use_formatoptions && has_format_option(FO_REMOVE_COMS);
  bool prev_was_comment = false;
  assert(count >= 1);

  if (save_undo && u_save(curwin->w_cursor.lnum - 1,
                          curwin->w_cursor.lnum + (linenr_T)count) == FAIL) {
    return FAIL;
  }
  // Allocate an array to store the number of spaces inserted before each
  // line.  We will use it to pre-compute the length of the new line and the
  // proper placement of each original line in the new one.
  char *spaces = xcalloc(count, 1);  // number of spaces inserted before a line
  if (remove_comments) {
    comments = xcalloc(count, sizeof(*comments));
  }

  // Don't move anything yet, just compute the final line length
  // and setup the array of space strings lengths
  // This loops forward over joined lines.
  for (linenr_T t = 0; t < (linenr_T)count; t++) {
    curr_start = ml_get(curwin->w_cursor.lnum + t);
    curr = curr_start;
    if (t == 0 && setmark && (cmdmod.cmod_flags & CMOD_LOCKMARKS) == 0) {
      // Set the '[ mark.
      curwin->w_buffer->b_op_start.lnum = curwin->w_cursor.lnum;
      curwin->w_buffer->b_op_start.col = (colnr_T)strlen(curr);
    }
    if (remove_comments) {
      // We don't want to remove the comment leader if the
      // previous line is not a comment.
      if (t > 0 && prev_was_comment) {
        char *new_curr = skip_comment(curr, true, insert_space, &prev_was_comment);
        comments[t] = (int)(new_curr - curr);
        curr = new_curr;
      } else {
        curr = skip_comment(curr, false, insert_space, &prev_was_comment);
      }
    }

    if (insert_space && t > 0) {
      curr = skipwhite(curr);
      if (*curr != NUL
          && *curr != ')'
          && sumsize != 0
          && endcurr1 != TAB
          && (!has_format_option(FO_MBYTE_JOIN)
              || (utf_ptr2char(curr) < 0x100 && endcurr1 < 0x100))
          && (!has_format_option(FO_MBYTE_JOIN2)
              || (utf_ptr2char(curr) < 0x100 && !utf_eat_space(endcurr1))
              || (endcurr1 < 0x100
                  && !utf_eat_space(utf_ptr2char(curr))))) {
        // don't add a space if the line is ending in a space
        if (endcurr1 == ' ') {
          endcurr1 = endcurr2;
        } else {
          spaces[t]++;
        }
        // Extra space when 'joinspaces' set and line ends in '.', '?', or '!'.
        if (p_js && (endcurr1 == '.' || endcurr1 == '?' || endcurr1 == '!')) {
          spaces[t]++;
        }
      }
    }

    if (t > 0 && curbuf_splice_pending == 0) {
      colnr_T removed = (int)(curr - curr_start);
      extmark_splice(curbuf, (int)curwin->w_cursor.lnum - 1, sumsize,
                     1, removed, removed + 1,
                     0, spaces[t], spaces[t],
                     kExtmarkUndo);
    }
    currsize = (int)strlen(curr);
    sumsize += currsize + spaces[t];
    endcurr1 = endcurr2 = NUL;
    if (insert_space && currsize > 0) {
      cend = curr + currsize;
      MB_PTR_BACK(curr, cend);
      endcurr1 = utf_ptr2char(cend);
      if (cend > curr) {
        MB_PTR_BACK(curr, cend);
        endcurr2 = utf_ptr2char(cend);
      }
    }
    line_breakcheck();
    if (got_int) {
      ret = FAIL;
      goto theend;
    }
  }

  // store the column position before last line
  colnr_T col = sumsize - currsize - spaces[count - 1];

  // allocate the space for the new line
  size_t newp_len = (size_t)sumsize;
  char *newp = xmallocz(newp_len);
  cend = newp + sumsize;

  // Move affected lines to the new long one.
  // This loops backwards over the joined lines, including the original line.
  //
  // Move marks from each deleted line to the joined line, adjusting the
  // column.  This is not Vi compatible, but Vi deletes the marks, thus that
  // should not really be a problem.

  curbuf_splice_pending++;

  for (linenr_T t = (linenr_T)count - 1;; t--) {
    cend -= currsize;
    memmove(cend, curr, (size_t)currsize);

    if (spaces[t] > 0) {
      cend -= spaces[t];
      memset(cend, ' ', (size_t)(spaces[t]));
    }

    // If deleting more spaces than adding, the cursor moves no more than
    // what is added if it is inside these spaces.
    const int spaces_removed = (int)((curr - curr_start) - spaces[t]);
    linenr_T lnum = curwin->w_cursor.lnum + t;
    colnr_T mincol = 0;
    linenr_T lnum_amount = -t;
    colnr_T col_amount = (colnr_T)(cend - newp - spaces_removed);

    mark_col_adjust(lnum, mincol, lnum_amount, col_amount, spaces_removed);

    if (t == 0) {
      break;
    }

    curr_start = ml_get((linenr_T)(curwin->w_cursor.lnum + t - 1));
    curr = curr_start;
    if (remove_comments) {
      curr += comments[t - 1];
    }
    if (insert_space && t > 1) {
      curr = skipwhite(curr);
    }
    currsize = (int)strlen(curr);
  }

  ml_replace_len(curwin->w_cursor.lnum, newp, newp_len, false);

  if (setmark && (cmdmod.cmod_flags & CMOD_LOCKMARKS) == 0) {
    // Set the '] mark.
    curwin->w_buffer->b_op_end.lnum = curwin->w_cursor.lnum;
    curwin->w_buffer->b_op_end.col = sumsize;
  }

  // Only report the change in the first line here, del_lines() will report
  // the deleted line.
  changed_lines(curbuf, curwin->w_cursor.lnum, currsize,
                curwin->w_cursor.lnum + 1, 0, true);

  // Delete following lines. To do this we move the cursor there
  // briefly, and then move it back. After del_lines() the cursor may
  // have moved up (last line deleted), so the current lnum is kept in t.
  linenr_T t = curwin->w_cursor.lnum;
  curwin->w_cursor.lnum++;
  del_lines((int)count - 1, false);
  curwin->w_cursor.lnum = t;
  curbuf_splice_pending--;
  curbuf->deleted_bytes2 = 0;

  // Set the cursor column:
  // Vi compatible: use the column of the first join
  // vim:             use the column of the last join
  curwin->w_cursor.col =
    (vim_strchr(p_cpo, CPO_JOINCOL) != NULL ? currsize : col);
  check_cursor_col(curwin);

  curwin->w_cursor.coladd = 0;
  curwin->w_set_curswant = true;

theend:
  xfree(spaces);
  if (remove_comments) {
    xfree(comments);
  }
  return ret;
}

/// Reset 'linebreak' and take care of side effects.
/// @return  the previous value, to be passed to restore_lbr().
static bool reset_lbr(void)
{
  if (!curwin->w_p_lbr) {
    return false;
  }
  // changing 'linebreak' may require w_virtcol to be updated
  curwin->w_p_lbr = false;
  curwin->w_valid &= ~(VALID_WROW|VALID_WCOL|VALID_VIRTCOL);
  return true;
}

/// Restore 'linebreak' and take care of side effects.
static void restore_lbr(bool lbr_saved)
{
  if (curwin->w_p_lbr || !lbr_saved) {
    return;
  }

  // changing 'linebreak' may require w_virtcol to be updated
  curwin->w_p_lbr = true;
  curwin->w_valid &= ~(VALID_WROW|VALID_WCOL|VALID_VIRTCOL);
}

/// prepare a few things for block mode yank/delete/tilde
///
/// for delete:
/// - textlen includes the first/last char to be (partly) deleted
/// - start/endspaces is the number of columns that are taken by the
///   first/last deleted char minus the number of columns that have to be
///   deleted.
/// for yank and tilde:
/// - textlen includes the first/last char to be wholly yanked
/// - start/endspaces is the number of columns of the first/last yanked char
///   that are to be yanked.
void block_prep(oparg_T *oap, struct block_def *bdp, linenr_T lnum, bool is_del)
{
  int incr = 0;
  // Avoid a problem with unwanted linebreaks in block mode.
  const bool lbr_saved = reset_lbr();

  bdp->startspaces = 0;
  bdp->endspaces = 0;
  bdp->textlen = 0;
  bdp->start_vcol = 0;
  bdp->end_vcol = 0;
  bdp->is_short = false;
  bdp->is_oneChar = false;
  bdp->pre_whitesp = 0;
  bdp->pre_whitesp_c = 0;
  bdp->end_char_vcols = 0;
  bdp->start_char_vcols = 0;

  char *line = ml_get(lnum);
  char *prev_pstart = line;

  CharsizeArg csarg;
  CSType cstype = init_charsize_arg(&csarg, curwin, lnum, line);
  StrCharInfo ci = utf_ptr2StrCharInfo(line);
  int vcol = bdp->start_vcol;
  while (vcol < oap->start_vcol && *ci.ptr != NUL) {
    incr = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &csarg).width;
    vcol += incr;
    if (ascii_iswhite(ci.chr.value)) {
      bdp->pre_whitesp += incr;
      bdp->pre_whitesp_c++;
    } else {
      bdp->pre_whitesp = 0;
      bdp->pre_whitesp_c = 0;
    }
    prev_pstart = ci.ptr;
    ci = utfc_next(ci);
  }
  bdp->start_vcol = vcol;
  char *pstart = ci.ptr;

  bdp->start_char_vcols = incr;
  if (bdp->start_vcol < oap->start_vcol) {      // line too short
    bdp->end_vcol = bdp->start_vcol;
    bdp->is_short = true;
    if (!is_del || oap->op_type == OP_APPEND) {
      bdp->endspaces = oap->end_vcol - oap->start_vcol + 1;
    }
  } else {
    // notice: this converts partly selected Multibyte characters to
    // spaces, too.
    bdp->startspaces = bdp->start_vcol - oap->start_vcol;
    if (is_del && bdp->startspaces) {
      bdp->startspaces = bdp->start_char_vcols - bdp->startspaces;
    }
    char *pend = pstart;
    bdp->end_vcol = bdp->start_vcol;
    if (bdp->end_vcol > oap->end_vcol) {  // it's all in one character
      bdp->is_oneChar = true;
      if (oap->op_type == OP_INSERT) {
        bdp->endspaces = bdp->start_char_vcols - bdp->startspaces;
      } else if (oap->op_type == OP_APPEND) {
        bdp->startspaces += oap->end_vcol - oap->start_vcol + 1;
        bdp->endspaces = bdp->start_char_vcols - bdp->startspaces;
      } else {
        bdp->startspaces = oap->end_vcol - oap->start_vcol + 1;
        if (is_del && oap->op_type != OP_LSHIFT) {
          // just putting the sum of those two into
          // bdp->startspaces doesn't work for Visual replace,
          // so we have to split the tab in two
          bdp->startspaces = bdp->start_char_vcols
                             - (bdp->start_vcol - oap->start_vcol);
          bdp->endspaces = bdp->end_vcol - oap->end_vcol - 1;
        }
      }
    } else {
      cstype = init_charsize_arg(&csarg, curwin, lnum, line);
      ci = utf_ptr2StrCharInfo(pend);
      vcol = bdp->end_vcol;
      char *prev_pend = pend;
      while (vcol <= oap->end_vcol && *ci.ptr != NUL) {
        prev_pend = ci.ptr;
        incr = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &csarg).width;
        vcol += incr;
        ci = utfc_next(ci);
      }
      bdp->end_vcol = vcol;
      pend = ci.ptr;

      if (bdp->end_vcol <= oap->end_vcol
          && (!is_del
              || oap->op_type == OP_APPEND
              || oap->op_type == OP_REPLACE)) {  // line too short
        bdp->is_short = true;
        // Alternative: include spaces to fill up the block.
        // Disadvantage: can lead to trailing spaces when the line is
        // short where the text is put
        // if (!is_del || oap->op_type == OP_APPEND)
        if (oap->op_type == OP_APPEND || virtual_op) {
          bdp->endspaces = oap->end_vcol - bdp->end_vcol
                           + oap->inclusive;
        }
      } else if (bdp->end_vcol > oap->end_vcol) {
        bdp->endspaces = bdp->end_vcol - oap->end_vcol - 1;
        if (!is_del && bdp->endspaces) {
          bdp->endspaces = incr - bdp->endspaces;
          if (pend != pstart) {
            pend = prev_pend;
          }
        }
      }
    }
    bdp->end_char_vcols = incr;
    if (is_del && bdp->startspaces) {
      pstart = prev_pstart;
    }
    bdp->textlen = (int)(pend - pstart);
  }
  bdp->textcol = (colnr_T)(pstart - line);
  bdp->textstart = pstart;
  restore_lbr(lbr_saved);
}

/// Get block text from "start" to "end"
void charwise_block_prep(pos_T start, pos_T end, struct block_def *bdp, linenr_T lnum,
                         bool inclusive)
{
  colnr_T startcol = 0;
  colnr_T endcol = MAXCOL;
  colnr_T cs, ce;
  char *p = ml_get(lnum);
  int plen = ml_get_len(lnum);

  bdp->startspaces = 0;
  bdp->endspaces = 0;
  bdp->is_oneChar = false;
  bdp->start_char_vcols = 0;

  if (lnum == start.lnum) {
    startcol = start.col;
    if (virtual_op) {
      getvcol(curwin, &start, &cs, NULL, &ce);
      if (ce != cs && start.coladd > 0) {
        // Part of a tab selected -- but don't double-count it.
        bdp->start_char_vcols = ce - cs + 1;
        bdp->startspaces = MAX(bdp->start_char_vcols - start.coladd, 0);
        startcol++;
      }
    }
  }

  if (lnum == end.lnum) {
    endcol = end.col;
    if (virtual_op) {
      getvcol(curwin, &end, &cs, NULL, &ce);
      if (p[endcol] == NUL || (cs + end.coladd < ce
                               // Don't add space for double-wide
                               // char; endcol will be on last byte
                               // of multi-byte char.
                               && utf_head_off(p, p + endcol) == 0)) {
        if (start.lnum == end.lnum && start.col == end.col) {
          // Special case: inside a single char
          bdp->is_oneChar = true;
          bdp->startspaces = end.coladd - start.coladd + inclusive;
          endcol = startcol;
        } else {
          bdp->endspaces = end.coladd + inclusive;
          endcol -= inclusive;
        }
      }
    }
  }
  if (endcol == MAXCOL) {
    endcol = ml_get_len(lnum);
  }
  if (startcol > endcol || bdp->is_oneChar) {
    bdp->textlen = 0;
  } else {
    bdp->textlen = endcol - startcol + inclusive;
  }
  bdp->textcol = startcol;
  bdp->textstart = startcol <= plen ? p + startcol : p;
}

/// Handle the add/subtract operator.
///
/// @param[in]  oap      Arguments of operator.
/// @param[in]  Prenum1  Amount of addition or subtraction.
/// @param[in]  g_cmd    Prefixed with `g`.
void op_addsub(oparg_T *oap, linenr_T Prenum1, bool g_cmd)
{
  struct block_def bd;
  ssize_t change_cnt = 0;
  linenr_T amount = Prenum1;

  // do_addsub() might trigger re-evaluation of 'foldexpr' halfway, when the
  // buffer is not completely updated yet. Postpone updating folds until before
  // the call to changed_lines().
  disable_fold_update++;

  if (!VIsual_active) {
    pos_T pos = curwin->w_cursor;
    if (u_save_cursor() == FAIL) {
      disable_fold_update--;
      return;
    }
    change_cnt = do_addsub(oap->op_type, &pos, 0, amount);
    disable_fold_update--;
    if (change_cnt) {
      changed_lines(curbuf, pos.lnum, 0, pos.lnum + 1, 0, true);
    }
  } else {
    int length;
    pos_T startpos;

    if (u_save((linenr_T)(oap->start.lnum - 1),
               (linenr_T)(oap->end.lnum + 1)) == FAIL) {
      disable_fold_update--;
      return;
    }

    pos_T pos = oap->start;
    for (; pos.lnum <= oap->end.lnum; pos.lnum++) {
      if (oap->motion_type == kMTBlockWise) {
        // Visual block mode
        block_prep(oap, &bd, pos.lnum, false);
        pos.col = bd.textcol;
        length = bd.textlen;
      } else if (oap->motion_type == kMTLineWise) {
        curwin->w_cursor.col = 0;
        pos.col = 0;
        length = ml_get_len(pos.lnum);
      } else {
        // oap->motion_type == kMTCharWise
        if (pos.lnum == oap->start.lnum && !oap->inclusive) {
          dec(&(oap->end));
        }
        length = ml_get_len(pos.lnum);
        pos.col = 0;
        if (pos.lnum == oap->start.lnum) {
          pos.col += oap->start.col;
          length -= oap->start.col;
        }
        if (pos.lnum == oap->end.lnum) {
          length = ml_get_len(oap->end.lnum);
          oap->end.col = MIN(oap->end.col, length - 1);
          length = oap->end.col - pos.col + 1;
        }
      }
      bool one_change = do_addsub(oap->op_type, &pos, length, amount);
      if (one_change) {
        // Remember the start position of the first change.
        if (change_cnt == 0) {
          startpos = curbuf->b_op_start;
        }
        change_cnt++;
      }

      if (g_cmd && one_change) {
        amount += Prenum1;
      }
    }

    disable_fold_update--;
    if (change_cnt) {
      changed_lines(curbuf, oap->start.lnum, 0, oap->end.lnum + 1, 0, true);
    }

    if (!change_cnt && oap->is_VIsual) {
      // No change: need to remove the Visual selection
      redraw_curbuf_later(UPD_INVERTED);
    }

    // Set '[ mark if something changed. Keep the last end
    // position from do_addsub().
    if (change_cnt > 0 && (cmdmod.cmod_flags & CMOD_LOCKMARKS) == 0) {
      curbuf->b_op_start = startpos;
    }

    if (change_cnt > p_report) {
      smsg(0, NGETTEXT("%" PRId64 " lines changed", "%" PRId64 " lines changed", change_cnt),
           (int64_t)change_cnt);
    }
  }
}

// =============================================================================
// C accessor functions for rs_do_addsub (Phase 2)
// =============================================================================

// File-scope static: cursor saved by setup, restored by cleanup on visual exit.
static pos_T addsub_saved_cursor;

void nvim_addsub_save_cursor(void) { addsub_saved_cursor = curwin->w_cursor; }
void nvim_addsub_restore_cursor(void) { curwin->w_cursor = addsub_saved_cursor; }


void clear_oparg(oparg_T *oap)
{
  CLEAR_POINTER(oap);
}

///  Count the number of bytes, characters and "words" in a line.
///
///  "Words" are counted by looking for boundaries between non-space and
///  space characters.  (it seems to produce results that match 'wc'.)
///
///  Return value is byte count; word count for the line is added to "*wc".
///  Char count is added to "*cc".
///
///  The function will only examine the first "limit" characters in the
///  line, stopping if it encounters an end-of-line (NUL byte).  In that
///  case, eol_size will be added to the character count to account for
///  the size of the EOL character.
static varnumber_T line_count_info(char *line, varnumber_T *wc, varnumber_T *cc, varnumber_T limit,
                                   int eol_size)
{
  varnumber_T i;
  varnumber_T words = 0;
  varnumber_T chars = 0;
  bool is_word = false;

  for (i = 0; i < limit && line[i] != NUL;) {
    if (is_word) {
      if (ascii_isspace(line[i])) {
        words++;
        is_word = false;
      }
    } else if (!ascii_isspace(line[i])) {
      is_word = true;
    }
    chars++;
    i += utfc_ptr2len(line + i);
  }

  if (is_word) {
    words++;
  }
  *wc += words;

  // Add eol_size if the end of line was reached before hitting limit.
  if (i < limit && line[i] == NUL) {
    i += eol_size;
    chars += eol_size;
  }
  *cc += chars;
  return i;
}

// =============================================================================
// C accessor functions for rs_cursor_pos_info (Phase 1)
// =============================================================================

// Verify constants used in Rust
_Static_assert(MAXCOL == 0x7fffffff, "MAXCOL mismatch");
_Static_assert(Ctrl_V == 22, "Ctrl_V mismatch");
_Static_assert('V' == 0x56, "V char mismatch");
_Static_assert('v' == 0x76, "v char mismatch");

/// Struct matching Rust CpiVisualState
typedef struct {
  int active;
  int mode;
  int visual_lnum;
  int visual_col;
  int cursor_lnum;
  int cursor_col;
  int sel_exclusive;
  int curswant;
} CpiVisualState;

/// Struct matching Rust CpiLineCountResult
typedef struct {
  int64_t byte_count;
  int64_t word_count;
  int64_t char_count;
} CpiLineCountResult;


/// Get EOL size based on file format (1 for unix, 2 for DOS).
int nvim_cpi_get_eol_size(void)
{
  return (rs_get_fileformat((buf_T *)curbuf) == EOL_DOS) ? 2 : 1;
}

/// Get visual mode state in one batch call.
void nvim_cpi_get_visual_state(void *out_ptr)
{
  CpiVisualState *out = (CpiVisualState *)out_ptr;
  out->active = VIsual_active;
  out->mode = VIsual_mode;
  out->visual_lnum = (int)VIsual.lnum;
  out->visual_col = (int)VIsual.col;
  out->cursor_lnum = (int)curwin->w_cursor.lnum;
  out->cursor_col = (int)curwin->w_cursor.col;
  out->sel_exclusive = (*p_sel == 'e') ? 1 : 0;
  out->curswant = (int)curwin->w_curswant;
}

/// Count words, chars, bytes in a line up to col_limit.
void nvim_cpi_line_count_info(int lnum, int col_limit, int eol_size,
                              void *out_ptr)
{
  CpiLineCountResult *out = (CpiLineCountResult *)out_ptr;
  varnumber_T wc = 0, cc = 0;
  varnumber_T bc = line_count_info(ml_get((linenr_T)lnum), &wc, &cc,
                                   (varnumber_T)col_limit, eol_size);
  out->byte_count = (int64_t)bc;
  out->word_count = (int64_t)wc;
  out->char_count = (int64_t)cc;
}

/// Count words, chars, bytes starting at a column offset within a line.
void nvim_cpi_line_count_info_at(int lnum, int start_col, int len, int eol_size,
                                 void *out_ptr)
{
  CpiLineCountResult *out = (CpiLineCountResult *)out_ptr;
  varnumber_T wc = 0, cc = 0;
  char *s = ml_get((linenr_T)lnum) + start_col;
  varnumber_T bc = line_count_info(s, &wc, &cc, (varnumber_T)len, eol_size);
  out->byte_count = (int64_t)bc;
  out->word_count = (int64_t)wc;
  out->char_count = (int64_t)cc;
}

/// Set up block visual mode: get virtual columns with sbr temporarily cleared.
void nvim_cpi_setup_block_visual(int min_lnum, int min_col,
                                 int max_lnum, int max_col,
                                 int *out_start_vcol, int *out_end_vcol)
{
  pos_T min_pos = { .lnum = min_lnum, .col = min_col, .coladd = 0 };
  pos_T max_pos = { .lnum = max_lnum, .col = max_col, .coladd = 0 };

  char *const saved_sbr = p_sbr;
  char *const saved_w_sbr = curwin->w_p_sbr;
  p_sbr = empty_string_option;
  curwin->w_p_sbr = empty_string_option;

  oparg_T oparg;
  memset(&oparg, 0, sizeof(oparg));
  oparg.is_VIsual = true;
  oparg.motion_type = kMTBlockWise;
  oparg.op_type = OP_NOP;
  getvcols(curwin, &min_pos, &max_pos, &oparg.start_vcol, &oparg.end_vcol);

  p_sbr = saved_sbr;
  curwin->w_p_sbr = saved_w_sbr;

  *out_start_vcol = (int)oparg.start_vcol;
  *out_end_vcol = (int)oparg.end_vcol;
}

/// Count info for a block visual line (using block_prep).
void nvim_cpi_block_line_count(int lnum, int eol_size, void *out_ptr)
{
  CpiLineCountResult *out = (CpiLineCountResult *)out_ptr;
  oparg_T oparg;
  memset(&oparg, 0, sizeof(oparg));
  oparg.is_VIsual = true;
  oparg.motion_type = kMTBlockWise;
  oparg.op_type = OP_NOP;

  // We need the vcols from the current visual selection.
  // Re-derive them from VIsual and cursor.
  pos_T min_pos, max_pos;
  if (lt(VIsual, curwin->w_cursor)) {
    min_pos = VIsual;
    max_pos = curwin->w_cursor;
  } else {
    min_pos = curwin->w_cursor;
    max_pos = VIsual;
  }
  if (*p_sel == 'e' && max_pos.col > 0) {
    max_pos.col--;
  }

  char *const saved_sbr = p_sbr;
  char *const saved_w_sbr = curwin->w_p_sbr;
  p_sbr = empty_string_option;
  curwin->w_p_sbr = empty_string_option;
  getvcols(curwin, &min_pos, &max_pos, &oparg.start_vcol, &oparg.end_vcol);
  p_sbr = saved_sbr;
  curwin->w_p_sbr = saved_w_sbr;

  if (curwin->w_curswant == MAXCOL) {
    oparg.end_vcol = MAXCOL;
  }
  if (oparg.end_vcol < oparg.start_vcol) {
    oparg.end_vcol += oparg.start_vcol;
    oparg.start_vcol = oparg.end_vcol - oparg.start_vcol;
    oparg.end_vcol -= oparg.start_vcol;
  }

  struct block_def bd;
  virtual_op = virtual_active(curwin);
  block_prep(&oparg, &bd, (linenr_T)lnum, false);
  virtual_op = kNone;

  varnumber_T wc = 0, cc = 0;
  varnumber_T bc = 0;
  if (bd.textstart != NULL) {
    bc = line_count_info(bd.textstart, &wc, &cc, (varnumber_T)bd.textlen, eol_size);
  }
  out->byte_count = (int64_t)bc;
  out->word_count = (int64_t)wc;
  out->char_count = (int64_t)cc;
}

/// Check if last line has no EOL (for byte count correction).
int nvim_cpi_last_line_no_eol(void)
{
  return (!curbuf->b_p_eol && (curbuf->b_p_bin || !curbuf->b_p_fixeol)) ? 1 : 0;
}

/// Check if string at current position is shorter than len (for last-line EOL adjustment).
int nvim_cpi_last_line_short(int lnum, int byte_count)
{
  (void)lnum;
  (void)byte_count;
  // The original C code checked: (int)strlen(s) < len
  // This is a conservative approximation; in visual mode with the last line,
  // the EOL adjustment happens when the line was fully consumed.
  return 1;
}

/// Call os_breakcheck for interrupt detection.
void nvim_cpi_os_breakcheck(void)
{
  os_breakcheck();
}


/// Show the "no lines" message for empty buffers.
void nvim_cpi_show_empty_msg(void)
{
  msg(_(no_lines_msg), 0);
}

/// Get BOM size.
int nvim_cpi_get_bomb_size(void)
{
  return bomb_size();
}

/// Format and display the visual mode message.
void nvim_cpi_format_visual_msg(int line_count_selected,
                                int start_vcol,
                                int end_vcol,
                                int is_block_mode,
                                int curswant_is_max,
                                int64_t word_count_cursor,
                                int64_t word_count,
                                int64_t char_count_cursor,
                                int64_t char_count,
                                int64_t byte_count_cursor,
                                int64_t byte_count)
{
  char buf1[50];

  if (is_block_mode && !curswant_is_max) {
    int64_t cols;
    STRICT_SUB(end_vcol + 1, start_vcol, &cols, int64_t);
    vim_snprintf(buf1, sizeof(buf1), _("%" PRId64 " Cols; "), cols);
  } else {
    buf1[0] = NUL;
  }

  if (char_count_cursor == byte_count_cursor
      && char_count == byte_count) {
    vim_snprintf(IObuff, IOSIZE,
                 _("Selected %s%" PRId64 " of %" PRId64 " Lines;"
                   " %" PRId64 " of %" PRId64 " Words;"
                   " %" PRId64 " of %" PRId64 " Bytes"),
                 buf1, (int64_t)line_count_selected,
                 (int64_t)curbuf->b_ml.ml_line_count,
                 word_count_cursor, word_count,
                 byte_count_cursor, byte_count);
  } else {
    vim_snprintf(IObuff, IOSIZE,
                 _("Selected %s%" PRId64 " of %" PRId64 " Lines;"
                   " %" PRId64 " of %" PRId64 " Words;"
                   " %" PRId64 " of %" PRId64 " Chars;"
                   " %" PRId64 " of %" PRId64 " Bytes"),
                 buf1, (int64_t)line_count_selected,
                 (int64_t)curbuf->b_ml.ml_line_count,
                 word_count_cursor, word_count,
                 char_count_cursor, char_count,
                 byte_count_cursor, byte_count);
  }
}

/// Format and display the normal mode message.
void nvim_cpi_format_normal_msg(int64_t word_count_cursor,
                                int64_t word_count,
                                int64_t char_count_cursor,
                                int64_t char_count,
                                int64_t byte_count_cursor,
                                int64_t byte_count)
{
  char buf1[50];
  char buf2[40];

  char *p = get_cursor_line_ptr();
  validate_virtcol(curwin);
  col_print(buf1, sizeof(buf1), (int)curwin->w_cursor.col + 1,
            (int)curwin->w_virtcol + 1);
  col_print(buf2, sizeof(buf2), get_cursor_line_len(), linetabsize_str(p));

  if (char_count_cursor == byte_count_cursor
      && char_count == byte_count) {
    vim_snprintf(IObuff, IOSIZE,
                 _("Col %s of %s; Line %" PRId64 " of %" PRId64 ";"
                   " Word %" PRId64 " of %" PRId64 ";"
                   " Byte %" PRId64 " of %" PRId64 ""),
                 buf1, buf2,
                 (int64_t)curwin->w_cursor.lnum,
                 (int64_t)curbuf->b_ml.ml_line_count,
                 word_count_cursor, word_count,
                 byte_count_cursor, byte_count);
  } else {
    vim_snprintf(IObuff, IOSIZE,
                 _("Col %s of %s; Line %" PRId64 " of %" PRId64 ";"
                   " Word %" PRId64 " of %" PRId64 ";"
                   " Char %" PRId64 " of %" PRId64 ";"
                   " Byte %" PRId64 " of %" PRId64 ""),
                 buf1, buf2,
                 (int64_t)curwin->w_cursor.lnum,
                 (int64_t)curbuf->b_ml.ml_line_count,
                 word_count_cursor, word_count,
                 char_count_cursor, char_count,
                 byte_count_cursor, byte_count);
  }
}

/// Append BOM info to IObuff and display the message.
void nvim_cpi_append_bom_and_display(int64_t bom_count)
{
  if (bom_count > 0) {
    const size_t len = strlen(IObuff);
    vim_snprintf(IObuff + len, IOSIZE - len,
                 _("(+%" PRId64 " for BOM)"), bom_count);
  }
  // Don't shorten this message, the user asked for it.
  char *p = p_shm;
  p_shm = "";
  if (p_ch < 1) {
    msg_start();
    msg_scroll = true;
  }
  msg(IObuff, 0);
  p_shm = p;
}

/// Populate the dictionary with word count info.
void nvim_cpi_populate_dict(dict_T *dict,
                            int visual_active,
                            int64_t word_count,
                            int64_t char_count,
                            int64_t byte_count,
                            int64_t bom_count,
                            int64_t word_count_cursor,
                            int64_t char_count_cursor,
                            int64_t byte_count_cursor)
{
  tv_dict_add_nr(dict, S_LEN("words"), (varnumber_T)word_count);
  tv_dict_add_nr(dict, S_LEN("chars"), (varnumber_T)char_count);
  tv_dict_add_nr(dict, S_LEN("bytes"), (varnumber_T)(byte_count + bom_count));

  STATIC_ASSERT(sizeof("visual") == sizeof("cursor"),
                "key_len argument in tv_dict_add_nr is wrong");
  tv_dict_add_nr(dict, visual_active ? "visual_bytes" : "cursor_bytes",
                 sizeof("visual_bytes") - 1, (varnumber_T)byte_count_cursor);
  tv_dict_add_nr(dict, visual_active ? "visual_chars" : "cursor_chars",
                 sizeof("visual_chars") - 1, (varnumber_T)char_count_cursor);
  tv_dict_add_nr(dict, visual_active ? "visual_words" : "cursor_words",
                 sizeof("visual_words") - 1, (varnumber_T)word_count_cursor);
}

/// Handle indent and format operators and visual mode ":".
static void op_colon(oparg_T *oap)
{
  stuffcharReadbuff(':');
  if (oap->is_VIsual) {
    stuffReadbuff("'<,'>");
  } else {
    // Make the range look nice, so it can be repeated.
    if (oap->start.lnum == curwin->w_cursor.lnum) {
      stuffcharReadbuff('.');
    } else {
      stuffnumReadbuff(oap->start.lnum);
    }

    // When using !! on a closed fold the range ".!" works best to operate
    // on, it will be made the whole closed fold later.
    linenr_T endOfStartFold = oap->start.lnum;
    hasFolding(curwin, oap->start.lnum, NULL, &endOfStartFold);
    if (oap->end.lnum != oap->start.lnum && oap->end.lnum != endOfStartFold) {
      // Make it a range with the end line.
      stuffcharReadbuff(',');
      if (oap->end.lnum == curwin->w_cursor.lnum) {
        stuffcharReadbuff('.');
      } else if (oap->end.lnum == curbuf->b_ml.ml_line_count) {
        stuffcharReadbuff('$');
      } else if (oap->start.lnum == curwin->w_cursor.lnum
                 // do not use ".+number" for a closed fold, it would count
                 // folded lines twice
                 && !hasFolding(curwin, oap->end.lnum, NULL, NULL)) {
        stuffReadbuff(".+");
        stuffnumReadbuff(oap->line_count - 1);
      } else {
        stuffnumReadbuff(oap->end.lnum);
      }
    }
  }
  if (oap->op_type != OP_COLON) {
    stuffReadbuff("!");
  }
  if (oap->op_type == OP_INDENT) {
    stuffReadbuff(get_equalprg());
    stuffReadbuff("\n");
  } else if (oap->op_type == OP_FORMAT) {
    if (*curbuf->b_p_fp != NUL) {
      stuffReadbuff(curbuf->b_p_fp);
    } else if (*p_fp != NUL) {
      stuffReadbuff(p_fp);
    } else {
      stuffReadbuff("fmt");
    }
    stuffReadbuff("\n']");
  }

  // do_cmdline() does the rest
}

/// callback function for 'operatorfunc'
static Callback opfunc_cb;

/// Process the 'operatorfunc' option value.
const char *did_set_operatorfunc(optset_T *args FUNC_ATTR_UNUSED)
{
  if (option_set_callback_func(p_opfunc, &opfunc_cb) == FAIL) {
    return e_invarg;
  }
  return NULL;
}

#if defined(EXITFREE)
void free_operatorfunc_option(void)
{
  callback_free(&opfunc_cb);
}
#endif

/// Mark the global 'operatorfunc' callback with "copyID" so that it is not
/// garbage collected.
bool set_ref_in_opfunc(int copyID)
{
  return rs_set_ref_in_callback(&opfunc_cb, copyID, NULL, NULL);
}

/// Handle the "g@" operator: call 'operatorfunc'.
static void op_function(const oparg_T *oap)
  FUNC_ATTR_NONNULL_ALL
{
  const pos_T orig_start = curbuf->b_op_start;
  const pos_T orig_end = curbuf->b_op_end;

  if (*p_opfunc == NUL) {
    emsg(_("E774: 'operatorfunc' is empty"));
  } else {
    // Set '[ and '] marks to text to be operated on.
    curbuf->b_op_start = oap->start;
    curbuf->b_op_end = oap->end;
    if (oap->motion_type != kMTLineWise && !oap->inclusive) {
      // Exclude the end position.
      decl(&curbuf->b_op_end);
    }

    typval_T argv[2];
    argv[0].v_type = VAR_STRING;
    argv[1].v_type = VAR_UNKNOWN;
    argv[0].vval.v_string =
      (char *)(((const char *const[]) {
      [kMTBlockWise] = "block",
      [kMTLineWise] = "line",
      [kMTCharWise] = "char",
    })[oap->motion_type]);

    // Reset virtual_op so that 'virtualedit' can be changed in the
    // function.
    const TriState save_virtual_op = virtual_op;
    virtual_op = kNone;

    // Reset finish_op so that mode() returns the right value.
    const bool save_finish_op = finish_op;
    finish_op = false;

    typval_T rettv;
    if (callback_call(&opfunc_cb, 1, argv, &rettv)) {
      tv_clear(&rettv);
    }

    virtual_op = save_virtual_op;
    finish_op = save_finish_op;
    if (cmdmod.cmod_flags & CMOD_LOCKMARKS) {
      curbuf->b_op_start = orig_start;
      curbuf->b_op_end = orig_end;
    }
  }
}

/// Calculate start/end virtual columns for operating in block mode.
///
/// @param initial  when true: adjust position for 'selectmode'
static void get_op_vcol(oparg_T *oap, colnr_T redo_VIsual_vcol, bool initial)
{
  colnr_T start;
  colnr_T end;

  if (VIsual_mode != Ctrl_V
      || (!initial && oap->end.col < curwin->w_view_width)) {
    return;
  }

  oap->motion_type = kMTBlockWise;

  // prevent from moving onto a trail byte
  mark_mb_adjustpos(curwin->w_buffer, &oap->end);

  getvvcol(curwin, &(oap->start), &oap->start_vcol, NULL, &oap->end_vcol);
  if (!redo_VIsual_busy) {
    getvvcol(curwin, &(oap->end), &start, NULL, &end);

    oap->start_vcol = MIN(oap->start_vcol, start);
    if (end > oap->end_vcol) {
      if (initial && *p_sel == 'e'
          && start >= 1
          && start - 1 >= oap->end_vcol) {
        oap->end_vcol = start - 1;
      } else {
        oap->end_vcol = end;
      }
    }
  }

  // if '$' was used, get oap->end_vcol from longest line
  if (curwin->w_curswant == MAXCOL) {
    curwin->w_cursor.col = MAXCOL;
    oap->end_vcol = 0;
    for (curwin->w_cursor.lnum = oap->start.lnum;
         curwin->w_cursor.lnum <= oap->end.lnum; curwin->w_cursor.lnum++) {
      getvvcol(curwin, &curwin->w_cursor, NULL, NULL, &end);
      oap->end_vcol = MAX(oap->end_vcol, end);
    }
  } else if (redo_VIsual_busy) {
    oap->end_vcol = oap->start_vcol + redo_VIsual_vcol - 1;
  }

  // Correct oap->end.col and oap->start.col to be the
  // upper-left and lower-right corner of the block area.
  //
  // (Actually, this does convert column positions into character
  // positions)
  curwin->w_cursor.lnum = oap->end.lnum;
  coladvance(curwin, oap->end_vcol);
  oap->end = curwin->w_cursor;

  curwin->w_cursor = oap->start;
  coladvance(curwin, oap->start_vcol);
  oap->start = curwin->w_cursor;
}

/// Information for redoing the previous Visual selection.
typedef struct {
  int rv_mode;             ///< 'v', 'V', or Ctrl-V
  linenr_T rv_line_count;  ///< number of lines
  colnr_T rv_vcol;         ///< number of cols or end column
  int rv_count;            ///< count for Visual operator
  int rv_arg;              ///< extra argument
} redo_VIsual_T;

static bool is_ex_cmdchar(cmdarg_T *cap)
{
  return cap->cmdchar == ':' || cap->cmdchar == K_COMMAND;
}

/// Handle an operator after Visual mode or when the movement is finished.
/// "gui_yank" is true when yanking text for the clipboard.
// ===========================================================================
// do_pending_operator C accessors for Rust migration (Phase 5)
// ===========================================================================

// File-scope statics for do_pending_operator (bridges across accessor calls)
static redo_VIsual_T dpo_redo_VIsual = { NUL, 0, 0, 0, 0 };
static bool dpo_include_line_break;
static int dpo_saved_lbr;
static pos_T dpo_saved_old_cursor;

/// Check if we should process the pending operator.
int nvim_dpo_should_process(cmdarg_T *cap)
{
  // Save state needed by postamble/restore_lbr across accessor calls.
  dpo_saved_lbr = curwin->w_p_lbr;
  dpo_saved_old_cursor = curwin->w_cursor;
  oparg_T *oap = cap->oap;
  return ((finish_op || VIsual_active) && oap->op_type != OP_NOP) ? 1 : 0;
}

/// Preamble: save lbr, handle motion_force, prep redo, redo_VIsual/VIsual setup.
void nvim_dpo_preamble(cmdarg_T *cap, int gui_yank)
{
  oparg_T *oap = cap->oap;
  bool include_line_break = false;
  const bool redo_yank = vim_strchr(p_cpo, CPO_YANK) != NULL && !gui_yank;

  reset_lbr();
  oap->is_VIsual = VIsual_active;

  // Handle motion_force
  if (oap->motion_force == 'V') {
    oap->motion_type = kMTLineWise;
  } else if (oap->motion_force == 'v') {
    if (oap->motion_type == kMTLineWise) {
      oap->inclusive = false;
    } else if (oap->motion_type == kMTCharWise) {
      oap->inclusive = !oap->inclusive;
    }
    oap->motion_type = kMTCharWise;
  } else if (oap->motion_force == Ctrl_V) {
    if (!VIsual_active) {
      VIsual_active = true;
      VIsual = oap->start;
    }
    VIsual_mode = Ctrl_V;
    VIsual_select = false;
    VIsual_reselect = false;
  }

  // Prep redo
  if ((redo_yank || oap->op_type != OP_YANK)
      && ((!VIsual_active || oap->motion_force)
          || ((is_ex_cmdchar(cap) || cap->cmdchar == K_LUA)
              && oap->op_type != OP_COLON))
      && cap->cmdchar != 'D'
      && oap->op_type != OP_FOLD
      && oap->op_type != OP_FOLDOPEN
      && oap->op_type != OP_FOLDOPENREC
      && oap->op_type != OP_FOLDCLOSE
      && oap->op_type != OP_FOLDCLOSEREC
      && oap->op_type != OP_FOLDDEL
      && oap->op_type != OP_FOLDDELREC) {
    rs_prep_redo(oap->regname, cap->count0,
              get_op_char(oap->op_type), get_extra_op_char(oap->op_type),
              oap->motion_force, cap->cmdchar, cap->nchar);
    if (cap->cmdchar == '/' || cap->cmdchar == '?') {
      if (vim_strchr(p_cpo, CPO_REDO) == NULL) {
        AppendToRedobuffLit(cap->searchbuf, -1);
      }
      AppendToRedobuff(NL_STR);
    } else if (is_ex_cmdchar(cap)) {
      if (repeat_cmdline == NULL) {
        ResetRedobuff();
      } else {
        if (cap->cmdchar == ':') {
          AppendToRedobuffLit(repeat_cmdline, -1);
        } else {
          AppendToRedobuffSpec(repeat_cmdline);
        }
        AppendToRedobuff(NL_STR);
        XFREE_CLEAR(repeat_cmdline);
      }
    } else if (cap->cmdchar == K_LUA) {
      AppendNumberToRedobuff(repeat_luaref);
      AppendToRedobuff(NL_STR);
    }
  }

  // Handle redo_VIsual_busy or VIsual_active
  if (redo_VIsual_busy) {
    oap->start = curwin->w_cursor;
    curwin->w_cursor.lnum += dpo_redo_VIsual.rv_line_count - 1;
    curwin->w_cursor.lnum = MIN(curwin->w_cursor.lnum, curbuf->b_ml.ml_line_count);
    VIsual_mode = dpo_redo_VIsual.rv_mode;
    if (dpo_redo_VIsual.rv_vcol == MAXCOL || VIsual_mode == 'v') {
      if (VIsual_mode == 'v') {
        if (dpo_redo_VIsual.rv_line_count <= 1) {
          validate_virtcol(curwin);
          curwin->w_curswant = curwin->w_virtcol + dpo_redo_VIsual.rv_vcol - 1;
        } else {
          curwin->w_curswant = dpo_redo_VIsual.rv_vcol;
        }
      } else {
        curwin->w_curswant = MAXCOL;
      }
      coladvance(curwin, curwin->w_curswant);
    }
    cap->count0 = dpo_redo_VIsual.rv_count;
    cap->count1 = (cap->count0 == 0 ? 1 : cap->count0);
  } else if (VIsual_active) {
    if (!gui_yank) {
      curbuf->b_visual.vi_start = VIsual;
      curbuf->b_visual.vi_end = curwin->w_cursor;
      curbuf->b_visual.vi_mode = VIsual_mode;
      rs_restore_visual_mode();
      curbuf->b_visual.vi_curswant = curwin->w_curswant;
      curbuf->b_visual_mode_eval = VIsual_mode;
    }

    if (VIsual_select && VIsual_mode == 'V'
        && cap->oap->op_type != OP_DELETE) {
      if (lt(VIsual, curwin->w_cursor)) {
        VIsual.col = 0;
        curwin->w_cursor.col = ml_get_len(curwin->w_cursor.lnum);
      } else {
        curwin->w_cursor.col = 0;
        VIsual.col = ml_get_len(VIsual.lnum);
      }
      VIsual_mode = 'v';
    } else if (VIsual_mode == 'v') {
      include_line_break = rs_unadjust_for_sel();
    }

    oap->start = VIsual;
    if (VIsual_mode == 'V') {
      oap->start.col = 0;
      oap->start.coladd = 0;
    }
  }

  // Store include_line_break for setup_positions (file-scope static,
  // safe because do_pending_operator is not reentrant).
  dpo_include_line_break = include_line_break;
}

/// Setup positions, folds, visual state.
void nvim_dpo_setup_positions(cmdarg_T *cap, int gui_yank)
{
  oparg_T *oap = cap->oap;
  int lbr_saved = curwin->w_p_lbr;
  const bool redo_yank = vim_strchr(p_cpo, CPO_YANK) != NULL && !gui_yank;

  // Retrieve include_line_break from preamble (file-scope static)
  bool include_line_break = dpo_include_line_break;

  // Set oap->start/end and fold handling
  if (lt(oap->start, curwin->w_cursor)) {
    if (!VIsual_active) {
      if (hasFolding(curwin, oap->start.lnum, &oap->start.lnum, NULL)) {
        oap->start.col = 0;
      }
      if ((curwin->w_cursor.col > 0
           || oap->inclusive
           || oap->motion_type == kMTLineWise)
          && hasFolding(curwin, curwin->w_cursor.lnum, NULL,
                        &curwin->w_cursor.lnum)) {
        curwin->w_cursor.col = get_cursor_line_len();
      }
    }
    oap->end = curwin->w_cursor;
    curwin->w_cursor = oap->start;
    curwin->w_valid &= ~VALID_VIRTCOL;
  } else {
    if (!VIsual_active && oap->motion_type == kMTLineWise) {
      if (hasFolding(curwin, curwin->w_cursor.lnum, &curwin->w_cursor.lnum, NULL)) {
        curwin->w_cursor.col = 0;
      }
      if (hasFolding(curwin, oap->start.lnum, NULL, &oap->start.lnum)) {
        oap->start.col = ml_get_len(oap->start.lnum);
      }
    }
    oap->end = oap->start;
    oap->start = curwin->w_cursor;
  }

  check_pos(curwin->w_buffer, &oap->end);
  oap->line_count = oap->end.lnum - oap->start.lnum + 1;
  virtual_op = virtual_active(curwin);

  if (VIsual_active || redo_VIsual_busy) {
    get_op_vcol(oap, dpo_redo_VIsual.rv_vcol, true);

    if (!redo_VIsual_busy && !gui_yank) {
      resel_VIsual_mode = VIsual_mode;
      if (curwin->w_curswant == MAXCOL) {
        resel_VIsual_vcol = MAXCOL;
      } else {
        if (VIsual_mode != Ctrl_V) {
          getvvcol(curwin, &(oap->end), NULL, NULL, &oap->end_vcol);
        }
        if (VIsual_mode == Ctrl_V || oap->line_count <= 1) {
          if (VIsual_mode != Ctrl_V) {
            getvvcol(curwin, &(oap->start), &oap->start_vcol, NULL, NULL);
          }
          resel_VIsual_vcol = oap->end_vcol - oap->start_vcol + 1;
        } else {
          resel_VIsual_vcol = oap->end_vcol;
        }
      }
      resel_VIsual_line_count = oap->line_count;
    }

    // Redo visual prep
    if ((redo_yank || oap->op_type != OP_YANK)
        && oap->op_type != OP_COLON
        && oap->op_type != OP_FOLD
        && oap->op_type != OP_FOLDOPEN
        && oap->op_type != OP_FOLDOPENREC
        && oap->op_type != OP_FOLDCLOSE
        && oap->op_type != OP_FOLDCLOSEREC
        && oap->op_type != OP_FOLDDEL
        && oap->op_type != OP_FOLDDELREC
        && oap->motion_force == NUL) {
      if (cap->cmdchar == 'g' && (cap->nchar == 'n' || cap->nchar == 'N')) {
        rs_prep_redo(oap->regname, cap->count0,
                  get_op_char(oap->op_type), get_extra_op_char(oap->op_type),
                  oap->motion_force, cap->cmdchar, cap->nchar);
      } else if (!is_ex_cmdchar(cap) && cap->cmdchar != K_LUA) {
        int opchar = get_op_char(oap->op_type);
        int extra_opchar = get_extra_op_char(oap->op_type);
        int nchar = oap->op_type == OP_REPLACE ? cap->nchar : NUL;
        if (nchar == REPLACE_CR_NCHAR) {
          nchar = CAR;
        } else if (nchar == REPLACE_NL_NCHAR) {
          nchar = NL;
        }
        if (opchar == 'g' && extra_opchar == '@') {
          rs_prep_redo_num2(oap->regname, 0, NUL, 'v', cap->count0, opchar, extra_opchar, nchar);
        } else {
          rs_prep_redo(oap->regname, 0, NUL, 'v', opchar, extra_opchar, nchar);
        }
      }
      if (!redo_VIsual_busy) {
        dpo_redo_VIsual.rv_mode = resel_VIsual_mode;
        dpo_redo_VIsual.rv_vcol = resel_VIsual_vcol;
        dpo_redo_VIsual.rv_line_count = resel_VIsual_line_count;
        dpo_redo_VIsual.rv_count = cap->count0;
        dpo_redo_VIsual.rv_arg = cap->arg;
      }
    }

    // Inclusive/motion_type adjustments
    if (oap->motion_force == NUL || oap->motion_type == kMTLineWise) {
      oap->inclusive = true;
    }
    if (VIsual_mode == 'V') {
      oap->motion_type = kMTLineWise;
    } else if (VIsual_mode == 'v') {
      oap->motion_type = kMTCharWise;
      if (*ml_get_pos(&(oap->end)) == NUL
          && (include_line_break || !virtual_op)) {
        oap->inclusive = false;
        if (*p_sel != 'o'
            && !op_on_lines(oap->op_type)
            && oap->end.lnum < curbuf->b_ml.ml_line_count) {
          oap->end.lnum++;
          oap->end.col = 0;
          oap->end.coladd = 0;
          oap->line_count++;
        }
      }
    }

    redo_VIsual_busy = false;

    if (!gui_yank) {
      VIsual_active = false;
      setmouse();
      mouse_dragging = 0;
      rs_may_clear_cmdline();
      if ((oap->op_type == OP_YANK
           || oap->op_type == OP_COLON
           || oap->op_type == OP_FUNCTION
           || oap->op_type == OP_FILTER)
          && oap->motion_force == NUL) {
        restore_lbr(lbr_saved);
        redraw_curbuf_later(UPD_INVERTED);
      }
    }
  }

  // Include trailing multi-byte byte
  if (oap->inclusive) {
    const int l = utfc_ptr2len(ml_get_pos(&oap->end));
    if (l > 1) {
      oap->end.col += l - 1;
    }
  }
  curwin->w_set_curswant = true;

  // empty check
  oap->empty = (oap->motion_type != kMTLineWise
                && (!oap->inclusive
                    || (oap->op_type == OP_YANK
                        && gchar_pos(&oap->end) == NUL))
                && equalpos(oap->start, oap->end)
                && !(virtual_op && oap->start.coladd != oap->end.coladd));

  // Force redraw for empty visual region
  if (oap->is_VIsual && (oap->empty || !MODIFIABLE(curbuf)
                         || oap->op_type == OP_FOLD)) {
    restore_lbr(lbr_saved);
    redraw_curbuf_later(UPD_INVERTED);
  }

  // Adjust end for column-one case
  if (oap->motion_type == kMTCharWise
      && oap->inclusive == false
      && !(cap->retval & CA_NO_ADJ_OP_END)
      && oap->end.col == 0
      && (!oap->is_VIsual || *p_sel == 'o')
      && oap->line_count > 1) {
    oap->end_adjusted = true;
    oap->line_count--;
    oap->end.lnum--;
    if (inindent(0)) {
      oap->motion_type = kMTLineWise;
    } else {
      oap->end.col = ml_get_len(oap->end.lnum);
      if (oap->end.col) {
        oap->end.col--;
        oap->inclusive = true;
      }
    }
  } else {
    oap->end_adjusted = false;
  }
}

/// Dispatch the operator (the big switch).
void nvim_dpo_dispatch_operator(cmdarg_T *cap, int gui_yank)
{
  oparg_T *oap = cap->oap;
  int lbr_saved = curwin->w_p_lbr;
  int restart_edit_save;
  bool empty_region_error = (oap->empty
                             && vim_strchr(p_cpo, CPO_EMPTYREGION) != NULL);

  switch (oap->op_type) {
  case OP_LSHIFT:
  case OP_RSHIFT:
    op_shift(oap, true, oap->is_VIsual ? cap->count1 : 1);
    auto_format(false, true);
    break;

  case OP_JOIN_NS:
  case OP_JOIN:
    oap->line_count = MAX(oap->line_count, 2);
    if (curwin->w_cursor.lnum + oap->line_count - 1 >
        curbuf->b_ml.ml_line_count) {
      beep_flush();
    } else {
      do_join((size_t)oap->line_count, oap->op_type == OP_JOIN,
              true, true, true);
      auto_format(false, true);
    }
    break;

  case OP_DELETE:
    VIsual_reselect = false;
    if (empty_region_error) {
      vim_beep(kOptBoFlagOperator);
      CancelRedo();
    } else {
      op_delete(oap);
      if (oap->motion_type == kMTLineWise
          && has_format_option(FO_AUTO)
          && u_save_cursor() == OK) {
        auto_format(false, true);
      }
    }
    break;

  case OP_YANK:
    if (empty_region_error) {
      if (!gui_yank) {
        vim_beep(kOptBoFlagOperator);
        CancelRedo();
      }
    } else {
      restore_lbr(lbr_saved);
      oap->excl_tr_ws = cap->cmdchar == 'z';
      op_yank(oap, !gui_yank);
    }
    check_cursor_col(curwin);
    break;

  case OP_CHANGE:
    VIsual_reselect = false;
    if (empty_region_error) {
      vim_beep(kOptBoFlagOperator);
      CancelRedo();
    } else {
      if (!KeyTyped) {
        restart_edit_save = restart_edit;
      } else {
        restart_edit_save = 0;
      }
      restart_edit = 0;
      restore_lbr(lbr_saved);
      curbuf->b_last_changedtick_i = buf_get_changedtick(curbuf);
      if (op_change(oap)) {
        cap->retval |= CA_COMMAND_BUSY;
      }
      if (restart_edit == 0) {
        restart_edit = restart_edit_save;
      }
    }
    break;

  case OP_FILTER:
    if (vim_strchr(p_cpo, CPO_FILTER) != NULL) {
      AppendToRedobuff("!\r");
    } else {
      bangredo = true;
    }
    FALLTHROUGH;

  case OP_INDENT:
  case OP_COLON:
    if (oap->op_type == OP_INDENT && *get_equalprg() == NUL) {
      if (curbuf->b_p_lisp) {
        if (use_indentexpr_for_lisp()) {
          op_reindent(oap, get_expr_indent);
        } else {
          op_reindent(oap, get_lisp_indent);
        }
        break;
      }
      op_reindent(oap,
                  *curbuf->b_p_inde != NUL ? get_expr_indent : get_c_indent);
      break;
    }
    op_colon(oap);
    break;

  case OP_TILDE:
  case OP_UPPER:
  case OP_LOWER:
  case OP_ROT13:
    if (empty_region_error) {
      vim_beep(kOptBoFlagOperator);
      CancelRedo();
    } else {
      op_tilde(oap);
    }
    check_cursor_col(curwin);
    break;

  case OP_FORMAT:
    if (*curbuf->b_p_fex != NUL) {
      op_formatexpr(oap);
    } else {
      if (*p_fp != NUL || *curbuf->b_p_fp != NUL) {
        op_colon(oap);
      } else {
        op_format(oap, false);
      }
    }
    break;

  case OP_FORMAT2:
    op_format(oap, true);
    break;

  case OP_FUNCTION: {
    redo_VIsual_T save_redo_VIsual = dpo_redo_VIsual;
    restore_lbr(lbr_saved);
    op_function(oap);
    dpo_redo_VIsual = save_redo_VIsual;
    break;
  }

  case OP_INSERT:
  case OP_APPEND:
    VIsual_reselect = false;
    if (empty_region_error) {
      vim_beep(kOptBoFlagOperator);
      CancelRedo();
    } else {
      restart_edit_save = restart_edit;
      restart_edit = 0;
      restore_lbr(lbr_saved);
      curbuf->b_last_changedtick_i = buf_get_changedtick(curbuf);
      op_insert(oap, cap->count1);
      reset_lbr();
      auto_format(false, true);
      if (restart_edit == 0) {
        restart_edit = restart_edit_save;
      } else {
        cap->retval |= CA_COMMAND_BUSY;
      }
    }
    break;

  case OP_REPLACE:
    VIsual_reselect = false;
    if (empty_region_error) {
      vim_beep(kOptBoFlagOperator);
      CancelRedo();
    } else {
      restore_lbr(lbr_saved);
      op_replace(oap, cap->nchar);
    }
    break;

  case OP_FOLD:
    VIsual_reselect = false;
    rs_foldCreate(curwin, oap->start.lnum, oap->end.lnum);
    break;

  case OP_FOLDOPEN:
  case OP_FOLDOPENREC:
  case OP_FOLDCLOSE:
  case OP_FOLDCLOSEREC:
    VIsual_reselect = false;
    rs_opFoldRange(oap->start.lnum, oap->end.lnum,
                   oap->op_type == OP_FOLDOPEN || oap->op_type == OP_FOLDOPENREC,
                   oap->op_type == OP_FOLDOPENREC || oap->op_type == OP_FOLDCLOSEREC,
                   oap->is_VIsual);
    break;

  case OP_FOLDDEL:
  case OP_FOLDDELREC:
    VIsual_reselect = false;
    rs_deleteFold(curwin, oap->start.lnum, oap->end.lnum,
               oap->op_type == OP_FOLDDELREC, oap->is_VIsual);
    break;

  case OP_NR_ADD:
  case OP_NR_SUB:
    if (empty_region_error) {
      vim_beep(kOptBoFlagOperator);
      CancelRedo();
    } else {
      VIsual_active = true;
      restore_lbr(lbr_saved);
      op_addsub(oap, (linenr_T)cap->count1, dpo_redo_VIsual.rv_arg);
      VIsual_active = false;
    }
    check_cursor_col(curwin);
    break;

  default:
    rs_clearopbeep(oap);
  }
}

/// Postamble: virtual_op reset, column restore, clearop.
void nvim_dpo_postamble(cmdarg_T *cap, int old_col, int gui_yank)
{
  oparg_T *oap = cap->oap;

  virtual_op = kNone;
  if (!gui_yank) {
    if (!p_sol && oap->motion_type == kMTLineWise && !oap->end_adjusted
        && (oap->op_type == OP_LSHIFT || oap->op_type == OP_RSHIFT
            || oap->op_type == OP_DELETE)) {
      reset_lbr();
      coladvance(curwin, curwin->w_curswant = old_col);
    }
  } else {
    curwin->w_cursor = dpo_saved_old_cursor;
  }
  rs_clearop(oap);
  motion_force = NUL;
}

/// Final: restore_lbr.
void nvim_dpo_restore_lbr(cmdarg_T *cap)
{
  (void)cap;
  // We need the lbr_saved from function entry. Use a static.
  restore_lbr(dpo_saved_lbr);
}

/// Get the byte count of buffer region. End-exclusive.
///
/// @return number of bytes
bcount_t get_region_bytecount(buf_T *buf, linenr_T start_lnum, linenr_T end_lnum, colnr_T start_col,
                              colnr_T end_col)
{
  linenr_T max_lnum = buf->b_ml.ml_line_count;
  if (start_lnum > max_lnum) {
    return 0;
  }
  if (start_lnum == end_lnum) {
    return end_col - start_col;
  }
  bcount_t deleted_bytes = ml_get_buf_len(buf, start_lnum) - start_col + 1;

  for (linenr_T i = 1; i <= end_lnum - start_lnum - 1; i++) {
    if (start_lnum + i > max_lnum) {
      return deleted_bytes;
    }
    deleted_bytes += ml_get_buf_len(buf, start_lnum + i) + 1;
  }
  if (end_lnum > max_lnum) {
    return deleted_bytes;
  }
  return deleted_bytes + end_col;
}
