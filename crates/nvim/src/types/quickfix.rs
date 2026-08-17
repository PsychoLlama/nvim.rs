#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;
use crate::file_search::Name;

pub type qf_info_T = qf_info_S;
/// A stack of quickfix (or location) lists. Windows and buffers point at one.
///
/// The quickfix stack is a single static; a location list stack is boxed,
/// shared by reference between windows and freed at the last reference.
/// Callers hold `*mut qf_info_T` throughout, because an autocommand can
/// reach the same stack while a command is walking it.
pub struct qf_info_S {
    /// How many windows point at this stack. Meaningless for the quickfix
    /// stack, which is never freed.
    pub qf_refcount: ::core::ffi::c_int,
    /// How many of [`qf_lists`](Self::qf_lists) hold a list. The rest are
    /// zeroed and unused.
    pub qf_listcount: ::core::ffi::c_int,
    /// Which list `:cc` and friends work on.
    pub qf_curlist: ::core::ffi::c_int,
    /// Room for `'chistory'` (or `'lhistory'`) lists, oldest first.
    pub qf_lists: Vec<qf_list_T>,
    pub qfl_type: qfltype_T,
    /// The buffer the quickfix window shows this stack in, or
    /// `INVALID_QFBUFNR`.
    pub qf_bufnr: ::core::ffi::c_int,
}

impl qf_info_S {
    /// An empty stack with no room for any list; [`qf_alloc_stack`] gives it
    /// its slots.
    ///
    /// [`qf_alloc_stack`]: ../quickfix/stack/fn.qf_alloc_stack.html
    pub const fn new(qfl_type: qfltype_T) -> Self {
        qf_info_S {
            qf_refcount: 0,
            qf_listcount: 0,
            qf_curlist: 0,
            qf_lists: Vec::new(),
            qfl_type,
            qf_bufnr: 0,
        }
    }

    /// How many lists the stack has room for — `'chistory'` for the
    /// quickfix stack, `'lhistory'` for a location list stack.
    pub fn max_count(&self) -> ::core::ffi::c_int {
        self.qf_lists.len() as ::core::ffi::c_int
    }
}
pub type qfltype_T = ::core::ffi::c_uint;
pub const QFLT_INTERNAL: qfltype_T = 2;
pub const QFLT_LOCATION: qfltype_T = 1;
pub const QFLT_QUICKFIX: qfltype_T = 0;
/// One quickfix list within a stack.
#[derive(Copy, Clone)]
pub struct qf_list_T {
    pub qf_id: ::core::ffi::c_uint,
    pub qfl_type: qfltype_T,
    pub qf_start: *mut qfline_T,
    pub qf_last: *mut qfline_T,
    pub qf_ptr: *mut qfline_T,
    pub qf_count: ::core::ffi::c_int,
    pub qf_index: ::core::ffi::c_int,
    pub qf_nonevalid: bool,
    pub qf_has_user_data: bool,
    pub qf_title: *mut ::core::ffi::c_char,
    pub qf_ctx: *mut typval_T,
    pub qf_qftf_cb: Callback,
    pub qf_dir_stack: *mut DirStack,
    pub qf_directory: *mut ::core::ffi::c_char,
    pub qf_file_stack: *mut DirStack,
    pub qf_currfile: *mut ::core::ffi::c_char,
    pub qf_multiline: bool,
    pub qf_multiignore: bool,
    pub qf_multiscan: bool,
    pub qf_changedtick: ::core::ffi::c_int,
}
/// The directories `%D`/`%X` (or `%O`/`%P`/`%Q`) pushed while parsing, the
/// one most recently entered last.
///
/// A list holds two of these, as raw pointers rather than by value, because
/// a list slot is created by zeroing it. Null means "no directory was ever
/// pushed"; see `quickfix::entry` for the operations.
pub struct DirStack {
    pub(crate) dirs: Vec<Name>,
}
pub type qfline_T = qfline_S;
/// One entry in a quickfix list.
#[derive(Copy, Clone)]
pub struct qfline_S {
    pub qf_next: *mut qfline_T,
    pub qf_prev: *mut qfline_T,
    pub qf_lnum: linenr_T,
    pub qf_end_lnum: linenr_T,
    pub qf_fnum: ::core::ffi::c_int,
    pub qf_col: ::core::ffi::c_int,
    pub qf_end_col: ::core::ffi::c_int,
    pub qf_nr: ::core::ffi::c_int,
    pub qf_module: *mut ::core::ffi::c_char,
    pub qf_fname: *mut ::core::ffi::c_char,
    pub qf_pattern: *mut ::core::ffi::c_char,
    pub qf_text: *mut ::core::ffi::c_char,
    pub qf_viscol: ::core::ffi::c_char,
    pub qf_cleared: ::core::ffi::c_char,
    pub qf_type: ::core::ffi::c_char,
    pub qf_user_data: typval_T,
    pub qf_valid: ::core::ffi::c_char,
}
