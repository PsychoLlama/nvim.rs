// highlight.c: low level code for UI and syntax highlighting

#include <assert.h>

#include "nvim/api/keysets_defs.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
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
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
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

// Rust FFI declarations
extern void rs_highlight_init(void);
extern HlAttrs rs_syn_attr2entry(int attr);
extern const char *rs_hl_get_url(uint32_t index);
extern int rs_hl_combine_attr(int char_attr, int prim_attr);
extern int rs_hl_blend_attrs(int back_attr, int front_attr, bool *through);
extern int rs_hl_get_syn_attr(int ns_id, int idx, HlAttrs at_en);
extern int rs_hl_add_url(int attr, const char *url);
extern int rs_hl_get_ui_attr(int ns_id, int idx, int final_id, bool optional);
extern int rs_win_bg_attr(win_T *wp);
extern bool rs_ns_hl_def(int ns_id, int hl_id, HlAttrs attrs, int link_id);
extern bool rs_hl_check_ns(void);
extern bool rs_win_check_ns_hl(win_T *wp);
extern int rs_hl_apply_winblend(int winbl, int attr);
extern void rs_update_window_hl(win_T *wp, bool invalid);
extern int rs_hl_get_underline(void);
extern int rs_hl_get_term_attr(HlAttrs *aep);
extern void rs_clear_hl_tables_full(bool reinit);
extern void rs_hl_invalidate_blends_full(void);
extern bool rs_highlight_use_hlstate_full(void);
extern void rs_ui_send_all_hls(RemoteUI *ui);
extern Dict rs_hl_get_attr_by_id(Integer attr_id, Boolean rgb, Arena *arena, Error *err);
extern void rs_hlattrs2dict(Dict *hl, Dict *hl_attrs, HlAttrs ae, bool use_rgb, bool short_keys);
extern HlAttrs rs_dict2hlattrs(Dict dict, bool use_rgb, int *link_id, Error *err);
extern int rs_object_to_color(Object val, char *key, bool rgb, Error *err);
extern Array rs_hl_inspect(int attr, Arena *arena);
extern void rs_update_ns_hl(int ns_id);
extern bool rs_get_hlstate_active(void);
extern int rs_ns_get_hl_full(NS *ns_hl, int hl_id, bool link, bool nodefault);

// ============================================================================
// C accessor functions (callable from Rust via FFI)
// ============================================================================

// Namespace globals
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
void nvim_set_need_highlight_changed(bool value) { need_highlight_changed = value; }

// Popupmenu / option accessors
int nvim_get_p_pb(void) { return (int)p_pb; }
bool nvim_get_pum_drawn(void) { return pum_drawn(); }
void nvim_set_must_redraw_pum(bool value) { must_redraw_pum = value; }

// HLF_* enum constant accessors
int nvim_get_hlf_pni(void) { return HLF_PNI; }
int nvim_get_hlf_pst(void) { return HLF_PST; }
int nvim_get_hlf_none(void) { return HLF_NONE; }
int nvim_get_hlf_inactive(void) { return HLF_INACTIVE; }
int nvim_get_hlf_nfloat(void) { return HLF_NFLOAT; }
int nvim_get_hlf_border(void) { return HLF_BORDER; }
int nvim_get_hlf_count(void) { return HLF_COUNT; }
int nvim_get_hlf_mc(void) { return HLF_MC; }
int nvim_get_hlf_cul(void) { return HLF_CUL; }
const char *nvim_get_hlf_name(int idx) { return hlf_names[idx]; }

// Arena management
void nvim_arena_init(Arena *a) { *a = (Arena)ARENA_EMPTY; }
void nvim_arena_finish_and_free(Arena *a) { arena_mem_free(arena_finish(a)); }

// Reinit callbacks for clear_hl_tables
void nvim_memset_highlight_attr_last(void) { memset(highlight_attr_last, -1, sizeof(highlight_attr_last)); }
void nvim_call_highlight_attr_set_all(void) { highlight_attr_set_all(); }
void nvim_call_highlight_changed(void) { highlight_changed(); }
void nvim_call_screen_invalidate_highlights(void) { screen_invalidate_highlights(); }

// Wrappers that call Rust
void nvim_update_ns_hl(int ns_id) { rs_update_ns_hl(ns_id); }
bool nvim_get_hlstate_active(void) { return rs_get_hlstate_active(); }

// ============================================================================
// C callback wrappers (callable from Rust via FFI)
// ============================================================================

void nvim_ui_call_hl_attr_define(int id, HlAttrs attrs, Array inspect)
{
  ui_call_hl_attr_define(id, attrs, attrs, inspect);
}

void nvim_highlight_emsg_overflow(void)
{
  emsg(_("E424: Too many different highlighting attributes in use"));
}

void nvim_remote_ui_hl_attr_define(RemoteUI *ui, int id, HlAttrs attrs, Array inspect)
{
  remote_ui_hl_attr_define(ui, (Integer)id, attrs, attrs, inspect);
}

void nvim_remote_ui_hl_group_set(RemoteUI *ui, const char *name, int id)
{
  remote_ui_hl_group_set(ui, cstr_as_string(name), id);
}

void nvim_hl_invalidate_blends_callbacks(void)
{
  highlight_changed();
  update_window_hl(curwin, true);
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

// ============================================================================
// Thin wrappers delegating to Rust
// ============================================================================

void highlight_init(void)
{
  rs_highlight_init();
}

/// @return true if hl table was reset
bool highlight_use_hlstate(void)
{
  return rs_highlight_use_hlstate_full();
}

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
    set_hl_group(hl_id, attrs, dict, link_id);
    return;
  }
  rs_ns_hl_def(ns_id, hl_id, attrs, link_id);
}

int ns_get_hl(NS *ns_hl, int hl_id, bool link, bool nodefault)
{
  return rs_ns_get_hl_full(ns_hl, hl_id, link, nodefault);
}

bool hl_check_ns(void)
{
  return rs_hl_check_ns();
}

bool win_check_ns_hl(win_T *wp)
{
  return rs_win_check_ns_hl(wp);
}

/// Get attribute code for a builtin highlight group.
int hl_get_ui_attr(int ns_id, int idx, int final_id, bool optional)
{
  return rs_hl_get_ui_attr(ns_id, idx, final_id, optional);
}

int hl_apply_winblend(int winbl, int attr)
{
  return rs_hl_apply_winblend(winbl, attr);
}

void update_window_hl(win_T *wp, bool invalid)
{
  rs_update_window_hl(wp, invalid);
}

int win_bg_attr(win_T *wp)
{
  return rs_win_bg_attr(wp);
}

int hl_get_underline(void)
{
  return rs_hl_get_underline();
}

int hl_add_url(int attr, const char *url)
{
  return rs_hl_add_url(attr, url);
}

const char *hl_get_url(uint32_t index)
{
  return rs_hl_get_url(index);
}

int hl_get_term_attr(HlAttrs *aep)
{
  return rs_hl_get_term_attr(aep);
}

void clear_hl_tables(bool reinit)
{
  rs_clear_hl_tables_full(reinit);
}

void hl_invalidate_blends(void)
{
  rs_hl_invalidate_blends_full();
}

int hl_combine_attr(int char_attr, int prim_attr)
{
  return rs_hl_combine_attr(char_attr, prim_attr);
}

int hl_blend_attrs(int back_attr, int front_attr, bool *through)
{
  return rs_hl_blend_attrs(back_attr, front_attr, through);
}

HlAttrs syn_attr2entry(int attr)
{
  return rs_syn_attr2entry(attr);
}

Dict hl_get_attr_by_id(Integer attr_id, Boolean rgb, Arena *arena, Error *err)
{
  return rs_hl_get_attr_by_id(attr_id, rgb, arena, err);
}

void hlattrs2dict(Dict *hl, Dict *hl_attrs, HlAttrs ae, bool use_rgb, bool short_keys)
{
  rs_hlattrs2dict(hl, hl_attrs, ae, use_rgb, short_keys);
}

// Generated table for Dict(highlight) fields
extern KeySetLink highlight_table[];

HlAttrs dict2hlattrs(Dict(highlight) *dict, bool use_rgb, int *link_id, Error *err)
{
  Arena arena = ARENA_EMPTY;
  Dict raw = api_keydict_to_dict(dict, highlight_table, 30, &arena);
  HlAttrs attrs = rs_dict2hlattrs(raw, use_rgb, link_id, err);
  arena_mem_free(arena_finish(&arena));
  return attrs;
}

int object_to_color(Object val, char *key, bool rgb, Error *err)
{
  return rs_object_to_color(val, key, rgb, err);
}

Array hl_inspect(int attr, Arena *arena)
{
  return rs_hl_inspect(attr, arena);
}
