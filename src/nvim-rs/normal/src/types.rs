//! `#[repr(C)]` mirrors for Neovim C structs used in normal mode.
//!
//! Layout is verified at compile time via `_Static_assert` in `normal_shim.c`.
//! These types allow direct field access from Rust, eliminating the need for
//! C accessor wrapper functions.

#![allow(dead_code)]

use std::ffi::{c_char, c_int};

// =============================================================================
// PosT -- mirrors pos_T from pos_defs.h
//   { linenr_T lnum; colnr_T col; colnr_T coladd; }
//   linenr_T = i32, colnr_T = int (c_int)
// =============================================================================

/// Mirror of C `pos_T`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PosT {
    pub lnum: i32,
    pub col: c_int,
    pub coladd: c_int,
}

impl PosT {
    /// Return true if self < other (mirrors C `lt(a, b)`).
    #[inline]
    #[must_use]
    pub fn lt(&self, other: &Self) -> bool {
        self.lnum < other.lnum || (self.lnum == other.lnum && self.col < other.col)
    }

    /// Return true if self == other (lnum + col, ignoring coladd).
    #[inline]
    #[must_use]
    pub fn eq_pos(&self, other: &Self) -> bool {
        self.lnum == other.lnum && self.col == other.col
    }
}

// =============================================================================
// OpargT -- mirrors oparg_T from normal_defs.h (17 fields)
//
// typedef struct {
//   int op_type;             // 0
//   int regname;             // 4
//   MotionType motion_type;  // 8   (int-backed enum)
//   int motion_force;        // 12
//   bool use_reg_one;        // 16  (C bool = 1 byte)
//   bool inclusive;          // 17
//   bool end_adjusted;       // 18
//   // padding: 1 byte       // 19
//   pos_T start;             // 20  (12 bytes: i32, c_int, c_int)
//   pos_T end;               // 32
//   pos_T cursor_start;      // 44
//   linenr_T line_count;     // 56  (i32)
//   bool empty;              // 60
//   bool is_VIsual;          // 61
//   // padding: 2 bytes      // 62
//   colnr_T start_vcol;      // 64  (c_int)
//   colnr_T end_vcol;        // 68  (c_int)
//   int prev_opcount;        // 72
//   int prev_count0;         // 76
//   bool excl_tr_ws;         // 80
//   // padding: 3 bytes      // 81..83
// } oparg_T;                 // sizeof = 84
// =============================================================================

/// Mirror of C `oparg_T`.
///
/// Field offsets verified by `_Static_assert` in `normal_shim.c`:
/// - op_type:0, regname:4, motion_type:8, motion_force:12
/// - use_reg_one:16, inclusive:17, end_adjusted:18, [pad:19]
/// - start:20, end:32, cursor_start:44
/// - line_count:56, empty:60, is_VIsual:61, [pad:62-63]
/// - start_vcol:64, end_vcol:68, prev_opcount:72, prev_count0:76
/// - excl_tr_ws:80, [pad:81-83]
///
/// sizeof(oparg_T) = 84
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OpargT {
    pub op_type: c_int,
    pub regname: c_int,
    pub motion_type: c_int, // MotionType enum, backed by int
    pub motion_force: c_int,
    pub use_reg_one: bool,
    pub inclusive: bool,
    pub end_adjusted: bool,
    _pad0: u8,
    pub start: PosT,
    pub end: PosT,
    pub cursor_start: PosT,
    pub line_count: i32,
    pub empty: bool,
    pub is_visual: bool, // C name: is_VIsual
    _pad1: [u8; 2],
    pub start_vcol: c_int,
    pub end_vcol: c_int,
    pub prev_opcount: c_int,
    pub prev_count0: c_int,
    pub excl_tr_ws: bool,
    _pad2: [u8; 3],
}

impl OpargT {
    /// Zero-initialise (mirrors `clear_oparg`).
    #[inline]
    pub fn clear(&mut self) {
        *self = unsafe { std::mem::zeroed() };
    }

    #[inline]
    #[must_use]
    pub fn get_op_type(&self) -> c_int {
        self.op_type
    }

    #[inline]
    #[must_use]
    pub fn is_nop(&self) -> bool {
        self.op_type == OP_NOP
    }
}

impl Default for OpargT {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

// =============================================================================
// CmdargT -- mirrors cmdarg_T from normal_defs.h (12 fields)
//
// typedef struct {
//   oparg_T *oap;             // 0  (pointer, 8 bytes)
//   int prechar;              // 8
//   int cmdchar;              // 12
//   int nchar;                // 16
//   char nchar_composing[32]; // 20  (MAX_SCHAR_SIZE = 32)
//   int nchar_len;            // 52
//   int extra_char;           // 56
//   int opcount;              // 60
//   int count0;               // 64
//   int count1;               // 68
//   int arg;                  // 72
//   int retval;               // 76
//   char *searchbuf;          // 80  (pointer, 8 bytes)
// } cmdarg_T;                 // sizeof = 88
// =============================================================================

/// Mirror of C `cmdarg_T`.
///
/// Field offsets verified by `_Static_assert` in `normal_shim.c`:
/// - oap:0 (ptr, 8 bytes), prechar:8, cmdchar:12, nchar:16
/// - nchar_composing:20 (32 bytes), nchar_len:52, extra_char:56
/// - opcount:60, count0:64, count1:68, arg:72, retval:76
/// - searchbuf:80 (ptr, 8 bytes)
///
/// sizeof(cmdarg_T) = 88
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CmdargT {
    pub oap: *mut OpargT,
    pub prechar: c_int,
    pub cmdchar: c_int,
    pub nchar: c_int,
    pub nchar_composing: [c_char; 32], // MAX_SCHAR_SIZE
    pub nchar_len: c_int,
    pub extra_char: c_int,
    pub opcount: c_int,
    pub count0: c_int,
    pub count1: c_int,
    pub arg: c_int,
    pub retval: c_int,
    pub searchbuf: *mut c_char,
}

impl Default for CmdargT {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

// =============================================================================
// VimState -- mirrors struct vim_state from state_defs.h
//   { state_check_callback check; state_execute_callback execute; }
//   Both are function pointers: fn(*mut VimState) -> c_int
// =============================================================================

/// Mirror of C `VimState` / `struct vim_state`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VimState {
    pub check: Option<unsafe extern "C" fn(*mut VimState) -> c_int>,
    pub execute: Option<unsafe extern "C" fn(*mut VimState, c_int) -> c_int>,
}

// =============================================================================
// NormalState -- mirrors NormalState from normal.c (defined in normal_shim.c)
//
// typedef struct {
//   VimState state;            // 0   (16 bytes: 2 fn ptrs)
//   bool command_finished;     // 16
//   bool ctrl_w;               // 17
//   bool need_flushbuf;        // 18
//   bool set_prevcount;        // 19
//   bool previous_got_int;     // 20
//   bool cmdwin;               // 21
//   bool noexmode;             // 22
//   bool toplevel;             // 23
//   oparg_T oa;                // 24  (84 bytes)
//   // padding to align oa:    (oa starts at 24 since 8 bools = 8 bytes, state=16, total=24)
//   cmdarg_T ca;               // 108 (88 bytes, starting after oa=84 bytes)
//   int mapped_len;            // 196
//   int old_mapped_len;        // 200
//   int idx;                   // 204
//   int c;                     // 208
//   int old_col;               // 212
//   // padding: 4 bytes        // 216
//   pos_T old_pos;             // 220 (12 bytes: i32, c_int, c_int)
// } NormalState;               // sizeof = 232
// =============================================================================

/// Mirror of C `NormalState`.
///
/// The first field is `VimState`, so a `*mut NormalState` can be cast to
/// `*mut VimState` (C guarantees this for the initial member).
///
/// Field offsets verified by `_Static_assert` in `normal_shim.c`:
/// - state:0 (16 bytes), command_finished:16, ctrl_w:17, need_flushbuf:18
/// - set_prevcount:19, previous_got_int:20, cmdwin:21, noexmode:22, toplevel:23
/// - oa:24 (84 bytes), [pad:108-111], ca:112 (88 bytes)
/// - mapped_len:200, old_mapped_len:204, idx:208, c:212, old_col:216
/// - old_pos:220 (12 bytes)
///
/// sizeof(NormalState) = 232
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NormalState {
    pub state: VimState,
    pub command_finished: bool,
    pub ctrl_w: bool,
    pub need_flushbuf: bool,
    pub set_prevcount: bool,
    pub previous_got_int: bool,
    pub cmdwin: bool,
    pub noexmode: bool,
    pub toplevel: bool,
    pub oa: OpargT,
    _pad_oa_ca: [u8; 4], // padding between oa (108) and ca (112)
    pub ca: CmdargT,
    pub mapped_len: c_int,
    pub old_mapped_len: c_int,
    pub idx: c_int,
    pub c: c_int,
    pub old_col: c_int,
    pub old_pos: PosT,
}

impl Default for NormalState {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

// =============================================================================
// Operator type constants (from ops.h)
// =============================================================================

pub const OP_NOP: c_int = 0;
pub const OP_DELETE: c_int = 1;
pub const OP_YANK: c_int = 2;
pub const OP_CHANGE: c_int = 3;
pub const OP_LSHIFT: c_int = 4;
pub const OP_RSHIFT: c_int = 5;
pub const OP_FILTER: c_int = 6;
pub const OP_TILDE: c_int = 7;
pub const OP_INDENT: c_int = 8;
pub const OP_FORMAT: c_int = 9;
pub const OP_COLON: c_int = 10;
pub const OP_UPPER: c_int = 11;
pub const OP_LOWER: c_int = 12;
pub const OP_JOIN: c_int = 13;
pub const OP_JOIN_NS: c_int = 14;
pub const OP_ROT13: c_int = 15;

// =============================================================================
// Motion type constants (from normal_defs.h MotionType enum)
// =============================================================================

pub const K_MT_CHARWISE: c_int = 0;
pub const K_MT_LINEWISE: c_int = 1;
pub const K_MT_BLOCKWISE: c_int = 2;
pub const K_MT_UNKNOWN: c_int = -1;

// =============================================================================
// Command retval constants (from normal_defs.h)
// =============================================================================

pub const CA_COMMAND_BUSY: c_int = 1;
pub const CA_NO_ADJ_OP_END: c_int = 2;

// NUL character
pub const NUL_CHAR: c_int = 0;
