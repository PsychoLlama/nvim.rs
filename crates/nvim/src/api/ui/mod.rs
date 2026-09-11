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
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

mod events;
mod line;
mod packer;
mod redraw;

pub use events::*;
pub use line::remote_ui_raw_line;
pub use packer::remote_ui_flush_pending_data;
pub use redraw::{remote_ui_event, remote_ui_hl_attr_define};

use crate::api::private::helpers::{api_typename, cstr_to_string, string_to_cstr};
use crate::api::private::validate::{err_bad_number, err_bad_value, err_expected};
use crate::api_error;
use crate::autocmd::{do_autocmd_focusgained, may_trigger_vim_suspend_resume};
use crate::channel::find_channel;
use crate::event::r#loop::process_events_until;
use crate::global_cell::GlobalCell;
use crate::memory::{strequal, xfree};
use crate::option::set_tty_option;
use crate::startup::{main_loop, starting, stdin_fd, stdin_isatty, stdout_isatty};
use crate::types::builders::{ArrayBuf, DictBuf};
use crate::types::ui::{
    kUICmdline, kUIExtCount, kUIHlState, kUILinegrid, kUIMessages, kUIMultigrid, kUIPopupmenu,
};
use crate::types::{
    ApiDict, Boolean, Error, Float, Handle, Integer, Object, ObjectType, PackerBuffer, RemoteUI,
    String_0, UIExtension, kErrorTypeException, kObjectTypeBoolean, kObjectTypeInteger,
    kObjectTypeString,
};
use crate::ui::state::{current_ui, t_colors, ui_ext_names};
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

/// The UI attached to `chan_id`, or why there is none.
fn get_ui_or_err(chan_id: u64) -> Result<*mut RemoteUI, Error> {
    match find_ui(chan_id) {
        Some(ui) if !ui.is_null() => Ok(ui),
        _ => Err(api_error!(
            kErrorTypeException,
            "UI not attached to channel: {chan_id}"
        )),
    }
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
    unsafe { xfree(ui.packer.start().cast()) };
    // SAFETY: as above.
    unsafe { xfree(ui.term_name.cast()) };
}

/// Detaches the UI on `channel_id`, optionally telling it why.
///
/// Safe: every `unsafe` below rests on the attach table, which this function
/// reads and updates itself, rather than on anything the caller promised.
pub fn remote_ui_disconnect(channel_id: u64, send_error_exit: bool) -> Result<(), Error> {
    let ui = get_ui_or_err(channel_id)?;
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
    Ok(())
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
    options: ApiDict,
) -> Result<(), Error> {
    if find_ui(channel_id).is_some() {
        return Err(api_error!(
            kErrorTypeException,
            "UI already attached to channel: {channel_id}"
        ));
    }
    if !ui_can_attach_more() {
        return Err(Error::exception(c"Maximum UI count reached"));
    }
    if width <= 0 || height <= 0 {
        return Err(Error::validation(c"Expected width > 0 and height > 0"));
    }

    let raw = Box::into_raw(Box::new(RemoteUI::new(channel_id, width, height)));
    // SAFETY: `raw` is the box just made, live until it is handed to the
    // attach table or dropped below.
    let mut ui = unsafe { Ui::new(raw) };
    // The packer reaches back to the UI to flush it when full, which is
    // only possible once the box has an address.
    ui.packer.anydata = raw.cast();

    for option in &options {
        // SAFETY: `raw` is live, and the value lives as long as the
        // caller's dictionary.
        let set = unsafe { ui_set_option(raw, true, option.key.clone(), option.value.clone()) };
        if let Err(e) = set {
            // Nothing has been published yet, so the half-configured UI
            // can simply be dropped. `term_name` is the only owned
            // field an option sets, and not on an error path.
            // SAFETY: nothing else names `raw` yet.
            drop(unsafe { Box::from_raw(raw) });
            return Err(e);
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
    Ok(())
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
            packer: PackerBuffer::detached(core::ptr::null_mut(), Some(packer::ui_flush_callback)),
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
pub fn nvim_ui_set_focus(channel_id: u64, gained: Boolean) -> Result<(), Error> {
    get_ui_or_err(channel_id)?;
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
pub fn nvim_ui_detach(channel_id: u64) -> Result<(), Error> {
    remote_ui_disconnect(channel_id, false)
}

/// Tells a UI to reconnect to `server_addr`.
///
/// Sent by `:restart`, where the server this UI is talking to is about to
/// be replaced by one listening elsewhere.
///
/// # Safety
///
/// `server_addr` a valid C string.
pub unsafe fn remote_ui_connect(channel_id: u64, server_addr: *mut c_char) -> Result<(), Error> {
    let ui = get_ui_or_err(channel_id)?;
    let mut args = ArrayBuf::<1>::new();
    // SAFETY: the caller's promise -- `server_addr` is a C string, and the
    // borrowed view of it does not outlive this call.
    args.push(Object::string(unsafe { cstr_to_string(server_addr) }));
    // SAFETY: `ui` is in the attach table, so it is live.
    unsafe { packer::push_call(ui, c"connect", args.array()) };
    Ok(())
}

/// Reports that this UI's window is now `width` by `height` cells.
///
/// # Safety
///
/// The editor must be running.
pub unsafe fn nvim_ui_try_resize(
    channel_id: u64,
    width: Integer,
    height: Integer,
) -> Result<(), Error> {
    let ui = get_ui_or_err(channel_id)?;
    if width <= 0 || height <= 0 {
        return Err(Error::validation(c"Expected width > 0 and height > 0"));
    }
    // SAFETY: `ui` is in the attach table, so it is live.
    let mut ui = unsafe { Ui::new(ui) };
    ui.width = width as c_int;
    ui.height = height as c_int;
    // The screen is the smallest attached UI, so one UI resizing can
    // change what every other one is sent.
    // SAFETY: no borrow of the UI is held across the refresh.
    unsafe { ui_refresh() };
    Ok(())
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
    let ui = get_ui_or_err(channel_id)?;
    // SAFETY: the UI just looked up, and the caller's value.
    unsafe { ui_set_option(ui, false, name, value) }
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
) -> Result<(), Error> {
    // SAFETY: the caller's promise -- `ui` is live for the call. `Live`
    // hands out a borrow only for the length of one field access, so the
    // calls below never run with one outstanding.
    let mut ui = unsafe { Ui::new(ui) };
    // `name.data` can be null, which `strequal` treats as no match; a
    // `CStr` conversion here would not survive it.
    // SAFETY: `name` is the caller's, and `strequal` accepts a null.
    let named = |want: &CStr| unsafe { strequal(name.data(), want.as_ptr()) };

    if named(c"override") {
        let on = want_boolean(c"override", value)?;
        // Asks for the highest capabilities any UI requested rather
        // than the intersection, for UIs that can cope with anything.
        ui.override_0 = on;
        return Ok(());
    }

    if named(c"rgb") {
        let on = want_boolean(c"rgb", value)?;
        ui.rgb = on;
        // Only the legacy protocol bakes the colour model into what it
        // is sent; a linegrid UI gets both and picks.
        if !init && !ui.ui_ext[kUILinegrid as usize] {
            // SAFETY: no borrow of the UI is held across the refresh.
            unsafe { ui_refresh() };
        }
        return Ok(());
    }

    if named(c"term_name") {
        let term = want_string(c"term_name", value)?;
        // 'term' is global, so the last UI to say what terminal it is
        // wins; the copy on the UI is what `nvim_list_uis` reports. Each
        // side gets its own allocation, since both are freed separately.
        // SAFETY: `term` is the caller's string, live for the call.
        unsafe { set_tty_option(c"term", string_to_cstr(&term)) };
        // SAFETY: as above.
        ui.term_name = string_to_cstr(&term);
        return Ok(());
    }

    if named(c"term_colors") {
        let colors = want_integer(c"term_colors", value)?;
        t_colors.set(colors as c_int);
        ui.term_colors = colors as c_int;
        return Ok(());
    }

    if named(c"stdin_fd") {
        let fd = want_integer(c"stdin_fd", value)?;
        if fd < 0 {
            return Err(err_bad_number(c"stdin_fd", fd));
        }
        // The editor reads its startup input from this descriptor,
        // which only means anything before startup has finished.
        if starting.get() != 2 {
            let why = c"stdin_fd can only be used with first attached UI";
            return Err(Error::validation(why));
        }
        stdin_fd.set(fd as c_int);
        return Ok(());
    }

    if named(c"stdin_tty") {
        let tty = want_boolean(c"stdin_tty", value)?;
        // Only the stdio channel is talking about the editor's own
        // standard streams.
        if ui.channel_id == CHAN_STDIO {
            stdin_isatty.set(tty);
        }
        ui.stdin_tty = tty;
        return Ok(());
    }

    if named(c"stdout_tty") {
        let tty = want_boolean(c"stdout_tty", value)?;
        if ui.channel_id == CHAN_STDIO {
            stdout_isatty.set(tty);
        }
        ui.stdout_tty = tty;
        return Ok(());
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
            // SAFETY: `name` is the caller's NUL-terminated option name.
            let name = name.as_cstr();
            return Err(wrong_type(name, kObjectTypeBoolean, value));
        };
        // Which protocol a UI speaks is decided at attach: the editor
        // has already sent it events in that protocol's shape.
        if !init && ext == kUILinegrid as usize && active != ui.ui_ext[ext] {
            return Err(Error::validation(c"ext_linegrid option cannot be changed"));
        }
        ui.ui_ext[ext] = active;
        if !init {
            // SAFETY: `ui` is live and no borrow of it is outstanding.
            unsafe { ui_set_ext_option(ui.raw(), ext as UIExtension, active) };
        }
        return Ok(());
    }

    // SAFETY: the caller's option name is NUL-terminated.
    let unknown = name.as_cstr();
    Err(err_bad_value(c"UI option", unknown))
}

/// `value` as the boolean `name` takes, or what arrived instead.
fn want_boolean(name: &CStr, value: Object) -> Result<Boolean, Error> {
    value
        .as_boolean()
        .ok_or_else(|| wrong_type(name, kObjectTypeBoolean, value))
}

/// [`want_boolean`] for an integer.
fn want_integer(name: &CStr, value: Object) -> Result<Integer, Error> {
    value
        .as_integer()
        .ok_or_else(|| wrong_type(name, kObjectTypeInteger, value))
}

/// [`want_boolean`] for a string.
fn want_string(name: &CStr, value: Object) -> Result<String_0, Error> {
    if value.as_string().is_none() {
        return Err(wrong_type(name, kObjectTypeString, value));
    }
    Ok(value.into_string().expect("the check above matched"))
}

/// Reports that `value` is not the `expected` type `name` takes.
fn wrong_type(name: &CStr, expected: ObjectType, value: Object) -> Error {
    let expected = api_typename(expected);
    let actual = api_typename(value.kind());
    err_expected(name, expected, Some(actual))
}

/// Resizes one grid, for a UI with `ext_multigrid`.
///
/// # Safety
///
/// The editor must be running.
pub unsafe fn nvim_ui_try_resize_grid(
    channel_id: u64,
    grid: Integer,
    width: Integer,
    height: Integer,
) -> Result<(), Error> {
    get_ui_or_err(channel_id)?;
    if grid == DEFAULT_GRID_HANDLE {
        // The default grid is the screen, so resizing it is a window
        // resize like any other.
        // SAFETY: the editor is running, per this function's contract.
        return unsafe { nvim_ui_try_resize(channel_id, width, height) };
    }
    let (grid, width, height) = (grid as Handle, width as c_int, height as c_int);
    ui_grid_resize(grid, width, height)
}

/// Tells the editor how many lines this UI's popupmenu can show.
///
/// # Safety
///
/// The editor must be running.
pub unsafe fn nvim_ui_pum_set_height(channel_id: u64, height: Integer) -> Result<(), Error> {
    let ui = get_ui_or_err(channel_id)?;
    if height <= 0 {
        return Err(Error::validation(c"Expected pum height > 0"));
    }
    // SAFETY: `ui` is in the attach table, so it is live.
    let mut ui = unsafe { Ui::new(ui) };
    if !ui.ui_ext[kUIPopupmenu as usize] {
        return Err(Error::validation(
            c"UI must support the ext_popupmenu option",
        ));
    }
    ui.pum_nlines = height as c_int;
    Ok(())
}

/// Tells the editor where this UI drew its popupmenu, so that `pumvisible()`
/// and the completion logic can reason about the screen area it covers.
///
/// # Safety
///
/// The editor must be running.
pub unsafe fn nvim_ui_pum_set_bounds(
    channel_id: u64,
    width: Float,
    height: Float,
    row: Float,
    col: Float,
) -> Result<(), Error> {
    let ui = get_ui_or_err(channel_id)?;
    // SAFETY: `ui` is in the attach table, so it is live.
    let mut ui = unsafe { Ui::new(ui) };
    if !ui.ui_ext[kUIPopupmenu as usize] {
        return Err(Error::validation(
            c"UI must support the ext_popupmenu option",
        ));
    }
    if width <= 0.0 {
        return Err(Error::validation(c"Expected width > 0"));
    }
    if height <= 0.0 {
        return Err(Error::validation(c"Expected height > 0"));
    }
    ui.pum_row = row;
    ui.pum_col = col;
    ui.pum_width = width;
    ui.pum_height = height;
    ui.pum_pos = true;
    Ok(())
}

/// Forwards `content` to every UI that owns a terminal, as `ui_send`.
pub fn nvim_ui_send(_channel_id: u64, content: String_0) {
    ui_call_ui_send(content);
}
