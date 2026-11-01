//! libvterm's C interface, and nothing else.
//!
//! The emulator is Rust; what is C about it is the surface a C consumer
//! drives, which the unit suite's fixture (`test/unit/fixtures/vterm_test.c`),
//! `test/unit/vterm_spec.lua` and the battery's `vtsweep` all reach by name:
//!
//! - the thirty exports of `docs/abi-ledger.md`'s vterm section, here, each a
//!   front over a [`VTerm`] method that recovers the terminal from the pointer
//!   it is handed (a state or screen pointer names its terminal through the
//!   `vt` field kept for exactly this);
//! - the callback tables a C consumer installs, held as the `Foreign*` values
//!   in [`foreign`], which implement the emulator's own report traits
//!   ([`super::host`]) by calling through the table;
//! - the character-set tables the specs call through ([`charset`]), and the
//!   `VTermColor` union's reads.
//!
//! Every `unsafe` here is one of those: dereferencing a pointer a C caller
//! handed in, or calling a function pointer a C caller installed. A table is
//! installed only by an export whose `# Safety` section requires it -- and
//! the data pointer beside it -- to stay valid while installed, and requires
//! its callbacks not to re-enter the terminal; that promise is what every call
//! through one rests on.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `vterm/abi/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

mod charset;
mod foreign;

use core::ffi::{c_char, c_int, c_void};
use core::ptr::NonNull;
use core::slice;

use crate::global_cell::GlobalCell;
use crate::types::{
    ScreenCell, VTerm, VTermColor, VTermKey, VTermLineInfo, VTermModifier, VTermOutputCallback,
    VTermParserCallbacks, VTermPos, VTermScreen, VTermScreenCallbacks, VTermScreenCell,
    VTermSelectionCallbacks, VTermState, VTermStateCallbacks, VTermStateFallbacks, size_t,
    uint32_t,
};
use crate::vterm::host::{Fallbacks, ScreenHost, SelectionHost};
use crate::vterm::mode;
use crate::vterm::pen::convert_color_to_rgb;
use crate::vterm::state::Consumer;
use crate::vterm::vterm::{OutputSink, ParserTarget};

pub use charset::vterm_lookup_encoding;
pub(crate) use charset::{ENCODING_DEC_DRAWING, ENCODING_USASCII, ENCODING_UTF8};
use foreign::{ForeignFallbacks, ForeignScreen, ForeignSelection};
pub(crate) use foreign::{ForeignOutput, ForeignParser, ForeignState};

// ------------------------------------------------------------- the colour

impl VTermColor {
    /// The type byte, which every arm of the union starts with.
    pub(crate) fn type_byte(&self) -> u8 {
        // SAFETY: every arm of the union is bytes and starts with the type
        // byte, and every colour is built whole (`color::UNSET` for an
        // unset one), so the byte is initialised whichever arm wrote it.
        unsafe { self.type_0 }
    }

    /// The palette index an indexed colour names.
    pub(crate) fn index(&self) -> u8 {
        // SAFETY: as for `type_byte`; the index is the second byte.
        unsafe { self.indexed.idx }
    }

    /// The red, green and blue channels of a literal colour.
    pub(crate) fn channels(&self) -> (u8, u8, u8) {
        // SAFETY: as for `type_byte`; colours are built through the four-byte
        // RGB arm, so all four bytes are initialised.
        let rgb = unsafe { self.rgb };
        (rgb.red, rgb.green, rgb.blue)
    }
}

// ------------------------------------------------------------ the terminal

/// Primary Device Attributes (DA1) response, `61;22;52` and its terminator.
/// Exported so that the TUI tests can substitute another terminal's answer
/// through FFI, which is why it is spelled as the `c_char` array the wire
/// wants rather than as a byte string.
#[unsafe(no_mangle)]
pub static vterm_primary_device_attr: GlobalCell<[c_char; 9]> = GlobalCell::new([
    b'6'.cast_signed(),
    b'1'.cast_signed(),
    b';'.cast_signed(),
    b'2'.cast_signed(),
    b'2'.cast_signed(),
    b';'.cast_signed(),
    b'5'.cast_signed(),
    b'2'.cast_signed(),
    0,
]);

/// A terminal `rows` by `cols`. It is never freed: nothing C-side owns one
/// for less than the process.
#[unsafe(no_mangle)]
pub extern "C" fn vterm_new(rows: c_int, cols: c_int) -> *mut VTerm {
    Box::into_raw(VTerm::new(rows, cols))
}

/// # Safety
///
/// `vt` must point at a live `VTerm`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_set_size(vt: *mut VTerm, rows: c_int, cols: c_int) {
    // SAFETY: the caller's promise.
    unsafe { &mut *vt }.set_size(rows, cols);
}

/// # Safety
///
/// `vt` must point at a live `VTerm`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_set_utf8(vt: *mut VTerm, is_utf8: c_int) {
    // SAFETY: the caller's promise. The bit takes the low bit of the `int`,
    // as libvterm's bitfield did.
    unsafe { &mut *vt }.mode.set_utf8(is_utf8.cast_unsigned());
}

/// Installs the sink replies are written to; null collects them in the
/// terminal's own buffer.
///
/// # Safety
///
/// `vt` must point at a live `VTerm`, unaliased for the call. `func` and
/// `user` must stay valid for as long as they are installed, and `func` must
/// not re-enter the terminal.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_output_set_callback(
    vt: *mut VTerm,
    func: Option<VTermOutputCallback>,
    user: *mut c_void,
) {
    let sink = match func {
        Some(func) => OutputSink::Foreign(ForeignOutput { func, data: user }),
        None => OutputSink::Buffer,
    };
    // SAFETY: the caller's promise.
    unsafe { &mut *vt }.set_output(sink);
}

/// # Safety
///
/// `vt` must point at a live `VTerm`, unaliased for the call. `bytes` must
/// point at `len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_input_write(
    vt: *mut VTerm,
    bytes: *const c_char,
    len: size_t,
) -> size_t {
    // SAFETY: the caller's promise.
    let input = unsafe { slice::from_raw_parts(bytes.cast::<u8>(), len) };
    // SAFETY: the caller's promise.
    unsafe { &mut *vt }.input_write(input);
    len
}

/// Installs a C consumer's parser callbacks, in place of the state machine's
/// if it had one.
///
/// # Safety
///
/// `vt` must point at a live `VTerm`, unaliased for the call. `callbacks` and
/// `user` must stay valid for as long as they are installed, and the
/// callbacks must not re-enter the terminal.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_parser_set_callbacks(
    vt: *mut VTerm,
    callbacks: *const VTermParserCallbacks,
    user: *mut c_void,
) {
    // SAFETY: the caller's promise.
    let vt = unsafe { &mut *vt };
    vt.parser.callbacks = callbacks;
    vt.parser.cbdata = user;
    let target = match NonNull::new(callbacks.cast_mut()) {
        Some(table) => ParserTarget::Foreign(ForeignParser { table, data: user }),
        None => ParserTarget::None,
    };
    vt.set_parser_target(target);
}

/// # Safety
///
/// `vt` must point at a live `VTerm`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_obtain_state(vt: *mut VTerm) -> *mut VTermState {
    // SAFETY: the caller's promise.
    core::ptr::from_mut(unsafe { &mut *vt }.obtain_state())
}

/// # Safety
///
/// `vt` must point at a live `VTerm`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_obtain_screen(vt: *mut VTerm) -> *mut VTermScreen {
    // SAFETY: the caller's promise.
    core::ptr::from_mut(unsafe { &mut *vt }.obtain_screen())
}

/// # Safety
///
/// `vt` must point at a live `VTerm`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_keyboard_unichar(vt: *mut VTerm, c: uint32_t, mods: VTermModifier) {
    // SAFETY: the caller's promise.
    unsafe { &mut *vt }.keyboard_unichar(c, mods);
}

/// # Safety
///
/// `vt` must point at a live `VTerm`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_keyboard_key(vt: *mut VTerm, key: VTermKey, mods: VTermModifier) {
    // SAFETY: the caller's promise.
    unsafe { &mut *vt }.keyboard_key(key, mods);
}

/// # Safety
///
/// `vt` must point at a live `VTerm`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_keyboard_start_paste(vt: *mut VTerm) {
    // SAFETY: the caller's promise.
    unsafe { &mut *vt }.keyboard_start_paste();
}

/// # Safety
///
/// `vt` must point at a live `VTerm`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_keyboard_end_paste(vt: *mut VTerm) {
    // SAFETY: the caller's promise.
    unsafe { &mut *vt }.keyboard_end_paste();
}

/// # Safety
///
/// `vt` must point at a live `VTerm`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_mouse_move(
    vt: *mut VTerm,
    row: c_int,
    col: c_int,
    mods: VTermModifier,
) {
    // SAFETY: the caller's promise.
    unsafe { &mut *vt }.mouse_move(row, col, mods);
}

/// # Safety
///
/// `vt` must point at a live `VTerm`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_mouse_button(
    vt: *mut VTerm,
    button: c_int,
    pressed: bool,
    mods: VTermModifier,
) {
    // SAFETY: the caller's promise.
    unsafe { &mut *vt }.mouse_button(button, pressed, mods);
}

// --------------------------------------------------------------- the state

/// The terminal a state pointer belongs to.
///
/// # Safety
///
/// `state` must point at a live `VTermState`; its terminal is then live too,
/// and the caller must not use `state` while the answer is borrowed.
unsafe fn state_terminal<'a>(state: *const VTermState) -> &'a mut VTerm {
    // SAFETY: the caller's promise; a state's `vt` is the terminal that
    // made it and owns it.
    unsafe { &mut *(*state).vt }
}

/// # Safety
///
/// `state` must point at a live `VTermState`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_reset(state: *mut VTermState, hard: c_int) {
    // SAFETY: the caller's promise.
    let vt = unsafe { state_terminal(state) };
    if let Some(mut state) = vt.state_cx() {
        mode::reset(&mut state, hard != 0);
    }
}

/// Installs a C consumer's state callbacks, in place of the screen if the
/// terminal has one.
///
/// # Safety
///
/// `state` must point at a live `VTermState`, unaliased for the call.
/// `callbacks` and `user` must stay valid for as long as they are installed,
/// and the callbacks must not re-enter the terminal.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_set_callbacks(
    state: *mut VTermState,
    callbacks: *const VTermStateCallbacks,
    user: *mut c_void,
) {
    // SAFETY: the caller's promise.
    let state = unsafe { &mut *state };
    let consumer =
        NonNull::new(callbacks.cast_mut()).map(|table| ForeignState { table, data: user });
    state.callbacks = callbacks;
    state.cbdata = if consumer.is_some() {
        user
    } else {
        core::ptr::null_mut()
    };
    state.set_consumer(consumer.map_or(Consumer::None, Consumer::Foreign));
    if let Some(consumer) = consumer {
        let _ = consumer.initpen();
    }
}

/// # Safety
///
/// `state` must point at a live `VTermState`, unaliased for the call.
/// `fallbacks` and `user` must stay valid for as long as they are installed,
/// and the callbacks must not re-enter the terminal.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_set_unrecognised_fallbacks(
    state: *mut VTermState,
    fallbacks: *const VTermStateFallbacks,
    user: *mut c_void,
) {
    // SAFETY: the caller's promise.
    let state = unsafe { &mut *state };
    let table = NonNull::new(fallbacks.cast_mut());
    state.fallbacks = fallbacks;
    state.fbdata = if table.is_some() {
        user
    } else {
        core::ptr::null_mut()
    };
    state.set_fallbacks(
        table.map(|table| Box::new(ForeignFallbacks { table, data: user }) as Box<dyn Fallbacks>),
    );
}

/// Installs a C consumer's OSC 52 callbacks. The state machine decodes into
/// a staging buffer of its own of `buflen` bytes; a `buffer` the caller
/// offers is not used.
///
/// # Safety
///
/// `state` must point at a live `VTermState`, unaliased for the call.
/// `callbacks` and `user` must stay valid for as long as they are installed,
/// and the callbacks must not re-enter the terminal.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_set_selection_callbacks(
    state: *mut VTermState,
    callbacks: *const VTermSelectionCallbacks,
    user: *mut c_void,
    _buffer: *mut c_char,
    buflen: size_t,
) {
    // SAFETY: the caller's promise.
    let state = unsafe { &mut *state };
    state.selection.callbacks = callbacks;
    state.selection.user = user;
    let host = NonNull::new(callbacks.cast_mut())
        .map(|table| Box::new(ForeignSelection { table, data: user }) as Box<dyn SelectionHost>);
    state.set_selection(host, buflen);
}

/// Reports that the terminal window gained focus, if the host asked to hear.
///
/// # Safety
///
/// `state` must point at a live `VTermState`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_focus_in(state: *mut VTermState) {
    // SAFETY: the caller's promise.
    if let Some(mut state) = unsafe { state_terminal(state) }.state_cx() {
        state.report_focus(b'I');
    }
}

/// Reports that the terminal window lost focus, if the host asked to hear.
///
/// # Safety
///
/// `state` must point at a live `VTermState`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_focus_out(state: *mut VTermState) {
    // SAFETY: the caller's promise.
    if let Some(mut state) = unsafe { state_terminal(state) }.state_cx() {
        state.report_focus(b'O');
    }
}

/// Where row `row`'s line marks are. Not bounds-checked, as libvterm did not
/// check it: the answer is only valid for a row of the state.
///
/// # Safety
///
/// `state` must point at a live `VTermState`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_get_lineinfo(
    state: *const VTermState,
    row: c_int,
) -> *const VTermLineInfo {
    // SAFETY: the caller's promise.
    let state = unsafe { &*state };
    state
        .lineinfo_base()
        .wrapping_offset(isize::try_from(row).unwrap_or(isize::MAX))
}

// -------------------------------------------------------------- the screen

/// The terminal a screen pointer belongs to.
///
/// # Safety
///
/// `screen` must point at a live `VTermScreen`; its terminal is then live
/// too, and the caller must not use `screen` while the answer is borrowed.
unsafe fn screen_terminal<'a>(screen: *const VTermScreen) -> &'a mut VTerm {
    // SAFETY: the caller's promise; a screen's `vt` is the terminal that
    // made it and owns it.
    unsafe { &mut *(*screen).vt }
}

/// The cell at `row`/`col`, or null outside the grid.
///
/// # Safety
///
/// `screen` must point at a live `VTermScreen`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getcell(
    screen: *const VTermScreen,
    row: c_int,
    col: c_int,
) -> *mut ScreenCell {
    // SAFETY: the caller's promise.
    let screen = unsafe { &mut *screen.cast_mut() };
    screen
        .cell_mut(row, col)
        .map_or(core::ptr::null_mut(), core::ptr::from_mut)
}

/// # Safety
///
/// `screen` must point at a live `VTermScreen`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_reset(screen: *mut VTermScreen, hard: c_int) {
    // SAFETY: the caller's promise.
    unsafe { screen_terminal(screen) }.screen_reset(hard != 0);
}

/// Copies the cell at `pos` into its reported form. Returns 0 for a position
/// outside the screen.
///
/// # Safety
///
/// `screen` must point at a live `VTermScreen`. `cell` must point at a live
/// `VTermScreenCell`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_get_cell(
    screen: *const VTermScreen,
    pos: VTermPos,
    cell: *mut VTermScreenCell,
) -> c_int {
    // SAFETY: the caller's promise.
    let (screen, out) = unsafe { (&*screen, &mut *cell) };
    screen.export_cell(pos, out)
}

/// # Safety
///
/// `screen` must point at a live `VTermScreen`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_enable_reflow(screen: *mut VTermScreen, reflow: bool) {
    // SAFETY: the caller's promise.
    unsafe { &mut *screen }.enable_reflow(reflow);
}

/// Allocates the alternate screen buffer, which the terminal starts without.
///
/// # Safety
///
/// `screen` must point at a live `VTermScreen`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_enable_altscreen(screen: *mut VTermScreen, altscreen: c_int) {
    // SAFETY: the caller's promise.
    let vt = unsafe { screen_terminal(screen) };
    if altscreen != 0 {
        vt.screen_enable_altscreen();
    }
}

/// # Safety
///
/// `screen` must point at a live `VTermScreen`, unaliased for the call.
/// `callbacks` and `user` must stay valid for as long as they are installed,
/// and the callbacks must not re-enter the terminal.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_set_callbacks(
    screen: *mut VTermScreen,
    callbacks: *const VTermScreenCallbacks,
    user: *mut c_void,
) {
    // SAFETY: the caller's promise.
    let screen = unsafe { &mut *screen };
    screen.callbacks = callbacks;
    screen.cbdata = user;
    let host = NonNull::new(callbacks.cast_mut())
        .map(|table| Box::new(ForeignScreen { table, data: user }) as Box<dyn ScreenHost>);
    screen.set_host(host);
}

/// [`convert_color_to_rgb`] against a screen's state machine.
///
/// # Safety
///
/// `screen` must point at a live `VTermScreen`. `col` must point at a live
/// `VTermColor`, unaliased for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_convert_color_to_rgb(
    screen: *const VTermScreen,
    col: *mut VTermColor,
) {
    // SAFETY: the caller's promise.
    let (vt, col) = unsafe { (screen_terminal(screen), &mut *col) };
    if let Some(state) = vt.inner.state.as_deref() {
        convert_color_to_rgb(state, col);
    }
}
