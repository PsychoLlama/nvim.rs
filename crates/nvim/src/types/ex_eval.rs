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

pub struct Cleanup {
    pub pending: ::core::ffi::c_int,
    pub(crate) exception: Option<ExcId>,
}

/// What a `:finally` postponed at one level of the condition stack, or
/// what a pending report is about: a `:return`'s value or an exception.
/// Upstream's union of two pointer arrays, with its tag made explicit.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Pend {
    None,
    /// The value a `:return` carries, null for none.
    Return(*mut ::core::ffi::c_void),
    Exception(ExcId),
}
pub struct CondStack {
    pub cs_flags: [crate::ex_eval::CsFlags; 50],
    pub cs_pending: [::core::ffi::c_char; 50],
    /// What the `:finally` clause at each level postponed: the pending
    /// `:return`'s value, or the pending exception -- also the exception an
    /// active catch clause caught. Which of the two is meaningful is what
    /// `cs_pending` says, and the accessors below are the only way in.
    cs_pend: [Pend; 50],
    pub cs_forinfo: [*mut ::core::ffi::c_void; 50],
    pub cs_line: [::core::ffi::c_int; 50],
    pub cs_idx: ::core::ffi::c_int,
    pub cs_looplevel: ::core::ffi::c_int,
    pub cs_trylevel: ::core::ffi::c_int,
    pub cs_emsg_silent_list: *mut EsList,
    pub cs_lflags: crate::ex_eval::CsLoopFlags,
}

impl CondStack {
    /// An empty stack.
    pub(crate) fn new() -> CondStack {
        CondStack {
            cs_flags: [crate::ex_eval::CsFlags::NONE; 50],
            cs_pending: [0; 50],
            cs_pend: [Pend::None; 50],
            cs_forinfo: [::core::ptr::null_mut(); 50],
            cs_line: [0; 50],
            cs_idx: -1,
            cs_looplevel: 0,
            cs_trylevel: 0,
            cs_emsg_silent_list: ::core::ptr::null_mut(),
            cs_lflags: crate::ex_eval::CsLoopFlags::NONE,
        }
    }

    /// The value a `:return` postponed at level `idx`, or null. Meaningful
    /// when `cs_pending[idx]` is `CSTP_RETURN`.
    pub fn pending_return(&self, idx: usize) -> *mut ::core::ffi::c_void {
        match self.cs_pend[idx] {
            Pend::Return(value) => value,
            _ => ::core::ptr::null_mut(),
        }
    }

    /// Postpone a `:return`'s value at level `idx`.
    pub fn set_pending_return(&mut self, idx: usize, result: *mut ::core::ffi::c_void) {
        self.cs_pend[idx] = Pend::Return(result);
    }

    /// The exception postponed at level `idx`, or the one its active catch
    /// clause caught. Meaningful when `cs_pending[idx]` carries
    /// `CSTP_THROW`, and when the level is in an active catch clause.
    pub(crate) fn pending_exception(&self, idx: usize) -> Option<ExcId> {
        match self.cs_pend[idx] {
            Pend::Exception(id) => Some(id),
            _ => None,
        }
    }

    /// Postpone an exception at level `idx`.
    pub(crate) fn set_pending_exception(&mut self, idx: usize, exception: Option<ExcId>) {
        self.cs_pend[idx] = exception.map_or(Pend::None, Pend::Exception);
    }
}
pub struct EsList {
    pub saved_emsg_silent: ::core::ffi::c_int,
    pub next: *mut EsList,
}
pub type ExceptType = ::core::ffi::c_uint;
pub struct ExceptionState {
    pub(crate) estate_current_exception: Option<ExcId>,
    pub estate_did_throw: bool,
    pub estate_need_rethrow: bool,
    pub estate_trylevel: ::core::ffi::c_int,
    pub estate_did_emsg: ::core::ffi::c_int,
}

/// One error message an error exception may be built from.
pub(crate) struct ErrorMsg {
    pub(crate) msg: crate::memory::XString,
    /// The script it came from, when there was one.
    pub(crate) sfile: Option<crate::memory::XString>,
    pub(crate) slnum: LineNr,
    pub(crate) multiline: bool,
}

/// The error messages one command gave on its way to becoming an error
/// exception: upstream's `msglist_T` chain, plus where in it the exception
/// value starts (the head's `throw_msg`) as an entry and a byte offset.
#[derive(Default)]
pub(crate) struct ErrorMsgs {
    pub(crate) entries: Vec<ErrorMsg>,
    pub(crate) throw_at: (usize, usize),
}

impl ErrorMsgs {
    /// The text the exception value is made from.
    pub(crate) fn throw_msg(&self) -> &[u8] {
        let (entry, offset) = self.throw_at;
        self.entries
            .get(entry)
            .map_or(&[], |m| &m.msg[offset.min(m.msg.len())..])
    }
}

pub struct Exception {
    pub type_0: ExceptType,
    pub value: *mut ::core::ffi::c_char,
    /// What an error exception was built from; empty for the other kinds.
    pub(crate) messages: ErrorMsgs,
    pub throw_name: *mut ::core::ffi::c_char,
    pub throw_lnum: LineNr,
    pub stacktrace: *mut List,
}

/// An exception's place in the exception table.
pub(crate) type ExcId = crate::slot_table::SlotId<Exception>;
