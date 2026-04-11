#include <inttypes.h>
#include <stdbool.h>
#include <stddef.h>
#include <string.h>
#include <time.h>
#include "auto/config.h"
#include "klib/kvec.h"
#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/state.h"
#include "nvim/bufwrite.h"
#include "nvim/change.h"
#include "nvim/channel.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/move.h"
#include "nvim/ops.h"
#include "nvim/plines.h"
#include "nvim/cmdhist.h"
#include "nvim/decoration.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_getln.h"
#include "nvim/extmark.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/globals.h"
#include "nvim/mark.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/os/shell.h"
#include "nvim/os/time.h"
#include "nvim/path.h"
#include "nvim/regexp.h"
#include "nvim/search.h"
#include "nvim/strings.h"
#include "nvim/terminal.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/window.h"
typedef struct {
  lpos_T start;  // start of the match (after substitution)
  lpos_T end;    // end of the match
  linenr_T pre_match;  // where to begin showing lines before the match
} SubResult;
typedef struct {
  kvec_t(SubResult) subresults;
  linenr_T lines_needed;  // lines needed in the preview window
} PreviewLines;
#include "ex_cmds_shim.c.generated.h"
_Static_assert(VV_SWAPCOMMAND == 49, "VV_SWAPCOMMAND mismatch with Rust constant");
_Static_assert(HLF_R == 18, "HLF_R mismatch with Rust constant");
extern const char *tv_list_find_str(list_T *l, int n);
extern int rs_ml_find_line_or_offset(buf_T *buf, linenr_T lnum, int *offp, bool no_ff);
extern int rs_win_valid(win_T *win);
extern void rs_foldMoveRange(win_T *wp, garray_T *gap, linenr_T line1, linenr_T line2, linenr_T dest);
extern void rs_foldUpdateAll(win_T *win);
extern int rs_magic_isset(void);
int nvim_exarg_get_cmdidx(exarg_T *eap) { return (int)eap->cmdidx; }
const char *nvim_exarg_get_arg(exarg_T *eap) { return eap->arg; }
linenr_T nvim_exarg_get_line1(exarg_T *eap) { return eap->line1; }
linenr_T nvim_exarg_get_line2(exarg_T *eap) { return eap->line2; }
int nvim_exarg_get_addr_count(exarg_T *eap) { return eap->addr_count; }
int nvim_exarg_get_forceit(exarg_T *eap) { return eap->forceit ? 1 : 0; }
int nvim_exarg_get_flags(exarg_T *eap) { return eap->flags; }
void nvim_exarg_set_line2(exarg_T *eap, linenr_T line2) { eap->line2 = line2; }
int nvim_curwin_get_w_p_rl(void) { return curwin->w_p_rl; }
int nvim_curbuf_get_b_p_tw(void) { return (int)curbuf->b_p_tw; }
int nvim_curbuf_get_b_p_wm(void) { return (int)curbuf->b_p_wm; }
int nvim_curwin_get_view_width(void) { return curwin->w_view_width; }
void nvim_curwin_set_cursor_lnum(linenr_T lnum) { curwin->w_cursor.lnum = lnum; }
int nvim_is_one_window(void) { return ONE_WINDOW ? 1 : 0; }
int64_t nvim_curwin_get_p_scr(void) { return curwin->w_p_scr; }
int nvim_curwin_get_view_height(void) { return curwin->w_view_height; }
void nvim_set_ex_no_reprint(int val) { ex_no_reprint = val != 0; }
int nvim_get_ex_no_reprint(void) { return ex_no_reprint ? 1 : 0; }
const char *nvim_get_e_empty_buffer(void) { return _(e_empty_buffer); }
int nvim_cmdmod_has_lockmarks(void) { return (cmdmod.cmod_flags & CMOD_LOCKMARKS) != 0; }
int nvim_cmdmod_has_keeppatterns(void) { return (cmdmod.cmod_flags & CMOD_KEEPPATTERNS) != 0; }
int nvim_curbuf_get_b_p_ai(void) { return curbuf->b_p_ai; }
void nvim_check_pos_visual(void) { check_pos(curbuf, &VIsual); }
void nvim_transchar_nonprint_curbuf(char *buf, int c) { transchar_nonprint(curbuf, buf, c); }
void nvim_curbuf_set_op_start(linenr_T lnum, colnr_T col) { curbuf->b_op_start.lnum = lnum; curbuf->b_op_start.col = col; }
void nvim_curbuf_set_op_end(linenr_T lnum, colnr_T col) { curbuf->b_op_end.lnum = lnum; curbuf->b_op_end.col = col; }
int nvim_curwin_get_w_p_nu(void) { return curwin->w_p_nu; }
void *nvim_excmds_regcomp(const char *pat, int magic_val) { regmatch_T *rm = xcalloc(1, sizeof(regmatch_T)); rm->regprog = vim_regcomp(pat, magic_val); if (rm->regprog == NULL) { xfree(rm); return NULL; } return rm; }
void nvim_excmds_regfree(void *rm) { if (rm != NULL) { vim_regfree(((regmatch_T *)rm)->regprog); xfree(rm); } }
const char *nvim_excmds_regmatch_startp0(const void *rm) { return ((const regmatch_T *)rm)->startp[0]; }
const char *nvim_excmds_regmatch_endp0(const void *rm) { return ((const regmatch_T *)rm)->endp[0]; }
void nvim_excmds_regmatch_set_ic(void *rm, int ic) { ((regmatch_T *)rm)->rm_ic = ic; }
void nvim_excmds_str2nr(const char *s, int what, int64_t *result) { varnumber_T val = 0; vim_str2nr(s, NULL, NULL, what, &val, NULL, 0, false, NULL); *result = (int64_t)val; }
void nvim_excmds_semsg_invarg2(const char *p) { semsg(_(e_invarg2), p); }
void nvim_excmds_emsg_invarg(void) { emsg(_(e_invarg)); }
void nvim_excmds_emsg_noprevre(void) { emsg(_(e_noprevre)); }
void nvim_excmds_emsg_interr(void) { emsg(_(e_interr)); }
void nvim_exarg_set_nextcmd(exarg_T *eap, const char *p) { eap->nextcmd = (char *)p; }
int nvim_exarg_is_nextcmd_null(exarg_T *eap) { return eap->nextcmd == NULL ? 1 : 0; }
void nvim_excmds_fold_move_range_all_wins(linenr_T line1, linenr_T line2, linenr_T dest)
{
  FOR_ALL_TAB_WINDOWS(tab, win) {
    if (win->w_buffer == curbuf) {
      rs_foldMoveRange(win, &win->w_folds, line1, line2, dest);
    }
  }
}
void nvim_excmds_smsg_lines_moved(int64_t num_lines) { smsg(0, NGETTEXT("%" PRId64 " line moved", "%" PRId64 " lines moved", (int)num_lines), num_lines); }
void nvim_excmds_emsg_e134(void) { emsg(_("E134: Cannot move a range of lines into itself")); }
void nvim_excmds_toggle_b_p_ai(void) { curbuf->b_p_ai = !curbuf->b_p_ai; }
int nvim_excmds_get_b_p_iminsert(void) { return curbuf->b_p_iminsert; }
int nvim_excmds_ea_getline_is_null(exarg_T *eap) { return eap->ea_getline == NULL ? 1 : 0; }
int nvim_excmds_get_cstack_looplevel(exarg_T *eap) { return eap->cstack->cs_looplevel; }
char *nvim_excmds_call_getline(exarg_T *eap, int c, int indent) { return eap->ea_getline(c, eap->cookie, indent, true); }
char *nvim_excmds_get_nextcmd(exarg_T *eap) { return eap->nextcmd; }
char *nvim_excmds_get_arg_mut(exarg_T *eap) { return eap->arg; }
int nvim_exarg_get_skip(exarg_T *eap) { return eap->skip; }
void nvim_exarg_set_flags(exarg_T *eap, int flags) { eap->flags = flags; }
const char *nvim_excmds_shell_name_tail(void) { return invocation_path_tail(p_sh, NULL); }
_Static_assert(ML_DEL_MESSAGE == 1, "ML_DEL_MESSAGE mismatch");
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
_Static_assert(EVENT_BUFADD == 0, "EVENT_BUFADD mismatch");
_Static_assert(EVENT_BUFFILEPOST == 4, "EVENT_BUFFILEPOST mismatch");
_Static_assert(EVENT_BUFFILEPRE == 5, "EVENT_BUFFILEPRE mismatch");
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
_Static_assert(CMD_tilde == 554, "CMD_tilde mismatch - update Rust constant");
_Static_assert(MAXLNUM == 0x7fffffff, "MAXLNUM mismatch - update Rust constant");
_Static_assert(SEARCH_HIS == 0x20, "SEARCH_HIS mismatch - update Rust constant");
_Static_assert(REGSUB_COPY == 1, "REGSUB_COPY mismatch - update Rust constant");
_Static_assert(REGSUB_MAGIC == 2, "REGSUB_MAGIC mismatch - update Rust constant");
_Static_assert(REGSUB_BACKSLASH == 4, "REGSUB_BACKSLASH mismatch - update Rust constant");
extern int rs_check_writable(const char *fname);
extern int rs_check_readonly(exarg_T *eap, buf_T *buf);
extern void rs_sub_set_replacement(char *sub, uint64_t timestamp, void *additional_data);
int append_indent = 0;
int global_need_beginline = 0;
bool nvim_excmds_format_sub_msg(bool count_only, int nsubs, int nlines)
{
  if (got_int) { STRCPY(msg_buf, _("(Interrupted) ")); } else { *msg_buf = NUL; }
  char *s = count_only
    ? NGETTEXT("%" PRId64 " match on %" PRId64 " line", "%" PRId64 " matches on %" PRId64 " line", nsubs)
    : NGETTEXT("%" PRId64 " substitution on %" PRId64 " line", "%" PRId64 " substitutions on %" PRId64 " line", nsubs);
  char *p = count_only
    ? NGETTEXT("%" PRId64 " match on %" PRId64 " lines", "%" PRId64 " matches on %" PRId64 " lines", nsubs)
    : NGETTEXT("%" PRId64 " substitution on %" PRId64 " lines", "%" PRId64 " substitutions on %" PRId64 " lines", nsubs);
  vim_snprintf_add(msg_buf, sizeof(msg_buf), NGETTEXT(s, p, nlines), (int64_t)nsubs, (int64_t)nlines);
  if (msg(msg_buf, 0)) { set_keep_msg(msg_buf, 0); }
  return true;
}
int nvim_excmds_curwin_get_pvw(void) { return curwin->w_p_pvw; }
void nvim_excmds_curwin_set_pvw(int val) { curwin->w_p_pvw = (bool)val; }
void nvim_excmds_curwin_set_wfh(int val) { curwin->w_p_wfh = (bool)val; }
void nvim_excmds_curwin_set_diff(int val) { curwin->w_p_diff = (bool)val; }
win_T *nvim_excmds_find_preview_win(void)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_p_pvw) {
      return wp;
    }
  }
  return NULL;
}
void nvim_excmds_reset_binding_curwin(void) { RESET_BINDING(curwin); }
void nvim_excmds_set_foldcolumn_zero(void) { set_option_direct(kOptFoldcolumn, STATIC_CSTR_AS_OPTVAL("0"), 0, SID_NONE); }
int nvim_excmds_oldfiles_count(void) { list_T *l = get_vim_var_list(VV_OLDFILES); return l == NULL ? 0 : (int)tv_list_len(l); }
const char *nvim_excmds_oldfiles_find_str(int idx) { list_T *l = get_vim_var_list(VV_OLDFILES); if (l == NULL) { return NULL; } return tv_list_find_str(l, idx); }
int nvim_excmds_cmdmod_has_browse(void) { return (cmdmod.cmod_flags & CMOD_BROWSE) != 0; }
void nvim_excmds_do_exedit_edit(exarg_T *eap, char *arg) { char *saved_arg = eap->arg; int saved_cmdidx = eap->cmdidx; eap->arg = arg; eap->cmdidx = CMD_edit; cmdmod.cmod_flags &= ~CMOD_BROWSE; do_exedit(eap, NULL); eap->arg = saved_arg; eap->cmdidx = saved_cmdidx; }
int nvim_excmds_any_buf_changed(void)
{
  FOR_ALL_BUFFERS(buf) {
    if (bufIsChanged(buf)) {
      return 1;
    }
  }
  return 0;
}
void nvim_excmds_do_cmdline_global(const char *cmd) { if (cmd == NULL || *cmd == NUL || *cmd == '\n') { do_cmdline("p", NULL, NULL, DOCMD_NOWAIT); } else { do_cmdline((char *)cmd, NULL, NULL, DOCMD_NOWAIT); } }
char *nvim_excmds_save_set_shortmess_F(void) { char *saved = xstrdup(p_shm); set_option_direct(kOptShortmess, STATIC_CSTR_AS_OPTVAL("F"), 0, SID_NONE); return saved; }
void nvim_excmds_restore_shortmess(char *saved) { set_option_direct(kOptShortmess, CSTR_AS_OPTVAL(saved), 0, SID_NONE); xfree(saved); }
int nvim_excmds_get_p_icm_first(void) { return (unsigned char)p_icm[0]; }
void nvim_excmds_bufhl_add_hl_pos_offset(buf_T *buf, int ns_id, int hl_id, linenr_T start_lnum, colnr_T start_col, linenr_T end_lnum, colnr_T end_col, colnr_T offset) { rs_bufhl_add_hl_pos_offset(buf, ns_id, hl_id, start_lnum, start_col, end_lnum, end_col, offset); }
size_t nvim_excmds_preview_lines_size(const void *pl) { return ((const PreviewLines *)pl)->subresults.size; }
void nvim_excmds_preview_lines_item(const void *pl, size_t idx, linenr_T *start_lnum, colnr_T *start_col, linenr_T *end_lnum, colnr_T *end_col, linenr_T *pre_match) { SubResult item = ((const PreviewLines *)pl)->subresults.items[idx]; *start_lnum = item.start.lnum; *start_col = item.start.col; *end_lnum = item.end.lnum; *end_col = item.end.col; *pre_match = item.pre_match; }
const char *nvim_exarg_get_cmd(const exarg_T *eap) { return eap->cmd; }
void *nvim_excmds_search_regcomp_multi(const char *pat, size_t patlen, const char **used_pat_out, int which_pat) { regmmatch_T *rm = xmalloc(sizeof(regmmatch_T)); memset(rm, 0, sizeof(*rm)); if (search_regcomp((char *)pat, patlen, (char **)used_pat_out, RE_BOTH, which_pat, SEARCH_HIS, rm) == FAIL) { xfree(rm); return NULL; } return rm; }
int nvim_excmds_vim_regexec_multi(void *regmatch, int lnum) { return vim_regexec_multi((regmmatch_T *)regmatch, curwin, curbuf, (linenr_T)lnum, 0, NULL, NULL); }
void nvim_excmds_vim_regfree_multi(void *regmatch) { if (regmatch == NULL) { return; } regmmatch_T *rm = (regmmatch_T *)regmatch; vim_regfree(rm->regprog); xfree(rm); }
int nvim_excmds_regmmatch_regprog_null(void *regmatch) { return ((regmmatch_T *)regmatch)->regprog == NULL ? 1 : 0; }
char *nvim_excmds_skip_regexp_ex_global(exarg_T *eap, char *pat, int delim) { return skip_regexp_ex(pat, (char)delim, rs_magic_isset(), &eap->arg, NULL, NULL); }
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
void nvim_excmds_disable_inccommand(void) { set_option_direct(kOptInccommand, STATIC_CSTR_AS_OPTVAL(""), 0, SID_NONE); }
int nvim_excmds_curwin_cursor_lnum(void) { return (int)curwin->w_cursor.lnum; }
void nvim_excmds_curwin_set_col_zero(void) { curwin->w_cursor.col = 0; }
void *nvim_excmds_get_curbuf_identity(void) { return (void *)curbuf; }
int nvim_excmds_curbuf_is(void *ptr) { return curbuf == (buf_T *)ptr ? 1 : 0; }
char *nvim_excmds_curbuf_get_ffname(void) { return curbuf->b_ffname; }
char *nvim_excmds_curbuf_get_sfname(void) { return curbuf->b_sfname; }
char *nvim_excmds_curbuf_get_fname(void) { return curbuf->b_fname; }
void nvim_excmds_curbuf_set_ffname(char *p) { curbuf->b_ffname = p; }
void nvim_excmds_curbuf_set_sfname(char *p) { curbuf->b_sfname = p; }
void nvim_excmds_curbuf_clear_filenames(void) { curbuf->b_ffname = NULL; curbuf->b_sfname = NULL; }
void nvim_excmds_curbuf_set_bf_notedited(void) { curbuf->b_flags |= BF_NOTEDITED; }
int nvim_excmds_cmdmod_has_keepalt(void) { return (cmdmod.cmod_flags & CMOD_KEEPALT) != 0 ? 1 : 0; }
void nvim_excmds_set_curwin_alt_fnum(int fnum) { curwin->w_alt_fnum = fnum; }
int nvim_excmds_curbuf_ffname_not_null(void) { return curbuf->b_ffname != NULL ? 1 : 0; }
int nvim_excmds_os_path_exists_curbuf_ffname(void) { return curbuf->b_ffname != NULL && os_path_exists(curbuf->b_ffname) ? 1 : 0; }
int nvim_exarg_cmdidx_is_saveas(const exarg_T *eap) { return eap->cmdidx == CMD_saveas ? 1 : 0; }
int nvim_exarg_get_usefilter(const exarg_T *eap) { return eap->usefilter ? 1 : 0; }
void nvim_exarg_set_line1(exarg_T *eap, int line1) { eap->line1 = (linenr_T)line1; }
int nvim_excmds_curwin_get_w_arg_idx(void) { return curwin->w_arg_idx; }
int nvim_exarg_get_cmd_byte1(const exarg_T *eap) { return (unsigned char)eap->cmd[1]; }
int nvim_excmds_arg_has_valid_delim(const exarg_T *eap) { return (*eap->arg && !ASCII_ISALNUM(*eap->arg)) ? 1 : 0; }
void nvim_excmds_eap_arg_restore(exarg_T *eap, char *saved) { eap->arg = saved; }
int nvim_excmds_curbuf_op_start_lnum(void) { return (int)curbuf->b_op_start.lnum; }
int nvim_excmds_curbuf_op_end_lnum(void) { return (int)curbuf->b_op_end.lnum; }
void nvim_excmds_curbuf_set_op_start_lnum(int lnum) { curbuf->b_op_start.lnum = (linenr_T)lnum; }
void nvim_excmds_curbuf_set_op_end_lnum(int lnum) { curbuf->b_op_end.lnum = (linenr_T)lnum; }
uint64_t nvim_excmds_curwin_cursor_save(void) { return ((uint64_t)(uint32_t)curwin->w_cursor.lnum << 32) | (uint32_t)(int32_t)curwin->w_cursor.col; }
void nvim_excmds_curwin_cursor_restore(uint64_t saved) { curwin->w_cursor.lnum = (linenr_T)(uint32_t)(saved >> 32); curwin->w_cursor.col = (colnr_T)(int32_t)(uint32_t)saved; }
int nvim_excmds_cmdmod_save_clear_lockmarks(void) { int saved = cmdmod.cmod_flags; cmdmod.cmod_flags &= ~CMOD_LOCKMARKS; return saved; }
void nvim_excmds_cmdmod_restore_flags(int saved) { cmdmod.cmod_flags = saved; }
int nvim_excmds_cmdmod_has_keepmarks_now(void) { return (cmdmod.cmod_flags & CMOD_KEEPMARKS) != 0 ? 1 : 0; }
int nvim_excmds_buf_write_filter(const char *itmp, int line1, int line2, exarg_T *eap) { return buf_write(curbuf, (char *)itmp, NULL, (linenr_T)line1, (linenr_T)line2, eap, false, false, false, true) == OK ? 1 : 0; }
int nvim_excmds_readfile_filter(const char *otmp, int line2, exarg_T *eap) { return readfile((char *)otmp, NULL, (linenr_T)line2, 0, (linenr_T)MAXLNUM, eap, READ_FILTER, false) == OK ? 1 : 0; }
int nvim_excmds_p_cpo_no_remmark(void) { return vim_strchr(p_cpo, CPO_REMMARK) == NULL ? 1 : 0; }
void nvim_excmds_msg_lines_filtered(int linecount) { char msg_buf[80]; vim_snprintf(msg_buf, sizeof(msg_buf), _("%" PRId64 " lines filtered"), (int64_t)linecount); if (msg(msg_buf, 0) && !msg_scroll) { set_keep_msg(msg_buf, 0); } }
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
  case 14: semsg(_("E143: Autocommands unexpectedly deleted new buffer %s"), arg == NULL ? "" : arg); au_new_curbuf.br_buf = NULL; au_new_curbuf.br_buf_free_count = 0; break;
  default:  break;
  }
}
void nvim_excmds_curbuf_op_save(uint64_t *out_start, uint64_t *out_end) { *out_start = ((uint64_t)(uint32_t)curbuf->b_op_start.lnum << 32) | (uint32_t)(int32_t)curbuf->b_op_start.col; *out_end = ((uint64_t)(uint32_t)curbuf->b_op_end.lnum << 32) | (uint32_t)(int32_t)curbuf->b_op_end.col; }
void nvim_excmds_curbuf_op_restore(uint64_t saved_start, uint64_t saved_end) { curbuf->b_op_start.lnum = (linenr_T)(uint32_t)(saved_start >> 32); curbuf->b_op_start.col = (colnr_T)(int32_t)(uint32_t)saved_start; curbuf->b_op_end.lnum = (linenr_T)(uint32_t)(saved_end >> 32); curbuf->b_op_end.col = (colnr_T)(int32_t)(uint32_t)saved_end; }
void nvim_excmds_curbuf_op_adjust_lnum(int delta) { curbuf->b_op_start.lnum += (linenr_T)delta; curbuf->b_op_end.lnum += (linenr_T)delta; }
int nvim_excmds_curbuf_ml_line_count(void) { return (int)curbuf->b_ml.ml_line_count; }
int nvim_excmds_curbuf_get_b_fnum(void) { return curbuf->b_fnum; }
int nvim_excmds_curbuf_get_b_nwindows(void) { return curbuf->b_nwindows; }
_Static_assert(CMD_xall == 537, "CMD_xall mismatch -- update the Rust constant in write.rs");
_Static_assert(CMD_wqall == 532, "CMD_wqall mismatch -- update the Rust constant in write.rs");
buf_T *nvim_excmds_buf_get_next(const buf_T *buf) { return buf->b_next; }
int nvim_excmds_buf_has_running_job(const buf_T *buf) { return (buf->terminal != NULL && channel_job_running((uint64_t)buf->b_p_channel)) ? 1 : 0; }
int nvim_excmds_buf_get_b_fnum(const buf_T *buf) { return buf->b_fnum; }
void nvim_excmds_semsg_e141(int64_t fnum) { semsg(_("E141: No file name for buffer %" PRId64), fnum); }
int nvim_excmds_check_readonly_buf(int forceit_in, buf_T *buf, int *forceit_out) { exarg_T fake_eap = { 0 }; fake_eap.forceit = (bool)forceit_in; int result = rs_check_readonly(&fake_eap, buf); *forceit_out = (int)fake_eap.forceit; return result; }
void *nvim_excmds_new_bufref(buf_T *buf) { bufref_T *ref = xmalloc(sizeof(bufref_T)); set_bufref(ref, buf); return ref; }
int nvim_excmds_bufref_valid(void *ref) { return bufref_valid((bufref_T *)ref) ? 1 : 0; }
int nvim_excmds_buf_get_b_flags(const buf_T *buf) { return (int)buf->b_flags; }
int nvim_excmds_cpo_no_overnew(void) { return vim_strchr(p_cpo, CPO_OVERNEW) == NULL ? 1 : 0; }
int nvim_excmds_dialog_overwrite(exarg_T *eap, const char *fname) { char buff[DIALOG_MSG_SIZE]; dialog_msg(buff, _("Overwrite existing file \"%s\"?"), (char *)fname); if (vim_dialog_yesno(VIM_QUESTION, NULL, buff, 2) == VIM_YES) { eap->forceit = true; return 1; } return 0; }
char *nvim_excmds_get_first_dir(void) { if (*p_dir == NUL) { char *dir = xmalloc(5); STRCPY(dir, "."); return dir; } char *dir = xmalloc(MAXPATHL); char *p = p_dir; copy_option_part(&p, dir, MAXPATHL, ","); return dir; }
char *nvim_excmds_makeswapname(const char *fname, const char *ffname, const char *dir) { return makeswapname((char *)fname, (char *)ffname, curbuf, (char *)dir); }
int nvim_excmds_dialog_swapfile(exarg_T *eap, const char *swapname) { char buff[DIALOG_MSG_SIZE]; dialog_msg(buff, _("Swap file \"%s\" exists, overwrite anyway?"), (char *)swapname); if (vim_dialog_yesno(VIM_QUESTION, NULL, buff, 2) == VIM_YES) { eap->forceit = true; return 1; } return 0; }
int nvim_excmds_eap_get_append(const exarg_T *eap) { return eap->append ? 1 : 0; }
int nvim_excmds_vim_strchr_cpo_altwrite(void) { return vim_strchr(p_cpo, CPO_ALTWRITE) != NULL ? 1 : 0; }
void *nvim_excmds_setaltfname(const char *ffname, const char *fname, int lnum) { return setaltfname((char *)ffname, (char *)fname, (linenr_T)lnum); }
void nvim_excmds_emsg_e_bufloaded(void) { emsg(_(e_bufloaded)); }
#ifdef UNIX
int nvim_excmds_curbuf_check_writable(void) { return rs_check_writable(curbuf->b_ffname); }
#else
int nvim_excmds_curbuf_check_writable(void) { return 1; }
#endif
int nvim_excmds_dialog_write_partial(void) { return vim_dialog_yesno(VIM_QUESTION, NULL, _("Write partial file?"), 2) == VIM_YES ? 1 : 0; }
void nvim_excmds_buf_swap_filenames(buf_T *alt_buf) { char *tmp; tmp = alt_buf->b_fname; alt_buf->b_fname = curbuf->b_fname; curbuf->b_fname = tmp; tmp = alt_buf->b_ffname; alt_buf->b_ffname = curbuf->b_ffname; curbuf->b_ffname = tmp; tmp = alt_buf->b_sfname; alt_buf->b_sfname = curbuf->b_sfname; curbuf->b_sfname = tmp; }
void nvim_excmds_buf_name_changed_curbuf(void) { buf_name_changed(curbuf); }
int nvim_excmds_buf_get_b_p_bl(const buf_T *buf) { return buf->b_p_bl ? 1 : 0; }
void nvim_excmds_buf_set_b_p_bl_true(buf_T *buf) { buf->b_p_bl = true; }
int nvim_excmds_buf_ft_is_empty(const buf_T *buf) { return *buf->b_p_ft == NUL ? 1 : 0; }
int nvim_excmds_augroup_exists_filetypedetect(void) { return augroup_exists("filetypedetect") ? 1 : 0; }
void nvim_excmds_do_doautocmd_filetypedetect(void) { do_doautocmd("filetypedetect BufRead", true, NULL); }
void nvim_excmds_do_modelines(void) { do_modelines(0); }
int nvim_excmds_buf_write_do_write(const char *ffname, const char *fname, int line1, int line2, exarg_T *eap, int append, int forceit) { return buf_write(curbuf, (char *)ffname, (char *)fname, (linenr_T)line1, (linenr_T)line2, eap, (bool)append, (bool)forceit, true, false) == OK ? 1 : 0; }
void nvim_excmds_saveas_post_success(void) { curbuf->b_p_ro = false; redraw_tabline = true; }
int nvim_excmds_eap_get_mkdir_p(const exarg_T *eap) { return eap->mkdir_p ? 1 : 0; }
int nvim_excmds_buf_get_b_p_ro(const buf_T *buf) { return buf->b_p_ro ? 1 : 0; }
const char *nvim_excmds_buf_get_b_fname(const buf_T *buf) { return buf->b_fname; }
const char *nvim_excmds_buf_get_b_ffname_ptr(const buf_T *buf) { return buf->b_ffname; }
int nvim_excmds_buf_ffname_path_exists(const buf_T *buf) { return (buf->b_ffname != NULL && os_path_exists(buf->b_ffname)) ? 1 : 0; }
int nvim_excmds_buf_ffname_is_writable(const buf_T *buf) { return (buf->b_ffname != NULL && os_file_is_writable(buf->b_ffname)) ? 1 : 0; }
int nvim_excmds_p_confirm_or_cmod_confirm(void) { return (p_confirm || (cmdmod.cmod_flags & CMOD_CONFIRM)) ? 1 : 0; }
int nvim_excmds_vim_dialog_yesno_question(const char *msg) { return vim_dialog_yesno(VIM_QUESTION, NULL, (char *)msg, 2) == VIM_YES ? 1 : 0; }
char *nvim_excmds_dialog_msg_readonly(int fmt_id, const char *arg) { char *buff = xmalloc(DIALOG_MSG_SIZE); if (fmt_id == 0) { dialog_msg(buff, _("'readonly' option is set for \"%s\".\nDo you wish to write anyway?"), (char *)arg); } else { dialog_msg(buff, _("File permissions of \"%s\" are read-only.\nIt may still be possible to write it.\nDo you wish to try?"), (char *)arg); } return buff; }
void nvim_excmds_set_forceit(exarg_T *eap, int val) { eap->forceit = (bool)val; }
int nvim_curwin_get_w_botline(void) { return (int)curwin->w_botline; }
int nvim_curwin_get_w_p_crb(void) { return curwin->w_p_crb ? 1 : 0; }
int nvim_curwin_get_w_p_fen(void) { return curwin->w_p_fen ? 1 : 0; }
void nvim_curwin_set_w_p_fen(int val) { curwin->w_p_fen = (bool)val; }
void nvim_curbuf_set_deleted_bytes2(int val) { curbuf->deleted_bytes2 = (bcount_t)val; }
char *nvim_do_sub_skip_regexp_ex(char *cmd, int delim, char **arg_ptr) { return skip_regexp_ex(cmd, (char)delim, rs_magic_isset(), arg_ptr, NULL, NULL); }
void nvim_do_sub_getvcol_start_end(int lnum, int start_col, int end_col, int *sc_out, int *ec_out) { pos_T pos = { (linenr_T)lnum, (colnr_T)start_col, 0 }; colnr_T sc = 0; getvcol(curwin, &pos, &sc, NULL, NULL); *sc_out = (int)sc; pos.col = (colnr_T)end_col; colnr_T ec = 0; getvcol(curwin, &pos, NULL, NULL, &ec); *ec_out = (int)ec; }
int nvim_do_sub_getcmdline_prompt(const char *prompt_str) { char *resp = rs_getcmdline_prompt(-1, (char *)prompt_str, 0, EXPAND_NOTHING, NULL, 0, NULL); msg_putchar('\n'); int typed = NUL; if (resp != NULL) { typed = (uint8_t)(*resp); xfree(resp); } return typed; }
void nvim_do_sub_update_screen_for_confirm(void) { update_topline(curwin); validate_cursor(curwin); redraw_later(curwin, UPD_SOME_VALID); show_cursor_info_later(true); update_screen(); redraw_later(curwin, UPD_SOME_VALID); }
void nvim_do_sub_set_op_start_end(int start_lnum, int end_lnum) { curbuf->b_op_start.lnum = (linenr_T)start_lnum; curbuf->b_op_start.col = 0; curbuf->b_op_end.lnum = (linenr_T)end_lnum; curbuf->b_op_end.col = 0; }
char *nvim_do_sub_format_confirm_prompt(const char *sub) { const char *p = _("replace with %s? (y)es/(n)o/(a)ll/(q)uit/(l)ast/scroll up(^E)/down(^Y)"); vim_snprintf(IObuff, IOSIZE, p, sub); return xstrdup(IObuff); }
void nvim_do_sub_changed_lines(int first, int last, int xtra) { changed_lines(curbuf, (linenr_T)first, 0, (linenr_T)last, (linenr_T)xtra, false); }
void nvim_do_sub_save_pat(const char *pat, size_t patlen, int which_pat) { save_re_pat(which_pat, (char *)pat, patlen, rs_magic_isset()); add_to_history(HIST_SEARCH, (char *)pat, patlen, true, NUL); }
void nvim_do_sub_set_replacement(const char *sub) { rs_sub_set_replacement(xstrdup(sub), (uint64_t)os_time(), NULL); }
int nvim_regmmatch_startpos0_lnum(void *rm) { return (int)((regmmatch_T *)rm)->startpos[0].lnum; }
int nvim_regmmatch_startpos0_col(void *rm) { return (int)((regmmatch_T *)rm)->startpos[0].col; }
int nvim_regmmatch_endpos0_lnum(void *rm) { return (int)((regmmatch_T *)rm)->endpos[0].lnum; }
int nvim_regmmatch_endpos0_col(void *rm) { return (int)((regmmatch_T *)rm)->endpos[0].col; }
void nvim_regmmatch_set_rmm_ic(void *rm, int ic) { ((regmmatch_T *)rm)->rmm_ic = (bool)ic; }
int nvim_regmmatch_get_rmm_ic(void *rm) { return ((regmmatch_T *)rm)->rmm_ic ? 1 : 0; }
int nvim_regmmatch_re_multiline(void *rm) { return re_multiline(((regmmatch_T *)rm)->regprog) ? 1 : 0; }
void *nvim_do_sub_search_regcomp(const char *pat, size_t patlen, int which_pat, int flags) { regmmatch_T *rm = xmalloc(sizeof(regmmatch_T)); memset(rm, 0, sizeof(*rm)); if (search_regcomp((char *)pat, patlen, NULL, RE_SUBST, which_pat, flags, rm) == FAIL) { xfree(rm); return NULL; } return rm; }
int nvim_do_sub_vim_regexec_multi(void *rm, int lnum, int col) { return vim_regexec_multi((regmmatch_T *)rm, curwin, curbuf, (linenr_T)lnum, (colnr_T)col, NULL, NULL); }
int nvim_do_sub_vim_regsub_multi(void *rm, int source_lnum, const char *sub, char *dest, int destlen, int flags) { return vim_regsub_multi((regmmatch_T *)rm, (linenr_T)source_lnum, (char *)sub, dest, destlen, flags); }
int nvim_ecmd_curbuf_get_b_flags(void) { return curbuf->b_flags; }
int nvim_ecmd_curbuf_get_terminal(void) { return curbuf->terminal != NULL ? 1 : 0; }
void nvim_ecmd_curbuf_set_did_filetype(int val) { curbuf->b_did_filetype = (bool)val; }
void nvim_ecmd_curbuf_clear_flags(int mask) { curbuf->b_flags &= ~mask; }
void nvim_ecmd_curbuf_set_flags(int mask) { curbuf->b_flags |= mask; }
void nvim_ecmd_curbuf_set_last_used(void) { curbuf->b_last_used = time(NULL); }
int nvim_ecmd_curbuf_get_kmap_state(void) { return curbuf->b_kmap_state; }
int nvim_ecmd_curbuf_get_help(void) { return curbuf->b_help ? 1 : 0; }
void nvim_ecmd_curbuf_clear_op_marks(void) { curbuf->b_op_start.lnum = 0; curbuf->b_op_end.lnum = 0; }
int nvim_ecmd_curwin_get_cursor_col(void) { return (int)curwin->w_cursor.col; }
void nvim_ecmd_curwin_set_coladd_curswant(void) { curwin->w_cursor.coladd = 0; curwin->w_set_curswant = true; }
int nvim_ecmd_curwin_get_topline(void) { return (int)curwin->w_topline; }
int nvim_ecmd_curwin_get_alt_fnum(void) { return curwin->w_alt_fnum; }
void nvim_ecmd_curwin_set_pcmark(int lnum, int col) { curwin->w_pcmark.lnum = (linenr_T)lnum; curwin->w_pcmark.col = (colnr_T)col; }
int nvim_ecmd_curwin_get_effective_p_so(void) { return (int)(curwin->w_p_so >= 0 ? curwin->w_p_so : p_so); }
void nvim_ecmd_curwin_set_effective_p_so(int val) { if (curwin->w_p_so >= 0) { curwin->w_p_so = val; } else { p_so = val; } }
void nvim_ecmd_curwin_diff_spell_state(int *diff_out, int *spell_out, int *spl_empty_out) { *diff_out = curwin->w_p_diff ? 1 : 0; *spell_out = curwin->w_p_spell ? 1 : 0; *spl_empty_out = *curwin->w_s->b_p_spl == NUL ? 1 : 0; }
void nvim_ecmd_curwin_set_scbind_pos_from_topline(void) { curwin->w_scbind_pos = plines_m_win_fill(curwin, 1, curwin->w_topline); }
int nvim_ecmd_curwin_buf_is_null(void) { return curwin->w_buffer == NULL ? 1 : 0; }
int nvim_ecmd_curwin_ws_is_own_buf(void) { return curwin->w_s == &(curwin->w_buffer->b_s) ? 1 : 0; }
int nvim_ecmd_buf_has_memfile(buf_T *buf) { return buf->b_ml.ml_mfp != NULL ? 1 : 0; }
int nvim_ecmd_buf_get_locked_split(buf_T *buf) { return buf->b_locked_split; }
void nvim_ecmd_buf_inc_nwindows(buf_T *buf) { buf->b_nwindows++; }
void nvim_ecmd_buf_inc_locked(buf_T *buf) { buf->b_locked++; }
void nvim_ecmd_buf_dec_locked(buf_T *buf) { buf->b_locked--; }
int nvim_ecmd_buf_is_curbuf(buf_T *buf) { return buf == curbuf ? 1 : 0; }
void nvim_ecmd_set_curbuf(buf_T *buf) { curwin->w_buffer = buf; curbuf = buf; curbuf->b_nwindows++; }
int nvim_ecmd_win_buf_is_null(win_T *win) { return win->w_buffer == NULL ? 1 : 0; }
void nvim_ecmd_win_restore_buffer(win_T *win, buf_T *buf) { win->w_buffer = buf; }
void nvim_ecmd_win_set_locked(win_T *win, int val) { win->w_locked = (bool)val; }
void *nvim_ecmd_new_bufref(void) { return xcalloc(1, sizeof(bufref_T)); }
void nvim_ecmd_set_bufref_to_curbuf(void *ref) { set_bufref((bufref_T *)ref, curbuf); }
int nvim_ecmd_bufref_is_curbuf(void *ref) { return ((bufref_T *)ref)->br_buf == curbuf ? 1 : 0; }
void nvim_ecmd_au_new_curbuf_set(buf_T *buf) { set_bufref(&au_new_curbuf, buf); }
int nvim_ecmd_au_new_curbuf_valid(void) { return bufref_valid(&au_new_curbuf) ? 1 : 0; }
buf_T *nvim_ecmd_au_new_curbuf_get_buf(void) { return au_new_curbuf.br_buf; }
void *nvim_ecmd_au_new_curbuf_save(void) { bufref_T *saved = xmalloc(sizeof(bufref_T)); *saved = au_new_curbuf; return saved; }
void nvim_ecmd_au_new_curbuf_restore(void *saved) { au_new_curbuf = *(bufref_T *)saved; }
int nvim_ecmd_cursor_eq(int lnum, int col) { pos_T orig = { .lnum = (linenr_T)lnum, .col = (colnr_T)col }; return equalpos(curwin->w_cursor, orig) ? 1 : 0; }
int nvim_ecmd_cursor_col_skipwhite(void) { const char *text = get_cursor_line_ptr(); return (int)(skipwhite(text) - text); }
void nvim_ecmd_apply_autocmds_bufenter_retval(int *retval) { apply_autocmds_retval(EVENT_BUFENTER, NULL, NULL, false, curbuf, retval); }
void nvim_ecmd_apply_autocmds_bufwinenter_retval(int *retval) { apply_autocmds_retval(EVENT_BUFWINENTER, NULL, NULL, false, curbuf, retval); }
int nvim_ecmd_cmdwin_buf_is_nonnull(void) { return cmdwin_buf != NULL ? 1 : 0; }
int nvim_ecmd_cmdwin_get_type(void) { return cmdwin_type; }
win_T *nvim_ecmd_cmdwin_get_win(void) { return cmdwin_win; }
win_T *nvim_ecmd_cmdwin_get_old_curwin(void) { return cmdwin_old_curwin; }
void nvim_ecmd_cmdwin_clear(void) { cmdwin_type = 0; cmdwin_win = NULL; cmdwin_old_curwin = NULL; }
void nvim_ecmd_cmdwin_restore(int type, win_T *win, win_T *old_curwin) { cmdwin_type = type; cmdwin_win = win; cmdwin_old_curwin = old_curwin; }
void nvim_ecmd_buflist_findfmark(buf_T *buf, int *lnum, int *col) { const pos_T *pos = &buflist_findfmark(buf)->mark; *lnum = (int)pos->lnum; *col = (int)pos->col; }
int nvim_ecmd_bufref_has_terminal(void *ref) { bufref_T *br = (bufref_T *)ref; return (bufref_valid(br) && br->br_buf->terminal != NULL) ? 1 : 0; }
void nvim_ecmd_bufref_terminal_check_size(void *ref) { terminal_check_size(((bufref_T *)ref)->br_buf->terminal); }
int nvim_ecmd_bufref_valid_is_curbuf(void *ref) { bufref_T *br = (bufref_T *)ref; return (bufref_valid(br) && br->br_buf == curbuf) ? 1 : 0; }
void nvim_ecmd_curbuf_terminal_check_size(void) { if (curbuf->terminal != NULL) { terminal_check_size(curbuf->terminal); } }
void nvim_ecmd_handle_swap_exists(void *old_curbuf_ref) { handle_swap_exists((bufref_T *)old_curbuf_ref); }
#ifdef CASE_INSENSITIVE_FILENAME
void nvim_ecmd_path_fix_case(char *sfname) { path_fix_case(sfname); }
int nvim_ecmd_has_case_insensitive_filename(void) { return 1; }
#else
void nvim_ecmd_path_fix_case(char *sfname) { (void)sfname; }
int nvim_ecmd_has_case_insensitive_filename(void) { return 0; }
#endif
void nvim_ecmd_fold_update_all_curbuf_wins(void)
{
  FOR_ALL_TAB_WINDOWS(tp, win) {
    if (win->w_buffer == curbuf) {
      rs_foldUpdateAll(win);
    }
  }
}
const char *nvim_ecmd_eap_get_do_ecmd_cmd(exarg_T *eap) { return eap != NULL ? eap->do_ecmd_cmd : NULL; }
void nvim_ecmd_emsg_closing_buffer(void) { emsg(_(e_cannot_switch_to_a_closing_buffer)); }
int nvim_ecmd_should_dec_nwindows_on_locked(win_T *oldwin) { return (oldwin == NULL && curwin->w_buffer != NULL && curwin->w_buffer->b_nwindows > 1) ? 1 : 0; }
void nvim_ecmd_curwin_set_ws_to_buf(buf_T *buf) { curwin->w_s = &(buf->b_s); }
void nvim_ecmd_dec_curwin_buf_nwindows_safe(void) { if (curwin->w_buffer != NULL && curwin->w_buffer->b_nwindows > 1) { curwin->w_buffer->b_nwindows--; } }
void nvim_cpi_get_col_info(int *col1, int *vcol1, int *linelen, int *tabsize)
{
  char *p = get_cursor_line_ptr();
  validate_virtcol(curwin);
  *col1 = (int)curwin->w_cursor.col + 1;
  *vcol1 = (int)curwin->w_virtcol + 1;
  *linelen = get_cursor_line_len();
  *tabsize = linetabsize_str(p);
}
char *nvim_cpi_save_clear_p_shm(void) { char *saved = p_shm; p_shm = ""; return saved; }
void nvim_cpi_restore_p_shm(char *saved) { p_shm = saved; }
void nvim_cpi_getvcols_no_sbr(int min_lnum, int min_col, int max_lnum, int max_col,
                              int *out_start_vcol, int *out_end_vcol)
{
  pos_T min_pos = { .lnum = min_lnum, .col = min_col, .coladd = 0 };
  pos_T max_pos = { .lnum = max_lnum, .col = max_col, .coladd = 0 };
  char *const saved_sbr = p_sbr;
  char *const saved_w_sbr = curwin->w_p_sbr;
  p_sbr = empty_string_option;
  curwin->w_p_sbr = empty_string_option;
  colnr_T start_vcol = 0, end_vcol = 0;
  getvcols(curwin, &min_pos, &max_pos, &start_vcol, &end_vcol);
  p_sbr = saved_sbr;
  curwin->w_p_sbr = saved_w_sbr;
  *out_start_vcol = (int)start_vcol;
  *out_end_vcol = (int)end_vcol;
}
void nvim_cpi_block_prep_text(int lnum, int start_vcol, int end_vcol,
                              const char **textstart_out, int *textlen_out)
{
  oparg_T oparg;
  memset(&oparg, 0, sizeof(oparg));
  oparg.is_VIsual = true;
  oparg.motion_type = kMTBlockWise;
  oparg.op_type = OP_NOP;
  oparg.start_vcol = (colnr_T)start_vcol;
  oparg.end_vcol = (colnr_T)end_vcol;
  struct block_def bd;
  virtual_op = virtual_active(curwin);
  block_prep(&oparg, &bd, (linenr_T)lnum, false);
  virtual_op = kNone;
  *textstart_out = bd.textstart;
  *textlen_out = bd.textlen;
}

// Phase 1: prep_exarg / set_forced_fenc exarg accessors
int nvim_exarg_get_force_enc(const exarg_T *eap) { return eap->force_enc; }
const char *nvim_exarg_get_cmd_ptr(const exarg_T *eap) { return eap->cmd; }
void nvim_exarg_set_cmd(exarg_T *eap, char *cmd) { eap->cmd = cmd; }
void nvim_exarg_set_force_enc(exarg_T *eap, int val) { eap->force_enc = val; }
void nvim_exarg_set_bad_char(exarg_T *eap, int val) { eap->bad_char = val; }
void nvim_exarg_set_force_ff(exarg_T *eap, int val) { eap->force_ff = val; }
void nvim_exarg_set_force_bin(exarg_T *eap, int val) { eap->force_bin = val; }
void nvim_exarg_set_read_edit(exarg_T *eap, int val) { eap->read_edit = val; }
void nvim_exarg_set_forceit(exarg_T *eap, int val) { eap->forceit = (bool)val; }

// cmdhist Phase 4: shims for symbols that were in cmdhist.c, now relocated here
// after cmdhist.c is deleted. These replace the identical functions in cmdhist.c.

// Compile-time assertions verifying constants match Rust-side expectations
#include "nvim/eval/typval_defs.h"
_Static_assert(sizeof(histentry_T) == 40,
               "sizeof(histentry_T) changed - update Rust HistoryEntry in state.rs");
_Static_assert(HIST_COUNT == 5, "HIST_COUNT changed - update Rust HIST_COUNT");
_Static_assert(CMOD_KEEPPATTERNS == 0x1000, "CMOD_KEEPPATTERNS changed - update Rust constant");
_Static_assert(RE_MAGIC == 1, "RE_MAGIC changed - update Rust constant");
_Static_assert(RE_STRING == 2, "RE_STRING changed - update Rust constant");
_Static_assert(VAR_UNKNOWN == 0, "VAR_UNKNOWN changed - update Rust constant");
_Static_assert(VAR_NUMBER == 1, "VAR_NUMBER changed - update Rust constant");
_Static_assert(VAR_STRING == 2, "VAR_STRING changed - update Rust constant");
_Static_assert(NUMBUFLEN == 65, "NUMBUFLEN changed - update Rust constant");
_Static_assert(IOSIZE == 1025, "IOSIZE changed - update Rust constant");

// Global accessor that requires C cmdmod struct access
int nvim_cmdhist_get_cmdmod_cmod_flags(void) { return (int)cmdmod.cmod_flags; }

// Regexp wrappers (opaque regmatch_T heap-allocated for Rust)
void *nvim_cmdhist_regcomp(const char *str, int flags)
{
  regmatch_T *rm = xmalloc(sizeof(regmatch_T));
  rm->regprog = vim_regcomp((char *)str, flags);
  if (rm->regprog == NULL) {
    xfree(rm);
    return NULL;
  }
  rm->rm_ic = false;  // always match case
  return (void *)rm;
}
int nvim_cmdhist_regexec(void *rm, const char *str)
{
  return vim_regexec((regmatch_T *)rm, (char *)str, 0);
}
void nvim_cmdhist_regfree(void *rm)
{
  regmatch_T *r = (regmatch_T *)rm;
  vim_regfree(r->regprog);
  xfree(r);
}

// typval_T wrappers
const char *nvim_cmdhist_tv_get_string_chk(typval_T *tv) { return tv_get_string_chk(tv); }
const char *nvim_cmdhist_tv_get_string_buf(typval_T *tv, char *buf) { return tv_get_string_buf(tv, buf); }
int64_t nvim_cmdhist_tv_get_number(typval_T *tv) { return (int64_t)tv_get_number(tv); }
int64_t nvim_cmdhist_tv_get_number_chk(typval_T *tv, void *error) { return (int64_t)tv_get_number_chk(tv, (bool *)error); }
int nvim_cmdhist_tv_get_type(typval_T *tv) { return tv->v_type; }
typval_T *nvim_cmdhist_tv_idx(typval_T *tv, int idx) { return &tv[idx]; }
void nvim_cmdhist_rettv_set_number(typval_T *rettv, int64_t val) { rettv->vval.v_number = (varnumber_T)val; }
void nvim_cmdhist_rettv_set_string(typval_T *rettv, char *s) { rettv->vval.v_string = s; }
void nvim_cmdhist_rettv_set_type(typval_T *rettv, int typ) { rettv->v_type = (VarType)typ; }
size_t nvim_cmdhist_strlen(const char *s) { return strlen(s); }

// msg_outtrans adapter (Rust cannot call 3-arg C function directly)
void nvim_cmdhist_msg_outtrans(const char *buf) { msg_outtrans(buf, 0, false); }

// History display formatting (uses IObuff global and vim_snprintf)
char *nvim_cmdhist_format_hist_header(const char *name)
{
  vim_snprintf(IObuff, IOSIZE, "\n      #  %s history", name);
  return IObuff;
}
int nvim_cmdhist_format_hist_entry(int is_current, int hisnum_val)
{
  return snprintf(IObuff, IOSIZE, "%c%6d  ", is_current ? '>' : ' ', hisnum_val);
}
char *nvim_cmdhist_get_IObuff(void) { return IObuff; }

// Error messages (gettext wrappers)
void nvim_cmdhist_semsg_trailing_arg(const char *s) { semsg(_(e_trailing_arg), s); }
void nvim_cmdhist_semsg_val_too_large(const char *s) { semsg(_(e_val_too_large), s); }
void nvim_cmdhist_msg_history_zero(void) { msg(_("'history' option is zero"), 0); }

// exarg_T / expand_T field accessors
char *nvim_cmdhist_eap_get_arg(exarg_T *eap) { return eap->arg; }
void nvim_cmdhist_xp_buf_set(expand_T *xp, int idx, char c) { xp->xp_buf[idx] = c; }
char *nvim_cmdhist_xp_buf_ptr(expand_T *xp) { return xp->xp_buf; }

