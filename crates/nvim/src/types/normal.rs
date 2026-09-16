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

pub type MotionType = ::core::ffi::c_int;

crate::flag_set! {
    /// What a handler tells the dispatcher about what it did, in the
    /// command's own [`CmdArg::outcome`].
    pub struct Outcome;

    /// The command is not finished: it left insert mode or a command line
    /// running, so nothing that would start one may run after it.
    const COMMAND_BUSY = 1;
    /// Leave the operator's end where the handler put it.
    const NO_ADJ_OP_END = 2;
}
pub struct CmdArg {
    pub oap: *mut OpArg,
    pub cmdchar: ::core::ffi::c_int,
    pub nchar: ::core::ffi::c_int,
    pub nchar_composing: [::core::ffi::c_char; 32],
    pub nchar_len: ::core::ffi::c_int,
    pub extra_char: ::core::ffi::c_int,
    pub opcount: ::core::ffi::c_int,
    pub count0: ::core::ffi::c_int,
    pub count1: ::core::ffi::c_int,
    pub arg: ::core::ffi::c_int,
    pub outcome: Outcome,
    pub searchbuf: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
pub struct OpArg {
    pub op_type: OpType,
    pub regname: ::core::ffi::c_int,
    pub motion_type: MotionType,
    pub motion_force: ::core::ffi::c_int,
    pub use_reg_one: bool,
    pub inclusive: bool,
    pub end_adjusted: bool,
    pub start: Pos,
    pub end: Pos,
    pub cursor_start: Pos,
    pub line_count: LineNr,
    pub empty: bool,
    pub is_visual: bool,
    pub start_vcol: ColNr,
    pub end_vcol: ColNr,
    pub prev_opcount: ::core::ffi::c_int,
    pub prev_count0: ::core::ffi::c_int,
    pub excl_tr_ws: bool,
}

impl OpArg {
    /// All zeros — no pending operator, an empty charwise region.
    ///
    /// This is what `clear_oparg` writes, and what a caller that only wants
    /// `block_prep`'s geometry (`cursor_pos_info`, the register API) starts
    /// from before filling in the two vcols.
    pub const ZERO: Self = OpArg {
        op_type: OpType::Nop,
        regname: 0,
        motion_type: 0,
        motion_force: 0,
        use_reg_one: false,
        inclusive: false,
        end_adjusted: false,
        start: Pos {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        end: Pos {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cursor_start: Pos {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        line_count: 0,
        empty: false,
        is_visual: false,
        start_vcol: 0,
        end_vcol: 0,
        prev_opcount: 0,
        prev_count0: 0,
        excl_tr_ws: false,
    };
}

impl Default for OpArg {
    fn default() -> Self {
        Self::ZERO
    }
}
