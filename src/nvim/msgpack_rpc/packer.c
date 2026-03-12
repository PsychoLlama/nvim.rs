#include <assert.h>
#include <lauxlib.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/memory.h"
#include "nvim/msgpack_rpc/packer.h"
#include "nvim/types_defs.h"

#include "msgpack_rpc/packer.c.generated.h"

// Rust implementations from nvim-msgpack crate
// mpack_check_buffer, mpack_uint64, mpack_integer, mpack_float8, mpack_raw,
// mpack_ext, mpack_handle are exported directly from Rust.
extern void rs_mpack_str(const char *data, size_t len, PackerBuffer *packer);
extern void rs_mpack_bin(const char *data, size_t len, PackerBuffer *packer);

// mpack_str and mpack_bin decompose a String struct before calling Rust.
void mpack_str(String str, PackerBuffer *packer)
{
  rs_mpack_str(str.data, str.size, packer);
}

void mpack_bin(String str, PackerBuffer *packer)
{
  rs_mpack_bin(str.data, str.size, packer);
}

void mpack_object(Object *obj, PackerBuffer *packer)
{
  mpack_object_inner(obj, NULL, 0, packer);
}

void mpack_object_array(Array arr, PackerBuffer *packer)
{
  mpack_array(&packer->ptr, (uint32_t)arr.size);
  if (arr.size > 0) {
    Object container = ARRAY_OBJ(arr);
    mpack_object_inner(&arr.items[0], arr.size > 1 ? &container : NULL, 1, packer);
  }
}

typedef struct {
  Object *container;
  size_t idx;
} ContainerStackItem;

void mpack_object_inner(Object *current, Object *container, size_t container_idx,
                        PackerBuffer *packer)
  FUNC_ATTR_NONNULL_ARG(1, 4)
{
  // The inner loop of this function packs "current" and then fetches the next
  // value from "container". "stack" is only used for nested containers.
  kvec_withinit_t(ContainerStackItem, 2) stack = KV_INITIAL_VALUE;
  kvi_init(stack);

  while (true) {
    mpack_check_buffer(packer);
    switch (current->type) {
    case kObjectTypeLuaRef:
      // TODO(bfredl): could also be an error. Though kObjectTypeLuaRef
      // should only appear when the caller has opted in to handle references,
      // see nlua_pop_Object.
      api_free_luaref(current->data.luaref);
      current->data.luaref = LUA_NOREF;
      FALLTHROUGH;
    case kObjectTypeNil:
      mpack_nil(&packer->ptr);
      break;
    case kObjectTypeBoolean:
      mpack_bool(&packer->ptr, current->data.boolean);
      break;
    case kObjectTypeInteger:
      mpack_integer(&packer->ptr, current->data.integer);
      break;
    case kObjectTypeFloat:
      mpack_float8(&packer->ptr, current->data.floating);
      break;
    case kObjectTypeString:
      mpack_str(current->data.string, packer);
      break;
    case kObjectTypeBuffer:
    case kObjectTypeWindow:
    case kObjectTypeTabpage:
      mpack_handle(current->type, (handle_T)current->data.integer, packer);
      break;
    case kObjectTypeDict:
    case kObjectTypeArray: {}
      size_t current_size;
      if (current->type == kObjectTypeArray) {
        current_size = current->data.array.size;
        mpack_array(&packer->ptr, (uint32_t)current_size);
      } else {
        current_size = current->data.dict.size;
        mpack_map(&packer->ptr, (uint32_t)current_size);
      }
      if (current_size > 0) {
        if (current->type == kObjectTypeArray && current_size == 1) {
          current = &current->data.array.items[0];
          continue;
        }
        if (container) {
          kvi_push(stack, ((ContainerStackItem) {
            .container = container,
            .idx = container_idx,
          }));
        }
        container = current;
        container_idx = 0;
      }
      break;
    }

    if (!container) {
      if (kv_size(stack)) {
        ContainerStackItem it = kv_pop(stack);
        container = it.container;
        container_idx = it.idx;
      } else {
        break;
      }
    }

    if (container->type == kObjectTypeArray) {
      Array arr = container->data.array;
      current = &arr.items[container_idx++];
      if (container_idx >= arr.size) {
        container = NULL;
      }
    } else {
      Dict dict = container->data.dict;
      KeyValuePair *it = &dict.items[container_idx++];
      mpack_check_buffer(packer);
      mpack_str(it->key, packer);
      current = &it->value;
      if (container_idx >= dict.size) {
        container = NULL;
      }
    }
  }
  kvi_destroy(stack);
}

PackerBuffer packer_string_buffer(void)
{
  const size_t initial_size = 64;  // must be larger than SHADA_MPACK_FREE_SPACE
  char *alloc = xmalloc(initial_size);
  return (PackerBuffer) {
    .startptr = alloc,
    .ptr = alloc,
    .endptr = alloc + initial_size,
    .packer_flush = flush_string_buffer,
  };
}

static void flush_string_buffer(PackerBuffer *buffer)
{
  size_t current_capacity = (size_t)(buffer->endptr - buffer->startptr);
  size_t new_capacity = 2 * current_capacity;
  size_t len = (size_t)(buffer->ptr - buffer->startptr);

  buffer->startptr = xrealloc(buffer->startptr, new_capacity);
  buffer->ptr = buffer->startptr + len;
  buffer->endptr = buffer->startptr + new_capacity;
}

/// can only be used with a PackerBuffer from `packer_string_buffer`
String packer_take_string(PackerBuffer *buffer)
{
  return (String){ .data = buffer->startptr, .size = (size_t)(buffer->ptr - buffer->startptr) };
}

// =============================================================================
// Rust accessor functions for opaque handle pattern
// =============================================================================

/// Get the current write pointer from a PackerBuffer (accessor for Rust).
char *nvim_packer_get_ptr(PackerBuffer *packer)
{
  return packer->ptr;
}

/// Set the current write pointer in a PackerBuffer (accessor for Rust).
void nvim_packer_set_ptr(PackerBuffer *packer, char *ptr)
{
  packer->ptr = ptr;
}

/// Get the end pointer from a PackerBuffer (accessor for Rust).
char *nvim_packer_get_endptr(PackerBuffer *packer)
{
  return packer->endptr;
}

/// Get the start pointer from a PackerBuffer (accessor for Rust).
char *nvim_packer_get_startptr(PackerBuffer *packer)
{
  return packer->startptr;
}

/// Call the packer_flush callback (accessor for Rust).
void nvim_packer_flush(PackerBuffer *packer)
{
  if (packer->packer_flush) {
    packer->packer_flush(packer);
  }
}
