//! `VTerm` Callbacks and C Bridge
//!
//! This module provides C-compatible callback wrappers and output functions
//! for the `VTerm` terminal emulator. It bridges Rust implementations with
//! C code that needs to provide or receive callbacks.

#![allow(clippy::module_name_repetitions)]
#![allow(clippy::option_if_let_else)] // Clearer with if-let for callbacks

use std::ffi::{c_char, c_int, c_long, c_void};
use std::ptr;
use std::slice;

use crate::parser::VTermParserCallbacks;
use crate::screen::VTermScreenCallbacks;
use crate::state::VTermStateCallbacks;
use crate::{VTermPos, VTermProp, VTermRect, VTermScreenCell, VTermStringFragment, VTermValue};

// =============================================================================
// Output Callback Types
// =============================================================================

/// Output callback function signature.
///
/// Called when the terminal needs to send data back to the PTY.
pub type OutputCallback = unsafe extern "C" fn(bytes: *const c_char, len: usize, user: *mut c_void);

/// Output context holding the callback and user data.
#[repr(C)]
pub struct OutputContext {
    /// The output callback function (if set).
    pub callback: Option<OutputCallback>,
    /// User data passed to the callback.
    pub user_data: *mut c_void,
}

impl Default for OutputContext {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputContext {
    /// Create a new output context with no callback.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            callback: None,
            user_data: ptr::null_mut(),
        }
    }

    /// Set the output callback.
    pub fn set_callback(&mut self, callback: Option<OutputCallback>, user_data: *mut c_void) {
        self.callback = callback;
        self.user_data = user_data;
    }

    /// Call the output callback with data.
    ///
    /// # Safety
    /// The callback must be valid and the `user_data` must match what the callback expects.
    pub unsafe fn output(&self, bytes: &[u8]) {
        if let Some(cb) = self.callback {
            cb(bytes.as_ptr().cast::<c_char>(), bytes.len(), self.user_data);
        }
    }

    /// Check if a callback is set.
    #[must_use]
    pub const fn has_callback(&self) -> bool {
        self.callback.is_some()
    }
}

// =============================================================================
// Output Buffer
// =============================================================================

/// Output buffer for accumulating terminal output.
///
/// Used when no callback is set, allowing output to be buffered
/// and retrieved later.
pub struct OutputBuffer {
    /// Internal buffer for output data.
    buffer: Vec<u8>,
    /// Maximum buffer capacity.
    max_capacity: usize,
}

impl Default for OutputBuffer {
    fn default() -> Self {
        Self::new(crate::VTERM_DEFAULT_OUTBUFFER_LEN)
    }
}

impl OutputBuffer {
    /// Create a new output buffer with the given capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            max_capacity: capacity,
        }
    }

    /// Push bytes to the buffer.
    ///
    /// Returns the number of bytes actually written (may be less if buffer is full).
    pub fn push(&mut self, bytes: &[u8]) -> usize {
        let available = self.max_capacity.saturating_sub(self.buffer.len());
        let to_write = bytes.len().min(available);
        self.buffer.extend_from_slice(&bytes[..to_write]);
        to_write
    }

    /// Get the current buffer contents.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    /// Get the current buffer length.
    #[must_use]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the buffer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Clear the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Read from the buffer into a destination slice.
    ///
    /// Returns the number of bytes read.
    pub fn read_into(&mut self, dest: &mut [u8]) -> usize {
        let to_read = dest.len().min(self.buffer.len());
        dest[..to_read].copy_from_slice(&self.buffer[..to_read]);
        // Remove read bytes from buffer
        self.buffer.drain(..to_read);
        to_read
    }
}

// =============================================================================
// Screen Callback Wrappers
// =============================================================================

/// Screen callback context combining callbacks and user data.
#[repr(C)]
pub struct ScreenCallbackContext {
    /// Screen callbacks.
    pub callbacks: *const VTermScreenCallbacks,
    /// User data for callbacks.
    pub user_data: *mut c_void,
}

impl Default for ScreenCallbackContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenCallbackContext {
    /// Create a new screen callback context.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            callbacks: ptr::null(),
            user_data: ptr::null_mut(),
        }
    }

    /// Set callbacks and user data.
    pub fn set_callbacks(&mut self, callbacks: *const VTermScreenCallbacks, user: *mut c_void) {
        self.callbacks = callbacks;
        self.user_data = user;
    }

    /// Invoke the damage callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn damage(&self, rect: VTermRect) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).damage {
            cb(rect, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the moverect callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn moverect(&self, dest: VTermRect, src: VTermRect) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).moverect {
            cb(dest, src, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the movecursor callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn movecursor(&self, pos: VTermPos, oldpos: VTermPos, visible: c_int) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).movecursor {
            cb(pos, oldpos, visible, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the settermprop callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn settermprop(&self, prop: VTermProp, val: *mut VTermValue) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).settermprop {
            cb(prop, val, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the bell callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn bell(&self) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).bell {
            cb(self.user_data)
        } else {
            0
        }
    }

    /// Invoke the resize callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn resize(&self, rows: c_int, cols: c_int) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).resize {
            cb(rows, cols, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the theme callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn theme(&self, dark: *mut bool) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).theme {
            cb(dark, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the `sb_pushline` callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn sb_pushline(&self, cols: c_int, cells: *const VTermScreenCell) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).sb_pushline {
            cb(cols, cells, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the `sb_popline` callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn sb_popline(&self, cols: c_int, cells: *mut VTermScreenCell) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).sb_popline {
            cb(cols, cells, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the `sb_clear` callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn sb_clear(&self) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).sb_clear {
            cb(self.user_data)
        } else {
            0
        }
    }
}

// =============================================================================
// State Callback Wrappers
// =============================================================================

/// State callback context combining callbacks and user data.
#[repr(C)]
pub struct StateCallbackContext {
    /// State callbacks.
    pub callbacks: *const VTermStateCallbacks,
    /// User data for callbacks.
    pub user_data: *mut c_void,
}

impl Default for StateCallbackContext {
    fn default() -> Self {
        Self::new()
    }
}

impl StateCallbackContext {
    /// Create a new state callback context.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            callbacks: ptr::null(),
            user_data: ptr::null_mut(),
        }
    }

    /// Set callbacks and user data.
    pub fn set_callbacks(&mut self, callbacks: *const VTermStateCallbacks, user: *mut c_void) {
        self.callbacks = callbacks;
        self.user_data = user;
    }

    /// Invoke the putglyph callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn putglyph(&self, info: *const crate::VTermGlyphInfo, pos: VTermPos) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).putglyph {
            cb(info, pos, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the movecursor callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn movecursor(&self, pos: VTermPos, oldpos: VTermPos, visible: c_int) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).movecursor {
            cb(pos, oldpos, visible, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the scrollrect callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn scrollrect(&self, rect: VTermRect, downward: c_int, rightward: c_int) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).scrollrect {
            cb(rect, downward, rightward, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the moverect callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn moverect(&self, dest: VTermRect, src: VTermRect) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).moverect {
            cb(dest, src, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the erase callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn erase(&self, rect: VTermRect, selective: c_int) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).erase {
            cb(rect, selective, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the initpen callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn initpen(&self) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).initpen {
            cb(self.user_data)
        } else {
            0
        }
    }

    /// Invoke the setpenattr callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn setpenattr(&self, attr: c_int, val: *const VTermValue) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).setpenattr {
            cb(attr, val, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the settermprop callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn settermprop(&self, prop: c_int, val: *const VTermValue) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).settermprop {
            cb(prop, val, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the bell callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn bell(&self) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).bell {
            cb(self.user_data)
        } else {
            0
        }
    }

    /// Invoke the resize callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn resize(
        &self,
        rows: c_int,
        cols: c_int,
        fields: *mut crate::VTermStateFields,
    ) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).resize {
            cb(rows, cols, fields, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the setlineinfo callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn setlineinfo(
        &self,
        row: c_int,
        newinfo: *const crate::VTermLineInfo,
        oldinfo: *const crate::VTermLineInfo,
    ) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).setlineinfo {
            cb(row, newinfo, oldinfo, self.user_data)
        } else {
            0
        }
    }
}

// =============================================================================
// Parser Callback Wrappers
// =============================================================================

/// Parser callback context combining callbacks and user data.
#[repr(C)]
pub struct ParserCallbackContext {
    /// Parser callbacks.
    pub callbacks: *const VTermParserCallbacks,
    /// User data for callbacks.
    pub user_data: *mut c_void,
}

impl Default for ParserCallbackContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserCallbackContext {
    /// Create a new parser callback context.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            callbacks: ptr::null(),
            user_data: ptr::null_mut(),
        }
    }

    /// Set callbacks and user data.
    pub fn set_callbacks(&mut self, callbacks: *const VTermParserCallbacks, user: *mut c_void) {
        self.callbacks = callbacks;
        self.user_data = user;
    }

    /// Invoke the text callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn text(&self, bytes: *const c_char, len: usize) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).text {
            cb(bytes, len, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the control callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn control(&self, control: u8) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).control {
            cb(control, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the escape callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn escape(&self, bytes: *const c_char, len: usize) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).escape {
            cb(bytes, len, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the CSI callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn csi(
        &self,
        leader: *const c_char,
        args: *const c_long,
        argcount: c_int,
        intermed: *const c_char,
        command: c_char,
    ) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).csi {
            cb(leader, args, argcount, intermed, command, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the OSC callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn osc(&self, command: c_int, frag: VTermStringFragment) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).osc {
            cb(command, frag, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the DCS callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn dcs(
        &self,
        command: *const c_char,
        commandlen: usize,
        frag: VTermStringFragment,
    ) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).dcs {
            cb(command, commandlen, frag, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the APC callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn apc(&self, frag: VTermStringFragment) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).apc {
            cb(frag, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the PM callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn pm(&self, frag: VTermStringFragment) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).pm {
            cb(frag, self.user_data)
        } else {
            0
        }
    }

    /// Invoke the SOS callback if set.
    ///
    /// # Safety
    /// The callbacks must be valid.
    #[must_use]
    pub unsafe fn sos(&self, frag: VTermStringFragment) -> c_int {
        if self.callbacks.is_null() {
            return 0;
        }
        if let Some(cb) = (*self.callbacks).sos {
            cb(frag, self.user_data)
        } else {
            0
        }
    }
}

// =============================================================================
// C FFI Functions
// =============================================================================

/// Set the output callback for a vterm instance.
///
/// # Safety
/// All pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_output_set_callback(
    ctx: *mut OutputContext,
    callback: Option<OutputCallback>,
    user: *mut c_void,
) {
    if let Some(ctx) = ctx.as_mut() {
        ctx.set_callback(callback, user);
    }
}

/// Push output bytes through the output callback or buffer.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_push_output_bytes(
    ctx: *const OutputContext,
    bytes: *const c_char,
    len: usize,
) {
    if ctx.is_null() || bytes.is_null() {
        return;
    }
    let ctx = &*ctx;
    let byte_slice = slice::from_raw_parts(bytes.cast::<u8>(), len);
    ctx.output(byte_slice);
}

/// Set screen callbacks.
///
/// # Safety
/// All pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_screen_set_callbacks(
    ctx: *mut ScreenCallbackContext,
    callbacks: *const VTermScreenCallbacks,
    user: *mut c_void,
) {
    if let Some(ctx) = ctx.as_mut() {
        ctx.set_callbacks(callbacks, user);
    }
}

/// Set state callbacks.
///
/// # Safety
/// All pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_set_callbacks(
    ctx: *mut StateCallbackContext,
    callbacks: *const VTermStateCallbacks,
    user: *mut c_void,
) {
    if let Some(ctx) = ctx.as_mut() {
        ctx.set_callbacks(callbacks, user);
    }
}

/// Set parser callbacks.
///
/// # Safety
/// All pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_parser_set_callbacks(
    ctx: *mut ParserCallbackContext,
    callbacks: *const VTermParserCallbacks,
    user: *mut c_void,
) {
    if let Some(ctx) = ctx.as_mut() {
        ctx.set_callbacks(callbacks, user);
    }
}

/// Create a new output buffer.
///
/// # Safety
/// The returned pointer must be freed with `rs_output_buffer_free`.
#[no_mangle]
pub extern "C" fn rs_output_buffer_new(capacity: usize) -> *mut OutputBuffer {
    Box::into_raw(Box::new(OutputBuffer::new(capacity)))
}

/// Free an output buffer.
///
/// # Safety
/// The pointer must have been created by `rs_output_buffer_new`.
#[no_mangle]
pub unsafe extern "C" fn rs_output_buffer_free(buf: *mut OutputBuffer) {
    if !buf.is_null() {
        drop(Box::from_raw(buf));
    }
}

/// Push bytes to an output buffer.
///
/// # Safety
/// Both pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_output_buffer_push(
    buf: *mut OutputBuffer,
    bytes: *const c_char,
    len: usize,
) -> usize {
    if buf.is_null() || bytes.is_null() {
        return 0;
    }
    let buf = &mut *buf;
    let byte_slice = slice::from_raw_parts(bytes.cast::<u8>(), len);
    buf.push(byte_slice)
}

/// Get the current length of an output buffer.
///
/// # Safety
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_output_buffer_len(buf: *const OutputBuffer) -> usize {
    if buf.is_null() {
        return 0;
    }
    (*buf).len()
}

/// Read from an output buffer into a destination.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_output_buffer_read(
    buf: *mut OutputBuffer,
    dest: *mut c_char,
    len: usize,
) -> usize {
    if buf.is_null() || dest.is_null() {
        return 0;
    }
    let buf = &mut *buf;
    let dest_slice = slice::from_raw_parts_mut(dest.cast::<u8>(), len);
    buf.read_into(dest_slice)
}

/// Clear an output buffer.
///
/// # Safety
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_output_buffer_clear(buf: *mut OutputBuffer) {
    if let Some(buf) = buf.as_mut() {
        buf.clear();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_context_new() {
        let ctx = OutputContext::new();
        assert!(!ctx.has_callback());
        assert!(ctx.user_data.is_null());
    }

    #[test]
    fn test_output_context_set_callback() {
        unsafe extern "C" fn dummy_callback(
            _bytes: *const c_char,
            _len: usize,
            _user: *mut c_void,
        ) {
        }

        let mut ctx = OutputContext::new();
        let user_data = 0x1234 as *mut c_void;
        ctx.set_callback(Some(dummy_callback), user_data);
        assert!(ctx.has_callback());
        assert_eq!(ctx.user_data, user_data);
    }

    #[test]
    fn test_output_buffer_new() {
        let buf = OutputBuffer::new(100);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_output_buffer_push() {
        let mut buf = OutputBuffer::new(100);
        let written = buf.push(b"hello");
        assert_eq!(written, 5);
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.as_slice(), b"hello");
    }

    #[test]
    fn test_output_buffer_push_overflow() {
        let mut buf = OutputBuffer::new(10);
        let written = buf.push(b"hello world and more");
        assert_eq!(written, 10);
        assert_eq!(buf.len(), 10);
        assert_eq!(buf.as_slice(), b"hello worl");
    }

    #[test]
    fn test_output_buffer_read_into() {
        let mut buf = OutputBuffer::new(100);
        buf.push(b"hello world");
        let mut dest = [0u8; 5];
        let read = buf.read_into(&mut dest);
        assert_eq!(read, 5);
        assert_eq!(&dest, b"hello");
        assert_eq!(buf.len(), 6); // " world" remaining
        assert_eq!(buf.as_slice(), b" world");
    }

    #[test]
    fn test_output_buffer_clear() {
        let mut buf = OutputBuffer::new(100);
        buf.push(b"hello");
        buf.clear();
        assert!(buf.is_empty());
    }

    #[test]
    fn test_screen_callback_context_new() {
        let ctx = ScreenCallbackContext::new();
        assert!(ctx.callbacks.is_null());
        assert!(ctx.user_data.is_null());
    }

    #[test]
    fn test_state_callback_context_new() {
        let ctx = StateCallbackContext::new();
        assert!(ctx.callbacks.is_null());
        assert!(ctx.user_data.is_null());
    }

    #[test]
    fn test_parser_callback_context_new() {
        let ctx = ParserCallbackContext::new();
        assert!(ctx.callbacks.is_null());
        assert!(ctx.user_data.is_null());
    }

    #[test]
    fn test_ffi_output_buffer() {
        unsafe {
            let buf = rs_output_buffer_new(100);
            assert!(!buf.is_null());

            let data = b"test data\0";
            let written = rs_output_buffer_push(buf, data.as_ptr().cast(), 9);
            assert_eq!(written, 9);

            let len = rs_output_buffer_len(buf);
            assert_eq!(len, 9);

            let mut dest = [0u8; 20];
            let read = rs_output_buffer_read(buf, dest.as_mut_ptr().cast(), 4);
            assert_eq!(read, 4);
            assert_eq!(&dest[..4], b"test");

            rs_output_buffer_clear(buf);
            let len = rs_output_buffer_len(buf);
            assert_eq!(len, 0);

            rs_output_buffer_free(buf);
        }
    }

    #[test]
    fn test_screen_callbacks_null_safety() {
        let ctx = ScreenCallbackContext::new();
        unsafe {
            // These should all return 0 without crashing
            assert_eq!(ctx.damage(VTermRect::default()), 0);
            assert_eq!(ctx.moverect(VTermRect::default(), VTermRect::default()), 0);
            assert_eq!(
                ctx.movecursor(VTermPos::default(), VTermPos::default(), 1),
                0
            );
            assert_eq!(ctx.bell(), 0);
            assert_eq!(ctx.resize(24, 80), 0);
        }
    }

    #[test]
    fn test_state_callbacks_null_safety() {
        let ctx = StateCallbackContext::new();
        unsafe {
            assert_eq!(
                ctx.movecursor(VTermPos::default(), VTermPos::default(), 1),
                0
            );
            assert_eq!(ctx.moverect(VTermRect::default(), VTermRect::default()), 0);
            assert_eq!(ctx.erase(VTermRect::default(), 0), 0);
            assert_eq!(ctx.initpen(), 0);
            assert_eq!(ctx.bell(), 0);
        }
    }

    #[test]
    fn test_parser_callbacks_null_safety() {
        let ctx = ParserCallbackContext::new();
        unsafe {
            assert_eq!(ctx.text(b"test".as_ptr().cast(), 4), 0);
            assert_eq!(ctx.control(0x0D), 0);
            assert_eq!(ctx.escape(b"[".as_ptr().cast(), 1), 0);
        }
    }
}
