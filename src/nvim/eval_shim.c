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
extern int rs_handle_subscript(const char **arg, typval_T *rettv, evalarg_T *evalarg,
                               bool verbose);
extern void rs_ex_echo(exarg_T *eap);
extern void rs_ex_execute(exarg_T *eap);
extern int rs_eval_option(const char **arg, typval_T *rettv, bool evaluate);
extern int rs_eval_env_var(char **arg, typval_T *rettv, int evaluate);
extern int rs_var_item_copy(const void *conv, typval_T *from, typval_T *to, bool deep,
                            int copyID);
extern char *rs_save_tv_as_string(typval_T *tv, ptrdiff_t *len, bool endnl, bool crlf);
// Phase 1 (eval_shim pass 5)
extern int rs_call_vim_function(const char *func, int argc, typval_T *argv, typval_T *rettv);
extern void *rs_call_func_retstr(const char *func, int argc, typval_T *argv);
extern void *rs_call_func_retlist(const char *func, int argc, typval_T *argv);
extern void rs_set_argv_var(char **argv, int argc);
extern void rs_var_set_global(const char *name, typval_T *vartv);
extern void rs_eval_fmt_source_name_line(char *buf, size_t bufsize);
extern const char *rs_find_option_var_end(const char **arg, int *opt_idxp, int *opt_flags);
// Phase 2 (eval_shim pass 5)
extern char *rs_prompt_get_input(buf_T *buf);
extern void rs_prompt_invoke_callback(void);
extern bool rs_invoke_prompt_interrupt(void);
// Phase 3 (eval_shim pass 5)
extern int rs_eval_foldexpr(win_T *wp, int *cp);
extern void rs_eval_foldtext(win_T *wp, Object *out);

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

// Rust implementations for Phase 5 (eval_shim pass 4)
extern void rs_fill_evalarg_from_eap(evalarg_T *evalarg, exarg_T *eap, bool skip);
extern void rs_clear_evalarg(evalarg_T *evalarg, exarg_T *eap);
extern int rs_may_call_simple_func(const char *arg, typval_T *rettv);
extern typval_T *rs_eval_expr_ext(char *arg, exarg_T *eap, bool use_simple_function);
extern void rs_partial_unref(partial_T *pt);
extern char *rs_typval_tostring(typval_T *arg, bool quotes);

void fill_evalarg_from_eap(evalarg_T *evalarg, exarg_T *eap, bool skip)
{
  rs_fill_evalarg_from_eap(evalarg, eap, skip);
}

// Rust implementations (in eval_exec crate, eval_top module)
extern bool rs_eval_to_bool(char *arg, bool *error, exarg_T *eap, bool skip,
                             bool use_simple_function);
extern char *rs_eval_to_string_skip(char *arg, exarg_T *eap, bool skip);
extern char *rs_eval_to_string_eap(char *arg, bool join_list, exarg_T *eap,
                                   bool use_simple_function);
extern char *rs_eval_to_string(char *arg, bool join_list, bool use_simple_function);
extern char *rs_eval_to_string_safe(char *arg, bool use_sandbox, bool use_simple_function);
extern int64_t rs_eval_to_number(char *expr, bool use_simple_function);
extern int rs_skip_expr(char **pp, evalarg_T *evalarg);
extern int rs_eval_expr_typval(const typval_T *expr, bool want_func, typval_T *argv, int argc,
                               typval_T *rettv);
extern bool rs_eval_expr_to_bool(const typval_T *expr, bool *error);

// Rust implementations for Phase 1 (eval_shim pass 4, eval_exec crate)
extern int rs_call_func_rettv(char **arg, evalarg_T *evalarg, typval_T *rettv, bool evaluate,
                              void *selfdict, typval_T *basetv, const char *lua_funcname);
extern int rs_eval_lambda(char **arg, typval_T *rettv, evalarg_T *evalarg, bool verbose);
extern int rs_eval1_emsg(char **arg, typval_T *rettv, exarg_T *eap);

// Rust implementations for Phase 3 (in eval crate, indexing module)
extern bool rs_var2fpos(const typval_T *tv, bool dollar_lnum, int *ret_fnum, bool charcol,
                        pos_T *out);
extern int rs_list2fpos(typval_T *arg, pos_T *posp, int *fnump, int *curswantp, bool charcol);

/// Top level evaluation function, returning a boolean.
/// Sets "error" to true if there was an error.
///
/// @param skip  only parse, don't execute
///
/// @return  true or false.
bool eval_to_bool(char *arg, bool *error, exarg_T *eap, const bool skip,
                  const bool use_simple_function)
{
  return rs_eval_to_bool(arg, error, eap, skip, use_simple_function);
}

// eval1_emsg migrated to Rust (rs_eval1_emsg in eval_exec crate, eval.rs).

/// Evaluate an expression, which can be a function, partial or string.
/// Pass arguments "argv[argc]".
/// Return the result in "rettv" and OK or FAIL.
///
/// @param want_func  if true, treat a string as a function name, not an expression
int eval_expr_typval(const typval_T *expr, bool want_func, typval_T *argv, int argc,
                     typval_T *rettv)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_eval_expr_typval(expr, want_func, argv, argc, rettv);
}

/// Like eval_to_bool() but using a typval_T instead of a string.
/// Works for string, funcref and partial.
bool eval_expr_to_bool(const typval_T *expr, bool *error)
  FUNC_ATTR_NONNULL_ARG(1, 2)
{
  return rs_eval_expr_to_bool(expr, error);
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
  return rs_eval_to_string_skip(arg, eap, skip);
}

/// Skip over an expression at "*pp".
///
/// @return  FAIL for an error, OK otherwise.
int skip_expr(char **pp, evalarg_T *const evalarg)
{
  return rs_skip_expr(pp, evalarg);
}

/// Top level evaluation function, returning a string.
///
/// @param join_list  when true convert a List into a sequence of lines.
///
/// @return  pointer to allocated memory, or NULL for failure.
char *eval_to_string_eap(char *arg, const bool join_list, exarg_T *eap,
                         const bool use_simple_function)
{
  return rs_eval_to_string_eap(arg, join_list, eap, use_simple_function);
}

char *eval_to_string(char *arg, const bool join_list, const bool use_simple_function)
{
  return rs_eval_to_string(arg, join_list, use_simple_function);
}

/// Call eval_to_string() without using current local variables and using
/// textlock.
///
/// @param use_sandbox  when true, use the sandbox.
char *eval_to_string_safe(char *arg, const bool use_sandbox, const bool use_simple_function)
{
  return rs_eval_to_string_safe(arg, use_sandbox, use_simple_function);
}

/// Top level evaluation function, returning a number.
/// Evaluates "expr" silently.
///
/// @return  -1 for an error.
varnumber_T eval_to_number(char *expr, const bool use_simple_function)
{
  return rs_eval_to_number(expr, use_simple_function);
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
  return rs_eval_expr_ext(arg, eap, use_simple_function);
}

/// Call some Vim script function and return the result in "*rettv".
/// Uses argv[0] to argv[argc - 1] for the function arguments. argv[argc]
/// should have type VAR_UNKNOWN.
///
/// @return  OK or FAIL.
int call_vim_function(const char *func, int argc, typval_T *argv, typval_T *rettv)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_call_vim_function(func, argc, argv, rettv);
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
  return rs_call_func_retstr(func, argc, argv);
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
  return rs_call_func_retlist(func, argc, argv);
}

/// Evaluate 'foldexpr'.  Returns the foldlevel, and any character preceding
/// it in "*cp".  Doesn't give error messages.
int eval_foldexpr(win_T *wp, int *cp)
{
  return rs_eval_foldexpr(wp, cp);
}

/// Evaluate 'foldtext', returning an Array or a String (NULL_STRING on failure).
Object eval_foldtext(win_T *wp)
{
  Object retval;
  rs_eval_foldtext(wp, &retval);
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
  rs_clear_evalarg(evalarg, eap);
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
  return rs_may_call_simple_func(arg, rettv);
}

/// Thin wrapper for remaining C callers (eval_foldexpr, eval_foldtext).
/// Logic migrated to Rust eval0_simple_funccal_impl.
static int eval0_simple_funccal(char *arg, typval_T *rettv, exarg_T *eap, evalarg_T *const evalarg)
{
  int r = rs_may_call_simple_func(arg, rettv);
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
// call_func_rettv migrated to Rust (rs_call_func_rettv in eval_exec crate, eval.rs).

// eval_lambda migrated to Rust (rs_eval_lambda in eval_exec crate, eval.rs).

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
  return rs_eval_option(arg, rettv, evaluate);
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

/// Unreference a closure: decrement the reference count and free it when it
/// becomes zero.
void partial_unref(partial_T *pt)
{
  rs_partial_unref(pt);
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
// eval_env_var: deleted -- replaced by rs_eval_env_var (Rust, Phase 2).

/// Non-static wrapper for eval_env_var - calls Rust rs_eval_env_var.
int nvim_eval_env_var_wrapper(char **arg, typval_T *rettv, int evaluate)
{
  return rs_eval_env_var(arg, rettv, evaluate);
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
  return rs_save_tv_as_string(tv, len, endnl, crlf);
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
  if (rs_var2fpos(tv, dollar_lnum, ret_fnum, charcol, &pos)) {
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
  return rs_list2fpos(arg, posp, fnump, (int *)curswantp, charcol);
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
  rs_set_argv_var(argv, argc);
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
  return rs_handle_subscript(arg, rettv, evalarg, verbose);
}

void set_selfdict(typval_T *const rettv, dict_T *const selfdict)
{
  // Inlined into rs_handle_subscript in Rust.
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
  return rs_var_item_copy(conv, from, to, deep, copyID);
}

/// ":echo expr1 ..."    print each argument separated with a space, add a
///                      newline at the end.
/// ":echon expr1 ..."   print each argument plain.
void ex_echo(exarg_T *eap)
{
  rs_ex_echo(eap);
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
  rs_ex_execute(eap);
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
  int opt_idx_int = 0;
  const char *end = rs_find_option_var_end(arg, &opt_idx_int, opt_flags);
  *opt_idxp = (OptIndex)opt_idx_int;
  return end;
}

void var_set_global(const char *const name, typval_T vartv)
{
  rs_var_set_global(name, &vartv);
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
  rs_eval_fmt_source_name_line(buf, bufsize);
}

/// Gets the current user-input in prompt buffer `buf`, or NULL if buffer is not a prompt buffer.
char *prompt_get_input(buf_T *buf)
{
  return rs_prompt_get_input(buf);
}

/// Invokes the user-defined callback defined for the current prompt-buffer.
void prompt_invoke_callback(void)
{
  rs_prompt_invoke_callback();
}

/// @return  true when the interrupt callback was invoked.
bool invoke_prompt_interrupt(void)
{
  return rs_invoke_prompt_interrupt();
}


/// Convert any type to a string, never give an error.
/// When "quotes" is true add quotes to a string.
/// Returns an allocated string.
char *typval_tostring(typval_T *arg, bool quotes)
{
  return rs_typval_tostring(arg, quotes);
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

/// Thin wrapper for rs_call_func_rettv with selfdict=NULL - accessor for Rust rs_eval_method.
int nvim_call_func_rettv_wrapper(char **arg, evalarg_T *evalarg, typval_T *rettv, bool evaluate,
                                 typval_T *basetv, const char *lua_funcname)
{
  return rs_call_func_rettv(arg, evalarg, rettv, evaluate, NULL, basetv, lua_funcname);
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

// =============================================================================
// Phase 5 (handle_subscript): new C accessor/wrapper functions
// =============================================================================

// nvim_tv_get_dict already exists in eval/typval.c -- no duplicate needed here.

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

/// Thin wrapper for rs_call_func_rettv with selfdict - accessor for Rust rs_handle_subscript.
int nvim_call_func_rettv_with_selfdict(char **arg, evalarg_T *evalarg, typval_T *rettv,
                                       bool evaluate, dict_T *selfdict,
                                       const char *lua_funcname)
{
  return rs_call_func_rettv(arg, evalarg, rettv, evaluate, selfdict, NULL, lua_funcname);
}

/// Thin wrapper for rs_eval_lambda - accessor for Rust rs_handle_subscript.
int nvim_eval_lambda_wrapper(char **arg, typval_T *rettv, evalarg_T *evalarg, bool verbose)
{
  return rs_eval_lambda(arg, rettv, evalarg, verbose);
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

/// Get partial->pt_dict (for set_selfdict check) - accessor for Rust rs_handle_subscript.
dict_T *nvim_partial_get_pt_dict_handle(const partial_T *pt)
{
  return pt->pt_dict;
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

/// Allocate and fill an evalarg_T from eap on the heap.
/// Caller must call nvim_evalarg_clear_and_free after use.
evalarg_T *nvim_evalarg_alloc_from_eap(exarg_T *eap, bool skip)
{
  evalarg_T *ea = xcalloc(1, sizeof(evalarg_T));
  fill_evalarg_from_eap(ea, eap, skip);
  return ea;
}

/// Clear evalarg and free it.
void nvim_evalarg_clear_and_free(evalarg_T *ea, exarg_T *eap)
{
  clear_evalarg(ea, eap);
  xfree(ea);
}

/// Non-static wrapper for eval1_emsg -- now delegates to Rust rs_eval1_emsg.
int nvim_eval1_emsg_wrapper(char **arg, typval_T *rettv, exarg_T *eap)
{
  return rs_eval1_emsg(arg, rettv, eap);
}

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

// nvim_msg_start already defined in undo.c -- no duplicate needed here.

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

/// Heap-allocate a char garray (ga_init(&ga, 1, 80)).
garray_T *nvim_ga_alloc_execute(void)
{
  garray_T *ga = xcalloc(1, sizeof(garray_T));
  ga_init(ga, 1, 80);
  return ga;
}

/// Grow a garray - accessor for Rust.
void nvim_ga_grow_wrapper(garray_T *ga, int n)
{
  ga_grow(ga, n);
}

/// Check if ga_data is NULL (GA_EMPTY equivalent for data ptr) - accessor for Rust.
bool nvim_ga_data_is_null(const garray_T *ga)
{
  return ga->ga_data == NULL;
}

/// Append a space character to garray data[ga_len++] - accessor for Rust.
void nvim_ga_append_space(garray_T *ga)
{
  ((char *)(ga->ga_data))[ga->ga_len++] = ' ';
}

/// Copy len+1 bytes from str to garray data[ga_len], advance ga_len - accessor for Rust.
void nvim_ga_append_str_len(garray_T *ga, const char *str, int len)
{
  memcpy((char *)(ga->ga_data) + ga->ga_len, str, (size_t)len + 1);
  ga->ga_len += len;
}

/// Get ga_data pointer - accessor for Rust.
char *nvim_ga_get_data(const garray_T *ga)
{
  return (char *)ga->ga_data;
}

/// ga_clear and xfree the garray - accessor for Rust.
void nvim_ga_clear_and_free(garray_T *ga)
{
  ga_clear(ga);
  xfree(ga);
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

/// semsg with e_invexpr2 format - accessor for Rust.
void nvim_semsg_invexpr2(const char *p)
{
  semsg(_(e_invexpr2), p);
}

/// GA_EMPTY check (ga_len == 0) for ex_execute - accessor for Rust.
/// Uses nvim_ga_is_empty_execute to avoid conflict with fold_shim.c's nvim_ga_is_empty.
bool nvim_ga_is_empty_execute(garray_T *ga)
{
  return GA_EMPTY(ga);
}

/// Set did_emsg global - accessor for Rust.
// nvim_set_did_emsg already defined in message.c -- no duplicate needed here.

// =============================================================================
// Phase 2 eval_top accessors - eval_to_* and eval_expr_* family
// =============================================================================

/// Increment emsg_off - accessor for Rust eval_top.
void nvim_eval_emsg_off_inc(void)
{
  emsg_off++;
}

/// Decrement emsg_off - accessor for Rust eval_top.
void nvim_eval_emsg_off_dec(void)
{
  emsg_off--;
}

/// Increment sandbox - accessor for Rust eval_top.
void nvim_eval_sandbox_inc(void)
{
  sandbox++;
}

/// Decrement sandbox - accessor for Rust eval_top.
void nvim_eval_sandbox_dec(void)
{
  sandbox--;
}

/// Increment textlock - accessor for Rust eval_top.
void nvim_eval_textlock_inc(void)
{
  textlock++;
}

/// Decrement textlock - accessor for Rust eval_top.
void nvim_eval_textlock_dec(void)
{
  textlock--;
}

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
  return rs_may_call_simple_func(arg, rettv);
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

/// Get v_type from typval - accessor for eval_top.
int nvim_eval_tv_vtype(const typval_T *tv)
{
  return (int)tv->v_type;
}

/// Get vval.v_list from typval - accessor for eval_top.
list_T *nvim_eval_tv_vlist(const typval_T *tv)
{
  return tv->vval.v_list;
}

/// call_func wrapper for eval_expr_partial - accessor for Rust eval_top.
int nvim_eval_call_func_partial(const char *s, partial_T *partial,
                                typval_T *argv, int argc, typval_T *rettv)
{
  funcexe_T funcexe = FUNCEXE_INIT;
  funcexe.fe_evaluate = true;
  funcexe.fe_partial = partial;
  return call_func(s, -1, rettv, argc, argv, &funcexe);
}

/// call_func wrapper for eval_expr_func - accessor for Rust eval_top.
int nvim_eval_call_func_simple(const char *s, typval_T *argv, int argc, typval_T *rettv)
{
  funcexe_T funcexe = FUNCEXE_INIT;
  funcexe.fe_evaluate = true;
  return call_func(s, -1, rettv, argc, argv, &funcexe);
}

/// Get vval.v_partial from typval - accessor for eval_top.
partial_T *nvim_eval_tv_vpartial(const typval_T *tv)
{
  return tv->vval.v_partial;
}

/// Get vval.v_string from typval (read-only) - accessor for eval_top.
const char *nvim_eval_tv_vstring_ro(const typval_T *tv)
{
  return tv->vval.v_string;
}

/// tv_get_string wrapper - accessor for eval_top.
const char *nvim_eval_tv_get_string(const typval_T *tv)
{
  return tv_get_string(tv);
}

/// xstrdup wrapper - accessor for eval_top.
char *nvim_eval_xstrdup(const char *s)
{
  return xstrdup(s);
}

// ============================================================================
// Phase 3: C accessors for var2fpos / list2fpos (used by Rust indexing module)
// ============================================================================

/// Get curwin->w_cursor.lnum.
int32_t nvim_curwin_cursor_lnum(void)
{
  return curwin->w_cursor.lnum;
}

/// Get curwin->w_cursor.col.
int nvim_curwin_cursor_col(void)
{
  return curwin->w_cursor.col;
}

/// Get curwin->w_cursor.coladd.
int nvim_curwin_cursor_coladd(void)
{
  return curwin->w_cursor.coladd;
}

/// Get curwin->w_topline.
int32_t nvim_curwin_topline(void)
{
  return curwin->w_topline;
}

/// Get curwin->w_botline.
int32_t nvim_curwin_botline(void)
{
  return curwin->w_botline;
}

/// Get VIsual_active flag.
bool nvim_visual_active(void)
{
  return VIsual_active;
}

/// Get VIsual.lnum.
int32_t nvim_visual_lnum(void)
{
  return VIsual.lnum;
}

/// Get VIsual.col.
int nvim_visual_col(void)
{
  return VIsual.col;
}

/// Get VIsual.coladd.
int nvim_visual_coladd(void)
{
  return VIsual.coladd;
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

/// Construct funcexe_T with selfdict and call get_func_tv.
/// Used by rs_call_func_rettv to handle both selfdict and non-selfdict cases.
int nvim_call_func_tv_with_selfdict(char *name, int len, typval_T *rettv, char **arg,
                                    evalarg_T *evalarg, bool evaluate,
                                    partial_T *pt, dict_T *selfdict, typval_T *basetv,
                                    linenr_T lnum)
{
  funcexe_T funcexe = FUNCEXE_INIT;
  funcexe.fe_firstline = lnum;
  funcexe.fe_lastline = lnum;
  funcexe.fe_evaluate = evaluate;
  funcexe.fe_partial = pt;
  funcexe.fe_selfdict = selfdict;
  funcexe.fe_basetv = basetv;
  return get_func_tv(name, len, rettv, arg, evalarg, &funcexe);
}

/// Wrap get_lambda_tv for Rust rs_eval_lambda.
int nvim_get_lambda_tv(char **arg, typval_T *rettv, evalarg_T *evalarg)
{
  return get_lambda_tv(arg, rettv, evalarg);
}

/// Emit "E274: No white space allowed before parenthesis" error.
void nvim_emsg_e_nowhitespace(void)
{
  emsg(_(e_nowhitespace));
}

/// Emit "E15: Invalid expression: %s" with semsg.
void nvim_semsg_e_missingparen(const char *name)
{
  semsg(_(e_missingparen), name);
}

/// Get vval.v_string from a const typval - accessor for rs_call_func_rettv.
const char *nvim_tv_get_vstring_ro(const typval_T *tv)
{
  return tv->vval.v_string;
}

/// Emit "E117: Unknown function" / empty function name error.
void nvim_emsg_e_empty_function_name(void)
{
  emsg(_(e_empty_function_name));
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

/// Emit "E112: Option name missing: %s" semsg.
void nvim_semsg_e112_option_name_missing(const char *arg)
{
  semsg(_("E112: Option name missing: %s"), arg);
}

/// Emit "E113: Unknown option: %s" semsg.
void nvim_semsg_e113_unknown_option(const char *arg)
{
  semsg(_("E113: Unknown option: %s"), arg);
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

/// Get tv->vval.v_string (read-only const accessor).
const char *nvim_tv_get_vstring_const(const typval_T *tv)
{
  return tv->vval.v_string;
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

// nvim_dict_get_copyid already exists in eval/typval.c -- no duplicate needed here.

/// Get dict->dv_copydict.
dict_T *nvim_dict_get_copydict(const dict_T *dict)
{
  return dict->dv_copydict;
}

// nvim_dict_refcount_inc already exists above (line ~3863) -- no duplicate needed here.

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

/// Emit "E698: variable nested too deep for making a copy" error.
void nvim_emsg_nested_too_deep(void)
{
  emsg(_(e_variable_nested_too_deep_for_making_copy));
}

// =============================================================================
// Accessors for Phase 4 (eval_shim pass 4): save_tv_as_string
// =============================================================================

// nvim_buflist_findnr already exists in buffer.c -- no duplicate needed here.

/// Get tv->vval.v_number (integer field) - for VAR_NUMBER branch.
varnumber_T nvim_tv_get_vnumber(const typval_T *tv)
{
  return tv->vval.v_number;
}

/// Emit "E86: Buffer % does not exist" semsg.
void nvim_semsg_e_nobufnr(varnumber_T nr)
{
  semsg(_(e_nobufnr), nr);
}

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

/// Zero-init evalarg_T and set eval_flags based on skip - accessor for Rust.
void nvim_evalarg_init_skip(evalarg_T *evalarg, bool skip)
{
  *evalarg = (evalarg_T){ .eval_flags = skip ? 0 : EVAL_EVALUATE };
}

/// Check if eap is sourcing a script - accessor for Rust fill_evalarg_from_eap.
bool nvim_sourcing_a_script(exarg_T *eap)
{
  return sourcing_a_script(eap);
}

/// Copy eval_getline and eval_cookie from eap to evalarg - accessor for Rust.
void nvim_evalarg_copy_getline_from_eap(evalarg_T *evalarg, const exarg_T *eap)
{
  evalarg->eval_getline = eap->ea_getline;
  evalarg->eval_cookie = eap->cookie;
}

/// Get evalarg->eval_tofree - accessor for Rust clear_evalarg.
char *nvim_evalarg_get_tofree(evalarg_T *evalarg)
{
  return evalarg->eval_tofree;
}

/// Set evalarg->eval_tofree - accessor for Rust clear_evalarg.
void nvim_evalarg_set_tofree(evalarg_T *evalarg, char *val)
{
  evalarg->eval_tofree = val;
}

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

/// Construct funcexe_T and call call_func - accessor for rs_call_vim_function.
/// Avoids replicating funcexe_T struct layout in Rust.
int nvim_call_func_with_partial(const char *func, int len, typval_T *rettv,
                                int argc, typval_T *argv, partial_T *partial)
{
  funcexe_T funcexe = FUNCEXE_INIT;
  funcexe.fe_firstline = curwin->w_cursor.lnum;
  funcexe.fe_lastline = curwin->w_cursor.lnum;
  funcexe.fe_evaluate = true;
  funcexe.fe_partial = partial;
  return call_func(func, len, rettv, argc, argv, &funcexe);
}

/// Get VV_LUA partial - accessor for rs_call_vim_function.
partial_T *nvim_get_vv_lua_partial_p1(void)
{
  return get_vim_var_partial(VV_LUA);
}

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

/// snprintf wrapper for eval_fmt_source_name_line - accessor for Rust.
void nvim_snprintf_source_line(char *buf, size_t bufsize, const char *name, linenr_T lnum)
{
  snprintf(buf, bufsize, "%s:%" PRIdLINENR, name, lnum);
}

/// snprintf single char wrapper - for "?" fallback in eval_fmt_source_name_line.
void nvim_snprintf_question(char *buf, size_t bufsize)
{
  snprintf(buf, bufsize, "?");
}

/// Wrap tv_get_string - accessor for rs_call_func_retstr.
/// Named nvim_shim_tv_get_string to avoid conflict with nvim_tv_get_string in typval.c.
const char *nvim_shim_tv_get_string(const typval_T *tv)
{
  return tv_get_string(tv);
}

// nvim_xstrdup is defined in register.c - no duplicate needed here.

// =============================================================================
// Accessors for Phase 2 (eval_shim pass 5): prompt functions
// =============================================================================

/// Get buf->b_prompt_start.mark.lnum - accessor for rs_prompt_get_input.
linenr_T nvim_buf_get_prompt_start_lnum(buf_T *buf)
{
  return buf->b_prompt_start.mark.lnum;
}

/// Set curbuf->b_prompt_start.mark.lnum - accessor for rs_prompt_invoke_callback.
void nvim_curbuf_set_prompt_start_lnum(linenr_T lnum)
{
  curbuf->b_prompt_start.mark.lnum = lnum;
}

/// Get &curbuf->b_prompt_callback - accessor for rs_prompt_invoke_callback.
Callback *nvim_curbuf_get_prompt_callback(void)
{
  return &curbuf->b_prompt_callback;
}

/// Get &curbuf->b_prompt_interrupt - accessor for rs_invoke_prompt_interrupt.
Callback *nvim_curbuf_get_prompt_interrupt(void)
{
  return &curbuf->b_prompt_interrupt;
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

/// Get curbuf handle - accessor for rs_prompt_invoke_callback.
buf_T *nvim_get_curbuf_ptr(void)
{
  return curbuf;
}

/// Get curbuf->b_ml.ml_line_count - accessor for rs_prompt_invoke_callback.
linenr_T nvim_curbuf_get_ml_line_count_lnr(void)
{
  return (linenr_T)curbuf->b_ml.ml_line_count;
}

/// Call callback_call with a string argument - accessor for rs_prompt_invoke_callback.
/// Constructs a [VAR_STRING(user_input), VAR_UNKNOWN] argv array on the stack
/// and calls callback_call. user_input ownership is transferred (freed by tv_clear).
/// Returns whether the callback was called successfully.
bool nvim_curbuf_prompt_callback_call(char *user_input)
{
  typval_T rettv;
  typval_T argv[2];
  argv[0].v_type = VAR_STRING;
  argv[0].vval.v_string = user_input;
  argv[1].v_type = VAR_UNKNOWN;
  callback_call(&curbuf->b_prompt_callback, 1, argv, &rettv);
  tv_clear(&argv[0]);
  tv_clear(&rettv);
  return true;
}

/// Call callback_call with no arguments for b_prompt_interrupt.
/// Returns the result of callback_call (OK/FAIL as bool).
int nvim_curbuf_prompt_interrupt_call(void)
{
  typval_T rettv;
  typval_T argv[1];
  argv[0].v_type = VAR_UNKNOWN;
  int ret = callback_call(&curbuf->b_prompt_interrupt, 0, argv, &rettv);
  tv_clear(&rettv);
  return ret;
}

// =============================================================================
// Accessors for Phase 3 (eval_shim pass 5): eval_foldexpr and eval_foldtext
// =============================================================================

/// Check was_set_insecurely for 'foldexpr' - accessor for rs_eval_foldexpr.
bool nvim_win_was_set_insecurely_foldexpr(win_T *wp)
{
  return was_set_insecurely(wp, kOptFoldexpr, OPT_LOCAL);
}

/// Check was_set_insecurely for 'foldtext' - accessor for rs_eval_foldtext.
bool nvim_win_was_set_insecurely_foldtext(win_T *wp)
{
  return was_set_insecurely(wp, kOptFoldtext, OPT_LOCAL);
}

/// Return skipwhite(wp->w_p_fde) - accessor for rs_eval_foldexpr.
char *nvim_win_get_foldexpr(win_T *wp)
{
  return skipwhite(wp->w_p_fde);
}

/// Return wp->w_p_fdt - accessor for rs_eval_foldtext.
char *nvim_win_get_foldtext(win_T *wp)
{
  return wp->w_p_fdt;
}

/// Set current_sctx = wp->w_p_script_ctx[kWinOptFoldexpr] - accessor for rs_eval_foldexpr.
void nvim_win_set_current_sctx_foldexpr(win_T *wp)
{
  current_sctx = wp->w_p_script_ctx[kWinOptFoldexpr];
}

/// Heap-allocate a copy of current_sctx and return it opaquely - for rs_eval_foldexpr save.
sctx_T *nvim_save_current_sctx(void)
{
  sctx_T *saved = xmalloc(sizeof(sctx_T));
  *saved = current_sctx;
  return saved;
}

/// Restore current_sctx from an opaque pointer and free it.
void nvim_restore_current_sctx(sctx_T *saved)
{
  current_sctx = *saved;
  xfree(saved);
}

/// Write STRING_OBJ(NULL_STRING) into *out - for rs_eval_foldtext failure case.
void nvim_foldtext_make_nil_obj(Object *out)
{
  *out = STRING_OBJ(NULL_STRING);
}

/// Write STRING_OBJ(cstr_to_string(tv_get_string(tv))) into *out.
void nvim_foldtext_make_string_obj(typval_T *tv, Object *out)
{
  *out = STRING_OBJ(cstr_to_string(tv_get_string(tv)));
}

/// Write vim_to_object(tv, NULL, false) into *out.
void nvim_foldtext_make_array_obj(typval_T *tv, Object *out)
{
  *out = vim_to_object(tv, NULL, false);
}

