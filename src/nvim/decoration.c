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

    rs_decor_range_add_from_inline(state, pair.start.pos.row, pair.start.pos.col,
                                   pair.end_pos.row, pair.end_pos.col,
                                   mt_decor(m).ext, mt_decor(m).data.ext.vt,
                                   mt_decor(m).data.ext.sh_idx,
                                   mt_decor(m).data.hl.flags, mt_decor(m).data.hl.priority,
                                   mt_decor(m).data.hl.hl_id, mt_decor(m).data.hl.conceal_char,
                                   false, m.ns, m.id);
  }

  return true;  // TODO(bfredl): check if available in the region
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
bool win_lines_concealed(win_T *wp) { return rs_hasAnyFolding(wp) || wp->w_p_cole >= 2; }

/// Wrapper for win_lines_concealed for Rust FFI.
int nvim_win_lines_concealed(win_T *wp) { return win_lines_concealed(wp) ? 1 : 0; }

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

/// Invalidate decor state for buffer.
void nvim_decor_state_invalidate(buf_T *buf) { decor_state_invalidate(buf); }

/// Count sign columns in a range (wrapper for Rust FFI).
void nvim_buf_signcols_count_range(buf_T *buf, int row1, int row2, int add, int clear)
{
  buf_signcols_count_range(buf, row1, row2, add, (TriState)clear);
}

/// Get sign name from DecorSignHighlight (for Rust sign crate).
char *nvim_decor_sh_get_sign_name(DecorSignHighlight *sh) { return sh ? sh->sign_name : NULL; }

/// Get highlight ID from DecorSignHighlight (for Rust sign crate).
int nvim_decor_sh_get_hl_id(DecorSignHighlight *sh) { return sh ? sh->hl_id : 0; }

/// Get priority from DecorSignHighlight (for Rust sign crate).
int nvim_decor_sh_get_priority(DecorSignHighlight *sh) { return sh ? sh->priority : 0; }

/// Get sign_add_id from DecorSignHighlight (for Rust sign crate).
int nvim_decor_sh_get_sign_add_id(DecorSignHighlight *sh) { return sh ? sh->sign_add_id : 0; }

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

