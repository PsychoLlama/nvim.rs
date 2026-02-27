// syntax_accessors.c: syntax highlighting - struct definitions, static state,
// helper functions, and FFI accessor functions for Rust interop

#include <assert.h>
#include <inttypes.h>
#include <limits.h>
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
// Phase 8: state stack cache management (Rust implementations)
extern synstate_T *rs_syn_stack_find_entry(int lnum);
extern void rs_syn_stack_free_entry(synblock_T *block, synstate_T *p);
extern void rs_syn_stack_free_block(synblock_T *block);
extern void rs_syn_stack_free_all(synblock_T *block);
extern int rs_syn_stack_cleanup(void);
extern void rs_syn_stack_alloc(void);
extern void rs_syn_stack_apply_changes_block(synblock_T *block, buf_T *buf);
extern void rs_syn_stack_apply_changes(buf_T *buf);
// Phase 8: line initialization (Rust implementations)
extern void rs_syn_update_ends(int startofline);
extern void rs_syn_start_line(void);
extern int rs_syn_match_linecont(int lnum);
extern void rs_save_chartab(char *chartab);
extern void rs_restore_chartab(const char *chartab);
extern void rs_clear_syn_state(synstate_T *p);
extern void rs_validate_current_state(void);
extern char *rs_syn_getcurline(void);
extern int rs_syn_getcurline_len(void);
extern void rs_syn_clear_time(syn_time_T *st);
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

// Phase 9: state_entry.rs Rust implementations
extern void rs_syn_stack_remove_entry(synstate_T *sp);
extern synstate_T *rs_syn_stack_alloc_entry(int lnum, synstate_T *after);
extern void rs_syn_store_state_to_entry(synstate_T *sp);
extern void rs_syn_do_stack_realloc(int len);

// Phase 11: state_entry.rs Phase 11 Rust implementations
extern void rs_syn_store_bufstates(synstate_T *sp);

// Phase 11: commands.rs Rust implementations for do_onoff and maybe_enable
extern void rs_syn_do_onoff_impl(exarg_T *eap, const char *name);
extern void rs_syn_do_maybe_enable_impl(void);

// Phase 11: keyword.rs Rust implementations for keyword_find and hash_insert_keyword
extern keyentry_T *rs_syn_keyword_find(char *keyword, int use_ic);
extern void rs_syn_hash_insert_keyword(const char *name_ic, int name_iclen,
                                        int id, int inc_tag, int flags,
                                        int conceal_char,
                                        int16_t *cont_in_list_copy,
                                        int16_t *next_list_copy,
                                        int use_ic);

// Phase 11: commands.rs / cluster.rs Rust implementations for ownsyntax_init, cluster_append
extern int rs_syn_ownsyntax_init(void);
extern int rs_synblock_cluster_append(void);

// Phase 9.2: state_ops.rs Rust implementations
extern void rs_syn_pop_current_state(void);
extern void rs_syn_push_current_state(int idx);
extern void rs_syn_set_cur_state_item(int idx, int si_idx, int si_flags, int si_seqnr,
                                      int si_cchar, reg_extmatch_T *extmatch);
extern int rs_syn_count_fold_items(void);
extern int rs_syn_state_item_spans_line(int idx, int lnum);
extern void rs_syn_clear_current_state(void);
extern stateitem_T *rs_stateitem_prev_if_trans_cont(stateitem_T *item);

// Phase 9.3: extmatch comparison and cur_state_set_matchcont
extern int rs_syn_extmatch_equal(reg_extmatch_T *a, reg_extmatch_T *b);
extern int rs_syn_extmatch_strings_equal(reg_extmatch_T *a, reg_extmatch_T *b,
                                          int subidx, int pat_idx);
extern void rs_cur_state_set_matchcont(int i);

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

// Rust list command FFI declaration
extern void rs_syn_cmd_list(exarg_T *eap, int syncing);

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


// Try to find a synchronisation point for line "lnum".
//
// This sets current_lnum and the current state.  One of three methods is
// used:
// 1. Search backwards for the end of a C-comment.
// 2. Search backwards for given sync patterns.
// 3. Simply start on a given number of lines above "lnum".

static void save_chartab(char *chartab)
{
  rs_save_chartab(chartab);
}

static void restore_chartab(char *chartab)
{
  rs_restore_chartab(chartab);
}

/// Return true if the line-continuation pattern matches in line "lnum".
static int syn_match_linecont(linenr_T lnum)
{
  return rs_syn_match_linecont((int)lnum);
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
  rs_syn_stack_free_block(block);
}
// Free b_sst_array[] for buffer "buf".
// Used when syntax items changed to force resyncing everywhere.
void syn_stack_free_all(synblock_T *block)
{
  rs_syn_stack_free_all(block);
}

// Allocate the syntax state stack for syn_buf when needed.
// Delegated to Rust; actual array allocation is in nvim_syn_do_stack_realloc().
static void syn_stack_alloc(void)
{
  rs_syn_stack_alloc();
}

// Check for changes in a buffer to affect stored syntax states.
// Delegated to Rust; window iteration stays in nvim_syn_apply_changes_for_windows().
void syn_stack_apply_changes(buf_T *buf)
{
  rs_syn_stack_apply_changes(buf);
}

static void syn_stack_apply_changes_block(synblock_T *block, buf_T *buf)
{
  rs_syn_stack_apply_changes_block(block, buf);
}

/// Reduce the number of entries in the state stack for syn_buf.
///
/// @return  true if at least one entry was freed.
static bool syn_stack_cleanup(void)
{
  return rs_syn_stack_cleanup() != 0;
}

// Free the allocated memory for a syn_state item.
// Move the entry into the free list.
static void syn_stack_free_entry(synblock_T *block, synstate_T *p)
{
  rs_syn_stack_free_entry(block, p);
}

// Find an entry in the list of state stacks at or before "lnum".
// Returns NULL when there is no entry or the first entry is after "lnum".
static synstate_T *syn_stack_find_entry(linenr_T lnum)
{
  return rs_syn_stack_find_entry((int)lnum);
}

// End of handling of the state stack.
// **************************************


/// Update an entry in the current_state stack for a start-skip-end pattern.
/// This finds the end of the current item, if it's in the current line.
///
/// @param startcol  where to start searching for the end
/// @param force     when true overrule a previous end
///


/// Thin C helper: set up regmmatch_T, call vim_regexec_multi, extract results.
/// Profiling/timeout/b_syn_slow logic is handled by the Rust caller.
/// Returns 1 on match (with positions set), 0 on no match, -1 if regprog is NULL.
/// Sets *out_timed_out if vim_regexec_multi reported a timeout.
int nvim_syn_do_regexec(void *regprog, int ic,
                        int lnum, int col,
                        int *out_s_lnum, int *out_s_col,
                        int *out_e_lnum, int *out_e_col,
                        void **out_regprog, int *out_timed_out)
{
  if (regprog == NULL) {
    return -1;
  }
  regmmatch_T regmatch;
  regmatch.regprog = (regprog_T *)regprog;
  regmatch.rmm_ic = ic;
  regmatch.rmm_maxcol = (colnr_T)syn_buf->b_p_smc;
  *out_timed_out = 0;
  int r = vim_regexec_multi(&regmatch, syn_win, syn_buf,
                            (linenr_T)lnum, (colnr_T)col,
                            syn_tm, out_timed_out);
  *out_regprog = regmatch.regprog;
  if (r > 0) {
    *out_s_lnum = regmatch.startpos[0].lnum + lnum;
    *out_s_col  = regmatch.startpos[0].col;
    *out_e_lnum = regmatch.endpos[0].lnum + lnum;
    *out_e_col  = regmatch.endpos[0].col;
    return 1;
  }
  return 0;
}

int nvim_syn_get_b_syn_slow(void) { return syn_win->w_s->b_syn_slow ? 1 : 0; }
void nvim_syn_set_b_syn_slow(int val) { syn_win->w_s->b_syn_slow = (val != 0); }

/// Get pointer to syn_block->b_syn_linecont_time as void*.
/// Returns pointer to the inline syn_time_T field in the synblock.
/// NOTE: nvim_syn_block_get_linecont_time_ptr already exists; this is a new alias.

/// Get pointer to sp_time for a pattern at index in the current synblock.
/// Used by Rust to pass as *mut c_void to nvim_syn_time_update.
void *nvim_syn_get_pat_time_ptr(int idx)
{
  if (syn_block == NULL || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) {
    return NULL;
  }
  return (void *)&SYN_ITEMS(syn_block)[idx].sp_time;
}

/// Update syn_time_T fields from Rust after a regex execution.
/// elapsed: elapsed time (proftime_T / u64).
/// matched: 1 if the regex matched, 0 otherwise.
void nvim_syn_time_update(void *st_ptr, uint64_t elapsed, int matched)
{
  if (st_ptr == NULL) return;
  syn_time_T *st = (syn_time_T *)st_ptr;
  st->total = profile_add(st->total, (proftime_T)elapsed);
  if (profile_cmp((proftime_T)elapsed, st->slowest) < 0) {
    st->slowest = (proftime_T)elapsed;
  }
  st->count++;
  if (matched) {
    st->match++;
  }
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

void syn_maybe_enable(void)
{
  rs_syn_maybe_enable();
}

// syn_cmd_list and listing functions are implemented in Rust (listing.rs).

// Phase 6+7: Forward declarations for Rust functions used below
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
extern void rs_syn_combine_list(int16_t **clstr1, int16_t **clstr2, int list_op);
extern int rs_syn_in_id_list(stateitem_T *cur_si, int16_t *list, int id, int inc_tag,
                              int16_t *cont_in_list, int flags);
extern char *rs_get_syn_pattern(char *arg, synpat_T *ci);

// Keep ITEM_* defines available for C wrappers
#define ITEM_START          0
#define ITEM_SKIP           1
#define ITEM_END            2
#define ITEM_MATCHGROUP     3

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

// Return the syntax ID at position "i" in the current stack.
// The caller must have called syn_get_id() before to fill the stack.
// Returns -1 when "i" is out of range.
int syn_get_stack_item(int i)
{
  return rs_syn_get_stack_item(i);
}

// ":syntime" and "get_syntime_arg" are implemented in Rust (syntime.rs).
// Their C thin wrappers are at the bottom of this file.


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

int nvim_stateitem_has_trans_cont(stateitem_T *item) { return (item->si_flags & HL_TRANS_CONT) != 0; }
int nvim_stateitem_has_match(stateitem_T *item) { return (item->si_flags & HL_MATCH) != 0; }
int16_t *nvim_stateitem_get_cont_list(stateitem_T *item) { return item->si_cont_list; }
int nvim_stateitem_has_cont_list(stateitem_T *item) { return item->si_cont_list != NULL; }

int nvim_syn_get_topgrp(void) { return curwin->w_s->b_syn_topgrp; }
void nvim_syn_set_topgrp(int topgrp) { curwin->w_s->b_syn_topgrp = topgrp; }

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
int nvim_syn_state_item_spans_line(int idx, int lnum)
{
  return rs_syn_state_item_spans_line(idx, lnum);
}

synstate_T *nvim_syn_stack_find_entry(int lnum) { return syn_stack_find_entry((linenr_T)lnum); }

void nvim_syn_stack_remove_entry(synstate_T *sp) { rs_syn_stack_remove_entry(sp); }
synstate_T *nvim_syn_stack_alloc_entry(int lnum, synstate_T *after)
{
  return rs_syn_stack_alloc_entry(lnum, after);
}
void nvim_syn_store_state_to_entry(synstate_T *sp) { rs_syn_store_state_to_entry(sp); }

void nvim_syn_set_state_stored(int stored) { current_state_stored = stored ? true : false; }

void nvim_syn_clear_current_state(void) { rs_syn_clear_current_state(); }

void nvim_syn_validate_current_state(void) { rs_validate_current_state(); }

void nvim_syn_invalidate_current_state(void) { rs_invalidate_current_state(); }

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

void nvim_syn_set_cur_state_item(int idx, int si_idx, int si_flags, int si_seqnr,
                                  int si_cchar, reg_extmatch_T *extmatch)
{
  rs_syn_set_cur_state_item(idx, si_idx, si_flags, si_seqnr, si_cchar, extmatch);
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
  return rs_syn_extmatch_equal(a, b);
}

/// Compare extmatch strings at given sub-index with ignore-case from pattern
/// Returns 1 if equal, 0 if different
int nvim_syn_extmatch_strings_equal(reg_extmatch_T *a, reg_extmatch_T *b,
                                     int subidx, int pat_idx)
{
  return rs_syn_extmatch_strings_equal(a, b, subidx, pat_idx);
}

/// Get the sp_ic (ignore case) flag for a pattern at index
int nvim_synblock_pattern_ic(int pat_idx)
{
  if (syn_block == NULL || pat_idx < 0 || pat_idx >= syn_block->b_syn_patterns.ga_len) {
    return 0;
  }
  return SYN_ITEMS(syn_block)[pat_idx].sp_ic;
}

/// Get matches[subidx] string from a reg_extmatch_T.
const char *nvim_extmatch_get_string(reg_extmatch_T *em, int subidx)
{
  if (em == NULL || subidx < 0 || subidx >= NSUBEXP) {
    return NULL;
  }
  return (const char *)em->matches[subidx];
}

/// Wrap mb_strcmp_ic for Rust callers.
int nvim_syn_mb_strcmp_ic(int ic, const char *a, const char *b)
{
  if (a == NULL || b == NULL) {
    return a == b ? 0 : 1;
  }
  return mb_strcmp_ic(ic, a, b);
}

/// Get si_extmatch from a stateitem
reg_extmatch_T *nvim_stateitem_get_extmatch(stateitem_T *item)
{
  if (item == NULL) {
    return NULL;
  }
  return item->si_extmatch;
}

/// Bulk getter: fetch all position fields of a stateitem.
/// Callers may pass NULL for fields they do not need.
void nvim_stateitem_get_positions(stateitem_T *item,
    int *m_lnum, int *m_startcol,
    int *m_end_lnum, int *m_end_col,
    int *h_start_lnum, int *h_start_col,
    int *h_end_lnum, int *h_end_col,
    int *eoe_lnum, int *eoe_col)
{
  if (!item) {
    if (m_lnum) { *m_lnum = 0; }
    if (m_startcol) { *m_startcol = 0; }
    if (m_end_lnum) { *m_end_lnum = 0; }
    if (m_end_col) { *m_end_col = 0; }
    if (h_start_lnum) { *h_start_lnum = 0; }
    if (h_start_col) { *h_start_col = 0; }
    if (h_end_lnum) { *h_end_lnum = 0; }
    if (h_end_col) { *h_end_col = 0; }
    if (eoe_lnum) { *eoe_lnum = 0; }
    if (eoe_col) { *eoe_col = 0; }
    return;
  }
  if (m_lnum) { *m_lnum = item->si_m_lnum; }
  if (m_startcol) { *m_startcol = item->si_m_startcol; }
  if (m_end_lnum) { *m_end_lnum = (int)item->si_m_endpos.lnum; }
  if (m_end_col) { *m_end_col = (int)item->si_m_endpos.col; }
  if (h_start_lnum) { *h_start_lnum = (int)item->si_h_startpos.lnum; }
  if (h_start_col) { *h_start_col = (int)item->si_h_startpos.col; }
  if (h_end_lnum) { *h_end_lnum = (int)item->si_h_endpos.lnum; }
  if (h_end_col) { *h_end_col = (int)item->si_h_endpos.col; }
  if (eoe_lnum) { *eoe_lnum = (int)item->si_eoe_pos.lnum; }
  if (eoe_col) { *eoe_col = (int)item->si_eoe_pos.col; }
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

void nvim_syn_set_next_match_idx(int idx) { next_match_idx = idx; }
void nvim_syn_set_next_match_col(int col) { next_match_col = col; }
void nvim_syn_check_state_ends(void) { rs_check_state_ends(); }

void nvim_syn_pop_current_state(void) { rs_syn_pop_current_state(); }
void nvim_syn_push_current_state(int idx) { rs_syn_push_current_state(idx); }

char nvim_syn_getcurline_at_col(void) { return rs_syn_getcurline()[current_col]; }

void nvim_syn_set_current_finished(int finished) { current_finished = finished ? true : false; }

int nvim_syn_id2attr_wrapper(int syn_id) { return syn_id2attr(syn_id); }

void nvim_syn_call_syn_update_ends(int syncing) { rs_syn_update_ends(syncing ? 1 : 0); }

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
  return rs_stateitem_prev_if_trans_cont(item);
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
int nvim_syn_has_keywords(void) { return syn_block != NULL && syn_block->b_keywtab.ht_used > 0 ? 1 : 0; }
int nvim_syn_has_keywords_ic(void) { return syn_block != NULL && syn_block->b_keywtab_ic.ht_used > 0 ? 1 : 0; }

keyentry_T *nvim_syn_keyword_find(char *keyword, int use_ic)
{
  return rs_syn_keyword_find(keyword, use_ic);
}

char *nvim_syn_getcurline(void) { return rs_syn_getcurline(); }

void nvim_syn_save_chartab(char *buf) { save_chartab(buf); }

void nvim_syn_restore_chartab(char *buf) { restore_chartab(buf); }

void nvim_syn_keyword_foldcase(char *src, int srclen, char *dst, int dstlen) { str_foldcase(src, srclen, dst, dstlen); }

int nvim_syn_utfc_ptr2len(char *p) { return utfc_ptr2len(p); }

void *nvim_syn_get_buf(void) { return syn_buf; }
void nvim_syn_set_syn_buf(void *buf) { syn_buf = (buf_T *)buf; }

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

int nvim_syn_incr_next_seqnr(void) { return next_seqnr++; }

/// Bulk getter for all 5 next_match position fields.
void nvim_syn_get_next_match_positions(int *h_start_lnum, int *h_start_col,
                                        int *m_end_lnum, int *m_end_col,
                                        int *h_end_lnum, int *h_end_col,
                                        int *eos_lnum, int *eos_col,
                                        int *eoe_lnum, int *eoe_col)
{
  *h_start_lnum = next_match_h_startpos.lnum;
  *h_start_col  = next_match_h_startpos.col;
  *m_end_lnum   = next_match_m_endpos.lnum;
  *m_end_col    = next_match_m_endpos.col;
  *h_end_lnum   = next_match_h_endpos.lnum;
  *h_end_col    = next_match_h_endpos.col;
  *eos_lnum     = next_match_eos_pos.lnum;
  *eos_col      = next_match_eos_pos.col;
  *eoe_lnum     = next_match_eoe_pos.lnum;
  *eoe_col      = next_match_eoe_pos.col;
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

/// Bulk setter for stateitem_T position fields.
/// Pass INT_MIN for any field that should not be modified.
void nvim_stateitem_set_positions(stateitem_T *item,
    int m_lnum, int m_startcol,
    int m_end_lnum, int m_end_col,
    int h_start_lnum, int h_start_col,
    int h_end_lnum, int h_end_col,
    int eoe_lnum, int eoe_col)
{
  if (!item) {
    return;
  }
  if (m_lnum != INT_MIN) { item->si_m_lnum = m_lnum; }
  if (m_startcol != INT_MIN) { item->si_m_startcol = m_startcol; }
  if (m_end_lnum != INT_MIN) { item->si_m_endpos.lnum = (linenr_T)m_end_lnum; }
  if (m_end_col != INT_MIN) { item->si_m_endpos.col = (colnr_T)m_end_col; }
  if (h_start_lnum != INT_MIN) { item->si_h_startpos.lnum = (linenr_T)h_start_lnum; }
  if (h_start_col != INT_MIN) { item->si_h_startpos.col = (colnr_T)h_start_col; }
  if (h_end_lnum != INT_MIN) { item->si_h_endpos.lnum = (linenr_T)h_end_lnum; }
  if (h_end_col != INT_MIN) { item->si_h_endpos.col = (colnr_T)h_end_col; }
  if (eoe_lnum != INT_MIN) { item->si_eoe_pos.lnum = (linenr_T)eoe_lnum; }
  if (eoe_col != INT_MIN) { item->si_eoe_pos.col = (colnr_T)eoe_col; }
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

void nvim_syn_start_line(void) { rs_syn_start_line(); }

int nvim_syn_finish_line(int syncing) { return rs_syn_finish_line(syncing); }

void nvim_syn_update_ends(int startofline) { rs_syn_update_ends(startofline); }

int nvim_syn_get_current_line_id(void) { return (int)current_line_id; }

void nvim_syn_incr_current_line_id(void) { current_line_id++; }

void *nvim_syn_get_syn_block(void) { return syn_block; }
void nvim_syn_set_syn_block(void *block) { syn_block = (synblock_T *)block; }
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
void nvim_syn_stack_alloc(void) { syn_stack_alloc(); }

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

void nvim_syn_stack_free_all(synblock_T *block) { rs_syn_stack_free_all(block); }
void nvim_syn_stack_apply_changes(buf_T *buf) { rs_syn_stack_apply_changes(buf); }

int nvim_buf_get_mod_top(buf_T *buf) { return (int)buf->b_mod_top; }
int nvim_buf_get_mod_bot(buf_T *buf) { return (int)buf->b_mod_bot; }
int nvim_buf_get_mod_xlines(buf_T *buf) { return (int)buf->b_mod_xlines; }

void nvim_synstate_set_lnum(synstate_T *state, int lnum) { state->sst_lnum = lnum; }

int nvim_synstate_next_list_eq(synstate_T *a, synstate_T *b) { return a->sst_next_list == b->sst_next_list; }


int nvim_synblock_has_containedin(synblock_T *block) { return block->b_syn_containedin ? 1 : 0; }

int nvim_synpat_get_inc_tag(synpat_T *pat) { return pat ? pat->sp_syn.inc_tag : 0; }

int nvim_synblock_is_spell_cluster(synblock_T *block, int id) { return id == block->b_spell_cluster_id; }
int nvim_synblock_is_nospell_cluster(synblock_T *block, int id) { return id == block->b_nospell_cluster_id; }

void nvim_syn_set_current_col(int col) { current_col = col; }

int nvim_buf_get_synmaxcol(buf_T *buf) { return (int)buf->b_p_smc; }

/// Validate current state if needed
void nvim_syn_ensure_current_state_valid(void)
{
  if (INVALID_STATE(&current_state)) {
    rs_validate_current_state();
  }
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

win_T *nvim_syn_get_win(void) { return syn_win; }

int nvim_syn_cur_foldlevel(void) { return rs_syn_count_fold_items(); }

char **nvim_syn_get_cmdlinep(void) { return syn_cmdlinep; }

int nvim_syn_get_include_link(void) { return include_link; }
int nvim_syn_get_include_default(void) { return include_default; }
int nvim_syn_get_include_none(void) { return include_none; }
int nvim_syn_get_running_inc_tag(void) { return running_syn_inc_tag; }
void nvim_syn_set_running_inc_tag(int tag) { running_syn_inc_tag = tag; }
/// Set tick on synstate
void nvim_synstate_set_tick(synstate_T *state, int tick)
{
  if (state) {
    state->sst_tick = tick;
  }
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

int nvim_syn_getcurline_byte_at(int col) { return (unsigned char)rs_syn_getcurline()[col]; }

void nvim_syn_set_current_attr(int attr) { current_attr = attr; }
void nvim_syn_set_current_sub_char(int c) { current_sub_char = c; }

/// Set all current_* fields from a stateitem (replaces 6 individual FFI calls).
void nvim_syn_set_current_from_stateitem(stateitem_T *item)
{
  current_attr = item->si_attr;
  current_id = item->si_id;
  current_trans_id = item->si_trans_id;
  current_flags = item->si_flags;
  current_seqnr = item->si_seqnr;
  current_sub_char = item->si_cchar;
}

/// Zero all current_* highlight fields (replaces 5 individual FFI calls).
void nvim_syn_zero_current(void)
{
  current_attr = 0;
  current_id = 0;
  current_trans_id = 0;
  current_flags = 0;
  current_seqnr = 0;
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

int nvim_syn_get_pattern_line_id(int idx) { return SYN_ITEMS(syn_block)[idx].sp_line_id; }
void nvim_syn_set_pattern_line_id(int idx, int line_id) { SYN_ITEMS(syn_block)[idx].sp_line_id = line_id; }
int nvim_syn_get_pattern_startcol(int idx) { return SYN_ITEMS(syn_block)[idx].sp_startcol; }
void nvim_syn_set_pattern_startcol(int idx, int col) { SYN_ITEMS(syn_block)[idx].sp_startcol = col; }
int nvim_syn_get_pattern_lc_off(int idx) { return SYN_ITEMS(syn_block)[idx].sp_offsets[SPO_LC_OFF]; }
int nvim_syn_get_pattern_syncing(int idx) { return SYN_ITEMS(syn_block)[idx].sp_syncing; }
int nvim_syn_get_pattern_display(int idx) { return (SYN_ITEMS(syn_block)[idx].sp_flags & HL_DISPLAY) != 0; }
int nvim_syn_get_pattern_ga_len(void) { return syn_block->b_syn_patterns.ga_len; }
int nvim_syn_has_containedin(void) { return syn_block->b_syn_containedin; }

/// Check in_id_list for a pattern by index against the current_next_list or
/// cur_si cont_list.
/// mode: 0 = check current_next_list (cur_si ignored)
///       1 = check cur_si->si_cont_list
///       2 = check HL_CONTAINED flag

/// Check in_id_list with a specific sp_syn (for spell checking)
int nvim_syn_in_id_list_spell(stateitem_T *sip, int16_t *list, int id)
{
  return rs_syn_in_id_list(sip, list, id, 0, NULL, 0);
}

int nvim_syn_get_syn_spell(void) { return syn_block->b_syn_spell; }

int nvim_syn_vim_iswordp_buf(char *p) { return vim_iswordp_buf(p, syn_buf); }

int nvim_syn_utf_head_off(char *base, char *p) { return utf_head_off(base, p); }

void nvim_syn_set_next_match_state(
    int idx, int col,
    int m_endpos_lnum, int m_endpos_col,
    int h_endpos_lnum, int h_endpos_col,
    int h_startpos_lnum, int h_startpos_col,
    int flags, int eos_pos_lnum, int eos_pos_col,
    int eoe_pos_lnum, int eoe_pos_col,
    int end_idx, reg_extmatch_T *extmatch)
{
  next_match_idx = idx;
  next_match_col = col;
  next_match_m_endpos.lnum = (linenr_T)m_endpos_lnum;
  next_match_m_endpos.col = (colnr_T)m_endpos_col;
  next_match_h_endpos.lnum = (linenr_T)h_endpos_lnum;
  next_match_h_endpos.col = (colnr_T)h_endpos_col;
  next_match_h_startpos.lnum = (linenr_T)h_startpos_lnum;
  next_match_h_startpos.col = (colnr_T)h_startpos_col;
  next_match_flags = flags;
  next_match_eos_pos.lnum = (linenr_T)eos_pos_lnum;
  next_match_eos_pos.col = (colnr_T)eos_pos_col;
  next_match_eoe_pos.lnum = (linenr_T)eoe_pos_lnum;
  next_match_eoe_pos.col = (colnr_T)eoe_pos_col;
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

/// Thin C helper for C-comment sync: saves/restores curwin, curbuf, and cursor;
/// does backslash-continuation skip; sets current_lnum; calls find_start_comment.
/// Returns the adjusted start_lnum in *out_start_lnum.
/// Returns 1 if find_start_comment found a comment, 0 otherwise.
int nvim_syn_ccomment_find(win_T *wp, int start_lnum, int *out_start_lnum)
{
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
  current_lnum = (linenr_T)start_lnum;

  pos_T cursor_save = wp->w_cursor;
  wp->w_cursor.lnum = (linenr_T)start_lnum;
  wp->w_cursor.col = 0;

  int found = (find_start_comment((int)syn_block->b_syn_sync_maxlines) != NULL) ? 1 : 0;

  wp->w_cursor = cursor_save;
  curwin = curwin_save;
  curbuf = curbuf_save;

  *out_start_lnum = start_lnum;
  return found;
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
int nvim_syn_name2id_wrapper(const char *name) { return syn_name2id(name); }

int nvim_syn_check_group_wrapper(const char *name, int len) { return syn_check_group(name, (size_t)len); }

int nvim_syn_highlight_num_groups(void) { return highlight_num_groups(); }
char *nvim_syn_highlight_group_name(int idx) { return highlight_group_name(idx); }

void *nvim_syn_vim_regcomp(char *pat, int flags) { return vim_regcomp(pat, flags); }


void nvim_syn_vim_regfree(void *regprog) { vim_regfree(regprog); }

int nvim_syn_foldmethod_is_syntax_curwin(void) { return rs_foldmethodIsSyntax(curwin); }

void nvim_syn_fold_update_all_curwin(void) { rs_foldUpdateAll(curwin); }


int nvim_syn_utf_ptr2char(const char *p) { return utf_ptr2char(p); }

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

char *nvim_syn_vim_strnsave_up(const char *str, int len) { return vim_strnsave_up(str, (size_t)len); }

void nvim_syn_set_nextcmd(exarg_T *eap, char *rest) { eap->nextcmd = check_nextcmd(rest); }
char *nvim_syn_get_eap_arg(const exarg_T *eap) { return eap->arg; }
int nvim_syn_get_eap_skip(const exarg_T *eap) { return eap->skip; }

_Static_assert(SF_CCOMMENT == 0x01, "SF_CCOMMENT");
_Static_assert(SF_MATCH == 0x02, "SF_MATCH");
_Static_assert(NONE_IDX == -2, "NONE_IDX");
_Static_assert(SYNID_ALLBUT == MAX_HL_ID, "SYNID_ALLBUT");
_Static_assert(SYNID_TOP == 21000, "SYNID_TOP");
_Static_assert(SYNID_CONTAINED == 22000, "SYNID_CONTAINED");
_Static_assert(HL_FOLD == 0x2000, "HL_FOLD");
_Static_assert(HL_EXCLUDENL == 0x800, "HL_EXCLUDENL");
_Static_assert(HL_INCLUDED_TOPLEVEL == 0x80000, "HL_INCLUDED_TOPLEVEL");

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
void nvim_synpat_set_offset(synpat_T *pat, int idx, int val) { pat->sp_offsets[idx] = val; }
int nvim_synpat_get_offset(synpat_T *pat, int idx) { return pat->sp_offsets[idx]; }
void nvim_synpat_clear_time(synpat_T *pat) { rs_syn_clear_time(&pat->sp_time); }

// =============================================================================
// Phase 2 accessors: syn_cmd_match migration
// =============================================================================

int nvim_syn_vim_regcomp_had_eol(void) { return vim_regcomp_had_eol(); }

// =============================================================================
// Phase 7 (pass 7) accessors: pattern_store migration
// =============================================================================

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

// =============================================================================
// Phase 4 accessors: syn_cmd_cluster migration
// =============================================================================

/// Combine a cluster's ID list with a new list, consuming both.
/// Calls syn_combine_list on the cluster's list and clstr_list.
void nvim_syn_combine_cluster_list(int scl_id, int16_t **clstr_list, int list_op)
{
  rs_syn_combine_list(&SYN_CLSTR(curwin->w_s)[scl_id].scl_list, clstr_list, list_op);
}

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

/// Set b_syn_linecont_pat (takes ownership of pat).
void nvim_synblock_set_linecont_pat(synblock_T *block, char *pat)
{
  block->b_syn_linecont_pat = pat;
}

/// Get b_syn_linecont_pat.
char *nvim_synblock_get_linecont_pat(synblock_T *block) { return block->b_syn_linecont_pat; }

/// Set b_syn_linecont_ic.
void nvim_synblock_set_linecont_ic(synblock_T *block, int ic) { block->b_syn_linecont_ic = ic; }

/// Set b_syn_linecont_prog.
void nvim_synblock_set_linecont_prog2(synblock_T *block, void *prog)
{
  block->b_syn_linecont_prog = (regprog_T *)prog;
}

/// XFREE_CLEAR b_syn_linecont_pat.
void nvim_syn_clear_linecont_pat(synblock_T *block) { XFREE_CLEAR(block->b_syn_linecont_pat); }

/// Get pointer to b_syn_linecont_time for a synblock.
void *nvim_synblock_get_linecont_time_ptr(synblock_T *block)
{
  return (void *)&block->b_syn_linecont_time;
}

/// Compile regexp with empty p_cpo (avoid 'l' flag side-effect).
/// Returns the compiled regprog, or NULL on failure.
void *nvim_syn_vim_regcomp_empty_cpo(char *pat, int flags)
{
  char *cpo_save = p_cpo;
  p_cpo = empty_string_option;
  void *prog = vim_regcomp(pat, flags);
  p_cpo = cpo_save;
  return prog;
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

// =============================================================================
// Phase 2 accessors: syn_cmd_clear migration
// =============================================================================

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
/// Thin wrapper: logic is in rs_syn_cmd_onoff (commands.rs).
void nvim_syn_do_onoff(exarg_T *eap, const char *name)
{
  rs_syn_do_onoff_impl(eap, name);
}

/// Wrap do_cmdline_cmd for Rust callers (Phase 11).
void nvim_syn_do_cmdline_cmd(const char *cmd)
{
  do_cmdline_cmd(cmd);
}

/// Set did_syntax_onoff flag (Phase 11).
void nvim_syn_set_did_syntax_onoff(int v)
{
  did_syntax_onoff = (bool)v;
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
  return rs_syn_ownsyntax_init();
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
/// Thin wrapper: logic is in rs_syn_clear_keyword (clearing.rs).
void nvim_syn_clear_keyword_in_ht(int id, hashtab_T *ht)
{
  rs_syn_clear_keyword(id, ht);
}

// =============================================================================
// Phase 15 bulk clearing accessors: syntax_clear / reset_synblock / syntax_sync_clear
// =============================================================================

/// Full clear of a synblock: keytabs, patterns, clusters, cluster_ids,
/// sync_flags, linecont, syn_isk, and all scalar flags.
/// Replaces the 8 individual clearing functions called by rs_syntax_clear.
void nvim_synblock_full_clear(synblock_T *block)
{
  // Clear keyword tables (call rs_clear_keywtab directly to avoid circular call)
  rs_clear_keywtab(&block->b_keywtab);
  rs_clear_keywtab(&block->b_keywtab_ic);
  // Free pattern and cluster arrays
  ga_clear(&block->b_syn_patterns);
  ga_clear(&block->b_syn_clusters);
  // Reset cluster IDs
  block->b_spell_cluster_id = 0;
  block->b_nospell_cluster_id = 0;
  // Reset sync flags
  block->b_syn_sync_flags = 0;
  block->b_syn_sync_minlines = 0;
  block->b_syn_sync_maxlines = 0;
  block->b_syn_sync_linebreaks = 0;
  block->b_syn_folditems = 0;
  // Free linecont
  vim_regfree(block->b_syn_linecont_prog);
  block->b_syn_linecont_prog = NULL;
  XFREE_CLEAR(block->b_syn_linecont_pat);
  // Clear iskeyword option
  clear_string_option(&block->b_syn_isk);
  // Reset scalar flags
  block->b_syn_error = false;
  block->b_syn_slow = false;
  block->b_syn_ic = false;
  block->b_syn_foldlevel = SYNFLD_START;
  block->b_syn_spell = SYNSPL_DEFAULT;
  block->b_syn_containedin = false;
  block->b_syn_conceal = false;
}

/// Sync-only clear of a synblock: reset sync_flags, linecont, syn_isk.
/// Replaces the 3 individual calls in rs_syntax_sync_clear.
void nvim_synblock_sync_clear(synblock_T *block)
{
  // Reset sync flags
  block->b_syn_sync_flags = 0;
  block->b_syn_sync_minlines = 0;
  block->b_syn_sync_maxlines = 0;
  block->b_syn_sync_linebreaks = 0;
  block->b_syn_folditems = 0;
  // Free linecont
  vim_regfree(block->b_syn_linecont_prog);
  block->b_syn_linecont_prog = NULL;
  XFREE_CLEAR(block->b_syn_linecont_pat);
  // Clear iskeyword option
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

// =============================================================================
// Phase 5 pass 5 Phase 3 accessors: syn_clear_keyword / clear_keywtab / invalidate_current_state
// =============================================================================

/// Clear a whole keyword hashtable (free entries, then hash_clear + hash_init).
/// Thin wrapper: logic is in rs_clear_keywtab (clearing.rs).
void nvim_syn_clear_keywtab_ht(hashtab_T *ht)
{
  rs_clear_keywtab(ht);
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
  return rs_synblock_cluster_append();
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
  rs_syn_hash_insert_keyword(name_ic, name_iclen, id, inc_tag, flags,
                              conceal_char, cont_in_list_copy, next_list_copy,
                              use_ic);
}

// =============================================================================
// Phase 8: State stack cache management accessors (for Rust migration)
// =============================================================================

/// Get b_sst_array pointer (raw array base) for a synblock.
synstate_T *nvim_synblock_get_sst_array_ptr(synblock_T *block)
{
  return block ? block->b_sst_array : NULL;
}

/// Set b_sst_array and b_sst_len for a synblock (takes ownership of ptr).
void nvim_synblock_set_sst_array(synblock_T *block, synstate_T *ptr, int len)
{
  if (!block) return;
  block->b_sst_array = ptr;
  block->b_sst_len = len;
}

/// Set b_sst_first for a synblock.
void nvim_synblock_set_sst_first(synblock_T *block, synstate_T *ptr)
{
  if (block) block->b_sst_first = ptr;
}

/// Set b_sst_firstfree for a synblock.
void nvim_synblock_set_sst_firstfree(synblock_T *block, synstate_T *ptr)
{
  if (block) block->b_sst_firstfree = ptr;
}

/// Set b_sst_freecount for a synblock.
void nvim_synblock_set_sst_freecount(synblock_T *block, int count)
{
  if (block) block->b_sst_freecount = count;
}

/// Set sst_next pointer on a synstate entry.
void nvim_synstate_set_next(synstate_T *state, synstate_T *next)
{
  if (state) state->sst_next = next;
}

/// Set sst_stacksize on a synstate entry.
void nvim_synstate_set_stacksize(synstate_T *state, int size)
{
  if (state) state->sst_stacksize = size;
}

/// Call rs_clear_syn_state on a synstate entry (releases extmatch pointers).
void nvim_syn_do_clear_syn_state(synstate_T *p)
{
  rs_clear_syn_state(p);
}

/// Allocate a new zeroed synstate array of given length.
/// Returns the pointer; caller owns the memory and must free with
/// nvim_syn_free_sst_array().
synstate_T *nvim_syn_xcalloc_synstate_array(int len)
{
  if (len <= 0) return NULL;
  return xcalloc((size_t)len, sizeof(synstate_T));
}

/// Free a synstate array previously allocated with nvim_syn_xcalloc_synstate_array.
void nvim_syn_free_sst_array(synstate_T *ptr)
{
  xfree(ptr);
}

/// Get a pointer to synstate entry at given index in array.
synstate_T *nvim_syn_sst_array_at(synstate_T *array, int idx)
{
  return array + idx;
}

/// Copy one synstate_T entry: *dst = *src.
void nvim_syn_sst_copy_entry(synstate_T *dst, const synstate_T *src)
{
  *dst = *src;
}

/// Get b_sst_lasttick for a synblock.
int nvim_synblock_get_sst_lasttick(synblock_T *block)
{
  return block ? (int)block->b_sst_lasttick : 0;
}

/// Get b_ml.ml_line_count from syn_buf.
int nvim_syn_buf_get_ml_line_count(void)
{
  return syn_buf ? (int)syn_buf->b_ml.ml_line_count : 0;
}

/// Get a pointer to the buf's b_s (main synblock).
synblock_T *nvim_buf_get_b_s(buf_T *buf)
{
  return buf ? &buf->b_s : NULL;
}

/// Do the FOR_ALL_WINDOWS_IN_TAB loop for syn_stack_free_all:
/// free block, then update folds for windows that use this block.
/// This is called by rs_syn_stack_free_all after the block has been freed by Rust.
void nvim_syn_fold_update_for_block(synblock_T *block)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_s == block && rs_foldmethodIsSyntax(wp)) {
      rs_foldUpdateAll(wp);
    }
  }
}

/// Do the FOR_ALL_WINDOWS_IN_TAB part of syn_stack_apply_changes:
/// apply changes to all window-local synblocks that differ from buf->b_s.
/// rs_syn_stack_apply_changes_block is called for each such window's block.
void nvim_syn_apply_changes_for_windows(buf_T *buf)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if ((wp->w_buffer == buf) && (wp->w_s != &buf->b_s)) {
      rs_syn_stack_apply_changes_block(wp->w_s, buf);
    }
  }
}

void nvim_syn_do_stack_realloc(int len) { rs_syn_do_stack_realloc(len); }

// =============================================================================
// Phase 8: Line initialization accessors (for Rust migration)
// =============================================================================

/// Return true if syn_block->b_syn_isk == empty_string_option.
int nvim_syn_block_isk_is_empty(void)
{
  return (syn_block && syn_block->b_syn_isk == empty_string_option) ? 1 : 0;
}

/// Copy 32 bytes from syn_buf->b_chartab to dst.
void nvim_syn_buf_chartab_get(char *dst)
{
  if (syn_buf) memmove(dst, syn_buf->b_chartab, 32);
}

/// Copy 32 bytes from src to syn_buf->b_chartab.
void nvim_syn_buf_chartab_set(const char *src)
{
  if (syn_buf) memmove(syn_buf->b_chartab, src, 32);
}

/// Copy 32 bytes from syn_win->w_s->b_syn_chartab to dst.
void nvim_syn_win_chartab_get(char *dst)
{
  if (syn_win) memmove(dst, syn_win->w_s->b_syn_chartab, 32);
}

/// Return true if syn_win->w_s->b_syn_isk != empty_string_option.
int nvim_syn_win_isk_not_empty(void)
{
  return (syn_win && syn_win->w_s->b_syn_isk != empty_string_option) ? 1 : 0;
}

/// Get syn_block->b_syn_linecont_prog (returns NULL if not set).
void *nvim_syn_block_get_linecont_prog(void)
{
  return (syn_block) ? syn_block->b_syn_linecont_prog : NULL;
}

/// Get syn_block->b_syn_linecont_ic.
int nvim_syn_block_get_linecont_ic(void)
{
  return syn_block ? syn_block->b_syn_linecont_ic : 0;
}

/// Get address of syn_block->b_syn_linecont_time (for syn_regexec timing).
void *nvim_syn_block_get_linecont_time_ptr(void)
{
  return syn_block ? (void *)&syn_block->b_syn_linecont_time : NULL;
}

/// Set syn_block->b_syn_linecont_prog.
void nvim_syn_block_set_linecont_prog(void *prog)
{
  if (syn_block) syn_block->b_syn_linecont_prog = (regprog_T *)prog;
}

/// Set next_seqnr to 1.
void nvim_syn_reset_next_seqnr(void) { next_seqnr = 1; }

/// Call validate_current_state() to set itemsize/growsize.
/// Direct implementation to avoid circular call.
void nvim_syn_do_validate_current_state(void)
{
  current_state.ga_itemsize = sizeof(stateitem_T);
  ga_set_growsize(&current_state, 3);
}

/// Return ml_get_buf(syn_buf, current_lnum).
/// Direct implementation to avoid circular call through syn_getcurline.
char *nvim_syn_do_getcurline(void) { return ml_get_buf(syn_buf, current_lnum); }

/// Return (int)ml_get_buf_len(syn_buf, current_lnum).
int nvim_syn_do_getcurline_len(void) { return (int)ml_get_buf_len(syn_buf, current_lnum); }

/// Zero out a syn_time_T struct.
void nvim_syn_do_clear_time(syn_time_T *st)
{
  // Direct implementation to avoid circular call with syn_clear_time -> rs_syn_clear_time -> here
  st->total = profile_zero();
  st->slowest = profile_zero();
  st->count = 0;
  st->match = 0;
}

/// Get CUR_STATE(i).si_idx for a given index.
int nvim_cur_state_get_si_idx(int i)
{
  if (i < 0 || i >= current_state.ga_len) return -3;
  return CUR_STATE(i).si_idx;
}

/// Get SYN_ITEMS(syn_block)[idx].sp_type for a given pattern index.
int nvim_syn_get_sptype_at(int idx)
{
  if (!syn_block || idx < 0 || idx >= syn_block->b_syn_patterns.ga_len) return -1;
  return (int)SYN_ITEMS(syn_block)[idx].sp_type;
}

/// Get CUR_STATE(i).si_m_endpos.lnum.
int nvim_cur_state_get_m_endpos_lnum(int i)
{
  if (i < 0 || i >= current_state.ga_len) return 0;
  return (int)CUR_STATE(i).si_m_endpos.lnum;
}

/// Set HL_MATCHCONT flag on CUR_STATE(i).si_flags and clear m_endpos.
void nvim_cur_state_set_matchcont(int i)
{
  rs_cur_state_set_matchcont(i);
}

/// Get CUR_STATE(i).si_flags.
int nvim_cur_state_get_si_flags(int i)
{
  if (i < 0 || i >= current_state.ga_len) return 0;
  return CUR_STATE(i).si_flags;
}

/// Set CUR_STATE(i).si_h_startpos to (current_lnum, 0).
void nvim_cur_state_set_h_startpos_cur(int i)
{
  if (i < 0 || i >= current_state.ga_len) return;
  CUR_STATE(i).si_h_startpos.col = 0;
  CUR_STATE(i).si_h_startpos.lnum = current_lnum;
}

// =============================================================================
// Phase 9: New C accessors for state_entry.rs migration
// =============================================================================

/// Set sst_next_flags on a synstate entry.
void nvim_synstate_set_sst_next_flags(synstate_T *state, int flags)
{
  if (state) state->sst_next_flags = (int16_t)flags;
}

/// Set sst_next_list on a synstate entry.
void nvim_synstate_set_sst_next_list(synstate_T *state, int16_t *list)
{
  if (state) state->sst_next_list = list;
}

/// Initialize and fill the sst_union bufstate array from current_state.
/// Thin wrapper: logic is in rs_syn_store_bufstates (state_entry.rs).
void nvim_syn_store_bufstates(synstate_T *sp)
{
  rs_syn_store_bufstates(sp);
}

/// Set sst_tick to current display_tick on a synstate entry.
void nvim_synstate_set_tick_to_display(synstate_T *state)
{
  if (state) state->sst_tick = display_tick;
}

// =============================================================================
// Phase 9.2: New C accessors for state_ops.rs migration (Phase 2)
// =============================================================================

/// Append a new zeroed stateitem to current_state and return it.
/// Combines GA_APPEND_VIA_PTR + CLEAR_POINTER.
stateitem_T *nvim_syn_append_new_stateitem(void)
{
  stateitem_T *p = GA_APPEND_VIA_PTR(stateitem_T, &current_state);
  CLEAR_POINTER(p);
  return p;
}

// =============================================================================
// Phase 11: New C accessors for clear_syn_state / store_bufstates migration
// =============================================================================

/// Call ga_clear on state->sst_union.sst_ga (for use after unreffing all extmatches
/// in the growarray path of clear_syn_state).
void nvim_synstate_ga_clear(synstate_T *state)
{
  if (state) ga_clear(&state->sst_union.sst_ga);
}

/// Fill one bufstate slot in sp->sst_union from CUR_STATE(i).
/// Handles both fixed-stack and growarray paths.
/// Must be called only after nvim_syn_store_bufstates_init() sets up the bp pointer.
/// Combined accessor: copies idx/flags/seqnr/cchar and calls ref_extmatch.
/// This avoids exposing CUR_STATE and ref_extmatch macros to Rust.
void nvim_synstate_fill_bufstate_from_curstate(synstate_T *sp, int i)
{
  bufstate_T *bp;
  if (sp->sst_stacksize > SST_FIX_STATES) {
    bp = SYN_STATE_P(&(sp->sst_union.sst_ga));
  } else {
    bp = sp->sst_union.sst_stack;
  }
  bp[i].bs_idx = CUR_STATE(i).si_idx;
  bp[i].bs_flags = CUR_STATE(i).si_flags;
  bp[i].bs_seqnr = CUR_STATE(i).si_seqnr;
  bp[i].bs_cchar = CUR_STATE(i).si_cchar;
  bp[i].bs_extmatch = ref_extmatch(CUR_STATE(i).si_extmatch);
}

/// Initialize the growarray path of sst_union for store_bufstates.
/// Only call when sp->sst_stacksize > SST_FIX_STATES.
void nvim_synstate_ga_init_for_store(synstate_T *sp)
{
  if (sp == NULL) return;
  ga_init(&sp->sst_union.sst_ga, (int)sizeof(bufstate_T), 1);
  ga_grow(&sp->sst_union.sst_ga, sp->sst_stacksize);
  sp->sst_union.sst_ga.ga_len = sp->sst_stacksize;
}

// =============================================================================
// Phase 11: New C accessors for hashtab keyword operations (Phase 1)
// =============================================================================

/// Free a keyentry_T's owned lists and the entry itself.
/// Frees kp->next_list, kp->k_syn.cont_in_list, and kp.
void nvim_ke_free(keyentry_T *kp)
{
  if (!kp) return;
  xfree(kp->next_list);
  xfree(kp->k_syn.cont_in_list);
  xfree(kp);
}

/// Call hash_clear + hash_init on a hashtab (for clearing keyword tables).
void nvim_ht_clear_and_init(hashtab_T *ht)
{
  if (!ht) return;
  hash_clear(ht);
  hash_init(ht);
}

/// Lock a hashtab for iteration (wraps hash_lock).
void nvim_ht_lock(hashtab_T *ht) { if (ht) hash_lock(ht); }

/// Unlock a hashtab after iteration (wraps hash_unlock).
void nvim_ht_unlock(hashtab_T *ht) { if (ht) hash_unlock(ht); }

/// Set ke->ke_next to next (for chain relink during clear_keyword_in_ht).
void nvim_ke_set_next(keyentry_T *kp, keyentry_T *next)
{
  if (kp) kp->ke_next = next;
}

/// Call hash_remove on ht at array index idx (hi = &ht->ht_array[idx]).
void nvim_ht_remove_at(hashtab_T *ht, size_t idx)
{
  if (!ht || idx >= (ht->ht_mask + 1)) return;
  hash_remove(ht, &ht->ht_array[idx]);
}

/// Get hi_key pointer for the KE2HIKEY of kp (the key used for hash chain relink).
/// Returns KE2HIKEY(kp) = kp->keyword.
char *nvim_ke_get_hikey(keyentry_T *kp)
{
  return kp ? KE2HIKEY(kp) : NULL;
}

/// Set hi_key at array index idx in ht to the given key (hi->hi_key = key).
void nvim_ht_set_hikey_at(hashtab_T *ht, size_t idx, char *key)
{
  if (!ht || idx >= (ht->ht_mask + 1)) return;
  ht->ht_array[idx].hi_key = key;
}

/// Get keyentry_T k_syn.id field (already exposed as nvim_keyentry_get_syn_id).
/// Alias here for clarity in the Phase 1 iteration context.
int nvim_ke_get_syn_id_int(keyentry_T *kp)
{
  return kp ? (int)kp->k_syn.id : 0;
}

/// Find a keyword in hashtab: wraps hash_find + HI2KE.
/// Returns the keyentry_T * for the first match, or NULL if not found / empty.
keyentry_T *nvim_ht_find_ke(hashtab_T *ht, char *keyword)
{
  if (!ht || !keyword) return NULL;
  hashitem_T *hi = hash_find(ht, keyword);
  if (HASHITEM_EMPTY(hi)) return NULL;
  return HI2KE(hi);
}

/// Allocate a keyentry_T, fill all fields, and insert into the given hashtab.
/// This accessor owns the offsetof arithmetic that Rust cannot replicate.
/// Ownership of cont_in_list_copy and next_list_copy is transferred.
/// Sets curwin->w_s->b_syn_containedin if cont_in_list_copy is non-NULL.
void nvim_ke_alloc_and_insert(hashtab_T *ht, const char *name_ic, int name_iclen,
                               int id, int inc_tag, int flags, int conceal_char,
                               int16_t *cont_in_list_copy, int16_t *next_list_copy)
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
  hashitem_T *const hi = hash_lookup(ht, kp->keyword, (size_t)name_iclen, hash);
  if (HASHITEM_EMPTY(hi)) {
    kp->ke_next = NULL;
    hash_add_item(ht, hi, kp->keyword, hash);
  } else {
    kp->ke_next = HI2KE(hi);
    hi->hi_key = KE2HIKEY(kp);
  }
}

/// Get the keywtab or keywtab_ic pointer for curwin's synblock.
hashtab_T *nvim_curwin_get_keywtab(int use_ic)
{
  return use_ic ? &curwin->w_s->b_keywtab_ic : &curwin->w_s->b_keywtab;
}

// =============================================================================
// Phase 11: ownsyntax_init and cluster_append C accessors
// =============================================================================

/// Check if curwin shares the buffer's synblock (i.e. has not yet called ownsyntax).
/// Returns 1 if they share (curwin->w_s == &curwin->w_buffer->b_s), else 0.
int nvim_curwin_shares_buf_synblock(void)
{
  return curwin->w_s == &curwin->w_buffer->b_s ? 1 : 0;
}

/// Allocate a new zeroed synblock_T, initialise its hashtabs and spell options.
/// Does NOT assign it to curwin->w_s.
/// Returns pointer to the new block.
synblock_T *nvim_syn_alloc_new_synblock(void)
{
  synblock_T *block = xcalloc(1, sizeof(synblock_T));
  hash_init(&block->b_keywtab);
  hash_init(&block->b_keywtab_ic);
  clear_string_option(&block->b_p_spc);
  clear_string_option(&block->b_p_spf);
  clear_string_option(&block->b_p_spl);
  clear_string_option(&block->b_p_spo);
  clear_string_option(&block->b_syn_isk);
  return block;
}

/// Assign synblock to curwin->w_s and clear curwin->w_p_spell.
void nvim_curwin_set_synblock(synblock_T *block)
{
  curwin->w_s = block;
  curwin->w_p_spell = false;
}

/// Append a new zeroed cluster entry to curwin->w_s->b_syn_clusters.
/// Initialises the garray if needed. Returns the index of the new entry,
/// or -1 if MAX_CLUSTER_ID has been reached.
/// Emits E848 on overflow.
int nvim_synblock_cluster_append_inner(void)
{
  synblock_T *block = curwin->w_s;
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
