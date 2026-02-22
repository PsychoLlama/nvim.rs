// vim: set fdm=marker fdl=1 fdc=3

// fold.c: code for folding

#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/extmark.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/buffer_updates.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/decoration.h"
#include "nvim/diff.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_session.h"
#include "nvim/extmark.h"
#include "nvim/extmark_defs.h"
#include "nvim/fold.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/ops.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/search.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/syntax.h"
#include "nvim/types_defs.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"

// local declarations. {{{1

// fold_T is defined in fold_defs.h

enum {
  FD_OPEN = 0,    // fold is open (nested ones can be closed)
  FD_CLOSED = 1,  // fold is closed
  FD_LEVEL = 2,   // depends on 'foldlevel' (nested folds too)
};

#define MAX_LEVEL       20      // maximum fold depth

// Define "fline_T", passed to get fold level for a line. {{{2
typedef struct {
  win_T *wp;              // window
  linenr_T lnum;                // current line number
  linenr_T off;                 // offset between lnum and real line number
  linenr_T lnum_save;           // line nr used by fold update recursion
  int lvl;                      // current level (-1 for undefined)
  int lvl_next;                 // level used for next line
  int start;                    // number of folds that are forced to start at
                                // this line.
  int end;                      // level of fold that is forced to end below
                                // this line
  int had_end;                  // level of fold that is forced to end above
                                // this line (copy of "end" of prev. line)
} fline_T;

// Flag is set when redrawing is needed.
static bool fold_changed;

// static functions {{{2

#include "fold_shim.c.generated.h"

// Rust FFI declarations (internal-only; fold method checks are in fold.h)
extern bool rs_diff_infold(win_T *wp, linenr_T lnum);
extern linenr_T rs_diff_lnum_win(linenr_T lnum, win_T *wp);
extern int rs_foldLevelWin(win_T *wp, linenr_T lnum);
extern void rs_foldUpdateIEMS(win_T *wp, linenr_T top, linenr_T bot);
extern foldinfo_T rs_fold_info(win_T *win, linenr_T lnum);

// Struct returned by rs_hasFoldingWin
typedef struct {
  int has_folding;
  linenr_T first;
  linenr_T last;
  int fi_level;
  linenr_T fi_lnum;
  int fi_low_level;
} FoldingResult;

extern FoldingResult rs_hasFoldingWin(win_T *win, linenr_T lnum, bool cache);

static const char *e_nofold = N_("E490: No fold found");

// While updating the folds lines between invalid_top and invalid_bot have an
// undefined fold level.  Only used for the window currently being updated.
static linenr_T invalid_top = 0;
static linenr_T invalid_bot = 0;

// When using 'foldexpr' we sometimes get the level of the next line, which
// calls foldlevel() to get the level of the current line, which hasn't been
// stored yet.  To get around this chicken-egg problem the level of the
// previous line is stored here when available.  prev_lnum is zero when the
// level is not available.
static linenr_T prev_lnum = 0;
static int prev_lnum_lvl = -1;

static size_t foldstartmarkerlen;
static char *foldendmarker;
static size_t foldendmarkerlen;

// Exported folding functions. {{{1

// hasFolding() {{{2
/// When returning true, *firstp and *lastp are set to the first and last
/// lnum of the sequence of folded lines (skipped when NULL).
///
/// @return  true if line "lnum" in window "win" is part of a closed fold.
bool hasFolding(win_T *win, linenr_T lnum, linenr_T *firstp, linenr_T *lastp)
{
  return hasFoldingWin(win, lnum, firstp, lastp, true, NULL);
}

/// Wrapper for hasFolding for Rust FFI.
int nvim_hasFolding(win_T *wp, linenr_T lnum, linenr_T *firstp, linenr_T *lastp)
{
  return hasFolding(wp, lnum, firstp, lastp) ? 1 : 0;
}

// hasFoldingWin() {{{2
/// Search folds starting at lnum
/// @param lnum first line to search
/// @param[out] first first line of fold containing lnum
/// @param[out] lastp last line with a fold
/// @param cache when true: use cached values of window
/// @param[out] infop where to store fold info
///
/// @return true if range contains folds
bool hasFoldingWin(win_T *const win, const linenr_T lnum, linenr_T *const firstp,
                   linenr_T *const lastp, const bool cache, foldinfo_T *const infop)
{
  FoldingResult result = rs_hasFoldingWin(win, lnum, cache);

  if (infop != NULL) {
    infop->fi_level = result.fi_level;
    infop->fi_lnum = result.fi_lnum;
    infop->fi_low_level = result.fi_low_level;
  }

  if (result.has_folding) {
    if (lastp != NULL) {
      *lastp = result.last;
    }
    if (firstp != NULL) {
      *firstp = result.first;
    }
    return true;
  }

  return false;
}

/// Wrapper for lineFolded for Rust FFI.
int nvim_lineFolded(win_T *wp, linenr_T lnum)
{
  return rs_lineFolded(wp, lnum);
}

// foldUpdate() {{{2
/// Update folds for changes in the buffer of a window.
/// Note that inserted/deleted lines must have already been taken care of by
/// calling foldMarkAdjust().
/// The changes in lines from top to bot (inclusive).
void foldUpdate(win_T *wp, linenr_T top, linenr_T bot)
{
  if (disable_fold_update || (State & MODE_INSERT && !rs_foldmethodIsIndent(wp))) {
    return;
  }

  if (need_diff_redraw) {
    // will update later
    return;
  }

  if (wp->w_folds.ga_len > 0) {
    // Mark all folds from top to bot (or bot to top) as maybe-small.
    linenr_T maybe_small_start = MIN(top, bot);
    linenr_T maybe_small_end = MAX(top, bot);

    fold_T *fp;
    foldFind(&wp->w_folds, maybe_small_start, &fp);
    while (fp < (fold_T *)wp->w_folds.ga_data + wp->w_folds.ga_len
           && fp->fd_top <= maybe_small_end) {
      fp->fd_small = kNone;
      fp++;
    }
  }

  if (rs_foldmethodIsIndent(wp) || rs_foldmethodIsDiff(wp)
      || rs_foldmethodIsExpr(wp) || rs_foldmethodIsMarker(wp)
      || rs_foldmethodIsSyntax(wp)) {
    int save_got_int = got_int;
    got_int = false;
    rs_foldUpdateIEMS(wp, top, bot);
    got_int |= save_got_int;
  }
}

// C accessor for foldUpdateAll (for Rust to call from foldUpdateAfterInsert)
void nvim_foldUpdateAll_c(win_T *win)
{
  win->w_foldinvalid = true;
  redraw_later(win, UPD_NOT_VALID);
}

// Internal functions for "fold_T" {{{1

// foldFind() {{{2
/// Search for line "lnum" in folds of growarray "gap".
/// Set "*fpp" to the fold struct for the fold that contains "lnum" or
/// the first fold below it (careful: it can be beyond the end of the array!).
///
/// @return  false when there is no fold that contains "lnum".
static bool foldFind(const garray_T *gap, linenr_T lnum, fold_T **fpp)
{
  if (gap->ga_len == 0) {
    *fpp = NULL;
    return false;
  }

  // Perform a binary search.
  // "low" is lowest index of possible match.
  // "high" is highest index of possible match.
  fold_T *fp = (fold_T *)gap->ga_data;
  linenr_T low = 0;
  linenr_T high = gap->ga_len - 1;
  while (low <= high) {
    linenr_T i = (low + high) / 2;
    if (fp[i].fd_top > lnum) {
      // fold below lnum, adjust high
      high = i - 1;
    } else if (fp[i].fd_top + fp[i].fd_len <= lnum) {
      // fold above lnum, adjust low
      low = i + 1;
    } else {
      // lnum is inside this fold
      *fpp = fp + i;
      return true;
    }
  }
  *fpp = fp + low;
  return false;
}

// deleteFoldRecurse() {{{2
/// Delete nested folds in a fold.
void deleteFoldRecurse(buf_T *bp, garray_T *gap)
{
#define DELETE_FOLD_NESTED(fd) deleteFoldRecurse(bp, &((fd)->fd_nested))
  GA_DEEP_CLEAR(gap, fold_T, DELETE_FOLD_NESTED);
}

// foldCreateMarkers() {{{2
/// Create a fold from line "start" to line "end" (inclusive) in window `wp`
/// by adding markers.
static void foldCreateMarkers(win_T *wp, pos_T start, pos_T end)
{
  buf_T *buf = wp->w_buffer;
  if (!MODIFIABLE(buf)) {
    emsg(_(e_modifiable));
    return;
  }
  parseMarker(wp);

  foldAddMarker(buf, start, wp->w_p_fmr, foldstartmarkerlen);
  foldAddMarker(buf, end, foldendmarker, foldendmarkerlen);

  // Update both changes here, to avoid all folds after the start are
  // changed when the start marker is inserted and the end isn't.
  changed_lines(buf, start.lnum, 0, end.lnum, 0, false);

  // Note: foldAddMarker() may not actually change start and/or end if
  // u_save() is unable to save the buffer line, but we send the
  // nvim_buf_lines_event anyway since it won't do any harm.
  int64_t num_changed = 1 + end.lnum - start.lnum;
  buf_updates_send_changes(buf, start.lnum, num_changed, num_changed);
}

// foldAddMarker() {{{2
/// Add "marker[markerlen]" in 'commentstring' to position `pos`.
static void foldAddMarker(buf_T *buf, pos_T pos, const char *marker, size_t markerlen)
{
  char *cms = buf->b_p_cms;
  char *p = strstr(buf->b_p_cms, "%s");
  bool line_is_comment = false;
  linenr_T lnum = pos.lnum;

  // Allocate a new line: old-line + 'cms'-start + marker + 'cms'-end
  char *line = ml_get_buf(buf, lnum);
  size_t line_len = (size_t)ml_get_buf_len(buf, lnum);
  size_t added = 0;

  if (u_save(lnum - 1, lnum + 1) != OK) {
    return;
  }

  // Check if the line ends with an unclosed comment
  skip_comment(line, false, false, &line_is_comment);
  char *newline = xmalloc(line_len + markerlen + strlen(cms) + 1);
  STRCPY(newline, line);
  // Append the marker to the end of the line
  if (p == NULL || line_is_comment) {
    xmemcpyz(newline + line_len, marker, markerlen);
    added = markerlen;
  } else {
    STRCPY(newline + line_len, cms);
    memcpy(newline + line_len + (p - cms), marker, markerlen);
    STRCPY(newline + line_len + (p - cms) + markerlen, p + 2);
    added = markerlen + strlen(cms) - 2;
  }
  ml_replace_buf(buf, lnum, newline, false, false);
  if (added) {
    extmark_splice_cols(buf, (int)lnum - 1, (int)line_len,
                        0, (int)added, kExtmarkUndo);
  }
}

// deleteFoldMarkers() {{{2
/// Delete the markers for a fold, causing it to be deleted.
///
/// @param lnum_off  offset for fp->fd_top
static void deleteFoldMarkers(win_T *wp, fold_T *fp, bool recursive, linenr_T lnum_off)
{
  if (recursive) {
    for (int i = 0; i < fp->fd_nested.ga_len; i++) {
      deleteFoldMarkers(wp, (fold_T *)fp->fd_nested.ga_data + i, true,
                        lnum_off + fp->fd_top);
    }
  }
  foldDelMarker(wp->w_buffer, fp->fd_top + lnum_off, wp->w_p_fmr,
                foldstartmarkerlen);
  foldDelMarker(wp->w_buffer, fp->fd_top + lnum_off + fp->fd_len - 1,
                foldendmarker, foldendmarkerlen);
}

// foldDelMarker() {{{2
/// Delete marker "marker[markerlen]" at the end of line "lnum".
/// Delete 'commentstring' if it matches.
/// If the marker is not found, there is no error message.  Could be a missing
/// close-marker.
static void foldDelMarker(buf_T *buf, linenr_T lnum, char *marker, size_t markerlen)
{
  // end marker may be missing and fold extends below the last line
  if (lnum > buf->b_ml.ml_line_count) {
    return;
  }

  char *cms = buf->b_p_cms;
  char *line = ml_get_buf(buf, lnum);
  for (char *p = line; *p != NUL; p++) {
    if (strncmp(p, marker, markerlen) != 0) {
      continue;
    }
    // Found the marker, include a digit if it's there.
    size_t len = markerlen;
    if (ascii_isdigit(p[len])) {
      len++;
    }
    if (*cms != NUL) {
      // Also delete 'commentstring' if it matches.
      char *cms2 = strstr(cms, "%s");
      if (cms2 != NULL && p - line >= cms2 - cms
          && strncmp(p - (cms2 - cms), cms, (size_t)(cms2 - cms)) == 0
          && strncmp(p + len, cms2 + 2, strlen(cms2 + 2)) == 0) {
        p -= cms2 - cms;
        len += strlen(cms) - 2;
      }
    }
    if (u_save(lnum - 1, lnum + 1) == OK) {
      // Make new line: text-before-marker + text-after-marker
      char *newline = xmalloc((size_t)ml_get_buf_len(buf, lnum) - len + 1);
      assert(p >= line);
      memcpy(newline, line, (size_t)(p - line));
      STRCPY(newline + (p - line), p + len);
      ml_replace_buf(buf, lnum, newline, false, false);
      extmark_splice_cols(buf, (int)lnum - 1, (int)(p - line),
                          (int)len, 0, kExtmarkUndo);
    }
    break;
  }
}

// get_foldtext() {{{2
/// Generates text to display
///
/// @param buf allocated memory of length FOLD_TEXT_LEN. Used when 'foldtext'
///            isn't set puts the result in "buf[FOLD_TEXT_LEN]".
/// @param at line "lnum", with last line "lnume".
/// @return the text for a closed fold
///
/// Otherwise the result is in allocated memory.
char *get_foldtext(win_T *wp, linenr_T lnum, linenr_T lnume, foldinfo_T foldinfo, char *buf,
                   VirtText *vt)
  FUNC_ATTR_NONNULL_ALL
{
  char *text = NULL;
  // an error occurred when evaluating 'fdt' setting
  static bool got_fdt_error = false;
  int save_did_emsg = did_emsg;
  static win_T *last_wp = NULL;
  static linenr_T last_lnum = 0;

  if (last_wp == NULL || last_wp != wp || last_lnum > lnum || last_lnum == 0) {
    // window changed, try evaluating foldtext setting once again
    got_fdt_error = false;
  }

  if (!got_fdt_error) {
    // a previous error should not abort evaluating 'foldexpr'
    did_emsg = false;
  }

  if (*wp->w_p_fdt != NUL) {
    char dashes[MAX_LEVEL + 2];

    // Set "v:foldstart" and "v:foldend".
    set_vim_var_nr(VV_FOLDSTART, (varnumber_T)lnum);
    set_vim_var_nr(VV_FOLDEND, (varnumber_T)lnume);

    // Set "v:folddashes" to a string of "level" dashes.
    // Set "v:foldlevel" to "level".
    int level = MIN(foldinfo.fi_level, (int)sizeof(dashes) - 1);
    memset(dashes, '-', (size_t)level);
    dashes[level] = NUL;
    set_vim_var_string(VV_FOLDDASHES, dashes, -1);
    set_vim_var_nr(VV_FOLDLEVEL, (varnumber_T)level);

    // skip evaluating 'foldtext' on errors
    if (!got_fdt_error) {
      win_T *const save_curwin = curwin;
      const sctx_T saved_sctx = current_sctx;

      curwin = wp;
      curbuf = wp->w_buffer;
      current_sctx = wp->w_p_script_ctx[kWinOptFoldtext];

      emsg_off++;  // handle exceptions, but don't display errors

      Object obj = eval_foldtext(wp);
      if (obj.type == kObjectTypeArray) {
        Error err = ERROR_INIT;
        *vt = parse_virt_text(obj.data.array, &err, NULL);
        if (!ERROR_SET(&err)) {
          *buf = NUL;
          text = buf;
        }
        api_clear_error(&err);
      } else if (obj.type == kObjectTypeString) {
        text = obj.data.string.data;
        obj = NIL;
      }
      api_free_object(obj);

      emsg_off--;

      if (text == NULL || did_emsg) {
        got_fdt_error = true;
      }

      curwin = save_curwin;
      curbuf = curwin->w_buffer;
      current_sctx = saved_sctx;
    }
    last_lnum = lnum;
    last_wp = wp;
    set_vim_var_string(VV_FOLDDASHES, NULL, -1);

    if (!did_emsg && save_did_emsg) {
      did_emsg = save_did_emsg;
    }

    if (text != NULL) {
      // Replace unprintable characters, if there are any.  But
      // replace a TAB with a space.
      char *p;
      for (p = text; *p != NUL; p++) {
        int len = utfc_ptr2len(p);

        if (len > 1) {
          if (!vim_isprintc(utf_ptr2char(p))) {
            break;
          }
          p += len - 1;
        } else if (*p == TAB) {
          *p = ' ';
        } else if (ptr2cells(p) > 1) {
          break;
        }
      }
      if (*p != NUL) {
        p = transstr(text, true);
        xfree(text);
        text = p;
      }
    }
  }
  if (text == NULL) {
    int count = lnume - lnum + 1;

    vim_snprintf(buf, FOLD_TEXT_LEN,
                 NGETTEXT("+--%3d line folded",
                          "+--%3d lines folded ", count),
                 count);
    text = buf;
  }
  return text;
}

// foldtext_cleanup() {{{2
/// Remove 'foldmarker' and 'commentstring' from "str" (in-place).
static void foldtext_cleanup(char *str)
{
  // Ignore leading and trailing white space in 'commentstring'.
  char *cms_start = skipwhite(curbuf->b_p_cms);
  size_t cms_slen = strlen(cms_start);
  while (cms_slen > 0 && ascii_iswhite(cms_start[cms_slen - 1])) {
    cms_slen--;
  }

  // locate "%s" in 'commentstring', use the part before and after it.
  char *cms_end = strstr(cms_start, "%s");
  size_t cms_elen = 0;
  if (cms_end != NULL) {
    cms_elen = cms_slen - (size_t)(cms_end - cms_start);
    cms_slen = (size_t)(cms_end - cms_start);

    // exclude white space before "%s"
    while (cms_slen > 0 && ascii_iswhite(cms_start[cms_slen - 1])) {
      cms_slen--;
    }

    // skip "%s" and white space after it
    char *s = skipwhite(cms_end + 2);
    cms_elen -= (size_t)(s - cms_end);
    cms_end = s;
  }
  parseMarker(curwin);

  bool did1 = false;
  bool did2 = false;

  for (char *s = str; *s != NUL;) {
    size_t len = 0;
    if (strncmp(s, curwin->w_p_fmr, foldstartmarkerlen) == 0) {
      len = foldstartmarkerlen;
    } else if (strncmp(s, foldendmarker, foldendmarkerlen) == 0) {
      len = foldendmarkerlen;
    }
    if (len > 0) {
      if (ascii_isdigit(s[len])) {
        len++;
      }

      // May remove 'commentstring' start.  Useful when it's a double
      // quote and we already removed a double quote.
      char *p;
      for (p = s; p > str && ascii_iswhite(p[-1]); p--) {}
      if (p >= str + cms_slen
          && strncmp(p - cms_slen, cms_start, cms_slen) == 0) {
        len += (size_t)(s - p) + cms_slen;
        s = p - cms_slen;
      }
    } else if (cms_end != NULL) {
      if (!did1 && cms_slen > 0 && strncmp(s, cms_start, cms_slen) == 0) {
        len = cms_slen;
        did1 = true;
      } else if (!did2 && cms_elen > 0
                 && strncmp(s, cms_end, cms_elen) == 0) {
        len = cms_elen;
        did2 = true;
      }
    }
    if (len != 0) {
      while (ascii_iswhite(s[len])) {
        len++;
      }
      STRMOVE(s, s + len);
    } else {
      MB_PTR_ADV(s);
    }
  }
}

// foldlevelIndent() {{{2
/// Low level function to get the foldlevel for the "indent" method.
/// Doesn't use any caching.
///
/// @return  a level of -1 if the foldlevel depends on surrounding lines.
static void foldlevelIndent(fline_T *flp)
{
  linenr_T lnum = flp->lnum + flp->off;

  buf_T *buf = flp->wp->w_buffer;
  char *s = skipwhite(ml_get_buf(buf, lnum));

  // empty line or lines starting with a character in 'foldignore': level
  // depends on surrounding lines
  if (*s == NUL || vim_strchr(flp->wp->w_p_fdi, (uint8_t)(*s)) != NULL) {
    // first and last line can't be undefined, use level 0
    flp->lvl = (lnum == 1 || lnum == buf->b_ml.ml_line_count) ? 0 : -1;
  } else {
    flp->lvl = get_indent_buf(buf, lnum) / get_sw_value(buf);
  }
  flp->lvl = MIN(flp->lvl, (int)MAX(0, flp->wp->w_p_fdn));
}

// foldlevelDiff() {{{2
/// Low level function to get the foldlevel for the "diff" method.
/// Doesn't use any caching.
static void foldlevelDiff(fline_T *flp)
{
  flp->lvl = (rs_diff_infold(flp->wp, flp->lnum + flp->off)) ? 1 : 0;
}

// foldlevelExpr() {{{2
/// Low level function to get the foldlevel for the "expr" method.
/// Doesn't use any caching.
///
/// @return  a level of -1 if the foldlevel depends on surrounding lines.
static void foldlevelExpr(fline_T *flp)
{
  linenr_T lnum = flp->lnum + flp->off;

  win_T *win = curwin;
  curwin = flp->wp;
  curbuf = flp->wp->w_buffer;
  set_vim_var_nr(VV_LNUM, (varnumber_T)lnum);

  flp->start = 0;
  flp->had_end = flp->end;
  flp->end = MAX_LEVEL + 1;
  if (lnum <= 1) {
    flp->lvl = 0;
  }

  // KeyTyped may be reset to 0 when calling a function which invokes
  // do_cmdline().  To make 'foldopen' work correctly restore KeyTyped.
  const bool save_keytyped = KeyTyped;

  int c;
  const int n = eval_foldexpr(flp->wp, &c);
  KeyTyped = save_keytyped;

  switch (c) {
  // "a1", "a2", .. : add to the fold level
  case 'a':
    if (flp->lvl >= 0) {
      flp->lvl += n;
      flp->lvl_next = flp->lvl;
    }
    flp->start = n;
    break;

  // "s1", "s2", .. : subtract from the fold level
  case 's':
    if (flp->lvl >= 0) {
      if (n > flp->lvl) {
        flp->lvl_next = 0;
      } else {
        flp->lvl_next = flp->lvl - n;
      }
      flp->end = flp->lvl_next + 1;
    }
    break;

  // ">1", ">2", .. : start a fold with a certain level
  case '>':
    flp->lvl = n;
    flp->lvl_next = n;
    flp->start = 1;
    break;

  // "<1", "<2", .. : end a fold with a certain level
  case '<':
    // To prevent an unexpected start of a new fold, the next
    // level must not exceed the level of the current fold.
    flp->lvl_next = MIN(flp->lvl, n - 1);
    flp->end = n;
    break;

  // "=": No change in level
  case '=':
    flp->lvl_next = flp->lvl;
    break;

  // "-1", "0", "1", ..: set fold level
  default:
    if (n < 0) {
      // Use the current level for the next line, so that "a1"
      // will work there.
      flp->lvl_next = flp->lvl;
    } else {
      flp->lvl_next = n;
    }
    flp->lvl = n;
    break;
  }

  // If the level is unknown for the first or the last line in the file, use
  // level 0.
  if (flp->lvl < 0) {
    if (lnum <= 1) {
      flp->lvl = 0;
      flp->lvl_next = 0;
    }
    if (lnum == curbuf->b_ml.ml_line_count) {
      flp->lvl_next = 0;
    }
  }

  curwin = win;
  curbuf = curwin->w_buffer;
}

// parseMarker() {{{2
/// Parse 'foldmarker' and set "foldendmarker", "foldstartmarkerlen" and
/// "foldendmarkerlen".
/// Relies on the option value to have been checked for correctness already.
static void parseMarker(win_T *wp)
{
  foldendmarker = vim_strchr(wp->w_p_fmr, ',');
  foldstartmarkerlen = (size_t)(foldendmarker++ - wp->w_p_fmr);
  foldendmarkerlen = strlen(foldendmarker);
}

// foldlevelMarker() {{{2
/// Low level function to get the foldlevel for the "marker" method.
/// "foldendmarker", "foldstartmarkerlen" and "foldendmarkerlen" must have been
/// set before calling this.
/// Requires that flp->lvl is set to the fold level of the previous line!
/// Careful: This means you can't call this function twice on the same line.
/// Doesn't use any caching.
/// Sets flp->start when a start marker was found.
static void foldlevelMarker(fline_T *flp)
{
  int start_lvl = flp->lvl;

  // cache a few values for speed
  char *startmarker = flp->wp->w_p_fmr;
  char cstart = *startmarker;
  startmarker++;
  char cend = *foldendmarker;

  // Default: no start found, next level is same as current level
  flp->start = 0;
  flp->lvl_next = flp->lvl;

  char *s = ml_get_buf(flp->wp->w_buffer, flp->lnum + flp->off);
  while (*s) {
    if (*s == cstart
        && strncmp(s + 1, startmarker, foldstartmarkerlen - 1) == 0) {
      // found startmarker: set flp->lvl
      s += foldstartmarkerlen;
      if (ascii_isdigit(*s)) {
        int n = atoi(s);
        if (n > 0) {
          flp->lvl = n;
          flp->lvl_next = n;
          flp->start = MAX(n - start_lvl, 1);
        }
      } else {
        flp->lvl++;
        flp->lvl_next++;
        flp->start++;
      }
    } else if (*s == cend
               && strncmp(s + 1, foldendmarker + 1, foldendmarkerlen - 1) == 0) {
      // found endmarker: set flp->lvl_next
      s += foldendmarkerlen;
      if (ascii_isdigit(*s)) {
        int n = atoi(s);
        if (n > 0) {
          flp->lvl = n;
          flp->lvl_next = n - 1;
          // never start a fold with an end marker
          flp->lvl_next = MIN(flp->lvl_next, start_lvl);
        }
      } else {
        flp->lvl_next--;
      }
    } else {
      MB_PTR_ADV(s);
    }
  }

  // The level can't go negative, must be missing a start marker.
  flp->lvl_next = MAX(flp->lvl_next, 0);
}

// foldlevelSyntax() {{{2
/// Low level function to get the foldlevel for the "syntax" method.
/// Doesn't use any caching.
static void foldlevelSyntax(fline_T *flp)
{
  linenr_T lnum = flp->lnum + flp->off;

  // Use the maximum fold level at the start of this line and the next.
  flp->lvl = syn_get_foldlevel(flp->wp, lnum);
  flp->start = 0;
  if (lnum < flp->wp->w_buffer->b_ml.ml_line_count) {
    int n = syn_get_foldlevel(flp->wp, lnum + 1);
    if (n > flp->lvl) {
      flp->start = n - flp->lvl;        // fold(s) start here
      flp->lvl = n;
    }
  }
}

// functions for storing the fold state in a View {{{1
// put_folds() {{{2
/// Write commands to "fd" to restore the manual folds in window "wp".
///
/// @return  FAIL if writing fails.
int put_folds(FILE *fd, win_T *wp)
{
  if (rs_foldmethodIsManual(wp)) {
    if (put_line(fd, "silent! normal! zE") == FAIL
        || put_folds_recurse(fd, &wp->w_folds, 0) == FAIL
        || put_line(fd, "let &fdl = &fdl") == FAIL) {
      return FAIL;
    }
  }

  // If some folds are manually opened/closed, need to restore that.
  if (wp->w_fold_manual) {
    return put_foldopen_recurse(fd, wp, &wp->w_folds, 0);
  }

  return OK;
}

// put_folds_recurse() {{{2
/// Write commands to "fd" to recreate manually created folds.
///
/// @return  FAIL when writing failed.
static int put_folds_recurse(FILE *fd, garray_T *gap, linenr_T off)
{
  fold_T *fp = (fold_T *)gap->ga_data;
  for (int i = 0; i < gap->ga_len; i++) {
    // Do nested folds first, they will be created closed.
    if (put_folds_recurse(fd, &fp->fd_nested, off + fp->fd_top) == FAIL) {
      return FAIL;
    }
    if (fprintf(fd, "sil! %" PRId64 ",%" PRId64 "fold",
                (int64_t)fp->fd_top + off,
                (int64_t)(fp->fd_top + off + fp->fd_len - 1)) < 0
        || put_eol(fd) == FAIL) {
      return FAIL;
    }
    fp++;
  }
  return OK;
}

// put_foldopen_recurse() {{{2
/// Write commands to "fd" to open and close manually opened/closed folds.
///
/// @return  FAIL when writing failed.
static int put_foldopen_recurse(FILE *fd, win_T *wp, garray_T *gap, linenr_T off)
{
  fold_T *fp = (fold_T *)gap->ga_data;
  for (int i = 0; i < gap->ga_len; i++) {
    if (fp->fd_flags != FD_LEVEL) {
      if (!GA_EMPTY(&fp->fd_nested)) {
        // open nested folds while this fold is open
        // ignore errors
        if (fprintf(fd, "%" PRId64, (int64_t)fp->fd_top + off) < 0
            || put_eol(fd) == FAIL
            || put_line(fd, "sil! normal! zo") == FAIL) {
          return FAIL;
        }
        if (put_foldopen_recurse(fd, wp, &fp->fd_nested,
                                 off + fp->fd_top)
            == FAIL) {
          return FAIL;
        }
        // close the parent when needed
        if (fp->fd_flags == FD_CLOSED) {
          if (put_fold_open_close(fd, fp, off) == FAIL) {
            return FAIL;
          }
        }
      } else {
        // Open or close the leaf according to the window foldlevel.
        // Do not close a leaf that is already closed, as it will close
        // the parent.
        int level = rs_foldLevelWin(wp, off + fp->fd_top);
        if ((fp->fd_flags == FD_CLOSED && wp->w_p_fdl >= level)
            || (fp->fd_flags != FD_CLOSED && wp->w_p_fdl < level)) {
          if (put_fold_open_close(fd, fp, off) == FAIL) {
            return FAIL;
          }
        }
      }
    }
    fp++;
  }

  return OK;
}

// put_fold_open_close() {{{2
/// Write the open or close command to "fd".
///
/// @return  FAIL when writing failed.
static int put_fold_open_close(FILE *fd, fold_T *fp, linenr_T off)
{
  if (fprintf(fd, "%" PRIdLINENR, fp->fd_top + off) < 0
      || put_eol(fd) == FAIL
      || fprintf(fd, "sil! normal! z%c",
                 fp->fd_flags == FD_CLOSED ? 'c' : 'o') < 0
      || put_eol(fd) == FAIL) {
    return FAIL;
  }

  return OK;
}

// }}}1

/// "foldclosed()" and "foldclosedend()" functions
static void foldclosed_both(typval_T *argvars, typval_T *rettv, bool end)
{
  const linenr_T lnum = tv_get_lnum(argvars);
  if (lnum >= 1 && lnum <= curbuf->b_ml.ml_line_count) {
    linenr_T first;
    linenr_T last;
    if (hasFoldingWin(curwin, lnum, &first, &last, false, NULL)) {
      rettv->vval.v_number = (varnumber_T)(end ? last : first);
      return;
    }
  }
  rettv->vval.v_number = -1;
}

/// "foldclosed()" function
void f_foldclosed(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  foldclosed_both(argvars, rettv, false);
}

/// "foldclosedend()" function
void f_foldclosedend(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  foldclosed_both(argvars, rettv, true);
}

/// "foldlevel()" function
void f_foldlevel(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  const linenr_T lnum = tv_get_lnum(argvars);
  if (lnum >= 1 && lnum <= curbuf->b_ml.ml_line_count) {
    rettv->vval.v_number = rs_foldLevel(lnum);
  }
}

/// "foldtext()" function
void f_foldtext(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = NULL;

  linenr_T foldstart = (linenr_T)get_vim_var_nr(VV_FOLDSTART);
  linenr_T foldend = (linenr_T)get_vim_var_nr(VV_FOLDEND);
  char *dashes = get_vim_var_str(VV_FOLDDASHES);
  if (foldstart > 0 && foldend <= curbuf->b_ml.ml_line_count) {
    // Find first non-empty line in the fold.
    linenr_T lnum;
    for (lnum = foldstart; lnum < foldend; lnum++) {
      if (!linewhite(lnum)) {
        break;
      }
    }

    // Find interesting text in this line.
    char *s = skipwhite(ml_get(lnum));
    // skip C comment-start
    if (s[0] == '/' && (s[1] == '*' || s[1] == '/')) {
      s = skipwhite(s + 2);
      if (*skipwhite(s) == NUL && lnum + 1 < foldend) {
        s = skipwhite(ml_get(lnum + 1));
        if (*s == '*') {
          s = skipwhite(s + 1);
        }
      }
    }
    int count = foldend - foldstart + 1;
    char *txt = NGETTEXT("+-%s%3d line: ", "+-%s%3d lines: ", count);
    size_t len = strlen(txt)
                 + strlen(dashes)  // for %s
                 + 20              // for %3ld
                 + strlen(s);      // concatenated
    char *r = xmalloc(len);
    snprintf(r, len, txt, dashes, count);
    len = strlen(r);
    strcat(r, s);
    // remove 'foldmarker' and 'commentstring'
    foldtext_cleanup(r + len);
    rettv->vval.v_string = r;
  }
}

/// "foldtextresult(lnum)" function
void f_foldtextresult(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  char buf[FOLD_TEXT_LEN];
  static bool entered = false;

  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = NULL;
  if (entered) {
    return;  // reject recursive use
  }
  entered = true;
  linenr_T lnum = tv_get_lnum(argvars);
  // Treat illegal types and illegal string values for {lnum} the same.
  lnum = MAX(lnum, 0);

  foldinfo_T info = rs_fold_info(curwin, lnum);
  if (info.fi_lines > 0) {
    VirtText vt = VIRTTEXT_EMPTY;
    char *text = get_foldtext(curwin, lnum, lnum + info.fi_lines - 1, info, buf, &vt);
    if (text == buf) {
      text = xstrdup(text);
    }
    if (kv_size(vt) > 0) {
      assert(*text == NUL);
      for (size_t i = 0; i < kv_size(vt);) {
        int attr = 0;
        char *new_text = next_virt_text_chunk(vt, &i, &attr);
        if (new_text == NULL) {
          break;
        }
        new_text = concat_str(text, new_text);
        xfree(text);
        text = new_text;
      }
    }
    clear_virttext(&vt);
    rettv->vval.v_string = text;
  }

  entered = false;
}

// ============================================================================
// Rust FFI accessor functions
// ============================================================================

/// Emit error message for cannot create fold with current foldmethod.
void nvim_emsg_fold_cannot_create(void)
{
  emsg(_("E350: Cannot create fold with current 'foldmethod'"));
}

/// Emit error message for cannot delete fold with current foldmethod.
void nvim_emsg_fold_cannot_delete(void)
{
  emsg(_("E351: Cannot delete fold with current 'foldmethod'"));
}

/// Get the w_p_fdl (foldlevel) field from a window.
int nvim_win_get_p_fdl(win_T *wp)
{
  return (int)wp->w_p_fdl;
}

/// Get the w_fold_manual field from a window.
int nvim_win_get_w_fold_manual(win_T *wp)
{
  return wp->w_fold_manual;
}

/// Get a pointer to the window's folds growarray.
garray_T *nvim_win_get_folds(win_T *wp)
{
  return &wp->w_folds;
}

/// Get the length of a garray.
int nvim_ga_len(garray_T *gap)
{
  return gap->ga_len;
}

/// Get a fold_T pointer at index in a garray.
/// Returns NULL if index is out of bounds.
fold_T *nvim_ga_fold_at(garray_T *gap, int idx)
{
  if (idx < 0 || idx >= gap->ga_len) {
    return NULL;
  }
  return &((fold_T *)gap->ga_data)[idx];
}

/// Get the fd_top field from a fold.
linenr_T nvim_fold_get_fd_top(fold_T *fp)
{
  return fp->fd_top;
}

/// Get the fd_len field from a fold.
linenr_T nvim_fold_get_fd_len(fold_T *fp)
{
  return fp->fd_len;
}

/// Get a pointer to the nested folds growarray.
garray_T *nvim_fold_get_fd_nested(fold_T *fp)
{
  return &fp->fd_nested;
}

/// Get the fd_flags field from a fold.
int nvim_fold_get_fd_flags(fold_T *fp)
{
  return (int)fp->fd_flags;
}

/// Get the w_foldinvalid field from a window.
bool nvim_win_get_w_foldinvalid(win_T *wp)
{
  return wp->w_foldinvalid;
}

/// Set the w_foldinvalid field in a window.
void nvim_win_set_w_foldinvalid(win_T *wp, bool val)
{
  wp->w_foldinvalid = val;
}

/// Get the w_lines_valid field from a window.
int nvim_win_get_w_lines_valid(win_T *wp)
{
  return wp->w_lines_valid;
}

/// Get a wline_T pointer at index in a window's w_lines array.
/// Returns NULL if index is out of bounds.
wline_T *nvim_win_get_wl_entry(win_T *wp, int idx)
{
  if (idx < 0 || idx >= wp->w_lines_valid) {
    return NULL;
  }
  return &wp->w_lines[idx];
}

/// Get the wl_lnum field from a wline_T.
linenr_T nvim_wline_get_lnum(wline_T *wl)
{
  return wl->wl_lnum;
}

/// Get the wl_foldend field from a wline_T.
linenr_T nvim_wline_get_foldend(wline_T *wl)
{
  return wl->wl_foldend;
}

/// Get the wl_valid field from a wline_T.
bool nvim_wline_get_valid(wline_T *wl)
{
  return wl->wl_valid;
}

/// Get the wl_folded field from a wline_T.
bool nvim_wline_get_folded(wline_T *wl)
{
  return wl->wl_folded;
}

/// Get the wl_size field from a wline_T.
uint16_t nvim_wline_get_size(wline_T *wl)
{
  return wl->wl_size;
}

/// Get the wl_lastlnum field from a wline_T.
linenr_T nvim_wline_get_lastlnum(wline_T *wl)
{
  return wl->wl_lastlnum;
}

// ============================================================================
// Accessors for recursive functions
// ============================================================================

/// Set the fd_flags field of a fold.
void nvim_fold_set_fd_flags(fold_T *fp, int flags)
{
  fp->fd_flags = (char)flags;
}

/// Get the fd_small field from a fold.
int nvim_fold_get_fd_small(fold_T *fp)
{
  return (int)fp->fd_small;
}

/// Set the fd_small field of a fold.
void nvim_fold_set_fd_small(fold_T *fp, int small)
{
  fp->fd_small = (TriState)small;
}

/// Swap two fold entries in a garray.
/// idx1 and idx2 must be valid indices.
void nvim_fold_swap(garray_T *gap, int idx1, int idx2)
{
  fold_T *data = (fold_T *)gap->ga_data;
  fold_T tmp = data[idx1];
  data[idx1] = data[idx2];
  data[idx2] = tmp;
}

// ============================================================================
// State query accessors
// ============================================================================

/// Get the w_p_fml (foldminlines) field from a window.
int nvim_win_get_p_fml(win_T *wp)
{
  return (int)wp->w_p_fml;
}

/// Get the number of screen lines for a physical line (no fold consideration).
int nvim_plines_win_nofold(win_T *wp, linenr_T lnum)
{
  return plines_win_nofold(wp, lnum);
}

// ============================================================================
// Foundation function accessors
// ============================================================================

/// Initialize the folds garray for a window (called from Rust).
void nvim_ga_init_folds(garray_T *gap)
{
  ga_init(gap, (int)sizeof(fold_T), 10);
}

// ============================================================================
// Core query accessors
// ============================================================================

/// Get the line count of the window's buffer.
linenr_T nvim_win_get_buf_line_count(win_T *wp)
{
  return wp->w_buffer->b_ml.ml_line_count;
}

// ============================================================================
// Fold Markers accessors
// ============================================================================

/// Get the w_p_fmr (foldmarker option) field from a window.
char *nvim_win_get_p_fmr(win_T *wp)
{
  return wp->w_p_fmr;
}

/// Get a line from a buffer (wrapper for ml_get_buf).
char *nvim_ml_get_buf(buf_T *buf, linenr_T lnum)
{
  return ml_get_buf(buf, lnum);
}

// ============================================================================
// Fold Level Calculation accessors
// ============================================================================

/// Get the w_p_fdi (foldignore option) field from a window.
char *nvim_win_get_p_fdi(win_T *wp)
{
  return wp->w_p_fdi;
}

/// Get the w_p_fdn (foldnestmax option) field from a window.
int nvim_win_get_p_fdn(win_T *wp)
{
  return (int)wp->w_p_fdn;
}

/// Get the indentation of a buffer line (wrapper for get_indent_buf).
int nvim_get_indent_buf(buf_T *buf, linenr_T lnum)
{
  return get_indent_buf(buf, lnum);
}

/// Get the shiftwidth value for a buffer (wrapper for get_sw_value).
int nvim_get_sw_value(buf_T *buf)
{
  return (int)get_sw_value(buf);
}

/// Check if a line is in a diff fold (wrapper for diff_infold).
int nvim_rs_diff_infold(win_T *wp, linenr_T lnum)
{
  return rs_diff_infold(wp, lnum);
}

/// Skip whitespace at the beginning of a string (wrapper for skipwhite).
char *nvim_skipwhite(const char *s)
{
  return skipwhite(s);
}

/// Find a character in a string (wrapper for vim_strchr).
char *nvim_vim_strchr(const char *s, int c)
{
  return vim_strchr(s, c);
}

// ============================================================================
// Fold Tree Manipulation accessors
// ============================================================================

/// Grow a garray to hold at least n more fold_T entries.
void nvim_ga_grow_folds(garray_T *gap, int n)
{
  ga_grow(gap, n);
}

/// Set the fd_top field of a fold.
void nvim_fold_set_fd_top(fold_T *fp, linenr_T top)
{
  fp->fd_top = top;
}

/// Set the fd_len field of a fold.
void nvim_fold_set_fd_len(fold_T *fp, linenr_T len)
{
  fp->fd_len = len;
}

/// Get the ga_data pointer from a garray (as fold_T*).
fold_T *nvim_ga_get_fold_data(garray_T *gap)
{
  return (fold_T *)gap->ga_data;
}

/// Set the ga_len field of a garray.
void nvim_ga_set_len(garray_T *gap, int len)
{
  gap->ga_len = len;
}

/// Move fold entries within a garray.
/// Moves `count` entries from src_idx to dst_idx.
void nvim_fold_memmove(garray_T *gap, int dst_idx, int src_idx, int count)
{
  fold_T *data = (fold_T *)gap->ga_data;
  memmove(&data[dst_idx], &data[src_idx], sizeof(fold_T) * (size_t)count);
}

/// Copy a fold entry from one location to another.
void nvim_fold_copy(fold_T *dst, const fold_T *src)
{
  *dst = *src;
}

/// Call deleteFoldRecurse from Rust (to recursively free nested fold memory).
void nvim_deleteFoldRecurse(buf_T *buf, garray_T *gap)
{
  deleteFoldRecurse(buf, gap);
}

/// Free the ga_data pointer of a garray (for nested folds).
void nvim_ga_free_data(garray_T *gap)
{
  xfree(gap->ga_data);
  gap->ga_data = NULL;
  gap->ga_len = 0;
}

/// Set the fold_changed flag.
void nvim_set_fold_changed(bool changed)
{
  fold_changed = changed;
}

/// Get the fold_changed flag.
bool nvim_get_fold_changed(void)
{
  return fold_changed;
}

// ============================================================================
// Fold State Management accessors
// ============================================================================

/// Set the w_fold_manual field in a window.
void nvim_win_set_w_fold_manual(win_T *wp, bool val)
{
  wp->w_fold_manual = val;
}

/// Call changed_window_setting for a window.
void nvim_changed_window_setting(win_T *wp)
{
  changed_window_setting(wp);
}

/// Emit the "no fold found" error message.
void nvim_emsg_nofold(void)
{
  emsg(_(e_nofold));
}

/// Get the w_p_scb (scrollbind) field from a window.
bool nvim_win_get_p_scb(win_T *wp)
{
  return wp->w_p_scb;
}

/// Get the first window in the current tab.
win_T *nvim_get_first_win_in_tab(void)
{
  return curtab->tp_firstwin;
}

/// Wrapper for diff_lnum_win.
linenr_T nvim_diff_lnum_win(linenr_T lnum, win_T *wp)
{
  return rs_diff_lnum_win(lnum, wp);
}

/// Set the w_p_fdl (foldlevel) field in a window.
void nvim_win_set_p_fdl(win_T *wp, int fdl)
{
  wp->w_p_fdl = fdl;
}

// ============================================================================
// Fold Creation and Deletion accessors
// ============================================================================

/// Initialize a garray with specified itemsize and growsize.
void nvim_ga_init_folds_ex(garray_T *gap, int itemsize, int growsize)
{
  ga_init(gap, itemsize, growsize);
}

/// Get the ga_itemsize field from a garray.
int nvim_ga_get_itemsize(garray_T *gap)
{
  return gap->ga_itemsize;
}

/// Get the ga_growsize field from a garray.
int nvim_ga_get_growsize(garray_T *gap)
{
  return gap->ga_growsize;
}

/// Check if a garray is empty.
bool nvim_ga_is_empty(garray_T *gap)
{
  return GA_EMPTY(gap);
}

// ============================================================================
// Manual Fold Operations accessors
// ============================================================================

/// Wrapper for foldCreateMarkers for Rust.
void nvim_foldCreateMarkers(win_T *wp, linenr_T start_lnum, linenr_T end_lnum)
{
  pos_T start = { start_lnum, 0, 0 };
  pos_T end = { end_lnum, 0, 0 };
  foldCreateMarkers(wp, start, end);
}

/// Wrapper for parseMarker for Rust.
void nvim_parseMarker(win_T *wp)
{
  parseMarker(wp);
}

/// Wrapper for deleteFoldMarkers for Rust.
void nvim_deleteFoldMarkers(win_T *wp, fold_T *fp, bool recursive, linenr_T lnum_off)
{
  deleteFoldMarkers(wp, fp, recursive, lnum_off);
}

/// Check if buffer is modifiable (for fold operations).
int nvim_fold_buf_is_modifiable(buf_T *buf)
{
  return MODIFIABLE(buf) ? 1 : 0;
}

/// Emit error message for buffer not modifiable (for fold operations).
void nvim_fold_emsg_modifiable(void)
{
  emsg(_(e_modifiable));
}

/// Wrapper for check_cursor_col for Rust.
void nvim_check_cursor_col(win_T *wp)
{
  check_cursor_col(wp);
}

/// Wrapper for changed_lines for Rust.
void nvim_changed_lines(buf_T *buf, linenr_T first, int col, linenr_T last, linenr_T xtra,
                        bool add_undo)
{
  changed_lines(buf, first, col, last, xtra, add_undo);
}

/// Wrapper for buf_updates_send_changes for Rust.
void nvim_buf_updates_send_changes(buf_T *buf, linenr_T firstlnum, int64_t num_added,
                                   int64_t num_removed)
{
  buf_updates_send_changes(buf, firstlnum, num_added, num_removed);
}

/// Redraw buffer later.
void nvim_redraw_buf_later(buf_T *buf, int redraw_type)
{
  redraw_buf_later(buf, redraw_type);
}

/// Redraw the current buffer later.
void nvim_redraw_curbuf_later(int redraw_type)
{
  redraw_curbuf_later(redraw_type);
}

// ============================================================================
// IEMS Algorithm accessors
// ============================================================================

// Note: nvim_get_got_int is defined in ex_eval.c

/// Call line_breakcheck.
void nvim_line_breakcheck(void)
{
  line_breakcheck();
}

/// Get buffer line count (for fold Rust code).
linenr_T nvim_fold_buf_get_line_count(buf_T *buf)
{
  return buf->b_ml.ml_line_count;
}

/// Get diff_context global.
linenr_T nvim_get_diff_context(void)
{
  return diff_context;
}

/// Redraw window range later.
void nvim_redraw_win_range_later(win_T *wp, linenr_T top, linenr_T bot)
{
  redraw_win_range_later(wp, top, bot);
}

/// Wrapper for foldlevelIndent that returns the FoldLevelResult.
FoldLevelResult_C nvim_foldlevelIndent(win_T *wp, linenr_T lnum, linenr_T off)
{
  fline_T flp;
  flp.wp = wp;
  flp.lnum = lnum;
  flp.off = off;
  flp.lvl = 0;
  flp.lvl_next = -1;
  flp.start = 0;
  flp.end = MAX_LEVEL + 1;
  flp.had_end = MAX_LEVEL + 1;

  foldlevelIndent(&flp);

  FoldLevelResult_C result = {
    .lvl = flp.lvl,
    .lvl_next = flp.lvl_next,
    .start = flp.start,
    .end = flp.end,
  };
  return result;
}

/// Wrapper for foldlevelDiff that returns the FoldLevelResult.
FoldLevelResult_C nvim_foldlevelDiff(win_T *wp, linenr_T lnum, linenr_T off)
{
  fline_T flp;
  flp.wp = wp;
  flp.lnum = lnum;
  flp.off = off;
  flp.lvl = 0;
  flp.lvl_next = -1;
  flp.start = 0;
  flp.end = MAX_LEVEL + 1;
  flp.had_end = MAX_LEVEL + 1;

  foldlevelDiff(&flp);

  FoldLevelResult_C result = {
    .lvl = flp.lvl,
    .lvl_next = flp.lvl_next,
    .start = flp.start,
    .end = flp.end,
  };
  return result;
}

// Note: nvim_parseMarker is already defined above (around line 3061)

/// Wrapper for foldlevelMarker that returns the FoldLevelResult.
/// Requires nvim_parseMarker to be called first.
/// Current level (flp.lvl) must be passed in - caller tracks state.
FoldLevelResult_C nvim_foldlevelMarker(win_T *wp, linenr_T lnum, linenr_T off, int current_lvl)
{
  fline_T flp;
  flp.wp = wp;
  flp.lnum = lnum;
  flp.off = off;
  flp.lvl = current_lvl;
  flp.lvl_next = current_lvl;
  flp.start = 0;
  flp.end = MAX_LEVEL + 1;
  flp.had_end = MAX_LEVEL + 1;

  foldlevelMarker(&flp);

  FoldLevelResult_C result = {
    .lvl = flp.lvl,
    .lvl_next = flp.lvl_next,
    .start = flp.start,
    .end = flp.end,
  };
  return result;
}

/// Wrapper for foldlevelExpr that returns the FoldLevelResult.
/// current_lvl must be passed in (for 'a' and 's' codes that modify the level).
FoldLevelResult_C nvim_foldlevelExpr(win_T *wp, linenr_T lnum, linenr_T off, int current_lvl)
{
  fline_T flp;
  flp.wp = wp;
  flp.lnum = lnum;
  flp.off = off;
  flp.lvl = current_lvl;
  flp.lvl_next = -1;
  flp.start = 0;
  flp.end = MAX_LEVEL + 1;
  flp.had_end = MAX_LEVEL + 1;

  foldlevelExpr(&flp);

  FoldLevelResult_C result = {
    .lvl = flp.lvl,
    .lvl_next = flp.lvl_next,
    .start = flp.start,
    .end = flp.end,
  };
  return result;
}

/// Wrapper for foldlevelSyntax that returns the FoldLevelResult.
FoldLevelResult_C nvim_foldlevelSyntax(win_T *wp, linenr_T lnum, linenr_T off)
{
  fline_T flp;
  flp.wp = wp;
  flp.lnum = lnum;
  flp.off = off;
  flp.lvl = 0;
  flp.lvl_next = -1;
  flp.start = 0;
  flp.end = MAX_LEVEL + 1;
  flp.had_end = MAX_LEVEL + 1;

  foldlevelSyntax(&flp);

  FoldLevelResult_C result = {
    .lvl = flp.lvl,
    .lvl_next = flp.lvl_next,
    .start = flp.start,
    .end = flp.end,
  };
  return result;
}

/// Wrapper for foldFind that returns index.
/// Returns 1 if found, 0 if not found. Sets found_idx to the index.
int nvim_foldFind(garray_T *gap, linenr_T lnum, int *found_idx)
{
  fold_T *fp;
  int result = foldFind(gap, lnum, &fp);
  *found_idx = (int)(fp - (fold_T *)gap->ga_data);
  return result ? 1 : 0;
}

// Accessors for fold statics
linenr_T nvim_get_invalid_top(void) { return invalid_top; }
void nvim_set_invalid_top(linenr_T val) { invalid_top = val; }
linenr_T nvim_get_invalid_bot(void) { return invalid_bot; }
void nvim_set_invalid_bot(linenr_T val) { invalid_bot = val; }
linenr_T nvim_get_prev_lnum(void) { return prev_lnum; }
void nvim_set_prev_lnum(linenr_T val) { prev_lnum = val; }
int nvim_get_prev_lnum_lvl(void) { return prev_lnum_lvl; }
void nvim_set_prev_lnum_lvl(int val) { prev_lnum_lvl = val; }

/// Get the p_fcl option value.
char *nvim_get_p_fcl(void) { return p_fcl; }

/// Get the disable_fold_update flag.
int nvim_get_disable_fold_update(void) { return disable_fold_update; }

/// Get the need_diff_redraw flag.
int nvim_get_need_diff_redraw(void) { return need_diff_redraw; }

/// Call foldUpdate from Rust.
void nvim_foldUpdate(win_T *wp, linenr_T top, linenr_T bot)
{
  foldUpdate(wp, top, bot);
}

// Note: nvim_win_get_p_fen is defined in window.c
