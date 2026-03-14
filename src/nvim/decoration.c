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

// Rust implementations
extern bool decor_virt_pos(const DecorRange *decor);
extern VirtTextPos decor_virt_pos_kind(const DecorRange *decor);
extern int rs_sign_item_cmp(int priority1, uint32_t id1, uint32_t add_id1,
                            int priority2, uint32_t id2, uint32_t add_id2);

// Additional Rust implementations for Phase 133
extern DecorSignHighlight rs_decor_sh_from_inline(uint16_t flags, uint16_t priority,
                                                   int hl_id, uint32_t conceal_char);
extern int rs_decor_init_draw_col_value(int win_col, int hidden, int kind,
                                        int pos, int vt_flags);
extern int rs_should_recheck_draw_col(int draw_col);
extern int rs_decor_kind_is_highlight(int kind);
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

// Rust implementations for Phase 1
extern void rs_decor_state_invalidate(void *state, buf_T *buf);

// Rust implementations for Phase 2
extern void *rs_decor_put_vt(DecorVirtText *vt_data, DecorVirtText *next);
extern void rs_decor_free(int ext, DecorVirtText *vt, uint32_t sh_idx);
extern void rs_decor_check_to_be_deleted(void);


// Rust implementations for Phase 4
extern void rs_decor_range_add_from_inline(void *state, int start_row, int start_col,
                                           int end_row, int end_col, bool ext,
                                           void *vt, uint32_t sh_idx,
                                           uint16_t hl_flags, uint16_t hl_priority,
                                           int hl_hl_id, uint32_t hl_conceal_char,
                                           bool owned, uint32_t ns, uint32_t mark_id);

// Rust implementations for Phase 5
extern void rs_decor_redraw(void *buf, int row1, int row2, int col1,
                            bool ext, void *vt, uint32_t sh_idx,
                            uint16_t hl_flags, uint16_t hl_priority,
                            int hl_hl_id, uint32_t hl_conceal_char);
extern void rs_buf_put_decor(void *buf, bool ext, void *vt, uint32_t sh_idx, int row, int row2);
extern void rs_buf_decor_remove(void *buf, int row1, int row2, int col1,
                                bool ext, void *vt, uint32_t sh_idx,
                                uint16_t hl_flags, uint16_t hl_priority,
                                int hl_hl_id, uint32_t hl_conceal_char,
                                bool do_free);

// Rust implementations for Phase 7
extern void *rs_decor_find_sign(bool ext, uint32_t sh_idx);

// clear_virttext and clear_virtlines are declared in decoration.h

/// Add highlighting to a buffer, bounded by two cursor positions,
/// with an offset.
///
/// TODO(bfredl): make decoration powerful enough so that this
/// can be done with a single ephemeral decoration.
///
/// @param buf Buffer to add highlights to
/// @param src_id src_id to use or 0 to use a new src_id group,
///               or -1 for ungrouped highlight.
/// @param hl_id Highlight group id
/// @param pos_start Cursor position to start the highlighting at
/// @param pos_end Cursor position to end the highlighting at
/// @param offset Move the whole highlighting this many columns to the right
void bufhl_add_hl_pos_offset(buf_T *buf, int src_id, int hl_id, lpos_T pos_start, lpos_T pos_end,
                             colnr_T offset)
{
  colnr_T hl_start = 0;
  colnr_T hl_end = 0;
  DecorInline decor = DECOR_INLINE_INIT;
  decor.data.hl.hl_id = hl_id;

  // TODO(bfredl): if decoration had blocky mode, we could avoid this loop
  for (linenr_T lnum = pos_start.lnum; lnum <= pos_end.lnum; lnum++) {
    int end_off = 0;
    if (pos_start.lnum < lnum && lnum < pos_end.lnum) {
      // TODO(bfredl): This is quite ad-hoc, but the space between |num| and
      // text being highlighted is the indication of \n being part of the
      // substituted text. But it would be more consistent to highlight
      // a space _after_ the previous line instead (like highlight EOL list
      // char)
      hl_start = MAX(offset - 1, 0);
      end_off = 1;
      hl_end = 0;
    } else if (lnum == pos_start.lnum && lnum < pos_end.lnum) {
      hl_start = pos_start.col + offset;
      end_off = 1;
      hl_end = 0;
    } else if (pos_start.lnum < lnum && lnum == pos_end.lnum) {
      hl_start = MAX(offset - 1, 0);
      hl_end = pos_end.col + offset;
    } else if (pos_start.lnum == lnum && pos_end.lnum == lnum) {
      hl_start = pos_start.col + offset;
      hl_end = pos_end.col + offset;
    }

    extmark_set(buf, (uint32_t)src_id, NULL,
                (int)lnum - 1, hl_start, (int)lnum - 1 + end_off, hl_end,
                decor, MT_FLAG_DECOR_HL, true, false, true, false, NULL);
  }
}

void decor_redraw(buf_T *buf, int row1, int row2, int col1, DecorInline decor)
{
  rs_decor_redraw(buf, row1, row2, col1, decor.ext, decor.data.ext.vt, decor.data.ext.sh_idx,
                  decor.data.hl.flags, decor.data.hl.priority,
                  decor.data.hl.hl_id, decor.data.hl.conceal_char);
}

void decor_redraw_sh(buf_T *buf, int row1, int row2, DecorSignHighlight sh)
{
  if (sh.hl_id || (sh.url != NULL)
      || rs_sh_is_sign(sh.flags) || rs_sh_is_spell_on(sh.flags)
      || rs_sh_is_spell_off(sh.flags) || rs_sh_is_conceal(sh.flags)) {
    if (row2 >= row1) {
      redraw_buf_range_later(buf, row1 + 1, row2 + 1);
    }
  }
  if (rs_sh_is_conceal_lines(sh.flags)) {
    FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
      // TODO(luukvbaal): redraw only unconcealed lines, and scroll lines below
      // it up or down. Also when opening/closing a fold.
      if (wp->w_buffer == buf) {
        changed_window_setting(wp);
      }
    }
  }
  if (rs_sh_is_ui_watched(sh.flags)) {
    redraw_buf_line_later(buf, row1 + 1, false);
  }
}

DecorVirtText *decor_put_vt(DecorVirtText vt, DecorVirtText *next)
{
  return rs_decor_put_vt(&vt, next);
}

DecorSignHighlight decor_sh_from_inline(DecorHighlightInline item)
{
  // TODO(bfredl): Eventually simple signs will be inlinable as well
  assert(!(item.flags & kSHIsSign));
  return rs_decor_sh_from_inline(item.flags, item.priority, item.hl_id, item.conceal_char);
}

void buf_put_decor(buf_T *buf, DecorInline decor, int row, int row2)
{
  rs_buf_put_decor(buf, decor.ext, decor.data.ext.vt, decor.data.ext.sh_idx, row, row2);
}

/// When displaying signs in the 'number' column, if the width of the number
/// column is less than 2, then force recomputing the width after placing or
/// unplacing the first sign in "buf".
static void may_force_numberwidth_recompute(buf_T *buf, bool unplace)
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

static int sign_add_id = 0;
void buf_put_decor_sh(buf_T *buf, DecorSignHighlight *sh, int row1, int row2)
{
  if (rs_sh_is_sign(sh->flags)) {
    sh->sign_add_id = sign_add_id++;
    if (sh->text[0]) {
      buf_signcols_count_range(buf, row1, row2, 1, kFalse);
      may_force_numberwidth_recompute(buf, false);
    }
  }
}

void buf_decor_remove(buf_T *buf, int row1, int row2, int col1, DecorInline decor, bool free)
{
  rs_buf_decor_remove(buf, row1, row2, col1, decor.ext, decor.data.ext.vt, decor.data.ext.sh_idx,
                      decor.data.hl.flags, decor.data.hl.priority,
                      decor.data.hl.hl_id, decor.data.hl.conceal_char, free);
}

void buf_remove_decor_sh(buf_T *buf, int row1, int row2, DecorSignHighlight *sh)
{
  if (rs_sh_is_sign(sh->flags)) {
    if (sh->text[0]) {
      if (buf_meta_total(buf, kMTMetaSignText)) {
        buf_signcols_count_range(buf, row1, row2, -1, kFalse);
      } else {
        may_force_numberwidth_recompute(buf, true);
        buf->b_signcols.count[0] = 0;
        buf->b_signcols.max = 0;
      }
    }
  }
}

void decor_free(DecorInline decor)
{
  if (!decor.ext) {
    return;
  }
  rs_decor_free(1, decor.data.ext.vt, decor.data.ext.sh_idx);
}

/// Check if we are in a callback while drawing, which might invalidate the marktree iterator.
///
/// This should be called whenever a structural modification has been done to a
/// marktree in a public API function (i e any change which adds or deletes marks).
void decor_state_invalidate(buf_T *buf)
{
  rs_decor_state_invalidate(&decor_state, buf);
}

void decor_check_to_be_deleted(void)
{
  assert(!decor_state.running_decor_provider);
  rs_decor_check_to_be_deleted();
}




/// Get the next chunk of a virtual text item.
///
/// @param[in]     vt    The virtual text item
/// @param[in,out] pos   Position in the virtual text item
/// @param[in,out] attr  Highlight attribute
///
/// @return  The text of the chunk, or NULL if there are no more chunks
char *next_virt_text_chunk(VirtText vt, size_t *pos, int *attr)
{
  char *text = NULL;
  for (; text == NULL && *pos < kv_size(vt); (*pos)++) {
    text = kv_A(vt, *pos).text;
    int hl_id = kv_A(vt, *pos).hl_id;
    if (hl_id >= 0) {
      *attr = MAX(*attr, 0);
      if (hl_id > 0) {
        *attr = hl_combine_attr(*attr, syn_id2attr(hl_id));
      }
    }
  }
  return text;
}

DecorVirtText *decor_find_virttext(buf_T *buf, int row, uint64_t ns_id)
{
  MarkTreeIter itr[1] = { 0 };
  rs_marktree_itr_get(buf->b_marktree, row, 0,  itr);
  while (true) {
    MTKey mark = rs_marktree_itr_current(itr);
    if (mark.pos.row < 0 || mark.pos.row > row) {
      break;
    } else if (mt_invalid(mark)) {
      goto next_mark;
    }
    DecorVirtText *decor = mt_decor_virt(mark);
    while (decor && (decor->flags & kVTIsLines)) {
      decor = decor->next;
    }
    if ((ns_id == 0 || ns_id == mark.ns) && decor) {
      return decor;
    }
next_mark:
    rs_marktree_itr_next(buf->b_marktree, itr);
  }
  return NULL;
}

/// @return true if decor has a virtual position (virtual text or ui_watched)
bool decor_redraw_start(win_T *wp, int top_row, DecorState *state)
{
  buf_T *buf = wp->w_buffer;
  state->top_row = top_row;
  state->itr_valid = true;

  if (!rs_marktree_itr_get_overlap(buf->b_marktree, top_row, 0, state->itr)) {
    return false;
  }
  MTPair pair;

  while (rs_marktree_itr_step_overlap(buf->b_marktree, state->itr, &pair)) {
    MTKey m = pair.start;
    if (mt_invalid(m) || !mt_decor_any(m)) {
      continue;
    }

    decor_range_add_from_inline(state, pair.start.pos.row, pair.start.pos.col, pair.end_pos.row,
                                pair.end_pos.col,
                                mt_decor(m), false, m.ns, m.id);
  }

  return true;  // TODO(bfredl): check if available in the region
}

static void decor_range_add_from_inline(DecorState *state, int start_row, int start_col,
                                        int end_row, int end_col, DecorInline decor, bool owned,
                                        uint32_t ns, uint32_t mark_id)
{
  rs_decor_range_add_from_inline(state, start_row, start_col, end_row, end_col,
                                 decor.ext, decor.data.ext.vt, decor.data.ext.sh_idx,
                                 decor.data.hl.flags, decor.data.hl.priority,
                                 decor.data.hl.hl_id, decor.data.hl.conceal_char,
                                 owned, ns, mark_id);
}

static void decor_range_insert(DecorState *state, DecorRange *range)
{
  range->ordering = state->new_range_ordering++;

  int index;
  // Get space for a new `DecorRange` from the freelist or allocate.
  if (state->free_slot_i >= 0) {
    index = state->free_slot_i;
    DecorRangeSlot *slot = &kv_A(state->slots, index);
    state->free_slot_i = slot->next_free_i;
    slot->range = *range;
  } else {
    index = (int)kv_size(state->slots);
    kv_pushp(state->slots)->range = *range;
  }

  int const row = range->start_row;
  int const col = range->start_col;

  int const count = (int)kv_size(state->ranges_i);
  int *const indices = state->ranges_i.items;
  DecorRangeSlot *const slots = state->slots.items;

  int begin = state->future_begin;
  int end = count;
  while (begin < end) {
    int const mid = begin + ((end - begin) >> 1);
    DecorRange *const mr = &slots[indices[mid]].range;

    int const mrow = mr->start_row;
    int const mcol = mr->start_col;
    if (mrow < row || (mrow == row && mcol <= col)) {
      begin = mid + 1;
      if (mrow == row && mcol == col) {
        break;
      }
    } else {
      end = mid;
    }
  }

  kv_pushp(state->ranges_i);
  int *const item = &kv_A(state->ranges_i, begin);
  memmove(item + 1, item, (size_t)(count - begin) * sizeof(*item));
  *item = index;
}

/// Initialize the draw_col of a newly-added virtual text item.
void decor_init_draw_col(int win_col, bool hidden, DecorRange *item)
{
  DecorVirtText *vt = item->kind == kDecorKindVirtText ? item->data.vt : NULL;
  VirtTextPos pos = decor_virt_pos_kind(item);
  int vt_flags = vt ? vt->flags : 0;
  item->draw_col = rs_decor_init_draw_col_value(win_col, hidden, item->kind, pos, vt_flags);
}

void decor_recheck_draw_col(int win_col, bool hidden, DecorState *state)
{
  int const end = state->current_end;
  int *const indices = state->ranges_i.items;
  DecorRangeSlot *const slots = state->slots.items;

  for (int i = 0; i < end; i++) {
    DecorRange *const r = &slots[indices[i]].range;
    if (rs_should_recheck_draw_col(r->draw_col)) {
      decor_init_draw_col(win_col, hidden, r);
    }
  }
}

static const uint32_t conceal_filter[kMTMetaCount] = {[kMTMetaConcealLines] = kMTFilterSelect };

/// Called by draw, move and plines code to determine whether a line is concealed.
/// Scans the marktree for conceal_line marks on "row" and invokes any
/// _on_conceal_line decoration provider callbacks, if necessary.
///
/// @param check_cursor If true, avoid an early return for an unconcealed cursorline.
///                     Depending on the callsite, we still want to know whether the
///                     cursor line would be concealed if it was not the cursorline.
///
/// @return whether "row" is concealed
bool decor_conceal_line(win_T *wp, int row, bool check_cursor)
{
  if (row < 0 || wp->w_p_cole < 2
      || (!check_cursor && wp == curwin && row + 1 == wp->w_cursor.lnum
          && !conceal_cursor_line(wp))) {
    return false;
  }

  // No need to scan the marktree if there are no conceal_line marks.
  if (!buf_meta_total(wp->w_buffer, kMTMetaConcealLines)) {
    return decor_providers_invoke_conceal_line(wp, row);
  }

  // Scan the marktree for any conceal_line marks on this row.
  MTPair pair;
  MarkTreeIter itr[1];
  rs_marktree_itr_get_overlap(wp->w_buffer->b_marktree, row, 0, itr);
  while (rs_marktree_itr_step_overlap(wp->w_buffer->b_marktree, itr, &pair)) {
    if (mt_conceal_lines(pair.start) && ns_in_win(pair.start.ns, wp)) {
      return true;
    }
  }

  rs_marktree_itr_step_out_filter(wp->w_buffer->b_marktree, itr, conceal_filter);

  while (itr->x) {
    MTKey mark = rs_marktree_itr_current(itr);
    if (mark.pos.row > row) {
      break;
    }
    if (mt_conceal_lines(mark) && ns_in_win(pair.start.ns, wp)) {
      return true;
    }
    rs_marktree_itr_next_filter(wp->w_buffer->b_marktree, itr, row + 1, 0, conceal_filter);
  }

  return decor_providers_invoke_conceal_line(wp, row);
}

/// Wrapper for decor_conceal_line for Rust FFI.
int nvim_decor_conceal_line(win_T *wp, int row, int check_cursor)
{
  return decor_conceal_line(wp, row, check_cursor != 0) ? 1 : 0;
}

/// @return whether a window may have folded or concealed lines
bool win_lines_concealed(win_T *wp)
{
  return rs_hasAnyFolding(wp) || wp->w_p_cole >= 2;
}

/// Wrapper for win_lines_concealed for Rust FFI.
int nvim_win_lines_concealed(win_T *wp)
{
  return win_lines_concealed(wp) ? 1 : 0;
}

int sign_item_cmp(const void *p1, const void *p2)
{
  const SignItem *s1 = (SignItem *)p1;
  const SignItem *s2 = (SignItem *)p2;
  return rs_sign_item_cmp(s1->sh->priority, s1->id, s1->sh->sign_add_id,
                          s2->sh->priority, s2->id, s2->sh->sign_add_id);
}

static const uint32_t sign_filter[kMTMetaCount] = {[kMTMetaSignText] = kMTFilterSelect,
                                                   [kMTMetaSignHL] = kMTFilterSelect };

/// Return the signs and highest priority sign attributes on a row.
///
/// @param[out] sattrs Output array for sign text and texthl id
/// @param[out] line_id Highest priority linehl id
/// @param[out] cul_id Highest priority culhl id
/// @param[out] num_id Highest priority numhl id
void decor_redraw_signs(win_T *wp, buf_T *buf, int row, SignTextAttrs sattrs[], int *line_id,
                        int *cul_id, int *num_id)
{
  if (!buf_has_signs(buf)) {
    return;
  }

  MTPair pair;
  int num_text = 0;
  MarkTreeIter itr[1];
  kvec_t(SignItem) signs = KV_INITIAL_VALUE;
  // TODO(bfredl): integrate with main decor loop.
  rs_marktree_itr_get_overlap(buf->b_marktree, row, 0, itr);
  while (rs_marktree_itr_step_overlap(buf->b_marktree, itr, &pair)) {
    if (!mt_invalid(pair.start) && mt_decor_sign(pair.start) && ns_in_win(pair.start.ns, wp)) {
      DecorSignHighlight *sh = decor_find_sign(mt_decor(pair.start));
      num_text += (sh->text[0] != NUL);
      kv_push(signs, ((SignItem){ sh, pair.start.id }));
    }
  }

  rs_marktree_itr_step_out_filter(buf->b_marktree, itr, sign_filter);

  while (itr->x) {
    MTKey mark = rs_marktree_itr_current(itr);
    if (mark.pos.row != row) {
      break;
    }
    if (!mt_invalid(mark) && !mt_end(mark) && mt_decor_sign(mark) && ns_in_win(mark.ns, wp)) {
      DecorSignHighlight *sh = decor_find_sign(mt_decor(mark));
      num_text += (sh->text[0] != NUL);
      kv_push(signs, ((SignItem){ sh, mark.id }));
    }

    rs_marktree_itr_next_filter(buf->b_marktree, itr, row + 1, 0, sign_filter);
  }

  if (kv_size(signs)) {
    int width = wp->w_minscwidth == SCL_NUM ? 1 : wp->w_scwidth;
    int len = MIN(width, num_text);
    int idx = 0;
    qsort((void *)&kv_A(signs, 0), kv_size(signs), sizeof(kv_A(signs, 0)), sign_item_cmp);

    for (size_t i = 0; i < kv_size(signs); i++) {
      DecorSignHighlight *sh = kv_A(signs, i).sh;
      if (sattrs && idx < len && sh->text[0]) {
        memcpy(sattrs[idx].text, sh->text, SIGN_WIDTH * sizeof(sattr_T));
        sattrs[idx++].hl_id = sh->hl_id;
      }
      if (num_id != NULL && *num_id <= 0) {
        *num_id = sh->number_hl_id;
      }
      if (line_id != NULL && *line_id <= 0) {
        *line_id = sh->line_hl_id;
      }
      if (cul_id != NULL && *cul_id <= 0) {
        *cul_id = sh->cursorline_hl_id;
      }
    }
    kv_destroy(signs);
  }
}

DecorSignHighlight *decor_find_sign(DecorInline decor)
{
  return rs_decor_find_sign(decor.ext, decor.data.ext.sh_idx);
}

static const uint32_t signtext_filter[kMTMetaCount] = {[kMTMetaSignText] = kMTFilterSelect };

/// Count the number of signs in a range after adding/removing a sign, or to
/// (re-)initialize a range in "b_signcols.count".
///
/// @param add  1, -1 or 0 for an added, deleted or initialized range.
/// @param clear  kFalse, kTrue or kNone for an, added/deleted, cleared, or initialized range.
void buf_signcols_count_range(buf_T *buf, int row1, int row2, int add, TriState clear)
{
  if (!buf->b_signcols.autom || row2 < row1 || !buf_meta_total(buf, kMTMetaSignText)) {
    return;
  }

  // Allocate an array of integers holding the number of signs in the range.
  int *count = xcalloc((size_t)(row2 + 1 - row1), sizeof(int));
  MarkTreeIter itr[1];
  MTPair pair = { 0 };

  // Increment count array for signs that start before "row1" but do overlap the range.
  rs_marktree_itr_get_overlap(buf->b_marktree, row1, 0, itr);
  while (rs_marktree_itr_step_overlap(buf->b_marktree, itr, &pair)) {
    if ((pair.start.flags & MT_FLAG_DECOR_SIGNTEXT) && !mt_invalid(pair.start)) {
      for (int i = row1; i <= MIN(row2, pair.end_pos.row); i++) {
        count[i - row1]++;
      }
    }
  }

  rs_marktree_itr_step_out_filter(buf->b_marktree, itr, signtext_filter);

  // Continue traversing the marktree until beyond "row2".
  while (itr->x) {
    MTKey mark = rs_marktree_itr_current(itr);
    if (mark.pos.row > row2) {
      break;
    }
    if ((mark.flags & MT_FLAG_DECOR_SIGNTEXT) && !mt_invalid(mark) && !mt_end(mark)) {
      // Increment count array for the range of a paired sign mark.
      MTPos end = rs_marktree_get_altpos(buf->b_marktree, mark, NULL);
      for (int i = mark.pos.row; i <= MIN(row2, end.row); i++) {
        count[i - row1]++;
      }
    }

    rs_marktree_itr_next_filter(buf->b_marktree, itr, row2 + 1, 0, signtext_filter);
  }

  // For each row increment "b_signcols.count" at the number of counted signs,
  // and decrement at the previous number of signs. These two operations are
  // split in separate calls if "clear" is not kFalse (surrounding a marktree splice).
  for (int i = 0; i < row2 + 1 - row1; i++) {
    int prevwidth = MIN(SIGN_SHOW_MAX, count[i] - add);
    if (clear != kNone && prevwidth > 0) {
      buf->b_signcols.count[prevwidth - 1]--;
#ifndef RELDEBUG
      // TODO(bfredl): correct marktree splicing so that this doesn't fail
      assert(buf->b_signcols.count[prevwidth - 1] >= 0);
#endif
    }
    int width = MIN(SIGN_SHOW_MAX, count[i]);
    if (clear != kTrue && width > 0) {
      buf->b_signcols.count[width - 1]++;
      if (width > buf->b_signcols.max) {
        buf->b_signcols.max = width;
      }
    }
  }

  xfree(count);
}

bool decor_redraw_eol(win_T *wp, DecorState *state, int *eol_attr, int eol_col)
{
  decor_redraw_col(wp, MAXCOL, MAXCOL, false, state);
  state->eol_col = eol_col;

  int const count = state->current_end;
  int *const indices = state->ranges_i.items;
  DecorRangeSlot *const slots = state->slots.items;

  bool has_virt_pos = false;
  for (int i = 0; i < count; i++) {
    DecorRange *r = &slots[indices[i]].range;
    has_virt_pos |= r->start_row == state->row && decor_virt_pos(r);

    if (rs_decor_kind_is_highlight(r->kind) && rs_sh_is_hl_eol(r->data.sh.flags)) {
      *eol_attr = hl_combine_attr(*eol_attr, r->attr_id);
    }
  }
  return has_virt_pos;
}

static const uint32_t lines_filter[kMTMetaCount] = {[kMTMetaLines] = kMTFilterSelect };

/// @param apply_folds Only count virtual lines that are not in folds.
int decor_virt_lines(win_T *wp, int start_row, int end_row, int *num_below, VirtLines *lines,
                     bool apply_folds)
{
  buf_T *buf = wp->w_buffer;
  if (!buf_meta_total(buf, kMTMetaLines)) {
    // Only pay for what you use: in case virt_lines feature is not active
    // in a buffer, plines do not need to access the marktree at all
    return 0;
  }

  MarkTreeIter itr[1] = { 0 };
  if (!rs_marktree_itr_get_filter(buf->b_marktree, MAX(start_row - 1, 0), 0, end_row, 0,
                               lines_filter, itr)) {
    return 0;
  }

  assert(start_row >= 0);

  int virt_lines = 0;
  while (true) {
    MTKey mark = rs_marktree_itr_current(itr);
    DecorVirtText *vt = mt_decor_virt(mark);
    if (!mt_invalid(mark) && ns_in_win(mark.ns, wp)) {
      while (vt) {
        if (rs_vt_is_lines(vt->flags)) {
          bool above = rs_vt_is_lines_above(vt->flags);
          int mrow = mark.pos.row;
          int draw_row = mrow + (above ? 0 : 1);
          if (draw_row >= start_row && draw_row < end_row
              && (!apply_folds || !(hasFolding(wp, mrow + 1, NULL, NULL)
                                    || decor_conceal_line(wp, mrow, false)))) {
            virt_lines += (int)kv_size(vt->data.virt_lines);
            if (lines) {
              kv_splice(*lines, vt->data.virt_lines);
            }
            if (num_below && !above) {
              (*num_below) += (int)kv_size(vt->data.virt_lines);
            }
          }
        }
        vt = vt->next;
      }
    }

    if (!rs_marktree_itr_next_filter(buf->b_marktree, itr, end_row, 0, lines_filter)) {
      break;
    }
  }

  return virt_lines;
}

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

uint16_t decor_type_flags(DecorInline decor)
{
  if (decor.ext) {
    uint16_t type_flags = kExtmarkNone;
    DecorVirtText *vt = decor.data.ext.vt;
    while (vt) {
      type_flags |= rs_vt_is_lines(vt->flags) ? kExtmarkVirtLines : kExtmarkVirtText;
      vt = vt->next;
    }
    uint32_t idx = decor.data.ext.sh_idx;
    while (idx != DECOR_ID_INVALID) {
      DecorSignHighlight *sh = &kv_A(decor_items, idx);
      type_flags |= rs_sh_is_sign(sh->flags) ? kExtmarkSign : kExtmarkHighlight;
      idx = sh->next;
    }
    return type_flags;
  } else {
    return rs_sh_is_sign(decor.data.hl.flags) ? kExtmarkSign : kExtmarkHighlight;
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

// ============================================================================
// Rust FFI accessor functions for DecorState
// ============================================================================

/// Get the global decor_state pointer for Rust FFI.
void *nvim_get_decor_state(void)
{
  return &decor_state;
}

/// Check if decor_state.win has a specific buffer.
int nvim_decor_state_win_has_buffer(void *state_ptr, buf_T *buf)
{
  DecorState *state = (DecorState *)state_ptr;
  return (state->win && state->win->w_buffer == buf) ? 1 : 0;
}

/// Destroy decor_state.slots kvec.
void nvim_decor_state_destroy_slots(void *state_ptr)
{
  DecorState *state = (DecorState *)state_ptr;
  kv_destroy(state->slots);
}

/// Destroy decor_state.ranges_i kvec.
void nvim_decor_state_destroy_ranges_i(void *state_ptr)
{
  DecorState *state = (DecorState *)state_ptr;
  kv_destroy(state->ranges_i);
}

// ============================================================================
// Phase 2: Memory management accessors
// ============================================================================

/// Get decor_freelist global.
uint32_t nvim_get_decor_freelist(void)
{
  return decor_freelist;
}

/// Set decor_freelist global.
void nvim_set_decor_freelist(uint32_t val)
{
  decor_freelist = val;
}

/// Get size of decor_items kvec.
uint32_t nvim_decor_items_size(void)
{
  return (uint32_t)kv_size(decor_items);
}

/// Get decor_items[idx].next field.
uint32_t nvim_decor_items_get_next(uint32_t idx)
{
  return kv_A(decor_items, idx).next;
}

/// Set decor_items[idx] to item.
void nvim_decor_items_set(uint32_t idx, DecorSignHighlight item)
{
  kv_A(decor_items, idx) = item;
}

/// Push item to decor_items, return its index.
uint32_t nvim_decor_items_push(DecorSignHighlight item)
{
  uint32_t pos = (uint32_t)kv_size(decor_items);
  kv_push(decor_items, item);
  return pos;
}

/// Get pointer to decor_items[idx].
DecorSignHighlight *nvim_decor_items_get(uint32_t idx)
{
  return &kv_A(decor_items, idx);
}

/// Get opaque pointer to decor_items[idx] (for FFI).
void *nvim_decor_items_get_ptr(uint32_t idx)
{
  return &kv_A(decor_items, idx);
}

/// Allocate a DecorVirtText on the heap.
void *nvim_xmalloc_decor_virt_text(void)
{
  return xmalloc(sizeof(DecorVirtText));
}

/// Free a heap-allocated pointer.
void nvim_xfree_ptr(void *ptr)
{
  xfree(ptr);
}

/// Get to_free_virt global.
void *nvim_get_to_free_virt(void)
{
  return to_free_virt;
}

/// Set to_free_virt global.
void nvim_set_to_free_virt(void *val)
{
  to_free_virt = (DecorVirtText *)val;
}

/// Get to_free_sh global.
uint32_t nvim_get_to_free_sh(void)
{
  return to_free_sh;
}

/// Set to_free_sh global.
void nvim_set_to_free_sh(uint32_t val)
{
  to_free_sh = val;
}

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

/// Get flags from decor_items[idx].
uint16_t nvim_decor_items_get_flags(uint32_t idx)
{
  return kv_A(decor_items, idx).flags;
}

/// Set flags on decor_items[idx].
void nvim_decor_items_set_flags(uint32_t idx, uint16_t flags)
{
  kv_A(decor_items, idx).flags = flags;
}

/// Set next on decor_items[idx].
void nvim_decor_items_set_next(uint32_t idx, uint32_t next)
{
  kv_A(decor_items, idx).next = next;
}

/// Clear and free sign_name on decor_items[idx] if it's a sign.
void nvim_decor_items_clear_sign_name(uint32_t idx)
{
  XFREE_CLEAR(kv_A(decor_items, idx).sign_name);
}

/// Clear and free url on decor_items[idx].
void nvim_decor_items_clear_url(uint32_t idx)
{
  XFREE_CLEAR(kv_A(decor_items, idx).url);
}

/// Get buf->b_marktree->n_keys.
int nvim_buf_get_marktree_n_keys(void *buf_ptr)
{
  buf_T *buf = (buf_T *)buf_ptr;
  return (int)buf->b_marktree->n_keys;
}

/// Get buffer from window: wp->w_buffer.
void *nvim_decor_win_get_buffer(void *wp_ptr)
{
  win_T *wp = (win_T *)wp_ptr;
  return wp->w_buffer;
}

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

// ============================================================================
// Phase 4: Range Insertion and Creation helpers
// ============================================================================

/// Insert a virtual text range into DecorState.
/// Constructs a DecorRange and calls decor_range_insert.
void nvim_decor_range_insert_vt(void *state_ptr, int start_row, int start_col,
                                int end_row, int end_col, void *vt_ptr, bool owned,
                                int kind, uint32_t priority_internal)
{
  DecorState *state = (DecorState *)state_ptr;
  DecorRange range = {
    .start_row = start_row, .start_col = start_col,
    .end_row = end_row, .end_col = end_col,
    .kind = (DecorRangeKind)kind,
    .data.vt = (DecorVirtText *)vt_ptr,
    .attr_id = 0,
    .owned = owned,
    .priority_internal = priority_internal,
    .draw_col = -10,
  };
  decor_range_insert(state, &range);
}

/// Insert a highlight range into DecorState.
/// Constructs a DecorRange with a copy of the DecorSignHighlight and calls decor_range_insert.
void nvim_decor_range_insert_hl(void *state_ptr, int start_row, int start_col,
                                int end_row, int end_col, void *sh_ptr, bool owned,
                                uint32_t priority_internal, int attr_id)
{
  DecorState *state = (DecorState *)state_ptr;
  DecorSignHighlight *sh = (DecorSignHighlight *)sh_ptr;
  DecorRange range = {
    .start_row = start_row, .start_col = start_col,
    .end_row = end_row, .end_col = end_col,
    .kind = kDecorKindHighlight,
    .data.sh = *sh,
    .attr_id = attr_id,
    .owned = owned,
    .priority_internal = priority_internal,
    .draw_col = -10,
  };
  decor_range_insert(state, &range);
}

/// Insert a UI watched range into DecorState.
void nvim_decor_range_insert_ui(void *state_ptr, int start_row, int start_col,
                                int end_row, int end_col, uint32_t ns_id, uint32_t mark_id,
                                int pos, bool owned, uint32_t priority_internal, int attr_id)
{
  DecorState *state = (DecorState *)state_ptr;
  DecorRange range = {
    .start_row = start_row, .start_col = start_col,
    .end_row = end_row, .end_col = end_col,
    .kind = kDecorKindUIWatched,
    .data.ui.ns_id = ns_id,
    .data.ui.mark_id = mark_id,
    .data.ui.pos = pos,
    .attr_id = attr_id,
    .owned = owned,
    .priority_internal = priority_internal,
    .draw_col = -10,
  };
  decor_range_insert(state, &range);
}

/// Get sh->flags from a DecorSignHighlight pointer.
uint16_t nvim_decor_sh_get_flags(void *sh_ptr)
{
  DecorSignHighlight *sh = (DecorSignHighlight *)sh_ptr;
  return sh->flags;
}

/// Get sh->priority from a DecorSignHighlight pointer.
uint16_t nvim_decor_sh_ptr_get_priority(void *sh_ptr)
{
  DecorSignHighlight *sh = (DecorSignHighlight *)sh_ptr;
  return sh->priority;
}

/// Get sh->hl_id from a DecorSignHighlight pointer.
int nvim_decor_sh_ptr_get_hl_id(void *sh_ptr)
{
  DecorSignHighlight *sh = (DecorSignHighlight *)sh_ptr;
  return sh->hl_id;
}

/// Get sh->url from a DecorSignHighlight pointer.
const char *nvim_decor_sh_ptr_get_url(void *sh_ptr)
{
  DecorSignHighlight *sh = (DecorSignHighlight *)sh_ptr;
  return sh->url;
}

/// Get sh->next from a DecorSignHighlight pointer.
uint32_t nvim_decor_sh_ptr_get_next(void *sh_ptr)
{
  DecorSignHighlight *sh = (DecorSignHighlight *)sh_ptr;
  return sh->next;
}

/// Get vt->flags from a DecorVirtText pointer.
uint8_t nvim_decor_vt_ptr_get_flags(void *vt_ptr)
{
  DecorVirtText *vt = (DecorVirtText *)vt_ptr;
  return vt->flags;
}

/// Get vt->priority from a DecorVirtText pointer.
uint16_t nvim_decor_vt_ptr_get_priority(void *vt_ptr)
{
  DecorVirtText *vt = (DecorVirtText *)vt_ptr;
  return vt->priority;
}

/// Get vt->next from a DecorVirtText pointer.
void *nvim_decor_vt_ptr_get_next(void *vt_ptr)
{
  DecorVirtText *vt = (DecorVirtText *)vt_ptr;
  return vt->next;
}

/// Handle the inline highlight path: convert DecorHighlightInline to
/// DecorSignHighlight and add the range via rs_decor_range_add_sh.
void nvim_decor_range_add_from_inline_hl(void *state, int start_row, int start_col,
                                          int end_row, int end_col,
                                          uint16_t hl_flags, uint16_t hl_priority,
                                          int hl_hl_id, uint32_t hl_conceal_char,
                                          bool owned, uint32_t ns, uint32_t mark_id)
{
  DecorHighlightInline hl = {
    .flags = hl_flags,
    .priority = hl_priority,
    .hl_id = hl_hl_id,
    .conceal_char = hl_conceal_char,
  };
  DecorSignHighlight sh = decor_sh_from_inline(hl);
  decor_range_add_sh(state, start_row, start_col, end_row, end_col, &sh, owned, ns, mark_id, 0);
}

// ============================================================================
// Phase 5: Redraw Dispatch and Buffer Operations helpers
// ============================================================================

/// Wrapper for redraw_buf_line_later for Rust FFI.
void nvim_redraw_buf_line_later(void *buf_ptr, int lnum, bool redraw)
{
  redraw_buf_line_later((buf_T *)buf_ptr, lnum, redraw);
}

/// Wrapper for changed_lines_invalidate_buf for Rust FFI.
void nvim_changed_lines_invalidate_buf(void *buf_ptr, int lnum1, int col1, int lnum2, int xtra)
{
  changed_lines_invalidate_buf((buf_T *)buf_ptr, lnum1, col1, lnum2, xtra);
}

/// Wrapper for redraw_buf_range_later for Rust FFI.
void nvim_redraw_buf_range_later(void *buf_ptr, int first, int last)
{
  redraw_buf_range_later((buf_T *)buf_ptr, first, last);
}

/// Get buf->b_ml.ml_line_count.
int nvim_decor_buf_get_line_count(void *buf_ptr)
{
  buf_T *buf = (buf_T *)buf_ptr;
  return buf->b_ml.ml_line_count;
}

/// Get VirtTextPos from a DecorVirtText pointer.
int nvim_decor_vt_ptr_get_pos(void *vt_ptr)
{
  DecorVirtText *vt = (DecorVirtText *)vt_ptr;
  return vt->pos;
}

/// Call decor_redraw_sh with decor_items[idx].
void nvim_decor_redraw_sh_by_idx(void *buf_ptr, int row1, int row2, uint32_t idx)
{
  DecorSignHighlight sh = kv_A(decor_items, idx);
  decor_redraw_sh((buf_T *)buf_ptr, row1, row2, sh);
}

/// Call decor_redraw_sh with inline highlight data.
void nvim_decor_redraw_sh_inline(void *buf_ptr, int row1, int row2,
                                 uint16_t hl_flags, uint16_t hl_priority,
                                 int hl_hl_id, uint32_t hl_conceal_char)
{
  DecorHighlightInline hl = {
    .flags = hl_flags,
    .priority = hl_priority,
    .hl_id = hl_hl_id,
    .conceal_char = hl_conceal_char,
  };
  decor_redraw_sh((buf_T *)buf_ptr, row1, row2, decor_sh_from_inline(hl));
}

/// Call buf_put_decor_sh with decor_items[idx].
void nvim_buf_put_decor_sh_by_idx(void *buf_ptr, uint32_t idx, int row1, int row2)
{
  buf_put_decor_sh((buf_T *)buf_ptr, &kv_A(decor_items, idx), row1, row2);
}

/// Call buf_remove_decor_sh with decor_items[idx].
void nvim_buf_remove_decor_sh_by_idx(void *buf_ptr, int row1, int row2, uint32_t idx)
{
  buf_remove_decor_sh((buf_T *)buf_ptr, row1, row2, &kv_A(decor_items, idx));
}

/// Get the row from decor_state.
int nvim_decor_state_get_row(void *state_ptr)
{
  DecorState *state = (DecorState *)state_ptr;
  return state->row;
}

/// Get the eol_col from decor_state.
int nvim_decor_state_get_eol_col(void *state_ptr)
{
  DecorState *state = (DecorState *)state_ptr;
  return state->eol_col;
}

/// Set the eol_col in decor_state.
void nvim_decor_state_set_eol_col(void *state_ptr, int val)
{
  DecorState *state = (DecorState *)state_ptr;
  state->eol_col = val;
}

/// Get the current_end from decor_state (number of active ranges).
int nvim_decor_state_get_current_end(void *state_ptr)
{
  DecorState *state = (DecorState *)state_ptr;
  return state->current_end;
}

/// Get the future_begin from decor_state.
int nvim_decor_state_get_future_begin(void *state_ptr)
{
  DecorState *state = (DecorState *)state_ptr;
  return state->future_begin;
}

/// Get the count of ranges_i (total number of ranges).
int nvim_decor_state_get_ranges_count(void *state_ptr)
{
  DecorState *state = (DecorState *)state_ptr;
  return (int)kv_size(state->ranges_i);
}

/// Get a DecorRange by slot index from the ranges_i/slots arrays.
/// This accesses ranges_i[idx] to get the slot index, then returns slots[slot_idx].range.
/// Returns NULL if index is out of bounds.
void *nvim_decor_state_get_range_by_idx(void *state_ptr, int idx)
{
  DecorState *state = (DecorState *)state_ptr;
  if (idx < 0 || idx >= (int)kv_size(state->ranges_i)) {
    return NULL;
  }
  int slot_idx = kv_A(state->ranges_i, idx);
  return &kv_A(state->slots, slot_idx).range;
}

/// Get the current attr from decor_state.
int nvim_decor_state_get_current(void *state_ptr)
{
  DecorState *state = (DecorState *)state_ptr;
  return state->current;
}


/// Get the start_row from a DecorRange.
int nvim_decor_range_get_start_row(void *range_ptr)
{
  DecorRange *range = (DecorRange *)range_ptr;
  return range->start_row;
}

/// Get the start_col from a DecorRange.
int nvim_decor_range_get_start_col(void *range_ptr)
{
  DecorRange *range = (DecorRange *)range_ptr;
  return range->start_col;
}


/// Get the draw_col from a DecorRange.
int nvim_decor_range_get_draw_col(void *range_ptr)
{
  DecorRange *range = (DecorRange *)range_ptr;
  return range->draw_col;
}

/// Set the draw_col in a DecorRange.
void nvim_decor_range_set_draw_col(void *range_ptr, int val)
{
  DecorRange *range = (DecorRange *)range_ptr;
  range->draw_col = val;
}

/// Get the kind from a DecorRange.
int nvim_decor_range_get_kind(void *range_ptr)
{
  DecorRange *range = (DecorRange *)range_ptr;
  return range->kind;
}


/// Check if a DecorRange has virtual text position set.
bool nvim_decor_range_has_virt_pos(void *range_ptr)
{
  DecorRange *range = (DecorRange *)range_ptr;
  return decor_virt_pos(range);
}

/// Get the virtual text position kind from a DecorRange.
int nvim_decor_range_get_virt_pos_kind(void *range_ptr)
{
  DecorRange *range = (DecorRange *)range_ptr;
  return decor_virt_pos_kind(range);
}

/// Get the DecorVirtText pointer from a DecorRange (for kDecorKindVirtText).
void *nvim_decor_range_get_virt_text(void *range_ptr)
{
  DecorRange *range = (DecorRange *)range_ptr;
  if (range->kind == kDecorKindVirtText) {
    return range->data.vt;
  }
  return NULL;
}

/// Get the hl_mode from a DecorVirtText.
int nvim_decor_virt_text_get_hl_mode(void *vt_ptr)
{
  DecorVirtText *vt = (DecorVirtText *)vt_ptr;
  return vt->hl_mode;
}

/// Get the pos from a DecorVirtText.
int nvim_decor_virt_text_get_pos(void *vt_ptr)
{
  DecorVirtText *vt = (DecorVirtText *)vt_ptr;
  return vt->pos;
}

/// Get the width from a DecorVirtText.
int nvim_decor_virt_text_get_width(void *vt_ptr)
{
  DecorVirtText *vt = (DecorVirtText *)vt_ptr;
  return vt->width;
}

/// Get the col from a DecorVirtText.
int nvim_decor_virt_text_get_col(void *vt_ptr)
{
  DecorVirtText *vt = (DecorVirtText *)vt_ptr;
  return vt->col;
}

/// Get the flags from a DecorVirtText.
int nvim_decor_virt_text_get_flags(void *vt_ptr)
{
  DecorVirtText *vt = (DecorVirtText *)vt_ptr;
  return vt->flags;
}


// ============================================================================
// Additional accessor functions for draw_virt_text migration
// ============================================================================

/// Get the VirtText data pointer from a DecorVirtText.
void *nvim_decor_virt_text_get_virt_text(void *vt_ptr)
{
  DecorVirtText *vt = (DecorVirtText *)vt_ptr;
  return &vt->data.virt_text;
}

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

/// Get UIWatched data ns_id from a DecorRange.
uint64_t nvim_decor_range_get_ui_ns_id(void *range_ptr)
{
  DecorRange *r = (DecorRange *)range_ptr;
  if (r->kind == kDecorKindUIWatched) {
    return r->data.ui.ns_id;
  }
  return 0;
}

/// Get UIWatched data mark_id from a DecorRange.
uint32_t nvim_decor_range_get_ui_mark_id(void *range_ptr)
{
  DecorRange *r = (DecorRange *)range_ptr;
  if (r->kind == kDecorKindUIWatched) {
    return r->data.ui.mark_id;
  }
  return 0;
}

// ============================================================================
// draw_virt_text helper: high-level iteration over active ranges
// ============================================================================

/// Get an active DecorRange by iteration index.
/// This uses the sorted ranges_i array to get the actual range.
void *nvim_decor_state_get_active_range(void *state_ptr, int i)
{
  DecorState *state = (DecorState *)state_ptr;
  if (i < 0 || i >= (int)kv_size(state->ranges_i)) {
    return NULL;
  }
  int idx = kv_A(state->ranges_i, i);
  return &kv_A(state->slots, idx).range;
}

/// Get the total width of EOL right-aligned virtual text from index i onwards.
/// This is a helper for draw_virt_text EOL right alignment calculation.
int nvim_decor_state_get_eol_right_width(void *state_ptr, int from_idx)
{
  DecorState *state = (DecorState *)state_ptr;
  int total_width = 0;
  int count = (int)kv_size(state->ranges_i);

  for (int j = from_idx; j < state->current_end && j < count; j++) {
    int idx = kv_A(state->ranges_i, j);
    DecorRange *r = &kv_A(state->slots, idx).range;

    if (r->start_row != state->row || !decor_virt_pos(r) || r->draw_col != -1) {
      continue;
    }

    if (decor_virt_pos_kind(r) == kVPosEndOfLineRightAlign) {
      DecorVirtText *vt = NULL;
      if (r->kind == kDecorKindVirtText) {
        vt = r->data.vt;
      }
      if (vt) {
        // An extra space is added for single character spacing
        total_width += (vt->width + 1);
      }
    }
  }

  // Remove one space since no space after last entry
  if (total_width > 0) {
    total_width--;
  }

  return total_width;
}

// ============================================================================
// Additional accessor functions for handle_inline_virtual_text migration
// ============================================================================

/// Wrapper for decor_init_draw_col for Rust FFI.
void nvim_decor_init_draw_col(int win_col, bool hidden, void *item_ptr)
{
  DecorRange *item = (DecorRange *)item_ptr;
  decor_init_draw_col(win_col, hidden, item);
}

/// Get data.vt->data.virt_text pointer from a DecorRange (for inline virtual text).
void *nvim_decor_range_get_virt_inline_data(void *range_ptr)
{
  DecorRange *r = (DecorRange *)range_ptr;
  if (r->kind == kDecorKindVirtText && r->data.vt != NULL) {
    return &r->data.vt->data.virt_text;
  }
  return NULL;
}

/// Get data.vt->hl_mode from a DecorRange.
int nvim_decor_range_get_virt_inline_hl_mode(void *range_ptr)
{
  DecorRange *r = (DecorRange *)range_ptr;
  if (r->kind == kDecorKindVirtText && r->data.vt != NULL) {
    return r->data.vt->hl_mode;
  }
  return 0;  // HL_MODE_UNKNOWN
}

// ============================================================================
// Extmark Decoration Accessor Functions (for Rust FFI - extmark crate)
// ============================================================================

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

/// Invalidate decor state for buffer.
void nvim_decor_state_invalidate(buf_T *buf)
{
  decor_state_invalidate(buf);
}

/// Count sign columns in a range (wrapper for Rust FFI).
void nvim_buf_signcols_count_range(buf_T *buf, int row1, int row2, int add, int clear)
{
  buf_signcols_count_range(buf, row1, row2, add, (TriState)clear);
}

/// Get sign name from DecorSignHighlight (for Rust sign crate).
char *nvim_decor_sh_get_sign_name(DecorSignHighlight *sh)
{
  return sh ? sh->sign_name : NULL;
}

/// Get highlight ID from DecorSignHighlight (for Rust sign crate).
int nvim_decor_sh_get_hl_id(DecorSignHighlight *sh)
{
  return sh ? sh->hl_id : 0;
}

/// Get priority from DecorSignHighlight (for Rust sign crate).
int nvim_decor_sh_get_priority(DecorSignHighlight *sh)
{
  return sh ? sh->priority : 0;
}

/// Get sign_add_id from DecorSignHighlight (for Rust sign crate).
int nvim_decor_sh_get_sign_add_id(DecorSignHighlight *sh)
{
  return sh ? sh->sign_add_id : 0;
}

/// Get type flags for decoration data (for Rust extmark FFI).
uint16_t nvim_decor_type_flags(DecorInlineData data, bool ext)
{
  return decor_type_flags((DecorInline){ .ext = ext, .data = data });
}

// ============================================================================
// Phase 6: Core Column Rendering helpers
// ============================================================================

/// Advance the marktree iterator and promote future ranges to active.
///
/// This handles the first two loops of decor_redraw_col_impl:
/// 1. Marktree iteration: advance iterator, add inline decorations
/// 2. Future-to-active promotion: binary search insertion of promoted ranges
///
/// Returns updated state via DecorColAdvanceResult.
DecorColAdvanceResult nvim_decor_col_advance(win_T *wp, int col, DecorState *state)
{
  buf_T *const buf = wp->w_buffer;
  int const row = state->row;
  int col_until = MAXCOL;

  // Part 1: Advance marktree iterator, adding inline decorations
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
    decor_range_add_from_inline(state, mark.pos.row, mark.pos.col, endpos.row, endpos.col,
                                mt_decor(mark), false, mark.ns, mark.id);

next_mark:
    rs_marktree_itr_next(buf->b_marktree, state->itr);
  }

  int *const indices = state->ranges_i.items;
  DecorRangeSlot *const slots = state->slots.items;

  int count = (int)kv_size(state->ranges_i);
  int cur_end = state->current_end;
  int fut_beg = state->future_begin;

  // Part 2: Promote future ranges before the cursor to active.
  for (; fut_beg < count; fut_beg++) {
    int const index = indices[fut_beg];
    DecorRange *const r = &slots[index].range;
    if (r->start_row > row || (r->start_row == row && r->start_col > col)) {
      break;
    }
    int const ordering = r->ordering;
    DecorPriorityInternal const priority = r->priority_internal;

    int begin = 0;
    int end = cur_end;
    while (begin < end) {
      int mid = begin + ((end - begin) >> 1);
      int mi = indices[mid];
      DecorRange *mr = &slots[mi].range;
      if (mr->priority_internal < priority
          || (mr->priority_internal == priority && mr->ordering < ordering)) {
        begin = mid + 1;
      } else {
        end = mid;
      }
    }

    int *const item = indices + begin;
    memmove(item + 1, item, (size_t)(cur_end - begin) * sizeof(*item));
    *item = index;
    cur_end++;
  }

  if (fut_beg < count) {
    DecorRange *r = &slots[indices[fut_beg]].range;
    if (r->start_row == row) {
      col_until = MIN(col_until, r->start_col - 1);
    }
  }

  return (DecorColAdvanceResult){
    .col_until = col_until,
    .cur_end = cur_end,
    .fut_beg = fut_beg,
    .count = count,
  };
}

