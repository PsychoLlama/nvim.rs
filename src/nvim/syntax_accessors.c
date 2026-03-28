// syntax_accessors.c: syntax highlighting - struct definitions, static state,
// helper functions, and FFI accessor functions for Rust interop

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

extern synstate_T *rs_syn_stack_find_entry(int lnum);
extern void rs_syn_stack_free_all(synblock_T *block);
extern void rs_syn_stack_alloc(void);
extern void rs_syn_stack_apply_changes_block(synblock_T *block, buf_T *buf);
extern void rs_syn_stack_apply_changes(buf_T *buf);
extern int rs_syn_match_linecont(int lnum);
extern void rs_save_chartab(char *chartab);
extern void rs_restore_chartab(const char *chartab);
extern void rs_validate_current_state(void);
extern char *rs_syn_getcurline(void);
extern void rs_syn_clear_time(syn_time_T *st);
extern void rs_update_si_attr(int idx);
extern void rs_syn_remove_pattern(synblock_T *block, int idx);
extern void rs_syntax_clear(synblock_T *block);
extern void rs_syntax_sync_clear(void);
extern void rs_clear_keywtab(hashtab_T *ht);
extern void rs_foldUpdateAll(win_T *win);
extern char *rs_get_syn_pattern(char *arg, synpat_T *ci);
extern int rs_syn_add_cluster(char *name);
extern void rs_syn_combine_list(int16_t **clstr1, int16_t **clstr2, int list_op);
extern int rs_syn_in_id_list(stateitem_T *cur_si, int16_t *list, int id, int inc_tag,
                              int16_t *cont_in_list, int flags);
extern void rs_synblock_full_clear(synblock_T *block);
extern void rs_synblock_sync_clear(synblock_T *block);

#define SPO_MS_OFF      0       // match  start offset
#define SPO_ME_OFF      1       // match  end   offset
#define SPO_HS_OFF      2       // highl. start offset
#define SPO_HE_OFF      3       // highl. end   offset
#define SPO_RS_OFF      4       // region start offset
#define SPO_RE_OFF      5       // region end   offset
#define SPO_LC_OFF      6       // leading context offset
#define SPO_COUNT       7

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

#define SPTYPE_MATCH    1       // match keyword with this group ID
#define SPTYPE_START    2       // match a regexp, start of item
#define SPTYPE_END      3       // match a regexp, end of item
#define SPTYPE_SKIP     4       // match a regexp, skip within item

#define SYN_ITEMS(buf)  ((synpat_T *)((buf)->b_syn_patterns.ga_data))
#define NONE_IDX        (-2)    // value of sp_sync_idx for "NONE"
#define SF_CCOMMENT     0x01    // sync on a C-style comment
#define SF_MATCH        0x02    // sync by matching a pattern
#define SYN_STATE_P(ssp)    ((bufstate_T *)((ssp)->ga_data))

extern int CURRENT_SUB_CHAR;
#define current_sub_char CURRENT_SUB_CHAR

#define SYN_CLSTR(buf)  ((syn_cluster_T *)((buf)->b_syn_clusters.ga_data))

#define SYNID_ALLBUT    MAX_HL_ID   // syntax group ID for contains=ALLBUT
#define SYNID_TOP       21000       // syntax group ID for contains=TOP
#define SYNID_CONTAINED 22000       // syntax group ID for contains=CONTAINED
#define SYNID_CLUSTER   23000       // first syntax group ID for clusters
#define MAX_CLUSTER_ID  (32767 - SYNID_CLUSTER)

static char **syn_cmdlinep;

extern int CURRENT_SYN_INC_TAG;
#define current_syn_inc_tag CURRENT_SYN_INC_TAG

static keyentry_T dumkey;
#define KE2HIKEY(kp)  ((kp)->keyword)
#define HIKEY2KE(p)   ((keyentry_T *)((p) - (dumkey.keyword - (char *)&dumkey)))
#define HI2KE(hi)      HIKEY2KE((hi)->hi_key)

#define KEYWORD_IDX     (-1)
#define ID_LIST_ALL     ((int16_t *)-1)

extern int NEXT_SEQNR;
#define next_seqnr NEXT_SEQNR

extern int NEXT_MATCH_COL;
extern int NEXT_MATCH_IDX;
extern int NEXT_MATCH_FLAGS;
extern int NEXT_MATCH_END_IDX;
extern reg_extmatch_T *NEXT_MATCH_EXTMATCH;
#define next_match_col       NEXT_MATCH_COL
#define next_match_idx       NEXT_MATCH_IDX
#define next_match_flags     NEXT_MATCH_FLAGS
#define next_match_end_idx   NEXT_MATCH_END_IDX
#define next_match_extmatch  NEXT_MATCH_EXTMATCH

static win_T *syn_win;                  // current window for highlighting
static buf_T *syn_buf;                  // current buffer for highlighting
static synblock_T *syn_block;              // current buffer for highlighting
static proftime_T *syn_tm;                 // timeout limit
extern int CURRENT_LNUM;
extern int CURRENT_COL;
#define current_lnum       ((linenr_T)CURRENT_LNUM)
#define current_col        ((colnr_T)CURRENT_COL)
extern garray_T CURRENT_STATE;
#define current_state CURRENT_STATE
extern int16_t *CURRENT_NEXT_LIST;
#define current_next_list CURRENT_NEXT_LIST

#define CUR_STATE(idx)  ((stateitem_T *)(current_state.ga_data))[idx]

extern int SYN_TIME_ON;
#define syn_time_on (SYN_TIME_ON != 0)

void syn_set_timeout(proftime_T *tm) { syn_tm = tm; }

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
synpat_T *nvim_synblock_get_pattern(synblock_T *block, int idx) { if (idx < 0 || idx >= block->b_syn_patterns.ga_len) return NULL; return &SYN_ITEMS(block)[idx]; }
syn_cluster_T *nvim_synblock_get_cluster(synblock_T *block, int idx) { if (idx < 0 || idx >= block->b_syn_clusters.ga_len) return NULL; return &SYN_CLSTR(block)[idx]; }
synstate_T *nvim_synstate_get_next(synstate_T *state) { return state->sst_next; }
int nvim_synstate_get_lnum(synstate_T *state) { return (int)state->sst_lnum; }
int nvim_synstate_get_stacksize(synstate_T *state) { return state->sst_stacksize; }
int nvim_synstate_get_next_flags(synstate_T *state) { return state->sst_next_flags; }
int nvim_synstate_get_tick(synstate_T *state) { return (int)state->sst_tick; }
int nvim_synstate_get_change_lnum(synstate_T *state) { return (int)state->sst_change_lnum; }
int nvim_syn_get_current_sub_char(void) { return current_sub_char; }
hashtab_T *nvim_synblock_get_keywtab(synblock_T *block) { return &block->b_keywtab; }
hashtab_T *nvim_synblock_get_keywtab_ic(synblock_T *block) { return &block->b_keywtab_ic; }
int nvim_synblock_has_keywords(synblock_T *block) { return block->b_keywtab.ht_used > 0; }
int nvim_synblock_has_keywords_ic(synblock_T *block) { return block->b_keywtab_ic.ht_used > 0; }
size_t nvim_synblock_keywtab_count(synblock_T *block) { return block->b_keywtab.ht_used; }
size_t nvim_synblock_keywtab_ic_count(synblock_T *block) { return block->b_keywtab_ic.ht_used; }
int16_t nvim_id_list_get(int16_t *list, int idx) { return list[idx]; }
synblock_T *nvim_syn_get_curwin_synblock(void) { return curwin->w_s; }
int nvim_syn_get_topgrp(void) { return curwin->w_s->b_syn_topgrp; }
void nvim_syn_set_topgrp(int topgrp) { curwin->w_s->b_syn_topgrp = topgrp; }
int16_t *nvim_synstate_get_next_list(synstate_T *state) { if (state == NULL) return NULL; return state->sst_next_list; }
bufstate_T *nvim_synstate_get_bufstate(synstate_T *state, int idx) { if (state == NULL || idx < 0 || idx >= state->sst_stacksize) return NULL; bufstate_T *bp = state->sst_stacksize > SST_FIX_STATES ? SYN_STATE_P(&state->sst_union.sst_ga) : state->sst_union.sst_stack; return &bp[idx]; }
void nvim_syn_update_si_attr(int idx) { if (idx >= 0 && idx < current_state.ga_len) rs_update_si_attr(idx); }
const char *nvim_extmatch_get_string(reg_extmatch_T *em, int subidx) { if (em == NULL || subidx < 0 || subidx >= NSUBEXP) return NULL; return (const char *)em->matches[subidx]; }
int nvim_syn_mb_strcmp_ic(int ic, const char *a, const char *b) { if (a == NULL || b == NULL) return a == b ? 0 : 1; return mb_strcmp_ic(ic, a, b); }
char nvim_syn_getcurline_at_col(void) { return rs_syn_getcurline()[current_col]; }
int nvim_syn_is_id_list_all(int16_t *list) { return list == ID_LIST_ALL ? 1 : 0; }
int16_t *nvim_syn_get_id_list_all(void) { return ID_LIST_ALL; }
int nvim_syn_has_keywords(void) { return syn_block != NULL && syn_block->b_keywtab.ht_used > 0 ? 1 : 0; }
int nvim_syn_has_keywords_ic(void) { return syn_block != NULL && syn_block->b_keywtab_ic.ht_used > 0 ? 1 : 0; }
void nvim_syn_keyword_foldcase(char *src, int srclen, char *dst, int dstlen) { str_foldcase(src, srclen, dst, dstlen); }
void *nvim_syn_get_buf(void) { return syn_buf; }
void nvim_syn_set_syn_buf(void *buf) { syn_buf = (buf_T *)buf; }
reg_extmatch_T *nvim_syn_ref_extmatch(reg_extmatch_T *em) { return ref_extmatch(em); }
void nvim_syn_unref_extmatch(reg_extmatch_T *em) { unref_extmatch(em); }
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
void nvim_synstate_set_change_lnum(synstate_T *p, int lnum) { if (p) p->sst_change_lnum = (linenr_T)lnum; }
void *nvim_win_get_synblock(void *wp) { return wp ? ((win_T *)wp)->w_s : NULL; }
void *nvim_syn_win_get_buffer_ptr(void *wp) { return wp ? ((win_T *)wp)->w_buffer : NULL; }
int nvim_win_get_foldnestmax(void *wp) { return wp ? (int)((win_T *)wp)->w_p_fdn : 0; }
int nvim_syn_buf_get_line_count(void *buf) { return buf ? (int)((buf_T *)buf)->b_ml.ml_line_count : 0; }
int nvim_syn_buf_get_changed_tick(void *buf) { return buf ? (int)buf_get_changedtick((buf_T *)buf) : 0; }
void nvim_syn_set_sst_lasttick(int tick) { if (syn_block) syn_block->b_sst_lasttick = (disptick_T)tick; }
int nvim_syn_get_display_tick(void) { return (int)display_tick; }
int nvim_syn_get_got_int(void) { return got_int; }
int nvim_syn_get_rows(void) { return (int)Rows; }
int nvim_buf_get_mod_top(buf_T *buf) { return (int)buf->b_mod_top; }
int nvim_buf_get_mod_bot(buf_T *buf) { return (int)buf->b_mod_bot; }
int nvim_buf_get_mod_xlines(buf_T *buf) { return (int)buf->b_mod_xlines; }
void nvim_synstate_set_lnum(synstate_T *state, int lnum) { state->sst_lnum = lnum; }
int nvim_synstate_next_list_eq(synstate_T *a, synstate_T *b) { return a->sst_next_list == b->sst_next_list; }
int nvim_synblock_has_containedin(synblock_T *block) { return block->b_syn_containedin ? 1 : 0; }
int nvim_synblock_is_spell_cluster(synblock_T *block, int id) { return id == block->b_spell_cluster_id; }
int nvim_synblock_is_nospell_cluster(synblock_T *block, int id) { return id == block->b_nospell_cluster_id; }
int nvim_buf_get_synmaxcol(buf_T *buf) { return (int)buf->b_p_smc; }
win_T *nvim_syn_get_win(void) { return syn_win; }
char **nvim_syn_get_cmdlinep(void) { return syn_cmdlinep; }
int nvim_syn_get_include_link(void) { return include_link; }
int nvim_syn_get_include_default(void) { return include_default; }
int nvim_syn_get_include_none(void) { return include_none; }
int nvim_syn_get_synblock_pattern_count(void) { if (syn_block == NULL) return 0; return syn_block->b_syn_patterns.ga_len; }
int nvim_syn_getcurline_len(void) { return (int)ml_get_buf_len(syn_buf, current_lnum); }
int nvim_syn_get_line_len(int lnum) { return (int)ml_get_buf_len(syn_buf, (linenr_T)lnum); }
int nvim_syn_get_buf_line_count(void) { return (int)syn_buf->b_ml.ml_line_count; }
void nvim_syn_set_extmatch_in(reg_extmatch_T *em) { unref_extmatch(re_extmatch_in); re_extmatch_in = ref_extmatch(em); }
void nvim_syn_clear_extmatch_in(void) { unref_extmatch(re_extmatch_in); re_extmatch_in = NULL; }
int nvim_syn_getcurline_byte_at(int col) { return (unsigned char)rs_syn_getcurline()[col]; }
reg_extmatch_T *nvim_syn_take_re_extmatch_out(void) { reg_extmatch_T *em = re_extmatch_out; re_extmatch_out = NULL; return em; }
void nvim_syn_clear_re_extmatch_out(void) { unref_extmatch(re_extmatch_out); re_extmatch_out = NULL; }
int nvim_syn_has_containedin(void) { return syn_block->b_syn_containedin; }
int nvim_syn_in_id_list_spell(stateitem_T *sip, int16_t *list, int id) { return rs_syn_in_id_list(sip, list, id, 0, NULL, 0); }
int nvim_syn_get_syn_spell(void) { return syn_block->b_syn_spell; }
int nvim_syn_vim_iswordp_buf(char *p) { return vim_iswordp_buf(p, syn_buf); }
char *nvim_syn_ml_get(linenr_T lnum) { return ml_get_buf(syn_buf, lnum); }

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
  CURRENT_LNUM = (int)start_lnum;

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

int nvim_syn_get_b_syn_conceal(void) { return curwin->w_s->b_syn_conceal; }
void nvim_syn_xmemcpyz(char *dst, const char *src, int len) { xmemcpyz(dst, src, (size_t)len); }
char *nvim_syn_strpbrk(const char *s, const char *chars) { return strpbrk(s, chars); }
void nvim_syn_semsg_1s(const char *fmt, const char *arg) { semsg(fmt, arg); }
int nvim_syn_ascii_iswhite_char(int c) { return ascii_iswhite(c); }
int nvim_syn_toupper_asc(int c) { return TOUPPER_ASC(c); }
char *nvim_syn_vim_strnsave_up(const char *str, int len) { return vim_strnsave_up(str, (size_t)len); }
void nvim_syn_set_nextcmd(exarg_T *eap, char *rest) { eap->nextcmd = check_nextcmd(rest); }
char *nvim_syn_get_eap_arg(const exarg_T *eap) { return eap->arg; }
int nvim_syn_get_eap_skip(const exarg_T *eap) { return eap->skip; }
char *nvim_syn_skip_regexp(char *arg, int delim, int magic) { return skip_regexp(arg, delim, magic); }
int nvim_syn_getdigits_int(char **pp, int strict, int def) { return getdigits_int(pp, (bool)strict, def); }
char *nvim_syn_get_p_cpo(void) { return p_cpo; }
void nvim_syn_set_p_cpo(char *val) { p_cpo = val; }
char *nvim_syn_get_empty_string_option(void) { return empty_string_option; }
int nvim_syn_get_curwin_syn_ic(void) { return curwin->w_s->b_syn_ic; }
int nvim_syn_vim_regcomp_had_eol(void) { return vim_regcomp_had_eol(); }
synpat_T *nvim_synblock_ga_append_pattern(void) { return GA_APPEND_VIA_PTR(synpat_T, &curwin->w_s->b_syn_patterns); }
void nvim_synblock_set_containedin(int val) { curwin->w_s->b_syn_containedin = (bool)val; }
void nvim_synblock_inc_folditems(void) { curwin->w_s->b_syn_folditems++; }
synpat_T *nvim_syn_xcalloc_synpat(void) { return xcalloc(1, sizeof(synpat_T)); }
void nvim_syn_free_synpat(synpat_T *pat) { if (pat != NULL) { vim_regfree(pat->sp_prog); xfree(pat->sp_pattern); xfree(pat); } }
void nvim_syn_set_reg_do_extmatch(int val) { reg_do_extmatch = val; }
void nvim_syn_semsg_2s(const char *fmt, const char *arg1, const char *arg2) { semsg(fmt, arg1, arg2); }
void nvim_syn_combine_cluster_list(int scl_id, int16_t **clstr_list, int list_op) { rs_syn_combine_list(&SYN_CLSTR(curwin->w_s)[scl_id].scl_list, clstr_list, list_op); }
void nvim_syn_find_nextcmd(exarg_T *eap, char *arg) { eap->nextcmd = find_nextcmd(arg); }
void nvim_syn_set_eap_arg(exarg_T *eap, char *arg) { eap->arg = arg; }
int nvim_syn_getdigits_int32(char **pp, int strict, int def) { return getdigits_int32(pp, (bool)strict, def); }
void nvim_synblock_or_sync_flags(synblock_T *block, int flags) { block->b_syn_sync_flags |= flags; }
void nvim_synblock_set_sync_id(synblock_T *block, int id) { block->b_syn_sync_id = (int16_t)id; }
void nvim_synblock_set_sync_minlines(synblock_T *block, int n) { block->b_syn_sync_minlines = (linenr_T)n; }
void nvim_synblock_set_sync_maxlines(synblock_T *block, int n) { block->b_syn_sync_maxlines = (linenr_T)n; }
void nvim_synblock_set_sync_linebreaks(synblock_T *block, int n) { block->b_syn_sync_linebreaks = (linenr_T)n; }
int nvim_synblock_get_linecont_pat_is_set(synblock_T *block) { return block->b_syn_linecont_pat != NULL ? 1 : 0; }
void nvim_synblock_set_linecont_pat(synblock_T *block, char *pat) { block->b_syn_linecont_pat = pat; }
char *nvim_synblock_get_linecont_pat(synblock_T *block) { return block->b_syn_linecont_pat; }
void nvim_synblock_set_linecont_ic(synblock_T *block, int ic) { block->b_syn_linecont_ic = ic; }
void nvim_synblock_set_linecont_prog2(synblock_T *block, void *prog) { block->b_syn_linecont_prog = (regprog_T *)prog; }
void nvim_syn_clear_linecont_pat(synblock_T *block) { XFREE_CLEAR(block->b_syn_linecont_pat); }
void nvim_synblock_set_sync_flags_zero(synblock_T *block) { block->b_syn_sync_flags = 0; }
void nvim_synblock_set_folditems(synblock_T *block, int n) { block->b_syn_folditems = n; }
void nvim_synblock_set_syn_error(synblock_T *block, int val) { block->b_syn_error = (bool)val; }
void nvim_synblock_set_syn_slow(synblock_T *block, int val) { block->b_syn_slow = (bool)val; }
void nvim_synblock_set_syn_containedin_b(synblock_T *block, int val) { block->b_syn_containedin = (bool)val; }
void nvim_synblock_set_syn_conceal(synblock_T *block, int val) { block->b_syn_conceal = (bool)val; }
void nvim_synblock_set_spell_cluster_id_b(synblock_T *block, int id) { block->b_spell_cluster_id = id; }
void nvim_synblock_set_nospell_cluster_id_b(synblock_T *block, int id) { block->b_nospell_cluster_id = id; }
void nvim_synblock_ga_clear_patterns(synblock_T *block) { ga_clear(&block->b_syn_patterns); }
void nvim_synblock_ga_clear_clusters(synblock_T *block) { ga_clear(&block->b_syn_clusters); }
void nvim_synblock_regfree_linecont_prog(synblock_T *block) { vim_regfree(block->b_syn_linecont_prog); block->b_syn_linecont_prog = NULL; }
void nvim_synblock_clear_syn_isk(synblock_T *block) { clear_string_option(&block->b_syn_isk); }
void *nvim_synblock_get_linecont_time_ptr(synblock_T *block) { return (void *)&block->b_syn_linecont_time; }
void *nvim_syn_vim_regcomp_empty_cpo(char *pat, int flags) { char *cpo_save = p_cpo; p_cpo = empty_string_option; void *prog = vim_regcomp(pat, flags); p_cpo = cpo_save; return prog; }
int nvim_syn_name2id_len_wrapper(const char *arg, int len) { return syn_name2id_len(arg, (size_t)len); }

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

void nvim_syn_init_highlight(int reset, int init) { init_highlight((bool)reset, (bool)init); }
void nvim_syn_do_cmdline_cmd(const char *cmd) { do_cmdline_cmd(cmd); }
void nvim_syn_redraw_later_curwin(void) { redraw_later(curwin, UPD_NOT_VALID); }
void nvim_syn_set_cmdlinep_from_eap(exarg_T *eap) { syn_cmdlinep = eap->cmdlinep; }
void nvim_syn_do_unlet(const char *name, int len) { do_unlet(name, (size_t)len, true); }
int nvim_synblock_is_buf_block(synblock_T *block) { return (block == &curwin->w_buffer->b_s) ? 1 : 0; }
void nvim_syn_redraw_curbuf_later(void) { redraw_curbuf_later(UPD_SOME_VALID); }
int nvim_syn_syntax_present_curwin(void) { return syntax_present(curwin) ? 1 : 0; }
int nvim_syn_get_columns(void) { return (int)Columns; }
void nvim_syn_set_include_link(int val) { include_link = val; }
void nvim_syn_set_include_default(int val) { include_default = val; }
void nvim_syn_set_include_none(int val) { include_none = val; }
int nvim_syn_get_expand_cluster_count(void) { return curwin->w_s->b_syn_clusters.ga_len; }
int nvim_syn_list_header(int did_header, int outlen, int id, int force_newline) { return syn_list_header((bool)did_header, outlen, id, (bool)force_newline) ? 1 : 0; }
void nvim_msg_puts_hl_syn(const char *s, int hl_id, bool hist) { msg_puts_hl(s, hl_id, hist); }
int nvim_syn_vim_strchr(const char *s, int c) { return vim_strchr(s, (uint8_t)c) != NULL ? 1 : 0; }
size_t nvim_ht_get_array_size(const hashtab_T *ht) { return ht->ht_mask + 1; }
size_t nvim_ht_get_used(const hashtab_T *ht) { return ht->ht_used; }
keyentry_T *nvim_ht_item_at(const hashtab_T *ht, size_t idx) { const hashitem_T *hi = &ht->ht_array[idx]; if (HASHITEM_EMPTY(hi)) return NULL; return HI2KE(hi); }
void nvim_syn_emsg_skip_inc(void) { emsg_skip++; }
void nvim_syn_emsg_skip_dec(void) { emsg_skip--; }
int nvim_syn_iskeyword_is_set(synblock_T *block) { return block->b_syn_isk != empty_string_option ? 1 : 0; }
char *nvim_syn_iskeyword_get(synblock_T *block) { return block->b_syn_isk; }
void nvim_syn_iskeyword_clear(synblock_T *block) { memmove(block->b_syn_chartab, curbuf->b_chartab, (size_t)32); clear_string_option(&block->b_syn_isk); }

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

void nvim_syn_msg_outtrans(const char *s) { msg_outtrans(s, 0, false); }
char *nvim_syn_get_var_value(const char *name) { return get_var_value(name); }
void nvim_syn_apply_autocmds_syntax(const char *arg) { apply_autocmds(EVENT_SYNTAX, arg, curbuf->b_fname, true, curbuf); }
void nvim_syn_set_internal_string_var(const char *name, const char *val) { set_internal_string_var(name, val); }
void nvim_syn_do_unlet_b_current_syntax(void) { do_unlet(S_LEN("b:current_syntax"), true); }
void nvim_synblock_set_pattern_count(synblock_T *block, int len) { block->b_syn_patterns.ga_len = len; }
void nvim_synblock_memmove_patterns(synblock_T *block, int dst_idx, int src_idx, int count) { memmove(SYN_ITEMS(block) + dst_idx, SYN_ITEMS(block) + src_idx, sizeof(synpat_T) * (size_t)count); }
void nvim_synblock_dec_folditems(synblock_T *block) { block->b_syn_folditems--; }
void nvim_win_release_synblock(win_T *wp) { if (wp->w_s != &wp->w_buffer->b_s) { syntax_clear(wp->w_s); xfree(wp->w_s); wp->w_s = &wp->w_buffer->b_s; } }
void nvim_synblock_set_spell_cluster_id(int id) { curwin->w_s->b_spell_cluster_id = id; }
void nvim_synblock_set_nospell_cluster_id(int id) { curwin->w_s->b_nospell_cluster_id = id; }
char *nvim_syn_vim_strsave_up(const char *s) { return vim_strsave_up(s); }
void nvim_synblock_ga_init_patterns(void) { curwin->w_s->b_syn_patterns.ga_itemsize = sizeof(synpat_T); ga_set_growsize(&curwin->w_s->b_syn_patterns, 10); }
synstate_T *nvim_synblock_get_sst_array_ptr(synblock_T *block) { return block ? block->b_sst_array : NULL; }
void nvim_synblock_set_sst_array(synblock_T *block, synstate_T *ptr, int len) { if (!block) return; block->b_sst_array = ptr; block->b_sst_len = len; }
void nvim_synblock_set_sst_first(synblock_T *block, synstate_T *ptr) { if (block) block->b_sst_first = ptr; }
void nvim_synblock_set_sst_firstfree(synblock_T *block, synstate_T *ptr) { if (block) block->b_sst_firstfree = ptr; }
void nvim_synblock_set_sst_freecount(synblock_T *block, int count) { if (block) block->b_sst_freecount = count; }
void nvim_synstate_set_next(synstate_T *state, synstate_T *next) { if (state) state->sst_next = next; }
void nvim_synstate_set_stacksize(synstate_T *state, int size) { if (state) state->sst_stacksize = size; }
synstate_T *nvim_syn_xcalloc_synstate_array(int len) { if (len <= 0) return NULL; return xcalloc((size_t)len, sizeof(synstate_T)); }
void nvim_syn_free_sst_array(synstate_T *ptr) { xfree(ptr); }
synstate_T *nvim_syn_sst_array_at(synstate_T *array, int idx) { return array + idx; }
void nvim_syn_sst_copy_entry(synstate_T *dst, const synstate_T *src) { *dst = *src; }
int nvim_synblock_get_sst_lasttick(synblock_T *block) { return block ? (int)block->b_sst_lasttick : 0; }
int nvim_syn_buf_get_ml_line_count(void) { return syn_buf ? (int)syn_buf->b_ml.ml_line_count : 0; }
synblock_T *nvim_buf_get_b_s(buf_T *buf) { return buf ? &buf->b_s : NULL; }

void nvim_syn_fold_update_for_block(synblock_T *block)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_s == block && rs_foldmethodIsSyntax(wp)) {
      rs_foldUpdateAll(wp);
    }
  }
}

void nvim_syn_apply_changes_for_windows(buf_T *buf)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if ((wp->w_buffer == buf) && (wp->w_s != &buf->b_s)) {
      rs_syn_stack_apply_changes_block(wp->w_s, buf);
    }
  }
}

int nvim_syn_block_isk_is_empty(void) { return (syn_block && syn_block->b_syn_isk == empty_string_option) ? 1 : 0; }
void nvim_syn_buf_chartab_get(char *dst) { if (syn_buf) memmove(dst, syn_buf->b_chartab, 32); }
void nvim_syn_buf_chartab_set(const char *src) { if (syn_buf) memmove(syn_buf->b_chartab, src, 32); }
void nvim_syn_win_chartab_get(char *dst) { if (syn_win) memmove(dst, syn_win->w_s->b_syn_chartab, 32); }
int nvim_syn_win_isk_not_empty(void) { return (syn_win && syn_win->w_s->b_syn_isk != empty_string_option) ? 1 : 0; }
void *nvim_syn_block_get_linecont_prog(void) { return (syn_block) ? syn_block->b_syn_linecont_prog : NULL; }
int nvim_syn_block_get_linecont_ic(void) { return syn_block ? syn_block->b_syn_linecont_ic : 0; }
void *nvim_syn_block_get_linecont_time_ptr(void) { return syn_block ? (void *)&syn_block->b_syn_linecont_time : NULL; }
void nvim_syn_block_set_linecont_prog(void *prog) { if (syn_block) syn_block->b_syn_linecont_prog = (regprog_T *)prog; }
char *nvim_syn_do_getcurline(void) { return ml_get_buf(syn_buf, current_lnum); }
int nvim_syn_do_getcurline_len(void) { return (int)ml_get_buf_len(syn_buf, current_lnum); }
synpat_T *nvim_synblock_get_patterns_ga_data(synblock_T *block) { if (block == NULL || block->b_syn_patterns.ga_len == 0) return NULL; return (synpat_T *)block->b_syn_patterns.ga_data; }
syn_cluster_T *nvim_synblock_get_clusters_ga_data(synblock_T *block) { if (block == NULL || block->b_syn_clusters.ga_len == 0) return NULL; return (syn_cluster_T *)block->b_syn_clusters.ga_data; }
void nvim_synstate_set_sst_next_flags(synstate_T *state, int flags) { if (state) state->sst_next_flags = (int16_t)flags; }
void nvim_synstate_set_sst_next_list(synstate_T *state, int16_t *list) { if (state) state->sst_next_list = list; }
void nvim_synstate_set_tick_to_display(synstate_T *state) { if (state) state->sst_tick = display_tick; }
void nvim_synstate_ga_clear(synstate_T *state) { if (state) ga_clear(&state->sst_union.sst_ga); }

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

void nvim_synstate_ga_init_for_store(synstate_T *sp) { if (sp == NULL) return; ga_init(&sp->sst_union.sst_ga, (int)sizeof(bufstate_T), 1); ga_grow(&sp->sst_union.sst_ga, sp->sst_stacksize); sp->sst_union.sst_ga.ga_len = sp->sst_stacksize; }
void nvim_ke_free(keyentry_T *kp) { if (!kp) return; xfree(kp->next_list); xfree(kp->k_syn.cont_in_list); xfree(kp); }
void nvim_ht_clear_and_init(hashtab_T *ht) { if (!ht) return; hash_clear(ht); hash_init(ht); }
void nvim_ht_lock(hashtab_T *ht) { if (ht) hash_lock(ht); }
void nvim_ht_unlock(hashtab_T *ht) { if (ht) hash_unlock(ht); }
void nvim_ke_set_next(keyentry_T *kp, keyentry_T *next) { if (kp) kp->ke_next = next; }
void nvim_ht_remove_at(hashtab_T *ht, size_t idx) { if (!ht || idx >= (ht->ht_mask + 1)) return; hash_remove(ht, &ht->ht_array[idx]); }
char *nvim_ke_get_hikey(keyentry_T *kp) { return kp ? KE2HIKEY(kp) : NULL; }
void nvim_ht_set_hikey_at(hashtab_T *ht, size_t idx, char *key) { if (!ht || idx >= (ht->ht_mask + 1)) return; ht->ht_array[idx].hi_key = key; }
keyentry_T *nvim_ht_find_ke(hashtab_T *ht, char *keyword) { if (!ht || !keyword) return NULL; hashitem_T *hi = hash_find(ht, keyword); if (HASHITEM_EMPTY(hi)) return NULL; return HI2KE(hi); }
hashtab_T *nvim_curwin_get_keywtab(int use_ic) { return use_ic ? &curwin->w_s->b_keywtab_ic : &curwin->w_s->b_keywtab; }
hash_T nvim_hash_hash(const char *key) { return hash_hash(key); }
hashitem_T *nvim_hash_lookup(hashtab_T *ht, const char *key, size_t len, hash_T hash) { return hash_lookup(ht, key, len, hash); }
int nvim_hashitem_is_empty(const hashitem_T *hi) { return HASHITEM_EMPTY(hi) ? 1 : 0; }
void nvim_hash_add_item(hashtab_T *ht, hashitem_T *hi, char *key, hash_T hash) { hash_add_item(ht, hi, key, hash); }
keyentry_T *nvim_hikey2ke(const hashitem_T *hi) { return hi ? HI2KE(hi) : NULL; }
char *nvim_ke2hikey(keyentry_T *kp) { return kp ? KE2HIKEY(kp) : NULL; }
void nvim_curwin_set_containedin(void) { curwin->w_s->b_syn_containedin = true; }
void nvim_hashitem_set_key(hashitem_T *hi, char *key) { if (hi) hi->hi_key = key; }
int nvim_curwin_shares_buf_synblock(void) { return curwin->w_s == &curwin->w_buffer->b_s ? 1 : 0; }

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

void nvim_curwin_set_synblock(synblock_T *block) { curwin->w_s = block; curwin->w_p_spell = false; }

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

const char *nvim_synblock_get_b_p_spc(synblock_T *block) { return block->b_p_spc; }
regprog_T *nvim_synblock_get_b_cap_prog(synblock_T *block) { return block->b_cap_prog; }
void nvim_synblock_set_b_cap_prog(synblock_T *block, regprog_T *prog) { block->b_cap_prog = prog; }
