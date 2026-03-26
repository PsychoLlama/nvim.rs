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
extern bool rs_ns_hl_def(int ns_id, int hl_id, HlAttrs attrs, int link_id);
extern HlAttrs rs_dict2hlattrs(Dict dict, bool use_rgb, int *link_id, Error *err);

// C accessor functions (callable from Rust via FFI)

// Namespace globals (ns_hl_global, ns_hl_win, ns_hl_fast, ns_hl_active,
// need_highlight_changed, p_pb, must_redraw_pum) are now accessed directly
// from Rust via extern static declarations (highlight/src/lib.rs).
const int *nvim_get_hl_attr_active(void) { return hl_attr_active; }
void nvim_set_hl_attr_active(const int *attrs) { hl_attr_active = (int *)attrs; }
const int *nvim_get_highlight_attr(void) { return highlight_attr; }

// Popupmenu / option accessors
bool nvim_get_pum_drawn(void) { return pum_drawn(); }
// Called from the popupmenu crate (c_int 0 = false)
void nvim_set_must_redraw_pum(int value) { must_redraw_pum = (bool)value; }

// HLF_* constants are now defined directly in Rust (highlight/src/lib.rs).
// Validated by _Static_assert lines above.
const char *nvim_get_hlf_name(int idx) { return hlf_names[idx]; }

// Arena management
void nvim_arena_init(Arena *a) { *a = (Arena)ARENA_EMPTY; }
void nvim_arena_finish_and_free(Arena *a) { arena_mem_free(arena_finish(a)); }

// Reinit callbacks for clear_hl_tables
void nvim_memset_highlight_attr_last(void) { memset(highlight_attr_last, -1, sizeof(highlight_attr_last)); }
void nvim_call_highlight_attr_set_all(void) { highlight_attr_set_all(); }
void nvim_call_highlight_changed(void) { highlight_changed(); }
void nvim_call_screen_invalidate_highlights(void) { screen_invalidate_highlights(); }


// C callback wrappers (callable from Rust via FFI)

void nvim_ui_call_hl_attr_define(int id, HlAttrs attrs, Array inspect)
{
  ui_call_hl_attr_define(id, attrs, attrs, inspect);
}

void nvim_highlight_emsg_overflow(void) { emsg(_("E424: Too many different highlighting attributes in use")); }

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

// Functions with non-trivial C logic (not pure wrappers)

void ns_hl_def(NS ns_id, int hl_id, HlAttrs attrs, int link_id, Dict(highlight) *dict)
{
  if (ns_id == 0) {
    assert(dict);
    set_hl_group(hl_id, attrs, dict, link_id);
    return;
  }
  rs_ns_hl_def(ns_id, hl_id, attrs, link_id);
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
