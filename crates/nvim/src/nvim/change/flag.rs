//! Whether the buffer counts as modified.
//!
//! `changed` is the front door every edit goes through: it flips `b_changed`,
//! warns once about a 'readonly' file (after giving FileChangedRO a chance to
//! clear it), makes sure a swap file exists, and bumps `b:changedtick`.
//! `unchanged` is the other direction -- `:w` and `:e!` -- and `save_file_ff` /
//! `file_ff_differs` are the pair that remembers a buffer's 'fileformat',
//! 'fileencoding' and BOM at load time so that `'cpo'`-`+` can tell a real
//! change from one the reader made.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn change_warning(mut buf: *mut buf_T, mut col: ::core::ffi::c_int) {
    unsafe {
        static w_readonly: GlobalCell<*const ::core::ffi::c_char> =
            GlobalCell::new(c"W10: Warning: Changing a readonly file".as_ptr());
        if (*buf).b_did_warn as ::core::ffi::c_int == false_0
            && curbufIsChanged() as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            && !autocmd_busy.get()
            && (*buf).b_p_ro != 0
        {
            (*buf).b_ro_locked += 1;
            apply_autocmds(
                EVENT_FILECHANGEDRO,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false,
                buf,
            );
            (*buf).b_ro_locked -= 1;
            if (*buf).b_p_ro == 0 {
                return;
            }
            msg_start();
            if msg_row.get() == Rows.get() - 1 as ::core::ffi::c_int {
                msg_col.set(col);
            }
            msg_source(HLF_W);
            msg_ext_set_kind(c"wmsg".as_ptr());
            msg_puts_hl(gettext(w_readonly.get()), HLF_W, true);
            set_vim_var_string(VV_WARNINGMSG, gettext(w_readonly.get()), -1 as ptrdiff_t);
            msg_clr_eos();
            msg_end();
            if msg_silent.get() == 0 as ::core::ffi::c_int && !silent_mode.get() && ui_active() != 0
            {
                msg_delay(1002 as uint64_t, true);
            }
            (*buf).b_did_warn = true;
            redraw_cmdline.set(false);
            if msg_row.get() < Rows.get() - 1 as ::core::ffi::c_int {
                showmode();
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn changed(mut buf: *mut buf_T) {
    unsafe {
        if (*buf).b_changed == 0 {
            let mut save_msg_scroll: ::core::ffi::c_int = msg_scroll.get();
            change_warning(buf, 0 as ::core::ffi::c_int);
            if (*buf).b_may_swap as ::core::ffi::c_int != 0 && !bt_dontwrite(buf) {
                let mut save_need_wait_return: bool = need_wait_return.get();
                need_wait_return.set(false);
                ml_open_file(buf);
                if need_wait_return.get() as ::core::ffi::c_int != 0
                    && emsg_silent.get() == 0 as ::core::ffi::c_int
                    && !in_assert_fails.get()
                    && !ui_has(kUIMessages)
                {
                    msg_delay(2002 as uint64_t, true);
                    wait_return(true_0);
                    msg_scroll.set(save_msg_scroll);
                } else {
                    need_wait_return.set(save_need_wait_return);
                }
            }
            changed_internal(buf);
        }
        buf_inc_changedtick(buf);
        highlight_match.set(false);
    }
}

pub unsafe extern "C" fn changed_internal(mut buf: *mut buf_T) {
    unsafe {
        (*buf).b_changed = true_0;
        (*buf).b_changed_invalid = true;
        ml_setflags(buf);
        redraw_buf_status_later(buf);
        redraw_tabline.set(true);
        need_maketitle.set(true);
    }
}

pub unsafe extern "C" fn unchanged(
    mut buf: *mut buf_T,
    mut ff: bool,
    mut always_inc_changedtick: bool,
) {
    unsafe {
        if (*buf).b_changed != 0
            || ff as ::core::ffi::c_int != 0
                && file_ff_differs(buf, false) as ::core::ffi::c_int != 0
        {
            (*buf).b_changed = false_0;
            (*buf).b_changed_invalid = true;
            ml_setflags(buf);
            if ff {
                save_file_ff(buf);
            }
            redraw_buf_status_later(buf);
            redraw_tabline.set(true);
            need_maketitle.set(true);
            buf_inc_changedtick(buf);
        } else if always_inc_changedtick {
            buf_inc_changedtick(buf);
        }
    }
}

pub unsafe extern "C" fn save_file_ff(mut buf: *mut buf_T) {
    unsafe {
        (*buf).b_start_ffc = *(*buf).b_p_ff as ::core::ffi::c_uchar as ::core::ffi::c_int;
        (*buf).b_start_eof = (*buf).b_p_eof;
        (*buf).b_start_eol = (*buf).b_p_eol;
        (*buf).b_start_bomb = (*buf).b_p_bomb;
        if (*buf).b_start_fenc.is_null()
            || strcmp((*buf).b_start_fenc, (*buf).b_p_fenc) != 0 as ::core::ffi::c_int
        {
            xfree((*buf).b_start_fenc as *mut ::core::ffi::c_void);
            (*buf).b_start_fenc = xstrdup((*buf).b_p_fenc);
        }
    }
}

pub unsafe extern "C" fn file_ff_differs(mut buf: *mut buf_T, mut ignore_empty: bool) -> bool {
    unsafe {
        if (*buf).b_flags & BF_NEVERLOADED != 0 {
            return false;
        }
        if ignore_empty as ::core::ffi::c_int != 0
            && (*buf).b_flags & BF_NEW != 0
            && (*buf).b_ml.ml_line_count == 1 as linenr_T
            && *ml_get_buf(buf, 1 as linenr_T) as ::core::ffi::c_int == NUL
        {
            return false;
        }
        if (*buf).b_start_ffc != *(*buf).b_p_ff as ::core::ffi::c_int {
            return true;
        }
        if ((*buf).b_p_bin != 0 || (*buf).b_p_fixeol == 0)
            && ((*buf).b_start_eof != (*buf).b_p_eof || (*buf).b_start_eol != (*buf).b_p_eol)
        {
            return true;
        }
        if (*buf).b_p_bin == 0 && (*buf).b_start_bomb != (*buf).b_p_bomb {
            return true;
        }
        if (*buf).b_start_fenc.is_null() {
            return *(*buf).b_p_fenc as ::core::ffi::c_int != NUL;
        }
        return strcmp((*buf).b_start_fenc, (*buf).b_p_fenc) != 0 as ::core::ffi::c_int;
    }
}
