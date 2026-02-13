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
  char *after_dot;
  char *p;
  char *path_name = NULL;
  bool unmenu;
  vimmenu_T *menu;

  xp->xp_context = EXPAND_UNSUCCESSFUL;

  // Check for priority numbers, enable and disable
  for (p = arg; *p; p++) {
    if (!ascii_isdigit(*p) && *p != '.') {
      break;
    }
  }

  if (!ascii_iswhite(*p)) {
    if (strncmp(arg, "enable", 6) == 0
        && (arg[6] == NUL || ascii_iswhite(arg[6]))) {
      p = arg + 6;
    } else if (strncmp(arg, "disable", 7) == 0
               && (arg[7] == NUL || ascii_iswhite(arg[7]))) {
      p = arg + 7;
    } else {
      p = arg;
    }
  }

  while (*p != NUL && ascii_iswhite(*p)) {
    p++;
  }

  arg = after_dot = p;

  for (; *p && !ascii_iswhite(*p); p++) {
    if ((*p == '\\' || *p == Ctrl_V) && p[1] != NUL) {
      p++;
    } else if (*p == '.') {
      after_dot = p + 1;
    }
  }

  // ":popup" only uses menus, not entries
  int expand_menus = !((*cmd == 't' && cmd[1] == 'e') || *cmd == 'p');
  expand_emenu = (*cmd == 'e');
  if (expand_menus && ascii_iswhite(*p)) {
    return NULL;  // TODO(vim): check for next command?
  }
  if (*p == NUL) {  // Complete the menu name
    // With :unmenu, you only want to match menus for the appropriate mode.
    // With :menu though you might want to add a menu with the same name as
    // one in another mode, so match menus from other modes too.
    expand_modes = get_menu_cmd_modes(cmd, forceit, NULL, &unmenu);
    if (!unmenu) {
      expand_modes = MENU_ALL_MODES;
    }

    menu = root_menu;
    if (after_dot > arg) {
      size_t path_len = (size_t)(after_dot - arg);
      path_name = xmalloc(path_len);
      xstrlcpy(path_name, arg, path_len);
    }
    char *name = path_name;
    while (name != NULL && *name) {
      p = menu_name_skip(name);
      while (menu != NULL) {
        if (menu_name_equal(name, menu)) {
          // Found menu
          if ((*p != NUL && menu->children == NULL)
              || ((menu->modes & expand_modes) == 0x0)) {
            // Menu path continues, but we have reached a leaf.
            // Or menu exists only in another mode.
            xfree(path_name);
            return NULL;
          }
          break;
        }
        menu = menu->next;
      }
      if (menu == NULL) {
        // No menu found with the name we were looking for
        xfree(path_name);
        return NULL;
      }
      name = p;
      menu = menu->children;
    }
    xfree(path_name);

    xp->xp_context = expand_menus ? EXPAND_MENUNAMES : EXPAND_MENUS;
    xp->xp_pattern = after_dot;
    expand_menu = menu;
  } else {                      // We're in the mapping part
    xp->xp_context = EXPAND_NOTHING;
  }
  return NULL;
}

// Function given to ExpandGeneric() to obtain the list of (sub)menus (not
// entries).
char *get_menu_name(expand_T *xp, int idx)
{
  static vimmenu_T *menu = NULL;
  char *str;
  static bool should_advance = false;

  if (idx == 0) {           // first call: start at first item
    menu = expand_menu;
    should_advance = false;
  }

  // Skip PopUp[nvoci].
  while (menu != NULL && (menu_is_hidden(menu->dname)
                          || menu_is_separator(menu->dname)
                          || menu->children == NULL)) {
    menu = menu->next;
  }

  if (menu == NULL) {       // at end of linked list
    return NULL;
  }

  if (menu->modes & expand_modes) {
    if (should_advance) {
      str = menu->en_dname;
    } else {
      str = menu->dname;
      if (menu->en_dname == NULL) {
        should_advance = true;
      }
    }
  } else {
    str = "";
  }

  if (should_advance) {
    // Advance to next menu entry.
    menu = menu->next;
  }

  should_advance = !should_advance;

  return str;
}

// Function given to ExpandGeneric() to obtain the list of menus and menu
// entries.
char *get_menu_names(expand_T *xp, int idx)
{
  static vimmenu_T *menu = NULL;
#define TBUFFER_LEN 256
  static char tbuffer[TBUFFER_LEN];         // hack
  char *str;
  static bool should_advance = false;

  if (idx == 0) {           // first call: start at first item
    menu = expand_menu;
    should_advance = false;
  }

  // Skip Browse-style entries, popup menus and separators.
  while (menu != NULL
         && (menu_is_hidden(menu->dname)
             || (expand_emenu && menu_is_separator(menu->dname))
             || menu->dname[strlen(menu->dname) - 1] == '.')) {
    menu = menu->next;
  }

  if (menu == NULL) {       // at end of linked list
    return NULL;
  }

  if (menu->modes & expand_modes) {
    if (menu->children != NULL) {
      if (should_advance) {
        xstrlcpy(tbuffer, menu->en_dname, TBUFFER_LEN);
      } else {
        xstrlcpy(tbuffer, menu->dname,  TBUFFER_LEN);
        if (menu->en_dname == NULL) {
          should_advance = true;
        }
      }
      // hack on menu separators:  use a 'magic' char for the separator
      // so that '.' in names gets escaped properly
      strcat(tbuffer, "\001");
      str = tbuffer;
    } else {
      if (should_advance) {
        str = menu->en_dname;
      } else {
        str = menu->dname;
        if (menu->en_dname == NULL) {
          should_advance = true;
        }
      }
    }
  } else {
    str = "";
  }

  if (should_advance) {
    // Advance to next menu entry.
    menu = menu->next;
  }

  should_advance = !should_advance;

  return str;
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
  char *arg = eap->arg;

  if (menutrans_ga.ga_itemsize == 0) {
    ga_init(&menutrans_ga, (int)sizeof(menutrans_T), 5);
  }

  // ":menutrans clear": clear all translations.
  if (strncmp(arg, "clear", 5) == 0 && ends_excmd(*skipwhite(arg + 5))) {
    GA_DEEP_CLEAR(&menutrans_ga, menutrans_T, FREE_MENUTRANS);

    // Delete all "menutrans_" global variables.
    del_menutrans_vars();
  } else {
    // ":menutrans from to": add translation
    char *from = arg;
    arg = menu_skip_part(arg);
    char *to = skipwhite(arg);
    *arg = NUL;
    arg = menu_skip_part(to);
    if (arg == to) {
      emsg(_(e_invarg));
    } else {
      from = xstrdup(from);
      char *from_noamp = menu_text(from, NULL, NULL);
      assert(arg >= to);
      to = xmemdupz(to, (size_t)(arg - to));
      menu_translate_tab_and_shift(from);
      menu_translate_tab_and_shift(to);
      menu_unescape_name(from);
      menu_unescape_name(to);
      menutrans_T *tp = GA_APPEND_VIA_PTR(menutrans_T, &menutrans_ga);
      tp->from = from;
      tp->from_noamp = from_noamp;
      tp->to = to;
    }
  }
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
  menutrans_T *tp = (menutrans_T *)menutrans_ga.ga_data;

  for (int i = 0; i < menutrans_ga.ga_len; i++) {
    if (STRNICMP(name, tp[i].from, len) == 0 && tp[i].from[len] == NUL) {
      return tp[i].to;
    }
  }

  // Now try again while ignoring '&' characters.
  char c = name[len];
  name[len] = NUL;
  char *dname = menu_text(name, NULL, NULL);
  name[len] = c;
  for (int i = 0; i < menutrans_ga.ga_len; i++) {
    if (STRICMP(dname, tp[i].from_noamp) == 0) {
      xfree(dname);
      return tp[i].to;
    }
  }
  xfree(dname);

  return NULL;
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
// C accessor functions for Rust FFI
// ============================================================================

/// Get the modes field from a menu.
int nvim_menu_get_modes(vimmenu_T *menu)
{
  return menu->modes;
}

/// Get the enabled field from a menu.
int nvim_menu_get_enabled(vimmenu_T *menu)
{
  return menu->enabled;
}

/// Get the name field from a menu.
const char *nvim_menu_get_name(vimmenu_T *menu)
{
  return menu->name;
}

/// Get the dname (display name) field from a menu.
const char *nvim_menu_get_dname(vimmenu_T *menu)
{
  return menu->dname;
}

/// Get the en_name (English name) field from a menu.
const char *nvim_menu_get_en_name(vimmenu_T *menu)
{
  return menu->en_name;
}

/// Get the en_dname (English display name) field from a menu.
const char *nvim_menu_get_en_dname(vimmenu_T *menu)
{
  return menu->en_dname;
}

/// Get the priority field from a menu.
int nvim_menu_get_priority(vimmenu_T *menu)
{
  return menu->priority;
}

/// Get the children field from a menu.
vimmenu_T *nvim_menu_get_children(vimmenu_T *menu)
{
  return menu->children;
}

/// Get the parent field from a menu.
vimmenu_T *nvim_menu_get_parent(vimmenu_T *menu)
{
  return menu->parent;
}

/// Get the next sibling field from a menu.
vimmenu_T *nvim_menu_get_next(vimmenu_T *menu)
{
  return menu->next;
}

/// Get the mnemonic field from a menu.
int nvim_menu_get_mnemonic(vimmenu_T *menu)
{
  return menu->mnemonic;
}

/// Get the actext (accelerator text) field from a menu.
const char *nvim_menu_get_actext(vimmenu_T *menu)
{
  return menu->actext;
}

/// Get the global State variable.
int nvim_menu_get_global_state(void)
{
  return State;
}

/// Get the VIsual_active global variable.
bool nvim_menu_get_visual_active(void)
{
  return VIsual_active;
}

/// Get the VIsual_select global variable.
bool nvim_menu_get_visual_select(void)
{
  return VIsual_select;
}

/// Get the finish_op global variable.
bool nvim_menu_get_finish_op(void)
{
  return finish_op;
}

/// Wrapper for utfc_ptr2len for Rust FFI.
int nvim_menu_utfc_ptr2len(const char *p)
{
  return utfc_ptr2len(p);
}

/// Get a string from the menu's strings array.
const char *nvim_menu_get_string(vimmenu_T *menu, int idx)
{
  if (idx < 0 || idx >= MENU_MODES) {
    return NULL;
  }
  return menu->strings[idx];
}

/// Get a noremap value from the menu's noremap array.
int nvim_menu_get_noremap(vimmenu_T *menu, int idx)
{
  if (idx < 0 || idx >= MENU_MODES) {
    return 0;
  }
  return menu->noremap[idx];
}

/// Get a silent flag from the menu's silent array.
bool nvim_menu_get_silent(vimmenu_T *menu, int idx)
{
  if (idx < 0 || idx >= MENU_MODES) {
    return false;
  }
  return menu->silent[idx];
}

/// Get the root_menu global.
vimmenu_T *nvim_menu_get_root_menu(void)
{
  return root_menu;
}

/// Get the got_int global.
int nvim_menu_get_got_int(void)
{
  return got_int;
}

// ============================================================================
// Phase 4: Mutation accessors
// ============================================================================

/// Set the modes field on a menu.
void nvim_menu_set_modes(vimmenu_T *m, int v)
{
  m->modes = v;
}

/// Set the enabled field on a menu.
void nvim_menu_set_enabled(vimmenu_T *m, int v)
{
  m->enabled = v;
}

/// Set the name field on a menu.
void nvim_menu_set_name(vimmenu_T *m, char *v)
{
  m->name = v;
}

/// Set the dname field on a menu.
void nvim_menu_set_dname(vimmenu_T *m, char *v)
{
  m->dname = v;
}

/// Set the en_name field on a menu.
void nvim_menu_set_en_name(vimmenu_T *m, char *v)
{
  m->en_name = v;
}

/// Set the en_dname field on a menu.
void nvim_menu_set_en_dname(vimmenu_T *m, char *v)
{
  m->en_dname = v;
}

/// Set the priority field on a menu.
void nvim_menu_set_priority(vimmenu_T *m, int v)
{
  m->priority = v;
}

/// Set the mnemonic field on a menu.
void nvim_menu_set_mnemonic(vimmenu_T *m, int v)
{
  m->mnemonic = v;
}

/// Set the actext field on a menu.
void nvim_menu_set_actext(vimmenu_T *m, char *v)
{
  m->actext = v;
}

/// Set the next field on a menu.
void nvim_menu_set_next(vimmenu_T *m, vimmenu_T *v)
{
  m->next = v;
}

/// Set the children field on a menu.
void nvim_menu_set_children(vimmenu_T *m, vimmenu_T *v)
{
  m->children = v;
}

/// Set the parent field on a menu.
void nvim_menu_set_parent(vimmenu_T *m, vimmenu_T *v)
{
  m->parent = v;
}

/// Set a string in the menu's strings array.
void nvim_menu_set_string(vimmenu_T *m, int idx, char *v)
{
  if (idx >= 0 && idx < MENU_MODES) {
    m->strings[idx] = v;
  }
}

/// Set a noremap value in the menu's noremap array.
void nvim_menu_set_noremap(vimmenu_T *m, int idx, int v)
{
  if (idx >= 0 && idx < MENU_MODES) {
    m->noremap[idx] = v;
  }
}

/// Set a silent flag in the menu's silent array.
void nvim_menu_set_silent(vimmenu_T *m, int idx, bool v)
{
  if (idx >= 0 && idx < MENU_MODES) {
    m->silent[idx] = v;
  }
}

/// Allocate a new vimmenu_T, zero-initialized.
vimmenu_T *nvim_menu_alloc(void)
{
  return xcalloc(1, sizeof(vimmenu_T));
}

/// Free a vimmenu_T struct.
void nvim_menu_free_struct(vimmenu_T *m)
{
  xfree(m);
}

/// Get a pointer to the root_menu pointer.
vimmenu_T **nvim_menu_root_menu_ptr(void)
{
  return &root_menu;
}

/// Get a pointer to a menu's children pointer.
vimmenu_T **nvim_menu_children_ptr(vimmenu_T *m)
{
  return &m->children;
}

/// Get a pointer to a menu's next pointer.
vimmenu_T **nvim_menu_next_ptr(vimmenu_T *m)
{
  return &m->next;
}

/// Read through a vimmenu_T** pointer.
vimmenu_T *nvim_menu_ptr_read(vimmenu_T **p)
{
  return *p;
}

/// Write through a vimmenu_T** pointer.
void nvim_menu_ptr_write(vimmenu_T **p, vimmenu_T *v)
{
  *p = v;
}

/// Lookup a menu name translation.
char *nvim_menutrans_lookup(char *name, int len)
{
  return menutrans_lookup(name, len);
}

/// Get the sys_menu global.
bool nvim_menu_get_sys_menu(void)
{
  return sys_menu;
}

/// Get menuarg->noremap[0] for use from Rust.
int nvim_menu_get_noremap_0(vimmenu_T *menuarg)
{
  return menuarg->noremap[0];
}

/// Get menuarg->silent[0] for use from Rust.
bool nvim_menu_get_silent_0(vimmenu_T *menuarg)
{
  return menuarg->silent[0];
}

// ============================================================================
// Phase 5: ExArg accessors and wrapper functions
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

/// Get p_cpo for use from Rust.
const char *nvim_menu_get_p_cpo(void)
{
  return p_cpo;
}

/// Get curbuf for use from Rust.
void *nvim_menu_get_curbuf(void)
{
  return curbuf;
}

/// Wrapper for apply_autocmds for use from Rust.
bool nvim_menu_apply_autocmds(int event, const char *pat, void *buf)
{
  return apply_autocmds(event, pat, NULL, false, buf);
}

/// Wrapper for pum_show_popupmenu for use from Rust.
void nvim_menu_pum_show_popupmenu(vimmenu_T *menu)
{
  pum_show_popupmenu(menu);
}

/// Wrapper to construct a menuarg struct and call add_menu_path from Rust.
void nvim_menu_call_add_menu_path(const char *menu_path, int modes, int noremap,
                                  bool silent, const int *pri_tab, const char *call_data)
{
  vimmenu_T menuarg;
  memset(&menuarg, 0, sizeof(menuarg));
  menuarg.modes = modes;
  menuarg.noremap[0] = noremap;
  menuarg.silent[0] = silent;
  add_menu_path(menu_path, &menuarg, pri_tab, call_data);
}
