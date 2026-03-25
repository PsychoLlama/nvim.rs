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


extern int rs_sort_func_compare(const void *s1, const void *s2);
extern int rs_cmdline_compl_use_pum(int need_wildmenu);
extern int rs_map_wildopts_to_ewflags(int options);
extern char *rs_showmatches_gettail(char *s, int eager);
extern int rs_expand_showtail(expand_T *xp);
extern void rs_expand_escape(expand_T *xp, char *str, int numfiles, char **files, int options);
extern char *rs_find_longest_match(expand_T *xp, int options);
extern const char *rs_set_context_in_argopt(expand_T *xp, const char *arg);
extern void rs_set_context_for_wildcard_arg(const char *arg, int is_shell_cmd,
                                            expand_T *xp, int *complp);
extern char *rs_get_filetypecmd_arg(expand_T *xp, int idx);
extern char *rs_get_breakadd_arg(expand_T *xp, int idx);
extern char *rs_get_retab_arg(expand_T *xp, int idx);
extern char *rs_get_messages_arg(expand_T *xp, int idx);
extern char *rs_get_mapclear_arg(expand_T *xp, int idx);
extern char *rs_get_scriptnames_arg(expand_T *xp, int idx);
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

/// Set the expansion context type.
void nvim_expand_set_context(expand_T *xp, int context)
{
  if (xp) {
    xp->xp_context = context;
  }
}

/// Check if cmdline_win is NULL (for Rust FFI).
int nvim_get_cmdline_win_is_null(void)
{
  return cmdline_win == NULL;
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

// nvim_cmdexpand_expand_user_defined -- deleted, Rust calls rs_expand_user_defined directly
// nvim_cmdexpand_expand_user_list -- deleted, Rust calls rs_expand_user_list directly
// nvim_cmdexpand_expand_user_lua -- deleted, Rust calls rs_expand_user_lua directly

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

/// Wrapper for cursorcmd (for Rust FFI).
void nvim_cmdexpand_cursorcmd(void)
{
  cursorcmd();
}

/// Wrapper for ui_flush (for Rust FFI).
void nvim_cmdexpand_ui_flush(void)
{
  ui_flush();
}

/// Wrapper for msg_puts (for Rust FFI).
void nvim_cmdexpand_msg_puts(const char *s)
{
  msg_puts(s);
}

/// Get cmd_silent (for Rust FFI).
int nvim_cmdexpand_get_cmd_silent(void)
{
  return cmd_silent;
}

/// Get got_int (for Rust FFI).
int nvim_cmdexpand_get_got_int(void)
{
  return got_int;
}

/// Set got_int (for Rust FFI).
void nvim_cmdexpand_set_got_int(int val)
{
  got_int = (bool)val;
}

/// Wrapper for xstrnsave (for Rust FFI).
char *nvim_cmdexpand_xstrnsave(const char *s, size_t n)
{
  return xstrnsave(s, n);
}

/// Set cmd_showtail (for Rust FFI).
void nvim_cmdexpand_set_cmd_showtail(int val)
{
  cmd_showtail = (bool)val;
}

/// Set may_expand_pattern (for Rust FFI).
void nvim_cmdexpand_set_may_expand_pattern(int val)
{
  may_expand_pattern = (bool)val;
}

/// Copy pre_incsearch_pos from xp->xp_pre_incsearch_pos (for Rust FFI).
void nvim_cmdexpand_copy_pre_incsearch_pos(expand_T *xp)
{
  pre_incsearch_pos = xp->xp_pre_incsearch_pos;
}

/// Save cmdline_orig from current ccline->cmdbuff (for Rust FFI).
void nvim_cmdexpand_save_cmdline_orig(void)
{
  CmdlineInfo *ccline = get_cmdline_info();
  xfree(cmdline_orig);
  cmdline_orig = xstrnsave(ccline->cmdbuff, (size_t)ccline->cmdlen);
}

/// Apply expansion result into ccline->cmdbuff (for Rust FFI).
/// @param xp         expand context, xp_pattern_len is used
/// @param i          offset of pattern start in cmdbuff
/// @param p          expansion result string (caller retains ownership)
/// @param plen       length of p
void nvim_cmdexpand_apply_expansion(expand_T *xp, int i, const char *p, int plen)
{
  CmdlineInfo *ccline = get_cmdline_info();
  int difflen = plen - (int)xp->xp_pattern_len;
  if (ccline->cmdlen + difflen + 4 > ccline->cmdbufflen) {
    realloc_cmdbuff(ccline->cmdlen + difflen + 4);
    xp->xp_pattern = ccline->cmdbuff + i;
  }
  assert(ccline->cmdpos <= ccline->cmdlen);
  memmove(&ccline->cmdbuff[ccline->cmdpos + difflen],
          &ccline->cmdbuff[ccline->cmdpos],
          (size_t)ccline->cmdlen - (size_t)ccline->cmdpos + 1);
  memmove(&ccline->cmdbuff[i], p, (size_t)plen);
  ccline->cmdlen += difflen;
  ccline->cmdpos += difflen;
}

/// Wrapper for nlua_expand_pat (for Rust FFI).
void nvim_cmdexpand_nlua_expand_pat(expand_T *xp)
{
  nlua_expand_pat(xp);
}

/// Set msg_didany (for Rust FFI).
void nvim_cmdexpand_set_msg_didany(int val)
{
  msg_didany = (bool)val;
}

/// Wrapper for msg_start (for Rust FFI).
void nvim_cmdexpand_msg_start(void)
{
  msg_start();
}

/// Wrapper for msg_putchar (for Rust FFI).
void nvim_cmdexpand_msg_putchar(int c)
{
  msg_putchar(c);
}

/// Get msg_row (for Rust FFI).
int nvim_cmdexpand_get_msg_row(void)
{
  return msg_row;
}

/// Set cmdline_row (for Rust FFI).
void nvim_cmdexpand_set_cmdline_row(int val)
{
  cmdline_row = val;
}

/// Wrapper for msg_ext_set_kind (for Rust FFI).
void nvim_cmdexpand_msg_ext_set_kind(const char *kind)
{
  msg_ext_set_kind(kind);
}

/// Get Columns (for Rust FFI).
int nvim_cmdexpand_get_columns(void)
{
  return Columns;
}

/// Free wild matches via FreeWild (for Rust FFI).
void nvim_cmdexpand_free_wild(int count, char **files)
{
  FreeWild(count, files);
}

/// Wrapper for pum_clear (for Rust FFI).
void nvim_cmdexpand_pum_clear(void)
{
  pum_clear();
}

/// Set compl_selected (for Rust FFI).
void nvim_cmdexpand_set_compl_selected(int val)
{
  compl_selected = val;
}

/// Create PUM from explicit matches array (for Rust FFI).
void nvim_cmdexpand_pum_create_from_matches(expand_T *xp, char **matches, int num_matches,
                                            int showtail, int noselect)
{
  cmdline_pum_create(get_cmdline_info(), xp, matches, num_matches,
                     showtail != 0, noselect != 0);
}

/// Wrapper for redraw_wildmenu with explicit matches (for Rust FFI).
void nvim_cmdexpand_redraw_wildmenu_ex(expand_T *xp, int num_matches, char **matches,
                                       int findex, int showtail)
{
  redraw_wildmenu(xp, num_matches, matches, findex, showtail != 0);
}

/// Get msg_col (for Rust FFI).
int nvim_cmdexpand_get_msg_col(void)
{
  return msg_col;
}

/// Wrapper for msg_clr_eos (for Rust FFI).
void nvim_cmdexpand_msg_clr_eos(void)
{
  msg_clr_eos();
}

/// Wrapper for msg_outtrans (for Rust FFI). Returns column after output.
int nvim_cmdexpand_msg_outtrans(const char *str, int attr, int maxcol)
{
  return msg_outtrans(str, attr, (bool)maxcol);
}

/// Wrapper for msg_outtrans_long (for Rust FFI).
void nvim_cmdexpand_msg_outtrans_long(const char *str, int attr)
{
  msg_outtrans_long(str, attr);
}

/// Wrapper for msg_advance (for Rust FFI).
void nvim_cmdexpand_msg_advance(int col)
{
  msg_advance(col);
}

/// Replace $HOME with ~ in matches[i] for display.
/// Returns pointer into NameBuff static buffer.
char *nvim_cmdexpand_home_replace_match(const char *s)
{
  home_replace(NULL, s, NameBuff, MAXPATHL, true);
  return NameBuff;
}

/// Wrapper for expand_env_save_opt (for Rust FFI).
char *nvim_cmdexpand_expand_env_save_opt(const char *str)
{
  return expand_env_save_opt((char *)str, true);
}

/// Wrapper for backslash_halve_save (for Rust FFI).
char *nvim_cmdexpand_backslash_halve_save(const char *str)
{
  return backslash_halve_save((char *)str);
}

/// Wrapper for os_isdir (for Rust FFI).
int nvim_cmdexpand_os_isdir(const char *str)
{
  return os_isdir(str);
}

/// Wrapper for vim_strsize (for Rust FFI).
int nvim_cmdexpand_vim_strsize(const char *str)
{
  return vim_strsize(str);
}

/// Wrapper for msg_puts_hl (for Rust FFI).
void nvim_cmdexpand_msg_puts_hl(const char *str, int attr, int maxcol)
{
  msg_puts_hl(str, attr, (bool)maxcol);
}

/// Wrapper for rs_showmatches_gettail via SHOW_MATCH macro (for Rust FFI).
/// Returns either gettail result (if showtail) or matches[m] directly.
char *nvim_cmdexpand_show_match(char **matches, int m, int showtail)
{
  return showtail ? rs_showmatches_gettail(matches[m], false) : matches[m];
}

/// Wrapper for rs_cmdline_compl_use_pum (for Rust FFI).
int nvim_cmdexpand_compl_use_pum(int need_wildmenu)
{
  return rs_cmdline_compl_use_pum(need_wildmenu);
}

#define SHOW_MATCH(m) (showtail ? rs_showmatches_gettail(matches[m], false) : matches[m])



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

/// Get the type of typval_T[idx] from an argvars array (for Rust FFI).
int nvim_cmdexpand_tv_get_type(typval_T *argvars, int idx)
{
  return (int)argvars[idx].v_type;
}

/// Check that argvars[idx] is a string (emits error if not). Returns FAIL on error.
int nvim_cmdexpand_tv_check_for_string_arg(typval_T *argvars, int idx)
{
  return tv_check_for_string_arg(argvars, idx);
}

/// Get string value from argvars[idx]. Returns "" on missing/invalid.
const char *nvim_cmdexpand_tv_get_string(typval_T *argvars, int idx)
{
  return tv_get_string(&argvars[idx]);
}

/// Get number from argvars[idx] with error check. Sets *errorp to 1 on error.
int64_t nvim_cmdexpand_tv_get_number_chk(typval_T *argvars, int idx, int *errorp)
{
  bool err = false;
  int64_t val = tv_get_number_chk(&argvars[idx], &err);
  if (errorp != NULL) {
    *errorp = err ? 1 : 0;
  }
  return val;
}

/// Allocate a list and set rettv to it (for Rust FFI).
void nvim_cmdexpand_tv_list_alloc_ret(typval_T *rettv, int estimated_count)
{
  tv_list_alloc_ret(rettv, estimated_count);
}

/// Append string to a list_T (from rettv->vval.v_list) (for Rust FFI).
void nvim_cmdexpand_tv_list_append_string(typval_T *rettv, const char *str, int64_t len)
{
  tv_list_append_string(rettv->vval.v_list, str, len);
}

/// Set rettv to type VAR_STRING with the given string value (for Rust FFI).
/// Passes ownership of str to the typval.
void nvim_cmdexpand_tv_set_string(typval_T *rettv, char *str)
{
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = str;
}

/// Allocate a dict and set rettv to it (for Rust FFI).
void nvim_cmdexpand_tv_dict_alloc_ret(typval_T *rettv)
{
  tv_dict_alloc_ret(rettv);
}

/// Add string to dict (for Rust FFI). Returns OK or FAIL.
int nvim_cmdexpand_tv_dict_add_str(typval_T *rettv, const char *key, size_t klen, const char *val)
{
  return tv_dict_add_str(rettv->vval.v_dict, key, klen, val);
}

/// Add integer to dict (for Rust FFI). Returns OK or FAIL.
int nvim_cmdexpand_tv_dict_add_nr(typval_T *rettv, const char *key, size_t klen, int64_t val)
{
  return tv_dict_add_nr(rettv->vval.v_dict, key, klen, val);
}

/// Allocate a list and add it to dict (for Rust FFI). Returns list handle.
list_T *nvim_cmdexpand_tv_dict_add_list(typval_T *rettv, const char *key, size_t klen, int count)
{
  list_T *li = tv_list_alloc(count);
  tv_dict_add_list(rettv->vval.v_dict, key, klen, li);
  return li;
}

/// Append string to a list_T directly (for Rust FFI).
void nvim_cmdexpand_list_append_string(list_T *li, const char *str, int64_t len)
{
  tv_list_append_string(li, str, len);
}

/// Get pum_visible() return value (for Rust FFI).
int nvim_cmdexpand_pum_visible(void)
{
  return pum_visible();
}

/// Get xp_selected from ccline->xpc (for Rust FFI).
int nvim_cmdexpand_get_ccline_xp_selected(void)
{
  CmdlineInfo *ccline = get_cmdline_info();
  if (ccline == NULL || ccline->xpc == NULL) {
    return 0;
  }
  return ccline->xpc->xp_selected;
}

/// Get xp_numfiles from ccline->xpc (for Rust FFI). Returns -1 if no xpc.
int nvim_cmdexpand_get_ccline_xp_numfiles(void)
{
  CmdlineInfo *ccline = get_cmdline_info();
  if (ccline == NULL || ccline->xpc == NULL) {
    return -1;
  }
  return ccline->xpc->xp_numfiles;
}

/// Get xp_files[idx] from ccline->xpc (for Rust FFI). Returns NULL if out of range.
const char *nvim_cmdexpand_get_ccline_xp_file(int idx)
{
  CmdlineInfo *ccline = get_cmdline_info();
  if (ccline == NULL || ccline->xpc == NULL || ccline->xpc->xp_files == NULL) {
    return NULL;
  }
  if (idx < 0 || idx >= ccline->xpc->xp_numfiles) {
    return NULL;
  }
  return ccline->xpc->xp_files[idx];
}

/// Check ccline->xpc->xp_files is not NULL (for Rust FFI).
int nvim_cmdexpand_ccline_has_xp_files(void)
{
  CmdlineInfo *ccline = get_cmdline_info();
  return ccline != NULL && ccline->xpc != NULL && ccline->xpc->xp_files != NULL;
}

/// Convert completion type string to context integer (for Rust FFI).
int nvim_cmdexpand_cmdcomplete_str_to_type(const char *type)
{
  return cmdcomplete_str_to_type(type);
}

/// Convert completion context integer (+ arg) to string (for Rust FFI). Returns xstrdup.
char *nvim_cmdexpand_cmdcomplete_type_to_str(int ctx, const char *arg)
{
  return cmdcomplete_type_to_str(ctx, arg);
}

/// Get the cmdline_orig static (for Rust FFI).
const char *nvim_cmdexpand_get_cmdline_orig(void)
{
  return cmdline_orig;
}

/// set_context_in_menu_cmd wrapper (for Rust FFI).
void nvim_cmdexpand_set_context_in_menu_cmd(expand_T *xp, const char *cmd, char *arg, bool delim_optional)
{
  set_context_in_menu_cmd(xp, cmd, arg, delim_optional);
}

/// set_context_in_sign_cmd wrapper (for Rust FFI).
void nvim_cmdexpand_set_context_in_sign_cmd(expand_T *xp, char *arg)
{
  set_context_in_sign_cmd(xp, arg);
}

/// set_context_in_runtime_cmd wrapper (for Rust FFI).
void nvim_cmdexpand_set_context_in_runtime_cmd(expand_T *xp, char *arg)
{
  set_context_in_runtime_cmd(xp, arg);
}

/// VAR_UNKNOWN constant (for Rust FFI).
int nvim_cmdexpand_get_var_unknown(void)
{
  return VAR_UNKNOWN;
}

/// filetype_expand_what = EXP_FILETYPECMD_ALL (for Rust FFI).
void nvim_cmdexpand_set_filetype_expand_all(void)
{
  filetype_expand_what = EXP_FILETYPECMD_ALL;
}

/// emsg(_(e_invarg)) wrapper (for Rust FFI).
void nvim_cmdexpand_emsg_invarg(void)
{
  emsg(_(e_invarg));
}

/// semsg(_(e_invarg2), type) wrapper (for Rust FFI).
void nvim_cmdexpand_semsg_invarg2(const char *type)
{
  semsg(_(e_invarg2), type);
}

/// xmemdupz wrapper (for Rust FFI).
char *nvim_cmdexpand_xmemdupz(const char *s, size_t len)
{
  return xmemdupz(s, len);
}

/// call_user_expand_func with call_func_retlist (for Rust FFI).
list_T *nvim_cmdexpand_call_user_expand_retlist(expand_T *xp)
{
  return call_user_expand_func(call_func_retlist, xp);
}

/// call_user_expand_func with call_func_retstr (for Rust FFI).
char *nvim_cmdexpand_call_user_expand_retstr(expand_T *xp)
{
  return call_user_expand_func(call_func_retstr, xp);
}

/// nlua_call_user_expand_func wrapper (for Rust FFI).
/// Caller must tv_clear rettv when done.
int nvim_cmdexpand_nlua_call_user_expand(expand_T *xp)
{
  typval_T rettv = TV_INITIAL_VALUE;
  nlua_call_user_expand_func(xp, &rettv);
  if (rettv.v_type != VAR_LIST) {
    tv_clear(&rettv);
    return -1;  // Not a list
  }
  // Return list refcount +1; caller must tv_list_unref.
  list_T *li = rettv.vval.v_list;
  tv_list_ref(li);
  tv_clear(&rettv);
  return 0;  // Not used; see nvim_cmdexpand_nlua_get_retlist.
}

/// nlua_call_user_expand_func wrapper returning list_T * (for Rust FFI).
/// Returns NULL if not a list. Caller must tv_list_unref.
list_T *nvim_cmdexpand_nlua_call_user_expand_retlist(expand_T *xp)
{
  typval_T rettv = TV_INITIAL_VALUE;
  nlua_call_user_expand_func(xp, &rettv);
  if (rettv.v_type != VAR_LIST) {
    tv_clear(&rettv);
    return NULL;
  }
  list_T *li = rettv.vval.v_list;
  tv_list_ref(li);
  tv_clear(&rettv);
  return li;
}

/// Convert list_T to a newly-allocated char ** array (for Rust FFI).
/// Skips non-string items.  Unrefs the list.
void nvim_cmdexpand_list_to_string_matches(list_T *list, char ***matches, int *numMatches)
{
  garray_T ga;
  ga_init(&ga, (int)sizeof(char *), 3);
  TV_LIST_ITER_CONST(list, li, {
    if (TV_LIST_ITEM_TV(li)->v_type != VAR_STRING
        || TV_LIST_ITEM_TV(li)->vval.v_string == NULL) {
      continue;
    }
    GA_APPEND(char *, &ga, xstrdup(TV_LIST_ITEM_TV(li)->vval.v_string));
  });
  tv_list_unref(list);
  *matches = ga.ga_data;
  *numMatches = ga.ga_len;
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

// cmdline_del -- used by nvim_cmdexpand_cmdline_del accessor only (kept for now)
static void cmdline_del(CmdlineInfo *cclp, int from)
{
  assert(cclp->cmdpos <= cclp->cmdlen);
  memmove(cclp->cmdbuff + from, cclp->cmdbuff + cclp->cmdpos,
          (size_t)cclp->cmdlen - (size_t)cclp->cmdpos + 1);
  cclp->cmdlen -= cclp->cmdpos - from;
  cclp->cmdpos = from;
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
