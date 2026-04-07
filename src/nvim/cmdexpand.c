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
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
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

// Rust FFI declarations
extern int rs_expand_tags(bool tagnames, char *pat, int *num_file, char ***file);
extern int rs_expand_setting_subtract(void *xp, void *regmatch, int *numMatches, char ***matches);
extern int rs_global_stl_height(void);
extern void rs_last_status(int morewin);
extern int rs_cmdline_fuzzy_completion_supported(int context);
extern void redraw_wildmenu(expand_T *xp, int num_matches, char **matches, int match,
                            bool showtail);
extern void cmdline_pum_create(void *ccline, expand_T *xp, char **matches, int num_matches,
                               bool showtail, bool noselect);

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

extern int rs_expand_pattern_in_buf(char *pat, int dir, char ***matches, int *numMatches);
extern int rs_expand_files_and_dirs(expand_T *xp, char *pat, char ***matches, int *numMatches,
                                    int flags, int options);
extern const char *rs_set_context_by_cmdname(const char *cmd, int cmdidx, expand_T *xp,
                                              const char *arg, uint32_t argt, int context,
                                              bool forceit);
extern const char *rs_set_one_cmd_context(expand_T *xp, const char *buff);

// C accessor for Rust FFI
unsigned nvim_get_wop_flags(void) { return wop_flags; }
int nvim_get_compl_match_array_not_null(void) { return compl_match_array != NULL; }
int nvim_cmdexpand_get_may_expand_pattern(void) { return may_expand_pattern ? 1 : 0; }
// C accessors for expand_T fields (Rust FFI)

int nvim_expand_get_context(const expand_T *xp) { return xp ? xp->xp_context : EXPAND_NOTHING; }
void nvim_expand_set_context(expand_T *xp, int context)
{
  if (xp) {
    xp->xp_context = context;
  }
}

int nvim_get_cmdline_win_is_null(void) { return cmdline_win == NULL; }
int nvim_get_pum_want_active(void) { return pum_want.active; }
int nvim_get_pum_want_item(void) { return pum_want.item; }
void nvim_set_pum_want_active(int val) { pum_want.active = (val != 0); }

int nvim_cmdexpand_rem_backslash(const char *p) { return rem_backslash(p); }
int nvim_cmdexpand_mb_ptr_adv_len(const char *p) { return utfc_ptr2len(p); }
void nvim_expand_clear(expand_T *xp)
{
  if (xp) {
    CLEAR_POINTER(xp);
  }
}

void nvim_expand_free_wild(expand_T *xp)
{
  if (xp) {
    FreeWild(xp->xp_numfiles, xp->xp_files);
  }
}

void nvim_expand_clear_orig(expand_T *xp)
{
  if (xp) {
    XFREE_CLEAR(xp->xp_orig);
  }
}

void nvim_clear_cmdline_orig(void) { XFREE_CLEAR(cmdline_orig); }
void nvim_set_compl_selected(int val) { compl_selected = val; }
int nvim_get_cmd_showtail(void) { return cmd_showtail; }
void nvim_cmdexpand_pum_display(int changed_array) { cmdline_pum_display(changed_array != 0); }
void nvim_cmdexpand_pum_create_for_nav(expand_T *xp, int showtail, int noselect)
{
  cmdline_pum_create(get_cmdline_info(), xp, xp->xp_files, xp->xp_numfiles,
                     showtail != 0, noselect != 0);
}

void nvim_cmdexpand_redraw_wildmenu(expand_T *xp, int num_matches, int findex, int showtail)
{
  redraw_wildmenu(xp, num_matches, xp->xp_files, findex, showtail != 0);
}

int nvim_cmdexpand_expand_from_context(expand_T *xp, const char *pat, int options)
{
  return ExpandFromContext(xp, (char *)pat, &xp->xp_files, &xp->xp_numfiles, options);
}

void nvim_cmdexpand_expand_escape(expand_T *xp, const char *str, int options)
{
  rs_expand_escape(xp, (char *)str, xp->xp_numfiles, xp->xp_files, options);
}

int nvim_cmdexpand_match_suffix(expand_T *xp, int i)
{
  if (!xp || i < 0 || i >= xp->xp_numfiles || !xp->xp_files) {
    return 0;
  }
  return match_suffix(xp->xp_files[i]);
}

void nvim_cmdexpand_semsg_nomatch(const char *str) { semsg(_(e_nomatch2), str); }
void nvim_cmdexpand_emsg_toomany(void) { emsg(_(e_toomany)); }
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

// nvim_expand_free_old_matches is implemented in Rust (helpers.rs)

// nvim_get_got_int already exists in ex_eval.c

/// xstpcpy wrapper (for Rust FFI): copies src to dst, returns pointer past NUL.
char *nvim_cmdexpand_xstpcpy(char *dst, const char *src) { return xstpcpy(dst, src); }
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

void nvim_expand_set_pattern(expand_T *xp, char *pattern)
{
  if (xp) {
    xp->xp_pattern = pattern;
  }
}

char *nvim_cmdexpand_get_xp_pattern(expand_T *xp) { return xp ? xp->xp_pattern : NULL; }
int nvim_cmdexpand_ascii_iswhite(int c) { return ascii_iswhite(c); }
int nvim_cmdexpand_vim_isfilec_or_wc(int c) { return vim_isfilec_or_wc(c); }
int nvim_cmdexpand_vim_isIDc(int c) { return vim_isIDc((uint8_t)c); }
int nvim_cmdexpand_get_cmdpos(void) { return get_cmdline_info()->cmdpos; }
char *nvim_cmdexpand_get_cmdbuff(void) { return get_cmdline_info()->cmdbuff; }
// parse_pattern_and_range is implemented in Rust (cmdexpand/src/incsearch.rs)
extern bool parse_pattern_and_range(pos_T *incsearch_start, int *search_delim,
                                    int *skiplen, int *patlen);

int nvim_cmdexpand_parse_pattern_and_range(int *skiplen, int *patlen)
{
  int dummy;
  return parse_pattern_and_range(&pre_incsearch_pos, &dummy, skiplen, patlen);
}

void nvim_cmdexpand_emsg_off_inc(void) { emsg_off++; }
void nvim_cmdexpand_emsg_off_dec(void) { emsg_off--; }
void nvim_cmdexpand_set_breakpt_expand_what(int val) { breakpt_expand_what = val; }
void nvim_cmdexpand_set_filetype_expand_what(int val) { filetype_expand_what = val; }
int nvim_cmdexpand_get_breakpt_expand_what(void) { return (int)breakpt_expand_what; }
int nvim_cmdexpand_get_filetype_expand_what(void) { return (int)filetype_expand_what; }

int nvim_cmdexpand_searchit(pos_T *pos, pos_T *end_pos, int dir, char *pat,
                            size_t patlen, int options)
{
  return searchit(NULL, curbuf, pos, end_pos, (Direction)dir, pat, patlen,
                  1L, options, RE_LAST, NULL);
}

int nvim_cmdexpand_curbuf_line_count(void) { return curbuf->b_ml.ml_line_count; }
int nvim_cmdexpand_char_avail(void) { return char_avail(); }
int nvim_cmdexpand_vpeekc(void) { return vpeekc(); }
int nvim_cmdexpand_get_search_first_line(void) { return search_first_line; }
int nvim_cmdexpand_get_search_last_line(void) { return search_last_line; }
pos_T nvim_cmdexpand_get_pre_incsearch_pos(void) { return pre_incsearch_pos; }
void nvim_cmdexpand_do_pum_display(int changed_array)
{
  pum_display(compl_match_array, compl_match_arraysize, compl_selected,
              changed_array != 0, compl_startcol);
}

void nvim_cmdexpand_do_pum_remove(int defer_redraw)
{
  pum_undisplay(defer_redraw == 0);
  XFREE_CLEAR(compl_match_array);
  compl_match_arraysize = 0;
}

void nvim_cmdexpand_do_pum_cleanup(void)
{
  cmdline_pum_remove(false);
  wildmenu_cleanup(get_cmdline_info());
}

void nvim_cmdexpand_set_compl_startcol(int val) { compl_startcol = val; }
void nvim_cmdexpand_set_compl_match_arraysize(int val) { compl_match_arraysize = val; }
void *nvim_cmdexpand_alloc_compl_match_array(int numMatches)
{
  compl_match_array = xmalloc(sizeof(pumitem_T) * (size_t)numMatches);
  return compl_match_array;
}

void nvim_cmdexpand_set_pum_text(int i, char *text)
{
  compl_match_array[i].pum_text = text;
  compl_match_array[i].pum_info = NULL;
  compl_match_array[i].pum_extra = NULL;
  compl_match_array[i].pum_kind = NULL;
  compl_match_array[i].pum_user_abbr_hlattr = -1;
  compl_match_array[i].pum_user_kind_hlattr = -1;
}

void *nvim_cmdexpand_get_msg_grid_adj_ptr(void) { return &msg_grid_adj; }
void *nvim_cmdexpand_get_default_gridview_ptr(void) { return &default_gridview; }
char *nvim_cmdexpand_get_compl_pattern(void)
{
  expand_T *xp = get_cmdline_info()->xpc;
  return xp == NULL ? NULL : xp->xp_orig;
}

int nvim_cmdexpand_ccline_xpc_supports_fuzzy(void)
{
  expand_T *xp = get_cmdline_info()->xpc;
  return xp != NULL && rs_cmdline_fuzzy_completion_supported(xp->xp_context);
}

int nvim_cmdexpand_get_cmdfirstc(void) { return get_cmdline_info()->cmdfirstc; }
int nvim_cmdexpand_get_input_fn(void) { return get_cmdline_info()->input_fn ? 1 : 0; }
int nvim_cmdexpand_get_cmdlen(void) { return get_cmdline_info()->cmdlen; }
int nvim_cmdexpand_get_p_wc(void) { return (int)p_wc; }
void nvim_cmdexpand_set_search_first_line(int val) { search_first_line = val; }
int nvim_cmdexpand_get_key_left(void) { return K_LEFT; }
int nvim_cmdexpand_get_key_right(void) { return K_RIGHT; }
int nvim_cmdexpand_get_key_down(void) { return K_DOWN; }
int nvim_cmdexpand_get_key_up(void) { return K_UP; }
int nvim_cmdexpand_get_key_kenter(void) { return K_KENTER; }

int nvim_cmdexpand_script_id_valid(int idx) { return SCRIPT_ID_VALID(idx + 1) ? 1 : 0; }
char *nvim_cmdexpand_get_script_name(int idx)
{
  scriptitem_T *si = SCRIPT_ITEM(idx + 1);
  home_replace(NULL, si->sn_name, NameBuff, MAXPATHL, true);
  return NameBuff;
}

int nvim_cmdexpand_get_p_wic(void) { return (int)p_wic; }
int nvim_cmdexpand_get_ccline_xp_context(void) { return get_cmdline_info()->xp_context; }
char *nvim_cmdexpand_get_ccline_xp_arg(void) { return get_cmdline_info()->xp_arg; }
void nvim_cmdexpand_set_context_for_expression(expand_T *xp, char *str, int cmdidx)
{
  set_context_for_expression(xp, str, (cmdidx_T)cmdidx);
}

void nvim_cmdexpand_set_cmdlen(int val) { get_cmdline_info()->cmdlen = val; }
void nvim_cmdexpand_set_cmdpos(int val) { get_cmdline_info()->cmdpos = val; }
int nvim_cmdexpand_get_cmdbufflen(void) { return get_cmdline_info()->cmdbufflen; }
// nvim_cmdexpand_cmdline_del: calls Rust rs_cmdexpand_cmdline_del
extern void rs_cmdexpand_cmdline_del(int from);
void nvim_cmdexpand_cmdline_del(int from) { rs_cmdexpand_cmdline_del(from); }
void nvim_cmdexpand_set_key_typed(int val) { KeyTyped = (bool)val; }
int nvim_cmdexpand_get_key_typed(void) { return (int)KeyTyped; }
void nvim_cmdexpand_put_on_cmdline(const char *str, int len, int redraw) { put_on_cmdline(str, len, (bool)redraw); }
int nvim_cmdexpand_utf_head_off(const char *base, const char *p) { return utf_head_off(base, p); }
int nvim_cmdexpand_get_redrawing_disabled(void) { return RedrawingDisabled; }
void nvim_cmdexpand_set_redrawing_disabled(int val) { RedrawingDisabled = val; }
void nvim_cmdexpand_set_no_hlsearch(int val) { set_no_hlsearch((bool)val); }
int nvim_cmdexpand_get_wm_scrolled(void) { return WM_SCROLLED; }
void nvim_cmdexpand_dec_cmdline_row(void) { cmdline_row--; }
void nvim_cmdexpand_win_redraw_last_status(void) { win_redraw_last_status(topframe); }
void nvim_cmdexpand_redraw_statuslines(void) { redraw_statuslines(); }
int nvim_cmdexpand_get_pathsep(void) { return PATHSEP; }
void nvim_cmdexpand_expand_generic(const char *pat, expand_T *xp, regmatch_T *regmatch,
                                   char ***matches, int *numMatches,
                                   CompleteListItemGetter func, int escaped)
{
  ExpandGeneric(pat, xp, regmatch, matches, numMatches, func, (bool)escaped);
}

void nvim_cmdexpand_regmatch_set_rm_ic(regmatch_T *rmp, int val) { rmp->rm_ic = (bool)val; }
void nvim_cmdexpand_regmatch_set_regprog(regmatch_T *rmp, void *prog)
{
  rmp->rm_ic = false;
  rmp->regprog = (regprog_T *)prog;
}

int nvim_cmdexpand_find_help_tags(const char *pat, int *numMatches, char ***matches)
{
  if (find_help_tags(*pat == NUL ? "help" : pat, numMatches, matches, false) == OK) {
    cleanup_help_tags(*numMatches, *matches);
    return 1;
  }
  return 0;
}

int nvim_cmdexpand_expand_old_setting(int *numMatches, char ***matches)
{
  return ExpandOldSetting(numMatches, matches);
}

int nvim_cmdexpand_expand_bufnames(const char *pat, int *numMatches, char ***matches, int options)
{
  return ExpandBufnames(pat, numMatches, matches, options);
}

int nvim_cmdexpand_expand_rtdir(const char *pat, int flags, int *numMatches, char ***matches,
                                char **directories)
{
  return ExpandRTDir(pat, flags, numMatches, matches, directories);
}

int nvim_cmdexpand_expand_pack_add_dir(const char *pat, int *numMatches, char ***matches)
{
  return ExpandPackAddDir(pat, numMatches, matches);
}

int nvim_cmdexpand_expand_runtime_cmd(const char *pat, int *numMatches, char ***matches)
{
  return expand_runtime_cmd(pat, numMatches, matches);
}

int nvim_cmdexpand_expand_settings(expand_T *xp, regmatch_T *regmatch, const char *pat,
                                   int *numMatches, char ***matches, int fuzzy)
{
  return ExpandSettings(xp, regmatch, pat, numMatches, matches, (bool)fuzzy);
}

int nvim_cmdexpand_expand_string_setting(expand_T *xp, regmatch_T *regmatch,
                                          int *numMatches, char ***matches)
{
  return ExpandStringSetting(xp, regmatch, numMatches, matches);
}

int nvim_cmdexpand_expand_mappings(const char *pat, regmatch_T *regmatch,
                                   int *numMatches, char ***matches)
{
  return ExpandMappings(pat, regmatch, numMatches, matches);
}

int nvim_cmdexpand_expand_argopt(const char *pat, expand_T *xp, regmatch_T *regmatch,
                                  char ***matches, int *numMatches)
{
  return expand_argopt(pat, xp, regmatch, matches, numMatches);
}

int nvim_cmdexpand_nlua_expand_get_matches(int *numMatches, char ***matches)
{
  return nlua_expand_get_matches(numMatches, matches);
}

int nvim_cmdexpand_get_dip_start_opt(void) { return DIP_START + DIP_OPT; }
int nvim_cmdexpand_get_re_magic(void) { return RE_MAGIC; }
int nvim_cmdexpand_magic_isset(void) { return rs_magic_isset(); }
char *nvim_cmdexpand_make_snr_pattern(const char *suffix)
{
  const size_t len = strlen(suffix) + 20;
  char *tofree = xmalloc(len);
  snprintf(tofree, len, "^<SNR>\\d\\+_%s", suffix);
  return tofree;
}

CompleteListItemGetter nvim_cmdexpand_get_fn_get_command_name(void) { return get_command_name; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_history_arg(void) { return get_history_arg; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_user_commands(void) { return get_user_commands; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_user_cmd_addr_type(void) { return get_user_cmd_addr_type; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_user_cmd_flags(void) { return get_user_cmd_flags; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_user_cmd_nargs(void) { return get_user_cmd_nargs; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_user_cmd_complete(void) { return get_user_cmd_complete; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_user_var_name(void) { return get_user_var_name; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_function_name(void) { return get_function_name; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_user_func_name(void) { return get_user_func_name; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_expr_name(void) { return get_expr_name; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_menu_name(void) { return get_menu_name; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_menu_names(void) { return get_menu_names; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_syntax_name(void) { return get_syntax_name; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_syntime_arg(void) { return get_syntime_arg; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_highlight_name(void) { return get_highlight_name; }
// Rust implementations (exported under same C names)
extern char *expand_get_event_name(expand_T *xp, int idx);
extern char *expand_get_augroup_name(expand_T *xp, int idx);
CompleteListItemGetter nvim_cmdexpand_get_fn_expand_get_event_name(void) { return expand_get_event_name; }
CompleteListItemGetter nvim_cmdexpand_get_fn_expand_get_augroup_name(void) { return expand_get_augroup_name; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_sign_name(void) { return get_sign_name; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_profile_name(void) { return get_profile_name; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_lang_arg(void) { return get_lang_arg; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_locales(void) { return get_locales; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_env_name(void) { return get_env_name; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_users(void) { return get_users; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_arglist_name(void) { return get_arglist_name; }
CompleteListItemGetter nvim_cmdexpand_get_fn_get_healthcheck_names(void) { return get_healthcheck_names; }
char *nvim_cmdexpand_xstrnsave(const char *s, size_t n) { return xstrnsave(s, n); }
void nvim_cmdexpand_set_cmd_showtail(int val) { cmd_showtail = (bool)val; }
void nvim_cmdexpand_set_may_expand_pattern(int val) { may_expand_pattern = (bool)val; }
void nvim_cmdexpand_copy_pre_incsearch_pos(expand_T *xp) { pre_incsearch_pos = xp->xp_pre_incsearch_pos; }
void nvim_cmdexpand_save_cmdline_orig(void)
{
  CmdlineInfo *ccline = get_cmdline_info();
  xfree(cmdline_orig);
  cmdline_orig = xstrnsave(ccline->cmdbuff, (size_t)ccline->cmdlen);
}

// nvim_cmdexpand_apply_expansion: implemented in Rust (helpers.rs)
extern void rs_cmdexpand_apply_expansion(expand_T *xp, int i, const char *p, int plen);
void nvim_cmdexpand_apply_expansion(expand_T *xp, int i, const char *p, int plen)
{
  rs_cmdexpand_apply_expansion(xp, i, p, plen);
}

void nvim_cmdexpand_nlua_expand_pat(expand_T *xp) { nlua_expand_pat(xp); }
void nvim_cmdexpand_msg_putchar(int c) { msg_putchar(c); }
void nvim_cmdexpand_msg_ext_set_kind(const char *kind) { msg_ext_set_kind(kind); }
void nvim_cmdexpand_free_wild(int count, char **files) { FreeWild(count, files); }
void nvim_cmdexpand_pum_clear(void) { pum_clear(); }
void nvim_cmdexpand_set_compl_selected(int val) { compl_selected = val; }
void nvim_cmdexpand_pum_create_from_matches(expand_T *xp, char **matches, int num_matches,
                                            int showtail, int noselect)
{
  cmdline_pum_create(get_cmdline_info(), xp, matches, num_matches,
                     showtail != 0, noselect != 0);
}

void nvim_cmdexpand_redraw_wildmenu_ex(expand_T *xp, int num_matches, char **matches,
                                       int findex, int showtail)
{
  redraw_wildmenu(xp, num_matches, matches, findex, showtail != 0);
}

void nvim_cmdexpand_msg_clr_eos(void) { msg_clr_eos(); }
int nvim_cmdexpand_msg_outtrans(const char *str, int attr, int maxcol) { return msg_outtrans(str, attr, (bool)maxcol); }
void nvim_cmdexpand_msg_outtrans_long(const char *str, int attr) { msg_outtrans_long(str, attr); }
void nvim_cmdexpand_msg_advance(int col) { msg_advance(col); }
char *nvim_cmdexpand_home_replace_match(const char *s)
{
  home_replace(NULL, s, NameBuff, MAXPATHL, true);
  return NameBuff;
}

char *nvim_cmdexpand_expand_env_save_opt(const char *str) { return expand_env_save_opt((char *)str, true); }
char *nvim_cmdexpand_backslash_halve_save(const char *str) { return backslash_halve_save((char *)str); }
int nvim_cmdexpand_os_isdir(const char *str) { return os_isdir(str); }
int nvim_cmdexpand_vim_strsize(const char *str) { return vim_strsize(str); }
void nvim_cmdexpand_msg_puts_hl(const char *str, int attr, int maxcol) { msg_puts_hl(str, attr, (bool)maxcol); }
char *nvim_cmdexpand_show_match(char **matches, int m, int showtail)
{
  return showtail ? rs_showmatches_gettail(matches[m], false) : matches[m];
}

int nvim_cmdexpand_compl_use_pum(int need_wildmenu) { return rs_cmdline_compl_use_pum(need_wildmenu); }

int nvim_cmdexpand_tv_get_type(typval_T *argvars, int idx) { return (int)argvars[idx].v_type; }
int nvim_cmdexpand_tv_check_for_string_arg(typval_T *argvars, int idx) { return tv_check_for_string_arg(argvars, idx); }
const char *nvim_cmdexpand_tv_get_string(typval_T *argvars, int idx) { return tv_get_string(&argvars[idx]); }
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

void nvim_cmdexpand_tv_set_string(typval_T *rettv, char *str)
{
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = str;
}

/// Allocate a dict and set rettv to it (for Rust FFI).
void nvim_cmdexpand_tv_dict_alloc_ret(typval_T *rettv) { tv_dict_alloc_ret(rettv); }
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

int nvim_cmdexpand_pum_visible(void) { return pum_visible(); }
int nvim_cmdexpand_get_ccline_xp_selected(void)
{
  CmdlineInfo *ccline = get_cmdline_info();
  if (ccline == NULL || ccline->xpc == NULL) {
    return 0;
  }
  return ccline->xpc->xp_selected;
}

int nvim_cmdexpand_get_ccline_xp_numfiles(void)
{
  CmdlineInfo *ccline = get_cmdline_info();
  if (ccline == NULL || ccline->xpc == NULL) {
    return -1;
  }
  return ccline->xpc->xp_numfiles;
}

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

int nvim_cmdexpand_ccline_has_xp_files(void)
{
  CmdlineInfo *ccline = get_cmdline_info();
  return ccline != NULL && ccline->xpc != NULL && ccline->xpc->xp_files != NULL;
}

int nvim_cmdexpand_cmdcomplete_str_to_type(const char *type) { return cmdcomplete_str_to_type(type); }
char *nvim_cmdexpand_cmdcomplete_type_to_str(int ctx, const char *arg) { return cmdcomplete_type_to_str(ctx, arg); }
const char *nvim_cmdexpand_get_cmdline_orig(void) { return cmdline_orig; }
/// set_context_in_menu_cmd wrapper (for Rust FFI).
void nvim_cmdexpand_set_context_in_menu_cmd(expand_T *xp, const char *cmd, char *arg, bool delim_optional)
{
  set_context_in_menu_cmd(xp, cmd, arg, delim_optional);
}

/// set_context_in_sign_cmd wrapper (for Rust FFI).
void nvim_cmdexpand_set_context_in_sign_cmd(expand_T *xp, char *arg) { set_context_in_sign_cmd(xp, arg); }
/// set_context_in_runtime_cmd wrapper (for Rust FFI).
void nvim_cmdexpand_set_context_in_runtime_cmd(expand_T *xp, char *arg) { set_context_in_runtime_cmd(xp, arg); }
/// VAR_UNKNOWN constant (for Rust FFI).
int nvim_cmdexpand_get_var_unknown(void) { return VAR_UNKNOWN; }
/// filetype_expand_what = EXP_FILETYPECMD_ALL (for Rust FFI).
void nvim_cmdexpand_set_filetype_expand_all(void) { filetype_expand_what = EXP_FILETYPECMD_ALL; }
/// emsg(_(e_invarg)) wrapper (for Rust FFI).
void nvim_cmdexpand_emsg_invarg(void) { emsg(_(e_invarg)); }
/// semsg(_(e_invarg2), type) wrapper (for Rust FFI).
void nvim_cmdexpand_semsg_invarg2(const char *type) { semsg(_(e_invarg2), type); }
/// xmemdupz wrapper (for Rust FFI).
char *nvim_cmdexpand_xmemdupz(const char *s, size_t len) { return xmemdupz(s, len); }
/// call_user_expand_func with call_func_retlist (for Rust FFI).
list_T *nvim_cmdexpand_call_user_expand_retlist(expand_T *xp) { return call_user_expand_func(call_func_retlist, xp); }
/// call_user_expand_func with call_func_retstr (for Rust FFI).
char *nvim_cmdexpand_call_user_expand_retstr(expand_T *xp) { return call_user_expand_func(call_func_retstr, xp); }
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

// cmdline_del and parse_pattern_and_range are now implemented in Rust (cmdexpand/src/incsearch.rs)
// and exported with the same C symbol name via #[unsafe(export_name = "parse_pattern_and_range")].
