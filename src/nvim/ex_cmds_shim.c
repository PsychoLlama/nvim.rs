// ex_cmds.c: some functions for command line commands

#include <assert.h>
#include <ctype.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "auto/config.h"
#include "klib/kvec.h"
#include "nvim/arglist.h"
#include "nvim/assert_defs.h"
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
#include "nvim/digraph.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds2.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_getln.h"
#include "nvim/extmark.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/getchar.h"
#include "nvim/globals.h"
#include "nvim/help.h"
#include "nvim/highlight_group.h"
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
#include "nvim/os/shell.h"
#include "nvim/path.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/profile.h"
#include "nvim/quickfix.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/search.h"
#include "nvim/spell.h"
#include "nvim/state.h"
#include "nvim/strings.h"
#include "nvim/terminal.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

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

// Rust FFI declarations (typval functions migrated to Rust)
extern const char *tv_list_find_str(list_T *l, int n);

// Rust FFI declarations
extern int rs_ml_find_line_or_offset(buf_T *buf, linenr_T lnum, int *offp, bool no_ff);
extern int rs_win_valid(win_T *win);
extern void rs_foldMoveRange(win_T *wp, garray_T *gap, linenr_T line1, linenr_T line2,
                             linenr_T dest);
extern void rs_foldUpdateAll(win_T *win);
extern int rs_magic_isset(void);
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
int nvim_cmdmod_has_keeppatterns(void) { return (cmdmod.cmod_flags & CMOD_KEEPPATTERNS) != 0; }
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

/// Copy oap->start to curbuf->b_op_start (full pos_T, including coladd).
void nvim_curbuf_set_op_start_from_oap_start(void *oap_ptr)
{
  oparg_T *oap = (oparg_T *)oap_ptr;
  curbuf->b_op_start = oap->start;
}

/// Copy oap->start to curbuf->b_op_end (full pos_T, including coladd).
void nvim_curbuf_set_op_end_from_oap_start(void *oap_ptr)
{
  oparg_T *oap = (oparg_T *)oap_ptr;
  curbuf->b_op_end = oap->start;
}

/// Set curbuf->b_op_end for block-wise delete: end.lnum + start.col.
void nvim_curbuf_set_op_end_blockwise(void *oap_ptr)
{
  oparg_T *oap = (oparg_T *)oap_ptr;
  curbuf->b_op_end.lnum = oap->end.lnum;
  curbuf->b_op_end.col = oap->start.col;
}

/// Copy oap->end to curbuf->b_op_end (full pos_T, including coladd).
void nvim_curbuf_set_op_end_from_oap_end(void *oap_ptr)
{
  oparg_T *oap = (oparg_T *)oap_ptr;
  curbuf->b_op_end = oap->end;
}

/// Set curwin->w_cursor = oap->start.
void nvim_curwin_set_cursor_from_oap_start(void *oap_ptr)
{
  oparg_T *oap = (oparg_T *)oap_ptr;
  curwin->w_cursor = oap->start;
}

void nvim_msg_multiline_cstr(const char *s, int hl_id, bool check_int, bool hist, bool *need_clear)
{
  msg_multiline(cstr_as_string(s), hl_id, check_int, hist, need_clear);
}

// print_line accessors
int nvim_curwin_get_w_p_nu(void) { return curwin->w_p_nu; }
int nvim_number_width_curwin(void) { return number_width(curwin); }
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

// Error message wrappers (can't call variadic semsg from Rust)
void nvim_excmds_semsg_invarg2(const char *p) { semsg(_(e_invarg2), p); }
void nvim_excmds_emsg_invarg(void) { emsg(_(e_invarg)); }
void nvim_excmds_emsg_noprevre(void) { emsg(_(e_noprevre)); }
void nvim_excmds_emsg_interr(void) { emsg(_(e_interr)); }

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

// Verify do_filter/write constants used as Rust compile-time values.
_Static_assert(kShellOptFilter == 1, "kShellOptFilter mismatch - update Rust constant");
_Static_assert(kShellOptRead == 16, "kShellOptRead mismatch - update Rust constant");
_Static_assert(kShellOptWrite == 32, "kShellOptWrite mismatch - update Rust constant");
_Static_assert(kShellOptDoOut == 4, "kShellOptDoOut mismatch - update Rust constant");
_Static_assert(GETFILE_ERROR == 1, "GETFILE_ERROR mismatch - update Rust constant");
_Static_assert(GETFILE_NOT_WRITTEN == 2, "GETFILE_NOT_WRITTEN mismatch - update Rust constant");
_Static_assert(GETFILE_SAME_FILE == 0, "GETFILE_SAME_FILE mismatch - update Rust constant");
_Static_assert(GETFILE_OPEN_OTHER == -1, "GETFILE_OPEN_OTHER mismatch - update Rust constant");
_Static_assert(BF_NOTEDITED == 0x08, "BF_NOTEDITED mismatch - update Rust constant");
_Static_assert(BF_NEW == 0x10, "BF_NEW mismatch - update Rust constant");
_Static_assert(BF_READERR == 0x40, "BF_READERR mismatch - update Rust constant");
_Static_assert(NODE_OTHER == 2, "NODE_OTHER mismatch - update Rust constant");

// Verify do_sub constants used as Rust compile-time values.
_Static_assert(CMD_tilde == 554, "CMD_tilde mismatch - update Rust constant");
_Static_assert(MAXCOL == 0x7fffffff, "MAXCOL mismatch - update Rust constant");
_Static_assert(MAXLNUM == 0x7fffffff, "MAXLNUM mismatch - update Rust constant");
_Static_assert(RE_SEARCH == 0, "RE_SEARCH mismatch - update Rust constant");
_Static_assert(RE_SUBST == 1, "RE_SUBST mismatch - update Rust constant");
_Static_assert(RE_LAST == 2, "RE_LAST mismatch - update Rust constant");
_Static_assert(SEARCH_HIS == 0x20, "SEARCH_HIS mismatch - update Rust constant");
_Static_assert(REGSUB_COPY == 1, "REGSUB_COPY mismatch - update Rust constant");
_Static_assert(REGSUB_MAGIC == 2, "REGSUB_MAGIC mismatch - update Rust constant");
_Static_assert(REGSUB_BACKSLASH == 4, "REGSUB_BACKSLASH mismatch - update Rust constant");

extern int rs_not_writing(void);
extern int rs_check_writable(const char *fname);
extern int rs_handle_mkdir_p_arg(exarg_T *eap, const char *fname);
extern int rs_check_readonly(exarg_T *eap, buf_T *buf);
extern void rs_delbuf_msg(char *name);

int append_indent = 0;              // autoindent for first line

// ex_append has been migrated to Rust (rs_ex_append in lines.rs)

/// Previous substitute replacement string
static SubReplacementString old_sub = { NULL, 0, NULL };

int global_need_beginline = 0;           // call beginline() after ":g"

// sub_get_replacement, sub_set_replacement, free_old_sub, ex_substitute, ex_substitute_preview
// implemented in Rust (ex_cmds/src/substitute.rs).
extern void rs_sub_get_replacement(void *ret_sub);
extern void rs_sub_set_replacement(char *sub, uint64_t timestamp, void *additional_data);


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

/// Accessor: return messaging() result.
int nvim_excmds_messaging(void) { return messaging() ? 1 : 0; }


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

void nvim_excmds_reset_binding_curwin(void)
{
  RESET_BINDING(curwin);
}

void nvim_excmds_set_foldcolumn_zero(void)
{
  set_option_direct(kOptFoldcolumn, STATIC_CSTR_AS_OPTVAL("0"), 0, SID_NONE);
}

// --- ex_oldfiles FFI accessors ---

int nvim_excmds_oldfiles_count(void)
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

void nvim_excmds_msg_outtrans(const char *s) { msg_outtrans((char *)s, 0, false); }
int nvim_excmds_cmdmod_has_browse(void) { return (cmdmod.cmod_flags & CMOD_BROWSE) != 0; }
int nvim_excmds_prompt_for_input(void) { return prompt_for_input(NULL, 0, false, NULL); }
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

// --- do_bang FFI accessors ---

void nvim_excmds_autowrite_all(void) { autowrite_all(); }
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
void nvim_excmds_apply_autocmds_shellfilterpost(void)
{
  apply_autocmds(EVENT_SHELLFILTERPOST, NULL, NULL, false, curbuf);
}

// --- do_shell FFI accessors ---
int nvim_excmds_any_buf_changed(void)
{
  FOR_ALL_BUFFERS(buf) {
    if (bufIsChanged(buf)) {
      return 1;
    }
  }
  return 0;
}
void nvim_excmds_call_shell(char *cmd, int flags)
{
  call_shell(cmd, flags, NULL);
}
void nvim_excmds_apply_autocmds_shellcmdpost(void)
{
  apply_autocmds(EVENT_SHELLCMDPOST, NULL, NULL, false, curbuf);
}

// --- global_exe FFI accessors ---
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

/// Dispatch no-arg error/message by id.
/// IDs: 1=e_noprev, 2=E147, 3=E148, 4=e_backslash, 5=e_invcmd, 6=e_interr(msg),
///      7=e_zerocount, 8=msg_empty, 9=msg_no_old_files, 10=msg_ext_set_kind_shell_cmd,
///      11=msg_puts_no_write_warning
void nvim_excmds_emsg_by_id(int id)
{
  switch (id) {
  case 1: emsg(_(e_noprev)); break;
  case 2: emsg(_("E147: Cannot do :global recursive with a range")); break;
  case 3: emsg(_("E148: Regular expression missing from global")); break;
  case 4: emsg(_(e_backslash)); break;
  case 5: emsg(_(e_invcmd)); break;
  case 6: msg(_(e_interr), 0); break;
  case 7: emsg(_(e_zerocount)); break;
  case 8: msg("", 0); break;
  case 9: msg(_("No old files"), 0); break;
  case 10: msg_ext_set_kind("shell_cmd"); break;
  case 11: msg_puts(_("[No write since last change]\n")); break;
  }
}

/// Dispatch message-with-string-arg by id.
/// IDs: 1=smsg_pattern_not_found, 2=smsg_pattern_found_every, 3=semsg_patnotf2,
///      4=semsg_trailing, 5=semsg_val_too_large
void nvim_excmds_emsg_with_arg(int id, const char *arg)
{
  switch (id) {
  case 1: smsg(0, _("Pattern not found: %s"), arg); break;
  case 2: smsg(0, _("Pattern found in every line: %s"), arg); break;
  case 3: semsg(_(e_patnotf2), arg); break;
  case 4: semsg(_(e_trailing_arg), arg); break;
  case 5: semsg(_(e_val_too_large), arg); break;
  }
}

/// Wrap syn_check_group("Substitute"). Returns hl_id.
int nvim_excmds_syn_check_sub_group(void)
{
  return syn_check_group(S_LEN("Substitute"));
}

/// Disable inccommand option.
void nvim_excmds_disable_inccommand(void)
{
  set_option_direct(kOptInccommand, STATIC_CSTR_AS_OPTVAL(""), 0, SID_NONE);
}

/// Get curwin->w_cursor.lnum for nested global handling.
int nvim_excmds_curwin_cursor_lnum(void) { return (int)curwin->w_cursor.lnum; }

/// Set curwin->w_cursor.col to 0 (for nested global).
void nvim_excmds_curwin_set_col_zero(void) { curwin->w_cursor.col = 0; }

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

/// Check !shortmess(SHM_FILEINFO): returns 1 if fileinfo should be shown.
int nvim_excmds_shortmess_not_fileinfo(void) { return !shortmess(SHM_FILEINFO) ? 1 : 0; }

/// Wrapper for fileinfo(false, false, forceit).
void nvim_excmds_fileinfo(int forceit) { fileinfo(false, false, (bool)forceit); }

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

/// Check if preview should proceed: *eap->arg is non-NUL and NOT alphanumeric.
/// Returns 1 if we should call do_sub (valid delimiter found), 0 otherwise.
int nvim_excmds_arg_has_valid_delim(const exarg_T *eap)
{
  return (*eap->arg && !ASCII_ISALNUM(*eap->arg)) ? 1 : 0;
}

/// Restore eap->arg to a saved pointer.
void nvim_excmds_eap_arg_restore(exarg_T *eap, char *saved) { eap->arg = saved; }

// --- do_filter FFI accessors ---

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

/// Wrapper for del_lines(count, true).
void nvim_excmds_del_lines(int count) { del_lines((linenr_T)count, true); }

/// Wrapper for write_lnum_adjust.
void nvim_excmds_write_lnum_adjust(int offset) { write_lnum_adjust((linenr_T)offset); }

/// Wrapper for redraw_curbuf_later(UPD_VALID).
void nvim_excmds_redraw_curbuf_later_valid(void) { redraw_curbuf_later(UPD_VALID); }

/// Wrapper for invalidate_botline(curwin).
void nvim_excmds_invalidate_botline(void) { invalidate_botline(curwin); }

/// Check vim_strchr(p_cpo, CPO_REMMARK) == NULL (returns 1 if NULL, 0 if found).
int nvim_excmds_p_cpo_no_remmark(void)
{
  return vim_strchr(p_cpo, CPO_REMMARK) == NULL ? 1 : 0;
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

/// Unified error message dispatcher for do_filter/write/check_overwrite error paths.
/// error_id values are defined as Rust constants (see shell.rs / write.rs).
void nvim_excmds_error_msg(int error_id, const char *arg)
{
  switch (error_id) {
  case 1:   semsg(_("E482: Can't create file %s"), arg); break;
  case 2:   semsg(_(e_notread), arg); break;
  case 3:   emsg(_(e_notmp)); break;
  case 4:   emsg(_("E135: *Filter* Autocommands must not change current buffer")); break;
  case 5:   emsg(_("E142: File not written: Writing is disabled by 'write' option")); break;
  case 6:   semsg(_("E503: \"%s\" is not a file or writable device"), arg); break;
  case 7:   semsg(_("E505: \"%s\" is read-only (add ! to override)"), arg); break;
  case 8:   emsg(_(e_readonly)); break;
  case 9:   semsg(_(e_isadir2), arg); break;
  case 10:  emsg(_(e_exists)); break;
  case 11:  semsg(_("E768: Swap file exists: %s (:silent! overrides)"), arg); break;
  case 12:  emsg(_("E140: Use ! to write partial buffer")); break;
  case 13:  emsg(_(e_argreq)); break;
  case 14:
    semsg(_("E143: Autocommands unexpectedly deleted new buffer %s"),
          arg == NULL ? "" : arg);
    au_new_curbuf.br_buf = NULL;
    au_new_curbuf.br_buf_free_count = 0;
    break;
  default:  break;
  }
}

/// Wrapper for wait_return(false).
void nvim_excmds_wait_return_false(void) { wait_return(false); }

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

/// Get curbuf->b_ml.ml_line_count.
int nvim_excmds_curbuf_ml_line_count(void) { return (int)curbuf->b_ml.ml_line_count; }

// --- getfile, set_swapcommand, delbuf_msg FFI accessors ---

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

/// Wrap autowrite(curbuf, forceit). Returns 1=OK, 0=FAIL.
int nvim_excmds_autowrite_curbuf(int forceit)
{
  return autowrite(curbuf, (bool)forceit) == OK ? 1 : 0;
}

/// Wrap dialog_changed(curbuf, false).
void nvim_excmds_dialog_changed_curbuf(void) { dialog_changed(curbuf, false); }

/// Wrap no_write_message().
void nvim_excmds_no_write_message(void) { no_write_message(); }


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

// --- do_wqall FFI accessors ---

/// Get CMD_xall constant.
int nvim_excmds_cmd_xall(void) { return (int)CMD_xall; }

/// Get CMD_wqall constant.
int nvim_excmds_cmd_wqall(void) { return (int)CMD_wqall; }

/// Wrap before_quit_all(eap). Returns 1=OK, 0=FAIL.
int nvim_excmds_before_quit_all(exarg_T *eap) { return before_quit_all(eap) == OK ? 1 : 0; }

/// Wrap getout(code). Diverges (process exit).
void nvim_excmds_getout(int code) { getout(code); }

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



/// Wrap buf_write_all(buf, forceit). Returns 1=OK, 0=FAIL.
int nvim_excmds_buf_write_all(buf_T *buf, int forceit)
{
  return buf_write_all(buf, (bool)forceit) == OK ? 1 : 0;
}

// --- check_overwrite FFI accessors ---

/// Wrap bt_nofilename(buf). Returns 1 if true.
int nvim_excmds_bt_nofilename(const buf_T *buf) { return bt_nofilename((buf_T *)buf) ? 1 : 0; }

/// Get buf->b_flags field.
int nvim_excmds_buf_get_b_flags(const buf_T *buf) { return (int)buf->b_flags; }

/// Check vim_strchr(p_cpo, CPO_OVERNEW) == NULL. Returns 1 if not found.
int nvim_excmds_cpo_no_overnew(void) { return vim_strchr(p_cpo, CPO_OVERNEW) == NULL ? 1 : 0; }

/// Wrap os_path_exists(ffname). Returns 1 if true.
int nvim_excmds_os_path_exists(const char *ffname) { return os_path_exists((char *)ffname) ? 1 : 0; }

/// Wrap os_isdir(ffname). Returns 1 if true.
int nvim_excmds_os_isdir(const char *ffname) { return os_isdir((char *)ffname) ? 1 : 0; }

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

// --- do_write FFI accessors ---

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

/// Check curbuf->b_ffname writable (Unix: rs_check_writable).
int nvim_excmds_curbuf_check_writable(void)
{
#ifdef UNIX
  return rs_check_writable(curbuf->b_ffname);
#else
  return 1;
#endif
}

/// Dialog: "Write partial file?" Returns 1 if user said yes.
int nvim_excmds_dialog_write_partial(void)
{
  return vim_dialog_yesno(VIM_QUESTION, NULL, _("Write partial file?"), 2) == VIM_YES ? 1 : 0;
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

// --- Write Validation Helpers FFI accessors ---

/// Wrap os_nodetype(fname). Returns NODE_OTHER constant value for comparison.
int nvim_excmds_os_nodetype(const char *fname) { return (int)os_nodetype(fname); }

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

/// Set eap->forceit to val.
void nvim_excmds_set_forceit(exarg_T *eap, int val) { eap->forceit = (bool)val; }

/// Get curwin->w_botline.
int nvim_curwin_get_w_botline(void) { return (int)curwin->w_botline; }

/// Get curwin->w_p_crb (cursor bind).
int nvim_curwin_get_w_p_crb(void) { return curwin->w_p_crb ? 1 : 0; }

/// Get curwin->w_p_fen (folding enable).
int nvim_curwin_get_w_p_fen(void) { return curwin->w_p_fen ? 1 : 0; }
/// Set curwin->w_p_fen.
void nvim_curwin_set_w_p_fen(int val) { curwin->w_p_fen = (bool)val; }

/// Set curbuf->deleted_bytes2.
void nvim_curbuf_set_deleted_bytes2(int val) { curbuf->deleted_bytes2 = (bcount_t)val; }

/// Wrap getdigits_int(&pp, true, INT_MAX). Returns int and advances *pp.
int nvim_do_sub_getdigits_int(char **pp)
{
  return getdigits_int(pp, true, INT_MAX);
}

/// Wrap skip_regexp_ex for do_sub. Updates *arg_ptr in place.
char *nvim_do_sub_skip_regexp_ex(char *cmd, int delim, char **arg_ptr)
{
  return skip_regexp_ex(cmd, (char)delim, rs_magic_isset(), arg_ptr, NULL, NULL);
}

/// Wrap changed_window_setting(curwin).
void nvim_do_sub_changed_window_setting(void) { changed_window_setting(curwin); }

/// Wrap getvcol for both start and end columns.
void nvim_do_sub_getvcol_start_end(int lnum, int start_col, int end_col,
                                    int *sc_out, int *ec_out)
{
  pos_T pos = { (linenr_T)lnum, (colnr_T)start_col, 0 };
  colnr_T sc = 0;
  getvcol(curwin, &pos, &sc, NULL, NULL);
  *sc_out = (int)sc;
  pos.col = (colnr_T)end_col;
  colnr_T ec = 0;
  getvcol(curwin, &pos, NULL, NULL, &ec);
  *ec_out = (int)ec;
}

/// Wrap getcmdline_prompt for exmode substitution confirm.
/// Returns the first character typed (or NUL).
int nvim_do_sub_getcmdline_prompt(const char *prompt_str)
{
  char *resp = getcmdline_prompt(-1, (char *)prompt_str, 0, EXPAND_NOTHING, NULL,
                                  CALLBACK_NONE, false, NULL);
  msg_putchar('\n');
  int typed = NUL;
  if (resp != NULL) {
    typed = (uint8_t)(*resp);
    xfree(resp);
  }
  return typed;
}

/// Wrap prompt_for_input(str, HLF_R, true, NULL). Returns typed character.
int nvim_do_sub_prompt_for_input(const char *str)
{
  return prompt_for_input((char *)str, HLF_R, true, NULL);
}

/// Call update_topline, validate_cursor, redraw and update_screen for confirm.
void nvim_do_sub_update_screen_for_confirm(void)
{
  update_topline(curwin);
  validate_cursor(curwin);
  redraw_later(curwin, UPD_SOME_VALID);
  show_cursor_info_later(true);
  update_screen();
  redraw_later(curwin, UPD_SOME_VALID);
}

/// Set curbuf->b_op_start.lnum and b_op_end.lnum (both col = 0).
void nvim_do_sub_set_op_start_end(int start_lnum, int end_lnum)
{
  curbuf->b_op_start.lnum = (linenr_T)start_lnum;
  curbuf->b_op_start.col = 0;
  curbuf->b_op_end.lnum = (linenr_T)end_lnum;
  curbuf->b_op_end.col = 0;
}

/// Format the confirm prompt string into IObuff and return xstrdup of it.
char *nvim_do_sub_format_confirm_prompt(const char *sub)
{
  const char *p = _("replace with %s? (y)es/(n)o/(a)ll/(q)uit/(l)ast/scroll up(^E)/down(^Y)");
  vim_snprintf(IObuff, IOSIZE, p, sub);
  return xstrdup(IObuff);
}

/// Wrap changed_lines(curbuf, first, 0, last, xtra, false) for do_sub.
void nvim_do_sub_changed_lines(int first, int last, int xtra)
{
  changed_lines(curbuf, (linenr_T)first, 0, (linenr_T)last, (linenr_T)xtra, false);
}

/// Save substitute pattern and history via save_re_pat + add_to_history.
void nvim_do_sub_save_pat(const char *pat, size_t patlen, int which_pat)
{
  save_re_pat(which_pat, (char *)pat, patlen, rs_magic_isset());
  add_to_history(HIST_SEARCH, (char *)pat, patlen, true, NUL);
}

/// Wrap sub_set_replacement for do_sub (xstrdup of sub, os_time, NULL additional).
void nvim_do_sub_set_replacement(const char *sub)
{
  sub_set_replacement((SubReplacementString) {
    .sub = xstrdup(sub),
    .timestamp = os_time(),
    .additional_data = NULL,
  });
}

// =============================================================================
// regmmatch_T opaque handle accessors for do_sub
// =============================================================================

/// Get regmatch->startpos[0].lnum
int nvim_regmmatch_startpos0_lnum(void *rm)
{
  return (int)((regmmatch_T *)rm)->startpos[0].lnum;
}

/// Get regmatch->startpos[0].col
int nvim_regmmatch_startpos0_col(void *rm)
{
  return (int)((regmmatch_T *)rm)->startpos[0].col;
}

/// Get regmatch->endpos[0].lnum
int nvim_regmmatch_endpos0_lnum(void *rm)
{
  return (int)((regmmatch_T *)rm)->endpos[0].lnum;
}

/// Get regmatch->endpos[0].col
int nvim_regmmatch_endpos0_col(void *rm)
{
  return (int)((regmmatch_T *)rm)->endpos[0].col;
}

/// Set regmatch->rmm_ic
void nvim_regmmatch_set_rmm_ic(void *rm, int ic)
{
  ((regmmatch_T *)rm)->rmm_ic = (bool)ic;
}

/// Get regmatch->rmm_ic
int nvim_regmmatch_get_rmm_ic(void *rm)
{
  return ((regmmatch_T *)rm)->rmm_ic ? 1 : 0;
}

/// Call re_multiline(regmatch->regprog)
int nvim_regmmatch_re_multiline(void *rm)
{
  return re_multiline(((regmmatch_T *)rm)->regprog) ? 1 : 0;
}

/// Wrap search_regcomp for do_sub, allocating and returning opaque regmmatch_T*.
/// Returns NULL on failure. flags=0 normally, flags=SEARCH_HIS to save history.
void *nvim_do_sub_search_regcomp(const char *pat, size_t patlen, int which_pat, int flags)
{
  regmmatch_T *rm = xmalloc(sizeof(regmmatch_T));
  memset(rm, 0, sizeof(*rm));
  if (search_regcomp((char *)pat, patlen, NULL, RE_SUBST, which_pat, flags, rm) == FAIL) {
    xfree(rm);
    return NULL;
  }
  return rm;
}

/// Wrap vim_regexec_multi for do_sub.
/// Returns match count (>0 means matched).
int nvim_do_sub_vim_regexec_multi(void *rm, int lnum, int col)
{
  return vim_regexec_multi((regmmatch_T *)rm, curwin, curbuf, (linenr_T)lnum, (colnr_T)col,
                           NULL, NULL);
}

/// Wrap vim_regsub_multi for do_sub.
/// source_lnum is sub_firstlnum - regmatch.startpos[0].lnum.
/// sub is the replacement string (source arg to vim_regsub_multi).
/// dest is the output buffer, destlen is its size.
/// Returns sublen (length including NUL), or 0 on error.
int nvim_do_sub_vim_regsub_multi(void *rm, int source_lnum, const char *sub,
                                  char *dest, int destlen, int flags)
{
  return vim_regsub_multi((regmmatch_T *)rm, (linenr_T)source_lnum,
                          (char *)sub, dest, destlen, flags);
}

/// Wrap regtilde for do_sub.
/// Returns new string (may be same as sub if no tilde, or newly allocated).
/// The caller is responsible for freeing if the pointer changed.
char *nvim_do_sub_regtilde(char *sub, int magic, int preview)
{
  return regtilde(sub, magic, preview != 0);
}

// =============================================================================
// do_ecmd accessor infrastructure
// These accessors are used by rs_do_ecmd in src/nvim-rs/ex_cmds/src/edit.rs
// =============================================================================

// --- curbuf field accessors ---

/// Get curbuf->b_flags
int nvim_ecmd_curbuf_get_b_flags(void) { return curbuf->b_flags; }

/// Returns 1 if curbuf->terminal != NULL
int nvim_ecmd_curbuf_get_terminal(void) { return curbuf->terminal != NULL ? 1 : 0; }

/// Set curbuf->b_did_filetype
void nvim_ecmd_curbuf_set_did_filetype(int val) { curbuf->b_did_filetype = (bool)val; }

/// Clear bits in curbuf->b_flags
void nvim_ecmd_curbuf_clear_flags(int mask) { curbuf->b_flags &= ~mask; }

/// Set bits in curbuf->b_flags
void nvim_ecmd_curbuf_set_flags(int mask) { curbuf->b_flags |= mask; }

/// Set curbuf->b_last_used to time(NULL)
void nvim_ecmd_curbuf_set_last_used(void) { curbuf->b_last_used = time(NULL); }

/// Get curbuf->b_kmap_state
int nvim_ecmd_curbuf_get_kmap_state(void) { return curbuf->b_kmap_state; }

/// Get curbuf->b_help
int nvim_ecmd_curbuf_get_help(void) { return curbuf->b_help ? 1 : 0; }

/// Set curbuf->b_op_start.lnum and col to 0
void nvim_ecmd_curbuf_clear_op_marks(void)
{
  curbuf->b_op_start.lnum = 0;
  curbuf->b_op_end.lnum = 0;
}

// --- curwin field accessors ---

/// Get curwin->w_cursor.lnum and col packed as two ints via out pointers
void nvim_ecmd_curwin_get_cursor(int *lnum_out, int *col_out)
{
  *lnum_out = (int)curwin->w_cursor.lnum;
  *col_out = (int)curwin->w_cursor.col;
}

/// Set curwin->w_cursor.lnum and col
void nvim_ecmd_curwin_set_cursor(int lnum, int col)
{
  curwin->w_cursor.lnum = (linenr_T)lnum;
  curwin->w_cursor.col = (colnr_T)col;
}

/// Get curwin->w_cursor.col
int nvim_ecmd_curwin_get_cursor_col(void) { return (int)curwin->w_cursor.col; }

/// Set curwin->w_cursor.coladd = 0 and curwin->w_set_curswant = true (for 'sol' off path)
void nvim_ecmd_curwin_set_coladd_curswant(void)
{
  curwin->w_cursor.coladd = 0;
  curwin->w_set_curswant = true;
}

/// Get curwin->w_topline
int nvim_ecmd_curwin_get_topline(void) { return (int)curwin->w_topline; }

/// Get curwin->w_alt_fnum
int nvim_ecmd_curwin_get_alt_fnum(void) { return curwin->w_alt_fnum; }

/// Set curwin->w_pcmark.lnum and col
void nvim_ecmd_curwin_set_pcmark(int lnum, int col)
{
  curwin->w_pcmark.lnum = (linenr_T)lnum;
  curwin->w_pcmark.col = (colnr_T)col;
}

/// Get effective scroll offset: curwin->w_p_so if >= 0, else p_so
int nvim_ecmd_curwin_get_effective_p_so(void)
{
  return (int)(curwin->w_p_so >= 0 ? curwin->w_p_so : p_so);
}

/// Set effective scroll offset (only if curwin->w_p_so >= 0, else set p_so)
void nvim_ecmd_curwin_set_effective_p_so(int val)
{
  if (curwin->w_p_so >= 0) {
    curwin->w_p_so = val;
  } else {
    p_so = val;
  }
}

/// Get diff/spell state: fills *diff_out, *spell_out, *spl_empty_out
void nvim_ecmd_curwin_diff_spell_state(int *diff_out, int *spell_out, int *spl_empty_out)
{
  *diff_out = curwin->w_p_diff ? 1 : 0;
  *spell_out = curwin->w_p_spell ? 1 : 0;
  *spl_empty_out = *curwin->w_s->b_p_spl == NUL ? 1 : 0;
}

/// Set curwin->w_scbind_pos to plines_m_win_fill(curwin, 1, topline)
void nvim_ecmd_curwin_set_scbind_pos_from_topline(void)
{
  curwin->w_scbind_pos = plines_m_win_fill(curwin, 1, curwin->w_topline);
}

/// Get curwin->w_buffer == NULL
int nvim_ecmd_curwin_buf_is_null(void) { return curwin->w_buffer == NULL ? 1 : 0; }

/// Get curwin->w_s == &curwin->w_buffer->b_s
int nvim_ecmd_curwin_ws_is_own_buf(void)
{
  return curwin->w_s == &(curwin->w_buffer->b_s) ? 1 : 0;
}

// --- buf_T opaque handle accessors ---

/// Get buf->b_ml.ml_mfp != NULL
int nvim_ecmd_buf_has_memfile(buf_T *buf) { return buf->b_ml.ml_mfp != NULL ? 1 : 0; }

/// Get buf->b_locked_split
int nvim_ecmd_buf_get_locked_split(buf_T *buf) { return buf->b_locked_split; }

/// Increment buf->b_nwindows
void nvim_ecmd_buf_inc_nwindows(buf_T *buf) { buf->b_nwindows++; }

/// Increment buf->b_locked
void nvim_ecmd_buf_inc_locked(buf_T *buf) { buf->b_locked++; }

/// Decrement buf->b_locked
void nvim_ecmd_buf_dec_locked(buf_T *buf) { buf->b_locked--; }

/// Returns 1 if buf == curbuf
int nvim_ecmd_buf_is_curbuf(buf_T *buf) { return buf == curbuf ? 1 : 0; }

/// Set curbuf = buf and buf->b_nwindows++ and curwin->w_buffer = buf
void nvim_ecmd_set_curbuf(buf_T *buf)
{
  curwin->w_buffer = buf;
  curbuf = buf;
  curbuf->b_nwindows++;
}

// --- win_T opaque handle accessors for oldwin ---

/// Get win->w_buffer == NULL
int nvim_ecmd_win_buf_is_null(win_T *win) { return win->w_buffer == NULL ? 1 : 0; }

/// Set win->w_buffer to was_curbuf (restore after close_buffer)
void nvim_ecmd_win_restore_buffer(win_T *win, buf_T *buf) { win->w_buffer = buf; }

/// Set win->w_locked
void nvim_ecmd_win_set_locked(win_T *win, int val) { win->w_locked = (bool)val; }

// --- bufref_T opaque handle accessors (heap-allocated, exposed as void*) ---
// bufref_T is not in the auto-generated header, so all public APIs use void*.

/// Allocate a new bufref_T on the heap. Must be freed with nvim_ecmd_free_bufref.
void *nvim_ecmd_new_bufref(void) { return xcalloc(1, sizeof(bufref_T)); }

/// Call set_bufref(ref, curbuf)
void nvim_ecmd_set_bufref_to_curbuf(void *ref) { set_bufref((bufref_T *)ref, curbuf); }

/// Call set_bufref(ref, buf)
void nvim_ecmd_set_bufref_to_buf(void *ref, buf_T *buf) { set_bufref((bufref_T *)ref, buf); }

/// Returns 1 if bufref_valid(ref)
int nvim_ecmd_bufref_valid(void *ref) { return bufref_valid((bufref_T *)ref) ? 1 : 0; }

/// Returns 1 if ref->br_buf == curbuf
int nvim_ecmd_bufref_is_curbuf(void *ref) { return ((bufref_T *)ref)->br_buf == curbuf ? 1 : 0; }

// --- au_new_curbuf global accessors ---

/// Set au_new_curbuf to buf
void nvim_ecmd_au_new_curbuf_set(buf_T *buf) { set_bufref(&au_new_curbuf, buf); }

/// Returns 1 if bufref_valid(&au_new_curbuf)
int nvim_ecmd_au_new_curbuf_valid(void) { return bufref_valid(&au_new_curbuf) ? 1 : 0; }

/// Returns au_new_curbuf.br_buf (NULL if not set)
buf_T *nvim_ecmd_au_new_curbuf_get_buf(void) { return au_new_curbuf.br_buf; }

/// Save au_new_curbuf into a heap-allocated bufref, return void*
void *nvim_ecmd_au_new_curbuf_save(void)
{
  bufref_T *saved = xmalloc(sizeof(bufref_T));
  *saved = au_new_curbuf;
  return saved;
}

/// Restore au_new_curbuf from heap-allocated void* bufref (does NOT free it)
void nvim_ecmd_au_new_curbuf_restore(void *saved) { au_new_curbuf = *(bufref_T *)saved; }

// --- Buffer operation wrappers ---

/// Call close_buffer(oldwin, curbuf, flags, false, false). Returns did_decrement as int.
int nvim_ecmd_close_buffer(win_T *oldwin, int flags)
{
  return close_buffer(oldwin, curbuf, flags, false, false) ? 1 : 0;
}

/// Call open_buffer(false, eap, flags). Returns should_abort(result) as int.
int nvim_ecmd_open_buffer(exarg_T *eap, int flags)
{
  return should_abort(open_buffer(false, eap, flags)) ? 1 : 0;
}

/// Call check_changed(curbuf, flags). Returns 1 if changed (need to abort).
int nvim_ecmd_check_changed(int flags)
{
  return check_changed(curbuf, flags) ? 1 : 0;
}

/// Call u_savecommon(curbuf, 0, line_count+1, 0, true). Returns OK/FAIL as 1/0.
int nvim_ecmd_u_savecommon(int line_count)
{
  return u_savecommon(curbuf, 0, (linenr_T)(line_count + 1), 0, true) == OK ? 1 : 0;
}

// --- Cursor manipulation wrappers ---

/// Call check_cursor_col(curwin)
void nvim_ecmd_check_cursor_col(void) { check_cursor_col(curwin); }

/// Returns 1 if cursor position equals orig (pass lnum and col)
int nvim_ecmd_cursor_eq(int lnum, int col)
{
  pos_T orig = { .lnum = (linenr_T)lnum, .col = (colnr_T)col };
  return equalpos(curwin->w_cursor, orig) ? 1 : 0;
}

/// Get pointer to current cursor line text and return col of first non-blank.
/// This is used to check if cursor moved to first non-blank.
int nvim_ecmd_cursor_col_skipwhite(void)
{
  const char *text = get_cursor_line_ptr();
  return (int)(skipwhite(text) - text);
}

// --- Autocmd wrappers ---

/// Call apply_autocmds(EVENT_BUFLEAVE, NULL, NULL, false, curbuf)
void nvim_ecmd_apply_autocmds_bufleave(void)
{
  apply_autocmds(EVENT_BUFLEAVE, NULL, NULL, false, curbuf);
}

/// Call apply_autocmds_retval(EVENT_BUFENTER, NULL, NULL, false, curbuf, retval)
void nvim_ecmd_apply_autocmds_bufenter_retval(int *retval)
{
  apply_autocmds_retval(EVENT_BUFENTER, NULL, NULL, false, curbuf, retval);
}

/// Call apply_autocmds_retval(EVENT_BUFWINENTER, NULL, NULL, false, curbuf, retval)
void nvim_ecmd_apply_autocmds_bufwinenter_retval(int *retval)
{
  apply_autocmds_retval(EVENT_BUFWINENTER, NULL, NULL, false, curbuf, retval);
}

// --- Global state accessors ---

/// Returns 1 if cmdwin_buf != NULL
int nvim_ecmd_cmdwin_buf_is_nonnull(void) { return cmdwin_buf != NULL ? 1 : 0; }

/// Save and clear cmdwin state, returns heap-allocated bundle {type, win, old_curwin}
/// Save and clear cmdwin state. Returns heap-allocated opaque bundle.
/// Must be freed with nvim_ecmd_cmdwin_restore_free after restoring.
void *nvim_ecmd_cmdwin_save_clear(void)
{
  typedef struct { int type; win_T *win; win_T *old_curwin; } CmdwinState;
  CmdwinState *s = xmalloc(sizeof(CmdwinState));
  s->type = cmdwin_type;
  s->win = cmdwin_win;
  s->old_curwin = cmdwin_old_curwin;
  cmdwin_type = 0;
  cmdwin_win = NULL;
  cmdwin_old_curwin = NULL;
  return s;
}

/// Restore cmdwin state from heap-allocated bundle and free it
void nvim_ecmd_cmdwin_restore_free(void *bundle)
{
  typedef struct { int type; win_T *win; win_T *old_curwin; } CmdwinState;
  CmdwinState *s = (CmdwinState *)bundle;
  cmdwin_type = s->type;
  cmdwin_win = s->win;
  cmdwin_old_curwin = s->old_curwin;
  xfree(s);
}

/// Check if CMOD_KEEPALT is set in cmdmod.cmod_flags
int nvim_ecmd_cmdmod_has_keepalt(void) { return (cmdmod.cmod_flags & CMOD_KEEPALT) != 0 ? 1 : 0; }


// --- Misc wrappers ---

/// Call buflist_altfpos(win)
void nvim_ecmd_buflist_altfpos(win_T *win) { buflist_altfpos(win); }

/// Get mark from buflist_findfmark(buf): fills *lnum and *col
void nvim_ecmd_buflist_findfmark(buf_T *buf, int *lnum, int *col)
{
  const pos_T *pos = &buflist_findfmark(buf)->mark;
  *lnum = (int)pos->lnum;
  *col = (int)pos->col;
}

/// Check terminal size at cleanup: call terminal_check_size on old_curbuf's terminal
/// if valid; if bufref is invalid or no longer curbuf, also check curbuf's terminal.
void nvim_ecmd_terminal_check_size_cleanup(void *ref)
{
  bufref_T *br = (bufref_T *)ref;
  if (bufref_valid(br) && br->br_buf->terminal != NULL) {
    terminal_check_size(br->br_buf->terminal);
  }
  if (!bufref_valid(br) || br->br_buf != curbuf) {
    if (curbuf->terminal != NULL) {
      terminal_check_size(curbuf->terminal);
    }
  }
}

/// Call handle_swap_exists with old_curbuf reference (void* = bufref_T*)
void nvim_ecmd_handle_swap_exists(void *old_curbuf_ref)
{
  handle_swap_exists((bufref_T *)old_curbuf_ref);
}

/// Call setaltfname(ffname, sfname, lnum)
void nvim_ecmd_setaltfname(char *ffname, char *sfname, int lnum)
{
  setaltfname(ffname, sfname, (linenr_T)(lnum < 0 ? 0 : lnum));
}

/// Call delbuf_msg(name). Also frees name.
void nvim_ecmd_delbuf_msg(char *name) { rs_delbuf_msg(name); }

/// Call otherfile(ffname). Returns 1 if different file.
int nvim_ecmd_otherfile(char *ffname) { return otherfile(ffname) ? 1 : 0; }

/// Call path_fix_case(sfname) [only on case-insensitive systems]
void nvim_ecmd_path_fix_case(char *sfname)
{
#ifdef CASE_INSENSITIVE_FILENAME
  path_fix_case(sfname);
#else
  (void)sfname;
#endif
}

/// Returns 1 if build has CASE_INSENSITIVE_FILENAME defined
int nvim_ecmd_has_case_insensitive_filename(void)
{
#ifdef CASE_INSENSITIVE_FILENAME
  return 1;
#else
  return 0;
#endif
}

/// Call buflist_new(ffname, sfname, lnum, flags). Returns buf_T* or NULL.
buf_T *nvim_ecmd_buflist_new(char *ffname, char *sfname, int lnum, int flags)
{
  return buflist_new(ffname, sfname, (linenr_T)lnum, flags);
}

/// Call set_file_options(true, eap) and set_forced_fenc(eap)
void nvim_ecmd_set_file_options(exarg_T *eap)
{
  set_file_options(true, eap);
  set_forced_fenc(eap);
}

/// Wrap the FOR_ALL_TAB_WINDOWS loop for fold update all curbuf wins
void nvim_ecmd_fold_update_all_curbuf_wins(void)
{
  FOR_ALL_TAB_WINDOWS(tp, win) {
    if (win->w_buffer == curbuf) {
      rs_foldUpdateAll(win);
    }
  }
}

/// Get eap->do_ecmd_cmd (the command to run after loading, e.g. for +cmd)
const char *nvim_ecmd_eap_get_do_ecmd_cmd(exarg_T *eap)
{
  return eap != NULL ? eap->do_ecmd_cmd : NULL;
}

/// Call do_cmdline(command, NULL, NULL, DOCMD_VERBOSE)
void nvim_ecmd_do_cmdline(const char *command)
{
  do_cmdline((char *)command, NULL, NULL, DOCMD_VERBOSE);
}

/// Call set_vim_var_string(VV_SWAPCOMMAND, NULL, -1) to clear swapcommand
void nvim_ecmd_clear_swapcommand(void) { set_vim_var_string(VV_SWAPCOMMAND, NULL, -1); }



/// Emit emsg(_(e_cannot_switch_to_a_closing_buffer))
void nvim_ecmd_emsg_closing_buffer(void) { emsg(_(e_cannot_switch_to_a_closing_buffer)); }

/// Get curwin->w_buffer != NULL && oldwin == NULL check for b_locked_split
int nvim_ecmd_should_dec_nwindows_on_locked(win_T *oldwin)
{
  return (oldwin == NULL && curwin->w_buffer != NULL
          && curwin->w_buffer->b_nwindows > 1) ? 1 : 0;
}

/// curwin->w_s = &buf->b_s (set synblock to buf)
void nvim_ecmd_curwin_set_ws_to_buf(buf_T *buf) { curwin->w_s = &(buf->b_s); }

/// Shortmess flag 'O' check for SHM_OVERALL
int nvim_ecmd_shortmess_overall(void) { return shortmess(SHM_OVERALL) ? 1 : 0; }

/// Shortmess flag 'F' check for SHM_FILEINFO
int nvim_ecmd_shortmess_fileinfo(void) { return shortmess(SHM_FILEINFO) ? 1 : 0; }

/// Decrement curwin->w_buffer->b_nwindows if nwindows > 1 (for b_locked_split case)
void nvim_ecmd_dec_curwin_buf_nwindows_safe(void)
{
  if (curwin->w_buffer != NULL && curwin->w_buffer->b_nwindows > 1) {
    curwin->w_buffer->b_nwindows--;
  }
}

// =============================================================================
// cursor_pos_info display accessors (moved from ops.c Phase 5)
// =============================================================================

/// Format and display the visual mode message.
void nvim_cpi_format_visual_msg(int line_count_selected,
                                int start_vcol,
                                int end_vcol,
                                int is_block_mode,
                                int curswant_is_max,
                                int64_t word_count_cursor,
                                int64_t word_count,
                                int64_t char_count_cursor,
                                int64_t char_count,
                                int64_t byte_count_cursor,
                                int64_t byte_count)
{
  char buf1[50];

  if (is_block_mode && !curswant_is_max) {
    int64_t cols;
    STRICT_SUB(end_vcol + 1, start_vcol, &cols, int64_t);
    vim_snprintf(buf1, sizeof(buf1), _("%" PRId64 " Cols; "), cols);
  } else {
    buf1[0] = NUL;
  }

  if (char_count_cursor == byte_count_cursor
      && char_count == byte_count) {
    vim_snprintf(IObuff, IOSIZE,
                 _("Selected %s%" PRId64 " of %" PRId64 " Lines;"
                   " %" PRId64 " of %" PRId64 " Words;"
                   " %" PRId64 " of %" PRId64 " Bytes"),
                 buf1, (int64_t)line_count_selected,
                 (int64_t)curbuf->b_ml.ml_line_count,
                 word_count_cursor, word_count,
                 byte_count_cursor, byte_count);
  } else {
    vim_snprintf(IObuff, IOSIZE,
                 _("Selected %s%" PRId64 " of %" PRId64 " Lines;"
                   " %" PRId64 " of %" PRId64 " Words;"
                   " %" PRId64 " of %" PRId64 " Chars;"
                   " %" PRId64 " of %" PRId64 " Bytes"),
                 buf1, (int64_t)line_count_selected,
                 (int64_t)curbuf->b_ml.ml_line_count,
                 word_count_cursor, word_count,
                 char_count_cursor, char_count,
                 byte_count_cursor, byte_count);
  }
}

/// Format and display the normal mode message.
void nvim_cpi_format_normal_msg(int64_t word_count_cursor,
                                int64_t word_count,
                                int64_t char_count_cursor,
                                int64_t char_count,
                                int64_t byte_count_cursor,
                                int64_t byte_count)
{
  char buf1[50];
  char buf2[40];

  char *p = get_cursor_line_ptr();
  validate_virtcol(curwin);
  col_print(buf1, sizeof(buf1), (int)curwin->w_cursor.col + 1,
            (int)curwin->w_virtcol + 1);
  col_print(buf2, sizeof(buf2), get_cursor_line_len(), linetabsize_str(p));

  if (char_count_cursor == byte_count_cursor
      && char_count == byte_count) {
    vim_snprintf(IObuff, IOSIZE,
                 _("Col %s of %s; Line %" PRId64 " of %" PRId64 ";"
                   " Word %" PRId64 " of %" PRId64 ";"
                   " Byte %" PRId64 " of %" PRId64 ""),
                 buf1, buf2,
                 (int64_t)curwin->w_cursor.lnum,
                 (int64_t)curbuf->b_ml.ml_line_count,
                 word_count_cursor, word_count,
                 byte_count_cursor, byte_count);
  } else {
    vim_snprintf(IObuff, IOSIZE,
                 _("Col %s of %s; Line %" PRId64 " of %" PRId64 ";"
                   " Word %" PRId64 " of %" PRId64 ";"
                   " Char %" PRId64 " of %" PRId64 ";"
                   " Byte %" PRId64 " of %" PRId64 ""),
                 buf1, buf2,
                 (int64_t)curwin->w_cursor.lnum,
                 (int64_t)curbuf->b_ml.ml_line_count,
                 word_count_cursor, word_count,
                 char_count_cursor, char_count,
                 byte_count_cursor, byte_count);
  }
}

/// Append BOM info to IObuff and display the message.
void nvim_cpi_append_bom_and_display(int64_t bom_count)
{
  if (bom_count > 0) {
    const size_t len = strlen(IObuff);
    vim_snprintf(IObuff + len, IOSIZE - len,
                 _("(+%" PRId64 " for BOM)"), bom_count);
  }
  // Don't shorten this message, the user asked for it.
  char *p = p_shm;
  p_shm = "";
  if (p_ch < 1) {
    msg_start();
    msg_scroll = true;
  }
  msg(IObuff, 0);
  p_shm = p;
}

/// Rust port of line_count_info (in ops crate).
extern varnumber_T nvim_rs_line_count_info(char *line, varnumber_T *wc, varnumber_T *cc,
                                           varnumber_T limit, int eol_size);

/// Struct matching Rust CpiLineCountResult.
typedef struct {
  int64_t byte_count;
  int64_t word_count;
  int64_t char_count;
} CpiLineCountResult;

/// Set up block visual mode: get virtual columns with sbr temporarily cleared.
void nvim_cpi_setup_block_visual(int min_lnum, int min_col,
                                 int max_lnum, int max_col,
                                 int *out_start_vcol, int *out_end_vcol)
{
  pos_T min_pos = { .lnum = min_lnum, .col = min_col, .coladd = 0 };
  pos_T max_pos = { .lnum = max_lnum, .col = max_col, .coladd = 0 };

  char *const saved_sbr = p_sbr;
  char *const saved_w_sbr = curwin->w_p_sbr;
  p_sbr = empty_string_option;
  curwin->w_p_sbr = empty_string_option;

  oparg_T oparg;
  memset(&oparg, 0, sizeof(oparg));
  oparg.is_VIsual = true;
  oparg.motion_type = kMTBlockWise;
  oparg.op_type = OP_NOP;
  getvcols(curwin, &min_pos, &max_pos, &oparg.start_vcol, &oparg.end_vcol);

  p_sbr = saved_sbr;
  curwin->w_p_sbr = saved_w_sbr;

  *out_start_vcol = (int)oparg.start_vcol;
  *out_end_vcol = (int)oparg.end_vcol;
}

/// Count info for a block visual line (using block_prep).
void nvim_cpi_block_line_count(int lnum, int eol_size, void *out_ptr)
{
  CpiLineCountResult *out = (CpiLineCountResult *)out_ptr;
  oparg_T oparg;
  memset(&oparg, 0, sizeof(oparg));
  oparg.is_VIsual = true;
  oparg.motion_type = kMTBlockWise;
  oparg.op_type = OP_NOP;

  // We need the vcols from the current visual selection.
  // Re-derive them from VIsual and cursor.
  pos_T min_pos, max_pos;
  if (lt(VIsual, curwin->w_cursor)) {
    min_pos = VIsual;
    max_pos = curwin->w_cursor;
  } else {
    min_pos = curwin->w_cursor;
    max_pos = VIsual;
  }
  if (*p_sel == 'e' && max_pos.col > 0) {
    max_pos.col--;
  }

  char *const saved_sbr = p_sbr;
  char *const saved_w_sbr = curwin->w_p_sbr;
  p_sbr = empty_string_option;
  curwin->w_p_sbr = empty_string_option;
  getvcols(curwin, &min_pos, &max_pos, &oparg.start_vcol, &oparg.end_vcol);
  p_sbr = saved_sbr;
  curwin->w_p_sbr = saved_w_sbr;

  if (curwin->w_curswant == MAXCOL) {
    oparg.end_vcol = MAXCOL;
  }
  if (oparg.end_vcol < oparg.start_vcol) {
    oparg.end_vcol += oparg.start_vcol;
    oparg.start_vcol = oparg.end_vcol - oparg.start_vcol;
    oparg.end_vcol -= oparg.start_vcol;
  }

  struct block_def bd;
  virtual_op = virtual_active(curwin);
  block_prep(&oparg, &bd, (linenr_T)lnum, false);
  virtual_op = kNone;

  varnumber_T wc = 0, cc = 0;
  varnumber_T bc = 0;
  if (bd.textstart != NULL) {
    bc = nvim_rs_line_count_info(bd.textstart, &wc, &cc, (varnumber_T)bd.textlen, eol_size);
  }
  out->byte_count = (int64_t)bc;
  out->word_count = (int64_t)wc;
  out->char_count = (int64_t)cc;
}

// General cpo/cmdmod/join accessors (migrated from edit.c)
bool nvim_p_cpo_has_backspace(void) { return vim_strchr(p_cpo, CPO_BACKSPACE) != NULL; }
bool nvim_p_cpo_has_replcnt(void) { return vim_strchr(p_cpo, CPO_REPLCNT) != NULL; }
bool nvim_cmod_keepjumps(void) { return (cmdmod.cmod_flags & CMOD_KEEPJUMPS) != 0; }
void nvim_do_join_simple(void) { do_join(2, false, false, false, false); }
