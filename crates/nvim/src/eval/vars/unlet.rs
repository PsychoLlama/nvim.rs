//! `:unlet`, `:lockvar` and `:unlockvar`.
//!
//! All three share [`ex_unletlock`]'s argument walk and differ only in the
//! callback it is given, so deleting and locking are written here together --
//! as they are upstream.  That one walk is what makes `:unlet` and
//! `:lockvar` agree on what an argument means.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::getdigits_int_at;
use crate::cstr::{self, byte_at};
use crate::eval::lval::{LValue, Slot, Span, Target};
use crate::eval::typval::{
    dict_is_watched, dict_watcher_notify, item_lock, list_iter_mut, list_locked, tv_copy,
    value_check_lock_named,
};
use crate::eval::{FNE_CHECK_START, env_name_len, get_lval};
use crate::ex_docmd::ends_excmd;
use crate::message::state::emsg_severe;
use crate::message_fmt::msg_bytes;
use crate::os::env::vim_unsetenv_named;
use crate::semsg;
use crate::types::{CmdIdx, ExArg, Failed, ListRef, VAR_DICT, VAR_LIST};
use core::ffi::c_int;

use super::{DI_FLAGS_FIX, DI_FLAGS_LOCK, GLV_QUIET, clear_local, do_unlet, with_var};
use crate::eval::typval::TV_INITIAL_VALUE;

/// What [`ex_unletlock`] does to each argument it resolves.
#[derive(Clone, Copy)]
enum Action {
    /// `:unlet`; `!` suppresses "no such variable".
    Unlet { forceit: bool },
    /// `:lockvar` (`lock`) or `:unlockvar`, `deep` levels down.
    Lock { lock: bool, deep: c_int },
}

/// `:unlet`.
pub fn ex_unlet(excmd: &mut ExArg) {
    // `:unlet!` means "do not complain", which reaches `get_lval` as
    // GLV_QUIET and `do_unlet` as `forceit`.
    let forceit = excmd.forceit;
    let glv_flags = if forceit { GLV_QUIET } else { 0 };
    let at = excmd.line.arg;
    ex_unletlock(excmd, at, glv_flags, Action::Unlet { forceit });
}

/// `:lockvar` and `:unlockvar`.
pub fn ex_lockvar(excmd: &mut ExArg) {
    let mut at = excmd.line.arg;
    // Two levels by default: the variable and what it directly holds.
    // `!` is everything, and an explicit count says how deep.
    let mut deep = 2;
    if excmd.forceit {
        deep = -1;
    } else if ascii_isdigit(c_int::from(excmd.line.byte_at(at))) {
        (deep, at) = getdigits_int_at(excmd.line.buffer_mut(), at, false, -1);
        at = excmd.line.skip_white(at);
    }
    let lock = excmd.cmdidx == CmdIdx::lockvar;
    ex_unletlock(excmd, at, 0, Action::Lock { lock, deep });
}

/// The argument walk `:unlet`, `:lockvar` and `:unlockvar` share, doing
/// `action` to each name it resolves from offset `start` on.
///
/// A failure does not stop the walk: parsing carries on so that the trailing
/// arguments are still checked, but `error` suppresses every later action.
fn ex_unletlock(excmd: &mut ExArg, start: usize, glv_flags: c_int, action: Action) {
    let skip = excmd.skip;
    let line = &excmd.line;
    let mut arg = start;
    let mut error = false;

    loop {
        let text = line.rest_of(arg);
        let name_end = if byte_at(text, 0) == b'$' {
            // An environment variable: `get_lval` does not parse one, so
            // the left-hand side is made by hand.
            let len = env_name_len(&text[1..]);
            if len == 0 {
                let text = msg_bytes(text);
                semsg!("E475: Invalid argument: {text}");
                return;
            }
            let lval = LValue::variable(text, 1 + len);
            if !error && !skip && act(lval, action).is_err() {
                error = true;
            }
            arg + 1 + len
        } else {
            let quiet = skip || error;
            let (lval, end) = get_lval(text, None, true, quiet, glv_flags, FNE_CHECK_START);
            if !lval.has_name() {
                // An error, but carry on parsing.
                error = true;
            }
            let trailing = end.map(|end| c_int::from(byte_at(text, end)));
            let Some(end) =
                end.filter(|_| trailing.is_some_and(|c| ascii_iswhite(c) || ends_excmd(c) != 0))
            else {
                if let Some(end) = end {
                    emsg_severe.set(true);
                    let rest = msg_bytes(&text[end..]);
                    semsg!("E488: Trailing characters: {rest}");
                }
                break;
            };
            if !error && !skip && act(lval, action).is_err() {
                error = true;
            }
            arg + end
        };
        arg = line.skip_white(name_end);
        if ends_excmd(c_int::from(line.byte_at(arg))) != 0 {
            break;
        }
    }

    excmd.line.next = excmd.line.check_next(arg);
}

/// Do `action` to what `lval` names.
fn act(lval: LValue<'_>, action: Action) -> Result<(), Failed> {
    match action {
        Action::Unlet { forceit } => do_unlet_var(lval, forceit),
        Action::Lock { lock, deep } => do_lock_var(lval, lock, deep),
    }
}

/// `:unlet`'s action: delete what `lval` names.
fn do_unlet_var(mut lval: LValue<'_>, forceit: bool) -> Result<(), Failed> {
    match &lval.target {
        Target::Variable | Target::Blob { .. } => {
            // A whole variable: an environment variable, a plain name or an
            // expanded one.  A Blob byte is not something `:unlet` removes,
            // so it is the name with its subscript that is looked for.
            let name = lval.name();
            if name.first() == Some(&b'$') {
                cstr::with_terminated(&name[1..], vim_unsetenv_named);
                Ok(())
            } else {
                do_unlet(name, forceit)
            }
        }
        Target::Slot {
            slot: Slot::Item { list, index },
            span,
        } => {
            let (list, index, span) = (list.clone(), *index, *span);
            if value_check_lock_named(list_locked(Some(&list)), lval.name()) {
                return Err(Failed);
            }
            if span.range {
                unlet_range(&list, index, span);
            } else {
                list.remove_at(index);
            }
            Ok(())
        }
        Target::Slot {
            slot: Slot::Key { dict, key },
            ..
        } => {
            let (mut dict, key) = (dict.clone(), key.clone());
            if value_check_lock_named(dict.dv_lock, lval.name()) {
                return Err(Failed);
            }
            let watched = dict_is_watched(Some(&dict));
            let mut oldtv = TV_INITIAL_VALUE;
            if watched {
                lval.with_slot(|tv, _| tv_copy(tv, &mut oldtv));
            }
            dict.remove_key(&key);
            if watched {
                cstr::with_terminated(&key, |key| {
                    dict_watcher_notify(&dict, key, None, Some(&oldtv))
                });
            }
            clear_local(&mut oldtv);
            Ok(())
        }
        // `v:lua` before a `.name` ends in trailing characters, and a key
        // that is not there is refused before it gets here.
        Target::Slot {
            slot: Slot::Variable,
            ..
        }
        | Target::NewKey { .. } => Ok(()),
    }
}

/// Delete the items of `list` from `first` through the range's last, or to
/// the end when it has none.  `first` must be an index into `list`.
fn unlet_range(list: &ListRef, first: usize, span: Span) {
    // The run ends at `n2` when there is one, and at the last item either
    // way.  An empty list has no run at all; `get_lval` refuses the index
    // that would name one, so this only guards the arithmetic.
    let Some(end) = list.len().checked_sub(1) else {
        return;
    };
    let last = if span.empty2 {
        end
    } else {
        first + usize::try_from(span.n2 - span.n1).unwrap_or(0)
    };
    list.remove_range(first, last.min(end));
}

/// `:lockvar`'s and `:unlockvar`'s action: lock or unlock what `lval` names,
/// to `deep` levels.
fn do_lock_var(mut lval: LValue<'_>, lock: bool, deep: c_int) -> Result<(), Failed> {
    match &lval.target {
        Target::Variable | Target::Blob { .. } => {
            // A whole variable.
            let name = lval.name();
            // The C quoted the name without cutting it, so the rest of the
            // command comes along.
            let cannot = || {
                let name = msg_bytes(lval.name_and_rest());
                semsg!("E940: Cannot lock or unlock variable {name}");
            };
            if name.first() == Some(&b'$') {
                // An environment variable has no lock to set.
                cannot();
                return Err(Failed);
            }
            // A fixed variable -- one of `v:` or a scope dictionary -- can
            // only be locked through the container it holds.
            let fixed = with_var(name, true, |item| {
                let kind = item.di_tv.v_type();
                item.di_flags & DI_FLAGS_FIX != 0 && kind != VAR_DICT && kind != VAR_LIST
            });
            match fixed {
                None => return Err(Failed),
                Some(true) => {
                    cannot();
                    return Err(Failed);
                }
                Some(false) => {}
            }
            with_var(name, true, |item| {
                if lock {
                    item.di_flags |= DI_FLAGS_LOCK;
                } else {
                    item.di_flags &= !DI_FLAGS_LOCK;
                }
                if deep != 0 {
                    item_lock(&mut item.di_lock, &mut item.di_tv, deep, lock, false);
                }
            });
        }
        Target::Slot {
            slot: Slot::Item { list, index },
            span,
        } if deep != 0 => {
            // The one List item the lvalue named, or the run of them a
            // range named -- which ends at `n2` unless the range was open,
            // and at the last item either way.
            let count = if !span.range {
                1
            } else if span.empty2 {
                usize::MAX
            } else {
                usize::try_from(span.n2 - span.n1 + 1).unwrap_or(0)
            };
            let (mut list, index) = (list.clone(), *index);
            for item in list_iter_mut(Some(&mut list)).skip(index).take(count) {
                item_lock(&mut item.li_lock, &mut item.li_tv, deep, lock, false);
            }
        }
        Target::Slot {
            slot: Slot::Key { .. },
            ..
        } if deep != 0 => {
            lval.with_slot(|tv, slot_lock| item_lock(slot_lock, tv, deep, lock, false));
        }
        Target::Slot { .. } | Target::NewKey { .. } => {}
    }
    Ok(())
}
