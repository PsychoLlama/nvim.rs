// mark_shim.c: Rust FFI accessors for mark crate.

#include <stdbool.h>
#include <string.h>

#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/errors.h"
#include "nvim/extmark.h"
#include "nvim/extmark_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/normal_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/strings.h"
#include "nvim/textobject.h"
#include "nvim/types_defs.h"

#include "nvim/mark_shim.h"
#include "mark_shim.c.generated.h"

extern void rs_tagstack_clear_entry(void *tg);

int nvim_mark_bt_prompt(buf_T *buf) { return bt_prompt(buf); }
buf_T *nvim_mark_buflist_findnr(int fnum) { return buflist_findnr(fnum); }
int nvim_mark_findsent(int dir, int count) { return (int)findsent(dir, count); }
void nvim_mark_extmark_adjust(buf_T *buf, linenr_T line1, linenr_T line2, linenr_T amount, linenr_T amount_after, int op) { extmark_adjust(buf, line1, line2, amount, amount_after, (ExtmarkOp)op); }
int nvim_mark_win_get_jumplistlen(win_T *win) { return win->w_jumplistlen; }
void nvim_mark_win_set_jumplistlen(win_T *win, int len) { win->w_jumplistlen = len; }
int nvim_mark_win_get_jumplistidx(win_T *win) { return win->w_jumplistidx; }
void nvim_mark_win_set_jumplistidx(win_T *win, int idx) { win->w_jumplistidx = idx; }
xfmark_T *nvim_mark_win_get_jumplist_entry(win_T *win, int idx) { return &win->w_jumplist[idx]; }
pos_T nvim_mark_win_get_pcmark(win_T *win) { return win->w_pcmark; }
void nvim_mark_win_set_pcmark(win_T *win, pos_T pos) { win->w_pcmark = pos; }
pos_T nvim_mark_win_get_prev_pcmark(win_T *win) { return win->w_prev_pcmark; }
void nvim_mark_win_set_prev_pcmark(win_T *win, pos_T pos) { win->w_prev_pcmark = pos; }
pos_T nvim_mark_win_get_cursor(win_T *win) { return win->w_cursor; }
buf_T *nvim_mark_win_get_buffer(win_T *win) { return win->w_buffer; }
fmark_T *nvim_mark_buf_get_last_cursor(buf_T *buf) { return &buf->b_last_cursor; }
const char *nvim_mark_get_e_umark(void) { return _(e_umark); }
const char *nvim_mark_get_e_marknotset(void) { return _(e_marknotset); }
const char *nvim_mark_get_e_markinval(void) { return _(e_markinval); }
fmark_T *nvim_mark_buf_get_namedm(buf_T *buf, int idx) { return &buf->b_namedm[idx]; }
fmark_T *nvim_mark_buf_get_last_insert(buf_T *buf) { return &buf->b_last_insert; }
fmark_T *nvim_mark_buf_get_last_change(buf_T *buf) { return &buf->b_last_change; }
pos_T *nvim_mark_buf_get_op_start(buf_T *buf) { return &buf->b_op_start; }
pos_T *nvim_mark_buf_get_op_end(buf_T *buf) { return &buf->b_op_end; }
pos_T nvim_mark_buf_get_op_start_val(buf_T *buf) { return buf->b_op_start; }
pos_T nvim_mark_buf_get_op_end_val(buf_T *buf) { return buf->b_op_end; }
pos_T nvim_mark_buf_get_visual_start(buf_T *buf) { return buf->b_visual.vi_start; }
pos_T nvim_mark_buf_get_visual_end(buf_T *buf) { return buf->b_visual.vi_end; }
pos_T *nvim_mark_buf_get_visual_start_ptr(buf_T *buf) { return &buf->b_visual.vi_start; }
pos_T *nvim_mark_buf_get_visual_end_ptr(buf_T *buf) { return &buf->b_visual.vi_end; }
int nvim_mark_buf_get_visual_mode(buf_T *buf) { return buf->b_visual.vi_mode; }
void nvim_mark_buf_set_visual_mode(buf_T *buf, int mode) { buf->b_visual.vi_mode = mode; }
fmark_T *nvim_mark_buf_get_prompt_start(buf_T *buf) { return &buf->b_prompt_start; }
fmark_T *nvim_mark_buf_get_changelist(buf_T *buf, int idx) { return &buf->b_changelist[idx]; }
int nvim_mark_buf_get_changelistlen(buf_T *buf) { return buf->b_changelistlen; }
void nvim_mark_buf_set_changelistlen(buf_T *buf, int len) { buf->b_changelistlen = len; }
unsigned nvim_mark_get_cmod_flags(void) { return cmdmod.cmod_flags; }
linenr_T nvim_mark_win_get_topline(win_T *win) { return win->w_topline; }
int nvim_mark_win_get_changelistidx(win_T *win) { return win->w_changelistidx; }
void nvim_mark_win_set_changelistidx(win_T *win, int idx) { win->w_changelistidx = idx; }
void nvim_mark_win_jumplist_remove(win_T *win, int from_idx, int len) { memmove(&win->w_jumplist[from_idx], &win->w_jumplist[from_idx + 1], (size_t)(len - from_idx) * sizeof(win->w_jumplist[0])); }
void nvim_mark_win_jumplist_shift_down(win_T *win) { memmove(&win->w_jumplist[0], &win->w_jumplist[1], (JUMPLISTSIZE - 1) * sizeof(win->w_jumplist[0])); }
void nvim_mark_win_jumplist_copy_entry(win_T *win, int to_idx, int from_idx) { win->w_jumplist[to_idx] = win->w_jumplist[from_idx]; }
int nvim_mark_win_get_tagstacklen(win_T *win) { return win->w_tagstacklen; }
void nvim_mark_win_set_tagstacklen(win_T *win, int len) { win->w_tagstacklen = len; }
int nvim_mark_win_get_tagstackidx(win_T *win) { return win->w_tagstackidx; }
void nvim_mark_win_set_tagstackidx(win_T *win, int idx) { win->w_tagstackidx = idx; }
int nvim_mark_win_get_tagstack_fnum(win_T *win, int idx) { return win->w_tagstack[idx].fmark.fnum; }
void nvim_mark_win_tagstack_clear_entry(win_T *win, int idx) { rs_tagstack_clear_entry(&win->w_tagstack[idx]); }
void nvim_mark_win_tagstack_remove(win_T *win, int from_idx, int len) { memmove(&win->w_tagstack[from_idx], &win->w_tagstack[from_idx + 1], (size_t)(len - from_idx) * sizeof(win->w_tagstack[0])); }
void nvim_mark_win_set_jumplist_xfmark(win_T *win, int idx, pos_T mark, int fnum, fmarkv_T view) { SET_XFMARK(&win->w_jumplist[idx], mark, fnum, view, NULL); }
int nvim_mark_win_get_jumplist_fnum(win_T *win, int idx) { return win->w_jumplist[idx].fmark.fnum; }
linenr_T nvim_mark_win_get_jumplist_lnum(win_T *win, int idx) { return win->w_jumplist[idx].fmark.mark.lnum; }
void nvim_mark_win_jumplist_free_fname(win_T *win, int idx) { xfree(win->w_jumplist[idx].fname); }
int nvim_mark_buf_get_has_qf_entry(buf_T *buf) { return buf->b_has_qf_entry; }
void nvim_mark_buf_set_has_qf_entry(buf_T *buf, int val) { buf->b_has_qf_entry = val; }
win_T *nvim_mark_win_get_next(win_T *win) { return win->w_next; }
buf_T *nvim_mark_win_get_buf(win_T *win) { return win->w_buffer; }
linenr_T nvim_mark_win_get_old_cursor_lnum(win_T *win) { return win->w_old_cursor_lnum; }
linenr_T *nvim_mark_win_get_old_cursor_lnum_ptr(win_T *win) { return &win->w_old_cursor_lnum; }
linenr_T *nvim_mark_win_get_old_visual_lnum_ptr(win_T *win) { return &win->w_old_visual_lnum; }
linenr_T nvim_mark_win_get_topline_val(win_T *win) { return win->w_topline; }
void nvim_mark_win_set_topline_val(win_T *win, linenr_T val) { win->w_topline = val; }
void nvim_mark_win_set_topfill(win_T *win, int val) { win->w_topfill = val; }
pos_T *nvim_mark_win_get_cursor_ptr(win_T *win) { return &win->w_cursor; }
pos_T *nvim_mark_win_get_pcmark_ptr(win_T *win) { return &win->w_pcmark; }
pos_T *nvim_mark_win_get_prev_pcmark_ptr(win_T *win) { return &win->w_prev_pcmark; }
tabpage_T *nvim_mark_tabpage_next(tabpage_T *tp) { return tp->tp_next; }
win_T *nvim_mark_tabpage_firstwin(tabpage_T *tp) { return (tp == curtab) ? firstwin : tp->tp_firstwin; }
int nvim_mark_buf_get_wininfo_count(buf_T *buf) { return (int)kv_size(buf->b_wininfo); }
pos_T *nvim_mark_buf_get_wininfo_mark(buf_T *buf, int idx) { return &kv_A(buf->b_wininfo, idx)->wi_mark.mark; }
pos_T *nvim_mark_win_get_jumplist_mark_ptr(win_T *win, int idx) { return &win->w_jumplist[idx].fmark.mark; }
pos_T *nvim_mark_win_get_tagstack_mark_ptr(win_T *win, int idx) { return &win->w_tagstack[idx].fmark.mark; }
void nvim_mark_emsg_invarg(void) { emsg(_(e_invarg)); }
void nvim_mark_emsg_argreq(void) { emsg(_(e_argreq)); }
void nvim_mark_semsg_invarg2(const char *p) { semsg(_(e_invarg2), p); }
int nvim_mark_findpar(int *inclusive, int dir, int count, int what, int do_sentences)
{
  bool pincl = false;
  int result = (int)findpar(&pincl, dir, count, what, (bool)do_sentences);
  if (inclusive) { *inclusive = (int)pincl; }
  return result;
}
void nvim_mark_win_set_cursor(win_T *win, pos_T pos) { win->w_cursor = pos; }

// Phase 7/8: display accessors
char *nvim_mark_get_iobuff(void) { return IObuff; }
int nvim_mark_get_iosize(void) { return IOSIZE; }

// Phase 1 (fname2fnum migration): path/env accessors
char *nvim_mark_get_namebuff(void) { return NameBuff; }
int nvim_mark_get_maxpathl(void) { return MAXPATHL; }
size_t nvim_mark_expand_env(const char *src, char *dst, size_t dstlen) { return expand_env((char *)src, dst, (int)dstlen); }
char *nvim_mark_path_shorten_fname(char *full_path, char *dir_name) { return path_shorten_fname(full_path, dir_name); }
buf_T *nvim_mark_buflist_new(char *ffname, char *sfname, int lnum, int flags) { return buflist_new(ffname, sfname, (linenr_T)lnum, flags); }
