//! Changing the tree, and the current directory -- `chdir()`, `getcwd()`,
//! `haslocaldir()`, `mkdir()`, `delete()`, `rename()`, `filecopy()` and
//! `tempname()`.
//!
//! Everything here has a side effect the next builtin can see, which is why it
//! is grouped: `f_chdir`/`f_getcwd`/`f_haslocaldir` are the window/tab/global
//! scope ladder over the current directory, and the rest create, move, copy or
//! remove files and directories.  `f_mkdir`'s `D`/`R` flags and `f_delete`'s
//! recursive form register deferred cleanups with the calling function, so the
//! effect can outlive the call.
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{__S_IFMT, FAIL, MAXPATHL, NUL, NULL, OK, false_0};
use crate::semsg_c;
use crate::src::nvim::eval::typval::{
    tv_check_for_string_arg, tv_get_number_chk, tv_get_string, tv_get_string_buf,
};
use crate::src::nvim::eval::userfunc::{add_defer, can_add_defer};
use crate::src::nvim::eval::window::find_win_by_nr;
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::ex_docmd::{changedir_func, vim_mkdir_emsg};
use crate::src::nvim::fileio::{delete_recursive, vim_copyfile, vim_rename, vim_tempname};
use crate::src::nvim::main::{
    curtab, curwin, e_invarg, e_invargNval, e_invexpr2, e_mkdir, globaldir,
};
use crate::src::nvim::memory::{xfree, xmalloc, xstrdup, xstrlcpy};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::fs::{
    os_dirname, os_fileinfo_link, os_mkdir_recurse, os_remove, os_rmdir,
};
use crate::src::nvim::os::libc::{abort, gettext, strcmp};
use crate::src::nvim::path::{FullName_save, path_tail, path_tail_with_sep};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    CdScope, EvalFuncData, FileInfo, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, int32_t,
    kCdScopeGlobal, kCdScopeInvalid, kCdScopeTabpage, kCdScopeWindow, size_t, tabpage_T, typval_T,
    typval_vval_union, uint64_t, uv_stat_t, uv_timespec_t, varnumber_T, win_T,
};
use crate::src::nvim::window::find_tabpage;

pub unsafe extern "C" fn f_chdir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return;
        }
        let mut cwd: *mut ::core::ffi::c_char =
            xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        if os_dirname(cwd, MAXPATHL as size_t) != FAIL {
            (*rettv).vval.v_string = xstrdup(cwd);
        }
        xfree(cwd as *mut ::core::ffi::c_void);
        let mut scope: CdScope = kCdScopeGlobal;
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut s: *const ::core::ffi::c_char =
                tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
            if strcmp(s, c"global".as_ptr()) == 0 as ::core::ffi::c_int {
                scope = kCdScopeGlobal;
            } else if strcmp(s, c"tabpage".as_ptr()) == 0 as ::core::ffi::c_int {
                scope = kCdScopeTabpage;
            } else if strcmp(s, c"window".as_ptr()) == 0 as ::core::ffi::c_int {
                scope = kCdScopeWindow;
            } else {
                semsg_c!(
                    gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
                    c"scope".as_ptr(),
                    s,
                );
                return;
            }
        } else if !(*curwin.get()).w_localdir.is_null() {
            scope = kCdScopeWindow;
        } else if !(*curtab.get()).tp_localdir.is_null() {
            scope = kCdScopeTabpage;
        }
        if !changedir_func(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_string,
            scope,
        ) {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*rettv).vval.v_string as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
        }
    }
}

pub unsafe extern "C" fn f_delete(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = -1 as varnumber_T;
        if check_secure() {
            return;
        }
        let name: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        if *name as ::core::ffi::c_int == NUL {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        let mut nbuf: [::core::ffi::c_char; 65] = [0; 65];
        let mut flags: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            flags = tv_get_string_buf(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut nbuf as *mut ::core::ffi::c_char,
            );
        } else {
            flags = c"".as_ptr();
        }
        if *flags as ::core::ffi::c_int == NUL {
            (*rettv).vval.v_number = (if os_remove(name) == 0 as ::core::ffi::c_int {
                0 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            }) as varnumber_T;
        } else if strcmp(flags, c"d".as_ptr()) == 0 as ::core::ffi::c_int {
            (*rettv).vval.v_number = (if os_rmdir(name) == 0 as ::core::ffi::c_int {
                0 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            }) as varnumber_T;
        } else if strcmp(flags, c"rf".as_ptr()) == 0 as ::core::ffi::c_int {
            (*rettv).vval.v_number = delete_recursive(name) as varnumber_T;
        } else {
            semsg_c!(
                gettext(&raw const e_invexpr2 as *const ::core::ffi::c_char),
                flags,
            );
        };
    }
}

pub unsafe extern "C" fn f_filecopy(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = false_0 as varnumber_T;
        if check_secure() as ::core::ffi::c_int != 0
            || tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
            || tv_check_for_string_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        {
            return;
        }
        let mut from: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut from_info: FileInfo = FileInfo {
            stat: uv_stat_t {
                st_dev: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_ino: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_flags: 0,
                st_gen: 0,
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
        };
        if os_fileinfo_link(from, &raw mut from_info) as ::core::ffi::c_int != 0
            && (from_info.stat.st_mode & __S_IFMT as uint64_t == 0o100000 as uint64_t
                || from_info.stat.st_mode & __S_IFMT as uint64_t == 0o120000 as uint64_t)
        {
            (*rettv).vval.v_number = (vim_copyfile(
                tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
                tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
            ) == OK) as ::core::ffi::c_int as varnumber_T;
        }
    }
}

pub unsafe extern "C" fn f_getcwd(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut scope: CdScope = kCdScopeInvalid;
        let mut scope_number: [::core::ffi::c_int; 2] =
            [0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int];
        let mut cwd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut from: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut tp: *mut tabpage_T = curtab.get();
        let mut win: *mut win_T = curwin.get();
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut i: ::core::ffi::c_int = kCdScopeWindow as ::core::ffi::c_int;
        while i < kCdScopeGlobal as ::core::ffi::c_int {
            if (*argvars.offset(i as isize)).v_type as ::core::ffi::c_uint
                == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                break;
            }
            if (*argvars.offset(i as isize)).v_type as ::core::ffi::c_uint
                != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return;
            }
            scope_number[i as usize] =
                (*argvars.offset(i as isize)).vval.v_number as ::core::ffi::c_int;
            if scope_number[i as usize] < -1 as ::core::ffi::c_int {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return;
            }
            if scope_number[i as usize] >= 0 as ::core::ffi::c_int
                && scope as ::core::ffi::c_int == kCdScopeInvalid as ::core::ffi::c_int
            {
                scope = i as CdScope;
            } else if scope_number[i as usize] < 0 as ::core::ffi::c_int {
                scope = (i + 1 as ::core::ffi::c_int) as CdScope;
            }
            i += 1;
        }
        if scope_number[kCdScopeTabpage as ::core::ffi::c_int as usize] > 0 as ::core::ffi::c_int {
            tp = find_tabpage(scope_number[kCdScopeTabpage as ::core::ffi::c_int as usize]);
            if tp.is_null() {
                emsg(gettext(c"E5000: Cannot find tab number.".as_ptr()));
                return;
            }
        }
        if scope_number[kCdScopeWindow as ::core::ffi::c_int as usize] >= 0 as ::core::ffi::c_int {
            if scope_number[kCdScopeTabpage as ::core::ffi::c_int as usize]
                < 0 as ::core::ffi::c_int
            {
                emsg(gettext(
                    c"E5001: Higher scope cannot be -1 if lower scope is >= 0.".as_ptr(),
                ));
                return;
            }
            if scope_number[kCdScopeWindow as ::core::ffi::c_int as usize] > 0 as ::core::ffi::c_int
            {
                win = find_win_by_nr(argvars.offset(0 as ::core::ffi::c_int as isize), tp);
                if win.is_null() {
                    emsg(gettext(c"E5002: Cannot find window number.".as_ptr()));
                    return;
                }
            }
        }
        cwd = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        's_250: {
            'c_30008: {
                'c_30005: {
                    match scope as ::core::ffi::c_int {
                        0 => {
                            debug_assert!(!win.is_null(), "win");
                            from = (*win).w_localdir;
                            if !from.is_null() {
                                break 's_250;
                            }
                        }
                        1 => {}
                        2 => {
                            break 'c_30005;
                        }
                        -1 => {
                            break 'c_30008;
                        }
                        _ => {
                            break 's_250;
                        }
                    }
                    debug_assert!(!tp.is_null(), "tp");
                    from = (*tp).tp_localdir;
                    if !from.is_null() {
                        break 's_250;
                    }
                }
                if !(*globaldir.ptr()).is_null() {
                    from = globaldir.get();
                    break 's_250;
                }
            }
            if os_dirname(cwd, MAXPATHL as size_t) == FAIL {
                from = c"".as_ptr() as *mut ::core::ffi::c_char;
            }
        }
        if !from.is_null() {
            xstrlcpy(cwd, from, MAXPATHL as size_t);
        }
        (*rettv).vval.v_string = xstrdup(cwd);
        xfree(cwd as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn f_haslocaldir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut scope: CdScope = kCdScopeInvalid;
        let mut scope_number: [::core::ffi::c_int; 2] =
            [0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int];
        let mut tp: *mut tabpage_T = curtab.get();
        let mut win: *mut win_T = curwin.get();
        (*rettv).v_type = VAR_NUMBER;
        (*rettv).vval.v_number = 0 as varnumber_T;
        let mut i: ::core::ffi::c_int = kCdScopeWindow as ::core::ffi::c_int;
        while i < kCdScopeGlobal as ::core::ffi::c_int {
            if (*argvars.offset(i as isize)).v_type as ::core::ffi::c_uint
                == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                break;
            }
            if (*argvars.offset(i as isize)).v_type as ::core::ffi::c_uint
                != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return;
            }
            scope_number[i as usize] =
                (*argvars.offset(i as isize)).vval.v_number as ::core::ffi::c_int;
            if scope_number[i as usize] < -1 as ::core::ffi::c_int {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return;
            }
            if scope_number[i as usize] >= 0 as ::core::ffi::c_int
                && scope as ::core::ffi::c_int == kCdScopeInvalid as ::core::ffi::c_int
            {
                scope = i as CdScope;
            } else if scope_number[i as usize] < 0 as ::core::ffi::c_int {
                scope = (i + 1 as ::core::ffi::c_int) as CdScope;
            }
            i += 1;
        }
        if scope as ::core::ffi::c_int == kCdScopeInvalid as ::core::ffi::c_int {
            scope = kCdScopeWindow;
        }
        if scope_number[kCdScopeTabpage as ::core::ffi::c_int as usize] > 0 as ::core::ffi::c_int {
            tp = find_tabpage(scope_number[kCdScopeTabpage as ::core::ffi::c_int as usize]);
            if tp.is_null() {
                emsg(gettext(c"E5000: Cannot find tab number.".as_ptr()));
                return;
            }
        }
        if scope_number[kCdScopeWindow as ::core::ffi::c_int as usize] >= 0 as ::core::ffi::c_int {
            if scope_number[kCdScopeTabpage as ::core::ffi::c_int as usize]
                < 0 as ::core::ffi::c_int
            {
                emsg(gettext(
                    c"E5001: Higher scope cannot be -1 if lower scope is >= 0.".as_ptr(),
                ));
                return;
            }
            if scope_number[kCdScopeWindow as ::core::ffi::c_int as usize] > 0 as ::core::ffi::c_int
            {
                win = find_win_by_nr(argvars.offset(0 as ::core::ffi::c_int as isize), tp);
                if win.is_null() {
                    emsg(gettext(c"E5002: Cannot find window number.".as_ptr()));
                    return;
                }
            }
        }
        match scope as ::core::ffi::c_int {
            0 => {
                debug_assert!(!win.is_null(), "win");
                (*rettv).vval.v_number = (if !(*win).w_localdir.is_null() {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as varnumber_T;
            }
            1 => {
                debug_assert!(!tp.is_null(), "tp");
                (*rettv).vval.v_number = (if !(*tp).tp_localdir.is_null() {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as varnumber_T;
            }
            -1 => {
                abort();
            }
            2 | _ => {}
        };
    }
}

pub unsafe extern "C" fn f_mkdir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut prot: ::core::ffi::c_int = 0o755 as ::core::ffi::c_int;
        (*rettv).vval.v_number = FAIL as varnumber_T;
        if check_secure() {
            return;
        }
        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
        let dir: *const ::core::ffi::c_char = tv_get_string_buf(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if *dir as ::core::ffi::c_int == NUL {
            return;
        }
        if *path_tail(dir) as ::core::ffi::c_int == NUL {
            *path_tail_with_sep(dir as *mut ::core::ffi::c_char) = NUL as ::core::ffi::c_char;
        }
        let mut defer: bool = false_0 != 0;
        let mut defer_recurse: bool = false_0 != 0;
        let mut created: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                prot = tv_get_number_chk(
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    ::core::ptr::null_mut::<bool>(),
                ) as ::core::ffi::c_int;
                if prot == -1 as ::core::ffi::c_int {
                    return;
                }
            }
            let mut arg2: *const ::core::ffi::c_char =
                tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
            defer = !vim_strchr(arg2, 'D' as ::core::ffi::c_int).is_null();
            defer_recurse = !vim_strchr(arg2, 'R' as ::core::ffi::c_int).is_null();
            if (defer as ::core::ffi::c_int != 0 || defer_recurse as ::core::ffi::c_int != 0)
                && !can_add_defer()
            {
                return;
            }
            if !vim_strchr(arg2, 'p' as ::core::ffi::c_int).is_null() {
                let mut failed_dir: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut ret: ::core::ffi::c_int = os_mkdir_recurse(
                    dir,
                    prot as int32_t,
                    &raw mut failed_dir,
                    if defer as ::core::ffi::c_int != 0 || defer_recurse as ::core::ffi::c_int != 0
                    {
                        &raw mut created
                    } else {
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>()
                    },
                );
                if ret != 0 as ::core::ffi::c_int {
                    semsg_c!(
                        gettext(&raw const e_mkdir as *const ::core::ffi::c_char),
                        failed_dir,
                        uv_strerror(ret),
                    );
                    xfree(failed_dir as *mut ::core::ffi::c_void);
                    (*rettv).vval.v_number = FAIL as varnumber_T;
                    return;
                }
                (*rettv).vval.v_number = OK as varnumber_T;
            }
        }
        if (*rettv).vval.v_number == FAIL as varnumber_T {
            (*rettv).vval.v_number = vim_mkdir_emsg(dir, prot) as varnumber_T;
        }
        if (*rettv).vval.v_number == OK as varnumber_T
            && created.is_null()
            && (defer as ::core::ffi::c_int != 0 || defer_recurse as ::core::ffi::c_int != 0)
        {
            created = FullName_save(dir, false_0 != 0);
        }
        if !created.is_null() {
            let mut tv: [typval_T; 2] = [typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            }; 2];
            tv[0 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
            tv[0 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
            tv[0 as ::core::ffi::c_int as usize].vval.v_string = created;
            tv[1 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
            tv[1 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
            tv[1 as ::core::ffi::c_int as usize].vval.v_string =
                xstrdup(if defer_recurse as ::core::ffi::c_int != 0 {
                    c"rf".as_ptr()
                } else {
                    c"d".as_ptr()
                });
            add_defer(
                c"delete".as_ptr() as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
                &raw mut tv as *mut typval_T,
            );
        }
    }
}

pub unsafe extern "C" fn f_rename(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        if check_secure() {
            (*rettv).vval.v_number = -1 as varnumber_T;
        } else {
            let mut buf: [::core::ffi::c_char; 65] = [0; 65];
            (*rettv).vval.v_number = vim_rename(
                tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
                tv_get_string_buf(
                    argvars.offset(1 as ::core::ffi::c_int as isize),
                    &raw mut buf as *mut ::core::ffi::c_char,
                ),
            ) as varnumber_T;
        };
    }
}

pub unsafe extern "C" fn f_tempname(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = vim_tempname();
    }
}
