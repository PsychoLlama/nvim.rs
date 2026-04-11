// Code for menus.  Used for the GUI and 'wildmenu'.
// GUI/Motif support by Robert Webb

#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/cursor.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/getchar_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memory.h"
#include "nvim/menu.h"
#include "nvim/menu_defs.h"
#include "nvim/message.h"
#include "nvim/option_vars.h"
#include "nvim/popupmenu.h"
#include "nvim/pos_defs.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/vim_defs.h"

#define MENUDEPTH   10          // maximum depth of menus

#include "menu.c.generated.h"

typedef struct {
  int modes;
  int noremap;
  bool unmenu;
  int consumed;
} MenuCmdResult;
extern MenuCmdResult rs_get_menu_cmd_modes(const char *cmd, bool forceit);

// Forward declarations for Rust-exported static functions used by remaining C code.
extern bool menu_is_hidden(char *name);
extern char *get_menu_mode_str(int modes);
extern vimmenu_T *find_menu(vimmenu_T *menu, char *name, int modes);
extern char *menu_name_skip(char *name);
extern bool menu_name_equal(const char *name, const vimmenu_T *menu);
extern void execute_menu(const exarg_T *eap, vimmenu_T *menu, int mode_idx);

/// The character for each menu mode
static char *menu_mode_chars[] = { "n", "v", "s", "o", "i", "c", "tl", "t" };

static vimmenu_T **get_root_menu(const char *const name)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return &root_menu;
}

/// Export menus
///
/// @param[in] menu if null, starts from root_menu
/// @param modes, a choice of \ref MENU_MODES
/// @return dict with name/commands
/// @see show_menus_recursive
/// @see menu_get
static dict_T *menu_get_recursive(const vimmenu_T *menu, int modes)
{
  if (!menu || (menu->modes & modes) == 0x0) {
    return NULL;
  }

  dict_T *dict = tv_dict_alloc();
  tv_dict_add_str(dict, S_LEN("name"), menu->dname);
  tv_dict_add_nr(dict, S_LEN("priority"), menu->priority);
  tv_dict_add_nr(dict, S_LEN("hidden"), menu_is_hidden(menu->dname));

  if (menu->mnemonic) {
    char buf[MB_MAXCHAR + 1] = { 0 };  // > max value of utf8_char2bytes
    utf_char2bytes(menu->mnemonic, buf);
    tv_dict_add_str(dict, S_LEN("shortcut"), buf);
  }

  if (menu->actext) {
    tv_dict_add_str(dict, S_LEN("actext"), menu->actext);
  }

  if (menu->modes & MENU_TIP_MODE && menu->strings[MENU_INDEX_TIP]) {
    tv_dict_add_str(dict, S_LEN("tooltip"),
                    menu->strings[MENU_INDEX_TIP]);
  }

  if (!menu->children) {
    // leaf menu
    dict_T *commands = tv_dict_alloc();
    tv_dict_add_dict(dict, S_LEN("mappings"), commands);

    for (int bit = 0; bit < MENU_MODES; bit++) {
      if ((menu->modes & modes & (1 << bit)) != 0) {
        dict_T *impl = tv_dict_alloc();
        tv_dict_add_allocated_str(impl, S_LEN("rhs"),
                                  str2special_save(menu->strings[bit], false, false));
        tv_dict_add_nr(impl, S_LEN("silent"), menu->silent[bit]);
        tv_dict_add_nr(impl, S_LEN("enabled"),
                       (menu->enabled & (1 << bit)) ? 1 : 0);
        tv_dict_add_nr(impl, S_LEN("noremap"),
                       (menu->noremap[bit] & REMAP_NONE) ? 1 : 0);
        tv_dict_add_nr(impl, S_LEN("sid"),
                       (menu->noremap[bit] & REMAP_SCRIPT) ? 1 : 0);
        tv_dict_add_dict(commands, menu_mode_chars[bit], 1, impl);
      }
    }
  } else {
    // visit recursively all children
    list_T *const children_list = tv_list_alloc(kListLenMayKnow);
    for (menu = menu->children; menu != NULL; menu = menu->next) {
      dict_T *d = menu_get_recursive(menu, modes);
      if (tv_dict_len(d) > 0) {
        tv_list_append_dict(children_list, d);
      }
    }
    tv_dict_add_list(dict, S_LEN("submenus"), children_list);
  }
  return dict;
}

/// Export menus matching path \p path_name
///
/// @param path_name
/// @param modes supported modes, see \ref MENU_MODES
/// @param[in,out] list must be allocated
/// @return false if could not find path_name
bool menu_get(char *const path_name, int modes, list_T *list)
{
  vimmenu_T *menu = *get_root_menu(path_name);
  if (*path_name != NUL) {
    menu = find_menu(menu, path_name, modes);
    if (!menu) {
      return false;
    }
  }
  for (; menu != NULL; menu = menu->next) {
    dict_T *d = menu_get_recursive(menu, modes);
    if (d && tv_dict_len(d) > 0) {
      tv_list_append_dict(list, d);
    }
    if (*path_name != NUL) {
      // If a (non-empty) path query was given, only the first node in the
      // find_menu() result is relevant.  Else we want all nodes.
      break;
    }
  }
  return true;
}

/// Returns the \ref MENU_MODES specified by menu command `cmd`.
///  (eg :menu! returns MENU_CMDLINE_MODE | MENU_INSERT_MODE)
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

/// Get the information about a menu item in mode 'which'
static void menuitem_getinfo(const char *menu_name, const vimmenu_T *menu, int modes, dict_T *dict)
  FUNC_ATTR_NONNULL_ALL
{
  if (*menu_name == NUL) {
    // Return all the top-level menus
    list_T *const l = tv_list_alloc(kListLenMayKnow);
    tv_dict_add_list(dict, S_LEN("submenus"), l);
    // get all the children.  Skip PopUp[nvoci].
    for (const vimmenu_T *topmenu = menu; topmenu != NULL; topmenu = topmenu->next) {
      if (!menu_is_hidden(topmenu->dname)) {
        tv_list_append_string(l, topmenu->dname, -1);
      }
    }
    return;
  }

  tv_dict_add_str(dict, S_LEN("name"), menu->name);
  tv_dict_add_str(dict, S_LEN("display"), menu->dname);
  if (menu->actext != NULL) {
    tv_dict_add_str(dict, S_LEN("accel"), menu->actext);
  }
  tv_dict_add_nr(dict, S_LEN("priority"), menu->priority);
  tv_dict_add_str(dict, S_LEN("modes"), get_menu_mode_str(menu->modes));

  char buf[NUMBUFLEN];
  buf[utf_char2bytes(menu->mnemonic, buf)] = NUL;
  tv_dict_add_str(dict, S_LEN("shortcut"), buf);

  if (menu->children == NULL) {  // leaf menu
    int bit;

    // Get the first mode in which the menu is available
    for (bit = 0; (bit < MENU_MODES) && !((1 << bit) & modes); bit++) {}

    if (bit < MENU_MODES) {  // just in case, avoid Coverity warning
      if (menu->strings[bit] != NULL) {
        tv_dict_add_allocated_str(dict, S_LEN("rhs"),
                                  *menu->strings[bit] == NUL
                                  ? xstrdup("<Nop>")
                                  : str2special_save(menu->strings[bit], false, false));
      }
      tv_dict_add_bool(dict, S_LEN("noremenu"), menu->noremap[bit] == REMAP_NONE);
      tv_dict_add_bool(dict, S_LEN("script"), menu->noremap[bit] == REMAP_SCRIPT);
      tv_dict_add_bool(dict, S_LEN("silent"), menu->silent[bit]);
      tv_dict_add_bool(dict, S_LEN("enabled"), (menu->enabled & (1 << bit)) != 0);
    }
  } else {
    // If there are submenus, add all the submenu display names
    list_T *const l = tv_list_alloc(kListLenMayKnow);
    tv_dict_add_list(dict, S_LEN("submenus"), l);
    const vimmenu_T *child = menu->children;
    while (child != NULL) {
      tv_list_append_string(l, child->dname, -1);
      child = child->next;
    }
  }
}

/// "menu_info()" function
/// Return information about a menu (including all the child menus)
void f_menu_info(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  tv_dict_alloc_ret(rettv);
  dict_T *const retdict = rettv->vval.v_dict;

  const char *const menu_name = tv_get_string_chk(&argvars[0]);
  if (menu_name == NULL) {
    return;
  }

  // menu mode
  const char *which;
  if (argvars[1].v_type != VAR_UNKNOWN) {
    which = tv_get_string_chk(&argvars[1]);
  } else {
    which = "";  // Default is modes for "menu"
  }
  if (which == NULL) {
    return;
  }

  const int modes = get_menu_cmd_modes(which, *which == '!', NULL, NULL);

  // Locate the specified menu or menu item
  const vimmenu_T *menu = *get_root_menu(menu_name);
  char *const saved_name = xstrdup(menu_name);
  if (*saved_name != NUL) {
    char *name = saved_name;
    while (*name) {
      // Find in the menu hierarchy
      char *p = menu_name_skip(name);
      while (menu != NULL) {
        if (menu_name_equal(name, menu)) {
          break;
        }
        menu = menu->next;
      }
      if (menu == NULL || *p == NUL) {
        break;
      }
      menu = menu->children;
      name = p;
    }
  }
  xfree(saved_name);

  if (menu == NULL) {  // specified menu not found
    return;
  }

  if (menu->modes & modes) {
    menuitem_getinfo(menu_name, menu, modes, retdict);
  }
}

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

/// Execute a menu item: clears ea, then calls execute_menu.
void nvim_menu_execute(vimmenu_T *mp)
{
  exarg_T ea;
  CLEAR_FIELD(ea);
  execute_menu(&ea, mp, -1);
}

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
