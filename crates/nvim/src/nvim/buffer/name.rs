//! A buffer's name -- setting it, comparing it, and the alternate file.
//!
//! [`setfname`] gives a buffer its file name, which means resolving it to a
//! full path, computing the file id used to recognise the same file under
//! another name, and telling the alternate-file and argument lists about it.
//! [`otherfile`] and [`buf_same_file_id`] are the comparison, [`setaltfname`]
//! and [`buflist_add`] maintain the `#` entry, and [`buflist_name_nr`] is the
//! `:buffers`-style lookup by number.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::arglist::check_arg_idx;
use crate::src::nvim::drawscreen::status_redraw_all;
use crate::src::nvim::main::{cmdmod, curbuf, curtab, curwin, e_noalt, first_tabpage, firstwin};
use crate::src::nvim::mark::fmarks_check_names;
use crate::src::nvim::memline::{ml_setname, ml_timestamp};
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::fs::{os_fileid, os_fileid_equal};
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::path::{fix_fname, path_fnamecmp};
use crate::src::nvim::types::{CMOD_KEEPALT, FileID, buf_T, linenr_T, tabpage_T, win_T};

pub unsafe extern "C" fn buflist_name_nr(
    mut fnum: ::core::ffi::c_int,
    mut fname: *mut *mut ::core::ffi::c_char,
    mut lnum: *mut linenr_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut buf: *mut buf_T = buflist_findnr(fnum);
        if buf.is_null() || (*buf).b_fname.is_null() {
            return FAIL;
        }
        *fname = (*buf).b_fname;
        *lnum = buflist_findlnum(buf);
        return OK;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn setfname(
    mut buf: *mut buf_T,
    mut ffname_arg: *mut ::core::ffi::c_char,
    mut sfname_arg: *mut ::core::ffi::c_char,
    mut message: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ffname: *mut ::core::ffi::c_char = ffname_arg;
        let mut sfname: *mut ::core::ffi::c_char = sfname_arg;
        let mut obuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut file_id: FileID = FileID {
            inode: 0,
            device_id: 0,
        };
        let mut file_id_valid: bool = false_0 != 0;
        if ffname.is_null() || *ffname as ::core::ffi::c_int == NUL {
            if (*buf).b_sfname != (*buf).b_ffname {
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut (*buf).b_sfname as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL_0;
                let _ = *ptr_;
            } else {
                (*buf).b_sfname = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut (*buf).b_ffname as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL_0;
            let _ = *ptr__0;
        } else {
            fname_expand(buf, &raw mut ffname, &raw mut sfname);
            if ffname.is_null() {
                return FAIL;
            }
            file_id_valid = os_fileid(ffname, &raw mut file_id);
            if (*buf).b_flags & BF_DUMMY == 0 {
                obuf = buflist_findname_file_id(ffname, &raw mut file_id, file_id_valid);
            }
            if !obuf.is_null() && obuf != buf {
                let mut in_use: bool = false_0 != 0;
                let mut tab: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                while !tab.is_null() {
                    let mut win: *mut win_T = if tab == curtab.get() {
                        firstwin.get()
                    } else {
                        (*tab).tp_firstwin
                    };
                    while !win.is_null() {
                        if (*win).w_buffer == obuf {
                            in_use = true_0 != 0;
                        }
                        win = (*win).w_next;
                    }
                    tab = (*tab).tp_next as *mut tabpage_T;
                }
                if !(*obuf).b_ml.ml_mfp.is_null() || in_use as ::core::ffi::c_int != 0 {
                    if message {
                        emsg(gettext(
                            c"E95: Buffer with this name already exists".as_ptr(),
                        ));
                    }
                    xfree(ffname as *mut ::core::ffi::c_void);
                    return FAIL;
                }
                close_buffer(
                    ::core::ptr::null_mut::<win_T>(),
                    obuf,
                    DOBUF_WIPE as ::core::ffi::c_int,
                    false_0 != 0,
                    false_0 != 0,
                );
            }
            sfname = xstrdup(sfname);
            if (*buf).b_sfname != (*buf).b_ffname {
                xfree((*buf).b_sfname as *mut ::core::ffi::c_void);
            }
            xfree((*buf).b_ffname as *mut ::core::ffi::c_void);
            (*buf).b_ffname = ffname;
            (*buf).b_sfname = sfname;
        }
        (*buf).b_fname = (*buf).b_sfname;
        if !file_id_valid {
            (*buf).file_id_valid = false_0 != 0;
        } else {
            (*buf).file_id_valid = true_0 != 0;
            (*buf).file_id = file_id;
        }
        buf_name_changed(buf);
        return OK;
    }
}

pub unsafe extern "C" fn buf_set_name(
    mut fnum: ::core::ffi::c_int,
    mut name: *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut buf: *mut buf_T = buflist_findnr(fnum);
        if buf.is_null() {
            return;
        }
        if (*buf).b_sfname != (*buf).b_ffname {
            xfree((*buf).b_sfname as *mut ::core::ffi::c_void);
        }
        xfree((*buf).b_ffname as *mut ::core::ffi::c_void);
        (*buf).b_ffname = xstrdup(name);
        (*buf).b_sfname = ::core::ptr::null_mut::<::core::ffi::c_char>();
        fname_expand(buf, &raw mut (*buf).b_ffname, &raw mut (*buf).b_sfname);
        (*buf).b_fname = (*buf).b_sfname;
    }
}

pub unsafe extern "C" fn buf_name_changed(mut buf: *mut buf_T) {
    unsafe {
        if !(*buf).b_ml.ml_mfp.is_null() {
            ml_setname(buf);
        }
        if (*curwin.get()).w_buffer == buf {
            check_arg_idx(curwin.get());
        }
        maketitle();
        status_redraw_all();
        fmarks_check_names(buf);
        ml_timestamp(buf);
    }
}

pub unsafe extern "C" fn setaltfname(
    mut ffname: *mut ::core::ffi::c_char,
    mut sfname: *mut ::core::ffi::c_char,
    mut lnum: linenr_T,
) -> *mut buf_T {
    unsafe {
        let mut buf: *mut buf_T = buflist_new(ffname, sfname, lnum, 0 as ::core::ffi::c_int);
        if !buf.is_null()
            && (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            (*curwin.get()).w_alt_fnum = (*buf).handle as ::core::ffi::c_int;
        }
        return buf;
    }
}

pub unsafe extern "C" fn getaltfname(mut errmsg: bool) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut dummy: linenr_T = 0;
        if buflist_name_nr(0 as ::core::ffi::c_int, &raw mut fname, &raw mut dummy) == FAIL {
            if errmsg {
                emsg(gettext(&raw const e_noalt as *const ::core::ffi::c_char));
            }
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        return fname;
    }
}

pub unsafe extern "C" fn buflist_add(
    mut fname: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut buf: *mut buf_T = buflist_new(
            fname,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as linenr_T,
            flags,
        );
        if !buf.is_null() {
            return (*buf).handle as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn buflist_altfpos(mut win: *mut win_T) {
    unsafe {
        buflist_setfpos(
            curbuf.get(),
            win,
            (*win).w_cursor.lnum,
            (*win).w_cursor.col,
            true_0 != 0,
        );
    }
}

pub unsafe extern "C" fn otherfile(mut ffname: *mut ::core::ffi::c_char) -> bool {
    unsafe {
        return otherfile_buf(
            curbuf.get(),
            ffname,
            ::core::ptr::null_mut::<FileID>(),
            false_0 != 0,
        );
    }
}

pub(crate) unsafe extern "C" fn otherfile_buf(
    mut buf: *mut buf_T,
    mut ffname: *mut ::core::ffi::c_char,
    mut file_id_p: *mut FileID,
    mut file_id_valid: bool,
) -> bool {
    unsafe {
        if ffname.is_null() || *ffname as ::core::ffi::c_int == NUL || (*buf).b_ffname.is_null() {
            return true_0 != 0;
        }
        if path_fnamecmp(ffname, (*buf).b_ffname) == 0 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        let mut file_id: FileID = FileID {
            inode: 0,
            device_id: 0,
        };
        if file_id_p.is_null() {
            file_id_p = &raw mut file_id;
            file_id_valid = os_fileid(ffname, file_id_p);
        }
        if !file_id_valid {
            return true_0 != 0;
        }
        if buf_same_file_id(buf, file_id_p) {
            buf_set_file_id(buf);
            if buf_same_file_id(buf, file_id_p) {
                return false_0 != 0;
            }
        }
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn buf_set_file_id(mut buf: *mut buf_T) {
    unsafe {
        let mut file_id: FileID = FileID {
            inode: 0,
            device_id: 0,
        };
        if !(*buf).b_fname.is_null()
            && os_fileid((*buf).b_fname, &raw mut file_id) as ::core::ffi::c_int != 0
        {
            (*buf).file_id_valid = true_0 != 0;
            (*buf).file_id = file_id;
        } else {
            (*buf).file_id_valid = false_0 != 0;
        };
    }
}

unsafe extern "C" fn buf_same_file_id(mut buf: *mut buf_T, mut file_id: *mut FileID) -> bool {
    unsafe {
        return (*buf).file_id_valid as ::core::ffi::c_int != 0
            && os_fileid_equal(&raw mut (*buf).file_id, file_id) as ::core::ffi::c_int != 0;
    }
}

pub unsafe extern "C" fn fname_expand(
    mut _buf: *mut buf_T,
    mut ffname: *mut *mut ::core::ffi::c_char,
    mut sfname: *mut *mut ::core::ffi::c_char,
) {
    unsafe {
        if (*ffname).is_null() {
            return;
        }
        if (*sfname).is_null() {
            *sfname = *ffname;
        }
        *ffname = fix_fname(*ffname);
    }
}
