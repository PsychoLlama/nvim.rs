// fold_shim.c: C accessor wrappers for the Rust fold crate (nvim-fold).
//
// These thin wrappers provide a stable C ABI for Rust code to call into
// Neovim's C internals.  Each function is called from one or more Rust
// modules in src/nvim-rs/fold/.

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

// ============================================================================
// VimL function accessors (for f_foldclosed, f_foldlevel, etc.)
// ============================================================================

/// Get the line number from the first element of argvars (tv_get_lnum).
linenr_T nvim_fold_tv_get_lnum(typval_T *argvars)
{
  return tv_get_lnum(argvars);
}

/// Set rettv->vval.v_number.
void nvim_fold_rettv_set_number(typval_T *rettv, varnumber_T nr)
{
  rettv->vval.v_number = nr;
}

/// Set rettv type to VAR_STRING and set rettv->vval.v_string.
void nvim_fold_rettv_init_string(typval_T *rettv, char *s)
{
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = s;
}

// ============================================================================
// Fold FFI accessors
// ============================================================================

/// Emit error message for cannot create fold with current foldmethod.
void nvim_emsg_fold_cannot_create(void)
{
  emsg(_("E350: Cannot create fold with current 'foldmethod'"));
}

/// Emit error message for cannot delete fold with current foldmethod.
void nvim_emsg_fold_cannot_delete(void)
{
  emsg(_("E351: Cannot delete fold with current 'foldmethod'"));
}

/// Get the w_p_fdl (foldlevel) field from a window.
int nvim_win_get_p_fdl(win_T *wp)
{
  return (int)wp->w_p_fdl;
}

/// Get the w_fold_manual field from a window.
int nvim_win_get_w_fold_manual(win_T *wp)
{
  return wp->w_fold_manual;
}

/// Get a pointer to the window's folds growarray.
garray_T *nvim_win_get_folds(win_T *wp)
{
  return &wp->w_folds;
}

/// Get the length of a garray.
int nvim_ga_len(garray_T *gap)
{
  return gap->ga_len;
}

/// Get a fold_T pointer at index in a garray.
/// Returns NULL if index is out of bounds.
fold_T *nvim_ga_fold_at(garray_T *gap, int idx)
{
  if (idx < 0 || idx >= gap->ga_len) {
    return NULL;
  }
  return &((fold_T *)gap->ga_data)[idx];
}

/// Get the fd_top field from a fold.
linenr_T nvim_fold_get_fd_top(fold_T *fp)
{
  return fp->fd_top;
}

/// Get the fd_len field from a fold.
linenr_T nvim_fold_get_fd_len(fold_T *fp)
{
  return fp->fd_len;
}

/// Get a pointer to the nested folds growarray.
garray_T *nvim_fold_get_fd_nested(fold_T *fp)
{
  return &fp->fd_nested;
}

/// Get the fd_flags field from a fold.
int nvim_fold_get_fd_flags(fold_T *fp)
{
  return (int)fp->fd_flags;
}

/// Get the w_foldinvalid field from a window.
bool nvim_win_get_w_foldinvalid(win_T *wp)
{
  return wp->w_foldinvalid;
}

/// Set the w_foldinvalid field in a window.
void nvim_win_set_w_foldinvalid(win_T *wp, bool val)
{
  wp->w_foldinvalid = val;
}

/// Set the fd_flags field of a fold.
void nvim_fold_set_fd_flags(fold_T *fp, int flags)
{
  fp->fd_flags = (char)flags;
}

/// Get the fd_small field from a fold.
int nvim_fold_get_fd_small(fold_T *fp)
{
  return (int)fp->fd_small;
}

/// Set the fd_small field of a fold.
void nvim_fold_set_fd_small(fold_T *fp, int small)
{
  fp->fd_small = (TriState)small;
}

/// Swap two fold entries in a garray.
/// idx1 and idx2 must be valid indices.
void nvim_fold_swap(garray_T *gap, int idx1, int idx2)
{
  fold_T *data = (fold_T *)gap->ga_data;
  fold_T tmp = data[idx1];
  data[idx1] = data[idx2];
  data[idx2] = tmp;
}

/// Get the w_p_fml (foldminlines) field from a window.
int nvim_win_get_p_fml(win_T *wp)
{
  return (int)wp->w_p_fml;
}

/// Initialize the folds garray for a window (called from Rust).
void nvim_ga_init_folds(garray_T *gap)
{
  ga_init(gap, (int)sizeof(fold_T), 10);
}

/// Get the line count of the window's buffer.
linenr_T nvim_win_get_buf_line_count(win_T *wp)
{
  return wp->w_buffer->b_ml.ml_line_count;
}

/// Get the w_p_fmr (foldmarker option) field from a window.
char *nvim_win_get_p_fmr(win_T *wp)
{
  return wp->w_p_fmr;
}

/// Get the length of a buffer line (wrapper for ml_get_buf_len).
colnr_T nvim_fold_ml_get_buf_len(buf_T *buf, linenr_T lnum)
{
  return ml_get_buf_len(buf, lnum);
}

/// Replace a buffer line, transferring ownership of newline (wrapper for ml_replace_buf).
/// The newline pointer must be heap-allocated; this call takes ownership.
int nvim_fold_ml_replace_buf(buf_T *buf, linenr_T lnum, char *newline)
{
  return ml_replace_buf(buf, lnum, newline, false, false);
}

/// Save undo for a line range common to fold operations: u_save(lnum-1, lnum+1).
int nvim_fold_u_save(linenr_T lnum)
{
  return u_save(lnum - 1, lnum + 1);
}

/// Wrapper for extmark_splice_cols for fold marker operations.
void nvim_fold_extmark_splice_cols(buf_T *buf, int lnum_0, colnr_T col, colnr_T old_col,
                                   colnr_T new_col)
{
  extmark_splice_cols(buf, lnum_0, col, old_col, new_col, kExtmarkUndo);
}

/// Check if a buffer line ends with an unclosed comment.
/// Wraps skip_comment(line, false, false, out_is_comment).
void nvim_fold_skip_comment(const char *line, int *out_is_comment)
{
  bool is_comment = false;
  skip_comment((char *)line, false, false, &is_comment);
  *out_is_comment = is_comment ? 1 : 0;
}

/// Get the commentstring option for a buffer (b_p_cms).
char *nvim_fold_get_buf_b_p_cms(buf_T *buf)
{
  return buf->b_p_cms;
}

/// Allocate memory using xmalloc (used by markers.rs to build buffer lines with correct allocator).
void *nvim_fold_xmalloc(size_t size)
{
  return xmalloc(size);
}

/// Get the w_p_fdi (foldignore option) field from a window.
char *nvim_win_get_p_fdi(win_T *wp)
{
  return wp->w_p_fdi;
}

/// Get the w_p_fdn (foldnestmax option) field from a window.
int nvim_win_get_p_fdn(win_T *wp)
{
  return (int)wp->w_p_fdn;
}

/// Get the indentation of a buffer line (wrapper for get_indent_buf).
int nvim_get_indent_buf(buf_T *buf, linenr_T lnum)
{
  return get_indent_buf(buf, lnum);
}

/// Get the shiftwidth value for a buffer (wrapper for get_sw_value).
int nvim_get_sw_value(buf_T *buf)
{
  return (int)get_sw_value(buf);
}

/// Get curbuf's commentstring option (b_p_cms).
char *nvim_get_curbuf_b_p_cms(void)
{
  return curbuf->b_p_cms;
}

/// Grow a garray to hold at least n more fold_T entries.
void nvim_ga_grow_folds(garray_T *gap, int n)
{
  ga_grow(gap, n);
}

/// Set the fd_top field of a fold.
void nvim_fold_set_fd_top(fold_T *fp, linenr_T top)
{
  fp->fd_top = top;
}

/// Set the fd_len field of a fold.
void nvim_fold_set_fd_len(fold_T *fp, linenr_T len)
{
  fp->fd_len = len;
}

/// Get the ga_data pointer from a garray (as fold_T*).
fold_T *nvim_ga_get_fold_data(garray_T *gap)
{
  return (fold_T *)gap->ga_data;
}

/// Set the ga_len field of a garray.
void nvim_ga_set_len(garray_T *gap, int len)
{
  gap->ga_len = len;
}

/// Move fold entries within a garray.
/// Moves `count` entries from src_idx to dst_idx.
void nvim_fold_memmove(garray_T *gap, int dst_idx, int src_idx, int count)
{
  fold_T *data = (fold_T *)gap->ga_data;
  memmove(&data[dst_idx], &data[src_idx], sizeof(fold_T) * (size_t)count);
}

/// Copy a fold entry from one location to another.
void nvim_fold_copy(fold_T *dst, const fold_T *src)
{
  *dst = *src;
}

/// Free the ga_data pointer of a garray (for nested folds).
void nvim_ga_free_data(garray_T *gap)
{
  xfree(gap->ga_data);
  gap->ga_data = NULL;
  gap->ga_len = 0;
}

/// Clear (free and reset) a garray of folds.
/// Equivalent to ga_clear(): frees ga_data, resets ga_len, ga_maxlen.
void nvim_ga_clear(garray_T *gap)
{
  ga_clear(gap);
}

/// Set the w_fold_manual field in a window.
void nvim_win_set_w_fold_manual(win_T *wp, bool val)
{
  wp->w_fold_manual = val;
}

/// Emit the "no fold found" error message.
void nvim_emsg_nofold(void)
{
  emsg(_(N_("E490: No fold found")));
}

/// Get the first window in the current tab.
win_T *nvim_get_first_win_in_tab(void)
{
  return curtab->tp_firstwin;
}

/// Set the w_p_fdl (foldlevel) field in a window.
void nvim_win_set_p_fdl(win_T *wp, int fdl)
{
  wp->w_p_fdl = fdl;
}

/// Initialize a garray with specified itemsize and growsize.
void nvim_ga_init_folds_ex(garray_T *gap, int itemsize, int growsize)
{
  ga_init(gap, itemsize, growsize);
}

/// Get the ga_itemsize field from a garray.
int nvim_ga_get_itemsize(garray_T *gap)
{
  return gap->ga_itemsize;
}

/// Get the ga_growsize field from a garray.
int nvim_ga_get_growsize(garray_T *gap)
{
  return gap->ga_growsize;
}

/// Check if a garray is empty.
bool nvim_ga_is_empty(garray_T *gap)
{
  return GA_EMPTY(gap);
}

/// Check if buffer is modifiable (for fold operations).
int nvim_fold_buf_is_modifiable(buf_T *buf)
{
  return MODIFIABLE(buf) ? 1 : 0;
}

/// Emit error message for buffer not modifiable (for fold operations).
void nvim_fold_emsg_modifiable(void)
{
  emsg(_(e_modifiable));
}

/// Wrapper for check_cursor_col for Rust.
void nvim_check_cursor_col(win_T *wp)
{
  check_cursor_col(wp);
}

/// Get buffer line count (for fold Rust code).
linenr_T nvim_fold_buf_get_line_count(buf_T *buf)
{
  return buf->b_ml.ml_line_count;
}

/// Get diff_context global.
linenr_T nvim_get_diff_context(void)
{
  return diff_context;
}

/// Redraw window range later.
void nvim_redraw_win_range_later(win_T *wp, linenr_T top, linenr_T bot)
{
  redraw_win_range_later(wp, top, bot);
}

/// Get the p_fcl option value.
char *nvim_get_p_fcl(void) { return p_fcl; }

/// Get the disable_fold_update flag.
int nvim_get_disable_fold_update(void) { return disable_fold_update; }

/// Get the need_diff_redraw flag.
int nvim_get_need_diff_redraw(void) { return need_diff_redraw; }

// ============================================================================
// Accessors for f_foldtext Rust implementation
// ============================================================================

/// Get a vim variable as a number (generic).
int64_t nvim_fold_get_vim_var_nr(int vv_idx)
{
  return (int64_t)get_vim_var_nr(vv_idx);
}

/// Set a vim variable as a number (generic).
void nvim_fold_set_vim_var_nr(int vv_idx, int64_t val)
{
  set_vim_var_nr(vv_idx, (varnumber_T)val);
}

/// Get a vim variable as a string (generic).
char *nvim_fold_get_vim_var_str(int vv_idx)
{
  return get_vim_var_str(vv_idx);
}

/// Get the localized fold text header format string (wraps NGETTEXT for foldtext).
const char *nvim_fold_ngettext_foldtext(int count)
{
  return NGETTEXT("+-%s%3d line: ", "+-%s%3d lines: ", count);
}

/// Get the localized default fold text format string (wraps NGETTEXT for default).
const char *nvim_fold_ngettext_default(int count)
{
  return NGETTEXT("+--%3d line folded", "+--%3d lines folded ", count);
}

/// Get curbuf's line count (still used by level.rs and lib.rs).
linenr_T nvim_fold_get_curbuf_line_count(void)
{
  return curbuf->b_ml.ml_line_count;
}

// ============================================================================
// Accessors for Rust fold level calculation
// ============================================================================

/// Get the syntax fold level for a line (wrapper for syn_get_foldlevel).
int nvim_syn_get_foldlevel(win_T *wp, linenr_T lnum)
{
  return syn_get_foldlevel(wp, lnum);
}

/// Evaluate 'foldexpr' for window wp at line v:lnum.
/// Returns the numeric value; sets *out_char to the prefix character.
int nvim_fold_eval_foldexpr(win_T *wp, int *out_char)
{
  return eval_foldexpr(wp, out_char);
}

/// Save curwin/curbuf and set them to wp/wp->w_buffer.
/// Returns the old curwin pointer.
win_T *nvim_fold_save_curwin(win_T *wp)
{
  win_T *saved = curwin;
  curwin = wp;
  curbuf = wp->w_buffer;
  return saved;
}

/// Restore curwin/curbuf from saved_win.
void nvim_fold_restore_curwin(win_T *saved_win)
{
  curwin = saved_win;
  curbuf = curwin->w_buffer;
}

/// Get the current value of KeyTyped.
int nvim_fold_get_keytyped(void)
{
  return (int)KeyTyped;
}

/// Set the KeyTyped global.
void nvim_fold_set_keytyped(int val)
{
  KeyTyped = (bool)val;
}

/// Set v:lnum to lnum.
void nvim_fold_set_vim_var_nr_lnum(linenr_T lnum)
{
  set_vim_var_nr(VV_LNUM, (varnumber_T)lnum);
}

// ============================================================================
// Accessors for get_foldtext Rust migration (display.rs)
// ============================================================================

/// Save current_sctx into *out_saved (sctx_T), then set current_sctx from
/// wp->w_p_script_ctx[kWinOptFoldtext].
void nvim_fold_save_sctx_foldtext(win_T *wp, void *out_saved)
{
  *(sctx_T *)out_saved = current_sctx;
  current_sctx = wp->w_p_script_ctx[kWinOptFoldtext];
}

/// Restore current_sctx from *saved (sctx_T).
void nvim_fold_restore_sctx(void *saved)
{
  current_sctx = *(sctx_T *)saved;
}

/// Call parse_virt_text on an Array embedded in an Object.
/// obj_ptr must point to an Object with type == kObjectTypeArray.
/// vt_out receives the VirtText result.
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

/// Get wp->w_p_fdt (foldtext option string).
char *nvim_fold_win_get_p_fdt(win_T *wp)
{
  return wp->w_p_fdt;
}

void nvim_inc_disable_fold_update(void) { disable_fold_update++; }
void nvim_dec_disable_fold_update(void) { disable_fold_update--; }
