// drawscreen.c: Code for updating all the windows on the screen.
// This is the top level, drawline.c is the middle and grid.c the lower level.

// update_screen() is the function that updates all windows and status lines.
// It is called from the main loop when must_redraw is non-zero.  It may be
// called from other places when an immediate screen update is needed.
//
// The part of the buffer that is displayed in a window is set with:
// - w_topline (first buffer line in window)
// - w_topfill (filler lines above the first line)
// - w_leftcol (leftmost window cell in window),
// - w_skipcol (skipped window cells of first line)
//
// Commands that only move the cursor around in a window, do not need to take
// action to update the display.  The main loop will check if w_topline is
// valid and update it (scroll the window) when needed.
//
// Commands that scroll a window change w_topline and must call
// check_cursor() to move the cursor into the visible part of the window, and
// call redraw_later(wp, UPD_VALID) to have the window displayed by update_screen()
// later.
//
// Commands that change text in the buffer must call changed_bytes() or
// changed_lines() to mark the area that changed and will require updating
// later.  The main loop will call update_screen(), which will update each
// window that shows the changed buffer.  This assumes text above the change
// can remain displayed as it is.  Text after the change may need updating for
// scrolling, folding and syntax highlighting.
//
// Commands that change how a window is displayed (e.g., setting 'list') or
// invalidate the contents of a window in another way (e.g., change fold
// settings), must call redraw_later(wp, UPD_NOT_VALID) to have the whole window
// redisplayed by update_screen() later.
//
// Commands that change how a buffer is displayed (e.g., setting 'tabstop')
// must call redraw_curbuf_later(UPD_NOT_VALID) to have all the windows for the
// buffer redisplayed by update_screen() later.
//
// Commands that change highlighting and possibly cause a scroll too must call
// redraw_later(wp, UPD_SOME_VALID) to update the whole window but still use
// scrolling to avoid redrawing everything.  But the length of displayed lines
// must not change, use UPD_NOT_VALID then.
//
// Commands that move the window position must call redraw_later(wp, UPD_NOT_VALID).
// TODO(neovim): should minimize redrawing by scrolling when possible.
//
// Commands that change everything (e.g., resizing the screen) must call
// redraw_all_later(UPD_NOT_VALID) or redraw_all_later(UPD_CLEAR).
//
// Things that are handled indirectly:
// - When messages scroll the screen up, msg_scrolled will be set and
//   update_screen() called to redraw.

#include <assert.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/private/defs.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/decoration.h"
#include "nvim/decoration_defs.h"
#include "nvim/decoration_provider.h"
#include "nvim/diff.h"
#include "nvim/digraph.h"
#include "nvim/drawline.h"
#include "nvim/drawscreen.h"
#include "nvim/ex_getln.h"
#include "nvim/fold.h"
#include "nvim/fold_defs.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/grid_defs.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/insexpand.h"
#include "nvim/marktree_defs.h"
#include "nvim/match.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/normal_defs.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/os_defs.h"
#include "nvim/plines.h"
#include "nvim/popupmenu.h"
#include "nvim/pos_defs.h"
#include "nvim/profile.h"
#include "nvim/regexp.h"
#include "nvim/search.h"
#include "nvim/spell.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/statusline.h"
#include "nvim/strings.h"
#include "nvim/syntax.h"
#include "nvim/syntax_defs.h"
#include "nvim/terminal.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_compositor.h"
#include "nvim/ui_defs.h"
#include "nvim/version.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

/// corner value flags for hsep_connected and vsep_connected
typedef enum {
  WC_TOP_LEFT = 0,
  WC_TOP_RIGHT,
  WC_BOTTOM_LEFT,
  WC_BOTTOM_RIGHT,
} WindowCorner;

#include "drawscreen.c.generated.h"

// Rust FFI declarations
extern int rs_global_stl_height(void);
extern int rs_min_rows(tabpage_T *tp);
extern int rs_cmdline_number_prompt(void);
extern int rs_hasAnyFolding(win_T *win);
extern foldinfo_T rs_fold_info(win_T *win, linenr_T lnum);
extern void rs_clear_showcmd(void);
extern void rs_draw_vsep_win(win_T *wp);
extern void rs_draw_hsep_win(win_T *wp);
extern void rs_draw_sep_connectors_win(win_T *wp);
extern void rs_win_scroll_lines(win_T *wp, int row, int line_count);
extern int rs_showmode(void);
extern void rs_screenclear(void);
extern void rs_drawscreen_screen_resize(int width, int height);
extern void rs_show_cursor_info_later(bool force);
extern int rs_update_screen(void);
extern void rs_win_update_visual_region(win_T *wp, buf_T *buf, int type,
                                        int top_end, bool scrolled_down,
                                        int *mid_start, int *mid_end);
extern bool redrawing(void);
extern void rs_ins_compl_show_pum(void);

bool redraw_popupmenu = false;
bool msg_grid_invalid = false;
bool resizing_autocmd = false;
static bool conceal_cursor_used = false;

static void win_update(win_T *wp);  // forward declaration


void screenclear(void) { rs_screenclear(); }

/// Set dimensions of the Nvim application "screen".
void screen_resize(int width, int height) { rs_drawscreen_screen_resize(width, height); }

// win_update visual region helper

/// Full implementation of the visual region update section of win_update().
/// Called from rs_win_update_visual_region() in Rust.
void nvim_win_visual_region_impl(win_T *wp, buf_T *buf, int type,
                                 int top_end, bool scrolled_down,
                                 int *mid_start, int *mid_end)
{
  // check if we are updating or removing the inverted part
  if ((VIsual_active && buf == curwin->w_buffer)
      || (wp->w_old_cursor_lnum != 0 && type != UPD_NOT_VALID)) {
    linenr_T from, to;

    if (VIsual_active) {
      if (VIsual_mode != wp->w_old_visual_mode || type == UPD_INVERTED_ALL) {
        if (curwin->w_cursor.lnum < VIsual.lnum) {
          from = curwin->w_cursor.lnum;
          to = VIsual.lnum;
        } else {
          from = VIsual.lnum;
          to = curwin->w_cursor.lnum;
        }
        from = MIN(MIN(from, wp->w_old_cursor_lnum), wp->w_old_visual_lnum);
        to = MAX(MAX(to, wp->w_old_cursor_lnum), wp->w_old_visual_lnum);
      } else {
        if (curwin->w_cursor.lnum < wp->w_old_cursor_lnum) {
          from = curwin->w_cursor.lnum;
          to = wp->w_old_cursor_lnum;
        } else {
          from = wp->w_old_cursor_lnum;
          to = curwin->w_cursor.lnum;
          if (from == 0) {
            from = to;
          }
        }

        if (VIsual.lnum != wp->w_old_visual_lnum
            || VIsual.col != wp->w_old_visual_col) {
          if (wp->w_old_visual_lnum < from
              && wp->w_old_visual_lnum != 0) {
            from = wp->w_old_visual_lnum;
          }
          to = MAX(MAX(to, wp->w_old_visual_lnum), VIsual.lnum);
          from = MIN(from, VIsual.lnum);
        }
      }

      if (VIsual_mode == Ctrl_V) {
        colnr_T fromc, toc;
        unsigned save_ve_flags = curwin->w_ve_flags;

        if (curwin->w_p_lbr) {
          curwin->w_ve_flags = kOptVeFlagAll;
        }

        getvcols(wp, &VIsual, &curwin->w_cursor, &fromc, &toc);
        toc++;
        curwin->w_ve_flags = save_ve_flags;
        if (curwin->w_curswant == MAXCOL) {
          if (get_ve_flags(curwin) & kOptVeFlagBlock) {
            pos_T pos;
            int cursor_above = curwin->w_cursor.lnum < VIsual.lnum;

            toc = 0;
            pos.coladd = 0;
            for (pos.lnum = curwin->w_cursor.lnum;
                 cursor_above ? pos.lnum <= VIsual.lnum : pos.lnum >= VIsual.lnum;
                 pos.lnum += cursor_above ? 1 : -1) {
              colnr_T t;

              pos.col = ml_get_buf_len(wp->w_buffer, pos.lnum);
              getvvcol(wp, &pos, NULL, NULL, &t);
              toc = MAX(toc, t);
            }
            toc++;
          } else {
            toc = MAXCOL;
          }
        }

        if (fromc != wp->w_old_cursor_fcol
            || toc != wp->w_old_cursor_lcol) {
          from = MIN(from, VIsual.lnum);
          to = MAX(to, VIsual.lnum);
        }
        wp->w_old_cursor_fcol = fromc;
        wp->w_old_cursor_lcol = toc;
      }
    } else {
      if (wp->w_old_cursor_lnum < wp->w_old_visual_lnum) {
        from = wp->w_old_cursor_lnum;
        to = wp->w_old_visual_lnum;
      } else {
        from = wp->w_old_visual_lnum;
        to = wp->w_old_cursor_lnum;
      }
    }

    from = MAX(from, wp->w_topline);

    if (wp->w_valid & VALID_BOTLINE) {
      from = MIN(from, wp->w_botline - 1);
      to = MIN(to, wp->w_botline - 1);
    }

    if (*mid_start > 0) {
      linenr_T lnum = wp->w_topline;
      int idx = 0;
      int srow = 0;
      if (scrolled_down) {
        *mid_start = top_end;
      } else {
        *mid_start = 0;
      }
      while (lnum < from && idx < wp->w_lines_valid) {
        if (wp->w_lines[idx].wl_valid) {
          *mid_start += wp->w_lines[idx].wl_size;
        } else if (!scrolled_down) {
          srow += wp->w_lines[idx].wl_size;
        }
        idx++;
        if (idx < wp->w_lines_valid && wp->w_lines[idx].wl_valid) {
          lnum = wp->w_lines[idx].wl_lnum;
        } else {
          lnum++;
        }
      }
      srow += *mid_start;
      *mid_end = wp->w_view_height;
      for (; idx < wp->w_lines_valid; idx++) {
        if (wp->w_lines[idx].wl_valid
            && wp->w_lines[idx].wl_lnum >= to + 1) {
          *mid_end = srow;
          break;
        }
        srow += wp->w_lines[idx].wl_size;
      }
    }
  }

  if (VIsual_active && buf == curwin->w_buffer) {
    wp->w_old_visual_mode = (char)VIsual_mode;
    wp->w_old_cursor_lnum = curwin->w_cursor.lnum;
    wp->w_old_visual_lnum = VIsual.lnum;
    wp->w_old_visual_col = VIsual.col;
    wp->w_old_curswant = curwin->w_curswant;
  } else {
    wp->w_old_visual_mode = 0;
    wp->w_old_cursor_lnum = 0;
    wp->w_old_visual_lnum = 0;
    wp->w_old_visual_col = 0;
  }
}


/// Redraw the parts of the screen that is marked for redraw.
///
/// Most code shouldn't call this directly, rather use redraw_later() and
/// and redraw_all_later() to mark parts of the screen as needing a redraw.
int update_screen(void) { return rs_update_screen(); }


/// Show current cursor info in ruler and various other places
///
/// @param always  if false, only show ruler if position has changed.
void show_cursor_info_later(bool force) { rs_show_cursor_info_later(force); }


/// Show the current mode and ruler.
///
/// If clear_cmdline is true, clear the rest of the cmdline.
/// If clear_cmdline is false there may be a message there that needs to be
/// cleared only if a mode is shown.
/// If redraw_mode is true show or clear the mode.
/// @return the length of the message (0 if no message).
int showmode(void) { return rs_showmode(); }

#define COL_RULER 17        // columns needed by standard ruler

/// Redraw entire window "wp" if "auto" 'signcolumn' width has changed.
static bool win_redraw_signcols(win_T *wp)
{
  buf_T *buf = wp->w_buffer;

  if (!buf->b_signcols.autom
      && (*wp->w_p_stc != NUL || (wp->w_maxscwidth > 1 && wp->w_minscwidth != wp->w_maxscwidth))) {
    buf->b_signcols.autom = true;
    buf_signcols_count_range(buf, 0, buf->b_ml.ml_line_count - 1, MAXLNUM, kFalse);
  }

  while (buf->b_signcols.max > 0 && buf->b_signcols.count[buf->b_signcols.max - 1] == 0) {
    buf->b_signcols.max--;
  }

  int width = MIN(wp->w_maxscwidth, buf->b_signcols.max);
  bool rebuild_stc = buf->b_signcols.max != buf->b_signcols.last_max && *wp->w_p_stc != NUL;

  if (rebuild_stc) {
    wp->w_nrwidth_line_count = 0;
  } else if (wp->w_minscwidth == 0 && wp->w_maxscwidth == 1) {
    width = buf_meta_total(buf, kMTMetaSignText) > 0;
  }

  int scwidth = wp->w_scwidth;
  wp->w_scwidth = MAX(MAX(0, wp->w_minscwidth), width);
  return (wp->w_scwidth != scwidth || rebuild_stc);
}


/// Update a single window.
///
/// This may cause the windows below it also to be redrawn (when clearing the
/// screen or scrolling lines).
///
/// How the window is redrawn depends on wp->w_redr_type.  Each type also
/// implies the one below it.
/// UPD_NOT_VALID    redraw the whole window
/// UPD_SOME_VALID   redraw the whole window but do scroll when possible
/// UPD_REDRAW_TOP   redraw the top w_upd_rows window lines, otherwise like UPD_VALID
/// UPD_INVERTED     redraw the changed part of the Visual area
/// UPD_INVERTED_ALL redraw the whole Visual area
/// UPD_VALID        1. scroll up/down to adjust for a changed w_topline
///                  2. update lines at the top when scrolled down
///                  3. redraw changed text:
///                     - if wp->w_buffer->b_mod_set set, update lines between
///                       b_mod_top and b_mod_bot.
///                     - if wp->w_redraw_top non-zero, redraw lines between
///                       wp->w_redraw_top and wp->w_redraw_bot.
///                     - continue redrawing when syntax status is invalid.
///                  4. if scrolled up, update lines at the bottom.
/// This results in three areas that may need updating:
/// top: from first row to top_end (when scrolled down)
/// mid: from mid_start to mid_end (update inversion or changed text)
/// bot: from bot_start to last row (when scrolled up)
static void win_update(win_T *wp)
{
  int top_end = 0;              // Below last row of the top area that needs
                                // updating.  0 when no top area updating.
  int mid_start = 999;          // first row of the mid area that needs
                                // updating.  999 when no mid area updating.
  int mid_end = 0;              // Below last row of the mid area that needs
                                // updating.  0 when no mid area updating.
  int bot_start = 999;          // first row of the bot area that needs
                                // updating.  999 when no bot area updating
  bool scrolled_down = false;   // true when scrolled down when w_topline got smaller a bit
  bool top_to_mod = false;      // redraw above mod_top

  int bot_scroll_start = 999;   // first line that needs to be redrawn due to
                                // scrolling. only used for EOB

  static bool recursive = false;  // being called recursively

  // Remember what happened to the previous line.
  enum {
    DID_NONE = 1,  // didn't update a line
    DID_LINE = 2,  // updated a normal line
    DID_FOLD = 3,  // updated a folded line
  } did_update = DID_NONE;

  linenr_T syntax_last_parsed = 0;              // last parsed text line
  linenr_T mod_top = 0;
  linenr_T mod_bot = 0;

  int type = wp->w_redr_type;

  if (type >= UPD_NOT_VALID) {
    wp->w_redr_status = true;
    wp->w_lines_valid = 0;
  }

  // Window is zero-height: Only need to draw the separator
  if (wp->w_view_height == 0) {
    // draw the horizontal separator below this window
    rs_draw_hsep_win(wp);
    rs_draw_sep_connectors_win(wp);
    wp->w_redr_type = 0;
    return;
  }

  // Window is zero-width: Only need to draw the separator.
  if (wp->w_view_width == 0) {
    // draw the vertical separator right of this window
    rs_draw_vsep_win(wp);
    rs_draw_sep_connectors_win(wp);
    wp->w_redr_type = 0;
    return;
  }

  buf_T *buf = wp->w_buffer;

  // reset got_int, otherwise regexp won't work
  int save_got_int = got_int;
  got_int = 0;
  // Set the time limit to 'redrawtime'.
  proftime_T syntax_tm = profile_setlimit(p_rdt);
  syn_set_timeout(&syntax_tm);

  win_extmark_arr.size = 0;

  decor_redraw_reset(wp, &decor_state);

  decor_providers_invoke_win(wp);

  FOR_ALL_WINDOWS_IN_TAB(win, curtab) {
    if (win->w_buffer == wp->w_buffer && win_redraw_signcols(win)) {
      changed_line_abv_curs_win(win);
      redraw_later(win, UPD_NOT_VALID);
    }
  }
  buf->b_signcols.last_max = buf->b_signcols.max;

  init_search_hl(wp, &screen_search_hl);

  // Make sure skipcol is valid, it depends on various options and the window
  // width.
  if (wp->w_skipcol > 0 && wp->w_view_width > win_col_off(wp)) {
    int w = 0;
    int width1 = wp->w_view_width - win_col_off(wp);
    int width2 = width1 + win_col_off2(wp);
    int add = width1;

    while (w < wp->w_skipcol) {
      if (w > 0) {
        add = width2;
      }
      w += add;
    }
    if (w != wp->w_skipcol) {
      // always round down, the higher value may not be valid
      wp->w_skipcol = w - add;
    }
  }

  const int nrwidth_before = wp->w_nrwidth;
  int nrwidth_new = (wp->w_p_nu || wp->w_p_rnu || *wp->w_p_stc) ? number_width(wp) : 0;
  // Force redraw when width of 'number' or 'relativenumber' column changes.
  if (wp->w_nrwidth != nrwidth_new) {
    type = UPD_NOT_VALID;
    changed_line_abv_curs_win(wp);
    wp->w_nrwidth = nrwidth_new;
  } else {
    // Set mod_top to the first line that needs displaying because of
    // changes.  Set mod_bot to the first line after the changes.
    mod_top = wp->w_redraw_top;
    if (wp->w_redraw_bot != 0) {
      mod_bot = wp->w_redraw_bot + 1;
    } else {
      mod_bot = 0;
    }
    if (buf->b_mod_set) {
      if (mod_top == 0 || mod_top > buf->b_mod_top) {
        mod_top = buf->b_mod_top;
        // Need to redraw lines above the change that may be included
        // in a pattern match.
        if (syntax_present(wp)) {
          mod_top -= buf->b_s.b_syn_sync_linebreaks;
          mod_top = MAX(mod_top, 1);
        }
      }
      if (mod_bot == 0 || mod_bot < buf->b_mod_bot) {
        mod_bot = buf->b_mod_bot;
      }

      // When 'hlsearch' is on and using a multi-line search pattern, a
      // change in one line may make the Search highlighting in a
      // previous line invalid.  Simple solution: redraw all visible
      // lines above the change.
      // Same for a match pattern.
      if (screen_search_hl.rm.regprog != NULL
          && re_multiline(screen_search_hl.rm.regprog)) {
        top_to_mod = true;
      } else {
        const matchitem_T *cur = wp->w_match_head;
        while (cur != NULL) {
          if (cur->mit_match.regprog != NULL
              && re_multiline(cur->mit_match.regprog)) {
            top_to_mod = true;
            break;
          }
          cur = cur->mit_next;
        }
      }
    }

    if (search_hl_has_cursor_lnum > 0) {
      // CurSearch was used last time, need to redraw the line with it to
      // avoid having two matches highlighted with CurSearch.
      if (mod_top == 0 || mod_top > search_hl_has_cursor_lnum) {
        mod_top = search_hl_has_cursor_lnum;
      }
      if (mod_bot == 0 || mod_bot < search_hl_has_cursor_lnum + 1) {
        mod_bot = search_hl_has_cursor_lnum + 1;
      }
    }

    if (mod_top != 0 && win_lines_concealed(wp)) {
      // A change in a line can cause lines above it to become folded or
      // unfolded.  Find the top most buffer line that may be affected.
      // If the line was previously folded and displayed, get the first
      // line of that fold.  If the line is folded now, get the first
      // folded line.  Use the minimum of these two.

      // Find last valid w_lines[] entry above mod_top.  Set lnumt to
      // the line below it.  If there is no valid entry, use w_topline.
      // Find the first valid w_lines[] entry below mod_bot.  Set lnumb
      // to this line.  If there is no valid entry, use MAXLNUM.
      linenr_T lnumt = wp->w_topline;
      linenr_T lnumb = MAXLNUM;
      for (int i = 0; i < wp->w_lines_valid; i++) {
        if (wp->w_lines[i].wl_valid) {
          if (wp->w_lines[i].wl_lastlnum < mod_top) {
            lnumt = wp->w_lines[i].wl_lastlnum + 1;
          }
          if (lnumb == MAXLNUM && wp->w_lines[i].wl_lnum >= mod_bot) {
            lnumb = wp->w_lines[i].wl_lnum;
            // When there is a fold column it might need updating
            // in the next line ("J" just above an open fold).
            if (compute_foldcolumn(wp, 0) > 0) {
              lnumb++;
            }
          }
        }
      }

      hasFolding(wp, mod_top, &mod_top, NULL);
      mod_top = MIN(mod_top, lnumt);

      // Now do the same for the bottom line (one above mod_bot).
      mod_bot--;
      hasFolding(wp, mod_bot, NULL, &mod_bot);
      mod_bot++;
      mod_bot = MAX(mod_bot, lnumb);
    }

    // When a change starts above w_topline and the end is below
    // w_topline, start redrawing at w_topline.
    // If the end of the change is above w_topline: do like no change was
    // made, but redraw the first line to find changes in syntax.
    if (mod_top != 0 && mod_top < wp->w_topline) {
      if (mod_bot > wp->w_topline) {
        mod_top = wp->w_topline;
      } else if (syntax_present(wp)) {
        top_end = 1;
      }
    }
  }

  wp->w_redraw_top = 0;  // reset for next time
  wp->w_redraw_bot = 0;
  search_hl_has_cursor_lnum = 0;

  // When only displaying the lines at the top, set top_end.  Used when
  // window has scrolled down for msg_scrolled.
  if (type == UPD_REDRAW_TOP) {
    int j = 0;
    for (int i = 0; i < wp->w_lines_valid; i++) {
      j += wp->w_lines[i].wl_size;
      if (j >= wp->w_upd_rows) {
        top_end = j;
        break;
      }
    }
    if (top_end == 0) {
      // not found (cannot happen?): redraw everything
      type = UPD_NOT_VALID;
    } else {
      // top area defined, the rest is UPD_VALID
      type = UPD_VALID;
    }
  }

  // Below logic compares wp->w_topline against wp->w_lines[0].wl_lnum,
  // which may point to a line below wp->w_topline if it is concealed;
  // incurring scrolling even though wp->w_topline is still the same.
  // Compare against an adjusted topline instead:
  linenr_T topline_conceal = wp->w_topline;
  while (topline_conceal < buf->b_ml.ml_line_count
         && decor_conceal_line(wp, topline_conceal - 1, false)) {
    topline_conceal++;
    hasFolding(wp, topline_conceal, NULL, &topline_conceal);
  }

  // If there are no changes on the screen that require a complete redraw,
  // handle three cases:
  // 1: we are off the top of the screen by a few lines: scroll down
  // 2: wp->w_topline is below wp->w_lines[0].wl_lnum: may scroll up
  // 3: wp->w_topline is wp->w_lines[0].wl_lnum: find first entry in
  //    w_lines[] that needs updating.
  if ((type == UPD_VALID || type == UPD_SOME_VALID
       || type == UPD_INVERTED || type == UPD_INVERTED_ALL)
      && !wp->w_botfill && !wp->w_old_botfill) {
    if (mod_top != 0
        && wp->w_topline == mod_top
        && (!wp->w_lines[0].wl_valid
            || topline_conceal == wp->w_lines[0].wl_lnum)) {
      // w_topline is the first changed line and window is not scrolled,
      // the scrolling from changed lines will be done further down.
    } else if (wp->w_lines[0].wl_valid
               && (topline_conceal < wp->w_lines[0].wl_lnum
                   || (topline_conceal == wp->w_lines[0].wl_lnum
                       && wp->w_topfill > wp->w_old_topfill))) {
      // New topline is above old topline: May scroll down.
      int j;
      if (win_lines_concealed(wp)) {
        // Count the number of lines we are off, counting a sequence
        // of folded lines as one, and skip concealed lines.
        j = 0;
        for (linenr_T ln = wp->w_topline; ln < wp->w_lines[0].wl_lnum; ln++) {
          j += !decor_conceal_line(wp, ln - 1, false);
          if (j >= wp->w_view_height - 2) {
            break;
          }
          hasFolding(wp, ln, NULL, &ln);
        }
      } else {
        j = wp->w_lines[0].wl_lnum - wp->w_topline;
      }
      if (j < wp->w_view_height - 2) {               // not too far off
        int i = plines_m_win(wp, wp->w_topline, wp->w_lines[0].wl_lnum - 1, wp->w_view_height);
        // insert extra lines for previously invisible filler lines
        if (wp->w_lines[0].wl_lnum != wp->w_topline) {
          i += win_get_fill(wp, wp->w_lines[0].wl_lnum) - wp->w_old_topfill;
        }
        if (i != 0 && i < wp->w_view_height - 2) {  // less than a screen off
          // Try to insert the correct number of lines.
          // If not the last window, delete the lines at the bottom.
          // win_ins_lines may fail when the terminal can't do it.
          win_scroll_lines(wp, 0, i);
          bot_scroll_start = 0;
          if (wp->w_lines_valid != 0) {
            // Need to update rows that are new, stop at the
            // first one that scrolled down.
            top_end = i;
            scrolled_down = true;

            // Move the entries that were scrolled, disable
            // the entries for the lines to be redrawn.
            if ((wp->w_lines_valid += (linenr_T)j) > wp->w_view_height) {
              wp->w_lines_valid = wp->w_view_height;
            }
            int idx;
            for (idx = wp->w_lines_valid; idx - j >= 0; idx--) {
              wp->w_lines[idx] = wp->w_lines[idx - j];
            }
            while (idx >= 0) {
              wp->w_lines[idx--].wl_valid = false;
            }
          }
        } else {
          mid_start = 0;  // redraw all lines
        }
      } else {
        mid_start = 0;  // redraw all lines
      }
    } else {
      // New topline is at or below old topline: May scroll up.
      // When topline didn't change, find first entry in w_lines[] that
      // needs updating.

      // try to find wp->w_topline in wp->w_lines[].wl_lnum
      int j = -1;
      int row = 0;
      for (int i = 0; i < wp->w_lines_valid; i++) {
        if (wp->w_lines[i].wl_valid
            && wp->w_lines[i].wl_lnum == wp->w_topline) {
          j = i;
          break;
        }
        row += wp->w_lines[i].wl_size;
      }
      if (j == -1) {
        // if wp->w_topline is not in wp->w_lines[].wl_lnum redraw all
        // lines
        mid_start = 0;
      } else {
        // Try to delete the correct number of lines.
        // wp->w_topline is at wp->w_lines[i].wl_lnum.

        // If the topline didn't change, delete old filler lines,
        // otherwise delete filler lines of the new topline...
        if (wp->w_lines[0].wl_lnum == wp->w_topline) {
          row += wp->w_old_topfill;
        } else {
          row += win_get_fill(wp, wp->w_topline);
        }
        // ... but don't delete new filler lines.
        row -= wp->w_topfill;
        if (row > 0) {
          win_scroll_lines(wp, 0, -row);
          bot_start = wp->w_view_height - row;
          bot_scroll_start = bot_start;
        }
        if ((row == 0 || bot_start < 999) && wp->w_lines_valid != 0) {
          // Skip the lines (below the deleted lines) that are still
          // valid and don't need redrawing.    Copy their info
          // upwards, to compensate for the deleted lines.  Set
          // bot_start to the first row that needs redrawing.
          bot_start = 0;
          int idx = 0;
          while (true) {
            wp->w_lines[idx] = wp->w_lines[j];
            // stop at line that didn't fit, unless it is still
            // valid (no lines deleted)
            if (row > 0 && bot_start + row
                + (int)wp->w_lines[j].wl_size > wp->w_view_height) {
              wp->w_lines_valid = idx + 1;
              break;
            }
            bot_start += wp->w_lines[idx++].wl_size;

            // stop at the last valid entry in w_lines[].wl_size
            if (++j >= wp->w_lines_valid) {
              wp->w_lines_valid = idx;
              break;
            }
          }

          // Correct the first entry for filler lines at the top
          // when it won't get updated below.
          if (win_may_fill(wp) && bot_start > 0) {
            wp->w_lines[0].wl_size
              = (uint16_t)plines_correct_topline(wp, wp->w_topline, NULL, true, NULL);
          }
        }
      }
    }

    // When starting redraw in the first line, redraw all lines.
    if (mid_start == 0) {
      mid_end = wp->w_view_height;
    }
  } else {
    // Not UPD_VALID or UPD_INVERTED: redraw all lines.
    mid_start = 0;
    mid_end = wp->w_view_height;
  }

  if (type == UPD_SOME_VALID) {
    // UPD_SOME_VALID: redraw all lines.
    mid_start = 0;
    mid_end = wp->w_view_height;
    type = UPD_NOT_VALID;
  }

  // check if we are updating or removing the inverted part
  rs_win_update_visual_region(wp, buf, type, top_end, scrolled_down,
                              &mid_start, &mid_end);

  foldinfo_T cursorline_fi = { 0 };
  win_update_cursorline(wp, &cursorline_fi);
  if (wp == curwin) {
    conceal_cursor_used = conceal_cursor_line(curwin);
  }

  win_check_ns_hl(wp);

  spellvars_T spv = { 0 };
  linenr_T lnum = wp->w_topline;  // first line shown in window
  // Initialize spell related variables for the first drawn line.
  if (spell_check_window(wp)) {
    spv.spv_has_spell = true;
    spv.spv_unchanged = mod_top == 0;
  }

  // Update all the window rows.
  int idx = 0;                    // first entry in w_lines[].wl_size
  int row = 0;                    // current window row to display
  int srow = 0;                   // starting row of the current line

  bool eof = false;             // if true, we hit the end of the file
  bool didline = false;         // if true, we finished the last line
  while (true) {
    // stop updating when reached the end of the window (check for _past_
    // the end of the window is at the end of the loop)
    if (row == wp->w_view_height) {
      didline = true;
      break;
    }

    // stop updating when hit the end of the file
    if (lnum > buf->b_ml.ml_line_count) {
      eof = true;
      break;
    }

    // Remember the starting row of the line that is going to be dealt
    // with.  It is used further down when the line doesn't fit.
    srow = row;

    // Update a line when it is in an area that needs updating, when it
    // has changes or w_lines[idx] is invalid.
    // "bot_start" may be halfway a wrapped line after using
    // win_scroll_lines(), check if the current line includes it.
    // When syntax folding is being used, the saved syntax states will
    // already have been updated, we can't see where the syntax state is
    // the same again, just update until the end of the window.
    if (row < top_end
        || (row >= mid_start && row < mid_end)
        || top_to_mod
        || idx >= wp->w_lines_valid
        || (row + wp->w_lines[idx].wl_size > bot_start)
        || (mod_top != 0
            && (lnum == mod_top
                || (lnum >= mod_top
                    && (lnum < mod_bot
                        || did_update == DID_FOLD
                        || (did_update == DID_LINE
                            && syntax_present(wp)
                            && ((rs_foldmethodIsSyntax(wp)
                                 && rs_hasAnyFolding(wp))
                                || syntax_check_changed(lnum)))
                        // match in fixed position might need redraw
                        // if lines were inserted or deleted
                        || (wp->w_match_head != NULL
                            && buf->b_mod_set && buf->b_mod_xlines != 0)))))
        || lnum == wp->w_cursorline
        || lnum == wp->w_last_cursorline) {
      if (lnum == mod_top) {
        top_to_mod = false;
      }

      // When lines are folded, display one line for all of them.
      // Otherwise, display normally (can be several display lines when
      // 'wrap' is on).
      foldinfo_T foldinfo = wp->w_p_cul && lnum == wp->w_cursor.lnum
                            ? cursorline_fi : rs_fold_info(wp, lnum);

      // If the line is concealed and has no filler lines, go to the next line.
      bool concealed = decor_conceal_line(wp, lnum - 1, false);
      if (concealed && win_get_fill(wp, lnum) == 0) {
        if (lnum == mod_top && lnum < mod_bot) {
          mod_top += foldinfo.fi_lines ? foldinfo.fi_lines : 1;
        }
        lnum += foldinfo.fi_lines ? foldinfo.fi_lines : 1;
        spv.spv_capcol_lnum = 0;
        continue;
      }

      // When at start of changed lines: May scroll following lines
      // up or down to minimize redrawing.
      // Don't do this when the change continues until the end.
      // Don't scroll when redrawing the top, scrolled already above.
      if (lnum == mod_top
          && mod_bot != MAXLNUM
          && row >= top_end) {
        int old_cline_height = 0;
        int old_rows = 0;
        linenr_T l;
        int i;

        // Count the old number of window rows, using w_lines[], which
        // should still contain the sizes for the lines as they are
        // currently displayed.
        for (i = idx; i < wp->w_lines_valid; i++) {
          // Only valid lines have a meaningful wl_lnum.  Invalid
          // lines are part of the changed area.
          if (wp->w_lines[i].wl_valid
              && wp->w_lines[i].wl_lnum == mod_bot) {
            break;
          }
          if (wp->w_lines[i].wl_lnum == wp->w_cursor.lnum) {
            old_cline_height = wp->w_lines[i].wl_size;
          }
          old_rows += wp->w_lines[i].wl_size;
          if (wp->w_lines[i].wl_valid
              && wp->w_lines[i].wl_lastlnum + 1 == mod_bot) {
            // Must have found the last valid entry above mod_bot.
            // Add following invalid entries.
            i++;
            while (i < wp->w_lines_valid
                   && !wp->w_lines[i].wl_valid) {
              old_rows += wp->w_lines[i++].wl_size;
            }
            break;
          }
        }

        if (i >= wp->w_lines_valid) {
          // We can't find a valid line below the changed lines,
          // need to redraw until the end of the window.
          // Inserting/deleting lines has no use.
          bot_start = 0;
          bot_scroll_start = 0;
        } else {
          int new_rows = 0;
          // Able to count old number of rows: Count new window
          // rows, and may insert/delete lines
          int j = idx;
          for (l = lnum; l < mod_bot; l++) {
            if (dollar_vcol >= 0 && wp == curwin
                && old_cline_height > 0 && l == wp->w_cursor.lnum) {
              // When dollar_vcol >= 0, cursor line isn't fully
              // redrawn, and its height remains unchanged.
              new_rows += old_cline_height;
              j++;
            } else {
              int n = plines_correct_topline(wp, l, &l, true, NULL);
              new_rows += n;
              j += n > 0;  // don't count concealed lines
            }
            if (new_rows > wp->w_view_height - row - 2) {
              // it's getting too much, must redraw the rest
              new_rows = 9999;
              break;
            }
          }
          int xtra_rows = new_rows - old_rows;
          if (xtra_rows < 0) {
            // May scroll text up.  If there is not enough
            // remaining text or scrolling fails, must redraw the
            // rest.  If scrolling works, must redraw the text
            // below the scrolled text.
            if (row - xtra_rows >= wp->w_view_height - 2) {
              mod_bot = MAXLNUM;
            } else {
              win_scroll_lines(wp, row, xtra_rows);
              bot_start = wp->w_view_height + xtra_rows;
              bot_scroll_start = bot_start;
            }
          } else if (xtra_rows > 0) {
            // May scroll text down.  If there is not enough
            // remaining text of scrolling fails, must redraw the
            // rest.
            if (row + xtra_rows >= wp->w_view_height - 2) {
              mod_bot = MAXLNUM;
            } else {
              win_scroll_lines(wp, row + old_rows, xtra_rows);
              bot_scroll_start = 0;
              if (top_end > row + old_rows) {
                // Scrolled the part at the top that requires
                // updating down.
                top_end += xtra_rows;
              }
            }
          }

          // When not updating the rest, may need to move w_lines[]
          // entries.
          if (mod_bot != MAXLNUM && i != j) {
            if (j < i) {
              int x = row + new_rows;

              // move entries in w_lines[] upwards
              while (true) {
                // stop at last valid entry in w_lines[]
                if (i >= wp->w_lines_valid) {
                  wp->w_lines_valid = j;
                  break;
                }
                wp->w_lines[j] = wp->w_lines[i];
                // stop at a line that won't fit
                if (x + (int)wp->w_lines[j].wl_size
                    > wp->w_view_height) {
                  wp->w_lines_valid = j + 1;
                  break;
                }
                x += wp->w_lines[j++].wl_size;
                i++;
              }
              bot_start = MIN(bot_start, x);
            } else {       // j > i
                           // move entries in w_lines[] downwards
              j -= i;
              wp->w_lines_valid += (linenr_T)j;
              wp->w_lines_valid = MIN(wp->w_lines_valid, wp->w_view_height);
              for (i = wp->w_lines_valid; i - j >= idx; i--) {
                wp->w_lines[i] = wp->w_lines[i - j];
              }

              // The w_lines[] entries for inserted lines are
              // now invalid, but wl_size may be used above.
              // Reset to zero.
              while (i >= idx) {
                wp->w_lines[i].wl_size = 0;
                wp->w_lines[i--].wl_valid = false;
              }
            }
          }
        }
      }

      if (foldinfo.fi_lines == 0
          && idx < wp->w_lines_valid
          && wp->w_lines[idx].wl_valid
          && wp->w_lines[idx].wl_lnum == lnum
          && lnum > wp->w_topline
          && !(dy_flags & (kOptDyFlagLastline | kOptDyFlagTruncate))
          && srow + wp->w_lines[idx].wl_size > wp->w_view_height
          && win_get_fill(wp, lnum) == 0) {
        // This line is not going to fit.  Don't draw anything here,
        // will draw "@  " lines below.
        row = wp->w_view_height + 1;
      } else {
        prepare_search_hl(wp, &screen_search_hl, lnum);
        // Let the syntax stuff know we skipped a few lines.
        if (syntax_last_parsed != 0 && syntax_last_parsed + 1 < lnum
            && syntax_present(wp)) {
          syntax_end_parsing(wp, syntax_last_parsed + 1);
        }

        bool display_buf_line = !concealed && (foldinfo.fi_lines == 0 || *wp->w_p_fdt == NUL);

        // Display one line
        spellvars_T zero_spv = { 0 };
        row = win_line(wp, lnum, srow, wp->w_view_height, 0, concealed,
                       display_buf_line ? &spv : &zero_spv, foldinfo);

        if (display_buf_line) {
          syntax_last_parsed = lnum;
        } else {
          spv.spv_capcol_lnum = 0;
        }

        linenr_T lastlnum = lnum + foldinfo.fi_lines - (foldinfo.fi_lines > 0);
        wp->w_lines[idx].wl_folded = foldinfo.fi_lines > 0;
        wp->w_lines[idx].wl_foldend = lastlnum;
        wp->w_lines[idx].wl_lastlnum = lastlnum;
        did_update = foldinfo.fi_lines > 0 ? DID_FOLD : DID_LINE;

        // Adjust "wl_lastlnum" for concealed lines below this line, unless it should
        // still be drawn for below virt_lines attached to the current line. Below
        // virt_lines attached to a second adjacent concealed line are concealed.
        bool virt_below = decor_virt_lines(wp, lastlnum, lastlnum + 1, NULL, NULL, true) > 0;
        while (!virt_below && wp->w_lines[idx].wl_lastlnum < buf->b_ml.ml_line_count
               && decor_conceal_line(wp, wp->w_lines[idx].wl_lastlnum, false)) {
          virt_below = false;
          wp->w_lines[idx].wl_lastlnum++;
          hasFolding(wp, wp->w_lines[idx].wl_lastlnum, NULL, &wp->w_lines[idx].wl_lastlnum);
        }
      }

      wp->w_lines[idx].wl_lnum = lnum;
      wp->w_lines[idx].wl_valid = true;

      bool is_curline = wp == curwin && lnum == wp->w_cursor.lnum;

      if (row > wp->w_view_height) {         // past end of grid
        // we may need the size of that too long line later on
        if (dollar_vcol == -1 || !is_curline) {
          wp->w_lines[idx].wl_size = (uint16_t)plines_win(wp, lnum, true);
        }
        idx++;
        break;
      }
      if (dollar_vcol == -1 || !is_curline) {
        wp->w_lines[idx].wl_size = (uint16_t)(row - srow);
      }
      lnum = wp->w_lines[idx++].wl_lastlnum + 1;
    } else {
      // If:
      // - 'number' is set and below inserted/deleted lines, or
      // - 'relativenumber' is set and cursor moved vertically,
      // the text doesn't need to be redrawn, but the number column does.
      if ((wp->w_p_nu && mod_top != 0 && lnum >= mod_bot
           && buf->b_mod_set && buf->b_mod_xlines != 0)
          || (wp->w_p_rnu && wp->w_last_cursor_lnum_rnu != wp->w_cursor.lnum)) {
        foldinfo_T info = wp->w_p_cul && lnum == wp->w_cursor.lnum
                          ? cursorline_fi : rs_fold_info(wp, lnum);
        win_line(wp, lnum, srow, wp->w_view_height, wp->w_lines[idx].wl_size, false, &spv, info);
      }

      // This line does not need to be drawn, advance to the next one.
      row += wp->w_lines[idx++].wl_size;
      if (row > wp->w_view_height) {  // past end of screen
        break;
      }
      lnum = wp->w_lines[idx - 1].wl_lastlnum + 1;
      did_update = DID_NONE;
      spv.spv_capcol_lnum = 0;
    }

    // 'statuscolumn' width has changed or errored, start from the top.
    if (wp->w_redr_statuscol) {
redr_statuscol:
      wp->w_redr_statuscol = false;
      idx = 0;
      row = 0;
      lnum = wp->w_topline;
      wp->w_lines_valid = 0;
      wp->w_valid &= ~VALID_WCOL;
      decor_redraw_reset(wp, &decor_state);
      decor_providers_invoke_win(wp);
      continue;
    }

    if (lnum > buf->b_ml.ml_line_count) {
      eof = true;
      break;
    }
  }
  // End of loop over all window lines.

  // Now that the window has been redrawn with the old and new cursor line,
  // update w_last_cursorline.
  wp->w_last_cursorline = wp->w_cursorline;

  wp->w_last_cursor_lnum_rnu = wp->w_p_rnu ? wp->w_cursor.lnum : 0;

  wp->w_lines_valid = MAX(wp->w_lines_valid, idx);

  // Let the syntax stuff know we stop parsing here.
  if (syntax_last_parsed != 0 && syntax_present(wp)) {
    syntax_end_parsing(wp, syntax_last_parsed + 1);
  }

  const linenr_T old_botline = wp->w_botline;

  // If we didn't hit the end of the file, and we didn't finish the last
  // line we were working on, then the line didn't fit.
  wp->w_empty_rows = 0;
  wp->w_filler_rows = 0;
  if (!eof && !didline) {
    int at_attr = hl_combine_attr(win_bg_attr(wp), win_hl_attr(wp, HLF_AT));
    if (lnum == wp->w_topline) {
      // Single line that does not fit!
      // Don't overwrite it, it can be edited.
      wp->w_botline = lnum + 1;
    } else if (win_get_fill(wp, lnum) >= wp->w_view_height - srow) {
      // Window ends in filler lines.
      wp->w_botline = lnum;
      wp->w_filler_rows = wp->w_view_height - srow;
    } else if (dy_flags & kOptDyFlagTruncate) {      // 'display' has "truncate"
      // Last line isn't finished: Display "@@@" in the last screen line.
      grid_line_start(&wp->w_grid, wp->w_view_height - 1);
      grid_line_fill(0, MIN(wp->w_view_width, 3), wp->w_p_fcs_chars.lastline, at_attr);
      grid_line_fill(3, wp->w_view_width, schar_from_ascii(' '), at_attr);
      grid_line_flush();
      set_empty_rows(wp, srow);
      wp->w_botline = lnum;
    } else if (dy_flags & kOptDyFlagLastline) {      // 'display' has "lastline"
      // Last line isn't finished: Display "@@@" at the end.
      // If this would split a doublewidth char in two, we need to display "@@@@" instead
      grid_line_start(&wp->w_grid, wp->w_view_height - 1);
      int width = grid_line_getchar(MAX(wp->w_view_width - 3, 0), NULL) == NUL ? 4 : 3;
      grid_line_fill(MAX(wp->w_view_width - width, 0), wp->w_view_width,
                     wp->w_p_fcs_chars.lastline, at_attr);
      grid_line_flush();
      set_empty_rows(wp, srow);
      wp->w_botline = lnum;
    } else {
      win_draw_end(wp, wp->w_p_fcs_chars.lastline, true, srow,
                   wp->w_view_height, HLF_AT);
      set_empty_rows(wp, srow);
      wp->w_botline = lnum;
    }
  } else {
    if (eof) {  // we hit the end of the file
      wp->w_botline = buf->b_ml.ml_line_count + 1;
      int j = win_get_fill(wp, wp->w_botline);
      if (j > 0 && !wp->w_botfill && row < wp->w_view_height) {
        // Display filler text below last line. win_line() will check
        // for ml_line_count+1 and only draw filler lines
        spellvars_T zero_spv = { 0 };
        foldinfo_T zero_foldinfo = { 0 };
        row = win_line(wp, wp->w_botline, row, wp->w_view_height, 0, false, &zero_spv,
                       zero_foldinfo);
        if (wp->w_redr_statuscol) {
          eof = false;
          goto redr_statuscol;
        }
      }
    } else if (dollar_vcol == -1 || wp != curwin) {
      wp->w_botline = lnum;
    }

    // Make sure the rest of the screen is blank.
    // write the "eob" character from 'fillchars' to rows that aren't part
    // of the file.
    // TODO(bfredl): just keep track of the valid EOB area from last redraw?
    int lastline = bot_scroll_start;
    if (mid_end >= row) {
      lastline = MIN(lastline, mid_start);
    }
    // if (mod_bot > buf->b_ml.ml_line_count + 1) {
    if (mod_bot > buf->b_ml.ml_line_count) {
      lastline = 0;
    }

    win_draw_end(wp, wp->w_p_fcs_chars.eob, false, MAX(lastline, row),
                 wp->w_view_height,
                 HLF_EOB);
    set_empty_rows(wp, row);
  }

  if (wp->w_redr_type >= UPD_REDRAW_TOP) {
    rs_draw_vsep_win(wp);
    rs_draw_hsep_win(wp);
    rs_draw_sep_connectors_win(wp);
  }
  syn_set_timeout(NULL);

  // Reset the type of redrawing required, the window has been updated.
  wp->w_redr_type = 0;
  wp->w_old_topfill = wp->w_topfill;
  wp->w_old_botfill = wp->w_botfill;

  // Send win_extmarks if needed
  for (size_t n = 0; n < kv_size(win_extmark_arr); n++) {
    ui_call_win_extmark(wp->w_grid_alloc.handle, wp->handle,
                        kv_A(win_extmark_arr, n).ns_id, (Integer)kv_A(win_extmark_arr, n).mark_id,
                        kv_A(win_extmark_arr, n).win_row, kv_A(win_extmark_arr, n).win_col);
  }

  if (dollar_vcol == -1 || wp != curwin) {
    // There is a trick with w_botline.  If we invalidate it on each
    // change that might modify it, this will cause a lot of expensive
    // calls to plines_win() in update_topline() each time.  Therefore the
    // value of w_botline is often approximated, and this value is used to
    // compute the value of w_topline.  If the value of w_botline was
    // wrong, check that the value of w_topline is correct (cursor is on
    // the visible part of the text).  If it's not, we need to redraw
    // again.  Mostly this just means scrolling up a few lines, so it
    // doesn't look too bad.  Only do this for the current window (where
    // changes are relevant).
    wp->w_valid |= VALID_BOTLINE;
    wp->w_viewport_invalid = true;
    if (wp == curwin && wp->w_botline != old_botline && !recursive) {
      recursive = true;
      curwin->w_valid &= ~VALID_TOPLINE;
      update_topline(curwin);  // may invalidate w_botline again
      // New redraw either due to updated topline or reset skipcol.
      if (must_redraw != 0) {
        // Don't update for changes in buffer again.
        int mod_set = curbuf->b_mod_set;
        curbuf->b_mod_set = false;
        curs_columns(curwin, true);
        win_update(curwin);
        must_redraw = 0;
        curbuf->b_mod_set = mod_set;
      }
      recursive = false;
    }
  }

  if (nrwidth_before != wp->w_nrwidth && buf->terminal) {
    terminal_check_size(buf->terminal);
  }

  // restore got_int, unless CTRL-C was hit while redrawing
  if (!got_int) {
    got_int = save_got_int;
  }
}

/// Handle the three window iteration loops of update_screen.
/// Defined here (not in drawscreen_shim.c) because win_update() is static.
/// Called from rs_update_screen() in Rust.
void nvim_update_screen_win_loop(int type, int hl_changed)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    update_window_hl(wp, type >= UPD_NOT_VALID || hl_changed);

    buf_T *buf = wp->w_buffer;
    if (buf->b_mod_set) {
      if (buf->b_mod_tick_syn < display_tick && syntax_present(wp)) {
        syn_stack_apply_changes(buf);
        buf->b_mod_tick_syn = display_tick;
      }

      if (buf->b_mod_tick_decor < display_tick) {
        decor_providers_invoke_buf(buf);
        buf->b_mod_tick_decor = display_tick;
      }
    }
  }

  screen_search_hl.rm.regprog = NULL;
  bool did_one = false;

  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_redr_type == UPD_CLEAR && wp->w_floating && wp->w_grid_alloc.chars) {
      grid_invalidate(&wp->w_grid_alloc);
      wp->w_redr_type = UPD_NOT_VALID;
    }

    win_check_ns_hl(wp);
    win_grid_alloc(wp);

    if (wp->w_redr_border || wp->w_redr_type >= UPD_NOT_VALID) {
      grid_draw_border(&wp->w_grid_alloc, &wp->w_config, wp->w_border_adj, (int)wp->w_p_winbl,
                       wp->w_ns_hl_attr);
    }

    if (wp->w_redr_type != 0) {
      if (!did_one) {
        did_one = true;
        start_search_hl();
      }
      win_update(wp);
    }

    if (wp->w_redr_status) {
      win_redr_winbar(wp);
      win_redr_status(wp);
    }
  }

  end_search_hl();

  if (pum_drawn() && must_redraw_pum) {
    win_check_ns_hl(curwin);
    pum_redraw();
  }

  win_check_ns_hl(NULL);

  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    wp->w_buffer->b_mod_set = false;
  }
}

/// Scroll `line_count` lines at 'row' in window 'wp'.
///
/// Positive `line_count` means scrolling down, so that more space is available
/// at 'row'. Negative `line_count` implies deleting lines at `row`.
void win_scroll_lines(win_T *wp, int row, int line_count) { rs_win_scroll_lines(wp, row, line_count); }

_Static_assert(HLF_FC == 29, "HLF_FC must be 29");
_Static_assert(HLF_SC == 35, "HLF_SC must be 35");
_Static_assert(HLF_N == 12, "HLF_N must be 12");
_Static_assert(HLF_CM == 11, "HLF_CM must be 11");
_Static_assert(COL_RULER == 17, "COL_RULER must be 17");
_Static_assert(SHOWCMD_COLS == 10, "SHOWCMD_COLS must be 10");

/// Get conceal_cursor_used (file-static).
int nvim_get_conceal_cursor_used(void) { return conceal_cursor_used ? 1 : 0; }

/// Set conceal_cursor_used (file-static).
void nvim_set_conceal_cursor_used(int val) { conceal_cursor_used = (val != 0); }

/// Check if screen_search_hl has a regprog.
int nvim_search_hl_has_regprog(void) { return screen_search_hl.rm.regprog != NULL ? 1 : 0; }

/// Prepare search highlight: set regprog and time limit.
/// Keeps regexp lifetime management in C.
void nvim_search_hl_start(void)
{
  last_pat_prog(&screen_search_hl.rm);
  screen_search_hl.tm = profile_setlimit(p_rdt);
}

/// Free search highlight regprog. Keeps regexp lifetime management in C.
void nvim_search_hl_end(void)
{
  vim_regfree(screen_search_hl.rm.regprog);
  screen_search_hl.rm.regprog = NULL;
}
