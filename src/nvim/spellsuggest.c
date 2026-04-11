// spellsuggest.c: functions for spelling suggestions

#include <stdbool.h>
#include <stddef.h>

#include "nvim/buffer_defs.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/hashtab_defs.h"
#include "nvim/highlight_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/pos_defs.h"
#include "nvim/spell.h"
#include "nvim/spell_defs.h"
#include "nvim/spellfile.h"
#include "nvim/spellsuggest.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

/// Information used when looking for suggestions.
typedef struct {
  garray_T su_ga;                  ///< suggestions, contains "suggest_T"
  int su_maxcount;                 ///< max. number of suggestions displayed
  int su_maxscore;                 ///< maximum score for adding to su_ga
  int su_sfmaxscore;               ///< idem, for when doing soundfold words
  garray_T su_sga;                 ///< like su_ga, sound-folded scoring
  char *su_badptr;                 ///< start of bad word in line
  int su_badlen;                   ///< length of detected bad word in line
  int su_badflags;                 ///< caps flags for bad word
  char su_badword[MAXWLEN];        ///< bad word truncated at su_badlen
  char su_fbadword[MAXWLEN];       ///< su_badword case-folded
  char su_sal_badword[MAXWLEN];    ///< su_badword soundfolded
  hashtab_T su_banned;             ///< table with banned words
  slang_T *su_sallang;             ///< default language for sound folding
} suginfo_T;

static int spell_suggest_timeout = 5000;

#include "spellsuggest.c.generated.h"

// values for sps_flags
enum {
  SPS_BEST = 1,
  SPS_FAST = 2,
  SPS_DOUBLE = 4,
};

static int sps_flags = SPS_BEST;  ///< flags from 'spellsuggest'
static int sps_limit = 9999;      ///< max nr of suggestions given

// C accessors for sps_flags and sps_limit (used by Rust via FFI).
void nvim_spellsug_set_sps_flags(int f) { sps_flags = f; }
void nvim_spellsug_set_sps_limit(int l) { sps_limit = l; }
int nvim_spellsug_get_sps_flags(void) { return sps_flags; }
int nvim_spellsug_get_sps_limit(void) { return sps_limit; }


// Accessor for the HLF_COUNT sentinel value (used by rs_check_suggestions).
int nvim_hlf_count(void) { return (int)HLF_COUNT; }

// Accessors for suginfo_T fields (used by rs_score_combine).
// Use void* for suginfo_T* to avoid exposing the type in the generated header.
garray_T *nvim_suginfo_get_ga(void *su) { return &((suginfo_T *)su)->su_ga; }
garray_T *nvim_suginfo_get_sga(void *su) { return &((suginfo_T *)su)->su_sga; }
const char *nvim_suginfo_get_fbadword(void *su) { return ((suginfo_T *)su)->su_fbadword; }
const char *nvim_suginfo_get_badword(void *su) { return ((suginfo_T *)su)->su_badword; }
const char *nvim_suginfo_get_badptr(void *su) { return ((suginfo_T *)su)->su_badptr; }
int nvim_suginfo_get_maxscore(void *su) { return ((suginfo_T *)su)->su_maxscore; }
int nvim_suginfo_get_maxcount(void *su) { return ((suginfo_T *)su)->su_maxcount; }
void nvim_suginfo_set_ga(void *su, garray_T ga) { ((suginfo_T *)su)->su_ga = ga; }
int nvim_suginfo_get_badlen(void *su) { return ((suginfo_T *)su)->su_badlen; }
int nvim_suginfo_get_sfmaxscore(void *su) { return ((suginfo_T *)su)->su_sfmaxscore; }
void nvim_suginfo_set_sfmaxscore(void *su, int v) { ((suginfo_T *)su)->su_sfmaxscore = v; }
void nvim_suginfo_set_maxscore(void *su, int v) { ((suginfo_T *)su)->su_maxscore = v; }
hashtab_T *nvim_suginfo_get_banned(void *su) { return &((suginfo_T *)su)->su_banned; }
const char *nvim_suginfo_get_sal_badword(void *su) { return ((suginfo_T *)su)->su_sal_badword; }
slang_T *nvim_suginfo_get_sallang(void *su) { return ((suginfo_T *)su)->su_sallang; }
int nvim_suginfo_get_badflags(void *su) { return ((suginfo_T *)su)->su_badflags; }
void nvim_suginfo_set_badflags(void *su, int v) { ((suginfo_T *)su)->su_badflags = v; }
int nvim_spellsug_get_timeout(void) { return spell_suggest_timeout; }
void nvim_spellsug_set_timeout(int t) { spell_suggest_timeout = t; }
void nvim_suginfo_set_badptr(void *su, char *ptr) { ((suginfo_T *)su)->su_badptr = ptr; }
void nvim_suginfo_set_badlen(void *su, int len) { ((suginfo_T *)su)->su_badlen = len; }
void nvim_suginfo_set_maxcount(void *su, int count) { ((suginfo_T *)su)->su_maxcount = count; }
void nvim_suginfo_set_sallang(void *su, void *sallang) { ((suginfo_T *)su)->su_sallang = sallang; }
// Clear the suginfo_T struct (CLEAR_POINTER equivalent)
void nvim_suginfo_clear(void *su) { CLEAR_POINTER((suginfo_T *)su); }
// Heap-allocate and heap-free a suginfo_T (size opaque to Rust)
void *nvim_suginfo_alloc(void) { return xcalloc(1, sizeof(suginfo_T)); }
void nvim_suginfo_free(void *su) { xfree(su); }

// Accessor for curbuf->b_s.b_langp (used by rs_spell_find_suggest)
garray_T *nvim_curbuf_get_b_langp(void) { return &curbuf->b_s.b_langp; }

// Accessor for p_sps option string (used by rs_spell_find_suggest)
const char *nvim_get_p_sps(void) { return p_sps; }

// Wrapper for copy_option_part (used by rs_spell_find_suggest)
size_t nvim_copy_option_part(char **pp, char *buf, size_t maxlen, char *sep_chars)
{
  return copy_option_part(pp, buf, maxlen, sep_chars);
}

// C helper: evaluate VimL expr for spell suggestions and iterate the result
// list, calling rs_add_suggestion for each valid item.
// This keeps VimL list types out of Rust FFI.
extern void rs_add_suggestion(void *su, garray_T *gap, const char *goodword, int badlenarg,
                              int score, int altscore, bool had_bonus, slang_T *slang,
                              bool maxsf);
void nvim_spell_suggest_expr_eval(void *su, char *expr)
{
  const char *p;
  const char *su_badword = nvim_suginfo_get_badword(su);
  int su_maxscore = nvim_suginfo_get_maxscore(su);
  int su_badlen = nvim_suginfo_get_badlen(su);
  slang_T *su_sallang = nvim_suginfo_get_sallang(su);

  list_T *const list = eval_spell_expr((char *)su_badword, expr);
  if (list != NULL) {
    TV_LIST_ITER(list, li, {
      if (TV_LIST_ITEM_TV(li)->v_type == VAR_LIST) {
        int score = get_spellword(TV_LIST_ITEM_TV(li)->vval.v_list, &p);
        if (score >= 0 && score <= su_maxscore) {
          rs_add_suggestion(su, nvim_suginfo_get_ga(su), p, su_badlen,
                            score, 0, true, su_sallang, false);
        }
      }
    });
    tv_list_unref(list);
  }
}

// Macro wrappers (macros can't be used directly in Rust)
void nvim_ss_mb_ptr_back(const char *line, char **p)
{
  MB_PTR_BACK(line, *p);
}
void nvim_ss_mb_ptr_adv(char **p)
{
  MB_PTR_ADV(*p);
}

// repl_from / repl_to management
void nvim_ss_clear_repl(void) { XFREE_CLEAR(repl_from); XFREE_CLEAR(repl_to); }
void nvim_ss_set_repl_from_nsave(const char *s, size_t len) { repl_from = xstrnsave(s, len); }
void nvim_ss_set_repl_to_strdup(const char *s) { repl_to = xstrdup(s); }

// Gettext wrapper
const char *nvim_ss_gettext(const char *msg) { return _(msg); }

// Position accessors for curwin->w_cursor
int nvim_ss_get_cursor_col(void) { return (int)curwin->w_cursor.col; }
void nvim_ss_set_cursor_col(int col) { curwin->w_cursor.col = (colnr_T)col; }
int nvim_ss_get_cursor_lnum(void) { return (int)curwin->w_cursor.lnum; }
void nvim_ss_set_cursor_lnum(int lnum) { curwin->w_cursor.lnum = (linenr_T)lnum; }

// Save/restore cursor position (pos_T is complex; use col+lnum pair)
void nvim_ss_save_cursor(int *col_out, int *lnum_out)
{
  *col_out = (int)curwin->w_cursor.col;
  *lnum_out = (int)curwin->w_cursor.lnum;
}
void nvim_ss_restore_cursor(int col, int lnum)
{
  curwin->w_cursor.col = (colnr_T)col;
  curwin->w_cursor.lnum = (linenr_T)lnum;
}

// Window spell settings
int nvim_ss_get_w_p_spell(void) { return (int)curwin->w_p_spell; }
void nvim_ss_set_w_p_spell(int v) { curwin->w_p_spell = (bool)v; }
bool nvim_ss_get_w_p_rl(void) { return curwin->w_p_rl; }
const char *nvim_ss_get_b_p_spl(void) { return curwin->w_s->b_p_spl; }

// Visual mode
bool nvim_ss_get_visual_active(void) { return VIsual_active; }
int nvim_ss_get_visual_col(void) { return (int)VIsual.col; }
int nvim_ss_get_visual_lnum(void) { return (int)VIsual.lnum; }

// Message globals
int nvim_ss_get_msg_scroll(void) { return msg_scroll; }
void nvim_ss_set_msg_scroll(int v) { msg_scroll = v; }
int nvim_ss_get_rows(void) { return Rows; }
void nvim_ss_set_lines_left(int v) { lines_left = v; }
bool nvim_ss_get_cmdmsg_rl(void) { return cmdmsg_rl; }
void nvim_ss_set_cmdmsg_rl(bool v) { cmdmsg_rl = v; }
void nvim_ss_set_msg_row(int v) { msg_row = v; }
void nvim_ss_set_msg_col(int v) { msg_col = v; }
int nvim_ss_get_cmdline_row(void) { return cmdline_row; }
int nvim_ss_get_mouse_row(void) { return mouse_row; }

// IObuff/IOSIZE
char *nvim_ss_get_iobuff(void) { return IObuff; }
int nvim_ss_get_iosize(void) { return IOSIZE; }

// p_verbose
long nvim_ss_get_p_verbose(void) { return p_verbose; }

// Reset spell_suggest_timeout to default
void nvim_ss_reset_timeout(void) { spell_suggest_timeout = 5000; }

// e_no_spell error string
const char *nvim_ss_e_no_spell(void) { return e_no_spell; }

