/// change.c: functions related to changing text

#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/assert_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/buffer_updates.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/diff.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/extmark.h"
#include "nvim/extmark_defs.h"
#include "nvim/fold.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/indent.h"
#include "nvim/indent_c.h"
#include "nvim/insexpand.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/marktree_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/time.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/search.h"
#include "nvim/spell.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/textformat.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"

#include "change.c.generated.h"

extern int rs_diff_internal(void);
extern void rs_diff_update_line(linenr_T lnum);

// Rust fold FFI declarations
extern int rs_hasAnyFolding(win_T *win);
extern int rs_find_wl_entry(win_T *win, linenr_T lnum);

/// Invalidate a window's w_valid flags and w_lines[] entries after changing lines.
static void changed_lines_invalidate_win(win_T *wp, linenr_T lnum, colnr_T col, linenr_T lnume,
                                         linenr_T xtra)
{
  // If the changed line is in a range of previously folded lines,
  // compare with the first line in that range.
  if (wp->w_cursor.lnum <= lnum) {
    int i = rs_find_wl_entry(wp, lnum);
    if (i >= 0 && wp->w_cursor.lnum > wp->w_lines[i].wl_lnum) {
      changed_line_abv_curs_win(wp);
    }
  }

  if (wp->w_cursor.lnum > lnum) {
    changed_line_abv_curs_win(wp);
  } else if (wp->w_cursor.lnum == lnum && wp->w_cursor.col >= col) {
    changed_cline_bef_curs(wp);
  }
  if (wp->w_botline >= lnum) {
    // Assume that botline doesn't change (inserted lines make
    // other lines scroll down below botline).
    approximate_botline_win(wp);
  }

  // If lines have been inserted/deleted and the buffer has virt_lines, or
  // inline virt_text with 'wrap' enabled, invalidate the line after the changed
  // lines. virt_lines may now be drawn above that line, and inline virt_text
  // may cause that line to wrap.
  if ((xtra < 0 && wp->w_p_wrap && buf_meta_total(wp->w_buffer, kMTMetaInline))
      || (xtra != 0 && buf_meta_total(wp->w_buffer, kMTMetaLines))) {
    lnume++;
  }
  // Check if any w_lines[] entries have become invalid.
  // For entries below the change: Correct the lnums for inserted/deleted lines.
  // Makes it possible to stop displaying after the change.
  for (int i = 0; i < wp->w_lines_valid; i++) {
    if (wp->w_lines[i].wl_valid) {
      if (wp->w_lines[i].wl_lnum >= lnum) {
        // Do not change wl_lnum at index zero, it is used to compare with w_topline.
        // Invalidate it instead.
        if (i == 0 || wp->w_lines[i].wl_lnum < lnume) {
          // line included in change
          wp->w_lines[i].wl_valid = false;
        } else if (xtra != 0) {
          // line below change
          wp->w_lines[i].wl_lnum += xtra;
          wp->w_lines[i].wl_foldend += xtra;
          wp->w_lines[i].wl_lastlnum += xtra;
        }
      } else if (wp->w_lines[i].wl_lastlnum >= lnum) {
        // change somewhere inside this range of folded or concealed lines,
        // may need to be redrawn
        wp->w_lines[i].wl_valid = false;
      }
    }
  }
}

/// Common code for when a change was made.
/// See changed_lines() for the arguments.
/// Careful: may trigger autocommands that reload the buffer.
void changed_common(buf_T *buf, linenr_T lnum, colnr_T col, linenr_T lnume, linenr_T xtra)
{
  // mark the buffer as modified
  changed(buf);

  FOR_ALL_WINDOWS_IN_TAB(win, curtab) {
    if (win->w_buffer == buf && win->w_p_diff && rs_diff_internal()) {
      curtab->tp_diff_update = true;
      rs_diff_update_line(lnum);
    }
  }

  // set the '. mark
  if ((cmdmod.cmod_flags & CMOD_KEEPJUMPS) == 0) {
    fmarkv_T view = INIT_FMARKV;
    // Set the markview only if lnum is visible, as changes might be done
    // outside of the current window view.

    if (curwin->w_buffer == buf) {
      if (lnum >= curwin->w_topline && lnum <= curwin->w_botline) {
        view = mark_view_make(curwin->w_topline, curwin->w_cursor);
      }
    }
    RESET_FMARK(&buf->b_last_change, ((pos_T) { lnum, col, 0 }), buf->handle, view);

    // Create a new entry if a new undo-able change was started or we
    // don't have an entry yet.
    if (buf->b_new_change || buf->b_changelistlen == 0) {
      bool add;
      if (buf->b_changelistlen == 0) {
        add = true;
      } else {
        // Don't create a new entry when the line number is the same
        // as the last one and the column is not too far away.  Avoids
        // creating many entries for typing "xxxxx".
        pos_T *p = &buf->b_changelist[buf->b_changelistlen - 1].mark;
        if (p->lnum != lnum) {
          add = true;
        } else {
          int cols = comp_textwidth(false);
          if (cols == 0) {
            cols = 79;
          }
          add = (p->col + cols < col || col + cols < p->col);
        }
      }
      if (add) {
        // This is the first of a new sequence of undo-able changes
        // and it's at some distance of the last change.  Use a new
        // position in the changelist.
        buf->b_new_change = false;

        if (buf->b_changelistlen == JUMPLISTSIZE) {
          // changelist is full: remove oldest entry
          buf->b_changelistlen = JUMPLISTSIZE - 1;
          memmove(buf->b_changelist, buf->b_changelist + 1,
                  sizeof(buf->b_changelist[0]) * (JUMPLISTSIZE - 1));
          FOR_ALL_TAB_WINDOWS(tp, wp) {
            // Correct position in changelist for other windows on
            // this buffer.
            if (wp->w_buffer == buf && wp->w_changelistidx > 0) {
              wp->w_changelistidx--;
            }
          }
        }
        FOR_ALL_TAB_WINDOWS(tp, wp) {
          // For other windows, if the position in the changelist is
          // at the end it stays at the end.
          if (wp->w_buffer == buf
              && wp->w_changelistidx == buf->b_changelistlen) {
            wp->w_changelistidx++;
          }
        }
        buf->b_changelistlen++;
      }
    }
    buf->b_changelist[buf->b_changelistlen - 1] =
      buf->b_last_change;
    // The current window is always after the last change, so that "g,"
    // takes you back to it.
    if (curwin->w_buffer == buf) {
      curwin->w_changelistidx = buf->b_changelistlen;
    }
  }

  if (curwin->w_buffer == buf && VIsual_active) {
    check_visual_pos();
  }

  FOR_ALL_TAB_WINDOWS(tp, wp) {
    if (wp->w_buffer == buf) {
      // Mark this window to be redrawn later.
      if (!redraw_not_allowed && wp->w_redr_type < UPD_VALID) {
        wp->w_redr_type = UPD_VALID;
      }

      // When inserting/deleting lines and the window has specific lines
      // to be redrawn, w_redraw_top and w_redraw_bot may now be invalid,
      // so just redraw everything.
      if (xtra != 0 && wp->w_redraw_top != 0) {
        redraw_later(wp, UPD_NOT_VALID);
      }

      linenr_T last = lnume + xtra - 1;  // last line after the change

      // Reset "w_skipcol" if the topline length has become smaller to
      // such a degree that nothing will be visible anymore, accounting
      // for 'smoothscroll' <<< or 'listchars' "precedes" marker.
      if (wp->w_skipcol > 0
          && (last < wp->w_topline
              || (wp->w_topline >= lnum
                  && wp->w_topline < lnume
                  && (linetabsize_eol(wp, wp->w_topline)
                      <= wp->w_skipcol + sms_marker_overlap(wp, -1))))) {
        wp->w_skipcol = 0;
      }

      // Check if a change in the buffer has invalidated the cached
      // values for the cursor.
      // Update the folds for this window.  Can't postpone this, because
      // a following operator might work on the whole fold: ">>dd".
      foldUpdate(wp, lnum, last);

      // The change may cause lines above or below the change to become
      // included in a fold.  Set lnum/lnume to the first/last line that
      // might be displayed differently.
      // Set w_cline_folded here as an efficient way to update it when
      // inserting lines just above a closed fold.
      bool folded = hasFoldingWin(wp, lnum, &lnum, NULL, false, NULL);
      if (wp->w_cursor.lnum == lnum) {
        wp->w_cline_folded = folded;
      }
      folded = hasFoldingWin(wp, last, NULL, &last, false, NULL);
      if (wp->w_cursor.lnum == last) {
        wp->w_cline_folded = folded;
      }

      changed_lines_invalidate_win(wp, lnum, col, lnume, xtra);

      // Take care of side effects for setting w_topline when folds have
      // changed.  Esp. when the buffer was changed in another window.
      if (rs_hasAnyFolding(wp)) {
        set_topline(wp, wp->w_topline);
      }

      // If lines have been added or removed, relative numbering always
      // requires an update even if cursor didn't move.
      if (wp->w_p_rnu && xtra != 0) {
        wp->w_last_cursor_lnum_rnu = 0;
      }

      if (wp->w_p_cul && wp->w_last_cursorline >= lnum) {
        if (wp->w_last_cursorline < lnume) {
          // If 'cursorline' was inside the change, it has already
          // been invalidated in w_lines[] by the loop above.
          wp->w_last_cursorline = 0;
        } else {
          // If 'cursorline' was below the change, adjust its lnum.
          wp->w_last_cursorline += xtra;
        }
      }
    }

    if (wp == curwin && xtra != 0 && search_hl_has_cursor_lnum >= lnum) {
      search_hl_has_cursor_lnum += xtra;
    }
  }

  // Call update_screen() later, which checks out what needs to be redrawn,
  // since it notices b_mod_set and then uses b_mod_*.
  set_must_redraw(UPD_VALID);

  // when the cursor line is changed always trigger CursorMoved
  if (last_cursormoved_win == curwin && curwin->w_buffer == buf
      && lnum <= curwin->w_cursor.lnum
      && lnume + (xtra < 0 ? -xtra : xtra) > curwin->w_cursor.lnum) {
    last_cursormoved.lnum = 0;
  }
}

