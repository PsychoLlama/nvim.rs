// User defined function support

#include <assert.h>
#include <ctype.h>
#include <inttypes.h>
#include <lauxlib.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/api/private/defs.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/debugger.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/encode.h"
#include "nvim/eval/funcs.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_eval_defs.h"
#include "nvim/ex_getln.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/getchar_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/insexpand.h"
#include "nvim/keycodes.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/path.h"
#include "nvim/profile.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/runtime.h"
#include "nvim/search.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/vim_defs.h"

extern int rs_ins_compl_active(void);
extern bool rs_eval_isnamec(int c);
extern bool rs_eval_isnamec1(int c);
extern int rs_get_id_len(const char **arg);
extern int rs_check_luafunc_name(const char *str, bool paren);
extern bool rs_is_luafunc(partial_T *partial);
extern const char *rs_find_name_end(const char *arg, const char **expr_start,
                                    const char **expr_end, int flags);
extern char *rs_partial_name(partial_T *pt);
extern bool rs_set_ref_in_ht(hashtab_T *ht, int copyID, list_stack_T **list_stack);
extern bool rs_set_ref_in_list_items(list_T *l, int copyID, ht_stack_T **ht_stack);
extern bool rs_set_ref_in_item(typval_T *tv, int copyID, ht_stack_T **ht_stack,
                               list_stack_T **list_stack);

// Phase 1: Function Listing (implemented in Rust userfunc/src/listing.rs)
extern int rs_cat_func_name(char *buf, size_t bufsize, ufunc_T *fp);
extern int rs_function_list_modified(int prev_ht_changed);
extern int rs_list_func_head(ufunc_T *fp, int indent, int force);
extern void rs_list_functions(void);
extern char *rs_list_functions_matching_pat(exarg_T *eap);
extern ufunc_T *rs_list_one_function(exarg_T *eap, const char *name, char *p);

// Phase 2: Function Name Translation (implemented in Rust userfunc/src/names.rs)
extern char *rs_fname_trans_sid(const char *name, char *fname_buf, char **tofree, int *error);
extern int rs_func_name_refcount(const char *name);
extern int rs_builtin_function(const char *name, int len);

// Phase 3: Defer Infrastructure (implemented in Rust userfunc/src/defer.rs)
extern void rs_handle_defer_one(funccall_T *funccal);
extern int rs_ex_defer_inner(char *name, char **arg, const partial_T *partial, evalarg_T *evalarg);

// Phase 4: Function Reference Counting (implemented in Rust userfunc/src/refcount.rs)
extern int rs_func_remove(ufunc_T *fp);
extern void rs_func_clear_items(ufunc_T *fp);
extern void rs_func_clear(ufunc_T *fp, int force);
extern void rs_func_free(ufunc_T *fp);
extern void rs_func_clear_free(ufunc_T *fp, int force);

// Phase 5: GC Support (implemented in Rust userfunc/src/gc.rs)
extern int rs_fc_referenced(const funccall_T *fc);
extern int rs_can_free_funccal(funccall_T *fc, int copyID);
extern int rs_set_ref_in_funccal(funccall_T *fc, int copyID);

// Phase 6: Scope Accessors + ex_return (implemented in Rust userfunc/src/scope.rs)

// Phase 7: Funccal Management + Helpers (implemented in Rust userfunc/src/funccal.rs)
extern void rs_free_funccal(funccall_T *fc);
extern void rs_free_funccal_contents(funccall_T *fc);
extern void rs_cleanup_function_call(funccall_T *fc);
extern void rs_funccal_unref(funccall_T *fc, ufunc_T *fp, int force);
extern void rs_user_func_error(int error, const char *name, int found_var);

#include "eval/userfunc.c.generated.h"

/// structure used as item in "fc_defer"
typedef struct {
  char *dr_name;  ///< function name, allocated
  typval_T dr_argvars[MAX_FUNC_ARGS + 1];
  int dr_argcount;
} defer_T;

static hashtab_T func_hashtab;

// Used by get_func_tv()
static garray_T funcargs = GA_EMPTY_INIT_VALUE;

// pointer to funccal for currently active function
static funccall_T *current_funccal = NULL;

// Pointer to list of previously used funccal, still around because some
// item in it is still being used.
static funccall_T *previous_funccal = NULL;

static const char *e_funcexts = N_("E122: Function %s already exists, add ! to replace it");
static const char *e_funcdict = N_("E717: Dictionary entry already exists");
static const char *e_funcref = N_("E718: Funcref required");
static const char *e_nofunc = N_("E130: Unknown function: %s");
static const char e_function_list_was_modified[]
  = N_("E454: Function list was modified");
static const char e_function_nesting_too_deep[]
  = N_("E1058: Function nesting too deep");
static const char e_no_white_space_allowed_before_str_str[]
  = N_("E1068: No white space allowed before '%s': %s");
static const char e_missing_heredoc_end_marker_str[]
  = N_("E1145: Missing heredoc end marker: %s");
static const char e_cannot_use_partial_with_dictionary_for_defer[]
  = N_("E1300: Cannot use a partial with dictionary for :defer");

void func_init(void) { hash_init(&func_hashtab); }

/// Return the function hash table
hashtab_T *func_tbl_get(void) { return &func_hashtab; }

// one_function_arg and get_function_args migrated to Rust (parsing.rs Phase 20)
extern int get_function_args(char **argp, char endchar, garray_T *newargs, int *varargs,
                             garray_T *default_args, bool skip);

/// Register function "fp" as using "current_funccal" as its scope.
static void register_closure(ufunc_T *fp)
{
  if (fp->uf_scoped == current_funccal) {
    // no change
    return;
  }
  rs_funccal_unref(fp->uf_scoped, fp, 0);
  fp->uf_scoped = current_funccal;
  current_funccal->fc_refcount++;
  ga_grow(&current_funccal->fc_ufuncs, 1);
  ((ufunc_T **)current_funccal->fc_ufuncs.ga_data)
  [current_funccal->fc_ufuncs.ga_len++] = fp;
}

static char lambda_name[8 + NUMBUFLEN];

/// @return  a name for a lambda.  Returned in static memory.
static String get_lambda_name(void)
{
  static int lambda_no = 0;

  int n = snprintf(lambda_name, sizeof(lambda_name), "<lambda>%d", ++lambda_no);

  return cbuf_as_string(lambda_name,
                        n < 1 ? 0 : (size_t)MIN(n, (int)sizeof(lambda_name) - 1));
}

/// Allocate a "ufunc_T" for a function called "name".
static ufunc_T *alloc_ufunc(const char *name, size_t namelen)
{
  size_t len = offsetof(ufunc_T, uf_name) + namelen + 1;
  ufunc_T *fp = xcalloc(1, len);
  xmemcpyz(fp->uf_name, name, namelen);
  fp->uf_namelen = namelen;

  if ((uint8_t)name[0] == K_SPECIAL) {
    len = namelen + 3;
    fp->uf_name_exp = xmalloc(len);
    snprintf(fp->uf_name_exp, len, "<SNR>%s", fp->uf_name + 3);
  }

  return fp;
}

/// Parse a lambda expression and get a Funcref from "*arg".
///
/// @return OK or FAIL.  Returns NOTDONE for dict or {expr}.
int get_lambda_tv(char **arg, typval_T *rettv, evalarg_T *evalarg)
{
  const bool evaluate = evalarg != NULL && (evalarg->eval_flags & EVAL_EVALUATE);
  garray_T newargs = GA_EMPTY_INIT_VALUE;
  garray_T *pnewargs;
  ufunc_T *fp = NULL;
  partial_T *pt = NULL;
  int varargs;
  bool *old_eval_lavars = eval_lavars_used;
  bool eval_lavars = false;
  char *tofree = NULL;

  // First, check if this is a lambda expression. "->" must exists.
  char *s = skipwhite(*arg + 1);
  int ret = get_function_args(&s, '-', NULL, NULL, NULL, true);
  if (ret == FAIL || *s != '>') {
    return NOTDONE;
  }

  // Parse the arguments again.
  if (evaluate) {
    pnewargs = &newargs;
  } else {
    pnewargs = NULL;
  }
  *arg = skipwhite(*arg + 1);
  ret = get_function_args(arg, '-', pnewargs, &varargs, NULL, false);
  if (ret == FAIL || **arg != '>') {
    goto errret;
  }

  // Set up a flag for checking local variables and arguments.
  if (evaluate) {
    eval_lavars_used = &eval_lavars;
  }

  // Get the start and the end of the expression.
  *arg = skipwhite((*arg) + 1);
  char *start = *arg;
  ret = skip_expr(arg, evalarg);
  char *end = *arg;
  if (ret == FAIL) {
    goto errret;
  }
  if (evalarg != NULL) {
    // avoid that the expression gets freed when another line break follows
    tofree = evalarg->eval_tofree;
    evalarg->eval_tofree = NULL;
  }

  *arg = skipwhite(*arg);
  if (**arg != '}') {
    semsg(_("E451: Expected }: %s"), *arg);
    goto errret;
  }
  (*arg)++;

  if (evaluate) {
    int flags = 0;
    garray_T newlines;

    String name = get_lambda_name();
    fp = alloc_ufunc(name.data, name.size);
    pt = xcalloc(1, sizeof(partial_T));

    ga_init(&newlines, (int)sizeof(char *), 1);
    ga_grow(&newlines, 1);

    // Add "return " before the expression.
    size_t len = (size_t)(7 + end - start + 1);
    char *p = xmalloc(len);
    ((char **)(newlines.ga_data))[newlines.ga_len++] = p;
    STRCPY(p, "return ");
    xmemcpyz(p + 7, start, (size_t)(end - start));
    if (strstr(p + 7, "a:") == NULL) {
      // No a: variables are used for sure.
      flags |= FC_NOARGS;
    }

    fp->uf_refcount = 1;
    hash_add(&func_hashtab, UF2HIKEY(fp));
    fp->uf_args = newargs;
    ga_init(&fp->uf_def_args, (int)sizeof(char *), 1);
    fp->uf_lines = newlines;
    if (current_funccal != NULL && eval_lavars) {
      flags |= FC_CLOSURE;
      register_closure(fp);
    } else {
      fp->uf_scoped = NULL;
    }

    if (prof_def_func()) {
      func_do_profile(fp);
    }
    if (sandbox) {
      flags |= FC_SANDBOX;
    }
    fp->uf_varargs = true;
    fp->uf_flags = flags;
    fp->uf_calls = 0;
    fp->uf_script_ctx = current_sctx;
    fp->uf_script_ctx.sc_lnum += SOURCING_LNUM - newlines.ga_len;

    pt->pt_func = fp;
    pt->pt_refcount = 1;
    rettv->vval.v_partial = pt;
    rettv->v_type = VAR_PARTIAL;
  }

  eval_lavars_used = old_eval_lavars;
  if (evalarg != NULL && evalarg->eval_tofree == NULL) {
    evalarg->eval_tofree = tofree;
  } else {
    xfree(tofree);
  }
  return OK;

errret:
  ga_clear_strings(&newargs);
  assert(fp == NULL);
  xfree(pt);
  if (evalarg != NULL && evalarg->eval_tofree == NULL) {
    evalarg->eval_tofree = tofree;
  } else {
    xfree(tofree);
  }
  eval_lavars_used = old_eval_lavars;
  return FAIL;
}

/// Return name of the function corresponding to `name`
///
/// If `name` points to variable that is either a function or partial then
/// corresponding function name is returned. Otherwise it returns `name` itself.
///
/// @param[in]  name  Function name to check.
/// @param[in,out]  lenp  Location where length of the returned name is stored.
///                       Must be set to the length of the `name` argument.
/// @param[out]  partialp  Location where partial will be stored if found
///                        function appears to be a partial. May be NULL if this
///                        is not needed.
/// @param[in]  no_autoload  If true, do not source autoload scripts if function
///                          was not found.
/// @param[out]  found_var  If not NULL and a variable was found set it to true.
///
/// @return name of the function.
// deref_func_name migrated to Rust (lookup.rs Phase 7)

/// Phase 7: C implementation shim for emsg_funcname (called from Rust).
// nvim_emsg_funcname_impl inlined into rs_emsg_funcname (Rust, Phase 13)

/// Give an error message with a function name.  Handle <SNR> things.
///
/// @param errmsg must be passed without translation (use N_() instead of _()).
/// @param name function name
/// Get function arguments at "*arg" and advance it.
/// Return them in "*argvars[MAX_FUNC_ARGS + 1]" and the count in "argcount".
/// On failure FAIL is returned but the "argvars[argcount]" are still set.
// get_func_arguments migrated to Rust (parsing.rs Phase 19)
extern int get_func_arguments(char **arg, evalarg_T *const evalarg, int partial_argc,
                              typval_T *argvars, int *argcount);

/// Call a function and put the result in "rettv".
///
/// @param name  name of the function
/// @param len  length of "name" or -1 to use strlen()
/// @param arg  argument, pointing to the '('
/// @param funcexe  various values
///
/// @return  OK or FAIL.
// get_func_tv migrated to Rust (Phase 23, funccal.rs)
extern int get_func_tv(const char *name, int len, typval_T *rettv, char **arg,
                       evalarg_T *const evalarg, funcexe_T *funcexe);

#define FLEN_FIXED 40

/// Check whether function name starts with <SID> or s:
///
/// @warning Only works for names previously checked by eval_fname_script(), if
///          it returned non-zero.
static inline bool eval_fname_sid(const char *const name)
  FUNC_ATTR_PURE FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_WARN_UNUSED_RESULT
  FUNC_ATTR_NONNULL_ALL
{
  return *name == 's' || TOUPPER_ASC(name[2]) == 'I';
}


// get_func_arity migrated to Rust (lookup.rs Phase 6)

/// Find a function by name, return pointer to it in ufuncs.
///
/// @return  NULL for unknown function.
ufunc_T *find_func(const char *name)
{
  hashitem_T *hi = hash_find(&func_hashtab, name);
  if (!HASHITEM_EMPTY(hi)) {
    return HI2UF(hi);
  }
  return NULL;
}


/// Add a number variable "name" to dict "dp" with value "nr".
static void add_nr_var(dict_T *dp, dictitem_T *v, char *name, varnumber_T nr)
{
  STRCPY(v->di_key, name);
  v->di_flags = DI_FLAGS_RO | DI_FLAGS_FIX;
  hash_add(&dp->dv_hashtab, v->di_key);
  v->di_tv.v_type = VAR_NUMBER;
  v->di_tv.v_lock = VAR_FIXED;
  v->di_tv.vval.v_number = nr;
}

/// Phase 7: C implementation shim for free_funccal (called from Rust).
// nvim_free_funccal_impl inlined into rs_free_funccal (Rust, Phase 13)


/// Phase 7: C implementation shim for free_funccal_contents (called from Rust).
void nvim_free_funccal_contents_impl(funccall_T *fc)
{
  // Free all l: variables.
  vars_clear(&fc->fc_l_vars.dv_hashtab);

  // Free all a: variables.
  vars_clear(&fc->fc_l_avars.dv_hashtab);

  // Free the a:000 variables.
  TV_LIST_ITER(&fc->fc_l_varlist, li, {
    tv_clear(TV_LIST_ITEM_TV(li));
  });

  rs_free_funccal(fc);
}

/// Phase 7: C implementation shim for cleanup_function_call (called from Rust).
void nvim_cleanup_function_call_impl(funccall_T *fc)
{
  bool may_free_fc = fc->fc_refcount <= 0;
  bool free_fc = true;

  current_funccal = fc->fc_caller;

  // Free all l: variables if not referred.
  if (may_free_fc && fc->fc_l_vars.dv_refcount == DO_NOT_FREE_CNT) {
    vars_clear(&fc->fc_l_vars.dv_hashtab);
  } else {
    free_fc = false;
  }

  // If the a:000 list and the l: and a: dicts are not referenced and
  // there is no closure using it, we can free the funccall_T and what's
  // in it.
  if (may_free_fc && fc->fc_l_avars.dv_refcount == DO_NOT_FREE_CNT) {
    vars_clear_ext(&fc->fc_l_avars.dv_hashtab, false);
  } else {
    free_fc = false;

    // Make a copy of the a: variables, since we didn't do that above.
    TV_DICT_ITER(&fc->fc_l_avars, di, {
      tv_copy(&di->di_tv, &di->di_tv);
    });
  }

  if (may_free_fc && fc->fc_l_varlist.lv_refcount   // NOLINT(runtime/deprecated)
      == DO_NOT_FREE_CNT) {
    fc->fc_l_varlist.lv_first = NULL;  // NOLINT(runtime/deprecated)
  } else {
    free_fc = false;

    // Make a copy of the a:000 items, since we didn't do that above.
    TV_LIST_ITER(&fc->fc_l_varlist, li, {
      tv_copy(TV_LIST_ITEM_TV(li), TV_LIST_ITEM_TV(li));
    });
  }

  if (free_fc) {
    rs_free_funccal(fc);
  } else {
    static int made_copy = 0;

    // "fc" is still in use.  This can happen when returning "a:000",
    // assigning "l:" to a global variable or defining a closure.
    // Link "fc" in the list for garbage collection later.
    fc->fc_caller = previous_funccal;
    previous_funccal = fc;

    if (want_garbage_collect) {
      // If garbage collector is ready, clear count.
      made_copy = 0;
    } else if (++made_copy >= (int)((4096 * 1024) / sizeof(*fc))) {
      // We have made a lot of copies, worth 4 Mbyte.  This can happen
      // when repetitively calling a function that creates a reference to
      // itself somehow.  Call the garbage collector soon to avoid using
      // too much memory.
      made_copy = 0;
      want_garbage_collect = true;
    }
  }
}


// nvim_funccal_unref_impl migrated to Rust (funccal.rs Phase 27).
// rs_funccal_unref now implements the logic directly.


/// Phase 4: C implementation shim for func_remove (called from Rust).
int nvim_func_remove_impl(ufunc_T *fp)
{
  hashitem_T *hi = hash_find(&func_hashtab, UF2HIKEY(fp));
  if (HASHITEM_EMPTY(hi)) {
    return false;
  }
  hash_remove(&func_hashtab, hi);
  return true;
}

/// Phase 4: C implementation shim for func_clear_items.
// nvim_func_clear_items_impl inlined into rs_func_clear_items (Rust, Phase 14)

// nvim_func_clear_impl inlined into rs_func_clear (Rust, Phase 9)

// nvim_func_free_impl inlined into rs_func_free (Rust, Phase 9)

// nvim_func_clear_free_impl inlined into rs_func_clear_free (Rust, Phase 8)

/// Phase 7: C implementation shim for create_funccal (called from Rust).
// nvim_create_funccal_impl inlined into rs_create_funccal (Rust, Phase 13)

// nvim_remove_funccal_impl inlined into rs_remove_funccal (Rust, Phase 8)

/// Call a user function
///
/// @param fp  Function to call.
/// @param[in] argcount  Number of arguments.
/// @param argvars  Arguments.
/// @param[out] rettv  Return value.
/// @param[in] firstline  First line of range.
/// @param[in] lastline  Last line of range.
/// @param selfdict  Dict for "self" for dictionary functions.
void call_user_func(ufunc_T *fp, int argcount, typval_T *argvars, typval_T *rettv,
                    linenr_T firstline, linenr_T lastline, dict_T *selfdict)
  FUNC_ATTR_NONNULL_ARG(1, 3, 4)
{
  bool using_sandbox = false;
  static int depth = 0;
  dictitem_T *v;
  int fixvar_idx = 0;           // index in fc_fixvar[]
  bool islambda = false;
  char numbuf[NUMBUFLEN];
  char *name;
  size_t namelen;
  typval_T *tv_to_free[MAX_FUNC_ARGS];
  int tv_to_free_len = 0;
  proftime_T wait_start;
  proftime_T call_start;
  bool started_profiling = false;
  bool did_save_redo = false;
  save_redo_T save_redo;

  // If depth of calling is getting too high, don't execute the function
  if (depth >= p_mfd) {
    emsg(_("E132: Function call depth is higher than 'maxfuncdepth'"));
    rettv->v_type = VAR_NUMBER;
    rettv->vval.v_number = -1;
    return;
  }
  depth++;
  // Save search patterns and redo buffer.
  save_search_patterns();
  if (!rs_ins_compl_active()) {
    saveRedobuff(&save_redo);
    did_save_redo = true;
  }
  fp->uf_calls++;
  // check for CTRL-C hit
  line_breakcheck();
  // prepare the funccall_T structure
  funccall_T *fc = create_funccal(fp, rettv);
  fc->fc_level = ex_nesting_level;
  // Check if this function has a breakpoint.
  fc->fc_breakpoint = dbg_find_breakpoint(false, fp->uf_name, 0);
  fc->fc_dbg_tick = debug_tick;
  // Set up fields for closure.
  ga_init(&fc->fc_ufuncs, sizeof(ufunc_T *), 1);

  if (strncmp(fp->uf_name, "<lambda>", 8) == 0) {
    islambda = true;
  }

  // Note about using fc->fc_fixvar[]: This is an array of FIXVAR_CNT variables
  // with names up to VAR_SHORT_LEN long.  This avoids having to alloc/free
  // each argument variable and saves a lot of time.
  //
  // Init l: variables.
  init_var_dict(&fc->fc_l_vars, &fc->fc_l_vars_var, VAR_DEF_SCOPE);
  if (selfdict != NULL) {
    // Set l:self to "selfdict".  Use "name" to avoid a warning from
    // some compiler that checks the destination size.
    v = (dictitem_T *)&fc->fc_fixvar[fixvar_idx++];
    name = (char *)v->di_key;
    STRCPY(name, "self");
    v->di_flags = DI_FLAGS_RO | DI_FLAGS_FIX;
    hash_add(&fc->fc_l_vars.dv_hashtab, v->di_key);
    v->di_tv.v_type = VAR_DICT;
    v->di_tv.v_lock = VAR_UNLOCKED;
    v->di_tv.vval.v_dict = selfdict;
    selfdict->dv_refcount++;
  }

  // Init a: variables, unless none found (in lambda).
  // Set a:0 to "argcount" less number of named arguments, if >= 0.
  // Set a:000 to a list with room for the "..." arguments.
  init_var_dict(&fc->fc_l_avars, &fc->fc_l_avars_var, VAR_SCOPE);
  if ((fp->uf_flags & FC_NOARGS) == 0) {
    add_nr_var(&fc->fc_l_avars, (dictitem_T *)&fc->fc_fixvar[fixvar_idx++], "0",
               (varnumber_T)(argcount >= fp->uf_args.ga_len
                             ? argcount - fp->uf_args.ga_len : 0));
  }
  fc->fc_l_avars.dv_lock = VAR_FIXED;
  if ((fp->uf_flags & FC_NOARGS) == 0) {
    // Use "name" to avoid a warning from some compiler that checks the
    // destination size.
    v = (dictitem_T *)&fc->fc_fixvar[fixvar_idx++];
    name = (char *)v->di_key;
    STRCPY(name, "000");
    v->di_flags = DI_FLAGS_RO | DI_FLAGS_FIX;
    hash_add(&fc->fc_l_avars.dv_hashtab, v->di_key);
    v->di_tv.v_type = VAR_LIST;
    v->di_tv.v_lock = VAR_FIXED;
    v->di_tv.vval.v_list = &fc->fc_l_varlist;
  }
  tv_list_init_static(&fc->fc_l_varlist);
  tv_list_set_lock(&fc->fc_l_varlist, VAR_FIXED);

  // Set a:firstline to "firstline" and a:lastline to "lastline".
  // Set a:name to named arguments.
  // Set a:N to the "..." arguments.
  // Skipped when no a: variables used (in lambda).
  if ((fp->uf_flags & FC_NOARGS) == 0) {
    add_nr_var(&fc->fc_l_avars, (dictitem_T *)&fc->fc_fixvar[fixvar_idx++],
               "firstline", (varnumber_T)firstline);
    add_nr_var(&fc->fc_l_avars, (dictitem_T *)&fc->fc_fixvar[fixvar_idx++],
               "lastline", (varnumber_T)lastline);
  }
  bool default_arg_err = false;
  for (int i = 0; i < argcount || i < fp->uf_args.ga_len; i++) {
    bool addlocal = false;
    bool isdefault = false;
    typval_T def_rettv;

    int ai = i - fp->uf_args.ga_len;
    if (ai < 0) {
      // named argument a:name
      name = FUNCARG(fp, i);
      if (islambda) {
        addlocal = true;
      }

      // evaluate named argument default expression
      isdefault = ai + fp->uf_def_args.ga_len >= 0 && i >= argcount;
      if (isdefault) {
        char *default_expr = NULL;
        def_rettv.v_type = VAR_NUMBER;
        def_rettv.vval.v_number = -1;

        default_expr = ((char **)(fp->uf_def_args.ga_data))
                       [ai + fp->uf_def_args.ga_len];
        if (eval1(&default_expr, &def_rettv, &EVALARG_EVALUATE) == FAIL) {
          default_arg_err = true;
          break;
        }
      }

      namelen = strlen(name);
    } else {
      if ((fp->uf_flags & FC_NOARGS) != 0) {
        // Bail out if no a: arguments used (in lambda).
        break;
      }
      // "..." argument a:1, a:2, etc.
      namelen = (size_t)snprintf(numbuf, sizeof(numbuf), "%d", ai + 1);
      name = numbuf;
    }
    if (fixvar_idx < FIXVAR_CNT && namelen <= VAR_SHORT_LEN) {
      v = (dictitem_T *)&fc->fc_fixvar[fixvar_idx++];
      v->di_flags = DI_FLAGS_RO | DI_FLAGS_FIX;
      STRCPY(v->di_key, name);
    } else {
      v = tv_dict_item_alloc_len(name, namelen);
      v->di_flags |= DI_FLAGS_RO | DI_FLAGS_FIX;
    }

    // Note: the values are copied directly to avoid alloc/free.
    // "argvars" must have VAR_FIXED for v_lock.
    v->di_tv = isdefault ? def_rettv : argvars[i];
    v->di_tv.v_lock = VAR_FIXED;

    if (isdefault) {
      // Need to free this later, no matter where it's stored.
      tv_to_free[tv_to_free_len++] = &v->di_tv;
    }

    if (addlocal) {
      // Named arguments can be accessed without the "a:" prefix in lambda
      // expressions. Add to the l: dict.
      tv_copy(&v->di_tv, &v->di_tv);
      hash_add(&fc->fc_l_vars.dv_hashtab, v->di_key);
    } else {
      hash_add(&fc->fc_l_avars.dv_hashtab, v->di_key);
    }

    if (ai >= 0 && ai < MAX_FUNC_ARGS) {
      listitem_T *li = &fc->fc_l_listitems[ai];

      *TV_LIST_ITEM_TV(li) = argvars[i];
      TV_LIST_ITEM_TV(li)->v_lock = VAR_FIXED;
      tv_list_append(&fc->fc_l_varlist, li);
    }
  }

  // Don't redraw while executing the function.
  RedrawingDisabled++;

  if (fp->uf_flags & FC_SANDBOX) {
    using_sandbox = true;
    sandbox++;
  }

  estack_push_ufunc(fp, 1);
  if (p_verbose >= 12) {
    no_wait_return++;
    verbose_enter_scroll();

    smsg(0, _("calling %s"), SOURCING_NAME);
    if (p_verbose >= 14) {
      msg_puts("(");
      for (int i = 0; i < argcount; i++) {
        if (i > 0) {
          msg_puts(", ");
        }
        if (argvars[i].v_type == VAR_NUMBER) {
          msg_outnum((int)argvars[i].vval.v_number);
        } else {
          // Do not want errors such as E724 here.
          emsg_off++;
          char *tofree = encode_tv2string(&argvars[i], NULL);
          emsg_off--;
          if (tofree != NULL) {
            char *s = tofree;
            char buf[MSG_BUF_LEN];
            if (vim_strsize(s) > MSG_BUF_CLEN) {
              trunc_string(s, buf, MSG_BUF_CLEN, sizeof(buf));
              s = buf;
            }
            msg_puts(s);
            xfree(tofree);
          }
        }
      }
      msg_puts(")");
    }
    msg_puts("\n");  // don't overwrite this either

    verbose_leave_scroll();
    no_wait_return--;
  }

  const bool do_profiling_yes = do_profiling == PROF_YES;

  bool func_not_yet_profiling_but_should =
    do_profiling_yes
    && !fp->uf_profiling && has_profiling(false, fp->uf_name, NULL);

  if (func_not_yet_profiling_but_should) {
    started_profiling = true;
    func_do_profile(fp);
  }

  bool func_or_func_caller_profiling =
    do_profiling_yes
    && (fp->uf_profiling
        || (fc->fc_caller != NULL && fc->fc_caller->fc_func->uf_profiling));

  if (func_or_func_caller_profiling) {
    fp->uf_tm_count++;
    call_start = profile_start();
    fp->uf_tm_children = profile_zero();
  }

  if (do_profiling_yes) {
    script_prof_save(&wait_start);
  }

  const sctx_T save_current_sctx = current_sctx;
  current_sctx = fp->uf_script_ctx;
  int save_did_emsg = did_emsg;
  did_emsg = false;

  if (default_arg_err && (fp->uf_flags & FC_ABORT || trylevel > 0)) {
    did_emsg = true;
  } else if (islambda) {
    char *p = *(char **)fp->uf_lines.ga_data + 7;

    // A Lambda always has the command "return {expr}".  It is much faster
    // to evaluate {expr} directly.
    ex_nesting_level++;
    eval1(&p, rettv, &EVALARG_EVALUATE);
    ex_nesting_level--;
  } else {
    // call do_cmdline() to execute the lines
    do_cmdline(NULL, get_func_line, (void *)fc,
               DOCMD_NOWAIT|DOCMD_VERBOSE|DOCMD_REPEAT);
  }

  // Invoke functions added with ":defer".
  rs_handle_defer_one(current_funccal);

  RedrawingDisabled--;

  // when the function was aborted because of an error, return -1
  if ((did_emsg
       && (fp->uf_flags & FC_ABORT)) || rettv->v_type == VAR_UNKNOWN) {
    tv_clear(rettv);
    rettv->v_type = VAR_NUMBER;
    rettv->vval.v_number = -1;
  }

  if (func_or_func_caller_profiling) {
    call_start = profile_end(call_start);
    call_start = profile_sub_wait(wait_start, call_start);
    fp->uf_tm_total = profile_add(fp->uf_tm_total, call_start);
    fp->uf_tm_self = profile_self(fp->uf_tm_self, call_start,
                                  fp->uf_tm_children);
    if (fc->fc_caller != NULL && fc->fc_caller->fc_func->uf_profiling) {
      fc->fc_caller->fc_func->uf_tm_children =
        profile_add(fc->fc_caller->fc_func->uf_tm_children, call_start);
      fc->fc_caller->fc_func->uf_tml_children =
        profile_add(fc->fc_caller->fc_func->uf_tml_children, call_start);
    }
    if (started_profiling) {
      // make a ":profdel func" stop profiling the function
      fp->uf_profiling = false;
    }
  }

  // when being verbose, mention the return value
  if (p_verbose >= 12) {
    no_wait_return++;
    verbose_enter_scroll();

    if (aborting()) {
      smsg(0, _("%s aborted"), SOURCING_NAME);
    } else if (fc->fc_rettv->v_type == VAR_NUMBER) {
      smsg(0, _("%s returning #%" PRId64 ""),
           SOURCING_NAME, (int64_t)fc->fc_rettv->vval.v_number);
    } else {
      char buf[MSG_BUF_LEN];

      // The value may be very long.  Skip the middle part, so that we
      // have some idea how it starts and ends. smsg() would always
      // truncate it at the end. Don't want errors such as E724 here.
      emsg_off++;
      char *s = encode_tv2string(fc->fc_rettv, NULL);
      char *tofree = s;
      emsg_off--;
      if (s != NULL) {
        if (vim_strsize(s) > MSG_BUF_CLEN) {
          trunc_string(s, buf, MSG_BUF_CLEN, MSG_BUF_LEN);
          s = buf;
        }
        smsg(0, _("%s returning %s"), SOURCING_NAME, s);
        xfree(tofree);
      }
    }
    msg_puts("\n");  // don't overwrite this either

    verbose_leave_scroll();
    no_wait_return--;
  }

  estack_pop();
  current_sctx = save_current_sctx;
  if (do_profiling_yes) {
    script_prof_restore(&wait_start);
  }
  if (using_sandbox) {
    sandbox--;
  }

  if (p_verbose >= 12 && SOURCING_NAME != NULL) {
    no_wait_return++;
    verbose_enter_scroll();

    smsg(0, _("continuing in %s"), SOURCING_NAME);
    msg_puts("\n");  // don't overwrite this either

    verbose_leave_scroll();
    no_wait_return--;
  }

  did_emsg |= save_did_emsg;
  depth--;
  for (int i = 0; i < tv_to_free_len; i++) {
    tv_clear(tv_to_free[i]);
  }
  rs_cleanup_function_call(fc);

  if (--fp->uf_calls <= 0 && fp->uf_refcount <= 0) {
    // Function was unreferenced while being used, free it now.
    rs_func_clear_free(fp, 0);
  }
  // restore search patterns and redo buffer
  if (did_save_redo) {
    restoreRedobuff(&save_redo);
  }
  restore_search_patterns();
}

/// There are two kinds of function names:
/// 1. ordinary names, function defined with :function
/// 2. numbered functions and lambdas
/// For the first we only count the name stored in func_hashtab as a reference,
/// using function() does not count as a reference, because the function is
/// looked up by name.

// check_user_func_argcount migrated to Rust (lookup.rs Phase 18)
extern int check_user_func_argcount(ufunc_T *fp, int argcount);

/// Call a user function after checking the arguments.
// call_user_func_check migrated to Rust (funccal.rs Phase 21)
extern int call_user_func_check(ufunc_T *fp, int argcount, typval_T *argvars, typval_T *rettv,
                                funcexe_T *funcexe, dict_T *selfdict);

static funccal_entry_T *funccal_stack = NULL;

/// Phase 7: C implementation shim for save_funccal (called from Rust).
// nvim_save_funccal_impl inlined into rs_save_funccal (Rust, Phase 13)

// nvim_restore_funccal_impl inlined into rs_restore_funccal (Rust, Phase 13)

funccall_T *get_current_funccal(void) { return current_funccal; }

void set_current_funccal(funccall_T *fc) { current_funccal = fc; }

#if defined(EXITFREE)
void free_all_functions(void)
{
  hashitem_T *hi;
  ufunc_T *fp;
  uint64_t skipped = 0;
  uint64_t todo = 1;
  int changed;

  // Clean up the current_funccal chain and the funccal stack.
  while (current_funccal != NULL) {
    tv_clear(current_funccal->fc_rettv);
    rs_cleanup_function_call(current_funccal);
    if (current_funccal == NULL && funccal_stack != NULL) {
      restore_funccal();
    }
  }

  // First clear what the functions contain. Since this may lower the
  // reference count of a function, it may also free a function and change
  // the hash table. Restart if that happens.
  while (todo > 0) {
    todo = func_hashtab.ht_used;
    for (hi = func_hashtab.ht_array; todo > 0; hi++) {
      if (!HASHITEM_EMPTY(hi)) {
        // Only free functions that are not refcounted, those are
        // supposed to be freed when no longer referenced.
        fp = HI2UF(hi);
        if (rs_func_name_refcount(fp->uf_name)) {
          skipped++;
        } else {
          changed = func_hashtab.ht_changed;
          rs_func_clear(fp, 1);
          if (changed != func_hashtab.ht_changed) {
            skipped = 0;
            break;
          }
        }
        todo--;
      }
    }
  }

  // Now actually free the functions. Need to start all over every time,
  // because func_free() may change the hash table.
  skipped = 0;
  while (func_hashtab.ht_used > skipped) {
    todo = func_hashtab.ht_used;
    for (hi = func_hashtab.ht_array; todo > 0; hi++) {
      if (!HASHITEM_EMPTY(hi)) {
        todo--;
        // Only free functions that are not refcounted, those are
        // supposed to be freed when no longer referenced.
        fp = HI2UF(hi);
        if (rs_func_name_refcount(fp->uf_name)) {
          skipped++;
        } else {
          rs_func_free(fp);
          skipped = 0;
          break;
        }
      }
    }
  }
  if (skipped == 0) {
    hash_clear(&func_hashtab);
  }
}

#endif


int func_call(char *name, typval_T *args, partial_T *partial, dict_T *selfdict, typval_T *rettv)
{
  typval_T argv[MAX_FUNC_ARGS + 1];
  int argc = 0;
  int r = 0;

  TV_LIST_ITER(args->vval.v_list, item, {
    if (argc == MAX_FUNC_ARGS - (partial == NULL ? 0 : partial->pt_argc)) {
      emsg(_("E699: Too many arguments"));
      goto func_call_skip_call;
    }
    // Make a copy of each argument.  This is needed to be able to set
    // v_lock to VAR_FIXED in the copy without changing the original list.
    tv_copy(TV_LIST_ITEM_TV(item), &argv[argc++]);
  });

  funcexe_T funcexe = FUNCEXE_INIT;
  funcexe.fe_firstline = curwin->w_cursor.lnum;
  funcexe.fe_lastline = curwin->w_cursor.lnum;
  funcexe.fe_evaluate = true;
  funcexe.fe_partial = partial;
  funcexe.fe_selfdict = selfdict;
  r = call_func(name, -1, rettv, argc, argv, &funcexe);

func_call_skip_call:
  // Free the arguments.
  while (argc > 0) {
    tv_clear(&argv[--argc]);
  }

  return r;
}

// callback_call_retnr migrated to Rust (funccal.rs Phase 15)

/// Phase 7: C implementation shim for user_func_error (called from Rust).
// nvim_user_func_error_impl inlined into rs_user_func_error (Rust, Phase 14)


// argv_add_base migrated to Rust (lookup.rs Phase 18)
extern void argv_add_base(typval_T *const basetv, typval_T **const argvars, int *const argcount,
                          typval_T *const new_argvars, int *const argv_base);

/// Call a function with its resolved parameters
///
/// @param funcname  name of the function
/// @param len  length of "name" or -1 to use strlen()
/// @param rettv  [out] value goes here
/// @param argcount_in  number of "argvars"
/// @param argvars_in  vars for arguments, must have "argcount" PLUS ONE elements!
/// @param funcexe  more arguments
///
/// @return FAIL if function cannot be called, else OK (even if an error
///         occurred while executing the function! Set `msg_list` to capture
///         the error, see do_cmdline()).
// call_func migrated to Rust (Phase 22, funccal.rs)
extern int call_func(const char *funcname, int len, typval_T *rettv, int argcount_in,
                     typval_T *argvars_in, funcexe_T *funcexe);

// call_simple_luafunc and call_simple_func migrated to Rust (Phase 16, funccal.rs)

/// Get a function name, translating "<SID>" and "<SNR>".
/// Also handles a Funcref in a List or Dict.
/// flags:
/// TFN_INT:         internal function name OK
/// TFN_QUIET:       be quiet
/// TFN_NO_AUTOLOAD: do not use script autoloading
/// TFN_NO_DEREF:    do not dereference a Funcref
/// Advances "pp" to just after the function name (if no error).
///
/// @param skip  only find the end, don't evaluate
/// @param fdp  return: info about dictionary used
/// @param partial  return: partial of a FuncRef
///
/// @return the function name in allocated memory, or NULL for failure.
char *trans_function_name(char **pp, bool skip, int flags, funcdict_T *fdp, partial_T **partial)
  FUNC_ATTR_NONNULL_ARG(1)
{
  char *name = NULL;
  int len;
  lval_T lv;

  if (fdp != NULL) {
    CLEAR_POINTER(fdp);
  }
  const char *start = *pp;

  // Check for hard coded <SNR>: already translated function ID (from a user
  // command).
  if ((uint8_t)(*pp)[0] == K_SPECIAL && (uint8_t)(*pp)[1] == KS_EXTRA && (*pp)[2] == KE_SNR) {
    *pp += 3;
    len = rs_get_id_len((const char **)pp) + 3;
    return xmemdupz(start, (size_t)len);
  }

  // A name starting with "<SID>" or "<SNR>" is local to a script.  But
  // don't skip over "s:", get_lval() needs it for "s:dict.func".
  int lead = eval_fname_script(start);
  if (lead > 2) {
    start += lead;
  }

  // Note that TFN_ flags use the same values as GLV_ flags.
  const char *end = get_lval((char *)start, NULL, &lv, false, skip, flags | GLV_READ_ONLY,
                             lead > 2 ? 0 : FNE_CHECK_START);
  if (end == start) {
    if (!skip) {
      emsg(_("E129: Function name required"));
    }
    goto theend;
  }
  if (end == NULL || (lv.ll_tv != NULL && (lead > 2 || lv.ll_range))) {
    // Report an invalid expression in braces, unless the expression
    // evaluation has been cancelled due to an aborting error, an
    // interrupt, or an exception.
    if (!aborting()) {
      if (end != NULL) {
        semsg(_(e_invarg2), start);
      }
    } else {
      *pp = (char *)rs_find_name_end(start, NULL, NULL, FNE_INCL_BR);
    }
    goto theend;
  }

  if (lv.ll_tv != NULL) {
    if (fdp != NULL) {
      fdp->fd_dict = lv.ll_dict;
      fdp->fd_newkey = lv.ll_newkey;
      lv.ll_newkey = NULL;
      fdp->fd_di = lv.ll_di;
    }
    if (lv.ll_tv->v_type == VAR_FUNC && lv.ll_tv->vval.v_string != NULL) {
      name = xstrdup(lv.ll_tv->vval.v_string);
      *pp = (char *)end;
    } else if (lv.ll_tv->v_type == VAR_PARTIAL
               && lv.ll_tv->vval.v_partial != NULL) {
      if (rs_is_luafunc(lv.ll_tv->vval.v_partial) && *end == '.') {
        len = rs_check_luafunc_name(end + 1, true);
        if (len == 0) {
          semsg(e_invexpr2, "v:lua");
          goto theend;
        }
        name = xmallocz((size_t)len);
        memcpy(name, end + 1, (size_t)len);
        *pp = (char *)end + 1 + len;
      } else {
        name = xstrdup(rs_partial_name(lv.ll_tv->vval.v_partial));
        *pp = (char *)end;
      }
      if (partial != NULL) {
        *partial = lv.ll_tv->vval.v_partial;
      }
    } else {
      if (!skip && !(flags & TFN_QUIET) && (fdp == NULL
                                            || lv.ll_dict == NULL
                                            || fdp->fd_newkey == NULL)) {
        emsg(_(e_funcref));
      } else {
        *pp = (char *)end;
      }
      name = NULL;
    }
    goto theend;
  }

  if (lv.ll_name == NULL) {
    // Error found, but continue after the function name.
    *pp = (char *)end;
    goto theend;
  }

  // Check if the name is a Funcref.  If so, use the value.
  if (lv.ll_exp_name != NULL) {
    len = (int)strlen(lv.ll_exp_name);
    name = deref_func_name(lv.ll_exp_name, &len, partial, flags & TFN_NO_AUTOLOAD, NULL);
    if (name == lv.ll_exp_name) {
      name = NULL;
    }
  } else if (!(flags & TFN_NO_DEREF)) {
    len = (int)(end - *pp);
    name = deref_func_name(*pp, &len, partial, flags & TFN_NO_AUTOLOAD, NULL);
    if (name == *pp) {
      name = NULL;
    }
  }
  if (name != NULL) {
    name = xstrdup(name);
    *pp = (char *)end;
    if (strncmp(name, "<SNR>", 5) == 0) {
      // Change "<SNR>" to the byte sequence.
      name[0] = (char)K_SPECIAL;
      name[1] = (char)KS_EXTRA;
      name[2] = KE_SNR;
      memmove(name + 3, name + 5, strlen(name + 5) + 1);
    }
    goto theend;
  }

  if (lv.ll_exp_name != NULL) {
    len = (int)strlen(lv.ll_exp_name);
    if (lead <= 2 && lv.ll_name == lv.ll_exp_name
        && lv.ll_name_len >= 2 && memcmp(lv.ll_name, "s:", 2) == 0) {
      // When there was "s:" already or the name expanded to get a
      // leading "s:" then remove it.
      lv.ll_name += 2;
      lv.ll_name_len -= 2;
      len -= 2;
      lead = 2;
    }
  } else {
    // Skip over "s:" and "g:".
    if (lead == 2 || (lv.ll_name[0] == 'g' && lv.ll_name[1] == ':')) {
      lv.ll_name += 2;
      lv.ll_name_len -= 2;
    }
    len = (int)(end - lv.ll_name);
  }

  size_t sid_buflen = 0;
  char sid_buf[20];

  // Copy the function name to allocated memory.
  // Accept <SID>name() inside a script, translate into <SNR>123_name().
  // Accept <SNR>123_name() outside a script.
  if (skip) {
    lead = 0;  // do nothing
  } else if (lead > 0) {
    lead = 3;
    if ((lv.ll_exp_name != NULL && eval_fname_sid(lv.ll_exp_name))
        || eval_fname_sid(*pp)) {
      // It's "s:" or "<SID>".
      if (current_sctx.sc_sid <= 0) {
        emsg(_(e_usingsid));
        goto theend;
      }
      sid_buflen = (size_t)snprintf(sid_buf, sizeof(sid_buf), "%" PRIdSCID "_",
                                    current_sctx.sc_sid);
      lead += (int)sid_buflen;
    }
  } else if (!(flags & TFN_INT) && rs_builtin_function(lv.ll_name, (int)lv.ll_name_len)) {
    semsg(_("E128: Function name must start with a capital or \"s:\": %s"),
          start);
    goto theend;
  }

  if (!skip && !(flags & TFN_QUIET) && !(flags & TFN_NO_DEREF)) {
    char *cp = xmemrchr(lv.ll_name, ':', lv.ll_name_len);

    if (cp != NULL && cp < end) {
      semsg(_("E884: Function name cannot contain a colon: %s"), start);
      goto theend;
    }
  }

  name = xmalloc((size_t)len + (size_t)lead + 1);
  if (!skip && lead > 0) {
    name[0] = (char)K_SPECIAL;
    name[1] = (char)KS_EXTRA;
    name[2] = KE_SNR;
    if (sid_buflen > 0) {  // If it's "<SID>"
      memcpy(name + 3, sid_buf, sid_buflen);
    }
  }
  memmove(name + lead, lv.ll_name, (size_t)len);
  name[lead + len] = NUL;
  *pp = (char *)end;

theend:
  clear_lval(&lv);
  return name;
}

/// If the "funcname" starts with "s:" or "<SID>", expands it to the current
/// script ID. Thin wrapper — logic lives in Rust (names.rs).
/// List functions.
///
/// @param regmatch  When NULL, all of them.
///                  Otherwise functions matching "regmatch".
static void list_functions(regmatch_T *regmatch)
{
  if (regmatch == NULL) {
    rs_list_functions();
  } else {
    // For pattern-based listing, use Rust via the hash iteration callback
    // but we need to pass regmatch -- call list_functions_matching_pat path
    // which uses C's regmatch. Iterate directly here with C regmatch.
    const int prev_ht_changed = func_hashtab.ht_changed;
    size_t todo = func_hashtab.ht_used;
    const hashitem_T *const ht_array = func_hashtab.ht_array;

    for (const hashitem_T *hi = ht_array; todo > 0 && !got_int; hi++) {
      if (!HASHITEM_EMPTY(hi)) {
        ufunc_T *fp = HI2UF(hi);
        todo--;
        if (!isdigit((uint8_t)(*fp->uf_name))
            && vim_regexec(regmatch, fp->uf_name, 0)) {
          if (rs_list_func_head(fp, 0, 0) != 0) {
            return;
          }
          if (rs_function_list_modified(prev_ht_changed)) {
            return;
          }
        }
      }
    }
  }
}


#define MAX_FUNC_NESTING 50

/// Read the body of a function, put every line in "newlines".
/// This stops at "endfunction".
/// "newlines" must already have been initialized.
static int get_function_body(exarg_T *eap, garray_T *newlines, char *line_arg_in,
                             char **line_to_free, bool show_block)
{
  bool saved_wait_return = need_wait_return;
  char *line_arg = line_arg_in;
  int indent = 2;
  int nesting = 0;
  char *skip_until = NULL;
  int ret = FAIL;
  bool is_heredoc = false;
  char *heredoc_trimmed = NULL;
  size_t heredoc_trimmedlen = 0;
  bool do_concat = true;

  while (true) {
    if (KeyTyped) {
      msg_scroll = true;
      saved_wait_return = false;
    }
    need_wait_return = false;

    char *theline;
    char *p;
    char *arg;

    if (line_arg != NULL) {
      // Use eap->arg, split up in parts by line breaks.
      theline = line_arg;
      p = vim_strchr(theline, '\n');
      if (p == NULL) {
        line_arg += strlen(line_arg);
      } else {
        *p = NUL;
        line_arg = p + 1;
      }
    } else {
      xfree(*line_to_free);
      if (eap->ea_getline == NULL) {
        theline = getcmdline(':', 0, indent, do_concat);
      } else {
        theline = eap->ea_getline(':', eap->cookie, indent, do_concat);
      }
      *line_to_free = theline;
    }
    if (KeyTyped) {
      lines_left = Rows - 1;
    }
    if (theline == NULL) {
      if (skip_until != NULL) {
        semsg(_(e_missing_heredoc_end_marker_str), skip_until);
      } else {
        emsg(_("E126: Missing :endfunction"));
      }
      goto theend;
    }
    if (show_block) {
      assert(indent >= 0);
      ui_ext_cmdline_block_append((size_t)indent, theline);
    }

    // Detect line continuation: SOURCING_LNUM increased more than one.
    linenr_T sourcing_lnum_off = get_sourced_lnum(eap->ea_getline, eap->cookie);
    if (SOURCING_LNUM < sourcing_lnum_off) {
      sourcing_lnum_off -= SOURCING_LNUM;
    } else {
      sourcing_lnum_off = 0;
    }

    if (skip_until != NULL) {
      // Don't check for ":endfunc" between
      // * ":append" and "."
      // * ":python <<EOF" and "EOF"
      // * ":let {var-name} =<< [trim] {marker}" and "{marker}"
      if (heredoc_trimmed == NULL
          || (is_heredoc && skipwhite(theline) == theline)
          || strncmp(theline, heredoc_trimmed, heredoc_trimmedlen) == 0) {
        if (heredoc_trimmed == NULL) {
          p = theline;
        } else if (is_heredoc) {
          p = skipwhite(theline) == theline ? theline : theline + heredoc_trimmedlen;
        } else {
          p = theline + heredoc_trimmedlen;
        }
        if (strcmp(p, skip_until) == 0) {
          XFREE_CLEAR(skip_until);
          XFREE_CLEAR(heredoc_trimmed);
          heredoc_trimmedlen = 0;
          do_concat = true;
          is_heredoc = false;
        }
      }
    } else {
      // skip ':' and blanks
      for (p = theline; ascii_iswhite(*p) || *p == ':'; p++) {}

      // Check for "endfunction".
      if (checkforcmd(&p, "endfunction", 4) && nesting-- == 0) {
        if (*p == '!') {
          p++;
        }
        char *nextcmd = NULL;
        if (*p == '|') {
          nextcmd = p + 1;
        } else if (line_arg != NULL && *skipwhite(line_arg) != NUL) {
          nextcmd = line_arg;
        } else if (*p != NUL && *p != '"' && p_verbose > 0) {
          swmsg(true, _("W22: Text found after :endfunction: %s"), p);
        }
        if (nextcmd != NULL) {
          // Another command follows. If the line came from "eap" we
          // can simply point into it, otherwise we need to change
          // "eap->cmdlinep".
          eap->nextcmd = nextcmd;
          if (*line_to_free != NULL) {
            xfree(*eap->cmdlinep);
            *eap->cmdlinep = *line_to_free;
            *line_to_free = NULL;
          }
        }
        break;
      }

      // Increase indent inside "if", "while", "for" and "try", decrease
      // at "end".
      if (indent > 2 && strncmp(p, "end", 3) == 0) {
        indent -= 2;
      } else if (strncmp(p, "if", 2) == 0
                 || strncmp(p, "wh", 2) == 0
                 || strncmp(p, "for", 3) == 0
                 || strncmp(p, "try", 3) == 0) {
        indent += 2;
      }

      // Check for defining a function inside this function.
      if (checkforcmd(&p, "function", 2)) {
        if (*p == '!') {
          p = skipwhite(p + 1);
        }
        p += eval_fname_script(p);
        xfree(trans_function_name(&p, true, 0, NULL, NULL));
        if (*skipwhite(p) == '(') {
          if (nesting == MAX_FUNC_NESTING - 1) {
            emsg(_(e_function_nesting_too_deep));
          } else {
            nesting++;
            indent += 2;
          }
        }
      }

      // Check for ":append", ":change", ":insert".
      char *const tp = p = skip_range(p, NULL);
      if ((checkforcmd(&p, "append", 1)
           || checkforcmd(&p, "change", 1)
           || checkforcmd(&p, "insert", 1))
          && (*p == '!' || *p == '|' || ascii_iswhite_nl_or_nul(*p))) {
        skip_until = xmemdupz(".", 1);
      } else {
        p = tp;
      }

      // heredoc: Check for ":python <<EOF", ":lua <<EOF", etc.
      arg = skipwhite(skiptowhite(p));
      if (arg[0] == '<' && arg[1] == '<'
          && ((p[0] == 'p' && p[1] == 'y'
               && (!ASCII_ISALNUM(p[2]) || p[2] == 't'
                   || ((p[2] == '3' || p[2] == 'x')
                       && !ASCII_ISALPHA(p[3]))))
              || (p[0] == 'p' && p[1] == 'e'
                  && (!ASCII_ISALPHA(p[2]) || p[2] == 'r'))
              || (p[0] == 't' && p[1] == 'c'
                  && (!ASCII_ISALPHA(p[2]) || p[2] == 'l'))
              || (p[0] == 'l' && p[1] == 'u' && p[2] == 'a'
                  && !ASCII_ISALPHA(p[3]))
              || (p[0] == 'r' && p[1] == 'u' && p[2] == 'b'
                  && (!ASCII_ISALPHA(p[3]) || p[3] == 'y'))
              || (p[0] == 'm' && p[1] == 'z'
                  && (!ASCII_ISALPHA(p[2]) || p[2] == 's')))) {
        // ":python <<" continues until a dot, like ":append"
        p = skipwhite(arg + 2);
        if (strncmp(p, "trim", 4) == 0
            && (p[4] == NUL || ascii_iswhite(p[4]))) {
          // Ignore leading white space.
          p = skipwhite(p + 4);
          heredoc_trimmedlen = (size_t)(skipwhite(theline) - theline);
          heredoc_trimmed = xmemdupz(theline, heredoc_trimmedlen);
        }
        if (*p == NUL) {
          skip_until = xmemdupz(".", 1);
        } else {
          skip_until = xmemdupz(p, (size_t)(skiptowhite(p) - p));
        }
        do_concat = false;
        is_heredoc = true;
      }

      if (!is_heredoc) {
        // Check for ":let v =<< [trim] EOF"
        //       and ":let [a, b] =<< [trim] EOF"
        arg = p;
        if (checkforcmd(&arg, "let", 2)) {
          int var_count = 0;
          int semicolon = 0;
          arg = (char *)skip_var_list(arg, &var_count, &semicolon, true);
          if (arg != NULL) {
            arg = skipwhite(arg);
          }
          if (arg != NULL && strncmp(arg, "=<<", 3) == 0) {
            p = skipwhite(arg + 3);
            bool has_trim = false;
            while (true) {
              if (strncmp(p, "trim", 4) == 0
                  && (p[4] == NUL || ascii_iswhite(p[4]))) {
                // Ignore leading white space.
                p = skipwhite(p + 4);
                has_trim = true;
                continue;
              }
              if (strncmp(p, "eval", 4) == 0
                  && (p[4] == NUL || ascii_iswhite(p[4]))) {
                // Ignore leading white space.
                p = skipwhite(p + 4);
                continue;
              }
              break;
            }
            if (has_trim) {
              heredoc_trimmedlen = (size_t)(skipwhite(theline) - theline);
              heredoc_trimmed = xmemdupz(theline, heredoc_trimmedlen);
            }
            XFREE_CLEAR(skip_until);
            skip_until = xmemdupz(p, (size_t)(skiptowhite(p) - p));
            do_concat = false;
            is_heredoc = true;
          }
        }
      }
    }

    // Add the line to the function.
    ga_grow(newlines, 1 + (int)sourcing_lnum_off);

    // Copy the line to newly allocated memory.  get_one_sourceline()
    // allocates 250 bytes per line, this saves 80% on average.  The cost
    // is an extra alloc/free.
    p = xstrdup(theline);
    ((char **)(newlines->ga_data))[newlines->ga_len++] = p;

    // Add NULL lines for continuation lines, so that the line count is
    // equal to the index in the growarray.
    while (sourcing_lnum_off-- > 0) {
      ((char **)(newlines->ga_data))[newlines->ga_len++] = NULL;
    }

    // Check for end of eap->arg.
    if (line_arg != NULL && *line_arg == NUL) {
      line_arg = NULL;
    }
  }

  // Return OK when no error was detected.
  if (!did_emsg) {
    ret = OK;
  }

theend:
  xfree(skip_until);
  xfree(heredoc_trimmed);
  need_wait_return |= saved_wait_return;
  return ret;
}

/// ":function"
void ex_function(exarg_T *eap)
{
  char *line_to_free = NULL;
  char *arg;
  char *line_arg = NULL;
  garray_T newargs;
  garray_T default_args;
  garray_T newlines;
  int varargs = false;
  int flags = 0;
  ufunc_T *fp = NULL;
  bool free_fp = false;
  bool overwrite = false;
  funcdict_T fudi;
  static int func_nr = 0;           // number for nameless function
  hashtab_T *ht;
  bool show_block = false;

  // ":function" without argument: list functions.
  if (ends_excmd(*eap->arg)) {
    if (!eap->skip) {
      list_functions(NULL);
    }
    eap->nextcmd = check_nextcmd(eap->arg);
    return;
  }

  // ":function /pat": list functions matching pattern.
  if (*eap->arg == '/') {
    char *p = rs_list_functions_matching_pat(eap);
    eap->nextcmd = check_nextcmd(p);
    return;
  }

  // Get the function name.  There are these situations:
  // func        function name
  //             "name" == func, "fudi.fd_dict" == NULL
  // dict.func   new dictionary entry
  //             "name" == NULL, "fudi.fd_dict" set,
  //             "fudi.fd_di" == NULL, "fudi.fd_newkey" == func
  // dict.func   existing dict entry with a Funcref
  //             "name" == func, "fudi.fd_dict" set,
  //             "fudi.fd_di" set, "fudi.fd_newkey" == NULL
  // dict.func   existing dict entry that's not a Funcref
  //             "name" == NULL, "fudi.fd_dict" set,
  //             "fudi.fd_di" set, "fudi.fd_newkey" == NULL
  // s:func      script-local function name
  // g:func      global function name, same as "func"
  char *p = eap->arg;
  char *name = save_function_name(&p, eap->skip, TFN_NO_AUTOLOAD, &fudi);
  int paren = (vim_strchr(p, '(') != NULL);
  if (name == NULL && (fudi.fd_dict == NULL || !paren) && !eap->skip) {
    // Return on an invalid expression in braces, unless the expression
    // evaluation has been cancelled due to an aborting error, an
    // interrupt, or an exception.
    if (!aborting()) {
      if (fudi.fd_newkey != NULL) {
        semsg(_(e_dictkey), fudi.fd_newkey);
      }
      xfree(fudi.fd_newkey);
      return;
    }
    eap->skip = true;
  }

  // An error in a function call during evaluation of an expression in magic
  // braces should not cause the function not to be defined.
  const int saved_did_emsg = did_emsg;
  did_emsg = false;

  // ":function func" with only function name: list function.
  if (!paren) {
    fp = rs_list_one_function(eap, name, p);
    goto ret_free;
  }

  // ":function name(arg1, arg2)" Define function.
  p = skipwhite(p);
  if (*p != '(') {
    if (!eap->skip) {
      semsg(_("E124: Missing '(': %s"), eap->arg);
      goto ret_free;
    }
    // attempt to continue by skipping some text
    if (vim_strchr(p, '(') != NULL) {
      p = vim_strchr(p, '(');
    }
  }
  p = skipwhite(p + 1);

  ga_init(&newargs, (int)sizeof(char *), 3);
  ga_init(&newlines, (int)sizeof(char *), 3);

  if (!eap->skip) {
    // Check the name of the function.  Unless it's a dictionary function
    // (that we are overwriting).
    if (name != NULL) {
      arg = name;
    } else {
      arg = fudi.fd_newkey;
    }
    if (arg != NULL && (fudi.fd_di == NULL || !tv_is_func(fudi.fd_di->di_tv))) {
      char *name_base = arg;
      if ((uint8_t)(*arg) == K_SPECIAL) {
        name_base = vim_strchr(arg, '_');
        if (name_base == NULL) {
          name_base = arg + 3;
        } else {
          name_base++;
        }
      }
      int i;
      for (i = 0; name_base[i] != NUL && (i == 0
                                          ? rs_eval_isnamec1(name_base[i])
                                          : rs_eval_isnamec(name_base[i])); i++) {}
      if (name_base[i] != NUL) {
        emsg_funcname(e_invarg2, arg);
        goto ret_free;
      }
    }
    // Disallow using the g: dict.
    if (fudi.fd_dict != NULL && fudi.fd_dict->dv_scope == VAR_DEF_SCOPE) {
      emsg(_("E862: Cannot use g: here"));
      goto ret_free;
    }
  }

  if (get_function_args(&p, ')', &newargs, &varargs,
                        &default_args, eap->skip) == FAIL) {
    goto errret_2;
  }

  if (KeyTyped && ui_has(kUICmdline)) {
    show_block = true;
    ui_ext_cmdline_block_append(0, eap->cmd);
  }

  // find extra arguments "range", "dict", "abort" and "closure"
  while (true) {
    p = skipwhite(p);
    if (strncmp(p, "range", 5) == 0) {
      flags |= FC_RANGE;
      p += 5;
    } else if (strncmp(p, "dict", 4) == 0) {
      flags |= FC_DICT;
      p += 4;
    } else if (strncmp(p, "abort", 5) == 0) {
      flags |= FC_ABORT;
      p += 5;
    } else if (strncmp(p, "closure", 7) == 0) {
      flags |= FC_CLOSURE;
      p += 7;
      if (current_funccal == NULL) {
        emsg_funcname(N_("E932: Closure function should not be at top level: %s"),
                      name == NULL ? "" : name);
        goto erret;
      }
    } else {
      break;
    }
  }

  // When there is a line break use what follows for the function body.
  // Makes 'exe "func Test()\n...\nendfunc"' work.
  if (*p == '\n') {
    line_arg = p + 1;
  } else if (*p != NUL && *p != '"' && !eap->skip && !did_emsg) {
    semsg(_(e_trailing_arg), p);
  }

  // Read the body of the function, until ":endfunction" is found.
  if (KeyTyped) {
    // Check if the function already exists, don't let the user type the
    // whole function before telling them it doesn't work!  For a script we
    // need to skip the body to be able to find what follows.
    if (!eap->skip && !eap->forceit) {
      if (fudi.fd_dict != NULL && fudi.fd_newkey == NULL) {
        emsg(_(e_funcdict));
      } else if (name != NULL && find_func(name) != NULL) {
        emsg_funcname(e_funcexts, name);
      }
    }

    if (!eap->skip && did_emsg) {
      goto erret;
    }

    if (!ui_has(kUICmdline)) {
      msg_putchar('\n');              // don't overwrite the function name
    }
    cmdline_row = msg_row;
  }

  // Save the starting line number.
  linenr_T sourcing_lnum_top = SOURCING_LNUM;

  // Do not define the function when getting the body fails and when skipping.
  if (get_function_body(eap, &newlines, line_arg, &line_to_free, show_block) == FAIL
      || eap->skip) {
    goto erret;
  }

  // If there are no errors, add the function
  size_t namelen = 0;
  if (fudi.fd_dict == NULL) {
    dictitem_T *v = find_var(name, strlen(name), &ht, false);
    if (v != NULL && v->di_tv.v_type == VAR_FUNC) {
      emsg_funcname(N_("E707: Function name conflicts with variable: %s"), name);
      goto erret;
    }

    fp = find_func(name);
    if (fp != NULL) {
      // Function can be replaced with "function!" and when sourcing the
      // same script again, but only once.
      if (!eap->forceit
          && (fp->uf_script_ctx.sc_sid != current_sctx.sc_sid
              || fp->uf_script_ctx.sc_seq == current_sctx.sc_seq)) {
        emsg_funcname(e_funcexts, name);
        goto errret_keep;
      }
      if (fp->uf_calls > 0) {
        emsg_funcname(N_("E127: Cannot redefine function %s: It is in use"), name);
        goto errret_keep;
      }
      if (fp->uf_refcount > 1) {
        // This function is referenced somewhere, don't redefine it but
        // create a new one.
        (fp->uf_refcount)--;
        fp->uf_flags |= FC_REMOVED;
        fp = NULL;
        overwrite = true;
      } else {
        char *exp_name = fp->uf_name_exp;
        // redefine existing function, keep the expanded name
        XFREE_CLEAR(name);
        fp->uf_name_exp = NULL;
        rs_func_clear_items(fp);
        fp->uf_name_exp = exp_name;
        fp->uf_profiling = false;
        fp->uf_prof_initialized = false;
      }
    }
  } else {
    char numbuf[NUMBUFLEN];

    fp = NULL;
    if (fudi.fd_newkey == NULL && !eap->forceit) {
      emsg(_(e_funcdict));
      goto erret;
    }
    if (fudi.fd_di == NULL) {
      if (value_check_lock(fudi.fd_dict->dv_lock, eap->arg, TV_CSTRING)) {
        // Can't add a function to a locked dictionary
        goto erret;
      }
    } else if (value_check_lock(fudi.fd_di->di_tv.v_lock, eap->arg, TV_CSTRING)) {
      // Can't change an existing function if it is locked
      goto erret;
    }

    // Give the function a sequential number.  Can only be used with a
    // Funcref!
    xfree(name);
    namelen = (size_t)snprintf(numbuf, sizeof(numbuf), "%d", ++func_nr);
    name = xmemdupz(numbuf, namelen);
  }

  if (fp == NULL) {
    if (fudi.fd_dict == NULL && vim_strchr(name, AUTOLOAD_CHAR) != NULL) {
      // Check that the autoload name matches the script name.
      int j = FAIL;
      if (SOURCING_NAME != NULL) {
        char *scriptname = autoload_name(name, strlen(name));
        p = vim_strchr(scriptname, '/');
        int plen = (int)strlen(p);
        int slen = (int)strlen(SOURCING_NAME);
        if (slen > plen && path_fnamecmp(p, SOURCING_NAME + slen - plen) == 0) {
          j = OK;
        }
        xfree(scriptname);
      }
      if (j == FAIL) {
        semsg(_("E746: Function name does not match script file name: %s"),
              name);
        goto erret;
      }
    }

    if (namelen == 0) {
      namelen = strlen(name);
    }
    fp = alloc_ufunc(name, namelen);

    if (fudi.fd_dict != NULL) {
      if (fudi.fd_di == NULL) {
        // Add new dict entry
        fudi.fd_di = tv_dict_item_alloc(fudi.fd_newkey);
        if (tv_dict_add(fudi.fd_dict, fudi.fd_di) == FAIL) {
          xfree(fudi.fd_di);
          XFREE_CLEAR(fp);
          goto erret;
        }
      } else {
        // Overwrite existing dict entry.
        tv_clear(&fudi.fd_di->di_tv);
      }
      fudi.fd_di->di_tv.v_type = VAR_FUNC;
      fudi.fd_di->di_tv.vval.v_string = xmemdupz(name, namelen);

      // behave like "dict" was used
      flags |= FC_DICT;
    }

    // insert the new function in the function list
    if (overwrite) {
      hashitem_T *hi = hash_find(&func_hashtab, name);
      hi->hi_key = UF2HIKEY(fp);
    } else if (hash_add(&func_hashtab, UF2HIKEY(fp)) == FAIL) {
      free_fp = true;
      goto erret;
    }
    fp->uf_refcount = 1;
  }
  fp->uf_args = newargs;
  fp->uf_def_args = default_args;
  fp->uf_lines = newlines;
  if ((flags & FC_CLOSURE) != 0) {
    register_closure(fp);
  } else {
    fp->uf_scoped = NULL;
  }
  if (prof_def_func()) {
    func_do_profile(fp);
  }
  fp->uf_varargs = varargs;
  if (sandbox) {
    flags |= FC_SANDBOX;
  }
  fp->uf_flags = flags;
  fp->uf_calls = 0;
  fp->uf_script_ctx = current_sctx;
  fp->uf_script_ctx.sc_lnum += sourcing_lnum_top;
  nlua_set_sctx(&fp->uf_script_ctx);

  goto ret_free;

erret:
  if (fp != NULL) {
    // these were set to "newargs" and "default_args", which are cleared below
    ga_init(&fp->uf_args, (int)sizeof(char *), 1);
    ga_init(&fp->uf_def_args, (int)sizeof(char *), 1);
  }
errret_2:
  if (fp != NULL) {
    XFREE_CLEAR(fp->uf_name_exp);
  }
  if (free_fp) {
    XFREE_CLEAR(fp);
  }
errret_keep:
  ga_clear_strings(&newargs);
  ga_clear_strings(&default_args);
  ga_clear_strings(&newlines);
ret_free:
  xfree(line_to_free);
  xfree(fudi.fd_newkey);
  xfree(name);
  did_emsg |= saved_did_emsg;
  if (show_block) {
    ui_ext_cmdline_block_leave();
  }
}

/// @return  5 if "p" starts with "<SID>" or "<SNR>" (ignoring case).
///          2 if "p" starts with "s:".
///          0 otherwise.
/// Function given to ExpandGeneric() to obtain the list of user defined
/// function names.
char *get_user_func_name(expand_T *xp, int idx)
{
  static size_t done;
  static int changed;
  static hashitem_T *hi;

  if (idx == 0) {
    done = 0;
    hi = func_hashtab.ht_array;
    changed = func_hashtab.ht_changed;
  }
  assert(hi);
  if (changed == func_hashtab.ht_changed && done < func_hashtab.ht_used) {
    if (done++ > 0) {
      hi++;
    }
    while (HASHITEM_EMPTY(hi)) {
      hi++;
    }
    ufunc_T *fp = HI2UF(hi);

    if ((fp->uf_flags & FC_DICT)
        || strncmp(fp->uf_name, "<lambda>", 8) == 0) {
      return "";       // don't show dict and lambda functions
    }

    if (fp->uf_namelen + 4 >= IOSIZE) {
      return fp->uf_name;  // Prevent overflow.
    }

    int len = rs_cat_func_name(IObuff, IOSIZE, fp);
    if (xp->xp_context != EXPAND_USER_FUNC) {
      xstrlcpy(IObuff + len, "(", IOSIZE - (size_t)len);
      if (!fp->uf_varargs && GA_EMPTY(&fp->uf_args)) {
        len++;
        xstrlcpy(IObuff + len, ")", IOSIZE - (size_t)len);
      }
    }
    return IObuff;
  }
  return NULL;
}

/// Phase 7: C implementation shim for ex_delfunction (called from Rust).
void nvim_ex_delfunction_impl(exarg_T *eap)
{
  ufunc_T *fp = NULL;
  funcdict_T fudi;

  char *p = eap->arg;
  char *name = trans_function_name(&p, eap->skip, 0, &fudi, NULL);
  xfree(fudi.fd_newkey);
  if (name == NULL) {
    if (fudi.fd_dict != NULL && !eap->skip) {
      emsg(_(e_funcref));
    }
    return;
  }
  if (!ends_excmd(*skipwhite(p))) {
    xfree(name);
    semsg(_(e_trailing_arg), p);
    return;
  }
  eap->nextcmd = check_nextcmd(p);
  if (eap->nextcmd != NULL) {
    *p = NUL;
  }

  if (isdigit((uint8_t)(*name)) && fudi.fd_dict == NULL) {
    if (!eap->skip) {
      semsg(_(e_invarg2), eap->arg);
    }
    xfree(name);
    return;
  }
  if (!eap->skip) {
    fp = find_func(name);
  }
  xfree(name);

  if (!eap->skip) {
    if (fp == NULL) {
      if (!eap->forceit) {
        semsg(_(e_nofunc), eap->arg);
      }
      return;
    }
    if (fp->uf_calls > 0) {
      semsg(_("E131: Cannot delete function %s: It is in use"), eap->arg);
      return;
    }
    // check `uf_refcount > 2` because deleting a function should also reduce
    // the reference count, and 1 is the initial refcount.
    if (fp->uf_refcount > 2) {
      semsg(_("Cannot delete function %s: It is being used internally"),
            eap->arg);
      return;
    }

    if (fudi.fd_dict != NULL) {
      // Delete the dict item that refers to the function, it will
      // invoke func_unref() and possibly delete the function.
      tv_dict_item_remove(fudi.fd_dict, fudi.fd_di);
    } else {
      // A normal function (not a numbered function or lambda) has a
      // refcount of 1 for the entry in the hashtable.  When deleting
      // it and the refcount is more than one, it should be kept.
      // A numbered function or lambda should be kept if the refcount is
      // one or more.
      if (fp->uf_refcount > (rs_func_name_refcount(fp->uf_name) ? 0 : 1)) {
        // Function is still referenced somewhere. Don't free it but
        // do remove it from the hashtable.
        if (rs_func_remove(fp)) {
          fp->uf_refcount--;
        }
        fp->uf_flags |= FC_DELETED;
      } else {
        rs_func_clear_free(fp, 0);
      }
    }
  }
}

/// ":delfunction {name}"
/// Check whether funccall is still referenced outside
///
/// It is supposed to be referenced if either it is referenced itself or if l:,
/// a: or a:000 are referenced as all these are statically allocated within
/// funccall structure.
// nvim_fc_referenced_impl inlined into rs_fc_referenced (Rust, Phase 12)

// nvim_can_free_funccal_impl inlined into rs_can_free_funccal (Rust, Phase 12)

/// Phase 6: C implementation shim for ex_return (called from Rust).
void nvim_ex_return_impl(exarg_T *eap)
{
  char *arg = eap->arg;
  typval_T rettv;
  bool returning = false;

  if (current_funccal == NULL) {
    emsg(_("E133: :return not inside a function"));
    return;
  }

  evalarg_T evalarg = { .eval_flags = eap->skip ? 0 : EVAL_EVALUATE };

  if (eap->skip) {
    emsg_skip++;
  }

  eap->nextcmd = NULL;
  if ((*arg != NUL && *arg != '|' && *arg != '\n')
      && eval0(arg, &rettv, eap, &evalarg) != FAIL) {
    if (!eap->skip) {
      returning = do_return(eap, false, true, &rettv);
    } else {
      tv_clear(&rettv);
    }
  } else if (!eap->skip) {  // It's safer to return also on error.
    // In return statement, cause_abort should be force_abort.
    update_force_abort();

    // Return unless the expression evaluation has been cancelled due to an
    // aborting error, an interrupt, or an exception.
    if (!aborting()) {
      returning = do_return(eap, false, true, NULL);
    }
  }

  // When skipping or the return gets pending, advance to the next command
  // in this line (!returning).  Otherwise, ignore the rest of the line.
  // Following lines will be ignored by get_func_line().
  if (returning) {
    eap->nextcmd = NULL;
  } else if (eap->nextcmd == NULL) {          // no argument
    eap->nextcmd = check_nextcmd(arg);
  }

  if (eap->skip) {
    emsg_skip--;
  }
  clear_evalarg(&evalarg, eap);
}

/// ":return [expr]"
/// Lower level implementation of "call".  Only called when not skipping.
// ex_call_inner migrated to Rust (funccal.rs Phase 25).
extern int ex_call_inner(exarg_T *eap, char *name, char **arg, char *startarg,
                         const funcexe_T *funcexe_init, evalarg_T *evalarg);

// nvim_ex_defer_inner_impl migrated to Rust (defer.rs Phase 24).
// rs_ex_defer_inner now implements the logic directly.


/// Return true if currently inside a function call.
/// Give an error message and return false when not.
/// Phase 3: C implementation shim for handle_defer_one (called from Rust).
void nvim_handle_defer_one_impl(funccall_T *funccal)
{
  for (int idx = funccal->fc_defer.ga_len - 1; idx >= 0; idx--) {
    defer_T *dr = ((defer_T *)funccal->fc_defer.ga_data) + idx;

    if (dr->dr_name == NULL) {
      // already being called, can happen if function does ":qa"
      continue;
    }

    funcexe_T funcexe = { .fe_evaluate = true };
    typval_T rettv;
    rettv.v_type = VAR_UNKNOWN;
    char *name = dr->dr_name;
    dr->dr_name = NULL;

    exception_state_T estate;
    exception_state_save(&estate);
    exception_state_clear();
    call_func(name, -1, &rettv, dr->dr_argcount, dr->dr_argvars, &funcexe);
    exception_state_restore(&estate);

    tv_clear(&rettv);
    xfree(name);
    for (int i = dr->dr_argcount - 1; i >= 0; i--) {
      tv_clear(&dr->dr_argvars[i]);
    }
  }
  ga_clear(&funccal->fc_defer);
}


/// ":1,25call func(arg1, arg2)" function call.
/// ":defer func(arg1, arg2)"    deferred function call.
void ex_call(exarg_T *eap)
{
  char *arg = eap->arg;
  bool failed = false;
  funcdict_T fudi;
  partial_T *partial = NULL;
  evalarg_T evalarg;

  fill_evalarg_from_eap(&evalarg, eap, eap->skip);
  if (eap->skip) {
    typval_T rettv;
    // trans_function_name() doesn't work well when skipping, use eval0()
    // instead to skip to any following command, e.g. for:
    //   :if 0 | call dict.foo().bar() | endif.
    emsg_skip++;
    if (eval0(eap->arg, &rettv, eap, &evalarg) != FAIL) {
      tv_clear(&rettv);
    }
    emsg_skip--;
    clear_evalarg(&evalarg, eap);
    return;
  }

  char *tofree = trans_function_name(&arg, false, TFN_INT, &fudi, &partial);
  if (fudi.fd_newkey != NULL) {
    // Still need to give an error message for missing key.
    semsg(_(e_dictkey), fudi.fd_newkey);
    xfree(fudi.fd_newkey);
  }
  if (tofree == NULL) {
    return;
  }

  // Increase refcount on dictionary, it could get deleted when evaluating
  // the arguments.
  if (fudi.fd_dict != NULL) {
    fudi.fd_dict->dv_refcount++;
  }

  // If it is the name of a variable of type VAR_FUNC or VAR_PARTIAL use its
  // contents. For VAR_PARTIAL get its partial, unless we already have one
  // from trans_function_name().
  int len = (int)strlen(tofree);
  bool found_var = false;
  char *name = deref_func_name(tofree, &len, partial != NULL ? NULL : &partial, false, &found_var);

  // Skip white space to allow ":call func ()".  Not good, but required for
  // backward compatibility.
  char *startarg = skipwhite(arg);

  if (*startarg != '(') {
    semsg(_(e_missingparen), eap->arg);
    goto end;
  }

  if (eap->cmdidx == CMD_defer) {
    arg = startarg;
    failed = rs_ex_defer_inner(name, &arg, partial, &evalarg) == FAIL;
  } else {
    funcexe_T funcexe = FUNCEXE_INIT;
    funcexe.fe_partial = partial;
    funcexe.fe_selfdict = fudi.fd_dict;
    funcexe.fe_firstline = eap->line1;
    funcexe.fe_lastline = eap->line2;
    funcexe.fe_found_var = found_var;
    funcexe.fe_evaluate = true;
    failed = ex_call_inner(eap, name, &arg, startarg, &funcexe, &evalarg);
  }

  // When inside :try we need to check for following "| catch" or "| endtry".
  // Not when there was an error, but do check if an exception was thrown.
  if ((!aborting() || did_throw) && (!failed || eap->cstack->cs_trylevel > 0)) {
    // Check for trailing illegal characters and a following command.
    if (!ends_excmd(*arg)) {
      if (!failed && !aborting()) {
        emsg_severe = true;
        semsg(_(e_trailing_arg), arg);
      }
    } else {
      eap->nextcmd = check_nextcmd(arg);
    }
  }
  clear_evalarg(&evalarg, eap);

end:
  tv_dict_unref(fudi.fd_dict);
  xfree(tofree);
}

/// Return from a function.  Possibly makes the return pending.  Also called
/// for a pending return at the ":endtry" or after returning from an extra
/// do_cmdline().  "reanimate" is used in the latter case.
///
/// @param reanimate  used after returning from an extra do_cmdline().
/// @param is_cmd     set when called due to a ":return" command.
/// @param rettv      may point to a typval_T with the return rettv.
///
/// @return  true when the return can be carried out,
///          false when the return gets pending.
/// Phase 6: C implementation shim for do_return (called from Rust).
int nvim_do_return_impl(exarg_T *eap, int reanimate, int is_cmd, void *rettv)
{
  cstack_T *const cstack = eap->cstack;

  if (reanimate) {
    // Undo the return.
    current_funccal->fc_returned = false;
  }

  // Cleanup (and deactivate) conditionals, but stop when a try conditional
  // not in its finally clause (which then is to be executed next) is found.
  // In this case, make the ":return" pending for execution at the ":endtry".
  // Otherwise, return normally.
  int idx = cleanup_conditionals(eap->cstack, 0, true);
  if (idx >= 0) {
    cstack->cs_pending[idx] = CSTP_RETURN;

    if (!is_cmd && !reanimate) {
      // A pending return again gets pending.  "rettv" points to an
      // allocated variable with the rettv of the original ":return"'s
      // argument if present or is NULL else.
      cstack->cs_rettv[idx] = rettv;
    } else {
      // When undoing a return in order to make it pending, get the stored
      // return rettv.
      if (reanimate) {
        assert(current_funccal->fc_rettv);
        rettv = current_funccal->fc_rettv;
      }

      if (rettv != NULL) {
        // Store the value of the pending return.
        cstack->cs_rettv[idx] = xcalloc(1, sizeof(typval_T));
        *(typval_T *)cstack->cs_rettv[idx] = *(typval_T *)rettv;
      } else {
        cstack->cs_rettv[idx] = NULL;
      }

      if (reanimate) {
        // The pending return value could be overwritten by a ":return"
        // without argument in a finally clause; reset the default
        // return value.
        current_funccal->fc_rettv->v_type = VAR_NUMBER;
        current_funccal->fc_rettv->vval.v_number = 0;
      }
    }
    report_make_pending(CSTP_RETURN, rettv);
  } else {
    current_funccal->fc_returned = true;

    // If the return is carried out now, store the return value.  For
    // a return immediately after reanimation, the value is already
    // there.
    if (!reanimate && rettv != NULL) {
      tv_clear(current_funccal->fc_rettv);
      *current_funccal->fc_rettv = *(typval_T *)rettv;
      if (!is_cmd) {
        xfree(rettv);
      }
    }
  }

  return idx < 0;
}

/// Phase 6: C implementation shim for get_return_cmd (called from Rust).
char *nvim_get_return_cmd_impl(void *rettv)
{
  char *s = NULL;
  char *tofree = NULL;
  size_t slen = 0;

  if (rettv != NULL) {
    tofree = s = encode_tv2echo((typval_T *)rettv, NULL);
  }
  if (s == NULL) {
    s = "";
  } else {
    slen = strlen(s);
  }

  xstrlcpy(IObuff, ":return ", IOSIZE);
  xstrlcpy(IObuff + 8, s, IOSIZE - 8);
  size_t IObufflen = 8 + slen;
  if (IObufflen >= IOSIZE) {
    STRCPY(IObuff + IOSIZE - 4, "...");
    IObufflen = IOSIZE - 1;
  }
  xfree(tofree);
  return xstrnsave(IObuff, IObufflen);
}

/// Get next function line.
/// Called by do_cmdline() to get the next line.
///
/// @return  allocated string, or NULL for end of function.
char *get_func_line(int c, void *cookie, int indent, bool do_concat)
{
  funccall_T *fcp = (funccall_T *)cookie;
  ufunc_T *fp = fcp->fc_func;
  char *retval;

  // If breakpoints have been added/deleted need to check for it.
  if (fcp->fc_dbg_tick != debug_tick) {
    fcp->fc_breakpoint = dbg_find_breakpoint(false, fp->uf_name, SOURCING_LNUM);
    fcp->fc_dbg_tick = debug_tick;
  }
  if (do_profiling == PROF_YES) {
    func_line_end(cookie);
  }

  garray_T *gap = &fp->uf_lines;  // growarray with function lines
  if (((fp->uf_flags & FC_ABORT) && did_emsg && !aborted_in_try())
      || fcp->fc_returned) {
    retval = NULL;
  } else {
    // Skip NULL lines (continuation lines).
    while (fcp->fc_linenr < gap->ga_len
           && ((char **)(gap->ga_data))[fcp->fc_linenr] == NULL) {
      fcp->fc_linenr++;
    }
    if (fcp->fc_linenr >= gap->ga_len) {
      retval = NULL;
    } else {
      retval = xstrdup(((char **)(gap->ga_data))[fcp->fc_linenr++]);
      SOURCING_LNUM = fcp->fc_linenr;
      if (do_profiling == PROF_YES) {
        func_line_start(cookie);
      }
    }
  }

  // Did we encounter a breakpoint?
  if (fcp->fc_breakpoint != 0 && fcp->fc_breakpoint <= SOURCING_LNUM) {
    dbg_breakpoint(fp->uf_name, SOURCING_LNUM);
    // Find next breakpoint.
    fcp->fc_breakpoint = dbg_find_breakpoint(false, fp->uf_name, SOURCING_LNUM);
    fcp->fc_dbg_tick = debug_tick;
  }

  return retval;
}

// func_has_ended, func_has_abort migrated to Rust (lookup.rs Phase 6)

// make_partial migrated to Rust (partial.rs Phase 10)

// func_name, func_breakpoint, func_dbg_tick, func_level migrated to Rust (lookup.rs Phase 6)

// C accessor for current_funccal->fc_returned (used by Rust)
int nvim_get_current_funccal_fc_returned(void) { return current_funccal->fc_returned; }

// Implemented in Rust (nvim-eval crate)
extern int current_func_returned(void);

// nvim_free_unref_funccal_impl migrated to Rust (gc.rs Phase 26).
// rs_free_unref_funccal now implements the logic directly.

// nvim_get_funccal_impl inlined into rs_get_funccal (Rust, Phase 13)
// nvim_get_funccal_local_dict_impl inlined into rs_get_funccal_local_dict (Rust, Phase 13)
// nvim_get_funccal_local_ht_impl inlined into rs_get_funccal_local_ht (Rust, Phase 13)
// nvim_get_funccal_local_var_impl inlined into rs_get_funccal_local_var (Rust, Phase 13)
// nvim_get_funccal_args_dict_impl inlined into rs_get_funccal_args_dict (Rust, Phase 13)
// nvim_get_funccal_args_ht_impl inlined into rs_get_funccal_args_ht (Rust, Phase 13)
// nvim_get_funccal_args_var_impl inlined into rs_get_funccal_args_var (Rust, Phase 13)
// nvim_list_func_vars_impl inlined into rs_list_func_vars (Rust, Phase 13)
// nvim_get_current_funccal_dict_impl inlined into rs_get_current_funccal_dict (Rust, Phase 13)

hashitem_T *nvim_find_hi_in_scoped_ht_impl(const char *name, hashtab_T **pht)
{
  if (current_funccal == NULL || current_funccal->fc_func->uf_scoped == NULL) {
    return NULL;
  }
  funccall_T *old_current_funccal = current_funccal;
  hashitem_T *hi = NULL;
  const size_t namelen = strlen(name);
  const char *varname;
  current_funccal = current_funccal->fc_func->uf_scoped;
  while (current_funccal != NULL) {
    hashtab_T *ht = find_var_ht(name, namelen, &varname);
    if (ht != NULL && *varname != NUL) {
      hi = hash_find_len(ht, varname, namelen - (size_t)(varname - name));
      if (!HASHITEM_EMPTY(hi)) {
        *pht = ht;
        break;
      }
    }
    if (current_funccal == current_funccal->fc_func->uf_scoped) {
      break;
    }
    current_funccal = current_funccal->fc_func->uf_scoped;
  }
  current_funccal = old_current_funccal;
  return hi;
}

dictitem_T *nvim_find_var_in_scoped_ht_impl(const char *name, const size_t namelen,
                                             int no_autoload)
{
  if (current_funccal == NULL || current_funccal->fc_func->uf_scoped == NULL) {
    return NULL;
  }
  dictitem_T *v = NULL;
  funccall_T *old_current_funccal = current_funccal;
  const char *varname;
  current_funccal = current_funccal->fc_func->uf_scoped;
  while (current_funccal) {
    hashtab_T *ht = find_var_ht(name, namelen, &varname);
    if (ht != NULL && *varname != NUL) {
      v = find_var_in_ht(ht, *name, varname, namelen - (size_t)(varname - name), no_autoload);
      if (v != NULL) {
        break;
      }
    }
    if (current_funccal == current_funccal->fc_func->uf_scoped) {
      break;
    }
    current_funccal = current_funccal->fc_func->uf_scoped;
  }
  current_funccal = old_current_funccal;
  return v;
}

// nvim_set_ref_in_previous_funccal_impl inlined into rs_set_ref_in_previous_funccal (Rust, Phase 12)

// nvim_set_ref_in_funccal_impl inlined into rs_set_ref_in_funccal (Rust, Phase 12)

// nvim_set_ref_in_call_stack_impl inlined into rs_set_ref_in_call_stack (Rust, Phase 12)

/// Phase 5: C implementation shim for set_ref_in_functions.
/// Cannot inline: requires HASHITEM_EMPTY and HI2UF macros for hash iteration.
int nvim_set_ref_in_functions_impl(int copyID)
{
  int todo = (int)func_hashtab.ht_used;
  for (hashitem_T *hi = func_hashtab.ht_array; todo > 0 && !got_int; hi++) {
    if (!HASHITEM_EMPTY(hi)) {
      todo--;
      ufunc_T *fp = HI2UF(hi);
      if (!rs_func_name_refcount(fp->uf_name) && set_ref_in_func(NULL, fp, copyID)) {
        return true;
      }
    }
  }
  return false;
}

// nvim_set_ref_in_func_args_impl inlined into rs_set_ref_in_func_args (Rust, Phase 12)

// nvim_set_ref_in_func_impl inlined into rs_set_ref_in_func (Rust, Phase 11)

/// Registers a luaref as a lambda.
char *register_luafunc(LuaRef ref)
{
  String name = get_lambda_name();
  ufunc_T *fp = alloc_ufunc(name.data, name.size);

  fp->uf_refcount = 1;
  fp->uf_varargs = true;
  fp->uf_flags = FC_LUAREF;
  fp->uf_calls = 0;
  fp->uf_script_ctx = current_sctx;
  fp->uf_luaref = ref;

  hash_add(&func_hashtab, UF2HIKEY(fp));

  // coverity[leaked_storage]
  return fp->uf_name;
}

// Rust FFI Accessor Functions

int nvim_ufunc_get_flags(const ufunc_T *fp) { return fp ? fp->uf_flags : 0; }

/// Get minimum number of arguments for ufunc.
int nvim_ufunc_get_min_args(const ufunc_T *fp)
{
  if (fp == NULL) {
    return 0;
  }
  // uf_args.ga_len is total args, uf_def_args.ga_len is optional args
  return fp->uf_args.ga_len - fp->uf_def_args.ga_len;
}

/// Get maximum number of arguments for ufunc (-1 if variadic).
int nvim_ufunc_get_max_args(const ufunc_T *fp)
{
  if (fp == NULL) {
    return 0;
  }
  if (fp->uf_varargs) {
    return -1;
  }
  return fp->uf_args.ga_len;
}

/// Get function from partial.
ufunc_T *nvim_partial_get_func(const partial_T *pt) { return pt ? pt->pt_func : NULL; }

/// Get argument count from partial.
int nvim_partial_get_argc(const partial_T *pt) { return pt ? pt->pt_argc : 0; }

dict_T *nvim_partial_get_dict(const partial_T *pt) { return pt ? pt->pt_dict : NULL; }

// nvim_partial_is_auto, nvim_funcexe_get_partial, nvim_funcexe_get_selfdict,
// nvim_funcexe_get_evaluate -- dead, no Rust callers (Phase 17)

/// Apply FuncUndefined autocmd and return result.
int apply_autocmds_for_funcundefined(const char *name)
{
  return apply_autocmds(EVENT_FUNCUNDEFINED, name, name, true, NULL);
}

/// Check if name is a builtin function (wrapper for static function).
int nvim_is_builtin_function(const char *name, int len) { return rs_builtin_function(name, len); }

// Profile FFI Accessor Functions

ufunc_T *nvim_fc_get_func(const funccall_T *fc) { return fc->fc_func; }

int nvim_ufunc_get_profiling(const ufunc_T *fp) { return fp->uf_profiling; }

void nvim_ufunc_set_profiling(ufunc_T *fp, int val) { fp->uf_profiling = val; }

int nvim_ufunc_get_prof_initialized(const ufunc_T *fp) { return fp->uf_prof_initialized; }

void nvim_ufunc_set_prof_initialized(ufunc_T *fp, int val) { fp->uf_prof_initialized = val; }

int nvim_ufunc_get_lines_len(const ufunc_T *fp) { return fp->uf_lines.ga_len; }

int nvim_ufunc_get_tml_idx(const ufunc_T *fp) { return fp->uf_tml_idx; }

void nvim_ufunc_set_tml_idx(ufunc_T *fp, int val) { fp->uf_tml_idx = val; }

int nvim_ufunc_get_tml_execed(const ufunc_T *fp) { return fp->uf_tml_execed; }

void nvim_ufunc_set_tml_execed(ufunc_T *fp, int val) { fp->uf_tml_execed = val; }

proftime_T nvim_ufunc_get_tml_start(const ufunc_T *fp) { return fp->uf_tml_start; }

void nvim_ufunc_set_tml_start(ufunc_T *fp, proftime_T val) { fp->uf_tml_start = val; }

proftime_T nvim_ufunc_get_tml_children(const ufunc_T *fp) { return fp->uf_tml_children; }

void nvim_ufunc_set_tml_children(ufunc_T *fp, proftime_T val) { fp->uf_tml_children = val; }

proftime_T nvim_ufunc_get_tml_wait(const ufunc_T *fp) { return fp->uf_tml_wait; }

void nvim_ufunc_set_tml_wait(ufunc_T *fp, proftime_T val) { fp->uf_tml_wait = val; }

void nvim_ufunc_set_tm_count(ufunc_T *fp, int val) { fp->uf_tm_count = val; }

int nvim_ufunc_get_tm_count(const ufunc_T *fp) { return fp->uf_tm_count; }

void nvim_ufunc_set_tm_total(ufunc_T *fp, proftime_T val) { fp->uf_tm_total = val; }

proftime_T nvim_ufunc_get_tm_total(const ufunc_T *fp) { return fp->uf_tm_total; }

void nvim_ufunc_set_tm_self(ufunc_T *fp, proftime_T val) { fp->uf_tm_self = val; }

proftime_T nvim_ufunc_get_tm_self(const ufunc_T *fp) { return fp->uf_tm_self; }

uint8_t nvim_ufunc_get_name_first_byte(const ufunc_T *fp) { return (uint8_t)fp->uf_name[0]; }

// Array element accessors for per-line profiling arrays

int nvim_ufunc_get_tml_count_i(const ufunc_T *fp, int i) { return fp->uf_tml_count[i]; }

void nvim_ufunc_set_tml_count_i(ufunc_T *fp, int i, int val) { fp->uf_tml_count[i] = val; }

proftime_T nvim_ufunc_get_tml_total_i(const ufunc_T *fp, int i) { return fp->uf_tml_total[i]; }

void nvim_ufunc_set_tml_total_i(ufunc_T *fp, int i, proftime_T val) { fp->uf_tml_total[i] = val; }

proftime_T nvim_ufunc_get_tml_self_i(const ufunc_T *fp, int i) { return fp->uf_tml_self[i]; }

void nvim_ufunc_set_tml_self_i(ufunc_T *fp, int i, proftime_T val) { fp->uf_tml_self[i] = val; }

// Null checks for profiling arrays
int nvim_ufunc_tml_count_is_null(const ufunc_T *fp) { return fp->uf_tml_count == NULL; }

int nvim_ufunc_tml_total_is_null(const ufunc_T *fp) { return fp->uf_tml_total == NULL; }

int nvim_ufunc_tml_self_is_null(const ufunc_T *fp) { return fp->uf_tml_self == NULL; }

// Array allocation
void nvim_ufunc_alloc_tml_count(ufunc_T *fp, int len) { fp->uf_tml_count = xcalloc((size_t)len, sizeof(int)); }

void nvim_ufunc_alloc_tml_total(ufunc_T *fp, int len) { fp->uf_tml_total = xcalloc((size_t)len, sizeof(proftime_T)); }

void nvim_ufunc_alloc_tml_self(ufunc_T *fp, int len) { fp->uf_tml_self = xcalloc((size_t)len, sizeof(proftime_T)); }

// FUNCLINE null check
int nvim_ufunc_funcline_is_null(const ufunc_T *fp, int idx) { return FUNCLINE(fp, idx) == NULL; }

// Phase 5: child profiling accessors
funccall_T *nvim_get_current_funccal(void) { return get_current_funccal(); }

proftime_T nvim_fc_get_prof_child(const funccall_T *fc) { return fc->fc_prof_child; }

void nvim_fc_set_prof_child(funccall_T *fc, proftime_T val) { fc->fc_prof_child = val; }

proftime_T nvim_ufunc_get_tm_children(const ufunc_T *fp) { return fp->uf_tm_children; }

void nvim_ufunc_set_tm_children(ufunc_T *fp, proftime_T val) { fp->uf_tm_children = val; }

// Phase 1: Function Listing Accessors
// (nvim_ufunc_get_name and nvim_ufunc_get_name_exp already exist in runtime_ffi.c)

size_t nvim_ufunc_get_namelen(const ufunc_T *fp) { return fp ? fp->uf_namelen : 0; }

int nvim_ufunc_get_args_len(const ufunc_T *fp)
{
  return fp ? fp->uf_args.ga_len : 0;
}

const char *nvim_ufunc_get_arg(const ufunc_T *fp, int i)
{
  if (fp == NULL || i < 0 || i >= fp->uf_args.ga_len) {
    return NULL;
  }
  return ((char **)(fp->uf_args.ga_data))[i];
}

int nvim_ufunc_get_def_args_len(const ufunc_T *fp)
{
  return fp ? fp->uf_def_args.ga_len : 0;
}

const char *nvim_ufunc_get_def_arg(const ufunc_T *fp, int i)
{
  if (fp == NULL || i < 0 || i >= fp->uf_def_args.ga_len) {
    return NULL;
  }
  return ((char **)(fp->uf_def_args.ga_data))[i];
}

int nvim_ufunc_get_varargs(const ufunc_T *fp) { return fp ? fp->uf_varargs : 0; }

sctx_T nvim_ufunc_get_script_ctx(const ufunc_T *fp)
{
  if (fp == NULL) {
    sctx_T empty = { 0, 0, 0, 0 };
    return empty;
  }
  return fp->uf_script_ctx;
}

const char *nvim_ufunc_get_funcline(const ufunc_T *fp, int i)
{
  if (fp == NULL || i < 0 || i >= fp->uf_lines.ga_len) {
    return NULL;
  }
  return FUNCLINE(fp, i);
}

// nvim_func_ht_array -- dead, no Rust callers (Phase 17)

int nvim_func_ht_changed(void) { return func_hashtab.ht_changed; }

int nvim_ufunc_name_refcount(const char *name)
{
  return rs_func_name_refcount(name) ? 1 : 0;
}

// nvim_func_hi_to_uf -- dead, no Rust callers (Phase 17)

void nvim_func_ht_foreach(void (*cb)(ufunc_T *fp, void *ctx), void *ctx)
{
  size_t todo = func_hashtab.ht_used;
  for (const hashitem_T *hi = func_hashtab.ht_array; todo > 0; hi++) {
    if (!HASHITEM_EMPTY(hi)) {
      cb(HI2UF(hi), ctx);
      todo--;
    }
  }
}

/// List functions matching regexp pattern string (for rs_list_functions_matching_pat).
void nvim_list_functions_matching_pat(const char *pat, bool ic)
{
  regmatch_T regmatch;
  regmatch.regprog = vim_regcomp(pat, RE_MAGIC);
  if (regmatch.regprog != NULL) {
    regmatch.rm_ic = ic;
    list_functions(&regmatch);
    vim_regfree(regmatch.regprog);
  }
}

// exarg_T field accessors: nvim_eap_get_arg, nvim_eap_get_skip, nvim_eap_get_forceit,
// nvim_eap_set_nextcmd already defined in ex_docmd.c and indent_ffi.c.

// Translated error messages for Rust (emsg wrappers with _(...)  gettext)
void nvim_emsg_function_list_modified(void) { emsg(_(e_function_list_was_modified)); }
void nvim_emsg_undefined_function(const char *name) { emsg_funcname(N_("E123: Undefined function: %s"), name); }
void nvim_emsg_trailing_arg(const char *name) { semsg(_(e_trailing_arg), name); }

// Phase 2: Function Name Translation Accessors
int nvim_script_id_valid(int sid) { return sid > 0 && sid <= script_items.ga_len; }
void nvim_emsg_usingsid(void) { emsg(_(e_usingsid)); }

// Phase 3: Defer Infrastructure Accessors
funccall_T *nvim_fc_get_caller(funccall_T *fc) { return fc ? fc->fc_caller : NULL; }
funccal_entry_T *nvim_funccal_stack_head(void) { return funccal_stack; }
funccall_T *nvim_funccal_entry_top(funccal_entry_T *fce) { return fce ? fce->top_funccal : NULL; }
funccal_entry_T *nvim_funccal_entry_next(funccal_entry_T *fce) { return fce ? fce->next : NULL; }

void nvim_fc_defer_append(funccall_T *fc, char *name, int argcount, typval_T *argvars)
{
  if (fc->fc_defer.ga_itemsize == 0) {
    ga_init(&fc->fc_defer, sizeof(defer_T), 10);
  }
  defer_T *dr = GA_APPEND_VIA_PTR(defer_T, &fc->fc_defer);
  dr->dr_name = name;
  dr->dr_argcount = argcount;
  for (int i = 0; i < argcount; i++) {
    dr->dr_argvars[i] = argvars[i];
  }
}

void nvim_emsg_defer_not_in_function(void)
{
  semsg(_(e_str_not_inside_function), "defer");
}

// Phase 4: Function Reference Counting Accessors
int nvim_ufunc_decrement_refcount(ufunc_T *fp) { return fp ? --fp->uf_refcount : 0; }
void nvim_ufunc_increment_refcount(ufunc_T *fp) { if (fp) { fp->uf_refcount++; } }
int nvim_ufunc_get_calls(const ufunc_T *fp) { return fp ? fp->uf_calls : 0; }

// Phase 6: Cookie Accessor Shims (funccall_T field access for Rust)
int nvim_fc_get_returned(const funccall_T *fc) { return fc ? fc->fc_returned : 0; }
int nvim_fc_get_level(const funccall_T *fc) { return fc ? fc->fc_level : 0; }
linenr_T *nvim_fc_get_breakpoint_ptr(funccall_T *fc) { return fc ? &fc->fc_breakpoint : NULL; }
int *nvim_fc_get_dbg_tick_ptr(funccall_T *fc) { return fc ? &fc->fc_dbg_tick : NULL; }

// Phase 9: ufunc_T accessors for inlining nvim_func_clear_impl in Rust
int nvim_ufunc_get_cleared(const ufunc_T *fp) { return fp ? fp->uf_cleared : 0; }
void nvim_ufunc_set_cleared(ufunc_T *fp, int v) { if (fp) { fp->uf_cleared = v; } }
funccall_T *nvim_ufunc_get_scoped(const ufunc_T *fp) { return fp ? fp->uf_scoped : NULL; }

// Phase 9: ufunc_T accessor for inlining nvim_func_free_impl
void nvim_ufunc_clear_name_exp(ufunc_T *fp) { if (fp) { XFREE_CLEAR(fp->uf_name_exp); } }

// Phase 6: Internal function arity accessor for Rust get_func_arity
// Returns 1 if found (sets *required and *optional), 0 if not a builtin.
int nvim_get_internal_func_arity(const char *name, int *required, int *optional)
{
  const EvalFuncDef *fdef = find_internal_func(name);
  if (fdef == NULL) {
    return 0;
  }
  *required = fdef->min_argc;
  *optional = fdef->max_argc - fdef->min_argc;
  return 1;
}

// Phase 10: partial_T accessor shims for Rust make_partial
char *nvim_partial_get_name(const partial_T *pt) { return pt ? pt->pt_name : NULL; }
typval_T *nvim_partial_get_argv(const partial_T *pt) { return pt ? pt->pt_argv : NULL; }
void nvim_partial_set_refcount(partial_T *pt, int v) { if (pt) { pt->pt_refcount = v; } }
void nvim_partial_set_dict(partial_T *pt, dict_T *d) { if (pt) { pt->pt_dict = d; } }
void nvim_partial_set_auto(partial_T *pt, int v) { if (pt) { pt->pt_auto = (bool)v; } }
void nvim_partial_set_name(partial_T *pt, char *name) { if (pt) { pt->pt_name = name; } }
void nvim_partial_set_func(partial_T *pt, ufunc_T *fp) { if (pt) { pt->pt_func = fp; } }
void nvim_partial_set_argv(partial_T *pt, typval_T *argv) { if (pt) { pt->pt_argv = argv; } }
void nvim_partial_set_argc(partial_T *pt, int argc) { if (pt) { pt->pt_argc = argc; } }
// nvim_tv_get_vstring_mut -- dead, no callers (Phase 17)

// Phase 12: funccall_T field accessors for inlining GC shims into Rust
funccall_T *nvim_get_previous_funccal(void) { return previous_funccal; }
void nvim_set_fc_copyID(funccall_T *fc, int v) { if (fc) { fc->fc_copyID = v; } }
int nvim_get_fc_copyID(const funccall_T *fc) { return fc ? fc->fc_copyID : 0; }
int nvim_get_fc_refcount(const funccall_T *fc) { return fc ? fc->fc_refcount : 0; }
// Sub-struct field accessors (refcount / copyID from fc_l_varlist, fc_l_vars, fc_l_avars)
int nvim_fc_varlist_lv_refcount(const funccall_T *fc) { return fc ? fc->fc_l_varlist.lv_refcount : 0; }
int nvim_fc_l_vars_dv_refcount(const funccall_T *fc) { return fc ? fc->fc_l_vars.dv_refcount : 0; }
int nvim_fc_l_avars_dv_refcount(const funccall_T *fc) { return fc ? fc->fc_l_avars.dv_refcount : 0; }
int nvim_fc_varlist_lv_copyID(const funccall_T *fc) { return fc ? fc->fc_l_varlist.lv_copyID : 0; }
int nvim_fc_l_vars_dv_copyID(const funccall_T *fc) { return fc ? fc->fc_l_vars.dv_copyID : 0; }
int nvim_fc_l_avars_dv_copyID(const funccall_T *fc) { return fc ? fc->fc_l_avars.dv_copyID : 0; }
hashtab_T *nvim_fc_l_vars_ht(funccall_T *fc) { return fc ? &fc->fc_l_vars.dv_hashtab : NULL; }
hashtab_T *nvim_fc_l_avars_ht(funccall_T *fc) { return fc ? &fc->fc_l_avars.dv_hashtab : NULL; }
list_T *nvim_fc_l_varlist(funccall_T *fc) { return fc ? &fc->fc_l_varlist : NULL; }
// nvim_get_current_funccal already defined above (line 3517)
// funcargs garray accessors for set_ref_in_func_args
int nvim_funcargs_len(void) { return funcargs.ga_len; }
typval_T *nvim_funcargs_item(int i) { return ((typval_T **)funcargs.ga_data)[i]; }

// Phase 13: C accessors for inlining more impl shims into Rust
// funccal_T sub-struct pointers (for scope and funccal shims)
dict_T *nvim_fc_l_vars_dict(funccall_T *fc) { return fc ? &fc->fc_l_vars : NULL; }
dict_T *nvim_fc_l_avars_dict(funccall_T *fc) { return fc ? &fc->fc_l_avars : NULL; }
ScopeDictDictItem *nvim_fc_l_vars_var_ptr(funccall_T *fc) { return fc ? &fc->fc_l_vars_var : NULL; }
ScopeDictDictItem *nvim_fc_l_avars_var_ptr(funccall_T *fc) { return fc ? &fc->fc_l_avars_var : NULL; }
void nvim_list_hashtable_vars(hashtab_T *ht, const char *prefix, int *first)
{
  list_hashtable_vars(ht, prefix, false, first);
}
// funccall_T setter for current_funccal
void nvim_set_current_funccal(funccall_T *fc) { current_funccal = fc; }
// funccal_entry_T setters
void nvim_fc_entry_set_top(funccal_entry_T *fce, funccall_T *fc) { if (fce) { fce->top_funccal = fc; } }
void nvim_fc_entry_set_next(funccal_entry_T *fce, funccal_entry_T *next) { if (fce) { fce->next = next; } }
funccal_entry_T *nvim_funccal_stack_head_mut(void) { return funccal_stack; }
void nvim_set_funccal_stack(funccal_entry_T *entry) { funccal_stack = entry; }
// fc_ufuncs garray accessors for free_funccal
int nvim_fc_ufuncs_len(const funccall_T *fc) { return fc ? fc->fc_ufuncs.ga_len : 0; }
ufunc_T *nvim_fc_ufuncs_item(const funccall_T *fc, int i)
{
  return fc ? ((ufunc_T **)(fc->fc_ufuncs.ga_data))[i] : NULL;
}
void nvim_fc_ufuncs_ga_clear(funccall_T *fc) { if (fc) { ga_clear(&fc->fc_ufuncs); } }
void nvim_ufunc_set_scoped(ufunc_T *fp, funccall_T *fc) { if (fp) { fp->uf_scoped = fc; } }
// ufunc profiling / lua accessors for func_clear_items
garray_T *nvim_ufunc_get_args_ga(ufunc_T *fp) { return fp ? &fp->uf_args : NULL; }
garray_T *nvim_ufunc_get_def_args_ga(ufunc_T *fp) { return fp ? &fp->uf_def_args : NULL; }
garray_T *nvim_ufunc_get_lines_ga(ufunc_T *fp) { return fp ? &fp->uf_lines : NULL; }
// nvim_ufunc_get_luaref is defined in mapping.c
void nvim_ufunc_clear_luaref(ufunc_T *fp)
{
  if (fp && (fp->uf_flags & FC_LUAREF)) {
    api_free_luaref(fp->uf_luaref);
    fp->uf_luaref = LUA_NOREF;
  }
}
// debug_backtrace_level accessor for inlining nvim_get_funccal_impl
int nvim_get_debug_backtrace_level(void) { return debug_backtrace_level; }
void nvim_set_debug_backtrace_level(int v) { debug_backtrace_level = v; }
// emsg_funcname helper for Rust: concat "<SNR>" + name+3 if K_SPECIAL
char *nvim_emsg_funcname_mk_snr(const char *name)
{
  if ((uint8_t)name[0] == K_SPECIAL && name[1] != NUL && name[2] != NUL) {
    return concat_str("<SNR>", name + 3);
  }
  return NULL;
}
void nvim_semsg_with_name(const char *errmsg, const char *name) { semsg(_(errmsg), name); }
void nvim_iemsg(const char *msg) { iemsg(msg); }
// Phase 14: func_clear_items helpers for Rust refcount.rs
void nvim_ga_clear_strings_wrapper(garray_T *ga) { ga_clear_strings(ga); }
void nvim_ufunc_xfree_tml(ufunc_T *fp)
{
  XFREE_CLEAR(fp->uf_tml_count);
  XFREE_CLEAR(fp->uf_tml_total);
  XFREE_CLEAR(fp->uf_tml_self);
}
// Phase 14: user_func_error semsg helper (for FCERR_UNKNOWN case)
void nvim_semsg_not_callable(const char *name) { semsg(_(e_not_callable_type_str), name); }
void nvim_fc_set_rettv(funccall_T *fc, typval_T *rettv) { if (fc) { fc->fc_rettv = rettv; } }
void nvim_fc_set_func(funccall_T *fc, ufunc_T *fp) { if (fc) { fc->fc_func = fp; } }
size_t nvim_sizeof_funccall(void) { return sizeof(funccall_T); }
void nvim_fc_set_caller(funccall_T *fc, funccall_T *caller) { if (fc) { fc->fc_caller = caller; } }

// Phase 16: shim for call_simple_func migration (call_user_func_check is static)
int nvim_call_user_func_check_simple(ufunc_T *fp, typval_T *argvars, typval_T *rettv)
{
  funcexe_T funcexe = FUNCEXE_INIT;
  funcexe.fe_evaluate = true;
  return call_user_func_check(fp, 0, argvars, rettv, &funcexe, NULL);
}

// Phase 20: shims for one_function_arg + get_function_args migration (parsing.rs)
// Phase 20: shims for get_function_args migration (parsing.rs)
void nvim_semsg_e125_illegal_arg(const char *arg) { semsg(_("E125: Illegal argument: %s"), arg); }
void nvim_semsg_e853_duplicate_arg(const char *arg) { semsg(_("E853: Duplicate argument name: %s"), arg); }
void nvim_emsg_e989_nondefault_follows(void) { emsg(_("E989: Non-default argument follows default argument")); }
void nvim_semsg_no_white_before_comma(const char *p)
{
  semsg(_(e_no_white_space_allowed_before_str_str), ",", p);
}
// nvim_semsg_invarg2 already exists in nvim-match Rust crate -- reuse directly

// Phase 21: accessors for call_user_func_check migration (funccal.rs)
// nvim_ufunc_get_luaref already exists in mapping.c
bool *nvim_funcexe_get_doesrange(funcexe_T *fe) { return fe ? fe->fe_doesrange : NULL; }
linenr_T nvim_funcexe_get_firstline(const funcexe_T *fe) { return fe ? fe->fe_firstline : 0; }
linenr_T nvim_funcexe_get_lastline(const funcexe_T *fe) { return fe ? fe->fe_lastline : 0; }

// Phase 22: accessors for call_func migration (funccal.rs)
void nvim_tv_set_unknown(typval_T *tv) { tv->v_type = VAR_UNKNOWN; }

// Phase 23: accessors for get_func_tv migration (funccal.rs)
bool nvim_evalarg_should_evaluate(const evalarg_T *ea) { return ea ? (ea->eval_flags & EVAL_EVALUATE) != 0 : false; }
int nvim_funcargs_ga_itemsize(void) { return funcargs.ga_itemsize; }
void nvim_funcargs_ga_init(void) { ga_init(&funcargs, (int)sizeof(typval_T *), 50); }
void nvim_funcargs_ga_grow(void) { ga_grow(&funcargs, 1); }
void nvim_funcargs_push_tv_ptr(typval_T *tv) { ((typval_T **)funcargs.ga_data)[funcargs.ga_len++] = tv; }
void nvim_funcargs_dec_len(int n) { funcargs.ga_len -= n; }
int nvim_get_testing_flag(void) { return get_vim_var_nr(VV_TESTING); }
void nvim_emsg_e740_too_many_args(const char *name) { emsg_funcname(N_("E740: Too many arguments for function %s"), name); }
void nvim_emsg_e116_invalid_args(const char *name) { emsg_funcname(N_("E116: Invalid arguments for function %s"), name); }
dict_T *nvim_funcexe_get_selfdict(const funcexe_T *fe) { return fe ? fe->fe_selfdict : NULL; }
partial_T *nvim_funcexe_get_partial(const funcexe_T *fe) { return fe ? fe->fe_partial : NULL; }
bool nvim_funcexe_get_evaluate(const funcexe_T *fe) { return fe ? fe->fe_evaluate : false; }
typval_T *nvim_funcexe_get_basetv(const funcexe_T *fe) { return fe ? fe->fe_basetv : NULL; }
bool nvim_funcexe_get_found_var(const funcexe_T *fe) { return fe ? fe->fe_found_var : false; }
/// Call fe_argv_func if non-null; returns unchanged argcount otherwise.
int nvim_funcexe_call_argv_func(funcexe_T *fe, int argcount, typval_T *argvars,
                                int argv_clear, ufunc_T *fp)
{
  if (fe && fe->fe_argv_func) {
    return fe->fe_argv_func(argcount, argvars, argv_clear, fp);
  }
  return argcount;
}
bool nvim_partial_get_auto(const partial_T *pt) { return pt ? pt->pt_auto : false; }

// Phase 24: accessors for nvim_ex_defer_inner_impl migration (defer.rs)
void nvim_emsg_cannot_use_partial_with_dict(void)
{
  emsg(_(e_cannot_use_partial_with_dictionary_for_defer));
}
/// Returns 0 if builtin function check passes, -1 if it fails.
int nvim_check_defer_builtin(const char *name, int argcount)
{
  const EvalFuncDef *const fdef = find_internal_func(name);
  if (fdef == NULL) {
    emsg_funcname(e_unknown_function_str, name);
    return -1;
  }
  return check_internal_func(fdef, argcount);
}

// Phase 26: accessor for free_unref_funccal migration (gc.rs)
void nvim_set_previous_funccal(funccall_T *fc) { previous_funccal = fc; }

// Phase 27: accessors for funccal_unref migration (funccal.rs)
int nvim_fc_decrement_refcount(funccall_T *fc) { return fc ? --fc->fc_refcount : 0; }
void nvim_fc_ufuncs_null_matching(funccall_T *fc, ufunc_T *fp)
{
  if (fc == NULL) {
    return;
  }
  for (int i = 0; i < fc->fc_ufuncs.ga_len; i++) {
    if (((ufunc_T **)(fc->fc_ufuncs.ga_data))[i] == fp) {
      ((ufunc_T **)(fc->fc_ufuncs.ga_data))[i] = NULL;
    }
  }
}

// Phase 25: accessors for ex_call_inner migration (funccal.rs)
linenr_T nvim_eap_get_line1(const exarg_T *eap) { return eap ? eap->line1 : 0; }
linenr_T nvim_eap_get_line2(const exarg_T *eap) { return eap ? eap->line2 : 0; }
int nvim_eap_get_addr_count(const exarg_T *eap) { return eap ? eap->addr_count : 0; }
/// Check lnum vs curbuf line count. If valid, advance cursor (lnum, col=0, coladd=0).
/// Returns 1 if lnum is beyond line count (break loop), 0 otherwise.
int nvim_ex_call_check_advance_cursor(linenr_T lnum)
{
  if (lnum > curbuf->b_ml.ml_line_count) {
    return 1;
  }
  curwin->w_cursor.lnum = lnum;
  curwin->w_cursor.col = 0;
  curwin->w_cursor.coladd = 0;
  return 0;
}
/// Call handle_subscript with &EVALARG_EVALUATE.
int nvim_handle_subscript_eval_evaluate(char **arg, typval_T *rettv)
{
  return handle_subscript((const char **)arg, rettv, &EVALARG_EVALUATE, true);
}
