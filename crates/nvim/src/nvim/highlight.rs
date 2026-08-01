use crate::src::nvim::api::private::dispatch::KeyDict_highlight_get_field;
use crate::src::nvim::api::private::helpers::{
    api_dict_to_keydict, arena_array, arena_dict, cstr_as_string,
};
use crate::src::nvim::api::ui::{remote_ui_hl_attr_define, remote_ui_hl_group_set};
use crate::src::nvim::decoration_provider::get_decor_provider;
use crate::src::nvim::drawscreen::screen_invalidate_highlights;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight_group::{
    highlight_attr_set_all, highlight_changed, set_hl_group, syn_check_group, syn_id2name,
    syn_ns_id2attr,
};
use crate::src::nvim::lua::executor::nlua_call_ref;
use crate::src::nvim::main::{
    curwin, highlight_attr, highlight_attr_last, hl_attr_active, hlf_names, must_redraw_pum,
    need_highlight_changed, normal_bg, normal_fg, normal_sp, ns_hl_active, ns_hl_fast,
    ns_hl_global, ns_hl_win, p_bg, p_pb,
};
use crate::src::nvim::map::{
    map_put_ref_ColorKey_ColorItem, map_put_ref_int_ptr_t, map_put_ref_uint64_t_int, mh_clear,
    mh_get_ColorKey, mh_get_int, mh_get_uint64_t, mh_put_HlEntry, mh_put_cstr_t,
};
use crate::src::nvim::memory::{
    ARENA_EMPTY, arena_finish, arena_mem_free, xfree, xmalloc, xstrdup,
};
use crate::src::nvim::message::emsg;
use crate::src::nvim::option::check_blending;
use crate::src::nvim::os::libc::{gettext, memset, strlen};
use crate::src::nvim::popupmenu::pum_drawn;
use crate::src::nvim::types::api::kErrorTypeNone;
use crate::src::nvim::types::{
    Arena, Array, ColorItem, ColorKey, DecorProvider, Dict, Error, HlAttrs, HlEntry, HlKind,
    Integer, KeyDict_highlight, KeySetLink, KeyValuePair, LuaRetMode, MHPutStatus,
    Map_ColorKey_ColorItem, Map_int_ptr_t, Map_uint64_t_int, MapHash, NS, Object, OptInt, RemoteUI,
    RgbValue, Set_ColorKey, Set_HlEntry, Set_cstr_t, Set_int, Set_uint64_t, cstr_t, int16_t,
    int32_t, kObjectTypeBoolean, kObjectTypeDict, kObjectTypeInteger, kObjectTypeNil,
    kObjectTypeString, key_value_pair, object, object_data as C2Rust_Unnamed, ptr_t, size_t,
    uint8_t, uint32_t, uint64_t, win_T,
};
use crate::src::nvim::ui::ui_call_hl_attr_define;

// Split out for size; the rest of the tree calls all of it as `highlight::*`.
pub mod dict;

pub use dict::{HLATTRS_DICT_SIZE, dict2hlattrs, hl_get_attr_by_id, hlattrs2dict};

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
pub const kRetObject: LuaRetMode = 0;
pub type NSHlAttr = [::core::ffi::c_int; 76];
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
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
pub const COLOR_ITEM_INITIALIZER: ColorItem = ColorItem {
    attr_id: -1 as ::core::ffi::c_int,
    link_id: -1 as ::core::ffi::c_int,
    version: -1 as ::core::ffi::c_int,
    is_default: false_0 != 0,
    link_global: false_0 != 0,
};
static value_init_int: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
static value_init_ColorItem: GlobalCell<ColorItem> = GlobalCell::new(COLOR_ITEM_INITIALIZER);
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
unsafe extern "C" fn set_has_ColorKey(mut set: *mut Set_ColorKey, mut key: ColorKey) -> bool {
    return mh_get_ColorKey(set, key) != MH_TOMBSTONE as uint32_t;
}
#[inline]
unsafe extern "C" fn map_get_int_ptr_t(
    mut map: *mut Map_int_ptr_t,
    mut key: ::core::ffi::c_int,
) -> ptr_t {
    let mut k: uint32_t = mh_get_int(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t.get()
    } else {
        *(*map).values.offset(k as isize)
    };
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
#[inline]
unsafe extern "C" fn map_put_ColorKey_ColorItem(
    mut map: *mut Map_ColorKey_ColorItem,
    mut key: ColorKey,
    mut value: ColorItem,
) {
    let mut val: *mut ColorItem = map_put_ref_ColorKey_ColorItem(
        map,
        key,
        ::core::ptr::null_mut::<*mut ColorKey>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
#[inline]
unsafe extern "C" fn map_get_ColorKey_ColorItem(
    mut map: *mut Map_ColorKey_ColorItem,
    mut key: ColorKey,
) -> ColorItem {
    let mut k: uint32_t = mh_get_ColorKey(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ColorItem.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
/// `fallback`'s bit in a `Dict(highlight)`'s `is_set__highlight_` mask.
const KEY_FALLBACK: ::core::ffi::c_int = 21;
pub const MAX_TYPENR: ::core::ffi::c_int = 65535 as ::core::ffi::c_int;
static hlstate_active: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static attr_entries: GlobalCell<Set_HlEntry> = GlobalCell::new(Set_HlEntry {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<HlEntry>(),
});
static combine_attr_entries: GlobalCell<Map_uint64_t_int> = GlobalCell::new(MAP_INIT);
static blend_attr_entries: GlobalCell<Map_uint64_t_int> = GlobalCell::new(MAP_INIT);
static blendthrough_attr_entries: GlobalCell<Map_uint64_t_int> = GlobalCell::new(MAP_INIT);
static urls: GlobalCell<Set_cstr_t> = GlobalCell::new(Set_cstr_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<cstr_t>(),
});
static ns_hls: GlobalCell<Map_ColorKey_ColorItem> = GlobalCell::new(Map_ColorKey_ColorItem {
    set: Set_ColorKey {
        h: MapHash {
            n_buckets: 0,
            size: 0,
            n_occupied: 0,
            upper_bound: 0,
            n_keys: 0,
            keys_capacity: 0,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<ColorKey>(),
    },
    values: ::core::ptr::null_mut::<ColorItem>(),
});
static ns_hl_attr: GlobalCell<Map_int_ptr_t> = GlobalCell::new(Map_int_ptr_t {
    set: Set_int {
        h: MapHash {
            n_buckets: 0,
            size: 0,
            n_occupied: 0,
            upper_bound: 0,
            n_keys: 0,
            keys_capacity: 0,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<::core::ffi::c_int>(),
    },
    values: ::core::ptr::null_mut::<ptr_t>(),
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
pub unsafe extern "C" fn ns_hl_def(
    mut ns_id: NS,
    mut hl_id: ::core::ffi::c_int,
    mut attrs: HlAttrs,
    mut link_id: ::core::ffi::c_int,
    mut dict: *mut KeyDict_highlight,
) {
    if ns_id == 0 as ::core::ffi::c_int {
        assert!(!dict.is_null(), "dict");
        set_hl_group(hl_id, attrs, dict, link_id);
        return;
    }
    if attrs.rgb_ae_attr & HL_DEFAULT as int32_t != 0
        && set_has_ColorKey(
            &raw mut (*ns_hls.ptr()).set,
            ColorKey {
                ns_id: ns_id,
                syn_id: hl_id,
            },
        ) as ::core::ffi::c_int
            != 0
    {
        return;
    }
    let mut p: *mut DecorProvider = get_decor_provider(ns_id, true_0 != 0);
    let mut attr_id: ::core::ffi::c_int = if link_id > 0 as ::core::ffi::c_int {
        -1 as ::core::ffi::c_int
    } else {
        hl_get_syn_attr(ns_id as ::core::ffi::c_int, hl_id, attrs)
    };
    let mut it: ColorItem = ColorItem {
        attr_id: attr_id,
        link_id: link_id,
        version: (*p).hl_valid,
        is_default: attrs.rgb_ae_attr & HL_DEFAULT as int32_t != 0,
        link_global: attrs.rgb_ae_attr & HL_GLOBAL as int32_t != 0,
    };
    map_put_ColorKey_ColorItem(
        ns_hls.ptr(),
        ColorKey {
            ns_id: ns_id,
            syn_id: hl_id,
        },
        it,
    );
    (*p).hl_cached = false_0 != 0;
}
pub unsafe extern "C" fn ns_get_hl(
    mut ns_hl: *mut NS,
    mut hl_id: ::core::ffi::c_int,
    mut link: bool,
    mut nodefault: bool,
) -> ::core::ffi::c_int {
    static recursive: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    if *ns_hl == 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    if *ns_hl < 0 as ::core::ffi::c_int {
        if ns_hl_active.get() <= 0 as ::core::ffi::c_int {
            return -1 as ::core::ffi::c_int;
        }
        *ns_hl = ns_hl_active.get();
    }
    let mut ns_id: ::core::ffi::c_int = *ns_hl as ::core::ffi::c_int;
    let mut p: *mut DecorProvider = get_decor_provider(ns_id as NS, true_0 != 0);
    let mut it: ColorItem = map_get_ColorKey_ColorItem(
        ns_hls.ptr(),
        ColorKey {
            ns_id: ns_id,
            syn_id: hl_id,
        },
    );
    let mut valid_item: bool = it.version >= (*p).hl_valid;
    if !valid_item && (*p).hl_def != LUA_NOREF && recursive.get() == 0 {
        let mut args: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut args__items: [Object; 3] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 3];
        args.capacity = 3 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh8 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh8 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: ns_id as Integer,
            },
        };
        let c2rust_fresh9 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh9 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(syn_id2name(hl_id)),
            },
        };
        let c2rust_fresh10 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh10 as isize) = object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed { boolean: link },
        };
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        (*recursive.ptr()) += 1;
        let mut ret: Object = nlua_call_ref(
            (*p).hl_def,
            b"hl_def\0".as_ptr() as *const ::core::ffi::c_char,
            args,
            kRetObject,
            ::core::ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        (*recursive.ptr()) -= 1;
        let mut fallback: bool = true_0 != 0;
        let mut tmp: bool = false_0 != 0;
        let mut attrs: HlAttrs = HLATTRS_INIT;
        if ret.type_0 as ::core::ffi::c_uint
            == kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            fallback = false_0 != 0;
            let mut dict = KeyDict_highlight::default();
            if api_dict_to_keydict(
                &raw mut dict as *mut ::core::ffi::c_void,
                Some(
                    KeyDict_highlight_get_field
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            size_t,
                        ) -> *mut KeySetLink,
                ),
                ret.data.dict,
                &raw mut err,
            ) {
                attrs = dict2hlattrs(
                    &dict,
                    true_0 != 0,
                    Some(&mut it.link_id),
                    None,
                    &raw mut err,
                );
                fallback = if dict.is_set__highlight_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEY_FALLBACK
                    != 0 as ::core::ffi::c_ulonglong
                {
                    dict.fallback as ::core::ffi::c_int
                } else {
                    true_0
                } != 0;
                tmp = dict.fallback as bool;
                if it.link_id >= 0 as ::core::ffi::c_int {
                    fallback = true_0 != 0;
                }
            }
        }
        it.attr_id = if fallback as ::core::ffi::c_int != 0 {
            -1 as ::core::ffi::c_int
        } else {
            hl_get_syn_attr(ns_id, hl_id, attrs)
        };
        it.version = (*p).hl_valid - tmp as ::core::ffi::c_int;
        it.is_default = attrs.rgb_ae_attr & HL_DEFAULT as int32_t != 0;
        it.link_global = attrs.rgb_ae_attr & HL_GLOBAL as int32_t != 0;
        map_put_ColorKey_ColorItem(
            ns_hls.ptr(),
            ColorKey {
                ns_id: ns_id,
                syn_id: hl_id,
            },
            it,
        );
        valid_item = true_0 != 0;
    }
    if it.is_default as ::core::ffi::c_int != 0 && nodefault as ::core::ffi::c_int != 0
        || !valid_item
    {
        return -1 as ::core::ffi::c_int;
    }
    if link {
        if it.attr_id >= 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        if it.link_global {
            *ns_hl = 0 as ::core::ffi::c_int as NS;
        }
        return it.link_id;
    } else {
        return it.attr_id;
    };
}
pub unsafe extern "C" fn hl_check_ns() -> bool {
    let mut ns: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if ns_hl_fast.get() > 0 as ::core::ffi::c_int {
        ns = ns_hl_fast.get() as ::core::ffi::c_int;
    } else if ns_hl_win.get() >= 0 as ::core::ffi::c_int {
        ns = ns_hl_win.get() as ::core::ffi::c_int;
    } else {
        ns = ns_hl_global.get() as ::core::ffi::c_int;
    }
    if ns_hl_active.get() == ns {
        return false_0 != 0;
    }
    ns_hl_active.set(ns as NS);
    hl_attr_active.set(highlight_attr.ptr() as *mut ::core::ffi::c_int);
    if ns > 0 as ::core::ffi::c_int {
        update_ns_hl(ns);
        let mut hl_def: *mut NSHlAttr = map_get_int_ptr_t(ns_hl_attr.ptr(), ns) as *mut NSHlAttr;
        if !hl_def.is_null() {
            hl_attr_active.set(&raw mut *hl_def as *mut ::core::ffi::c_int);
        }
    }
    need_highlight_changed.set(true_0 != 0);
    return true_0 != 0;
}
pub unsafe extern "C" fn win_check_ns_hl(mut wp: *mut win_T) -> bool {
    ns_hl_win.set(
        (if !wp.is_null() {
            (*wp).w_ns_hl
        } else {
            -1 as ::core::ffi::c_int
        }) as NS,
    );
    return hl_check_ns();
}
pub unsafe extern "C" fn hl_ns_get_attrs(
    mut ns_id: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
    mut optional: *mut bool,
    mut attrs: *mut HlAttrs,
) -> bool {
    let mut opt: bool = if !optional.is_null() {
        *optional as ::core::ffi::c_int
    } else {
        true_0
    } != 0;
    let mut syn_attr: ::core::ffi::c_int = syn_ns_id2attr(ns_id, hl_id, &raw mut opt);
    if !optional.is_null() {
        *optional = opt;
    }
    if syn_attr <= 0 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    *attrs = syn_attr2entry(syn_attr);
    return true_0 != 0;
}
pub unsafe extern "C" fn hl_get_ui_attr(
    mut ns_id: ::core::ffi::c_int,
    mut idx: ::core::ffi::c_int,
    mut final_id: ::core::ffi::c_int,
    mut optional: bool,
) -> ::core::ffi::c_int {
    let mut attrs: HlAttrs = HLATTRS_INIT;
    let mut available: bool = false_0 != 0;
    if final_id > 0 as ::core::ffi::c_int {
        available = hl_ns_get_attrs(ns_id, final_id, &raw mut optional, &raw mut attrs);
    }
    if HLF_PNI as ::core::ffi::c_int <= idx && idx <= HLF_PST as ::core::ffi::c_int {
        if attrs.hl_blend == -1 as int32_t && p_pb.get() > 0 as OptInt {
            attrs.hl_blend = p_pb.get() as ::core::ffi::c_int as int32_t;
        }
        if pum_drawn() {
            must_redraw_pum.set(true_0 != 0);
        }
    }
    if optional as ::core::ffi::c_int != 0 && !available {
        return 0 as ::core::ffi::c_int;
    }
    return get_attr_entry(HlEntry {
        attr: attrs,
        kind: kHlUI,
        id1: idx,
        id2: final_id,
        winid: 0,
    });
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
pub unsafe extern "C" fn update_window_hl(mut wp: *mut win_T, mut invalid: bool) {
    let mut ns_id: ::core::ffi::c_int = (*wp).w_ns_hl;
    update_ns_hl(ns_id);
    if ns_id != (*wp).w_ns_hl_active || (*wp).w_ns_hl_attr.is_null() {
        (*wp).w_ns_hl_active = ns_id;
        let mut hl_def_ptr: *mut NSHlAttr =
            map_get_int_ptr_t(ns_hl_attr.ptr(), ns_id) as *mut NSHlAttr;
        if !hl_def_ptr.is_null() {
            (*wp).w_ns_hl_attr = &raw mut *hl_def_ptr as *mut ::core::ffi::c_int;
        } else {
            (*wp).w_ns_hl_attr = highlight_attr.ptr() as *mut ::core::ffi::c_int;
        }
    }
    let mut hl_def: *mut ::core::ffi::c_int = (*wp).w_ns_hl_attr;
    if (*wp).w_hl_needs_update == 0 && !invalid {
        return;
    }
    (*wp).w_hl_needs_update = false_0;
    let mut float_win: bool =
        (*wp).w_floating as ::core::ffi::c_int != 0 && !(*wp).w_config.external;
    if float_win as ::core::ffi::c_int != 0
        && *hl_def.offset(HLF_NFLOAT as ::core::ffi::c_int as isize) != 0 as ::core::ffi::c_int
        && ns_id > 0 as ::core::ffi::c_int
    {
        (*wp).w_hl_attr_normal = *hl_def.offset(HLF_NFLOAT as ::core::ffi::c_int as isize);
    } else if *hl_def.offset(HLF_NONE as ::core::ffi::c_int as isize) > 0 as ::core::ffi::c_int {
        (*wp).w_hl_attr_normal = *hl_def.offset(HLF_NONE as ::core::ffi::c_int as isize);
    } else if float_win {
        (*wp).w_hl_attr_normal = if *(*hl_attr_active.ptr())
            .offset(HLF_NFLOAT as ::core::ffi::c_int as isize)
            > 0 as ::core::ffi::c_int
        {
            *(*hl_attr_active.ptr()).offset(HLF_NFLOAT as ::core::ffi::c_int as isize)
        } else {
            (*highlight_attr.ptr())[HLF_NFLOAT as ::core::ffi::c_int as usize]
        };
    } else {
        (*wp).w_hl_attr_normal = 0 as ::core::ffi::c_int;
    }
    if (*wp).w_floating {
        (*wp).w_hl_attr_normal = hl_apply_winblend(
            (*wp).w_onebuf_opt.wo_winbl as ::core::ffi::c_int,
            (*wp).w_hl_attr_normal,
        );
    }
    (*wp).w_config.shadow = false_0 != 0;
    if (*wp).w_floating as ::core::ffi::c_int != 0
        && (*wp).w_config.border as ::core::ffi::c_int != 0
    {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 8 as ::core::ffi::c_int {
            let mut attr: ::core::ffi::c_int =
                *hl_def.offset(HLF_BORDER as ::core::ffi::c_int as isize);
            if (*wp).w_config.border_hl_ids[i as usize] != 0 {
                attr = hl_get_ui_attr(
                    ns_id,
                    HLF_BORDER as ::core::ffi::c_int,
                    (*wp).w_config.border_hl_ids[i as usize],
                    false_0 != 0,
                );
            }
            attr = hl_apply_winblend((*wp).w_onebuf_opt.wo_winbl as ::core::ffi::c_int, attr);
            if syn_attr2entry(attr).hl_blend > 0 as int32_t {
                (*wp).w_config.shadow = true_0 != 0;
            }
            (*wp).w_config.border_attr[i as usize] = attr;
            i += 1;
        }
    }
    check_blending(wp);
    if *hl_def.offset(HLF_INACTIVE as ::core::ffi::c_int as isize) == 0 as ::core::ffi::c_int {
        (*wp).w_hl_attr_normalnc = hl_combine_attr(
            *(*hl_attr_active.ptr()).offset(HLF_INACTIVE as ::core::ffi::c_int as isize),
            (*wp).w_hl_attr_normal,
        );
    } else {
        (*wp).w_hl_attr_normalnc = *hl_def.offset(HLF_INACTIVE as ::core::ffi::c_int as isize);
    }
    if (*wp).w_floating {
        (*wp).w_hl_attr_normalnc = hl_apply_winblend(
            (*wp).w_onebuf_opt.wo_winbl as ::core::ffi::c_int,
            (*wp).w_hl_attr_normalnc,
        );
    }
}
pub unsafe extern "C" fn update_ns_hl(mut ns_id: ::core::ffi::c_int) {
    if ns_id <= 0 as ::core::ffi::c_int {
        return;
    }
    let mut p: *mut DecorProvider = get_decor_provider(ns_id as NS, true_0 != 0);
    if (*p).hl_cached {
        return;
    }
    let mut alloc: *mut *mut NSHlAttr = map_put_ref_int_ptr_t(
        ns_hl_attr.ptr(),
        ns_id,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_int>(),
        ::core::ptr::null_mut::<bool>(),
    ) as *mut *mut NSHlAttr;
    if (*alloc).is_null() {
        *alloc = xmalloc(::core::mem::size_of::<NSHlAttr>()) as *mut NSHlAttr;
    }
    let mut hl_attrs: *mut ::core::ffi::c_int = &raw mut **alloc as *mut ::core::ffi::c_int;
    let mut hlf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while hlf < HLF_COUNT as ::core::ffi::c_int {
        let mut id: ::core::ffi::c_int = syn_check_group(
            *(hlf_names.ptr() as *mut *const ::core::ffi::c_char).offset(hlf as isize),
            strlen(*(hlf_names.ptr() as *mut *const ::core::ffi::c_char).offset(hlf as isize)),
        );
        let mut optional: bool =
            hlf == HLF_INACTIVE as ::core::ffi::c_int || hlf == HLF_NFLOAT as ::core::ffi::c_int;
        *hl_attrs.offset(hlf as isize) = hl_get_ui_attr(ns_id, hlf, id, optional);
        hlf += 1;
    }
    let mut normality: ::core::ffi::c_int = syn_check_group(
        b"Normal\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
    );
    *hl_attrs.offset(HLF_NONE as ::core::ffi::c_int as isize) =
        hl_get_ui_attr(ns_id, -1 as ::core::ffi::c_int, normality, true_0 != 0);
    p = get_decor_provider(ns_id as NS, true_0 != 0);
    (*p).hl_cached = true_0 != 0;
}
pub unsafe extern "C" fn win_bg_attr(mut wp: *mut win_T) -> ::core::ffi::c_int {
    if ns_hl_fast.get() < 0 as ::core::ffi::c_int {
        let mut local: ::core::ffi::c_int = if wp == curwin.get() {
            (*wp).w_hl_attr_normal
        } else {
            (*wp).w_hl_attr_normalnc
        };
        if local != 0 {
            return local;
        }
    }
    if wp == curwin.get()
        || *(*hl_attr_active.ptr()).offset(HLF_INACTIVE as ::core::ffi::c_int as isize)
            == 0 as ::core::ffi::c_int
    {
        return *(*hl_attr_active.ptr()).offset(HLF_NONE as ::core::ffi::c_int as isize);
    } else {
        return *(*hl_attr_active.ptr()).offset(HLF_INACTIVE as ::core::ffi::c_int as isize);
    };
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
        mh_clear(&raw mut (*blend_attr_entries.ptr()).set.h);
        mh_clear(&raw mut (*blendthrough_attr_entries.ptr()).set.h);
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
        xfree((*blend_attr_entries.ptr()).set.keys as *mut ::core::ffi::c_void);
        xfree((*blend_attr_entries.ptr()).set.h.hash as *mut ::core::ffi::c_void);
        (*blend_attr_entries.ptr()).set = Set_uint64_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<uint64_t>(),
        };
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*blend_attr_entries.ptr()).values as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
        xfree((*blendthrough_attr_entries.ptr()).set.keys as *mut ::core::ffi::c_void);
        xfree((*blendthrough_attr_entries.ptr()).set.h.hash as *mut ::core::ffi::c_void);
        (*blendthrough_attr_entries.ptr()).set = Set_uint64_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<uint64_t>(),
        };
        let mut ptr__1: *mut *mut ::core::ffi::c_void =
            &raw mut (*blendthrough_attr_entries.ptr()).values as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__1);
        *ptr__1 = NULL_0;
        let _ = *ptr__1;
        xfree((*ns_hls.ptr()).set.keys as *mut ::core::ffi::c_void);
        xfree((*ns_hls.ptr()).set.h.hash as *mut ::core::ffi::c_void);
        (*ns_hls.ptr()).set = Set_ColorKey {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<ColorKey>(),
        };
        let mut ptr__2: *mut *mut ::core::ffi::c_void =
            &raw mut (*ns_hls.ptr()).values as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__2);
        *ptr__2 = NULL_0;
        let _ = *ptr__2;
        xfree((*urls.ptr()).keys as *mut ::core::ffi::c_void);
        xfree((*urls.ptr()).h.hash as *mut ::core::ffi::c_void);
        urls.set(Set_cstr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<cstr_t>(),
        });
    };
}
pub unsafe extern "C" fn hl_invalidate_blends() {
    mh_clear(&raw mut (*blend_attr_entries.ptr()).set.h);
    mh_clear(&raw mut (*blendthrough_attr_entries.ptr()).set.h);
    highlight_changed();
    update_window_hl(curwin.get(), true_0 != 0);
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
unsafe extern "C" fn get_colors_force(mut attrs: HlAttrs) -> HlAttrs {
    if attrs.rgb_bg_color == -1 as RgbValue {
        attrs.rgb_bg_color = normal_bg.get();
    }
    if attrs.rgb_fg_color == -1 as RgbValue {
        attrs.rgb_fg_color = normal_fg.get();
    }
    if attrs.rgb_sp_color == -1 as RgbValue {
        attrs.rgb_sp_color = normal_sp.get();
    }
    let mut dark_: bool = *p_bg.get() as ::core::ffi::c_int == 'd' as ::core::ffi::c_int;
    attrs.rgb_fg_color = if attrs.rgb_fg_color != -1 as RgbValue {
        attrs.rgb_fg_color
    } else if dark_ as ::core::ffi::c_int != 0 {
        0xffffff as RgbValue
    } else {
        0 as RgbValue
    };
    attrs.rgb_bg_color = if attrs.rgb_bg_color != -1 as RgbValue {
        attrs.rgb_bg_color
    } else if dark_ as ::core::ffi::c_int != 0 {
        0 as RgbValue
    } else {
        0xffffff as RgbValue
    };
    attrs.rgb_sp_color = if attrs.rgb_sp_color != -1 as RgbValue {
        attrs.rgb_sp_color
    } else {
        0xff0000 as RgbValue
    };
    if attrs.rgb_ae_attr & HL_INVERSE as int32_t != 0 {
        let mut temp: ::core::ffi::c_int = attrs.rgb_bg_color as ::core::ffi::c_int;
        attrs.rgb_bg_color = attrs.rgb_fg_color;
        attrs.rgb_fg_color = temp as RgbValue;
        attrs.rgb_ae_attr = (attrs.rgb_ae_attr as ::core::ffi::c_int & !(HL_INVERSE)) as int32_t;
    }
    return attrs;
}
pub unsafe extern "C" fn hl_blend_attrs(
    mut back_attr: ::core::ffi::c_int,
    mut front_attr: ::core::ffi::c_int,
    mut through: *mut bool,
) -> ::core::ffi::c_int {
    if front_attr < 0 as ::core::ffi::c_int || back_attr < 0 as ::core::ffi::c_int {
        return front_attr;
    }
    let mut fattrs_raw: HlAttrs = syn_attr2entry(front_attr);
    let mut fattrs: HlAttrs = get_colors_force(fattrs_raw);
    let mut ratio: ::core::ffi::c_int = fattrs.hl_blend as ::core::ffi::c_int;
    if ratio <= 0 as ::core::ffi::c_int {
        *through = false_0 != 0;
        return front_attr;
    }
    let mut combine_tag: uint64_t = (back_attr as uint32_t as uint64_t) << 32 as ::core::ffi::c_int
        | front_attr as uint32_t as uint64_t;
    let mut map: *mut Map_uint64_t_int = if *through as ::core::ffi::c_int != 0 {
        blendthrough_attr_entries.ptr()
    } else {
        blend_attr_entries.ptr()
    };
    let mut id: ::core::ffi::c_int = map_get_uint64_t_int(map, combine_tag);
    if id > 0 as ::core::ffi::c_int {
        return id;
    }
    let mut battrs_raw: HlAttrs = syn_attr2entry(back_attr);
    let mut battrs: HlAttrs = get_colors_force(battrs_raw);
    let mut cattrs: HlAttrs = HlAttrs {
        rgb_ae_attr: 0,
        cterm_ae_attr: 0,
        rgb_fg_color: 0,
        rgb_bg_color: 0,
        rgb_sp_color: 0,
        cterm_fg_color: 0,
        cterm_bg_color: 0,
        hl_blend: 0,
        url: 0,
    };
    if *through {
        cattrs = battrs;
        cattrs.rgb_fg_color = rgb_blend(
            ratio,
            battrs.rgb_fg_color as ::core::ffi::c_int,
            fattrs.rgb_bg_color as ::core::ffi::c_int,
        ) as RgbValue;
        if cattrs.rgb_ae_attr & HL_UNDERLINE_MASK as int32_t != 0
            && battrs_raw.rgb_sp_color != -1 as RgbValue
        {
            cattrs.rgb_sp_color = rgb_blend(
                ratio,
                battrs.rgb_sp_color as ::core::ffi::c_int,
                fattrs.rgb_bg_color as ::core::ffi::c_int,
            ) as RgbValue;
        } else {
            cattrs.rgb_sp_color = -1 as ::core::ffi::c_int as RgbValue;
        }
        cattrs.cterm_bg_color = fattrs.cterm_bg_color;
        cattrs.cterm_fg_color =
            cterm_blend(ratio, battrs.cterm_fg_color, fattrs.cterm_bg_color) as int16_t;
        cattrs.rgb_ae_attr = (cattrs.rgb_ae_attr as ::core::ffi::c_int
            & !(HL_FG_INDEXED | HL_BG_INDEXED)) as int32_t;
    } else {
        cattrs = fattrs;
        cattrs.rgb_fg_color = rgb_blend(
            ratio / 2 as ::core::ffi::c_int,
            battrs.rgb_fg_color as ::core::ffi::c_int,
            fattrs.rgb_fg_color as ::core::ffi::c_int,
        ) as RgbValue;
        if cattrs.rgb_ae_attr & HL_UNDERLINE_MASK as int32_t != 0 {
            cattrs.rgb_sp_color = rgb_blend(
                ratio / 2 as ::core::ffi::c_int,
                battrs.rgb_bg_color as ::core::ffi::c_int,
                fattrs.rgb_sp_color as ::core::ffi::c_int,
            ) as RgbValue;
        } else {
            cattrs.rgb_sp_color = -1 as ::core::ffi::c_int as RgbValue;
        }
        cattrs.rgb_ae_attr = (cattrs.rgb_ae_attr as ::core::ffi::c_int
            & !(HL_FG_INDEXED | HL_BG_INDEXED)) as int32_t;
    }
    if ratio == 100 as ::core::ffi::c_int && battrs_raw.rgb_bg_color == -1 as RgbValue {
        cattrs.rgb_bg_color = -1 as ::core::ffi::c_int as RgbValue;
    } else {
        cattrs.rgb_bg_color = (if battrs_raw.rgb_bg_color == -1 as RgbValue
            && fattrs_raw.rgb_bg_color == -1 as RgbValue
        {
            -1 as ::core::ffi::c_int
        } else {
            rgb_blend(
                ratio,
                battrs.rgb_bg_color as ::core::ffi::c_int,
                fattrs.rgb_bg_color as ::core::ffi::c_int,
            )
        }) as RgbValue;
    }
    cattrs.hl_blend = -1 as ::core::ffi::c_int as int32_t;
    let mut kind: HlKind = (if *through as ::core::ffi::c_int != 0 {
        kHlBlendThrough as ::core::ffi::c_int
    } else {
        kHlBlend as ::core::ffi::c_int
    }) as HlKind;
    id = get_attr_entry(HlEntry {
        attr: cattrs,
        kind: kind,
        id1: back_attr,
        id2: front_attr,
        winid: 0,
    });
    if id > 0 as ::core::ffi::c_int {
        map_put_uint64_t_int(map, combine_tag, id);
    }
    return id;
}
unsafe extern "C" fn rgb_blend(
    mut ratio: ::core::ffi::c_int,
    mut rgb1: ::core::ffi::c_int,
    mut rgb2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut a: ::core::ffi::c_int = ratio;
    let mut b: ::core::ffi::c_int = 100 as ::core::ffi::c_int - ratio;
    let mut r1: ::core::ffi::c_int =
        (rgb1 & 0xff0000 as ::core::ffi::c_int) >> 16 as ::core::ffi::c_int;
    let mut g1: ::core::ffi::c_int =
        (rgb1 & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
    let mut b1: ::core::ffi::c_int = (rgb1 & 0xff as ::core::ffi::c_int) >> 0 as ::core::ffi::c_int;
    let mut r2: ::core::ffi::c_int =
        (rgb2 & 0xff0000 as ::core::ffi::c_int) >> 16 as ::core::ffi::c_int;
    let mut g2: ::core::ffi::c_int =
        (rgb2 & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
    let mut b2: ::core::ffi::c_int = (rgb2 & 0xff as ::core::ffi::c_int) >> 0 as ::core::ffi::c_int;
    let mut mr: ::core::ffi::c_int = (a * r1 + b * r2) / 100 as ::core::ffi::c_int;
    let mut mg: ::core::ffi::c_int = (a * g1 + b * g2) / 100 as ::core::ffi::c_int;
    let mut mb: ::core::ffi::c_int = (a * b1 + b * b2) / 100 as ::core::ffi::c_int;
    return (mr << 16 as ::core::ffi::c_int) + (mg << 8 as ::core::ffi::c_int) + mb;
}
unsafe extern "C" fn cterm_blend(
    mut ratio: ::core::ffi::c_int,
    mut c1: int16_t,
    mut c2: int16_t,
) -> ::core::ffi::c_int {
    let mut rgb1: ::core::ffi::c_int = hl_cterm2rgb_color(c1 as ::core::ffi::c_int);
    let mut rgb2: ::core::ffi::c_int = hl_cterm2rgb_color(c2 as ::core::ffi::c_int);
    let mut rgb_blended: ::core::ffi::c_int = rgb_blend(ratio, rgb1, rgb2);
    return hl_rgb2cterm_color(rgb_blended);
}
unsafe extern "C" fn hl_rgb2cterm_color(mut rgb: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int =
        (rgb & 0xff0000 as ::core::ffi::c_int) >> 16 as ::core::ffi::c_int;
    let mut g: ::core::ffi::c_int = (rgb & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
    let mut b: ::core::ffi::c_int = (rgb & 0xff as ::core::ffi::c_int) >> 0 as ::core::ffi::c_int;
    return r * 6 as ::core::ffi::c_int / 256 as ::core::ffi::c_int * 36 as ::core::ffi::c_int
        + g * 6 as ::core::ffi::c_int / 256 as ::core::ffi::c_int * 6 as ::core::ffi::c_int
        + b * 6 as ::core::ffi::c_int / 256 as ::core::ffi::c_int;
}
unsafe extern "C" fn hl_cterm2rgb_color(mut nr: ::core::ffi::c_int) -> ::core::ffi::c_int {
    static cube_value: GlobalCell<[::core::ffi::c_int; 6]> = GlobalCell::new([
        0 as ::core::ffi::c_int,
        0x5f as ::core::ffi::c_int,
        0x87 as ::core::ffi::c_int,
        0xaf as ::core::ffi::c_int,
        0xd7 as ::core::ffi::c_int,
        0xff as ::core::ffi::c_int,
    ]);
    static grey_ramp: GlobalCell<[::core::ffi::c_int; 24]> = GlobalCell::new([
        0x8 as ::core::ffi::c_int,
        0x12 as ::core::ffi::c_int,
        0x1c as ::core::ffi::c_int,
        0x26 as ::core::ffi::c_int,
        0x30 as ::core::ffi::c_int,
        0x3a as ::core::ffi::c_int,
        0x44 as ::core::ffi::c_int,
        0x4e as ::core::ffi::c_int,
        0x58 as ::core::ffi::c_int,
        0x62 as ::core::ffi::c_int,
        0x6c as ::core::ffi::c_int,
        0x76 as ::core::ffi::c_int,
        0x80 as ::core::ffi::c_int,
        0x8a as ::core::ffi::c_int,
        0x94 as ::core::ffi::c_int,
        0x9e as ::core::ffi::c_int,
        0xa8 as ::core::ffi::c_int,
        0xb2 as ::core::ffi::c_int,
        0xbc as ::core::ffi::c_int,
        0xc6 as ::core::ffi::c_int,
        0xd0 as ::core::ffi::c_int,
        0xda as ::core::ffi::c_int,
        0xe4 as ::core::ffi::c_int,
        0xee as ::core::ffi::c_int,
    ]);
    static ansi_table: GlobalCell<[[uint8_t; 4]; 16]> = GlobalCell::new([
        [0 as uint8_t, 0 as uint8_t, 0 as uint8_t, 1 as uint8_t],
        [224 as uint8_t, 0 as uint8_t, 0 as uint8_t, 2 as uint8_t],
        [0 as uint8_t, 224 as uint8_t, 0 as uint8_t, 3 as uint8_t],
        [224 as uint8_t, 224 as uint8_t, 0 as uint8_t, 4 as uint8_t],
        [0 as uint8_t, 0 as uint8_t, 224 as uint8_t, 5 as uint8_t],
        [224 as uint8_t, 0 as uint8_t, 224 as uint8_t, 6 as uint8_t],
        [0 as uint8_t, 224 as uint8_t, 224 as uint8_t, 7 as uint8_t],
        [224 as uint8_t, 224 as uint8_t, 224 as uint8_t, 8 as uint8_t],
        [128 as uint8_t, 128 as uint8_t, 128 as uint8_t, 9 as uint8_t],
        [255 as uint8_t, 64 as uint8_t, 64 as uint8_t, 10 as uint8_t],
        [64 as uint8_t, 255 as uint8_t, 64 as uint8_t, 11 as uint8_t],
        [255 as uint8_t, 255 as uint8_t, 64 as uint8_t, 12 as uint8_t],
        [64 as uint8_t, 64 as uint8_t, 255 as uint8_t, 13 as uint8_t],
        [255 as uint8_t, 64 as uint8_t, 255 as uint8_t, 14 as uint8_t],
        [64 as uint8_t, 255 as uint8_t, 255 as uint8_t, 15 as uint8_t],
        [
            255 as uint8_t,
            255 as uint8_t,
            255 as uint8_t,
            16 as uint8_t,
        ],
    ]);
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut g: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut b: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut idx: ::core::ffi::c_int = 0;
    if nr < 16 as ::core::ffi::c_int {
        r = (*ansi_table.ptr())[nr as usize][0 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int;
        g = (*ansi_table.ptr())[nr as usize][1 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int;
        b = (*ansi_table.ptr())[nr as usize][2 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int;
    } else if nr < 232 as ::core::ffi::c_int {
        idx = nr - 16 as ::core::ffi::c_int;
        r = (*cube_value.ptr())
            [(idx / 36 as ::core::ffi::c_int % 6 as ::core::ffi::c_int) as usize];
        g = (*cube_value.ptr())[(idx / 6 as ::core::ffi::c_int % 6 as ::core::ffi::c_int) as usize];
        b = (*cube_value.ptr())[(idx % 6 as ::core::ffi::c_int) as usize];
    } else if nr < 256 as ::core::ffi::c_int {
        idx = nr - 232 as ::core::ffi::c_int;
        r = (*grey_ramp.ptr())[idx as usize];
        g = (*grey_ramp.ptr())[idx as usize];
        b = (*grey_ramp.ptr())[idx as usize];
    }
    return (r << 16 as ::core::ffi::c_int) + (g << 8 as ::core::ffi::c_int) + b;
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

/// The attribute the window resolves highlight group `hlf` to: its own
/// namespace's table when one is active, otherwise the global one.
pub unsafe fn win_hl_attr(wp: *mut win_T, hlf: ::core::ffi::c_int) -> ::core::ffi::c_int {
    *if !(*wp).w_ns_hl_attr.is_null() && ns_hl_fast.get() < 0 as ::core::ffi::c_int {
        (*wp).w_ns_hl_attr
    } else {
        hl_attr_active.get()
    }
    .offset(hlf as isize)
}
