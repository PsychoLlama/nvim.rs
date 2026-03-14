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


// C accessor functions called by Rust (Phase 2)
void nvim_uc_emsg(const char *msg)
{
  emsg(_(msg));
}

void nvim_uc_semsg_1(const char *fmt, const char *arg)
{
  semsg(_(fmt), arg);
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

const void *nvim_uc_get_cmdmod(void)
{
  return &cmdmod;
}


// C accessor functions called by Rust (Phase 5)
void *nvim_uc_get_curbuf_ucmds(void)
{
  return &curbuf->b_ucmds;
}

void nvim_uc_cmd_set_script_ctx(ucmd_T *cmd)
{
  cmd->uc_script_ctx = current_sctx;
  cmd->uc_script_ctx.sc_lnum += SOURCING_LNUM;
  nlua_set_sctx(&cmd->uc_script_ctx);
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

// C accessor functions called by Rust (Phase 6)

int nvim_uc_ends_excmd(int c)
{
  return ends_excmd(c) ? 1 : 0;
}

int nvim_uc_curbuf_is_null(void)
{
  return curbuf == NULL ? 1 : 0;
}


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





// C accessor functions called by Rust (Phase 7)
int nvim_uc_eap_get_cmdidx(const void *eap)
{
  return (int)((const exarg_T *)eap)->cmdidx;
}

int nvim_uc_eap_get_useridx(const void *eap)
{
  return ((const exarg_T *)eap)->useridx;
}

ucmd_T *nvim_uc_prevwin_curwin_buf_ucmd(int idx)
{
  return USER_CMD_GA(&prevwin_curwin()->w_buffer->b_ucmds, idx);
}

int nvim_uc_nlua_do_ucmd(ucmd_T *cmd, void *eap, int preview)
{
  return nlua_do_ucmd((ucmd_T *)cmd, (exarg_T *)eap, preview != 0);
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


void nvim_uc_msg_puts_title(const char *s)
{
  msg_puts_title(_(s));
}


void nvim_uc_msg_puts(const char *s)
{
  msg_puts(s);
}


void nvim_uc_msg(const char *s, int attr)
{
  msg(_(s), attr);
}

int nvim_uc_got_int(void)
{
  return got_int;
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

ucmd_T *nvim_uc_prevwin_curwin_buf_ucmd_ga(int i)
{
  return USER_CMD_GA(&prevwin_curwin()->w_buffer->b_ucmds, i);
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
