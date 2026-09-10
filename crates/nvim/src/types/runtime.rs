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

/// What `do_in_runtimepath` calls for each match: `(num_files, files, all,
/// cookie)`, returning whether the walk should stop.
///
/// Spelled separately from [`DoInRuntimepathCB`] so a caller can name the bare
/// function type when it hands one over.
pub type DoInRuntimepathCBFn = unsafe fn(
    ::core::ffi::c_int,
    *mut *mut ::core::ffi::c_char,
    bool,
    *mut ::core::ffi::c_void,
) -> bool;
pub type DoInRuntimepathCB = Option<DoInRuntimepathCBFn>;
#[derive(Copy, Clone)]
pub struct EStack {
    pub es_lnum: LineNr,
    pub es_name: *mut ::core::ffi::c_char,
    pub es_type: EStackType,
    pub es_info: EstackInfo,
}

/// What an execution-stack frame is running, for the two kinds of frame
/// that name something beyond `es_name`.
///
/// Upstream is a four-armed union keyed by `es_type`; only these two arms
/// were ever read, and an autocommand frame whose walk has run out holds
/// [`EstackInfo::None`] rather than a null pointer under an `aucmd` tag.
#[derive(Copy, Clone)]
pub enum EstackInfo {
    /// Nothing beyond the frame's name: the bottom frame, a script, a
    /// modeline, an exception, `--cmd` arguments, and so on.
    None,
    /// A user function -- the frame's `es_name` is its name.
    UserFunction(*mut UserFunc),
    /// The autocommand walk this frame is running.
    Autocommand(*mut AutoPatCmd),
}

impl EstackInfo {
    /// The user function this frame is running, if it is running one.
    pub fn user_function(self) -> Option<*mut UserFunc> {
        match self {
            EstackInfo::UserFunction(ufunc) => Some(ufunc),
            _ => None,
        }
    }

    /// The autocommand walk this frame is running, if it is running one.
    pub fn autocommand(self) -> Option<*mut AutoPatCmd> {
        match self {
            EstackInfo::Autocommand(aucmd) => Some(aucmd),
            _ => None,
        }
    }
}
pub type EStackArg = ::core::ffi::c_uint;
pub type EStackType = ::core::ffi::c_uint;
/// Per-line counters of a profiled script, the element type of
/// [`ScriptItem::sn_prl_ga`].
#[derive(Copy, Clone, Default)]
pub(crate) struct SnPrl {
    pub(crate) snp_count: ::core::ffi::c_int,
    pub(crate) sn_prl_total: ProfTime,
    pub(crate) sn_prl_self: ProfTime,
}

pub struct ScriptItem {
    pub sn_vars: *mut ScriptVar,
    pub sn_name: *mut ::core::ffi::c_char,
    pub sn_lua: bool,
    pub sn_prof_on: bool,
    pub sn_pr_force: bool,
    pub sn_pr_child: ProfTime,
    pub sn_pr_nest: ::core::ffi::c_int,
    pub sn_pr_count: ::core::ffi::c_int,
    pub sn_pr_total: ProfTime,
    pub sn_pr_self: ProfTime,
    pub sn_pr_start: ProfTime,
    pub sn_pr_children: ProfTime,
    pub(crate) sn_prl_ga: Vec<SnPrl>,
    pub sn_prl_start: ProfTime,
    pub sn_prl_children: ProfTime,
    pub sn_prl_wait: ProfTime,
    pub sn_prl_idx: LineNr,
    pub sn_prl_execed: ::core::ffi::c_int,
}

impl ScriptItem {
    /// A script that has never been sourced -- what upstream's
    /// `xcalloc(1, sizeof(ScriptItem))` handed a new registry slot.
    pub fn new() -> Self {
        Self {
            sn_vars: ::core::ptr::null_mut(),
            sn_name: ::core::ptr::null_mut(),
            sn_lua: false,
            sn_prof_on: false,
            sn_pr_force: false,
            sn_pr_child: 0,
            sn_pr_nest: 0,
            sn_pr_count: 0,
            sn_pr_total: 0,
            sn_pr_self: 0,
            sn_pr_start: 0,
            sn_pr_children: 0,
            sn_prl_ga: Vec::new(),
            sn_prl_start: 0,
            sn_prl_children: 0,
            sn_prl_wait: 0,
            sn_prl_idx: 0,
            sn_prl_execed: 0,
        }
    }
}

impl Default for ScriptItem {
    fn default() -> Self {
        Self::new()
    }
}
/// Not `Clone`: it holds a script's `s:` scope by value, and a dictionary
/// owns the items its hash table indexes.
pub struct ScriptVar {
    pub sv_var: ScopeDictItem,
    pub sv_dict: Dict,
}
