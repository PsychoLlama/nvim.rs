// insexpand.c: functions for Insert mode completion

#include <assert.h>
#include <limits.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/executor.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_getln.h"
#include "nvim/extmark.h"
#include "nvim/extmark_defs.h"
#include "nvim/fileio.h"
#include "nvim/fuzzy.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/indent.h"
#include "nvim/indent_c.h"
#include "nvim/insexpand.h"
#include "nvim/keycodes.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/input.h"
#include "nvim/os/time.h"
#include "nvim/path.h"
#include "nvim/popupmenu.h"
#include "nvim/pos_defs.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/register.h"
#include "nvim/search.h"
#include "nvim/spell.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/tag.h"
#include "nvim/textformat.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"
#include "nvim/winfloat.h"

// Rust rs_* function declarations (only those still called from this file)
extern bool rs_set_ref_in_callback(Callback *callback, int copyID, ht_stack_T **ht_stack,
                                   list_stack_T **list_stack);
extern int rs_magic_isset(void);
extern int rs_ctrl_x_mode_normal(void);
extern int rs_ctrl_x_mode_whole_line(void);
extern int rs_ctrl_x_mode_path_defines(void);
extern int rs_ctrl_x_mode_function(void);
extern int rs_ctrl_x_mode_line_or_eval(void);
extern int rs_ctrl_x_mode_not_default(void);
extern int rs_compl_status_adding(void);
extern char *rs_find_word_start(char *ptr);
extern char *rs_find_word_end(char *ptr);
extern char *rs_find_line_end(char *ptr);
extern int rs_compl_dir_forward(void);
extern int rs_cot_fuzzy(void);
extern int rs_is_nearest_active(void);
extern int rs_ins_compl_make_cyclic(void);
extern int rs_get_cpt_sources_count(void);
extern int rs_ins_compl_key2dir(int c);
extern int rs_ins_compl_key2count(int c);
extern void rs_ins_compl_make_linear(void);
extern void rs_ins_compl_clear(void);
extern int rs_ins_compl_interrupted(void);
extern int rs_ins_compl_has_preinsert(void);
extern int rs_ins_compl_preinsert_effect(void);
extern int rs_ins_compl_preinsert_longest(void);
extern const char *rs_ins_compl_leader(void);
extern size_t rs_ins_compl_leader_len(void);
extern unsigned rs_get_cot_flags(void);
extern int rs_ctrl_x_mode_eval(void);
extern void rs_ins_compl_free(void);
extern void rs_strip_caret_numbers_in_place(char *str);
extern unsigned rs_quote_meta(char *dest, char *src, int len);
extern int rs_ins_compl_equal(void *m, const char *str, size_t len);
extern void rs_ins_compl_update_sequence_numbers(void);
extern int rs_ins_compl_col_range_attr(int lnum, int col);
extern void rs_ins_compl_insert(int move_cursor, int insert_prefix);
// rs_ins_ctrl_x deleted: now exported as ins_ctrl_x via #[export_name]
// rs_check_compl_option deleted: now exported as check_compl_option via #[export_name]
extern void rs_ins_compl_check_keys(int frequency, int in_compl_func);
extern void rs_sort_compl_match_list(int compare_type);
extern void rs_ins_compl_new_leader(void);
extern void rs_ins_compl_del_pum(void);
// Phase 1 Rust exports
extern const char *rs_ins_compl_mode(void);
// Phase 3 (pass 5) Rust exports
// Phase 4 (pass 5) Rust exports -- rs_fuzzy_longest_match no longer called from C (Phase 15)
// Phase 1 (pass 5) Rust exports
// Phase 5 (pass 5) Rust exports
// rs_did_set_completefunc deleted: now exported as did_set_completefunc via #[export_name]
// rs_did_set_omnifunc deleted: now exported as did_set_omnifunc via #[export_name]
// rs_did_set_thesaurusfunc deleted: now exported as did_set_thesaurusfunc via #[export_name]
// rs_set_ref_in_insexpand_funcs deleted: now exported as set_ref_in_insexpand_funcs via #[export_name]
// Phase 2 Rust exports
// Phase 3 Rust exports
// Phase 4 (pass 3) Rust exports
// Phase 2 (pass 4) Rust exports
// Phase 3 (pass 4) Rust exports
// Phase 4 (pass 4) Rust exports
extern void rs_compl_source_start_timer(int source_idx);
extern int rs_advance_cpt_sources_index_safe(void);
// Phase 5 (pass 4) Rust exports
// Phase 6 (pass 4) Rust exports
// Phase 1 (pass 6) Rust exports
extern void rs_show_pum(int prev_w_wrow, int prev_w_leftcol);
extern void rs_ins_compl_add_matches(int num_matches, char **matches, int icase);
// Phase 1 (pass 7) Rust exports
extern void rs_save_orig_extmarks(void);
// rs_free_insexpand_stuff deleted: now exported as free_insexpand_stuff via #[export_name]
// Phase 2 (pass 7) Rust exports
extern int rs_compl_get_info(char *line, int startcol, int curs_col, int *line_invalid);
// Phase 3 (pass 7) Rust exports
// Phase 4 (pass 7) Rust exports
// rs_ins_complete deleted: now exported as ins_complete via #[export_name]
// Phase 2 (pass 6) Rust exports
extern void rs_ins_compl_longest_match(void *match);
extern const char *rs_find_common_prefix(size_t *prefix_len, int curbuf_only);
// Phase 1 (pass 11) Rust exports: leader-for-startcol
// Phase 2 (pass 11) Rust exports: ins_compl_build_pum
extern int rs_ins_compl_build_pum(void);
// Phase 3 (pass 11) Rust exports: find_common_prefix (replaces nvim_find_common_prefix_data)
// Phase 3 (pass 6) Rust exports
extern void rs_setup_cpt_sources(void);
extern void rs_prepare_cpt_compl_funcs(void);
extern void rs_get_cpt_func_completion_matches(void *cb_opaque);
// Phase 2 (pass 13) Rust export: get_userdefined_compl_info migration
extern int rs_get_userdefined_compl_info(int curs_col, void *cb_opaque, int *startcol);
// Phase 1 (pass 8) Rust exports
extern int rs_ins_compl_next(int allow_get_expansion, int count, int insert_match);
// Phase 2 (pass 8) Rust exports
extern int rs_ins_compl_get_exp(int lnum, int col);

// Phase 9 (pass 9): Forward declarations for compound C accessors.
// These are defined at the bottom of this file and used by the static
// functions whose bodies were migrated to Rust.
int nvim_ins_compl_add_tv_impl(void *tv, int dir, int fast);
void nvim_ins_compl_add_list_impl(void *list);
void nvim_ins_compl_add_dict_impl(void *dict);
void nvim_expand_by_function_full_impl(int type, char *base, void *cb);
// Phase 9 (pass 9): Phase 2/3/4 compound accessor forward declarations.
void nvim_f_complete_impl(void *argvars, void *rettv);
void nvim_f_complete_add_impl(void *argvars, void *rettv);
void nvim_f_complete_check_impl(void *rettv);
void nvim_f_preinserted_impl(void *rettv);
void nvim_f_complete_info_impl(void *argvars, void *rettv);
void nvim_set_completion_impl(int startcol, void *list);
void nvim_cpt_compl_refresh_impl(void);
void *nvim_get_callback_if_cpt_func_impl(const char *p, int idx);

// Definitions used for CTRL-X submode.
// Note: If you change CTRL-X submode, you must also maintain ctrl_x_msgs[]
// and ctrl_x_mode_names[].

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

#define CTRL_X_MSG(i) ctrl_x_msgs[(i) & ~CTRL_X_WANT_IDENT]

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

static char *ctrl_x_mode_names[] = {
  "keyword",
  "ctrl_x",
  "scroll",
  "whole_line",
  "files",
  "tags",
  "path_patterns",
  "path_defines",
  "unknown",          // CTRL_X_FINISHED
  "dictionary",
  "thesaurus",
  "cmdline",
  "function",
  "omni",
  "spell",
  NULL,               // CTRL_X_LOCAL_MSG only used in "ctrl_x_msgs"
  "eval",
  "cmdline",
  NULL,               // CTRL_X_BUFNAME
  "register",
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

// Static state for ins_compl_get_exp, moved to file scope for accessor visibility.
static ins_compl_next_state_T ins_compl_st;
static bool ins_compl_st_cleared = false;

// In large buffers, timeout may miss nearby matches — search above cursor
#define LOOKBACK_LINE_COUNT     1000

#include "insexpand_shim.c.generated.h"
extern buf_T *rs_ins_compl_next_buf(buf_T *buf, int flag);

/// values for cp_flags
typedef enum {
  CP_ORIGINAL_TEXT = 1,  ///< the original text when the expansion begun
  CP_FREE_FNAME = 2,     ///< cp_fname is allocated
  CP_CONT_S_IPOS = 4,    ///< use CONT_S_IPOS for compl_cont_status
  CP_EQUAL = 8,          ///< rs_ins_compl_equal() always returns true
  CP_ICASE = 16,         ///< ins_compl_equal ignores case
  CP_FAST = 32,          ///< use fast_breakcheck instead of os_breakcheck
} cp_flags_T;

static const char e_hitend[] = N_("Hit end of paragraph");
static const char e_compldel[] = N_("E840: Completion function deleted text");

// All the current matches are stored in a list.
// "compl_first_match" points to the start of the list.
// "compl_curr_match" points to the currently selected entry.
// "compl_shown_match" is different from compl_curr_match during
// ins_compl_get_exp(), when new matches are added to the list.
// "compl_old_match" points to previous "compl_curr_match".

static compl_T *compl_first_match = NULL;
static compl_T *compl_curr_match = NULL;
static compl_T *compl_shown_match = NULL;
static compl_T *compl_old_match = NULL;

/// list used to store the compl_T which have the max score
static compl_T **compl_best_matches = NULL;
static int compl_num_bests = 0;

/// After using a cursor key <Enter> selects a match in the popup menu,
/// otherwise it inserts a line break.
static bool compl_enter_selects = false;

/// When "compl_leader" is not NULL only matches that start with this string
/// are used.
static String compl_leader = STRING_INIT;

static bool compl_get_longest = false;  ///< put longest common string in compl_leader

/// This flag is false when no match is selected (by ^N/^P) or the match was
/// edited or using the longest common string.
static bool compl_used_match;

/// didn't finish finding completions.
static bool compl_was_interrupted = false;

// Set when character typed while looking for matches and it means we should
// stop looking for matches.
static bool compl_interrupted = false;

// compl_restarting: moved to Rust static COMPL_RESTARTING in state.rs (Phase 2)

/// When the first completion is done "compl_started" is set.  When it's
/// false the word to be completed must be located.
static bool compl_started = false;

/// Which Ctrl-X mode are we in?
static int ctrl_x_mode = CTRL_X_NORMAL;

static int compl_matches = 0;           ///< number of completion matches
static String compl_pattern = STRING_INIT;      ///< search pattern for matching items
static String cpt_compl_pattern = STRING_INIT;  ///< pattern returned by func in 'cpt'
static Direction compl_direction = FORWARD;
static Direction compl_shows_dir = FORWARD;
// compl_pending: moved to Rust static COMPL_PENDING in state.rs (Phase 2)
static pos_T compl_startpos;
/// Length in bytes of the text being completed (this is deleted to be replaced
/// by the match.)
static int compl_length = 0;
static linenr_T compl_lnum = 0;         ///< lnum where the completion start
static colnr_T compl_col = 0;           ///< column where the text starts
                                        ///< that is being completed
static colnr_T compl_ins_end_col = 0;
static String compl_orig_text = STRING_INIT;  ///< text as it was before
                                              ///< completion started
/// Undo information to restore extmarks for original text.
static extmark_undo_vec_t compl_orig_extmarks;
static int compl_cont_mode = 0;
static expand_T compl_xp;

static win_T *compl_curr_win = NULL;  ///< win where completion is active
static buf_T *compl_curr_buf = NULL;  ///< buf where completion is active

#define COMPL_INITIAL_TIMEOUT_MS    80
// Autocomplete uses a decaying timeout: starting from COMPL_INITIAL_TIMEOUT_MS,
// if the current source exceeds its timeout, it is interrupted and the next
// begins with half the time. A small minimum timeout ensures every source
// gets at least a brief chance.
// Special case: when 'complete' contains "F" or "o" (function sources), a
// longer fixed timeout is used (COMPL_FUNC_TIMEOUT_MS or
// COMPL_FUNC_TIMEOUT_NON_KW_MS). - girish
static bool compl_autocomplete = false;        ///< whether autocompletion is active
static uint64_t compl_timeout_ms = COMPL_INITIAL_TIMEOUT_MS;
static bool compl_time_slice_expired = false;  ///< time budget exceeded for current source
static bool compl_from_nonkeyword = false;     ///< completion started from non-keyword
// compl_hi_on_autocompl_longest: moved to Rust COMPL_HI_ON_AUTOCOMPL_LONGEST in state.rs (Phase 2)

// Halve the current completion timeout, simulating exponential decay.
#define COMPL_MIN_TIMEOUT_MS    5
#define DECAY_COMPL_TIMEOUT() \
  do { \
    if (compl_timeout_ms > COMPL_MIN_TIMEOUT_MS) { \
      compl_timeout_ms /= 2; \
    } \
  } while (0)

// Timeout values for F{func}, F and o values in 'complete'
#define COMPL_FUNC_TIMEOUT_MS           300
#define COMPL_FUNC_TIMEOUT_NON_KW_MS    1000

// List of flags for method of completion.
static int compl_cont_status = 0;
#define CONT_ADDING    1        ///< "normal" or "adding" expansion
#define CONT_INTRPT    (2 + 4)  ///< a ^X interrupted the current expansion
                                ///< it's set only iff N_ADDS is set
#define CONT_N_ADDS    4        ///< next ^X<> will add-new or expand-current
#define CONT_S_IPOS    8        ///< next ^X<> will set initial_pos?
                                ///< if so, word-wise-expansion will set SOL
#define CONT_SOL       16       ///< pattern includes start of line, just for
                                ///< word-wise expansion, not set for ^X^L
#define CONT_LOCAL     32       ///< for ctrl_x_mode 0, ^X^P/^X^N do a local
                                ///< expansion, (eg use complete=.)

static bool compl_opt_refresh_always = false;

static size_t spell_bad_len = 0;   // length of located bad word

static int compl_selected_item = -1;

// compl_fuzzy_scores deleted (Phase 15): moved to Rust Vec in rs_get_next_filename_completion

/// Define the structure for completion source (in 'cpt' option) information
typedef struct cpt_source_T {
  bool cs_refresh_always;   ///< Whether 'refresh:always' is set for func
  int cs_startcol;          ///< Start column returned by func
  int cs_max_matches;       ///< Max items to display from this source
  uint64_t compl_start_tv;  ///< Timestamp when match collection starts
  char cs_flag;             ///< Flag indicating the type of source
} cpt_source_T;

/// Pointer to the array of completion sources
static cpt_source_T *cpt_sources_array;
/// Total number of completion sources specified in the 'cpt' option
static int cpt_sources_count;
/// Index of the current completion source being expanded
static int cpt_sources_index = -1;

// "compl_match_array" points the currently displayed list of entries in the
// popup menu.  It is NULL when there is no popup menu.
static pumitem_T *compl_match_array = NULL;
static int compl_match_arraysize;

// ins_ctrl_x deleted: Rust exports under the C name directly via #[export_name = "ins_ctrl_x"].

// check_compl_option deleted: Rust exports under the C name directly via #[export_name].

extern int rs_vim_is_ctrl_x_key(int c);
extern int rs_may_advance_cpt_index(const char *cpt);
extern int rs_ins_compl_prep(int c);
extern int rs_ins_compl_bs(void);

/// @return  true if "match" is the original text when the completion began.
static bool match_at_original_text(const compl_T *const match)
{
  return match->cp_flags & CP_ORIGINAL_TEXT;
}

/// @return  true if "match" is the first match in the completion list.
static bool is_first_match(const compl_T *const match)
{
  return match == compl_first_match;
}

/// Get the completed text by inferring the case of the originally typed text.
/// If the result is in allocated memory "tofree" is set to it.
char *nvim_ins_compl_infercase_gettext_impl(const char *str, int char_len, int compl_char_len,
                                            int min_len, char **tofree)
{
  bool has_lower = false;

  // Allocate wide character array for the completion and fill it.
  int *const wca = xmalloc((size_t)char_len * sizeof(*wca));
  {
    const char *p = str;
    for (int i = 0; i < char_len; i++) {
      wca[i] = mb_ptr2char_adv(&p);
    }
  }

  // Rule 1: Were any chars converted to lower?
  {
    const char *p = compl_orig_text.data;
    for (int i = 0; i < min_len; i++) {
      const int c = mb_ptr2char_adv(&p);
      if (mb_islower(c)) {
        has_lower = true;
        if (mb_isupper(wca[i])) {
          // Rule 1 is satisfied.
          for (i = compl_char_len; i < char_len; i++) {
            wca[i] = mb_tolower(wca[i]);
          }
          break;
        }
      }
    }
  }

  // Rule 2: No lower case, 2nd consecutive letter converted to
  // upper case.
  if (!has_lower) {
    bool was_letter = false;
    const char *p = compl_orig_text.data;
    for (int i = 0; i < min_len; i++) {
      const int c = mb_ptr2char_adv(&p);
      if (was_letter && mb_isupper(c) && mb_islower(wca[i])) {
        // Rule 2 is satisfied.
        for (i = compl_char_len; i < char_len; i++) {
          wca[i] = mb_toupper(wca[i]);
        }
        break;
      }
      was_letter = mb_islower(c) || mb_isupper(c);
    }
  }

  // Copy the original case of the part we typed.
  {
    const char *p = compl_orig_text.data;
    for (int i = 0; i < min_len; i++) {
      const int c = mb_ptr2char_adv(&p);
      if (mb_islower(c)) {
        wca[i] = mb_tolower(wca[i]);
      } else if (mb_isupper(c)) {
        wca[i] = mb_toupper(wca[i]);
      }
    }
  }

  // Generate encoding specific output from wide character array.
  garray_T gap;
  char *p = IObuff;
  int i = 0;
  ga_init(&gap, 1, 500);
  while (i < char_len) {
    if (gap.ga_data != NULL) {
      ga_grow(&gap, 10);
      assert(gap.ga_data != NULL);  // suppress clang "Dereference of NULL pointer"
      p = (char *)gap.ga_data + gap.ga_len;
      gap.ga_len += utf_char2bytes(wca[i++], p);
    } else if ((p - IObuff) + 6 >= IOSIZE) {
      // Multi-byte characters can occupy up to five bytes more than
      // ASCII characters, and we also need one byte for NUL, so when
      // getting to six bytes from the edge of IObuff switch to using a
      // growarray.  Add the character in the next round.
      ga_grow(&gap, IOSIZE);
      *p = NUL;
      STRCPY(gap.ga_data, IObuff);
      gap.ga_len = (int)(p - IObuff);
    } else {
      p += utf_char2bytes(wca[i++], p);
    }
  }
  xfree(wca);

  if (gap.ga_data != NULL) {
    *tofree = gap.ga_data;
    return gap.ga_data;
  }

  *p = NUL;
  return IObuff;
}

/// This is like ins_compl_add(), but if 'ic' and 'inf' are set, then the
/// case of the originally typed text is used, and the case of the completed
/// text is inferred, ie this tries to work out what case you probably wanted
/// the rest of the word to be in -- webb
///
/// @param[in]  cont_s_ipos  next ^X<> will set initial_pos
int ins_compl_add_infercase(char *str_arg, int len, bool icase, char *fname, Direction dir,
                            bool cont_s_ipos, int score)
  FUNC_ATTR_NONNULL_ARG(1)
{
  char *str = str_arg;
  int char_len;  // count multi-byte characters
  int compl_char_len;
  int flags = 0;
  char *tofree = NULL;

  if (p_ic && curbuf->b_p_inf && len > 0) {
    // Infer case of completed part.

    // Find actual length of completion.
    {
      const char *p = str;
      char_len = 0;
      while (*p != NUL) {
        MB_PTR_ADV(p);
        char_len++;
      }
    }

    // Find actual length of original text.
    {
      const char *p = compl_orig_text.data;
      compl_char_len = 0;
      while (*p != NUL) {
        MB_PTR_ADV(p);
        compl_char_len++;
      }
    }

    // "char_len" may be smaller than "compl_char_len" when using
    // thesaurus, only use the minimum when comparing.
    int min_len = MIN(char_len, compl_char_len);

    str = nvim_ins_compl_infercase_gettext_impl(str, char_len, compl_char_len, min_len, &tofree);
  }
  if (cont_s_ipos) {
    flags |= CP_CONT_S_IPOS;
  }
  if (icase) {
    flags |= CP_ICASE;
  }

  int res = ins_compl_add(str, len, fname, NULL, false, NULL, dir, flags, false, NULL, score);
  xfree(tofree);
  return res;
}

/// free cptext
static inline void free_cptext(char *const *const cptext)
{
  if (cptext != NULL) {
    for (size_t i = 0; i < CPT_COUNT; i++) {
      xfree(cptext[i]);
    }
  }
}

/// Add a match to the list of matches
///
/// @param[in]  str     text of the match to add
/// @param[in]  len     length of "str". If -1, then the length of "str" is computed.
/// @param[in]  fname   file name to associate with this match. May be NULL.
/// @param[in]  cptext  list of strings to use with this match (for abbr, menu, info
///                     and kind). May be NULL.
///                     If not NULL, must have exactly #CPT_COUNT items.
/// @param[in]  cptext_allocated  If true, will not copy cptext strings.
///
///                               @note Will free strings in case of error.
///                                     cptext itself will not be freed.
/// @param[in]  user_data  user supplied data (any vim type) for this match
/// @param[in]  cdir       match direction. If 0, use "compl_direction".
/// @param[in]  flags_arg  match flags (cp_flags)
/// @param[in]  adup       accept this match even if it is already present.
/// @param[in]  user_hl    list of extra highlight attributes for abbr kind.
///
/// If "cdir" is FORWARD, then the match is added after the current match.
/// Otherwise, it is added before the current match.
///
/// @return NOTDONE if the given string is already in the list of completions,
///         otherwise it is added to the list and  OK is returned. FAIL will be
///         returned in case of error.
static int ins_compl_add(char *const str, int len, char *const fname, char *const *const cptext,
                         const bool cptext_allocated, typval_T *user_data, const Direction cdir,
                         int flags_arg, const bool adup, const int *user_hl, const int score)
  FUNC_ATTR_NONNULL_ARG(1)
{
  compl_T *match;
  const Direction dir = (cdir == kDirectionNotSet ? compl_direction : cdir);
  int flags = flags_arg;
  bool inserted = false;

  if (flags & CP_FAST) {
    fast_breakcheck();
  } else {
    os_breakcheck();
  }
  if (got_int) {
    if (cptext_allocated) {
      free_cptext(cptext);
    }
    return FAIL;
  }
  if (len < 0) {
    len = (int)strlen(str);
  }

  // If the same match is already present, don't add it.
  if (compl_first_match != NULL && !adup) {
    match = compl_first_match;
    do {
      if (!match_at_original_text(match)
          && strncmp(match->cp_str.data, str, (size_t)len) == 0
          && ((int)match->cp_str.size <= len || match->cp_str.data[len] == NUL)) {
        if (rs_is_nearest_active() && score > 0 && score < match->cp_score) {
          match->cp_score = score;
        }
        if (cptext_allocated) {
          free_cptext(cptext);
        }
        return NOTDONE;
      }
      match = match->cp_next;
    } while (match != NULL && !is_first_match(match));
  }

  // Remove any popup menu before changing the list of matches.
  rs_ins_compl_del_pum();

  // Allocate a new match structure.
  // Copy the values to the new match structure.
  match = xcalloc(1, sizeof(compl_T));
  match->cp_number = flags & CP_ORIGINAL_TEXT ? 0 : -1;
  match->cp_str = cbuf_to_string(str, (size_t)len);

  // match-fname is:
  // - compl_curr_match->cp_fname if it is a string equal to fname.
  // - a copy of fname, CP_FREE_FNAME is set to free later THE allocated mem.
  // - NULL otherwise.  --Acevedo
  if (fname != NULL
      && compl_curr_match != NULL
      && compl_curr_match->cp_fname != NULL
      && strcmp(fname, compl_curr_match->cp_fname) == 0) {
    match->cp_fname = compl_curr_match->cp_fname;
  } else if (fname != NULL) {
    match->cp_fname = xstrdup(fname);
    flags |= CP_FREE_FNAME;
  } else {
    match->cp_fname = NULL;
  }
  match->cp_flags = flags;
  match->cp_user_abbr_hlattr = user_hl ? user_hl[0] : -1;
  match->cp_user_kind_hlattr = user_hl ? user_hl[1] : -1;
  match->cp_score = score;
  match->cp_cpt_source_idx = cpt_sources_index;

  if (cptext != NULL) {
    for (int i = 0; i < CPT_COUNT; i++) {
      if (cptext[i] == NULL) {
        continue;
      }
      if (*cptext[i] != NUL) {
        match->cp_text[i] = (cptext_allocated ? cptext[i] : xstrdup(cptext[i]));
      } else if (cptext_allocated) {
        xfree(cptext[i]);
      }
    }
  }

  if (user_data != NULL) {
    match->cp_user_data = *user_data;
  }

  // Link the new match structure after (FORWARD) or before (BACKWARD) the
  // current match in the list of matches .
  if (compl_first_match == NULL) {
    match->cp_next = match->cp_prev = NULL;
  } else if (rs_cot_fuzzy() && score != FUZZY_SCORE_NONE && compl_get_longest) {
    compl_T *current = compl_first_match->cp_next;
    compl_T *prev = compl_first_match;
    inserted = false;
    // The direction is ignored when using longest and fuzzy match, because
    // matches are inserted and sorted by score.
    while (current != NULL && current != compl_first_match) {
      if (current->cp_score < score) {
        match->cp_next = current;
        match->cp_prev = current->cp_prev;
        if (current->cp_prev) {
          current->cp_prev->cp_next = match;
        }
        current->cp_prev = match;
        inserted = true;
        break;
      }
      prev = current;
      current = current->cp_next;
    }
    if (!inserted) {
      prev->cp_next = match;
      match->cp_prev = prev;
      match->cp_next = compl_first_match;
      compl_first_match->cp_prev = match;
    }
  } else if (dir == FORWARD) {
    match->cp_next = compl_curr_match->cp_next;
    match->cp_prev = compl_curr_match;
  } else {    // BACKWARD
    match->cp_next = compl_curr_match;
    match->cp_prev = compl_curr_match->cp_prev;
  }
  if (match->cp_next) {
    match->cp_next->cp_prev = match;
  }
  if (match->cp_prev) {
    match->cp_prev->cp_next = match;
  } else {        // if there's nothing before, it is the first match
    compl_first_match = match;
  }
  compl_curr_match = match;

  // Find the longest common string if still doing that.
  if (compl_get_longest && (flags & CP_ORIGINAL_TEXT) == 0 && !rs_cot_fuzzy()
      && !rs_ins_compl_preinsert_longest()) {
    rs_ins_compl_longest_match(match);
  }

  return OK;
}

// NOTE: ins_compl_col_range_attr deleted (Phase 15).
// drawline.c already calls rs_ins_compl_col_range_attr() directly.


/// Convert to complete item dict
static dict_T *ins_compl_dict_alloc(compl_T *match)
{
  // { word, abbr, menu, kind, info }
  dict_T *dict = tv_dict_alloc_lock(VAR_FIXED);
  tv_dict_add_str(dict, S_LEN("word"), match->cp_str.data);
  tv_dict_add_str(dict, S_LEN("abbr"), match->cp_text[CPT_ABBR]);
  tv_dict_add_str(dict, S_LEN("menu"), match->cp_text[CPT_MENU]);
  tv_dict_add_str(dict, S_LEN("kind"), match->cp_text[CPT_KIND]);
  tv_dict_add_str(dict, S_LEN("info"), match->cp_text[CPT_INFO]);
  if (match->cp_user_data.v_type == VAR_UNKNOWN) {
    tv_dict_add_str(dict, S_LEN("user_data"), "");
  } else {
    tv_dict_add_tv(dict, S_LEN("user_data"), &match->cp_user_data);
  }
  return dict;
}

/// Trigger the CompleteChanged autocmd event. Invoked each time the Insert mode
/// completion menu is changed.
static void trigger_complete_changed_event(int cur)
{
  static bool recursive = false;
  save_v_event_T save_v_event;

  if (recursive) {
    return;
  }

  dict_T *item = cur < 0 ? tv_dict_alloc() : ins_compl_dict_alloc(compl_curr_match);
  dict_T *v_event = get_v_event(&save_v_event);
  tv_dict_add_dict(v_event, S_LEN("completed_item"), item);
  pum_set_event_info(v_event);
  tv_dict_set_keys_readonly(v_event);

  recursive = true;
  textlock++;
  apply_autocmds(EVENT_COMPLETECHANGED, NULL, NULL, false, curbuf);
  textlock--;
  recursive = false;

  restore_v_event(v_event, &save_v_event);
}

// Helper functions for nvim_mergesort_compl_list_raw() -- C function pointer
// callbacks required by mergesort_list(); sort logic lives in rs_sort_compl_match_list.

static void *cp_get_next(void *node)
{
  return ((compl_T *)node)->cp_next;
}

static void cp_set_next(void *node, void *next)
{
  ((compl_T *)node)->cp_next = (compl_T *)next;
}

static void *cp_get_prev(void *node)
{
  return ((compl_T *)node)->cp_prev;
}

static void cp_set_prev(void *node, void *prev)
{
  ((compl_T *)node)->cp_prev = (compl_T *)prev;
}

static int cp_compare_fuzzy(const void *a, const void *b)
{
  int score_a = ((compl_T *)a)->cp_score;
  int score_b = ((compl_T *)b)->cp_score;
  return (score_b > score_a) ? 1 : (score_b < score_a) ? -1 : 0;
}

static int cp_compare_nearest(const void *a, const void *b)
{
  int score_a = ((compl_T *)a)->cp_score;
  int score_b = ((compl_T *)b)->cp_score;
  if (score_a == FUZZY_SCORE_NONE || score_b == FUZZY_SCORE_NONE) {
    return 0;
  }
  return (score_a > score_b) ? 1 : (score_a < score_b) ? -1 : 0;
}

#define DICT_FIRST      (1)     ///< use just first element in "dict"
#define DICT_EXACT      (2)     ///< "dict" is the exact name of a file

/// Add any identifiers that match the given pattern "pat" in the list of
/// dictionary files "dict_start" to the list of completions.
///
/// @param flags      DICT_FIRST and/or DICT_EXACT
/// @param thesaurus  Thesaurus completion
/// Compound accessor for Phase 3 (pass 5): full ins_compl_dictionaries logic
/// (previously ins_compl_dictionaries + ins_compl_files + thesaurus_add_words_in_line).
/// Uses compl_pattern.data as the pattern.
void nvim_ins_compl_dictionaries_impl(const char *dict_start, int flags, int thesaurus)
{
  char *dict = (char *)dict_start;
  const char *pat = compl_pattern.data;
  char *ptr;
  regmatch_T regmatch;
  char **files;
  int count;
  Direction dir = compl_direction;

  if (*dict == NUL) {
    if (!thesaurus && curwin->w_p_spell) {
      dict = "spell";
    } else {
      return;
    }
  }

  char *buf = xmalloc(LSIZE);
  regmatch.regprog = NULL;

  int save_p_scs = p_scs;
  if (curbuf->b_p_inf) {
    p_scs = false;
  }

  if (rs_ctrl_x_mode_line_or_eval()) {
    char *pat_esc = vim_strsave_escaped(pat, "\\");
    size_t len = strlen(pat_esc) + 10;
    ptr = xmalloc(len);
    vim_snprintf(ptr, len, "^\\s*\\zs\\V%s", pat_esc);
    regmatch.regprog = vim_regcomp(ptr, RE_MAGIC);
    xfree(pat_esc);
    xfree(ptr);
  } else {
    regmatch.regprog = vim_regcomp(pat, rs_magic_isset() ? RE_MAGIC : 0);
    if (regmatch.regprog == NULL) {
      goto theend;
    }
  }

  regmatch.rm_ic = ignorecase(pat);
  while (*dict != NUL && !got_int && !compl_interrupted) {
    if (flags == DICT_EXACT) {
      count = 1;
      files = &dict;
    } else {
      copy_option_part(&dict, buf, LSIZE, ",");
      if (!thesaurus && strcmp(buf, "spell") == 0) {
        count = -1;
      } else if (vim_strchr(buf, '`') != NULL
                 || expand_wildcards(1, &buf, &count, &files,
                                     EW_FILE|EW_SILENT) != OK) {
        count = 0;
      }
    }

    if (count == -1) {
      if (pat[0] == '\\' && pat[1] == '<') {
        ptr = pat + 2;
      } else {
        ptr = (char *)pat;
      }
      spell_dump_compl(ptr, regmatch.rm_ic, &dir, 0);
    } else if (count > 0) {
      // ins_compl_files inlined:
      char *leader = rs_cot_fuzzy() ? (char *)rs_ins_compl_leader() : NULL;
      int leader_len = rs_cot_fuzzy() ? (int)rs_ins_compl_leader_len() : 0;
      for (int i = 0; i < count && !got_int && !rs_ins_compl_interrupted(); i++) {
        FILE *fp = os_fopen(files[i], "r");
        if (flags != DICT_EXACT && !shortmess(SHM_COMPLETIONSCAN) && !compl_autocomplete) {
          msg_hist_off = true;
          msg_ext_set_kind("completion");
          vim_snprintf(IObuff, IOSIZE, _("Scanning dictionary: %s"), files[i]);
          msg_trunc(IObuff, true, HLF_R);
        }
        if (fp == NULL) {
          continue;
        }
        while (!got_int && !rs_ins_compl_interrupted() && !vim_fgets(buf, LSIZE, fp)) {
          char *lptr = buf;
          if (rs_cot_fuzzy() && leader_len > 0) {
            char *line_end = rs_find_line_end(lptr);
            while (lptr < line_end) {
              int score = 0;
              int len = 0;
              if (fuzzy_match_str_in_line(&lptr, leader, &len, NULL, &score)) {
                char *end_ptr = rs_ctrl_x_mode_line_or_eval()
                                ? rs_find_line_end(lptr) : rs_find_word_end(lptr);
                int add_r = ins_compl_add_infercase(lptr, (int)(end_ptr - lptr),
                                                    p_ic, files[i], dir, false, score);
                if (add_r == FAIL) {
                  break;
                }
                lptr = end_ptr;
                if (compl_get_longest && rs_ctrl_x_mode_normal()
                    && compl_first_match->cp_next
                    && score == compl_first_match->cp_next->cp_score) {
                  compl_num_bests++;
                }
              }
            }
          } else if (regmatch.regprog != NULL) {
            while (vim_regexec(&regmatch, buf, (colnr_T)(lptr - buf))) {
              lptr = regmatch.startp[0];
              lptr = rs_ctrl_x_mode_line_or_eval() ? rs_find_line_end(lptr)
                                                   : rs_find_word_end(lptr);
              int add_r = ins_compl_add_infercase(regmatch.startp[0],
                                                  (int)(lptr - regmatch.startp[0]),
                                                  p_ic, files[i], dir, false,
                                                  FUZZY_SCORE_NONE);
              if (thesaurus) {
                // thesaurus_add_words_in_line inlined:
                lptr = buf;
                while (!got_int) {
                  lptr = rs_find_word_start(lptr);
                  if (*lptr == NUL || *lptr == NL) {
                    break;
                  }
                  char *wstart = lptr;
                  while (*lptr != NUL) {
                    const int l = utfc_ptr2len(lptr);
                    if (l < 2 && !vim_iswordc((uint8_t)(*lptr))) {
                      break;
                    }
                    lptr += l;
                  }
                  if (wstart != regmatch.startp[0]) {
                    add_r = ins_compl_add_infercase(wstart, (int)(lptr - wstart), p_ic,
                                                    files[i], dir, false, FUZZY_SCORE_NONE);
                    if (add_r == FAIL) {
                      break;
                    }
                  }
                }
              }
              if (add_r == OK) {
                dir = FORWARD;
              } else if (add_r == FAIL) {
                break;
              }
              if (*lptr == '\n' || got_int) {
                break;
              }
            }
          }
          line_breakcheck();
          rs_ins_compl_check_keys(50, 0);
        }
        fclose(fp);
      }
      if (flags != DICT_EXACT) {
        FreeWild(count, files);
      }
    }
    if (flags != 0) {
      break;
    }
  }

theend:
  p_scs = save_p_scs;
  vim_regfree(regmatch.regprog);
  xfree(buf);
}

/// Compound accessor: returns the effective thesaurus option string.
/// Returns curbuf->b_p_tsr if non-empty, else p_tsr.
const char *nvim_get_curbuf_b_p_tsr(void)
{
  return *curbuf->b_p_tsr == NUL ? p_tsr : curbuf->b_p_tsr;
}

/// Compound accessor: returns the effective dictionary option string.
/// Returns curbuf->b_p_dict if non-empty, else p_dict.
const char *nvim_get_curbuf_b_p_dict(void)
{
  return *curbuf->b_p_dict == NUL ? p_dict : curbuf->b_p_dict;
}

/// Compound accessor: calls expand_by_function(type, compl_pattern.data, NULL).
void nvim_expand_by_function_impl(int compl_type)
{
  expand_by_function(compl_type, compl_pattern.data, NULL);
}

/// Free a completion item in the list
static void ins_compl_item_free(compl_T *match)
{
  API_CLEAR_STRING(match->cp_str);
  // several entries may use the same fname, free it just once.
  if (match->cp_flags & CP_FREE_FNAME) {
    xfree(match->cp_fname);
  }
  free_cptext(match->cp_text);
  tv_clear(&match->cp_user_data);
  xfree(match);
}

// C accessors for completion state (used by Rust)
int nvim_get_compl_interrupted(void) { return compl_interrupted ? 1 : 0; }
int nvim_get_compl_time_slice_expired(void) { return compl_time_slice_expired ? 1 : 0; }
int nvim_get_compl_enter_selects(void) { return compl_enter_selects ? 1 : 0; }
int nvim_get_compl_used_match(void) { return compl_used_match ? 1 : 0; }
int nvim_get_compl_length(void) { return compl_length; }
int nvim_get_compl_was_interrupted(void) { return compl_was_interrupted ? 1 : 0; }
int nvim_get_compl_opt_refresh_always(void) { return compl_opt_refresh_always ? 1 : 0; }
win_T *nvim_get_compl_curr_win(void) { return compl_curr_win; }
buf_T *nvim_get_compl_curr_buf(void) { return compl_curr_buf; }
int nvim_get_compl_col(void) { return compl_col; }

// Match list accessors for Rust (using void* since compl_T is static)
void *nvim_compl_get_first_match(void) { return compl_first_match; }
void nvim_compl_set_first_match(void *m) { compl_first_match = (compl_T *)m; }
void *nvim_compl_get_curr_match(void) { return compl_curr_match; }
void nvim_compl_set_curr_match(void *m) { compl_curr_match = (compl_T *)m; }
void *nvim_compl_get_shown_match(void) { return compl_shown_match; }
void nvim_compl_set_shown_match(void *m) { compl_shown_match = (compl_T *)m; }
void *nvim_compl_get_old_match(void) { return compl_old_match; }
void nvim_compl_set_old_match(void *m) { compl_old_match = (compl_T *)m; }

// Match node accessors for Rust (using void* since compl_T is static)
void *nvim_compl_match_get_next(void *m) { return m ? ((compl_T *)m)->cp_next : NULL; }
void nvim_compl_match_set_next(void *m, void *next) { if (m) ((compl_T *)m)->cp_next = (compl_T *)next; }
void *nvim_compl_match_get_prev(void *m) { return m ? ((compl_T *)m)->cp_prev : NULL; }
void nvim_compl_match_set_prev(void *m, void *prev) { if (m) ((compl_T *)m)->cp_prev = (compl_T *)prev; }
int nvim_compl_match_get_flags(void *m) { return m ? ((compl_T *)m)->cp_flags : 0; }
int nvim_compl_match_get_score(void *m) { return m ? ((compl_T *)m)->cp_score : -1; }
int nvim_compl_is_first_match(void *m) { return m == compl_first_match ? 1 : 0; }
int nvim_compl_match_at_original_text(void *m) { return (m && (((compl_T *)m)->cp_flags & CP_ORIGINAL_TEXT)) ? 1 : 0; }
int nvim_compl_match_get_cpt_source_idx(void *m) { return m ? ((compl_T *)m)->cp_cpt_source_idx : -1; }
int nvim_compl_match_get_in_match_array(void *m) { return (m && ((compl_T *)m)->cp_in_match_array) ? 1 : 0; }
// Phase 2 (pass 11): new match field accessors for ins_compl_build_pum migration
void nvim_compl_match_set_in_match_array(void *m, int val) { if (m) ((compl_T *)m)->cp_in_match_array = (val != 0); }
void *nvim_compl_match_get_match_next(void *m) { return m ? ((compl_T *)m)->cp_match_next : NULL; }
void nvim_compl_match_set_match_next(void *m, void *next) { if (m) ((compl_T *)m)->cp_match_next = (compl_T *)next; }
void nvim_compl_match_clear_icase(void *m) { if (m) ((compl_T *)m)->cp_flags &= ~CP_ICASE; }
int nvim_get_compl_match_arraysize(void) { return compl_match_arraysize; }
void nvim_set_compl_match_arraysize(int val) { compl_match_arraysize = val; }
int nvim_compl_leader_eq_orig_text(void) {
  return (compl_leader.data && compl_orig_text.data
          && strequal(compl_leader.data, compl_orig_text.data)) ? 1 : 0;
}
void nvim_set_compl_shown_to_first_or_next(int no_select) {
  compl_shown_match = no_select ? compl_first_match : compl_first_match->cp_next;
}
/// Build and fill compl_match_array from the cp_match_next linked list.
/// Allocates compl_match_array[0..count-1] and populates pumitem_T fields.
/// Returns the count of filled entries (same as count parameter).
// nvim_xmalloc_ints: deleted (Phase 3, Rust calls xmalloc directly)
int nvim_build_pum_fill_array(void *match_head_void, int count) {
  compl_T *match_head = (compl_T *)match_head_void;
  assert(count >= 0);
  compl_match_array = xcalloc((size_t)count, sizeof(pumitem_T));
  int i = 0;
  compl_T *comp = match_head;
  while (comp != NULL) {
    compl_match_array[i].pum_text = comp->cp_text[CPT_ABBR] != NULL
                                    ? comp->cp_text[CPT_ABBR] : comp->cp_str.data;
    compl_match_array[i].pum_kind = comp->cp_text[CPT_KIND];
    compl_match_array[i].pum_info = comp->cp_text[CPT_INFO];
    compl_match_array[i].pum_cpt_source_idx = comp->cp_cpt_source_idx;
    compl_match_array[i].pum_user_abbr_hlattr = comp->cp_user_abbr_hlattr;
    compl_match_array[i].pum_user_kind_hlattr = comp->cp_user_kind_hlattr;
    compl_match_array[i++].pum_extra = comp->cp_text[CPT_MENU] != NULL
                                       ? comp->cp_text[CPT_MENU] : comp->cp_fname;
    compl_T *match_next = comp->cp_match_next;
    comp->cp_match_next = NULL;
    comp = match_next;
  }
  return i;
}
/// Find the shown match in the compl_match_array by pointer identity.
/// Returns the index, or -1 if not found.
int nvim_find_shown_match_in_match_array(void) {
  if (!compl_match_array || !compl_shown_match) { return -1; }
  for (int i = 0; i < compl_match_arraysize; i++) {
    if (compl_match_array[i].pum_text == compl_shown_match->cp_str.data
        || compl_match_array[i].pum_text == compl_shown_match->cp_text[CPT_ABBR]) {
      return i;
    }
  }
  return -1;
}

// Memory operations for Rust
void nvim_compl_item_free(void *m) { if (m) ins_compl_item_free((compl_T *)m); }
void nvim_compl_clear_pattern(void) { API_CLEAR_STRING(compl_pattern); }
void nvim_compl_clear_leader(void) { API_CLEAR_STRING(compl_leader); }
// NOTE: nvim_ins_compl_del_pum deleted (Phase 15). Rust calls rs_ins_compl_del_pum() directly.
// nvim_pum_clear: deleted (Phase 3, Rust calls pum_clear directly)
int nvim_get_compl_match_array_exists(void) { return compl_match_array != NULL ? 1 : 0; }

// Completion state accessors (used by Rust insexpand crate)
int nvim_compl_match_get_cp_number(void *m) { return m ? ((compl_T *)m)->cp_number : -1; }
void nvim_compl_match_set_cp_number(void *m, int num) { if (m) ((compl_T *)m)->cp_number = num; }
const char *nvim_curbuf_get_b_p_cpt(void) { return curbuf->b_p_cpt; }
uint64_t nvim_get_cpt_start_tv(void) { return cpt_sources_array[cpt_sources_index].compl_start_tv; }
void nvim_set_cpt_sources_start_tv(int idx, uint64_t ts) { cpt_sources_array[idx].compl_start_tv = ts; }
uint64_t nvim_get_compl_timeout_ms(void) { return compl_timeout_ms; }
void nvim_set_compl_time_slice_expired(int val) { compl_time_slice_expired = val != 0; }
void nvim_decay_compl_timeout(void) { DECAY_COMPL_TIMEOUT(); }

void nvim_set_compl_cont_status(int val) { compl_cont_status = val; }
void nvim_set_compl_started(int val) { compl_started = val != 0; }
void nvim_set_compl_matches(int val) { compl_matches = val; }
void nvim_set_compl_selected_item(int val) { compl_selected_item = val; }
void nvim_set_compl_ins_end_col(int val) { compl_ins_end_col = val; }
void nvim_clear_compl_curr_win(void) { compl_curr_win = NULL; }
void nvim_clear_compl_curr_buf(void) { compl_curr_buf = NULL; }
void nvim_set_compl_enter_selects(int val) { compl_enter_selects = val != 0; }
void nvim_set_compl_autocomplete(int val) { compl_autocomplete = val != 0; }
void nvim_set_compl_get_longest(int val) { compl_get_longest = val != 0; }
void nvim_set_compl_from_nonkeyword(int val) { compl_from_nonkeyword = val != 0; }
void nvim_set_compl_num_bests(int val) { compl_num_bests = val; }
void nvim_clear_edit_submode_extra(void) { edit_submode_extra = NULL; }
void nvim_clear_compl_orig_extmarks(void) { kv_destroy(compl_orig_extmarks); }
void nvim_compl_clear_orig_text(void) { API_CLEAR_STRING(compl_orig_text); }
void nvim_cpt_sources_clear(void) { XFREE_CLEAR(cpt_sources_array); cpt_sources_index = -1; cpt_sources_count = 0; }
void nvim_set_completed_item_empty(void) { set_vim_var_dict(VV_COMPLETED_ITEM, tv_dict_alloc_lock(VAR_FIXED)); }

void nvim_append_char_to_redobuff(int c) { AppendCharToRedobuff(c); }
void nvim_append_to_redobuff_lit(const char *s, int len) { AppendToRedobuffLit(s, len); }
int nvim_utf_head_off(const char *base, const char *p) { return utf_head_off(base, p); }
void nvim_compl_match_set_score(void *m, int score) { if (m) { ((compl_T *)m)->cp_score = score; } }
const char *nvim_compl_match_get_cp_str_data(void *m) { return m ? ((compl_T *)m)->cp_str.data : NULL; }
size_t nvim_compl_match_get_cp_str_size(void *m) { return m ? ((compl_T *)m)->cp_str.size : 0; }
int nvim_vim_strnicmp(const char *s1, const char *s2, size_t len) { return STRNICMP(s1, s2, len); }
int nvim_fuzzy_match_str(char *str, const char *pat) { return fuzzy_match_str(str, pat); }

_Static_assert(-(('k') + (('b') << 8)) == -25195, "K_BS value mismatch");

// NOTE: ins_compl_bs deleted (Phase 15). edit.c now calls rs_ins_compl_bs() directly.

// NOTE: ins_compl_next_buf migrated to Rust as rs_ins_compl_next_buf (Phase 14).

static Callback cfu_cb;    ///< 'completefunc' callback function
static Callback ofu_cb;    ///< 'omnifunc' callback function
static Callback tsrfu_cb;  ///< 'thesaurusfunc' callback function
static Callback *cpt_cb;   ///< Callback functions associated with F{func}
static int cpt_cb_count;   ///< Number of cpt callbacks

/// Copy a global callback function to a buffer local callback.
static void copy_global_to_buflocal_cb(Callback *globcb, Callback *bufcb)
{
  callback_free(bufcb);
  if (globcb->type != kCallbackNone) {
    callback_copy(bufcb, globcb);
  }
}

// Phase 5 (pass 5): Rust wrappers for callback management functions
/// Compound accessor for Phase 5 (pass 5): did_set_completefunc implementation.
const char *nvim_did_set_completefunc_impl(void *args_v)
{
  optset_T *args = (optset_T *)args_v;
  buf_T *buf = (buf_T *)args->os_buf;
  if (option_set_callback_func(buf->b_p_cfu, &cfu_cb) == FAIL) {
    return e_invarg;
  }
  set_buflocal_cfu_callback(buf);
  return NULL;
}

// did_set_completefunc deleted: now exported from Rust userfunc.rs via #[export_name]

/// Copy the global 'completefunc' callback function to the buffer-local
/// 'completefunc' callback for "buf".
void set_buflocal_cfu_callback(buf_T *buf)
{
  copy_global_to_buflocal_cb(&cfu_cb, &buf->b_cfu_cb);
}

/// Compound accessor for Phase 5 (pass 5): did_set_omnifunc implementation.
const char *nvim_did_set_omnifunc_impl(void *args_v)
{
  optset_T *args = (optset_T *)args_v;
  buf_T *buf = (buf_T *)args->os_buf;
  if (option_set_callback_func(buf->b_p_ofu, &ofu_cb) == FAIL) {
    return e_invarg;
  }
  set_buflocal_ofu_callback(buf);
  return NULL;
}

// did_set_omnifunc deleted: now exported from Rust userfunc.rs via #[export_name]

/// Copy the global 'omnifunc' callback function to the buffer-local 'omnifunc'
/// callback for "buf".
void set_buflocal_ofu_callback(buf_T *buf)
{
  copy_global_to_buflocal_cb(&ofu_cb, &buf->b_ofu_cb);
}

/// Free an array of 'complete' F{func} callbacks and set the pointer to NULL.
void clear_cpt_callbacks(Callback **callbacks, int count)
{
  if (callbacks == NULL || *callbacks == NULL) {
    return;
  }

  for (int i = 0; i < count; i++) {
    callback_free(&(*callbacks)[i]);
  }

  XFREE_CLEAR(*callbacks);
}

/// Copies a list of Callback structs from src to *dest, clearing any existing
/// entries and allocating memory for the destination.
static void copy_cpt_callbacks(Callback **dest, int *dest_cnt, Callback *src, int cnt)
{
  if (cnt == 0) {
    return;
  }

  clear_cpt_callbacks(dest, *dest_cnt);
  *dest = xcalloc((size_t)cnt, sizeof(Callback));
  *dest_cnt = cnt;

  for (int i = 0; i < cnt; i++) {
    if (src[i].type != kCallbackNone) {
      callback_copy(&(*dest)[i], &src[i]);
    }
  }
}

/// Copy global 'complete' F{func} callbacks into the given buffer's local
/// callback array. Clears any existing buffer-local callbacks first.
void set_buflocal_cpt_callbacks(buf_T *buf)
{
  if (buf == NULL || cpt_cb_count == 0) {
    return;
  }
  copy_cpt_callbacks(&buf->b_p_cpt_cb, &buf->b_p_cpt_count, cpt_cb, cpt_cb_count);
}

/// Parse 'complete' option and initialize F{func} callbacks.
/// Frees any existing callbacks and allocates new ones.
/// Only F{func} entries are processed; others are ignored.
int set_cpt_callbacks(optset_T *args)
{
  bool local = (args->os_flags & OPT_LOCAL) != 0;

  if (curbuf == NULL) {
    return FAIL;
  }

  clear_cpt_callbacks(&curbuf->b_p_cpt_cb, curbuf->b_p_cpt_count);
  curbuf->b_p_cpt_count = 0;

  int count = rs_get_cpt_sources_count();
  if (count == 0) {
    return OK;
  }

  curbuf->b_p_cpt_cb = xcalloc((size_t)count, sizeof(Callback));
  curbuf->b_p_cpt_count = count;

  char buf[LSIZE];
  int idx = 0;
  for (char *p = curbuf->b_p_cpt; *p != NUL;) {
    while (*p == ',' || *p == ' ') {
      p++;  // Skip delimiters
    }
    if (*p != NUL) {
      size_t slen = copy_option_part(&p, buf, LSIZE, ",");  // Advance p
      if (slen > 0 && buf[0] == 'F' && buf[1] != NUL) {
        char *caret = vim_strchr(buf, '^');
        if (caret != NULL) {
          *caret = NUL;
        }
        if (option_set_callback_func(buf + 1, &curbuf->b_p_cpt_cb[idx]) != OK) {
          curbuf->b_p_cpt_cb[idx].type = kCallbackNone;
        }
      }
      idx++;
    }
  }

  if (!local) {  // ':set' used instead of ':setlocal'
    // Cache the callback array
    copy_cpt_callbacks(&cpt_cb, &cpt_cb_count, curbuf->b_p_cpt_cb,
                       curbuf->b_p_cpt_count);
  }

  return OK;
}

/// Compound accessor for Phase 5 (pass 5): did_set_thesaurusfunc implementation.
const char *nvim_did_set_thesaurusfunc_impl(void *args_v)
{
  optset_T *args = (optset_T *)args_v;
  buf_T *buf = (buf_T *)args->os_buf;
  int retval;

  if (args->os_flags & OPT_LOCAL) {
    retval = option_set_callback_func(buf->b_p_tsrfu, &buf->b_tsrfu_cb);
  } else {
    retval = option_set_callback_func(p_tsrfu, &tsrfu_cb);
    if (!(args->os_flags & OPT_GLOBAL)) {
      callback_free(&buf->b_tsrfu_cb);
    }
  }

  return retval == FAIL ? e_invarg : NULL;
}

// did_set_thesaurusfunc deleted: now exported from Rust userfunc.rs via #[export_name]

/// Mark "copyID" references in an array of F{func} callbacks so that they are
/// not garbage collected.
bool set_ref_in_cpt_callbacks(Callback *callbacks, int count, int copyID)
{
  bool abort = false;

  if (callbacks == NULL) {
    return false;
  }

  for (int i = 0; i < count; i++) {
    abort = abort || rs_set_ref_in_callback(&callbacks[i], copyID, NULL, NULL);
  }
  return abort;
}

/// Compound accessor for Phase 5 (pass 5): set_ref_in_insexpand_funcs implementation.
int nvim_set_ref_in_insexpand_funcs_impl(int copyID)
{
  bool abort = rs_set_ref_in_callback(&cfu_cb, copyID, NULL, NULL);
  abort = abort || rs_set_ref_in_callback(&ofu_cb, copyID, NULL, NULL);
  abort = abort || rs_set_ref_in_callback(&tsrfu_cb, copyID, NULL, NULL);
  abort = abort || set_ref_in_cpt_callbacks(cpt_cb, cpt_cb_count, copyID);
  return abort ? 1 : 0;
}

// set_ref_in_insexpand_funcs deleted: now exported from Rust userfunc.rs via #[export_name]

/// Get the user-defined completion function name for completion "type"
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

/// Get the callback to use for insert mode completion.
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

/// Execute user defined complete function 'completefunc', 'omnifunc' or
/// 'thesaurusfunc', and get matches in "matches".
///
/// @param type  one of CTRL_X_OMNI or CTRL_X_FUNCTION or CTRL_X_THESAURUS
/// @param cb    set if triggered by a function in 'cpt' option, otherwise NULL
// NOTE: Body migrated to Rust rs_expand_by_function (Phase 9, pass 9).
// Logic lives in nvim_expand_by_function_full_impl compound accessor.
static void expand_by_function(int type, char *base, Callback *cb)
{
  nvim_expand_by_function_full_impl(type, base, (void *)cb);
}

static inline int get_user_highlight_attr(const char *hlname)
{
  if (hlname != NULL && *hlname != NUL) {
    return syn_name2attr(hlname);
  }
  return -1;
}

/// Add a match to the list of matches from Vimscript object
///
/// NOTE: Body migrated to Rust rs_ins_compl_add_tv (Phase 9, pass 9).
/// Logic lives in nvim_ins_compl_add_tv_impl compound accessor.
// FUNC_ATTR_NONNULL_ALL removed since body is now in compound accessor
static int ins_compl_add_tv(typval_T *const tv, const Direction dir, bool fast)
{
  return nvim_ins_compl_add_tv_impl((void *)tv, (int)dir, fast ? 1 : 0);
}

/// Add completions from a list.
/// NOTE: Body migrated to Rust rs_ins_compl_add_list (Phase 9, pass 9).
/// Logic lives in nvim_ins_compl_add_list_impl compound accessor.
static void ins_compl_add_list(list_T *const list)
{
  nvim_ins_compl_add_list_impl((void *)list);
}

/// Add completions from a dict.
/// NOTE: Body migrated to Rust rs_ins_compl_add_dict (Phase 9, pass 9).
/// Logic lives in nvim_ins_compl_add_dict_impl compound accessor.
static void ins_compl_add_dict(dict_T *dict)
{
  nvim_ins_compl_add_dict_impl((void *)dict);
}

/// Compound accessor: save extmarks before completion modifies text.
void nvim_save_orig_extmarks_impl(void)
{
  extmark_splice_delete(curbuf, curwin->w_cursor.lnum - 1, compl_col, curwin->w_cursor.lnum - 1,
                        compl_col + compl_length, &compl_orig_extmarks, true, kExtmarkUndo);
}

/// Compound accessor: restore extmarks in reverse order.
static void restore_orig_extmarks(void)
{
  for (long i = (int)kv_size(compl_orig_extmarks) - 1; i > -1; i--) {
    ExtmarkUndoObject undo_info = kv_A(compl_orig_extmarks, i);
    extmark_apply_undo(undo_info, true);
  }
}

/// Start completion for the complete() function.
///
/// @param startcol  where the matched text starts (1 is first column).
/// @param list      the list of matches.
// NOTE: Body migrated to Rust rs_set_completion (Phase 9, pass 9).
// Logic lives in nvim_set_completion_impl compound accessor.
static void set_completion(colnr_T startcol, list_T *list)
{
  nvim_set_completion_impl((int)startcol, (void *)list);
}

// NOTE: f_complete, f_complete_add, f_complete_check exported from Rust via
// #[export_name] in src/nvim-rs/insexpand/src/funcexpand.rs (Phase 1).

/// Fill the dict of complete_info
// Phase 2 (pass 5): rs_get_complete_info -- Rust wrapper
extern void rs_get_complete_info(void *what_list, void *retdict);

/// Compound accessor for Phase 2 (pass 5): complete_info() implementation.
/// Contains the full what_flag parsing and dictionary population logic.
void nvim_get_complete_info_impl(void *what_list_v, void *retdict_v)
{
  list_T *what_list = (list_T *)what_list_v;
  dict_T *retdict = (dict_T *)retdict_v;

#define CI_WHAT_MODE                0x01
#define CI_WHAT_PUM_VISIBLE         0x02
#define CI_WHAT_ITEMS               0x04
#define CI_WHAT_SELECTED            0x08
#define CI_WHAT_COMPLETED           0x10
#define CI_WHAT_MATCHES             0x20
#define CI_WHAT_PREINSERTED_TEXT    0x40
#define CI_WHAT_ALL                 0xff
  int what_flag;

  if (what_list == NULL) {
    what_flag = CI_WHAT_ALL & ~(CI_WHAT_MATCHES|CI_WHAT_COMPLETED);
  } else {
    what_flag = 0;
    for (listitem_T *item = tv_list_first(what_list)
         ; item != NULL
         ; item = TV_LIST_ITEM_NEXT(what_list, item)) {
      const char *what = tv_get_string(TV_LIST_ITEM_TV(item));

      if (strcmp(what, "mode") == 0) {
        what_flag |= CI_WHAT_MODE;
      } else if (strcmp(what, "pum_visible") == 0) {
        what_flag |= CI_WHAT_PUM_VISIBLE;
      } else if (strcmp(what, "items") == 0) {
        what_flag |= CI_WHAT_ITEMS;
      } else if (strcmp(what, "selected") == 0) {
        what_flag |= CI_WHAT_SELECTED;
      } else if (strcmp(what, "completed") == 0) {
        what_flag |= CI_WHAT_COMPLETED;
      } else if (strcmp(what, "preinserted_text") == 0) {
        what_flag |= CI_WHAT_PREINSERTED_TEXT;
      } else if (strcmp(what, "matches") == 0) {
        what_flag |= CI_WHAT_MATCHES;
      }
    }
  }

  int ret = OK;
  if (what_flag & CI_WHAT_MODE) {
    ret = tv_dict_add_str(retdict, S_LEN("mode"), rs_ins_compl_mode());
  }

  if (ret == OK && (what_flag & CI_WHAT_PUM_VISIBLE)) {
    ret = tv_dict_add_nr(retdict, S_LEN("pum_visible"), pum_visible());
  }

  if (ret == OK && (what_flag & CI_WHAT_PREINSERTED_TEXT)) {
    char *line = get_cursor_line_ptr();
    int len = compl_ins_end_col - curwin->w_cursor.col;
    ret = tv_dict_add_str_len(retdict, S_LEN("preinserted_text"),
                              len > 0 ? line + curwin->w_cursor.col : "", MAX(len, 0));
  }

  if (ret == OK && (what_flag & (CI_WHAT_ITEMS|CI_WHAT_SELECTED
                                 |CI_WHAT_MATCHES|CI_WHAT_COMPLETED))) {
    list_T *li = NULL;
    int selected_idx = -1;
    bool has_items = what_flag & CI_WHAT_ITEMS;
    bool has_matches = what_flag & CI_WHAT_MATCHES;
    bool has_completed = what_flag & CI_WHAT_COMPLETED;
    if (has_items || has_matches) {
      li = tv_list_alloc(kListLenMayKnow);
      const char *key = (has_matches && !has_items) ? "matches" : "items";
      ret = tv_dict_add_list(retdict, key, strlen(key), li);
    }
    if (ret == OK && what_flag & CI_WHAT_SELECTED) {
      if (compl_curr_match != NULL && compl_curr_match->cp_number == -1) {
        rs_ins_compl_update_sequence_numbers();
      }
    }
    if (ret == OK && compl_first_match != NULL) {
      int list_idx = 0;
      compl_T *match = compl_first_match;
      do {
        if (!match_at_original_text(match)) {
          if (has_items || (has_matches && match->cp_in_match_array)) {
            dict_T *di = tv_dict_alloc();
            tv_list_append_dict(li, di);
            tv_dict_add_str(di, S_LEN("word"), match->cp_str.data);
            tv_dict_add_str(di, S_LEN("abbr"), match->cp_text[CPT_ABBR]);
            tv_dict_add_str(di, S_LEN("menu"), match->cp_text[CPT_MENU]);
            tv_dict_add_str(di, S_LEN("kind"), match->cp_text[CPT_KIND]);
            tv_dict_add_str(di, S_LEN("info"), match->cp_text[CPT_INFO]);
            if (has_matches && has_items) {
              tv_dict_add_bool(di, S_LEN("match"), match->cp_in_match_array);
            }
            if (match->cp_user_data.v_type == VAR_UNKNOWN) {
              tv_dict_add_str(di, S_LEN("user_data"), "");
            } else {
              tv_dict_add_tv(di, S_LEN("user_data"), &match->cp_user_data);
            }
          }
          if (compl_curr_match != NULL
              && compl_curr_match->cp_number == match->cp_number) {
            selected_idx = list_idx;
          }
          if (!has_matches || match->cp_in_match_array) {
            list_idx++;
          }
        }
        match = match->cp_next;
      } while (match != NULL && !is_first_match(match));
    }
    if (ret == OK && (what_flag & CI_WHAT_SELECTED)) {
      ret = tv_dict_add_nr(retdict, S_LEN("selected"), selected_idx);
      win_T *wp = win_float_find_preview();
      if (wp != NULL) {
        tv_dict_add_nr(retdict, S_LEN("preview_winid"), wp->handle);
        tv_dict_add_nr(retdict, S_LEN("preview_bufnr"), wp->w_buffer->handle);
      }
    }
    if (ret == OK && selected_idx != -1 && has_completed) {
      dict_T *di = tv_dict_alloc();
      tv_dict_add_str(di, S_LEN("word"), compl_curr_match->cp_str.data);
      tv_dict_add_str(di, S_LEN("abbr"), compl_curr_match->cp_text[CPT_ABBR]);
      tv_dict_add_str(di, S_LEN("menu"), compl_curr_match->cp_text[CPT_MENU]);
      tv_dict_add_str(di, S_LEN("kind"), compl_curr_match->cp_text[CPT_KIND]);
      tv_dict_add_str(di, S_LEN("info"), compl_curr_match->cp_text[CPT_INFO]);
      if (compl_curr_match->cp_user_data.v_type == VAR_UNKNOWN) {
        tv_dict_add_str(di, S_LEN("user_data"), "");
      } else {
        tv_dict_add_tv(di, S_LEN("user_data"), &compl_curr_match->cp_user_data);
      }
      ret = tv_dict_add_dict(retdict, S_LEN("completed"), di);
    }
  }

  (void)ret;
}

// NOTE: f_complete_info exported from Rust via #[export_name] in
// src/nvim-rs/insexpand/src/info.rs (Phase 1).

// NOTE: process_next_cpt_value, INS_COMPL_CPT_* enum migrated to Rust
// as rs_process_next_cpt_value (Phase 14, Phase 3).

// NOTE: compare_scores and get_next_filename_completion deleted (Phase 15).
// Ported to Rust as rs_get_next_filename_completion in file.rs.

// NOTE: ins_compl_get_next_word_or_line and get_next_default_completion deleted (Phase 14, Phase 4).
// Logic now lives in rs_get_next_default_completion (expand.rs) and the
// nvim_ins_compl_st_do_search / nvim_ins_compl_st_add_word_or_line compound accessors.

/// Return the callback function associated with "p" if it refers to a
/// user-defined function in the 'complete' option.
/// The "idx" parameter is used for indexing callback entries.
// NOTE: Body migrated to Rust rs_get_callback_if_cpt_func (Phase 9, pass 9).
// Logic lives in nvim_get_callback_if_cpt_func_impl compound accessor.
static Callback *get_callback_if_cpt_func(char *p, int idx)
{
  return (Callback *)nvim_get_callback_if_cpt_func_impl((const char *)p, idx);
}


/// Compound accessor: get the pattern, column and length for command-line completion.
/// Sets the global variables: compl_col, compl_length and compl_pattern.
int nvim_get_cmdline_compl_info_impl(char *line, int curs_col)
{
  compl_pattern = cbuf_to_string(line, (size_t)curs_col);
  set_cmd_context(&compl_xp, compl_pattern.data,
                  (int)compl_pattern.size, curs_col, false);
  if (compl_xp.xp_context == EXPAND_LUA) {
    nlua_expand_pat(&compl_xp);
  }
  if (compl_xp.xp_context == EXPAND_UNSUCCESSFUL
      || compl_xp.xp_context == EXPAND_NOTHING) {
    // No completion possible, use an empty pattern to get a
    // "pattern not found" message.
    compl_col = curs_col;
  } else {
    compl_col = (int)(compl_xp.xp_pattern - compl_pattern.data);
  }
  compl_length = curs_col - compl_col;

  return OK;
}

/// Compound accessor: set global variables related to completion:
/// compl_col, compl_length, compl_pattern, and cpt_compl_pattern.
void nvim_set_compl_globals_impl(int startcol, int curs_col, int is_cpt_compl)
{
  if (is_cpt_compl) {
    API_CLEAR_STRING(cpt_compl_pattern);
    if (startcol < compl_col) {
      // Inline prepend_startcol_text: prepend line[startcol..compl_col] to compl_orig_text
      int prepend_len = compl_col - startcol;
      int new_length = prepend_len + (int)compl_orig_text.size;
      cpt_compl_pattern.size = (size_t)new_length;
      cpt_compl_pattern.data = xmalloc((size_t)new_length + 1);
      char *_line = ml_get(curwin->w_cursor.lnum);
      memmove(cpt_compl_pattern.data, _line + startcol, (size_t)prepend_len);
      memmove(cpt_compl_pattern.data + prepend_len, compl_orig_text.data, compl_orig_text.size);
      cpt_compl_pattern.data[new_length] = NUL;
      return;
    } else {
      cpt_compl_pattern = copy_string(compl_orig_text, NULL);
    }
  } else {
    if (startcol < 0 || startcol > curs_col) {
      startcol = curs_col;
    }

    // Re-obtain line in case it has changed
    char *line = ml_get(curwin->w_cursor.lnum);
    int len = curs_col - startcol;

    compl_pattern = cbuf_to_string(line + startcol, (size_t)len);
    compl_col = startcol;
    compl_length = len;
  }
}

/// Get the completion pattern, column and length.
///
/// @param startcol  start column number of the completion pattern/text
/// @param cur_col   current cursor column
///
/// On return, "line_invalid" is set to true, if the current line may have
/// become invalid and needs to be fetched again.
///
/// @return  OK on success.
static int compl_get_info(char *line, int startcol, colnr_T curs_col, bool *line_invalid)
{
  int line_invalid_int = 0;
  int ret = rs_compl_get_info(line, startcol, (int)curs_col, &line_invalid_int);
  if (line_invalid_int) {
    *line_invalid = true;
  }
  return ret;
}

// =============================================================================
// Phase 10 (pass 10): New fine-grained accessors for start/init migration
// =============================================================================

// nvim_get_did_ai: defined in change_ffi.c (bool nvim_get_did_ai(void))
// nvim_set_did_ai: defined in change_ffi.c (void nvim_set_did_ai(bool val))
void nvim_clear_indent_flags(void) { did_si = false; can_si = false; can_si_back = false; }
void nvim_set_compl_lnum_to_cursor(void) { compl_lnum = curwin->w_cursor.lnum; }
void nvim_ins_eol_wrap(int c) { ins_eol(c); }
void nvim_set_curbuf_b_p_com_empty(void) { curbuf->b_p_com = ""; }
void nvim_restore_curbuf_b_p_com(const char *old_val) { curbuf->b_p_com = (char *)old_val; }
const char *nvim_get_curbuf_b_p_com(void) { return curbuf->b_p_com; }
void nvim_set_compl_startpos_lnum_col(int lnum_to_cursor, int col) {
  if (lnum_to_cursor) {
    compl_startpos.lnum = curwin->w_cursor.lnum;
  }
  compl_startpos.col = (colnr_T)col;
}
/// Set compl_orig_text from line+compl_col with length compl_length.
void nvim_set_compl_orig_text_from_line(const char *line) {
  API_CLEAR_STRING(compl_orig_text);
  kv_destroy(compl_orig_extmarks);
  compl_orig_text = cbuf_to_string(line + compl_col, (size_t)compl_length);
}
/// Add orig text as first completion match. Returns OK or FAIL.
/// On FAIL, clears pattern/orig_text/extmarks and restores did_ai.
int nvim_ins_compl_add_orig_text(int flags, int save_did_ai) {
  if (ins_compl_add(compl_orig_text.data, (int)compl_orig_text.size,
                    NULL, NULL, false, NULL, 0,
                    flags, false, NULL, FUZZY_SCORE_NONE) != OK) {
    API_CLEAR_STRING(compl_pattern);
    API_CLEAR_STRING(compl_orig_text);
    kv_destroy(compl_orig_extmarks);
    did_ai = save_did_ai != 0;
    return FAIL;
  }
  return OK;
}
void nvim_set_edit_submode_extra_searching(void) { edit_submode_extra = _("-- Searching..."); }
// nvim_showmode_wrap: deleted (Phase 3, Rust calls showmode directly)

/// Compound accessor: set compl_startpos to the current cursor position.
void nvim_set_compl_startpos_to_cursor(void)
{
  compl_startpos = curwin->w_cursor;
}

/// Compound accessor: set compl_col to 0.
void nvim_set_compl_col_zero(void)
{
  compl_col = 0;
}

/// Compound accessor: set compl_startpos.col = compl_col.
void nvim_set_compl_startpos_col_to_compl_col(void)
{
  compl_startpos.col = (colnr_T)compl_col;
}

/// Compound accessor: restore did_ai from saved value.
void nvim_restore_did_ai(int saved_val)
{
  did_ai = saved_val != 0;
}

/// Compound accessor: set edit_submode to the CTRL-X mode message.
void nvim_set_edit_submode_ctrl_x_local_or_mode(void)
{
  if (compl_cont_status & CONT_LOCAL) {
    edit_submode = _(ctrl_x_msgs[CTRL_X_LOCAL_MSG]);
  } else {
    edit_submode = _(CTRL_X_MSG(ctrl_x_mode));
  }
}

/// Compound accessor: set edit_submode_pre to _(" Adding").
void nvim_set_edit_submode_adding(void)
{
  edit_submode_pre = _(" Adding");
}

/// Compound accessor: clear edit_submode_pre (set to NULL).
void nvim_clear_edit_submode_pre(void)
{
  edit_submode_pre = NULL;
}

/// Accessor: return the current buffer line at cursor position.
const char *nvim_ml_get_curline(void)
{
  return ml_get(curwin->w_cursor.lnum);
}

/// Compound accessor: set compl_direction.
void nvim_set_compl_direction(int val) { compl_direction = val; }

/// Compound accessor: set up completion window/buffer/match/direction state.
/// Sets compl_curr_win, compl_curr_buf, compl_shown_match, compl_shows_dir.
void nvim_ins_complete_setup_match_state(int direction)
{
  compl_curr_win = curwin;
  compl_curr_buf = curwin->w_buffer;
  compl_shown_match = compl_curr_match;
  compl_shows_dir = direction;
}

/// Compound accessor: return os_hrtime().
// nvim_os_hrtime: deleted (Phase 3, Rust calls os_hrtime directly)

/// Compound accessor: check if first_match->cp_next is the first match (no matches case).
int nvim_compl_first_match_next_is_first(void)
{
  return (compl_first_match && compl_first_match->cp_next
          && is_first_match(compl_first_match->cp_next)) ? 1 : 0;
}

/// Compound accessor: update compl_cont_status based on compl_curr_match->cp_flags.
void nvim_ins_complete_update_cont_s_ipos(void)
{
  if (compl_curr_match && (compl_curr_match->cp_flags & CP_CONT_S_IPOS)) {
    compl_cont_status |= CONT_S_IPOS;
  } else {
    compl_cont_status &= ~CONT_S_IPOS;
  }
}

/// Compound accessor: remove CONT_N_ADDS from compl_cont_status.
void nvim_compl_cont_status_remove_n_adds(void)
{
  compl_cont_status &= ~CONT_N_ADDS;
}

/// Compound accessor: eat the ESC vgetc() returns after CTRL-C (got_int handling).
void nvim_ins_complete_eat_got_int(void)
{
  if (got_int && !global_busy) {
    vgetc();
    got_int = false;
  }
}

/// Compound accessor: set compl_was_interrupted = compl_interrupted, compl_interrupted = false.
void nvim_ins_complete_finish_interrupted(void)
{
  compl_was_interrupted = compl_interrupted;
  compl_interrupted = false;
}

/// Compound accessor: set compl_ins_end_col = compl_col.
void nvim_set_compl_ins_end_col_to_compl_col(void)
{
  compl_ins_end_col = compl_col;
}

// ins_complete deleted: now exported from Rust entry.rs via #[export_name = "ins_complete"]

/// Compound accessor: free all completion global state at process exit.
void nvim_free_insexpand_stuff_impl(void)
{
  API_CLEAR_STRING(compl_orig_text);
  kv_destroy(compl_orig_extmarks);
  callback_free(&cfu_cb);
  callback_free(&ofu_cb);
  callback_free(&tsrfu_cb);
  clear_cpt_callbacks(&cpt_cb, cpt_cb_count);
}

// free_insexpand_stuff deleted: Rust exports under the C name directly via #[export_name].

/// Called when starting CTRL_X_SPELL mode: Move backwards to a previous badly
/// spelled word, if there is one.
void nvim_spell_back_to_badword_impl(void)
{
  pos_T tpos = curwin->w_cursor;
  spell_bad_len = spell_move_to(curwin, BACKWARD, SMT_ALL, true, NULL);
  if (curwin->w_cursor.col != tpos.col) {
    start_arrow(&tpos);
  }
}

// NOTE: f_preinserted exported from Rust via #[export_name] in
// src/nvim-rs/insexpand/src/funcexpand.rs (Phase 1).

// Completion state accessors (used by Rust insexpand crate)
int nvim_get_ctrl_x_mode(void) { return ctrl_x_mode; }
int nvim_get_compl_cont_status(void) { return compl_cont_status; }
int nvim_get_compl_started(void) { return compl_started; }
unsigned nvim_get_cot_flags_global(void) { return cot_flags; }
unsigned nvim_curbuf_get_b_cot_flags(void) { return curbuf->b_cot_flags; }
int nvim_get_compl_autocomplete(void) { return compl_autocomplete ? 1 : 0; }
// nvim_get_compl_restarting: deleted (Phase 2, COMPL_RESTARTING moved to Rust)
int nvim_get_compl_from_nonkeyword(void) { return compl_from_nonkeyword ? 1 : 0; }
int nvim_get_compl_direction(void) { return compl_direction; }
int nvim_get_compl_shows_dir(void) { return compl_shows_dir; }
int nvim_get_p_ic(void) { return p_ic ? 1 : 0; }
int nvim_get_p_inf(void) { return curbuf->b_p_inf ? 1 : 0; }
int nvim_get_compl_ins_end_col(void) { return compl_ins_end_col; }
int nvim_get_p_ac(void) { return p_ac ? 1 : 0; }
int nvim_curbuf_get_b_p_ac(void) { return curbuf->b_p_ac; }
int nvim_get_compl_lnum(void) { return (int)compl_lnum; }
int nvim_get_curwin_cursor_lnum(void) { return (int)curwin->w_cursor.lnum; }
// nvim_get_compl_hi_on_autocompl_longest: deleted (Phase 2, moved to Rust)
int nvim_syn_name2attr(const char *name) { return syn_name2attr(name); }
const char *nvim_get_compl_leader_data(void) { return compl_leader.data; }
size_t nvim_get_compl_leader_size(void) { return compl_leader.size; }
const char *nvim_get_compl_orig_text_data(void) { return compl_orig_text.data; }
size_t nvim_get_compl_orig_text_size(void) { return compl_orig_text.size; }
int nvim_compl_shown_match_exists(void) { return compl_shown_match != NULL ? 1 : 0; }
int nvim_get_compl_selected_item(void) { return compl_selected_item; }
int nvim_get_pum_want_insert(void) { return pum_want.insert ? 1 : 0; }
int nvim_pum_visible(void) { return pum_visible() ? 1 : 0; }

// Match list and popup menu accessors (used by Rust insexpand crate)
int nvim_compl_first_match_is_null(void) { return compl_first_match == NULL ? 1 : 0; }
int nvim_compl_curr_match_is_null(void) { return compl_curr_match == NULL ? 1 : 0; }
int nvim_get_compl_matches(void) { return compl_matches; }
int nvim_get_compl_get_longest(void) { return compl_get_longest ? 1 : 0; }
int nvim_get_compl_cont_mode(void) { return compl_cont_mode; }
int nvim_curbuf_get_b_p_inf(void) { return curbuf->b_p_inf ? 1 : 0; }

// Complex null-guard accessors
int nvim_compl_shown_match_is_singular(void) { return compl_shown_match ? (compl_shown_match == compl_shown_match->cp_next ? 1 : 0) : 0; }
int nvim_compl_shown_match_is_first(void) { return compl_shown_match ? (is_first_match(compl_shown_match) ? 1 : 0) : 0; }
size_t nvim_compl_shown_match_str_size(void) { return (compl_shown_match && compl_shown_match->cp_str.data) ? compl_shown_match->cp_str.size : 0; }
int nvim_compl_shown_match_has_newline(void) { return (compl_shown_match && compl_shown_match->cp_str.data) ? (vim_strchr(compl_shown_match->cp_str.data, '\n') != NULL ? 1 : 0) : 0; }
int nvim_compl_curr_match_at_original_text(void) { return compl_curr_match ? ((compl_curr_match->cp_flags & CP_ORIGINAL_TEXT) ? 1 : 0) : 0; }
// Accessors for set_ctrl_x_mode / may_advance_cpt_index (Phase 1)
void nvim_set_ctrl_x_mode(int val) { ctrl_x_mode = val; }
void nvim_set_compl_cont_mode(int val) { compl_cont_mode = val; }
void nvim_set_edit_submode_scroll(int is_replace) { edit_submode = is_replace ? _(" (replace) Scroll (^E/^Y)") : _(" (insert) Scroll (^E/^Y)"); edit_submode_pre = NULL; redraw_mode = true; }
void nvim_set_edit_submode_null(void) { edit_submode = NULL; }
void nvim_set_edit_submode_pre_null(void) { edit_submode_pre = NULL; }
void nvim_set_redraw_mode_true(void) { redraw_mode = true; }
int nvim_get_state_replace_flag(void) { return (State & REPLACE_FLAG) ? 1 : 0; }
void nvim_spell_back_safe(void) { emsg_off++; nvim_spell_back_to_badword_impl(); emsg_off--; }
// nvim_vpeekc: deleted (Phase 3, Rust calls vpeekc directly)
int nvim_get_cpt_sources_index(void) { return cpt_sources_index; }

// Accessor for ins_compl_prep (Phase 2)
void nvim_set_compl_used_match(int val) { compl_used_match = val != 0; }
void nvim_ins_redraw(int ready) { ins_redraw(ready != 0); }
// Accessors for Phase 2 (pass 12): ins_compl_longest_match
// nvim_utf_ptr2char is defined in mbyte.c; re-use it via extern declaration
// nvim_mb_tolower: deleted (Phase 3, Rust calls mb_tolower directly)
int nvim_cursor_col_gt_compl_col(void) { return curwin->w_cursor.col > compl_col ? 1 : 0; }

// Accessors for ins_compl_stop (Phase 3)
const char *nvim_get_compl_curr_match_str_data(void) { return compl_curr_match ? compl_curr_match->cp_str.data : NULL; }
char *nvim_get_compl_shown_match_str_dup(void) { return compl_shown_match ? xstrdup(compl_shown_match->cp_str.data) : NULL; }
void nvim_clear_compl_best_matches(void) { compl_best_matches = 0; }
int nvim_cursor_on_nul(void) { char *line = get_cursor_line_ptr(); return (line && line[curwin->w_cursor.col] != NUL) ? 1 : 0; }
// Compound accessors for ins_compl_stop (Phase 3)
void nvim_ins_apply_autocmds_completedonepre(void) { ins_apply_autocmds(EVENT_COMPLETEDONEPRE); }
bool nvim_shortmess_completionmenu(void) { return shortmess(SHM_COMPLETIONMENU); }
bool nvim_in_cinkeys_key_complete(int when, bool line_is_empty) { return in_cinkeys(KEY_COMPLETE, when, line_is_empty); }
void nvim_set_edit_submode_null_if_set(void) { if (edit_submode != NULL) { edit_submode = NULL; redraw_mode = true; } }
void nvim_ins_compl_insert_bytes(const char *p, int len) {
  char *q = (char *)p;
  if (len == -1) {
    len = (int)strlen(q);
  }
  assert(len >= 0);
  ins_bytes_len(q, (size_t)len);
  compl_ins_end_col = curwin->w_cursor.col;
}
void nvim_restore_orig_extmarks(void) { restore_orig_extmarks(); }

// Accessor for internal_error in compl_get_info dispatch
void nvim_internal_error_compl_get_info(void) { internal_error("ins_complete()"); }

// Compound accessors for ins_compl_show_pum (Phase 2)
// nvim_update_screen() already exists in drawscreen.c
// nvim_get_cursor_col() already exists in normal_shim.c
// nvim_ins_compl_build_pum: deleted (Rust calls rs_ins_compl_build_pum directly)
// nvim_find_shown_match_in_array: deleted (Rust calls nvim_find_shown_match_in_match_array directly)
void nvim_trigger_complete_changed(int cur) { trigger_complete_changed_event(cur); }
int nvim_has_completechanged_event(void) { return has_event(EVENT_COMPLETECHANGED) ? 1 : 0; }
void nvim_set_dollar_vcol_minus_one(void) { dollar_vcol = -1; }
void nvim_set_cursor_col_to_compl_col(void) { curwin->w_cursor.col = (colnr_T)compl_col; }
void nvim_restore_cursor_col(int col) { curwin->w_cursor.col = (colnr_T)col; }
void nvim_pum_display_compl(int cur, int array_changed) { pum_display(compl_match_array, compl_match_arraysize, cur, array_changed != 0, 0); }
int nvim_compl_curr_neq_shown(void) { return (compl_curr_match != compl_shown_match) ? 1 : 0; }
void nvim_compl_set_curr_to_shown(void) { compl_curr_match = compl_shown_match; }

// Compound accessors for ins_compl_delete (Phase 3)
int nvim_ins_compl_delete_body(int col) {
  String remaining = STRING_INIT;
  if (curwin->w_cursor.lnum > compl_lnum) {
    if (curwin->w_cursor.col < get_cursor_line_len()) {
      remaining = cbuf_to_string(get_cursor_pos_ptr(), (size_t)get_cursor_pos_len());
    }
    while (curwin->w_cursor.lnum > compl_lnum) {
      if (ml_delete(curwin->w_cursor.lnum) == FAIL) {
        xfree(remaining.data);
        return 0;
      }
      deleted_lines_mark(curwin->w_cursor.lnum, 1);
      curwin->w_cursor.lnum--;
    }
    curwin->w_cursor.col = get_cursor_line_len();
  }
  if ((int)curwin->w_cursor.col > col) {
    if (stop_arrow() == FAIL) {
      xfree(remaining.data);
      return 0;
    }
    backspace_until_column(col);
    compl_ins_end_col = curwin->w_cursor.col;
  }
  if (remaining.data != NULL) {
    int orig_col = curwin->w_cursor.col;
    ins_str(remaining.data, remaining.size);
    curwin->w_cursor.col = orig_col;
    xfree(remaining.data);
  }
  changed_cline_bef_curs(curwin);
  set_vim_var_dict(VV_COMPLETED_ITEM, tv_dict_alloc_lock(VAR_FIXED));
  return 1;
}
void nvim_set_cursor_col_to_ins_end(void) { curwin->w_cursor.col = (colnr_T)compl_ins_end_col; }

// Compound accessors for ins_compl_insert (Phase 3)
const char *nvim_compl_shown_cp_str_data(void) { return compl_shown_match ? compl_shown_match->cp_str.data : NULL; }
size_t nvim_compl_shown_cp_str_size(void) { return compl_shown_match ? compl_shown_match->cp_str.size : 0; }
// nvim_find_common_prefix_data: deleted (Rust calls rs_find_common_prefix directly)
int nvim_compl_shown_cp_cpt_source_idx(void) { return compl_shown_match ? compl_shown_match->cp_cpt_source_idx : -1; }
int nvim_get_cpt_source_startcol(int idx) { return (cpt_sources_array && idx >= 0) ? cpt_sources_array[idx].cs_startcol : -1; }
int nvim_cpt_sources_array_exists(void) { return cpt_sources_array != NULL ? 1 : 0; }
int nvim_get_cpt_source_cs_flag(int idx) { return (cpt_sources_array && idx >= 0) ? (int)(unsigned char)cpt_sources_array[idx].cs_flag : 0; }
int nvim_get_cpt_source_cs_max_matches(int idx) { return (cpt_sources_array && idx >= 0) ? cpt_sources_array[idx].cs_max_matches : 0; }
int nvim_mb_byte2len(int b) { return (b >= 0 && b <= 255) ? MB_BYTE2LEN((uint8_t)b) : 1; }
// nvim_get_cursor_line_ptr: defined in change_ffi.c (returns char *)
// nvim_get_curwin_cursor_col: defined in change_ffi.c (returns colnr_T)
// nvim_ascii_iswhite_or_nul: defined in normal_shim.c as bool nvim_ascii_iswhite_or_nul(int c)
int nvim_get_cpt_sources_count(void) { return cpt_sources_count; }
// nvim_xcalloc_ints: deleted (Phase 3, Rust calls xcalloc directly)
void nvim_ins_compl_expand_multiple_skip(const char *str, int skip) {
  char *start = (char *)str + skip;
  char *curr = start;
  int base_indent = get_indent();
  while (*curr != NUL) {
    if (*curr == '\n') {
      if (curr > start) {
        ins_char_bytes(start, (size_t)(curr - start));
      }
      open_line(FORWARD, OPENLINE_KEEPTRAIL | OPENLINE_FORCE_INDENT, base_indent, NULL);
      start = curr + 1;
    }
    curr++;
  }
  if (curr > start) {
    ins_char_bytes(start, (size_t)(curr - start));
  }
  compl_ins_end_col = curwin->w_cursor.col;
}
void nvim_ins_compl_insert_bytes_len(const char *cp_str, int compl_len, int ins_len) { nvim_ins_compl_insert_bytes(cp_str + compl_len, ins_len); }
void nvim_cursor_col_sub(int n) { curwin->w_cursor.col -= (colnr_T)n; }
int nvim_compl_shown_match_at_orig_text(void) { return compl_shown_match ? (match_at_original_text(compl_shown_match) ? 1 : 0) : 0; }
void nvim_ins_compl_dict_alloc_set_shown(void) { set_vim_var_dict(VV_COMPLETED_ITEM, ins_compl_dict_alloc(compl_shown_match)); }
// nvim_set_compl_hi_on_longest: deleted (Phase 2, COMPL_HI_ON_AUTOCOMPL_LONGEST moved to Rust)

// Compound accessors for Phase 4 (ins_compl_restart, ins_ctrl_x, check_compl_option,
// ins_compl_addleader, ins_compl_addfrommatch, ins_compl_set_original_text,
// ins_compl_check_keys)
void nvim_compl_cont_status_or(int mask) { compl_cont_status |= mask; }
void nvim_set_edit_submode_ctrl_x_msg(int mode) { edit_submode = _(CTRL_X_MSG(mode)); }
// nvim_may_trigger_modechanged: defined in normal_shim.c
// nvim_stop_arrow: deleted (Phase 3, Rust calls stop_arrow directly)
// nvim_utf_char2bytes: defined in change_ffi.c (int nvim_utf_char2bytes(int c, char *buf))
// nvim_utf_char2len, nvim_ins_char, nvim_ins_char_bytes: deleted (Phase 3, Rust calls directly)
void nvim_api_clear_compl_leader(void) { API_CLEAR_STRING(compl_leader); }
void nvim_set_compl_leader_from_cursor(void) {
  compl_leader = cbuf_to_string(get_cursor_line_ptr() + compl_col,
                                (size_t)(curwin->w_cursor.col - compl_col));
}
void nvim_ins_compl_set_original_text_impl(const char *str, size_t len) {
  if (match_at_original_text(compl_first_match)) {
    API_CLEAR_STRING(compl_first_match->cp_str);
    compl_first_match->cp_str = cbuf_to_string(str, len);
  } else if (compl_first_match->cp_prev != NULL
             && match_at_original_text(compl_first_match->cp_prev)) {
    API_CLEAR_STRING(compl_first_match->cp_prev->cp_str);
    compl_first_match->cp_prev->cp_str = cbuf_to_string(str, len);
  }
}
int nvim_check_compl_option_dict(void) {
  return (*curbuf->b_p_dict == NUL && *p_dict == NUL && !curwin->w_p_spell) ? 1 : 0;
}
int nvim_check_compl_option_tsr(void) {
  return (*curbuf->b_p_tsr == NUL && *p_tsr == NUL
          && *curbuf->b_p_tsrfu == NUL && *p_tsrfu == NUL) ? 1 : 0;
}
void nvim_emsg_dict_empty(int is_dict) {
  emsg(is_dict ? _("'dictionary' option is empty") : _("'thesaurus' option is empty"));
}
int nvim_emsg_silent_is_zero(void) { return emsg_silent == 0 ? 1 : 0; }
// nvim_in_assert_fails: defined in change_ffi.c (bool nvim_in_assert_fails(void))
void nvim_vim_beep_complete(void) { vim_beep(kOptBoFlagComplete); }
// nvim_setcursor: deleted (Phase 3, Rust calls setcursor directly)
// nvim_ui_has_messages: defined in message.c (int nvim_ui_has_messages(void))
// nvim_ui_flush: defined in change_ffi.c (void nvim_ui_flush(void))
// nvim_os_delay: defined in change_ffi.c (void nvim_os_delay(long ms, bool allow_input))
// nvim_vpeekc_any: deleted (Phase 3, Rust calls vpeekc_any directly)
int nvim_test_disable_char_avail(void) { return test_disable_char_avail ? 1 : 0; }
// NOTE: nvim_vim_is_ctrl_x_key deleted (Phase 15). keys.rs calls rs_vim_is_ctrl_x_key() directly.
// nvim_safe_vgetc, nvim_vungetc: deleted (Phase 3, Rust calls directly)
int nvim_got_int(void) { return got_int ? 1 : 0; }
int nvim_key_typed(void) { return KeyTyped ? 1 : 0; }
void nvim_set_compl_interrupted(int val) { compl_interrupted = val != 0; }
int nvim_using_script(void) { return using_script() ? 1 : 0; }
int nvim_ex_normal_busy(void) { return ex_normal_busy ? 1 : 0; }
// nvim_get/set_compl_pending: deleted (Phase 2, COMPL_PENDING moved to Rust)
int nvim_cot_flags_has_noinsert_fuzzy(void) { return (cot_flags & (kOptCotFlagNoinsert | kOptCotFlagFuzzy)) ? 1 : 0; }
// nvim_get_compl_shows_dir: already defined above (line 5277)
void nvim_set_compl_shows_dir(int val) { compl_shows_dir = val; }
// NOTE: nvim_ins_compl_key2dir and nvim_ins_compl_key2count deleted (Phase 15).
// keys.rs calls rs_ins_compl_key2dir() and rs_ins_compl_key2count() directly.

// Phase 1 (pass 8) accessors for rs_ins_compl_next / find_next_completion_match
int nvim_get_compl_startpos_lnum(void) { return (int)compl_startpos.lnum; }
int nvim_get_compl_startpos_col(void) { return (int)compl_startpos.col; }
int nvim_compl_shown_match_score(void) { return compl_shown_match ? compl_shown_match->cp_score : FUZZY_SCORE_NONE; }
int nvim_compl_shown_match_has_fname(void) { return (compl_shown_match && compl_shown_match->cp_fname != NULL) ? 1 : 0; }
int nvim_compl_shown_match_str_eq_orig(void) {
  return (compl_shown_match && compl_orig_text.data
          && strequal(compl_shown_match->cp_str.data, compl_orig_text.data)) ? 1 : 0;
}
int nvim_cpt_sources_index_non_neg(void) { return cpt_sources_index >= 0 ? 1 : 0; }
int nvim_p_cto(void) { return (int)p_cto; }

// Phase 2 (pass 8): compound accessors for rs_ins_compl_get_exp / get_next_completion_match

// --- ins_compl_st field accessors ---
char *nvim_ins_compl_st_get_dict(void) { return ins_compl_st.dict; }
int nvim_ins_compl_st_get_dict_f(void) { return ins_compl_st.dict_f; }
void nvim_ins_compl_st_clear_dict(void) { ins_compl_st.dict = NULL; }
void *nvim_ins_compl_st_get_func_cb(void) { return ins_compl_st.func_cb; }
int nvim_ins_compl_st_get_first_lnum(void) { return (int)ins_compl_st.first_match_pos.lnum; }
void nvim_ins_compl_st_set_found_all(int val) { ins_compl_st.found_all = val != 0; }
int nvim_ins_compl_st_get_found_all(void) { return ins_compl_st.found_all ? 1 : 0; }
int nvim_ins_compl_st_e_cpt_is_nul(void) { return *ins_compl_st.e_cpt == NUL ? 1 : 0; }
void nvim_ins_compl_st_reset_set_match_pos(void) { ins_compl_st.set_match_pos = false; }
int nvim_ins_compl_st_buf_valid(void) { return buf_valid(ins_compl_st.ins_buf) ? 1 : 0; }
int nvim_ins_compl_st_ins_buf_is_curbuf(void) { return ins_compl_st.ins_buf == curbuf ? 1 : 0; }
void nvim_ins_compl_st_mark_ins_buf_scanned(void) {
  if (ins_compl_st.ins_buf) {
    ins_compl_st.ins_buf->b_scanned = true;
  }
}

// --- compl_old_match / compl_curr_match compound ops ---
void nvim_ins_compl_set_old_match_to_curr(void) { compl_old_match = compl_curr_match; }
int nvim_compl_curr_vs_old_match_changed(void) {
  return (compl_curr_match != compl_old_match) ? 1 : 0;
}
void nvim_compl_old_match_advance_curr(void) {
  // Called at end of ins_compl_get_exp to advance compl_curr_match.
  if (compl_old_match != NULL) {
    compl_curr_match = rs_compl_dir_forward()
                       ? compl_old_match->cp_next
                       : compl_old_match->cp_prev;
    if (compl_curr_match == NULL) {
      compl_curr_match = compl_old_match;
    }
  }
}
void nvim_compl_curr_rewind_to_head(void) {
  // For ^P: walk back to the list head (skip original text sentinel)
  while (compl_curr_match && compl_curr_match->cp_prev
         && !match_at_original_text(compl_curr_match->cp_prev)) {
    compl_curr_match = compl_curr_match->cp_prev;
  }
}

// --- timeout / cpt_sources_index / misc ---
void nvim_set_cpt_sources_index(int val) { cpt_sources_index = val; }
void nvim_semsg_list_index_out_of_range(int idx) { semsg(_(e_list_index_out_of_range_nr), idx); }
int nvim_get_compl_num_bests(void) { return compl_num_bests; }
void nvim_set_compl_timeout_ms(uint64_t val) { compl_timeout_ms = val; }
int nvim_get_compl_pattern_is_null(void) { return compl_pattern.data == NULL ? 1 : 0; }
int nvim_get_p_act(void) { return (int)p_act; }
int nvim_normal_mode_strict(void) {
  return (rs_ctrl_x_mode_normal() && !rs_ctrl_x_mode_line_or_eval()
          && !(compl_cont_status & CONT_LOCAL)
          && cpt_sources_array != NULL) ? 1 : 0;
}

// --- ins_compl_st compound initialization ---
// Called at the top of ins_compl_get_exp when !compl_started.
// Sets up ins_compl_st for a fresh search from position (lnum, col).
// Returns the (possibly adjusted) start position as lnum/col via out params.
void nvim_ins_compl_get_exp_init_state(int lnum, int col, int *out_lnum, int *out_col) {
  FOR_ALL_BUFFERS(buf) {
    buf->b_scanned = false;
  }
  if (!ins_compl_st_cleared) {
    CLEAR_FIELD(ins_compl_st);
    ins_compl_st_cleared = true;
  }
  ins_compl_st.found_all = false;
  ins_compl_st.ins_buf = curbuf;
  xfree(ins_compl_st.e_cpt_copy);
  ins_compl_st.e_cpt_copy = xstrdup((compl_cont_status & CONT_LOCAL) ? "." : curbuf->b_p_cpt);
  rs_strip_caret_numbers_in_place(ins_compl_st.e_cpt_copy);
  ins_compl_st.e_cpt = ins_compl_st.e_cpt_copy;

  pos_T start_pos = { .lnum = (linenr_T)lnum, .col = (colnr_T)col };
  if (compl_autocomplete && rs_is_nearest_active()) {
    start_pos.lnum = MAX(1, start_pos.lnum - LOOKBACK_LINE_COUNT);
    start_pos.col = 0;
  }
  ins_compl_st.last_match_pos = ins_compl_st.first_match_pos = start_pos;

  *out_lnum = (int)start_pos.lnum;
  *out_col  = (int)start_pos.col;
}

// Called at the top of ins_compl_get_exp when compl_started && ins_buf != curbuf.
void nvim_ins_compl_get_exp_check_buf(void) {
  if (ins_compl_st.ins_buf != curbuf && !buf_valid(ins_compl_st.ins_buf)) {
    ins_compl_st.ins_buf = curbuf;
  }
}

// Sets st.cur_match_pos based on current direction (call after init).
void nvim_ins_compl_st_set_cur_match_dir(void) {
  ins_compl_st.cur_match_pos = rs_compl_dir_forward()
                               ? &ins_compl_st.last_match_pos
                               : &ins_compl_st.first_match_pos;
}

// NOTE: nvim_process_next_cpt_value_wrap deleted (Phase 14, Phase 3).
// rs_ins_compl_get_exp now calls rs_process_next_cpt_value directly.

// NOTE: nvim_get_next_default_completion_wrap deleted (Phase 14, Phase 4).
// rs_get_next_default_completion in expand.rs calls compound accessors directly.

// NOTE: nvim_get_next_filename_completion_wrap deleted (Phase 15).
// rs_get_next_filename_completion in file.rs calls compound accessors directly.

// Phase 15: thin accessors for filename completion migration to Rust
int nvim_expand_wildcards_files(int count, char **pat, int *num_matches, char ***matches)
{
  return expand_wildcards(count, pat, num_matches, matches,
                          EW_FILE|EW_DIR|EW_ADDSLASH|EW_SILENT);
}
void nvim_tilde_replace_wrap(char *pat, int num_matches, char **matches)
{
  tilde_replace(pat, num_matches, matches);
}
int nvim_get_p_fic_or_wic(void) { return (p_fic || p_wic) ? 1 : 0; }
void nvim_compl_pattern_set_star(void)
{
  API_CLEAR_STRING(compl_pattern);
  compl_pattern = cbuf_to_string("*", 1);
}
void nvim_compl_pattern_set_from_alloc(char *data, size_t size)
{
  API_CLEAR_STRING(compl_pattern);
  compl_pattern.data = data;
  compl_pattern.size = size;
}
char *nvim_compl_pattern_get_data(void) { return compl_pattern.data; }

// NOTE: nvim_ins_compl_new_leader_wrapper deleted (Phase 15).
// insert.rs and leader.rs call rs_ins_compl_new_leader() directly.
// NOTE: nvim_ins_compl_addfrommatch_body deleted (Phase 15).
// Ported to Rust as rs_ins_compl_addfrommatch in leader.rs.
// edit.c now calls rs_ins_compl_addfrommatch() directly.
// Accessors for Phase 2: ins_compl_bs migration
// nvim_get_cursor_line_ptr: defined in change_ffi.c as char *nvim_get_cursor_line_ptr(void)
const char *nvim_mb_ptr_back(const char *line, const char *p) {
  const char *pp = p;
  MB_PTR_BACK(line, pp);
  return pp;
}
int nvim_can_bs_start(void) { return can_bs(BS_START) ? 1 : 0; }
void nvim_api_clear_and_set_compl_leader(const char *data, size_t len) {
  API_CLEAR_STRING(compl_leader);
  compl_leader = cbuf_to_string(data, len);
}
int nvim_compl_shown_match_is_null(void) { return compl_shown_match == NULL ? 1 : 0; }
void nvim_compl_set_shown_to_first(void) { compl_shown_match = compl_first_match; }

// Raw mergesort accessor: sorts linked list starting at `head`, returns new head.
// compare_type: 0 = fuzzy (descending score), 1 = nearest (ascending score).
void *nvim_mergesort_compl_list_raw(void *head, int compare_type)
{
  return mergesort_list(head, cp_get_next, cp_set_next, cp_get_prev, cp_set_prev,
                        compare_type == 0 ? cp_compare_fuzzy : cp_compare_nearest);
}
void *nvim_compl_first_match_get_prev(void) { return compl_first_match ? compl_first_match->cp_prev : NULL; }
// Returns 1 if compl_shown_match equals sentinel (compl_first_match for forward,
// compl_first_match->cp_prev for backward), 0 otherwise
int nvim_compl_shown_match_is_sentinel(int forward) {
  if (!compl_shown_match || !compl_first_match) {
    return 1;
  }
  void *sentinel = forward ? (void *)compl_first_match : (void *)compl_first_match->cp_prev;
  return compl_shown_match == sentinel ? 1 : 0;
}

// Accessors for Phase 4: ins_compl_new_leader migration
int nvim_get_p_acl(void) { return (int)p_acl; }
void nvim_pum_undisplay(int undo) { pum_undisplay(undo != 0); }
void nvim_redraw_later_valid(void) { redraw_later(curwin, UPD_VALID); }
int nvim_is_cpt_func_refresh_always(void) {
  for (int i = 0; i < cpt_sources_count; i++) {
    if (cpt_sources_array[i].cs_refresh_always) {
      return 1;
    }
  }
  return 0;
}
void nvim_cpt_compl_refresh(void) { nvim_cpt_compl_refresh_impl(); }
void nvim_set_spell_bad_len(int val) { spell_bad_len = val; }
// nvim_set_compl_restarting: deleted (Phase 2, COMPL_RESTARTING moved to Rust)
int nvim_ins_complete_ctrl_n(void) { return ins_complete(Ctrl_N, true); }
// Compound accessor: compl_enter_selects = !compl_used_match && compl_selected_item != -1
void nvim_update_compl_enter_selects(void) { compl_enter_selects = !compl_used_match && compl_selected_item != -1; }

// Accessor for Phase 5: ins_compl_del_pum migration
void nvim_xfree_compl_match_array(void) { XFREE_CLEAR(compl_match_array); }

// Accessors for Phase 1 (pass 3): ins_compl_mode, thesaurus_func_complete,
// get_next_*_completion, do_autocmd_completedone, ins_compl_show_filename
int nvim_get_p_tsrfu_nonempty(void) { return *p_tsrfu != NUL ? 1 : 0; }
int nvim_get_curbuf_b_p_tsrfu_nonempty(void) { return *curbuf->b_p_tsrfu != NUL ? 1 : 0; }

// Compound accessor: delegates to the original C implementation logic
void nvim_get_next_include_file_completion(int compl_type)
{
  find_pattern_in_path(compl_pattern.data, compl_direction,
                       compl_pattern.size, false, false,
                       ((compl_type == CTRL_X_PATH_DEFINES
                         && !(compl_cont_status & CONT_SOL))
                        ? FIND_DEFINE : FIND_ANY),
                       1, ACTION_EXPAND, 1, MAXLNUM, false, compl_autocomplete);
}

void nvim_get_next_cmdline_completion_impl(void)
{
  char **matches;
  int num_matches;
  if (expand_cmdline(&compl_xp, compl_pattern.data,
                     (int)compl_pattern.size, &num_matches, &matches) == EXPAND_OK) {
    rs_ins_compl_add_matches(num_matches, matches, false);
  }
}

void nvim_get_next_spell_completion_impl(int lnum)
{
  char **matches;
  int num_matches = expand_spelling((linenr_T)lnum, compl_pattern.data, &matches);
  if (num_matches > 0) {
    rs_ins_compl_add_matches(num_matches, matches, p_ic);
  } else {
    xfree(matches);
  }
}

void nvim_do_autocmd_completedone_impl(int c, int mode, char *word)
{
  save_v_event_T save_v_event;
  dict_T *v_event = get_v_event(&save_v_event);

  mode = mode & ~CTRL_X_WANT_IDENT;
  char *mode_str = NULL;
  if (ctrl_x_mode_names[mode]) {
    mode_str = ctrl_x_mode_names[mode];
  }
  tv_dict_add_str(v_event, S_LEN("complete_word"), word != NULL ? word : "");
  tv_dict_add_str(v_event, S_LEN("complete_type"), mode_str != NULL ? mode_str : "");

  tv_dict_add_str(v_event, S_LEN("reason"),
                  (c == Ctrl_Y ? "accept" : (c == Ctrl_E ? "cancel" : "discard")));
  tv_dict_set_keys_readonly(v_event);

  ins_apply_autocmds(EVENT_COMPLETEDONE);
  restore_v_event(v_event, &save_v_event);
}

void nvim_ins_compl_show_filename_impl(void)
{
  char *const lead = _("match in file");
  int space = sc_col - vim_strsize(lead) - 2;
  if (space <= 0) {
    return;
  }

  char *s;
  char *e;
  for (s = e = compl_shown_match->cp_fname; *e != NUL; MB_PTR_ADV(e)) {
    space -= ptr2cells(e);
    while (space < 0) {
      space += ptr2cells(s);
      MB_PTR_ADV(s);
    }
  }
  if (!compl_autocomplete) {
    msg_hist_off = true;
    vim_snprintf(IObuff, IOSIZE, "%s %s%s", lead,
                 s > compl_shown_match->cp_fname ? "<" : "", s);
    msg(IObuff, 0);
    msg_hist_off = false;
    redraw_cmdline = false;  // don't overwrite!
  }
}

// Compound accessors for Phase 2 (pass 3): pattern helper functions.
// These contain the original C logic (moved here from the function bodies
// above) to avoid circular call chains: the original function names now become
// thin Rust wrappers, so the logic must live here.

int nvim_get_normal_compl_info_impl(char *line, int startcol, int curs_col)
{
  if ((compl_cont_status & CONT_SOL) || rs_ctrl_x_mode_path_defines()) {
    if (!rs_compl_status_adding()) {
      while (--startcol >= 0 && vim_isIDc((uint8_t)line[startcol])) {}
      compl_col += ++startcol;
      compl_length = curs_col - startcol;
    }
    if (p_ic) {
      compl_pattern = cstr_as_string(str_foldcase(line + compl_col,
                                                  compl_length, NULL, 0));
    } else {
      compl_pattern = cbuf_to_string(line + compl_col, (size_t)compl_length);
    }
  } else if (rs_compl_status_adding()) {
    char *prefix = "\\<";
    size_t prefixlen = STRLEN_LITERAL("\\<");

    if (!vim_iswordp(line + compl_col)
        || (compl_col > 0
            && (vim_iswordp(mb_prevptr(line, line + compl_col))))) {
      prefix = "";
      prefixlen = 0;
    }

    // we need up to 2 extra chars for the prefix
    size_t n = rs_quote_meta(NULL, line + compl_col, compl_length) + prefixlen;
    compl_pattern.data = xmalloc(n);
    STRCPY(compl_pattern.data, prefix);
    rs_quote_meta(compl_pattern.data + prefixlen, line + compl_col, compl_length);
    compl_pattern.size = n - 1;
  } else if (--startcol < 0
             || !vim_iswordp(mb_prevptr(line, line + startcol + 1))) {
    // Match any word of at least two chars
    compl_pattern = cbuf_to_string(S_LEN("\\<\\k\\k"));
    compl_col += curs_col;
    compl_length = 0;
    compl_from_nonkeyword = true;
  } else {
    // Search the point of change class of multibyte character
    // or not a word single byte character backward.
    startcol -= utf_head_off(line, line + startcol);
    int base_class = mb_get_class(line + startcol);
    while (--startcol >= 0) {
      int head_off = utf_head_off(line, line + startcol);
      if (base_class != mb_get_class(line + startcol - head_off)) {
        break;
      }
      startcol -= head_off;
    }

    compl_col += ++startcol;
    compl_length = (int)curs_col - startcol;
    if (compl_length == 1) {
      // Only match word with at least two chars -- webb
      // there's no need to call quote_meta,
      // xmalloc(7) is enough  -- Acevedo
      compl_pattern.data = xmalloc(7);
      STRCPY(compl_pattern.data, "\\<");
      rs_quote_meta(compl_pattern.data + 2, line + compl_col, 1);
      strcat(compl_pattern.data, "\\k");
      compl_pattern.size = strlen(compl_pattern.data);
    } else {
      size_t n = rs_quote_meta(NULL, line + compl_col, compl_length) + 2;
      compl_pattern.data = xmalloc(n);
      STRCPY(compl_pattern.data, "\\<");
      rs_quote_meta(compl_pattern.data + 2, line + compl_col, compl_length);
      compl_pattern.size = n - 1;
    }
  }

  // Call functions in 'complete' with 'findstart=1'
  if (rs_ctrl_x_mode_normal() && !(compl_cont_status & CONT_LOCAL)) {
    // ^N completion, not complete() or ^X^N
    rs_setup_cpt_sources();
    rs_prepare_cpt_compl_funcs();
  }

  return OK;
}

int nvim_get_wholeline_compl_info_impl(char *line, int curs_col)
{
  compl_col = (colnr_T)getwhitecols(line);
  compl_length = (int)curs_col - (int)compl_col;
  if (compl_length < 0) {  // cursor in indent: empty pattern
    compl_length = 0;
  }
  if (p_ic) {
    compl_pattern = cstr_as_string(str_foldcase(line + compl_col,
                                                compl_length, NULL, 0));
  } else {
    compl_pattern = cbuf_to_string(line + compl_col, (size_t)compl_length);
  }

  return OK;
}

int nvim_get_filename_compl_info_impl(char *line, int startcol, int curs_col)
{
  // Go back to just before the first filename character.
  if (startcol > 0) {
    char *p = line + startcol;

    MB_PTR_BACK(line, p);
    while (p > line && vim_isfilec(utf_ptr2char(p))) {
      MB_PTR_BACK(line, p);
    }
    bool p_is_filec = false;
#ifdef MSWIN
    // check for drive letters on mswin
    if (p > line && path_has_drive_letter(p - 1, line + startcol - (p - 1))) {
      p -= p == line + 1 ? 1 : 2;
      p_is_filec = true;
    }
#endif
    p_is_filec = p_is_filec || vim_isfilec(utf_ptr2char(p));

    if (p == line && p_is_filec) {
      startcol = 0;
    } else {
      startcol = (int)(p - line) + 1;
    }
  }

  compl_col += startcol;
  compl_length = (int)curs_col - startcol;
  compl_pattern = cstr_as_string(addstar(line + compl_col,
                                         (size_t)compl_length, EXPAND_FILES));

  return OK;
}

int nvim_get_spell_compl_info_impl(int startcol, int curs_col)
{
  if (spell_bad_len > 0) {
    assert(spell_bad_len <= INT_MAX);
    compl_col = curs_col - (int)spell_bad_len;
  } else {
    compl_col = spell_word_start(startcol);
  }
  if (compl_col >= (colnr_T)startcol) {
    compl_length = 0;
    compl_col = curs_col;
  } else {
    spell_expand_check_cap(compl_col);
    compl_length = (int)curs_col - compl_col;
  }
  // Need to obtain "line" again, it may have become invalid.
  char *line = ml_get(curwin->w_cursor.lnum);
  compl_pattern = cbuf_to_string(line + compl_col, (size_t)compl_length);

  return OK;
}

// =============================================================================
// Phase 10 (pass 10): New fine-grained accessors for continue_search and
// show_statusmsg migration to Rust.
// =============================================================================

// --- compl_col / compl_length / compl_startpos setters ---
void nvim_set_compl_col(int val) { compl_col = (colnr_T)val; }
void nvim_set_compl_length(int val) { compl_length = val; }
void nvim_set_compl_startpos_col(int val) { compl_startpos.col = (colnr_T)val; }
void nvim_set_compl_startpos_lnum_to_cursor(void) { compl_startpos.lnum = curwin->w_cursor.lnum; }

// --- getwhitecols / skipwhite wrappers ---
int nvim_getwhitecols_of_line(const char *line) { return (int)getwhitecols(line); }
/// Returns the column offset of skipwhite(line + length + start_col) relative to line.
int nvim_skipwhite_offset(const char *line, int length, int start_col) {
  return (int)(skipwhite(line + length + start_col) - line);
}

// --- edit_submode_extra compound setters (keep _() in C) ---
void nvim_set_edit_submode_extra_hitend(void) { edit_submode_extra = _(e_hitend); }
void nvim_set_edit_submode_extra_patnotf(void) { edit_submode_extra = _(e_patnotf); }
void nvim_set_edit_submode_extra_back_at_original(void) { edit_submode_extra = _("Back at original"); }
void nvim_set_edit_submode_extra_word_from_other_line(void) { edit_submode_extra = _("Word from other line"); }
void nvim_set_edit_submode_extra_the_only_match(void) { edit_submode_extra = _("The only match"); }
/// Format "match %d of %d" or "match %d" into static buffer and set edit_submode_extra.
void nvim_set_edit_submode_extra_match_ref(int cp_number, int compl_matches_val) {
  static char match_ref[81];
  if (compl_matches_val > 0) {
    vim_snprintf(match_ref, sizeof(match_ref), _("match %d of %d"), cp_number, compl_matches_val);
  } else {
    vim_snprintf(match_ref, sizeof(match_ref), _("match %d"), cp_number);
  }
  edit_submode_extra = match_ref;
}
int nvim_get_edit_submode_extra_is_null(void) { return edit_submode_extra == NULL ? 1 : 0; }
const char *nvim_get_edit_submode_extra_ptr(void) { return edit_submode_extra; }

// --- edit_submode_highl setters (using HLF enum values) ---
void nvim_set_edit_submode_highl_e(void) { edit_submode_highl = HLF_E; }
void nvim_set_edit_submode_highl_w(void) { edit_submode_highl = HLF_W; }
void nvim_set_edit_submode_highl_r(void) { edit_submode_highl = HLF_R; }
void nvim_set_edit_submode_highl_count(void) { edit_submode_highl = HLF_COUNT; }
/// Returns (edit_submode_highl < HLF_COUNT ? edit_submode_highl + 1 : 0)
int nvim_get_edit_submode_highl_attr(void) {
  return edit_submode_highl < HLF_COUNT ? (int)edit_submode_highl + 1 : 0;
}

// --- compl_curr_match accessors ---
int nvim_compl_curr_match_cp_number(void) { return compl_curr_match ? compl_curr_match->cp_number : -1; }
void nvim_compl_curr_match_set_cp_number(int val) { if (compl_curr_match) compl_curr_match->cp_number = val; }
int nvim_compl_curr_match_next_eq_prev(void) {
  return (compl_curr_match
          && compl_curr_match->cp_next == compl_curr_match->cp_prev) ? 1 : 0;
}

// --- misc message / display wrappers ---
// nvim_get_p_smd: defined in normal_shim.c
// nvim_get_dollar_vcol: defined in edit.c
void nvim_curs_columns_curwin(void) { curs_columns(curwin, false); }
void nvim_msg_hist_off_set(int val) { msg_hist_off = val != 0; }
void nvim_msg_ext_set_kind_completion(void) { msg_ext_set_kind("completion"); }
void nvim_msg_with_attr(const char *s, int attr) { msg(s, attr); }
void nvim_msg_clr_cmdline_wrap(void) { msg_clr_cmdline(); }

// Compound accessor for Phase 2 (pass 4): get_next_bufname_token
void nvim_get_next_bufname_token_impl(void)
{
  FOR_ALL_BUFFERS(b) {
    if (b->b_p_bl && b->b_sfname != NULL) {
      char *tail = path_tail(b->b_sfname);
      if (strncmp(tail, compl_orig_text.data, compl_orig_text.size) == 0) {
        ins_compl_add(tail, (int)strlen(tail), NULL, NULL, false, NULL, 0,
                      p_ic ? CP_ICASE : 0, false, NULL, FUZZY_SCORE_NONE);
      }
    }
  }
}

// Compound accessor for Phase 3 (pass 4): get_next_tag_completion
void nvim_get_next_tag_completion_impl(void)
{
  // set p_ic according to p_ic, p_scs and pat for find_tags().
  const int save_p_ic = p_ic;
  p_ic = ignorecase(compl_pattern.data);

  // Find up to TAG_MANY matches.  Avoids that an enormous number
  // of matches is found when compl_pattern is empty
  g_tag_at_cursor = true;
  char **matches;
  int num_matches;
  if (find_tags(compl_pattern.data, &num_matches, &matches,
                TAG_REGEXP | TAG_NAMES | TAG_NOIC | TAG_INS_COMP
                | (rs_ctrl_x_mode_not_default() ? TAG_VERBOSE : 0),
                TAG_MANY, curbuf->b_ffname) == OK && num_matches > 0) {
    rs_ins_compl_add_matches(num_matches, matches, p_ic);
  }
  g_tag_at_cursor = false;
  p_ic = save_p_ic;
}

// Compound accessors for Phase 4 (pass 4): compl_source_start_timer and
// advance_cpt_sources_index_safe

// nvim_compl_source_start_timer_impl: migrated to Rust rs_compl_source_start_timer (Phase 4)
// nvim_advance_cpt_sources_index_safe_impl: migrated to Rust rs_advance_cpt_sources_index_safe (Phase 4)

// Accessors for Phase 3 (pass 12): register completion migration
int nvim_get_num_registers(void) { return NUM_REGISTERS; }
size_t nvim_yankreg_y_size(void *reg) { return reg ? ((yankreg_T *)reg)->y_size : 0; }
int nvim_yankreg_y_array_null(void *reg) { return (!reg || ((yankreg_T *)reg)->y_array == NULL) ? 1 : 0; }
const char *nvim_yankreg_y_array_entry_data(void *reg, size_t j)
{
  if (!reg) {
    return NULL;
  }
  yankreg_T *r = (yankreg_T *)reg;
  if (j >= r->y_size || r->y_array == NULL) {
    return NULL;
  }
  return r->y_array[j].data;
}
int nvim_ins_compl_add_infercase_ffi(const char *str, int len, int icase, const char *fname,
                                     int dir, int cont_s_ipos, int score)
{
  return ins_compl_add_infercase((char *)str, len, icase != 0, (char *)fname, (Direction)dir,
                                 cont_s_ipos != 0, score);
}



// Accessors for Phase 1 (pass 6): show_pum, rs_ins_compl_add_matches, spell_back_to_badword
void nvim_set_redrawing_disabled(int val) { RedrawingDisabled = val; }
int nvim_get_curwin_w_wrow(void) { return curwin->w_wrow; }
// Accessor for Phase 4 (pass 12): rs_ins_compl_add_matches migration
int nvim_ins_compl_add_simple(const char *str, int len, int dir, int flags, int score)
{
  return ins_compl_add((char *)str, len, NULL, NULL, false, NULL, (Direction)dir, flags, false,
                       NULL, score);
}

// Accessors for Phase 5 (pass 12): cpt-source migration
void nvim_cpt_sources_alloc(int count)
{
  XFREE_CLEAR(cpt_sources_array);
  cpt_sources_index = -1;
  cpt_sources_count = 0;
  if (count > 0) {
    cpt_sources_array = xcalloc((size_t)count, sizeof(cpt_source_T));
    cpt_sources_count = count;
  }
}
void nvim_cpt_sources_set_flag(int idx, int flag)
{
  if (cpt_sources_array && idx >= 0 && idx < cpt_sources_count) {
    cpt_sources_array[idx].cs_flag = (char)flag;
  }
}
void nvim_cpt_sources_set_max_matches(int idx, int val)
{
  if (cpt_sources_array && idx >= 0 && idx < cpt_sources_count) {
    cpt_sources_array[idx].cs_max_matches = val;
  }
}
void nvim_cpt_sources_set_startcol(int idx, int val)
{
  if (cpt_sources_array && idx >= 0 && idx < cpt_sources_count) {
    cpt_sources_array[idx].cs_startcol = val;
  }
}
void nvim_cpt_sources_set_refresh_always(int idx, int val)
{
  if (cpt_sources_array && idx >= 0 && idx < cpt_sources_count) {
    cpt_sources_array[idx].cs_refresh_always = val != 0;
  }
}
int nvim_cpt_sources_get_refresh_always(int idx)
{
  return (cpt_sources_array && idx >= 0 && idx < cpt_sources_count)
         ? (cpt_sources_array[idx].cs_refresh_always ? 1 : 0) : 0;
}
// copy_option_part wrapper: advances *src past the next option segment, writes to buf.
// Returns the number of bytes written (excluding NUL).
size_t nvim_copy_option_part_ffi(char **src, char *buf, int maxlen, const char *sep)
{
  return copy_option_part(src, buf, (size_t)maxlen, sep);
}
// vim_strchr wrapper: returns pointer to first occurrence of c in s, or NULL.
const char *nvim_vim_strchr_ffi(const char *s, int c) { return vim_strchr(s, c); }
// nvim_xstrdup is defined in register.c; use it via the existing declaration.
// Returns 1 if the completion function name for ctrl_x_mode is empty, 0 otherwise.
int nvim_get_complete_funcname_empty(int ctrl_x_mode_val)
{
  return *get_complete_funcname(ctrl_x_mode_val) == NUL ? 1 : 0;
}


// Returns an opaque pointer to the Callback for the given ctrl_x_mode.
void *nvim_get_insert_callback_opaque(int ctrl_x_mode_val)
{
  return (void *)get_insert_callback(ctrl_x_mode_val);
}

// Call the completion callback with findstart=1 to get the start column.
// Saves/restores State and cursor. Returns the column number from the callback,
// or INT_MIN if the cursor was moved (error emitted).
int nvim_callback_call_findstart(void *cb_opaque)
{
  const int save_State = State;
  typval_T args[3];
  args[0].v_type = VAR_NUMBER;
  args[1].v_type = VAR_STRING;
  args[2].v_type = VAR_UNKNOWN;
  args[0].vval.v_number = 1;
  args[1].vval.v_string = "";

  pos_T pos = curwin->w_cursor;
  textlock++;
  colnr_T col = (colnr_T)callback_call_retnr((Callback *)cb_opaque, 2, args);
  textlock--;

  State = save_State;
  curwin->w_cursor = pos;
  check_cursor(curwin);
  validate_cursor(curwin);
  if (!equalpos(curwin->w_cursor, pos)) {
    emsg(_(e_compldel));
    return INT_MIN;
  }
  return (int)col;
}

// Reset ctrl_x_mode to CTRL_X_NORMAL and clear the menu status message.
void nvim_ctrl_x_mode_reset_to_normal(void)
{
  ctrl_x_mode = CTRL_X_NORMAL;
  edit_submode = NULL;
  if (!shortmess(SHM_COMPLETIONMENU)) {
    msg_clr_cmdline();
  }
}
// Emit "option not set" error for completefunc/omnifunc.
void nvim_emit_completefunc_not_set_error(int is_function)
{
  semsg(_(e_notset), is_function ? "completefunc" : "omnifunc");
}
// expand_by_function with non-NULL callback (for cpt func sources).
void nvim_expand_by_function_with_cb(void *cb_opaque)
{
  expand_by_function(0, cpt_compl_pattern.data, (Callback *)cb_opaque);
}
// set compl_opt_refresh_always = false
void nvim_clear_compl_opt_refresh_always(void) { compl_opt_refresh_always = false; }
// Advance p past cpt option segment using IObuff (for prepare_cpt_compl_funcs)
size_t nvim_copy_option_part_iobuff_ffi(char **src)
{
  return copy_option_part(src, IObuff, IOSIZE, ",");
}

// =============================================================================
// Phase 9 (pass 9): Compound C accessors for funcexpand.rs
// These expose static C functions to Rust via the compound-accessor pattern.
// =============================================================================

// Forward declarations for the Rust wrappers defined in funcexpand.rs
extern void rs_expand_by_function(int type, char *base, void *cb);
extern int rs_ins_compl_add_tv(void *tv, int dir, int fast);
extern void rs_ins_compl_add_list(void *list);
extern void rs_ins_compl_add_dict(void *dict);
// NOTE: rs_f_complete, rs_f_complete_add, rs_f_complete_check, rs_f_preinserted
// are now exported directly as f_complete etc. via #[export_name] (Phase 1).
extern void rs_set_completion(int startcol, void *list);
extern void rs_remove_old_matches(void);
extern void *rs_get_callback_if_cpt_func(const char *p, int idx);

// Phase 1 accessors: wrap static functions for Rust

/// Compound accessor: full expand_by_function logic, callable from Rust.
/// Contains the same logic as the static expand_by_function but calls
/// nvim_ins_compl_add_list_impl / nvim_ins_compl_add_dict_impl to avoid
/// circular calls after expand_by_function becomes a thin wrapper.
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
    if (*funcname == NUL) {
      return;
    }
    cb = get_insert_callback(type);
  }

  typval_T args[3];
  args[0].v_type = VAR_NUMBER;
  args[1].v_type = VAR_STRING;
  args[2].v_type = VAR_UNKNOWN;
  args[0].vval.v_number = 0;
  args[1].vval.v_string = base != NULL ? base : "";

  pos_T pos = curwin->w_cursor;
  textlock++;

  if (callback_call(cb, 2, args, &rettv)) {
    switch (rettv.v_type) {
    case VAR_LIST:
      matchlist = rettv.vval.v_list;
      break;
    case VAR_DICT:
      matchdict = rettv.vval.v_dict;
      break;
    case VAR_SPECIAL:
      FALLTHROUGH;
    default:
      tv_clear(&rettv);
      break;
    }
  }
  textlock--;

  curwin->w_cursor = pos;
  check_cursor(curwin);
  validate_cursor(curwin);
  if (!equalpos(curwin->w_cursor, pos)) {
    emsg(_(e_compldel));
    goto theend;
  }

  if (matchlist != NULL) {
    nvim_ins_compl_add_list_impl(matchlist);
  } else if (matchdict != NULL) {
    nvim_ins_compl_add_dict_impl(matchdict);
  }

theend:
  State = save_State;

  if (matchdict != NULL) {
    tv_dict_unref(matchdict);
  }
  if (matchlist != NULL) {
    tv_list_unref(matchlist);
  }
}

/// Compound accessor: ins_compl_add_tv logic, callable from Rust.
/// Contains the full logic moved from the static ins_compl_add_tv function.
int nvim_ins_compl_add_tv_impl(void *tv_opaque, int dir, int fast)
{
  typval_T *tv = (typval_T *)tv_opaque;
  const char *word;
  bool dup = false;
  bool empty = false;
  int flags = fast ? CP_FAST : 0;
  char *(cptext[CPT_COUNT]);
  char *user_abbr_hlname = NULL;
  char *user_kind_hlname = NULL;
  int user_hl[2] = { -1, -1 };
  typval_T user_data;

  user_data.v_type = VAR_UNKNOWN;
  if (tv->v_type == VAR_DICT && tv->vval.v_dict != NULL) {
    word = tv_dict_get_string(tv->vval.v_dict, "word", false);
    cptext[CPT_ABBR] = tv_dict_get_string(tv->vval.v_dict, "abbr", true);
    cptext[CPT_MENU] = tv_dict_get_string(tv->vval.v_dict, "menu", true);
    cptext[CPT_KIND] = tv_dict_get_string(tv->vval.v_dict, "kind", true);
    cptext[CPT_INFO] = tv_dict_get_string(tv->vval.v_dict, "info", true);

    user_abbr_hlname = tv_dict_get_string(tv->vval.v_dict, "abbr_hlgroup", false);
    user_hl[0] = get_user_highlight_attr(user_abbr_hlname);

    user_kind_hlname = tv_dict_get_string(tv->vval.v_dict, "kind_hlgroup", false);
    user_hl[1] = get_user_highlight_attr(user_kind_hlname);

    tv_dict_get_tv(tv->vval.v_dict, "user_data", &user_data);

    if (tv_dict_get_number(tv->vval.v_dict, "icase")) {
      flags |= CP_ICASE;
    }
    dup = (bool)tv_dict_get_number(tv->vval.v_dict, "dup");
    empty = (bool)tv_dict_get_number(tv->vval.v_dict, "empty");
    if (tv_dict_get_string(tv->vval.v_dict, "equal", false) != NULL
        && tv_dict_get_number(tv->vval.v_dict, "equal")) {
      flags |= CP_EQUAL;
    }
  } else {
    word = tv_get_string_chk(tv);
    CLEAR_FIELD(cptext);
  }
  if (word == NULL || (!empty && *word == NUL)) {
    free_cptext(cptext);
    tv_clear(&user_data);
    return FAIL;
  }
  int status = ins_compl_add((char *)word, -1, NULL, cptext, true,
                             &user_data, dir, flags, dup, user_hl, FUZZY_SCORE_NONE);
  if (status != OK) {
    tv_clear(&user_data);
  }
  return status;
}

/// Compound accessor: ins_compl_add_list logic, callable from Rust.
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

/// Compound accessor: ins_compl_add_dict logic, callable from Rust.
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

// Phase 2 accessors: full logic for f_complete, f_complete_add,
// f_complete_check, f_preinserted -- callable from Rust.

/// Compound accessor: f_complete logic, callable from Rust.
/// Contains the full logic from the original f_complete function.
void nvim_f_complete_impl(void *argvars_v, void *rettv_v)
{
  typval_T *argvars = (typval_T *)argvars_v;
  (void)rettv_v;  // not used by f_complete

  if ((State & MODE_INSERT) == 0) {
    emsg(_("E785: complete() can only be used in Insert mode"));
    return;
  }
  if (!undo_allowed(curbuf)) {
    return;
  }
  if (argvars[1].v_type != VAR_LIST) {
    emsg(_(e_invarg));
  } else {
    const colnr_T startcol = (colnr_T)tv_get_number_chk(&argvars[0], NULL);
    if (startcol > 0) {
      nvim_set_completion_impl((int)(startcol - 1), argvars[1].vval.v_list);
    }
  }
}

/// Compound accessor: f_complete_add logic, callable from Rust.
/// Contains the full logic from the original f_complete_add function.
void nvim_f_complete_add_impl(void *argvars_v, void *rettv_v)
{
  typval_T *argvars = (typval_T *)argvars_v;
  typval_T *rettv = (typval_T *)rettv_v;
  rettv->vval.v_number = nvim_ins_compl_add_tv_impl((void *)&argvars[0], 0, 0);
}

/// Compound accessor: f_complete_check logic, callable from Rust.
/// Contains the full logic from the original f_complete_check function.
void nvim_f_complete_check_impl(void *rettv_v)
{
  typval_T *rettv = (typval_T *)rettv_v;
  int saved = RedrawingDisabled;
  RedrawingDisabled = 0;
  rs_ins_compl_check_keys(0, 1);
  rettv->vval.v_number = rs_ins_compl_interrupted();
  RedrawingDisabled = saved;
}

/// Compound accessor: f_preinserted logic, callable from Rust.
/// Contains the full logic from the original f_preinserted function.
void nvim_f_preinserted_impl(void *rettv_v)
{
  typval_T *rettv = (typval_T *)rettv_v;
  if (rs_ins_compl_preinsert_effect()) {
    rettv->vval.v_number = 1;
  }
}

/// Compound accessor: f_complete_info logic, callable from Rust.
/// Contains the argument parsing from the original f_complete_info function.
void nvim_f_complete_info_impl(void *argvars_v, void *rettv_v)
{
  typval_T *argvars = (typval_T *)argvars_v;
  typval_T *rettv = (typval_T *)rettv_v;
  tv_dict_alloc_ret(rettv);

  list_T *what_list = NULL;

  if (argvars[0].v_type != VAR_UNKNOWN) {
    if (argvars[0].v_type != VAR_LIST) {
      emsg(_(e_listreq));
      return;
    }
    what_list = argvars[0].vval.v_list;
  }
  rs_get_complete_info(what_list, rettv->vval.v_dict);
}

// Phase 3 accessor: set_completion

/// Compound accessor: set_completion logic, callable from Rust.
/// Contains the full logic from the original set_completion function.
void nvim_set_completion_impl(int startcol_arg, void *list_opaque)
{
  colnr_T startcol = (colnr_T)startcol_arg;
  list_T *list = (list_T *)list_opaque;

  int flags = CP_ORIGINAL_TEXT;
  unsigned cur_cot_flags = rs_get_cot_flags();
  bool compl_longest = (cur_cot_flags & kOptCotFlagLongest) != 0;
  bool compl_no_insert = (cur_cot_flags & kOptCotFlagNoinsert) != 0;
  bool compl_no_select = (cur_cot_flags & kOptCotFlagNoselect) != 0;

  // If already doing completions stop it.
  if (rs_ctrl_x_mode_not_default()) {
    rs_ins_compl_prep(' ');
  }
  rs_ins_compl_clear();
  rs_ins_compl_free();
  compl_get_longest = compl_longest;

  compl_direction = FORWARD;
  if (startcol > curwin->w_cursor.col) {
    startcol = curwin->w_cursor.col;
  }
  compl_col = startcol;
  compl_lnum = curwin->w_cursor.lnum;
  compl_length = curwin->w_cursor.col - startcol;
  compl_orig_text = cbuf_to_string(get_cursor_line_ptr() + compl_col,
                                   (size_t)compl_length);
  rs_save_orig_extmarks();
  if (p_ic) {
    flags |= CP_ICASE;
  }
  if (ins_compl_add(compl_orig_text.data, (int)compl_orig_text.size,
                    NULL, NULL, false, NULL, 0,
                    flags | CP_FAST, false, NULL, FUZZY_SCORE_NONE) != OK) {
    return;
  }

  ctrl_x_mode = CTRL_X_EVAL;

  nvim_ins_compl_add_list_impl(list);
  compl_matches = rs_ins_compl_make_cyclic();
  compl_started = true;
  compl_used_match = true;
  compl_cont_status = 0;
  int save_w_wrow = curwin->w_wrow;
  int save_w_leftcol = curwin->w_leftcol;

  compl_curr_match = compl_first_match;
  bool no_select = compl_no_select || compl_longest;
  if (compl_no_insert || no_select) {
    ins_complete(K_DOWN, false);
    if (no_select) {
      ins_complete(K_UP, false);
    }
  } else {
    ins_complete(Ctrl_N, false);
  }
  compl_enter_selects = compl_no_insert;

  if (!compl_interrupted) {
    rs_show_pum(save_w_wrow, save_w_leftcol);
  }

  may_trigger_modechanged();
  ui_flush();
}

/// Compound accessor: cpt_compl_refresh logic, callable from Rust.
/// Contains the full logic from the original cpt_compl_refresh function.
void nvim_cpt_compl_refresh_impl(void)
{
  // Make the completion list linear (non-cyclic)
  rs_ins_compl_make_linear();
  // Make a copy of 'cpt' in case the buffer gets wiped out
  char *cpt = xstrdup(curbuf->b_p_cpt);
  rs_strip_caret_numbers_in_place(cpt);

  cpt_sources_index = 0;
  for (char *p = cpt; *p;) {
    while (*p == ',' || *p == ' ') {  // Skip delimiters
      p++;
    }
    if (*p == NUL) {
      break;
    }

    if (cpt_sources_array[cpt_sources_index].cs_refresh_always) {
      Callback *cb = (Callback *)nvim_get_callback_if_cpt_func_impl(p, cpt_sources_index);
      if (cb) {
        rs_remove_old_matches();
        int startcol = 0;
        int ret = rs_get_userdefined_compl_info(curwin->w_cursor.col, cb, &startcol);
        if (ret == FAIL) {
          if (startcol == -3) {
            cpt_sources_array[cpt_sources_index].cs_refresh_always = false;
          } else {
            startcol = -2;
          }
        } else if (startcol < 0 || startcol > curwin->w_cursor.col) {
          startcol = curwin->w_cursor.col;
        }
        cpt_sources_array[cpt_sources_index].cs_startcol = startcol;
        if (ret == OK) {
          rs_compl_source_start_timer(cpt_sources_index);
          rs_get_cpt_func_completion_matches(cb);
        }
      }
    }

    (void)copy_option_part(&p, IObuff, IOSIZE, ",");  // Advance p
    if (rs_may_advance_cpt_index(p) != 0) {
      (void)rs_advance_cpt_sources_index_safe();
    }
  }
  cpt_sources_index = -1;

  xfree(cpt);
  // Make the list cyclic
  compl_matches = rs_ins_compl_make_cyclic();
}

/// Compound accessor: get_callback_if_cpt_func logic, callable from Rust.
/// Contains the full logic from the original get_callback_if_cpt_func function.
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

// Phase 14 accessors: buffer/window traversal and search for ins_compl_next_buf migration.

/// Returns buf->b_next (next buffer in the buffer list).
buf_T *nvim_buf_get_b_next(buf_T *buf) { return buf->b_next; }

/// Returns buf->b_scanned.
int nvim_buf_get_b_scanned(buf_T *buf) { return buf->b_scanned; }


/// Returns wp->w_next (next window in the window list).
win_T *nvim_win_get_w_next(win_T *wp) { return wp->w_next; }

/// Returns wp->w_config.focusable.
int nvim_win_get_focusable(win_T *wp) { return wp->w_config.focusable ? 1 : 0; }

/// Returns wp->w_buffer (for insexpand, distinct name to avoid declaration conflicts).
buf_T *nvim_win_get_w_buffer_raw(win_T *wp) { return wp->w_buffer; }

/// Compound accessor: wraps the insexpand-specific searchit() call.
/// Equivalent to: searchit(NULL, buf, pos, NULL, dir, pat, patlen, 1,
///                         SEARCH_KEEP+SEARCH_NFMSG, RE_LAST, NULL)
/// Returns FAIL/OK; modifies pos in place.

// Phase 14 accessors for process_next_cpt_value migration (Phase 3).

/// Returns curbuf->b_scanned.
int nvim_curbuf_get_b_scanned(void) { return curbuf->b_scanned ? 1 : 0; }

/// Returns the current character at ins_compl_st.e_cpt (as unsigned int).
/// Returns 0 (NUL) if e_cpt is NULL.
int nvim_ins_compl_st_get_e_cpt_char(void) {
  return ins_compl_st.e_cpt ? (int)(unsigned char)*ins_compl_st.e_cpt : 0;
}

/// Skips commas and spaces at the start of ins_compl_st.e_cpt.
void nvim_ins_compl_st_skip_delimiters(void) {
  while (*ins_compl_st.e_cpt == ',' || *ins_compl_st.e_cpt == ' ') {
    ins_compl_st.e_cpt++;
  }
}

/// Sets ins_compl_st.ins_buf = curbuf and copies start_pos into first/last_match_pos.
/// Performs the initial pos setup for the '.' (current buffer) case.
/// Returns 1 if rs_ctrl_x_mode_normal() and dec(&first_match_pos) underflowed
/// (i.e., position moved before the buffer start; caller should wrap to end).
/// In that case, sets first_match_pos to (ml_line_count, ml_get_len(ml_line_count)).
int nvim_ins_compl_st_set_dot_source(int start_lnum, int start_col, int fuzzy_collect)
{
  ins_compl_st.ins_buf = curbuf;
  ins_compl_st.first_match_pos.lnum = (linenr_T)start_lnum;
  ins_compl_st.first_match_pos.col  = (colnr_T)start_col;
  int wrapped = 0;
  if (rs_ctrl_x_mode_normal() && (!fuzzy_collect && dec(&ins_compl_st.first_match_pos) < 0)) {
    ins_compl_st.first_match_pos.lnum = ins_compl_st.ins_buf->b_ml.ml_line_count;
    ins_compl_st.first_match_pos.col  = ml_get_len(ins_compl_st.first_match_pos.lnum);
    wrapped = 1;
  }
  ins_compl_st.last_match_pos = ins_compl_st.first_match_pos;
  ins_compl_st.set_match_pos = true;
  return wrapped;
}

/// Calls rs_ins_compl_next_buf(ins_compl_st.ins_buf, flag) and updates
/// ins_compl_st.ins_buf. Returns 1 if the returned buffer != curbuf, else 0.
/// If the buffer is loaded, sets compl_started=true and initialises match
/// positions for a full-buffer scan. Otherwise marks found_all=true.
/// Returns:
///   2  = buffer found and loaded (compl_type should be 0)
///   1  = buffer found but unloaded (compl_type should be CTRL_X_DICTIONARY)
///   0  = no new buffer (wrapped back to curbuf)
int nvim_ins_compl_st_advance_buf(int flag)
{
  buf_T *next = rs_ins_compl_next_buf(ins_compl_st.ins_buf, flag);
  if (next == curbuf) {
    return 0;
  }
  ins_compl_st.ins_buf = next;
  if (ins_compl_st.ins_buf->b_ml.ml_mfp != NULL) {
    // Loaded buffer: set up for full scan
    compl_started = true;
    ins_compl_st.first_match_pos.col = ins_compl_st.last_match_pos.col = 0;
    ins_compl_st.first_match_pos.lnum = ins_compl_st.ins_buf->b_ml.ml_line_count + 1;
    ins_compl_st.last_match_pos.lnum = 0;
    return 2;
  }
  // Unloaded buffer: scan like dictionary
  ins_compl_st.found_all = true;
  return 1;
}

/// Returns ins_compl_st.ins_buf->b_fname (may be NULL) as a C string pointer.
const char *nvim_ins_compl_st_get_ins_buf_fname(void)
{
  return ins_compl_st.ins_buf ? ins_compl_st.ins_buf->b_fname : NULL;
}

/// Returns ins_compl_st.ins_buf->b_sfname (may be NULL).
/// Emits the "Scanning: <name>" completion message for ins_compl_st.ins_buf.
void nvim_ins_compl_st_msg_scanning(void)
{
  if (!shortmess(SHM_COMPLETIONSCAN) && !compl_autocomplete) {
    msg_hist_off = true;
    msg_ext_set_kind("completion");
    vim_snprintf(IObuff, IOSIZE, _("Scanning: %s"),
                 ins_compl_st.ins_buf->b_fname == NULL
                 ? buf_spname(ins_compl_st.ins_buf)
                 : ins_compl_st.ins_buf->b_sfname == NULL
                 ? ins_compl_st.ins_buf->b_fname
                 : ins_compl_st.ins_buf->b_sfname);
    msg_trunc(IObuff, true, HLF_R);
  }
}

/// Emits the "Scanning tags." completion message.
void nvim_ins_compl_st_msg_scanning_tags(void)
{
  if (!shortmess(SHM_COMPLETIONSCAN) && !compl_autocomplete) {
    msg_ext_set_kind("completion");
    msg_hist_off = true;
    vim_snprintf(IObuff, IOSIZE, "%s", _("Scanning tags."));
    msg_trunc(IObuff, true, HLF_R);
  }
}

/// Sets ins_compl_st.dict = ins_compl_st.e_cpt and dict_f = DICT_FIRST.
void nvim_ins_compl_st_set_dict_from_e_cpt(void)
{
  ins_compl_st.dict = ins_compl_st.e_cpt;
  ins_compl_st.dict_f = DICT_FIRST;
}

/// Advances ins_compl_st.e_cpt by one character (for 'k'/'s' option path).
void nvim_ins_compl_st_e_cpt_inc(void)
{
  ins_compl_st.e_cpt++;
}

/// Returns the character one past the current e_cpt position (peeks ahead).
/// Sets ins_compl_st.func_cb via get_callback_if_cpt_func logic.
/// Returns 1 if a valid callback was found, 0 if not.
int nvim_ins_compl_st_set_func_cb_from_e_cpt(int cpt_idx)
{
  ins_compl_st.func_cb = get_callback_if_cpt_func(ins_compl_st.e_cpt, cpt_idx);
  return ins_compl_st.func_cb != NULL ? 1 : 0;
}

/// Sets ins_compl_st.dict = ins_compl_st.ins_buf->b_fname and
/// ins_compl_st.dict_f = DICT_EXACT.
/// Caller should have verified b_fname != NULL first.
void nvim_ins_compl_st_set_dict_from_ins_buf(void)
{
  ins_compl_st.dict   = ins_compl_st.ins_buf->b_fname;
  ins_compl_st.dict_f = DICT_EXACT;
}

/// Advances ins_compl_st.e_cpt using copy_option_part (skips to next comma-
/// separated entry). Returns 1 if rs_may_advance_cpt_index() says we should
/// advance the cpt sources index, 0 otherwise.
int nvim_ins_compl_st_advance_e_cpt(void)
{
  (void)copy_option_part(&ins_compl_st.e_cpt, IObuff, IOSIZE, ",");
  return rs_may_advance_cpt_index(ins_compl_st.e_cpt) != 0 ? 1 : 0;
}

// =============================================================================
// Phase 4 (pass 14): get_next_default_completion / ins_compl_get_next_word_or_line
// =============================================================================

/// Save p_scs, and if ins_compl_st.ins_buf has 'infercase' set, disable p_scs.
/// Returns the old p_scs value so caller can restore it.
int nvim_compl_p_scs_save_set(void)
{
  int save = (int)p_scs;
  if (ins_compl_st.ins_buf && ins_compl_st.ins_buf->b_p_inf) {
    p_scs = false;
  }
  return save;
}

/// Save p_ws and set it based on in_curbuf and the current e_cpt flag.
/// Returns the old p_ws value.
int nvim_compl_p_ws_save_set(void)
{
  int save = (int)p_ws;
  bool in_curbuf = ins_compl_st.ins_buf == curbuf;
  if (!in_curbuf) {
    p_ws = false;
  } else if (*ins_compl_st.e_cpt == '.') {
    p_ws = true;
  }
  return save;
}

/// Restore p_scs and p_ws to previously saved values.
void nvim_compl_restore_p_scs_ws(int save_p_scs, int save_p_ws)
{
  p_scs = save_p_scs != 0;
  p_ws  = save_p_ws  != 0;
}

/// Returns 1 if ins_compl_st.ins_buf == curbuf, else 0.
int nvim_ins_compl_st_is_in_curbuf(void)
{
  return (ins_compl_st.ins_buf == curbuf) ? 1 : 0;
}

/// Call the appropriate search function (fuzzy / exact-line / searchit) for one
/// step of the default-completion loop.  Increments msg_silent before the call
/// and decrements it after (to suppress wrapscan messages).
///
/// Returns OK if a new position was found, FAIL otherwise.
/// in_fuzzy: 1 if fuzzy-collect mode; start_lnum/start_col: the loop start pos.
/// fuzzy_ptr_out/fuzzy_len_out/fuzzy_score_out: set only in fuzzy mode.
int nvim_ins_compl_st_do_search(int in_fuzzy, int start_lnum, int start_col,
                                char **fuzzy_ptr_out, int *fuzzy_len_out,
                                int *fuzzy_score_out)
{
  int found = FAIL;
  msg_silent++;
  pos_T start_pos = { .lnum = (linenr_T)start_lnum, .col = (colnr_T)start_col };
  if (in_fuzzy) {
    char *leader = (char *)rs_ins_compl_leader();
    found = (int)search_for_fuzzy_match(ins_compl_st.ins_buf,
                                        ins_compl_st.cur_match_pos,
                                        leader, compl_direction,
                                        &start_pos,
                                        fuzzy_len_out, fuzzy_ptr_out, fuzzy_score_out);
  } else if (rs_ctrl_x_mode_whole_line() || rs_ctrl_x_mode_eval()
             || (compl_cont_status & CONT_SOL)) {
    found = search_for_exact_line(ins_compl_st.ins_buf, ins_compl_st.cur_match_pos,
                                  compl_direction, compl_pattern.data);
  } else {
    found = searchit(NULL, ins_compl_st.ins_buf, ins_compl_st.cur_match_pos,
                     NULL, compl_direction, compl_pattern.data, compl_pattern.size,
                     1, SEARCH_KEEP + SEARCH_NFMSG, RE_LAST, NULL);
  }
  msg_silent--;
  return found;
}

/// After a search step: if first-time or set_match_pos, update first/last match
/// positions and set compl_started.  Returns:
///   0  first/last positions were just set (caller should continue looping)
///  -1  first_match_pos == last_match_pos (caller should break with FAIL)
///   2  normal case (caller should proceed to add-step)
int nvim_ins_compl_st_check_and_update_match_pos(void)
{
  if (!compl_started || ins_compl_st.set_match_pos) {
    compl_started = true;
    ins_compl_st.first_match_pos = *ins_compl_st.cur_match_pos;
    ins_compl_st.last_match_pos  = *ins_compl_st.cur_match_pos;
    ins_compl_st.set_match_pos   = false;
    return 0;
  }
  if (ins_compl_st.first_match_pos.lnum == ins_compl_st.last_match_pos.lnum
      && ins_compl_st.first_match_pos.col == ins_compl_st.last_match_pos.col) {
    return -1;
  }
  return 2;
}

/// Sets ins_compl_st.prev_match_pos = *ins_compl_st.cur_match_pos.
void nvim_ins_compl_st_set_prev_from_cur(void)
{
  ins_compl_st.prev_match_pos = *ins_compl_st.cur_match_pos;
}

/// Read accessors for cur_match_pos, prev_match_pos, first_match_pos, last_match_pos.
int nvim_ins_compl_st_get_cur_match_lnum(void) { return (int)ins_compl_st.cur_match_pos->lnum; }
int nvim_ins_compl_st_get_cur_match_col(void)  { return (int)ins_compl_st.cur_match_pos->col; }
int nvim_ins_compl_st_get_prev_match_lnum(void) { return (int)ins_compl_st.prev_match_pos.lnum; }
int nvim_ins_compl_st_get_prev_match_col(void)  { return (int)ins_compl_st.prev_match_pos.col; }

/// Attempt to add the word or line at the current match position to the
/// completion list.  Inlines ins_compl_get_next_word_or_line logic, then
/// calls ins_compl_add_infercase.
///
/// Parameters:
///   in_fuzzy    -- 1 if in fuzzy-collect mode (ptr/len/score already known)
///   fuzzy_ptr   -- string pointer (used when in_fuzzy == 1)
///   fuzzy_len   -- length (used when in_fuzzy == 1)
///   fuzzy_score -- score (used when in_fuzzy == 1; also used for nearest-active)
///
/// Returns:
///   0  ptr is NULL or preinsert-skip (caller should continue loop)
///   1  ins_compl_add_infercase returned NOTDONE (duplicate)
///   2  match successfully added (caller should break)
int nvim_ins_compl_st_add_word_or_line(int in_fuzzy, char *fuzzy_ptr, int fuzzy_len,
                                       int fuzzy_score)
{
  char *ptr  = fuzzy_ptr;
  int   len  = fuzzy_len;
  int   score = fuzzy_score;
  bool  cont_s_ipos = false;

  if (!in_fuzzy) {
    // Inlined ins_compl_get_next_word_or_line logic:
    buf_T *ins_buf = ins_compl_st.ins_buf;
    pos_T *cur_match_pos = ins_compl_st.cur_match_pos;
    len = 0;
    ptr = ml_get_buf(ins_buf, cur_match_pos->lnum) + cur_match_pos->col;
    int raw_len = ml_get_buf_len(ins_buf, cur_match_pos->lnum) - cur_match_pos->col;
    len = raw_len;
    if (rs_ctrl_x_mode_line_or_eval()) {
      if (rs_compl_status_adding()) {
        if (cur_match_pos->lnum >= ins_buf->b_ml.ml_line_count) {
          ptr = NULL;
          goto add_word_check;
        }
        ptr = ml_get_buf(ins_buf, cur_match_pos->lnum + 1);
        len = ml_get_buf_len(ins_buf, cur_match_pos->lnum + 1);
        if (!p_paste) {
          char *tmp_ptr = ptr;
          ptr = skipwhite(tmp_ptr);
          len -= (int)(ptr - tmp_ptr);
        }
      }
    } else {
      char *tmp_ptr = ptr;
      if (rs_compl_status_adding() && compl_length <= len) {
        tmp_ptr += compl_length;
        if (vim_iswordp(tmp_ptr)) {
          ptr = NULL;
          goto add_word_check;
        }
        tmp_ptr = rs_find_word_start(tmp_ptr);
      }
      tmp_ptr = rs_find_word_end(tmp_ptr);
      len = (int)(tmp_ptr - ptr);
      if (rs_compl_status_adding() && len == compl_length) {
        if (cur_match_pos->lnum < ins_buf->b_ml.ml_line_count) {
          strncpy(IObuff, ptr, (size_t)len);  // NOLINT(runtime/printf)
          ptr = ml_get_buf(ins_buf, cur_match_pos->lnum + 1);
          tmp_ptr = ptr = skipwhite(ptr);
          tmp_ptr = rs_find_word_start(tmp_ptr);
          tmp_ptr = rs_find_word_end(tmp_ptr);
          if (tmp_ptr > ptr) {
            if (*ptr != ')' && IObuff[len - 1] != TAB) {
              if (IObuff[len - 1] != ' ') {
                IObuff[len++] = ' ';
              }
              if (p_js
                  && (IObuff[len - 2] == '.'
                      || IObuff[len - 2] == '?'
                      || IObuff[len - 2] == '!')) {
                IObuff[len++] = ' ';
              }
            }
            if (tmp_ptr - ptr >= IOSIZE - len) {
              tmp_ptr = ptr + IOSIZE - len - 1;
            }
            xstrlcpy(IObuff + len, ptr, (size_t)(IOSIZE - len));
            len += (int)(tmp_ptr - ptr);
            cont_s_ipos = true;
          }
          IObuff[len] = NUL;
          ptr = IObuff;
        }
        if (len == compl_length) {
          ptr = NULL;
          goto add_word_check;
        }
      }
    }
  }

add_word_check:
  if (ptr == NULL || (rs_ins_compl_has_preinsert()
                      && strcmp(ptr, (char *)rs_ins_compl_leader()) == 0)) {
    return 0;
  }

  if (rs_is_nearest_active() && ins_compl_st.ins_buf == curbuf) {
    score = (int)ins_compl_st.cur_match_pos->lnum - (int)curwin->w_cursor.lnum;
    if (score < 0) {
      score = -score;
    }
  }

  const char *sfname = (ins_compl_st.ins_buf == curbuf)
                       ? NULL : ins_compl_st.ins_buf->b_sfname;

  int add_r = ins_compl_add_infercase(ptr, len, p_ic, (char *)sfname,
                                      0, cont_s_ipos, score);
  if (add_r == NOTDONE) {
    return 1;
  }
  // add_r is OK (or FAIL which shouldn't happen here)
  if (in_fuzzy && compl_first_match && compl_first_match->cp_next
      && score == compl_first_match->cp_next->cp_score) {
    compl_num_bests++;
  }
  return 2;
}

void nvim_update_can_si_from_may_do_si(void) { can_si = may_do_si(); }
