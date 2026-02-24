// ex_cmds.c: some functions for command line commands

#include <assert.h>
#include <ctype.h>
#include <float.h>
#include <inttypes.h>
#include <limits.h>
#include <math.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "auto/config.h"
#include "klib/kvec.h"
#include "nvim/api/private/helpers.h"
#include "nvim/arglist.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/buffer_updates.h"
#include "nvim/bufwrite.h"
#include "nvim/change.h"
#include "nvim/channel.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/cmdhist.h"
#include "nvim/cursor.h"
#include "nvim/decoration.h"
#include "nvim/diff.h"
#include "nvim/digraph.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds2.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_getln.h"
#include "nvim/extmark.h"
#include "nvim/extmark_defs.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/help.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/indent.h"
#include "nvim/input.h"
#include "nvim/macros_defs.h"
#include "nvim/main.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/fs_defs.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/shell.h"
#include "nvim/os/time.h"
#include "nvim/path.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/profile.h"
#include "nvim/quickfix.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/search.h"
#include "nvim/spell.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/terminal.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

/// Case matching style to use for :substitute
typedef enum {
  kSubHonorOptions = 0,  ///< Honor the user's 'ignorecase'/'smartcase' options
  kSubIgnoreCase,        ///< Ignore case of the search
  kSubMatchCase,         ///< Match case of the search
} SubIgnoreType;

/// Flags kept between calls to :substitute.
typedef struct {
  bool do_all;          ///< do multiple substitutions per line
  bool do_ask;          ///< ask for confirmation
  bool do_count;        ///< count only
  bool do_error;        ///< if false, ignore errors
  bool do_print;        ///< print last line with subs
  bool do_list;         ///< list last line with subs
  bool do_number;       ///< list last line with line nr
  SubIgnoreType do_ic;  ///< ignore case flag
} subflags_T;

/// Partial result of a substitution during :substitute.
/// Numbers refer to the buffer _after_ substitution
typedef struct {
  lpos_T start;  // start of the match
  lpos_T end;    // end of the match
  linenr_T pre_match;  // where to begin showing lines before the match
} SubResult;

// Collected results of a substitution for showing them in
// the preview window
typedef struct {
  kvec_t(SubResult) subresults;
  linenr_T lines_needed;  // lines needed in the preview window
} PreviewLines;

#include "ex_cmds_shim.c.generated.h"

// Rust FFI declarations
extern int rs_ml_find_line_or_offset(buf_T *buf, linenr_T lnum, int *offp, bool no_ff);
extern int rs_win_valid(win_T *win);
extern int rs_win_valid_any_tab(win_T *win);
extern void rs_reset_VIsual(void);
extern void rs_check_lnums(int do_curwin);
extern int rs_hasAnyFolding(win_T *win);
extern void rs_foldMoveRange(win_T *wp, garray_T *gap, linenr_T line1, linenr_T line2,
                             linenr_T dest);
extern void rs_foldUpdateAll(win_T *win);
extern int rs_magic_isset(void);
extern void rs_diff_buf_add(buf_T *buf);
extern void rs_diff_invalidate(buf_T *buf);
extern int rs_check_regexp_delim(int c);
extern char *rs_skip_substitute(char *start, int delimiter);
extern bool rs_do_sub_msg(bool count_only);

// ExArg accessors
int nvim_exarg_get_cmdidx(exarg_T *eap) { return (int)eap->cmdidx; }
const char *nvim_exarg_get_arg(exarg_T *eap) { return eap->arg; }
linenr_T nvim_exarg_get_line1(exarg_T *eap) { return eap->line1; }
linenr_T nvim_exarg_get_line2(exarg_T *eap) { return eap->line2; }
int nvim_exarg_get_addr_count(exarg_T *eap) { return eap->addr_count; }
int nvim_exarg_get_forceit(exarg_T *eap) { return eap->forceit ? 1 : 0; }
int nvim_exarg_get_flags(exarg_T *eap) { return eap->flags; }
void nvim_exarg_set_line2(exarg_T *eap, linenr_T line2) { eap->line2 = line2; }

// Window/buffer accessors
int nvim_curwin_get_w_p_rl(void) { return curwin->w_p_rl; }
int nvim_curbuf_get_b_p_tw(void) { return (int)curbuf->b_p_tw; }
int nvim_curbuf_get_b_p_wm(void) { return (int)curbuf->b_p_wm; }
int nvim_curwin_get_view_width(void) { return curwin->w_view_width; }
void nvim_curwin_set_cursor_lnum(linenr_T lnum) { curwin->w_cursor.lnum = lnum; }
int nvim_linetabsize_str(char *s) { return linetabsize_col(0, s); }
int nvim_is_one_window(void) { return ONE_WINDOW ? 1 : 0; }
int64_t nvim_curwin_get_p_scr(void) { return curwin->w_p_scr; }
int nvim_curwin_get_view_height(void) { return curwin->w_view_height; }
void nvim_set_ex_no_reprint(int val) { ex_no_reprint = val != 0; }
int nvim_cmdmod_has_lockmarks(void) { return (cmdmod.cmod_flags & CMOD_LOCKMARKS) != 0; }
int nvim_curbuf_get_b_p_ai(void) { return curbuf->b_p_ai; }
void nvim_check_pos_visual(void) { check_pos(curbuf, &VIsual); }
void nvim_transchar_nonprint_curbuf(char *buf, int c) { transchar_nonprint(curbuf, buf, c); }

void nvim_curbuf_set_op_start(linenr_T lnum, colnr_T col)
{
  curbuf->b_op_start.lnum = lnum;
  curbuf->b_op_start.col = col;
}

void nvim_curbuf_set_op_end(linenr_T lnum, colnr_T col)
{
  curbuf->b_op_end.lnum = lnum;
  curbuf->b_op_end.col = col;
}

void nvim_msg_multiline_cstr(const char *s, int hl_id, bool check_int, bool hist, bool *need_clear)
{
  msg_multiline(cstr_as_string(s), hl_id, check_int, hist, need_clear);
}

// print_line accessors
int nvim_curwin_get_w_p_nu(void) { return curwin->w_p_nu; }
int nvim_number_width_curwin(void) { return number_width(curwin); }
int nvim_get_info_message(void) { return info_message; }
void nvim_msg_prt_line(const char *s, int list) { msg_prt_line((char *)s, list != 0); }
int nvim_message_filtered(const char *msg) { return message_filtered((char *)msg); }
void nvim_msg_ext_set_kind_excmd(const char *kind) { msg_ext_set_kind(kind); }
void nvim_msg_puts_hl_excmd(const char *s, int hl_id) { msg_puts_hl(s, hl_id, false); }

// --- Sort/uniq accessor functions for Rust FFI ---

// Regex accessors (opaque regmatch_T handle)
void *nvim_excmds_regcomp(const char *pat, int magic_val)
{
  regmatch_T *rm = xcalloc(1, sizeof(regmatch_T));
  rm->regprog = vim_regcomp(pat, magic_val);
  if (rm->regprog == NULL) {
    xfree(rm);
    return NULL;
  }
  return rm;
}

int nvim_excmds_regexec(void *rm, const char *line)
{
  return vim_regexec((regmatch_T *)rm, (char *)line, 0);
}

void nvim_excmds_regfree(void *rm)
{
  if (rm != NULL) {
    vim_regfree(((regmatch_T *)rm)->regprog);
    xfree(rm);
  }
}

const char *nvim_excmds_regmatch_startp0(const void *rm)
{
  return ((const regmatch_T *)rm)->startp[0];
}

const char *nvim_excmds_regmatch_endp0(const void *rm)
{
  return ((const regmatch_T *)rm)->endp[0];
}

void nvim_excmds_regmatch_set_ic(void *rm, int ic)
{
  ((regmatch_T *)rm)->rm_ic = ic;
}

// Search/skip accessors
const char *nvim_excmds_last_search_pat(void) { return last_search_pat(); }
char *nvim_excmds_check_nextcmd(const char *p) { return check_nextcmd((char *)p); }
char *nvim_excmds_skip_regexp_err(const char *p, int delim)
{
  return skip_regexp_err((char *)p, delim, true);
}

// Number parsing wrapper
void nvim_excmds_str2nr(const char *s, int what, int64_t *result)
{
  varnumber_T val = 0;
  vim_str2nr(s, NULL, NULL, what, &val, NULL, 0, false, NULL);
  *result = (int64_t)val;
}

// Skip functions
char *nvim_excmds_skiptohex(const char *p) { return skiptohex((char *)p); }
char *nvim_excmds_skiptobin(const char *p) { return (char *)skiptobin((char *)p); }
char *nvim_excmds_skiptodigit(const char *p) { return skiptodigit((char *)p); }

// Interrupt check
int nvim_excmds_got_int(void) { return got_int; }

// Error message wrappers (can't call variadic semsg from Rust)
void nvim_excmds_semsg_invarg2(const char *p) { semsg(_(e_invarg2), p); }
void nvim_excmds_emsg_invarg(void) { emsg(_(e_invarg)); }
void nvim_excmds_emsg_noprevre(void) { emsg(_(e_noprevre)); }
void nvim_excmds_emsg_interr(void) { emsg(_(e_interr)); }

// Global option accessor
int nvim_excmds_get_p_ic(void) { return p_ic; }

// Exarg mutation
void nvim_exarg_set_nextcmd(exarg_T *eap, const char *p)
{
  eap->nextcmd = (char *)p;
}

int nvim_exarg_is_nextcmd_null(exarg_T *eap) { return eap->nextcmd == NULL ? 1 : 0; }

// Mark/extmark wrappers
void nvim_excmds_mark_adjust(linenr_T line1, linenr_T line2, int amount, int amount_after,
                              int etype)
{
  mark_adjust(line1, line2, (long)amount, (long)amount_after, (ExtmarkOp)etype);
}

void nvim_excmds_extmark_splice(int start_row, int start_col,
                                 int old_row, int old_col, int64_t old_byte,
                                 int new_row, int new_col, int64_t new_byte,
                                 int etype)
{
  extmark_splice(curbuf, start_row, start_col,
                 old_row, old_col, (bcount_t)old_byte,
                 new_row, new_col, (bcount_t)new_byte,
                 (ExtmarkOp)etype);
}

// --- do_move accessor functions for Rust FFI ---

void nvim_excmds_mark_adjust_nofold(linenr_T line1, linenr_T line2,
                                     int amount, int amount_after, int etype)
{
  mark_adjust_nofold(line1, line2, (long)amount, (long)amount_after, (ExtmarkOp)etype);
}

int64_t nvim_excmds_ml_find_line_or_offset(linenr_T lnum)
{
  return (int64_t)rs_ml_find_line_or_offset(curbuf, lnum, NULL, true);
}

int nvim_excmds_ml_delete_flags(linenr_T lnum, int flags)
{
  return ml_delete_flags(lnum, flags);
}

void nvim_excmds_extmark_move_region(int start_row, int start_col, int64_t start_byte,
                                      int extent_row, int extent_col, int64_t extent_byte,
                                      int new_row, int new_col, int64_t new_byte, int etype)
{
  extmark_move_region(curbuf, start_row, (colnr_T)start_col, (bcount_t)start_byte,
                      extent_row, (colnr_T)extent_col, (bcount_t)extent_byte,
                      new_row, (colnr_T)new_col, (bcount_t)new_byte, (ExtmarkOp)etype);
}

void nvim_excmds_buf_updates_send_changes(linenr_T lnum, int64_t added, int64_t deleted)
{
  buf_updates_send_changes(curbuf, lnum, added, deleted);
}

// Wrap the FOR_ALL_TAB_WINDOWS loop for fold move range.
void nvim_excmds_fold_move_range_all_wins(linenr_T line1, linenr_T line2, linenr_T dest)
{
  FOR_ALL_TAB_WINDOWS(tab, win) {
    if (win->w_buffer == curbuf) {
      rs_foldMoveRange(win, &win->w_folds, line1, line2, dest);
    }
  }
}

void nvim_excmds_disable_fold_update_inc(void) { disable_fold_update++; }
void nvim_excmds_disable_fold_update_dec(void) { disable_fold_update--; }

int nvim_excmds_global_busy(void) { return global_busy; }
int64_t nvim_excmds_p_report(void) { return (int64_t)p_report; }

void nvim_excmds_smsg_lines_moved(int64_t num_lines)
{
  smsg(0, NGETTEXT("%" PRId64 " line moved",
                   "%" PRId64 " lines moved", (int)num_lines),
       num_lines);
}

void nvim_excmds_emsg_e134(void) {
  emsg(_("E134: Cannot move a range of lines into itself"));
}

// --- ex_append FFI accessors ---
// Toggle curbuf->b_p_ai (autoindent)
void nvim_excmds_toggle_b_p_ai(void) { curbuf->b_p_ai = !curbuf->b_p_ai; }
// Get curbuf->b_p_iminsert
int nvim_excmds_get_b_p_iminsert(void) { return curbuf->b_p_iminsert; }
// Check if eap->ea_getline is NULL
int nvim_excmds_ea_getline_is_null(exarg_T *eap) { return eap->ea_getline == NULL ? 1 : 0; }
// Get eap->cstack->cs_looplevel
int nvim_excmds_get_cstack_looplevel(exarg_T *eap) { return eap->cstack->cs_looplevel; }
// Call eap->ea_getline(c, eap->cookie, indent, true)
char *nvim_excmds_call_getline(exarg_T *eap, int c, int indent)
{
  return eap->ea_getline(c, eap->cookie, indent, true);
}
// Get eap->nextcmd pointer
char *nvim_excmds_get_nextcmd(exarg_T *eap) { return eap->nextcmd; }
// Set eap->nextcmd directly
void nvim_excmds_set_nextcmd_direct(exarg_T *eap, char *p) { eap->nextcmd = p; }
// Get mutable eap->arg
char *nvim_excmds_get_arg_mut(exarg_T *eap) { return eap->arg; }
// Get eap->cookie
void *nvim_excmds_get_cookie(exarg_T *eap) { return eap->cookie; }

// --- sub_joining_lines + sub_grow_buf FFI accessors ---
// Get eap->skip
int nvim_exarg_get_skip(exarg_T *eap) { return eap->skip; }
// Set eap->flags
void nvim_exarg_set_flags(exarg_T *eap, int flags) { eap->flags = flags; }
// do_join wrapper (count, insert_space=false, save_undo=true, use_fo=false, setmark=true)
int nvim_excmds_do_join(int count)
{
  return do_join((size_t)count, false, true, false, true);
}
// Get/set sub_nsubs global
int nvim_excmds_get_sub_nsubs(void) { return sub_nsubs; }
void nvim_excmds_set_sub_nsubs(int val) { sub_nsubs = val; }
// Get/set sub_nlines global (linenr_T)
int nvim_excmds_get_sub_nlines(void) { return (int)sub_nlines; }
void nvim_excmds_set_sub_nlines(int val) { sub_nlines = (linenr_T)val; }
// nvim_excmds_do_sub_msg is defined below (near do_sub_msg) as the full implementation.
// Call ex_may_print
void nvim_excmds_ex_may_print(exarg_T *eap) { ex_may_print(eap); }
// Call save_re_pat
void nvim_excmds_save_re_pat(int idx, const char *pat, size_t patlen, int magic)
{
  save_re_pat(idx, (char *)pat, patlen, magic);
}
// Call add_to_history(HIST_SEARCH, ...)
void nvim_excmds_add_to_hist_search(const char *pat, size_t patlen)
{
  add_to_history(HIST_SEARCH, (char *)pat, patlen, true, NUL);
}

// --- make_filter_cmd FFI accessors ---
// Get shell name tail (e.g., "bash" from "/bin/bash")
const char *nvim_excmds_shell_name_tail(void) { return invocation_path_tail(p_sh, NULL); }
// Get p_srr (shellredir option)
const char *nvim_excmds_get_p_srr(void) { return p_srr; }
// Get p_shq (shellquote option)
const char *nvim_excmds_get_p_shq(void) { return p_shq; }
// xmalloc wrapper for make_filter_cmd
void *nvim_excmds_xmalloc(size_t size) { return xmalloc(size); }

_Static_assert(ML_DEL_MESSAGE == 1, "ML_DEL_MESSAGE mismatch");

// Verify sort-related constants for Rust
_Static_assert(STR2NR_BIN == (1 << 0), "STR2NR_BIN mismatch");
_Static_assert(STR2NR_OCT == (1 << 1), "STR2NR_OCT mismatch");
_Static_assert(STR2NR_HEX == (1 << 2), "STR2NR_HEX mismatch");
_Static_assert(STR2NR_FORCE == (1 << 7), "STR2NR_FORCE mismatch");
_Static_assert(RE_MAGIC == 1, "RE_MAGIC mismatch");
_Static_assert(kExtmarkNOOP == 0, "kExtmarkNOOP mismatch");
_Static_assert(kExtmarkUndo == 1, "kExtmarkUndo mismatch");

_Static_assert(CMOD_LOCKMARKS == 0x0800, "CMOD_LOCKMARKS mismatch");
_Static_assert(EOL_MAC == 2, "EOL_MAC mismatch");
_Static_assert(ML_EMPTY == 0x01, "ML_EMPTY mismatch");

// Verify constants used in Rust code.
_Static_assert(CMD_left == 229, "CMD_left mismatch");
_Static_assert(CMD_center == 63, "CMD_center mismatch");
_Static_assert(CMD_right == 372, "CMD_right mismatch");
_Static_assert(BL_WHITE == 1, "BL_WHITE mismatch");
_Static_assert(BL_FIX == 4, "BL_FIX mismatch");
_Static_assert(TAB == '\011', "TAB mismatch");
_Static_assert(EXFLAG_LIST == 0x01, "EXFLAG_LIST mismatch");
_Static_assert(EXFLAG_NR == 0x02, "EXFLAG_NR mismatch");
_Static_assert(MODE_INSERT == 0x10, "MODE_INSERT mismatch");
_Static_assert(MODE_LANGMAP == 0x20, "MODE_LANGMAP mismatch");
_Static_assert(MODE_CMDLINE == 0x08, "MODE_CMDLINE mismatch");
_Static_assert(MODE_NORMAL == 0x01, "MODE_NORMAL mismatch");
_Static_assert(B_IMODE_LMAP == 1, "B_IMODE_LMAP mismatch");
_Static_assert(BL_SOL == 2, "BL_SOL mismatch");

// do_bang, prevcmd management implemented in Rust (rs_do_bang, rs_free_prev_shellcmd)
extern void rs_do_bang(int addr_count, exarg_T *eap, bool forceit, bool do_in, bool do_out);
extern void rs_free_prev_shellcmd(void);

/// Handle the ":!cmd" command. Thin wrapper calling the Rust implementation.
void do_bang(int addr_count, exarg_T *eap, bool forceit, bool do_in, bool do_out)
  FUNC_ATTR_NONNULL_ALL
{
  rs_do_bang(addr_count, eap, forceit, do_in, do_out);
}

#if defined(EXITFREE)
void free_prev_shellcmd(void)
{
  rs_free_prev_shellcmd();
}

#endif

// do_filter implemented in Rust (rs_do_filter in ex_cmds/src/shell.rs)
extern void rs_do_filter(int line1, int line2, exarg_T *eap, const char *cmd, int do_in,
                         int do_out);

/// Filter lines through an external command. Thin wrapper calling the Rust implementation.
static void do_filter(linenr_T line1, linenr_T line2, exarg_T *eap, char *cmd, bool do_in,
                      bool do_out)
{
  rs_do_filter((int)line1, (int)line2, eap, cmd, do_in ? 1 : 0, do_out ? 1 : 0);
}

// do_shell implemented in Rust (rs_do_shell in ex_cmds/src/shell.rs)
extern void rs_do_shell(char *cmd, int flags);

/// Call a shell to execute a command. Thin wrapper calling the Rust implementation.
void do_shell(char *cmd, int flags)
{
  rs_do_shell(cmd, flags);
}

// make_filter_cmd and find_pipe implemented in Rust (rs_make_filter_cmd in ex_cmds/src/shell.rs)
extern char *rs_make_filter_cmd(const char *cmd, const char *itmp, const char *otmp, int do_in);

/// Create a shell command from a command string, input redirection file and
/// output redirection file. Thin wrapper calling the Rust implementation.
char *make_filter_cmd(char *cmd, char *itmp, char *otmp, bool do_in)
{
  return rs_make_filter_cmd(cmd, itmp, otmp, (int)do_in);
}

// append_redir implemented in Rust (rs_append_redir in ex_cmds/src/shell.rs)
extern void rs_append_redir(char *buf, size_t buflen, const char *opt, const char *fname);

/// Append output redirection for the given file to the end of the buffer.
/// Thin wrapper calling the Rust implementation.
void append_redir(char *const buf, const size_t buflen, const char *const opt,
                  const char *const fname)
{
  rs_append_redir(buf, buflen, opt, fname);
}

// rename_buffer + ex_file implemented in Rust (rs_rename_buffer, rs_ex_file in ex_cmds/src/buffer.rs)
extern int rs_rename_buffer(const char *new_fname);
extern void rs_ex_file(exarg_T *eap);

/// Rename the current buffer to a new file name. Thin wrapper calling Rust.
int rename_buffer(char *new_fname)
{
  return rs_rename_buffer(new_fname) ? OK : FAIL;
}

/// ":file[!] [fname]". Thin wrapper calling Rust.
void ex_file(exarg_T *eap)
{
  rs_ex_file(eap);
}

// ex_update, ex_write, ex_wnext implemented in Rust (ex_cmds/src/write.rs)
extern void rs_ex_update(exarg_T *eap);
extern void rs_ex_write(exarg_T *eap);
extern void rs_ex_wnext(exarg_T *eap);
extern int rs_not_writing(void);
extern int rs_check_writable(const char *fname);
extern int rs_handle_mkdir_p_arg(exarg_T *eap, const char *fname);
extern int rs_check_readonly(exarg_T *eap, buf_T *buf);
extern int rs_do_write(exarg_T *eap);
extern int rs_check_overwrite(exarg_T *eap, buf_T *buf, const char *fname, const char *ffname, int other);
extern void rs_do_wqall(exarg_T *eap);
extern int rs_getfile(int fnum, char *ffname, char *sfname, int setpm, int lnum, int forceit);
extern int rs_set_swapcommand(const char *command, int newlnum);
extern void rs_delbuf_msg(char *name);

/// ":update". Thin wrapper calling Rust.
void ex_update(exarg_T *eap)
{
  rs_ex_update(eap);
}

/// ":write" and ":saveas". Thin wrapper calling Rust.
void ex_write(exarg_T *eap)
{
  rs_ex_write(eap);
}

/// Thin wrapper calling Rust rs_check_writable.
static int check_writable(const char *fname)
{
  return rs_check_writable(fname) == 1 ? OK : FAIL;
}

/// Thin wrapper calling Rust rs_handle_mkdir_p_arg.
static int handle_mkdir_p_arg(exarg_T *eap, char *fname)
{
  return rs_handle_mkdir_p_arg(eap, fname) == 1 ? OK : FAIL;
}

/// Thin wrapper calling Rust rs_do_write.
///
/// Write current buffer to file "eap->arg".
/// If "eap->append" is true, append to the file.
///
/// @return  FAIL for failure, OK otherwise.
int do_write(exarg_T *eap)
{
  return rs_do_write(eap) != 0 ? OK : FAIL;
}

/// Check if it is allowed to overwrite a file.  If b_flags has BF_NOTEDITED,
/// BF_NEW or BF_READERR, check for overwriting current file.
/// Thin wrapper calling Rust rs_check_overwrite.
///
/// @param fname   file name to be used (can differ from buf->ffname)
/// @param ffname  full path version of fname
/// @param other   writing under other name
///
/// @return  OK if it's OK, FAIL if it is not.
int check_overwrite(exarg_T *eap, buf_T *buf, char *fname, char *ffname, bool other)
{
  return rs_check_overwrite(eap, buf, fname, ffname, (int)other) != 0 ? OK : FAIL;
}

/// Handle ":wnext", ":wNext" and ":wprevious" commands. Thin wrapper calling Rust.
void ex_wnext(exarg_T *eap)
{
  rs_ex_wnext(eap);
}

/// Thin wrapper calling Rust rs_do_wqall.
///
/// ":wall", ":wqall" and ":xall": Write all changed files (and exit).
void do_wqall(exarg_T *eap)
{
  rs_do_wqall(eap);
}

/// Thin wrapper calling Rust rs_not_writing.
///
/// @return  true and give a message when writing is disabled.
static bool not_writing(void)
{
  return rs_not_writing() != 0;
}

/// Thin wrapper calling Rust rs_check_readonly.
/// Note: rs_check_readonly reads/sets forceit via the eap pointer.
/// This wrapper reconstructs an eap with the passed forceit for compatibility.
static int check_readonly(int *forceit, buf_T *buf)
{
  // We use a local exarg_T to pass forceit through the rs_ interface
  exarg_T fake_eap = { 0 };
  fake_eap.forceit = (bool)*forceit;
  int result = rs_check_readonly(&fake_eap, buf);
  *forceit = (int)fake_eap.forceit;
  return result;
}

/// Thin wrapper calling Rust rs_getfile.
///
/// @param fnum  the number of the file, if zero use "ffname_arg"/"sfname_arg".
/// @param lnum  the line number for the cursor in the new file (if non-zero).
///
/// @return:
///           GETFILE_ERROR for "normal" error,
///           GETFILE_NOT_WRITTEN for "not written" error,
///           GETFILE_SAME_FILE for success
///           GETFILE_OPEN_OTHER for successfully opening another file.
int getfile(int fnum, char *ffname_arg, char *sfname_arg, bool setpm, linenr_T lnum, bool forceit)
{
  return rs_getfile(fnum, ffname_arg, sfname_arg, (int)setpm, (int)lnum, (int)forceit);
}

/// Thin wrapper calling Rust rs_set_swapcommand.
///
/// @param command  [+cmd] to be executed (e.g. +10).
/// @param newlnum  if > 0: put cursor on this line number (if possible)
//
/// @return 1 if swapcommand was actually set, 0 otherwise
bool set_swapcommand(char *command, linenr_T newlnum)
{
  return rs_set_swapcommand(command, (int)newlnum) != 0;
}

/// start editing a new file
///
/// @param fnum     file number; if zero use ffname/sfname
/// @param ffname   the file name
///                 - full path if sfname used,
///                 - any file name if sfname is NULL
///                 - empty string to re-edit with the same file name (but may
///                   be in a different directory)
///                 - NULL to start an empty buffer
/// @param sfname   the short file name (or NULL)
/// @param eap      contains the command to be executed after loading the file
///                 and forced 'ff' and 'fenc'. Can be NULL!
/// @param newlnum  if > 0: put cursor on this line number (if possible)
///                 ECMD_LASTL: use last position in loaded file
///                 ECMD_LAST: use last position in all files
///                 ECMD_ONE: use first line
/// @param flags    ECMD_HIDE: if true don't free the current buffer
///                 ECMD_SET_HELP: set b_help flag of (new) buffer before
///                 opening file
///                 ECMD_OLDBUF: use existing buffer if it exists
///                 ECMD_FORCEIT: ! used for Ex command
///                 ECMD_ADDBUF: don't edit, just add to buffer list
///                 ECMD_ALTBUF: like ECMD_ADDBUF and also set the alternate
///                 file
///                 ECMD_NOWINENTER: Do not trigger BufWinEnter
/// @param oldwin   Should be "curwin" when editing a new buffer in the current
///                 window, NULL when splitting the window first.  When not NULL
///                 info of the previous buffer for "oldwin" is stored.
///
/// @return FAIL for failure, OK otherwise
int do_ecmd(int fnum, char *ffname, char *sfname, exarg_T *eap, linenr_T newlnum, int flags,
            win_T *oldwin)
{
  bool other_file;                      // true if editing another file
  int oldbuf;                           // true if using existing buffer
  bool auto_buf = false;                // true if autocommands brought us
                                        // into the buffer unexpectedly
  char *new_name = NULL;
  bool did_set_swapcommand = false;
  buf_T *buf;
  bufref_T bufref;
  bufref_T old_curbuf;
  char *free_fname = NULL;
  int retval = FAIL;
  linenr_T topline = 0;
  int newcol = -1;
  int solcol = -1;
  char *command = NULL;
  bool did_get_winopts = false;
  int readfile_flags = 0;
  bool did_inc_redrawing_disabled = false;
  OptInt *so_ptr = curwin->w_p_so >= 0 ? &curwin->w_p_so : &p_so;

  if (eap != NULL) {
    command = eap->do_ecmd_cmd;
  }

  set_bufref(&old_curbuf, curbuf);

  if (fnum != 0) {
    if (fnum == curbuf->b_fnum) {       // file is already being edited
      return OK;                        // nothing to do
    }
    other_file = true;
  } else {
    // if no short name given, use ffname for short name
    if (sfname == NULL) {
      sfname = ffname;
    }
#ifdef CASE_INSENSITIVE_FILENAME
    if (sfname != NULL) {
      path_fix_case(sfname);             // set correct case for sfname
    }
#endif

    if ((flags & (ECMD_ADDBUF | ECMD_ALTBUF))
        && (ffname == NULL || *ffname == NUL)) {
      goto theend;
    }

    if (ffname == NULL) {
      other_file = true;
    } else if (*ffname == NUL && curbuf->b_ffname == NULL) {  // there is no file name
      other_file = false;
    } else {
      if (*ffname == NUL) {                 // re-edit with same file name
        ffname = curbuf->b_ffname;
        sfname = curbuf->b_fname;
      }
      free_fname = fix_fname(ffname);       // may expand to full path name
      if (free_fname != NULL) {
        ffname = free_fname;
      }
      other_file = otherfile(ffname);
    }
  }

  // Re-editing a terminal buffer: skip most buffer re-initialization.
  if (!other_file && curbuf->terminal) {
    check_arg_idx(curwin);  // Needed when called from do_argfile().
    maketitle();            // Title may show the arg index, e.g. "(2 of 5)".
    retval = OK;
    goto theend;
  }

  // If the file was changed we may not be allowed to abandon it:
  // - if we are going to re-edit the same file
  // - or if we are the only window on this file and if ECMD_HIDE is false
  if (((!other_file && !(flags & ECMD_OLDBUF))
       || (curbuf->b_nwindows == 1
           && !(flags & (ECMD_HIDE | ECMD_ADDBUF | ECMD_ALTBUF))))
      && check_changed(curbuf, (p_awa ? CCGD_AW : 0)
                       | (other_file ? 0 : CCGD_MULTWIN)
                       | ((flags & ECMD_FORCEIT) ? CCGD_FORCEIT : 0)
                       | (eap == NULL ? 0 : CCGD_EXCMD))) {
    if (fnum == 0 && other_file && ffname != NULL) {
      setaltfname(ffname, sfname, newlnum < 0 ? 0 : newlnum);
    }
    goto theend;
  }

  // End Visual mode before switching to another buffer, so the text can be
  // copied into the GUI selection buffer.
  // Careful: may trigger ModeChanged() autocommand

  // Should we block autocommands here?
  rs_reset_VIsual();

  // autocommands freed window :(
  if (oldwin != NULL && !rs_win_valid(oldwin)) {
    oldwin = NULL;
  }

  did_set_swapcommand = set_swapcommand(command, newlnum);

  // If we are starting to edit another file, open a (new) buffer.
  // Otherwise we re-use the current buffer.
  if (other_file) {
    const int prev_alt_fnum = curwin->w_alt_fnum;

    if (!(flags & (ECMD_ADDBUF | ECMD_ALTBUF))) {
      if ((cmdmod.cmod_flags & CMOD_KEEPALT) == 0) {
        curwin->w_alt_fnum = curbuf->b_fnum;
      }
      if (oldwin != NULL) {
        buflist_altfpos(oldwin);
      }
    }

    if (fnum) {
      buf = buflist_findnr(fnum);
    } else {
      if (flags & (ECMD_ADDBUF | ECMD_ALTBUF)) {
        // Default the line number to zero to avoid that a wininfo item
        // is added for the current window.
        linenr_T tlnum = 0;

        if (command != NULL) {
          tlnum = (linenr_T)atol(command);
          if (tlnum <= 0) {
            tlnum = 1;
          }
        }
        // Add BLN_NOCURWIN to avoid a new wininfo items are associated
        // with the current window.
        const buf_T *const newbuf
          = buflist_new(ffname, sfname, tlnum, BLN_LISTED | BLN_NOCURWIN);
        if (newbuf != NULL && (flags & ECMD_ALTBUF)) {
          curwin->w_alt_fnum = newbuf->b_fnum;
        }
        goto theend;
      }
      buf = buflist_new(ffname, sfname, 0,
                        BLN_CURBUF | (flags & ECMD_SET_HELP ? 0 : BLN_LISTED));
      // Autocmds may change curwin and curbuf.
      if (oldwin != NULL) {
        oldwin = curwin;
      }
      set_bufref(&old_curbuf, curbuf);
    }
    if (buf == NULL) {
      goto theend;
    }
    // autocommands try to edit a closing buffer, which like splitting, can
    // result in more windows displaying it; abort
    if (buf->b_locked_split) {
      // window was split, but not editing the new buffer, reset b_nwindows again
      if (oldwin == NULL
          && curwin->w_buffer != NULL
          && curwin->w_buffer->b_nwindows > 1) {
        curwin->w_buffer->b_nwindows--;
      }
      emsg(_(e_cannot_switch_to_a_closing_buffer));
      goto theend;
    }
    if (curwin->w_alt_fnum == buf->b_fnum && prev_alt_fnum != 0) {
      // reusing the buffer, keep the old alternate file
      curwin->w_alt_fnum = prev_alt_fnum;
    }
    if (buf->b_ml.ml_mfp == NULL) {
      // No memfile yet.
      oldbuf = false;
    } else {
      // Existing memfile.
      oldbuf = true;
      set_bufref(&bufref, buf);
      buf_check_timestamp(buf);
      // Check if autocommands made buffer invalid or changed the current
      // buffer.
      if (!bufref_valid(&bufref) || curbuf != old_curbuf.br_buf) {
        goto theend;
      }
      if (aborting()) {
        // Autocmds may abort script processing.
        goto theend;
      }
    }

    // May jump to last used line number for a loaded buffer or when asked
    // for explicitly
    if ((oldbuf && newlnum == ECMD_LASTL) || newlnum == ECMD_LAST) {
      pos_T *pos = &buflist_findfmark(buf)->mark;
      newlnum = pos->lnum;
      solcol = pos->col;
    }

    // Make the (new) buffer the one used by the current window.
    // If the old buffer becomes unused, free it if ECMD_HIDE is false.
    // If the current buffer was empty and has no file name, curbuf
    // is returned by buflist_new(), nothing to do here.
    if (buf != curbuf) {
      // Should only be possible to get here if the cmdwin is closed, or
      // if it's opening and its buffer hasn't been set yet (the new
      // buffer is for it).
      assert(cmdwin_buf == NULL);

      const int save_cmdwin_type = cmdwin_type;
      win_T *const save_cmdwin_win = cmdwin_win;
      win_T *const save_cmdwin_old_curwin = cmdwin_old_curwin;

      // BufLeave applies to the old buffer.
      cmdwin_type = 0;
      cmdwin_win = NULL;
      cmdwin_old_curwin = NULL;

      // Be careful: The autocommands may delete any buffer and change
      // the current buffer.
      // - If the buffer we are going to edit is deleted, give up.
      // - If the current buffer is deleted, prefer to load the new
      //   buffer when loading a buffer is required.  This avoids
      //   loading another buffer which then must be closed again.
      // - If we ended up in the new buffer already, need to skip a few
      //         things, set auto_buf.
      if (buf->b_fname != NULL) {
        new_name = xstrdup(buf->b_fname);
      }
      const bufref_T save_au_new_curbuf = au_new_curbuf;
      set_bufref(&au_new_curbuf, buf);
      apply_autocmds(EVENT_BUFLEAVE, NULL, NULL, false, curbuf);

      cmdwin_type = save_cmdwin_type;
      cmdwin_win = save_cmdwin_win;
      cmdwin_old_curwin = save_cmdwin_old_curwin;

      if (!bufref_valid(&au_new_curbuf)) {
        // New buffer has been deleted.
        delbuf_msg(new_name);  // Frees new_name.
        au_new_curbuf = save_au_new_curbuf;
        goto theend;
      }
      if (aborting()) {             // autocmds may abort script processing
        xfree(new_name);
        au_new_curbuf = save_au_new_curbuf;
        goto theend;
      }
      if (buf == curbuf) {  // already in new buffer
        auto_buf = true;
      } else {
        win_T *the_curwin = curwin;
        buf_T *was_curbuf = curbuf;

        // Set w_locked to avoid that autocommands close the window.
        // Set b_locked for the same reason.
        the_curwin->w_locked = true;
        buf->b_locked++;

        if (curbuf == old_curbuf.br_buf) {
          buf_copy_options(buf, BCO_ENTER);
        }

        // Close the link to the current buffer. This will set
        // oldwin->w_buffer to NULL.
        u_sync(false);
        const bool did_decrement
          = close_buffer(oldwin, curbuf, (flags & ECMD_HIDE) || curbuf->terminal ? 0 : DOBUF_UNLOAD,
                         false, false);

        // Autocommands may have closed the window.
        if (rs_win_valid(the_curwin)) {
          the_curwin->w_locked = false;
        }
        buf->b_locked--;

        // autocmds may abort script processing
        if (aborting() && curwin->w_buffer != NULL) {
          xfree(new_name);
          au_new_curbuf = save_au_new_curbuf;
          goto theend;
        }
        // Be careful again, like above.
        if (!bufref_valid(&au_new_curbuf)) {
          // New buffer has been deleted.
          delbuf_msg(new_name);  // Frees new_name.
          au_new_curbuf = save_au_new_curbuf;
          goto theend;
        }
        if (buf == curbuf) {  // already in new buffer
          // close_buffer() has decremented the window count,
          // increment it again here and restore w_buffer.
          if (did_decrement && buf_valid(was_curbuf)) {
            was_curbuf->b_nwindows++;
          }
          if (rs_win_valid_any_tab(oldwin) && oldwin->w_buffer == NULL) {
            oldwin->w_buffer = was_curbuf;
          }
          auto_buf = true;
        } else {
          // <VN> We could instead free the synblock
          // and re-attach to buffer, perhaps.
          if (curwin->w_buffer == NULL
              || curwin->w_s == &(curwin->w_buffer->b_s)) {
            curwin->w_s = &(buf->b_s);
          }

          curwin->w_buffer = buf;
          curbuf = buf;
          curbuf->b_nwindows++;

          // Set 'fileformat', 'binary' and 'fenc' when forced.
          if (!oldbuf && eap != NULL) {
            set_file_options(true, eap);
            set_forced_fenc(eap);
          }
        }

        // May get the window options from the last time this buffer
        // was in this window (or another window).  If not used
        // before, reset the local window options to the global
        // values.  Also restores old folding stuff.
        get_winopts(curbuf);
        did_get_winopts = true;
      }
      xfree(new_name);
      au_new_curbuf = save_au_new_curbuf;
    }

    curwin->w_pcmark.lnum = 1;
    curwin->w_pcmark.col = 0;
  } else {  // !other_file
    if ((flags & (ECMD_ADDBUF | ECMD_ALTBUF)) || check_fname() == FAIL) {
      goto theend;
    }
    oldbuf = (flags & ECMD_OLDBUF);
  }

  // Don't redraw until the cursor is in the right line, otherwise
  // autocommands may cause ml_get errors.
  RedrawingDisabled++;
  did_inc_redrawing_disabled = true;

  buf = curbuf;
  if ((flags & ECMD_SET_HELP) || keep_help_flag) {
    prepare_help_buffer();
  } else if (!curbuf->b_help) {
    // Don't make a buffer listed if it's a help buffer.  Useful when using
    // CTRL-O to go back to a help file.
    set_buflisted(true);
  }

  // If autocommands change buffers under our fingers, forget about
  // editing the file.
  if (buf != curbuf) {
    goto theend;
  }
  if (aborting()) {         // autocmds may abort script processing
    goto theend;
  }

  // Since we are starting to edit a file, consider the filetype to be
  // unset.  Helps for when an autocommand changes files and expects syntax
  // highlighting to work in the other file.
  curbuf->b_did_filetype = false;

  // other_file oldbuf
  //  false     false       re-edit same file, buffer is re-used
  //  false     true        re-edit same file, nothing changes
  //  true      false       start editing new file, new buffer
  //  true      true        start editing in existing buffer (nothing to do)
  if (!other_file && !oldbuf) {         // re-use the buffer
    set_last_cursor(curwin);            // may set b_last_cursor
    if (newlnum == ECMD_LAST || newlnum == ECMD_LASTL) {
      newlnum = curwin->w_cursor.lnum;
      solcol = curwin->w_cursor.col;
    }
    buf = curbuf;
    if (buf->b_fname != NULL) {
      new_name = xstrdup(buf->b_fname);
    } else {
      new_name = NULL;
    }
    set_bufref(&bufref, buf);

    // If the buffer was used before, store the current contents so that
    // the reload can be undone.  Do not do this if the (empty) buffer is
    // being re-used for another file.
    if (!(curbuf->b_flags & BF_NEVERLOADED)
        && (p_ur < 0 || curbuf->b_ml.ml_line_count <= p_ur)) {
      // Sync first so that this is a separate undo-able action.
      u_sync(false);
      if (u_savecommon(curbuf, 0, curbuf->b_ml.ml_line_count + 1, 0, true)
          == FAIL) {
        xfree(new_name);
        goto theend;
      }
      u_unchanged(curbuf);
      buf_freeall(curbuf, BFA_KEEP_UNDO);

      // Tell readfile() not to clear or reload undo info.
      readfile_flags = READ_KEEP_UNDO;
    } else {
      buf_freeall(curbuf, 0);  // Free all things for buffer.
    }
    // If autocommands deleted the buffer we were going to re-edit, give
    // up and jump to the end.
    if (!bufref_valid(&bufref)) {
      delbuf_msg(new_name);  // Frees new_name.
      goto theend;
    }
    xfree(new_name);

    // If autocommands change buffers under our fingers, forget about
    // re-editing the file.  Should do the buf_clear_file(), but perhaps
    // the autocommands changed the buffer...
    if (buf != curbuf) {
      goto theend;
    }
    if (aborting()) {       // autocmds may abort script processing
      goto theend;
    }
    buf_clear_file(curbuf);
    curbuf->b_op_start.lnum = 0;        // clear '[ and '] marks
    curbuf->b_op_end.lnum = 0;
  }

  // If we get here we are sure to start editing

  // Assume success now
  retval = OK;

  // If the file name was changed, reset the not-edit flag so that ":write"
  // works.
  if (!other_file) {
    curbuf->b_flags &= ~BF_NOTEDITED;
  }

  // Check if we are editing the w_arg_idx file in the argument list.
  check_arg_idx(curwin);

  if (!auto_buf) {
    // Set cursor and init window before reading the file and executing
    // autocommands.  This allows for the autocommands to position the
    // cursor.
    curwin_init();

    // It's possible that all lines in the buffer changed.  Need to update
    // automatic folding for all windows where it's used.
    FOR_ALL_TAB_WINDOWS(tp, win) {
      if (win->w_buffer == curbuf) {
        rs_foldUpdateAll(win);
      }
    }

    // Change directories when the 'acd' option is set.
    do_autochdir();

    // Careful: open_buffer() and apply_autocmds() may change the current
    // buffer and window.
    pos_T orig_pos = curwin->w_cursor;
    topline = curwin->w_topline;
    if (!oldbuf) {                          // need to read the file
      swap_exists_action = SEA_DIALOG;
      curbuf->b_flags |= BF_CHECK_RO;       // set/reset 'ro' flag

      // Open the buffer and read the file.
      if (flags & ECMD_NOWINENTER) {
        readfile_flags |= READ_NOWINENTER;
      }
      if (should_abort(open_buffer(false, eap, readfile_flags))) {
        retval = FAIL;
      }

      if (swap_exists_action == SEA_QUIT) {
        retval = FAIL;
      }
      handle_swap_exists(&old_curbuf);
    } else {
      // Read the modelines, but only to set window-local options.  Any
      // buffer-local options have already been set and may have been
      // changed by the user.
      do_modelines(OPT_WINONLY);

      apply_autocmds_retval(EVENT_BUFENTER, NULL, NULL, false, curbuf,
                            &retval);
      if ((flags & ECMD_NOWINENTER) == 0) {
        apply_autocmds_retval(EVENT_BUFWINENTER, NULL, NULL, false, curbuf,
                              &retval);
      }
    }
    check_arg_idx(curwin);

    // If autocommands change the cursor position or topline, we should
    // keep it.  Also when it moves within a line. But not when it moves
    // to the first non-blank.
    if (!equalpos(curwin->w_cursor, orig_pos)) {
      const char *text = get_cursor_line_ptr();

      if (curwin->w_cursor.lnum != orig_pos.lnum
          || curwin->w_cursor.col != (int)(skipwhite(text) - text)) {
        newlnum = curwin->w_cursor.lnum;
        newcol = curwin->w_cursor.col;
      }
    }
    if (curwin->w_topline == topline) {
      topline = 0;
    }

    // Even when cursor didn't move we need to recompute topline.
    changed_line_abv_curs();

    maketitle();
  }

  // Tell the diff stuff that this buffer is new and/or needs updating.
  // Also needed when re-editing the same buffer, because unloading will
  // have removed it as a diff buffer.
  if (curwin->w_p_diff) {
    rs_diff_buf_add(curbuf);
    rs_diff_invalidate(curbuf);
  }

  // If the window options were changed may need to set the spell language.
  // Can only do this after the buffer has been properly setup.
  if (did_get_winopts && curwin->w_p_spell && *curwin->w_s->b_p_spl != NUL) {
    parse_spelllang(curwin);
  }

  if (command == NULL) {
    if (newcol >= 0) {          // position set by autocommands
      curwin->w_cursor.lnum = newlnum;
      curwin->w_cursor.col = newcol;
      check_cursor(curwin);
    } else if (newlnum > 0) {  // line number from caller or old position
      curwin->w_cursor.lnum = newlnum;
      check_cursor_lnum(curwin);
      if (solcol >= 0 && !p_sol) {
        // 'sol' is off: Use last known column.
        curwin->w_cursor.col = solcol;
        check_cursor_col(curwin);
        curwin->w_cursor.coladd = 0;
        curwin->w_set_curswant = true;
      } else {
        beginline(BL_SOL | BL_FIX);
      }
    } else {                  // no line number, go to last line in Ex mode
      if (exmode_active) {
        curwin->w_cursor.lnum = curbuf->b_ml.ml_line_count;
      }
      beginline(BL_WHITE | BL_FIX);
    }
  }

  // Check if cursors in other windows on the same buffer are still valid
  rs_check_lnums(0);

  // Did not read the file, need to show some info about the file.
  // Do this after setting the cursor.
  if (oldbuf
      && !auto_buf) {
    int msg_scroll_save = msg_scroll;

    // Obey the 'O' flag in 'cpoptions': overwrite any previous file
    // message.
    if (shortmess(SHM_OVERALL) && !msg_listdo_overwrite && !exiting && p_verbose == 0) {
      msg_scroll = false;
    }
    if (!msg_scroll) {          // wait a bit when overwriting an error msg
      msg_check_for_delay(false);
    }
    msg_start();
    msg_scroll = msg_scroll_save;
    msg_scrolled_ign = true;

    if (!shortmess(SHM_FILEINFO)) {
      fileinfo(false, true, false);
    }

    msg_scrolled_ign = false;
  }

  curbuf->b_last_used = time(NULL);

  if (command != NULL) {
    do_cmdline(command, NULL, NULL, DOCMD_VERBOSE);
  }

  if (curbuf->b_kmap_state & KEYMAP_INIT) {
    keymap_init();
  }

  RedrawingDisabled--;
  did_inc_redrawing_disabled = false;
  if (!skip_redraw) {
    OptInt n = *so_ptr;
    if (topline == 0 && command == NULL) {
      *so_ptr = 999;    // force cursor to be vertically centered in the window
    }
    update_topline(curwin);
    curwin->w_scbind_pos = plines_m_win_fill(curwin, 1, curwin->w_topline);
    *so_ptr = n;
    redraw_curbuf_later(UPD_NOT_VALID);  // redraw this buffer later
  }

  // Change directories when the 'acd' option is set.
  do_autochdir();

theend:
  if (bufref_valid(&old_curbuf) && old_curbuf.br_buf->terminal != NULL) {
    terminal_check_size(old_curbuf.br_buf->terminal);
  }
  if ((!bufref_valid(&old_curbuf) || curbuf != old_curbuf.br_buf) && curbuf->terminal != NULL) {
    terminal_check_size(curbuf->terminal);
  }

  if (did_inc_redrawing_disabled) {
    RedrawingDisabled--;
  }
  if (did_set_swapcommand) {
    set_vim_var_string(VV_SWAPCOMMAND, NULL, -1);
  }
  xfree(free_fname);
  return retval;
}

/// Thin wrapper calling Rust rs_delbuf_msg.
static void delbuf_msg(char *name)
{
  rs_delbuf_msg(name);
}

static int append_indent = 0;       // autoindent for first line

/// Set append_indent (used by ex_change before calling ex_append).
void nvim_set_append_indent(int val)
{
  append_indent = val;
}

/// Get append_indent value
int nvim_excmds_get_append_indent(void) { return append_indent; }

// ex_append has been migrated to Rust (rs_ex_append in lines.rs)

/// Previous substitute replacement string
static SubReplacementString old_sub = { NULL, 0, NULL };

static int global_need_beginline;       // call beginline() after ":g"

// sub_get_replacement, sub_set_replacement, free_old_sub, ex_substitute, ex_substitute_preview
// implemented in Rust (ex_cmds/src/substitute.rs).
extern void rs_sub_get_replacement(void *ret_sub);
extern void rs_sub_set_replacement(char *sub, uint64_t timestamp, void *additional_data);
extern void rs_free_old_sub(void);
extern void rs_ex_substitute(exarg_T *eap);
extern int rs_ex_substitute_preview(exarg_T *eap, int cmdpreview_ns, handle_T cmdpreview_bufnr);

/// Get old substitute replacement string. Thin wrapper calling Rust.
///
/// @param[out]  ret_sub    Location where old string will be saved.
void sub_get_replacement(SubReplacementString *const ret_sub)
  FUNC_ATTR_NONNULL_ALL
{
  rs_sub_get_replacement((void *)ret_sub);
}

/// Set substitute string and timestamp. Thin wrapper calling Rust.
///
/// @warning `sub` must be in allocated memory. It is not copied.
///
/// @param[in]  sub  New replacement string.
void sub_set_replacement(SubReplacementString sub)
{
  rs_sub_set_replacement(sub.sub, (uint64_t)sub.timestamp, (void *)sub.additional_data);
}

// sub_joining_lines, sub_grow_buf, sub_parse_flags implemented in Rust.
// See the corresponding rs_* functions in ex_cmds/src/substitute.rs.
extern int rs_sub_joining_lines(exarg_T *eap, const char *pat, size_t patlen, const char *sub,
                                const char *cmd, int save, int keeppatterns);
extern char *rs_sub_grow_buf(char **new_start, int *new_start_len, int needed_len);
extern char *rs_sub_parse_flags(char *cmd, subflags_T *subflags, int *which_pat);

/// Perform a substitution from line eap->line1 to line eap->line2 using the
/// command pointed to by eap->arg which should be of the form:
///
/// /pattern/substitution/{flags}
///
/// The usual escapes are supported as described in the regexp docs.
///
/// @param cmdpreview_ns  The namespace to show 'inccommand' preview highlights.
///                       If <= 0, preview shouldn't be shown.
/// @return  0, 1 or 2. See cmdpreview_may_show() for more information on the meaning.
static int do_sub(exarg_T *eap, const proftime_T timeout, const int cmdpreview_ns,
                  const handle_T cmdpreview_bufnr)
{
#define ADJUST_SUB_FIRSTLNUM() \
  do { \
    /* For a multi-line match, make a copy of the last matched */ \
    /* line and continue in that one. */ \
    if (nmatch > 1) { \
      sub_firstlnum += (linenr_T)nmatch - 1; \
      xfree(sub_firstline); \
      sub_firstline = xstrnsave(ml_get(sub_firstlnum), \
                                (size_t)ml_get_len(sub_firstlnum)); \
      /* When going beyond the last line, stop substituting. */ \
      if (sub_firstlnum <= line2) { \
        do_again = true; \
      } else { \
        subflags.do_all = false; \
      } \
    } \
    if (skip_match) { \
      /* Already hit end of the buffer, sub_firstlnum is one */ \
      /* less than what it ought to be. */ \
      xfree(sub_firstline); \
      sub_firstline = xstrdup(""); \
      copycol = 0; \
    } \
  } while (0)

  int i = 0;
  regmmatch_T regmatch;
  static subflags_T subflags = {
    .do_all = false,
    .do_ask = false,
    .do_count = false,
    .do_error = true,
    .do_print = false,
    .do_list = false,
    .do_number = false,
    .do_ic = kSubHonorOptions
  };
  char *pat = NULL;
  char *sub = NULL;  // init for GCC
  size_t patlen = 0;
  int delimiter;
  bool has_second_delim = false;
  int sublen;
  bool got_quit = false;
  bool got_match = false;
  int which_pat;
  char *cmd = eap->arg;
  linenr_T first_line = 0;  // first changed line
  linenr_T last_line = 0;    // below last changed line AFTER the change
  linenr_T old_line_count = curbuf->b_ml.ml_line_count;
  char *sub_firstline;    // allocated copy of first sub line
  bool endcolumn = false;   // cursor in last column when done
  const bool keeppatterns = cmdmod.cmod_flags & CMOD_KEEPPATTERNS;
  PreviewLines preview_lines = { KV_INITIAL_VALUE, 0 };
  static int pre_hl_id = 0;
  pos_T old_cursor = curwin->w_cursor;
  int start_nsubs;

  bool did_save = false;

  if (!global_busy) {
    sub_nsubs = 0;
    sub_nlines = 0;
  }
  start_nsubs = sub_nsubs;

  if (eap->cmdidx == CMD_tilde) {
    which_pat = RE_LAST;        // use last used regexp
  } else {
    which_pat = RE_SUBST;       // use last substitute regexp
  }
  // new pattern and substitution
  if (eap->cmd[0] == 's' && *cmd != NUL && !ascii_iswhite(*cmd)
      && vim_strchr("0123456789cegriIp|\"", (uint8_t)(*cmd)) == NULL) {
    // don't accept alphanumeric for separator
    if (rs_check_regexp_delim(*cmd) == FAIL) {
      return 0;
    }

    // undocumented vi feature:
    //  "\/sub/" and "\?sub?" use last used search pattern (almost like
    //  //sub/r).  "\&sub&" use last substitute pattern (like //sub/).
    if (*cmd == '\\') {
      cmd++;
      if (vim_strchr("/?&", (uint8_t)(*cmd)) == NULL) {
        emsg(_(e_backslash));
        return 0;
      }
      if (*cmd != '&') {
        which_pat = RE_SEARCH;              // use last '/' pattern
      }
      pat = "";                   // empty search pattern
      patlen = 0;
      delimiter = (uint8_t)(*cmd++);                   // remember delimiter character
      has_second_delim = true;
    } else {          // find the end of the regexp
      which_pat = RE_LAST;                  // use last used regexp
      delimiter = (uint8_t)(*cmd++);                   // remember delimiter character
      pat = cmd;                            // remember start of search pat
      cmd = skip_regexp_ex(cmd, delimiter, rs_magic_isset(), &eap->arg, NULL, NULL);
      if (cmd[0] == delimiter) {            // end delimiter found
        *cmd++ = NUL;                       // replace it with a NUL
        has_second_delim = true;
      }
      patlen = strlen(pat);
    }

    // Small incompatibility: vi sees '\n' as end of the command, but in
    // Vim we want to use '\n' to find/substitute a NUL.
    char *p = cmd;  // remember the start of the substitution
    cmd = rs_skip_substitute(cmd, delimiter);
    sub = xstrdup(p);

    if (!eap->skip && !keeppatterns && cmdpreview_ns <= 0) {
      sub_set_replacement((SubReplacementString) {
        .sub = xstrdup(sub),
        .timestamp = os_time(),
        .additional_data = NULL,
      });
    }
  } else if (!eap->skip) {    // use previous pattern and substitution
    if (old_sub.sub == NULL) {      // there is no previous command
      emsg(_(e_nopresub));
      return 0;
    }
    pat = NULL;                 // search_regcomp() will use previous pattern
    patlen = 0;
    sub = xstrdup(old_sub.sub);

    // Vi compatibility quirk: repeating with ":s" keeps the cursor in the
    // last column after using "$".
    endcolumn = (curwin->w_curswant == MAXCOL);
  }

  if (sub != NULL && rs_sub_joining_lines(eap, pat, patlen, sub, cmd, cmdpreview_ns <= 0 ? 1 : 0,
                                          keeppatterns ? 1 : 0) != 0) {
    xfree(sub);
    return 0;
  }

  cmd = rs_sub_parse_flags(cmd, &subflags, &which_pat);

  bool save_do_all = subflags.do_all;  // remember user specified 'g' flag
  bool save_do_ask = subflags.do_ask;  // remember user specified 'c' flag

  // check for a trailing count
  cmd = skipwhite(cmd);
  if (ascii_isdigit(*cmd)) {
    i = getdigits_int(&cmd, true, INT_MAX);
    if (i <= 0 && !eap->skip && subflags.do_error) {
      emsg(_(e_zerocount));
      xfree(sub);
      return 0;
    } else if (i >= INT_MAX) {
      char buf[20];
      vim_snprintf(buf, sizeof(buf), "%d", i);
      semsg(_(e_val_too_large), buf);
      xfree(sub);
      return 0;
    }
    eap->line1 = eap->line2;
    eap->line2 += (linenr_T)i - 1;
    eap->line2 = MIN(eap->line2, curbuf->b_ml.ml_line_count);
  }

  // check for trailing command or garbage
  cmd = skipwhite(cmd);
  if (*cmd && *cmd != '"') {        // if not end-of-line or comment
    eap->nextcmd = check_nextcmd(cmd);
    if (eap->nextcmd == NULL) {
      semsg(_(e_trailing_arg), cmd);
      xfree(sub);
      return 0;
    }
  }

  if (eap->skip) {          // not executing commands, only parsing
    xfree(sub);
    return 0;
  }

  if (!subflags.do_count && !MODIFIABLE(curbuf)) {
    // Substitution is not allowed in non-'modifiable' buffer
    emsg(_(e_modifiable));
    xfree(sub);
    return 0;
  }

  if (search_regcomp(pat, patlen, NULL, RE_SUBST, which_pat,
                     (cmdpreview_ns > 0 ? 0 : SEARCH_HIS), &regmatch) == FAIL) {
    if (subflags.do_error) {
      emsg(_(e_invcmd));
    }
    xfree(sub);
    return 0;
  }

  // the 'i' or 'I' flag overrules 'ignorecase' and 'smartcase'
  if (subflags.do_ic == kSubIgnoreCase) {
    regmatch.rmm_ic = true;
  } else if (subflags.do_ic == kSubMatchCase) {
    regmatch.rmm_ic = false;
  }

  sub_firstline = NULL;

  assert(sub != NULL);

  // If the substitute pattern starts with "\=" then it's an expression.
  // Make a copy, a recursive function may free it.
  // Otherwise, '~' in the substitute pattern is replaced with the old
  // pattern.  We do it here once to avoid it to be replaced over and over
  // again.
  if (sub[0] == '\\' && sub[1] == '=') {
    char *p = xstrdup(sub);
    xfree(sub);
    sub = p;
  } else {
    char *p = regtilde(sub, rs_magic_isset(), cmdpreview_ns > 0);
    if (p != sub) {
      xfree(sub);
      sub = p;
    }
  }

  // Check for a match on each line.
  // If preview: limit to max('cmdwinheight', viewport).
  linenr_T line2 = eap->line2;

  for (linenr_T lnum = eap->line1;
       lnum <= line2 && !got_quit && !aborting()
       && (cmdpreview_ns <= 0 || preview_lines.lines_needed <= (linenr_T)p_cwh
           || lnum <= curwin->w_botline);
       lnum++) {
    int nmatch = vim_regexec_multi(&regmatch, curwin, curbuf, lnum,
                                   0, NULL, NULL);
    if (nmatch) {
      colnr_T copycol;
      colnr_T matchcol;
      colnr_T prev_matchcol = MAXCOL;
      char *new_end;
      char *new_start = NULL;
      int new_start_len = 0;
      char *p1;
      bool did_sub = false;
      int lastone;
      linenr_T nmatch_tl = 0;               // nr of lines matched below lnum
      int do_again;                     // do it again after joining lines
      bool skip_match = false;
      linenr_T sub_firstlnum;           // nr of first sub line

      // Track where substitutions started (set once per line).
      linenr_T lnum_start = 0;

      // Track per-line data for each match.
      // Will be sent as a batch to `extmark_splice` after the substitution is done.
      typedef struct {
        int start_col;         // Position in new text where replacement goes
        lpos_T start;          // Match start position in original text
        lpos_T end;            // Match end position in original text
        int matchcols;         // Columns deleted from original text
        bcount_t matchbytes;   // Bytes deleted from original text
        int subcols;           // Columns in replacement text
        bcount_t subbytes;     // Bytes in replacement text
        linenr_T lnum_before;  // Line number before this substitution
        linenr_T lnum_after;   // Line number after this substitution
      } LineData;

      kvec_t(LineData) line_matches = KV_INITIAL_VALUE;

      // The new text is build up step by step, to avoid too much
      // copying.  There are these pieces:
      // sub_firstline  The old text, unmodified.
      // copycol                Column in the old text where we started
      //                        looking for a match; from here old text still
      //                        needs to be copied to the new text.
      // matchcol               Column number of the old text where to look
      //                        for the next match.  It's just after the
      //                        previous match or one further.
      // prev_matchcol  Column just after the previous match (if any).
      //                        Mostly equal to matchcol, except for the first
      //                        match and after skipping an empty match.
      // regmatch.*pos  Where the pattern matched in the old text.
      // new_start      The new text, all that has been produced so
      //                        far.
      // new_end                The new text, where to append new text.
      //
      // lnum           The line number where we found the start of
      //                        the match.  Can be below the line we searched
      //                        when there is a \n before a \zs in the
      //                        pattern.
      // sub_firstlnum  The line number in the buffer where to look
      //                        for a match.  Can be different from "lnum"
      //                        when the pattern or substitute string contains
      //                        line breaks.
      //
      // Special situations:
      // - When the substitute string contains a line break, the part up
      //   to the line break is inserted in the text, but the copy of
      //   the original line is kept.  "sub_firstlnum" is adjusted for
      //   the inserted lines.
      // - When the matched pattern contains a line break, the old line
      //   is taken from the line at the end of the pattern.  The lines
      //   in the match are deleted later, "sub_firstlnum" is adjusted
      //   accordingly.
      //
      // The new text is built up in new_start[].  It has some extra
      // room to avoid using xmalloc()/free() too often.  new_start_len is
      // the length of the allocated memory at new_start.
      //
      // Make a copy of the old line, so it won't be taken away when
      // updating the screen or handling a multi-line match.  The "old_"
      // pointers point into this copy.
      sub_firstlnum = lnum;
      copycol = 0;
      matchcol = 0;

      // At first match, remember current cursor position.
      if (!got_match) {
        setpcmark();
        got_match = true;
      }

      // Loop until nothing more to replace in this line.
      // 1. Handle match with empty string.
      // 2. If subflags.do_ask is set, ask for confirmation.
      // 3. substitute the string.
      // 4. if subflags.do_all is set, find next match
      // 5. break if there isn't another match in this line
      while (true) {
        SubResult current_match = {
          .start = { 0, 0 },
          .end = { 0, 0 },
          .pre_match = 0,
        };
        // lnum is where the match start, but maybe not the pattern match,
        // since we can have \n before \zs in the pattern

        // Advance "lnum" to the line where the match starts.  The
        // match does not start in the first line when there is a line
        // break before \zs.
        if (regmatch.startpos[0].lnum > 0) {
          current_match.pre_match = lnum;
          lnum += regmatch.startpos[0].lnum;
          sub_firstlnum += regmatch.startpos[0].lnum;
          nmatch -= regmatch.startpos[0].lnum;
          XFREE_CLEAR(sub_firstline);
        }

        // Now we're at the line where the pattern match starts
        // Note: If not first match on a line, column can't be known here
        current_match.start.lnum = sub_firstlnum;

        // Match might be after the last line for "\n\zs" matching at
        // the end of the last line.
        if (lnum > curbuf->b_ml.ml_line_count) {
          break;
        }
        if (sub_firstline == NULL) {
          sub_firstline = xstrnsave(ml_get(sub_firstlnum),
                                    (size_t)ml_get_len(sub_firstlnum));
        }

        // Save the line number of the last change for the final
        // cursor position (just like Vi).
        curwin->w_cursor.lnum = lnum;
        do_again = false;

        // 1. Match empty string does not count, except for first
        // match.  This reproduces the strange vi behaviour.
        // This also catches endless loops.
        if (matchcol == prev_matchcol
            && regmatch.endpos[0].lnum == 0
            && matchcol == regmatch.endpos[0].col) {
          if (sub_firstline[matchcol] == NUL) {
            // We already were at the end of the line.  Don't look
            // for a match in this line again.
            skip_match = true;
          } else {
            // search for a match at next column
            matchcol += utfc_ptr2len(sub_firstline + matchcol);
          }
          // match will be pushed to preview_lines, bring it into a proper state
          current_match.start.col = matchcol;
          current_match.end.lnum = sub_firstlnum;
          current_match.end.col = matchcol;
          goto skip;
        }

        // Normally we continue searching for a match just after the
        // previous match.
        matchcol = regmatch.endpos[0].col;
        prev_matchcol = matchcol;

        // 2. If subflags.do_count is set only increase the counter.
        //    If do_ask is set, ask for confirmation.
        if (subflags.do_count) {
          // For a multi-line match, put matchcol at the NUL at
          // the end of the line and set nmatch to one, so that
          // we continue looking for a match on the next line.
          // Avoids that ":s/\nB\@=//gc" get stuck.
          if (nmatch > 1) {
            matchcol = (colnr_T)strlen(sub_firstline);
            nmatch = 1;
            skip_match = true;
          }
          sub_nsubs++;
          did_sub = true;
          // Skip the substitution, unless an expression is used,
          // then it is evaluated in the sandbox.
          if (!(sub[0] == '\\' && sub[1] == '=')) {
            goto skip;
          }
        }

        if (subflags.do_ask && cmdpreview_ns <= 0) {
          int typed = 0;
          int save_State = State;
          curwin->w_cursor.col = regmatch.startpos[0].col;

          if (curwin->w_p_crb) {
            do_check_cursorbind();
          }

          // When 'cpoptions' contains "u" don't sync undo when
          // asking for confirmation.
          if (vim_strchr(p_cpo, CPO_UNDO) != NULL) {
            no_u_sync++;
          }

          // Loop until 'y', 'n', 'q', CTRL-E or CTRL-Y typed.
          while (subflags.do_ask) {
            if (exmode_active) {
              rs_print_line_no_prefix(lnum, subflags.do_number, subflags.do_list);

              colnr_T sc, ec;
              getvcol(curwin, &curwin->w_cursor, &sc, NULL, NULL);
              curwin->w_cursor.col = MAX(regmatch.endpos[0].col - 1, 0);

              getvcol(curwin, &curwin->w_cursor, NULL, NULL, &ec);
              curwin->w_cursor.col = regmatch.startpos[0].col;
              if (subflags.do_number || curwin->w_p_nu) {
                int numw = number_width(curwin) + 1;
                sc += numw;
                ec += numw;
              }

              char *prompt = xmallocz((size_t)ec + 1);
              memset(prompt, ' ', (size_t)sc);
              memset(prompt + sc, '^', (size_t)(ec - sc) + 1);
              char *resp = getcmdline_prompt(-1, prompt, 0, EXPAND_NOTHING, NULL,
                                             CALLBACK_NONE, false, NULL);
              msg_putchar('\n');
              xfree(prompt);
              if (resp != NULL) {
                typed = (uint8_t)(*resp);
                xfree(resp);
              } else {
                // getcmdline_prompt() returns NULL if there is no command line to return.
                typed = NUL;
              }
              // When ":normal" runs out of characters we get
              // an empty line.  Use "q" to get out of the
              // loop.
              if (ex_normal_busy && typed == NUL) {
                typed = 'q';
              }
            } else {
              char *orig_line = NULL;
              int len_change = 0;
              const bool save_p_lz = p_lz;
              int save_p_fen = curwin->w_p_fen;

              curwin->w_p_fen = false;
              // Invert the matched string.
              // Remove the inversion afterwards.
              int temp = RedrawingDisabled;
              RedrawingDisabled = 0;

              // avoid calling update_screen() in vgetorpeek()
              p_lz = false;

              if (new_start != NULL) {
                // There already was a substitution, we would
                // like to show this to the user.  We cannot
                // really update the line, it would change
                // what matches.  Temporarily replace the line
                // and change it back afterwards.
                orig_line = xstrnsave(ml_get(lnum), (size_t)ml_get_len(lnum));
                char *new_line = concat_str(new_start, sub_firstline + copycol);

                // Position the cursor relative to the end of the line, the
                // previous substitute may have inserted or deleted characters
                // before the cursor.
                len_change = (int)strlen(new_line) - (int)strlen(orig_line);
                curwin->w_cursor.col += len_change;
                ml_replace(lnum, new_line, false);
              }

              search_match_lines = regmatch.endpos[0].lnum
                                   - regmatch.startpos[0].lnum;
              search_match_endcol = regmatch.endpos[0].col
                                    + len_change;
              if (search_match_lines == 0 && search_match_endcol == 0) {
                // highlight at least one character for /^/
                search_match_endcol = 1;
              }
              highlight_match = true;

              update_topline(curwin);
              validate_cursor(curwin);
              redraw_later(curwin, UPD_SOME_VALID);
              show_cursor_info_later(true);
              update_screen();
              redraw_later(curwin, UPD_SOME_VALID);

              curwin->w_p_fen = save_p_fen;

              char *p = _("replace with %s? (y)es/(n)o/(a)ll/(q)uit/(l)ast/scroll up(^E)/down(^Y)");
              snprintf(IObuff, IOSIZE, p, sub);
              p = xstrdup(IObuff);
              typed = prompt_for_input(p, HLF_R, true, NULL);
              highlight_match = false;
              xfree(p);

              msg_didout = false;                 // don't scroll up
              gotocmdline(true);
              p_lz = save_p_lz;
              RedrawingDisabled = temp;

              // restore the line
              if (orig_line != NULL) {
                ml_replace(lnum, orig_line, false);
              }
            }

            need_wait_return = false;             // no hit-return prompt
            if (typed == 'q' || typed == ESC || typed == Ctrl_C) {
              got_quit = true;
              break;
            }
            if (typed == 'n') {
              break;
            }
            if (typed == 'y') {
              break;
            }
            if (typed == 'l') {
              // last: replace and then stop
              subflags.do_all = false;
              line2 = lnum;
              break;
            }
            if (typed == 'a') {
              subflags.do_ask = false;
              break;
            }
            if (typed == Ctrl_E) {
              scrollup_clamp();
            } else if (typed == Ctrl_Y) {
              scrolldown_clamp();
            }
          }
          State = save_State;
          setmouse();
          if (vim_strchr(p_cpo, CPO_UNDO) != NULL) {
            no_u_sync--;
          }

          if (typed == 'n') {
            // For a multi-line match, put matchcol at the NUL at
            // the end of the line and set nmatch to one, so that
            // we continue looking for a match on the next line.
            // Avoids that ":%s/\nB\@=//gc" and ":%s/\n/,\r/gc"
            // get stuck when pressing 'n'.
            if (nmatch > 1) {
              matchcol = (colnr_T)strlen(sub_firstline);
              skip_match = true;
            }
            goto skip;
          }
          if (got_quit) {
            goto skip;
          }
        }

        // Move the cursor to the start of the match, so that we can
        // use "\=col(".").
        curwin->w_cursor.col = regmatch.startpos[0].col;

        // When the match included the "$" of the last line it may
        // go beyond the last line of the buffer.
        if (nmatch > curbuf->b_ml.ml_line_count - sub_firstlnum + 1) {
          nmatch = curbuf->b_ml.ml_line_count - sub_firstlnum + 1;
          current_match.end.lnum = sub_firstlnum + (linenr_T)nmatch;
          skip_match = true;
          // safety check
          if (nmatch < 0) {
            goto skip;
          }
        }

        // Save the line numbers for the preview buffer
        // NOTE: If the pattern matches a final newline, the next line will
        // be shown also, but should not be highlighted. Intentional for now.
        if (cmdpreview_ns > 0 && !has_second_delim) {
          current_match.start.col = regmatch.startpos[0].col;
          if (current_match.end.lnum == 0) {
            current_match.end.lnum = sub_firstlnum + (linenr_T)nmatch - 1;
          }
          current_match.end.col = regmatch.endpos[0].col;

          ADJUST_SUB_FIRSTLNUM();
          lnum += (linenr_T)nmatch - 1;

          goto skip;
        }

        // 3. Substitute the string. During 'inccommand' preview only do this if
        //    there is a replace pattern.
        if (cmdpreview_ns <= 0 || has_second_delim) {
          lnum_start = lnum;  // save the start lnum
          int save_ma = curbuf->b_p_ma;
          int save_sandbox = sandbox;
          if (subflags.do_count) {
            // prevent accidentally changing the buffer by a function
            curbuf->b_p_ma = false;
            sandbox++;
          }
          // Save flags for recursion.  They can change for e.g.
          // :s/^/\=execute("s#^##gn")
          subflags_T subflags_save = subflags;

          // Disallow changing text or switching window in an expression.
          textlock++;
          // Get length of substitution part, including the NUL.
          // When it fails sublen is zero.
          sublen = vim_regsub_multi(&regmatch,
                                    sub_firstlnum - regmatch.startpos[0].lnum,
                                    sub, sub_firstline, 0,
                                    REGSUB_BACKSLASH
                                    | (rs_magic_isset() ? REGSUB_MAGIC : 0));
          textlock--;

          // If getting the substitute string caused an error, don't do
          // the replacement.
          // Don't keep flags set by a recursive call
          subflags = subflags_save;
          if (sublen == 0 || aborting() || subflags.do_count) {
            curbuf->b_p_ma = save_ma;
            sandbox = save_sandbox;
            goto skip;
          }

          // Need room for:
          // - result so far in new_start (not for first sub in line)
          // - original text up to match
          // - length of substituted part
          // - original text after match
          if (nmatch == 1) {
            p1 = sub_firstline;
          } else {
            p1 = ml_get(sub_firstlnum + (linenr_T)nmatch - 1);
            nmatch_tl += nmatch - 1;
          }
          int copy_len = regmatch.startpos[0].col - copycol;
          new_end = rs_sub_grow_buf(&new_start, &new_start_len,
                                 (colnr_T)strlen(p1) - regmatch.endpos[0].col
                                 + copy_len + sublen + 1);

          // copy the text up to the part that matched
          memmove(new_end, sub_firstline + copycol, (size_t)copy_len);
          new_end += copy_len;

          if (new_start_len - copy_len < sublen) {
            sublen = new_start_len - copy_len - 1;
          }

          // Finally, at this point we can know where the match actually will
          // start in the new text
          int start_col = (int)(new_end - new_start);
          current_match.start.col = start_col;

          textlock++;
          vim_regsub_multi(&regmatch,
                           sub_firstlnum - regmatch.startpos[0].lnum,
                           sub, new_end, sublen,
                           REGSUB_COPY | REGSUB_BACKSLASH
                           | (rs_magic_isset() ? REGSUB_MAGIC : 0));
          textlock--;
          sub_nsubs++;
          did_sub = true;

          // Move the cursor to the start of the line, to avoid that it
          // is beyond the end of the line after the substitution.
          curwin->w_cursor.col = 0;

          // Remember next character to be copied.
          copycol = regmatch.endpos[0].col;

          ADJUST_SUB_FIRSTLNUM();

          // TODO(bfredl): this has some robustness issues, look into later.
          bcount_t replaced_bytes = 0;
          lpos_T start = regmatch.startpos[0];
          lpos_T end = regmatch.endpos[0];
          for (i = 0; i < nmatch - 1; i++) {
            replaced_bytes += (bcount_t)strlen(ml_get((linenr_T)(lnum_start + i))) + 1;
          }
          replaced_bytes += end.col - start.col;

          // Save the line number before processing newlines.
          linenr_T lnum_before_newlines = lnum;

          // Now the trick is to replace CTRL-M chars with a real line
          // break.  This would make it impossible to insert a CTRL-M in
          // the text.  The line break can be avoided by preceding the
          // CTRL-M with a backslash.  To be able to insert a backslash,
          // they must be doubled in the string and are halved here.
          // That is Vi compatible.
          for (p1 = new_end; *p1; p1++) {
            if (p1[0] == '\\' && p1[1] != NUL) {            // remove backslash
              sublen--;  // correct the byte counts for extmark_splice()
              STRMOVE(p1, p1 + 1);
            } else if (*p1 == CAR) {
              if (u_inssub(lnum) == OK) {             // prepare for undo
                *p1 = NUL;                            // truncate up to the CR
                ml_append(lnum - 1, new_start,
                          (colnr_T)(p1 - new_start + 1), false);
                mark_adjust(lnum + 1, (linenr_T)MAXLNUM, 1, 0, kExtmarkNOOP);

                if (subflags.do_ask) {
                  appended_lines(lnum - 1, 1);
                } else {
                  if (first_line == 0) {
                    first_line = lnum;
                  }
                  last_line = lnum + 1;
                }
                // All line numbers increase.
                sub_firstlnum++;
                lnum++;
                line2++;
                // move the cursor to the new line, like Vi
                curwin->w_cursor.lnum++;
                // copy the rest
                STRMOVE(new_start, p1 + 1);
                p1 = new_start - 1;
              }
            } else {
              p1 += utfc_ptr2len(p1) - 1;
            }
          }
          colnr_T new_endcol = (colnr_T)strlen(new_start);
          current_match.end.col = new_endcol;
          current_match.end.lnum = lnum;

          int matchcols = end.col - ((end.lnum == start.lnum)
                                     ? start.col : 0);
          int subcols = new_endcol - ((lnum == lnum_start) ? start_col : 0);
          if (!did_save) {
            // Required for Undo to work for extmarks.
            u_save_cursor();
            did_save = true;
          }

          // Store extmark data for this match.
          LineData *data = kv_pushp(line_matches);
          data->start_col = start_col;
          data->start = start;
          data->end = end;
          data->matchcols = matchcols;
          data->matchbytes = replaced_bytes;
          data->subcols = subcols;
          data->subbytes = sublen - 1;
          data->lnum_before = lnum_before_newlines;
          data->lnum_after = lnum;
        }

        // 4. If subflags.do_all is set, find next match.
        // Prevent endless loop with patterns that match empty
        // strings, e.g. :s/$/pat/g or :s/[a-z]* /(&)/g.
        // But ":s/\n/#/" is OK.
skip:
        // We already know that we did the last subst when we are at
        // the end of the line, except that a pattern like
        // "bar\|\nfoo" may match at the NUL.  "lnum" can be below
        // "line2" when there is a \zs in the pattern after a line
        // break.
        lastone = (skip_match
                   || got_int
                   || got_quit
                   || lnum > line2
                   || !(subflags.do_all || do_again)
                   || (sub_firstline[matchcol] == NUL && nmatch <= 1
                       && !re_multiline(regmatch.regprog)));
        nmatch = -1;

        // Replace the line in the buffer when needed.  This is
        // skipped when there are more matches.
        // The check for nmatch_tl is needed for when multi-line
        // matching must replace the lines before trying to do another
        // match, otherwise "\@<=" won't work.
        // When the match starts below where we start searching also
        // need to replace the line first (using \zs after \n).
        if (lastone
            || nmatch_tl > 0
            || (nmatch = vim_regexec_multi(&regmatch, curwin,
                                           curbuf, sub_firstlnum,
                                           matchcol, NULL, NULL)) == 0
            || regmatch.startpos[0].lnum > 0) {
          if (new_start != NULL) {
            // Copy the rest of the line, that didn't match.
            // "matchcol" has to be adjusted, we use the end of
            // the line as reference, because the substitute may
            // have changed the number of characters.  Same for
            // "prev_matchcol".
            strcat(new_start, sub_firstline + copycol);
            matchcol = (colnr_T)strlen(sub_firstline) - matchcol;
            prev_matchcol = (colnr_T)strlen(sub_firstline)
                            - prev_matchcol;

            if (u_savesub(lnum) != OK) {
              break;
            }
            ml_replace(lnum, new_start, true);

            // Call extmark_splice for each match on this line.
            for (size_t match_idx = 0; match_idx < kv_size(line_matches); match_idx++) {
              LineData *match = &kv_A(line_matches, match_idx);

              extmark_splice(curbuf, (int)match->lnum_before - 1, match->start_col,
                             match->end.lnum - match->start.lnum, match->matchcols,
                             match->matchbytes,
                             match->lnum_after - match->lnum_before,
                             match->subcols,
                             match->subbytes, kExtmarkUndo);
            }

            // Reset the match data for the next line.
            kv_size(line_matches) = 0;

            if (nmatch_tl > 0) {
              // Matched lines have now been substituted and are
              // useless, delete them.  The part after the match
              // has been appended to new_start, we don't need
              // it in the buffer.
              lnum++;
              if (u_savedel(lnum, nmatch_tl) != OK) {
                break;
              }
              for (i = 0; i < nmatch_tl; i++) {
                ml_delete(lnum);
              }
              mark_adjust(lnum, lnum + nmatch_tl - 1, MAXLNUM, -nmatch_tl, kExtmarkNOOP);
              if (subflags.do_ask) {
                deleted_lines(lnum, nmatch_tl);
              }
              lnum--;
              line2 -= nmatch_tl;  // nr of lines decreases
              nmatch_tl = 0;
            }

            // When asking, undo is saved each time, must also set
            // changed flag each time.
            if (subflags.do_ask) {
              changed_bytes(lnum, 0);
            } else {
              if (first_line == 0) {
                first_line = lnum;
              }
              last_line = lnum + 1;
            }

            sub_firstlnum = lnum;
            xfree(sub_firstline);                // free the temp buffer
            sub_firstline = new_start;
            new_start = NULL;
            matchcol = (colnr_T)strlen(sub_firstline) - matchcol;
            prev_matchcol = (colnr_T)strlen(sub_firstline)
                            - prev_matchcol;
            copycol = 0;
          }
          if (nmatch == -1 && !lastone) {
            nmatch = vim_regexec_multi(&regmatch, curwin, curbuf,
                                       sub_firstlnum, matchcol, NULL, NULL);
          }

          // 5. break if there isn't another match in this line
          if (nmatch <= 0) {
            // If the match found didn't start where we were
            // searching, do the next search in the line where we
            // found the match.
            if (nmatch == -1) {
              lnum -= regmatch.startpos[0].lnum;
            }

            // uncrustify:off

#define PUSH_PREVIEW_LINES() \
  do { \
    if (cmdpreview_ns > 0) { \
      linenr_T match_lines = current_match.end.lnum \
                             - current_match.start.lnum +1; \
      if (preview_lines.subresults.size > 0) { \
        linenr_T last = kv_last(preview_lines.subresults).end.lnum; \
        if (last == current_match.start.lnum) { \
          preview_lines.lines_needed += match_lines - 1; \
        } else { \
          preview_lines.lines_needed += match_lines; \
        } \
      } else { \
        preview_lines.lines_needed += match_lines; \
      } \
      kv_push(preview_lines.subresults, current_match); \
    } \
  } while (0)

            // uncrustify:on

            // Push the match to preview_lines.
            PUSH_PREVIEW_LINES();

            break;
          }
        }
        // Push the match to preview_lines.
        PUSH_PREVIEW_LINES();

        line_breakcheck();
      }

      if (did_sub) {
        sub_nlines++;
      }
      xfree(new_start);              // for when substitute was cancelled
      XFREE_CLEAR(sub_firstline);    // free the copy of the original line
      kv_destroy(line_matches);      // clean up match data
    }

    line_breakcheck();

    if (profile_passed_limit(timeout)) {
      got_quit = true;
    }
  }

  curbuf->deleted_bytes2 = 0;

  if (first_line != 0) {
    // Need to subtract the number of added lines from "last_line" to get
    // the line number before the change (same as adding the number of
    // deleted lines).
    i = curbuf->b_ml.ml_line_count - old_line_count;
    changed_lines(curbuf, first_line, 0, last_line - (linenr_T)i, (linenr_T)i, false);

    int64_t num_added = last_line - first_line;
    int64_t num_removed = num_added - i;
    buf_updates_send_changes(curbuf, first_line, num_added, num_removed);
  }

  xfree(sub_firstline);   // may have to free allocated copy of the line

  // ":s/pat//n" doesn't move the cursor
  if (subflags.do_count) {
    curwin->w_cursor = old_cursor;
  }

  if (sub_nsubs > start_nsubs) {
    if ((cmdmod.cmod_flags & CMOD_LOCKMARKS) == 0) {
      // Set the '[ and '] marks.
      curbuf->b_op_start.lnum = eap->line1;
      curbuf->b_op_end.lnum = line2;
      curbuf->b_op_start.col = curbuf->b_op_end.col = 0;
    }

    if (!global_busy) {
      // when interactive leave cursor on the match
      if (!subflags.do_ask) {
        if (endcolumn) {
          coladvance(curwin, MAXCOL);
        } else {
          beginline(BL_WHITE | BL_FIX);
        }
      }
      if (cmdpreview_ns <= 0 && !rs_do_sub_msg(subflags.do_count) && subflags.do_ask && p_ch > 0) {
        msg("", 0);
      }
    } else {
      global_need_beginline = true;
    }
    if (subflags.do_print) {
      rs_print_line(curwin->w_cursor.lnum, subflags.do_number, subflags.do_list, true);
    }
  } else if (!global_busy) {
    if (got_int) {
      // interrupted
      emsg(_(e_interr));
    } else if (got_match) {
      // did find something but nothing substituted
      if (p_ch > 0) {
        msg("", 0);
      }
    } else if (subflags.do_error) {
      // nothing found
      semsg(_(e_patnotf2), get_search_pat());
    }
  }

  if (subflags.do_ask && rs_hasAnyFolding(curwin)) {
    // Cursor position may require updating
    changed_window_setting(curwin);
  }

  vim_regfree(regmatch.regprog);
  xfree(sub);

  // Restore the flag values, they can be used for ":&&".
  subflags.do_all = save_do_all;
  subflags.do_ask = save_do_ask;

  int retv = 0;

  // Show 'inccommand' preview if there are matched lines.
  if (cmdpreview_ns > 0 && !aborting()) {
    if (got_quit || profile_passed_limit(timeout)) {  // Too slow, disable.
      set_option_direct(kOptInccommand, STATIC_CSTR_AS_OPTVAL(""), 0, SID_NONE);
    } else if (*p_icm != NUL && pat != NULL) {
      if (pre_hl_id == 0) {
        pre_hl_id = syn_check_group(S_LEN("Substitute"));
      }
      retv = show_sub(eap, old_cursor, &preview_lines, pre_hl_id, cmdpreview_ns, cmdpreview_bufnr);
    }
  }

  kv_destroy(preview_lines.subresults);
  return retv;
#undef ADJUST_SUB_FIRSTLNUM
#undef PUSH_PREVIEW_LINES
}

/// Format and display the substitution count message.
///
/// Handles the NGETTEXT formatting (which must stay in C for i18n) and message
/// display. This is called by rs_do_sub_msg when thresholds are met.
/// Returns true if the message was displayed.
bool nvim_excmds_format_sub_msg(bool count_only)
{
  if (got_int) {
    STRCPY(msg_buf, _("(Interrupted) "));
  } else {
    *msg_buf = NUL;
  }

  char *msg_single = count_only
                     ? NGETTEXT("%" PRId64 " match on %" PRId64 " line",
                                "%" PRId64 " matches on %" PRId64 " line", sub_nsubs)
                     : NGETTEXT("%" PRId64 " substitution on %" PRId64 " line",
                                "%" PRId64 " substitutions on %" PRId64 " line", sub_nsubs);
  char *msg_plural = count_only
                     ? NGETTEXT("%" PRId64 " match on %" PRId64 " lines",
                                "%" PRId64 " matches on %" PRId64 " lines", sub_nsubs)
                     : NGETTEXT("%" PRId64 " substitution on %" PRId64 " lines",
                                "%" PRId64 " substitutions on %" PRId64 " lines", sub_nsubs);
  vim_snprintf_add(msg_buf, sizeof(msg_buf),
                   NGETTEXT(msg_single, msg_plural, sub_nlines),
                   (int64_t)sub_nsubs, (int64_t)sub_nlines);
  if (msg(msg_buf, 0)) {
    set_keep_msg(msg_buf, 0);
  }
  return true;
}

/// Accessor: return KeyTyped global.
int nvim_excmds_get_KeyTyped(void) { return KeyTyped ? 1 : 0; }

/// Accessor: return messaging() result.
int nvim_excmds_messaging(void) { return messaging() ? 1 : 0; }

/// Give message for number of substitutions.
/// Can also be used after a ":global" command.
///
/// @param count_only  used 'n' flag for ":s"
///
/// @return            true if a message was given.
bool do_sub_msg(bool count_only)
{
  return rs_do_sub_msg(count_only);
}

// ex_global implemented in Rust (rs_ex_global in ex_cmds/src/global.rs)
extern void rs_ex_global(exarg_T *eap);

/// Execute a global command of the form:
///
/// g/pattern/X : execute X on all lines where pattern matches
/// v/pattern/X : execute X on all lines where pattern does not match
///
/// where 'X' is an EX command
///
/// The command character (as well as the trailing slash) is optional, and
/// is assumed to be 'p' if missing.
///
/// This is implemented in two passes: first we scan the file for the pattern and
/// set a mark for each line that (not) matches. Secondly we execute the command
/// for each line that has a mark. This is required because after deleting
/// lines we do not know where to search for the next match.
void ex_global(exarg_T *eap)
{
  rs_ex_global(eap);
}

// global_exe + global_exe_one implemented in Rust (rs_global_exe in ex_cmds/src/global.rs)
extern void rs_global_exe(char *cmd);

/// Execute `cmd` on lines marked with ml_setmarked(). Thin wrapper calling Rust.
void global_exe(char *cmd)
{
  rs_global_exe(cmd);
}

#if defined(EXITFREE)
/// EXITFREE cleanup for old_sub. Thin wrapper calling Rust.
void free_old_sub(void)
{
  rs_free_old_sub();
}

#endif

/// Set up for a tagpreview.
///
/// @param undo_sync  sync undo when leaving the window
///
/// @return           true when it was created.
// --- prepare_tagpreview FFI accessors ---

int nvim_excmds_curwin_get_pvw(void) { return curwin->w_p_pvw; }
void nvim_excmds_curwin_set_pvw(int val) { curwin->w_p_pvw = (bool)val; }
void nvim_excmds_curwin_set_wfh(int val) { curwin->w_p_wfh = (bool)val; }
void nvim_excmds_curwin_set_diff(int val) { curwin->w_p_diff = (bool)val; }

// Returns non-NULL win_T pointer to the first preview window found in curtab, or NULL.
win_T *nvim_excmds_find_preview_win(void)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_p_pvw) {
      return wp;
    }
  }
  return NULL;
}

void nvim_excmds_win_enter(win_T *wp, int undo_sync)
{
  win_enter(wp, (bool)undo_sync);
}

// Returns 0 on success (OK), -1 on failure (FAIL)
int nvim_excmds_win_split(int size, int flags)
{
  return win_split(size, flags) == FAIL ? -1 : 0;
}

int nvim_excmds_get_g_do_tagpreview(void) { return g_do_tagpreview; }

void nvim_excmds_reset_binding_curwin(void)
{
  RESET_BINDING(curwin);
}

void nvim_excmds_set_foldcolumn_zero(void)
{
  set_option_direct(kOptFoldcolumn, STATIC_CSTR_AS_OPTVAL("0"), 0, SID_NONE);
}

// prepare_tagpreview implemented in Rust (rs_prepare_tagpreview in ex_cmds/src/window.rs)
extern bool rs_prepare_tagpreview(int undo_sync);

/// Sets up the preview window for tag preview. Thin wrapper calling the Rust implementation.
bool prepare_tagpreview(bool undo_sync)
{
  return rs_prepare_tagpreview((int)undo_sync);
}

// show_sub implemented in Rust (rs_show_sub in ex_cmds/src/substitute.rs)
extern int rs_show_sub(exarg_T *eap, linenr_T old_cusr_lnum, colnr_T old_cusr_col,
                       const PreviewLines *preview_lines, int hl_id,
                       int cmdpreview_ns, handle_T cmdpreview_bufnr);

/// Shows the effects of :substitute for inccommand. Thin wrapper calling Rust.
static int show_sub(exarg_T *eap, pos_T old_cusr, PreviewLines *preview_lines, int hl_id,
                    int cmdpreview_ns, handle_T cmdpreview_bufnr)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_show_sub(eap, old_cusr.lnum, old_cusr.col, preview_lines, hl_id,
                     cmdpreview_ns, cmdpreview_bufnr);
}

/// :substitute command.
/// :substitute command. Thin wrapper calling Rust.
void ex_substitute(exarg_T *eap)
{
  rs_ex_substitute(eap);
}

/// :substitute command preview callback. Thin wrapper calling Rust.
int ex_substitute_preview(exarg_T *eap, int cmdpreview_ns, handle_T cmdpreview_bufnr)
{
  return rs_ex_substitute_preview(eap, cmdpreview_ns, cmdpreview_bufnr);
}

// --- ex_oldfiles FFI accessors ---

// Iterate v:oldfiles list: returns opaque iterator handle, or NULL if empty.
// Each call to nvim_excmds_oldfiles_iter_next() returns the next filename string (or NULL).
// Call nvim_excmds_oldfiles_iter_free() to release the handle.
typedef struct {
  list_T *list;
  listitem_T *item;
  int len;
} OldfilesIter;

void *nvim_excmds_oldfiles_iter_start(int *out_len)
{
  list_T *l = get_vim_var_list(VV_OLDFILES);
  if (l == NULL) {
    *out_len = 0;
    return NULL;
  }
  OldfilesIter *it = xmalloc(sizeof(OldfilesIter));
  it->list = l;
  it->item = tv_list_first(l);
  it->len = (int)tv_list_len(l);
  *out_len = it->len;
  return it;
}

const char *nvim_excmds_oldfiles_iter_next(void *handle)
{
  OldfilesIter *it = (OldfilesIter *)handle;
  if (it->item == NULL) {
    return NULL;
  }
  const char *s = tv_get_string(TV_LIST_ITEM_TV(it->item));
  it->item = TV_LIST_ITEM_NEXT(it->list, it->item);
  return s;
}

void nvim_excmds_oldfiles_iter_free(void *handle)
{
  xfree(handle);
}

int nvim_excmds_oldfiles_list_len(void)
{
  list_T *l = get_vim_var_list(VV_OLDFILES);
  return l == NULL ? 0 : (int)tv_list_len(l);
}

const char *nvim_excmds_oldfiles_find_str(int idx)
{
  list_T *l = get_vim_var_list(VV_OLDFILES);
  if (l == NULL) {
    return NULL;
  }
  return tv_list_find_str(l, idx);
}

void nvim_excmds_msg_start(void) { msg_start(); }
void nvim_excmds_set_msg_scroll(int val) { msg_scroll = (bool)val; }
void nvim_excmds_msg_outnum(int nr) { msg_outnum((long)nr); }
void nvim_excmds_msg_outtrans(const char *s) { msg_outtrans((char *)s, 0, false); }
void nvim_excmds_msg_clr_eos(void) { msg_clr_eos(); }
void nvim_excmds_msg_putchar(int c) { msg_putchar(c); }
void nvim_excmds_os_breakcheck(void) { os_breakcheck(); }
void nvim_excmds_set_got_int(int val) { got_int = (bool)val; }
int nvim_excmds_cmdmod_has_browse(void) { return (cmdmod.cmod_flags & CMOD_BROWSE) != 0; }
void nvim_excmds_cmdmod_clear_browse(void) { cmdmod.cmod_flags &= ~CMOD_BROWSE; }
void nvim_excmds_set_quit_more(int val) { quit_more = (bool)val; }
int nvim_excmds_prompt_for_input(void) { return prompt_for_input(NULL, 0, false, NULL); }
void nvim_excmds_msg_starthere(void) { msg_starthere(); }
char *nvim_excmds_expand_env_save(const char *p) { return expand_env_save((char *)p); }
void nvim_excmds_do_exedit_edit(exarg_T *eap, char *arg)
{
  char *saved_arg = eap->arg;
  int saved_cmdidx = eap->cmdidx;
  eap->arg = arg;
  eap->cmdidx = CMD_edit;
  cmdmod.cmod_flags &= ~CMOD_BROWSE;
  do_exedit(eap, NULL);
  eap->arg = saved_arg;
  eap->cmdidx = saved_cmdidx;
}
void nvim_excmds_xfree(void *ptr) { xfree(ptr); }
void nvim_excmds_msg_no_old_files(void) { msg(_("No old files"), 0); }

// ex_oldfiles implemented in Rust (rs_ex_oldfiles in ex_cmds/src/display.rs)
extern void rs_ex_oldfiles(exarg_T *eap);

/// List v:oldfiles in a nice way. Thin wrapper calling the Rust implementation.
void ex_oldfiles(exarg_T *eap)
{
  rs_ex_oldfiles(eap);
}

// --- do_bang FFI accessors ---

int nvim_excmds_get_msg_scroll(void) { return msg_scroll ? 1 : 0; }
void nvim_excmds_autowrite_all(void) { autowrite_all(); }
int nvim_excmds_get_bangredo(void) { return bangredo ? 1 : 0; }
void nvim_excmds_set_bangredo(int val) { bangredo = (bool)val; }
char *nvim_excmds_vim_strsave_escaped(const char *s, const char *chars)
{
  return vim_strsave_escaped((char *)s, (char *)chars);
}
void nvim_excmds_append_to_redobuff_lit(const char *s, int len)
{
  AppendToRedobuffLit((char *)s, len);
}
void nvim_excmds_append_to_redobuff(const char *s) { AppendToRedobuff((char *)s); }
void nvim_excmds_ui_cursor_goto(int row, int col) { ui_cursor_goto(row, col); }
int nvim_excmds_get_msg_row(void) { return msg_row; }
int nvim_excmds_get_msg_col(void) { return msg_col; }
void nvim_excmds_do_shell_wrapper(char *cmd, int flags) { do_shell(cmd, flags); }
void nvim_excmds_do_filter_wrapper(linenr_T line1, linenr_T line2, exarg_T *eap,
                                    char *cmd, bool do_in, bool do_out)
{
  do_filter(line1, line2, eap, cmd, do_in, do_out);
}
void nvim_excmds_apply_autocmds_shellfilterpost(void)
{
  apply_autocmds(EVENT_SHELLFILTERPOST, NULL, NULL, false, curbuf);
}
void nvim_excmds_emsg_e_noprev(void) { emsg(_(e_noprev)); }
void nvim_excmds_msg_ext_set_kind_shell_cmd(void) { msg_ext_set_kind("shell_cmd"); }

// --- do_shell FFI accessors ---
int nvim_excmds_get_p_warn(void) { return p_warn ? 1 : 0; }
int nvim_excmds_get_autocmd_busy(void) { return autocmd_busy ? 1 : 0; }
int nvim_excmds_get_msg_silent(void) { return msg_silent; }
int nvim_excmds_any_buf_changed(void)
{
  FOR_ALL_BUFFERS(buf) {
    if (bufIsChanged(buf)) {
      return 1;
    }
  }
  return 0;
}
void nvim_excmds_msg_puts_no_write_warning(void)
{
  msg_puts(_("[No write since last change]\n"));
}
void nvim_excmds_call_shell(char *cmd, int flags)
{
  call_shell(cmd, flags, NULL);
}
void nvim_excmds_set_msg_didout(int val) { msg_didout = (bool)val; }
void nvim_excmds_set_did_check_timestamps(int val) { did_check_timestamps = (bool)val; }
void nvim_excmds_set_need_check_timestamps(int val) { need_check_timestamps = (bool)val; }
void nvim_excmds_set_msg_row(int val) { msg_row = val; }
void nvim_excmds_set_msg_col(int val) { msg_col = val; }
void nvim_excmds_apply_autocmds_shellcmdpost(void)
{
  apply_autocmds(EVENT_SHELLCMDPOST, NULL, NULL, false, curbuf);
}

// --- global_exe FFI accessors ---
void nvim_excmds_setpcmark(void) { setpcmark(); }
void nvim_excmds_set_global_busy(int val) { global_busy = val; }
int nvim_excmds_get_global_need_beginline(void) { return global_need_beginline ? 1 : 0; }
void nvim_excmds_set_global_need_beginline(int val) { global_need_beginline = (bool)val; }
linenr_T nvim_excmds_ml_firstmarked(void) { return ml_firstmarked(); }
void nvim_excmds_do_cmdline_global(const char *cmd)
{
  if (cmd == NULL || *cmd == NUL || *cmd == '\n') {
    do_cmdline("p", NULL, NULL, DOCMD_NOWAIT);
  } else {
    do_cmdline((char *)cmd, NULL, NULL, DOCMD_NOWAIT);
  }
}
void nvim_excmds_check_cursor_curwin(void) { check_cursor(curwin); }
void nvim_excmds_changed_line_abv_curs(void) { changed_line_abv_curs(); }
int nvim_excmds_get_msg_scrolled(void) { return msg_scrolled; }
void *nvim_excmds_get_curbuf_ptr(void) { return (void *)curbuf; }

// --- show_sub FFI accessors ---

/// Save and set p_shm to "F" (disable file info message). Returns strdup of p_shm.
char *nvim_excmds_save_set_shortmess_F(void)
{
  char *saved = xstrdup(p_shm);
  set_option_direct(kOptShortmess, STATIC_CSTR_AS_OPTVAL("F"), 0, SID_NONE);
  return saved;
}
/// Restore p_shm from a previously saved value (and free the saved string).
void nvim_excmds_restore_shortmess(char *saved)
{
  set_option_direct(kOptShortmess, CSTR_AS_OPTVAL(saved), 0, SID_NONE);
  xfree(saved);
}
/// Return first char of p_icm option.
int nvim_excmds_get_p_icm_first(void) { return (unsigned char)p_icm[0]; }
/// Wrapper for buflist_findnr.
buf_T *nvim_excmds_buflist_findnr(int nr) { return buflist_findnr(nr); }
/// Wrapper for buf_ensure_loaded.
void nvim_excmds_buf_ensure_loaded(buf_T *buf) { buf_ensure_loaded(buf); }
/// Wrapper for ml_get_buf.
const char *nvim_excmds_ml_get_buf(buf_T *buf, linenr_T lnum)
{
  return ml_get_buf(buf, lnum);
}
/// Wrapper for ml_get_buf_len.
int nvim_excmds_ml_get_buf_len(buf_T *buf, linenr_T lnum)
{
  return ml_get_buf_len(buf, lnum);
}
/// Wrapper for ml_replace_buf.
void nvim_excmds_ml_replace_buf(buf_T *buf, linenr_T lnum, char *line, bool copy, bool keep_dirty)
{
  ml_replace_buf(buf, lnum, line, copy, keep_dirty);
}
/// Wrapper for ml_append_buf.
void nvim_excmds_ml_append_buf(buf_T *buf, linenr_T lnum, char *line, int len, bool newfile)
{
  ml_append_buf(buf, lnum, line, (colnr_T)len, newfile);
}
/// Wrapper for bufhl_add_hl_pos_offset.
void nvim_excmds_bufhl_add_hl_pos_offset(buf_T *buf, int ns_id, int hl_id,
                                          linenr_T start_lnum, colnr_T start_col,
                                          linenr_T end_lnum, colnr_T end_col,
                                          colnr_T offset)
{
  lpos_T start = { start_lnum, start_col };
  lpos_T end = { end_lnum, end_col };
  bufhl_add_hl_pos_offset(buf, ns_id, hl_id, start, end, offset);
}
/// Wrapper for update_topline(curwin).
void nvim_excmds_update_topline_curwin(void) { update_topline(curwin); }
/// Return b_ml.ml_line_count of orig_buf (curbuf).
int nvim_excmds_orig_buf_line_count(void) { return curbuf->b_ml.ml_line_count; }

/// Accessor: preview_lines->subresults.size
size_t nvim_excmds_preview_lines_size(const void *pl)
{
  return ((const PreviewLines *)pl)->subresults.size;
}
/// Accessor: preview_lines->subresults.items[idx] fields
void nvim_excmds_preview_lines_item(const void *pl, size_t idx,
                                     linenr_T *start_lnum, colnr_T *start_col,
                                     linenr_T *end_lnum, colnr_T *end_col,
                                     linenr_T *pre_match)
{
  SubResult item = ((const PreviewLines *)pl)->subresults.items[idx];
  *start_lnum = item.start.lnum;
  *start_col = item.start.col;
  *end_lnum = item.end.lnum;
  *end_col = item.end.col;
  *pre_match = item.pre_match;
}

// --- ex_global FFI accessors ---

/// Get eap->cmd pointer (first byte determines global type: 'g' or 'v').
const char *nvim_exarg_get_cmd(const exarg_T *eap) { return eap->cmd; }

/// Allocate and call search_regcomp for ex_global. Returns opaque regmmatch_T* or NULL on failure.
/// On success, *used_pat_out is set to the pattern actually used.
void *nvim_excmds_search_regcomp_multi(const char *pat, size_t patlen, const char **used_pat_out,
                                        int which_pat)
{
  regmmatch_T *rm = xmalloc(sizeof(regmmatch_T));
  memset(rm, 0, sizeof(*rm));
  if (search_regcomp((char *)pat, patlen, (char **)used_pat_out, RE_BOTH,
                     which_pat, SEARCH_HIS, rm) == FAIL) {
    xfree(rm);
    return NULL;
  }
  return rm;
}

/// Call vim_regexec_multi on an opaque regmmatch_T handle for the given lnum.
/// Returns match count (> 0 means matched).
int nvim_excmds_vim_regexec_multi(void *regmatch, int lnum)
{
  regmmatch_T *rm = (regmmatch_T *)regmatch;
  return vim_regexec_multi(rm, curwin, curbuf, (linenr_T)lnum, 0, NULL, NULL);
}

/// Free an opaque regmmatch_T handle (calls vim_regfree on regprog, then xfree).
void nvim_excmds_vim_regfree_multi(void *regmatch)
{
  if (regmatch == NULL) { return; }
  regmmatch_T *rm = (regmmatch_T *)regmatch;
  vim_regfree(rm->regprog);
  xfree(rm);
}

/// Check if regmatch->regprog is NULL (re-compile failed mid-loop).
int nvim_excmds_regmmatch_regprog_null(void *regmatch)
{
  regmmatch_T *rm = (regmmatch_T *)regmatch;
  return rm->regprog == NULL ? 1 : 0;
}

/// Wrapper for skip_regexp_ex that also updates eap->arg in place.
/// Returns pointer to the character after the end delimiter (or to NUL).
char *nvim_excmds_skip_regexp_ex_global(exarg_T *eap, char *pat, int delim)
{
  return skip_regexp_ex(pat, (char)delim, rs_magic_isset(), &eap->arg, NULL, NULL);
}

/// Wrapper for ml_setmarked.
void nvim_excmds_ml_setmarked(int lnum) { ml_setmarked((linenr_T)lnum); }

/// Wrapper for ml_clearmarked.
void nvim_excmds_ml_clearmarked(void) { ml_clearmarked(); }

/// Wrapper for line_breakcheck (used in global pass 1 loop).
void nvim_excmds_line_breakcheck(void) { line_breakcheck(); }

/// Emit "Pattern not found: %s" message.
void nvim_excmds_smsg_pattern_not_found(const char *pat)
{
  smsg(0, _("Pattern not found: %s"), pat);
}

/// Emit "Pattern found in every line: %s" message.
void nvim_excmds_smsg_pattern_found_every(const char *pat)
{
  smsg(0, _("Pattern found in every line: %s"), pat);
}

/// Emit E147 error: Cannot do :global recursive with a range.
void nvim_excmds_emsg_e147(void)
{
  emsg(_("E147: Cannot do :global recursive with a range"));
}

/// Emit E148 error: Regular expression missing from global.
void nvim_excmds_emsg_e148(void)
{
  emsg(_("E148: Regular expression missing from global"));
}

/// Emit e_backslash error.
void nvim_excmds_emsg_backslash(void)
{
  emsg(_(e_backslash));
}

/// Emit e_invcmd error.
void nvim_excmds_emsg_invcmd(void)
{
  emsg(_(e_invcmd));
}

/// Emit e_interr error message.
void nvim_excmds_emsg_interr_msg(void)
{
  msg(_(e_interr), 0);
}

/// Get curwin->w_cursor.lnum for nested global handling.
int nvim_excmds_curwin_cursor_lnum(void) { return (int)curwin->w_cursor.lnum; }

/// Set curwin->w_cursor.col to 0 (for nested global).
void nvim_excmds_curwin_set_col_zero(void) { curwin->w_cursor.col = 0; }

// rs_ex_global implemented in Rust (ex_cmds/src/global.rs)
extern void rs_ex_global(exarg_T *eap);

// --- rename_buffer + ex_file FFI accessors ---

/// Save and return curbuf identity (opaque pointer for comparison).
void *nvim_excmds_get_curbuf_identity(void) { return (void *)curbuf; }

/// Apply BufFilePre autocmd on curbuf.
void nvim_excmds_apply_autocmds_buffilepre(void)
{
  apply_autocmds(EVENT_BUFFILEPRE, NULL, NULL, false, curbuf);
}

/// Apply BufFilePost autocmd on curbuf.
void nvim_excmds_apply_autocmds_buffilepost(void)
{
  apply_autocmds(EVENT_BUFFILEPOST, NULL, NULL, false, curbuf);
}

/// Check if curbuf matches a saved identity pointer.
int nvim_excmds_curbuf_is(void *ptr) { return curbuf == (buf_T *)ptr ? 1 : 0; }

/// Wrapper for aborting().
int nvim_excmds_aborting(void) { return aborting() ? 1 : 0; }

/// Get curbuf->b_ffname (full file name).
char *nvim_excmds_curbuf_get_ffname(void) { return curbuf->b_ffname; }

/// Get curbuf->b_sfname (short file name).
char *nvim_excmds_curbuf_get_sfname(void) { return curbuf->b_sfname; }

/// Get curbuf->b_fname (file name).
char *nvim_excmds_curbuf_get_fname(void) { return curbuf->b_fname; }

/// Set curbuf->b_ffname directly (used for save/restore on failure).
void nvim_excmds_curbuf_set_ffname(char *p) { curbuf->b_ffname = p; }

/// Set curbuf->b_sfname directly (used for save/restore on failure).
void nvim_excmds_curbuf_set_sfname(char *p) { curbuf->b_sfname = p; }

/// Set curbuf->b_ffname and b_sfname to NULL (before setfname).
void nvim_excmds_curbuf_clear_filenames(void)
{
  curbuf->b_ffname = NULL;
  curbuf->b_sfname = NULL;
}

/// Call setfname(curbuf, name, NULL, true). Returns OK (1) or FAIL (0).
int nvim_excmds_setfname(const char *name)
{
  return setfname(curbuf, (char *)name, NULL, true) == OK ? 1 : 0;
}

/// Set BF_NOTEDITED flag on curbuf.
void nvim_excmds_curbuf_set_bf_notedited(void)
{
  curbuf->b_flags |= BF_NOTEDITED;
}

/// Call buflist_new(fname, xfname, lnum, 0). Returns opaque buf pointer (may be NULL).
void *nvim_excmds_buflist_new_rename(const char *fname, const char *xfname, int lnum)
{
  return (void *)buflist_new((char *)fname, (char *)xfname, (linenr_T)lnum, 0);
}

/// Get b_fnum from an opaque buf pointer.
int nvim_excmds_buf_get_fnum(void *buf) { return ((buf_T *)buf)->b_fnum; }

/// Check if CMOD_KEEPALT is set.
int nvim_excmds_cmdmod_has_keepalt(void)
{
  return (cmdmod.cmod_flags & CMOD_KEEPALT) != 0 ? 1 : 0;
}

/// Set curwin->w_alt_fnum.
void nvim_excmds_set_curwin_alt_fnum(int fnum) { curwin->w_alt_fnum = fnum; }

/// Wrapper for do_autochdir().
void nvim_excmds_do_autochdir(void) { do_autochdir(); }

/// Get curwin->w_cursor.lnum (for buflist_new lnum argument).
int nvim_excmds_curwin_cursor_lnum_raw(void) { return (int)curwin->w_cursor.lnum; }

/// Set redraw_tabline = true.
void nvim_excmds_set_redraw_tabline(void) { redraw_tabline = true; }

/// Check !shortmess(SHM_FILEINFO): returns 1 if fileinfo should be shown.
int nvim_excmds_shortmess_not_fileinfo(void) { return !shortmess(SHM_FILEINFO) ? 1 : 0; }

/// Wrapper for fileinfo(false, false, forceit).
void nvim_excmds_fileinfo(int forceit) { fileinfo(false, false, (bool)forceit); }

/// Get eap->addr_count.
int nvim_exarg_get_addr_count_val(const exarg_T *eap) { return (int)eap->addr_count; }

/// Get eap->arg (mutable).
char *nvim_exarg_get_arg_mutable(exarg_T *eap) { return eap->arg; }

// --- ex_update, ex_write, ex_wnext FFI accessors ---

/// Check if current buffer has been changed (curbufIsChanged).
int nvim_excmds_curbufIsChanged(void) { return curbufIsChanged() ? 1 : 0; }

/// Check if current buffer has no file name (bt_nofilename(curbuf)).
int nvim_excmds_bt_nofilename_curbuf(void) { return bt_nofilename(curbuf) ? 1 : 0; }

/// Check if curbuf->b_ffname is not NULL.
int nvim_excmds_curbuf_ffname_not_null(void) { return curbuf->b_ffname != NULL ? 1 : 0; }

/// Check if curbuf->b_ffname path exists.
int nvim_excmds_os_path_exists_curbuf_ffname(void)
{
  return curbuf->b_ffname != NULL && os_path_exists(curbuf->b_ffname) ? 1 : 0;
}

/// Wrapper for do_write(eap). Returns OK (1) or FAIL (0).
int nvim_excmds_do_write(exarg_T *eap) { return do_write(eap) == OK ? 1 : 0; }

/// Wrapper for do_bang with the specific args used by ex_write filter.
void nvim_excmds_do_bang_write_filter(exarg_T *eap)
{
  do_bang(1, eap, false, true, false);
}

/// Check if eap->cmdidx == CMD_saveas.
int nvim_exarg_cmdidx_is_saveas(const exarg_T *eap)
{
  return eap->cmdidx == CMD_saveas ? 1 : 0;
}

/// Get eap->usefilter.
int nvim_exarg_get_usefilter(const exarg_T *eap) { return eap->usefilter ? 1 : 0; }

/// Set eap->line1 to a value.
void nvim_exarg_set_line1(exarg_T *eap, int line1) { eap->line1 = (linenr_T)line1; }

/// Get curwin->w_arg_idx.
int nvim_excmds_curwin_get_w_arg_idx(void) { return curwin->w_arg_idx; }

/// Get second byte of eap->cmd (eap->cmd[1]).
int nvim_exarg_get_cmd_byte1(const exarg_T *eap) { return (unsigned char)eap->cmd[1]; }

/// Wrapper for do_argfile(eap, i).
void nvim_excmds_do_argfile(exarg_T *eap, int i) { do_argfile(eap, i); }

// --- sub_get_replacement, sub_set_replacement, free_old_sub, ex_substitute, ex_substitute_preview FFI ---

/// Get old_sub->sub field.
char *nvim_excmds_old_sub_get_sub(void) { return old_sub.sub; }
/// Get old_sub->timestamp field.
uint64_t nvim_excmds_old_sub_get_timestamp(void) { return (uint64_t)old_sub.timestamp; }
/// Get old_sub->additional_data field (opaque).
void *nvim_excmds_old_sub_get_additional_data(void) { return (void *)old_sub.additional_data; }
/// Set old_sub from its three fields, freeing old memory.
void nvim_excmds_old_sub_set(char *sub, uint64_t timestamp, void *additional_data)
{
  xfree(old_sub.sub);
  if ((void *)old_sub.additional_data != additional_data) {
    xfree(old_sub.additional_data);
  }
  old_sub.sub = sub;
  old_sub.timestamp = (Timestamp)timestamp;
  old_sub.additional_data = (AdditionalData *)additional_data;
}

/// Wrapper for do_sub(eap, timeout, ns, bufnr). Returns 0, 1, or 2.
int nvim_excmds_do_sub(exarg_T *eap, int cmdpreview_ns, handle_T cmdpreview_bufnr, int use_rdt)
{
  proftime_T timeout = use_rdt ? profile_setlimit(p_rdt) : profile_zero();
  return do_sub(eap, timeout, cmdpreview_ns, cmdpreview_bufnr);
}

/// Check if preview should proceed: *eap->arg is non-NUL and NOT alphanumeric.
/// Returns 1 if we should call do_sub (valid delimiter found), 0 otherwise.
int nvim_excmds_arg_has_valid_delim(const exarg_T *eap)
{
  return (*eap->arg && !ASCII_ISALNUM(*eap->arg)) ? 1 : 0;
}

/// Save eap->arg and return saved pointer.
char *nvim_excmds_eap_arg_save(exarg_T *eap) { return eap->arg; }

/// Restore eap->arg to a saved pointer.
void nvim_excmds_eap_arg_restore(exarg_T *eap, char *saved) { eap->arg = saved; }

// --- do_filter FFI accessors ---

/// Get p_stmp (shelltemp option).
int nvim_excmds_get_p_stmp(void) { return p_stmp ? 1 : 0; }

/// Get curbuf->b_op_start.lnum
int nvim_excmds_curbuf_op_start_lnum(void) { return (int)curbuf->b_op_start.lnum; }

/// Get curbuf->b_op_end.lnum
int nvim_excmds_curbuf_op_end_lnum(void) { return (int)curbuf->b_op_end.lnum; }

/// Set curbuf->b_op_start.lnum and .col (from curbuf's fields).
void nvim_excmds_curbuf_set_op_start_lnum(int lnum) { curbuf->b_op_start.lnum = (linenr_T)lnum; }
void nvim_excmds_curbuf_set_op_end_lnum(int lnum) { curbuf->b_op_end.lnum = (linenr_T)lnum; }

/// Save entire cursor position (returns lnum, col as a packed 64-bit int: high32=lnum, low32=col).
uint64_t nvim_excmds_curwin_cursor_save(void)
{
  return ((uint64_t)(uint32_t)curwin->w_cursor.lnum << 32)
         | (uint32_t)(int32_t)curwin->w_cursor.col;
}

/// Restore cursor position from packed 64-bit int.
void nvim_excmds_curwin_cursor_restore(uint64_t saved)
{
  curwin->w_cursor.lnum = (linenr_T)(uint32_t)(saved >> 32);
  curwin->w_cursor.col = (colnr_T)(int32_t)(uint32_t)saved;
}

/// Save cmdmod.cmod_flags and clear CMOD_LOCKMARKS.
int nvim_excmds_cmdmod_save_clear_lockmarks(void)
{
  int saved = cmdmod.cmod_flags;
  cmdmod.cmod_flags &= ~CMOD_LOCKMARKS;
  return saved;
}

/// Restore cmdmod.cmod_flags.
void nvim_excmds_cmdmod_restore_flags(int saved) { cmdmod.cmod_flags = saved; }

/// Check if CMOD_LOCKMARKS is currently set.
int nvim_excmds_cmdmod_has_lockmarks(void)
{
  return (cmdmod.cmod_flags & CMOD_LOCKMARKS) != 0 ? 1 : 0;
}

/// Check if CMOD_KEEPMARKS is currently set.
int nvim_excmds_cmdmod_has_keepmarks_now(void)
{
  return (cmdmod.cmod_flags & CMOD_KEEPMARKS) != 0 ? 1 : 0;
}

/// Wrapper for vim_tempname().
char *nvim_excmds_vim_tempname(void) { return vim_tempname(); }

/// Call buf_write for the filter temp file write. Returns 1=OK, 0=FAIL.
int nvim_excmds_buf_write_filter(const char *itmp, int line1, int line2, exarg_T *eap)
{
  return buf_write(curbuf, (char *)itmp, NULL, (linenr_T)line1, (linenr_T)line2,
                   eap, false, false, false, true) == OK ? 1 : 0;
}

/// Increment no_wait_return.
void nvim_excmds_no_wait_return_inc(void) { no_wait_return++; }

/// Decrement no_wait_return.
void nvim_excmds_no_wait_return_dec(void) { no_wait_return--; }

/// Call readfile for filter output. Returns 1=OK, 0=FAIL.
int nvim_excmds_readfile_filter(const char *otmp, int line2, exarg_T *eap)
{
  return readfile((char *)otmp, NULL, (linenr_T)line2, 0, (linenr_T)MAXLNUM,
                  eap, READ_FILTER, false) == OK ? 1 : 0;
}

/// Call call_shell for filter command.
void nvim_excmds_call_shell_filter(const char *cmd, int flags)
{
  call_shell((char *)cmd, flags, NULL);
}

/// Set did_check_timestamps=false, need_check_timestamps=true.
void nvim_excmds_after_shell(void)
{
  did_check_timestamps = false;
  need_check_timestamps = true;
}

/// Set got_int = false.
void nvim_excmds_clear_got_int(void) { got_int = false; }

/// Wrapper for del_lines(count, true).
void nvim_excmds_del_lines(int count) { del_lines((linenr_T)count, true); }

/// Wrapper for write_lnum_adjust.
void nvim_excmds_write_lnum_adjust(int offset) { write_lnum_adjust((linenr_T)offset); }

/// Wrapper for redraw_curbuf_later(UPD_VALID).
void nvim_excmds_redraw_curbuf_later_valid(void) { redraw_curbuf_later(UPD_VALID); }

/// Wrapper for invalidate_botline(curwin).
void nvim_excmds_invalidate_botline(void) { invalidate_botline(curwin); }

/// Get p_report option value.
int nvim_excmds_get_p_report_int(void) { return (int)p_report; }

/// Check vim_strchr(p_cpo, CPO_REMMARK) == NULL (returns 1 if NULL, 0 if found).
int nvim_excmds_p_cpo_no_remmark(void)
{
  return vim_strchr(p_cpo, CPO_REMMARK) == NULL ? 1 : 0;
}

/// Call mark_adjust(line1, line2, amount, 0, kExtmarkNOOP).
void nvim_excmds_mark_adjust_noop(int line1, int line2, int amount)
{
  mark_adjust((linenr_T)line1, (linenr_T)line2, (linenr_T)amount, 0, kExtmarkNOOP);
}

/// Wrapper for rs_foldUpdate(curwin, ...).
extern void rs_foldUpdate(win_T *win, int top, int bot);
void nvim_excmds_fold_update_curwin(int top, int bot)
{
  rs_foldUpdate(curwin, top, bot);
}

/// Format and display "N lines filtered" message via set_keep_msg if scrolled.
void nvim_excmds_msg_lines_filtered(int linecount)
{
  char msg_buf[80];
  vim_snprintf(msg_buf, sizeof(msg_buf),
               _("%" PRId64 " lines filtered"), (int64_t)linecount);
  if (msg(msg_buf, 0) && !msg_scroll) {
    set_keep_msg(msg_buf, 0);
  }
}

/// Emit E482 error: Can't create file.
void nvim_excmds_semsg_e482(const char *fname)
{
  semsg(_("E482: Can't create file %s"), fname);
}

/// Emit e_notread error with filename.
void nvim_excmds_semsg_e_notread(const char *fname)
{
  semsg(_(e_notread), fname);
}

/// Emit e_notmp error.
void nvim_excmds_emsg_e_notmp(void) { emsg(_(e_notmp)); }

/// Emit E135 error.
void nvim_excmds_emsg_e135(void)
{
  emsg(_("E135: *Filter* Autocommands must not change current buffer"));
}

/// Wrapper for wait_return(false).
void nvim_excmds_wait_return_false(void) { wait_return(false); }

/// Get msg_scroll.
int nvim_excmds_get_msg_scroll_val(void) { return msg_scroll ? 1 : 0; }

/// Save curbuf->b_op_start and b_op_end as packed values.
/// Returns two packed uint64 values via out pointers.
void nvim_excmds_curbuf_op_save(uint64_t *out_start, uint64_t *out_end)
{
  *out_start = ((uint64_t)(uint32_t)curbuf->b_op_start.lnum << 32)
               | (uint32_t)(int32_t)curbuf->b_op_start.col;
  *out_end = ((uint64_t)(uint32_t)curbuf->b_op_end.lnum << 32)
             | (uint32_t)(int32_t)curbuf->b_op_end.col;
}

/// Restore curbuf->b_op_start and b_op_end from packed values.
void nvim_excmds_curbuf_op_restore(uint64_t saved_start, uint64_t saved_end)
{
  curbuf->b_op_start.lnum = (linenr_T)(uint32_t)(saved_start >> 32);
  curbuf->b_op_start.col = (colnr_T)(int32_t)(uint32_t)saved_start;
  curbuf->b_op_end.lnum = (linenr_T)(uint32_t)(saved_end >> 32);
  curbuf->b_op_end.col = (colnr_T)(int32_t)(uint32_t)saved_end;
}

/// Adjust curbuf->b_op_start.lnum and b_op_end.lnum by delta.
void nvim_excmds_curbuf_op_adjust_lnum(int delta)
{
  curbuf->b_op_start.lnum += (linenr_T)delta;
  curbuf->b_op_end.lnum += (linenr_T)delta;
}

/// os_remove wrapper.
int nvim_excmds_os_remove(const char *path) { return os_remove(path); }

/// kShellOpt* constants accessor.
int nvim_excmds_kShellOptFilter(void) { return kShellOptFilter; }
int nvim_excmds_kShellOptRead(void) { return kShellOptRead; }
int nvim_excmds_kShellOptWrite(void) { return kShellOptWrite; }
int nvim_excmds_kShellOptDoOut(void) { return kShellOptDoOut; }

/// Get curbuf->b_ml.ml_line_count.
int nvim_excmds_curbuf_ml_line_count(void) { return (int)curbuf->b_ml.ml_line_count; }

// --- Phase 5: getfile, set_swapcommand, delbuf_msg FFI accessors ---

/// GETFILE_* constants.
int nvim_excmds_getfile_error(void) { return GETFILE_ERROR; }
int nvim_excmds_getfile_not_written(void) { return GETFILE_NOT_WRITTEN; }
int nvim_excmds_getfile_same_file(void) { return GETFILE_SAME_FILE; }
int nvim_excmds_getfile_open_other(void) { return GETFILE_OPEN_OTHER; }

/// Wrap check_can_set_curbuf_forceit(forceit). Returns 1 if allowed.
int nvim_excmds_check_can_set_curbuf_forceit(int forceit)
{
  return check_can_set_curbuf_forceit((bool)forceit) ? 1 : 0;
}

/// Wrap text_locked(). Returns 1 if locked.
int nvim_excmds_text_locked(void) { return text_locked() ? 1 : 0; }

/// Wrap curbuf_locked(). Returns 1 if locked.
int nvim_excmds_curbuf_locked(void) { return curbuf_locked() ? 1 : 0; }

/// Expand fname and sfname for curbuf. Both pointers are modified in-place.
/// ffname_ptr and sfname_ptr point to the buffers, fname_expand fills them.
/// Returns the expanded ffname (allocated, caller must free) via *out_ffname.
/// *out_sfname is set to point into the allocated ffname or the original sfname.
void nvim_excmds_fname_expand(char *ffname_in, char *sfname_in,
                              char **out_ffname, char **out_sfname)
{
  *out_ffname = ffname_in;
  *out_sfname = sfname_in;
  fname_expand(curbuf, out_ffname, out_sfname);
}

/// Get curbuf->b_fnum.
int nvim_excmds_curbuf_get_b_fnum(void) { return curbuf->b_fnum; }

/// Get curbuf->b_nwindows.
int nvim_excmds_curbuf_get_b_nwindows(void) { return curbuf->b_nwindows; }

/// Wrap buf_hide(curbuf). Returns 1 if true.
int nvim_excmds_buf_hide_curbuf(void) { return buf_hide(curbuf) ? 1 : 0; }

/// Wrap curbufIsChanged(). Returns 1 if true.
int nvim_excmds_curbufIsChanged_val(void) { return curbufIsChanged() ? 1 : 0; }

/// Wrap autowrite(curbuf, forceit). Returns 1=OK, 0=FAIL.
int nvim_excmds_autowrite_curbuf(int forceit)
{
  return autowrite(curbuf, (bool)forceit) == OK ? 1 : 0;
}

/// Get p_confirm option.
int nvim_excmds_get_p_confirm(void) { return p_confirm ? 1 : 0; }

/// Wrap dialog_changed(curbuf, false).
void nvim_excmds_dialog_changed_curbuf(void) { dialog_changed(curbuf, false); }

/// Wrap no_write_message().
void nvim_excmds_no_write_message(void) { no_write_message(); }

/// Set curwin->w_cursor.lnum.
void nvim_excmds_curwin_set_cursor_lnum(int lnum) { curwin->w_cursor.lnum = (linenr_T)lnum; }

/// Wrap check_cursor_lnum(curwin).
void nvim_excmds_check_cursor_lnum(void) { check_cursor_lnum(curwin); }

/// BL_SOL | BL_FIX constants.
int nvim_excmds_bl_sol_fix(void) { return BL_SOL | BL_FIX; }

/// Wrap do_ecmd(fnum, ffname, sfname, NULL, lnum, flags, curwin). Returns 1=OK, 0=FAIL.
int nvim_excmds_do_ecmd_getfile(int fnum, char *ffname, char *sfname, int lnum, int flags)
{
  return do_ecmd(fnum, ffname, sfname, NULL, (linenr_T)lnum, flags, curwin) == OK ? 1 : 0;
}

/// ECMD_HIDE and ECMD_FORCEIT constants.
int nvim_excmds_ecmd_hide(void) { return ECMD_HIDE; }
int nvim_excmds_ecmd_forceit(void) { return ECMD_FORCEIT; }

/// For set_swapcommand: get_vim_var_str(VV_SWAPCOMMAND). Returns the string (not owned).
const char *nvim_excmds_get_vim_var_str_swapcommand(void)
{
  return get_vim_var_str(VV_SWAPCOMMAND);
}

/// For set_swapcommand: set_vim_var_string(VV_SWAPCOMMAND, p, -1).
void nvim_excmds_set_vim_var_string_swapcommand(const char *p)
{
  set_vim_var_string(VV_SWAPCOMMAND, (char *)p, -1);
}

/// For set_swapcommand: vim_snprintf into allocated buffer.
/// Returns newly allocated string: ":%s\r" if command != NULL, else "NNG\0".
char *nvim_excmds_format_swapcommand(const char *command, int64_t newlnum)
{
  size_t len = (command != NULL) ? strlen(command) + 3 : 30;
  char *p = xmalloc(len);
  if (command != NULL) {
    vim_snprintf(p, len, ":%s\r", command);
  } else {
    vim_snprintf(p, len, "%" PRId64 "G", newlnum);
  }
  return p;
}

/// For delbuf_msg: semsg E143 and clear au_new_curbuf.
void nvim_excmds_semsg_e143_and_clear(const char *name)
{
  semsg(_("E143: Autocommands unexpectedly deleted new buffer %s"),
        name == NULL ? "" : name);
  au_new_curbuf.br_buf = NULL;
  au_new_curbuf.br_buf_free_count = 0;
}

// --- Phase 4: do_wqall FFI accessors ---

/// Get CMD_xall constant.
int nvim_excmds_cmd_xall(void) { return (int)CMD_xall; }

/// Get CMD_wqall constant.
int nvim_excmds_cmd_wqall(void) { return (int)CMD_wqall; }

/// Wrap before_quit_all(eap). Returns 1=OK, 0=FAIL.
int nvim_excmds_before_quit_all(exarg_T *eap) { return before_quit_all(eap) == OK ? 1 : 0; }

/// Set exiting=true.
void nvim_excmds_set_exiting(void) { exiting = true; }

/// Get exiting global.
int nvim_excmds_get_exiting(void) { return exiting ? 1 : 0; }

/// Wrap getout(code). Diverges (process exit).
void nvim_excmds_getout(int code) { getout(code); }

/// Wrap not_exiting().
void nvim_excmds_not_exiting(void) { not_exiting(); }

/// Get firstbuf (the first buffer in the buffer list).
buf_T *nvim_excmds_get_firstbuf(void) { return firstbuf; }

/// Get buf->b_next (next buffer in list, or NULL).
buf_T *nvim_excmds_buf_get_next(const buf_T *buf) { return buf->b_next; }

/// Check if buf has a terminal running (exiting && buf->terminal && channel_job_running).
int nvim_excmds_buf_has_running_job(const buf_T *buf)
{
  return (buf->terminal != NULL && channel_job_running((uint64_t)buf->b_p_channel)) ? 1 : 0;
}

/// Wrap no_write_message_nobang(buf).
void nvim_excmds_no_write_message_nobang(buf_T *buf) { no_write_message_nobang(buf); }

/// Check bufIsChanged(buf). Returns 1 if true.
int nvim_excmds_bufIsChanged(buf_T *buf) { return bufIsChanged(buf) ? 1 : 0; }

/// Check bt_dontwrite(buf). Returns 1 if true.
int nvim_excmds_bt_dontwrite(const buf_T *buf) { return bt_dontwrite(buf) ? 1 : 0; }

/// Get buf->b_fnum.
int nvim_excmds_buf_get_b_fnum(const buf_T *buf) { return buf->b_fnum; }

/// semsg E141: No file name for buffer N.
void nvim_excmds_semsg_e141(int64_t fnum)
{
  semsg(_("E141: No file name for buffer %" PRId64), fnum);
}

/// Wrap check_readonly via rs_ (Rust). Takes buf, uses fake_eap with forceit.
/// Returns 1 if readonly (error), 0 if OK. Updates *forceit_out.
int nvim_excmds_check_readonly_buf(int forceit_in, buf_T *buf, int *forceit_out)
{
  exarg_T fake_eap = { 0 };
  fake_eap.forceit = (bool)forceit_in;
  int result = rs_check_readonly(&fake_eap, buf);
  *forceit_out = (int)fake_eap.forceit;
  return result;
}

/// Wrap check_overwrite for wqall (other=false). Returns 1=OK, 0=FAIL.
int nvim_excmds_check_overwrite_wqall(exarg_T *eap, buf_T *buf)
{
  return check_overwrite(eap, buf, buf->b_fname, buf->b_ffname, false) == OK ? 1 : 0;
}

/// Set a bufref to buf. Opaque handle for buffer reference tracking.
/// Uses a C-allocated bufref_T. Returns opaque pointer to bufref.
void *nvim_excmds_new_bufref(buf_T *buf)
{
  bufref_T *ref = xmalloc(sizeof(bufref_T));
  set_bufref(ref, buf);
  return ref;
}

/// Check if bufref is still valid. Returns 1 if valid.
int nvim_excmds_bufref_valid(void *ref)
{
  return bufref_valid((bufref_T *)ref) ? 1 : 0;
}

/// Free a bufref created with nvim_excmds_new_bufref.
void nvim_excmds_free_bufref(void *ref) { xfree(ref); }

/// Wrap handle_mkdir_p_arg(eap, buf->b_fname). Returns 1=OK, 0=FAIL.
int nvim_excmds_handle_mkdir_p_wqall(exarg_T *eap, buf_T *buf)
{
  return handle_mkdir_p_arg(eap, buf->b_fname) == OK ? 1 : 0;
}

/// Wrap buf_write_all(buf, forceit). Returns 1=OK, 0=FAIL.
int nvim_excmds_buf_write_all(buf_T *buf, int forceit)
{
  return buf_write_all(buf, (bool)forceit) == OK ? 1 : 0;
}

// --- Phase 3: check_overwrite FFI accessors ---

/// Wrap bt_nofilename(buf). Returns 1 if true.
int nvim_excmds_bt_nofilename(const buf_T *buf) { return bt_nofilename((buf_T *)buf) ? 1 : 0; }

/// Get buf->b_flags field.
int nvim_excmds_buf_get_b_flags(const buf_T *buf) { return (int)buf->b_flags; }

/// BF_NOTEDITED constant.
int nvim_excmds_bf_notedited(void) { return BF_NOTEDITED; }
/// BF_NEW constant.
int nvim_excmds_bf_new(void) { return BF_NEW; }
/// BF_READERR constant.
int nvim_excmds_bf_readerr(void) { return BF_READERR; }

/// Check vim_strchr(p_cpo, CPO_OVERNEW) == NULL. Returns 1 if not found.
int nvim_excmds_cpo_no_overnew(void) { return vim_strchr(p_cpo, CPO_OVERNEW) == NULL ? 1 : 0; }

/// Wrap os_path_exists(ffname). Returns 1 if true.
int nvim_excmds_os_path_exists(const char *ffname) { return os_path_exists((char *)ffname) ? 1 : 0; }

/// Wrap os_isdir(ffname). Returns 1 if true.
int nvim_excmds_os_isdir(const char *ffname) { return os_isdir((char *)ffname) ? 1 : 0; }

/// semsg e_isadir2: "%s" is a directory.
void nvim_excmds_semsg_isadir2(const char *ffname) { semsg(_(e_isadir2), ffname); }

/// emsg e_exists: File exists.
void nvim_excmds_emsg_e_exists(void) { emsg(_(e_exists)); }

/// Dialog: "Overwrite existing file "fname"?" Returns 1 if user said yes, sets forceit.
int nvim_excmds_dialog_overwrite(exarg_T *eap, const char *fname)
{
  char buff[DIALOG_MSG_SIZE];
  dialog_msg(buff, _("Overwrite existing file \"%s\"?"), (char *)fname);
  if (vim_dialog_yesno(VIM_QUESTION, NULL, buff, 2) == VIM_YES) {
    eap->forceit = true;
    return 1;
  }
  return 0;
}

/// Get p_dir (directory option string).
const char *nvim_excmds_get_p_dir(void) { return p_dir; }

/// Allocate a copy_option_part result for the first dir entry.
/// Returns newly allocated string (caller must free). Returns "." if p_dir is empty.
char *nvim_excmds_get_first_dir(void)
{
  if (*p_dir == NUL) {
    char *dir = xmalloc(5);
    STRCPY(dir, ".");
    return dir;
  }
  char *dir = xmalloc(MAXPATHL);
  char *p = p_dir;
  copy_option_part(&p, dir, MAXPATHL, ",");
  return dir;
}

/// Wrap makeswapname(fname, ffname, curbuf, dir). Returns allocated string (caller must free).
char *nvim_excmds_makeswapname(const char *fname, const char *ffname, const char *dir)
{
  return makeswapname((char *)fname, (char *)ffname, curbuf, (char *)dir);
}

/// Get emsg_silent. Returns 1 if silent, 0 otherwise.
int nvim_excmds_get_emsg_silent(void) { return emsg_silent ? 1 : 0; }

/// Dialog: "Swap file "sname" exists, overwrite anyway?" Returns 1 if yes, sets forceit.
int nvim_excmds_dialog_swapfile(exarg_T *eap, const char *swapname)
{
  char buff[DIALOG_MSG_SIZE];
  dialog_msg(buff, _("Swap file \"%s\" exists, overwrite anyway?"), (char *)swapname);
  if (vim_dialog_yesno(VIM_QUESTION, NULL, buff, 2) == VIM_YES) {
    eap->forceit = true;
    return 1;
  }
  return 0;
}

/// semsg E768: Swap file exists: %s.
void nvim_excmds_semsg_e768(const char *swapname)
{
  semsg(_("E768: Swap file exists: %s (:silent! overrides)"), swapname);
}

// --- Phase 2: do_write FFI accessors ---

/// Get eap->arg (the argument string, mutable).
char *nvim_excmds_eap_get_arg_rw(exarg_T *eap) { return eap->arg; }

/// Get eap->append.
int nvim_excmds_eap_get_append(const exarg_T *eap) { return eap->append ? 1 : 0; }

/// Get eap->line1.
int nvim_excmds_eap_get_line1(const exarg_T *eap) { return (int)eap->line1; }

/// Get eap->line2.
int nvim_excmds_eap_get_line2_val(const exarg_T *eap) { return (int)eap->line2; }

/// Wrap fix_fname(ffname). Returns allocated string or NULL. Caller must free.
char *nvim_excmds_fix_fname(const char *ffname) { return fix_fname((char *)ffname); }

/// Wrap otherfile(ffname). Returns 1 if it's a different file than current.
int nvim_excmds_otherfile(const char *ffname) { return otherfile((char *)ffname) ? 1 : 0; }

/// Check vim_strchr(p_cpo, CPO_ALTWRITE) != NULL.
int nvim_excmds_vim_strchr_cpo_altwrite(void)
{
  return vim_strchr(p_cpo, CPO_ALTWRITE) != NULL ? 1 : 0;
}

/// Wrap setaltfname(ffname, fname, 1). Returns opaque buf pointer (may be NULL).
void *nvim_excmds_setaltfname(const char *ffname, const char *fname, int lnum)
{
  return setaltfname((char *)ffname, (char *)fname, (linenr_T)lnum);
}

/// Wrap buflist_findname(ffname). Returns opaque buf pointer (may be NULL).
void *nvim_excmds_buflist_findname(const char *ffname)
{
  return buflist_findname((char *)ffname);
}

/// Check buf->b_ml.ml_mfp != NULL (buffer has memfile).
int nvim_excmds_buf_has_mfp(const void *buf)
{
  return ((const buf_T *)buf)->b_ml.ml_mfp != NULL ? 1 : 0;
}

/// emsg(_(e_bufloaded)).
void nvim_excmds_emsg_e_bufloaded(void) { emsg(_(e_bufloaded)); }

/// Wrap bt_dontwrite_msg(curbuf). Returns 1 if true.
int nvim_excmds_bt_dontwrite_msg_curbuf(void) { return bt_dontwrite_msg(curbuf) ? 1 : 0; }

/// Wrap check_fname(). Returns 1=OK, 0=FAIL.
int nvim_excmds_check_fname(void) { return check_fname() == OK ? 1 : 0; }

/// Check curbuf->b_ffname writable (Unix: check_writable).
int nvim_excmds_curbuf_check_writable(void)
{
#ifdef UNIX
  return check_writable(curbuf->b_ffname) == OK ? 1 : 0;
#else
  return 1;
#endif
}

/// Get p_wa option.
int nvim_excmds_get_p_wa(void) { return p_wa ? 1 : 0; }

/// Dialog: "Write partial file?" Returns 1 if user said yes.
int nvim_excmds_dialog_write_partial(void)
{
  return vim_dialog_yesno(VIM_QUESTION, NULL, _("Write partial file?"), 2) == VIM_YES ? 1 : 0;
}

/// emsg E140: Use ! to write partial buffer.
void nvim_excmds_emsg_e140(void)
{
  emsg(_("E140: Use ! to write partial buffer"));
}

/// emsg(_(e_argreq)): Argument required.
void nvim_excmds_emsg_e_argreq(void) { emsg(_(e_argreq)); }

/// Get curbuf->b_ffname (may be NULL).
const char *nvim_excmds_curbuf_get_b_ffname(void) { return curbuf->b_ffname; }

/// Get curbuf->b_fname (may be NULL).
const char *nvim_excmds_curbuf_get_b_fname(void) { return curbuf->b_fname; }

/// Wrap check_overwrite via rs_ (call it directly). Returns 1=OK, 0=FAIL.
int nvim_excmds_check_overwrite(exarg_T *eap, buf_T *buf, const char *fname,
                                const char *ffname, int other)
{
  return check_overwrite(eap, buf, (char *)fname, (char *)ffname, (bool)other) == OK ? 1 : 0;
}

/// Do saveas: apply BufFilePre, swap names, BufFilePost, BufAdd autocmds.
/// Returns 1=OK (write can proceed), 0=FAIL (buffer changed, abort).
/// Updates curbuf->b_sfname; returns the sfname via out_sfname (borrowed from curbuf).
int nvim_excmds_do_saveas_swap(buf_T *alt_buf, const char **out_sfname)
{
  buf_T *was_curbuf = curbuf;

  apply_autocmds(EVENT_BUFFILEPRE, NULL, NULL, false, curbuf);
  apply_autocmds(EVENT_BUFFILEPRE, NULL, NULL, false, alt_buf);
  if (curbuf != was_curbuf || aborting()) {
    return 0;
  }
  // Exchange the file names
  char *tmp;
  tmp = alt_buf->b_fname;
  alt_buf->b_fname = curbuf->b_fname;
  curbuf->b_fname = tmp;
  tmp = alt_buf->b_ffname;
  alt_buf->b_ffname = curbuf->b_ffname;
  curbuf->b_ffname = tmp;
  tmp = alt_buf->b_sfname;
  alt_buf->b_sfname = curbuf->b_sfname;
  curbuf->b_sfname = tmp;
  buf_name_changed(curbuf);
  apply_autocmds(EVENT_BUFFILEPOST, NULL, NULL, false, curbuf);
  apply_autocmds(EVENT_BUFFILEPOST, NULL, NULL, false, alt_buf);
  if (!alt_buf->b_p_bl) {
    alt_buf->b_p_bl = true;
    apply_autocmds(EVENT_BUFADD, NULL, NULL, false, alt_buf);
  }
  if (curbuf != was_curbuf || aborting()) {
    return 0;
  }
  // If 'filetype' was empty try detecting it now.
  if (*curbuf->b_p_ft == NUL) {
    if (augroup_exists("filetypedetect")) {
      do_doautocmd("filetypedetect BufRead", true, NULL);
    }
    do_modelines(0);
  }
  // Autocommands may have changed buffer names.
  *out_sfname = curbuf->b_sfname;
  return 1;
}

/// Wrap buf_write for do_write. Returns 1=OK, 0=FAIL.
int nvim_excmds_buf_write_do_write(const char *ffname, const char *fname,
                                   int line1, int line2,
                                   exarg_T *eap, int append, int forceit)
{
  return buf_write(curbuf, (char *)ffname, (char *)fname,
                   (linenr_T)line1, (linenr_T)line2,
                   eap, (bool)append, (bool)forceit, true, false) == OK ? 1 : 0;
}

/// After saveas: set curbuf->b_p_ro=false, redraw_tabline=true.
void nvim_excmds_saveas_post_success(void)
{
  curbuf->b_p_ro = false;
  redraw_tabline = true;
}

/// Check curbuf->b_ffname == NULL (name was missing before write).
int nvim_excmds_curbuf_ffname_null(void) { return curbuf->b_ffname == NULL ? 1 : 0; }

/// Wrap do_autochdir().
void nvim_excmds_do_autochdir_wrapper(void) { do_autochdir(); }

// --- Phase 1: Write Validation Helpers FFI accessors ---

/// Get p_write option.
int nvim_excmds_get_p_write(void) { return p_write ? 1 : 0; }

/// Wrap os_nodetype(fname). Returns NODE_OTHER constant value for comparison.
int nvim_excmds_os_nodetype(const char *fname) { return (int)os_nodetype(fname); }

/// Return NODE_OTHER constant value.
int nvim_excmds_node_other_val(void) { return (int)NODE_OTHER; }

/// Get eap->mkdir_p field.
int nvim_excmds_eap_get_mkdir_p(const exarg_T *eap) { return eap->mkdir_p ? 1 : 0; }

/// Wrap os_file_mkdir(fname, 0755). Returns 0 on success, negative on error.
int nvim_excmds_os_file_mkdir(const char *fname) { return os_file_mkdir((char *)fname, 0755); }

/// Get buf->b_p_ro (readonly option).
int nvim_excmds_buf_get_b_p_ro(const buf_T *buf) { return buf->b_p_ro ? 1 : 0; }

/// Get buf->b_fname (short file name).
const char *nvim_excmds_buf_get_b_fname(const buf_T *buf) { return buf->b_fname; }

/// Get buf->b_ffname (full file name).
const char *nvim_excmds_buf_get_b_ffname_ptr(const buf_T *buf) { return buf->b_ffname; }

/// Check os_path_exists(buf->b_ffname). Returns 1 if exists.
int nvim_excmds_buf_ffname_path_exists(const buf_T *buf)
{
  return (buf->b_ffname != NULL && os_path_exists(buf->b_ffname)) ? 1 : 0;
}

/// Check os_file_is_writable(buf->b_ffname). Returns 1 if writable.
int nvim_excmds_buf_ffname_is_writable(const buf_T *buf)
{
  return (buf->b_ffname != NULL && os_file_is_writable(buf->b_ffname)) ? 1 : 0;
}

/// Check p_confirm || (cmdmod.cmod_flags & CMOD_CONFIRM).
int nvim_excmds_p_confirm_or_cmod_confirm(void)
{
  return (p_confirm || (cmdmod.cmod_flags & CMOD_CONFIRM)) ? 1 : 0;
}

/// Wrap vim_dialog_yesno(VIM_QUESTION, NULL, msg, 2). Returns VIM_YES (1) or not.
int nvim_excmds_vim_dialog_yesno_question(const char *msg)
{
  return vim_dialog_yesno(VIM_QUESTION, NULL, (char *)msg, 2) == VIM_YES ? 1 : 0;
}

/// Format dialog_msg(buff, fmt, arg) into a newly allocated string. Caller must free.
/// fmt_id: 0 = readonly option, 1 = read-only permissions
char *nvim_excmds_dialog_msg_readonly(int fmt_id, const char *arg)
{
  char *buff = xmalloc(DIALOG_MSG_SIZE);
  if (fmt_id == 0) {
    dialog_msg(buff,
               _("'readonly' option is set for \"%s\".\nDo you wish to write anyway?"),
               (char *)arg);
  } else {
    dialog_msg(buff,
               _("File permissions of \"%s\" are read-only.\nIt may still be possible to "
                 "write it.\nDo you wish to try?"),
               (char *)arg);
  }
  return buff;
}

/// emsg(_(e_readonly)). Returns 1 (true).
int nvim_excmds_emsg_readonly(void) { return emsg(_(e_readonly)); }

/// semsg E505: "%s" is not a file or writable device.
void nvim_excmds_semsg_e503(const char *fname)
{
  semsg(_("E503: \"%s\" is not a file or writable device"), fname);
}

/// semsg E505: "%s" is read-only (add ! to override).
void nvim_excmds_semsg_e505(const char *fname)
{
  semsg(_("E505: \"%s\" is read-only (add ! to override)"), fname);
}

/// emsg E142: File not written: Writing is disabled by 'write' option.
void nvim_excmds_emsg_e142(void)
{
  emsg(_("E142: File not written: Writing is disabled by 'write' option"));
}

/// Set eap->forceit to val.
void nvim_excmds_set_forceit(exarg_T *eap, int val) { eap->forceit = (bool)val; }

/// Get eap->forceit.
int nvim_excmds_eap_get_forceit(const exarg_T *eap) { return eap->forceit ? 1 : 0; }

/// Get mutable pointer to eap->forceit field (for check_readonly pattern).
int *nvim_excmds_eap_forceit_ptr(exarg_T *eap) { return (int *)&eap->forceit; }

/// Set *forceit_ptr to val (used by check_readonly Rust wrapper).
void nvim_excmds_set_forceit_ptr(int *forceit_ptr, int val) { *forceit_ptr = val; }
