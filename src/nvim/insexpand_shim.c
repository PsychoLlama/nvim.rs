// insexpand_shim.c: C shim for Insert mode completion (Rust insexpand crate)


#include "klib/kvec.h"
#include "nvim/autocmd.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_getln.h"
#include "nvim/extmark.h"
#include "nvim/fuzzy.h"
#include "nvim/globals.h"
#include "nvim/highlight_group.h"
#include "nvim/insexpand.h"
#include "nvim/lua/executor.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/path.h"
#include "nvim/popupmenu.h"
#include "nvim/register.h"
#include "nvim/spell.h"
#include "nvim/tag.h"
#include "nvim/winfloat.h"

// Rust rs_* function declarations called from this file
extern bool rs_set_ref_in_callback(Callback *callback, int copyID, ht_stack_T **ht_stack,
                                   list_stack_T **list_stack);
extern int rs_get_cpt_sources_count(void);
extern int rs_ins_compl_add(char *str, int len, char *fname, char **cptext,
                            int cptext_allocated, void *user_data, int cdir,
                            int flags_arg, int adup, const int *user_hl, int score);
extern int rs_ins_compl_add_infercase(char *str, int len, int icase, const char *fname,
                                      int dir, int cont_s_ipos, int score);

int nvim_ins_compl_add_tv_impl(void *tv, int dir, int fast);
void nvim_ins_compl_add_list_impl(void *list);
void nvim_ins_compl_add_dict_impl(void *dict);
void nvim_expand_by_function_full_impl(int type, char *base, void *cb);
void *nvim_get_callback_if_cpt_func_impl(const char *p, int idx);

#define CTRL_X_WANT_IDENT       0x100

enum {
  CTRL_X_NORMAL = 0,  ///< CTRL-N CTRL-P completion, default
  CTRL_X_NOT_DEFINED_YET = 1,
  CTRL_X_SCROLL = 2,
  CTRL_X_WHOLE_LINE = 3,
  CTRL_X_FILES = 4,
  CTRL_X_TAGS = (5 + CTRL_X_WANT_IDENT),
  CTRL_X_PATH_PATTERNS = (6 + CTRL_X_WANT_IDENT),
  CTRL_X_PATH_DEFINES = (7 + CTRL_X_WANT_IDENT),
  CTRL_X_FINISHED = 8,
  CTRL_X_DICTIONARY = (9 + CTRL_X_WANT_IDENT),
  CTRL_X_THESAURUS = (10 + CTRL_X_WANT_IDENT),
  CTRL_X_CMDLINE = 11,
  CTRL_X_FUNCTION = 12,
  CTRL_X_OMNI = 13,
  CTRL_X_SPELL = 14,
  CTRL_X_LOCAL_MSG = 15,       ///< only used in "ctrl_x_msgs"
  CTRL_X_EVAL = 16,            ///< for builtin function complete()
  CTRL_X_CMDLINE_CTRL_X = 17,  ///< CTRL-X typed in CTRL_X_CMDLINE
  CTRL_X_BUFNAMES = 18,
  CTRL_X_REGISTER = 19,        ///< complete words from registers
};

/// Message for CTRL-X mode, index is ctrl_x_mode.
static char *ctrl_x_msgs[] = {
  N_(" Keyword completion (^N^P)"),  // CTRL_X_NORMAL, ^P/^N compl.
  N_(" ^X mode (^]^D^E^F^I^K^L^N^O^P^Rs^U^V^Y)"),
  NULL,  // CTRL_X_SCROLL: depends on state
  N_(" Whole line completion (^L^N^P)"),
  N_(" File name completion (^F^N^P)"),
  N_(" Tag completion (^]^N^P)"),
  N_(" Path pattern completion (^N^P)"),
  N_(" Definition completion (^D^N^P)"),
  NULL,  // CTRL_X_FINISHED
  N_(" Dictionary completion (^K^N^P)"),
  N_(" Thesaurus completion (^T^N^P)"),
  N_(" Command-line completion (^V^N^P)"),
  N_(" User defined completion (^U^N^P)"),
  N_(" Omni completion (^O^N^P)"),
  N_(" Spelling suggestion (^S^N^P)"),
  N_(" Keyword Local completion (^N^P)"),
  NULL,  // CTRL_X_EVAL doesn't use msg.
  N_(" Command-line completion (^V^N^P)"),
  NULL,
  N_(" Register completion (^N^P)"),
};

/// Structure used to store one match for insert completion.
typedef struct compl_S compl_T;
struct compl_S {
  compl_T *cp_next;
  compl_T *cp_prev;
  compl_T *cp_match_next;        ///< matched next compl_T
  String cp_str;                 ///< matched text
  char *(cp_text[CPT_COUNT]);    ///< text for the menu
  typval_T cp_user_data;
  char *cp_fname;                ///< file containing the match, allocated when
                                 ///< cp_flags has CP_FREE_FNAME
  int cp_flags;                  ///< CP_ values
  int cp_number;                 ///< sequence number
  int cp_score;                  ///< fuzzy match score or proximity score
  bool cp_in_match_array;        ///< collected by compl_match_array
  int cp_user_abbr_hlattr;       ///< highlight attribute for abbr
  int cp_user_kind_hlattr;       ///< highlight attribute for kind
  int cp_cpt_source_idx;         ///< index of this match's source in 'cpt' option
};

/// state information used for getting the next set of insert completion
/// matches.
typedef struct {
  char *e_cpt_copy;       ///< copy of 'complete'
  char *e_cpt;            ///< current entry in "e_cpt_copy"
  buf_T *ins_buf;         ///< buffer being scanned
  pos_T *cur_match_pos;   ///< current match position
  pos_T prev_match_pos;   ///< previous match position
  bool set_match_pos;     ///< save first_match_pos/last_match_pos
  pos_T first_match_pos;  ///< first match position
  pos_T last_match_pos;   ///< last match position
  bool found_all;         ///< found all matches of a certain type.
  char *dict;             ///< dictionary file to search
  int dict_f;             ///< "dict" is an exact file name or not
  Callback *func_cb;      ///< callback of function in 'cpt' option
} ins_compl_next_state_T;

// Static state for ins_compl_get_exp, made non-static for Rust access
ins_compl_next_state_T ins_compl_st;
bool ins_compl_st_cleared = false;  ///< made non-static for Rust access

// In large buffers, timeout may miss nearby matches — search above cursor
#define LOOKBACK_LINE_COUNT     1000

#include "insexpand_shim.c.generated.h"

/// values for cp_flags
typedef enum {
  CP_ORIGINAL_TEXT = 1,  ///< the original text when the expansion begun
  CP_FREE_FNAME = 2,     ///< cp_fname is allocated
  CP_CONT_S_IPOS = 4,    ///< use CONT_S_IPOS for compl_cont_status
  CP_EQUAL = 8,          ///< rs_ins_compl_equal() always returns true
  CP_ICASE = 16,         ///< ins_compl_equal ignores case
  CP_FAST = 32,          ///< use fast_breakcheck instead of os_breakcheck
} cp_flags_T;

static const char e_compldel[] = N_("E840: Completion function deleted text");

// Completion match list: compl_first_match = start, compl_curr_match = selected,
// compl_shown_match = displayed during ins_compl_get_exp, compl_old_match = previous.
compl_T *compl_first_match = NULL;
compl_T *compl_curr_match = NULL;
compl_T *compl_shown_match = NULL;
compl_T *compl_old_match = NULL;
compl_T **compl_best_matches = NULL;  ///< compl_T entries with the max score
int compl_num_bests = 0;
bool compl_enter_selects = false;     ///< <Enter> selects match in popup menu
String compl_leader = STRING_INIT;    ///< only matches starting with this string are used
bool compl_get_longest = false;       ///< put longest common string in compl_leader
bool compl_used_match;                ///< false when no match selected or match edited
bool compl_was_interrupted = false;   ///< didn't finish finding completions
bool compl_interrupted = false;       ///< stop looking for matches
bool compl_started = false;           ///< false = word to be completed must be located
int ctrl_x_mode = CTRL_X_NORMAL;      ///< which Ctrl-X mode are we in?
int compl_matches = 0;                ///< number of completion matches
String compl_pattern = STRING_INIT;   ///< search pattern for matching items
String cpt_compl_pattern = STRING_INIT;  ///< pattern returned by func in 'cpt'
Direction compl_direction = FORWARD;
Direction compl_shows_dir = FORWARD;
pos_T compl_startpos;
int compl_length = 0;                 ///< length in bytes of text being completed
linenr_T compl_lnum = 0;              ///< lnum where the completion starts
colnr_T compl_col = 0;                ///< column where the completed text starts
colnr_T compl_ins_end_col = 0;
String compl_orig_text = STRING_INIT;
static extmark_undo_vec_t compl_orig_extmarks;  ///< undo info for restoring extmarks
int compl_cont_mode = 0;
expand_T compl_xp;
win_T *compl_curr_win = NULL;         ///< win where completion is active
buf_T *compl_curr_buf = NULL;         ///< buf where completion is active
bool compl_autocomplete = false;      ///< whether autocompletion is active
uint64_t compl_timeout_ms = 80;       ///< current timeout (COMPL_INITIAL_TIMEOUT_MS = 80ms)
bool compl_time_slice_expired = false;  ///< time budget exceeded for current source
bool compl_from_nonkeyword = false;   ///< completion started from non-keyword
int compl_cont_status = 0;
bool compl_opt_refresh_always = false;
size_t spell_bad_len = 0;
int compl_selected_item = -1;

/// Completion source info (one entry per 'cpt' option source)
typedef struct cpt_source_T {
  bool cs_refresh_always;   ///< Whether 'refresh:always' is set for func
  int cs_startcol;          ///< Start column returned by func
  int cs_max_matches;       ///< Max items to display from this source
  uint64_t compl_start_tv;  ///< Timestamp when match collection starts
  char cs_flag;             ///< Flag indicating the type of source
} cpt_source_T;

cpt_source_T *cpt_sources_array;
int cpt_sources_count;    ///< total number of completion sources in 'cpt' option
int cpt_sources_index = -1;  ///< current source index being expanded
pumitem_T *compl_match_array = NULL;  ///< popup menu entries (NULL = no popup)
int compl_match_arraysize;

/// @return  true if "match" is the original text when the completion began.
static bool match_at_original_text(const compl_T *const match) { return match->cp_flags & CP_ORIGINAL_TEXT; }
/// @return  true if "match" is the first match in the completion list.
static bool is_first_match(const compl_T *const match) { return match == compl_first_match; }
int ins_compl_add_infercase(char *str_arg, int len, bool icase, char *fname, Direction dir,
                            bool cont_s_ipos, int score)
  FUNC_ATTR_NONNULL_ARG(1)
  { return rs_ins_compl_add_infercase(str_arg, len, icase ? 1 : 0, fname, (int)dir, cont_s_ipos ? 1 : 0, score); }

static inline void free_cptext(char *const *const cptext)
  { if (cptext == NULL) { return; } for (size_t i = 0; i < CPT_COUNT; i++) { xfree(cptext[i]); } }

static dict_T *ins_compl_dict_alloc(compl_T *match)
{
  dict_T *dict = tv_dict_alloc_lock(VAR_FIXED);
  tv_dict_add_str(dict, S_LEN("word"), match->cp_str.data);
  tv_dict_add_str(dict, S_LEN("abbr"), match->cp_text[CPT_ABBR]);
  tv_dict_add_str(dict, S_LEN("menu"), match->cp_text[CPT_MENU]);
  tv_dict_add_str(dict, S_LEN("kind"), match->cp_text[CPT_KIND]);
  tv_dict_add_str(dict, S_LEN("info"), match->cp_text[CPT_INFO]);
  if (match->cp_user_data.v_type == VAR_UNKNOWN) { tv_dict_add_str(dict, S_LEN("user_data"), ""); }
  else { tv_dict_add_tv(dict, S_LEN("user_data"), &match->cp_user_data); }
  return dict;
}

static void *cp_get_next(void *node) { return ((compl_T *)node)->cp_next; }
static void cp_set_next(void *node, void *next) { ((compl_T *)node)->cp_next = (compl_T *)next; }
static void *cp_get_prev(void *node) { return ((compl_T *)node)->cp_prev; }
static void cp_set_prev(void *node, void *prev) { ((compl_T *)node)->cp_prev = (compl_T *)prev; }
static int cp_compare_fuzzy(const void *a, const void *b)
  { int sa = ((compl_T *)a)->cp_score, sb = ((compl_T *)b)->cp_score; return (sb > sa) ? 1 : (sb < sa) ? -1 : 0; }
static int cp_compare_nearest(const void *a, const void *b)
  { int sa = ((compl_T *)a)->cp_score, sb = ((compl_T *)b)->cp_score; return (sa == FUZZY_SCORE_NONE || sb == FUZZY_SCORE_NONE) ? 0 : (sa > sb) ? 1 : (sa < sb) ? -1 : 0; }

const char *nvim_get_curbuf_b_p_tsr(void) { return *curbuf->b_p_tsr == NUL ? p_tsr : curbuf->b_p_tsr; }
const char *nvim_get_curbuf_b_p_dict(void) { return *curbuf->b_p_dict == NUL ? p_dict : curbuf->b_p_dict; }
static void ins_compl_item_free(compl_T *match)
  { API_CLEAR_STRING(match->cp_str); if (match->cp_flags & CP_FREE_FNAME) { xfree(match->cp_fname); } free_cptext(match->cp_text); tv_clear(&match->cp_user_data); xfree(match); }

void *nvim_compl_match_get_next(void *m) { return m ? ((compl_T *)m)->cp_next : NULL; }
void nvim_compl_match_set_next(void *m, void *next) { if (m) ((compl_T *)m)->cp_next = (compl_T *)next; }
void *nvim_compl_match_get_prev(void *m) { return m ? ((compl_T *)m)->cp_prev : NULL; }
void nvim_compl_match_set_prev(void *m, void *prev) { if (m) ((compl_T *)m)->cp_prev = (compl_T *)prev; }
int nvim_compl_match_get_flags(void *m) { return m ? ((compl_T *)m)->cp_flags : 0; }
int nvim_compl_match_get_score(void *m) { return m ? ((compl_T *)m)->cp_score : -1; }
int nvim_compl_match_at_original_text(void *m) { return (m && (((compl_T *)m)->cp_flags & CP_ORIGINAL_TEXT)) ? 1 : 0; }
int nvim_compl_match_get_cpt_source_idx(void *m) { return m ? ((compl_T *)m)->cp_cpt_source_idx : -1; }
int nvim_compl_match_get_in_match_array(void *m) { return (m && ((compl_T *)m)->cp_in_match_array) ? 1 : 0; }
void nvim_compl_match_set_in_match_array(void *m, int val) { if (m) ((compl_T *)m)->cp_in_match_array = (val != 0); }
void *nvim_compl_match_get_match_next(void *m) { return m ? ((compl_T *)m)->cp_match_next : NULL; }
void nvim_compl_match_set_match_next(void *m, void *next) { if (m) ((compl_T *)m)->cp_match_next = (compl_T *)next; }
void nvim_compl_match_clear_icase(void *m) { if (m) ((compl_T *)m)->cp_flags &= ~CP_ICASE; }
int nvim_build_pum_fill_array(void *match_head_void, int count) {
  assert(count >= 0);
  compl_match_array = xcalloc((size_t)count, sizeof(pumitem_T));
  int i = 0;
  for (compl_T *comp = (compl_T *)match_head_void; comp != NULL;) {
    compl_match_array[i].pum_text = comp->cp_text[CPT_ABBR] ? comp->cp_text[CPT_ABBR] : comp->cp_str.data;
    compl_match_array[i].pum_kind = comp->cp_text[CPT_KIND]; compl_match_array[i].pum_info = comp->cp_text[CPT_INFO];
    compl_match_array[i].pum_cpt_source_idx = comp->cp_cpt_source_idx;
    compl_match_array[i].pum_user_abbr_hlattr = comp->cp_user_abbr_hlattr; compl_match_array[i].pum_user_kind_hlattr = comp->cp_user_kind_hlattr;
    compl_match_array[i++].pum_extra = comp->cp_text[CPT_MENU] ? comp->cp_text[CPT_MENU] : comp->cp_fname;
    compl_T *next = comp->cp_match_next; comp->cp_match_next = NULL; comp = next;
  }
  return i;
}
int nvim_find_shown_match_in_match_array(void) {
  if (!compl_match_array || !compl_shown_match) { return -1; }
  for (int i = 0; i < compl_match_arraysize; i++) {
    if (compl_match_array[i].pum_text == compl_shown_match->cp_str.data || compl_match_array[i].pum_text == compl_shown_match->cp_text[CPT_ABBR]) { return i; }
  }
  return -1;
}

void nvim_compl_item_free(void *m) { if (m) ins_compl_item_free((compl_T *)m); }
int nvim_compl_match_get_cp_number(void *m) { return m ? ((compl_T *)m)->cp_number : -1; }
void nvim_compl_match_set_cp_number(void *m, int num) { if (m) ((compl_T *)m)->cp_number = num; }
const char *nvim_curbuf_get_b_p_cpt(void) { return curbuf->b_p_cpt; }
void nvim_clear_compl_orig_extmarks(void) { kv_destroy(compl_orig_extmarks); }
void nvim_set_completed_item_empty(void) { set_vim_var_dict(VV_COMPLETED_ITEM, tv_dict_alloc_lock(VAR_FIXED)); }
void nvim_compl_match_set_score(void *m, int score) { if (m) { ((compl_T *)m)->cp_score = score; } }
const char *nvim_compl_match_get_cp_str_data(void *m) { return m ? ((compl_T *)m)->cp_str.data : NULL; }
size_t nvim_compl_match_get_cp_str_size(void *m) { return m ? ((compl_T *)m)->cp_str.size : 0; }
int nvim_compl_match_has_fname(void *m) { return (m && ((compl_T *)m)->cp_fname != NULL) ? 1 : 0; }
const char *nvim_compl_shown_match_fname(void) { return compl_shown_match ? compl_shown_match->cp_fname : NULL; }
const char *nvim_compl_match_get_cp_text_abbr(void *m) { return m ? ((compl_T *)m)->cp_text[CPT_ABBR] : NULL; }
const char *nvim_compl_match_get_cp_text_menu(void *m) { return m ? ((compl_T *)m)->cp_text[CPT_MENU] : NULL; }
const char *nvim_compl_match_get_cp_text_kind(void *m) { return m ? ((compl_T *)m)->cp_text[CPT_KIND] : NULL; }
const char *nvim_compl_match_get_cp_text_info(void *m) { return m ? ((compl_T *)m)->cp_text[CPT_INFO] : NULL; }
int nvim_compl_match_user_data_is_unknown(void *m) { return (!m || ((compl_T *)m)->cp_user_data.v_type == VAR_UNKNOWN) ? 1 : 0; }
void nvim_compl_match_copy_user_data_tv(void *m, void *dest_tv) { if (m && dest_tv) *(typval_T *)dest_tv = ((compl_T *)m)->cp_user_data; }
void *nvim_compl_T_alloc(void) { return xcalloc(1, sizeof(compl_T)); }
void nvim_compl_match_set_flags(void *m, int f) { if (m) ((compl_T *)m)->cp_flags = f; }
void nvim_compl_match_set_cp_str(void *m, const char *s, size_t l) { if (m) ((compl_T *)m)->cp_str = cbuf_to_string(s, l); }
void nvim_compl_match_replace_cp_str(void *m, const char *s, size_t l) { if (m) { API_CLEAR_STRING(((compl_T *)m)->cp_str); ((compl_T *)m)->cp_str = cbuf_to_string(s, l); } }
const char *nvim_compl_match_get_cp_fname(void *m) { return m ? ((compl_T *)m)->cp_fname : NULL; }
void nvim_compl_match_set_cp_fname_dup(void *m, const char *f) { if (m) ((compl_T *)m)->cp_fname = xstrdup(f); }
void nvim_compl_match_set_cp_fname_ref(void *m, const char *f) { if (m) ((compl_T *)m)->cp_fname = (char *)f; }
void nvim_compl_match_set_hl_attrs(void *m, int abbr, int kind) { if (m) { ((compl_T *)m)->cp_user_abbr_hlattr = abbr; ((compl_T *)m)->cp_user_kind_hlattr = kind; } }
void nvim_compl_match_set_cpt_source_idx(void *m, int idx) { if (m) ((compl_T *)m)->cp_cpt_source_idx = idx; }
void nvim_compl_match_set_cp_text_take(void *m, int i, char *s) { if (m) ((compl_T *)m)->cp_text[i] = s; }
void nvim_compl_match_set_cp_text_copy(void *m, int i, const char *s) { if (m) ((compl_T *)m)->cp_text[i] = xstrdup(s); }
void nvim_compl_match_set_user_data_move(void *m, void *tv) { if (m && tv) ((compl_T *)m)->cp_user_data = *(typval_T *)tv; }
void *nvim_ci_dict_alloc(void) { return tv_dict_alloc(); }
void *nvim_ci_list_alloc_known(void) { return tv_list_alloc(kListLenMayKnow); }
int nvim_ci_dict_add_str(void *d, const char *key, size_t klen, const char *val) { return tv_dict_add_str((dict_T *)d, key, klen, val); }
int nvim_ci_dict_add_str_len(void *d, const char *key, size_t klen, const char *val, int vlen) { return tv_dict_add_str_len((dict_T *)d, key, klen, val, vlen); }
int nvim_ci_dict_add_nr(void *d, const char *key, size_t klen, int64_t nr) { return tv_dict_add_nr((dict_T *)d, key, klen, (varnumber_T)nr); }
int nvim_ci_dict_add_bool(void *d, const char *key, size_t klen, int val) { return tv_dict_add_bool((dict_T *)d, key, klen, (BoolVarValue)val); }
int nvim_ci_dict_add_tv(void *d, const char *key, size_t klen, void *tv) { return tv_dict_add_tv((dict_T *)d, key, klen, (typval_T *)tv); }
int nvim_ci_dict_add_dict(void *d, const char *key, size_t klen, void *val_dict) { return tv_dict_add_dict((dict_T *)d, key, klen, (dict_T *)val_dict); }
int nvim_ci_dict_add_list(void *d, const char *key, size_t klen, void *list) { return tv_dict_add_list((dict_T *)d, key, klen, (list_T *)list); }
void nvim_ci_list_append_dict(void *list, void *dict) { tv_list_append_dict((list_T *)list, (dict_T *)dict); }
int nvim_pum_visible(void) { return pum_visible(); }
void *nvim_win_float_find_preview(void) { return win_float_find_preview(); }
int nvim_ci_win_get_handle(void *wp) { return wp ? ((win_T *)wp)->handle : -1; }
int nvim_ci_win_get_buf_handle(void *wp) { return (wp && ((win_T *)wp)->w_buffer) ? ((win_T *)wp)->w_buffer->handle : -1; }
void nvim_compl_set_vim_var_dict_shown(void) { set_vim_var_dict(VV_COMPLETED_ITEM, ins_compl_dict_alloc(compl_shown_match)); }
const char *nvim_ci_preinserted_text_ptr(void) { char *line = get_cursor_line_ptr(); int len = compl_ins_end_col - curwin->w_cursor.col; return (len > 0) ? line + curwin->w_cursor.col : ""; }
int nvim_ci_preinserted_text_len(void) { int len = compl_ins_end_col - curwin->w_cursor.col; return len > 0 ? len : 0; }
void nvim_trigger_complete_changed_guarded(int cur)
{
  static bool recursive = false;
  if (recursive) { return; }
  save_v_event_T save_v_event;
  dict_T *v_event = get_v_event(&save_v_event);
  tv_dict_add_dict(v_event, S_LEN("completed_item"), cur < 0 ? tv_dict_alloc() : ins_compl_dict_alloc(compl_curr_match));
  pum_set_event_info(v_event); tv_dict_set_keys_readonly(v_event);
  recursive = true; textlock++;
  apply_autocmds(EVENT_COMPLETECHANGED, NULL, NULL, false, curbuf);
  textlock--; recursive = false;
  restore_v_event(v_event, &save_v_event);
}

static Callback cfu_cb;    ///< 'completefunc' callback function
static Callback ofu_cb;    ///< 'omnifunc' callback function
static Callback tsrfu_cb;  ///< 'thesaurusfunc' callback function
static Callback *cpt_cb;   ///< Callback functions associated with F{func}
static int cpt_cb_count;   ///< Number of cpt callbacks

static void copy_global_to_buflocal_cb(Callback *globcb, Callback *bufcb)
  { callback_free(bufcb); if (globcb->type != kCallbackNone) { callback_copy(bufcb, globcb); } }
const char *nvim_did_set_completefunc_impl(void *a)
  { buf_T *buf = ((optset_T *)a)->os_buf; if (option_set_callback_func(buf->b_p_cfu, &cfu_cb) == FAIL) { return e_invarg; } set_buflocal_cfu_callback(buf); return NULL; }
void set_buflocal_cfu_callback(buf_T *buf) { copy_global_to_buflocal_cb(&cfu_cb, &buf->b_cfu_cb); }
const char *nvim_did_set_omnifunc_impl(void *a)
  { buf_T *buf = ((optset_T *)a)->os_buf; if (option_set_callback_func(buf->b_p_ofu, &ofu_cb) == FAIL) { return e_invarg; } set_buflocal_ofu_callback(buf); return NULL; }
void set_buflocal_ofu_callback(buf_T *buf) { copy_global_to_buflocal_cb(&ofu_cb, &buf->b_ofu_cb); }
void clear_cpt_callbacks(Callback **callbacks, int count)
  { if (callbacks == NULL || *callbacks == NULL) { return; } for (int i = 0; i < count; i++) { callback_free(&(*callbacks)[i]); } XFREE_CLEAR(*callbacks); }
static void copy_cpt_callbacks(Callback **dest, int *dest_cnt, Callback *src, int cnt)
  { if (cnt == 0) { return; } clear_cpt_callbacks(dest, *dest_cnt); *dest = xcalloc((size_t)cnt, sizeof(Callback)); *dest_cnt = cnt; for (int i = 0; i < cnt; i++) { if (src[i].type != kCallbackNone) { callback_copy(&(*dest)[i], &src[i]); } } }
void set_buflocal_cpt_callbacks(buf_T *buf)
  { if (buf != NULL && cpt_cb_count != 0) { copy_cpt_callbacks(&buf->b_p_cpt_cb, &buf->b_p_cpt_count, cpt_cb, cpt_cb_count); } }
int set_cpt_callbacks(optset_T *args)
{
  if (curbuf == NULL) { return FAIL; }
  bool local = (args->os_flags & OPT_LOCAL) != 0;
  clear_cpt_callbacks(&curbuf->b_p_cpt_cb, curbuf->b_p_cpt_count);
  curbuf->b_p_cpt_count = 0;
  int count = rs_get_cpt_sources_count();
  if (count == 0) { return OK; }
  curbuf->b_p_cpt_cb = xcalloc((size_t)count, sizeof(Callback));
  curbuf->b_p_cpt_count = count;
  char buf[LSIZE];
  int idx = 0;
  for (char *p = curbuf->b_p_cpt; *p != NUL;) {
    while (*p == ',' || *p == ' ') { p++; }
    if (*p != NUL) {
      size_t slen = copy_option_part(&p, buf, LSIZE, ",");
      if (slen > 0 && buf[0] == 'F' && buf[1] != NUL) {
        char *caret = vim_strchr(buf, '^');
        if (caret != NULL) { *caret = NUL; }
        if (option_set_callback_func(buf + 1, &curbuf->b_p_cpt_cb[idx]) != OK) { curbuf->b_p_cpt_cb[idx].type = kCallbackNone; }
      }
      idx++;
    }
  }
  if (!local) { copy_cpt_callbacks(&cpt_cb, &cpt_cb_count, curbuf->b_p_cpt_cb, curbuf->b_p_cpt_count); }
  return OK;
}
const char *nvim_did_set_thesaurusfunc_impl(void *a) {
  optset_T *args = (optset_T *)a; buf_T *buf = args->os_buf; int rv;
  if (args->os_flags & OPT_LOCAL) { rv = option_set_callback_func(buf->b_p_tsrfu, &buf->b_tsrfu_cb); }
  else { rv = option_set_callback_func(p_tsrfu, &tsrfu_cb); if (!(args->os_flags & OPT_GLOBAL)) { callback_free(&buf->b_tsrfu_cb); } }
  return rv == FAIL ? e_invarg : NULL;
}
bool set_ref_in_cpt_callbacks(Callback *callbacks, int count, int copyID)
{
  if (callbacks == NULL) { return false; }
  bool abort = false;
  for (int i = 0; i < count; i++) { abort = abort || rs_set_ref_in_callback(&callbacks[i], copyID, NULL, NULL); }
  return abort;
}
int nvim_set_ref_in_insexpand_funcs_impl(int copyID)
{
  bool ab = rs_set_ref_in_callback(&cfu_cb, copyID, NULL, NULL) || rs_set_ref_in_callback(&ofu_cb, copyID, NULL, NULL)
            || rs_set_ref_in_callback(&tsrfu_cb, copyID, NULL, NULL) || set_ref_in_cpt_callbacks(cpt_cb, cpt_cb_count, copyID);
  return ab ? 1 : 0;
}

static char *get_complete_funcname(int type)
{
  switch (type) {
  case CTRL_X_FUNCTION:
    return curbuf->b_p_cfu;
  case CTRL_X_OMNI:
    return curbuf->b_p_ofu;
  case CTRL_X_THESAURUS:
    return *curbuf->b_p_tsrfu == NUL ? p_tsrfu : curbuf->b_p_tsrfu;
  default:
    return "";
  }
}

static Callback *get_insert_callback(int type)
{
  if (type == CTRL_X_FUNCTION) {
    return &curbuf->b_cfu_cb;
  }
  if (type == CTRL_X_OMNI) {
    return &curbuf->b_ofu_cb;
  }
  // CTRL_X_THESAURUS
  return (*curbuf->b_p_tsrfu != NUL) ? &curbuf->b_tsrfu_cb : &tsrfu_cb;
}

void nvim_extmark_splice_delete_compl(void)
  { extmark_splice_delete(curbuf, curwin->w_cursor.lnum - 1, compl_col, curwin->w_cursor.lnum - 1, compl_col + compl_length, &compl_orig_extmarks, true, kExtmarkUndo); }
size_t nvim_compl_orig_extmarks_size(void) { return kv_size(compl_orig_extmarks); }
void nvim_extmark_apply_undo_at(size_t idx) { extmark_apply_undo(kv_A(compl_orig_extmarks, idx), true); }
void nvim_api_clear_string_compl_orig_text(void) { API_CLEAR_STRING(compl_orig_text); }
void nvim_callback_free_cfu(void) { callback_free(&cfu_cb); }
void nvim_callback_free_ofu(void) { callback_free(&ofu_cb); }
void nvim_callback_free_tsrfu(void) { callback_free(&tsrfu_cb); }
void nvim_clear_static_cpt_callbacks(void) { clear_cpt_callbacks(&cpt_cb, cpt_cb_count); }

void nvim_set_curbuf_b_p_com_empty(void) { curbuf->b_p_com = ""; }
void nvim_restore_curbuf_b_p_com(const char *old_val) { curbuf->b_p_com = (char *)old_val; }
const char *nvim_get_curbuf_b_p_com(void) { return curbuf->b_p_com; }
int nvim_compl_xp_get_context(void) { return compl_xp.xp_context; }
const char *nvim_compl_xp_get_pattern(void) { return compl_xp.xp_pattern; }
void nvim_compl_xp_set_cmd_context(int len, int col) { set_cmd_context(&compl_xp, compl_pattern.data, len, col, false); }
void nvim_compl_xp_nlua_expand(void) { nlua_expand_pat(&compl_xp); }
const char *nvim_ml_get_curline(void) { return ml_get(curwin->w_cursor.lnum); }
char *nvim_get_ctrl_x_msg(int idx) { return _(ctrl_x_msgs[idx & ~CTRL_X_WANT_IDENT]); }
unsigned nvim_curbuf_get_b_cot_flags(void) { return curbuf->b_cot_flags; }
int nvim_get_p_ic(void) { return p_ic ? 1 : 0; }
int nvim_get_p_inf(void) { return curbuf->b_p_inf ? 1 : 0; }
int nvim_curbuf_get_b_p_ac(void) { return curbuf->b_p_ac; }
int nvim_get_curwin_cursor_lnum(void) { return (int)curwin->w_cursor.lnum; }
void nvim_set_edit_submode_scroll(int is_replace) { edit_submode = is_replace ? _(" (replace) Scroll (^E/^Y)") : _(" (insert) Scroll (^E/^Y)"); edit_submode_pre = NULL; redraw_mode = true; }
void nvim_spell_back_safe(void)
  { emsg_off++; pos_T tpos = curwin->w_cursor; spell_bad_len = spell_move_to(curwin, BACKWARD, SMT_ALL, true, NULL); if (curwin->w_cursor.col != tpos.col) { start_arrow(&tpos); } emsg_off--; }
char *nvim_get_compl_shown_match_str_dup(void) { return compl_shown_match ? xstrdup(compl_shown_match->cp_str.data) : NULL; }
int nvim_cursor_on_nul(void) { char *line = get_cursor_line_ptr(); return (line && line[curwin->w_cursor.col] != NUL) ? 1 : 0; }
void nvim_ins_apply_autocmds_completedonepre(void) { ins_apply_autocmds(EVENT_COMPLETEDONEPRE); }
int nvim_has_completechanged_event(void) { return has_event(EVENT_COMPLETECHANGED) ? 1 : 0; }
void nvim_pum_display_compl(int cur, int array_changed) { pum_display(compl_match_array, compl_match_arraysize, cur, array_changed != 0, 0); }
void nvim_set_edit_submode_ctrl_x_msg(int mode) { edit_submode = _(ctrl_x_msgs[(mode) & ~CTRL_X_WANT_IDENT]); }
int nvim_check_compl_option_dict(void) { return (*curbuf->b_p_dict == NUL && *p_dict == NUL && !curwin->w_p_spell) ? 1 : 0; }
int nvim_check_compl_option_tsr(void) { return (*curbuf->b_p_tsr == NUL && *p_tsr == NUL && *curbuf->b_p_tsrfu == NUL && *p_tsrfu == NUL) ? 1 : 0; }
void nvim_ins_compl_st_mark_ins_buf_scanned(void) { if (ins_compl_st.ins_buf) { ins_compl_st.ins_buf->b_scanned = true; } }
void nvim_clear_all_buf_scanned(void) { FOR_ALL_BUFFERS(buf) { buf->b_scanned = false; } }
void nvim_clear_ins_compl_st(void) { CLEAR_FIELD(ins_compl_st); }
int nvim_expand_wildcards_files(int count, char **pat, int *num_matches, char ***matches) { return expand_wildcards(count, pat, num_matches, matches, EW_FILE|EW_DIR|EW_ADDSLASH|EW_SILENT); }
void nvim_tilde_replace_wrap(char *pat, int num_matches, char **matches) { tilde_replace(pat, num_matches, matches); }
void *nvim_mergesort_compl_list_raw(void *head, int compare_type) { return mergesort_list(head, cp_get_next, cp_set_next, cp_get_prev, cp_set_prev, compare_type == 0 ? cp_compare_fuzzy : cp_compare_nearest); }
void nvim_redraw_later_valid(void) { redraw_later(curwin, UPD_VALID); }
int nvim_get_curbuf_b_p_tsrfu_nonempty(void) { return *curbuf->b_p_tsrfu != NUL ? 1 : 0; }

void nvim_do_autocmd_completedone_with_strs(const char *word, const char *complete_type, const char *reason)
{
  save_v_event_T save_v_event;
  dict_T *v_event = get_v_event(&save_v_event);
  tv_dict_add_str(v_event, S_LEN("complete_word"), word); tv_dict_add_str(v_event, S_LEN("complete_type"), complete_type); tv_dict_add_str(v_event, S_LEN("reason"), reason);
  tv_dict_set_keys_readonly(v_event); ins_apply_autocmds(EVENT_COMPLETEDONE); restore_v_event(v_event, &save_v_event);
}

int nvim_skipwhite_offset(const char *line, int length, int start_col) { return (int)(skipwhite(line + length + start_col) - line); }
size_t nvim_yankreg_y_size(void *reg) { return reg ? ((yankreg_T *)reg)->y_size : 0; }
int nvim_yankreg_y_array_null(void *reg) { return (!reg || ((yankreg_T *)reg)->y_array == NULL) ? 1 : 0; }
const char *nvim_yankreg_y_array_entry_data(void *reg, size_t j)
  { yankreg_T *r = (yankreg_T *)reg; return (!r || j >= r->y_size || !r->y_array) ? NULL : r->y_array[j].data; }
int nvim_get_curwin_w_wrow(void) { return curwin->w_wrow; }
size_t nvim_copy_option_part_ffi(char **src, char *buf, int maxlen, const char *sep) { return copy_option_part(src, buf, (size_t)maxlen, sep); }
int nvim_get_complete_funcname_empty(int ctrl_x_mode_val) { return *get_complete_funcname(ctrl_x_mode_val) == NUL ? 1 : 0; }
void *nvim_get_insert_callback_opaque(int ctrl_x_mode_val) { return (void *)get_insert_callback(ctrl_x_mode_val); }
int nvim_callback_call_findstart(void *cb_opaque)
{
  const int save_State = State;
  typval_T args[3];
  args[0].v_type = VAR_NUMBER; args[1].v_type = VAR_STRING; args[2].v_type = VAR_UNKNOWN;
  args[0].vval.v_number = 1; args[1].vval.v_string = "";
  pos_T pos = curwin->w_cursor;
  textlock++;
  colnr_T col = (colnr_T)callback_call_retnr((Callback *)cb_opaque, 2, args);
  textlock--;
  State = save_State; curwin->w_cursor = pos; check_cursor(curwin); validate_cursor(curwin);
  if (!equalpos(curwin->w_cursor, pos)) { emsg(_(e_compldel)); return INT_MIN; }
  return (int)col;
}

void nvim_ctrl_x_mode_reset_to_normal(void)
  { ctrl_x_mode = CTRL_X_NORMAL; edit_submode = NULL; if (!shortmess(SHM_COMPLETIONMENU)) { msg_clr_cmdline(); } }
void nvim_emit_completefunc_not_set_error(int is_function) { semsg(_(e_notset), is_function ? "completefunc" : "omnifunc"); }
size_t nvim_copy_option_part_iobuff_ffi(char **src) { return copy_option_part(src, IObuff, IOSIZE, ","); }
void nvim_expand_by_function_full_impl(int type, char *base, void *cb_opaque)
{
  Callback *cb = (Callback *)cb_opaque;
  list_T *matchlist = NULL;
  dict_T *matchdict = NULL;
  typval_T rettv;
  const int save_State = State;
  assert(curbuf != NULL);
  const bool is_cpt_function = (cb != NULL);
  if (!is_cpt_function) {
    char *funcname = get_complete_funcname(type);
    if (*funcname == NUL) { return; }
    cb = get_insert_callback(type);
  }
  typval_T args[3];
  args[0].v_type = VAR_NUMBER; args[1].v_type = VAR_STRING; args[2].v_type = VAR_UNKNOWN;
  args[0].vval.v_number = 0; args[1].vval.v_string = base != NULL ? base : "";
  pos_T pos = curwin->w_cursor;
  textlock++;
  if (callback_call(cb, 2, args, &rettv)) {
    switch (rettv.v_type) {
    case VAR_LIST: matchlist = rettv.vval.v_list; break;
    case VAR_DICT: matchdict = rettv.vval.v_dict; break;
    case VAR_SPECIAL: FALLTHROUGH;
    default: tv_clear(&rettv); break;
    }
  }
  textlock--;
  curwin->w_cursor = pos;
  check_cursor(curwin);
  validate_cursor(curwin);
  if (!equalpos(curwin->w_cursor, pos)) { emsg(_(e_compldel)); goto theend; }
  if (matchlist != NULL) { nvim_ins_compl_add_list_impl(matchlist); }
  else if (matchdict != NULL) { nvim_ins_compl_add_dict_impl(matchdict); }
theend:
  State = save_State;
  if (matchdict != NULL) { tv_dict_unref(matchdict); }
  if (matchlist != NULL) { tv_list_unref(matchlist); }
}

int nvim_ins_compl_add_tv_impl(void *tv_opaque, int dir, int fast)
{
  typval_T *tv = (typval_T *)tv_opaque;
  const char *word;
  bool dup = false, empty = false;
  int flags = fast ? CP_FAST : 0;
  char *(cptext[CPT_COUNT]);
  int user_hl[2] = { -1, -1 };
  typval_T user_data;
  user_data.v_type = VAR_UNKNOWN;
  if (tv->v_type == VAR_DICT && tv->vval.v_dict != NULL) {
    word = tv_dict_get_string(tv->vval.v_dict, "word", false);
    cptext[CPT_ABBR] = tv_dict_get_string(tv->vval.v_dict, "abbr", true);
    cptext[CPT_MENU] = tv_dict_get_string(tv->vval.v_dict, "menu", true);
    cptext[CPT_KIND] = tv_dict_get_string(tv->vval.v_dict, "kind", true);
    cptext[CPT_INFO] = tv_dict_get_string(tv->vval.v_dict, "info", true);
    char *abbr_hl = tv_dict_get_string(tv->vval.v_dict, "abbr_hlgroup", false);
    user_hl[0] = (abbr_hl != NULL && *abbr_hl != NUL) ? syn_name2attr(abbr_hl) : -1;
    char *kind_hl = tv_dict_get_string(tv->vval.v_dict, "kind_hlgroup", false);
    user_hl[1] = (kind_hl != NULL && *kind_hl != NUL) ? syn_name2attr(kind_hl) : -1;
    tv_dict_get_tv(tv->vval.v_dict, "user_data", &user_data);
    if (tv_dict_get_number(tv->vval.v_dict, "icase")) { flags |= CP_ICASE; }
    dup = (bool)tv_dict_get_number(tv->vval.v_dict, "dup");
    empty = (bool)tv_dict_get_number(tv->vval.v_dict, "empty");
    if (tv_dict_get_string(tv->vval.v_dict, "equal", false) != NULL
        && tv_dict_get_number(tv->vval.v_dict, "equal")) { flags |= CP_EQUAL; }
  } else {
    word = tv_get_string_chk(tv);
    CLEAR_FIELD(cptext);
  }
  if (word == NULL || (!empty && *word == NUL)) { free_cptext(cptext); tv_clear(&user_data); return FAIL; }
  int status = rs_ins_compl_add((char *)word, -1, NULL, cptext, 1,
                               &user_data, dir, flags, dup ? 1 : 0, user_hl, FUZZY_SCORE_NONE);
  if (status != OK) { tv_clear(&user_data); }
  return status;
}

void nvim_ins_compl_add_list_impl(void *list_opaque)
{
  list_T *list = (list_T *)list_opaque;
  Direction dir = compl_direction;

  TV_LIST_ITER(list, li, {
    if (nvim_ins_compl_add_tv_impl(TV_LIST_ITEM_TV(li), (int)dir, 1) == OK) {
      dir = FORWARD;
    } else if (did_emsg) {
      break;
    }
  });
}

void nvim_ins_compl_add_dict_impl(void *dict_opaque)
{
  dict_T *dict = (dict_T *)dict_opaque;
  // Check for optional "refresh" item.
  compl_opt_refresh_always = false;
  dictitem_T *di_refresh = tv_dict_find(dict, S_LEN("refresh"));
  if (di_refresh != NULL && di_refresh->di_tv.v_type == VAR_STRING) {
    const char *v = di_refresh->di_tv.vval.v_string;
    if (v != NULL && strcmp(v, "always") == 0) {
      compl_opt_refresh_always = true;
    }
  }

  // Add completions from a "words" list.
  dictitem_T *di_words = tv_dict_find(dict, S_LEN("words"));
  if (di_words != NULL && di_words->di_tv.v_type == VAR_LIST) {
    nvim_ins_compl_add_list_impl(di_words->di_tv.vval.v_list);
  }
}

void *nvim_get_callback_if_cpt_func_impl(const char *p, int idx)
{
  if (*p == 'o') {
    return &curbuf->b_ofu_cb;
  }

  if (*p == 'F') {
    const char *q = p + 1;
    if (*q != ',' && *q != NUL) {
      // 'F{func}' case
      return curbuf->b_p_cpt_cb[idx].type != kCallbackNone
             ? &curbuf->b_p_cpt_cb[idx] : NULL;
    } else {
      return &curbuf->b_cfu_cb;  // 'cfu'
    }
  }

  return NULL;
}

buf_T *nvim_buf_get_b_next(buf_T *buf) { return buf->b_next; }
int nvim_buf_get_b_scanned(buf_T *buf) { return buf->b_scanned; }
int nvim_buf_get_b_p_inf_void(void *buf) { return ((buf_T *)buf)->b_p_inf ? 1 : 0; }
win_T *nvim_win_get_w_next(win_T *wp) { return wp->w_next; }
int nvim_win_get_focusable(win_T *wp) { return wp->w_config.focusable ? 1 : 0; }
buf_T *nvim_win_get_w_buffer_raw(win_T *wp) { return wp->w_buffer; }
int nvim_curbuf_get_b_scanned(void) { return curbuf->b_scanned ? 1 : 0; }
const char *nvim_ins_compl_st_get_ins_buf_fname(void) { return ins_compl_st.ins_buf ? ins_compl_st.ins_buf->b_fname : NULL; }
char *nvim_ins_compl_ml_get_buf_at(void *buf, linenr_T lnum, int col) { return ml_get_buf((buf_T *)buf, lnum) + col; }
const char *nvim_ins_compl_st_ins_buf_get_sfname(void) { return (!ins_compl_st.ins_buf || ins_compl_st.ins_buf == curbuf) ? NULL : ins_compl_st.ins_buf->b_sfname; }
void nvim_ins_apply_insertenter(void) { ins_apply_autocmds(EVENT_INSERTENTER); }
void nvim_ins_apply_insertleave(void) { ins_apply_autocmds(EVENT_INSERTLEAVE); }
int nvim_ins_apply_autocmds_insertcharpre(void) { return ins_apply_autocmds(EVENT_INSERTCHARPRE); }
int nvim_get_cpt_first_char(void) { return (unsigned char)*curbuf->b_p_cpt; }
int nvim_get_pum_want_finish(void) { return pum_want.finish ? 1 : 0; }
