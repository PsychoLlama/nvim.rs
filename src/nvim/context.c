// Context: snapshot of the entire editor state as one big object/map

#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "nvim/api/keysets_defs.h"
#include "nvim/api/private/converter.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/vimscript.h"
#include "nvim/context.h"
#include "nvim/eval/encode.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/userfunc.h"
#include "nvim/ex_docmd.h"
#include "nvim/hashtab.h"
#include "nvim/keycodes.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"

#include "context.c.generated.h"

extern size_t rs_ctx_size(void);
extern void rs_ctx_free(Context *ctx);
extern Context *rs_ctx_get(size_t index);
extern void rs_ctx_free_all(void);
extern void rs_ctx_save(Context *ctx, int flags);
extern bool rs_ctx_restore(Context *ctx, int flags);
extern Dict rs_ctx_to_dict(Context *ctx, Arena *arena);
extern int rs_ctx_from_dict(Dict dict, Context *ctx, Error *err);

int kCtxAll = (kCtxRegs | kCtxJumps | kCtxBufs | kCtxGVars | kCtxSFuncs
               | kCtxFuncs);

static ContextVec ctx_stack = KV_INITIAL_VALUE;

/// Clears and frees the context stack
void ctx_free_all(void)
{
  rs_ctx_free_all();
}

/// Returns the size of the context stack.
size_t ctx_size(void)
  FUNC_ATTR_PURE
{
  return rs_ctx_size();
}

/// Returns pointer to Context object with given zero-based index from the top
/// of context stack or NULL if index is out of bounds.
Context *ctx_get(size_t index)
  FUNC_ATTR_PURE
{
  return rs_ctx_get(index);
}

/// Free resources used by Context object.
///
/// param[in]  ctx  pointer to Context object to free.
void ctx_free(Context *ctx)
  FUNC_ATTR_NONNULL_ALL
{
  rs_ctx_free(ctx);
}

/// Saves the editor state to a context.
///
/// If "context" is NULL, pushes context on context stack.
/// Use "flags" to select particular types of context.
///
/// @param  ctx    Save to this context, or push on context stack if NULL.
/// @param  flags  Flags, see ContextTypeFlags enum.
void ctx_save(Context *ctx, const int flags)
{
  rs_ctx_save(ctx, flags);
}

/// Restores the editor state from a context.
///
/// If "context" is NULL, pops context from context stack.
/// Use "flags" to select particular types of context.
///
/// @param  ctx    Restore from this context. Pop from context stack if NULL.
/// @param  flags  Flags, see ContextTypeFlags enum.
///
/// @return true on success, false otherwise (i.e.: empty context stack).
bool ctx_restore(Context *ctx, const int flags)
{
  return rs_ctx_restore(ctx, flags);
}

/// Converts Context to Dict representation.
Dict ctx_to_dict(Context *ctx, Arena *arena)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_ctx_to_dict(ctx, arena);
}

/// Converts Dict representation of Context back to Context object.
int ctx_from_dict(Dict dict, Context *ctx, Error *err)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_ctx_from_dict(dict, ctx, err);
}

// Rust FFI accessor functions

// Compile-time verification that Rust constants match C values
_Static_assert(kCtxRegs == 1, "kCtxRegs must be 1");
_Static_assert(kCtxJumps == 2, "kCtxJumps must be 2");
_Static_assert(kCtxBufs == 4, "kCtxBufs must be 4");
_Static_assert(kCtxGVars == 8, "kCtxGVars must be 8");
_Static_assert(kCtxSFuncs == 16, "kCtxSFuncs must be 16");
_Static_assert(kCtxFuncs == 32, "kCtxFuncs must be 32");

size_t nvim_get_ctx_stack_size(void)
{
  return kv_size(ctx_stack);
}

/// Returns a pointer to Context at given index from the top of the stack.
/// Returns NULL if index is out of bounds.
Context *nvim_get_ctx_at_index(size_t index)
{
  if (index < kv_size(ctx_stack)) {
    return &kv_Z(ctx_stack, index);
  }
  return NULL;
}

/// Returns a pointer to Context at given forward index.
/// Returns NULL if index is out of bounds.
Context *nvim_ctx_stack_at_forward(size_t index)
{
  if (index < kv_size(ctx_stack)) {
    return &kv_A(ctx_stack, index);
  }
  return NULL;
}

/// Push a CONTEXT_INIT onto the stack.
void nvim_ctx_stack_push_init(void)
{
  kv_push(ctx_stack, CONTEXT_INIT);
}

/// Returns a pointer to the last element of the stack.
Context *nvim_ctx_stack_last(void)
{
  return &kv_last(ctx_stack);
}

/// Pops the last element from the stack and returns a pointer to the
/// (still-allocated) slot. The memory is valid until the next push/destroy.
Context *nvim_ctx_stack_pop(void)
{
  // kv_pop decrements size and returns the value.
  // The slot memory at items[size] is still allocated, so we can return its address.
  (void)kv_pop(ctx_stack);
  return &ctx_stack.items[ctx_stack.size];
}

/// Destroy the stack (free items array, reset to empty).
void nvim_ctx_stack_destroy(void)
{
  kv_destroy(ctx_stack);
}

// ShaDa option save/restore accessors.
// Wraps the save-set-restore pattern for the shada option so Rust
// never needs to touch the OptVal struct.
static OptVal saved_shada_opt;

void nvim_ctx_save_shada_opt(void)
{
  saved_shada_opt = get_option_value(kOptShada, OPT_GLOBAL);
}

void nvim_ctx_set_shada_restore(void)
{
  set_option_value(kOptShada, STATIC_CSTR_AS_OPTVAL("!,'100,%"), OPT_GLOBAL);
}

void nvim_ctx_restore_shada_opt(void)
{
  set_option_value(kOptShada, saved_shada_opt, OPT_GLOBAL);
  optval_free(saved_shada_opt);
}

/// Restores functions from a context (C accessor for Rust).
/// Kept in C due to do_cmdline_cmd coupling.
void nvim_ctx_restore_funcs(Context *ctx)
{
  for (size_t i = 0; i < ctx->funcs.size; i++) {
    do_cmdline_cmd(ctx->funcs.items[i].data.string.data);
  }
}

/// Converts readfile()-style array to String (C accessor for Rust).
/// Kept in C due to object_to_vim, encode_vim_list_to_buf coupling.
static inline String nvim_ctx_array_to_string(Array array, Error *err)
{
  String sbuf = STRING_INIT;

  typval_T list_tv;
  object_to_vim(ARRAY_OBJ(array), &list_tv, err);

  assert(list_tv.v_type == VAR_LIST);
  if (!encode_vim_list_to_buf(list_tv.vval.v_list, &sbuf.size, &sbuf.data)) {
    api_set_error(err, kErrorTypeException, "%s",
                  "E474: Failed to convert list to msgpack string buffer");
  }

  tv_clear(&list_tv);
  return sbuf;
}

/// Converts Context to Dict representation (C accessor for Rust).
/// Kept in C due to Arena, PUT_C, string_to_array, copy_array coupling.
Dict nvim_ctx_to_dict_impl(Context *ctx, Arena *arena)
{
  assert(ctx != NULL);

  Dict rv = arena_dict(arena, 5);

  PUT_C(rv, "regs", ARRAY_OBJ(string_to_array(ctx->regs, false, arena)));
  PUT_C(rv, "jumps", ARRAY_OBJ(string_to_array(ctx->jumps, false, arena)));
  PUT_C(rv, "bufs", ARRAY_OBJ(string_to_array(ctx->bufs, false, arena)));
  PUT_C(rv, "gvars", ARRAY_OBJ(string_to_array(ctx->gvars, false, arena)));
  PUT_C(rv, "funcs", ARRAY_OBJ(copy_array(ctx->funcs, arena)));

  return rv;
}

/// Converts Dict representation of Context back to Context object (C accessor for Rust).
/// Kept in C due to array_to_string, copy_object, ERROR_SET coupling.
int nvim_ctx_from_dict_impl(Dict dict, Context *ctx, Error *err)
{
  assert(ctx != NULL);

  int types = 0;
  for (size_t i = 0; i < dict.size && !ERROR_SET(err); i++) {
    KeyValuePair item = dict.items[i];
    if (item.value.type != kObjectTypeArray) {
      continue;
    }
    if (strequal(item.key.data, "regs")) {
      types |= kCtxRegs;
      ctx->regs = nvim_ctx_array_to_string(item.value.data.array, err);
    } else if (strequal(item.key.data, "jumps")) {
      types |= kCtxJumps;
      ctx->jumps = nvim_ctx_array_to_string(item.value.data.array, err);
    } else if (strequal(item.key.data, "bufs")) {
      types |= kCtxBufs;
      ctx->bufs = nvim_ctx_array_to_string(item.value.data.array, err);
    } else if (strequal(item.key.data, "gvars")) {
      types |= kCtxGVars;
      ctx->gvars = nvim_ctx_array_to_string(item.value.data.array, err);
    } else if (strequal(item.key.data, "funcs")) {
      types |= kCtxFuncs;
      ctx->funcs = copy_object(item.value, NULL).data.array;
    }
  }

  return types;
}

/// Saves functions to a context (C accessor for Rust).
/// Kept in C due to HASHTAB_ITER, exec_impl, and Dict(exec_opts) coupling.
void nvim_ctx_save_funcs(Context *ctx, bool scriptonly)
{
  ctx->funcs = (Array)ARRAY_DICT_INIT;
  Error err = ERROR_INIT;

  HASHTAB_ITER(func_tbl_get(), hi, {
    const char *const name = hi->hi_key;
    bool islambda = (strncmp(name, "<lambda>", 8) == 0);
    bool isscript = ((uint8_t)name[0] == K_SPECIAL);

    if (!islambda && (!scriptonly || isscript)) {
      size_t cmd_len = sizeof("func! ") + strlen(name);
      char *cmd = xmalloc(cmd_len);
      snprintf(cmd, cmd_len, "func! %s", name);
      Dict(exec_opts) opts = { .output = true };
      String func_body = exec_impl(VIML_INTERNAL_CALL, cstr_as_string(cmd),
                                   &opts, &err);
      xfree(cmd);
      if (!ERROR_SET(&err)) {
        ADD(ctx->funcs, STRING_OBJ(func_body));
      }
      api_clear_error(&err);
    }
  });
}
