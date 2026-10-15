//! The execution stack -- what `<sfile>`, `<stack>` and every error message's
//! "line N of ..." prefix are read from.
//!
//! `EXESTACK` is a stack of [`EStack`] entries, one per nested thing being
//! executed: a sourced script, a user function, an autocommand.  [`estack_push`]
//! and [`estack_pop`] bracket each of them, and [`estack_sfile`] renders the
//! stack the three ways vimscript can ask for it -- `<sfile>` (the innermost
//! name), `<stack>` (the whole chain, `..`-joined and carrying line numbers),
//! and the `ESTACK_SCRIPT` form that answers "which script *defined* the frame
//! I am in".  [`stacktrace_create`] and [`f_getstacktrace`] are the same data
//! as a list of dicts.
//!
//! The stack is a `Vec<EStack>` behind a [`GlobalCell`], so every walk below
//! is checked code. A frame owns what it names: its name is a literal, a
//! copy, or -- for a function -- the function itself, which the frame holds
//! a reference to; an autocommand frame carries the script context of the
//! command it is running. `with`/`with_mut` are also the guard: a push made
//! while a walk holds a borrow is a debug panic rather than a reallocated
//! buffer under a live slice.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::eval::typval::ListRef;

use core::ffi::CStr;
use std::ffi::CString;
use std::rc::Rc;

/// A stack entry with no `es_info` payload yet.
fn entry_for(es_type: EStackType, name: FrameName, lnum: LineNr) -> EStack {
    EStack {
        es_lnum: lnum,
        es_name: name,
        es_type,
        es_info: EstackInfo::None,
    }
}

/// Append `entry` to the execution stack.
fn push_entry(entry: EStack) {
    EXESTACK.with_mut(|stack| stack.push(entry));
}

/// Push the bottom frame, the one that stands for "not executing anything".
pub fn estack_init() {
    // Ten frames of nesting are the common case; upstream pre-grows for it.
    EXESTACK.with_mut(|stack| stack.reserve(10));
    push_entry(entry_for(ETYPE_TOP, FrameName::None, 0));
}

/// Add an item to the execution stack.
///
/// The `es_info` payload, where the frame has one, is filled in afterwards
/// through [`with_innermost`]; upstream hands the caller a pointer to the new
/// slot instead, and no caller here wanted one.
pub(crate) fn estack_push(es_type: EStackType, name: FrameName, lnum: LineNr) {
    push_entry(entry_for(es_type, name, lnum));
}

/// Add a user function to the execution stack. The frame holds the function
/// for as long as it is on the stack, and is named by it.
pub fn estack_push_ufunc(func: &Rc<UserFunc>, lnum: LineNr) {
    push_entry(EStack {
        es_lnum: lnum,
        es_name: FrameName::Function,
        es_type: ETYPE_UFUNC,
        es_info: EstackInfo::UserFunction(Rc::clone(func)),
    });
}

/// The innermost frame's name, copied; `None` when it has none.
pub(crate) fn sourcing_name_bytes() -> Option<Vec<u8>> {
    with_sourcing_name(|name| name.map(<[u8]>::to_vec))
}

/// Take an item off of the execution stack. The bottom frame stays.
pub fn estack_pop() {
    let popped = EXESTACK.with_mut(|stack| if stack.len() > 1 { stack.pop() } else { None });
    // The frame's name and function go outside the borrow.
    drop(popped);
}

/// The frame being executed, copied, or `None` when nothing is.
///
/// This is the tree's one spelling of `SOURCING_*`. Upstream cannot express
/// the `None`: its macros index the stack unconditionally and rely on
/// [`estack_init`] having run. Eight files each had their own copy of that
/// index before the stack became a `Vec`.
pub fn innermost() -> Option<EStack> {
    EXESTACK.with(|stack| stack.last().cloned())
}

/// The frame being executed, copied, for the callers that cannot express
/// "none" and want the whole frame -- the line number alone is
/// [`sourcing_lnum`].
///
/// The stack is never empty after [`estack_init`] -- the bottom frame goes on
/// before `main` reads anything and only [`estack_pop`], which keeps it, ever
/// removes one -- and upstream's `SOURCING_*` macros index it unconditionally
/// on the strength of that. A `debug_assert` says so where the tests can see
/// it; a release build answers the bottom frame rather than reading past the
/// end of the stack, which is what upstream would do.
#[track_caller]
pub fn innermost_frame() -> EStack {
    match innermost() {
        Some(entry) => entry,
        None => {
            debug_assert!(false, "the execution stack has no frame");
            entry_for(ETYPE_TOP, FrameName::None, 0)
        }
    }
}

/// Run `f` over the innermost frame, if there is one. See [`innermost_frame`]
/// for why there always is.
#[track_caller]
pub(crate) fn with_innermost(f: impl FnOnce(&mut EStack)) {
    EXESTACK.with_mut(|stack| match stack.last_mut() {
        Some(entry) => f(entry),
        None => debug_assert!(false, "the execution stack has no frame"),
    });
}

/// Rename the innermost frame -- an autocommand frame moving on to its next
/// pattern. The old name is dropped once the stack's borrow is released.
pub(crate) fn set_sourcing_name(name: FrameName) {
    let old = EXESTACK.with_mut(|stack| {
        stack
            .last_mut()
            .map(|entry| core::mem::replace(&mut entry.es_name, name))
    });
    drop(old);
}

/// The line number the innermost frame is on -- upstream's `SOURCING_LNUM`.
pub(crate) fn sourcing_lnum() -> LineNr {
    EXESTACK.with(|stack| stack.last().map_or(0, |entry| entry.es_lnum))
}

/// Whether the innermost frame has a name -- upstream's
/// `SOURCING_NAME != NULL`.
pub(crate) fn has_sourcing_name() -> bool {
    with_sourcing_name(|name| name.is_some())
}

/// The name of the innermost frame, copied -- `None` when nothing is
/// executing.
pub(crate) fn sourcing_name_copy() -> Option<Vec<u8>> {
    sourcing_name_bytes()
}

/// Run `f` over the innermost frame's name, borrowed -- `None` when nothing
/// is executing or the frame has no name. For a reader that only compares
/// or prints it, which [`sourcing_name_copy`] would make allocate.
///
/// `f` must not push onto or pop the execution stack: the stack is borrowed
/// for the call, and the cell catches a nested borrow.
pub(crate) fn with_sourcing_name<R>(f: impl FnOnce(Option<&[u8]>) -> R) -> R {
    EXESTACK.with(|stack| f(stack.last().and_then(EStack::name).map(CStr::to_bytes)))
}

/// Move the innermost frame to `lnum`.
pub(crate) fn set_sourcing_lnum(lnum: LineNr) {
    EXESTACK.with_mut(|stack| {
        if let Some(entry) = stack.last_mut() {
            entry.es_lnum = lnum;
        }
    });
}

/// The current value for `<sfile>`, `<stack>` or `<script>`; `None` where
/// nothing names one.
///
/// `which` is `ESTACK_SFILE` for `<sfile>`, `ESTACK_STACK` for `<stack>` or
/// `ESTACK_SCRIPT` for `<script>`.
pub(crate) fn estack_sfile(which: EStackArg) -> Option<XString> {
    // Nothing reached from inside the borrow pushes onto the stack, which is
    // what makes holding it across these calls sound -- and `with` is now the
    // thing that would catch it if that ever stopped being true.
    EXESTACK.with(|stack| {
        let innermost = stack.last()?;

        if which == ESTACK_SFILE && innermost.es_type != ETYPE_UFUNC {
            return innermost.name().map(XString::from_cstr);
        }

        // Evaluated in a function or an autocommand: report the script that
        // *defined* it. At script level the current script's path is the
        // answer.
        if which == ESTACK_SCRIPT {
            return defining_script(stack);
        }

        render_stack(stack, which)
    })
}

/// Walk out from the innermost frame until something says which script we are
/// running under, and return a copy of that script's path.
fn defining_script(stack: &[EStack]) -> Option<XString> {
    for entry in stack.iter().rev() {
        match entry.es_type {
            ETYPE_UFUNC | ETYPE_AUCMD => {
                let def_ctx = match &entry.es_info {
                    EstackInfo::UserFunction(func) => func.script_ctx.get(),
                    EstackInfo::Autocommand(sctx) => *sctx,
                    // An autocommand frame whose walk has finished.
                    EstackInfo::None => continue,
                };
                if def_ctx.sc_sid <= 0 {
                    return None;
                }
                // A positive `sc_sid` is a registry id; an anonymous chunk's
                // item has no name.
                return script_name(def_ctx.sc_sid);
            }
            // A script frame's name is its path.
            ETYPE_SCRIPT => return entry.name().map(XString::from_cstr),
            _ => {}
        }
    }
    None
}

/// Compose the whole stack up to the root, the way it has always been spelled:
/// `"function One[123]..Two[456]..Three"`.
///
/// `None` when no frame had a name.
fn render_stack(stack: &[EStack], which: EStackArg) -> Option<XString> {
    let mut text = Vec::<u8>::new();
    // Whether any frame contributed: a stack of unnamed frames answers
    // `None`, which is not the same answer as an empty string.
    let mut named = false;

    let innermost = stack.len() - 1;
    let mut last_type: EStackType = ETYPE_SCRIPT;
    for (idx, entry) in stack.iter().enumerate() {
        let Some(name) = entry.name() else {
            continue;
        };
        let mut type_name: &CStr = c"";
        if entry.es_type != last_type {
            type_name = match entry.es_type {
                ETYPE_SCRIPT => c"script ",
                ETYPE_UFUNC => c"function ",
                _ => c"",
            };
            last_type = entry.es_type;
        }
        // The bottom entry of `<sfile>` leaves its line number out -- that is
        // what `<slnum>` is for. So does any entry whose number is unset.
        let lnum = if idx == innermost && which != ESTACK_STACK {
            0
        } else {
            entry.es_lnum
        };

        named = true;
        text.extend_from_slice(type_name.to_bytes());
        text.extend_from_slice(name.to_bytes());
        if lnum != 0 {
            text.extend_from_slice(format!("[{lnum}]").as_bytes());
        }
        if idx != innermost {
            text.extend_from_slice(b"..");
        }
    }

    named.then(|| XString::from_vec(text))
}

/// Append one `getstacktrace()` frame to `list`.
///
/// Exactly one of `func` (a user function) and `event` (an autocommand event
/// name) is set; a script frame has neither.
fn stacktrace_push_item(
    list: &ListRef,
    func: Option<&UserFunc>,
    event: Option<&CStr>,
    lnum: LineNr,
    filepath: Option<&CStr>,
) {
    let d_held = tv_dict_alloc_lock(VarLock::Fixed);
    {
        let d = d_held.edit();
        if let Some(func) = func {
            let _ = d.add_func(b"funcref", func.name().as_cstr());
        }
        if event.is_some() {
            let _ = d.add_str(b"event", event);
        }
        let _ = d.add_number(b"lnum", VarNumber::from(lnum));
        let _ = d.add_str(b"filepath", filepath);
    }
    // Upstream marks this local `VAR_LOCKED`; the lock never travels, because
    // `List::push_copy` copies it into a fresh item and a copy is unlocked.
    // The value holds the one reference the allocator handed out, and the
    // append takes a second; both are given back when this frame ends.
    let tv = TypVal::dict(Some(d_held));
    list.edit().push_copy(&tv);
}

/// The execution stack as `getstacktrace()` reports it: one dict per frame,
/// outermost first.
pub fn stacktrace_create() -> Option<ListRef> {
    // A copy of the stack, because building the dicts below runs arbitrary
    // allocation and it is not worth holding the cell's borrow across it.
    let stack = EXESTACK.with(|stack| stack.clone());
    let list = tv_list_alloc(stack.len().cast_signed());

    for entry in &stack {
        match entry.es_type {
            // A script frame's name is its path.
            ETYPE_SCRIPT => stacktrace_push_item(&list, None, None, entry.es_lnum, entry.name()),
            ETYPE_UFUNC => {
                let Some(func) = entry.es_info.user_function() else {
                    continue;
                };
                let sctx = func.script_ctx.get();
                let lnum = entry.es_lnum + sctx.sc_lnum;
                let path = script_path(sctx);
                stacktrace_push_item(&list, Some(func), None, lnum, Some(&path));
            }
            ETYPE_AUCMD => {
                // An autocommand frame whose walk has finished names nothing.
                let Some(sctx) = entry.es_info.autocommand() else {
                    continue;
                };
                let lnum = entry.es_lnum + sctx.sc_lnum;
                let path = script_path(sctx);
                stacktrace_push_item(&list, None, entry.name(), lnum, Some(&path));
            }
            _ => {}
        }
    }
    Some(list)
}

/// The path a frame's defining script was read from, or `""` when it has none
/// (a `-c` argument, a modeline, Lua, ...).
fn script_path(sctx: ScriptCtx) -> CString {
    if sctx.sc_sid <= 0 {
        return c"".to_owned();
    }
    get_scriptname(sctx, false)
}

/// `getstacktrace()` function
pub fn f_getstacktrace(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_list(stacktrace_create());
}
