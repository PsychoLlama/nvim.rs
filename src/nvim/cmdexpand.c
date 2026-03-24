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

// Rust FFI: fuzzy completion support check (takes context int, not expand_T*)
extern int rs_cmdline_fuzzy_completion_supported(int context);

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


// Phase 1: Leaf utility functions from Rust
extern int rs_sort_func_compare(const void *s1, const void *s2);
extern int rs_cmdline_compl_use_pum(int need_wildmenu);
extern int rs_map_wildopts_to_ewflags(int options);
extern char *rs_showmatches_gettail(char *s, int eager);
extern int rs_expand_showtail(expand_T *xp);
extern void rs_expand_escape(expand_T *xp, char *str, int numfiles, char **files, int options);


// Phase 3: Match navigation
extern char *rs_find_longest_match(expand_T *xp, int options);


// Phase 5: Context-setting helpers (used from set_one_cmd_context)
extern const char *rs_set_context_in_argopt(expand_T *xp, const char *arg);
extern void rs_set_context_for_wildcard_arg(const char *arg, int is_shell_cmd,
                                            expand_T *xp, int *complp);

// Phase 6: Callback generators
extern char *rs_get_filetypecmd_arg(expand_T *xp, int idx);
extern char *rs_get_breakadd_arg(expand_T *xp, int idx);
extern char *rs_get_retab_arg(expand_T *xp, int idx);
extern char *rs_get_messages_arg(expand_T *xp, int idx);
extern char *rs_get_mapclear_arg(expand_T *xp, int idx);
extern char *rs_get_scriptnames_arg(expand_T *xp, int idx);

// Wave 4: Wildmenu display helpers
extern int rs_skip_wildmenu_char(expand_T *xp, const char *s);
extern int rs_wildmenu_match_len(expand_T *xp, char *s);

// Wave 4: Pattern-in-buffer helpers
extern int rs_expand_pattern_in_buf(char *pat, int dir, char ***matches, int *numMatches);
extern int rs_expand_files_and_dirs(expand_T *xp, char *pat, char ***matches, int *numMatches,
                                    int flags, int options);
extern const char *rs_set_context_by_cmdname(const char *cmd, int cmdidx, expand_T *xp,
                                              const char *arg, uint32_t argt, int context,
                                              bool forceit);
extern const char *rs_set_one_cmd_context(expand_T *xp, const char *buff);

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

/// C accessor for may_expand_pattern (used by Rust rs_set_context_by_cmdname).
int nvim_cmdexpand_get_may_expand_pattern(void)
{
  return may_expand_pattern ? 1 : 0;
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

/// Set pum_want.active (for Rust FFI).
void nvim_set_pum_want_active(int val)
{
  pum_want.active = (val != 0);
}


// =============================================================================
// Phase 1: C accessors for Rust leaf utility functions
// =============================================================================

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

// =============================================================================
// Phase 3: C accessors for match navigation
// =============================================================================


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
  rs_expand_escape(xp, (char *)str, xp->xp_numfiles, xp->xp_files, options);
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

// Layout validation for expand_T repr(C) struct in Rust
_Static_assert(sizeof(expand_T) == 392, "expand_T size mismatch");
_Static_assert(offsetof(expand_T, xp_pattern) == 0, "xp_pattern offset mismatch");
_Static_assert(offsetof(expand_T, xp_context) == 8, "xp_context offset mismatch");
_Static_assert(offsetof(expand_T, xp_pattern_len) == 16, "xp_pattern_len offset mismatch");
_Static_assert(offsetof(expand_T, xp_prefix) == 24, "xp_prefix offset mismatch");
_Static_assert(offsetof(expand_T, xp_arg) == 32, "xp_arg offset mismatch");
_Static_assert(offsetof(expand_T, xp_luaref) == 40, "xp_luaref offset mismatch");
_Static_assert(offsetof(expand_T, xp_script_ctx) == 48, "xp_script_ctx offset mismatch");
_Static_assert(offsetof(expand_T, xp_backslash) == 72, "xp_backslash offset mismatch");
_Static_assert(offsetof(expand_T, xp_numfiles) == 80, "xp_numfiles offset mismatch");
_Static_assert(offsetof(expand_T, xp_col) == 84, "xp_col offset mismatch");
_Static_assert(offsetof(expand_T, xp_selected) == 88, "xp_selected offset mismatch");
_Static_assert(offsetof(expand_T, xp_orig) == 96, "xp_orig offset mismatch");
_Static_assert(offsetof(expand_T, xp_files) == 104, "xp_files offset mismatch");
_Static_assert(offsetof(expand_T, xp_line) == 112, "xp_line offset mismatch");
_Static_assert(offsetof(expand_T, xp_buf) == 120, "xp_buf offset mismatch");
_Static_assert(offsetof(expand_T, xp_search_dir) == 376, "xp_search_dir offset mismatch");
_Static_assert(offsetof(expand_T, xp_pre_incsearch_pos) == 380, "xp_pre_incsearch_pos offset mismatch");
#ifndef BACKSLASH_IN_FILENAME
_Static_assert(offsetof(expand_T, xp_shell) == 76, "xp_shell offset mismatch");
#endif

// =============================================================================
// Phase 4: C accessors for ExpandOne orchestrator
// =============================================================================


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


/// xstpcpy wrapper (for Rust FFI): copies src to dst, returns pointer past NUL.
char *nvim_cmdexpand_xstpcpy(char *dst, const char *src)
{
  return xstpcpy(dst, src);
}

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


// =============================================================================
// Phase 2: C accessors for expand_pattern_in_buf
// =============================================================================

/// Wrapper for searchit(NULL, curbuf, ...) for Rust FFI.
/// Returns FAIL or OK.
int nvim_cmdexpand_searchit(pos_T *pos, pos_T *end_pos, int dir, char *pat,
                            size_t patlen, int options)
{
  return searchit(NULL, curbuf, pos, end_pos, (Direction)dir, pat, patlen,
                  1L, options, RE_LAST, NULL);
}

/// Get curbuf->b_ml.ml_line_count (for Rust FFI).
int nvim_cmdexpand_curbuf_line_count(void)
{
  return curbuf->b_ml.ml_line_count;
}

/// Wrapper for char_avail() (for Rust FFI).
int nvim_cmdexpand_char_avail(void)
{
  return char_avail();
}

/// Wrapper for vpeekc() (for Rust FFI).
int nvim_cmdexpand_vpeekc(void)
{
  return vpeekc();
}

/// Get search_first_line (for Rust FFI).
int nvim_cmdexpand_get_search_first_line(void)
{
  return search_first_line;
}

/// Get search_last_line (for Rust FFI).
int nvim_cmdexpand_get_search_last_line(void)
{
  return search_last_line;
}

/// Get pre_incsearch_pos (for Rust FFI).
pos_T nvim_cmdexpand_get_pre_incsearch_pos(void)
{
  return pre_incsearch_pos;
}

// =============================================================================
// Phase 1: C accessors for PUM, wildmenu, scriptnames, and set_expand_context
// =============================================================================

/// Run pum_display with the current compl_* statics (for Rust FFI).
void nvim_cmdexpand_do_pum_display(int changed_array)
{
  pum_display(compl_match_array, compl_match_arraysize, compl_selected,
              changed_array != 0, compl_startcol);
}

/// Run pum_undisplay + free compl_match_array + reset arraysize (for Rust FFI).
void nvim_cmdexpand_do_pum_remove(int defer_redraw)
{
  pum_undisplay(defer_redraw == 0);
  XFREE_CLEAR(compl_match_array);
  compl_match_arraysize = 0;
}

/// Run cmdline_pum_remove(false) + wildmenu_cleanup(get_cmdline_info()) (for Rust FFI).
void nvim_cmdexpand_do_pum_cleanup(void)
{
  cmdline_pum_remove(false);
  wildmenu_cleanup(get_cmdline_info());
}

/// Return get_cmdline_info()->xpc->xp_orig or NULL if xpc is NULL (for Rust FFI).
char *nvim_cmdexpand_get_compl_pattern(void)
{
  expand_T *xp = get_cmdline_info()->xpc;
  return xp == NULL ? NULL : xp->xp_orig;
}

/// Return get_cmdline_info()->xpc if non-null and context supports fuzzy, else 0.
int nvim_cmdexpand_ccline_xpc_supports_fuzzy(void)
{
  expand_T *xp = get_cmdline_info()->xpc;
  return xp != NULL && rs_cmdline_fuzzy_completion_supported(xp->xp_context);
}

/// Return xp_context of get_cmdline_info()->xpc, or EXPAND_NOTHING if xpc is NULL.
int nvim_cmdexpand_ccline_xpc_context(void)
{
  expand_T *xp = get_cmdline_info()->xpc;
  return xp ? xp->xp_context : EXPAND_NOTHING;
}

/// Return the xpc pointer from get_cmdline_info() (for Rust FFI).
expand_T *nvim_cmdexpand_get_ccline_xpc(void)
{
  return get_cmdline_info()->xpc;
}

/// Get cmdfirstc from get_cmdline_info() (for Rust FFI).
int nvim_cmdexpand_get_cmdfirstc(void)
{
  return get_cmdline_info()->cmdfirstc;
}

/// Get input_fn from get_cmdline_info() (for Rust FFI).
int nvim_cmdexpand_get_input_fn(void)
{
  return get_cmdline_info()->input_fn ? 1 : 0;
}

/// Get cmdlen from get_cmdline_info() (for Rust FFI).
int nvim_cmdexpand_get_cmdlen(void)
{
  return get_cmdline_info()->cmdlen;
}

/// Get p_wc wildchar option value (for Rust FFI).
int nvim_cmdexpand_get_p_wc(void)
{
  return (int)p_wc;
}

/// Set search_first_line (for Rust FFI).
void nvim_cmdexpand_set_search_first_line(int val)
{
  search_first_line = val;
}

/// Get K_LEFT key code (for Rust FFI).
int nvim_cmdexpand_get_key_left(void)
{
  return K_LEFT;
}

/// Get K_RIGHT key code (for Rust FFI).
int nvim_cmdexpand_get_key_right(void)
{
  return K_RIGHT;
}

/// Get K_DOWN key code (for Rust FFI).
int nvim_cmdexpand_get_key_down(void)
{
  return K_DOWN;
}

/// Get K_UP key code (for Rust FFI).
int nvim_cmdexpand_get_key_up(void)
{
  return K_UP;
}

/// Get K_KENTER key code (for Rust FFI).
int nvim_cmdexpand_get_key_kenter(void)
{
  return K_KENTER;
}


/// Check SCRIPT_ID_VALID(idx+1) (for Rust FFI).
int nvim_cmdexpand_script_id_valid(int idx)
{
  return SCRIPT_ID_VALID(idx + 1) ? 1 : 0;
}

/// Get home_replace()-processed script name into NameBuff and return it (for Rust FFI).
char *nvim_cmdexpand_get_script_name(int idx)
{
  scriptitem_T *si = SCRIPT_ITEM(idx + 1);
  home_replace(NULL, si->sn_name, NameBuff, MAXPATHL, true);
  return NameBuff;
}

// =============================================================================
// Phase 2: C accessors for wildmenu key handlers, wildmenu_cleanup,
//           expand_cmdline, and set_cmd_context
// =============================================================================

/// Get p_wic wildchar-if-count option (for Rust FFI).
int nvim_cmdexpand_get_p_wic(void)
{
  return (int)p_wic;
}

/// Get xp_context from ccline (for Rust FFI).
int nvim_cmdexpand_get_ccline_xp_context(void)
{
  return get_cmdline_info()->xp_context;
}

/// Get xp_arg from ccline (for Rust FFI).
char *nvim_cmdexpand_get_ccline_xp_arg(void)
{
  return get_cmdline_info()->xp_arg;
}

/// Wrapper for set_context_for_expression (for Rust FFI).
void nvim_cmdexpand_set_context_for_expression(expand_T *xp, char *str, int cmdidx)
{
  set_context_for_expression(xp, str, (cmdidx_T)cmdidx);
}

/// Wrapper for addstar (for Rust FFI).
char *nvim_cmdexpand_addstar(char *fname, size_t len, int context)
{
  return addstar(fname, len, context);
}

/// Wrapper for cmdline_del on the real ccline (for Rust FFI).
void nvim_cmdexpand_cmdline_del(int from)
{
  cmdline_del(get_cmdline_info(), from);
}

/// Set KeyTyped global (for Rust FFI).
void nvim_cmdexpand_set_key_typed(int val)
{
  KeyTyped = (bool)val;
}

/// Get KeyTyped global (for Rust FFI).
int nvim_cmdexpand_get_key_typed(void)
{
  return (int)KeyTyped;
}

/// Wrapper for put_on_cmdline (for Rust FFI).
void nvim_cmdexpand_put_on_cmdline(const char *str, int len, int redraw)
{
  put_on_cmdline(str, len, (bool)redraw);
}

/// Wrapper for utf_head_off (for Rust FFI).
int nvim_cmdexpand_utf_head_off(const char *base, const char *p)
{
  return utf_head_off(base, p);
}

/// Get RedrawingDisabled (for Rust FFI).
int nvim_cmdexpand_get_redrawing_disabled(void)
{
  return RedrawingDisabled;
}

/// Set RedrawingDisabled (for Rust FFI).
void nvim_cmdexpand_set_redrawing_disabled(int val)
{
  RedrawingDisabled = val;
}

/// Wrapper for set_no_hlsearch (for Rust FFI).
void nvim_cmdexpand_set_no_hlsearch(int val)
{
  set_no_hlsearch((bool)val);
}

/// Get WM_SCROLLED constant (for Rust FFI).
int nvim_cmdexpand_get_wm_scrolled(void)
{
  return WM_SCROLLED;
}

/// Get cmdline_row global (for Rust FFI).
int nvim_cmdexpand_get_cmdline_row(void)
{
  return cmdline_row;
}

/// Decrement cmdline_row (for Rust FFI).
void nvim_cmdexpand_dec_cmdline_row(void)
{
  cmdline_row--;
}

/// Wrapper for redrawcmd (for Rust FFI).
void nvim_cmdexpand_redrawcmd(void)
{
  redrawcmd();
}

/// Set wild_menu_showing (for Rust FFI).
void nvim_cmdexpand_set_wild_menu_showing(int val)
{
  wild_menu_showing = val;
}

/// Get save_p_ls (for Rust FFI).
int nvim_cmdexpand_get_save_p_ls(void)
{
  return (int)save_p_ls;
}

/// Set save_p_ls (for Rust FFI).
void nvim_cmdexpand_set_save_p_ls(int val)
{
  save_p_ls = val;
}

/// Get save_p_wmh (for Rust FFI).
int nvim_cmdexpand_get_save_p_wmh(void)
{
  return (int)save_p_wmh;
}

/// Set save_p_wmh (for Rust FFI).
void nvim_cmdexpand_set_save_p_wmh(int val)
{
  save_p_wmh = val;
}

/// Set p_ls (for Rust FFI).
void nvim_cmdexpand_set_p_ls(int64_t val)
{
  p_ls = val;
}

/// Set p_wmh (for Rust FFI).
void nvim_cmdexpand_set_p_wmh(int64_t val)
{
  p_wmh = val;
}

/// Wrapper for update_screen (for Rust FFI).
void nvim_cmdexpand_update_screen(void)
{
  update_screen();
}

/// Wrapper for win_redraw_last_status(topframe) (for Rust FFI).
void nvim_cmdexpand_win_redraw_last_status(void)
{
  win_redraw_last_status(topframe);
}

/// Wrapper for redraw_statuslines (for Rust FFI).
void nvim_cmdexpand_redraw_statuslines(void)
{
  redraw_statuslines();
}

/// Get PATHSEP character (for Rust FFI).
int nvim_cmdexpand_get_pathsep(void)
{
  return PATHSEP;
}

// =============================================================================
// Phase 3: C accessors for ExpandOther, ExpandFromContext, and ExpandGeneric
// =============================================================================

/// Wrapper for ExpandGeneric (for Rust FFI).
void nvim_cmdexpand_expand_generic(const char *pat, expand_T *xp, regmatch_T *regmatch,
                                   char ***matches, int *numMatches,
                                   CompleteListItemGetter func, int escaped)
{
  ExpandGeneric(pat, xp, regmatch, matches, numMatches, func, (bool)escaped);
}

/// Wrapper for vim_regcomp (for Rust FFI).
void *nvim_cmdexpand_vim_regcomp(const char *pat, int flags)
{
  return (void *)vim_regcomp(pat, flags);
}

/// Wrapper for vim_regfree (for Rust FFI).
void nvim_cmdexpand_vim_regfree(void *prog)
{
  vim_regfree((regprog_T *)prog);
}

/// Wrapper for ignorecase (for Rust FFI).
int nvim_cmdexpand_ignorecase(const char *pat)
{
  return ignorecase(pat);
}

/// Set regmatch.rm_ic (for Rust FFI).
void nvim_cmdexpand_regmatch_set_rm_ic(regmatch_T *rmp, int val)
{
  rmp->rm_ic = (bool)val;
}

/// Set regmatch.regprog (for Rust FFI).
void nvim_cmdexpand_regmatch_set_regprog(regmatch_T *rmp, void *prog)
{
  rmp->rm_ic = false;
  rmp->regprog = (regprog_T *)prog;
}

/// Wrapper for find_help_tags (for Rust FFI). Returns OK or FAIL.
int nvim_cmdexpand_find_help_tags(const char *pat, int *numMatches, char ***matches)
{
  if (find_help_tags(*pat == NUL ? "help" : pat, numMatches, matches, false) == OK) {
    cleanup_help_tags(*numMatches, *matches);
    return 1;
  }
  return 0;
}

/// Wrapper for expand_shellcmd (for Rust FFI).
void nvim_cmdexpand_expand_shellcmd(char *filepat, char ***matches, int *numMatches, int flags)
{
  expand_shellcmd(filepat, matches, numMatches, flags);
}

/// Wrapper for ExpandOldSetting (for Rust FFI).
int nvim_cmdexpand_expand_old_setting(int *numMatches, char ***matches)
{
  return ExpandOldSetting(numMatches, matches);
}

/// Wrapper for ExpandBufnames (for Rust FFI).
int nvim_cmdexpand_expand_bufnames(const char *pat, int *numMatches, char ***matches, int options)
{
  return ExpandBufnames(pat, numMatches, matches, options);
}

/// Wrapper for ExpandRTDir with NULL-terminated directories array.
/// dirs_count must match the number of dirs.
int nvim_cmdexpand_expand_rtdir(const char *pat, int flags, int *numMatches, char ***matches,
                                char **directories)
{
  return ExpandRTDir(pat, flags, numMatches, matches, directories);
}

/// Wrapper for ExpandPackAddDir (for Rust FFI).
int nvim_cmdexpand_expand_pack_add_dir(const char *pat, int *numMatches, char ***matches)
{
  return ExpandPackAddDir(pat, numMatches, matches);
}

/// Wrapper for expand_runtime_cmd (for Rust FFI).
int nvim_cmdexpand_expand_runtime_cmd(const char *pat, int *numMatches, char ***matches)
{
  return expand_runtime_cmd(pat, numMatches, matches);
}

/// Wrapper for ExpandSettings (for Rust FFI).
int nvim_cmdexpand_expand_settings(expand_T *xp, regmatch_T *regmatch, const char *pat,
                                   int *numMatches, char ***matches, int fuzzy)
{
  return ExpandSettings(xp, regmatch, pat, numMatches, matches, (bool)fuzzy);
}

/// Wrapper for ExpandStringSetting (for Rust FFI).
int nvim_cmdexpand_expand_string_setting(expand_T *xp, regmatch_T *regmatch,
                                          int *numMatches, char ***matches)
{
  return ExpandStringSetting(xp, regmatch, numMatches, matches);
}

/// Wrapper for ExpandMappings (for Rust FFI).
int nvim_cmdexpand_expand_mappings(const char *pat, regmatch_T *regmatch,
                                   int *numMatches, char ***matches)
{
  return ExpandMappings(pat, regmatch, numMatches, matches);
}

/// Wrapper for expand_argopt (for Rust FFI).
int nvim_cmdexpand_expand_argopt(const char *pat, expand_T *xp, regmatch_T *regmatch,
                                  char ***matches, int *numMatches)
{
  return expand_argopt(pat, xp, regmatch, matches, numMatches);
}

/// Wrapper for ExpandUserDefined (for Rust FFI).
int nvim_cmdexpand_expand_user_defined(const char *pat, expand_T *xp, regmatch_T *regmatch,
                                        char ***matches, int *numMatches)
{
  return ExpandUserDefined(pat, xp, regmatch, matches, numMatches);
}

/// Wrapper for ExpandUserList (for Rust FFI).
int nvim_cmdexpand_expand_user_list(expand_T *xp, char ***matches, int *numMatches)
{
  return ExpandUserList(xp, matches, numMatches);
}

/// Wrapper for ExpandUserLua (for Rust FFI).
int nvim_cmdexpand_expand_user_lua(expand_T *xp, int *numMatches, char ***matches)
{
  return ExpandUserLua(xp, numMatches, matches);
}

/// Wrapper for nlua_expand_get_matches (for Rust FFI).
int nvim_cmdexpand_nlua_expand_get_matches(int *numMatches, char ***matches)
{
  return nlua_expand_get_matches(numMatches, matches);
}

/// Get DIP_START + DIP_OPT flags (for Rust FFI).
int nvim_cmdexpand_get_dip_start_opt(void)
{
  return DIP_START + DIP_OPT;
}

/// Get RE_MAGIC constant (for Rust FFI).
int nvim_cmdexpand_get_re_magic(void)
{
  return RE_MAGIC;
}

/// Wrapper for rs_magic_isset() (already declared, but needs int return for Rust FFI).
int nvim_cmdexpand_magic_isset(void)
{
  return rs_magic_isset();
}

/// Wrapper for xmalloc(n) then snprintf pattern for EXPAND_USER_FUNC s: prefix.
/// Returns allocated string "^<SNR>\\d\\+_<suffix>" or NULL.
char *nvim_cmdexpand_make_snr_pattern(const char *suffix)
{
  const size_t len = strlen(suffix) + 20;
  char *tofree = xmalloc(len);
  snprintf(tofree, len, "^<SNR>\\d\\+_%s", suffix);
  return tofree;
}

/// Function pointer accessors for ExpandOther dispatch table (for Rust FFI).
CompleteListItemGetter nvim_cmdexpand_get_fn_get_command_name(void)
{
  return get_command_name;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_history_arg(void)
{
  return get_history_arg;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_user_commands(void)
{
  return get_user_commands;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_user_cmd_addr_type(void)
{
  return get_user_cmd_addr_type;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_user_cmd_flags(void)
{
  return get_user_cmd_flags;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_user_cmd_nargs(void)
{
  return get_user_cmd_nargs;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_user_cmd_complete(void)
{
  return get_user_cmd_complete;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_user_var_name(void)
{
  return get_user_var_name;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_function_name(void)
{
  return get_function_name;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_user_func_name(void)
{
  return get_user_func_name;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_expr_name(void)
{
  return get_expr_name;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_menu_name(void)
{
  return get_menu_name;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_menu_names(void)
{
  return get_menu_names;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_syntax_name(void)
{
  return get_syntax_name;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_syntime_arg(void)
{
  return get_syntime_arg;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_highlight_name(void)
{
  return get_highlight_name;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_expand_get_event_name(void)
{
  return expand_get_event_name;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_expand_get_augroup_name(void)
{
  return expand_get_augroup_name;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_sign_name(void)
{
  return get_sign_name;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_profile_name(void)
{
  return get_profile_name;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_lang_arg(void)
{
  return get_lang_arg;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_locales(void)
{
  return get_locales;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_env_name(void)
{
  return get_env_name;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_users(void)
{
  return get_users;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_arglist_name(void)
{
  return get_arglist_name;
}
CompleteListItemGetter nvim_cmdexpand_get_fn_get_healthcheck_names(void)
{
  return get_healthcheck_names;
}

#define SHOW_MATCH(m) (showtail ? rs_showmatches_gettail(matches[m], false) : matches[m])



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
    cmd_showtail = rs_expand_showtail(xp) != 0;
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
    if (rs_cmdline_fuzzy_completion_supported(xp->xp_context)
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
  char *endpos = showtail ? rs_showmatches_gettail(xp->xp_pattern, noselect) : xp->xp_pattern;
  if (ui_has(kUICmdline) && cmdline_win == NULL) {
    compl_startcol = (int)(endpos - ccline->cmdbuff);
  } else {
    compl_startcol = cmd_screencol((int)(endpos - ccline->cmdbuff));
  }
}

// cmdline_pum_display, cmdline_pum_remove, cmdline_pum_cleanup,
// cmdline_compl_pattern, cmdline_compl_is_fuzzy -- migrated to Rust (pum.rs)


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
  int clen = rs_wildmenu_match_len(xp, SHOW_MATCH(match)) + 3;  // length in screen cells
  if (match == 0) {
    first_match = 0;
  } else if (match < first_match) {
    // jumping left, as far as we can go
    first_match = match;
    add_left = true;
  } else {
    // check if match fits on the screen
    for (i = first_match; i < match; i++) {
      clen += rs_wildmenu_match_len(xp, SHOW_MATCH(i)) + 2;
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
        clen += rs_wildmenu_match_len(xp, SHOW_MATCH(i)) + 2;
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
      clen += rs_wildmenu_match_len(xp, SHOW_MATCH(first_match - 1)) + 2;
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
  while (clen + rs_wildmenu_match_len(xp, SHOW_MATCH(i)) + 2 < Columns) {
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
        s += rs_skip_wildmenu_char(xp, s);
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
    showtail = rs_expand_showtail(xp) != 0;
  } else {
    numMatches = xp->xp_numfiles;
    matches = xp->xp_files;
    showtail = cmd_showtail;
  }

  if (rs_cmdline_compl_use_pum(display_wildmenu && !display_list)) {
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
// set_expand_context -- migrated to Rust (lib.rs)

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

/// C accessor for Rust: wraps set_cmd_index and returns cmdidx via out-param.
///
/// Allows Rust to call set_cmd_index without needing a repr(C) exarg_T layout.
const char *nvim_cmdexpand_set_cmd_index(const char *cmd, expand_T *xp, int *complp,
                                         int *cmdidx_out)
{
  exarg_T ea = { 0 };
  const char *p = set_cmd_index(cmd, &ea, xp, complp);
  *cmdidx_out = (int)ea.cmdidx;
  return p;
}

// set_cmd_context -- migrated to Rust (lib.rs)

// expand_cmdline -- migrated to Rust (lib.rs)

// get_scriptnames_arg -- migrated to Rust (callbacks.rs)


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

// ExpandOther and ExpandFromContext -- migrated to Rust (expand.rs)

// ExpandGeneric -- migrated to Rust (expand.rs)

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
        rs_expand_escape(&xpc, buf, num_p, p, WILD_SILENT | expand_options);

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

// wildmenu_translate_key -- migrated to Rust (wildmenu.rs)

// cmdline_del -- used by nvim_cmdexpand_cmdline_del accessor only (kept for now)
static void cmdline_del(CmdlineInfo *cclp, int from)
{
  assert(cclp->cmdpos <= cclp->cmdlen);
  memmove(cclp->cmdbuff + from, cclp->cmdbuff + cclp->cmdpos,
          (size_t)cclp->cmdlen - (size_t)cclp->cmdpos + 1);
  cclp->cmdlen -= cclp->cmdpos - from;
  cclp->cmdpos = from;
}

// wildmenu_process_key_menunames -- migrated to Rust (wildmenu.rs)
// wildmenu_process_key_filenames -- migrated to Rust (wildmenu.rs)
// wildmenu_process_key -- migrated to Rust (wildmenu.rs)
// wildmenu_cleanup -- migrated to Rust (wildmenu.rs)

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
    rs_set_context_for_wildcard_arg(xpc.xp_pattern, 0, &xpc, &context);
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
  if (rs_cmdline_fuzzy_completion_supported(xpc.xp_context)) {
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
// Rust helper for empty pattern check
extern int rs_empty_pattern_magic(const char *p, size_t len, int magic_val);

/// Parse the command pattern and range for incsearch highlighting.
/// Sets search_first_line, search_last_line, and the skip/pat lengths.
/// Returns true if a valid non-empty pattern was found, false otherwise.
bool parse_pattern_and_range(pos_T *incsearch_start, int *search_delim, int *skiplen,
                             int *patlen)
  FUNC_ATTR_NONNULL_ALL
{
  char *p;
  bool delim_optional = false;
  const char *dummy;
  magic_T magic = 0;

  *skiplen = 0;
  *patlen = nvim_get_ccline_cmdlen();

  // Default range
  search_first_line = 0;
  search_last_line = MAXLNUM;

  exarg_T ea = {
    .line1 = 1,
    .line2 = 1,
    .cmd = nvim_get_ccline_cmdbuff(),
    .addr_type = ADDR_LINES,
  };

  cmdmod_T dummy_cmdmod;
  // Skip over command modifiers
  parse_command_modifiers(&ea, &dummy, &dummy_cmdmod, true);

  // Skip over the range to find the command.
  char *cmd = skip_range(ea.cmd, NULL);
  if (vim_strchr("sgvl", (uint8_t)(*cmd)) == NULL) {
    return false;
  }

  // Skip over command name to find pattern separator
  for (p = cmd; ASCII_ISALPHA(*p); p++) {}
  if (*skipwhite(p) == NUL) {
    return false;
  }

  if (strncmp(cmd, "substitute", (size_t)(p - cmd)) == 0
      || strncmp(cmd, "smagic", (size_t)(p - cmd)) == 0
      || strncmp(cmd, "snomagic", (size_t)MAX(p - cmd, 3)) == 0
      || strncmp(cmd, "vglobal", (size_t)(p - cmd)) == 0) {
    if (*cmd == 's' && cmd[1] == 'm') {
      magic_overruled = OPTION_MAGIC_ON;
    } else if (*cmd == 's' && cmd[1] == 'n') {
      magic_overruled = OPTION_MAGIC_OFF;
    }
  } else if (strncmp(cmd, "sort", (size_t)MAX(p - cmd, 3)) == 0
             || strncmp(cmd, "uniq", (size_t)MAX(p - cmd, 3)) == 0) {
    // skip over ! and flags
    if (*p == '!') {
      p = skipwhite(p + 1);
    }
    while (ASCII_ISALPHA(*(p = skipwhite(p)))) {
      p++;
    }
    if (*p == NUL) {
      return false;
    }
  } else if (strncmp(cmd, "vimgrep", (size_t)MAX(p - cmd, 3)) == 0
             || strncmp(cmd, "vimgrepadd", (size_t)MAX(p - cmd, 8)) == 0
             || strncmp(cmd, "lvimgrep", (size_t)MAX(p - cmd, 2)) == 0
             || strncmp(cmd, "lvimgrepadd", (size_t)MAX(p - cmd, 9)) == 0
             || strncmp(cmd, "global", (size_t)(p - cmd)) == 0) {
    // skip optional "!"
    if (*p == '!') {
      p++;
      if (*skipwhite(p) == NUL) {
        return false;
      }
    }
    if (*cmd != 'g') {
      delim_optional = true;
    }
  } else {
    return false;
  }

  p = skipwhite(p);
  int delim = (delim_optional && vim_isIDc((uint8_t)(*p))) ? ' ' : *p++;
  *search_delim = delim;

  char *end = skip_regexp_ex(p, delim, rs_magic_isset(), NULL, NULL, &magic);
  bool use_last_pat = end == p && *end == delim;

  if (end == p && !use_last_pat) {
    return false;
  }

  // Skip if the pattern matches everything (e.g., for 'hlsearch')
  if (!use_last_pat) {
    char c = *end;
    *end = NUL;
    bool empty = (bool)rs_empty_pattern_magic(p, (size_t)(end - p), (int)magic);
    *end = c;
    if (empty) {
      return false;
    }
  }

  // Found a non-empty pattern or //
  *skiplen = (int)(p - nvim_get_ccline_cmdbuff());
  *patlen = (int)(end - p);

  // Parse the address range
  pos_T save_cursor = curwin->w_cursor;
  curwin->w_cursor = *incsearch_start;

  parse_cmd_address(&ea, &dummy, true);

  if (ea.addr_count > 0) {
    // Allow for reverse match.
    search_first_line = MIN(ea.line2, ea.line1);
    search_last_line = MAX(ea.line2, ea.line1);
  } else if (cmd[0] == 's' && cmd[1] != 'o') {
    // :s defaults to the current line
    search_first_line = search_last_line = curwin->w_cursor.lnum;
  }

  curwin->w_cursor = save_cursor;
  return true;
}
