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

// Rust FFI declarations
extern int64_t rs_num_divide(int64_t n1, int64_t n2);
extern int64_t rs_num_modulus(int64_t n1, int64_t n2);
extern bool rs_eval_isdictc(int c);
extern const char *rs_skip_luafunc_name(const char *p);
extern int rs_check_luafunc_name(const char *str, bool paren);
extern bool rs_is_luafunc(partial_T *partial);
extern char *rs_partial_name(partial_T *pt);
extern int rs_get_copyID(void);
extern bool rs_set_ref_in_tagfunc(int copyID);
extern size_t rs_string2float(const char *text, float_T *ret_value);
extern char *rs_char_from_string(const char *str, varnumber_T index);
extern char *rs_string_slice(const char *str, varnumber_T first, varnumber_T last, bool exclusive);
extern int rs_get_env_len(const char **arg);
extern int rs_get_id_len(const char **arg);
extern const char *rs_to_name_end(const char *arg, bool use_namespace);
extern const char *rs_find_name_end(const char *arg, const char **expr_start,
                                    const char **expr_end, int flags);
extern int rs_buf_byteidx_to_charidx(buf_T *buf, linenr_T lnum, int byteidx);
extern int rs_buf_charidx_to_byteidx(buf_T *buf, linenr_T lnum, int charidx);
extern int rs_pattern_match(const char *pat, const char *text, bool ic);
extern int rs_is_tty_option(const char *name);
extern int rs_get_callback_depth(void);
extern bool rs_set_ref_in_item(typval_T *tv, int copyID, ht_stack_T **ht_stack,
                               list_stack_T **list_stack);
extern bool rs_set_ref_in_callback(Callback *callback, int copyID, ht_stack_T **ht_stack,
                                   list_stack_T **list_stack);
extern MultiQueue *rs_loop_get_events(Loop *loop);
extern bool rs_set_ref_in_callback_reader(CallbackReader *reader, int copyID,
                                          ht_stack_T **ht_stack, list_stack_T **list_stack);
extern int rs_eval0(char *arg, typval_T *rettv, exarg_T *eap, void *evalarg);
extern int rs_eval1(char **arg, typval_T *rettv, void *evalarg);
extern int rs_eval_multdiv_number(typval_T *tv1, typval_T *tv2, int op);
extern int rs_eval_func(char **arg, evalarg_T *evalarg, char *name, int name_len,
                        typval_T *rettv, int flags, typval_T *basetv);
extern char *rs_get_lval(char *name, typval_T *rettv, lval_T *lp, bool unlet, bool skip,
                         int flags, int fne_flags);
extern void rs_clear_lval(lval_T *lp);
extern void rs_set_var_lval(lval_T *lp, char *endp, typval_T *rettv, bool copy,
                            bool is_const, const char *op);
extern int rs_eval_number(char **arg, typval_T *rettv, bool evaluate, bool want_string);
extern int rs_eval_list(char **arg, typval_T *rettv, void *evalarg);
extern int rs_eval_index(char **arg, typval_T *rettv, evalarg_T *evalarg, bool verbose);
extern int rs_check_can_index(typval_T *rettv, bool evaluate, bool verbose);
extern int rs_eval_index_inner(typval_T *rettv, bool is_range, typval_T *var1, typval_T *var2,
                               bool exclusive, const char *key, ptrdiff_t keylen, bool verbose);
extern void rs_f_slice(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern int rs_eval_method(char **arg, typval_T *rettv, evalarg_T *evalarg, bool verbose);
extern int rs_eval_lit_string(char **arg, typval_T *rettv, bool evaluate, bool interpolate);
extern int rs_eval_string(char **arg, typval_T *rettv, bool evaluate, bool interpolate);
extern int rs_eval_dict(char **arg, typval_T *rettv, evalarg_T *evalarg, bool literal);
extern int rs_eval_lit_dict(char **arg, typval_T *rettv, evalarg_T *evalarg);
extern int rs_eval6(char **arg, typval_T *rettv, evalarg_T *evalarg, bool want_string);
extern int rs_eval7(char **arg, typval_T *rettv, evalarg_T *evalarg, bool want_string);
extern int rs_eval_interp_string(char **arg, typval_T *rettv, bool evaluate);
extern void *rs_eval_for_line(const char *arg, bool *errp, exarg_T *eap, evalarg_T *evalarg);
extern bool rs_next_for_item(void *fi_void, char *arg);
extern void rs_free_for_info(void *fi_void);
extern bool rs_callback_call(const void *callback, int argcount, void *argvars, void *rettv);
extern int rs_free_unref_items(int copyID);

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

// C accessors for typval fields (used by Rust callback module)
int nvim_eval_tv_get_type(const typval_T *tv)
{
  return (int)tv->v_type;
}

char *nvim_eval_tv_get_vstring(const typval_T *tv)
{
  return tv->vval.v_string;
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

// C accessors for Callback struct setters
void nvim_eval_cb_set_partial(Callback *cb, partial_T *pt)
{
  cb->data.partial = pt;
  cb->type = kCallbackPartial;
}

void nvim_eval_cb_set_funcref(Callback *cb, char *name)
{
  cb->data.funcref = name;
  cb->type = kCallbackFuncref;
}

void nvim_eval_cb_set_none(Callback *cb)
{
  cb->data.funcref = NULL;
  cb->type = kCallbackNone;
}

// Error message helper
void nvim_eval_emsg_e921(void)
{
  emsg(_("E921: Invalid callback argument"));
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

// Callback struct accessors for GC
int nvim_eval_cb_get_type(const Callback *cb)
{
  return (int)cb->type;
}

partial_T *nvim_eval_cb_get_partial(const Callback *cb)
{
  return cb->data.partial;
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

// Construct a typval_T with VAR_DICT and call rs_set_ref_in_item
bool nvim_eval_set_ref_dict_tv(dict_T *dict, int copyID, ht_stack_T **ht_stack,
                               list_stack_T **list_stack)
{
  typval_T dtv;
  dtv.v_type = VAR_DICT;
  dtv.vval.v_dict = dict;
  return rs_set_ref_in_item(&dtv, copyID, ht_stack, list_stack);
}

// Construct a typval_T with VAR_PARTIAL and call rs_set_ref_in_item
bool nvim_eval_set_ref_partial_tv(partial_T *partial, int copyID, ht_stack_T **ht_stack,
                                  list_stack_T **list_stack)
{
  typval_T tv;
  tv.v_type = VAR_PARTIAL;
  tv.vval.v_partial = partial;
  return rs_set_ref_in_item(&tv, copyID, ht_stack, list_stack);
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

#define loop_get_events(l) rs_loop_get_events(l)

// TODO(ZyX-I): Remove DICT_MAXNEST, make users be non-recursive instead

#define DICT_MAXNEST 100        // maximum nesting of lists and dicts

static const char *e_missbrac = N_("E111: Missing ']'");
static const char *e_list_end = N_("E697: Missing end of List ']': %s");
static const char e_cannot_slice_dictionary[]
  = N_("E719: Cannot slice a Dictionary");
static const char e_cannot_index_special_variable[]
  = N_("E909: Cannot index a special variable");
static const char *e_nowhitespace
  = N_("E274: No white space allowed before parenthesis");
static const char e_cannot_index_a_funcref[]
  = N_("E695: Cannot index a Funcref");
static const char e_variable_nested_too_deep_for_making_copy[]
  = N_("E698: Variable nested too deep for making a copy");
static const char e_string_list_or_blob_required[]
  = N_("E1098: String, List or Blob required");
static const char e_expression_too_recursive_str[]
  = N_("E1169: Expression too recursive: %s");
static const char e_dot_can_only_be_used_on_dictionary_str[]
  = N_("E1203: Dot can only be used on a dictionary: %s");
static const char e_empty_function_name[]
  = N_("E1192: Empty function name");
static const char e_cannot_use_partial_here[]
  = N_("E1265: Cannot use a partial here");

static char * const namespace_char = "abglstvw";

/// Used for checking if local variables or arguments used in a lambda.
bool *eval_lavars_used = NULL;

static int echo_hl_id = 0;   // highlight id used for ":echo"

/// Info used by a ":for" loop.
typedef struct {
  int fi_semicolon;             // true if ending in '; var]'
  int fi_varcount;              // nr of variables in the list
  listwatch_T fi_lw;            // keep an eye on the item used.
  list_T *fi_list;              // list being used
  int fi_bi;                    // index of blob
  blob_T *fi_blob;              // blob being used
  char *fi_string;            // copy of string being used
  int fi_byte_idx;              // byte index in fi_string
} forinfo_T;

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

/// Initialize the global and v: variables.
void eval_init(void)
{
  evalvars_init();
  func_init();
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

void fill_evalarg_from_eap(evalarg_T *evalarg, exarg_T *eap, bool skip)
{
  *evalarg = (evalarg_T){ .eval_flags = skip ? 0 : EVAL_EVALUATE };

  if (eap == NULL) {
    return;
  }

  if (sourcing_a_script(eap)) {
    evalarg->eval_getline = eap->ea_getline;
    evalarg->eval_cookie = eap->cookie;
  }
}

/// Top level evaluation function, returning a boolean.
/// Sets "error" to true if there was an error.
///
/// @param skip  only parse, don't execute
///
/// @return  true or false.
bool eval_to_bool(char *arg, bool *error, exarg_T *eap, const bool skip,
                  const bool use_simple_function)
{
  typval_T tv;
  bool retval = false;
  evalarg_T evalarg;

  fill_evalarg_from_eap(&evalarg, eap, skip);

  if (skip) {
    emsg_skip++;
  }
  int r = use_simple_function ? eval0_simple_funccal(arg, &tv, eap, &evalarg)
                              : eval0(arg, &tv, eap, &evalarg);
  if (r == FAIL) {
    *error = true;
  } else {
    *error = false;
    if (!skip) {
      retval = (tv_get_number_chk(&tv, error) != 0);
      tv_clear(&tv);
    }
  }
  if (skip) {
    emsg_skip--;
  }
  clear_evalarg(&evalarg, eap);

  return retval;
}

/// Call eval1() and give an error message if not done at a lower level.
static int eval1_emsg(char **arg, typval_T *rettv, exarg_T *eap)
  FUNC_ATTR_NONNULL_ARG(1, 2)
{
  const char *const start = *arg;
  const int did_emsg_before = did_emsg;
  const int called_emsg_before = called_emsg;
  evalarg_T evalarg;

  fill_evalarg_from_eap(&evalarg, eap, eap != NULL && eap->skip);

  const int ret = eval1(arg, rettv, &evalarg);
  if (ret == FAIL) {
    // Report the invalid expression unless the expression evaluation has
    // been cancelled due to an aborting error, an interrupt, or an
    // exception, or we already gave a more specific error.
    // Also check called_emsg for when using assert_fails().
    if (!aborting()
        && did_emsg == did_emsg_before
        && called_emsg == called_emsg_before) {
      semsg(_(e_invexpr2), start);
    }
  }
  clear_evalarg(&evalarg, eap);
  return ret;
}

/// Evaluate a partial.
/// Pass arguments "argv[argc]".
/// Return the result in "rettv" and OK or FAIL.
static int eval_expr_partial(const typval_T *expr, typval_T *argv, int argc, typval_T *rettv)
  FUNC_ATTR_NONNULL_ALL
{
  partial_T *const partial = expr->vval.v_partial;
  if (partial == NULL) {
    return FAIL;
  }

  const char *const s = rs_partial_name(partial);
  if (s == NULL || *s == NUL) {
    return FAIL;
  }

  funcexe_T funcexe = FUNCEXE_INIT;
  funcexe.fe_evaluate = true;
  funcexe.fe_partial = partial;
  if (call_func(s, -1, rettv, argc, argv, &funcexe) == FAIL) {
    return FAIL;
  }

  return OK;
}

/// Evaluate an expression which is a function.
/// Pass arguments "argv[argc]".
/// Return the result in "rettv" and OK or FAIL.
static int eval_expr_func(const typval_T *expr, typval_T *argv, int argc, typval_T *rettv)
  FUNC_ATTR_NONNULL_ALL
{
  char buf[NUMBUFLEN];
  const char *const s = (expr->v_type == VAR_FUNC
                         ? expr->vval.v_string
                         : tv_get_string_buf_chk(expr, buf));
  if (s == NULL || *s == NUL) {
    return FAIL;
  }

  funcexe_T funcexe = FUNCEXE_INIT;
  funcexe.fe_evaluate = true;
  if (call_func(s, -1, rettv, argc, argv, &funcexe) == FAIL) {
    return FAIL;
  }

  return OK;
}

/// Evaluate an expression, which is a string.
/// Return the result in "rettv" and OK or FAIL.
static int eval_expr_string(const typval_T *expr, typval_T *rettv)
  FUNC_ATTR_NONNULL_ALL
{
  char buf[NUMBUFLEN];
  char *s = (char *)tv_get_string_buf_chk(expr, buf);
  if (s == NULL) {
    return FAIL;
  }

  s = skipwhite(s);
  if (eval1_emsg(&s, rettv, NULL) == FAIL) {
    return FAIL;
  }

  if (*skipwhite(s) != NUL) {  // check for trailing chars after expr
    tv_clear(rettv);
    semsg(_(e_invexpr2), s);
    return FAIL;
  }

  return OK;
}

/// Evaluate an expression, which can be a function, partial or string.
/// Pass arguments "argv[argc]".
/// Return the result in "rettv" and OK or FAIL.
///
/// @param want_func  if true, treat a string as a function name, not an expression
int eval_expr_typval(const typval_T *expr, bool want_func, typval_T *argv, int argc,
                     typval_T *rettv)
  FUNC_ATTR_NONNULL_ALL
{
  if (expr->v_type == VAR_PARTIAL) {
    return eval_expr_partial(expr, argv, argc, rettv);
  }
  if (expr->v_type == VAR_FUNC || want_func) {
    return eval_expr_func(expr, argv, argc, rettv);
  }

  return eval_expr_string(expr, rettv);
}

/// Like eval_to_bool() but using a typval_T instead of a string.
/// Works for string, funcref and partial.
bool eval_expr_to_bool(const typval_T *expr, bool *error)
  FUNC_ATTR_NONNULL_ARG(1, 2)
{
  typval_T argv, rettv;

  if (eval_expr_typval(expr, false, &argv, 0, &rettv) == FAIL) {
    *error = true;
    return false;
  }
  const bool res = (tv_get_number_chk(&rettv, error) != 0);
  tv_clear(&rettv);
  return res;
}

/// Top level evaluation function, returning a string
///
/// @param[in]  arg  String to evaluate.
/// @param[in]  skip  If true, only do parsing to nextcmd without reporting
///                   errors or actually evaluating anything.
///
/// @return [allocated] string result of evaluation or NULL in case of error or
///                     when skipping.
char *eval_to_string_skip(char *arg, exarg_T *eap, const bool skip)
  FUNC_ATTR_MALLOC FUNC_ATTR_NONNULL_ARG(1) FUNC_ATTR_WARN_UNUSED_RESULT
{
  typval_T tv;
  char *retval;
  evalarg_T evalarg;

  fill_evalarg_from_eap(&evalarg, eap, skip);
  if (skip) {
    emsg_skip++;
  }
  if (eval0(arg, &tv, eap, &evalarg) == FAIL || skip) {
    retval = NULL;
  } else {
    retval = xstrdup(tv_get_string(&tv));
    tv_clear(&tv);
  }
  if (skip) {
    emsg_skip--;
  }
  clear_evalarg(&evalarg, eap);

  return retval;
}

/// Skip over an expression at "*pp".
///
/// @return  FAIL for an error, OK otherwise.
int skip_expr(char **pp, evalarg_T *const evalarg)
{
  const int save_flags = evalarg == NULL ? 0 : evalarg->eval_flags;

  // Don't evaluate the expression.
  if (evalarg != NULL) {
    evalarg->eval_flags &= ~EVAL_EVALUATE;
  }

  *pp = skipwhite(*pp);
  typval_T rettv;
  int res = eval1(pp, &rettv, NULL);

  if (evalarg != NULL) {
    evalarg->eval_flags = save_flags;
  }

  return res;
}

/// Convert "tv" to a string.
///
/// @param join_list  when true convert a List into a sequence of lines.
///
/// @return  an allocated string.
static char *typval2string(typval_T *tv, bool join_list)
{
  if (join_list && tv->v_type == VAR_LIST) {
    garray_T ga;
    ga_init(&ga, (int)sizeof(char), 80);
    if (tv->vval.v_list != NULL) {
      tv_list_join(&ga, tv->vval.v_list, "\n");
      if (tv_list_len(tv->vval.v_list) > 0) {
        ga_append(&ga, NL);
      }
    }
    ga_append(&ga, NUL);
    return (char *)ga.ga_data;
  } else if (tv->v_type == VAR_LIST || tv->v_type == VAR_DICT) {
    return encode_tv2string(tv, NULL);
  }
  return xstrdup(tv_get_string(tv));
}

/// Top level evaluation function, returning a string.
///
/// @param join_list  when true convert a List into a sequence of lines.
///
/// @return  pointer to allocated memory, or NULL for failure.
char *eval_to_string_eap(char *arg, const bool join_list, exarg_T *eap,
                         const bool use_simple_function)
{
  typval_T tv;
  char *retval;

  evalarg_T evalarg;
  fill_evalarg_from_eap(&evalarg, eap, eap != NULL && eap->skip);
  int r = use_simple_function ? eval0_simple_funccal(arg, &tv, NULL, &evalarg)
                              : eval0(arg, &tv, NULL, &evalarg);
  if (r == FAIL) {
    retval = NULL;
  } else {
    retval = typval2string(&tv, join_list);
    tv_clear(&tv);
  }
  clear_evalarg(&evalarg, NULL);

  return retval;
}

char *eval_to_string(char *arg, const bool join_list, const bool use_simple_function)
{
  return eval_to_string_eap(arg, join_list, NULL, use_simple_function);
}

/// Call eval_to_string() without using current local variables and using
/// textlock.
///
/// @param use_sandbox  when true, use the sandbox.
char *eval_to_string_safe(char *arg, const bool use_sandbox, const bool use_simple_function)
{
  char *retval;
  funccal_entry_T funccal_entry;

  save_funccal(&funccal_entry);
  if (use_sandbox) {
    sandbox++;
  }
  textlock++;
  retval = eval_to_string(arg, false, use_simple_function);
  if (use_sandbox) {
    sandbox--;
  }
  textlock--;
  restore_funccal();
  return retval;
}

/// Top level evaluation function, returning a number.
/// Evaluates "expr" silently.
///
/// @return  -1 for an error.
varnumber_T eval_to_number(char *expr, const bool use_simple_function)
{
  typval_T rettv;
  varnumber_T retval;
  char *p = skipwhite(expr);
  int r = NOTDONE;

  emsg_off++;

  if (use_simple_function) {
    r = may_call_simple_func(expr, &rettv);
  }
  if (r == NOTDONE) {
    r = eval1(&p, &rettv, &EVALARG_EVALUATE);
  }
  if (r == FAIL) {
    retval = -1;
  } else {
    retval = tv_get_number_chk(&rettv, NULL);
    tv_clear(&rettv);
  }
  emsg_off--;

  return retval;
}

/// Top level evaluation function.
///
/// @return  an allocated typval_T with the result or
///          NULL when there is an error.
typval_T *eval_expr(char *arg, exarg_T *eap)
{
  return eval_expr_ext(arg, eap, false);
}

typval_T *eval_expr_ext(char *arg, exarg_T *eap, const bool use_simple_function)
{
  typval_T *tv = xmalloc(sizeof(*tv));
  evalarg_T evalarg;

  fill_evalarg_from_eap(&evalarg, eap, eap != NULL && eap->skip);

  int r = NOTDONE;

  if (use_simple_function) {
    r = eval0_simple_funccal(arg, tv, eap, &evalarg);
  }
  if (r == NOTDONE) {
    r = eval0(arg, tv, eap, &evalarg);
  }

  if (r == FAIL) {
    XFREE_CLEAR(tv);
  }

  clear_evalarg(&evalarg, eap);
  return tv;
}

/// Call some Vim script function and return the result in "*rettv".
/// Uses argv[0] to argv[argc - 1] for the function arguments. argv[argc]
/// should have type VAR_UNKNOWN.
///
/// @return  OK or FAIL.
int call_vim_function(const char *func, int argc, typval_T *argv, typval_T *rettv)
  FUNC_ATTR_NONNULL_ALL
{
  int ret;
  int len = (int)strlen(func);
  partial_T *pt = NULL;

  if (len >= 6 && !memcmp(func, "v:lua.", 6)) {
    func += 6;
    len = rs_check_luafunc_name(func, false);
    if (len == 0) {
      ret = FAIL;
      goto fail;
    }
    pt = get_vim_var_partial(VV_LUA);
  }

  rettv->v_type = VAR_UNKNOWN;  // tv_clear() uses this.
  funcexe_T funcexe = FUNCEXE_INIT;
  funcexe.fe_firstline = curwin->w_cursor.lnum;
  funcexe.fe_lastline = curwin->w_cursor.lnum;
  funcexe.fe_evaluate = true;
  funcexe.fe_partial = pt;
  ret = call_func(func, len, rettv, argc, argv, &funcexe);

fail:
  if (ret == FAIL) {
    tv_clear(rettv);
  }

  return ret;
}

/// Call Vim script function and return the result as a string.
/// Uses "argv[0]" to "argv[argc - 1]" for the function arguments. "argv[argc]"
/// should have type VAR_UNKNOWN.
///
/// @param[in]  func  Function name.
/// @param[in]  argc  Number of arguments.
/// @param[in]  argv  Array with typval_T arguments.
///
/// @return [allocated] NULL when calling function fails, allocated string
///                     otherwise.
void *call_func_retstr(const char *const func, int argc, typval_T *argv)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_MALLOC
{
  typval_T rettv;
  // All arguments are passed as strings, no conversion to number.
  if (call_vim_function(func, argc, argv, &rettv)
      == FAIL) {
    return NULL;
  }

  char *const retval = xstrdup(tv_get_string(&rettv));
  tv_clear(&rettv);
  return retval;
}

/// Call Vim script function and return the result as a List.
/// Uses "argv" and "argc" as call_func_retstr().
///
/// @param[in]  func  Function name.
/// @param[in]  argc  Number of arguments.
/// @param[in]  argv  Array with typval_T arguments.
///
/// @return [allocated] NULL when calling function fails or return tv is not a
///                     List, allocated List otherwise.
void *call_func_retlist(const char *func, int argc, typval_T *argv)
  FUNC_ATTR_NONNULL_ALL
{
  typval_T rettv;

  // All arguments are passed as strings, no conversion to number.
  if (call_vim_function(func, argc, argv, &rettv) == FAIL) {
    return NULL;
  }

  if (rettv.v_type != VAR_LIST) {
    tv_clear(&rettv);
    return NULL;
  }

  return rettv.vval.v_list;
}

/// Evaluate 'foldexpr'.  Returns the foldlevel, and any character preceding
/// it in "*cp".  Doesn't give error messages.
int eval_foldexpr(win_T *wp, int *cp)
{
  const sctx_T saved_sctx = current_sctx;
  const bool use_sandbox = was_set_insecurely(wp, kOptFoldexpr, OPT_LOCAL);

  char *arg = skipwhite(wp->w_p_fde);
  current_sctx = wp->w_p_script_ctx[kWinOptFoldexpr];

  emsg_off++;
  if (use_sandbox) {
    sandbox++;
  }
  textlock++;
  *cp = NUL;

  typval_T tv;
  varnumber_T retval;
  // Evaluate the expression.  If the expression is "FuncName()" call the
  // function directly.
  if (eval0_simple_funccal(arg, &tv, NULL, &EVALARG_EVALUATE) == FAIL) {
    retval = 0;
  } else {
    // If the result is a number, just return the number.
    if (tv.v_type == VAR_NUMBER) {
      retval = tv.vval.v_number;
    } else if (tv.v_type != VAR_STRING || tv.vval.v_string == NULL) {
      retval = 0;
    } else {
      // If the result is a string, check if there is a non-digit before
      // the number.
      char *s = tv.vval.v_string;
      if (*s != NUL && !ascii_isdigit(*s) && *s != '-') {
        *cp = (uint8_t)(*s++);
      }
      retval = atol(s);
    }
    tv_clear(&tv);
  }

  emsg_off--;
  if (use_sandbox) {
    sandbox--;
  }
  textlock--;
  clear_evalarg(&EVALARG_EVALUATE, NULL);
  current_sctx = saved_sctx;

  return (int)retval;
}

/// Evaluate 'foldtext', returning an Array or a String (NULL_STRING on failure).
Object eval_foldtext(win_T *wp)
{
  const bool use_sandbox = was_set_insecurely(wp, kOptFoldtext, OPT_LOCAL);
  char *arg = wp->w_p_fdt;
  funccal_entry_T funccal_entry;

  save_funccal(&funccal_entry);
  if (use_sandbox) {
    sandbox++;
  }
  textlock++;

  typval_T tv;
  Object retval;
  if (eval0_simple_funccal(arg, &tv, NULL, &EVALARG_EVALUATE) == FAIL) {
    retval = STRING_OBJ(NULL_STRING);
  } else {
    if (tv.v_type == VAR_LIST) {
      retval = vim_to_object(&tv, NULL, false);
    } else {
      retval = STRING_OBJ(cstr_to_string(tv_get_string(&tv)));
    }
    tv_clear(&tv);
  }
  clear_evalarg(&EVALARG_EVALUATE, NULL);

  if (use_sandbox) {
    sandbox--;
  }
  textlock--;
  restore_funccal();

  return retval;
}


/// Get an lvalue
///
/// Lvalue may be
/// - variable: "name", "na{me}"
/// - dictionary item: "dict.key", "dict['key']"
/// - list item: "list[expr]"
/// - list slice: "list[expr:expr]"
///
/// Indexing only works if trying to use it with an existing List or Dictionary.
///
/// @param[in]  name  Name to parse.
/// @param  rettv  Pointer to the value to be assigned or NULL.
/// @param[out]  lp  Lvalue definition. When evaluation errors occur `->ll_name`
///                  is NULL.
/// @param[in]  unlet  True if using `:unlet`. This results in slightly
///                    different behaviour when something is wrong; must end in
///                    space or cmd separator.
/// @param[in]  skip  True when skipping.
/// @param[in]  flags  @see GetLvalFlags.
/// @param[in]  fne_flags  Flags for find_name_end().
///
/// @return A pointer to just after the name, including indexes. Returns NULL
///         for a parsing error, but it is still needed to free items in lp.
char *get_lval(char *const name, typval_T *const rettv, lval_T *const lp, const bool unlet,
               const bool skip, const int flags, const int fne_flags)
  FUNC_ATTR_NONNULL_ARG(1, 3)
{
  return rs_get_lval(name, rettv, lp, unlet, skip, flags, fne_flags);
}

/// Clear lval "lp" that was filled by get_lval().
void clear_lval(lval_T *lp)
{
  rs_clear_lval(lp);
}

/// Set a variable that was parsed by get_lval() to "rettv".
///
/// @param endp  points to just after the parsed name.
/// @param op    NULL, "+" for "+=", "-" for "-=", "*" for "*=", "/" for "/=",
///              "%" for "%=", "." for ".=" or "=" for "=".
void set_var_lval(lval_T *lp, char *endp, typval_T *rettv, bool copy, const bool is_const,
                  const char *op)
{
  rs_set_var_lval(lp, endp, rettv, copy, is_const, op);
}

/// Evaluate the expression used in a ":for var in expr" command.
/// "arg" points to "var".
///
/// @param[out] *errp  set to true for an error, false otherwise;
///
/// @return  a pointer that holds the info.  Null when there is an error.
void *eval_for_line(const char *arg, bool *errp, exarg_T *eap, evalarg_T *const evalarg)
{
  return rs_eval_for_line(arg, errp, eap, evalarg);
}

/// Use the first item in a ":for" list.  Advance to the next.
/// Assign the values to the variable (list).  "arg" points to the first one.
///
/// @return  true when a valid item was found, false when at end of list or
///          something wrong.
bool next_for_item(void *fi_void, char *arg)
{
  return rs_next_for_item(fi_void, arg);
}

/// Free the structure used to store info used by ":for".
void free_for_info(void *fi_void)
{
  rs_free_for_info(fi_void);
}

// =============================================================================
// Accessors for rs_eval_for_line / rs_next_for_item / rs_free_for_info (Phase 3)
// All use void* to avoid exposing the local forinfo_T typedef in generated headers.
// =============================================================================

/// Allocate a zeroed forinfo_T and return as opaque pointer.
void *nvim_forinfo_alloc(void)
{
  return xcalloc(1, sizeof(forinfo_T));
}

/// Free a forinfo_T struct (does NOT free list/blob/string refs).
void nvim_forinfo_free(void *fi_void)
{
  xfree(fi_void);
}

/// Get fi->fi_varcount.
int nvim_forinfo_get_varcount(void *fi_void)
{
  return ((forinfo_T *)fi_void)->fi_varcount;
}

/// Set fi->fi_varcount.
void nvim_forinfo_set_varcount(void *fi_void, int n)
{
  ((forinfo_T *)fi_void)->fi_varcount = n;
}

/// Get fi->fi_semicolon.
int nvim_forinfo_get_semicolon(void *fi_void)
{
  return ((forinfo_T *)fi_void)->fi_semicolon;
}

/// Set fi->fi_semicolon.
void nvim_forinfo_set_semicolon(void *fi_void, int v)
{
  ((forinfo_T *)fi_void)->fi_semicolon = v;
}

/// Return true if fi->fi_list != NULL.
bool nvim_forinfo_has_list(void *fi_void)
{
  return ((forinfo_T *)fi_void)->fi_list != NULL;
}

/// Return true if fi->fi_blob != NULL.
bool nvim_forinfo_has_blob(void *fi_void)
{
  return ((forinfo_T *)fi_void)->fi_blob != NULL;
}

/// Return true if fi->fi_string != NULL.
bool nvim_forinfo_has_string(void *fi_void)
{
  return ((forinfo_T *)fi_void)->fi_string != NULL;
}

/// Get fi->fi_bi.
int nvim_forinfo_get_bi(void *fi_void)
{
  return ((forinfo_T *)fi_void)->fi_bi;
}

/// Set fi->fi_bi.
void nvim_forinfo_set_bi(void *fi_void, int n)
{
  ((forinfo_T *)fi_void)->fi_bi = n;
}

/// Get fi->fi_byte_idx.
int nvim_forinfo_get_byte_idx(void *fi_void)
{
  return ((forinfo_T *)fi_void)->fi_byte_idx;
}

/// Set fi->fi_byte_idx.
void nvim_forinfo_set_byte_idx(void *fi_void, int n)
{
  ((forinfo_T *)fi_void)->fi_byte_idx = n;
}

/// Get fi->fi_string.
char *nvim_forinfo_get_string(void *fi_void)
{
  return ((forinfo_T *)fi_void)->fi_string;
}

/// Set fi->fi_string (takes ownership).
void nvim_forinfo_set_string(void *fi_void, char *s)
{
  ((forinfo_T *)fi_void)->fi_string = s;
}

/// Get fi->fi_list as void *.
void *nvim_forinfo_get_list(void *fi_void)
{
  return ((forinfo_T *)fi_void)->fi_list;
}

/// Set fi->fi_list.
void nvim_forinfo_set_list(void *fi_void, void *l)
{
  ((forinfo_T *)fi_void)->fi_list = (list_T *)l;
}

/// Get fi->fi_blob as void *.
void *nvim_forinfo_get_blob(void *fi_void)
{
  return ((forinfo_T *)fi_void)->fi_blob;
}

/// Set fi->fi_blob.
void nvim_forinfo_set_blob(void *fi_void, void *b)
{
  ((forinfo_T *)fi_void)->fi_blob = (blob_T *)b;
}

/// Get the lw_item (listitem_T *) from fi->fi_lw as void *.
void *nvim_forinfo_get_lw_item(void *fi_void)
{
  return ((forinfo_T *)fi_void)->fi_lw.lw_item;
}

/// Set fi->fi_lw.lw_item.
void nvim_forinfo_set_lw_item(void *fi_void, void *item)
{
  ((forinfo_T *)fi_void)->fi_lw.lw_item = (listitem_T *)item;
}

/// Call tv_list_watch_add(l, &fi->fi_lw).
void nvim_forinfo_list_watch_add(void *fi_void, void *l)
{
  forinfo_T *fi = (forinfo_T *)fi_void;
  tv_list_watch_add((list_T *)l, &fi->fi_lw);
}

/// Call tv_list_watch_remove(fi->fi_list, &fi->fi_lw).
void nvim_forinfo_list_watch_remove(void *fi_void)
{
  forinfo_T *fi = (forinfo_T *)fi_void;
  tv_list_watch_remove(fi->fi_list, &fi->fi_lw);
}

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

/// Call skip_var_list(arg, &varcount, &semicolon, nested).
const char *nvim_skip_var_list(const char *arg, int *varcount, int *semicolon, bool nested)
{
  return skip_var_list(arg, varcount, semicolon, nested);
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

/// Increment emsg_skip.
void nvim_emsg_skip_inc(void)
{
  emsg_skip++;
}

/// Decrement emsg_skip.
void nvim_emsg_skip_dec(void)
{
  emsg_skip--;
}

extern void rs_set_context_for_expression(expand_T *xp, char *arg, int cmdidx);
void set_context_for_expression(expand_T *xp, char *arg, cmdidx_T cmdidx)
  FUNC_ATTR_NONNULL_ALL
{
  rs_set_context_for_expression(xp, arg, (int)cmdidx);
}

/// Does not use 'cpo' and always uses 'magic'.
///
/// @return  true if "pat" matches "text".
int pattern_match(const char *pat, const char *text, bool ic)
{
  return rs_pattern_match(pat, text, ic);
}

/// Handle a name followed by "(".  Both for just "name(arg)" and for
/// "expr->name(arg)".
///
/// @param arg  Points to "(", will be advanced
/// @param basetv  "expr" for "expr->name(arg)"
///
/// @return OK or FAIL.
static int eval_func(char **const arg, evalarg_T *const evalarg, char *const name,
                     const int name_len, typval_T *const rettv, const int flags,
                     typval_T *const basetv)
  FUNC_ATTR_NONNULL_ARG(1, 3, 5)
{
  return rs_eval_func(arg, evalarg, name, name_len, rettv, flags, basetv);
}

/// After using "evalarg" filled from "eap": free the memory.
void clear_evalarg(evalarg_T *evalarg, exarg_T *eap)
{
  if (evalarg == NULL) {
    return;
  }

  if (evalarg->eval_tofree != NULL) {
    if (eap != NULL) {
      // We may need to keep the original command line, e.g. for
      // ":let" it has the variable names.  But we may also need the
      // new one, "nextcmd" points into it.  Keep both.
      xfree(eap->cmdline_tofree);
      eap->cmdline_tofree = *eap->cmdlinep;
      *eap->cmdlinep = evalarg->eval_tofree;
    } else {
      xfree(evalarg->eval_tofree);
    }
    evalarg->eval_tofree = NULL;
  }
}

/// The "eval" functions have an "evalarg" argument: When NULL or
/// "evalarg->eval_flags" does not have EVAL_EVALUATE, then the argument is only
/// parsed but not executed.  The functions may return OK, but the rettv will be
/// of type VAR_UNKNOWN.  The functions still returns FAIL for a syntax error.

/// Handle zero level expression.
/// This calls eval1() and handles error message and nextcmd.
/// Put the result in "rettv" when returning OK and "evaluate" is true.
///
/// @param evalarg  can be NULL, &EVALARG_EVALUATE or a pointer.
///
/// @return OK or FAIL.
int eval0(char *arg, typval_T *rettv, exarg_T *eap, evalarg_T *const evalarg)
{
  return rs_eval0(arg, rettv, eap, evalarg);
}

/// If "arg" is a simple function call without arguments then call it and return
/// the result.  Otherwise return NOTDONE.
int may_call_simple_func(const char *arg, typval_T *rettv)
{
  const char *parens = strstr(arg, "()");
  int r = NOTDONE;

  // If the expression is "FuncName()" then we can skip a lot of overhead.
  if (parens != NULL && *skipwhite(parens + 2) == NUL) {
    if (strnequal(arg, "v:lua.", 6)) {
      const char *p = arg + 6;
      if (p != parens && rs_skip_luafunc_name(p) == parens) {
        r = call_simple_luafunc(p, (size_t)(parens - p), rettv);
      }
    } else {
      const char *p = strncmp(arg, "<SNR>", 5) == 0 ? skipdigits(arg + 5) : arg;
      if (rs_to_name_end(p, true) == parens) {
        r = call_simple_func(arg, (size_t)(parens - arg), rettv);
      }
    }
  }
  return r;
}

/// Handle zero level expression with optimization for a simple function call.
/// Same arguments and return value as eval0().
static int eval0_simple_funccal(char *arg, typval_T *rettv, exarg_T *eap, evalarg_T *const evalarg)
{
  int r = may_call_simple_func(arg, rettv);

  if (r == NOTDONE) {
    r = eval0(arg, rettv, eap, evalarg);
  }
  return r;
}

/// Handle top level expression:
///      expr2 ? expr1 : expr1
///      expr2 ?? expr1
///
/// "arg" must point to the first non-white of the expression.
/// "arg" is advanced to the next non-white after the recognized expression.
///
/// @return  OK or FAIL.
int eval1(char **arg, typval_T *rettv, evalarg_T *const evalarg)
{
  return rs_eval1(arg, rettv, evalarg);
}

/// Handle fifth level expression:
///  - *  number multiplication
///  - /  number division
///  - %  number modulo
///
/// @param[in,out]  arg  Points to the first non-whitespace character of the
///                      expression.  Is advanced to the next non-whitespace
///                      character after the recognized expression.
/// @param[out]  rettv  Location where result is saved.
/// @param[in]  want_string  True if "." is string_concatenation, otherwise
///                          float
/// @return  OK or FAIL.
int eval6(char **arg, typval_T *rettv, evalarg_T *const evalarg, bool want_string)
{
  return rs_eval6(arg, rettv, evalarg, want_string);
}

// eval7 and eval7_leader migrated to Rust (rs_eval7 in eval_exec crate).

/// Call the function referred to in "rettv".
/// @param lua_funcname  If `rettv` refers to a v:lua function, this must point
///                      to the name of the Lua function to call (after the
///                      "v:lua." prefix).
/// @return  OK on success, FAIL on failure.
static int call_func_rettv(char **const arg, evalarg_T *const evalarg, typval_T *const rettv,
                           const bool evaluate, dict_T *const selfdict, typval_T *const basetv,
                           const char *const lua_funcname)
  FUNC_ATTR_NONNULL_ARG(1, 3)
{
  partial_T *pt = NULL;
  typval_T functv;
  const char *funcname;
  bool is_lua = false;
  int ret;

  // need to copy the funcref so that we can clear rettv
  if (evaluate) {
    functv = *rettv;
    rettv->v_type = VAR_UNKNOWN;

    // Invoke the function.  Recursive!
    if (functv.v_type == VAR_PARTIAL) {
      pt = functv.vval.v_partial;
      is_lua = rs_is_luafunc(pt);
      funcname = is_lua ? lua_funcname : rs_partial_name(pt);
    } else {
      funcname = functv.vval.v_string;
      if (funcname == NULL || *funcname == NUL) {
        emsg(_(e_empty_function_name));
        ret = FAIL;
        goto theend;
      }
    }
  } else {
    funcname = "";
  }

  funcexe_T funcexe = FUNCEXE_INIT;
  funcexe.fe_firstline = curwin->w_cursor.lnum;
  funcexe.fe_lastline = curwin->w_cursor.lnum;
  funcexe.fe_evaluate = evaluate;
  funcexe.fe_partial = pt;
  funcexe.fe_selfdict = selfdict;
  funcexe.fe_basetv = basetv;
  ret = get_func_tv(funcname, is_lua ? (int)(*arg - funcname) : -1, rettv,
                    arg, evalarg, &funcexe);

theend:
  // Clear the funcref afterwards, so that deleting it while
  // evaluating the arguments is possible (see test55).
  if (evaluate) {
    tv_clear(&functv);
  }

  return ret;
}

/// Evaluate "->method()".
///
/// @param verbose  if true, give error messages.
/// @param *arg     points to the '-'.
///
/// @return  FAIL or OK.
///
/// @note "*arg" is advanced to after the ')'.
static int eval_lambda(char **const arg, typval_T *const rettv, evalarg_T *const evalarg,
                       const bool verbose)
  FUNC_ATTR_NONNULL_ARG(1, 2)
{
  const bool evaluate = evalarg != NULL && (evalarg->eval_flags & EVAL_EVALUATE);
  // Skip over the ->.
  *arg += 2;
  typval_T base = *rettv;
  rettv->v_type = VAR_UNKNOWN;

  int ret = get_lambda_tv(arg, rettv, evalarg);
  if (ret != OK) {
    return FAIL;
  } else if (**arg != '(') {
    if (verbose) {
      if (*skipwhite(*arg) == '(') {
        emsg(_(e_nowhitespace));
      } else {
        semsg(_(e_missingparen), "lambda");
      }
    }
    tv_clear(rettv);
    ret = FAIL;
  } else {
    ret = call_func_rettv(arg, evalarg, rettv, evaluate, NULL, &base, NULL);
  }

  // Clear the funcref afterwards, so that deleting it while
  // evaluating the arguments is possible (see test55).
  if (evaluate) {
    tv_clear(&base);
  }

  return ret;
}

/// Evaluate "->method()" or "->v:lua.method()".
///
/// @param *arg  points to the '-'.
///
/// @return  FAIL or OK. "*arg" is advanced to after the ')'.
static int eval_method(char **const arg, typval_T *const rettv, evalarg_T *const evalarg,
                       const bool verbose)
  FUNC_ATTR_NONNULL_ARG(1, 2)
{
  return rs_eval_method(arg, rettv, evalarg, verbose);
}

/// Evaluate an "[expr]" or "[expr:expr]" index.  Also "dict.key".
/// "*arg" points to the '[' or '.'.
///
/// @param verbose  give error messages
///
/// @returns FAIL or OK. "*arg" is advanced to after the ']'.
static int eval_index(char **arg, typval_T *rettv, evalarg_T *const evalarg, bool verbose)
{
  return rs_eval_index(arg, rettv, evalarg, verbose);
}

/// Check if "rettv" can have an [index] or [sli:ce]
static int check_can_index(typval_T *rettv, bool evaluate, bool verbose)
{
  return rs_check_can_index(rettv, evaluate, verbose);
}

/// slice() function
void f_slice(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_slice(argvars, rettv, fptr);
}

/// Apply index or range to "rettv".
///
/// @param var1  the first index, NULL for [:expr].
/// @param var2  the second index, NULL for [expr] and [expr: ]
/// @param exclusive  true for slice(): second index is exclusive, use character
///                                     index for string.
/// Alternatively, "key" is not NULL, then key[keylen] is the dict index.
static int eval_index_inner(typval_T *rettv, bool is_range, typval_T *var1, typval_T *var2,
                            bool exclusive, const char *key, ptrdiff_t keylen, bool verbose)
{
  return rs_eval_index_inner(rettv, is_range, var1, var2, exclusive, key, keylen, verbose);
}

/// Get an option value
///
/// @param[in,out] arg  Points to the '&' or '+' before the option name. Is
///                      advanced to the character after the option name.
/// @param[out] rettv  Location where result is saved.
/// @param[in] evaluate  If not true, rettv is not populated.
///
/// @return  OK or FAIL.
int eval_option(const char **const arg, typval_T *const rettv, const bool evaluate)
  FUNC_ATTR_NONNULL_ARG(1)
{
  const bool working = (**arg == '+');  // has("+option")
  OptIndex opt_idx;
  int opt_flags;

  // Isolate the option name and find its value.
  char *const option_end = (char *)find_option_var_end(arg, &opt_idx, &opt_flags);

  if (option_end == NULL) {
    if (rettv != NULL) {
      semsg(_("E112: Option name missing: %s"), *arg);
    }
    return FAIL;
  }

  if (!evaluate) {
    *arg = option_end;
    return OK;
  }

  char c = *option_end;
  *option_end = NUL;

  int ret = OK;
  bool is_tty_opt = rs_is_tty_option(*arg);

  if (opt_idx == kOptInvalid && !is_tty_opt) {
    // Only give error if result is going to be used.
    if (rettv != NULL) {
      semsg(_("E113: Unknown option: %s"), *arg);
    }

    ret = FAIL;
  } else if (rettv != NULL) {
    OptVal value = is_tty_opt ? get_tty_option(*arg) : get_option_value(opt_idx, opt_flags);
    assert(value.type != kOptValTypeNil);

    *rettv = optval_as_tv(value, true);
  } else if (working && !is_tty_opt && is_option_hidden(opt_idx)) {
    ret = FAIL;
  }

  *option_end = c;                  // put back for error messages
  *arg = option_end;

  return ret;
}


/// Evaluate a single or double quoted string possibly containing expressions.
/// "arg" points to the '$'.  The result is put in "rettv".
///
/// @return  OK or FAIL.
int eval_interp_string(char **arg, typval_T *rettv, bool evaluate)
{
  return rs_eval_interp_string(arg, rettv, evaluate);
}

// =============================================================================
// Accessors for rs_eval_interp_string (Rust Phase 2)
// =============================================================================

/// Allocate and initialize a char garray with given growth size.
/// Returns an opaque pointer to be passed to other nvim_ga_* functions.
garray_T *nvim_ga_alloc_char(int growsize)
{
  garray_T *ga = xmalloc(sizeof(garray_T));
  ga_init(ga, 1, growsize);
  return ga;
}

/// Concatenate a C string into the garray.
void nvim_ga_concat_str(garray_T *ga, const char *s)
{
  if (s != NULL) {
    ga_concat(ga, s);
  }
}

/// Append a NUL byte to the garray.
void nvim_ga_append_nul(garray_T *ga)
{
  ga_append(ga, NUL);
}

/// Take the ga_data pointer and free the garray struct.
/// The caller owns the returned string.
char *nvim_ga_take_data(garray_T *ga)
{
  char *data = ga->ga_data;
  xfree(ga);
  return data;
}

/// Free the garray (including data) and the struct.
void nvim_ga_free(garray_T *ga)
{
  ga_clear(ga);
  xfree(ga);
}

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

/// Get partial_T->pt_name (accessor for Rust).
char *nvim_partial_get_pt_name(partial_T *pt)
{
  return pt->pt_name;
}

/// Get partial_T->pt_func->uf_name (accessor for Rust).
char *nvim_partial_get_pt_func_uf_name(partial_T *pt)
{
  if (pt->pt_func != NULL) {
    return pt->pt_func->uf_name;
  }
  return NULL;
}

static void partial_free(partial_T *pt)
{
  for (int i = 0; i < pt->pt_argc; i++) {
    tv_clear(&pt->pt_argv[i]);
  }
  xfree(pt->pt_argv);
  tv_dict_unref(pt->pt_dict);
  if (pt->pt_name != NULL) {
    func_unref(pt->pt_name);
    xfree(pt->pt_name);
  } else {
    func_ptr_unref(pt->pt_func);
  }
  xfree(pt);
}

/// Unreference a closure: decrement the reference count and free it when it
/// becomes zero.
void partial_unref(partial_T *pt)
{
  if (pt == NULL) {
    return;
  }

  if (--pt->pt_refcount <= 0) {
    partial_free(pt);
  }
}


/// Garbage collection for lists and dictionaries.
///
/// We use reference counts to be able to free most items right away when they
/// are no longer used.  But for composite items it's possible that it becomes
/// unused while the reference count is > 0: When there is a recursive
/// reference.  Example:
///      :let l = [1, 2, 3]
///      :let d = {9: l}
///      :let l[1] = d
///
/// Since this is quite unusual we handle this with garbage collection: every
/// once in a while find out which lists and dicts are not referenced from any
/// variable.
///
/// Here is a good reference text about garbage collection (refers to Python
/// but it applies to all reference-counting mechanisms):
///      http://python.ca/nas/python/gc/

/// Do garbage collection for lists and dicts.
///
/// @param testing  true if called from test_garbagecollect_now().
///
/// @return  true if some memory was freed.
bool garbage_collect(bool testing)
{
  bool abort = false;
#define ABORTING(func) abort = abort || func

  if (!testing) {
    // Only do this once.
    want_garbage_collect = false;
    may_garbage_collect = false;
    garbage_collect_at_exit = false;
  }

  // The execution stack can grow big, limit the size.
  if (exestack.ga_maxlen - exestack.ga_len > 500) {
    // Keep 150% of the current size, with a minimum of the growth size.
    int n = exestack.ga_len / 2;
    if (n < exestack.ga_growsize) {
      n = exestack.ga_growsize;
    }

    // Don't make it bigger though.
    if (exestack.ga_len + n < exestack.ga_maxlen) {
      size_t new_len = (size_t)exestack.ga_itemsize * (size_t)(exestack.ga_len + n);
      char *pp = xrealloc(exestack.ga_data, new_len);
      exestack.ga_maxlen = exestack.ga_len + n;
      exestack.ga_data = pp;
    }
  }

  // We advance by two (COPYID_INC) because we add one for items referenced
  // through previous_funccal.
  const int copyID = rs_get_copyID();

  // 1. Go through all accessible variables and mark all lists and dicts
  // with copyID.

  // Don't free variables in the previous_funccal list unless they are only
  // referenced through previous_funccal.  This must be first, because if
  // the item is referenced elsewhere the funccal must not be freed.
  ABORTING(set_ref_in_previous_funccal)(copyID);

  // script-local variables
  ABORTING(garbage_collect_scriptvars)(copyID);

  FOR_ALL_BUFFERS(buf) {
    // buffer-local variables
    ABORTING(rs_set_ref_in_item)(&buf->b_bufvar.di_tv, copyID, NULL, NULL);

    // buffer callback functions
    ABORTING(rs_set_ref_in_callback)(&buf->b_prompt_callback, copyID, NULL, NULL);
    ABORTING(rs_set_ref_in_callback)(&buf->b_prompt_interrupt, copyID, NULL, NULL);
    ABORTING(rs_set_ref_in_callback)(&buf->b_cfu_cb, copyID, NULL, NULL);
    ABORTING(rs_set_ref_in_callback)(&buf->b_ofu_cb, copyID, NULL, NULL);
    ABORTING(rs_set_ref_in_callback)(&buf->b_tsrfu_cb, copyID, NULL, NULL);
    ABORTING(rs_set_ref_in_callback)(&buf->b_tfu_cb, copyID, NULL, NULL);
    ABORTING(rs_set_ref_in_callback)(&buf->b_ffu_cb, copyID, NULL, NULL);
    if (!abort && buf->b_p_cpt_cb != NULL) {
      ABORTING(set_ref_in_cpt_callbacks)(buf->b_p_cpt_cb, buf->b_p_cpt_count, copyID);
    }
  }

  // 'completefunc', 'omnifunc' and 'thesaurusfunc' callbacks
  ABORTING(set_ref_in_insexpand_funcs)(copyID);

  // 'operatorfunc' callback
  ABORTING(set_ref_in_opfunc)(copyID);

  // 'tagfunc' callback
  ABORTING(rs_set_ref_in_tagfunc)(copyID);

  // 'findfunc' callback
  ABORTING(set_ref_in_findfunc)(copyID);

  FOR_ALL_TAB_WINDOWS(tp, wp) {
    // window-local variables
    ABORTING(rs_set_ref_in_item)(&wp->w_winvar.di_tv, copyID, NULL, NULL);
  }
  // window-local variables in autocmd windows
  for (int i = 0; i < AUCMD_WIN_COUNT; i++) {
    if (aucmd_win[i].auc_win != NULL) {
      ABORTING(rs_set_ref_in_item)(&aucmd_win[i].auc_win->w_winvar.di_tv, copyID, NULL, NULL);
    }
  }

  // registers (ShaDa additional data)
  {
    const void *reg_iter = NULL;
    do {
      yankreg_T reg;
      char name = NUL;
      bool is_unnamed = false;
      reg_iter = op_global_reg_iter(reg_iter, &name, &reg, &is_unnamed);
    } while (reg_iter != NULL);
  }

  // global marks (ShaDa additional data)
  {
    const void *mark_iter = NULL;
    do {
      xfmark_T fm;
      char name = NUL;
      mark_iter = mark_global_iter(mark_iter, &name, &fm);
    } while (mark_iter != NULL);
  }

  // tabpage-local variables
  FOR_ALL_TABS(tp) {
    ABORTING(rs_set_ref_in_item)(&tp->tp_winvar.di_tv, copyID, NULL, NULL);
  }

  // global variables
  ABORTING(garbage_collect_globvars)(copyID);

  // function-local variables
  ABORTING(set_ref_in_call_stack)(copyID);

  // named functions (matters for closures)
  ABORTING(set_ref_in_functions)(copyID);

  // Channels
  {
    Channel *data;
    map_foreach_value(&channels, data, {
      rs_set_ref_in_callback_reader(&data->on_data, copyID, NULL, NULL);
      rs_set_ref_in_callback_reader(&data->on_stderr, copyID, NULL, NULL);
      rs_set_ref_in_callback(&data->on_exit, copyID, NULL, NULL);
    })
  }

  // Timers
  {
    timer_T *timer;
    map_foreach_value(&timers, timer, {
      rs_set_ref_in_callback(&timer->callback, copyID, NULL, NULL);
    })
  }

  // function call arguments, if v:testing is set.
  ABORTING(set_ref_in_func_args)(copyID);

  // v: vars
  ABORTING(garbage_collect_vimvars)(copyID);

  ABORTING(set_ref_in_quickfix)(copyID);

  bool did_free = false;
  if (!abort) {
    // 2. Free lists and dictionaries that are not referenced.
    did_free = free_unref_items(copyID);

    // 3. Check if any funccal can be freed now.
    //    This may call us back recursively.
    did_free = free_unref_funccal(copyID, testing) || did_free;
  } else if (p_verbose > 0) {
    verb_msg(_("Not enough memory to set references, garbage collection aborted!"));
  }
#undef ABORTING
  return did_free;
}

/// Free lists and dictionaries that are no longer referenced.
///
/// @note  This function may only be called from garbage_collect().
///
/// @param copyID  Free lists/dictionaries that don't have this ID.
///
/// @return  true, if something was freed.
static int free_unref_items(int copyID)
{
  return rs_free_unref_items(copyID);
}

/// Convert the string to a floating point number
///
/// This uses strtod().  setlocale(LC_NUMERIC, "C") has been used earlier to
/// make sure this always uses a decimal point.
///
/// Get the value of an environment variable.
///
/// If the environment variable was not set, silently assume it is empty.
///
/// @param arg  Points to the '$'.  It is advanced to after the name.
///
/// @return  FAIL if the name is invalid.
static int eval_env_var(char **arg, typval_T *rettv, int evaluate)
{
  (*arg)++;
  char *name = *arg;
  int len = rs_get_env_len((const char **)arg);

  if (evaluate) {
    if (len == 0) {
      return FAIL;  // Invalid empty name.
    }
    int cc = (int)name[len];
    name[len] = NUL;
    // First try vim_getenv(), fast for normal environment vars.
    char *string = vim_getenv(name);
    if (string == NULL || *string == NUL) {
      xfree(string);

      // Next try expanding things like $VIM and ${HOME}.
      string = expand_env_save(name - 1);
      if (string != NULL && *string == '$') {
        XFREE_CLEAR(string);
      }
    }
    name[len] = (char)cc;
    rettv->v_type = VAR_STRING;
    rettv->vval.v_string = string;
    rettv->v_lock = VAR_UNLOCKED;
  }

  return OK;
}

/// Non-static wrapper for static eval_env_var - accessor for Rust rs_eval7.
int nvim_eval_env_var_wrapper(char **arg, typval_T *rettv, int evaluate)
{
  return eval_env_var(arg, rettv, evaluate);
}

/// Builds a process argument vector from a Vimscript object (typval_T).
///
/// @param[in]  cmd_tv      Vimscript object
/// @param[out] cmd         Returns the command or executable name.
/// @param[out] executable  Returns `false` if argv[0] is not executable.
///
/// @return  Result of `shell_build_argv()` if `cmd_tv` is a String.
///          Else, string values of `cmd_tv` copied to a (char **) list with
///          argv[0] resolved to full path ($PATHEXT-resolved on Windows).
char **tv_to_argv(typval_T *cmd_tv, const char **cmd, bool *executable)
{
  if (cmd_tv->v_type == VAR_STRING) {  // String => "shell semantics".
    const char *cmd_str = tv_get_string(cmd_tv);
    if (cmd) {
      *cmd = cmd_str;
    }
    return shell_build_argv(cmd_str, NULL);
  }

  if (cmd_tv->v_type != VAR_LIST) {
    semsg(_(e_invarg2), "expected String or List");
    return NULL;
  }

  list_T *argl = cmd_tv->vval.v_list;
  int argc = tv_list_len(argl);
  if (!argc) {
    emsg(_(e_invarg));  // List must have at least one item.
    return NULL;
  }

  const char *arg0 = tv_get_string_chk(TV_LIST_ITEM_TV(tv_list_first(argl)));
  char *exe_resolved = NULL;
  if (!arg0 || !os_can_exe(arg0, &exe_resolved, true)) {
    if (arg0 && executable) {
      char buf[IOSIZE];
      snprintf(buf, sizeof(buf), "'%s' is not executable", arg0);
      semsg(_(e_invargNval), "cmd", buf);
      *executable = false;
    }
    return NULL;
  }

  if (cmd) {
    *cmd = exe_resolved;
  }

  // Build the argument vector
  int i = 0;
  char **argv = xcalloc((size_t)argc + 1, sizeof(char *));
  TV_LIST_ITER_CONST(argl, arg, {
    const char *a = tv_get_string_chk(TV_LIST_ITEM_TV(arg));
    if (!a) {
      // Did emsg in tv_get_string_chk; just deallocate argv.
      shell_free_argv(argv);
      xfree(exe_resolved);
      return NULL;
    }
    argv[i++] = xstrdup(a);
  });
  // Replace argv[0] with absolute path. The only reason for this is to make
  // $PATHEXT work on Windows with jobstart([…]). #9569
  xfree(argv[0]);
  argv[0] = exe_resolved;

  return argv;
}

static list_T *string_to_list(const char *str, size_t len, const bool keepempty)
{
  if (!keepempty && str[len - 1] == NL) {
    len--;
  }
  list_T *const list = tv_list_alloc(kListLenMayKnow);
  encode_list_write(list, str, len);
  return list;
}

/// os_system wrapper. Handles 'verbose', :profile, and v:shell_error.
static void get_system_output_as_rettv(typval_T *argvars, typval_T *rettv, bool retlist)
{
  proftime_T wait_time;
  bool profiling = do_profiling == PROF_YES;

  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = NULL;

  if (rs_check_secure()) {
    return;
  }

  // get input to the shell command (if any), and its length
  ptrdiff_t input_len;
  char *input = save_tv_as_string(&argvars[1], &input_len, false, false);
  if (input_len < 0) {
    assert(input == NULL);
    return;
  }

  // get shell command to execute
  bool executable = true;
  char **argv = tv_to_argv(&argvars[0], NULL, &executable);
  if (!argv) {
    if (!executable) {
      set_vim_var_nr(VV_SHELL_ERROR, -1);
    }
    xfree(input);
    return;  // Already did emsg.
  }

  if (p_verbose > 3) {
    char *cmdstr = shell_argv_to_str(argv);
    verbose_enter_scroll();
    smsg(0, _("Executing command: \"%s\""), cmdstr);
    msg_puts("\n\n");
    verbose_leave_scroll();
    xfree(cmdstr);
  }

  if (profiling) {
    prof_child_enter(&wait_time);
  }

  // execute the command
  size_t nread = 0;
  char *res = NULL;
  int status = os_system(argv, input, (size_t)input_len, &res, &nread);

  if (profiling) {
    prof_child_exit(&wait_time);
  }

  xfree(input);

  set_vim_var_nr(VV_SHELL_ERROR, status);

  if (res == NULL) {
    if (retlist) {
      // return an empty list when there's no output
      tv_list_alloc_ret(rettv, 0);
    } else {
      rettv->vval.v_string = xstrdup("");
    }
    return;
  }

  if (retlist) {
    int keepempty = 0;
    if (argvars[1].v_type != VAR_UNKNOWN && argvars[2].v_type != VAR_UNKNOWN) {
      keepempty = (int)tv_get_number(&argvars[2]);
    }
    rettv->vval.v_list = string_to_list(res, nread, (bool)keepempty);
    tv_list_ref(rettv->vval.v_list);
    rettv->v_type = VAR_LIST;

    xfree(res);
  } else {
    // res may contain several NULs before the final terminating one.
    // Replace them with SOH (1) like in get_cmd_output() to avoid truncation.
    memchrsub(res, NUL, 1, nread);
#ifdef USE_CRNL
    // translate <CR><NL> into <NL>
    char *d = res;
    for (char *s = res; *s; s++) {
      if (s[0] == CAR && s[1] == NL) {
        s++;
      }

      *d++ = *s;
    }

    *d = NUL;
#endif
    rettv->vval.v_string = res;
  }
}

/// f_system - the Vimscript system() function
void f_system(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  get_system_output_as_rettv(argvars, rettv, false);
}

void f_systemlist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  get_system_output_as_rettv(argvars, rettv, true);
}

static int callback_depth = 0;

/// C accessor for callback_depth static.
int nvim_get_callback_depth(void)
{
  return callback_depth;
}

// =============================================================================
// Accessors for rs_callback_call (Phase 4)
// =============================================================================

/// Get callback->data.funcref name string.
char *nvim_cb_get_funcref_name(const Callback *cb)
{
  return cb->data.funcref;
}

/// Get callback->data.luaref.
LuaRef nvim_cb_get_luaref(const Callback *cb)
{
  return cb->data.luaref;
}

/// Increment callback_depth.
void nvim_callback_depth_inc(void)
{
  callback_depth++;
}

/// Decrement callback_depth.
void nvim_callback_depth_dec(void)
{
  callback_depth--;
}

/// Check if callback_depth > p_mfd.
bool nvim_callback_depth_exceeded(void)
{
  return callback_depth > p_mfd;
}

/// Check if name is a v:lua funcref and return the Lua function name portion.
/// Returns NULL if not a v:lua funcref or if name is invalid.
/// Caller must not free the returned pointer.
const char *nvim_cb_check_vlua_funcref(const char *name)
{
  int len = (int)strlen(name);
  if (len >= 6 && !memcmp(name, "v:lua.", 6)) {
    const char *luaname = name + 6;
    int lualen = rs_check_luafunc_name(luaname, false);
    if (lualen == 0) {
      return NULL;
    }
    return luaname;
  }
  return NULL;
}

/// Get the VV_LUA partial.
partial_T *nvim_get_vv_lua_partial(void)
{
  return get_vim_var_partial(VV_LUA);
}

/// Handle the kCallbackLua case: call nlua_call_ref and return LUARET_TRUTHY.
bool nvim_callback_call_lua(LuaRef luaref)
{
  Array args = ARRAY_DICT_INIT;
  Object rv = nlua_call_ref(luaref, NULL, args, kRetNilBool, NULL, NULL);
  return LUARET_TRUTHY(rv);
}

/// Call a funcref or partial callback (handles funcexe_T construction).
/// @param name     Function name
/// @param partial  partial_T * or NULL
/// @param argcount Number of arguments
/// @param argvars  Argument typvals
/// @param rettv    Result typval
/// @return  true on success (call_func returned OK)
bool nvim_callback_call_func(const char *name, partial_T *partial,
                             int argcount, typval_T *argvars, typval_T *rettv)
{
  funcexe_T funcexe = FUNCEXE_INIT;
  funcexe.fe_firstline = curwin->w_cursor.lnum;
  funcexe.fe_lastline = curwin->w_cursor.lnum;
  funcexe.fe_evaluate = true;
  funcexe.fe_partial = partial;

  return call_func(name, -1, rettv, argcount, argvars, &funcexe);
}

/// @return  whether the callback could be called.
bool callback_call(Callback *const callback, const int argcount_in, typval_T *const argvars_in,
                   typval_T *const rettv)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_callback_call(callback, argcount_in, argvars_in, rettv);
}

timer_T *find_timer_by_nr(varnumber_T xx)
{
  return pmap_get(uint64_t)(&timers, (uint64_t)xx);
}

void add_timer_info(typval_T *rettv, timer_T *timer)
{
  list_T *list = rettv->vval.v_list;
  dict_T *dict = tv_dict_alloc();

  tv_list_append_dict(list, dict);
  tv_dict_add_nr(dict, S_LEN("id"), timer->timer_id);
  tv_dict_add_nr(dict, S_LEN("time"), timer->timeout);
  tv_dict_add_nr(dict, S_LEN("paused"), timer->paused);

  tv_dict_add_nr(dict, S_LEN("repeat"),
                 (timer->repeat_count < 0 ? -1 : timer->repeat_count));

  dictitem_T *di = tv_dict_item_alloc("callback");
  if (tv_dict_add(dict, di) == FAIL) {
    xfree(di);
    return;
  }

  callback_put(&timer->callback, &di->di_tv);
}

void add_timer_info_all(typval_T *rettv)
{
  tv_list_alloc_ret(rettv, map_size(&timers));
  timer_T *timer;
  map_foreach_value(&timers, timer, {
    if (!timer->stopped || timer->refcount > 1) {
      add_timer_info(rettv, timer);
    }
  })
}

/// invoked on the main loop
void timer_due_cb(TimeWatcher *tw, void *data)
{
  timer_T *timer = (timer_T *)data;
  int save_did_emsg = did_emsg;
  const int called_emsg_before = called_emsg;
  const bool save_ex_pressedreturn = get_pressedreturn();

  if (timer->stopped || timer->paused) {
    return;
  }

  timer->refcount++;
  // if repeat was negative repeat forever
  if (timer->repeat_count >= 0 && --timer->repeat_count == 0) {
    timer_stop(timer);
  }

  typval_T argv[2] = { TV_INITIAL_VALUE, TV_INITIAL_VALUE };
  argv[0].v_type = VAR_NUMBER;
  argv[0].vval.v_number = timer->timer_id;
  typval_T rettv = TV_INITIAL_VALUE;

  callback_call(&timer->callback, 1, argv, &rettv);

  // Handle error message
  if (called_emsg > called_emsg_before && did_emsg) {
    timer->emsg_count++;
    if (did_throw) {
      discard_current_exception();
    }
  }
  did_emsg = save_did_emsg;
  set_pressedreturn(save_ex_pressedreturn);

  if (timer->emsg_count >= 3) {
    timer_stop(timer);
  }

  tv_clear(&rettv);

  if (!timer->stopped && timer->timeout == 0) {
    // special case: timeout=0 means the callback will be
    // invoked again on the next event loop tick.
    // we don't use uv_idle_t to not spin the event loop
    // when the main loop is blocked.
    time_watcher_start(&timer->tw, timer_due_cb, 0, 0);
  }
  timer_decref(timer);
}

uint64_t timer_start(const int64_t timeout, const int repeat_count, const Callback *const callback)
{
  timer_T *timer = xmalloc(sizeof *timer);
  timer->refcount = 1;
  timer->stopped = false;
  timer->paused = false;
  timer->emsg_count = 0;
  timer->repeat_count = repeat_count;
  timer->timeout = timeout;
  timer->timer_id = (int)last_timer_id++;
  timer->callback = *callback;

  time_watcher_init(&main_loop, &timer->tw, timer);
  timer->tw.events = multiqueue_new_child(loop_get_events(&main_loop));
  // if main loop is blocked, don't queue up multiple events
  timer->tw.blockable = true;
  time_watcher_start(&timer->tw, timer_due_cb, (uint64_t)timeout, (uint64_t)timeout);

  pmap_put(uint64_t)(&timers, (uint64_t)timer->timer_id, timer);
  return (uint64_t)timer->timer_id;
}

void timer_stop(timer_T *timer)
{
  if (timer->stopped) {
    // avoid double free
    return;
  }
  timer->stopped = true;
  time_watcher_stop(&timer->tw);
  time_watcher_close(&timer->tw, timer_close_cb);
}

/// This will be run on the main loop after the last timer_due_cb, so at this
/// point it is safe to free the callback.
static void timer_close_cb(TimeWatcher *tw, void *data)
{
  timer_T *timer = (timer_T *)data;
  multiqueue_free(timer->tw.events);
  callback_free(&timer->callback);
  pmap_del(uint64_t)(&timers, (uint64_t)timer->timer_id, NULL);
  timer_decref(timer);
}

static void timer_decref(timer_T *timer)
{
  if (--timer->refcount == 0) {
    xfree(timer);
  }
}

void timer_stop_all(void)
{
  timer_T *timer;
  map_foreach_value(&timers, timer, {
    timer_stop(timer);
  })
}

void timer_teardown(void)
{
  timer_stop_all();
}

/// Saves a typval_T as a string.
///
/// For lists or buffers, replaces NLs with NUL and separates items with NLs.
///
/// @param[in]  tv   Value to store as a string.
/// @param[out] len  Length of the resulting string or -1 on error.
/// @param[in]  endnl If true, the output will end in a newline (if a list).
/// @param[in]  crlf  If true, list items will be joined with CRLF (if a list).
/// @returns an allocated string if `tv` represents a Vimscript string, list, or
///          number; NULL otherwise.
char *save_tv_as_string(typval_T *tv, ptrdiff_t *const len, bool endnl, bool crlf)
  FUNC_ATTR_MALLOC FUNC_ATTR_NONNULL_ALL
{
  *len = 0;
  if (tv->v_type == VAR_UNKNOWN) {
    return NULL;
  }

  // For other types, let tv_get_string_buf_chk() get the value or
  // print an error.
  if (tv->v_type != VAR_LIST && tv->v_type != VAR_NUMBER) {
    const char *ret = tv_get_string_chk(tv);
    if (ret) {
      *len = (ptrdiff_t)strlen(ret);
      return xmemdupz(ret, (size_t)(*len));
    } else {
      *len = -1;
      return NULL;
    }
  }

  if (tv->v_type == VAR_NUMBER) {  // Treat number as a buffer-id.
    buf_T *buf = buflist_findnr((int)tv->vval.v_number);
    if (buf) {
      for (linenr_T lnum = 1; lnum <= buf->b_ml.ml_line_count; lnum++) {
        for (char *p = ml_get_buf(buf, lnum); *p != NUL; p++) {
          *len += 1;
        }
        *len += 1;
      }
    } else {
      semsg(_(e_nobufnr), tv->vval.v_number);
      *len = -1;
      return NULL;
    }

    if (*len == 0) {
      return NULL;
    }

    char *ret = xmalloc((size_t)(*len) + 1);
    char *end = ret;
    for (linenr_T lnum = 1; lnum <= buf->b_ml.ml_line_count; lnum++) {
      for (char *p = ml_get_buf(buf, lnum); *p != NUL; p++) {
        *end++ = (*p == '\n') ? NUL : *p;
      }
      *end++ = '\n';
    }
    *end = NUL;
    *len = end - ret;
    return ret;
  }

  assert(tv->v_type == VAR_LIST);
  // Pre-calculate the resulting length.
  list_T *list = tv->vval.v_list;
  TV_LIST_ITER_CONST(list, li, {
    *len += (ptrdiff_t)strlen(tv_get_string(TV_LIST_ITEM_TV(li))) + (crlf ? 2 : 1);
  });

  if (*len == 0) {
    return NULL;
  }

  char *ret = xmalloc((size_t)(*len) + (endnl ? (crlf ? 2 : 1) : 0));
  char *end = ret;
  TV_LIST_ITER_CONST(list, li, {
    for (const char *s = tv_get_string(TV_LIST_ITEM_TV(li)); *s != NUL; s++) {
      *end++ = (*s == '\n') ? NUL : *s;
    }
    if (endnl || TV_LIST_ITEM_NEXT(list, li) != NULL) {
      if (crlf) {
        *end++ = '\r';
      }
      *end++ = '\n';
    }
  });
  *end = NUL;
  *len = end - ret;
  return ret;
}

/// Translate a Vimscript object into a position
///
/// Accepts VAR_LIST and VAR_STRING objects. Does not give an error for invalid
/// type.
///
/// @param[in]  tv  Object to translate.
/// @param[in]  dollar_lnum  True when "$" is last line.
/// @param[out]  ret_fnum  Set to fnum for marks.
/// @param[in]  charcol  True to return character column.
///
/// @return Pointer to position or NULL in case of error (e.g. invalid type).
pos_T *var2fpos(const typval_T *const tv, const bool dollar_lnum, int *const ret_fnum,
                const bool charcol)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  static pos_T pos;

  // Argument can be [lnum, col, coladd].
  if (tv->v_type == VAR_LIST) {
    bool error = false;

    list_T *l = tv->vval.v_list;
    if (l == NULL) {
      return NULL;
    }

    // Get the line number.
    pos.lnum = (linenr_T)tv_list_find_nr(l, 0, &error);
    if (error || pos.lnum <= 0 || pos.lnum > curbuf->b_ml.ml_line_count) {
      // Invalid line number.
      return NULL;
    }

    // Get the column number.
    pos.col = (colnr_T)tv_list_find_nr(l, 1, &error);
    if (error) {
      return NULL;
    }
    int len;
    if (charcol) {
      len = mb_charlen(ml_get(pos.lnum));
    } else {
      len = ml_get_len(pos.lnum);
    }

    // We accept "$" for the column number: last column.
    listitem_T *li = tv_list_find(l, 1);
    if (li != NULL && TV_LIST_ITEM_TV(li)->v_type == VAR_STRING
        && TV_LIST_ITEM_TV(li)->vval.v_string != NULL
        && strcmp(TV_LIST_ITEM_TV(li)->vval.v_string, "$") == 0) {
      pos.col = len + 1;
    }

    // Accept a position up to the NUL after the line.
    if (pos.col == 0 || (int)pos.col > len + 1) {
      // Invalid column number.
      return NULL;
    }
    pos.col--;

    // Get the virtual offset.  Defaults to zero.
    pos.coladd = (colnr_T)tv_list_find_nr(l, 2, &error);
    if (error) {
      pos.coladd = 0;
    }

    return &pos;
  }

  const char *const name = tv_get_string_chk(tv);
  if (name == NULL) {
    return NULL;
  }

  pos.lnum = 0;
  if (name[0] == '.') {
    // cursor
    pos = curwin->w_cursor;
  } else if (name[0] == 'v' && name[1] == NUL) {
    // Visual start
    if (VIsual_active) {
      pos = VIsual;
    } else {
      pos = curwin->w_cursor;
    }
  } else if (name[0] == '\'') {
    // mark
    int mname = (uint8_t)name[1];
    const fmark_T *const fm = mark_get(curbuf, curwin, NULL, kMarkAll, mname);
    if (fm == NULL || fm->mark.lnum <= 0) {
      return NULL;
    }
    pos = fm->mark;
    // Vimscript behavior, only provide fnum if mark is global.
    *ret_fnum = ASCII_ISUPPER(mname) || ascii_isdigit(mname) ? fm->fnum : *ret_fnum;
  }
  if (pos.lnum != 0) {
    if (charcol) {
      pos.col = rs_buf_byteidx_to_charidx(curbuf, pos.lnum, pos.col);
    }
    return &pos;
  }

  pos.coladd = 0;

  if (name[0] == 'w' && dollar_lnum) {
    // the "w_valid" flags are not reset when moving the cursor, but they
    // do matter for update_topline() and validate_botline().
    check_cursor_moved(curwin);

    pos.col = 0;
    if (name[1] == '0') {               // "w0": first visible line
      update_topline(curwin);
      // In silent Ex mode topline is zero, but that's not a valid line
      // number; use one instead.
      pos.lnum = curwin->w_topline > 0 ? curwin->w_topline : 1;
      return &pos;
    } else if (name[1] == '$') {      // "w$": last visible line
      validate_botline(curwin);
      // In silent Ex mode botline is zero, return zero then.
      pos.lnum = curwin->w_botline > 0 ? curwin->w_botline - 1 : 0;
      return &pos;
    }
  } else if (name[0] == '$') {        // last column or line
    if (dollar_lnum) {
      pos.lnum = curbuf->b_ml.ml_line_count;
      pos.col = 0;
    } else {
      pos.lnum = curwin->w_cursor.lnum;
      if (charcol) {
        pos.col = (colnr_T)mb_charlen(get_cursor_line_ptr());
      } else {
        pos.col = get_cursor_line_len();
      }
    }
    return &pos;
  }
  return NULL;
}

/// Convert list in "arg" into position "posp" and optional file number "fnump".
/// When "fnump" is NULL there is no file number, only 3 items: [lnum, col, off]
/// Note that the column is passed on as-is, the caller may want to decrement
/// it to use 1 for the first column.
///
/// @param charcol  if true, use the column as the character index instead of the
///                 byte index.
///
/// @return  FAIL when conversion is not possible, doesn't check the position for
///          validity.
int list2fpos(typval_T *arg, pos_T *posp, int *fnump, colnr_T *curswantp, bool charcol)
{
  list_T *l;

  // List must be: [fnum, lnum, col, coladd, curswant], where "fnum" is only
  // there when "fnump" isn't NULL; "coladd" and "curswant" are optional.
  if (arg->v_type != VAR_LIST
      || (l = arg->vval.v_list) == NULL
      || tv_list_len(l) < (fnump == NULL ? 2 : 3)
      || tv_list_len(l) > (fnump == NULL ? 4 : 5)) {
    return FAIL;
  }

  int i = 0;
  int n;
  if (fnump != NULL) {
    n = (int)tv_list_find_nr(l, i++, NULL);  // fnum
    if (n < 0) {
      return FAIL;
    }
    if (n == 0) {
      n = curbuf->b_fnum;  // Current buffer.
    }
    *fnump = n;
  }

  n = (int)tv_list_find_nr(l, i++, NULL);  // lnum
  if (n < 0) {
    return FAIL;
  }
  posp->lnum = n;

  n = (int)tv_list_find_nr(l, i++, NULL);  // col
  if (n < 0) {
    return FAIL;
  }
  // If character position is specified, then convert to byte position
  // If the line number is zero use the cursor line.
  if (charcol) {
    // Get the text for the specified line in a loaded buffer
    buf_T *buf = buflist_findnr(fnump == NULL ? curbuf->b_fnum : *fnump);
    if (buf == NULL || buf->b_ml.ml_mfp == NULL) {
      return FAIL;
    }
    n = rs_buf_charidx_to_byteidx(buf,
                               posp->lnum == 0 ? curwin->w_cursor.lnum : posp->lnum,
                               n) + 1;
  }
  posp->col = n;

  n = (int)tv_list_find_nr(l, i, NULL);  // off
  if (n < 0) {
    posp->coladd = 0;
  } else {
    posp->coladd = n;
  }

  if (curswantp != NULL) {
    *curswantp = (colnr_T)tv_list_find_nr(l, i + 1, NULL);  // curswant
  }

  return OK;
}

/// Get the length of the name of a variable or function.
/// Only the name is recognized, does not handle ".key" or "[idx]".
///
/// @param arg  is advanced to the first non-white character after the name.
///             If the name contains 'magic' {}'s, expand them and return the
///             expanded name in an allocated string via 'alias' - caller must free.
///
/// @return  -1 if curly braces expansion failed or
///           0 if something else is wrong.
int get_name_len(const char **const arg, char **alias, bool evaluate, bool verbose)
{
  *alias = NULL;    // default to no alias

  if ((*arg)[0] == (char)K_SPECIAL && (*arg)[1] == (char)KS_EXTRA
      && (*arg)[2] == (char)KE_SNR) {
    // Hard coded <SNR>, already translated.
    *arg += 3;
    return rs_get_id_len(arg) + 3;
  }
  int len = eval_fname_script(*arg);
  if (len > 0) {
    // literal "<SID>", "s:" or "<SNR>"
    *arg += len;
  }

  // Find the end of the name; check for {} construction.
  char *expr_start;
  char *expr_end;
  const char *p = rs_find_name_end((*arg), (const char **)&expr_start, (const char **)&expr_end,
                                len > 0 ? 0 : FNE_CHECK_START);
  if (expr_start != NULL) {
    if (!evaluate) {
      len += (int)(p - *arg);
      *arg = skipwhite(p);
      return len;
    }

    // Include any <SID> etc in the expanded string:
    // Thus the -len here.
    char *temp_string = make_expanded_name(*arg - len, expr_start, expr_end, (char *)p);
    if (temp_string == NULL) {
      return -1;
    }
    *alias = temp_string;
    *arg = skipwhite(p);
    return (int)strlen(temp_string);
  }

  len += rs_get_id_len(arg);
  // Only give an error when there is something, otherwise it will be
  // reported at a higher level.
  if (len == 0 && verbose && **arg != NUL) {
    semsg(_(e_invexpr2), *arg);
  }

  return len;
}

/// Expands out the 'magic' {}'s in a variable/function name.
/// Note that this can call itself recursively, to deal with
/// constructs like foo{bar}{baz}{bam}
/// The four pointer arguments point to "foo{expre}ss{ion}bar"
///                      "in_start"      ^
///                      "expr_start"       ^
///                      "expr_end"               ^
///                      "in_end"                            ^
///
/// @return  a new allocated string, which the caller must free or
///          NULL for failure.
static char *make_expanded_name(const char *in_start, char *expr_start, char *expr_end,
                                char *in_end)
{
  if (expr_end == NULL || in_end == NULL) {
    return NULL;
  }

  char *retval = NULL;

  *expr_start = NUL;
  *expr_end = NUL;
  char c1 = *in_end;
  *in_end = NUL;

  char *temp_result = eval_to_string(expr_start + 1, false, false);
  if (temp_result != NULL) {
    size_t retvalsize = (size_t)(expr_start - in_start)
                        + strlen(temp_result)
                        + (size_t)(in_end - expr_end) + 1;
    retval = xmalloc(retvalsize);
    vim_snprintf(retval, retvalsize, "%s%s%s", in_start, temp_result, expr_end + 1);
  }
  xfree(temp_result);

  *in_end = c1;                 // put char back for error messages
  *expr_start = '{';
  *expr_end = '}';

  if (retval != NULL) {
    temp_result = (char *)rs_find_name_end(retval,
                                        (const char **)&expr_start,
                                        (const char **)&expr_end, 0);
    if (expr_start != NULL) {
      // Further expansion!
      temp_result = make_expanded_name(retval, expr_start,
                                       expr_end, temp_result);
      xfree(retval);
      retval = temp_result;
    }
  }

  return retval;
}

/// Set the v:argv list.
void set_argv_var(char **argv, int argc)
{
  list_T *l = tv_list_alloc(argc);

  tv_list_set_lock(l, VAR_FIXED);
  for (int i = 0; i < argc; i++) {
    tv_list_append_string(l, (const char *const)argv[i], -1);
    TV_LIST_ITEM_TV(tv_list_last(l))->v_lock = VAR_FIXED;
  }
  set_vim_var_list(VV_ARGV, l);
}

/// Get v:lua partial pointer (accessor for Rust).
partial_T *nvim_get_vlua_partial(void)
{
  return get_vim_var_partial(VV_LUA);
}

/// check if special v:lua value for calling lua functions
static bool tv_is_luafunc(typval_T *tv)
{
  return tv->v_type == VAR_PARTIAL && rs_is_luafunc(tv->vval.v_partial);
}

/// Handle:
/// - expr[expr], expr[expr:expr] subscript
/// - ".name" lookup
/// - function call with Funcref variable: func(expr)
/// - method call: var->method()
///
/// Can all be combined in any order: dict.func(expr)[idx]['func'](expr)->len()
///
/// @param verbose  give error messages
/// @param start_leader  start of '!' and '-' prefixes
/// @param end_leaderp  end of '!' and '-' prefixes
int handle_subscript(const char **const arg, typval_T *rettv, evalarg_T *const evalarg,
                     bool verbose)
{
  const bool evaluate = evalarg != NULL && (evalarg->eval_flags & EVAL_EVALUATE);
  int ret = OK;
  dict_T *selfdict = NULL;
  const char *lua_funcname = NULL;

  if (tv_is_luafunc(rettv)) {
    if (!evaluate) {
      tv_clear(rettv);
    }

    if (**arg != '.') {
      tv_clear(rettv);
      ret = FAIL;
    } else {
      (*arg)++;

      lua_funcname = *arg;
      const int len = rs_check_luafunc_name(*arg, true);
      if (len == 0) {
        tv_clear(rettv);
        ret = FAIL;
      }
      (*arg) += len;
    }
  }

  // "." is ".name" lookup when we found a dict.
  while (ret == OK
         && (((**arg == '[' || (**arg == '.' && rettv->v_type == VAR_DICT)
               || (**arg == '(' && (!evaluate || tv_is_func(*rettv))))
              && !ascii_iswhite(*(*arg - 1)))
             || (**arg == '-' && (*arg)[1] == '>'))) {
    if (**arg == '(') {
      ret = call_func_rettv((char **)arg, evalarg, rettv, evaluate, selfdict, NULL, lua_funcname);

      // Stop the expression evaluation when immediately aborting on
      // error, or when an interrupt occurred or an exception was thrown
      // but not caught.
      if (aborting()) {
        if (ret == OK) {
          tv_clear(rettv);
        }
        ret = FAIL;
      }
      tv_dict_unref(selfdict);
      selfdict = NULL;
    } else if (**arg == '-') {
      if ((*arg)[2] == '{') {
        // expr->{lambda}()
        ret = eval_lambda((char **)arg, rettv, evalarg, verbose);
      } else {
        // expr->name()
        ret = eval_method((char **)arg, rettv, evalarg, verbose);
      }
    } else {  // **arg == '[' || **arg == '.'
      tv_dict_unref(selfdict);
      if (rettv->v_type == VAR_DICT) {
        selfdict = rettv->vval.v_dict;
        if (selfdict != NULL) {
          selfdict->dv_refcount++;
        }
      } else {
        selfdict = NULL;
      }
      if (eval_index((char **)arg, rettv, evalarg, verbose) == FAIL) {
        tv_clear(rettv);
        ret = FAIL;
      }
    }
  }

  // Turn "dict.Func" into a partial for "Func" bound to "dict".
  if (selfdict != NULL && tv_is_func(*rettv)) {
    set_selfdict(rettv, selfdict);
  }

  tv_dict_unref(selfdict);
  return ret;
}

void set_selfdict(typval_T *const rettv, dict_T *const selfdict)
{
  // Don't do this when "dict.Func" is already a partial that was bound
  // explicitly (pt_auto is false).
  if (rettv->v_type == VAR_PARTIAL && !rettv->vval.v_partial->pt_auto
      && rettv->vval.v_partial->pt_dict != NULL) {
    return;
  }
  make_partial(selfdict, rettv);
}

/// Make a copy of an item
///
/// Lists and Dictionaries are also copied.
///
/// @param[in]  conv  If not NULL, convert all copied strings.
/// @param[in]  from  Value to copy.
/// @param[out]  to  Location where to copy to.
/// @param[in]  deep  If true, use copy the container and all of the contained
///                   containers (nested).
/// @param[in]  copyID  If non-zero then when container is referenced more then
///                     once then copy of it that was already done is used. E.g.
///                     when copying list `list = [list2, list2]` (`list[0] is
///                     list[1]`) var_item_copy with zero copyID will emit
///                     a copy with (`copy[0] isnot copy[1]`), with non-zero it
///                     will emit a copy with (`copy[0] is copy[1]`) like in the
///                     original list. Not used when deep is false.
int var_item_copy(const vimconv_T *const conv, typval_T *const from, typval_T *const to,
                  const bool deep, const int copyID)
  FUNC_ATTR_NONNULL_ARG(2, 3)
{
  static int recurse = 0;
  int ret = OK;

  if (recurse >= DICT_MAXNEST) {
    emsg(_(e_variable_nested_too_deep_for_making_copy));
    return FAIL;
  }
  recurse++;

  switch (from->v_type) {
  case VAR_NUMBER:
  case VAR_FLOAT:
  case VAR_FUNC:
  case VAR_PARTIAL:
  case VAR_BOOL:
  case VAR_SPECIAL:
    tv_copy(from, to);
    break;
  case VAR_STRING:
    if (conv == NULL || conv->vc_type == CONV_NONE
        || from->vval.v_string == NULL) {
      tv_copy(from, to);
    } else {
      to->v_type = VAR_STRING;
      to->v_lock = VAR_UNLOCKED;
      if ((to->vval.v_string = string_convert((vimconv_T *)conv,
                                              from->vval.v_string,
                                              NULL))
          == NULL) {
        to->vval.v_string = xstrdup(from->vval.v_string);
      }
    }
    break;
  case VAR_LIST:
    to->v_type = VAR_LIST;
    to->v_lock = VAR_UNLOCKED;
    if (from->vval.v_list == NULL) {
      to->vval.v_list = NULL;
    } else if (copyID != 0 && tv_list_copyid(from->vval.v_list) == copyID) {
      // Use the copy made earlier.
      to->vval.v_list = tv_list_latest_copy(from->vval.v_list);
      tv_list_ref(to->vval.v_list);
    } else {
      to->vval.v_list = tv_list_copy(conv, from->vval.v_list, deep, copyID);
    }
    if (to->vval.v_list == NULL && from->vval.v_list != NULL) {
      ret = FAIL;
    }
    break;
  case VAR_BLOB:
    tv_blob_copy(from->vval.v_blob, to);
    break;
  case VAR_DICT:
    to->v_type = VAR_DICT;
    to->v_lock = VAR_UNLOCKED;
    if (from->vval.v_dict == NULL) {
      to->vval.v_dict = NULL;
    } else if (copyID != 0 && from->vval.v_dict->dv_copyID == copyID) {
      // use the copy made earlier
      to->vval.v_dict = from->vval.v_dict->dv_copydict;
      to->vval.v_dict->dv_refcount++;
    } else {
      to->vval.v_dict = tv_dict_copy(conv, from->vval.v_dict, deep, copyID);
    }
    if (to->vval.v_dict == NULL && from->vval.v_dict != NULL) {
      ret = FAIL;
    }
    break;
  case VAR_UNKNOWN:
    internal_error("var_item_copy(UNKNOWN)");
    ret = FAIL;
  }
  recurse--;
  return ret;
}

/// ":echo expr1 ..."    print each argument separated with a space, add a
///                      newline at the end.
/// ":echon expr1 ..."   print each argument plain.
void ex_echo(exarg_T *eap)
{
  char *arg = eap->arg;
  typval_T rettv;
  bool atstart = true;
  bool need_clear = true;
  const int did_emsg_before = did_emsg;
  const int called_emsg_before = called_emsg;
  evalarg_T evalarg;

  fill_evalarg_from_eap(&evalarg, eap, eap->skip);

  if (eap->skip) {
    emsg_skip++;
  }
  while (*arg != NUL && *arg != '|' && *arg != '\n' && !got_int) {
    // If eval1() causes an error message the text from the command may
    // still need to be cleared. E.g., "echo 22,44".
    need_clr_eos = true;

    {
      char *p = arg;
      if (eval1(&arg, &rettv, &evalarg) == FAIL) {
        // Report the invalid expression unless the expression evaluation
        // has been cancelled due to an aborting error, an interrupt, or an
        // exception.
        if (!aborting() && did_emsg == did_emsg_before
            && called_emsg == called_emsg_before) {
          semsg(_(e_invexpr2), p);
        }
        need_clr_eos = false;
        break;
      }
      need_clr_eos = false;
    }

    if (!eap->skip) {
      if (atstart) {
        atstart = false;
        msg_ext_set_kind("echo");
        // Call msg_start() after eval1(), evaluating the expression
        // may cause a message to appear.
        if (eap->cmdidx == CMD_echo) {
          if (!msg_didout) {
            // Mark the saved text as finishing the line, so that what
            // follows is displayed on a new line when scrolling back
            // at the more prompt.
            msg_sb_eol();
          }
          msg_start();
        }
      } else if (eap->cmdidx == CMD_echo) {
        msg_puts_hl(" ", echo_hl_id, false);
      }
      char *tofree = encode_tv2echo(&rettv, NULL);
      msg_ext_append = eap->cmdidx == CMD_echon;
      msg_multiline(cstr_as_string(tofree), echo_hl_id, true, false, &need_clear);
      xfree(tofree);
    }
    tv_clear(&rettv);
    arg = skipwhite(arg);
  }
  eap->nextcmd = check_nextcmd(arg);
  clear_evalarg(&evalarg, eap);

  if (eap->skip) {
    emsg_skip--;
  } else {
    // remove text that may still be there from the command
    if (need_clear) {
      msg_clr_eos();
    }
    if (eap->cmdidx == CMD_echo) {
      msg_end();
    }
  }
}

/// ":echohl {name}".
void ex_echohl(exarg_T *eap)
{
  echo_hl_id = syn_name2id(eap->arg);
}

/// C accessor for echo_hl_id static.
int nvim_get_echo_hl_id(void)
{
  return echo_hl_id;
}

/// ":execute expr1 ..." execute the result of an expression.
/// ":echomsg expr1 ..." Print a message
/// ":echoerr expr1 ..." Print an error
/// Each gets spaces around each argument and a newline at the end for
/// echo commands
void ex_execute(exarg_T *eap)
{
  char *arg = eap->arg;
  typval_T rettv;
  int ret = OK;
  garray_T ga;

  ga_init(&ga, 1, 80);

  if (eap->skip) {
    emsg_skip++;
  }
  while (*arg != NUL && *arg != '|' && *arg != '\n') {
    ret = eval1_emsg(&arg, &rettv, eap);
    if (ret == FAIL) {
      break;
    }

    if (!eap->skip) {
      const char *const argstr = eap->cmdidx == CMD_execute
                                 ? tv_get_string(&rettv)
                                 : rettv.v_type == VAR_STRING
                                 ? encode_tv2echo(&rettv, NULL)
                                 : encode_tv2string(&rettv, NULL);
      const size_t len = strlen(argstr);
      ga_grow(&ga, (int)len + 2);
      if (!GA_EMPTY(&ga)) {
        ((char *)(ga.ga_data))[ga.ga_len++] = ' ';
      }
      memcpy((char *)(ga.ga_data) + ga.ga_len, argstr, len + 1);
      if (eap->cmdidx != CMD_execute) {
        xfree((void *)argstr);
      }
      ga.ga_len += (int)len;
    }

    tv_clear(&rettv);
    arg = skipwhite(arg);
  }

  if (ret != FAIL && ga.ga_data != NULL) {
    if (eap->cmdidx == CMD_echomsg) {
      msg_ext_set_kind("echomsg");
      msg(ga.ga_data, echo_hl_id);
    } else if (eap->cmdidx == CMD_echoerr) {
      // We don't want to abort following commands, restore did_emsg.
      int save_did_emsg = did_emsg;
      emsg_multiline(ga.ga_data, "echoerr", HLF_E, true);
      if (!force_abort) {
        did_emsg = save_did_emsg;
      }
    } else if (eap->cmdidx == CMD_execute) {
      do_cmdline(ga.ga_data, eap->ea_getline, eap->cookie, DOCMD_NOWAIT|DOCMD_VERBOSE);
    }
  }

  ga_clear(&ga);

  if (eap->skip) {
    emsg_skip--;
  }

  eap->nextcmd = check_nextcmd(arg);
}

/// Skip over the name of an option variable: "&option", "&g:option" or "&l:option".
///
/// @param[in,out]  arg        Points to the "&" or '+' when called, to "option" when returning.
/// @param[out]     opt_idxp   Set to option index in options[] table.
/// @param[out]     opt_flags  Option flags.
///
/// @return NULL when no option name found. Otherwise pointer to the char after the option name.
const char *find_option_var_end(const char **const arg, OptIndex *const opt_idxp,
                                int *const opt_flags)
{
  const char *p = *arg;

  p++;
  if (*p == 'g' && p[1] == ':') {
    *opt_flags = OPT_GLOBAL;
    p += 2;
  } else if (*p == 'l' && p[1] == ':') {
    *opt_flags = OPT_LOCAL;
    p += 2;
  } else {
    *opt_flags = 0;
  }

  const char *end = find_option_end(p, opt_idxp);
  *arg = end == NULL ? *arg : p;
  return end;
}

void var_set_global(const char *const name, typval_T vartv)
{
  funccal_entry_T funccall_entry;

  save_funccal(&funccall_entry);
  set_var(name, strlen(name), &vartv, false);
  restore_funccal();
}

/// Display script name where an item was last set.
/// Should only be invoked when 'verbose' is non-zero.
void last_set_msg(sctx_T script_ctx)
{
  if (script_ctx.sc_sid == 0) {
    return;
  }

  bool should_free;
  char *p = get_scriptname(script_ctx, &should_free);

  verbose_enter();
  msg_puts(_("\n\tLast set from "));
  msg_puts(p);
  if (script_ctx.sc_lnum > 0) {
    msg_puts(_(line_msg));
    msg_outnum(script_ctx.sc_lnum);
  } else if (script_is_lua(script_ctx.sc_sid)) {
    msg_puts(_(" (run Nvim with -V1 for more details)"));
  }
  if (should_free) {
    xfree(p);
  }
  verbose_leave();
}

/// Perform a substitution on "str" with pattern "pat" and substitute "sub".
/// When "sub" is NULL "expr" is used, must be a VAR_FUNC or VAR_PARTIAL.
/// "flags" can be "g" to do a global substitute.
///
/// @param ret_len  length of returned buffer
///
/// @return  an allocated string, NULL for error.
char *do_string_sub(char *str, size_t len, char *pat, char *sub, typval_T *expr, const char *flags,
                    size_t *ret_len)
{
  regmatch_T regmatch;
  garray_T ga;

  // Make 'cpoptions' empty, so that the 'l' flag doesn't work here
  char *save_cpo = p_cpo;
  p_cpo = empty_string_option;

  ga_init(&ga, 1, 200);

  regmatch.rm_ic = p_ic;
  regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
  if (regmatch.regprog != NULL) {
    char *tail = str;
    char *end = str + len;
    bool do_all = (flags[0] == 'g');
    int sublen;
    char *zero_width = NULL;

    while (vim_regexec_nl(&regmatch, str, (colnr_T)(tail - str))) {
      // Skip empty match except for first match.
      if (regmatch.startp[0] == regmatch.endp[0]) {
        if (zero_width == regmatch.startp[0]) {
          // avoid getting stuck on a match with an empty string
          int i = utfc_ptr2len(tail);
          memmove((char *)ga.ga_data + ga.ga_len, tail, (size_t)i);
          ga.ga_len += i;
          tail += i;
          continue;
        }
        zero_width = regmatch.startp[0];
      }

      // Get some space for a temporary buffer to do the substitution
      // into.  It will contain:
      // - The text up to where the match is.
      // - The substituted text.
      // - The text after the match.
      sublen = vim_regsub(&regmatch, sub, expr, tail, 0, REGSUB_MAGIC);
      if (sublen <= 0) {
        ga_clear(&ga);
        break;
      }
      ga_grow(&ga, (int)((end - tail) + sublen -
                         (regmatch.endp[0] - regmatch.startp[0])));

      // copy the text up to where the match is
      int i = (int)(regmatch.startp[0] - tail);
      memmove((char *)ga.ga_data + ga.ga_len, tail, (size_t)i);
      // add the substituted text
      vim_regsub(&regmatch, sub, expr,
                 (char *)ga.ga_data + ga.ga_len + i, sublen,
                 REGSUB_COPY | REGSUB_MAGIC);
      ga.ga_len += i + sublen - 1;
      tail = regmatch.endp[0];
      if (*tail == NUL) {
        break;
      }
      if (!do_all) {
        break;
      }
    }

    if (ga.ga_data != NULL) {
      STRCPY((char *)ga.ga_data + ga.ga_len, tail);
      ga.ga_len += (int)(end - tail);
    }

    vim_regfree(regmatch.regprog);
  }

  if (ga.ga_data != NULL) {
    str = ga.ga_data;
    len = (size_t)ga.ga_len;
  }
  char *ret = xstrnsave(str, len);
  ga_clear(&ga);
  if (p_cpo == empty_string_option) {
    p_cpo = save_cpo;
  } else {
    // Darn, evaluating {sub} expression or {expr} changed the value.
    // If it's still empty it was changed and restored, need to restore in
    // the complicated way.
    if (*p_cpo == NUL) {
      set_option_value_give_err(kOptCpoptions, CSTR_AS_OPTVAL(save_cpo), 0);
    }
    free_string_option(save_cpo);
  }

  if (ret_len != NULL) {
    *ret_len = len;
  }

  return ret;
}

/// Common code for getting job callbacks for `jobstart`.
///
/// @return true/false on success/failure.
bool common_job_callbacks(dict_T *vopts, CallbackReader *on_stdout, CallbackReader *on_stderr,
                          Callback *on_exit)
{
  if (tv_dict_get_callback(vopts, S_LEN("on_stdout"), &on_stdout->cb)
      && tv_dict_get_callback(vopts, S_LEN("on_stderr"), &on_stderr->cb)
      && tv_dict_get_callback(vopts, S_LEN("on_exit"), on_exit)) {
    on_stdout->buffered = tv_dict_get_number(vopts, "stdout_buffered");
    on_stderr->buffered = tv_dict_get_number(vopts, "stderr_buffered");
    if (on_stdout->buffered && on_stdout->cb.type == kCallbackNone) {
      on_stdout->self = vopts;
    }
    if (on_stderr->buffered && on_stderr->cb.type == kCallbackNone) {
      on_stderr->self = vopts;
    }
    vopts->dv_refcount++;
    return true;
  }

  callback_reader_free(on_stdout);
  callback_reader_free(on_stderr);
  callback_free(on_exit);
  return false;
}

Channel *find_job(uint64_t id, bool show_error)
{
  Channel *data = find_channel(id);
  if (!data || data->streamtype != kChannelStreamProc
      || proc_is_stopped(&data->stream.proc)) {
    if (show_error) {
      if (data && data->streamtype != kChannelStreamProc) {
        emsg(_(e_invchanjob));
      } else {
        emsg(_(e_invchan));
      }
    }
    return NULL;
  }
  return data;
}

void script_host_eval(char *name, typval_T *argvars, typval_T *rettv)
{
  if (rs_check_secure()) {
    return;
  }

  if (argvars[0].v_type != VAR_STRING) {
    emsg(_(e_invarg));
    return;
  }

  list_T *args = tv_list_alloc(1);
  tv_list_append_string(args, argvars[0].vval.v_string, -1);
  *rettv = eval_call_provider(name, "eval", args, false);
}

/// @param discard  Clears the value returned by the provider and returns
///                 an empty typval_T.
typval_T eval_call_provider(char *provider, char *method, list_T *arguments, bool discard)
{
  if (!eval_has_provider(provider, false)) {
    semsg("E319: No \"%s\" provider found. Run \":checkhealth vim.provider\"",
          provider);
    return (typval_T){
      .v_type = VAR_NUMBER,
      .v_lock = VAR_UNLOCKED,
      .vval.v_number = 0
    };
  }

  char func[256];
  int name_len = snprintf(func, sizeof(func), "provider#%s#Call", provider);

  // Save caller scope information
  struct caller_scope saved_provider_caller_scope = provider_caller_scope;
  provider_caller_scope = (struct caller_scope) {
    .script_ctx = current_sctx,
    .es_entry = ((estack_T *)exestack.ga_data)[exestack.ga_len - 1],
    .autocmd_fname = autocmd_fname,
    .autocmd_match = autocmd_match,
    .autocmd_fname_full = autocmd_fname_full,
    .autocmd_bufnr = autocmd_bufnr,
    .funccalp = (void *)get_current_funccal()
  };
  funccal_entry_T funccal_entry;
  save_funccal(&funccal_entry);
  provider_call_nesting++;

  typval_T argvars[3] = {
    { .v_type = VAR_STRING, .vval.v_string = method,
      .v_lock = VAR_UNLOCKED },
    { .v_type = VAR_LIST, .vval.v_list = arguments, .v_lock = VAR_UNLOCKED },
    { .v_type = VAR_UNKNOWN }
  };
  typval_T rettv = { .v_type = VAR_UNKNOWN, .v_lock = VAR_UNLOCKED };
  tv_list_ref(arguments);

  funcexe_T funcexe = FUNCEXE_INIT;
  funcexe.fe_firstline = curwin->w_cursor.lnum;
  funcexe.fe_lastline = curwin->w_cursor.lnum;
  funcexe.fe_evaluate = true;
  call_func(func, name_len, &rettv, 2, argvars, &funcexe);

  tv_list_unref(arguments);
  // Restore caller scope information
  restore_funccal();
  provider_caller_scope = saved_provider_caller_scope;
  provider_call_nesting--;
  assert(provider_call_nesting >= 0);

  if (discard) {
    tv_clear(&rettv);
  }

  return rettv;
}

/// Checks if provider for feature `feat` is enabled.
bool eval_has_provider(const char *feat, bool throw_if_fast)
{
  if (!strequal(feat, "clipboard")
      && !strequal(feat, "python3")
      && !strequal(feat, "python3_compiled")
      && !strequal(feat, "python3_dynamic")
      && !strequal(feat, "perl")
      && !strequal(feat, "ruby")
      && !strequal(feat, "node")) {
    // Avoid autoload for non-provider has() features.
    return false;
  }

  if (throw_if_fast && !nlua_is_deferred_safe()) {
    semsg(e_fast_api_disabled, "Vimscript function");
    return false;
  }

  char name[32];  // Normalized: "python3_compiled" => "python3".
  snprintf(name, sizeof(name), "%s", feat);
  strchrsub(name, '_', NUL);  // Chop any "_xx" suffix.

  char buf[256];
  typval_T tv;
  // Get the g:loaded_xx_provider variable.
  int len = snprintf(buf, sizeof(buf), "g:loaded_%s_provider", name);
  if (eval_variable(buf, len, &tv, NULL, false, true) == FAIL) {
    // Trigger autoload once.
    len = snprintf(buf, sizeof(buf), "provider#%s#bogus", name);
    script_autoload(buf, (size_t)len, false);

    // Retry the (non-autoload-style) variable.
    len = snprintf(buf, sizeof(buf), "g:loaded_%s_provider", name);
    if (eval_variable(buf, len, &tv, NULL, false, true) == FAIL) {
      // Show a hint if Call() is defined but g:loaded_xx_provider is missing.
      snprintf(buf, sizeof(buf), "provider#%s#Call", name);
      if (!!find_func(buf) && p_lpl) {
        semsg("provider: %s: missing required variable g:loaded_%s_provider",
              name, name);
      }
      return false;
    }
  }

  bool ok = (tv.v_type == VAR_NUMBER)
            ? 2 == tv.vval.v_number  // Value of 2 means "loaded and working".
            : false;

  if (ok) {
    // Call() must be defined if provider claims to be working.
    snprintf(buf, sizeof(buf), "provider#%s#Call", name);
    if (!find_func(buf)) {
      semsg("provider: %s: g:loaded_%s_provider=2 but %s is not defined",
            name, name, buf);
      ok = false;
    }
  }

  return ok;
}

/// Writes "<sourcing_name>:<sourcing_lnum>" to `buf[bufsize]`.
void eval_fmt_source_name_line(char *buf, size_t bufsize)
{
  if (SOURCING_NAME) {
    snprintf(buf, bufsize, "%s:%" PRIdLINENR, SOURCING_NAME, SOURCING_LNUM);
  } else {
    snprintf(buf, bufsize, "?");
  }
}

/// Gets the current user-input in prompt buffer `buf`, or NULL if buffer is not a prompt buffer.
char *prompt_get_input(buf_T *buf)
{
  if (!bt_prompt(buf)) {
    return NULL;
  }
  linenr_T lnum_start = buf->b_prompt_start.mark.lnum;
  linenr_T lnum_last = buf->b_ml.ml_line_count;

  char *text = ml_get_buf(buf, lnum_start);
  char *prompt = prompt_text();
  if (strlen(text) >= strlen(prompt)) {
    text += strlen(prompt);
  }

  char *full_text = xstrdup(text);
  for (linenr_T i = lnum_start + 1; i <= lnum_last; i++) {
    char *half_text = concat_str(full_text, "\n");
    xfree(full_text);
    full_text = concat_str(half_text, ml_get_buf(buf, i));
    xfree(half_text);
  }
  return full_text;
}

/// Invokes the user-defined callback defined for the current prompt-buffer.
void prompt_invoke_callback(void)
{
  typval_T rettv;
  typval_T argv[2];
  linenr_T lnum = curbuf->b_ml.ml_line_count;

  char *user_input = prompt_get_input(curbuf);

  if (!user_input) {
    return;
  }

  // Add a new line for the prompt before invoking the callback, so that
  // text can always be inserted above the last line.
  ml_append(lnum, "", 0, false);
  appended_lines_mark(lnum, 1);
  curwin->w_cursor.lnum = lnum + 1;
  curwin->w_cursor.col = 0;
  curbuf->b_prompt_start.mark.lnum = lnum + 1;

  if (curbuf->b_prompt_callback.type == kCallbackNone) {
    xfree(user_input);
    goto theend;
  }

  argv[0].v_type = VAR_STRING;
  argv[0].vval.v_string = user_input;
  argv[1].v_type = VAR_UNKNOWN;

  callback_call(&curbuf->b_prompt_callback, 1, argv, &rettv);
  tv_clear(&argv[0]);
  tv_clear(&rettv);

theend:
  // clear undo history on submit
  u_clearallandblockfree(curbuf);

  curbuf->b_prompt_start.mark.lnum = curbuf->b_ml.ml_line_count;
}

/// @return  true when the interrupt callback was invoked.
bool invoke_prompt_interrupt(void)
{
  typval_T rettv;
  typval_T argv[1];

  if (curbuf->b_prompt_interrupt.type == kCallbackNone) {
    return false;
  }
  argv[0].v_type = VAR_UNKNOWN;

  got_int = false;  // don't skip executing commands
  int ret = callback_call(&curbuf->b_prompt_interrupt, 0, argv, &rettv);
  tv_clear(&rettv);
  return ret != FAIL;
}


/// Convert any type to a string, never give an error.
/// When "quotes" is true add quotes to a string.
/// Returns an allocated string.
char *typval_tostring(typval_T *arg, bool quotes)
{
  if (arg == NULL) {
    return xstrdup("(does not exist)");
  }
  if (!quotes && arg->v_type == VAR_STRING) {
    return xstrdup(arg->vval.v_string == NULL ? "" : arg->vval.v_string);
  }
  return encode_tv2string(arg, NULL);
}

// =============================================================================
// Accessor functions for Rust FFI
// =============================================================================

/// Get eval_flags from evalarg_T (accessor for Rust).
int evalarg_get_flags(const evalarg_T *evalarg)
{
  return evalarg ? evalarg->eval_flags : 0;
}

/// Set eval_flags in evalarg_T (accessor for Rust).
void evalarg_set_flags(evalarg_T *evalarg, int flags)
{
  if (evalarg) {
    evalarg->eval_flags = flags;
  }
}

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

/// Get partial pointer from typval - accessor for Rust.
partial_T *nvim_tv_get_partial(const typval_T *tv)
{
  return tv->vval.v_partial;
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

/// Zero out lval_T struct - accessor for Rust.
void nvim_lval_clear(lval_T *lp)
{
  CLEAR_POINTER(lp);
}

/// Get ll_name field from lval_T - accessor for Rust.
const char *nvim_lval_get_name(const lval_T *lp)
{
  return lp->ll_name;
}

/// Set ll_name field in lval_T - accessor for Rust.
void nvim_lval_set_name(lval_T *lp, const char *name)
{
  lp->ll_name = name;
}

/// Get ll_name_len field from lval_T - accessor for Rust.
size_t nvim_lval_get_name_len(const lval_T *lp)
{
  return lp->ll_name_len;
}

/// Set ll_name_len field in lval_T - accessor for Rust.
void nvim_lval_set_name_len(lval_T *lp, size_t len)
{
  lp->ll_name_len = len;
}

/// Get ll_exp_name field from lval_T - accessor for Rust.
char *nvim_lval_get_exp_name(const lval_T *lp)
{
  return lp->ll_exp_name;
}

/// Set ll_exp_name field in lval_T - accessor for Rust.
void nvim_lval_set_exp_name(lval_T *lp, char *exp_name)
{
  lp->ll_exp_name = exp_name;
}

/// Get ll_tv field from lval_T - accessor for Rust.
typval_T *nvim_lval_get_tv(const lval_T *lp)
{
  return lp->ll_tv;
}

/// Set ll_tv field in lval_T - accessor for Rust.
void nvim_lval_set_tv(lval_T *lp, typval_T *tv)
{
  lp->ll_tv = tv;
}

/// Get ll_newkey field from lval_T - accessor for Rust.
char *nvim_lval_get_newkey(const lval_T *lp)
{
  return lp->ll_newkey;
}

/// Check if ll_name is NULL - accessor for Rust.
bool nvim_lval_name_is_null(const lval_T *lp)
{
  return lp->ll_name == NULL;
}

/// Wrapper for the static make_expanded_name - accessor for Rust.
char *nvim_make_expanded_name(const char *in_start, char *expr_start, char *expr_end,
                              char *in_end)
{
  return make_expanded_name(in_start, expr_start, expr_end, in_end);
}

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

/// Check if a typval_T is a Lua function - accessor for Rust.
/// This wraps the static tv_is_luafunc function.
bool nvim_tv_is_luafunc_wrapper(typval_T *tv)
{
  return tv_is_luafunc(tv);
}

/// Return address of the EVALARG_EVALUATE global - accessor for Rust.
evalarg_T *nvim_get_evalarg_evaluate_ptr(void)
{
  return &EVALARG_EVALUATE;
}

/// Set ll_name_len to (p - ll_name) - accessor for Rust.
void nvim_lval_compute_name_len(lval_T *lp, const char *p)
{
  lp->ll_name_len = (size_t)(p - lp->ll_name);
}

/// Emit "E488: Trailing characters: %s" error - accessor for Rust.
void nvim_semsg_trailing_arg(const char *p)
{
  semsg(_(e_trailing_arg), p);
}

// nvim_semsg_invarg2 already exists in match.c - reuse it.

/// Emit "E121: Undefined variable: %.*s" error - accessor for Rust.
void nvim_semsg_undef_var(int len, const char *name)
{
  semsg(_("E121: Undefined variable: %.*s"), len, name);
}

// =============================================================================
// Phase 3: set_var_lval helpers (additional lval_T accessors for rs_set_var_lval)
// =============================================================================

/// Get ll_blob field from lval_T - accessor for Rust.
blob_T *nvim_lval_get_blob(const lval_T *lp)
{
  return lp->ll_blob;
}

/// Get ll_range field from lval_T - accessor for Rust.
bool nvim_lval_get_range(const lval_T *lp)
{
  return lp->ll_range;
}

/// Get ll_empty2 field from lval_T - accessor for Rust.
bool nvim_lval_get_empty2(const lval_T *lp)
{
  return lp->ll_empty2;
}

/// Get ll_n1 field from lval_T - accessor for Rust.
int nvim_lval_get_n1(const lval_T *lp)
{
  return lp->ll_n1;
}

/// Get ll_n2 field from lval_T - accessor for Rust.
int nvim_lval_get_n2(const lval_T *lp)
{
  return lp->ll_n2;
}

/// Set ll_n2 field in lval_T - accessor for Rust.
void nvim_lval_set_n2(lval_T *lp, int n2)
{
  lp->ll_n2 = n2;
}

/// Get ll_list field from lval_T - accessor for Rust.
list_T *nvim_lval_get_list(const lval_T *lp)
{
  return lp->ll_list;
}

/// Get ll_dict field from lval_T - accessor for Rust.
dict_T *nvim_lval_get_dict(const lval_T *lp)
{
  return lp->ll_dict;
}

/// Get ll_di field from lval_T - accessor for Rust.
dictitem_T *nvim_lval_get_di(const lval_T *lp)
{
  return lp->ll_di;
}

/// Get bv_lock from a blob_T - accessor for Rust.
VarLockStatus nvim_blob_get_bv_lock(const blob_T *blob)
{
  return blob->bv_lock;
}

/// Get v_lock from a typval_T - accessor for Rust.
VarLockStatus nvim_tv_get_v_lock(const typval_T *tv)
{
  return tv->v_lock;
}

/// Set v_lock in a typval_T - accessor for Rust.
void nvim_tv_set_v_lock(typval_T *tv, VarLockStatus lock)
{
  tv->v_lock = lock;
}

/// Get dv_lock from ll_tv->vval.v_dict - composite accessor for Rust.
/// Returns 0 (VAR_UNLOCKED) if ll_tv or v_dict is NULL.
VarLockStatus nvim_lval_get_dict_dv_lock(const lval_T *lp)
{
  if (lp->ll_tv == NULL || lp->ll_tv->vval.v_dict == NULL) {
    return VAR_UNLOCKED;
  }
  return lp->ll_tv->vval.v_dict->dv_lock;
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

/// Emit "E988: cannot modify existing variable" error via e_cannot_mod - accessor for Rust.
void nvim_emsg_cannot_mod(void)
{
  emsg(_(e_cannot_mod));
}

/// Emit "E1223: letwrong" error with operator - accessor for Rust.
void nvim_semsg_letwrong(const char *op)
{
  semsg(_(e_letwrong), op);
}

/// Emit "E996: Cannot lock a range" error - accessor for Rust.
void nvim_emsg_cannot_lock_range(void)
{
  emsg(_("E996: Cannot lock a range"));
}

/// Emit "E996: Cannot lock a list or dict" error - accessor for Rust.
void nvim_emsg_cannot_lock_list_or_dict(void)
{
  emsg(_("E996: Cannot lock a list or dict"));
}

/// Emit e_dictkey error for a key - accessor for Rust.
void nvim_semsg_dictkey(const char *key)
{
  semsg(_(e_dictkey), key);
}

/// Get TV_CSTRING constant value - accessor for Rust.
int nvim_tv_cstring_flag(void)
{
  return TV_CSTRING;
}

/// value_check_lock wrapper - accessor for Rust.
bool nvim_value_check_lock(int lock, const char *name)
{
  return value_check_lock((VarLockStatus)lock, name, TV_CSTRING);
}

/// Get vval.v_dict from a typval_T (direct field) - accessor for Rust.
dict_T *nvim_tv_get_v_dict(const typval_T *tv)
{
  return tv->vval.v_dict;
}

/// Get vval.v_list from a typval_T (direct field) - accessor for Rust.
list_T *nvim_tv_get_v_list(const typval_T *tv)
{
  return tv->vval.v_list;
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
// Phase 1 continuation: funcexe wrapper
// =============================================================================

/// Construct funcexe_T and call get_func_tv - wrapper for Rust eval_func.
///
/// This avoids replicating the funcexe_T struct layout in Rust.
int nvim_call_func_tv_wrapper(char *name, int len, typval_T *rettv, char **arg,
                              evalarg_T *evalarg, bool evaluate,
                              partial_T *partial, typval_T *basetv,
                              bool found_var, linenr_T lnum)
{
  funcexe_T funcexe = FUNCEXE_INIT;
  funcexe.fe_firstline = lnum;
  funcexe.fe_lastline = lnum;
  funcexe.fe_evaluate = evaluate;
  funcexe.fe_partial = partial;
  funcexe.fe_basetv = basetv;
  funcexe.fe_found_var = found_var;
  return get_func_tv(name, len, rettv, arg, evalarg, &funcexe);
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

// =============================================================================
// Phase 1: eval_index / check_can_index error message accessors
// =============================================================================

/// Emit E111 Missing ']' error - accessor for Rust rs_eval_index.
void nvim_emsg_missbrac(void)
{
  emsg(_(e_missbrac));
}

/// Emit E695 Cannot index a Funcref error - accessor for Rust rs_check_can_index.
void nvim_emsg_cannot_index_funcref(void)
{
  emsg(_(e_cannot_index_a_funcref));
}

/// Emit E806 Using a Float as a String error - accessor for Rust rs_check_can_index.
void nvim_emsg_using_float_as_string(void)
{
  emsg(_(e_using_float_as_string));
}

/// Emit E909 Cannot index a special variable error - accessor for Rust rs_check_can_index.
void nvim_emsg_cannot_index_special(void)
{
  emsg(_(e_cannot_index_special_variable));
}

/// Emit E719 Cannot slice a Dictionary error - accessor for Rust rs_eval_index_inner.
void nvim_emsg_cannot_slice_dict(void)
{
  emsg(_(e_cannot_slice_dictionary));
}

/// Emit E716 Key not present (with length) error - accessor for Rust rs_eval_index_inner.
void nvim_semsg_dictkey_len(ptrdiff_t keylen, const char *key)
{
  semsg(_(e_dictkey_len), keylen, key);
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

/// Increment pt->pt_refcount - accessor for Rust rs_eval_method.
void nvim_partial_incref(partial_T *pt)
{
  pt->pt_refcount++;
}

/// Set tv->vval.v_partial = pt without clearing - accessor for Rust rs_eval_method.
void nvim_tv_set_partial_raw(typval_T *tv, partial_T *pt)
{
  tv->vval.v_partial = pt;
}

/// Non-static wrapper for call_func_rettv with selfdict=NULL - accessor for Rust rs_eval_method.
int nvim_call_func_rettv_wrapper(char **arg, evalarg_T *evalarg, typval_T *rettv, bool evaluate,
                                 typval_T *basetv, const char *lua_funcname)
{
  return call_func_rettv(arg, evalarg, rettv, evaluate, NULL, basetv, lua_funcname);
}

/// Non-static wrapper for rs_eval7 - accessor for Rust rs_eval_method.
int nvim_eval7_wrapper(char **arg, typval_T *rettv, evalarg_T *evalarg, bool want_string)
{
  return rs_eval7(arg, rettv, evalarg, want_string);
}

// =============================================================================
// Phase 1 (lval subscript): new C accessor/wrapper functions for rs_get_lval_subscript
// =============================================================================

/// Set lp->ll_list - accessor for Rust.
void nvim_lval_set_list(lval_T *lp, list_T *list)
{
  lp->ll_list = list;
}

/// Set lp->ll_dict - accessor for Rust.
void nvim_lval_set_dict(lval_T *lp, dict_T *dict)
{
  lp->ll_dict = dict;
}

/// Set lp->ll_di - accessor for Rust.
void nvim_lval_set_di(lval_T *lp, dictitem_T *di)
{
  lp->ll_di = di;
}

/// Set lp->ll_n1 - accessor for Rust.
void nvim_lval_set_n1(lval_T *lp, int n1)
{
  lp->ll_n1 = n1;
}

/// Set lp->ll_range - accessor for Rust.
void nvim_lval_set_range(lval_T *lp, bool range)
{
  lp->ll_range = range;
}

/// Set lp->ll_empty2 - accessor for Rust.
void nvim_lval_set_empty2(lval_T *lp, bool empty2)
{
  lp->ll_empty2 = empty2;
}

/// Set lp->ll_blob - accessor for Rust.
void nvim_lval_set_blob(lval_T *lp, blob_T *blob)
{
  lp->ll_blob = blob;
}

/// Set lp->ll_li - accessor for Rust.
void nvim_lval_set_li(lval_T *lp, listitem_T *li)
{
  lp->ll_li = li;
}

/// Set lp->ll_newkey - accessor for Rust.
void nvim_lval_set_newkey(lval_T *lp, char *key)
{
  lp->ll_newkey = key;
}

/// Get lp->ll_li as opaque void* - accessor for Rust.
listitem_T *nvim_lval_get_li(const lval_T *lp)
{
  return lp->ll_li;
}

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

/// Emit "E689: Can only index a List, Dictionary or Blob" - accessor for Rust.
void nvim_emsg_e689(void)
{
  emsg(_("E689: Can only index a List, Dictionary or Blob"));
}

/// Emit "E708: [:] must come last" - accessor for Rust.
void nvim_emsg_e708(void)
{
  emsg(_("E708: [:] must come last"));
}

/// Emit "E713: Cannot use empty key after ." - accessor for Rust.
void nvim_emsg_e713(void)
{
  emsg(_("E713: Cannot use empty key after ."));
}

/// Emit "E709: [:] requires a List or Blob value" - accessor for Rust.
void nvim_emsg_e709(void)
{
  emsg(_("E709: [:] requires a List or Blob value"));
}

/// Emit e_dot_can_only_be_used_on_dictionary_str with name - accessor for Rust.
void nvim_semsg_e_dot_dict(const char *name)
{
  semsg(_(e_dot_can_only_be_used_on_dictionary_str), name);
}

/// Emit e_illvar with name (no translation - used for v:lua case) - accessor for Rust.
void nvim_semsg_e_illvar_raw(const char *name)
{
  semsg(e_illvar, name);
}

/// Emit e_illvar with name (with translation) - accessor for Rust.
void nvim_semsg_e_illvar(const char *name)
{
  semsg(_(e_illvar), name);
}

/// Emit e_cannot_slice_dictionary - accessor for Rust.
void nvim_semsg_e_cannot_slice_dict(void)
{
  emsg(_(e_cannot_slice_dictionary));
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

/// Get tv->vval.v_dict from the ll_tv - composite accessor for Rust.
dict_T *nvim_lval_tv_get_dict(const lval_T *lp)
{
  return lp->ll_tv->vval.v_dict;
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

/// tv_dict_find wrapper accepting key_len as int for ll_dict - composite for Rust.
dictitem_T *nvim_lval_dict_find(const lval_T *lp, const char *key, int len)
{
  return tv_dict_find(lp->ll_dict, key, (ptrdiff_t)len);
}

/// Set lp->ll_di = tv_dict_find(lp->ll_dict, key, len) - composite setter for Rust.
void nvim_lval_set_di_from_dict(lval_T *lp, const char *key, int len)
{
  lp->ll_di = tv_dict_find(lp->ll_dict, key, (ptrdiff_t)len);
}

/// Get di->di_tv from lp->ll_di - composite accessor for Rust.
typval_T *nvim_lval_di_get_tv(const lval_T *lp)
{
  return &lp->ll_di->di_tv;
}

/// Set lp->ll_tv = &lp->ll_di->di_tv - composite setter for Rust.
void nvim_lval_set_tv_from_ll_di(lval_T *lp)
{
  lp->ll_tv = &lp->ll_di->di_tv;
}

/// Check if lp->ll_di->di_tv is a lua func wrapper - composite accessor for Rust.
bool nvim_lval_di_is_luafunc(const lval_T *lp)
{
  return tv_is_luafunc(&lp->ll_di->di_tv);
}

/// var_check_ro for ll_di->di_flags - composite for Rust.
bool nvim_lval_di_var_check_ro(const lval_T *lp, const char *name, size_t name_len)
{
  return var_check_ro(lp->ll_di->di_flags, name, name_len);
}

/// var_check_lock for ll_di->di_flags - composite for Rust.
bool nvim_lval_di_var_check_lock(const lval_T *lp, const char *name, size_t name_len)
{
  return var_check_lock(lp->ll_di->di_flags, name, name_len);
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

/// valid_varname wrapper - accessor for Rust.
bool nvim_valid_varname(const char *varname)
{
  return valid_varname(varname);
}

/// var_wrong_func_name wrapper - accessor for Rust.
bool nvim_var_wrong_func_name(const char *name, bool new_var)
{
  return var_wrong_func_name(name, new_var);
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

