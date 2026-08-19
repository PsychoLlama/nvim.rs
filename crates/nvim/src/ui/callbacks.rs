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

#![deny(unsafe_op_in_unsafe_fn)]

use super::sinks::log_event;
use super::{ui_at, ui_count, ui_refresh};
use crate::api::extmark::describe_ns;
use crate::api::private::helpers::api_clear_error;
use crate::api::ui::remote_ui_event;
use crate::global_cell::GlobalCell;
use crate::guard::Allow;
use crate::log::{LOGLVL_ERR, logmsg_c};
use crate::lua::executor::{api_free_luaref, nlua_call_ref_ctx};
use crate::main::ui_event_ns_id;
use crate::types::ui::{kUICmdline, kUILinegrid, kUIMessages};
use crate::types::{
    Arena, Array, Error, LuaRef, LuaRetMode, NS, kErrorTypeNone, kObjectTypeBoolean,
};
use crate::{msg_schedule_semsg_c, msg_schedule_semsg_multiline_c};
use core::ffi::{CStr, c_char};

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

/// Registers `cb` for `ns_id`, replacing any handler it already had.
///
/// # Safety
///
/// `ext_widgets` must address [`kUILinegrid`] readable `bool`s.
pub unsafe fn ui_add_cb(ns_id: u32, cb: LuaRef, ext_widgets: *mut bool) {
    let mut ext_widgets: [bool; kUILinegrid as usize] =
        unsafe { core::slice::from_raw_parts(ext_widgets, kUILinegrid as usize) }
            .try_into()
            .expect("slice is the array's length");
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
                unsafe { api_free_luaref(existing.callback) };
                *existing = handler;
            }
            None => handlers.push(handler),
        },
    );
    update_ext();
    unsafe { ui_refresh() };
}

/// Unregisters `ns_id`'s handler.
///
/// With `checkerr`, this is the error path: the handler's error count goes
/// up and it is only dropped once it passes [`MAX_ERRORS`].
pub unsafe fn ui_remove_cb(ns_id: u32, checkerr: bool) {
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
    unsafe { api_free_luaref(handler.callback) };
    update_ext();
    unsafe { ui_refresh() };
    if checkerr {
        let ns = unsafe { describe_ns(ns_id as NS, c"(UNKNOWN PLUGIN)".as_ptr()) };
        unsafe {
            msg_schedule_semsg_c!(
                c"Excessive errors in vim.ui_attach() callback (ns=%s)".as_ptr(),
                ns,
            )
        };
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
///
/// # Safety
///
/// `args` must be a valid array for the duration of the call; the handlers
/// and the serializers both read it.
pub unsafe fn ui_call_event(name: &'static CStr, args: Array) {
    let handled = unsafe { offer_to_handlers(name, args) };
    if !handled {
        let mut any_call = false;
        let mut i = 0;
        while i < ui_count() {
            unsafe { remote_ui_event(ui_at(i), name, args) };
            any_call = true;
            i += 1;
        }
        if any_call {
            log_event(c"event");
        }
    }
    log_event(name);
}

/// Runs the handlers, returning whether one of them claimed the event.
///
/// # Safety
///
/// As [`ui_call_event`].
unsafe fn offer_to_handlers(name: &CStr, args: Array) -> bool {
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
        let mut err = Error {
            type_0: kErrorTypeNone,
            msg: core::ptr::null_mut(),
        };
        ui_event_ns_id.set(ns_id);
        let res = unsafe {
            nlua_call_ref_ctx(
                is_fast(name, args),
                callback,
                name.as_ptr().cast_mut(),
                args,
                kRetNilBool,
                core::ptr::null_mut::<Arena>(),
                &raw mut err,
            )
        };
        ui_event_ns_id.set(0);
        if res.type_0 == kObjectTypeBoolean && unsafe { res.data.boolean } {
            handled = true;
        }
        if err.type_0 != kErrorTypeNone {
            unsafe {
                report_error(ns_id, name.as_ptr(), err.msg);
                ui_remove_cb(ns_id, true);
            }
        }
        unsafe { api_clear_error(&raw mut err) };
    }

    handled
}

/// Whether the handler may run on the fast event loop, where the editor
/// state is not safe to touch.
///
/// Only `msg_show` qualifies, and only for the kinds a redraw produces on
/// its own. The rest are raised by something the user asked for, so the
/// handler is allowed to be a full-fat callback.
///
/// # Safety
///
/// As [`ui_call_event`].
unsafe fn is_fast(name: &CStr, args: Array) -> bool {
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
    let kind = unsafe { (*args.items).data.string.data() };
    if kind.is_null() {
        return true;
    }
    let kind = unsafe { CStr::from_ptr(kind) };
    !SLOW_KINDS.contains(&kind)
}

/// # Safety
///
/// `name` and `msg` must be valid C strings.
unsafe fn report_error(ns_id: u32, name: *const c_char, msg: *const c_char) {
    let ns = unsafe { describe_ns(ns_id as NS, c"(UNKNOWN PLUGIN)".as_ptr()) };
    let format = c"Error in \"%s\" UI event handler (ns=%s):\n%s".as_ptr();
    unsafe {
        logmsg_c!(
            LOGLVL_ERR,
            core::ptr::null(),
            c"report_error".as_ptr(),
            line!() as i32,
            true,
            format,
            name,
            ns,
            msg,
        );
        msg_schedule_semsg_multiline_c!(format, name, ns, msg);
    }
}
