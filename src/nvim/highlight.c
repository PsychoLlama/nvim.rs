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
extern int rs_attr_entry_count(void);
extern HlAttrs rs_syn_attr2entry(int attr);
extern HlEntry rs_get_attr_entry_by_id(int attr);

// Result type for rs_get_attr_entry
typedef struct {
  int id;
  bool is_new;
} GetAttrEntryResult;

extern GetAttrEntryResult rs_get_attr_entry(HlEntry entry);
extern void rs_clear_hl_tables(bool reinit);
extern bool rs_highlight_use_hlstate(void);
extern void rs_hl_invalidate_blends(void);

// Cache functions
extern int rs_combine_cache_get(int combine_tag);
extern void rs_combine_cache_put(int combine_tag, int id);
extern int rs_blend_cache_get(int combine_tag, bool through);
extern void rs_blend_cache_put(int combine_tag, int id, bool through);

// URL functions
extern uint32_t rs_hl_add_url_index(const char *url);
extern const char *rs_hl_get_url(uint32_t index);

// Color forcing
extern HlAttrs rs_get_colors_force(HlAttrs attrs);

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

// Forward declaration for update_ns_hl wrapper
static void update_ns_hl(int ns_id);

// Wrapper for update_ns_hl (called from Rust)
void nvim_update_ns_hl(int ns_id) { update_ns_hl(ns_id); }

static bool hlstate_active = false;

void highlight_init(void)
{
  // Rust handles the attribute entry store including the dummy entry at index 0
  rs_highlight_init();
}

/// @return true if hl table was reset
bool highlight_use_hlstate(void)
{
  if (hlstate_active) {
    return false;
  }
  hlstate_active = true;
  // Notify Rust about hlstate mode change
  rs_highlight_use_hlstate();
  // hl tables must now be rebuilt.
  clear_hl_tables(true);
  return true;
}

/// Return the attr number for a set of colors and font, and optionally
/// a semantic description (see ext_hlstate documentation).
/// Add a new entry to the attr_entries array if the combination is new.
/// @return 0 for error.
static int get_attr_entry(HlEntry entry)
{
  // Rust handles hlstate_active normalization internally
  static bool recursive = false;
  bool retried = false;

retry: {}
  GetAttrEntryResult rs_result = rs_get_attr_entry(entry);

  if (rs_result.id == -1) {
    // Table overflow - Rust signals this with id == -1
    // Running out of attribute entries! Remove all attributes and
    // compute new ones for all groups.
    // When called recursively, we are really out of numbers.
    if (recursive || retried) {
      emsg(_("E424: Too many different highlighting attributes in use"));
      return 0;
    }
    recursive = true;

    clear_hl_tables(true);

    recursive = false;
    if (entry.kind == kHlCombine) {
      // This entry is now invalid, don't put it
      return 0;
    }
    retried = true;
    goto retry;
  }

  if (!rs_result.is_new) {
    // Existing entry - just return the ID
    return rs_result.id;
  }

  // New attr id, send event to remote UIs
  int id = rs_result.id;

  Arena arena = ARENA_EMPTY;
  Array inspect = hl_inspect(id, &arena);

  // Note: internally we don't distinguish between cterm and rgb attributes,
  // remote_ui_hl_attr_define will however.
  ui_call_hl_attr_define(id, entry.attr, entry.attr, inspect);
  arena_mem_free(arena_finish(&arena));
  return id;
}

/// When a UI connects, we need to send it the table of highlights used so far.
void ui_send_all_hls(RemoteUI *ui)
{
  int count = rs_attr_entry_count();
  for (int i = 1; i < count; i++) {
    Arena arena = ARENA_EMPTY;
    Array inspect = hl_inspect(i, &arena);
    HlAttrs attr = rs_syn_attr2entry(i);
    remote_ui_hl_attr_define(ui, (Integer)i, attr, attr, inspect);
    arena_mem_free(arena_finish(&arena));
  }
  for (size_t hlf = 0; hlf < HLF_COUNT; hlf++) {
    remote_ui_hl_group_set(ui, cstr_as_string(hlf_names[hlf]),
                           highlight_attr[hlf]);
  }
}

/// Get attribute code for a syntax group.
int hl_get_syn_attr(int ns_id, int idx, HlAttrs at_en)
{
  // TODO(bfredl): should we do this unconditionally
  if (at_en.cterm_fg_color != 0 || at_en.cterm_bg_color != 0
      || at_en.rgb_fg_color != -1 || at_en.rgb_bg_color != -1
      || at_en.rgb_sp_color != -1 || at_en.cterm_ae_attr != 0
      || at_en.rgb_ae_attr != 0 || ns_id != 0) {
    return get_attr_entry((HlEntry){ .attr = at_en, .kind = kHlSyntax,
                                     .id1 = idx, .id2 = ns_id });
  }
  // If all the fields are cleared, clear the attr field back to default value
  return 0;
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

#ifdef USE_RUST_HIGHLIGHT
// Wrapper for ns_get_hl callable from Rust
// nodefault is sg_set flags in this context
int c_ns_get_hl(int *ns_id, int hl_id, bool link, int nodefault)
{
  NS ns = *ns_id;
  int result = ns_get_hl(&ns, hl_id, link, nodefault);
  *ns_id = ns;
  return result;
}

// Wrapper for get_attr_entry callable from Rust
// Allows Rust to create highlight attribute entries with proper UI dispatch
int c_get_attr_entry(HlEntry entry)
{
  return get_attr_entry(entry);
}
#endif

int ns_get_hl(NS *ns_hl, int hl_id, bool link, bool nodefault)
{
  static int recursive = 0;

  // Pre-callback phase: check cache, resolve namespace
  NsGetHlPreResult pre = rs_ns_get_hl_pre(*ns_hl, hl_id, link, nodefault);
  *ns_hl = pre.ns_id;

  // If no callback needed, return the result directly
  if (!pre.need_callback) {
    if (pre.set_ns_to_zero) {
      *ns_hl = 0;
    }
    return pre.result;
  }

  // Lua callback phase - only runs if cache miss and callback is defined
  if (recursive) {
    // Avoid infinite recursion
    return -1;
  }

  int ns_id = pre.ns_id;
  DecorProvider *p = get_decor_provider(ns_id, true);
  ColorItem it = pre.item;

  MAXSIZE_TEMP_ARRAY(args, 3);
  ADD_C(args, INTEGER_OBJ((Integer)ns_id));
  ADD_C(args, CSTR_AS_OBJ(syn_id2name(hl_id)));
  ADD_C(args, BOOLEAN_OBJ(link));

  Error err = ERROR_INIT;
  recursive++;
  Object ret = nlua_call_ref(p->hl_def, "hl_def", args, kRetObject, NULL, &err);
  recursive--;

  // Parse Lua callback result
  bool fallback = true;
  int tmp = false;
  HlAttrs attrs = HLATTRS_INIT;
  int link_id = it.link_id;
  if (ret.type == kObjectTypeDict) {
    fallback = false;
    Dict(highlight) dict = KEYDICT_INIT;
    if (api_dict_to_keydict(&dict, KeyDict_highlight_get_field, ret.data.dict, &err)) {
      attrs = dict2hlattrs(&dict, true, &link_id, &err);
      fallback = GET_BOOL_OR_TRUE(&dict, highlight, fallback);
      tmp = dict.fallback;  // or false
      if (link_id >= 0) {
        fallback = true;
      }
    }
  }

  // Post-callback phase: store result and compute final return value
  NsGetHlPreResult post = rs_ns_get_hl_post(ns_id, hl_id, attrs, link_id,
                                            fallback, tmp, link, nodefault);
  if (post.set_ns_to_zero) {
    *ns_hl = 0;
  }
  return post.result;
}

bool hl_check_ns(void)
{
  // Namespace switching logic is implemented in Rust
  return rs_hl_check_ns();
}

/// prepare for drawing window `wp` or global elements if NULL
///
/// Note: pum should be drawn in the context of the current window!
#ifdef USE_RUST_HIGHLIGHT
extern bool rs_win_check_ns_hl(win_T *wp);

bool win_check_ns_hl(win_T *wp)
{
  return rs_win_check_ns_hl(wp);
}
#else
bool win_check_ns_hl(win_T *wp)
{
  ns_hl_win = wp ? wp->w_ns_hl : -1;
  return hl_check_ns();
}
#endif

/// Get attribute code for a builtin highlight group.
///
/// The final syntax group could be modified by hi-link or 'winhighlight'.
int hl_get_ui_attr(int ns_id, int idx, int final_id, bool optional)
{
  HlAttrs attrs = HLATTRS_INIT;
  bool available = false;

  if (final_id > 0) {
    int syn_attr = syn_ns_id2attr(ns_id, final_id, &optional);
    if (syn_attr > 0) {
      attrs = syn_attr2entry(syn_attr);
      available = true;
    }
  }

  if (HLF_PNI <= idx && idx <= HLF_PST) {
    if (attrs.hl_blend == -1 && p_pb > 0) {
      attrs.hl_blend = (int)p_pb;
    }
    if (pum_drawn()) {
      must_redraw_pum = true;
    }
  }

  if (optional && !available) {
    return 0;
  }
  return get_attr_entry((HlEntry){ .attr = attrs, .kind = kHlUI,
                                   .id1 = idx, .id2 = final_id });
}

/// Apply 'winblend' to highlight attributes.
///
/// @param winbl The 'winblend' value.
/// @param attr  The original attribute code.
///
/// @return      The attribute code with 'winblend' applied.
#ifdef USE_RUST_HIGHLIGHT
extern int rs_hl_apply_winblend(int winbl, int attr);

int hl_apply_winblend(int winbl, int attr)
{
  return rs_hl_apply_winblend(winbl, attr);
}
#else
int hl_apply_winblend(int winbl, int attr)
{
  HlEntry entry = rs_get_attr_entry_by_id(attr);
  // if blend= attribute is not set, 'winblend' value overrides it.
  if (entry.attr.hl_blend == -1 && winbl > 0) {
    entry.attr.hl_blend = winbl;
    attr = get_attr_entry(entry);
  }
  return attr;
}
#endif

void update_window_hl(win_T *wp, bool invalid)
{
  int ns_id = wp->w_ns_hl;

  update_ns_hl(ns_id);
  if (ns_id != wp->w_ns_hl_active || wp->w_ns_hl_attr == NULL) {
    wp->w_ns_hl_active = ns_id;

    wp->w_ns_hl_attr = (int *)rs_ns_hl_attr_get(ns_id);
    if (!wp->w_ns_hl_attr) {
      // No specific highlights, use the defaults.
      wp->w_ns_hl_attr = highlight_attr;
    }
  }

  int *hl_def = wp->w_ns_hl_attr;

  if (!wp->w_hl_needs_update && !invalid) {
    return;
  }
  wp->w_hl_needs_update = false;

  // If a floating window is blending it always have a named
  // wp->w_hl_attr_normal group. HL_ATTR(HLF_NFLOAT) is always named.

  // determine window specific background set in 'winhighlight'
  bool float_win = wp->w_floating && !wp->w_config.external;
  if (float_win && hl_def[HLF_NFLOAT] != 0 && ns_id > 0) {
    wp->w_hl_attr_normal = hl_def[HLF_NFLOAT];
  } else if (hl_def[HLF_NONE] > 0) {
    wp->w_hl_attr_normal = hl_def[HLF_NONE];
  } else if (float_win) {
    wp->w_hl_attr_normal = HL_ATTR(HLF_NFLOAT) > 0
                           ? HL_ATTR(HLF_NFLOAT) : highlight_attr[HLF_NFLOAT];
  } else {
    wp->w_hl_attr_normal = 0;
  }

  if (wp->w_floating) {
    wp->w_hl_attr_normal = hl_apply_winblend((int)wp->w_p_winbl, wp->w_hl_attr_normal);
  }

  wp->w_config.shadow = false;
  if (wp->w_floating && wp->w_config.border) {
    for (int i = 0; i < 8; i++) {
      int attr = hl_def[HLF_BORDER];
      if (wp->w_config.border_hl_ids[i]) {
        attr = hl_get_ui_attr(ns_id, HLF_BORDER,
                              wp->w_config.border_hl_ids[i], false);
      }
      attr = hl_apply_winblend((int)wp->w_p_winbl, attr);
      if (syn_attr2entry(attr).hl_blend > 0) {
        wp->w_config.shadow = true;
      }
      wp->w_config.border_attr[i] = attr;
    }
  }

  // shadow might cause blending
  check_blending(wp);

  // TODO(bfredl): this a bit ad-hoc. move it from highlight ns logic to 'winhl'
  // implementation?
  if (hl_def[HLF_INACTIVE] == 0) {
    wp->w_hl_attr_normalnc = hl_combine_attr(HL_ATTR(HLF_INACTIVE),
                                             wp->w_hl_attr_normal);
  } else {
    wp->w_hl_attr_normalnc = hl_def[HLF_INACTIVE];
  }

  if (wp->w_floating) {
    wp->w_hl_attr_normalnc = hl_apply_winblend((int)wp->w_p_winbl, wp->w_hl_attr_normalnc);
  }
}

static void update_ns_hl(int ns_id)
{
  if (ns_id <= 0) {
    return;
  }
  DecorProvider *p = get_decor_provider(ns_id, true);
  if (p->hl_cached) {
    return;
  }

  // Get or create the attribute array in Rust storage
  int *hl_attrs = rs_ns_hl_attr_get_or_create(ns_id);

  for (int hlf = 1; hlf < HLF_COUNT; hlf++) {
    int id = syn_check_group(hlf_names[hlf], strlen(hlf_names[hlf]));
    bool optional = (hlf == HLF_INACTIVE || hlf == HLF_NFLOAT);
    hl_attrs[hlf] = hl_get_ui_attr(ns_id, hlf, id, optional);
  }

  // NOOOO! You cannot just pretend that "Normal" is just like any other
  // syntax group! It needs at least 10 layers of special casing! Noooooo!
  //
  // haha, tema engine go brrr
  int normality = syn_check_group(S_LEN("Normal"));
  hl_attrs[HLF_NONE] = hl_get_ui_attr(ns_id, -1, normality, true);

  // hl_get_ui_attr might have invalidated the decor provider
  p = get_decor_provider(ns_id, true);
  p->hl_cached = true;
}

int win_bg_attr(win_T *wp)
{
  if (ns_hl_fast < 0) {
    int local = (wp == curwin) ? wp->w_hl_attr_normal : wp->w_hl_attr_normalnc;
    if (local) {
      return local;
    }
  }

  if (wp == curwin || hl_attr_active[HLF_INACTIVE] == 0) {
    return hl_attr_active[HLF_NONE];
  } else {
    return hl_attr_active[HLF_INACTIVE];
  }
}

/// Gets HL_UNDERLINE highlight.
#ifdef USE_RUST_HIGHLIGHT
extern int rs_hl_get_underline(void);

int hl_get_underline(void)
{
  return rs_hl_get_underline();
}
#else
int hl_get_underline(void)
{
  return get_attr_entry((HlEntry){
    .attr = (HlAttrs){
      .cterm_ae_attr = (int16_t)HL_UNDERLINE,
      .cterm_fg_color = 0,
      .cterm_bg_color = 0,
      .rgb_ae_attr = (int16_t)HL_UNDERLINE,
      .rgb_fg_color = -1,
      .rgb_bg_color = -1,
      .rgb_sp_color = -1,
      .hl_blend = -1,
      .url = -1,
    },
    .kind = kHlUI,
    .id1 = 0,
    .id2 = 0,
  });
}
#endif

/// Augment an existing attribute with a URL.
///
/// @param attr Existing attribute to combine with
/// @param url The URL to associate with the highlight attribute
/// @return Combined attribute
int hl_add_url(int attr, const char *url)
{
  HlAttrs attrs = HLATTRS_INIT;

  // Rust handles URL storage
  uint32_t k = rs_hl_add_url_index(url);
  attrs.url = (int32_t)k;

  int new = get_attr_entry((HlEntry){
    .attr = attrs,
    .kind = kHlUI,
    .id1 = 0,
    .id2 = 0,
  });

  return hl_combine_attr(attr, new);
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
#ifdef USE_RUST_HIGHLIGHT
extern int rs_hl_get_term_attr(HlAttrs *aep);

int hl_get_term_attr(HlAttrs *aep)
{
  return rs_hl_get_term_attr(aep);
}
#else
int hl_get_term_attr(HlAttrs *aep)
{
  return get_attr_entry((HlEntry){ .attr = *aep, .kind = kHlTerminal,
                                   .id1 = 0, .id2 = 0 });
}
#endif

/// Clear all highlight tables.
void clear_hl_tables(bool reinit)
{
  // Rust handles all attribute entry, cache, URL, and namespace storage
  rs_clear_hl_tables(reinit);

  if (reinit) {
    memset(highlight_attr_last, -1, sizeof(highlight_attr_last));
    highlight_attr_set_all();
    highlight_changed();
    screen_invalidate_highlights();
  }
  // Note: ns_hls destruction is now handled by Rust in rs_clear_hl_tables
}

void hl_invalidate_blends(void)
{
  // Rust handles blend cache management
  rs_hl_invalidate_blends();
  highlight_changed();
  update_window_hl(curwin, true);
}

// Combine special attributes (e.g., for spelling) with other attributes
// (e.g., for syntax highlighting).
// "prim_attr" overrules "char_attr".
// This creates a new group when required.
// Since we expect there to be a lot of spelling mistakes we cache the result.
// Return the resulting attributes.
int hl_combine_attr(int char_attr, int prim_attr)
{
  if (char_attr == 0) {
    return prim_attr;
  } else if (prim_attr == 0) {
    return char_attr;
  }

  // Check Rust cache first
  int combine_tag = (char_attr << 16) + prim_attr;
  int id = rs_combine_cache_get(combine_tag);
  if (id > 0) {
    return id;
  }

  // Compute combined attributes in Rust
  HlAttrs char_aep = syn_attr2entry(char_attr);
  HlAttrs prim_aep = syn_attr2entry(prim_attr);
  HlCombineInput input = {
    .char_aep = char_aep,
    .prim_aep = prim_aep,
  };
  HlAttrs new_en = rs_hl_combine_attrs_compute(input);

  id = get_attr_entry((HlEntry){ .attr = new_en, .kind = kHlCombine,
                                 .id1 = char_attr, .id2 = prim_attr });
  if (id > 0) {
    rs_combine_cache_put(combine_tag, id);
  }

  return id;
}

/// Get the used rgb colors for an attr group.
///
/// If colors are unset, use builtin default colors. Never returns -1
/// Cterm colors are unchanged.
static HlAttrs get_colors_force(HlAttrs attrs)
{
  return rs_get_colors_force(attrs);
}

/// Blend overlay attributes (for popupmenu) with other attributes
///
/// This creates a new group when required.
/// This is called per-cell, so cache the result.
///
/// @return the resulting attributes.
int hl_blend_attrs(int back_attr, int front_attr, bool *through)
{
  // Cannot blend uninitialized cells, use front_attr for uninitialized background cells.
  if (front_attr < 0 || back_attr < 0) {
    return front_attr;
  }

  HlAttrs fattrs_raw = syn_attr2entry(front_attr);
  HlAttrs fattrs = get_colors_force(fattrs_raw);
  int ratio = fattrs.hl_blend;
  if (ratio <= 0) {
    *through = false;
    return front_attr;
  }

  // Check Rust cache first
  int combine_tag = (back_attr << 16) + front_attr;
  int id = rs_blend_cache_get(combine_tag, *through);
  if (id > 0) {
    return id;
  }

  // Compute blended attributes in Rust
  HlAttrs battrs_raw = syn_attr2entry(back_attr);
  HlAttrs battrs = get_colors_force(battrs_raw);
  HlBlendInput input = {
    .battrs_raw = battrs_raw,
    .battrs = battrs,
    .fattrs_raw = fattrs_raw,
    .fattrs = fattrs,
    .ratio = ratio,
    .through = *through,
  };
  HlAttrs cattrs = rs_hl_blend_attrs_compute(input);

  HlKind kind = *through ? kHlBlendThrough : kHlBlend;
  id = get_attr_entry((HlEntry){ .attr = cattrs, .kind = kind,
                                 .id1 = back_attr, .id2 = front_attr });
  if (id > 0) {
    rs_blend_cache_put(combine_tag, id, *through);
  }
  return id;
}

/// Get highlight attributes for a attribute code
HlAttrs syn_attr2entry(int attr)
{
  // Rust handles bounds checking and returns HLATTRS_INIT equivalent for invalid IDs
  return rs_syn_attr2entry(attr);
}

/// Gets highlight description for id `attr_id` as a map.
Dict hl_get_attr_by_id(Integer attr_id, Boolean rgb, Arena *arena, Error *err)
{
  Dict dic = ARRAY_DICT_INIT;

  if (attr_id == 0) {
    return dic;
  }

  if (attr_id <= 0 || attr_id >= rs_attr_entry_count()) {
    api_set_error(err, kErrorTypeException,
                  "Invalid attribute id: %" PRId64, attr_id);
    return dic;
  }
  Dict retval = arena_dict(arena, HLATTRS_DICT_SIZE);
  hlattrs2dict(&retval, NULL, syn_attr2entry((int)attr_id), rgb, false);
  return retval;
}

/// Converts an HlAttrs into Dict
///
/// @param[in/out] hl Dict with pre-allocated space for HLATTRS_DICT_SIZE elements
/// @param[in] aep data to convert
/// @param use_rgb use 'gui*' settings if true, else resorts to 'cterm*'
/// @param short_keys change (foreground, background, special) to (fg, bg, sp) for 'gui*' settings
///                          (foreground, background) to (ctermfg, ctermbg) for 'cterm*' settings
void hlattrs2dict(Dict *hl, Dict *hl_attrs, HlAttrs ae, bool use_rgb, bool short_keys)
{
  hl_attrs = hl_attrs ? hl_attrs : hl;
  assert(hl->capacity >= HLATTRS_DICT_SIZE);  // at most 16 items
  assert(hl_attrs->capacity >= HLATTRS_DICT_SIZE);  // at most 16 items
  int mask = use_rgb ? ae.rgb_ae_attr : ae.cterm_ae_attr;

  if (mask & HL_INVERSE) {
    PUT_C(*hl_attrs, "reverse", BOOLEAN_OBJ(true));
  }

  if (mask & HL_BOLD) {
    PUT_C(*hl_attrs, "bold", BOOLEAN_OBJ(true));
  }

  if (mask & HL_ITALIC) {
    PUT_C(*hl_attrs, "italic", BOOLEAN_OBJ(true));
  }

  switch (mask & HL_UNDERLINE_MASK) {
  case HL_UNDERLINE:
    PUT_C(*hl_attrs, "underline", BOOLEAN_OBJ(true));
    break;

  case HL_UNDERCURL:
    PUT_C(*hl_attrs, "undercurl", BOOLEAN_OBJ(true));
    break;

  case HL_UNDERDOUBLE:
    PUT_C(*hl_attrs, "underdouble", BOOLEAN_OBJ(true));
    break;

  case HL_UNDERDOTTED:
    PUT_C(*hl_attrs, "underdotted", BOOLEAN_OBJ(true));
    break;

  case HL_UNDERDASHED:
    PUT_C(*hl_attrs, "underdashed", BOOLEAN_OBJ(true));
    break;
  }

  if (mask & HL_STANDOUT) {
    PUT_C(*hl_attrs, "standout", BOOLEAN_OBJ(true));
  }

  if (mask & HL_STRIKETHROUGH) {
    PUT_C(*hl_attrs, "strikethrough", BOOLEAN_OBJ(true));
  }

  if (mask & HL_ALTFONT) {
    PUT_C(*hl_attrs, "altfont", BOOLEAN_OBJ(true));
  }

  if (mask & HL_NOCOMBINE) {
    PUT_C(*hl_attrs, "nocombine", BOOLEAN_OBJ(true));
  }

  if (use_rgb) {
    if (ae.rgb_fg_color != -1) {
      PUT_C(*hl, short_keys ? "fg" : "foreground", INTEGER_OBJ(ae.rgb_fg_color));
    }

    if (ae.rgb_bg_color != -1) {
      PUT_C(*hl, short_keys ? "bg" : "background", INTEGER_OBJ(ae.rgb_bg_color));
    }

    if (ae.rgb_sp_color != -1) {
      PUT_C(*hl, short_keys ? "sp" : "special", INTEGER_OBJ(ae.rgb_sp_color));
    }

    if (!short_keys) {
      if (mask & HL_FG_INDEXED) {
        PUT_C(*hl, "fg_indexed", BOOLEAN_OBJ(true));
      }

      if (mask & HL_BG_INDEXED) {
        PUT_C(*hl, "bg_indexed", BOOLEAN_OBJ(true));
      }
    }
  } else {
    if (ae.cterm_fg_color != 0) {
      PUT_C(*hl, short_keys ? "ctermfg" : "foreground", INTEGER_OBJ(ae.cterm_fg_color - 1));
    }

    if (ae.cterm_bg_color != 0) {
      PUT_C(*hl, short_keys ? "ctermbg" : "background", INTEGER_OBJ(ae.cterm_bg_color - 1));
    }
  }

  if (ae.hl_blend > -1 && (use_rgb || !short_keys)) {
    PUT_C(*hl, "blend", INTEGER_OBJ(ae.hl_blend));
  }
}

HlAttrs dict2hlattrs(Dict(highlight) *dict, bool use_rgb, int *link_id, Error *err)
{
#define HAS_KEY_X(d, key) HAS_KEY(d, highlight, key)
  HlAttrs hlattrs = HLATTRS_INIT;
  int32_t fg = -1;
  int32_t bg = -1;
  int32_t ctermfg = -1;
  int32_t ctermbg = -1;
  int32_t sp = -1;
  int blend = -1;
  int16_t mask = 0;
  int16_t cterm_mask = 0;
  bool cterm_mask_provided = false;

#define CHECK_FLAG(d, m, name, extra, flag) \
  if (d->name##extra) { \
    if (flag & HL_UNDERLINE_MASK) { \
      m &= ~HL_UNDERLINE_MASK; \
    } \
    m |= flag; \
  }

  CHECK_FLAG(dict, mask, reverse, , HL_INVERSE);
  CHECK_FLAG(dict, mask, bold, , HL_BOLD);
  CHECK_FLAG(dict, mask, italic, , HL_ITALIC);
  CHECK_FLAG(dict, mask, underline, , HL_UNDERLINE);
  CHECK_FLAG(dict, mask, undercurl, , HL_UNDERCURL);
  CHECK_FLAG(dict, mask, underdouble, , HL_UNDERDOUBLE);
  CHECK_FLAG(dict, mask, underdotted, , HL_UNDERDOTTED);
  CHECK_FLAG(dict, mask, underdashed, , HL_UNDERDASHED);
  CHECK_FLAG(dict, mask, standout, , HL_STANDOUT);
  CHECK_FLAG(dict, mask, strikethrough, , HL_STRIKETHROUGH);
  CHECK_FLAG(dict, mask, altfont, , HL_ALTFONT);
  if (use_rgb) {
    CHECK_FLAG(dict, mask, fg_indexed, , HL_FG_INDEXED);
    CHECK_FLAG(dict, mask, bg_indexed, , HL_BG_INDEXED);
  }
  CHECK_FLAG(dict, mask, nocombine, , HL_NOCOMBINE);
  CHECK_FLAG(dict, mask, default, _, HL_DEFAULT);

  if (HAS_KEY_X(dict, fg)) {
    fg = object_to_color(dict->fg, "fg", use_rgb, err);
  } else if (HAS_KEY_X(dict, foreground)) {
    fg = object_to_color(dict->foreground, "foreground", use_rgb, err);
  }
  if (ERROR_SET(err)) {
    return hlattrs;
  }

  if (HAS_KEY_X(dict, bg)) {
    bg = object_to_color(dict->bg, "bg", use_rgb, err);
  } else if (HAS_KEY_X(dict, background)) {
    bg = object_to_color(dict->background, "background", use_rgb, err);
  }
  if (ERROR_SET(err)) {
    return hlattrs;
  }

  if (HAS_KEY_X(dict, sp)) {
    sp = object_to_color(dict->sp, "sp", true, err);
  } else if (HAS_KEY_X(dict, special)) {
    sp = object_to_color(dict->special, "special", true, err);
  }
  if (ERROR_SET(err)) {
    return hlattrs;
  }

  if (HAS_KEY_X(dict, blend)) {
    Integer blend0 = dict->blend;
    VALIDATE_RANGE((blend0 >= 0 && blend0 <= 100), "blend", {
      return hlattrs;
    });
    blend = (int)blend0;
  }

  if (HAS_KEY_X(dict, link) || HAS_KEY_X(dict, global_link)) {
    if (!link_id) {
      api_set_error(err, kErrorTypeValidation, "Invalid Key: '%s'",
                    HAS_KEY_X(dict, global_link) ? "global_link" : "link");
      return hlattrs;
    }
    if (HAS_KEY_X(dict, global_link)) {
      *link_id = (int)dict->global_link;
      mask |= HL_GLOBAL;
    } else {
      *link_id = (int)dict->link;
    }

    if (ERROR_SET(err)) {
      return hlattrs;
    }
  }

  // Handle cterm attrs
  if (dict->cterm.type == kObjectTypeDict) {
    Dict(highlight_cterm) cterm[1] = KEYDICT_INIT;
    if (!api_dict_to_keydict(cterm, KeyDict_highlight_cterm_get_field,
                             dict->cterm.data.dict, err)) {
      return hlattrs;
    }

    cterm_mask_provided = true;
    CHECK_FLAG(cterm, cterm_mask, reverse, , HL_INVERSE);
    CHECK_FLAG(cterm, cterm_mask, bold, , HL_BOLD);
    CHECK_FLAG(cterm, cterm_mask, italic, , HL_ITALIC);
    CHECK_FLAG(cterm, cterm_mask, underline, , HL_UNDERLINE);
    CHECK_FLAG(cterm, cterm_mask, undercurl, , HL_UNDERCURL);
    CHECK_FLAG(cterm, cterm_mask, underdouble, , HL_UNDERDOUBLE);
    CHECK_FLAG(cterm, cterm_mask, underdotted, , HL_UNDERDOTTED);
    CHECK_FLAG(cterm, cterm_mask, underdashed, , HL_UNDERDASHED);
    CHECK_FLAG(cterm, cterm_mask, standout, , HL_STANDOUT);
    CHECK_FLAG(cterm, cterm_mask, strikethrough, , HL_STRIKETHROUGH);
    CHECK_FLAG(cterm, cterm_mask, altfont, , HL_ALTFONT);
    CHECK_FLAG(cterm, cterm_mask, nocombine, , HL_NOCOMBINE);
  } else if (dict->cterm.type == kObjectTypeArray && dict->cterm.data.array.size == 0) {
    // empty list from Lua API should clear all cterm attributes
    // TODO(clason): handle via gen_api_dispatch
    cterm_mask_provided = true;
  } else if (HAS_KEY_X(dict, cterm)) {
    VALIDATE_EXP(false, "cterm", "Dict", api_typename(dict->cterm.type), {
      return hlattrs;
    });
  }
#undef CHECK_FLAG

  if (HAS_KEY_X(dict, ctermfg)) {
    ctermfg = object_to_color(dict->ctermfg, "ctermfg", false, err);
    if (ERROR_SET(err)) {
      return hlattrs;
    }
  }

  if (HAS_KEY_X(dict, ctermbg)) {
    ctermbg = object_to_color(dict->ctermbg, "ctermbg", false, err);
    if (ERROR_SET(err)) {
      return hlattrs;
    }
  }

  if (use_rgb) {
    // apply gui mask as default for cterm mask
    if (!cterm_mask_provided) {
      cterm_mask = mask;
    }
    hlattrs.rgb_ae_attr = mask;
    hlattrs.rgb_bg_color = bg;
    hlattrs.rgb_fg_color = fg;
    hlattrs.rgb_sp_color = sp;
    hlattrs.hl_blend = blend;
    hlattrs.cterm_bg_color = ctermbg == -1 ? 0 : (int16_t)(ctermbg + 1);
    hlattrs.cterm_fg_color = ctermfg == -1 ? 0 : (int16_t)(ctermfg + 1);
    hlattrs.cterm_ae_attr = cterm_mask;
  } else {
    hlattrs.cterm_bg_color = bg == -1 ? 0 : (int16_t)(bg + 1);
    hlattrs.cterm_fg_color = fg == -1 ? 0 : (int16_t)(fg + 1);
    hlattrs.cterm_ae_attr = mask;
  }

  return hlattrs;
#undef HAS_KEY_X
}

int object_to_color(Object val, char *key, bool rgb, Error *err)
{
  if (val.type == kObjectTypeInteger) {
    return (int)val.data.integer;
  } else if (val.type == kObjectTypeString) {
    String str = val.data.string;
    // TODO(bfredl): be more fancy with "bg", "fg" etc
    if (!str.size || STRICMP(str.data, "NONE") == 0) {
      return -1;
    }
    int color;
    if (rgb) {
      int dummy;
      color = name_to_color(str.data, &dummy);
    } else {
      color = name_to_ctermcolor(str.data);
    }
    VALIDATE_S((color >= 0), "highlight color", str.data, {
      return color;
    });
    return color;
  } else {
    VALIDATE_EXP(false, key, "String or Integer", NULL, {
      return 0;
    });
  }
}

Array hl_inspect(int attr, Arena *arena)
{
  if (!hlstate_active) {
    return (Array)ARRAY_DICT_INIT;
  }
  Array ret = arena_array(arena, hl_inspect_size(attr));
  hl_inspect_impl(&ret, attr, arena);
  return ret;
}

static size_t hl_inspect_size(int attr)
{
  if (attr <= 0 || attr >= rs_attr_entry_count()) {
    return 0;
  }

  HlEntry e = rs_get_attr_entry_by_id(attr);
  if (e.kind == kHlCombine || e.kind == kHlBlend || e.kind == kHlBlendThrough) {
    return hl_inspect_size(e.id1) + hl_inspect_size(e.id2);
  }
  return 1;
}

static void hl_inspect_impl(Array *arr, int attr, Arena *arena)
{
  Dict item = ARRAY_DICT_INIT;
  if (attr <= 0 || attr >= rs_attr_entry_count()) {
    return;
  }

  HlEntry e = rs_get_attr_entry_by_id(attr);
  switch (e.kind) {
  case kHlSyntax:
    item = arena_dict(arena, 3);
    PUT_C(item, "kind", CSTR_AS_OBJ("syntax"));
    PUT_C(item, "hi_name", CSTR_AS_OBJ(syn_id2name(e.id1)));
    break;

  case kHlUI:
    item = arena_dict(arena, 4);
    PUT_C(item, "kind", CSTR_AS_OBJ("ui"));
    const char *ui_name = (e.id1 == -1) ? "Normal" : hlf_names[e.id1];
    PUT_C(item, "ui_name", CSTR_AS_OBJ(ui_name));
    PUT_C(item, "hi_name", CSTR_AS_OBJ(syn_id2name(e.id2)));
    break;

  case kHlTerminal:
    item = arena_dict(arena, 2);
    PUT_C(item, "kind", CSTR_AS_OBJ("term"));
    break;

  case kHlCombine:
  case kHlBlend:
  case kHlBlendThrough:
    // attribute combination is associative, so flatten to an array
    hl_inspect_impl(arr, e.id1, arena);
    hl_inspect_impl(arr, e.id2, arena);
    return;

  case kHlUnknown:
  case kHlInvalid:
    return;
  }
  PUT_C(item, "id", INTEGER_OBJ(attr));
  ADD_C(*arr, DICT_OBJ(item));
}
