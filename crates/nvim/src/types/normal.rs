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
/// One normal-mode command as the dispatcher hands it to its handler.
pub struct CmdArg {
    /// The operator the command is pending on: the normal-mode frame's own,
    /// held in the frame table so `op_pending` can read it from inside
    /// whatever the handler re-enters.
    pub(crate) oap: crate::winlayer::Live<OpArg>,
    pub cmdchar: ::core::ffi::c_int,
    pub nchar: ::core::ffi::c_int,
    /// `nchar` and its combining characters, encoded, NUL-terminated
    /// (`nchar_len` bytes); empty when `nchar` has none.
    pub nchar_composing: [u8; 32],
    pub nchar_len: ::core::ffi::c_int,
    pub extra_char: ::core::ffi::c_int,
    pub opcount: ::core::ffi::c_int,
    pub count0: ::core::ffi::c_int,
    pub count1: ::core::ffi::c_int,
    pub arg: ::core::ffi::c_int,
    pub outcome: Outcome,
    /// The pattern `/` or `?` read, kept for the redo.
    pub searchbuf: Option<crate::memory::XString>,
}

impl CmdArg {
    /// Back to [`CmdArg::new`]'s state, still pending on the same operator.
    /// Field by field: the dispatcher does this once per command, and
    /// building a whole new value to move over this one costs a copy and a
    /// drop.
    pub(crate) fn reset(&mut self) {
        self.cmdchar = 0;
        self.nchar = 0;
        self.nchar_composing = [0; 32];
        self.nchar_len = 0;
        self.extra_char = 0;
        self.opcount = 0;
        self.count0 = 0;
        self.count1 = 0;
        self.arg = 0;
        self.outcome = Outcome::NONE;
        self.searchbuf = None;
    }

    /// A command with nothing read yet, pending on `op`.
    pub(crate) fn new(op: crate::winlayer::Live<OpArg>) -> Self {
        CmdArg {
            oap: op,
            cmdchar: 0,
            nchar: 0,
            nchar_composing: [0; 32],
            nchar_len: 0,
            extra_char: 0,
            opcount: 0,
            count0: 0,
            count1: 0,
            arg: 0,
            outcome: Outcome::NONE,
            searchbuf: None,
        }
    }
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
