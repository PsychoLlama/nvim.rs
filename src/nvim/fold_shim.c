#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/extmark.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/buffer_defs.h"
#include "nvim/cursor.h"
#include "nvim/decoration.h"
#include "nvim/diff.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/vars.h"
#include "nvim/extmark.h"
#include "nvim/extmark_defs.h"
#include "nvim/fold.h"
#include "nvim/garray.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/ops.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/pos_defs.h"
#include "nvim/syntax.h"
#include "nvim/types_defs.h"
#include "nvim/undo.h"

#include "fold_shim.c.generated.h"

linenr_T nvim_fold_tv_get_lnum(typval_T *argvars) { return tv_get_lnum(argvars); }
void nvim_fold_rettv_set_number(typval_T *rettv, varnumber_T nr) { rettv->vval.v_number = nr; }

void nvim_fold_rettv_init_string(typval_T *rettv, char *s)
{
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = s;
}

void nvim_emsg_fold_cannot_create(void) { emsg(_("E350: Cannot create fold with current 'foldmethod'")); }
void nvim_emsg_fold_cannot_delete(void) { emsg(_("E351: Cannot delete fold with current 'foldmethod'")); }
int nvim_win_get_p_fdl(win_T *wp) { return (int)wp->w_p_fdl; }
int nvim_win_get_w_fold_manual(win_T *wp) { return wp->w_fold_manual; }
garray_T *nvim_win_get_folds(win_T *wp) { return &wp->w_folds; }
int nvim_ga_len(garray_T *gap) { return gap->ga_len; }

/// Returns NULL if index is out of bounds.
fold_T *nvim_ga_fold_at(garray_T *gap, int idx)
{
  if (idx < 0 || idx >= gap->ga_len) {
    return NULL;
  }
  return &((fold_T *)gap->ga_data)[idx];
}

linenr_T nvim_fold_get_fd_top(fold_T *fp) { return fp->fd_top; }
linenr_T nvim_fold_get_fd_len(fold_T *fp) { return fp->fd_len; }
garray_T *nvim_fold_get_fd_nested(fold_T *fp) { return &fp->fd_nested; }
int nvim_fold_get_fd_flags(fold_T *fp) { return (int)fp->fd_flags; }
bool nvim_win_get_w_foldinvalid(win_T *wp) { return wp->w_foldinvalid; }
void nvim_win_set_w_foldinvalid(win_T *wp, bool val) { wp->w_foldinvalid = val; }
void nvim_fold_set_fd_flags(fold_T *fp, int flags) { fp->fd_flags = (char)flags; }
int nvim_fold_get_fd_small(fold_T *fp) { return (int)fp->fd_small; }
void nvim_fold_set_fd_small(fold_T *fp, int small) { fp->fd_small = (TriState)small; }

void nvim_fold_swap(garray_T *gap, int idx1, int idx2)
{
  fold_T *data = (fold_T *)gap->ga_data;
  fold_T tmp = data[idx1];
  data[idx1] = data[idx2];
  data[idx2] = tmp;
}

int nvim_win_get_p_fml(win_T *wp) { return (int)wp->w_p_fml; }
void nvim_ga_init_folds(garray_T *gap) { ga_init(gap, (int)sizeof(fold_T), 10); }
linenr_T nvim_win_get_buf_line_count(win_T *wp) { return wp->w_buffer->b_ml.ml_line_count; }
char *nvim_win_get_p_fmr(win_T *wp) { return wp->w_p_fmr; }
colnr_T nvim_fold_ml_get_buf_len(buf_T *buf, linenr_T lnum) { return ml_get_buf_len(buf, lnum); }
int nvim_fold_ml_replace_buf(buf_T *buf, linenr_T lnum, char *newline) { return ml_replace_buf(buf, lnum, newline, false, false); }
int nvim_fold_u_save(linenr_T lnum) { return u_save(lnum - 1, lnum + 1); }

void nvim_fold_extmark_splice_cols(buf_T *buf, int lnum_0, colnr_T col, colnr_T old_col,
                                   colnr_T new_col)
{
  extmark_splice_cols(buf, lnum_0, col, old_col, new_col, kExtmarkUndo);
}

/// Wraps skip_comment(line, false, false, out_is_comment).
void nvim_fold_skip_comment(const char *line, int *out_is_comment)
{
  bool is_comment = false;
  skip_comment((char *)line, false, false, &is_comment);
  *out_is_comment = is_comment ? 1 : 0;
}

char *nvim_fold_get_buf_b_p_cms(buf_T *buf) { return buf->b_p_cms; }
void *nvim_fold_xmalloc(size_t size) { return xmalloc(size); }
char *nvim_win_get_p_fdi(win_T *wp) { return wp->w_p_fdi; }
int nvim_win_get_p_fdn(win_T *wp) { return (int)wp->w_p_fdn; }
int nvim_get_indent_buf(buf_T *buf, linenr_T lnum) { return get_indent_buf(buf, lnum); }
int nvim_get_sw_value(buf_T *buf) { return (int)get_sw_value(buf); }
char *nvim_get_curbuf_b_p_cms(void) { return curbuf->b_p_cms; }
void nvim_ga_grow_folds(garray_T *gap, int n) { ga_grow(gap, n); }
void nvim_fold_set_fd_top(fold_T *fp, linenr_T top) { fp->fd_top = top; }
void nvim_fold_set_fd_len(fold_T *fp, linenr_T len) { fp->fd_len = len; }
fold_T *nvim_ga_get_fold_data(garray_T *gap) { return (fold_T *)gap->ga_data; }
void nvim_ga_set_len(garray_T *gap, int len) { gap->ga_len = len; }

void nvim_fold_memmove(garray_T *gap, int dst_idx, int src_idx, int count)
{
  fold_T *data = (fold_T *)gap->ga_data;
  memmove(&data[dst_idx], &data[src_idx], sizeof(fold_T) * (size_t)count);
}

void nvim_fold_copy(fold_T *dst, const fold_T *src) { *dst = *src; }

void nvim_ga_free_data(garray_T *gap)
{
  xfree(gap->ga_data);
  gap->ga_data = NULL;
  gap->ga_len = 0;
}

void nvim_ga_clear(garray_T *gap) { ga_clear(gap); }
void nvim_win_set_w_fold_manual(win_T *wp, bool val) { wp->w_fold_manual = val; }
void nvim_emsg_nofold(void) { emsg(_(N_("E490: No fold found"))); }
win_T *nvim_get_first_win_in_tab(void) { return curtab->tp_firstwin; }
void nvim_win_set_p_fdl(win_T *wp, int fdl) { wp->w_p_fdl = fdl; }
void nvim_ga_init_folds_ex(garray_T *gap, int itemsize, int growsize) { ga_init(gap, itemsize, growsize); }
int nvim_ga_get_itemsize(garray_T *gap) { return gap->ga_itemsize; }
int nvim_ga_get_growsize(garray_T *gap) { return gap->ga_growsize; }
bool nvim_ga_is_empty(garray_T *gap) { return GA_EMPTY(gap); }
int nvim_fold_buf_is_modifiable(buf_T *buf) { return MODIFIABLE(buf) ? 1 : 0; }
void nvim_fold_emsg_modifiable(void) { emsg(_(e_modifiable)); }
void nvim_check_cursor_col(win_T *wp) { check_cursor_col(wp); }
linenr_T nvim_fold_buf_get_line_count(buf_T *buf) { return buf->b_ml.ml_line_count; }
linenr_T nvim_get_diff_context(void) { return diff_context; }
void nvim_redraw_win_range_later(win_T *wp, linenr_T top, linenr_T bot) { redraw_win_range_later(wp, top, bot); }
char *nvim_get_p_fcl(void) { return p_fcl; }
int nvim_get_disable_fold_update(void) { return disable_fold_update; }
int nvim_get_need_diff_redraw(void) { return need_diff_redraw; }

int64_t nvim_fold_get_vim_var_nr(int vv_idx) { return (int64_t)get_vim_var_nr(vv_idx); }
void nvim_fold_set_vim_var_nr(int vv_idx, int64_t val) { set_vim_var_nr(vv_idx, (varnumber_T)val); }
char *nvim_fold_get_vim_var_str(int vv_idx) { return get_vim_var_str(vv_idx); }
const char *nvim_fold_ngettext_foldtext(int count) { return NGETTEXT("+-%s%3d line: ", "+-%s%3d lines: ", count); }
const char *nvim_fold_ngettext_default(int count) { return NGETTEXT("+--%3d line folded", "+--%3d lines folded ", count); }
linenr_T nvim_fold_get_curbuf_line_count(void) { return curbuf->b_ml.ml_line_count; }

int nvim_syn_get_foldlevel(win_T *wp, linenr_T lnum) { return syn_get_foldlevel(wp, lnum); }
int nvim_fold_eval_foldexpr(win_T *wp, int *out_char) { return eval_foldexpr(wp, out_char); }

/// Save curwin/curbuf and set them to wp/wp->w_buffer. Returns old curwin.
win_T *nvim_fold_save_curwin(win_T *wp)
{
  win_T *saved = curwin;
  curwin = wp;
  curbuf = wp->w_buffer;
  return saved;
}

void nvim_fold_restore_curwin(win_T *saved_win)
{
  curwin = saved_win;
  curbuf = curwin->w_buffer;
}

int nvim_fold_get_keytyped(void) { return (int)KeyTyped; }
void nvim_fold_set_keytyped(int val) { KeyTyped = (bool)val; }
void nvim_fold_set_vim_var_nr_lnum(linenr_T lnum) { set_vim_var_nr(VV_LNUM, (varnumber_T)lnum); }

/// Save current_sctx into *out_saved, then set from wp->w_p_script_ctx[kWinOptFoldtext].
void nvim_fold_save_sctx_foldtext(win_T *wp, void *out_saved)
{
  *(sctx_T *)out_saved = current_sctx;
  current_sctx = wp->w_p_script_ctx[kWinOptFoldtext];
}

void nvim_fold_restore_sctx(void *saved) { current_sctx = *(sctx_T *)saved; }

/// obj_ptr must point to an Object with type == kObjectTypeArray.
/// *out_error is set to 1 if parse_virt_text fails.
void nvim_fold_parse_virt_text_from_obj(void *obj_ptr, void *vt_out, int *out_error)
{
  Object *obj = (Object *)obj_ptr;
  Error err = ERROR_INIT;
  *(VirtText *)vt_out = parse_virt_text(obj->data.array, &err, NULL);
  if (ERROR_SET(&err)) {
    *out_error = 1;
  }
  api_clear_error(&err);
}

char *nvim_fold_win_get_p_fdt(win_T *wp) { return wp->w_p_fdt; }

void nvim_inc_disable_fold_update(void) { disable_fold_update++; }
void nvim_dec_disable_fold_update(void) { disable_fold_update--; }
