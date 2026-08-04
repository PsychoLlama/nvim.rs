//! The message scrollback, which `g<` and the pager page through.
//!
//! Every line [`crate::src::nvim::message::msg_puts_display`] emits is also
//! copied into a linked list of [`msgchunk_T`] chunks ([`store_sb_text`]), so
//! that the pager can scroll backwards past what the screen still holds.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn store_sb_text(
    mut sb_str: *mut *const ::core::ffi::c_char,
    mut s: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut sb_col: *mut ::core::ffi::c_int,
    mut finish: ::core::ffi::c_int,
) {
    unsafe {
        let mut mp: *mut msgchunk_T = ::core::ptr::null_mut::<msgchunk_T>();
        if do_clear_sb_text.get() as ::core::ffi::c_uint
            == SB_CLEAR_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
            || do_clear_sb_text.get() as ::core::ffi::c_uint
                == SB_CLEAR_CMDLINE_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            clear_sb_text(
                do_clear_sb_text.get() as ::core::ffi::c_uint
                    == SB_CLEAR_ALL as ::core::ffi::c_int as ::core::ffi::c_uint,
            );
            msg_sb_eol();
            if do_clear_sb_text.get() as ::core::ffi::c_uint
                == SB_CLEAR_CMDLINE_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
                && s > *sb_str
                && **sb_str as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
            {
                *sb_str = (*sb_str).offset(1);
            }
            do_clear_sb_text.set(SB_CLEAR_NONE);
        }
        if s > *sb_str {
            mp = xmalloc(
                (28 as size_t)
                    .wrapping_add(s.offset_from(*sb_str) as size_t)
                    .wrapping_add(1 as size_t),
            ) as *mut msgchunk_T;
            (*mp).sb_eol = finish as ::core::ffi::c_char;
            (*mp).sb_msg_col = *sb_col;
            (*mp).sb_hl_id = hl_id;
            memcpy(
                &raw mut (*mp).sb_text as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                *sb_str as *const ::core::ffi::c_void,
                s.offset_from(*sb_str) as size_t,
            );
            *(&raw mut (*mp).sb_text as *mut ::core::ffi::c_char)
                .offset(s.offset_from(*sb_str) as isize) = NUL as ::core::ffi::c_char;
            if (*last_msgchunk.ptr()).is_null() {
                last_msgchunk.set(mp);
                (*mp).sb_prev = ::core::ptr::null_mut::<msgchunk_T>();
            } else {
                (*mp).sb_prev = last_msgchunk.get();
                (*last_msgchunk.get()).sb_next = mp;
                last_msgchunk.set(mp);
            }
            (*mp).sb_next = ::core::ptr::null_mut::<msgchunk_T>();
        } else if finish != 0 && !(*last_msgchunk.ptr()).is_null() {
            (*last_msgchunk.get()).sb_eol = true_0 as ::core::ffi::c_char;
        }
        *sb_str = s;
        *sb_col = 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn may_clear_sb_text() {
    unsafe {
        msg_ext_ui_flush();
        do_clear_sb_text.set(SB_CLEAR_ALL);
        do_clear_hist_temp.set(true_0 != 0);
    }
}

pub unsafe extern "C" fn sb_text_start_cmdline() {
    unsafe {
        if do_clear_sb_text.get() as ::core::ffi::c_uint
            == SB_CLEAR_CMDLINE_BUSY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            sb_text_restart_cmdline();
        } else {
            msg_sb_eol();
            do_clear_sb_text.set(SB_CLEAR_CMDLINE_BUSY);
        };
    }
}

pub unsafe extern "C" fn sb_text_restart_cmdline() {
    unsafe {
        do_clear_sb_text.set(SB_CLEAR_CMDLINE_BUSY);
        if (*last_msgchunk.ptr()).is_null()
            || (*last_msgchunk.get()).sb_eol as ::core::ffi::c_int != 0
        {
            return;
        }
        let mut tofree: *mut msgchunk_T = msg_sb_start(last_msgchunk.get());
        last_msgchunk.set((*tofree).sb_prev);
        if !(*last_msgchunk.ptr()).is_null() {
            (*last_msgchunk.get()).sb_next = ::core::ptr::null_mut::<msgchunk_T>();
        }
        while !tofree.is_null() {
            let mut tofree_next: *mut msgchunk_T = (*tofree).sb_next;
            xfree(tofree as *mut ::core::ffi::c_void);
            tofree = tofree_next;
        }
    }
}

pub unsafe extern "C" fn sb_text_end_cmdline() {
    do_clear_sb_text.set(SB_CLEAR_CMDLINE_DONE);
}

pub unsafe extern "C" fn clear_sb_text(mut all: bool) {
    unsafe {
        let mut mp: *mut msgchunk_T = ::core::ptr::null_mut::<msgchunk_T>();
        let mut lastp: *mut *mut msgchunk_T = ::core::ptr::null_mut::<*mut msgchunk_T>();
        if all {
            lastp = last_msgchunk.ptr();
        } else {
            if (*last_msgchunk.ptr()).is_null() {
                return;
            }
            lastp = &raw mut (*(msg_sb_start
                as unsafe extern "C" fn(*mut msgchunk_T) -> *mut msgchunk_T)(
                last_msgchunk.get()
            ))
            .sb_prev;
        }
        while !(*lastp).is_null() {
            mp = (**lastp).sb_prev;
            xfree(*lastp as *mut ::core::ffi::c_void);
            *lastp = mp;
        }
    }
}

pub unsafe extern "C" fn show_sb_text() {
    unsafe {
        if ui_has(kUIMessages) {
            let mut ea: exarg_T = exarg {
                arg: b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                arglens: ::core::ptr::null_mut::<size_t>(),
                argc: 0,
                nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmdidx: CMD_append,
                argt: 0,
                skip: true_0,
                forceit: 0,
                addr_count: 0,
                line1: 0,
                line2: 0,
                addr_type: ADDR_LINES,
                flags: 0,
                do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                do_ecmd_lnum: 0,
                append: 0,
                usefilter: 0,
                amount: 0,
                regname: 0,
                force_bin: 0,
                read_edit: 0,
                mkdir_p: 0,
                force_ff: 0,
                force_enc: 0,
                bad_char: 0,
                useridx: 0,
                errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ea_getline: None,
                cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                cstack: ::core::ptr::null_mut::<cstack_T>(),
            };
            ex_messages(&raw mut ea);
            return;
        }
        let mut mp: *mut msgchunk_T = msg_sb_start(last_msgchunk.get());
        if mp.is_null() || (*mp).sb_prev.is_null() {
            vim_beep(kOptBoFlagMess as ::core::ffi::c_int as ::core::ffi::c_uint);
        } else {
            do_more_prompt('G' as ::core::ffi::c_int);
            wait_return(false_0);
        };
    }
}

pub(crate) unsafe extern "C" fn msg_sb_start(mut mps: *mut msgchunk_T) -> *mut msgchunk_T {
    unsafe {
        let mut mp: *mut msgchunk_T = mps;
        while !mp.is_null() && !(*mp).sb_prev.is_null() && (*(*mp).sb_prev).sb_eol == 0 {
            mp = (*mp).sb_prev;
        }
        return mp;
    }
}

pub unsafe extern "C" fn msg_sb_eol() {
    unsafe {
        if !(*last_msgchunk.ptr()).is_null() {
            (*last_msgchunk.get()).sb_eol = true_0 as ::core::ffi::c_char;
        }
    }
}

pub(crate) unsafe extern "C" fn disp_sb_line(
    mut row: ::core::ffi::c_int,
    mut smp: *mut msgchunk_T,
) -> *mut msgchunk_T {
    unsafe {
        let mut mp: *mut msgchunk_T = smp;
        loop {
            msg_row.set(row);
            msg_col.set((*mp).sb_msg_col);
            let mut p: *mut ::core::ffi::c_char =
                &raw mut (*mp).sb_text as *mut ::core::ffi::c_char;
            msg_puts_display(p, -1 as ::core::ffi::c_int, (*mp).sb_hl_id, true_0);
            if (*mp).sb_eol as ::core::ffi::c_int != 0 || (*mp).sb_next.is_null() {
                break;
            }
            mp = (*mp).sb_next;
        }
        return (*mp).sb_next;
    }
}
