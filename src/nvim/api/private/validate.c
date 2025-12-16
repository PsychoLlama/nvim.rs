#include <inttypes.h>
#include <stdio.h>
#include <string.h>

#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/private/validate.h"
#include "nvim/ascii_defs.h"
#include "nvim/globals.h"

/// Creates "Invalid …" message and sets it on `err`.
void api_err_invalid(Error *err, const char *name, const char *val_s, int64_t val_n, bool quote_val)
{
  rs_api_err_invalid(err, name, val_s, val_n, quote_val);
}

/// Creates "Invalid …: expected …" message and sets it on `err`.
void api_err_exp(Error *err, const char *name, const char *expected, const char *actual)
{
  rs_api_err_exp(err, name, expected, actual);
}

bool check_string_array(Array arr, char *name, bool disallow_nl, Error *err)
{
  snprintf(IObuff, sizeof(IObuff), "'%s' item", name);
  for (size_t i = 0; i < arr.size; i++) {
    VALIDATE_T(IObuff, kObjectTypeString, arr.items[i].type, {
      return false;
    });
    // Disallow newlines in the middle of the line.
    if (disallow_nl) {
      const String l = arr.items[i].data.string;
      VALIDATE(!memchr(l.data, NL, l.size), "'%s' item contains newlines", name, {
        return false;
      });
    }
  }
  return true;
}
