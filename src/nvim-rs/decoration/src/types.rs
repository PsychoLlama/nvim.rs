//! Rust mirror of C struct layouts for direct field access via repr(C).
//!
//! These definitions must match the C structs exactly. Static size/offset
//! assertions are included to catch layout mismatches at compile time.
//!
//! Layout verified against decoration.h, decoration_defs.h, marktree_defs.h.
//!
//! Sizes (x86_64 linux, confirmed by offsetof checks):
//!   MarkTreeIter = 216 bytes, align 8
//!   DecorState   = 328 bytes
//!   DecorRange   = 96 bytes
//!   DecorSignHighlight = 56 bytes
//!   DecorVirtText = 48 bytes

#![allow(unsafe_code)]
#![allow(dead_code)]

use std::ffi::{c_char, c_int, c_void};

extern crate libc;

// =============================================================================
// KVec -- matches kvec_t(T) macro: { size_t size; size_t capacity; T *items; }
// =============================================================================

/// Rust mirror of kvec_t(T).
///
/// # Safety
/// Layout must match `struct { size_t size; size_t capacity; T *items; }`.
#[repr(C)]
pub struct KVec<T> {
    pub size: usize,
    pub capacity: usize,
    pub items: *mut T,
}

impl<T> Default for KVec<T> {
    fn default() -> Self {
        Self {
            size: 0,
            capacity: 0,
            items: std::ptr::null_mut(),
        }
    }
}

impl<T> KVec<T> {
    /// Get the number of items.
    pub fn len(&self) -> usize {
        self.size
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Get a pointer to item at index (unchecked).
    ///
    /// # Safety
    /// `idx` must be in `[0, self.size)` and `items` must be valid.
    pub unsafe fn get_unchecked(&self, idx: usize) -> *mut T {
        self.items.add(idx)
    }

    /// Get a reference to item at index.
    ///
    /// # Safety
    /// `idx` must be in `[0, self.size)` and `items` must be valid.
    pub unsafe fn get(&self, idx: usize) -> &T {
        &*self.items.add(idx)
    }

    /// Get a mutable reference to item at index.
    ///
    /// # Safety
    /// `idx` must be in `[0, self.size)` and `items` must be valid.
    pub unsafe fn get_mut(&mut self, idx: usize) -> &mut T {
        &mut *self.items.add(idx)
    }

    /// Get a slice view of the items.
    ///
    /// # Safety
    /// `items` must be valid for `size` elements.
    pub unsafe fn as_slice(&self) -> &[T] {
        if self.items.is_null() || self.size == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(self.items, self.size)
        }
    }

    /// Push a new element, growing the buffer if necessary (like `kv_pushp`).
    ///
    /// Uses `libc::realloc` to grow the buffer (matching C's `xrealloc` on
    /// standard malloc platforms). Capacity doubles when exhausted, starting at 8.
    ///
    /// # Panics
    /// Panics if `libc::realloc` returns null (out of memory).
    ///
    /// # Safety
    /// The `items` pointer must be from `libc::malloc`/`libc::realloc` or null.
    /// The element `val` must be valid to write.
    pub unsafe fn push(&mut self, val: T) {
        if self.size == self.capacity {
            let new_cap = if self.capacity == 0 {
                8
            } else {
                self.capacity * 2
            };
            let new_size = new_cap * std::mem::size_of::<T>();
            let new_ptr = libc::realloc(self.items.cast::<libc::c_void>(), new_size);
            assert!(!new_ptr.is_null(), "KVec::push: realloc failed");
            self.items = new_ptr.cast::<T>();
            self.capacity = new_cap;
        }
        self.items.add(self.size).write(val);
        self.size += 1;
    }
}

// =============================================================================
// MarkTreeIter -- opaque, only size and alignment matter
// =============================================================================

/// Opaque mirror of MarkTreeIter. Size = 216, align = 8.
///
/// We never interpret the fields; we only need the correct size
/// so that `DecorState.itr[1]` has the right size in the Rust layout.
#[repr(C, align(8))]
pub struct MarkTreeIter {
    _data: [u8; 216],
}

impl MarkTreeIter {
    /// Create a zeroed MarkTreeIter (equivalent to `= {0}` in C).
    pub fn new() -> Self {
        Self { _data: [0u8; 216] }
    }
}

impl Default for MarkTreeIter {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// VirtTextChunk -- matches struct { char *text; int hl_id; }
// =============================================================================

/// Rust mirror of VirtTextChunk.
#[repr(C)]
pub struct VirtTextChunk {
    pub text: *mut c_char,
    pub hl_id: c_int,
}

// =============================================================================
// VirtText / VirtLines -- kvec_t of chunks / lines
// =============================================================================

/// Rust mirror of VirtText (kvec_t(VirtTextChunk)).
pub type VirtText = KVec<VirtTextChunk>;

/// A single virt_line entry: struct { VirtText line; int flags; }
#[repr(C)]
pub struct VirtLine {
    pub line: VirtText,
    pub flags: c_int,
}

/// Rust mirror of VirtLines (kvec_t(virt_line)).
pub type VirtLines = KVec<VirtLine>;

// =============================================================================
// VirtTextPos -- matches VirtTextPos enum
// =============================================================================

/// Rust mirror of VirtTextPos enum (must match C values exactly).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtTextPosC {
    EndOfLine = 0,
    EndOfLineRightAlign = 1,
    Inline = 2,
    Overlay = 3,
    RightAlign = 4,
    WinCol = 5,
}

// =============================================================================
// DecorVirtText -- matches struct DecorVirtText
//
// Offsets verified:
//   flags=0, hl_mode=1, priority=2, width=4, col=8, pos=12, data=16, next=40
// =============================================================================

/// Union data field inside DecorVirtText.
/// Either virt_text or virt_lines (same size since both are KVec of 3 pointers = 24 bytes).
#[repr(C)]
pub union DecorVirtTextData {
    pub virt_text: std::mem::ManuallyDrop<VirtText>,
    pub virt_lines: std::mem::ManuallyDrop<VirtLines>,
}

/// Rust mirror of struct DecorVirtText. Size = 48 bytes.
#[repr(C)]
pub struct DecorVirtText {
    pub flags: u8,
    pub hl_mode: u8,
    pub priority: u16,
    pub width: c_int,
    pub col: c_int,
    pub pos: c_int, // VirtTextPos as i32
    pub data: DecorVirtTextData,
    pub next: *mut DecorVirtText,
}

// =============================================================================
// DecorSignHighlight -- matches struct DecorSignHighlight
//
// Offsets verified:
//   flags=0, priority=2, hl_id=4, text=8, sign_name=16, sign_add_id=24,
//   number_hl_id=28, line_hl_id=32, cursorline_hl_id=36, next=40, url=48
// Size = 56 bytes
// =============================================================================

/// Rust mirror of DecorSignHighlight. Size = 56 bytes.
#[repr(C)]
pub struct DecorSignHighlight {
    pub flags: u16,
    pub priority: u16,
    pub hl_id: c_int,
    /// text[SIGN_WIDTH] -- SIGN_WIDTH = 2 x u32 (schar_T is u32)
    pub text: [u32; 2],
    pub sign_name: *mut c_char,
    pub sign_add_id: c_int,
    pub number_hl_id: c_int,
    pub line_hl_id: c_int,
    pub cursorline_hl_id: c_int,
    pub next: u32,
    #[allow(clippy::pub_underscore_fields)]
    pub _pad_next: u32, // padding between next(u32) and url(*ptr) at offset 48
    pub url: *const c_char,
}

// Wait -- let's recheck: next=40, url=48.
// next is u32 at offset 40. url is *const char at offset 48 (pointer = 8 bytes, aligned to 8).
// So after next (4 bytes at offset 40), we need 4 bytes padding to reach offset 48.
// Above struct should handle this: next:u32 + _pad_next:u32 = 8 bytes from 40..48, then url.

// =============================================================================
// DecorRange union data field
// =============================================================================

/// UI watched data within a DecorRange.
#[repr(C)]
pub struct DecorRangeUiData {
    pub ns_id: u32,
    pub mark_id: u32,
    pub pos: c_int, // VirtTextPos as int
}

/// DecorRange.data union.
/// Size = sizeof(DecorSignHighlight) = 56 bytes.
#[repr(C)]
pub union DecorRangeData {
    pub sh: std::mem::ManuallyDrop<DecorSignHighlight>,
    pub vt: *mut DecorVirtText,
    pub ui: std::mem::ManuallyDrop<DecorRangeUiData>,
}

// =============================================================================
// DecorRange -- matches struct DecorRange
//
// Offsets verified:
//   start_row=0, start_col=4, end_row=8, end_col=12,
//   ordering=16, priority_internal=20, owned=24, kind=25,
//   data=32 (aligned to 8 for pointer in union), attr_id=88, draw_col=92
// Size = 96 bytes
// =============================================================================

/// Rust mirror of DecorRange. Size = 96 bytes.
#[repr(C)]
pub struct DecorRange {
    pub start_row: c_int,
    pub start_col: c_int,
    pub end_row: c_int,
    pub end_col: c_int,
    pub ordering: c_int,
    pub priority_internal: u32,
    pub owned: bool,
    pub kind: u8, // DecorRangeKind
    /// 6 bytes padding to align data to offset 32
    #[allow(clippy::pub_underscore_fields)]
    pub _pad: [u8; 6],
    pub data: DecorRangeData,
    pub attr_id: c_int,
    pub draw_col: c_int,
}

// =============================================================================
// DecorRangeSlot -- matches union { DecorRange range; int next_free_i; }
// =============================================================================

/// Rust mirror of DecorRangeSlot. Size = 96 bytes.
#[repr(C)]
pub union DecorRangeSlot {
    pub range: std::mem::ManuallyDrop<DecorRange>,
    pub next_free_i: c_int,
}

// =============================================================================
// DecorState -- matches struct DecorState
//
// Offsets verified:
//   itr=0(216), slots=216(24), ranges_i=240(24),
//   current_end=264, future_begin=268, free_slot_i=272, new_range_ordering=276,
//   win=280(8), top_row=288, row=292, col_until=296, current=300, eol_col=304,
//   conceal=308, conceal_char=312, conceal_attr=316, spell=320,
//   running_decor_provider=324, itr_valid=325
// Size = 328 bytes
// =============================================================================

/// Opaque handle to win_T.
pub type WinT = c_void;

/// Rust mirror of DecorState. Size = 328 bytes.
#[repr(C)]
pub struct DecorState {
    /// itr[1] -- MarkTreeIter inline array, size = 216
    pub itr: MarkTreeIter,
    /// slots kvec -- size 24
    pub slots: KVec<DecorRangeSlot>,
    /// ranges_i kvec -- size 24
    pub ranges_i: KVec<c_int>,
    pub current_end: c_int,
    pub future_begin: c_int,
    pub free_slot_i: c_int,
    pub new_range_ordering: c_int,
    pub win: *mut WinT,
    pub top_row: c_int,
    pub row: c_int,
    pub col_until: c_int,
    pub current: c_int,
    pub eol_col: c_int,
    pub conceal: c_int,
    pub conceal_char: u32, // schar_T
    pub conceal_attr: c_int,
    pub spell: c_int, // TriState
    pub running_decor_provider: bool,
    pub itr_valid: bool,
}

// =============================================================================
// DecorHighlightInline -- matches struct { uint16_t flags; uint16_t priority; int hl_id; schar_T conceal_char; }
// =============================================================================

/// Rust mirror of DecorHighlightInline. Size = 12 bytes.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DecorHighlightInline {
    pub flags: u16,
    pub priority: u16,
    pub hl_id: c_int,
    pub conceal_char: u32,
}

// =============================================================================
// DecorExt -- matches struct { uint32_t sh_idx; DecorVirtText *vt; }
// Offsets: sh_idx=0, vt=8 (pointer aligned). Size = 16 bytes.
// =============================================================================

/// Rust mirror of DecorExt. Size = 16 bytes.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DecorExt {
    pub sh_idx: u32,
    #[allow(clippy::pub_underscore_fields)]
    pub _pad: u32,
    pub vt: *mut DecorVirtText,
}

// =============================================================================
// DecorInlineData -- union { DecorHighlightInline hl; DecorExt ext; }
// Size = 16 bytes.
// =============================================================================

/// Rust mirror of DecorInlineData union. Size = 16 bytes.
#[repr(C)]
pub union DecorInlineData {
    pub hl: std::mem::ManuallyDrop<DecorHighlightInline>,
    pub ext: std::mem::ManuallyDrop<DecorExt>,
}

// Safety: all fields are #[repr(C)] with only Copy primitives or raw pointers.
#[allow(clippy::non_canonical_clone_impl)]
impl Clone for DecorInlineData {
    fn clone(&self) -> Self {
        // Safe: we copy the raw bytes of the union.
        // SAFETY: DecorInlineData is a union of 16 bytes; copying it bitwise is correct.
        unsafe { std::ptr::read(self) }
    }
}

impl Copy for DecorInlineData {}

impl DecorInlineData {
    pub const fn zero() -> Self {
        Self {
            hl: std::mem::ManuallyDrop::new(DecorHighlightInline {
                flags: 0,
                priority: 0,
                hl_id: 0,
                conceal_char: 0,
            }),
        }
    }
}

// =============================================================================
// DecorInline -- struct { bool ext; [7 bytes pad]; DecorInlineData data; }
// Offsets: ext=0, data=8. Size = 24 bytes.
// =============================================================================

/// Rust mirror of DecorInline. Size = 24 bytes.
#[repr(C)]
pub struct DecorInline {
    pub ext: bool,
    #[allow(clippy::pub_underscore_fields)]
    pub _pad: [u8; 7],
    pub data: DecorInlineData,
}

// =============================================================================
// Static size/offset assertions
// =============================================================================

const _: () = {
    assert!(std::mem::size_of::<DecorHighlightInline>() == 12);
    assert!(std::mem::size_of::<DecorExt>() == 16);
    assert!(std::mem::offset_of!(DecorExt, sh_idx) == 0);
    assert!(std::mem::offset_of!(DecorExt, vt) == 8);
    assert!(std::mem::size_of::<DecorInlineData>() == 16);
    assert!(std::mem::size_of::<DecorInline>() == 24);
    assert!(std::mem::offset_of!(DecorInline, ext) == 0);
    assert!(std::mem::offset_of!(DecorInline, data) == 8);
    assert!(std::mem::size_of::<MarkTreeIter>() == 216);
    assert!(std::mem::align_of::<MarkTreeIter>() == 8);
    assert!(std::mem::size_of::<DecorSignHighlight>() == 56);
    assert!(std::mem::size_of::<DecorVirtText>() == 48);
    assert!(std::mem::size_of::<DecorRange>() == 96);
    assert!(std::mem::size_of::<DecorRangeSlot>() == 96);
    assert!(std::mem::size_of::<DecorState>() == 328);

    // DecorState field offsets
    assert!(std::mem::offset_of!(DecorState, itr) == 0);
    assert!(std::mem::offset_of!(DecorState, slots) == 216);
    assert!(std::mem::offset_of!(DecorState, ranges_i) == 240);
    assert!(std::mem::offset_of!(DecorState, current_end) == 264);
    assert!(std::mem::offset_of!(DecorState, future_begin) == 268);
    assert!(std::mem::offset_of!(DecorState, free_slot_i) == 272);
    assert!(std::mem::offset_of!(DecorState, new_range_ordering) == 276);
    assert!(std::mem::offset_of!(DecorState, win) == 280);
    assert!(std::mem::offset_of!(DecorState, top_row) == 288);
    assert!(std::mem::offset_of!(DecorState, row) == 292);
    assert!(std::mem::offset_of!(DecorState, col_until) == 296);
    assert!(std::mem::offset_of!(DecorState, current) == 300);
    assert!(std::mem::offset_of!(DecorState, eol_col) == 304);
    assert!(std::mem::offset_of!(DecorState, conceal) == 308);
    assert!(std::mem::offset_of!(DecorState, conceal_char) == 312);
    assert!(std::mem::offset_of!(DecorState, conceal_attr) == 316);
    assert!(std::mem::offset_of!(DecorState, spell) == 320);
    assert!(std::mem::offset_of!(DecorState, running_decor_provider) == 324);
    assert!(std::mem::offset_of!(DecorState, itr_valid) == 325);

    // DecorRange field offsets
    assert!(std::mem::offset_of!(DecorRange, start_row) == 0);
    assert!(std::mem::offset_of!(DecorRange, start_col) == 4);
    assert!(std::mem::offset_of!(DecorRange, end_row) == 8);
    assert!(std::mem::offset_of!(DecorRange, end_col) == 12);
    assert!(std::mem::offset_of!(DecorRange, ordering) == 16);
    assert!(std::mem::offset_of!(DecorRange, priority_internal) == 20);
    assert!(std::mem::offset_of!(DecorRange, owned) == 24);
    assert!(std::mem::offset_of!(DecorRange, kind) == 25);
    assert!(std::mem::offset_of!(DecorRange, data) == 32);
    assert!(std::mem::offset_of!(DecorRange, attr_id) == 88);
    assert!(std::mem::offset_of!(DecorRange, draw_col) == 92);

    // DecorSignHighlight field offsets
    assert!(std::mem::offset_of!(DecorSignHighlight, flags) == 0);
    assert!(std::mem::offset_of!(DecorSignHighlight, priority) == 2);
    assert!(std::mem::offset_of!(DecorSignHighlight, hl_id) == 4);
    assert!(std::mem::offset_of!(DecorSignHighlight, text) == 8);
    assert!(std::mem::offset_of!(DecorSignHighlight, sign_name) == 16);
    assert!(std::mem::offset_of!(DecorSignHighlight, sign_add_id) == 24);
    assert!(std::mem::offset_of!(DecorSignHighlight, number_hl_id) == 28);
    assert!(std::mem::offset_of!(DecorSignHighlight, line_hl_id) == 32);
    assert!(std::mem::offset_of!(DecorSignHighlight, cursorline_hl_id) == 36);
    assert!(std::mem::offset_of!(DecorSignHighlight, next) == 40);
    assert!(std::mem::offset_of!(DecorSignHighlight, url) == 48);

    // DecorVirtText field offsets
    assert!(std::mem::offset_of!(DecorVirtText, flags) == 0);
    assert!(std::mem::offset_of!(DecorVirtText, hl_mode) == 1);
    assert!(std::mem::offset_of!(DecorVirtText, priority) == 2);
    assert!(std::mem::offset_of!(DecorVirtText, width) == 4);
    assert!(std::mem::offset_of!(DecorVirtText, col) == 8);
    assert!(std::mem::offset_of!(DecorVirtText, pos) == 12);
    assert!(std::mem::offset_of!(DecorVirtText, data) == 16);
    assert!(std::mem::offset_of!(DecorVirtText, next) == 40);
};

// =============================================================================
// MTPos -- matches struct { int32_t row; int32_t col; }
// =============================================================================

/// Rust mirror of MTPos. Size = 8 bytes.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MTPos {
    pub row: i32,
    pub col: i32,
}

// =============================================================================
// MTKey -- matches struct MTKey
//
// Layout:
//   pos=0(8), ns=8(4), id=12(4), flags=16(2), pad=18(6), decor_data=24(16)
// Size = 40 bytes, align = 8
// =============================================================================

/// Rust mirror of MTKey. Size = 40 bytes.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MTKey {
    pub pos: MTPos,
    pub ns: u32,
    pub id: u32,
    pub flags: u16,
    #[allow(clippy::pub_underscore_fields)]
    pub _pad: [u8; 6],
    pub decor_data: DecorInlineData,
}

impl Default for MTKey {
    fn default() -> Self {
        Self {
            pos: MTPos { row: -1, col: -1 },
            ns: 0,
            id: 0,
            flags: 0,
            _pad: [0; 6],
            decor_data: DecorInlineData::zero(),
        }
    }
}

// =============================================================================
// MTPair -- matches struct { MTKey start; MTPos end_pos; bool end_right_gravity; }
//
// Layout: start=0(40), end_pos=40(8), end_right_gravity=48(1), pad=49(7)
// Size = 56 bytes
// =============================================================================

/// Rust mirror of MTPair. Size = 56 bytes.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MTPair {
    pub start: MTKey,
    pub end_pos: MTPos,
    pub end_right_gravity: bool,
    #[allow(clippy::pub_underscore_fields)]
    pub _pad: [u8; 7],
}

#[allow(clippy::derivable_impls)]
impl Default for MTPair {
    fn default() -> Self {
        Self {
            start: MTKey::default(),
            end_pos: MTPos::default(),
            end_right_gravity: false,
            _pad: [0; 7],
        }
    }
}

// Static size/offset assertions for marktree types
const _: () = {
    assert!(std::mem::size_of::<MTPos>() == 8);
    assert!(std::mem::size_of::<MTKey>() == 40);
    assert!(std::mem::offset_of!(MTKey, pos) == 0);
    assert!(std::mem::offset_of!(MTKey, ns) == 8);
    assert!(std::mem::offset_of!(MTKey, id) == 12);
    assert!(std::mem::offset_of!(MTKey, flags) == 16);
    assert!(std::mem::offset_of!(MTKey, decor_data) == 24);
    assert!(std::mem::size_of::<MTPair>() == 56);
    assert!(std::mem::offset_of!(MTPair, start) == 0);
    assert!(std::mem::offset_of!(MTPair, end_pos) == 40);
    assert!(std::mem::offset_of!(MTPair, end_right_gravity) == 48);
};
