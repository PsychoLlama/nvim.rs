//! `nvim_buf_set_text()`: replacing an arbitrary byte range.
//!
//! The one API call that can start and end mid-line, which is why it owns
//! three cursor fixups of its own: `fix_cursor` for a whole-line change,
//! `fix_pos_col` for a mark or cursor column inside the replaced span, and
//! `fix_cursor_cols` for the columns of every window showing the buffer.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, NIL, Reported, array_add};
use crate::api::private::validate::err_out_of_range;
use crate::r#move::WinValid;
use crate::normal::{set_visual_anchor, visual_active, visual_anchor, visual_mode};
use crate::types::NUL;
use crate::winlayer::{Buf, Pos, Win, tab_windows};

pub unsafe fn nvim_buf_set_text(
    channel_id: uint64_t,
    buf: Buffer,
    mut start_row: Integer,
    mut start_col: Integer,
    mut end_row: Integer,
    mut end_col: Integer,
    mut replacement: Array,
    arena: *mut Arena,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let mut scratch: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut scratch__items: [Object; 1] = [NIL; 1];
    scratch.capacity = 1 as size_t;
    scratch.items = &raw mut scratch__items as *mut Object;
    if replacement.size == 0 as size_t {
        let put_value = Object::string(String_0::from_raw_parts(
            c"".as_ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1]>().wrapping_sub(1 as size_t),
        ));
        // SAFETY: the collection is this call's own.
        unsafe { array_add(&mut scratch, put_value) };
        replacement = scratch;
    }
    let mut b: *mut buf_T = unsafe { api_buf_ensure_loaded(buf, &mut error) };
    if b.is_null() {
        return ().reported(error);
    }
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
    let mut str_at_start: *mut ::core::ffi::c_char =
        unsafe { ml_get_buf(b, start_row as linenr_T) };
    let mut len_at_start: colnr_T = unsafe { ml_get_buf_len(b, start_row as linenr_T) };
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
    let mut str_at_end: *mut ::core::ffi::c_char = unsafe { ml_get_buf(b, end_row as linenr_T) };
    let mut len_at_end: colnr_T = unsafe { ml_get_buf_len(b, end_row as linenr_T) };
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
    let mut disallow_nl: bool = channel_id != VIML_INTERNAL_CALL;
    // SAFETY: `replacement` is the caller's array.
    unsafe { check_string_array(replacement, c"replacement string", disallow_nl) }?;
    let mut new_len: size_t = replacement.size;
    let mut new_byte: bcount_t = 0 as bcount_t;
    let mut old_byte: bcount_t = 0 as bcount_t;
    if start_row == end_row {
        old_byte = end_col as bcount_t - start_col as bcount_t;
    } else {
        old_byte = (old_byte as ::core::ffi::c_long
            + (len_at_start as Integer - start_col) as ::core::ffi::c_long)
            as bcount_t;
        let mut i: int64_t = 1 as int64_t;
        while i < end_row - start_row {
            let mut lnum: int64_t = start_row as int64_t + i;
            old_byte += (unsafe { ml_get_buf_len(b, lnum as linenr_T) } + 1 as ::core::ffi::c_int)
                as bcount_t;
            i += 1;
        }
        old_byte += end_col as bcount_t + 1 as bcount_t;
    }
    let last_index = replacement.size.wrapping_sub(1 as size_t);
    // SAFETY: `replacement` is a non-empty array, so both indices are in it.
    let first_item: String_0 = unsafe { (*replacement.items).data.string };
    // SAFETY: as above.
    let last_item: String_0 = unsafe { (*replacement.items.add(last_index)).data.string };
    let mut firstlen: size_t = (start_col as size_t).wrapping_add(first_item.len());
    let mut last_part_len: size_t = (len_at_end as size_t).wrapping_sub(end_col as size_t);
    if replacement.size == 1 as size_t {
        firstlen = firstlen.wrapping_add(last_part_len);
    }
    let mut first: *mut ::core::ffi::c_char = unsafe { arena_allocz(arena, firstlen) };
    let mut last: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    // `memchrsub` turns embedded NULs back into the newlines they stand for.
    let nul = NUL as ::core::ffi::c_char;
    let nl = NL as ::core::ffi::c_char;
    // SAFETY: `first` has `firstlen` writable bytes, and `start_col` is
    // within the line it was measured against.
    let head = unsafe { first.offset(start_col as isize) } as *mut ::core::ffi::c_void;
    // SAFETY: `first` holds `start_col` bytes of the old line's head.
    unsafe { memcpy(first.cast(), str_at_start.cast(), start_col as size_t) };
    let src = first_item.data() as *const ::core::ffi::c_void;
    // SAFETY: `head` has `first_item.len()` writable bytes after `start_col`.
    unsafe { memcpy(head, src, first_item.len()) };
    // SAFETY: as above.
    unsafe { memchrsub(head, nul, nl, first_item.len()) };
    // SAFETY: `end_col` is within the line `str_at_end` copied.
    let tail = unsafe { str_at_end.offset(end_col as isize) } as *const ::core::ffi::c_void;
    if replacement.size == 1 as size_t {
        // SAFETY: `firstlen` counted `last_part_len` in as well.
        let after = unsafe { first.offset(start_col as isize).add(first_item.len()) };
        let after = after as *mut ::core::ffi::c_void;
        // SAFETY: `after` has `last_part_len` writable bytes.
        unsafe { memcpy(after, tail, last_part_len) };
    } else {
        let lastlen = last_item.len().wrapping_add(last_part_len);
        // SAFETY: the arena hands back `lastlen` writable bytes.
        last = unsafe { arena_allocz(arena, lastlen) };
        let src = last_item.data() as *const ::core::ffi::c_void;
        // SAFETY: `last` has `lastlen` writable bytes.
        unsafe { memcpy(last.cast(), src, last_item.len()) };
        // SAFETY: as above.
        unsafe { memchrsub(last.cast(), nul, nl, last_item.len()) };
        // SAFETY: the tail sits after the item, still inside `lastlen`.
        let after = unsafe { last.add(last_item.len()) } as *mut ::core::ffi::c_void;
        // SAFETY: `after` has `last_part_len` writable bytes.
        unsafe { memcpy(after, tail, last_part_len) };
    }
    let mut lines: *mut *mut ::core::ffi::c_char = unsafe {
        arena_alloc(
            arena,
            new_len.wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
            true,
        )
    } as *mut *mut ::core::ffi::c_char;
    unsafe { *lines.offset(0 as ::core::ffi::c_int as isize) = first };
    new_byte += first_item.len() as bcount_t;
    let mut i_0: size_t = 1 as size_t;
    while i_0 < new_len.wrapping_sub(1 as size_t) {
        let l: String_0 = unsafe { (*replacement.items.add(i_0)).data.string };
        unsafe { *lines.add(i_0) = arena_memdupz(arena, l.data(), l.len()) };
        // SAFETY: `i_0` is below `new_len`, so the slot was just written.
        let line = unsafe { *lines.add(i_0) } as *mut ::core::ffi::c_void;
        // SAFETY: `line` holds `l.len()` bytes.
        unsafe { memchrsub(line, nul, nl, l.len()) };
        new_byte += l.len() as bcount_t + 1 as bcount_t;
        i_0 = i_0.wrapping_add(1);
    }
    if replacement.size > 1 as size_t {
        unsafe { *lines.add(replacement.size.wrapping_sub(1 as size_t)) = last };
        new_byte += last_item.len() as bcount_t + 1 as bcount_t;
    }
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    unsafe { try_enter(&raw mut tstate) };
    's_652: {
        if unsafe { (*b).b_p_ma } == 0 {
            let why = c"Buffer is not 'modifiable'";
            error = Error::exception(why);
        } else if u_save_buf(
            unsafe { Buf::new(b) },
            start_row as linenr_T - 1 as linenr_T,
            end_row as linenr_T + 1 as linenr_T,
        )
        .is_err()
        {
            let why = c"Failed to save undo information";
            error = Error::exception(why);
        } else {
            let mut extra: ptrdiff_t = 0 as ptrdiff_t;
            let mut old_len: size_t = (end_row - start_row + 1 as Integer) as size_t;
            let mut to_delete: size_t = if new_len < old_len {
                old_len.wrapping_sub(new_len)
            } else {
                0 as size_t
            };
            let mut i_1: size_t = 0 as size_t;
            while i_1 < to_delete {
                if unsafe { ml_delete_buf(b, start_row as linenr_T, false) }.is_err() {
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
            let mut to_replace: size_t = if old_len < new_len { old_len } else { new_len };
            let mut i_2: size_t = 0 as size_t;
            while i_2 < to_replace {
                let mut lnum_0: int64_t = start_row as int64_t + i_2 as int64_t;
                if !(lnum_0 < MAXLNUM as ::core::ffi::c_int as int64_t) {
                    let why = c"Index out of bounds";
                    error = Error::validation(why);
                    break 's_652;
                } else if unsafe {
                    ml_replace_buf(b, lnum_0 as linenr_T, *lines.add(i_2), false, true)
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
                let mut lnum_1: int64_t = start_row as int64_t + i_3 as int64_t - 1 as int64_t;
                if !(lnum_1 < MAXLNUM as ::core::ffi::c_int as int64_t) {
                    let why = c"Index out of bounds";
                    error = Error::validation(why);
                    break 's_652;
                } else if unsafe {
                    ml_append_buf(b, lnum_1 as linenr_T, *lines.add(i_3), 0 as colnr_T, false)
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
            let mut col_extent: colnr_T = (end_col
                - (if end_row == start_row {
                    start_col
                } else {
                    0 as Integer
                })) as colnr_T;
            let mut adjust: linenr_T = if end_row >= start_row {
                MAXLNUM as ::core::ffi::c_int as linenr_T
            } else {
                0 as linenr_T
            };
            unsafe {
                mark_adjust_buf(
                    b,
                    start_row as linenr_T,
                    end_row as linenr_T - 1 as linenr_T,
                    adjust,
                    extra as linenr_T,
                    true,
                    kMarkAdjustApi,
                    kExtmarkNOOP,
                )
            };
            if visual_active() && b == curbuf.get() && !visual_mode().is_block() {
                let mut anchor = visual_anchor();
                unsafe {
                    fix_pos_col(
                        b,
                        &raw mut anchor,
                        start_row as linenr_T,
                        start_col as colnr_T,
                        end_row as linenr_T,
                        end_col as colnr_T,
                        new_len as linenr_T,
                        last_item.len() as colnr_T,
                        1 as colnr_T,
                    )
                };
                set_visual_anchor(anchor);
                unsafe { check_visual_pos() };
            }
            unsafe {
                extmark_splice(
                    b,
                    start_row as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    start_col as colnr_T,
                    (end_row - start_row) as ::core::ffi::c_int,
                    col_extent,
                    old_byte,
                    new_len as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    last_item.len() as colnr_T,
                    new_byte,
                    kExtmarkUndo,
                )
            };
            changed_lines(
                unsafe { Buf::new(b) },
                start_row as linenr_T,
                start_col as colnr_T,
                end_row as linenr_T + 1 as linenr_T,
                extra as linenr_T,
                true,
            );
            for win in tab_windows().map(Win::raw) {
                if unsafe { (*win).w_buffer } == b {
                    if unsafe { (*win).w_cursor.lnum } as Integer >= start_row
                        && unsafe { (*win).w_cursor.lnum } as Integer <= end_row
                    {
                        unsafe {
                            fix_cursor_cols(
                                win,
                                start_row as linenr_T,
                                start_col as colnr_T,
                                end_row as linenr_T,
                                end_col as colnr_T,
                                new_len as linenr_T,
                                last_item.len() as colnr_T,
                            )
                        };
                    } else {
                        let (lo, hi) = (start_row as linenr_T, end_row as linenr_T);
                        // SAFETY: a live window showing this buffer.
                        unsafe { fix_cursor(win, lo, hi, extra as linenr_T) };
                    }
                }
            }
        }
    }
    unsafe { try_leave(&raw mut tstate, &mut error) };
    ().reported(error)
}

pub(crate) unsafe fn fix_cursor(
    win: *mut win_T,
    mut lo: linenr_T,
    mut hi: linenr_T,
    mut extra: linenr_T,
) {
    // SAFETY: the caller's promise -- `win` is a live window.
    let mut win = unsafe { Win::new(win) };
    if win.w_cursor.lnum >= lo {
        if win.w_cursor.lnum >= hi {
            win.w_cursor.lnum += extra;
        } else if extra < 0 as linenr_T {
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
    buf: *mut buf_T,
    pos: *mut pos_T,
    mut start_row: linenr_T,
    mut start_col: colnr_T,
    mut end_row: linenr_T,
    mut end_col: colnr_T,
    mut new_rows: linenr_T,
    mut new_cols_at_end_row: colnr_T,
    mut mode_col_adj: colnr_T,
) {
    // SAFETY: the caller's promise -- `pos` is a live position, and nothing
    // below can move it.
    let mut pos = unsafe { Pos::new(pos) };
    if pos.lnum < start_row {
        return;
    }
    let mut old_rows: linenr_T = end_row - start_row + 1 as linenr_T;
    let mut lnum_shift: linenr_T = new_rows - old_rows;
    if pos.lnum > end_row {
        pos.lnum += lnum_shift;
        return;
    }
    let mut end_row_change_start: colnr_T = if new_rows == 1 as linenr_T {
        start_col
    } else {
        0 as colnr_T
    };
    let mut end_row_change_end: colnr_T = end_row_change_start + new_cols_at_end_row;
    if pos.lnum == end_row && pos.col + mode_col_adj > end_col {
        pos.lnum += lnum_shift;
        pos.col += end_row_change_end - end_col;
        return;
    }
    let mut old_coladd: colnr_T = pos.coladd;
    let coladd = pos.coladd;
    pos.col += coladd;
    pos.coladd = 0 as ::core::ffi::c_int as colnr_T;
    let mut new_end_row: linenr_T = start_row + new_rows - 1 as linenr_T;
    if pos.lnum > new_end_row {
        pos.lnum = new_end_row;
        let mut len: colnr_T = unsafe { ml_get_buf_len(buf, new_end_row) };
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
    mut win: *mut win_T,
    mut start_row: linenr_T,
    mut start_col: colnr_T,
    mut end_row: linenr_T,
    mut end_col: colnr_T,
    mut new_rows: linenr_T,
    mut new_cols_at_end_row: colnr_T,
) {
    let mut mode_col_adj: colnr_T = if win == curwin.get() && State.get() & MODE_INSERT != 0 {
        0 as colnr_T
    } else {
        1 as colnr_T
    };
    unsafe {
        fix_pos_col(
            (*win).w_buffer,
            &raw mut (*win).w_cursor,
            start_row,
            start_col,
            end_row,
            end_col,
            new_rows,
            new_cols_at_end_row,
            mode_col_adj,
        )
    };
    check_cursor_col(unsafe { Win::new(win) });
    changed_cline_bef_curs(unsafe { Win::new(win) });
    invalidate_botline_win(unsafe { Win::new(win) });
}
