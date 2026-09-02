#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::private::helpers::{
    api_typename, arena_array, arena_dict, copy_string, cstr_as_string, find_buffer_by_handle,
    find_window_by_handle, has_key, object_to_hl_id, string_to_cstr,
};
use crate::charset::{transstr, vim_isprintc};
use crate::decoration::{
    clear_virtlines, clear_virttext, decor_free, decor_put_sh, decor_put_vt, decor_range_add_sh,
    decor_range_add_virt, decor_sh_from_inline, decor_to_dict_legacy, hl_group_name, kHlModeBlend,
    kHlModeCombine, kHlModeReplace, kHlModeUnknown, kVLLeftcol, kVLScroll, kVPosEndOfLine,
    kVPosEndOfLineRightAlign, kVPosInline, kVPosOverlay, kVPosRightAlign, kVPosWinCol, kVTHide,
    kVTIsLines, kVTLinesAbove, kVTRepeatLinebreak,
};
use crate::decoration_provider::{decor_provider_clear, get_decor_provider, kDecorProviderActive};
use crate::drawscreen::{UPD_NOT_VALID, redraw_all_later};
use crate::extmark::{extmark_clear, extmark_del_id, extmark_from_id, extmark_get, extmark_set};
use crate::global_cell::GlobalCell;
use crate::grid::schar_high;
use crate::main::{namespace_ids, namespace_localscope, next_namespace_id};
use crate::map::{
    map_put_ref_string_int, mh_delete_uint32_t, mh_get_ptr_t, mh_get_string, mh_put_ptr_t,
    mh_put_uint32_t, set_has_uint32_t,
};
use crate::marktree::key::{
    MtFlags, mt_decor, mt_invalid, mt_invalidate, mt_no_undo, mt_paired, mt_right,
};
use crate::marktree::mt_inspect;
use crate::mbyte::{mb_string2cells, utfc_ptr2schar};
use crate::memory::{strequal, xfree, xrealloc};
use crate::r#move::changed_window_setting;
use crate::pos::{MAXCOL, MAXLNUM};
use crate::sign::init_sign_text;
use crate::types::{
    Arena, Array, Boolean, Buffer, DecorExt, DecorHighlightInline, DecorInline, DecorInlineData,
    DecorPriority, DecorProvider, DecorSignHighlight, DecorVirtText, DecorVirtText_data, Dict,
    Error, ExtmarkInfoArray, ExtmarkType, Integer, KeyDict_get_extmark, KeyDict_get_extmarks,
    KeyDict_ns_opts, KeyDict_set_decoration_provider, KeyDict_set_extmark, KeySetLink, LuaRef,
    MHPutStatus, MTKey, MTPair, Map_String_int, Map_uint32_t_uint32_t, MapHash, NS, Object,
    OptionalKeys, Set_ptr_t, Set_uint32_t, String_0, UndoObjectType, VirtLines, VirtText,
    VirtTextChunk, Window, buf_T, colnr_T, handle_T, int32_t, kErrorTypeNone, kObjectTypeArray,
    linenr_T, ptr_t, schar_T, size_t, uint8_t, uint16_t, uint32_t, virt_line, win_T,
};

// The carve of the transpiled module; see each child's docs.
mod decor;
mod ns;
mod query;
mod set;

pub use self::decor::*;
pub use self::ns::*;
pub use self::query::*;
pub use self::set::*;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub const kSHConcealLines: ::core::ffi::c_uint = 128;
pub const kSHConceal: ::core::ffi::c_uint = 64;
pub const kSHSpellOff: ::core::ffi::c_uint = 32;
pub const kSHSpellOn: ::core::ffi::c_uint = 16;
pub const kSHUIWatchedOverlay: ::core::ffi::c_uint = 8;
pub const kSHUIWatched: ::core::ffi::c_uint = 4;
pub const kSHHlEol: ::core::ffi::c_uint = 2;
pub const kSHIsSign: ::core::ffi::c_uint = 1;
pub const kMHExisting: MHPutStatus = 0;
pub const kExtmarkHighlight: ExtmarkType = 32;
pub const kExtmarkVirtLines: ExtmarkType = 16;
pub const kExtmarkVirtText: ExtmarkType = 8;
pub const kExtmarkSign: ExtmarkType = 2;
pub const kExtmarkNone: ExtmarkType = 1;
pub struct DecorProviderCallback {
    pub name: *const ::core::ffi::c_char,
    pub source: *mut LuaRef,
    pub dest: *mut LuaRef,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const ARRAY_DICT_INIT: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const DECOR_ID_INVALID: ::core::ffi::c_uint = UINT32_MAX;
pub const DECOR_PRIORITY_BASE: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const DECOR_HIGHLIGHT_INLINE_INIT: DecorHighlightInline = DecorHighlightInline {
    flags: 0 as uint16_t,
    priority: DECOR_PRIORITY_BASE as DecorPriority,
    hl_id: 0 as ::core::ffi::c_int,
    conceal_char: 0 as schar_T,
};
pub const DECOR_SIGN_HIGHLIGHT_INIT: DecorSignHighlight = DecorSignHighlight {
    flags: 0 as uint16_t,
    priority: DECOR_PRIORITY_BASE as DecorPriority,
    hl_id: 0 as ::core::ffi::c_int,
    text: [0 as schar_T, 0 as schar_T],
    sign_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    sign_add_id: 0 as ::core::ffi::c_int,
    number_hl_id: 0 as ::core::ffi::c_int,
    line_hl_id: 0 as ::core::ffi::c_int,
    cursorline_hl_id: 0 as ::core::ffi::c_int,
    next: DECOR_ID_INVALID as uint32_t,
    url: ::core::ptr::null::<::core::ffi::c_char>(),
};
pub const DECOR_INLINE_INIT: DecorInline = DecorInline {
    ext: false,
    data: DecorInlineData {
        hl: DECOR_HIGHLIGHT_INLINE_INIT,
    },
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
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
pub const KEYSET_OPTIDX_set_extmark__id: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__url: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__spell: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__strict: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__end_col: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__conceal: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__hl_mode: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__end_row: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__end_line: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__hl_group: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__priority: ::core::ffi::c_int = 13 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__sign_text: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__virt_text: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__virt_lines: ::core::ffi::c_int = 19 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark___subpriority: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__undo_restore: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__conceal_lines: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__right_gravity: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__virt_text_pos: ::core::ffi::c_int = 26 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__virt_text_win_col: ::core::ffi::c_int =
    31 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__virt_lines_overflow: ::core::ffi::c_int =
    34 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_extmark__hl_name: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_extmarks__type: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_extmarks__limit: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_extmarks__hl_name: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_ns_opts__wins: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYDICT_INIT: KeyDict_ns_opts = KeyDict_ns_opts {
    is_set__ns_opts_: 0 as OptionalKeys,
    wins: Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    },
};
