//! The redo buffer: what `.` replays.
//!
//! Normal-mode commands append themselves to `redobuff` as they run
//! ([`AppendToRedobuff`] and friends); `.` calls [`start_redo`], which copies
//! that buffer into the read buffer so the keys are re-read as if stuffed.
//! `old_redobuff` keeps the previous one so an aborted redo can be undone.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn ResetRedobuff() {
    unsafe {
        if block_redo.get() {
            return;
        }
        free_buff(old_redobuff.ptr());
        old_redobuff.set(redobuff.get());
        (*redobuff.ptr()).bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
    }
}

pub unsafe extern "C" fn CancelRedo() {
    unsafe {
        if block_redo.get() {
            return;
        }
        free_buff(redobuff.ptr());
        redobuff.set(old_redobuff.get());
        (*old_redobuff.ptr()).bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
        start_stuff();
        while read_readbuffers(true_0 != 0) != NUL {}
    }
}

pub unsafe extern "C" fn saveRedobuff(mut save_redo: *mut save_redo_T) {
    unsafe {
        (*save_redo).sr_redobuff = redobuff.get();
        (*redobuff.ptr()).bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
        (*save_redo).sr_old_redobuff = old_redobuff.get();
        (*old_redobuff.ptr()).bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
        let mut slen: size_t = 0;
        let s: *mut ::core::ffi::c_char =
            get_buffcont(&raw mut (*save_redo).sr_redobuff, false_0, &raw mut slen);
        if s.is_null() {
            return;
        }
        add_buff(redobuff.ptr(), s, slen as ptrdiff_t);
        xfree(s as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn restoreRedobuff(mut save_redo: *mut save_redo_T) {
    unsafe {
        free_buff(redobuff.ptr());
        redobuff.set((*save_redo).sr_redobuff);
        free_buff(old_redobuff.ptr());
        old_redobuff.set((*save_redo).sr_old_redobuff);
    }
}

pub unsafe extern "C" fn AppendToRedobuff(mut s: *const ::core::ffi::c_char) {
    unsafe {
        if !block_redo.get() {
            add_buff(redobuff.ptr(), s, -1 as ptrdiff_t);
        }
    }
}

pub unsafe extern "C" fn AppendToRedobuffLit(
    mut str: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) {
    unsafe {
        if block_redo.get() {
            return;
        }
        let mut s: *const ::core::ffi::c_char = str;
        while if len < 0 as ::core::ffi::c_int {
            (*s as ::core::ffi::c_int != NUL) as ::core::ffi::c_int
        } else {
            (s.offset_from(str) < len as isize) as ::core::ffi::c_int
        } != 0
        {
            let mut start: *const ::core::ffi::c_char = s;
            while *s as ::core::ffi::c_int >= ' ' as ::core::ffi::c_int
                && (*s as ::core::ffi::c_int) < DEL
                && (len < 0 as ::core::ffi::c_int || s.offset_from(str) < len as isize)
            {
                s = s.offset(1);
            }
            if *s as ::core::ffi::c_int == NUL
                && (*s.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '0' as ::core::ffi::c_int
                    || *s.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '^' as ::core::ffi::c_int)
            {
                s = s.offset(-1);
            }
            if s > start {
                add_buff(redobuff.ptr(), start, s.offset_from(start));
            }
            if *s as ::core::ffi::c_int == NUL
                || len >= 0 as ::core::ffi::c_int && s.offset_from(str) >= len as isize
            {
                break;
            }
            let c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut s);
            if c < ' ' as ::core::ffi::c_int
                || c == DEL
                || *s as ::core::ffi::c_int == NUL
                    && (c == '0' as ::core::ffi::c_int || c == '^' as ::core::ffi::c_int)
            {
                add_char_buff(redobuff.ptr(), Ctrl_V);
            }
            if *s as ::core::ffi::c_int == NUL && c == '0' as ::core::ffi::c_int {
                add_buff(
                    redobuff.ptr(),
                    b"048\0".as_ptr() as *const ::core::ffi::c_char,
                    3 as ptrdiff_t,
                );
            } else {
                add_char_buff(redobuff.ptr(), c);
            }
        }
    }
}

pub unsafe extern "C" fn AppendToRedobuffSpec(mut s: *const ::core::ffi::c_char) {
    unsafe {
        if block_redo.get() {
            return;
        }
        while *s as ::core::ffi::c_int != NUL {
            if *s as uint8_t as ::core::ffi::c_int == K_SPECIAL
                && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                add_buff(redobuff.ptr(), s, 3 as ptrdiff_t);
                s = s.offset(3 as ::core::ffi::c_int as isize);
            } else {
                add_char_buff(redobuff.ptr(), mb_cptr2char_adv(&raw mut s));
            }
        }
    }
}

pub unsafe extern "C" fn AppendCharToRedobuff(mut c: ::core::ffi::c_int) {
    unsafe {
        if !block_redo.get() {
            add_char_buff(redobuff.ptr(), c);
        }
    }
}

pub unsafe extern "C" fn AppendNumberToRedobuff(mut n: ::core::ffi::c_int) {
    unsafe {
        if !block_redo.get() {
            add_num_buff(redobuff.ptr(), n);
        }
    }
}

pub(crate) unsafe extern "C" fn read_redo(
    mut init: bool,
    mut old_redo: bool,
) -> ::core::ffi::c_int {
    unsafe {
        static bp: GlobalCell<*mut buffblock_T> =
            GlobalCell::new(::core::ptr::null_mut::<buffblock_T>());
        static p: GlobalCell<*mut uint8_t> = GlobalCell::new(::core::ptr::null_mut::<uint8_t>());
        let mut c: ::core::ffi::c_int = 0;
        let mut n: ::core::ffi::c_int = 0;
        let mut buf: [uint8_t; 22] = [0; 22];
        if init {
            bp.set(
                (if old_redo as ::core::ffi::c_int != 0 {
                    (*old_redobuff.ptr()).bh_first.b_next
                } else {
                    (*redobuff.ptr()).bh_first.b_next
                }) as *mut buffblock_T,
            );
            if (*bp.ptr()).is_null() {
                return FAIL;
            }
            p.set(&raw mut (*bp.get()).b_str as *mut ::core::ffi::c_char as *mut uint8_t);
            return OK;
        }
        c = *p.get() as ::core::ffi::c_int;
        if c == NUL {
            return c;
        }
        if c != K_SPECIAL
            || *(*p.ptr()).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == KS_SPECIAL
        {
            n = if c < 0 as ::core::ffi::c_int || c > 255 as ::core::ffi::c_int {
                1 as ::core::ffi::c_int
            } else {
                (*utf8len_tab.ptr())[c as usize] as ::core::ffi::c_int
            };
        } else {
            n = 1 as ::core::ffi::c_int;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            if c == K_SPECIAL {
                c = if *(*p.ptr()).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == KS_SPECIAL
                {
                    K_SPECIAL
                } else if *(*p.ptr()).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == KS_ZERO
                {
                    K_ZERO
                } else {
                    -(*(*p.ptr()).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        + ((*(*p.ptr()).offset(2 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int)
                            << 8 as ::core::ffi::c_int))
                };
                p.set((*p.ptr()).offset(2 as ::core::ffi::c_int as isize));
            }
            p.set((*p.ptr()).offset(1));
            if *p.get() as ::core::ffi::c_int == NUL && !(*bp.get()).b_next.is_null() {
                bp.set((*bp.get()).b_next as *mut buffblock_T);
                p.set(&raw mut (*bp.get()).b_str as *mut ::core::ffi::c_char as *mut uint8_t);
            }
            buf[i as usize] = c as uint8_t;
            if i == n - 1 as ::core::ffi::c_int {
                if n != 1 as ::core::ffi::c_int {
                    c = utf_ptr2char(&raw mut buf as *mut uint8_t as *mut ::core::ffi::c_char);
                }
                break;
            } else {
                c = *p.get() as ::core::ffi::c_int;
                if c == NUL {
                    break;
                }
                i += 1;
            }
        }
        return c;
    }
}

pub(crate) unsafe extern "C" fn copy_redo(mut old_redo: bool) {
    unsafe {
        let mut c: ::core::ffi::c_int = 0;
        loop {
            c = read_redo(false_0 != 0, old_redo);
            if c == NUL {
                break;
            }
            add_char_buff(readbuf2.ptr(), c);
        }
    }
}

pub unsafe extern "C" fn start_redo(
    mut count: ::core::ffi::c_int,
    mut old_redo: bool,
) -> ::core::ffi::c_int {
    unsafe {
        if read_redo(true_0 != 0, old_redo) == FAIL {
            return FAIL;
        }
        let mut c: ::core::ffi::c_int = read_redo(false_0 != 0, old_redo);
        if c == '"' as ::core::ffi::c_int {
            add_buff(
                readbuf2.ptr(),
                b"\"\0".as_ptr() as *const ::core::ffi::c_char,
                1 as ptrdiff_t,
            );
            c = read_redo(false_0 != 0, old_redo);
            if c >= '1' as ::core::ffi::c_int && c < '9' as ::core::ffi::c_int {
                c += 1;
            }
            add_char_buff(readbuf2.ptr(), c);
            if c == '=' as ::core::ffi::c_int {
                add_char_buff(readbuf2.ptr(), CAR);
                cmd_silent.set(true_0 != 0);
            }
            c = read_redo(false_0 != 0, old_redo);
        }
        if c == 'v' as ::core::ffi::c_int {
            VIsual.set((*curwin.get()).w_cursor);
            VIsual_active.set(true_0 != 0);
            VIsual_select.set(false_0 != 0);
            VIsual_reselect.set(true_0);
            redo_VIsual_busy.set(true_0 != 0);
            c = read_redo(false_0 != 0, old_redo);
        }
        if count != 0 {
            while ascii_isdigit(c) {
                c = read_redo(false_0 != 0, old_redo);
            }
            add_num_buff(readbuf2.ptr(), count);
        }
        add_char_buff(readbuf2.ptr(), c);
        copy_redo(old_redo);
        return OK;
    }
}

pub unsafe extern "C" fn start_redo_ins() -> ::core::ffi::c_int {
    unsafe {
        let mut c: ::core::ffi::c_int = 0;
        if read_redo(true_0 != 0, false_0 != 0) == FAIL {
            return FAIL;
        }
        start_stuff();
        loop {
            c = read_redo(false_0 != 0, false_0 != 0);
            if c == NUL {
                break;
            }
            if vim_strchr(b"AaIiRrOo\0".as_ptr() as *const ::core::ffi::c_char, c).is_null() {
                continue;
            }
            if c == 'O' as ::core::ffi::c_int || c == 'o' as ::core::ffi::c_int {
                add_buff(readbuf2.ptr(), NL_STR.as_ptr(), -1 as ptrdiff_t);
            }
            break;
        }
        copy_redo(false_0 != 0);
        block_redo.set(true_0 != 0);
        return OK;
    }
}

pub unsafe extern "C" fn stop_redo_ins() {
    block_redo.set(false_0 != 0);
}
