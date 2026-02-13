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

// Rust FFI declarations
extern bool rs_file_ff_differs(buf_T *buf, bool ignore_empty);
extern void rs_save_file_ff(buf_T *buf);
extern void rs_unchanged(buf_T *buf, bool ff, bool always_inc_changedtick);
extern int rs_get_leader_len(const char *line, char **flags, bool backward, bool include_space);
extern int rs_get_last_leader_offset(const char *line, char **flags);
extern void rs_change_warning(buf_T *buf, int col);
extern void rs_changed(buf_T *buf);
extern void rs_changed_internal(buf_T *buf);
extern void rs_changed_lines_invalidate_buf(buf_T *buf, linenr_T lnum, colnr_T col,
                                            linenr_T lnume, linenr_T xtra);
extern void rs_changed_lines_redraw_buf(buf_T *buf, linenr_T lnum, linenr_T lnume, linenr_T xtra);
extern void rs_changed_bytes(linenr_T lnum, colnr_T col);
extern void rs_inserted_bytes(linenr_T lnum, colnr_T start_col, int old_col, int new_col);
extern void rs_changed_lines(buf_T *buf, linenr_T lnum, colnr_T col, linenr_T lnume,
                             linenr_T xtra, bool do_buf_event);
extern void rs_ins_bytes(const char *p);
extern void rs_ins_bytes_len(const char *p, size_t len);
extern void rs_ins_char(int c);
extern void rs_ins_char_bytes(char *buf, size_t charlen);
extern void rs_ins_str(const char *s, size_t slen);
extern int rs_del_char(bool fixpos);
extern int rs_del_chars(int count, int fixpos);
extern int rs_del_bytes(colnr_T count, bool fixpos_arg, bool use_delcombine);
extern void rs_truncate_line(int fixpos);
extern void rs_appended_lines_buf(buf_T *buf, linenr_T lnum, linenr_T count);
extern void rs_appended_lines(linenr_T lnum, linenr_T count);
extern void rs_appended_lines_mark(linenr_T lnum, int count);
extern void rs_deleted_lines_buf(buf_T *buf, linenr_T lnum, linenr_T count);
extern void rs_deleted_lines(linenr_T lnum, linenr_T count);
extern void rs_deleted_lines_mark(linenr_T lnum, int count);
extern void rs_del_lines(linenr_T nlines, bool undo);
extern bool rs_open_line(int dir, int flags, int second_line_indent, bool *did_do_comment);

void change_warning(buf_T *buf, int col)
{
  rs_change_warning(buf, col);
}

void changed(buf_T *buf)
{
  rs_changed(buf);
}

void changed_internal(buf_T *buf)
{
  rs_changed_internal(buf);
}

/// Invalidate a window's w_valid flags and w_lines[] entries after changing lines.
static void changed_lines_invalidate_win(win_T *wp, linenr_T lnum, colnr_T col, linenr_T lnume,
                                         linenr_T xtra)
{
  // If the changed line is in a range of previously folded lines,
  // compare with the first line in that range.
  if (wp->w_cursor.lnum <= lnum) {
    int i = find_wl_entry(wp, lnum);
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

/// Line changed_lines_invalidate_win(), but for all windows displaying a buffer.
void changed_lines_invalidate_buf(buf_T *buf, linenr_T lnum, colnr_T col, linenr_T lnume,
                                  linenr_T xtra)
{
  rs_changed_lines_invalidate_buf(buf, lnum, col, lnume, xtra);
}

/// Common code for when a change was made.
/// See changed_lines() for the arguments.
/// Careful: may trigger autocommands that reload the buffer.
void changed_common(buf_T *buf, linenr_T lnum, colnr_T col, linenr_T lnume, linenr_T xtra)
{
  // mark the buffer as modified
  changed(buf);

  FOR_ALL_WINDOWS_IN_TAB(win, curtab) {
    if (win->w_buffer == buf && win->w_p_diff && diff_internal()) {
      curtab->tp_diff_update = true;
      diff_update_line(lnum);
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
      if (hasAnyFolding(wp)) {
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

/// Changed bytes within a single line for the current buffer.
/// - marks the windows on this buffer to be redisplayed
/// - marks the buffer changed by calling changed()
/// - invalidates cached values
/// Careful: may trigger autocommands that reload the buffer.
void changed_bytes(linenr_T lnum, colnr_T col)
{
  rs_changed_bytes(lnum, col);
}

void inserted_bytes(linenr_T lnum, colnr_T start_col, int old_col, int new_col)
{
  rs_inserted_bytes(lnum, start_col, old_col, new_col);
}

/// Appended "count" lines below line "lnum" in the given buffer.
/// Must be called AFTER the change and after mark_adjust().
/// Takes care of marking the buffer to be redrawn and sets the changed flag.
void appended_lines_buf(buf_T *buf, linenr_T lnum, linenr_T count)
{
  rs_appended_lines_buf(buf, lnum, count);
}

/// Appended "count" lines below line "lnum" in the current buffer.
/// Must be called AFTER the change and after mark_adjust().
/// Takes care of marking the buffer to be redrawn and sets the changed flag.
void appended_lines(linenr_T lnum, linenr_T count)
{
  rs_appended_lines(lnum, count);
}

/// Like appended_lines(), but adjust marks first.
void appended_lines_mark(linenr_T lnum, int count)
{
  rs_appended_lines_mark(lnum, count);
}

/// Deleted "count" lines at line "lnum" in the given buffer.
/// Must be called AFTER the change and after mark_adjust().
/// Takes care of marking the buffer to be redrawn and sets the changed flag.
void deleted_lines_buf(buf_T *buf, linenr_T lnum, linenr_T count)
{
  rs_deleted_lines_buf(buf, lnum, count);
}

/// Deleted "count" lines at line "lnum" in the current buffer.
/// Must be called AFTER the change and after mark_adjust().
/// Takes care of marking the buffer to be redrawn and sets the changed flag.
void deleted_lines(linenr_T lnum, linenr_T count)
{
  rs_deleted_lines(lnum, count);
}

/// Like deleted_lines(), but adjust marks first.
/// Make sure the cursor is on a valid line before calling, a GUI callback may
/// be triggered to display the cursor.
void deleted_lines_mark(linenr_T lnum, int count)
{
  rs_deleted_lines_mark(lnum, count);
}

/// Marks the area to be redrawn after a change.
/// Consider also calling changed_lines_invalidate_buf().
///
/// @param buf the buffer where lines were changed
/// @param lnum first line with change
/// @param lnume line below last changed line
/// @param xtra number of extra lines (negative when deleting)
void changed_lines_redraw_buf(buf_T *buf, linenr_T lnum, linenr_T lnume, linenr_T xtra)
{
  rs_changed_lines_redraw_buf(buf, lnum, lnume, xtra);
}

/// Changed lines for a buffer.
/// Must be called AFTER the change and after mark_adjust().
/// - mark the buffer changed by calling changed()
/// - mark the windows on this buffer to be redisplayed
/// - invalidate cached values
/// "lnum" is the first line that needs displaying, "lnume" the first line
/// below the changed lines (BEFORE the change).
/// When only inserting lines, "lnum" and "lnume" are equal.
/// Takes care of calling changed() and updating b_mod_*.
/// Careful: may trigger autocommands that reload the buffer.
///
/// @param lnum  first line with change
/// @param col  column in first line with change
/// @param lnume  line below last changed line
/// @param xtra  number of extra lines (negative when deleting)
/// @param do_buf_event  some callers like undo/redo call changed_lines() and
/// then increment changedtick *again*. This flag allows these callers to send
/// the nvim_buf_lines_event events after they're done modifying changedtick.
void changed_lines(buf_T *buf, linenr_T lnum, colnr_T col, linenr_T lnume, linenr_T xtra,
                   bool do_buf_event)
{
  rs_changed_lines(buf, lnum, col, lnume, xtra, do_buf_event);
}

/// Called when the changed flag must be reset for buffer `buf`.
/// When `ff` is true also reset 'fileformat'.
/// When `always_inc_changedtick` is true b:changedtick is incremented even
/// when the changed flag was off.
void unchanged(buf_T *buf, bool ff, bool always_inc_changedtick)
{
  rs_unchanged(buf, ff, always_inc_changedtick);
}

/// Save the current values of 'fileformat' and 'fileencoding', so that we know
/// the file must be considered changed when the value is different.
void save_file_ff(buf_T *buf)
{
  rs_save_file_ff(buf);
}

/// Return true if 'fileformat' and/or 'fileencoding' has a different value
/// from when editing started (save_file_ff() called).
/// Also when 'endofline' was changed and 'binary' is set, or when 'bomb' was
/// changed and 'binary' is not set.
/// Also when 'endofline' was changed and 'fixeol' is not set.
/// When "ignore_empty" is true don't consider a new, empty buffer to be
/// changed.
bool file_ff_differs(buf_T *buf, bool ignore_empty)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_file_ff_differs(buf, ignore_empty);
}

/// Insert string "p" at the cursor position.  Stops at a NUL byte.
/// Handles Replace mode and multi-byte characters.
void ins_bytes(char *p)
{
  rs_ins_bytes(p);
}

void ins_bytes_len(char *p, size_t len)
{
  rs_ins_bytes_len(p, len);
}

void ins_char(int c)
{
  rs_ins_char(c);
}

void ins_char_bytes(char *buf, size_t charlen)
{
  rs_ins_char_bytes(buf, charlen);
}

void ins_str(char *s, size_t slen)
{
  rs_ins_str(s, slen);
}

// Delete one character under the cursor.
// If "fixpos" is true, don't leave the cursor on the NUL after the line.
// Caller must have prepared for undo.
//
// return FAIL for failure, OK otherwise
int del_char(bool fixpos)
{
  return rs_del_char(fixpos);
}

/// Like del_bytes(), but delete characters instead of bytes.
int del_chars(int count, int fixpos)
{
  return rs_del_chars(count, fixpos);
}

/// Delete "count" bytes under the cursor.
/// If "fixpos" is true, don't leave the cursor on the NUL after the line.
/// Caller must have prepared for undo.
///
/// @param  count           number of bytes to be deleted
/// @param  fixpos_arg      leave the cursor on the NUL after the line
/// @param  use_delcombine  'delcombine' option applies
///
/// @return FAIL for failure, OK otherwise
int del_bytes(colnr_T count, bool fixpos_arg, bool use_delcombine)
{
  return rs_del_bytes(count, fixpos_arg, use_delcombine);
}

/// open_line: Add a new line below or above the current line.
///
/// For MODE_VREPLACE state, we only add a new line when we get to the end of
/// the file, otherwise we just start replacing the next line.
///
/// Caller must take care of undo.  Since MODE_VREPLACE may affect any number of
/// lines however, it may call u_save_cursor() again when starting to change a
/// new line.
/// "flags": OPENLINE_DELSPACES delete spaces after cursor
///          OPENLINE_DO_COM    format comments
///          OPENLINE_KEEPTRAIL keep trailing spaces
///          OPENLINE_MARKFIX   adjust mark positions after the line break
///          OPENLINE_COM_LIST  format comments with list or 2nd line indent
///          OPENLINE_FORCE_INDENT  set indent from second_line_indent, ignore 'autoindent'
///
/// "second_line_indent": indent for after ^^D in Insert mode or if flag
///                       OPENLINE_COM_LIST
/// "did_do_comment" is set to true when intentionally putting the comment
/// leader in front of the new line.
///
/// @param dir  FORWARD or BACKWARD
///
/// @return true on success, false on failure
bool open_line(int dir, int flags, int second_line_indent, bool *did_do_comment)
{
  return rs_open_line(dir, flags, second_line_indent, did_do_comment);
}

/// Delete from cursor to end of line.
/// Caller must have prepared for undo.
/// If "fixpos" is true fix the cursor position when done.
void truncate_line(int fixpos)
{
  rs_truncate_line(fixpos);
}

/// Delete "nlines" lines at the cursor.
/// Saves the lines for undo first if "undo" is true.
void del_lines(linenr_T nlines, bool undo)
{
  rs_del_lines(nlines, undo);
}

/// Returns the length in bytes of the prefix of the given string which introduces a comment.
///
/// If this string is not a comment then 0 is returned.
/// When "flags" is not NULL, it is set to point to the flags of the recognized
/// comment leader.
/// "backward" must be true for the "O" command.
/// If "include_space" is set, include trailing whitespace while calculating
/// the length.
int get_leader_len(char *line, char **flags, bool backward, bool include_space)
{
  return rs_get_leader_len(line, flags, backward, include_space);
}

/// Return the offset at which the last comment in line starts.
/// If there is no comment in the whole line, -1 is returned.
///
/// When "flags" is not null, it is set to point to the flags describing the
/// recognized comment leader.
int get_last_leader_offset(char *line, char **flags)
{
  return rs_get_last_leader_offset(line, flags);
}