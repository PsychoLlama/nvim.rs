//! The remote UI protocol: attaching a channel, and what it is told.
//!
//! A channel becomes a UI by calling `nvim_ui_attach`, which is where the
//! [`RemoteUI`] below is created and handed to
//! [`ui`](crate::src::nvim::ui) to be added to the attach table. From then
//! on the editor announces every screen change through that table's sinks,
//! and each sink calls a serializer here to turn the event into bytes on
//! that one UI's channel.
//!
//! What lives where: the outgoing buffer and its msgpack framing in
//! [`packer`], one serializer per event in [`events`], the cell-run encoder
//! in [`line`], and the externalised-widget events in [`redraw`]. This file
//! keeps the `nvim_ui_*` API entry points and the option negotiation they
//! share, which is the only part a UI author calls directly.
//!
//! Every serializer takes a `*mut RemoteUI` rather than a reference: a UI
//! that cannot be written to disconnects itself from inside the write,
//! which frees the very struct the caller is iterating over.
//!
//! The `nvim_ui_*` entry points keep `extern "C"` because that is how
//! `tools/apigen` recognises an API function; nothing else here does.

#![deny(unsafe_op_in_unsafe_fn)]

mod events;
mod line;
mod packer;
mod redraw;

pub use events::*;
pub use line::remote_ui_raw_line;
pub use packer::remote_ui_flush_pending_data;
pub use redraw::{remote_ui_event, remote_ui_hl_attr_define};

use crate::src::nvim::api::private::helpers::{
    api_set_error, api_typename, cstr_as_string, string_to_cstr,
};
use crate::src::nvim::api::private::validate::{api_err_exp, api_err_invalid};
use crate::src::nvim::autocmd::{do_autocmd_focusgained, may_trigger_vim_suspend_resume};
use crate::src::nvim::channel::find_channel;
use crate::src::nvim::event::r#loop::process_events_until;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    current_ui, main_loop, starting, stdin_fd, stdin_isatty, stdout_isatty, t_colors, ui_ext_names,
};
use crate::src::nvim::memory::{strequal, xfree};
use crate::src::nvim::option::set_tty_option;
use crate::src::nvim::types::api::{kErrorTypeException, kErrorTypeNone, kErrorTypeValidation};
use crate::src::nvim::types::builders::{ArrayBuf, DictBuf};
use crate::src::nvim::types::{
    Boolean, Dict, Error, Float, Integer, Object, ObjectType, PackerBuffer, RemoteUI, String_0,
    UIExtension, handle_T, kObjectTypeBoolean, kObjectTypeInteger, kObjectTypeString,
};
use crate::src::nvim::ui::{
    kUICmdline, kUIExtCount, kUIHlState, kUILinegrid, kUIMessages, kUIMultigrid, kUIPopupmenu,
    ui_active, ui_attach_impl, ui_call_ui_send, ui_can_attach_more, ui_detach_impl, ui_grid_resize,
    ui_refresh, ui_set_ext_option,
};
use core::ffi::{CStr, c_char, c_int};

/// The channel id `--embed` and `nvim -` use, which is the only one whose
/// tty-ness is the editor's own.
const CHAN_STDIO: u64 = 1;

/// The handle of the grid every UI has, before `ext_multigrid` adds more.
const DEFAULT_GRID_HANDLE: Integer = 1;

/// Every attached UI, in attach order.
///
/// A list rather than a map keyed by channel: the table is bounded by
/// `MAX_UI_COUNT` and every operation on it is an attach, a detach or an
/// error path, none of which is worth a hashtable.
static connected_uis: GlobalCell<Vec<*mut RemoteUI>> = GlobalCell::new(Vec::new());

/// The UI attached to `chan_id`, or null with `err` set.
///
/// # Safety
///
/// `err` must be writable or null.
unsafe fn get_ui_or_err(chan_id: u64, err: *mut Error) -> *mut RemoteUI {
    let ui = find_ui(chan_id).unwrap_or(core::ptr::null_mut());
    if ui.is_null() && !err.is_null() {
        unsafe {
            api_set_error(
                err,
                kErrorTypeException,
                c"UI not attached to channel: %ld".as_ptr(),
                chan_id,
            )
        };
    }
    ui
}

/// The UI attached to `chan_id`, if there is one.
fn find_ui(chan_id: u64) -> Option<*mut RemoteUI> {
    connected_uis.with(|uis| {
        uis.iter()
            .copied()
            // SAFETY: every entry is live until it is removed here.
            .find(|&ui| unsafe { (*ui).channel_id } == chan_id)
    })
}

/// Releases a detached UI and everything it owns.
///
/// # Safety
///
/// `ui` must be live, detached, and unreferenced.
unsafe fn remote_ui_destroy(ui: *mut RemoteUI) {
    unsafe {
        // The pending block, if the UI went away mid-batch.
        xfree((*ui).packer.startptr.cast());
        xfree((*ui).term_name.cast());
        drop(Box::from_raw(ui));
    }
}

/// Detaches the UI on `channel_id`, optionally telling it why.
///
/// # Safety
///
/// `err` must be writable or null.
pub unsafe fn remote_ui_disconnect(channel_id: u64, err: *mut Error, send_error_exit: bool) {
    unsafe {
        let ui = get_ui_or_err(channel_id, err);
        if ui.is_null() {
            return;
        }
        if send_error_exit {
            // A UI told to exit is one whose server is going away, so this
            // has to go out before the channel does.
            let mut args = ArrayBuf::<1>::new();
            args.push(Object::integer(0));
            packer::push_call(ui, c"error_exit", args.array());
            packer::ui_flush_buf(ui, false);
        }
        connected_uis.with_mut(|uis| uis.retain(|&entry| entry != ui));
        ui_detach_impl(ui, channel_id);
        let chan = find_channel(channel_id);
        if !chan.is_null() && (*chan).rpc.ui == ui {
            (*chan).rpc.ui = core::ptr::null_mut();
        }
        remote_ui_destroy(ui);
    }
}

/// Pumps the event loop until some UI has attached.
///
/// Used at startup by `--embed` without `--headless`, where the editor must
/// not draw anything until it knows the terminal size.
///
/// # Safety
///
/// The main loop must be running.
pub unsafe fn remote_ui_wait_for_attach() {
    unsafe {
        process_events_until(main_loop.ptr(), (*main_loop.ptr()).events, -1, || {
            ui_active() != 0
        })
    };
}

/// Attaches the calling channel as a UI of `width` by `height` cells.
///
/// # Safety
///
/// `err` must be writable, and `options` valid for the duration.
pub unsafe extern "C" fn nvim_ui_attach(
    channel_id: u64,
    width: Integer,
    height: Integer,
    options: Dict,
    err: *mut Error,
) {
    unsafe {
        if find_ui(channel_id).is_some() {
            api_set_error(
                err,
                kErrorTypeException,
                c"UI already attached to channel: %ld".as_ptr(),
                channel_id,
            );
            return;
        }
        if !ui_can_attach_more() {
            api_set_error(
                err,
                kErrorTypeException,
                c"Maximum UI count reached".as_ptr(),
            );
            return;
        }
        if width <= 0 || height <= 0 {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"Expected width > 0 and height > 0".as_ptr(),
            );
            return;
        }

        let ui = Box::into_raw(Box::new(RemoteUI::new(channel_id, width, height)));
        // The packer reaches back to the UI to flush it when full, which is
        // only possible once the box has an address.
        (*ui).packer.anydata = ui.cast();

        for i in 0..options.size {
            let option = *options.items.add(i);
            ui_set_option(ui, true, option.key, option.value, err);
            if (*err).type_0 != kErrorTypeNone {
                // Nothing has been published yet, so the half-configured UI
                // can simply be dropped. `term_name` is the only owned
                // field an option sets, and not on an error path.
                drop(Box::from_raw(ui));
                return;
            }
        }

        // Options that imply others. A UI asking for anything the linegrid
        // protocol introduced is asking for the linegrid protocol; external
        // messages are drawn in a cmdline the UI must also own.
        if (*ui).ui_ext[kUIHlState as usize] || (*ui).ui_ext[kUIMultigrid as usize] {
            (*ui).ui_ext[kUILinegrid as usize] = true;
        }
        if (*ui).ui_ext[kUIMessages as usize] {
            (*ui).ui_ext[kUILinegrid as usize] = true;
            (*ui).ui_ext[kUICmdline as usize] = true;
        }

        connected_uis.with_mut(|uis| uis.push(ui));
        current_ui.set(channel_id);
        ui_attach_impl(ui, channel_id);
        let chan = find_channel(channel_id);
        if !chan.is_null() {
            (*chan).rpc.ui = ui;
        }
        may_trigger_vim_suspend_resume(false);
    }
}

impl RemoteUI {
    /// A UI with nothing negotiated yet: RGB on, every extension off, no
    /// buffer, and the legacy cursor column marked unknown.
    fn new(channel_id: u64, width: Integer, height: Integer) -> Self {
        Self {
            rgb: true,
            override_0: false,
            composed: false,
            ui_ext: [false; kUIExtCount as usize],
            width: width as c_int,
            height: height as c_int,
            pum_nlines: 0,
            pum_pos: false,
            // Negative means the UI has not reported where its popupmenu
            // is, so the editor places it itself.
            pum_row: -1.0,
            pum_col: -1.0,
            pum_height: 0.0,
            pum_width: 0.0,
            term_name: core::ptr::null_mut(),
            term_colors: 0,
            stdin_tty: false,
            stdout_tty: false,
            channel_id,
            packer: PackerBuffer {
                startptr: core::ptr::null_mut(),
                ptr: core::ptr::null_mut(),
                endptr: core::ptr::null_mut(),
                anydata: core::ptr::null_mut(),
                anyint: 0,
                packer_flush: Some(packer::ui_flush_callback),
            },
            cur_event: core::ptr::null(),
            nevents_pos: core::ptr::null_mut(),
            ncalls_pos: core::ptr::null_mut(),
            nevents: 0,
            ncalls: 0,
            flushed_events: false,
            incomplete_event: false,
            ncells_pending: 0,
            hl_id: 0,
            cursor_row: 0,
            cursor_col: 0,
            client_row: 0,
            client_col: -1,
            wildmenu_active: false,
        }
    }
}

/// [`nvim_ui_attach`] with only the `rgb` option, kept for the deprecated
/// `ui_attach` API name.
///
/// # Safety
///
/// As [`nvim_ui_attach`].
pub unsafe extern "C" fn ui_attach(
    channel_id: u64,
    width: Integer,
    height: Integer,
    enable_rgb: Boolean,
    err: *mut Error,
) {
    let mut opts = DictBuf::<1>::new();
    opts.insert(c"rgb", Object::boolean(enable_rgb));
    unsafe { nvim_ui_attach(channel_id, width, height, opts.dict(), err) };
}

/// Tells the editor that this UI gained or lost the user's attention.
///
/// # Safety
///
/// `error` must be writable.
pub unsafe extern "C" fn nvim_ui_set_focus(channel_id: u64, gained: Boolean, error: *mut Error) {
    unsafe {
        if get_ui_or_err(channel_id, error).is_null() {
            return;
        }
        if gained {
            // Whichever UI was focused last is the one `nvim_get_current_ui`
            // means and the one a `:suspend` applies to.
            current_ui.set(channel_id);
            may_trigger_vim_suspend_resume(false);
        }
        do_autocmd_focusgained(gained);
    }
}

/// Detaches the UI on `channel_id`.
///
/// # Safety
///
/// `err` must be writable.
pub unsafe extern "C" fn nvim_ui_detach(channel_id: u64, err: *mut Error) {
    unsafe { remote_ui_disconnect(channel_id, err, false) };
}

/// Tells a UI to reconnect to `server_addr`.
///
/// Sent by `:restart`, where the server this UI is talking to is about to
/// be replaced by one listening elsewhere.
///
/// # Safety
///
/// `err` must be writable and `server_addr` a valid C string.
pub unsafe fn remote_ui_connect(channel_id: u64, server_addr: *mut c_char, err: *mut Error) {
    unsafe {
        let ui = get_ui_or_err(channel_id, err);
        if ui.is_null() {
            return;
        }
        let mut args = ArrayBuf::<1>::new();
        args.push(Object::string(cstr_as_string(server_addr)));
        packer::push_call(ui, c"connect", args.array());
    }
}

/// Reports that this UI's window is now `width` by `height` cells.
///
/// # Safety
///
/// `err` must be writable.
pub unsafe extern "C" fn nvim_ui_try_resize(
    channel_id: u64,
    width: Integer,
    height: Integer,
    err: *mut Error,
) {
    unsafe {
        let ui = get_ui_or_err(channel_id, err);
        if ui.is_null() {
            return;
        }
        if width <= 0 || height <= 0 {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"Expected width > 0 and height > 0".as_ptr(),
            );
            return;
        }
        (*ui).width = width as c_int;
        (*ui).height = height as c_int;
        // The screen is the smallest attached UI, so one UI resizing can
        // change what every other one is sent.
        ui_refresh();
    }
}

/// Changes one negotiated option after attaching.
///
/// # Safety
///
/// `error` must be writable and `value` valid for the duration.
pub unsafe extern "C" fn nvim_ui_set_option(
    channel_id: u64,
    name: String_0,
    value: Object,
    error: *mut Error,
) {
    let ui = unsafe { get_ui_or_err(channel_id, error) };
    if ui.is_null() {
        return;
    }
    unsafe { ui_set_option(ui, false, name, value, error) };
}

/// Applies one option to `ui`.
///
/// `init` distinguishes the options passed to `nvim_ui_attach`, which are
/// applied to a UI the editor has not seen yet, from later changes, which
/// have to be published.
///
/// # Safety
///
/// `ui` must be live, `err` writable, and `value` valid for the duration.
unsafe fn ui_set_option(
    ui: *mut RemoteUI,
    init: bool,
    name: String_0,
    value: Object,
    err: *mut Error,
) {
    // Every branch reads `value`'s union and writes through `ui`; the whole
    // body is one unsafe region rather than thirty.
    unsafe {
        // `name.data` can be null, which `strequal` treats as no match; a
        // `CStr` conversion here would not survive it.
        let named = |want: &CStr| strequal(name.data, want.as_ptr());

        if named(c"override") {
            if wrong_type(err, c"override".as_ptr(), kObjectTypeBoolean, value) {
                return;
            }
            // Asks for the highest capabilities any UI requested rather
            // than the intersection, for UIs that can cope with anything.
            (*ui).override_0 = value.data.boolean;
            return;
        }

        if named(c"rgb") {
            if wrong_type(err, c"rgb".as_ptr(), kObjectTypeBoolean, value) {
                return;
            }
            (*ui).rgb = value.data.boolean;
            // Only the legacy protocol bakes the colour model into what it
            // is sent; a linegrid UI gets both and picks.
            if !init && !(*ui).ui_ext[kUILinegrid as usize] {
                ui_refresh();
            }
            return;
        }

        if named(c"term_name") {
            if wrong_type(err, c"term_name".as_ptr(), kObjectTypeString, value) {
                return;
            }
            // 'term' is global, so the last UI to say what terminal it is
            // wins; the copy on the UI is what `nvim_list_uis` reports.
            set_tty_option(c"term".as_ptr(), string_to_cstr(value.data.string));
            (*ui).term_name = string_to_cstr(value.data.string);
            return;
        }

        if named(c"term_colors") {
            if wrong_type(err, c"term_colors".as_ptr(), kObjectTypeInteger, value) {
                return;
            }
            t_colors.set(value.data.integer as c_int);
            (*ui).term_colors = value.data.integer as c_int;
            return;
        }

        if named(c"stdin_fd") {
            if wrong_type(err, c"stdin_fd".as_ptr(), kObjectTypeInteger, value) {
                return;
            }
            let fd = value.data.integer;
            if fd < 0 {
                api_err_invalid(err, c"stdin_fd".as_ptr(), core::ptr::null(), fd, false);
                return;
            }
            // The editor reads its startup input from this descriptor,
            // which only means anything before startup has finished.
            if starting.get() != 2 {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"%s".as_ptr(),
                    c"stdin_fd can only be used with first attached UI".as_ptr(),
                );
                return;
            }
            stdin_fd.set(fd as c_int);
            return;
        }

        if named(c"stdin_tty") {
            if wrong_type(err, c"stdin_tty".as_ptr(), kObjectTypeBoolean, value) {
                return;
            }
            // Only the stdio channel is talking about the editor's own
            // standard streams.
            if (*ui).channel_id == CHAN_STDIO {
                stdin_isatty.set(value.data.boolean);
            }
            (*ui).stdin_tty = value.data.boolean;
            return;
        }

        if named(c"stdout_tty") {
            if wrong_type(err, c"stdout_tty".as_ptr(), kObjectTypeBoolean, value) {
                return;
            }
            if (*ui).channel_id == CHAN_STDIO {
                stdout_isatty.set(value.data.boolean);
            }
            (*ui).stdout_tty = value.data.boolean;
            return;
        }

        // The extensions, by their protocol names. `popupmenu_external` is
        // the pre-0.3 spelling of `ext_popupmenu` and still accepted.
        let is_popupmenu = named(c"popupmenu_external");
        for ext in 0..kUIExtCount as usize {
            if !strequal(name.data, ui_ext_names.get()[ext])
                && !(ext == kUIPopupmenu as usize && is_popupmenu)
            {
                continue;
            }
            if value.type_0 != kObjectTypeBoolean {
                api_err_exp(
                    err,
                    name.data,
                    c"Boolean".as_ptr(),
                    api_typename(value.type_0),
                );
                return;
            }
            let active = value.data.boolean;
            // Which protocol a UI speaks is decided at attach: the editor
            // has already sent it events in that protocol's shape.
            if !init && ext == kUILinegrid as usize && active != (*ui).ui_ext[ext] {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"ext_linegrid option cannot be changed".as_ptr(),
                );
            }
            (*ui).ui_ext[ext] = active;
            if !init {
                ui_set_ext_option(ui, ext as UIExtension, active);
            }
            return;
        }

        api_err_invalid(err, c"UI option".as_ptr(), name.data, 0, true);
    }
}

/// Reports that `value` is not the type `name` takes, and whether so.
///
/// # Safety
///
/// `err` must be writable and `name` a valid C string.
unsafe fn wrong_type(
    err: *mut Error,
    name: *const c_char,
    expected: ObjectType,
    value: Object,
) -> bool {
    if value.type_0 == expected {
        return false;
    }
    unsafe {
        api_err_exp(
            err,
            name,
            api_typename(expected),
            api_typename(value.type_0),
        )
    };
    true
}

/// Resizes one grid, for a UI with `ext_multigrid`.
///
/// # Safety
///
/// `err` must be writable.
pub unsafe extern "C" fn nvim_ui_try_resize_grid(
    channel_id: u64,
    grid: Integer,
    width: Integer,
    height: Integer,
    err: *mut Error,
) {
    unsafe {
        if get_ui_or_err(channel_id, err).is_null() {
            return;
        }
        if grid == DEFAULT_GRID_HANDLE {
            // The default grid is the screen, so resizing it is a window
            // resize like any other.
            nvim_ui_try_resize(channel_id, width, height, err);
        } else {
            ui_grid_resize(grid as handle_T, width as c_int, height as c_int, err);
        }
    }
}

/// Tells the editor how many lines this UI's popupmenu can show.
///
/// # Safety
///
/// `err` must be writable.
pub unsafe extern "C" fn nvim_ui_pum_set_height(channel_id: u64, height: Integer, err: *mut Error) {
    unsafe {
        let ui = get_ui_or_err(channel_id, err);
        if ui.is_null() {
            return;
        }
        if height <= 0 {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"Expected pum height > 0".as_ptr(),
            );
            return;
        }
        if !(*ui).ui_ext[kUIPopupmenu as usize] {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"UI must support the ext_popupmenu option".as_ptr(),
            );
            return;
        }
        (*ui).pum_nlines = height as c_int;
    }
}

/// Tells the editor where this UI drew its popupmenu, so that `pumvisible()`
/// and the completion logic can reason about the screen area it covers.
///
/// # Safety
///
/// `err` must be writable.
pub unsafe extern "C" fn nvim_ui_pum_set_bounds(
    channel_id: u64,
    width: Float,
    height: Float,
    row: Float,
    col: Float,
    err: *mut Error,
) {
    unsafe {
        let ui = get_ui_or_err(channel_id, err);
        if ui.is_null() {
            return;
        }
        if !(*ui).ui_ext[kUIPopupmenu as usize] {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"UI must support the ext_popupmenu option".as_ptr(),
            );
            return;
        }
        if width <= 0.0 {
            api_set_error(err, kErrorTypeValidation, c"Expected width > 0".as_ptr());
            return;
        }
        if height <= 0.0 {
            api_set_error(err, kErrorTypeValidation, c"Expected height > 0".as_ptr());
            return;
        }
        (*ui).pum_row = row;
        (*ui).pum_col = col;
        (*ui).pum_width = width;
        (*ui).pum_height = height;
        (*ui).pum_pos = true;
    }
}

/// Forwards `content` to every UI that owns a terminal, as `ui_send`.
///
/// # Safety
///
/// `content` must be valid for the duration of the call.
pub unsafe extern "C" fn nvim_ui_send(_channel_id: u64, content: String_0, _err: *mut Error) {
    ui_call_ui_send(content);
}
