//! Writing a value to a file -- `writefile()`.
//!
//! `f_writefile` parses the flags (`a` append, `b` binary, `s`/`S` fsync or
//! not, `p` create parent directories, `D` delete the file when the calling
//! function returns) and then hands the List or Blob to `write_list`,
//! `write_blob` or `write_string`, which do the buffered `file_write` calls
//! and turn a NL inside a string back into the NUL it stood for.
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{
    NUL, e_error_while_writing_str, false_0, kFileAppend, kFileCreate, kFileMkDir, kFileTruncate,
    true_0,
};
use crate::semsg_c;
use crate::src::nvim::eval::typval::tv_blob_len;
use crate::src::nvim::eval::typval::{
    tv_check_str_or_nr, tv_get_string_buf_chk, tv_get_string_chk,
};
use crate::src::nvim::eval::userfunc::{add_defer, can_add_defer};
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::main::{current_sctx, e_invarg2, p_fs};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::fileio::{file_close, file_flush, file_open, file_write};
use crate::src::nvim::os::libc::{gettext, strlen};
use crate::src::nvim::path::FullName_save;
use crate::src::nvim::runtime::script_is_lua;
use crate::src::nvim::types::{
    EvalFuncData, FileDescriptor, VAR_BLOB, VAR_LIST, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED,
    blob_T, list_T, listitem_T, ptrdiff_t, size_t, typval_T, typval_vval_union, varnumber_T,
};

unsafe extern "C" fn write_list(
    fp: *mut FileDescriptor,
    list: *const list_T,
    binary: bool,
) -> bool {
    unsafe {
        let mut error: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let l_: *const list_T = list;
        '_write_list_error: {
            's_131: {
                if !l_.is_null() {
                    let mut li: *const listitem_T = (*l_).lv_first;
                    loop {
                        if li.is_null() {
                            break 's_131;
                        }
                        let s: *const ::core::ffi::c_char =
                            tv_get_string_chk(&raw const (*li).li_tv);
                        if s.is_null() {
                            return false;
                        }
                        let mut hunk_start: *const ::core::ffi::c_char = s;
                        let mut p: *const ::core::ffi::c_char = hunk_start;
                        loop {
                            if *p as ::core::ffi::c_int == '\0' as ::core::ffi::c_int
                                || *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                            {
                                if p != hunk_start {
                                    let written: ptrdiff_t = file_write(
                                        fp,
                                        hunk_start,
                                        p.offset_from(hunk_start) as size_t,
                                    );
                                    if written < 0 as ptrdiff_t {
                                        error = written as ::core::ffi::c_int;
                                        break '_write_list_error;
                                    }
                                }
                                if *p as ::core::ffi::c_int == '\0' as ::core::ffi::c_int {
                                    break;
                                }
                                hunk_start = p.offset(1 as ::core::ffi::c_int as isize);
                                let mut c2rust_lvalue: [::core::ffi::c_char; 1] =
                                    ['\0' as ::core::ffi::c_char];
                                let written_0: ptrdiff_t = file_write(
                                    fp,
                                    &raw mut c2rust_lvalue as *mut ::core::ffi::c_char,
                                    1 as size_t,
                                );
                                if written_0 < 0 as ptrdiff_t {
                                    error = written_0 as ::core::ffi::c_int;
                                    break;
                                }
                            }
                            p = p.offset(1);
                        }
                        if !binary || !(*li).li_next.is_null() {
                            let written_1: ptrdiff_t = file_write(fp, c"\n".as_ptr(), 1 as size_t);
                            if written_1 < 0 as ptrdiff_t {
                                error = written_1 as ::core::ffi::c_int;
                                break '_write_list_error;
                            }
                        }
                        li = (*li).li_next;
                    }
                }
            }
            error = file_flush(fp);
            if error == 0 as ::core::ffi::c_int {
                return true_0 != 0;
            }
        }
        semsg_c!(
            gettext(e_error_while_writing_str.as_ptr()),
            uv_strerror(error),
        );
        return false_0 != 0;
    }
}

unsafe extern "C" fn write_data(
    fp: *mut FileDescriptor,
    data: *const ::core::ffi::c_char,
    len: size_t,
) -> bool {
    unsafe {
        let mut error: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        '_write_blob_error: {
            if len > 0 as size_t {
                let written: ptrdiff_t = file_write(fp, data, len);
                if written < len as ptrdiff_t {
                    error = written as ::core::ffi::c_int;
                    break '_write_blob_error;
                }
            }
            error = file_flush(fp);
            if error == 0 as ::core::ffi::c_int {
                return true_0 != 0;
            }
        }
        semsg_c!(
            gettext(e_error_while_writing_str.as_ptr()),
            uv_strerror(error),
        );
        return false_0 != 0;
    }
}

unsafe extern "C" fn write_blob(fp: *mut FileDescriptor, blob: *const blob_T) -> bool {
    unsafe {
        return write_data(
            fp,
            (*blob).bv_ga.ga_data as *const ::core::ffi::c_char,
            tv_blob_len(blob) as size_t,
        );
    }
}

unsafe extern "C" fn write_string(
    fp: *mut FileDescriptor,
    data: *const ::core::ffi::c_char,
) -> bool {
    unsafe {
        return write_data(fp, data, strlen(data));
    }
}

pub unsafe extern "C" fn f_writefile(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = -1 as varnumber_T;
        if check_secure() {
            return;
        }
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let l_: *const list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list;
            if !l_.is_null() {
                let mut li: *const listitem_T = (*l_).lv_first;
                while !li.is_null() {
                    if !tv_check_str_or_nr(&raw const (*li).li_tv) {
                        return;
                    }
                    li = (*li).li_next;
                }
            }
        } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
            && !((*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                && script_is_lua((*current_sctx.ptr()).sc_sid) as ::core::ffi::c_int != 0)
        {
            semsg_c!(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                gettext(c"writefile() first argument must be a List or a Blob".as_ptr(),),
            );
            return;
        }
        let mut binary: bool = false_0 != 0;
        let mut append: bool = false_0 != 0;
        let mut defer: bool = false_0 != 0;
        let mut do_fsync: bool = p_fs.get() != 0;
        let mut mkdir_p: bool = false_0 != 0;
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let flags: *const ::core::ffi::c_char =
                tv_get_string_chk(argvars.offset(2 as ::core::ffi::c_int as isize));
            if flags.is_null() {
                return;
            }
            let mut p: *const ::core::ffi::c_char = flags;
            while *p != 0 {
                match *p as ::core::ffi::c_int {
                    98 => {
                        binary = true_0 != 0;
                    }
                    97 => {
                        append = true_0 != 0;
                    }
                    68 => {
                        defer = true_0 != 0;
                    }
                    115 => {
                        do_fsync = true_0 != 0;
                    }
                    83 => {
                        do_fsync = false_0 != 0;
                    }
                    112 => {
                        mkdir_p = true_0 != 0;
                    }
                    _ => {
                        semsg_c!(gettext(c"E5060: Unknown flag: %s".as_ptr()), p,);
                        return;
                    }
                }
                p = p.offset(1);
            }
        }
        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
        let fname: *const ::core::ffi::c_char = tv_get_string_buf_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if fname.is_null() {
            return;
        }
        if defer as ::core::ffi::c_int != 0 && !can_add_defer() {
            return;
        }
        let mut fp: FileDescriptor = FileDescriptor {
            fd: 0,
            buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            wr: false,
            eof: false,
            non_blocking: false,
            bytes_read: 0,
        };
        let mut error: ::core::ffi::c_int = 0;
        if *fname as ::core::ffi::c_int == NUL {
            emsg(gettext(
                c"E482: Can't open file with an empty name".as_ptr(),
            ));
        } else {
            error = file_open(
                &raw mut fp,
                fname,
                (if append as ::core::ffi::c_int != 0 {
                    kFileAppend as ::core::ffi::c_int
                } else {
                    kFileTruncate as ::core::ffi::c_int
                }) | (if mkdir_p as ::core::ffi::c_int != 0 {
                    kFileMkDir as ::core::ffi::c_int
                } else {
                    kFileCreate as ::core::ffi::c_int
                }) | kFileCreate as ::core::ffi::c_int,
                0o666 as ::core::ffi::c_int,
            );
            if error != 0 as ::core::ffi::c_int {
                semsg_c!(
                    gettext(c"E482: Can't open file %s for writing: %s".as_ptr()),
                    fname,
                    uv_strerror(error),
                );
            } else {
                if defer {
                    let mut tv: typval_T = typval_T {
                        v_type: VAR_STRING,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union {
                            v_string: FullName_save(fname, false_0 != 0),
                        },
                    };
                    add_defer(
                        c"delete".as_ptr() as *mut ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                        &raw mut tv,
                    );
                }
                let mut write_ok: bool = false;
                if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    write_ok = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                        .vval
                        .v_blob
                        .is_null()
                        || write_blob(
                            &raw mut fp,
                            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                                .vval
                                .v_blob,
                        ) as ::core::ffi::c_int
                            != 0;
                } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type
                    as ::core::ffi::c_uint
                    == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    write_ok = write_string(
                        &raw mut fp,
                        (*argvars.offset(0 as ::core::ffi::c_int as isize))
                            .vval
                            .v_string,
                    );
                } else {
                    write_ok = write_list(
                        &raw mut fp,
                        (*argvars.offset(0 as ::core::ffi::c_int as isize))
                            .vval
                            .v_list,
                        binary,
                    );
                }
                if write_ok {
                    (*rettv).vval.v_number = 0 as varnumber_T;
                }
                error = file_close(&raw mut fp, do_fsync);
                if error != 0 as ::core::ffi::c_int {
                    semsg_c!(
                        gettext(c"E80: Error when closing file %s: %s".as_ptr()),
                        fname,
                        uv_strerror(error),
                    );
                }
            }
        };
    }
}
