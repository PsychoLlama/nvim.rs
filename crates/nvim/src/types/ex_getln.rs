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
/// so `text.len()` is exactly C's `cmdbufflen`, and a command line opened
/// for `n` bytes grows when C's did.
#[derive(Clone, Default)]
pub(crate) struct CmdBuff {
    /// The allocation. Empty exactly when no command line is in use.
    text: Vec<u8>,
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
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.text[..self.len]
    }

    /// The byte at `i`, reading the terminator -- and anything past it --
    /// as the NUL C's walks stopped at.
    pub(crate) fn byte_at(&self, i: usize) -> u8 {
        self.bytes().get(i).copied().unwrap_or(0)
    }

    /// The byte at `i` of the allocation, the terminator and the slack past
    /// it included: what C read through `cmdbuff + i` when `i` may be past
    /// the text.  0 past the allocation, and with no command line in use.
    pub(crate) fn raw_byte_at(&self, i: usize) -> u8 {
        self.text.get(i).copied().unwrap_or(0)
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

    /// Write `with` over the start of the buffer and make it the whole text:
    /// C's `STRCPY(cmdbuff, ..)` with the `cmdlen` that followed.
    /// [`CmdBuff::open`] or [`CmdBuff::reserve`] must have made the room.
    pub(crate) fn write_text(&mut self, with: &[u8]) {
        assert!(
            with.len() < self.text.len(),
            "command line longer than its buffer"
        );
        self.text[..with.len()].copy_from_slice(with);
        self.len = with.len();
        self.text[with.len()] = 0;
    }

    /// Fill the start of the buffer with `n` copies of `byte` and make that
    /// the text: the indent `:insert` starts its lines with.
    pub(crate) fn fill(&mut self, n: usize, byte: u8) {
        assert!(n < self.text.len(), "command line longer than its buffer");
        self.text[..n].fill(byte);
        self.len = n;
        self.text[n] = 0;
    }

    /// Write one byte of the text in place, leaving the length alone: C's
    /// `cmdbuff[i] = c`.  A NUL written this way ends the line for every
    /// reader that stops at one, as it did in C.
    pub(crate) fn set_byte(&mut self, i: usize, byte: u8) {
        assert!(i < self.text.len(), "a byte of the command line");
        self.text[i] = byte;
    }

    /// Replace `text[from..to]` with `with`, moving the rest of the line --
    /// terminator included -- along. [`CmdBuff::reserve`] must have made the
    /// room.
    pub(crate) fn splice(&mut self, from: ::core::ffi::c_int, to: ::core::ffi::c_int, with: &[u8]) {
        let (from, to) = (CmdBuff::index(from), CmdBuff::index(to));
        assert!(from <= to && to <= self.len, "a span of the command line");
        let new_len = self.len - (to - from) + with.len();
        assert!(
            new_len < self.text.len(),
            "command line longer than its buffer"
        );
        self.text.copy_within(to..=self.len, from + with.len());
        self.text[from..from + with.len()].copy_from_slice(with);
        self.len = new_len;
    }

    /// Replace the text with `bytes`, opening a command line if none was in
    /// use.
    pub(crate) fn set(&mut self, bytes: &[u8]) {
        self.text.clear();
        self.text.extend_from_slice(bytes);
        self.text.push(0);
        self.len = bytes.len();
    }
}
/// A command line's colouring, cached against the line it was computed from.
///
/// [`CmdBuff`]'s shape, and for the same reason. C kept a `char *` copy of
/// the line the chunks describe and a kvec of the chunks, and released the
/// pair by hand at each exit that had to -- including a `theend:` label
/// reached from a dozen `goto`s. Both are `Vec`s here, so the only release
/// is the one `Drop` does, and re-caching a line cannot leak the last one.
#[derive(Clone, Default)]
pub(crate) struct ColoredCmdline {
    /// C's `prompt_id`: which command line the chunks were computed for.
    prompt_id: ::core::ffi::c_uint,
    /// The line they were computed from, without a terminator. Empty is C's
    /// NULL `cmdbuff`: nothing is cached.
    text: Vec<u8>,
    /// C's `colors`, in the order they were pushed.
    chunks: Vec<CmdlineColorChunk>,
}

impl ColoredCmdline {
    /// Nothing cached: C's `(ColoredCmdline){ .cmdbuff = NULL,
    /// .colors = KV_INITIAL_VALUE }`.
    pub(crate) const NONE: ColoredCmdline = ColoredCmdline {
        prompt_id: 0,
        text: Vec::new(),
        chunks: Vec::new(),
    };

    /// Whether the cache is the colouring of `text` under `prompt_id`.
    ///
    /// C spelled this a NULL check and a `strcmp`; an empty `text` is the
    /// NULL, and comparing the whole slice rather than up to a NUL can only
    /// turn a false hit into a miss.
    pub(crate) fn is_current(&self, prompt_id: ::core::ffi::c_uint, text: &[u8]) -> bool {
        self.prompt_id == prompt_id && !self.text.is_empty() && self.text == text
    }

    /// The chunks, in order.
    ///
    /// Taken afresh at each use rather than held: the drawing code this
    /// feeds re-enters the editor, which can replace the whole command line.
    pub(crate) fn chunks(&self) -> &[CmdlineColorChunk] {
        &self.chunks
    }

    /// Throw the chunks away, keeping the room they had: C's
    /// `colors.size = 0`.
    pub(crate) fn clear_chunks(&mut self) {
        self.chunks.clear();
    }

    /// Room for `n` chunks: C's `kv_resize`. The pushes that follow may
    /// still go past it.
    pub(crate) fn reserve_chunks(&mut self, n: usize) {
        self.chunks.reserve(n.saturating_sub(self.chunks.len()));
    }

    /// C's `kv_push`.
    pub(crate) fn push(&mut self, chunk: CmdlineColorChunk) {
        self.chunks.push(chunk);
    }

    /// Forget which line the chunks describe, so that the next colouring is
    /// computed rather than taken from here: C's `xfree(cmdbuff)` with the
    /// NULL that follows it.
    pub(crate) fn forget(&mut self) {
        self.text.clear();
    }

    /// Remember `text` as the line the chunks describe.
    pub(crate) fn remember(&mut self, prompt_id: ::core::ffi::c_uint, text: &[u8]) {
        self.prompt_id = prompt_id;
        self.text.clear();
        self.text.extend_from_slice(text);
    }
}
/// One command line: the one being edited (`ccline`), or one suspended under
/// it on the saved stack.
pub struct CmdlineInfo {
    pub(crate) cmdbuff: CmdBuff,
    pub cmdpos: ::core::ffi::c_int,
    pub cmdspos: ::core::ffi::c_int,
    pub cmdfirstc: ::core::ffi::c_int,
    pub cmdindent: ::core::ffi::c_int,
    /// The text shown before the line, a copy of the caller's: C's
    /// `cmdprompt`, which borrowed it.
    pub(crate) cmdprompt: Option<crate::memory::XString>,
    pub hl_id: ::core::ffi::c_int,
    pub overstrike: ::core::ffi::c_int,
    /// The completion in progress, owned by the command line while its key
    /// loop runs and `None` outside it: C's `xpc`, which pointed into the
    /// loop's stack frame. The loop moves it out for the length of each call
    /// into the completion machinery (`Cc::with_xpc`), so user code that
    /// asks about it then finds an empty one rather than one held mutably.
    pub(crate) xpc: Option<Expand>,
    pub xp_context: ExpandContext,
    pub xp_arg: Option<crate::memory::XString>,
    pub input_fn: ::core::ffi::c_int,
    pub cmdbuff_replaced: bool,
    pub prompt_id: ::core::ffi::c_uint,
    pub highlight_callback: Callback,
    pub(crate) last_colors: ColoredCmdline,
    pub level: ::core::ffi::c_int,
    pub special_char: ::core::ffi::c_char,
    pub special_shift: bool,
    pub redraw_state: CmdRedraw,
    pub one_key: bool,
    /// Whether a click may answer the prompt (C's `mouse_used != NULL`, a
    /// number prompt), and whether one did (`*mouse_used = true`): the
    /// prompt's caller reads the answer back when it returns.
    pub(crate) mouse_prompt: bool,
    pub(crate) mouse_used: bool,
}
