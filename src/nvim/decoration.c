#include <assert.h>
#include <limits.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/api/extmark.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/decoration.h"
#include "nvim/decoration_provider.h"
#include "nvim/drawscreen.h"
#include "nvim/extmark.h"
#include "nvim/fold.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/highlight.h"
#include "nvim/highlight_group.h"
#include "nvim/marktree.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/move.h"
#include "nvim/option_vars.h"
#include "nvim/pos_defs.h"
#include "nvim/sign.h"

#include "decoration.c.generated.h"

// Rust fold FFI declaration
extern int rs_hasAnyFolding(win_T *win);

// Rust decoration FFI declarations (migrated from decoration.c to Rust)
extern bool decor_conceal_line(win_T *wp, int row, bool check_cursor);
extern bool win_lines_concealed(win_T *wp);

// Rust implementations
extern bool decor_virt_pos(const DecorRange *decor);
extern VirtTextPos decor_virt_pos_kind(const DecorRange *decor);
extern int rs_sh_is_sign(uint16_t flags);
extern int rs_sh_is_hl_eol(uint16_t flags);
extern int rs_sh_is_ui_watched(uint16_t flags);
extern int rs_sh_is_conceal(uint16_t flags);
extern int rs_sh_is_spell_on(uint16_t flags);
extern int rs_sh_is_spell_off(uint16_t flags);
extern int rs_sh_is_conceal_lines(uint16_t flags);
extern int rs_vt_is_lines(uint8_t flags);
extern int rs_vt_is_hide(uint8_t flags);
extern int rs_vt_is_lines_above(uint8_t flags);
extern int rs_vt_repeat_linebreak(uint8_t flags);

uint32_t decor_freelist = UINT32_MAX;

// Decorations might be requested to be deleted in a callback in the middle of redrawing.
// In this case, there might still be live references to the memory allocated for the decoration.
// Keep a "to free" list which can be safely processed when redrawing is done.
DecorVirtText *to_free_virt = NULL;
uint32_t to_free_sh = UINT32_MAX;

// Rust FFI declarations (range insertion)
extern void rs_decor_range_add_from_inline(void *state, int start_row, int start_col,
                                           int end_row, int end_col, bool ext,
                                           void *vt, uint32_t sh_idx,
                                           uint16_t hl_flags, uint16_t hl_priority,
                                           int hl_hl_id, uint32_t hl_conceal_char,
                                           bool owned, uint32_t ns, uint32_t mark_id);

// clear_virttext and clear_virtlines are declared in decoration.h

// Rust implementation of bufhl_add_hl_pos_offset
extern void rs_bufhl_add_hl_pos_offset(buf_T *buf, int src_id, int hl_id,
                                       linenr_T start_lnum, colnr_T start_col,
                                       linenr_T end_lnum, colnr_T end_col,
                                       colnr_T offset);

/// C helper: set a single highlight range via extmark_set, for Rust FFI.
void nvim_extmark_set_hl(buf_T *buf, int ns_id, int row, int col, int end_row, int end_col,
                         int hl_id)
{
  DecorInline decor = DECOR_INLINE_INIT;
  decor.data.hl.hl_id = hl_id;
  extmark_set(buf, (uint32_t)ns_id, NULL, row, col, end_row, end_col,
              decor, MT_FLAG_DECOR_HL, true, false, true, false, NULL);
}

/// Add highlighting to a buffer, bounded by two cursor positions, with an offset.
void bufhl_add_hl_pos_offset(buf_T *buf, int src_id, int hl_id, lpos_T pos_start, lpos_T pos_end,
                             colnr_T offset)
{
  rs_bufhl_add_hl_pos_offset(buf, src_id, hl_id,
                             pos_start.lnum, pos_start.col,
                             pos_end.lnum, pos_end.col, offset);
}


// Phase 1 C accessor wrappers for Rust FFI
// Note: nvim_get_curwin, nvim_win_get_p_cole, nvim_win_get_cursor_lnum are
// already exported from the Rust window crate (nvim-window).

/// Get buf_meta_total(buf, kMTMetaConcealLines) for Rust FFI.
int nvim_buf_meta_total_conceal_lines(buf_T *buf) { return (int)buf_meta_total(buf, kMTMetaConcealLines); }
/// Invoke conceal_line decoration providers for Rust FFI.
int nvim_decor_providers_invoke_conceal_line(win_T *wp, int row) { return decor_providers_invoke_conceal_line(wp, row) ? 1 : 0; }
/// Check conceal_cursor_line(wp) for Rust FFI.
int nvim_conceal_cursor_line(win_T *wp) { return conceal_cursor_line(wp) ? 1 : 0; }
/// Check ns_in_win(ns, wp) for Rust FFI.
int nvim_ns_in_win(uint32_t ns, win_T *wp) { return ns_in_win(ns, wp) ? 1 : 0; }
/// Check mt_conceal_lines(mark) for Rust FFI.
int nvim_mt_conceal_lines(MTKey mark) { return mt_conceal_lines(mark) ? 1 : 0; }

/// This assumes maximum one entry of each kind, which will not always be the case.
///
/// NB: assumes caller has allocated enough space in dict for all fields!
void decor_to_dict_legacy(Dict *dict, DecorInline decor, bool hl_name, Arena *arena)
{
  DecorSignHighlight sh_hl = DECOR_SIGN_HIGHLIGHT_INIT;
  DecorSignHighlight sh_sign = DECOR_SIGN_HIGHLIGHT_INIT;
  DecorVirtText *virt_text = NULL;
  DecorVirtText *virt_lines = NULL;
  int32_t priority = -1;  // sentinel value which cannot actually be set

  if (decor.ext) {
    DecorVirtText *vt = decor.data.ext.vt;
    while (vt) {
      if (rs_vt_is_lines(vt->flags)) {
        virt_lines = vt;
      } else {
        virt_text = vt;
      }
      vt = vt->next;
    }

    uint32_t idx = decor.data.ext.sh_idx;
    while (idx != DECOR_ID_INVALID) {
      DecorSignHighlight *sh = &kv_A(decor_items, idx);
      if (rs_sh_is_sign(sh->flags)) {
        sh_sign = *sh;
      } else {
        sh_hl = *sh;
      }
      idx = sh->next;
    }
  } else {
    sh_hl = decor_sh_from_inline(decor.data.hl);
  }

  if (sh_hl.hl_id) {
    PUT_C(*dict, "hl_group", hl_group_name(sh_hl.hl_id, hl_name));
    PUT_C(*dict, "hl_eol", BOOLEAN_OBJ(rs_sh_is_hl_eol(sh_hl.flags)));
    priority = sh_hl.priority;
  }

  if (rs_sh_is_conceal(sh_hl.flags)) {
    char buf[MAX_SCHAR_SIZE];
    schar_get(buf, sh_hl.text[0]);
    PUT_C(*dict, "conceal", CSTR_TO_ARENA_OBJ(arena, buf));
  }

  if (rs_sh_is_conceal_lines(sh_hl.flags)) {
    PUT_C(*dict, "conceal_lines", STRING_OBJ(cstr_as_string("")));
  }

  if (rs_sh_is_spell_on(sh_hl.flags)) {
    PUT_C(*dict, "spell", BOOLEAN_OBJ(true));
  } else if (rs_sh_is_spell_off(sh_hl.flags)) {
    PUT_C(*dict, "spell", BOOLEAN_OBJ(false));
  }

  if (rs_sh_is_ui_watched(sh_hl.flags)) {
    PUT_C(*dict, "ui_watched", BOOLEAN_OBJ(true));
  }

  if (sh_hl.url != NULL) {
    PUT_C(*dict, "url", STRING_OBJ(cstr_as_string(sh_hl.url)));
  }

  if (virt_text) {
    if (virt_text->hl_mode) {
      PUT_C(*dict, "hl_mode", CSTR_AS_OBJ(hl_mode_str[virt_text->hl_mode]));
    }

    Array chunks = virt_text_to_array(virt_text->data.virt_text, hl_name, arena);
    PUT_C(*dict, "virt_text", ARRAY_OBJ(chunks));
    PUT_C(*dict, "virt_text_hide", BOOLEAN_OBJ(rs_vt_is_hide(virt_text->flags)));
    PUT_C(*dict, "virt_text_repeat_linebreak", BOOLEAN_OBJ(rs_vt_repeat_linebreak(virt_text->flags)));
    if (virt_text->pos == kVPosWinCol) {
      PUT_C(*dict, "virt_text_win_col", INTEGER_OBJ(virt_text->col));
    }
    PUT_C(*dict, "virt_text_pos", CSTR_AS_OBJ(virt_text_pos_str[virt_text->pos]));
    priority = virt_text->priority;
  }

  if (virt_lines) {
    Array all_chunks = arena_array(arena, kv_size(virt_lines->data.virt_lines));
    int virt_lines_flags = 0;
    for (size_t i = 0; i < kv_size(virt_lines->data.virt_lines); i++) {
      virt_lines_flags = kv_A(virt_lines->data.virt_lines, i).flags;
      Array chunks = virt_text_to_array(kv_A(virt_lines->data.virt_lines, i).line, hl_name, arena);
      ADD(all_chunks, ARRAY_OBJ(chunks));
    }
    PUT_C(*dict, "virt_lines", ARRAY_OBJ(all_chunks));
    PUT_C(*dict, "virt_lines_above", BOOLEAN_OBJ(rs_vt_is_lines_above(virt_lines->flags)));
    PUT_C(*dict, "virt_lines_leftcol", BOOLEAN_OBJ(virt_lines_flags & kVLLeftcol));
    PUT_C(*dict, "virt_lines_overflow",
          CSTR_AS_OBJ(virt_lines_flags & kVLScroll ? "scroll" : "trunc"));
    priority = virt_lines->priority;
  }

  if (rs_sh_is_sign(sh_sign.flags)) {
    if (sh_sign.text[0]) {
      char buf[SIGN_WIDTH * MAX_SCHAR_SIZE];
      describe_sign_text(buf, sh_sign.text);
      PUT_C(*dict, "sign_text", CSTR_TO_ARENA_OBJ(arena, buf));
    }

    if (sh_sign.sign_name) {
      PUT_C(*dict, "sign_name", CSTR_AS_OBJ(sh_sign.sign_name));
    }

    // uncrustify:off

    struct { char *name; const int val; } hls[] = {
      { "sign_hl_group"      , sh_sign.hl_id            },
      { "number_hl_group"    , sh_sign.number_hl_id     },
      { "line_hl_group"      , sh_sign.line_hl_id       },
      { "cursorline_hl_group", sh_sign.cursorline_hl_id },
      { NULL, 0 },
    };

    // uncrustify:on

    for (int j = 0; hls[j].name; j++) {
      if (hls[j].val) {
        PUT_C(*dict, hls[j].name, hl_group_name(hls[j].val, hl_name));
      }
    }
    priority = sh_sign.priority;
  }

  if (priority != -1) {
    PUT_C(*dict, "priority", INTEGER_OBJ(priority));
  }
}

Object hl_group_name(int hl_id, bool hl_name)
{
  if (hl_name) {
    return CSTR_AS_OBJ(syn_id2name(hl_id));
  } else {
    return INTEGER_OBJ(hl_id);
  }
}

// Rust FFI accessor functions for DecorState

void *nvim_get_decor_state(void) { return &decor_state; }

/// Check if decor_state.win has a specific buffer.
int nvim_decor_state_win_has_buffer(void *state_ptr, buf_T *buf)
{
  DecorState *state = (DecorState *)state_ptr;
  return (state->win && state->win->w_buffer == buf) ? 1 : 0;
}

void nvim_decor_state_destroy_slots(void *state_ptr) { kv_destroy(((DecorState *)state_ptr)->slots); }
void nvim_decor_state_destroy_ranges_i(void *state_ptr) { kv_destroy(((DecorState *)state_ptr)->ranges_i); }

// Memory management accessors

uint32_t nvim_get_decor_freelist(void) { return decor_freelist; }
void nvim_set_decor_freelist(uint32_t val) { decor_freelist = val; }
uint32_t nvim_decor_items_size(void) { return (uint32_t)kv_size(decor_items); }
uint32_t nvim_decor_items_get_next(uint32_t idx) { return kv_A(decor_items, idx).next; }
void nvim_decor_items_set(uint32_t idx, DecorSignHighlight item) { kv_A(decor_items, idx) = item; }
uint32_t nvim_decor_items_push(DecorSignHighlight item)
{
  uint32_t pos = (uint32_t)kv_size(decor_items);
  kv_push(decor_items, item);
  return pos;
}
DecorSignHighlight *nvim_decor_items_get(uint32_t idx) { return &kv_A(decor_items, idx); }
void *nvim_decor_items_get_ptr(uint32_t idx) { return &kv_A(decor_items, idx); }
void *nvim_xmalloc_decor_virt_text(void) { return xmalloc(sizeof(DecorVirtText)); }
void nvim_xfree_ptr(void *ptr) { xfree(ptr); }
void *nvim_get_to_free_virt(void) { return to_free_virt; }
void nvim_set_to_free_virt(void *val) { to_free_virt = (DecorVirtText *)val; }
uint32_t nvim_get_to_free_sh(void) { return to_free_sh; }
void nvim_set_to_free_sh(uint32_t val) { to_free_sh = val; }

/// Clear VirtText (free chunks + destroy kvec). Does the actual work.
void nvim_clear_virttext(void *vt_ptr)
{
  VirtText *text = (VirtText *)vt_ptr;
  for (size_t i = 0; i < kv_size(*text); i++) {
    xfree(kv_A(*text, i).text);
  }
  kv_destroy(*text);
  *text = (VirtText)KV_INITIAL_VALUE;
}

/// Clear VirtLines (free all lines + destroy kvec). Does the actual work.
void nvim_clear_virtlines(void *vt_ptr)
{
  VirtLines *lines = (VirtLines *)vt_ptr;
  for (size_t i = 0; i < kv_size(*lines); i++) {
    nvim_clear_virttext(&kv_A(*lines, i).line);
  }
  kv_destroy(*lines);
  *lines = (VirtLines)KV_INITIAL_VALUE;
}

uint16_t nvim_decor_items_get_flags(uint32_t idx) { return kv_A(decor_items, idx).flags; }
void nvim_decor_items_set_flags(uint32_t idx, uint16_t flags) { kv_A(decor_items, idx).flags = flags; }
void nvim_decor_items_set_next(uint32_t idx, uint32_t next) { kv_A(decor_items, idx).next = next; }
void nvim_decor_items_clear_sign_name(uint32_t idx) { XFREE_CLEAR(kv_A(decor_items, idx).sign_name); }
void nvim_decor_items_clear_url(uint32_t idx) { XFREE_CLEAR(kv_A(decor_items, idx).url); }

int nvim_buf_get_marktree_n_keys(void *buf_ptr) { return (int)((buf_T *)buf_ptr)->b_marktree->n_keys; }
void *nvim_decor_win_get_buffer(void *wp_ptr) { return ((win_T *)wp_ptr)->w_buffer; }

/// Get the row from marktree_itr_current on decor_state.itr.
int nvim_decor_state_itr_current_row(void *state_ptr)
{
  DecorState *state = (DecorState *)state_ptr;
  MTKey k = rs_marktree_itr_current(state->itr);
  return k.pos.row;
}

/// Call marktree_itr_get on decor_state.itr using the buffer's marktree.
void nvim_decor_state_itr_get(void *state_ptr, void *buf_ptr, int row, int col)
{
  DecorState *state = (DecorState *)state_ptr;
  buf_T *buf = (buf_T *)buf_ptr;
  rs_marktree_itr_get(buf->b_marktree, row, col, state->itr);
}

// Redraw Dispatch and Buffer Operations helpers

void nvim_redraw_buf_line_later(void *buf_ptr, int lnum, bool redraw) { redraw_buf_line_later((buf_T *)buf_ptr, lnum, redraw); }
void nvim_changed_lines_invalidate_buf(void *buf_ptr, int lnum1, int col1, int lnum2, int xtra) { changed_lines_invalidate_buf((buf_T *)buf_ptr, lnum1, col1, lnum2, xtra); }
void nvim_redraw_buf_range_later(void *buf_ptr, int first, int last) { redraw_buf_range_later((buf_T *)buf_ptr, first, last); }

int nvim_decor_buf_get_line_count(void *buf_ptr) { return ((buf_T *)buf_ptr)->b_ml.ml_line_count; }

/// Iterator for VirtText chunks - returns text and advances position.
/// This wraps next_virt_text_chunk for FFI use.
/// @param[in] vt_ptr Pointer to VirtText (the kvec)
/// @param[in,out] pos Position in the VirtText
/// @param[out] attr Highlight attribute (must be non-NULL)
/// @return Text of the chunk, or NULL if no more chunks
const char *nvim_next_virt_text_chunk(void *vt_ptr, size_t *pos, int *attr)
{
  // VirtText is a kvec_t(VirtTextChunk)
  VirtText *vt = (VirtText *)vt_ptr;
  return next_virt_text_chunk(*vt, pos, attr);
}

// Extmark Decoration Accessor Functions (for Rust FFI - extmark crate)

/// Free decoration data.
void nvim_decor_free(DecorInlineData data, bool ext)
{
  decor_free((DecorInline){ .ext = ext, .data = data });
}

/// Remove decoration from a buffer.
void nvim_buf_decor_remove(buf_T *buf, int row_start, int row_end, int col_start,
                           DecorInlineData decor_data, bool decor_ext, bool free_decor)
{
  buf_decor_remove(buf, row_start, row_end, col_start,
                   (DecorInline){ .ext = decor_ext, .data = decor_data }, free_decor);
}

/// Add decoration to a buffer.
void nvim_buf_put_decor(buf_T *buf, DecorInlineData decor_data, bool decor_ext,
                        int row_start, int row_end)
{
  buf_put_decor(buf, (DecorInline){ .ext = decor_ext, .data = decor_data }, row_start, row_end);
}

/// Redraw decoration in a buffer.
void nvim_decor_redraw(buf_T *buf, int row_start, int row_end, int col_start,
                       DecorInlineData decor_data, bool decor_ext)
{
  decor_redraw(buf, row_start, row_end, col_start,
               (DecorInline){ .ext = decor_ext, .data = decor_data });
}

/// Count sign columns in a range (wrapper for Rust FFI).
void nvim_buf_signcols_count_range(buf_T *buf, int row1, int row2, int add, int clear)
{
  buf_signcols_count_range(buf, row1, row2, add, (TriState)clear);
}


/// Get type flags for decoration data (for Rust extmark FFI).
uint16_t nvim_decor_type_flags(DecorInlineData data, bool ext)
{
  return decor_type_flags((DecorInline){ .ext = ext, .data = data });
}

/// Counter for sign placement order.
static int sign_add_id = 0;

// Phase 3: Sign buffer operation helpers for Rust FFI

/// Trigger window setting change for Rust FFI.
void nvim_changed_window_setting(win_T *wp) { changed_window_setting(wp); }

/// Recompute number column width if needed (FOR_ALL_TAB_WINDOWS loop stays in C).
void nvim_may_force_numberwidth_recompute(buf_T *buf, bool unplace)
{
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    if (wp->w_buffer == buf
        && wp->w_minscwidth == SCL_NUM
        && (wp->w_p_nu || wp->w_p_rnu)
        && (unplace || wp->w_nrwidth_width < 2)) {
      wp->w_nrwidth_line_count = 0;
    }
  }
}

/// Get b_signcols.count[0] for Rust FFI.
int nvim_buf_signcols_get_count0(buf_T *buf) { return buf->b_signcols.count[0]; }
/// Set b_signcols.count[0] for Rust FFI.
void nvim_buf_signcols_set_count0(buf_T *buf, int val) { buf->b_signcols.count[0] = val; }
/// Indexed get/set for b_signcols.count[] (Phase 3).
int nvim_buf_signcols_get_count_at(buf_T *buf, int idx) { return buf->b_signcols.count[idx]; }
void nvim_buf_signcols_set_count_at(buf_T *buf, int idx, int val) { buf->b_signcols.count[idx] = val; }

/// Get current sign_add_id for Rust FFI.
int nvim_get_sign_add_id(void) { return sign_add_id; }
/// Post-increment sign_add_id, return old value for Rust FFI.
int nvim_incr_sign_add_id(void) { return sign_add_id++; }

// Core Column Rendering helpers

/// Advance the marktree iterator to position (row, col), adding inline decorations.
///
/// Returns the col_until value from the marktree (next mark column - 1, or MAXCOL).
/// The future-to-active promotion loop (Part 2) is done in Rust (promote_future_ranges).
int nvim_decor_col_iter_marks(win_T *wp, int col, DecorState *state)
{
  buf_T *const buf = wp->w_buffer;
  int const row = state->row;
  int col_until = MAXCOL;

  while (true) {
    MTKey mark = rs_marktree_itr_current(state->itr);
    if (mark.pos.row < 0 || mark.pos.row > row) {
      break;
    } else if (mark.pos.row == row && mark.pos.col > col) {
      col_until = mark.pos.col - 1;
      break;
    }

    if (mt_invalid(mark) || mt_end(mark) || !mt_decor_any(mark) || !ns_in_win(mark.ns, wp)) {
      goto next_mark;
    }

    MTPos endpos = rs_marktree_get_altpos(buf->b_marktree, mark, NULL);
    DecorInline d = mt_decor(mark);
    rs_decor_range_add_from_inline(state, mark.pos.row, mark.pos.col, endpos.row, endpos.col,
                                   d.ext, d.data.ext.vt, d.data.ext.sh_idx,
                                   d.data.hl.flags, d.data.hl.priority,
                                   d.data.hl.hl_id, d.data.hl.conceal_char,
                                   false, mark.ns, mark.id);

next_mark:
    rs_marktree_itr_next(buf->b_marktree, state->itr);
  }

  return col_until;
}

