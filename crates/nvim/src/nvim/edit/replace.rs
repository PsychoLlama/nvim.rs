//! The replace stack: what Replace mode has to put back.
//!
//! In Replace mode a typed character overwrites one that was already there,
//! and backspacing has to restore it.  The stack is a flat byte vector of
//! NUL-headed lists, one per character position the insert has passed over:
//! `replace_push` adds the bytes a character replaced, `replace_push_nul`
//! ends an entry, and `replace_do_bs` pops one entry and writes it back.  A
//! newline pushes *two* entries, the second holding the white space that was
//! deleted after the cursor, which is what `replace_join` merges back.
//!
//! `truncate_spaces`, `backspace_until_column` and `del_char_after_col` are
//! the delete primitives that know about all of this: every one of them has
//! a Replace-mode arm that unwinds the stack rather than deleting text.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn truncate_spaces(mut line: *mut ::core::ffi::c_char, mut len: size_t) {
    unsafe {
        let mut i: ::core::ffi::c_int = 0;
        i = len as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int
            && ascii_iswhite(*line.offset(i as isize) as ::core::ffi::c_int) as ::core::ffi::c_int
                != 0
        {
            if State.get() & REPLACE_FLAG != 0 {
                replace_join(0 as ::core::ffi::c_int);
            }
            i -= 1;
        }
        *line.offset((i + 1 as ::core::ffi::c_int) as isize) = NUL as ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn backspace_until_column(mut col: ::core::ffi::c_int) {
    unsafe {
        while (*curwin.get()).w_cursor.col > col {
            (*curwin.get()).w_cursor.col -= 1;
            if State.get() & REPLACE_FLAG != 0 {
                replace_do_bs(col);
            } else if !del_char_after_col(col) {
                break;
            }
        }
    }
}

unsafe extern "C" fn del_char_after_col(mut limit_col: ::core::ffi::c_int) -> bool {
    unsafe {
        if limit_col >= 0 as ::core::ffi::c_int {
            let mut ecol: colnr_T = (*curwin.get()).w_cursor.col + 1 as colnr_T;
            mb_adjust_cursor();
            while (*curwin.get()).w_cursor.col < limit_col {
                let mut l: ::core::ffi::c_int = utf_ptr2len(get_cursor_pos_ptr());
                if l == 0 as ::core::ffi::c_int {
                    break;
                }
                (*curwin.get()).w_cursor.col += l;
            }
            if *get_cursor_pos_ptr() as ::core::ffi::c_int == NUL
                || (*curwin.get()).w_cursor.col == ecol
            {
                return false_0 != 0;
            }
            del_bytes(
                ecol - (*curwin.get()).w_cursor.col,
                false_0 != 0,
                true_0 != 0,
            );
        } else {
            del_char(false_0 != 0);
        }
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn replace_push(mut str: *mut ::core::ffi::c_char, mut len: size_t) {
    unsafe {
        if (*replace_stack.ptr()).size < replace_offset.get() as size_t {
            return;
        }
        if (*replace_stack.ptr()).capacity < (*replace_stack.ptr()).size.wrapping_add(len) {
            (*replace_stack.ptr()).capacity = (*replace_stack.ptr()).size.wrapping_add(len);
            (*replace_stack.ptr()).capacity = (*replace_stack.ptr()).capacity.wrapping_sub(1);
            (*replace_stack.ptr()).capacity |=
                (*replace_stack.ptr()).capacity >> 1 as ::core::ffi::c_int;
            (*replace_stack.ptr()).capacity |=
                (*replace_stack.ptr()).capacity >> 2 as ::core::ffi::c_int;
            (*replace_stack.ptr()).capacity |=
                (*replace_stack.ptr()).capacity >> 4 as ::core::ffi::c_int;
            (*replace_stack.ptr()).capacity |=
                (*replace_stack.ptr()).capacity >> 8 as ::core::ffi::c_int;
            (*replace_stack.ptr()).capacity |=
                (*replace_stack.ptr()).capacity >> 16 as ::core::ffi::c_int;
            (*replace_stack.ptr()).capacity = (*replace_stack.ptr()).capacity.wrapping_add(1);
            (*replace_stack.ptr()).capacity = (*replace_stack.ptr()).capacity;
            (*replace_stack.ptr()).items = xrealloc(
                (*replace_stack.ptr()).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul((*replace_stack.ptr()).capacity),
            ) as *mut ::core::ffi::c_char;
        }
        let mut p: *mut ::core::ffi::c_char = (*replace_stack.ptr())
            .items
            .offset((*replace_stack.ptr()).size as isize)
            .offset(-(replace_offset.get() as isize));
        if replace_offset.get() != 0 {
            memmove(
                p.offset(len as isize) as *mut ::core::ffi::c_void,
                p as *const ::core::ffi::c_void,
                replace_offset.get() as size_t,
            );
        }
        memcpy(
            p as *mut ::core::ffi::c_void,
            str as *const ::core::ffi::c_void,
            len,
        );
        (*replace_stack.ptr()).size = (*replace_stack.ptr()).size.wrapping_add(len);
    }
}

pub unsafe extern "C" fn replace_push_nul() {
    unsafe {
        replace_push(
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            1 as size_t,
        );
    }
}

pub(crate) unsafe extern "C" fn replace_pop_if_nul() -> ::core::ffi::c_int {
    unsafe {
        let mut ch: ::core::ffi::c_int = if (*replace_stack.ptr()).size != 0 {
            *(*replace_stack.ptr())
                .items
                .offset((*replace_stack.ptr()).size.wrapping_sub(1 as size_t) as isize)
                as uint8_t as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
        if ch == NUL {
            (*replace_stack.ptr()).size = (*replace_stack.ptr()).size.wrapping_sub(1);
        }
        return ch;
    }
}

pub unsafe extern "C" fn replace_join(mut off: ::core::ffi::c_int) {
    unsafe {
        let mut i: ssize_t = (*replace_stack.ptr()).size as ssize_t;
        loop {
            i -= 1;
            if i < 0 as ssize_t {
                break;
            }
            if *(*replace_stack.ptr()).items.offset(i as isize) as ::core::ffi::c_int == NUL && {
                let c2rust_fresh1 = off;
                off = off - 1;
                c2rust_fresh1 <= 0 as ::core::ffi::c_int
            } {
                (*replace_stack.ptr()).size = (*replace_stack.ptr()).size.wrapping_sub(1);
                memmove(
                    (*replace_stack.ptr()).items.offset(i as isize) as *mut ::core::ffi::c_void,
                    (*replace_stack.ptr())
                        .items
                        .offset((i + 1 as ssize_t) as isize)
                        as *const ::core::ffi::c_void,
                    (*replace_stack.ptr()).size.wrapping_sub(i as size_t),
                );
                return;
            }
        }
    }
}

pub(crate) unsafe extern "C" fn replace_pop_ins() {
    unsafe {
        let mut oldState: ::core::ffi::c_int = State.get();
        State.set(MODE_NORMAL);
        while replace_pop_if_nul() > 0 as ::core::ffi::c_int {
            mb_replace_pop_ins();
            dec_cursor();
        }
        State.set(oldState);
    }
}

pub(crate) unsafe extern "C" fn mb_replace_pop_ins() {
    unsafe {
        let mut len: ::core::ffi::c_int = utf_head_off(
            (*replace_stack.ptr())
                .items
                .offset(0 as ::core::ffi::c_int as isize),
            (*replace_stack.ptr())
                .items
                .offset((*replace_stack.ptr()).size.wrapping_sub(1 as size_t) as isize),
        ) + 1 as ::core::ffi::c_int;
        (*replace_stack.ptr()).size = (*replace_stack.ptr()).size.wrapping_sub(len as size_t);
        ins_bytes_len(
            (*replace_stack.ptr())
                .items
                .offset((*replace_stack.ptr()).size as isize),
            len as size_t,
        );
    }
}

pub(crate) unsafe extern "C" fn replace_do_bs(mut limit_col: ::core::ffi::c_int) {
    unsafe {
        let mut start_vcol: colnr_T = 0;
        let l_State: ::core::ffi::c_int = State.get();
        let mut cc: ::core::ffi::c_int = replace_pop_if_nul();
        if cc > 0 as ::core::ffi::c_int {
            let mut orig_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut orig_vcols: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if l_State & VREPLACE_FLAG != 0 {
                getvcol(
                    curwin.get(),
                    &raw mut (*curwin.get()).w_cursor,
                    ::core::ptr::null_mut::<colnr_T>(),
                    &raw mut start_vcol,
                    ::core::ptr::null_mut::<colnr_T>(),
                );
                orig_vcols = win_chartabsize(curwin.get(), get_cursor_pos_ptr(), start_vcol);
            }
            del_char_after_col(limit_col);
            if l_State & VREPLACE_FLAG != 0 {
                orig_len = get_cursor_pos_len() as ::core::ffi::c_int;
            }
            replace_pop_ins();
            if l_State & VREPLACE_FLAG != 0 {
                let mut p: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                let mut ins_len: ::core::ffi::c_int = get_cursor_pos_len() - orig_len;
                let mut vcol: ::core::ffi::c_int = start_vcol as ::core::ffi::c_int;
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < ins_len {
                    vcol += win_chartabsize(curwin.get(), p.offset(i as isize), vcol as colnr_T);
                    i += utfc_ptr2len(p) - 1 as ::core::ffi::c_int;
                    i += 1;
                }
                vcol -= start_vcol as ::core::ffi::c_int;
                (*curwin.get()).w_cursor.col += ins_len;
                while vcol > orig_vcols && gchar_cursor() == ' ' as ::core::ffi::c_int {
                    del_char(false_0 != 0);
                    orig_vcols += 1;
                }
                (*curwin.get()).w_cursor.col -= ins_len;
            }
            changed_bytes((*curwin.get()).w_cursor.lnum, (*curwin.get()).w_cursor.col);
        } else if cc == 0 as ::core::ffi::c_int {
            del_char_after_col(limit_col);
        }
    }
}
