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

// Rust implementations of menu name classification functions
extern bool rs_menu_is_winbar(const char *name);
extern bool rs_menu_is_popup(const char *name);
extern bool rs_menu_is_toolbar(const char *name);
extern bool rs_menu_is_menubar(const char *name);
extern bool rs_menu_is_separator(const char *name);
extern bool rs_menu_is_hidden(const char *name);
extern bool rs_menu_name_equal(const char *name, vimmenu_T *menu);

// Rust implementations for Phase 1
typedef struct {
  int modes;
  int noremap;
  bool unmenu;
  int consumed;
} MenuCmdResult;
extern MenuCmdResult rs_get_menu_cmd_modes(const char *cmd, bool forceit);
extern const char *rs_get_menu_mode_str(int modes);
extern char *rs_popup_mode_name(const char *name, int idx);
extern int rs_get_menu_mode(void);
extern int rs_get_menu_mode_flag(void);

// Rust implementations for Phase 2
extern char *rs_menu_name_skip(char *name);
extern const char *rs_menu_skip_part(const char *p);
extern void rs_menu_unescape_name(char *name);
extern char *rs_menu_translate_tab_and_shift(char *arg_start);

typedef struct {
  char *text;
  int mnemonic;
  char *actext;
} MenuTextResult;
extern MenuTextResult rs_menu_text(const char *str);

// Rust implementations for Phase 3
extern vimmenu_T *rs_find_menu(vimmenu_T *menu, char *name, int modes);

// Rust implementations for Phase 4
extern void rs_free_menu_string(vimmenu_T *menu, int idx);
extern void rs_free_menu(vimmenu_T **menup);
extern int rs_menu_enable_recurse(vimmenu_T *menu, char *name, int modes, int enable);
extern int rs_remove_menu(vimmenu_T **menup, char *name, int modes, bool silent);
extern int rs_add_menu_path(const char *menu_path, vimmenu_T *menuarg, const int *pri_tab,
                            const char *call_data);
extern vimmenu_T *rs_menu_getbyname(char *name_arg);
extern vimmenu_T *rs_menu_find(const char *path_name);
extern int rs_show_menus(char *path_name, int modes);
extern void rs_show_menus_recursive(vimmenu_T *menu, int modes, int depth);

// Rust implementations for Phase 5
extern void rs_ex_menu(exarg_T *eap);
extern void rs_ex_emenu(exarg_T *eap);
extern void rs_show_popupmenu(void);

// Rust implementations for Phase 6
extern char *rs_set_context_in_menu_cmd(expand_T *xp, const char *cmd, char *arg, bool forceit);
extern char *rs_get_menu_name(expand_T *xp, int idx);
extern char *rs_get_menu_names(expand_T *xp, int idx);
extern void rs_ex_menutranslate(exarg_T *eap);
extern char *rs_menutrans_lookup(char *name, int len);

/// The character for each menu mode
static char *menu_mode_chars[] = { "n", "v", "s", "o", "i", "c", "tl", "t" };

static const char e_notsubmenu[] = N_("E327: Part of menu-item path is not sub-menu");
static const char e_nomenu[] = N_("E329: No menu \"%s\"");

// Return true if "name" is a window toolbar menu name.
static bool menu_is_winbar(const char *const name)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return rs_menu_is_winbar(name);
}

static vimmenu_T **get_root_menu(const char *const name)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return &root_menu;
}

/// Do the :menu command and relatives.
/// @param eap Ex command arguments
void ex_menu(exarg_T *eap)
{
  rs_ex_menu(eap);
}

/// Add the menu with the given name to the menu hierarchy
///
/// @param[out]  menuarg menu entry
/// @param[] pri_tab priority table
/// @param[in] call_data Right hand side command
static int add_menu_path(const char *const menu_path, vimmenu_T *menuarg, const int *const pri_tab,
                         const char *const call_data)
{
  return rs_add_menu_path(menu_path, menuarg, pri_tab, call_data);
}

// Set the (sub)menu with the given name to enabled or disabled.
// Called recursively.
static int menu_enable_recurse(vimmenu_T *menu, char *name, int modes, int enable)
{
  return rs_menu_enable_recurse(menu, name, modes, enable);
}

/// Remove the (sub)menu with the given name from the menu hierarchy
/// Called recursively.
///
/// @param silent  don't give error messages
static int remove_menu(vimmenu_T **menup, char *name, int modes, bool silent)
{
  return rs_remove_menu(menup, name, modes, silent);
}

// Free the given menu structure and remove it from the linked list.
static void free_menu(vimmenu_T **menup)
{
  rs_free_menu(menup);
}

// Free the menu->string with the given index.
static void free_menu_string(vimmenu_T *menu, int idx)
{
  rs_free_menu_string(menu, idx);
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

/// Find menu matching `name` and `modes`. Does not handle empty `name`.
///
/// @param menu top menu to start looking from
/// @param name path towards the menu
/// @return found menu or NULL
static vimmenu_T *find_menu(vimmenu_T *menu, char *name, int modes)
{
  return rs_find_menu(menu, name, modes);
}

/// Show the mapping associated with a menu item or hierarchy in a sub-menu.
static int show_menus(char *const path_name, int modes)
{
  return rs_show_menus(path_name, modes);
}

/// Recursively show the mappings associated with the menus under the given one
static void show_menus_recursive(vimmenu_T *menu, int modes, int depth)
{
  rs_show_menus_recursive(menu, modes, depth);
}

// Used when expanding menu names.
static vimmenu_T *expand_menu = NULL;
static int expand_modes = 0x0;
static int expand_emenu;                // true for ":emenu" command

// Work out what to complete when doing command line completion of menu names.
char *set_context_in_menu_cmd(expand_T *xp, const char *cmd, char *arg, bool forceit)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_set_context_in_menu_cmd(xp, cmd, arg, forceit);
}

// Function given to ExpandGeneric() to obtain the list of (sub)menus (not
// entries).
char *get_menu_name(expand_T *xp, int idx)
{
  return rs_get_menu_name(xp, idx);
}

// Function given to ExpandGeneric() to obtain the list of menus and menu
// entries.
char *get_menu_names(expand_T *xp, int idx)
{
  return rs_get_menu_names(xp, idx);
}

/// Skip over this element of the menu path and return the start of the next
/// element.  Any \ and ^Vs are removed from the current element.
///
/// @param name may be modified.
/// @return start of the next element
static char *menu_name_skip(char *const name)
{
  return rs_menu_name_skip(name);
}

/// Return true when "name" matches with menu "menu".  The name is compared in
/// two ways: raw menu name and menu name without '&'.  ignore part after a TAB.
static bool menu_name_equal(const char *const name, const vimmenu_T *const menu)
{
  return rs_menu_name_equal(name, (vimmenu_T *)menu);
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

/// Return the string representation of the menu modes. Does the opposite
/// of get_menu_cmd_modes().
static char *get_menu_mode_str(int modes)
{
  return (char *)rs_get_menu_mode_str(modes);
}

// Modify a menu name starting with "PopUp" to include the mode character.
// Returns the name in allocated memory.
static char *popup_mode_name(char *name, int idx)
{
  return rs_popup_mode_name(name, idx);
}

/// Duplicate the menu item text and then process to see if a mnemonic key
/// and/or accelerator text has been identified.
///
/// @param str The menu item text.
/// @param[out] mnemonic If non-NULL, *mnemonic is set to the character after
///             the first '&'.
/// @param[out] actext If non-NULL, *actext is set to the text after the first
///             TAB, but only if a TAB was found. Memory pointed to is newly
///             allocated.
///
/// @return a pointer to allocated memory.
static char *menu_text(const char *str, int *mnemonic, char **actext)
  FUNC_ATTR_NONNULL_RET FUNC_ATTR_WARN_UNUSED_RESULT
  FUNC_ATTR_NONNULL_ARG(1)
{
  MenuTextResult result = rs_menu_text(str);
  if (mnemonic != NULL) {
    *mnemonic = result.mnemonic;
  }
  if (actext != NULL && result.actext != NULL) {
    *actext = result.actext;
  } else {
    xfree(result.actext);
  }
  return result.text;
}

// Return true if "name" can be a menu in the MenuBar.
bool menu_is_menubar(const char *const name)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return rs_menu_is_menubar(name);
}

// Return true if "name" is a popup menu name.
bool menu_is_popup(const char *const name)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return rs_menu_is_popup(name);
}

// Return true if "name" is a toolbar menu name.
bool menu_is_toolbar(const char *const name)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return rs_menu_is_toolbar(name);
}

/// @return  true if the name is a menu separator identifier: Starts and ends
///          with '-'
bool menu_is_separator(char *name)
{
  return rs_menu_is_separator(name);
}

/// True if a popup menu or starts with \ref MNU_HIDDEN_CHAR
///
/// @return true if the menu is hidden
static bool menu_is_hidden(char *name)
{
  return rs_menu_is_hidden(name);
}

static int get_menu_mode(void)
{
  return rs_get_menu_mode();
}

int get_menu_mode_flag(void)
{
  return rs_get_menu_mode_flag();
}

/// Display the Special "PopUp" menu as a pop-up at the current mouse
/// position.  The "PopUpn" menu is for Normal mode, "PopUpi" for Insert mode,
/// etc.
void show_popupmenu(void)
{
  rs_show_popupmenu();
}

/// Execute "menu".  Use by ":emenu" and the window toolbar.
/// @param eap  NULL for the window toolbar.
/// @param mode_idx  specify a MENU_INDEX_ value,
///                  use MENU_INDEX_INVALID to depend on the current state
void execute_menu(const exarg_T *eap, vimmenu_T *menu, int mode_idx)
  FUNC_ATTR_NONNULL_ARG(2)
{
  int idx = mode_idx;

  if (idx < 0) {
    // Use the Insert mode entry when returning to Insert mode.
    if (((State & MODE_INSERT) || restart_edit) && current_sctx.sc_sid == 0) {
      idx = MENU_INDEX_INSERT;
    } else if (State & MODE_CMDLINE) {
      idx = MENU_INDEX_CMDLINE;
    } else if (State & MODE_TERMINAL) {
      idx = MENU_INDEX_TERMINAL;
    } else if (get_real_state() & MODE_VISUAL) {
      // Detect real visual mode -- if we are really in visual mode we
      // don't need to do any guesswork to figure out what the selection
      // is. Just execute the visual binding for the menu.
      idx = MENU_INDEX_VISUAL;
    } else if (eap != NULL && eap->addr_count) {
      pos_T tpos;

      idx = MENU_INDEX_VISUAL;

      // GEDDES: This is not perfect - but it is a
      // quick way of detecting whether we are doing this from a
      // selection - see if the range matches up with the visual
      // select start and end.
      if ((curbuf->b_visual.vi_start.lnum == eap->line1)
          && (curbuf->b_visual.vi_end.lnum) == eap->line2) {
        // Set it up for visual mode - equivalent to gv.
        VIsual_mode = curbuf->b_visual.vi_mode;
        tpos = curbuf->b_visual.vi_end;
        curwin->w_cursor = curbuf->b_visual.vi_start;
        curwin->w_curswant = curbuf->b_visual.vi_curswant;
      } else {
        // Set it up for line-wise visual mode
        VIsual_mode = 'V';
        curwin->w_cursor.lnum = eap->line1;
        curwin->w_cursor.col = 1;
        tpos.lnum = eap->line2;
        tpos.col = MAXCOL;
        tpos.coladd = 0;
      }

      // Activate visual mode
      VIsual_active = true;
      VIsual_reselect = true;
      check_cursor(curwin);
      VIsual = curwin->w_cursor;
      curwin->w_cursor = tpos;

      check_cursor(curwin);

      // Adjust the cursor to make sure it is in the correct pos
      // for exclusive mode
      if (*p_sel == 'e' && gchar_cursor() != NUL) {
        curwin->w_cursor.col++;
      }
    }
  }

  if (idx == MENU_INDEX_INVALID || eap == NULL) {
    idx = MENU_INDEX_NORMAL;
  }

  if (menu->strings[idx] != NULL && (menu->modes & (1 << idx))) {
    // When executing a script or function execute the commands right now.
    // Also for the window toolbar
    // Otherwise put them in the typeahead buffer.
    if (eap == NULL || current_sctx.sc_sid != 0) {
      save_state_T save_state;

      ex_normal_busy++;
      if (save_current_state(&save_state)) {
        exec_normal_cmd(menu->strings[idx], menu->noremap[idx],
                        menu->silent[idx]);
      }
      restore_current_state(&save_state);
      ex_normal_busy--;
    } else {
      ins_typebuf(menu->strings[idx], menu->noremap[idx], 0, true,
                  menu->silent[idx]);
    }
  } else if (eap != NULL) {
    char *mode;
    switch (idx) {
    case MENU_INDEX_VISUAL:
      mode = "Visual";
      break;
    case MENU_INDEX_SELECT:
      mode = "Select";
      break;
    case MENU_INDEX_OP_PENDING:
      mode = "Op-pending";
      break;
    case MENU_INDEX_TERMINAL:
      mode = "Terminal";
      break;
    case MENU_INDEX_INSERT:
      mode = "Insert";
      break;
    case MENU_INDEX_CMDLINE:
      mode = "Cmdline";
      break;
    // case MENU_INDEX_TIP: cannot happen
    default:
      mode = "Normal";
    }
    semsg(_("E335: Menu not defined for %s mode"), mode);
  }
}

/// Lookup a menu by the descriptor name e.g. "File.New"
/// Returns NULL if the menu is not found
static vimmenu_T *menu_getbyname(char *name_arg)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_menu_getbyname(name_arg);
}

/// Given a menu descriptor, e.g. "File.New", find it in the menu hierarchy and
/// execute it.
void ex_emenu(exarg_T *eap)
{
  rs_ex_emenu(eap);
}

/// Given a menu descriptor, e.g. "File.New", find it in the menu hierarchy.
vimmenu_T *menu_find(const char *path_name)
{
  return rs_menu_find(path_name);
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

// ":menutrans".
// This function is also defined without the +multi_lang feature, in which
// case the commands are ignored.
void ex_menutranslate(exarg_T *eap)
{
  rs_ex_menutranslate(eap);
}

// Find the character just after one part of a menu name.
static char *menu_skip_part(char *p)
{
  return (char *)rs_menu_skip_part(p);
}

// Lookup part of a menu name in the translations.
// Return a pointer to the translation or NULL if not found.
static char *menutrans_lookup(char *name, int len)
{
  return rs_menutrans_lookup(name, len);
}

// Unescape the name in the translate dictionary table.
static void menu_unescape_name(char *name)
{
  rs_menu_unescape_name(name);
}

// Isolate the menu name.
// Skip the menu name, and translate <Tab> into a real TAB.
static char *menu_translate_tab_and_shift(char *arg_start)
{
  return rs_menu_translate_tab_and_shift(arg_start);
}

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

// ============================================================================
// Remaining C accessor functions for Rust FFI
// ============================================================================

/// Get the cmd field from an exarg_T.
const char *nvim_menu_eap_get_cmd(exarg_T *eap)
{
  return eap->cmd;
}

/// Get the arg field from an exarg_T.
char *nvim_menu_eap_get_arg(exarg_T *eap)
{
  return eap->arg;
}

/// Get the forceit field from an exarg_T.
bool nvim_menu_eap_get_forceit(exarg_T *eap)
{
  return eap->forceit;
}

/// Get the addr_count field from an exarg_T.
int nvim_menu_eap_get_addr_count(exarg_T *eap)
{
  return eap->addr_count;
}

/// Get the line2 field from an exarg_T.
int nvim_menu_eap_get_line2(exarg_T *eap)
{
  return (int)eap->line2;
}

// ============================================================================
// Completion + Translation accessors for Rust FFI
// ============================================================================

// expand_T field accessors
void nvim_menu_xp_set_context(expand_T *xp, int ctx)
{
  xp->xp_context = ctx;
}

void nvim_menu_xp_set_pattern(expand_T *xp, char *pattern)
{
  xp->xp_pattern = pattern;
}

// Static variable accessors for expand_menu, expand_modes, expand_emenu
vimmenu_T *nvim_menu_get_expand_menu(void)
{
  return expand_menu;
}

void nvim_menu_set_expand_menu(vimmenu_T *menu)
{
  expand_menu = menu;
}

int nvim_menu_get_expand_modes(void)
{
  return expand_modes;
}

void nvim_menu_set_expand_modes(int modes)
{
  expand_modes = modes;
}

int nvim_menu_get_expand_emenu(void)
{
  return expand_emenu;
}

void nvim_menu_set_expand_emenu(int v)
{
  expand_emenu = v;
}

// menutrans_ga accessors
int nvim_menu_menutrans_ga_itemsize(void)
{
  return menutrans_ga.ga_itemsize;
}

int nvim_menu_menutrans_ga_init(void)
{
  ga_init(&menutrans_ga, (int)sizeof(menutrans_T), 5);
  return 1;
}

int nvim_menu_menutrans_ga_len(void)
{
  return menutrans_ga.ga_len;
}

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

void nvim_menu_menutrans_clear(void)
{
  GA_DEEP_CLEAR(&menutrans_ga, menutrans_T, FREE_MENUTRANS);
}

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
void nvim_menu_emsg_invarg(void)
{
  emsg(_(e_invarg));
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
