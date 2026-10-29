//! `vim.ui_attach()`: Lua handlers for UI events.
//!
//! A namespace registers one callback and the set of external widgets it
//! wants to draw itself. Registering makes the editor behave as though a UI
//! had asked for those widgets — [`ui_cb_ext`] is OR'd into the real UIs'
//! extension set by [`ui_refresh`](super::ui_refresh) — so `cmdline_show`
//! and friends start being emitted even when no attached UI asked for them.
//!
//! Everything routed through here arrives as a `(name, args)` pair rather
//! than a typed call, because a handler that returns `true` consumes the
//! event and no UI sees it at all. That is why the sinks in
//! [`sinks`](super::sinks) are split the way they are: the ones a Lua
//! handler may intercept pay for an [`Object`] array, the rest do not.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::sinks::log_event;
use super::{Reach, each_ui_reached, ui_refresh};
use crate::api::extmark::describe_ns;
use crate::api::ui::remote_ui_event;
use crate::global_cell::GlobalCell;
use crate::guard::Allow;
use crate::log::{LOGLVL_ERR, logmsg};
use crate::lua::executor::{nlua_call_ref_ctx, release_luaref};
use crate::message_fmt::{msg_bytes, msg_cstr};
use crate::msg_schedule_semsg;
use crate::msg_schedule_semsg_multiline;
use crate::types::ui::{kUICmdline, kUILinegrid, kUIMessages};
use crate::types::{Array, LuaRef, LuaRetMode};
use crate::ui::state::ui_event_ns_id;
use core::ffi::CStr;

const kRetNilBool: LuaRetMode = 1;

/// How many times a handler may raise an error before it is unregistered.
const MAX_ERRORS: u8 = 3;

/// One registered `vim.ui_attach()` handler.
struct Handler {
    /// The namespace that registered it, and the key it is removed by.
    ns_id: u32,
    callback: LuaRef,
    /// Errors raised so far. The handler is dropped past [`MAX_ERRORS`],
    /// because a handler that throws on every redraw makes the editor
    /// unusable and there is no other way to stop it.
    errors: u8,
    /// The widgets this handler draws, indexed by [`UIExtension`]. Only the
    /// external widgets have entries — everything from `ext_linegrid` on is
    /// about how a UI wants the screen described, not about who draws it.
    ///
    /// [`UIExtension`]: crate::types::UIExtension
    ext_widgets: [bool; kUILinegrid as usize],
}

/// The registered handlers, in registration order.
///
/// A `Vec` rather than a map: there are rarely more than a handful, and the
/// order they run in is then something a plugin author can predict.
static registered: GlobalCell<Vec<Handler>> = GlobalCell::new(Vec::new());

/// The union of every handler's [`Handler::ext_widgets`].
pub static ui_cb_ext: GlobalCell<[bool; 10]> = GlobalCell::new([false; 10]);

/// Registers `cb` for `ns_id`, replacing any handler it already had;
/// `ext_widgets` says which of the first [`kUILinegrid`] extensions it draws.
pub fn ui_add_cb(ns_id: u32, cb: LuaRef, mut ext_widgets: [bool; kUILinegrid as usize]) {
    // `ext_messages` moves the whole cmdline out of the message area, so a
    // handler drawing messages has to draw the cmdline too.
    if ext_widgets[kUIMessages as usize] {
        ext_widgets[kUICmdline as usize] = true;
    }
    let handler = Handler {
        ns_id,
        callback: cb,
        errors: 0,
        ext_widgets,
    };
    registered.with_mut(
        |handlers| match handlers.iter_mut().find(|h| h.ns_id == ns_id) {
            Some(existing) => {
                release_luaref(existing.callback);
                *existing = handler;
            }
            None => handlers.push(handler),
        },
    );
    update_ext();
    ui_refresh();
}

/// Unregisters `ns_id`'s handler.
///
/// With `checkerr`, this is the error path: the handler's error count goes
/// up and it is only dropped once it passes [`MAX_ERRORS`].
pub fn ui_remove_cb(ns_id: u32, checkerr: bool) {
    let removed = registered.with_mut(|handlers| {
        let index = handlers.iter().position(|h| h.ns_id == ns_id)?;
        if checkerr {
            handlers[index].errors += 1;
            if handlers[index].errors <= MAX_ERRORS {
                return None;
            }
        }
        Some(handlers.remove(index))
    });
    let Some(handler) = removed else {
        return;
    };
    release_luaref(handler.callback);
    update_ext();
    ui_refresh();
    if checkerr {
        let ns = describe_ns(ns_id.cast_signed(), b"(UNKNOWN PLUGIN)");
        let ns = msg_bytes(&ns);
        msg_schedule_semsg!("Excessive errors in vim.ui_attach() callback (ns={ns})");
    }
}

/// Recomputes [`ui_cb_ext`] from the registered handlers.
fn update_ext() {
    let mut ext = [false; 10];
    registered.with(|handlers| {
        for (widget, slot) in ext[..kUILinegrid as usize].iter_mut().enumerate() {
            *slot = handlers.iter().any(|h| h.ext_widgets[widget]);
        }
    });
    ui_cb_ext.set(ext);
}

/// Offers `name` to every registered handler, then to the attached UIs
/// unless a handler claimed it by returning `true`.
pub fn ui_call_event(name: &'static CStr, args: Array) {
    let handled = offer_to_handlers(name, &args);
    if !handled {
        let any_call = each_ui_reached(Reach::All, |ui| remote_ui_event(ui, name, args.clone()));
        if any_call {
            log_event(c"event");
        }
    }
    log_event(name);
}

/// Runs the handlers, returning whether one of them claimed the event.
fn offer_to_handlers(name: &CStr, args: &Array) -> bool {
    // A handler is arbitrary Lua and may legitimately want to move the
    // cursor or set a variable, which the locks held while redrawing would
    // forbid. Upstream lifts them for the duration and puts them back.
    let _unlocked_expr_map = Allow::expr_map();
    let _unlocked_text = Allow::text_changes();

    // Snapshot the namespaces rather than iterating the live list: a
    // handler can register or unregister handlers, including its own.
    let namespaces: Vec<u32> =
        registered.with(|handlers| handlers.iter().map(|h| h.ns_id).collect());
    let mut handled = false;
    for ns_id in namespaces {
        let Some(callback) = registered.with(|handlers| {
            handlers
                .iter()
                .find(|h| h.ns_id == ns_id)
                .map(|h| h.callback)
        }) else {
            continue;
        };
        ui_event_ns_id.set(ns_id);
        let fast = is_fast(name, args);
        let res = nlua_call_ref_ctx(fast, callback, Some(name), args.clone(), kRetNilBool, true);
        ui_event_ns_id.set(0);
        match res {
            Ok(res) => {
                if res.as_boolean() == Some(true) {
                    handled = true;
                }
            }
            Err(e) => {
                report_error(ns_id, name, e.message_or_empty());
                ui_remove_cb(ns_id, true);
            }
        }
    }

    handled
}

/// Whether the handler may run on the fast event loop, where the editor
/// state is not safe to touch.
///
/// Only `msg_show` qualifies, and only for the kinds a redraw produces on
/// its own. The rest are raised by something the user asked for, so the
/// handler is allowed to be a full-fat callback.
fn is_fast(name: &CStr, args: &Array) -> bool {
    /// `msg_show` kinds that are not redraw-driven.
    const SLOW_KINDS: [&CStr; 12] = [
        c"empty",
        c"echo",
        c"echomsg",
        c"echoerr",
        c"list_cmd",
        c"lua_error",
        c"lua_print",
        c"progress",
        c"shell_cmd",
        c"shell_err",
        c"shell_out",
        c"shell_ret",
    ];
    if name != c"msg_show" {
        return false;
    }
    // `kind` is `msg_show`'s first argument, and an unkinded message
    // carries it as an empty string with no buffer behind it at all.
    let kind = args[0]
        .as_string()
        .expect("`msg_show`'s first argument is its kind");
    if kind.is_null() {
        return true;
    }
    !SLOW_KINDS.contains(&kind.as_cstr())
}

/// Reports that `ns_id`'s handler for `name` raised `msg`.
fn report_error(ns_id: u32, name: &CStr, msg: &CStr) {
    let ns = describe_ns(ns_id.cast_signed(), b"(UNKNOWN PLUGIN)");
    let shown_ns = msg_bytes(&ns);
    let (shown_name, shown_msg) = (msg_cstr(name), msg_cstr(msg));
    logmsg!(
        LOGLVL_ERR,
        c"report_error",
        line!().cast_signed(),
        "Error in \"{shown_name}\" UI event handler (ns={shown_ns}):\n{shown_msg}"
    );
    msg_schedule_semsg_multiline!(
        "Error in \"{shown_name}\" UI event handler (ns={shown_ns}):\n{shown_msg}"
    );
}
