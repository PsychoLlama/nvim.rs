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
pub struct ParserHighlightChunk {
    pub start: ParserPosition,
    pub end_col: size_t,
    pub group: *const ::core::ffi::c_char,
}
#[derive(Clone)]
pub struct ParserInputReader {
    pub get_line: ParserLineGetter,
    pub cookie: *mut ::core::ffi::c_void,
    pub lines: ParserInputReader_lines,
    pub conv: vimconv_T,
}
#[derive(Copy, Clone)]
pub struct ParserInputReader_lines {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ParserLine,
    pub init_array: [ParserLine; 4],
}
#[derive(Copy, Clone)]
pub struct ParserLine {
    pub data: *const ::core::ffi::c_char,
    pub size: size_t,
    pub allocated: bool,
}
pub type ParserLineGetter = Option<unsafe fn(*mut ::core::ffi::c_void, *mut ParserLine) -> ()>;
/// A line and column inside a parsed string.
///
/// `Copy`: a position is a value.
#[derive(Copy, Clone)]
pub struct ParserPosition {
    pub line: size_t,
    pub col: size_t,
}
#[derive(Clone)]
pub struct ParserState {
    pub reader: ParserInputReader,
    pub pos: ParserPosition,
    pub stack: ParserState_stack,
    pub colors: *mut ParserHighlight,
    pub can_continuate: bool,
}
#[derive(Copy, Clone)]
pub struct ParserStateItem {
    pub type_0: ParserStateItem_type_0,
    /// Which kind of expression an expression frame is parsing. Upstream
    /// wraps this in a one-armed union of a one-field struct; nothing in
    /// the tree reads either, because the stack is only pushed and popped.
    pub expr_type: ParserStateItem_data_expr_type_0,
}
pub type ParserStateItem_data_expr_type_0 = ::core::ffi::c_uint;
pub type ParserStateItem_type_0 = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
pub struct ParserState_stack {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ParserStateItem,
    pub init_array: [ParserStateItem; 16],
}
