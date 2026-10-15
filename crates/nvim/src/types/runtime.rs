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

/// One frame of the execution stack -- see `runtime::estack`.
#[derive(Clone)]
pub struct EStack {
    pub es_lnum: LineNr,
    pub(crate) es_name: FrameName,
    pub es_type: EStackType,
    pub es_info: EstackInfo,
}

impl EStack {
    /// The frame's name: what `<sfile>` and every "line N of ..." prefix
    /// read. A function's frame is named by its function.
    pub(crate) fn name(&self) -> Option<&::core::ffi::CStr> {
        match &self.es_name {
            FrameName::None => None,
            FrameName::Static(name) => Some(name),
            FrameName::Owned(name) => Some(name.as_cstr()),
            FrameName::Function => match &self.es_info {
                EstackInfo::UserFunction(func) => Some(func.printable_name().as_cstr()),
                _ => None,
            },
        }
    }
}

/// What a frame is called, owned by the frame -- upstream's `es_name`, a
/// pointer into whatever the pusher kept alive.
#[derive(Clone, Default)]
pub(crate) enum FrameName {
    /// No name: the bottom frame, an autocommand frame between commands.
    #[default]
    None,
    /// A literal: `modelines`, `command line`.
    Static(&'static ::core::ffi::CStr),
    /// A copy: a script's path, an autocommand's description.
    Owned(crate::memory::XString),
    /// The name of the function in [`EstackInfo::UserFunction`].
    Function,
}

impl FrameName {
    /// A frame named by a copy of `name`.
    pub(crate) fn owned(name: &::core::ffi::CStr) -> Self {
        FrameName::Owned(crate::memory::XString::from_cstr(name))
    }
}

/// What an execution-stack frame is running, for the two kinds of frame
/// that name something beyond `es_name`.
///
/// Upstream is a four-armed union keyed by `es_type`; only these two arms
/// were ever read, and an autocommand frame whose walk has run out holds
/// [`EstackInfo::None`] rather than a null pointer under an `aucmd` tag.
#[derive(Clone)]
pub enum EstackInfo {
    /// Nothing beyond the frame's name: the bottom frame, a script, a
    /// modeline, an exception, `--cmd` arguments, and so on.
    None,
    /// A user function, held for as long as the frame is.
    UserFunction(::std::rc::Rc<UserFunc>),
    /// An autocommand walk: the script context of the command it is
    /// running, which the walk updates as it moves from one to the next.
    Autocommand(ScriptCtx),
}

impl EstackInfo {
    /// The user function this frame is running, if it is running one.
    pub fn user_function(&self) -> Option<&::std::rc::Rc<UserFunc>> {
        match self {
            EstackInfo::UserFunction(ufunc) => Some(ufunc),
            _ => None,
        }
    }

    /// The script context of the autocommand this frame is running, if it
    /// is running one.
    pub fn autocommand(&self) -> Option<ScriptCtx> {
        match self {
            EstackInfo::Autocommand(sctx) => Some(*sctx),
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
    pub sn_vars: Option<Box<ScriptVar>>,
    /// The script's resolved file name; `None` for an anonymous chunk.
    pub sn_name: Option<crate::memory::XString>,
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
            sn_vars: None,
            sn_name: None,
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
/// A script's `s:` scope: the entry a bare `s:` resolves to, whose value is
/// the scope's dictionary.
pub struct ScriptVar {
    pub sv_var: ScopeDictItem,
}
