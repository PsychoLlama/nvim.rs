use crate::src::nvim::api::private::helpers::{arena_array, arena_dict, cstr_as_string};
use crate::src::nvim::api::ui::{remote_ui_hl_attr_define, remote_ui_hl_group_set};
use crate::src::nvim::drawscreen::screen_invalidate_highlights;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight_group::{highlight_attr_set_all, highlight_changed, syn_id2name};
use crate::src::nvim::main::{highlight_attr, highlight_attr_last, hlf_names};
use crate::src::nvim::map::{
    map_put_ref_uint64_t_int, mh_clear, mh_get_uint64_t, mh_put_HlEntry, mh_put_cstr_t,
};
use crate::src::nvim::memory::{ARENA_EMPTY, arena_finish, arena_mem_free, xfree, xstrdup};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::{gettext, memset};
use crate::src::nvim::types::{
    Arena, Array, Dict, HlAttrs, HlEntry, HlKind, Integer, KeyValuePair, MHPutStatus,
    Map_uint64_t_int, MapHash, Object, RemoteUI, RgbValue, Set_HlEntry, Set_cstr_t, Set_uint64_t,
    cstr_t, int16_t, int32_t, kObjectTypeDict, kObjectTypeInteger, kObjectTypeString,
    key_value_pair, object, object_data as C2Rust_Unnamed, size_t, uint32_t, uint64_t,
};
use crate::src::nvim::ui::ui_call_hl_attr_define;

// Split out for size; the rest of the tree calls all of it as `highlight::*`.
pub mod blend;
pub mod cache;
pub mod dict;
pub mod namespace;

pub use blend::{hl_blend_attrs, hl_invalidate_blends};
pub use dict::{HLATTRS_DICT_SIZE, dict2hlattrs, hl_get_attr_by_id, hlattrs2dict};
pub use namespace::{
    hl_check_ns, hl_get_ui_attr, hl_ns_get_attrs, ns_get_hl, ns_hl_def, update_ns_hl,
    update_window_hl, win_bg_attr, win_check_ns_hl, win_hl_attr,
};

/// The attribute bits an `HlAttrs`' `rgb_ae_attr`/`cterm_ae_attr` carry.
pub type HlAttrFlags = ::core::ffi::c_int;
pub const HL_GLOBAL: HlAttrFlags = 16384;
pub const HL_DEFAULT: HlAttrFlags = 8192;
pub const HL_FG_INDEXED: HlAttrFlags = 4096;
pub const HL_BG_INDEXED: HlAttrFlags = 2048;
pub const HL_NOCOMBINE: HlAttrFlags = 1024;
pub const HL_OVERLINE: HlAttrFlags = 131072;
pub const HL_CONCEALED: HlAttrFlags = 65536;
pub const HL_BLINK: HlAttrFlags = 32768;
pub const HL_DIM: HlAttrFlags = 512;
pub const HL_ALTFONT: HlAttrFlags = 256;
pub const HL_STRIKETHROUGH: HlAttrFlags = 128;
pub const HL_STANDOUT: HlAttrFlags = 64;
pub const HL_UNDERDASHED: HlAttrFlags = 40;
pub const HL_UNDERDOTTED: HlAttrFlags = 32;
pub const HL_UNDERDOUBLE: HlAttrFlags = 24;
pub const HL_UNDERCURL: HlAttrFlags = 16;
pub const HL_UNDERLINE: HlAttrFlags = 8;
pub const HL_UNDERLINE_MASK: HlAttrFlags = 56;
pub const HL_ITALIC: HlAttrFlags = 4;
pub const HL_BOLD: HlAttrFlags = 2;
pub const HL_INVERSE: HlAttrFlags = 1;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_15 = 76;
pub const HLF_BORDER: C2Rust_Unnamed_15 = 64;
pub const HLF_NFLOAT: C2Rust_Unnamed_15 = 62;
pub const HLF_INACTIVE: C2Rust_Unnamed_15 = 60;
pub const HLF_PST: C2Rust_Unnamed_15 = 50;
pub const HLF_PNI: C2Rust_Unnamed_15 = 41;
pub const HLF_NONE: C2Rust_Unnamed_15 = 0;
pub const kHlInvalid: HlKind = 7;
pub const kHlBlendThrough: HlKind = 6;
pub const kHlBlend: HlKind = 5;
pub const kHlCombine: HlKind = 4;
pub const kHlTerminal: HlKind = 3;
pub const kHlSyntax: HlKind = 2;
pub const kHlUI: HlKind = 1;
pub const kHlUnknown: HlKind = 0;
pub const kMHExisting: MHPutStatus = 0;
pub type NSHlAttr = [::core::ffi::c_int; 76];
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const HLATTRS_INIT: HlAttrs = HlAttrs {
    rgb_ae_attr: 0 as int32_t,
    cterm_ae_attr: 0 as int32_t,
    rgb_fg_color: -1 as RgbValue,
    rgb_bg_color: -1 as RgbValue,
    rgb_sp_color: -1 as RgbValue,
    cterm_fg_color: 0 as int16_t,
    cterm_bg_color: 0 as int16_t,
    hl_blend: -1 as int32_t,
    url: -1 as int32_t,
};
static value_init_int: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const MAP_INIT: Map_uint64_t_int = Map_uint64_t_int {
    set: Set_uint64_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<uint64_t>(),
    },
    values: ::core::ptr::null_mut::<::core::ffi::c_int>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_put_HlEntry(
    mut set: *mut Set_HlEntry,
    mut key: HlEntry,
    mut key_alloc: *mut *mut HlEntry,
) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_HlEntry(set, key, &raw mut status);
    if !key_alloc.is_null() {
        *key_alloc = (*set).keys.offset(k as isize);
    }
    return status as ::core::ffi::c_uint
        != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
}
#[inline]
unsafe extern "C" fn map_get_uint64_t_int(
    mut map: *mut Map_uint64_t_int,
    mut key: uint64_t,
) -> ::core::ffi::c_int {
    let mut k: uint32_t = mh_get_uint64_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_int.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
#[inline]
unsafe extern "C" fn map_put_uint64_t_int(
    mut map: *mut Map_uint64_t_int,
    mut key: uint64_t,
    mut value: ::core::ffi::c_int,
) {
    let mut val: *mut ::core::ffi::c_int = map_put_ref_uint64_t_int(
        map,
        key,
        ::core::ptr::null_mut::<*mut uint64_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
pub const MAX_TYPENR: ::core::ffi::c_int = 65535 as ::core::ffi::c_int;
static hlstate_active: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static attr_entries: GlobalCell<Set_HlEntry> = GlobalCell::new(Set_HlEntry {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<HlEntry>(),
});
static combine_attr_entries: GlobalCell<Map_uint64_t_int> = GlobalCell::new(MAP_INIT);
static urls: GlobalCell<Set_cstr_t> = GlobalCell::new(Set_cstr_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<cstr_t>(),
});
pub unsafe extern "C" fn highlight_init() {
    set_put_HlEntry(
        attr_entries.ptr(),
        HlEntry {
            attr: HlAttrs {
                rgb_ae_attr: 0 as int32_t,
                cterm_ae_attr: 0 as int32_t,
                rgb_fg_color: -1 as RgbValue,
                rgb_bg_color: -1 as RgbValue,
                rgb_sp_color: -1 as RgbValue,
                cterm_fg_color: 0 as int16_t,
                cterm_bg_color: 0 as int16_t,
                hl_blend: -1 as int32_t,
                url: -1 as int32_t,
            },
            kind: kHlInvalid,
            id1: 0 as ::core::ffi::c_int,
            id2: 0 as ::core::ffi::c_int,
            winid: 0,
        },
        ::core::ptr::null_mut::<*mut HlEntry>(),
    );
}
pub unsafe extern "C" fn highlight_use_hlstate() -> bool {
    if hlstate_active.get() {
        return false_0 != 0;
    }
    hlstate_active.set(true_0 != 0);
    clear_hl_tables(true_0 != 0);
    return true_0 != 0;
}
unsafe extern "C" fn get_attr_entry(mut entry: HlEntry) -> ::core::ffi::c_int {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = 0;
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut retried: bool = false_0 != 0;
    if !hlstate_active.get() {
        entry.kind = kHlUnknown;
        entry.id1 = 0 as ::core::ffi::c_int;
        entry.id2 = 0 as ::core::ffi::c_int;
    }
    loop {
        status = kMHExisting;
        k = mh_put_HlEntry(attr_entries.ptr(), entry, &raw mut status);
        if status as ::core::ffi::c_uint == kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return k as ::core::ffi::c_int;
        }
        if (*attr_entries.ptr()).h.size <= MAX_TYPENR as uint32_t {
            break;
        }
        if recursive.get() as ::core::ffi::c_int != 0 || retried as ::core::ffi::c_int != 0 {
            emsg(gettext(
                b"E424: Too many different highlighting attributes in use\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return 0 as ::core::ffi::c_int;
        }
        recursive.set(true_0 != 0);
        clear_hl_tables(true_0 != 0);
        recursive.set(false_0 != 0);
        if entry.kind as ::core::ffi::c_uint
            == kHlCombine as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return 0 as ::core::ffi::c_int;
        }
        retried = true_0 != 0;
    }
    let mut id: ::core::ffi::c_int = k as ::core::ffi::c_int;
    let mut arena: Arena = ARENA_EMPTY;
    let mut inspect: Array = hl_inspect(id, &raw mut arena);
    ui_call_hl_attr_define(id as Integer, entry.attr, entry.attr, inspect);
    arena_mem_free(arena_finish(&raw mut arena));
    return id;
}
pub unsafe extern "C" fn ui_send_all_hls(mut ui: *mut RemoteUI) {
    let mut i: size_t = 1 as size_t;
    while i < (*attr_entries.ptr()).h.size as size_t {
        let mut arena: Arena = ARENA_EMPTY;
        let mut inspect: Array = hl_inspect(i as ::core::ffi::c_int, &raw mut arena);
        let mut attr: HlAttrs = (*(*attr_entries.ptr()).keys.offset(i as isize)).attr;
        remote_ui_hl_attr_define(ui, i as Integer, attr, attr, inspect);
        arena_mem_free(arena_finish(&raw mut arena));
        i = i.wrapping_add(1);
    }
    let mut hlf: size_t = 0 as size_t;
    while hlf < HLF_COUNT as ::core::ffi::c_int as size_t {
        remote_ui_hl_group_set(
            ui,
            cstr_as_string(
                *(hlf_names.ptr() as *mut *const ::core::ffi::c_char).offset(hlf as isize),
            ),
            (*highlight_attr.ptr())[hlf as usize] as Integer,
        );
        hlf = hlf.wrapping_add(1);
    }
}
pub unsafe extern "C" fn hl_get_syn_attr(
    mut ns_id: ::core::ffi::c_int,
    mut idx: ::core::ffi::c_int,
    mut at_en: HlAttrs,
) -> ::core::ffi::c_int {
    if at_en.cterm_fg_color as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        || at_en.cterm_bg_color as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        || at_en.rgb_fg_color != -1 as RgbValue
        || at_en.rgb_bg_color != -1 as RgbValue
        || at_en.rgb_sp_color != -1 as RgbValue
        || at_en.cterm_ae_attr != 0 as int32_t
        || at_en.rgb_ae_attr != 0 as int32_t
        || ns_id != 0 as ::core::ffi::c_int
    {
        return get_attr_entry(HlEntry {
            attr: at_en,
            kind: kHlSyntax,
            id1: idx,
            id2: ns_id,
            winid: 0,
        });
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn hl_apply_winblend(
    mut winbl: ::core::ffi::c_int,
    mut attr: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut entry: HlEntry = *(*attr_entries.ptr()).keys.offset(attr as isize);
    if entry.attr.hl_blend == -1 as int32_t && winbl > 0 as ::core::ffi::c_int {
        entry.attr.hl_blend = winbl as int32_t;
        attr = get_attr_entry(entry);
    }
    return attr;
}
pub unsafe extern "C" fn hl_get_underline() -> ::core::ffi::c_int {
    let mut attrs: HlAttrs = HLATTRS_INIT;
    attrs.cterm_ae_attr = HL_UNDERLINE as int16_t as int32_t;
    attrs.rgb_ae_attr = HL_UNDERLINE as int16_t as int32_t;
    return get_attr_entry(HlEntry {
        attr: attrs,
        kind: kHlUI,
        id1: 0 as ::core::ffi::c_int,
        id2: 0 as ::core::ffi::c_int,
        winid: 0,
    });
}
pub unsafe extern "C" fn hl_add_url(
    mut attr: ::core::ffi::c_int,
    mut url: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut attrs: HlAttrs = HLATTRS_INIT;
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_cstr_t(urls.ptr(), url as cstr_t, &raw mut status);
    if status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint {
        *(*urls.ptr()).keys.offset(k as isize) = xstrdup(url) as cstr_t;
    }
    attrs.url = k as int32_t;
    let mut new: ::core::ffi::c_int = get_attr_entry(HlEntry {
        attr: attrs,
        kind: kHlUI,
        id1: 0 as ::core::ffi::c_int,
        id2: 0 as ::core::ffi::c_int,
        winid: 0,
    });
    return hl_combine_attr(attr, new);
}
pub unsafe extern "C" fn hl_get_url(mut index: uint32_t) -> *const ::core::ffi::c_char {
    assert!(!(*urls.ptr()).keys.is_null(), "urls.keys");
    return *(*urls.ptr()).keys.offset(index as isize) as *const ::core::ffi::c_char;
}
pub unsafe extern "C" fn hl_get_term_attr(mut aep: *mut HlAttrs) -> ::core::ffi::c_int {
    return get_attr_entry(HlEntry {
        attr: *aep,
        kind: kHlTerminal,
        id1: 0 as ::core::ffi::c_int,
        id2: 0 as ::core::ffi::c_int,
        winid: 0,
    });
}
pub unsafe extern "C" fn clear_hl_tables(mut reinit: bool) {
    let mut url: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*urls.ptr()).h.n_keys {
        url = *(*urls.ptr()).keys.offset(__i as isize) as *const ::core::ffi::c_char;
        xfree(url as *mut ::core::ffi::c_void);
        __i = __i.wrapping_add(1);
    }
    if reinit {
        mh_clear(&raw mut (*attr_entries.ptr()).h);
        highlight_init();
        mh_clear(&raw mut (*combine_attr_entries.ptr()).set.h);
        blend::clear_caches();
        mh_clear(&raw mut (*urls.ptr()).h);
        memset(
            highlight_attr_last.ptr() as *mut ::core::ffi::c_int as *mut ::core::ffi::c_void,
            -1 as ::core::ffi::c_int,
            ::core::mem::size_of::<[::core::ffi::c_int; 76]>(),
        );
        highlight_attr_set_all();
        highlight_changed();
        screen_invalidate_highlights();
    } else {
        xfree((*attr_entries.ptr()).keys as *mut ::core::ffi::c_void);
        xfree((*attr_entries.ptr()).h.hash as *mut ::core::ffi::c_void);
        attr_entries.set(Set_HlEntry {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<HlEntry>(),
        });
        xfree((*combine_attr_entries.ptr()).set.keys as *mut ::core::ffi::c_void);
        xfree((*combine_attr_entries.ptr()).set.h.hash as *mut ::core::ffi::c_void);
        (*combine_attr_entries.ptr()).set = Set_uint64_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<uint64_t>(),
        };
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*combine_attr_entries.ptr()).values as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        blend::clear_caches();
        namespace::clear_ns_defs();
        xfree((*urls.ptr()).keys as *mut ::core::ffi::c_void);
        xfree((*urls.ptr()).h.hash as *mut ::core::ffi::c_void);
        urls.set(Set_cstr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<cstr_t>(),
        });
    };
}
unsafe extern "C" fn hl_combine_ae(mut char_ae: int32_t, mut prim_ae: int32_t) -> int32_t {
    let mut char_ul: int32_t = char_ae & HL_UNDERLINE_MASK as int32_t;
    let mut prim_ul: int32_t = prim_ae & HL_UNDERLINE_MASK as int32_t;
    let mut new_ul: int32_t = if prim_ul != 0 { prim_ul } else { char_ul };
    return char_ae & !(HL_UNDERLINE_MASK as int32_t)
        | prim_ae & !(HL_UNDERLINE_MASK as int32_t)
        | new_ul;
}
pub unsafe extern "C" fn hl_combine_attr(
    mut char_attr: ::core::ffi::c_int,
    mut prim_attr: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if char_attr == 0 as ::core::ffi::c_int {
        return prim_attr;
    } else if prim_attr == 0 as ::core::ffi::c_int {
        return char_attr;
    }
    let mut combine_tag: uint64_t = (char_attr as uint32_t as uint64_t) << 32 as ::core::ffi::c_int
        | prim_attr as uint32_t as uint64_t;
    let mut id: ::core::ffi::c_int = map_get_uint64_t_int(combine_attr_entries.ptr(), combine_tag);
    if id > 0 as ::core::ffi::c_int {
        return id;
    }
    let mut char_aep: HlAttrs = syn_attr2entry(char_attr);
    let mut prim_aep: HlAttrs = syn_attr2entry(prim_attr);
    let mut new_en: HlAttrs = char_aep;
    if prim_aep.cterm_ae_attr & HL_NOCOMBINE as int32_t != 0 {
        new_en.cterm_ae_attr = prim_aep.cterm_ae_attr;
    } else {
        new_en.cterm_ae_attr = hl_combine_ae(new_en.cterm_ae_attr, prim_aep.cterm_ae_attr);
    }
    if prim_aep.rgb_ae_attr & HL_NOCOMBINE as int32_t != 0 {
        new_en.rgb_ae_attr = prim_aep.rgb_ae_attr;
    } else {
        new_en.rgb_ae_attr = hl_combine_ae(new_en.rgb_ae_attr, prim_aep.rgb_ae_attr);
    }
    if prim_aep.cterm_fg_color as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
        new_en.cterm_fg_color = prim_aep.cterm_fg_color;
        new_en.rgb_ae_attr = (new_en.rgb_ae_attr as ::core::ffi::c_int
            & (!(HL_FG_INDEXED as int32_t) | prim_aep.rgb_ae_attr & HL_FG_INDEXED as int32_t)
                as ::core::ffi::c_int) as int32_t;
    }
    if prim_aep.cterm_bg_color as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
        new_en.cterm_bg_color = prim_aep.cterm_bg_color;
        new_en.rgb_ae_attr = (new_en.rgb_ae_attr as ::core::ffi::c_int
            & (!(HL_BG_INDEXED as int32_t) | prim_aep.rgb_ae_attr & HL_BG_INDEXED as int32_t)
                as ::core::ffi::c_int) as int32_t;
    }
    if prim_aep.rgb_fg_color >= 0 as RgbValue {
        new_en.rgb_fg_color = prim_aep.rgb_fg_color;
        new_en.rgb_ae_attr = (new_en.rgb_ae_attr as ::core::ffi::c_int
            & (!(HL_FG_INDEXED as int32_t) | prim_aep.rgb_ae_attr & HL_FG_INDEXED as int32_t)
                as ::core::ffi::c_int) as int32_t;
    }
    if prim_aep.rgb_bg_color >= 0 as RgbValue {
        new_en.rgb_bg_color = prim_aep.rgb_bg_color;
        new_en.rgb_ae_attr = (new_en.rgb_ae_attr as ::core::ffi::c_int
            & (!(HL_BG_INDEXED as int32_t) | prim_aep.rgb_ae_attr & HL_BG_INDEXED as int32_t)
                as ::core::ffi::c_int) as int32_t;
    }
    if prim_aep.rgb_sp_color >= 0 as RgbValue {
        new_en.rgb_sp_color = prim_aep.rgb_sp_color;
    }
    if prim_aep.hl_blend >= 0 as int32_t {
        new_en.hl_blend = prim_aep.hl_blend;
    }
    if new_en.url == -1 as int32_t && prim_aep.url >= 0 as int32_t {
        new_en.url = prim_aep.url;
    }
    id = get_attr_entry(HlEntry {
        attr: new_en,
        kind: kHlCombine,
        id1: char_attr,
        id2: prim_attr,
        winid: 0,
    });
    if id > 0 as ::core::ffi::c_int {
        map_put_uint64_t_int(combine_attr_entries.ptr(), combine_tag, id);
    }
    return id;
}
/// The number of attribute ids handed out so far, counting the id-0
/// sentinel. Every id below this is a live entry.
pub unsafe fn attr_entry_count() -> ::core::ffi::c_int {
    unsafe { (*attr_entries.ptr()).h.size as ::core::ffi::c_int }
}
pub unsafe extern "C" fn syn_attr2entry(mut attr: ::core::ffi::c_int) -> HlAttrs {
    if attr <= 0 as ::core::ffi::c_int || attr >= (*attr_entries.ptr()).h.size as ::core::ffi::c_int
    {
        return HLATTRS_INIT;
    }
    return (*(*attr_entries.ptr()).keys.offset(attr as isize)).attr;
}
pub unsafe extern "C" fn hl_inspect(mut attr: ::core::ffi::c_int, mut arena: *mut Arena) -> Array {
    if !hlstate_active.get() {
        return Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
    }
    let mut ret: Array = arena_array(arena, hl_inspect_size(attr));
    hl_inspect_impl(&raw mut ret, attr, arena);
    return ret;
}
unsafe extern "C" fn hl_inspect_size(mut attr: ::core::ffi::c_int) -> size_t {
    if attr <= 0 as ::core::ffi::c_int || attr >= (*attr_entries.ptr()).h.size as ::core::ffi::c_int
    {
        return 0 as size_t;
    }
    let mut e: HlEntry = *(*attr_entries.ptr()).keys.offset(attr as isize);
    if e.kind as ::core::ffi::c_uint == kHlCombine as ::core::ffi::c_int as ::core::ffi::c_uint
        || e.kind as ::core::ffi::c_uint == kHlBlend as ::core::ffi::c_int as ::core::ffi::c_uint
        || e.kind as ::core::ffi::c_uint
            == kHlBlendThrough as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return hl_inspect_size(e.id1).wrapping_add(hl_inspect_size(e.id2));
    }
    return 1 as size_t;
}
unsafe extern "C" fn hl_inspect_impl(
    mut arr: *mut Array,
    mut attr: ::core::ffi::c_int,
    mut arena: *mut Arena,
) {
    let mut item: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    if attr <= 0 as ::core::ffi::c_int || attr >= (*attr_entries.ptr()).h.size as ::core::ffi::c_int
    {
        return;
    }
    let mut e: HlEntry = *(*attr_entries.ptr()).keys.offset(attr as isize);
    let mut ui_name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    match e.kind as ::core::ffi::c_uint {
        2 => {
            item = arena_dict(arena, 3 as size_t);
            let c2rust_fresh0 = item.size;
            item.size = item.size.wrapping_add(1);
            *item.items.offset(c2rust_fresh0 as isize) = key_value_pair {
                key: cstr_as_string(b"kind\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(b"syntax\0".as_ptr() as *const ::core::ffi::c_char),
                    },
                },
            };
            let c2rust_fresh1 = item.size;
            item.size = item.size.wrapping_add(1);
            *item.items.offset(c2rust_fresh1 as isize) = key_value_pair {
                key: cstr_as_string(b"hi_name\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(syn_id2name(e.id1)),
                    },
                },
            };
        }
        1 => {
            item = arena_dict(arena, 4 as size_t);
            let c2rust_fresh2 = item.size;
            item.size = item.size.wrapping_add(1);
            *item.items.offset(c2rust_fresh2 as isize) = key_value_pair {
                key: cstr_as_string(b"kind\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(b"ui\0".as_ptr() as *const ::core::ffi::c_char),
                    },
                },
            };
            ui_name = if e.id1 == -1 as ::core::ffi::c_int {
                b"Normal\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                *(hlf_names.ptr() as *mut *const ::core::ffi::c_char).offset(e.id1 as isize)
            };
            let c2rust_fresh3 = item.size;
            item.size = item.size.wrapping_add(1);
            *item.items.offset(c2rust_fresh3 as isize) = key_value_pair {
                key: cstr_as_string(b"ui_name\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(ui_name),
                    },
                },
            };
            let c2rust_fresh4 = item.size;
            item.size = item.size.wrapping_add(1);
            *item.items.offset(c2rust_fresh4 as isize) = key_value_pair {
                key: cstr_as_string(b"hi_name\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(syn_id2name(e.id2)),
                    },
                },
            };
        }
        3 => {
            item = arena_dict(arena, 2 as size_t);
            let c2rust_fresh5 = item.size;
            item.size = item.size.wrapping_add(1);
            *item.items.offset(c2rust_fresh5 as isize) = key_value_pair {
                key: cstr_as_string(b"kind\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(b"term\0".as_ptr() as *const ::core::ffi::c_char),
                    },
                },
            };
        }
        4 | 5 | 6 => {
            hl_inspect_impl(arr, e.id1, arena);
            hl_inspect_impl(arr, e.id2, arena);
            return;
        }
        0 | 7 => return,
        _ => {}
    }
    let c2rust_fresh6 = item.size;
    item.size = item.size.wrapping_add(1);
    *item.items.offset(c2rust_fresh6 as isize) = key_value_pair {
        key: cstr_as_string(b"id\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: attr as Integer,
            },
        },
    };
    let c2rust_fresh7 = (*arr).size;
    (*arr).size = (*arr).size.wrapping_add(1);
    *(*arr).items.offset(c2rust_fresh7 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: item },
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
