// Accessor shims for Rust f_* VimL function migration.
// These bridge Rust code to C typval_T internals without exposing the struct layout.

#include <stdbool.h>
#include <stddef.h>
#include <string.h>

#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/errors.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"

// === typval_T array indexing ===

typval_T *nvim_strings_tv_idx(typval_T *argvars, int i)
{
  return &argvars[i];
}

// === rettv field setters ===

void nvim_strings_rettv_set_number(typval_T *rettv, int64_t n)
{
  rettv->vval.v_number = (varnumber_T)n;
}

void nvim_strings_rettv_set_string(typval_T *rettv, char *s)
{
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = s;
}

void nvim_strings_rettv_set_type(typval_T *rettv, int typ)
{
  rettv->v_type = (VarType)typ;
}

// === typval_T field readers ===

int nvim_strings_tv_get_type(typval_T *tv)
{
  return (int)tv->v_type;
}

// === list accessors ===

list_T *nvim_strings_rettv_list(typval_T *rettv)
{
  return rettv->vval.v_list;
}

// === error message string accessors ===

const char *nvim_strings_get_e_invarg(void)
{
  return _(e_invarg);
}

const char *nvim_strings_get_e_invarg2(void)
{
  return _(e_invarg2);
}

const char *nvim_strings_get_e_using_number_as_bool_nr(void)
{
  return _(e_using_number_as_bool_nr);
}
