//! `vim.schedule()` and `vim.wait()`.
//!
//! [`schedule`] defers a Lua reference onto the main loop's event queue,
//! which is how a fast callback reaches anything that is not fast. [`wait`]
//! is the other direction: it pumps the loop until a condition callback
//! returns true, a timeout expires or the user interrupts, and it is the one
//! place a Lua function drives the event loop re-entrantly.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use super::{fast_api_refusal, in_fast_callback, main_stack, nlua_error, nlua_pcall};
use crate::eval::funcs::find_builtin;
use crate::event::r#loop::{Drain, main_loop_is_closing, wait_on_main_loop};
use crate::event::multiqueue::put_id_event_on_main_loop;
use crate::event::time::ScopedTimer;
use crate::getchar::state::got_int;
use crate::getchar::vgetc;
use crate::guard::Allow;
use crate::lua::ffi::LUA_MULTRET;
use crate::lua::stack::{At, Kind, Raise, Stack};
use crate::types::{LuaRef, lua_Integer};
use crate::ui::state::ui_event_ns_id;
use crate::ui::{ui_flush, ui_remove_cb};

/// How often `vim.wait` wakes to re-test its condition, in milliseconds.
const DEFAULT_INTERVAL: u64 = 200;

/// `vim.wait`'s answer for "the user interrupted".
const WAIT_INTERRUPTED: lua_Integer = -2;
/// `vim.wait`'s answer for "the timeout expired".
const WAIT_TIMED_OUT: lua_Integer = -1;

/// Run a deferred callback on the main loop: `callback` is the reference
/// [`schedule`] took, `ns_id` the UI event namespace it was scheduled from.
///
/// A callback scheduled from a UI event handler (`ns_id > 0`) is let out of
/// the text and expression locks, because it is not running inside whatever
/// held them; if it fails, that handler is detached.
fn run_scheduled(callback: u64, ns_id: u64) {
    let callback = LuaRef::try_from(callback).expect("a reference `schedule` took");
    let ns_id = u32::try_from(ns_id).expect("a namespace `schedule` read");
    let mut stack = main_stack();
    stack.push_reference(callback);
    stack.unreference(callback);

    // A UI-event callback is arbitrary Lua and may legitimately want to move
    // the cursor or set a variable, so the locks come off for it. They go
    // back either way: the callback ran unsupervised.
    let _unlocked_expr_map = Allow::expr_map_when(ns_id > 0);
    let _unlocked_text = Allow::text_changes_when(ns_id > 0);
    if nlua_pcall(&mut stack, 0, 0).is_err() {
        nlua_error(&mut stack, c"vim.schedule callback: %.*s");
        ui_remove_cb(ns_id, true);
    }
}

/// `vim.schedule(fn)`.
///
/// Answers `nil, nil` on success and `nil, "main loop is closing"` when there
/// is no queue left to defer onto — the second value is the reason, so a
/// caller can tell the two apart.
pub(crate) fn schedule(stack: &mut Stack) -> Result<c_int, Raise> {
    if stack.kind(At::arg(1)) != Kind::Function {
        stack.push_bytes(b"vim.schedule: expected function");
        return Err(Raise::top_value());
    }
    stack.push_nil();
    if main_loop_is_closing() {
        stack.push_bytes(b"main loop is closing");
        return Ok(2);
    }

    let callback = stack.reference(At::arg(1));
    let callback = u64::try_from(callback).expect("a function's reference is positive");
    put_id_event_on_main_loop(run_scheduled, callback, u64::from(ui_event_ns_id.get()));
    stack.push_nil();
    Ok(2)
}

/// How one call of `vim.wait`'s condition went.
enum Condition {
    /// Keep waiting: the condition answered nothing, or something falsy.
    Unmet,
    /// Stop: the condition answered something truthy, which is removed; the
    /// `n` values it answered after it are left on top.
    Met(c_int),
    /// Stop: the condition raised, and its error is on top.
    Raised,
}

/// Call `vim.wait`'s condition (slot 2) once. A falsy answer leaves the
/// stack as it was found.
fn wait_condition(stack: &mut Stack) -> Condition {
    let top = stack.top();
    stack.push_value(At::arg(2));
    if nlua_pcall(stack, 0, LUA_MULTRET).is_err() {
        return Condition::Raised;
    }
    let nresults = stack.top().index() - top.index();
    if nresults == 0 {
        return Condition::Unmet;
    }
    let answer = top.above(1);
    if !stack.to_boolean(answer) {
        stack.set_top(top);
        return Condition::Unmet;
    }
    stack.remove(answer);
    Condition::Met(nresults - 1)
}

/// `vim.wait(timeout[, condition[, interval[, fast_only]]])`.
pub(crate) fn wait(stack: &mut Stack) -> Result<c_int, Raise> {
    if in_fast_callback.get() != 0 {
        return Err(Raise::message(fast_api_refusal(b"vim.wait")));
    }

    let timeout_number = stack.check_number(At::arg(1))?;
    if timeout_number < 0.0 {
        return Err(Raise::message("timeout must be >= 0"));
    }
    // A NaN or anything past `i64` waits as long as it can; `as` saturates
    // the rest the way the C cast did.
    #[allow(clippy::cast_possible_truncation)]
    let timeout: i64 = if timeout_number.is_nan() || timeout_number > i64::MAX as f64 {
        i64::MAX
    } else {
        timeout_number as i64
    };

    let lua_top = stack.top().index();

    // The condition may be a function or anything with a `__call`.
    let mut is_function = false;
    if lua_top >= 2 && stack.kind(At::arg(2)) != Kind::Nil {
        is_function = stack.kind(At::arg(2)) == Kind::Function;
        if !is_function && stack.get_metafield(At::arg(2), c"__call") {
            is_function = stack.kind(At::TOP) == Kind::Function;
            stack.pop(1);
        }
        if !is_function {
            stack.push_bytes(b"vim.wait: callback must be callable");
            return Err(Raise::top_value());
        }
    }

    let mut interval = DEFAULT_INTERVAL;
    if lua_top >= 3 && stack.kind(At::arg(3)) != Kind::Nil {
        let given = stack.check_integer(At::arg(3))?;
        let Ok(given) = u64::try_from(given) else {
            return Err(Raise::message("interval must be >= 0"));
        };
        interval = given;
    }

    let fast_only = lua_top >= 4 && stack.to_boolean(At::arg(4));
    let drain = if fast_only {
        Drain::FastOnly
    } else {
        Drain::Events
    };

    // The watcher is what makes the loop wake up on `interval`; it does
    // nothing else. It is dropped before anything below can raise.
    let waker = ScopedTimer::wake_every(interval);

    let mut outcome = Condition::Unmet;
    ui_flush();
    wait_on_main_loop(drain, timeout, || {
        if got_int.get() {
            return true;
        }
        if !is_function {
            return false;
        }
        outcome = wait_condition(stack);
        !matches!(outcome, Condition::Unmet)
    });

    drop(waker);

    match outcome {
        Condition::Raised => Err(Raise::top_value()),
        Condition::Met(nresults) => {
            stack.push_boolean(true);
            if nresults == 0 {
                stack.push_nil();
                Ok(2)
            } else {
                stack.insert(At::from_top(1 + nresults));
                Ok(nresults + 1)
            }
        }
        Condition::Unmet if got_int.get() => {
            // Swallow the interrupt: `vim.wait` reports it rather than
            // letting it escape into whatever called it.
            got_int.set(false);
            vgetc();
            stack.push_boolean(false);
            stack.push_integer(WAIT_INTERRUPTED);
            Ok(2)
        }
        Condition::Unmet => {
            stack.push_boolean(false);
            stack.push_integer(WAIT_TIMED_OUT);
            Ok(2)
        }
    }
}

/// `vim.in_fast_event()`.
pub(crate) fn in_fast_event(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.push_boolean(in_fast_callback.get() > 0);
    Ok(1)
}

/// Whether the named Vimscript builtin is marked `fast`, i.e. callable from
/// a fast callback.
pub(crate) fn viml_func_is_fast(name: &[u8]) -> bool {
    find_builtin(name).is_some_and(|fdef| fdef.fast)
}

/// Whether anything that is not `fast` may run right now.
pub fn nlua_is_deferred_safe() -> bool {
    in_fast_callback.get() == 0
}
