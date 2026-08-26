//! The 'showcmd' area: the partial command echoed while it is still being
//! typed, and the size of the Visual selection while there is one.
//!
//! `add_to_showcmd` runs once per keystroke, so everything here follows the
//! per-key rules: no iterator adaptors on a path a character takes. The text
//! itself is a [`ShowCmd`] value -- the C `[char; SHOWCMD_BUFLEN]` with its
//! length beside it -- which the cell hands out by copy, so no caller holds
//! a borrow of it across the redraw the display functions run.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::Win;
use core::ptr;

use crate::api::private::helpers::cstr_as_string;
use crate::charset::CHAR_DISPLAY_LEN;
use crate::charset::{transchar, vim_isprintc};
use crate::cstr;
use crate::cursor::get_cursor_pos_ptr;
use crate::drawscreen::setcursor;
use crate::fold::has_folding;
use crate::getchar::char_avail;
use crate::global_cell::GlobalCell;
use crate::grid::{grid_line_flush, grid_line_puts, grid_line_start};
use crate::main::{
    Rows, curwin, ex_normal_busy, hl_attr_active, msg_silent, p_ch, p_sbr, p_sc, p_sel, p_sloc,
    redraw_tabline, sc_col,
};
use crate::mbyte::{utf_char2bytes, utfc_ptr2len};
use crate::memline::ml_get_pos;
use crate::message::{msg_grid_validate, msg_grid_view};
use crate::normal::{
    ARRAY_DICT_INIT, SHOWCMD_BUFLEN, SHOWCMD_COLS, VisualSelection, showcmd_is_clear,
    showcmd_visual, visual_selection,
};
use crate::optionstr::empty_option;
use crate::plines::getvcols;
use crate::pos::lt;
use crate::statusline::{draw_tabline, win_redr_status};
use crate::types::{
    Array, Integer, NUL, Object, OptInt, colnr_T, kObjectTypeArray, kObjectTypeInteger,
    kObjectTypeNil, kObjectTypeString, linenr_T, object,
};
use crate::ui::{ui_call_msg_showcmd, ui_has};
use ::libc::strcpy;
use core::ffi::{CStr, c_char, c_int};

use crate::highlight_group::HLF_MSG;
use crate::keycodes::{
    KE_EVENT, KE_IGNORE, KE_LEFTDRAG, KE_LEFTMOUSE, KE_LEFTRELEASE, KE_MIDDLEDRAG, KE_MIDDLEMOUSE,
    KE_MIDDLERELEASE, KE_MOUSEDOWN, KE_MOUSELEFT, KE_MOUSEMOVE, KE_MOUSERIGHT, KE_MOUSEUP,
    KE_RIGHTDRAG, KE_RIGHTMOUSE, KE_RIGHTRELEASE, KE_X1DRAG, KE_X1MOUSE, KE_X1RELEASE, KE_X2DRAG,
    KE_X2MOUSE, KE_X2RELEASE,
};
use crate::types::object_data;
use crate::types::ui::kUIMessages;

/// The 'showcmd' text: at most `SHOWCMD_BUFLEN - 1` bytes and the
/// terminator C consumers still expect, carried by value.
///
/// The C buffer was a shared global with no length, so every reader had to
/// walk it for one and any writer reached from a redraw could move the text
/// under a reader. Owning it as a `Copy` value retires both.
#[derive(Clone, Copy)]
pub(crate) struct ShowCmd {
    /// `len` bytes of text, then a NUL, then whatever the last longer text
    /// left behind.
    chars: [c_char; SHOWCMD_BUFLEN as usize],
    len: usize,
}

impl ShowCmd {
    /// The longest text that fits, terminator excluded.
    const CAP: usize = SHOWCMD_BUFLEN as usize - 1;

    const fn new() -> Self {
        Self {
            chars: [0; SHOWCMD_BUFLEN as usize],
            len: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The text as the NUL-terminated string a C consumer takes.
    pub(crate) fn as_cstr(&self) -> &CStr {
        cstr::in_chars(&self.chars)
    }

    /// The whole array, for a caller that wants a fixed number of cells and
    /// lets the terminator stop it early.
    pub(crate) fn cells(&self, n: usize) -> &[c_char] {
        &self.chars[..n.min(SHOWCMD_BUFLEN as usize)]
    }

    /// Drop everything past `at` bytes.
    fn truncate(&mut self, at: usize) {
        if at < self.len {
            self.len = at;
            self.chars[at] = NUL as c_char;
        }
    }

    /// Replace the text with `text`, cut at the tail to what fits -- which
    /// is what the `snprintf` upstream formats it with does.
    fn set_text(&mut self, text: &[u8]) {
        self.len = text.len().min(Self::CAP);
        for (at, &b) in text[..self.len].iter().enumerate() {
            self.chars[at] = b as c_char;
        }
        self.chars[self.len] = NUL as c_char;
    }

    /// Append `text`, shifting the oldest bytes out rather than refusing to
    /// grow once the total passes `limit`.
    fn append(&mut self, text: &[u8], limit: usize) {
        let limit = limit.min(Self::CAP);
        let text = &text[text.len().saturating_sub(limit)..];
        let overflow = (self.len + text.len()).saturating_sub(limit);
        self.chars.copy_within(overflow..self.len, 0);
        self.len -= overflow;
        for &b in text {
            self.chars[self.len] = b as c_char;
            self.len += 1;
        }
        self.chars[self.len] = NUL as c_char;
    }
}

/// The 'showcmd' area's text.
pub(crate) static showcmd_buf: GlobalCell<ShowCmd> = GlobalCell::new(ShowCmd::new());

/// What [`push_showcmd`] parked, for [`pop_showcmd`] to put back.
static old_showcmd_buf: GlobalCell<ShowCmd> = GlobalCell::new(ShowCmd::new());

/// How many bytes of command may be shown. The message UI gets the whole
/// buffer; the last screen line gets the ten columns reserved for it.
#[inline(always)]
fn showcmd_limit() -> usize {
    if ui_has(kUIMessages) {
        SHOWCMD_BUFLEN as usize - 1
    } else {
        SHOWCMD_COLS as usize
    }
}

/// Keys that never appear in the 'showcmd' area: the mouse events and the
/// two internal no-ops. Upstream spells this as a NUL-terminated array and
/// walks it; the terminator is not a member, so it is not here either.
const IGNORED: [c_int; 22] = [
    -(253 + ((KE_IGNORE as c_int) << 8)),
    -(253 + ((KE_LEFTMOUSE as c_int) << 8)),
    -(253 + ((KE_LEFTDRAG as c_int) << 8)),
    -(253 + ((KE_LEFTRELEASE as c_int) << 8)),
    -(253 + ((KE_MOUSEMOVE as c_int) << 8)),
    -(253 + ((KE_MIDDLEMOUSE as c_int) << 8)),
    -(253 + ((KE_MIDDLEDRAG as c_int) << 8)),
    -(253 + ((KE_MIDDLERELEASE as c_int) << 8)),
    -(253 + ((KE_RIGHTMOUSE as c_int) << 8)),
    -(253 + ((KE_RIGHTDRAG as c_int) << 8)),
    -(253 + ((KE_RIGHTRELEASE as c_int) << 8)),
    -(253 + ((KE_MOUSEDOWN as c_int) << 8)),
    -(253 + ((KE_MOUSEUP as c_int) << 8)),
    -(253 + ((KE_MOUSELEFT as c_int) << 8)),
    -(253 + ((KE_MOUSERIGHT as c_int) << 8)),
    -(253 + ((KE_X1MOUSE as c_int) << 8)),
    -(253 + ((KE_X1DRAG as c_int) << 8)),
    -(253 + ((KE_X1RELEASE as c_int) << 8)),
    -(253 + ((KE_X2MOUSE as c_int) << 8)),
    -(253 + ((KE_X2DRAG as c_int) << 8)),
    -(253 + ((KE_X2RELEASE as c_int) << 8)),
    -(253 + ((KE_EVENT as c_int) << 8)),
];

/// The two lines the Visual selection spans, with any fold at either end
/// opened out to its whole range.
fn visual_line_range(sel: VisualSelection, cursor_bot: bool) -> (linenr_T, linenr_T) {
    // SAFETY (throughout): `curwin` is the current window.
    let (mut top, mut bot) = if cursor_bot {
        (sel.anchor.lnum, cur_win().w_cursor.lnum)
    } else {
        (cur_win().w_cursor.lnum, sel.anchor.lnum)
    };
    unsafe { has_folding(curwin.get(), top, &raw mut top, ptr::null_mut()) };
    unsafe { has_folding(curwin.get(), bot, ptr::null_mut(), &raw mut bot) };
    (top, bot)
}

/// The width of a blockwise selection, measured with 'showbreak' suppressed
/// so a wrapped line does not add the leader to the column count.
fn blockwise_width(sel: VisualSelection) -> c_int {
    let (mut leftcol, mut rightcol): (colnr_T, colnr_T) = (0, 0);
    // A copy of the anchor: `getvcols` only reads it.
    let mut anchor = sel.anchor;
    // SAFETY: both positions are in the current buffer, and the two
    // 'showbreak' values are put back before returning.
    let saved_sbr = p_sbr.get();
    let saved_w_sbr = cur_win().w_onebuf_opt.wo_sbr;
    p_sbr.set(empty_option());
    cur_win().w_onebuf_opt.wo_sbr = empty_option();
    let win = cur_win();
    let (cursor, other) = (win.cursor().raw(), &raw mut anchor);
    let (l, r) = (&raw mut leftcol, &raw mut rightcol);
    unsafe { getvcols(win.raw(), cursor, other, l, r) };
    p_sbr.set(saved_sbr);
    cur_win().w_onebuf_opt.wo_sbr = saved_w_sbr;
    rightcol - leftcol + 1
}

/// Characters and bytes between the two ends of a charwise selection that
/// lies within one line.
///
/// 'selection' decides whether the far end is included. A character the
/// decoder rejects counts as one byte and one character and ends the walk,
/// which is what stops an invalid byte looping.
fn charwise_extent(sel: VisualSelection, cursor_bot: bool) -> (c_int, c_int) {
    let (mut bytes, mut chars) = (0, 0);
    let anchor = sel.anchor;
    // SAFETY: both pointers are into the current line, and the walk stops at
    // or before `e`.
    let (mut s, e) = if cursor_bot {
        (
            unsafe { ml_get_pos(&raw const anchor) },
            get_cursor_pos_ptr(),
        )
    } else {
        (get_cursor_pos_ptr(), unsafe {
            ml_get_pos(&raw const anchor)
        })
    };
    let exclusive = unsafe { *p_sel.get() } as c_int == 'e' as c_int;
    while if exclusive { s < e } else { s <= e } {
        let l = unsafe { utfc_ptr2len(s) };
        if l == 0 {
            bytes += 1;
            chars += 1;
            break;
        }
        bytes += l;
        chars += 1;
        s = unsafe { s.offset(l as isize) };
    }
    (chars, bytes)
}

/// Describe the Visual selection into the 'showcmd' buffer.
fn show_visual_size(sel: VisualSelection) {
    // SAFETY: `curwin` is the current window.
    let cursor_bot = lt(sel.anchor, cur_win().w_cursor);
    let (top, bot) = visual_line_range(sel, cursor_bot);
    let lines = (bot - top + 1) as c_int;

    // SAFETY: `curwin` is the current window.
    let same_line = sel.anchor.lnum == cur_win().w_cursor.lnum;
    let text = if sel.mode.is_block() {
        format!("{lines}x{}", blockwise_width(sel))
    } else if sel.mode.is_line() || !same_line {
        format!("{lines}")
    } else {
        let (chars, bytes) = charwise_extent(sel, cursor_bot);
        if bytes == chars {
            format!("{chars}")
        } else {
            format!("{chars}-{bytes}")
        }
    };

    let mut sc = showcmd_buf.get();
    sc.set_text(text.as_bytes());
    sc.truncate(showcmd_limit());
    showcmd_buf.set(sc);
    showcmd_visual.set(true);
}

/// Throw away the partial command, or replace it with the size of the
/// Visual selection while there is one and nothing is waiting to be typed.
pub(crate) fn clear_showcmd() {
    if p_sc.get() == 0 {
        return;
    }
    // SAFETY: reads the typeahead state.
    if let Some(sel) = visual_selection().filter(|_| !unsafe { char_avail() }) {
        show_visual_size(sel);
    } else {
        showcmd_buf.set(ShowCmd::new());
        showcmd_visual.set(false);
        if showcmd_is_clear.get() {
            return;
        }
    }
    display_showcmd();
}

/// Append one typed key to the 'showcmd' area.
///
/// Answers whether the area was redrawn: the caller uses that to decide
/// whether it must put the cursor back.
pub(crate) fn add_to_showcmd(c: c_int) -> bool {
    if p_sc.get() == 0 || msg_silent.get() != 0 || ex_normal_busy.get() != 0 {
        return false;
    }
    // A Visual size sitting in the area is replaced, not appended to.
    if showcmd_visual.get() {
        showcmd_buf.set(ShowCmd::new());
        showcmd_visual.set(false);
    }
    if c < 0 && IGNORED.contains(&c) {
        return false;
    }

    let mut mbyte_buf: [c_char; 7] = [0; 7];
    let mut display = [0 as c_char; CHAR_DISPLAY_LEN];
    // SAFETY: `transchar` answers a NUL-terminated rendering; the multibyte
    // branch writes at most MB_MAXBYTES + 1 into `mbyte_buf`, and both
    // outlive the borrow.
    // SAFETY: `c` is a plain character value.
    let extra = if c <= 0x7f || !unsafe { vim_isprintc(c) } {
        display = unsafe { transchar(c) };
        // A space is shown as its byte value, so it is not lost in the
        // padding the area is drawn with.
        if display[0] as c_int == ' ' as c_int {
            unsafe { strcpy(display.as_mut_ptr(), c"<20>".as_ptr().cast_mut()) };
        }
        unsafe { CStr::from_ptr(display.as_ptr()) }
    } else {
        let n = unsafe { utf_char2bytes(c, mbyte_buf.as_mut_ptr()) };
        mbyte_buf[n as usize] = NUL as c_char;
        unsafe { CStr::from_ptr(mbyte_buf.as_ptr()) }
    };

    let mut sc = showcmd_buf.get();
    sc.append(extra.to_bytes(), showcmd_limit());
    showcmd_buf.set(sc);

    // SAFETY: reads the typeahead state.
    if unsafe { char_avail() } {
        return false;
    }
    display_showcmd();
    true
}

/// `add_to_showcmd`, for a caller that always wants the cursor put back.
pub(crate) fn add_to_showcmd_c(c: c_int) {
    add_to_showcmd(c);
    // SAFETY: moves the terminal cursor to the current window's.
    unsafe { setcursor() };
}

/// Drop the last `len` bytes of the partial command.
pub(crate) fn del_from_showcmd(len: c_int) {
    if p_sc.get() == 0 {
        return;
    }
    let mut sc = showcmd_buf.get();
    sc.truncate(sc.len.saturating_sub(len as usize));
    showcmd_buf.set(sc);
    // SAFETY: reads the typeahead state.
    if !unsafe { char_avail() } {
        display_showcmd();
    }
}

/// Save the partial command across something that shows its own.
pub(crate) fn push_showcmd() {
    if p_sc.get() != 0 {
        old_showcmd_buf.set(showcmd_buf.get());
    }
}

/// Put back what `push_showcmd` saved.
pub(crate) fn pop_showcmd() {
    if p_sc.get() == 0 {
        return;
    }
    showcmd_buf.set(old_showcmd_buf.get());
    display_showcmd();
}

/// Hand the area's current contents to whichever of the four places
/// 'showcmdloc' names is showing it.
pub(crate) fn display_showcmd() {
    let clear = showcmd_buf.get().is_empty();
    showcmd_is_clear.set(clear);

    // SAFETY: 'showcmdloc' is a non-empty string option.
    let loc = unsafe { *p_sloc.get() as c_int };
    if loc == 's' as c_int {
        // SAFETY: `curwin` is the current window.
        if clear {
            cur_win().w_redr_status = true;
        } else {
            unsafe { win_redr_status(curwin.get()) };
            unsafe { setcursor() };
        }
        return;
    }
    if loc == 't' as c_int {
        if clear {
            redraw_tabline.set(true);
        } else {
            // SAFETY: redraws the tab line and puts the cursor back.
            unsafe { draw_tabline() };
            unsafe { setcursor() };
        }
        return;
    }

    if ui_has(kUIMessages) {
        show_through_ui(clear);
        return;
    }
    if p_ch.get() == 0 as OptInt {
        return;
    }
    draw_on_last_line(clear);
}

/// The message-UI form: one chunk of `[attr, text, hl_id]` inside a
/// one-element content array.
///
/// Both arrays borrow stack storage for the length of the call, which is
/// what upstream does too -- `ui_call_msg_showcmd` copies what it keeps.
fn show_through_ui(clear: bool) {
    // The text outlives the call, which is all `cstr_as_string`'s borrow
    // needs: `ui_call_msg_showcmd` copies what it keeps.
    let sc = showcmd_buf.get();
    let text = sc.as_cstr();
    let mut chunk_items: [Object; 3] = [NIL; 3];
    let mut content_items: [Object; 1] = [NIL; 1];
    let mut chunk: Array = ARRAY_DICT_INIT;
    chunk.capacity = 3;
    chunk.items = chunk_items.as_mut_ptr();
    let mut content: Array = ARRAY_DICT_INIT;
    content.capacity = 1;
    content.items = content_items.as_mut_ptr();

    if !clear {
        // SAFETY: `chunk` has capacity 3 and `content` capacity 1, and the
        // three writes below are the only ones.
        // SAFETY: `text` is NUL-terminated.
        let string = unsafe { cstr_as_string(text.as_ptr()) };
        let shown = object {
            type_0: kObjectTypeString,
            data: object_data { string },
        };
        unsafe { *chunk.items.add(0) = integer_object(0) };
        unsafe { *chunk.items.add(1) = shown };
        unsafe { *chunk.items.add(2) = integer_object(0) };
        chunk.size = 3;
        let line = object {
            type_0: kObjectTypeArray,
            data: object_data { array: chunk },
        };
        unsafe { *content.items.add(0) = line };
        content.size = 1;
    }
    ui_call_msg_showcmd(content);
}

/// A nil `Object`, which is what both arrays start out full of.
const NIL: Object = Object {
    type_0: kObjectTypeNil,
    data: object_data { boolean: false },
};

fn integer_object(n: Integer) -> Object {
    object {
        type_0: kObjectTypeInteger,
        data: object_data { integer: n },
    }
}

/// The built-in form: write the text into the reserved columns of the last
/// screen line, then blank the rest of them.
fn draw_on_last_line(clear: bool) {
    let sc = showcmd_buf.get();
    // SAFETY: the message view is the message grid and `showcmd_row` is its
    // last row; `grid_line_puts` bounds itself to the line it was started on.
    unsafe { msg_grid_validate() };
    let showcmd_row = Rows.get() - 1;
    unsafe { grid_line_start(msg_grid_view(), showcmd_row) };
    let attr = unsafe { *hl_attr_active.get().offset(HLF_MSG as isize) };
    let mut len = 0;
    if !clear {
        len = unsafe { grid_line_puts(sc_col.get(), sc.as_cstr().as_ptr(), -1, attr) };
    }
    // The padding is the same ten spaces every time; only the tail past
    // what was written is drawn.
    let pad = c"          ".as_ptr().cast_mut();
    // SAFETY: `pad` is ten spaces and a terminator, and `len` is at most ten.
    let tail = unsafe { pad.offset(len as isize) };
    unsafe { grid_line_puts(sc_col.get() + len, tail, -1, attr) };
    unsafe { grid_line_flush() };
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
