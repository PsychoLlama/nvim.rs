// Context shim: small C helpers that Rust cannot implement directly.
// These are kept in C because they use macros or types too complex to
// replicate in Rust (typval_T, HASHTAB_ITER, exec_impl keyset types).

#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "nvim/api/private/converter.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/vimscript.h"
#include "nvim/eval/encode.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/userfunc.h"
#include "nvim/hashtab.h"
#include "nvim/keycodes.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/context.h"

/// Converts readfile()-style array to String.
/// Called from Rust ctx_from_dict_impl via FFI.
/// Kept in C due to typval_T stack allocation and encode_vim_list_to_buf coupling.
String nvim_ctx_array_to_string(Array array, Error *err)
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

/// Saves functions to a context.
/// Called from Rust rs_ctx_save via FFI.
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
