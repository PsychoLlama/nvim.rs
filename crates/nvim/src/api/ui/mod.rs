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
//! Every serializer takes the UI as `&mut RemoteUI`, lent by the attach
//! table in [`crate::ui`] for the one event; anything that runs user code
//! in between finds the UI again by its channel id.
//!
//! The `nvim_ui_*` entry points keep `extern "C"` because that is how
//! `tools/apigen` recognises an API function; nothing else here does.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

mod events;
mod line;
mod packer;
mod redraw;

pub use events::*;
pub use line::remote_ui_raw_line;
pub use redraw::{remote_ui_event, remote_ui_hl_attr_define};

use crate::api::private::helpers::api_typename;
use crate::api::private::validate::handle_arg;
use crate::api::private::validate::{err_bad_number, err_bad_value, err_expected};
use crate::api_error;
use crate::autocmd::{do_autocmd_focusgained, may_trigger_vim_suspend_resume};
use crate::event::r#loop::{Drain, wait_on_main_loop};
use crate::memory::XString;
use crate::narrow::number_as_int;
use crate::option::set_tty_option;
use crate::startup::{STDIN_FD, STDIN_ISATTY, STDOUT_ISATTY, starting};
use crate::types::builders::{ArrayBuf, DictBuf};
use crate::types::{
    ApiDict, ApiString, Boolean, ERROR_TYPE_EXCEPTION, Error, Float, Integer, OBJECT_TYPE_BOOLEAN,
    OBJECT_TYPE_INTEGER, OBJECT_TYPE_STRING, Object, ObjectType,
};
use crate::ui::state::{CURRENT_UI, T_COLORS, UI_EXT_NAMES};
use crate::ui::state::{RemoteUI, UIExtension};
use crate::ui::state::{
    UI_CMDLINE, UI_EXT_COUNT, UI_HL_STATE, UI_LINEGRID, UI_MESSAGES, UI_MULTIGRID, UI_POPUPMENU,
};
use crate::ui::{
    ui_active, ui_attach_impl, ui_call_ui_send, ui_can_attach_more, ui_detach_impl, ui_grid_resize,
    ui_is_attached, ui_refresh, ui_set_ext_option, with_ui,
};
use core::ffi::CStr;

/// The channel id `--embed` and `nvim -` use, which is the only one whose
/// tty-ness is the editor's own.
const CHAN_STDIO: u64 = 1;

/// The handle of the grid every UI has, before `ext_multigrid` adds more.
const DEFAULT_GRID_HANDLE: Integer = 1;

/// Why a call that needs the UI on `chan_id` cannot have it.
fn not_attached(chan_id: u64) -> Error {
    api_error!(
        ERROR_TYPE_EXCEPTION,
        "UI not attached to channel: {chan_id}"
    )
}

/// Runs `f` on the UI attached to `chan_id`, or says there is none.
fn on_ui<R>(chan_id: u64, f: impl FnOnce(&mut RemoteUI) -> R) -> Result<R, Error> {
    with_ui(chan_id, f).ok_or_else(|| not_attached(chan_id))
}

/// Detaches the UI on `channel_id`, optionally telling it why.
pub fn remote_ui_disconnect(channel_id: u64, send_error_exit: bool) -> Result<(), Error> {
    on_ui(channel_id, |ui| {
        if send_error_exit {
            // A UI told to exit is one whose server is going away, so this
            // has to go out before the channel does.
            let mut args = ArrayBuf::<1>::new();
            args.push(Object::integer(0));
            packer::push_call(ui, c"error_exit", args.array());
            packer::ui_flush_buf(ui);
        }
    })?;
    // Out of the table, the UI and whatever it still had packed are dropped.
    drop(ui_detach_impl(channel_id));
    Ok(())
}

/// Pumps the event loop until some UI has attached.
///
/// Used at startup by `--embed` without `--headless`, where the editor must
/// not draw anything until it knows the terminal size.
pub fn remote_ui_wait_for_attach() {
    wait_on_main_loop(Drain::Events, -1, || ui_active() != 0);
}

/// Attaches the calling channel as a UI of `width` by `height` cells.
pub fn nvim_ui_attach(
    channel_id: u64,
    width: Integer,
    height: Integer,
    options: ApiDict,
) -> Result<(), Error> {
    if ui_is_attached(channel_id) {
        return Err(api_error!(
            ERROR_TYPE_EXCEPTION,
            "UI already attached to channel: {channel_id}"
        ));
    }
    if !ui_can_attach_more() {
        return Err(Error::exception(c"Maximum UI count reached"));
    }
    if width <= 0 || height <= 0 {
        return Err(Error::validation(c"Expected width > 0 and height > 0"));
    }

    // Configured before anyone else can see it: on an error the
    // half-configured UI is simply dropped.
    let mut ui = RemoteUI::new(channel_id, width, height);
    for option in &options {
        let name = ApiString::from_bytes(option.key.bytes());
        ui_set_option(&mut ui, true, name, option.value.clone())?;
    }

    // Options that imply others. A UI asking for anything the linegrid
    // protocol introduced is asking for the linegrid protocol; external
    // messages are drawn in a cmdline the UI must also own.
    if ui.ui_ext[UI_HL_STATE as usize] || ui.ui_ext[UI_MULTIGRID as usize] {
        ui.ui_ext[UI_LINEGRID as usize] = true;
    }
    if ui.ui_ext[UI_MESSAGES as usize] {
        ui.ui_ext[UI_LINEGRID as usize] = true;
        ui.ui_ext[UI_CMDLINE as usize] = true;
    }

    CURRENT_UI.set(channel_id);
    ui_attach_impl(ui);
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
            ui_ext: [false; UI_EXT_COUNT as usize],
            width: number_as_int(width),
            height: number_as_int(height),
            pum_nlines: 0,
            pum_pos: false,
            // Negative means the UI has not reported where its popupmenu
            // is, so the editor places it itself.
            pum_row: -1.0,
            pum_col: -1.0,
            pum_height: 0.0,
            pum_width: 0.0,
            term_name: ApiString::NULL,
            term_colors: 0,
            stdin_tty: false,
            stdout_tty: false,
            channel_id,
            packer: packer::ui_packer(),
            cur_event: None,
            nevents_pos: None,
            ncalls_pos: None,
            nevents: 0,
            ncalls: 0,
            flushed_events: false,
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
pub fn ui_attach(
    channel_id: u64,
    width: Integer,
    height: Integer,
    enable_rgb: Boolean,
) -> Result<(), Error> {
    let mut opts = DictBuf::<1>::new();
    opts.insert(c"rgb", Object::boolean(enable_rgb));
    nvim_ui_attach(channel_id, width, height, opts.dict())
}

/// Tells the editor that this UI gained or lost the user's attention.
pub fn nvim_ui_set_focus(channel_id: u64, gained: Boolean) -> Result<(), Error> {
    on_ui(channel_id, |_| ())?;
    if gained {
        // Whichever UI was focused last is the one `nvim_get_current_ui`
        // means and the one a `:suspend` applies to.
        CURRENT_UI.set(channel_id);
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
pub fn remote_ui_connect(channel_id: u64, server_addr: &CStr) -> Result<(), Error> {
    on_ui(channel_id, |ui| {
        let mut args = ArrayBuf::<1>::new();
        args.push(Object::string(ApiString::from_cstr(server_addr)));
        packer::push_call(ui, c"connect", args.array());
    })
}

/// Reports that this UI's window is now `width` by `height` cells.
pub fn nvim_ui_try_resize(channel_id: u64, width: Integer, height: Integer) -> Result<(), Error> {
    on_ui(channel_id, |_| ())?;
    if width <= 0 || height <= 0 {
        return Err(Error::validation(c"Expected width > 0 and height > 0"));
    }
    on_ui(channel_id, |ui| {
        ui.width = number_as_int(width);
        ui.height = number_as_int(height);
    })?;
    // The screen is the smallest attached UI, so one UI resizing can
    // change what every other one is sent.
    ui_refresh();
    Ok(())
}

/// Changes one negotiated option after attaching.
pub fn nvim_ui_set_option(channel_id: u64, name: ApiString, value: Object) -> Result<(), Error> {
    // Applied to the table's copy; what it means for the screen is
    // published afterwards, with the table no longer lent out.
    let publish = on_ui(channel_id, |ui| ui_set_option(ui, false, name, value))??;
    match publish {
        Publish::Nothing => {}
        Publish::Refresh => ui_refresh(),
        Publish::Extension(ext, active) => ui_set_ext_option(channel_id, ext, active),
    }
    Ok(())
}

/// What an option change after attach has to tell the rest of the editor.
#[derive(Debug)]
enum Publish {
    Nothing,
    /// The colour model of a legacy UI changed.
    Refresh,
    /// An `ext_*` option changed.
    Extension(UIExtension, bool),
}

/// Applies one option to `ui`.
///
/// `init` distinguishes the options passed to `nvim_ui_attach`, which are
/// applied to a UI the editor has not seen yet, from later changes, which
/// the caller publishes as the answer says.
fn ui_set_option(
    ui: &mut RemoteUI,
    init: bool,
    name: ApiString,
    value: Object,
) -> Result<Publish, Error> {
    // `name.data` can be null, which `as_cstr` answers as `c""` -- no
    // match, since every `want` below has bytes.
    let named = |want: &CStr| name.as_cstr() == want;

    if named(c"override") {
        let on = want_boolean(c"override", value)?;
        // Asks for the highest capabilities any UI requested rather
        // than the intersection, for UIs that can cope with anything.
        ui.override_0 = on;
        return Ok(Publish::Nothing);
    }

    if named(c"rgb") {
        let on = want_boolean(c"rgb", value)?;
        ui.rgb = on;
        // Only the legacy protocol bakes the colour model into what it
        // is sent; a linegrid UI gets both and picks.
        if !init && !ui.ui_ext[UI_LINEGRID as usize] {
            return Ok(Publish::Refresh);
        }
        return Ok(Publish::Nothing);
    }

    if named(c"term_name") {
        let term = want_string(c"term_name", value)?;
        // 'term' is global, so the last UI to say what terminal it is
        // wins; the copy on the UI is what `nvim_list_uis` reports.
        let _ = set_tty_option(c"term", XString::from_bytes(term.as_bytes()));
        ui.term_name = term;
        return Ok(Publish::Nothing);
    }

    if named(c"term_colors") {
        let colors = want_integer(c"term_colors", value)?;
        T_COLORS.set(number_as_int(colors));
        ui.term_colors = number_as_int(colors);
        return Ok(Publish::Nothing);
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
        STDIN_FD.set(number_as_int(fd));
        return Ok(Publish::Nothing);
    }

    if named(c"stdin_tty") {
        let tty = want_boolean(c"stdin_tty", value)?;
        // Only the stdio channel is talking about the editor's own
        // standard streams.
        if ui.channel_id == CHAN_STDIO {
            STDIN_ISATTY.set(tty);
        }
        ui.stdin_tty = tty;
        return Ok(Publish::Nothing);
    }

    if named(c"stdout_tty") {
        let tty = want_boolean(c"stdout_tty", value)?;
        if ui.channel_id == CHAN_STDIO {
            STDOUT_ISATTY.set(tty);
        }
        ui.stdout_tty = tty;
        return Ok(Publish::Nothing);
    }

    // The extensions, by their protocol names. `popupmenu_external` is
    // the pre-0.3 spelling of `ext_popupmenu` and still accepted.
    let is_popupmenu = named(c"popupmenu_external");
    for (ext, ext_name) in UI_EXT_NAMES.iter().enumerate().take(UI_EXT_COUNT as usize) {
        let matched = name.as_cstr() == *ext_name;
        if !matched && !(ext == UI_POPUPMENU as usize && is_popupmenu) {
            continue;
        }
        let Some(active) = value.as_boolean() else {
            let name = name.as_cstr();
            return Err(wrong_type(name, OBJECT_TYPE_BOOLEAN, value));
        };
        // Which protocol a UI speaks is decided at attach: the editor
        // has already sent it events in that protocol's shape.
        if !init && ext == UI_LINEGRID as usize && active != ui.ui_ext[ext] {
            return Err(Error::validation(c"ext_linegrid option cannot be changed"));
        }
        ui.ui_ext[ext] = active;
        if !init {
            let ext = UIExtension::try_from(ext).expect("`ui_ext` is indexed by extension");
            return Ok(Publish::Extension(ext, active));
        }
        return Ok(Publish::Nothing);
    }

    let unknown = name.as_cstr();
    Err(err_bad_value(c"UI option", unknown))
}

/// `value` as the boolean `name` takes, or what arrived instead.
fn want_boolean(name: &CStr, value: Object) -> Result<Boolean, Error> {
    value
        .as_boolean()
        .ok_or_else(|| wrong_type(name, OBJECT_TYPE_BOOLEAN, value))
}

/// [`want_boolean`] for an integer.
fn want_integer(name: &CStr, value: Object) -> Result<Integer, Error> {
    value
        .as_integer()
        .ok_or_else(|| wrong_type(name, OBJECT_TYPE_INTEGER, value))
}

/// [`want_boolean`] for a string.
fn want_string(name: &CStr, value: Object) -> Result<ApiString, Error> {
    if value.as_string().is_none() {
        return Err(wrong_type(name, OBJECT_TYPE_STRING, value));
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
pub fn nvim_ui_try_resize_grid(
    channel_id: u64,
    grid: Integer,
    width: Integer,
    height: Integer,
) -> Result<(), Error> {
    on_ui(channel_id, |_| ())?;
    if grid == DEFAULT_GRID_HANDLE {
        // The default grid is the screen, so resizing it is a window
        // resize like any other.
        return nvim_ui_try_resize(channel_id, width, height);
    }
    let (grid, width, height) = (
        handle_arg(c"grid handle", grid)?,
        number_as_int(width),
        number_as_int(height),
    );
    ui_grid_resize(grid, width, height)
}

/// Tells the editor how many lines this UI's popupmenu can show.
pub fn nvim_ui_pum_set_height(channel_id: u64, height: Integer) -> Result<(), Error> {
    on_ui(channel_id, |_| ())?;
    if height <= 0 {
        return Err(Error::validation(c"Expected pum height > 0"));
    }
    on_ui(channel_id, |ui| {
        if !ui.ui_ext[UI_POPUPMENU as usize] {
            return Err(Error::validation(
                c"UI must support the ext_popupmenu option",
            ));
        }
        ui.pum_nlines = number_as_int(height);
        Ok(())
    })?
}

/// Tells the editor where this UI drew its popupmenu, so that `pumvisible()`
/// and the completion logic can reason about the screen area it covers.
pub fn nvim_ui_pum_set_bounds(
    channel_id: u64,
    width: Float,
    height: Float,
    row: Float,
    col: Float,
) -> Result<(), Error> {
    on_ui(channel_id, |ui| {
        if !ui.ui_ext[UI_POPUPMENU as usize] {
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
    })?
}

/// Forwards `content` to every UI that owns a terminal, as `ui_send`.
pub fn nvim_ui_send(_channel_id: u64, content: ApiString) {
    ui_call_ui_send(content);
}
