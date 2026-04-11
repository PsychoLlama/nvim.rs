// Code for menus.  Used for the GUI and 'wildmenu'.
// GUI/Motif support by Robert Webb

#include <stdbool.h>

#include "nvim/buffer_defs.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/errors.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/memory.h"
#include "nvim/menu.h"
#include "nvim/menu_defs.h"
#include "nvim/message.h"
#include "nvim/pos_defs.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

#include "menu.c.generated.h"

typedef struct {
  int modes;
  int noremap;
  bool unmenu;
  int consumed;
} MenuCmdResult;
extern MenuCmdResult rs_get_menu_cmd_modes(const char *cmd, bool forceit);

/// Returns the \ref MENU_MODES specified by menu command `cmd`.
/// (eg :menu! returns MENU_CMDLINE_MODE | MENU_INSERT_MODE)
///
/// @param[in] cmd      string like "nmenu", "vmenu", etc.
/// @param[in] forceit  bang (!) was given after the command
/// @param[out] noremap If not NULL, the flag it points to is set according
///                     to whether the command is a "nore" command.
/// @param[out] unmenu  If not NULL, the flag it points to is set according
///                     to whether the command is an "unmenu" command.
int get_menu_cmd_modes(const char *cmd, bool forceit, int *noremap, bool *unmenu)
{
  MenuCmdResult result = rs_get_menu_cmd_modes(cmd, forceit);
  if (noremap != NULL) {
    *noremap = result.noremap;
  }
  if (unmenu != NULL) {
    *unmenu = result.unmenu;
  }
  return result.modes;
}

// Translation of menu names.  Just a simple lookup table.

typedef struct {
  char *from;            // English name
  char *from_noamp;      // same, without '&'
  char *to;              // translated name
} menutrans_T;

static garray_T menutrans_ga = GA_EMPTY_INIT_VALUE;

#define FREE_MENUTRANS(mt) \
  menutrans_T *_mt = (mt); \
  xfree(_mt->from); \
  xfree(_mt->from_noamp); \
  xfree(_mt->to)

// Remaining C accessor functions for Rust FFI

/// Get the cmd field from an exarg_T.
const char *nvim_menu_eap_get_cmd(exarg_T *eap) { return eap->cmd; }

/// Get the arg field from an exarg_T.
char *nvim_menu_eap_get_arg(exarg_T *eap) { return eap->arg; }

/// Get the forceit field from an exarg_T.
bool nvim_menu_eap_get_forceit(exarg_T *eap) { return eap->forceit; }

/// Get the addr_count field from an exarg_T.
int nvim_menu_eap_get_addr_count(exarg_T *eap) { return eap->addr_count; }

/// Get the line2 field from an exarg_T.
int nvim_menu_eap_get_line2(exarg_T *eap) { return (int)eap->line2; }

/// Get the line1 field from an exarg_T.
int nvim_menu_eap_get_line1(exarg_T *eap) { return (int)eap->line1; }

/// Get b_visual.vi_start.lnum from curbuf.
int nvim_menu_buf_visual_start_lnum(buf_T *buf) { return buf->b_visual.vi_start.lnum; }

/// Get b_visual.vi_end.lnum from curbuf.
int nvim_menu_buf_visual_end_lnum(buf_T *buf) { return buf->b_visual.vi_end.lnum; }

/// Get b_visual.vi_mode from curbuf.
int nvim_menu_buf_visual_mode(buf_T *buf) { return buf->b_visual.vi_mode; }

/// Get b_visual.vi_curswant from curbuf.
int nvim_menu_buf_visual_curswant(buf_T *buf) { return (int)buf->b_visual.vi_curswant; }

/// Get b_visual.vi_start from curbuf as pos_T.
pos_T nvim_menu_buf_visual_start(buf_T *buf) { return buf->b_visual.vi_start; }

/// Get b_visual.vi_end from curbuf as pos_T.
pos_T nvim_menu_buf_visual_end(buf_T *buf) { return buf->b_visual.vi_end; }

// Completion + Translation accessors for Rust FFI

// expand_T field accessors
void nvim_menu_xp_set_context(expand_T *xp, int ctx) { xp->xp_context = ctx; }

void nvim_menu_xp_set_pattern(expand_T *xp, char *pattern) { xp->xp_pattern = pattern; }

// menutrans_ga accessors
int nvim_menu_menutrans_ga_itemsize(void) { return menutrans_ga.ga_itemsize; }

int nvim_menu_menutrans_ga_init(void)
{
  ga_init(&menutrans_ga, (int)sizeof(menutrans_T), 5);
  return 1;
}

int nvim_menu_menutrans_ga_len(void) { return menutrans_ga.ga_len; }

const char *nvim_menu_menutrans_get_from(int idx)
{
  menutrans_T *tp = (menutrans_T *)menutrans_ga.ga_data;
  return tp[idx].from;
}

const char *nvim_menu_menutrans_get_from_noamp(int idx)
{
  menutrans_T *tp = (menutrans_T *)menutrans_ga.ga_data;
  return tp[idx].from_noamp;
}

const char *nvim_menu_menutrans_get_to(int idx)
{
  menutrans_T *tp = (menutrans_T *)menutrans_ga.ga_data;
  return tp[idx].to;
}

void nvim_menu_menutrans_clear(void) { GA_DEEP_CLEAR(&menutrans_ga, menutrans_T, FREE_MENUTRANS); }

void nvim_menu_menutrans_append(char *from, char *from_noamp, char *to)
{
  if (menutrans_ga.ga_itemsize == 0) {
    ga_init(&menutrans_ga, (int)sizeof(menutrans_T), 5);
  }
  menutrans_T *tp = GA_APPEND_VIA_PTR(menutrans_T, &menutrans_ga);
  tp->from = from;
  tp->from_noamp = from_noamp;
  tp->to = to;
}

// Error message wrapper
void nvim_menu_emsg_invarg(void) { emsg(_(e_invarg)); }

// _Static_assert for Phase 6 constants
_Static_assert(EXPAND_MENUS == 11, "EXPAND_MENUS must be 11");
_Static_assert(EXPAND_MENUNAMES == 21, "EXPAND_MENUNAMES must be 21");
_Static_assert(EXPAND_UNSUCCESSFUL == -2, "EXPAND_UNSUCCESSFUL must be -2");
_Static_assert(EXPAND_NOTHING == 0, "EXPAND_NOTHING must be 0");

// Phase 1: Layout validation for Rust VimMenu #[repr(C)] struct.
// These asserts ensure the Rust struct matches vimmenu_T exactly.
#include <stddef.h>
_Static_assert(sizeof(vimmenu_T) == 192, "vimmenu_T size");
_Static_assert(offsetof(vimmenu_T, modes)    == 0,   "modes offset");
_Static_assert(offsetof(vimmenu_T, enabled)  == 4,   "enabled offset");
_Static_assert(offsetof(vimmenu_T, name)     == 8,   "name offset");
_Static_assert(offsetof(vimmenu_T, dname)    == 16,  "dname offset");
_Static_assert(offsetof(vimmenu_T, en_name)  == 24,  "en_name offset");
_Static_assert(offsetof(vimmenu_T, en_dname) == 32,  "en_dname offset");
_Static_assert(offsetof(vimmenu_T, mnemonic) == 40,  "mnemonic offset");
_Static_assert(offsetof(vimmenu_T, actext)   == 48,  "actext offset");
_Static_assert(offsetof(vimmenu_T, priority) == 56,  "priority offset");
_Static_assert(offsetof(vimmenu_T, strings)  == 64,  "strings offset");
_Static_assert(offsetof(vimmenu_T, noremap)  == 128, "noremap offset");
_Static_assert(offsetof(vimmenu_T, silent)   == 160, "silent offset");
_Static_assert(offsetof(vimmenu_T, children) == 168, "children offset");
_Static_assert(offsetof(vimmenu_T, parent)   == 176, "parent offset");
_Static_assert(offsetof(vimmenu_T, next)     == 184, "next offset");
