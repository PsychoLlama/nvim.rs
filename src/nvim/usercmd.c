// usercmd.c: User defined command support

#include <assert.h>
#include <inttypes.h>
#include <lauxlib.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

#include "auto/config.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/eval.h"
#include "nvim/ex_docmd.h"
#include "nvim/garray.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/keycodes.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/mapping.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/menu.h"
#include "nvim/message.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/runtime.h"
#include "nvim/strings.h"
#include "nvim/usercmd.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

#include "usercmd.c.generated.h"

garray_T ucmds = { 0, 0, sizeof(ucmd_T), 4, NULL };

// ============================================================================
// Static assertions: verify Rust-mirrored constants match C definitions.
// ============================================================================

// cmd_addr_T (ex_cmds_defs.h)
_Static_assert(ADDR_LINES == 0, "ADDR_LINES");
_Static_assert(ADDR_WINDOWS == 1, "ADDR_WINDOWS");
_Static_assert(ADDR_ARGUMENTS == 2, "ADDR_ARGUMENTS");
_Static_assert(ADDR_LOADED_BUFFERS == 3, "ADDR_LOADED_BUFFERS");
_Static_assert(ADDR_BUFFERS == 4, "ADDR_BUFFERS");
_Static_assert(ADDR_TABS == 5, "ADDR_TABS");
_Static_assert(ADDR_TABS_RELATIVE == 6, "ADDR_TABS_RELATIVE");
_Static_assert(ADDR_QUICKFIX_VALID == 7, "ADDR_QUICKFIX_VALID");
_Static_assert(ADDR_QUICKFIX == 8, "ADDR_QUICKFIX");
_Static_assert(ADDR_UNSIGNED == 9, "ADDR_UNSIGNED");
_Static_assert(ADDR_OTHER == 10, "ADDR_OTHER");
_Static_assert(ADDR_NONE == 11, "ADDR_NONE");

// EX_* flags (ex_cmds_defs.h)
_Static_assert(EX_RANGE == 0x001, "EX_RANGE");
_Static_assert(EX_BANG == 0x002, "EX_BANG");
_Static_assert(EX_EXTRA == 0x004, "EX_EXTRA");
_Static_assert(EX_XFILE == 0x008, "EX_XFILE");
_Static_assert(EX_NOSPC == 0x010, "EX_NOSPC");
_Static_assert(EX_DFLALL == 0x020, "EX_DFLALL");
_Static_assert(EX_NEEDARG == 0x080, "EX_NEEDARG");
_Static_assert(EX_TRLBAR == 0x100, "EX_TRLBAR");
_Static_assert(EX_REGSTR == 0x200, "EX_REGSTR");
_Static_assert(EX_COUNT == 0x400, "EX_COUNT");
_Static_assert(EX_ZEROR == 0x1000, "EX_ZEROR");
_Static_assert(EX_BUFNAME == 0x8000, "EX_BUFNAME");
_Static_assert(EX_KEEPSCRIPT == 0x4000000, "EX_KEEPSCRIPT");

// CMOD_* flags (ex_cmds_defs.h)
_Static_assert(CMOD_SANDBOX == 0x0001, "CMOD_SANDBOX");
_Static_assert(CMOD_SILENT == 0x0002, "CMOD_SILENT");
_Static_assert(CMOD_ERRSILENT == 0x0004, "CMOD_ERRSILENT");
_Static_assert(CMOD_UNSILENT == 0x0008, "CMOD_UNSILENT");
_Static_assert(CMOD_NOAUTOCMD == 0x0010, "CMOD_NOAUTOCMD");
_Static_assert(CMOD_HIDE == 0x0020, "CMOD_HIDE");
_Static_assert(CMOD_BROWSE == 0x0040, "CMOD_BROWSE");
_Static_assert(CMOD_CONFIRM == 0x0080, "CMOD_CONFIRM");
_Static_assert(CMOD_KEEPALT == 0x0100, "CMOD_KEEPALT");
_Static_assert(CMOD_KEEPMARKS == 0x0200, "CMOD_KEEPMARKS");
_Static_assert(CMOD_KEEPJUMPS == 0x0400, "CMOD_KEEPJUMPS");
_Static_assert(CMOD_LOCKMARKS == 0x0800, "CMOD_LOCKMARKS");
_Static_assert(CMOD_KEEPPATTERNS == 0x1000, "CMOD_KEEPPATTERNS");
_Static_assert(CMOD_NOSWAPFILE == 0x2000, "CMOD_NOSWAPFILE");

// WSP_* flags (window.h)
_Static_assert(WSP_ROOM == 0x01, "WSP_ROOM");
_Static_assert(WSP_VERT == 0x02, "WSP_VERT");
_Static_assert(WSP_HOR == 0x04, "WSP_HOR");
_Static_assert(WSP_TOP == 0x08, "WSP_TOP");
_Static_assert(WSP_BOT == 0x10, "WSP_BOT");
_Static_assert(WSP_BELOW == 0x40, "WSP_BELOW");
_Static_assert(WSP_ABOVE == 0x80, "WSP_ABOVE");

// K_SPECIAL / KS_SPECIAL / KE_FILLER (keycodes.h)
_Static_assert(K_SPECIAL == 0x80, "K_SPECIAL");
_Static_assert(KS_SPECIAL == 254, "KS_SPECIAL");
_Static_assert(KE_FILLER == 'X', "KE_FILLER");

// UC_BUFFER (usercmd.h)
_Static_assert(UC_BUFFER == 1, "UC_BUFFER");

// CMD_USER / CMD_USER_BUF (ex_cmds_enum.generated.h)
_Static_assert(CMD_USER == -1, "CMD_USER");
_Static_assert(CMD_USER_BUF == -2, "CMD_USER_BUF");

// DOCMD_* flags (ex_docmd.h)
_Static_assert(DOCMD_VERBOSE == 0x01, "DOCMD_VERBOSE");
_Static_assert(DOCMD_NOWAIT == 0x02, "DOCMD_NOWAIT");
_Static_assert(DOCMD_KEYTYPED == 0x08, "DOCMD_KEYTYPED");

// EXPAND_* (cmdexpand_defs.h) — spot-check key values
_Static_assert(EXPAND_UNSUCCESSFUL == -2, "EXPAND_UNSUCCESSFUL");
_Static_assert(EXPAND_NOTHING == 0, "EXPAND_NOTHING");
_Static_assert(EXPAND_COMMANDS == 1, "EXPAND_COMMANDS");
_Static_assert(EXPAND_FILES == 2, "EXPAND_FILES");
_Static_assert(EXPAND_DIRECTORIES == 3, "EXPAND_DIRECTORIES");
_Static_assert(EXPAND_BUFFERS == 9, "EXPAND_BUFFERS");
_Static_assert(EXPAND_MENUS == 11, "EXPAND_MENUS");
_Static_assert(EXPAND_MAPPINGS == 16, "EXPAND_MAPPINGS");
_Static_assert(EXPAND_USER_COMMANDS == 22, "EXPAND_USER_COMMANDS");
_Static_assert(EXPAND_USER_CMD_FLAGS == 23, "EXPAND_USER_CMD_FLAGS");
_Static_assert(EXPAND_USER_NARGS == 24, "EXPAND_USER_NARGS");
_Static_assert(EXPAND_USER_COMPLETE == 25, "EXPAND_USER_COMPLETE");
_Static_assert(EXPAND_USER_DEFINED == 30, "EXPAND_USER_DEFINED");
_Static_assert(EXPAND_USER_LIST == 31, "EXPAND_USER_LIST");
_Static_assert(EXPAND_USER_LUA == 32, "EXPAND_USER_LUA");
_Static_assert(EXPAND_SHELLCMD == 33, "EXPAND_SHELLCMD");
_Static_assert(EXPAND_USER_ADDR_TYPE == 43, "EXPAND_USER_ADDR_TYPE");
_Static_assert(EXPAND_SHELLCMDLINE == 57, "EXPAND_SHELLCMDLINE");
_Static_assert(EXPAND_LUA == 63, "EXPAND_LUA");

// Rust FFI declarations (Phase 1: static data tables & trivial lookups)
extern const char *rs_get_command_complete(int arg);
extern const char *rs_get_user_cmd_complete(int idx);
extern int rs_cmdcomplete_type_to_str(int expand, const char *compl_arg, char *buf, size_t buflen);
extern int rs_usercmd_str_to_type(const char *complete_str);
extern const char *rs_get_user_cmd_addr_type(int idx);
extern const char *rs_get_user_cmd_flags(int idx);
extern const char *rs_get_user_cmd_nargs(int idx);

// Rust FFI declarations (Phase 2: argument parsing)
extern const char *rs_uc_validate_name(const char *name);
extern int rs_parse_addr_type_arg(const char *value, int vallen, int *addr_type_arg);
extern int rs_parse_compl_arg(const char *value, int vallen, int *complp, uint32_t *argt,
                              char **compl_arg);
extern int rs_uc_scan_attr(char *attr, size_t len, uint32_t *argt, int *def, int *flags,
                           int *complp, char **compl_arg, int *addr_type_arg);

// C accessor functions called by Rust (Phase 2)
void nvim_uc_emsg(const char *msg)
{
  emsg(_(msg));
}

void nvim_uc_semsg_1(const char *fmt, const char *arg)
{
  semsg(_(fmt), arg);
}

int nvim_uc_getdigits_int(char **pp, int strict, int def)
{
  return getdigits_int(pp, strict != 0, def);
}

char *nvim_uc_xstrnsave(const char *s, size_t len)
{
  return xstrnsave(s, len);
}

// Rust FFI declarations (Phase 3: modifier string generation)
extern size_t rs_add_win_cmd_modifiers(char *buf, const void *cmod, int *multi_mods);
extern size_t rs_uc_mods(char *buf, const void *cmod, int quote);

// C accessor functions called by Rust (Phase 3)
int nvim_uc_cmod_get_split(const void *cmod)
{
  return ((const cmdmod_T *)cmod)->cmod_split;
}

int nvim_uc_cmod_get_flags(const void *cmod)
{
  return ((const cmdmod_T *)cmod)->cmod_flags;
}

int nvim_uc_cmod_get_tab(const void *cmod)
{
  return ((const cmdmod_T *)cmod)->cmod_tab;
}

int nvim_uc_cmod_get_verbose(const void *cmod)
{
  return ((const cmdmod_T *)cmod)->cmod_verbose;
}

int nvim_uc_tabpage_index_curtab(void)
{
  return tabpage_index(curtab);
}

// Rust FFI declarations (Phase 4: argument expansion)
extern char *rs_uc_split_args(const char *arg, char **args, const size_t *arglens, size_t argc,
                              size_t *lenp);
extern size_t rs_uc_check_code(char *code, size_t len, char *buf, void *cmd, void *eap,
                               char **split_buf, size_t *split_len);

// C accessor functions called by Rust (Phase 4)
const char *nvim_uc_eap_get_arg(const void *eap)
{
  return ((const exarg_T *)eap)->arg;
}

uint32_t nvim_uc_eap_get_argt(const void *eap)
{
  return ((const exarg_T *)eap)->argt;
}

int nvim_uc_eap_get_forceit(const void *eap)
{
  return ((const exarg_T *)eap)->forceit ? 1 : 0;
}

int nvim_uc_eap_get_line1(const void *eap)
{
  return (int)((const exarg_T *)eap)->line1;
}

int nvim_uc_eap_get_line2(const void *eap)
{
  return (int)((const exarg_T *)eap)->line2;
}

int nvim_uc_eap_get_addr_count(const void *eap)
{
  return ((const exarg_T *)eap)->addr_count;
}

int nvim_uc_eap_get_regname(const void *eap)
{
  return ((const exarg_T *)eap)->regname;
}

char **nvim_uc_eap_get_args(const void *eap)
{
  return ((const exarg_T *)eap)->args;
}

size_t *nvim_uc_eap_get_arglens(const void *eap)
{
  return ((const exarg_T *)eap)->arglens;
}

size_t nvim_uc_eap_get_argc(const void *eap)
{
  return ((const exarg_T *)eap)->argc;
}

int64_t nvim_uc_cmd_get_def(const void *cmd)
{
  return ((const ucmd_T *)cmd)->uc_def;
}

int nvim_uc_utfc_ptr2len(const char *p)
{
  return utfc_ptr2len(p);
}

void nvim_uc_mb_copy_char(const char **pp, char **qq)
{
  mb_copy_char(pp, qq);
}

char *nvim_uc_xmalloc(size_t size)
{
  return xmalloc(size);
}

const void *nvim_uc_get_cmdmod(void)
{
  return &cmdmod;
}

// Rust FFI declarations (Phase 5: command definition management)
extern int rs_uc_add_command(char *name, size_t name_len, const char *rep,
                             uint32_t argt, int64_t def, int flags, int context,
                             char *compl_arg, int compl_luaref, int preview_luaref,
                             int addr_type, int luaref, int force);
extern void rs_free_ucmd(void *cmd);
extern void rs_uc_clear(void *gap);

// C accessor functions called by Rust (Phase 5)
void *nvim_uc_get_curbuf_ucmds(void)
{
  return &curbuf->b_ucmds;
}

void *nvim_uc_get_ucmds(void)
{
  return &ucmds;
}

int nvim_uc_ga_get_len(void *gap)
{
  return ((garray_T *)gap)->ga_len;
}

int nvim_uc_ga_get_itemsize(void *gap)
{
  return ((garray_T *)gap)->ga_itemsize;
}

void nvim_uc_ga_set_len(void *gap, int len)
{
  ((garray_T *)gap)->ga_len = len;
}

void nvim_uc_ga_init_ucmd(void *gap)
{
  ga_init((garray_T *)gap, (int)sizeof(ucmd_T), 4);
}

void nvim_uc_ga_grow(void *gap, int n)
{
  ga_grow((garray_T *)gap, n);
}

void nvim_uc_ga_clear(void *gap)
{
  ga_clear((garray_T *)gap);
}

void *nvim_uc_ga_get_cmd(void *gap, int i)
{
  return USER_CMD_GA((garray_T *)gap, i);
}

void nvim_uc_cmd_memmove_down(void *gap, int i)
{
  garray_T *g = (garray_T *)gap;
  ucmd_T *cmd = USER_CMD_GA(g, i);
  memmove(cmd + 1, cmd, (size_t)(g->ga_len - i) * sizeof(ucmd_T));
}

const char *nvim_uc_cmd_get_name(const void *cmd)
{
  return ((const ucmd_T *)cmd)->uc_name;
}

int nvim_uc_cmd_get_sc_sid(const void *cmd)
{
  return ((const ucmd_T *)cmd)->uc_script_ctx.sc_sid;
}

int nvim_uc_cmd_get_sc_seq(const void *cmd)
{
  return ((const ucmd_T *)cmd)->uc_script_ctx.sc_seq;
}

void nvim_uc_cmd_set_name(void *cmd, char *name)
{
  ((ucmd_T *)cmd)->uc_name = name;
}

void nvim_uc_cmd_set_rep(void *cmd, char *rep)
{
  ((ucmd_T *)cmd)->uc_rep = rep;
}

void nvim_uc_cmd_set_argt(void *cmd, uint32_t argt)
{
  ((ucmd_T *)cmd)->uc_argt = argt;
}

void nvim_uc_cmd_set_def(void *cmd, int64_t def)
{
  ((ucmd_T *)cmd)->uc_def = def;
}

void nvim_uc_cmd_set_compl(void *cmd, int compl_val)
{
  ((ucmd_T *)cmd)->uc_compl = compl_val;
}

void nvim_uc_cmd_set_compl_arg(void *cmd, char *arg)
{
  ((ucmd_T *)cmd)->uc_compl_arg = arg;
}

void nvim_uc_cmd_set_addr_type(void *cmd, int addr_type)
{
  ((ucmd_T *)cmd)->uc_addr_type = (cmd_addr_T)addr_type;
}

void nvim_uc_cmd_set_luaref(void *cmd, int luaref)
{
  ((ucmd_T *)cmd)->uc_luaref = luaref;
}

void nvim_uc_cmd_set_compl_luaref(void *cmd, int luaref)
{
  ((ucmd_T *)cmd)->uc_compl_luaref = luaref;
}

void nvim_uc_cmd_set_preview_luaref(void *cmd, int luaref)
{
  ((ucmd_T *)cmd)->uc_preview_luaref = luaref;
}

void nvim_uc_cmd_set_script_ctx(void *cmd)
{
  ((ucmd_T *)cmd)->uc_script_ctx = current_sctx;
  ((ucmd_T *)cmd)->uc_script_ctx.sc_lnum += SOURCING_LNUM;
  nlua_set_sctx(&((ucmd_T *)cmd)->uc_script_ctx);
}

void nvim_uc_cmd_free_rep(void *cmd)
{
  XFREE_CLEAR(((ucmd_T *)cmd)->uc_rep);
}

void nvim_uc_cmd_free_compl_arg(void *cmd)
{
  XFREE_CLEAR(((ucmd_T *)cmd)->uc_compl_arg);
}

void nvim_uc_cmd_clear_luaref(void *cmd)
{
  NLUA_CLEAR_REF(((ucmd_T *)cmd)->uc_luaref);
}

void nvim_uc_cmd_clear_compl_luaref(void *cmd)
{
  NLUA_CLEAR_REF(((ucmd_T *)cmd)->uc_compl_luaref);
}

void nvim_uc_cmd_clear_preview_luaref(void *cmd)
{
  NLUA_CLEAR_REF(((ucmd_T *)cmd)->uc_preview_luaref);
}

void nvim_uc_free_ucmd(void *cmd)
{
  xfree(((ucmd_T *)cmd)->uc_name);
  xfree(((ucmd_T *)cmd)->uc_rep);
  xfree(((ucmd_T *)cmd)->uc_compl_arg);
  NLUA_CLEAR_REF(((ucmd_T *)cmd)->uc_compl_luaref);
  NLUA_CLEAR_REF(((ucmd_T *)cmd)->uc_luaref);
  NLUA_CLEAR_REF(((ucmd_T *)cmd)->uc_preview_luaref);
}

void nvim_uc_xfree(void *ptr)
{
  xfree(ptr);
}

void nvim_uc_nlua_clear_ref(int ref_val)
{
  NLUA_CLEAR_REF(ref_val);
}

char *nvim_uc_replace_termcodes(const char *rep, size_t replen)
{
  char *buf = NULL;
  replace_termcodes(rep, replen, &buf, 0, 0, NULL, p_cpo);
  if (buf == NULL) {
    buf = xstrdup(rep);
  }
  return buf;
}

int nvim_uc_get_current_sctx_sid(void)
{
  return current_sctx.sc_sid;
}

int nvim_uc_get_current_sctx_seq(void)
{
  return current_sctx.sc_seq;
}

// Rust FFI declarations (Phase 6: ex command handlers)
extern void rs_ex_command(void *eap);
extern void rs_ex_comclear(void *eap);
extern void rs_ex_delcommand(void *eap);

// C accessor functions called by Rust (Phase 6)
char *nvim_uc_skiptowhite(const char *p)
{
  return skiptowhite(p);
}

char *nvim_uc_skipwhite(const char *p)
{
  return skipwhite(p);
}

int nvim_uc_ends_excmd(int c)
{
  return ends_excmd(c) ? 1 : 0;
}

void nvim_uc_list(const char *name, size_t name_len)
{
  uc_list((char *)name, name_len);
}

void nvim_uc_cmd_memmove_up(void *gap, int i)
{
  garray_T *g = (garray_T *)gap;
  ucmd_T *cmd = USER_CMD_GA(g, i);
  memmove(cmd, cmd + 1, (size_t)(g->ga_len - i) * sizeof(ucmd_T));
}

int nvim_uc_curbuf_is_null(void)
{
  return curbuf == NULL ? 1 : 0;
}

static const char e_argument_required_for_str[]
  = N_("E179: Argument required for %s");
static const char e_no_such_user_defined_command_str[]
  = N_("E184: No such user-defined command: %s");
static const char e_complete_used_without_allowing_arguments[]
  = N_("E1208: -complete used without allowing arguments");
static const char e_no_such_user_defined_command_in_current_buffer_str[]
  = N_("E1237: No such user-defined command in current buffer: %s");

/// List of names for completion for ":command" with the EXPAND_ flag.
/// Must be alphabetical for completion.
static const char *command_complete[] = {
  [EXPAND_ARGLIST] = "arglist",
  [EXPAND_AUGROUP] = "augroup",
  [EXPAND_BUFFERS] = "buffer",
  [EXPAND_CHECKHEALTH] = "checkhealth",
  [EXPAND_COLORS] = "color",
  [EXPAND_COMMANDS] = "command",
  [EXPAND_COMPILER] = "compiler",
  [EXPAND_USER_DEFINED] = "custom",
  [EXPAND_USER_LIST] = "customlist",
  [EXPAND_USER_LUA] = "<Lua function>",
  [EXPAND_DIFF_BUFFERS] = "diff_buffer",
  [EXPAND_DIRECTORIES] = "dir",
  [EXPAND_ENV_VARS] = "environment",
  [EXPAND_EVENTS] = "event",
  [EXPAND_EXPRESSION] = "expression",
  [EXPAND_FILES] = "file",
  [EXPAND_FILES_IN_PATH] = "file_in_path",
  [EXPAND_FILETYPE] = "filetype",
  [EXPAND_FILETYPECMD] = "filetypecmd",
  [EXPAND_FUNCTIONS] = "function",
  [EXPAND_HELP] = "help",
  [EXPAND_HIGHLIGHT] = "highlight",
  [EXPAND_HISTORY] = "history",
  [EXPAND_KEYMAP] = "keymap",
#ifdef HAVE_WORKING_LIBINTL
  [EXPAND_LOCALES] = "locale",
#endif
  [EXPAND_LUA] = "lua",
  [EXPAND_MAPCLEAR] = "mapclear",
  [EXPAND_MAPPINGS] = "mapping",
  [EXPAND_MENUS] = "menu",
  [EXPAND_MESSAGES] = "messages",
  [EXPAND_OWNSYNTAX] = "syntax",
  [EXPAND_SYNTIME] = "syntime",
  [EXPAND_SETTINGS] = "option",
  [EXPAND_PACKADD] = "packadd",
  [EXPAND_RETAB] = "retab",
  [EXPAND_RUNTIME] = "runtime",
  [EXPAND_SHELLCMD] = "shellcmd",
  [EXPAND_SHELLCMDLINE] = "shellcmdline",
  [EXPAND_SIGN] = "sign",
  [EXPAND_TAGS] = "tag",
  [EXPAND_TAGS_LISTFILES] = "tag_listfiles",
  [EXPAND_USER] = "user",
  [EXPAND_USER_VARS] = "var",
  [EXPAND_BREAKPOINT] = "breakpoint",
  [EXPAND_SCRIPTNAMES] = "scriptnames",
  [EXPAND_DIRS_IN_CDPATH] = "dir_in_path",
};

/// List of names of address types.  Must be alphabetical for completion.
static struct {
  cmd_addr_T expand;
  char *name;
  char *shortname;
} addr_type_complete[] = {
  { ADDR_ARGUMENTS, "arguments", "arg" },
  { ADDR_LINES, "lines", "line" },
  { ADDR_LOADED_BUFFERS, "loaded_buffers", "load" },
  { ADDR_TABS, "tabs", "tab" },
  { ADDR_BUFFERS, "buffers", "buf" },
  { ADDR_WINDOWS, "windows", "win" },
  { ADDR_QUICKFIX, "quickfix", "qf" },
  { ADDR_OTHER, "other", "?" },
  { ADDR_NONE, NULL, NULL }
};

/// Search for a user command that matches "eap->cmd".
/// Return cmdidx in "eap->cmdidx", flags in "eap->argt", idx in "eap->useridx".
/// Return a pointer to just after the command.
/// Return NULL if there is no matching command.
///
/// @param *p      end of the command (possibly including count)
/// @param full    set to true for a full match
/// @param xp      used for completion, NULL otherwise
/// @param complp  completion flags or NULL
char *find_ucmd(exarg_T *eap, char *p, int *full, expand_T *xp, int *complp)
{
  int len = (int)(p - eap->cmd);
  int matchlen = 0;
  bool found = false;
  bool possible = false;
  bool amb_local = false;            // Found ambiguous buffer-local command,
                                     // only full match global is accepted.

  // Look for buffer-local user commands first, then global ones.
  garray_T *gap = &prevwin_curwin()->w_buffer->b_ucmds;
  while (true) {
    int j;
    for (j = 0; j < gap->ga_len; j++) {
      ucmd_T *uc = USER_CMD_GA(gap, j);
      char *cp = eap->cmd;
      char *np = uc->uc_name;
      int k = 0;
      while (k < len && *np != NUL && *cp++ == *np++) {
        k++;
      }
      if (k == len || (*np == NUL && ascii_isdigit(eap->cmd[k]))) {
        // If finding a second match, the command is ambiguous.  But
        // not if a buffer-local command wasn't a full match and a
        // global command is a full match.
        if (k == len && found && *np != NUL) {
          if (gap == &ucmds) {
            return NULL;
          }
          amb_local = true;
        }

        if (!found || (k == len && *np == NUL)) {
          // If we matched up to a digit, then there could
          // be another command including the digit that we
          // should use instead.
          if (k == len) {
            found = true;
          } else {
            possible = true;
          }

          if (gap == &ucmds) {
            eap->cmdidx = CMD_USER;
          } else {
            eap->cmdidx = CMD_USER_BUF;
          }
          eap->argt = uc->uc_argt;
          eap->useridx = j;
          eap->addr_type = uc->uc_addr_type;

          if (complp != NULL) {
            *complp = uc->uc_compl;
          }
          if (xp != NULL) {
            xp->xp_luaref = uc->uc_compl_luaref;
            xp->xp_arg = uc->uc_compl_arg;
            xp->xp_script_ctx = uc->uc_script_ctx;
            xp->xp_script_ctx.sc_lnum += SOURCING_LNUM;
          }
          // Do not search for further abbreviations
          // if this is an exact match.
          matchlen = k;
          if (k == len && *np == NUL) {
            if (full != NULL) {
              *full = true;
            }
            amb_local = false;
            break;
          }
        }
      }
    }

    // Stop if we found a full match or searched all.
    if (j < gap->ga_len || gap == &ucmds) {
      break;
    }
    gap = &ucmds;
  }

  // Only found ambiguous matches.
  if (amb_local) {
    if (xp != NULL) {
      xp->xp_context = EXPAND_UNSUCCESSFUL;
    }
    return NULL;
  }

  // The match we found may be followed immediately by a number.  Move "p"
  // back to point to it.
  if (found || possible) {
    return p + (matchlen - len);
  }
  return p;
}

/// Set completion context for :command
const char *set_context_in_user_cmd(expand_T *xp, const char *arg_in)
{
  const char *arg = arg_in;
  const char *p;

  // Check for attributes
  while (*arg == '-') {
    arg++;  // Skip "-".
    p = skiptowhite(arg);
    if (*p == NUL) {
      // Cursor is still in the attribute.
      p = strchr(arg, '=');
      if (p == NULL) {
        // No "=", so complete attribute names.
        xp->xp_context = EXPAND_USER_CMD_FLAGS;
        xp->xp_pattern = (char *)arg;
        return NULL;
      }

      // For the -complete, -nargs and -addr attributes, we complete
      // their arguments as well.
      if (STRNICMP(arg, "complete", p - arg) == 0) {
        xp->xp_context = EXPAND_USER_COMPLETE;
        xp->xp_pattern = (char *)p + 1;
        return NULL;
      } else if (STRNICMP(arg, "nargs", p - arg) == 0) {
        xp->xp_context = EXPAND_USER_NARGS;
        xp->xp_pattern = (char *)p + 1;
        return NULL;
      } else if (STRNICMP(arg, "addr", p - arg) == 0) {
        xp->xp_context = EXPAND_USER_ADDR_TYPE;
        xp->xp_pattern = (char *)p + 1;
        return NULL;
      }
      return NULL;
    }
    arg = skipwhite(p);
  }

  // After the attributes comes the new command name.
  p = skiptowhite(arg);
  if (*p == NUL) {
    xp->xp_context = EXPAND_USER_COMMANDS;
    xp->xp_pattern = (char *)arg;
    return NULL;
  }

  // And finally comes a normal command.
  return skipwhite(p);
}

/// Set the completion context for the argument of a user defined command.
const char *set_context_in_user_cmdarg(const char *cmd FUNC_ATTR_UNUSED, const char *arg,
                                       uint32_t argt, int context, expand_T *xp, bool forceit)
{
  if (context == EXPAND_NOTHING) {
    return NULL;
  }

  if (argt & EX_XFILE) {
    // EX_XFILE: file names are handled before this call.
    return NULL;
  }

  if (context == EXPAND_MENUS) {
    return set_context_in_menu_cmd(xp, cmd, (char *)arg, forceit);
  }
  if (context == EXPAND_COMMANDS) {
    return arg;
  }
  if (context == EXPAND_MAPPINGS) {
    return set_context_in_map_cmd(xp, "map", (char *)arg, forceit, false, false,
                                  CMD_map);
  }
  // Find start of last argument.
  const char *p = arg;
  while (*p) {
    if (*p == ' ') {
      // argument starts after a space
      arg = p + 1;
    } else if (*p == '\\' && *(p + 1) != NUL) {
      p++;  // skip over escaped character
    }
    MB_PTR_ADV(p);
  }
  xp->xp_pattern = (char *)arg;
  xp->xp_context = context;

  return NULL;
}

char *expand_user_command_name(int idx)
{
  return get_user_commands(NULL, idx - CMD_SIZE);
}

/// Function given to ExpandGeneric() to obtain the list of user command names.
char *get_user_commands(expand_T *xp FUNC_ATTR_UNUSED, int idx)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  // In cmdwin, the alternative buffer should be used.
  const buf_T *const buf = prevwin_curwin()->w_buffer;

  if (idx < buf->b_ucmds.ga_len) {
    return USER_CMD_GA(&buf->b_ucmds, idx)->uc_name;
  }

  idx -= buf->b_ucmds.ga_len;
  if (idx < ucmds.ga_len) {
    char *name = USER_CMD(idx)->uc_name;

    for (int i = 0; i < buf->b_ucmds.ga_len; i++) {
      if (strcmp(name, USER_CMD_GA(&buf->b_ucmds, i)->uc_name) == 0) {
        // global command is overruled by buffer-local one
        return "";
      }
    }
    return name;
  }
  return NULL;
}

/// Get the name of user command "idx".  "cmdidx" can be CMD_USER or
/// CMD_USER_BUF.
///
/// @return  NULL if the command is not found.
char *get_user_command_name(int idx, int cmdidx)
{
  if (cmdidx == CMD_USER && idx < ucmds.ga_len) {
    return USER_CMD(idx)->uc_name;
  }
  if (cmdidx == CMD_USER_BUF) {
    // In cmdwin, the alternative buffer should be used.
    const buf_T *const buf = prevwin_curwin()->w_buffer;

    if (idx < buf->b_ucmds.ga_len) {
      return USER_CMD_GA(&buf->b_ucmds, idx)->uc_name;
    }
  }
  return NULL;
}

/// Function given to ExpandGeneric() to obtain the list of user address type names.
char *get_user_cmd_addr_type(expand_T *xp, int idx)
{
  return (char *)rs_get_user_cmd_addr_type(idx);
}

/// Function given to ExpandGeneric() to obtain the list of user command
/// attributes.
char *get_user_cmd_flags(expand_T *xp, int idx)
{
  return (char *)rs_get_user_cmd_flags(idx);
}

/// Function given to ExpandGeneric() to obtain the list of values for -nargs.
char *get_user_cmd_nargs(expand_T *xp, int idx)
{
  return (char *)rs_get_user_cmd_nargs(idx);
}

static char *get_command_complete(int arg)
{
  return (char *)rs_get_command_complete(arg);
}

/// Function given to ExpandGeneric() to obtain the list of values for -complete.
char *get_user_cmd_complete(expand_T *xp, int idx)
{
  return (char *)rs_get_user_cmd_complete(idx);
}

/// Get the name of completion type "expand" as an allocated string.
/// "compl_arg" is the function name for "custom" and "customlist" types.
/// Returns NULL if no completion is available.
char *cmdcomplete_type_to_str(int expand, const char *compl_arg)
{
  // Query length from Rust
  int len = rs_cmdcomplete_type_to_str(expand, compl_arg, NULL, 0);
  if (len < 0) {
    return NULL;
  }
  char *buf = xmalloc((size_t)len + 1);
  rs_cmdcomplete_type_to_str(expand, compl_arg, buf, (size_t)len + 1);
  return buf;
}

int cmdcomplete_str_to_type(const char *complete_str)
{
  return rs_usercmd_str_to_type(complete_str);
}

static void uc_list(char *name, size_t name_len)
{
  bool found = false;

  msg_ext_set_kind("list_cmd");
  // In cmdwin, the alternative buffer should be used.
  const garray_T *gap = &prevwin_curwin()->w_buffer->b_ucmds;
  while (true) {
    int i;
    for (i = 0; i < gap->ga_len; i++) {
      ucmd_T *cmd = USER_CMD_GA(gap, i);
      uint32_t a = cmd->uc_argt;

      // Skip commands which don't match the requested prefix and
      // commands filtered out.
      if (strncmp(name, cmd->uc_name, name_len) != 0
          || message_filtered(cmd->uc_name)) {
        continue;
      }

      // Put out the title first time
      if (!found) {
        msg_puts_title(_("\n    Name              Args Address Complete    Definition"));
      }
      found = true;
      msg_putchar('\n');
      if (got_int) {
        break;
      }

      // Special cases
      size_t len = 4;
      if (a & EX_BANG) {
        msg_putchar('!');
        len--;
      }
      if (a & EX_REGSTR) {
        msg_putchar('"');
        len--;
      }
      if (gap != &ucmds) {
        msg_putchar('b');
        len--;
      }
      if (a & EX_TRLBAR) {
        msg_putchar('|');
        len--;
      }
      if (len != 0) {
        msg_puts(&"    "[4 - len]);
      }

      msg_outtrans(cmd->uc_name, HLF_D, false);
      len = strlen(cmd->uc_name) + 4;

      if (len < 21) {
        // Field padding spaces   12345678901234567
        static char spaces[18] = "                 ";
        msg_puts(&spaces[len - 4]);
        len = 21;
      }
      msg_putchar(' ');
      len++;

      // "over" is how much longer the name is than the column width for
      // the name, we'll try to align what comes after.
      const int64_t over = (int64_t)len - 22;
      len = 0;

      // Arguments
      switch (a & (EX_EXTRA | EX_NOSPC | EX_NEEDARG)) {
      case 0:
        IObuff[len++] = '0';
        break;
      case (EX_EXTRA):
        IObuff[len++] = '*';
        break;
      case (EX_EXTRA | EX_NOSPC):
        IObuff[len++] = '?';
        break;
      case (EX_EXTRA | EX_NEEDARG):
        IObuff[len++] = '+';
        break;
      case (EX_EXTRA | EX_NOSPC | EX_NEEDARG):
        IObuff[len++] = '1';
        break;
      }

      do {
        IObuff[len++] = ' ';
      } while ((int64_t)len < 5 - over);

      // Address / Range
      if (a & (EX_RANGE | EX_COUNT)) {
        if (a & EX_COUNT) {
          // -count=N
          int rc = snprintf(IObuff + len, IOSIZE - len, "%" PRId64 "c", cmd->uc_def);
          assert(rc > 0);
          len += (size_t)rc;
        } else if (a & EX_DFLALL) {
          IObuff[len++] = '%';
        } else if (cmd->uc_def >= 0) {
          // -range=N
          int rc = snprintf(IObuff + len, IOSIZE - len, "%" PRId64 "", cmd->uc_def);
          assert(rc > 0);
          len += (size_t)rc;
        } else {
          IObuff[len++] = '.';
        }
      }

      do {
        IObuff[len++] = ' ';
      } while ((int64_t)len < 8 - over);

      // Address Type
      for (int j = 0; addr_type_complete[j].expand != ADDR_NONE; j++) {
        if (addr_type_complete[j].expand != ADDR_LINES
            && addr_type_complete[j].expand == cmd->uc_addr_type) {
          int rc = snprintf(IObuff + len, IOSIZE - len, "%s", addr_type_complete[j].shortname);
          assert(rc > 0);
          len += (size_t)rc;
          break;
        }
      }

      do {
        IObuff[len++] = ' ';
      } while ((int64_t)len < 13 - over);

      // Completion
      char *cmd_compl = get_command_complete(cmd->uc_compl);
      if (cmd_compl != NULL) {
        int rc = snprintf(IObuff + len, IOSIZE - len, "%s", get_command_complete(cmd->uc_compl));
        assert(rc > 0);
        len += (size_t)rc;
      }

      do {
        IObuff[len++] = ' ';
      } while ((int64_t)len < 25 - over);

      IObuff[len] = NUL;
      msg_outtrans(IObuff, 0, false);

      if (cmd->uc_luaref != LUA_NOREF) {
        char *fn = nlua_funcref_str(cmd->uc_luaref, NULL);
        msg_puts_hl(fn, HLF_8, false);
        xfree(fn);
        // put the description on a new line
        if (*cmd->uc_rep != NUL) {
          msg_puts("\n                                               ");
        }
      }

      msg_outtrans_special(cmd->uc_rep, false,
                           name_len == 0 ? Columns - 47 : 0);
      if (p_verbose > 0) {
        last_set_msg(cmd->uc_script_ctx);
      }
      line_breakcheck();
      if (got_int) {
        break;
      }
    }
    if (gap == &ucmds || i < gap->ga_len) {
      break;
    }
    gap = &ucmds;
  }

  if (!found) {
    msg(_("No user-defined commands found"), 0);
  }
}

/// Parse address type argument
int parse_addr_type_arg(char *value, int vallen, cmd_addr_T *addr_type_arg)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_parse_addr_type_arg(value, vallen, (int *)addr_type_arg);
}

/// Parse a completion argument "value[vallen]".
/// The detected completion goes in "*complp", argument type in "*argt".
/// When there is an argument, for function and user defined completion, it's
/// copied to allocated memory and stored in "*compl_arg".
///
/// @return  FAIL if something is wrong.
int parse_compl_arg(const char *value, int vallen, int *complp, uint32_t *argt, char **compl_arg)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_parse_compl_arg(value, vallen, complp, argt, compl_arg);
}

static int uc_scan_attr(char *attr, size_t len, uint32_t *argt, int *def, int *flags, int *complp,
                        char **compl_arg, cmd_addr_T *addr_type_arg)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_uc_scan_attr(attr, len, argt, def, flags, complp, compl_arg, (int *)addr_type_arg);
}

/// Check for a valid user command name
///
/// If the given {name} is valid, then a pointer to the end of the valid name is returned.
/// Otherwise, returns NULL.
char *uc_validate_name(char *name)
{
  return (char *)rs_uc_validate_name(name);
}

/// Create a new user command {name}, if one doesn't already exist.
///
/// This function takes ownership of compl_arg, compl_luaref, and luaref.
///
/// @return  OK if the command is created, FAIL otherwise.
int uc_add_command(char *name, size_t name_len, const char *rep, uint32_t argt, int64_t def,
                   int flags, int context, char *compl_arg, LuaRef compl_luaref,
                   LuaRef preview_luaref, cmd_addr_T addr_type, LuaRef luaref, bool force)
  FUNC_ATTR_NONNULL_ARG(1, 3)
{
  return rs_uc_add_command(name, name_len, rep, argt, def, flags, context,
                           compl_arg, compl_luaref, preview_luaref,
                           (int)addr_type, luaref, force ? 1 : 0);
}

/// ":command ..."
void ex_command(exarg_T *eap)
{
  rs_ex_command(eap);
}

/// ":comclear"
/// Clear all user commands, global and for current buffer.
void ex_comclear(exarg_T *eap)
{
  rs_ex_comclear(eap);
}

void free_ucmd(ucmd_T *cmd)
{
  rs_free_ucmd(cmd);
}

/// Clear all user commands for "gap".
void uc_clear(garray_T *gap)
{
  rs_uc_clear(gap);
}

void ex_delcommand(exarg_T *eap)
{
  rs_ex_delcommand(eap);
}

/// Split a string by unescaped whitespace (space & tab), used for f-args on Lua commands callback.
/// Similar to uc_split_args(), but does not allocate, add quotes, add commas and is an iterator.
///
/// @param[in]  arg String to split
/// @param[in]  arglen Length of {arg}
/// @param[inout] end Index of last character of previous iteration
/// @param[out] buf Buffer to copy string into
/// @param[out] len Length of string in {buf}
///
/// @return true if iteration is complete, else false
bool uc_split_args_iter(const char *arg, size_t arglen, size_t *end, char *buf, size_t *len)
{
  if (!arglen) {
    return true;
  }

  size_t pos = *end;
  while (pos < arglen && ascii_iswhite(arg[pos])) {
    pos++;
  }

  size_t l = 0;
  for (; pos < arglen - 1; pos++) {
    if (arg[pos] == '\\' && (arg[pos + 1] == '\\' || ascii_iswhite(arg[pos + 1]))) {
      buf[l++] = arg[++pos];
    } else {
      buf[l++] = arg[pos];
    }
    if (ascii_iswhite(arg[pos + 1])) {
      *end = pos + 1;
      *len = l;
      return false;
    }
  }

  if (pos < arglen && !ascii_iswhite(arg[pos])) {
    buf[l++] = arg[pos];
  }

  *len = l;
  return true;
}

size_t uc_nargs_upper_bound(const char *arg, size_t arglen)
{
  bool was_white = true;  // space before first arg
  size_t nargs = 0;
  for (size_t i = 0; i < arglen; i++) {
    bool is_white = ascii_iswhite(arg[i]);
    if (was_white && !is_white) {
      nargs++;
    }
    was_white = is_white;
  }
  return nargs;
}

/// split and quote args for <f-args>
static char *uc_split_args(const char *arg, char **args, const size_t *arglens, size_t argc,
                           size_t *lenp)
{
  return rs_uc_split_args(arg, args, arglens, argc, lenp);
}

/// Add modifiers from "cmod->cmod_split" to "buf".  Set "multi_mods" when one
/// was added.
///
/// @return the number of bytes added
size_t add_win_cmd_modifiers(char *buf, const cmdmod_T *cmod, bool *multi_mods)
{
  int mm = *multi_mods ? 1 : 0;
  size_t result = rs_add_win_cmd_modifiers(buf, cmod, &mm);
  *multi_mods = mm != 0;
  return result;
}

/// Generate text for the "cmod" command modifiers.
/// If "buf" is NULL just return the length.
size_t uc_mods(char *buf, const cmdmod_T *cmod, bool quote)
{
  return rs_uc_mods(buf, cmod, quote ? 1 : 0);
}

/// Check for a <> code in a user command.
///
/// @param code       points to the '<'.  "len" the length of the <> (inclusive).
/// @param buf        is where the result is to be added.
/// @param cmd        the user command we're expanding
/// @param eap        ex arguments
/// @param split_buf  points to a buffer used for splitting, caller should free it.
/// @param split_len  is the length of what "split_buf" contains.
///
/// @return           the length of the replacement, which has been added to "buf".
///                   Return -1 if there was no match, and only the "<" has been copied.
static size_t uc_check_code(char *code, size_t len, char *buf, ucmd_T *cmd, exarg_T *eap,
                            char **split_buf, size_t *split_len)
{
  return rs_uc_check_code(code, len, buf, cmd, eap, split_buf, split_len);
}

// Rust FFI declarations (Phase 7: do_ucmd)
extern int rs_do_ucmd(void *eap, int preview);

// C accessor functions called by Rust (Phase 7)
int nvim_uc_eap_get_cmdidx(const void *eap)
{
  return (int)((const exarg_T *)eap)->cmdidx;
}

int nvim_uc_eap_get_useridx(const void *eap)
{
  return ((const exarg_T *)eap)->useridx;
}

void *nvim_uc_user_cmd(int idx)
{
  return USER_CMD(idx);
}

void *nvim_uc_prevwin_curwin_buf_ucmd(int idx)
{
  return USER_CMD_GA(&prevwin_curwin()->w_buffer->b_ucmds, idx);
}

int nvim_uc_cmd_get_preview_luaref(const void *cmd)
{
  return ((const ucmd_T *)cmd)->uc_preview_luaref;
}

int nvim_uc_cmd_get_luaref(const void *cmd)
{
  return ((const ucmd_T *)cmd)->uc_luaref;
}

const char *nvim_uc_cmd_get_rep(const void *cmd)
{
  return ((const ucmd_T *)cmd)->uc_rep;
}

uint32_t nvim_uc_cmd_get_argt(const void *cmd)
{
  return ((const ucmd_T *)cmd)->uc_argt;
}

int nvim_uc_nlua_do_ucmd(void *cmd, void *eap, int preview)
{
  return nlua_do_ucmd((ucmd_T *)cmd, (exarg_T *)eap, preview != 0);
}

char *nvim_uc_vim_strchr(const char *p, int c)
{
  return vim_strchr(p, c);
}

void nvim_uc_do_cmdline_with_sctx(char *buf, void *eap, uint32_t argt, int sc_sid)
{
  sctx_T save;
  bool restore = false;
  if ((argt & EX_KEEPSCRIPT) == 0) {
    restore = true;
    save = current_sctx;
    current_sctx.sc_sid = sc_sid;
  }
  do_cmdline(buf, ((exarg_T *)eap)->ea_getline, ((exarg_T *)eap)->cookie,
             DOCMD_VERBOSE | DOCMD_NOWAIT | DOCMD_KEYTYPED);
  if (restore) {
    current_sctx = save;
  }
}

int do_ucmd(exarg_T *eap, bool preview)
{
  return rs_do_ucmd(eap, preview ? 1 : 0);
}

/// Gets a map of maps describing user-commands defined for buffer `buf` or
/// defined globally if `buf` is NULL.
///
/// @param buf  Buffer to inspect, or NULL to get global commands.
///
/// @return Map of maps describing commands
Dict commands_array(buf_T *buf, Arena *arena)
{
  garray_T *gap = (buf == NULL) ? &ucmds : &buf->b_ucmds;

  Dict rv = arena_dict(arena, (size_t)gap->ga_len);
  for (int i = 0; i < gap->ga_len; i++) {
    char arg[2] = { 0, 0 };
    Dict d = arena_dict(arena, 16);
    ucmd_T *cmd = USER_CMD_GA(gap, i);

    PUT_C(d, "name", CSTR_AS_OBJ(cmd->uc_name));
    PUT_C(d, "definition", CSTR_AS_OBJ(cmd->uc_rep));
    PUT_C(d, "script_id", INTEGER_OBJ(cmd->uc_script_ctx.sc_sid));
    PUT_C(d, "bang", BOOLEAN_OBJ(!!(cmd->uc_argt & EX_BANG)));
    PUT_C(d, "bar", BOOLEAN_OBJ(!!(cmd->uc_argt & EX_TRLBAR)));
    PUT_C(d, "register", BOOLEAN_OBJ(!!(cmd->uc_argt & EX_REGSTR)));
    PUT_C(d, "keepscript", BOOLEAN_OBJ(!!(cmd->uc_argt & EX_KEEPSCRIPT)));

    if (cmd->uc_preview_luaref != LUA_NOREF) {
      PUT_C(d, "preview", LUAREF_OBJ(api_new_luaref(cmd->uc_preview_luaref)));
    }

    if (cmd->uc_luaref != LUA_NOREF) {
      PUT_C(d, "callback", LUAREF_OBJ(api_new_luaref(cmd->uc_luaref)));
    }

    switch (cmd->uc_argt & (EX_EXTRA | EX_NOSPC | EX_NEEDARG)) {
    case 0:
      arg[0] = '0'; break;
    case (EX_EXTRA):
      arg[0] = '*'; break;
    case (EX_EXTRA | EX_NOSPC):
      arg[0] = '?'; break;
    case (EX_EXTRA | EX_NEEDARG):
      arg[0] = '+'; break;
    case (EX_EXTRA | EX_NOSPC | EX_NEEDARG):
      arg[0] = '1'; break;
    }
    PUT_C(d, "nargs", CSTR_TO_ARENA_OBJ(arena, arg));

    if (cmd->uc_compl_luaref != LUA_NOREF) {
      PUT_C(d, "complete", LUAREF_OBJ(api_new_luaref(cmd->uc_compl_luaref)));
    } else {
      char *cmd_compl = get_command_complete(cmd->uc_compl);

      PUT_C(d, "complete", (cmd_compl == NULL
                            ? NIL : CSTR_AS_OBJ(cmd_compl)));
    }
    PUT_C(d, "complete_arg", cmd->uc_compl_arg == NULL
          ? NIL : CSTR_AS_OBJ(cmd->uc_compl_arg));

    Object obj = NIL;
    if (cmd->uc_argt & EX_COUNT) {
      if (cmd->uc_def >= 0) {
        obj = STRING_OBJ(arena_printf(arena, "%" PRId64, cmd->uc_def));    // -count=N
      } else {
        obj = CSTR_AS_OBJ("0");    // -count
      }
    }
    PUT_C(d, "count", obj);

    obj = NIL;
    if (cmd->uc_argt & EX_RANGE) {
      if (cmd->uc_argt & EX_DFLALL) {
        obj = STATIC_CSTR_AS_OBJ("%");    // -range=%
      } else if (cmd->uc_def >= 0) {
        obj = STRING_OBJ(arena_printf(arena, "%" PRId64, cmd->uc_def));    // -range=N
      } else {
        obj = STATIC_CSTR_AS_OBJ(".");    // -range
      }
    }
    PUT_C(d, "range", obj);

    obj = NIL;
    for (int j = 0; addr_type_complete[j].expand != ADDR_NONE; j++) {
      if (addr_type_complete[j].expand != ADDR_LINES
          && addr_type_complete[j].expand == cmd->uc_addr_type) {
        obj = CSTR_AS_OBJ(addr_type_complete[j].name);
        break;
      }
    }
    PUT_C(d, "addr", obj);

    PUT_C(rv, cmd->uc_name, DICT_OBJ(d));
  }
  return rv;
}
