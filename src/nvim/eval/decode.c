/// Decode JSON/msgpack into typval_T.
///
/// All implementations live in the Rust eval_codec crate
/// (src/nvim-rs/eval_codec/src/decode.rs).  This file retains only:
///   - the thin C wrapper for decode_string() so existing callers compile
///     unchanged (Rust uses the pointer-out form rs_decode_string_into).
///   - a thin wrapper nvim_blob_ga_concat_len() to give Rust a unique symbol
///     that avoids a clashing-extern-declarations warning in the Rust crate.

#include <stdbool.h>
#include <stddef.h>

#include "nvim/eval/decode.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/garray.h"
#include "nvim/memory.h"

extern void rs_decode_string_into(const char *s, size_t len, bool force_blob, bool s_allocated,
                                  typval_T *rettv);

/// Convert char* string to typval_T — thin C wrapper around Rust rs_decode_string_into.
typval_T decode_string(const char *const s, const size_t len, const bool force_blob,
                       const bool s_allocated)
{
  typval_T tv;
  rs_decode_string_into(s, len, force_blob, s_allocated, &tv);
  return tv;
}

/// Wrapper for ga_concat_len with a unique C name to avoid clashing extern
/// declarations in the Rust eval_codec crate (which also uses ga_concat_len
/// through lib.rs with the concrete garray_T* type).
void nvim_blob_ga_concat_len(garray_T *gap, const char *data, size_t len)
{
  ga_concat_len(gap, data, len);
}

// All other functions (decode_create_map_special_dict, json_decode_string,
// mpack_parse_typval, typval_parser_error_free, unpack_typval) are provided
// by the Rust eval_codec crate via export_name attributes and resolve at link
// time from the generated header declarations.
