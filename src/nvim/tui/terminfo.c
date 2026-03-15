// Built-in fallback terminfo entries.

#include <stdbool.h>
#include <string.h>

#ifdef HAVE_UNIBILIUM
# include <unibilium.h>
#endif

#include "klib/kvec.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/charset.h"
#include "nvim/memory.h"
#include "nvim/tui/terminfo.h"
#include "nvim/tui/terminfo_builtin.h"

#ifdef __FreeBSD__
# include "nvim/os/os.h"
#endif

#include "tui/terminfo.c.generated.h"

extern bool terminfo_is_term_family(const char *term, const char *family);
extern bool terminfo_is_bsd_console(const char *term);

bool terminfo_from_database(TerminfoEntry *ti, char *termname, Arena *arena)
{
#ifdef HAVE_UNIBILIUM
  unibi_term *ut = unibi_from_term(termname);
  if (!ut) {
    return false;
  }

  ti->bce = unibi_get_bool(ut, unibi_back_color_erase);
  ti->max_colors = unibi_get_num(ut, unibi_max_colors);
  ti->lines = unibi_get_num(ut, unibi_lines);
  ti->columns = unibi_get_num(ut, unibi_columns);

  // Check for Tc or RGB
  ti->has_Tc_or_RGB = false;
  ti->Su = false;
  for (size_t i = 0; i < unibi_count_ext_bool(ut); i++) {
    const char *n = unibi_get_ext_bool_name(ut, i);
    if (n && (!strcmp(n, "Tc") || !strcmp(n, "RGB"))) {
      ti->has_Tc_or_RGB = true;
    } else if (n && !strcmp(n, "Su")) {
      ti->Su = true;
    }
  }

  static const enum unibi_string uni_ids[] = {
# define X(name) unibi_##name,
    XLIST_TERMINFO_BUILTIN
# undef X
  };

  for (size_t i = 0; i < ARRAY_SIZE(uni_ids); i++) {
    const char *val = unibi_get_str(ut, uni_ids[i]);
    ti->defs[i] = val ? arena_strdup(arena, val) : NULL;
  }

  static const char *uni_ext[] = {
# define X(informal_name, terminfo_name) #terminfo_name,
    XLIST_TERMINFO_EXT
# undef X
  };

  size_t max = unibi_count_ext_str(ut);
  for (size_t i = 0; i < ARRAY_SIZE(uni_ext); i++) {
    const char *name = uni_ext[i];
    for (size_t val = 0; val < max; val++) {
      const char *n = unibi_get_ext_str_name(ut, val);
      if (n && strequal(n, name)) {
        const char *data = unibi_get_ext_str(ut, val);
        ti->defs[kTermExtOffset + i] = data ? arena_strdup(arena, data) : NULL;
        break;
      }
    }
  }

# define X(name) { unibi_key_##name, unibi_string_begin_ },
# define Y(name) { unibi_key_##name, unibi_key_s##name },
  static const enum unibi_string uni_keys[][2] = {
    XYLIST_TERMINFO_KEYS
  };
# undef X
# undef Y

  for (size_t i = 0; i < ARRAY_SIZE(uni_keys); i++) {
    const char *val = unibi_get_str(ut, uni_keys[i][0]);
    if (val) {
      ti->keys[i][0] = arena_strdup(arena, val);
      if (uni_keys[i][1] != unibi_string_begin_) {
        const char *sval = unibi_get_str(ut, uni_keys[i][1]);
        ti->keys[i][1] = sval ? arena_strdup(arena, sval) : NULL;
      }
    }
  }

  static const enum unibi_string uni_fkeys[] = {
# define X(name) unibi_key_##name,
    XLIST_TERMINFO_FKEYS
# undef X
  };

  for (size_t i = 0; i < ARRAY_SIZE(uni_fkeys); i++) {
    const char *val = unibi_get_str(ut, uni_fkeys[i]);
    ti->f_keys[i] = val ? arena_strdup(arena, val) : NULL;
  }

  unibi_destroy(ut);
  return true;
#else
  return false;
#endif
}

static const char *fmt(bool val)
{
  return val ? "true" : "false";
}

/// Dumps termcap info to the messages area.
/// Serves a similar purpose as Vim `:set termcap` (removed in Nvim).
///
/// @return allocated string
String terminfo_info_msg(const TerminfoEntry *ti, const char *termname, bool from_db)
{
  StringBuilder data = KV_INITIAL_VALUE;

  kv_printf(data, "&term: %s\n", termname);
  if (from_db) {
    kv_printf(data, "using terminfo database\n");
  } else {
    kv_printf(data, "using builtin terminfo\n");
  }
  kv_printf(data, "\n");

  kv_printf(data, "Boolean capabilities:\n");
  kv_printf(data, "  back_color_erase: %s\n", fmt(ti->bce));
  kv_printf(data, "  truecolor ('Tc' or 'RGB'): %s\n", fmt(ti->has_Tc_or_RGB));
  kv_printf(data, "  extended underline ('Su'): %s\n", fmt(ti->Su));
  kv_printf(data, "\n");

  kv_printf(data, "Numeric capabilities: (-1 for unknown)\n");
  kv_printf(data, "  lines: %d\n", ti->lines);
  kv_printf(data, "  columns: %d\n", ti->columns);
  kv_printf(data, "  max_colors: %d\n", ti->columns);
  kv_printf(data, "\n");

  kv_printf(data, "String capabilities:\n");

  static const char *string_names[] = {
#define X(name) #name,
    XLIST_TERMINFO_BUILTIN
#undef X
#define X(internal_name, terminfo_name) (#internal_name " (" #terminfo_name ")"),
    XLIST_TERMINFO_EXT
#undef X
  };

  for (size_t i = 0; i < ARRAY_SIZE(string_names); i++) {
    const char *s = ti->defs[i];
    if (s) {
      kv_printf(data, "  %-31s = ", string_names[i]);
      // Most of these strings will contain escape sequences.
      kv_transstr(&data, s, false);
      kv_push(data, '\n');
    }
  }

  static const char *key_names[] = {
#define X(name) #name,
#define Y(name) #name,
    XYLIST_TERMINFO_KEYS
#undef X
#undef Y
  };

  for (size_t i = 0 + 1; i < ARRAY_SIZE(key_names); i++) {
    const char *s = ti->keys[i][0];
    if (s) {
      kv_printf(data, "  key_%-27s = ", key_names[i]);
      kv_transstr(&data, s, false);
      const char *ss = ti->keys[i][1];
      if (ss) {
        kv_printf(data, ", key_s%s = ", key_names[i]);
        kv_transstr(&data, ss, false);
      }
      kv_push(data, '\n');
    }
  }

  static const char *fkey_names[] = {
#define X(name) #name,
    XLIST_TERMINFO_FKEYS
#undef X
  };

  for (size_t i = 0 + 1; i < ARRAY_SIZE(fkey_names); i++) {
    const char *s = ti->f_keys[i];
    if (s) {
      kv_printf(data, "  key_%-27s = ", fkey_names[i]);
      kv_transstr(&data, s, false);
      kv_push(data, '\n');
    }
  }

  kv_push(data, NUL);
  return cbuf_as_string(data.items, data.size - 1);
}

