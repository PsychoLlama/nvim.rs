//! Tree-sitter C ABI types.
//!
//! These must match the layout in `<tree_sitter/api.h>` exactly.
//! All types are `#[repr(C)]` to ensure ABI compatibility.

use std::ffi::c_void;

/// Lua state opaque handle (matching nvim-lua's LuaState)
#[repr(C)]
pub struct LuaState {
    _opaque: [u8; 0],
}

/// Tree-sitter node (24 bytes, must match TSNode in api.h exactly)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TSNode {
    pub context: [u32; 4],
    pub id: *const c_void,
    pub tree: *const c_void,
}

// TSNode is Copy-safe: it's a value type in the tree-sitter C API.
// SAFETY: tree-sitter nodes are pointer-stable across calls; they do not
// contain any thread-local or !Send data beyond raw pointers we never
// dereference concurrently.
unsafe impl Send for TSNode {}
unsafe impl Sync for TSNode {}

/// Tree-sitter point (row, column)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TSPoint {
    pub row: u32,
    pub column: u32,
}

/// Tree-sitter range
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TSRange {
    pub start_point: TSPoint,
    pub end_point: TSPoint,
    pub start_byte: u32,
    pub end_byte: u32,
}

/// Tree-sitter input edit
#[repr(C)]
pub struct TSInputEdit {
    pub start_byte: u32,
    pub old_end_byte: u32,
    pub new_end_byte: u32,
    pub start_point: TSPoint,
    pub old_end_point: TSPoint,
    pub new_end_point: TSPoint,
}

/// Tree-sitter query capture
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TSQueryCapture {
    pub node: TSNode,
    pub index: u32,
}

/// Tree-sitter query match
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TSQueryMatch {
    pub id: u32,
    pub pattern_index: u16,
    pub capture_count: u16,
    pub captures: *const TSQueryCapture,
}

/// Tree-sitter query predicate step type
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TSQueryPredicateStepType {
    Done = 0,
    Capture = 1,
    String = 2,
}

/// Tree-sitter query predicate step
#[repr(C)]
pub struct TSQueryPredicateStep {
    pub type_: TSQueryPredicateStepType,
    pub value_id: u32,
}

/// TSLuaTree userdata wrapper (must match C-side struct exactly)
#[repr(C)]
pub struct TSLuaTree {
    pub tree: *const c_void, // const TSTree *
}

/// Type aliases matching tree-sitter typedefs
pub type TSSymbol = u16;
pub type TSFieldId = u16;

/// Lua registry index (LuaJIT/Lua 5.1)
pub const LUA_REGISTRYINDEX: i32 = -10000;
/// Lua globals index
pub const LUA_GLOBALSINDEX: i32 = -10002;
/// LuaJIT type nil constant
pub const LUA_TNIL: i32 = 0;
/// LuaJIT type string constant
pub const LUA_TSTRING: i32 = 4;
/// LuaJIT type table constant
pub const LUA_TTABLE: i32 = 5;

/// Metatable name constants (must match C-side TS_META_* defines)
pub const TS_META_NODE: &std::ffi::CStr = c"treesitter_node";
pub const TS_META_TREE: &std::ffi::CStr = c"treesitter_tree";
pub const TS_META_QUERY: &std::ffi::CStr = c"treesitter_query";
pub const TS_META_QUERYCURSOR: &std::ffi::CStr = c"treesitter_querycursor";
pub const TS_META_QUERYMATCH: &std::ffi::CStr = c"treesitter_querymatch";
pub const TS_META_PARSER: &std::ffi::CStr = c"treesitter_parser";

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, offset_of, size_of};

    #[test]
    fn tsnode_layout() {
        // TSNode: uint32_t[4] + void* + TSTree* = 16 + 4*2 = 24 on 64-bit
        // On 64-bit: 4*4=16 + 8 + 8 = 32
        assert_eq!(size_of::<TSNode>(), 32);
        assert_eq!(align_of::<TSNode>(), 8);
    }

    #[test]
    fn tspoint_layout() {
        assert_eq!(size_of::<TSPoint>(), 8);
        assert_eq!(align_of::<TSPoint>(), 4);
    }

    #[test]
    fn tsrange_layout() {
        // TSPoint + TSPoint + u32 + u32 = 8 + 8 + 4 + 4 = 24
        assert_eq!(size_of::<TSRange>(), 24);
    }

    #[test]
    fn tsquerymatch_layout() {
        // u32 + u16 + u16 + *const = 4 + 2 + 2 + 8 = 16
        assert_eq!(size_of::<TSQueryMatch>(), 16);
        assert_eq!(offset_of!(TSQueryMatch, id), 0);
        assert_eq!(offset_of!(TSQueryMatch, pattern_index), 4);
        assert_eq!(offset_of!(TSQueryMatch, capture_count), 6);
        assert_eq!(offset_of!(TSQueryMatch, captures), 8);
    }
}
