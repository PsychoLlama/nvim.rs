#pragma once

#include <stdbool.h>
#include <stddef.h>

#include "nvim/cmdexpand_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"
#include "nvim/eval_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/hashtab_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"
#include "nvim/types_defs.h"  // IWYU pragma: keep

// From user function to hashitem and back.
#define UF2HIKEY(fp) ((fp)->uf_name)
#define HIKEY2UF(p)  ((ufunc_T *)((p) - offsetof(ufunc_T, uf_name)))
#define HI2UF(hi)    HIKEY2UF((hi)->hi_key)

// flags used in uf_flags
#define FC_ABORT    0x01          // abort function on error
#define FC_RANGE    0x02          // function accepts range
#define FC_DICT     0x04          // Dict function, uses "self"
#define FC_CLOSURE  0x08          // closure, uses outer scope variables
#define FC_DELETED  0x10          // :delfunction used while uf_refcount > 0
#define FC_REMOVED  0x20          // function redefined while uf_refcount > 0
#define FC_SANDBOX  0x40          // function defined in the sandbox
// #define FC_DEAD     0x80          // function kept only for reference to dfunc
// #define FC_EXPORT   0x100         // "export def Func()"
#define FC_NOARGS   0x200         // no a: variables in lambda
// #define FC_VIM9     0x400         // defined in vim9 script file
#define FC_LUAREF  0x800          // luaref callback

/// Structure used by trans_function_name()
typedef struct {
  dict_T *fd_dict;    ///< Dict used.
  char *fd_newkey;    ///< New key in "dict" in allocated memory.
  dictitem_T *fd_di;  ///< Dict item used.
} funcdict_T;

typedef struct funccal_entry funccal_entry_T;
struct funccal_entry {
  void *top_funccal;
  funccal_entry_T *next;
};

/// errors for when calling a function
typedef enum {
  FCERR_UNKNOWN = 0,
  FCERR_TOOMANY = 1,
  FCERR_TOOFEW = 2,
  FCERR_SCRIPT = 3,
  FCERR_DICT = 4,
  FCERR_NONE = 5,
  FCERR_OTHER = 6,
  FCERR_DELETED = 7,
  FCERR_NOTMETHOD = 8,  ///< function cannot be used as a method
} FnameTransError;

/// Used in funcexe_T. Returns the new argcount.
typedef int (*ArgvFunc)(int current_argcount, typval_T *argv, int partial_argcount,
                        ufunc_T *called_func);

/// Structure passed between functions dealing with function call execution.
typedef struct {
  ArgvFunc fe_argv_func;  ///< when not NULL, can be used to fill in arguments only
                          ///< when the invoked function uses them
  linenr_T fe_firstline;  ///< first line of range
  linenr_T fe_lastline;   ///< last line of range
  bool *fe_doesrange;     ///< [out] if not NULL: function handled range
  bool fe_evaluate;       ///< actually evaluate expressions
  partial_T *fe_partial;  ///< for extra arguments
  dict_T *fe_selfdict;    ///< Dict for "self"
  typval_T *fe_basetv;    ///< base for base->method()
  bool fe_found_var;      ///< if the function is not found then give an
                          ///< error that a variable is not callable.
} funcexe_T;

#define FUNCEXE_INIT (funcexe_T) { \
  .fe_argv_func = NULL, \
  .fe_firstline = 0, \
  .fe_lastline = 0, \
  .fe_doesrange = NULL, \
  .fe_evaluate = false, \
  .fe_partial = NULL, \
  .fe_selfdict = NULL, \
  .fe_basetv = NULL, \
  .fe_found_var = false, \
}

#define FUNCARG(fp, j)  ((char **)(fp->uf_args.ga_data))[j]
#define FUNCLINE(fp, j) ((char **)(fp->uf_lines.ga_data))[j]

// Functions implemented in Rust (nvim-eval crate)
extern int current_func_returned(void);

// Functions exported directly from Rust (nvim-rs userfunc crate)
// These replaced C thin wrappers in Phase 5b.
#include <stdbool.h>
extern void emsg_funcname(const char *errmsg, const char *name);
extern void func_unref(char *name);
extern void func_ptr_unref(ufunc_T *fp);
extern void func_ref(char *name);
extern void func_ptr_ref(ufunc_T *fp);
extern funccall_T *create_funccal(ufunc_T *fp, typval_T *rettv);
extern void remove_funccal(void);
extern void save_funccal(funccal_entry_T *entry);
extern void restore_funccal(void);
extern void ex_delfunction(exarg_T *eap);
extern int eval_fname_script(const char *p);
extern bool translated_function_exists(const char *name);
extern bool function_exists(const char *name, bool no_deref);
extern char *get_scriptlocal_funcname(char *funcname);
extern char *save_function_name(char **name, bool skip, int flags, funcdict_T *fudi);
extern char *printable_func_name(ufunc_T *fp);
extern bool can_add_defer(void);
extern void add_defer(char *name, int argcount, typval_T *argvars);
extern void invoke_all_defer(void);
extern void ex_return(exarg_T *eap);
extern bool do_return(exarg_T *eap, bool reanimate, bool is_cmd, void *rettv);
extern char *get_return_cmd(void *rettv);
extern bool free_unref_funccal(int copyID, int testing);
extern bool set_ref_in_previous_funccal(int copyID);
extern bool set_ref_in_call_stack(int copyID);
extern bool set_ref_in_functions(int copyID);
extern bool set_ref_in_func_args(int copyID);
extern bool set_ref_in_func(char *name, ufunc_T *fp_in, int copyID);
extern funccall_T *get_funccal(void);
extern dict_T *get_funccal_local_dict(void);
extern hashtab_T *get_funccal_local_ht(void);
extern dictitem_T *get_funccal_local_var(void);
extern dict_T *get_funccal_args_dict(void);
extern hashtab_T *get_funccal_args_ht(void);
extern dictitem_T *get_funccal_args_var(void);
extern void list_func_vars(int *first);
extern dict_T *get_current_funccal_dict(hashtab_T *ht);
extern hashitem_T *find_hi_in_scoped_ht(const char *name, hashtab_T **pht);
extern dictitem_T *find_var_in_scoped_ht(const char *name, size_t namelen, int no_autoload);

// Phase 6/7: Functions migrated to Rust (lookup.rs)
extern int get_func_arity(const char *name, int *required, int *optional, bool *varargs);
// Phase 15: callback_call_retnr migrated to Rust (funccal.rs)
extern varnumber_T callback_call_retnr(Callback *callback, int argcount, typval_T *argvars);
extern char *deref_func_name(const char *name, int *lenp, partial_T **const partialp,
                             bool no_autoload, bool *found_var);
extern int func_has_ended(void *cookie);
extern int func_has_abort(void *cookie);
extern char *func_name(void *cookie);
extern linenr_T *func_breakpoint(void *cookie);
extern int *func_dbg_tick(void *cookie);
extern int func_level(void *cookie);

// Phase 22: call_func migrated to Rust (funccal.rs)
extern int call_func(const char *funcname, int len, typval_T *rettv, int argcount_in,
                     typval_T *argvars_in, funcexe_T *funcexe);

// Phase 35: func_call migrated to Rust (funccal.rs)
extern int func_call(char *name, typval_T *args, partial_T *partial, dict_T *selfdict,
                     typval_T *rettv);

// Phase 2 (plan db85cc6b): register_luafunc migrated to Rust (lambda.rs)
extern char *register_luafunc(LuaRef ref);

// Phase 3 (plan db85cc6b): get_user_func_name migrated to Rust (expand.rs)
extern char *get_user_func_name(expand_T *xp, int idx);

// Phase 23: get_func_tv migrated to Rust (funccal.rs)
extern int get_func_tv(const char *name, int len, typval_T *rettv, char **arg,
                       evalarg_T *const evalarg, funcexe_T *funcexe);

// Wave 2 Phase 1: find_func and apply_autocmds_for_funcundefined migrated to Rust (hashtab.rs).
extern ufunc_T *find_func(const char *name);
extern int apply_autocmds_for_funcundefined(const char *name);
// Wave 2 Phase 2: ex_call migrated to Rust (excmd.rs).
extern void ex_call(exarg_T *eap);

#include "eval/userfunc.h.generated.h"
