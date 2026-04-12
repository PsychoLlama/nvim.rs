// highlight_group.c: code for managing highlight groups

#include <assert.h>
#include <ctype.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/keysets_defs.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/dispatch.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/private/validate.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/cursor_shape.h"
#include "nvim/decoration_provider.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_docmd.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid_defs.h"
#include "nvim/highlight.h"
#include "nvim/highlight_group.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/map_defs.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/time.h"
#include "nvim/runtime.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/vim_defs.h"


/// \addtogroup SG_SET
/// @{
enum {
  SG_CTERM = 2,  // cterm has been set
  SG_GUI = 4,    // gui has been set
  SG_LINK = 8,   // link has been set
};
/// @}

#define MAX_SYN_NAME 200

// builtin |highlight-groups|
garray_T highlight_ga = GA_EMPTY_INIT_VALUE;

// arena for object with same lifetime as highlight_ga (aka hl_table)
Arena highlight_arena = ARENA_EMPTY;

Map(cstr_t, int) highlight_unames = MAP_INIT;


/// Structure that stores information about a highlight group.
/// The ID of a highlight group is also called group ID.  It is the index in
/// the highlight_ga array PLUS ONE.
typedef struct {
  char *sg_name;                ///< highlight group name
  char *sg_name_u;              ///< uppercase of sg_name
  bool sg_cleared;              ///< "hi clear" was used
  int sg_attr;                  ///< Screen attr @see ATTR_ENTRY
  int sg_link;                  ///< link to this highlight group ID
  int sg_deflink;               ///< default link; restored in highlight_clear()
  int sg_set;                   ///< combination of flags in \ref SG_SET
  sctx_T sg_deflink_sctx;       ///< script where the default link was set
  sctx_T sg_script_ctx;         ///< script in which the group was last set for terminal UIs
  int sg_cterm;                 ///< "cterm=" highlighting attr
                                ///< (combination of \ref HlAttrFlags)
  int sg_cterm_fg;              ///< terminal fg color number + 1
  int sg_cterm_bg;              ///< terminal bg color number + 1
  bool sg_cterm_bold;           ///< bold attr was set for light color for RGB UIs
  int sg_gui;                   ///< "gui=" highlighting attributes
                                ///< (combination of \ref HlAttrFlags)
  RgbValue sg_rgb_fg;           ///< RGB foreground color
  RgbValue sg_rgb_bg;           ///< RGB background color
  RgbValue sg_rgb_sp;           ///< RGB special color
  int sg_rgb_fg_idx;            ///< RGB foreground color index
  int sg_rgb_bg_idx;            ///< RGB background color index
  int sg_rgb_sp_idx;            ///< RGB special color index

  int sg_blend;                 ///< blend level (0-100 inclusive), -1 if unset

  int sg_parent;                ///< parent of @nested.group
} HlGroup;

enum {
  kColorIdxNone = -1,
  kColorIdxHex = -2,
  kColorIdxFg = -3,
  kColorIdxBg = -4,
};

#include "highlight_group.c.generated.h"


#define hl_table ((HlGroup *)((highlight_ga.ga_data)))

// The default highlight groups.  Compiled-in for fast startup.
// Defined in Rust: src/nvim-rs/highlight_group/src/init_tables.rs
extern const char *highlight_init_both[];
extern const char *highlight_init_light[];
extern const char *highlight_init_dark[];


/// Lookup a highlight group by uppercase name.
/// @param name_u Uppercase name to look up (must be null-terminated)
/// @return The highlight group ID (1-based), or 0 if not found
int nvim_highlight_name_lookup(const char *name_u) { return map_get(cstr_t, int)(&highlight_unames, name_u); }


// C accessors called from Rust (Phase 1: syn_add_group migration)

void *nvim_hlg_alloc_entry(int *id_out)
{
  if (highlight_ga.ga_data == NULL) {
    highlight_ga.ga_itemsize = (int)sizeof(HlGroup);
    ga_set_growsize(&highlight_ga, 10);
    ga_grow(&highlight_ga, 300);
  }
  if (highlight_ga.ga_len >= MAX_HL_ID) {
    *id_out = 0;
    return NULL;
  }
  HlGroup *hlgp = GA_APPEND_VIA_PTR(HlGroup, &highlight_ga);
  CLEAR_POINTER(hlgp);
  *id_out = highlight_ga.ga_len;
  return (void *)hlgp;
}

char *nvim_hlg_arena_memdupz(const char *name, size_t len)
{
  return arena_memdupz(&highlight_arena, name, len);
}

void nvim_hlg_vim_strup(char *s)
{
  vim_strup(s);
}

void nvim_hlg_unames_put(const char *name_u, int id)
{
  map_put(cstr_t, int)(&highlight_unames, name_u, id);
}

void nvim_hlg_emsg(const char *msg)
{
  emsg(_(msg));
}

void nvim_hlg_msg_source(void)
{
  msg_source(HLF_W);
}

int nvim_hlg_vim_isprintc(int c)
{
  return vim_isprintc(c);
}

void *nvim_hlg_xmemrchr(const void *s, int c, size_t n)
{
  return xmemrchr(s, c, n);
}


// C bridge for set_hl_group Phase 2 migration

HlGroupSetInfo nvim_hlg_extract_set_info(Dict(highlight) *dict)
{
  HlGroupSetInfo info = { 0 };
  info.force = dict->force;
  Object fg = HAS_KEY(dict, highlight, fg) ? dict->fg : dict->foreground;
  Object bg = HAS_KEY(dict, highlight, bg) ? dict->bg : dict->background;
  Object sp = HAS_KEY(dict, highlight, sp) ? dict->sp : dict->special;
  if (fg.type == kObjectTypeString && fg.data.string.size > 0) {
    info.fg_name = fg.data.string.data;
  }
  if (bg.type == kObjectTypeString && bg.data.string.size > 0) {
    info.bg_name = bg.data.string.data;
  }
  if (sp.type == kObjectTypeString && sp.data.string.size > 0) {
    info.sp_name = sp.data.string.data;
  }
  return info;
}

/// Get current_sctx value.
sctx_T nvim_hlg_get_current_sctx(void)
{
  return current_sctx;
}

/// Get SOURCING_LNUM value.
int nvim_hlg_get_sourcing_lnum(void)
{
  return SOURCING_LNUM;
}

/// Call nlua_set_sctx.
void nvim_hlg_nlua_set_sctx(sctx_T *sctx)
{
  nlua_set_sctx(sctx);
}

/// Call name_to_color.
RgbValue nvim_hlg_name_to_color(const char *name, int *idx)
{
  return name_to_color(name, idx);
}

/// Call highlight_attr_set_all().
void nvim_hlg_highlight_attr_set_all(void)
{
  highlight_attr_set_all();
}

/// Call ui_default_colors_set().
void nvim_hlg_ui_default_colors_set(void)
{
  ui_default_colors_set();
}

/// Call redraw_all_later(UPD_NOT_VALID).
void nvim_hlg_redraw_all_later(void)
{
  redraw_all_later(UPD_NOT_VALID);
}

/// Get updating_screen global.
bool nvim_hlg_updating_screen(void)
{
  return updating_screen;
}

/// Set need_highlight_changed to true.
void nvim_hlg_set_need_highlight_changed(void)
{
  need_highlight_changed = true;
}


// set_hl_group migrated to Rust (Phase 2).
// See src/nvim-rs/highlight_group/src/ffi.rs.


static bool hlgroup2dict(Dict *hl, NS ns_id, int hl_id, Arena *arena)
{
  HlGroup *sgp = &hl_table[hl_id - 1];
  int link = ns_id == 0 ? sgp->sg_link : ns_get_hl(&ns_id, hl_id, true, sgp->sg_set);
  if (link == -1) {
    return false;
  }
  if (ns_id == 0 && sgp->sg_cleared && sgp->sg_set == 0) {
    // table entry was created but not ever set
    return false;
  }
  HlAttrs attr =
    syn_attr2entry(ns_id == 0 ? sgp->sg_attr : ns_get_hl(&ns_id, hl_id, false, sgp->sg_set));
  *hl = arena_dict(arena, HLATTRS_DICT_SIZE + 1);
  if (attr.rgb_ae_attr & HL_DEFAULT) {
    PUT_C(*hl, "default", BOOLEAN_OBJ(true));
  }
  if (link > 0) {
    assert(1 <= link && link <= highlight_ga.ga_len);
    PUT_C(*hl, "link", CSTR_AS_OBJ(hl_table[link - 1].sg_name));
  }
  Dict hl_cterm = arena_dict(arena, HLATTRS_DICT_SIZE);
  hlattrs2dict(hl, NULL, attr, true, true);
  hlattrs2dict(hl, &hl_cterm, attr, false, true);
  if (kv_size(hl_cterm)) {
    PUT_C(*hl, "cterm", DICT_OBJ(hl_cterm));
  }
  return true;
}

Dict ns_get_hl_defs(NS ns_id, Dict(get_highlight) *opts, Arena *arena, Error *err)
{
  Boolean link = GET_BOOL_OR_TRUE(opts, get_highlight, link);
  int id = -1;
  if (HAS_KEY(opts, get_highlight, name)) {
    Boolean create = GET_BOOL_OR_TRUE(opts, get_highlight, create);
    id = create ? syn_check_group(opts->name.data, opts->name.size)
                : syn_name2id_len(opts->name.data, opts->name.size);
    if (id == 0 && !create) {
      Dict attrs = ARRAY_DICT_INIT;
      return attrs;
    }
  } else if (HAS_KEY(opts, get_highlight, id)) {
    id = (int)opts->id;
  }

  if (id != -1) {
    VALIDATE(1 <= id && id <= highlight_ga.ga_len, "%s", "Highlight id out of bounds", {
      goto cleanup;
    });
    Dict attrs = ARRAY_DICT_INIT;
    hlgroup2dict(&attrs, ns_id, link ? id : syn_get_final_id(id), arena);
    return attrs;
  }
  if (ERROR_SET(err)) {
    goto cleanup;
  }

  Dict rv = arena_dict(arena, (size_t)highlight_ga.ga_len);
  for (int i = 1; i <= highlight_ga.ga_len; i++) {
    Dict attrs = ARRAY_DICT_INIT;
    if (!hlgroup2dict(&attrs, ns_id, i, arena)) {
      continue;
    }
    PUT_C(rv, hl_table[(link ? i : syn_get_final_id(i)) - 1].sg_name, DICT_OBJ(attrs));
  }

  return rv;

cleanup:
  return (Dict)ARRAY_DICT_INIT;
}






