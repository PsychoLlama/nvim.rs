#pragma once

#include <stdint.h>  // IWYU pragma: keep
#include <stdio.h>  // IWYU pragma: keep

#include "nvim/api/keysets_defs.h"  // IWYU pragma: keep
#include "nvim/api/private/defs.h"  // IWYU pragma: keep
#include "nvim/cmdexpand_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/mapping_defs.h"  // IWYU pragma: keep
#include "nvim/option_defs.h"  // IWYU pragma: keep
#include "nvim/regexp_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

#include "mapping.h.generated.h"

// Declarations for functions migrated to Rust (via #[export_name]).
// These were previously auto-generated from mapping.c definitions.
// NOTE: bool parameters are declared as int to match Rust c_int ABI.

void map_mode_to_chars(int mode, char *buf);
int do_map(int maptype, char *arg, int mode, int is_abbrev);
void map_clear_mode(buf_T *buf, int mode, int local, int abbr);
int map_to_exists(const char *str, const char *modechars, int abbr);
int map_to_exists_mode(const char *rhs, int mode, int abbr);
char *set_context_in_map_cmd(expand_T *xp, char *cmd, char *arg, int forceit, int isabbrev,
                             int isunmap, int cmdidx);
int ExpandMappings(char *pat, regmatch_T *regmatch, int *numMatches, char ***matches);
char *check_map(char *keys, int mode, int exact, int ign_mod, int abbr, mapblock_T **mp_ptr,
                int *local_ptr, int *rhs_lua);
int langmap_adjust_mb(int c);
void langmap_init(void);

/// Used for the first argument of do_map()
enum {
  MAPTYPE_MAP       = 0,
  MAPTYPE_UNMAP     = 1,
  MAPTYPE_NOREMAP   = 2,
  MAPTYPE_UNMAP_LHS = 3,
};

/// Adjust chars in a language according to 'langmap' option.
/// NOTE that there is no noticeable overhead if 'langmap' is not set.
/// When set the overhead for characters < 256 is small.
/// Don't apply 'langmap' if the character comes from the Stuff buffer or from a
/// mapping and the langnoremap option was set.
/// The do-while is just to ignore a ';' after the macro.
#define LANGMAP_ADJUST(c, condition) \
  do { \
    if (*p_langmap \
        && (condition) \
        && (p_lrm || (vgetc_busy ? typebuf_maplen() == 0 : KeyTyped)) \
        && !KeyStuffed \
        && (c) >= 0) \
    { \
      if ((c) < 256) \
      c = langmap_mapchar[c]; \
      else \
      c = langmap_adjust_mb(c); \
    } \
  } while (0)
