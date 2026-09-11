#![deny(unsafe_op_in_unsafe_fn)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::api::private::helpers::{
    api_typename, cstr_to_string, find_buffer_by_handle, find_window_by_handle, object_to_hl_id,
    string_to_cstr,
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
use crate::grid::schar_high;
use crate::marktree::key::{
    MtFlags, mt_decor, mt_invalid, mt_invalidate, mt_no_undo, mt_paired, mt_right,
};
use crate::marktree::mt_inspect;
use crate::mbyte::{mb_string2cells, utfc_ptr2schar};
use crate::memory::{strequal, xfree};
use crate::r#move::changed_window_setting;
use crate::pos::{MAXCOL, MAXLNUM};
use crate::sign::init_sign_text;
use crate::types::{
    ApiDict, Array, Boolean, BufferHandle, ColNr, DecorExt, DecorHighlightInline, DecorInline,
    DecorInlineData, DecorPriority, DecorProvider, DecorSignHighlight, DecorVirtText,
    DecorVirtText_data, Error, ExtmarkInfoArray, ExtmarkType, Integer, KeyDict_get_extmark,
    KeyDict_get_extmarks, KeyDict_ns_opts, KeyDict_set_decoration_provider, KeyDict_set_extmark,
    LineNr, LuaRef, MTKey, MTPair, NS, Object, ScreenChar, String_0, VirtLines, VirtText,
    VirtTextChunk, Window, WindowHandle, int32_t, kObjectTypeArray, size_t, uint8_t, uint16_t,
    uint32_t, virt_line,
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
pub const kSHConcealLines: ::core::ffi::c_uint = 128;
pub const kSHConceal: ::core::ffi::c_uint = 64;
pub const kSHSpellOff: ::core::ffi::c_uint = 32;
pub const kSHSpellOn: ::core::ffi::c_uint = 16;
pub const kSHUIWatchedOverlay: ::core::ffi::c_uint = 8;
pub const kSHUIWatched: ::core::ffi::c_uint = 4;
pub const kSHHlEol: ::core::ffi::c_uint = 2;
pub const kSHIsSign: ::core::ffi::c_uint = 1;
pub const kExtmarkHighlight: ExtmarkType = 32;
pub const kExtmarkVirtLines: ExtmarkType = 16;
pub const kExtmarkVirtText: ExtmarkType = 8;
pub const kExtmarkSign: ExtmarkType = 2;
pub const kExtmarkNone: ExtmarkType = 1;
/// No allocated decoration: the index a `DecorSignHighlight` chain ends on.
pub const DECOR_ID_INVALID: uint32_t = u32::MAX;
/// Where a decoration with no `priority` of its own sits.
pub const DECOR_PRIORITY_BASE: ::core::ffi::c_int = 0x1000;
pub const DECOR_HIGHLIGHT_INLINE_INIT: DecorHighlightInline = DecorHighlightInline {
    flags: 0,
    // `DECOR_PRIORITY_BASE`, in the width the field carries it.
    priority: 0x1000,
    hl_id: 0,
    conceal_char: 0,
};
pub const DECOR_SIGN_HIGHLIGHT_INIT: DecorSignHighlight = DecorSignHighlight {
    flags: 0,
    // `DECOR_PRIORITY_BASE`, in the width the field carries it.
    priority: 0x1000,
    hl_id: 0,
    text: [0; 2],
    sign_name: ::core::ptr::null_mut(),
    sign_add_id: 0,
    number_hl_id: 0,
    line_hl_id: 0,
    cursorline_hl_id: 0,
    next: DECOR_ID_INVALID,
    url: ::core::ptr::null(),
};
pub const DECOR_INLINE_INIT: DecorInline = DecorInline {
    ext: false,
    data: DecorInlineData {
        hl: DECOR_HIGHLIGHT_INLINE_INIT,
    },
};
