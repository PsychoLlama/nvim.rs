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
/// Do yank and register handling.
/// Returns: OK (1) if should return OK (read-only reg), 2 if proceed normally.


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

void clear_oparg(oparg_T *oap) { CLEAR_POINTER(oap); }

