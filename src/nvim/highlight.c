// highlight.c: low level code for UI and syntax highlighting

#include <assert.h>
#include <inttypes.h>

#include <lauxlib.h>
#include <string.h>

#include "nvim/api/keysets_defs.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/dispatch.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/private/validate.h"
#include "nvim/api/ui.h"
#include "nvim/decoration_defs.h"
#include "nvim/decoration_provider.h"
#include "nvim/drawscreen.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/map_defs.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/popupmenu.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/vim_defs.h"

#include "highlight.c.generated.h"

_Static_assert(HLF_COUNT == 76, "HLF_COUNT changed - update Rust HLF_COUNT in highlight/src/lib.rs");
_Static_assert(HLF_MSGSEP == 61, "HLF_MSGSEP changed - update Rust constants");
_Static_assert(HLF_W == 26, "HLF_W changed - update Rust constants");
_Static_assert(HLF_E == 6, "HLF_E changed - update Rust constants");
_Static_assert(HLF_S == 19, "HLF_S changed - update Rust constants");
_Static_assert(HLF_MSG == 63, "HLF_MSG changed - update Rust constants");

// Rust FFI declarations - highlight storage and computation is handled by Rust

// Input struct for rs_hl_combine_attrs_compute
typedef struct {
  HlAttrs char_aep;
  HlAttrs prim_aep;
} HlCombineInput;

extern HlAttrs rs_hl_combine_attrs_compute(HlCombineInput input);

// Input struct for rs_hl_blend_attrs_compute
typedef struct {
  HlAttrs battrs_raw;
  HlAttrs battrs;
  HlAttrs fattrs_raw;
  HlAttrs fattrs;
  int ratio;
  bool through;
} HlBlendInput;

extern HlAttrs rs_hl_blend_attrs_compute(HlBlendInput input);

// Attribute entry management functions
extern void rs_highlight_init(void);
extern HlAttrs rs_syn_attr2entry(int attr);

// URL functions
extern const char *rs_hl_get_url(uint32_t index);

// Full attribute combination functions (Phase 14)
extern int rs_hl_combine_attr(int char_attr, int prim_attr);
extern int rs_hl_blend_attrs(int back_attr, int front_attr, bool *through);
extern int rs_hl_get_syn_attr(int ns_id, int idx, HlAttrs at_en);
extern int rs_hl_add_url(int attr, const char *url);

// UI highlight attribute function (Phase 15)
extern int rs_hl_get_ui_attr(int ns_id, int idx, int final_id, bool optional);

// Window background attribute function (Phase 16)
extern int rs_win_bg_attr(win_T *wp);

// Namespace highlight storage functions (ns_hls now in Rust)
extern bool rs_ns_hls_has(int ns_id, int syn_id);
extern ColorItem rs_ns_hls_get(int ns_id, int syn_id);
extern void rs_ns_hls_put(int ns_id, int syn_id, ColorItem item);

// Per-namespace UI highlight attribute functions (ns_hl_attr now in Rust)
extern const int *rs_ns_hl_attr_get(int ns_id);
extern int *rs_ns_hl_attr_get_or_create(int ns_id);

// Namespace highlight definition (ns_hl_def core logic in Rust)
extern bool rs_ns_hl_def(int ns_id, int hl_id, HlAttrs attrs, int link_id);

// Result from rs_ns_get_hl_pre()
typedef struct {
  int ns_id;
  bool need_callback;
  int result;
  bool set_ns_to_zero;
  ColorItem item;
} NsGetHlPreResult;

// Namespace highlight lookup (ns_get_hl pre/post split)
extern NsGetHlPreResult rs_ns_get_hl_pre(int ns_hl, int hl_id, bool link, bool nodefault);
extern NsGetHlPreResult rs_ns_get_hl_post(int ns_id, int hl_id, HlAttrs attrs, int link_id,
                                          bool fallback, int version_offset, bool link,
                                          bool nodefault);

// hl_check_ns implementation in Rust
extern bool rs_hl_check_ns(void);

// C accessor functions for namespace globals (callable from Rust)
int nvim_get_ns_hl_global(void) { return ns_hl_global; }
void nvim_set_ns_hl_global(int ns) { ns_hl_global = ns; }

int nvim_get_ns_hl_win(void) { return ns_hl_win; }
void nvim_set_ns_hl_win(int ns) { ns_hl_win = ns; }

int nvim_get_ns_hl_fast(void) { return ns_hl_fast; }
void nvim_set_ns_hl_fast(int ns) { ns_hl_fast = ns; }

int nvim_get_ns_hl_active(void) { return ns_hl_active; }
void nvim_set_ns_hl_active(int ns) { ns_hl_active = ns; }

const int *nvim_get_hl_attr_active(void) { return hl_attr_active; }
void nvim_set_hl_attr_active(const int *attrs) { hl_attr_active = (int *)attrs; }

const int *nvim_get_highlight_attr(void) { return highlight_attr; }

// Accessor for need_highlight_changed global
void nvim_set_need_highlight_changed(bool value) { need_highlight_changed = value; }

// Accessors for hl_get_ui_attr (Phase 15)
int nvim_get_p_pb(void) { return (int)p_pb; }
bool nvim_get_pum_drawn(void) { return pum_drawn(); }
void nvim_set_must_redraw_pum(bool value) { must_redraw_pum = value; }
int nvim_get_hlf_pni(void) { return HLF_PNI; }
int nvim_get_hlf_pst(void) { return HLF_PST; }

// Accessors for win_bg_attr (Phase 16)
int nvim_get_hlf_none(void) { return HLF_NONE; }
int nvim_get_hlf_inactive(void) { return HLF_INACTIVE; }

// Accessors for hl_inspect (Phase 18)
const char *nvim_get_hlf_name(int idx) { return hlf_names[idx]; }

// Accessors for update_window_hl (Phase 17)
int nvim_get_hlf_nfloat(void) { return HLF_NFLOAT; }
int nvim_get_hlf_border(void) { return HLF_BORDER; }
int nvim_get_hlf_count(void) { return HLF_COUNT; }
int nvim_get_hlf_mc(void) { return HLF_MC; }
int nvim_get_hlf_cul(void) { return HLF_CUL; }
// nvim_get_highlight_attr is already defined above (line 148)

// Arena management accessors (callable from Rust)
void nvim_arena_init(Arena *a) { *a = (Arena)ARENA_EMPTY; }
void nvim_arena_finish_and_free(Arena *a) { arena_mem_free(arena_finish(a)); }

// Accessors for clear_hl_tables reinit callbacks (Phase 2)
void nvim_memset_highlight_attr_last(void) { memset(highlight_attr_last, -1, sizeof(highlight_attr_last)); }
void nvim_call_highlight_attr_set_all(void) { highlight_attr_set_all(); }
void nvim_call_highlight_changed(void) { highlight_changed(); }
void nvim_call_screen_invalidate_highlights(void) { screen_invalidate_highlights(); }

extern void rs_update_ns_hl(int ns_id);
// Wrapper for update_ns_hl - calls Rust version
void nvim_update_ns_hl(int ns_id) { rs_update_ns_hl(ns_id); }

// hlstate_active is now owned by Rust (ATTR_STORE.hlstate_active)
// This accessor calls Rust to get the value
extern bool rs_get_hlstate_active(void);
bool nvim_get_hlstate_active(void) { return rs_get_hlstate_active(); }

// C wrapper for UI dispatch - callable from Rust
// Sends hl_attr_define event to all UIs
void nvim_ui_call_hl_attr_define(int id, HlAttrs attrs, Array inspect)
{
  ui_call_hl_attr_define(id, attrs, attrs, inspect);
}

// C wrapper for emsg - callable from Rust
void nvim_highlight_emsg_overflow(void)
{
  emsg(_("E424: Too many different highlighting attributes in use"));
}

// C wrapper for remote_ui_hl_attr_define - callable from Rust
void nvim_remote_ui_hl_attr_define(RemoteUI *ui, int id, HlAttrs attrs, Array inspect)
{
  remote_ui_hl_attr_define(ui, (Integer)id, attrs, attrs, inspect);
}

// C wrapper for remote_ui_hl_group_set - callable from Rust
void nvim_remote_ui_hl_group_set(RemoteUI *ui, const char *name, int id)
{
  remote_ui_hl_group_set(ui, cstr_as_string(name), id);
}

void highlight_init(void)
{
  // Rust handles the attribute entry store including the dummy entry at index 0
  rs_highlight_init();
}

extern bool rs_highlight_use_hlstate_full(void);

/// @return true if hl table was reset
bool highlight_use_hlstate(void)
{
  return rs_highlight_use_hlstate_full();
}

extern void rs_ui_send_all_hls(RemoteUI *ui);

/// When a UI connects, we need to send it the table of highlights used so far.
void ui_send_all_hls(RemoteUI *ui)
{
  rs_ui_send_all_hls(ui);
}

/// Get attribute code for a syntax group.
int hl_get_syn_attr(int ns_id, int idx, HlAttrs at_en)
{
  return rs_hl_get_syn_attr(ns_id, idx, at_en);
}

void ns_hl_def(NS ns_id, int hl_id, HlAttrs attrs, int link_id, Dict(highlight) *dict)
{
  if (ns_id == 0) {
    assert(dict);
    // set in global (':highlight') namespace
    set_hl_group(hl_id, attrs, dict, link_id);
    return;
  }
  // Namespace highlight definition is handled by Rust
  rs_ns_hl_def(ns_id, hl_id, attrs, link_id);
}

// C bridge for ns_get_hl Lua callback.
// Handles recursion guard, DecorProvider lookup, args building, and nlua_call_ref.
NsGetHlLuaResult c_ns_get_hl_lua_call(int ns_id, int hl_id, bool link)
{
  static int recursive = 0;
  NsGetHlLuaResult result = { .ret = NIL, .is_recursive = false };

  if (recursive) {
    result.is_recursive = true;
    return result;
  }

  DecorProvider *p = get_decor_provider(ns_id, true);

  MAXSIZE_TEMP_ARRAY(args, 3);
  ADD_C(args, INTEGER_OBJ((Integer)ns_id));
  ADD_C(args, CSTR_AS_OBJ(syn_id2name(hl_id)));
  ADD_C(args, BOOLEAN_OBJ(link));

  Error err = ERROR_INIT;
  recursive++;
  result.ret = nlua_call_ref(p->hl_def, "hl_def", args, kRetObject, NULL, &err);
  recursive--;

  return result;
}

extern int rs_ns_get_hl_full(NS *ns_hl, int hl_id, bool link, bool nodefault);

int ns_get_hl(NS *ns_hl, int hl_id, bool link, bool nodefault)
{
  return rs_ns_get_hl_full(ns_hl, hl_id, link, nodefault);
}

bool hl_check_ns(void)
{
  // Namespace switching logic is implemented in Rust
  return rs_hl_check_ns();
}

/// prepare for drawing window `wp` or global elements if NULL
///
/// Note: pum should be drawn in the context of the current window!
extern bool rs_win_check_ns_hl(win_T *wp);

bool win_check_ns_hl(win_T *wp)
{
  return rs_win_check_ns_hl(wp);
}

/// Get attribute code for a builtin highlight group.
int hl_get_ui_attr(int ns_id, int idx, int final_id, bool optional)
{
  return rs_hl_get_ui_attr(ns_id, idx, final_id, optional);
}

/// Apply 'winblend' to highlight attributes.
///
/// @param winbl The 'winblend' value.
/// @param attr  The original attribute code.
///
/// @return      The attribute code with 'winblend' applied.
extern int rs_hl_apply_winblend(int winbl, int attr);

int hl_apply_winblend(int winbl, int attr)
{
  return rs_hl_apply_winblend(winbl, attr);
}

extern void rs_update_window_hl(win_T *wp, bool invalid);

void update_window_hl(win_T *wp, bool invalid)
{
  rs_update_window_hl(wp, invalid);
}


int win_bg_attr(win_T *wp)
{
  return rs_win_bg_attr(wp);
}

/// Gets HL_UNDERLINE highlight.
extern int rs_hl_get_underline(void);

int hl_get_underline(void)
{
  return rs_hl_get_underline();
}

/// Augment an existing attribute with a URL.
int hl_add_url(int attr, const char *url)
{
  return rs_hl_add_url(attr, url);
}

/// Get a URL by its index.
///
/// @param index URL index
/// @return URL
const char *hl_get_url(uint32_t index)
{
  // Rust handles URL storage
  return rs_hl_get_url(index);
}

/// Get attribute code for forwarded :terminal highlights.
extern int rs_hl_get_term_attr(HlAttrs *aep);

int hl_get_term_attr(HlAttrs *aep)
{
  return rs_hl_get_term_attr(aep);
}

extern void rs_clear_hl_tables_full(bool reinit);

/// Clear all highlight tables.
void clear_hl_tables(bool reinit)
{
  rs_clear_hl_tables_full(reinit);
}

// Callback for rs_hl_invalidate_blends_full - runs after blend caches are cleared
void nvim_hl_invalidate_blends_callbacks(void)
{
  highlight_changed();
  update_window_hl(curwin, true);
}

extern void rs_hl_invalidate_blends_full(void);

void hl_invalidate_blends(void)
{
  rs_hl_invalidate_blends_full();
}

// Combine special attributes (e.g., for spelling) with other attributes.
int hl_combine_attr(int char_attr, int prim_attr)
{
  return rs_hl_combine_attr(char_attr, prim_attr);
}

/// Blend overlay attributes (for popupmenu) with other attributes.
int hl_blend_attrs(int back_attr, int front_attr, bool *through)
{
  return rs_hl_blend_attrs(back_attr, front_attr, through);
}

/// Get highlight attributes for a attribute code
HlAttrs syn_attr2entry(int attr)
{
  // Rust handles bounds checking and returns HLATTRS_INIT equivalent for invalid IDs
  return rs_syn_attr2entry(attr);
}

extern Dict rs_hl_get_attr_by_id(Integer attr_id, Boolean rgb, Arena *arena, Error *err);

/// Gets highlight description for id `attr_id` as a map.
Dict hl_get_attr_by_id(Integer attr_id, Boolean rgb, Arena *arena, Error *err)
{
  return rs_hl_get_attr_by_id(attr_id, rgb, arena, err);
}

/// Converts an HlAttrs into Dict
///
/// @param[in/out] hl Dict with pre-allocated space for HLATTRS_DICT_SIZE elements
/// @param[in] aep data to convert
/// @param use_rgb use 'gui*' settings if true, else resorts to 'cterm*'
/// @param short_keys change (foreground, background, special) to (fg, bg, sp) for 'gui*' settings
///                          (foreground, background) to (ctermfg, ctermbg) for 'cterm*' settings
extern void rs_hlattrs2dict(Dict *hl, Dict *hl_attrs, HlAttrs ae, bool use_rgb, bool short_keys);

void hlattrs2dict(Dict *hl, Dict *hl_attrs, HlAttrs ae, bool use_rgb, bool short_keys)
{
  rs_hlattrs2dict(hl, hl_attrs, ae, use_rgb, short_keys);
}

extern HlAttrs rs_dict2hlattrs(Dict dict, bool use_rgb, int *link_id, Error *err);

// Generated table for Dict(highlight) fields
extern KeySetLink highlight_table[];

HlAttrs dict2hlattrs(Dict(highlight) *dict, bool use_rgb, int *link_id, Error *err)
{
  // Convert Dict(highlight)* to raw Dict for Rust parsing
  Arena arena = ARENA_EMPTY;
  Dict raw = api_keydict_to_dict(dict, highlight_table, 30, &arena);
  HlAttrs attrs = rs_dict2hlattrs(raw, use_rgb, link_id, err);
  arena_mem_free(arena_finish(&arena));
  return attrs;
}

extern int rs_object_to_color(Object val, char *key, bool rgb, Error *err);

int object_to_color(Object val, char *key, bool rgb, Error *err)
{
  return rs_object_to_color(val, key, rgb, err);
}

extern Array rs_hl_inspect(int attr, Arena *arena);

Array hl_inspect(int attr, Arena *arena)
{
  return rs_hl_inspect(attr, arena);
}
