//! The remote UI protocol: attaching a channel, and what it is told.
//!
//! A channel becomes a UI by calling `nvim_ui_attach`, which is where the
//! [`RemoteUI`] below is created and handed to
//! [`ui`](crate::ui) to be added to the attach table. From then
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

use crate::api::private::helpers::{
    ERROR_INIT, Reported, api_typename, cstr_as_string, string_to_cstr,
};
use crate::api::private::validate::{err_expected_ptr, err_invalid_ptr};
use crate::api_error;
use crate::autocmd::{do_autocmd_focusgained, may_trigger_vim_suspend_resume};
use crate::channel::find_channel;
use crate::event::r#loop::process_events_until;
use crate::global_cell::GlobalCell;
use crate::main::{
    current_ui, main_loop, starting, stdin_fd, stdin_isatty, stdout_isatty, t_colors, ui_ext_names,
};
use crate::memory::{strequal, xfree};
use crate::option::set_tty_option;
use crate::types::builders::{ArrayBuf, DictBuf};
use crate::types::ui::{
    kUICmdline, kUIExtCount, kUIHlState, kUILinegrid, kUIMessages, kUIMultigrid, kUIPopupmenu,
};
use crate::types::{
    Boolean, Dict, Error, Float, Integer, Object, ObjectType, PackerBuffer, RemoteUI, String_0,
    UIExtension, handle_T, kErrorTypeException, kObjectTypeBoolean, kObjectTypeInteger,
    kObjectTypeString,
};
use crate::ui::{
    ui_active, ui_attach_impl, ui_call_ui_send, ui_can_attach_more, ui_detach_impl, ui_grid_resize,
    ui_refresh, ui_set_ext_option,
};
use crate::winlayer::Live;
use core::ffi::{CStr, c_char, c_int};

/// One attached UI, with checked field access.
///
/// Every serializer is handed the raw pointer rather than a reference, for
/// the reason the module docs give. `Live` is the reader's half of that:
/// it hands out a borrow for exactly the length of one field access, so no
/// call ever runs with one outstanding.
type Ui = Live<RemoteUI>;

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
fn get_ui_or_err(chan_id: u64, err: &mut Error) -> *mut RemoteUI {
    let ui = find_ui(chan_id).unwrap_or(core::ptr::null_mut());
    if ui.is_null() {
        *err = api_error!(kErrorTypeException, "UI not attached to channel: {chan_id}");
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
    // SAFETY: the caller's promise -- `ui` is the `Box::into_raw` pointer
    // `nvim_ui_attach` made, and nothing else names it any more.
    let ui = unsafe { Box::from_raw(ui) };
    // The pending block, if the UI went away mid-batch, and the terminal
    // name an option set: the two allocations the struct itself owns.
    // SAFETY: both are this UI's own, and it is about to be dropped.
    unsafe { xfree(ui.packer.startptr.cast()) };
    // SAFETY: as above.
    unsafe { xfree(ui.term_name.cast()) };
}

/// Detaches the UI on `channel_id`, optionally telling it why.
///
/// # Safety
///
/// `err` must be writable or null.
pub unsafe fn remote_ui_disconnect(channel_id: u64, err: &mut Error, send_error_exit: bool) {
    let ui = get_ui_or_err(channel_id, err);
    if ui.is_null() {
        return;
    }
    if send_error_exit {
        // A UI told to exit is one whose server is going away, so this
        // has to go out before the channel does.
        let mut args = ArrayBuf::<1>::new();
        args.push(Object::integer(0));
        // SAFETY: `ui` is in the attach table, so it is live until the
        // `remote_ui_destroy` below.
        unsafe {
            packer::push_call(ui, c"error_exit", args.array());
            packer::ui_flush_buf(ui, false);
        }
    }
    connected_uis.with_mut(|uis| uis.retain(|&entry| entry != ui));
    // SAFETY: as above -- `ui` is live, and now out of the attach table.
    unsafe { ui_detach_impl(ui, channel_id) };
    let chan = find_channel(channel_id);
    // SAFETY: `find_channel` answered null or a live channel, and nothing
    // has run the event loop since.
    if !chan.is_null() && unsafe { (*chan).rpc.ui } == ui {
        // SAFETY: as above.
        unsafe { (*chan).rpc.ui = core::ptr::null_mut() };
    }
    // SAFETY: `ui` is detached and nothing references it any more.
    unsafe { remote_ui_destroy(ui) };
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
    let loop_0 = main_loop.ptr();
    // SAFETY: the caller's promise -- the main loop is running, so its
    // event queue is live.
    let events = unsafe { (*loop_0).events };
    // SAFETY: as above.
    unsafe { process_events_until(loop_0, events, -1, || ui_active() != 0) };
}

/// Attaches the calling channel as a UI of `width` by `height` cells.
///
/// # Safety
///
/// `err` must be writable, and `options` valid for the duration.
pub unsafe fn nvim_ui_attach(
    channel_id: u64,
    width: Integer,
    height: Integer,
    options: Dict,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    if find_ui(channel_id).is_some() {
        error = api_error!(
            kErrorTypeException,
            "UI already attached to channel: {channel_id}"
        );
        return ().reported(error);
    }
    if !ui_can_attach_more() {
        error = Error::exception(c"Maximum UI count reached");
        return ().reported(error);
    }
    if width <= 0 || height <= 0 {
        error = Error::validation(c"Expected width > 0 and height > 0");
        return ().reported(error);
    }

    let raw = Box::into_raw(Box::new(RemoteUI::new(channel_id, width, height)));
    // SAFETY: `raw` is the box just made, live until it is handed to the
    // attach table or dropped below.
    let mut ui = unsafe { Ui::new(raw) };
    // The packer reaches back to the UI to flush it when full, which is
    // only possible once the box has an address.
    ui.packer.anydata = raw.cast();

    for i in 0..options.size {
        // SAFETY: `i` is below `size`, so the slot is inside `items`.
        let option = unsafe { *options.items.add(i) };
        // SAFETY: `raw` is live, `error` is this frame's slot, and the value
        // lives as long as the caller's dictionary.
        unsafe { ui_set_option(raw, true, option.key, option.value, &mut error) };
        if error.is_set() {
            // Nothing has been published yet, so the half-configured UI
            // can simply be dropped. `term_name` is the only owned
            // field an option sets, and not on an error path.
            // SAFETY: nothing else names `raw` yet.
            drop(unsafe { Box::from_raw(raw) });
            return ().reported(error);
        }
    }

    // Options that imply others. A UI asking for anything the linegrid
    // protocol introduced is asking for the linegrid protocol; external
    // messages are drawn in a cmdline the UI must also own.
    if ui.ui_ext[kUIHlState as usize] || ui.ui_ext[kUIMultigrid as usize] {
        ui.ui_ext[kUILinegrid as usize] = true;
    }
    if ui.ui_ext[kUIMessages as usize] {
        ui.ui_ext[kUILinegrid as usize] = true;
        ui.ui_ext[kUICmdline as usize] = true;
    }

    connected_uis.with_mut(|uis| uis.push(raw));
    current_ui.set(channel_id);
    // SAFETY: `raw` is live and now in the attach table.
    unsafe { ui_attach_impl(raw, channel_id) };
    let chan = find_channel(channel_id);
    // SAFETY: `find_channel` answered null or a live channel, and nothing
    // has run the event loop since.
    if !chan.is_null() {
        unsafe { (*chan).rpc.ui = raw };
    }
    may_trigger_vim_suspend_resume(false);
    ().reported(error)
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
pub unsafe fn ui_attach(
    channel_id: u64,
    width: Integer,
    height: Integer,
    enable_rgb: Boolean,
) -> Result<(), Error> {
    let mut opts = DictBuf::<1>::new();
    opts.insert(c"rgb", Object::boolean(enable_rgb));
    let opts = opts.dict();
    // SAFETY: the caller's promise, and `opts` outlives the call.
    unsafe { nvim_ui_attach(channel_id, width, height, opts) }
}

/// Tells the editor that this UI gained or lost the user's attention.
///
/// # Safety
///
/// The editor must be running.
pub unsafe fn nvim_ui_set_focus(channel_id: u64, gained: Boolean) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    if get_ui_or_err(channel_id, &mut error).is_null() {
        return Err(error);
    }
    if gained {
        // Whichever UI was focused last is the one `nvim_get_current_ui`
        // means and the one a `:suspend` applies to.
        current_ui.set(channel_id);
        may_trigger_vim_suspend_resume(false);
    }
    do_autocmd_focusgained(gained);
    Ok(())
}

/// Detaches the UI on `channel_id`.
///
/// # Safety
///
/// `err` must be writable.
pub unsafe fn nvim_ui_detach(channel_id: u64) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    unsafe { remote_ui_disconnect(channel_id, &mut error, false) };
    ().reported(error)
}

/// Tells a UI to reconnect to `server_addr`.
///
/// Sent by `:restart`, where the server this UI is talking to is about to
/// be replaced by one listening elsewhere.
///
/// # Safety
///
/// `err` must be writable and `server_addr` a valid C string.
pub unsafe fn remote_ui_connect(channel_id: u64, server_addr: *mut c_char, err: &mut Error) {
    let ui = get_ui_or_err(channel_id, err);
    if ui.is_null() {
        return;
    }
    let mut args = ArrayBuf::<1>::new();
    // SAFETY: the caller's promise -- `server_addr` is a C string, and the
    // borrowed view of it does not outlive this call.
    args.push(Object::string(unsafe { cstr_as_string(server_addr) }));
    // SAFETY: `ui` is in the attach table, so it is live.
    unsafe { packer::push_call(ui, c"connect", args.array()) };
}

/// Reports that this UI's window is now `width` by `height` cells.
///
/// # Safety
///
/// `err` must be writable.
pub unsafe fn nvim_ui_try_resize(
    channel_id: u64,
    width: Integer,
    height: Integer,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let ui = get_ui_or_err(channel_id, &mut error);
    if ui.is_null() {
        return ().reported(error);
    }
    if width <= 0 || height <= 0 {
        error = Error::validation(c"Expected width > 0 and height > 0");
        return ().reported(error);
    }
    // SAFETY: `ui` is in the attach table, so it is live.
    let mut ui = unsafe { Ui::new(ui) };
    ui.width = width as c_int;
    ui.height = height as c_int;
    // The screen is the smallest attached UI, so one UI resizing can
    // change what every other one is sent.
    // SAFETY: no borrow of the UI is held across the refresh.
    unsafe { ui_refresh() };
    ().reported(error)
}

/// Changes one negotiated option after attaching.
///
/// # Safety
///
/// `value` must stay valid for the duration.
pub unsafe fn nvim_ui_set_option(
    channel_id: u64,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let ui = get_ui_or_err(channel_id, &mut error);
    if ui.is_null() {
        return Err(error);
    }
    // SAFETY: the UI just looked up, and the caller's value.
    unsafe { ui_set_option(ui, false, name, value, &mut error) };
    ().reported(error)
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
    err: &mut Error,
) {
    // SAFETY: the caller's promise -- `ui` is live for the call. `Live`
    // hands out a borrow only for the length of one field access, so the
    // calls below never run with one outstanding.
    let mut ui = unsafe { Ui::new(ui) };
    // `name.data` can be null, which `strequal` treats as no match; a
    // `CStr` conversion here would not survive it.
    // SAFETY: `name` is the caller's, and `strequal` accepts a null.
    let named = |want: &CStr| unsafe { strequal(name.data(), want.as_ptr()) };

    if named(c"override") {
        // SAFETY: the caller's promise about `err`; the name is static.
        let Some(on) = (unsafe { want_boolean(err, c"override", value) }) else {
            return;
        };
        // Asks for the highest capabilities any UI requested rather
        // than the intersection, for UIs that can cope with anything.
        ui.override_0 = on;
        return;
    }

    if named(c"rgb") {
        // SAFETY: as above.
        let Some(on) = (unsafe { want_boolean(err, c"rgb", value) }) else {
            return;
        };
        ui.rgb = on;
        // Only the legacy protocol bakes the colour model into what it
        // is sent; a linegrid UI gets both and picks.
        if !init && !ui.ui_ext[kUILinegrid as usize] {
            // SAFETY: no borrow of the UI is held across the refresh.
            unsafe { ui_refresh() };
        }
        return;
    }

    if named(c"term_name") {
        // SAFETY: as above.
        let Some(term) = (unsafe { want_string(err, c"term_name", value) }) else {
            return;
        };
        // 'term' is global, so the last UI to say what terminal it is
        // wins; the copy on the UI is what `nvim_list_uis` reports. Each
        // side gets its own allocation, since both are freed separately.
        // SAFETY: `term` is the caller's string, live for the call.
        unsafe { set_tty_option(c"term", string_to_cstr(term)) };
        // SAFETY: as above.
        ui.term_name = unsafe { string_to_cstr(term) };
        return;
    }

    if named(c"term_colors") {
        // SAFETY: as above.
        let Some(colors) = (unsafe { want_integer(err, c"term_colors", value) }) else {
            return;
        };
        t_colors.set(colors as c_int);
        ui.term_colors = colors as c_int;
        return;
    }

    if named(c"stdin_fd") {
        // SAFETY: as above.
        let Some(fd) = (unsafe { want_integer(err, c"stdin_fd", value) }) else {
            return;
        };
        if fd < 0 {
            let null = core::ptr::null();
            // SAFETY: the names and values are NUL-terminated strings.
            unsafe { *err = err_invalid_ptr(c"stdin_fd".as_ptr(), null, fd, false) };
            return;
        }
        // The editor reads its startup input from this descriptor,
        // which only means anything before startup has finished.
        if starting.get() != 2 {
            *err = Error::validation(c"stdin_fd can only be used with first attached UI");
            return;
        }
        stdin_fd.set(fd as c_int);
        return;
    }

    if named(c"stdin_tty") {
        // SAFETY: as above.
        let Some(tty) = (unsafe { want_boolean(err, c"stdin_tty", value) }) else {
            return;
        };
        // Only the stdio channel is talking about the editor's own
        // standard streams.
        if ui.channel_id == CHAN_STDIO {
            stdin_isatty.set(tty);
        }
        ui.stdin_tty = tty;
        return;
    }

    if named(c"stdout_tty") {
        // SAFETY: as above.
        let Some(tty) = (unsafe { want_boolean(err, c"stdout_tty", value) }) else {
            return;
        };
        if ui.channel_id == CHAN_STDIO {
            stdout_isatty.set(tty);
        }
        ui.stdout_tty = tty;
        return;
    }

    // The extensions, by their protocol names. `popupmenu_external` is
    // the pre-0.3 spelling of `ext_popupmenu` and still accepted.
    let is_popupmenu = named(c"popupmenu_external");
    for ext in 0..kUIExtCount as usize {
        // SAFETY: `name` is the caller's and the table holds static names.
        let matched = unsafe { strequal(name.data(), ui_ext_names[ext]) };
        if !matched && !(ext == kUIPopupmenu as usize && is_popupmenu) {
            continue;
        }
        let Some(active) = value.as_boolean() else {
            // SAFETY: the caller's promise about `err`, and `name` is the
            // caller's C string.
            unsafe { wrong_type(err, name.data(), kObjectTypeBoolean, value) };
            return;
        };
        // Which protocol a UI speaks is decided at attach: the editor
        // has already sent it events in that protocol's shape.
        if !init && ext == kUILinegrid as usize && active != ui.ui_ext[ext] {
            *err = Error::validation(c"ext_linegrid option cannot be changed");
        }
        ui.ui_ext[ext] = active;
        if !init {
            // SAFETY: `ui` is live and no borrow of it is outstanding.
            unsafe { ui_set_ext_option(ui.raw(), ext as UIExtension, active) };
        }
        return;
    }

    let unknown = name.data();
    // SAFETY: the names and values are NUL-terminated strings.
    unsafe { *err = err_invalid_ptr(c"UI option".as_ptr(), unknown, 0, true) };
}

/// `value` as the boolean `name` takes, or `None` with `err` set to say
/// what arrived instead.
///
/// # Safety
///
/// `err` must be writable.
unsafe fn want_boolean(err: &mut Error, name: &CStr, value: Object) -> Option<Boolean> {
    let got = value.as_boolean();
    if got.is_none() {
        // SAFETY: the caller's promise about `err`; `name` is a C string.
        unsafe { wrong_type(err, name.as_ptr(), kObjectTypeBoolean, value) };
    }
    got
}

/// [`want_boolean`] for an integer.
///
/// # Safety
///
/// As [`want_boolean`].
unsafe fn want_integer(err: &mut Error, name: &CStr, value: Object) -> Option<Integer> {
    let got = value.as_integer();
    if got.is_none() {
        // SAFETY: as [`want_boolean`].
        unsafe { wrong_type(err, name.as_ptr(), kObjectTypeInteger, value) };
    }
    got
}

/// [`want_boolean`] for a string.
///
/// # Safety
///
/// As [`want_boolean`].
unsafe fn want_string(err: &mut Error, name: &CStr, value: Object) -> Option<String_0> {
    let got = value.as_string();
    if got.is_none() {
        // SAFETY: as [`want_boolean`].
        unsafe { wrong_type(err, name.as_ptr(), kObjectTypeString, value) };
    }
    got
}

/// Reports that `value` is not the `expected` type `name` takes.
///
/// # Safety
///
/// `err` must be writable and `name` a valid C string.
unsafe fn wrong_type(err: &mut Error, name: *const c_char, expected: ObjectType, value: Object) {
    let expected = api_typename(expected);
    let actual = api_typename(value.type_0);
    // SAFETY: the names and values are NUL-terminated strings.
    unsafe { *err = err_expected_ptr(name, expected, Some(actual)) };
}

/// Resizes one grid, for a UI with `ext_multigrid`.
///
/// # Safety
///
/// `err` must be writable.
pub unsafe fn nvim_ui_try_resize_grid(
    channel_id: u64,
    grid: Integer,
    width: Integer,
    height: Integer,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    if get_ui_or_err(channel_id, &mut error).is_null() {
        return ().reported(error);
    }
    if grid == DEFAULT_GRID_HANDLE {
        // The default grid is the screen, so resizing it is a window
        // resize like any other.
        // SAFETY: `error` is this frame's slot.
        return unsafe { nvim_ui_try_resize(channel_id, width, height) };
    }
    let (grid, width, height) = (grid as handle_T, width as c_int, height as c_int);
    // SAFETY: `error` is this frame's slot.
    unsafe { ui_grid_resize(grid, width, height, &mut error) };
    ().reported(error)
}

/// Tells the editor how many lines this UI's popupmenu can show.
///
/// # Safety
///
/// `err` must be writable.
pub unsafe fn nvim_ui_pum_set_height(channel_id: u64, height: Integer) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let ui = get_ui_or_err(channel_id, &mut error);
    if ui.is_null() {
        return ().reported(error);
    }
    if height <= 0 {
        error = Error::validation(c"Expected pum height > 0");
        return ().reported(error);
    }
    // SAFETY: `ui` is in the attach table, so it is live.
    let mut ui = unsafe { Ui::new(ui) };
    if !ui.ui_ext[kUIPopupmenu as usize] {
        error = Error::validation(c"UI must support the ext_popupmenu option");
        return ().reported(error);
    }
    ui.pum_nlines = height as c_int;
    ().reported(error)
}

/// Tells the editor where this UI drew its popupmenu, so that `pumvisible()`
/// and the completion logic can reason about the screen area it covers.
///
/// # Safety
///
/// `err` must be writable.
pub unsafe fn nvim_ui_pum_set_bounds(
    channel_id: u64,
    width: Float,
    height: Float,
    row: Float,
    col: Float,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let ui = get_ui_or_err(channel_id, &mut error);
    if ui.is_null() {
        return ().reported(error);
    }
    // SAFETY: `ui` is in the attach table, so it is live.
    let mut ui = unsafe { Ui::new(ui) };
    if !ui.ui_ext[kUIPopupmenu as usize] {
        error = Error::validation(c"UI must support the ext_popupmenu option");
        return ().reported(error);
    }
    if width <= 0.0 {
        error = Error::validation(c"Expected width > 0");
        return ().reported(error);
    }
    if height <= 0.0 {
        error = Error::validation(c"Expected height > 0");
        return ().reported(error);
    }
    ui.pum_row = row;
    ui.pum_col = col;
    ui.pum_width = width;
    ui.pum_height = height;
    ui.pum_pos = true;
    ().reported(error)
}

/// Forwards `content` to every UI that owns a terminal, as `ui_send`.
///
/// # Safety
///
/// `content` must be valid for the duration of the call.
pub unsafe fn nvim_ui_send(_channel_id: u64, content: String_0) {
    ui_call_ui_send(content);
}
