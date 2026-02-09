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
// Rust extern declarations — script host functions
// =============================================================================

extern void rs_ex_ruby(exarg_T *eap);
extern void rs_ex_rubyfile(exarg_T *eap);
extern void rs_ex_rubydo(exarg_T *eap);
extern void rs_ex_python3(exarg_T *eap);
extern void rs_ex_py3file(exarg_T *eap);
extern void rs_ex_pydo3(exarg_T *eap);
extern void rs_ex_perl(exarg_T *eap);
extern void rs_ex_perlfile(exarg_T *eap);
extern void rs_ex_perldo(exarg_T *eap);

// =============================================================================
// Rust extern declarations — autowrite / buffer check functions
// =============================================================================

extern int rs_check_fname(void);
extern int rs_buf_write_all(buf_T *buf, bool forceit);
extern int rs_autowrite(buf_T *buf, bool forceit);
extern void rs_autowrite_all(void);
extern bool rs_can_abandon(buf_T *buf, bool forceit);
extern bool rs_check_changed(buf_T *buf, int flags);

// =============================================================================
// Rust extern declarations — dialog functions
// =============================================================================

extern bool rs_dialog_close_terminal(buf_T *buf);
extern void rs_dialog_changed(buf_T *buf, bool checkall);

// =============================================================================
// Rust extern declarations — check_changed_any / ex_checktime
// =============================================================================

extern bool rs_check_changed_any(bool hidden, bool unload);
extern void rs_ex_checktime(exarg_T *eap);

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
  eval_call_provider(provider, method, arguments, discard);
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

void nvim_ex2_dialog_changed(buf_T *buf, bool checkall)
{
  dialog_changed(buf, checkall);
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

static const char e_compiler_not_supported_str[]
  = N_("E666: Compiler not supported: %s");

void ex_ruby(exarg_T *eap)
{
  rs_ex_ruby(eap);
}

void ex_rubyfile(exarg_T *eap)
{
  rs_ex_rubyfile(eap);
}

void ex_rubydo(exarg_T *eap)
{
  rs_ex_rubydo(eap);
}

void ex_python3(exarg_T *eap)
{
  rs_ex_python3(eap);
}

void ex_py3file(exarg_T *eap)
{
  rs_ex_py3file(eap);
}

void ex_pydo3(exarg_T *eap)
{
  rs_ex_pydo3(eap);
}

void ex_perl(exarg_T *eap)
{
  rs_ex_perl(eap);
}

void ex_perlfile(exarg_T *eap)
{
  rs_ex_perlfile(eap);
}

void ex_perldo(exarg_T *eap)
{
  rs_ex_perldo(eap);
}

/// If 'autowrite' option set, try to write the file.
/// Careful: autocommands may make "buf" invalid!
///
/// @return FAIL for failure, OK otherwise
int autowrite(buf_T *buf, bool forceit)
{
  return rs_autowrite(buf, forceit);
}

/// Flush all buffers, except the ones that are readonly or are never written.
void autowrite_all(void)
{
  rs_autowrite_all();
}

/// @return  true if buffer was changed and cannot be abandoned.
/// For flags use the CCGD_ values.
bool check_changed(buf_T *buf, int flags)
{
  return rs_check_changed(buf, flags);
}

/// Ask the user what to do when abandoning a changed buffer.
/// Must check 'write' option first!
///
/// @param buf
/// @param checkall may abandon all changed buffers
void dialog_changed(buf_T *buf, bool checkall)
{
  rs_dialog_changed(buf, checkall);
}

/// Ask the user whether to close the terminal buffer or not.
///
/// @param buf The terminal buffer.
/// @return bool Whether to close the buffer or not.
bool dialog_close_terminal(buf_T *buf)
{
  return rs_dialog_close_terminal(buf);
}

/// @return true if the buffer "buf" can be abandoned, either by making it
/// hidden, autowriting it or unloading it.
bool can_abandon(buf_T *buf, bool forceit)
{
  return rs_can_abandon(buf, forceit);
}

// add_bufnum moved to Rust (Vec-based instead of xmalloc array)

/// Check if any buffer was changed and cannot be abandoned.
/// That changed buffer becomes the current buffer.
/// When "unload" is true the current buffer is unloaded instead of making it
/// hidden.  This is used for ":q!".
///
/// @param[in] hidden specifies whether to check only hidden buffers.
/// @param[in] unload specifies whether to unload, instead of hide, the buffer.
///
/// @returns          true if any buffer is changed and cannot be abandoned
bool check_changed_any(bool hidden, bool unload)
{
  return rs_check_changed_any(hidden, unload);
}

/// @return  FAIL if there is no file name, OK if there is one.
///          Give error message for FAIL.
int check_fname(void)
{
  return rs_check_fname();
}

/// Flush the contents of a buffer, unless it has no file name.
///
/// @return  FAIL for failure, OK otherwise
int buf_write_all(buf_T *buf, bool forceit)
{
  return rs_buf_write_all(buf, forceit);
}

/// ":argdo", ":windo", ":bufdo", ":tabdo", ":cdo", ":ldo", ":cfdo" and ":lfdo"
void ex_listdo(exarg_T *eap)
{
  if (curwin->w_p_wfb) {
    if ((eap->cmdidx == CMD_ldo || eap->cmdidx == CMD_lfdo) && !eap->forceit) {
      // Disallow :ldo if 'winfixbuf' is applied
      emsg(_(e_winfixbuf_cannot_go_to_buffer));
      return;
    }

    if (win_valid(prevwin) && !prevwin->w_p_wfb) {
      // 'winfixbuf' is set; attempt to change to a window without it.
      win_goto(prevwin);
    }
    if (curwin->w_p_wfb) {
      // Split the window, which will be 'nowinfixbuf', and set curwin to that
      (void)win_split(0, 0);

      if (curwin->w_p_wfb) {
        // Autocommands set 'winfixbuf' or sent us to another window
        // with it set, or we failed to split the window. Give up.
        emsg(_(e_winfixbuf_cannot_go_to_buffer));
        return;
      }
    }
  }

  char *save_ei = NULL;

  // Temporarily override SHM_OVER and SHM_OVERALL to avoid that file
  // message overwrites output from the command.
  msg_listdo_overwrite++;

  if (eap->cmdidx != CMD_windo && eap->cmdidx != CMD_tabdo) {
    // Don't do syntax HL autocommands.  Skipping the syntax file is a
    // great speed improvement.
    save_ei = au_event_disable(",Syntax");

    FOR_ALL_BUFFERS(buf) {
      buf->b_flags &= ~BF_SYN_SET;
    }
  }

  if (eap->cmdidx == CMD_windo
      || eap->cmdidx == CMD_tabdo
      || buf_hide(curbuf)
      || !check_changed(curbuf, CCGD_AW
                        | (eap->forceit ? CCGD_FORCEIT : 0)
                        | CCGD_EXCMD)) {
    int next_fnum = 0;
    int i = 0;
    // start at the eap->line1 argument/window/buffer
    win_T *wp = firstwin;
    tabpage_T *tp = first_tabpage;
    switch (eap->cmdidx) {
    case CMD_windo:
      for (; wp != NULL && i + 1 < eap->line1; wp = wp->w_next) {
        i++;
      }
      break;
    case CMD_tabdo:
      for (; tp != NULL && i + 1 < eap->line1; tp = tp->tp_next) {
        i++;
      }
      break;
    case CMD_argdo:
      i = (int)eap->line1 - 1;
      break;
    default:
      break;
    }

    buf_T *buf = curbuf;
    size_t qf_size = 0;

    // set pcmark now
    if (eap->cmdidx == CMD_bufdo) {
      // Advance to the first listed buffer after "eap->line1".
      for (buf = firstbuf;
           buf != NULL && (buf->b_fnum < eap->line1 || !buf->b_p_bl);
           buf = buf->b_next) {
        if (buf->b_fnum > eap->line2) {
          buf = NULL;
          break;
        }
      }
      if (buf != NULL) {
        goto_buffer(eap, DOBUF_FIRST, FORWARD, buf->b_fnum);
      }
    } else if (eap->cmdidx == CMD_cdo || eap->cmdidx == CMD_ldo
               || eap->cmdidx == CMD_cfdo || eap->cmdidx == CMD_lfdo) {
      qf_size = qf_get_valid_size(eap);
      assert(eap->line1 >= 0);
      if (qf_size == 0 || (size_t)eap->line1 > qf_size) {
        buf = NULL;
      } else {
        ex_cc(eap);

        buf = curbuf;
        i = (int)eap->line1 - 1;
        if (eap->addr_count <= 0) {
          // Default to all quickfix/location list entries.
          assert(qf_size < MAXLNUM);
          eap->line2 = (linenr_T)qf_size;
        }
      }
    } else {
      setpcmark();
    }
    listcmd_busy = true;            // avoids setting pcmark below

    while (!got_int && buf != NULL) {
      bool execute = true;
      if (eap->cmdidx == CMD_argdo) {
        // go to argument "i"
        if (i == ARGCOUNT) {
          break;
        }
        // Don't call do_argfile() when already there, it will try
        // reloading the file.
        if (curwin->w_arg_idx != i || !editing_arg_idx(curwin)) {
          // Clear 'shm' to avoid that the file message overwrites
          // any output from the command.
          do_argfile(eap, i);
        }
        if (curwin->w_arg_idx != i) {
          break;
        }
      } else if (eap->cmdidx == CMD_windo) {
        // go to window "wp"
        if (!win_valid(wp)) {
          break;
        }
        assert(wp);
        execute = !wp->w_floating || (!wp->w_config.hide && wp->w_config.focusable);
        if (execute) {
          win_goto(wp);
          if (curwin != wp) {
            break;    // something must be wrong
          }
        }
        wp = wp->w_next;
      } else if (eap->cmdidx == CMD_tabdo) {
        // go to window "tp"
        if (!valid_tabpage(tp)) {
          break;
        }
        assert(tp);
        goto_tabpage_tp(tp, true, true);
        tp = tp->tp_next;
      } else if (eap->cmdidx == CMD_bufdo) {
        // Remember the number of the next listed buffer, in case
        // ":bwipe" is used or autocommands do something strange.
        next_fnum = -1;
        for (buf_T *bp = curbuf->b_next; bp != NULL; bp = bp->b_next) {
          if (bp->b_p_bl) {
            next_fnum = bp->b_fnum;
            break;
          }
        }
      }

      i++;
      // execute the command
      if (execute) {
        do_cmdline(eap->arg, eap->ea_getline, eap->cookie, DOCMD_VERBOSE + DOCMD_NOWAIT);
      }

      if (eap->cmdidx == CMD_bufdo) {
        // Done?
        if (next_fnum < 0 || next_fnum > eap->line2) {
          break;
        }

        // Check if the buffer still exists.
        bool buf_still_exists = false;
        FOR_ALL_BUFFERS(bp) {
          if (bp->b_fnum == next_fnum) {
            buf_still_exists = true;
            break;
          }
        }
        if (!buf_still_exists) {
          break;
        }

        // Go to the next buffer.
        goto_buffer(eap, DOBUF_FIRST, FORWARD, next_fnum);

        // If autocommands took us elsewhere, quit here.
        if (curbuf->b_fnum != next_fnum) {
          break;
        }
      }

      if (eap->cmdidx == CMD_cdo || eap->cmdidx == CMD_ldo
          || eap->cmdidx == CMD_cfdo || eap->cmdidx == CMD_lfdo) {
        assert(i >= 0);
        if ((size_t)i >= qf_size || i >= eap->line2) {
          break;
        }

        size_t qf_idx = qf_get_cur_idx(eap);

        ex_cnext(eap);

        // If jumping to the next quickfix entry fails, quit here.
        if (qf_get_cur_idx(eap) == qf_idx) {
          break;
        }
      }

      if (eap->cmdidx == CMD_windo && execute) {
        validate_cursor(curwin);              // cursor may have moved
        // required when 'scrollbind' has been set
        if (curwin->w_p_scb) {
          do_check_scrollbind(true);
        }
      }
      if (eap->cmdidx == CMD_windo || eap->cmdidx == CMD_tabdo) {
        if (i + 1 > eap->line2) {
          break;
        }
      }
      if (eap->cmdidx == CMD_argdo && i >= eap->line2) {
        break;
      }
    }
    listcmd_busy = false;
  }

  msg_listdo_overwrite--;
  if (save_ei != NULL) {
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
}

/// ":compiler[!] {name}"
void ex_compiler(exarg_T *eap)
{
  char *old_cur_comp = NULL;

  if (*eap->arg == NUL) {
    // List all compiler scripts.
    do_cmdline_cmd("echo globpath(&rtp, 'compiler/*.vim')");  // NOLINT
    do_cmdline_cmd("echo globpath(&rtp, 'compiler/*.lua')");  // NOLINT
    return;
  }

  size_t bufsize = strlen(eap->arg) + 14;
  char *buf = xmalloc(bufsize);

  if (eap->forceit) {
    // ":compiler! {name}" sets global options
    do_cmdline_cmd("command -nargs=* -keepscript CompilerSet set <args>");
  } else {
    // ":compiler! {name}" sets local options.
    // To remain backwards compatible "current_compiler" is always
    // used.  A user's compiler plugin may set it, the distributed
    // plugin will then skip the settings.  Afterwards set
    // "b:current_compiler" and restore "current_compiler".
    // Explicitly prepend "g:" to make it work in a function.
    old_cur_comp = get_var_value("g:current_compiler");
    if (old_cur_comp != NULL) {
      old_cur_comp = xstrdup(old_cur_comp);
    }
    do_cmdline_cmd("command -nargs=* -keepscript CompilerSet setlocal <args>");
  }
  do_unlet(S_LEN("g:current_compiler"), true);
  do_unlet(S_LEN("b:current_compiler"), true);

  snprintf(buf, bufsize, "compiler/%s.*", eap->arg);
  if (source_runtime_vim_lua(buf, DIP_ALL) == FAIL) {
    semsg(_(e_compiler_not_supported_str), eap->arg);
  }
  xfree(buf);

  do_cmdline_cmd(":delcommand CompilerSet");

  // Set "b:current_compiler" from "current_compiler".
  char *p = get_var_value("g:current_compiler");
  if (p != NULL) {
    set_internal_string_var("b:current_compiler", p);
  }

  // Restore "current_compiler" for ":compiler {name}".
  if (!eap->forceit) {
    if (old_cur_comp != NULL) {
      set_internal_string_var("g:current_compiler", old_cur_comp);
      xfree(old_cur_comp);
    } else {
      do_unlet(S_LEN("g:current_compiler"), true);
    }
  }
}

/// ":checktime [buffer]"
void ex_checktime(exarg_T *eap)
{
  rs_ex_checktime(eap);
}

// script_host_execute, script_host_execute_file, script_host_do_range
// moved to Rust: src/nvim-rs/ex_cmds2/src/script_host.rs

/// ":drop"
/// Opens the first argument in a window, and the argument list is redefined.
void ex_drop(exarg_T *eap)
{
  bool split = false;

  // Check if the first argument is already being edited in a window.  If
  // so, jump to that window.
  // We would actually need to check all arguments, but that's complicated
  // and mostly only one file is dropped.
  // This also ignores wildcards, since it is very unlikely the user is
  // editing a file name with a wildcard character.
  set_arglist(eap->arg);

  // Expanding wildcards may result in an empty argument list.  E.g. when
  // editing "foo.pyc" and ".pyc" is in 'wildignore'.  Assume that we
  // already did an error message for this.
  if (ARGCOUNT == 0) {
    return;
  }

  if (cmdmod.cmod_tab) {
    // ":tab drop file ...": open a tab for each argument that isn't
    // edited in a window yet.  It's like ":tab all" but without closing
    // windows or tabs.
    ex_all(eap);
    cmdmod.cmod_tab = 0;
    ex_rewind(eap);
    return;
  }

  // ":drop file ...": Edit the first argument.  Jump to an existing
  // window if possible, edit in current window if the current buffer
  // can be abandoned, otherwise open a new window.
  buf_T *buf = buflist_findnr(ARGLIST[0].ae_fnum);

  FOR_ALL_TAB_WINDOWS(tp, wp) {
    if (wp->w_buffer == buf) {
      goto_tabpage_win(tp, wp);
      curwin->w_arg_idx = 0;
      if (!bufIsChanged(curbuf)) {
        const int save_ar = curbuf->b_p_ar;

        // reload the file if it is newer
        curbuf->b_p_ar = true;
        buf_check_timestamp(curbuf);
        curbuf->b_p_ar = save_ar;
      }
      if (curbuf->b_ml.ml_flags & ML_EMPTY) {
        ex_rewind(eap);
      }

      // execute [+cmd]
      if (eap->do_ecmd_cmd) {
        bool did_set_swapcommand = set_swapcommand(eap->do_ecmd_cmd, 0);
        do_cmdline(eap->do_ecmd_cmd, NULL, NULL, DOCMD_VERBOSE);
        if (did_set_swapcommand) {
          set_vim_var_string(VV_SWAPCOMMAND, NULL, -1);
        }
      }

      // no need to execute [++opts] - they only apply for newly loaded buffers.
      return;
    }
  }

  // Check whether the current buffer is changed. If so, we will need
  // to split the current window or data could be lost.
  // Skip the check if the 'hidden' option is set, as in this case the
  // buffer won't be lost.
  if (!buf_hide(curbuf)) {
    emsg_off++;
    split = check_changed(curbuf, CCGD_AW | CCGD_EXCMD);
    emsg_off--;
  }

  // Fake a ":sfirst" or ":first" command edit the first argument.
  if (split) {
    eap->cmdidx = CMD_sfirst;
    eap->cmd[0] = 's';
  } else {
    eap->cmdidx = CMD_first;
  }
  ex_rewind(eap);
}
