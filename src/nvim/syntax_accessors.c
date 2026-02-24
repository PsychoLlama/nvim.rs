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

// Pass 5: clearing.rs Rust functions
extern void rs_syn_clear_pattern(synblock_T *block, int i);
extern void rs_syn_clear_cluster(synblock_T *block, int i);
extern void rs_syn_remove_pattern(synblock_T *block, int idx);
extern void rs_syn_clear_one(int id, int syncing);
extern void rs_syntax_clear(synblock_T *block);
extern void rs_reset_synblock(win_T *wp);
extern void rs_syntax_sync_clear(void);
extern void rs_syn_clear_keyword(int id, hashtab_T *ht);
extern void rs_clear_keywtab(hashtab_T *ht);
extern void rs_invalidate_current_state(void);

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

// Rust pattern parse FFI declaration
extern char *rs_get_syn_pattern(char *arg, synpat_T *ci);

// Rust match command FFI declaration
extern void rs_syn_cmd_match(exarg_T *eap, int syncing);

// Rust keyword command FFI declaration
extern void rs_syn_cmd_keyword(exarg_T *eap, int syncing);

// Rust cluster command FFI declaration
extern void rs_syn_cmd_cluster(exarg_T *eap, int syncing);

// Rust sync command FFI declaration
extern void rs_syn_cmd_sync(exarg_T *eap, int syncing);

// Rust clear command FFI declaration
extern void rs_syn_cmd_clear(exarg_T *eap, int syncing);

// Rust include command FFI declaration
extern void rs_syn_cmd_include(exarg_T *eap, int syncing);

// Rust simple subcommand FFI declarations (Phase 4)
extern void rs_syn_cmd_reset(exarg_T *eap, int syncing);
extern void rs_syn_cmd_onoff(exarg_T *eap, const char *name, int syncing);
extern void rs_syn_maybe_enable(void);

// Rust dispatch wrappers (Phase 1 of pass 4): replace C thin wrappers
extern void rs_syn_cmd_case_dispatch(exarg_T *eap, int syncing);
extern void rs_syn_cmd_conceal_dispatch(exarg_T *eap, int syncing);
extern void rs_syn_cmd_foldlevel_dispatch(exarg_T *eap, int syncing);
extern void rs_syn_cmd_spell_dispatch(exarg_T *eap, int syncing);
extern void rs_syn_cmd_on_dispatch(exarg_T *eap, int syncing);
extern void rs_syn_cmd_manual_dispatch(exarg_T *eap, int syncing);
extern void rs_syn_cmd_off_dispatch(exarg_T *eap, int syncing);

// Rust Phase 4 (pass 4): iskeyword + ownsyntax
extern void rs_syn_cmd_iskeyword(exarg_T *eap, int syncing);
extern void rs_ex_ownsyntax(exarg_T *eap);

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
    if (wp->w_s == block && rs_foldmethodIsSyntax(wp)) {
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
  rs_invalidate_current_state();
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

/// Handle ":syntax iskeyword" command.
static void syn_cmd_iskeyword(exarg_T *eap, int syncing)
{
  rs_syn_cmd_iskeyword(eap, syncing);
}

// Clear all syntax info for one buffer.
void syntax_clear(synblock_T *block)
{
  rs_syntax_clear(block);
}

// Get rid of ownsyntax for window "wp".
void reset_synblock(win_T *wp)
{
  rs_reset_synblock(wp);
}

// Clear syncing info for one buffer.
static void syntax_sync_clear(void)
{
  rs_syntax_sync_clear();
}

// Remove one pattern from the buffer's pattern list.
static void syn_remove_pattern(synblock_T *block, int idx)
{
  rs_syn_remove_pattern(block, idx);
}

// Clear and free one syntax pattern.  When clearing all, must be called from
// last to first!
static void syn_clear_pattern(synblock_T *block, int i)
{
  rs_syn_clear_pattern(block, i);
}

// Clear and free one syntax cluster.
static void syn_clear_cluster(synblock_T *block, int i)
{
  rs_syn_clear_cluster(block, i);
}

// Clear one syntax group for the current buffer.
static void syn_clear_one(const int id, const bool syncing)
{
  rs_syn_clear_one(id, (int)syncing);
}

void syn_maybe_enable(void)
{
  rs_syn_maybe_enable();
}

// syn_cmd_list and helper listing functions are implemented in Rust (listing.rs).
// The C stub syn_cmd_list is at the bottom of this file.
// Listing functions (syn_cmd_list, syn_list_one, etc.) are implemented in Rust (listing.rs).

static void syn_clear_keyword(int id, hashtab_T *ht)
{
  rs_syn_clear_keyword(id, ht);
}

// Clear a whole keyword table.
static void clear_keywtab(hashtab_T *ht)
{
  rs_clear_keywtab(ht);
}

// Phase 6: Forward declarations for Rust functions used below
extern void rs_add_keyword(char *name, int namelen, int id, int flags,
                            int16_t *cont_in_list, int16_t *next_list, int conceal_char);
extern int16_t *rs_copy_id_list(const int16_t *list);
extern char *rs_get_group_name(char *arg, char **name_end);
extern void rs_syn_incl_toplevel(int id, int *flagsp);
extern void rs_init_syn_patterns(void);
extern int rs_syn_scl_name2id(char *name);
extern int rs_syn_scl_namen2id(char *linep, int len);
extern int rs_syn_check_cluster(char *pp, int len);
extern int rs_syn_add_cluster(char *name);

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
  rs_add_keyword(name, (int)namelen, id, flags, cont_in_list, next_list, conceal_char);
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
  return rs_get_group_name(arg, name_end);
}


// Adjustments to syntax item when declared in a ":syn include"'d file.
// Set the contained flag, and if the item is not already contained, add it
// to the specified top-level group, if any.
static void syn_incl_toplevel(int id, int *flagsp)
{
  rs_syn_incl_toplevel(id, flagsp);
}





// Keep ITEM_* defines available for C wrappers
#define ITEM_START          0
#define ITEM_SKIP           1
#define ITEM_END            2
#define ITEM_MATCHGROUP     3

// Combines lists of syntax clusters.
// *clstr1 and *clstr2 must both be allocated memory; they will be consumed.
// Implemented in Rust: see rs_syn_combine_list in nvim-syntax/src/cluster.rs
extern void rs_syn_combine_list(int16_t **clstr1, int16_t **clstr2, int list_op);
static void syn_combine_list(int16_t **const clstr1, int16_t **const clstr2, const int list_op)
{
  rs_syn_combine_list(clstr1, clstr2, list_op);
}

/// Lookup a syntax cluster name and return its ID.
/// If it is not found, 0 is returned.
static int syn_scl_name2id(char *name)
{
  return rs_syn_scl_name2id(name);
}

/// Like syn_scl_name2id(), but take a pointer + length argument.
static int syn_scl_namen2id(char *linep, int len)
{
  return rs_syn_scl_namen2id(linep, len);
}

/// Find syntax cluster name in the table and return its ID.
/// The argument is a pointer to the name and the length of the name.
/// If it doesn't exist yet, a new entry is created.
///
/// @return  0 for failure.
static int syn_check_cluster(char *pp, int len)
{
  return rs_syn_check_cluster(pp, len);
}

/// Add new syntax cluster and return its ID.
/// "name" must be an allocated string, it will be consumed.
///
/// @return  0 for failure.
static int syn_add_cluster(char *name)
{
  return rs_syn_add_cluster(name);
}

// On first call for current buffer: Init growing array.
static void init_syn_patterns(void)
{
  rs_init_syn_patterns();
}

/// Get one pattern for a ":syntax match" or ":syntax region" command.
/// Stores the pattern and program in a synpat_T.
///
/// @return  a pointer to the next argument, or NULL in case of an error.
static char *get_syn_pattern(char *arg, synpat_T *ci)
{
  return rs_get_syn_pattern(arg, ci);
}



// Make a copy of an ID list.
static int16_t *copy_id_list(const int16_t *const list)
{
  return rs_copy_id_list(list);
}

/// in_id_list implemented in Rust.
/// See rs_syn_in_id_list in nvim-syntax/src/containment.rs
extern int rs_syn_in_id_list(stateitem_T *cur_si, int16_t *list, int id, int inc_tag,
                              int16_t *cont_in_list, int flags);

/// Check if syntax group "ssp" is in the ID list "list" of "cur_si".
/// Thin shim calling the Rust implementation.
static int in_id_list(stateitem_T *cur_si, int16_t *list, struct sp_syn *ssp, int flags)
{
  return rs_syn_in_id_list(cur_si, list, ssp->id, ssp->inc_tag, ssp->cont_in_list, flags);
}

/// Rust implementation of the `:syntax` dispatcher.
extern void rs_ex_syntax(exarg_T *eap);

/// ":syntax" -- thin wrapper delegating to Rust.
void ex_syntax(exarg_T *eap)
{
  rs_ex_syntax(eap);
}

/// @deprecated -- thin wrapper delegating to Rust.
void ex_ownsyntax(exarg_T *eap)
{
  rs_ex_ownsyntax(eap);
}

static enum {
  EXP_SUBCMD,       // expand ":syn" sub-commands
  EXP_CASE,         // expand ":syn case" arguments
  EXP_SPELL,        // expand ":syn spell" arguments
  EXP_SYNC,         // expand ":syn sync" arguments
  EXP_CLUSTER,      // expand ":syn list @cluster" arguments
} expand_what;

// set_context_in_syntax_cmd, get_syntax_name, set_context_in_echohl_cmd,
// and reset_expand_highlight are implemented in Rust (expand.rs).
// Their C thin wrappers are at the bottom of this file.

// Rust implementations of query API (Phase 3 of pass 4)
extern int rs_syn_get_id(win_T *wp, linenr_T lnum, colnr_T col, int trans, int *spellp,
                         int keep_state);
extern int rs_get_syntax_info(int *seqnrp);
extern int rs_syn_get_concealed_id(win_T *wp, linenr_T lnum, colnr_T col);
extern int rs_syn_get_stack_item(int i);

/// Function called for expression evaluation: get syntax ID at file position.
///
/// @param trans       remove transparency
/// @param spellp      return: can do spell checking
/// @param keep_state  keep state of char at "col"
int syn_get_id(win_T *wp, linenr_T lnum, colnr_T col, int trans, bool *spellp, int keep_state)
{
  int sp = 0;
  int id = rs_syn_get_id(wp, lnum, col, trans, spellp ? &sp : NULL, keep_state);
  if (spellp) {
    *spellp = sp != 0;
  }
  return id;
}

// Get extra information about the syntax item.  Must be called right after
// get_syntax_attr().
// Stores the current item sequence nr in "*seqnrp".
// Returns the current flags.
int get_syntax_info(int *seqnrp)
{
  return rs_get_syntax_info(seqnrp);
}

/// Get the sequence number of the concealed file position.
///
/// @return seqnr if the file position is concealed, 0 otherwise.
int syn_get_concealed_id(win_T *wp, linenr_T lnum, colnr_T col)
{
  return rs_syn_get_concealed_id(wp, lnum, col);
}

// C accessor for current_sub_char (used by Rust)
int nvim_get_current_sub_char(void) { return current_sub_char; }

// Return the syntax ID at position "i" in the current stack.
// The caller must have called syn_get_id() before to fill the stack.
// Returns -1 when "i" is out of range.
int syn_get_stack_item(int i)
{
  return rs_syn_get_stack_item(i);
}

// ":syntime" and "get_syntime_arg" are implemented in Rust (syntime.rs).
// Their C thin wrappers are at the bottom of this file.

// syn_clear_time is a file-local helper used by nvim_synpat_clear_time
// and syn_start_line's b_syn_linecont_time reset.
static void syn_clear_time(syn_time_T *st)
{
  st->total = profile_zero();
  st->slowest = profile_zero();
  st->count = 0;
  st->match = 0;
}

int nvim_win_get_syn_patterns_len(win_T *win) { return win->w_s->b_syn_patterns.ga_len; }
int nvim_win_get_syn_clusters_len(win_T *win) { return win->w_s->b_syn_clusters.ga_len; }
int nvim_win_get_keywtab_used(win_T *win) { return (int)win->w_s->b_keywtab.ht_used; }
int nvim_win_get_keywtab_ic_used(win_T *win) { return (int)win->w_s->b_keywtab_ic.ht_used; }

int nvim_synblock_get_pattern_count(synblock_T *block) { return block->b_syn_patterns.ga_len; }
int nvim_synblock_get_cluster_count(synblock_T *block) { return block->b_syn_clusters.ga_len; }
int nvim_synblock_get_syn_ic(synblock_T *block) { return block->b_syn_ic; }
void nvim_synblock_set_syn_ic(synblock_T *block, int ic) { block->b_syn_ic = ic; }
int nvim_synblock_get_syn_spell(synblock_T *block) { return block->b_syn_spell; }
void nvim_synblock_set_syn_spell(synblock_T *block, int spell) { block->b_syn_spell = spell; }
int nvim_synblock_get_syn_foldlevel(synblock_T *block) { return block->b_syn_foldlevel; }
void nvim_synblock_set_syn_foldlevel(synblock_T *block, int foldlevel) { block->b_syn_foldlevel = foldlevel; }
int nvim_synblock_get_containedin(synblock_T *block) { return block->b_syn_containedin; }
int nvim_synblock_get_sync_flags(synblock_T *block) { return block->b_syn_sync_flags; }
int16_t nvim_synblock_get_sync_id(synblock_T *block) { return block->b_syn_sync_id; }
int nvim_synblock_get_sync_minlines(synblock_T *block) { return (int)block->b_syn_sync_minlines; }
int nvim_synblock_get_sync_maxlines(synblock_T *block) { return (int)block->b_syn_sync_maxlines; }
int nvim_synblock_get_sync_linebreaks(synblock_T *block) { return (int)block->b_syn_sync_linebreaks; }
int nvim_synblock_get_topgrp(synblock_T *block) { return block->b_syn_topgrp; }
int nvim_synblock_get_conceal(synblock_T *block) { return block->b_syn_conceal; }
void nvim_synblock_set_conceal(synblock_T *block, int conceal) { block->b_syn_conceal = conceal; }
int nvim_synblock_get_folditems(synblock_T *block) { return block->b_syn_folditems; }
int nvim_synblock_get_sst_len(synblock_T *block) { return block->b_sst_len; }
int nvim_synblock_get_sst_freecount(synblock_T *block) { return block->b_sst_freecount; }
int nvim_synblock_get_sst_check_lnum(synblock_T *block) { return (int)block->b_sst_check_lnum; }
int nvim_synblock_get_syn_error(synblock_T *block) { return block->b_syn_error ? 1 : 0; }
int nvim_synblock_get_syn_slow(synblock_T *block) { return block->b_syn_slow ? 1 : 0; }
int nvim_synblock_get_spell_cluster_id(synblock_T *block) { return block->b_spell_cluster_id; }
int nvim_synblock_get_nospell_cluster_id(synblock_T *block) { return block->b_nospell_cluster_id; }
synstate_T *nvim_synblock_get_sst_first(synblock_T *block) { return block->b_sst_first; }
synstate_T *nvim_synblock_get_sst_firstfree(synblock_T *block) { return block->b_sst_firstfree; }
int nvim_synblock_has_sst_array(synblock_T *block) { return block->b_sst_array != NULL ? 1 : 0; }

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

synstate_T *nvim_synstate_get_next(synstate_T *state) { return state->sst_next; }
int nvim_synstate_get_lnum(synstate_T *state) { return (int)state->sst_lnum; }
int nvim_synstate_get_stacksize(synstate_T *state) { return state->sst_stacksize; }
int nvim_synstate_get_next_flags(synstate_T *state) { return state->sst_next_flags; }
int nvim_synstate_get_tick(synstate_T *state) { return (int)state->sst_tick; }
int nvim_synstate_get_change_lnum(synstate_T *state) { return (int)state->sst_change_lnum; }

int nvim_synpat_get_type(synpat_T *pat) { return (int)pat->sp_type; }
int nvim_synpat_get_syncing(synpat_T *pat) { return pat->sp_syncing ? 1 : 0; }
int16_t nvim_synpat_get_syn_match_id(synpat_T *pat) { return pat->sp_syn_match_id; }
int16_t nvim_synpat_get_off_flags(synpat_T *pat) { return pat->sp_off_flags; }
int nvim_synpat_get_flags(synpat_T *pat) { return pat->sp_flags; }
int nvim_synpat_get_cchar(synpat_T *pat) { return pat->sp_cchar; }
int nvim_synpat_get_ic(synpat_T *pat) { return pat->sp_ic; }
int nvim_synpat_get_sync_idx(synpat_T *pat) { return pat->sp_sync_idx; }
char *nvim_synpat_get_pattern(synpat_T *pat) { return pat->sp_pattern; }
int16_t nvim_synpat_get_syn_id(synpat_T *pat) { return pat->sp_syn.id; }
int nvim_synpat_get_syn_inc_tag(synpat_T *pat) { return pat->sp_syn.inc_tag; }

char *nvim_syncluster_get_name(syn_cluster_T *cluster) { return cluster->scl_name; }
char *nvim_syncluster_get_name_u(syn_cluster_T *cluster) { return cluster->scl_name_u; }

int nvim_stateitem_get_idx(stateitem_T *item) { return item->si_idx; }
int nvim_stateitem_get_id(stateitem_T *item) { return item->si_id; }
int nvim_stateitem_get_trans_id(stateitem_T *item) { return item->si_trans_id; }
int nvim_stateitem_get_m_lnum(stateitem_T *item) { return item->si_m_lnum; }
int nvim_stateitem_get_m_startcol(stateitem_T *item) { return item->si_m_startcol; }
int nvim_stateitem_get_attr(stateitem_T *item) { return item->si_attr; }
int nvim_stateitem_get_flags(stateitem_T *item) { return item->si_flags; }
int nvim_stateitem_get_seqnr(stateitem_T *item) { return item->si_seqnr; }
int nvim_stateitem_get_cchar(stateitem_T *item) { return item->si_cchar; }
int nvim_stateitem_get_end_idx(stateitem_T *item) { return item->si_end_idx; }
int nvim_stateitem_get_ends(stateitem_T *item) { return item->si_ends; }

keyentry_T *nvim_keyentry_get_next(keyentry_T *ke) { return ke->ke_next; }
int16_t nvim_keyentry_get_syn_id(keyentry_T *ke) { return ke->k_syn.id; }
int nvim_keyentry_get_syn_inc_tag(keyentry_T *ke) { return ke->k_syn.inc_tag; }
int nvim_keyentry_get_flags(keyentry_T *ke) { return ke->flags; }
int nvim_keyentry_get_char(keyentry_T *ke) { return ke->k_char; }
char *nvim_keyentry_get_keyword(keyentry_T *ke) { return ke->keyword; }

int nvim_syn_get_current_lnum(void) { return (int)current_lnum; }
int nvim_syn_get_current_col(void) { return (int)current_col; }
int nvim_syn_is_current_finished(void) { return current_finished ? 1 : 0; }
int nvim_syn_is_current_state_stored(void) { return current_state_stored ? 1 : 0; }
int nvim_syn_get_current_state_len(void) { return current_state.ga_len; }
int nvim_syn_is_current_state_valid(void) { return VALID_STATE(&current_state) ? 1 : 0; }
int nvim_syn_get_current_id(void) { return current_id; }
int nvim_syn_get_current_trans_id(void) { return current_trans_id; }
int nvim_syn_get_current_attr(void) { return current_attr; }
int nvim_syn_get_current_flags(void) { return current_flags; }
int nvim_syn_get_current_seqnr(void) { return current_seqnr; }
int nvim_syn_get_current_sub_char(void) { return current_sub_char; }
int nvim_syn_get_current_next_flags(void) { return current_next_flags; }
int nvim_syn_get_keepend_level(void) { return keepend_level; }

/// Get state item at index (NULL if out of bounds)
stateitem_T *nvim_syn_get_cur_state(int idx)
{
  if (idx < 0 || idx >= current_state.ga_len) {
    return NULL;
  }
  return &CUR_STATE(idx);
}

synblock_T *nvim_syn_get_synblock(void) { return syn_block; }

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

regprog_T *nvim_synpat_get_prog(synpat_T *pat) { return pat->sp_prog; }
int nvim_synpat_has_prog(synpat_T *pat) { return pat->sp_prog != NULL; }
int16_t *nvim_synpat_get_cont_list(synpat_T *pat) { return pat->sp_cont_list; }
int16_t *nvim_synpat_get_next_list(synpat_T *pat) { return pat->sp_next_list; }
int16_t *nvim_synpat_get_cont_in_list(synpat_T *pat) { return pat->sp_syn.cont_in_list; }
int nvim_synpat_has_cont_list(synpat_T *pat) { return pat->sp_cont_list != NULL; }
int nvim_synpat_has_next_list(synpat_T *pat) { return pat->sp_next_list != NULL; }
int nvim_synpat_has_cont_in_list(synpat_T *pat) { return pat->sp_syn.cont_in_list != NULL; }

hashtab_T *nvim_synblock_get_keywtab(synblock_T *block) { return &block->b_keywtab; }
hashtab_T *nvim_synblock_get_keywtab_ic(synblock_T *block) { return &block->b_keywtab_ic; }
int nvim_synblock_has_keywords(synblock_T *block) { return block->b_keywtab.ht_used > 0; }
int nvim_synblock_has_keywords_ic(synblock_T *block) { return block->b_keywtab_ic.ht_used > 0; }

size_t nvim_synblock_keywtab_count(synblock_T *block) { return block->b_keywtab.ht_used; }
size_t nvim_synblock_keywtab_ic_count(synblock_T *block) { return block->b_keywtab_ic.ht_used; }

int16_t *nvim_keyentry_get_next_list(keyentry_T *ke) { return ke->next_list; }
int16_t *nvim_keyentry_get_cont_in_list(keyentry_T *ke) { return ke->k_syn.cont_in_list; }
int nvim_keyentry_has_next_list(keyentry_T *ke) { return ke->next_list != NULL; }
int nvim_keyentry_has_cont_in_list(keyentry_T *ke) { return ke->k_syn.cont_in_list != NULL; }

int16_t *nvim_syncluster_get_list(syn_cluster_T *cluster) { return cluster->scl_list; }
int nvim_syncluster_has_list(syn_cluster_T *cluster) { return cluster->scl_list != NULL; }

/// Get the first item in an ID list (returns 0 if list is NULL)
int16_t nvim_id_list_first(int16_t *list)
{
  if (list == NULL) {
    return 0;
  }
  return *list;
}

int16_t nvim_id_list_get(int16_t *list, int idx) { return list[idx]; }

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

int nvim_syn_get_next_match_idx(void) { return next_match_idx; }
int nvim_syn_get_next_match_col(void) { return next_match_col; }
int nvim_syn_has_next_match(void) { return next_match_idx >= 0; }
int16_t *nvim_syn_get_current_next_list(void) { return current_next_list; }
int nvim_syn_has_current_next_list(void) { return current_next_list != NULL; }
synblock_T *nvim_syn_get_curwin_synblock(void) { return curwin->w_s; }

int nvim_synblock_get_spell_cluster(synblock_T *block) { return block->b_spell_cluster_id; }
int nvim_synblock_get_nospell_cluster(synblock_T *block) { return block->b_nospell_cluster_id; }

int nvim_stateitem_has_trans_cont(stateitem_T *item) { return (item->si_flags & HL_TRANS_CONT) != 0; }
int nvim_stateitem_has_match(stateitem_T *item) { return (item->si_flags & HL_MATCH) != 0; }
int16_t *nvim_stateitem_get_cont_list(stateitem_T *item) { return item->si_cont_list; }
int nvim_stateitem_has_cont_list(stateitem_T *item) { return item->si_cont_list != NULL; }

int nvim_syn_get_topgrp(void) { return curwin->w_s->b_syn_topgrp; }
void nvim_syn_set_topgrp(int topgrp) { curwin->w_s->b_syn_topgrp = topgrp; }

int nvim_synblock_get_conceal_setting(synblock_T *block) { return block->b_syn_conceal; }
int nvim_synblock_get_ic_setting(synblock_T *block) { return block->b_syn_ic; }

/// Subcommand names list (mirrors SUBCOMMANDS in Rust commands.rs).
/// Used by tab-completion (expand.rs) to iterate subcommand names.
static const char *const subcommand_names[] = {
  "case", "clear", "cluster", "conceal", "enable", "foldlevel",
  "include", "iskeyword", "keyword", "list", "manual", "match",
  "on", "off", "region", "reset", "spell", "sync", "",
};

int nvim_syn_get_subcommand_count(void)
{
  return (int)(sizeof(subcommand_names) / sizeof(subcommand_names[0]));
}

/// Get subcommand name by index
const char *nvim_syn_get_subcommand_name(int idx)
{
  int count = (int)(sizeof(subcommand_names) / sizeof(subcommand_names[0]));
  if (idx < 0 || idx >= count) {
    return NULL;
  }
  return subcommand_names[idx];
}

/// Check if a pattern at index is for syncing
int nvim_synblock_pattern_is_syncing(synblock_T *block, int idx)
{
  if (idx < 0 || idx >= block->b_syn_patterns.ga_len) {
    return 0;
  }
  return SYN_ITEMS(block)[idx].sp_syncing;
}

int nvim_synpat_get_hl_group(synpat_T *pat) { return pat->sp_syn.id - 1; }

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

int nvim_syn_get_expand_what(void) { return expand_what; }
void nvim_syn_set_expand_what(int what) { expand_what = what; }

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

synstate_T *nvim_syn_stack_find_entry(int lnum) { return syn_stack_find_entry((linenr_T)lnum); }

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

void nvim_syn_set_state_stored(int stored) { current_state_stored = stored ? true : false; }

/// Call clear_current_state()
void nvim_syn_clear_current_state(void)
{
#define UNREF_STATEITEM_EXTMATCH(si) unref_extmatch((si)->si_extmatch)
  GA_DEEP_CLEAR(&current_state, stateitem_T, UNREF_STATEITEM_EXTMATCH);
}

void nvim_syn_validate_current_state(void) { validate_current_state(); }

void nvim_syn_invalidate_current_state(void) { invalidate_current_state(); }

void nvim_syn_set_keepend_level(int level) { keepend_level = level; }

void nvim_syn_grow_current_state(int size) { ga_grow(&current_state, size); }

void nvim_syn_set_current_state_len(int len) { current_state.ga_len = len; }
void nvim_syn_set_current_next_list(int16_t *list) { current_next_list = list; }
void nvim_syn_set_current_next_flags(int flags) { current_next_flags = flags; }
void nvim_syn_set_current_lnum(int lnum) { current_lnum = (linenr_T)lnum; }

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

int nvim_bufstate_get_idx(bufstate_T *bs) { return bs ? bs->bs_idx : 0; }
int nvim_bufstate_get_flags(bufstate_T *bs) { return bs ? bs->bs_flags : 0; }
int nvim_bufstate_get_seqnr(bufstate_T *bs) { return bs ? bs->bs_seqnr : 0; }
int nvim_bufstate_get_cchar(bufstate_T *bs) { return bs ? bs->bs_cchar : 0; }
reg_extmatch_T *nvim_bufstate_get_extmatch(bufstate_T *bs) { return bs ? bs->bs_extmatch : NULL; }

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

int nvim_syn_get_nsubexp(void) { return NSUBEXP; }

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

int nvim_stateitem_get_m_endpos_lnum(stateitem_T *item) { return item ? (int)item->si_m_endpos.lnum : 0; }
int nvim_stateitem_get_m_endpos_col(stateitem_T *item) { return item ? (int)item->si_m_endpos.col : 0; }
int nvim_stateitem_get_h_startpos_lnum(stateitem_T *item) { return item ? (int)item->si_h_startpos.lnum : 0; }
int nvim_stateitem_get_h_startpos_col(stateitem_T *item) { return item ? (int)item->si_h_startpos.col : 0; }
int nvim_stateitem_get_h_endpos_lnum(stateitem_T *item) { return item ? (int)item->si_h_endpos.lnum : 0; }
int nvim_stateitem_get_h_endpos_col(stateitem_T *item) { return item ? (int)item->si_h_endpos.col : 0; }
int nvim_stateitem_get_eoe_pos_lnum(stateitem_T *item) { return item ? (int)item->si_eoe_pos.lnum : 0; }
int nvim_stateitem_get_eoe_pos_col(stateitem_T *item) { return item ? (int)item->si_eoe_pos.col : 0; }

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

int nvim_syn_get_next_match_idx_value(void) { return next_match_idx; }
void nvim_syn_set_next_match_idx(int idx) { next_match_idx = idx; }
void nvim_syn_set_next_match_col(int col) { next_match_col = col; }
void nvim_syn_set_current_next_list_ptr(int16_t *list) { current_next_list = list; }
int16_t *nvim_syn_get_current_next_list_ptr(void) { return current_next_list; }
void nvim_syn_check_state_ends(void) { rs_check_state_ends(); }

void nvim_syn_call_update_si_attr(int idx) { rs_update_si_attr(idx); }

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

char nvim_syn_getcurline_at_col(void) { return syn_getcurline()[current_col]; }

int nvim_syn_current_state_is_empty(void) { return GA_EMPTY(&current_state) ? 1 : 0; }

void nvim_syn_set_current_finished(int finished) { current_finished = finished ? true : false; }

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

int nvim_syn_id2attr_wrapper(int syn_id) { return syn_id2attr(syn_id); }

void nvim_syn_call_syn_update_ends(int syncing) { syn_update_ends(syncing ? true : false); }

int16_t *nvim_stateitem_get_next_list(stateitem_T *item) { return item ? item->si_next_list : NULL; }

/// Set si_next_list for stateitem
void nvim_stateitem_set_next_list(stateitem_T *item, int16_t *list)
{
  if (item) {
    item->si_next_list = list;
  }
}

int nvim_syn_is_id_list_all(int16_t *list) { return list == ID_LIST_ALL ? 1 : 0; }
int16_t *nvim_syn_get_id_list_all(void) { return ID_LIST_ALL; }

/// Walk back through transparent items in current_state starting from item.
/// Returns the previous item if:
///   - item has HL_TRANS_CONT flag AND
///   - item is not the first element of current_state
/// Otherwise returns item unchanged.
stateitem_T *nvim_stateitem_prev_if_trans_cont(stateitem_T *item)
{
  if (item == NULL) {
    return NULL;
  }
  while ((item->si_flags & HL_TRANS_CONT)
         && item > (stateitem_T *)(current_state.ga_data)) {
    item--;
  }
  return item;
}

/// Get SYN_ITEMS(syn_block)[idx].sp_syn.id
int16_t nvim_syn_get_pattern_sp_syn_id(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return 0;
  }
  return SYN_ITEMS(syn_block)[idx].sp_syn.id;
}

/// Get SYN_ITEMS(syn_block)[idx].sp_syn.inc_tag
int nvim_syn_get_pattern_sp_syn_inc_tag(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return 0;
  }
  return SYN_ITEMS(syn_block)[idx].sp_syn.inc_tag;
}

/// Get SYN_ITEMS(syn_block)[idx].sp_syn.cont_in_list
int16_t *nvim_syn_get_pattern_sp_syn_cont_in_list(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return NULL;
  }
  return SYN_ITEMS(syn_block)[idx].sp_syn.cont_in_list;
}

/// Get SYN_CLSTR(syn_block)[idx].scl_list
int16_t *nvim_syn_get_cluster_scl_list(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_clusters.ga_len) {
    return NULL;
  }
  return SYN_CLSTR(syn_block)[idx].scl_list;
}

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

int nvim_syn_has_keywords(void) { return syn_block != NULL && syn_block->b_keywtab.ht_used > 0 ? 1 : 0; }
int nvim_syn_has_keywords_ic(void) { return syn_block != NULL && syn_block->b_keywtab_ic.ht_used > 0 ? 1 : 0; }

char *nvim_syn_getcurline(void) { return syn_getcurline(); }

void nvim_syn_save_chartab(char *buf) { save_chartab(buf); }

void nvim_syn_restore_chartab(char *buf) { restore_chartab(buf); }

int nvim_syn_get_maxkeywlen(void) { return MAXKEYWLEN; }

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

void nvim_syn_keyword_foldcase(char *src, int srclen, char *dst, int dstlen) { str_foldcase(src, srclen, dst, dstlen); }

int nvim_syn_utfc_ptr2len(char *p) { return utfc_ptr2len(p); }

void *nvim_syn_get_buf(void) { return syn_buf; }
void nvim_syn_set_syn_buf(void *buf) { syn_buf = (buf_T *)buf; }

int nvim_syn_current_state_len(void) { return current_state.ga_len; }

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

int nvim_syn_get_next_seqnr(void) { return next_seqnr; }
void nvim_syn_set_next_seqnr(int seqnr) { next_seqnr = seqnr; }

int nvim_syn_incr_next_seqnr(void) { return next_seqnr++; }

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

int nvim_syn_get_next_match_flags(void) { return next_match_flags; }
int nvim_syn_get_next_match_end_idx(void) { return next_match_end_idx; }
reg_extmatch_T *nvim_syn_get_next_match_extmatch(void) { return next_match_extmatch; }

reg_extmatch_T *nvim_syn_ref_extmatch(reg_extmatch_T *em) { return ref_extmatch(em); }

void nvim_syn_unref_extmatch(reg_extmatch_T *em) { unref_extmatch(em); }

stateitem_T *nvim_syn_push_next_match(void) { return rs_push_next_match(); }

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

int nvim_syn_is_current_state_empty(void) { return GA_EMPTY(&current_state) ? 1 : 0; }

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

int nvim_syn_get_sptype_start(void) { return SPTYPE_START; }
int nvim_syn_get_hl_oneline(void) { return HL_ONELINE; }
int nvim_syn_get_hl_keepend(void) { return HL_KEEPEND; }
int nvim_syn_get_hl_match(void) { return HL_MATCH; }
int nvim_syn_get_hl_conceal(void) { return HL_CONCEAL; }
int nvim_syn_get_hl_concealends(void) { return HL_CONCEALENDS; }
void nvim_syn_start_line(void) { syn_start_line(); }

int nvim_syn_finish_line(int syncing) { return rs_syn_finish_line(syncing); }

void nvim_syn_update_ends(int startofline) { syn_update_ends(startofline != 0); }

int nvim_syn_get_current_line_id(void) { return (int)current_line_id; }

void nvim_syn_incr_current_line_id(void) { current_line_id++; }

void *nvim_syn_get_syn_block(void) { return syn_block; }
void nvim_syn_set_syn_block(void *block) { syn_block = (synblock_T *)block; }
void *nvim_syn_get_syn_win(void) { return syn_win; }
void nvim_syn_set_syn_win(void *win) { syn_win = (win_T *)win; }
int nvim_syn_get_sync_minlines(void) { return syn_block ? (int)syn_block->b_syn_sync_minlines : 0; }
int nvim_syn_get_sync_maxlines(void) { return syn_block ? (int)syn_block->b_syn_sync_maxlines : 0; }
int nvim_syn_get_sync_flags(void) { return syn_block ? syn_block->b_syn_sync_flags : 0; }
int nvim_syn_get_sync_id(void) { return syn_block ? syn_block->b_syn_sync_id : 0; }
void *nvim_syn_get_sst_first(void) { return syn_block ? syn_block->b_sst_first : NULL; }
void *nvim_syn_get_sst_array(void) { return syn_block ? syn_block->b_sst_array : NULL; }
int nvim_syn_get_sst_len(void) { return syn_block ? syn_block->b_sst_len : 0; }

/// Set synstate sst_change_lnum
void nvim_synstate_set_change_lnum(synstate_T *p, int lnum)
{
  if (p) {
    p->sst_change_lnum = (linenr_T)lnum;
  }
}

void nvim_syn_set_current_id(int id) { current_id = (int16_t)id; }
void nvim_syn_set_current_trans_id(int id) { current_trans_id = (int16_t)id; }
void nvim_syn_set_current_flags(int flags) { current_flags = (int16_t)flags; }
void nvim_syn_set_current_seqnr(int seqnr) { current_seqnr = seqnr; }
int nvim_syn_get_hl_matchcont(void) { return HL_MATCHCONT; }
int nvim_syn_get_hl_extend(void) { return HL_EXTEND; }
int nvim_syn_get_sf_ccomment(void) { return SF_CCOMMENT; }
int nvim_syn_get_sf_match(void) { return SF_MATCH; }
int nvim_syn_get_hl_sync_here(void) { return HL_SYNC_HERE; }
int nvim_syn_get_hl_sync_there(void) { return HL_SYNC_THERE; }
int nvim_syn_get_sptype_match(void) { return SPTYPE_MATCH; }

void nvim_syn_stack_alloc(void) { syn_stack_alloc(); }
void *nvim_syn_stack_find_entry_ptr(int lnum) { return syn_stack_find_entry((linenr_T)lnum); }

void *nvim_win_get_synblock(void *wp) { return wp ? ((win_T *)wp)->w_s : NULL; }

void *nvim_syn_win_get_buffer_ptr(void *wp) { return wp ? ((win_T *)wp)->w_buffer : NULL; }

int nvim_win_get_foldnestmax(void *wp) { return wp ? (int)((win_T *)wp)->w_p_fdn : 0; }

int nvim_syn_buf_get_line_count(void *buf) { return buf ? (int)((buf_T *)buf)->b_ml.ml_line_count : 0; }
int nvim_syn_buf_get_changed_tick(void *buf) { return buf ? (int)buf_get_changedtick((buf_T *)buf) : 0; }

/// Set b_sst_lasttick in syn_block
void nvim_syn_set_sst_lasttick(int tick)
{
  if (syn_block) {
    syn_block->b_sst_lasttick = (disptick_T)tick;
  }
}

int nvim_syn_get_display_tick(void) { return (int)display_tick; }

void nvim_syn_line_breakcheck(void) { line_breakcheck(); }

int nvim_syn_get_got_int(void) { return got_int; }
int nvim_syn_get_rows(void) { return (int)Rows; }

void nvim_syn_stack_free_all(synblock_T *block) { syn_stack_free_all(block); }
void nvim_syn_stack_apply_changes(buf_T *buf) { syn_stack_apply_changes(buf); }

int nvim_buf_get_mod_top(buf_T *buf) { return (int)buf->b_mod_top; }
int nvim_buf_get_mod_bot(buf_T *buf) { return (int)buf->b_mod_bot; }
int nvim_buf_get_mod_xlines(buf_T *buf) { return (int)buf->b_mod_xlines; }

int nvim_synblock_get_linebreaks(synblock_T *block) { return block->b_syn_sync_linebreaks; }

void nvim_synstate_set_lnum(synstate_T *state, int lnum) { state->sst_lnum = lnum; }

int nvim_synstate_next_list_eq(synstate_T *a, synstate_T *b) { return a->sst_next_list == b->sst_next_list; }

/// Forward declaration for syn_scl_name2id

static int syn_scl_name2id(char *name);

int nvim_syn_cluster_name2id(const char *name) { return syn_scl_name2id((char *)name); }

int nvim_synblock_has_containedin(synblock_T *block) { return block->b_syn_containedin ? 1 : 0; }

int nvim_synblock_pattern_count(synblock_T *block) { return block->b_syn_patterns.ga_len; }

int nvim_synpat_get_inc_tag(synpat_T *pat) { return pat ? pat->sp_syn.inc_tag : 0; }

int nvim_synblock_is_spell_cluster(synblock_T *block, int id) { return id == block->b_spell_cluster_id; }
int nvim_synblock_is_nospell_cluster(synblock_T *block, int id) { return id == block->b_nospell_cluster_id; }

// nvim_syn_get_current_lnum already defined at line ~6119

// nvim_syn_get_current_col already defined at line ~6125

void nvim_syn_set_current_col(int col) { current_col = col; }
int nvim_syn_get_current_finished(void) { return current_finished; }
int nvim_syn_get_current_state_stored(void) { return current_state_stored; }

// nvim_synblock_get_syn_spell already defined at line ~5735

int nvim_buf_get_synmaxcol(buf_T *buf) { return (int)buf->b_p_smc; }

int nvim_syn_current_state_valid(void) { return !INVALID_STATE(&current_state); }

/// Validate current state if needed
void nvim_syn_ensure_current_state_valid(void)
{
  if (INVALID_STATE(&current_state)) {
    validate_current_state();
  }
}

const char *nvim_syn_get_current_line(void) { return syn_getcurline(); }

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

synblock_T *nvim_syn_get_block(void) { return syn_block; }
win_T *nvim_syn_get_win(void) { return syn_win; }

int nvim_syn_cur_foldlevel(void) { return nvim_syn_count_fold_items(); }

char **nvim_syn_get_cmdlinep(void) { return syn_cmdlinep; }

synblock_T *nvim_get_curwin_synblock(void) { return curwin->w_s; }

// nvim_get_curwin already defined in window.c

int nvim_syn_get_include_link(void) { return include_link; }
int nvim_syn_get_include_default(void) { return include_default; }
int nvim_syn_get_include_none(void) { return include_none; }
int nvim_syn_get_running_inc_tag(void) { return running_syn_inc_tag; }
void nvim_syn_set_running_inc_tag(int tag) { running_syn_inc_tag = tag; }
int nvim_syn_get_conceal_setting(synblock_T *block) { return block->b_syn_conceal; }
int nvim_syn_get_ic_setting(synblock_T *block) { return block->b_syn_ic; }

int nvim_get_syntax_info(int *seqnrp) { return get_syntax_info(seqnrp); }

void nvim_syntax_end_parsing(win_T *wp, int lnum) { syntax_end_parsing(wp, (linenr_T)lnum); }

/// Set tick on synstate
void nvim_synstate_set_tick(synstate_T *state, int tick)
{
  if (state) {
    state->sst_tick = tick;
  }
}

int nvim_synstate_get_tick_val(synstate_T *state) { return state ? state->sst_tick : 0; }

// Note: The following accessor functions are already defined earlier in the file:

// nvim_stateitem_get_m_lnum, nvim_stateitem_get_m_startcol,

// nvim_stateitem_set_m_lnum, nvim_stateitem_set_m_startcol,

// nvim_stateitem_set_cchar, nvim_stateitem_set_h_startpos,

// nvim_stateitem_get_cchar, nvim_stateitem_get_end_idx, nvim_stateitem_get_ends

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

int nvim_syn_getcurline_len(void) { return (int)ml_get_buf_len(syn_buf, current_lnum); }

int nvim_syn_get_line_len(int lnum) { return (int)ml_get_buf_len(syn_buf, (linenr_T)lnum); }
int nvim_syn_get_buf_line_count(void) { return (int)syn_buf->b_ml.ml_line_count; }

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

void nvim_syn_set_next_match_col_val(int col) { next_match_col = col; }

int nvim_syn_getcurline_byte_at(int col) { return (unsigned char)syn_getcurline()[col]; }

void nvim_syn_set_current_attr(int attr) { current_attr = attr; }
void nvim_syn_set_current_sub_char(int c) { current_sub_char = c; }

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

int nvim_syn_get_pattern_line_id(int idx) { return SYN_ITEMS(syn_block)[idx].sp_line_id; }
void nvim_syn_set_pattern_line_id(int idx, int line_id) { SYN_ITEMS(syn_block)[idx].sp_line_id = line_id; }
int nvim_syn_get_pattern_startcol(int idx) { return SYN_ITEMS(syn_block)[idx].sp_startcol; }
void nvim_syn_set_pattern_startcol(int idx, int col) { SYN_ITEMS(syn_block)[idx].sp_startcol = col; }
int nvim_syn_get_pattern_lc_off(int idx) { return SYN_ITEMS(syn_block)[idx].sp_offsets[SPO_LC_OFF]; }
int nvim_syn_get_pattern_syncing(int idx) { return SYN_ITEMS(syn_block)[idx].sp_syncing; }
int nvim_syn_get_pattern_display(int idx) { return (SYN_ITEMS(syn_block)[idx].sp_flags & HL_DISPLAY) != 0; }
int nvim_syn_get_pattern_ga_len(void) { return syn_block->b_syn_patterns.ga_len; }
int nvim_syn_has_containedin(void) { return syn_block->b_syn_containedin; }

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

int nvim_syn_get_spell_cluster_id(void) { return syn_block->b_spell_cluster_id; }
int nvim_syn_get_nospell_cluster_id(void) { return syn_block->b_nospell_cluster_id; }
int nvim_syn_get_syn_spell(void) { return syn_block->b_syn_spell; }

int nvim_syn_vim_iswordp_buf(char *p) { return vim_iswordp_buf(p, syn_buf); }

int nvim_syn_utf_head_off(char *base, char *p) { return utf_head_off(base, p); }

int nvim_syn_ascii_iswhite(int c) { return ascii_iswhite(c); }

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

void nvim_syn_load_current_state(synstate_T *from) { rs_load_current_state(from); }

int nvim_syn_match_linecont(linenr_T lnum) { return syn_match_linecont(lnum); }

/// Get sp_sync_idx for the current synblock pattern at index idx.
int nvim_syn_get_pattern_sync_idx(int idx)
{
  if (idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return -2;  // NONE_IDX
  }
  return SYN_ITEMS(syn_block)[idx].sp_sync_idx;
}

char *nvim_syn_ml_get(linenr_T lnum) { return ml_get_buf(syn_buf, lnum); }

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

_Static_assert(HL_DISPLAY == 0x1000, "HL_DISPLAY");
_Static_assert(HL_SKIPWHITE == 0x100, "HL_SKIPWHITE");
_Static_assert(HL_SYNC_HERE == 0x10, "HL_SYNC_HERE");
_Static_assert(HL_SYNC_THERE == 0x20, "HL_SYNC_THERE");
_Static_assert(SYNSPL_DEFAULT == 0, "SYNSPL_DEFAULT");
_Static_assert(SYNSPL_TOP == 1, "SYNSPL_TOP");
_Static_assert(SYNSPL_NOTOP == 2, "SYNSPL_NOTOP");
_Static_assert(KEYWORD_IDX == -1, "KEYWORD_IDX");

int nvim_syn_get_current_inc_tag(void) { return current_syn_inc_tag; }
int nvim_syn_get_b_syn_conceal(void) { return curwin->w_s->b_syn_conceal; }
int nvim_syn_check_cluster(char *pp, int len) { return syn_check_cluster(pp, len); }

int nvim_syn_name2id_wrapper(const char *name) { return syn_name2id(name); }

int nvim_syn_check_group_wrapper(const char *name, int len) { return syn_check_group(name, (size_t)len); }

int nvim_syn_highlight_num_groups(void) { return highlight_num_groups(); }
char *nvim_syn_highlight_group_name(int idx) { return highlight_group_name(idx); }

void *nvim_syn_vim_regcomp(char *pat, int flags) { return vim_regcomp(pat, flags); }

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

void nvim_syn_vim_regfree(void *regprog) { vim_regfree(regprog); }

int nvim_syn_foldmethod_is_syntax_curwin(void) { return rs_foldmethodIsSyntax(curwin); }

void nvim_syn_fold_update_all_curwin(void) { rs_foldUpdateAll(curwin); }

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

int nvim_syn_utf_ptr2char(const char *p) { return utf_ptr2char(p); }

// nvim_syn_utfc_ptr2len already defined above (line ~6694) with char * param

int nvim_syn_vim_isprintc(int c) { return vim_isprintc(c); }

char *nvim_syn_xstrnsave(const char *s, int len) { return xstrnsave(s, (size_t)len); }

void nvim_syn_xfree(void *ptr) { xfree(ptr); }

void *nvim_syn_xmalloc(int size) { return xmalloc((size_t)size); }

void nvim_syn_xmemcpyz(char *dst, const char *src, int len) { xmemcpyz(dst, src, (size_t)len); }

char *nvim_syn_strpbrk(const char *s, const char *chars) { return strpbrk(s, chars); }

void nvim_syn_emsg(const char *msg) { emsg(msg); }

void nvim_syn_semsg_1s(const char *fmt, const char *arg) { semsg(fmt, arg); }

char *nvim_syn_skipwhite(const char *p) { return skipwhite(p); }

char *nvim_syn_skiptowhite(const char *p) { return skiptowhite(p); }

int nvim_syn_ends_excmd(int c) { return ends_excmd(c); }

int nvim_syn_ascii_iswhite_char(int c) { return ascii_iswhite(c); }

int nvim_syn_toupper_asc(int c) { return TOUPPER_ASC(c); }

char *nvim_syn_get_group_name(char *arg, char **name_end) { return get_group_name(arg, name_end); }

void nvim_syn_init_patterns(void) { init_syn_patterns(); }

char *nvim_syn_vim_strnsave_up(const char *str, int len) { return vim_strnsave_up(str, (size_t)len); }

void nvim_syn_set_nextcmd(exarg_T *eap, char *rest) { eap->nextcmd = check_nextcmd(rest); }
char *nvim_syn_get_eap_arg(const exarg_T *eap) { return eap->arg; }
int nvim_syn_get_eap_skip(const exarg_T *eap) { return eap->skip; }

void nvim_syn_incl_toplevel(int id, int *flagsp) { syn_incl_toplevel(id, flagsp); }


_Static_assert(SF_CCOMMENT == 0x01, "SF_CCOMMENT");
_Static_assert(SF_MATCH == 0x02, "SF_MATCH");
_Static_assert(NONE_IDX == -2, "NONE_IDX");
_Static_assert(SYNID_ALLBUT == MAX_HL_ID, "SYNID_ALLBUT");
_Static_assert(SYNID_TOP == 21000, "SYNID_TOP");
_Static_assert(SYNID_CONTAINED == 22000, "SYNID_CONTAINED");
_Static_assert(HL_FOLD == 0x2000, "HL_FOLD");
_Static_assert(HL_EXCLUDENL == 0x800, "HL_EXCLUDENL");
_Static_assert(HL_HAS_EOL == 0x08, "HL_HAS_EOL");
_Static_assert(HL_INCLUDED_TOPLEVEL == 0x80000, "HL_INCLUDED_TOPLEVEL");
_Static_assert(SPTYPE_START == 2, "SPTYPE_START");
_Static_assert(SPTYPE_SKIP == 4, "SPTYPE_SKIP");
_Static_assert(SPTYPE_END == 3, "SPTYPE_END");

// =============================================================================
// Phase 1 accessors: get_syn_pattern migration
// =============================================================================

char *nvim_syn_skip_regexp(char *arg, int delim, int magic)
{
  return skip_regexp(arg, delim, magic);
}

int nvim_syn_getdigits_int(char **pp, int strict, int def)
{
  return getdigits_int(pp, (bool)strict, def);
}

char *nvim_syn_get_p_cpo(void) { return p_cpo; }
void nvim_syn_set_p_cpo(char *val) { p_cpo = val; }
char *nvim_syn_get_empty_string_option(void) { return empty_string_option; }
int nvim_syn_get_curwin_syn_ic(void) { return curwin->w_s->b_syn_ic; }

void nvim_synpat_set_pattern(synpat_T *pat, char *pattern) { pat->sp_pattern = pattern; }
void nvim_synpat_set_prog(synpat_T *pat, void *prog) { pat->sp_prog = prog; }
void nvim_synpat_set_ic(synpat_T *pat, int ic) { pat->sp_ic = ic; }
void nvim_synpat_set_off_flags(synpat_T *pat, int16_t flags) { pat->sp_off_flags = flags; }
// nvim_synpat_get_off_flags already defined at line 2875
void nvim_synpat_set_offset(synpat_T *pat, int idx, int val) { pat->sp_offsets[idx] = val; }
int nvim_synpat_get_offset(synpat_T *pat, int idx) { return pat->sp_offsets[idx]; }
void nvim_synpat_clear_time(synpat_T *pat) { syn_clear_time(&pat->sp_time); }

// =============================================================================
// Phase 2 accessors: syn_cmd_match migration
// =============================================================================

int nvim_syn_vim_regcomp_had_eol(void) { return vim_regcomp_had_eol(); }

// =============================================================================
// Phase 7 (pass 7) accessors: pattern_store migration
// =============================================================================

/// Grow the curwin b_syn_patterns garray by count slots.
void nvim_synblock_ga_grow_patterns(int count) { ga_grow(&curwin->w_s->b_syn_patterns, count); }

/// Append one slot to curwin b_syn_patterns and return a pointer to it.
synpat_T *nvim_synblock_ga_append_pattern(void)
{
  return GA_APPEND_VIA_PTR(synpat_T, &curwin->w_s->b_syn_patterns);
}

/// Copy synpat_T: *dst = *src.
void nvim_synpat_copy_from(synpat_T *dst, synpat_T *src) { *dst = *src; }

void nvim_synpat_set_syncing(synpat_T *pat, int syncing) { pat->sp_syncing = syncing; }
void nvim_synpat_set_type(synpat_T *pat, int type) { pat->sp_type = type; }
void nvim_synpat_set_flags(synpat_T *pat, int flags) { pat->sp_flags = flags; }
void nvim_synpat_or_flags(synpat_T *pat, int flags) { pat->sp_flags |= flags; }
void nvim_synpat_set_syn_id(synpat_T *pat, int id) { pat->sp_syn.id = (int16_t)id; }
void nvim_synpat_set_syn_inc_tag(synpat_T *pat, int tag) { pat->sp_syn.inc_tag = tag; }
void nvim_synpat_set_syn_match_id(synpat_T *pat, int id) { pat->sp_syn_match_id = (int16_t)id; }
void nvim_synpat_set_cchar(synpat_T *pat, int c) { pat->sp_cchar = c; }
void nvim_synpat_set_sync_idx(synpat_T *pat, int idx) { pat->sp_sync_idx = idx; }
void nvim_synpat_set_cont_list(synpat_T *pat, int16_t *list) { pat->sp_cont_list = list; }
void nvim_synpat_set_cont_in_list(synpat_T *pat, int16_t *list) { pat->sp_syn.cont_in_list = list; }
void nvim_synpat_set_next_list(synpat_T *pat, int16_t *list) { pat->sp_next_list = list; }
void nvim_synblock_set_containedin(int val) { curwin->w_s->b_syn_containedin = (bool)val; }
void nvim_synblock_or_sync_flags_curwin(int flags) { curwin->w_s->b_syn_sync_flags |= flags; }
void nvim_synblock_inc_folditems(void) { curwin->w_s->b_syn_folditems++; }

/// Allocate a zeroed synpat_T on the heap and return it.
synpat_T *nvim_syn_xcalloc_synpat(void) { return xcalloc(1, sizeof(synpat_T)); }

/// Free a heap-allocated synpat_T including sp_prog and sp_pattern.
void nvim_syn_free_synpat(synpat_T *pat)
{
  if (pat != NULL) {
    vim_regfree(pat->sp_prog);
    xfree(pat->sp_pattern);
    xfree(pat);
  }
}

/// Set the global reg_do_extmatch variable (REX_SET=1, REX_USE=2, 0=off).
void nvim_syn_set_reg_do_extmatch(int val) { reg_do_extmatch = val; }

// =============================================================================
// Phase 3 accessors: syn_cmd_keyword migration
// =============================================================================

void nvim_syn_semsg_2s(const char *fmt, const char *arg1, const char *arg2)
{
  semsg(fmt, arg1, arg2);
}

/// Wrapper for add_keyword.
void nvim_syn_add_keyword(char *name, int namelen, int id, int flags,
                           int16_t *cont_in_list, int16_t *next_list,
                           int conceal_char)
{
  add_keyword(name, (size_t)namelen, id, flags, cont_in_list, next_list, conceal_char);
}

// nvim_syn_keyword_redraw_and_free removed in pass 5 Phase 4 (replaced by
// nvim_syn_redraw_curbuf_later + nvim_syn_stack_free_all calls from Rust).

// =============================================================================
// Phase 4 accessors: syn_cmd_cluster migration
// =============================================================================

/// Return pointer to SYN_CLSTR(curwin->w_s)[scl_id].scl_list for Rust.
int16_t **nvim_syn_get_cluster_list_ptr(int scl_id)
{
  return &SYN_CLSTR(curwin->w_s)[scl_id].scl_list;
}

/// Combine a cluster's ID list with a new list, consuming both.
/// Calls syn_combine_list on the cluster's list and clstr_list.
void nvim_syn_combine_cluster_list(int scl_id, int16_t **clstr_list, int list_op)
{
  syn_combine_list(&SYN_CLSTR(curwin->w_s)[scl_id].scl_list, clstr_list, list_op);
}

// nvim_syn_redraw_and_free_all removed in pass 5 Phase 4 (replaced by
// nvim_syn_redraw_curbuf_later + nvim_syn_stack_free_all calls from Rust).

/// Set eap->nextcmd using find_nextcmd(arg).
void nvim_syn_find_nextcmd(exarg_T *eap, char *arg)
{
  eap->nextcmd = find_nextcmd(arg);
}

// =============================================================================
// Phase 1 accessors: syn_cmd_sync migration
// =============================================================================

/// Set eap->arg directly (used by sync MATCH/REGION/CLEAR sub-paths).
void nvim_syn_set_eap_arg(exarg_T *eap, char *arg)
{
  eap->arg = arg;
}

/// Wrap getdigits_int32 for Rust.
int nvim_syn_getdigits_int32(char **pp, int strict, int def)
{
  return getdigits_int32(pp, (bool)strict, def);
}

/// OR flags into b_syn_sync_flags.
void nvim_synblock_or_sync_flags(synblock_T *block, int flags)
{
  block->b_syn_sync_flags |= flags;
}

/// Set b_syn_sync_id.
void nvim_synblock_set_sync_id(synblock_T *block, int id)
{
  block->b_syn_sync_id = (int16_t)id;
}

/// Set b_syn_sync_minlines.
void nvim_synblock_set_sync_minlines(synblock_T *block, int n)
{
  block->b_syn_sync_minlines = (linenr_T)n;
}

/// Set b_syn_sync_maxlines.
void nvim_synblock_set_sync_maxlines(synblock_T *block, int n)
{
  block->b_syn_sync_maxlines = (linenr_T)n;
}

/// Set b_syn_sync_linebreaks.
void nvim_synblock_set_sync_linebreaks(synblock_T *block, int n)
{
  block->b_syn_sync_linebreaks = (linenr_T)n;
}

/// Return 1 if b_syn_linecont_pat is set (non-NULL), else 0.
int nvim_synblock_get_linecont_pat_is_set(synblock_T *block)
{
  return block->b_syn_linecont_pat != NULL ? 1 : 0;
}

/// Store linecont pattern: allocate, compile regexp, clear time.
/// pat_start points to the pattern text (not the delimiter).
/// pat_len is the length in bytes.
/// Returns 1 on success, 0 on regexp compile failure.
int nvim_synblock_set_linecont(synblock_T *block, const char *pat_start, int pat_len)
{
  block->b_syn_linecont_pat = xstrnsave(pat_start, (size_t)pat_len);
  block->b_syn_linecont_ic = block->b_syn_ic;

  // Make 'cpoptions' empty to avoid the 'l' flag
  char *cpo_save = p_cpo;
  p_cpo = empty_string_option;
  block->b_syn_linecont_prog = vim_regcomp(block->b_syn_linecont_pat, RE_MAGIC);
  p_cpo = cpo_save;
  syn_clear_time(&block->b_syn_linecont_time);

  if (block->b_syn_linecont_prog == NULL) {
    XFREE_CLEAR(block->b_syn_linecont_pat);
    return 0;
  }
  return 1;
}

/// Forward to rs_syn_cmd_match.
void nvim_syn_cmd_match_wrapper(exarg_T *eap, int syncing)
{
  rs_syn_cmd_match(eap, syncing);
}

/// Forward to rs_syn_cmd_region.
void nvim_syn_cmd_region_wrapper(exarg_T *eap, int syncing)
{
  rs_syn_cmd_region(eap, syncing);
}

/// Forward to rs_syn_cmd_clear.
void nvim_syn_cmd_clear_wrapper(exarg_T *eap, int syncing)
{
  rs_syn_cmd_clear(eap, syncing);
}

/// Forward to syn_cmd_list.
void nvim_syn_cmd_list_wrapper(exarg_T *eap, int syncing)
{
  syn_cmd_list(eap, syncing);
}

// =============================================================================
// Phase 2 accessors: syn_cmd_clear migration
// =============================================================================

/// Look up a syntax cluster name+length and return its ID (with SYNID_CLUSTER offset).
int nvim_syn_scl_namen2id(const char *arg, int len)
{
  return syn_scl_namen2id((char *)arg, len);
}

/// Look up a syntax group name+length and return its ID.
int nvim_syn_name2id_len_wrapper(const char *arg, int len)
{
  return syn_name2id_len(arg, (size_t)len);
}

/// Clear the scl_list of a cluster (by scl_id, not offset by SYNID_CLUSTER).
void nvim_synblock_clear_cluster_scl_list(synblock_T *block, int scl_id)
{
  XFREE_CLEAR(SYN_CLSTR(block)[scl_id].scl_list);
}

// nvim_syn_clear_one_wrapper, nvim_syntax_clear_wrapper, nvim_syntax_sync_clear_wrapper,
// and nvim_syn_clear_unlet_vars removed in pass 5 Phase 4 (Rust now calls rs_* directly).

// =============================================================================
// Phase 3 accessors: syn_cmd_include migration
// =============================================================================

/// Set current_syn_inc_tag.
void nvim_syn_set_current_inc_tag(int tag)
{
  current_syn_inc_tag = tag;
}

/// Atomically do: current_syn_inc_tag = ++running_syn_inc_tag.
/// Returns the new current_syn_inc_tag value.
int nvim_syn_increment_and_set_inc_tag(void)
{
  current_syn_inc_tag = ++running_syn_inc_tag;
  return current_syn_inc_tag;
}

/// Prepare for :syntax include.
/// Sets EX_XFILE|EX_NOSPC argt flags, calls separate_nextcmd,
/// checks if path is absolute/$/<, and optionally calls expand_filename.
/// Returns 1 if file should be :source'd (absolute path),
///         0 if file should be loaded via source_runtime,
///        -1 on expand_filename failure.
int nvim_syn_include_prepare(exarg_T *eap)
{
  eap->argt |= (EX_XFILE | EX_NOSPC);
  separate_nextcmd(eap);
  if (*eap->arg == '<' || *eap->arg == '$' || path_is_absolute(eap->arg)) {
    const char *errormsg = NULL;
    if (expand_filename(eap, syn_cmdlinep, &errormsg) == FAIL) {
      if (errormsg != NULL) {
        emsg(errormsg);
      }
      return -1;
    }
    return 1;  // use :source
  }
  return 0;  // use source_runtime
}

/// Execute :syntax include -- source or runtime load the file.
/// Returns 0 on success, -1 on failure (caller should emit e_notopen).
int nvim_syn_include_source(exarg_T *eap, int use_source)
{
  if (use_source) {
    if (do_source(eap->arg, false, DOSO_NONE, NULL) == FAIL) {
      return -1;
    }
  } else {
    if (source_runtime(eap->arg, DIP_ALL) == FAIL) {
      return -1;
    }
  }
  return 0;
}

// =============================================================================
// Phase 4 accessors: simple subcommands and syn_maybe_enable migration
// =============================================================================

/// Set eap->nextcmd = check_nextcmd(arg) (accessor for rs_syn_cmd_reset).
void nvim_syn_check_nextcmd(exarg_T *eap, char *arg)
{
  eap->nextcmd = check_nextcmd(arg);
}

/// Wrap init_highlight for rs_syn_cmd_reset.
void nvim_syn_init_highlight(int reset, int init)
{
  init_highlight((bool)reset, (bool)init);
}

/// Getter for did_syntax_onoff.
int nvim_syn_get_did_syntax_onoff(void)
{
  return did_syntax_onoff ? 1 : 0;
}

/// Execute the on/off/manual logic: set did_syntax_onoff, build "so ..." cmd, run it.
void nvim_syn_do_onoff(exarg_T *eap, const char *name)
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

/// Enable syntax (syn_maybe_enable helper): create minimal exarg_T and call rs_syn_cmd_on_dispatch.
void nvim_syn_do_maybe_enable(void)
{
  exarg_T ea;
  ea.arg = "";
  ea.skip = false;
  rs_syn_cmd_on_dispatch(&ea, false);
}

/// Redraw curwin with UPD_NOT_VALID (used after :syntax spell dispatch from Rust).
void nvim_syn_redraw_later_curwin(void)
{
  redraw_later(curwin, UPD_NOT_VALID);
}

/// Set syn_cmdlinep from eap->cmdlinep.
void nvim_syn_set_cmdlinep_from_eap(exarg_T *eap)
{
  syn_cmdlinep = eap->cmdlinep;
}

/// Call do_unlet(name, len, true) -- wrapper for Rust use.
void nvim_syn_do_unlet(const char *name, int len)
{
  do_unlet(name, (size_t)len, true);
}

/// Return 1 if block is the buffer's own synblock (block == &curwin->w_buffer->b_s).
int nvim_synblock_is_buf_block(synblock_T *block)
{
  return (block == &curwin->w_buffer->b_s) ? 1 : 0;
}

/// Trigger redraw_curbuf_later(UPD_SOME_VALID).
void nvim_syn_redraw_curbuf_later(void)
{
  redraw_curbuf_later(UPD_SOME_VALID);
}

// =============================================================================
// Phase syntime: accessors for syn_time_on and synpat timing fields
// =============================================================================

int nvim_syn_get_syn_time_on(void) { return syn_time_on ? 1 : 0; }
void nvim_syn_set_syn_time_on(int val) { syn_time_on = (val != 0); }

int nvim_synpat_get_time_count(synpat_T *pat) { return pat->sp_time.count; }
int nvim_synpat_get_time_match(synpat_T *pat) { return pat->sp_time.match; }
uint64_t nvim_synpat_get_time_total(synpat_T *pat) { return (uint64_t)pat->sp_time.total; }
uint64_t nvim_synpat_get_time_slowest(synpat_T *pat) { return (uint64_t)pat->sp_time.slowest; }

int nvim_syn_syntax_present_curwin(void) { return syntax_present(curwin) ? 1 : 0; }
int nvim_syn_get_columns(void) { return (int)Columns; }

/// Thin wrapper: ex_syntime body delegated to Rust.
extern void rs_ex_syntime(exarg_T *eap);
void ex_syntime(exarg_T *eap) { rs_ex_syntime(eap); }

/// Thin wrapper: get_syntime_arg body delegated to Rust.
extern char *rs_get_syntime_arg(expand_T *xp, int idx);
char *get_syntime_arg(expand_T *xp, int idx) { return rs_get_syntime_arg(xp, idx); }

// =============================================================================
// Phase expand: accessors for tab completion functions
// =============================================================================

void nvim_syn_set_include_link(int val) { include_link = val; }
void nvim_syn_set_include_default(int val) { include_default = val; }
void nvim_syn_set_include_none(int val) { include_none = val; }

/// Format cluster name "@name" into xp->xp_buf and return it.
char *nvim_syn_expand_cluster_name(expand_T *xp, int idx)
{
  synblock_T *block = curwin->w_s;
  if (idx < 0 || idx >= block->b_syn_clusters.ga_len) {
    return NULL;
  }
  vim_snprintf(xp->xp_buf, EXPAND_BUF_LEN, "@%s",
               SYN_CLSTR(block)[idx].scl_name);
  return xp->xp_buf;
}

int nvim_syn_get_expand_cluster_count(void)
{
  return curwin->w_s->b_syn_clusters.ga_len;
}

/// Thin wrappers: expand/context functions delegated to Rust.
extern void rs_set_context_in_syntax_cmd(expand_T *xp, const char *arg);
void set_context_in_syntax_cmd(expand_T *xp, const char *arg)
{
  rs_set_context_in_syntax_cmd(xp, arg);
}

extern char *rs_get_syntax_name(expand_T *xp, int idx);
char *get_syntax_name(expand_T *xp, int idx) { return rs_get_syntax_name(xp, idx); }

extern void rs_set_context_in_echohl_cmd(expand_T *xp, const char *arg);
void set_context_in_echohl_cmd(expand_T *xp, const char *arg)
{
  rs_set_context_in_echohl_cmd(xp, arg);
}

extern void rs_reset_expand_highlight(void);
void reset_expand_highlight(void) { rs_reset_expand_highlight(); }

// =============================================================================
// Phase listing: accessors for syn_cmd_list migration
// =============================================================================

/// Get the highlight link ID for highlight group at index (0-based).
int nvim_syn_highlight_link_id(int id) { return highlight_link_id(id); }

/// Wrap syn_list_header.
int nvim_syn_list_header(int did_header, int outlen, int id, int force_newline)
{
  return syn_list_header((bool)did_header, outlen, id, (bool)force_newline) ? 1 : 0;
}

/// Get msg_col.
int nvim_get_msg_col_syn(void) { return msg_col; }

/// nvim_msg_putchar already defined in message.c.

/// Wrap msg_puts_hl.
void nvim_msg_puts_hl_syn(const char *s, int hl_id, bool hist)
{
  msg_puts_hl(s, hl_id, hist);
}

/// Returns non-zero if character c is found in string s (wraps vim_strchr).
int nvim_syn_vim_strchr(const char *s, int c)
{
  return vim_strchr(s, (uint8_t)c) != NULL ? 1 : 0;
}

/// Hashtab iteration: returns (ht_mask + 1) = array size.
size_t nvim_ht_get_array_size(const hashtab_T *ht) { return ht->ht_mask + 1; }

/// Hashtab iteration: returns used count.
size_t nvim_ht_get_used(const hashtab_T *ht) { return ht->ht_used; }

/// Hashtab iteration: returns HI2KE at index if not empty, else NULL.
keyentry_T *nvim_ht_item_at(const hashtab_T *ht, size_t idx)
{
  const hashitem_T *hi = &ht->ht_array[idx];
  if (HASHITEM_EMPTY(hi)) {
    return NULL;
  }
  return HI2KE(hi);
}

/// Get b_keywtab pointer for block.
hashtab_T *nvim_synblock_keywtab_ptr(synblock_T *block) { return &block->b_keywtab; }

/// Get b_keywtab_ic pointer for block.
hashtab_T *nvim_synblock_keywtab_ic_ptr(synblock_T *block) { return &block->b_keywtab_ic; }

/// Get cluster name for curwin block at index.
char *nvim_curwin_syncluster_name(int idx)
{
  if (idx < 0 || idx >= curwin->w_s->b_syn_clusters.ga_len) {
    return NULL;
  }
  return SYN_CLSTR(curwin->w_s)[idx].scl_name;
}

/// Get cluster list for curwin block at index.
int16_t *nvim_curwin_syncluster_list(int idx)
{
  if (idx < 0 || idx >= curwin->w_s->b_syn_clusters.ga_len) {
    return NULL;
  }
  return SYN_CLSTR(curwin->w_s)[idx].scl_list;
}

/// Get curwin synblock cluster count.
int nvim_curwin_syncluster_count(void) { return curwin->w_s->b_syn_clusters.ga_len; }

/// Get curwin synblock pattern by index.
synpat_T *nvim_curwin_synpat_at(int idx)
{
  if (idx < 0 || idx >= curwin->w_s->b_syn_patterns.ga_len) {
    return NULL;
  }
  return &SYN_ITEMS(curwin->w_s)[idx];
}

/// Get curwin synblock pattern count.
int nvim_curwin_synpat_count(void) { return curwin->w_s->b_syn_patterns.ga_len; }

/// Get curwin synblock sync flags.
int nvim_curwin_syn_sync_flags(void) { return curwin->w_s->b_syn_sync_flags; }

/// Get curwin synblock sync minlines.
int nvim_curwin_syn_sync_minlines(void) { return (int)curwin->w_s->b_syn_sync_minlines; }

/// Get curwin synblock sync maxlines.
int nvim_curwin_syn_sync_maxlines(void) { return (int)curwin->w_s->b_syn_sync_maxlines; }

/// Get curwin synblock sync linebreaks.
int nvim_curwin_syn_sync_linebreaks(void) { return (int)curwin->w_s->b_syn_sync_linebreaks; }

/// Get the synpat at a specific index from the curwin synblock.
/// Unlike nvim_curwin_synpat_at, used by syn_list_one for the loop.
synpat_T *nvim_curwin_get_synpat_at(int idx)
{
  if (idx < 0 || idx >= curwin->w_s->b_syn_patterns.ga_len) {
    return NULL;
  }
  return &SYN_ITEMS(curwin->w_s)[idx];
}

/// Call find_nextcmd on arg for eap.
char *nvim_syn_find_nextcmd_arg(const char *arg) { return find_nextcmd(arg); }

// nvim_syn_ends_excmd already defined above (as nvim_syn_ends_excmd).
// nvim_syn_skiptowhite and nvim_syn_skipwhite already defined above.

/// Wrap syn_name2id_len (int length version for listing).
int nvim_syn_name2id_len(const char *name, int len)
{
  return (int)syn_name2id_len(name, (size_t)len);
}

// nvim_syn_scl_namen2id is already defined above.

/// Wrap check_nextcmd and store in eap->nextcmd (for listing).
void nvim_syn_eap_check_nextcmd(exarg_T *eap, char *arg)
{
  eap->nextcmd = check_nextcmd(arg);
}

// nvim_syn_get_eap_arg and nvim_syn_get_eap_skip already defined above.

// =============================================================================
// Phase 2 (pass 4): ex_syntax dispatcher migration accessors
// =============================================================================

/// Increment emsg_skip (suppress error messages during :syntax with skip set).
void nvim_syn_emsg_skip_inc(void)
{
  emsg_skip++;
}

/// Decrement emsg_skip.
void nvim_syn_emsg_skip_dec(void)
{
  emsg_skip--;
}

// =============================================================================
// Phase 4 (pass 4): syn_cmd_iskeyword and ex_ownsyntax migration accessors
// =============================================================================

/// Return 1 if the synblock's b_syn_isk is set (not empty_string_option).
int nvim_syn_iskeyword_is_set(synblock_T *block)
{
  return block->b_syn_isk != empty_string_option ? 1 : 0;
}

/// Return block->b_syn_isk (the iskeyword option string).
char *nvim_syn_iskeyword_get(synblock_T *block)
{
  return block->b_syn_isk;
}

/// Clear the iskeyword setting: copy curbuf->b_chartab into block->b_syn_chartab
/// and clear block->b_syn_isk.
void nvim_syn_iskeyword_clear(synblock_T *block)
{
  memmove(block->b_syn_chartab, curbuf->b_chartab, (size_t)32);
  clear_string_option(&block->b_syn_isk);
}

/// Set the iskeyword: save curbuf state, set b_p_isk to arg, run buf_init_chartab,
/// copy result into block->b_syn_chartab, restore curbuf state, transfer isk to block.
void nvim_syn_iskeyword_set(synblock_T *block, const char *arg)
{
  char save_chartab[32];
  memmove(save_chartab, curbuf->b_chartab, (size_t)32);
  char *save_isk = curbuf->b_p_isk;
  curbuf->b_p_isk = xstrdup(arg);
  buf_init_chartab(curbuf, false);
  memmove(block->b_syn_chartab, curbuf->b_chartab, (size_t)32);
  memmove(curbuf->b_chartab, save_chartab, (size_t)32);
  clear_string_option(&block->b_syn_isk);
  block->b_syn_isk = curbuf->b_p_isk;
  curbuf->b_p_isk = save_isk;
}

/// msg_outtrans wrapper for syn_cmd_iskeyword display.
void nvim_syn_msg_outtrans(const char *s)
{
  msg_outtrans(s, 0, false);
}

/// Initialise ownsyntax: allocate new synblock for curwin (if sharing buffer's),
/// initialise hashtabs, clear spell and string options.
/// Returns 1 if a new block was created, 0 if curwin already owns its synblock.
int nvim_syn_ownsyntax_init(void)
{
  if (curwin->w_s == &curwin->w_buffer->b_s) {
    curwin->w_s = xcalloc(1, sizeof(synblock_T));
    hash_init(&curwin->w_s->b_keywtab);
    hash_init(&curwin->w_s->b_keywtab_ic);
    curwin->w_p_spell = false;
    clear_string_option(&curwin->w_s->b_p_spc);
    clear_string_option(&curwin->w_s->b_p_spf);
    clear_string_option(&curwin->w_s->b_p_spl);
    clear_string_option(&curwin->w_s->b_p_spo);
    clear_string_option(&curwin->w_s->b_syn_isk);
    return 1;
  }
  return 0;
}

/// Get value of a Vim variable (b:current_syntax etc.).
/// Returns pointer to the string (owned by Vim eval), or NULL.
char *nvim_syn_get_var_value(const char *name)
{
  return get_var_value(name);
}

/// Duplicate a C string via xstrdup (callers must free with xfree).
char *nvim_syn_xstrdup(const char *s)
{
  return xstrdup(s);
}

/// Apply EVENT_SYNTAX autocmds for :ownsyntax.
void nvim_syn_apply_autocmds_syntax(const char *arg)
{
  apply_autocmds(EVENT_SYNTAX, arg, curbuf->b_fname, true, curbuf);
}

/// Set an internal Vim string variable.
void nvim_syn_set_internal_string_var(const char *name, const char *val)
{
  set_internal_string_var(name, val);
}

/// Unlet b:current_syntax.
void nvim_syn_do_unlet_b_current_syntax(void)
{
  do_unlet(S_LEN("b:current_syntax"), true);
}

/// Forward syn_cmd_iskeyword (static) for Rust dispatch table.
void nvim_syn_cmd_iskeyword_wrapper(exarg_T *eap, int syncing)
{
  syn_cmd_iskeyword(eap, syncing);
}

/// Thin wrapper: syn_cmd_list body delegated to Rust.
extern void rs_syn_cmd_list(exarg_T *eap, int syncing);
static void syn_cmd_list(exarg_T *eap, int syncing)
{
  rs_syn_cmd_list(eap, syncing);
}

// =============================================================================
// Phase 5 (pass 5) accessors: clearing.rs support
// =============================================================================

/// Set b_syn_patterns.ga_len (used by rs_syn_remove_pattern to compact).
void nvim_synblock_set_pattern_count(synblock_T *block, int len)
{
  block->b_syn_patterns.ga_len = len;
}

/// Memmove within SYN_ITEMS array: copy [src_idx..src_idx+count) to dst_idx.
void nvim_synblock_memmove_patterns(synblock_T *block, int dst_idx, int src_idx, int count)
{
  synpat_T *base = SYN_ITEMS(block);
  memmove(base + dst_idx, base + src_idx, sizeof(synpat_T) * (size_t)count);
}

/// Decrement b_syn_folditems.
void nvim_synblock_dec_folditems(synblock_T *block)
{
  block->b_syn_folditems--;
}

/// Clear keyword entries with given syn id from a hashtab.
/// Keeps all HI2KE/KE2HIKEY/hash_remove pointer arithmetic in C.
/// NOTE: This must NOT call syn_clear_keyword() to avoid circular calls.
void nvim_syn_clear_keyword_in_ht(int id, hashtab_T *ht)
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

// =============================================================================
// Phase 5 pass 5 Phase 2 accessors: syntax_clear / reset_synblock / syntax_sync_clear
// =============================================================================

/// Clear both keyword tables for a synblock.
/// NOTE: Calls nvim_syn_clear_keywtab_ht directly to avoid circular calls
/// through clear_keywtab -> rs_clear_keywtab -> nvim_syn_clear_keywtab_ht.
void nvim_synblock_clear_keytabs(synblock_T *block)
{
  nvim_syn_clear_keywtab_ht(&block->b_keywtab);
  nvim_syn_clear_keywtab_ht(&block->b_keywtab_ic);
}

/// ga_clear on b_syn_patterns.
void nvim_synblock_ga_clear_patterns(synblock_T *block)
{
  ga_clear(&block->b_syn_patterns);
}

/// ga_clear on b_syn_clusters.
void nvim_synblock_ga_clear_clusters(synblock_T *block)
{
  ga_clear(&block->b_syn_clusters);
}

/// Free b_syn_linecont_prog and clear b_syn_linecont_pat.
void nvim_synblock_clear_linecont(synblock_T *block)
{
  vim_regfree(block->b_syn_linecont_prog);
  block->b_syn_linecont_prog = NULL;
  XFREE_CLEAR(block->b_syn_linecont_pat);
}

/// Reset all b_syn_sync_* flags and b_syn_folditems to defaults.
void nvim_synblock_reset_sync_flags(synblock_T *block)
{
  block->b_syn_sync_flags = 0;
  block->b_syn_sync_minlines = 0;
  block->b_syn_sync_maxlines = 0;
  block->b_syn_sync_linebreaks = 0;
  block->b_syn_folditems = 0;
}

/// Reset b_spell_cluster_id and b_nospell_cluster_id.
void nvim_synblock_reset_cluster_ids(synblock_T *block)
{
  block->b_spell_cluster_id = 0;
  block->b_nospell_cluster_id = 0;
}

/// Reset error/timeout/case/foldlevel/spell/containedin/conceal flags.
void nvim_synblock_reset_flags(synblock_T *block)
{
  block->b_syn_error = false;
  block->b_syn_slow = false;
  block->b_syn_ic = false;
  block->b_syn_foldlevel = SYNFLD_START;
  block->b_syn_spell = SYNSPL_DEFAULT;
  block->b_syn_containedin = false;
  block->b_syn_conceal = false;
}

/// clear_string_option on b_syn_isk.
void nvim_synblock_clear_syn_isk(synblock_T *block)
{
  clear_string_option(&block->b_syn_isk);
}

/// Reset running_syn_inc_tag to 0.
void nvim_syn_reset_inc_tag(void)
{
  running_syn_inc_tag = 0;
}

/// Release ownsyntax block for a window: clear it, free it, reset to buf's b_s.
void nvim_win_release_synblock(win_T *wp)
{
  if (wp->w_s != &wp->w_buffer->b_s) {
    syntax_clear(wp->w_s);
    xfree(wp->w_s);
    wp->w_s = &wp->w_buffer->b_s;
  }
}

/// clear_syn_state wrapper for use from Rust.
void nvim_syn_clear_syn_state(synstate_T *p)
{
  clear_syn_state(p);
}

// =============================================================================
// Phase 5 pass 5 Phase 3 accessors: syn_clear_keyword / clear_keywtab / invalidate_current_state
// =============================================================================

/// Clear a whole keyword hashtable (free entries, then hash_clear + hash_init).
/// NOTE: This must NOT call clear_keywtab() to avoid circular calls.
void nvim_syn_clear_keywtab_ht(hashtab_T *ht)
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

/// Set current_state.ga_itemsize = 0 to mark state as invalid.
void nvim_syn_set_current_state_invalid(void)
{
  current_state.ga_itemsize = 0;
}

// =============================================================================
// Phase 6 accessors: cluster management migration (syn_scl_name2id, syn_add_cluster)
// =============================================================================

/// Append a new (zeroed) cluster entry to curwin->w_s->b_syn_clusters.
/// Initializes the garray if needed. Returns the index of the new entry,
/// or -1 if we have hit MAX_CLUSTER_ID.
int nvim_synblock_cluster_append(void)
{
  synblock_T *block = curwin->w_s;
  // First call for this growarray: init growing array.
  if (block->b_syn_clusters.ga_data == NULL) {
    block->b_syn_clusters.ga_itemsize = sizeof(syn_cluster_T);
    ga_set_growsize(&block->b_syn_clusters, 10);
  }
  int len = block->b_syn_clusters.ga_len;
  if (len >= MAX_CLUSTER_ID) {
    emsg(_("E848: Too many syntax clusters"));
    return -1;
  }
  syn_cluster_T *scp = GA_APPEND_VIA_PTR(syn_cluster_T, &block->b_syn_clusters);
  CLEAR_POINTER(scp);
  return len;
}

/// Set the scl_name field of cluster at index idx in curwin->w_s->b_syn_clusters.
void nvim_synblock_set_cluster_name(int idx, char *name)
{
  SYN_CLSTR(curwin->w_s)[idx].scl_name = name;
}

/// Set the scl_name_u field of cluster at index idx in curwin->w_s->b_syn_clusters.
void nvim_synblock_set_cluster_name_u(int idx, char *name_u)
{
  SYN_CLSTR(curwin->w_s)[idx].scl_name_u = name_u;
}

/// Set the scl_list field of cluster at index idx in curwin->w_s->b_syn_clusters.
void nvim_synblock_set_cluster_list(int idx, int16_t *list)
{
  SYN_CLSTR(curwin->w_s)[idx].scl_list = list;
}

/// Set b_spell_cluster_id on curwin->w_s.
void nvim_synblock_set_spell_cluster_id(int id)
{
  curwin->w_s->b_spell_cluster_id = id;
}

/// Set b_nospell_cluster_id on curwin->w_s.
void nvim_synblock_set_nospell_cluster_id(int id)
{
  curwin->w_s->b_nospell_cluster_id = id;
}

/// vim_strsave_up for a null-terminated string -- used by rs_syn_add_cluster.
char *nvim_syn_vim_strsave_up(const char *s)
{
  return vim_strsave_up(s);
}

// =============================================================================
// Phase 6 accessors: init_syn_patterns migration
// =============================================================================

/// Initialize b_syn_patterns garray on curwin->w_s.
void nvim_synblock_ga_init_patterns(void)
{
  curwin->w_s->b_syn_patterns.ga_itemsize = sizeof(synpat_T);
  ga_set_growsize(&curwin->w_s->b_syn_patterns, 10);
}

// =============================================================================
// Phase 6 accessors: syn_incl_toplevel migration
// =============================================================================

/// Get b_syn_topgrp from curwin->w_s.
int nvim_syn_get_topgrp_curwin(void)
{
  return curwin->w_s->b_syn_topgrp;
}

// =============================================================================
// Phase 6 accessors: add_keyword + copy_id_list migration
// =============================================================================

/// Perform the full hashtab keyword insertion for add_keyword.
/// name_ic is a NUL-terminated foldcased (or original) keyword string.
/// Allocates a keyentry_T of the appropriate size, fills all fields,
/// and inserts into curwin->w_s->b_{keywtab,keywtab_ic}.
/// Ownership of cont_in_list_copy and next_list_copy is transferred.
void nvim_syn_hash_insert_keyword(const char *name_ic, int name_iclen,
                                   int id, int inc_tag, int flags,
                                   int conceal_char,
                                   int16_t *cont_in_list_copy,
                                   int16_t *next_list_copy,
                                   int use_ic)
{
  keyentry_T *const kp = xmalloc(offsetof(keyentry_T, keyword) + (size_t)name_iclen + 1);
  STRCPY(kp->keyword, name_ic);
  kp->k_syn.id = (int16_t)id;
  kp->k_syn.inc_tag = inc_tag;
  kp->flags = flags;
  kp->k_char = conceal_char;
  kp->k_syn.cont_in_list = cont_in_list_copy;
  if (cont_in_list_copy != NULL) {
    curwin->w_s->b_syn_containedin = true;
  }
  kp->next_list = next_list_copy;

  const hash_T hash = hash_hash(kp->keyword);
  hashtab_T *const ht = use_ic ? &curwin->w_s->b_keywtab_ic : &curwin->w_s->b_keywtab;
  hashitem_T *const hi = hash_lookup(ht, kp->keyword, (size_t)name_iclen, hash);

  if (HASHITEM_EMPTY(hi)) {
    kp->ke_next = NULL;
    hash_add_item(ht, hi, kp->keyword, hash);
  } else {
    kp->ke_next = HI2KE(hi);
    hi->hi_key = KE2HIKEY(kp);
  }
}
