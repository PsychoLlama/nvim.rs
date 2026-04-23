// eval/vars.c: functions for dealing with variables

#include <assert.h>
#include <ctype.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>

#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/encode.h"
#include "nvim/eval/funcs.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/eval/window.h"
#include "nvim/eval_defs.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/garray.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/os/os.h"
#include "nvim/register.h"
#include "nvim/runtime.h"
#include "nvim/search.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/version.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

_Static_assert(VV_WARNINGMSG == 4, "VV_WARNINGMSG changed - update Rust constants");
_Static_assert(VV_EXCEPTION == 30, "VV_EXCEPTION changed - update Rust constants");
_Static_assert(VV_TESTING == 76, "VV_TESTING changed - update Rust constants");

typedef int (*ex_unletlock_callback)(lval_T *, char *, exarg_T *, int);

#include "eval/vars.c.generated.h"
extern int rs_valid_tabpage(tabpage_T *tpc);

// Rust FFI declarations (typval functions migrated to Rust)
extern const char *tv_list_find_str(list_T *l, int n);
extern varnumber_T tv_list_find_nr(list_T *l, int n, bool *ret_error);

// Rust FFI declarations (window wrappers removed)
extern tabpage_T *rs_find_tabpage(int n);

extern void rs_optval_free(OptVal o);
extern int rs_is_tty_option(const char *name);
extern int64_t rs_num_divide(int64_t n1, int64_t n2);
extern int64_t rs_num_modulus(int64_t n1, int64_t n2);
extern bool rs_eval_isnamec1(int c);
extern int rs_get_env_len(const char **arg);
extern const char *rs_find_name_end(const char *arg, const char **expr_start,
                                    const char **expr_end, int flags);
extern bool rs_set_ref_in_ht(hashtab_T *ht, int copyID, list_stack_T **list_stack);

// Phase 1: eval_helpers migrated to Rust
extern const char *rs_skip_var_list(const char *arg, int *var_count, int *semicolon,
                                    bool silent);
extern char *rs_eval_one_expr_in_str(char *p, garray_T *gap, bool evaluate);
extern char *rs_eval_all_expr_in_str(char *str);
extern int rs_get_spellword(list_T *list, const char **ret_word);

// Phase 3 partial: ex_let_env, ex_let_register migrated to Rust
extern char *rs_ex_let_env(char *arg, typval_T *tv, bool is_const,
                            const char *endchars, const char *op);
extern char *rs_ex_let_register(char *arg, typval_T *tv, bool is_const,
                                 const char *endchars, const char *op);

// Phase 1: variable validation/check functions migrated to Rust
extern bool rs_var_check_ro(int flags, const char *name, size_t name_len);
extern bool rs_var_check_lock(int flags, const char *name, size_t name_len);
extern bool rs_var_check_fixed(int flags, const char *name, size_t name_len);
extern bool rs_var_wrong_func_name(const char *name, bool new_var);
extern bool rs_valid_varname(const char *varname);

// Phase 4: v: variable accessor functions migrated to Rust
extern typval_T *rs_get_vim_var_tv(VimVarIndex idx);
extern char *rs_get_vim_var_name(VimVarIndex idx);
extern varnumber_T rs_get_vim_var_nr(VimVarIndex idx);
extern list_T *rs_get_vim_var_list(VimVarIndex idx);
extern dict_T *rs_get_vim_var_dict(VimVarIndex idx);
extern char *rs_get_vim_var_str(VimVarIndex idx);
extern partial_T *rs_get_vim_var_partial(VimVarIndex idx);
extern void rs_set_vim_var_tv(VimVarIndex idx, typval_T *tv);
extern void rs_set_vim_var_type(VimVarIndex idx, VarType type);
extern void rs_set_vim_var_nr(VimVarIndex idx, varnumber_T val);
extern void rs_set_vim_var_bool(VimVarIndex idx, BoolVarValue val);
extern void rs_set_vim_var_special(VimVarIndex idx, SpecialVarValue val);
extern void rs_set_vim_var_char(int c);
extern void rs_set_vim_var_string(VimVarIndex idx, const char *val, ptrdiff_t len);
extern void rs_set_vim_var_list(VimVarIndex idx, list_T *val);
extern void rs_set_vim_var_dict(VimVarIndex idx, dict_T *val);
extern void rs_set_vim_var_partial(VimVarIndex idx, partial_T *val);

// Phase 1 (new): option conversion and set_cmdarg migrated to Rust
extern OptVal rs_tv_to_optval(typval_T *tv, OptIndex opt_idx, const char *option, bool *error);
extern void rs_optval_as_tv(OptVal value, bool numbool, typval_T *rettv);
extern void rs_set_option_from_tv(const char *varname, typval_T *varp);
extern char *rs_set_cmdarg(exarg_T *eap, char *oldarg);

// Phase 2: VimL f_ functions migrated to Rust
extern void rs_get_var_from(const char *varname, typval_T *rettv, typval_T *deftv, int htname,
                             tabpage_T *tp, win_T *win, buf_T *buf);
// (rs_f_* functions are declared below)

// Phase 4: heredoc and unlet functions migrated to Rust
extern list_T *rs_heredoc_get(exarg_T *eap, char *cmd, int script_get);
extern void rs_ex_unlet(exarg_T *eap);
extern void rs_ex_lockvar(exarg_T *eap);
extern int rs_do_unlet(const char *name, size_t name_len, int forceit);

// Phase 3: listing and redirection functions migrated to Rust
extern char *rs_cat_prefix_varname(int prefix, const char *name);
extern char *rs_get_user_var_name(expand_T *xp, int idx);
extern void rs_var_redir_str(const char *value, int value_len);
extern void rs_list_hashtable_vars(hashtab_T *ht, const char *prefix, int empty, int *first);
extern void rs_list_one_var(dictitem_T *v, const char *prefix, int *first);
extern void rs_list_one_var_a(const char *prefix, const char *name, ptrdiff_t name_len,
                               VarType type, const char *string, int *first);
extern void rs_f_gettabvar(typval_T *argvars, typval_T *rettv);
extern void rs_f_gettabwinvar(typval_T *argvars, typval_T *rettv);
extern void rs_f_getwinvar(typval_T *argvars, typval_T *rettv);
extern void rs_f_getbufvar(typval_T *argvars, typval_T *rettv);
extern void rs_f_settabvar(typval_T *argvars);
extern void rs_f_settabwinvar(typval_T *argvars);
extern void rs_f_setwinvar(typval_T *argvars);
extern void rs_f_setbufvar(typval_T *argvars);

// Phase 5: helper and utility functions migrated to Rust
extern void rs_set_reg_var(int c);
extern char *rs_v_exception(char *oldval);
extern char *rs_v_throwpoint(char *oldval);
extern void rs_set_vcount(int64_t count, int64_t count1, bool set_prevcount);
extern void rs_reset_v_option_vars(void);
extern void rs_assert_error(const char *ga_data, int ga_len);

// TODO(ZyX-I): Remove DICT_MAXNEST, make users be non-recursive instead

#define DICT_MAXNEST 100        // maximum nesting of lists and dicts

static const char *e_letunexp = N_("E18: Unexpected characters in :let");
static const char e_double_semicolon_in_list_of_variables[]
  = N_("E452: Double ; in list of variables");
static const char *e_lock_unlock = N_("E940: Cannot lock or unlock variable %s");
static const char e_setting_v_str_to_value_with_wrong_type[]
  = N_("E963: Setting v:%s to value with wrong type");
static const char e_missing_end_marker_str[] = N_("E990: Missing end marker '%s'");
static const char e_cannot_use_heredoc_here[] = N_("E991: Cannot use =<< here");

/// Variable used for g:
static ScopeDictDictItem globvars_var;
static dict_T globvardict;                  // Dict with g: variables
/// g: value
#define globvarht globvardict.dv_hashtab

/// Old Vim variables such as "v:version" are also available without the "v:".
/// Also in functions.  We need a special hashtable for them.
static hashtab_T compat_hashtab;

// values for vv_flags:
#define VV_COMPAT       1       // compatible, also used without "v:"
#define VV_RO           2       // read-only
#define VV_RO_SBX       4       // read-only in the sandbox

#define VV(idx, name, type, flags) \
  [idx] = { \
    .vv_name = (name), \
    .vv_di = { \
      .di_tv = { .v_type = (type) }, \
      .di_flags = 0, \
      .di_key = { 0 }, \
    }, \
    .vv_flags = (flags), \
  }

#define VIMVAR_KEY_LEN 16  // Maximum length of the key of v:variables

// Array to hold the value of v: variables.
// The value is in a dictitem, so that it can also be used in the v: scope.
// The reason to use this table anyway is for very quick access to the
// variables with the VV_ defines.
static struct vimvar {
  char *vv_name;  ///< Name of the variable, without v:.
  TV_DICTITEM_STRUCT(VIMVAR_KEY_LEN + 1) vv_di;  ///< Value and name for key (max 16 chars).
  char vv_flags;  ///< Flags: #VV_COMPAT, #VV_RO, #VV_RO_SBX.
} vimvars[] = {
  // VV_ tails differing from upcased string literals:
  // VV_CC_FROM "charconvert_from"
  // VV_CC_TO "charconvert_to"
  // VV_SEND_SERVER "servername"
  // VV_REG "register"
  // VV_OP "operator"
  VV(VV_COUNT,            "count",            VAR_NUMBER, VV_RO),
  VV(VV_COUNT1,           "count1",           VAR_NUMBER, VV_RO),
  VV(VV_PREVCOUNT,        "prevcount",        VAR_NUMBER, VV_RO),
  VV(VV_ERRMSG,           "errmsg",           VAR_STRING, 0),
  VV(VV_WARNINGMSG,       "warningmsg",       VAR_STRING, 0),
  VV(VV_STATUSMSG,        "statusmsg",        VAR_STRING, 0),
  VV(VV_SHELL_ERROR,      "shell_error",      VAR_NUMBER, VV_RO),
  VV(VV_THIS_SESSION,     "this_session",     VAR_STRING, 0),
  VV(VV_VERSION,          "version",          VAR_NUMBER, VV_COMPAT + VV_RO),
  VV(VV_LNUM,             "lnum",             VAR_NUMBER, VV_RO_SBX),
  VV(VV_TERMRESPONSE,     "termresponse",     VAR_STRING, VV_RO),
  VV(VV_TERMREQUEST,      "termrequest",      VAR_STRING, VV_RO),
  VV(VV_FNAME,            "fname",            VAR_STRING, VV_RO),
  VV(VV_LANG,             "lang",             VAR_STRING, VV_RO),
  VV(VV_LC_TIME,          "lc_time",          VAR_STRING, VV_RO),
  VV(VV_CTYPE,            "ctype",            VAR_STRING, VV_RO),
  VV(VV_CC_FROM,          "charconvert_from", VAR_STRING, VV_RO),
  VV(VV_CC_TO,            "charconvert_to",   VAR_STRING, VV_RO),
  VV(VV_FNAME_IN,         "fname_in",         VAR_STRING, VV_RO),
  VV(VV_FNAME_OUT,        "fname_out",        VAR_STRING, VV_RO),
  VV(VV_FNAME_NEW,        "fname_new",        VAR_STRING, VV_RO),
  VV(VV_FNAME_DIFF,       "fname_diff",       VAR_STRING, VV_RO),
  VV(VV_CMDARG,           "cmdarg",           VAR_STRING, VV_RO),
  VV(VV_FOLDSTART,        "foldstart",        VAR_NUMBER, VV_RO_SBX),
  VV(VV_FOLDEND,          "foldend",          VAR_NUMBER, VV_RO_SBX),
  VV(VV_FOLDDASHES,       "folddashes",       VAR_STRING, VV_RO_SBX),
  VV(VV_FOLDLEVEL,        "foldlevel",        VAR_NUMBER, VV_RO_SBX),
  VV(VV_PROGNAME,         "progname",         VAR_STRING, VV_RO),
  VV(VV_SEND_SERVER,      "servername",       VAR_STRING, VV_RO),
  VV(VV_DYING,            "dying",            VAR_NUMBER, VV_RO),
  VV(VV_EXCEPTION,        "exception",        VAR_STRING, VV_RO),
  VV(VV_THROWPOINT,       "throwpoint",       VAR_STRING, VV_RO),
  VV(VV_REG,              "register",         VAR_STRING, VV_RO),
  VV(VV_CMDBANG,          "cmdbang",          VAR_NUMBER, VV_RO),
  VV(VV_INSERTMODE,       "insertmode",       VAR_STRING, VV_RO),
  VV(VV_VAL,              "val",              VAR_UNKNOWN, VV_RO),
  VV(VV_KEY,              "key",              VAR_UNKNOWN, VV_RO),
  VV(VV_PROFILING,        "profiling",        VAR_NUMBER, VV_RO),
  VV(VV_FCS_REASON,       "fcs_reason",       VAR_STRING, VV_RO),
  VV(VV_FCS_CHOICE,       "fcs_choice",       VAR_STRING, 0),
  VV(VV_BEVAL_BUFNR,      "beval_bufnr",      VAR_NUMBER, VV_RO),
  VV(VV_BEVAL_WINNR,      "beval_winnr",      VAR_NUMBER, VV_RO),
  VV(VV_BEVAL_WINID,      "beval_winid",      VAR_NUMBER, VV_RO),
  VV(VV_BEVAL_LNUM,       "beval_lnum",       VAR_NUMBER, VV_RO),
  VV(VV_BEVAL_COL,        "beval_col",        VAR_NUMBER, VV_RO),
  VV(VV_BEVAL_TEXT,       "beval_text",       VAR_STRING, VV_RO),
  VV(VV_SCROLLSTART,      "scrollstart",      VAR_STRING, 0),
  VV(VV_SWAPNAME,         "swapname",         VAR_STRING, VV_RO),
  VV(VV_SWAPCHOICE,       "swapchoice",       VAR_STRING, 0),
  VV(VV_SWAPCOMMAND,      "swapcommand",      VAR_STRING, VV_RO),
  VV(VV_CHAR,             "char",             VAR_STRING, 0),
  VV(VV_MOUSE_WIN,        "mouse_win",        VAR_NUMBER, 0),
  VV(VV_MOUSE_WINID,      "mouse_winid",      VAR_NUMBER, 0),
  VV(VV_MOUSE_LNUM,       "mouse_lnum",       VAR_NUMBER, 0),
  VV(VV_MOUSE_COL,        "mouse_col",        VAR_NUMBER, 0),
  VV(VV_OP,               "operator",         VAR_STRING, VV_RO),
  VV(VV_SEARCHFORWARD,    "searchforward",    VAR_NUMBER, 0),
  VV(VV_HLSEARCH,         "hlsearch",         VAR_NUMBER, 0),
  VV(VV_OLDFILES,         "oldfiles",         VAR_LIST, 0),
  VV(VV_WINDOWID,         "windowid",         VAR_NUMBER, VV_RO_SBX),
  VV(VV_PROGPATH,         "progpath",         VAR_STRING, VV_RO),
  VV(VV_COMPLETED_ITEM,   "completed_item",   VAR_DICT, 0),
  VV(VV_OPTION_NEW,       "option_new",       VAR_STRING, VV_RO),
  VV(VV_OPTION_OLD,       "option_old",       VAR_STRING, VV_RO),
  VV(VV_OPTION_OLDLOCAL,  "option_oldlocal",  VAR_STRING, VV_RO),
  VV(VV_OPTION_OLDGLOBAL, "option_oldglobal", VAR_STRING, VV_RO),
  VV(VV_OPTION_COMMAND,   "option_command",   VAR_STRING, VV_RO),
  VV(VV_OPTION_TYPE,      "option_type",      VAR_STRING, VV_RO),
  VV(VV_ERRORS,           "errors",           VAR_LIST, 0),
  VV(VV_FALSE,            "false",            VAR_BOOL, VV_RO),
  VV(VV_TRUE,             "true",             VAR_BOOL, VV_RO),
  VV(VV_NULL,             "null",             VAR_SPECIAL, VV_RO),
  VV(VV_NUMBERMAX,        "numbermax",        VAR_NUMBER, VV_RO),
  VV(VV_NUMBERMIN,        "numbermin",        VAR_NUMBER, VV_RO),
  VV(VV_NUMBERSIZE,       "numbersize",       VAR_NUMBER, VV_RO),
  VV(VV_VIM_DID_ENTER,    "vim_did_enter",    VAR_NUMBER, VV_RO),
  VV(VV_TESTING,          "testing",          VAR_NUMBER, 0),
  VV(VV_TYPE_NUMBER,      "t_number",         VAR_NUMBER, VV_RO),
  VV(VV_TYPE_STRING,      "t_string",         VAR_NUMBER, VV_RO),
  VV(VV_TYPE_FUNC,        "t_func",           VAR_NUMBER, VV_RO),
  VV(VV_TYPE_LIST,        "t_list",           VAR_NUMBER, VV_RO),
  VV(VV_TYPE_DICT,        "t_dict",           VAR_NUMBER, VV_RO),
  VV(VV_TYPE_FLOAT,       "t_float",          VAR_NUMBER, VV_RO),
  VV(VV_TYPE_BOOL,        "t_bool",           VAR_NUMBER, VV_RO),
  VV(VV_TYPE_BLOB,        "t_blob",           VAR_NUMBER, VV_RO),
  VV(VV_EVENT,            "event",            VAR_DICT, VV_RO),
  VV(VV_VERSIONLONG,      "versionlong",      VAR_NUMBER, VV_RO),
  VV(VV_ECHOSPACE,        "echospace",        VAR_NUMBER, VV_RO),
  VV(VV_ARGV,             "argv",             VAR_LIST, VV_RO),
  VV(VV_COLLATE,          "collate",          VAR_STRING, VV_RO),
  VV(VV_EXITING,          "exiting",          VAR_NUMBER, VV_RO),
  VV(VV_MAXCOL,           "maxcol",           VAR_NUMBER, VV_RO),
  VV(VV_STACKTRACE,       "stacktrace",       VAR_LIST, VV_RO),
  VV(VV_VIM_DID_INIT,     "vim_did_init",     VAR_NUMBER, VV_RO),
  // Neovim
  VV(VV_STDERR,           "stderr",           VAR_NUMBER, VV_RO),
  VV(VV_MSGPACK_TYPES,    "msgpack_types",    VAR_DICT, VV_RO),
  VV(VV__NULL_STRING,     "_null_string",     VAR_STRING, VV_RO),
  VV(VV__NULL_LIST,       "_null_list",       VAR_LIST, VV_RO),
  VV(VV__NULL_DICT,       "_null_dict",       VAR_DICT, VV_RO),
  VV(VV__NULL_BLOB,       "_null_blob",       VAR_BLOB, VV_RO),
  VV(VV_LUA,              "lua",              VAR_PARTIAL, VV_RO),
  VV(VV_RELNUM,           "relnum",           VAR_NUMBER, VV_RO),
  VV(VV_VIRTNUM,          "virtnum",          VAR_NUMBER, VV_RO),
};
#undef VV

// shorthand
#define vv_type         vv_di.di_tv.v_type
#define vv_str          vv_di.di_tv.vval.v_string
#define vv_list         vv_di.di_tv.vval.v_list
#define vv_tv           vv_di.di_tv

/// Variable used for v:
static ScopeDictDictItem vimvars_var;
static dict_T vimvardict;                   // Dict with v: variables
/// v: hashtab
#define vimvarht  vimvardict.dv_hashtab

static const char *const msgpack_type_names[] = {
  [kMPNil] = "nil",
  [kMPBoolean] = "boolean",
  [kMPInteger] = "integer",
  [kMPFloat] = "float",
  [kMPString] = "string",
  [kMPArray] = "array",
  [kMPMap] = "map",
  [kMPExt] = "ext",
};
const list_T *eval_msgpack_type_lists[] = {
  [kMPNil] = NULL,
  [kMPBoolean] = NULL,
  [kMPInteger] = NULL,
  [kMPFloat] = NULL,
  [kMPString] = NULL,
  [kMPArray] = NULL,
  [kMPMap] = NULL,
  [kMPExt] = NULL,
};

#define SCRIPT_SV(id) (SCRIPT_ITEM(id)->sn_vars)
#define SCRIPT_VARS(id) (SCRIPT_SV(id)->sv_dict.dv_hashtab)

void evalvars_init(void)
{
  init_var_dict(get_globvar_dict(), &globvars_var, VAR_DEF_SCOPE);
  init_var_dict(&vimvardict, &vimvars_var, VAR_SCOPE);
  vimvardict.dv_lock = VAR_FIXED;
  hash_init(&compat_hashtab);

  for (size_t i = 0; i < ARRAY_SIZE(vimvars); i++) {
    struct vimvar *p = &vimvars[i];
    assert(strlen(p->vv_name) <= VIMVAR_KEY_LEN);
    STRCPY(p->vv_di.di_key, p->vv_name);
    if (p->vv_flags & VV_RO) {
      p->vv_di.di_flags = DI_FLAGS_RO | DI_FLAGS_FIX;
    } else if (p->vv_flags & VV_RO_SBX) {
      p->vv_di.di_flags = DI_FLAGS_RO_SBX | DI_FLAGS_FIX;
    } else {
      p->vv_di.di_flags = DI_FLAGS_FIX;
    }

    // add to v: scope dict, unless the value is not always available
    if (p->vv_type != VAR_UNKNOWN) {
      hash_add(&vimvarht, p->vv_di.di_key);
    }
    if (p->vv_flags & VV_COMPAT) {
      // add to compat scope dict
      hash_add(&compat_hashtab, p->vv_di.di_key);
    }
  }
  const int vim_version = min_vim_version();
  set_vim_var_nr(VV_VERSION, vim_version);
  set_vim_var_nr(VV_VERSIONLONG, vim_version * 10000 + highest_patch());

  dict_T *const msgpack_types_dict = tv_dict_alloc();
  for (size_t i = 0; i < ARRAY_SIZE(msgpack_type_names); i++) {
    list_T *const type_list = tv_list_alloc(0);
    tv_list_set_lock(type_list, VAR_FIXED);
    tv_list_ref(type_list);
    dictitem_T *const di = tv_dict_item_alloc(msgpack_type_names[i]);
    di->di_flags |= DI_FLAGS_RO|DI_FLAGS_FIX;
    di->di_tv = (typval_T) {
      .v_type = VAR_LIST,
      .vval = { .v_list = type_list, },
    };
    eval_msgpack_type_lists[i] = type_list;
    if (tv_dict_add(msgpack_types_dict, di) == FAIL) {
      // There must not be duplicate items in this dictionary by definition.
      abort();
    }
  }
  msgpack_types_dict->dv_lock = VAR_FIXED;

  set_vim_var_dict(VV_MSGPACK_TYPES, msgpack_types_dict);
  set_vim_var_dict(VV_COMPLETED_ITEM, tv_dict_alloc_lock(VAR_FIXED));

  set_vim_var_dict(VV_EVENT, tv_dict_alloc_lock(VAR_FIXED));
  set_vim_var_list(VV_ERRORS, tv_list_alloc(kListLenUnknown));
  set_vim_var_nr(VV_STDERR,   CHAN_STDERR);
  set_vim_var_nr(VV_SEARCHFORWARD, 1);
  set_vim_var_nr(VV_HLSEARCH, 1);
  set_vim_var_nr(VV_COUNT1, 1);
  set_vim_var_special(VV_EXITING, kSpecialVarNull);

  set_vim_var_nr(VV_TYPE_NUMBER, VAR_TYPE_NUMBER);
  set_vim_var_nr(VV_TYPE_STRING, VAR_TYPE_STRING);
  set_vim_var_nr(VV_TYPE_FUNC,   VAR_TYPE_FUNC);
  set_vim_var_nr(VV_TYPE_LIST,   VAR_TYPE_LIST);
  set_vim_var_nr(VV_TYPE_DICT,   VAR_TYPE_DICT);
  set_vim_var_nr(VV_TYPE_FLOAT,  VAR_TYPE_FLOAT);
  set_vim_var_nr(VV_TYPE_BOOL,   VAR_TYPE_BOOL);
  set_vim_var_nr(VV_TYPE_BLOB,   VAR_TYPE_BLOB);

  set_vim_var_bool(VV_FALSE, kBoolVarFalse);
  set_vim_var_bool(VV_TRUE, kBoolVarTrue);
  set_vim_var_special(VV_NULL, kSpecialVarNull);
  set_vim_var_nr(VV_NUMBERMAX, VARNUMBER_MAX);
  set_vim_var_nr(VV_NUMBERMIN, VARNUMBER_MIN);
  set_vim_var_nr(VV_NUMBERSIZE, sizeof(varnumber_T) * 8);
  set_vim_var_nr(VV_MAXCOL, MAXCOL);

  set_vim_var_nr(VV_ECHOSPACE,    sc_col - 1);

  // vimvars[VV_LUA].vv_type = VAR_PARTIAL;
  partial_T *vvlua_partial = xcalloc(1, sizeof(partial_T));
  // this value shouldn't be printed, but if it is, do not crash
  vvlua_partial->pt_name = xmallocz(0);
  vvlua_partial->pt_refcount++;
  set_vim_var_partial(VV_LUA, vvlua_partial);

  set_reg_var(0);  // default for v:register is not 0 but '"'
}

#if defined(EXITFREE)
void evalvars_clear(void)
{
  for (size_t i = 0; i < ARRAY_SIZE(vimvars); i++) {
    struct vimvar *p = &vimvars[i];
    if (p->vv_di.di_tv.v_type == VAR_STRING) {
      XFREE_CLEAR(p->vv_str);
    } else if (p->vv_di.di_tv.v_type == VAR_LIST) {
      tv_list_unref(p->vv_list);
      p->vv_list = NULL;
    }
  }

  partial_unref(get_vim_var_partial(VV_LUA));
  set_vim_var_partial(VV_LUA, NULL);
  hash_clear(&vimvarht);
  hash_init(&vimvarht);    // garbage_collect() will access it
  hash_clear(&compat_hashtab);

  // global variables
  vars_clear(get_globvar_ht());

  // Script-local variables. Clear all the variables here.
  // The scriptvar_T is cleared later in free_scriptnames(), because a
  // variable in one script might hold a reference to the whole scope of
  // another script.
  for (int i = 1; i <= script_items.ga_len; i++) {
    vars_clear(&SCRIPT_VARS(i));
  }
}
#endif

int garbage_collect_globvars(int copyID) { return rs_set_ref_in_ht(&globvarht, copyID, NULL); }

bool garbage_collect_vimvars(int copyID) { return rs_set_ref_in_ht(&vimvarht, copyID, NULL); }

bool garbage_collect_scriptvars(int copyID)
{
  bool abort = false;

  for (int i = 1; i <= script_items.ga_len; i++) {
    abort = abort || rs_set_ref_in_ht(&SCRIPT_VARS(i), copyID, NULL);
  }

  return abort;
}

/// Set an internal variable to a string value. Creates the variable if it does
/// not already exist.
void set_internal_string_var(const char *name, char *value)  // NOLINT(readability-non-const-parameter)
  FUNC_ATTR_NONNULL_ARG(1)
{
  typval_T tv = {
    .v_type = VAR_STRING,
    .vval.v_string = value,
  };

  set_var(name, strlen(name), &tv, true);
}

int eval_charconvert(const char *const enc_from, const char *const enc_to,
                     const char *const fname_from, const char *const fname_to)
{
  const sctx_T saved_sctx = current_sctx;

  set_vim_var_string(VV_CC_FROM, enc_from, -1);
  set_vim_var_string(VV_CC_TO, enc_to, -1);
  set_vim_var_string(VV_FNAME_IN, fname_from, -1);
  set_vim_var_string(VV_FNAME_OUT, fname_to, -1);
  sctx_T *ctx = get_option_sctx(kOptCharconvert);
  if (ctx != NULL) {
    current_sctx = *ctx;
  }

  bool err = false;
  if (eval_to_bool(p_ccv, &err, NULL, false, true)) {
    err = true;
  }

  set_vim_var_string(VV_CC_FROM, NULL, -1);
  set_vim_var_string(VV_CC_TO, NULL, -1);
  set_vim_var_string(VV_FNAME_IN, NULL, -1);
  set_vim_var_string(VV_FNAME_OUT, NULL, -1);
  current_sctx = saved_sctx;

  if (err) {
    return FAIL;
  }
  return OK;
}

void eval_diff(const char *const origfile, const char *const newfile, const char *const outfile)
{
  const sctx_T saved_sctx = current_sctx;
  set_vim_var_string(VV_FNAME_IN, origfile, -1);
  set_vim_var_string(VV_FNAME_NEW, newfile, -1);
  set_vim_var_string(VV_FNAME_OUT, outfile, -1);

  sctx_T *ctx = get_option_sctx(kOptDiffexpr);
  if (ctx != NULL) {
    current_sctx = *ctx;
  }

  // errors are ignored
  typval_T *tv = eval_expr_ext(p_dex, NULL, true);
  tv_free(tv);

  set_vim_var_string(VV_FNAME_IN, NULL, -1);
  set_vim_var_string(VV_FNAME_NEW, NULL, -1);
  set_vim_var_string(VV_FNAME_OUT, NULL, -1);
  current_sctx = saved_sctx;
}

void eval_patch(const char *const origfile, const char *const difffile, const char *const outfile)
{
  const sctx_T saved_sctx = current_sctx;
  set_vim_var_string(VV_FNAME_IN, origfile, -1);
  set_vim_var_string(VV_FNAME_DIFF, difffile, -1);
  set_vim_var_string(VV_FNAME_OUT, outfile, -1);

  sctx_T *ctx = get_option_sctx(kOptPatchexpr);
  if (ctx != NULL) {
    current_sctx = *ctx;
  }

  // errors are ignored
  typval_T *tv = eval_expr_ext(p_pex, NULL, true);
  tv_free(tv);

  set_vim_var_string(VV_FNAME_IN, NULL, -1);
  set_vim_var_string(VV_FNAME_DIFF, NULL, -1);
  set_vim_var_string(VV_FNAME_OUT, NULL, -1);
  current_sctx = saved_sctx;
}

/// Evaluate an expression to a list with suggestions.
/// For the "expr:" part of 'spellsuggest'.
///
/// @return  NULL when there is an error.
list_T *eval_spell_expr(char *badword, char *expr)
{
  typval_T save_val;
  typval_T rettv;
  list_T *list = NULL;
  char *p = skipwhite(expr);
  const sctx_T saved_sctx = current_sctx;

  // Set "v:val" to the bad word.
  prepare_vimvar(VV_VAL, &save_val);
  set_vim_var_string(VV_VAL, badword, -1);
  if (p_verbose == 0) {
    emsg_off++;
  }
  sctx_T *ctx = get_option_sctx(kOptSpellsuggest);
  if (ctx != NULL) {
    current_sctx = *ctx;
  }

  int r = may_call_simple_func(p, &rettv);
  if (r == NOTDONE) {
    r = eval1(&p, &rettv, &EVALARG_EVALUATE);
  }
  if (r == OK) {
    if (rettv.v_type != VAR_LIST) {
      tv_clear(&rettv);
    } else {
      list = rettv.vval.v_list;
    }
  }

  if (p_verbose == 0) {
    emsg_off--;
  }
  tv_clear(get_vim_var_tv(VV_VAL));
  restore_vimvar(VV_VAL, &save_val);
  current_sctx = saved_sctx;

  return list;
}

int get_spellword(list_T *const list, const char **ret_word) { return rs_get_spellword(list, ret_word); }

void prepare_vimvar(int idx, typval_T *save_tv)
{
  *save_tv = vimvars[idx].vv_tv;
  vimvars[idx].vv_str = NULL;  // don't free it now
  if (vimvars[idx].vv_type == VAR_UNKNOWN) {
    hash_add(&vimvarht, vimvars[idx].vv_di.di_key);
  }
}

void restore_vimvar(int idx, typval_T *save_tv)
{
  vimvars[idx].vv_tv = *save_tv;
  if (vimvars[idx].vv_type != VAR_UNKNOWN) {
    return;
  }

  hashitem_T *hi = hash_find(&vimvarht, vimvars[idx].vv_di.di_key);
  if (HASHITEM_EMPTY(hi)) {
    internal_error("restore_vimvar()");
  } else {
    hash_remove(&vimvarht, hi);
  }
}

static void list_vim_vars(int *first) { list_hashtable_vars(&vimvarht, "v:", false, first); }
static void list_script_vars(int *first)
{
  if (current_sctx.sc_sid > 0 && current_sctx.sc_sid <= script_items.ga_len) {
    list_hashtable_vars(&SCRIPT_VARS(current_sctx.sc_sid), "s:", false, first);
  }
}

char *eval_one_expr_in_str(char *p, garray_T *gap, bool evaluate) { return rs_eval_one_expr_in_str(p, gap, evaluate); }
static char *eval_all_expr_in_str(char *str) { return rs_eval_all_expr_in_str(str); }

/// Get a list of lines from a HERE document. The here document is a list of
/// lines surrounded by a marker.
///     cmd << {marker}
///       {line1}
///       {line2}
///       ....
///     {marker}
///
/// The {marker} is a string. If the optional 'trim' word is supplied before the
/// marker, then the leading indentation before the lines (matching the
/// indentation in the 'cmd' line) is stripped.
///
/// When getting lines for an embedded script (e.g. python, lua, perl, ruby,
/// tcl, mzscheme), "script_get" is set to true. In this case, if the marker is
/// missing, then '.' is accepted as a marker.
///
/// @return  a List with {lines} or NULL on failure.
list_T *heredoc_get(exarg_T *eap, char *cmd, bool script_get)
{ return rs_heredoc_get(eap, cmd, script_get); }

/// ":let" list all variable values
/// ":let var1 var2" list variable values
/// ":let var = expr" assignment command.
/// ":let var += expr" assignment command.
/// ":let var -= expr" assignment command.
/// ":let var *= expr" assignment command.
/// ":let var /= expr" assignment command.
/// ":let var %= expr" assignment command.
/// ":let var .= expr" assignment command.
/// ":let var ..= expr" assignment command.
/// ":let [var1, var2] = expr" unpack list.
/// ":let [name, ..., ; lastname] = expr" unpack list.
///
/// ":cons[t] var = expr1" define constant
/// ":cons[t] [name1, name2, ...] = expr1" define constants unpacking list
/// ":cons[t] [name, ..., ; lastname] = expr" define constants unpacking list
void ex_let(exarg_T *eap)
{
  const bool is_const = eap->cmdidx == CMD_const;
  char *arg = eap->arg;
  char *expr = NULL;
  typval_T rettv;
  int var_count = 0;
  int semicolon = 0;
  char op[2];
  const char *argend;
  int first = true;

  argend = skip_var_list(arg, &var_count, &semicolon, false);
  if (argend == NULL) {
    return;
  }
  expr = skipwhite(argend);
  bool concat = strncmp(expr, "..=", 3) == 0;
  bool has_assign = *expr == '=' || (vim_strchr("+-*/%.", (uint8_t)(*expr)) != NULL
                                     && expr[1] == '=');
  if (!has_assign && !concat) {
    // ":let" without "=": list variables
    if (*arg == '[') {
      emsg(_(e_invarg));
    } else if (!ends_excmd(*arg)) {
      // ":let var1 var2"
      arg = (char *)list_arg_vars(eap, arg, &first);
    } else if (!eap->skip) {
      // ":let"
      list_glob_vars(&first);
      list_buf_vars(&first);
      list_win_vars(&first);
      list_tab_vars(&first);
      list_script_vars(&first);
      list_func_vars(&first);
      list_vim_vars(&first);
    }
    eap->nextcmd = check_nextcmd(arg);
    return;
  }

  if (expr[0] == '=' && expr[1] == '<' && expr[2] == '<') {
    // HERE document
    list_T *l = heredoc_get(eap, expr + 3, false);
    if (l != NULL) {
      tv_list_set_ret(&rettv, l);
      if (!eap->skip) {
        op[0] = '=';
        op[1] = NUL;
        ex_let_vars(eap->arg, &rettv, false, semicolon, var_count, is_const, op);
      }
      tv_clear(&rettv);
    }
    return;
  }

  rettv.v_type = VAR_UNKNOWN;

  op[0] = '=';
  op[1] = NUL;
  if (*expr != '=') {
    if (vim_strchr("+-*/%.", (uint8_t)(*expr)) != NULL) {
      op[0] = *expr;  // +=, -=, *=, /=, %= or .=
      if (expr[0] == '.' && expr[1] == '.') {  // ..=
        expr++;
      }
    }
    expr += 2;
  } else {
    expr += 1;
  }

  expr = skipwhite(expr);

  if (eap->skip) {
    emsg_skip++;
  }
  evalarg_T evalarg;
  fill_evalarg_from_eap(&evalarg, eap, eap->skip);
  int eval_res = eval0(expr, &rettv, eap, &evalarg);
  if (eap->skip) {
    emsg_skip--;
  }
  clear_evalarg(&evalarg, eap);

  if (!eap->skip && eval_res != FAIL) {
    ex_let_vars(eap->arg, &rettv, false, semicolon, var_count, is_const, op);
  }
  if (eval_res != FAIL) {
    tv_clear(&rettv);
  }
}

/// Assign the typevalue "tv" to the variable or variables at "arg_start".
/// Handles both "var" with any type and "[var, var; var]" with a list type.
/// When "op" is not NULL it points to a string with characters that
/// must appear after the variable(s).  Use "+", "-" or "." for add, subtract
/// or concatenate.
///
/// @param copy  copy values from "tv", don't move
/// @param semicolon  from skip_var_list()
/// @param var_count  from skip_var_list()
/// @param is_const  lock variables for :const
///
/// @return  OK or FAIL;
int ex_let_vars(char *arg_start, typval_T *tv, int copy, int semicolon, int var_count, int is_const,
                char *op)
{
  char *arg = arg_start;
  typval_T ltv;

  if (*arg != '[') {
    // ":let var = expr" or ":for var in list"
    if (ex_let_one(arg, tv, copy, is_const, op, op) == NULL) {
      return FAIL;
    }
    return OK;
  }

  // ":let [v1, v2] = list" or ":for [v1, v2] in listlist"
  if (tv->v_type != VAR_LIST) {
    emsg(_(e_listreq));
    return FAIL;
  }
  list_T *const l = tv->vval.v_list;

  const int len = tv_list_len(l);
  if (semicolon == 0 && var_count < len) {
    emsg(_("E687: Less targets than List items"));
    return FAIL;
  }
  if (var_count - semicolon > len) {
    emsg(_("E688: More targets than List items"));
    return FAIL;
  }
  // List l may actually be NULL, but it should fail with E688 or even earlier
  // if you try to do ":let [] = v:_null_list".
  assert(l != NULL);

  listitem_T *item = tv_list_first(l);
  size_t rest_len = (size_t)tv_list_len(l);
  while (*arg != ']') {
    arg = skipwhite(arg + 1);
    arg = ex_let_one(arg, TV_LIST_ITEM_TV(item), true, is_const, ",;]", op);
    if (arg == NULL) {
      return FAIL;
    }
    rest_len--;

    item = TV_LIST_ITEM_NEXT(l, item);
    arg = skipwhite(arg);
    if (*arg == ';') {
      // Put the rest of the list (may be empty) in the var after ';'.
      // Create a new list for this.
      list_T *const rest_list = tv_list_alloc((ptrdiff_t)rest_len);
      while (item != NULL) {
        tv_list_append_tv(rest_list, TV_LIST_ITEM_TV(item));
        item = TV_LIST_ITEM_NEXT(l, item);
      }

      ltv.v_type = VAR_LIST;
      ltv.v_lock = VAR_UNLOCKED;
      ltv.vval.v_list = rest_list;
      tv_list_ref(rest_list);

      arg = ex_let_one(skipwhite(arg + 1), &ltv, false, is_const, "]", op);
      tv_clear(&ltv);
      if (arg == NULL) {
        return FAIL;
      }
      break;
    } else if (*arg != ',' && *arg != ']') {
      internal_error("ex_let_vars()");
      return FAIL;
    }
  }

  return OK;
}

/// Skip over assignable variable "var" or list of variables "[var, var]".
/// Used for ":let varvar = expr" and ":for varvar in expr".
/// For "[var, var]" increment "*var_count" for each variable.
/// for "[var, var; var]" set "semicolon" to 1.
/// If "silent" is true do not give an "invalid argument" error message.
///
/// @return  NULL for an error.
const char *skip_var_list(const char *arg, int *var_count, int *semicolon, bool silent)
{
  return rs_skip_var_list(arg, var_count, semicolon, silent);
}

/// List variables for hashtab "ht" with prefix "prefix".
///
/// @param empty  if true also list NULL strings as empty strings.
void list_hashtable_vars(hashtab_T *ht, const char *prefix, int empty, int *first)
{
  rs_list_hashtable_vars(ht, prefix, empty, first);
}

/// List global variables.
static void list_glob_vars(int *first) { list_hashtable_vars(&globvarht, "", true, first); }

/// List buffer variables.
static void list_buf_vars(int *first) { list_hashtable_vars(&curbuf->b_vars->dv_hashtab, "b:", true, first); }

/// List window variables.
static void list_win_vars(int *first) { list_hashtable_vars(&curwin->w_vars->dv_hashtab, "w:", true, first); }

/// List tab page variables.
static void list_tab_vars(int *first) { list_hashtable_vars(&curtab->tp_vars->dv_hashtab, "t:", true, first); }

/// List variables in "arg".
static const char *list_arg_vars(exarg_T *eap, const char *arg, int *first)
{
  bool error = false;
  int len;
  const char *name;
  const char *name_start;
  typval_T tv;

  while (!ends_excmd(*arg) && !got_int) {
    if (error || eap->skip) {
      arg = rs_find_name_end(arg, NULL, NULL, FNE_INCL_BR | FNE_CHECK_START);
      if (!ascii_iswhite(*arg) && !ends_excmd(*arg)) {
        emsg_severe = true;
        semsg(_(e_trailing_arg), arg);
        break;
      }
    } else {
      // get_name_len() takes care of expanding curly braces
      name_start = name = arg;
      char *tofree;
      len = get_name_len(&arg, &tofree, true, true);
      if (len <= 0) {
        // This is mainly to keep test 49 working: when expanding
        // curly braces fails overrule the exception error message.
        if (len < 0 && !aborting()) {
          emsg_severe = true;
          semsg(_(e_invarg2), arg);
          break;
        }
        error = true;
      } else {
        if (tofree != NULL) {
          name = tofree;
        }
        if (eval_variable(name, len, &tv, NULL, true, false) == FAIL) {
          error = true;
        } else {
          // handle d.key, l[idx], f(expr)
          const char *const arg_subsc = arg;
          if (handle_subscript(&arg, &tv, &EVALARG_EVALUATE, true) == FAIL) {
            error = true;
          } else {
            if (arg == arg_subsc && len == 2 && name[1] == ':') {
              switch (*name) {
              case 'g':
                list_glob_vars(first); break;
              case 'b':
                list_buf_vars(first); break;
              case 'w':
                list_win_vars(first); break;
              case 't':
                list_tab_vars(first); break;
              case 'v':
                list_vim_vars(first); break;
              case 's':
                list_script_vars(first); break;
              case 'l':
                list_func_vars(first); break;
              default:
                semsg(_("E738: Can't list variables for %s"), name);
              }
            } else {
              char *const s = encode_tv2echo(&tv, NULL);
              const char *const used_name = (arg == arg_subsc
                                             ? name
                                             : name_start);
              assert(used_name != NULL);
              const ptrdiff_t name_size = (used_name == tofree
                                           ? (ptrdiff_t)strlen(used_name)
                                           : (arg - used_name));
              list_one_var_a("", used_name, name_size,
                             tv.v_type, s == NULL ? "" : s, first);
              xfree(s);
            }
            tv_clear(&tv);
          }
        }
      }

      xfree(tofree);
    }

    arg = skipwhite(arg);
  }

  return arg;
}

/// Set an environment variable, part of ex_let_one().
static char *ex_let_env(char *arg, typval_T *const tv, const bool is_const,
                        const char *const endchars, const char *const op)
  FUNC_ATTR_NONNULL_ARG(1, 2) FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_ex_let_env(arg, tv, is_const, endchars, op);
}

/// Set an option, part of ex_let_one().
static char *ex_let_option(char *arg, typval_T *const tv, const bool is_const,
                           const char *const endchars, const char *const op)
  FUNC_ATTR_NONNULL_ARG(1, 2) FUNC_ATTR_WARN_UNUSED_RESULT
{
  if (is_const) {
    emsg(_("E996: Cannot lock an option"));
    return NULL;
  }

  // Find the end of the name.
  char *arg_end = NULL;
  OptIndex opt_idx;
  int opt_flags;

  char *const p = (char *)find_option_var_end((const char **)&arg, &opt_idx, &opt_flags);

  if (p == NULL || (endchars != NULL && vim_strchr(endchars, (uint8_t)(*skipwhite(p))) == NULL)) {
    emsg(_(e_letunexp));
    return NULL;
  }

  const char c1 = *p;
  *p = NUL;

  bool is_tty_opt = rs_is_tty_option(arg);
  bool hidden = is_option_hidden(opt_idx);
  OptVal curval = is_tty_opt ? get_tty_option(arg) : get_option_value(opt_idx, opt_flags);
  OptVal newval = NIL_OPTVAL;

  if (curval.type == kOptValTypeNil) {
    semsg(_(e_unknown_option2), arg);
    goto theend;
  }
  if (op != NULL && *op != '='
      && ((curval.type != kOptValTypeString && *op == '.')
          || (curval.type == kOptValTypeString && *op != '.'))) {
    semsg(_(e_letwrong), op);
    goto theend;
  }

  bool error;
  newval = tv_to_optval(tv, opt_idx, arg, &error);
  if (error) {
    goto theend;
  }

  // Current value and new value must have the same type.
  assert(curval.type == newval.type);
  const bool is_num = curval.type == kOptValTypeNumber || curval.type == kOptValTypeBoolean;
  const bool is_string = curval.type == kOptValTypeString;

  if (op != NULL && *op != '=') {
    if (!hidden && is_num) {  // number or bool
      OptInt cur_n = curval.type == kOptValTypeNumber ? curval.data.number : curval.data.boolean;
      OptInt new_n = newval.type == kOptValTypeNumber ? newval.data.number : newval.data.boolean;

      switch (*op) {
      case '+':
        new_n = cur_n + new_n; break;
      case '-':
        new_n = cur_n - new_n; break;
      case '*':
        new_n = cur_n * new_n; break;
      case '/':
        new_n = rs_num_divide(cur_n, new_n); break;
      case '%':
        new_n = rs_num_modulus(cur_n, new_n); break;
      }

      if (curval.type == kOptValTypeNumber) {
        newval = NUMBER_OPTVAL(new_n);
      } else {
        newval = BOOLEAN_OPTVAL(TRISTATE_FROM_INT(new_n));
      }
    } else if (!hidden && is_string) {  // string
      const char *curval_data = curval.data.string.data;
      const char *newval_data = newval.data.string.data;

      if (curval_data != NULL && newval_data != NULL) {
        OptVal newval_old = newval;
        newval = CSTR_AS_OPTVAL(concat_str(curval_data, newval_data));
        rs_optval_free(newval_old);
      }
    }
  }

  const char *err = set_option_value_handle_tty(arg, opt_idx, newval, opt_flags);
  arg_end = p;
  if (err != NULL) {
    emsg(_(err));
  }

theend:
  *p = c1;
  rs_optval_free(curval);
  rs_optval_free(newval);
  return arg_end;
}

/// Set a register, part of ex_let_one().
static char *ex_let_register(char *arg, typval_T *const tv, const bool is_const,
                             const char *const endchars, const char *const op)
  FUNC_ATTR_NONNULL_ARG(1, 2) FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_ex_let_register(arg, tv, is_const, endchars, op);
}

/// Set one item of `:let var = expr` or `:let [v1, v2] = list` to its value
///
/// @param[in]  arg  Start of the variable name.
/// @param[in]  tv  Value to assign to the variable.
/// @param[in]  copy  If true, copy value from `tv`.
/// @param[in]  endchars  Valid characters after variable name or NULL.
/// @param[in]  op  Operation performed: *op is `+`, `-`, `.` for `+=`, etc.
///                 NULL for `=`.
///
/// @return a pointer to the char just after the var name or NULL in case of
///         error.
static char *ex_let_one(char *arg, typval_T *const tv, const bool copy, const bool is_const,
                        const char *const endchars, const char *const op)
  FUNC_ATTR_NONNULL_ARG(1, 2) FUNC_ATTR_WARN_UNUSED_RESULT
{
  char *arg_end = NULL;

  if (*arg == '$') {
    // ":let $VAR = expr": Set environment variable.
    return ex_let_env(arg, tv, is_const, endchars, op);
  } else if (*arg == '&') {
    // ":let &option = expr": Set option value.
    // ":let &l:option = expr": Set local option value.
    // ":let &g:option = expr": Set global option value.
    return ex_let_option(arg, tv, is_const, endchars, op);
  } else if (*arg == '@') {
    // ":let @r = expr": Set register contents.
    return ex_let_register(arg, tv, is_const, endchars, op);
  } else if (rs_eval_isnamec1(*arg) || *arg == '{') {
    // ":let var = expr": Set internal variable.
    // ":let {expr} = expr": Idem, name made with curly braces
    lval_T lv;
    char *const p = get_lval(arg, tv, &lv, false, false, 0, FNE_CHECK_START);
    if (p != NULL && lv.ll_name != NULL) {
      if (endchars != NULL && vim_strchr(endchars, (uint8_t)(*skipwhite(p))) == NULL) {
        emsg(_(e_letunexp));
      } else {
        set_var_lval(&lv, p, tv, copy, is_const, op);
        arg_end = p;
      }
    }
    clear_lval(&lv);
  } else {
    semsg(_(e_invarg2), arg);
  }

  return arg_end;
}

/// ":unlet[!] var1 ... " command.
void ex_unlet(exarg_T *eap) { rs_ex_unlet(eap); }

/// ":lockvar" and ":unlockvar" commands
void ex_lockvar(exarg_T *eap) { rs_ex_lockvar(eap); }

/// Common parsing logic for :unlet, :lockvar and :unlockvar.
///
/// Invokes `callback` afterwards if successful and `eap->skip == false`.
///
/// @param[in]  eap  Ex command arguments for the command.
/// @param[in]  argstart  Start of the string argument for the command.
/// @param[in]  deep  Levels to (un)lock for :(un)lockvar, -1 to (un)lock
///                   everything.
/// @param[in]  callback  Appropriate handler for the command.
static void ex_unletlock(exarg_T *eap, char *argstart, int deep, int glv_flags,
                         ex_unletlock_callback callback)
  FUNC_ATTR_NONNULL_ALL
{
  char *arg = argstart;
  char *name_end;
  bool error = false;
  lval_T lv;

  do {
    if (*arg == '$') {
      lv.ll_name = arg;
      lv.ll_tv = NULL;
      arg++;
      if (rs_get_env_len((const char **)&arg) == 0) {
        semsg(_(e_invarg2), arg - 1);
        return;
      }
      assert(*lv.ll_name == '$');  // suppress clang "Uninitialized argument value"
      if (!error && !eap->skip && callback(&lv, arg, eap, deep) == FAIL) {
        error = true;
      }
      name_end = arg;
    } else {
      // Parse the name and find the end.
      name_end = get_lval(arg, NULL, &lv, true, eap->skip || error,
                          glv_flags, FNE_CHECK_START);
      if (lv.ll_name == NULL) {
        error = true;  // error, but continue parsing.
      }
      if (name_end == NULL
          || (!ascii_iswhite(*name_end) && !ends_excmd(*name_end))) {
        if (name_end != NULL) {
          emsg_severe = true;
          semsg(_(e_trailing_arg), name_end);
        }
        if (!(eap->skip || error)) {
          clear_lval(&lv);
        }
        break;
      }

      if (!error && !eap->skip && callback(&lv, name_end, eap, deep) == FAIL) {
        error = true;
      }

      if (!eap->skip) {
        clear_lval(&lv);
      }
    }
    arg = skipwhite(name_end);
  } while (!ends_excmd(*arg));

  eap->nextcmd = check_nextcmd(arg);
}

/// Unlet a variable indicated by `lp`.
///
/// @param[in]  lp  The lvalue.
/// @param[in]  name_end  End of the string argument for the command.
/// @param[in]  eap  Ex command arguments for :unlet.
/// @param[in]  deep  Unused.
///
/// @return OK on success, or FAIL on failure.
static int do_unlet_var(lval_T *lp, char *name_end, exarg_T *eap, int deep FUNC_ATTR_UNUSED)
  FUNC_ATTR_NONNULL_ALL
{
  int forceit = eap->forceit;
  int ret = OK;

  if (lp->ll_tv == NULL) {
    int cc = (uint8_t)(*name_end);
    *name_end = NUL;

    // Environment variable, normal name or expanded name.
    if (*lp->ll_name == '$') {
      vim_unsetenv_ext(lp->ll_name + 1);
    } else if (do_unlet(lp->ll_name, lp->ll_name_len, forceit) == FAIL) {
      ret = FAIL;
    }
    *name_end = (char)cc;
  } else if ((lp->ll_list != NULL
              // ll_list is not NULL when lvalue is not in a list, NULL lists
              // yield E689.
              && value_check_lock(tv_list_locked(lp->ll_list),
                                  lp->ll_name,
                                  lp->ll_name_len))
             || (lp->ll_dict != NULL
                 && value_check_lock(lp->ll_dict->dv_lock,
                                     lp->ll_name,
                                     lp->ll_name_len))) {
    return FAIL;
  } else if (lp->ll_range) {
    tv_list_unlet_range(lp->ll_list, lp->ll_li, lp->ll_n1, !lp->ll_empty2, lp->ll_n2);
  } else if (lp->ll_list != NULL) {
    // unlet a List item.
    tv_list_item_remove(lp->ll_list, lp->ll_li);
  } else {
    // unlet a Dict item.
    dict_T *d = lp->ll_dict;
    assert(d != NULL);
    dictitem_T *di = lp->ll_di;
    bool watched = tv_dict_is_watched(d);
    char *key = NULL;
    typval_T oldtv;

    if (watched) {
      tv_copy(&di->di_tv, &oldtv);
      // need to save key because dictitem_remove will free it
      key = xstrdup(di->di_key);
    }

    tv_dict_item_remove(d, di);

    if (watched) {
      tv_dict_watcher_notify(d, key, NULL, &oldtv);
      tv_clear(&oldtv);
      xfree(key);
    }
  }

  return ret;
}

/// Unlet one item or a range of items from a list.
/// Return OK or FAIL.
static void tv_list_unlet_range(list_T *const l, listitem_T *const li_first, const int n1_arg,
                                const bool has_n2, const int n2)
{
  assert(l != NULL);
  // Delete a range of List items.
  listitem_T *li_last = li_first;
  int n1 = n1_arg;
  while (true) {
    listitem_T *const li = TV_LIST_ITEM_NEXT(l, li_last);
    n1++;
    if (li == NULL || (has_n2 && n2 < n1)) {
      break;
    }
    li_last = li;
  }
  tv_list_remove_items(l, li_first, li_last);
}

/// unlet a variable
///
/// @param[in]  name  Variable name to unlet.
/// @param[in]  name_len  Variable name length.
/// @param[in]  forceit  If true, do not complain if variable doesn’t exist.
///
/// @return OK if it existed, FAIL otherwise.
int do_unlet(const char *const name, const size_t name_len, const bool forceit)
{ return rs_do_unlet(name, name_len, forceit); }

/// Lock or unlock variable indicated by `lp`.
///
/// Locks if `eap->cmdidx == CMD_lockvar`, unlocks otherwise.
///
/// @param[in]  lp  The lvalue.
/// @param[in]  name_end  Unused.
/// @param[in]  eap  Ex command arguments for :(un)lockvar.
/// @param[in]  deep  Levels to (un)lock, -1 to (un)lock everything.
///
/// @return OK on success, or FAIL on failure.
static int do_lock_var(lval_T *lp, char *name_end FUNC_ATTR_UNUSED, exarg_T *eap, int deep)
  FUNC_ATTR_NONNULL_ARG(1, 3)
{
  bool lock = eap->cmdidx == CMD_lockvar;
  int ret = OK;

  if (lp->ll_tv == NULL) {
    if (*lp->ll_name == '$') {
      semsg(_(e_lock_unlock), lp->ll_name);
      ret = FAIL;
    } else {
      // Normal name or expanded name.
      dictitem_T *const di = find_var(lp->ll_name, lp->ll_name_len, NULL,
                                      true);
      if (di == NULL) {
        ret = FAIL;
      } else if ((di->di_flags & DI_FLAGS_FIX)
                 && di->di_tv.v_type != VAR_DICT
                 && di->di_tv.v_type != VAR_LIST) {
        // For historical reasons this error is not given for Lists and
        // Dictionaries. E.g. b: dictionary may be locked/unlocked.
        semsg(_(e_lock_unlock), lp->ll_name);
        ret = FAIL;
      } else {
        if (lock) {
          di->di_flags |= DI_FLAGS_LOCK;
        } else {
          di->di_flags &= (uint8_t)(~DI_FLAGS_LOCK);
        }
        if (deep != 0) {
          tv_item_lock(&di->di_tv, deep, lock, false);
        }
      }
    }
  } else if (deep == 0) {
    // nothing to do
  } else if (lp->ll_range) {
    listitem_T *li = lp->ll_li;

    // (un)lock a range of List items.
    while (li != NULL && (lp->ll_empty2 || lp->ll_n2 >= lp->ll_n1)) {
      tv_item_lock(TV_LIST_ITEM_TV(li), deep, lock, false);
      li = TV_LIST_ITEM_NEXT(lp->ll_list, li);
      lp->ll_n1++;
    }
  } else if (lp->ll_list != NULL) {
    // (un)lock a List item.
    tv_item_lock(TV_LIST_ITEM_TV(lp->ll_li), deep, lock, false);
  } else {
    // (un)lock a Dict item.
    tv_item_lock(&lp->ll_di->di_tv, deep, lock, false);
  }

  return ret;
}

/// Delete all "menutrans_" variables.
void del_menutrans_vars(void)
{
  hash_lock(&globvarht);
  HASHTAB_ITER(&globvarht, hi, {
    if (strncmp(hi->hi_key, "menutrans_", 10) == 0) {
      delete_var(&globvarht, hi);
    }
  });
  hash_unlock(&globvarht);
}

dict_T *get_globvar_dict(void) FUNC_ATTR_PURE FUNC_ATTR_NONNULL_RET { return &globvardict; }
hashtab_T *get_globvar_ht(void) { return &globvarht; }
dict_T *get_vimvar_dict(void) FUNC_ATTR_PURE FUNC_ATTR_NONNULL_RET { return &vimvardict; }

// v: variable get/set wrappers (logic in Rust vimvar_accessors.rs)
void set_vim_var_tv(const VimVarIndex idx, typval_T *const tv) { rs_set_vim_var_tv(idx, tv); }
char *get_vim_var_name(const VimVarIndex idx) FUNC_ATTR_NONNULL_RET
{ return rs_get_vim_var_name(idx); }
typval_T *get_vim_var_tv(const VimVarIndex idx) { return &vimvars[idx].vv_tv; }
varnumber_T get_vim_var_nr(const VimVarIndex idx) FUNC_ATTR_PURE { return rs_get_vim_var_nr(idx); }
list_T *get_vim_var_list(const VimVarIndex idx) FUNC_ATTR_PURE { return rs_get_vim_var_list(idx); }
dict_T *get_vim_var_dict(const VimVarIndex idx) FUNC_ATTR_PURE { return rs_get_vim_var_dict(idx); }
char *get_vim_var_str(const VimVarIndex idx) FUNC_ATTR_PURE FUNC_ATTR_NONNULL_RET
{ return rs_get_vim_var_str(idx); }
partial_T *get_vim_var_partial(const VimVarIndex idx) FUNC_ATTR_PURE
{ return rs_get_vim_var_partial(idx); }

/// Local string buffer for the next two functions to store a variable name
/// with its prefix. Allocated in cat_prefix_varname(), freed later in
/// get_user_var_name().

static char *varnamebuf = NULL;
static size_t varnamebuflen = 0;

/// Function to concatenate a prefix and a variable name.
char *cat_prefix_varname(int prefix, const char *name)
  FUNC_ATTR_NONNULL_ALL
{ return rs_cat_prefix_varname(prefix, name); }

/// Function given to ExpandGeneric() to obtain the list of user defined
/// (global/buffer/window/built-in) variable names.
char *get_user_var_name(expand_T *xp, int idx)
{ return rs_get_user_var_name(xp, idx); }

// More v: variable set wrappers (logic in Rust)
void set_vim_var_type(const VimVarIndex idx, const VarType type) { rs_set_vim_var_type(idx, type); }
void set_vim_var_nr(const VimVarIndex idx, const varnumber_T val) { rs_set_vim_var_nr(idx, val); }
void set_vim_var_bool(const VimVarIndex idx, const BoolVarValue val) { rs_set_vim_var_bool(idx, val); }
void set_vim_var_special(const VimVarIndex idx, const SpecialVarValue val) { rs_set_vim_var_special(idx, val); }
void set_vim_var_char(int c) { rs_set_vim_var_char(c); }
void set_vim_var_string(const VimVarIndex idx, const char *const val, const ptrdiff_t len)
{ rs_set_vim_var_string(idx, val, len); }
void set_vim_var_list(const VimVarIndex idx, list_T *const val) { rs_set_vim_var_list(idx, val); }
void set_vim_var_dict(const VimVarIndex idx, dict_T *const val) { rs_set_vim_var_dict(idx, val); }
void set_vim_var_partial(const VimVarIndex idx, partial_T *val) { rs_set_vim_var_partial(idx, val); }
void set_reg_var(int c) { rs_set_reg_var(c); }

char *v_exception(char *oldval) { return rs_v_exception(oldval); }

/// Set v:cmdarg.
/// If "eap" != NULL, use "eap" to generate the value and return the old value.
/// If "oldarg" != NULL, restore the value to "oldarg" and return NULL.
/// Must always be called in pairs!
char *set_cmdarg(exarg_T *eap, char *oldarg)
{
  return rs_set_cmdarg(eap, oldarg);
}

char *v_throwpoint(char *oldval) { return rs_v_throwpoint(oldval); }
void set_vcount(int64_t count, int64_t count1, bool set_prevcount)
{ rs_set_vcount(count, count1, set_prevcount); }

/// Get the value of internal variable "name".
/// Return OK or FAIL.  If OK is returned "rettv" must be cleared.
///
/// @param len  length of "name"
/// @param rettv  NULL when only checking existence
/// @param dip  non-NULL when typval's dict item is needed
/// @param verbose  may give error message
/// @param no_autoload  do not use script autoloading
int eval_variable(const char *name, int len, typval_T *rettv, dictitem_T **dip, bool verbose,
                  bool no_autoload)
{
  int ret = OK;
  typval_T *tv = NULL;
  dictitem_T *v;

  v = find_var(name, (size_t)len, NULL, no_autoload);
  if (v != NULL) {
    tv = &v->di_tv;
    if (dip != NULL) {
      *dip = v;
    }
  }

  if (tv == NULL) {
    if (rettv != NULL && verbose) {
      semsg(_("E121: Undefined variable: %.*s"), len, name);
    }
    ret = FAIL;
  } else if (rettv != NULL) {
    tv_copy(tv, rettv);
  }

  return ret;
}

/// Check if variable "name[len]" is a local variable or an argument.
/// If so, "*eval_lavars_used" is set to true.
void check_vars(const char *name, size_t len)
{
  if (eval_lavars_used == NULL) {
    return;
  }

  const char *varname;
  hashtab_T *ht = find_var_ht(name, len, &varname);

  if (ht == get_funccal_local_ht() || ht == get_funccal_args_ht()) {
    if (find_var(name, len, NULL, true) != NULL) {
      *eval_lavars_used = true;
    }
  }
}

/// Find variable "name" in the list of variables.
/// Careful: "a:0" variables don't have a name.
/// When "htp" is not NULL we are writing to the variable, set "htp" to the
/// hashtab_T used.
///
/// @return  a pointer to it if found, NULL if not found.
dictitem_T *find_var(const char *const name, const size_t name_len, hashtab_T **htp,
                     int no_autoload)
{
  const char *varname;
  hashtab_T *const ht = find_var_ht(name, name_len, &varname);
  if (htp != NULL) {
    *htp = ht;
  }
  if (ht == NULL) {
    return NULL;
  }
  dictitem_T *const ret = find_var_in_ht(ht, *name,
                                         varname,
                                         name_len - (size_t)(varname - name),
                                         no_autoload || htp != NULL);
  if (ret != NULL) {
    return ret;
  }

  // Search in parent scope for lambda
  return find_var_in_scoped_ht(name, name_len, no_autoload || htp != NULL);
}

/// Find variable in hashtab.
/// When "varname" is empty returns curwin/curtab/etc vars dictionary.
///
/// @param[in]  ht  Hashtab to find variable in.
/// @param[in]  htname  Hashtab name (first character).
/// @param[in]  varname  Variable name.
/// @param[in]  varname_len  Variable name length.
/// @param[in]  no_autoload  If true then autoload scripts will not be sourced
///                          if autoload variable was not found.
///
/// @return pointer to the dictionary item with the found variable or NULL if it
///         was not found.
dictitem_T *find_var_in_ht(hashtab_T *const ht, int htname, const char *const varname,
                           const size_t varname_len, int no_autoload)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  if (varname_len == 0) {
    // Must be something like "s:", otherwise "ht" would be NULL.
    switch (htname) {
    case 's':
      return (dictitem_T *)&SCRIPT_SV(current_sctx.sc_sid)->sv_var;
    case 'g':
      return (dictitem_T *)&globvars_var;
    case 'v':
      return (dictitem_T *)&vimvars_var;
    case 'b':
      return (dictitem_T *)&curbuf->b_bufvar;
    case 'w':
      return (dictitem_T *)&curwin->w_winvar;
    case 't':
      return (dictitem_T *)&curtab->tp_winvar;
    case 'l':
      return get_funccal_local_var();
    case 'a':
      return get_funccal_args_var();
    }
    return NULL;
  }

  hashitem_T *hi = hash_find_len(ht, varname, varname_len);
  if (HASHITEM_EMPTY(hi)) {
    // For global variables we may try auto-loading the script.  If it
    // worked find the variable again.  Don't auto-load a script if it was
    // loaded already, otherwise it would be loaded every time when
    // checking if a function name is a Funcref variable.
    if (ht == get_globvar_ht() && !no_autoload) {
      // Note: script_autoload() may make "hi" invalid. It must either
      // be obtained again or not used.
      if (!script_autoload(varname, varname_len, false) || aborting()) {
        return NULL;
      }
      hi = hash_find_len(ht, varname, varname_len);
    }
    if (HASHITEM_EMPTY(hi)) {
      return NULL;
    }
  }
  return TV_DICT_HI2DI(hi);
}

/// Finds the dict (g:, l:, s:, …) and hashtable used for a variable.
///
/// Assigns SID if s: scope is accessed from Lua or anonymous Vimscript. #15994
///
/// @param[in]  name  Variable name, possibly with scope prefix.
/// @param[in]  name_len  Variable name length.
/// @param[out]  varname  Will be set to the start of the name without scope
///                       prefix.
/// @param[out]  d  Scope dictionary.
///
/// @return Scope hashtab, NULL if name is not valid.
static hashtab_T *find_var_ht_dict(const char *name, const size_t name_len, const char **varname,
                                   dict_T **d)
{
  *d = NULL;

  if (name_len == 0) {
    return NULL;
  }
  if (name_len == 1 || name[1] != ':') {
    // name has implicit scope
    if (name[0] == ':' || name[0] == AUTOLOAD_CHAR) {
      // The name must not start with a colon or #.
      return NULL;
    }
    *varname = name;

    // "version" is "v:version" in all scopes
    hashitem_T *hi = hash_find_len(&compat_hashtab, name, name_len);
    if (!HASHITEM_EMPTY(hi)) {
      return &compat_hashtab;
    }

    *d = get_funccal_local_dict();
    if (*d != NULL) {  // local variable
      goto end;
    }

    *d = get_globvar_dict();  // global variable
    goto end;
  }

  *varname = name + 2;
  if (*name == 'g') {  // global variable
    *d = get_globvar_dict();
  } else if (name_len > 2
             && (memchr(name + 2, ':', name_len - 2) != NULL
                 || memchr(name + 2, AUTOLOAD_CHAR, name_len - 2) != NULL)) {
    // There must be no ':' or '#' in the rest of the name if g: was not used
    return NULL;
  }

  if (*name == 'b') {  // buffer variable
    *d = curbuf->b_vars;
  } else if (*name == 'w') {  // window variable
    *d = curwin->w_vars;
  } else if (*name == 't') {  // tab page variable
    *d = curtab->tp_vars;
  } else if (*name == 'v') {  // v: variable
    *d = get_vimvar_dict();
  } else if (*name == 'a') {  // a: function argument
    *d = get_funccal_args_dict();
  } else if (*name == 'l') {  // l: local variable
    *d = get_funccal_local_dict();
  } else if (*name == 's'  // script variable
             && (current_sctx.sc_sid > 0 || current_sctx.sc_sid == SID_STR
                 || current_sctx.sc_sid == SID_LUA)
             && current_sctx.sc_sid <= script_items.ga_len) {
    // For anonymous scripts without a script item, create one now so script vars can be used
    // Try to resolve lua filename & linenr so it can be shown in last-set messages.
    nlua_set_sctx(&current_sctx);
    if (current_sctx.sc_sid == SID_STR || current_sctx.sc_sid == SID_LUA) {
      // Create SID if s: scope is accessed from Lua or anon Vimscript. #15994
      new_script_item(NULL, &current_sctx.sc_sid);
    }
    *d = &SCRIPT_SV(current_sctx.sc_sid)->sv_dict;
  }

end:
  return *d ? &(*d)->dv_hashtab : NULL;
}

/// Find the hashtable used for a variable
///
/// @param[in]  name  Variable name, possibly with scope prefix.
/// @param[in]  name_len  Variable name length.
/// @param[out]  varname  Will be set to the start of the name without scope
///                       prefix.
///
/// @return Scope hashtab, NULL if name is not valid.
hashtab_T *find_var_ht(const char *name, const size_t name_len, const char **varname)
{
  dict_T *d;
  return find_var_ht_dict(name, name_len, varname, &d);
}

/// @return  the string value of a (global/local) variable or
///          NULL when it doesn't exist.
///
/// @see  tv_get_string() for how long the pointer remains valid.
char *get_var_value(const char *const name)
{
  dictitem_T *v;

  v = find_var(name, strlen(name), NULL, false);
  if (v == NULL) {
    return NULL;
  }
  return (char *)tv_get_string(&v->di_tv);
}

/// Allocate a new hashtab for a sourced script.  It will be used while
/// sourcing this script and when executing functions defined in the script.
void new_script_vars(scid_T id)
{
  scriptvar_T *sv = xcalloc(1, sizeof(scriptvar_T));
  init_var_dict(&sv->sv_dict, &sv->sv_var, VAR_SCOPE);
  SCRIPT_ITEM(id)->sn_vars = sv;
}

/// Initialize dictionary "dict" as a scope and set variable "dict_var" to
/// point to it.
void init_var_dict(dict_T *dict, ScopeDictDictItem *dict_var, ScopeType scope)
{
  hash_init(&dict->dv_hashtab);
  dict->dv_lock = VAR_UNLOCKED;
  dict->dv_scope = scope;
  dict->dv_refcount = DO_NOT_FREE_CNT;
  dict->dv_copyID = 0;
  dict_var->di_tv.vval.v_dict = dict;
  dict_var->di_tv.v_type = VAR_DICT;
  dict_var->di_tv.v_lock = VAR_FIXED;
  dict_var->di_flags = DI_FLAGS_RO | DI_FLAGS_FIX;
  dict_var->di_key[0] = NUL;
  QUEUE_INIT(&dict->watchers);
}

/// Unreference a dictionary initialized by init_var_dict().
void unref_var_dict(dict_T *dict)
{
  // Now the dict needs to be freed if no one else is using it, go back to
  // normal reference counting.
  dict->dv_refcount -= DO_NOT_FREE_CNT - 1;
  tv_dict_unref(dict);
}

/// Clean up a list of internal variables.
/// Frees all allocated variables and the value they contain.
/// Clears hashtab "ht", does not free it.
void vars_clear(hashtab_T *ht) { vars_clear_ext(ht, true); }

/// Like vars_clear(), but only free the value if "free_val" is true.
void vars_clear_ext(hashtab_T *ht, bool free_val)
{
  int todo;
  hashitem_T *hi;
  dictitem_T *v;

  hash_lock(ht);
  todo = (int)ht->ht_used;
  for (hi = ht->ht_array; todo > 0; hi++) {
    if (!HASHITEM_EMPTY(hi)) {
      todo--;

      // Free the variable.  Don't remove it from the hashtab,
      // ht_array might change then.  hash_clear() takes care of it
      // later.
      v = TV_DICT_HI2DI(hi);
      if (free_val) {
        tv_clear(&v->di_tv);
      }
      if (v->di_flags & DI_FLAGS_ALLOC) {
        xfree(v);
      }
    }
  }
  hash_clear(ht);
  hash_init(ht);
}

/// Delete a variable from hashtab "ht" at item "hi".
/// Clear the variable value and free the dictitem.
static void delete_var(hashtab_T *ht, hashitem_T *hi)
{
  dictitem_T *di = TV_DICT_HI2DI(hi);

  hash_remove(ht, hi);
  tv_clear(&di->di_tv);
  xfree(di);
}

/// List the value of one internal variable.
static void list_one_var(dictitem_T *v, const char *prefix, int *first)
{
  rs_list_one_var(v, prefix, first);
}

/// @param[in]  name_len  Length of the name. May be -1, in this case strlen()
///                       will be used.
/// @param[in,out]  first  When true clear rest of screen and set to false.
static void list_one_var_a(const char *prefix, const char *name, const ptrdiff_t name_len,
                           const VarType type, const char *string, int *first)
{
  rs_list_one_var_a(prefix, name, name_len, type, string, first);
}

/// Additional handling for setting a v: variable.
///
/// @return  true if the variable should be set normally,
///          false if nothing else needs to be done.
bool before_set_vvar(const char *const varname, dictitem_T *const di, typval_T *const tv,
                     const bool copy, const bool watched, bool *const type_error)
{
  if (di->di_tv.v_type == VAR_STRING) {
    typval_T oldtv = TV_INITIAL_VALUE;
    if (watched) {
      tv_copy(&di->di_tv, &oldtv);
    }
    XFREE_CLEAR(di->di_tv.vval.v_string);
    if (copy || tv->v_type != VAR_STRING) {
      const char *const val = tv_get_string(tv);
      // Careful: when assigning to v:errmsg and tv_get_string()
      // causes an error message the variable will already be set.
      if (di->di_tv.vval.v_string == NULL) {
        di->di_tv.vval.v_string = xstrdup(val);
      }
    } else {
      // Take over the string to avoid an extra alloc/free.
      di->di_tv.vval.v_string = tv->vval.v_string;
      tv->vval.v_string = NULL;
    }
    // Notify watchers
    if (watched) {
      tv_dict_watcher_notify(&vimvardict, varname, &di->di_tv, &oldtv);
      tv_clear(&oldtv);
    }
    return false;
  } else if (di->di_tv.v_type == VAR_NUMBER) {
    typval_T oldtv = TV_INITIAL_VALUE;
    if (watched) {
      tv_copy(&di->di_tv, &oldtv);
    }
    di->di_tv.vval.v_number = tv_get_number(tv);
    if (strcmp(varname, "searchforward") == 0) {
      set_search_direction(di->di_tv.vval.v_number ? '/' : '?');
    } else if (strcmp(varname, "hlsearch") == 0) {
      no_hlsearch = !di->di_tv.vval.v_number;
      redraw_all_later(UPD_SOME_VALID);
    }
    // Notify watchers
    if (watched) {
      tv_dict_watcher_notify(&vimvardict, varname, &di->di_tv, &oldtv);
      tv_clear(&oldtv);
    }
    return false;
  } else if (di->di_tv.v_type != tv->v_type) {
    *type_error = true;
    return false;
  }
  return true;
}

/// Set variable to the given value
///
/// If the variable already exists, the value is updated. Otherwise the variable
/// is created.
///
/// @param[in]  name  Variable name to set.
/// @param[in]  name_len  Length of the variable name.
/// @param  tv  Variable value.
/// @param[in]  copy  True if value in tv is to be copied.
void set_var(const char *name, const size_t name_len, typval_T *const tv, const bool copy)
  FUNC_ATTR_NONNULL_ALL
{
  set_var_const(name, name_len, tv, copy, false);
}

/// Set variable to the given value
///
/// If the variable already exists, the value is updated. Otherwise the variable
/// is created.
///
/// @param[in]  name  Variable name to set.
/// @param[in]  name_len  Length of the variable name.
/// @param  tv  Variable value.
/// @param[in]  copy  True if value in tv is to be copied.
/// @param[in]  is_const  True if value in tv is to be locked.
void set_var_const(const char *name, const size_t name_len, typval_T *const tv, const bool copy,
                   const bool is_const)
  FUNC_ATTR_NONNULL_ALL
{
  const char *varname;
  dict_T *dict;
  hashtab_T *ht = find_var_ht_dict(name, name_len, &varname, &dict);
  const bool watched = tv_dict_is_watched(dict);

  if (ht == NULL || *varname == NUL) {
    semsg(_(e_illvar), name);
    return;
  }
  const size_t varname_len = name_len - (size_t)(varname - name);
  dictitem_T *di = find_var_in_ht(ht, 0, varname, varname_len, true);

  // Search in parent scope which is possible to reference from lambda
  if (di == NULL) {
    di = find_var_in_scoped_ht(name, name_len, true);
  }

  if (tv_is_func(*tv) && var_wrong_func_name(name, di == NULL)) {
    return;
  }

  typval_T oldtv = TV_INITIAL_VALUE;
  if (di != NULL) {
    if (is_const) {
      emsg(_(e_cannot_mod));
      return;
    }

    // Check in this order for backwards compatibility:
    // - Whether the variable is read-only
    // - Whether the variable value is locked
    // - Whether the variable is locked
    if (var_check_ro(di->di_flags, name, name_len)
        || value_check_lock(di->di_tv.v_lock, name, name_len)
        || var_check_lock(di->di_flags, name, name_len)) {
      return;
    }

    // existing variable, need to clear the value

    // Handle setting internal v: variables separately where needed to
    // prevent changing the type.
    bool type_error = false;
    if (ht == &vimvarht
        && !before_set_vvar(varname, di, tv, copy, watched, &type_error)) {
      if (type_error) {
        semsg(_(e_setting_v_str_to_value_with_wrong_type), varname);
      }
      return;
    }

    if (watched) {
      tv_copy(&di->di_tv, &oldtv);
    }
    tv_clear(&di->di_tv);
  } else {  // Add a new variable.
    // Can't add "v:" or "a:" variable.
    if (ht == &vimvarht || ht == get_funccal_args_ht()) {
      semsg(_(e_illvar), name);
      return;
    }

    // Make sure the variable name is valid.
    if (!valid_varname(varname)) {
      return;
    }

    // Make sure dict is valid
    assert(dict != NULL);

    di = xmalloc(offsetof(dictitem_T, di_key) + varname_len + 1);
    memcpy(di->di_key, varname, varname_len + 1);
    if (hash_add(ht, di->di_key) == FAIL) {
      xfree(di);
      return;
    }
    di->di_flags = DI_FLAGS_ALLOC;
    if (is_const) {
      di->di_flags |= DI_FLAGS_LOCK;
    }
  }

  if (copy || tv->v_type == VAR_NUMBER || tv->v_type == VAR_FLOAT) {
    tv_copy(tv, &di->di_tv);
  } else {
    di->di_tv = *tv;
    di->di_tv.v_lock = VAR_UNLOCKED;
    tv_init(tv);
  }

  if (watched) {
    tv_dict_watcher_notify(dict, di->di_key, &di->di_tv, &oldtv);
    tv_clear(&oldtv);
  }

  if (is_const) {
    // Like :lockvar! name: lock the value and what it contains, but only
    // if the reference count is up to one.  That locks only literal
    // values.
    tv_item_lock(&di->di_tv, DICT_MAXNEST, true, true);
  }
}

// Variable check wrappers (logic in Rust checks.rs)
bool var_check_ro(const int flags, const char *name, size_t name_len)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{ return rs_var_check_ro(flags, name, name_len); }
bool var_check_lock(const int flags, const char *name, size_t name_len)
{ return rs_var_check_lock(flags, name, name_len); }
bool var_check_fixed(const int flags, const char *name, size_t name_len)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{ return rs_var_check_fixed(flags, name, name_len); }
bool var_wrong_func_name(const char *const name, const bool new_var)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT
{ return rs_var_wrong_func_name(name, new_var); }
bool valid_varname(const char *varname)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT
{ return rs_valid_varname(varname); }

/// Implements the logic to retrieve local variable and option values.
/// Used by "getwinvar()" "gettabvar()" "gettabwinvar()" "getbufvar()".
///
/// @param deftv   default value if not found
/// @param htname  't'ab, 'w'indow or 'b'uffer local
/// @param tp      can be NULL
/// @param buf     ignored if htname is not 'b'
static void get_var_from(const char *varname, typval_T *rettv, typval_T *deftv, int htname,
                         tabpage_T *tp, win_T *win, buf_T *buf)
{
  rs_get_var_from(varname, rettv, deftv, htname, tp, win, buf);
}

/// getwinvar() and gettabwinvar()
///
/// @param off  1 for gettabwinvar()
static void getwinvar(typval_T *argvars, typval_T *rettv, int off)
{
  if (off == 1) {
    rs_f_gettabwinvar(argvars, rettv);
  } else {
    rs_f_getwinvar(argvars, rettv);
  }
}

/// Convert typval to option value for a particular option.
///
/// @param[in]   tv      typval to convert.
/// @param[in]   option  Option name.
/// @param[in]   flags   Option flags.
/// @param[out]  error   Whether an error occurred.
///
/// @return  Typval converted to OptVal. Must be freed by caller.
///          Returns NIL_OPTVAL for invalid option name.
static OptVal tv_to_optval(typval_T *tv, OptIndex opt_idx, const char *option, bool *error)
{
  return rs_tv_to_optval(tv, opt_idx, option, error);
}

/// Convert an option value to typval.
///
/// @param[in]  value    Option value to convert.
/// @param      numbool  Whether to convert boolean values to number.
///                      Used for backwards compatibility.
///
/// @return  OptVal converted to typval.
typval_T optval_as_tv(OptVal value, bool numbool)
{
  typval_T rettv;
  rs_optval_as_tv(value, numbool, &rettv);
  return rettv;
}

/// Set option "varname" to the value of "varp" for the current buffer/window.
static void set_option_from_tv(const char *varname, typval_T *varp)
{
  rs_set_option_from_tv(varname, varp);
}

/// "setwinvar()" and "settabwinvar()" functions
static void setwinvar(typval_T *argvars, int off)
{
  if (off == 1) {
    rs_f_settabwinvar(argvars);
  } else {
    rs_f_setwinvar(argvars);
  }
}

void reset_v_option_vars(void) { rs_reset_v_option_vars(); }
void assert_error(garray_T *gap) { rs_assert_error(gap->ga_data, gap->ga_len); }

bool var_exists(const char *var)
  FUNC_ATTR_NONNULL_ALL
{
  char *tofree;
  bool n = false;

  // get_name_len() takes care of expanding curly braces
  const char *name = var;
  const int len = get_name_len(&var, &tofree, true, false);
  if (len > 0) {
    typval_T tv;

    if (tofree != NULL) {
      name = tofree;
    }
    n = eval_variable(name, len, &tv, NULL, false, true) == OK;
    if (n) {
      // Handle d.key, l[idx], f(expr).
      n = handle_subscript(&var, &tv, &EVALARG_EVALUATE, false) == OK;
      if (n) {
        tv_clear(&tv);
      }
    }
  }
  if (*var != NUL) {
    n = false;
  }

  xfree(tofree);
  return n;
}

static lval_T *redir_lval = NULL;
static garray_T redir_ga;  // Only valid when redir_lval is not NULL.
static char *redir_endp = NULL;
static char *redir_varname = NULL;

/// Start recording command output to a variable
///
/// @param append  append to an existing variable
///
/// @return  OK if successfully completed the setup.  FAIL otherwise.
int var_redir_start(char *name, bool append)
{
  // Catch a bad name early.
  if (!rs_eval_isnamec1(*name)) {
    emsg(_(e_invarg));
    return FAIL;
  }

  // Make a copy of the name, it is used in redir_lval until redir ends.
  redir_varname = xstrdup(name);

  redir_lval = xcalloc(1, sizeof(lval_T));

  // The output is stored in growarray "redir_ga" until redirection ends.
  ga_init(&redir_ga, (int)sizeof(char), 500);

  // Parse the variable name (can be a dict or list entry).
  redir_endp = get_lval(redir_varname, NULL, redir_lval, false, false,
                        0, FNE_CHECK_START);
  if (redir_endp == NULL || redir_lval->ll_name == NULL
      || *redir_endp != NUL) {
    clear_lval(redir_lval);
    if (redir_endp != NULL && *redir_endp != NUL) {
      // Trailing characters are present after the variable name
      semsg(_(e_trailing_arg), redir_endp);
    } else {
      semsg(_(e_invarg2), name);
    }
    redir_endp = NULL;      // don't store a value, only cleanup
    var_redir_stop();
    return FAIL;
  }

  // check if we can write to the variable: set it to or append an empty
  // string
  const int called_emsg_before = called_emsg;
  did_emsg = false;
  typval_T tv;
  tv.v_type = VAR_STRING;
  tv.vval.v_string = "";
  if (append) {
    set_var_lval(redir_lval, redir_endp, &tv, true, false, ".");
  } else {
    set_var_lval(redir_lval, redir_endp, &tv, true, false, "=");
  }
  clear_lval(redir_lval);
  if (called_emsg > called_emsg_before) {
    redir_endp = NULL;      // don't store a value, only cleanup
    var_redir_stop();
    return FAIL;
  }

  return OK;
}

/// Append "value[value_len]" to the variable set by var_redir_start().
/// The actual appending is postponed until redirection ends, because the value
/// appended may in fact be the string we write to, changing it may cause freed
/// memory to be used:
///   :redir => foo
///   :let foo
///   :redir END
void var_redir_str(const char *value, int value_len)
{ rs_var_redir_str(value, value_len); }

/// Stop redirecting command output to a variable.
/// Frees the allocated memory.
void var_redir_stop(void)
{
  if (redir_lval != NULL) {
    // If there was no error: assign the text to the variable.
    if (redir_endp != NULL) {
      ga_append(&redir_ga, NUL);        // Append the trailing NUL.
      typval_T tv;
      tv.v_type = VAR_STRING;
      tv.vval.v_string = redir_ga.ga_data;
      // Call get_lval() again, if it's inside a Dict or List it may
      // have changed.
      redir_endp = get_lval(redir_varname, NULL, redir_lval,
                            false, false, 0, FNE_CHECK_START);
      if (redir_endp != NULL && redir_lval->ll_name != NULL) {
        set_var_lval(redir_lval, redir_endp, &tv, false, false, ".");
      }
      clear_lval(redir_lval);
    }

    // free the collected output
    XFREE_CLEAR(redir_ga.ga_data);

    XFREE_CLEAR(redir_lval);
  }
  XFREE_CLEAR(redir_varname);
}

/// "gettabvar()" function
void f_gettabvar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{ rs_f_gettabvar(argvars, rettv); }

/// "gettabwinvar()" function
void f_gettabwinvar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { getwinvar(argvars, rettv, 1); }

/// "getwinvar()" function
void f_getwinvar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { getwinvar(argvars, rettv, 0); }

/// "getbufvar()" function
void f_getbufvar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{ rs_f_getbufvar(argvars, rettv); }

/// "settabvar()" function
void f_settabvar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{ rs_f_settabvar(argvars); }

/// "settabwinvar()" function
void f_settabwinvar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { setwinvar(argvars, 1); }

/// "setwinvar()" function
void f_setwinvar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { setwinvar(argvars, 0); }

/// "setbufvar()" function
void f_setbufvar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{ rs_f_setbufvar(argvars); }

// Rust FFI Accessor Functions

/// Get the current buffer's variables dictionary.
dict_T *nvim_curbuf_get_vars(void) { return curbuf->b_vars; }

/// Get the current window's variables dictionary.
dict_T *nvim_curwin_get_vars(void) { return curwin->w_vars; }

/// Get the current tab's variables dictionary.
dict_T *nvim_curtab_get_vars(void) { return curtab->tp_vars; }

/// Get the hashtab from a dictionary.
hashtab_T *nvim_dict_get_hashtab(dict_T *dict)
{
  if (dict == NULL) {
    return NULL;
  }
  return &dict->dv_hashtab;
}

/// Check if a hashitem is empty.
int nvim_hashitem_empty(hashitem_T *hi) { return HASHITEM_EMPTY(hi); }

/// Convert hashitem to dictitem.
dictitem_T *nvim_hi2dictitem(hashitem_T *hi) { return TV_DICT_HI2DI(hi); }

/// Get the compat hashtab (for variables accessible without v: prefix).
hashtab_T *nvim_get_compat_hashtab(void) { return &compat_hashtab; }

/// Get script variables dictionary by script ID.
dict_T *nvim_get_script_vars_dict(int sid)
{
  if (sid <= 0 || sid > script_items.ga_len) {
    return NULL;
  }
  scriptvar_T *sv = SCRIPT_SV(sid);
  return sv ? &sv->sv_dict : NULL;
}

/// Get current script context SID.
int nvim_get_current_sctx_sid(void) { return current_sctx.sc_sid; }

/// Get typval from dictitem.
typval_T *nvim_dictitem_get_tv(dictitem_T *di)
{
  if (di == NULL) {
    return NULL;
  }
  return &di->di_tv;
}

// Phase 4: typval_T field accessor functions for Rust FFI
// (nvim_tv_get_type, nvim_tv_set_type, nvim_tv_get/set_number, nvim_tv_get/set_bool,
//  nvim_tv_get/set_list, nvim_tv_get/set_dict, nvim_tv_get_lock already in typval.c)

/// Get vval.v_special from typval_T.
int nvim_tv_get_special(const typval_T *tv) { return (int)tv->vval.v_special; }

/// Set vval.v_special in typval_T.
void nvim_tv_set_special(typval_T *tv, int val) { tv->vval.v_special = (SpecialVarValue)val; }

/// Get vval.v_string from typval_T.
char *nvim_tv_get_string_val(const typval_T *tv) { return tv->vval.v_string; }

/// Set vval.v_string in typval_T (raw pointer assignment, no copy).
void nvim_tv_set_string_val(typval_T *tv, char *s) { tv->vval.v_string = s; }

/// Get vval.v_partial from typval_T.
partial_T *nvim_tv_get_partial(const typval_T *tv) { return tv->vval.v_partial; }

/// Set vval.v_partial in typval_T.
void nvim_tv_set_partial(typval_T *tv, partial_T *p) { tv->vval.v_partial = p; }

/// Get vimvars[idx].vv_name.
char *nvim_vimvar_get_name(int idx) { return vimvars[idx].vv_name; }

/// Get vimvars[idx].vv_flags.
int nvim_vimvar_get_flags(int idx) { return (int)vimvars[idx].vv_flags; }

/// Set vimvars[idx].vv_type to VAR_UNKNOWN (used in prepare_vimvar).
void nvim_vimvar_set_str_null(int idx) { vimvars[idx].vv_str = NULL; }

/// Get vimvarht pointer (for prepare_vimvar/restore_vimvar).
hashtab_T *nvim_get_vimvarht(void) { return &vimvarht; }

/// Get globvarht pointer (for del_menutrans_vars).
hashtab_T *nvim_get_globvarht(void) { return &globvarht; }

/// Increment a dict_T refcount (for set_vim_var_dict).
/// nvim_dict_incr_refcount is not in any other shim file, so define it here.
void nvim_dict_incr_refcount(dict_T *d) { if (d != NULL) { d->dv_refcount++; } }

/// Make all dict keys read-only (for set_vim_var_dict).
/// nvim_dict_set_keys_readonly is not in any other shim file, so define here.
void nvim_dict_set_keys_readonly(dict_T *d) { tv_dict_set_keys_readonly(d); }

/// Emit E5700 error for invalid spellsuggest expression result.
void nvim_vars_emsg_e5700(void)
{
  emsg(_("E5700: Expression from 'spellsuggest' must yield lists with "
         "exactly two values"));
}

// Phase 2: buf_T / win_T / tabpage_T field accessors for Rust FFI
// Using void* to avoid exposing buffer_defs.h types in generated headers.

/// Non-inline wrapper for tv_dict_set_ret (for Rust FFI).
void nvim_tv_dict_set_ret(typval_T *tv, dict_T *d) { tv_dict_set_ret(tv, d); }

/// Get buf->b_vars (variables dict).
void *nvim_buf_get_vars(void *buf) { return buf ? ((buf_T *)buf)->b_vars : NULL; }

/// Get win->w_vars (variables dict).
void *nvim_win_get_vars(void *win) { return win ? ((win_T *)win)->w_vars : NULL; }

/// Get tp->tp_vars (variables dict).
void *nvim_tab_get_vars(void *tp) { return tp ? ((tabpage_T *)tp)->tp_vars : NULL; }

/// Get &buf->b_bufvar.di_tv (scope dict item tv).
void *nvim_buf_get_bufvar_tv(void *buf) { return buf ? &((buf_T *)buf)->b_bufvar.di_tv : NULL; }

/// Get &win->w_winvar.di_tv (scope dict item tv).
void *nvim_win_get_winvar_tv(void *win) { return win ? &((win_T *)win)->w_winvar.di_tv : NULL; }

/// Get &tp->tp_winvar.di_tv (scope dict item tv).
void *nvim_tab_get_winvar_tv(void *tp) { return tp ? &((tabpage_T *)tp)->tp_winvar.di_tv : NULL; }

/// Get tp->tp_firstwin.
void *nvim_tab_get_firstwin(void *tp) { return tp ? ((tabpage_T *)tp)->tp_firstwin : NULL; }

/// Increment emsg_off.
void nvim_emsg_off_inc(void) { emsg_off++; }

/// Decrement emsg_off.
void nvim_emsg_off_dec(void) { emsg_off--; }

/// Heap-allocate a switchwin_T and call switch_win; returns opaque pointer or NULL on fail.
/// Caller must call nvim_vars_switch_win_restore() later.
void *nvim_vars_switch_win(void *win, void *tp)
{
  switchwin_T *sw = xcalloc(1, sizeof(switchwin_T));
  if (switch_win(sw, (win_T *)win, (tabpage_T *)tp, true) != OK) {
    xfree(sw);
    return NULL;
  }
  return sw;
}

/// Call restore_win on a heap-allocated switchwin_T and free it.
void nvim_vars_switch_win_restore(void *sw)
{
  if (sw != NULL) {
    restore_win((switchwin_T *)sw, true);
    xfree(sw);
  }
}

/// Check if win and tp are the current win/tab pair.
bool nvim_is_curwin_curtab(void *win, void *tp)
{
  return (tabpage_T *)tp == curtab && (win_T *)win == curwin;
}

/// Call goto_tabpage_tp (wraps bool args as int).
void nvim_goto_tabpage_tp(void *tp, int trigger_enter, int trigger_leave)
{
  goto_tabpage_tp((tabpage_T *)tp, (bool)trigger_enter, (bool)trigger_leave);
}

// Phase 3: hashtab iteration and listing accessors for Rust FFI

/// Get ht->ht_array (pointer to first hashitem_T).
void *nvim_vars_ht_get_array(void *ht) { return ((hashtab_T *)ht)->ht_array; }

/// Get ht->ht_used (void* param to avoid conflicting with syntax_accessors.c).
size_t nvim_vars_ht_get_used(void *ht) { return ((hashtab_T *)ht)->ht_used; }

/// Get hi->hi_key (void* param to avoid conflicting with typval.c).
const char *nvim_vars_hashitem_get_key(void *hi) { return ((hashitem_T *)hi)->hi_key; }

/// Get di->di_key (void* param to avoid conflicting with typval.c).
const char *nvim_vars_dictitem_get_key(void *di) { return ((dictitem_T *)di)->di_key; }

/// Get xp->xp_pattern (void* param to avoid conflicting with option_shim.c).
const char *nvim_vars_xp_get_pattern(void *xp) { return ((expand_T *)xp)->xp_pattern; }

/// got_int accessor (bool return to avoid conflicting with typval.c c_int).
bool nvim_vars_got_int(void) { return got_int; }

/// Advance a hashitem pointer by one (pointer arithmetic).
void *nvim_hashitem_advance(void *hi) { return (hashitem_T *)hi + 1; }

/// Get &globvarht.
void *nvim_get_globvarht_ptr(void) { return &globvarht; }

/// Get &vimvarht.
void *nvim_get_vimvarht_ptr(void) { return &vimvarht; }

/// Get prevwin_curwin() buf vars dv_hashtab pointer (for get_user_var_name).
void *nvim_prevwin_curwin_buf_vars_ht(void) { return &prevwin_curwin()->w_buffer->b_vars->dv_hashtab; }

/// Get prevwin_curwin() win vars dv_hashtab pointer (for get_user_var_name).
void *nvim_prevwin_curwin_win_vars_ht(void) { return &prevwin_curwin()->w_vars->dv_hashtab; }

/// Get curtab tp_vars dv_hashtab pointer.
void *nvim_curtab_tp_vars_ht(void) { return &curtab->tp_vars->dv_hashtab; }

/// Get vimvars array size.
int nvim_vimvars_array_size(void) { return (int)ARRAY_SIZE(vimvars); }

/// Get varnamebuf pointer address.
char **nvim_get_varnamebuf_ptr(void) { return &varnamebuf; }

/// Get varnamebuflen pointer address.
size_t *nvim_get_varnamebuflen_ptr(void) { return &varnamebuflen; }

/// IOSIZE constant.
int nvim_get_iosize(void) { return IOSIZE; }

/// xstrlcpy wrapper with IOSIZE limit.
void nvim_xstrlcpy_iosize(char *dst, const char *src) { xstrlcpy(dst, src, IOSIZE); }

/// xstrlcat wrapper with IOSIZE limit.
void nvim_xstrlcat_iosize(char *dst, const char *src) { xstrlcat(dst, src, IOSIZE); }

/// message_filtered wrapper.
bool nvim_message_filtered(const char *buf) { return message_filtered(buf); }

/// get_vim_var_name wrapper.
const char *nvim_get_vim_var_name(int idx) { return get_vim_var_name((VimVarIndex)idx); }

/// redir_ga ga_grow wrapper.
void nvim_redir_ga_grow(int len) { ga_grow(&redir_ga, len); }

/// Append bytes to redir_ga.
void nvim_redir_ga_append(const char *value, int len) {
  memmove((char *)redir_ga.ga_data + redir_ga.ga_len, value, (size_t)len);
  redir_ga.ga_len += len;
}

/// Get redir_lval pointer.
void *nvim_get_redir_lval(void) { return redir_lval; }

/// Get redir_varname pointer.
char *nvim_get_redir_varname(void) { return redir_varname; }

/// Non-inline wrapper for STRCPY (for Rust FFI).
void nvim_strcpy(char *dst, const char *src) { STRCPY(dst, src); }

// Phase 4: heredoc_get accessor shims for Rust FFI

/// Check if eap->ea_getline is non-NULL.
int nvim_eap_has_getline(const void *eap) { return ((const exarg_T *)eap)->ea_getline != NULL; }

/// Call eap->ea_getline(c, cookie, indent, false) and return the result.
char *nvim_eap_call_getline(void *eap_void, int c, int indent)
{
  exarg_T *eap = (exarg_T *)eap_void;
  return eap->ea_getline((char)c, eap->cookie, indent, false);
}

/// Get *eap->cmdlinep (the command line string for trim indent detection).
const char *nvim_eap_get_cmdlinep_str(const void *eap) { return *((const exarg_T *)eap)->cmdlinep; }

// Phase 4b: do_unlet and ex_lockvar shims for Rust FFI

/// Wrapper for ex_unletlock with do_unlet_var callback (for :unlet).
void nvim_ex_unletlock_unlet(void *eap, char *arg, int deep, int glv_flags)
{ ex_unletlock((exarg_T *)eap, arg, deep, glv_flags, do_unlet_var); }

/// Wrapper for ex_unletlock with do_lock_var callback (for :lockvar/:unlockvar).
void nvim_ex_unletlock_lock(void *eap, char *arg, int deep)
{ ex_unletlock((exarg_T *)eap, arg, deep, 0, do_lock_var); }

/// find_var_ht_dict wrapper for Rust FFI (static in C).
/// Returns ht or NULL; sets *varname and *dict_out.
void *nvim_vars_find_var_ht_dict(const char *name, size_t name_len, const char **varname, void **dict_out)
{ return find_var_ht_dict(name, name_len, varname, (dict_T **)dict_out); }

/// delete_var wrapper for Rust FFI (static in C).
void nvim_vars_delete_var(void *ht, void *hi)
{ delete_var((hashtab_T *)ht, (hashitem_T *)hi); }

/// Get the dict from a dictitem's inner typval (di->di_tv.vval.v_dict).
void *nvim_vars_dictitem_inner_dict(void *di)
{ return ((dictitem_T *)di)->di_tv.vval.v_dict; }

/// Get di->di_flags as int.
int nvim_vars_dictitem_get_flags(void *di)
{ return (int)((dictitem_T *)di)->di_flags; }

/// Get the tv of a dictitem (di->di_tv pointer).
void *nvim_vars_dictitem_get_tv_ptr(void *di)
{ return &((dictitem_T *)di)->di_tv; }

/// hash_find wrapper (for do_unlet in Rust).
void *nvim_vars_hash_find(void *ht, const char *key)
{ return hash_find((hashtab_T *)ht, key); }

/// Get a script line to execute, from a heredoc (<<) or regular string.
/// Used by python, tcl, etc. when the argument starts with "<<".
/// @param eap  ex argument
/// @param lenp  set to length of returned string
/// @return  allocated string, or NULL on error/skip
char *script_get(exarg_T *const eap, size_t *const lenp)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_MALLOC
{
  char *cmd = eap->arg;

  if (cmd[0] != '<' || cmd[1] != '<' || eap->ea_getline == NULL) {
    *lenp = strlen(eap->arg);
    return eap->skip ? NULL : xmemdupz(eap->arg, *lenp);
  }
  cmd += 2;

  garray_T ga = { .ga_data = NULL, .ga_len = 0 };

  list_T *const l = heredoc_get(eap, cmd, true);
  if (l == NULL) {
    return NULL;
  }

  if (!eap->skip) {
    ga_init(&ga, 1, 0x400);
  }

  TV_LIST_ITER_CONST(l, li, {
    if (!eap->skip) {
      ga_concat(&ga, tv_get_string(TV_LIST_ITEM_TV(li)));
      ga_append(&ga, '\n');
    }
  });
  *lenp = (size_t)ga.ga_len;  // Set length without trailing NUL.
  if (!eap->skip) {
    ga_append(&ga, NUL);
  }

  tv_list_free(l);
  return (char *)ga.ga_data;
}
