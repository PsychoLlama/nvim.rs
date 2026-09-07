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
use crate::file_search::Name;

/// A stack of quickfix (or location) lists. Windows and buffers point at one.
///
/// The quickfix stack is a single static; a location list stack is boxed,
/// shared by reference between windows and freed at the last reference.
/// Callers hold `*mut QfInfo` throughout, because an autocommand can
/// reach the same stack while a command is walking it.
pub struct QfInfo {
    /// How many windows point at this stack. Meaningless for the quickfix
    /// stack, which is never freed.
    pub qf_refcount: Refcount,
    /// How many of [`qf_lists`](Self::qf_lists) hold a list. The rest are
    /// zeroed and unused.
    pub qf_listcount: ::core::ffi::c_int,
    /// Which list `:cc` and friends work on.
    pub qf_curlist: ::core::ffi::c_int,
    /// Room for `'chistory'` (or `'lhistory'`) lists, oldest first.
    pub qf_lists: Vec<QfList>,
    pub qfl_type: QfListType,
    /// The buffer the quickfix window shows this stack in, or
    /// `INVALID_QFBUFNR`.
    pub qf_bufnr: ::core::ffi::c_int,
}

impl QfInfo {
    /// An empty stack with no room for any list. A location list stack gets
    /// its slots from [`qf_alloc_stack`]; the quickfix stack — this type's
    /// one `const` use, the static behind `QfStack::Global` — gets them from
    /// `qf_init_stack` during startup.
    ///
    /// [`qf_alloc_stack`]: ../quickfix/stack/fn.qf_alloc_stack.html
    pub const fn new(qfl_type: QfListType) -> Self {
        QfInfo {
            qf_refcount: Refcount::ZERO,
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
        // 'chistory'/'lhistory' cap the stack three orders of magnitude below
        // `c_int::MAX`, so the saturation is unreachable.
        ::core::ffi::c_int::try_from(self.qf_lists.len()).unwrap_or(::core::ffi::c_int::MAX)
    }
}
pub type QfListType = ::core::ffi::c_uint;
pub const QFLT_INTERNAL: QfListType = 2;
pub const QFLT_LOCATION: QfListType = 1;
pub const QFLT_QUICKFIX: QfListType = 0;
/// One quickfix list within a stack.
#[derive(Clone)]
pub struct QfList {
    pub qf_id: ::core::ffi::c_uint,
    pub qfl_type: QfListType,
    pub qf_start: *mut QfLine,
    pub qf_last: *mut QfLine,
    pub qf_ptr: *mut QfLine,
    pub qf_count: ::core::ffi::c_int,
    pub qf_index: ::core::ffi::c_int,
    pub qf_nonevalid: bool,
    pub qf_has_user_data: bool,
    pub qf_title: *mut ::core::ffi::c_char,
    pub qf_ctx: *mut TypVal,
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
/// One entry in a quickfix list.
pub struct QfLine {
    pub qf_next: *mut QfLine,
    pub qf_prev: *mut QfLine,
    pub qf_lnum: LineNr,
    pub qf_end_lnum: LineNr,
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
    pub qf_user_data: TypVal,
    pub qf_valid: ::core::ffi::c_char,
}
