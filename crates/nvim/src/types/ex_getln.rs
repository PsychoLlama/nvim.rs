#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

pub type CmdRedraw = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
pub struct CmdlineColorChunk {
    pub start: ::core::ffi::c_int,
    pub end: ::core::ffi::c_int,
    pub hl_id: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
pub struct CmdlineColors {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut CmdlineColorChunk,
}
pub type CmdlineInfo = cmdline_info;

/// The command line's text: an owned, NUL-terminated growable buffer.
///
/// C spreads this over three fields and a rule nobody can see: `cmdbuff` is
/// the allocation, `cmdbufflen` its size, `cmdlen` the length of the text in
/// it, `cmdbuff[cmdlen]` is always a NUL, and a NULL `cmdbuff` means no
/// command line is in use at all. All four live here instead, and the one
/// operation that used to move the allocation out from under every pointer
/// into it -- `realloc_cmdbuff` -- is [`CmdBuff::reserve`], which cannot be
/// called for one command line and applied to another.
///
/// The `Vec` is kept at its full allocated length rather than at the text's,
/// so `text.len()` is exactly C's `cmdbufflen`. That is what lets the editing
/// code keep C's order of work: open a gap by writing past `len`, then close
/// it with [`CmdBuff::set_len`]. Growing the `Vec` instead would fill those
/// bytes with NULs after the gap had been written.
#[derive(Clone, Default)]
pub(crate) struct CmdBuff {
    /// The allocation. Empty exactly when no command line is in use.
    text: Vec<::core::ffi::c_char>,
    /// C's `cmdlen`: `text[..len]` is the command line, `text[len]` a NUL.
    len: usize,
}

impl CmdBuff {
    /// No command line in use: C's NULL `cmdbuff`.
    pub(crate) const NONE: CmdBuff = CmdBuff {
        text: Vec::new(),
        len: 0,
    };

    /// C's `alloc_cmdbuff`: the size it rounds a request of `want` up to.
    fn alloc_size(want: ::core::ffi::c_int) -> usize {
        // Give some extra space to avoid having to allocate all the time.
        if want < 80 {
            100
        } else {
            CmdBuff::index(want) + 20
        }
    }

    /// A C `int` offset into the buffer.  A negative one is 0, as every
    /// `.offset(i)` in the C would have been out of bounds anyway.
    fn index(i: ::core::ffi::c_int) -> usize {
        usize::try_from(i).unwrap_or(0)
    }

    /// Whether a command line is in use: C's `cmdbuff != NULL`.
    pub(crate) fn in_use(&self) -> bool {
        !self.text.is_empty()
    }

    /// C's `cmdlen`.
    pub(crate) fn len(&self) -> ::core::ffi::c_int {
        // C carried `cmdlen` as an `int` and every consumer still does; a
        // command line that long cannot be typed, pasted or completed.
        ::core::ffi::c_int::try_from(self.len).expect("command line over INT_MAX")
    }

    /// The text, without its terminator.
    pub(crate) fn bytes(&self) -> &[::core::ffi::c_char] {
        &self.text[..self.len]
    }

    /// The allocation, terminator and slack included: what a `cmdbuff`
    /// pointer used to reach.  NULL when no command line is in use, because
    /// that is the distinction every caller tests.
    pub(crate) fn as_mut_ptr(&mut self) -> *mut ::core::ffi::c_char {
        if self.text.is_empty() {
            ::core::ptr::null_mut()
        } else {
            self.text.as_mut_ptr()
        }
    }

    /// Open an empty command line with room for `want` bytes: C's
    /// `alloc_cmdbuff` plus the `cmdlen = 0; *cmdbuff = NUL` every caller
    /// wrote after it.
    pub(crate) fn open(&mut self, want: ::core::ffi::c_int) {
        self.text = vec![0; CmdBuff::alloc_size(want)];
        self.len = 0;
    }

    /// C's `dealloc_cmdbuff`: no command line in use.
    pub(crate) fn close(&mut self) {
        *self = CmdBuff::NONE;
    }

    /// C's `realloc_cmdbuff`: make room for `want` bytes, text and terminator
    /// included, without changing the text.
    ///
    /// Nothing moves out from under a pointer that was not already about to:
    /// the `Vec` reallocates exactly when C's `alloc_cmdbuff` did.
    pub(crate) fn reserve(&mut self, want: ::core::ffi::c_int) {
        if CmdBuff::index(want) < self.text.len() {
            return;
        }
        self.text.resize(CmdBuff::alloc_size(want), 0);
    }

    /// Set C's `cmdlen`, writing the terminator that goes with it.
    ///
    /// The bytes below `n` are whatever is there — the caller has usually
    /// just memmoved them into place, which is why this does not fill them.
    /// [`CmdBuff::reserve`] must have made the room.
    pub(crate) fn set_len(&mut self, n: ::core::ffi::c_int) {
        let n = CmdBuff::index(n);
        assert!(n < self.text.len(), "command line longer than its buffer");
        self.len = n;
        self.text[n] = 0;
    }

    /// Replace the text with `bytes`, opening a command line if none was in
    /// use.
    pub(crate) fn set(&mut self, bytes: &[::core::ffi::c_char]) {
        self.text.clear();
        self.text.extend_from_slice(bytes);
        self.text.push(0);
        self.len = bytes.len();
    }
}
#[derive(Copy, Clone)]
pub struct ColoredCmdline {
    pub prompt_id: ::core::ffi::c_uint,
    pub cmdbuff: *mut ::core::ffi::c_char,
    pub colors: CmdlineColors,
}
#[derive(Clone)]
pub struct cmdline_info {
    pub(crate) cmdbuff: CmdBuff,
    pub cmdpos: ::core::ffi::c_int,
    pub cmdspos: ::core::ffi::c_int,
    pub cmdfirstc: ::core::ffi::c_int,
    pub cmdindent: ::core::ffi::c_int,
    pub cmdprompt: *mut ::core::ffi::c_char,
    pub hl_id: ::core::ffi::c_int,
    pub overstrike: ::core::ffi::c_int,
    pub xpc: *mut expand_T,
    pub xp_context: ExpandContext,
    pub xp_arg: *mut ::core::ffi::c_char,
    pub input_fn: ::core::ffi::c_int,
    pub cmdbuff_replaced: bool,
    pub prompt_id: ::core::ffi::c_uint,
    pub highlight_callback: Callback,
    pub last_colors: ColoredCmdline,
    pub level: ::core::ffi::c_int,
    pub special_char: ::core::ffi::c_char,
    pub special_shift: bool,
    pub redraw_state: CmdRedraw,
    pub one_key: bool,
    pub mouse_used: *mut bool,
}
