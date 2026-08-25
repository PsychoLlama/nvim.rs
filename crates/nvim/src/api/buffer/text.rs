//! `nvim_buf_set_text()`: replacing an arbitrary byte range.
//!
//! The one API call that can start and end mid-line, which is why it owns
//! three cursor fixups of its own: `fix_cursor` for a whole-line change,
//! `fix_pos_col` for a mark or cursor column inside the replaced span, and
//! `fix_cursor_cols` for the columns of every window showing the buffer.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, NIL, Reported, array_add};
use crate::r#move::WinValid;
use crate::normal::{set_visual_anchor, visual_active, visual_anchor, visual_mode};
use crate::types::NUL;

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
    let err = &raw mut error;
    unsafe {
        let mut scratch: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut scratch__items: [Object; 1] = [NIL; 1];
        scratch.capacity = 1 as size_t;
        scratch.items = &raw mut scratch__items as *mut Object;
        if replacement.size == 0 as size_t {
            array_add(
                &mut scratch,
                Object::string(String_0::from_raw_parts(
                    c"".as_ptr() as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1]>().wrapping_sub(1 as size_t),
                )),
            );
            replacement = scratch;
        }
        let mut b: *mut buf_T = api_buf_ensure_loaded(buf, err);
        if b.is_null() {
            return ().reported(error);
        }
        let mut oob: bool = false;
        start_row = normalize_index(b, start_row as int64_t, false, &raw mut oob) as Integer;
        if oob {
            api_err_invalid(
                err,
                c"start_row".as_ptr(),
                c"out of range".as_ptr(),
                0 as int64_t,
                false,
            );
            return ().reported(error);
        }
        end_row = normalize_index(b, end_row as int64_t, false, &raw mut oob) as Integer;
        if oob {
            api_err_invalid(
                err,
                c"end_row".as_ptr(),
                c"out of range".as_ptr(),
                0 as int64_t,
                false,
            );
            return ().reported(error);
        }
        let mut str_at_start: *mut ::core::ffi::c_char = ml_get_buf(b, start_row as linenr_T);
        let mut len_at_start: colnr_T = ml_get_buf_len(b, start_row as linenr_T);
        str_at_start = arena_memdupz(arena, str_at_start, len_at_start as size_t);
        start_col = if start_col < 0 as Integer {
            len_at_start as Integer + start_col + 1 as Integer
        } else {
            start_col
        };
        if !(start_col >= 0 as Integer && start_col <= len_at_start as Integer) {
            api_err_invalid(
                err,
                c"start_col".as_ptr(),
                c"out of range".as_ptr(),
                0 as int64_t,
                false,
            );
            return ().reported(error);
        }
        let mut str_at_end: *mut ::core::ffi::c_char = ml_get_buf(b, end_row as linenr_T);
        let mut len_at_end: colnr_T = ml_get_buf_len(b, end_row as linenr_T);
        str_at_end = arena_memdupz(arena, str_at_end, len_at_end as size_t);
        end_col = if end_col < 0 as Integer {
            len_at_end as Integer + end_col + 1 as Integer
        } else {
            end_col
        };
        if !(end_col >= 0 as Integer && end_col <= len_at_end as Integer) {
            api_err_invalid(
                err,
                c"end_col".as_ptr(),
                c"out of range".as_ptr(),
                0 as int64_t,
                false,
            );
            return ().reported(error);
        }
        if !(start_row <= end_row && !(end_row == start_row && start_col > end_col)) {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                c"'start' is higher than 'end'".as_ptr(),
            );
            return ().reported(error);
        }
        let mut disallow_nl: bool = channel_id != VIML_INTERNAL_CALL;
        if !check_string_array(
            replacement,
            c"replacement string".as_ptr() as *mut ::core::ffi::c_char,
            disallow_nl,
            err,
        ) {
            return ().reported(error);
        }
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
                old_byte +=
                    (ml_get_buf_len(b, lnum as linenr_T) + 1 as ::core::ffi::c_int) as bcount_t;
                i += 1;
            }
            old_byte += end_col as bcount_t + 1 as bcount_t;
        }
        let mut first_item: String_0 =
            (*replacement.items.offset(0 as ::core::ffi::c_int as isize))
                .data
                .string;
        let mut last_item: String_0 = (*replacement
            .items
            .add(replacement.size.wrapping_sub(1 as size_t)))
        .data
        .string;
        let mut firstlen: size_t = (start_col as size_t).wrapping_add(first_item.len());
        let mut last_part_len: size_t = (len_at_end as size_t).wrapping_sub(end_col as size_t);
        if replacement.size == 1 as size_t {
            firstlen = firstlen.wrapping_add(last_part_len);
        }
        let mut first: *mut ::core::ffi::c_char = arena_allocz(arena, firstlen);
        let mut last: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        memcpy(
            first as *mut ::core::ffi::c_void,
            str_at_start as *const ::core::ffi::c_void,
            start_col as size_t,
        );
        memcpy(
            first.offset(start_col as isize) as *mut ::core::ffi::c_void,
            first_item.data() as *const ::core::ffi::c_void,
            first_item.len(),
        );
        memchrsub(
            first.offset(start_col as isize) as *mut ::core::ffi::c_void,
            NUL as ::core::ffi::c_char,
            NL as ::core::ffi::c_char,
            first_item.len(),
        );
        if replacement.size == 1 as size_t {
            memcpy(
                first.offset(start_col as isize).add(first_item.len()) as *mut ::core::ffi::c_void,
                str_at_end.offset(end_col as isize) as *const ::core::ffi::c_void,
                last_part_len,
            );
        } else {
            last = arena_allocz(arena, last_item.len().wrapping_add(last_part_len));
            memcpy(
                last as *mut ::core::ffi::c_void,
                last_item.data() as *const ::core::ffi::c_void,
                last_item.len(),
            );
            memchrsub(
                last as *mut ::core::ffi::c_void,
                NUL as ::core::ffi::c_char,
                NL as ::core::ffi::c_char,
                last_item.len(),
            );
            memcpy(
                last.add(last_item.len()) as *mut ::core::ffi::c_void,
                str_at_end.offset(end_col as isize) as *const ::core::ffi::c_void,
                last_part_len,
            );
        }
        let mut lines: *mut *mut ::core::ffi::c_char = arena_alloc(
            arena,
            new_len.wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
            true,
        ) as *mut *mut ::core::ffi::c_char;
        *lines.offset(0 as ::core::ffi::c_int as isize) = first;
        new_byte += first_item.len() as bcount_t;
        let mut i_0: size_t = 1 as size_t;
        while i_0 < new_len.wrapping_sub(1 as size_t) {
            let l: String_0 = (*replacement.items.add(i_0)).data.string;
            *lines.add(i_0) = arena_memdupz(arena, l.data(), l.len());
            memchrsub(
                *lines.add(i_0) as *mut ::core::ffi::c_void,
                NUL as ::core::ffi::c_char,
                NL as ::core::ffi::c_char,
                l.len(),
            );
            new_byte += l.len() as bcount_t + 1 as bcount_t;
            i_0 = i_0.wrapping_add(1);
        }
        if replacement.size > 1 as size_t {
            *lines.add(replacement.size.wrapping_sub(1 as size_t)) = last;
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
        try_enter(&raw mut tstate);
        's_652: {
            if (*b).b_p_ma == 0 {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"Buffer is not 'modifiable'".as_ptr(),
                );
            } else if u_save_buf(
                b,
                start_row as linenr_T - 1 as linenr_T,
                end_row as linenr_T + 1 as linenr_T,
            ) == 0 as ::core::ffi::c_int
            {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"Failed to save undo information".as_ptr(),
                );
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
                    if ml_delete_buf(b, start_row as linenr_T, false) == 0 as ::core::ffi::c_int {
                        api_set_error(err, kErrorTypeException, c"Failed to delete line".as_ptr());
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
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            c"%s".as_ptr(),
                            c"Index out of bounds".as_ptr(),
                        );
                        break 's_652;
                    } else if ml_replace_buf(b, lnum_0 as linenr_T, *lines.add(i_2), false, true)
                        == 0 as ::core::ffi::c_int
                    {
                        api_set_error(err, kErrorTypeException, c"Failed to replace line".as_ptr());
                        break 's_652;
                    } else {
                        i_2 = i_2.wrapping_add(1);
                    }
                }
                let mut i_3: size_t = to_replace;
                while i_3 < new_len {
                    let mut lnum_1: int64_t = start_row as int64_t + i_3 as int64_t - 1 as int64_t;
                    if !(lnum_1 < MAXLNUM as ::core::ffi::c_int as int64_t) {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            c"%s".as_ptr(),
                            c"Index out of bounds".as_ptr(),
                        );
                        break 's_652;
                    } else if ml_append_buf(
                        b,
                        lnum_1 as linenr_T,
                        *lines.add(i_3),
                        0 as colnr_T,
                        false,
                    ) == 0 as ::core::ffi::c_int
                    {
                        api_set_error(err, kErrorTypeException, c"Failed to insert line".as_ptr());
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
                mark_adjust_buf(
                    b,
                    start_row as linenr_T,
                    end_row as linenr_T - 1 as linenr_T,
                    adjust,
                    extra as linenr_T,
                    true,
                    kMarkAdjustApi,
                    kExtmarkNOOP,
                );
                if visual_active() && b == curbuf.get() && !visual_mode().is_block() {
                    let mut anchor = visual_anchor();
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
                    );
                    set_visual_anchor(anchor);
                    check_visual_pos();
                }
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
                );
                changed_lines(
                    b,
                    start_row as linenr_T,
                    start_col as colnr_T,
                    end_row as linenr_T + 1 as linenr_T,
                    extra as linenr_T,
                    true,
                );
                let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                while !tp.is_null() {
                    let mut win: *mut win_T = if tp == curtab.get() {
                        firstwin.get()
                    } else {
                        (*tp).tp_firstwin
                    };
                    while !win.is_null() {
                        if (*win).w_buffer == b {
                            if (*win).w_cursor.lnum as Integer >= start_row
                                && (*win).w_cursor.lnum as Integer <= end_row
                            {
                                fix_cursor_cols(
                                    win,
                                    start_row as linenr_T,
                                    start_col as colnr_T,
                                    end_row as linenr_T,
                                    end_col as colnr_T,
                                    new_len as linenr_T,
                                    last_item.len() as colnr_T,
                                );
                            } else {
                                fix_cursor(
                                    win,
                                    start_row as linenr_T,
                                    end_row as linenr_T,
                                    extra as linenr_T,
                                );
                            }
                        }
                        win = (*win).w_next;
                    }
                    tp = (*tp).tp_next as *mut tabpage_T;
                }
            }
        }
        try_leave(&raw mut tstate, err);
    }
    ().reported(error)
}

pub(crate) unsafe fn fix_cursor(
    mut win: *mut win_T,
    mut lo: linenr_T,
    mut hi: linenr_T,
    mut extra: linenr_T,
) {
    unsafe {
        if (*win).w_cursor.lnum >= lo {
            if (*win).w_cursor.lnum >= hi {
                (*win).w_cursor.lnum += extra;
            } else if extra < 0 as linenr_T {
                check_cursor_lnum(win);
            }
            check_cursor_col(win);
            changed_cline_bef_curs(win);
            (*win).w_valid.clear(WinValid::BOTLINE_AP);
            update_topline(win);
        } else {
            invalidate_botline_win(win);
        };
    }
}

unsafe fn fix_pos_col(
    mut buf: *mut buf_T,
    mut pos: *mut pos_T,
    mut start_row: linenr_T,
    mut start_col: colnr_T,
    mut end_row: linenr_T,
    mut end_col: colnr_T,
    mut new_rows: linenr_T,
    mut new_cols_at_end_row: colnr_T,
    mut mode_col_adj: colnr_T,
) {
    unsafe {
        if (*pos).lnum < start_row {
            return;
        }
        let mut old_rows: linenr_T = end_row - start_row + 1 as linenr_T;
        let mut lnum_shift: linenr_T = new_rows - old_rows;
        if (*pos).lnum > end_row {
            (*pos).lnum += lnum_shift;
            return;
        }
        let mut end_row_change_start: colnr_T = if new_rows == 1 as linenr_T {
            start_col
        } else {
            0 as colnr_T
        };
        let mut end_row_change_end: colnr_T = end_row_change_start + new_cols_at_end_row;
        if (*pos).lnum == end_row && (*pos).col + mode_col_adj > end_col {
            (*pos).lnum += lnum_shift;
            (*pos).col += end_row_change_end - end_col;
            return;
        }
        let mut old_coladd: colnr_T = (*pos).coladd;
        (*pos).col += (*pos).coladd;
        (*pos).coladd = 0 as ::core::ffi::c_int as colnr_T;
        let mut new_end_row: linenr_T = start_row + new_rows - 1 as linenr_T;
        if (*pos).lnum > new_end_row {
            (*pos).lnum = new_end_row;
            let mut len: colnr_T = ml_get_buf_len(buf, new_end_row);
            if (*pos).col < len {
                (*pos).col = len;
            }
        }
        if (*pos).lnum == new_end_row
            && (*pos).col > end_row_change_end
            && old_coladd == 0 as ::core::ffi::c_int
        {
            (*pos).col = end_row_change_end;
            if (*pos).col - mode_col_adj >= end_row_change_start {
                (*pos).col -= mode_col_adj;
            }
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
    unsafe {
        let mut mode_col_adj: colnr_T = if win == curwin.get() && State.get() & MODE_INSERT != 0 {
            0 as colnr_T
        } else {
            1 as colnr_T
        };
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
        );
        check_cursor_col(win);
        changed_cline_bef_curs(win);
        invalidate_botline_win(win);
    }
}
