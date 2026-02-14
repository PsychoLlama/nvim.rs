// Implements extended marks for plugins. Marks sit in a MarkTree
// datastructure which provides both efficient mark insertations/lookups
// and adjustment to text changes. See marktree.c for more details.
//
// A map of pointers to the marks is used for fast lookup by mark id.
//
// Marks are moved by calls to extmark_splice. Some standard interfaces
// mark_adjust and inserted_bytes already adjust marks, check if these are
// being used before adding extmark_splice calls!
//
// Undo/Redo of marks is implemented by storing the call arguments to
// extmark_splice. The list of arguments is applied in extmark_apply_undo.
// We have to copy extmark positions when the extmarks are within a
// deleted/changed region.
//
// Marks live in namespaces that allow plugins/users to segregate marks
// from other users.
//
// Deleting marks only happens when explicitly calling extmark_del, deleting
// over a range of marks will only move the marks. Deleting on a mark will
// leave it in same position unless it is on the EOL of a line.
//
// Extmarks are used to implement buffer decoration. Decoration is mostly
// regarded as an application of extmarks, however for practical reasons code
// that deletes an extmark with decoration will call back into the decoration
// code for redrawing the line with the deleted decoration.

#include <assert.h>
#include <stddef.h>

#include "nvim/api/private/defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/buffer_updates.h"
#include "nvim/decoration.h"
#include "nvim/decoration_defs.h"
#include "nvim/extmark.h"
#include "nvim/extmark_defs.h"
#include "nvim/api/extmark.h"
#include "nvim/globals.h"
#include "nvim/map_defs.h"
#include "nvim/marktree.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/pos_defs.h"
#include "nvim/types_defs.h"
#include "nvim/undo.h"
#include "nvim/undo_defs.h"

#include "extmark.c.generated.h"

// Rust FFI declarations for extmark core functions
extern bool rs_extmark_del_id(buf_T *buf, uint32_t ns_id, uint32_t id);
extern void rs_extmark_del(buf_T *buf, MarkTreeIter *itr, MTKey key, bool restore);
extern bool rs_extmark_clear(buf_T *buf, uint32_t ns_id, int l_row, colnr_T l_col,
                              int u_row, colnr_T u_col);
extern MTPair rs_extmark_from_id(buf_T *buf, uint32_t ns_id, uint32_t id);
extern void rs_extmark_free_all(buf_T *buf);
extern void rs_extmark_splice_delete(buf_T *buf, int l_row, colnr_T l_col, int u_row,
                                      colnr_T u_col, extmark_undo_vec_t *uvp, bool only_copy,
                                      ExtmarkOp op);
extern void rs_extmark_apply_undo(ExtmarkUndoObject undo_info, bool undo);
extern void rs_extmark_adjust(buf_T *buf, linenr_T line1, linenr_T line2, linenr_T amount,
                               linenr_T amount_after, ExtmarkOp undo);
extern void rs_extmark_splice(buf_T *buf, int start_row, colnr_T start_col, int old_row,
                               colnr_T old_col, bcount_t old_byte, int new_row, colnr_T new_col,
                               bcount_t new_byte, ExtmarkOp undo);
extern void rs_extmark_splice_impl(buf_T *buf, int start_row, colnr_T start_col,
                                    bcount_t start_byte, int old_row, colnr_T old_col,
                                    bcount_t old_byte, int new_row, colnr_T new_col,
                                    bcount_t new_byte, ExtmarkOp undo);
extern void rs_extmark_splice_cols(buf_T *buf, int start_row, colnr_T start_col, colnr_T old_col,
                                    colnr_T new_col, ExtmarkOp undo);
extern void rs_extmark_move_region(buf_T *buf, int start_row, colnr_T start_col,
                                    bcount_t start_byte, int extent_row, colnr_T extent_col,
                                    bcount_t extent_byte, int new_row, colnr_T new_col,
                                    bcount_t new_byte, ExtmarkOp undo);

// Rust FFI declarations for extmark helpers (used by extmark_set and extmark_get)
extern int rs_flags_paired(uint16_t flags);
extern int rs_flags_end(uint16_t flags);
extern int rs_flags_right(uint16_t flags);
extern int rs_flags_invalid(uint16_t flags);
extern int rs_flags_invalidate(uint16_t flags);
extern int rs_flags_no_undo(uint16_t flags);
extern int rs_flags_decor_any(uint16_t flags);
extern int rs_flags_decor_ext(uint16_t flags);
extern uint16_t rs_flags_compute(int right_gravity, int no_undo, int invalidate, int decor_ext);
extern int rs_pos_eq(int row1, int col1, int row2, int col2);
extern int rs_row_invalid(int row);
extern int rs_pos_after(int row1, int col1, int row2, int col2);

/// Create or update an extmark
///
/// must not be used during iteration!
void extmark_set(buf_T *buf, uint32_t ns_id, uint32_t *idp, int row, colnr_T col, int end_row,
                 colnr_T end_col, DecorInline decor, uint16_t decor_flags, bool right_gravity,
                 bool end_right_gravity, bool no_undo, bool invalidate, Error *err)
{
  uint32_t *ns = map_put_ref(uint32_t, uint32_t)(buf->b_extmark_ns, ns_id, NULL, NULL);
  uint32_t id = idp ? *idp : 0;

  uint16_t flags = rs_flags_compute(right_gravity, no_undo, invalidate, decor.ext) | decor_flags;
  if (id == 0) {
    id = ++*ns;
  } else {
    MarkTreeIter itr[1] = { 0 };
    MTKey old_mark = marktree_lookup_ns(buf->b_marktree, ns_id, id, false, itr);
    if (old_mark.id) {
      if (rs_flags_paired(old_mark.flags) || end_row > -1) {
        extmark_del_id(buf, ns_id, id);
      } else {
        assert(marktree_itr_valid(itr));
        if (rs_pos_eq(old_mark.pos.row, old_mark.pos.col, row, col)) {
          // not paired: we can revise in place
          if (!rs_flags_invalid(old_mark.flags) && rs_flags_decor_any(old_mark.flags)) {
            mt_itr_rawkey(itr).flags &= (uint16_t) ~MT_FLAG_EXTERNAL_MASK;
            buf_decor_remove(buf, row, row, col, mt_decor(old_mark), true);
          }
          mt_itr_rawkey(itr).flags |= flags;
          mt_itr_rawkey(itr).decor_data = decor.data;
          marktree_revise_meta(buf->b_marktree, itr, old_mark);
          goto revised;
        }
        marktree_del_itr(buf->b_marktree, itr, false);
        if (!rs_flags_invalid(old_mark.flags)) {
          buf_decor_remove(buf, old_mark.pos.row, old_mark.pos.row, old_mark.pos.col,
                           mt_decor(old_mark), true);
        }
      }
    } else {
      *ns = MAX(*ns, id);
    }
  }

  MTKey mark = { { row, col }, ns_id, id, flags, decor.data };

  marktree_put(buf->b_marktree, mark, end_row, end_col, end_right_gravity);
  decor_state_invalidate(buf);

revised:
  if (decor_flags || decor.ext) {
    buf_put_decor(buf, decor, row, end_row > -1 ? end_row : row);
    decor_redraw(buf, row, end_row > -1 ? end_row : row, col, decor);
  }

  if (idp) {
    *idp = id;
  }
}


/// Remove an extmark in "ns_id" by "id"
///
/// @return false on missing id
bool extmark_del_id(buf_T *buf, uint32_t ns_id, uint32_t id)
{
  return rs_extmark_del_id(buf, ns_id, id);
}

/// Remove a (paired) extmark "key" pointed to by "itr"
void extmark_del(buf_T *buf, MarkTreeIter *itr, MTKey key, bool restore)
{
  rs_extmark_del(buf, itr, key, restore);
}

/// Free extmarks in a ns between lines
/// if ns = 0, it means clear all namespaces
bool extmark_clear(buf_T *buf, uint32_t ns_id, int l_row, colnr_T l_col, int u_row, colnr_T u_col)
{
  return rs_extmark_clear(buf, ns_id, l_row, l_col, u_row, u_col);
}

/// @return  the position of marks between a range,
///          marks found at the start or end index will be included.
///
/// if upper_lnum or upper_col are negative the buffer
/// will be searched to the start, or end
/// amount = amount of marks to find or INT64_MAX for all
ExtmarkInfoArray extmark_get(buf_T *buf, uint32_t ns_id, int l_row, colnr_T l_col, int u_row,
                             colnr_T u_col, int64_t amount, ExtmarkType type_filter, bool overlap)
{
  ExtmarkInfoArray array = KV_INITIAL_VALUE;
  MarkTreeIter itr[1];

  if (overlap) {
    // Find all the marks overlapping the start position
    if (!marktree_itr_get_overlap(buf->b_marktree, l_row, l_col, itr)) {
      return array;
    }

    MTPair pair;
    while (marktree_itr_step_overlap(buf->b_marktree, itr, &pair)) {
      push_mark(&array, ns_id, type_filter, pair);
    }
  } else {
    // Find all the marks beginning with the start position
    marktree_itr_get_ext(buf->b_marktree, MTPos(l_row, l_col),
                         itr, false, false, NULL, NULL);
  }

  while ((int64_t)kv_size(array) < amount) {
    MTKey mark = marktree_itr_current(itr);
    if (rs_row_invalid(mark.pos.row) || rs_pos_after(mark.pos.row, mark.pos.col, u_row, u_col)) {
      break;
    }
    if (!rs_flags_end(mark.flags)) {
      MTKey end = marktree_get_alt(buf->b_marktree, mark, NULL);
      push_mark(&array, ns_id, type_filter, mtpair_from(mark, end));
    }
    marktree_itr_next(buf->b_marktree, itr);
  }
  return array;
}

static void push_mark(ExtmarkInfoArray *array, uint32_t ns_id, ExtmarkType type_filter, MTPair mark)
{
  if (!(ns_id == UINT32_MAX || mark.start.ns == ns_id)) {
    return;
  }
  if (type_filter != kExtmarkNone) {
    if (!rs_flags_decor_any(mark.start.flags)) {
      return;
    }
    uint16_t type_flags = decor_type_flags(mt_decor(mark.start));

    if (!(type_flags & type_filter)) {
      return;
    }
  }

  kv_push(*array, mark);
}

/// Lookup an extmark by id
MTPair extmark_from_id(buf_T *buf, uint32_t ns_id, uint32_t id)
{
  return rs_extmark_from_id(buf, ns_id, id);
}

/// free extmarks from the buffer
void extmark_free_all(buf_T *buf)
{
  rs_extmark_free_all(buf);
}

/// invalidate extmarks between range and copy to undo header
///
/// copying is useful when we cannot simply reverse the operation. This will do
/// nothing on redo, enforces correct position when undo.
void extmark_splice_delete(buf_T *buf, int l_row, colnr_T l_col, int u_row, colnr_T u_col,
                           extmark_undo_vec_t *uvp, bool only_copy, ExtmarkOp op)
{
  rs_extmark_splice_delete(buf, l_row, l_col, u_row, u_col, uvp, only_copy, op);
}

/// undo or redo an extmark operation
void extmark_apply_undo(ExtmarkUndoObject undo_info, bool undo)
{
  rs_extmark_apply_undo(undo_info, undo);
}

/// Adjust extmark row for inserted/deleted rows (columns stay fixed).
void extmark_adjust(buf_T *buf, linenr_T line1, linenr_T line2, linenr_T amount,
                    linenr_T amount_after, ExtmarkOp undo)
{
  rs_extmark_adjust(buf, line1, line2, amount, amount_after, undo);
}

// Adjusts extmarks after a text edit, and emits the `on_bytes` event (`:h api-buffer-updates`).
void extmark_splice(buf_T *buf, int start_row, colnr_T start_col, int old_row, colnr_T old_col,
                    bcount_t old_byte, int new_row, colnr_T new_col, bcount_t new_byte,
                    ExtmarkOp undo)
{
  rs_extmark_splice(buf, start_row, start_col, old_row, old_col, old_byte,
                    new_row, new_col, new_byte, undo);
}

void extmark_splice_impl(buf_T *buf, int start_row, colnr_T start_col, bcount_t start_byte,
                         int old_row, colnr_T old_col, bcount_t old_byte, int new_row,
                         colnr_T new_col, bcount_t new_byte, ExtmarkOp undo)
{
  rs_extmark_splice_impl(buf, start_row, start_col, start_byte,
                         old_row, old_col, old_byte,
                         new_row, new_col, new_byte, undo);
}

void extmark_splice_cols(buf_T *buf, int start_row, colnr_T start_col, colnr_T old_col,
                         colnr_T new_col, ExtmarkOp undo)
{
  rs_extmark_splice_cols(buf, start_row, start_col, old_col, new_col, undo);
}

void extmark_move_region(buf_T *buf, int start_row, colnr_T start_col, bcount_t start_byte,
                         int extent_row, colnr_T extent_col, bcount_t extent_byte, int new_row,
                         colnr_T new_col, bcount_t new_byte, ExtmarkOp undo)
{
  rs_extmark_move_region(buf, start_row, start_col, start_byte,
                         extent_row, extent_col, extent_byte,
                         new_row, new_col, new_byte, undo);
}

// ============================================================================
// Global Accessor Functions (for Rust FFI)
// ============================================================================

/// Get the curbuf_splice_pending global.
int nvim_get_curbuf_splice_pending(void)
{
  return curbuf_splice_pending;
}

// ============================================================================
// Extmark Namespace Map Accessor Functions (for Rust FFI)
// ============================================================================

/// Get the size of the extmark namespace map.
size_t nvim_extmark_ns_map_size(buf_T *buf)
{
  return map_size(buf->b_extmark_ns);
}

/// Get a pointer to a namespace ID in the extmark namespace map.
/// Returns NULL if not found. Used for incrementing the ID counter.
uint32_t *nvim_extmark_ns_get_ref(buf_T *buf, uint32_t ns_id)
{
  return map_ref(uint32_t, uint32_t)(buf->b_extmark_ns, ns_id, NULL);
}

/// Delete a namespace ID from the extmark namespace map.
void nvim_extmark_ns_del(buf_T *buf, uint32_t ns_id)
{
  map_del(uint32_t, uint32_t)(buf->b_extmark_ns, ns_id, NULL);
}

/// Destroy and reinitialize the extmark namespace map.
void nvim_extmark_ns_destroy(buf_T *buf)
{
  map_destroy(uint32_t, buf->b_extmark_ns);
  *buf->b_extmark_ns = (Map(uint32_t, uint32_t)) MAP_INIT;
}

// ============================================================================
// Extmark Undo Vector Accessor Functions (for Rust FFI)
// ============================================================================

/// Get the size of an extmark undo vector.
size_t nvim_extmark_undo_vec_size(extmark_undo_vec_t *uvp)
{
  return uvp ? kv_size(*uvp) : 0;
}

/// Push an undo object onto an extmark undo vector.
void nvim_extmark_undo_vec_push(extmark_undo_vec_t *uvp, ExtmarkUndoObject obj)
{
  if (uvp) {
    kv_push(*uvp, obj);
  }
}

/// Get a pointer to the last element of an extmark undo vector.
ExtmarkUndoObject *nvim_extmark_undo_vec_last(extmark_undo_vec_t *uvp)
{
  if (!uvp || kv_size(*uvp) == 0) {
    return NULL;
  }
  return &kv_last(*uvp);
}

/// Delete extmark by ID (wrapper for Rust FFI)
bool nvim_extmark_del_id(buf_T *buf, uint32_t ns_id, uint32_t id)
{
  return extmark_del_id(buf, ns_id, id);
}

// ============================================================================
// Namespace Accessor Functions (for Rust FFI)
// ============================================================================

/// Get the name of a namespace by its ID (wrapper for Rust FFI)
const char *nvim_describe_ns(int ns_id, const char *unknown)
{
  return describe_ns((NS)ns_id, unknown);
}

/// Look up a namespace ID by name (wrapper for Rust FFI)
int nvim_namespace_lookup(const char *name)
{
  if (name == NULL || *name == '\0') {
    return -1;
  }
  String key = { .data = (char *)name, .size = strlen(name) };
  handle_T id = map_get(String, int)(&namespace_ids, key);
  return id > 0 ? (int)id : -1;
}
