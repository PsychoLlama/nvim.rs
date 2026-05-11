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

#include "eval/vars.c.generated.h"

// Rust function declarations
extern bool rs_set_ref_in_ht(hashtab_T *ht, int copyID, list_stack_T **list_stack);
extern const char *rs_skip_var_list(const char *arg, int *var_count, int *semicolon, bool silent);
extern char *rs_eval_one_expr_in_str(char *p, garray_T *gap, bool evaluate);
extern int rs_get_spellword(list_T *list, const char **ret_word);
extern bool rs_garbage_collect_scriptvars(int copy_id);
extern void rs_set_internal_string_var(const char *name, char *value);
extern char *rs_ex_let_one(char *arg, typval_T *tv, bool copy, bool is_const,
                            const char *endchars, const char *op);
extern int rs_ex_let_vars(char *arg_start, typval_T *tv, int copy, int semicolon,
                           int var_count, int is_const, char *op);
extern const char *rs_get_var_value(const char *name);
extern int rs_eval_variable(const char *name, int len, void *rettv, void **dip, bool verbose,
                             bool no_autoload);
extern void rs_check_vars(const char *name, size_t len);
extern const char *rs_list_arg_vars(exarg_T *eap, const char *arg, int *first);
extern void rs_del_menutrans_vars(void);
extern list_T *rs_eval_spell_expr(const char *badword, char *expr);
extern int rs_eval_charconvert(const char *enc_from, const char *enc_to,
                                const char *fname_from, const char *fname_to);
extern void rs_eval_diff(const char *origfile, const char *newfile, const char *outfile);
extern void rs_eval_patch(const char *origfile, const char *difffile, const char *outfile);
extern bool rs_var_check_ro(int flags, const char *name, size_t name_len);
extern bool rs_var_check_lock(int flags, const char *name, size_t name_len);
extern bool rs_var_check_fixed(int flags, const char *name, size_t name_len);
extern bool rs_var_wrong_func_name(const char *name, bool new_var);
extern bool rs_valid_varname(const char *varname);

// v: variable accessor functions in Rust
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

// Phase 1/7: option conversion, set_cmdarg, and ex_let_option migrated to Rust
extern OptVal rs_tv_to_optval(typval_T *tv, OptIndex opt_idx, const char *option, bool *error);
extern void rs_optval_as_tv(OptVal value, bool numbool, typval_T *rettv);
extern void rs_set_option_from_tv(const char *varname, typval_T *varp);
extern char *rs_set_cmdarg(exarg_T *eap, char *oldarg);
extern char *rs_ex_let_option(char *arg, typval_T *tv, bool is_const, const char *endchars,
                              const char *op);

// Phase 2: VimL f_ functions migrated to Rust
// (rs_f_* functions are declared below)

// Phase 4/5: heredoc, script_get and unlet functions migrated to Rust
extern list_T *rs_heredoc_get(exarg_T *eap, char *cmd, int script_get);
extern char *rs_script_get(exarg_T *eap, size_t *lenp);
extern void rs_ex_unlet(exarg_T *eap);
extern void rs_ex_lockvar(exarg_T *eap);
extern int rs_do_unlet(const char *name, size_t name_len, int forceit);

// Phase 6: var_exists and var_redir functions migrated to Rust
extern bool rs_var_exists(const char *var);
extern int rs_var_redir_start(char *name, bool append);
extern void rs_var_redir_stop(void);

// Phase 3: listing and redirection functions migrated to Rust
extern char *rs_cat_prefix_varname(int prefix, const char *name);
extern char *rs_get_user_var_name(expand_T *xp, int idx);
extern void rs_var_redir_str(const char *value, int value_len);
extern void rs_list_hashtable_vars(hashtab_T *ht, const char *prefix, int empty, int *first);
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
static const char e_setting_v_str_to_value_with_wrong_type[]
  = N_("E963: Setting v:%s to value with wrong type");

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

bool garbage_collect_scriptvars(int copyID) { return rs_garbage_collect_scriptvars(copyID); }

/// Set an internal variable to a string value. Creates the variable if it does
/// not already exist.
void set_internal_string_var(const char *name, char *value)  // NOLINT(readability-non-const-parameter)
  FUNC_ATTR_NONNULL_ARG(1)
{ rs_set_internal_string_var(name, value); }

int eval_charconvert(const char *const enc_from, const char *const enc_to,
                     const char *const fname_from, const char *const fname_to)
{ return rs_eval_charconvert(enc_from, enc_to, fname_from, fname_to); }

void eval_diff(const char *const origfile, const char *const newfile, const char *const outfile)
{ rs_eval_diff(origfile, newfile, outfile); }

void eval_patch(const char *const origfile, const char *const difffile, const char *const outfile)
{ rs_eval_patch(origfile, difffile, outfile); }

/// Evaluate an expression to a list with suggestions.
/// For the "expr:" part of 'spellsuggest'.
///
/// @return  NULL when there is an error.
list_T *eval_spell_expr(char *badword, char *expr)
{ return (list_T *)rs_eval_spell_expr(badword, expr); }

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

char *eval_one_expr_in_str(char *p, garray_T *gap, bool evaluate) { return rs_eval_one_expr_in_str(p, gap, evaluate); }

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
      arg = (char *)rs_list_arg_vars(eap, arg, &first);
    } else if (!eap->skip) {
      // ":let"
      rs_list_hashtable_vars(&globvarht, "", true, &first);
      rs_list_hashtable_vars(&curbuf->b_vars->dv_hashtab, "b:", true, &first);
      rs_list_hashtable_vars(&curwin->w_vars->dv_hashtab, "w:", true, &first);
      rs_list_hashtable_vars(&curtab->tp_vars->dv_hashtab, "t:", true, &first);
      nvim_list_script_vars(&first);
      list_func_vars(&first);
      nvim_list_vim_vars(&first);
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
{ return rs_ex_let_vars(arg_start, tv, copy, semicolon, var_count, is_const, op); }

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


/// ":unlet[!] var1 ... " command.
void ex_unlet(exarg_T *eap) { rs_ex_unlet(eap); }

/// ":lockvar" and ":unlockvar" commands
void ex_lockvar(exarg_T *eap) { rs_ex_lockvar(eap); }

/// unlet a variable
///
/// @param[in]  name  Variable name to unlet.
/// @param[in]  name_len  Variable name length.
/// @param[in]  forceit  If true, do not complain if variable doesn’t exist.
///
/// @return OK if it existed, FAIL otherwise.
int do_unlet(const char *const name, const size_t name_len, const bool forceit)
{ return rs_do_unlet(name, name_len, forceit); }

/// Delete all "menutrans_" variables.
void del_menutrans_vars(void) { rs_del_menutrans_vars(); }

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

// Buffer for cat_prefix_varname(), freed in get_user_var_name().
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
{ return rs_eval_variable(name, len, rettv, (void **)dip, verbose, no_autoload); }

/// Check if variable "name[len]" is a local variable or an argument.
/// If so, "*eval_lavars_used" is set to true.
void check_vars(const char *name, size_t len)
{ rs_check_vars(name, len); }

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
{ return (char *)rs_get_var_value(name); }

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

void reset_v_option_vars(void) { rs_reset_v_option_vars(); }
void assert_error(garray_T *gap) { rs_assert_error(gap->ga_data, gap->ga_len); }

bool var_exists(const char *var)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_var_exists(var);
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
  return rs_var_redir_start(name, append);
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
  rs_var_redir_stop();
}

/// "gettabvar()" function
void f_gettabvar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{ rs_f_gettabvar(argvars, rettv); }

/// "gettabwinvar()" function
void f_gettabwinvar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { rs_f_gettabwinvar(argvars, rettv); }

/// "getwinvar()" function
void f_getwinvar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { rs_f_getwinvar(argvars, rettv); }

/// "getbufvar()" function
void f_getbufvar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{ rs_f_getbufvar(argvars, rettv); }

/// "settabvar()" function
void f_settabvar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{ rs_f_settabvar(argvars); }

/// "settabwinvar()" function
void f_settabwinvar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { rs_f_settabwinvar(argvars); }

/// "setwinvar()" function
void f_setwinvar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { rs_f_setwinvar(argvars); }

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

/// Get script_items.ga_len (total number of script items allocated).
int nvim_script_items_len(void) { return script_items.ga_len; }

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
  return rs_script_get(eap, lenp);
}

// Phase 6: var_exists, var_redir_start, var_redir_stop accessors for Rust FFI.

/// Get ll_name from lval_T pointer.
const char *nvim_lval_get_name(void *lv) { return ((lval_T *)lv)->ll_name; }

/// Wrapper for get_lval() with no rettv, no unlet, no skip, no extra flags.
char *nvim_vars_get_lval(char *name, void *lv)
{ return get_lval(name, NULL, (lval_T *)lv, false, false, 0, FNE_CHECK_START); }

/// Wrapper for set_var_lval() for redir use.
void nvim_vars_set_var_lval(void *lv, char *endp, void *tv, bool copy, bool is_const,
                             const char *op)
{ set_var_lval((lval_T *)lv, endp, (typval_T *)tv, copy, is_const, op); }

/// Wrapper for clear_lval().
void nvim_vars_clear_lval(void *lv) { clear_lval((lval_T *)lv); }

/// Allocate a zeroed lval_T on the heap; returns void* for Rust.
void *nvim_vars_alloc_lval(void) { return xcalloc(1, sizeof(lval_T)); }

/// Initialize the redir growing-array.
void nvim_vars_redir_ga_init(void) { ga_init(&redir_ga, (int)sizeof(char), 500); }

/// Append a single NUL byte to redir_ga.
void nvim_vars_redir_ga_append_nul(void) { ga_append(&redir_ga, NUL); }

/// Get redir_ga.ga_data pointer.
void *nvim_vars_redir_ga_data(void) { return redir_ga.ga_data; }

/// Get redir_ga.ga_len.
int nvim_vars_redir_ga_len(void) { return redir_ga.ga_len; }

/// Free redir_ga.ga_data and set to NULL.
void nvim_vars_redir_ga_data_clear(void) { XFREE_CLEAR(redir_ga.ga_data); }

/// Free redir_lval and set to NULL.
void nvim_vars_redir_lval_free(void) { XFREE_CLEAR(redir_lval); }

/// Set redir_lval.
void nvim_vars_set_redir_lval(void *lv) { redir_lval = (lval_T *)lv; }

/// Set redir_varname.
void nvim_vars_set_redir_varname(char *n) { redir_varname = n; }

/// Set redir_endp.
void nvim_vars_set_redir_endp(char *e) { redir_endp = e; }

/// Get redir_endp.
char *nvim_vars_get_redir_endp(void) { return redir_endp; }

/// Get called_emsg.
int nvim_vars_get_called_emsg(void) { return called_emsg; }

/// Get did_emsg.
int nvim_vars_get_did_emsg(void) { return (int)did_emsg; }

/// Set did_emsg.
void nvim_vars_set_did_emsg(int v) { did_emsg = (bool)v; }

/// eval_variable wrapper for Rust (passes NULL for dip, verbose=false, no_autoload=true).
int nvim_vars_eval_variable(const char *name, int len, void *tv)
{ return eval_variable(name, len, (typval_T *)tv, NULL, false, true); }

/// handle_subscript wrapper for Rust (EVALARG_EVALUATE, verbose=false).
int nvim_vars_handle_subscript_check(const char **arg, void *tv)
{ return handle_subscript(arg, (typval_T *)tv, &EVALARG_EVALUATE, false); }

// Phase 8b: lval_T and misc accessors for ex_unletlock/do_unlet_var/do_lock_var migration.

/// get_lval() with unlet=true for use in :unlet/:lockvar parsing.
char *nvim_vars_get_lval_unlet(char *name, void *lv, bool skip, int glv_flags)
{ return get_lval(name, NULL, (lval_T *)lv, true, skip, glv_flags, FNE_CHECK_START); }

/// Return true if ll_tv is NULL (lvalue is a simple variable name).
bool nvim_lval_is_tv_null(const void *lv) { return ((const lval_T *)lv)->ll_tv == NULL; }

/// Set ll_name and ll_tv=NULL on lval_T (for $ENV var case in ex_unletlock).
void nvim_lval_set_name_and_clear_tv(void *lv, char *name)
{ lval_T *p = (lval_T *)lv; p->ll_name = name; p->ll_tv = NULL; }

/// ends_excmd() check for a char value.
bool nvim_ends_excmd_char(int c) { return ends_excmd(c); }

/// check_nextcmd() wrapper.
char *nvim_check_nextcmd(char *arg) { return check_nextcmd(arg); }

/// Set emsg_severe to true (used for trailing chars error in ex_unletlock).
void nvim_emsg_severe_set(void) { emsg_severe = true; }

/// Get eap->cmdidx as int.
int nvim_eap_get_cmdidx(const void *eap) { return (int)((const exarg_T *)eap)->cmdidx; }

/// CMD_lockvar constant.
int nvim_cmd_lockvar(void) { return CMD_lockvar; }

/// Get ll_name_len from lval_T.
size_t nvim_lval_get_name_len(const void *lv) { return ((const lval_T *)lv)->ll_name_len; }

/// Get ll_list from lval_T (as void*).
void *nvim_lval_get_list(const void *lv) { return ((const lval_T *)lv)->ll_list; }

/// Get ll_dict from lval_T (as void*).
void *nvim_lval_get_dict(const void *lv) { return ((const lval_T *)lv)->ll_dict; }

/// Get ll_di from lval_T (as void*).
void *nvim_lval_get_di(const void *lv) { return ((const lval_T *)lv)->ll_di; }

/// Get ll_li from lval_T (as void*).
void *nvim_lval_get_li(const void *lv) { return ((const lval_T *)lv)->ll_li; }

/// Get ll_range from lval_T.
bool nvim_lval_get_range(const void *lv) { return ((const lval_T *)lv)->ll_range; }

/// Get ll_empty2 from lval_T.
bool nvim_lval_get_empty2(const void *lv) { return ((const lval_T *)lv)->ll_empty2; }

/// Get ll_n1 from lval_T.
int nvim_lval_get_n1(const void *lv) { return ((const lval_T *)lv)->ll_n1; }

/// Get ll_n2 from lval_T.
int nvim_lval_get_n2(const void *lv) { return ((const lval_T *)lv)->ll_n2; }

/// Increment ll_n1 in lval_T.
void nvim_lval_inc_n1(void *lv) { ((lval_T *)lv)->ll_n1++; }

/// tv_list_locked() wrapper.
int nvim_tv_list_locked(const void *l) { return (int)tv_list_locked((const list_T *)l); }

/// tv_list_item_remove() wrapper.
void nvim_tv_list_item_remove(void *l, void *li)
{ tv_list_item_remove((list_T *)l, (listitem_T *)li); }

/// tv_list_remove_items() wrapper.
void nvim_tv_list_remove_items(void *l, void *li_first, void *li_last)
{ tv_list_remove_items((list_T *)l, (listitem_T *)li_first, (listitem_T *)li_last); }

/// vim_unsetenv_ext() wrapper (strips leading '$' from name).
void nvim_vim_unsetenv_ext(const char *name_with_dollar)
{ vim_unsetenv_ext(name_with_dollar + 1); }

/// find_var() wrapper returning void* (NULL for not found).
void *nvim_find_var(const char *name, size_t name_len, bool no_autoload)
{ return find_var(name, name_len, NULL, no_autoload); }

/// tv_item_lock() wrapper.
void nvim_tv_item_lock(void *tv, int deep, bool lock, bool check_refcount)
{ tv_item_lock((typval_T *)tv, deep, lock, check_refcount); }

/// Get di->di_tv.v_type as int.
int nvim_dictitem_get_tv_type(const void *di)
{ return (int)((const dictitem_T *)di)->di_tv.v_type; }

/// Set or clear DI_FLAGS_LOCK on di->di_flags.
void nvim_dictitem_set_lock_bit(void *di, bool lock)
{
  dictitem_T *d = (dictitem_T *)di;
  if (lock) {
    d->di_flags |= DI_FLAGS_LOCK;
  } else {
    d->di_flags &= (uint8_t)(~DI_FLAGS_LOCK);
  }
}

/// DI_FLAGS_FIX constant.
int nvim_di_flags_fix(void) { return DI_FLAGS_FIX; }

/// VAR_DICT constant.
int nvim_var_dict(void) { return VAR_DICT; }

/// VAR_LIST constant.
int nvim_var_list(void) { return VAR_LIST; }

// Phase 12: eval_variable, check_vars, list_arg_vars, eval_spell_expr, del_menutrans_vars shims.

/// Get eval_lavars_used global pointer (may be NULL).
bool *nvim_vars_get_eval_lavars_used(void) { return eval_lavars_used; }

/// Set eval_lavars_used global pointer.
void nvim_vars_set_eval_lavars_used(bool *ptr) { eval_lavars_used = ptr; }

/// Get get_funccal_local_ht() as void*.
void *nvim_vars_get_funccal_local_ht(void) { return get_funccal_local_ht(); }

/// Get get_funccal_args_ht() as void*.
void *nvim_vars_get_funccal_args_ht(void) { return get_funccal_args_ht(); }

/// find_var wrapper returning void*; sets *htp if non-NULL.
void *nvim_vars_find_var(const char *name, size_t name_len, void **htp, int no_autoload)
{ return find_var(name, name_len, (hashtab_T **)htp, no_autoload); }

/// eval_variable wrapper returning full result (verbose, no_autoload params).
int nvim_vars_eval_variable_full(const char *name, int len, void *rettv, void **dip, bool verbose,
                                  bool no_autoload)
{ return rs_eval_variable(name, len, rettv, dip, verbose, no_autoload); }

/// List b: variables.
void nvim_list_buf_vars(int *first)
{ rs_list_hashtable_vars(&curbuf->b_vars->dv_hashtab, "b:", true, first); }

/// List w: variables.
void nvim_list_win_vars(int *first)
{ rs_list_hashtable_vars(&curwin->w_vars->dv_hashtab, "w:", true, first); }

/// List t: variables.
void nvim_list_tab_vars(int *first)
{ rs_list_hashtable_vars(&curtab->tp_vars->dv_hashtab, "t:", true, first); }

/// List v: variables.
void nvim_list_vim_vars(int *first) { rs_list_hashtable_vars(&vimvarht, "v:", false, first); }

/// List s: variables for current script.
void nvim_list_script_vars(int *first)
{
  if (current_sctx.sc_sid > 0 && current_sctx.sc_sid <= script_items.ga_len) {
    rs_list_hashtable_vars(&SCRIPT_VARS(current_sctx.sc_sid), "s:", false, first);
  }
}

/// list_func_vars wrapper.
void nvim_list_func_vars(int *first) { list_func_vars(first); }

/// aborting() wrapper.
bool nvim_aborting(void) { return aborting(); }

/// get_name_len wrapper for list_arg_vars.
int nvim_get_name_len(const char **arg, char **tofree, bool evaluate, bool verbose)
{ return get_name_len(arg, tofree, evaluate, verbose); }

/// handle_subscript wrapper for list_arg_vars (EVALARG_EVALUATE, verbose=true).
int nvim_vars_handle_subscript_listarg(const char **arg, void *tv)
{ return handle_subscript(arg, (typval_T *)tv, &EVALARG_EVALUATE, true); }

/// eap->skip getter.
int nvim_eap_get_skip_val(const void *eap) { return eap ? ((const exarg_T *)eap)->skip : 0; }

/// E738 error message string.
const char *nvim_e738_cant_list(void) { return _("E738: Can't list variables for %s"); }

/// E475 invalid argument error string.
const char *nvim_e_invarg2(void) { return _(e_invarg2); }

/// E488 trailing chars error string.
const char *nvim_e_trailing_arg(void) { return _(e_trailing_arg); }

/// prepare_vimvar wrapper.
void nvim_prepare_vimvar(int idx, void *save_tv) { prepare_vimvar(idx, (typval_T *)save_tv); }

/// restore_vimvar wrapper.
void nvim_restore_vimvar(int idx, void *save_tv) { restore_vimvar(idx, (typval_T *)save_tv); }

/// kOptSpellsuggest constant.
int nvim_kopt_spellsuggest(void) { return (int)kOptSpellsuggest; }

/// may_call_simple_func wrapper.
int nvim_may_call_simple_func(const char *p, void *rettv)
{ return may_call_simple_func(p, (typval_T *)rettv); }

/// eval1 wrapper with EVALARG_EVALUATE.
int nvim_eval1_evaluate(char **arg, void *rettv)
{ return eval1(arg, (typval_T *)rettv, &EVALARG_EVALUATE); }

/// Evaluate the p_sps spellsuggest option context.
void nvim_apply_spellsuggest_sctx(void)
{
  sctx_T *ctx = get_option_sctx(kOptSpellsuggest);
  if (ctx != NULL) {
    current_sctx = *ctx;
  }
}

// Phase 11: eval_charconvert / eval_diff / eval_patch accessor shims.

/// Save current_sctx to a heap-allocated sctx_T and return pointer.
/// Caller must pass returned pointer to nvim_restore_current_sctx() to free it.
sctx_T *nvim_save_current_sctx(void)
{
  sctx_T *s = xmalloc(sizeof(sctx_T));
  *s = current_sctx;
  return s;
}

/// Apply option sctx to current_sctx if non-NULL.
void nvim_apply_option_sctx(int opt)
{
  sctx_T *ctx = get_option_sctx((OptIndex)opt);
  if (ctx != NULL) {
    current_sctx = *ctx;
  }
}

/// kOptCharconvert constant.
int nvim_kopt_charconvert(void) { return (int)kOptCharconvert; }

/// kOptDiffexpr constant.
int nvim_kopt_diffexpr(void) { return (int)kOptDiffexpr; }

/// kOptPatchexpr constant.
int nvim_kopt_patchexpr(void) { return (int)kOptPatchexpr; }

/// Evaluate p_ccv for charconvert. Returns true if error occurred.
bool nvim_eval_charconvert_expr(void)
{
  bool err = false;
  if (eval_to_bool(p_ccv, &err, NULL, false, true)) {
    err = true;
  }
  return err;
}

/// Evaluate p_dex (diffexpr). Errors are ignored.
void nvim_eval_diffexpr(void)
{
  typval_T *tv = eval_expr_ext(p_dex, NULL, true);
  tv_free(tv);
}

/// Evaluate p_pex (patchexpr). Errors are ignored.
void nvim_eval_patchexpr(void)
{
  typval_T *tv = eval_expr_ext(p_pex, NULL, true);
  tv_free(tv);
}

// Phase 10: ex_let_one / ex_let_vars accessor shims.

/// get_lval() with tv argument (for ex_let_one name case).
char *nvim_vars_get_lval_with_tv(char *name, void *tv, void *lv)
{ return get_lval(name, (typval_T *)tv, (lval_T *)lv, false, false, 0, FNE_CHECK_START); }

/// Emit E18 "Unexpected characters in :let" error.
void nvim_vars_emsg_letunexp(void) { emsg(_(e_letunexp)); }

/// Build a VAR_LIST typval from the remaining list items starting at "item".
/// Appends all items to a new list, sets ltv fields, refs the list.
void nvim_vars_build_rest_list(void *item, void *ltv, size_t rest_len)
{
  list_T *const rest_list = tv_list_alloc((ptrdiff_t)rest_len);
  listitem_T *li = (listitem_T *)item;
  while (li != NULL) {
    tv_list_append_tv(rest_list, TV_LIST_ITEM_TV(li));
    li = TV_LIST_ITEM_NEXT(NULL, li);
  }
  typval_T *tv = (typval_T *)ltv;
  tv->v_type = VAR_LIST;
  tv->v_lock = VAR_UNLOCKED;
  tv->vval.v_list = rest_list;
  tv_list_ref(rest_list);
}

/// Accessor for eval_msgpack_type_lists[idx] — used by Rust decode module.
list_T *nvim_eval_msgpack_type_list(int idx)
{
  return (list_T *)eval_msgpack_type_lists[idx];
}
