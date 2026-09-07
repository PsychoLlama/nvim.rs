//! `nvim_buf_set_text()`: replacing an arbitrary byte range.
//!
//! The one API call that can start and end mid-line, which is why it owns
//! three cursor fixups of its own: `fix_cursor` for a whole-line change,
//! `fix_pos_col` for a mark or cursor column inside the replaced span, and
//! `fix_cursor_cols` for the columns of every window showing the buffer.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{Reported, array_add};
use crate::api::private::validate::err_out_of_range;
use crate::r#move::WinValid;
use crate::normal::{set_visual_anchor, visual_active, visual_anchor, visual_mode};
use crate::types::NUL;
use crate::winlayer::{Buf, PosRef, Win, tab_windows};

pub unsafe fn nvim_buf_set_text(
    channel_id: uint64_t,
    buf: BufferHandle,
    mut start_row: Integer,
    mut start_col: Integer,
    mut end_row: Integer,
    mut end_col: Integer,
    mut replacement: Array,
    arena: *mut Arena,
) -> Result<(), Error> {
    let mut error = Error::none();
    let mut scratch: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut scratch_items: [Object; 1] = [Object::Nil; 1];
    scratch.capacity = 1 as size_t;
    scratch.items = &raw mut scratch_items as *mut Object;
    if replacement.size == 0 as size_t {
        let put_value = Object::string(String_0::from_raw_parts(
            c"".as_ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1]>().wrapping_sub(1 as size_t),
        ));
        // SAFETY: the collection is this call's own.
        unsafe { array_add(&mut scratch, put_value) };
        replacement = scratch;
    }
    let Some(b) = api_buf_ensure_loaded(buf, &mut error) else {
        return ().reported(error);
    };
    let buffer = b;
    let mut oob: bool = false;
    start_row = unsafe { normalize_index(b, start_row as int64_t, false, &raw mut oob) } as Integer;
    if oob {
        error = err_out_of_range(c"start_row");
        return ().reported(error);
    }
    end_row = unsafe { normalize_index(b, end_row as int64_t, false, &raw mut oob) } as Integer;
    if oob {
        error = err_out_of_range(c"end_row");
        return ().reported(error);
    }
    let mut str_at_start: *mut ::core::ffi::c_char = unsafe { ml_get_buf(b, start_row as LineNr) };
    let len_at_start: ColNr = unsafe { ml_get_buf_len(b, start_row as LineNr) };
    str_at_start = unsafe { arena_memdupz(arena, str_at_start, len_at_start as size_t) };
    start_col = if start_col < 0 as Integer {
        len_at_start as Integer + start_col + 1 as Integer
    } else {
        start_col
    };
    if !(start_col >= 0 as Integer && start_col <= len_at_start as Integer) {
        error = err_out_of_range(c"start_col");
        return ().reported(error);
    }
    let mut str_at_end: *mut ::core::ffi::c_char = unsafe { ml_get_buf(b, end_row as LineNr) };
    let len_at_end: ColNr = unsafe { ml_get_buf_len(b, end_row as LineNr) };
    str_at_end = unsafe { arena_memdupz(arena, str_at_end, len_at_end as size_t) };
    end_col = if end_col < 0 as Integer {
        len_at_end as Integer + end_col + 1 as Integer
    } else {
        end_col
    };
    if !(end_col >= 0 as Integer && end_col <= len_at_end as Integer) {
        error = err_out_of_range(c"end_col");
        return ().reported(error);
    }
    if !(start_row <= end_row && !(end_row == start_row && start_col > end_col)) {
        let why = c"'start' is higher than 'end'";
        error = Error::validation(why);
        return ().reported(error);
    }
    let disallow_nl: bool = channel_id != VIML_INTERNAL_CALL;
    // SAFETY: `replacement` is the caller's array.
    unsafe { check_string_array(replacement, c"replacement string", disallow_nl) }?;
    let new_len: size_t = replacement.size;
    let mut new_byte: BCount = 0 as BCount;
    let mut old_byte: BCount = 0 as BCount;
    if start_row == end_row {
        old_byte = end_col as BCount - start_col as BCount;
    } else {
        old_byte = (old_byte as ::core::ffi::c_long
            + (len_at_start as Integer - start_col) as ::core::ffi::c_long)
            as BCount;
        let mut i: int64_t = 1 as int64_t;
        while i < end_row - start_row {
            let lnum: int64_t = start_row as int64_t + i;
            old_byte +=
                (unsafe { ml_get_buf_len(b, lnum as LineNr) } + 1 as ::core::ffi::c_int) as BCount;
            i += 1;
        }
        old_byte += end_col as BCount + 1 as BCount;
    }
    let last_index = replacement.size.wrapping_sub(1 as size_t);
    // Every item is a String: `check_string_array` above turned anything else
    // into an error.
    let only_strings = "check_string_array accepted only Strings";
    // SAFETY: `replacement` is a non-empty array, so both indices are in it.
    let first_item: String_0 = unsafe { *replacement.items }
        .as_string()
        .expect(only_strings);
    // SAFETY: as above.
    let last_item: String_0 = unsafe { *replacement.items.add(last_index) }
        .as_string()
        .expect(only_strings);
    let mut firstlen: size_t = (start_col as size_t).wrapping_add(first_item.len());
    let last_part_len: size_t = (len_at_end as size_t).wrapping_sub(end_col as size_t);
    if replacement.size == 1 as size_t {
        firstlen = firstlen.wrapping_add(last_part_len);
    }
    let first: *mut ::core::ffi::c_char = unsafe { arena_allocz(arena, firstlen) };
    let mut last: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    // `memchrsub` turns embedded NULs back into the newlines they stand for.
    let nul = NUL as ::core::ffi::c_char;
    let nl = NL as ::core::ffi::c_char;
    // SAFETY: `first` has `firstlen` writable bytes, and `start_col` is
    // within the line it was measured against.
    let head = unsafe { first.offset(start_col as isize) } as *mut ::core::ffi::c_void;
    // SAFETY: `first` holds `start_col` bytes of the old line's head.
    let into = first.cast::<u8>();
    unsafe { into.copy_from_nonoverlapping(str_at_start.cast(), start_col as size_t) };
    let src = first_item.data() as *const ::core::ffi::c_void;
    // SAFETY: `head` has `first_item.len()` writable bytes after `start_col`.
    let into = head.cast::<u8>();
    unsafe { into.copy_from_nonoverlapping(src.cast(), first_item.len()) };
    // SAFETY: as above.
    unsafe { memchrsub(head, nul, nl, first_item.len()) };
    // SAFETY: `end_col` is within the line `str_at_end` copied.
    let tail = unsafe { str_at_end.offset(end_col as isize) } as *const ::core::ffi::c_void;
    if replacement.size == 1 as size_t {
        // SAFETY: `firstlen` counted `last_part_len` in as well.
        let after = unsafe { first.offset(start_col as isize).add(first_item.len()) };
        let after = after as *mut ::core::ffi::c_void;
        // SAFETY: `after` has `last_part_len` writable bytes.
        let into = after.cast::<u8>();
        unsafe { into.copy_from_nonoverlapping(tail.cast(), last_part_len) };
    } else {
        let lastlen = last_item.len().wrapping_add(last_part_len);
        // SAFETY: the arena hands back `lastlen` writable bytes.
        last = unsafe { arena_allocz(arena, lastlen) };
        let src = last_item.data() as *const ::core::ffi::c_void;
        // SAFETY: `last` has `lastlen` writable bytes.
        let into = last.cast::<u8>();
        unsafe { into.copy_from_nonoverlapping(src.cast(), last_item.len()) };
        // SAFETY: as above.
        unsafe { memchrsub(last.cast(), nul, nl, last_item.len()) };
        // SAFETY: the tail sits after the item, still inside `lastlen`.
        let after = unsafe { last.add(last_item.len()) } as *mut ::core::ffi::c_void;
        // SAFETY: `after` has `last_part_len` writable bytes.
        let into = after.cast::<u8>();
        unsafe { into.copy_from_nonoverlapping(tail.cast(), last_part_len) };
    }
    let lines: *mut *mut ::core::ffi::c_char = unsafe {
        arena_alloc(
            arena,
            new_len.wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
            true,
        )
    } as *mut *mut ::core::ffi::c_char;
    unsafe { *lines.offset(0 as ::core::ffi::c_int as isize) = first };
    new_byte += first_item.len() as BCount;
    let mut i_0: size_t = 1 as size_t;
    while i_0 < new_len.wrapping_sub(1 as size_t) {
        // SAFETY: `i_0` is below `replacement.size`.
        let l: String_0 = unsafe { *replacement.items.add(i_0) }
            .as_string()
            .expect(only_strings);
        unsafe { *lines.add(i_0) = arena_memdupz(arena, l.data(), l.len()) };
        // SAFETY: `i_0` is below `new_len`, so the slot was just written.
        let line = unsafe { *lines.add(i_0) } as *mut ::core::ffi::c_void;
        // SAFETY: `line` holds `l.len()` bytes.
        unsafe { memchrsub(line, nul, nl, l.len()) };
        new_byte += l.len() as BCount + 1 as BCount;
        i_0 = i_0.wrapping_add(1);
    }
    if replacement.size > 1 as size_t {
        unsafe { *lines.add(replacement.size.wrapping_sub(1 as size_t)) = last };
        new_byte += last_item.len() as BCount + 1 as BCount;
    }
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<Exception>(),
        private_msg_list: ::core::ptr::null_mut::<MsgList>(),
        msg_list: ::core::ptr::null::<*const MsgList>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    unsafe { try_enter(&raw mut tstate) };
    's_652: {
        if b.b_p_ma == 0 {
            let why = c"Buffer is not 'modifiable'";
            error = Error::exception(why);
        } else if u_save_buf(
            buffer,
            start_row as LineNr - 1 as LineNr,
            end_row as LineNr + 1 as LineNr,
        )
        .is_err()
        {
            let why = c"Failed to save undo information";
            error = Error::exception(why);
        } else {
            let mut extra: ptrdiff_t = 0 as ptrdiff_t;
            let old_len: size_t = (end_row - start_row + 1 as Integer) as size_t;
            let to_delete: size_t = if new_len < old_len {
                old_len.wrapping_sub(new_len)
            } else {
                0 as size_t
            };
            let mut i_1: size_t = 0 as size_t;
            while i_1 < to_delete {
                if unsafe { ml_delete_buf(b, start_row as LineNr, false) }.is_err() {
                    let why = c"Failed to delete line";
                    error = Error::exception(why);
                    break 's_652;
                } else {
                    i_1 = i_1.wrapping_add(1);
                }
            }
            if to_delete > 0 as size_t {
                extra -= to_delete as ptrdiff_t;
            }
            let to_replace: size_t = if old_len < new_len { old_len } else { new_len };
            let mut i_2: size_t = 0 as size_t;
            while i_2 < to_replace {
                let lnum_0: int64_t = start_row as int64_t + i_2 as int64_t;
                if !(lnum_0 < MAXLNUM as ::core::ffi::c_int as int64_t) {
                    let why = c"Index out of bounds";
                    error = Error::validation(why);
                    break 's_652;
                } else if unsafe {
                    ml_replace_buf(b, lnum_0 as LineNr, *lines.add(i_2), false, true)
                }
                .is_err()
                {
                    let why = c"Failed to replace line";
                    error = Error::exception(why);
                    break 's_652;
                } else {
                    i_2 = i_2.wrapping_add(1);
                }
            }
            let mut i_3: size_t = to_replace;
            while i_3 < new_len {
                let lnum_1: int64_t = start_row as int64_t + i_3 as int64_t - 1 as int64_t;
                if !(lnum_1 < MAXLNUM as ::core::ffi::c_int as int64_t) {
                    let why = c"Index out of bounds";
                    error = Error::validation(why);
                    break 's_652;
                } else if unsafe {
                    ml_append_buf(b, lnum_1 as LineNr, *lines.add(i_3), 0 as ColNr, false)
                }
                .is_err()
                {
                    let why = c"Failed to insert line";
                    error = Error::exception(why);
                    break 's_652;
                } else {
                    extra += 1;
                    i_3 = i_3.wrapping_add(1);
                }
            }
            let col_extent: ColNr = (end_col
                - (if end_row == start_row {
                    start_col
                } else {
                    0 as Integer
                })) as ColNr;
            let adjust: LineNr = if end_row >= start_row {
                MAXLNUM as ::core::ffi::c_int as LineNr
            } else {
                0 as LineNr
            };
            unsafe {
                mark_adjust_buf(
                    buffer,
                    start_row as LineNr,
                    end_row as LineNr - 1 as LineNr,
                    adjust,
                    extra as LineNr,
                    true,
                    kMarkAdjustApi,
                    kExtmarkNOOP,
                )
            };
            if visual_active() && b.raw() == Buf::current_raw() && !visual_mode().is_block() {
                let mut anchor = visual_anchor();
                unsafe {
                    fix_pos_col(
                        b,
                        &raw mut anchor,
                        start_row as LineNr,
                        start_col as ColNr,
                        end_row as LineNr,
                        end_col as ColNr,
                        new_len as LineNr,
                        last_item.len() as ColNr,
                        1 as ColNr,
                    )
                };
                set_visual_anchor(anchor);
                check_visual_pos();
            }
            extmark_splice(
                buffer,
                start_row as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                start_col as ColNr,
                (end_row - start_row) as ::core::ffi::c_int,
                col_extent,
                old_byte,
                new_len as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                last_item.len() as ColNr,
                new_byte,
                kExtmarkUndo,
            );
            changed_lines(
                buffer,
                start_row as LineNr,
                start_col as ColNr,
                end_row as LineNr + 1 as LineNr,
                extra as LineNr,
                true,
            );
            for win in tab_windows().map(Win::raw) {
                if unsafe { (*win).w_buffer } == b.raw() {
                    if unsafe { (*win).w_cursor.lnum } as Integer >= start_row
                        && unsafe { (*win).w_cursor.lnum } as Integer <= end_row
                    {
                        // SAFETY: a live window.
                        let win = unsafe { Win::new(win) };
                        unsafe {
                            fix_cursor_cols(
                                win,
                                start_row as LineNr,
                                start_col as ColNr,
                                end_row as LineNr,
                                end_col as ColNr,
                                new_len as LineNr,
                                last_item.len() as ColNr,
                            )
                        };
                    } else {
                        let (lo, hi) = (start_row as LineNr, end_row as LineNr);
                        // SAFETY: a live window showing this buffer.
                        unsafe { fix_cursor(Win::new(win), lo, hi, extra as LineNr) };
                    }
                }
            }
        }
    }
    unsafe { try_leave(&raw mut tstate, &mut error) };
    ().reported(error)
}

pub(crate) fn fix_cursor(mut win: Win, lo: LineNr, hi: LineNr, extra: LineNr) {
    if win.w_cursor.lnum >= lo {
        if win.w_cursor.lnum >= hi {
            win.w_cursor.lnum += extra;
        } else if extra < 0 as LineNr {
            check_cursor_lnum(win);
        }
        check_cursor_col(win);
        changed_cline_bef_curs(win);
        win.w_valid.clear(WinValid::BOTLINE_AP);
        update_topline(win);
    } else {
        invalidate_botline_win(win);
    };
}

unsafe fn fix_pos_col(
    buffer: Buf,
    pos: *mut Pos,
    start_row: LineNr,
    start_col: ColNr,
    end_row: LineNr,
    end_col: ColNr,
    new_rows: LineNr,
    new_cols_at_end_row: ColNr,
    mode_col_adj: ColNr,
) {
    // SAFETY: the caller's promise -- `pos` is a live position, and nothing
    // below can move it.
    let mut pos = unsafe { PosRef::new(pos) };
    if pos.lnum < start_row {
        return;
    }
    let old_rows: LineNr = end_row - start_row + 1 as LineNr;
    let lnum_shift: LineNr = new_rows - old_rows;
    if pos.lnum > end_row {
        pos.lnum += lnum_shift;
        return;
    }
    let end_row_change_start: ColNr = if new_rows == 1 as LineNr {
        start_col
    } else {
        0 as ColNr
    };
    let end_row_change_end: ColNr = end_row_change_start + new_cols_at_end_row;
    if pos.lnum == end_row && pos.col + mode_col_adj > end_col {
        pos.lnum += lnum_shift;
        pos.col += end_row_change_end - end_col;
        return;
    }
    let old_coladd: ColNr = pos.coladd;
    let coladd = pos.coladd;
    pos.col += coladd;
    pos.coladd = 0 as ::core::ffi::c_int as ColNr;
    let new_end_row: LineNr = start_row + new_rows - 1 as LineNr;
    if pos.lnum > new_end_row {
        pos.lnum = new_end_row;
        let len: ColNr = unsafe { ml_get_buf_len(buffer, new_end_row) };
        if pos.col < len {
            pos.col = len;
        }
    }
    if pos.lnum == new_end_row
        && pos.col > end_row_change_end
        && old_coladd == 0 as ::core::ffi::c_int
    {
        pos.col = end_row_change_end;
        if pos.col - mode_col_adj >= end_row_change_start {
            pos.col -= mode_col_adj;
        }
    }
}

unsafe fn fix_cursor_cols(
    mut win: Win,
    start_row: LineNr,
    start_col: ColNr,
    end_row: LineNr,
    end_col: ColNr,
    new_rows: LineNr,
    new_cols_at_end_row: ColNr,
) {
    let mode_col_adj: ColNr = if win == Win::current() && State.get() & MODE_INSERT != 0 {
        0 as ColNr
    } else {
        1 as ColNr
    };
    unsafe {
        fix_pos_col(
            win.buffer(),
            &raw mut win.w_cursor,
            start_row,
            start_col,
            end_row,
            end_col,
            new_rows,
            new_cols_at_end_row,
            mode_col_adj,
        )
    };
    check_cursor_col(win);
    changed_cline_bef_curs(win);
    invalidate_botline_win(win);
}
