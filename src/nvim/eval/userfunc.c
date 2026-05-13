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

extern void rs_list_hashtable_vars(hashtab_T *ht, const char *prefix, int empty, int *first);

// Phase 1: Function Listing (implemented in Rust userfunc/src/listing.rs)
extern int rs_cat_func_name(char *buf, size_t bufsize, ufunc_T *fp);
extern int rs_function_list_modified(int prev_ht_changed);
extern int rs_list_func_head(ufunc_T *fp, int indent, int force);
extern void rs_list_functions(void);
extern void rs_list_functions_regmatch(regmatch_T *regmatch);
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
extern char *get_func_line(int c, void *cookie, int indent, bool do_concat);

// Phase 1 (plan db85cc6b): Small hashtab foundations (implemented in Rust userfunc/src/hashtab.rs)
extern void rs_register_closure(ufunc_T *fp);
extern void rs_add_nr_var(dict_T *dp, dictitem_T *v, const char *name, int64_t nr);

// Phase 2 (plan db85cc6b): lambda / alloc_ufunc / register_luafunc (Rust userfunc/src/lambda.rs)
extern size_t rs_get_lambda_name(char *buf, size_t bufsize);
extern ufunc_T *rs_alloc_ufunc(const char *name, size_t namelen);

// Phase 5 (plan db85cc6b): get_lambda_tv (Rust userfunc/src/lambda.rs)
extern int rs_get_lambda_tv(char **arg, typval_T *rettv, evalarg_T *evalarg);

// Phase 6 (plan db85cc6b): free_all_functions (Rust userfunc/src/teardown.rs)
#if defined(EXITFREE)
extern void rs_free_all_functions(void);
#endif

// Phase 7 (plan db85cc6b): get_function_body (Rust userfunc/src/body.rs)
extern int rs_get_function_body(exarg_T *eap, garray_T *newlines, char *line_arg_in,
                                char **line_to_free, bool show_block);

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
// Wave 2 Phase 4: sequential number for nameless dict functions (was local to ex_function)
static int func_nr = 0;
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
/// Logic lives in Rust (hashtab.rs Phase 1). Thin wrapper for C callers.
static void register_closure(ufunc_T *fp) { rs_register_closure(fp); }

static char lambda_name[8 + NUMBUFLEN];

/// @return  a name for a lambda.  Returned in static memory.
/// Logic (counter) lives in Rust (lambda.rs Phase 2). Thin wrapper for C callers.
static String get_lambda_name(void)
{
  size_t len = rs_get_lambda_name(lambda_name, sizeof(lambda_name));
  return cbuf_as_string(lambda_name, len);
}

/// Allocate a "ufunc_T" for a function called "name".
/// Logic lives in Rust (lambda.rs Phase 2). Thin wrapper for C callers.
static ufunc_T *alloc_ufunc(const char *name, size_t namelen)
{
  return rs_alloc_ufunc(name, namelen);
}

/// Parse a lambda expression and get a Funcref from "*arg".
/// Logic lives in Rust (lambda.rs Phase 5). Thin wrapper for C callers.
///
/// @return OK or FAIL.  Returns NOTDONE for dict or {expr}.
int get_lambda_tv(char **arg, typval_T *rettv, evalarg_T *evalarg)
{
  return rs_get_lambda_tv(arg, rettv, evalarg);
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

// find_func migrated to Rust (hashtab.rs Wave 2 Phase 1). Symbol provided by libnvim_rs.a.
extern ufunc_T *find_func(const char *name);

/// Add a number variable "name" to dict "dp" with value "nr".
/// Logic lives in Rust (hashtab.rs Phase 1). Thin wrapper for C callers.
static void add_nr_var(dict_T *dp, dictitem_T *v, char *name, varnumber_T nr)
{
  rs_add_nr_var(dp, v, name, (int64_t)nr);
}

// Phase 7-31: free_funccal, free_funccal_contents, cleanup_function_call, funccal_unref migrated to Rust.

// nvim_func_remove_impl deleted: logic inlined into rs_func_remove_ht (hashtab.rs Phase 1)

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
/// Free all user-defined functions and clean up the funccal stack.
/// Logic lives in Rust (teardown.rs Phase 6). Thin wrapper for EXITFREE build.
void free_all_functions(void)
{
  rs_free_all_functions();
}
#endif


// func_call migrated to Rust (funccal.rs Phase 35).
// rs_func_call now implements the logic directly.
extern int func_call(char *name, typval_T *args, partial_T *partial, dict_T *selfdict,
                     typval_T *rettv);

// Phase 14-15: callback_call_retnr, user_func_error migrated to Rust.
// argv_add_base migrated to Rust (lookup.rs Phase 18)
extern void argv_add_base(typval_T *const basetv, typval_T **const argvars, int *const argcount,
                          typval_T *const new_argvars, int *const argv_base);

// call_func migrated to Rust (Phase 22, funccal.rs)
extern int call_func(const char *funcname, int len, typval_T *rettv, int argcount_in,
                     typval_T *argvars_in, funcexe_T *funcexe);

// call_simple_luafunc and call_simple_func migrated to Rust (Phase 16, funccal.rs)

// Wave 2 Phase 3: trans_function_name migrated to Rust (names.rs).

// list_functions deleted (Phase 4): logic moved to Rust rs_list_functions and
// rs_list_functions_regmatch (listing.rs).

/// Read the body of a function, put every line in "newlines".
/// This stops at "endfunction".
/// "newlines" must already have been initialized.
/// Phase 7 (plan db85cc6b): body moved to Rust rs_get_function_body (body.rs).
// No longer static: Rust (excmd.rs) calls this as an extern C function.
int get_function_body(exarg_T *eap, garray_T *newlines, char *line_arg_in,
                      char **line_to_free, bool show_block)
{
  return rs_get_function_body(eap, newlines, line_arg_in, line_to_free, show_block);
}

// Wave 2 Phase 4: ex_function migrated to Rust (excmd.rs).

// get_user_func_name migrated to Rust (expand.rs Phase 3).
// rs_get_user_func_name now implements the logic directly via export_name = "get_user_func_name".

/// Phase 7: C implementation shim for ex_delfunction (called from Rust).
// nvim_ex_delfunction_impl migrated to Rust (funccal.rs Phase 32).
// rs_ex_delfunction now implements the logic directly.

/// ":delfunction {name}"
/// Check whether funccall is still referenced outside
///
/// It is supposed to be referenced if either it is referenced itself or if l:,
/// a: or a:000 are referenced as all these are statically allocated within
/// funccall structure.
// nvim_fc_referenced_impl inlined into rs_fc_referenced (Rust, Phase 12)

// nvim_can_free_funccal_impl inlined into rs_can_free_funccal (Rust, Phase 12)

// nvim_ex_return_impl migrated to Rust (funccal.rs Phase 28).
// rs_ex_return_impl now implements the logic directly.

/// ":return [expr]"
/// Lower level implementation of "call".  Only called when not skipping.
// ex_call_inner migrated to Rust (funccal.rs Phase 25).
extern int ex_call_inner(exarg_T *eap, char *name, char **arg, char *startarg,
                         const funcexe_T *funcexe_init, evalarg_T *evalarg);

// nvim_ex_defer_inner_impl migrated to Rust (defer.rs Phase 24).
// rs_ex_defer_inner now implements the logic directly.


// nvim_handle_defer_one_impl migrated to Rust (defer.rs Phase 30).
// rs_handle_defer_one now implements the logic directly.


// ex_call migrated to Rust (excmd.rs Wave 2 Phase 2). Symbol provided by libnvim_rs.a.
extern void ex_call(exarg_T *eap);

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
// nvim_do_return_impl migrated to Rust (scope.rs Phase 34).
// rs_do_return now implements the logic directly.

// nvim_get_return_cmd_impl migrated to Rust (scope.rs Phase 34).
// rs_get_return_cmd now implements the logic directly.

/// Get next function line.
/// Called by do_cmdline() to get the next line.
///
/// @return  allocated string, or NULL for end of function.
// get_func_line migrated to Rust (funccal.rs Phase 33).
// rs_get_func_line now implements the logic directly.

// func_has_ended, func_has_abort migrated to Rust (lookup.rs Phase 6)

// make_partial migrated to Rust (partial.rs Phase 10)

// func_name, func_breakpoint, func_dbg_tick, func_level migrated to Rust (lookup.rs Phase 6)

// nvim_get_current_funccal_fc_returned inlined into rs_current_func_returned (eval/src/lib.rs Phase 36)

// nvim_free_unref_funccal_impl migrated to Rust (gc.rs Phase 26).
// rs_free_unref_funccal now implements the logic directly.

// Phase 12-13: funccal scope accessors inlined into Rust (scope.rs).
// Phase 12: GC funccal impls inlined into Rust (gc.rs).
// Phase 29: scoped_ht impls inlined into Rust (scope.rs).

// nvim_set_ref_in_functions_impl deleted: logic inlined into rs_set_ref_in_functions_inner
// (hashtab.rs Phase 1) using nvim_func_ht_foreach.

// nvim_set_ref_in_func_args_impl inlined into rs_set_ref_in_func_args (Rust, Phase 12)

// nvim_set_ref_in_func_impl inlined into rs_set_ref_in_func (Rust, Phase 11)

// register_luafunc migrated to Rust (lambda.rs Phase 2).
// rs_register_luafunc now implements the logic directly via export_name = "register_luafunc".

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

// apply_autocmds_for_funcundefined migrated to Rust (hashtab.rs Wave 2 Phase 1).
extern int apply_autocmds_for_funcundefined(const char *name);

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
    rs_list_functions_regmatch(&regmatch);
    vim_regfree(regmatch.regprog);
  }
}

// exarg_T field accessors: see Phase 28 accessors at the bottom of this file.

// Translated error messages for Rust (emsg wrappers with _(...)  gettext)
void nvim_emsg_function_list_modified(void) { emsg(_(e_function_list_was_modified)); }
void nvim_emsg_undefined_function(const char *name) { emsg_funcname(N_("E123: Undefined function: %s"), name); }
void nvim_emsg_trailing_arg(const char *name) { semsg(_(e_trailing_arg), name); }
// Phase 5 (plan db85cc6b): get_lambda_tv error messages
void nvim_semsg_e451_expected_cbrace(const char *p) { semsg(_("E451: Expected }: %s"), p); }

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
  rs_list_hashtable_vars(ht, prefix, false, first);
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

// Phase 28: accessors for nvim_ex_return_impl migration (funccal.rs)
char *nvim_eap_get_arg(const exarg_T *eap) { return eap ? eap->arg : NULL; }
int nvim_eap_get_skip(const exarg_T *eap) { return eap ? eap->skip : 0; }
void nvim_eap_set_nextcmd(exarg_T *eap, char *val) { if (eap) { eap->nextcmd = val; } }
char *nvim_eap_get_nextcmd(const exarg_T *eap) { return eap ? eap->nextcmd : NULL; }
void nvim_emsg_return_not_in_func(void) { emsg(_("E133: :return not inside a function")); }

// Phase 30: accessors for nvim_handle_defer_one_impl migration (defer.rs)
int nvim_fc_defer_ga_len(const funccall_T *fc) { return fc ? fc->fc_defer.ga_len : 0; }
/// Takes dr_name from slot idx (returns it, sets to NULL).
char *nvim_fc_defer_item_take_name(funccall_T *fc, int idx)
{
  if (!fc) {
    return NULL;
  }
  defer_T *dr = ((defer_T *)fc->fc_defer.ga_data) + idx;
  char *name = dr->dr_name;
  dr->dr_name = NULL;
  return name;
}
int nvim_fc_defer_item_argcount(const funccall_T *fc, int idx)
{
  if (!fc) {
    return 0;
  }
  return ((const defer_T *)fc->fc_defer.ga_data)[idx].dr_argcount;
}
typval_T *nvim_fc_defer_item_argvars(funccall_T *fc, int idx)
{
  if (!fc) {
    return NULL;
  }
  return ((defer_T *)fc->fc_defer.ga_data)[idx].dr_argvars;
}
void nvim_fc_defer_ga_clear(funccall_T *fc)
{
  if (fc) {
    ga_clear(&fc->fc_defer);
  }
}

// Phase 31: accessors for nvim_free_funccal_contents_impl and nvim_cleanup_function_call_impl (funccal.rs)
void nvim_fc_l_vars_ht_clear(funccall_T *fc)
{
  if (fc) {
    vars_clear(&fc->fc_l_vars.dv_hashtab);
  }
}
void nvim_fc_l_avars_ht_clear(funccall_T *fc)
{
  if (fc) {
    vars_clear(&fc->fc_l_avars.dv_hashtab);
  }
}
void nvim_fc_l_varlist_tv_clear_all(funccall_T *fc)
{
  if (fc) {
    TV_LIST_ITER(&fc->fc_l_varlist, li, {
      tv_clear(TV_LIST_ITEM_TV(li));
    });
  }
}
void nvim_fc_pop_current_funccal(funccall_T *fc)
{
  if (fc) {
    current_funccal = fc->fc_caller;
  }
}
void nvim_fc_l_avars_ht_clear_ext_false(funccall_T *fc)
{
  if (fc) {
    vars_clear_ext(&fc->fc_l_avars.dv_hashtab, false);
  }
}
void nvim_fc_l_avars_tv_copy_all(funccall_T *fc)
{
  if (fc) {
    TV_DICT_ITER(&fc->fc_l_avars, di, {
      tv_copy(&di->di_tv, &di->di_tv);
    });
  }
}
void nvim_fc_l_varlist_set_lv_first_null(funccall_T *fc)
{
  if (fc) {
    fc->fc_l_varlist.lv_first = NULL;  // NOLINT(runtime/deprecated)
  }
}
void nvim_fc_l_varlist_tv_copy_all(funccall_T *fc)
{
  if (fc) {
    TV_LIST_ITER(&fc->fc_l_varlist, li, {
      tv_copy(TV_LIST_ITEM_TV(li), TV_LIST_ITEM_TV(li));
    });
  }
}
/// Called when fc cannot be freed immediately.
/// Links fc into the previous_funccal list and handles the GC threshold logic.
void nvim_cleanup_function_call_put_in_prev_list(funccall_T *fc)
{
  if (!fc) {
    return;
  }
  static int made_copy = 0;
  fc->fc_caller = previous_funccal;
  previous_funccal = fc;
  if (want_garbage_collect) {
    made_copy = 0;
  } else if (++made_copy >= (int)((4096 * 1024) / sizeof(*fc))) {
    made_copy = 0;
    want_garbage_collect = true;
  }
}

// Phase 32: accessors for nvim_ex_delfunction_impl migration (funccal.rs)
int nvim_eap_get_forceit_int(const exarg_T *eap) { return eap ? eap->forceit : 0; }
int nvim_ufunc_get_refcount(const ufunc_T *fp) { return fp ? fp->uf_refcount : 0; }
void nvim_ufunc_or_flags_deleted(ufunc_T *fp) { if (fp) { fp->uf_flags |= FC_DELETED; } }
dict_T *nvim_fudi_get_dict(const funcdict_T *fudi) { return fudi ? fudi->fd_dict : NULL; }
char *nvim_fudi_get_newkey(const funcdict_T *fudi) { return fudi ? fudi->fd_newkey : NULL; }
dictitem_T *nvim_fudi_get_di(const funcdict_T *fudi) { return fudi ? fudi->fd_di : NULL; }
void nvim_tv_dict_item_remove(dict_T *dict, dictitem_T *di) { tv_dict_item_remove(dict, di); }
void nvim_emsg_funcref(void) { emsg(_(e_funcref)); }
int nvim_ends_excmd_skipwhite(const char *p) { return ends_excmd(*skipwhite(p)); }
void nvim_semsg_e_invarg2(const char *arg) { semsg(_(e_invarg2), arg); }
void nvim_semsg_nofunc(const char *arg) { semsg(_(e_nofunc), arg); }
void nvim_semsg_e131_in_use(const char *arg)
{
  semsg(_("E131: Cannot delete function %s: It is in use"), arg);
}
void nvim_semsg_cannot_delete_internal(const char *arg)
{
  semsg(_("Cannot delete function %s: It is being used internally"), arg);
}

// Phase 33: accessors for get_func_line migration (funccal.rs)
int nvim_fc_get_linenr(const funccall_T *fc) { return fc ? fc->fc_linenr : 0; }
void nvim_fc_set_linenr(funccall_T *fc, int v) { if (fc) { fc->fc_linenr = v; } }
// Increment fc_linenr and return the post-increment value.
int nvim_fc_postincrement_linenr(funccall_T *fc) { return fc ? fc->fc_linenr++ : 0; }

// Phase 34: accessors for nvim_do_return_impl and nvim_get_return_cmd_impl migration (scope.rs)
void nvim_fc_set_returned(funccall_T *fc, int v) { if (fc) { fc->fc_returned = (bool)v; } }
typval_T *nvim_fc_get_rettv(funccall_T *fc) { return fc ? fc->fc_rettv : NULL; }
cstack_T *nvim_eap_get_cstack(const exarg_T *eap) { return eap ? eap->cstack : NULL; }
// Wave 2 Phase 2: ex_call migration accessors
void nvim_eap_set_skip(exarg_T *eap, int v) { if (eap) { eap->skip = (bool)v; } }
int nvim_cstack_get_trylevel(const cstack_T *cs) { return cs ? cs->cs_trylevel : 0; }
int nvim_cmd_defer_idx(void) { return (int)CMD_defer; }
void nvim_semsg_e_missingparen(const char *name) { semsg(_(e_missingparen), name); }
// Wave 2 Phase 3: trans_function_name migration accessors
void nvim_emsg_e129_funcname_required(void) { emsg(_("E129: Function name required")); }
void nvim_semsg_e128_func_start_capital(const char *start)
{
  semsg(_("E128: Function name must start with a capital or \"s:\": %s"), start);
}
void nvim_semsg_e884_func_no_colon(const char *start)
{
  semsg(_("E884: Function name cannot contain a colon: %s"), start);
}
void nvim_semsg_invexpr2_vlua(void) { semsg(e_invexpr2, "v:lua"); }
void nvim_cstack_set_pending(cstack_T *cs, int idx, int val)
{
  if (cs && idx >= 0 && idx < CSTACK_LEN) { cs->cs_pending[idx] = (char)val; }
}
void nvim_cstack_set_rv(cstack_T *cs, int idx, void *val)
{
  if (cs && idx >= 0 && idx < CSTACK_LEN) { cs->cs_rettv[idx] = val; }
}
typval_T *nvim_xcalloc_typval(void) { return xcalloc(1, sizeof(typval_T)); }
void nvim_tv_reset_to_number_zero(typval_T *tv)
{
  if (tv) { tv->v_type = VAR_NUMBER; tv->vval.v_number = 0; }
}
char *nvim_encode_tv2echo(void *tv) { return encode_tv2echo((typval_T *)tv, NULL); }

// Phase 35: accessors for func_call migration (funccal.rs)
/// Iterates args list, copies each typval into argv[].
/// Returns argc on success, -1 on "E699: Too many arguments" (frees copied args).
int nvim_func_call_iter_args(typval_T *args, typval_T *argv, int max_args)
{
  int argc = 0;
  bool overflow = false;
  TV_LIST_ITER(args->vval.v_list, item, {
    if (argc == max_args) {
      emsg(_("E699: Too many arguments"));
      overflow = true;
      break;
    }
    tv_copy(TV_LIST_ITEM_TV(item), &argv[argc++]);
  });
  if (overflow) {
    while (argc > 0) { tv_clear(&argv[--argc]); }
    return -1;
  }
  return argc;
}
linenr_T nvim_curwin_cursor_lnum(void) { return curwin->w_cursor.lnum; }

// Phase 1 (plan db85cc6b): Small hashtab foundations
/// Convert a hashitem key pointer to the owning ufunc_T.
ufunc_T *nvim_hi_key_to_ufunc(const hashitem_T *hi)
{
  return HI2UF(hi);
}

/// Look up a ufunc_T in the global function hashtab by its current HIKEY.
/// Returns NULL if not found.
ufunc_T *nvim_func_ht_find_uf(const ufunc_T *fp)
{
  hashitem_T *hi = hash_find(&func_hashtab, UF2HIKEY(fp));
  if (HASHITEM_EMPTY(hi)) {
    return NULL;
  }
  return HI2UF(hi);
}

/// Remove the hashitem for fp from the global function hashtab.
/// Returns 1 if removed, 0 if not found.
int nvim_func_ht_remove_fp(ufunc_T *fp)
{
  hashitem_T *hi = hash_find(&func_hashtab, UF2HIKEY(fp));
  if (HASHITEM_EMPTY(hi)) {
    return 0;
  }
  hash_remove(&func_hashtab, hi);
  return 1;
}

/// Push fp onto fc->fc_ufuncs (grow by 1).
void nvim_fc_ufuncs_push(funccall_T *fc, ufunc_T *fp)
{
  ga_grow(&fc->fc_ufuncs, 1);
  ((ufunc_T **)fc->fc_ufuncs.ga_data)[fc->fc_ufuncs.ga_len++] = fp;
}

/// Increment fc->fc_refcount.
void nvim_fc_increment_refcount(funccall_T *fc)
{
  if (fc) {
    fc->fc_refcount++;
  }
}

/// Add number variable "name" with value nr to dict dp using dictitem v.
/// Thin wrapper around the C macro-heavy init pattern for Rust callers.
void nvim_add_nr_var(dict_T *dp, dictitem_T *v, const char *name, int64_t nr)
{
  STRCPY(v->di_key, name);
  v->di_flags = DI_FLAGS_RO | DI_FLAGS_FIX;
  hash_add(&dp->dv_hashtab, v->di_key);
  v->di_tv.v_type = VAR_NUMBER;
  v->di_tv.v_lock = VAR_FIXED;
  v->di_tv.vval.v_number = nr;
}

// Phase 2 (plan db85cc6b): alloc_ufunc / get_lambda_name / register_luafunc accessors

/// Size of ufunc_T up to (but not including) uf_name flexible member.
size_t nvim_sizeof_ufunc_header(void)
{
  return offsetof(ufunc_T, uf_name);
}

/// Initialize the name fields of a freshly xcalloc'd ufunc_T.
/// Sets uf_name (copy of name[0..namelen]), uf_namelen, and (if K_SPECIAL prefix)
/// allocates uf_name_exp as "<SNR>...".
void nvim_ufunc_init_name(ufunc_T *fp, const char *name, size_t namelen)
{
  xmemcpyz(fp->uf_name, name, namelen);
  fp->uf_namelen = namelen;
  if ((uint8_t)name[0] == K_SPECIAL) {
    size_t len = namelen + 3;
    fp->uf_name_exp = xmalloc(len);
    snprintf(fp->uf_name_exp, len, "<SNR>%s", fp->uf_name + 3);
  }
}

/// Set the luaref-specific fields on fp (after init_name).
void nvim_ufunc_init_luaref_fields(ufunc_T *fp, LuaRef ref)
{
  fp->uf_refcount = 1;
  fp->uf_varargs = true;
  fp->uf_flags = FC_LUAREF;
  fp->uf_calls = 0;
  fp->uf_script_ctx = current_sctx;
  fp->uf_luaref = ref;
}

/// Add fp to the global function hashtab (hash_add). Returns OK on success, FAIL on collision.
int nvim_func_ht_try_add_fp(ufunc_T *fp) { return hash_add(&func_hashtab, UF2HIKEY(fp)); }
/// Add fp to the global function hashtab (hash_add). Asserts no collision (use for known-unique).
void nvim_func_ht_add_fp(ufunc_T *fp)
{
  hash_add(&func_hashtab, UF2HIKEY(fp));
}

/// Return fp->uf_name (pointer into the flexible member).
char *nvim_ufunc_get_name_ptr(ufunc_T *fp)
{
  return fp ? fp->uf_name : NULL;
}

/// Return current_sctx.
sctx_T nvim_get_current_sctx(void) { return current_sctx; }

// Phase 3 (plan db85cc6b): get_user_func_name accessors

/// Return pointer to func_hashtab.ht_array (first hashitem_T slot).
hashitem_T *nvim_func_ht_array(void) { return func_hashtab.ht_array; }

/// Return func_hashtab.ht_used.
size_t nvim_func_ht_used(void) { return func_hashtab.ht_used; }

/// Convert hashitem_T * to ufunc_T * via HI2UF (same as nvim_hi_key_to_ufunc).
ufunc_T *nvim_hashitem_to_ufunc(const hashitem_T *hi) { return HI2UF(hi); }

/// Return 1 if uf_args is empty (GA_EMPTY), 0 otherwise.
int nvim_ufunc_get_args_empty(const ufunc_T *fp) { return fp ? GA_EMPTY(&fp->uf_args) : 1; }

/// Return IObuff pointer (shared global output buffer).
char *nvim_get_iobuff(void) { return IObuff; }

/// Return IOSIZE.
int nvim_get_iosize_int(void) { return IOSIZE; }

/// Return EXPAND_USER_FUNC constant.
int nvim_expand_user_func(void) { return EXPAND_USER_FUNC; }

/// xstrlcpy(IObuff + offset, src, IOSIZE - offset) wrapper.
void nvim_iobuff_xstrlcpy_at(int offset, const char *src)
{
  xstrlcpy(IObuff + offset, src, (size_t)(IOSIZE - offset));
}

// Phase 5 (plan db85cc6b): get_lambda_tv accessors

/// Set fp->uf_refcount.
void nvim_ufunc_set_refcount(ufunc_T *fp, int v) { if (fp) { fp->uf_refcount = v; } }

/// Set fp->uf_varargs.
void nvim_ufunc_set_varargs(ufunc_T *fp, int v) { if (fp) { fp->uf_varargs = (bool)v; } }

/// Set fp->uf_flags.
void nvim_ufunc_set_flags(ufunc_T *fp, int v) { if (fp) { fp->uf_flags = v; } }

/// Set fp->uf_calls.
void nvim_ufunc_set_calls(ufunc_T *fp, int v) { if (fp) { fp->uf_calls = v; } }

/// Move (struct-assign) *src into fp->uf_args.
void nvim_ufunc_move_args_ga(ufunc_T *fp, const garray_T *src)
{
  if (fp && src) { fp->uf_args = *src; }
}

/// Move (struct-assign) *src into fp->uf_lines.
void nvim_ufunc_move_lines_ga(ufunc_T *fp, const garray_T *src)
{
  if (fp && src) { fp->uf_lines = *src; }
}

/// Set fp->uf_script_ctx = current_sctx, then adjust sc_lnum by SOURCING_LNUM.
/// newlines_len is the number of lines in uf_lines (used for sc_lnum offset).
void nvim_ufunc_finalize_script_ctx(ufunc_T *fp, int newlines_len)
{
  if (fp) {
    fp->uf_script_ctx = current_sctx;
    fp->uf_script_ctx.sc_lnum += SOURCING_LNUM - newlines_len;
  }
}

/// Size of partial_T.
size_t nvim_sizeof_partial(void) { return sizeof(partial_T); }

/// skip_expr(arg, evalarg) wrapper for Rust (with evalarg support).
int nvim_skip_expr(char **arg, evalarg_T *evalarg) { return skip_expr(arg, evalarg); }

/// Get evalarg->eval_tofree.
char *nvim_evalarg_get_tofree(evalarg_T *ea) { return ea ? ea->eval_tofree : NULL; }

/// Set evalarg->eval_tofree.
void nvim_evalarg_set_tofree(evalarg_T *ea, char *v) { if (ea) { ea->eval_tofree = v; } }

// Phase 6 (plan db85cc6b): free_all_functions accessor

/// hash_clear the global function hashtab (called after free_all_functions).
void nvim_func_ht_hash_clear(void) { hash_clear(&func_hashtab); }

// Phase 7 (plan db85cc6b): get_function_body accessors

/// Call eap->ea_getline with a do_concat flag (unlike vars.c wrapper which hard-codes false).
char *nvim_eap_call_getline_concat(void *eap_void, int c, int indent, bool do_concat)
{
  exarg_T *eap = (exarg_T *)eap_void;
  if (eap->ea_getline == NULL) {
    return NULL;
  }
  return eap->ea_getline((char)c, eap->cookie, indent, do_concat);
}

/// Return get_sourced_lnum(eap->ea_getline, eap->cookie).
linenr_T nvim_eap_get_sourced_lnum(const exarg_T *eap)
{
  return eap ? get_sourced_lnum(eap->ea_getline, eap->cookie) : (linenr_T)0;
}

/// Return eap->cmdlinep (char **).
char **nvim_eap_get_cmdlinep(exarg_T *eap) { return eap ? eap->cmdlinep : NULL; }

/// Return eap->cmd.
char *nvim_eap_get_cmd(const exarg_T *eap) { return eap ? eap->cmd : NULL; }

/// Emit warning W22 via swmsg (for text after :endfunction).
void nvim_userfunc_swmsg_w22(const char *p) { swmsg(true, _("W22: Text found after :endfunction: %s"), p); }

/// Emit error E126 (missing :endfunction).
void nvim_userfunc_emsg_e126(void) { emsg(_("E126: Missing :endfunction")); }

/// Emit error E1058 (function nesting too deep).
void nvim_userfunc_emsg_e1058(void) { emsg(_(e_function_nesting_too_deep)); }

/// Emit error E1145 (missing heredoc end marker).
void nvim_userfunc_semsg_e1145(const char *marker) { semsg(_(e_missing_heredoc_end_marker_str), marker); }

// Wave 2 Phase 4: ex_function migration accessors
int nvim_keytyped(void) { return (int)KeyTyped; }
int nvim_get_msg_row(void) { return msg_row; }
void nvim_set_cmdline_row(int v) { cmdline_row = v; }
/// Increment the file-level func_nr counter and return the new value.
/// Used to generate sequential nameless-function names ("%d").
int nvim_next_func_nr(void) { return ++func_nr; }
/// Overwrite existing func_hashtab entry for fp: sets hi->hi_key = UF2HIKEY(fp).
void nvim_func_ht_overwrite_fp(const char *name, ufunc_T *fp)
{
  hashitem_T *hi = hash_find(&func_hashtab, name);
  hi->hi_key = UF2HIKEY(fp);
}
/// Copy newargs / default_args / newlines garrays into fp.
void nvim_ufunc_set_garray_fields(ufunc_T *fp, garray_T *args, garray_T *def_args,
                                  garray_T *lines)
{
  if (!fp) { return; }
  fp->uf_args = *args;
  fp->uf_def_args = *def_args;
  fp->uf_lines = *lines;
}
/// Set fp->uf_varargs, uf_flags (adding FC_SANDBOX if sandbox is on),
/// uf_calls=0, uf_script_ctx = current_sctx adjusted by sourcing_lnum_top,
/// then call nlua_set_sctx.
void nvim_ufunc_finalize_user_func(ufunc_T *fp, int varargs, int flags, linenr_T sourcing_lnum_top)
{
  if (!fp) { return; }
  fp->uf_varargs = varargs;
  if (sandbox) { flags |= FC_SANDBOX; }
  fp->uf_flags = flags;
  fp->uf_calls = 0;
  fp->uf_script_ctx = current_sctx;
  fp->uf_script_ctx.sc_lnum += sourcing_lnum_top;
  nlua_set_sctx(&fp->uf_script_ctx);
}
/// Emit error messages for ex_function
void nvim_emsg_e124_missing_paren(const char *arg) { semsg(_("E124: Missing '(': %s"), arg); }
void nvim_emsg_e707_func_name_conflict(const char *name)
{
  emsg_funcname(N_("E707: Function name conflicts with variable: %s"), name);
}
void nvim_emsg_e127_cannot_redefine(const char *name)
{
  emsg_funcname(N_("E127: Cannot redefine function %s: It is in use"), name);
}
void nvim_emsg_e122_func_exists(const char *name) { emsg_funcname(e_funcexts, name); }
void nvim_emsg_e746_autoload_mismatch(const char *name)
{
  semsg(_("E746: Function name does not match script file name: %s"), name);
}
void nvim_emsg_e932_closure_toplevel(const char *name)
{
  emsg_funcname(N_("E932: Closure function should not be at top level: %s"),
                name == NULL ? "" : name);
}
void nvim_emsg_e862_no_g_dict(void) { emsg(_("E862: Cannot use g: here")); }
void nvim_emsg_e717_funcdict(void) { emsg(_(e_funcdict)); }
// Additional Phase 4 accessors for ufunc fields and script context
int nvim_ufunc_get_script_ctx_seq(const ufunc_T *fp) { return fp ? fp->uf_script_ctx.sc_seq : 0; }
int nvim_current_sctx_get_seq(void) { return current_sctx.sc_seq; }
void nvim_ufunc_set_name_exp(ufunc_T *fp, char *v) { if (fp) { fp->uf_name_exp = v; } }
/// XFREE_CLEAR(fp->uf_name_exp): free and set to NULL.
void nvim_ufunc_free_name_exp(ufunc_T *fp) { if (fp) { XFREE_CLEAR(fp->uf_name_exp); } }
/// Return fp->uf_name (pointer to the name array inside ufunc_T).
const char *nvim_ufunc_get_uf_name_ptr(const ufunc_T *fp) { return fp ? fp->uf_name : NULL; }
/// Set di->di_tv to VAR_FUNC with xmemdupz(name, namelen).
void nvim_dictitem_set_tv_func(dictitem_T *di, const char *name, size_t namelen)
{
  if (!di) { return; }
  di->di_tv.v_type = VAR_FUNC;
  di->di_tv.vval.v_string = xmemdupz(name, namelen);
}
/// Decrement fp->uf_refcount by 1.
void nvim_ufunc_dec_refcount(ufunc_T *fp) { if (fp) { (fp->uf_refcount)--; } }
/// Get v_lock from a typval_T pointer.
int nvim_tv_get_lock(const typval_T *tv) { return tv ? (int)tv->v_lock : 0; }
