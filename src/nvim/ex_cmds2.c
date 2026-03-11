/// @file ex_cmds2.c
///
/// Some more functions for command line commands

#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

#include "nvim/arglist.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/bufwrite.h"
#include "nvim/change.h"
#include "nvim/channel.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
// Phase 12: rs_eval_call_provider replaces the C eval_call_provider wrapper
extern void rs_eval_call_provider(const char *provider, const char *method,
                                  list_T *arguments, bool discard, typval_T *out_rettv);

// Rust-implemented win/tabpage validity checkers (exported as rs_* symbols)
extern int rs_win_valid(win_T *win);
extern int rs_valid_tabpage(tabpage_T *tpc);
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds2.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_getln.h"
#include "nvim/fileio.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/option_vars.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/quickfix.h"
#include "nvim/runtime.h"
#include "nvim/types_defs.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

#include "ex_cmds2.c.generated.h"

// =============================================================================
// Static assertions for constants used by Rust
// =============================================================================

_Static_assert(CCGD_AW == 1, "CCGD_AW mismatch");
_Static_assert(CCGD_MULTWIN == 2, "CCGD_MULTWIN mismatch");
_Static_assert(CCGD_FORCEIT == 4, "CCGD_FORCEIT mismatch");
_Static_assert(CCGD_ALLBUF == 8, "CCGD_ALLBUF mismatch");
_Static_assert(CCGD_EXCMD == 16, "CCGD_EXCMD mismatch");
_Static_assert(OK == 1, "OK mismatch");
_Static_assert(FAIL == 0, "FAIL mismatch");
_Static_assert(MAXPATHL >= 4096, "MAXPATHL too small");
_Static_assert(CMOD_CONFIRM == 0x0080, "CMOD_CONFIRM mismatch");
_Static_assert(HLF_W == 26, "HLF_W mismatch");
_Static_assert(DIALOG_MSG_SIZE == 1000, "DIALOG_MSG_SIZE mismatch");
_Static_assert(VIM_YES == 2, "VIM_YES mismatch");
_Static_assert(VIM_NO == 3, "VIM_NO mismatch");
_Static_assert(VIM_CANCEL == 4, "VIM_CANCEL mismatch");
_Static_assert(VIM_ALL == 5, "VIM_ALL mismatch");
_Static_assert(VIM_DISCARDALL == 6, "VIM_DISCARDALL mismatch");


// =============================================================================
// C accessor functions for Rust (script host)
// =============================================================================

char *nvim_ex2_eap_get_arg(exarg_T *eap)
{
  return eap->arg;
}

linenr_T nvim_ex2_eap_get_line1(exarg_T *eap)
{
  return eap->line1;
}

linenr_T nvim_ex2_eap_get_line2(exarg_T *eap)
{
  return eap->line2;
}

int nvim_ex2_eap_get_skip(exarg_T *eap)
{
  return eap->skip;
}

char *nvim_ex2_script_get(exarg_T *eap, size_t *lenp)
{
  return script_get(eap, lenp);
}

list_T *nvim_ex2_tv_list_alloc(ptrdiff_t len)
{
  return tv_list_alloc(len);
}

void nvim_ex2_tv_list_append_allocated_string(list_T *l, char *s)
{
  tv_list_append_allocated_string(l, s);
}

void nvim_ex2_tv_list_append_number(list_T *l, varnumber_T n)
{
  tv_list_append_number(l, n);
}

void nvim_ex2_tv_list_append_string(list_T *l, const char *s, ssize_t len)
{
  tv_list_append_string(l, s, len);
}

void nvim_ex2_eval_call_provider(char *provider, char *method, list_T *arguments, bool discard)
{
  typval_T rettv;
  rs_eval_call_provider(provider, method, arguments, discard, &rettv);
}

int nvim_ex2_vim_fullname(const char *fname, char *buf, size_t len, bool force)
{
  return vim_FullName(fname, buf, len, force);
}

// =============================================================================
// C accessor functions for Rust (autowrite / buffer check)
// =============================================================================

const char *nvim_ex2_buf_get_ffname(buf_T *buf)
{
  return buf->b_ffname;
}

const char *nvim_ex2_buf_get_fname(buf_T *buf)
{
  return buf->b_fname;
}

int nvim_ex2_buf_get_b_p_ro(buf_T *buf)
{
  return buf->b_p_ro;
}

int nvim_ex2_buf_get_nwindows(buf_T *buf)
{
  return buf->b_nwindows;
}

linenr_T nvim_ex2_buf_get_ml_line_count(buf_T *buf)
{
  return buf->b_ml.ml_line_count;
}

buf_T *nvim_ex2_get_firstbuf(void)
{
  return firstbuf;
}

buf_T *nvim_ex2_buf_next(buf_T *buf)
{
  return buf->b_next;
}

bool nvim_ex2_get_p_aw(void)
{
  return p_aw;
}

bool nvim_ex2_get_p_awa(void)
{
  return p_awa;
}

bool nvim_ex2_get_p_write(void)
{
  return p_write;
}

bool nvim_ex2_get_p_confirm(void)
{
  return p_confirm;
}

buf_T *nvim_ex2_get_curbuf(void)
{
  return curbuf;
}

uint32_t nvim_ex2_get_cmod_flags(void)
{
  return cmdmod.cmod_flags;
}

bufref_T *nvim_ex2_bufref_create(buf_T *buf)
{
  bufref_T *br = xmalloc(sizeof(bufref_T));
  set_bufref(br, buf);
  return br;
}

bool nvim_ex2_bufref_valid(bufref_T *br)
{
  return bufref_valid(br);
}

void nvim_ex2_bufref_free(bufref_T *br)
{
  xfree(br);
}

bool nvim_ex2_bufIsChanged(buf_T *buf)
{
  return bufIsChanged(buf);
}

bool nvim_ex2_bt_dontwrite(buf_T *buf)
{
  return bt_dontwrite(buf);
}

bool nvim_ex2_buf_hide(buf_T *buf)
{
  return buf_hide(buf);
}

int nvim_ex2_buf_write(buf_T *buf, const char *ffname, const char *fname,
                       linenr_T start, linenr_T end, exarg_T *eap,
                       bool append, bool forceit, bool reset_changed,
                       bool filtering)
{
  return buf_write(buf, (char *)ffname, (char *)fname, start, end, eap,
                   append, forceit, reset_changed, filtering);
}

void nvim_ex2_msg_source(int hl)
{
  msg_source(hl);
}

void nvim_ex2_msg(const char *s, int attr)
{
  msg(s, attr);
}

void nvim_ex2_no_write_message(void)
{
  no_write_message();
}

void nvim_ex2_no_write_message_nobang(buf_T *buf)
{
  no_write_message_nobang(buf);
}

bool nvim_ex2_emsg(const char *s)
{
  return emsg(s);
}

const char *nvim_ex2_gettext(const char *s)
{
  return _(s);
}

// =============================================================================
// C accessor functions for Rust (dialog)
// =============================================================================

int nvim_ex2_buf_get_fnum(buf_T *buf)
{
  return buf->b_fnum;
}

void nvim_ex2_dialog_msg(char *buff, char *format, char *fname)
{
  dialog_msg(buff, format, fname);
}

int nvim_ex2_vim_dialog_yesnocancel(int type, char *title, char *message, int dflt)
{
  return vim_dialog_yesnocancel(type, title, message, dflt);
}

int nvim_ex2_vim_dialog_yesnoallcancel(int type, char *title, char *message, int dflt)
{
  return vim_dialog_yesnoallcancel(type, title, message, dflt);
}

int nvim_ex2_check_overwrite(buf_T *buf, const char *fname, const char *ffname)
{
  exarg_T ea = {
    .append = false,
    .forceit = false,
  };
  return check_overwrite(&ea, buf, (char *)fname, (char *)ffname, false);
}

void nvim_ex2_unchanged(buf_T *buf, bool ff, bool always_inc_changedtick)
{
  unchanged(buf, ff, always_inc_changedtick);
}

void nvim_ex2_buf_set_name(int fnum, const char *name)
{
  buf_set_name(fnum, (char *)name);
}

void nvim_ex2_buf_clear_names(buf_T *buf)
{
  XFREE_CLEAR(buf->b_ffname);
  XFREE_CLEAR(buf->b_sfname);
}

void nvim_ex2_buf_set_fname_null(buf_T *buf)
{
  buf->b_fname = NULL;
}

// =============================================================================
// C accessor functions for Rust (check_changed_any / ex_checktime)
// =============================================================================

_Static_assert(DOBUF_UNLOAD == 2, "DOBUF_UNLOAD mismatch");
_Static_assert(DOBUF_GOTO == 0, "DOBUF_GOTO mismatch");

win_T *nvim_ex2_get_firstwin(void)
{
  return firstwin;
}

win_T *nvim_ex2_win_next(win_T *win)
{
  return win->w_next;
}

buf_T *nvim_ex2_win_get_buffer(win_T *win)
{
  return win->w_buffer;
}

tabpage_T *nvim_ex2_get_curtab(void)
{
  return curtab;
}

tabpage_T *nvim_ex2_get_first_tabpage(void)
{
  return first_tabpage;
}

tabpage_T *nvim_ex2_tp_next(tabpage_T *tp)
{
  return tp->tp_next;
}

win_T *nvim_ex2_tp_firstwin(tabpage_T *tp)
{
  if (tp == curtab) {
    return firstwin;
  }
  return tp->tp_firstwin;
}

int nvim_ex2_get_vgetc_busy(void)
{
  return vgetc_busy;
}

int nvim_ex2_get_msg_row(void)
{
  return msg_row;
}

void nvim_ex2_set_msg_row(int val)
{
  msg_row = val;
}

int nvim_ex2_get_msg_col(void)
{
  return msg_col;
}

void nvim_ex2_set_msg_col(int val)
{
  msg_col = val;
}

void nvim_ex2_set_msg_didout(bool val)
{
  msg_didout = val;
}

bool nvim_ex2_get_msg_didany(void)
{
  return msg_didany;
}

int nvim_ex2_get_cmdline_row(void)
{
  return cmdline_row;
}

int nvim_ex2_get_no_wait_return(void)
{
  return no_wait_return;
}

void nvim_ex2_set_no_wait_return(int val)
{
  no_wait_return = val;
}

bool nvim_ex2_get_exiting(void)
{
  return exiting;
}

void nvim_ex2_set_exiting(bool val)
{
  exiting = val;
}

buf_T *nvim_ex2_buflist_findnr(int nr)
{
  return buflist_findnr(nr);
}

void nvim_ex2_set_curbuf(buf_T *buf, int action, bool prevbuf)
{
  set_curbuf(buf, action, prevbuf);
}

const char *nvim_ex2_buf_spname(buf_T *buf)
{
  return buf_spname(buf);
}

void nvim_ex2_goto_tabpage_win(tabpage_T *tp, win_T *wp)
{
  goto_tabpage_win(tp, wp);
}

void nvim_ex2_wait_return(bool redraw)
{
  wait_return(redraw);
}

bool nvim_ex2_buf_has_running_job(buf_T *buf)
{
  return buf->terminal && channel_job_running((uint64_t)buf->b_p_channel);
}

bool nvim_ex2_semsg(const char *fmt, const char *arg)
{
  return semsg(fmt, arg);
}

int nvim_ex2_eap_get_addr_count(exarg_T *eap)
{
  return eap->addr_count;
}

int nvim_ex2_get_no_check_timestamps(void)
{
  return no_check_timestamps;
}

void nvim_ex2_set_no_check_timestamps(int val)
{
  no_check_timestamps = val;
}

void nvim_ex2_check_timestamps(bool focus)
{
  check_timestamps(focus);
}

void nvim_ex2_buf_check_timestamp(buf_T *buf)
{
  (void)buf_check_timestamp(buf);
}

// =============================================================================
// C accessor functions for Rust (compiler)
// =============================================================================

_Static_assert(DIP_ALL == 0x01, "DIP_ALL mismatch");

int nvim_ex2_eap_get_forceit(exarg_T *eap)
{
  return eap->forceit;
}

void nvim_ex2_do_cmdline_cmd(const char *cmd)
{
  do_cmdline_cmd(cmd);
}

char *nvim_ex2_get_var_value(const char *name)
{
  return get_var_value(name);
}

void nvim_ex2_set_internal_string_var(const char *name, const char *val)
{
  set_internal_string_var(name, (char *)val);
}

void nvim_ex2_do_unlet(const char *name, size_t name_len, bool forceit)
{
  do_unlet(name, name_len, forceit);
}

int nvim_ex2_source_runtime_vim_lua(const char *name, int flags)
{
  return source_runtime_vim_lua((char *)name, flags);
}

char *nvim_ex2_xstrdup(const char *s)
{
  return xstrdup(s);
}

void nvim_ex2_xfree(void *p)
{
  xfree(p);
}

char *nvim_ex2_xmalloc(size_t size)
{
  return xmalloc(size);
}

void nvim_ex2_snprintf(char *buf, size_t size, const char *fmt, const char *arg)
{
  snprintf(buf, size, fmt, arg);
}

size_t nvim_ex2_strlen(const char *s)
{
  return strlen(s);
}

// =============================================================================
// C accessor functions for Rust (drop)
// =============================================================================

_Static_assert(ML_EMPTY == 0x01, "ML_EMPTY mismatch");
_Static_assert(DOCMD_VERBOSE == 0x01, "DOCMD_VERBOSE mismatch");
_Static_assert(DOCMD_NOWAIT == 0x02, "DOCMD_NOWAIT mismatch");

void nvim_ex2_set_arglist(char *arg)
{
  set_arglist(arg);
}

int nvim_ex2_get_argcount(void)
{
  return ARGCOUNT;
}

int nvim_ex2_get_arglist_fnum(int idx)
{
  return ARGLIST[idx].ae_fnum;
}

void nvim_ex2_ex_all(exarg_T *eap)
{
  ex_all(eap);
}

void nvim_ex2_ex_rewind(exarg_T *eap)
{
  ex_rewind(eap);
}

win_T *nvim_ex2_get_curwin(void)
{
  return curwin;
}

int nvim_ex2_curwin_get_arg_idx(void)
{
  return curwin->w_arg_idx;
}

void nvim_ex2_curwin_set_arg_idx(int val)
{
  curwin->w_arg_idx = val;
}

int nvim_ex2_curbuf_get_b_p_ar(void)
{
  return curbuf->b_p_ar;
}

void nvim_ex2_curbuf_set_b_p_ar(int val)
{
  curbuf->b_p_ar = val;
}

int nvim_ex2_curbuf_get_ml_flags(void)
{
  return curbuf->b_ml.ml_flags;
}

char *nvim_ex2_eap_get_do_ecmd_cmd(exarg_T *eap)
{
  return eap->do_ecmd_cmd;
}

bool nvim_ex2_set_swapcommand(const char *cmd, int zero)
{
  return set_swapcommand(cmd, zero);
}

void nvim_ex2_do_cmdline(char *cmd, LineGetter getline_fn, void *cookie, int flags)
{
  do_cmdline(cmd, getline_fn, cookie, flags);
}

void nvim_ex2_clear_swapcommand(void)
{
  set_vim_var_string(VV_SWAPCOMMAND, NULL, -1);
}

void nvim_ex2_buf_check_timestamp_curbuf(void)
{
  (void)buf_check_timestamp(curbuf);
}

int nvim_ex2_get_emsg_off(void)
{
  return emsg_off;
}

void nvim_ex2_set_emsg_off(int val)
{
  emsg_off = val;
}

int nvim_ex2_get_cmod_tab(void)
{
  return cmdmod.cmod_tab;
}

void nvim_ex2_set_cmod_tab(int val)
{
  cmdmod.cmod_tab = val;
}

void nvim_ex2_eap_set_cmdidx(exarg_T *eap, int val)
{
  eap->cmdidx = (cmdidx_T)val;
}

void nvim_ex2_eap_set_cmd0(exarg_T *eap, int ch)
{
  eap->cmd[0] = (char)ch;
}

int nvim_ex2_get_cmd_sfirst(void)
{
  return (int)CMD_sfirst;
}

int nvim_ex2_get_cmd_first(void)
{
  return (int)CMD_first;
}

// =============================================================================
// C accessor functions for Rust (listdo)
// =============================================================================

_Static_assert(BF_SYN_SET == 0x200, "BF_SYN_SET mismatch");
_Static_assert(FORWARD == 1, "FORWARD mismatch");
_Static_assert(DOBUF_FIRST == 1, "DOBUF_FIRST mismatch");

int nvim_ex2_eap_get_cmdidx(exarg_T *eap)
{
  return (int)eap->cmdidx;
}

void *nvim_ex2_eap_get_ea_getline(exarg_T *eap)
{
  return (void *)eap->ea_getline;
}

void *nvim_ex2_eap_get_cookie(exarg_T *eap)
{
  return eap->cookie;
}

void nvim_ex2_eap_set_line2(exarg_T *eap, linenr_T val)
{
  eap->line2 = val;
}

int nvim_ex2_get_cmd_windo(void) { return (int)CMD_windo; }
int nvim_ex2_get_cmd_tabdo(void) { return (int)CMD_tabdo; }
int nvim_ex2_get_cmd_bufdo(void) { return (int)CMD_bufdo; }
int nvim_ex2_get_cmd_argdo(void) { return (int)CMD_argdo; }
int nvim_ex2_get_cmd_cdo(void) { return (int)CMD_cdo; }
int nvim_ex2_get_cmd_ldo(void) { return (int)CMD_ldo; }
int nvim_ex2_get_cmd_cfdo(void) { return (int)CMD_cfdo; }
int nvim_ex2_get_cmd_lfdo(void) { return (int)CMD_lfdo; }

bool nvim_ex2_get_curwin_w_p_wfb(void)
{
  return curwin->w_p_wfb;
}

win_T *nvim_ex2_get_prevwin(void)
{
  return prevwin;
}

bool nvim_ex2_win_get_w_p_wfb(win_T *win)
{
  return win->w_p_wfb;
}

void nvim_ex2_win_goto(win_T *win)
{
  win_goto(win);
}

int nvim_ex2_win_split(void)
{
  return win_split(0, 0);
}

void nvim_ex2_emsg_winfixbuf(void)
{
  emsg(_(e_winfixbuf_cannot_go_to_buffer));
}

void nvim_ex2_inc_msg_listdo_overwrite(void)
{
  msg_listdo_overwrite++;
}

void nvim_ex2_dec_msg_listdo_overwrite(void)
{
  msg_listdo_overwrite--;
}

char *nvim_ex2_au_event_disable_syntax(void)
{
  return au_event_disable(",Syntax");
}

void nvim_ex2_au_event_restore(char *save_ei)
{
  au_event_restore(save_ei);
}

bool nvim_ex2_win_valid(win_T *win)
{
  return rs_win_valid(win) != 0;
}

void nvim_ex2_buf_clear_bf_syn_set(buf_T *buf)
{
  buf->b_flags &= ~BF_SYN_SET;
}

int nvim_ex2_buf_get_b_flags(buf_T *buf)
{
  return buf->b_flags;
}

bool nvim_ex2_buf_get_b_p_bl(buf_T *buf)
{
  return buf->b_p_bl;
}

char *nvim_ex2_buf_get_b_p_syn(buf_T *buf)
{
  return buf->b_p_syn;
}

size_t nvim_ex2_qf_get_valid_size(exarg_T *eap)
{
  return qf_get_valid_size(eap);
}

size_t nvim_ex2_qf_get_cur_idx(exarg_T *eap)
{
  return qf_get_cur_idx(eap);
}

extern void rs_ex_cc(void *eap);
extern void rs_ex_cnext(void *eap);

void nvim_ex2_ex_cc(exarg_T *eap)
{
  rs_ex_cc(eap);
}

void nvim_ex2_ex_cnext(exarg_T *eap)
{
  rs_ex_cnext(eap);
}

bool nvim_ex2_get_got_int(void)
{
  return got_int;
}

void nvim_ex2_set_listcmd_busy(bool val)
{
  listcmd_busy = val;
}

void nvim_ex2_setpcmark(void)
{
  setpcmark();
}

bool nvim_ex2_editing_arg_idx(void)
{
  return editing_arg_idx(curwin);
}

void nvim_ex2_do_argfile(exarg_T *eap, int idx)
{
  do_argfile(eap, idx);
}

void nvim_ex2_goto_buffer(exarg_T *eap, int start, int dir, int fnum)
{
  goto_buffer(eap, start, dir, fnum);
}

bool nvim_ex2_win_get_w_floating(win_T *win)
{
  return win->w_floating;
}

bool nvim_ex2_win_get_w_config_hide(win_T *win)
{
  return win->w_config.hide;
}

bool nvim_ex2_win_get_w_config_focusable(win_T *win)
{
  return win->w_config.focusable;
}

bool nvim_ex2_valid_tabpage(tabpage_T *tp)
{
  return rs_valid_tabpage(tp) != 0;
}

void nvim_ex2_goto_tabpage_tp(tabpage_T *tp, bool trigger_enter, bool trigger_leave)
{
  goto_tabpage_tp(tp, trigger_enter, trigger_leave);
}

void nvim_ex2_validate_cursor(void)
{
  validate_cursor(curwin);
}

bool nvim_ex2_curwin_get_w_p_scb(void)
{
  return curwin->w_p_scb;
}

void nvim_ex2_do_check_scrollbind(bool check)
{
  do_check_scrollbind(check);
}

/// Restore syntax autocommands after listdo.
/// This handles the aco_save_T stack-local struct by keeping it entirely in C.
void nvim_ex2_listdo_restore_syntax(char *save_ei)
{
  buf_T *bnext;
  aco_save_T aco;

  au_event_restore(save_ei);

  for (buf_T *buf = firstbuf; buf != NULL; buf = bnext) {
    bnext = buf->b_next;
    if (buf->b_nwindows > 0 && (buf->b_flags & BF_SYN_SET)) {
      buf->b_flags &= ~BF_SYN_SET;

      // buffer was opened while Syntax autocommands were disabled,
      // need to trigger them now.
      if (buf == curbuf) {
        apply_autocmds(EVENT_SYNTAX, curbuf->b_p_syn, curbuf->b_fname, true,
                       curbuf);
      } else {
        aucmd_prepbuf(&aco, buf);
        apply_autocmds(EVENT_SYNTAX, buf->b_p_syn, buf->b_fname, true, buf);
        aucmd_restbuf(&aco);
      }

      // start over, in case autocommands messed things up.
      bnext = firstbuf;
    }
  }
}

