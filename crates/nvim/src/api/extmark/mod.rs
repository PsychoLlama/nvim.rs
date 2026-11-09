#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::api::keysets::{
    KeyDict_get_extmark, KeyDict_get_extmarks, KeyDict_ns_opts, KeyDict_set_decoration_provider,
    KeyDict_set_extmark,
};
use crate::api::private::helpers::{
    api_typename, find_buffer_by_handle, find_window_by_handle, object_to_hl_id,
};
use crate::charset::vim_isprintc;
use crate::decoration::defs::{
    DecorExt, DecorHighlightInline, DecorInline, DecorPriority, DecorSignHighlight, DecorVirtText,
    DecorVirtText_data, VirtTextChunk,
};
use crate::decoration::defs::{VirtLine, VirtText};
use crate::decoration::{
    HL_MODE_BLEND, HL_MODE_COMBINE, HL_MODE_REPLACE, HL_MODE_UNKNOWN, HlSource, V_POS_END_OF_LINE,
    V_POS_END_OF_LINE_RIGHT_ALIGN, V_POS_INLINE, V_POS_OVERLAY, V_POS_RIGHT_ALIGN, V_POS_WIN_COL,
    VL_LEFTCOL, VL_SCROLL, VT_HIDE, VT_IS_LINES, VT_LINES_ABOVE, VT_REPEAT_LINEBREAK, decor_put_sh,
    decor_put_vt, decor_range_add_sh, decor_range_add_virt, decor_sh_from_inline,
    decor_to_dict_legacy, hl_group_name,
};
use crate::decoration_provider::{
    DECOR_PROVIDER_ACTIVE, decor_provider_clear, with_decor_provider,
};
use crate::drawscreen::{UPD_NOT_VALID, redraw_all_later};
use crate::extmark::ExtmarkType;
use crate::extmark::{extmark_clear, extmark_del_id, extmark_from_id, extmark_get};
use crate::grid::schar_high;
use crate::marktree::key::{
    MtFlags, mt_decor, mt_invalid, mt_invalidate, mt_no_undo, mt_paired, mt_right,
};
use crate::marktree::mt_inspect;
use crate::marktree::{MTKey, MTPair};
use crate::mbyte::c_cluster_schar;
use crate::r#move::changed_window_setting;
use crate::os::libc::{int32_t, size_t, uint8_t, uint32_t};
use crate::pos::{MAXCOL, MAXLNUM};
use crate::sign::init_sign_text;
use crate::types::{
    ApiDict, ApiString, Array, Boolean, BufferHandle, ColNr, Error, Integer, LineNr, LuaRef, NS,
    OBJECT_TYPE_ARRAY, Object, Window,
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
pub const EXTMARK_HIGHLIGHT: ExtmarkType = 32;
pub const EXTMARK_VIRT_LINES: ExtmarkType = 16;
pub const EXTMARK_VIRT_TEXT: ExtmarkType = 8;
pub const EXTMARK_SIGN: ExtmarkType = 2;
/// No allocated decoration: the index a `DecorSignHighlight` chain ends on.
pub const DECOR_ID_INVALID: uint32_t = u32::MAX;
pub(crate) use crate::decoration::{
    DECOR_HIGHLIGHT_INLINE_INIT, DECOR_PRIORITY_BASE, DECOR_SIGN_HIGHLIGHT_INIT, SH_CONCEAL,
    SH_CONCEAL_LINES, SH_HL_EOL, SH_IS_SIGN, SH_SPELL_OFF, SH_SPELL_ON, SHUI_WATCHED,
    SHUI_WATCHED_OVERLAY,
};
