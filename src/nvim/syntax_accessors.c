// syntax_accessors.c: syntax highlighting - struct definitions, static state,
// helper functions, and FFI accessor functions for Rust interop

#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/fold.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/hashtab_defs.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/indent_c.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/os/input.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/profile.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/runtime.h"
#include "nvim/strings.h"
#include "nvim/syntax.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

// Rust FFI declarations - functions called from C code in this file
extern void rs_load_current_state(synstate_T *from);
extern void rs_update_si_end(stateitem_T *sip, int startcol, int force);
extern void rs_check_state_ends(void);
extern void rs_update_si_attr(int idx);
extern void rs_check_keepend(void);
extern stateitem_T *rs_push_next_match(void);
extern int rs_syn_finish_line(int syncing);
extern char *rs_get_syn_options(char *arg, int *opt_flags, int opt_keyword,
                                int *opt_sync_idx, int opt_has_cont_list,
                                int16_t **opt_cont_list, int16_t **opt_cont_in_list,
                                int16_t **opt_next_list, int *conceal_char, int skip);
extern int rs_get_id_list(char **arg, int keylen, int16_t **list, int skip);
extern void rs_syn_cmd_region(exarg_T *eap, int syncing);
extern int rs_syn_cmd_case(synblock_T *block, const char *arg, const char *arg_end);
extern int rs_syn_cmd_conceal(synblock_T *block, const char *arg, const char *arg_end);
extern int rs_syn_cmd_spell(synblock_T *block, const char *arg, const char *arg_end);
extern int rs_syn_cmd_foldlevel(synblock_T *block, const char *arg, const char *arg_end);

static bool did_syntax_onoff = false;

// different types of offsets that are possible
#define SPO_MS_OFF      0       // match  start offset
#define SPO_ME_OFF      1       // match  end   offset
#define SPO_HS_OFF      2       // highl. start offset
#define SPO_HE_OFF      3       // highl. end   offset
#define SPO_RS_OFF      4       // region start offset
#define SPO_RE_OFF      5       // region end   offset
#define SPO_LC_OFF      6       // leading context offset
#define SPO_COUNT       7

static const char e_trailing_char_after_rsb_str_str[]
  = N_("E890: Trailing char after ']': %s]%s");

// The patterns that are being searched for are stored in a syn_pattern.
// A match item consists of one pattern.
// A start/end item consists of n start patterns and m end patterns.
// A start/skip/end item consists of n start patterns, one skip pattern and m
// end patterns.
// For the latter two, the patterns are always consecutive: start-skip-end.
//
// A character offset can be given for the matched text (_m_start and _m_end)
// and for the actually highlighted text (_h_start and _h_end).
//
// Note that ordering of members is optimized to reduce padding.
// synpat_T is forward-declared in syntax_defs.h
struct synpat_S {
  char sp_type;                         // see SPTYPE_ defines below
  bool sp_syncing;                      // this item used for syncing
  int16_t sp_syn_match_id;              // highlight group ID of pattern
  int16_t sp_off_flags;                 // see below
  int sp_offsets[SPO_COUNT];            // offsets
  int sp_flags;                         // see HL_ defines below
  int sp_cchar;                         // conceal substitute character
  int sp_ic;                            // ignore-case flag for sp_prog
  int sp_sync_idx;                      // sync item index (syncing only)
  int sp_line_id;                       // ID of last line where tried
  int sp_startcol;                      // next match in sp_line_id line
  int16_t *sp_cont_list;                // cont. group IDs, if non-zero
  int16_t *sp_next_list;                // next group IDs, if non-zero
  struct sp_syn sp_syn;                 // struct passed to in_id_list()
  char *sp_pattern;                     // regexp to match, pattern
  regprog_T *sp_prog;                   // regexp to match, program
  syn_time_T sp_time;
};

// syn_cluster_T is forward-declared in syntax_defs.h
struct syn_cluster_S {
  char *scl_name;           // syntax cluster name
  char *scl_name_u;         // uppercase of scl_name
  int16_t *scl_list;        // IDs in this syntax cluster
};

// For the current state we need to remember more than just the idx.
// When si_m_endpos.lnum is 0, the items other than si_idx are unknown.
// (The end positions have the column number of the next char)
// stateitem_T is forward-declared in syntax_defs.h
struct stateitem_S {
  int si_idx;                           // index of syntax pattern or
                                        // KEYWORD_IDX
  int si_id;                            // highlight group ID for keywords
  int si_trans_id;                      // idem, transparency removed
  int si_m_lnum;                        // lnum of the match
  int si_m_startcol;                    // starting column of the match
  lpos_T si_m_endpos;                   // just after end posn of the match
  lpos_T si_h_startpos;                 // start position of the highlighting
  lpos_T si_h_endpos;                   // end position of the highlighting
  lpos_T si_eoe_pos;                    // end position of end pattern
  int si_end_idx;                       // group ID for end pattern or zero
  int si_ends;                          // if match ends before si_m_endpos
  int si_attr;                          // attributes in this state
  int si_flags;                         // HL_HAS_EOL flag in this state, and
                                        // HL_SKIP* for si_next_list
  int si_seqnr;                         // sequence number
  int si_cchar;                         // substitution character for conceal
  int16_t *si_cont_list;                // list of contained groups
  int16_t *si_next_list;                // nextgroup IDs after this item ends
  reg_extmatch_T *si_extmatch;          // \z(...\) matches from start
                                        // pattern
};

// Struct to reduce the number of arguments to get_syn_options(), it's used
// very often.
typedef struct {
  int flags;                   // flags for contained and transparent
  bool keyword;                // true for ":syn keyword"
  int *sync_idx;               // syntax item for "grouphere" argument, NULL
                               // if not allowed
  bool has_cont_list;          // true if "cont_list" can be used
  int16_t *cont_list;          // group IDs for "contains" argument
  int16_t *cont_in_list;       // group IDs for "containedin" argument
  int16_t *next_list;          // group IDs for "nextgroup" argument
} syn_opt_arg_T;

typedef struct {
  proftime_T total;
  int count;
  int match;
  proftime_T slowest;
  proftime_T average;
  int id;
  char *pattern;
} time_entry_T;

#include "syntax_accessors.c.generated.h"

// Rust fold FFI declaration
extern void rs_foldUpdateAll(win_T *win);

static char *(spo_name_tab[SPO_COUNT]) =
{ "ms=", "me=", "hs=", "he=", "rs=", "re=", "lc=" };

// The sp_off_flags are computed like this:
// offset from the start of the matched text: (1 << SPO_XX_OFF)
// offset from the end   of the matched text: (1 << (SPO_XX_OFF + SPO_COUNT))
// When both are present, only one is used.

#define SPTYPE_MATCH    1       // match keyword with this group ID
#define SPTYPE_START    2       // match a regexp, start of item
#define SPTYPE_END      3       // match a regexp, end of item
#define SPTYPE_SKIP     4       // match a regexp, skip within item

#define SYN_ITEMS(buf)  ((synpat_T *)((buf)->b_syn_patterns.ga_data))

#define NONE_IDX        (-2)    // value of sp_sync_idx for "NONE"

// Flags for b_syn_sync_flags:
#define SF_CCOMMENT     0x01    // sync on a C-style comment
#define SF_MATCH        0x02    // sync by matching a pattern

#define SYN_STATE_P(ssp)    ((bufstate_T *)((ssp)->ga_data))

#define MAXKEYWLEN      80          // maximum length of a keyword

// The attributes of the syntax item that has been recognized.
static int current_attr = 0;        // attr of current syntax word
static int current_id = 0;          // ID of current char for syn_get_id()
static int current_trans_id = 0;    // idem, transparency removed
static int current_flags = 0;
static int current_seqnr = 0;
static int current_sub_char = 0;

// Methods of combining two clusters
#define CLUSTER_REPLACE     1   // replace first list with second
#define CLUSTER_ADD         2   // add second list to first
#define CLUSTER_SUBTRACT    3   // subtract second list from first

#define SYN_CLSTR(buf)  ((syn_cluster_T *)((buf)->b_syn_clusters.ga_data))

// Syntax group IDs have different types:
//     0 - 19999  normal syntax groups
// 20000 - 20999  ALLBUT indicator (current_syn_inc_tag added)
// 21000 - 21999  TOP indicator (current_syn_inc_tag added)
// 22000 - 22999  CONTAINED indicator (current_syn_inc_tag added)
// 23000 - 32767  cluster IDs (subtract SYNID_CLUSTER for the cluster ID)
#define SYNID_ALLBUT    MAX_HL_ID   // syntax group ID for contains=ALLBUT
#define SYNID_TOP       21000       // syntax group ID for contains=TOP
#define SYNID_CONTAINED 22000       // syntax group ID for contains=CONTAINED
#define SYNID_CLUSTER   23000       // first syntax group ID for clusters

#define MAX_SYN_INC_TAG 999         // maximum before the above overflow
#define MAX_CLUSTER_ID  (32767 - SYNID_CLUSTER)

// Annoying Hack(TM):  ":syn include" needs this pointer to pass to
// expand_filename().  Most of the other syntax commands don't need it, so
// instead of passing it to them, we stow it here.
static char **syn_cmdlinep;

// Another Annoying Hack(TM):  To prevent rules from other ":syn include"'d
// files from leaking into ALLBUT lists, we assign a unique ID to the
// rules in each ":syn include"'d file.
static int current_syn_inc_tag = 0;
static int running_syn_inc_tag = 0;

// In a hashtable item "hi_key" points to "keyword" in a keyentry.
// This avoids adding a pointer to the hashtable item.
// KE2HIKEY() converts a var pointer to a hashitem key pointer.
// HIKEY2KE() converts a hashitem key pointer to a var pointer.
// HI2KE() converts a hashitem pointer to a var pointer.
static keyentry_T dumkey;
#define KE2HIKEY(kp)  ((kp)->keyword)
#define HIKEY2KE(p)   ((keyentry_T *)((p) - (dumkey.keyword - (char *)&dumkey)))
#define HI2KE(hi)      HIKEY2KE((hi)->hi_key)

// To reduce the time spent in keepend(), remember at which level in the state
// stack the first item with "keepend" is present.  When "-1", there is no
// "keepend" on the stack.
static int keepend_level = -1;

static char msg_no_items[] = N_("No Syntax items defined for this buffer");

// value of si_idx for keywords
#define KEYWORD_IDX     (-1)
// valid of si_cont_list for containing all but contained groups
#define ID_LIST_ALL     ((int16_t *)-1)

static int next_seqnr = 1;              // value to use for si_seqnr

// The next possible match in the current line for any pattern is remembered,
// to avoid having to try for a match in each column.
// If next_match_idx == -1, not tried (in this line) yet.
// If next_match_col == MAXCOL, no match found in this line.
// (All end positions have the column of the char after the end)
static int next_match_col;              // column for start of next match
static lpos_T next_match_m_endpos;      // position for end of next match
static lpos_T next_match_h_startpos;    // pos. for highl. start of next match
static lpos_T next_match_h_endpos;      // pos. for highl. end of next match
static int next_match_idx;              // index of matched item
static int next_match_flags;            // flags for next match
static lpos_T next_match_eos_pos;       // end of start pattn (start region)
static lpos_T next_match_eoe_pos;       // pos. for end of end pattern
static int next_match_end_idx;          // ID of group for end pattn or zero
static reg_extmatch_T *next_match_extmatch = NULL;

// A state stack is an array of integers or stateitem_T, stored in a
// garray_T.  A state stack is invalid if its itemsize entry is zero.
#define INVALID_STATE(ssp)  ((ssp)->ga_itemsize == 0)
#define VALID_STATE(ssp)    ((ssp)->ga_itemsize != 0)

// The current state (within the line) of the recognition engine.
// When current_state.ga_itemsize is 0 the current state is invalid.
static win_T *syn_win;                  // current window for highlighting
static buf_T *syn_buf;                  // current buffer for highlighting
static synblock_T *syn_block;              // current buffer for highlighting
static proftime_T *syn_tm;                 // timeout limit
static linenr_T current_lnum = 0;          // lnum of current state
static colnr_T current_col = 0;            // column of current state
static bool current_state_stored = false;  // true if stored current state
                                           // after setting current_finished
static bool current_finished = false;      // current line has been finished
static garray_T current_state              // current stack of state_items
  = GA_EMPTY_INIT_VALUE;
static int16_t *current_next_list = NULL;  // when non-zero, nextgroup list
static int current_next_flags = 0;         // flags for current_next_list
static int current_line_id = 0;            // unique number for current line

#define CUR_STATE(idx)  ((stateitem_T *)(current_state.ga_data))[idx]

static bool syn_time_on = false;
#define IF_SYN_TIME(p) (p)

// Set the timeout used for syntax highlighting.
// Use NULL to reset, no timeout.
void syn_set_timeout(proftime_T *tm)
{
  syn_tm = tm;
}


// We cannot simply discard growarrays full of state_items or buf_states; we
// have to manually release their extmatch pointers first.
static void clear_syn_state(synstate_T *p)
{
  if (p->sst_stacksize > SST_FIX_STATES) {
#define UNREF_BUFSTATE_EXTMATCH(bs) unref_extmatch((bs)->bs_extmatch)
    GA_DEEP_CLEAR(&(p->sst_union.sst_ga), bufstate_T, UNREF_BUFSTATE_EXTMATCH);
  } else {
    for (int i = 0; i < p->sst_stacksize; i++) {
      unref_extmatch(p->sst_union.sst_stack[i].bs_extmatch);
    }
  }
}


// Try to find a synchronisation point for line "lnum".
//
// This sets current_lnum and the current state.  One of three methods is
// used:
// 1. Search backwards for the end of a C-comment.
// 2. Search backwards for given sync patterns.
// 3. Simply start on a given number of lines above "lnum".

static void save_chartab(char *chartab)
{
  if (syn_block->b_syn_isk == empty_string_option) {
    return;
  }

  memmove(chartab, syn_buf->b_chartab, (size_t)32);
  memmove(syn_buf->b_chartab, syn_win->w_s->b_syn_chartab, (size_t)32);
}

static void restore_chartab(char *chartab)
{
  if (syn_win->w_s->b_syn_isk != empty_string_option) {
    memmove(syn_buf->b_chartab, chartab, (size_t)32);
  }
}

/// Return true if the line-continuation pattern matches in line "lnum".
static int syn_match_linecont(linenr_T lnum)
{
  if (syn_block->b_syn_linecont_prog == NULL) {
    return false;
  }

  regmmatch_T regmatch;
  // chartab array for syn iskeyword
  char buf_chartab[32];
  save_chartab(buf_chartab);

  regmatch.rmm_ic = syn_block->b_syn_linecont_ic;
  regmatch.regprog = syn_block->b_syn_linecont_prog;
  int r = syn_regexec(&regmatch, lnum, 0,
                      IF_SYN_TIME(&syn_block->b_syn_linecont_time));
  syn_block->b_syn_linecont_prog = regmatch.regprog;

  restore_chartab(buf_chartab);
  return r;
}

// Prepare the current state for the start of a line.
static void syn_start_line(void)
{
  current_finished = false;
  current_col = 0;

  // Need to update the end of a start/skip/end that continues from the
  // previous line and regions that have "keepend".
  if (!GA_EMPTY(&current_state)) {
    syn_update_ends(true);
    rs_check_state_ends();
  }

  next_match_idx = -1;
  current_line_id++;
  next_seqnr = 1;
}

/// Check for items in the stack that need their end updated.
///
/// @param startofline  if true the last item is always updated.
///                     if false the item with "keepend" is forcefully updated.
static void syn_update_ends(bool startofline)
{
  stateitem_T *cur_si;

  if (startofline) {
    // Check for a match carried over from a previous line with a
    // contained region.  The match ends as soon as the region ends.
    for (int i = 0; i < current_state.ga_len; i++) {
      cur_si = &CUR_STATE(i);
      if (cur_si->si_idx >= 0
          && (SYN_ITEMS(syn_block)[cur_si->si_idx]).sp_type
          == SPTYPE_MATCH
          && cur_si->si_m_endpos.lnum < current_lnum) {
        cur_si->si_flags |= HL_MATCHCONT;
        cur_si->si_m_endpos.lnum = 0;
        cur_si->si_m_endpos.col = 0;
        cur_si->si_h_endpos = cur_si->si_m_endpos;
        cur_si->si_ends = true;
      }
    }
  }

  // Need to update the end of a start/skip/end that continues from the
  // previous line.  And regions that have "keepend", because they may
  // influence contained items.  If we've just removed "extend"
  // (startofline == 0) then we should update ends of normal regions
  // contained inside "keepend" because "extend" could have extended
  // these "keepend" regions as well as contained normal regions.
  // Then check for items ending in column 0.
  int i = current_state.ga_len - 1;
  if (keepend_level >= 0) {
    for (; i > keepend_level; i--) {
      if (CUR_STATE(i).si_flags & HL_EXTEND) {
        break;
      }
    }
  }

  bool seen_keepend = false;
  for (; i < current_state.ga_len; i++) {
    cur_si = &CUR_STATE(i);
    if ((cur_si->si_flags & HL_KEEPEND)
        || (seen_keepend && !startofline)
        || (i == current_state.ga_len - 1 && startofline)) {
      cur_si->si_h_startpos.col = 0;            // start highl. in col 0
      cur_si->si_h_startpos.lnum = current_lnum;

      if (!(cur_si->si_flags & HL_MATCHCONT)) {
        rs_update_si_end(cur_si, (int)current_col, !startofline ? 1 : 0);
      }

      if (!startofline && (cur_si->si_flags & HL_KEEPEND)) {
        seen_keepend = true;
      }
    }
  }
  rs_check_keepend();
}

/////////////////////////////////////////
// Handling of the state stack cache.

// EXPLANATION OF THE SYNTAX STATE STACK CACHE
//
// To speed up syntax highlighting, the state stack for the start of some
// lines is cached.  These entries can be used to start parsing at that point.
//
// The stack is kept in b_sst_array[] for each buffer.  There is a list of
// valid entries.  b_sst_first points to the first one, then follow sst_next.
// The entries are sorted on line number.  The first entry is often for line 2
// (line 1 always starts with an empty stack).
// There is also a list for free entries.  This construction is used to avoid
// having to allocate and free memory blocks too often.
//
// When making changes to the buffer, this is logged in b_mod_*.  When calling
// update_screen() to update the display, it will call
// syn_stack_apply_changes() for each displayed buffer to adjust the cached
// entries.  The entries which are inside the changed area are removed,
// because they must be recomputed.  Entries below the changed have their line
// number adjusted for deleted/inserted lines, and have their sst_change_lnum
// set to indicate that a check must be made if the changed lines would change
// the cached entry.
//
// When later displaying lines, an entry is stored for each line.  Displayed
// lines are likely to be displayed again, in which case the state at the
// start of the line is needed.
// For not displayed lines, an entry is stored for every so many lines.  These
// entries will be used e.g., when scrolling backwards.  The distance between
// entries depends on the number of lines in the buffer.  For small buffers
// the distance is fixed at SST_DIST, for large buffers there is a fixed
// number of entries SST_MAX_ENTRIES, and the distance is computed.

static void syn_stack_free_block(synblock_T *block)
{
  if (block->b_sst_array == NULL) {
    return;
  }

  for (synstate_T *p = block->b_sst_first; p != NULL; p = p->sst_next) {
    clear_syn_state(p);
  }
  XFREE_CLEAR(block->b_sst_array);
  block->b_sst_first = NULL;
  block->b_sst_len = 0;
}
// Free b_sst_array[] for buffer "buf".
// Used when syntax items changed to force resyncing everywhere.
void syn_stack_free_all(synblock_T *block)
{
  syn_stack_free_block(block);

  // When using "syntax" fold method, must update all folds.
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_s == block && foldmethodIsSyntax(wp)) {
      rs_foldUpdateAll(wp);
    }
  }
}

// Allocate the syntax state stack for syn_buf when needed.
// If the number of entries in b_sst_array[] is much too big or a bit too
// small, reallocate it.
// Also used to allocate b_sst_array[] for the first time.
static void syn_stack_alloc(void)
{
  int len = syn_buf->b_ml.ml_line_count / SST_DIST + Rows * 2;
  if (len < SST_MIN_ENTRIES) {
    len = SST_MIN_ENTRIES;
  } else if (len > SST_MAX_ENTRIES) {
    len = SST_MAX_ENTRIES;
  }
  if (syn_block->b_sst_len > len * 2 || syn_block->b_sst_len < len) {
    // Allocate 50% too much, to avoid reallocating too often.
    len = syn_buf->b_ml.ml_line_count;
    len = (len + len / 2) / SST_DIST + Rows * 2;
    if (len < SST_MIN_ENTRIES) {
      len = SST_MIN_ENTRIES;
    } else if (len > SST_MAX_ENTRIES) {
      len = SST_MAX_ENTRIES;
    }

    if (syn_block->b_sst_array != NULL) {
      // When shrinking the array, cleanup the existing stack.
      // Make sure that all valid entries fit in the new array.
      while (syn_block->b_sst_len - syn_block->b_sst_freecount + 2 > len
             && syn_stack_cleanup()) {}
      if (len < syn_block->b_sst_len - syn_block->b_sst_freecount + 2) {
        len = syn_block->b_sst_len - syn_block->b_sst_freecount + 2;
      }
    }

    assert(len >= 0);
    synstate_T *sstp = xcalloc((size_t)len, sizeof(synstate_T));

    synstate_T *to = sstp - 1;
    if (syn_block->b_sst_array != NULL) {
      // Move the states from the old array to the new one.
      for (synstate_T *from = syn_block->b_sst_first; from != NULL;
           from = from->sst_next) {
        to++;
        *to = *from;
        to->sst_next = to + 1;
      }
    }
    if (to != sstp - 1) {
      to->sst_next = NULL;
      syn_block->b_sst_first = sstp;
      syn_block->b_sst_freecount = len - (int)(to - sstp) - 1;
    } else {
      syn_block->b_sst_first = NULL;
      syn_block->b_sst_freecount = len;
    }

    // Create the list of free entries.
    syn_block->b_sst_firstfree = to + 1;
    while (++to < sstp + len) {
      to->sst_next = to + 1;
    }
    (sstp + len - 1)->sst_next = NULL;

    xfree(syn_block->b_sst_array);
    syn_block->b_sst_array = sstp;
    syn_block->b_sst_len = len;
  }
}

// Check for changes in a buffer to affect stored syntax states.  Uses the
// b_mod_* fields.
// Called from update_screen(), before screen is being updated, once for each
// displayed buffer.
void syn_stack_apply_changes(buf_T *buf)
{
  syn_stack_apply_changes_block(&buf->b_s, buf);

  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if ((wp->w_buffer == buf) && (wp->w_s != &buf->b_s)) {
      syn_stack_apply_changes_block(wp->w_s, buf);
    }
  }
}

static void syn_stack_apply_changes_block(synblock_T *block, buf_T *buf)
{
  synstate_T *prev = NULL;
  for (synstate_T *p = block->b_sst_first; p != NULL;) {
    if (p->sst_lnum + block->b_syn_sync_linebreaks > buf->b_mod_top) {
      linenr_T n = p->sst_lnum + buf->b_mod_xlines;
      if (n <= buf->b_mod_bot) {
        // this state is inside the changed area, remove it
        synstate_T *np = p->sst_next;
        if (prev == NULL) {
          block->b_sst_first = np;
        } else {
          prev->sst_next = np;
        }
        syn_stack_free_entry(block, p);
        p = np;
        continue;
      }
      // This state is below the changed area.  Remember the line
      // that needs to be parsed before this entry can be made valid
      // again.
      if (p->sst_change_lnum != 0 && p->sst_change_lnum > buf->b_mod_top) {
        if (p->sst_change_lnum + buf->b_mod_xlines > buf->b_mod_top) {
          p->sst_change_lnum += buf->b_mod_xlines;
        } else {
          p->sst_change_lnum = buf->b_mod_top;
        }
      }
      if (p->sst_change_lnum == 0
          || p->sst_change_lnum < buf->b_mod_bot) {
        p->sst_change_lnum = buf->b_mod_bot;
      }

      p->sst_lnum = n;
    }
    prev = p;
    p = p->sst_next;
  }
}

/// Reduce the number of entries in the state stack for syn_buf.
///
/// @return  true if at least one entry was freed.
static bool syn_stack_cleanup(void)
{
  synstate_T *prev;
  disptick_T tick;
  int dist;
  bool retval = false;

  if (syn_block->b_sst_first == NULL) {
    return retval;
  }

  // Compute normal distance between non-displayed entries.
  if (syn_block->b_sst_len <= Rows) {
    dist = 999999;
  } else {
    dist = syn_buf->b_ml.ml_line_count / (syn_block->b_sst_len - Rows) + 1;
  }

  // Go through the list to find the "tick" for the oldest entry that can
  // be removed.  Set "above" when the "tick" for the oldest entry is above
  // "b_sst_lasttick" (the display tick wraps around).
  tick = syn_block->b_sst_lasttick;
  bool above = false;
  prev = syn_block->b_sst_first;
  for (synstate_T *p = prev->sst_next; p != NULL; prev = p, p = p->sst_next) {
    if (prev->sst_lnum + dist > p->sst_lnum) {
      if (p->sst_tick > syn_block->b_sst_lasttick) {
        if (!above || p->sst_tick < tick) {
          tick = p->sst_tick;
        }
        above = true;
      } else if (!above && p->sst_tick < tick) {
        tick = p->sst_tick;
      }
    }
  }

  // Go through the list to make the entries for the oldest tick at an
  // interval of several lines.
  prev = syn_block->b_sst_first;
  for (synstate_T *p = prev->sst_next; p != NULL; prev = p, p = p->sst_next) {
    if (p->sst_tick == tick && prev->sst_lnum + dist > p->sst_lnum) {
      // Move this entry from used list to free list
      prev->sst_next = p->sst_next;
      syn_stack_free_entry(syn_block, p);
      p = prev;
      retval = true;
    }
  }
  return retval;
}

// Free the allocated memory for a syn_state item.
// Move the entry into the free list.
static void syn_stack_free_entry(synblock_T *block, synstate_T *p)
{
  clear_syn_state(p);
  p->sst_next = block->b_sst_firstfree;
  block->b_sst_firstfree = p;
  block->b_sst_freecount++;
}

// Find an entry in the list of state stacks at or before "lnum".
// Returns NULL when there is no entry or the first entry is after "lnum".
static synstate_T *syn_stack_find_entry(linenr_T lnum)
{
  synstate_T *prev = NULL;
  for (synstate_T *p = syn_block->b_sst_first; p != NULL; prev = p, p = p->sst_next) {
    if (p->sst_lnum == lnum) {
      return p;
    }
    if (p->sst_lnum > lnum) {
      break;
    }
  }
  return prev;
}




// End of handling of the state stack.
// **************************************

static void invalidate_current_state(void)
{
  nvim_syn_clear_current_state();
  current_state.ga_itemsize = 0;        // mark current_state invalid
  current_next_list = NULL;
  keepend_level = -1;
}

static void validate_current_state(void)
{
  current_state.ga_itemsize = sizeof(stateitem_T);
  ga_set_growsize(&current_state, 3);
}




/// Update an entry in the current_state stack for a start-skip-end pattern.
/// This finds the end of the current item, if it's in the current line.
///
/// @param startcol  where to start searching for the end
/// @param force     when true overrule a previous end
///


/// Get current line in syntax buffer.
static char *syn_getcurline(void)
{
  return ml_get_buf(syn_buf, current_lnum);
}

/// Get length of current line in syntax buffer.
static colnr_T syn_getcurline_len(void)
{
  return ml_get_buf_len(syn_buf, current_lnum);
}

// Call vim_regexec() to find a match with "rmp" in "syn_buf".
// Returns true when there is a match.
static bool syn_regexec(regmmatch_T *rmp, linenr_T lnum, colnr_T col, syn_time_T *st)
{
  int timed_out = 0;
  proftime_T pt;
  const bool l_syn_time_on = syn_time_on;

  if (l_syn_time_on) {
    pt = profile_start();
  }

  if (rmp->regprog == NULL) {
    // This can happen if a previous call to vim_regexec_multi() tried to
    // use the NFA engine, which resulted in NFA_TOO_EXPENSIVE, and
    // compiling the pattern with the other engine fails.
    return false;
  }

  rmp->rmm_maxcol = (colnr_T)syn_buf->b_p_smc;
  int r = vim_regexec_multi(rmp, syn_win, syn_buf, lnum, col, syn_tm, &timed_out);

  if (l_syn_time_on) {
    pt = profile_end(pt);
    st->total = profile_add(st->total, pt);
    if (profile_cmp(pt, st->slowest) < 0) {
      st->slowest = pt;
    }
    st->count++;
    if (r > 0) {
      st->match++;
    }
  }
  if (timed_out && !syn_win->w_s->b_syn_slow) {
    syn_win->w_s->b_syn_slow = true;
    msg(_("'redrawtime' exceeded, syntax highlighting disabled"), 0);
  }

  if (r > 0) {
    rmp->startpos[0].lnum += lnum;
    rmp->endpos[0].lnum += lnum;
    return true;
  }
  return false;
}


// Handle ":syntax conceal" command.
static void syn_cmd_conceal(exarg_T *eap, int syncing)
{
  char *arg = eap->arg;

  eap->nextcmd = find_nextcmd(arg);
  if (eap->skip) {
    return;
  }

  char *next = skiptowhite(arg);
  rs_syn_cmd_conceal(curwin->w_s, arg, next);
}

/// Handle ":syntax case" command.
static void syn_cmd_case(exarg_T *eap, int syncing)
{
  char *arg = eap->arg;

  eap->nextcmd = find_nextcmd(arg);
  if (eap->skip) {
    return;
  }

  char *next = skiptowhite(arg);
  rs_syn_cmd_case(curwin->w_s, arg, next);
}

/// Handle ":syntax foldlevel" command.
static void syn_cmd_foldlevel(exarg_T *eap, int syncing)
{
  char *arg = eap->arg;

  eap->nextcmd = find_nextcmd(arg);
  if (eap->skip) {
    return;
  }

  char *arg_end = skiptowhite(arg);
  rs_syn_cmd_foldlevel(curwin->w_s, arg, arg_end);
}

/// Handle ":syntax spell" command.
static void syn_cmd_spell(exarg_T *eap, int syncing)
{
  char *arg = eap->arg;

  eap->nextcmd = find_nextcmd(arg);
  if (eap->skip) {
    return;
  }

  char *next = skiptowhite(arg);
  rs_syn_cmd_spell(curwin->w_s, arg, next);

  // assume spell checking changed, force a redraw
  redraw_later(curwin, UPD_NOT_VALID);
}

/// Handle ":syntax iskeyword" command.
static void syn_cmd_iskeyword(exarg_T *eap, int syncing)
{
  char *arg = eap->arg;
  char save_chartab[32];
  char *save_isk;

  if (eap->skip) {
    return;
  }

  arg = skipwhite(arg);
  if (*arg == NUL) {
    msg_puts("\n");
    if (curwin->w_s->b_syn_isk != empty_string_option) {
      msg_puts("syntax iskeyword ");
      msg_outtrans(curwin->w_s->b_syn_isk, 0, false);
    } else {
      msg_outtrans(_("syntax iskeyword not set"), 0, false);
    }
  } else {
    if (STRNICMP(arg, "clear", 5) == 0) {
      memmove(curwin->w_s->b_syn_chartab, curbuf->b_chartab, (size_t)32);
      clear_string_option(&curwin->w_s->b_syn_isk);
    } else {
      memmove(save_chartab, curbuf->b_chartab, (size_t)32);
      save_isk = curbuf->b_p_isk;
      curbuf->b_p_isk = xstrdup(arg);

      buf_init_chartab(curbuf, false);
      memmove(curwin->w_s->b_syn_chartab, curbuf->b_chartab, (size_t)32);
      memmove(curbuf->b_chartab, save_chartab, (size_t)32);
      clear_string_option(&curwin->w_s->b_syn_isk);
      curwin->w_s->b_syn_isk = curbuf->b_p_isk;
      curbuf->b_p_isk = save_isk;
    }
  }
  redraw_later(curwin, UPD_NOT_VALID);
}

// Clear all syntax info for one buffer.
void syntax_clear(synblock_T *block)
{
  block->b_syn_error = false;           // clear previous error
  block->b_syn_slow = false;            // clear previous timeout
  block->b_syn_ic = false;              // Use case, by default
  block->b_syn_foldlevel = SYNFLD_START;
  block->b_syn_spell = SYNSPL_DEFAULT;  // default spell checking
  block->b_syn_containedin = false;
  block->b_syn_conceal = false;

  // free the keywords
  clear_keywtab(&block->b_keywtab);
  clear_keywtab(&block->b_keywtab_ic);

  // free the syntax patterns
  for (int i = block->b_syn_patterns.ga_len; --i >= 0;) {
    syn_clear_pattern(block, i);
  }
  ga_clear(&block->b_syn_patterns);

  // free the syntax clusters
  for (int i = block->b_syn_clusters.ga_len; --i >= 0;) {
    syn_clear_cluster(block, i);
  }
  ga_clear(&block->b_syn_clusters);
  block->b_spell_cluster_id = 0;
  block->b_nospell_cluster_id = 0;

  block->b_syn_sync_flags = 0;
  block->b_syn_sync_minlines = 0;
  block->b_syn_sync_maxlines = 0;
  block->b_syn_sync_linebreaks = 0;

  vim_regfree(block->b_syn_linecont_prog);
  block->b_syn_linecont_prog = NULL;
  XFREE_CLEAR(block->b_syn_linecont_pat);
  block->b_syn_folditems = 0;
  clear_string_option(&block->b_syn_isk);

  // free the stored states
  syn_stack_free_all(block);
  invalidate_current_state();

  // Reset the counter for ":syn include"
  running_syn_inc_tag = 0;
}

// Get rid of ownsyntax for window "wp".
void reset_synblock(win_T *wp)
{
  if (wp->w_s != &wp->w_buffer->b_s) {
    syntax_clear(wp->w_s);
    xfree(wp->w_s);
    wp->w_s = &wp->w_buffer->b_s;
  }
}

// Clear syncing info for one buffer.
static void syntax_sync_clear(void)
{
  // free the syntax patterns
  for (int i = curwin->w_s->b_syn_patterns.ga_len; --i >= 0;) {
    if (SYN_ITEMS(curwin->w_s)[i].sp_syncing) {
      syn_remove_pattern(curwin->w_s, i);
    }
  }

  curwin->w_s->b_syn_sync_flags = 0;
  curwin->w_s->b_syn_sync_minlines = 0;
  curwin->w_s->b_syn_sync_maxlines = 0;
  curwin->w_s->b_syn_sync_linebreaks = 0;

  vim_regfree(curwin->w_s->b_syn_linecont_prog);
  curwin->w_s->b_syn_linecont_prog = NULL;
  XFREE_CLEAR(curwin->w_s->b_syn_linecont_pat);
  clear_string_option(&curwin->w_s->b_syn_isk);

  syn_stack_free_all(curwin->w_s);              // Need to recompute all syntax.
}

// Remove one pattern from the buffer's pattern list.
static void syn_remove_pattern(synblock_T *block, int idx)
{
  synpat_T *spp;

  spp = &(SYN_ITEMS(block)[idx]);
  if (spp->sp_flags & HL_FOLD) {
    block->b_syn_folditems--;
  }
  syn_clear_pattern(block, idx);
  memmove(spp, spp + 1, sizeof(synpat_T) * (size_t)(block->b_syn_patterns.ga_len - idx - 1));
  block->b_syn_patterns.ga_len--;
}

// Clear and free one syntax pattern.  When clearing all, must be called from
// last to first!
static void syn_clear_pattern(synblock_T *block, int i)
{
  xfree(SYN_ITEMS(block)[i].sp_pattern);
  vim_regfree(SYN_ITEMS(block)[i].sp_prog);
  // Only free sp_cont_list and sp_next_list of first start pattern
  if (i == 0 || SYN_ITEMS(block)[i - 1].sp_type != SPTYPE_START) {
    xfree(SYN_ITEMS(block)[i].sp_cont_list);
    xfree(SYN_ITEMS(block)[i].sp_next_list);
    xfree(SYN_ITEMS(block)[i].sp_syn.cont_in_list);
  }
}

// Clear and free one syntax cluster.
static void syn_clear_cluster(synblock_T *block, int i)
{
  xfree(SYN_CLSTR(block)[i].scl_name);
  xfree(SYN_CLSTR(block)[i].scl_name_u);
  xfree(SYN_CLSTR(block)[i].scl_list);
}

/// Handle ":syntax clear" command.
static void syn_cmd_clear(exarg_T *eap, int syncing)
{
  char *arg = eap->arg;
  char *arg_end;
  int id;

  eap->nextcmd = find_nextcmd(arg);
  if (eap->skip) {
    return;
  }

  // We have to disable this within ":syn include @group filename",
  // because otherwise @group would get deleted.
  // Only required for Vim 5.x syntax files, 6.0 ones don't contain ":syn
  // clear".
  if (curwin->w_s->b_syn_topgrp != 0) {
    return;
  }

  if (ends_excmd(*arg)) {
    // No argument: Clear all syntax items.
    if (syncing) {
      syntax_sync_clear();
    } else {
      syntax_clear(curwin->w_s);
      if (curwin->w_s == &curwin->w_buffer->b_s) {
        do_unlet(S_LEN("b:current_syntax"), true);
      }
      do_unlet(S_LEN("w:current_syntax"), true);
    }
  } else {
    // Clear the group IDs that are in the argument.
    while (!ends_excmd(*arg)) {
      arg_end = skiptowhite(arg);
      if (*arg == '@') {
        id = syn_scl_namen2id(arg + 1, (int)(arg_end - arg - 1));
        if (id == 0) {
          semsg(_("E391: No such syntax cluster: %s"), arg);
          break;
        }
        // We can't physically delete a cluster without changing
        // the IDs of other clusters, so we do the next best thing
        // and make it empty.
        int scl_id = id - SYNID_CLUSTER;

        XFREE_CLEAR(SYN_CLSTR(curwin->w_s)[scl_id].scl_list);
      } else {
        id = syn_name2id_len(arg, (size_t)(arg_end - arg));
        if (id == 0) {
          semsg(_(e_nogroup), arg);
          break;
        }
        syn_clear_one(id, syncing);
      }
      arg = skipwhite(arg_end);
    }
  }
  redraw_curbuf_later(UPD_SOME_VALID);
  syn_stack_free_all(curwin->w_s);              // Need to recompute all syntax.
}

// Clear one syntax group for the current buffer.
static void syn_clear_one(const int id, const bool syncing)
{
  synpat_T *spp;

  // Clear keywords only when not ":syn sync clear group-name"
  if (!syncing) {
    syn_clear_keyword(id, &curwin->w_s->b_keywtab);
    syn_clear_keyword(id, &curwin->w_s->b_keywtab_ic);
  }

  // clear the patterns for "id"
  for (int idx = curwin->w_s->b_syn_patterns.ga_len; --idx >= 0;) {
    spp = &(SYN_ITEMS(curwin->w_s)[idx]);
    if (spp->sp_syn.id != id || spp->sp_syncing != syncing) {
      continue;
    }
    syn_remove_pattern(curwin->w_s, idx);
  }
}

// Handle ":syntax on" command.
static void syn_cmd_on(exarg_T *eap, int syncing)
{
  syn_cmd_onoff(eap, "syntax");
}

// Handle ":syntax reset" command.
// It actually resets highlighting, not syntax.
static void syn_cmd_reset(exarg_T *eap, int syncing)
{
  eap->nextcmd = check_nextcmd(eap->arg);
  if (!eap->skip) {
    init_highlight(true, true);
  }
}

// Handle ":syntax manual" command.
static void syn_cmd_manual(exarg_T *eap, int syncing)
{
  syn_cmd_onoff(eap, "manual");
}

// Handle ":syntax off" command.
static void syn_cmd_off(exarg_T *eap, int syncing)
{
  syn_cmd_onoff(eap, "nosyntax");
}

static void syn_cmd_onoff(exarg_T *eap, char *name)
  FUNC_ATTR_NONNULL_ALL
{
  eap->nextcmd = check_nextcmd(eap->arg);
  if (!eap->skip) {
    did_syntax_onoff = true;
    char buf[100];
    memcpy(buf, "so ", 4);
    vim_snprintf(buf + 3, sizeof(buf) - 3, SYNTAX_FNAME, name);
    do_cmdline_cmd(buf);
  }
}

void syn_maybe_enable(void)
{
  if (!did_syntax_onoff) {
    exarg_T ea;
    ea.arg = "";
    ea.skip = false;
    syn_cmd_on(&ea, false);
  }
}

/// Handle ":syntax [list]" command: list current syntax words.
///
/// @param syncing  when true: list syncing items
static void syn_cmd_list(exarg_T *eap, int syncing)
{
  char *arg = eap->arg;
  char *arg_end;

  eap->nextcmd = find_nextcmd(arg);
  if (eap->skip) {
    return;
  }

  if (!syntax_present(curwin)) {
    msg(_(msg_no_items), 0);
    return;
  }

  if (syncing) {
    if (curwin->w_s->b_syn_sync_flags & SF_CCOMMENT) {
      msg_puts(_("syncing on C-style comments"));
      syn_lines_msg();
      syn_match_msg();
      return;
    } else if (!(curwin->w_s->b_syn_sync_flags & SF_MATCH)) {
      if (curwin->w_s->b_syn_sync_minlines == 0) {
        msg_puts(_("no syncing"));
      } else {
        if (curwin->w_s->b_syn_sync_minlines == MAXLNUM) {
          msg_puts(_("syncing starts at the first line"));
        } else {
          msg_puts(_("syncing starts "));
          msg_outnum(curwin->w_s->b_syn_sync_minlines);
          msg_puts(_(" lines before top line"));
        }
        syn_match_msg();
      }
      return;
    }
    msg_puts_title(_("\n--- Syntax sync items ---"));
    if (curwin->w_s->b_syn_sync_minlines > 0
        || curwin->w_s->b_syn_sync_maxlines > 0
        || curwin->w_s->b_syn_sync_linebreaks > 0) {
      msg_puts(_("\nsyncing on items"));
      syn_lines_msg();
      syn_match_msg();
    }
  } else {
    msg_puts_title(_("\n--- Syntax items ---"));
  }
  if (ends_excmd(*arg)) {
    // No argument: List all group IDs and all syntax clusters.
    for (int id = 1; id <= highlight_num_groups() && !got_int; id++) {
      syn_list_one(id, syncing, false);
    }
    for (int id = 0; id < curwin->w_s->b_syn_clusters.ga_len && !got_int; id++) {
      syn_list_cluster(id);
    }
  } else {
    // List the group IDs and syntax clusters that are in the argument.
    while (!ends_excmd(*arg) && !got_int) {
      arg_end = skiptowhite(arg);
      if (*arg == '@') {
        int id = syn_scl_namen2id(arg + 1, (int)(arg_end - arg - 1));
        if (id == 0) {
          semsg(_("E392: No such syntax cluster: %s"), arg);
        } else {
          syn_list_cluster(id - SYNID_CLUSTER);
        }
      } else {
        int id = syn_name2id_len(arg, (size_t)(arg_end - arg));
        if (id == 0) {
          semsg(_(e_nogroup), arg);
        } else {
          syn_list_one(id, syncing, true);
        }
      }
      arg = skipwhite(arg_end);
    }
  }
  eap->nextcmd = check_nextcmd(arg);
}

static void syn_lines_msg(void)
{
  if (curwin->w_s->b_syn_sync_maxlines > 0
      || curwin->w_s->b_syn_sync_minlines > 0) {
    msg_puts("; ");
    if (curwin->w_s->b_syn_sync_minlines == MAXLNUM) {
      msg_puts(_("from the first line"));
    } else {
      if (curwin->w_s->b_syn_sync_minlines > 0) {
        msg_puts(_("minimal "));
        msg_outnum(curwin->w_s->b_syn_sync_minlines);
        if (curwin->w_s->b_syn_sync_maxlines) {
          msg_puts(", ");
        }
      }
      if (curwin->w_s->b_syn_sync_maxlines > 0) {
        msg_puts(_("maximal "));
        msg_outnum(curwin->w_s->b_syn_sync_maxlines);
      }
      msg_puts(_(" lines before top line"));
    }
  }
}

static void syn_match_msg(void)
{
  if (curwin->w_s->b_syn_sync_linebreaks > 0) {
    msg_puts(_("; match "));
    msg_outnum(curwin->w_s->b_syn_sync_linebreaks);
    msg_puts(_(" line breaks"));
  }
}

static int last_matchgroup;

/// List one syntax item, for ":syntax" or "syntax list syntax_name".
///
/// @param syncing    when true: list syncing items
/// @param link_only  when true; list link-only too
static void syn_list_one(const int id, const bool syncing, const bool link_only)
{
  bool did_header = false;
  static keyvalue_T namelist1[] = {
    KEYVALUE_ENTRY(HL_DISPLAY, "display"),
    KEYVALUE_ENTRY(HL_CONTAINED, "contained"),
    KEYVALUE_ENTRY(HL_ONELINE, "oneline"),
    KEYVALUE_ENTRY(HL_KEEPEND, "keepend"),
    KEYVALUE_ENTRY(HL_EXTEND, "extend"),
    KEYVALUE_ENTRY(HL_EXCLUDENL, "excludenl"),
    KEYVALUE_ENTRY(HL_TRANSP, "transparent"),
    KEYVALUE_ENTRY(HL_FOLD, "fold"),
    KEYVALUE_ENTRY(HL_CONCEAL, "conceal"),
    KEYVALUE_ENTRY(HL_CONCEALENDS, "concealends"),
  };
  static keyvalue_T namelist2[] = {
    KEYVALUE_ENTRY(HL_SKIPWHITE, "skipwhite"),
    KEYVALUE_ENTRY(HL_SKIPNL, "skipnl"),
    KEYVALUE_ENTRY(HL_SKIPEMPTY, "skipempty"),
  };

  const int hl_id = HLF_D;      // highlight like directories

  // list the keywords for "id"
  if (!syncing) {
    did_header = syn_list_keywords(id, &curwin->w_s->b_keywtab, false, hl_id);
    did_header = syn_list_keywords(id, &curwin->w_s->b_keywtab_ic, did_header, hl_id);
  }

  // list the patterns for "id"
  for (int idx = 0;
       idx < curwin->w_s->b_syn_patterns.ga_len && !got_int;
       idx++) {
    const synpat_T *const spp = &(SYN_ITEMS(curwin->w_s)[idx]);
    if (spp->sp_syn.id != id || spp->sp_syncing != syncing) {
      continue;
    }

    syn_list_header(did_header, 0, id, true);
    did_header = true;
    last_matchgroup = 0;
    if (spp->sp_type == SPTYPE_MATCH) {
      put_pattern("match", ' ', spp, hl_id);
      msg_putchar(' ');
    } else if (spp->sp_type == SPTYPE_START) {
      while (SYN_ITEMS(curwin->w_s)[idx].sp_type == SPTYPE_START) {
        put_pattern("start", '=', &SYN_ITEMS(curwin->w_s)[idx++], hl_id);
      }
      if (SYN_ITEMS(curwin->w_s)[idx].sp_type == SPTYPE_SKIP) {
        put_pattern("skip", '=', &SYN_ITEMS(curwin->w_s)[idx++], hl_id);
      }
      while (idx < curwin->w_s->b_syn_patterns.ga_len
             && SYN_ITEMS(curwin->w_s)[idx].sp_type == SPTYPE_END) {
        put_pattern("end", '=', &SYN_ITEMS(curwin->w_s)[idx++], hl_id);
      }
      idx--;
      msg_putchar(' ');
    }
    syn_list_flags(namelist1, ARRAY_SIZE(namelist1), spp->sp_flags, hl_id);

    if (spp->sp_cont_list != NULL) {
      put_id_list("contains", spp->sp_cont_list, hl_id);
    }

    if (spp->sp_syn.cont_in_list != NULL) {
      put_id_list("containedin", spp->sp_syn.cont_in_list, hl_id);
    }

    if (spp->sp_next_list != NULL) {
      put_id_list("nextgroup", spp->sp_next_list, hl_id);
      syn_list_flags(namelist2, ARRAY_SIZE(namelist2), spp->sp_flags, hl_id);
    }
    if (spp->sp_flags & (HL_SYNC_HERE|HL_SYNC_THERE)) {
      if (spp->sp_flags & HL_SYNC_HERE) {
        msg_puts_hl("grouphere", hl_id, false);
      } else {
        msg_puts_hl("groupthere", hl_id, false);
      }
      msg_putchar(' ');
      if (spp->sp_sync_idx >= 0) {
        msg_outtrans(highlight_group_name(SYN_ITEMS(curwin->w_s)
                                          [spp->sp_sync_idx].sp_syn.id - 1), 0, false);
      } else {
        msg_puts("NONE");
      }
      msg_putchar(' ');
    }
  }

  // list the link, if there is one
  if (highlight_link_id(id - 1) && (did_header || link_only) && !got_int) {
    syn_list_header(did_header, 0, id, true);
    msg_puts_hl("links to", hl_id, false);
    msg_putchar(' ');
    msg_outtrans(highlight_group_name(highlight_link_id(id - 1) - 1), 0, false);
  }
}

static void syn_list_flags(keyvalue_T *nlist, size_t nr_entries, int flags, int hl_id)
{
  for (size_t i = 0; i < nr_entries; i++) {
    if (flags & nlist[i].key) {
      msg_puts_hl(nlist[i].value, hl_id, false);
      msg_putchar(' ');
    }
  }
}

// List one syntax cluster, for ":syntax" or "syntax list syntax_name".
static void syn_list_cluster(int id)
{
  int endcol = 15;

  // slight hack:  roughly duplicate the guts of syn_list_header()
  msg_putchar('\n');
  msg_outtrans(SYN_CLSTR(curwin->w_s)[id].scl_name, 0, false);

  if (msg_col >= endcol) {      // output at least one space
    endcol = msg_col + 1;
  }
  if (Columns <= endcol) {      // avoid hang for tiny window
    endcol = Columns - 1;
  }

  msg_advance(endcol);
  if (SYN_CLSTR(curwin->w_s)[id].scl_list != NULL) {
    put_id_list("cluster", SYN_CLSTR(curwin->w_s)[id].scl_list, HLF_D);
  } else {
    msg_puts_hl("cluster", HLF_D, false);
    msg_puts("=NONE");
  }
}

static void put_id_list(const char *const name, const int16_t *const list, const int hl_id)
{
  msg_puts_hl(name, hl_id, false);
  msg_putchar('=');
  for (const int16_t *p = list; *p; p++) {
    if (*p >= SYNID_ALLBUT && *p < SYNID_TOP) {
      if (p[1]) {
        msg_puts("ALLBUT");
      } else {
        msg_puts("ALL");
      }
    } else if (*p >= SYNID_TOP && *p < SYNID_CONTAINED) {
      msg_puts("TOP");
    } else if (*p >= SYNID_CONTAINED && *p < SYNID_CLUSTER) {
      msg_puts("CONTAINED");
    } else if (*p >= SYNID_CLUSTER) {
      int scl_id = *p - SYNID_CLUSTER;

      msg_putchar('@');
      msg_outtrans(SYN_CLSTR(curwin->w_s)[scl_id].scl_name, 0, false);
    } else {
      msg_outtrans(highlight_group_name(*p - 1), 0, false);
    }
    if (p[1]) {
      msg_putchar(',');
    }
  }
  msg_putchar(' ');
}

static void put_pattern(const char *const s, const int c, const synpat_T *const spp,
                        const int hl_id)
{
  static const char *const sepchars = "/+=-#@\"|'^&";
  int i;

  // May have to write "matchgroup=group"
  if (last_matchgroup != spp->sp_syn_match_id) {
    last_matchgroup = spp->sp_syn_match_id;
    msg_puts_hl("matchgroup", hl_id, false);
    msg_putchar('=');
    if (last_matchgroup == 0) {
      msg_outtrans("NONE", 0, false);
    } else {
      msg_outtrans(highlight_group_name(last_matchgroup - 1), 0, false);
    }
    msg_putchar(' ');
  }

  // Output the name of the pattern and an '=' or ' '.
  msg_puts_hl(s, hl_id, false);
  msg_putchar(c);

  // output the pattern, in between a char that is not in the pattern
  for (i = 0; vim_strchr(spp->sp_pattern, (uint8_t)sepchars[i]) != NULL;) {
    if (sepchars[++i] == NUL) {
      i = 0;            // no good char found, just use the first one
      break;
    }
  }
  msg_putchar(sepchars[i]);
  msg_outtrans(spp->sp_pattern, 0, false);
  msg_putchar(sepchars[i]);

  // output any pattern options
  bool first = true;
  for (i = 0; i < SPO_COUNT; i++) {
    const int mask = (1 << i);
    if (!(spp->sp_off_flags & (mask + (mask << SPO_COUNT)))) {
      continue;
    }
    if (!first) {
      msg_putchar(',');  // Separate with commas.
    }
    msg_puts(spo_name_tab[i]);
    const int n = spp->sp_offsets[i];
    if (i != SPO_LC_OFF) {
      if (spp->sp_off_flags & mask) {
        msg_putchar('s');
      } else {
        msg_putchar('e');
      }
      if (n > 0) {
        msg_putchar('+');
      }
    }
    if (n || i == SPO_LC_OFF) {
      msg_outnum(n);
    }
    first = false;
  }
  msg_putchar(' ');
}

/// List or clear the keywords for one syntax group.
///
/// @param did_header  header has already been printed
///
/// @return            true if the header has been printed.
static bool syn_list_keywords(const int id, const hashtab_T *const ht, bool did_header,
                              const int hl_id)
{
  int prev_contained = 0;
  const int16_t *prev_next_list = NULL;
  const int16_t *prev_cont_in_list = NULL;
  int prev_skipnl = 0;
  int prev_skipwhite = 0;
  int prev_skipempty = 0;

  // Unfortunately, this list of keywords is not sorted on alphabet but on
  // hash value...
  size_t todo = ht->ht_used;
  for (const hashitem_T *hi = ht->ht_array; todo > 0 && !got_int; hi++) {
    if (HASHITEM_EMPTY(hi)) {
      continue;
    }
    todo--;
    for (keyentry_T *kp = HI2KE(hi); kp != NULL && !got_int; kp = kp->ke_next) {
      if (kp->k_syn.id == id) {
        int outlen = 0;
        bool force_newline = false;
        if (prev_contained != (kp->flags & HL_CONTAINED)
            || prev_skipnl != (kp->flags & HL_SKIPNL)
            || prev_skipwhite != (kp->flags & HL_SKIPWHITE)
            || prev_skipempty != (kp->flags & HL_SKIPEMPTY)
            || prev_cont_in_list != kp->k_syn.cont_in_list
            || prev_next_list != kp->next_list) {
          force_newline = true;
        } else {
          outlen = (int)strlen(kp->keyword);
        }
        // output "contained" and "nextgroup" on each line
        if (syn_list_header(did_header, outlen, id, force_newline)) {
          prev_contained = 0;
          prev_next_list = NULL;
          prev_cont_in_list = NULL;
          prev_skipnl = 0;
          prev_skipwhite = 0;
          prev_skipempty = 0;
        }
        did_header = true;
        if (prev_contained != (kp->flags & HL_CONTAINED)) {
          msg_puts_hl("contained", hl_id, false);
          msg_putchar(' ');
          prev_contained = (kp->flags & HL_CONTAINED);
        }
        if (kp->k_syn.cont_in_list != prev_cont_in_list) {
          put_id_list("containedin", kp->k_syn.cont_in_list, hl_id);
          msg_putchar(' ');
          prev_cont_in_list = kp->k_syn.cont_in_list;
        }
        if (kp->next_list != prev_next_list) {
          put_id_list("nextgroup", kp->next_list, hl_id);
          msg_putchar(' ');
          prev_next_list = kp->next_list;
          if (kp->flags & HL_SKIPNL) {
            msg_puts_hl("skipnl", hl_id, false);
            msg_putchar(' ');
            prev_skipnl = (kp->flags & HL_SKIPNL);
          }
          if (kp->flags & HL_SKIPWHITE) {
            msg_puts_hl("skipwhite", hl_id, false);
            msg_putchar(' ');
            prev_skipwhite = (kp->flags & HL_SKIPWHITE);
          }
          if (kp->flags & HL_SKIPEMPTY) {
            msg_puts_hl("skipempty", hl_id, false);
            msg_putchar(' ');
            prev_skipempty = (kp->flags & HL_SKIPEMPTY);
          }
        }
        msg_outtrans(kp->keyword, 0, false);
      }
    }
  }

  return did_header;
}

static void syn_clear_keyword(int id, hashtab_T *ht)
{
  hash_lock(ht);
  int todo = (int)ht->ht_used;
  for (hashitem_T *hi = ht->ht_array; todo > 0; hi++) {
    if (HASHITEM_EMPTY(hi)) {
      continue;
    }
    todo--;
    keyentry_T *kp_prev = NULL;
    for (keyentry_T *kp = HI2KE(hi); kp != NULL;) {
      if (kp->k_syn.id == id) {
        keyentry_T *kp_next = kp->ke_next;
        if (kp_prev == NULL) {
          if (kp_next == NULL) {
            hash_remove(ht, hi);
          } else {
            hi->hi_key = KE2HIKEY(kp_next);
          }
        } else {
          kp_prev->ke_next = kp_next;
        }
        xfree(kp->next_list);
        xfree(kp->k_syn.cont_in_list);
        xfree(kp);
        kp = kp_next;
      } else {
        kp_prev = kp;
        kp = kp->ke_next;
      }
    }
  }
  hash_unlock(ht);
}

// Clear a whole keyword table.
static void clear_keywtab(hashtab_T *ht)
{
  keyentry_T *kp_next;

  int todo = (int)ht->ht_used;
  for (hashitem_T *hi = ht->ht_array; todo > 0; hi++) {
    if (!HASHITEM_EMPTY(hi)) {
      todo--;
      for (keyentry_T *kp = HI2KE(hi); kp != NULL; kp = kp_next) {
        kp_next = kp->ke_next;
        xfree(kp->next_list);
        xfree(kp->k_syn.cont_in_list);
        xfree(kp);
      }
    }
  }
  hash_clear(ht);
  hash_init(ht);
}

/// Add a keyword to the list of keywords.
///
/// @param name name of keyword
/// @param id group ID for this keyword
/// @param flags flags for this keyword
/// @param cont_in_list containedin for this keyword
/// @param next_list nextgroup for this keyword
static void add_keyword(char *const name, size_t namelen, const int id, const int flags,
                        int16_t *const cont_in_list, int16_t *const next_list,
                        const int conceal_char)
{
  char name_folded[MAXKEYWLEN + 1];
  const char *name_ic;
  size_t name_iclen;
  if (curwin->w_s->b_syn_ic) {
    name_ic = str_foldcase(name, (int)namelen, name_folded, MAXKEYWLEN + 1);
    name_iclen = strlen(name_ic);
  } else {
    name_ic = name;
    name_iclen = namelen;
  }

  keyentry_T *const kp = xmalloc(offsetof(keyentry_T, keyword) + name_iclen + 1);
  STRCPY(kp->keyword, name_ic);
  kp->k_syn.id = (int16_t)id;
  kp->k_syn.inc_tag = current_syn_inc_tag;
  kp->flags = flags;
  kp->k_char = conceal_char;
  kp->k_syn.cont_in_list = copy_id_list(cont_in_list);
  if (cont_in_list != NULL) {
    curwin->w_s->b_syn_containedin = true;
  }
  kp->next_list = copy_id_list(next_list);

  const hash_T hash = hash_hash(kp->keyword);
  hashtab_T *const ht = (curwin->w_s->b_syn_ic)
                        ? &curwin->w_s->b_keywtab_ic
                        : &curwin->w_s->b_keywtab;
  hashitem_T *const hi = hash_lookup(ht, kp->keyword,
                                     strlen(kp->keyword), hash);

  // even though it looks like only the kp->keyword member is
  // being used here, vim uses some pointer trickery to get the original
  // struct again later by using knowledge of the offset of the keyword
  // field in the struct. See the definition of the HI2KE macro.
  if (HASHITEM_EMPTY(hi)) {
    // new keyword, add to hashtable
    kp->ke_next = NULL;
    hash_add_item(ht, hi, kp->keyword, hash);
  } else {
    // keyword already exists, prepend to list
    kp->ke_next = HI2KE(hi);
    hi->hi_key = KE2HIKEY(kp);
  }
}

/// Get the start and end of the group name argument.
///
/// @param arg       start of the argument
/// @param name_end  pointer to end of the name
///
/// @return          a pointer to the first argument.
///                  Return NULL if the end of the command was found instead of further args.
static char *get_group_name(char *arg, char **name_end)
{
  *name_end = skiptowhite(arg);
  char *rest = skipwhite(*name_end);

  // Check if there are enough arguments.  The first argument may be a
  // pattern, where '|' is allowed, so only check for NUL.
  if (ends_excmd(*arg) || *rest == NUL) {
    return NULL;
  }
  return rest;
}

/// Check for syntax command option arguments.
/// This can be called at any place in the list of arguments, and just picks
/// out the arguments that are known.  Can be called several times in a row to
/// collect all options in between other arguments.
///
/// @param arg   next argument to be checked
/// @param opt   various things
/// @param skip  true if skipping over command
///
/// @return      a pointer to the next argument (which isn't an option).
///              Return NULL for any error;
static char *get_syn_options(char *arg, syn_opt_arg_T *opt, int *conceal_char, int skip)
{
  return rs_get_syn_options(arg, &opt->flags, opt->keyword,
                            opt->sync_idx, opt->has_cont_list,
                            &opt->cont_list, &opt->cont_in_list,
                            &opt->next_list, conceal_char, skip);
}

// Adjustments to syntax item when declared in a ":syn include"'d file.
// Set the contained flag, and if the item is not already contained, add it
// to the specified top-level group, if any.
static void syn_incl_toplevel(int id, int *flagsp)
{
  if ((*flagsp & HL_CONTAINED) || curwin->w_s->b_syn_topgrp == 0) {
    return;
  }
  *flagsp |= HL_CONTAINED | HL_INCLUDED_TOPLEVEL;
  if (curwin->w_s->b_syn_topgrp >= SYNID_CLUSTER) {
    // We have to alloc this, because syn_combine_list() will free it.
    int16_t *grp_list = xmalloc(2 * sizeof(*grp_list));
    int tlg_id = curwin->w_s->b_syn_topgrp - SYNID_CLUSTER;

    grp_list[0] = (int16_t)id;
    grp_list[1] = 0;
    syn_combine_list(&SYN_CLSTR(curwin->w_s)[tlg_id].scl_list, &grp_list,
                     CLUSTER_ADD);
  }
}

// Handle ":syntax include [@{group-name}] filename" command.
static void syn_cmd_include(exarg_T *eap, int syncing)
{
  char *arg = eap->arg;
  int sgl_id = 1;
  char *group_name_end;
  const char *errormsg = NULL;
  bool source = false;

  eap->nextcmd = find_nextcmd(arg);
  if (eap->skip) {
    return;
  }

  if (arg[0] == '@') {
    arg++;
    char *rest = get_group_name(arg, &group_name_end);
    if (rest == NULL) {
      emsg(_("E397: Filename required"));
      return;
    }
    sgl_id = syn_check_cluster(arg, (int)(group_name_end - arg));
    if (sgl_id == 0) {
      return;
    }
    // separate_nextcmd() and expand_filename() depend on this
    eap->arg = rest;
  }

  // Everything that's left, up to the next command, should be the
  // filename to include.
  eap->argt |= (EX_XFILE | EX_NOSPC);
  separate_nextcmd(eap);
  if (*eap->arg == '<' || *eap->arg == '$' || path_is_absolute(eap->arg)) {
    // For an absolute path, "$VIM/..." or "<sfile>.." we ":source" the
    // file.  Need to expand the file name first.  In other cases
    // ":runtime!" is used.
    source = true;
    if (expand_filename(eap, syn_cmdlinep, &errormsg) == FAIL) {
      if (errormsg != NULL) {
        emsg(errormsg);
      }
      return;
    }
  }

  // Save and restore the existing top-level grouplist id and ":syn
  // include" tag around the actual inclusion.
  if (running_syn_inc_tag >= MAX_SYN_INC_TAG) {
    emsg(_("E847: Too many syntax includes"));
    return;
  }
  int prev_syn_inc_tag = current_syn_inc_tag;
  current_syn_inc_tag = ++running_syn_inc_tag;
  int prev_toplvl_grp = curwin->w_s->b_syn_topgrp;
  curwin->w_s->b_syn_topgrp = sgl_id;
  if (source
      ? do_source(eap->arg, false, DOSO_NONE, NULL) == FAIL
      : source_runtime(eap->arg, DIP_ALL) == FAIL) {
    semsg(_(e_notopen), eap->arg);
  }
  curwin->w_s->b_syn_topgrp = prev_toplvl_grp;
  current_syn_inc_tag = prev_syn_inc_tag;
}

// Handle ":syntax keyword {group-name} [{option}] keyword .." command.
static void syn_cmd_keyword(exarg_T *eap, int syncing)
{
  char *arg = eap->arg;
  char *group_name_end;
  int syn_id;
  char *keyword_copy = NULL;
  syn_opt_arg_T syn_opt_arg;
  int conceal_char = NUL;

  char *rest = get_group_name(arg, &group_name_end);

  if (rest != NULL) {
    if (eap->skip) {
      syn_id = -1;
    } else {
      syn_id = syn_check_group(arg, (size_t)(group_name_end - arg));
    }
    if (syn_id != 0) {
      // Allocate a buffer, for removing backslashes in the keyword.
      keyword_copy = xmalloc(strlen(rest) + 1);
    }
    if (keyword_copy != NULL) {
      syn_opt_arg.flags = 0;
      syn_opt_arg.keyword = true;
      syn_opt_arg.sync_idx = NULL;
      syn_opt_arg.has_cont_list = false;
      syn_opt_arg.cont_in_list = NULL;
      syn_opt_arg.next_list = NULL;

      // The options given apply to ALL keywords, so all options must be
      // found before keywords can be created.
      // 1: collect the options and copy the keywords to keyword_copy.
      int cnt = 0;
      char *p = keyword_copy;
      for (; rest != NULL && !ends_excmd(*rest); rest = skipwhite(rest)) {
        rest = get_syn_options(rest, &syn_opt_arg, &conceal_char, eap->skip);
        if (rest == NULL || ends_excmd(*rest)) {
          break;
        }
        // Copy the keyword, removing backslashes, and add a NUL.
        while (*rest != NUL && !ascii_iswhite(*rest)) {
          if (*rest == '\\' && rest[1] != NUL) {
            rest++;
          }
          *p++ = *rest++;
        }
        *p++ = NUL;
        cnt++;
      }

      if (!eap->skip) {
        // Adjust flags for use of ":syn include".
        syn_incl_toplevel(syn_id, &syn_opt_arg.flags);

        // 2: Add an entry for each keyword.
        size_t kwlen = 0;
        for (char *kw = keyword_copy; --cnt >= 0; kw += kwlen + 1) {
          for (p = vim_strchr(kw, '[');;) {
            if (p == NULL) {
              kwlen = strlen(kw);
            } else {
              *p = NUL;
              kwlen = (size_t)(p - kw);
            }
            add_keyword(kw, kwlen, syn_id, syn_opt_arg.flags,
                        syn_opt_arg.cont_in_list,
                        syn_opt_arg.next_list, conceal_char);
            if (p == NULL) {
              break;
            }
            if (p[1] == NUL) {
              semsg(_("E789: Missing ']': %s"), kw);
              goto error;
            }
            if (p[1] == ']') {
              if (p[2] != NUL) {
                semsg(_(e_trailing_char_after_rsb_str_str), kw, &p[2]);
                goto error;
              }
              kw = p + 1;
              kwlen = 1;
              break;   // skip over the "]"
            }
            const int l = utfc_ptr2len(p + 1);

            memmove(p, p + 1, (size_t)l);
            p += l;
          }
        }
      }

error:
      xfree(keyword_copy);
      xfree(syn_opt_arg.cont_in_list);
      xfree(syn_opt_arg.next_list);
    }
  }

  if (rest != NULL) {
    eap->nextcmd = check_nextcmd(rest);
  } else {
    semsg(_(e_invarg2), arg);
  }

  redraw_curbuf_later(UPD_SOME_VALID);
  syn_stack_free_all(curwin->w_s);              // Need to recompute all syntax.
}

/// Handle ":syntax match {name} [{options}] {pattern} [{options}]".
///
/// Also ":syntax sync match {name} [[grouphere | groupthere] {group-name}] .."
///
/// @param syncing  true for ":syntax sync match .. "
static void syn_cmd_match(exarg_T *eap, int syncing)
{
  char *arg = eap->arg;
  char *group_name_end;
  synpat_T item;                // the item found in the line
  int syn_id;
  syn_opt_arg_T syn_opt_arg;
  int sync_idx = 0;
  int conceal_char = NUL;

  // Isolate the group name, check for validity
  char *rest = get_group_name(arg, &group_name_end);

  // Get options before the pattern
  syn_opt_arg.flags = 0;
  syn_opt_arg.keyword = false;
  syn_opt_arg.sync_idx = syncing ? &sync_idx : NULL;
  syn_opt_arg.has_cont_list = true;
  syn_opt_arg.cont_list = NULL;
  syn_opt_arg.cont_in_list = NULL;
  syn_opt_arg.next_list = NULL;
  rest = get_syn_options(rest, &syn_opt_arg, &conceal_char, eap->skip);

  // get the pattern.
  init_syn_patterns();
  CLEAR_FIELD(item);
  rest = get_syn_pattern(rest, &item);
  if (vim_regcomp_had_eol() && !(syn_opt_arg.flags & HL_EXCLUDENL)) {
    syn_opt_arg.flags |= HL_HAS_EOL;
  }

  // Get options after the pattern
  rest = get_syn_options(rest, &syn_opt_arg, &conceal_char, eap->skip);

  if (rest != NULL) {           // all arguments are valid
    // Check for trailing command and illegal trailing arguments.
    eap->nextcmd = check_nextcmd(rest);
    if (!ends_excmd(*rest) || eap->skip) {
      rest = NULL;
    } else {
      if ((syn_id = syn_check_group(arg, (size_t)(group_name_end - arg))) != 0) {
        syn_incl_toplevel(syn_id, &syn_opt_arg.flags);
        // Store the pattern in the syn_items list
        synpat_T *spp = GA_APPEND_VIA_PTR(synpat_T,
                                          &curwin->w_s->b_syn_patterns);
        *spp = item;
        spp->sp_syncing = syncing;
        spp->sp_type = SPTYPE_MATCH;
        spp->sp_syn.id = (int16_t)syn_id;
        spp->sp_syn.inc_tag = current_syn_inc_tag;
        spp->sp_flags = syn_opt_arg.flags;
        spp->sp_sync_idx = sync_idx;
        spp->sp_cont_list = syn_opt_arg.cont_list;
        spp->sp_syn.cont_in_list = syn_opt_arg.cont_in_list;
        spp->sp_cchar = conceal_char;
        if (syn_opt_arg.cont_in_list != NULL) {
          curwin->w_s->b_syn_containedin = true;
        }
        spp->sp_next_list = syn_opt_arg.next_list;

        // remember that we found a match for syncing on
        if (syn_opt_arg.flags & (HL_SYNC_HERE|HL_SYNC_THERE)) {
          curwin->w_s->b_syn_sync_flags |= SF_MATCH;
        }
        if (syn_opt_arg.flags & HL_FOLD) {
          curwin->w_s->b_syn_folditems++;
        }

        redraw_curbuf_later(UPD_SOME_VALID);
        syn_stack_free_all(curwin->w_s);          // Need to recompute all syntax.
        return;           // don't free the progs and patterns now
      }
    }
  }

  // Something failed, free the allocated memory.
  vim_regfree(item.sp_prog);
  xfree(item.sp_pattern);
  xfree(syn_opt_arg.cont_list);
  xfree(syn_opt_arg.cont_in_list);
  xfree(syn_opt_arg.next_list);

  if (rest == NULL) {
    semsg(_(e_invarg2), arg);
  }
}

/// Handle ":syntax region {group-name} [matchgroup={group-name}]
///              start {start} .. [skip {skip}] end {end} .. [{options}]".
///
/// @param syncing  true for ":syntax sync region .."
static void syn_cmd_region(exarg_T *eap, int syncing)
{
  rs_syn_cmd_region(eap, syncing);
}

// Keep ITEM_* defines available for C wrappers
#define ITEM_START          0
#define ITEM_SKIP           1
#define ITEM_END            2
#define ITEM_MATCHGROUP     3

// A simple syntax group ID comparison function suitable for use in qsort()
static int syn_compare_stub(const void *const v1, const void *const v2)
{
  const int16_t *const s1 = v1;
  const int16_t *const s2 = v2;

  return *s1 > *s2 ? 1 : *s1 < *s2 ? -1 : 0;
}

// Combines lists of syntax clusters.
// *clstr1 and *clstr2 must both be allocated memory; they will be consumed.
static void syn_combine_list(int16_t **const clstr1, int16_t **const clstr2, const int list_op)
{
  size_t count1 = 0;
  size_t count2 = 0;
  const int16_t *g1;
  const int16_t *g2;
  int16_t *clstr = NULL;

  // Handle degenerate cases.
  if (*clstr2 == NULL) {
    return;
  }
  if (*clstr1 == NULL || list_op == CLUSTER_REPLACE) {
    if (list_op == CLUSTER_REPLACE) {
      xfree(*clstr1);
    }
    if (list_op == CLUSTER_REPLACE || list_op == CLUSTER_ADD) {
      *clstr1 = *clstr2;
    } else {
      xfree(*clstr2);
    }
    return;
  }

  for (g1 = *clstr1; *g1; g1++) {
    count1++;
  }
  for (g2 = *clstr2; *g2; g2++) {
    count2++;
  }

  // For speed purposes, sort both lists.
  qsort(*clstr1, count1, sizeof(**clstr1), syn_compare_stub);
  qsort(*clstr2, count2, sizeof(**clstr2), syn_compare_stub);

  // We proceed in two passes; in round 1, we count the elements to place
  // in the new list, and in round 2, we allocate and populate the new
  // list.  For speed, we use a mergesort-like method, adding the smaller
  // of the current elements in each list to the new list.
  for (int round = 1; round <= 2; round++) {
    g1 = *clstr1;
    g2 = *clstr2;
    int count = 0;

    // First, loop through the lists until one of them is empty.
    while (*g1 && *g2) {
      // We always want to add from the first list.
      if (*g1 < *g2) {
        if (round == 2) {
          clstr[count] = *g1;
        }
        count++;
        g1++;
        continue;
      }
      // We only want to add from the second list if we're adding the
      // lists.
      if (list_op == CLUSTER_ADD) {
        if (round == 2) {
          clstr[count] = *g2;
        }
        count++;
      }
      if (*g1 == *g2) {
        g1++;
      }
      g2++;
    }

    // Now add the leftovers from whichever list didn't get finished
    // first.  As before, we only want to add from the second list if
    // we're adding the lists.
    for (; *g1; g1++, count++) {
      if (round == 2) {
        clstr[count] = *g1;
      }
    }
    if (list_op == CLUSTER_ADD) {
      for (; *g2; g2++, count++) {
        if (round == 2) {
          clstr[count] = *g2;
        }
      }
    }

    if (round == 1) {
      // If the group ended up empty, we don't need to allocate any
      // space for it.
      if (count == 0) {
        clstr = NULL;
        break;
      }
      clstr = xmalloc(((size_t)count + 1) * sizeof(*clstr));
      clstr[count] = 0;
    }
  }

  // Finally, put the new list in place.
  xfree(*clstr1);
  xfree(*clstr2);
  *clstr1 = clstr;
}

/// Lookup a syntax cluster name and return its ID.
/// If it is not found, 0 is returned.
static int syn_scl_name2id(char *name)
{
  // Avoid using stricmp() too much, it's slow on some systems
  char *name_u = vim_strsave_up(name);
  int i;
  for (i = curwin->w_s->b_syn_clusters.ga_len; --i >= 0;) {
    if (SYN_CLSTR(curwin->w_s)[i].scl_name_u != NULL
        && strcmp(name_u, SYN_CLSTR(curwin->w_s)[i].scl_name_u) == 0) {
      break;
    }
  }
  xfree(name_u);
  return i < 0 ? 0 : i + SYNID_CLUSTER;
}

/// Like syn_scl_name2id(), but take a pointer + length argument.
static int syn_scl_namen2id(char *linep, int len)
{
  char *name = xstrnsave(linep, (size_t)len);
  int id = syn_scl_name2id(name);
  xfree(name);

  return id;
}

/// Find syntax cluster name in the table and return its ID.
/// The argument is a pointer to the name and the length of the name.
/// If it doesn't exist yet, a new entry is created.
///
/// @return  0 for failure.
static int syn_check_cluster(char *pp, int len)
{
  char *name = xstrnsave(pp, (size_t)len);
  int id = syn_scl_name2id(name);
  if (id == 0) {                        // doesn't exist yet
    id = syn_add_cluster(name);
  } else {
    xfree(name);
  }
  return id;
}

/// Add new syntax cluster and return its ID.
/// "name" must be an allocated string, it will be consumed.
///
/// @return  0 for failure.
static int syn_add_cluster(char *name)
{
  // First call for this growarray: init growing array.
  if (curwin->w_s->b_syn_clusters.ga_data == NULL) {
    curwin->w_s->b_syn_clusters.ga_itemsize = sizeof(syn_cluster_T);
    ga_set_growsize(&curwin->w_s->b_syn_clusters, 10);
  }

  int len = curwin->w_s->b_syn_clusters.ga_len;
  if (len >= MAX_CLUSTER_ID) {
    emsg(_("E848: Too many syntax clusters"));
    xfree(name);
    return 0;
  }

  syn_cluster_T *scp = GA_APPEND_VIA_PTR(syn_cluster_T,
                                         &curwin->w_s->b_syn_clusters);
  CLEAR_POINTER(scp);
  scp->scl_name = name;
  scp->scl_name_u = vim_strsave_up(name);
  scp->scl_list = NULL;

  if (STRICMP(name, "Spell") == 0) {
    curwin->w_s->b_spell_cluster_id = len + SYNID_CLUSTER;
  }
  if (STRICMP(name, "NoSpell") == 0) {
    curwin->w_s->b_nospell_cluster_id = len + SYNID_CLUSTER;
  }

  return len + SYNID_CLUSTER;
}

// Handle ":syntax cluster {cluster-name} [contains={groupname},..]
//              [add={groupname},..] [remove={groupname},..]".
static void syn_cmd_cluster(exarg_T *eap, int syncing)
{
  char *arg = eap->arg;
  char *group_name_end;
  bool got_clstr = false;
  int opt_len;
  int list_op;

  eap->nextcmd = find_nextcmd(arg);
  if (eap->skip) {
    return;
  }

  char *rest = get_group_name(arg, &group_name_end);

  if (rest != NULL) {
    int scl_id = syn_check_cluster(arg, (int)(group_name_end - arg));
    if (scl_id == 0) {
      return;
    }
    scl_id -= SYNID_CLUSTER;

    while (true) {
      if (STRNICMP(rest, "add", 3) == 0
          && (ascii_iswhite(rest[3]) || rest[3] == '=')) {
        opt_len = 3;
        list_op = CLUSTER_ADD;
      } else if (STRNICMP(rest, "remove", 6) == 0
                 && (ascii_iswhite(rest[6]) || rest[6] == '=')) {
        opt_len = 6;
        list_op = CLUSTER_SUBTRACT;
      } else if (STRNICMP(rest, "contains", 8) == 0
                 && (ascii_iswhite(rest[8]) || rest[8] == '=')) {
        opt_len = 8;
        list_op = CLUSTER_REPLACE;
      } else {
        break;
      }

      int16_t *clstr_list = NULL;
      if (get_id_list(&rest, opt_len, &clstr_list, eap->skip) == FAIL) {
        semsg(_(e_invarg2), rest);
        break;
      }
      if (scl_id >= 0) {
        syn_combine_list(&SYN_CLSTR(curwin->w_s)[scl_id].scl_list,
                         &clstr_list, list_op);
      } else {
        xfree(clstr_list);
      }
      got_clstr = true;
    }

    if (got_clstr) {
      redraw_curbuf_later(UPD_SOME_VALID);
      syn_stack_free_all(curwin->w_s);          // Need to recompute all.
    }
  }

  if (!got_clstr) {
    emsg(_("E400: No cluster specified"));
  }
  if (rest == NULL || !ends_excmd(*rest)) {
    semsg(_(e_invarg2), arg);
  }
}

// On first call for current buffer: Init growing array.
static void init_syn_patterns(void)
{
  curwin->w_s->b_syn_patterns.ga_itemsize = sizeof(synpat_T);
  ga_set_growsize(&curwin->w_s->b_syn_patterns, 10);
}

/// Get one pattern for a ":syntax match" or ":syntax region" command.
/// Stores the pattern and program in a synpat_T.
///
/// @return  a pointer to the next argument, or NULL in case of an error.
static char *get_syn_pattern(char *arg, synpat_T *ci)
{
  int idx;

  // need at least three chars
  if (arg == NULL || arg[0] == NUL || arg[1] == NUL || arg[2] == NUL) {
    return NULL;
  }

  char *end = skip_regexp(arg + 1, *arg, true);
  if (*end != *arg) {                       // end delimiter not found
    semsg(_("E401: Pattern delimiter not found: %s"), arg);
    return NULL;
  }
  // store the pattern and compiled regexp program
  ci->sp_pattern = xstrnsave(arg + 1, (size_t)(end - arg) - 1);

  // Make 'cpoptions' empty, to avoid the 'l' flag
  char *cpo_save = p_cpo;
  p_cpo = empty_string_option;
  ci->sp_prog = vim_regcomp(ci->sp_pattern, RE_MAGIC);
  p_cpo = cpo_save;

  if (ci->sp_prog == NULL) {
    return NULL;
  }
  ci->sp_ic = curwin->w_s->b_syn_ic;
  syn_clear_time(&ci->sp_time);

  // Check for a match, highlight or region offset.
  end++;
  do {
    for (idx = SPO_COUNT; --idx >= 0;) {
      if (strncmp(end, spo_name_tab[idx], 3) == 0) {
        break;
      }
    }
    if (idx >= 0) {
      int *p = &(ci->sp_offsets[idx]);
      if (idx != SPO_LC_OFF) {
        switch (end[3]) {
        case 's':
          break;
        case 'b':
          break;
        case 'e':
          idx += SPO_COUNT; break;
        default:
          idx = -1; break;
        }
      }
      if (idx >= 0) {
        ci->sp_off_flags |= (int16_t)(1 << idx);
        if (idx == SPO_LC_OFF) {            // lc=99
          end += 3;
          *p = getdigits_int(&end, true, 0);

          // "lc=" offset automatically sets "ms=" offset
          if (!(ci->sp_off_flags & (1 << SPO_MS_OFF))) {
            ci->sp_off_flags |= (1 << SPO_MS_OFF);
            ci->sp_offsets[SPO_MS_OFF] = *p;
          }
        } else {                          // yy=x+99
          end += 4;
          if (*end == '+') {
            end++;
            *p = getdigits_int(&end, true, 0);    // positive offset
          } else if (*end == '-') {
            end++;
            *p = -getdigits_int(&end, true, 0);   // negative offset
          }
        }
        if (*end != ',') {
          break;
        }
        end++;
      }
    }
  } while (idx >= 0);

  if (!ends_excmd(*end) && !ascii_iswhite(*end)) {
    semsg(_("E402: Garbage after pattern: %s"), arg);
    return NULL;
  }
  return skipwhite(end);
}

/// Handle ":syntax sync .." command.
static void syn_cmd_sync(exarg_T *eap, int syncing)
{
  char *arg_start = eap->arg;
  char *key = NULL;
  bool illegal = false;
  bool finished = false;

  if (ends_excmd(*arg_start)) {
    syn_cmd_list(eap, true);
    return;
  }

  while (!ends_excmd(*arg_start)) {
    char *arg_end = skiptowhite(arg_start);
    char *next_arg = skipwhite(arg_end);
    xfree(key);
    key = vim_strnsave_up(arg_start, (size_t)(arg_end - arg_start));
    if (strcmp(key, "CCOMMENT") == 0) {
      if (!eap->skip) {
        curwin->w_s->b_syn_sync_flags |= SF_CCOMMENT;
      }
      if (!ends_excmd(*next_arg)) {
        arg_end = skiptowhite(next_arg);
        if (!eap->skip) {
          curwin->w_s->b_syn_sync_id =
            (int16_t)syn_check_group(next_arg, (size_t)(arg_end - next_arg));
        }
        next_arg = skipwhite(arg_end);
      } else if (!eap->skip) {
        curwin->w_s->b_syn_sync_id = (int16_t)syn_name2id("Comment");
      }
    } else if (strncmp(key, "LINES", 5) == 0
               || strncmp(key, "MINLINES", 8) == 0
               || strncmp(key, "MAXLINES", 8) == 0
               || strncmp(key, "LINEBREAKS", 10) == 0) {
      if (key[4] == 'S') {
        arg_end = key + 6;
      } else if (key[0] == 'L') {
        arg_end = key + 11;
      } else {
        arg_end = key + 9;
      }
      if (arg_end[-1] != '=' || !ascii_isdigit(*arg_end)) {
        illegal = true;
        break;
      }
      linenr_T n = getdigits_int32(&arg_end, false, 0);
      if (!eap->skip) {
        if (key[4] == 'B') {
          curwin->w_s->b_syn_sync_linebreaks = n;
        } else if (key[1] == 'A') {
          curwin->w_s->b_syn_sync_maxlines = n;
        } else {
          curwin->w_s->b_syn_sync_minlines = n;
        }
      }
    } else if (strcmp(key, "FROMSTART") == 0) {
      if (!eap->skip) {
        curwin->w_s->b_syn_sync_minlines = MAXLNUM;
        curwin->w_s->b_syn_sync_maxlines = 0;
      }
    } else if (strcmp(key, "LINECONT") == 0) {
      if (*next_arg == NUL) {  // missing pattern
        illegal = true;
        break;
      }
      if (curwin->w_s->b_syn_linecont_pat != NULL) {
        emsg(_("E403: syntax sync: line continuations pattern specified twice"));
        finished = true;
        break;
      }
      arg_end = skip_regexp(next_arg + 1, *next_arg, true);
      if (*arg_end != *next_arg) {          // end delimiter not found
        illegal = true;
        break;
      }

      if (!eap->skip) {
        // store the pattern and compiled regexp program
        curwin->w_s->b_syn_linecont_pat =
          xstrnsave(next_arg + 1, (size_t)(arg_end - next_arg) - 1);
        curwin->w_s->b_syn_linecont_ic = curwin->w_s->b_syn_ic;

        // Make 'cpoptions' empty, to avoid the 'l' flag
        char *cpo_save = p_cpo;
        p_cpo = empty_string_option;
        curwin->w_s->b_syn_linecont_prog =
          vim_regcomp(curwin->w_s->b_syn_linecont_pat, RE_MAGIC);
        p_cpo = cpo_save;
        syn_clear_time(&curwin->w_s->b_syn_linecont_time);

        if (curwin->w_s->b_syn_linecont_prog == NULL) {
          XFREE_CLEAR(curwin->w_s->b_syn_linecont_pat);
          finished = true;
          break;
        }
      }
      next_arg = skipwhite(arg_end + 1);
    } else {
      eap->arg = next_arg;
      if (strcmp(key, "MATCH") == 0) {
        syn_cmd_match(eap, true);
      } else if (strcmp(key, "REGION") == 0) {
        syn_cmd_region(eap, true);
      } else if (strcmp(key, "CLEAR") == 0) {
        syn_cmd_clear(eap, true);
      } else {
        illegal = true;
      }
      finished = true;
      break;
    }
    arg_start = next_arg;
  }
  xfree(key);
  if (illegal) {
    semsg(_("E404: Illegal arguments: %s"), arg_start);
  } else if (!finished) {
    eap->nextcmd = check_nextcmd(arg_start);
    redraw_curbuf_later(UPD_SOME_VALID);
    syn_stack_free_all(curwin->w_s);            // Need to recompute all syntax.
  }
}

/// Convert a line of highlight group names into a list of group ID numbers.
/// "arg" should point to the "contains" or "nextgroup" keyword.
/// "arg" is advanced to after the last group name.
/// Careful: the argument is modified (NULs added).
///
/// @param keylen  length of keyword
/// @param list    where to store the resulting list, if not NULL, the list is silently skipped!
///
/// @return        FAIL for some error, OK for success.
static int get_id_list(char **const arg, const int keylen, int16_t **const list, const bool skip)
{
  return rs_get_id_list(arg, keylen, list, skip);
}

// Make a copy of an ID list.
static int16_t *copy_id_list(const int16_t *const list)
{
  if (list == NULL) {
    return NULL;
  }

  int count;
  for (count = 0; list[count]; count++) {}
  const size_t len = ((size_t)count + 1) * sizeof(int16_t);
  int16_t *const retval = xmalloc(len);
  memmove(retval, list, len);

  return retval;
}

/// Check if syntax group "ssp" is in the ID list "list" of "cur_si".
/// "cur_si" can be NULL if not checking the "containedin" list.
/// Used to check if a syntax item is in the "contains" or "nextgroup" list of
/// the current item.
/// This function is called very often, keep it fast!!
///
/// @param cur_si     current item or NULL
/// @param list       id list
/// @param ssp        group id and ":syn include" tag of group
/// @param flags      group flags
static int in_id_list(stateitem_T *cur_si, int16_t *list, struct sp_syn *ssp, int flags)
{
  int retval;
  int16_t id = ssp->id;
  static int depth = 0;

  // If ssp has a "containedin" list and "cur_si" is in it, return true.
  if (cur_si != NULL && ssp->cont_in_list != NULL
      && !(cur_si->si_flags & HL_MATCH)) {
    // Ignore transparent items without a contains argument.  Double check
    // that we don't go back past the first one.
    while ((cur_si->si_flags & HL_TRANS_CONT)
           && cur_si > (stateitem_T *)(current_state.ga_data)) {
      cur_si--;
    }
    // cur_si->si_idx is -1 for keywords, these never contain anything.
    if (cur_si->si_idx >= 0 && in_id_list(NULL, ssp->cont_in_list,
                                          &(SYN_ITEMS(syn_block)[cur_si->si_idx].sp_syn),
                                          SYN_ITEMS(syn_block)[cur_si->si_idx].sp_flags)) {
      return true;
    }
  }

  if (list == NULL) {
    return false;
  }

  // If list is ID_LIST_ALL, we are in a transparent item that isn't
  // inside anything.  Only allow not-contained groups.
  if (list == ID_LIST_ALL) {
    return !(flags & HL_CONTAINED);
  }

  // Is this top-level (i.e. not 'contained') in the file it was declared in?
  // For included files, this is different from HL_CONTAINED, which is set
  // unconditionally.
  bool toplevel = !(flags & HL_CONTAINED) || (flags & HL_INCLUDED_TOPLEVEL);

  // If the first item is "ALLBUT", return true if "id" is NOT in the
  // contains list.  We also require that "id" is at the same ":syn include"
  // level as the list.
  int16_t item = *list;
  if (item >= SYNID_ALLBUT && item < SYNID_CLUSTER) {
    if (item < SYNID_TOP) {
      // ALL or ALLBUT: accept all groups in the same file
      if (item - SYNID_ALLBUT != ssp->inc_tag) {
        return false;
      }
    } else if (item < SYNID_CONTAINED) {
      // TOP: accept all not-contained groups in the same file
      if (item - SYNID_TOP != ssp->inc_tag || !toplevel) {
        return false;
      }
    } else {
      // CONTAINED: accept all contained groups in the same file
      if (item - SYNID_CONTAINED != ssp->inc_tag || toplevel) {
        return false;
      }
    }
    item = *++list;
    retval = false;
  } else {
    retval = true;
  }

  // Return "retval" if id is in the contains list.
  while (item != 0) {
    if (item == id) {
      return retval;
    }
    if (item >= SYNID_CLUSTER) {
      int16_t *scl_list = SYN_CLSTR(syn_block)[item - SYNID_CLUSTER].scl_list;
      // restrict recursiveness to 30 to avoid an endless loop for a
      // cluster that includes itself (indirectly)
      if (scl_list != NULL && depth < 30) {
        depth++;
        int r = in_id_list(NULL, scl_list, ssp, flags);
        depth--;
        if (r) {
          return retval;
        }
      }
    }
    item = *++list;
  }
  return !retval;
}

struct subcommand {
  char *name;                                // subcommand name
  void (*func)(exarg_T *, int);              // function to call
};

static struct subcommand subcommands[] = {
  { "case",      syn_cmd_case },
  { "clear",     syn_cmd_clear },
  { "cluster",   syn_cmd_cluster },
  { "conceal",   syn_cmd_conceal },
  { "enable",    syn_cmd_on },
  { "foldlevel", syn_cmd_foldlevel },
  { "include",   syn_cmd_include },
  { "iskeyword", syn_cmd_iskeyword },
  { "keyword",   syn_cmd_keyword },
  { "list",      syn_cmd_list },
  { "manual",    syn_cmd_manual },
  { "match",     syn_cmd_match },
  { "on",        syn_cmd_on },
  { "off",       syn_cmd_off },
  { "region",    syn_cmd_region },
  { "reset",     syn_cmd_reset },
  { "spell",     syn_cmd_spell },
  { "sync",      syn_cmd_sync },
  { "",          syn_cmd_list },
};

/// ":syntax".
/// This searches the subcommands[] table for the subcommand name, and calls a
/// syntax_subcommand() function to do the rest.
void ex_syntax(exarg_T *eap)
{
  char *arg = eap->arg;
  char *subcmd_end;

  syn_cmdlinep = eap->cmdlinep;

  // isolate subcommand name
  for (subcmd_end = arg; ASCII_ISALPHA(*subcmd_end); subcmd_end++) {}
  char *const subcmd_name = xstrnsave(arg, (size_t)(subcmd_end - arg));
  if (eap->skip) {  // skip error messages for all subcommands
    emsg_skip++;
  }
  size_t i;
  for (i = 0; i < ARRAY_SIZE(subcommands); i++) {
    if (strcmp(subcmd_name, subcommands[i].name) == 0) {
      eap->arg = skipwhite(subcmd_end);
      (subcommands[i].func)(eap, false);
      break;
    }
  }

  if (i == ARRAY_SIZE(subcommands)) {
    semsg(_("E410: Invalid :syntax subcommand: %s"), subcmd_name);
  }

  xfree(subcmd_name);
  if (eap->skip) {
    emsg_skip--;
  }
}

/// @deprecated
void ex_ownsyntax(exarg_T *eap)
{
  if (curwin->w_s == &curwin->w_buffer->b_s) {
    curwin->w_s = xcalloc(1, sizeof(synblock_T));
    hash_init(&curwin->w_s->b_keywtab);
    hash_init(&curwin->w_s->b_keywtab_ic);
    // TODO(vim): Keep the spell checking as it was.
    curwin->w_p_spell = false;  // No spell checking
    // make sure option values are "empty_string_option" instead of NULL
    clear_string_option(&curwin->w_s->b_p_spc);
    clear_string_option(&curwin->w_s->b_p_spf);
    clear_string_option(&curwin->w_s->b_p_spl);
    clear_string_option(&curwin->w_s->b_p_spo);
    clear_string_option(&curwin->w_s->b_syn_isk);
  }

  // Save value of b:current_syntax.
  char *old_value = get_var_value("b:current_syntax");
  if (old_value != NULL) {
    old_value = xstrdup(old_value);
  }

  // Apply the "syntax" autocommand event, this finds and loads the syntax file.
  apply_autocmds(EVENT_SYNTAX, eap->arg, curbuf->b_fname, true, curbuf);

  // Move value of b:current_syntax to w:current_syntax.
  char *new_value = get_var_value("b:current_syntax");
  if (new_value != NULL) {
    set_internal_string_var("w:current_syntax", new_value);
  }

  // Restore value of b:current_syntax.
  if (old_value == NULL) {
    do_unlet(S_LEN("b:current_syntax"), true);
  } else {
    set_internal_string_var("b:current_syntax", old_value);
    xfree(old_value);
  }
}

static enum {
  EXP_SUBCMD,       // expand ":syn" sub-commands
  EXP_CASE,         // expand ":syn case" arguments
  EXP_SPELL,        // expand ":syn spell" arguments
  EXP_SYNC,         // expand ":syn sync" arguments
  EXP_CLUSTER,      // expand ":syn list @cluster" arguments
} expand_what;

// Reset include_link, include_default, include_none to 0.
// Called when we are done expanding.
void reset_expand_highlight(void)
{
  include_link = include_default = include_none = 0;
}

// Handle command line completion for :match and :echohl command: Add "None"
// as highlight group.
void set_context_in_echohl_cmd(expand_T *xp, const char *arg)
{
  xp->xp_context = EXPAND_HIGHLIGHT;
  xp->xp_pattern = (char *)arg;
  include_none = 1;
}

// Handle command line completion for :syntax command.
void set_context_in_syntax_cmd(expand_T *xp, const char *arg)
{
  // Default: expand subcommands.
  xp->xp_context = EXPAND_SYNTAX;
  expand_what = EXP_SUBCMD;
  xp->xp_pattern = (char *)arg;
  include_link = 0;
  include_default = 0;

  if (*arg == NUL) {
    return;
  }

  // (part of) subcommand already typed
  const char *p = skiptowhite(arg);
  if (*p == NUL) {
    return;
  }

  // past first world
  xp->xp_pattern = skipwhite(p);
  if (*skiptowhite(xp->xp_pattern) != NUL) {
    xp->xp_context = EXPAND_NOTHING;
  } else if (STRNICMP(arg, "case", p - arg) == 0) {
    expand_what = EXP_CASE;
  } else if (STRNICMP(arg, "spell", p - arg) == 0) {
    expand_what = EXP_SPELL;
  } else if (STRNICMP(arg, "sync", p - arg) == 0) {
    expand_what = EXP_SYNC;
  } else if (STRNICMP(arg, "list", p - arg) == 0) {
    p = skipwhite(p);
    if (*p == '@') {
      expand_what = EXP_CLUSTER;
    } else {
      xp->xp_context = EXPAND_HIGHLIGHT;
    }
  } else if (STRNICMP(arg, "keyword", p - arg) == 0
             || STRNICMP(arg, "region", p - arg) == 0
             || STRNICMP(arg, "match", p - arg) == 0) {
    xp->xp_context = EXPAND_HIGHLIGHT;
  } else {
    xp->xp_context = EXPAND_NOTHING;
  }
}

// Function given to ExpandGeneric() to obtain the list syntax names for
// expansion.
char *get_syntax_name(expand_T *xp, int idx)
{
  switch (expand_what) {
  case EXP_SUBCMD:
    if (idx < 0 || idx >= (int)ARRAY_SIZE(subcommands)) {
      return NULL;
    }
    return subcommands[idx].name;
  case EXP_CASE: {
    static char *case_args[] = { "match", "ignore", NULL };
    return case_args[idx];
  }
  case EXP_SPELL: {
    static char *spell_args[] =
    { "toplevel", "notoplevel", "default", NULL };
    return spell_args[idx];
  }
  case EXP_SYNC: {
    static char *sync_args[] =
    { "ccomment", "clear", "fromstart",
      "linebreaks=", "linecont", "lines=", "match",
      "maxlines=", "minlines=", "region", NULL };
    return sync_args[idx];
  }
  case EXP_CLUSTER:
    if (idx < curwin->w_s->b_syn_clusters.ga_len) {
      vim_snprintf(xp->xp_buf, EXPAND_BUF_LEN, "@%s",
                   SYN_CLSTR(curwin->w_s)[idx].scl_name);
      return xp->xp_buf;
    } else {
      return NULL;
    }
  }
  return NULL;
}

/// Function called for expression evaluation: get syntax ID at file position.
///
/// @param trans       remove transparency
/// @param spellp      return: can do spell checking
/// @param keep_state  keep state of char at "col"
int syn_get_id(win_T *wp, linenr_T lnum, colnr_T col, int trans, bool *spellp, int keep_state)
{
  // When the position is not after the current position and in the same
  // line of the same window with the same buffer, need to restart parsing.
  if (wp != syn_win || wp->w_buffer != syn_buf || lnum != current_lnum || col < current_col) {
    syntax_start(wp, lnum);
  } else if (col > current_col) {
    // next_match may not be correct when moving around, e.g. with the
    // "skip" expression in searchpair()
    next_match_idx = -1;
  }

  get_syntax_attr(col, spellp, keep_state);

  return trans ? current_trans_id : current_id;
}

// Get extra information about the syntax item.  Must be called right after
// get_syntax_attr().
// Stores the current item sequence nr in "*seqnrp".
// Returns the current flags.
int get_syntax_info(int *seqnrp)
{
  *seqnrp = current_seqnr;
  return current_flags;
}

/// Get the sequence number of the concealed file position.
///
/// @return seqnr if the file position is concealed, 0 otherwise.
int syn_get_concealed_id(win_T *wp, linenr_T lnum, colnr_T col)
{
  int seqnr;

  syn_get_id(wp, lnum, col, false, NULL, false);
  int syntax_flags = get_syntax_info(&seqnr);

  if (syntax_flags & HL_CONCEAL) {
    return seqnr;
  }
  return 0;
}

// C accessor for current_sub_char (used by Rust)
int nvim_get_current_sub_char(void) { return current_sub_char; }

// Return the syntax ID at position "i" in the current stack.
// The caller must have called syn_get_id() before to fill the stack.
// Returns -1 when "i" is out of range.
int syn_get_stack_item(int i)
{
  if (i >= current_state.ga_len) {
    // Need to invalidate the state, because we didn't properly finish it
    // for the last character, "keep_state" was true.
    invalidate_current_state();
    current_col = MAXCOL;
    return -1;
  }
  return CUR_STATE(i).si_id;
}

// ":syntime".
void ex_syntime(exarg_T *eap)
{
  if (strcmp(eap->arg, "on") == 0) {
    syn_time_on = true;
  } else if (strcmp(eap->arg, "off") == 0) {
    syn_time_on = false;
  } else if (strcmp(eap->arg, "clear") == 0) {
    syntime_clear();
  } else if (strcmp(eap->arg, "report") == 0) {
    syntime_report();
  } else {
    semsg(_(e_invarg2), eap->arg);
  }
}

static void syn_clear_time(syn_time_T *st)
{
  st->total = profile_zero();
  st->slowest = profile_zero();
  st->count = 0;
  st->match = 0;
}

// Clear the syntax timing for the current buffer.
static void syntime_clear(void)
{
  synpat_T *spp;

  if (!syntax_present(curwin)) {
    msg(_(msg_no_items), 0);
    return;
  }
  for (int idx = 0; idx < curwin->w_s->b_syn_patterns.ga_len; idx++) {
    spp = &(SYN_ITEMS(curwin->w_s)[idx]);
    syn_clear_time(&spp->sp_time);
  }
}

// Function given to ExpandGeneric() to obtain the possible arguments of the
// ":syntime {on,off,clear,report}" command.
char *get_syntime_arg(expand_T *xp, int idx)
{
  switch (idx) {
  case 0:
    return "on";
  case 1:
    return "off";
  case 2:
    return "clear";
  case 3:
    return "report";
  }
  return NULL;
}

static int syn_compare_syntime(const void *v1, const void *v2)
{
  const time_entry_T *s1 = v1;
  const time_entry_T *s2 = v2;

  return profile_cmp(s1->total, s2->total);
}

// Clear the syntax timing for the current buffer.
static void syntime_report(void)
{
  if (!syntax_present(curwin)) {
    msg(_(msg_no_items), 0);
    return;
  }

  garray_T ga;
  ga_init(&ga, sizeof(time_entry_T), 50);

  proftime_T total_total = profile_zero();
  int total_count = 0;
  time_entry_T *p;
  for (int idx = 0; idx < curwin->w_s->b_syn_patterns.ga_len; idx++) {
    synpat_T *spp = &(SYN_ITEMS(curwin->w_s)[idx]);
    if (spp->sp_time.count > 0) {
      p = GA_APPEND_VIA_PTR(time_entry_T, &ga);
      p->total = spp->sp_time.total;
      total_total = profile_add(total_total, spp->sp_time.total);
      p->count = spp->sp_time.count;
      p->match = spp->sp_time.match;
      total_count += spp->sp_time.count;
      p->slowest = spp->sp_time.slowest;
      proftime_T tm = profile_divide(spp->sp_time.total, spp->sp_time.count);
      p->average = tm;
      p->id = spp->sp_syn.id;
      p->pattern = spp->sp_pattern;
    }
  }

  // Sort on total time. Skip if there are no items to avoid passing NULL
  // pointer to qsort().
  if (ga.ga_len > 1) {
    qsort(ga.ga_data, (size_t)ga.ga_len, sizeof(time_entry_T),
          syn_compare_syntime);
  }

  msg_puts_title(_("  TOTAL      COUNT  MATCH   SLOWEST     AVERAGE   NAME               PATTERN"));
  msg_puts("\n");
  for (int idx = 0; idx < ga.ga_len && !got_int; idx++) {
    p = ((time_entry_T *)ga.ga_data) + idx;

    msg_puts(profile_msg(p->total));
    msg_puts(" ");     // make sure there is always a separating space
    msg_advance(13);
    msg_outnum(p->count);
    msg_puts(" ");
    msg_advance(20);
    msg_outnum(p->match);
    msg_puts(" ");
    msg_advance(26);
    msg_puts(profile_msg(p->slowest));
    msg_puts(" ");
    msg_advance(38);
    msg_puts(profile_msg(p->average));
    msg_puts(" ");
    msg_advance(50);
    msg_outtrans(highlight_group_name(p->id - 1), 0, false);
    msg_puts(" ");

    msg_advance(69);
    int len;
    if (Columns < 80) {
      len = 20;       // will wrap anyway
    } else {
      len = Columns - 70;
    }
    int patlen = (int)strlen(p->pattern);
    len = MIN(len, patlen);
    msg_outtrans_len(p->pattern, len, 0, false);
    msg_puts("\n");
  }
  ga_clear(&ga);
  if (!got_int) {
    msg_puts("\n");
    msg_puts(profile_msg(total_total));
    msg_advance(13);
    msg_outnum(total_count);
    msg_puts("\n");
  }
}

// ============================================================================
// Rust FFI accessor functions
// ============================================================================

/// Get the number of syntax patterns defined for a window.
int nvim_win_get_syn_patterns_len(win_T *win)
{
  return win->w_s->b_syn_patterns.ga_len;
}

/// Get the number of syntax clusters defined for a window.
int nvim_win_get_syn_clusters_len(win_T *win)
{
  return win->w_s->b_syn_clusters.ga_len;
}

/// Get the number of used entries in the keyword hashtab.
int nvim_win_get_keywtab_used(win_T *win)
{
  return (int)win->w_s->b_keywtab.ht_used;
}

/// Get the number of used entries in the case-insensitive keyword hashtab.
int nvim_win_get_keywtab_ic_used(win_T *win)
{
  return (int)win->w_s->b_keywtab_ic.ht_used;
}

// ============================================================================
// synblock_T accessors (syntax block)
// ============================================================================

/// Get b_syn_patterns.ga_len (number of syntax patterns)
int nvim_synblock_get_pattern_count(synblock_T *block)
{
  return block->b_syn_patterns.ga_len;
}

/// Get b_syn_clusters.ga_len (number of syntax clusters)
int nvim_synblock_get_cluster_count(synblock_T *block)
{
  return block->b_syn_clusters.ga_len;
}

/// Get b_syn_ic (ignore case for :syn cmds)
int nvim_synblock_get_syn_ic(synblock_T *block)
{
  return block->b_syn_ic;
}

/// Set b_syn_ic (ignore case for :syn cmds)
void nvim_synblock_set_syn_ic(synblock_T *block, int ic)
{
  block->b_syn_ic = ic;
}

/// Get b_syn_spell (SYNSPL_ values)
int nvim_synblock_get_syn_spell(synblock_T *block)
{
  return block->b_syn_spell;
}

/// Set b_syn_spell (SYNSPL_ values)
void nvim_synblock_set_syn_spell(synblock_T *block, int spell)
{
  block->b_syn_spell = spell;
}

/// Get b_syn_foldlevel
int nvim_synblock_get_syn_foldlevel(synblock_T *block)
{
  return block->b_syn_foldlevel;
}

/// Set b_syn_foldlevel
void nvim_synblock_set_syn_foldlevel(synblock_T *block, int foldlevel)
{
  block->b_syn_foldlevel = foldlevel;
}

/// Get b_syn_containedin (true if any item has containedin)
int nvim_synblock_get_containedin(synblock_T *block)
{
  return block->b_syn_containedin;
}

/// Get b_syn_sync_flags
int nvim_synblock_get_sync_flags(synblock_T *block)
{
  return block->b_syn_sync_flags;
}

/// Get b_syn_sync_id
int16_t nvim_synblock_get_sync_id(synblock_T *block)
{
  return block->b_syn_sync_id;
}

/// Get b_syn_sync_minlines
int nvim_synblock_get_sync_minlines(synblock_T *block)
{
  return (int)block->b_syn_sync_minlines;
}

/// Get b_syn_sync_maxlines
int nvim_synblock_get_sync_maxlines(synblock_T *block)
{
  return (int)block->b_syn_sync_maxlines;
}

/// Get b_syn_sync_linebreaks
int nvim_synblock_get_sync_linebreaks(synblock_T *block)
{
  return (int)block->b_syn_sync_linebreaks;
}

/// Get b_syn_topgrp (for :syntax include)
int nvim_synblock_get_topgrp(synblock_T *block)
{
  return block->b_syn_topgrp;
}

/// Get b_syn_conceal (auto-conceal for :syn cmds)
int nvim_synblock_get_conceal(synblock_T *block)
{
  return block->b_syn_conceal;
}

/// Set b_syn_conceal (auto-conceal for :syn cmds)
void nvim_synblock_set_conceal(synblock_T *block, int conceal)
{
  block->b_syn_conceal = conceal;
}

/// Get b_syn_folditems (number of patterns with HL_FOLD)
int nvim_synblock_get_folditems(synblock_T *block)
{
  return block->b_syn_folditems;
}

/// Get b_sst_len (number of entries in b_sst_array)
int nvim_synblock_get_sst_len(synblock_T *block)
{
  return block->b_sst_len;
}

/// Get b_sst_freecount (number of free entries)
int nvim_synblock_get_sst_freecount(synblock_T *block)
{
  return block->b_sst_freecount;
}

/// Get b_sst_check_lnum (entries after this need to be checked)
int nvim_synblock_get_sst_check_lnum(synblock_T *block)
{
  return (int)block->b_sst_check_lnum;
}

/// Get b_syn_error (true when error occurred in HL)
int nvim_synblock_get_syn_error(synblock_T *block)
{
  return block->b_syn_error ? 1 : 0;
}

/// Get b_syn_slow (true when 'redrawtime' reached)
int nvim_synblock_get_syn_slow(synblock_T *block)
{
  return block->b_syn_slow ? 1 : 0;
}

/// Get b_spell_cluster_id (@Spell cluster ID or 0)
int nvim_synblock_get_spell_cluster_id(synblock_T *block)
{
  return block->b_spell_cluster_id;
}

/// Get b_nospell_cluster_id (@NoSpell cluster ID or 0)
int nvim_synblock_get_nospell_cluster_id(synblock_T *block)
{
  return block->b_nospell_cluster_id;
}

/// Get b_sst_first (first used entry in state array)
synstate_T *nvim_synblock_get_sst_first(synblock_T *block)
{
  return block->b_sst_first;
}

/// Get b_sst_firstfree (first free entry in state array)
synstate_T *nvim_synblock_get_sst_firstfree(synblock_T *block)
{
  return block->b_sst_firstfree;
}

/// Check if b_sst_array is allocated
int nvim_synblock_has_sst_array(synblock_T *block)
{
  return block->b_sst_array != NULL ? 1 : 0;
}

/// Get synpat_T at index from b_syn_patterns
synpat_T *nvim_synblock_get_pattern(synblock_T *block, int idx)
{
  if (idx < 0 || idx >= block->b_syn_patterns.ga_len) {
    return NULL;
  }
  return &SYN_ITEMS(block)[idx];
}

/// Get syn_cluster_T at index from b_syn_clusters
syn_cluster_T *nvim_synblock_get_cluster(synblock_T *block, int idx)
{
  if (idx < 0 || idx >= block->b_syn_clusters.ga_len) {
    return NULL;
  }
  return &SYN_CLSTR(block)[idx];
}

// ============================================================================
// synstate_T accessors (syntax state)
// ============================================================================

/// Get sst_next (next entry in used or free list)
synstate_T *nvim_synstate_get_next(synstate_T *state)
{
  return state->sst_next;
}

/// Get sst_lnum (line number for this state)
int nvim_synstate_get_lnum(synstate_T *state)
{
  return (int)state->sst_lnum;
}

/// Get sst_stacksize (number of states on the stack)
int nvim_synstate_get_stacksize(synstate_T *state)
{
  return state->sst_stacksize;
}

/// Get sst_next_flags (flags for sst_next_list)
int nvim_synstate_get_next_flags(synstate_T *state)
{
  return state->sst_next_flags;
}

/// Get sst_tick (tick when last displayed)
int nvim_synstate_get_tick(synstate_T *state)
{
  return (int)state->sst_tick;
}

/// Get sst_change_lnum (line where change may have invalidated state)
int nvim_synstate_get_change_lnum(synstate_T *state)
{
  return (int)state->sst_change_lnum;
}

// ============================================================================
// synpat_T accessors (syntax pattern)
// ============================================================================

/// Get sp_type (SPTYPE_* values)
int nvim_synpat_get_type(synpat_T *pat)
{
  return (int)pat->sp_type;
}

/// Get sp_syncing (this item used for syncing)
int nvim_synpat_get_syncing(synpat_T *pat)
{
  return pat->sp_syncing ? 1 : 0;
}

/// Get sp_syn_match_id (highlight group ID of pattern)
int16_t nvim_synpat_get_syn_match_id(synpat_T *pat)
{
  return pat->sp_syn_match_id;
}

/// Get sp_off_flags (offset flags)
int16_t nvim_synpat_get_off_flags(synpat_T *pat)
{
  return pat->sp_off_flags;
}

/// Get sp_flags (HL_ flags)
int nvim_synpat_get_flags(synpat_T *pat)
{
  return pat->sp_flags;
}

/// Get sp_cchar (conceal substitute character)
int nvim_synpat_get_cchar(synpat_T *pat)
{
  return pat->sp_cchar;
}

/// Get sp_ic (ignore-case flag for sp_prog)
int nvim_synpat_get_ic(synpat_T *pat)
{
  return pat->sp_ic;
}

/// Get sp_sync_idx (sync item index, syncing only)
int nvim_synpat_get_sync_idx(synpat_T *pat)
{
  return pat->sp_sync_idx;
}

/// Get sp_pattern (pattern string)
char *nvim_synpat_get_pattern(synpat_T *pat)
{
  return pat->sp_pattern;
}

/// Get sp_syn.id (highlight group ID)
int16_t nvim_synpat_get_syn_id(synpat_T *pat)
{
  return pat->sp_syn.id;
}

/// Get sp_syn.inc_tag (include tag)
int nvim_synpat_get_syn_inc_tag(synpat_T *pat)
{
  return pat->sp_syn.inc_tag;
}

// ============================================================================
// syn_cluster_T accessors (syntax cluster)
// ============================================================================

/// Get scl_name (cluster name)
char *nvim_syncluster_get_name(syn_cluster_T *cluster)
{
  return cluster->scl_name;
}

/// Get scl_name_u (uppercase cluster name)
char *nvim_syncluster_get_name_u(syn_cluster_T *cluster)
{
  return cluster->scl_name_u;
}

// ============================================================================
// stateitem_T accessors (current state item)
// ============================================================================

/// Get si_idx (index of syntax pattern or KEYWORD_IDX)
int nvim_stateitem_get_idx(stateitem_T *item)
{
  return item->si_idx;
}

/// Get si_id (highlight group ID for keywords)
int nvim_stateitem_get_id(stateitem_T *item)
{
  return item->si_id;
}

/// Get si_trans_id (highlight group ID, transparency removed)
int nvim_stateitem_get_trans_id(stateitem_T *item)
{
  return item->si_trans_id;
}

/// Get si_m_lnum (lnum of the match)
int nvim_stateitem_get_m_lnum(stateitem_T *item)
{
  return item->si_m_lnum;
}

/// Get si_m_startcol (starting column of the match)
int nvim_stateitem_get_m_startcol(stateitem_T *item)
{
  return item->si_m_startcol;
}

/// Get si_attr (attributes in this state)
int nvim_stateitem_get_attr(stateitem_T *item)
{
  return item->si_attr;
}

/// Get si_flags (HL_ flags and skip flags)
int nvim_stateitem_get_flags(stateitem_T *item)
{
  return item->si_flags;
}

/// Get si_seqnr (sequence number)
int nvim_stateitem_get_seqnr(stateitem_T *item)
{
  return item->si_seqnr;
}

/// Get si_cchar (substitution character for conceal)
int nvim_stateitem_get_cchar(stateitem_T *item)
{
  return item->si_cchar;
}

/// Get si_end_idx (group ID for end pattern or zero)
int nvim_stateitem_get_end_idx(stateitem_T *item)
{
  return item->si_end_idx;
}

/// Get si_ends (if match ends before si_m_endpos)
int nvim_stateitem_get_ends(stateitem_T *item)
{
  return item->si_ends;
}

// ============================================================================
// keyentry_T accessors (keyword entry)
// ============================================================================

/// Get ke_next (next entry with identical keyword)
keyentry_T *nvim_keyentry_get_next(keyentry_T *ke)
{
  return ke->ke_next;
}

/// Get k_syn.id (highlight group ID)
int16_t nvim_keyentry_get_syn_id(keyentry_T *ke)
{
  return ke->k_syn.id;
}

/// Get k_syn.inc_tag (include tag)
int nvim_keyentry_get_syn_inc_tag(keyentry_T *ke)
{
  return ke->k_syn.inc_tag;
}

/// Get flags
int nvim_keyentry_get_flags(keyentry_T *ke)
{
  return ke->flags;
}

/// Get k_char (conceal substitute character)
int nvim_keyentry_get_char(keyentry_T *ke)
{
  return ke->k_char;
}

/// Get keyword string
char *nvim_keyentry_get_keyword(keyentry_T *ke)
{
  return ke->keyword;
}

// ============================================================================
// Syntax state global accessors (for Rust interop)
// ============================================================================

/// Get the current line number being processed
int nvim_syn_get_current_lnum(void)
{
  return (int)current_lnum;
}

/// Get the current column being processed
int nvim_syn_get_current_col(void)
{
  return (int)current_col;
}

/// Check if the current line has been finished
int nvim_syn_is_current_finished(void)
{
  return current_finished ? 1 : 0;
}

/// Check if the current state has been stored
int nvim_syn_is_current_state_stored(void)
{
  return current_state_stored ? 1 : 0;
}

/// Get the current state stack size (number of state items)
int nvim_syn_get_current_state_len(void)
{
  return current_state.ga_len;
}

/// Check if the current state is valid
int nvim_syn_is_current_state_valid(void)
{
  return VALID_STATE(&current_state) ? 1 : 0;
}

/// Get the current highlight ID
int nvim_syn_get_current_id(void)
{
  return current_id;
}

/// Get the current transparent ID
int nvim_syn_get_current_trans_id(void)
{
  return current_trans_id;
}

/// Get the current attribute
int nvim_syn_get_current_attr(void)
{
  return current_attr;
}

/// Get the current flags
int nvim_syn_get_current_flags(void)
{
  return current_flags;
}

/// Get the current sequence number
int nvim_syn_get_current_seqnr(void)
{
  return current_seqnr;
}

/// Get the current substitution character
int nvim_syn_get_current_sub_char(void)
{
  return current_sub_char;
}

/// Get the current next flags
int nvim_syn_get_current_next_flags(void)
{
  return current_next_flags;
}

/// Get the keepend level
int nvim_syn_get_keepend_level(void)
{
  return keepend_level;
}

/// Get state item at index (NULL if out of bounds)
stateitem_T *nvim_syn_get_cur_state(int idx)
{
  if (idx < 0 || idx >= current_state.ga_len) {
    return NULL;
  }
  return &CUR_STATE(idx);
}

/// Get the current synblock
synblock_T *nvim_syn_get_synblock(void)
{
  return syn_block;
}

/// Count items with HL_FOLD flag in current state
int nvim_syn_count_fold_items(void)
{
  int count = 0;
  for (int i = 0; i < current_state.ga_len; i++) {
    if (CUR_STATE(i).si_flags & HL_FOLD) {
      count++;
    }
  }
  return count;
}

// ============================================================================
// Phase 4: Pattern matching accessors
// ============================================================================

/// Get sp_prog (compiled regex program) - returns NULL if not compiled
regprog_T *nvim_synpat_get_prog(synpat_T *pat)
{
  return pat->sp_prog;
}

/// Check if pattern has a compiled program
int nvim_synpat_has_prog(synpat_T *pat)
{
  return pat->sp_prog != NULL;
}

/// Get sp_cont_list (contains list)
int16_t *nvim_synpat_get_cont_list(synpat_T *pat)
{
  return pat->sp_cont_list;
}

/// Get sp_next_list (nextgroup list)
int16_t *nvim_synpat_get_next_list(synpat_T *pat)
{
  return pat->sp_next_list;
}

/// Get sp_syn.cont_in_list (containedin list)
int16_t *nvim_synpat_get_cont_in_list(synpat_T *pat)
{
  return pat->sp_syn.cont_in_list;
}

/// Check if the pattern has a contains list
int nvim_synpat_has_cont_list(synpat_T *pat)
{
  return pat->sp_cont_list != NULL;
}

/// Check if the pattern has a nextgroup list
int nvim_synpat_has_next_list(synpat_T *pat)
{
  return pat->sp_next_list != NULL;
}

/// Check if the pattern has a containedin list
int nvim_synpat_has_cont_in_list(synpat_T *pat)
{
  return pat->sp_syn.cont_in_list != NULL;
}

// ============================================================================
// Phase 4: Keyword hashtable accessors
// ============================================================================

/// Get the matching-case keyword hashtable from synblock
hashtab_T *nvim_synblock_get_keywtab(synblock_T *block)
{
  return &block->b_keywtab;
}

/// Get the ignore-case keyword hashtable from synblock
hashtab_T *nvim_synblock_get_keywtab_ic(synblock_T *block)
{
  return &block->b_keywtab_ic;
}

/// Check if the matching-case keyword hashtable has entries
int nvim_synblock_has_keywords(synblock_T *block)
{
  return block->b_keywtab.ht_used > 0;
}

/// Check if the ignore-case keyword hashtable has entries
int nvim_synblock_has_keywords_ic(synblock_T *block)
{
  return block->b_keywtab_ic.ht_used > 0;
}

/// Get number of entries in matching-case keyword hashtable
size_t nvim_synblock_keywtab_count(synblock_T *block)
{
  return block->b_keywtab.ht_used;
}

/// Get number of entries in ignore-case keyword hashtable
size_t nvim_synblock_keywtab_ic_count(synblock_T *block)
{
  return block->b_keywtab_ic.ht_used;
}

// ============================================================================
// Phase 4: Keyentry accessors
// ============================================================================

/// Get ke_next_list (nextgroup list for keyword)
int16_t *nvim_keyentry_get_next_list(keyentry_T *ke)
{
  return ke->next_list;
}

/// Get k_syn.cont_in_list (containedin list for keyword)
int16_t *nvim_keyentry_get_cont_in_list(keyentry_T *ke)
{
  return ke->k_syn.cont_in_list;
}

/// Check if keyword has a nextgroup list
int nvim_keyentry_has_next_list(keyentry_T *ke)
{
  return ke->next_list != NULL;
}

/// Check if keyword has a containedin list
int nvim_keyentry_has_cont_in_list(keyentry_T *ke)
{
  return ke->k_syn.cont_in_list != NULL;
}

// ============================================================================
// Phase 4: Cluster list accessors
// ============================================================================

/// Get scl_list (cluster contains list)
int16_t *nvim_syncluster_get_list(syn_cluster_T *cluster)
{
  return cluster->scl_list;
}

/// Check if cluster has a list
int nvim_syncluster_has_list(syn_cluster_T *cluster)
{
  return cluster->scl_list != NULL;
}

// ============================================================================
// Phase 4: ID list iteration helpers
// ============================================================================

/// Get the first item in an ID list (returns 0 if list is NULL)
int16_t nvim_id_list_first(int16_t *list)
{
  if (list == NULL) {
    return 0;
  }
  return *list;
}

/// Get the item at index in an ID list (no bounds checking - caller responsibility)
int16_t nvim_id_list_get(int16_t *list, int idx)
{
  return list[idx];
}

/// Check if list starts with ALLBUT/TOP/CONTAINED marker
int nvim_id_list_is_special(int16_t *list)
{
  if (list == NULL) {
    return 0;
  }
  int16_t first = *list;
  return first >= SYNID_ALLBUT && first < SYNID_CLUSTER;
}

/// Count items in an ID list (terminated by 0)
int nvim_id_list_count(int16_t *list)
{
  if (list == NULL) {
    return 0;
  }
  int count = 0;
  while (*list != 0) {
    count++;
    list++;
  }
  return count;
}

// ============================================================================
// Phase 4: Pattern matching state accessors
// ============================================================================

/// Get next_match_idx (index of next pattern to match)
int nvim_syn_get_next_match_idx(void)
{
  return next_match_idx;
}

/// Get next_match_col (column where next match starts)
int nvim_syn_get_next_match_col(void)
{
  return next_match_col;
}

/// Check if there is a pending next match
int nvim_syn_has_next_match(void)
{
  return next_match_idx >= 0;
}

/// Get current_next_list (nextgroup list for current state)
int16_t *nvim_syn_get_current_next_list(void)
{
  return current_next_list;
}

/// Check if there is a current nextgroup list
int nvim_syn_has_current_next_list(void)
{
  return current_next_list != NULL;
}

// ============================================================================
// Phase 5: Cluster & containedin logic accessors
// ============================================================================


/// Get the current synblock from curwin->w_s
synblock_T *nvim_syn_get_curwin_synblock(void)
{
  return curwin->w_s;
}

/// Get the spell cluster ID from a synblock
int nvim_synblock_get_spell_cluster(synblock_T *block)
{
  return block->b_spell_cluster_id;
}

/// Get the nospell cluster ID from a synblock
int nvim_synblock_get_nospell_cluster(synblock_T *block)
{
  return block->b_nospell_cluster_id;
}

/// Check if a stateitem has the HL_TRANS_CONT flag
int nvim_stateitem_has_trans_cont(stateitem_T *item)
{
  return (item->si_flags & HL_TRANS_CONT) != 0;
}

/// Check if a stateitem has the HL_MATCH flag
int nvim_stateitem_has_match(stateitem_T *item)
{
  return (item->si_flags & HL_MATCH) != 0;
}

/// Get si_cont_list (containedin list for state item)
int16_t *nvim_stateitem_get_cont_list(stateitem_T *item)
{
  return item->si_cont_list;
}

/// Check if stateitem has a containedin list
int nvim_stateitem_has_cont_list(stateitem_T *item)
{
  return item->si_cont_list != NULL;
}

// ============================================================================
// Phase 6: Command & user interface accessors
// ============================================================================

/// Get the current syntax topgrp (for :syn include)
int nvim_syn_get_topgrp(void)
{
  return curwin->w_s->b_syn_topgrp;
}

/// Set the current syntax topgrp
void nvim_syn_set_topgrp(int topgrp)
{
  curwin->w_s->b_syn_topgrp = topgrp;
}

/// Get the syntax block's conceal setting
int nvim_synblock_get_conceal_setting(synblock_T *block)
{
  return block->b_syn_conceal;
}

/// Get the syntax block's case ignore setting
int nvim_synblock_get_ic_setting(synblock_T *block)
{
  return block->b_syn_ic;
}

/// Get the number of subcommands
int nvim_syn_get_subcommand_count(void)
{
  return (int)(sizeof(subcommands) / sizeof(subcommands[0]));
}

/// Get subcommand name by index
const char *nvim_syn_get_subcommand_name(int idx)
{
  int count = (int)(sizeof(subcommands) / sizeof(subcommands[0]));
  if (idx < 0 || idx >= count) {
    return NULL;
  }
  return subcommands[idx].name;
}

/// Check if a pattern at index is for syncing
int nvim_synblock_pattern_is_syncing(synblock_T *block, int idx)
{
  if (idx < 0 || idx >= block->b_syn_patterns.ga_len) {
    return 0;
  }
  return SYN_ITEMS(block)[idx].sp_syncing;
}

/// Get the hl group name from a pattern's syn.id
/// Note: This returns the pattern's highlight group ID minus 1
int nvim_synpat_get_hl_group(synpat_T *pat)
{
  return pat->sp_syn.id - 1;
}

/// Count patterns with a specific highlight group ID
int nvim_synblock_count_patterns_for_id(synblock_T *block, int id)
{
  int count = 0;
  for (int i = 0; i < block->b_syn_patterns.ga_len; i++) {
    if (SYN_ITEMS(block)[i].sp_syn.id == id) {
      count++;
    }
  }
  return count;
}

/// Get expand_what variable (for command completion)
int nvim_syn_get_expand_what(void)
{
  return expand_what;
}

/// Set expand_what variable
void nvim_syn_set_expand_what(int what)
{
  expand_what = what;
}

// ============================================================================
// Cluster ID accessors (for Rust FFI)
// ============================================================================

/// Get cluster ID from synblock at index
/// Returns the cluster ID (SYNID_CLUSTER + idx) or 0 if invalid
int nvim_synblock_get_cluster_id(synblock_T *block, int idx)
{
  if (block == NULL || idx < 0 || idx >= block->b_syn_clusters.ga_len) {
    return 0;
  }
  return SYNID_CLUSTER + idx;
}

/// Get cluster ID from a cluster pointer
/// This computes the ID by finding the offset in the current synblock's clusters
int nvim_syncluster_get_id(syn_cluster_T *cluster)
{
  if (cluster == NULL || curwin == NULL || curwin->w_s == NULL) {
    return 0;
  }
  synblock_T *block = curwin->w_s;
  syn_cluster_T *clusters = (syn_cluster_T *)block->b_syn_clusters.ga_data;
  if (clusters == NULL) {
    return 0;
  }
  int idx = (int)(cluster - clusters);
  if (idx < 0 || idx >= block->b_syn_clusters.ga_len) {
    return 0;
  }
  return SYNID_CLUSTER + idx;
}

// ============================================================================
// Phase 24.1: State Management Helpers (for Rust FFI)
// ============================================================================

/// Check if a state item at idx has a position spanning past current line
/// Used by store_current_state to decide if state can be stored
int nvim_syn_state_item_spans_line(int idx, int lnum)
{
  if (idx < 0 || idx >= current_state.ga_len) {
    return 0;
  }
  stateitem_T *cur_si = &CUR_STATE(idx);
  if (cur_si->si_h_startpos.lnum >= lnum
      || cur_si->si_m_endpos.lnum >= lnum
      || cur_si->si_h_endpos.lnum >= lnum
      || (cur_si->si_end_idx && cur_si->si_eoe_pos.lnum >= lnum)) {
    return 1;
  }
  return 0;
}

/// Find a state entry in the synblock at or before given line
/// Returns the entry or NULL
synstate_T *nvim_syn_stack_find_entry(int lnum)
{
  return syn_stack_find_entry((linenr_T)lnum);
}

/// Remove a state entry from the used list and move to free list
void nvim_syn_stack_remove_entry(synstate_T *sp)
{
  if (sp == NULL || syn_block == NULL) {
    return;
  }
  synstate_T *p;
  if (syn_block->b_sst_first == sp) {
    syn_block->b_sst_first = sp->sst_next;
  } else {
    for (p = syn_block->b_sst_first; p != NULL; p = p->sst_next) {
      if (p->sst_next == sp) {
        break;
      }
    }
    if (p != NULL) {
      p->sst_next = sp->sst_next;
    }
  }
  syn_stack_free_entry(syn_block, sp);
}

/// Allocate a new state entry for the given line
/// Returns NULL if no free entries or after insert position
synstate_T *nvim_syn_stack_alloc_entry(int lnum, synstate_T *after)
{
  if (syn_block == NULL) {
    return NULL;
  }

  // If no free items, cleanup the array first
  if (syn_block->b_sst_freecount == 0) {
    syn_stack_cleanup();
  }

  // Still no free items?
  if (syn_block->b_sst_freecount == 0) {
    return NULL;
  }

  // Take the first item from the free list
  synstate_T *p = syn_block->b_sst_firstfree;
  syn_block->b_sst_firstfree = p->sst_next;
  syn_block->b_sst_freecount--;

  if (after == NULL) {
    // Insert in front of the list
    p->sst_next = syn_block->b_sst_first;
    syn_block->b_sst_first = p;
  } else {
    // Insert in list after *after
    p->sst_next = after->sst_next;
    after->sst_next = p;
  }

  p->sst_stacksize = 0;
  p->sst_lnum = (linenr_T)lnum;
  return p;
}

/// Store the current state into a synstate entry
/// This copies current_state items to the synstate's bufstate array
void nvim_syn_store_state_to_entry(synstate_T *sp)
{
  if (sp == NULL) {
    return;
  }

  // Clear any existing state
  clear_syn_state(sp);
  sp->sst_stacksize = current_state.ga_len;

  bufstate_T *bp;
  if (current_state.ga_len > SST_FIX_STATES) {
    // Need to use growarray for long stacks
    ga_init(&sp->sst_union.sst_ga, (int)sizeof(bufstate_T), 1);
    ga_grow(&sp->sst_union.sst_ga, current_state.ga_len);
    sp->sst_union.sst_ga.ga_len = current_state.ga_len;
    bp = SYN_STATE_P(&(sp->sst_union.sst_ga));
  } else {
    bp = sp->sst_union.sst_stack;
  }

  for (int i = 0; i < sp->sst_stacksize; i++) {
    bp[i].bs_idx = CUR_STATE(i).si_idx;
    bp[i].bs_flags = CUR_STATE(i).si_flags;
    bp[i].bs_seqnr = CUR_STATE(i).si_seqnr;
    bp[i].bs_cchar = CUR_STATE(i).si_cchar;
    bp[i].bs_extmatch = ref_extmatch(CUR_STATE(i).si_extmatch);
  }

  sp->sst_next_flags = current_next_flags;
  sp->sst_next_list = current_next_list;
  sp->sst_tick = display_tick;
  sp->sst_change_lnum = 0;
}

/// Mark current state as stored
void nvim_syn_set_state_stored(int stored)
{
  current_state_stored = stored ? true : false;
}

/// Call clear_current_state()
void nvim_syn_clear_current_state(void)
{
#define UNREF_STATEITEM_EXTMATCH(si) unref_extmatch((si)->si_extmatch)
  GA_DEEP_CLEAR(&current_state, stateitem_T, UNREF_STATEITEM_EXTMATCH);
}

/// Call validate_current_state()
void nvim_syn_validate_current_state(void)
{
  validate_current_state();
}

/// Call invalidate_current_state()
void nvim_syn_invalidate_current_state(void)
{
  invalidate_current_state();
}

/// Set keepend_level
void nvim_syn_set_keepend_level(int level)
{
  keepend_level = level;
}

/// Grow current_state array and set item at index
void nvim_syn_grow_current_state(int size)
{
  ga_grow(&current_state, size);
}

/// Set current_state.ga_len
void nvim_syn_set_current_state_len(int len)
{
  current_state.ga_len = len;
}

/// Set current_next_list
void nvim_syn_set_current_next_list(int16_t *list)
{
  current_next_list = list;
}

/// Set current_next_flags
void nvim_syn_set_current_next_flags(int flags)
{
  current_next_flags = flags;
}

/// Set current_lnum
void nvim_syn_set_current_lnum(int lnum)
{
  current_lnum = (linenr_T)lnum;
}

/// Get sst_next_list from a synstate
int16_t *nvim_synstate_get_next_list(synstate_T *state)
{
  if (state == NULL) {
    return NULL;
  }
  return state->sst_next_list;
}

/// Get bufstate item from synstate at index
/// Returns NULL if index out of bounds
bufstate_T *nvim_synstate_get_bufstate(synstate_T *state, int idx)
{
  if (state == NULL || idx < 0 || idx >= state->sst_stacksize) {
    return NULL;
  }
  bufstate_T *bp;
  if (state->sst_stacksize > SST_FIX_STATES) {
    bp = SYN_STATE_P(&(state->sst_union.sst_ga));
  } else {
    bp = state->sst_union.sst_stack;
  }
  return &bp[idx];
}

/// Get bs_idx from bufstate
int nvim_bufstate_get_idx(bufstate_T *bs)
{
  return bs ? bs->bs_idx : 0;
}

/// Get bs_flags from bufstate
int nvim_bufstate_get_flags(bufstate_T *bs)
{
  return bs ? bs->bs_flags : 0;
}

/// Get bs_seqnr from bufstate
int nvim_bufstate_get_seqnr(bufstate_T *bs)
{
  return bs ? bs->bs_seqnr : 0;
}

/// Get bs_cchar from bufstate
int nvim_bufstate_get_cchar(bufstate_T *bs)
{
  return bs ? bs->bs_cchar : 0;
}

/// Get bs_extmatch from bufstate (opaque pointer)
reg_extmatch_T *nvim_bufstate_get_extmatch(bufstate_T *bs)
{
  return bs ? bs->bs_extmatch : NULL;
}

/// Set stateitem fields at index (used by load_current_state)
void nvim_syn_set_cur_state_item(int idx, int si_idx, int si_flags, int si_seqnr,
                                  int si_cchar, reg_extmatch_T *extmatch)
{
  if (idx < 0 || idx >= current_state.ga_len) {
    return;
  }
  CUR_STATE(idx).si_idx = si_idx;
  CUR_STATE(idx).si_flags = si_flags;
  CUR_STATE(idx).si_seqnr = si_seqnr;
  CUR_STATE(idx).si_cchar = si_cchar;
  CUR_STATE(idx).si_extmatch = ref_extmatch(extmatch);
  CUR_STATE(idx).si_ends = false;
  CUR_STATE(idx).si_m_lnum = 0;
  CUR_STATE(idx).si_next_list = NULL;
  if (si_idx >= 0) {
    CUR_STATE(idx).si_next_list = SYN_ITEMS(syn_block)[si_idx].sp_next_list;
  }
}

/// Call update_si_attr for item at index
void nvim_syn_update_si_attr(int idx)
{
  if (idx >= 0 && idx < current_state.ga_len) {
    rs_update_si_attr(idx);
  }
}

/// Compare two extmatch pointers (for syn_stack_equal)
/// Returns 1 if they match, 0 if different, -1 if needs string comparison
int nvim_syn_extmatch_equal(reg_extmatch_T *a, reg_extmatch_T *b)
{
  if (a == b) {
    return 1;
  }
  if (a == NULL || b == NULL) {
    return 0;
  }
  return -1;  // Need string comparison
}

/// Compare extmatch strings at given sub-index with ignore-case from pattern
/// Returns 1 if equal, 0 if different
int nvim_syn_extmatch_strings_equal(reg_extmatch_T *a, reg_extmatch_T *b,
                                     int subidx, int pat_idx)
{
  if (subidx < 0 || subidx >= NSUBEXP) {
    return 0;
  }
  if (a->matches[subidx] == b->matches[subidx]) {
    return 1;
  }
  if (a->matches[subidx] == NULL || b->matches[subidx] == NULL) {
    return 0;
  }

  int ic = 0;
  if (pat_idx >= 0 && syn_block != NULL && pat_idx < syn_block->b_syn_patterns.ga_len) {
    ic = SYN_ITEMS(syn_block)[pat_idx].sp_ic;
  }

  return mb_strcmp_ic(ic, (const char *)a->matches[subidx],
                      (const char *)b->matches[subidx]) == 0 ? 1 : 0;
}

/// Get NSUBEXP constant
int nvim_syn_get_nsubexp(void)
{
  return NSUBEXP;
}

/// Get the sp_ic (ignore case) flag for a pattern at index
int nvim_synblock_pattern_ic(int pat_idx)
{
  if (syn_block == NULL || pat_idx < 0 || pat_idx >= syn_block->b_syn_patterns.ga_len) {
    return 0;
  }
  return SYN_ITEMS(syn_block)[pat_idx].sp_ic;
}

/// Get si_extmatch from a stateitem
reg_extmatch_T *nvim_stateitem_get_extmatch(stateitem_T *item)
{
  if (item == NULL) {
    return NULL;
  }
  return item->si_extmatch;
}

// ============================================================================
// Phase 24.2: Core Pattern Matching Helpers (for Rust FFI)
// ============================================================================

/// Get si_m_endpos.lnum from stateitem
int nvim_stateitem_get_m_endpos_lnum(stateitem_T *item)
{
  return item ? (int)item->si_m_endpos.lnum : 0;
}

/// Get si_m_endpos.col from stateitem
int nvim_stateitem_get_m_endpos_col(stateitem_T *item)
{
  return item ? (int)item->si_m_endpos.col : 0;
}

/// Get si_h_startpos.lnum from stateitem
int nvim_stateitem_get_h_startpos_lnum(stateitem_T *item)
{
  return item ? (int)item->si_h_startpos.lnum : 0;
}

/// Get si_h_startpos.col from stateitem
int nvim_stateitem_get_h_startpos_col(stateitem_T *item)
{
  return item ? (int)item->si_h_startpos.col : 0;
}

/// Get si_h_endpos.lnum from stateitem
int nvim_stateitem_get_h_endpos_lnum(stateitem_T *item)
{
  return item ? (int)item->si_h_endpos.lnum : 0;
}

/// Get si_h_endpos.col from stateitem
int nvim_stateitem_get_h_endpos_col(stateitem_T *item)
{
  return item ? (int)item->si_h_endpos.col : 0;
}

/// Get si_eoe_pos.lnum from stateitem
int nvim_stateitem_get_eoe_pos_lnum(stateitem_T *item)
{
  return item ? (int)item->si_eoe_pos.lnum : 0;
}

/// Get si_eoe_pos.col from stateitem
int nvim_stateitem_get_eoe_pos_col(stateitem_T *item)
{
  return item ? (int)item->si_eoe_pos.col : 0;
}

/// Set si_m_endpos
void nvim_stateitem_set_m_endpos(stateitem_T *item, int lnum, int col)
{
  if (item) {
    item->si_m_endpos.lnum = (linenr_T)lnum;
    item->si_m_endpos.col = (colnr_T)col;
  }
}

/// Set si_h_endpos
void nvim_stateitem_set_h_endpos(stateitem_T *item, int lnum, int col)
{
  if (item) {
    item->si_h_endpos.lnum = (linenr_T)lnum;
    item->si_h_endpos.col = (colnr_T)col;
  }
}

/// Set si_eoe_pos
void nvim_stateitem_set_eoe_pos(stateitem_T *item, int lnum, int col)
{
  if (item) {
    item->si_eoe_pos.lnum = (linenr_T)lnum;
    item->si_eoe_pos.col = (colnr_T)col;
  }
}

/// Set si_idx
void nvim_stateitem_set_idx(stateitem_T *item, int idx)
{
  if (item) {
    item->si_idx = idx;
  }
}

/// Set si_end_idx
void nvim_stateitem_set_end_idx(stateitem_T *item, int end_idx)
{
  if (item) {
    item->si_end_idx = end_idx;
  }
}

/// Set si_flags
void nvim_stateitem_set_flags(stateitem_T *item, int flags)
{
  if (item) {
    item->si_flags = flags;
  }
}

/// Add flags to si_flags
void nvim_stateitem_add_flags(stateitem_T *item, int flags)
{
  if (item) {
    item->si_flags |= flags;
  }
}

/// Set si_seqnr
void nvim_stateitem_set_seqnr(stateitem_T *item, int seqnr)
{
  if (item) {
    item->si_seqnr = seqnr;
  }
}

/// Set si_ends
void nvim_stateitem_set_ends(stateitem_T *item, int ends)
{
  if (item) {
    item->si_ends = ends ? 1 : 0;
  }
}

/// Set si_id
void nvim_stateitem_set_id(stateitem_T *item, int id)
{
  if (item) {
    item->si_id = id;
  }
}

/// Set si_trans_id
void nvim_stateitem_set_trans_id(stateitem_T *item, int trans_id)
{
  if (item) {
    item->si_trans_id = trans_id;
  }
}

/// Set si_attr
void nvim_stateitem_set_attr(stateitem_T *item, int attr)
{
  if (item) {
    item->si_attr = attr;
  }
}

/// Set si_cont_list
void nvim_stateitem_set_cont_list(stateitem_T *item, int16_t *list)
{
  if (item) {
    item->si_cont_list = list;
  }
}

/// Get next_match_idx
int nvim_syn_get_next_match_idx_value(void)
{
  return next_match_idx;
}

/// Set next_match_idx
void nvim_syn_set_next_match_idx(int idx)
{
  next_match_idx = idx;
}

/// Set next_match_col
void nvim_syn_set_next_match_col(int col)
{
  next_match_col = col;
}

/// Set current_next_list
void nvim_syn_set_current_next_list_ptr(int16_t *list)
{
  current_next_list = list;
}

/// Get current_next_list
int16_t *nvim_syn_get_current_next_list_ptr(void)
{
  return current_next_list;
}

/// Call check_state_ends
void nvim_syn_check_state_ends(void)
{
  rs_check_state_ends();
}

/// Call update_si_attr
void nvim_syn_call_update_si_attr(int idx)
{
  rs_update_si_attr(idx);
}

/// Call pop_current_state
void nvim_syn_pop_current_state(void)
{
  if (!GA_EMPTY(&current_state)) {
    unref_extmatch(CUR_STATE(current_state.ga_len - 1).si_extmatch);
    current_state.ga_len--;
  }
  // after the end of a pattern, try matching a keyword or pattern
  next_match_idx = -1;

  // if first state with "keepend" is popped, reset keepend_level
  if (keepend_level >= current_state.ga_len) {
    keepend_level = -1;
  }
}

/// Call push_current_state
void nvim_syn_push_current_state(int idx)
{
  stateitem_T *p = GA_APPEND_VIA_PTR(stateitem_T, &current_state);
  CLEAR_POINTER(p);
  p->si_idx = idx;
}

/// Get the current line at the current column
char nvim_syn_getcurline_at_col(void)
{
  return syn_getcurline()[current_col];
}

/// Check if current_state is empty
int nvim_syn_current_state_is_empty(void)
{
  return GA_EMPTY(&current_state) ? 1 : 0;
}

/// Set current_finished
void nvim_syn_set_current_finished(int finished)
{
  current_finished = finished ? true : false;
}

/// Get synpat_T sp_type for pattern at index
int nvim_synblock_pattern_type(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return 0;
  }
  return SYN_ITEMS(syn_block)[idx].sp_type;
}

/// Get synpat_T sp_flags for pattern at index
int nvim_synblock_pattern_flags(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return 0;
  }
  return SYN_ITEMS(syn_block)[idx].sp_flags;
}

/// Get synpat_T sp_syn.id for pattern at index
int nvim_synblock_pattern_syn_id(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return 0;
  }
  return SYN_ITEMS(syn_block)[idx].sp_syn.id;
}

/// Get synpat_T sp_syn_match_id for pattern at index
int nvim_synblock_pattern_match_id(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return 0;
  }
  return SYN_ITEMS(syn_block)[idx].sp_syn_match_id;
}

/// Get synpat_T sp_cont_list for pattern at index
int16_t *nvim_synblock_pattern_cont_list(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return NULL;
  }
  return SYN_ITEMS(syn_block)[idx].sp_cont_list;
}

/// Get synpat_T sp_next_list for pattern at index
int16_t *nvim_synblock_pattern_next_list(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return NULL;
  }
  return SYN_ITEMS(syn_block)[idx].sp_next_list;
}

/// Call syn_id2attr (from highlight_group.c)
int nvim_syn_id2attr_wrapper(int syn_id)
{
  return syn_id2attr(syn_id);
}

/// Call syn_update_ends
void nvim_syn_call_syn_update_ends(int syncing)
{
  syn_update_ends(syncing ? true : false);
}

/// Get si_next_list from stateitem
int16_t *nvim_stateitem_get_next_list(stateitem_T *item)
{
  return item ? item->si_next_list : NULL;
}

/// Set si_next_list for stateitem
void nvim_stateitem_set_next_list(stateitem_T *item, int16_t *list)
{
  if (item) {
    item->si_next_list = list;
  }
}

/// Check if the ID_LIST_ALL constant matches a pointer
int nvim_syn_is_id_list_all(int16_t *list)
{
  return list == ID_LIST_ALL ? 1 : 0;
}

/// Get the ID_LIST_ALL pointer
int16_t *nvim_syn_get_id_list_all(void)
{
  return ID_LIST_ALL;
}

// ============================================================================
// Phase 24.3: Keyword Matching Helpers (for Rust FFI)
// ============================================================================

/// Call check_keyword_id from Rust
/// Returns the syntax ID if matched, 0 otherwise
int nvim_syn_check_keyword_id(char *line, int startcol, int *endcolp,
                               int *flagsp, int16_t **next_listp,
                               stateitem_T *cur_si, int *ccharp)
{
  // Find first character after the keyword.  First character was already
  // checked.
  char *const kwp = line + startcol;
  int kwlen = 0;
  do {
    kwlen += utfc_ptr2len(kwp + kwlen);
  } while (vim_iswordp_buf(kwp + kwlen, syn_buf));

  if (kwlen > MAXKEYWLEN) {
    return 0;
  }

  // Must make a copy of the keyword, so we can add a NUL and make it
  // lowercase.
  char keyword[MAXKEYWLEN + 1];         // assume max. keyword len is 80
  xmemcpyz(keyword, kwp, (size_t)kwlen);

  keyentry_T *kp = NULL;

  // matching case
  if (syn_block->b_keywtab.ht_used != 0) {
    kp = nvim_syn_match_keyword(keyword, 0, cur_si);
  }

  // ignoring case
  if (kp == NULL && syn_block->b_keywtab_ic.ht_used != 0) {
    str_foldcase(kwp, kwlen, keyword, MAXKEYWLEN + 1);
    kp = nvim_syn_match_keyword(keyword, 1, cur_si);
  }

  if (kp != NULL) {
    *endcolp = startcol + kwlen;
    *flagsp = kp->flags;
    *next_listp = kp->next_list;
    *ccharp = kp->k_char;
    return kp->k_syn.id;
  }

  return 0;
}

/// Call in_id_list from Rust
/// Returns 1 if in list, 0 otherwise
int nvim_syn_in_id_list(stateitem_T *cur_si, int16_t *list, int id, int inc_tag,
                         int16_t *cont_in_list, int flags)
{
  struct sp_syn ssp;
  ssp.id = (int16_t)id;
  ssp.inc_tag = inc_tag;
  ssp.cont_in_list = cont_in_list;
  return in_id_list(cur_si, list, &ssp, flags);
}

/// Check if there are keywords (case sensitive) in synblock
int nvim_syn_has_keywords(void)
{
  return syn_block != NULL && syn_block->b_keywtab.ht_used > 0 ? 1 : 0;
}

/// Check if there are keywords (case insensitive) in synblock
int nvim_syn_has_keywords_ic(void)
{
  return syn_block != NULL && syn_block->b_keywtab_ic.ht_used > 0 ? 1 : 0;
}

/// Get the current line from syn_getcurline
char *nvim_syn_getcurline(void)
{
  return syn_getcurline();
}

/// Call save_chartab for syntax iskeyword
void nvim_syn_save_chartab(char *buf)
{
  save_chartab(buf);
}

/// Call restore_chartab
void nvim_syn_restore_chartab(char *buf)
{
  restore_chartab(buf);
}

/// Get MAXKEYWLEN constant
int nvim_syn_get_maxkeywlen(void)
{
  return MAXKEYWLEN;
}

/// Call hash_find for keyword hashtab
keyentry_T *nvim_syn_keyword_find(char *keyword, int use_ic)
{
  if (syn_block == NULL) {
    return NULL;
  }
  hashtab_T *ht = use_ic ? &syn_block->b_keywtab_ic : &syn_block->b_keywtab;
  if (ht->ht_used == 0) {
    return NULL;
  }
  hashitem_T *hi = hash_find(ht, keyword);
  if (HASHITEM_EMPTY(hi)) {
    return NULL;
  }
  return HI2KE(hi);
}

/// Call match_keyword from Rust
keyentry_T *nvim_syn_match_keyword(char *keyword, int use_ic, stateitem_T *cur_si)
{
  if (syn_block == NULL) {
    return NULL;
  }
  hashtab_T *ht = use_ic ? &syn_block->b_keywtab_ic : &syn_block->b_keywtab;
  hashitem_T *hi = hash_find(ht, keyword);
  if (!HASHITEM_EMPTY(hi)) {
    for (keyentry_T *kp = HI2KE(hi); kp != NULL; kp = kp->ke_next) {
      if (current_next_list != 0
          ? in_id_list(NULL, current_next_list, &kp->k_syn, 0)
          : (cur_si == NULL
             ? !(kp->flags & HL_CONTAINED)
             : in_id_list(cur_si, cur_si->si_cont_list,
                          &kp->k_syn, kp->flags))) {
        return kp;
      }
    }
  }
  return NULL;
}

/// Copy and fold case for keyword
void nvim_syn_keyword_foldcase(char *src, int srclen, char *dst, int dstlen)
{
  str_foldcase(src, srclen, dst, dstlen);
}

/// Get utfc_ptr2len for keyword length calculation
int nvim_syn_utfc_ptr2len(char *p)
{
  return utfc_ptr2len(p);
}

/// Get syn_buf pointer (for keyword char checks)
void *nvim_syn_get_buf(void)
{
  return syn_buf;
}

/// Set the syn_buf pointer (for Rust FFI)
void nvim_syn_set_syn_buf(void *buf)
{
  syn_buf = (buf_T *)buf;
}

// ============================================================================
// Phase 24.4: Pattern Stack Operations Helpers (for Rust interop)
// Note: push/pop_current_state, get/set_next_match_idx, get/set_keepend_level
// are already defined earlier in Phase 24.2
// ============================================================================

/// Get the current_state garray length
int nvim_syn_current_state_len(void)
{
  return current_state.ga_len;
}

/// Get a stateitem from current_state by index
stateitem_T *nvim_syn_get_stateitem(int index)
{
  if (index < 0 || index >= current_state.ga_len) {
    return NULL;
  }
  return &CUR_STATE(index);
}

/// Get top stateitem from current_state
stateitem_T *nvim_syn_get_top_stateitem(void)
{
  if (current_state.ga_len == 0) {
    return NULL;
  }
  return &CUR_STATE(current_state.ga_len - 1);
}

/// Get next_seqnr global
int nvim_syn_get_next_seqnr(void)
{
  return next_seqnr;
}

/// Set next_seqnr global
void nvim_syn_set_next_seqnr(int seqnr)
{
  next_seqnr = seqnr;
}

/// Increment and get next_seqnr global
int nvim_syn_incr_next_seqnr(void)
{
  return next_seqnr++;
}

/// Get next_match_h_startpos
void nvim_syn_get_next_match_h_startpos(int *lnum, int *col)
{
  if (lnum) {
    *lnum = next_match_h_startpos.lnum;
  }
  if (col) {
    *col = next_match_h_startpos.col;
  }
}

/// Get next_match_m_endpos
void nvim_syn_get_next_match_m_endpos(int *lnum, int *col)
{
  if (lnum) {
    *lnum = next_match_m_endpos.lnum;
  }
  if (col) {
    *col = next_match_m_endpos.col;
  }
}

/// Get next_match_h_endpos
void nvim_syn_get_next_match_h_endpos(int *lnum, int *col)
{
  if (lnum) {
    *lnum = next_match_h_endpos.lnum;
  }
  if (col) {
    *col = next_match_h_endpos.col;
  }
}

/// Get next_match_eos_pos
void nvim_syn_get_next_match_eos_pos(int *lnum, int *col)
{
  if (lnum) {
    *lnum = next_match_eos_pos.lnum;
  }
  if (col) {
    *col = next_match_eos_pos.col;
  }
}

/// Get next_match_eoe_pos
void nvim_syn_get_next_match_eoe_pos(int *lnum, int *col)
{
  if (lnum) {
    *lnum = next_match_eoe_pos.lnum;
  }
  if (col) {
    *col = next_match_eoe_pos.col;
  }
}

/// Get next_match_flags
int nvim_syn_get_next_match_flags(void)
{
  return next_match_flags;
}

/// Get next_match_end_idx
int nvim_syn_get_next_match_end_idx(void)
{
  return next_match_end_idx;
}

/// Get next_match_extmatch
reg_extmatch_T *nvim_syn_get_next_match_extmatch(void)
{
  return next_match_extmatch;
}

/// Call ref_extmatch
reg_extmatch_T *nvim_syn_ref_extmatch(reg_extmatch_T *em)
{
  return ref_extmatch(em);
}

/// Call unref_extmatch
void nvim_syn_unref_extmatch(reg_extmatch_T *em)
{
  unref_extmatch(em);
}

/// Call push_next_match from Rust
stateitem_T *nvim_syn_push_next_match(void)
{
  return rs_push_next_match();
}



/// Get synpat sp_flags by index
int nvim_syn_get_pattern_flags(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return 0;
  }
  return SYN_ITEMS(syn_block)[idx].sp_flags;
}

/// Get synpat sp_cchar by index
int nvim_syn_get_pattern_cchar(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return 0;
  }
  return SYN_ITEMS(syn_block)[idx].sp_cchar;
}

/// Get synpat sp_next_list by index
int16_t *nvim_syn_get_pattern_next_list(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return NULL;
  }
  return SYN_ITEMS(syn_block)[idx].sp_next_list;
}

/// Get synpat sp_type by index
int nvim_syn_get_pattern_type(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return 0;
  }
  return SYN_ITEMS(syn_block)[idx].sp_type;
}

/// Get synpat sp_syn_match_id by index
int nvim_syn_get_pattern_syn_match_id(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return 0;
  }
  return SYN_ITEMS(syn_block)[idx].sp_syn_match_id;
}

/// Get GA_EMPTY(&current_state) check (Phase 24.4 - new name to avoid conflict)
int nvim_syn_is_current_state_empty(void)
{
  return GA_EMPTY(&current_state) ? 1 : 0;
}

/// Set si_h_startpos
void nvim_stateitem_set_h_startpos(stateitem_T *item, int lnum, int col)
{
  if (item) {
    item->si_h_startpos.lnum = lnum;
    item->si_h_startpos.col = col;
  }
}

/// Set si_m_startcol
void nvim_stateitem_set_m_startcol(stateitem_T *item, int col)
{
  if (item) {
    item->si_m_startcol = col;
  }
}

/// Set si_m_lnum
void nvim_stateitem_set_m_lnum(stateitem_T *item, int lnum)
{
  if (item) {
    item->si_m_lnum = lnum;
  }
}

/// Or si_flags with a value
void nvim_stateitem_or_flags(stateitem_T *item, int flags)
{
  if (item) {
    item->si_flags |= flags;
  }
}

/// Set si_cchar
void nvim_stateitem_set_cchar(stateitem_T *item, int cchar)
{
  if (item) {
    item->si_cchar = cchar;
  }
}

/// Set si_extmatch
void nvim_stateitem_set_extmatch(stateitem_T *item, reg_extmatch_T *em)
{
  if (item) {
    item->si_extmatch = em;
  }
}

/// Get SPTYPE_START constant
int nvim_syn_get_sptype_start(void)
{
  return SPTYPE_START;
}

/// Get HL_ONELINE constant
int nvim_syn_get_hl_oneline(void)
{
  return HL_ONELINE;
}

/// Get HL_KEEPEND constant
int nvim_syn_get_hl_keepend(void)
{
  return HL_KEEPEND;
}

/// Get HL_MATCH constant
int nvim_syn_get_hl_match(void)
{
  return HL_MATCH;
}

/// Get HL_CONCEAL constant
int nvim_syn_get_hl_conceal(void)
{
  return HL_CONCEAL;
}

/// Get HL_CONCEALENDS constant
int nvim_syn_get_hl_concealends(void)
{
  return HL_CONCEALENDS;
}

// =============================================================================
// Phase 24.5: Sync and Line Operations Helpers
// =============================================================================

/// Call syn_start_line from Rust
void nvim_syn_start_line(void)
{
  syn_start_line();
}

/// Call syn_finish_line from Rust
int nvim_syn_finish_line(int syncing)
{
  return rs_syn_finish_line(syncing);
}

/// Call syn_update_ends from Rust
void nvim_syn_update_ends(int startofline)
{
  syn_update_ends(startofline != 0);
}

/// Get current_line_id global
int nvim_syn_get_current_line_id(void)
{
  return (int)current_line_id;
}

/// Increment current_line_id global
void nvim_syn_incr_current_line_id(void)
{
  current_line_id++;
}

/// Get syn_block pointer
void *nvim_syn_get_syn_block(void)
{
  return syn_block;
}

/// Set syn_block pointer
void nvim_syn_set_syn_block(void *block)
{
  syn_block = (synblock_T *)block;
}

/// Get syn_win pointer
void *nvim_syn_get_syn_win(void)
{
  return syn_win;
}

/// Set syn_win pointer
void nvim_syn_set_syn_win(void *win)
{
  syn_win = (win_T *)win;
}

/// Get b_syn_sync_minlines from syn_block
int nvim_syn_get_sync_minlines(void)
{
  return syn_block ? (int)syn_block->b_syn_sync_minlines : 0;
}

/// Get b_syn_sync_maxlines from syn_block
int nvim_syn_get_sync_maxlines(void)
{
  return syn_block ? (int)syn_block->b_syn_sync_maxlines : 0;
}

/// Get b_syn_sync_flags from syn_block
int nvim_syn_get_sync_flags(void)
{
  return syn_block ? syn_block->b_syn_sync_flags : 0;
}

/// Get b_syn_sync_id from syn_block
int nvim_syn_get_sync_id(void)
{
  return syn_block ? syn_block->b_syn_sync_id : 0;
}

/// Get b_sst_first from syn_block (first in valid state list)
void *nvim_syn_get_sst_first(void)
{
  return syn_block ? syn_block->b_sst_first : NULL;
}

/// Get b_sst_array from syn_block
void *nvim_syn_get_sst_array(void)
{
  return syn_block ? syn_block->b_sst_array : NULL;
}

/// Get b_sst_len from syn_block
int nvim_syn_get_sst_len(void)
{
  return syn_block ? syn_block->b_sst_len : 0;
}

/// Set synstate sst_change_lnum
void nvim_synstate_set_change_lnum(synstate_T *p, int lnum)
{
  if (p) {
    p->sst_change_lnum = (linenr_T)lnum;
  }
}

/// Set current_id global
void nvim_syn_set_current_id(int id)
{
  current_id = (int16_t)id;
}

/// Set current_trans_id global
void nvim_syn_set_current_trans_id(int id)
{
  current_trans_id = (int16_t)id;
}

/// Set current_flags global
void nvim_syn_set_current_flags(int flags)
{
  current_flags = (int16_t)flags;
}

/// Set current_seqnr global
void nvim_syn_set_current_seqnr(int seqnr)
{
  current_seqnr = seqnr;
}

/// Get HL_MATCHCONT constant
int nvim_syn_get_hl_matchcont(void)
{
  return HL_MATCHCONT;
}

/// Get HL_EXTEND constant
int nvim_syn_get_hl_extend(void)
{
  return HL_EXTEND;
}

/// Get SF_CCOMMENT constant
int nvim_syn_get_sf_ccomment(void)
{
  return SF_CCOMMENT;
}

/// Get SF_MATCH constant
int nvim_syn_get_sf_match(void)
{
  return SF_MATCH;
}

/// Get HL_SYNC_HERE constant
int nvim_syn_get_hl_sync_here(void)
{
  return HL_SYNC_HERE;
}

/// Get HL_SYNC_THERE constant
int nvim_syn_get_hl_sync_there(void)
{
  return HL_SYNC_THERE;
}

/// Get SPTYPE_MATCH constant
int nvim_syn_get_sptype_match(void)
{
  return SPTYPE_MATCH;
}

/// Call syn_stack_alloc from Rust
void nvim_syn_stack_alloc(void)
{
  syn_stack_alloc();
}

/// Call syn_stack_find_entry from Rust (void* return for FFI)
void *nvim_syn_stack_find_entry_ptr(int lnum)
{
  return syn_stack_find_entry((linenr_T)lnum);
}

/// Get w_s from window (synblock)
void *nvim_win_get_synblock(void *wp)
{
  return wp ? ((win_T *)wp)->w_s : NULL;
}

/// Get w_buffer from window (void* return for FFI)
void *nvim_syn_win_get_buffer_ptr(void *wp)
{
  return wp ? ((win_T *)wp)->w_buffer : NULL;
}

/// Get w_p_fdn (foldnestmax) from window
int nvim_win_get_foldnestmax(void *wp)
{
  return wp ? (int)((win_T *)wp)->w_p_fdn : 0;
}

/// Get ml_line_count from buffer (void* input for FFI)
int nvim_syn_buf_get_line_count(void *buf)
{
  return buf ? (int)((buf_T *)buf)->b_ml.ml_line_count : 0;
}

/// Call buf_get_changedtick from Rust (void* input for FFI)
int nvim_syn_buf_get_changed_tick(void *buf)
{
  return buf ? (int)buf_get_changedtick((buf_T *)buf) : 0;
}

/// Set b_sst_lasttick in syn_block
void nvim_syn_set_sst_lasttick(int tick)
{
  if (syn_block) {
    syn_block->b_sst_lasttick = (disptick_T)tick;
  }
}

/// Get display_tick global
int nvim_syn_get_display_tick(void)
{
  return (int)display_tick;
}

/// Call line_breakcheck from Rust
void nvim_syn_line_breakcheck(void)
{
  line_breakcheck();
}

/// Get got_int global
int nvim_syn_get_got_int(void)
{
  return got_int;
}

/// Get Rows global
int nvim_syn_get_rows(void)
{
  return (int)Rows;
}

// =============================================================================
// Rust-callable wrappers for state stack management
// =============================================================================

/// Wrapper for syn_stack_free_all - free all syntax state entries for a synblock
void nvim_syn_stack_free_all(synblock_T *block)
{
  syn_stack_free_all(block);
}

/// Wrapper for syn_stack_apply_changes - apply buffer changes to syntax states
void nvim_syn_stack_apply_changes(buf_T *buf)
{
  syn_stack_apply_changes(buf);
}

/// Get b_mod_top from buffer (line where change starts)
int nvim_buf_get_mod_top(buf_T *buf)
{
  return (int)buf->b_mod_top;
}

/// Get b_mod_bot from buffer (line after change)
int nvim_buf_get_mod_bot(buf_T *buf)
{
  return (int)buf->b_mod_bot;
}

/// Get b_mod_xlines from buffer (number of extra lines)
int nvim_buf_get_mod_xlines(buf_T *buf)
{
  return (int)buf->b_mod_xlines;
}

/// Get b_syn_sync_linebreaks from synblock
int nvim_synblock_get_linebreaks(synblock_T *block)
{
  return block->b_syn_sync_linebreaks;
}

/// Set sst_lnum on a synstate
void nvim_synstate_set_lnum(synstate_T *state, int lnum)
{
  state->sst_lnum = lnum;
}

/// Get sst_next_list pointer equality check
int nvim_synstate_next_list_eq(synstate_T *a, synstate_T *b)
{
  return a->sst_next_list == b->sst_next_list;
}

// =============================================================================
// Rust-callable wrappers for cluster operations (Phase 32.3)
// =============================================================================

/// Forward declaration for syn_scl_name2id
static int syn_scl_name2id(char *name);

/// Lookup a cluster by name and return its ID.
/// Returns 0 if not found.
int nvim_syn_cluster_name2id(const char *name)
{
  return syn_scl_name2id((char *)name);
}

/// Check if the synblock has containedin items.
int nvim_synblock_has_containedin(synblock_T *block)
{
  return block->b_syn_containedin ? 1 : 0;
}

/// Get the pattern count for synblock.
int nvim_synblock_pattern_count(synblock_T *block)
{
  return block->b_syn_patterns.ga_len;
}

/// Get the inc_tag from a pattern.
int nvim_synpat_get_inc_tag(synpat_T *pat)
{
  return pat ? pat->sp_syn.inc_tag : 0;
}

/// Check if this is a spell/nospell cluster.
int nvim_synblock_is_spell_cluster(synblock_T *block, int id)
{
  return id == block->b_spell_cluster_id;
}

int nvim_synblock_is_nospell_cluster(synblock_T *block, int id)
{
  return id == block->b_nospell_cluster_id;
}

// =============================================================================
// Rust-callable wrappers for line highlighting (Phase 32.4)
// =============================================================================



// nvim_syn_get_current_lnum already defined at line ~6119
// nvim_syn_get_current_col already defined at line ~6125

/// Set current_col
void nvim_syn_set_current_col(int col)
{
  current_col = col;
}

/// Get current_finished
int nvim_syn_get_current_finished(void)
{
  return current_finished;
}

/// Get current_state_stored
int nvim_syn_get_current_state_stored(void)
{
  return current_state_stored;
}

// nvim_synblock_get_syn_spell already defined at line ~5735

/// Get synmaxcol setting from buffer
int nvim_buf_get_synmaxcol(buf_T *buf)
{
  return (int)buf->b_p_smc;
}

/// Check if current state is valid
int nvim_syn_current_state_valid(void)
{
  return !INVALID_STATE(&current_state);
}

/// Validate current state if needed
void nvim_syn_ensure_current_state_valid(void)
{
  if (INVALID_STATE(&current_state)) {
    validate_current_state();
  }
}

/// Get the current line text
const char *nvim_syn_get_current_line(void)
{
  return syn_getcurline();
}

/// Get the attribute for the next match
int nvim_syn_get_next_match_attr(void)
{
  // Get the attribute from current state
  if (current_id > 0) {
    return syn_id2attr(current_id);
  }
  return 0;
}

// nvim_syn_get_next_match_idx already defined at line ~6413
// nvim_syn_get_next_match_col already defined at line ~6419

// =============================================================================
// Rust-callable wrappers for buffer attachment (Phase 32.5)
// =============================================================================

/// Get the current syn_block
synblock_T *nvim_syn_get_block(void)
{
  return syn_block;
}

/// Get the current syn_win
win_T *nvim_syn_get_win(void)
{
  return syn_win;
}

/// Get the current fold level from syntax state
int nvim_syn_cur_foldlevel(void)
{
  return nvim_syn_count_fold_items();
}

// =============================================================================
// Rust-callable wrappers for Ex commands (Phase 32.6)
// =============================================================================

/// Get the command line pointer
char **nvim_syn_get_cmdlinep(void)
{
  return syn_cmdlinep;
}

/// Get current window's synblock
synblock_T *nvim_get_curwin_synblock(void)
{
  return curwin->w_s;
}

// nvim_get_curwin already defined in window.c

/// Get include_link flag
int nvim_syn_get_include_link(void)
{
  return include_link;
}

/// Get include_default flag
int nvim_syn_get_include_default(void)
{
  return include_default;
}

/// Get include_none flag
int nvim_syn_get_include_none(void)
{
  return include_none;
}

/// Get running_syn_inc_tag
int nvim_syn_get_running_inc_tag(void)
{
  return running_syn_inc_tag;
}

/// Set running_syn_inc_tag
void nvim_syn_set_running_inc_tag(int tag)
{
  running_syn_inc_tag = tag;
}

/// Get the conceal setting for a synblock
int nvim_syn_get_conceal_setting(synblock_T *block)
{
  return block->b_syn_conceal;
}

/// Get the case-insensitive setting for a synblock
int nvim_syn_get_ic_setting(synblock_T *block)
{
  return block->b_syn_ic;
}

// =============================================================================
// Phase 143: Syntax State Machine Migration - C Accessors
// =============================================================================

/// Wrapper for get_syntax_info - get extra info about syntax item
int nvim_get_syntax_info(int *seqnrp)
{
  return get_syntax_info(seqnrp);
}

/// Wrapper for syntax_end_parsing
void nvim_syntax_end_parsing(win_T *wp, int lnum)
{
  syntax_end_parsing(wp, (linenr_T)lnum);
}

/// Set tick on synstate
void nvim_synstate_set_tick(synstate_T *state, int tick)
{
  if (state) {
    state->sst_tick = tick;
  }
}

/// Get tick from synstate
int nvim_synstate_get_tick_val(synstate_T *state)
{
  return state ? state->sst_tick : 0;
}

// Note: The following accessor functions are already defined earlier in the file:
// nvim_stateitem_get_m_lnum, nvim_stateitem_get_m_startcol,
// nvim_stateitem_set_m_lnum, nvim_stateitem_set_m_startcol,
// nvim_stateitem_set_cchar, nvim_stateitem_set_h_startpos,
// nvim_stateitem_get_cchar, nvim_stateitem_get_end_idx, nvim_stateitem_get_ends

// =============================================================================
// Phase 1: Accessor/wrapper functions for migrated helpers
// =============================================================================

/// Wrapper around syn_regexec for calling from Rust.
/// Executes regex match on a pattern in the current synblock by index.
/// Returns 1 if matched, 0 if not. Fills out-params with match positions.
int nvim_syn_regexec_pat(int idx, int lnum, int col,
                         int *start_lnum, int *start_col,
                         int *end_lnum, int *end_col)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return 0;
  }
  synpat_T *spp = &SYN_ITEMS(syn_block)[idx];
  regmmatch_T regmatch;
  regmatch.rmm_ic = spp->sp_ic;
  regmatch.regprog = spp->sp_prog;
  bool r = syn_regexec(&regmatch, (linenr_T)lnum, (colnr_T)col,
                        IF_SYN_TIME(&spp->sp_time));
  spp->sp_prog = regmatch.regprog;
  if (r) {
    if (start_lnum) { *start_lnum = regmatch.startpos[0].lnum; }
    if (start_col) { *start_col = regmatch.startpos[0].col; }
    if (end_lnum) { *end_lnum = regmatch.endpos[0].lnum; }
    if (end_col) { *end_col = regmatch.endpos[0].col; }
    return 1;
  }
  return 0;
}

/// Get a synpat offset value by pattern index and offset index.
int nvim_syn_get_pattern_offset(int pat_idx, int off_idx)
{
  if (syn_block == NULL || pat_idx < 0 || pat_idx >= syn_block->b_syn_patterns.ga_len
      || off_idx < 0 || off_idx >= SPO_COUNT) {
    return 0;
  }
  return SYN_ITEMS(syn_block)[pat_idx].sp_offsets[off_idx];
}

/// Get a synpat off_flags by pattern index.
int nvim_syn_get_pattern_off_flags(int pat_idx)
{
  if (syn_block == NULL || pat_idx < 0 || pat_idx >= syn_block->b_syn_patterns.ga_len) {
    return 0;
  }
  return SYN_ITEMS(syn_block)[pat_idx].sp_off_flags;
}

/// Get the sp_syn.id by pattern index.
int nvim_syn_get_pattern_syn_id(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return 0;
  }
  return SYN_ITEMS(syn_block)[idx].sp_syn.id;
}

/// Get the sp_cont_list by pattern index.
int16_t *nvim_syn_get_pattern_cont_list(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return NULL;
  }
  return SYN_ITEMS(syn_block)[idx].sp_cont_list;
}

/// Get pattern count in the current synblock.
int nvim_syn_get_synblock_pattern_count(void)
{
  if (syn_block == NULL) {
    return 0;
  }
  return syn_block->b_syn_patterns.ga_len;
}

/// Get current line length (syn_getcurline_len).
int nvim_syn_getcurline_len(void)
{
  return (int)ml_get_buf_len(syn_buf, current_lnum);
}

/// Get line length for arbitrary lnum in syn_buf.
int nvim_syn_get_line_len(int lnum)
{
  return (int)ml_get_buf_len(syn_buf, (linenr_T)lnum);
}

/// Get syn_buf line count.
int nvim_syn_get_buf_line_count(void)
{
  return (int)syn_buf->b_ml.ml_line_count;
}

/// Apply multibyte character offset to a column position.
/// Given a line number and starting column, advance (off > 0) or retreat
/// (off < 0) by `off` characters, respecting multibyte boundaries.
/// Returns the new column.
int nvim_syn_mb_adjust_col(int lnum, int col, int off)
{
  if (off == 0) {
    return col;
  }
  char *base = ml_get_buf(syn_buf, (linenr_T)lnum);
  char *p = base + col;
  if (off > 0) {
    while (off-- > 0 && *p != NUL) {
      MB_PTR_ADV(p);
    }
  } else {
    while (off++ < 0 && base < p) {
      MB_PTR_BACK(base, p);
    }
  }
  return (int)(p - base);
}

/// Same as nvim_syn_mb_adjust_col but don't go past NUL (for start offsets).
int nvim_syn_mb_adjust_col_start(int lnum, int col, int off)
{
  if (off == 0) {
    return col;
  }
  char *base = ml_get_buf(syn_buf, (linenr_T)lnum);
  char *p = base + col;
  if (off > 0) {
    while (off-- && *p != NUL) {
      MB_PTR_ADV(p);
    }
  } else {
    while (off++ && base < p) {
      MB_PTR_BACK(base, p);
    }
  }
  return (int)(p - base);
}

/// Get/set re_extmatch_in for find_endpos.
void nvim_syn_set_extmatch_in(reg_extmatch_T *em)
{
  unref_extmatch(re_extmatch_in);
  re_extmatch_in = ref_extmatch(em);
}

/// Clear re_extmatch_in.
void nvim_syn_clear_extmatch_in(void)
{
  unref_extmatch(re_extmatch_in);
  re_extmatch_in = NULL;
}

/// Set the next_match_col static.
void nvim_syn_set_next_match_col_val(int col)
{
  next_match_col = col;
}

/// Get the syn_getcurline() result at a specific column.
int nvim_syn_getcurline_byte_at(int col)
{
  return (unsigned char)syn_getcurline()[col];
}

// =============================================================================
// Phase 2 accessor functions for syn_current_attr migration
// =============================================================================

/// Set current_attr
void nvim_syn_set_current_attr(int attr)
{
  current_attr = attr;
}

/// Set current_sub_char
void nvim_syn_set_current_sub_char(int c)
{
  current_sub_char = c;
}

/// Get re_extmatch_out, take ownership (sets it to NULL in C)
reg_extmatch_T *nvim_syn_take_re_extmatch_out(void)
{
  reg_extmatch_T *em = re_extmatch_out;
  re_extmatch_out = NULL;
  return em;
}

/// Unref and clear re_extmatch_out
void nvim_syn_clear_re_extmatch_out(void)
{
  unref_extmatch(re_extmatch_out);
  re_extmatch_out = NULL;
}

/// Get pattern sp_line_id
int nvim_syn_get_pattern_line_id(int idx)
{
  return SYN_ITEMS(syn_block)[idx].sp_line_id;
}

/// Set pattern sp_line_id
void nvim_syn_set_pattern_line_id(int idx, int line_id)
{
  SYN_ITEMS(syn_block)[idx].sp_line_id = line_id;
}

/// Get pattern sp_startcol
int nvim_syn_get_pattern_startcol(int idx)
{
  return SYN_ITEMS(syn_block)[idx].sp_startcol;
}

/// Set pattern sp_startcol
void nvim_syn_set_pattern_startcol(int idx, int col)
{
  SYN_ITEMS(syn_block)[idx].sp_startcol = col;
}

/// Get pattern sp_offsets[SPO_LC_OFF]
int nvim_syn_get_pattern_lc_off(int idx)
{
  return SYN_ITEMS(syn_block)[idx].sp_offsets[SPO_LC_OFF];
}

/// Get pattern sp_syncing
int nvim_syn_get_pattern_syncing(int idx)
{
  return SYN_ITEMS(syn_block)[idx].sp_syncing;
}

/// Check if pattern has HL_DISPLAY flag
int nvim_syn_get_pattern_display(int idx)
{
  return (SYN_ITEMS(syn_block)[idx].sp_flags & HL_DISPLAY) != 0;
}

/// Get synblock pattern count (b_syn_patterns.ga_len)
int nvim_syn_get_pattern_ga_len(void)
{
  return syn_block->b_syn_patterns.ga_len;
}

/// Get syn_block b_syn_containedin
int nvim_syn_has_containedin(void)
{
  return syn_block->b_syn_containedin;
}

/// Execute syn_regexec for a pattern by index, return match result.
/// Returns 1 if matched, 0 if not. Writes start/end positions to out-params.
/// Also updates sp_prog if needed.
int nvim_syn_regexec_by_idx(int idx, int lnum, int col,
                            int *s_lnum, int *s_col,
                            int *e_lnum, int *e_col)
{
  synpat_T *spp = &(SYN_ITEMS(syn_block)[idx]);
  regmmatch_T regmatch;
  regmatch.rmm_ic = spp->sp_ic;
  regmatch.regprog = spp->sp_prog;
  int r = syn_regexec(&regmatch, lnum, col,
                      IF_SYN_TIME(&spp->sp_time));
  spp->sp_prog = regmatch.regprog;
  if (r) {
    *s_lnum = regmatch.startpos[0].lnum;
    *s_col = regmatch.startpos[0].col;
    *e_lnum = regmatch.endpos[0].lnum;
    *e_col = regmatch.endpos[0].col;
  }
  return r;
}

/// Check in_id_list for a pattern by index against the current_next_list or
/// cur_si cont_list.
/// mode: 0 = check current_next_list (cur_si ignored)
///       1 = check cur_si->si_cont_list
///       2 = check HL_CONTAINED flag
int nvim_syn_check_pattern_containment(int pat_idx, int si_idx, int has_next_list, int has_cur_si)
{
  synpat_T *spp = &(SYN_ITEMS(syn_block)[pat_idx]);
  if (has_next_list) {
    return in_id_list(NULL, current_next_list, &spp->sp_syn, 0);
  } else if (!has_cur_si) {
    return !(spp->sp_flags & HL_CONTAINED);
  } else {
    stateitem_T *cur_si = &CUR_STATE(si_idx);
    return in_id_list(cur_si, cur_si->si_cont_list, &spp->sp_syn, spp->sp_flags);
  }
}

/// Check in_id_list with a specific sp_syn (for spell checking)
int nvim_syn_in_id_list_spell(stateitem_T *sip, int16_t *list, int id)
{
  struct sp_syn sps;
  sps.inc_tag = 0;
  sps.id = (int16_t)id;
  sps.cont_in_list = NULL;
  return in_id_list(sip, list, &sps, 0);
}

/// Get spell-related synblock fields
int nvim_syn_get_spell_cluster_id(void)
{
  return syn_block->b_spell_cluster_id;
}

int nvim_syn_get_nospell_cluster_id(void)
{
  return syn_block->b_nospell_cluster_id;
}

int nvim_syn_get_syn_spell(void)
{
  return syn_block->b_syn_spell;
}

/// Vim iswordp check for syn_buf
int nvim_syn_vim_iswordp_buf(char *p)
{
  return vim_iswordp_buf(p, syn_buf);
}

/// utf_head_off wrapper
int nvim_syn_utf_head_off(char *base, char *p)
{
  return utf_head_off(base, p);
}

/// ascii_iswhite wrapper
int nvim_syn_ascii_iswhite(int c)
{
  return ascii_iswhite(c);
}

/// Set next_match positions (bulk setter for all next_match_* variables)
void nvim_syn_set_next_match_state(
    int idx, int col,
    int m_endpos_lnum, int m_endpos_col,
    int h_endpos_lnum, int h_endpos_col,
    int h_startpos_lnum, int h_startpos_col,
    int flags,
    int eos_pos_lnum, int eos_pos_col,
    int eoe_pos_lnum, int eoe_pos_col,
    int end_idx,
    reg_extmatch_T *extmatch)
{
  next_match_idx = idx;
  next_match_col = col;
  next_match_m_endpos.lnum = m_endpos_lnum;
  next_match_m_endpos.col = m_endpos_col;
  next_match_h_endpos.lnum = h_endpos_lnum;
  next_match_h_endpos.col = h_endpos_col;
  next_match_h_startpos.lnum = h_startpos_lnum;
  next_match_h_startpos.col = h_startpos_col;
  next_match_flags = flags;
  next_match_eos_pos.lnum = eos_pos_lnum;
  next_match_eos_pos.col = eos_pos_col;
  next_match_eoe_pos.lnum = eoe_pos_lnum;
  next_match_eoe_pos.col = eoe_pos_col;
  next_match_end_idx = end_idx;
  unref_extmatch(next_match_extmatch);
  next_match_extmatch = extmatch;
}

/// _Static_assert for Phase 1 constants
_Static_assert(SPO_MS_OFF == 0, "SPO_MS_OFF");
_Static_assert(SPO_ME_OFF == 1, "SPO_ME_OFF");
_Static_assert(SPO_HS_OFF == 2, "SPO_HS_OFF");
_Static_assert(SPO_HE_OFF == 3, "SPO_HE_OFF");
_Static_assert(SPO_RS_OFF == 4, "SPO_RS_OFF");
_Static_assert(SPO_RE_OFF == 5, "SPO_RE_OFF");
_Static_assert(SPO_LC_OFF == 6, "SPO_LC_OFF");
_Static_assert(SPO_COUNT == 7, "SPO_COUNT");
_Static_assert(SPTYPE_MATCH == 1, "SPTYPE_MATCH");
_Static_assert(SPTYPE_START == 2, "SPTYPE_START");
_Static_assert(SPTYPE_END == 3, "SPTYPE_END");
_Static_assert(SPTYPE_SKIP == 4, "SPTYPE_SKIP");
_Static_assert(HL_HAS_EOL == 0x08, "HL_HAS_EOL");
_Static_assert(HL_ONELINE == 0x04, "HL_ONELINE");
_Static_assert(HL_KEEPEND == 0x400, "HL_KEEPEND");
_Static_assert(HL_EXTEND == 0x4000, "HL_EXTEND");
_Static_assert(HL_MATCHCONT == 0x8000, "HL_MATCHCONT");
_Static_assert(HL_CONCEALENDS == 0x40000, "HL_CONCEALENDS");
_Static_assert(HL_MATCH == 0x40, "HL_MATCH");
_Static_assert(HL_CONCEAL == 0x20000, "HL_CONCEAL");
_Static_assert(HL_TRANSP == 0x02, "HL_TRANSP");
_Static_assert(HL_TRANS_CONT == 0x10000, "HL_TRANS_CONT");
_Static_assert(HL_CONTAINED == 0x01, "HL_CONTAINED");
_Static_assert(HL_SKIPNL == 0x80, "HL_SKIPNL");
_Static_assert(HL_SKIPEMPTY == 0x200, "HL_SKIPEMPTY");

// =============================================================================
// Phase 3 accessor functions for syn_sync migration
// =============================================================================

/// Wrap load_current_state() for Rust.
void nvim_syn_load_current_state(synstate_T *from)
{
  rs_load_current_state(from);
}

/// Wrap syn_match_linecont() for Rust.
int nvim_syn_match_linecont(linenr_T lnum)
{
  return syn_match_linecont(lnum);
}

/// Get sp_sync_idx for the current synblock pattern at index idx.
int nvim_syn_get_pattern_sync_idx(int idx)
{
  if (idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return -2;  // NONE_IDX
  }
  return SYN_ITEMS(syn_block)[idx].sp_sync_idx;
}

/// Get the line contents (ml_get) for a given lnum using syn_buf.
char *nvim_syn_ml_get(linenr_T lnum)
{
  return ml_get_buf(syn_buf, lnum);
}

/// Handle the entire C-comment sync setup path.
/// This saves/restores curwin, curbuf, cursor and does the find_start_comment
/// logic. Returns the adjusted start_lnum.
linenr_T nvim_syn_ccomment_sync_setup(win_T *wp, linenr_T start_lnum)
{
  // Save curwin/curbuf and set to syn_buf/wp
  win_T *curwin_save = curwin;
  curwin = wp;
  buf_T *curbuf_save = curbuf;
  curbuf = syn_buf;

  // Skip lines that end in a backslash.
  for (; start_lnum > 1; start_lnum--) {
    char *l = ml_get(start_lnum - 1);
    if (*l == NUL || *(l + ml_get_len(start_lnum - 1) - 1) != '\\') {
      break;
    }
  }
  current_lnum = start_lnum;

  // set cursor to start of search
  pos_T cursor_save = wp->w_cursor;
  wp->w_cursor.lnum = start_lnum;
  wp->w_cursor.col = 0;

  // If the line is inside a comment, need to find the syntax item that
  // defines the comment.
  if (find_start_comment((int)syn_block->b_syn_sync_maxlines) != NULL) {
    for (int idx = syn_block->b_syn_patterns.ga_len; --idx >= 0;) {
      if (SYN_ITEMS(syn_block)[idx].sp_syn.id
          == syn_block->b_syn_sync_id
          && SYN_ITEMS(syn_block)[idx].sp_type == SPTYPE_START) {
        validate_current_state();
        nvim_syn_push_current_state(idx);
        rs_update_si_attr(current_state.ga_len - 1);
        break;
      }
    }
  }

  // restore cursor and buffer
  wp->w_cursor = cursor_save;
  curwin = curwin_save;
  curbuf = curbuf_save;

  return start_lnum;
}

/// _Static_assert for Phase 2 constants
_Static_assert(HL_DISPLAY == 0x1000, "HL_DISPLAY");
_Static_assert(HL_SKIPWHITE == 0x100, "HL_SKIPWHITE");
_Static_assert(HL_SYNC_HERE == 0x10, "HL_SYNC_HERE");
_Static_assert(HL_SYNC_THERE == 0x20, "HL_SYNC_THERE");
_Static_assert(SYNSPL_DEFAULT == 0, "SYNSPL_DEFAULT");
_Static_assert(SYNSPL_TOP == 1, "SYNSPL_TOP");
_Static_assert(SYNSPL_NOTOP == 2, "SYNSPL_NOTOP");
_Static_assert(KEYWORD_IDX == -1, "KEYWORD_IDX");

// =============================================================================
// Phase 4 accessor functions for get_syn_options + get_id_list migration
// =============================================================================

/// Get current_syn_inc_tag global.
int nvim_syn_get_current_inc_tag(void)
{
  return current_syn_inc_tag;
}

/// Get b_syn_conceal from curwin.
int nvim_syn_get_b_syn_conceal(void)
{
  return curwin->w_s->b_syn_conceal;
}

/// Wrap syn_check_cluster() for Rust.
int nvim_syn_check_cluster(char *pp, int len)
{
  return syn_check_cluster(pp, len);
}

/// Wrap syn_name2id() for Rust.
int nvim_syn_name2id_wrapper(const char *name)
{
  return syn_name2id(name);
}

/// Wrap syn_check_group() for Rust.
int nvim_syn_check_group_wrapper(const char *name, int len)
{
  return syn_check_group(name, (size_t)len);
}

/// Wrap highlight_num_groups() for Rust.
int nvim_syn_highlight_num_groups(void)
{
  return highlight_num_groups();
}

/// Wrap highlight_group_name() for Rust.
char *nvim_syn_highlight_group_name(int idx)
{
  return highlight_group_name(idx);
}

/// Compile a regexp for group name matching.
/// Returns opaque regprog handle, or NULL on failure.
void *nvim_syn_vim_regcomp(char *pat, int flags)
{
  return vim_regcomp(pat, flags);
}

/// Execute a regexp match against a string.
/// regprog is from nvim_syn_vim_regcomp, ic is ignore case.
int nvim_syn_vim_regexec(void *regprog, int ic, char *str)
{
  regmatch_T regmatch;
  regmatch.regprog = regprog;
  regmatch.rm_ic = ic;
  int ret = vim_regexec(&regmatch, str, 0);
  // Don't free regprog here - caller manages it
  return ret;
}

/// Free a compiled regexp.
void nvim_syn_vim_regfree(void *regprog)
{
  vim_regfree(regprog);
}

/// Wrap foldmethodIsSyntax(curwin) for Rust.
int nvim_syn_foldmethod_is_syntax_curwin(void)
{
  return foldmethodIsSyntax(curwin);
}

/// Wrap rs_foldUpdateAll(curwin) for Rust.
void nvim_syn_fold_update_all_curwin(void)
{
  rs_foldUpdateAll(curwin);
}

/// Find the pattern index matching sync_id + SPTYPE_START in curwin's patterns.
/// Returns the index, or -1 if not found.
int nvim_syn_find_sync_pattern_idx(int syn_id)
{
  for (int i = curwin->w_s->b_syn_patterns.ga_len; --i >= 0;) {
    if (SYN_ITEMS(curwin->w_s)[i].sp_syn.id == syn_id
        && SYN_ITEMS(curwin->w_s)[i].sp_type == SPTYPE_START) {
      return i;
    }
  }
  return -1;
}

/// Wrap utf_ptr2char() for Rust.
int nvim_syn_utf_ptr2char(const char *p)
{
  return utf_ptr2char(p);
}

// nvim_syn_utfc_ptr2len already defined above (line ~6694) with char * param

/// Wrap vim_isprintc() for Rust.
int nvim_syn_vim_isprintc(int c)
{
  return vim_isprintc(c);
}

/// Wrap xstrnsave() for Rust. Caller must call nvim_syn_xfree() to free.
char *nvim_syn_xstrnsave(const char *s, int len)
{
  return xstrnsave(s, (size_t)len);
}

/// Wrap xfree() for Rust.
void nvim_syn_xfree(void *ptr)
{
  xfree(ptr);
}

/// Wrap xmalloc() for Rust.
void *nvim_syn_xmalloc(int size)
{
  return xmalloc((size_t)size);
}

/// Wrap xmemcpyz() for Rust.
void nvim_syn_xmemcpyz(char *dst, const char *src, int len)
{
  xmemcpyz(dst, src, (size_t)len);
}

/// Wrap strpbrk() for Rust.
char *nvim_syn_strpbrk(const char *s, const char *chars)
{
  return strpbrk(s, chars);
}

/// Wrap emsg() for Rust.
void nvim_syn_emsg(const char *msg)
{
  emsg(msg);
}

/// Wrap semsg() for Rust (one string arg).
void nvim_syn_semsg_1s(const char *fmt, const char *arg)
{
  semsg(fmt, arg);
}

/// Wrap skipwhite() for Rust.
char *nvim_syn_skipwhite(const char *p)
{
  return skipwhite(p);
}

/// Wrap skiptowhite() for Rust.
char *nvim_syn_skiptowhite(const char *p)
{
  return skiptowhite(p);
}

/// Wrap ends_excmd() for Rust.
int nvim_syn_ends_excmd(int c)
{
  return ends_excmd(c);
}

/// Wrap ascii_iswhite() for Rust.
int nvim_syn_ascii_iswhite_char(int c)
{
  return ascii_iswhite(c);
}

/// Wrap TOUPPER_ASC() for Rust.
int nvim_syn_toupper_asc(int c)
{
  return TOUPPER_ASC(c);
}

// =============================================================================
// Phase 5 accessor functions for syn_cmd_region + get_syn_pattern migration
// =============================================================================

/// Wrap get_group_name() for Rust.
char *nvim_syn_get_group_name(char *arg, char **name_end)
{
  return get_group_name(arg, name_end);
}

/// Wrap init_syn_patterns() for Rust.
void nvim_syn_init_patterns(void)
{
  init_syn_patterns();
}

/// Wrap vim_strnsave_up() for Rust.
char *nvim_syn_vim_strnsave_up(const char *str, int len)
{
  return vim_strnsave_up(str, (size_t)len);
}

/// Wrap check_nextcmd() for Rust, and set eap->nextcmd.
void nvim_syn_set_nextcmd(exarg_T *eap, char *rest)
{
  eap->nextcmd = check_nextcmd(rest);
}

/// Get eap->arg.
char *nvim_syn_get_eap_arg(const exarg_T *eap)
{
  return eap->arg;
}

/// Get eap->skip.
int nvim_syn_get_eap_skip(const exarg_T *eap)
{
  return eap->skip;
}

/// Allocate and compile a syntax pattern via get_syn_pattern().
/// Returns an opaque handle to a heap-allocated synpat_T (or NULL on error).
/// The caller must free with nvim_syn_free_compiled_pattern() on error.
/// item_type: 0=START, 1=SKIP, 2=END
/// current_flags: current syn_opt_arg.flags at the time of pattern compilation
synpat_T *nvim_syn_compile_pattern(char *arg, int item_type, int opt_flags, char **rest_out)
{
  // Enable the appropriate \z specials.
  if (item_type == ITEM_START) {
    reg_do_extmatch = REX_SET;
  } else {
    reg_do_extmatch = REX_USE;
  }

  synpat_T *pat = xcalloc(1, sizeof(synpat_T));
  char *rest = get_syn_pattern(arg, pat);
  reg_do_extmatch = 0;

  if (rest == NULL) {
    xfree(pat);
    *rest_out = NULL;
    return NULL;
  }

  // Check for HL_HAS_EOL on end patterns (only if HL_EXCLUDENL not set)
  if (item_type == ITEM_END && vim_regcomp_had_eol()
      && !(opt_flags & HL_EXCLUDENL)) {
    pat->sp_flags |= HL_HAS_EOL;
  }

  *rest_out = rest;
  return pat;
}

/// Free a compiled pattern on error.
void nvim_syn_free_compiled_pattern(synpat_T *pat)
{
  if (pat != NULL) {
    vim_regfree(pat->sp_prog);
    xfree(pat->sp_pattern);
    xfree(pat);
  }
}

/// Wrap syn_incl_toplevel() for Rust.
void nvim_syn_incl_toplevel(int id, int *flagsp)
{
  syn_incl_toplevel(id, flagsp);
}

/// Store completed region patterns into the synblock.
/// pat_data is an array of {synpat_T *pat, int matchgroup_id, int item_type} triples.
/// pat_count is the number of entries.
/// flags, syn_id, conceal_char are from option parsing.
/// cont_list, cont_in_list, next_list are from option parsing (ownership transferred on success).
/// Returns 1 on success (don't free patterns), 0 on failure.
int nvim_syn_store_region_patterns(
  synpat_T **pats,
  int *matchgroup_ids,
  int *item_types,
  int pat_count,
  int flags,
  int syn_id,
  int conceal_char,
  int16_t *cont_list,
  int16_t *cont_in_list,
  int16_t *next_list,
  int syncing)
{
  ga_grow(&(curwin->w_s->b_syn_patterns), pat_count);

  int idx = curwin->w_s->b_syn_patterns.ga_len;
  for (int i = 0; i < pat_count; i++) {
    SYN_ITEMS(curwin->w_s)[idx] = *(pats[i]);
    SYN_ITEMS(curwin->w_s)[idx].sp_syncing = syncing;
    SYN_ITEMS(curwin->w_s)[idx].sp_type =
      (item_types[i] == ITEM_START) ? SPTYPE_START
                                    : (item_types[i] == ITEM_SKIP) ? SPTYPE_SKIP : SPTYPE_END;
    SYN_ITEMS(curwin->w_s)[idx].sp_flags |= flags;
    SYN_ITEMS(curwin->w_s)[idx].sp_syn.id = (int16_t)syn_id;
    SYN_ITEMS(curwin->w_s)[idx].sp_syn.inc_tag = current_syn_inc_tag;
    SYN_ITEMS(curwin->w_s)[idx].sp_syn_match_id = (int16_t)matchgroup_ids[i];
    SYN_ITEMS(curwin->w_s)[idx].sp_cchar = conceal_char;
    if (item_types[i] == ITEM_START) {
      SYN_ITEMS(curwin->w_s)[idx].sp_cont_list = cont_list;
      SYN_ITEMS(curwin->w_s)[idx].sp_syn.cont_in_list = cont_in_list;
      if (cont_in_list != NULL) {
        curwin->w_s->b_syn_containedin = true;
      }
      SYN_ITEMS(curwin->w_s)[idx].sp_next_list = next_list;
    }
    curwin->w_s->b_syn_patterns.ga_len++;
    idx++;
    if (flags & HL_FOLD) {
      curwin->w_s->b_syn_folditems++;
    }
  }

  redraw_curbuf_later(UPD_SOME_VALID);
  syn_stack_free_all(curwin->w_s);
  return 1;
}

/// _Static_assert for Phase 3 constants
_Static_assert(SF_CCOMMENT == 0x01, "SF_CCOMMENT");
_Static_assert(SF_MATCH == 0x02, "SF_MATCH");

/// _Static_assert for Phase 4 constants
_Static_assert(NONE_IDX == -2, "NONE_IDX");
_Static_assert(SYNID_ALLBUT == MAX_HL_ID, "SYNID_ALLBUT");
_Static_assert(SYNID_TOP == 21000, "SYNID_TOP");
_Static_assert(SYNID_CONTAINED == 22000, "SYNID_CONTAINED");
_Static_assert(HL_FOLD == 0x2000, "HL_FOLD");
_Static_assert(HL_EXCLUDENL == 0x800, "HL_EXCLUDENL");

/// _Static_assert for Phase 5 constants
_Static_assert(HL_HAS_EOL == 0x08, "HL_HAS_EOL");
_Static_assert(HL_INCLUDED_TOPLEVEL == 0x80000, "HL_INCLUDED_TOPLEVEL");
_Static_assert(SPTYPE_START == 2, "SPTYPE_START");
_Static_assert(SPTYPE_SKIP == 4, "SPTYPE_SKIP");
_Static_assert(SPTYPE_END == 3, "SPTYPE_END");
