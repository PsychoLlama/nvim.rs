//! What kind of buffer is this -- the `'buftype'` predicates.
//!
//! The `bt_*` family answers `'buftype'` questions the rest of the editor
//! asks constantly -- is this a help buffer, a quickfix list, a terminal, a
//! prompt; does it have a file name; may it be written -- and
//! [`buf_spname`] gives the special buffers the name that is displayed
//! instead of a file.  [`buf_hide`] is the `'hidden'`/`'bufhidden'` decision,
//! [`set_buflisted`] the `'buflisted'` half, and the `changedtick` pair the
//! `b:changedtick` counter every change bumps.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::autocmd::{EVENT_BUFADD, EVENT_BUFDELETE, apply_autocmds};
use crate::src::nvim::eval::typval::tv_dict_is_watched;
use crate::src::nvim::eval::typval::{tv_dict_find, tv_dict_watcher_notify};
use crate::src::nvim::main::{cmdmod, cmdwin_buf, curbuf, msg_loclist, msg_qflist, p_hid};
use crate::src::nvim::memline::ml_get_buf;
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::quickfix::qf_stack_get_bufnr;
use crate::src::nvim::types::{
    CMOD_HIDE, VAR_FIXED, VAR_NUMBER, buf_T, dictitem_T, linenr_T, ptrdiff_t, typval_T, varnumber_T,
};

pub unsafe extern "C" fn bt_prompt(mut buf: *mut buf_T) -> bool {
    unsafe {
        return !buf.is_null()
            && *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'p' as ::core::ffi::c_int;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bt_help(buf: *const buf_T) -> bool {
    unsafe {
        return !buf.is_null() && (*buf).b_help as ::core::ffi::c_int != 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bt_normal(buf: *const buf_T) -> bool {
    unsafe {
        return !buf.is_null()
            && *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == NUL;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bt_quickfix(buf: *const buf_T) -> bool {
    unsafe {
        return !buf.is_null()
            && *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'q' as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn bt_terminal(buf: *const buf_T) -> bool {
    unsafe {
        return !buf.is_null()
            && *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 't' as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn bt_nofilename(buf: *const buf_T) -> bool {
    unsafe {
        return !buf.is_null()
            && (*(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'n' as ::core::ffi::c_int
                && *(*buf).b_p_bt.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'f' as ::core::ffi::c_int
                || *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'a' as ::core::ffi::c_int
                || !(*buf).terminal.is_null()
                || *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'p' as ::core::ffi::c_int);
    }
}

pub(crate) unsafe extern "C" fn bt_nofileread(buf: *const buf_T) -> bool {
    unsafe {
        return !buf.is_null()
            && (*(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'n' as ::core::ffi::c_int
                && *(*buf).b_p_bt.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'f' as ::core::ffi::c_int
                || *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 't' as ::core::ffi::c_int
                || *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'q' as ::core::ffi::c_int
                || *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'p' as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn bt_nofile(buf: *const buf_T) -> bool {
    unsafe {
        return !buf.is_null()
            && *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'n' as ::core::ffi::c_int
            && *(*buf).b_p_bt.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'f' as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn bt_dontwrite(buf: *const buf_T) -> bool {
    unsafe {
        return !buf.is_null()
            && (*(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'n' as ::core::ffi::c_int
                || !(*buf).terminal.is_null()
                || *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'p' as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn bt_dontwrite_msg(buf: *const buf_T) -> bool {
    unsafe {
        if bt_dontwrite(buf) {
            emsg(gettext(
                c"E382: Cannot write, 'buftype' option is set".as_ptr(),
            ));
            return true_0 != 0;
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn buf_hide(buf: *const buf_T) -> bool {
    unsafe {
        match *(*buf).b_p_bh.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            117 | 119 | 100 => return false_0 != 0,
            104 => return true_0 != 0,
            _ => {}
        }
        return p_hid.get() != 0
            || (*cmdmod.ptr()).cmod_flags & CMOD_HIDE as ::core::ffi::c_int != 0;
    }
}

pub unsafe extern "C" fn buf_spname(mut buf: *mut buf_T) -> *mut ::core::ffi::c_char {
    unsafe {
        if bt_quickfix(buf) {
            if (*buf).handle == qf_stack_get_bufnr() {
                return gettext(msg_qflist.get());
            }
            return gettext(msg_loclist.get());
        }
        if bt_nofilename(buf) {
            if !(*buf).b_fname.is_null() {
                return (*buf).b_fname;
            }
            if buf == cmdwin_buf.get() {
                return gettext(c"[Command Line]".as_ptr());
            }
            if bt_prompt(buf) {
                return gettext(c"[Prompt]".as_ptr());
            }
            return gettext(c"[Scratch]".as_ptr());
        }
        if (*buf).b_fname.is_null() {
            return buf_get_fname(buf);
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn buf_get_fname(mut buf: *const buf_T) -> *mut ::core::ffi::c_char {
    unsafe {
        if (*buf).b_fname.is_null() {
            return gettext(c"[No Name]".as_ptr());
        }
        return (*buf).b_fname;
    }
}

pub unsafe extern "C" fn set_buflisted(mut on: ::core::ffi::c_int) {
    unsafe {
        if on == (*curbuf.get()).b_p_bl {
            return;
        }
        (*curbuf.get()).b_p_bl = on;
        if on != 0 {
            apply_autocmds(
                EVENT_BUFADD,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        } else {
            apply_autocmds(
                EVENT_BUFDELETE,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        };
    }
}

pub unsafe extern "C" fn buf_is_empty(mut buf: *mut buf_T) -> bool {
    unsafe {
        return (*buf).b_ml.ml_line_count == 1 as linenr_T
            && *ml_get_buf(buf, 1 as linenr_T) as ::core::ffi::c_int == NUL;
    }
}

pub unsafe extern "C" fn buf_inc_changedtick(buf: *mut buf_T) {
    unsafe {
        buf_set_changedtick(buf, buf_get_changedtick(buf) + 1 as varnumber_T);
    }
}

pub unsafe extern "C" fn buf_set_changedtick(buf: *mut buf_T, changedtick: varnumber_T) {
    unsafe {
        let mut old_val: typval_T = (*buf).changedtick_di.di_tv;
        let changedtick_di: *mut dictitem_T = tv_dict_find(
            (*buf).b_vars,
            c"changedtick".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1_usize) as ptrdiff_t,
        );
        debug_assert!(!changedtick_di.is_null(), "changedtick_di != NULL");
        assert!(
            (*changedtick_di).di_tv.v_type as ::core::ffi::c_uint
                == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint,
            "changedtick_di->di_tv.v_type == VAR_NUMBER"
        );
        assert!(
            (*changedtick_di).di_tv.v_lock as ::core::ffi::c_uint
                == VAR_FIXED as ::core::ffi::c_int as ::core::ffi::c_uint,
            "changedtick_di->di_tv.v_lock == VAR_FIXED"
        );
        assert!(
            (*changedtick_di).di_flags as ::core::ffi::c_int
                == DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int,
            "changedtick_di->di_flags == (DI_FLAGS_RO|DI_FLAGS_FIX)"
        );
        debug_assert!(
            changedtick_di == &raw mut (*buf).changedtick_di as *mut dictitem_T,
            "changedtick_di == (dictitem_T *)&buf->changedtick_di"
        );
        (*buf).changedtick_di.di_tv.vval.v_number = changedtick;
        if tv_dict_is_watched((*buf).b_vars) {
            (*buf).b_locked += 1;
            tv_dict_watcher_notify(
                (*buf).b_vars,
                &raw mut (*buf).changedtick_di.di_key as *mut ::core::ffi::c_char,
                &raw mut (*buf).changedtick_di.di_tv,
                &raw mut old_val,
            );
            (*buf).b_locked -= 1;
        }
    }
}
