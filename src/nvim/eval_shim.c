// eval.c: Expression evaluation.

#include <assert.h>
#include <ctype.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>

#include "auto/config.h"
#include "nvim/api/private/converter.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/channel.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/cursor.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/encode.h"
#include "nvim/eval/executor.h"
#include "nvim/eval/gc.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/event/proc.h"
#include "nvim/event/time.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/highlight_group.h"
#include "nvim/insexpand.h"
#include "nvim/keycodes.h"
#include "nvim/lib/queue_defs.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/main.h"
#include "nvim/map_defs.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/msgpack_rpc/channel_defs.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/os/fs.h"
#include "nvim/os/lang.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/shell.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/profile.h"
#include "nvim/quickfix.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/register.h"
#include "nvim/runtime.h"
#include "nvim/runtime_defs.h"
#include "nvim/strings.h"
#include "nvim/tag.h"
#include "nvim/types_defs.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

// Rust FFI declarations (typval functions migrated to Rust)
extern int tv_blob_check_index(int bloblen, varnumber_T n1, bool quiet);
extern int tv_blob_check_range(int bloblen, varnumber_T n1, varnumber_T n2, bool quiet);
extern bool tv_list_equal(list_T *l1, list_T *l2, bool ic);
extern varnumber_T tv_list_find_nr(list_T *l, int n, bool *ret_error);
extern const char *tv_list_find_str(list_T *l, int n);
extern bool tv2bool(const typval_T *tv);

extern bool rs_set_ref_in_item(typval_T *tv, int copyID, ht_stack_T **ht_stack,
                               list_stack_T **list_stack);
extern bool rs_set_ref_in_callback(Callback *callback, int copyID, ht_stack_T **ht_stack,
                                   list_stack_T **list_stack);
extern MultiQueue *rs_loop_get_events(Loop *loop);
extern bool rs_set_ref_in_callback_reader(CallbackReader *reader, int copyID,
                                          ht_stack_T **ht_stack, list_stack_T **list_stack);
extern int rs_eval_func(char **arg, evalarg_T *evalarg, char *name, int name_len,
                        typval_T *rettv, int flags, typval_T *basetv);
extern int rs_eval_index(char **arg, typval_T *rettv, evalarg_T *evalarg, bool verbose);
extern int rs_eval_method(char **arg, typval_T *rettv, evalarg_T *evalarg, bool verbose);
extern void rs_timer_close_cb(TimeWatcher *tw, void *data);

_Static_assert(VARNUMBER_MAX == INT64_MAX, "VARNUMBER_MAX mismatch");
_Static_assert(FNE_INCL_BR == 1, "FNE_INCL_BR mismatch");
_Static_assert(FNE_CHECK_START == 2, "FNE_CHECK_START mismatch");
_Static_assert(RE_MAGIC == 1, "RE_MAGIC mismatch");
_Static_assert(RE_STRING == 2, "RE_STRING mismatch");
_Static_assert(VAR_NUMBER == 1, "VAR_NUMBER mismatch");
_Static_assert(VAR_STRING == 2, "VAR_STRING mismatch");
_Static_assert(VAR_FUNC == 3, "VAR_FUNC mismatch");
_Static_assert(VAR_SPECIAL == 8, "VAR_SPECIAL mismatch");
_Static_assert(VAR_PARTIAL == 9, "VAR_PARTIAL mismatch");
_Static_assert(VAR_DICT == 5, "VAR_DICT mismatch");
_Static_assert(VAR_LIST == 4, "VAR_LIST mismatch");
_Static_assert(kCallbackNone == 0, "kCallbackNone mismatch");
_Static_assert(kCallbackFuncref == 1, "kCallbackFuncref mismatch");
_Static_assert(kCallbackPartial == 2, "kCallbackPartial mismatch");

// Phase 11: lval_T layout assertions (Rust LvalT must match exactly).
_Static_assert(sizeof(lval_T) == 96, "lval_T size mismatch: Rust LvalT must be updated");
// Phase 11: funcexe_T layout assertions (Rust FuncExeT must match exactly).
_Static_assert(sizeof(funcexe_T) == 64, "funcexe_T size mismatch: Rust FuncExeT must be updated");
_Static_assert(offsetof(funcexe_T, fe_argv_func) == 0, "funcexe_T fe_argv_func offset mismatch");
_Static_assert(offsetof(funcexe_T, fe_firstline) == 8, "funcexe_T fe_firstline offset mismatch");
_Static_assert(offsetof(funcexe_T, fe_lastline) == 12, "funcexe_T fe_lastline offset mismatch");
_Static_assert(offsetof(funcexe_T, fe_doesrange) == 16, "funcexe_T fe_doesrange offset mismatch");
_Static_assert(offsetof(funcexe_T, fe_evaluate) == 24, "funcexe_T fe_evaluate offset mismatch");
_Static_assert(offsetof(funcexe_T, fe_partial) == 32, "funcexe_T fe_partial offset mismatch");
_Static_assert(offsetof(funcexe_T, fe_selfdict) == 40, "funcexe_T fe_selfdict offset mismatch");
_Static_assert(offsetof(funcexe_T, fe_basetv) == 48, "funcexe_T fe_basetv offset mismatch");
_Static_assert(offsetof(funcexe_T, fe_found_var) == 56, "funcexe_T fe_found_var offset mismatch");
// Phase 11: typval_T size for provider.rs argvars array (3 * sizeof(typval_T)).
_Static_assert(sizeof(typval_T) == 16, "typval_T size mismatch: update provider.rs argvars stride");
_Static_assert(offsetof(lval_T, ll_name) == 0, "lval_T ll_name offset mismatch");
_Static_assert(offsetof(lval_T, ll_name_len) == 8, "lval_T ll_name_len offset mismatch");
_Static_assert(offsetof(lval_T, ll_exp_name) == 16, "lval_T ll_exp_name offset mismatch");
_Static_assert(offsetof(lval_T, ll_tv) == 24, "lval_T ll_tv offset mismatch");
_Static_assert(offsetof(lval_T, ll_li) == 32, "lval_T ll_li offset mismatch");
_Static_assert(offsetof(lval_T, ll_list) == 40, "lval_T ll_list offset mismatch");
_Static_assert(offsetof(lval_T, ll_range) == 48, "lval_T ll_range offset mismatch");
_Static_assert(offsetof(lval_T, ll_empty2) == 49, "lval_T ll_empty2 offset mismatch");
_Static_assert(offsetof(lval_T, ll_n1) == 52, "lval_T ll_n1 offset mismatch");
_Static_assert(offsetof(lval_T, ll_n2) == 56, "lval_T ll_n2 offset mismatch");
_Static_assert(offsetof(lval_T, ll_dict) == 64, "lval_T ll_dict offset mismatch");
_Static_assert(offsetof(lval_T, ll_di) == 72, "lval_T ll_di offset mismatch");
_Static_assert(offsetof(lval_T, ll_newkey) == 80, "lval_T ll_newkey offset mismatch");
_Static_assert(offsetof(lval_T, ll_blob) == 88, "lval_T ll_blob offset mismatch");
// Phase 14: evalarg_T layout assertions (Rust EvalargT must match exactly).
_Static_assert(sizeof(evalarg_T) == 32, "evalarg_T size mismatch: Rust EvalargT must be updated");
_Static_assert(offsetof(evalarg_T, eval_flags) == 0, "evalarg_T eval_flags offset mismatch");
_Static_assert(offsetof(evalarg_T, eval_getline) == 8, "evalarg_T eval_getline offset mismatch");
_Static_assert(offsetof(evalarg_T, eval_cookie) == 16, "evalarg_T eval_cookie offset mismatch");
_Static_assert(offsetof(evalarg_T, eval_tofree) == 24, "evalarg_T eval_tofree offset mismatch");

// C accessors for typval fields (used by Rust callback module)
int nvim_eval_tv_get_type(const typval_T *tv)
{
  return (int)tv->v_type;
}

partial_T *nvim_eval_tv_get_partial(const typval_T *tv)
{
  return tv->vval.v_partial;
}

int64_t nvim_eval_tv_get_vnumber(const typval_T *tv)
{
  return tv->vval.v_number;
}

// C accessors for partial fields
dict_T *nvim_eval_partial_get_dict(partial_T *pt)
{
  return pt->pt_dict;
}

int nvim_eval_partial_get_argc(partial_T *pt)
{
  return pt->pt_argc;
}

typval_T *nvim_eval_partial_get_argv(partial_T *pt, int idx)
{
  return pt->pt_argv + idx;
}

void nvim_eval_partial_incref(partial_T *pt)
{
  pt->pt_refcount++;
}

// _Static_assert for Callback layout (validated by Rust CallbackT #[repr(C)]):
_Static_assert(sizeof(Callback) == 16, "Callback size must be 16 bytes");
_Static_assert(offsetof(Callback, data) == 0, "Callback.data must be at offset 0");
_Static_assert(offsetof(Callback, type) == 8, "Callback.type must be at offset 8");

/// Accessor for p_mfd option (max function depth).
int nvim_p_mfd_get(void)
{
  return (int)p_mfd;
}

// C accessors for typval dict/list fields
dict_T *nvim_eval_tv_get_dict(const typval_T *tv)
{
  return tv->vval.v_dict;
}

list_T *nvim_eval_tv_get_list(const typval_T *tv)
{
  return tv->vval.v_list;
}

// Dict accessors
int nvim_eval_dict_get_copyid(dict_T *dd)
{
  return dd->dv_copyID;
}

void nvim_eval_dict_set_copyid(dict_T *dd, int copyid)
{
  dd->dv_copyID = copyid;
}

hashtab_T *nvim_eval_dict_get_ht(dict_T *dd)
{
  return &dd->dv_hashtab;
}

// List accessors
int nvim_eval_list_get_copyid(list_T *ll)
{
  return ll->lv_copyID;
}

void nvim_eval_list_set_copyid(list_T *ll, int copyid)
{
  ll->lv_copyID = copyid;
}

// Partial accessors for GC
int nvim_eval_partial_get_copyid(partial_T *pt)
{
  return pt->pt_copyID;
}

void nvim_eval_partial_set_copyid(partial_T *pt, int copyid)
{
  pt->pt_copyID = copyid;
}

char *nvim_eval_partial_get_name(partial_T *pt)
{
  return pt->pt_name;
}

ufunc_T *nvim_eval_partial_get_func(partial_T *pt)
{
  return pt->pt_func;
}

// CallbackReader accessors
Callback *nvim_eval_cbr_get_cb(CallbackReader *reader)
{
  return &reader->cb;
}

dict_T *nvim_eval_cbr_get_self(CallbackReader *reader)
{
  return reader->self;
}

// Hashtab iteration: iterate over entries and call rs_set_ref_in_item for each
bool nvim_eval_ht_foreach_di_tv(hashtab_T *ht, int copyID, ht_stack_T **ht_stack,
                                list_stack_T **list_stack)
{
  bool abort = false;
  HASHTAB_ITER(ht, hi, {
    abort = abort || rs_set_ref_in_item(&TV_DICT_HI2DI(hi)->di_tv, copyID, ht_stack, list_stack);
  });
  return abort;
}

// List iteration: iterate over items and call rs_set_ref_in_item for each
bool nvim_eval_list_foreach_tv(list_T *l, int copyID, ht_stack_T **ht_stack,
                               list_stack_T **list_stack)
{
  bool abort = false;
  TV_LIST_ITER(l, li, {
    if (abort) {
      break;
    }
    abort = rs_set_ref_in_item(TV_LIST_ITEM_TV(li), copyID, ht_stack, list_stack);
  });
  return abort;
}

// Dict watcher iteration
void nvim_eval_dict_foreach_watcher_callback(dict_T *dd, int copyID, ht_stack_T **ht_stack,
                                             list_stack_T **list_stack)
{
  QUEUE *w = NULL;
  DictWatcher *watcher = NULL;
  QUEUE_FOREACH(w, &dd->watchers, {
    watcher = tv_dict_watcher_node_data(w);
    rs_set_ref_in_callback(&watcher->callback, copyID, ht_stack, list_stack);
  })
}

// Stack operations for ht_stack
void nvim_eval_ht_stack_push(ht_stack_T **stack, hashtab_T *ht)
{
  ht_stack_T *newitem = xmalloc(sizeof(ht_stack_T));
  newitem->ht = ht;
  newitem->prev = *stack;
  *stack = newitem;
}

hashtab_T *nvim_eval_ht_stack_pop(ht_stack_T **stack)
{
  ht_stack_T *item = *stack;
  hashtab_T *ht = item->ht;
  *stack = item->prev;
  xfree(item);
  return ht;
}

// Stack operations for list_stack
void nvim_eval_list_stack_push(list_stack_T **stack, list_T *list)
{
  list_stack_T *newitem = xmalloc(sizeof(list_stack_T));
  newitem->list = list;
  newitem->prev = *stack;
  *stack = newitem;
}

list_T *nvim_eval_list_stack_pop(list_stack_T **stack)
{
  list_stack_T *item = *stack;
  list_T *list = item->list;
  *stack = item->prev;
  xfree(item);
  return list;
}

// C accessors for buffer operations (used by Rust indexing module)
int nvim_eval_buf_ml_valid(const buf_T *buf)
{
  return buf != NULL && buf->b_ml.ml_mfp != NULL;
}

int nvim_eval_buf_line_count(const buf_T *buf)
{
  return buf->b_ml.ml_line_count;
}

const char *nvim_eval_ml_get_buf(buf_T *buf, linenr_T lnum)
{
  return ml_get_buf(buf, lnum);
}

// C accessors for p_cpo save/restore (used by Rust pattern_match)
static char *saved_eval_p_cpo;

void nvim_eval_save_set_cpo(void)
{
  saved_eval_p_cpo = p_cpo;
  p_cpo = empty_string_option;
}

void nvim_eval_restore_cpo(void)
{
  p_cpo = saved_eval_p_cpo;
}

/// Get p_cpo pointer (accessor for rs_do_string_sub).
char *nvim_p_cpo_get(void)
{
  return p_cpo;
}

/// Set p_cpo pointer (accessor for rs_do_string_sub).
void nvim_p_cpo_set(char *val)
{
  p_cpo = val;
}

/// Get empty_string_option pointer (accessor for rs_do_string_sub).
char *nvim_empty_string_option(void)
{
  return empty_string_option;
}

/// Restore p_cpo via set_option_value_give_err when the expression changed it
/// during substitution. Handles the complex path in do_string_sub where p_cpo
/// changed but is now NUL (was changed and restored).
void nvim_do_string_sub_restore_cpo_complex(char *save_cpo)
{
  if (*p_cpo == NUL) {
    set_option_value_give_err(kOptCpoptions, CSTR_AS_OPTVAL(save_cpo), 0);
  }
  free_string_option(save_cpo);
}

#define loop_get_events(l) rs_loop_get_events(l)

/// Used for checking if local variables or arguments used in a lambda.
bool *eval_lavars_used = NULL;

typedef enum {
  GLV_FAIL,
  GLV_OK,
  GLV_STOP,
} glv_status_T;

#include "eval_shim.c.generated.h"

static uint64_t last_timer_id = 1;
static PMap(uint64_t) timers = MAP_INIT;

dict_T *get_v_event(save_v_event_T *sve)
{
  dict_T *v_event = get_vim_var_dict(VV_EVENT);

  if (v_event->dv_hashtab.ht_used > 0) {
    // recursive use of v:event, save, make empty and restore later
    sve->sve_did_save = true;
    sve->sve_hashtab = v_event->dv_hashtab;
    hash_init(&v_event->dv_hashtab);
  } else {
    sve->sve_did_save = false;
  }
  return v_event;
}

void restore_v_event(dict_T *v_event, save_v_event_T *sve)
{
  tv_dict_free_contents(v_event);
  if (sve->sve_did_save) {
    v_event->dv_hashtab = sve->sve_hashtab;
  } else {
    hash_init(&v_event->dv_hashtab);
  }
}

#if defined(EXITFREE)
void eval_clear(void)
{
  evalvars_clear();
  free_scriptnames();  // must come after evalvars_clear().
# ifdef HAVE_WORKING_LIBINTL
  free_locales();
# endif

  // autoloaded script names
  free_autoload_scriptnames();

  // unreferenced lists and dicts
  garbage_collect(false);

  // functions not garbage collected
  free_all_functions();
}

#endif

extern int rs_call_func_rettv(char **arg, evalarg_T *evalarg, typval_T *rettv, bool evaluate,
                              void *selfdict, typval_T *basetv, const char *lua_funcname);
extern int rs_eval_lambda(char **arg, typval_T *rettv, evalarg_T *evalarg, bool verbose);

/// Get TV_LIST_ITEM_NEXT(fi->fi_list, item).
listitem_T *nvim_list_item_next(list_T *l, listitem_T *item)
{
  return TV_LIST_ITEM_NEXT(l, item);
}

/// Get TV_LIST_ITEM_TV(item) (typval_T *).
typval_T *nvim_list_item_tv(listitem_T *item)
{
  return TV_LIST_ITEM_TV(item);
}

/// Call tv_list_unref(l).
void nvim_tv_list_unref(list_T *l)
{
  tv_list_unref(l);
}

/// Call tv_blob_unref(b).
void nvim_tv_blob_unref(blob_T *b)
{
  tv_blob_unref(b);
}

/// Call tv_blob_copy(from, to).
void nvim_tv_blob_copy(blob_T *from, typval_T *to)
{
  tv_blob_copy(from, to);
}

/// Call ex_let_vars with a number typval.
bool nvim_ex_let_vars_number(char *arg, varnumber_T n, bool copy, int semicolon,
                             int varcount)
{
  typval_T tv;
  tv.v_type = VAR_NUMBER;
  tv.v_lock = VAR_FIXED;
  tv.vval.v_number = n;
  return ex_let_vars(arg, &tv, copy, semicolon, varcount, false, NULL) == OK;
}

/// Call ex_let_vars with a string typval (takes ownership of s).
bool nvim_ex_let_vars_string_owned(char *arg, char *s, int semicolon, int varcount)
{
  typval_T tv;
  tv.v_type = VAR_STRING;
  tv.v_lock = VAR_FIXED;
  tv.vval.v_string = s;
  bool result = ex_let_vars(arg, &tv, true, semicolon, varcount, false, NULL) == OK;
  xfree(tv.vval.v_string);
  return result;
}

/// Call ex_let_vars with a list item typval.
bool nvim_ex_let_vars_list_item(char *arg, listitem_T *item, int semicolon, int varcount)
{
  return ex_let_vars(arg, TV_LIST_ITEM_TV(item), true, semicolon, varcount, false, NULL) == OK;
}

// Note: C callers pass cmdidx_T which implicitly converts to int (the Rust parameter type).

/// The "eval" functions have an "evalarg" argument: When NULL or
/// "evalarg->eval_flags" does not have EVAL_EVALUATE, then the argument is only
/// parsed but not executed.  The functions may return OK, but the rettv will be
/// of type VAR_UNKNOWN.  The functions still returns FAIL for a syntax error.

// directly with ga_init/ga_concat/ga_append/ga_clear/xfree (Phase 12).

/// Non-static wrapper for eval_one_expr_in_str -- used by rs_eval_interp_string.
char *nvim_eval_one_expr_in_str(char *p, garray_T *gap, bool evaluate)
{
  return eval_one_expr_in_str(p, gap, evaluate);
}

/// Get tv->vval.v_string (accessor for Rust interp_string).
char *nvim_tv_get_vstring(typval_T *tv)
{
  return tv->vval.v_string;
}

/// Set tv->v_type = VAR_STRING and tv->vval.v_string = s (takes ownership of s).
void nvim_tv_set_vstring_owned(typval_T *tv, char *s)
{
  tv->v_type = VAR_STRING;
  tv->vval.v_string = s;
}

/// Get partial_T->pt_func->uf_name (accessor for Rust).
char *nvim_partial_get_pt_func_uf_name(partial_T *pt)
{
  if (pt->pt_func != NULL) {
    return pt->pt_func->uf_name;
  }
  return NULL;
}

// =============================================================================
// GC composite iteration wrappers (for Rust garbage_collect, Phase 13)
// =============================================================================

/// Mark buffer-local variables and callbacks with copyID.
/// Iterates all buffers and calls rs_set_ref_in_item / rs_set_ref_in_callback
/// for each buffer's variables and callback functions.
/// @param abort  in/out: if true on entry, short-circuits all marking.
/// @return updated abort value.
bool nvim_gc_mark_buffers(int copyID, bool abort)
{
  FOR_ALL_BUFFERS(buf) {
    if (!abort) {
      abort = abort || rs_set_ref_in_item(&buf->b_bufvar.di_tv, copyID, NULL, NULL);
    }
    if (!abort) {
      abort = abort || rs_set_ref_in_callback(&buf->b_prompt_callback, copyID, NULL, NULL);
    }
    if (!abort) {
      abort = abort || rs_set_ref_in_callback(&buf->b_prompt_interrupt, copyID, NULL, NULL);
    }
    if (!abort) {
      abort = abort || rs_set_ref_in_callback(&buf->b_cfu_cb, copyID, NULL, NULL);
    }
    if (!abort) {
      abort = abort || rs_set_ref_in_callback(&buf->b_ofu_cb, copyID, NULL, NULL);
    }
    if (!abort) {
      abort = abort || rs_set_ref_in_callback(&buf->b_tsrfu_cb, copyID, NULL, NULL);
    }
    if (!abort) {
      abort = abort || rs_set_ref_in_callback(&buf->b_tfu_cb, copyID, NULL, NULL);
    }
    if (!abort) {
      abort = abort || rs_set_ref_in_callback(&buf->b_ffu_cb, copyID, NULL, NULL);
    }
    if (!abort && buf->b_p_cpt_cb != NULL) {
      abort = abort || set_ref_in_cpt_callbacks(buf->b_p_cpt_cb, buf->b_p_cpt_count, copyID);
    }
  }
  return abort;
}

/// Mark window-local variables (all tab windows + autocmd windows) with copyID.
/// @param abort  in/out abort value.
/// @return updated abort value.
bool nvim_gc_mark_tab_windows(int copyID, bool abort)
{
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    if (!abort) {
      abort = abort || rs_set_ref_in_item(&wp->w_winvar.di_tv, copyID, NULL, NULL);
    }
  }
  for (int i = 0; i < AUCMD_WIN_COUNT; i++) {
    if (!abort && aucmd_win[i].auc_win != NULL) {
      abort = abort
              || rs_set_ref_in_item(&aucmd_win[i].auc_win->w_winvar.di_tv, copyID, NULL, NULL);
    }
  }
  return abort;
}

/// Mark tabpage-local variables with copyID.
/// @param abort  in/out abort value.
/// @return updated abort value.
bool nvim_gc_mark_tabs(int copyID, bool abort)
{
  FOR_ALL_TABS(tp) {
    if (!abort) {
      abort = abort || rs_set_ref_in_item(&tp->tp_winvar.di_tv, copyID, NULL, NULL);
    }
  }
  return abort;
}

/// Mark channel callback references with copyID.
/// Iterates the global channels map.
/// @param abort  in/out abort value.
/// @return updated abort value.
bool nvim_gc_mark_channels(int copyID, bool abort)
{
  Channel *data;
  map_foreach_value(&channels, data, {
    if (!abort) {
      abort = abort
              || rs_set_ref_in_callback_reader(&data->on_data, copyID, NULL, NULL);
    }
    if (!abort) {
      abort = abort
              || rs_set_ref_in_callback_reader(&data->on_stderr, copyID, NULL, NULL);
    }
    if (!abort) {
      abort = abort || rs_set_ref_in_callback(&data->on_exit, copyID, NULL, NULL);
    }
  })
  return abort;
}

/// Mark timer callback references with copyID.
/// Iterates the global timers map.
/// @param abort  in/out abort value.
/// @return updated abort value.
bool nvim_gc_mark_timers(int copyID, bool abort)
{
  timer_T *timer;
  map_foreach_value(&timers, timer, {
    if (!abort) {
      abort = abort || rs_set_ref_in_callback(&timer->callback, copyID, NULL, NULL);
    }
  })
  return abort;
}

/// Iterate registers (ShaDa additional data) -- no marking, preserves side effects.
void nvim_gc_iterate_registers(void)
{
  const void *reg_iter = NULL;
  do {
    yankreg_T reg;
    char name = NUL;
    bool is_unnamed = false;
    reg_iter = op_global_reg_iter(reg_iter, &name, &reg, &is_unnamed);
  } while (reg_iter != NULL);
}

/// Iterate global marks (ShaDa additional data) -- no marking, preserves side effects.
void nvim_gc_iterate_marks(void)
{
  const void *mark_iter = NULL;
  do {
    xfmark_T fm;
    char name = NUL;
    mark_iter = mark_global_iter(mark_iter, &name, &fm);
  } while (mark_iter != NULL);
}

/// Shrink the execution stack if it is too large.
/// Mirrors the exestack compaction logic from garbage_collect.
void nvim_gc_shrink_exestack(void)
{
  if (exestack.ga_maxlen - exestack.ga_len > 500) {
    int n = exestack.ga_len / 2;
    if (n < exestack.ga_growsize) {
      n = exestack.ga_growsize;
    }
    if (exestack.ga_len + n < exestack.ga_maxlen) {
      size_t new_len = (size_t)exestack.ga_itemsize * (size_t)(exestack.ga_len + n);
      char *pp = xrealloc(exestack.ga_data, new_len);
      exestack.ga_maxlen = exestack.ga_len + n;
      exestack.ga_data = pp;
    }
  }
}

/// Clear the garbage collection trigger flags.
/// Called by rs_garbage_collect when not in testing mode.
void nvim_gc_clear_flags(void)
{
  want_garbage_collect = false;
  may_garbage_collect = false;
  garbage_collect_at_exit = false;
}

/// Emit the "not enough memory" GC abort verbose message, if p_verbose > 0.
void nvim_gc_verb_msg_abort(void)
{
  if (p_verbose > 0) {
    verb_msg(_("Not enough memory to set references, garbage collection aborted!"));
  }
}

/// Handle the kCallbackLua case: call nlua_call_ref and return LUARET_TRUTHY.
/// Retained in C because LUARET_TRUTHY is a C macro that cannot be called from Rust.
bool nvim_callback_call_lua(LuaRef luaref)
{
  Array args = ARRAY_DICT_INIT;
  Object rv = nlua_call_ref(luaref, NULL, args, kRetNilBool, NULL, NULL);
  return LUARET_TRUTHY(rv);
}

/// Get v:lua partial pointer (accessor for Rust).
partial_T *nvim_get_vlua_partial(void)
{
  return get_vim_var_partial(VV_LUA);
}

// _Static_assert for sctx_T layout:
_Static_assert(sizeof(sctx_T) == 24, "sctx_T size must be 24 bytes");

// =============================================================================
// Accessor functions for Rust FFI
// =============================================================================

/// Get did_emsg global (accessor for Rust).
int did_emsg_get(void)
{
  return did_emsg;
}

/// Get called_emsg global (accessor for Rust).
int called_emsg_get(void)
{
  return called_emsg;
}

/// Set nextcmd in exarg_T (accessor for Rust).
void exarg_set_nextcmd(exarg_T *eap, char *nextcmd)
{
  if (eap) {
    eap->nextcmd = nextcmd;
  }
}

/// Get p_ic option value (accessor for Rust).
int p_ic_get(void)
{
  return p_ic;
}

/// Set v_type in typval_T (accessor for Rust).
void nvim_tv_set_type(typval_T *tv, int vtype)
{
  if (tv) {
    tv->v_type = (VarType)vtype;
  }
}

/// Get grow array from blob (accessor for Rust).
garray_T *blob_get_ga(blob_T *blob)
{
  return blob ? &blob->bv_ga : NULL;
}

/// Wrapper for tv_blob_len inline function (accessor for Rust eval_exec).
int nvim_blob_len(const blob_T *b)
{
  return tv_blob_len(b);
}

/// Wrapper for tv_blob_get inline function (accessor for Rust eval_exec).
int nvim_blob_get(const blob_T *b, int idx)
{
  return (int)tv_blob_get(b, idx);
}

/// Wrapper for tv_blob_alloc (accessor for Rust eval_exec).
blob_T *nvim_blob_alloc(void)
{
  return tv_blob_alloc();
}

/// Clear blob's ga and free the blob - for error path in Rust eval_exec.
void nvim_blob_ga_clear_and_free(blob_T *b)
{
  if (b != NULL) {
    ga_clear(&b->bv_ga);
    xfree(b);
  }
}

/// Wrapper for tv_blob_set_ret inline function (accessor for Rust eval_exec).
void nvim_blob_set_ret(typval_T *tv, blob_T *b)
{
  tv_blob_set_ret(tv, b);
}

/// Check if typval is a function (VAR_FUNC or VAR_PARTIAL) - accessor for Rust.
int nvim_tv_is_func(const typval_T *tv)
{
  return tv->v_type == VAR_FUNC || tv->v_type == VAR_PARTIAL;
}

// =============================================================================
// Phase 1: eval_func helpers (accessor functions for rs_eval_func)
// =============================================================================

/// Set vval.v_string in typval without clearing (raw assignment) - accessor for Rust.
void nvim_tv_set_vstring_raw(typval_T *tv, char *s)
{
  tv->vval.v_string = s;
}

/// Return address of the tv_empty_string global - accessor for Rust.
const char *nvim_get_tv_empty_string(void)
{
  return tv_empty_string;
}

// =============================================================================
// Phase 2: get_lval / clear_lval helpers (lval_T accessors for rs_get_lval)
// =============================================================================

/// Wrapper for find_var with no-write mode - accessor for Rust.
/// Returns dictitem_T* for the named variable. Sets *htp if not NULL.
dictitem_T *nvim_find_var(const char *name, size_t name_len, hashtab_T **htp,
                          bool no_autoload)
{
  return find_var(name, name_len, htp, no_autoload);
}

/// Get a pointer to di->di_tv from a dictitem_T - accessor for Rust.
typval_T *nvim_di_get_tv(dictitem_T *di)
{
  return &di->di_tv;
}

/// Return address of the EVALARG_EVALUATE global - accessor for Rust.
evalarg_T *nvim_get_evalarg_evaluate_ptr(void)
{
  return &EVALARG_EVALUATE;
}

// nvim_semsg_invarg2 is in match.c (not in eval_shim.c).

// =============================================================================
// Phase 3: set_var_lval helpers (additional lval_T accessors for rs_set_var_lval)
// =============================================================================

/// Get bv_lock from a blob_T - accessor for Rust.
VarLockStatus nvim_blob_get_bv_lock(const blob_T *blob)
{
  return blob->bv_lock;
}

/// Set v_lock in a typval_T - accessor for Rust.
void nvim_tv_set_v_lock(typval_T *tv, VarLockStatus lock)
{
  tv->v_lock = lock;
}

/// Get value_check_lock condition for set_var_lval - composite accessor for Rust.
///
/// Returns true if the lock check should skip assignment (locked).
/// Mirrors: value_check_lock(lp->ll_newkey == NULL ? lp->ll_tv->v_lock
///                                                 : lp->ll_tv->vval.v_dict->dv_lock, ...)
bool nvim_lval_check_tv_lock(const lval_T *lp, const char *name)
{
  VarLockStatus lock = lp->ll_newkey == NULL
                       ? lp->ll_tv->v_lock
                       : lp->ll_tv->vval.v_dict->dv_lock;
  return value_check_lock(lock, name, TV_CSTRING);
}

/// Direct struct copy: *dst = *src for typval_T - accessor for Rust.
void nvim_tv_assign_direct(typval_T *dst, const typval_T *src)
{
  *dst = *src;
}

/// Call tv_init on a typval_T - accessor for Rust.
void nvim_tv_init(typval_T *tv)
{
  tv_init(tv);
}

/// Get di_key from a dictitem_T - accessor for Rust.
const char *nvim_di_get_key(const dictitem_T *di)
{
  return di->di_key;
}

/// Get v_lock constant VAR_UNLOCKED - accessor for Rust.
int nvim_var_unlocked(void)
{
  return VAR_UNLOCKED;
}

// nvim_value_check_lock moved to typval.c (3-param version)

/// Set vval.v_list in typval_T (raw assignment, does not update type) - accessor for Rust.
void nvim_tv_set_v_list(typval_T *tv, list_T *l)
{
  tv->vval.v_list = l;
}

/// Allocate and initialize a zero typval_T on the heap - accessor for Rust.
/// Replaces TV_INITIAL_VALUE macro (which initializes on the stack).
typval_T *nvim_tv_alloc_zero(void)
{
  typval_T *tv = xcalloc(1, sizeof(typval_T));
  tv->v_type = VAR_UNKNOWN;
  return tv;
}

/// Free a heap-allocated typval_T - accessor for Rust.
void nvim_tv_free(typval_T *tv)
{
  xfree(tv);
}

/// Check if di_flags indicate read-only and report error - accessor for Rust.
bool nvim_di_check_ro(const dictitem_T *di, const char *name)
{
  return var_check_ro(di->di_flags, name, TV_CSTRING);
}

/// Check if di has lock set and report error - accessor for Rust.
bool nvim_di_check_lock(const dictitem_T *di, const char *name)
{
  return tv_check_lock(&di->di_tv, name, TV_CSTRING);
}

/// Set lp->ll_tv to &di->di_tv - composite setter for Rust.
void nvim_lval_set_tv_from_di(lval_T *lp, dictitem_T *di)
{
  lp->ll_tv = &di->di_tv;
}

/// Wrapper for tv_dict_is_watched inline function - accessor for Rust.
bool nvim_tv_dict_is_watched(const dict_T *d)
{
  return tv_dict_is_watched(d);
}

/// Free a dictitem_T that failed to be added - accessor for Rust.
void nvim_tv_dict_item_free(dictitem_T *di)
{
  xfree(di);
}

// =============================================================================
// Phase 3: eval_list accessor wrappers for Rust
// =============================================================================

/// Wrapper for tv_list_alloc - accessor for Rust eval_exec.
list_T *nvim_eval_tv_list_alloc(ptrdiff_t len)
{
  return tv_list_alloc(len);
}

/// Wrapper for tv_list_free - accessor for Rust eval_exec.
void nvim_eval_tv_list_free(list_T *l)
{
  tv_list_free(l);
}

/// Wrapper for tv_list_append_owned_tv taking a pointer - accessor for Rust eval_exec.
/// Takes a typval_T pointer and copies by value, avoiding FFI struct-by-value issues.
void nvim_eval_tv_list_append_owned_tv_ptr(list_T *l, typval_T *tv)
{
  tv->v_lock = VAR_UNLOCKED;
  tv_list_append_owned_tv(l, *tv);
}

/// Wrapper for tv_list_set_ret inline function - accessor for Rust eval_exec.
void nvim_eval_tv_list_set_ret(typval_T *rettv, list_T *l)
{
  tv_list_set_ret(rettv, l);
}

/// Set di->di_tv = *tv with v_lock = VAR_UNLOCKED - accessor for Rust eval_dict.
void nvim_eval_di_set_tv_from_typval(dictitem_T *di, typval_T *tv)
{
  di->di_tv = *tv;
  di->di_tv.v_lock = VAR_UNLOCKED;
}

/// Wrapper for tv_dict_set_ret inline function - accessor for Rust eval_dict.
void nvim_eval_tv_dict_set_ret(typval_T *rettv, dict_T *d)
{
  tv_dict_set_ret(rettv, d);
}

/// Get argvars[1] pointer from argvars array - accessor for Rust rs_f_slice.
typval_T *nvim_f_slice_get_arg1(typval_T *argvars)
{
  return &argvars[1];
}

/// Get argvars[2] pointer from argvars array - accessor for Rust rs_f_slice.
typval_T *nvim_f_slice_get_arg2(typval_T *argvars)
{
  return &argvars[2];
}

// =============================================================================
// Phase 1 (eval_method): new C accessor/wrapper functions
// =============================================================================

/// Set tv->vval.v_partial = pt without clearing - accessor for Rust rs_eval_method.
void nvim_tv_set_partial_raw(typval_T *tv, partial_T *pt)
{
  tv->vval.v_partial = pt;
}

// =============================================================================
// Phase 1 (lval subscript): composite C accessor/wrapper functions for rs_get_lval_subscript
// =============================================================================

/// Returns true if ll_dict is v: or a: scope dict - composite accessor for Rust.
bool nvim_lval_dict_is_v_or_a_scope(const lval_T *lp)
{
  return lp->ll_dict == get_vimvar_dict()
         || &lp->ll_dict->dv_hashtab == get_funccal_args_ht();
}

/// Returns dv_scope from ll_dict - accessor for Rust.
int nvim_lval_dict_scope(const lval_T *lp)
{
  return lp->ll_dict->dv_scope;
}

/// Composite: var_check_ro || var_check_lock on di_flags - accessor for Rust.
bool nvim_lval_di_check_ro_lock(const lval_T *lp, const char *name, size_t name_len)
{
  return var_check_ro(lp->ll_di->di_flags, name, name_len)
         || var_check_lock(lp->ll_di->di_flags, name, name_len);
}

/// Set lp->ll_tv = TV_LIST_ITEM_TV(lp->ll_li) - composite setter for Rust.
void nvim_lval_set_tv_to_li_tv(lval_T *lp)
{
  lp->ll_tv = TV_LIST_ITEM_TV(lp->ll_li);
}

/// Increment dict refcount and assign to ll_tv->vval.v_dict; set ll_dict = dict - accessor for Rust.
/// Replicates: lp->ll_tv->vval.v_dict = tv_dict_alloc(); lp->ll_tv->vval.v_dict->dv_refcount++;
/// lp->ll_dict = lp->ll_tv->vval.v_dict
void nvim_lval_alloc_dict_if_null(lval_T *lp)
{
  if (lp->ll_tv->vval.v_dict == NULL) {
    lp->ll_tv->vval.v_dict = tv_dict_alloc();
    lp->ll_tv->vval.v_dict->dv_refcount++;
  }
  lp->ll_dict = lp->ll_tv->vval.v_dict;
}

/// Get tv->vval.v_blob from the ll_tv - composite accessor for Rust.
blob_T *nvim_lval_tv_get_blob(const lval_T *lp)
{
  return lp->ll_tv->vval.v_blob;
}

/// Get tv->vval.v_list from the ll_tv - composite accessor for Rust.
list_T *nvim_lval_tv_get_list(const lval_T *lp)
{
  return lp->ll_tv->vval.v_list;
}

/// Get v_type from the ll_tv - composite accessor for Rust.
int nvim_lval_tv_get_type(const lval_T *lp)
{
  return lp->ll_tv->v_type;
}

/// Alloc list and set ll_tv - composite accessor for Rust.
void nvim_lval_tv_list_alloc_ret(lval_T *lp)
{
  tv_list_alloc_ret(lp->ll_tv, kListLenUnknown);
}

/// Alloc blob and set ll_tv - composite accessor for Rust.
void nvim_lval_tv_blob_alloc_ret(lval_T *lp)
{
  tv_blob_alloc_ret(lp->ll_tv);
}

/// Get tv->v_type from ll_tv - same as nvim_lval_tv_get_type but returns int - accessor for Rust.
/// Also get tv_blob_len from ll_tv blob.
int nvim_lval_tv_blob_len(const lval_T *lp)
{
  return tv_blob_len(lp->ll_tv->vval.v_blob);
}

/// Set lp->ll_di = tv_dict_find(lp->ll_dict, key, len) - composite setter for Rust.
void nvim_lval_set_di_from_dict(lval_T *lp, const char *key, int len)
{
  lp->ll_di = tv_dict_find(lp->ll_dict, key, (ptrdiff_t)len);
}

/// Set lp->ll_tv = &lp->ll_di->di_tv - composite setter for Rust.
void nvim_lval_set_tv_from_ll_di(lval_T *lp)
{
  lp->ll_tv = &lp->ll_di->di_tv;
}

/// Check if lp->ll_di is NULL - accessor for Rust.
bool nvim_lval_di_is_null(const lval_T *lp)
{
  return lp->ll_di == NULL;
}

/// tv_blob_check_index wrapper - accessor for Rust.
int nvim_tv_blob_check_index(int bloblen, int n1, bool quiet)
{
  return tv_blob_check_index(bloblen, (varnumber_T)n1, quiet);
}

/// tv_blob_check_range wrapper - accessor for Rust.
int nvim_tv_blob_check_range(int bloblen, int n1, int n2, bool quiet)
{
  return tv_blob_check_range(bloblen, (varnumber_T)n1, (varnumber_T)n2, quiet);
}

/// tv_list_check_range_index_one returning opaque listitem_T* - accessor for Rust.
listitem_T *nvim_tv_list_check_range_index_one(lval_T *lp, bool quiet)
{
  return tv_list_check_range_index_one(lp->ll_list, &lp->ll_n1, quiet);
}

/// tv_list_check_range_index_two via lp fields - accessor for Rust.
int nvim_tv_list_check_range_index_two(lval_T *lp, bool quiet)
{
  return tv_list_check_range_index_two(lp->ll_list, &lp->ll_n1, lp->ll_li, &lp->ll_n2, quiet);
}

// =============================================================================
// Phase 5 (handle_subscript): new C accessor/wrapper functions
// =============================================================================

/// Increment dict->dv_refcount - accessor for Rust rs_handle_subscript.
void nvim_dict_refcount_inc(dict_T *dict)
{
  if (dict != NULL) {
    dict->dv_refcount++;
  }
}

/// tv_dict_unref wrapper - accessor for Rust rs_handle_subscript.
void nvim_dict_unref(dict_T *dict)
{
  tv_dict_unref(dict);
}

/// make_partial wrapper - accessor for Rust rs_handle_subscript.
void nvim_make_partial(dict_T *selfdict, typval_T *rettv)
{
  make_partial(selfdict, rettv);
}

/// aborting() wrapper - accessor for Rust rs_handle_subscript.
bool nvim_aborting(void)
{
  return aborting();
}

/// Get partial->pt_auto - accessor for Rust rs_handle_subscript.
bool nvim_partial_get_pt_auto(const partial_T *pt)
{
  return pt->pt_auto;
}

/// Scope check for get_lval_dict_item: set key[len]=NUL, check scope, restore.
/// Returns true if the variable is 'wrong' (validation failed).
bool nvim_lval_dict_scope_check(lval_T *lp, char *key, int len, const typval_T *rettv)
{
  char prevval;
  if (len != -1) {
    prevval = key[len];
    key[len] = NUL;
  } else {
    prevval = 0;
  }
  bool wrong = ((lp->ll_dict->dv_scope == VAR_DEF_SCOPE
                 && tv_is_func(*rettv)
                 && var_wrong_func_name(key, lp->ll_di == NULL))
                || !valid_varname(key));
  if (len != -1) {
    key[len] = prevval;
  }
  return wrong;
}

// =============================================================================
// Phase 6 (ex_echo + ex_execute): new C accessor/wrapper functions
// =============================================================================

/// encode_tv2echo wrapper - accessor for Rust.
char *nvim_encode_tv2echo(typval_T *tv)
{
  return encode_tv2echo(tv, NULL);
}

/// encode_tv2string wrapper - accessor for Rust.
char *nvim_encode_tv2string_wrapper(typval_T *tv)
{
  return encode_tv2string(tv, NULL);
}

/// tv_get_string wrapper for ex_execute - accessor for Rust.
/// Uses nvim_eval_tv_get_str to avoid conflict with nvim_tv_get_string in eval/typval.c.
const char *nvim_eval_tv_get_str(const typval_T *tv)
{
  return tv_get_string(tv);
}

/// msg_sb_eol wrapper - accessor for Rust.
void nvim_msg_sb_eol(void)
{
  msg_sb_eol();
}

/// Set msg_ext_append global - accessor for Rust.
void nvim_set_msg_ext_append(bool val)
{
  msg_ext_append = val;
}

/// emsg_multiline for echoerr - accessor for Rust.
/// Wraps: emsg_multiline(str, "echoerr", HLF_E, true)
void nvim_emsg_multiline_echoerr(const char *str)
{
  emsg_multiline(str, "echoerr", HLF_E, true);
}

/// Get force_abort global - accessor for Rust.
int nvim_get_force_abort(void);  // forward: defined in ex_eval.c

/// msg() wrapper for echomsg - accessor for Rust.
void nvim_msg_echomsg(const char *str, int hl_id)
{
  msg(str, hl_id);
}

/// do_cmdline wrapper for :execute - accessor for Rust.
void nvim_do_cmdline_execute(char *cmd, exarg_T *eap)
{
  do_cmdline(cmd, eap->ea_getline, eap->cookie, DOCMD_NOWAIT|DOCMD_VERBOSE);
}


/// Get eap->skip - accessor for Rust (local, avoids dependency on ex_docmd).
int nvim_eap_get_skip_local(const exarg_T *eap)
{
  return eap->skip;
}

/// Get eap->arg - accessor for Rust (local).
char *nvim_eap_get_arg_local(const exarg_T *eap)
{
  return eap->arg;
}

/// Set eap->nextcmd from check_nextcmd - accessor for Rust.
void nvim_eap_set_nextcmd_checked(exarg_T *eap, char *arg)
{
  eap->nextcmd = check_nextcmd(arg);
}


// =============================================================================
// Phase 2 eval_top accessors - eval_to_* and eval_expr_* family
// =============================================================================


/// Heap-allocate a funccal_entry_T and call save_funccal - accessor for Rust.
/// Returns opaque void* so callers don't need the full type definition.
void *nvim_eval_save_funccal(void)
{
  funccal_entry_T *entry = xcalloc(1, sizeof(funccal_entry_T));
  save_funccal(entry);
  return entry;
}

/// Call restore_funccal and free the entry - accessor for Rust.
/// Takes opaque void* matching the return type of nvim_eval_save_funccal.
void nvim_eval_restore_funccal(void *entry)
{
  restore_funccal();
  xfree(entry);
}

/// may_call_simple_func wrapper - accessor for Rust eval_top.
int nvim_eval_may_call_simple_func(const char *arg, typval_T *rettv)
{
  return may_call_simple_func(arg, rettv);
}

/// tv_list_join with newline separator - wrapper for typval2string.
/// Appends all list items joined by "\n" followed by NL if list non-empty, then NUL.
/// Returns heap-allocated string (caller must xfree).
char *nvim_eval_tv_list_join_nl(list_T *l)
{
  garray_T ga;
  ga_init(&ga, (int)sizeof(char), 80);
  if (l != NULL) {
    tv_list_join(&ga, l, "\n");
    if (tv_list_len(l) > 0) {
      ga_append(&ga, NL);
    }
  }
  ga_append(&ga, NUL);
  return (char *)ga.ga_data;
}

/// xstrdup wrapper - accessor for eval_top.
char *nvim_eval_xstrdup(const char *s)
{
  return xstrdup(s);
}

// ============================================================================
// Phase 3: C accessors for var2fpos / list2fpos (used by Rust indexing module)
// ============================================================================

// NvimCursorVisualState typedef is in eval.h (Phase 15).
_Static_assert(sizeof(NvimCursorVisualState) == 40,
               "NvimCursorVisualState size mismatch: expected 40 bytes");

/// Bulk-read cursor/visual state into *out.
void nvim_read_cursor_visual_state(NvimCursorVisualState *out)
{
  out->cursor_lnum = curwin->w_cursor.lnum;
  out->cursor_col = curwin->w_cursor.col;
  out->cursor_coladd = curwin->w_cursor.coladd;
  out->topline = curwin->w_topline;
  out->botline = curwin->w_botline;
  out->visual_active = VIsual_active;
  out->visual_lnum = VIsual.lnum;
  out->visual_col = VIsual.col;
  out->visual_coladd = VIsual.coladd;
  out->curbuf_fnum = curbuf->b_fnum;
}

/// Get curbuf->b_fnum.
int nvim_curbuf_fnum(void)
{
  return curbuf->b_fnum;
}

/// mark_get wrapper for Rust var2fpos.
/// Returns true if mark was found and is valid (lnum > 0).
/// Fills lnum_out, col_out, coladd_out, fnum_out when returning true.
bool nvim_mark_get_wrapper(int mname, int32_t *lnum_out, int *col_out, int *coladd_out,
                           int *fnum_out)
{
  const fmark_T *const fm = mark_get(curbuf, curwin, NULL, kMarkAll, mname);
  if (fm == NULL || fm->mark.lnum <= 0) {
    return false;
  }
  *lnum_out = fm->mark.lnum;
  *col_out = fm->mark.col;
  *coladd_out = fm->mark.coladd;
  *fnum_out = fm->fnum;
  return true;
}

/// Call update_topline(curwin).
void nvim_update_topline_curwin(void)
{
  update_topline(curwin);
}

/// Call check_cursor_moved(curwin).
void nvim_check_cursor_moved_curwin(void)
{
  check_cursor_moved(curwin);
}

/// tv_list_find_nr wrapper with bool error output.
/// Returns the number at list index n. Sets *error_out to true on error.
int64_t nvim_tv_list_find_nr(list_T *l, int n, bool *error_out)
{
  return (int64_t)tv_list_find_nr(l, n, error_out);
}

/// Returns true if the list item at index idx is a string equal to "$".
bool nvim_tv_list_item_is_dollar(list_T *l, int idx)
{
  listitem_T *li = tv_list_find(l, idx);
  return li != NULL
         && TV_LIST_ITEM_TV(li)->v_type == VAR_STRING
         && TV_LIST_ITEM_TV(li)->vval.v_string != NULL
         && strcmp(TV_LIST_ITEM_TV(li)->vval.v_string, "$") == 0;
}

/// tv_list_len wrapper.
int nvim_tv_list_len(const list_T *l)
{
  return tv_list_len(l);
}

/// tv_get_string_chk wrapper for Rust (Phase 3 accessor, no out_len).
const char *nvim_eval_tv_string_chk(const typval_T *tv)
{
  return tv_get_string_chk(tv);
}

/// mb_charlen(ml_get(lnum)) for current buffer.
int nvim_mb_charlen_ml(int32_t lnum)
{
  return mb_charlen(ml_get(lnum));
}

/// mb_charlen(get_cursor_line_ptr()) wrapper.
int nvim_get_cursor_line_charlen(void)
{
  return mb_charlen(get_cursor_line_ptr());
}

// =============================================================================
// Phase 1 (eval_shim pass 4): C accessors for rs_call_func_rettv / rs_eval_lambda
// =============================================================================

/// Wrap get_lambda_tv for Rust rs_eval_lambda.
int nvim_get_lambda_tv(char **arg, typval_T *rettv, evalarg_T *evalarg)
{
  return get_lambda_tv(arg, rettv, evalarg);
}

/// Raw-copy a typval_T by value from src to dst (memcpy of sizeof(typval_T)).
/// Sets src->v_type to VAR_UNKNOWN after the copy.
/// Used by rs_call_func_rettv to implement `functv = *rettv; rettv->v_type = VAR_UNKNOWN`.
void nvim_tv_raw_copy_and_reset(typval_T *dst, typval_T *src)
{
  *dst = *src;
  src->v_type = VAR_UNKNOWN;
}

// =============================================================================
// Accessors for Phase 2 (eval_shim pass 4): eval_option + eval_env_var
// =============================================================================

/// Wraps find_option_var_end: parse &[g:|l:]optname from *arg.
/// On success, *arg is set to "optname" and returned value is pointer after name.
/// opt_idxp and opt_flagsp are set.
/// Returns NULL when no option name found (error).
const char *nvim_find_option_var_end(const char **arg, int *opt_idxp, int *opt_flagsp)
{
  OptIndex opt_idx = kOptInvalid;
  int opt_flags = 0;
  const char *end = find_option_var_end(arg, &opt_idx, &opt_flags);
  *opt_idxp = (int)opt_idx;
  *opt_flagsp = opt_flags;
  return end;
}

/// Get option value as typval using get_option_value() + optval_as_tv().
/// opt_idx must not be kOptInvalid.
void nvim_get_option_value_as_tv(int opt_idx, int opt_flags, typval_T *rettv)
{
  OptVal value = get_option_value((OptIndex)opt_idx, opt_flags);
  assert(value.type != kOptValTypeNil);
  *rettv = optval_as_tv(value, true);
}

/// Get tty option value as typval using get_tty_option() + optval_as_tv().
void nvim_get_tty_option_as_tv(const char *name, typval_T *rettv)
{
  OptVal value = get_tty_option(name);
  assert(value.type != kOptValTypeNil);
  *rettv = optval_as_tv(value, true);
}

/// Call vim_getenv(name) - returns allocated string or NULL.
/// Caller must xfree the result.
char *nvim_vim_getenv(const char *name)
{
  return vim_getenv(name);
}

/// Call expand_env_save(src) - expands $VAR style from src.
/// Returns allocated string. Caller must xfree.
char *nvim_expand_env_save(const char *src)
{
  return expand_env_save((char *)src);
}

// =============================================================================
// Accessors for Phase 3 (eval_shim pass 4): var_item_copy
// =============================================================================

/// Get conv->vc_type; returns CONV_NONE (0) if conv is NULL.
int nvim_vimconv_get_type(const vimconv_T *conv)
{
  return conv == NULL ? CONV_NONE : (int)conv->vc_type;
}

/// Wrap string_convert(conv, str, NULL).
/// Returns converted string (allocated) or NULL. Caller must xfree.
char *nvim_string_convert(const vimconv_T *conv, const char *str)
{
  return string_convert((vimconv_T *)conv, (char *)str, NULL);
}

/// Get tv_list_copyid(list).
int nvim_tv_list_copyid(const list_T *list)
{
  return tv_list_copyid(list);
}

/// Get tv_list_latest_copy(list).
list_T *nvim_tv_list_latest_copy(const list_T *list)
{
  return tv_list_latest_copy(list);
}

/// Call tv_list_ref(list) to increment refcount.
void nvim_tv_list_ref(list_T *list)
{
  tv_list_ref(list);
}

/// Call tv_list_copy(conv, list, deep, copyID) - deep copy a list.
list_T *nvim_tv_list_copy(const vimconv_T *conv, list_T *list, bool deep, int copyID)
{
  return tv_list_copy(conv, list, deep, copyID);
}

/// Set tv->vval.v_list = list.
void nvim_tv_set_list(typval_T *tv, list_T *list)
{
  tv->vval.v_list = list;
}

/// Get dict->dv_copydict.
dict_T *nvim_dict_get_copydict(const dict_T *dict)
{
  return dict->dv_copydict;
}

/// Call tv_dict_copy(conv, dict, deep, copyID) - deep copy a dict.
dict_T *nvim_tv_dict_copy(const vimconv_T *conv, dict_T *dict, bool deep, int copyID)
{
  return tv_dict_copy(conv, dict, deep, copyID);
}

/// Set tv->vval.v_dict = dict.
void nvim_tv_set_dict(typval_T *tv, dict_T *dict)
{
  tv->vval.v_dict = dict;
}

// =============================================================================
// Accessors for Phase 4 (eval_shim pass 4): save_tv_as_string
// =============================================================================

/// Get first item of list (tv_list_first). Returns NULL for empty/NULL list.
listitem_T *nvim_list_first_item(const list_T *l)
{
  return tv_list_first(l);
}

/// Call tv_get_string on a listitem's tv.
const char *nvim_list_item_get_string(listitem_T *item)
{
  return tv_get_string(TV_LIST_ITEM_TV(item));
}

// =============================================================================
// Accessors for Phase 5 (eval_shim pass 4): fill_evalarg_from_eap,
//   clear_evalarg, may_call_simple_func, eval_expr_ext, partial_unref,
//   typval_tostring
// =============================================================================

/// Get eap->cmdline_tofree - accessor for Rust clear_evalarg.
char *nvim_eap_get_cmdline_tofree(exarg_T *eap)
{
  return eap->cmdline_tofree;
}

/// Set eap->cmdline_tofree - accessor for Rust clear_evalarg.
void nvim_eap_set_cmdline_tofree(exarg_T *eap, char *val)
{
  eap->cmdline_tofree = val;
}

/// Get *eap->cmdlinep (dereference the cmdlinep pointer) - accessor for Rust.
char *nvim_eap_get_cmdlinep_deref(const exarg_T *eap)
{
  return *eap->cmdlinep;
}

/// Set *eap->cmdlinep = val - accessor for Rust.
void nvim_eap_set_cmdlinep_deref(exarg_T *eap, char *val)
{
  *eap->cmdlinep = val;
}

/// Get eap->ea_getline function pointer - accessor for Rust fill_evalarg_from_eap.
LineGetter nvim_eap_get_getline(const exarg_T *eap)
{
  return eap->ea_getline;
}

/// Get eap->cookie - accessor for Rust fill_evalarg_from_eap.
void *nvim_eap_get_cookie(const exarg_T *eap)
{
  return eap->cookie;
}

/// Wrapper for call_simple_luafunc - accessor for Rust may_call_simple_func.
int nvim_call_simple_luafunc(const char *name, size_t len, typval_T *rettv)
{
  return call_simple_luafunc(name, len, rettv);
}

/// Wrapper for call_simple_func - accessor for Rust may_call_simple_func.
int nvim_call_simple_func(const char *name, size_t len, typval_T *rettv)
{
  return call_simple_func(name, len, rettv);
}

/// Allocate exactly sizeof(typval_T) bytes for a heap typval - accessor for Rust.
typval_T *nvim_alloc_typval(void)
{
  return xmalloc(sizeof(typval_T));
}

/// Wrapper for func_unref - accessor for Rust partial_free.
void nvim_func_unref(char *name)
{
  func_unref(name);
}

/// Wrapper for func_ptr_unref - accessor for Rust partial_free.
void nvim_func_ptr_unref(ufunc_T *func)
{
  func_ptr_unref(func);
}

/// Decrement pt->pt_refcount and return true if it drops to <= 0.
/// Accessor for Rust partial_unref.
bool nvim_partial_decref_and_check(partial_T *pt)
{
  return --pt->pt_refcount <= 0;
}

// =============================================================================
// Accessors for Phase 1 (eval_shim pass 5): call_vim_function family +
//   set_argv_var, var_set_global, eval_fmt_source_name_line, find_option_var_end
// =============================================================================

/// Wrap set_var(name, name_len, tv, false) - accessor for rs_var_set_global.
void nvim_set_var_wrapper(const char *name, size_t name_len, typval_T *tv)
{
  set_var(name, name_len, tv, false);
}

/// Wrap set_vim_var_list(VV_ARGV, list) - accessor for rs_set_argv_var.
void nvim_set_vim_var_argv_list(list_T *list)
{
  set_vim_var_list(VV_ARGV, list);
}

/// Return SOURCING_NAME - accessor for rs_eval_fmt_source_name_line.
const char *nvim_sourcing_name_get(void)
{
  return SOURCING_NAME;
}

/// Return SOURCING_LNUM - accessor for rs_eval_fmt_source_name_line.
linenr_T nvim_sourcing_lnum_get(void)
{
  return SOURCING_LNUM;
}

/// Wrap find_option_end(p, opt_idxp) - accessor for rs_find_option_var_end.
/// Returns pointer after option name, or NULL on failure.
const char *nvim_find_option_end_wrapper(const char *p, int *opt_idxp)
{
  OptIndex opt_idx = kOptInvalid;
  const char *end = find_option_end(p, &opt_idx);
  *opt_idxp = (int)opt_idx;
  return end;
}

/// Wrap tv_list_set_lock - accessor for rs_set_argv_var.
void nvim_tv_list_set_lock(list_T *l, int lock)
{
  tv_list_set_lock(l, (VarLockStatus)lock);
}

/// Wrap tv_list_append_string - accessor for rs_set_argv_var.
void nvim_tv_list_append_string(list_T *l, const char *str, ssize_t len)
{
  tv_list_append_string(l, str, len);
}

/// Get tv_list_last item's typval v_lock field address - accessor for rs_set_argv_var.
/// Sets VAR_FIXED lock on the last item's tv.
void nvim_tv_list_last_fix_lock(list_T *l)
{
  TV_LIST_ITEM_TV(tv_list_last(l))->v_lock = VAR_FIXED;
}


// =============================================================================
// Accessors for Phase 2 (eval_shim pass 5): prompt functions
// Phase 16: consolidated into NvimPromptState bulk struct.
// =============================================================================

/// Bulk-read prompt state from curbuf into *out - accessor for Rust prompt functions.
void nvim_read_prompt_state(NvimPromptState *out)
{
  out->curbuf = curbuf;
  out->ml_line_count = (int32_t)curbuf->b_ml.ml_line_count;
  out->prompt_start_lnum = (int32_t)curbuf->b_prompt_start.mark.lnum;
  out->prompt_callback = &curbuf->b_prompt_callback;
  out->prompt_interrupt = &curbuf->b_prompt_interrupt;
}

/// Write back prompt_start_lnum to curbuf - accessor for Rust prompt functions.
void nvim_write_prompt_start_lnum(int32_t lnum)
{
  curbuf->b_prompt_start.mark.lnum = (linenr_T)lnum;
}

/// Get buf->b_prompt_start.mark.lnum - accessor for rs_prompt_get_input.
linenr_T nvim_buf_get_prompt_start_lnum(buf_T *buf)
{
  return buf->b_prompt_start.mark.lnum;
}

/// Wrap appended_lines_mark(lnum, count) - accessor for rs_prompt_invoke_callback.
void nvim_appended_lines_mark(linenr_T lnum, int count)
{
  appended_lines_mark(lnum, count);
}

/// Wrap u_clearallandblockfree(curbuf) - accessor for rs_prompt_invoke_callback.
void nvim_curbuf_u_clearallandblockfree(void)
{
  u_clearallandblockfree(curbuf);
}

// =============================================================================
// Accessors for Phase 3 (eval_shim pass 5): eval_foldexpr and eval_foldtext
// Phase 16: consolidated into NvimFoldEvalState bulk struct.
// =============================================================================

/// Bulk-read fold eval state from wp into *out - accessor for Rust fold functions.
void nvim_read_fold_eval_state(win_T *wp, NvimFoldEvalState *out)
{
  out->insecure_foldexpr = was_set_insecurely(wp, kOptFoldexpr, OPT_LOCAL);
  out->insecure_foldtext = was_set_insecurely(wp, kOptFoldtext, OPT_LOCAL);
  out->foldexpr = skipwhite(wp->w_p_fde);
  out->foldtext = wp->w_p_fdt;
}

/// Save current_sctx and set it from wp's foldexpr script context.
/// Returns an opaque pointer to the saved sctx (caller must call nvim_restore_current_sctx).
sctx_T *nvim_fold_sctx_save_and_set(win_T *wp)
{
  sctx_T *saved = xmalloc(sizeof(sctx_T));
  *saved = current_sctx;
  current_sctx = wp->w_p_script_ctx[kWinOptFoldexpr];
  return saved;
}

/// Restore current_sctx from an opaque pointer and free it.
void nvim_restore_current_sctx(sctx_T *saved)
{
  current_sctx = *saved;
  xfree(saved);
}

/// Construct an Object result for foldtext evaluation.
/// tv_type: VAR_LIST (4) -> vim_to_object; otherwise -> STRING_OBJ(cstr_to_string(tv_get_string)).
/// If tv is NULL, write STRING_OBJ(NULL_STRING).
void nvim_foldtext_make_obj(typval_T *tv, int tv_type, Object *out)
{
  if (tv == NULL) {
    *out = STRING_OBJ(NULL_STRING);
  } else if (tv_type == VAR_LIST) {
    *out = vim_to_object(tv, NULL, false);
  } else {
    *out = STRING_OBJ(cstr_to_string(tv_get_string(tv)));
  }
}

// =============================================================================
// Accessors for Phase 4 (eval_shim pass 5): get_name_len / make_expanded_name
// =============================================================================

/// Wrap eval_fname_script - accessor for rs_get_name_len.
int nvim_eval_fname_script(const char *p)
{
  return eval_fname_script(p);
}


// =============================================================================
// Accessors for Phase 2 (eval_shim pass 6): tv_to_argv + system output
// =============================================================================

/// shell_build_argv wrapper -- build an argument vector from a shell command string.
char **nvim_shell_build_argv(const char *cmd, const char *extra)
{
  return shell_build_argv(cmd, extra);
}

/// shell_free_argv wrapper -- free an argument vector built by nvim_shell_build_argv.
void nvim_shell_free_argv(char **argv)
{
  shell_free_argv(argv);
}

/// shell_argv_to_str wrapper -- convert argv to a printable string (caller must free).
char *nvim_shell_argv_to_str(char **const argv)
{
  return shell_argv_to_str(argv);
}

/// os_system wrapper -- run command and capture output.
/// Returns exit status. On success, *output_out and *nread_out are set.
int nvim_os_system(char **argv, const char *input, size_t input_len,
                   char **output_out, size_t *nread_out)
{
  return os_system(argv, input, input_len, output_out, nread_out);
}

/// encode_list_write wrapper -- write a NL-separated string into a VimL list.
void nvim_encode_list_write(list_T *list, const char *str, size_t len)
{
  encode_list_write(list, str, len);
}

/// set_vim_var_nr wrapper for VV_SHELL_ERROR.
void nvim_set_shell_error(int status)
{
  set_vim_var_nr(VV_SHELL_ERROR, (varnumber_T)status);
}

/// memchrsub wrapper for system output -- replace all occurrences of c with x.
void nvim_eval_memchrsub(char *data, char c, char x, size_t len)
{
  memchrsub(data, c, x, len);
}

/// Emit the "Executing command: ..." verbose message.
void nvim_smsg_system_cmd(const char *cmdstr)
{
  smsg(0, _("Executing command: \"%s\""), cmdstr);
}

/// Get p_verbose (the 'verbose' option value).
int nvim_p_verbose_get(void)
{
  return (int)p_verbose;
}

/// do_profiling accessor -- returns true when profiling is active (PROF_YES).
bool nvim_do_profiling_active(void)
{
  return do_profiling == PROF_YES;
}

/// prof_child_enter wrapper -- record start of child profiling.
void nvim_prof_child_enter(uint64_t *tm)
{
  prof_child_enter((proftime_T *)tm);
}

/// prof_child_exit wrapper -- record end of child profiling.
void nvim_prof_child_exit(uint64_t *tm)
{
  prof_child_exit((proftime_T *)tm);
}

/// tv_list_alloc_ret wrapper -- alloc list and assign it to rettv.
list_T *nvim_tv_list_alloc_ret(typval_T *rettv, ptrdiff_t count_hint)
{
  return tv_list_alloc_ret(rettv, count_hint);
}

/// os_can_exe wrapper for tv_to_argv -- check if the command is executable.
/// Returns true if executable. Sets *abspath to the resolved path (caller must free).
bool nvim_eval_os_can_exe(const char *name, char **abspath)
{
  return os_can_exe(name, abspath, true);
}

// =============================================================================
// Accessors for Phase 3 (eval_shim pass 6): provider infrastructure
// =============================================================================

/// eval_variable wrapper for Rust.
int nvim_eval_variable(const char *name, int len, typval_T *rettv, bool verbose,
                       bool import_script)
{
  return eval_variable(name, len, rettv, NULL, verbose, import_script);
}

/// script_autoload wrapper for Rust.
bool nvim_script_autoload(const char *name, size_t name_len, bool reload)
{
  return script_autoload(name, name_len, reload);
}

/// find_func wrapper for Rust -- returns non-NULL if function is defined.
bool nvim_eval_find_func(const char *name)
{
  return find_func(name) != NULL;
}

/// p_lpl accessor for Rust.
bool nvim_eval_get_p_lpl(void)
{
  return p_lpl;
}

/// nlua_is_deferred_safe accessor for Rust.
bool nvim_eval_nlua_is_deferred_safe(void)
{
  return nlua_is_deferred_safe();
}

/// Save the provider_caller_scope and related globals to an opaque heap blob.
/// Returns a pointer that must be passed to nvim_restore_provider_caller_scope.
void *nvim_save_provider_caller_scope(void)
{
  struct caller_scope *saved = xmalloc(sizeof(struct caller_scope));
  *saved = provider_caller_scope;
  provider_caller_scope = (struct caller_scope){
    .script_ctx = current_sctx,
    .es_entry = ((estack_T *)exestack.ga_data)[exestack.ga_len - 1],
    .autocmd_fname = autocmd_fname,
    .autocmd_match = autocmd_match,
    .autocmd_fname_full = autocmd_fname_full,
    .autocmd_bufnr = autocmd_bufnr,
    .funccalp = (void *)get_current_funccal()
  };
  return saved;
}

/// Restore the provider_caller_scope from the saved blob and free it.
void nvim_restore_provider_caller_scope(void *saved)
{
  provider_caller_scope = *(struct caller_scope *)saved;
  xfree(saved);
}

/// Increment provider_call_nesting.
void nvim_provider_call_nesting_inc(void)
{
  provider_call_nesting++;
}

/// Decrement provider_call_nesting (with assertion).
void nvim_provider_call_nesting_dec(void)
{
  provider_call_nesting--;
  assert(provider_call_nesting >= 0);
}

/// tv_list_alloc with explicit count (for provider args list).
list_T *nvim_eval_list_alloc_n(int n)
{
  return tv_list_alloc((ptrdiff_t)n);
}

/// tv_list_ref wrapper for provider list argument.
void nvim_eval_list_ref(list_T *l)
{
  tv_list_ref(l);
}

// nvim_eval_save_funccal and nvim_eval_restore_funccal already defined above (line 3567).

/// Set typval_T to a VAR_NUMBER 0 return (provider not found fallback).
void nvim_tv_set_number_zero(typval_T *tv)
{
  tv->v_type = VAR_NUMBER;
  tv->v_lock = VAR_UNLOCKED;
  tv->vval.v_number = 0;
}

// =============================================================================
// Timer accessors for Rust timer migration (Phase 8)
// =============================================================================

/// Allocate and zero-initialize a timer_T.
timer_T *nvim_timer_alloc(void)
{
  return xcalloc(1, sizeof(timer_T));
}

/// Free a timer_T (only after refcount reaches 0).
void nvim_timer_free(timer_T *timer)
{
  xfree(timer);
}

// nvim_timer_get_id, nvim_timer_set_id, nvim_timer_get_repeat_count, nvim_timer_set_repeat_count,
// nvim_timer_get_refcount, nvim_timer_set_refcount, nvim_timer_get_emsg_count,
// nvim_timer_set_emsg_count, nvim_timer_get_timeout, nvim_timer_get_stopped,
// nvim_timer_set_stopped, nvim_timer_get_paused:
// deleted -- replaced by nvim_timer_read_fields / nvim_timer_write_fields (Phase 13).

// NvimTimerFields typedef is in eval.h (Phase 13).
// Verify field offsets match expected layout.
_Static_assert(offsetof(NvimTimerFields, timer_id) == 0, "NvimTimerFields.timer_id offset");
_Static_assert(offsetof(NvimTimerFields, repeat_count) == 4, "NvimTimerFields.repeat_count offset");
_Static_assert(offsetof(NvimTimerFields, refcount) == 8, "NvimTimerFields.refcount offset");
_Static_assert(offsetof(NvimTimerFields, emsg_count) == 12, "NvimTimerFields.emsg_count offset");
_Static_assert(offsetof(NvimTimerFields, timeout) == 16, "NvimTimerFields.timeout offset");
_Static_assert(offsetof(NvimTimerFields, stopped) == 24, "NvimTimerFields.stopped offset");
_Static_assert(offsetof(NvimTimerFields, paused) == 25, "NvimTimerFields.paused offset");

/// Bulk-read all scalar timer fields into a NvimTimerFields struct.
void nvim_timer_read_fields(const timer_T *timer, NvimTimerFields *out)
{
  out->timer_id = timer->timer_id;
  out->repeat_count = timer->repeat_count;
  out->refcount = timer->refcount;
  out->emsg_count = timer->emsg_count;
  out->timeout = timer->timeout;
  out->stopped = timer->stopped;
  out->paused = timer->paused;
}

/// Bulk-write all scalar timer fields from a NvimTimerFields struct.
void nvim_timer_write_fields(timer_T *timer, const NvimTimerFields *fields)
{
  timer->timer_id = fields->timer_id;
  timer->repeat_count = fields->repeat_count;
  timer->refcount = fields->refcount;
  timer->emsg_count = fields->emsg_count;
  timer->timeout = fields->timeout;
  timer->stopped = fields->stopped;
  timer->paused = fields->paused;
}

/// Get pointer to the callback field.
Callback *nvim_timer_get_callback_ptr(timer_T *timer)
{
  return &timer->callback;
}

/// Copy callback into timer (sets timer->callback = *cb).
void nvim_timer_set_callback(timer_T *timer, const Callback *cb)
{
  timer->callback = *cb;
}

/// Initialize the TimeWatcher embedded in timer (calls time_watcher_init).
void nvim_timer_tw_init(timer_T *timer)
{
  time_watcher_init(&main_loop, &timer->tw, timer);
}

/// Start the TimeWatcher (calls time_watcher_start with timer_due_cb).
void nvim_timer_tw_start(timer_T *timer, uint64_t timeout, uint64_t repeat)
{
  time_watcher_start(&timer->tw, timer_due_cb, timeout, repeat);
}

/// Stop the TimeWatcher (calls time_watcher_stop).
void nvim_timer_tw_stop(timer_T *timer)
{
  time_watcher_stop(&timer->tw);
}

/// Close the TimeWatcher (calls time_watcher_close with rs_timer_close_cb).
void nvim_timer_tw_close(timer_T *timer)
{
  time_watcher_close(&timer->tw, rs_timer_close_cb);
}

/// Set timer->tw.events to a new child queue of main_loop events.
void nvim_timer_tw_set_events_child(timer_T *timer)
{
  timer->tw.events = multiqueue_new_child(loop_get_events(&main_loop));
}

/// Set timer->tw.blockable.
void nvim_timer_tw_set_blockable(timer_T *timer, int blockable)
{
  timer->tw.blockable = blockable != 0;
}

/// Free the timer's tw.events multiqueue.
void nvim_timer_tw_free_events(timer_T *timer)
{
  multiqueue_free(timer->tw.events);
}

/// Get a timer_T from the timers PMap by numeric ID.
timer_T *nvim_timers_get(int64_t id)
{
  return pmap_get(uint64_t)(&timers, (uint64_t)id);
}

/// Insert a timer_T into the timers PMap.
void nvim_timers_put(timer_T *timer)
{
  pmap_put(uint64_t)(&timers, (uint64_t)timer->timer_id, timer);
}

/// Remove a timer_T from the timers PMap.
void nvim_timers_del(int64_t id)
{
  pmap_del(uint64_t)(&timers, (uint64_t)id, NULL);
}

/// Return the current map size (number of timers).
size_t nvim_timers_size(void)
{
  return map_size(&timers);
}

/// Get and increment last_timer_id, returning the OLD value.
uint64_t nvim_timers_next_id(void)
{
  return last_timer_id++;
}

/// Iterate all timers, calling cb(timer, userdata) for each.
/// Used from Rust to implement add_timer_info_all and timer_stop_all.
void nvim_timers_foreach(void (*cb)(timer_T *, void *), void *userdata)
{
  timer_T *timer;
  map_foreach_value(&timers, timer, {
    cb(timer, userdata);
  })
}

/// Alloc a dictitem_T with the given key.
dictitem_T *nvim_tv_dict_item_alloc_key(const char *key)
{
  return tv_dict_item_alloc(key);
}

/// Add dictitem to dict. Returns OK (0) or FAIL (-1 in C: but we expose as bool).
int nvim_tv_dict_add_item(dict_T *dict, dictitem_T *di)
{
  return tv_dict_add(dict, di);
}

/// Wrapper for get_pressedreturn() -- returns 1 if true.
int nvim_get_pressedreturn(void)
{
  return get_pressedreturn() ? 1 : 0;
}

/// Wrapper for set_pressedreturn().
void nvim_set_pressedreturn(int val)
{
  set_pressedreturn(val != 0);
}

/// Wrapper for discard_current_exception().
void nvim_discard_current_exception(void)
{
  discard_current_exception();
}

// =============================================================================
// Job helper accessors for Rust Phase 2 (eval_shim pass 8)
// =============================================================================

/// Set the buffered field of a CallbackReader.
void nvim_cbr_set_buffered(CallbackReader *reader, int buffered)
{
  reader->buffered = buffered != 0;
}

/// Set the self field of a CallbackReader.
void nvim_cbr_set_self(CallbackReader *reader, dict_T *self)
{
  reader->self = self;
}

/// Wrapper for callback_reader_free.
void nvim_callback_reader_free(CallbackReader *reader)
{
  callback_reader_free(reader);
}

// nvim_dict_refcount_inc already defined above.

/// Wrapper for tv_dict_get_callback.
bool nvim_tv_dict_get_callback(dict_T *dict, const char *key, ptrdiff_t key_len,
                                Callback *result)
{
  return tv_dict_get_callback(dict, key, key_len, result);
}

/// Wrapper for tv_dict_get_number.
int64_t nvim_tv_dict_get_number(const dict_T *dict, const char *key)
{
  return tv_dict_get_number(dict, key);
}

/// Check if a Channel is a proc stream and not stopped.
/// Returns 1 if it is a valid job channel, 0 otherwise.
int nvim_channel_is_valid_job(Channel *chan)
{
  if (chan == NULL) {
    return 0;
  }
  if (chan->streamtype != kChannelStreamProc) {
    return 0;
  }
  if (proc_is_stopped(&chan->stream.proc)) {
    return 0;
  }
  return 1;
}

/// Check if a Channel is NOT a proc stream (for find_job error reporting).
int nvim_channel_is_not_proc(Channel *chan)
{
  return (chan != NULL && chan->streamtype != kChannelStreamProc) ? 1 : 0;
}

/// Find channel by ID (accessor for Rust FFI - used by eval/src/job.rs).
Channel *nvim_find_channel(uint64_t id)
{
  return find_channel(id);
}

// =============================================================================
// Exception accessors for handle_did_throw_impl (Rust Phase N+9)
// =============================================================================

/// Returns current_exception->type as int (ET_USER=0, ET_ERROR=1, ET_INTERRUPT=2).
int nvim_exception_get_type(void) { return (int)current_exception->type; }
/// Returns current_exception->value.
char *nvim_exception_get_value(void) { return current_exception->value; }
/// Takes current_exception->messages (sets it to NULL) and returns it.
void *nvim_exception_take_messages(void)
{
  void *m = current_exception->messages;
  current_exception->messages = NULL;
  return m;
}
/// Returns current_exception->throw_name.
char *nvim_exception_get_throw_name(void) { return current_exception->throw_name; }
/// Returns current_exception->throw_lnum.
int nvim_exception_get_throw_lnum(void) { return (int)current_exception->throw_lnum; }
/// Sets current_exception->throw_name to NULL.
void nvim_exception_set_throw_name_null(void) { current_exception->throw_name = NULL; }
/// Returns msglist->next.
void *nvim_msglist_get_next(void *m) { return ((msglist_T *)m)->next; }
/// Returns msglist->msg.
char *nvim_msglist_get_msg(void *m) { return ((msglist_T *)m)->msg; }
/// Returns msglist->multiline.
int nvim_msglist_is_multiline(void *m) { return ((msglist_T *)m)->multiline ? 1 : 0; }
/// Frees msg, sfile, and the item itself.
void nvim_msglist_free_item(void *m)
{
  msglist_T *item = (msglist_T *)m;
  xfree(item->msg);
  xfree(item->sfile);
  xfree(item);
}
/// Sets suppress_errthrow = val.
void nvim_set_suppress_errthrow(bool val) { suppress_errthrow = val; }
/// Sets force_abort = val.
void nvim_set_force_abort(bool val) { force_abort = val; }
/// Format "E605: Exception not caught: %s" with value into allocated string.
char *nvim_docmd_fmt_exception_not_caught(const char *value)
{
  vim_snprintf(IObuff, IOSIZE, _("E605: Exception not caught: %s"), value);
  return xstrdup(IObuff);
}
