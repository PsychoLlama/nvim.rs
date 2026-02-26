// clipboard.c: Thin wrappers delegating to Rust (src/nvim-rs/clipboard/src/lib.rs)
// C accessor functions for typval/eval operations needed by Rust.

#include <assert.h>

#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/clipboard.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/message.h"
// Phase 12: rs_eval_call_provider replaces the C eval_call_provider wrapper
extern void rs_eval_call_provider(const char *provider, const char *method,
                                  list_T *arguments, bool discard, typval_T *out_rettv);
#include "nvim/option_vars.h"
#include "nvim/register.h"

#include "clipboard.c.generated.h"

// Static assertions for constants used in Rust
_Static_assert(STAR_REGISTER == 37, "STAR_REGISTER mismatch");
_Static_assert(PLUS_REGISTER == 38, "PLUS_REGISTER mismatch");
_Static_assert(kMTCharWise == 0, "kMTCharWise mismatch");
_Static_assert(kMTLineWise == 1, "kMTLineWise mismatch");
_Static_assert(kMTBlockWise == 2, "kMTBlockWise mismatch");
_Static_assert(kMTUnknown == -1, "kMTUnknown mismatch");
_Static_assert(kOptCbFlagUnnamed == 0x01, "kOptCbFlagUnnamed mismatch");
_Static_assert(kOptCbFlagUnnamedplus == 0x02, "kOptCbFlagUnnamedplus mismatch");
_Static_assert(Ctrl_V == 22, "Ctrl_V mismatch");

// Rust implementations
extern yankreg_T *rs_adjust_clipboard_name(int *name, bool quiet, bool writing);
extern bool rs_get_clipboard(int name, yankreg_T **target, bool quiet);
extern void rs_set_clipboard(int name, yankreg_T *reg);
extern void rs_start_batch_changes(void);
extern void rs_end_batch_changes(void);
extern int rs_save_batch_count(void);
extern void rs_restore_batch_count(int save_count);

// =============================================================================
// C accessor functions for Rust to call back into
// =============================================================================

bool nvim_clipboard_eval_has_provider(void)
{
  return eval_has_provider("clipboard", false);
}

void nvim_clipboard_msg(const char *s)
{
  msg(s, 0);
}

bool nvim_clipboard_redirecting(void)
{
  return redirecting() != 0;
}

/// Provider get: calls eval_call_provider("clipboard","get",...), parses the
/// result, and populates reg.  Returns true on success.
/// This keeps all typval manipulation in C.
bool nvim_clipboard_provider_get(int name, yankreg_T *reg)
{
  bool errmsg = true;

  list_T *const args = tv_list_alloc(1);
  const char regname = (char)name;
  tv_list_append_string(args, &regname, 1);

  typval_T result;
  rs_eval_call_provider("clipboard", "get", args, false, &result);

  if (result.v_type != VAR_LIST) {
    if (result.v_type == VAR_NUMBER && result.vval.v_number == 0) {
      errmsg = false;
    }
    goto err;
  }

  list_T *res = result.vval.v_list;
  list_T *lines = NULL;
  if (tv_list_len(res) == 2
      && TV_LIST_ITEM_TV(tv_list_first(res))->v_type == VAR_LIST) {
    lines = TV_LIST_ITEM_TV(tv_list_first(res))->vval.v_list;
    if (TV_LIST_ITEM_TV(tv_list_last(res))->v_type != VAR_STRING) {
      goto err;
    }
    char *regtype = TV_LIST_ITEM_TV(tv_list_last(res))->vval.v_string;
    if (regtype == NULL || strlen(regtype) > 1) {
      goto err;
    }
    switch (regtype[0]) {
    case 0:
      reg->y_type = kMTUnknown;
      break;
    case 'v':
    case 'c':
      reg->y_type = kMTCharWise;
      break;
    case 'V':
    case 'l':
      reg->y_type = kMTLineWise;
      break;
    case 'b':
    case Ctrl_V:
      reg->y_type = kMTBlockWise;
      break;
    default:
      goto err;
    }
  } else {
    lines = res;
    reg->y_type = kMTUnknown;
  }

  reg->y_array = xcalloc((size_t)tv_list_len(lines), sizeof(String));
  reg->y_size = (size_t)tv_list_len(lines);
  reg->y_width = 0;
  reg->additional_data = NULL;
  reg->timestamp = 0;

  size_t tv_idx = 0;
  TV_LIST_ITER_CONST(lines, li, {
    if (TV_LIST_ITEM_TV(li)->v_type != VAR_STRING) {
      goto err;
    }
    const char *s = TV_LIST_ITEM_TV(li)->vval.v_string;
    reg->y_array[tv_idx++] = cstr_to_string(s != NULL ? s : "");
  });

  if (reg->y_size > 0 && reg->y_array[reg->y_size - 1].size == 0) {
    if (reg->y_type != kMTCharWise) {
      xfree(reg->y_array[reg->y_size - 1].data);
      reg->y_size--;
      if (reg->y_type == kMTUnknown) {
        reg->y_type = kMTLineWise;
      }
    }
  } else {
    if (reg->y_type == kMTUnknown) {
      reg->y_type = kMTCharWise;
    }
  }

  return true;

err:
  if (reg->y_array) {
    for (size_t i = 0; i < reg->y_size; i++) {
      xfree(reg->y_array[i].data);
    }
    xfree(reg->y_array);
  }
  reg->y_array = NULL;
  reg->y_size = 0;
  reg->additional_data = NULL;
  reg->timestamp = 0;
  if (errmsg) {
    emsg("clipboard: provider returned invalid data");
  }
  return false;
}

/// Provider set: builds a list from reg and calls eval_call_provider.
/// This keeps all typval manipulation in C.
void nvim_clipboard_provider_set(int name, yankreg_T *reg)
{
  list_T *const lines = tv_list_alloc((ptrdiff_t)reg->y_size + (reg->y_type != kMTCharWise));

  for (size_t i = 0; i < reg->y_size; i++) {
    tv_list_append_string(lines, reg->y_array[i].data, -1);
  }

  char regtype;
  switch (reg->y_type) {
  case kMTLineWise:
    regtype = 'V';
    tv_list_append_string(lines, NULL, 0);
    break;
  case kMTCharWise:
    regtype = 'v';
    break;
  case kMTBlockWise:
    regtype = 'b';
    tv_list_append_string(lines, NULL, 0);
    break;
  case kMTUnknown:
    abort();
  }

  list_T *args_list = tv_list_alloc(3);
  tv_list_append_list(args_list, lines);
  tv_list_append_string(args_list, &regtype, 1);
  tv_list_append_string(args_list, ((char[]) { (char)name }), 1);

  typval_T rettv;
  rs_eval_call_provider("clipboard", "set", args_list, true, &rettv);
}

// =============================================================================
// Thin wrappers delegating to Rust
// =============================================================================

yankreg_T *adjust_clipboard_name(int *name, bool quiet, bool writing)
{
  return rs_adjust_clipboard_name(name, quiet, writing);
}

bool get_clipboard(int name, yankreg_T **target, bool quiet)
{
  return rs_get_clipboard(name, target, quiet);
}

void set_clipboard(int name, yankreg_T *reg)
{
  rs_set_clipboard(name, reg);
}

void start_batch_changes(void)
{
  rs_start_batch_changes();
}

void end_batch_changes(void)
{
  rs_end_batch_changes();
}

int save_batch_count(void)
{
  return rs_save_batch_count();
}

void restore_batch_count(int save_count)
{
  rs_restore_batch_count(save_count);
}
