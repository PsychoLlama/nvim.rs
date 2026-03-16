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

#include <stddef.h>
#include <string.h>

#include "nvim/api/extmark.h"
#include "nvim/api/private/defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/decoration_defs.h"
#include "nvim/extmark.h"
#include "nvim/extmark_defs.h"
#include "nvim/globals.h"
#include "nvim/map_defs.h"
#include "nvim/marktree.h"
#include "nvim/pos_defs.h"
#include "nvim/types_defs.h"
#include "nvim/undo_defs.h"

#include "extmark.c.generated.h"

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

/// Get or create a namespace ID entry in the extmark namespace map.
/// Returns a pointer to the ID counter (creates entry initialized to 0 if not found).
uint32_t *nvim_extmark_ns_put_ref(buf_T *buf, uint32_t ns_id)
{
  return map_put_ref(uint32_t, uint32_t)(buf->b_extmark_ns, ns_id, NULL, NULL);
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

// ============================================================================
// ExtmarkInfoArray Accessor Functions (for Rust FFI)
// ============================================================================

/// Get the size of an ExtmarkInfoArray.
int64_t nvim_extmark_array_size(ExtmarkInfoArray *array)
{
  return array ? (int64_t)kv_size(*array) : 0;
}

/// Push an MTPair onto an ExtmarkInfoArray.
void nvim_extmark_array_push(ExtmarkInfoArray *array, MTPair pair)
{
  if (array) {
    kv_push(*array, pair);
  }
}

// ============================================================================
// Namespace Accessor Functions (for Rust FFI)
// ============================================================================

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
