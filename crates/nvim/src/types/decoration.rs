#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorExt {
    pub sh_idx: uint32_t,
    pub vt: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorHighlightInline {
    pub flags: uint16_t,
    pub priority: DecorPriority,
    pub hl_id: ::core::ffi::c_int,
    pub conceal_char: schar_T,
}
/// A mark's decoration: either a whole highlight held inline, or -- when
/// `ext` -- a pair of indices into the process-wide decoration store.
///
/// `Copy` stays. Both arms are *handles*: the inline arm is a plain value,
/// and the indexed arm names entries the store owns, acquired through
/// `decor_put_sh` and released through `decor_free`. Copying one is a second
/// name for the same entries, not a second owner of them -- which is why a
/// range of marks all carry the same decoration and only the mark that
/// outlives the rest frees it.
#[derive(Copy, Clone)]
pub struct DecorInline {
    pub ext: bool,
    pub data: DecorInlineData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union DecorInlineData {
    pub hl: DecorHighlightInline,
    pub ext: DecorExt,
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
    Highlight(DecorSignHighlight),
    /// A virtual-text link, inline or lines by its own flags. May be null.
    Virt(*mut DecorVirtText),
    /// A position reported to the UI rather than drawn.
    UIWatched(DecorRange_data_ui),
}

impl DecorRange_data {
    /// The highlight item, for a highlight range.
    pub fn highlight(&self) -> DecorSignHighlight {
        match self {
            DecorRange_data::Highlight(sh) => *sh,
            _ => unreachable!("decor: this range draws no highlight"),
        }
    }

    /// The virtual-text link, for a virtual-text or virtual-lines range.
    pub fn virt(&self) -> *mut DecorVirtText {
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorSignHighlight {
    pub flags: uint16_t,
    pub priority: DecorPriority,
    pub hl_id: ::core::ffi::c_int,
    pub text: [schar_T; 2],
    pub sign_name: *mut ::core::ffi::c_char,
    pub sign_add_id: ::core::ffi::c_int,
    pub number_hl_id: ::core::ffi::c_int,
    pub line_hl_id: ::core::ffi::c_int,
    pub cursorline_hl_id: ::core::ffi::c_int,
    pub next: uint32_t,
    pub url: *const ::core::ffi::c_char,
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
    pub win: *mut win_T,
    pub top_row: ::core::ffi::c_int,
    pub row: ::core::ffi::c_int,
    pub col_last: ::core::ffi::c_int,
    pub current: ::core::ffi::c_int,
    pub eol_col: ::core::ffi::c_int,
    pub conceal: ::core::ffi::c_int,
    pub conceal_char: schar_T,
    pub conceal_attr: ::core::ffi::c_int,
    pub spell: Option<bool>,
    pub itr_valid: bool,
}
#[derive(Copy, Clone)]
pub struct DecorVirtText {
    pub flags: uint8_t,
    pub hl_mode: uint8_t,
    pub priority: DecorPriority,
    pub width: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
    pub pos: VirtTextPos,
    pub data: DecorVirtText_data,
    pub next: *mut DecorVirtText,
}
/// What one link of a virtual-text chain carries: inline chunks, or whole
/// virtual lines. `kVTIsLines` in the link's `flags` says which, and the
/// two are always set together.
#[derive(Copy, Clone)]
pub enum DecorVirtText_data {
    Text(VirtText),
    Lines(VirtLines),
}

impl DecorVirtText_data {
    /// The inline chunks, for a link that carries them.
    pub fn text(&self) -> VirtText {
        *self.text_ref()
    }

    fn text_ref(&self) -> &VirtText {
        match self {
            DecorVirtText_data::Text(text) => text,
            DecorVirtText_data::Lines(_) => {
                unreachable!("decor: a virtual-lines link has no inline text")
            }
        }
    }

    /// [`Self::text`], to write to -- the code that fills a link in and the
    /// code that frees one.
    pub fn text_mut(&mut self) -> &mut VirtText {
        match self {
            DecorVirtText_data::Text(text) => text,
            DecorVirtText_data::Lines(_) => {
                unreachable!("decor: a virtual-lines link has no inline text")
            }
        }
    }

    /// The block of virtual lines, for a link that carries one.
    pub fn lines(&self) -> VirtLines {
        *self.lines_ref()
    }

    fn lines_ref(&self) -> &VirtLines {
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
#[derive(Copy, Clone)]
pub struct VirtTextChunk {
    pub text: *mut ::core::ffi::c_char,
    pub hl_id: ::core::ffi::c_int,
}
pub type VirtTextPos = ::core::ffi::c_uint;
