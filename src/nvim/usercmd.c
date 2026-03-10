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

// Rust FFI declarations (window wrappers removed)
extern int rs_tabpage_index(tabpage_T *ftp);

// Forward declarations for Rust-exported functions used within this file
// (formerly static C functions, now globally exported by Rust)
extern char *get_command_complete(int arg);

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
  return rs_tabpage_index(curtab);
}


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


// Phase 8 static assertions
_Static_assert(CMD_map == 274, "CMD_map");
_Static_assert(CMD_SIZE == 556, "CMD_SIZE");
_Static_assert(HLF_D == 5, "HLF_D");
_Static_assert(HLF_8 == 1, "HLF_8");


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

// C accessor functions called by Rust (Phase 8)

// --- find_ucmd accessors ---

void *nvim_uc_prevwin_curwin_buf_ucmds(void)
{
  return &prevwin_curwin()->w_buffer->b_ucmds;
}

void nvim_uc_eap_set_cmdidx(void *eap, int cmdidx)
{
  ((exarg_T *)eap)->cmdidx = (cmdidx_T)cmdidx;
}

void nvim_uc_eap_set_argt(void *eap, uint32_t argt)
{
  ((exarg_T *)eap)->argt = argt;
}

void nvim_uc_eap_set_useridx(void *eap, int useridx)
{
  ((exarg_T *)eap)->useridx = useridx;
}

void nvim_uc_eap_set_addr_type(void *eap, int addr_type)
{
  ((exarg_T *)eap)->addr_type = (cmd_addr_T)addr_type;
}

const char *nvim_uc_eap_get_cmd(const void *eap)
{
  return ((const exarg_T *)eap)->cmd;
}

int nvim_uc_cmd_get_compl(const void *cmd)
{
  return ((const ucmd_T *)cmd)->uc_compl;
}

int nvim_uc_cmd_get_addr_type(const void *cmd)
{
  return (int)((const ucmd_T *)cmd)->uc_addr_type;
}

int nvim_uc_cmd_get_compl_luaref(const void *cmd)
{
  return ((const ucmd_T *)cmd)->uc_compl_luaref;
}

const char *nvim_uc_cmd_get_compl_arg(const void *cmd)
{
  return ((const ucmd_T *)cmd)->uc_compl_arg;
}

int nvim_uc_ascii_isdigit(int c)
{
  return ascii_isdigit(c) ? 1 : 0;
}

// --- expand_T (xp) accessors ---

void nvim_uc_xp_set_context(void *xp, int context)
{
  ((expand_T *)xp)->xp_context = context;
}

void nvim_uc_xp_set_luaref(void *xp, int luaref)
{
  ((expand_T *)xp)->xp_luaref = luaref;
}

void nvim_uc_xp_set_arg(void *xp, char *arg)
{
  ((expand_T *)xp)->xp_arg = arg;
}

void nvim_uc_xp_set_script_ctx(void *xp, const void *cmd)
{
  ((expand_T *)xp)->xp_script_ctx = ((const ucmd_T *)cmd)->uc_script_ctx;
  ((expand_T *)xp)->xp_script_ctx.sc_lnum += SOURCING_LNUM;
}

void nvim_uc_xp_set_pattern(void *xp, char *pattern)
{
  ((expand_T *)xp)->xp_pattern = pattern;
}

// --- uc_list accessors ---

void nvim_uc_msg_ext_set_kind(const char *kind)
{
  msg_ext_set_kind(kind);
}

void nvim_uc_msg_puts_title(const char *s)
{
  msg_puts_title(_(s));
}

void nvim_uc_msg_putchar(int c)
{
  msg_putchar(c);
}

void nvim_uc_msg_puts(const char *s)
{
  msg_puts(s);
}

void nvim_uc_msg_outtrans(const char *s, int attr, int keep)
{
  msg_outtrans(s, attr, keep != 0);
}

void nvim_uc_msg_outtrans_special(const char *s, int from_part, int maxlen)
{
  msg_outtrans_special(s, from_part != 0, maxlen);
}

void nvim_uc_msg_puts_hl(const char *s, int attr, int keep)
{
  msg_puts_hl(s, attr, keep != 0);
}

void nvim_uc_msg(const char *s, int attr)
{
  msg(_(s), attr);
}

int nvim_uc_got_int(void)
{
  return got_int;
}

void nvim_uc_line_breakcheck(void)
{
  line_breakcheck();
}

int nvim_uc_message_filtered(const char *msg_str)
{
  return message_filtered(msg_str) ? 1 : 0;
}

int nvim_uc_get_p_verbose(void)
{
  return p_verbose;
}

int nvim_uc_get_Columns(void)
{
  return Columns;
}

char *nvim_uc_get_IObuff(void)
{
  return IObuff;
}

size_t nvim_uc_get_IOSIZE(void)
{
  return IOSIZE;
}

char *nvim_uc_nlua_funcref_str(int luaref)
{
  return nlua_funcref_str(luaref, NULL);
}

void nvim_uc_last_set_msg(const void *cmd)
{
  last_set_msg(((const ucmd_T *)cmd)->uc_script_ctx);
}

// --- set_context_in_user_cmdarg accessors ---

const char *nvim_uc_set_context_in_menu_cmd(void *xp, const char *cmd, char *arg, int forceit)
{
  return set_context_in_menu_cmd((expand_T *)xp, cmd, arg, forceit != 0);
}

const char *nvim_uc_set_context_in_map_cmd(void *xp, char *arg, int forceit)
{
  return set_context_in_map_cmd((expand_T *)xp, "map", arg, forceit != 0, false, false,
                                CMD_map);
}

void nvim_uc_MB_PTR_ADV(const char **pp)
{
  MB_PTR_ADV(*pp);
}

// --- get_user_commands / get_user_command_name accessors ---

int nvim_uc_prevwin_curwin_buf_ucmds_len(void)
{
  return prevwin_curwin()->w_buffer->b_ucmds.ga_len;
}

void *nvim_uc_prevwin_curwin_buf_ucmd_ga(int i)
{
  return USER_CMD_GA(&prevwin_curwin()->w_buffer->b_ucmds, i);
}

int nvim_uc_get_ucmds_len(void)
{
  return ucmds.ga_len;
}

void *nvim_uc_user_cmd_global(int idx)
{
  return USER_CMD(idx);
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
