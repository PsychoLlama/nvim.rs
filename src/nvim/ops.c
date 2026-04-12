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



// block_insert and op_insert migrated to Rust (op_insert_full.rs)

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


void clear_oparg(oparg_T *oap) { CLEAR_POINTER(oap); }

