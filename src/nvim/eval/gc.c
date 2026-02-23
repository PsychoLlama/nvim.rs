#include <stddef.h>

#include "nvim/eval/gc.h"

#include "eval/gc.c.generated.h"  // IWYU pragma: export

/// Head of list of all dictionaries
dict_T *gc_first_dict = NULL;
/// Head of list of all lists
list_T *gc_first_list = NULL;

/// Get gc_first_dict (accessor for Rust).
dict_T *nvim_gc_get_first_dict(void)
{
  return gc_first_dict;
}

/// Get gc_first_list (accessor for Rust).
list_T *nvim_gc_get_first_list(void)
{
  return gc_first_list;
}
