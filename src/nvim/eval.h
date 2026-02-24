#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "nvim/channel_defs.h"  // IWYU pragma: keep
#include "nvim/cmdexpand_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"
#include "nvim/eval_defs.h"  // IWYU pragma: keep
#include "nvim/event/defs.h"
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/grid_defs.h"  // IWYU pragma: keep
#include "nvim/hashtab_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte_defs.h"  // IWYU pragma: keep
#include "nvim/msgpack_rpc/channel_defs.h"  // IWYU pragma: keep
#include "nvim/option_defs.h"  // IWYU pragma: keep
#include "nvim/os/fileio_defs.h"  // IWYU pragma: keep
#include "nvim/os/stdpaths_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep
#include "nvim/vim_defs.h"  // IWYU pragma: keep

#define COPYID_INC 2
#define COPYID_MASK (~0x1)

// Structure returned by get_lval() and used by set_var_lval().
// For a plain name:
//      "name"      points to the variable name.
//      "exp_name"  is NULL.
//      "tv"        is NULL
// For a magic braces name:
//      "name"      points to the expanded variable name.
//      "exp_name"  is non-NULL, to be freed later.
//      "tv"        is NULL
// For an index in a list:
//      "name"      points to the (expanded) variable name.
//      "exp_name"  NULL or non-NULL, to be freed later.
//      "tv"        points to the (first) list item value
//      "li"        points to the (first) list item
//      "range", "n1", "n2" and "empty2" indicate what items are used.
// For an existing Dict item:
//      "name"      points to the (expanded) variable name.
//      "exp_name"  NULL or non-NULL, to be freed later.
//      "tv"        points to the dict item value
//      "newkey"    is NULL
// For a non-existing Dict item:
//      "name"      points to the (expanded) variable name.
//      "exp_name"  NULL or non-NULL, to be freed later.
//      "tv"        points to the Dictionary typval_T
//      "newkey"    is the key for the new item.
typedef struct {
  const char *ll_name;  ///< Start of variable name (can be NULL).
  size_t ll_name_len;   ///< Length of the .ll_name.
  char *ll_exp_name;    ///< NULL or expanded name in allocated memory.
  typval_T *ll_tv;      ///< Typeval of item being used.  If "newkey"
  ///< isn't NULL it's the Dict to which to add the item.
  listitem_T *ll_li;  ///< The list item or NULL.
  list_T *ll_list;    ///< The list or NULL.
  bool ll_range;      ///< true when a [i:j] range was used.
  bool ll_empty2;     ///< Second index is empty: [i:].
  int ll_n1;          ///< First index for list.
  int ll_n2;          ///< Second index for list range.
  dict_T *ll_dict;    ///< The Dict or NULL.
  dictitem_T *ll_di;  ///< The dictitem or NULL.
  char *ll_newkey;    ///< New key for Dict in allocated memory or NULL.
  blob_T *ll_blob;    ///< The Blob or NULL.
} lval_T;

/// enum used by var_flavour()
typedef enum {
  VAR_FLAVOUR_DEFAULT = 1,   // doesn't start with uppercase
  VAR_FLAVOUR_SESSION = 2,   // starts with uppercase, some lower
  VAR_FLAVOUR_SHADA   = 4,  // all uppercase
} var_flavour_T;

// Struct passed to get_v_event() and restore_v_event().
typedef struct {
  bool sve_did_save;
  hashtab_T sve_hashtab;
} save_v_event_T;

/// trans_function_name() flags
typedef enum {
  TFN_INT = 1,  ///< May use internal function name
  TFN_QUIET = 2,  ///< Do not emit error messages.
  TFN_NO_AUTOLOAD = 4,  ///< Do not use script autoloading.
  TFN_NO_DEREF = 8,  ///< Do not dereference a Funcref.
  TFN_READ_ONLY = 16,  ///< Will not change the variable.
} TransFunctionNameFlags;

/// get_lval() flags
typedef enum {
  GLV_QUIET = TFN_QUIET,  ///< Do not emit error messages.
  GLV_NO_AUTOLOAD = TFN_NO_AUTOLOAD,  ///< Do not use script autoloading.
  GLV_READ_ONLY = TFN_READ_ONLY,  ///< Indicates that caller will not change
                                  ///< the value (prevents error message).
} GetLvalFlags;

/// flags for find_name_end()
#define FNE_INCL_BR     1       // find_name_end(): include [] in name
#define FNE_CHECK_START 2       // find_name_end(): check name starts with
                                // valid character

typedef struct {
  TimeWatcher tw;
  int timer_id;
  int repeat_count;
  int refcount;
  int emsg_count;  ///< Errors in a repeating timer.
  int64_t timeout;
  bool stopped;
  bool paused;
  Callback callback;
} timer_T;

/// types for expressions.
typedef enum {
  EXPR_UNKNOWN = 0,
  EXPR_EQUAL,         ///< ==
  EXPR_NEQUAL,        ///< !=
  EXPR_GREATER,       ///< >
  EXPR_GEQUAL,        ///< >=
  EXPR_SMALLER,       ///< <
  EXPR_SEQUAL,        ///< <=
  EXPR_MATCH,         ///< =~
  EXPR_NOMATCH,       ///< !~
  EXPR_IS,            ///< is
  EXPR_ISNOT,         ///< isnot
} exprtype_T;

// Used for checking if local variables or arguments used in a lambda.
extern bool *eval_lavars_used;

// Character used as separated in autoload function/variable names.
#define AUTOLOAD_CHAR '#'

/// Flag for expression evaluation.
enum {
  EVAL_EVALUATE = 1,  ///< when missing don't actually evaluate
};

/// Passed to an eval() function to enable evaluation.
EXTERN evalarg_T EVALARG_EVALUATE INIT( = { EVAL_EVALUATE, NULL, NULL, NULL });

/// Rust-exported FFI symbols (renamed from rs_* in Phase 3, eval_shim pass 8).
/// These replace C thin wrappers that were deleted from eval_shim.c.
int pattern_match(const char *pat, const char *text, bool ic);
void fill_evalarg_from_eap(evalarg_T *evalarg, exarg_T *eap, bool skip);
void clear_evalarg(evalarg_T *evalarg, exarg_T *eap);
int eval0(char *arg, typval_T *rettv, exarg_T *eap, evalarg_T *const evalarg);
int may_call_simple_func(const char *arg, typval_T *rettv);
int eval1(char **arg, typval_T *rettv, evalarg_T *const evalarg);
int eval6(char **arg, typval_T *rettv, evalarg_T *const evalarg, bool want_string);
void f_slice(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_system(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_systemlist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
bool callback_call(Callback *const callback, const int argcount_in, typval_T *const argvars_in,
                   typval_T *const rettv);

/// Rust-exported FFI symbols (renamed from rs_* in Phase 2, eval_shim pass 9).
/// These replace C thin wrappers that were deleted from eval_shim.c.
bool eval_to_bool(char *arg, bool *error, exarg_T *eap, bool skip, bool use_simple_function);
int eval_expr_typval(const typval_T *expr, bool want_func, typval_T *argv, int argc,
                     typval_T *rettv);
bool eval_expr_to_bool(const typval_T *expr, bool *error);
char *eval_to_string_skip(char *arg, exarg_T *eap, bool skip);
int skip_expr(char **pp, evalarg_T *evalarg);
char *eval_to_string_eap(char *arg, bool join_list, exarg_T *eap, bool use_simple_function);
char *eval_to_string(char *arg, bool join_list, bool use_simple_function);
char *eval_to_string_safe(char *arg, bool use_sandbox, bool use_simple_function);
varnumber_T eval_to_number(char *expr, bool use_simple_function);
typval_T *eval_expr_ext(char *arg, exarg_T *eap, bool use_simple_function);
int call_vim_function(const char *func, int argc, typval_T *argv, typval_T *rettv);
void *call_func_retstr(const char *func, int argc, typval_T *argv);
void *call_func_retlist(const char *func, int argc, typval_T *argv);
int eval_option(const char **arg, typval_T *rettv, bool evaluate);
int eval_interp_string(char **arg, typval_T *rettv, bool evaluate);
void partial_unref(partial_T *pt);
int var_item_copy(const vimconv_T *conv, typval_T *from, typval_T *to, bool deep, int copyID);
int get_name_len(const char **arg, char **alias, bool evaluate, bool verbose);
char *typval_tostring(typval_T *arg, bool quotes);
char *save_tv_as_string(typval_T *tv, ptrdiff_t *len, bool endnl, bool crlf);

/// Rust-exported FFI symbols (renamed from rs_* in Phase 3, eval_shim pass 9).
/// These replace C thin wrappers that were deleted from eval_shim.c.
char *get_lval(char *name, typval_T *rettv, lval_T *lp, bool unlet, bool skip, int flags,
               int fne_flags);
void clear_lval(lval_T *lp);
void set_var_lval(lval_T *lp, char *endp, typval_T *rettv, bool copy, bool is_const,
                  const char *op);
void *eval_for_line(const char *arg, bool *errp, exarg_T *eap, evalarg_T *evalarg);
bool next_for_item(void *fi_void, char *arg);
void free_for_info(void *fi_void);
int handle_subscript(const char **arg, typval_T *rettv, evalarg_T *evalarg, bool verbose);
void set_selfdict(typval_T *rettv, dict_T *selfdict);
void ex_echo(exarg_T *eap);
void ex_execute(exarg_T *eap);
int eval_foldexpr(win_T *wp, int *cp);
char **tv_to_argv(typval_T *cmd_tv, const char **cmd, bool *executable);
int list2fpos(typval_T *arg, pos_T *posp, int *fnump, colnr_T *curswantp, bool charcol);
void set_argv_var(char **argv, int argc);
bool eval_has_provider(const char *feat, bool throw_if_fast);
void eval_fmt_source_name_line(char *buf, size_t bufsize);
char *do_string_sub(char *str, size_t len, char *pat, char *sub, typval_T *expr,
                    const char *flags, size_t *ret_len);
void var_set_global(const char *name, typval_T *vartv);

#include "eval_shim.h.generated.h"
