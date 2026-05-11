/// @file encode.c
///
/// Vimscript value encoding — thin C shim.
///
/// All encoding functions are implemented in the Rust eval_codec crate
/// (src/nvim-rs/eval_codec/src/encode.rs).  This file retains only:
///   - the two const string arrays that reference C enum values
///     (kBoolVarTrue, kBoolVarFalse, kSpecialVarNull) which are generated in C.
///
/// The Rust crate exports every other symbol with #[export_name = "..."] so
/// existing callers compile and link unchanged.

#include <stdbool.h>
#include <stddef.h>

#include "nvim/eval/encode.h"
#include "nvim/eval/typval_defs.h"

const char *const encode_bool_var_names[] = {
  [kBoolVarTrue] = "v:true",
  [kBoolVarFalse] = "v:false",
};

const char *const encode_special_var_names[] = {
  [kSpecialVarNull] = "v:null",
};
