#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use std::collections::VecDeque;

use super::*;

/// How much of a selection sequence vterm can reassemble before it gives up
/// and reports the write truncated.
pub const SELECTIONBUF_SIZE: usize = 0x400;

/// One `:terminal` buffer's emulator state.
///
/// Owned by the buffer it draws into: `terminal_alloc` leaks a `Box` and
/// `terminal_destroy` reclaims it. Not `Copy` — there is exactly one of
/// these per terminal buffer, reached through `buf_T::terminal`, and
/// duplicating it would duplicate the allocations it owns.
///
/// Still `repr(C)`: `buf_T` is `repr(C)` and holds a `*mut Terminal`, so
/// the FFI-safety lint follows the pointer into this definition.
#[repr(C)]
pub struct terminal {
    pub opts: TerminalOptions,
    pub vt: *mut VTerm,
    pub vts: *mut VTermScreen,
    /// Where a screen row is rendered to text before being written into
    /// the buffer's lines. Grown to fit the widest row seen and then
    /// reused, so the refresh does not allocate per line.
    pub textbuf: Vec<::core::ffi::c_char>,
    pub sb: Scrollback,
    pub old_sb_deleted: usize,
    pub old_height: ::core::ffi::c_int,
    /// The window title being reassembled from the fragments vterm hands
    /// over. Empty between titles.
    pub title: Vec<u8>,
    pub buf_handle: handle_T,
    pub in_altscreen: bool,
    pub suspended: bool,
    pub closed: bool,
    pub destroy: bool,
    pub forward_mouse: bool,
    pub invalid_start: ::core::ffi::c_int,
    pub invalid_end: ::core::ffi::c_int,
    pub cursor: TerminalCursor,
    pub pending: TerminalPending,
    pub streamed_paste: bool,
    pub theme_updates: bool,
    pub synchronized_output: bool,
    pub sync_flush_pending: bool,
    pub color_set: [bool; 16],
    /// Scratch space vterm reassembles selection sequences in. Handed over
    /// as a raw pointer when the callbacks are installed and held by vterm
    /// for as long as the terminal lives, so it is never resized.
    pub selection_buffer: Box<[::core::ffi::c_char; SELECTIONBUF_SIZE]>,
    /// The clipboard write being reassembled. Empty between writes.
    pub selection: Vec<u8>,
    /// The escape sequence currently being reassembled from the fragments
    /// vterm hands over, terminator excluded.
    pub termrequest_buffer: Vec<u8>,
    pub termrequest_terminator: VTermTerminator,
    pub refcount: size_t,
}
impl terminal {
    /// A terminal with everything at rest, for `terminal_alloc` to fill in.
    ///
    /// This stands in for the `xcalloc` the transpiled code used: every
    /// field starts as the all-zeroes value C would have produced, so the
    /// initialisation that follows is the same one the C did.
    pub fn new(opts: TerminalOptions, buf_handle: handle_T) -> Self {
        Self {
            opts,
            buf_handle,
            vt: ::core::ptr::null_mut(),
            vts: ::core::ptr::null_mut(),
            textbuf: Vec::new(),
            sb: Scrollback::default(),
            old_sb_deleted: 0,
            old_height: 0,
            title: Vec::new(),
            in_altscreen: false,
            suspended: false,
            closed: false,
            destroy: false,
            forward_mouse: false,
            invalid_start: 0,
            invalid_end: 0,
            cursor: TerminalCursor {
                row: 0,
                col: 0,
                shape: 0,
                visible: false,
                blink: false,
            },
            pending: TerminalPending {
                resize: false,
                cursor: false,
                send: ::core::ptr::null_mut(),
                events: ::core::ptr::null_mut(),
            },
            streamed_paste: false,
            theme_updates: false,
            synchronized_output: false,
            sync_flush_pending: false,
            color_set: [false; 16],
            selection_buffer: vec![0; SELECTIONBUF_SIZE]
                .into_boxed_slice()
                .try_into()
                .expect("sized to SELECTIONBUF_SIZE"),
            selection: Vec::new(),
            termrequest_buffer: Vec::new(),
            termrequest_terminator: 0,
            refcount: 0,
        }
    }
}

/// The rows that have scrolled off the top of a terminal's screen.
///
/// Newest first: index 0 scrolled away most recently, index `len() - 1` is
/// the oldest still kept. vterm pushes at the new end and pops from it, and
/// eviction happens at the old end, so both are ends of a deque rather than
/// a shift of the whole array — which is what the C did, once per pushed
/// row.
///
/// A capacity of zero means the scrollback has not been sized yet: that
/// waits until there is a buffer to read `'scrollback'` from.
#[derive(Default)]
pub struct Scrollback {
    rows: VecDeque<Box<[VTermScreenCell]>>,
    capacity: usize,
    /// Rows pushed but not yet mirrored into the buffer by the refresh.
    pending: ::core::ffi::c_int,
    /// Rows evicted since the terminal opened. Buffer line numbers are
    /// relative to this, so it only ever grows.
    deleted: usize,
}

impl Scrollback {
    pub fn is_sized(&self) -> bool {
        self.capacity > 0
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn pending(&self) -> ::core::ffi::c_int {
        self.pending
    }

    pub fn deleted(&self) -> usize {
        self.deleted
    }

    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity;
    }

    /// The row `index` places above the top of the screen, newest first.
    pub fn row(&self, index: usize) -> Option<&[VTermScreenCell]> {
        self.rows.get(index).map(|row| &**row)
    }

    /// Take `cells` as the row that just scrolled off the top.
    ///
    /// When the scrollback is full the oldest row makes way, and its
    /// allocation is reused if it is the right width — the common case,
    /// since a terminal that is not being resized pushes one width forever.
    pub fn push(&mut self, cells: &[VTermScreenCell]) {
        let mut reused = None;
        if self.rows.len() >= self.capacity
            && let Some(oldest) = self.rows.pop_back()
        {
            self.deleted += 1;
            if oldest.len() == cells.len() {
                reused = Some(oldest);
            }
        }
        let row = match reused {
            Some(mut row) => {
                row.copy_from_slice(cells);
                row
            }
            None => cells.to_vec().into_boxed_slice(),
        };
        self.rows.push_front(row);
        // Capped rather than counted: the refresh can only ever owe the
        // buffer as many rows as are actually kept.
        if self.pending < self.capacity as ::core::ffi::c_int {
            self.pending += 1;
        }
    }

    /// Give the most recently scrolled-off row back to the screen.
    ///
    /// `cells` is vterm's row buffer. A stored row narrower than that leaves
    /// the tail blank; a wider one is truncated.
    pub fn pop(
        &mut self,
        cells: &mut [VTermScreenCell],
        old_height: &mut ::core::ffi::c_int,
    ) -> bool {
        let Some(row) = self.rows.pop_front() else {
            return false;
        };
        // A row the refresh had not yet mirrored simply cancels out.
        // Anything else means the screen grew by a line.
        if self.pending > 0 {
            self.pending -= 1;
        } else {
            *old_height += 1;
        }
        let copied = row.len().min(cells.len());
        cells[..copied].copy_from_slice(&row[..copied]);
        for cell in &mut cells[copied..] {
            cell.schar = 0;
            cell.width = 1;
        }
        true
    }

    /// Forget every row kept, without giving up the sizing.
    pub fn clear(&mut self) {
        self.deleted += self.rows.len();
        self.rows.clear();
        self.pending = 0;
    }

    /// Drop the oldest row, for trimming down to a lowered `'scrollback'`.
    ///
    /// Unlike eviction by [`Self::push`] this does not bump `deleted`: the
    /// caller is deleting the buffer's line itself and moving the marks, so
    /// the line numbering does not shift.
    pub fn drop_oldest(&mut self) {
        self.rows.pop_back();
    }

    /// Record that the refresh has appended one owed row to the buffer.
    pub fn mark_mirrored(&mut self) {
        self.pending -= 1;
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TerminalPending {
    pub resize: bool,
    pub cursor: bool,
    /// Where writes to the child go while a `TermRequest` autocommand is
    /// running, so that a handler's reply lands after the request has been
    /// fully reported. Null the rest of the time, meaning "write through".
    /// Borrowed from the in-flight request, which outlives the borrow.
    pub send: *mut Vec<u8>,
    pub events: *mut MultiQueue,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TerminalCursor {
    pub row: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
    pub shape: ::core::ffi::c_int,
    pub visible: bool,
    pub blink: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TerminalOptions {
    pub data: *mut ::core::ffi::c_void,
    pub width: uint16_t,
    pub height: uint16_t,
    pub read_pause_cb: terminal_read_pause_cb,
    pub write_cb: terminal_write_cb,
    pub resize_cb: terminal_resize_cb,
    pub resume_cb: terminal_resume_cb,
    pub close_cb: terminal_close_cb,
    pub force_crlf: bool,
}
pub type terminal_close_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub type terminal_read_pause_cb =
    Option<unsafe extern "C" fn(bool, *mut ::core::ffi::c_void) -> ()>;
pub type terminal_resize_cb =
    Option<unsafe extern "C" fn(uint16_t, uint16_t, *mut ::core::ffi::c_void) -> ()>;
pub type terminal_resume_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub type terminal_write_cb = Option<
    unsafe extern "C" fn(*const ::core::ffi::c_char, size_t, *mut ::core::ffi::c_void) -> (),
>;
