//! `-s` script input: reading keys from a file.
//!
//! [`openscript`] pushes a file onto the `scriptin` stack (up to `NSCRIPT`
//! deep) and `inchar` reads a byte at a time from the innermost one until
//! EOF, when [`closescript`] pops it.  [`updatescript`] is the other
//! direction: the `'scriptout'` copy of what was typed.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn openscript(mut name: *mut ::core::ffi::c_char, mut directly: bool) {
    unsafe {
        if curscript.get() + 1 as ::core::ffi::c_int == NSCRIPT as ::core::ffi::c_int {
            emsg(gettext(&raw const e_nesting as *const ::core::ffi::c_char));
            return;
        }
        if check_secure() {
            return;
        }
        if ignore_script.get() {
            return;
        }
        (*curscript.ptr()) += 1;
        expand_env(name, NameBuff.ptr() as *mut ::core::ffi::c_char, MAXPATHL);
        let mut error: ::core::ffi::c_int = file_open(
            (scriptin.ptr() as *mut FileDescriptor).offset(curscript.get() as isize),
            NameBuff.ptr() as *mut ::core::ffi::c_char,
            kFileReadOnly as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        if error != 0 {
            semsg(
                gettext(&raw const e_notopen_2 as *const ::core::ffi::c_char),
                name,
                uv_strerror(error),
            );
            (*curscript.ptr()) -= 1;
            return;
        }
        save_typebuf();
        if directly {
            let mut oa: oparg_T = oparg_T {
                op_type: 0,
                regname: 0,
                motion_type: kMTCharWise,
                motion_force: 0,
                use_reg_one: false,
                inclusive: false,
                end_adjusted: false,
                start: pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
                end: pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
                cursor_start: pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
                line_count: 0,
                empty: false,
                is_VIsual: false,
                start_vcol: 0,
                end_vcol: 0,
                prev_opcount: 0,
                prev_count0: 0,
                excl_tr_ws: false,
            };
            let mut save_State: ::core::ffi::c_int = State.get();
            let mut save_restart_edit: ::core::ffi::c_int = restart_edit.get();
            let mut save_finish_op: ::core::ffi::c_int = finish_op.get() as ::core::ffi::c_int;
            let mut save_msg_scroll: ::core::ffi::c_int = msg_scroll.get();
            State.set(MODE_NORMAL);
            msg_scroll.set(false_0);
            restart_edit.set(0 as ::core::ffi::c_int);
            clear_oparg(&raw mut oa);
            finish_op.set(false_0 != 0);
            let mut oldcurscript: ::core::ffi::c_int = curscript.get();
            loop {
                update_topline_cursor();
                normal_cmd(&raw mut oa, false_0 != 0);
                vpeekc();
                if curscript.get() < oldcurscript {
                    break;
                }
            }
            State.set(save_State);
            msg_scroll.set(save_msg_scroll);
            restart_edit.set(save_restart_edit);
            finish_op.set(save_finish_op != 0);
        }
    }
}

pub(crate) unsafe extern "C" fn closescript() {
    unsafe {
        '_c2rust_label: {
            if curscript.get() >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"curscript >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1450 as ::core::ffi::c_uint,
                    b"void closescript(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        free_typebuf();
        typebuf.set((*saved_typebuf.ptr())[curscript.get() as usize]);
        file_close(
            (scriptin.ptr() as *mut FileDescriptor).offset(curscript.get() as isize),
            false_0 != 0,
        );
        (*curscript.ptr()) -= 1;
    }
}

pub unsafe extern "C" fn open_scriptin(mut scriptin_name: *mut ::core::ffi::c_char) -> bool {
    unsafe {
        '_c2rust_label: {
            if curscript.get() == -1 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"curscript == -1\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1471 as ::core::ffi::c_uint,
                    b"_Bool open_scriptin(char *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        (*curscript.ptr()) += 1;
        let mut error: ::core::ffi::c_int = 0;
        if strequal(scriptin_name, b"-\0".as_ptr() as *const ::core::ffi::c_char) {
            error = file_open_stdin(
                (scriptin.ptr() as *mut FileDescriptor).offset(0 as ::core::ffi::c_int as isize),
            );
        } else {
            error = file_open(
                (scriptin.ptr() as *mut FileDescriptor).offset(0 as ::core::ffi::c_int as isize),
                scriptin_name,
                kFileReadOnly as ::core::ffi::c_int | kFileNonBlocking as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        }
        if error != 0 {
            fprintf(
                stderr,
                gettext(b"Cannot open for reading: \"%s\": %s\n\0".as_ptr()
                    as *const ::core::ffi::c_char),
                scriptin_name,
                uv_strerror(error),
            );
            (*curscript.ptr()) -= 1;
            return false_0 != 0;
        }
        save_typebuf();
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn using_script() -> ::core::ffi::c_int {
    return (curscript.get() >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}

pub unsafe extern "C" fn before_blocking() {
    unsafe {
        updatescript(0 as ::core::ffi::c_int);
        if may_garbage_collect.get() {
            garbage_collect(false_0 != 0);
        }
    }
}

pub(crate) unsafe extern "C" fn updatescript(mut c: ::core::ffi::c_int) {
    unsafe {
        static count: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        if c != 0 && !(*scriptout.ptr()).is_null() {
            putc(c, scriptout.get());
        }
        let mut idle: bool = c == 0 as ::core::ffi::c_int;
        if idle as ::core::ffi::c_int != 0
            || p_uc.get() > 0 as OptInt && {
                (*count.ptr()) += 1;
                count.get() as OptInt >= p_uc.get()
            }
        {
            ml_sync_all(
                idle as ::core::ffi::c_int,
                true_0,
                p_fs.get() != 0 || idle as ::core::ffi::c_int != 0,
            );
            count.set(0 as ::core::ffi::c_int);
        }
    }
}
