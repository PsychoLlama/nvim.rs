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

// =============================================================================
// Rust function declarations
// =============================================================================

extern int rs_cls(bool bigword);
extern bool rs_skip_chars(int cclass, int dir, bool bigword);
extern void rs_back_in_line(bool bigword);
extern int rs_fwd_word(int count, bool bigword, bool eol);
extern int rs_bck_word(int count, bool bigword, bool stop);
extern int rs_end_word(int count, bool bigword, bool stop, bool empty);
extern int rs_bckend_word(int count, bool bigword, bool eol);
extern int rs_find_next_quote(char *line, int col, int quotechar, char *escape);
extern int rs_find_prev_quote(char *line, int col_start, int quotechar, char *escape);
extern int rs_current_word(oparg_T *oap, int count, bool include, bool bigword);
extern bool rs_inmacro(char *opt, const char *s);
extern bool rs_startPS(int lnum, int para, bool both);
extern bool rs_findpar(bool *pincl, int dir, int count, int what, bool both);
extern int rs_current_par(oparg_T *oap, int count, bool include, int type);
extern int rs_findsent(int dir, int count);
extern void rs_find_first_blank(pos_T *posp);
extern void rs_findsent_forward(int count, bool at_start_sent);
extern int rs_current_sent(oparg_T *oap, int count, bool include);
extern int rs_current_block(oparg_T *oap, int count, bool include, int what, int other);

// =============================================================================
// C accessor functions for Rust
// =============================================================================

/// Get character at cursor position (accessor for Rust).
int nvim_textobj_gchar_cursor(void)
{
  return gchar_cursor();
}

/// Increment cursor position (accessor for Rust).
int nvim_textobj_inc_cursor(void)
{
  return inc_cursor();
}

/// Decrement cursor position (accessor for Rust).
int nvim_textobj_dec_cursor(void)
{
  return dec_cursor();
}

/// Get UTF character class (accessor for Rust).
int nvim_textobj_utf_class(int c)
{
  return utf_class(c);
}

/// Get current cursor column (accessor for Rust).
int nvim_textobj_get_cursor_col(void)
{
  return curwin->w_cursor.col;
}

/// Get current cursor line number (accessor for Rust).
int nvim_textobj_get_cursor_lnum(void)
{
  return curwin->w_cursor.lnum;
}

/// Get total line count in current buffer (accessor for Rust).
int nvim_textobj_get_ml_line_count(void)
{
  return curbuf->b_ml.ml_line_count;
}

/// Check if a line is empty (accessor for Rust).
bool nvim_textobj_is_lineempty(int lnum)
{
  return LINEEMPTY(lnum);
}

/// Get pointer to cursor line content (accessor for Rust).
char *nvim_textobj_get_cursor_line_ptr(void)
{
  return get_cursor_line_ptr();
}

/// Set cursor coladd to 0 (accessor for Rust).
void nvim_textobj_set_cursor_coladd_zero(void)
{
  curwin->w_cursor.coladd = 0;
}

/// Check for folding at line (accessor for Rust).
bool nvim_textobj_hasFolding(int lnum, int *first, int *last)
{
  linenr_T first_lnum = 0;
  linenr_T last_lnum = 0;
  bool result = hasFolding(curwin, lnum, first ? &first_lnum : NULL, last ? &last_lnum : NULL);
  if (first) {
    *first = first_lnum;
  }
  if (last) {
    *last = last_lnum;
  }
  return result;
}

/// Move cursor to given column (accessor for Rust).
void nvim_textobj_coladvance(int col)
{
  coladvance(curwin, col);
}

/// Adjust skipcol after cursor movement (accessor for Rust).
void nvim_textobj_adjust_skipcol(void)
{
  adjust_skipcol();
}

/// Set cursor line number (accessor for Rust).
void nvim_textobj_set_cursor_lnum(int lnum)
{
  curwin->w_cursor.lnum = lnum;
}

/// Set cursor column (accessor for Rust).
void nvim_textobj_set_cursor_col(int col)
{
  curwin->w_cursor.col = col;
}

/// Unadjust for exclusive selection if needed (accessor for Rust).
void nvim_textobj_unadjust_for_sel_if_needed(void)
{
  if (*p_sel == 'e' && VIsual_active && VIsual_mode == 'v' && VIsual_select_exclu_adj) {
    unadjust_for_sel();
  }
}

/// Get length of multibyte character (accessor for Rust).
int nvim_textobj_utfc_ptr2len(const char *p)
{
  return utfc_ptr2len(p);
}

/// Get head offset for multibyte char (accessor for Rust).
int nvim_textobj_utf_head_off(const char *base, const char *p)
{
  return utf_head_off(base, p);
}

/// Search for character in string (accessor for Rust).
char *nvim_textobj_vim_strchr(const char *s, int c)
{
  return vim_strchr(s, c);
}

// =============================================================================
// Accessors for current_word function
// =============================================================================

/// Get VIsual_active state (accessor for Rust).
bool nvim_textobj_get_VIsual_active(void)
{
  return VIsual_active;
}

/// Get VIsual position lnum (accessor for Rust).
int nvim_textobj_get_VIsual_lnum(void)
{
  return VIsual.lnum;
}

/// Get VIsual position column (accessor for Rust).
int nvim_textobj_get_VIsual_col(void)
{
  return VIsual.col;
}

/// Get VIsual_mode (accessor for Rust).
int nvim_textobj_get_VIsual_mode(void)
{
  return VIsual_mode;
}

/// Set VIsual_mode (accessor for Rust).
void nvim_textobj_set_VIsual_mode(int mode)
{
  VIsual_mode = mode;
}

/// Set VIsual position (accessor for Rust).
void nvim_textobj_set_VIsual(int lnum, int col)
{
  VIsual.lnum = lnum;
  VIsual.col = col;
}

/// Get selection option first char (accessor for Rust).
int nvim_textobj_get_p_sel_first(void)
{
  return *p_sel;
}

/// Check if cursor is less than VIsual (accessor for Rust).
bool nvim_textobj_lt_cursor_VIsual(void)
{
  return lt(curwin->w_cursor, VIsual);
}

/// Check if cursor equals VIsual (accessor for Rust).
bool nvim_textobj_equalpos_cursor_VIsual(void)
{
  return equalpos(curwin->w_cursor, VIsual);
}

/// Check if VIsual is less than cursor (accessor for Rust).
bool nvim_textobj_lt_VIsual_cursor(void)
{
  return lt(VIsual, curwin->w_cursor);
}

/// Check if VIsual is less than or equal to cursor (accessor for Rust).
bool nvim_textobj_ltoreq_VIsual_cursor(void)
{
  return ltoreq(VIsual, curwin->w_cursor);
}

/// Clear a position on the stack (accessor for Rust, uses cursor position).
void nvim_textobj_clearpos_cursor(void)
{
  // This sets cursor to an invalid pos indicating no position set
  clearpos(&curwin->w_cursor);
}

/// Set operator argument start position from cursor (accessor for Rust).
void nvim_textobj_set_oap_start_from_cursor(oparg_T *oap)
{
  oap->start = curwin->w_cursor;
}

/// Set operator argument motion type (accessor for Rust).
void nvim_textobj_set_oap_motion_type(oparg_T *oap, int type)
{
  oap->motion_type = type;
}

/// Set operator argument inclusive flag (accessor for Rust).
void nvim_textobj_set_oap_inclusive(oparg_T *oap, bool val)
{
  oap->inclusive = val;
}

/// Call oneleft() (accessor for Rust).
int nvim_textobj_oneleft(void)
{
  return oneleft();
}

/// Call incl on cursor (accessor for Rust).
int nvim_textobj_incl_cursor(void)
{
  return incl(&curwin->w_cursor);
}

/// Call decl on cursor (accessor for Rust).
int nvim_textobj_decl_cursor(void)
{
  return decl(&curwin->w_cursor);
}

/// Call redraw_curbuf_later (accessor for Rust).
void nvim_textobj_redraw_curbuf_later(int type)
{
  redraw_curbuf_later(type);
}

/// Set redraw_cmdline flag (accessor for Rust).
void nvim_textobj_set_redraw_cmdline(bool val)
{
  redraw_cmdline = val;
}

/// Save and restore cursor positions (accessor for Rust).
/// Saves current cursor, sets cursor from oap->start, then calls
/// back_in_line and cls. Returns position info needed.

/// Get cursor position as lnum/col pair (accessor for Rust).
void nvim_textobj_get_cursor_pos(int *lnum, int *col)
{
  *lnum = curwin->w_cursor.lnum;
  *col = curwin->w_cursor.col;
}

/// Set cursor position from lnum/col pair (accessor for Rust).
void nvim_textobj_set_cursor_pos(int lnum, int col)
{
  curwin->w_cursor.lnum = lnum;
  curwin->w_cursor.col = col;
}

/// Set VIsual from cursor (accessor for Rust).
void nvim_textobj_set_VIsual_from_cursor(void)
{
  VIsual = curwin->w_cursor;
}

/// Set oap->start from stored position (accessor for Rust).
void nvim_textobj_set_oap_start(oparg_T *oap, int lnum, int col)
{
  oap->start.lnum = lnum;
  oap->start.col = col;
}

// =============================================================================
// Accessors for paragraph functions
// =============================================================================

/// Get p_sections option pointer (accessor for Rust).
char *nvim_textobj_get_p_sections(void)
{
  return p_sections;
}

/// Get p_para option pointer (accessor for Rust).
char *nvim_textobj_get_p_para(void)
{
  return p_para;
}

/// Get line content at lnum (accessor for Rust).
char *nvim_textobj_ml_get(int lnum)
{
  return ml_get(lnum);
}

/// Get line length at lnum (accessor for Rust).
int nvim_textobj_ml_get_len(int lnum)
{
  return ml_get_len(lnum);
}

/// Check if line is all whitespace (accessor for Rust).
bool nvim_textobj_linewhite(int lnum)
{
  return linewhite(lnum);
}

/// Call setpcmark (accessor for Rust).
void nvim_textobj_setpcmark(void)
{
  setpcmark();
}

/// Call showmode (accessor for Rust).
void nvim_textobj_showmode(void)
{
  showmode();
}

/// Get character at position (accessor for Rust).
int nvim_textobj_gchar_pos(pos_T *pos)
{
  return gchar_pos(pos);
}

/// Increment position with incl (accessor for Rust).
int nvim_textobj_incl_pos(pos_T *pos)
{
  return incl(pos);
}

/// Decrement position with decl (accessor for Rust).
int nvim_textobj_decl_pos(pos_T *pos)
{
  return decl(pos);
}

/// Increment position with inc (accessor for Rust).
int nvim_textobj_inc_pos(pos_T *pos)
{
  return inc(pos);
}

/// Check if two positions are equal (accessor for Rust).
bool nvim_textobj_equalpos(pos_T *a, pos_T *b)
{
  return equalpos(*a, *b);
}

/// Check if position a is less than position b (accessor for Rust).
bool nvim_textobj_lt_pos(pos_T *a, pos_T *b)
{
  return lt(*a, *b);
}

/// Get p_cpo option string (accessor for Rust).
char *nvim_textobj_get_p_cpo(void)
{
  return p_cpo;
}

/// Get cursor as pos_T pointer (accessor for Rust).
pos_T *nvim_textobj_get_cursor_ptr(void)
{
  return &curwin->w_cursor;
}

/// Get VIsual as pos_T pointer (accessor for Rust).
pos_T *nvim_textobj_get_VIsual_ptr(void)
{
  return &VIsual;
}

/// Copy position from src to dst (accessor for Rust).
void nvim_textobj_copy_pos(pos_T *dst, pos_T *src)
{
  *dst = *src;
}

/// Get position lnum (accessor for Rust).
int nvim_textobj_pos_get_lnum(pos_T *pos)
{
  return (int)pos->lnum;
}

/// Get position col (accessor for Rust).
int nvim_textobj_pos_get_col(pos_T *pos)
{
  return pos->col;
}

/// Set position lnum (accessor for Rust).
void nvim_textobj_pos_set_lnum(pos_T *pos, int lnum)
{
  pos->lnum = lnum;
}

/// Set position col (accessor for Rust).
void nvim_textobj_pos_set_col(pos_T *pos, int col)
{
  pos->col = col;
}

/// Check if line is empty using LINEEMPTY macro (accessor for Rust).
bool nvim_textobj_lineempty(int lnum)
{
  return LINEEMPTY(lnum);
}

/// Check if character is ASCII whitespace (accessor for Rust).
bool nvim_textobj_ascii_iswhite(int c)
{
  return ascii_iswhite(c);
}

/// Allocate a temporary pos_T on the C side (accessor for Rust).
pos_T *nvim_textobj_alloc_pos(void)
{
  pos_T *p = xmalloc(sizeof(pos_T));
  clearpos(p);
  return p;
}

/// Free a temporary pos_T (accessor for Rust).
void nvim_textobj_free_pos(pos_T *pos)
{
  xfree(pos);
}

/// Set cursor from position (accessor for Rust).
void nvim_textobj_set_cursor_from_pos(pos_T *pos)
{
  curwin->w_cursor = *pos;
}

/// Find matching bracket (accessor for Rust).
pos_T *nvim_textobj_findmatch(int what)
{
  return findmatch(NULL, what);
}

/// Find matching bracket with limit (accessor for Rust).
pos_T *nvim_textobj_findmatchlimit(int what, int flags, int64_t maxtravel)
{
  return findmatchlimit(NULL, what, flags, maxtravel);
}

/// Check if cursor is in indent (accessor for Rust).
bool nvim_textobj_inindent(void)
{
  return inindent(1);
}

/// Increment cursor with inc_cursor (accessor for Rust).
int nvim_textobj_inc_cursor_int(void)
{
  return inc_cursor();
}

/// Increment position with inc (accessor for Rust).
int nvim_textobj_inc(pos_T *pos)
{
  return inc(pos);
}

static char *saved_p_cpo = NULL;

/// Set p_cpo temporarily (accessor for Rust).
void nvim_textobj_set_p_cpo_temp(const char *val)
{
  saved_p_cpo = p_cpo;
  p_cpo = (char *)val;
}

/// Restore p_cpo (accessor for Rust).
void nvim_textobj_restore_p_cpo(void)
{
  if (saved_p_cpo != NULL) {
    p_cpo = saved_p_cpo;
    saved_p_cpo = NULL;
  }
}

/// Check if cpo contains MATCHBSL (accessor for Rust).
bool nvim_textobj_cpo_has_matchbsl(void)
{
  return vim_strchr(p_cpo, CPO_MATCHBSL) != NULL;
}

/// Check if position a <= position b (accessor for Rust).
bool nvim_textobj_ltoreq_pos(pos_T *a, pos_T *b)
{
  return ltoreq(*a, *b);
}

/// Find the start of the next sentence, searching in the direction specified
/// by the "dir" argument.  The cursor is positioned on the start of the next
/// sentence when found.  If the next sentence is found, return OK.  Return FAIL
/// otherwise.  See ":h sentence" for the precise definition of a "sentence"
/// text object.
int findsent(Direction dir, int count)
{
  return rs_findsent(dir, count);
}

/// Find the next paragraph or section in direction 'dir'.
/// Paragraphs are currently supposed to be separated by empty lines.
/// If 'what' is NUL we go to the next paragraph.
/// If 'what' is '{' or '}' we go to the next section.
/// If 'both' is true also stop at '}'.
///
/// @param pincl  Return: true if last char is to be included
///
/// @return       true if the next paragraph or section was found.
bool findpar(bool *pincl, int dir, int count, int what, bool both)
{
  return rs_findpar(pincl, dir, count, what, both);
}

/// check if the string 's' is a nroff macro that is in option 'opt'
static bool inmacro(char *opt, const char *s)
{
  return rs_inmacro(opt, s);
}

/// startPS: return true if line 'lnum' is the start of a section or paragraph.
/// If 'para' is '{' or '}' only check for sections.
/// If 'both' is true also stop at '}'
bool startPS(linenr_T lnum, int para, bool both)
{
  return rs_startPS(lnum, para, both);
}

// The following routines do the word searches performed by the 'w', 'W',
// 'b', 'B', 'e', and 'E' commands.

// To perform these searches, characters are placed into one of three
// classes, and transitions between classes determine word boundaries.
//
// The classes are:
//
// 0 - white space
// 1 - punctuation
// 2 or higher - keyword characters (letters, digits and underscore)

static bool cls_bigword;  ///< true for "W", "B" or "E"

/// cls() - returns the class of character at curwin->w_cursor
///
/// If a 'W', 'B', or 'E' motion is being done (cls_bigword == true), chars
/// from class 2 and higher are reported as class 1 since only white space
/// boundaries are of interest.
static int cls(void)
{
  return rs_cls(cls_bigword);
}

/// fwd_word(count, type, eol) - move forward one word
///
/// @return  FAIL if the cursor was already at the end of the file.
/// If eol is true, last word stops at end of line (for operators).
///
/// @param bigword  "W", "E" or "B"
int fwd_word(int count, bool bigword, bool eol)
{
  cls_bigword = bigword;
  return rs_fwd_word(count, bigword, eol);
}

/// bck_word() - move backward 'count' words
///
/// If stop is true and we are already on the start of a word, move one less.
///
/// Returns FAIL if top of the file was reached.
int bck_word(int count, bool bigword, bool stop)
{
  cls_bigword = bigword;
  return rs_bck_word(count, bigword, stop);
}

/// end_word() - move to the end of the word
///
/// There is an apparent bug in the 'e' motion of the real vi. At least on the
/// System V Release 3 version for the 80386. Unlike 'b' and 'w', the 'e'
/// motion crosses blank lines. When the real vi crosses a blank line in an
/// 'e' motion, the cursor is placed on the FIRST character of the next
/// non-blank line. The 'E' command, however, works correctly. Since this
/// appears to be a bug, I have not duplicated it here.
///
/// Returns FAIL if end of the file was reached.
///
/// If stop is true and we are already on the end of a word, move one less.
/// If empty is true stop on an empty line.
int end_word(int count, bool bigword, bool stop, bool empty)
{
  cls_bigword = bigword;
  return rs_end_word(count, bigword, stop, empty);
}

/// Move back to the end of the word.
///
/// @param bigword  true for "B"
/// @param eol      if true, then stop at end of line.
///
/// @return         FAIL if start of the file was reached.
int bckend_word(int count, bool bigword, bool eol)
{
  cls_bigword = bigword;
  return rs_bckend_word(count, bigword, eol);
}

/// Skip a row of characters of the same class.
///
/// @return  true when end-of-file reached, false otherwise.
static bool skip_chars(int cclass, int dir)
{
  return rs_skip_chars(cclass, dir, cls_bigword);
}

/// Go back to the start of the word or the start of white space
static void back_in_line(void)
{
  rs_back_in_line(cls_bigword);
}

static void find_first_blank(pos_T *posp)
{
  rs_find_first_blank(posp);
}

/// Skip count/2 sentences and count/2 separating white spaces.
///
/// @param at_start_sent  cursor is at start of sentence
static void findsent_forward(int count, bool at_start_sent)
{
  rs_findsent_forward(count, at_start_sent);
}

/// Find word under cursor, cursor at end.
/// Used while an operator is pending, and in Visual mode.
///
/// @param include  true: include word and white space
/// @param bigword  false == word, true == WORD
int current_word(oparg_T *oap, int count, bool include, bool bigword)
{
  cls_bigword = bigword;
  return rs_current_word(oap, count, include, bigword);
}

/// Find sentence(s) under the cursor, cursor at end.
/// When Visual active, extend it by one or more sentences.
int current_sent(oparg_T *oap, int count, bool include)
{
  return rs_current_sent(oap, count, include);
}

/// Find block under the cursor, cursor at end.
/// "what" and "other" are two matching parenthesis/brace/etc.
///
/// @param include  true == include white space
/// @param what     '(', '{', etc.
/// @param other    ')', '}', etc.
int current_block(oparg_T *oap, int count, bool include, int what, int other)
{
  return rs_current_block(oap, count, include, what, other);
}

/// @param end_tag  when true, return true if the cursor is on "</aaa>".
///
/// @return         true if the cursor is on a "<aaa>" tag.  Ignore "<aaa/>".
static bool in_html_tag(bool end_tag)
{
  char *line = get_cursor_line_ptr();
  char *p;
  int lc = NUL;
  pos_T pos;

  for (p = line + curwin->w_cursor.col; p > line;) {
    if (*p == '<') {           // find '<' under/before cursor
      break;
    }
    MB_PTR_BACK(line, p);
    if (*p == '>') {           // find '>' before cursor
      break;
    }
  }
  if (*p != '<') {
    return false;
  }

  pos.lnum = curwin->w_cursor.lnum;
  pos.col = (colnr_T)(p - line);

  MB_PTR_ADV(p);
  if (end_tag) {
    // check that there is a '/' after the '<'
    return *p == '/';
  }

  // check that there is no '/' after the '<'
  if (*p == '/') {
    return false;
  }

  // check that the matching '>' is not preceded by '/'
  while (true) {
    if (inc(&pos) < 0) {
      return false;
    }
    int c = (uint8_t)(*ml_get_pos(&pos));
    if (c == '>') {
      break;
    }
    lc = c;
  }
  return lc != '/';
}

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

/// @param include  true == include white space
/// @param type     'p' for paragraph, 'S' for section
int current_par(oparg_T *oap, int count, bool include, int type)
{
  return rs_current_par(oap, count, include, type);
}

/// Search quote char from string line[col].
/// Quote character escaped by one of the characters in "escape" is not counted
/// as a quote.
///
/// @param escape  escape characters, can be NULL
///
/// @return        column number of "quotechar" or -1 when not found.
static int find_next_quote(char *line, int col, int quotechar, char *escape)
{
  return rs_find_next_quote(line, col, quotechar, escape);
}

/// Search backwards in "line" from column "col_start" to find "quotechar".
/// Quote character escaped by one of the characters in "escape" is not counted
/// as a quote.
///
/// @param escape  escape characters, can be NULL
///
/// @return        the found column or zero.
static int find_prev_quote(char *line, int col_start, int quotechar, char *escape)
{
  return rs_find_prev_quote(line, col_start, quotechar, escape);
}

/// Find quote under the cursor, cursor at end.
///
/// @param include    true == include quote char
/// @param quotechar  Quote character
///
/// @return           true if found, else false.
bool current_quote(oparg_T *oap, int count, bool include, int quotechar)
  FUNC_ATTR_NONNULL_ALL
{
  char *line = get_cursor_line_ptr();
  int col_end;
  int col_start = curwin->w_cursor.col;
  bool inclusive = false;
  bool vis_empty = true;                // Visual selection <= 1 char
  bool vis_bef_curs = false;            // Visual starts before cursor
  bool did_exclusive_adj = false;       // adjusted pos for 'selection'
  bool inside_quotes = false;           // Looks like "i'" done before
  bool selected_quote = false;          // Has quote inside selection
  int i;
  bool restore_vis_bef = false;         // restore VIsual on abort

  // When 'selection' is "exclusive" move the cursor to where it would be
  // with 'selection' "inclusive", so that the logic is the same for both.
  // The cursor then is moved forward after adjusting the area.
  if (VIsual_active) {
    // this only works within one line
    if (VIsual.lnum != curwin->w_cursor.lnum) {
      return false;
    }

    vis_bef_curs = lt(VIsual, curwin->w_cursor);
    vis_empty = equalpos(VIsual, curwin->w_cursor);
    if (*p_sel == 'e') {
      if (vis_bef_curs) {
        dec_cursor();
        did_exclusive_adj = true;
      } else if (!vis_empty) {
        dec(&VIsual);
        did_exclusive_adj = true;
      }
      vis_empty = equalpos(VIsual, curwin->w_cursor);
      if (!vis_bef_curs && !vis_empty) {
        // VIsual needs to be start of Visual selection.
        pos_T t = curwin->w_cursor;

        curwin->w_cursor = VIsual;
        VIsual = t;
        vis_bef_curs = true;
        restore_vis_bef = true;
      }
    }
  }

  if (!vis_empty) {
    // Check if the existing selection exactly spans the text inside
    // quotes.
    if (vis_bef_curs) {
      inside_quotes = VIsual.col > 0
                      && (uint8_t)line[VIsual.col - 1] == quotechar
                      && line[curwin->w_cursor.col] != NUL
                      && (uint8_t)line[curwin->w_cursor.col + 1] == quotechar;
      i = VIsual.col;
      col_end = curwin->w_cursor.col;
    } else {
      inside_quotes = curwin->w_cursor.col > 0
                      && (uint8_t)line[curwin->w_cursor.col - 1] == quotechar
                      && line[VIsual.col] != NUL
                      && (uint8_t)line[VIsual.col + 1] == quotechar;
      i = curwin->w_cursor.col;
      col_end = VIsual.col;
    }

    // Find out if we have a quote in the selection.
    while (i <= col_end) {
      // check for going over the end of the line, which can happen if
      // the line was changed after the Visual area was selected.
      if (line[i] == NUL) {
        break;
      }
      if ((uint8_t)line[i++] == quotechar) {
        selected_quote = true;
        break;
      }
    }
  }

  if (!vis_empty && (uint8_t)line[col_start] == quotechar) {
    // Already selecting something and on a quote character.  Find the
    // next quoted string.
    if (vis_bef_curs) {
      // Assume we are on a closing quote: move to after the next
      // opening quote.
      col_start = find_next_quote(line, col_start + 1, quotechar, NULL);
      if (col_start < 0) {
        goto abort_search;
      }
      col_end = find_next_quote(line, col_start + 1, quotechar, curbuf->b_p_qe);
      if (col_end < 0) {
        // We were on a starting quote perhaps?
        col_end = col_start;
        col_start = curwin->w_cursor.col;
      }
    } else {
      col_end = find_prev_quote(line, col_start, quotechar, NULL);
      if ((uint8_t)line[col_end] != quotechar) {
        goto abort_search;
      }
      col_start = find_prev_quote(line, col_end, quotechar, curbuf->b_p_qe);
      if ((uint8_t)line[col_start] != quotechar) {
        // We were on an ending quote perhaps?
        col_start = col_end;
        col_end = curwin->w_cursor.col;
      }
    }
  } else if ((uint8_t)line[col_start] == quotechar || !vis_empty) {
    int first_col = col_start;

    if (!vis_empty) {
      if (vis_bef_curs) {
        first_col = find_next_quote(line, col_start, quotechar, NULL);
      } else {
        first_col = find_prev_quote(line, col_start, quotechar, NULL);
      }
    }
    // The cursor is on a quote, we don't know if it's the opening or
    // closing quote.  Search from the start of the line to find out.
    // Also do this when there is a Visual area, a' may leave the cursor
    // in between two strings.
    col_start = 0;
    while (true) {
      // Find open quote character.
      col_start = find_next_quote(line, col_start, quotechar, NULL);
      if (col_start < 0 || col_start > first_col) {
        goto abort_search;
      }
      // Find close quote character.
      col_end = find_next_quote(line, col_start + 1, quotechar, curbuf->b_p_qe);
      if (col_end < 0) {
        goto abort_search;
      }
      // If is cursor between start and end quote character, it is
      // target text object.
      if (col_start <= first_col && first_col <= col_end) {
        break;
      }
      col_start = col_end + 1;
    }
  } else {
    // Search backward for a starting quote.
    col_start = find_prev_quote(line, col_start, quotechar, curbuf->b_p_qe);
    if ((uint8_t)line[col_start] != quotechar) {
      // No quote before the cursor, look after the cursor.
      col_start = find_next_quote(line, col_start, quotechar, NULL);
      if (col_start < 0) {
        goto abort_search;
      }
    }

    // Find close quote character.
    col_end = find_next_quote(line, col_start + 1, quotechar, curbuf->b_p_qe);
    if (col_end < 0) {
      goto abort_search;
    }
  }

  // When "include" is true, include spaces after closing quote or before
  // the starting quote.
  if (include) {
    if (ascii_iswhite(line[col_end + 1])) {
      while (ascii_iswhite(line[col_end + 1])) {
        col_end++;
      }
    } else {
      while (col_start > 0 && ascii_iswhite(line[col_start - 1])) {
        col_start--;
      }
    }
  }

  // Set start position.  After vi" another i" must include the ".
  // For v2i" include the quotes.
  if (!include && count < 2 && (vis_empty || !inside_quotes)) {
    col_start++;
  }
  curwin->w_cursor.col = col_start;
  if (VIsual_active) {
    // Set the start of the Visual area when the Visual area was empty, we
    // were just inside quotes or the Visual area didn't start at a quote
    // and didn't include a quote.
    if (vis_empty
        || (vis_bef_curs
            && !selected_quote
            && (inside_quotes
                || ((uint8_t)line[VIsual.col] != quotechar
                    && (VIsual.col == 0
                        || (uint8_t)line[VIsual.col - 1] != quotechar))))) {
      VIsual = curwin->w_cursor;
      redraw_curbuf_later(UPD_INVERTED);
    }
  } else {
    oap->start = curwin->w_cursor;
    oap->motion_type = kMTCharWise;
  }

  // Set end position.
  curwin->w_cursor.col = col_end;
  if ((include || count > 1
       // After vi" another i" must include the ".
       || (!vis_empty && inside_quotes)) && inc_cursor() == 2) {
    inclusive = true;
  }
  if (VIsual_active) {
    if (vis_empty || vis_bef_curs) {
      // decrement cursor when 'selection' is not exclusive
      if (*p_sel != 'e') {
        dec_cursor();
      }
    } else {
      // Cursor is at start of Visual area.  Set the end of the Visual
      // area when it was just inside quotes or it didn't end at a
      // quote.
      if (inside_quotes
          || (!selected_quote
              && (uint8_t)line[VIsual.col] != quotechar
              && (line[VIsual.col] == NUL
                  || (uint8_t)line[VIsual.col + 1] != quotechar))) {
        dec_cursor();
        VIsual = curwin->w_cursor;
      }
      curwin->w_cursor.col = col_start;
    }
    if (VIsual_mode == 'V') {
      VIsual_mode = 'v';
      redraw_cmdline = true;                    // show mode later
    }
  } else {
    // Set inclusive and other oap's flags.
    oap->inclusive = inclusive;
  }

  return true;

abort_search:
  if (VIsual_active && *p_sel == 'e') {
    if (did_exclusive_adj) {
      inc_cursor();
    }
    if (restore_vis_bef) {
      pos_T t = curwin->w_cursor;

      curwin->w_cursor = VIsual;
      VIsual = t;
    }
  }
  return false;
}
