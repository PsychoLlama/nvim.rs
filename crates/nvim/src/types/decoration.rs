#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// c2rust's names for the decoration structs' anonymous members.
#![allow(non_camel_case_types)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;
use crate::winlayer::WinId;

/// The two chains an `ext` decoration names: indices of the first
/// sign/highlight item and of the first virtual-text link in the
/// decoration stores, either `DECOR_ID_INVALID` for none.
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct DecorExt {
    pub sh_idx: uint32_t,
    pub vt_idx: uint32_t,
}
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct DecorHighlightInline {
    pub flags: uint16_t,
    pub priority: DecorPriority,
    pub hl_id: ::core::ffi::c_int,
    pub conceal_char: ScreenChar,
}
/// A mark's decoration: a whole highlight held inline, or the chains of an
/// `ext` decoration in the decoration stores.
///
/// `Copy` stays. Both arms are *values*: the inline arm owns nothing, and
/// the `ext` arm names store entries the store owns, acquired through
/// `decor_put_sh`/`decor_put_vt` and released through `decor_free`.
/// Copying one is a second name for the same entries, not a second owner of
/// them -- which is why both keys of a paired mark carry the decoration and
/// only the start key's removal frees it.
///
/// `repr(C, u32)` so that `MTKey`, which stores one, keeps a C layout the
/// unit suite's declarations can spell.
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C, u32)]
pub enum DecorInline {
    Hl(DecorHighlightInline),
    Ext(DecorExt),
}

impl DecorInline {
    /// Whether this is the `ext` arm, which is what the mark's
    /// `MtFlags::DECOR_EXT` says too.
    #[inline(always)]
    pub fn is_ext(self) -> bool {
        matches!(self, DecorInline::Ext(_))
    }
}
pub type DecorPriority = uint16_t;
pub type DecorPriorityInternal = uint32_t;
#[derive(Copy, Clone)]
pub struct DecorProvider {
    pub ns_id: NS,
    pub state: DecorProvider_state,
    pub win_skip_row: ::core::ffi::c_int,
    pub win_skip_col: ::core::ffi::c_int,
    pub redraw_start: LuaRef,
    pub redraw_buf: LuaRef,
    pub redraw_win: LuaRef,
    pub redraw_line: LuaRef,
    pub redraw_range: LuaRef,
    pub redraw_end: LuaRef,
    pub hl_def: LuaRef,
    pub spell_nav: LuaRef,
    pub conceal_line: LuaRef,
    pub hl_valid: ::core::ffi::c_int,
    pub hl_cached: bool,
    pub error_count: uint8_t,
}
pub type DecorProvider_state = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorRange {
    pub start_row: ::core::ffi::c_int,
    pub start_col: ::core::ffi::c_int,
    pub end_row: ::core::ffi::c_int,
    pub end_col: ::core::ffi::c_int,
    pub ordering: ::core::ffi::c_int,
    pub priority_internal: DecorPriorityInternal,
    pub owned: bool,
    pub kind: DecorRangeKind,
    pub data: DecorRange_data,
    pub attr_id: ::core::ffi::c_int,
    pub draw_col: ::core::ffi::c_int,
}
pub type DecorRangeKind = uint8_t;
/// How a virtual text's highlight combines with what is under it.
pub type HlMode = ::core::ffi::c_uint;
/// One slot of the `DecorRange` slab: either a range, or a link in the
/// freelist threaded through the slab itself.
#[derive(Copy, Clone)]
pub enum DecorRangeSlot {
    /// An occupied slot; one of the two sorted index lists names it.
    Range(DecorRange),
    /// A free slot, holding the index of the next free one, or −1.
    Free(::core::ffi::c_int),
}
/// What a `DecorRange` draws. Its `kind` says the same thing and more --
/// it tells inline virtual text from virtual lines, which share a link.
#[derive(Copy, Clone)]
pub enum DecorRange_data {
    /// A highlight, a conceal, a spell override or a URL.
    Highlight(RangeHighlight),
    /// A virtual-text link, inline or lines by its own flags: an index into
    /// the virtual-text store.
    Virt(uint32_t),
    /// A position reported to the UI rather than drawn.
    UIWatched(DecorRange_data_ui),
}

/// What a highlight range needs of the item it came from, which is not all
/// of it: the item's own strings stay in the store, and `url` names the
/// store entry holding the URL (`DECOR_ID_INVALID` for none).
#[derive(Copy, Clone)]
pub struct RangeHighlight {
    pub flags: uint16_t,
    pub conceal_char: ScreenChar,
    pub url: uint32_t,
}

impl DecorRange_data {
    /// The highlight item, for a highlight range.
    pub fn highlight(&self) -> RangeHighlight {
        match self {
            DecorRange_data::Highlight(sh) => *sh,
            _ => unreachable!("decor: this range draws no highlight"),
        }
    }

    /// The virtual-text link, for a virtual-text or virtual-lines range.
    pub fn virt(&self) -> uint32_t {
        match self {
            DecorRange_data::Virt(vt) => *vt,
            _ => unreachable!("decor: this range draws no virtual text"),
        }
    }

    /// The reported position, for a UI-watched range.
    pub fn ui_watched(&self) -> DecorRange_data_ui {
        match self {
            DecorRange_data::UIWatched(ui) => *ui,
            _ => unreachable!("decor: this range is not UI-watched"),
        }
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorRange_data_ui {
    pub ns_id: uint32_t,
    pub mark_id: uint32_t,
    pub pos: VirtTextPos,
}
/// One sign/highlight item of the decoration store.
///
/// Not `Copy`: the definition name a `:sign` placement was copied from and
/// a `url` are the item's own strings, dropped when its slot is freed.
#[derive(Clone)]
pub struct DecorSignHighlight {
    pub flags: uint16_t,
    pub priority: DecorPriority,
    pub hl_id: ::core::ffi::c_int,
    pub text: [ScreenChar; 2],
    pub sign_name: Option<::std::ffi::CString>,
    pub sign_add_id: ::core::ffi::c_int,
    pub number_hl_id: ::core::ffi::c_int,
    pub line_hl_id: ::core::ffi::c_int,
    pub cursorline_hl_id: ::core::ffi::c_int,
    pub next: uint32_t,
    pub url: Option<::std::ffi::CString>,
}
/// Not `Copy`: the two vectors below own their storage. `spell/navigate.rs`
/// puts the state aside for the length of a scan, which is a `mem::take`
/// rather than the struct copy upstream does.
#[derive(Default)]
pub struct DecorState {
    pub itr: [MarkTreeIter; 1],
    /// The `DecorRange` slab, with a freelist through `next_free_i`.
    pub slots: Vec<DecorRangeSlot>,
    /// Two sorted index lists over `slots`; see `decoration::state`.
    pub ranges_i: Vec<::core::ffi::c_int>,
    pub current_end: ::core::ffi::c_int,
    pub future_begin: ::core::ffi::c_int,
    pub free_slot_i: ::core::ffi::c_int,
    pub new_range_ordering: ::core::ffi::c_int,
    /// The window being drawn, between `decor_redraw_reset` and the end of
    /// the redraw. An id: a decoration provider's Lua runs in between.
    pub(crate) win: Option<WinId>,
    pub top_row: ::core::ffi::c_int,
    pub row: ::core::ffi::c_int,
    pub col_last: ::core::ffi::c_int,
    pub current: ::core::ffi::c_int,
    pub eol_col: ::core::ffi::c_int,
    pub conceal: ::core::ffi::c_int,
    pub conceal_char: ScreenChar,
    pub conceal_attr: ::core::ffi::c_int,
    pub spell: Option<bool>,
    pub itr_valid: bool,
}
/// One link of a decoration's virtual-text chain, in the virtual-text store.
/// `next` is the index of the next link, `DECOR_ID_INVALID` at the end.
#[derive(Clone)]
pub struct DecorVirtText {
    pub flags: uint8_t,
    pub hl_mode: uint8_t,
    pub priority: DecorPriority,
    pub width: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
    pub pos: VirtTextPos,
    pub data: DecorVirtText_data,
    pub next: uint32_t,
}
/// What one link of a virtual-text chain carries: inline chunks, or whole
/// virtual lines. `kVTIsLines` in the link's `flags` says which, and the
/// two are always set together.
#[derive(Clone)]
pub enum DecorVirtText_data {
    Text(VirtText),
    Lines(VirtLines),
}

impl DecorVirtText_data {
    /// The inline chunks, for a link that carries them.
    pub fn text(&self) -> &VirtText {
        match self {
            DecorVirtText_data::Text(text) => text,
            DecorVirtText_data::Lines(_) => {
                unreachable!("decor: a virtual-lines link has no inline text")
            }
        }
    }

    /// [`Self::text`], to write to -- the code that fills a link in.
    pub fn text_mut(&mut self) -> &mut VirtText {
        match self {
            DecorVirtText_data::Text(text) => text,
            DecorVirtText_data::Lines(_) => {
                unreachable!("decor: a virtual-lines link has no inline text")
            }
        }
    }

    /// The block of virtual lines, for a link that carries one.
    pub fn lines(&self) -> &VirtLines {
        match self {
            DecorVirtText_data::Lines(lines) => lines,
            DecorVirtText_data::Text(_) => {
                unreachable!("decor: an inline virtual-text link has no lines")
            }
        }
    }

    /// [`Self::lines`], to write to.
    pub fn lines_mut(&mut self) -> &mut VirtLines {
        match self {
            DecorVirtText_data::Lines(lines) => lines,
            DecorVirtText_data::Text(_) => {
                unreachable!("decor: an inline virtual-text link has no lines")
            }
        }
    }
}
/// One chunk of a virtual text: its text, already made printable, and the
/// highlight group it is drawn in (-1 for none). A chunk with no text only
/// stacks its highlight onto the next one.
#[derive(Clone)]
pub struct VirtTextChunk {
    pub text: Option<::std::ffi::CString>,
    pub hl_id: ::core::ffi::c_int,
}
pub type VirtTextPos = ::core::ffi::c_uint;
