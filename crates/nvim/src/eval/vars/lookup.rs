//! Resolving a name to the `DictItem` that holds it.
//!
//! [`find_var_ht_dict`] picks the scope from the name's prefix,
//! [`find_var_in_ht`] finds the entry in it (and is where a bare
//! `g:`/`b:`/`l:` becomes the scope's own dictionary item), and
//! [`eval_variable`] is the whole path an expression takes.
//! [`get_user_var_name`] walks the same scopes for completion.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::winlayer::TabPage;
use crate::winlayer::{Buf, Win};
use core::ffi::CStr;
use std::ffi::CString;

use super::*;
use crate::eval::typval::DictRef;
use crate::eval::typval::NumBuf;
use crate::types::{Candidate, Failed, NUL};

/// `"<prefix>:<name>"`, as a completion candidate.
pub(crate) fn cat_prefix_varname(prefix: u8, name: &CStr) -> Candidate {
    let mut out = Vec::with_capacity(name.count_bytes() + 3);
    out.extend_from_slice(&[prefix, b':']);
    out.extend_from_slice(name.to_bytes());
    Candidate::Owned(CString::new(out).expect("a variable name holds no NUL"))
}

/// The `idx`-th variable name for command-line completion, or `None` when
/// there are no more.
///
/// This is a generator, not a function: `idx == 0` restarts the walk and
/// every later call resumes it, so the cursor into each scope lives in a
/// `static`.  The five scopes are visited in turn -- `g:`, `b:`, `w:`, `t:`,
/// then the whole `v:` table -- and only `g:` answers a bare name, because
/// that is the scope an unprefixed one completes in.
pub fn get_user_var_name(expand: &Expand, idx: usize) -> Option<Candidate> {
    static gdone: GlobalCell<size_t> = GlobalCell::new(0);
    static bdone: GlobalCell<size_t> = GlobalCell::new(0);
    static wdone: GlobalCell<size_t> = GlobalCell::new(0);
    static tdone: GlobalCell<size_t> = GlobalCell::new(0);
    static vidx: GlobalCell<size_t> = GlobalCell::new(0);
    /// The hashtab cursor, shared by the four hashtab scopes: only one of
    /// them is being walked at a time.
    ///
    /// A slot *index*, parked across calls into the editor: the table's
    /// small run lives inside the table, so a pointer cursor would not
    /// survive the next mutation of it.
    static slot: GlobalCell<usize> = GlobalCell::new(0);

    if idx == 0 {
        gdone.set(0);
        bdone.set(0);
        wdone.set(0);
        tdone.set(0);
        vidx.set(0);
    }

    // One step through `dict`: the first call starts at the array, every
    // later one advances past the slot the previous call answered and then
    // skips the empty and removed ones. The key is copied out.
    let step = |done: &GlobalCell<size_t>, dict: Option<DictRef>| -> Option<CString> {
        let dict = dict?;
        let n = done.get();
        if n >= dict.dv_hashtab.ht_used {
            return None;
        }
        done.set(n + 1);
        let mut at = if n == 0 { 0 } else { slot.get() + 1 };
        // The table holds `ht_used > n` live items, so a kept slot is still
        // ahead of the cursor.
        let key = loop {
            if let Some(item) = dict.item_at(at) {
                break item.key().to_vec();
            }
            at += 1;
        };
        slot.set(at);
        Some(CString::new(key).expect("a variable name holds no NUL"))
    };

    if let Some(key) = step(&gdone, Some(globvar_dict())) {
        if expand.pattern_starts_with(b"g:") {
            return Some(cat_prefix_varname(b'g', &key));
        }
        return Some(Candidate::Owned(key));
    }
    // The window this completes for is the one the command line was
    // opened over, which is `prevwin` while the command-line window is
    // current.
    let win = prevwin_curwin();
    let bvars = win.buffer().b_bufvar.di_tv.dict_handle();
    if let Some(key) = step(&bdone, bvars) {
        return Some(cat_prefix_varname(b'b', &key));
    }
    if let Some(key) = step(&wdone, win.w_winvar.di_tv.dict_handle()) {
        return Some(cat_prefix_varname(b'w', &key));
    }
    let tvars = TabPage::current().tp_winvar.di_tv.dict_handle();
    if let Some(key) = step(&tdone, tvars) {
        return Some(cat_prefix_varname(b't', &key));
    }
    let v = vidx.get();
    let vv = Vv::try_from(v).ok()?;
    vidx.set(v + 1);
    Some(cat_prefix_varname(b'v', get_vim_var_name(vv)))
}

/// Read the variable `name` into `result`, reporting E121 if it does not
/// exist.
///
/// `result` may be `None` to ask only whether the variable exists.
/// `verbose` allows the error; the message is suppressed for a lookup that
/// is allowed to fail.
pub(crate) fn eval_variable(
    name: &[u8],
    result: Option<&mut TypVal>,
    verbose: bool,
    no_autoload: bool,
) -> Result<(), Failed> {
    let Some(found) = locate(name, no_autoload) else {
        if result.is_some() && verbose {
            let name = msg_bytes(name);
            semsg!("E121: Undefined variable: {name}");
        }
        return Err(Failed);
    };
    if let Some(result) = result {
        // One borrow of the item, copying the value out: nothing runs
        // between finding the variable and reading it.
        match found {
            Located::Item { dict, slot } => {
                if let Some(item) = dict.item_at(slot) {
                    tv_copy(&item.di_tv, result);
                }
            }
            Located::Entry(kind) => {
                with_scope_entry(kind, |item| tv_copy(&item.di_tv, result));
            }
        }
    }
    Ok(())
}

/// Note in [`LAMBDA_USES_LOCALS`] that `name` is a function-local variable
/// or an argument, which is what makes a lambda capture it.
pub(crate) fn check_vars(name: &[u8]) {
    if LAMBDA_USES_LOCALS.get().is_none() {
        return;
    }
    let local = find_var_home(name)
        .is_some_and(|home| matches!(home.kind, ScopeKind::Local | ScopeKind::Args));
    if local && locate(name, true).is_some() {
        LAMBDA_USES_LOCALS.set(Some(true));
    }
}

/// Which scope a variable name lives in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ScopeKind {
    /// `g:`, and an unprefixed name outside a function.
    Global,
    /// `v:`.
    Vim,
    /// An unprefixed name that means a `v:` variable (`version`): the `v:`
    /// dictionary, but with no watchers of its own and none of `v:`'s
    /// assignment rules.
    Compat,
    /// `b:`.
    Buffer,
    /// `w:`.
    Window,
    /// `t:`.
    Tab,
    /// `s:`.
    Script,
    /// `l:`, and an unprefixed name inside a function.
    Local,
    /// `a:`.
    Args,
}

/// Where a variable name lives: the scope, its dictionary, and where in the
/// name the variable's own name starts -- past the `x:` prefix, or 0.
pub(crate) struct VarHome {
    pub(crate) kind: ScopeKind,
    pub(crate) dict: DictRef,
    pub(crate) name_at: usize,
}

/// What [`locate`] found for a name.
pub(crate) enum Located {
    /// A variable: the dictionary holding it and its slot there, good
    /// until user code runs.
    Item { dict: DictRef, slot: usize },
    /// A bare scope name (`g:`, `l:`, ...): the scope's own entry.
    Entry(ScopeKind),
}

/// Run `f` over the variable `name`, answering `None` when there is none.
///
/// The borrow is the item's: `f` must not run user code, nor reach the
/// dictionary the item is in -- the reason this takes a closure rather than
/// answering the item.
pub(crate) fn with_var<R>(
    name: &[u8],
    no_autoload: bool,
    f: impl FnOnce(&mut DictItem) -> R,
) -> Option<R> {
    match locate(name, no_autoload)? {
        Located::Item { dict, slot } => dict.edit().item_at_mut(slot).map(f),
        Located::Entry(kind) => with_scope_entry(kind, f),
    }
}

/// Run `f` over the entry a bare scope name evaluates to, answering `None`
/// when the scope does not exist (no function running, no script).
pub(crate) fn with_scope_entry<R>(
    kind: ScopeKind,
    f: impl FnOnce(&mut DictItem) -> R,
) -> Option<R> {
    match kind {
        ScopeKind::Global => {
            drop(globvar_dict());
            Some(scope_globals.with_mut(|entry| f(entry)))
        }
        ScopeKind::Vim => {
            drop(vimvar_dict());
            Some(scope_vim.with_mut(|vim| f(&mut vim.entry)))
        }
        ScopeKind::Compat => None,
        ScopeKind::Buffer => Some(f(&mut Buf::current().b_bufvar)),
        ScopeKind::Window => Some(f(&mut Win::current().w_winvar)),
        ScopeKind::Tab => Some(f(&mut TabPage::current().tp_winvar)),
        ScopeKind::Script => {
            let sid = current_sctx.get().sc_sid;
            if !script_id_valid(sid) {
                return None;
            }
            with_script_item(sid, |si| {
                si.sn_vars.as_mut().map(|vars| f(&mut vars.sv_var))
            })
        }
        ScopeKind::Local => with_funccal_scope_entry(true, f),
        ScopeKind::Args => with_funccal_scope_entry(false, f),
    }
}

/// The dictionary of the scope `kind` names, as of now.
pub(crate) fn scope_dict(kind: ScopeKind) -> Option<DictRef> {
    match kind {
        ScopeKind::Global => Some(globvar_dict()),
        ScopeKind::Vim | ScopeKind::Compat => Some(vimvar_dict()),
        ScopeKind::Buffer => Buf::current().b_bufvar.di_tv.dict_handle(),
        ScopeKind::Window => Win::current().w_winvar.di_tv.dict_handle(),
        ScopeKind::Tab => TabPage::current().tp_winvar.di_tv.dict_handle(),
        ScopeKind::Script => script_scope_dict(current_sctx.get().sc_sid),
        ScopeKind::Local => funccal_scope(true),
        ScopeKind::Args => funccal_scope(false),
    }
}

/// The variable `name`: where it is, or `None`.
///
/// A miss in `g:` may source an autoload script (user code) and look
/// again; past that, the scopes a lambda closed over are searched. A slot
/// answered here is good until user code runs.
#[inline]
pub(crate) fn locate(name: &[u8], no_autoload: bool) -> Option<Located> {
    let home = find_var_home(name)?;
    // The home's handle moves into the answer: one reference per lookup.
    let varname = &name[home.name_at..];
    if varname.is_empty() {
        return Some(Located::Entry(home.kind));
    }
    if let Some(slot) = home.dict.slot_of(varname) {
        return Some(Located::Item {
            dict: home.dict,
            slot,
        });
    }
    locate_missed(&home, name, no_autoload)
}

/// [`locate`] in a home [`find_var_home`] already answered for `name`.
pub(crate) fn locate_in(home: &VarHome, name: &[u8], no_autoload: bool) -> Option<Located> {
    let varname = &name[home.name_at..];
    if varname.is_empty() {
        // Something like "s:": the scope itself.
        return Some(Located::Entry(home.kind));
    }
    if let Some(slot) = home.dict.slot_of(varname) {
        return Some(Located::Item {
            dict: home.dict.clone(),
            slot,
        });
    }
    locate_missed(home, name, no_autoload)
}

/// [`locate`] past a miss in the home scope itself.
pub(crate) fn locate_missed(home: &VarHome, name: &[u8], no_autoload: bool) -> Option<Located> {
    let varname = &name[home.name_at..];
    // A global may be an autoload variable; sourcing its script may define
    // it.  Don't source one that ran already, or every check of "is this
    // name a Funcref variable" would re-run it.
    if home.kind == ScopeKind::Global
        && !no_autoload
        && script_autoload_named(varname, false)
        && !aborting()
        && let Some(slot) = home.dict.slot_of(varname)
    {
        return Some(Located::Item {
            dict: home.dict.clone(),
            slot,
        });
    }
    // Search the parent scope, which a lambda can reference.
    locate_scoped(name, no_autoload)
}

/// The variable `name` in a scope a lambda closed over.
fn locate_scoped(name: &[u8], no_autoload: bool) -> Option<Located> {
    if !current_func_has_scope() {
        return None;
    }
    // Each probe answers owned handles: nothing is borrowed across the
    // walk's steps.
    walk_scoped_funccals(|| {
        let home = find_var_home(name)?;
        let varname = &name[home.name_at..];
        if varname.is_empty() {
            return None;
        }
        if let Some(slot) = home.dict.slot_of(varname) {
            return Some(Located::Item {
                dict: home.dict,
                slot,
            });
        }
        if home.kind == ScopeKind::Global
            && !no_autoload
            && script_autoload_named(varname, false)
            && !aborting()
        {
            let slot = home.dict.slot_of(varname)?;
            return Some(Located::Item {
                dict: home.dict,
                slot,
            });
        }
        None
    })
}

/// The scope `name` belongs to, or `None` when the name names no scope.
///
/// A name with no prefix is `v:version` if it is a compat name, otherwise
/// the function-local scope if there is one and `g:` if not.  A prefixed one
/// names its scope directly -- and `s:` is where an anonymous Lua or
/// `:execute` chunk is given a script id, so that it can have script
/// variables at all (#15994).
#[inline]
pub(crate) fn find_var_home(name: &[u8]) -> Option<VarHome> {
    let (&lead, _) = name.split_first()?;
    let home = |kind, dict: Option<DictRef>, name_at| {
        dict.map(|dict| VarHome {
            kind,
            dict,
            name_at,
        })
    };
    if name.get(1) != Some(&b':') {
        // An implicit scope. The name must not start with a colon or a
        // '#'.
        if lead == b':' || lead == AUTOLOAD_CHAR as u8 {
            return None;
        }
        // "version" is "v:version" in every scope.
        if is_compat_name(name) {
            return home(ScopeKind::Compat, Some(vimvar_dict()), 0);
        }
        return match funccal_scope(true) {
            Some(local) => home(ScopeKind::Local, Some(local), 0),
            None => home(ScopeKind::Global, Some(globvar_dict()), 0),
        };
    }
    if lead == b'g' {
        return home(ScopeKind::Global, Some(globvar_dict()), 2);
    }
    // Without `g:` there must be no ':' or '#' in the rest.
    if name[2..]
        .iter()
        .any(|&c| c == b':' || c == AUTOLOAD_CHAR as u8)
    {
        return None;
    }
    match lead {
        b'b' => home(ScopeKind::Buffer, scope_dict(ScopeKind::Buffer), 2),
        b'w' => home(ScopeKind::Window, scope_dict(ScopeKind::Window), 2),
        b't' => home(ScopeKind::Tab, scope_dict(ScopeKind::Tab), 2),
        b'v' => home(ScopeKind::Vim, Some(vimvar_dict()), 2),
        b'a' => home(ScopeKind::Args, funccal_scope(false), 2),
        b'l' => home(ScopeKind::Local, funccal_scope(true), 2),
        b's' => home(ScopeKind::Script, script_scope_for_lookup(), 2),
        _ => None,
    }
}

/// The current script's `s:` dictionary for a lookup, giving an anonymous
/// Lua or `:execute` chunk a script item first.
#[inline(never)]
fn script_scope_for_lookup() -> Option<DictRef> {
    // Both calls below fill `sctx` in, and neither reads the cell, so the
    // round trip through a local is what the C's write-through-the-pointer
    // amounts to.
    let mut sctx = current_sctx.get();
    if !((sctx.sc_sid > 0 || sctx.sc_sid == SID_STR || sctx.sc_sid == SID_LUA)
        && sctx.sc_sid <= script_count())
    {
        return None;
    }
    // Resolve the Lua filename and line number, so that a later "Last set
    // from" can name them.
    nlua_set_sctx_in(&mut sctx);
    if sctx.sc_sid == SID_STR || sctx.sc_sid == SID_LUA {
        // An anonymous chunk has no script item yet.
        sctx.sc_sid = new_unnamed_script_item();
    }
    current_sctx.set(sctx);
    script_scope_dict(sctx.sc_sid)
}

/// The string value of the variable `name`, or `None` when it does not
/// exist. A Number renders as its digits.
pub(crate) fn var_string_value(name: &[u8]) -> Option<Vec<u8>> {
    // Copied out, then rendered: rendering a value that is not a string can
    // report an error, which writes `v:errmsg`.
    let value = with_var(name, false, |item| item.di_tv.clone())?;
    let mut numbuf = NumBuf::new();
    Some(numbuf.string(&value).to_bytes().to_vec())
}

/// `exists()` over a variable name: whether `var` names something, including
/// everything a subscript on it reaches.
pub(crate) fn var_exists(var: &[u8]) -> bool {
    let mut n = false;
    // Get the variable name, expanding a `{curly}` name into `expanded`.
    let mut cursor = Cursor::new(var);
    let (len, expanded) = get_name_len(&mut cursor, true, false);
    if let Ok(len) = usize::try_from(len)
        && len > 0
    {
        let mut tv = TV_INITIAL_VALUE;
        let name: &[u8] = match &expanded {
            Some(expanded) => expanded.get(..len).unwrap_or(expanded),
            None => &var[..len],
        };
        n = eval_variable(name, Some(&mut tv), false, true).is_ok();
        if n {
            // Handle `d.key`, `l[idx]` and `Func()`.
            n = handle_subscript(&mut cursor, &mut tv, true, false).is_ok();
            if n {
                clear_local(&mut tv);
            }
        }
    }
    if cursor.byte() != NUL as u8 {
        n = false;
    }
    n
}
