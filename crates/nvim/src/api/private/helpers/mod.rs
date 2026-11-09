//! The plumbing every `nvim_*` function shares. Nothing here is API surface
//! of its own.
//!
//! This file holds the constants the whole family reads, plus the two jobs
//! that are only a few functions each:
//!
//! - **Handles.** Turning a `Buffer`/`Window`/`Tabpage` id from the wire back
//!   into a pointer, or reporting that it names nothing.
//! - **Errors.** An API call reports failure through an `Error`
//!   out-parameter rather than by throwing, so [`try_enter`]/[`try_leave`]
//!   bracket a call that runs Vimscript and turn whatever it threw — an
//!   exception, an `:echoerr`, a `CTRL-C` — into one.
//!
//! The rest is one module per kind of value: [`value`] for the `Object` tree
//! and who owns it, [`text`] for strings and buffer text, [`vimdict`] for
//! Vimscript dictionaries, and [`keydict`] for the generated option structs.
//! They are re-exported here, so callers name `helpers::` as they always
//! did.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::deref::deref_to_field;
use crate::ex_eval::defs::ExcId;
use crate::narrow::{number_as_int, number_as_lnum};
use crate::runtime::{SID_API_CLIENT, SID_LUA, script_is_lua};
use crate::types::{ColNr, LineNr};
use core::ffi::{CStr, c_int};

use crate::ex_eval::state::{
    CURRENT_EXCEPTION, DID_THROW, FORCE_ABORT, MSG_LISTS, NEED_RETHROW, TRYLEVEL,
};
use crate::ex_eval::{
    discard_current_exception, error_exception_string, pop_msg_list, push_msg_list, take_msg_list,
};
use crate::getchar::state::GOT_INT;
use crate::guard::{SavedSctx, Script};
use crate::highlight_group::syn_id2name;
use crate::mark::setmark_at;
use crate::message::state::DID_EMSG;
use crate::os::libc::uint64_t;
use crate::pos::MAXCOL;
use crate::runtime::state::CURRENT_SCTX;
use crate::types::{
    ApiString, BufferHandle, ERROR_TYPE_EXCEPTION, Error, Integer, Pos, TabpageHandle, WindowHandle,
};
use crate::winlayer::{self, Buf, TabPage, Win};

mod keydict;
mod text;
mod value;
mod vimdict;

pub(crate) use self::keydict::*;
pub(crate) use self::text::*;
pub(crate) use self::value::*;
// Reached by name from `crates/nvim/tests/unit`, which links the library from
// outside; the rest of `value` stays in-crate.
pub(crate) use self::vimdict::*;

const NL: u8 = b'\n';
const CAR: u8 = b'\r';

/// Channel ids with the top bit set are not channels at all: they mark a call
/// nvim made of itself, from Vimscript or from Lua.
const INTERNAL_CALL_MASK: uint64_t = 1 << (uint64_t::BITS - 1);
const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL + 1;

use crate::api::private::validate::{Bad, err_bad_number, err_invalid};
use crate::api_error;
use crate::message_fmt::{msg_bytes, msg_cstr};
// -- Handles ---------------------------------------------------------------

// The handle off the wire is an integer, so nothing about the lookup is
// unsafe: the registry only hands back what is live, and the answer is a
// `winlayer` wrapper whose construction discharged the liveness promise
// once.
//
// `Ok(None)` is not a failure. Handle 0 means "the current one", and when
// there is none -- during startup, and after the last window of a tab page
// has gone -- upstream answered null without setting the error, and the
// caller answered a default rather than refusing. Only a *named* handle that
// resolves to nothing is an `Err`.

/// The buffer `buffer` names, or the current one for 0. `Ok(None)` when 0
/// names nothing; `Err` when a named handle does.
pub(crate) fn find_buffer_by_handle(buffer: BufferHandle) -> Result<Option<Buf>, Error> {
    if buffer.is_current() {
        return Ok(Buf::current_or_none());
    }
    match winlayer::buffer(buffer) {
        Some(buf) => Ok(Some(buf)),
        None => Err(err_bad_number(c"buffer id", Integer::from(buffer))),
    }
}

/// [`find_buffer_by_handle`] for a window.
pub(crate) fn find_window_by_handle(window: WindowHandle) -> Result<Option<Win>, Error> {
    if window.is_current() {
        return Ok(Win::current_or_none());
    }
    match winlayer::window(window) {
        Some(win) => Ok(Some(win)),
        None => Err(err_bad_number(c"window id", Integer::from(window))),
    }
}

/// [`find_buffer_by_handle`] for a tab page.
pub(crate) fn find_tab_by_handle(tabpage: TabpageHandle) -> Result<Option<TabPage>, Error> {
    if tabpage.is_current() {
        return Ok(TabPage::current_or_none());
    }
    match winlayer::tabpage(tabpage) {
        Some(tp) => Ok(Some(tp)),
        None => Err(err_bad_number(c"tabpage id", Integer::from(tabpage))),
    }
}

// -- Errors and the try/catch bracket --------------------------------------

/// Start catching what Vimscript throws, saving the state to put back into
/// `tstate`. Pairs with [`try_leave`].
pub(crate) fn try_enter(tstate: &mut TryState) {
    let saved = TryState {
        current_exception: CURRENT_EXCEPTION.get(),
        got_int: c_int::from(GOT_INT.get()),
        did_throw: DID_THROW.get(),
        need_rethrow: c_int::from(NEED_RETHROW.get()),
        did_emsg: DID_EMSG.get(),
    };
    *tstate = saved;
    // Errors go to a list of the call's own from here on, so that an
    // `:echoerr` inside the call does not reach an enclosing `:try`.
    push_msg_list();
    CURRENT_EXCEPTION.set(None);
    GOT_INT.set(false);
    DID_THROW.set(false);
    NEED_RETHROW.set(false);
    DID_EMSG.set(0);
    TRYLEVEL.set(TRYLEVEL.get() + 1);
}

/// Stop catching, answer whatever was caught, and restore what
/// [`try_enter`] saved into `tstate`.
pub(crate) fn try_leave(tstate: &TryState) -> Result<(), Error> {
    debug_assert!(TRYLEVEL.get() > 0);
    TRYLEVEL.set(TRYLEVEL.get() - 1);
    DID_EMSG.set(0);
    FORCE_ABORT.set(false);

    // The call's own list, which `try_enter` started and which stays the
    // innermost until the state is put back below.
    let has_messages =
        MSG_LISTS.with(|lists| lists.innermost().is_some_and(|l| !l.entries.is_empty()));

    let mut caught = None;
    if GOT_INT.get() {
        // An interrupt outranks anything that was thrown along the way.
        if DID_THROW.get() {
            discard_current_exception();
        }
        caught = Some(Error::exception(c"Keyboard interrupt"));
        GOT_INT.set(false);
    } else if has_messages {
        let messages = take_msg_list();
        let msg = error_exception_string(&messages, b"");
        caught = Some(Error::from_message(ERROR_TYPE_EXCEPTION, msg.as_cstr()));
    } else if DID_THROW.get() || NEED_RETHROW.get() {
        // Either flag says there is an exception being unwound.
        let (name, lnum, value) = CURRENT_EXCEPTION
            .get()
            .expect("an exception being thrown")
            .with(|ex| (ex.throw_name.clone(), ex.throw_lnum, ex.value.clone()));
        // `throw_name` is empty for a throw with no script to name.
        if name.is_empty() {
            caught = Some(Error::from_message(ERROR_TYPE_EXCEPTION, value.as_cstr()));
        } else {
            let (name, value) = (msg_cstr(name.as_cstr()), msg_cstr(value.as_cstr()));
            caught = Some(if lnum != LineNr::ZERO {
                api_error!(ERROR_TYPE_EXCEPTION, "{name}, line {lnum}: {value}")
            } else {
                api_error!(ERROR_TYPE_EXCEPTION, "{name}: {value}")
            });
        }
        discard_current_exception();
    }

    // What the matching `try_enter` filled in.
    let saved = *tstate;
    pop_msg_list();
    CURRENT_EXCEPTION.set(saved.current_exception);
    GOT_INT.set(saved.got_int != 0);
    DID_THROW.set(saved.did_throw);
    NEED_RETHROW.set(saved.need_rethrow != 0);
    DID_EMSG.set(saved.did_emsg);

    match caught {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

/// Run `body` with whatever it throws caught, and report that through `err`:
/// the [`try_enter`]/[`try_leave`] pair as one safe call.
///
/// The `TryState` the two halves communicate through is this frame's own and
/// nothing else can name it. `try_enter` overwrites the state before
/// `try_leave` reads it, so the value handed in is never read.
///
/// `body` runs between the two, exactly as the statements between a
/// hand-written pair did. What it *answers* is discarded when something was
/// thrown: an exception outranks whatever the body got as far as computing,
/// which is what a hand-written pair did too by overwriting the slot.
pub(crate) fn api_try<T>(body: impl FnOnce() -> T) -> Result<T, Error> {
    let mut tstate = TryState::default();
    try_enter(&mut tstate);
    let value = body();
    try_leave(&tstate)?;
    Ok(value)
}

// -- Odds and ends ---------------------------------------------------------

/// Set the mark `name` in `buffer` to line/column, or delete it when `line` is
/// 0. Refuses when the position is out of range or the mark name is not one
/// that can be set.
pub(crate) fn set_mark(
    buffer: Option<Buf>,
    name: ApiString,
    line: Integer,
    col: Integer,
) -> Result<(), Error> {
    let buffer = buffer.unwrap_or_else(Buf::current);
    let mut col = col;
    let mut deleting = false;
    let out_of_range = c"out of range";
    if line == 0 {
        col = 0;
        deleting = true;
    } else {
        if col > Integer::from(MAXCOL) {
            let why = Bad::Bare(out_of_range);
            return Err(err_invalid(c"column", why));
        }
        let line_count = Integer::from(buffer.b_ml.ml_line_count);
        if line < 1 || line > line_count {
            let why = Bad::Bare(out_of_range);
            return Err(err_invalid(c"line", why));
        }
    }
    debug_assert!((Integer::from(i32::MIN)..=Integer::from(i32::MAX)).contains(&line));

    let pos = Pos {
        lnum: number_as_lnum(line),
        col: ColNr::new(number_as_int(col)),
        coladd: 0,
    };
    // A byte past ASCII names no mark, as upstream's negative `char` did. A
    // name that is not one character is the caller's check; an empty one
    // reads NUL.
    let mark = name.as_bytes().first().copied().unwrap_or(0);
    let handle = buffer.handle;
    if setmark_at(mark, pos, Some(handle)).is_ok() {
        return Ok(());
    }
    // `%c` wrote the one byte, whatever it was.
    let byte = mark;
    let mark = msg_bytes(core::slice::from_ref(&byte));
    Err(match deleting {
        true => api_error!(ERROR_TYPE_EXCEPTION, "Failed to delete named mark: {mark}"),
        false => api_error!(ERROR_TYPE_EXCEPTION, "Failed to set named mark: {mark}"),
    })
}

/// The highlight group a status line, window bar or status column defaults
/// to when its 'statusline' text names none. A null window is the tab line.
pub(crate) fn get_default_stl_hl(
    window: Option<Win>,
    use_winbar: bool,
    stc_hl_id: c_int,
) -> &'static CStr {
    // `window` is only compared, never followed.
    if window.is_none() {
        c"TabLineFill"
    } else if use_winbar {
        if window == Win::current_or_none() {
            c"WinBar"
        } else {
            c"WinBarNC"
        }
    } else if stc_hl_id > 0 {
        syn_id2name(stc_hl_id)
    } else if window == Win::current_or_none() {
        c"StatusLine"
    } else {
        c"StatusLineNC"
    }
}

/// Point `CURRENT_SCTX` at whoever made this API call, so that `:verbose`
/// and `<sfile>` name them, for as long as the returned guard lives.
///
/// This is the C's `WITH_SCRIPT_CONTEXT`, whose trailing `current_sctx =
/// save_current_sctx` every caller had to write out by hand. Holding the
/// restore in a [`SavedSctx`] means no way out of the call -- an early
/// return, a `?`, a panic -- can leave the editor believing the API client
/// is still the one running. `nvim_create_augroup` returns early on a bad
/// group name and upstream leaks the client's context out of the call that
/// way; here it cannot.
pub(crate) fn api_set_sctx(channel_id: uint64_t) -> SavedSctx {
    let caller = CURRENT_SCTX.get();
    // A call from Vimscript is already running in the right context.
    if channel_id == VIML_INTERNAL_CALL {
        return Script::saved();
    }
    let mut sctx = caller.with_lnum(LineNr::ZERO);
    if channel_id == LUA_INTERNAL_CALL {
        // Unless the caller is a Lua script, which keeps its own id.
        if !script_is_lua(sctx.sc_sid) {
            sctx.sc_sid = SID_LUA;
        }
    } else {
        sctx.sc_sid = SID_API_CLIENT;
        sctx.sc_chan = channel_id;
    }
    Script::context(sctx)
}

#[derive(Copy, Clone, Debug)]
pub struct TryState {
    pub(crate) current_exception: Option<ExcId>,
    pub got_int: ::core::ffi::c_int,
    pub did_throw: bool,
    pub need_rethrow: ::core::ffi::c_int,
    pub did_emsg: ::core::ffi::c_int,
}

impl TryState {
    /// The zeroed state a caller declares before handing it to `try_enter`,
    /// which overwrites every field. Nothing reads one of these before that.
    pub(crate) const INIT: TryState = TryState {
        current_exception: None,
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
}

impl Default for TryState {
    fn default() -> Self {
        Self::INIT
    }
}

/// A vector of owned API strings: ShaDa's register contents, the one keyset
/// field whose value is a list of binaries rather than an [`Object`](crate::types::Object) tree.
#[derive(Clone, Default, Debug)]
pub struct StringArray(Vec<ApiString>);

impl StringArray {
    /// No strings and nothing allocated.
    pub const EMPTY: Self = Self(Vec::new());

    pub fn push(&mut self, str: ApiString) {
        self.0.push(str);
    }

    /// The strings, as the vector they are kept in.
    pub fn into_vec(self) -> Vec<ApiString> {
        self.0
    }
}

impl From<Vec<ApiString>> for StringArray {
    fn from(items: Vec<ApiString>) -> Self {
        Self(items)
    }
}

deref_to_field!(StringArray => [ApiString], 0);

impl<'a> IntoIterator for &'a StringArray {
    type Item = &'a ApiString;
    type IntoIter = ::core::slice::Iter<'a, ApiString>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
