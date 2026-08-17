#![forbid(unsafe_code)]

//! The terminal user interface's state.
//!
//! One [`TUIData`] exists per UI process, holding everything the TUI knows
//! about the terminal it drives, with the input layer's [`TermInput`] inside
//! it. The lifecycle that fills these in lives in
//! [`tui`](crate::tui::tui); what is here is the shape, and the
//! few invariants that can be kept by construction rather than by
//! convention -- see [`Staging`].

use super::*;
use core::ffi::{CStr, c_char, c_int};

pub type TermMode = ::core::ffi::c_uint;
pub type TermModeState = ::core::ffi::c_uint;

/// The staging buffer's size. A flush is one `uv_write`, so this also caps
/// how much can be written without a syscall.
pub const BUF_SIZE: usize = 65535;

/// How many named keys can wait for the next `nvim_input`, and the size of
/// the buffer they wait in.
pub const KEY_BUFFER_SIZE: usize = 0x1000;

/// Everything the TUI knows about the terminal it is driving.
///
/// One of these exists per UI process. It is built by [`tui_start`](crate::tui::tui::tui_start) and
/// lives as long as the process does: libuv handles and the input layer
/// hold pointers into it, so it can neither move nor be freed while the
/// loop it registered with is still running.
///
/// Holding a `&mut TUIData` means holding a TUI that `tui_start` finished
/// setting up: the write handles are open and `ti` describes the terminal
/// on the other end. The modules that only paint (see
/// [`output`](crate::tui::output),
/// [`paint`](crate::tui::paint)) are safe on the
/// strength of that, rather than re-asserting it at every call. The half of
/// that which can be enforced rather than promised -- that what is staged
/// really is within the buffer -- is [`Staging`]'s doing.
pub struct TUIData {
    /// The editor's event loop, borrowed. Not the loop writes go out on.
    pub loop_0: *mut Loop,
    /// Bytes on their way to the terminal.
    pub staging: Staging,
    pub input: TermInput,
    /// The loop writes run on, private to the TUI so a flush can be
    /// synchronous without pumping the editor's loop.
    pub write_loop: uv_loop_t,
    /// This terminal's capabilities.
    pub ti: TerminfoEntry,
    /// `$TERM`, or the name of the built-in entry standing in for it.
    pub term: *mut c_char,
    pub output_handle: OutputHandle,
    pub out_isatty: bool,
    pub winch_handle: SignalWatcher,
    /// Fires once, shortly after startup, for the sequences that must not
    /// be sent until the terminal has settled.
    pub startup_delay_timer: uv_timer_t,
    /// What the TUI believes is on the screen.
    pub grid: UGrid,
    /// Rectangles waiting to be repainted at the next flush.
    pub invalid_regions: Vec<Rect>,
    /// Where the editor last asked for the cursor, as opposed to where the
    /// terminal's cursor actually is (`grid.row`/`grid.col`).
    pub row: c_int,
    pub col: c_int,
    pub out_fd: c_int,
    /// Resizes the terminal made itself, whose echo is not to be acted on.
    pub pending_resize_events: c_int,
    /// Did `$TERM` resolve against the terminfo database, or fall back?
    pub terminfo_found_in_db: bool,
    pub can_change_scroll_region: bool,
    pub has_left_and_right_margin_mode: bool,
    pub has_sync_mode: bool,
    pub can_set_lr_margin: bool,
    pub can_scroll: bool,
    pub can_erase_chars: bool,
    /// Does the terminal leave the final column immediately, rather than on
    /// the next character?
    pub immediate_wrap_after_last_column: bool,
    /// "Background colour erase": erasing paints the current background.
    pub bce: bool,
    pub mouse_enabled: bool,
    pub mouse_move_enabled: bool,
    /// Mouse state to restore after a suspend.
    pub mouse_enabled_save: bool,
    pub title_enabled: bool,
    pub sync_output: bool,
    pub busy: bool,
    /// Is the terminal's cursor hidden right now?
    pub is_invisible: bool,
    /// Does the editor want it hidden?
    pub want_invisible: bool,
    pub set_cursor_color_as_str: bool,
    pub cursor_has_color: bool,
    /// Between the TUI starting and the first mode change.
    pub is_starting: bool,
    pub resize_events_enabled: bool,
    /// Terminal modes this TUI turned on and owes the terminal a reset for.
    pub modes: TermModes,
    /// Where a screenshot is being written instead of to the terminal.
    pub screenshot: *mut FILE,
    pub cursor_shapes: [cursorentry_T; 18],
    /// The colours unset colours fall back to.
    pub clear_attrs: HlAttrs,
    /// Every highlight the editor has defined, indexed by id. Ids the
    /// editor has not defined read as the default highlight.
    pub attrs: Vec<HlAttrs>,
    /// The attribute id the terminal is currently dressed for, or -1.
    pub print_attr_id: c_int,
    /// Is the terminal back at its own default attributes?
    pub default_attr: bool,
    pub set_default_colors: bool,
    /// Would erasing paint the right thing in the current attributes?
    pub can_clear_attr: bool,
    pub showing_mode: usize,
    pub verbose: Integer,
    pub terminfo_ext: TerminfoExt,
    pub can_set_title: bool,
    pub can_set_underline_color: bool,
    pub can_resize_screen: bool,
    pub stopped: bool,
    /// The size the editor has been told about.
    pub width: c_int,
    pub height: c_int,
    /// `'termguicolors'`.
    pub rgb: bool,
    pub screen_or_tmux: bool,
    /// The hyperlink the cells being painted are inside, or -1.
    pub url: c_int,
    pub ti_arena: Arena,
}

/// The bytes on their way to the terminal.
///
/// The fields are private, and that is the point: everything outside knows
/// only that what is staged is staged and that there is room for what it is
/// about to add. [`output`](crate::tui::output) writes what is
/// here straight to
/// the terminal without bounds-checking it first, which is sound because
/// nothing can put more in than fits.
pub struct Staging {
    /// Boxed: 64 KiB has no business on the stack of whoever builds a TUI.
    buf: Box<[u8; BUF_SIZE]>,
    len: usize,
}

impl Staging {
    pub fn new() -> Self {
        // A `[0; BUF_SIZE]` literal would be built on the stack and then
        // copied into the box, which is 64 KiB of pointless memcpy.
        let buf: Box<[u8; BUF_SIZE]> = vec![0; BUF_SIZE]
            .into_boxed_slice()
            .try_into()
            .expect("a boxed slice of exactly BUF_SIZE bytes");
        Self { buf, len: 0 }
    }

    /// How much more will fit before a flush is needed.
    pub fn room(&self) -> usize {
        BUF_SIZE - self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Stage `bytes`. The caller flushes first if they do not fit --
    /// dropping them silently would put half a sequence on the wire.
    pub fn push(&mut self, bytes: &[u8]) {
        let end = self.len + bytes.len();
        assert!(end <= BUF_SIZE, "staged past the end of the buffer");
        self.buf[self.len..end].copy_from_slice(bytes);
        self.len = end;
    }

    /// Stage up to `count` copies of `byte`, returning how many fit.
    pub fn fill(&mut self, byte: u8, count: usize) -> usize {
        let taken = count.min(self.room());
        let end = self.len + taken;
        self.buf[self.len..end].fill(byte);
        self.len = end;
        taken
    }

    /// The room left, for the one writer that fills it in place: terminfo
    /// expands a capability straight into the buffer. What it wrote is
    /// staged by [`Self::commit`].
    pub fn spare(&mut self) -> &mut [u8] {
        &mut self.buf[self.len..]
    }

    /// Stage the `used` bytes [`Self::spare`] was just filled with.
    pub fn commit(&mut self, used: usize) {
        assert!(
            used <= self.room(),
            "committed more than there was room for"
        );
        self.len += used;
    }

    /// What a flush writes, as libuv wants it. The pointer is this buffer's
    /// own and stays good until the next call that stages anything.
    pub fn staged(&mut self) -> (*mut c_char, usize) {
        (self.buf.as_mut_ptr().cast::<c_char>(), self.len)
    }

    /// Everything staged has been written.
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

/// A rectangle of the screen, in half-open rows and columns.
#[derive(Copy, Clone)]
pub struct Rect {
    pub top: c_int,
    pub bot: c_int,
    pub left: c_int,
    pub right: c_int,
}

/// Terminal modes the TUI turned on, packed so the whole set can be reset.
#[derive(Copy, Clone)]
pub struct TermModes {
    pub grapheme_clusters_theme_updates_resize_events: [u8; 1],
}
crate::bitfield_accessors! {
    impl TermModes.grapheme_clusters_theme_updates_resize_events {
        0..=0 => grapheme_clusters, set_grapheme_clusters: bool;
        1..=1 => theme_updates, set_theme_updates: bool;
        2..=2 => resize_events, set_resize_events: bool;
    }
}

/// Where output goes: a tty when there is one, a pipe otherwise.
#[derive(Copy, Clone)]
pub union OutputHandle {
    pub tty: uv_tty_t,
    pub pipe: uv_pipe_t,
}

/// Everything needed to read one terminal.
///
/// It lives inside the [`TUIData`] it belongs to, and holds that one's
/// pointer: what the terminal says about its modes and its size is the TUI's
/// business rather than the editor's.
pub struct TermInput {
    /// The terminal's file descriptor: this process's own stdin.
    pub in_fd: c_int,
    /// Which phase of a bracketed paste this is; see
    /// [`input`](crate::tui::input)'s `PASTE_*`.
    pub paste: i8,
    /// `'ttimeout'`: should an incomplete sequence time out at all?
    pub ttimeout: bool,
    pub callbacks: TermInputCallbacks,
    pub key_encoding: KeyEncoding,
    /// `'ttimeoutlen'`: how long to wait for the rest of a sequence.
    pub ttimeoutlen: OptInt,
    /// termkey's parser, holding whatever sequence is half-read.
    pub tk: *mut TermKey,
    /// How termkey asks the TUI for a capability, which is how nvim gets key
    /// sequences the terminal's own description does not name.
    pub tk_ti_hook_fn: Option<TermKey_Terminfo_Getstr_Hook>,
    /// Fires when an incomplete sequence has waited long enough.
    pub timer_handle: uv_timer_t,
    /// Rations the background-colour queries that a terminal announcing a
    /// theme change would otherwise provoke one of per announcement.
    pub bg_query_timer: uv_timer_t,
    pub loop_0: *mut Loop,
    pub read_stream: RStream,
    /// The TUI this belongs to.
    pub tui_data: *mut TUIData,
    /// Keys named but not yet sent. One `nvim_input` carries as many as have
    /// accumulated by the end of a read.
    pub key_buffer: [u8; KEY_BUFFER_SIZE],
    pub key_buffer_len: usize,
}

/// What the TUI wants to be told about.
#[derive(Copy, Clone)]
pub struct TermInputCallbacks {
    /// Called on the next reply to a device-attributes query, once. The TUI
    /// sends such a query after resetting the terminal, so the reply is what
    /// says the reset has been processed.
    pub primary_device_attr: Option<unsafe fn(*mut TUIData)>,
}

/// The extra capability strings nvim carries because terminfo has no slot
/// for them. All are fixed sequences chosen by terminal, never parameterised.
#[derive(Clone, Copy, Default)]
pub struct TerminfoExt {
    pub enable_focus_reporting: Option<&'static CStr>,
    pub disable_focus_reporting: Option<&'static CStr>,
    pub reset_scroll_region: Option<&'static CStr>,
    pub enter_altfont_mode: Option<&'static CStr>,
}
