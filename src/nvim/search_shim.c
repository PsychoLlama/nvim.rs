// search_shim.c: C accessor functions and remaining logic for search module
// (migrated from search.c; logic functions will be moved to Rust in future phases)

#include <assert.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cmdhist.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_getln.h"
#include "nvim/file_search.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/indent_c.h"
#include "nvim/insexpand.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/input.h"
#include "nvim/os/time.h"
#include "nvim/path.h"
#include "nvim/plines.h"
#include "nvim/profile.h"
#include "nvim/regexp.h"
#include "nvim/search.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/tag.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

#include "search_shim.c.generated.h"
extern int rs_win_valid(win_T *win);

static const char e_search_hit_top_without_match_for_str[]
  = N_("E384: Search hit TOP without match for: %s");
static const char e_search_hit_bottom_without_match_for_str[]
  = N_("E385: Search hit BOTTOM without match for: %s");

//  This file contains various searching-related routines. These fall into
//  three groups:
//  1. string searches (for /, ?, n, and N)
//  2. character searches within a single line (for f, F, t, T, etc)
//  3. "other" kinds of searches like the '%' command, and 'word' searches.
//
//
//  String searches
//
//  The string search functions are divided into two levels:
//  lowest:  searchit(); uses a pos_T for starting position and found match.
//  Highest: do_search(); uses curwin->w_cursor; calls searchit().
//
//  The last search pattern is remembered for repeating the same search.
//  This pattern is shared between the :g, :s, ? and / commands.
//  This is in search_regcomp().
//
//  The actual string matching is done using a heavily modified version of
//  Henry Spencer's regular expression library.  See regexp.c.
//
//
//
// Two search patterns are remembered: One for the :substitute command and
// one for other searches.  last_idx points to the one that was used the last
// time.

// spats[2] and last_idx now live in Rust: search_state::SPATS and LAST_IDX

// Rust FFI declarations for character search state
extern int rs_magic_isset(void);
extern int rs_is_search_forward(void);

// Rust FFI declaration for find_pattern_in_path
extern void rs_find_pattern_in_path(const char *ptr, int dir, size_t len,
                                    int whole, int skip_comments,
                                    int type, int count, int action,
                                    linenr_T start_lnum, linenr_T end_lnum,
                                    int forceit, int silent);

// Rust FFI declarations for pattern utilities
extern int rs_compl_status_adding(void);
extern int rs_compl_status_sol(void);
extern int rs_ins_compl_len(void);
extern int rs_ins_compl_interrupted(void);
extern char *rs_find_word_start(char *ptr);
extern char *rs_find_word_end(char *ptr);

// Rust FFI declaration for Phase 9
extern void rs_searchcount_compute(int pos_lnum, int pos_col, int pos_coladd,
                                    int maxcount, int timeout, bool recompute,
                                    const char *pattern, searchstat_T *stat);


/// Check if line 'lnum' in curbuf is empty or has only white chars (accessor for Rust).
/// Returns a pointer past whitespace in the line.
char *nvim_search_skipwhite_ml_get(linenr_T lnum)
{
  return skipwhite(ml_get(lnum));
}

// All spats[], saved_spats[], mr_pattern, last_idx, and their accessor functions
// have been deleted. These now live in Rust: search_state::SPATS, SAVED_SPATS,
// MR_PATTERN, LAST_IDX, and associated accessor functions.

/// Call set_vv_searchforward (wrapper for Rust).
/// Reads search direction from Rust-owned state via rs_is_search_forward().
void nvim_call_set_vv_searchforward(void)
{
  // Direction now lives in Rust SPATS; read it via rs_is_search_forward()
  set_vim_var_nr(VV_SEARCHFORWARD, rs_is_search_forward());
}

// Type used by find_pattern_in_path() to remember which included files have
// been searched already.
typedef struct {
  FILE *fp;              // File pointer
  char *name;            // Full name of file
  linenr_T lnum;                // Line we were up to in file
  int matched;                  // Found a match in this file
} SearchedFile;

/// translate search pattern for vim_regcomp()
///
/// pat_save == RE_SEARCH: save pat in spats[RE_SEARCH].pat (normal search cmd)
/// pat_save == RE_SUBST: save pat in spats[RE_SUBST].pat (:substitute command)
/// pat_save == RE_BOTH: save pat in both patterns (:global command)
/// pat_use  == RE_SEARCH: use previous search pattern if "pat" is NULL
/// pat_use  == RE_SUBST: use previous substitute pattern if "pat" is NULL
/// pat_use  == RE_LAST: use last used pattern if "pat" is NULL
/// options & SEARCH_HIS: put search string in history
/// options & SEARCH_KEEP: keep previous search pattern
///
/// @param regmatch  return: pattern and ignore-case flag
///
/// @return          FAIL if failed, OK otherwise.

// save_level, free_spat, saved_last_search_spat, did_save_last_search_spat,
// and related statics and accessors deleted: now live in Rust search_state module.

/// Get the search_match_lines global (accessor for Rust).
int nvim_get_search_match_lines(void)
{
  return (int)search_match_lines;
}

/// Get the search_match_endcol global (accessor for Rust).
int nvim_get_search_match_endcol(void)
{
  return (int)search_match_endcol;
}

/// Set the search_match_lines global (setter for Rust).
void nvim_set_search_match_lines(int val)
{
  search_match_lines = (linenr_T)val;
}

/// Set the search_match_endcol global (setter for Rust).
void nvim_set_search_match_endcol(int val)
{
  search_match_endcol = (colnr_T)val;
}

/// Get the p_is option (incsearch) for Rust.
int nvim_get_p_is(void)
{
  return p_is ? 1 : 0;
}

/// Set highlight_match global (for Rust incsearch).
void nvim_set_highlight_match(int value)
{
  highlight_match = value != 0;
}

/// Get search_first_line global (for Rust incsearch).
linenr_T nvim_get_search_first_line(void)
{
  return search_first_line;
}

/// Set search_first_line global (for Rust incsearch).
void nvim_set_search_first_line(linenr_T value)
{
  search_first_line = value;
}

/// Get search_last_line global (for Rust incsearch).
linenr_T nvim_get_search_last_line(void)
{
  return search_last_line;
}

/// Set search_last_line global (for Rust incsearch).
void nvim_set_search_last_line(linenr_T value)
{
  search_last_line = value;
}

/// Get the no_hlsearch global (accessor for Rust).
int nvim_get_no_hlsearch(void)
{
  return no_hlsearch ? 1 : 0;
}

/// Get the no_smartcase global (accessor for Rust).
int nvim_get_no_smartcase(void)
{
  return no_smartcase ? 1 : 0;
}

/// Set the no_smartcase global (setter for Rust).
void nvim_set_no_smartcase(int val)
{
  no_smartcase = val != 0;
}

/// Get the p_scs option (smartcase) for Rust.
int nvim_get_p_scs(void)
{
  return p_scs ? 1 : 0;
}

/// Get the p_ws option (wrapscan) for Rust.
int nvim_get_p_ws(void)
{
  return p_ws ? 1 : 0;
}

/// Get the p_hls option (hlsearch) for Rust.
int nvim_get_p_hls(void)
{
  return p_hls ? 1 : 0;
}

/// Save and restore the search pattern for incremental highlight search
/// feature.
///
/// It's similar to but different from save_search_patterns() and
/// restore_search_patterns(), because the search pattern must be restored when
/// cancelling incremental searching even if it's called inside user functions.












/// Lowest level search function.
/// Search for 'count'th occurrence of pattern "pat" in direction "dir".
/// Start at position "pos" and return the found position in "pos".
///
/// if (options & SEARCH_MSG) == 0 don't give any messages
/// if (options & SEARCH_MSG) == SEARCH_NFMSG don't give 'notfound' messages
/// if (options & SEARCH_MSG) == SEARCH_MSG give all messages
/// if (options & SEARCH_HIS) put search pattern in history
/// if (options & SEARCH_END) return position at end of match
/// if (options & SEARCH_START) accept match at pos itself
/// if (options & SEARCH_KEEP) keep previous search pattern
/// if (options & SEARCH_FOLD) match only once in a closed fold
/// if (options & SEARCH_PEEK) check for typed char, cancel search
/// if (options & SEARCH_COL) start at pos->col instead of zero
///
/// @param win        window to search in; can be NULL for a buffer without a window!
/// @param end_pos    set to end of the match, unless NULL
/// @param pat_use    which pattern to use when "pat" is empty
/// @param extra_arg  optional extra arguments, can be NULL
///
/// @returns          FAIL (zero) for failure, non-zero for success.
///                   the index of the first matching
///                   subpattern plus one; one if there was none.




// set_vv_searchforward deleted: direction now lives in Rust SPATS.



// Character Searches

/// Search for a character in a line.  If "t_cmd" is false, move to the
/// position of the character, otherwise move to just before the char.
/// Do this "cap->count1" times.
/// Return FAIL or OK.

// "Other" Searches


// check_prevcol(), find_rawstring_end(), and find_mps_values() have been
// migrated to Rust in search/src/matchparen.rs

// findmatchlimit -- find the matching paren or brace, if it exists within
// maxtravel lines of the cursor.  A maxtravel of 0 means search until falling
// off the edge of the file.
//
// "initc" is the character to find a match for.  NUL means to find the
// character at or after the cursor. Special values:
// '*'  look for C-style comment / *
// '/'  look for C-style comment / *, ignoring comment-end
// '#'  look for preprocessor directives
// 'R'  look for raw string start: R"delim(text)delim" (only backwards)
//
// flags: FM_BACKWARD search backwards (when initc is '/', '*' or '#')
//    FM_FORWARD  search forwards (when initc is '/', '*' or '#')
//    FM_BLOCKSTOP  stop at start/end of block ({ or } in column 0)
//    FM_SKIPCOMM skip comments (not implemented yet!)
//
// "oap" is only used to set oap->motion_type for a linewise motion, it can be
// NULL




/// Move cursor briefly to character matching the one under the cursor.
/// Used for Insert mode and "r" command.
/// Show the match only if it is visible on the screen.
/// If there isn't a match, then beep.
///
/// @param c  char to show match for
/// Perform the cursor-display-delay loop for showmatch.
/// Called from Rust after the match position has been determined.
void nvim_showmatch_display_cursor(int match_lnum, int match_col, int match_coladd)
{
  OptInt *so = curwin->w_p_so >= 0 ? &curwin->w_p_so : &p_so;
  OptInt *siso = curwin->w_p_siso >= 0 ? &curwin->w_p_siso : &p_siso;

  pos_T mpos = { match_lnum, match_col, match_coladd };
  pos_T save_cursor = curwin->w_cursor;
  OptInt save_so = *so;
  OptInt save_siso = *siso;
  // Handle "$" in 'cpo': If the ')' is typed on top of the "$",
  // stop displaying the "$".
  if (dollar_vcol >= 0 && dollar_vcol == curwin->w_virtcol) {
    dollar_vcol = -1;
  }
  curwin->w_virtcol++;              // do display ')' just before "$"

  colnr_T save_dollar_vcol = dollar_vcol;
  int save_state = State;
  State = MODE_SHOWMATCH;
  ui_cursor_shape();                // may show different cursor shape
  curwin->w_cursor = mpos;          // move to matching char
  *so = 0;                          // don't use 'scrolloff' here
  *siso = 0;                        // don't use 'sidescrolloff' here
  show_cursor_info_later(false);
  update_screen();                  // show the new char
  setcursor();
  ui_flush();
  // Restore dollar_vcol(), because setcursor() may call curs_rows()
  // which resets it if the matching position is in a previous line
  // and has a higher column number.
  dollar_vcol = save_dollar_vcol;

  // brief pause, unless 'm' is present in 'cpo' and a character is
  // available.
  if (vim_strchr(p_cpo, CPO_SHOWMATCH) != NULL) {
    os_delay((uint64_t)p_mat * 100 + 8, true);
  } else if (!char_avail()) {
    os_delay((uint64_t)p_mat * 100 + 9, false);
  }
  curwin->w_cursor = save_cursor;           // restore cursor position
  *so = save_so;
  *siso = save_siso;
  State = save_state;
  ui_cursor_shape();                // may show different cursor shape
}

// "searchcount()" function
void f_searchcount(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  pos_T pos = curwin->w_cursor;
  char *pattern = NULL;
  int maxcount = (int)p_msc;
  int timeout = SEARCH_STAT_DEF_TIMEOUT;
  bool recompute = true;
  searchstat_T stat;

  tv_dict_alloc_ret(rettv);

  if (shortmess(SHM_SEARCHCOUNT)) {  // 'shortmess' contains 'S' flag
    recompute = true;
  }

  if (argvars[0].v_type != VAR_UNKNOWN) {
    dict_T *dict;
    dictitem_T *di;
    bool error = false;

    if (tv_check_for_nonnull_dict_arg(argvars, 0) == FAIL) {
      return;
    }
    dict = argvars[0].vval.v_dict;
    di = tv_dict_find(dict, "timeout", -1);
    if (di != NULL) {
      timeout = (int)tv_get_number_chk(&di->di_tv, &error);
      if (error) {
        return;
      }
    }
    di = tv_dict_find(dict, "maxcount", -1);
    if (di != NULL) {
      maxcount = (int)tv_get_number_chk(&di->di_tv, &error);
      if (error) {
        return;
      }
    }
    di = tv_dict_find(dict, "recompute", -1);
    if (di != NULL) {
      recompute = tv_get_number_chk(&di->di_tv, &error);
      if (error) {
        return;
      }
    }
    di = tv_dict_find(dict, "pattern", -1);
    if (di != NULL) {
      pattern = (char *)tv_get_string_chk(&di->di_tv);
      if (pattern == NULL) {
        return;
      }
    }
    di = tv_dict_find(dict, "pos", -1);
    if (di != NULL) {
      if (di->di_tv.v_type != VAR_LIST) {
        semsg(_(e_invarg2), "pos");
        return;
      }
      if (tv_list_len(di->di_tv.vval.v_list) != 3) {
        semsg(_(e_invarg2), "List format should be [lnum, col, off]");
        return;
      }
      listitem_T *li = tv_list_find(di->di_tv.vval.v_list, 0);
      if (li != NULL) {
        pos.lnum = (linenr_T)tv_get_number_chk(TV_LIST_ITEM_TV(li), &error);
        if (error) {
          return;
        }
      }
      li = tv_list_find(di->di_tv.vval.v_list, 1);
      if (li != NULL) {
        pos.col = (colnr_T)tv_get_number_chk(TV_LIST_ITEM_TV(li), &error) - 1;
        if (error) {
          return;
        }
      }
      li = tv_list_find(di->di_tv.vval.v_list, 2);
      if (li != NULL) {
        pos.coladd = (colnr_T)tv_get_number_chk(TV_LIST_ITEM_TV(li), &error);
        if (error) {
          return;
        }
      }
    }
  }

  // Rust handles: save/restore patterns, pattern setup, stat computation
  rs_searchcount_compute(pos.lnum, pos.col, pos.coladd,
                          maxcount, timeout, recompute, pattern, &stat);

  tv_dict_add_nr(rettv->vval.v_dict, S_LEN("current"), stat.cur);
  tv_dict_add_nr(rettv->vval.v_dict, S_LEN("total"), stat.cnt);
  tv_dict_add_nr(rettv->vval.v_dict, S_LEN("exact_match"), stat.exact_match);
  tv_dict_add_nr(rettv->vval.v_dict, S_LEN("incomplete"), stat.incomplete);
  tv_dict_add_nr(rettv->vval.v_dict, S_LEN("maxcount"), stat.last_maxcount);
}

/// Get line "lnum" and copy it into "buf[LSIZE]".
/// The copy is made because the regexp may make the line invalid when using a
/// mark.
static char *get_line_and_copy(linenr_T lnum, char *buf)
{
  char *line = ml_get(lnum);
  xstrlcpy(buf, line, LSIZE);
  return buf;
}

/// Find identifiers or defines in included files.
/// If p_ic && rs_compl_status_sol() then ptr must be in lowercase.
///
/// @param ptr            pointer to search pattern
/// @param dir            direction of expansion
/// @param len            length of search pattern
/// @param whole          match whole words only
/// @param skip_comments  don't match inside comments
/// @param type           Type of search; are we looking for a type? a macro?
/// @param action         What to do when we find it
/// @param start_lnum     first line to start searching
/// @param end_lnum       last line for searching
/// @param forceit        If true, always switch to the found path
/// @param silent         Do not print messages when ACTION_EXPAND
void find_pattern_in_path(char *ptr, Direction dir, size_t len, bool whole, bool skip_comments,
                          int type, int count, int action, linenr_T start_lnum, linenr_T end_lnum,
                          bool forceit, bool silent)
{
  rs_find_pattern_in_path(ptr, dir, len,
                          whole ? 1 : 0, skip_comments ? 1 : 0,
                          type, count, action,
                          start_lnum, end_lnum,
                          forceit ? 1 : 0, silent ? 1 : 0);
}

// =========================================================================
// Phase 4: Batch C helpers for find_pattern_in_path
// =========================================================================

/// Opaque state for the find_pattern_in_path operation.
typedef struct {
  SearchedFile *files;
  int max_path_depth;
  int old_files;
  int depth;
  int depth_displayed;
  int match_count;
  Direction dir;
  char *ptr;
  size_t len;
  bool whole;
  bool skip_comments;
  int type;
  int count;
  int action;
  linenr_T start_lnum;
  linenr_T end_lnum;
  bool forceit;
  bool silent;
  char *file_line;
  char *curr_fname;
  char *prev_fname;
  regmatch_T regmatch;
  regmatch_T incl_regmatch;
  regmatch_T def_regmatch;
  char *inc_opt;
  bool did_show;
  bool found;
  linenr_T lnum;
  int l_g_do_tagpreview;
} FpipState;

/// Initialize the fpip state.
FpipInitResult nvim_fpip_init(const char *ptr, int dir, size_t len,
                              int whole, int skip_comments,
                              int type, int count, int action,
                              linenr_T start_lnum, linenr_T end_lnum,
                              int forceit, int silent)
{
  FpipState *st = xcalloc(1, sizeof(FpipState));
  st->max_path_depth = 50;
  st->match_count = 1;
  st->ptr = (char *)ptr;
  st->dir = (Direction)dir;
  st->len = len;
  st->whole = whole != 0;
  st->skip_comments = skip_comments != 0;
  st->type = type;
  st->count = count;
  st->action = action;
  st->start_lnum = start_lnum;
  st->end_lnum = end_lnum;
  st->forceit = forceit != 0;
  st->silent = silent != 0;
  st->curr_fname = curbuf->b_fname;
  st->prev_fname = NULL;
  st->did_show = false;
  st->found = false;
  st->l_g_do_tagpreview = g_do_tagpreview;
  st->depth = -1;
  st->depth_displayed = -1;

  st->regmatch.regprog = NULL;
  st->incl_regmatch.regprog = NULL;
  st->def_regmatch.regprog = NULL;

  st->file_line = xmalloc(LSIZE);

  if (type != CHECK_PATH && type != FIND_DEFINE
      && !rs_compl_status_sol()) {
    size_t patsize = len + 5;
    char *pat = xmalloc(patsize);
    assert(len <= INT_MAX);
    snprintf(pat, patsize, st->whole ? "\\<%.*s\\>" : "%.*s", (int)len, ptr);
    st->regmatch.rm_ic = ignorecase(pat);
    st->regmatch.regprog = vim_regcomp(pat, rs_magic_isset() ? RE_MAGIC : 0);
    xfree(pat);
    if (st->regmatch.regprog == NULL) {
      return (FpipInitResult){ st, 0 };
    }
  }

  st->inc_opt = (*curbuf->b_p_inc == NUL) ? p_inc : curbuf->b_p_inc;
  if (*st->inc_opt != NUL) {
    st->incl_regmatch.regprog = vim_regcomp(st->inc_opt, rs_magic_isset() ? RE_MAGIC : 0);
    if (st->incl_regmatch.regprog == NULL) {
      return (FpipInitResult){ st, 0 };
    }
    st->incl_regmatch.rm_ic = false;
  }

  if (type == FIND_DEFINE && (*curbuf->b_p_def != NUL || *p_def != NUL)) {
    st->def_regmatch.regprog = vim_regcomp(
        *curbuf->b_p_def == NUL ? p_def : curbuf->b_p_def,
        rs_magic_isset() ? RE_MAGIC : 0);
    if (st->def_regmatch.regprog == NULL) {
      return (FpipInitResult){ st, 0 };
    }
    st->def_regmatch.rm_ic = false;
  }

  st->files = xcalloc((size_t)st->max_path_depth, sizeof(SearchedFile));
  st->old_files = st->max_path_depth;

  st->end_lnum = MIN(st->end_lnum, curbuf->b_ml.ml_line_count);
  st->lnum = MIN(st->start_lnum, st->end_lnum);

  return (FpipInitResult){ st, 1 };
}

/// Run the main search loop. Contains the entire original while(true) loop
/// and all the match/action handling.
void nvim_fpip_run(void *handle)
{
  FpipState *st = (FpipState *)handle;

  // Local aliases for readability (matching original variable names)
  SearchedFile *files = st->files;
  int max_path_depth = st->max_path_depth;
  int old_files = st->old_files;
  int depth = st->depth;
  int depth_displayed = st->depth_displayed;
  int match_count = st->match_count;
  char *ptr = st->ptr;
  size_t len = st->len;
  int type = st->type;
  int action = st->action;
  linenr_T end_lnum = st->end_lnum;
  linenr_T lnum = st->lnum;
  char *file_line = st->file_line;
  char *curr_fname = st->curr_fname;
  char *prev_fname = st->prev_fname;
  regmatch_T *regmatch = &st->regmatch;
  regmatch_T *incl_regmatch = &st->incl_regmatch;
  regmatch_T *def_regmatch = &st->def_regmatch;
  char *inc_opt = st->inc_opt;
  bool did_show = st->did_show;
  bool found = st->found;
  int l_g_do_tagpreview = st->l_g_do_tagpreview;

  char *new_fname;
  char *p;
  bool define_matched;
  bool matched = false;
  int i;
  char *already = NULL;
  char *startp = NULL;
  win_T *curwin_save = NULL;

  char *line = get_line_and_copy(lnum, file_line);

  while (true) {
    if (incl_regmatch->regprog != NULL
        && vim_regexec(incl_regmatch, line, 0)) {
      char *p_fname = (curr_fname == curbuf->b_fname)
                      ? curbuf->b_ffname : curr_fname;

      if (inc_opt != NULL && strstr(inc_opt, "\\zs") != NULL) {
        new_fname = find_file_name_in_path(incl_regmatch->startp[0],
                                           (size_t)(incl_regmatch->endp[0]
                                                    - incl_regmatch->startp[0]),
                                           FNAME_EXP|FNAME_INCL|FNAME_REL,
                                           1, p_fname);
      } else {
        new_fname = file_name_in_line(incl_regmatch->endp[0], 0,
                                      FNAME_EXP|FNAME_INCL|FNAME_REL, 1, p_fname,
                                      NULL);
      }
      bool already_searched = false;
      if (new_fname != NULL) {
        for (i = 0;; i++) {
          if (i == depth + 1) {
            i = old_files;
          }
          if (i == max_path_depth) {
            break;
          }
          if (path_full_compare(new_fname, files[i].name, true,
                                true) & kEqualFiles) {
            if (type != CHECK_PATH
                && action == ACTION_SHOW_ALL && files[i].matched) {
              msg_putchar('\n');
              if (!got_int) {
                msg_home_replace(new_fname);
                msg_puts(_(" (includes previously listed match)"));
                prev_fname = NULL;
              }
            }
            XFREE_CLEAR(new_fname);
            already_searched = true;
            break;
          }
        }
      }

      if (type == CHECK_PATH && (action == ACTION_SHOW_ALL
                                 || (new_fname == NULL && !already_searched))) {
        if (did_show) {
          msg_putchar('\n');
        } else {
          gotocmdline(true);
          msg_puts_title(_("--- Included files "));
          if (action != ACTION_SHOW_ALL) {
            msg_puts_title(_("not found "));
          }
          msg_puts_title(_("in path ---\n"));
        }
        did_show = true;
        while (depth_displayed < depth && !got_int) {
          depth_displayed++;
          for (i = 0; i < depth_displayed; i++) {
            msg_puts("  ");
          }
          msg_home_replace(files[depth_displayed].name);
          msg_puts(" -->\n");
        }
        if (!got_int) {
          for (i = 0; i <= depth_displayed; i++) {
            msg_puts("  ");
          }
          if (new_fname != NULL) {
            msg_outtrans(new_fname, HLF_D, false);
          } else {
            if (inc_opt != NULL
                && strstr(inc_opt, "\\zs") != NULL) {
              p = incl_regmatch->startp[0];
              i = (int)(incl_regmatch->endp[0]
                        - incl_regmatch->startp[0]);
            } else {
              for (p = incl_regmatch->endp[0];
                   *p && !vim_isfilec((uint8_t)(*p)); p++) {}
              for (i = 0; vim_isfilec((uint8_t)p[i]); i++) {}
            }

            if (i == 0) {
              p = incl_regmatch->endp[0];
              i = (int)strlen(p);
            } else if (p > line) {
              if (p[-1] == '"' || p[-1] == '<') {
                p--;
                i++;
              }
              if (p[i] == '"' || p[i] == '>') {
                i++;
              }
            }
            char save_char = p[i];
            p[i] = NUL;
            msg_outtrans(p, HLF_D, false);
            p[i] = save_char;
          }

          if (new_fname == NULL && action == ACTION_SHOW_ALL) {
            if (already_searched) {
              msg_puts(_("  (Already listed)"));
            } else {
              msg_puts(_("  NOT FOUND"));
            }
          }
        }
      }

      if (new_fname != NULL) {
        SearchedFile *bigger;
        if (depth + 1 == old_files) {
          bigger = xmalloc((size_t)max_path_depth * 2 * sizeof(SearchedFile));
          for (i = 0; i <= depth; i++) {
            bigger[i] = files[i];
          }
          for (i = depth + 1; i < old_files + max_path_depth; i++) {
            bigger[i].fp = NULL;
            bigger[i].name = NULL;
            bigger[i].lnum = 0;
            bigger[i].matched = false;
          }
          for (i = old_files; i < max_path_depth; i++) {
            bigger[i + max_path_depth] = files[i];
          }
          old_files += max_path_depth;
          max_path_depth *= 2;
          xfree(files);
          files = bigger;
        }
        if ((files[depth + 1].fp = os_fopen(new_fname, "r")) == NULL) {
          xfree(new_fname);
        } else {
          if (++depth == old_files) {
            xfree(files[old_files].name);
            old_files++;
          }
          files[depth].name = curr_fname = new_fname;
          files[depth].lnum = 0;
          files[depth].matched = false;
          if (action == ACTION_EXPAND && !shortmess(SHM_COMPLETIONSCAN) && !st->silent) {
            msg_hist_off = true;
            vim_snprintf(IObuff, IOSIZE,
                         _("Scanning included file: %s"),
                         new_fname);
            msg_trunc(IObuff, true, HLF_R);
          } else if (p_verbose >= 5) {
            verbose_enter();
            smsg(0, _("Searching included file %s"), new_fname);
            verbose_leave();
          }
        }
      }
    } else {
      p = line;
search_line:
      define_matched = false;
      if (def_regmatch->regprog != NULL
          && vim_regexec(def_regmatch, line, 0)) {
        p = def_regmatch->endp[0];
        while (*p && !vim_iswordc((uint8_t)(*p))) {
          p++;
        }
        define_matched = true;
      }

      if (def_regmatch->regprog == NULL || define_matched) {
        if (define_matched || rs_compl_status_sol()) {
          startp = skipwhite(p);
          if (p_ic) {
            matched = !mb_strnicmp(startp, ptr, len);
          } else {
            matched = !strncmp(startp, ptr, len);
          }
          if (matched && define_matched && st->whole
              && vim_iswordc((uint8_t)startp[len])) {
            matched = false;
          }
        } else if (regmatch->regprog != NULL
                   && vim_regexec(regmatch, line, (colnr_T)(p - line))) {
          matched = true;
          startp = regmatch->startp[0];
          if (st->skip_comments) {
            if ((*line != '#'
                 || strncmp(skipwhite(line + 1), "define", 6) != 0)
                && get_leader_len(line, NULL, false, true)) {
              matched = false;
            }

            p = skipwhite(line);
            if (matched
                || (p[0] == '/' && p[1] == '*') || p[0] == '*') {
              for (p = line; *p && p < startp; p++) {
                if (matched
                    && p[0] == '/'
                    && (p[1] == '*' || p[1] == '/')) {
                  matched = false;
                  if (p[1] == '/') {
                    break;
                  }
                  p++;
                } else if (!matched && p[0] == '*' && p[1] == '/') {
                  matched = true;
                  p++;
                }
              }
            }
          }
        }
      }
    }
    if (matched) {
      if (action == ACTION_EXPAND) {
        bool cont_s_ipos = false;

        if (depth == -1 && lnum == curwin->w_cursor.lnum) {
          break;
        }
        found = true;
        char *aux = p = startp;
        if (rs_compl_status_adding() && (int)strlen(p) >= rs_ins_compl_len()) {
          p += rs_ins_compl_len();
          if (vim_iswordp(p)) {
            goto exit_matched;
          }
          p = rs_find_word_start(p);
        }
        p = rs_find_word_end(p);
        i = (int)(p - aux);

        if (rs_compl_status_adding() && i == rs_ins_compl_len()) {
          strncpy(IObuff, aux, (size_t)i);  // NOLINT(runtime/printf)

          if (depth < 0) {
            if (lnum >= end_lnum) {
              goto exit_matched;
            }
            line = get_line_and_copy(++lnum, file_line);
          } else if (vim_fgets(line = file_line,
                               LSIZE, files[depth].fp)) {
            goto exit_matched;
          }

          already = aux = p = skipwhite(line);
          p = rs_find_word_start(p);
          p = rs_find_word_end(p);
          if (p > aux) {
            if (*aux != ')' && IObuff[i - 1] != TAB) {
              if (IObuff[i - 1] != ' ') {
                IObuff[i++] = ' ';
              }
              if (p_js
                  && (IObuff[i - 2] == '.'
                      || IObuff[i - 2] == '?'
                      || IObuff[i - 2] == '!')) {
                IObuff[i++] = ' ';
              }
            }
            if (p - aux >= IOSIZE - i) {
              p = aux + IOSIZE - i - 1;
            }
            strncpy(IObuff + i, aux, (size_t)(p - aux));  // NOLINT(runtime/printf)
            i += (int)(p - aux);
            cont_s_ipos = true;
          }
          IObuff[i] = NUL;
          aux = IObuff;

          if (i == rs_ins_compl_len()) {
            goto exit_matched;
          }
        }

        const int add_r = ins_compl_add_infercase(aux, i, p_ic,
                                                  curr_fname == curbuf->b_fname
                                                  ? NULL : curr_fname,
                                                  st->dir, cont_s_ipos, 0);
        if (add_r == OK) {
          st->dir = FORWARD;
        } else if (add_r == FAIL) {
          break;
        }
      } else if (action == ACTION_SHOW_ALL) {
        found = true;
        if (!did_show) {
          gotocmdline(true);
        }
        if (curr_fname != prev_fname) {
          if (did_show) {
            msg_putchar('\n');
          }
          if (!got_int) {
            msg_home_replace(curr_fname);
          }
          prev_fname = curr_fname;
        }
        did_show = true;
        if (!got_int) {
          show_pat_in_path(line, type, true, action,
                           (depth == -1) ? NULL : files[depth].fp,
                           (depth == -1) ? &lnum : &files[depth].lnum,
                           match_count++);
        }

        for (i = 0; i <= depth; i++) {
          files[i].matched = true;
        }
      } else if (--st->count <= 0) {
        found = true;
        if (depth == -1 && lnum == curwin->w_cursor.lnum
            && l_g_do_tagpreview == 0) {
          emsg(_("E387: Match is on current line"));
        } else if (action == ACTION_SHOW) {
          show_pat_in_path(line, type, did_show, action,
                           (depth == -1) ? NULL : files[depth].fp,
                           (depth == -1) ? &lnum : &files[depth].lnum, 1);
          did_show = true;
        } else {
          if (l_g_do_tagpreview != 0) {
            curwin_save = curwin;
            prepare_tagpreview(true);
          }
          if (action == ACTION_SPLIT) {
            if (win_split(0, 0) == FAIL) {
              break;
            }
            RESET_BINDING(curwin);
          }
          if (depth == -1) {
            if (l_g_do_tagpreview != 0) {
              if (!rs_win_valid(curwin_save)) {
                break;
              }
              if (!GETFILE_SUCCESS(getfile(curwin_save->w_buffer->b_fnum, NULL,
                                           NULL, true, lnum, st->forceit))) {
                break;
              }
            } else {
              setpcmark();
            }
            curwin->w_cursor.lnum = lnum;
            check_cursor(curwin);
          } else {
            if (!GETFILE_SUCCESS(getfile(0, files[depth].name, NULL, true,
                                         files[depth].lnum, st->forceit))) {
              break;
            }
            curwin->w_cursor.lnum = files[depth].lnum;
          }
        }
        if (action != ACTION_SHOW) {
          curwin->w_cursor.col = (colnr_T)(startp - line);
          curwin->w_set_curswant = true;
        }

        if (l_g_do_tagpreview != 0
            && curwin != curwin_save && rs_win_valid(curwin_save)) {
          validate_cursor(curwin);
          redraw_later(curwin, UPD_VALID);
          win_enter(curwin_save, true);
        }
        break;
      }
exit_matched:
      matched = false;
      if (def_regmatch->regprog == NULL
          && action == ACTION_EXPAND
          && !rs_compl_status_sol()
          && *startp != NUL
          && *(startp + utfc_ptr2len(startp)) != NUL) {
        goto search_line;
      }
    }
    line_breakcheck();
    if (action == ACTION_EXPAND) {
      rs_ins_compl_check_keys(30, 0);
    }
    if (got_int || rs_ins_compl_interrupted()) {
      break;
    }

    while (depth >= 0 && !already
           && vim_fgets(line = file_line, LSIZE, files[depth].fp)) {
      fclose(files[depth].fp);
      old_files--;
      files[old_files].name = files[depth].name;
      files[old_files].matched = files[depth].matched;
      depth--;
      curr_fname = (depth == -1) ? curbuf->b_fname
                                 : files[depth].name;
      depth_displayed = MIN(depth_displayed, depth);
    }
    if (depth >= 0) {
      files[depth].lnum++;
      i = (int)strlen(line);
      if (i > 0 && line[i - 1] == '\n') {
        line[--i] = NUL;
      }
      if (i > 0 && line[i - 1] == '\r') {
        line[--i] = NUL;
      }
    } else if (!already) {
      if (++lnum > end_lnum) {
        break;
      }
      line = get_line_and_copy(lnum, file_line);
    }
    already = NULL;
  }

  // Close any files still open
  for (i = 0; i <= depth; i++) {
    fclose(files[i].fp);
    xfree(files[i].name);
  }
  for (i = old_files; i < max_path_depth; i++) {
    xfree(files[i].name);
  }
  xfree(files);

  if (type == CHECK_PATH) {
    if (!did_show) {
      if (action != ACTION_SHOW_ALL) {
        msg(_("All included files were found"), 0);
      } else {
        msg(_("No included files"), 0);
      }
    }
  } else if (!found && action != ACTION_EXPAND && !st->silent) {
    if (got_int || rs_ins_compl_interrupted()) {
      emsg(_(e_interr));
    } else if (type == FIND_DEFINE) {
      emsg(_("E388: Couldn't find definition"));
    } else {
      emsg(_("E389: Couldn't find pattern"));
    }
  }
  if (action == ACTION_SHOW || action == ACTION_SHOW_ALL) {
    msg_end();
  }

  // Write back state that cleanup needs
  st->files = files;
  st->max_path_depth = max_path_depth;
  st->old_files = old_files;
  st->did_show = did_show;
  st->found = found;
}

/// Clean up fpip state.
void nvim_fpip_cleanup(void *handle)
{
  FpipState *st = (FpipState *)handle;
  xfree(st->file_line);
  vim_regfree(st->regmatch.regprog);
  vim_regfree(st->incl_regmatch.regprog);
  vim_regfree(st->def_regmatch.regprog);
  xfree(st);
}


static void show_pat_in_path(char *line, int type, bool did_show, int action, FILE *fp,
                             linenr_T *lnum, int count)
  FUNC_ATTR_NONNULL_ARG(1, 6)
{
  if (did_show) {
    msg_putchar('\n');          // cursor below last one
  } else if (!msg_silent) {
    gotocmdline(true);          // cursor at status line
  }
  if (got_int) {                // 'q' typed at "--more--" message
    return;
  }
  size_t linelen = strlen(line);
  while (true) {
    char *p = line + linelen - 1;
    if (fp != NULL) {
      // We used fgets(), so get rid of newline at end
      if (p >= line && *p == '\n') {
        p--;
      }
      if (p >= line && *p == '\r') {
        p--;
      }
      *(p + 1) = NUL;
    }
    if (action == ACTION_SHOW_ALL) {
      snprintf(IObuff, IOSIZE, "%3d: ", count);  // Show match nr.
      msg_puts(IObuff);
      snprintf(IObuff, IOSIZE, "%4" PRIdLINENR, *lnum);  // Show line nr.
      // Highlight line numbers.
      msg_puts_hl(IObuff, HLF_N, false);
      msg_puts(" ");
    }
    msg_prt_line(line, false);

    // Definition continues until line that doesn't end with '\'
    if (got_int || type != FIND_DEFINE || p < line || *p != '\\') {
      break;
    }

    if (fp != NULL) {
      if (vim_fgets(line, LSIZE, fp)) {     // end of file
        break;
      }
      linelen = strlen(line);
      (*lnum)++;
    } else {
      if (++*lnum > curbuf->b_ml.ml_line_count) {
        break;
      }
      line = ml_get(*lnum);
      linelen = (size_t)ml_get_len(*lnum);
    }
    msg_putchar('\n');
  }
}







// Batch accessors for pattern save/restore, pattern compilation, and all spat
// accessor functions have been deleted. These operations now live in the Rust
// search_state module (src/nvim-rs/search/src/search_state.rs).

// =============================================================================
// Accessor wrappers for C globals still needed by Rust crates
// =============================================================================

/// Call iemsg() for the restore mismatch error.
void nvim_call_iemsg_restore_mismatch(void)
{
  iemsg("restore_last_search_pattern() called more often than"
        " save_last_search_pattern()");
}

/// Emit "no previous substitute regular expression" error.
void nvim_emsg_nopresub(void)
{
  emsg(_(e_nopresub));
}

/// Set rc_did_emsg = true.
void nvim_set_rc_did_emsg(void)
{
  rc_did_emsg = true;
}

/// Get rc_did_emsg.
int nvim_get_rc_did_emsg(void)
{
  return rc_did_emsg;
}

/// Clear rc_did_emsg.
void nvim_clear_rc_did_emsg(void)
{
  rc_did_emsg = false;
}

/// Add search pattern to history.
void nvim_search_add_to_history(const char *pat, size_t patlen)
{
  add_to_history(HIST_SEARCH, pat, patlen, true, NUL);
}

/// Check if cmdmod has keeppatterns flag.
int nvim_get_cmdmod_keeppatterns(void)
{
  return (cmdmod.cmod_flags & CMOD_KEEPPATTERNS) != 0;
}

/// Compile regex: call vim_regcomp and set regmatch fields.
/// Returns 1 on success, 0 on failure.
int nvim_search_regcomp_compile(const char *pat, int magic, regmmatch_T *regmatch)
{
  regmatch->rmm_ic = ignorecase(pat);
  regmatch->rmm_maxcol = 0;
  regmatch->regprog = vim_regcomp(pat, magic ? RE_MAGIC : 0);
  return regmatch->regprog != NULL ? 1 : 0;
}

/// Increment emsg_off.
void nvim_inc_emsg_off(void)
{
  emsg_off++;
}

/// Decrement emsg_off.
void nvim_dec_emsg_off(void)
{
  emsg_off--;
}

// =============================================================================
// C accessor functions for rs_searchit (Phase 1)
// =============================================================================

// Constant verification
_Static_assert(MAXCOL == 0x7fffffff, "MAXCOL mismatch");
_Static_assert(CPO_SEARCH == 'c', "CPO_SEARCH mismatch");
_Static_assert(SHM_SEARCH == 's', "SHM_SEARCH mismatch");
_Static_assert(SHM_SEARCHCOUNT == 'S', "SHM_SEARCHCOUNT mismatch");
_Static_assert(SEARCH_HIS == 0x20, "SEARCH_HIS mismatch");
_Static_assert(SEARCH_KEEP == 0x400, "SEARCH_KEEP mismatch");
_Static_assert(SEARCH_MSG == 0x0c, "SEARCH_MSG mismatch");
_Static_assert(SEARCH_COL == 0x1000, "SEARCH_COL mismatch");
_Static_assert(SEARCH_END == 0x40, "SEARCH_END mismatch");
_Static_assert(SEARCH_START == 0x100, "SEARCH_START mismatch");
_Static_assert(SEARCH_NOOF == 0x80, "SEARCH_NOOF mismatch");
_Static_assert(SEARCH_PEEK == 0x800, "SEARCH_PEEK mismatch");
_Static_assert(FORWARD == 1, "FORWARD mismatch");
_Static_assert(BACKWARD == -1, "BACKWARD mismatch");
_Static_assert(FAIL == 0, "FAIL mismatch");

/// Compile search pattern for rs_searchit.
/// Returns FAIL (0) or OK (1). regmatch is an opaque handle.
int nvim_search_regcomp(char *pat, size_t patlen, int pat_use, int options, void *regmatch_out)
{
  return search_regcomp(pat, patlen, NULL, RE_SEARCH, pat_use, options, (regmmatch_T *)regmatch_out);
}

/// Execute multi-line regex match with timeout support.
int nvim_searchit_regexec_multi(void *regmatch, void *win, void *buf,
                                linenr_T lnum, colnr_T col,
                                void *tm, int *timed_out)
{
  return vim_regexec_multi((regmmatch_T *)regmatch, (win_T *)win, (buf_T *)buf,
                           lnum, col, (proftime_T *)tm, timed_out);
}

/// Free regprog from regmmatch_T.
void nvim_searchit_regfree(void *regmatch)
{
  regmmatch_T *rm = (regmmatch_T *)regmatch;
  vim_regfree(rm->regprog);
}

/// Check if regprog is NULL (pattern was freed during search).
int nvim_regmatch_regprog_is_null(const void *regmatch)
{
  return ((const regmmatch_T *)regmatch)->regprog == NULL ? 1 : 0;
}

/// Get rmm_matchcol from regmmatch_T.
colnr_T nvim_regmatch_rmm_matchcol(const void *regmatch)
{
  return ((const regmmatch_T *)regmatch)->rmm_matchcol;
}

/// Check if p_cpo contains CPO_SEARCH ('c').
int nvim_cpo_has_search(void)
{
  return vim_strchr(p_cpo, CPO_SEARCH) != NULL ? 1 : 0;
}

/// Check if profile time limit has been passed.
int nvim_profile_passed_limit(void *tm)
{
  if (tm == NULL) {
    return 0;
  }
  return profile_passed_limit(*(proftime_T *)tm) ? 1 : 0;
}

/// Emit "Pattern not found" error with the pattern from Rust-owned mr_pattern.
void nvim_searchit_emsg_patnotf(int p_ws_val, linenr_T lnum)
{
  // get_search_pat is the export_name for rs_get_mr_pattern (reads Rust MR_PATTERN)
  char *pat = get_search_pat();
  if (p_ws_val) {
    semsg(_(e_patnotf2), pat);
  } else if (lnum == 0) {
    semsg(_(e_search_hit_top_without_match_for_str), pat);
  } else {
    semsg(_(e_search_hit_bottom_without_match_for_str), pat);
  }
}

/// Emit "E383: Invalid search string" error using Rust-owned mr_pattern.
void nvim_searchit_emsg_invalid(void)
{
  semsg(_("E383: Invalid search string: %s"), get_search_pat());
}

/// Emit "Interrupted" error.
void nvim_searchit_emsg_interr(void)
{
  emsg(_(e_interr));
}

/// Give search wrap-around warning message.
void nvim_searchit_give_warning(int dir)
{
  give_warning(_(dir == BACKWARD ? top_bot_msg : bot_top_msg), true);
}

/// Allocate a regmmatch_T on the heap and return as opaque handle.
void *nvim_regmmatch_alloc(void)
{
  regmmatch_T *rm = xcalloc(1, sizeof(regmmatch_T));
  return rm;
}

/// Free a heap-allocated regmmatch_T.
void nvim_regmmatch_free(void *rm)
{
  xfree(rm);
}

// =============================================================================
// C accessor functions for rs_do_search (Phase 2)
// =============================================================================

_Static_assert(SEARCH_REV == 0x01, "SEARCH_REV mismatch");
_Static_assert(SEARCH_ECHO == 0x02, "SEARCH_ECHO mismatch");
_Static_assert(SEARCH_OPT == 0x10, "SEARCH_OPT mismatch");
_Static_assert(SEARCH_NOOF == 0x80, "SEARCH_NOOF mismatch");
_Static_assert(SEARCH_MARK == 0x200, "SEARCH_MARK mismatch");
_Static_assert(RE_SEARCH == 0, "RE_SEARCH mismatch");
_Static_assert(RE_SUBST == 1, "RE_SUBST mismatch");
_Static_assert(RE_LAST == 2, "RE_LAST mismatch");

/// Check if Rust-owned spats[0].off.line is set and CPO_LINEOFF is in p_cpo.
int nvim_do_search_check_lineoff(void)
{
  extern int rs_get_search_offset_line(int idx);
  return (rs_get_search_offset_line(0) && vim_strchr(p_cpo, CPO_LINEOFF) != NULL) ? 1 : 0;
}

/// Clear Rust-owned spats[0].off.line and off if CPO_LINEOFF applies.
void nvim_do_search_clear_lineoff(void)
{
  extern void rs_set_search_offset_line_end_off(int line, int end, int64_t off);
  extern int rs_get_search_offset_end(int idx);
  rs_set_search_offset_line_end_off(0, rs_get_search_offset_end(0), 0);
}

/// Get the dirc from Rust-owned spats[0].off.dir.
int nvim_do_search_get_dirc(void)
{
  extern char rs_search_get_dir(void);
  return (uint8_t)rs_search_get_dir();
}

/// Set Rust-owned spats[0].off.dir and update vv:searchforward.
void nvim_do_search_set_dirc(int dirc)
{
  extern void rs_search_set_dir(int dir);
  rs_search_set_dir(dirc);
}

/// Handle fold adjustment for do_search position.
/// For forward: adjusts to end of fold. For backward: adjusts to start of fold.
/// Returns the adjusted pos (lnum, col).
DoSearchPos nvim_do_search_fold_adjust(int dirc, linenr_T lnum, colnr_T col)
{
  pos_T pos;
  pos.lnum = lnum;
  pos.col = col;
  pos.coladd = 0;

  if (dirc == '/') {
    if (hasFolding(curwin, pos.lnum, NULL, &pos.lnum)) {
      pos.col = MAXCOL - 2;
    }
  } else {
    if (hasFolding(curwin, pos.lnum, &pos.lnum, NULL)) {
      pos.col = 0;
    }
  }
  return (DoSearchPos){ pos.lnum, pos.col };
}

/// Turn hlsearch back on if needed.
void nvim_do_search_hlsearch_on(int options)
{
  if (no_hlsearch && !(options & SEARCH_KEEP)) {
    redraw_all_later(UPD_SOME_VALID);
    set_no_hlsearch(false);
  }
}

/// Get previous search pattern (Rust-owned SPATS[RE_SEARCH].pat).
char *nvim_do_search_get_search_pat(void)
{
  extern const char *rs_get_search_pattern(void);
  return (char *)rs_get_search_pattern();
}

/// Get previous subst pattern (Rust-owned SPATS[RE_SUBST].pat).
char *nvim_do_search_get_subst_pat(void)
{
  extern const char *rs_get_subst_pattern(void);
  return (char *)rs_get_subst_pattern();
}

size_t nvim_do_search_get_subst_patlen(void)
{
  extern size_t rs_get_subst_pattern_len(void);
  return rs_get_subst_pattern_len();
}

/// Call skip_regexp_ex for do_search pattern parsing.
/// Returns: pointer to end of regexp.  Sets *newp if copy was made.
char *nvim_do_search_skip_regexp(char *pat, int delim, char **newp)
{
  return skip_regexp_ex(pat, delim, rs_magic_isset(), newp, NULL, NULL);
}

/// Set the searchcmdlen global.
void nvim_do_search_set_searchcmdlen(int val)
{
  searchcmdlen = val;
}

/// Get the searchcmdlen global.
int nvim_do_search_get_searchcmdlen(void)
{
  return searchcmdlen;
}

/// Set Rust-owned SPATS[0].off fields.
void nvim_do_search_set_off(int off_line, int off_end, int64_t off_off)
{
  extern void rs_set_search_offset_line_end_off(int line, int end, int64_t off);
  rs_set_search_offset_line_end_off(off_line, off_end, off_off);
}

/// Get Rust-owned SPATS[0].off.end for the SEARCH_END computation.
int nvim_do_search_get_off_end(void)
{
  extern int rs_get_search_offset_end(int idx);
  return rs_get_search_offset_end(0);
}

/// Get Rust-owned SPATS[0].off.line.
int nvim_do_search_get_off_line(void)
{
  extern int rs_get_search_offset_line(int idx);
  return rs_get_search_offset_line(0);
}

/// Get Rust-owned SPATS[0].off.off.
int64_t nvim_do_search_get_off_off(void)
{
  extern int64_t rs_get_search_offset_off(int idx);
  return rs_get_search_offset_off(0);
}

/// Batch helper: handle echo/display section of do_search.
/// Returns: msgbuf (allocated), sets *msgbuflen_out, sets *show_search_stats.
/// Caller must xfree() the returned buffer (or NULL).
DoSearchEchoResult nvim_do_search_echo(int dirc, int options,
                                       const char *searchstr, size_t searchstrlen)
{
  DoSearchEchoResult result = { NULL, 0, 0 };

  if (!((options & SEARCH_ECHO) && messaging() && !msg_silent
        && (!cmd_silent || !shortmess(SHM_SEARCHCOUNT)))) {
    return result;
  }

  char off_buf[40];
  size_t off_len = 0;

  msg_start();
  msg_ext_set_kind("search_cmd");

  // Read search offset fields from Rust-owned state
  extern int rs_get_search_offset_end(int idx);
  extern int rs_get_search_offset_line(int idx);
  extern int64_t rs_get_search_offset_off(int idx);
  int off_end = rs_get_search_offset_end(0);
  int off_line = rs_get_search_offset_line(0);
  int64_t off_off = rs_get_search_offset_off(0);

  if (!cmd_silent && (off_line || off_end || off_off)) {
    off_buf[off_len++] = (char)dirc;
    if (off_end) {
      off_buf[off_len++] = 'e';
    } else if (!off_line) {
      off_buf[off_len++] = 's';
    }
    off_buf[off_len] = NUL;
    if (off_off != 0 || off_line) {
      off_len += (size_t)snprintf(off_buf + off_len, sizeof(off_buf) - off_len,
                                  "%+" PRId64, off_off);
    }
  }

  // Read search pattern from Rust-owned state
  extern const char *rs_get_search_pattern(void);
  extern size_t rs_get_search_pattern_len(void);
  const char *p;
  size_t plen;
  if (*searchstr == NUL) {
    p = rs_get_search_pattern();
    plen = rs_get_search_pattern_len();
  } else {
    p = searchstr;
    plen = searchstrlen;
  }

  size_t msgbufsize;
  if (!shortmess(SHM_SEARCHCOUNT) || cmd_silent) {
    if (ui_has(kUIMessages)) {
      msgbufsize = 0;
    } else if (msg_scrolled != 0 && !cmd_silent) {
      msgbufsize = (size_t)((Rows - msg_row) * Columns - 1);
    } else {
      msgbufsize = (size_t)((Rows - msg_row - 1) * Columns + sc_col - 1);
    }
    if (msgbufsize < plen + off_len + SEARCH_STAT_BUF_LEN + 3) {
      msgbufsize = plen + off_len + SEARCH_STAT_BUF_LEN + 3;
    }
  } else {
    msgbufsize = plen + off_len + 3;
  }

  char *msgbuf = xmalloc(msgbufsize);
  memset(msgbuf, ' ', msgbufsize);
  size_t msgbuflen = msgbufsize - 1;
  msgbuf[msgbuflen] = NUL;

  if (!cmd_silent) {
    ui_busy_start();
    msgbuf[0] = (char)dirc;
    if (utf_iscomposing_first(utf_ptr2char(p))) {
      msgbuf[1] = ' ';
      memmove(msgbuf + 2, p, plen);
    } else {
      memmove(msgbuf + 1, p, plen);
    }
    if (off_len > 0) {
      memmove(msgbuf + plen + 1, off_buf, off_len);
    }

    char *trunc = msg_strtrunc(msgbuf, true);
    if (trunc != NULL) {
      xfree(msgbuf);
      msgbuf = trunc;
      msgbuflen = strlen(msgbuf);
    }

    if (curwin->w_p_rl && *curwin->w_p_rlc == 's') {
      char *r = reverse_text(msgbuf);
      xfree(msgbuf);
      msgbuf = r;
      msgbuflen = strlen(msgbuf);
      while (*r == ' ') {
        r++;
      }
      size_t pat_len = (size_t)(msgbuf + msgbuflen - r);
      memmove(msgbuf, r, pat_len);
      if ((size_t)(r - msgbuf) >= pat_len) {
        memset(r, ' ', pat_len);
      } else {
        memset(msgbuf + pat_len, ' ', (size_t)(r - msgbuf));
      }
    }
    msg_outtrans(msgbuf, 0, false);
    msg_clr_eos();
    msg_check();

    gotocmdline(false);
    ui_flush();
    ui_busy_stop();
    msg_nowait = true;
  }

  result.msgbuf = msgbuf;
  result.msgbuflen = msgbuflen;
  result.show_search_stats = !shortmess(SHM_SEARCHCOUNT) ? 1 : 0;

  return result;
}

/// Free the echo result.
void nvim_do_search_echo_free(char *msgbuf)
{
  xfree(msgbuf);
}

/// Pre-searchit character offset subtraction.
/// This handles the "?pat?e+2" / "/pat/s-2" case.
DoSearchPos nvim_do_search_pre_offset(linenr_T lnum, colnr_T col)
{
  extern int rs_get_search_offset_line(int idx);
  extern int64_t rs_get_search_offset_off(int idx);
  pos_T pos = { .lnum = lnum, .col = col, .coladd = 0 };
  int64_t off = rs_get_search_offset_off(0);

  if (!rs_get_search_offset_line(0) && off && pos.col < MAXCOL - 2) {
    if (off > 0) {
      int64_t c;
      for (c = off; c; c--) {
        if (decl(&pos) == -1) {
          break;
        }
      }
      if (c) {
        pos.lnum = 0;
        pos.col = MAXCOL;
      }
    } else {
      int64_t c;
      for (c = off; c; c++) {
        if (incl(&pos) == -1) {
          break;
        }
      }
      if (c) {
        pos.lnum = curbuf->b_ml.ml_line_count + 1;
        pos.col = 0;
      }
    }
  }

  return (DoSearchPos){ pos.lnum, pos.col };
}

/// Post-searchit line/char offset addition.
DoSearchPostOffset nvim_do_search_post_offset(linenr_T lnum, colnr_T col, int options,
                                              int pat_has_semicolon)
{
  DoSearchPostOffset result = { lnum, col, 1, 0 };
  pos_T pos = { .lnum = lnum, .col = col, .coladd = 0 };
  pos_T org_pos = pos;

  extern int rs_get_search_offset_line(int idx);
  extern int64_t rs_get_search_offset_off(int idx);
  int off_line = rs_get_search_offset_line(0);
  int64_t off_off = rs_get_search_offset_off(0);

  if (!(options & SEARCH_NOOF) || pat_has_semicolon) {
    if (off_line) {
      int64_t c = pos.lnum + off_off;
      if (c < 1) {
        pos.lnum = 1;
      } else if (c > curbuf->b_ml.ml_line_count) {
        pos.lnum = curbuf->b_ml.ml_line_count;
      } else {
        pos.lnum = (linenr_T)c;
      }
      pos.col = 0;
      result.retval = 2;
    } else if (pos.col < MAXCOL - 2) {
      int64_t c = off_off;
      if (c > 0) {
        while (c-- > 0) {
          if (incl(&pos) == -1) {
            break;
          }
        }
      } else {
        while (c++ < 0) {
          if (decl(&pos) == -1) {
            break;
          }
        }
      }
    }
    if (!equalpos(pos, org_pos)) {
      result.has_offset = 1;
    }
  }

  result.lnum = pos.lnum;
  result.col = pos.col;
  return result;
}

/// Check if search wrapped (show top/bot msg).
int nvim_do_search_show_top_bot(int dirc, linenr_T pos_lnum, colnr_T pos_col)
{
  if (shortmess(SHM_SEARCH)) {
    return 0;
  }
  pos_T pos = { .lnum = pos_lnum, .col = pos_col, .coladd = 0 };
  if ((dirc == '/' && lt(pos, curwin->w_cursor))
      || (dirc == '?' && lt(curwin->w_cursor, pos))) {
    return 1;
  }
  return 0;
}

/// Set oap->inclusive if needed.
void nvim_do_search_set_oap_inclusive(void *oap)
{
  // Read spats[0].off.end from Rust-owned state
  extern int rs_get_search_offset_end(int idx);
  if (oap != NULL && rs_get_search_offset_end(0)) {
    ((oparg_T *)oap)->inclusive = true;
  }
}

/// Fire EVENT_SEARCHWRAPPED autocmd.
void nvim_do_search_autocmd_wrapped(void)
{
  apply_autocmds(EVENT_SEARCHWRAPPED, NULL, NULL, false, NULL);
}

/// Show search stats (cmdline_search_stat).
void nvim_do_search_show_stats(int dirc, linenr_T pos_lnum, colnr_T pos_col,
                               int show_top_bot, char *msgbuf, size_t msgbuflen,
                               int count, int has_offset)
{
  pos_T pos = { .lnum = pos_lnum, .col = pos_col, .coladd = 0 };
  cmdline_search_stat(dirc, &pos, &curwin->w_cursor,
                      show_top_bot != 0, msgbuf, msgbuflen,
                      (count != 1 || has_offset
                       || (!(fdo_flags & kOptFdoFlagSearch)
                           && hasFolding(curwin, curwin->w_cursor.lnum, NULL, NULL))),
                      (int)p_msc,
                      SEARCH_STAT_DEF_TIMEOUT);
}

/// Emit E386 error.
void nvim_do_search_emsg_e386(void)
{
  emsg(_("E386: Expected '?' or '/'  after ';'"));
}

/// Emit e_noprevre error.
void nvim_do_search_emsg_noprevre(void)
{
  emsg(_(e_noprevre));
}

/// Set pcmark and cursor for search result.
void nvim_do_search_finish(int options, linenr_T lnum, colnr_T col)
{
  if (options & SEARCH_MARK) {
    setpcmark();
  }
  curwin->w_cursor.lnum = lnum;
  curwin->w_cursor.col = col;
  curwin->w_cursor.coladd = 0;
  curwin->w_set_curswant = true;
}

/// Save Rust-owned spats[0].off, returns opaque data via struct.
SavedSearchOff nvim_do_search_save_off(void)
{
  extern char rs_search_get_dir(void);
  extern int rs_get_search_offset_line(int idx);
  extern int rs_get_search_offset_end(int idx);
  extern int64_t rs_get_search_offset_off(int idx);
  return (SavedSearchOff){
    rs_search_get_dir(),
    rs_get_search_offset_line(0),
    rs_get_search_offset_end(0),
    rs_get_search_offset_off(0)
  };
}

/// Restore Rust-owned spats[0].off from saved data.
void nvim_do_search_restore_off(SavedSearchOff saved)
{
  // set_search_direction is the export_name for rs_set_search_direction_raw (no vv_searchforward update)
  extern void set_search_direction(int dir);
  extern void rs_set_search_offset_line_end_off(int line, int end, int64_t off);
  // restore dir without calling set_vv_searchforward (matching original behavior)
  set_search_direction(saved.dir);
  rs_set_search_offset_line_end_off(saved.line, saved.end, saved.off);
}

/// Get cursor position for do_search start.
DoSearchPos nvim_do_search_get_cursor(void)
{
  return (DoSearchPos){ curwin->w_cursor.lnum, curwin->w_cursor.col };
}

// =========================================================================
// Phase 3: findmatchlimit C accessors
// =========================================================================

_Static_assert(FM_BACKWARD == 0x01, "FM_BACKWARD mismatch");
_Static_assert(FM_FORWARD == 0x02, "FM_FORWARD mismatch");
_Static_assert(FM_BLOCKSTOP == 0x04, "FM_BLOCKSTOP mismatch");
_Static_assert(CPO_MATCH == '%', "CPO_MATCH mismatch");
_Static_assert(CPO_MATCHBSL == 'M', "CPO_MATCHBSL mismatch");
_Static_assert(kMTLineWise == 1, "kMTLineWise mismatch");

/// Get ml_get(lnum) for curbuf.
char *nvim_search_ml_get(linenr_T lnum)
{
  return ml_get(lnum);
}

/// Get ml_get_len(lnum) for curbuf.
colnr_T nvim_search_ml_get_len(linenr_T lnum)
{
  return ml_get_len(lnum);
}

/// Get curbuf->b_ml.ml_line_count.
linenr_T nvim_search_get_line_count(void)
{
  return curbuf->b_ml.ml_line_count;
}

/// Get curbuf->b_p_mps.
char *nvim_search_get_curbuf_b_p_mps(void)
{
  return curbuf->b_p_mps;
}

/// Get curbuf->b_p_lisp.
int nvim_search_get_curbuf_b_p_lisp(void)
{
  return curbuf->b_p_lisp ? 1 : 0;
}

/// Get curwin->w_p_rl.
int nvim_search_get_curwin_w_p_rl(void)
{
  return curwin->w_p_rl ? 1 : 0;
}

/// Get curwin->w_cursor.lnum.
linenr_T nvim_search_get_curwin_cursor_lnum(void)
{
  return curwin->w_cursor.lnum;
}

/// Get curwin->w_cursor.col.
colnr_T nvim_search_get_curwin_cursor_col(void)
{
  return curwin->w_cursor.col;
}

/// Wrap check_linecomment().
int nvim_search_check_linecomment(const char *line)
{
  return check_linecomment(line);
}

/// Set oap->motion_type.
void nvim_search_set_oap_motion_type(void *oap, int motion_type)
{
  if (oap != NULL) {
    ((oparg_T *)oap)->motion_type = (MotionType)motion_type;
  }
}

/// Get a pointer to cap->nchar_composing.
const char *nvim_cap_get_nchar_composing_ptr(cmdarg_T *cap)
{
  return cap ? cap->nchar_composing : NULL;
}

// =============================================================================
// Phase 6: update_search_stat / cmdline_search_stat accessors
// =============================================================================

/// Set the p_ws option (wrapscan).
void nvim_set_p_ws(int val)
{
  p_ws = val;
}

/// Get the p_msc option (maxsearchcount).
long nvim_get_p_msc(void)
{
  return (long)p_msc;
}

/// Get buf_get_changedtick(curbuf).
int nvim_curbuf_get_changedtick(void)
{
  return (int)buf_get_changedtick(curbuf);
}

/// Get curbuf as opaque pointer for identity comparison.
void *nvim_search_get_curbuf_ptr(void)
{
  return (void *)curbuf;
}

/// Call searchit from Rust for update_search_stat.
/// This wraps the pos_T marshalling.
int nvim_searchit_for_stat(int *pos_lnum, int *pos_col, int *pos_coladd,
                           int *end_lnum, int *end_col, int *end_coladd)
{
  pos_T pos = { *pos_lnum, *pos_col, *pos_coladd };
  pos_T endpos = { 0, 0, 0 };
  int retval = searchit(curwin, curbuf, &pos, &endpos,
                         FORWARD, NULL, 0, 1, SEARCH_KEEP, RE_LAST, NULL);
  *pos_lnum = pos.lnum;
  *pos_col = pos.col;
  *pos_coladd = pos.coladd;
  *end_lnum = endpos.lnum;
  *end_col = endpos.col;
  *end_coladd = endpos.coladd;
  return retval;
}

/// Set profile limit for search stat timeout.
proftime_T nvim_profile_setlimit_ms(int timeout)
{
  return profile_setlimit(timeout);
}

/// Check if profile time limit has been passed (for search stat).
int nvim_profile_passed_limit_val(proftime_T start)
{
  return profile_passed_limit(start) ? 1 : 0;
}

// nvim_stat_spats_pat_matches and nvim_stat_copy_spats_pat deleted:
// these operations now use Rust search_state::spats_pat_matches and copy_spats_last_pat.

/// Free a pointer allocated by nvim_stat_copy_spats_pat.
void nvim_stat_free_pat(char *pat)
{
  xfree(pat);
}

/// Check if curwin->w_p_rl is set and curwin->w_p_rlc starts with 's'.
int nvim_curwin_rl_with_rlc_s(void)
{
  return (curwin->w_p_rl && *curwin->w_p_rlc == 's') ? 1 : 0;
}

/// Display the cmdline search stat message.
/// Handles msg_hist_off, msg_ext_overwrite, msg_ext_set_kind, give_warning.
void nvim_cmdline_stat_display(const char *msgbuf)
{
  msg_hist_off = true;
  msg_ext_overwrite = true;
  msg_ext_set_kind("search_count");
  give_warning(msgbuf, false);
  msg_hist_off = false;
}

// =============================================================================
// Phase 7: Integration function accessors
// =============================================================================

/// Check if curbuf has lisp mode enabled.
int nvim_curbuf_is_lisp(void)
{
  return curbuf->b_p_lisp ? 1 : 0;
}

/// Wrap is_pos_in_string() for Rust.
int nvim_is_pos_in_string(const char *line, int col)
{
  return is_pos_in_string(line, (colnr_T)col);
}

// Phase 7b: is_zero_width accessors
// NOTE: nvim_get_called_emsg() already exists in message.c

// nvim_regmmatch_alloc() and nvim_regmmatch_free() already defined above (~line 3717)

// nvim_get_last_spat_pat deleted: use Rust search_state::get_last_pat_for_searchit instead.

/// Call search_regcomp for is_zero_width.
int nvim_is_zero_width_regcomp(const char *pat, size_t patlen, void *regmatch)
{
  return search_regcomp((char *)pat, patlen, NULL, RE_SEARCH, RE_SEARCH,
                         SEARCH_KEEP, (regmmatch_T *)regmatch);
}

/// Set regmatch.startpos[0].col.
void nvim_regmatch_set_startcol(void *regmatch, int col)
{
  ((regmmatch_T *)regmatch)->startpos[0].col = (colnr_T)col;
}

/// Get regmatch.startpos[0].col.
int nvim_regmatch_get_startcol(const void *regmatch)
{
  return ((const regmmatch_T *)regmatch)->startpos[0].col;
}

/// Get regmatch.startpos[0].lnum.
int nvim_regmatch_get_startlnum(const void *regmatch)
{
  return ((const regmmatch_T *)regmatch)->startpos[0].lnum;
}

/// Get regmatch.endpos[0].col.
int nvim_regmatch_get_endcol(const void *regmatch)
{
  return ((const regmmatch_T *)regmatch)->endpos[0].col;
}

/// Get regmatch.endpos[0].lnum.
int nvim_regmatch_get_endlnum(const void *regmatch)
{
  return ((const regmmatch_T *)regmatch)->endpos[0].lnum;
}

/// Call vim_regexec_multi for is_zero_width.
int nvim_is_zero_width_regexec(void *regmatch, int lnum, int col)
{
  return vim_regexec_multi((regmmatch_T *)regmatch, curwin, curbuf,
                            (linenr_T)lnum, (colnr_T)col, NULL, NULL);
}

/// Call searchit for is_zero_width (pos_T marshalling).
int nvim_is_zero_width_searchit(const char *pat, size_t patlen, int dir,
                                int flags, int *pos_lnum, int *pos_col,
                                int *pos_coladd)
{
  pos_T pos = { *pos_lnum, *pos_col, *pos_coladd };
  int result = searchit(curwin, curbuf, &pos, NULL, (Direction)dir,
                        (char *)pat, patlen, 1,
                        SEARCH_KEEP + flags, RE_SEARCH, NULL);
  *pos_lnum = pos.lnum;
  *pos_col = pos.col;
  *pos_coladd = pos.coladd;
  return result;
}

// Phase 7c: search_for_exact_line accessors

/// Get buf->b_ml.ml_line_count.
int nvim_buf_ml_line_count(void *buf)
{
  return ((buf_T *)buf)->b_ml.ml_line_count;
}

/// Get ml_get_buf(buf, lnum) and skipwhite offset.
const char *nvim_buf_get_line_skipwhite(void *buf, int lnum, int *skipwhite_off)
{
  char *ptr = ml_get_buf((buf_T *)buf, (linenr_T)lnum);
  char *p = skipwhite(ptr);
  *skipwhite_off = (int)(p - ptr);
  return p;
}


/// Compare with mb_strcmp_ic.
int nvim_mb_strcmp_ic_wrapper(int ic, const char *s1, const char *s2)
{
  return mb_strcmp_ic((bool)ic, s1, s2);
}

/// Compare with mb_strnicmp.
int nvim_mb_strnicmp_wrapper(const char *s1, const char *s2, size_t len)
{
  return mb_strnicmp(s1, s2, len);
}

/// Get p_ic option.
int nvim_search_get_p_ic(void)
{
  return p_ic ? 1 : 0;
}

/// Call shortmess(SHM_SEARCH).
int nvim_shortmess_search(void)
{
  return shortmess(SHM_SEARCH) ? 1 : 0;
}

/// Give top_bot_msg or bot_top_msg warning.
void nvim_give_search_wrap_warning(int at_top)
{
  give_warning(_(at_top ? top_bot_msg : bot_top_msg), true);
}

// =============================================================================
// Phase 8: showmatch accessors
// =============================================================================

/// Get p_ri option (reverse insert).
int nvim_showmatch_get_p_ri(void)
{
  return p_ri ? 1 : 0;
}

/// Call findmatch(NULL, NUL) and check if the match is visible.
/// Returns: -1 = no match pair found (beep), 0 = match not visible, 1 = visible.
/// On success (1), out_lnum/out_col/out_coladd are set.
int nvim_showmatch_find_and_check(int *out_lnum, int *out_col, int *out_coladd)
{
  pos_T *lpos = findmatch(NULL, NUL);
  if (lpos == NULL) {
    return -1;  // no match
  }

  if (lpos->lnum < curwin->w_topline || lpos->lnum >= curwin->w_botline) {
    return 0;  // not visible vertically
  }

  colnr_T vcol = 0;
  if (!curwin->w_p_wrap) {
    getvcol(curwin, lpos, NULL, &vcol, NULL);
  }

  bool col_visible = curwin->w_p_wrap
                     || (vcol >= curwin->w_leftcol
                         && vcol < curwin->w_leftcol + curwin->w_view_width);
  if (!col_visible) {
    return 0;  // not visible horizontally
  }

  *out_lnum = lpos->lnum;
  *out_col = lpos->col;
  *out_coladd = lpos->coladd;
  return 1;
}

/// Beep for showmatch.
void nvim_showmatch_beep(void)
{
  vim_beep(kOptBoFlagShowmatch);
}

// =============================================================================
// Phase 9: f_searchcount accessors
// =============================================================================

// nvim_searchcount_set_pattern and nvim_searchcount_has_pattern are now
// handled directly in Rust via search_state::searchcount_set_pattern and
// search_state::searchcount_has_pattern (which operate on Rust-owned SPATS).

// Phase 4: find_pattern_in_path constants
_Static_assert(FIND_ANY == 1, "FIND_ANY mismatch");
_Static_assert(FIND_DEFINE == 2, "FIND_DEFINE mismatch");
_Static_assert(CHECK_PATH == 3, "CHECK_PATH mismatch");
_Static_assert(ACTION_SHOW == 1, "ACTION_SHOW mismatch");
_Static_assert(ACTION_GOTO == 2, "ACTION_GOTO mismatch");
_Static_assert(ACTION_SPLIT == 3, "ACTION_SPLIT mismatch");
_Static_assert(ACTION_SHOW_ALL == 4, "ACTION_SHOW_ALL mismatch");
_Static_assert(ACTION_EXPAND == 5, "ACTION_EXPAND mismatch");

// =============================================================================
// Phase 7d: current_search accessors
// =============================================================================

/// Get curwin->w_cursor.coladd.
colnr_T nvim_search_get_curwin_cursor_coladd(void)
{
  return curwin->w_cursor.coladd;
}

/// incl() on a position passed by components.
/// Updates *lnum, *col, *coladd in place.
void nvim_search_incl_pos(int *lnum, int *col, int *coladd)
{
  pos_T pos = { *lnum, *col, *coladd };
  incl(&pos);
  *lnum = pos.lnum;
  *col = pos.col;
  *coladd = pos.coladd;
}

/// decl() on a position passed by components.
/// Updates *lnum, *col, *coladd in place.
void nvim_search_decl_pos(int *lnum, int *col, int *coladd)
{
  pos_T pos = { *lnum, *col, *coladd };
  decl(&pos);
  *lnum = pos.lnum;
  *col = pos.col;
  *coladd = pos.coladd;
}

/// Call searchit for current_search.
/// Marshals pos and end_pos from/to integer components.
/// dir: 1 = FORWARD, 0 = BACKWARD.
/// flags: SEARCH_* flags.
/// pat/patlen: the search pattern (from Rust-owned SPATS[last_idx]).
/// Returns 1 if found, 0 if not found.
int nvim_search_current_searchit(int dir, int flags, int count,
                                 int *pos_lnum, int *pos_col, int *pos_coladd,
                                 int *end_lnum, int *end_col, int *end_coladd)
{
  _Static_assert(kOptFdoFlagSearch == 0x40,
                 "kOptFdoFlagSearch changed - update K_OPT_FDO_FLAG_SEARCH in search/src/commands.rs");
  // Read pat/patlen from Rust-owned state
  extern const char *rs_get_last_used_pattern(void);
  extern size_t rs_get_last_used_pattern_len(void);
  const char *pat = rs_get_last_used_pattern();
  size_t patlen = rs_get_last_used_pattern_len();
  pos_T pos = { *pos_lnum, *pos_col, *pos_coladd };
  pos_T end_pos = { *end_lnum, *end_col, *end_coladd };
  int result = searchit(curwin, curbuf, &pos, &end_pos,
                        dir ? FORWARD : BACKWARD,
                        (char *)pat, patlen,
                        count, SEARCH_KEEP | flags, RE_SEARCH, NULL);
  *pos_lnum = pos.lnum;
  *pos_col = pos.col;
  *pos_coladd = pos.coladd;
  *end_lnum = end_pos.lnum;
  *end_col = end_pos.col;
  *end_coladd = end_pos.coladd;
  return result;
}

