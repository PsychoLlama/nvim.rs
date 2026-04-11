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

#include "context.c.generated.h"

int kCtxAll = (kCtxRegs | kCtxJumps | kCtxBufs | kCtxGVars | kCtxSFuncs
               | kCtxFuncs);

ContextVec ctx_stack = KV_INITIAL_VALUE;


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
