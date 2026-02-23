// cmdexpand.c: functions for command-line completion

#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>

#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/arglist.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/cmdhist.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/funcs.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_getln.h"
#include "nvim/fuzzy.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/hashtab.h"
#include "nvim/hashtab_defs.h"
#include "nvim/help.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/insexpand.h"
#include "nvim/keycodes.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/mapping.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/menu.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/lang.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/popupmenu.h"
#include "nvim/profile.h"
#include "nvim/regexp.h"
#include "nvim/runtime.h"
#include "nvim/runtime_defs.h"
#include "nvim/search.h"
#include "nvim/sign.h"
#include "nvim/statusline.h"
#include "nvim/strings.h"
#include "nvim/syntax.h"
#include "nvim/tag.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/usercmd.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

/// Type used by call_user_expand_func
typedef void *(*user_expand_func_T)(const char *, int, typval_T *);

#include "cmdexpand.c.generated.h"

// Rust FFI declarations (tag module)
extern int rs_expand_tags(bool tagnames, char *pat, int *num_file, char ***file);

// Rust FFI declarations (option expand module)
extern int rs_expand_setting_subtract(void *xp, void *regmatch, int *numMatches, char ***matches);

// Rust FFI declarations (window wrappers removed)
extern int rs_global_stl_height(void);
extern void rs_last_status(int morewin);

static bool cmd_showtail;  ///< Only show path tail in lists ?
static bool may_expand_pattern = false;
static pos_T pre_incsearch_pos;  ///< Cursor position when incsearch started

/// "compl_match_array" points the currently displayed list of entries in the
/// popup menu.  It is NULL when there is no popup menu.
static pumitem_T *compl_match_array = NULL;
static int compl_match_arraysize;
/// First column in cmdline of the matched item for completion.
static int compl_startcol;
static int compl_selected;
/// cmdline before expansion
static char *cmdline_orig = NULL;

extern int rs_magic_isset(void);
extern int rs_cmdline_fuzzy_complete(const char *fuzzystr);
extern int rs_cmdline_pum_active(void);
extern int rs_cmdline_fuzzy_completion_supported(int context);

// Phase 6: Backslash constants from Rust
extern int rs_xp_bs_none(void);
extern int rs_xp_bs_one(void);
extern int rs_xp_bs_three(void);
extern int rs_xp_bs_comma(void);

// Phase 6: Wild mode constants from Rust
extern int rs_wild_free(void);
extern int rs_wild_expand_free(void);
extern int rs_wild_expand_keep(void);
extern int rs_wild_next(void);
extern int rs_wild_prev(void);
extern int rs_wild_all(void);
extern int rs_wild_longest(void);
extern int rs_wild_all_keep(void);
extern int rs_wild_cancel(void);
extern int rs_wild_apply(void);
extern int rs_wild_pageup(void);
extern int rs_wild_pagedown(void);
extern int rs_wild_pum_want(void);

// Phase 6: Wild options constants from Rust
extern int rs_wild_list_notfound(void);
extern int rs_wild_home_replace(void);
extern int rs_wild_use_nl(void);
extern int rs_wild_no_beep(void);
extern int rs_wild_add_slash(void);
extern int rs_wild_keep_all(void);
extern int rs_wild_silent(void);
extern int rs_wild_escape(void);
extern int rs_wild_icase(void);
extern int rs_wild_alllinks(void);
extern int rs_wild_ignore_completeslash(void);
extern int rs_wild_noerror(void);
extern int rs_wild_buflastused(void);
extern int rs_buf_diff_filter(void);
extern int rs_wild_noselect(void);
extern int rs_wild_may_expand_pattern(void);
extern int rs_wild_func_trigger(void);

// Phase 6: Expand context constants from Rust
extern int rs_expand_unsuccessful(void);
extern int rs_expand_ok(void);
extern int rs_expand_nothing(void);
extern int rs_expand_commands(void);
extern int rs_expand_files(void);
extern int rs_expand_directories(void);
extern int rs_expand_settings(void);
extern int rs_expand_buffers(void);
extern int rs_expand_help(void);
extern int rs_expand_functions(void);
extern int rs_expand_user_commands(void);
extern int rs_expand_lua(void);

// Phase 6: Validation functions from Rust
extern int rs_expand_context_valid(int context);
extern int rs_is_file_expand_context(int context);
extern int rs_is_user_expand_context(int context);
extern int rs_wild_mode_is_navigation(int mode);
extern int rs_wild_mode_needs_list(int mode);

// Phase 1: Leaf utility functions from Rust
extern int rs_sort_func_compare(const void *s1, const void *s2);
extern int rs_cmdline_compl_use_pum(int need_wildmenu);
extern int rs_map_wildopts_to_ewflags(int options);
extern char *rs_showmatches_gettail(char *s, int eager);
extern int rs_expand_showtail(expand_T *xp);
extern void rs_wildescape(expand_T *xp, const char *str, int numfiles, char **files);
extern void rs_expand_escape(expand_T *xp, char *str, int numfiles, char **files, int options);

// Phase 2: Expand struct operations
extern void rs_expand_init(expand_T *xp);
extern void rs_expand_cleanup(expand_T *xp);
extern void rs_clear_cmdline_orig(void);
extern char *rs_addstar(const char *fname, size_t len, int context);

// Phase 3: Match navigation
extern char *rs_get_next_or_prev_match(int mode, expand_T *xp);
extern char *rs_expand_one_start(int mode, expand_T *xp, const char *str, int options);
extern char *rs_find_longest_match(expand_T *xp, int options);

// Phase 4: ExpandOne orchestrator
extern char *rs_expand_one(expand_T *xp, char *str, char *orig, int options, int mode);

// Phase 5: Context-setting helpers
extern const char *rs_find_cmd_after_global_cmd(const char *arg);
extern const char *rs_find_cmd_after_substitute_cmd(const char *arg);
extern const char *rs_find_cmd_after_isearch_cmd(expand_T *xp, const char *arg);
extern const char *rs_set_context_in_argopt(expand_T *xp, const char *arg);
extern const char *rs_set_context_in_filter_cmd(expand_T *xp, const char *arg);
extern const char *rs_set_context_in_match_cmd(expand_T *xp, const char *arg);
extern const char *rs_set_context_in_unlet_cmd(expand_T *xp, const char *arg);
extern const char *rs_set_context_in_lang_cmd(expand_T *xp, const char *arg);
extern const char *rs_set_context_in_breakadd_cmd(expand_T *xp, const char *arg, int breakpt_cmd_type);
extern const char *rs_set_context_in_scriptnames_cmd(expand_T *xp, const char *arg);
extern const char *rs_set_context_in_filetype_cmd(expand_T *xp, const char *arg);
extern void rs_set_context_with_pattern(expand_T *xp);
extern void rs_set_context_for_wildcard_arg(const char *arg, int is_shell_cmd,
                                            expand_T *xp, int *complp);

// Phase 6: Callback generators
extern char *rs_get_filetypecmd_arg(expand_T *xp, int idx);
extern char *rs_get_breakadd_arg(expand_T *xp, int idx);
extern char *rs_get_retab_arg(expand_T *xp, int idx);
extern char *rs_get_messages_arg(expand_T *xp, int idx);
extern char *rs_get_mapclear_arg(expand_T *xp, int idx);

// Wave 4: Wildmenu display helpers
extern int rs_skip_wildmenu_char(expand_T *xp, const char *s);
extern int rs_wildmenu_match_len(expand_T *xp, char *s);

// Wave 4: Pattern-in-buffer helpers
extern int rs_is_regex_match(const char *pat, const char *str);
extern char *rs_concat_pattern_with_buffer_match(const char *pat, int pat_len,
                                                  const pos_T *end_match_pos, int lowercase);
extern int rs_copy_substring_from_pos(pos_T *start, pos_T *end, char **match, pos_T *match_end);

// C accessor for Rust FFI
unsigned nvim_get_wop_flags(void)
{
  return wop_flags;
}

/// C accessor for compl_match_array != NULL check.
int nvim_get_compl_match_array_not_null(void)
{
  return compl_match_array != NULL;
}

// =============================================================================
// C accessors for expand_T fields (Rust FFI)
// =============================================================================

/// Get the expansion context type.
int nvim_expand_get_context(const expand_T *xp)
{
  return xp ? xp->xp_context : EXPAND_NOTHING;
}

/// Get the expansion pattern pointer.
const char *nvim_expand_get_pattern(const expand_T *xp)
{
  return xp ? xp->xp_pattern : NULL;
}

/// Get the expansion pattern length.
size_t nvim_expand_get_pattern_len(const expand_T *xp)
{
  return xp ? xp->xp_pattern_len : 0;
}

/// Get the backslash flags.
int nvim_expand_get_backslash(const expand_T *xp)
{
  return xp ? xp->xp_backslash : XP_BS_NONE;
}

/// Get the number of files.
int nvim_expand_get_numfiles(const expand_T *xp)
{
  return xp ? xp->xp_numfiles : -1;
}

/// Get the selected index.
int nvim_expand_get_selected(const expand_T *xp)
{
  return xp ? xp->xp_selected : -1;
}

/// Get the column position.
int nvim_expand_get_col(const expand_T *xp)
{
  return xp ? xp->xp_col : 0;
}

/// Get the prefix type.
int nvim_expand_get_prefix(const expand_T *xp)
{
  return xp ? (int)xp->xp_prefix : 0;
}

/// Get the shell flag.
int nvim_expand_get_shell(const expand_T *xp)
{
#ifndef BACKSLASH_IN_FILENAME
  return xp ? xp->xp_shell : 0;
#else
  (void)xp;
  return 0;
#endif
}

/// Set the expansion context type.
void nvim_expand_set_context(expand_T *xp, int context)
{
  if (xp) {
    xp->xp_context = context;
  }
}

/// Set the backslash flags.
void nvim_expand_set_backslash(expand_T *xp, int backslash)
{
  if (xp) {
    xp->xp_backslash = backslash;
  }
}

/// Set the selected index.
void nvim_expand_set_selected(expand_T *xp, int selected)
{
  if (xp) {
    xp->xp_selected = selected;
  }
}

/// Check if cmdline_win is NULL (for Rust FFI).
int nvim_get_cmdline_win_is_null(void)
{
  return cmdline_win == NULL;
}

/// Check if xp->xp_orig is not NULL (for Rust FFI).
int nvim_expand_get_orig_not_null(const expand_T *xp)
{
  return xp && xp->xp_orig != NULL;
}

/// Get pum_want.active (for Rust FFI).
int nvim_get_pum_want_active(void)
{
  return pum_want.active;
}

/// Get pum_want.item (for Rust FFI).
int nvim_get_pum_want_item(void)
{
  return pum_want.item;
}

/// Get wild_menu_showing (for Rust FFI).
int nvim_get_wild_menu_showing(void)
{
  return wild_menu_showing;
}

// =============================================================================
// Phase 1: Static asserts for Rust-hardcoded constants
// =============================================================================

// EW flags (path.h) used in rs_map_wildopts_to_ewflags
_Static_assert(EW_DIR == 0x01, "EW_DIR mismatch");
_Static_assert(EW_FILE == 0x02, "EW_FILE mismatch");
_Static_assert(EW_NOTFOUND == 0x04, "EW_NOTFOUND mismatch");
_Static_assert(EW_ADDSLASH == 0x08, "EW_ADDSLASH mismatch");
_Static_assert(EW_KEEPALL == 0x10, "EW_KEEPALL mismatch");
_Static_assert(EW_SILENT == 0x20, "EW_SILENT mismatch");
_Static_assert(EW_NOERROR == 0x200, "EW_NOERROR mismatch");
_Static_assert(EW_ALLLINKS == 0x1000, "EW_ALLLINKS mismatch");

// VSE flags (ex_getln.h) used in rs_wildescape
_Static_assert(VSE_NONE == 0, "VSE_NONE mismatch");
_Static_assert(VSE_SHELL == 1, "VSE_SHELL mismatch");
_Static_assert(VSE_BUFFER == 2, "VSE_BUFFER mismatch");

// UI extension enum values (ui_defs.h) used in rs_cmdline_compl_use_pum
_Static_assert(kUICmdline == 0, "kUICmdline mismatch");
_Static_assert(kUIPopupmenu == 1, "kUIPopupmenu mismatch");
_Static_assert(kUIWildmenu == 3, "kUIWildmenu mismatch");

// wildoptions flag used in rs_cmdline_compl_use_pum
_Static_assert(kOptWopFlagPum == 0x04, "kOptWopFlagPum mismatch");

// Wave 4: Constants used in wildmenu and pattern-in-buffer helpers
_Static_assert(kOptWopFlagExacttext == 0x08, "kOptWopFlagExacttext mismatch");
_Static_assert(RE_MAGIC == 1, "RE_MAGIC mismatch");
_Static_assert(RE_STRING == 2, "RE_STRING mismatch");
_Static_assert(EXPAND_HELP == 8, "EXPAND_HELP mismatch");
_Static_assert(EXPAND_MENUS == 11, "EXPAND_MENUS mismatch");
_Static_assert(EXPAND_MENUNAMES == 21, "EXPAND_MENUNAMES mismatch");
_Static_assert(EXPAND_PATTERN_IN_BUF == 60, "EXPAND_PATTERN_IN_BUF mismatch");
_Static_assert(sizeof(pos_T) == 12, "pos_T size mismatch");
_Static_assert(offsetof(pos_T, lnum) == 0, "pos_T.lnum offset mismatch");
_Static_assert(offsetof(pos_T, col) == 4, "pos_T.col offset mismatch");
_Static_assert(offsetof(pos_T, coladd) == 8, "pos_T.coladd offset mismatch");

// =============================================================================
// Phase 1: C accessors for Rust leaf utility functions
// =============================================================================

/// Check if UI extension is active (for Rust FFI).
int nvim_cmdexpand_ui_has(int ext)
{
  return ui_has(ext);
}

/// Check if character is a path separator (for Rust FFI).
int nvim_cmdexpand_vim_ispathsep(int c)
{
  return vim_ispathsep(c);
}

/// Check if backslash should be removed (for Rust FFI).
int nvim_cmdexpand_rem_backslash(const char *p)
{
  return rem_backslash(p);
}

/// Get the byte length for multibyte pointer advance (for Rust FFI).
int nvim_cmdexpand_mb_ptr_adv_len(const char *p)
{
  return utfc_ptr2len(p);
}

/// Get display cell width of character at pointer (for Rust FFI).
int nvim_cmdexpand_ptr2cells(const char *p)
{
  return ptr2cells(p);
}

/// Check if string is a menu separator (for Rust FFI).
int nvim_cmdexpand_menu_is_separator(const char *s)
{
  return menu_is_separator((char *)s);
}

/// Get buffer line text (for Rust FFI).
const char *nvim_cmdexpand_ml_get(linenr_T lnum)
{
  return ml_get(lnum);
}

/// Get buffer line length (for Rust FFI).
int nvim_cmdexpand_ml_get_len(linenr_T lnum)
{
  return ml_get_len(lnum);
}

/// Increment msg_silent (for Rust FFI).
void nvim_cmdexpand_msg_silent_inc(void)
{
  msg_silent++;
}

/// Decrement msg_silent (for Rust FFI).
void nvim_cmdexpand_msg_silent_dec(void)
{
  msg_silent--;
}

/// Get p_ic (ignorecase option) value (for Rust FFI).
int nvim_cmdexpand_get_p_ic(void)
{
  return p_ic;
}

/// Get p_scs (smartcase option) value (for Rust FFI).
int nvim_cmdexpand_get_p_scs(void)
{
  return p_scs;
}

// =============================================================================
// Phase 2: C accessors for expand_T struct operations
// =============================================================================

/// Zero the entire expand_T struct (for Rust FFI).
void nvim_expand_clear(expand_T *xp)
{
  if (xp) {
    CLEAR_POINTER(xp);
  }
}

/// Set xp_prefix (for Rust FFI).
void nvim_expand_set_prefix(expand_T *xp, int prefix)
{
  if (xp) {
    xp->xp_prefix = (xp_prefix_T)prefix;
  }
}

/// Set xp_numfiles (for Rust FFI).
void nvim_expand_set_numfiles(expand_T *xp, int numfiles)
{
  if (xp) {
    xp->xp_numfiles = numfiles;
  }
}

/// Free wild matches via FreeWild (for Rust FFI).
void nvim_expand_free_wild(expand_T *xp)
{
  if (xp) {
    FreeWild(xp->xp_numfiles, xp->xp_files);
  }
}

/// Free and NULL xp_orig (for Rust FFI).
void nvim_expand_clear_orig(expand_T *xp)
{
  if (xp) {
    XFREE_CLEAR(xp->xp_orig);
  }
}

/// Free and NULL the static cmdline_orig (for Rust FFI).
void nvim_clear_cmdline_orig(void)
{
  XFREE_CLEAR(cmdline_orig);
}

// Static assert for XP_PREFIX_NONE used in rs_expand_init
_Static_assert(XP_PREFIX_NONE == 0, "XP_PREFIX_NONE mismatch");

// =============================================================================
// Phase 3: C accessors for match navigation
// =============================================================================

/// Get xp->xp_orig pointer (for Rust FFI).
const char *nvim_expand_get_orig(const expand_T *xp)
{
  return xp ? xp->xp_orig : NULL;
}

/// Get xp->xp_files[i] (for Rust FFI).
const char *nvim_expand_get_files_item(const expand_T *xp, int i)
{
  if (!xp || i < 0 || i >= xp->xp_numfiles || !xp->xp_files) {
    return NULL;
  }
  return xp->xp_files[i];
}

/// Get compl_selected (for Rust FFI).
int nvim_get_compl_selected(void)
{
  return compl_selected;
}

/// Set compl_selected (for Rust FFI).
void nvim_set_compl_selected(int val)
{
  compl_selected = val;
}

/// Get cmd_showtail (for Rust FFI).
int nvim_get_cmd_showtail(void)
{
  return cmd_showtail;
}

/// Get p_wmnu (for Rust FFI).
int nvim_get_p_wmnu(void)
{
  return p_wmnu;
}

/// Wrapper for cmdline_pum_display (for Rust FFI).
void nvim_cmdexpand_pum_display(int changed_array)
{
  cmdline_pum_display(changed_array != 0);
}

/// Wrapper for cmdline_pum_create for navigation (for Rust FFI).
/// Creates PUM with xp->xp_files/xp_numfiles and given showtail/noselect flags.
void nvim_cmdexpand_pum_create_for_nav(expand_T *xp, int showtail, int noselect)
{
  cmdline_pum_create(get_cmdline_info(), xp, xp->xp_files, xp->xp_numfiles,
                     showtail != 0, noselect != 0);
}

/// Wrapper for redraw_wildmenu (for Rust FFI).
void nvim_cmdexpand_redraw_wildmenu(expand_T *xp, int num_matches, int findex, int showtail)
{
  redraw_wildmenu(xp, num_matches, xp->xp_files, findex, showtail != 0);
}

/// Wrapper for ExpandFromContext (for Rust FFI).
/// Calls ExpandFromContext and stores results into xp->xp_files/xp_numfiles.
/// Returns FAIL (0) or OK (1).
int nvim_cmdexpand_expand_from_context(expand_T *xp, const char *pat, int options)
{
  return ExpandFromContext(xp, (char *)pat, &xp->xp_files, &xp->xp_numfiles, options);
}

/// Wrapper for ExpandEscape (for Rust FFI).
void nvim_cmdexpand_expand_escape(expand_T *xp, const char *str, int options)
{
  ExpandEscape(xp, (char *)str, xp->xp_numfiles, xp->xp_files, options);
}

/// Wrapper for match_suffix on xp->xp_files[i] (for Rust FFI).
int nvim_cmdexpand_match_suffix(expand_T *xp, int i)
{
  if (!xp || i < 0 || i >= xp->xp_numfiles || !xp->xp_files) {
    return 0;
  }
  return match_suffix(xp->xp_files[i]);
}

/// Wrapper for semsg(e_nomatch2, str) (for Rust FFI).
void nvim_cmdexpand_semsg_nomatch(const char *str)
{
  semsg(_(e_nomatch2), str);
}

/// Wrapper for emsg(e_toomany) (for Rust FFI).
void nvim_cmdexpand_emsg_toomany(void)
{
  emsg(_(e_toomany));
}

// Static assert for kOptBoFlagWildmode used in rs_find_longest_match
_Static_assert(kOptBoFlagWildmode == 0x80000, "kOptBoFlagWildmode mismatch");

// =============================================================================
// Phase 4: C accessors for ExpandOne orchestrator
// =============================================================================

/// Set xp->xp_orig (for Rust FFI). Takes ownership of the pointer.
void nvim_expand_set_orig(expand_T *xp, char *orig)
{
  if (xp) {
    xp->xp_orig = orig;
  }
}

/// Free old wild matches, set numfiles=-1, clear orig, remove PUM if needed.
/// Used in ExpandOne before starting a new expansion.
void nvim_expand_free_old_matches(expand_T *xp)
{
  if (!xp) {
    return;
  }
  if (xp->xp_numfiles != -1) {
    FreeWild(xp->xp_numfiles, xp->xp_files);
    xp->xp_numfiles = -1;
    XFREE_CLEAR(xp->xp_orig);

    if (compl_match_array != NULL) {
      cmdline_pum_remove(false);
    }
  }
}

// nvim_get_got_int already exists in ex_eval.c

/// Get strlen of xp->xp_files[i] (for Rust FFI).
size_t nvim_expand_get_files_item_len(const expand_T *xp, int i)
{
  if (!xp || i < 0 || i >= xp->xp_numfiles || !xp->xp_files) {
    return 0;
  }
  return strlen(xp->xp_files[i]);
}

/// xstpcpy wrapper (for Rust FFI): copies src to dst, returns pointer past NUL.
char *nvim_cmdexpand_xstpcpy(char *dst, const char *src)
{
  return xstpcpy(dst, src);
}

// Static asserts for XP_PREFIX values used in rs_expand_one
_Static_assert(XP_PREFIX_NO == 1, "XP_PREFIX_NO mismatch");
_Static_assert(XP_PREFIX_INV == 2, "XP_PREFIX_INV mismatch");

// =============================================================================
// Phase 5: Static enums and constants (moved here for accessor visibility)
// =============================================================================

static enum {
  EXP_FILETYPECMD_ALL,     ///< expand all :filetype values
  EXP_FILETYPECMD_PLUGIN,  ///< expand plugin on off
  EXP_FILETYPECMD_INDENT,  ///< expand indent on off
  EXP_FILETYPECMD_ONOFF,   ///< expand on off
} filetype_expand_what;

enum {
  EXPAND_FILETYPECMD_PLUGIN = 0x01,
  EXPAND_FILETYPECMD_INDENT = 0x02,
  EXPAND_FILETYPECMD_ONOFF  = 0x04,
};

static enum {
  EXP_BREAKPT_ADD,  ///< expand ":breakadd" sub-commands
  EXP_BREAKPT_DEL,  ///< expand ":breakdel" sub-commands
  EXP_PROFDEL,      ///< expand ":profdel" sub-commands
} breakpt_expand_what;

// =============================================================================
// Phase 5: C accessors for context-setting helpers
// =============================================================================

/// Set xp->xp_pattern (for Rust FFI).
void nvim_expand_set_pattern(expand_T *xp, char *pattern)
{
  if (xp) {
    xp->xp_pattern = pattern;
  }
}

/// Set xp->xp_pattern_len (for Rust FFI).
void nvim_expand_set_pattern_len(expand_T *xp, size_t len)
{
  if (xp) {
    xp->xp_pattern_len = len;
  }
}

/// Set xp->xp_search_dir (for Rust FFI).
void nvim_expand_set_search_dir(expand_T *xp, int dir)
{
  if (xp) {
    xp->xp_search_dir = (Direction)dir;
  }
}

/// Set xp->xp_shell (for Rust FFI).
void nvim_expand_set_shell(expand_T *xp, int shell)
{
  if (xp) {
    xp->xp_shell = shell != 0;
  }
}

/// Get xp->xp_pattern (for Rust FFI).
char *nvim_cmdexpand_get_xp_pattern(expand_T *xp)
{
  return xp ? xp->xp_pattern : NULL;
}

/// Wrapper for ascii_iswhite (for Rust FFI).
int nvim_cmdexpand_ascii_iswhite(int c)
{
  return ascii_iswhite(c);
}

/// Wrapper for vim_isfilec_or_wc (for Rust FFI).
int nvim_cmdexpand_vim_isfilec_or_wc(int c)
{
  return vim_isfilec_or_wc(c);
}

/// Wrapper for vim_isIDc (for Rust FFI).
int nvim_cmdexpand_vim_isIDc(int c)
{
  return vim_isIDc((uint8_t)c);
}

/// Get ccline->cmdpos (for Rust FFI).
int nvim_cmdexpand_get_cmdpos(void)
{
  return get_cmdline_info()->cmdpos;
}

/// Get ccline->cmdbuff (for Rust FFI).
char *nvim_cmdexpand_get_cmdbuff(void)
{
  return get_cmdline_info()->cmdbuff;
}

/// Wrapper for parse_pattern_and_range (for Rust FFI).
/// Returns OK(1)/FAIL(0). Writes skiplen and patlen via out pointers.
int nvim_cmdexpand_parse_pattern_and_range(int *skiplen, int *patlen)
{
  int dummy;
  return parse_pattern_and_range(&pre_incsearch_pos, &dummy, skiplen, patlen);
}

/// Increment emsg_off (for Rust FFI).
void nvim_cmdexpand_emsg_off_inc(void)
{
  emsg_off++;
}

/// Decrement emsg_off (for Rust FFI).
void nvim_cmdexpand_emsg_off_dec(void)
{
  emsg_off--;
}

/// Set the static breakpt_expand_what variable (for Rust FFI).
void nvim_cmdexpand_set_breakpt_expand_what(int val)
{
  breakpt_expand_what = val;
}

/// Set the static filetype_expand_what variable (for Rust FFI).
void nvim_cmdexpand_set_filetype_expand_what(int val)
{
  filetype_expand_what = val;
}

/// Get the static breakpt_expand_what variable (for Rust FFI).
int nvim_cmdexpand_get_breakpt_expand_what(void)
{
  return (int)breakpt_expand_what;
}

/// Get the static filetype_expand_what variable (for Rust FFI).
int nvim_cmdexpand_get_filetype_expand_what(void)
{
  return (int)filetype_expand_what;
}

// Static asserts for Phase 5 constants
_Static_assert(FORWARD == 1, "FORWARD mismatch");
_Static_assert(EXP_BREAKPT_ADD == 0, "EXP_BREAKPT_ADD mismatch");
_Static_assert(EXP_BREAKPT_DEL == 1, "EXP_BREAKPT_DEL mismatch");
_Static_assert(EXP_PROFDEL == 2, "EXP_PROFDEL mismatch");
_Static_assert(EXP_FILETYPECMD_ALL == 0, "EXP_FILETYPECMD_ALL mismatch");
_Static_assert(EXP_FILETYPECMD_PLUGIN == 1, "EXP_FILETYPECMD_PLUGIN mismatch");
_Static_assert(EXP_FILETYPECMD_INDENT == 2, "EXP_FILETYPECMD_INDENT mismatch");
_Static_assert(EXP_FILETYPECMD_ONOFF == 3, "EXP_FILETYPECMD_ONOFF mismatch");
_Static_assert(EXPAND_FILETYPECMD_PLUGIN == 0x01, "EXPAND_FILETYPECMD_PLUGIN mismatch");
_Static_assert(EXPAND_FILETYPECMD_INDENT == 0x02, "EXPAND_FILETYPECMD_INDENT mismatch");

// Phase 6: Wrapper functions for Rust implementations

/// Get XP_BS_NONE constant. Rust implementation.
static int xp_const_bs_none(void)
  FUNC_ATTR_PURE
{
  return rs_xp_bs_none();
}

/// Get XP_BS_ONE constant. Rust implementation.
static int xp_const_bs_one(void)
  FUNC_ATTR_PURE
{
  return rs_xp_bs_one();
}

/// Get XP_BS_THREE constant. Rust implementation.
static int xp_const_bs_three(void)
  FUNC_ATTR_PURE
{
  return rs_xp_bs_three();
}

/// Get XP_BS_COMMA constant. Rust implementation.
static int xp_const_bs_comma(void)
  FUNC_ATTR_PURE
{
  return rs_xp_bs_comma();
}

/// Get WILD_FREE constant. Rust implementation.
static int wild_const_free(void)
  FUNC_ATTR_PURE
{
  return rs_wild_free();
}

/// Get WILD_EXPAND_FREE constant. Rust implementation.
static int wild_const_expand_free(void)
  FUNC_ATTR_PURE
{
  return rs_wild_expand_free();
}

/// Get WILD_EXPAND_KEEP constant. Rust implementation.
static int wild_const_expand_keep(void)
  FUNC_ATTR_PURE
{
  return rs_wild_expand_keep();
}

/// Get WILD_NEXT constant. Rust implementation.
static int wild_const_next(void)
  FUNC_ATTR_PURE
{
  return rs_wild_next();
}

/// Get WILD_PREV constant. Rust implementation.
static int wild_const_prev(void)
  FUNC_ATTR_PURE
{
  return rs_wild_prev();
}

/// Get WILD_ALL constant. Rust implementation.
static int wild_const_all(void)
  FUNC_ATTR_PURE
{
  return rs_wild_all();
}

/// Get WILD_LONGEST constant. Rust implementation.
static int wild_const_longest(void)
  FUNC_ATTR_PURE
{
  return rs_wild_longest();
}

/// Get WILD_ALL_KEEP constant. Rust implementation.
static int wild_const_all_keep(void)
  FUNC_ATTR_PURE
{
  return rs_wild_all_keep();
}

/// Get WILD_CANCEL constant. Rust implementation.
static int wild_const_cancel(void)
  FUNC_ATTR_PURE
{
  return rs_wild_cancel();
}

/// Get WILD_APPLY constant. Rust implementation.
static int wild_const_apply(void)
  FUNC_ATTR_PURE
{
  return rs_wild_apply();
}

/// Get WILD_PAGEUP constant. Rust implementation.
static int wild_const_pageup(void)
  FUNC_ATTR_PURE
{
  return rs_wild_pageup();
}

/// Get WILD_PAGEDOWN constant. Rust implementation.
static int wild_const_pagedown(void)
  FUNC_ATTR_PURE
{
  return rs_wild_pagedown();
}

/// Get WILD_PUM_WANT constant. Rust implementation.
static int wild_const_pum_want(void)
  FUNC_ATTR_PURE
{
  return rs_wild_pum_want();
}

/// Get WILD_LIST_NOTFOUND constant. Rust implementation.
static int wild_opt_list_notfound(void)
  FUNC_ATTR_PURE
{
  return rs_wild_list_notfound();
}

/// Get WILD_HOME_REPLACE constant. Rust implementation.
static int wild_opt_home_replace(void)
  FUNC_ATTR_PURE
{
  return rs_wild_home_replace();
}

/// Get WILD_USE_NL constant. Rust implementation.
static int wild_opt_use_nl(void)
  FUNC_ATTR_PURE
{
  return rs_wild_use_nl();
}

/// Get WILD_NO_BEEP constant. Rust implementation.
static int wild_opt_no_beep(void)
  FUNC_ATTR_PURE
{
  return rs_wild_no_beep();
}

/// Get WILD_ADD_SLASH constant. Rust implementation.
static int wild_opt_add_slash(void)
  FUNC_ATTR_PURE
{
  return rs_wild_add_slash();
}

/// Get WILD_KEEP_ALL constant. Rust implementation.
static int wild_opt_keep_all(void)
  FUNC_ATTR_PURE
{
  return rs_wild_keep_all();
}

/// Get WILD_SILENT constant. Rust implementation.
static int wild_opt_silent(void)
  FUNC_ATTR_PURE
{
  return rs_wild_silent();
}

/// Get WILD_ESCAPE constant. Rust implementation.
static int wild_opt_escape(void)
  FUNC_ATTR_PURE
{
  return rs_wild_escape();
}

/// Get WILD_ICASE constant. Rust implementation.
static int wild_opt_icase(void)
  FUNC_ATTR_PURE
{
  return rs_wild_icase();
}

/// Get WILD_ALLLINKS constant. Rust implementation.
static int wild_opt_alllinks(void)
  FUNC_ATTR_PURE
{
  return rs_wild_alllinks();
}

/// Get WILD_IGNORE_COMPLETESLASH constant. Rust implementation.
static int wild_opt_ignore_completeslash(void)
  FUNC_ATTR_PURE
{
  return rs_wild_ignore_completeslash();
}

/// Get WILD_NOERROR constant. Rust implementation.
static int wild_opt_noerror(void)
  FUNC_ATTR_PURE
{
  return rs_wild_noerror();
}

/// Get WILD_BUFLASTUSED constant. Rust implementation.
static int wild_opt_buflastused(void)
  FUNC_ATTR_PURE
{
  return rs_wild_buflastused();
}

/// Get BUF_DIFF_FILTER constant. Rust implementation.
static int wild_opt_buf_diff_filter(void)
  FUNC_ATTR_PURE
{
  return rs_buf_diff_filter();
}

/// Get WILD_NOSELECT constant. Rust implementation.
static int wild_opt_noselect(void)
  FUNC_ATTR_PURE
{
  return rs_wild_noselect();
}

/// Get WILD_MAY_EXPAND_PATTERN constant. Rust implementation.
static int wild_opt_may_expand_pattern(void)
  FUNC_ATTR_PURE
{
  return rs_wild_may_expand_pattern();
}

/// Get WILD_FUNC_TRIGGER constant. Rust implementation.
static int wild_opt_func_trigger(void)
  FUNC_ATTR_PURE
{
  return rs_wild_func_trigger();
}

/// Get EXPAND_UNSUCCESSFUL constant. Rust implementation.
static int expand_const_unsuccessful(void)
  FUNC_ATTR_PURE
{
  return rs_expand_unsuccessful();
}

/// Get EXPAND_OK constant. Rust implementation.
static int expand_const_ok(void)
  FUNC_ATTR_PURE
{
  return rs_expand_ok();
}

/// Get EXPAND_NOTHING constant. Rust implementation.
static int expand_const_nothing(void)
  FUNC_ATTR_PURE
{
  return rs_expand_nothing();
}

/// Get EXPAND_COMMANDS constant. Rust implementation.
static int expand_const_commands(void)
  FUNC_ATTR_PURE
{
  return rs_expand_commands();
}

/// Get EXPAND_FILES constant. Rust implementation.
static int expand_const_files(void)
  FUNC_ATTR_PURE
{
  return rs_expand_files();
}

/// Get EXPAND_DIRECTORIES constant. Rust implementation.
static int expand_const_directories(void)
  FUNC_ATTR_PURE
{
  return rs_expand_directories();
}

/// Get EXPAND_SETTINGS constant. Rust implementation.
static int expand_const_settings(void)
  FUNC_ATTR_PURE
{
  return rs_expand_settings();
}

/// Get EXPAND_BUFFERS constant. Rust implementation.
static int expand_const_buffers(void)
  FUNC_ATTR_PURE
{
  return rs_expand_buffers();
}

/// Get EXPAND_HELP constant. Rust implementation.
static int expand_const_help(void)
  FUNC_ATTR_PURE
{
  return rs_expand_help();
}

/// Get EXPAND_FUNCTIONS constant. Rust implementation.
static int expand_const_functions(void)
  FUNC_ATTR_PURE
{
  return rs_expand_functions();
}

/// Get EXPAND_USER_COMMANDS constant. Rust implementation.
static int expand_const_user_commands(void)
  FUNC_ATTR_PURE
{
  return rs_expand_user_commands();
}

/// Get EXPAND_LUA constant. Rust implementation.
static int expand_const_lua(void)
  FUNC_ATTR_PURE
{
  return rs_expand_lua();
}

/// Check if expand context is valid. Rust implementation.
static bool expand_context_is_valid(int context)
  FUNC_ATTR_PURE
{
  return rs_expand_context_valid(context) != 0;
}

/// Check if context is file expansion. Rust implementation.
static bool expand_is_file_context(int context)
  FUNC_ATTR_PURE
{
  return rs_is_file_expand_context(context) != 0;
}

/// Check if context is user expansion. Rust implementation.
static bool expand_is_user_context(int context)
  FUNC_ATTR_PURE
{
  return rs_is_user_expand_context(context) != 0;
}

/// Check if wild mode is navigation. Rust implementation.
static bool wild_mode_is_nav(int mode)
  FUNC_ATTR_PURE
{
  return rs_wild_mode_is_navigation(mode) != 0;
}

/// Check if wild mode needs list. Rust implementation.
static bool wild_mode_wants_list(int mode)
  FUNC_ATTR_PURE
{
  return rs_wild_mode_needs_list(mode) != 0;
}

/// Check if fuzzy completion is supported for context. Rust implementation.
static bool fuzzy_complete_ctx_supported(int context)
  FUNC_ATTR_PURE
{
  return rs_cmdline_fuzzy_completion_supported(context) != 0;
}

#define SHOW_MATCH(m) (showtail ? showmatches_gettail(matches[m], false) : matches[m])

/// Returns true if fuzzy completion is supported for a given cmdline completion
/// context.
static bool cmdline_fuzzy_completion_supported(const expand_T *const xp)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL FUNC_ATTR_PURE
{
  switch (xp->xp_context) {
  case EXPAND_BOOL_SETTINGS:
  case EXPAND_COLORS:
  case EXPAND_COMPILER:
  case EXPAND_DIRECTORIES:
  case EXPAND_DIRS_IN_CDPATH:
  case EXPAND_FILES:
  case EXPAND_FILES_IN_PATH:
  case EXPAND_FILETYPE:
  case EXPAND_FILETYPECMD:
  case EXPAND_FINDFUNC:
  case EXPAND_HELP:
  case EXPAND_KEYMAP:
  case EXPAND_LUA:
  case EXPAND_OLD_SETTING:
  case EXPAND_STRING_SETTING:
  case EXPAND_SETTING_SUBTRACT:
  case EXPAND_OWNSYNTAX:
  case EXPAND_PACKADD:
  case EXPAND_RUNTIME:
  case EXPAND_SHELLCMD:
  case EXPAND_SHELLCMDLINE:
  case EXPAND_TAGS:
  case EXPAND_TAGS_LISTFILES:
  case EXPAND_USER_LIST:
  case EXPAND_USER_LUA:
    return false;

  default:
    break;
  }

  return wop_flags & kOptWopFlagFuzzy;
}

/// Returns true if fuzzy completion for cmdline completion is enabled and
/// "fuzzystr" is not empty.  If search pattern is empty, then don't use fuzzy
/// matching.
bool cmdline_fuzzy_complete(const char *const fuzzystr)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL FUNC_ATTR_PURE
{
  return rs_cmdline_fuzzy_complete(fuzzystr) != 0;
}

/// Sort function for the completion matches.
/// <SNR> functions should be sorted to the end. Rust implementation.
static int sort_func_compare(const void *s1, const void *s2)
{
  return rs_sort_func_compare(s1, s2);
}

/// Escape special characters in the cmdline completion matches. Rust implementation.
static void wildescape(expand_T *xp, const char *str, int numfiles, char **files)
{
  rs_wildescape(xp, str, numfiles, files);
}

/// Escape special characters in the cmdline completion matches. Rust implementation.
static void ExpandEscape(expand_T *xp, char *str, int numfiles, char **files, int options)
{
  rs_expand_escape(xp, str, numfiles, files, options);
}

/// Return FAIL if this is not an appropriate context in which to do
/// completion of anything, return OK if it is (even if there are no matches).
/// For the caller, this means that the character is just passed through like a
/// normal character (instead of being expanded).  This allows :s/^I^D etc.
///
/// @param options  extra options for ExpandOne()
/// @param escape  if true, escape the returned matches
int nextwild(expand_T *xp, int type, int options, bool escape)
{
  CmdlineInfo *const ccline = get_cmdline_info();
  char *p;
  bool from_wildtrigger_func = options & WILD_FUNC_TRIGGER;
  bool wild_navigate = (type == WILD_NEXT || type == WILD_PREV
                        || type == WILD_PAGEUP || type == WILD_PAGEDOWN
                        || type == WILD_PUM_WANT);

  if (xp->xp_numfiles == -1) {
    pre_incsearch_pos = xp->xp_pre_incsearch_pos;
    if (ccline->input_fn && ccline->xp_context == EXPAND_COMMANDS) {
      // Expand commands typed in input() function
      set_cmd_context(xp, ccline->cmdbuff, ccline->cmdlen, ccline->cmdpos, false);
    } else {
      may_expand_pattern = options & WILD_MAY_EXPAND_PATTERN;
      set_expand_context(xp);
      may_expand_pattern = false;
    }
    if (xp->xp_context == EXPAND_LUA) {
      nlua_expand_pat(xp);
    }
    cmd_showtail = expand_showtail(xp);
  }

  if (xp->xp_context == EXPAND_UNSUCCESSFUL) {
    beep_flush();
    return OK;      // Something illegal on command line
  }
  if (xp->xp_context == EXPAND_NOTHING) {
    // Caller can use the character as a normal char instead
    return FAIL;
  }

  int i = (int)(xp->xp_pattern - ccline->cmdbuff);
  assert(ccline->cmdpos >= i);
  xp->xp_pattern_len = (size_t)ccline->cmdpos - (size_t)i;

  // Skip showing matches if prefix is invalid during wildtrigger()
  if (from_wildtrigger_func && xp->xp_context == EXPAND_COMMANDS
      && xp->xp_pattern_len == 0) {
    return FAIL;
  }

  // If cmd_silent is set then don't show the dots, because redrawcmd() below
  // won't remove them.
  if (!cmd_silent && !from_wildtrigger_func && !wild_navigate
      && !(ui_has(kUICmdline) || ui_has(kUIWildmenu))) {
    msg_puts("...");  // show that we are busy
    ui_flush();
  }

  if (wild_navigate) {
    // Get next/previous match for a previous expanded pattern.
    p = ExpandOne(xp, NULL, NULL, 0, type);
  } else {
    char *tmp;
    if (cmdline_fuzzy_completion_supported(xp)
        || xp->xp_context == EXPAND_PATTERN_IN_BUF) {
      // Don't modify the search string
      tmp = xstrnsave(xp->xp_pattern, xp->xp_pattern_len);
    } else {
      tmp = addstar(xp->xp_pattern, xp->xp_pattern_len, xp->xp_context);
    }
    // Translate string into pattern and expand it.
    const int use_options = (options
                             | WILD_HOME_REPLACE
                             | WILD_ADD_SLASH
                             | WILD_SILENT
                             | (escape ? WILD_ESCAPE : 0)
                             | (p_wic ? WILD_ICASE : 0));
    p = ExpandOne(xp, tmp, xstrnsave(&ccline->cmdbuff[i], xp->xp_pattern_len),
                  use_options, type);
    xfree(tmp);
    // Longest match: make sure it is not shorter, happens with :help.
    if (p != NULL && type == WILD_LONGEST) {
      int j;
      for (j = 0; (size_t)j < xp->xp_pattern_len; j++) {
        char c = ccline->cmdbuff[i + j];
        if (c == '*' || c == '?') {
          break;
        }
      }
      if ((int)strlen(p) < j) {
        XFREE_CLEAR(p);
      }
    }
  }

  // Save cmdline before inserting selected item
  if (!wild_navigate && ccline->cmdbuff != NULL) {
    xfree(cmdline_orig);
    cmdline_orig = xstrnsave(ccline->cmdbuff, (size_t)ccline->cmdlen);
  }

  if (p != NULL && !got_int && !(options & WILD_NOSELECT)) {
    size_t plen = strlen(p);
    int difflen = (int)plen - (int)(xp->xp_pattern_len);
    if (ccline->cmdlen + difflen + 4 > ccline->cmdbufflen) {
      realloc_cmdbuff(ccline->cmdlen + difflen + 4);
      xp->xp_pattern = ccline->cmdbuff + i;
    }
    assert(ccline->cmdpos <= ccline->cmdlen);
    memmove(&ccline->cmdbuff[ccline->cmdpos + difflen],
            &ccline->cmdbuff[ccline->cmdpos],
            (size_t)ccline->cmdlen - (size_t)ccline->cmdpos + 1);
    memmove(&ccline->cmdbuff[i], p, plen);
    ccline->cmdlen += difflen;
    ccline->cmdpos += difflen;
  }

  redrawcmd();
  cursorcmd();

  // When expanding a ":map" command and no matches are found, assume that
  // the key is supposed to be inserted literally
  if (xp->xp_context == EXPAND_MAPPINGS && p == NULL) {
    return FAIL;
  }

  if (xp->xp_numfiles <= 0 && p == NULL) {
    beep_flush();
  } else if (xp->xp_numfiles == 1 && !(options & WILD_NOSELECT) && !wild_navigate) {
    // free expanded pattern
    ExpandOne(xp, NULL, NULL, 0, WILD_FREE);
  }

  xfree(p);

  return OK;
}

/// Create completion popup menu with items from "matches".
static void cmdline_pum_create(CmdlineInfo *ccline, expand_T *xp, char **matches, int numMatches,
                               bool showtail, bool noselect)
{
  assert(numMatches >= 0);
  // Add all the completion matches
  compl_match_array = xmalloc(sizeof(pumitem_T) * (size_t)numMatches);
  compl_match_arraysize = numMatches;
  for (int i = 0; i < numMatches; i++) {
    compl_match_array[i] = (pumitem_T){
      .pum_text = SHOW_MATCH(i),
      .pum_info = NULL,
      .pum_extra = NULL,
      .pum_kind = NULL,
      .pum_user_abbr_hlattr = -1,
      .pum_user_kind_hlattr = -1,
    };
  }

  // Compute the popup menu starting column
  char *endpos = showtail ? showmatches_gettail(xp->xp_pattern, noselect) : xp->xp_pattern;
  if (ui_has(kUICmdline) && cmdline_win == NULL) {
    compl_startcol = (int)(endpos - ccline->cmdbuff);
  } else {
    compl_startcol = cmd_screencol((int)(endpos - ccline->cmdbuff));
  }
}

void cmdline_pum_display(bool changed_array)
{
  pum_display(compl_match_array, compl_match_arraysize, compl_selected,
              changed_array, compl_startcol);
}

/// Returns true if the cmdline completion popup menu is being displayed.
bool cmdline_pum_active(void)
{
  return rs_cmdline_pum_active() != 0;
}

/// Remove the cmdline completion popup menu (if present), free the list of items.
void cmdline_pum_remove(bool defer_redraw)
{
  pum_undisplay(!defer_redraw);
  XFREE_CLEAR(compl_match_array);
  compl_match_arraysize = 0;
}

void cmdline_pum_cleanup(CmdlineInfo *cclp)
{
  cmdline_pum_remove(false);
  wildmenu_cleanup(cclp);
}

/// Returns the current cmdline completion pattern.
char *cmdline_compl_pattern(void)
{
  expand_T *xp = get_cmdline_info()->xpc;
  return xp == NULL ? NULL : xp->xp_orig;
}

/// Returns true if fuzzy cmdline completion is active, false otherwise.
bool cmdline_compl_is_fuzzy(void)
{
  expand_T *xp = get_cmdline_info()->xpc;
  return xp != NULL && cmdline_fuzzy_completion_supported(xp);
}

/// Checks whether popup menu should be used for cmdline completion wildmenu.
/// Rust implementation.
///
/// @param wildmenu  whether wildmenu is needed by current 'wildmode' part
static bool cmdline_compl_use_pum(bool need_wildmenu)
{
  return rs_cmdline_compl_use_pum(need_wildmenu) != 0;
}

/// Return the number of characters that should be skipped in the wildmenu
/// These are backslashes used for escaping.  Do show backslashes in help tags
/// and in search pattern completion matches.
/// Rust implementation.
static int skip_wildmenu_char(expand_T *xp, char *s)
{
  return rs_skip_wildmenu_char(xp, s);
}

/// Get the length of an item as it will be shown in the status line.
/// Rust implementation.
static int wildmenu_match_len(expand_T *xp, char *s)
{
  return rs_wildmenu_match_len(xp, s);
}

/// Show wildchar matches in the status line.
/// Show at least the "match" item.
/// We start at item "first_match" in the list and show all matches that fit.
///
/// If inversion is possible we use it. Else '=' characters are used.
///
/// @param matches  list of matches
static void redraw_wildmenu(expand_T *xp, int num_matches, char **matches, int match, bool showtail)
{
  bool highlight = true;
  char *selstart = NULL;
  int selstart_col = 0;
  char *selend = NULL;
  static int first_match = 0;
  bool add_left = false;
  int i, l;

  if (matches == NULL) {        // interrupted completion?
    return;
  }

  char *buf = xmalloc((size_t)Columns * MB_MAXBYTES + 1);

  if (match == -1) {    // don't show match but original text
    match = 0;
    highlight = false;
  }
  // count 1 for the ending ">"
  int clen = wildmenu_match_len(xp, SHOW_MATCH(match)) + 3;  // length in screen cells
  if (match == 0) {
    first_match = 0;
  } else if (match < first_match) {
    // jumping left, as far as we can go
    first_match = match;
    add_left = true;
  } else {
    // check if match fits on the screen
    for (i = first_match; i < match; i++) {
      clen += wildmenu_match_len(xp, SHOW_MATCH(i)) + 2;
    }
    if (first_match > 0) {
      clen += 2;
    }
    // jumping right, put match at the left
    if (clen > Columns) {
      first_match = match;
      // if showing the last match, we can add some on the left
      clen = 2;
      for (i = match; i < num_matches; i++) {
        clen += wildmenu_match_len(xp, SHOW_MATCH(i)) + 2;
        if (clen >= Columns) {
          break;
        }
      }
      if (i == num_matches) {
        add_left = true;
      }
    }
  }
  if (add_left) {
    while (first_match > 0) {
      clen += wildmenu_match_len(xp, SHOW_MATCH(first_match - 1)) + 2;
      if (clen >= Columns) {
        break;
      }
      first_match--;
    }
  }

  int len;
  hlf_T group;
  schar_T fillchar = fillchar_status(&group, curwin);
  int attr = win_hl_attr(curwin, (int)group);

  if (first_match == 0) {
    *buf = NUL;
    len = 0;
  } else {
    STRCPY(buf, "< ");
    len = 2;
  }
  clen = len;

  i = first_match;
  while (clen + wildmenu_match_len(xp, SHOW_MATCH(i)) + 2 < Columns) {
    if (i == match) {
      selstart = buf + len;
      selstart_col = clen;
    }

    char *s = SHOW_MATCH(i);
    // Check for menu separators - replace with '|'
    int emenu = (xp->xp_context == EXPAND_MENUS || xp->xp_context == EXPAND_MENUNAMES);
    if (emenu && menu_is_separator(s)) {
      STRCPY(buf + len, transchar('|'));
      l = (int)strlen(buf + len);
      len += l;
      clen += l;
    } else {
      for (; *s != NUL; s++) {
        s += skip_wildmenu_char(xp, s);
        clen += ptr2cells(s);
        if ((l = utfc_ptr2len(s)) > 1) {
          strncpy(buf + len, s, (size_t)l);  // NOLINT(runtime/printf)
          s += l - 1;
          len += l;
        } else {
          STRCPY(buf + len, transchar_byte((uint8_t)(*s)));
          len += (int)strlen(buf + len);
        }
      }
    }
    if (i == match) {
      selend = buf + len;
    }

    *(buf + len++) = ' ';
    *(buf + len++) = ' ';
    clen += 2;
    if (++i == num_matches) {
      break;
    }
  }

  if (i != num_matches) {
    *(buf + len++) = '>';
    clen++;
  }

  buf[len] = NUL;

  int row = cmdline_row - 1;
  if (row >= 0) {
    if (wild_menu_showing == 0) {
      if (msg_scrolled > 0) {
        // Put the wildmenu just above the command line.  If there is
        // no room, scroll the screen one line up.
        if (cmdline_row == Rows - 1) {
          msg_scroll_up(false, false);
          msg_scrolled++;
        } else {
          cmdline_row++;
          row++;
        }
        wild_menu_showing = WM_SCROLLED;
      } else {
        // Create status line if needed by setting 'laststatus' to 2.
        // Set 'winminheight' to zero to avoid that the window is
        // resized.
        if (lastwin->w_status_height == 0 && rs_global_stl_height() == 0) {
          save_p_ls = (int)p_ls;
          save_p_wmh = (int)p_wmh;
          p_ls = 2;
          p_wmh = 0;
          rs_last_status(0);
        }
        wild_menu_showing = WM_SHOWN;
      }
    }

    // Tricky: wildmenu can be drawn either over a status line, or at empty
    // scrolled space in the message output
    grid_line_start((wild_menu_showing == WM_SCROLLED) ? &msg_grid_adj : &default_gridview, row);

    grid_line_puts(0, buf, -1, attr);
    if (selstart != NULL && highlight) {
      *selend = NUL;
      grid_line_puts(selstart_col, selstart, -1, HL_ATTR(HLF_WM));
    }

    grid_line_fill(clen, Columns, fillchar, attr);

    grid_line_flush();
  }

  win_redraw_last_status(topframe);
  xfree(buf);
}

/// Get the next or prev cmdline completion match. The index of the match is set
/// in "xp->xp_selected"
/// Navigate to next/previous completion match. Rust implementation.
static char *get_next_or_prev_match(int mode, expand_T *xp)
{
  return rs_get_next_or_prev_match(mode, xp);
}

/// Start the command-line expansion and get the matches. Rust implementation.
static char *ExpandOne_start(int mode, expand_T *xp, char *str, int options)
{
  return rs_expand_one_start(mode, xp, str, options);
}

/// Return the longest common part in the list of cmdline completion matches.
/// Rust implementation.
static char *find_longest_match(expand_T *xp, int options)
{
  return rs_find_longest_match(xp, options);
}

/// Do wildcard expansion on the string "str".
/// Chars that should not be expanded must be preceded with a backslash.
/// Return a pointer to allocated memory containing the new string.
/// Return NULL for failure.
///
/// "orig" is the originally expanded string, copied to allocated memory.  It
/// should either be kept in "xp->xp_orig" or freed.  When "mode" is WILD_NEXT
/// or WILD_PREV "orig" should be NULL.
///
/// Results are cached in xp->xp_files and xp->xp_numfiles, except when "mode"
/// is WILD_EXPAND_FREE or WILD_ALL.
///
/// mode = WILD_FREE:        just free previously expanded matches
/// mode = WILD_EXPAND_FREE: normal expansion, do not keep matches
/// mode = WILD_EXPAND_KEEP: normal expansion, keep matches
/// mode = WILD_NEXT:        use next match in multiple match, wrap to first
/// mode = WILD_PREV:        use previous match in multiple match, wrap to first
/// mode = WILD_ALL:         return all matches concatenated
/// mode = WILD_LONGEST:     return longest matched part
/// mode = WILD_ALL_KEEP:    get all matches, keep matches
/// mode = WILD_APPLY:       apply the item selected in the cmdline completion
///                          popup menu and close the menu.
/// mode = WILD_CANCEL:      cancel and close the cmdline completion popup and
///                          use the original text.
/// mode = WILD_PUM_WANT:    use the match at index pum_want.item
///
/// options = WILD_LIST_NOTFOUND:    list entries without a match
/// options = WILD_HOME_REPLACE:     do home_replace() for buffer names
/// options = WILD_USE_NL:           Use '\n' for WILD_ALL
/// options = WILD_NO_BEEP:          Don't beep for multiple matches
/// options = WILD_ADD_SLASH:        add a slash after directory names
/// options = WILD_KEEP_ALL:         don't remove 'wildignore' entries
/// options = WILD_SILENT:           don't print warning messages
/// options = WILD_ESCAPE:           put backslash before special chars
/// options = WILD_ICASE:            ignore case for files
///
/// The variables xp->xp_context and xp->xp_backslash must have been set!
///
/// @param orig  allocated copy of original of expanded string
/// Do wildcard expansion on the string "str". Rust implementation.
char *ExpandOne(expand_T *xp, char *str, char *orig, int options, int mode)
{
  return rs_expand_one(xp, str, orig, options, mode);
}

/// Prepare an expand structure for use. Rust implementation.
void ExpandInit(expand_T *xp)
  FUNC_ATTR_NONNULL_ALL
{
  rs_expand_init(xp);
}

/// Cleanup an expand structure after use. Rust implementation.
void ExpandCleanup(expand_T *xp)
{
  rs_expand_cleanup(xp);
}

/// Clear the static cmdline_orig. Rust implementation.
void clear_cmdline_orig(void)
{
  rs_clear_cmdline_orig();
}

/// Display one line of completion matches. Multiple matches are displayed in
/// each line (used by wildmode=list and CTRL-D)
///
/// @param matches      list of completion match names
/// @param numMatches   number of completion matches in "matches"
/// @param lines        number of output lines
/// @param linenr       line number of matches to display
/// @param maxlen       maximum number of characters in each line
/// @param showtail     display only the tail of the full path of a file name
static void showmatches_oneline(expand_T *xp, char **matches, int numMatches, int lines, int linenr,
                                int maxlen, bool showtail)
{
  char *p;
  int lastlen = 999;
  for (int j = linenr; j < numMatches; j += lines) {
    if (xp->xp_context == EXPAND_TAGS_LISTFILES) {
      msg_outtrans(matches[j], HLF_D, false);
      p = matches[j] + strlen(matches[j]) + 1;
      msg_advance(maxlen + 1);
      msg_puts(p);
      msg_advance(maxlen + 3);
      msg_outtrans_long(p + 2, HLF_D);
      break;
    }
    for (int i = maxlen - lastlen; --i >= 0;) {
      msg_putchar(' ');
    }
    bool isdir;
    if (xp->xp_context == EXPAND_FILES
        || xp->xp_context == EXPAND_SHELLCMD
        || xp->xp_context == EXPAND_BUFFERS) {
      // highlight directories
      if (xp->xp_numfiles != -1) {
        // Expansion was done before and special characters
        // were escaped, need to halve backslashes.  Also
        // $HOME has been replaced with ~/.
        char *exp_path = expand_env_save_opt(matches[j], true);
        char *path = exp_path != NULL ? exp_path : matches[j];
        char *halved_slash = backslash_halve_save(path);
        isdir = os_isdir(halved_slash);
        xfree(exp_path);
        if (halved_slash != path) {
          xfree(halved_slash);
        }
      } else {
        // Expansion was done here, file names are literal.
        isdir = os_isdir(matches[j]);
      }
      if (showtail) {
        p = SHOW_MATCH(j);
      } else {
        home_replace(NULL, matches[j], NameBuff, MAXPATHL, true);
        p = NameBuff;
      }
    } else {
      isdir = false;
      p = SHOW_MATCH(j);
    }
    lastlen = msg_outtrans(p, isdir ? HLF_D : 0, false);
  }
  if (msg_col > 0) {  // when not wrapped around
    msg_clr_eos();
    msg_putchar('\n');
  }
}

/// Display completion matches.
/// Returns EXPAND_NOTHING when the character that triggered expansion should be
///   inserted as a normal character.
int showmatches(expand_T *xp, bool display_wildmenu, bool display_list, bool noselect)
{
  CmdlineInfo *const ccline = get_cmdline_info();
  int numMatches;
  char **matches;
  int maxlen;
  int lines;
  int columns;
  bool showtail;

  if (xp->xp_numfiles == -1) {
    set_expand_context(xp);
    if (xp->xp_context == EXPAND_LUA) {
      nlua_expand_pat(xp);
    }
    int retval = expand_cmdline(xp, ccline->cmdbuff, ccline->cmdpos,
                                &numMatches, &matches);
    if (retval != EXPAND_OK) {
      return retval;
    }
    showtail = expand_showtail(xp);
  } else {
    numMatches = xp->xp_numfiles;
    matches = xp->xp_files;
    showtail = cmd_showtail;
  }

  if (cmdline_compl_use_pum(display_wildmenu && !display_list)) {
    cmdline_pum_create(ccline, xp, matches, numMatches, showtail, noselect);
    compl_selected = noselect ? -1 : 0;
    pum_clear();
    cmdline_pum_display(true);
    return EXPAND_OK;
  }

  if (display_list) {
    msg_didany = false;                 // lines_left will be set
    msg_start();                        // prepare for paging
    msg_putchar('\n');
    ui_flush();
    cmdline_row = msg_row;
    msg_didany = false;                 // lines_left will be set again
    msg_ext_set_kind("wildlist");
    msg_start();                        // prepare for paging
  }

  if (got_int) {
    got_int = false;  // only interrupt the completion, not the cmd line
  } else if (display_wildmenu && !display_list) {
    redraw_wildmenu(xp, numMatches, matches, noselect ? -1 : 0,
                    showtail);  // display statusbar menu
  } else if (display_list) {
    // find the length of the longest file name
    maxlen = 0;
    for (int i = 0; i < numMatches; i++) {
      int len;
      if (!showtail && (xp->xp_context == EXPAND_FILES
                        || xp->xp_context == EXPAND_SHELLCMD
                        || xp->xp_context == EXPAND_BUFFERS)) {
        home_replace(NULL, matches[i], NameBuff, MAXPATHL, true);
        len = vim_strsize(NameBuff);
      } else {
        len = vim_strsize(SHOW_MATCH(i));
      }
      maxlen = MAX(maxlen, len);
    }

    if (xp->xp_context == EXPAND_TAGS_LISTFILES) {
      lines = numMatches;
    } else {
      // compute the number of columns and lines for the listing
      maxlen += 2;          // two spaces between file names
      columns = (Columns + 2) / maxlen;
      if (columns < 1) {
        columns = 1;
      }
      lines = (numMatches + columns - 1) / columns;
    }

    if (xp->xp_context == EXPAND_TAGS_LISTFILES) {
      msg_puts_hl(_("tagname"), HLF_T, false);
      msg_clr_eos();
      msg_advance(maxlen - 3);
      msg_puts_hl(_(" kind file\n"), HLF_T, false);
    }

    // list the files line by line
    for (int i = 0; i < lines; i++) {
      showmatches_oneline(xp, matches, numMatches, lines, i, maxlen, showtail);
      if (got_int) {
        got_int = false;
        break;
      }
    }

    // we redraw the command below the lines that we have just listed
    // This is a bit tricky, but it saves a lot of screen updating.
    cmdline_row = msg_row;      // will put it back later
  }

  if (xp->xp_numfiles == -1) {
    FreeWild(numMatches, matches);
  }

  return EXPAND_OK;
}

/// path_tail() version for showmatches() and redraw_wildmenu():
/// Return the tail of file name path "s", ignoring a trailing "/".
/// Rust implementation.
static char *showmatches_gettail(char *s, bool eager)
{
  return rs_showmatches_gettail(s, eager);
}

/// Return true if we only need to show the tail of completion matches.
/// When not completing file names or there is a wildcard in the path false is
/// returned. Rust implementation.
static bool expand_showtail(expand_T *xp)
{
  return rs_expand_showtail(xp) != 0;
}

/// Prepare a string for expansion. Rust implementation.
///
/// When expanding file names: The string will be used with expand_wildcards().
/// Copy "fname[len]" into allocated memory and add a '*' at the end.
/// When expanding other names: The string will be used with regcomp().  Copy
/// the name into allocated memory and prepend "^".
///
/// @param context EXPAND_FILES etc.
char *addstar(char *fname, size_t len, int context)
  FUNC_ATTR_NONNULL_RET
{
  return rs_addstar(fname, len, context);
}

/// Must parse the command line so far to work out what context we are in.
/// Completion can then be done based on that context.
/// This routine sets the variables:
///  xp->xp_pattern          The start of the pattern to be expanded within
///                              the command line (ends at the cursor).
///  xp->xp_context          The type of thing to expand.  Will be one of:
///
///  EXPAND_UNSUCCESSFUL     Used sometimes when there is something illegal on
///                          the command line, like an unknown command.  Caller
///                          should beep.
///  EXPAND_NOTHING          Unrecognised context for completion, use char like
///                          a normal char, rather than for completion.  eg
///                          :s/^I/
///  EXPAND_COMMANDS         Cursor is still touching the command, so complete
///                          it.
///  EXPAND_BUFFERS          Complete file names for :buf and :sbuf commands.
///  EXPAND_FILES            After command with EX_XFILE set, or after setting
///                          with kOptFlagExpand set.  eg :e ^I, :w>>^I
///  EXPAND_DIRECTORIES      In some cases this is used instead of the latter
///                          when we know only directories are of interest.
///                          E.g.  :set dir=^I  and  :cd ^I
///  EXPAND_SHELLCMD         After ":!cmd", ":r !cmd"  or ":w !cmd".
///  EXPAND_SETTINGS         Complete variable names.  eg :set d^I
///  EXPAND_BOOL_SETTINGS    Complete boolean variables only,  eg :set no^I
///  EXPAND_TAGS             Complete tags from the files in p_tags.  eg :ta a^I
///  EXPAND_TAGS_LISTFILES   As above, but list filenames on ^D, after :tselect
///  EXPAND_HELP             Complete tags from the file 'helpfile'/tags
///  EXPAND_EVENTS           Complete event names
///  EXPAND_SYNTAX           Complete :syntax command arguments
///  EXPAND_HIGHLIGHT        Complete highlight (syntax) group names
///  EXPAND_AUGROUP          Complete autocommand group names
///  EXPAND_USER_VARS        Complete user defined variable names, eg :unlet a^I
///  EXPAND_MAPPINGS         Complete mapping and abbreviation names,
///                            eg :unmap a^I , :cunab x^I
///  EXPAND_FUNCTIONS        Complete internal or user defined function names,
///                            eg :call sub^I
///  EXPAND_USER_FUNC        Complete user defined function names, eg :delf F^I
///  EXPAND_EXPRESSION       Complete internal or user defined function/variable
///                          names in expressions, eg :while s^I
///  EXPAND_ENV_VARS         Complete environment variable names
///  EXPAND_USER             Complete user names
///  EXPAND_PATTERN_IN_BUF   Complete pattern in '/', '?', ':s', ':g', etc.
void set_expand_context(expand_T *xp)
{
  CmdlineInfo *const ccline = get_cmdline_info();

  // Handle search commands: '/' or '?'
  if ((ccline->cmdfirstc == '/' || ccline->cmdfirstc == '?')
      && may_expand_pattern) {
    xp->xp_context = EXPAND_PATTERN_IN_BUF;
    xp->xp_search_dir = (ccline->cmdfirstc == '/') ? FORWARD : BACKWARD;
    xp->xp_pattern = ccline->cmdbuff;
    xp->xp_pattern_len = (size_t)ccline->cmdpos;
    search_first_line = 0;  // Search entire buffer
    return;
  }

  // Only handle ':', '>', or '=' command-lines, or expression input
  if (ccline->cmdfirstc != ':'
      && ccline->cmdfirstc != '>' && ccline->cmdfirstc != '='
      && !ccline->input_fn) {
    xp->xp_context = EXPAND_NOTHING;
    return;
  }

  // Fallback to command-line expansion
  set_cmd_context(xp, ccline->cmdbuff, ccline->cmdlen, ccline->cmdpos, true);
}

/// Sets the index of a built-in or user defined command "cmd" in eap->cmdidx.
/// For user defined commands, the completion context is set in "xp" and the
/// completion flags in "complp".
///
/// @return  a pointer to the text after the command or NULL for failure.
static const char *set_cmd_index(const char *cmd, exarg_T *eap, expand_T *xp, int *complp)
{
  const char *p = NULL;
  const bool fuzzy = cmdline_fuzzy_complete(cmd);

  // Isolate the command and search for it in the command table.
  // Exceptions:
  // - the 'k' command can directly be followed by any character, but do
  // accept "keepmarks", "keepalt" and "keepjumps". As fuzzy matching can
  // find matches anywhere in the command name, do this only for command
  // expansion based on regular expression and not for fuzzy matching.
  // - the 's' command can be followed directly by 'c', 'g', 'i', 'I' or 'r'
  if (!fuzzy && (*cmd == 'k' && cmd[1] != 'e')) {
    eap->cmdidx = CMD_k;
    p = cmd + 1;
  } else {
    p = cmd;
    while (ASCII_ISALPHA(*p) || *p == '*') {  // Allow * wild card
      p++;
    }
    // a user command may contain digits
    if (ASCII_ISUPPER(cmd[0])) {
      while (ASCII_ISALNUM(*p) || *p == '*') {
        p++;
      }
    }
    // for python 3.x: ":py3*" commands completion
    if (cmd[0] == 'p' && cmd[1] == 'y' && p == cmd + 2 && *p == '3') {
      p++;
      while (ASCII_ISALPHA(*p) || *p == '*') {
        p++;
      }
    }
    // check for non-alpha command
    if (p == cmd && vim_strchr("@*!=><&~#", (uint8_t)(*p)) != NULL) {
      p++;
    }
    size_t len = (size_t)(p - cmd);

    if (len == 0) {
      xp->xp_context = EXPAND_UNSUCCESSFUL;
      return NULL;
    }

    eap->cmdidx = excmd_get_cmdidx(cmd, len);

    // User defined commands support alphanumeric characters.
    // Also when doing fuzzy expansion for non-shell commands, support
    // alphanumeric characters.
    if ((cmd[0] >= 'A' && cmd[0] <= 'Z')
        || (fuzzy && eap->cmdidx != CMD_bang && *p != NUL)) {
      while (ASCII_ISALNUM(*p) || *p == '*') {  // Allow * wild card
        p++;
      }
    }
  }

  // If the cursor is touching the command, and it ends in an alphanumeric
  // character, complete the command name.
  if (*p == NUL && ASCII_ISALNUM(p[-1])) {
    return NULL;
  }

  if (eap->cmdidx == CMD_SIZE) {
    if (*cmd == 's' && vim_strchr("cgriI", (uint8_t)cmd[1]) != NULL) {
      eap->cmdidx = CMD_substitute;
      p = cmd + 1;
    } else if (cmd[0] >= 'A' && cmd[0] <= 'Z') {
      eap->cmd = (char *)cmd;
      p = find_ucmd(eap, (char *)p, NULL, xp, complp);
      if (p == NULL) {
        eap->cmdidx = CMD_SIZE;  // Ambiguous user command.
      }
    }
  }
  if (eap->cmdidx == CMD_SIZE) {
    // Not still touching the command and it was an illegal one
    xp->xp_context = EXPAND_UNSUCCESSFUL;
    return NULL;
  }

  return p;
}

/// Set the completion context for a command argument with wild card characters.
static void set_context_for_wildcard_arg(exarg_T *eap, const char *arg, bool usefilter,
                                         expand_T *xp, int *complp)
{
  int is_shell_cmd = usefilter
                     || (eap != NULL && (eap->cmdidx == CMD_bang || eap->cmdidx == CMD_terminal));
  rs_set_context_for_wildcard_arg(arg, is_shell_cmd ? 1 : 0, xp, complp);
}

/// Set the completion context for the "++opt=arg" argument.  Always returns NULL.
static const char *set_context_in_argopt(expand_T *xp, const char *arg)
{
  return rs_set_context_in_argopt(xp, arg);
}

/// Set the completion context for the :filter command. Returns a pointer to the
/// next command after the :filter command.
static const char *set_context_in_filter_cmd(expand_T *xp, const char *arg)
{
  return rs_set_context_in_filter_cmd(xp, arg);
}

/// Set the completion context for the :match command. Returns a pointer to the
/// next command after the :match command.
static const char *set_context_in_match_cmd(expand_T *xp, const char *arg)
{
  return rs_set_context_in_match_cmd(xp, arg);
}

/// Returns a pointer to the next command after a :global or a :v command.
/// Returns NULL if there is no next command.
static const char *find_cmd_after_global_cmd(const char *arg)
{
  return rs_find_cmd_after_global_cmd(arg);
}

/// Returns a pointer to the next command after a :substitute or a :& command.
/// Returns NULL if there is no next command.
static const char *find_cmd_after_substitute_cmd(const char *arg)
{
  return rs_find_cmd_after_substitute_cmd(arg);
}

/// Returns a pointer to the next command after a :isearch/:dsearch/:ilist
/// :dlist/:ijump/:psearch/:djump/:isplit/:dsplit command.
/// Returns NULL if there is no next command.
static const char *find_cmd_after_isearch_cmd(expand_T *xp, const char *arg)
{
  return rs_find_cmd_after_isearch_cmd(xp, arg);
}

/// Set the completion context for the :unlet command. Always returns NULL.
static const char *set_context_in_unlet_cmd(expand_T *xp, const char *arg)
{
  return rs_set_context_in_unlet_cmd(xp, arg);
}

/// Set the completion context for the :language command. Always returns NULL.
static const char *set_context_in_lang_cmd(expand_T *xp, const char *arg)
{
  return rs_set_context_in_lang_cmd(xp, arg);
}

/// Set the completion context for the :breakadd command. Always returns NULL.
static const char *set_context_in_breakadd_cmd(expand_T *xp, const char *arg, cmdidx_T cmdidx)
{
  int breakpt_cmd_type;
  if (cmdidx == CMD_breakadd) {
    breakpt_cmd_type = 0;  // BREAKPT_CMD_ADD
  } else if (cmdidx == CMD_breakdel) {
    breakpt_cmd_type = 1;  // BREAKPT_CMD_DEL
  } else {
    breakpt_cmd_type = 2;  // BREAKPT_CMD_PROFDEL
  }
  return rs_set_context_in_breakadd_cmd(xp, arg, breakpt_cmd_type);
}

static const char *set_context_in_scriptnames_cmd(expand_T *xp, const char *arg)
{
  return rs_set_context_in_scriptnames_cmd(xp, arg);
}

/// Set the completion context for the :filetype command. Always returns NULL.
static const char *set_context_in_filetype_cmd(expand_T *xp, const char *arg)
{
  return rs_set_context_in_filetype_cmd(xp, arg);
}

/// Sets the completion context for commands that involve a search pattern
/// and a line range (e.g., :s, :g, :v).
static void set_context_with_pattern(expand_T *xp)
{
  rs_set_context_with_pattern(xp);
}

/// Set the completion context in "xp" for command "cmd" with index "cmdidx".
/// The argument to the command is "arg" and the argument flags is "argt".
/// For user-defined commands and for environment variables, "context" has the
/// completion type.
///
/// @return  a pointer to the next command, or NULL if there is no next command.
static const char *set_context_by_cmdname(const char *cmd, cmdidx_T cmdidx, expand_T *xp,
                                          const char *arg, uint32_t argt, int context, bool forceit)
{
  const char *nextcmd;

  switch (cmdidx) {
  case CMD_find:
  case CMD_sfind:
  case CMD_tabfind:
    if (xp->xp_context == EXPAND_FILES) {
      xp->xp_context = *get_findfunc() != NUL ? EXPAND_FINDFUNC : EXPAND_FILES_IN_PATH;
    }
    break;
  case CMD_cd:
  case CMD_chdir:
  case CMD_lcd:
  case CMD_lchdir:
  case CMD_tcd:
  case CMD_tchdir:
    if (xp->xp_context == EXPAND_FILES) {
      xp->xp_context = EXPAND_DIRS_IN_CDPATH;
    }
    break;
  case CMD_help:
    xp->xp_context = EXPAND_HELP;
    xp->xp_pattern = (char *)arg;
    break;

  // Command modifiers: return the argument.
  // Also for commands with an argument that is a command.
  case CMD_aboveleft:
  case CMD_argdo:
  case CMD_belowright:
  case CMD_botright:
  case CMD_browse:
  case CMD_bufdo:
  case CMD_cdo:
  case CMD_cfdo:
  case CMD_confirm:
  case CMD_debug:
  case CMD_folddoclosed:
  case CMD_folddoopen:
  case CMD_hide:
  case CMD_horizontal:
  case CMD_keepalt:
  case CMD_keepjumps:
  case CMD_keepmarks:
  case CMD_keeppatterns:
  case CMD_ldo:
  case CMD_leftabove:
  case CMD_lfdo:
  case CMD_lockmarks:
  case CMD_noautocmd:
  case CMD_noswapfile:
  case CMD_restart:
  case CMD_rightbelow:
  case CMD_sandbox:
  case CMD_silent:
  case CMD_tab:
  case CMD_tabdo:
  case CMD_topleft:
  case CMD_unsilent:
  case CMD_verbose:
  case CMD_vertical:
  case CMD_windo:
    return arg;

  case CMD_filter:
    return set_context_in_filter_cmd(xp, arg);

  case CMD_match:
    return set_context_in_match_cmd(xp, arg);

  // All completion for the +cmdline_compl feature goes here.

  case CMD_command:
    return set_context_in_user_cmd(xp, arg);

  case CMD_delcommand:
    xp->xp_context = EXPAND_USER_COMMANDS;
    xp->xp_pattern = (char *)arg;
    break;

  case CMD_global:
  case CMD_vglobal:
    nextcmd = find_cmd_after_global_cmd(arg);
    if (!nextcmd && may_expand_pattern) {
      set_context_with_pattern(xp);
    }
    return nextcmd;

  case CMD_and:
  case CMD_substitute:
    nextcmd = find_cmd_after_substitute_cmd(arg);
    if (!nextcmd && may_expand_pattern) {
      set_context_with_pattern(xp);
    }
    return nextcmd;

  case CMD_isearch:
  case CMD_dsearch:
  case CMD_ilist:
  case CMD_dlist:
  case CMD_ijump:
  case CMD_psearch:
  case CMD_djump:
  case CMD_isplit:
  case CMD_dsplit:
    return find_cmd_after_isearch_cmd(xp, arg);
  case CMD_autocmd:
    return set_context_in_autocmd(xp, (char *)arg, false);

  case CMD_doautocmd:
  case CMD_doautoall:
    return set_context_in_autocmd(xp, (char *)arg, true);
  case CMD_set:
    set_context_in_set_cmd(xp, (char *)arg, 0);
    break;
  case CMD_setglobal:
    set_context_in_set_cmd(xp, (char *)arg, OPT_GLOBAL);
    break;
  case CMD_setlocal:
    set_context_in_set_cmd(xp, (char *)arg, OPT_LOCAL);
    break;
  case CMD_tag:
  case CMD_stag:
  case CMD_ptag:
  case CMD_ltag:
  case CMD_tselect:
  case CMD_stselect:
  case CMD_ptselect:
  case CMD_tjump:
  case CMD_stjump:
  case CMD_ptjump:
    if (wop_flags & kOptWopFlagTagfile) {
      xp->xp_context = EXPAND_TAGS_LISTFILES;
    } else {
      xp->xp_context = EXPAND_TAGS;
    }
    xp->xp_pattern = (char *)arg;
    break;
  case CMD_augroup:
    xp->xp_context = EXPAND_AUGROUP;
    xp->xp_pattern = (char *)arg;
    break;
  case CMD_syntax:
    set_context_in_syntax_cmd(xp, arg);
    break;
  case CMD_const:
  case CMD_let:
  case CMD_if:
  case CMD_elseif:
  case CMD_while:
  case CMD_for:
  case CMD_echo:
  case CMD_echon:
  case CMD_execute:
  case CMD_echomsg:
  case CMD_echoerr:
  case CMD_call:
  case CMD_return:
  case CMD_cexpr:
  case CMD_caddexpr:
  case CMD_cgetexpr:
  case CMD_lexpr:
  case CMD_laddexpr:
  case CMD_lgetexpr:
    set_context_for_expression(xp, (char *)arg, cmdidx);
    break;

  case CMD_unlet:
    return set_context_in_unlet_cmd(xp, arg);
  case CMD_function:
  case CMD_delfunction:
    xp->xp_context = EXPAND_USER_FUNC;
    xp->xp_pattern = (char *)arg;
    break;

  case CMD_echohl:
    set_context_in_echohl_cmd(xp, arg);
    break;
  case CMD_highlight:
    set_context_in_highlight_cmd(xp, arg);
    break;
  case CMD_sign:
    set_context_in_sign_cmd(xp, (char *)arg);
    break;
  case CMD_bdelete:
  case CMD_bwipeout:
  case CMD_bunload:
    while ((xp->xp_pattern = strchr(arg, ' ')) != NULL) {
      arg = xp->xp_pattern + 1;
    }
    FALLTHROUGH;
  case CMD_buffer:
  case CMD_sbuffer:
  case CMD_pbuffer:
  case CMD_checktime:
    xp->xp_context = EXPAND_BUFFERS;
    xp->xp_pattern = (char *)arg;
    break;
  case CMD_diffget:
  case CMD_diffput:
    // If current buffer is in diff mode, complete buffer names
    // which are in diff mode, and different than current buffer.
    xp->xp_context = EXPAND_DIFF_BUFFERS;
    xp->xp_pattern = (char *)arg;
    break;

  case CMD_USER:
  case CMD_USER_BUF:
    return set_context_in_user_cmdarg(cmd, arg, argt, context, xp, forceit);

  case CMD_map:
  case CMD_noremap:
  case CMD_nmap:
  case CMD_nnoremap:
  case CMD_vmap:
  case CMD_vnoremap:
  case CMD_omap:
  case CMD_onoremap:
  case CMD_imap:
  case CMD_inoremap:
  case CMD_cmap:
  case CMD_cnoremap:
  case CMD_lmap:
  case CMD_lnoremap:
  case CMD_smap:
  case CMD_snoremap:
  case CMD_xmap:
  case CMD_xnoremap:
    return set_context_in_map_cmd(xp, (char *)cmd, (char *)arg, forceit, false,
                                  false, cmdidx);
  case CMD_unmap:
  case CMD_nunmap:
  case CMD_vunmap:
  case CMD_ounmap:
  case CMD_iunmap:
  case CMD_cunmap:
  case CMD_lunmap:
  case CMD_sunmap:
  case CMD_xunmap:
    return set_context_in_map_cmd(xp, (char *)cmd, (char *)arg, forceit, false,
                                  true, cmdidx);
  case CMD_mapclear:
  case CMD_nmapclear:
  case CMD_vmapclear:
  case CMD_omapclear:
  case CMD_imapclear:
  case CMD_cmapclear:
  case CMD_lmapclear:
  case CMD_smapclear:
  case CMD_xmapclear:
    xp->xp_context = EXPAND_MAPCLEAR;
    xp->xp_pattern = (char *)arg;
    break;

  case CMD_abbreviate:
  case CMD_noreabbrev:
  case CMD_cabbrev:
  case CMD_cnoreabbrev:
  case CMD_iabbrev:
  case CMD_inoreabbrev:
    return set_context_in_map_cmd(xp, (char *)cmd, (char *)arg, forceit, true,
                                  false, cmdidx);
  case CMD_unabbreviate:
  case CMD_cunabbrev:
  case CMD_iunabbrev:
    return set_context_in_map_cmd(xp, (char *)cmd, (char *)arg, forceit, true,
                                  true, cmdidx);
  case CMD_menu:
  case CMD_noremenu:
  case CMD_unmenu:
  case CMD_amenu:
  case CMD_anoremenu:
  case CMD_aunmenu:
  case CMD_nmenu:
  case CMD_nnoremenu:
  case CMD_nunmenu:
  case CMD_vmenu:
  case CMD_vnoremenu:
  case CMD_vunmenu:
  case CMD_omenu:
  case CMD_onoremenu:
  case CMD_ounmenu:
  case CMD_imenu:
  case CMD_inoremenu:
  case CMD_iunmenu:
  case CMD_cmenu:
  case CMD_cnoremenu:
  case CMD_cunmenu:
  case CMD_tlmenu:
  case CMD_tlnoremenu:
  case CMD_tlunmenu:
  case CMD_tmenu:
  case CMD_tunmenu:
  case CMD_popup:
  case CMD_emenu:
    return set_context_in_menu_cmd(xp, cmd, (char *)arg, forceit);

  case CMD_colorscheme:
    xp->xp_context = EXPAND_COLORS;
    xp->xp_pattern = (char *)arg;
    break;

  case CMD_compiler:
    xp->xp_context = EXPAND_COMPILER;
    xp->xp_pattern = (char *)arg;
    break;

  case CMD_ownsyntax:
    xp->xp_context = EXPAND_OWNSYNTAX;
    xp->xp_pattern = (char *)arg;
    break;

  case CMD_setfiletype:
    xp->xp_context = EXPAND_FILETYPE;
    xp->xp_pattern = (char *)arg;
    break;

  case CMD_packadd:
    xp->xp_context = EXPAND_PACKADD;
    xp->xp_pattern = (char *)arg;
    break;

  case CMD_runtime:
    set_context_in_runtime_cmd(xp, arg);
    break;

  case CMD_language:
    return set_context_in_lang_cmd(xp, arg);

  case CMD_profile:
    set_context_in_profile_cmd(xp, arg);
    break;
  case CMD_checkhealth:
    xp->xp_context = EXPAND_CHECKHEALTH;
    break;

  case CMD_retab:
    xp->xp_context = EXPAND_RETAB;
    xp->xp_pattern = (char *)arg;
    break;

  case CMD_messages:
    xp->xp_context = EXPAND_MESSAGES;
    xp->xp_pattern = (char *)arg;
    break;

  case CMD_history:
    xp->xp_context = EXPAND_HISTORY;
    xp->xp_pattern = (char *)arg;
    break;
  case CMD_syntime:
    xp->xp_context = EXPAND_SYNTIME;
    xp->xp_pattern = (char *)arg;
    break;

  case CMD_argdelete:
    while ((xp->xp_pattern = vim_strchr(arg, ' ')) != NULL) {
      arg = (xp->xp_pattern + 1);
    }
    xp->xp_context = EXPAND_ARGLIST;
    xp->xp_pattern = (char *)arg;
    break;

  case CMD_breakadd:
  case CMD_profdel:
  case CMD_breakdel:
    return set_context_in_breakadd_cmd(xp, arg, cmdidx);

  case CMD_scriptnames:
    return set_context_in_scriptnames_cmd(xp, arg);

  case CMD_filetype:
    return set_context_in_filetype_cmd(xp, arg);

  case CMD_lua:
  case CMD_equal:
    xp->xp_context = EXPAND_LUA;
    break;

  default:
    break;
  }
  return NULL;
}

/// This is all pretty much copied from do_one_cmd(), with all the extra stuff
/// we don't need/want deleted.  Maybe this could be done better if we didn't
/// repeat all this stuff.  The only problem is that they may not stay
/// perfectly compatible with each other, but then the command line syntax
/// probably won't change that much -- webb.
///
/// @param buff  buffer for command string
static const char *set_one_cmd_context(expand_T *xp, const char *buff)
{
  size_t len = 0;
  exarg_T ea;
  int context = EXPAND_NOTHING;
  bool forceit = false;
  bool usefilter = false;  // Filter instead of file name.

  ExpandInit(xp);
  xp->xp_pattern = (char *)buff;
  xp->xp_line = (char *)buff;
  xp->xp_context = EXPAND_COMMANDS;  // Default until we get past command
  ea.argt = 0;

  // 1. skip comment lines and leading space, colons or bars
  const char *cmd;
  for (cmd = buff; vim_strchr(" \t:|", (uint8_t)(*cmd)) != NULL; cmd++) {}
  xp->xp_pattern = (char *)cmd;

  if (*cmd == NUL) {
    return NULL;
  }
  if (*cmd == '"') {  // ignore comment lines
    xp->xp_context = EXPAND_NOTHING;
    return NULL;
  }

  // 3. skip over a range specifier of the form: addr [,addr] [;addr] ..
  cmd = skip_range(cmd, &xp->xp_context);
  xp->xp_pattern = (char *)cmd;
  if (*cmd == NUL) {
    return NULL;
  }
  if (*cmd == '"') {
    xp->xp_context = EXPAND_NOTHING;
    return NULL;
  }

  if (*cmd == '|' || *cmd == '\n') {
    return cmd + 1;  // There's another command
  }

  // Get the command index.
  const char *p = set_cmd_index(cmd, &ea, xp, &context);
  if (p == NULL) {
    return NULL;
  }

  xp->xp_context = EXPAND_NOTHING;  // Default now that we're past command

  if (*p == '!') {  // forced commands
    forceit = true;
    p++;
  }

  // 6. parse arguments
  if (!IS_USER_CMDIDX(ea.cmdidx)) {
    ea.argt = excmd_get_argt(ea.cmdidx);
  }

  const char *arg = skipwhite(p);

  // Does command allow "++argopt" argument?
  if (ea.argt & EX_ARGOPT) {
    while (*arg != NUL && strncmp(arg, "++", 2) == 0) {
      p = arg + 2;
      while (*p && !ascii_isspace(*p)) {
        MB_PTR_ADV(p);
      }

      // Still touching the command after "++"?
      if (*p == NUL) {
        if (ea.argt & EX_ARGOPT) {
          return set_context_in_argopt(xp, arg + 2);
        }
      }

      arg = skipwhite(p);
    }
  }

  if (ea.cmdidx == CMD_write || ea.cmdidx == CMD_update) {
    if (*arg == '>') {  // append
      if (*++arg == '>') {
        arg++;
      }
      arg = skipwhite(arg);
    } else if (*arg == '!' && ea.cmdidx == CMD_write) {  // :w !filter
      arg++;
      usefilter = true;
    }
  }

  if (ea.cmdidx == CMD_read) {
    usefilter = forceit;  // :r! filter if forced
    if (*arg == '!') {    // :r !filter
      arg++;
      usefilter = true;
    }
  }

  if (ea.cmdidx == CMD_lshift || ea.cmdidx == CMD_rshift) {
    while (*arg == *cmd) {  // allow any number of '>' or '<'
      arg++;
    }
    arg = skipwhite(arg);
  }

  // Does command allow "+command"?
  if ((ea.argt & EX_CMDARG) && !usefilter && *arg == '+') {
    // Check if we're in the +command
    p = arg + 1;
    arg = skip_cmd_arg((char *)arg, false);

    // Still touching the command after '+'?
    if (*arg == NUL) {
      return p;
    }

    // Skip space(s) after +command to get to the real argument.
    arg = skipwhite(arg);
  }

  // Check for '|' to separate commands and '"' to start comments.
  // Don't do this for ":read !cmd" and ":write !cmd".
  if ((ea.argt & EX_TRLBAR) && !usefilter) {
    p = arg;
    // ":redir @" is not the start of a comment
    if (ea.cmdidx == CMD_redir && p[0] == '@' && p[1] == '"') {
      p += 2;
    }
    while (*p) {
      if (*p == Ctrl_V) {
        if (p[1] != NUL) {
          p++;
        }
      } else if ((*p == '"' && !(ea.argt & EX_NOTRLCOM))
                 || *p == '|'
                 || *p == '\n') {
        if (*(p - 1) != '\\') {
          if (*p == '|' || *p == '\n') {
            return p + 1;
          }
          return NULL;  // It's a comment
        }
      }
      MB_PTR_ADV(p);
    }
  }

  if (!(ea.argt & EX_EXTRA) && *arg != NUL && strchr("|\"", *arg) == NULL) {
    // no arguments allowed but there is something
    return NULL;
  }

  // Find start of last argument (argument just before cursor):
  p = buff;
  xp->xp_pattern = (char *)p;
  len = strlen(buff);
  while (*p && p < buff + len) {
    if (*p == ' ' || *p == TAB) {
      // argument starts after a space
      xp->xp_pattern = (char *)++p;
    } else {
      if (*p == '\\' && *(p + 1) != NUL) {
        p++;  // skip over escaped character
      }
      MB_PTR_ADV(p);
    }
  }

  if (ea.argt & EX_XFILE) {
    set_context_for_wildcard_arg(&ea, arg, usefilter, xp, &context);
  }

  // Switch on command name.
  return set_context_by_cmdname(cmd, ea.cmdidx, xp, arg, ea.argt, context, forceit);
}

/// Set the completion context in "xp" for command "str"
///
/// @param str  start of command line
/// @param len  length of command line (excl. NUL)
/// @param col  position of cursor
/// @param use_ccline  use ccline for info
void set_cmd_context(expand_T *xp, char *str, int len, int col, int use_ccline)
{
  CmdlineInfo *const ccline = get_cmdline_info();
  char old_char = NUL;

  // Avoid a UMR warning from Purify, only save the character if it has been
  // written before.
  if (col < len) {
    old_char = str[col];
  }
  str[col] = NUL;
  const char *nextcomm = str;

  if (use_ccline && ccline->cmdfirstc == '=') {
    // pass CMD_SIZE because there is no real command
    set_context_for_expression(xp, str, CMD_SIZE);
  } else if (use_ccline && ccline->input_fn) {
    xp->xp_context = ccline->xp_context;
    xp->xp_pattern = ccline->cmdbuff;
    xp->xp_arg = ccline->xp_arg;
    if (xp->xp_context == EXPAND_SHELLCMDLINE) {
      int context = xp->xp_context;
      set_context_for_wildcard_arg(NULL, xp->xp_pattern, false, xp, &context);
    }
  } else {
    while (nextcomm != NULL) {
      nextcomm = set_one_cmd_context(xp, nextcomm);
    }
  }

  // Store the string here so that call_user_expand_func() can get to them
  // easily.
  xp->xp_line = str;
  xp->xp_col = col;

  str[col] = old_char;
}

/// Expand the command line "str" from context "xp".
/// "xp" must have been set by set_cmd_context().
/// xp->xp_pattern points into "str", to where the text that is to be expanded
/// starts.
/// Returns EXPAND_UNSUCCESSFUL when there is something illegal before the
/// cursor.
/// Returns EXPAND_NOTHING when there is nothing to expand, might insert the
/// key that triggered expansion literally.
/// Returns EXPAND_OK otherwise.
///
/// @param str  start of command line
/// @param col  position of cursor
/// @param matchcount  return: nr of matches
/// @param matches  return: array of pointers to matches
int expand_cmdline(expand_T *xp, const char *str, int col, int *matchcount, char ***matches)
{
  char *file_str = NULL;
  int options = WILD_ADD_SLASH|WILD_SILENT;

  if (xp->xp_context == EXPAND_UNSUCCESSFUL) {
    beep_flush();
    return EXPAND_UNSUCCESSFUL;      // Something illegal on command line
  }
  if (xp->xp_context == EXPAND_NOTHING) {
    // Caller can use the character as a normal char instead
    return EXPAND_NOTHING;
  }

  // add star to file name, or convert to regexp if not exp. files.
  assert((str + col) - xp->xp_pattern >= 0);
  xp->xp_pattern_len = (size_t)((str + col) - xp->xp_pattern);
  if (cmdline_fuzzy_completion_supported(xp)) {
    // If fuzzy matching, don't modify the search string
    file_str = xstrdup(xp->xp_pattern);
  } else {
    file_str = addstar(xp->xp_pattern, xp->xp_pattern_len, xp->xp_context);
  }

  if (p_wic) {
    options += WILD_ICASE;
  }

  // find all files that match the description
  if (ExpandFromContext(xp, file_str, matches, matchcount, options) == FAIL) {
    *matchcount = 0;
    *matches = NULL;
  }
  xfree(file_str);

  return EXPAND_OK;
}

/// Expand file or directory names.
static int expand_files_and_dirs(expand_T *xp, char *pat, char ***matches, int *numMatches,
                                 int flags, int options)
{
  bool free_pat = false;

  // for ":set path=" and ":set tags=" halve backslashes for escaped space
  if (xp->xp_backslash != XP_BS_NONE) {
    free_pat = true;
    size_t pat_len = strlen(pat);
    pat = xstrnsave(pat, pat_len);

    char *pat_end = pat + pat_len;
    for (char *p = pat; *p != NUL; p++) {
      if (*p != '\\') {
        continue;
      }

      if (xp->xp_backslash & XP_BS_THREE
          && *(p + 1) == '\\'
          && *(p + 2) == '\\'
          && *(p + 3) == ' ') {
        char *from = p + 3;
        memmove(p, from, (size_t)(pat_end - from) + 1);  // +1 for NUL
        pat_end -= 3;
      } else if (xp->xp_backslash & XP_BS_ONE
                 && *(p + 1) == ' ') {
        char *from = p + 1;
        memmove(p, from, (size_t)(pat_end - from) + 1);  // +1 for NUL
        pat_end--;
      } else if (xp->xp_backslash & XP_BS_COMMA) {
        if (*(p + 1) == '\\' && *(p + 2) == ',') {
          char *from = p + 2;
          memmove(p, from, (size_t)(pat_end - from) + 1);  // +1 for NUL
          pat_end -= 2;
#ifdef BACKSLASH_IN_FILENAME
        } else if (*(p + 1) == ',') {
          char *from = p + 1;
          memmove(p, from, (size_t)(pat_end - from) + 1);  // +1 for NUL
          pat_end--;
#endif
        }
      }
    }
  }

  int ret = FAIL;
  if (xp->xp_context == EXPAND_FINDFUNC) {
    ret = expand_findfunc(pat, matches, numMatches);
  } else {
    if (xp->xp_context == EXPAND_FILES) {
      flags |= EW_FILE;
    } else if (xp->xp_context == EXPAND_FILES_IN_PATH) {
      flags |= (EW_FILE | EW_PATH);
    } else if (xp->xp_context == EXPAND_DIRS_IN_CDPATH) {
      flags = (flags | EW_DIR | EW_CDPATH) & ~EW_FILE;
    } else {
      flags = (flags | EW_DIR) & ~EW_FILE;
    }
    if (options & WILD_ICASE) {
      flags |= EW_ICASE;
    }
    // Expand wildcards, supporting %:h and the like.
    ret = expand_wildcards_eval(&pat, numMatches, matches, flags);
  }
  if (free_pat) {
    xfree(pat);
  }
#ifdef BACKSLASH_IN_FILENAME
  if (p_csl[0] != NUL && (options & WILD_IGNORE_COMPLETESLASH) == 0) {
    for (int j = 0; j < *numMatches; j++) {
      char *ptr = (*matches)[j];
      while (*ptr != NUL) {
        if (p_csl[0] == 's' && *ptr == '\\') {
          *ptr = '/';
        } else if (p_csl[0] == 'b' && *ptr == '/') {
          *ptr = '\\';
        }
        ptr += utfc_ptr2len(ptr);
      }
    }
  }
#endif
  return ret;
}

/// Function given to ExpandGeneric() to obtain the possible arguments of the
/// ":filetype {plugin,indent}" command.
static char *get_filetypecmd_arg(expand_T *xp FUNC_ATTR_UNUSED, int idx)
{
  return rs_get_filetypecmd_arg(xp, idx);
}

/// Function given to ExpandGeneric() to obtain the possible arguments of the
/// ":breakadd {expr, file, func, here}" command.
/// ":breakdel {func, file, here}" command.
static char *get_breakadd_arg(expand_T *xp FUNC_ATTR_UNUSED, int idx)
{
  return rs_get_breakadd_arg(xp, idx);
}

/// Function given to ExpandGeneric() to obtain the possible arguments for the
/// ":scriptnames" command.
static char *get_scriptnames_arg(expand_T *xp FUNC_ATTR_UNUSED, int idx)
{
  if (!SCRIPT_ID_VALID(idx + 1)) {
    return NULL;
  }

  scriptitem_T *si = SCRIPT_ITEM(idx + 1);
  home_replace(NULL, si->sn_name, NameBuff, MAXPATHL, true);
  return NameBuff;
}

/// Function given to ExpandGeneric() to obtain the possible arguments of the
/// ":retab {-indentonly}" option.
static char *get_retab_arg(expand_T *xp FUNC_ATTR_UNUSED, int idx)
{
  return rs_get_retab_arg(xp, idx);
}

/// Function given to ExpandGeneric() to obtain the possible arguments of the
/// ":messages {clear}" command.
static char *get_messages_arg(expand_T *xp FUNC_ATTR_UNUSED, int idx)
{
  return rs_get_messages_arg(xp, idx);
}

static char *get_mapclear_arg(expand_T *xp FUNC_ATTR_UNUSED, int idx)
{
  return rs_get_mapclear_arg(xp, idx);
}

/// Completion for |:checkhealth| command.
///
/// Given to ExpandGeneric() to obtain all available heathcheck names.
/// @param[in] idx  Index of the healthcheck item.
/// @param[in] xp  Not used.
static char *get_healthcheck_names(expand_T *xp FUNC_ATTR_UNUSED, int idx)
{
  static Object names = OBJECT_INIT;
  static unsigned last_gen = 0;

  if (last_gen != get_cmdline_last_prompt_id() || last_gen == 0) {
    Array a = ARRAY_DICT_INIT;
    Error err = ERROR_INIT;
    Object res = NLUA_EXEC_STATIC("return vim.health._complete()", a, kRetObject, NULL, &err);
    api_clear_error(&err);
    api_free_object(names);
    names = res;
    last_gen = get_cmdline_last_prompt_id();
  }

  if (names.type == kObjectTypeArray && idx < (int)names.data.array.size
      && names.data.array.items[idx].type == kObjectTypeString) {
    return names.data.array.items[idx].data.string.data;
  }
  return NULL;
}

/// Do the expansion based on xp->xp_context and "rmp".
static int ExpandOther(char *pat, expand_T *xp, regmatch_T *rmp, char ***matches, int *numMatches)
{
  typedef CompleteListItemGetter ExpandFunc;
  static struct expgen {
    int context;
    ExpandFunc func;
    int ic;
    int escaped;
  } tab[] = {
    { EXPAND_COMMANDS, get_command_name, false, true },
    { EXPAND_FILETYPECMD, get_filetypecmd_arg, true, true },
    { EXPAND_MAPCLEAR, get_mapclear_arg, true, true },
    { EXPAND_MESSAGES, get_messages_arg, true, true },
    { EXPAND_HISTORY, get_history_arg, true, true },
    { EXPAND_USER_COMMANDS, get_user_commands, false, true },
    { EXPAND_USER_ADDR_TYPE, get_user_cmd_addr_type, false, true },
    { EXPAND_USER_CMD_FLAGS, get_user_cmd_flags, false, true },
    { EXPAND_USER_NARGS, get_user_cmd_nargs, false, true },
    { EXPAND_USER_COMPLETE, get_user_cmd_complete, false, true },
    { EXPAND_USER_VARS, get_user_var_name, false, true },
    { EXPAND_FUNCTIONS, get_function_name, false, true },
    { EXPAND_USER_FUNC, get_user_func_name, false, true },
    { EXPAND_EXPRESSION, get_expr_name, false, true },
    { EXPAND_MENUS, get_menu_name, false, true },
    { EXPAND_MENUNAMES, get_menu_names, false, true },
    { EXPAND_SYNTAX, get_syntax_name, true, true },
    { EXPAND_SYNTIME, get_syntime_arg, true, true },
    { EXPAND_HIGHLIGHT, get_highlight_name, true, false },
    { EXPAND_EVENTS, expand_get_event_name, true, false },
    { EXPAND_AUGROUP, expand_get_augroup_name, true, false },
    { EXPAND_SIGN, get_sign_name, true, true },
    { EXPAND_PROFILE, get_profile_name, true, true },
    { EXPAND_LANGUAGE, get_lang_arg, true, false },
    { EXPAND_LOCALES, get_locales, true, false },
    { EXPAND_ENV_VARS, get_env_name, true, true },
    { EXPAND_USER, get_users, true, false },
    { EXPAND_ARGLIST, get_arglist_name, true, false },
    { EXPAND_BREAKPOINT, get_breakadd_arg, true, true },
    { EXPAND_SCRIPTNAMES, get_scriptnames_arg, true, false },
    { EXPAND_RETAB, get_retab_arg, true, true },
    { EXPAND_CHECKHEALTH, get_healthcheck_names, true, false },
  };
  int ret = FAIL;

  // Find a context in the table and call the ExpandGeneric() with the
  // right function to do the expansion.
  for (int i = 0; i < (int)ARRAY_SIZE(tab); i++) {
    if (xp->xp_context == tab[i].context) {
      if (tab[i].ic) {
        rmp->rm_ic = true;
      }
      ExpandGeneric(pat, xp, rmp, matches, numMatches, tab[i].func, tab[i].escaped);
      ret = OK;
      break;
    }
  }

  return ret;
}

/// Map wild expand options to flags for expand_wildcards(). Rust implementation.
static int map_wildopts_to_ewflags(int options)
{
  return rs_map_wildopts_to_ewflags(options);
}

/// Do the expansion based on xp->xp_context and "pat".
///
/// @param options  WILD_ flags
static int ExpandFromContext(expand_T *xp, char *pat, char ***matches, int *numMatches, int options)
{
  regmatch_T regmatch = { .rm_ic = false };
  int ret;
  int flags = map_wildopts_to_ewflags(options);
  const bool fuzzy = cmdline_fuzzy_complete(pat)
                     && cmdline_fuzzy_completion_supported(xp);

  if (xp->xp_context == EXPAND_FILES
      || xp->xp_context == EXPAND_DIRECTORIES
      || xp->xp_context == EXPAND_FILES_IN_PATH
      || xp->xp_context == EXPAND_FINDFUNC
      || xp->xp_context == EXPAND_DIRS_IN_CDPATH) {
    return expand_files_and_dirs(xp, pat, matches, numMatches, flags, options);
  }

  *matches = NULL;
  *numMatches = 0;
  if (xp->xp_context == EXPAND_HELP) {
    // With an empty argument we would get all the help tags, which is
    // very slow.  Get matches for "help" instead.
    if (find_help_tags(*pat == NUL ? "help" : pat,
                       numMatches, matches, false) == OK) {
      cleanup_help_tags(*numMatches, *matches);
      return OK;
    }
    return FAIL;
  }

  if (xp->xp_context == EXPAND_SHELLCMD) {
    expand_shellcmd(pat, matches, numMatches, flags);
    return OK;
  }
  if (xp->xp_context == EXPAND_OLD_SETTING) {
    return ExpandOldSetting(numMatches, matches);
  }
  if (xp->xp_context == EXPAND_BUFFERS) {
    return ExpandBufnames(pat, numMatches, matches, options);
  }
  if (xp->xp_context == EXPAND_DIFF_BUFFERS) {
    return ExpandBufnames(pat, numMatches, matches, options | BUF_DIFF_FILTER);
  }
  if (xp->xp_context == EXPAND_TAGS
      || xp->xp_context == EXPAND_TAGS_LISTFILES) {
    return rs_expand_tags(xp->xp_context == EXPAND_TAGS, pat, numMatches, matches);
  }
  if (xp->xp_context == EXPAND_COLORS) {
    char *directories[] = { "colors", NULL };
    return ExpandRTDir(pat, DIP_START + DIP_OPT, numMatches, matches, directories);
  }
  if (xp->xp_context == EXPAND_COMPILER) {
    char *directories[] = { "compiler", NULL };
    return ExpandRTDir(pat, 0, numMatches, matches, directories);
  }
  if (xp->xp_context == EXPAND_OWNSYNTAX) {
    char *directories[] = { "syntax", NULL };
    return ExpandRTDir(pat, 0, numMatches, matches, directories);
  }
  if (xp->xp_context == EXPAND_FILETYPE) {
    char *directories[] = { "syntax", "indent", "ftplugin", NULL };
    return ExpandRTDir(pat, 0, numMatches, matches, directories);
  }
  if (xp->xp_context == EXPAND_KEYMAP) {
    char *directories[] = { "keymap", NULL };
    return ExpandRTDir(pat, 0, numMatches, matches, directories);
  }
  if (xp->xp_context == EXPAND_USER_LIST) {
    return ExpandUserList(xp, matches, numMatches);
  }
  if (xp->xp_context == EXPAND_USER_LUA) {
    return ExpandUserLua(xp, numMatches, matches);
  }
  if (xp->xp_context == EXPAND_PACKADD) {
    return ExpandPackAddDir(pat, numMatches, matches);
  }
  if (xp->xp_context == EXPAND_RUNTIME) {
    return expand_runtime_cmd(pat, numMatches, matches);
  }
  if (xp->xp_context == EXPAND_PATTERN_IN_BUF) {
    return expand_pattern_in_buf(pat, xp->xp_search_dir, matches, numMatches);
  }

  // When expanding a function name starting with s:, match the <SNR>nr_
  // prefix.
  char *tofree = NULL;
  if (xp->xp_context == EXPAND_USER_FUNC && strncmp(pat, "^s:", 3) == 0) {
    const size_t len = strlen(pat) + 20;

    tofree = xmalloc(len);
    snprintf(tofree, len, "^<SNR>\\d\\+_%s", pat + 3);
    pat = tofree;
  }

  if (xp->xp_context == EXPAND_LUA) {
    return nlua_expand_get_matches(numMatches, matches);
  }

  if (!fuzzy) {
    regmatch.regprog = vim_regcomp(pat, rs_magic_isset() ? RE_MAGIC : 0);
    if (regmatch.regprog == NULL) {
      return FAIL;
    }

    // set ignore-case according to p_ic, p_scs and pat
    regmatch.rm_ic = ignorecase(pat);
  }

  if (xp->xp_context == EXPAND_SETTINGS
      || xp->xp_context == EXPAND_BOOL_SETTINGS) {
    ret = ExpandSettings(xp, &regmatch, pat, numMatches, matches, fuzzy);
  } else if (xp->xp_context == EXPAND_STRING_SETTING) {
    ret = ExpandStringSetting(xp, &regmatch, numMatches, matches);
  } else if (xp->xp_context == EXPAND_SETTING_SUBTRACT) {
    ret = rs_expand_setting_subtract(xp, &regmatch, numMatches, matches);
  } else if (xp->xp_context == EXPAND_MAPPINGS) {
    ret = ExpandMappings(pat, &regmatch, numMatches, matches);
  } else if (xp->xp_context == EXPAND_ARGOPT) {
    ret = expand_argopt(pat, xp, &regmatch, matches, numMatches);
  } else if (xp->xp_context == EXPAND_USER_DEFINED) {
    ret = ExpandUserDefined(pat, xp, &regmatch, matches, numMatches);
  } else {
    ret = ExpandOther(pat, xp, &regmatch, matches, numMatches);
  }

  if (!fuzzy) {
    vim_regfree(regmatch.regprog);
  }
  xfree(tofree);

  return ret;
}

/// Expand a list of names.
///
/// Generic function for command line completion.  It calls a function to
/// obtain strings, one by one.  The strings are matched against a regexp
/// program.  Matching strings are copied into an array, which is returned.
///
/// @param func  returns a string from the list
void ExpandGeneric(const char *const pat, expand_T *xp, regmatch_T *regmatch, char ***matches,
                   int *numMatches, CompleteListItemGetter func, bool escaped)
{
  const bool fuzzy = cmdline_fuzzy_complete(pat);
  *matches = NULL;
  *numMatches = 0;

  garray_T ga;
  if (!fuzzy) {
    ga_init(&ga, sizeof(char *), 30);
  } else {
    ga_init(&ga, sizeof(fuzmatch_str_T), 30);
  }

  for (int i = 0;; i++) {
    char *str = (*func)(xp, i);
    if (str == NULL) {  // End of list.
      break;
    }
    if (*str == NUL) {  // Skip empty strings.
      continue;
    }

    bool match;
    int score = 0;
    if (xp->xp_pattern[0] != NUL) {
      if (!fuzzy) {
        match = vim_regexec(regmatch, str, 0);
      } else {
        score = fuzzy_match_str(str, pat);
        match = (score != FUZZY_SCORE_NONE);
      }
    } else {
      match = true;
    }

    if (!match) {
      continue;
    }

    if (escaped) {
      str = vim_strsave_escaped(str, " \t\\.");
    } else {
      str = xstrdup(str);
    }

    if (fuzzy) {
      GA_APPEND(fuzmatch_str_T, &ga, ((fuzmatch_str_T){
        .idx = ga.ga_len,
        .str = str,
        .score = score,
      }));
    } else {
      GA_APPEND(char *, &ga, str);
    }

    if (func == get_menu_names) {
      // Test for separator added by get_menu_names().
      str += strlen(str) - 1;
      if (*str == '\001') {
        *str = '.';
      }
    }
  }

  if (ga.ga_len == 0) {
    return;
  }

  // Sort the matches when using regular expression matching and sorting
  // applies to the completion context. Menus and scriptnames should be kept
  // in the specified order.
  const bool sort_matches = !fuzzy
                            && xp->xp_context != EXPAND_MENUNAMES
                            && xp->xp_context != EXPAND_STRING_SETTING
                            && xp->xp_context != EXPAND_MENUS
                            && xp->xp_context != EXPAND_SCRIPTNAMES
                            && xp->xp_context != EXPAND_ARGOPT;

  // <SNR> functions should be sorted to the end.
  const bool funcsort = xp->xp_context == EXPAND_EXPRESSION
                        || xp->xp_context == EXPAND_FUNCTIONS
                        || xp->xp_context == EXPAND_USER_FUNC;

  // Sort the matches.
  if (sort_matches) {
    if (funcsort) {
      // <SNR> functions should be sorted to the end.
      qsort(ga.ga_data, (size_t)ga.ga_len, sizeof(char *), sort_func_compare);
    } else {
      sort_strings(ga.ga_data, ga.ga_len);
    }
  }

  if (!fuzzy) {
    *matches = ga.ga_data;
    *numMatches = ga.ga_len;
  } else {
    fuzzymatches_to_strmatches(ga.ga_data, matches, ga.ga_len, funcsort);
    *numMatches = ga.ga_len;
  }

  // Reset the variables used for special highlight names expansion, so that
  // they don't show up when getting normal highlight names by ID.
  reset_expand_highlight();
}

/// Expand shell command matches in one directory of $PATH.
///
/// @param pathed_pattern  fully pathed pattern
/// @param pathlen         length of the path portion of pathed_pattern (0 if no path)
static void expand_shellcmd_onedir(char *pathed_pattern, size_t pathlen, char ***matches,
                                   int *numMatches, int flags, hashtab_T *ht, garray_T *gap)
{
  // Expand matches in one directory of $PATH.
  if (expand_wildcards(1, &pathed_pattern, numMatches, matches, flags) != OK) {
    return;
  }

  ga_grow(gap, *numMatches);

  for (int i = 0; i < *numMatches; i++) {
    char *name = (*matches)[i];
    size_t namelen = strlen(name);

    if (namelen > pathlen) {
      // Check if this name was already found.
      hash_T hash = hash_hash(name + pathlen);
      hashitem_T *hi = hash_lookup(ht, name + pathlen, namelen - pathlen, hash);
      if (HASHITEM_EMPTY(hi)) {
        // Remove the path that was prepended.
        memmove(name, name + pathlen, namelen - pathlen + 1);  // +1 for NUL
        ((char **)gap->ga_data)[gap->ga_len++] = name;
        hash_add_item(ht, hi, name, hash);
        name = NULL;
      }
    }
    xfree(name);
  }
  xfree(*matches);
}

/// Complete a shell command.
///
/// @param      filepat     is a pattern to match with command names.
/// @param[out] matches     is pointer to array of pointers to matches.
///                         *matches will either be set to NULL or point to
///                         allocated memory.
/// @param[out] numMatches  is pointer to number of matches.
/// @param      flagsarg    is a combination of EW_* flags.
static void expand_shellcmd(char *filepat, char ***matches, int *numMatches, int flagsarg)
  FUNC_ATTR_NONNULL_ALL
{
  char *path = NULL;
  garray_T ga;
  char *buf = xmalloc(MAXPATHL);
  int flags = flagsarg;
  bool did_curdir = false;

  // for ":set path=" and ":set tags=" halve backslashes for escaped space
  size_t patlen = strlen(filepat);
  char *pat = xmemdupz(filepat, patlen);
  // Replace "\ " with " ".
  char *e = pat + patlen;
  for (char *s = pat; *s != NUL; s++) {
    if (*s != '\\') {
      continue;
    }
    char *p = s + 1;
    if (*p == ' ') {
      memmove(s, p, (size_t)(e - p) + 1);  // +1 for NUL
      e--;
    }
  }
  patlen = (size_t)(e - pat);

  flags |= EW_FILE | EW_EXEC | EW_SHELLCMD;

  bool mustfree = false;  // Track memory allocation for *path.
  if (pat[0] == '.' && (vim_ispathsep(pat[1])
                        || (pat[1] == '.' && vim_ispathsep(pat[2])))) {
    path = ".";
  } else {
    // For an absolute name we don't use $PATH.
    if (!path_is_absolute(pat)) {
      path = vim_getenv("PATH");
    }
    if (path == NULL) {
      path = "";
    } else {
      mustfree = true;
    }
  }

  // Go over all directories in $PATH.  Expand matches in that directory and
  // collect them in "ga". When "." is not in $PATH also expand for the
  // current directory, to find "subdir/cmd".
  ga_init(&ga, (int)sizeof(char *), 10);
  hashtab_T found_ht;
  hash_init(&found_ht);
  for (char *s = path;; s = e) {
    size_t pathlen;  // length of the path portion of buf (including trailing slash).
    size_t seplen;

    if (*s == NUL) {
      if (did_curdir) {
        break;
      }

      // Find directories in the current directory, path is empty.
      did_curdir = true;
      flags |= EW_DIR;

      e = s;
      pathlen = 0;
      seplen = 0;
    } else {
      e = vim_strchr(s, ENV_SEPCHAR);
      if (e == NULL) {
        e = s + strlen(s);
      }

      pathlen = (size_t)(e - s);
      if (strncmp(s, ".", pathlen) == 0) {
        did_curdir = true;
        flags |= EW_DIR;
      } else {
        // Do not match directories inside a $PATH item.
        flags &= ~EW_DIR;
      }

      seplen = !after_pathsep(s, e) ? STRLEN_LITERAL(PATHSEPSTR) : 0;
    }

    // Make sure that the pathed pattern (ie the path and pattern concatenated
    // together) will fit inside the buffer. If not skip it and move on to the
    // next path.
    if (pathlen + seplen + patlen + 1 <= MAXPATHL) {
      if (pathlen > 0) {
        xmemcpyz(buf, s, pathlen);
        if (seplen > 0) {
          xmemcpyz(buf + pathlen, S_LEN(PATHSEPSTR));
          pathlen += seplen;
        }
      }
      xmemcpyz(buf + pathlen, pat, patlen);

      expand_shellcmd_onedir(buf, pathlen, matches, numMatches, flags, &found_ht, &ga);
    }

    if (*e != NUL) {
      e++;
    }
  }
  *matches = ga.ga_data;
  *numMatches = ga.ga_len;

  xfree(buf);
  xfree(pat);
  if (mustfree) {
    xfree(path);
  }
  hash_clear(&found_ht);
}

/// Call "user_expand_func()" to invoke a user defined Vim script function and
/// return the result (either a string, a List or NULL).
static void *call_user_expand_func(user_expand_func_T user_expand_func, expand_T *xp)
  FUNC_ATTR_NONNULL_ALL
{
  CmdlineInfo *const ccline = get_cmdline_info();
  char keep = 0;
  typval_T args[4];
  const sctx_T save_current_sctx = current_sctx;

  if (xp->xp_arg == NULL || xp->xp_arg[0] == NUL || xp->xp_line == NULL) {
    return NULL;
  }

  if (ccline->cmdbuff != NULL) {
    keep = ccline->cmdbuff[ccline->cmdlen];
    ccline->cmdbuff[ccline->cmdlen] = 0;
  }

  char *pat = xstrnsave(xp->xp_pattern, xp->xp_pattern_len);
  args[0].v_type = VAR_STRING;
  args[1].v_type = VAR_STRING;
  args[2].v_type = VAR_NUMBER;
  args[3].v_type = VAR_UNKNOWN;
  args[0].vval.v_string = pat;
  args[1].vval.v_string = xp->xp_line;
  args[2].vval.v_number = xp->xp_col;

  current_sctx = xp->xp_script_ctx;

  void *const ret = user_expand_func(xp->xp_arg, 3, args);

  current_sctx = save_current_sctx;
  if (ccline->cmdbuff != NULL) {
    ccline->cmdbuff[ccline->cmdlen] = keep;
  }

  xfree(pat);
  return ret;
}

/// Expand names with a function defined by the user (EXPAND_USER_DEFINED and
/// EXPAND_USER_LIST).
static int ExpandUserDefined(const char *const pat, expand_T *xp, regmatch_T *regmatch,
                             char ***matches, int *numMatches)
{
  const bool fuzzy = cmdline_fuzzy_complete(pat);
  *matches = NULL;
  *numMatches = 0;

  char *const retstr = call_user_expand_func(call_func_retstr, xp);
  if (retstr == NULL) {
    return FAIL;
  }

  garray_T ga;
  if (!fuzzy) {
    ga_init(&ga, (int)sizeof(char *), 3);
  } else {
    ga_init(&ga, (int)sizeof(fuzmatch_str_T), 3);
  }

  for (char *s = retstr, *e; *s != NUL; s = e) {
    e = vim_strchr(s, '\n');
    if (e == NULL) {
      e = s + strlen(s);
    }
    const char keep = *e;
    *e = NUL;

    bool match;
    int score = 0;
    if (xp->xp_pattern[0] != NUL) {
      if (!fuzzy) {
        match = vim_regexec(regmatch, s, 0);
      } else {
        score = fuzzy_match_str(s, pat);
        match = (score != FUZZY_SCORE_NONE);
      }
    } else {
      match = true;               // match everything
    }

    *e = keep;

    if (match) {
      if (!fuzzy) {
        GA_APPEND(char *, &ga, xmemdupz(s, (size_t)(e - s)));
      } else {
        GA_APPEND(fuzmatch_str_T, &ga, ((fuzmatch_str_T){
          .idx = ga.ga_len,
          .str = xmemdupz(s, (size_t)(e - s)),
          .score = score,
        }));
      }
    }

    if (*e != NUL) {
      e++;
    }
  }
  xfree(retstr);

  if (ga.ga_len == 0) {
    return OK;
  }

  if (!fuzzy) {
    *matches = ga.ga_data;
    *numMatches = ga.ga_len;
  } else {
    fuzzymatches_to_strmatches(ga.ga_data, matches, ga.ga_len, false);
    *numMatches = ga.ga_len;
  }
  return OK;
}

/// Expand names with a list returned by a function defined by the user.
static int ExpandUserList(expand_T *xp, char ***matches, int *numMatches)
{
  *matches = NULL;
  *numMatches = 0;
  list_T *const retlist = call_user_expand_func(call_func_retlist, xp);
  if (retlist == NULL) {
    return FAIL;
  }

  garray_T ga;
  ga_init(&ga, (int)sizeof(char *), 3);
  // Loop over the items in the list.
  TV_LIST_ITER_CONST(retlist, li, {
    if (TV_LIST_ITEM_TV(li)->v_type != VAR_STRING
        || TV_LIST_ITEM_TV(li)->vval.v_string == NULL) {
      continue;  // Skip non-string items and empty strings.
    }

    GA_APPEND(char *, &ga, xstrdup(TV_LIST_ITEM_TV(li)->vval.v_string));
  });
  tv_list_unref(retlist);

  *matches = ga.ga_data;
  *numMatches = ga.ga_len;
  return OK;
}

static int ExpandUserLua(expand_T *xp, int *num_file, char ***file)
{
  typval_T rettv = TV_INITIAL_VALUE;
  nlua_call_user_expand_func(xp, &rettv);
  if (rettv.v_type != VAR_LIST) {
    tv_clear(&rettv);
    return FAIL;
  }

  list_T *const retlist = rettv.vval.v_list;

  garray_T ga;
  ga_init(&ga, (int)sizeof(char *), 3);
  // Loop over the items in the list.
  TV_LIST_ITER_CONST(retlist, li, {
    if (TV_LIST_ITEM_TV(li)->v_type != VAR_STRING
        || TV_LIST_ITEM_TV(li)->vval.v_string == NULL) {
      continue;  // Skip non-string items and empty strings.
    }

    GA_APPEND(char *, &ga, xstrdup(TV_LIST_ITEM_TV(li)->vval.v_string));
  });
  tv_list_unref(retlist);

  *file = ga.ga_data;
  *num_file = ga.ga_len;
  return OK;
}

/// Expand `file` for all comma-separated directories in `path`.
/// Adds matches to `ga`.
/// If "dirs" is true only expand directory names.
void globpath(char *path, char *file, garray_T *ga, int expand_options, bool dirs)
  FUNC_ATTR_NONNULL_ALL
{
  char *buf = xmalloc(MAXPATHL);

  expand_T xpc;
  ExpandInit(&xpc);
  xpc.xp_context = dirs ? EXPAND_DIRECTORIES : EXPAND_FILES;

  size_t filelen = strlen(file);

#if defined(MSWIN)
  // Using the platform's path separator (\) makes vim incorrectly
  // treat it as an escape character, use '/' instead.
# define TMP_PATHSEPSTR "/"
#else
# define TMP_PATHSEPSTR PATHSEPSTR
#endif

  // Loop over all entries in {path}.
  while (*path != NUL) {
    // Copy one item of the path to buf[] and concatenate the file name.

    // length of the path portion of buf (including trailing slash).
    size_t pathlen = copy_option_part(&path, buf, MAXPATHL, ",");
    size_t seplen = (*buf != NUL && !after_pathsep(buf, buf + pathlen))
                    ? STRLEN_LITERAL(TMP_PATHSEPSTR) : 0;

    if (pathlen + seplen + filelen + 1 <= MAXPATHL) {
      if (seplen > 0) {
        xmemcpyz(buf + pathlen, S_LEN(TMP_PATHSEPSTR));
        pathlen += seplen;
      }
      xmemcpyz(buf + pathlen, file, filelen);

      char **p;
      int num_p = 0;
      ExpandFromContext(&xpc, buf, &p, &num_p, WILD_SILENT | expand_options);
      if (num_p > 0) {
        ExpandEscape(&xpc, buf, num_p, p, WILD_SILENT | expand_options);

        // Concatenate new results to previous ones.
        ga_grow(ga, num_p);
        // take over the pointers and put them in "ga"
        for (int i = 0; i < num_p; i++) {
          ((char **)ga->ga_data)[ga->ga_len] = p[i];
          ga->ga_len++;
        }
        xfree(p);
      }
    }
  }

  xfree(buf);
}
#undef TMP_PATHSEPSTR

/// Translate some keys pressed when 'wildmenu' is used.
int wildmenu_translate_key(CmdlineInfo *cclp, int key, expand_T *xp, bool did_wild_list)
{
  int c = key;

  if (cmdline_pum_active() || did_wild_list || wild_menu_showing) {
    if (c == K_LEFT) {
      c = Ctrl_P;
    } else if (c == K_RIGHT) {
      c = Ctrl_N;
    }
  }

  // Hitting CR after "emenu Name.": complete submenu
  if (xp->xp_context == EXPAND_MENUNAMES
      && cclp->cmdpos > 1
      && cclp->cmdbuff[cclp->cmdpos - 1] == '.'
      && cclp->cmdbuff[cclp->cmdpos - 2] != '\\'
      && (c == '\n' || c == '\r' || c == K_KENTER)) {
    c = K_DOWN;
  }

  return c;
}

/// Delete characters on the command line, from "from" to the current position.
static void cmdline_del(CmdlineInfo *cclp, int from)
{
  assert(cclp->cmdpos <= cclp->cmdlen);
  memmove(cclp->cmdbuff + from, cclp->cmdbuff + cclp->cmdpos,
          (size_t)cclp->cmdlen - (size_t)cclp->cmdpos + 1);
  cclp->cmdlen -= cclp->cmdpos - from;
  cclp->cmdpos = from;
}

/// Handle a key pressed when the wild menu for the menu names
/// (EXPAND_MENUNAMES) is displayed.
static int wildmenu_process_key_menunames(CmdlineInfo *cclp, int key, expand_T *xp)
{
  // Hitting <Down> after "emenu Name.": complete submenu
  if (key == K_DOWN && cclp->cmdpos > 0
      && cclp->cmdbuff[cclp->cmdpos - 1] == '.') {
    key = (int)p_wc;
    KeyTyped = true;  // in case the key was mapped
  } else if (key == K_UP) {
    // Hitting <Up>: Remove one submenu name in front of the
    // cursor
    bool found = false;

    int j = (int)(xp->xp_pattern - cclp->cmdbuff);
    int i = 0;
    while (--j > 0) {
      // check for start of menu name
      if (cclp->cmdbuff[j] == ' '
          && cclp->cmdbuff[j - 1] != '\\') {
        i = j + 1;
        break;
      }
      // check for start of submenu name
      if (cclp->cmdbuff[j] == '.'
          && cclp->cmdbuff[j - 1] != '\\') {
        if (found) {
          i = j + 1;
          break;
        } else {
          found = true;
        }
      }
    }
    if (i > 0) {
      cmdline_del(cclp, i);
    }
    key = (int)p_wc;
    KeyTyped = true;  // in case the key was mapped
    xp->xp_context = EXPAND_NOTHING;
  }

  return key;
}

/// Handle a key pressed when the wild menu for file names (EXPAND_FILES) or
/// directory names (EXPAND_DIRECTORIES) or shell command names
/// (EXPAND_SHELLCMD) is displayed.
static int wildmenu_process_key_filenames(CmdlineInfo *cclp, int key, expand_T *xp)
{
  char upseg[5];
  upseg[0] = PATHSEP;
  upseg[1] = '.';
  upseg[2] = '.';
  upseg[3] = PATHSEP;
  upseg[4] = NUL;

  if (key == K_DOWN
      && cclp->cmdpos > 0
      && cclp->cmdbuff[cclp->cmdpos - 1] == PATHSEP
      && (cclp->cmdpos < 3
          || cclp->cmdbuff[cclp->cmdpos - 2] != '.'
          || cclp->cmdbuff[cclp->cmdpos - 3] != '.')) {
    // go down a directory
    key = (int)p_wc;
    KeyTyped = true;  // in case the key was mapped
  } else if (strncmp(xp->xp_pattern, upseg + 1, 3) == 0 && key == K_DOWN) {
    // If in a direct ancestor, strip off one ../ to go down
    bool found = false;

    int j = cclp->cmdpos;
    int i = (int)(xp->xp_pattern - cclp->cmdbuff);
    while (--j > i) {
      j -= utf_head_off(cclp->cmdbuff, cclp->cmdbuff + j);
      if (vim_ispathsep(cclp->cmdbuff[j])) {
        found = true;
        break;
      }
    }
    if (found
        && cclp->cmdbuff[j - 1] == '.'
        && cclp->cmdbuff[j - 2] == '.'
        && (vim_ispathsep(cclp->cmdbuff[j - 3]) || j == i + 2)) {
      cmdline_del(cclp, j - 2);
      key = (int)p_wc;
      KeyTyped = true;  // in case the key was mapped
    }
  } else if (key == K_UP) {
    // go up a directory
    bool found = false;

    int j = cclp->cmdpos - 1;
    int i = (int)(xp->xp_pattern - cclp->cmdbuff);
    while (--j > i) {
      j -= utf_head_off(cclp->cmdbuff, cclp->cmdbuff + j);
      if (vim_ispathsep(cclp->cmdbuff[j])
#ifdef BACKSLASH_IN_FILENAME
          && vim_strchr(" *?[{`$%#", (uint8_t)cclp->cmdbuff[j + 1]) == NULL
#endif
          ) {
        if (found) {
          i = j + 1;
          break;
        } else {
          found = true;
        }
      }
    }

    if (!found) {
      j = i;
    } else if (strncmp(cclp->cmdbuff + j, upseg, 4) == 0) {
      j += 4;
    } else if (strncmp(cclp->cmdbuff + j, upseg + 1, 3) == 0
               && j == i) {
      j += 3;
    } else {
      j = 0;
    }

    if (j > 0) {
      // TODO(tarruda): this is only for DOS/Unix systems - need to put in
      // machine-specific stuff here and in upseg init
      cmdline_del(cclp, j);
      put_on_cmdline(upseg + 1, 3, false);
    } else if (cclp->cmdpos > i) {
      cmdline_del(cclp, i);
    }

    // Now complete in the new directory. Set KeyTyped in case the
    // Up key came from a mapping.
    key = (int)p_wc;
    KeyTyped = true;
  }

  return key;
}

/// Handle a key pressed when wild menu is displayed
int wildmenu_process_key(CmdlineInfo *cclp, int key, expand_T *xp)
{
  // Special translations for 'wildmenu'
  if (xp->xp_context == EXPAND_MENUNAMES) {
    return wildmenu_process_key_menunames(cclp, key, xp);
  }
  if (xp->xp_context == EXPAND_FILES
      || xp->xp_context == EXPAND_DIRECTORIES
      || xp->xp_context == EXPAND_SHELLCMD) {
    return wildmenu_process_key_filenames(cclp, key, xp);
  }

  return key;
}

/// Free expanded names when finished walking through the matches
void wildmenu_cleanup(CmdlineInfo *cclp)
{
  if (!p_wmnu || wild_menu_showing == 0) {
    return;
  }

  const bool skt = KeyTyped;
  const int old_RedrawingDisabled = RedrawingDisabled;

  if (cclp->input_fn) {
    RedrawingDisabled = 0;
  }

  // Clear highlighting applied during wildmenu activity
  set_no_hlsearch(true);

  if (wild_menu_showing == WM_SCROLLED) {
    // Entered command line, move it up
    cmdline_row--;
    redrawcmd();
    wild_menu_showing = 0;
  } else if (save_p_ls != -1) {
    // restore 'laststatus' and 'winminheight'
    p_ls = save_p_ls;
    p_wmh = save_p_wmh;
    rs_last_status(0);
    update_screen();  // redraw the screen NOW
    redrawcmd();
    save_p_ls = -1;
    wild_menu_showing = 0;
  } else {
    win_redraw_last_status(topframe);
    wild_menu_showing = 0;  // must be before redraw_statuslines #8385
    redraw_statuslines();
  }
  KeyTyped = skt;
  if (cclp->input_fn) {
    RedrawingDisabled = old_RedrawingDisabled;
  }
}

/// "getcompletion()" function
void f_getcompletion(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  expand_T xpc;
  bool filtered = false;
  int options = WILD_SILENT | WILD_USE_NL | WILD_ADD_SLASH
                | WILD_NO_BEEP | WILD_HOME_REPLACE;

  if (tv_check_for_string_arg(argvars, 1) == FAIL) {
    return;
  }
  const char *const type = tv_get_string(&argvars[1]);

  if (argvars[2].v_type != VAR_UNKNOWN) {
    filtered = (bool)tv_get_number_chk(&argvars[2], NULL);
  }

  if (p_wic) {
    options |= WILD_ICASE;
  }

  // For filtered results, 'wildignore' is used
  if (!filtered) {
    options |= WILD_KEEP_ALL;
  }

  if (argvars[0].v_type != VAR_STRING) {
    emsg(_(e_invarg));
    return;
  }
  const char *const pattern = tv_get_string(&argvars[0]);
  const char *pattern_start = pattern;

  if (strcmp(type, "cmdline") == 0) {
    const int cmdline_len = (int)strlen(pattern);
    set_cmd_context(&xpc, (char *)pattern, cmdline_len, cmdline_len, false);
    pattern_start = xpc.xp_pattern;
    xpc.xp_pattern_len = strlen(xpc.xp_pattern);
    xpc.xp_col = cmdline_len;
    goto theend;
  }

  ExpandInit(&xpc);
  xpc.xp_pattern = (char *)pattern;
  xpc.xp_pattern_len = strlen(xpc.xp_pattern);
  xpc.xp_line = (char *)pattern;

  xpc.xp_context = cmdcomplete_str_to_type(type);
  switch (xpc.xp_context) {
  case EXPAND_NOTHING:
    semsg(_(e_invarg2), type);
    return;

  case EXPAND_USER_DEFINED:
    // Must be "custom,funcname" pattern
    if (strncmp(type, "custom,", 7) != 0) {
      semsg(_(e_invarg2), type);
      return;
    }

    xpc.xp_arg = (char *)(type + 7);
    break;

  case EXPAND_USER_LIST:
    // Must be "customlist,funcname" pattern
    if (strncmp(type, "customlist,", 11) != 0) {
      semsg(_(e_invarg2), type);
      return;
    }

    xpc.xp_arg = (char *)(type + 11);
    break;

  case EXPAND_MENUS:
    set_context_in_menu_cmd(&xpc, "menu", xpc.xp_pattern, false);
    xpc.xp_pattern_len -= (size_t)(xpc.xp_pattern - pattern_start);
    break;

  case EXPAND_SIGN:
    set_context_in_sign_cmd(&xpc, xpc.xp_pattern);
    xpc.xp_pattern_len -= (size_t)(xpc.xp_pattern - pattern_start);
    break;

  case EXPAND_RUNTIME:
    set_context_in_runtime_cmd(&xpc, xpc.xp_pattern);
    xpc.xp_pattern_len -= (size_t)(xpc.xp_pattern - pattern_start);
    break;

  case EXPAND_SHELLCMDLINE: {
    int context = EXPAND_SHELLCMDLINE;
    set_context_for_wildcard_arg(NULL, xpc.xp_pattern, false, &xpc, &context);
    xpc.xp_pattern_len -= (size_t)(xpc.xp_pattern - pattern_start);
    break;
  }

  case EXPAND_FILETYPECMD:
    filetype_expand_what = EXP_FILETYPECMD_ALL;
    break;

  default:
    break;
  }

theend:
  if (xpc.xp_context == EXPAND_LUA) {
    xpc.xp_col = (int)strlen(xpc.xp_line);
    nlua_expand_pat(&xpc);
    xpc.xp_pattern_len -= (size_t)(xpc.xp_pattern - pattern_start);
  }
  char *pat;
  if (cmdline_fuzzy_completion_supported(&xpc)) {
    // when fuzzy matching, don't modify the search string
    pat = xmemdupz(xpc.xp_pattern, xpc.xp_pattern_len);
  } else {
    pat = addstar(xpc.xp_pattern, xpc.xp_pattern_len, xpc.xp_context);
  }

  ExpandOne(&xpc, pat, NULL, options, WILD_ALL_KEEP);
  tv_list_alloc_ret(rettv, xpc.xp_numfiles);

  for (int i = 0; i < xpc.xp_numfiles; i++) {
    tv_list_append_string(rettv->vval.v_list, xpc.xp_files[i], -1);
  }
  xfree(pat);
  ExpandCleanup(&xpc);
}

/// "getcompletiontype()" function
void f_getcompletiontype(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = NULL;

  if (tv_check_for_string_arg(argvars, 0) == FAIL) {
    return;
  }

  const char *pat = tv_get_string(&argvars[0]);
  expand_T xpc;
  ExpandInit(&xpc);

  int cmdline_len = (int)strlen(pat);
  set_cmd_context(&xpc, (char *)pat, cmdline_len, cmdline_len, false);
  rettv->vval.v_string = cmdcomplete_type_to_str(xpc.xp_context, xpc.xp_arg);

  ExpandCleanup(&xpc);
}

/// "cmdcomplete_info()" function
void f_cmdcomplete_info(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  CmdlineInfo *ccline = get_cmdline_info();

  tv_dict_alloc_ret(rettv);
  if (ccline == NULL || ccline->xpc == NULL || ccline->xpc->xp_files == NULL) {
    return;
  }

  dict_T *retdict = rettv->vval.v_dict;
  int ret = tv_dict_add_str(retdict, S_LEN("cmdline_orig"), cmdline_orig);
  if (ret == OK) {
    ret = tv_dict_add_nr(retdict, S_LEN("pum_visible"), pum_visible());
  }
  if (ret == OK) {
    ret = tv_dict_add_nr(retdict, S_LEN("selected"), ccline->xpc->xp_selected);
  }
  if (ret == OK) {
    list_T *li = tv_list_alloc(ccline->xpc->xp_numfiles);
    ret = tv_dict_add_list(retdict, S_LEN("matches"), li);
    for (int idx = 0; ret == OK && idx < ccline->xpc->xp_numfiles; idx++) {
      tv_list_append_string(li, ccline->xpc->xp_files[idx], -1);
    }
  }
}

/// Copy a substring from the current buffer (curbuf), spanning from the given
/// 'start' position to the word boundary after 'end' position.
/// The copied string is stored in '*match', and the actual end position of the
/// matched text is returned in '*match_end'.
/// Rust implementation.
static int copy_substring_from_pos(pos_T *start, pos_T *end, char **match, pos_T *match_end)
{
  return rs_copy_substring_from_pos(start, end, match, match_end);
}

/// Returns true if the given string `str` matches the regex pattern `pat`.
/// Honors the 'ignorecase' (p_ic) and 'smartcase' (p_scs) settings to determine
/// case sensitivity.
/// Rust implementation.
static bool is_regex_match(char *pat, char *str)
{
  return rs_is_regex_match(pat, str) != 0;
}

/// Constructs a new match string by appending text from the buffer (starting at
/// end_match_pos) to the given pattern `pat`. The result is a concatenation of
/// `pat` and the word following end_match_pos.
/// If 'lowercase' is true, the appended text is converted to lowercase before
/// being combined. Returns the newly allocated match string, or NULL on failure.
/// Rust implementation.
static char *concat_pattern_with_buffer_match(char *pat, int pat_len, pos_T *end_match_pos,
                                              bool lowercase)
  FUNC_ATTR_NONNULL_RET
{
  return rs_concat_pattern_with_buffer_match(pat, pat_len, end_match_pos, lowercase);
}

/// Search for strings matching "pat" in the specified range and return them.
/// Returns OK on success, FAIL otherwise.
///
/// @param      pat        pattern to match
/// @param      dir        FORWARD or BACKWARD
/// @param[out] matches    array with matched string
/// @param[out] numMatches number of matches
static int expand_pattern_in_buf(char *pat, Direction dir, char ***matches, int *numMatches)
{
  bool exacttext = wop_flags & kOptWopFlagExacttext;
  bool has_range = search_first_line != 0;

  *matches = NULL;
  *numMatches = 0;

  if (pat == NULL || *pat == NUL) {
    return FAIL;
  }

  int pat_len = (int)strlen(pat);
  pos_T cur_match_pos = { 0 }, prev_match_pos = { 0 };
  if (has_range) {
    cur_match_pos.lnum = search_first_line;
  } else {
    cur_match_pos = pre_incsearch_pos;
  }

  int search_flags = SEARCH_OPT | SEARCH_NOOF | SEARCH_PEEK | SEARCH_NFMSG
                     | (has_range ? SEARCH_START : 0);

  garray_T ga;
  ga_init(&ga, sizeof(char *), 10);  // Use growable array of char *

  pos_T end_match_pos, word_end_pos;
  bool looped_around = false;
  bool compl_started = false;
  char *match, *full_match;

  while (true) {
    emsg_off++;
    msg_silent++;
    int found_new_match = searchit(NULL, curbuf, &cur_match_pos,
                                   &end_match_pos, dir, pat, (size_t)pat_len, 1L,
                                   search_flags, RE_LAST, NULL);
    msg_silent--;
    emsg_off--;

    if (found_new_match == FAIL) {
      break;
    }

    // If in range mode, check if match is within the range
    if (has_range && (cur_match_pos.lnum < search_first_line
                      || cur_match_pos.lnum > search_last_line)) {
      break;
    }

    if (compl_started) {
      // If we've looped back to an earlier match, stop
      if ((dir == FORWARD && ltoreq(cur_match_pos, prev_match_pos))
          || (dir == BACKWARD && ltoreq(prev_match_pos, cur_match_pos))) {
        if (looped_around) {
          break;
        } else {
          looped_around = true;
        }
      }
    }

    compl_started = true;
    prev_match_pos = cur_match_pos;

    // Abort if user typed a character or interrupted
    if (char_avail() || got_int) {
      if (got_int) {
        (void)vpeekc();  // Remove <C-C> from input stream
        got_int = false;  // Don't abandon the command line
      }
      goto cleanup;
    }

    // searchit() can return line number +1 past the last line when
    // searching for "foo\n" if "foo" is at end of buffer.
    if (end_match_pos.lnum > curbuf->b_ml.ml_line_count) {
      cur_match_pos.lnum = 1;
      cur_match_pos.col = 0;
      cur_match_pos.coladd = 0;
      continue;
    }

    // Extract the matching text prepended to completed word
    if (!copy_substring_from_pos(&cur_match_pos, &end_match_pos, &full_match,
                                 &word_end_pos)) {
      break;
    }

    if (exacttext) {
      match = full_match;
    } else {
      // Construct a new match from completed word appended to pattern itself
      match = concat_pattern_with_buffer_match(pat, pat_len, &end_match_pos, false);

      // The regex pattern may include '\C' or '\c'. First, try matching the
      // buffer word as-is. If it doesn't match, try again with the lowercase
      // version of the word to handle smartcase behavior.
      if (!is_regex_match(match, full_match)) {
        xfree(match);
        match = concat_pattern_with_buffer_match(pat, pat_len, &end_match_pos, true);
        if (!is_regex_match(match, full_match)) {
          xfree(match);
          xfree(full_match);
          continue;
        }
      }
      xfree(full_match);
    }

    // Include this match if it is not a duplicate
    for (int i = 0; i < ga.ga_len; i++) {
      if (strcmp(match, ((char **)ga.ga_data)[i]) == 0) {
        XFREE_CLEAR(match);
        break;
      }
    }
    if (match != NULL) {
      ga_grow(&ga, 1);
      ((char **)ga.ga_data)[ga.ga_len++] = match;
      if (ga.ga_len > TAG_MANY) {
        break;
      }
    }
    if (has_range) {
      cur_match_pos = word_end_pos;
    }
  }

  *matches = (char **)ga.ga_data;
  *numMatches = ga.ga_len;
  return OK;

cleanup:
  ga_clear_strings(&ga);
  return FAIL;
}
