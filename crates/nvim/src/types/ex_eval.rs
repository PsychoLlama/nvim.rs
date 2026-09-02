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

pub type cleanup_T = cleanup_stuff;
#[derive(Copy, Clone)]
pub struct cleanup_stuff {
    pub pending: ::core::ffi::c_int,
    pub exception: *mut except_T,
}
#[derive(Copy, Clone)]
pub struct cstack_T {
    pub cs_flags: [crate::ex_eval::CsFlags; 50],
    pub cs_pending: [::core::ffi::c_char; 50],
    /// What the `:finally` clause at each level postponed: the pending
    /// `:return`'s value, or the pending exception. Which of the two is
    /// meaningful is what `cs_pending` says, and the two accessors below
    /// are the only way in. Upstream is a union of two arrays of the same
    /// pointer type -- two names for one array, not a pun.
    cs_pend: [*mut ::core::ffi::c_void; 50],
    pub cs_forinfo: [*mut ::core::ffi::c_void; 50],
    pub cs_line: [::core::ffi::c_int; 50],
    pub cs_idx: ::core::ffi::c_int,
    pub cs_looplevel: ::core::ffi::c_int,
    pub cs_trylevel: ::core::ffi::c_int,
    pub cs_emsg_silent_list: *mut eslist_T,
    pub cs_lflags: crate::ex_eval::CsLoopFlags,
}

impl cstack_T {
    /// The value a `:return` postponed at level `idx`. Meaningful when
    /// `cs_pending[idx]` is `CSTP_RETURN`.
    pub fn pending_return(&self, idx: usize) -> *mut ::core::ffi::c_void {
        self.cs_pend[idx]
    }

    /// Postpone a `:return`'s value at level `idx`.
    pub fn set_pending_return(&mut self, idx: usize, rettv: *mut ::core::ffi::c_void) {
        self.cs_pend[idx] = rettv;
    }

    /// The exception postponed at level `idx`. Meaningful when
    /// `cs_pending[idx]` carries `CSTP_THROW`, and when the level is in an
    /// active catch clause.
    pub fn pending_exception(&self, idx: usize) -> *mut except_T {
        self.cs_pend[idx].cast::<except_T>()
    }

    /// Postpone an exception at level `idx`.
    pub fn set_pending_exception(&mut self, idx: usize, exception: *mut except_T) {
        self.cs_pend[idx] = exception.cast::<::core::ffi::c_void>();
    }
}
pub type eslist_T = eslist_elem;
#[derive(Copy, Clone)]
pub struct eslist_elem {
    pub saved_emsg_silent: ::core::ffi::c_int,
    pub next: *mut eslist_T,
}
pub type except_T = vim_exception;
pub type except_type_T = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
pub struct exception_state_S {
    pub estate_current_exception: *mut except_T,
    pub estate_did_throw: bool,
    pub estate_need_rethrow: bool,
    pub estate_trylevel: ::core::ffi::c_int,
    pub estate_did_emsg: ::core::ffi::c_int,
}
pub type exception_state_T = exception_state_S;
#[derive(Copy, Clone)]
pub struct msglist {
    pub next: *mut msglist_T,
    pub msg: *mut ::core::ffi::c_char,
    pub throw_msg: *mut ::core::ffi::c_char,
    pub sfile: *mut ::core::ffi::c_char,
    pub slnum: linenr_T,
    pub multiline: bool,
}
pub type msglist_T = msglist;
#[derive(Copy, Clone)]
pub struct vim_exception {
    pub type_0: except_type_T,
    pub value: *mut ::core::ffi::c_char,
    pub messages: *mut msglist_T,
    pub throw_name: *mut ::core::ffi::c_char,
    pub throw_lnum: linenr_T,
    pub stacktrace: *mut list_T,
    pub caught: *mut except_T,
}
