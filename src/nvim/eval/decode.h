#pragma once

#include <stdbool.h>  // IWYU pragma: keep
#include <stddef.h>  // IWYU pragma: keep
#include <stdint.h>  // IWYU pragma: keep

#include "mpack/object.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

// Functions implemented in Rust eval_codec crate (export_name bindings).
// The generated header no longer declares them since the C bodies are gone.
list_T *decode_create_map_special_dict(typval_T *ret_tv, ptrdiff_t len);
int json_decode_string(const char *buf, size_t buf_len, typval_T *rettv);
void typval_parser_error_free(mpack_parser_t *parser);
int mpack_parse_typval(mpack_parser_t *parser, const char **data, size_t *size);
int unpack_typval(const char **data, size_t *size, typval_T *ret);

#include "eval/decode.h.generated.h"
