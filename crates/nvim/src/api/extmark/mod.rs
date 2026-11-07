#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::api::keysets::{
    KeyDict_get_extmark, KeyDict_get_extmarks, KeyDict_ns_opts, KeyDict_set_decoration_provider,
    KeyDict_set_extmark,
};
use crate::api::private::helpers::{
    api_typename, find_buffer_by_handle, find_window_by_handle, object_to_hl_id,
};
use crate::charset::vim_isprintc;
use crate::decoration::{
    HlSource, decor_put_sh, decor_put_vt, decor_range_add_sh, decor_range_add_virt,
    decor_sh_from_inline, decor_to_dict_legacy, hl_group_name, kHlModeBlend, kHlModeCombine,
    kHlModeReplace, kHlModeUnknown, kVLLeftcol, kVLScroll, kVPosEndOfLine,
    kVPosEndOfLineRightAlign, kVPosInline, kVPosOverlay, kVPosRightAlign, kVPosWinCol, kVTHide,
    kVTIsLines, kVTLinesAbove, kVTRepeatLinebreak,
};
use crate::decoration_provider::{decor_provider_clear, kDecorProviderActive, with_decor_provider};
use crate::drawscreen::{UPD_NOT_VALID, redraw_all_later};
use crate::extmark::{extmark_clear, extmark_del_id, extmark_from_id, extmark_get, extmark_set};
use crate::grid::schar_high;
use crate::marktree::key::{
    MtFlags, mt_decor, mt_invalid, mt_invalidate, mt_no_undo, mt_paired, mt_right,
};
use crate::marktree::mt_inspect;
use crate::mbyte::c_cluster_schar;
use crate::r#move::changed_window_setting;
use crate::os::libc::{int32_t, size_t, uint8_t, uint32_t};
use crate::pos::{MAXCOL, MAXLNUM};
use crate::sign::init_sign_text;
use crate::types::{
    ApiDict, ApiString, Array, Boolean, BufferHandle, ColNr, DecorExt, DecorHighlightInline,
    DecorInline, DecorPriority, DecorSignHighlight, DecorVirtText, DecorVirtText_data, Error,
    ExtmarkType, Integer, LineNr, LuaRef, MTKey, MTPair, NS, Object, VirtLine, VirtText,
    VirtTextChunk, Window, kObjectTypeArray,
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
pub const kExtmarkHighlight: ExtmarkType = 32;
pub const kExtmarkVirtLines: ExtmarkType = 16;
pub const kExtmarkVirtText: ExtmarkType = 8;
pub const kExtmarkSign: ExtmarkType = 2;
/// No allocated decoration: the index a `DecorSignHighlight` chain ends on.
pub const DECOR_ID_INVALID: uint32_t = u32::MAX;
pub(crate) use crate::decoration::{
    DECOR_HIGHLIGHT_INLINE_INIT, DECOR_PRIORITY_BASE, DECOR_SIGN_HIGHLIGHT_INIT, kSHConceal,
    kSHConcealLines, kSHHlEol, kSHIsSign, kSHSpellOff, kSHSpellOn, kSHUIWatched,
    kSHUIWatchedOverlay,
};
