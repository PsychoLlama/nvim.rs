// search.c: code for normal mode searching commands

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

#include "search.c.generated.h"

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

static SearchPattern spats[2] = {
  // Last used search pattern
  [0] = { NULL, 0, true, false, 0, { '/', false, false, 0 }, NULL },
  // Last used substitute pattern
  [1] = { NULL, 0, true, false, 0, { '/', false, false, 0 }, NULL }
};

static int last_idx = 0;        // index in spats[] for RE_LAST

static uint8_t lastc[2] = { NUL, NUL };   // last character searched for
static Direction lastcdir = FORWARD;      // last direction of character search
static bool last_t_cmd = true;            // last search t_cmd
static char lastc_bytes[MAX_SCHAR_SIZE + 1];

// Rust FFI declarations for character search state
extern int rs_last_csearch_forward(void);
extern int rs_last_csearch_until(void);
extern const char *rs_last_csearch(void);
extern int rs_search_was_last_used(void);
extern void rs_set_last_csearch(int c, const char *s, int len);
extern void rs_set_csearch_direction(int dir);
extern void rs_set_csearch_until(int until);
extern void rs_set_search_direction_raw(int cdir);
extern void rs_reset_search_dir(void);
extern int rs_search_linewhite(int lnum);

// Rust FFI declarations for pattern save/restore
extern void rs_save_search_patterns(void);
extern void rs_restore_search_patterns(void);
extern void rs_save_last_search_pattern(void);
extern void rs_restore_last_search_pattern(void);
extern void rs_save_incsearch_state(void);
extern void rs_restore_incsearch_state(void);

// Rust FFI declarations for ShaDa pattern get/set
extern void rs_get_search_pattern_shada(SearchPattern *pat);
extern void rs_get_substitute_pattern_shada(SearchPattern *pat);
extern void rs_set_search_pattern_shada(const SearchPattern *pat);
extern void rs_set_substitute_pattern_shada(const SearchPattern *pat);
extern void rs_set_last_used_pattern(int is_substitute_pattern);
extern void rs_free_search_patterns(void);

// Rust FFI declarations for search_regcomp and pattern compilation
extern int rs_search_regcomp(char *pat, size_t patlen, char **used_pat,
                              int pat_save, int pat_use, int options,
                              regmmatch_T *regmatch);
extern void rs_save_re_pat(int idx, const char *pat, size_t patlen, int magic);
extern void rs_set_last_search_pat(const char *s, int idx, int magic, int setlast);
extern void rs_last_pat_prog(regmmatch_T *regmatch);
extern void rs_set_vv_searchforward(void);

// Rust FFI declarations for searchc()
extern int rs_searchc(cmdarg_T *cap, bool t_cmd);

// Rust FFI declarations for search statistics
extern void rs_update_search_stat(int dirc, int pos_lnum, int pos_col, int pos_coladd,
                                  int cursor_lnum, int cursor_col, int cursor_coladd,
                                  searchstat_T *stat, bool recompute,
                                  int maxcount, int timeout);
extern void rs_cmdline_search_stat(int dirc, int pos_lnum, int pos_col, int pos_coladd,
                                   int cursor_lnum, int cursor_col, int cursor_coladd,
                                   bool show_top_bot_msg, char *msgbuf, size_t msgbuflen,
                                   bool recompute, int maxcount, int timeout);

// Rust FFI declarations for pattern utilities
extern int rs_pat_has_uppercase(const char *pat);
extern int rs_ignorecase(const char *pat);
extern int rs_ignorecase_opt(const char *pat, int ic, int scs);
extern int rs_needs_previous_pattern(const char *pat);

// Rust FFI declarations for pattern accessors
extern const char *rs_get_search_pattern(void);
extern const char *rs_get_subst_pattern(void);
extern const char *rs_get_last_used_pattern(void);
extern const char *rs_get_mr_pattern(void);

// Rust FFI declarations for incremental search
extern void rs_incsearch_state_save(void *state);
extern void rs_incsearch_state_restore(const void *state);

// Rust FFI declarations for Phase 7 integration functions
extern int rs_check_linecomment(const char *line);
extern int rs_is_zero_width(const char *pattern, size_t patternlen, bool move,
                             int cur_lnum, int cur_col, int cur_coladd, int direction);
extern int rs_search_for_exact_line(void *buf, int *pos_lnum, int *pos_col,
                                     int dir, const char *pat);

// Rust FFI declaration for Phase 8
extern int rs_showmatch_find_match(int c, int *out_lnum, int *out_col, int *out_coladd);

/// Get the lastcdir static variable (accessor for Rust).
int nvim_get_lastcdir(void)
{
  return lastcdir;
}

/// Get the last_t_cmd static variable (accessor for Rust).
int nvim_get_last_t_cmd(void)
{
  return last_t_cmd;
}

/// Get the lastc_bytes static variable (accessor for Rust).
const char *nvim_get_lastc_bytes(void)
{
  return lastc_bytes;
}

/// Get the last_idx static variable (accessor for Rust).
int nvim_get_last_idx(void)
{
  return last_idx;
}

/// Set the last_idx static variable (setter for Rust).
void nvim_set_last_idx(int idx)
{
  last_idx = idx;
}

static int lastc_bytelen = 1;             // >1 for multi-byte char

/// Get the lastc_bytelen static variable (accessor for Rust).
int nvim_get_lastc_bytelen(void)
{
  return lastc_bytelen;
}

/// Set the lastc_bytelen static variable (setter for Rust).
void nvim_set_lastc_bytelen(int len)
{
  lastc_bytelen = len;
}

/// Get a value from the lastc array (accessor for Rust).
uint8_t nvim_get_lastc(int idx)
{
  if (idx >= 0 && idx < 2) {
    return lastc[idx];
  }
  return 0;
}

/// Set a value in the lastc array (setter for Rust).
void nvim_set_lastc(int idx, uint8_t val)
{
  if (idx >= 0 && idx < 2) {
    lastc[idx] = val;
  }
}

/// Set the lastcdir static variable (setter for Rust).
void nvim_set_lastcdir(int dir)
{
  lastcdir = dir;
}

/// Set the last_t_cmd static variable (setter for Rust).
void nvim_set_last_t_cmd(int t_cmd)
{
  last_t_cmd = t_cmd;
}

/// Bulk copy bytes into lastc_bytes[] and clear if len is 0 (accessor for Rust).
void nvim_set_lastc_bytes_raw(const char *s, int len)
{
  if (len > 0 && s != NULL) {
    memcpy(lastc_bytes, s, (size_t)len);
  } else {
    CLEAR_FIELD(lastc_bytes);
  }
}

/// Check if line 'lnum' in curbuf is empty or has only white chars (accessor for Rust).
/// Returns a pointer past whitespace in the line.
char *nvim_search_skipwhite_ml_get(linenr_T lnum)
{
  return skipwhite(ml_get(lnum));
}

// copy of spats[], for keeping the search patterns while executing autocmds
static SearchPattern saved_spats[ARRAY_SIZE(spats)];
static char *saved_mr_pattern = NULL;
static size_t saved_mr_patternlen = 0;
static int saved_spats_last_idx = 0;
static bool saved_spats_no_hlsearch = false;

// allocated copy of pattern used by search_regcomp()
static char *mr_pattern = NULL;
static size_t mr_patternlen = 0;

/// Get the mr_pattern static variable (accessor for Rust).
const char *nvim_get_mr_pattern(void)
{
  return mr_pattern;
}

/// Get the mr_patternlen static variable (accessor for Rust).
size_t nvim_get_mr_patternlen(void)
{
  return mr_patternlen;
}

/// Get the pattern string from spats array (accessor for Rust).
const char *nvim_get_spat_pat(int idx)
{
  if (idx >= 0 && idx < 2) {
    return spats[idx].pat;
  }
  return NULL;
}

/// Check if the pattern at given index is NULL (accessor for Rust).
int nvim_spat_pat_is_null(int idx)
{
  if (idx >= 0 && idx < 2) {
    return spats[idx].pat == NULL;
  }
  return 1;
}

/// Get the pattern length from spats array (accessor for Rust).
size_t nvim_get_spat_patlen(int idx)
{
  if (idx >= 0 && idx < 2) {
    return spats[idx].patlen;
  }
  return 0;
}

/// Get the magic flag from spats array (accessor for Rust).
int nvim_get_spat_magic(int idx)
{
  if (idx >= 0 && idx < 2) {
    return spats[idx].magic;
  }
  return 0;
}

/// Get the no_scs flag from spats array (accessor for Rust).
int nvim_get_spat_no_scs(int idx)
{
  if (idx >= 0 && idx < 2) {
    return spats[idx].no_scs;
  }
  return 0;
}

/// Get the search direction from spats array (accessor for Rust).
char nvim_get_spat_off_dir(int idx)
{
  if (idx >= 0 && idx < 2) {
    return spats[idx].off.dir;
  }
  return '/';
}

/// Get the line offset flag from spats array (accessor for Rust).
int nvim_get_spat_off_line(int idx)
{
  if (idx >= 0 && idx < 2) {
    return spats[idx].off.line;
  }
  return 0;
}

/// Get the end offset flag from spats array (accessor for Rust).
int nvim_get_spat_off_end(int idx)
{
  if (idx >= 0 && idx < 2) {
    return spats[idx].off.end;
  }
  return 0;
}

/// Get the offset value from spats array (accessor for Rust).
int64_t nvim_get_spat_off_off(int idx)
{
  if (idx >= 0 && idx < 2) {
    return spats[idx].off.off;
  }
  return 0;
}

/// Set the search direction in spats array (setter for Rust).
void nvim_set_spat_off_dir(int idx, char dir)
{
  if (idx >= 0 && idx < 2) {
    spats[idx].off.dir = dir;
  }
}

/// Set the line offset flag in spats array (setter for Rust).
void nvim_set_spat_off_line(int idx, int line)
{
  if (idx >= 0 && idx < 2) {
    spats[idx].off.line = line;
  }
}

/// Set the end offset flag in spats array (setter for Rust).
void nvim_set_spat_off_end(int idx, int end)
{
  if (idx >= 0 && idx < 2) {
    spats[idx].off.end = end;
  }
}

/// Set the offset value in spats array (setter for Rust).
void nvim_set_spat_off_off(int idx, int64_t off)
{
  if (idx >= 0 && idx < 2) {
    spats[idx].off.off = off;
  }
}

/// Call set_vv_searchforward (wrapper for Rust).
void nvim_call_set_vv_searchforward(void)
{
  set_vv_searchforward();
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
int search_regcomp(char *pat, size_t patlen, char **used_pat, int pat_save, int pat_use,
                   int options, regmmatch_T *regmatch)
{
  return rs_search_regcomp(pat, patlen, used_pat, pat_save, pat_use, options, regmatch);
}

/// Get search pattern used by search_regcomp().
char *get_search_pat(void)
{
  return mr_pattern;
}

void save_re_pat(int idx, char *pat, size_t patlen, int magic)
{
  rs_save_re_pat(idx, pat, patlen, magic);
}

// Save the search patterns, so they can be restored later.
// Used before/after executing autocommands and user functions.
static int save_level = 0;

/// Get the save_level static variable (accessor for Rust).
int nvim_get_save_level(void)
{
  return save_level;
}

void save_search_patterns(void)
{
  rs_save_search_patterns();
}

void restore_search_patterns(void)
{
  rs_restore_search_patterns();
}

static inline void free_spat(SearchPattern *const spat)
{
  xfree(spat->pat);
  xfree(spat->additional_data);
}

#if defined(EXITFREE)
void free_search_patterns(void)
{
  rs_free_search_patterns();
}

#endif

// copy of spats[RE_SEARCH], for keeping the search patterns while incremental
// searching
static SearchPattern saved_last_search_spat;
static int did_save_last_search_spat = 0;
static int saved_last_idx = 0;
static bool saved_no_hlsearch = false;
static colnr_T saved_search_match_endcol;
static linenr_T saved_search_match_lines;

/// Get the did_save_last_search_spat counter (accessor for Rust).
int nvim_get_did_save_last_search_spat(void)
{
  return did_save_last_search_spat;
}

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

/// Set search_first_line global (for Rust incsearch).
void nvim_set_search_first_line(linenr_T value)
{
  search_first_line = value;
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
void save_last_search_pattern(void)
{
  rs_save_last_search_pattern();
}

void restore_last_search_pattern(void)
{
  rs_restore_last_search_pattern();
}

/// Save and restore the incsearch highlighting variables.
/// This is required so that calling searchcount() at does not invalidate the
/// incsearch highlighting.
static void save_incsearch_state(void)
{
  rs_save_incsearch_state();
}

static void restore_incsearch_state(void)
{
  rs_restore_incsearch_state();
}

char *last_search_pattern(void)
{
  return spats[RE_SEARCH].pat;
}

size_t last_search_pattern_len(void)
{
  return spats[RE_SEARCH].patlen;
}

/// Return true when case should be ignored for search pattern "pat".
/// Uses the 'ignorecase' and 'smartcase' options.
int ignorecase(char *pat)
{
  return ignorecase_opt(pat, p_ic, p_scs);
}

/// As ignorecase() but pass the "ic" and "scs" flags.
int ignorecase_opt(char *pat, int ic_in, int scs)
{
  int ic = ic_in;
  if (ic && !no_smartcase && scs
      && !(ctrl_x_mode_not_default()
           && curbuf->b_p_inf)) {
    ic = !pat_has_uppercase(pat);
  }
  no_smartcase = false;

  return ic;
}

/// Returns true if pattern `pat` has an uppercase character.
bool pat_has_uppercase(char *pat)
  FUNC_ATTR_NONNULL_ALL
{
  char *p = pat;
  magic_T magic_val = MAGIC_ON;

  // get the magicness of the pattern
  skip_regexp_ex(pat, NUL, magic_isset(), NULL, NULL, &magic_val);

  while (*p != NUL) {
    const int l = utfc_ptr2len(p);

    if (l > 1) {
      if (mb_isupper(utf_ptr2char(p))) {
        return true;
      }
      p += l;
    } else if (*p == '\\' && magic_val <= MAGIC_ON) {
      if (p[1] == '_' && p[2] != NUL) {  // skip "\_X"
        p += 3;
      } else if (p[1] == '%' && p[2] != NUL) {  // skip "\%X"
        p += 3;
      } else if (p[1] != NUL) {  // skip "\X"
        p += 2;
      } else {
        p += 1;
      }
    } else if ((*p == '%' || *p == '_') && magic_val == MAGIC_ALL) {
      if (p[1] != NUL) {  // skip "_X" and %X
        p += 2;
      } else {
        p++;
      }
    } else if (mb_isupper((uint8_t)(*p))) {
      return true;
    } else {
      p++;
    }
  }
  return false;
}

extern const char *rs_last_csearch(void);

const char *last_csearch(void)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_last_csearch();
}

extern int rs_last_csearch_forward(void);

int last_csearch_forward(void)
{
  return rs_last_csearch_forward();
}

extern int rs_last_csearch_until(void);

int last_csearch_until(void)
{
  return rs_last_csearch_until();
}

void set_last_csearch(int c, char *s, int len)
{
  rs_set_last_csearch(c, s, len);
}

void set_csearch_direction(Direction cdir)
{
  rs_set_csearch_direction(cdir);
}

void set_csearch_until(int t_cmd)
{
  rs_set_csearch_until(t_cmd);
}

char *last_search_pat(void)
{
  return spats[last_idx].pat;
}

// Reset search direction to forward.  For "gd" and "gD" commands.
void reset_search_dir(void)
{
  rs_reset_search_dir();
}

// Set the last search pattern.  For ":let @/ =" and ShaDa file.
// Also set the saved search pattern, so that this works in an autocommand.
void set_last_search_pat(const char *s, int idx, int magic, bool setlast)
{
  rs_set_last_search_pat(s, idx, magic, setlast);
}

// Get a regexp program for the last used search pattern.
// This is used for highlighting all matches in a window.
// Values returned in regmatch->regprog and regmatch->rmm_ic.
void last_pat_prog(regmmatch_T *regmatch)
{
  rs_last_pat_prog(regmatch);
}

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

// Rust implementation of searchit
typedef struct {
  int retval;
  int pos_lnum;
  int pos_col;
  int pos_coladd;
  int end_lnum;
  int end_col;
  int end_coladd;
  int end_pos_set;
  int sa_timed_out;
  int sa_wrapped;
} SearchitResult;

extern int rs_do_search(void *oap, int dirc_in, int search_delim,
                        char *pat_in, size_t patlen_in, int count,
                        int search_options,
                        int has_sia,
                        linenr_T sa_stop_lnum, void *sa_tm,
                        int *sa_timed_out_out, int *sa_wrapped_out);

// Rust implementation of findmatchlimit
typedef struct {
  bool found;
  linenr_T lnum;
  colnr_T col;
} RsFindMatchResult;

extern RsFindMatchResult rs_findmatchlimit(void *oap, int initc, int flags, int64_t maxtravel);

extern void rs_find_pattern_in_path(const char *ptr, int dir, size_t len,
                                    int whole, int skip_comments,
                                    int type, int count, int action,
                                    linenr_T start_lnum, linenr_T end_lnum,
                                    int forceit, int silent);

extern int rs_searchit(void *win, void *buf,
                       linenr_T pos_lnum, colnr_T pos_col, colnr_T pos_coladd,
                       int has_end_pos, int dir,
                       char *pat, size_t patlen,
                       int count, int options, int pat_use,
                       linenr_T sa_stop_lnum, void *sa_tm,
                       int has_extra_arg,
                       SearchitResult *result);

int searchit(win_T *win, buf_T *buf, pos_T *pos, pos_T *end_pos, Direction dir, char *pat,
             size_t patlen, int count, int options, int pat_use, searchit_arg_T *extra_arg)
{
  SearchitResult result;

  int retval = rs_searchit(
    win, buf,
    pos->lnum, pos->col, pos->coladd,
    end_pos != NULL ? 1 : 0,
    (int)dir,
    pat, patlen, count, options, pat_use,
    extra_arg != NULL ? extra_arg->sa_stop_lnum : 0,
    extra_arg != NULL ? (void *)extra_arg->sa_tm : NULL,
    extra_arg != NULL ? 1 : 0,
    &result);

  // Copy results back
  pos->lnum = result.pos_lnum;
  pos->col = result.pos_col;
  pos->coladd = result.pos_coladd;

  if (end_pos != NULL && result.end_pos_set) {
    end_pos->lnum = result.end_lnum;
    end_pos->col = result.end_col;
    end_pos->coladd = result.end_coladd;
  }

  if (extra_arg != NULL) {
    extra_arg->sa_timed_out = result.sa_timed_out;
    if (result.sa_wrapped) {
      extra_arg->sa_wrapped = true;
    }
  }

  return retval;
}

void set_search_direction(int cdir)
{
  rs_set_search_direction_raw(cdir);
}

static void set_vv_searchforward(void)
{
  set_vim_var_nr(VV_SEARCHFORWARD, spats[0].off.dir == '/');
}

/// Highest level string search function.
/// Search for the 'count'th occurrence of pattern 'pat' in direction 'dirc'
///
/// Careful: If spats[0].off.line == true and spats[0].off.off == 0 this
/// makes the movement linewise without moving the match position.
///
/// @param dirc          if 0: use previous dir.
/// @param pat           NULL or empty : use previous string.
/// @param options       if true and
///                      SEARCH_REV   == true : go in reverse of previous dir.
///                      SEARCH_ECHO  == true : echo the search command and handle options
///                      SEARCH_MSG   == true : may give error message
///                      SEARCH_OPT   == true : interpret optional flags
///                      SEARCH_HIS   == true : put search pattern in history
///                      SEARCH_NOOF  == true : don't add offset to position
///                      SEARCH_MARK  == true : set previous context mark
///                      SEARCH_KEEP  == true : keep previous search pattern
///                      SEARCH_START == true : accept match at curpos itself
///                      SEARCH_PEEK  == true : check for typed char, cancel search
/// @param oap           can be NULL
/// @param dirc          '/' or '?'
/// @param search_delim  delimiter for search, e.g. '%' in s%regex%replacement
/// @param sia           optional arguments or NULL
///
/// @return              0 for failure, 1 for found, 2 for found and line offset added.
int do_search(oparg_T *oap, int dirc, int search_delim, char *pat, size_t patlen, int count,
              int options, searchit_arg_T *sia)
{
  int has_sia = (sia != NULL) ? 1 : 0;
  linenr_T sa_stop_lnum = sia ? sia->sa_stop_lnum : 0;
  void *sa_tm = sia ? (void *)&sia->sa_tm : NULL;
  int sa_timed_out = sia ? (sia->sa_timed_out ? 1 : 0) : 0;
  int sa_wrapped = sia ? (sia->sa_wrapped ? 1 : 0) : 0;

  int retval = rs_do_search(oap, dirc, search_delim, pat, patlen, count, options,
                            has_sia, sa_stop_lnum, sa_tm,
                            &sa_timed_out, &sa_wrapped);

  if (sia) {
    sia->sa_timed_out = sa_timed_out != 0;
    sia->sa_wrapped = sa_wrapped != 0;
  }

  return retval;
}

// search_for_exact_line(buf, pos, dir, pat)
//
// Search for a line starting with the given pattern (ignoring leading
// white-space), starting from pos and going in direction "dir". "pos" will
// contain the position of the match found.    Blank lines match only if
// ADDING is set.  If p_ic is set then the pattern must be in lowercase.
// Return OK for success, or FAIL if no line found.
int search_for_exact_line(buf_T *buf, pos_T *pos, Direction dir, char *pat)
{
  int lnum = pos->lnum;
  int col = pos->col;
  int result = rs_search_for_exact_line(buf, &lnum, &col, (int)dir, pat);
  pos->lnum = lnum;
  pos->col = col;
  return result;
}

// Character Searches

/// Search for a character in a line.  If "t_cmd" is false, move to the
/// position of the character, otherwise move to just before the char.
/// Do this "cap->count1" times.
/// Return FAIL or OK.
int searchc(cmdarg_T *cap, bool t_cmd)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_searchc(cap, t_cmd);
}

// "Other" Searches

// findmatch - find the matching paren or brace
//
// Improvement over vi: Braces inside quotes are ignored.
pos_T *findmatch(oparg_T *oap, int initc)
{
  return findmatchlimit(oap, initc, 0, 0);
}

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
pos_T *findmatchlimit(oparg_T *oap, int initc, int flags, int64_t maxtravel)
{
  static pos_T pos;
  RsFindMatchResult result = rs_findmatchlimit(oap, initc, flags, maxtravel);
  if (!result.found) {
    return NULL;
  }
  pos.lnum = result.lnum;
  pos.col = result.col;
  pos.coladd = 0;
  return &pos;
}

#if 0  // Migrated to Rust (matchparen.rs)
static pos_T *findmatchlimit_old(oparg_T *oap, int initc, int flags, int64_t maxtravel)
{
  static pos_T pos;
  int findc = 0;
  int count = 0;
  bool backwards = false;
  bool raw_string = false;
  bool inquote = false;
  char *ptr;
  int hash_dir = 0;                     // Direction searched for # things
  int comment_dir = 0;                  // Direction searched for comments
  int traveled = 0;                     // how far we've searched so far
  bool ignore_cend = false;             // ignore comment end
  int match_escaped = 0;                // search for escaped match
  int dir;                              // Direction to search
  int comment_col = MAXCOL;             // start of / / comment
  bool lispcomm = false;                // inside of Lisp-style comment
  bool lisp = curbuf->b_p_lisp;         // engage Lisp-specific hacks ;)

  pos = curwin->w_cursor;
  pos.coladd = 0;
  char *linep = ml_get(pos.lnum);     // pointer to current line

  // vi compatible matching
  bool cpo_match = (vim_strchr(p_cpo, CPO_MATCH) != NULL);
  // don't recognize backslashes
  bool cpo_bsl = (vim_strchr(p_cpo, CPO_MATCHBSL) != NULL);

  // Direction to search when initc is '/', '*' or '#'
  if (flags & FM_BACKWARD) {
    dir = BACKWARD;
  } else if (flags & FM_FORWARD) {
    dir = FORWARD;
  } else {
    dir = 0;
  }

  // if initc given, look in the table for the matching character
  // '/' and '*' are special cases: look for start or end of comment.
  // When '/' is used, we ignore running backwards into a star-slash, for
  // "[*" command, we just want to find any comment.
  if (initc == '/' || initc == '*' || initc == 'R') {
    comment_dir = dir;
    if (initc == '/') {
      ignore_cend = true;
    }
    backwards = (dir == FORWARD) ? false : true;
    raw_string = (initc == 'R');
    initc = NUL;
  } else if (initc != '#' && initc != NUL) {
    find_mps_values(&initc, &findc, &backwards, true);
    if (dir) {
      backwards = (dir == FORWARD) ? false : true;
    }
    if (findc == NUL) {
      return NULL;
    }
  } else {
    // Either initc is '#', or no initc was given and we need to look
    // under the cursor.
    if (initc == '#') {
      hash_dir = dir;
    } else {
      // initc was not given, must look for something to match under
      // or near the cursor.
      // Only check for special things when 'cpo' doesn't have '%'.
      if (!cpo_match) {
        // Are we before or at #if, #else etc.?
        ptr = skipwhite(linep);
        if (*ptr == '#' && pos.col <= (colnr_T)(ptr - linep)) {
          ptr = skipwhite(ptr + 1);
          if (strncmp(ptr, "if", 2) == 0
              || strncmp(ptr, "endif", 5) == 0
              || strncmp(ptr, "el", 2) == 0) {
            hash_dir = 1;
          }
        } else if (linep[pos.col] == '/') {  // Are we on a comment?
          if (linep[pos.col + 1] == '*') {
            comment_dir = FORWARD;
            backwards = false;
            pos.col++;
          } else if (pos.col > 0 && linep[pos.col - 1] == '*') {
            comment_dir = BACKWARD;
            backwards = true;
            pos.col--;
          }
        } else if (linep[pos.col] == '*') {
          if (linep[pos.col + 1] == '/') {
            comment_dir = BACKWARD;
            backwards = true;
          } else if (pos.col > 0 && linep[pos.col - 1] == '/') {
            comment_dir = FORWARD;
            backwards = false;
          }
        }
      }

      // If we are not on a comment or the # at the start of a line, then
      // look for brace anywhere on this line after the cursor.
      if (!hash_dir && !comment_dir) {
        // Find the brace under or after the cursor.
        // If beyond the end of the line, use the last character in
        // the line.
        if (linep[pos.col] == NUL && pos.col) {
          pos.col--;
        }
        while (true) {
          initc = utf_ptr2char(linep + pos.col);
          if (initc == NUL) {
            break;
          }

          find_mps_values(&initc, &findc, &backwards, false);
          if (findc) {
            break;
          }
          pos.col += utfc_ptr2len(linep + pos.col);
        }
        if (!findc) {
          // no brace in the line, maybe use "  #if" then
          if (!cpo_match && *skipwhite(linep) == '#') {
            hash_dir = 1;
          } else {
            return NULL;
          }
        } else if (!cpo_bsl) {
          int bslcnt = 0;

          // Set "match_escaped" if there are an odd number of
          // backslashes.
          for (int col = pos.col; check_prevcol(linep, col, '\\', &col);) {
            bslcnt++;
          }
          match_escaped = (bslcnt & 1);
        }
      }
    }
    if (hash_dir) {
      // Look for matching #if, #else, #elif, or #endif
      if (oap != NULL) {
        oap->motion_type = kMTLineWise;  // Linewise for this case only
      }
      if (initc != '#') {
        ptr = skipwhite(skipwhite(linep) + 1);
        if (strncmp(ptr, "if", 2) == 0 || strncmp(ptr, "el", 2) == 0) {
          hash_dir = 1;
        } else if (strncmp(ptr, "endif", 5) == 0) {
          hash_dir = -1;
        } else {
          return NULL;
        }
      }
      pos.col = 0;
      while (!got_int) {
        if (hash_dir > 0) {
          if (pos.lnum == curbuf->b_ml.ml_line_count) {
            break;
          }
        } else if (pos.lnum == 1) {
          break;
        }
        pos.lnum += hash_dir;
        linep = ml_get(pos.lnum);
        line_breakcheck();              // check for CTRL-C typed
        ptr = skipwhite(linep);
        if (*ptr != '#') {
          continue;
        }
        pos.col = (colnr_T)(ptr - linep);
        ptr = skipwhite(ptr + 1);
        if (hash_dir > 0) {
          if (strncmp(ptr, "if", 2) == 0) {
            count++;
          } else if (strncmp(ptr, "el", 2) == 0) {
            if (count == 0) {
              return &pos;
            }
          } else if (strncmp(ptr, "endif", 5) == 0) {
            if (count == 0) {
              return &pos;
            }
            count--;
          }
        } else {
          if (strncmp(ptr, "if", 2) == 0) {
            if (count == 0) {
              return &pos;
            }
            count--;
          } else if (initc == '#' && strncmp(ptr, "el", 2) == 0) {
            if (count == 0) {
              return &pos;
            }
          } else if (strncmp(ptr, "endif", 5) == 0) {
            count++;
          }
        }
      }
      return NULL;
    }
  }

  // This is just guessing: when 'rightleft' is set, search for a matching
  // paren/brace in the other direction.
  if (curwin->w_p_rl && vim_strchr("()[]{}<>", initc) != NULL) {
    backwards = !backwards;
  }

  int do_quotes = -1;                 // check for quotes in current line
  int at_start;                       // do_quotes value at start position
  TriState start_in_quotes = kNone;   // start position is in quotes
  pos_T match_pos;                    // Where last slash-star was found
  clearpos(&match_pos);

  // backward search: Check if this line contains a single-line comment
  if ((backwards && comment_dir) || lisp) {
    comment_col = check_linecomment(linep);
  }
  if (lisp && comment_col != MAXCOL && pos.col > (colnr_T)comment_col) {
    lispcomm = true;        // find match inside this comment
  }

  while (!got_int) {
    // Go to the next position, forward or backward. We could use
    // inc() and dec() here, but that is much slower
    if (backwards) {
      // char to match is inside of comment, don't search outside
      if (lispcomm && pos.col < (colnr_T)comment_col) {
        break;
      }
      if (pos.col == 0) {               // at start of line, go to prev. one
        if (pos.lnum == 1) {            // start of file
          break;
        }
        pos.lnum--;

        if (maxtravel > 0 && ++traveled > maxtravel) {
          break;
        }

        linep = ml_get(pos.lnum);
        pos.col = ml_get_len(pos.lnum);  // pos.col on trailing NUL
        do_quotes = -1;
        line_breakcheck();

        // Check if this line contains a single-line comment
        if (comment_dir || lisp) {
          comment_col = check_linecomment(linep);
        }
        // skip comment
        if (lisp && comment_col != MAXCOL) {
          pos.col = comment_col;
        }
      } else {
        pos.col--;
        pos.col -= utf_head_off(linep, linep + pos.col);
      }
    } else {                          // forward search
      if (linep[pos.col] == NUL
          // at end of line, go to next one
          // For lisp don't search for match in comment
          || (lisp && comment_col != MAXCOL
              && pos.col == (colnr_T)comment_col)) {
        if (pos.lnum == curbuf->b_ml.ml_line_count          // end of file
            // line is exhausted and comment with it,
            // don't search for match in code
            || lispcomm) {
          break;
        }
        pos.lnum++;

        if (maxtravel && traveled++ > maxtravel) {
          break;
        }

        linep = ml_get(pos.lnum);
        pos.col = 0;
        do_quotes = -1;
        line_breakcheck();
        if (lisp) {         // find comment pos in new line
          comment_col = check_linecomment(linep);
        }
      } else {
        pos.col += utfc_ptr2len(linep + pos.col);
      }
    }

    // If FM_BLOCKSTOP given, stop at a '{' or '}' in column 0.
    if (pos.col == 0 && (flags & FM_BLOCKSTOP)
        && (linep[0] == '{' || linep[0] == '}')) {
      if (linep[0] == findc && count == 0) {  // match!
        return &pos;
      }
      break;  // out of scope
    }

    if (comment_dir) {
      // Note: comments do not nest, and we ignore quotes in them
      // TODO(vim): ignore comment brackets inside strings
      if (comment_dir == FORWARD) {
        if (linep[pos.col] == '*' && linep[pos.col + 1] == '/') {
          pos.col++;
          return &pos;
        }
      } else {    // Searching backwards
        // A comment may contain / * or / /, it may also start or end
        // with / * /. Ignore a / * after / / and after *.
        if (pos.col == 0) {
          continue;
        } else if (raw_string) {
          if (linep[pos.col - 1] == 'R'
              && linep[pos.col] == '"'
              && vim_strchr(linep + pos.col + 1, '(') != NULL) {
            // Possible start of raw string. Now that we have the
            // delimiter we can check if it ends before where we
            // started searching, or before the previously found
            // raw string start.
            if (!find_rawstring_end(linep, &pos,
                                    count > 0 ? &match_pos : &curwin->w_cursor)) {
              count++;
              match_pos = pos;
              match_pos.col--;
            }
            linep = ml_get(pos.lnum);  // may have been released
          }
        } else if (linep[pos.col - 1] == '/'
                   && linep[pos.col] == '*'
                   && (pos.col == 1 || linep[pos.col - 2] != '*')
                   && (int)pos.col < comment_col) {
          count++;
          match_pos = pos;
          match_pos.col--;
        } else if (linep[pos.col - 1] == '*' && linep[pos.col] == '/') {
          if (count > 0) {
            pos = match_pos;
          } else if (pos.col > 1 && linep[pos.col - 2] == '/'
                     && (int)pos.col <= comment_col) {
            pos.col -= 2;
          } else if (ignore_cend) {
            continue;
          } else {
            return NULL;
          }
          return &pos;
        }
      }
      continue;
    }

    // If smart matching ('cpoptions' does not contain '%'), braces inside
    // of quotes are ignored, but only if there is an even number of
    // quotes in the line.
    if (cpo_match) {
      do_quotes = 0;
    } else if (do_quotes == -1) {
      // Count the number of quotes in the line, skipping \" and '"'.
      // Watch out for "\\".
      at_start = do_quotes;
      for (ptr = linep; *ptr; ptr++) {
        if (ptr == linep + pos.col + backwards) {
          at_start = (do_quotes & 1);
        }
        if (*ptr == '"'
            && (ptr == linep || ptr[-1] != '\'' || ptr[1] != '\'')) {
          do_quotes++;
        }
        if (*ptr == '\\' && ptr[1] != NUL) {
          ptr++;
        }
      }
      do_quotes &= 1;               // result is 1 with even number of quotes

      // If we find an uneven count, check current line and previous
      // one for a '\' at the end.
      if (!do_quotes) {
        inquote = false;
        if (ptr[-1] == '\\') {
          do_quotes = 1;
          if (start_in_quotes == kNone) {
            // Do we need to use at_start here?
            inquote = true;
            start_in_quotes = kTrue;
          } else if (backwards) {
            inquote = true;
          }
        }
        if (pos.lnum > 1) {
          ptr = ml_get(pos.lnum - 1);
          if (*ptr && *(ptr + ml_get_len(pos.lnum - 1) - 1) == '\\') {
            do_quotes = 1;
            if (start_in_quotes == kNone) {
              inquote = at_start;
              if (inquote) {
                start_in_quotes = kTrue;
              }
            } else if (!backwards) {
              inquote = true;
            }
          }

          // ml_get() only keeps one line, need to get linep again
          linep = ml_get(pos.lnum);
        }
      }
    }
    if (start_in_quotes == kNone) {
      start_in_quotes = kFalse;
    }

    // If 'smartmatch' is set:
    //   Things inside quotes are ignored by setting 'inquote'.  If we
    //   find a quote without a preceding '\' invert 'inquote'.  At the
    //   end of a line not ending in '\' we reset 'inquote'.
    //
    //   In lines with an uneven number of quotes (without preceding '\')
    //   we do not know which part to ignore. Therefore we only set
    //   inquote if the number of quotes in a line is even, unless this
    //   line or the previous one ends in a '\'.  Complicated, isn't it?
    const int c = utf_ptr2char(linep + pos.col);
    switch (c) {
    case NUL:
      // at end of line without trailing backslash, reset inquote
      if (pos.col == 0 || linep[pos.col - 1] != '\\') {
        inquote = false;
        start_in_quotes = kFalse;
      }
      break;

    case '"':
      // a quote that is preceded with an odd number of backslashes is
      // ignored
      if (do_quotes) {
        int col;

        for (col = pos.col - 1; col >= 0; col--) {
          if (linep[col] != '\\') {
            break;
          }
        }
        if ((((int)pos.col - 1 - col) & 1) == 0) {
          inquote = !inquote;
          start_in_quotes = kFalse;
        }
      }
      break;

    // If smart matching ('cpoptions' does not contain '%'):
    //   Skip things in single quotes: 'x' or '\x'.  Be careful for single
    //   single quotes, eg jon's.  Things like '\233' or '\x3f' are not
    //   skipped, there is never a brace in them.
    //   Ignore this when finding matches for `'.
    case '\'':
      if (!cpo_match && initc != '\'' && findc != '\'') {
        if (backwards) {
          if (pos.col > 1) {
            if (linep[pos.col - 2] == '\'') {
              pos.col -= 2;
              break;
            } else if (linep[pos.col - 2] == '\\'
                       && pos.col > 2 && linep[pos.col - 3] == '\'') {
              pos.col -= 3;
              break;
            }
          }
        } else if (linep[pos.col + 1]) {  // forward search
          if (linep[pos.col + 1] == '\\'
              && linep[pos.col + 2] && linep[pos.col + 3] == '\'') {
            pos.col += 3;
            break;
          } else if (linep[pos.col + 2] == '\'') {
            pos.col += 2;
            break;
          }
        }
      }
      FALLTHROUGH;

    default:
      // For Lisp skip over backslashed (), {} and [].
      // (actually, we skip #\( et al)
      if (curbuf->b_p_lisp
          && vim_strchr("(){}[]", c) != NULL
          && pos.col > 1
          && check_prevcol(linep, pos.col, '\\', NULL)
          && check_prevcol(linep, pos.col - 1, '#', NULL)) {
        break;
      }

      // Check for match outside of quotes, and inside of
      // quotes when the start is also inside of quotes.
      if ((!inquote || start_in_quotes == kTrue)
          && (c == initc || c == findc)) {
        int bslcnt = 0;

        if (!cpo_bsl) {
          for (int col = pos.col; check_prevcol(linep, col, '\\', &col);) {
            bslcnt++;
          }
        }
        // Only accept a match when 'M' is in 'cpo' or when escaping
        // is what we expect.
        if (cpo_bsl || (bslcnt & 1) == match_escaped) {
          if (c == initc) {
            count++;
          } else {
            if (count == 0) {
              return &pos;
            }
            count--;
          }
        }
      }
    }
  }

  if (comment_dir == BACKWARD && count > 0) {
    pos = match_pos;
    return &pos;
  }
  return (pos_T *)NULL;         // never found it
}
#endif  // Migrated to Rust

/// Check if line[] contains a / / comment.
/// @returns MAXCOL if not, otherwise return the column.
int check_linecomment(const char *line)
{
  return rs_check_linecomment(line);
}

/// Move cursor briefly to character matching the one under the cursor.
/// Used for Insert mode and "r" command.
/// Show the match only if it is visible on the screen.
/// If there isn't a match, then beep.
///
/// @param c  char to show match for
void showmatch(int c)
{
  OptInt *so = curwin->w_p_so >= 0 ? &curwin->w_p_so : &p_so;
  OptInt *siso = curwin->w_p_siso >= 0 ? &curwin->w_p_siso : &p_siso;

  // Rust handles: matchpairs scanning, findmatch, visibility check
  int match_lnum, match_col, match_coladd;
  if (!rs_showmatch_find_match(c, &match_lnum, &match_col, &match_coladd)) {
    return;
  }

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

/// Find next search match under cursor, cursor at end.
/// Used while an operator is pending, and in Visual mode.
///
/// @param forward  true for forward, false for backward
int current_search(int count, bool forward)
{
  bool old_p_ws = p_ws;
  pos_T save_VIsual = VIsual;

  // Correct cursor when 'selection' is exclusive
  if (VIsual_active && *p_sel == 'e' && lt(VIsual, curwin->w_cursor)) {
    dec_cursor();
  }

  // When searching forward and the cursor is at the start of the Visual
  // area, skip the first search backward, otherwise it doesn't move.
  const bool skip_first_backward = forward && VIsual_active
                                   && lt(curwin->w_cursor, VIsual);

  pos_T pos = curwin->w_cursor;       // position after the pattern
  pos_T orig_pos = curwin->w_cursor;  // position of the cursor at beginning
  if (VIsual_active) {
    // Searching further will extend the match.
    if (forward) {
      incl(&pos);
    } else {
      decl(&pos);
    }
  }

  // Is the pattern is zero-width?, this time, don't care about the direction
  int zero_width = is_zero_width(spats[last_idx].pat, spats[last_idx].patlen,
                                 true, &curwin->w_cursor, FORWARD);
  if (zero_width == -1) {
    return FAIL;  // pattern not found
  }

  pos_T end_pos;  // end position of the pattern match
  int result;     // result of various function calls

  // The trick is to first search backwards and then search forward again,
  // so that a match at the current cursor position will be correctly
  // captured.  When "forward" is false do it the other way around.
  for (int i = 0; i < 2; i++) {
    int dir;
    if (forward) {
      if (i == 0 && skip_first_backward) {
        continue;
      }
      dir = i;
    } else {
      dir = !i;
    }

    int flags = 0;

    if (!dir && !zero_width) {
      flags = SEARCH_END;
    }
    end_pos = pos;

    // wrapping should not occur in the first round
    if (i == 0) {
      p_ws = false;
    }

    result = searchit(curwin, curbuf, &pos, &end_pos,
                      (dir ? FORWARD : BACKWARD),
                      spats[last_idx].pat, spats[last_idx].patlen, i ? count : 1,
                      SEARCH_KEEP | flags, RE_SEARCH, NULL);

    p_ws = old_p_ws;

    // First search may fail, but then start searching from the
    // beginning of the file (cursor might be on the search match)
    // except when Visual mode is active, so that extending the visual
    // selection works.
    if (i == 1 && !result) {  // not found, abort
      curwin->w_cursor = orig_pos;
      if (VIsual_active) {
        VIsual = save_VIsual;
      }
      return FAIL;
    } else if (i == 0 && !result) {
      if (forward) {  // try again from start of buffer
        clearpos(&pos);
      } else {  // try again from end of buffer
                // searching backwards, so set pos to last line and col
        pos.lnum = curwin->w_buffer->b_ml.ml_line_count;
        pos.col = ml_get_len(curwin->w_buffer->b_ml.ml_line_count);
      }
    }
  }

  pos_T start_pos = pos;

  if (!VIsual_active) {
    VIsual = start_pos;
  }

  // put the cursor after the match
  curwin->w_cursor = end_pos;
  if (lt(VIsual, end_pos) && forward) {
    if (skip_first_backward) {
      // put the cursor on the start of the match
      curwin->w_cursor = pos;
    } else {
      // put the cursor on last character of match
      dec_cursor();
    }
  } else if (VIsual_active && lt(curwin->w_cursor, VIsual) && forward) {
    curwin->w_cursor = pos;   // put the cursor on the start of the match
  }
  VIsual_active = true;
  VIsual_mode = 'v';

  if (*p_sel == 'e') {
    // Correction for exclusive selection depends on the direction.
    if (forward && ltoreq(VIsual, curwin->w_cursor)) {
      inc_cursor();
    } else if (!forward && ltoreq(curwin->w_cursor, VIsual)) {
      inc(&VIsual);
    }
  }

  if (fdo_flags & kOptFdoFlagSearch && KeyTyped) {
    foldOpenCursor();
  }

  may_start_select('c');
  setmouse();
  redraw_curbuf_later(UPD_INVERTED);
  showmode();

  return OK;
}

/// Check if the pattern is zero-width.
/// If move is true, check from the beginning of the buffer,
/// else from position "cur".
/// "direction" is FORWARD or BACKWARD.
/// Returns true, false or -1 for failure.
static int is_zero_width(char *pattern, size_t patternlen, bool move, pos_T *cur,
                         Direction direction)
{
  return rs_is_zero_width(pattern, patternlen, move,
                           cur ? cur->lnum : 0, cur ? cur->col : 0,
                           cur ? cur->coladd : 0, (int)direction);
}

/// @return  true if line 'lnum' is empty or has white chars only.
bool linewhite(linenr_T lnum)
{
  return rs_search_linewhite(lnum);
}

/// Add the search count "[3/19]" to "msgbuf".
/// See update_search_stat() for other arguments.
static void cmdline_search_stat(int dirc, pos_T *pos, pos_T *cursor_pos, bool show_top_bot_msg,
                                char *msgbuf, size_t msgbuflen, bool recompute, int maxcount,
                                int timeout)
{
  rs_cmdline_search_stat(dirc, pos->lnum, pos->col, pos->coladd,
                         cursor_pos->lnum, cursor_pos->col, cursor_pos->coladd,
                         show_top_bot_msg, msgbuf, msgbuflen,
                         recompute, maxcount, timeout);
}

// Add the search count information to "stat".
// "stat" must not be NULL.
// When "recompute" is true always recompute the numbers.
// dirc == 0: don't find the next/previous match (only set the result to "stat")
// dirc == '/': find the next match
// dirc == '?': find the previous match
static void update_search_stat(int dirc, pos_T *pos, pos_T *cursor_pos, searchstat_T *stat,
                               bool recompute, int maxcount, int timeout)
{
  rs_update_search_stat(dirc, pos->lnum, pos->col, pos->coladd,
                        cursor_pos->lnum, cursor_pos->col, cursor_pos->coladd,
                        stat, recompute, maxcount, timeout);
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

  save_last_search_pattern();
  save_incsearch_state();
  if (pattern != NULL) {
    if (*pattern == NUL) {
      goto the_end;
    }
    xfree(spats[last_idx].pat);
    spats[last_idx].patlen = strlen(pattern);
    spats[last_idx].pat = xstrnsave(pattern, spats[last_idx].patlen);
  }
  if (spats[last_idx].pat == NULL || *spats[last_idx].pat == NUL) {
    goto the_end;  // the previous pattern was never defined
  }

  update_search_stat(0, &pos, &pos, &stat, recompute, maxcount, timeout);

  tv_dict_add_nr(rettv->vval.v_dict, S_LEN("current"), stat.cur);
  tv_dict_add_nr(rettv->vval.v_dict, S_LEN("total"), stat.cnt);
  tv_dict_add_nr(rettv->vval.v_dict, S_LEN("exact_match"), stat.exact_match);
  tv_dict_add_nr(rettv->vval.v_dict, S_LEN("incomplete"), stat.incomplete);
  tv_dict_add_nr(rettv->vval.v_dict, S_LEN("maxcount"), stat.last_maxcount);

the_end:
  restore_last_search_pattern();
  restore_incsearch_state();
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
/// If p_ic && compl_status_sol() then ptr must be in lowercase.
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
      && !compl_status_sol()) {
    size_t patsize = len + 5;
    char *pat = xmalloc(patsize);
    assert(len <= INT_MAX);
    snprintf(pat, patsize, st->whole ? "\\<%.*s\\>" : "%.*s", (int)len, ptr);
    st->regmatch.rm_ic = ignorecase(pat);
    st->regmatch.regprog = vim_regcomp(pat, magic_isset() ? RE_MAGIC : 0);
    xfree(pat);
    if (st->regmatch.regprog == NULL) {
      return (FpipInitResult){ st, 0 };
    }
  }

  st->inc_opt = (*curbuf->b_p_inc == NUL) ? p_inc : curbuf->b_p_inc;
  if (*st->inc_opt != NUL) {
    st->incl_regmatch.regprog = vim_regcomp(st->inc_opt, magic_isset() ? RE_MAGIC : 0);
    if (st->incl_regmatch.regprog == NULL) {
      return (FpipInitResult){ st, 0 };
    }
    st->incl_regmatch.rm_ic = false;
  }

  if (type == FIND_DEFINE && (*curbuf->b_p_def != NUL || *p_def != NUL)) {
    st->def_regmatch.regprog = vim_regcomp(
        *curbuf->b_p_def == NUL ? p_def : curbuf->b_p_def,
        magic_isset() ? RE_MAGIC : 0);
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
        if (define_matched || compl_status_sol()) {
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
        if (compl_status_adding() && (int)strlen(p) >= ins_compl_len()) {
          p += ins_compl_len();
          if (vim_iswordp(p)) {
            goto exit_matched;
          }
          p = find_word_start(p);
        }
        p = find_word_end(p);
        i = (int)(p - aux);

        if (compl_status_adding() && i == ins_compl_len()) {
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
          p = find_word_start(p);
          p = find_word_end(p);
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

          if (i == ins_compl_len()) {
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
              if (!win_valid(curwin_save)) {
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
            && curwin != curwin_save && win_valid(curwin_save)) {
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
          && !compl_status_sol()
          && *startp != NUL
          && *(startp + utfc_ptr2len(startp)) != NUL) {
        goto search_line;
      }
    }
    line_breakcheck();
    if (action == ACTION_EXPAND) {
      ins_compl_check_keys(30, false);
    }
    if (got_int || ins_compl_interrupted()) {
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
    if (got_int || ins_compl_interrupted()) {
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

// Original find_pattern_in_path body (kept for reference in #if 0)
#if 0
static void find_pattern_in_path_old(char *ptr, Direction dir, size_t len, bool whole,
                                     bool skip_comments, int type, int count, int action,
                                     linenr_T start_lnum, linenr_T end_lnum,
                                     bool forceit, bool silent)
{
  SearchedFile *files;                  // Stack of included files
  SearchedFile *bigger;                 // When we need more space
  int max_path_depth = 50;
  int match_count = 1;

  char *new_fname;
  char *curr_fname = curbuf->b_fname;
  char *prev_fname = NULL;
  int depth_displayed;                  // For type==CHECK_PATH
  char *p;
  bool define_matched;
  regmatch_T regmatch;
  regmatch_T incl_regmatch;
  regmatch_T def_regmatch;
  bool matched = false;
  bool did_show = false;
  bool found = false;
  int i;
  char *already = NULL;
  char *startp = NULL;
  win_T *curwin_save = NULL;
  const int l_g_do_tagpreview = g_do_tagpreview;

  regmatch.regprog = NULL;
  incl_regmatch.regprog = NULL;
  def_regmatch.regprog = NULL;

  char *file_line = xmalloc(LSIZE);

  if (type != CHECK_PATH && type != FIND_DEFINE
      // when CONT_SOL is set compare "ptr" with the beginning of the
      // line is faster than quote_meta/regcomp/regexec "ptr" -- Acevedo
      && !compl_status_sol()) {
    size_t patsize = len + 5;
    char *pat = xmalloc(patsize);
    assert(len <= INT_MAX);
    snprintf(pat, patsize, whole ? "\\<%.*s\\>" : "%.*s", (int)len, ptr);
    // ignore case according to p_ic, p_scs and pat
    regmatch.rm_ic = ignorecase(pat);
    regmatch.regprog = vim_regcomp(pat, magic_isset() ? RE_MAGIC : 0);
    xfree(pat);
    if (regmatch.regprog == NULL) {
      goto fpip_end;
    }
  }
  char *inc_opt = (*curbuf->b_p_inc == NUL) ? p_inc : curbuf->b_p_inc;
  if (*inc_opt != NUL) {
    incl_regmatch.regprog = vim_regcomp(inc_opt, magic_isset() ? RE_MAGIC : 0);
    if (incl_regmatch.regprog == NULL) {
      goto fpip_end;
    }
    incl_regmatch.rm_ic = false;        // don't ignore case in incl. pat.
  }
  if (type == FIND_DEFINE && (*curbuf->b_p_def != NUL || *p_def != NUL)) {
    def_regmatch.regprog = vim_regcomp(*curbuf->b_p_def == NUL ? p_def : curbuf->b_p_def,
                                       magic_isset() ? RE_MAGIC : 0);
    if (def_regmatch.regprog == NULL) {
      goto fpip_end;
    }
    def_regmatch.rm_ic = false;         // don't ignore case in define pat.
  }
  files = xcalloc((size_t)max_path_depth, sizeof(SearchedFile));
  int old_files = max_path_depth;
  int depth = depth_displayed = -1;

  end_lnum = MIN(end_lnum, curbuf->b_ml.ml_line_count);
  linenr_T lnum = MIN(start_lnum, end_lnum);  // do at least one line
  char *line = get_line_and_copy(lnum, file_line);

  while (true) {
    if (incl_regmatch.regprog != NULL
        && vim_regexec(&incl_regmatch, line, 0)) {
      char *p_fname = (curr_fname == curbuf->b_fname)
                      ? curbuf->b_ffname : curr_fname;

      if (inc_opt != NULL && strstr(inc_opt, "\\zs") != NULL) {
        // Use text from '\zs' to '\ze' (or end) of 'include'.
        new_fname = find_file_name_in_path(incl_regmatch.startp[0],
                                           (size_t)(incl_regmatch.endp[0]
                                                    - incl_regmatch.startp[0]),
                                           FNAME_EXP|FNAME_INCL|FNAME_REL,
                                           1, p_fname);
      } else {
        // Use text after match with 'include'.
        new_fname = file_name_in_line(incl_regmatch.endp[0], 0,
                                      FNAME_EXP|FNAME_INCL|FNAME_REL, 1, p_fname,
                                      NULL);
      }
      bool already_searched = false;
      if (new_fname != NULL) {
        // Check whether we have already searched in this file
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
              msg_putchar('\n');  // cursor below last one
              if (!got_int) {  // don't display if 'q' typed at "--more--" message
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
          msg_putchar('\n');  // cursor below last one
        } else {
          gotocmdline(true);  // cursor at status line
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
        if (!got_int) {                     // don't display if 'q' typed
                                            // for "--more--" message
          for (i = 0; i <= depth_displayed; i++) {
            msg_puts("  ");
          }
          if (new_fname != NULL) {
            // using "new_fname" is more reliable, e.g., when
            // 'includeexpr' is set.
            msg_outtrans(new_fname, HLF_D, false);
          } else {
            // Isolate the file name.
            // Include the surrounding "" or <> if present.
            if (inc_opt != NULL
                && strstr(inc_opt, "\\zs") != NULL) {
              // pattern contains \zs, use the match
              p = incl_regmatch.startp[0];
              i = (int)(incl_regmatch.endp[0]
                        - incl_regmatch.startp[0]);
            } else {
              // find the file name after the end of the match
              for (p = incl_regmatch.endp[0];
                   *p && !vim_isfilec((uint8_t)(*p)); p++) {}
              for (i = 0; vim_isfilec((uint8_t)p[i]); i++) {}
            }

            if (i == 0) {
              // Nothing found, use the rest of the line.
              p = incl_regmatch.endp[0];
              i = (int)strlen(p);
            } else if (p > line) {
              // Avoid checking before the start of the line, can
              // happen if \zs appears in the regexp.
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
        // Push the new file onto the file stack
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
            // Something wrong. We will forget one of our already visited files
            // now.
            xfree(files[old_files].name);
            old_files++;
          }
          files[depth].name = curr_fname = new_fname;
          files[depth].lnum = 0;
          files[depth].matched = false;
          if (action == ACTION_EXPAND && !shortmess(SHM_COMPLETIONSCAN) && !silent) {
            msg_hist_off = true;                // reset in msg_trunc()
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
      // Check if the line is a define (type == FIND_DEFINE)
      p = line;
search_line:
      define_matched = false;
      if (def_regmatch.regprog != NULL
          && vim_regexec(&def_regmatch, line, 0)) {
        // Pattern must be first identifier after 'define', so skip
        // to that position before checking for match of pattern.  Also
        // don't let it match beyond the end of this identifier.
        p = def_regmatch.endp[0];
        while (*p && !vim_iswordc((uint8_t)(*p))) {
          p++;
        }
        define_matched = true;
      }

      // Look for a match.  Don't do this if we are looking for a
      // define and this line didn't match define_prog above.
      if (def_regmatch.regprog == NULL || define_matched) {
        if (define_matched || compl_status_sol()) {
          // compare the first "len" chars from "ptr"
          startp = skipwhite(p);
          if (p_ic) {
            matched = !mb_strnicmp(startp, ptr, len);
          } else {
            matched = !strncmp(startp, ptr, len);
          }
          if (matched && define_matched && whole
              && vim_iswordc((uint8_t)startp[len])) {
            matched = false;
          }
        } else if (regmatch.regprog != NULL
                   && vim_regexec(&regmatch, line, (colnr_T)(p - line))) {
          matched = true;
          startp = regmatch.startp[0];
          // Check if the line is not a comment line (unless we are
          // looking for a define).  A line starting with "# define"
          // is not considered to be a comment line.
          if (skip_comments) {
            if ((*line != '#'
                 || strncmp(skipwhite(line + 1), "define", 6) != 0)
                && get_leader_len(line, NULL, false, true)) {
              matched = false;
            }

            // Also check for a "/ *" or "/ /" before the match.
            // Skips lines like "int backwards;  / * normal index
            // * /" when looking for "normal".
            // Note: Doesn't skip "/ *" in comments.
            p = skipwhite(line);
            if (matched
                || (p[0] == '/' && p[1] == '*') || p[0] == '*') {
              for (p = line; *p && p < startp; p++) {
                if (matched
                    && p[0] == '/'
                    && (p[1] == '*' || p[1] == '/')) {
                  matched = false;
                  // After "//" all text is comment
                  if (p[1] == '/') {
                    break;
                  }
                  p++;
                } else if (!matched && p[0] == '*' && p[1] == '/') {
                  // Can find match after "* /".
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
        if (compl_status_adding() && (int)strlen(p) >= ins_compl_len()) {
          p += ins_compl_len();
          if (vim_iswordp(p)) {
            goto exit_matched;
          }
          p = find_word_start(p);
        }
        p = find_word_end(p);
        i = (int)(p - aux);

        if (compl_status_adding() && i == ins_compl_len()) {
          // IOSIZE > compl_length, so the strncpy works
          strncpy(IObuff, aux, (size_t)i);  // NOLINT(runtime/printf)

          // Get the next line: when "depth" < 0  from the current
          // buffer, otherwise from the included file.  Jump to
          // exit_matched when past the last line.
          if (depth < 0) {
            if (lnum >= end_lnum) {
              goto exit_matched;
            }
            line = get_line_and_copy(++lnum, file_line);
          } else if (vim_fgets(line = file_line,
                               LSIZE, files[depth].fp)) {
            goto exit_matched;
          }

          // we read a line, set "already" to check this "line" later
          // if depth >= 0 we'll increase files[depth].lnum far
          // below  -- Acevedo
          already = aux = p = skipwhite(line);
          p = find_word_start(p);
          p = find_word_end(p);
          if (p > aux) {
            if (*aux != ')' && IObuff[i - 1] != TAB) {
              if (IObuff[i - 1] != ' ') {
                IObuff[i++] = ' ';
              }
              // IObuf =~ "\(\k\|\i\).* ", thus i >= 2
              if (p_js
                  && (IObuff[i - 2] == '.'
                      || IObuff[i - 2] == '?'
                      || IObuff[i - 2] == '!')) {
                IObuff[i++] = ' ';
              }
            }
            // copy as much as possible of the new word
            if (p - aux >= IOSIZE - i) {
              p = aux + IOSIZE - i - 1;
            }
            strncpy(IObuff + i, aux, (size_t)(p - aux));  // NOLINT(runtime/printf)
            i += (int)(p - aux);
            cont_s_ipos = true;
          }
          IObuff[i] = NUL;
          aux = IObuff;

          if (i == ins_compl_len()) {
            goto exit_matched;
          }
        }

        const int add_r = ins_compl_add_infercase(aux, i, p_ic,
                                                  curr_fname == curbuf->b_fname
                                                  ? NULL : curr_fname,
                                                  dir, cont_s_ipos, 0);
        if (add_r == OK) {
          // if dir was BACKWARD then honor it just once
          dir = FORWARD;
        } else if (add_r == FAIL) {
          break;
        }
      } else if (action == ACTION_SHOW_ALL) {
        found = true;
        if (!did_show) {
          gotocmdline(true);                    // cursor at status line
        }
        if (curr_fname != prev_fname) {
          if (did_show) {
            msg_putchar('\n');                  // cursor below last one
          }
          if (!got_int) {             // don't display if 'q' typed
                                      // at "--more--" message
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

        // Set matched flag for this file and all the ones that
        // include it
        for (i = 0; i <= depth; i++) {
          files[i].matched = true;
        }
      } else if (--count <= 0) {
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
          // ":psearch" uses the preview window
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
            // match in current file
            if (l_g_do_tagpreview != 0) {
              if (!win_valid(curwin_save)) {
                break;
              }
              if (!GETFILE_SUCCESS(getfile(curwin_save->w_buffer->b_fnum, NULL,
                                           NULL, true, lnum, forceit))) {
                break;    // failed to jump to file
              }
            } else {
              setpcmark();
            }
            curwin->w_cursor.lnum = lnum;
            check_cursor(curwin);
          } else {
            if (!GETFILE_SUCCESS(getfile(0, files[depth].name, NULL, true,
                                         files[depth].lnum, forceit))) {
              break;    // failed to jump to file
            }
            // autocommands may have changed the lnum, we don't
            // want that here
            curwin->w_cursor.lnum = files[depth].lnum;
          }
        }
        if (action != ACTION_SHOW) {
          curwin->w_cursor.col = (colnr_T)(startp - line);
          curwin->w_set_curswant = true;
        }

        if (l_g_do_tagpreview != 0
            && curwin != curwin_save && win_valid(curwin_save)) {
          // Return cursor to where we were
          validate_cursor(curwin);
          redraw_later(curwin, UPD_VALID);
          win_enter(curwin_save, true);
        }
        break;
      }
exit_matched:
      matched = false;
      // look for other matches in the rest of the line if we
      // are not at the end of it already
      if (def_regmatch.regprog == NULL
          && action == ACTION_EXPAND
          && !compl_status_sol()
          && *startp != NUL
          && *(startp + utfc_ptr2len(startp)) != NUL) {
        goto search_line;
      }
    }
    line_breakcheck();
    if (action == ACTION_EXPAND) {
      ins_compl_check_keys(30, false);
    }
    if (got_int || ins_compl_interrupted()) {
      break;
    }

    // Read the next line.  When reading an included file and encountering
    // end-of-file, close the file and continue in the file that included
    // it.
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
    if (depth >= 0) {           // we could read the line
      files[depth].lnum++;
      // Remove any CR and LF from the line.
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
  // End of big while (true) loop.

  // Close any files that are still open.
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
  } else if (!found && action != ACTION_EXPAND && !silent) {
    if (got_int || ins_compl_interrupted()) {
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

fpip_end:
  xfree(file_line);
  vim_regfree(regmatch.regprog);
  vim_regfree(incl_regmatch.regprog);
  vim_regfree(def_regmatch.regprog);
}
#endif  // #if 0 - old find_pattern_in_path body

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

/// Get last search pattern
void get_search_pattern(SearchPattern *const pat)
{
  rs_get_search_pattern_shada(pat);
}

/// Get last substitute pattern
void get_substitute_pattern(SearchPattern *const pat)
{
  rs_get_substitute_pattern_shada(pat);
}

/// Set last search pattern
void set_search_pattern(const SearchPattern pat)
{
  rs_set_search_pattern_shada(&pat);
}

/// Set last substitute pattern
void set_substitute_pattern(const SearchPattern pat)
{
  rs_set_substitute_pattern_shada(&pat);
}

/// Set last used search pattern
///
/// @param[in]  is_substitute_pattern  If true set substitute pattern as last
///                                    used. Otherwise sets search pattern.
void set_last_used_pattern(const bool is_substitute_pattern)
{
  rs_set_last_used_pattern(is_substitute_pattern);
}

/// Returns true if search pattern was the last used one
bool search_was_last_used(void)
{
  return rs_search_was_last_used() != 0;
}

// =============================================================================
// Batch accessors for pattern save/restore (Phase 3)
// =============================================================================

/// Save search patterns batch: deep-copy spats[] → saved_spats[], save mr_pattern,
/// last_idx, no_hlsearch. Only acts at top level (save_level was 0).
void nvim_save_search_patterns_batch(void)
{
  for (size_t i = 0; i < ARRAY_SIZE(spats); i++) {
    saved_spats[i] = spats[i];
    if (spats[i].pat != NULL) {
      saved_spats[i].pat = xstrnsave(spats[i].pat, spats[i].patlen);
      saved_spats[i].patlen = spats[i].patlen;
    }
  }
  if (mr_pattern == NULL) {
    saved_mr_pattern = NULL;
    saved_mr_patternlen = 0;
  } else {
    saved_mr_pattern = xstrnsave(mr_pattern, mr_patternlen);
    saved_mr_patternlen = mr_patternlen;
  }
  saved_spats_last_idx = last_idx;
  saved_spats_no_hlsearch = no_hlsearch;
}

/// Restore search patterns batch: free spats[], restore from saved_spats[],
/// restore mr_pattern, last_idx, no_hlsearch. Only acts when save_level reaches 0.
void nvim_restore_search_patterns_batch(void)
{
  for (size_t i = 0; i < ARRAY_SIZE(spats); i++) {
    free_spat(&spats[i]);
    spats[i] = saved_spats[i];
  }
  set_vv_searchforward();
  xfree(mr_pattern);
  mr_pattern = saved_mr_pattern;
  mr_patternlen = saved_mr_patternlen;
  last_idx = saved_spats_last_idx;
  set_no_hlsearch(saved_spats_no_hlsearch);
}

/// Increment save_level and return old value.
int nvim_inc_save_level(void)
{
  return save_level++;
}

/// Decrement save_level and return new value.
int nvim_dec_save_level(void)
{
  return --save_level;
}

/// Save last search pattern for incsearch: deep-copy spats[RE_SEARCH] →
/// saved_last_search_spat, save last_idx, no_hlsearch.
void nvim_save_last_search_spat_batch(void)
{
  saved_last_search_spat = spats[RE_SEARCH];
  if (spats[RE_SEARCH].pat != NULL) {
    saved_last_search_spat.pat = xstrnsave(spats[RE_SEARCH].pat, spats[RE_SEARCH].patlen);
    saved_last_search_spat.patlen = spats[RE_SEARCH].patlen;
  }
  saved_last_idx = last_idx;
  saved_no_hlsearch = no_hlsearch;
}

/// Restore last search pattern for incsearch: free spats[RE_SEARCH],
/// restore from saved_last_search_spat, restore last_idx, no_hlsearch.
void nvim_restore_last_search_spat_batch(void)
{
  xfree(spats[RE_SEARCH].pat);
  spats[RE_SEARCH] = saved_last_search_spat;
  saved_last_search_spat.pat = NULL;
  saved_last_search_spat.patlen = 0;
  set_vv_searchforward();
  last_idx = saved_last_idx;
  set_no_hlsearch(saved_no_hlsearch);
}

/// Increment did_save_last_search_spat and return old value.
int nvim_inc_did_save(void)
{
  return did_save_last_search_spat++;
}

/// Decrement did_save_last_search_spat and return new value.
int nvim_dec_did_save(void)
{
  return --did_save_last_search_spat;
}

/// Call iemsg() for the restore mismatch error.
void nvim_call_iemsg_restore_mismatch(void)
{
  iemsg("restore_last_search_pattern() called more often than"
        " save_last_search_pattern()");
}

/// Save incsearch state (search_match_endcol, search_match_lines).
void nvim_save_incsearch_state_batch(void)
{
  saved_search_match_endcol = search_match_endcol;
  saved_search_match_lines = search_match_lines;
}

/// Restore incsearch state.
void nvim_restore_incsearch_state_batch(void)
{
  search_match_endcol = saved_search_match_endcol;
  search_match_lines = saved_search_match_lines;
}

// =============================================================================
// Batch accessors for search_regcomp and pattern compilation (Phase 4)
// =============================================================================

// nvim_emsg_noprevre is already defined in register.c

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

/// Set mr_pattern from pat (or reversed if rl mode).
void nvim_set_mr_pattern(const char *pat, size_t patlen)
{
  xfree(mr_pattern);
  if (curwin->w_p_rl && *curwin->w_p_rlc == 's') {
    mr_pattern = reverse_text(pat);
  } else {
    mr_pattern = xstrnsave(pat, patlen);
  }
  mr_patternlen = patlen;
}

/// Check if cmdmod has keeppatterns flag.
int nvim_get_cmdmod_keeppatterns(void)
{
  return (cmdmod.cmod_flags & CMOD_KEEPPATTERNS) != 0;
}

/// Batch save_re_pat: update spats[idx] with new pattern.
void nvim_save_re_pat_batch(int idx, const char *pat, size_t patlen, int magic)
{
  if (spats[idx].pat == pat) {
    return;
  }
  free_spat(&spats[idx]);
  spats[idx].pat = xstrnsave(pat, patlen);
  spats[idx].patlen = patlen;
  spats[idx].magic = magic;
  spats[idx].no_scs = no_smartcase;
  spats[idx].timestamp = os_time();
  spats[idx].additional_data = NULL;
  last_idx = idx;
  if (p_hls) {
    redraw_all_later(UPD_SOME_VALID);
  }
  set_no_hlsearch(false);
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

/// Batch set_last_search_pat: update spats[idx] and saved_spats[idx] from a string.
void nvim_set_last_search_pat_batch(const char *s, int idx, int magic, int setlast)
{
  free_spat(&spats[idx]);
  if (*s == NUL) {
    spats[idx].pat = NULL;
    spats[idx].patlen = 0;
  } else {
    spats[idx].patlen = strlen(s);
    spats[idx].pat = xstrnsave(s, spats[idx].patlen);
  }
  spats[idx].timestamp = os_time();
  spats[idx].additional_data = NULL;
  spats[idx].magic = magic;
  spats[idx].no_scs = false;
  spats[idx].off.dir = '/';
  set_vv_searchforward();
  spats[idx].off.line = false;
  spats[idx].off.end = false;
  spats[idx].off.off = 0;
  if (setlast) {
    last_idx = idx;
  }
  if (save_level) {
    free_spat(&saved_spats[idx]);
    saved_spats[idx] = spats[0];
    if (spats[idx].pat == NULL) {
      saved_spats[idx].pat = NULL;
      saved_spats[idx].patlen = 0;
    } else {
      saved_spats[idx].pat = xstrnsave(spats[idx].pat, spats[idx].patlen);
      saved_spats[idx].patlen = spats[idx].patlen;
    }
    saved_spats_last_idx = last_idx;
  }
  if (p_hls && idx == last_idx && !no_hlsearch) {
    redraw_all_later(UPD_SOME_VALID);
  }
}

// nvim_get_emsg_off is already defined in message.c

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

/// Check if spats[idx].pat is NULL.
int nvim_spats_pat_is_null(int idx)
{
  return spats[idx].pat == NULL ? 1 : 0;
}

/// Get spats[idx].pat pointer and patlen.
const char *nvim_spats_get_pat_and_len(int idx, size_t *patlen, int *magic, int *no_scs)
{
  if (patlen) *patlen = spats[idx].patlen;
  if (magic) *magic = spats[idx].magic;
  if (no_scs) *no_scs = spats[idx].no_scs;
  return spats[idx].pat;
}

// =============================================================================
// Batch accessors for ShaDa pattern get/set (Phase 2)
// =============================================================================

/// Copy spats[idx] out to caller-provided buffer.
void nvim_spat_memcpy_out(int idx, SearchPattern *out)
{
  if (idx >= 0 && idx < 2 && out != NULL) {
    memcpy(out, &spats[idx], sizeof(spats[0]));
  }
}

/// Free spats[idx] and copy new value in.
void nvim_spat_memcpy_in(int idx, const SearchPattern *in)
{
  if (idx >= 0 && idx < 2 && in != NULL) {
    free_spat(&spats[idx]);
    memcpy(&spats[idx], in, sizeof(spats[0]));
  }
}

/// Free the pattern and additional_data of spats[idx].
void nvim_free_spat(int idx)
{
  if (idx >= 0 && idx < 2) {
    free_spat(&spats[idx]);
  }
}

/// Clear spats[idx].off fields.
void nvim_clear_spat_off(int idx)
{
  if (idx >= 0 && idx < 2) {
    CLEAR_FIELD(spats[idx].off);
  }
}

/// Clear all spats entries (for free_search_patterns).
void nvim_clear_spats(void)
{
  CLEAR_FIELD(spats);
}

/// Free mr_pattern and reset mr_patternlen.
void nvim_free_mr_pattern(void)
{
  XFREE_CLEAR(mr_pattern);
  mr_patternlen = 0;
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

/// Emit "Pattern not found" error with the pattern from mr_pattern.
void nvim_searchit_emsg_patnotf(int p_ws_val, linenr_T lnum)
{
  if (p_ws_val) {
    semsg(_(e_patnotf2), mr_pattern);
  } else if (lnum == 0) {
    semsg(_(e_search_hit_top_without_match_for_str), mr_pattern);
  } else {
    semsg(_(e_search_hit_bottom_without_match_for_str), mr_pattern);
  }
}

/// Emit "E383: Invalid search string" error using mr_pattern.
void nvim_searchit_emsg_invalid(void)
{
  semsg(_("E383: Invalid search string: %s"), mr_pattern);
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

/// Get the sizeof(regmmatch_T) for stack allocation.
int nvim_regmmatch_size(void)
{
  return (int)sizeof(regmmatch_T);
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

/// Check if spats[0].off.line is set and CPO_LINEOFF is in p_cpo.
int nvim_do_search_check_lineoff(void)
{
  return (spats[0].off.line && vim_strchr(p_cpo, CPO_LINEOFF) != NULL) ? 1 : 0;
}

/// Clear spats[0].off.line and off if CPO_LINEOFF applies.
void nvim_do_search_clear_lineoff(void)
{
  spats[0].off.line = false;
  spats[0].off.off = 0;
}

/// Get the dirc from spats[0].off.dir.
int nvim_do_search_get_dirc(void)
{
  return (uint8_t)spats[0].off.dir;
}

/// Set spats[0].off.dir and update vv:searchforward.
void nvim_do_search_set_dirc(int dirc)
{
  spats[0].off.dir = (char)dirc;
  set_vv_searchforward();
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

/// Get previous search pattern (spats[RE_SEARCH].pat).
char *nvim_do_search_get_search_pat(void)
{
  return spats[RE_SEARCH].pat;
}

/// Get previous subst pattern info.
char *nvim_do_search_get_subst_pat(void)
{
  return spats[RE_SUBST].pat;
}

size_t nvim_do_search_get_subst_patlen(void)
{
  return spats[RE_SUBST].patlen;
}

/// Call skip_regexp_ex for do_search pattern parsing.
/// Returns: pointer to end of regexp.  Sets *newp if copy was made.
char *nvim_do_search_skip_regexp(char *pat, int delim, char **newp)
{
  return skip_regexp_ex(pat, delim, magic_isset(), newp, NULL, NULL);
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

/// Set spats[0].off fields.
void nvim_do_search_set_off(int off_line, int off_end, int64_t off_off)
{
  spats[0].off.line = off_line != 0;
  spats[0].off.end = off_end != 0;
  spats[0].off.off = off_off;
}

/// Get spats[0].off.end for the SEARCH_END computation.
int nvim_do_search_get_off_end(void)
{
  return spats[0].off.end ? 1 : 0;
}

/// Get spats[0].off.line.
int nvim_do_search_get_off_line(void)
{
  return spats[0].off.line ? 1 : 0;
}

/// Get spats[0].off.off.
int64_t nvim_do_search_get_off_off(void)
{
  return spats[0].off.off;
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

  if (!cmd_silent
      && (spats[0].off.line || spats[0].off.end || spats[0].off.off)) {
    off_buf[off_len++] = (char)dirc;
    if (spats[0].off.end) {
      off_buf[off_len++] = 'e';
    } else if (!spats[0].off.line) {
      off_buf[off_len++] = 's';
    }
    off_buf[off_len] = NUL;
    if (spats[0].off.off != 0 || spats[0].off.line) {
      off_len += (size_t)snprintf(off_buf + off_len, sizeof(off_buf) - off_len,
                                  "%+" PRId64, spats[0].off.off);
    }
  }

  const char *p;
  size_t plen;
  if (*searchstr == NUL) {
    p = spats[0].pat;
    plen = spats[0].patlen;
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
  pos_T pos = { .lnum = lnum, .col = col, .coladd = 0 };
  int64_t off = spats[0].off.off;

  if (!spats[0].off.line && off && pos.col < MAXCOL - 2) {
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

  if (!(options & SEARCH_NOOF) || pat_has_semicolon) {
    if (spats[0].off.line) {
      int64_t c = pos.lnum + spats[0].off.off;
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
      int64_t c = spats[0].off.off;
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
  if (oap != NULL && spats[0].off.end) {
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

/// Save spats[0].off, returns opaque data via struct.
SavedSearchOff nvim_do_search_save_off(void)
{
  return (SavedSearchOff){
    spats[0].off.dir,
    spats[0].off.line,
    spats[0].off.end,
    spats[0].off.off
  };
}

/// Restore spats[0].off from saved data.
void nvim_do_search_restore_off(SavedSearchOff saved)
{
  spats[0].off.dir = saved.dir;
  spats[0].off.line = saved.line != 0;
  spats[0].off.end = saved.end != 0;
  spats[0].off.off = saved.off;
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

/// Get curwin->w_p_rlc.
char *nvim_search_get_curwin_w_p_rlc(void)
{
  return curwin->w_p_rlc;
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

// =============================================================================
// Phase 5: searchc() accessors
// =============================================================================

/// Batch save lastc state for searchc() migration.
/// Sets lastc[0], lastcdir, last_t_cmd, and the lastc_bytes/lastc_bytelen.
/// If nchar_len > 0, copies composing_bytes to lastc_bytes.
/// Otherwise, encodes `c` using utf_char2bytes into lastc_bytes.
void nvim_searchc_save_lastc_state(int c, int nchar_len, const char *composing_bytes)
{
  *lastc = (uint8_t)c;
  if (nchar_len > 0) {
    lastc_bytelen = nchar_len;
    memcpy(lastc_bytes, composing_bytes, (size_t)nchar_len);
  } else {
    lastc_bytelen = utf_char2bytes(c, lastc_bytes);
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

/// Check if spats[last_idx].pat matches a given pattern.
int nvim_stat_spats_pat_matches(const char *pat, size_t patlen)
{
  if (pat == NULL || spats[last_idx].pat == NULL) {
    return 0;
  }
  return (strncmp(pat, spats[last_idx].pat, patlen) == 0
          && patlen == spats[last_idx].patlen) ? 1 : 0;
}

/// Copy spats[last_idx].pat using xstrnsave (for cache update).
/// Returns a newly allocated string the caller must xfree.
char *nvim_stat_copy_spats_pat(size_t *out_len)
{
  if (spats[last_idx].pat == NULL) {
    *out_len = 0;
    return NULL;
  }
  *out_len = spats[last_idx].patlen;
  return xstrnsave(spats[last_idx].pat, spats[last_idx].patlen);
}

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

/// Get spats[last_idx].pat and patlen for is_zero_width.
const char *nvim_get_last_spat_pat(size_t *out_len)
{
  *out_len = spats[last_idx].patlen;
  return spats[last_idx].pat;
}

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

/// Check compl_status_adding().
int nvim_search_compl_status_adding(void)
{
  return compl_status_adding() ? 1 : 0;
}

/// Check compl_status_sol().
int nvim_search_compl_status_sol(void)
{
  return compl_status_sol() ? 1 : 0;
}

/// Get ins_compl_len().
int nvim_search_ins_compl_len(void)
{
  return ins_compl_len();
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

// Phase 4: find_pattern_in_path constants
_Static_assert(FIND_ANY == 1, "FIND_ANY mismatch");
_Static_assert(FIND_DEFINE == 2, "FIND_DEFINE mismatch");
_Static_assert(CHECK_PATH == 3, "CHECK_PATH mismatch");
_Static_assert(ACTION_SHOW == 1, "ACTION_SHOW mismatch");
_Static_assert(ACTION_GOTO == 2, "ACTION_GOTO mismatch");
_Static_assert(ACTION_SPLIT == 3, "ACTION_SPLIT mismatch");
_Static_assert(ACTION_SHOW_ALL == 4, "ACTION_SHOW_ALL mismatch");
_Static_assert(ACTION_EXPAND == 5, "ACTION_EXPAND mismatch");

