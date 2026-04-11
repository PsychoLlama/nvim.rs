// textobject.c: functions for text objects

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/eval/funcs.h"
#include "nvim/fold.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/option_vars.h"
#include "nvim/pos_defs.h"
#include "nvim/search.h"
#include "nvim/strings.h"
#include "nvim/textobject.h"
#include "nvim/vim_defs.h"

#include "textobject.c.generated.h"

// Rust function declarations

extern bool in_html_tag(bool end_tag);

/// @param end_tag  when true, return true if the cursor is on "</aaa>".
///
/// Find tag block under the cursor, cursor at end.
///
/// @param include  true == include white space
int current_tagblock(oparg_T *oap, int count_arg, bool include)
{
  int count = count_arg;
  char *cp;
  bool do_include = include;
  bool save_p_ws = p_ws;
  int retval = FAIL;
  bool is_inclusive = true;

  p_ws = false;

  pos_T old_pos = curwin->w_cursor;
  pos_T old_end = curwin->w_cursor;               // remember where we started
  pos_T old_start = old_end;
  if (!VIsual_active || *p_sel == 'e') {
    decl(&old_end);                         // old_end is inclusive
  }

  // If we start on "<aaa>" select that block.
  if (!VIsual_active || equalpos(VIsual, curwin->w_cursor)) {
    setpcmark();

    // ignore indent
    while (inindent(1)) {
      if (inc_cursor() != 0) {
        break;
      }
    }

    if (in_html_tag(false)) {
      // cursor on start tag, move to its '>'
      while (*get_cursor_pos_ptr() != '>') {
        if (inc_cursor() < 0) {
          break;
        }
      }
    } else if (in_html_tag(true)) {
      // cursor on end tag, move to just before it
      while (*get_cursor_pos_ptr() != '<') {
        if (dec_cursor() < 0) {
          break;
        }
      }
      dec_cursor();
      old_end = curwin->w_cursor;
    }
  } else if (lt(VIsual, curwin->w_cursor)) {
    old_start = VIsual;
    curwin->w_cursor = VIsual;              // cursor at low end of Visual
  } else {
    old_end = VIsual;
  }

again:
  // Search backwards for unclosed "<aaa>".
  // Put this position in start_pos.
  for (int n = 0; n < count; n++) {
    if (do_searchpair("<[^ \t>/!]\\+\\%(\\_s\\_[^>]\\{-}[^/]>\\|$\\|\\_s\\=>\\)",
                      "",
                      "</[^>]*>", BACKWARD, NULL, 0,
                      NULL, 0, 0) <= 0) {
      curwin->w_cursor = old_pos;
      goto theend;
    }
  }
  pos_T start_pos = curwin->w_cursor;

  // Search for matching "</aaa>".  First isolate the "aaa".
  inc_cursor();
  char *p = get_cursor_pos_ptr();
  for (cp = p;
       *cp != NUL && *cp != '>' && !ascii_iswhite(*cp);
       MB_PTR_ADV(cp)) {}
  int len = (int)(cp - p);
  if (len == 0) {
    curwin->w_cursor = old_pos;
    goto theend;
  }
  const size_t spat_len = (size_t)len + 39;
  char *const spat = xmalloc(spat_len);
  const size_t epat_len = (size_t)len + 9;
  char *const epat = xmalloc(epat_len);
  snprintf(spat, spat_len,
           "<%.*s\\>\\%%(\\_s\\_[^>]\\{-}\\_[^/]>\\|\\_s\\?>\\)\\c", len, p);
  snprintf(epat, epat_len, "</%.*s>\\c", len, p);

  const int r = do_searchpair(spat, "", epat, FORWARD, NULL, 0, NULL, 0, 0);

  xfree(spat);
  xfree(epat);

  if (r < 1 || lt(curwin->w_cursor, old_end)) {
    // Can't find other end or it's before the previous end.  Could be a
    // HTML tag that doesn't have a matching end.  Search backwards for
    // another starting tag.
    count = 1;
    curwin->w_cursor = start_pos;
    goto again;
  }

  if (do_include) {
    // Include up to the '>'.
    while (*get_cursor_pos_ptr() != '>') {
      if (inc_cursor() < 0) {
        break;
      }
    }
  } else {
    char *c = get_cursor_pos_ptr();
    // Exclude the '<' of the end tag.
    // If the closing tag is on new line, do not decrement cursor, but make
    // operation exclusive, so that the linefeed will be selected
    if (*c == '<' && !VIsual_active && curwin->w_cursor.col == 0) {
      // do not decrement cursor
      is_inclusive = false;
    } else if (*c == '<') {
      dec_cursor();
    }
  }
  pos_T end_pos = curwin->w_cursor;

  if (!do_include) {
    // Exclude the start tag,
    // but skip over '>' if it appears in quotes
    bool in_quotes = false;
    curwin->w_cursor = start_pos;
    while (inc_cursor() >= 0) {
      p = get_cursor_pos_ptr();
      if (*p == '>' && !in_quotes) {
        inc_cursor();
        start_pos = curwin->w_cursor;
        break;
      } else if (*p == '"' || *p == '\'') {
        in_quotes = !in_quotes;
      }
    }
    curwin->w_cursor = end_pos;

    // If we are in Visual mode and now have the same text as before set
    // "do_include" and try again.
    if (VIsual_active
        && equalpos(start_pos, old_start)
        && equalpos(end_pos, old_end)) {
      do_include = true;
      curwin->w_cursor = old_start;
      count = count_arg;
      goto again;
    }
  }

  if (VIsual_active) {
    // If the end is before the start there is no text between tags, select
    // the char under the cursor.
    if (lt(end_pos, start_pos)) {
      curwin->w_cursor = start_pos;
    } else if (*p_sel == 'e') {
      inc_cursor();
    }
    VIsual = start_pos;
    VIsual_mode = 'v';
    redraw_curbuf_later(UPD_INVERTED);  // update the inversion
    showmode();
  } else {
    oap->start = start_pos;
    oap->motion_type = kMTCharWise;
    if (lt(end_pos, start_pos)) {
      // End is before the start: there is no text between tags; operate
      // on an empty area.
      curwin->w_cursor = start_pos;
      oap->inclusive = false;
    } else {
      oap->inclusive = is_inclusive;
    }
  }
  retval = OK;

theend:
  p_ws = save_p_ws;
  return retval;
}
