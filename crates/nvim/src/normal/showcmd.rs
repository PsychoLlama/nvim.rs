//! The 'showcmd' area: the partial command echoed while it is still being
//! typed, and the size of the Visual selection while there is one.
//!
//! `add_to_showcmd` runs once per keystroke, so everything here follows the
//! per-key rules: no `GlobalCell::with`, no iterator adaptors on a path a
//! character takes, and the buffer stays the C `[c_char; SHOWCMD_BUFLEN]`
//! that `drawscreen.rs` and `getchar.rs` also write.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ptr;

use crate::api::private::helpers::cstr_as_string;
use crate::charset::{transchar, vim_isprintc};
use crate::cursor::get_cursor_pos_ptr;
use crate::drawscreen::setcursor;
use crate::fold::has_folding;
use crate::getchar::char_avail;
use crate::grid::{grid_line_flush, grid_line_puts, grid_line_start};
use crate::main::{
    Rows, VIsual, VIsual_active, VIsual_mode, curwin, ex_normal_busy, hl_attr_active, msg_grid_adj,
    msg_silent, p_ch, p_sbr, p_sc, p_sel, p_sloc, redraw_tabline, sc_col, showcmd_buf,
};
use crate::mbyte::{utf_char2bytes, utfc_ptr2len};
use crate::memline::ml_get_pos;
use crate::message::msg_grid_validate;
use crate::normal::{
    ARRAY_DICT_INIT, SHOWCMD_BUFLEN, SHOWCMD_COLS, old_showcmd_buf, showcmd_is_clear,
    showcmd_visual,
};
use crate::optionstr::empty_option;
use crate::os::cshim::{memmove, snprintf};
use crate::plines::getvcols;
use crate::pos::lt;
use crate::statusline::{draw_tabline, win_redr_status};
use crate::types::{
    Array, Integer, NUL, Object, OptInt, colnr_T, int64_t, kObjectTypeArray, kObjectTypeInteger,
    kObjectTypeNil, kObjectTypeString, linenr_T, object, size_t,
};
use crate::ui::{ui_call_msg_showcmd, ui_has};
use ::libc::{strcat, strcpy, strlen};
use core::ffi::{c_char, c_int, c_void};

use crate::highlight_group::HLF_MSG;
use crate::keycodes::{
    Ctrl_V, KE_EVENT, KE_IGNORE, KE_LEFTDRAG, KE_LEFTMOUSE, KE_LEFTRELEASE, KE_MIDDLEDRAG,
    KE_MIDDLEMOUSE, KE_MIDDLERELEASE, KE_MOUSEDOWN, KE_MOUSELEFT, KE_MOUSEMOVE, KE_MOUSERIGHT,
    KE_MOUSEUP, KE_RIGHTDRAG, KE_RIGHTMOUSE, KE_RIGHTRELEASE, KE_X1DRAG, KE_X1MOUSE, KE_X1RELEASE,
    KE_X2DRAG, KE_X2MOUSE, KE_X2RELEASE,
};
use crate::types::object_data;
use crate::types::ui::kUIMessages;

/// The buffer holds a NUL-terminated C string; this is its length.
#[inline(always)]
fn showcmd_len() -> usize {
    // SAFETY: `showcmd_buf` is a `[c_char; SHOWCMD_BUFLEN]` and every writer
    // in the tree keeps it NUL-terminated.
    unsafe { strlen(showcmd_buf.ptr().cast::<c_char>()) as usize }
}

/// Truncate the echoed text to `at` bytes.
#[inline(always)]
fn showcmd_truncate(at: usize) {
    // SAFETY: callers pass an index within the array.
    unsafe { (*showcmd_buf.ptr())[at] = NUL as c_char }
}

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
fn visual_line_range(cursor_bot: bool) -> (linenr_T, linenr_T) {
    // SAFETY: `curwin` is the current window and `VIsual` is only read while
    // `VIsual_active`, which the one caller has already tested.
    unsafe {
        let (mut top, mut bot) = if cursor_bot {
            (VIsual.get().lnum, (*curwin.get()).w_cursor.lnum)
        } else {
            ((*curwin.get()).w_cursor.lnum, VIsual.get().lnum)
        };
        has_folding(curwin.get(), top, &raw mut top, ptr::null_mut());
        has_folding(curwin.get(), bot, ptr::null_mut(), &raw mut bot);
        (top, bot)
    }
}

/// The width of a blockwise selection, measured with 'showbreak' suppressed
/// so a wrapped line does not add the leader to the column count.
fn blockwise_width() -> c_int {
    let (mut leftcol, mut rightcol): (colnr_T, colnr_T) = (0, 0);
    // A copy of the anchor: `getvcols` only reads it.
    let mut anchor = VIsual.get();
    // SAFETY: both positions are in the current buffer, and the two
    // 'showbreak' values are put back before returning.
    unsafe {
        let saved_sbr = p_sbr.get();
        let saved_w_sbr = (*curwin.get()).w_onebuf_opt.wo_sbr;
        p_sbr.set(empty_option());
        (*curwin.get()).w_onebuf_opt.wo_sbr = empty_option();
        getvcols(
            curwin.get(),
            &raw mut (*curwin.get()).w_cursor,
            &raw mut anchor,
            &raw mut leftcol,
            &raw mut rightcol,
        );
        p_sbr.set(saved_sbr);
        (*curwin.get()).w_onebuf_opt.wo_sbr = saved_w_sbr;
    }
    rightcol - leftcol + 1
}

/// Characters and bytes between the two ends of a charwise selection that
/// lies within one line.
///
/// 'selection' decides whether the far end is included. A character the
/// decoder rejects counts as one byte and one character and ends the walk,
/// which is what stops an invalid byte looping.
fn charwise_extent(cursor_bot: bool) -> (c_int, c_int) {
    let (mut bytes, mut chars) = (0, 0);
    let anchor = VIsual.get();
    // SAFETY: both pointers are into the current line, and the walk stops at
    // or before `e`.
    unsafe {
        let (mut s, e) = if cursor_bot {
            (ml_get_pos(&raw const anchor), get_cursor_pos_ptr())
        } else {
            (get_cursor_pos_ptr(), ml_get_pos(&raw const anchor))
        };
        let exclusive = *p_sel.get() as c_int == 'e' as c_int;
        while if exclusive { s < e } else { s <= e } {
            let l = utfc_ptr2len(s);
            if l == 0 {
                bytes += 1;
                chars += 1;
                break;
            }
            bytes += l;
            chars += 1;
            s = s.offset(l as isize);
        }
    }
    (chars, bytes)
}

/// Describe the Visual selection into the 'showcmd' buffer.
fn show_visual_size() {
    // SAFETY: `VIsual_active` is set, so `VIsual` holds a real position.
    let cursor_bot = unsafe { lt(VIsual.get(), (*curwin.get()).w_cursor) };
    let (top, bot) = visual_line_range(cursor_bot);
    let lines = (bot - top + 1) as c_int;

    let buf = showcmd_buf.ptr().cast::<c_char>();
    let cap = SHOWCMD_BUFLEN as size_t;
    // SAFETY: every call writes at most `SHOWCMD_BUFLEN` bytes into the
    // buffer, which is that long. snprintf's truncation is the behaviour
    // upstream relies on for a very wide block.
    unsafe {
        if VIsual_mode.get() == Ctrl_V {
            snprintf(
                buf,
                cap,
                c"%ldx%ld".as_ptr(),
                lines as int64_t,
                blockwise_width() as int64_t,
            );
        } else if VIsual_mode.get() == 'V' as c_int
            || VIsual.get().lnum != (*curwin.get()).w_cursor.lnum
        {
            snprintf(buf, cap, c"%ld".as_ptr(), lines as int64_t);
        } else {
            let (chars, bytes) = charwise_extent(cursor_bot);
            if bytes == chars {
                snprintf(buf, cap, c"%d".as_ptr(), chars);
            } else {
                snprintf(buf, cap, c"%d-%d".as_ptr(), chars, bytes);
            }
        }
    }
    showcmd_truncate(showcmd_limit());
    showcmd_visual.set(true);
}

/// Throw away the partial command, or replace it with the size of the
/// Visual selection while there is one and nothing is waiting to be typed.
pub(crate) fn clear_showcmd() {
    if p_sc.get() == 0 {
        return;
    }
    // SAFETY: reads a flag and the typeahead state.
    if VIsual_active.get() && unsafe { !char_avail() } {
        show_visual_size();
    } else {
        showcmd_truncate(0);
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
        showcmd_truncate(0);
        showcmd_visual.set(false);
    }
    if c < 0 && IGNORED.contains(&c) {
        return false;
    }

    let mut mbyte_buf: [c_char; 7] = [0; 7];
    // SAFETY: `transchar` answers a pointer to a static buffer; the
    // multibyte branch writes at most MB_MAXBYTES + 1 into `mbyte_buf`.
    let p = unsafe {
        if c <= 0x7f || !vim_isprintc(c) {
            let p = transchar(c);
            // A space is shown as its byte value, so it is not lost in the
            // padding the area is drawn with.
            if *p as c_int == ' ' as c_int {
                strcpy(p, c"<20>".as_ptr().cast_mut());
            }
            p
        } else {
            let n = utf_char2bytes(c, mbyte_buf.as_mut_ptr());
            mbyte_buf[n as usize] = NUL as c_char;
            mbyte_buf.as_mut_ptr()
        }
    };

    let buf = showcmd_buf.ptr().cast::<c_char>();
    // SAFETY: both are NUL-terminated C strings.
    let (old_len, extra_len) = unsafe { (showcmd_len(), strlen(p) as usize) };
    let limit = showcmd_limit();
    // Overflowing shifts the oldest bytes out rather than refusing to grow.
    if old_len + extra_len > limit {
        let overflow = old_len + extra_len - limit;
        // SAFETY: `overflow <= old_len`, so the source range and the length
        // (including the terminator) are both inside the buffer.
        unsafe {
            memmove(
                buf.cast::<c_void>(),
                buf.add(overflow).cast::<c_void>(),
                (old_len - overflow + 1) as size_t,
            );
        }
    }
    // SAFETY: the shift above left room for `extra_len` more bytes.
    unsafe { strcat(buf, p) };

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
    let old_len = showcmd_len();
    let len = (len as usize).min(old_len);
    showcmd_truncate(old_len - len);
    // SAFETY: reads the typeahead state.
    if unsafe { !char_avail() } {
        display_showcmd();
    }
}

/// Save the partial command across something that shows its own.
pub(crate) fn push_showcmd() {
    if p_sc.get() != 0 {
        // SAFETY: both arrays are SHOWCMD_BUFLEN long and NUL-terminated.
        unsafe {
            strcpy(
                old_showcmd_buf.ptr().cast::<c_char>(),
                showcmd_buf.ptr().cast::<c_char>(),
            )
        };
    }
}

/// Put back what `push_showcmd` saved.
pub(crate) fn pop_showcmd() {
    if p_sc.get() == 0 {
        return;
    }
    // SAFETY: as above.
    unsafe {
        strcpy(
            showcmd_buf.ptr().cast::<c_char>(),
            old_showcmd_buf.ptr().cast::<c_char>(),
        )
    };
    display_showcmd();
}

/// Hand the area's current contents to whichever of the four places
/// 'showcmdloc' names is showing it.
pub(crate) fn display_showcmd() {
    // SAFETY: reads the first byte of the buffer.
    showcmd_is_clear.set(unsafe { (*showcmd_buf.ptr())[0] as c_int == NUL });
    let clear = showcmd_is_clear.get();

    // SAFETY: 'showcmdloc' is a non-empty string option.
    let loc = unsafe { *p_sloc.get() as c_int };
    if loc == 's' as c_int {
        // SAFETY: `curwin` is the current window.
        unsafe {
            if clear {
                (*curwin.get()).w_redr_status = true;
            } else {
                win_redr_status(curwin.get());
                setcursor();
            }
        }
        return;
    }
    if loc == 't' as c_int {
        if clear {
            redraw_tabline.set(true);
        } else {
            // SAFETY: redraws the tab line and puts the cursor back.
            unsafe {
                draw_tabline();
                setcursor();
            }
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
        unsafe {
            *chunk.items.add(0) = integer_object(0);
            *chunk.items.add(1) = object {
                type_0: kObjectTypeString,
                data: object_data {
                    string: cstr_as_string(showcmd_buf.ptr().cast::<c_char>()),
                },
            };
            *chunk.items.add(2) = integer_object(0);
            chunk.size = 3;
            *content.items.add(0) = object {
                type_0: kObjectTypeArray,
                data: object_data { array: chunk },
            };
            content.size = 1;
        }
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
    // SAFETY: `msg_grid_adj` is the message grid and `showcmd_row` is its
    // last row; `grid_line_puts` bounds itself to the line it was started on.
    unsafe {
        msg_grid_validate();
        let showcmd_row = Rows.get() - 1;
        grid_line_start(msg_grid_adj.ptr(), showcmd_row);
        let attr = *(*hl_attr_active.ptr()).offset(HLF_MSG as isize);
        let mut len = 0;
        if !clear {
            len = grid_line_puts(sc_col.get(), showcmd_buf.ptr().cast::<c_char>(), -1, attr);
        }
        // The padding is the same ten spaces every time; only the tail past
        // what was written is drawn.
        grid_line_puts(
            sc_col.get() + len,
            c"          ".as_ptr().cast_mut().offset(len as isize),
            -1,
            attr,
        );
        grid_line_flush();
    }
}
