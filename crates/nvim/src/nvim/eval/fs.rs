use crate::semsg_c;
use crate::src::nvim::cmdexpand::{ExpandCleanup, ExpandInit, ExpandOne, globpath};
use crate::src::nvim::eval::typval::{
    tv_blob_alloc_ret, tv_blob_free, tv_check_for_nonempty_string_arg, tv_check_for_string_arg,
    tv_check_str_or_nr, tv_clear, tv_get_number, tv_get_number_chk, tv_get_string,
    tv_get_string_buf, tv_get_string_buf_chk, tv_get_string_chk, tv_list_alloc_ret,
    tv_list_append_owned_tv, tv_list_append_string, tv_list_item_remove,
};
use crate::src::nvim::eval::typval::{tv_blob_len, tv_list_first, tv_list_len, tv_list_set_ret};
use crate::src::nvim::eval::userfunc::{add_defer, can_add_defer};
use crate::src::nvim::eval::vars::{prepare_vimvar, restore_vimvar, set_vim_var_string};
use crate::src::nvim::eval::window::find_win_by_nr;
use crate::src::nvim::eval::{do_string_sub, eval_expr_typval};
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::ex_docmd::{changedir_func, vim_mkdir_emsg};
use crate::src::nvim::file_search::{find_file_in_path_option, vim_findfile_cleanup};
use crate::src::nvim::fileio::{
    delete_recursive, file_pat_to_reg_pat, readdir_core, vim_copyfile, vim_rename, vim_tempname,
};
use crate::src::nvim::garray::{ga_clear_strings, ga_concat_strings, ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    curbuf, current_sctx, curtab, curwin, e_cant_read_file_str, e_invarg, e_invarg2, e_invargNval,
    e_invexpr2, e_isadir2, e_mkdir, e_notopen, globaldir, p_fs, p_path, p_wic,
};
use crate::src::nvim::mbyte::{utf_head_off, utfc_ptr2len};
use crate::src::nvim::memory::{
    xfree, xmalloc, xmallocz, xmemdupz, xrealloc, xstrdup, xstrlcat, xstrlcpy,
};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::env::{expand_env_save, home_replace};
use crate::src::nvim::os::fileio::{file_close, file_flush, file_open, file_write};
use crate::src::nvim::os::fs::{
    os_can_exe, os_dirname, os_file_is_readable, os_file_is_writable, os_fileinfo, os_fileinfo_fd,
    os_fileinfo_link, os_fileinfo_size, os_fopen, os_getperm, os_isdir, os_mkdir_recurse,
    os_remove, os_rmdir,
};
use crate::src::nvim::os::libc::{
    abort, fclose, fileno, fread, fseeko, gettext, memcpy, memmove, readlink, strcmp, strlen,
};
use crate::src::nvim::path::{
    FullName_save, add_pathsep, after_pathsep, get_past_head, path_fnamencmp, path_is_absolute,
    path_next_component, path_tail, path_tail_with_sep, shorten_dir_len, simplify_filename,
    vim_isAbsName, vim_ispathsep,
};
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::runtime::script_is_lua;
use crate::src::nvim::strings::{concat_str, vim_strchr, vim_strsave_shellescape, xstrnsave};
use crate::src::nvim::types::{
    __off_t, CdScope, Direction, EvalFuncData, FILE, FileDescriptor, FileInfo, VAR_BLOB, VAR_LIST,
    VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, VV_VAL, blob_T, buf_T, expand_T, garray_T,
    int32_t, int64_t, kCdScopeGlobal, kCdScopeInvalid, kCdScopeTabpage, kCdScopeWindow,
    kListLenUnknown, list_T, listitem_T, off_T, pos_T, ptrdiff_t, sctx_T, size_t, ssize_t,
    tabpage_T, typval_T, typval_vval_union, uint8_t, uint64_t, uv_stat_t, uv_timespec_t,
    varnumber_T, win_T, xp_prefix_T,
};
use crate::src::nvim::window::find_tabpage;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const EXPAND_FILES: C2Rust_Unnamed_14 = 2;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const WILD_ALL_KEEP: C2Rust_Unnamed_15 = 8;
pub const WILD_ALL: C2Rust_Unnamed_15 = 6;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const WILD_IGNORE_COMPLETESLASH: C2Rust_Unnamed_16 = 1024;
pub const WILD_ALLLINKS: C2Rust_Unnamed_16 = 512;
pub const WILD_ICASE: C2Rust_Unnamed_16 = 256;
pub const WILD_SILENT: C2Rust_Unnamed_16 = 64;
pub const WILD_KEEP_ALL: C2Rust_Unnamed_16 = 32;
pub const WILD_USE_NL: C2Rust_Unnamed_16 = 4;
pub const VALID_PATH: C2Rust_Unnamed_17 = 1;
pub const VALID_HEAD: C2Rust_Unnamed_17 = 2;
pub const FINDFILE_DIR: C2Rust_Unnamed_18 = 1;
pub const FINDFILE_FILE: C2Rust_Unnamed_18 = 0;
pub const kFileCreate: C2Rust_Unnamed_19 = 2;
pub const kFileMkDir: C2Rust_Unnamed_19 = 256;
pub const kFileTruncate: C2Rust_Unnamed_19 = 32;
pub const kFileAppend: C2Rust_Unnamed_19 = 64;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kFileCreateOnly: C2Rust_Unnamed_19 = 16;
pub const kFileNoSymlink: C2Rust_Unnamed_19 = 8;
pub const kFileWriteOnly: C2Rust_Unnamed_19 = 4;
pub const kFileReadOnly: C2Rust_Unnamed_19 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static e_error_while_writing_str: GlobalCell<[::core::ffi::c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E80: Error while writing: %s\0",
    )
});
pub unsafe extern "C" fn modify_fname(
    mut src: *mut ::core::ffi::c_char,
    mut tilde_file: bool,
    mut usedlen: *mut size_t,
    mut fnamep: *mut *mut ::core::ffi::c_char,
    mut bufp: *mut *mut ::core::ffi::c_char,
    mut fnamelen: *mut size_t,
) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut valid: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pbuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dirname: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut has_fullname: bool = false_0 != 0;
    let mut has_homerelative: bool = false_0 != 0;
    loop {
        if *src.add(*usedlen) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                == 'p' as ::core::ffi::c_int
        {
            has_fullname = true_0 != 0;
            valid |= VALID_PATH as ::core::ffi::c_int;
            *usedlen = (*usedlen).wrapping_add(2 as size_t);
            if *(*fnamep).offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '~' as ::core::ffi::c_int
                && !(tilde_file as ::core::ffi::c_int != 0
                    && *(*fnamep).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == NUL)
            {
                *fnamep = expand_env_save(*fnamep);
                xfree(*bufp as *mut ::core::ffi::c_void);
                *bufp = *fnamep;
                if (*fnamep).is_null() {
                    return -1 as ::core::ffi::c_int;
                }
            }
            p = *fnamep;
            while *p as ::core::ffi::c_int != NUL {
                if vim_ispathsep(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '.' as ::core::ffi::c_int
                    && (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                        || vim_ispathsep(
                            *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0
                        || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '.' as ::core::ffi::c_int
                            && (*p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == NUL
                                || vim_ispathsep(*p.offset(3 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0))
                {
                    break;
                }
                p = p.offset(utfc_ptr2len(p) as isize);
            }
            if *p as ::core::ffi::c_int != NUL || !vim_isAbsName(*fnamep) {
                *fnamep = FullName_save(*fnamep, *p as ::core::ffi::c_int != NUL);
                xfree(*bufp as *mut ::core::ffi::c_void);
                *bufp = *fnamep;
                if (*fnamep).is_null() {
                    return -1 as ::core::ffi::c_int;
                }
            }
            if os_isdir(*fnamep) {
                *fnamep = xstrnsave(*fnamep, strlen(*fnamep).wrapping_add(2 as size_t));
                xfree(*bufp as *mut ::core::ffi::c_void);
                *bufp = *fnamep;
                add_pathsep(*fnamep);
            }
        }
        c = 0;
        while *src.add(*usedlen) as ::core::ffi::c_int == ':' as ::core::ffi::c_int && {
            c = *src.add((*usedlen).wrapping_add(1 as size_t)) as uint8_t as ::core::ffi::c_int;
            c == '.' as ::core::ffi::c_int
                || c == '~' as ::core::ffi::c_int
                || c == '8' as ::core::ffi::c_int
        } {
            *usedlen = (*usedlen).wrapping_add(2 as size_t);
            if c == '8' as ::core::ffi::c_int {
                continue;
            }
            pbuf = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if !has_fullname && !has_homerelative {
                if **fnamep as ::core::ffi::c_int == '~' as ::core::ffi::c_int {
                    pbuf = expand_env_save(*fnamep);
                    p = pbuf;
                } else {
                    pbuf = FullName_save(*fnamep, false_0 != 0);
                    p = pbuf;
                }
            } else {
                p = *fnamep;
            }
            has_fullname = false_0 != 0;
            if !p.is_null() {
                if c == '.' as ::core::ffi::c_int {
                    os_dirname(
                        &raw mut dirname as *mut ::core::ffi::c_char,
                        MAXPATHL as size_t,
                    );
                    if has_homerelative {
                        s = xstrdup(&raw mut dirname as *mut ::core::ffi::c_char);
                        home_replace(
                            ::core::ptr::null::<buf_T>(),
                            s,
                            &raw mut dirname as *mut ::core::ffi::c_char,
                            MAXPATHL as size_t,
                            true_0 != 0,
                        );
                        xfree(s as *mut ::core::ffi::c_void);
                    }
                    let mut namelen: size_t = strlen(&raw mut dirname as *mut ::core::ffi::c_char);
                    if path_fnamencmp(p, &raw mut dirname as *mut ::core::ffi::c_char, namelen)
                        == 0 as ::core::ffi::c_int
                    {
                        p = p.add(namelen);
                        if vim_ispathsep(*p as ::core::ffi::c_int) {
                            while *p as ::core::ffi::c_int != 0
                                && vim_ispathsep(*p as ::core::ffi::c_int) as ::core::ffi::c_int
                                    != 0
                            {
                                p = p.offset(1);
                            }
                            *fnamep = p;
                            if !pbuf.is_null() {
                                xfree(*bufp as *mut ::core::ffi::c_void);
                                *bufp = pbuf;
                                pbuf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                            }
                        }
                    }
                } else {
                    home_replace(
                        ::core::ptr::null::<buf_T>(),
                        p,
                        &raw mut dirname as *mut ::core::ffi::c_char,
                        MAXPATHL as size_t,
                        true_0 != 0,
                    );
                    if *(&raw mut dirname as *mut ::core::ffi::c_char) as ::core::ffi::c_int
                        == '~' as ::core::ffi::c_int
                    {
                        s = xstrdup(&raw mut dirname as *mut ::core::ffi::c_char);
                        debug_assert!(!s.is_null(), "s != NULL");
                        *fnamep = s;
                        xfree(*bufp as *mut ::core::ffi::c_void);
                        *bufp = s;
                        has_homerelative = true_0 != 0;
                    }
                }
                xfree(pbuf as *mut ::core::ffi::c_void);
            }
        }
        tail = path_tail(*fnamep);
        *fnamelen = strlen(*fnamep);
        while *src.add(*usedlen) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                == 'h' as ::core::ffi::c_int
        {
            valid |= VALID_HEAD as ::core::ffi::c_int;
            *usedlen = (*usedlen).wrapping_add(2 as size_t);
            s = get_past_head(*fnamep);
            while tail > s && after_pathsep(s, tail) != 0 {
                tail = tail.offset(
                    -((utf_head_off(*fnamep, tail.offset(-(1 as ::core::ffi::c_int as isize)))
                        + 1 as ::core::ffi::c_int) as isize),
                );
            }
            *fnamelen = tail.offset_from(*fnamep) as size_t;
            if *fnamelen == 0 as size_t {
                xfree(*bufp as *mut ::core::ffi::c_void);
                tail = xstrdup(c".".as_ptr());
                *fnamep = tail;
                *bufp = *fnamep;
                *fnamelen = 1 as size_t;
            } else {
                while tail > s && after_pathsep(s, tail) == 0 {
                    tail = tail.offset(
                        -((utf_head_off(*fnamep, tail.offset(-(1 as ::core::ffi::c_int as isize)))
                            + 1 as ::core::ffi::c_int) as isize),
                    );
                }
            }
        }
        if *src.add(*usedlen) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                == '8' as ::core::ffi::c_int
        {
            *usedlen = (*usedlen).wrapping_add(2 as size_t);
        }
        if *src.add(*usedlen) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                == 't' as ::core::ffi::c_int
        {
            *usedlen = (*usedlen).wrapping_add(2 as size_t);
            *fnamelen = (*fnamelen).wrapping_sub(tail.offset_from(*fnamep) as size_t);
            *fnamep = tail;
        }
        while *src.add(*usedlen) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && (*src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                == 'e' as ::core::ffi::c_int
                || *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                    == 'r' as ::core::ffi::c_int)
        {
            let is_second_e: bool = *fnamep > tail;
            if *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                == 'e' as ::core::ffi::c_int
                && is_second_e as ::core::ffi::c_int != 0
            {
                s = (*fnamep).offset(-(2 as ::core::ffi::c_int as isize));
            } else {
                s = (*fnamep)
                    .add(*fnamelen)
                    .offset(-(1 as ::core::ffi::c_int as isize));
            }
            while s > tail {
                if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                {
                    break;
                }
                s = s.offset(-1);
            }
            if *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                == 'e' as ::core::ffi::c_int
            {
                if s > tail || false && is_second_e as ::core::ffi::c_int != 0 && s == tail {
                    let mut newstart: *mut ::core::ffi::c_char =
                        s.offset(1 as ::core::ffi::c_int as isize);
                    let mut distance_stepped_back: size_t =
                        (*fnamep).offset_from(newstart) as size_t;
                    *fnamelen = (*fnamelen).wrapping_add(distance_stepped_back);
                    *fnamep = newstart;
                } else if *fnamep <= tail {
                    *fnamelen = 0 as size_t;
                }
            } else if s > (if tail > *fnamep { tail } else { *fnamep }) {
                *fnamelen = s.offset_from(*fnamep) as size_t;
            }
            *usedlen = (*usedlen).wrapping_add(2 as size_t);
        }
        if !(*src.add(*usedlen) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && (*src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                == 's' as ::core::ffi::c_int
                || *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                    == 'g' as ::core::ffi::c_int
                    && *src.add((*usedlen).wrapping_add(2 as size_t)) as ::core::ffi::c_int
                        == 's' as ::core::ffi::c_int))
        {
            break;
        }
        let mut didit: bool = false_0 != 0;
        let mut flags: *mut ::core::ffi::c_char = c"".as_ptr() as *mut ::core::ffi::c_char;
        s = src.add(*usedlen).offset(2 as ::core::ffi::c_int as isize);
        if *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
            == 'g' as ::core::ffi::c_int
        {
            flags = c"g".as_ptr() as *mut ::core::ffi::c_char;
            s = s.offset(1);
        }
        let c2rust_fresh0 = s;
        s = s.offset(1);
        let mut sep: ::core::ffi::c_int = *c2rust_fresh0 as uint8_t as ::core::ffi::c_int;
        if sep == 0 {
            break;
        }
        p = vim_strchr(s, sep);
        if !p.is_null() {
            let pat: *mut ::core::ffi::c_char =
                xmemdupz(s as *const ::core::ffi::c_void, p.offset_from(s) as size_t)
                    as *mut ::core::ffi::c_char;
            s = p.offset(1 as ::core::ffi::c_int as isize);
            p = vim_strchr(s, sep);
            if !p.is_null() {
                let sub: *mut ::core::ffi::c_char =
                    xmemdupz(s as *const ::core::ffi::c_void, p.offset_from(s) as size_t)
                        as *mut ::core::ffi::c_char;
                let str: *mut ::core::ffi::c_char =
                    xmemdupz(*fnamep as *const ::core::ffi::c_void, *fnamelen)
                        as *mut ::core::ffi::c_char;
                *usedlen = p.offset(1 as ::core::ffi::c_int as isize).offset_from(src) as size_t;
                let mut slen: size_t = 0;
                s = do_string_sub(
                    str,
                    *fnamelen,
                    pat,
                    sub,
                    ::core::ptr::null_mut::<typval_T>(),
                    flags,
                    &raw mut slen,
                );
                *fnamep = s;
                *fnamelen = slen;
                xfree(*bufp as *mut ::core::ffi::c_void);
                *bufp = s;
                didit = true_0 != 0;
                xfree(sub as *mut ::core::ffi::c_void);
                xfree(str as *mut ::core::ffi::c_void);
            }
            xfree(pat as *mut ::core::ffi::c_void);
        }
        if !didit {
            break;
        }
    }
    if *src.add(*usedlen) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
        && *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
            == 'S' as ::core::ffi::c_int
    {
        c = *(*fnamep).add(*fnamelen) as uint8_t as ::core::ffi::c_int;
        if c != NUL {
            *(*fnamep).add(*fnamelen) = NUL as ::core::ffi::c_char;
        }
        p = vim_strsave_shellescape(*fnamep, false_0 != 0, false_0 != 0);
        if c != NUL {
            *(*fnamep).add(*fnamelen) = c as ::core::ffi::c_char;
        }
        xfree(*bufp as *mut ::core::ffi::c_void);
        *fnamep = p;
        *bufp = *fnamep;
        *fnamelen = strlen(p);
        *usedlen = (*usedlen).wrapping_add(2 as size_t);
    }
    return valid;
}
pub unsafe extern "C" fn f_chdir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    let mut cwd: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
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
pub unsafe extern "C" fn f_delete(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
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
pub unsafe extern "C" fn f_executable(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    (*rettv).vval.v_number = os_can_exe(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        true_0 != 0,
    ) as varnumber_T;
}
pub unsafe extern "C" fn f_exepath(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_nonempty_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    os_can_exe(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
        &raw mut path,
        true_0 != 0,
    );
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = path;
}
pub unsafe extern "C" fn f_filecopy(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
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
pub unsafe extern "C" fn f_filereadable(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let p: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).vval.v_number = (*p as ::core::ffi::c_int != 0
        && !os_isdir(p)
        && os_file_is_readable(p) as ::core::ffi::c_int != 0)
        as ::core::ffi::c_int as varnumber_T;
}
pub unsafe extern "C" fn f_filewritable(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut filename: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).vval.v_number = os_file_is_writable(filename) as varnumber_T;
}
unsafe extern "C" fn findfilendir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut find_what: ::core::ffi::c_int,
) {
    let mut fresult: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut path: *mut ::core::ffi::c_char =
        if *(*curbuf.get()).b_p_path as ::core::ffi::c_int == NUL {
            p_path.get()
        } else {
            (*curbuf.get()).b_p_path
        };
    let mut count: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut first: bool = true_0 != 0;
    let mut error: bool = false_0 != 0;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*rettv).v_type = VAR_STRING;
    let mut fname: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut pathbuf: [::core::ffi::c_char; 65] = [0; 65];
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut p: *const ::core::ffi::c_char = tv_get_string_buf_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut pathbuf as *mut ::core::ffi::c_char,
        );
        if p.is_null() {
            error = true_0 != 0;
        } else {
            if *p as ::core::ffi::c_int != NUL {
                path = p as *mut ::core::ffi::c_char;
            }
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                count = tv_get_number_chk(
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    &raw mut error,
                ) as ::core::ffi::c_int;
            }
        }
    }
    if count < 0 as ::core::ffi::c_int {
        tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    }
    if *fname as ::core::ffi::c_int != NUL && !error {
        let mut file_to_find: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut search_ctx: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        loop {
            if (*rettv).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*rettv).v_type as ::core::ffi::c_uint
                    == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                xfree(fresult as *mut ::core::ffi::c_void);
            }
            fresult = find_file_in_path_option(
                if first as ::core::ffi::c_int != 0 {
                    fname as *mut ::core::ffi::c_char
                } else {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                },
                if first as ::core::ffi::c_int != 0 {
                    strlen(fname)
                } else {
                    0 as size_t
                },
                0 as ::core::ffi::c_int,
                first,
                path,
                find_what,
                (*curbuf.get()).b_ffname,
                (if find_what == FINDFILE_DIR as ::core::ffi::c_int {
                    c"".as_ptr()
                } else {
                    (*curbuf.get()).b_p_sua as *const ::core::ffi::c_char
                }) as *mut ::core::ffi::c_char,
                &raw mut file_to_find,
                &raw mut search_ctx,
            );
            first = false_0 != 0;
            if !fresult.is_null()
                && (*rettv).v_type as ::core::ffi::c_uint
                    == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                tv_list_append_string((*rettv).vval.v_list, fresult, -1 as ssize_t);
            }
            if !(((*rettv).v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                || {
                    count -= 1;
                    count > 0 as ::core::ffi::c_int
                })
                && !fresult.is_null())
            {
                break;
            }
        }
        xfree(file_to_find as *mut ::core::ffi::c_void);
        vim_findfile_cleanup(search_ctx as *mut ::core::ffi::c_void);
    }
    if (*rettv).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*rettv).vval.v_string = fresult;
    }
}
pub unsafe extern "C" fn f_finddir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    findfilendir(argvars, rettv, FINDFILE_DIR as ::core::ffi::c_int);
}
pub unsafe extern "C" fn f_findfile(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    findfilendir(argvars, rettv, FINDFILE_FILE as ::core::ffi::c_int);
}
pub unsafe extern "C" fn f_fnamemodify(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut fbuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0 as size_t;
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut fname: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mods: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    if mods.is_null() || fname.is_null() {
        fname = ::core::ptr::null::<::core::ffi::c_char>();
    } else {
        len = strlen(fname);
        if *mods as ::core::ffi::c_int != NUL {
            let mut usedlen: size_t = 0 as size_t;
            modify_fname(
                mods as *mut ::core::ffi::c_char,
                false_0 != 0,
                &raw mut usedlen,
                &raw mut fname as *mut *mut ::core::ffi::c_char,
                &raw mut fbuf,
                &raw mut len,
            );
        }
    }
    (*rettv).v_type = VAR_STRING;
    if fname.is_null() {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        (*rettv).vval.v_string =
            xmemdupz(fname as *const ::core::ffi::c_void, len) as *mut ::core::ffi::c_char;
    }
    xfree(fbuf as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn f_getcwd(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
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
        if scope_number[kCdScopeTabpage as ::core::ffi::c_int as usize] < 0 as ::core::ffi::c_int {
            emsg(gettext(
                c"E5001: Higher scope cannot be -1 if lower scope is >= 0.".as_ptr(),
            ));
            return;
        }
        if scope_number[kCdScopeWindow as ::core::ffi::c_int as usize] > 0 as ::core::ffi::c_int {
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
pub unsafe extern "C" fn f_getfperm(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut perm: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut flags: [::core::ffi::c_char; 4] =
        ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"rwx\0");
    let mut filename: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut file_perm: int32_t = os_getperm(filename);
    if file_perm >= 0 as int32_t {
        perm = xstrdup(c"---------".as_ptr());
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 9 as ::core::ffi::c_int {
            if file_perm & (1 as int32_t) << 8 as ::core::ffi::c_int - i != 0 {
                *perm.offset(i as isize) = flags[(i % 3 as ::core::ffi::c_int) as usize];
            }
            i += 1;
        }
    }
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = perm;
}
pub unsafe extern "C" fn f_getfsize(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut fname: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).v_type = VAR_NUMBER;
    let mut file_info: FileInfo = FileInfo {
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
    if os_fileinfo(fname, &raw mut file_info) {
        let mut filesize: uint64_t = os_fileinfo_size(&raw mut file_info);
        if os_isdir(fname) {
            (*rettv).vval.v_number = 0 as varnumber_T;
        } else {
            (*rettv).vval.v_number = filesize as varnumber_T;
            if (*rettv).vval.v_number as uint64_t != filesize {
                (*rettv).vval.v_number = -2 as varnumber_T;
            }
        }
    } else {
        (*rettv).vval.v_number = -1 as varnumber_T;
    };
}
pub unsafe extern "C" fn f_getftime(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut fname: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut file_info: FileInfo = FileInfo {
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
    if os_fileinfo(fname, &raw mut file_info) {
        (*rettv).vval.v_number = file_info.stat.st_mtim.tv_sec as varnumber_T;
    } else {
        (*rettv).vval.v_number = -1 as varnumber_T;
    };
}
pub unsafe extern "C" fn f_getftype(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut type_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut t: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fname: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).v_type = VAR_STRING;
    let mut file_info: FileInfo = FileInfo {
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
    if os_fileinfo_link(fname, &raw mut file_info) {
        let mut mode: uint64_t = file_info.stat.st_mode;
        if mode & __S_IFMT as uint64_t == 0o100000 as uint64_t {
            t = c"file".as_ptr() as *mut ::core::ffi::c_char;
        } else if mode & __S_IFMT as uint64_t == 0o40000 as uint64_t {
            t = c"dir".as_ptr() as *mut ::core::ffi::c_char;
        } else if mode & __S_IFMT as uint64_t == 0o120000 as uint64_t {
            t = c"link".as_ptr() as *mut ::core::ffi::c_char;
        } else if mode & __S_IFMT as uint64_t == 0o60000 as uint64_t {
            t = c"bdev".as_ptr() as *mut ::core::ffi::c_char;
        } else if mode & __S_IFMT as uint64_t == 0o20000 as uint64_t {
            t = c"cdev".as_ptr() as *mut ::core::ffi::c_char;
        } else if mode & __S_IFMT as uint64_t == 0o10000 as uint64_t {
            t = c"fifo".as_ptr() as *mut ::core::ffi::c_char;
        } else if mode & __S_IFMT as uint64_t == 0o140000 as uint64_t {
            t = c"socket".as_ptr() as *mut ::core::ffi::c_char;
        } else {
            t = c"other".as_ptr() as *mut ::core::ffi::c_char;
        }
        type_0 = xstrdup(t);
    }
    (*rettv).vval.v_string = type_0;
}
pub unsafe extern "C" fn f_glob(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut options: ::core::ffi::c_int =
        WILD_SILENT as ::core::ffi::c_int | WILD_USE_NL as ::core::ffi::c_int;
    let mut xpc: expand_T = expand_T {
        xp_pattern: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        xp_context: 0,
        xp_pattern_len: 0,
        xp_prefix: XP_PREFIX_NONE,
        xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        xp_luaref: 0,
        xp_script_ctx: sctx_T {
            sc_sid: 0,
            sc_seq: 0,
            sc_lnum: 0,
            sc_chan: 0,
        },
        xp_backslash: 0,
        xp_shell: false,
        xp_numfiles: 0,
        xp_col: 0,
        xp_selected: 0,
        xp_orig: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        xp_files: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        xp_line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        xp_buf: [0; 256],
        xp_search_dir: kDirectionNotSet,
        xp_pre_incsearch_pos: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
    };
    let mut error: bool = false_0 != 0;
    (*rettv).v_type = VAR_STRING;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) != 0
        {
            options |= WILD_KEEP_ALL as ::core::ffi::c_int;
        }
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) != 0
            {
                tv_list_set_ret(rettv, ::core::ptr::null_mut::<list_T>());
            }
            if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && tv_get_number_chk(
                    argvars.offset(3 as ::core::ffi::c_int as isize),
                    &raw mut error,
                ) != 0
            {
                options |= WILD_ALLLINKS as ::core::ffi::c_int;
            }
        }
    }
    if !error {
        ExpandInit(&raw mut xpc);
        xpc.xp_context = EXPAND_FILES as ::core::ffi::c_int;
        if p_wic.get() != 0 {
            options += WILD_ICASE as ::core::ffi::c_int;
        }
        if (*rettv).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*rettv).vval.v_string = ExpandOne(
                &raw mut xpc,
                tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
                    as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                options,
                WILD_ALL as ::core::ffi::c_int,
            );
        } else {
            ExpandOne(
                &raw mut xpc,
                tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
                    as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                options,
                WILD_ALL_KEEP as ::core::ffi::c_int,
            );
            tv_list_alloc_ret(rettv, xpc.xp_numfiles as ptrdiff_t);
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < xpc.xp_numfiles {
                tv_list_append_string(
                    (*rettv).vval.v_list,
                    *xpc.xp_files.offset(i as isize),
                    -1 as ssize_t,
                );
                i += 1;
            }
            ExpandCleanup(&raw mut xpc);
        }
    } else {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    };
}
pub unsafe extern "C" fn f_globpath(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut flags: ::core::ffi::c_int = WILD_IGNORE_COMPLETESLASH as ::core::ffi::c_int;
    let mut error: bool = false_0 != 0;
    (*rettv).v_type = VAR_STRING;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_get_number_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) != 0
        {
            flags |= WILD_KEEP_ALL as ::core::ffi::c_int;
        }
        if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if tv_get_number_chk(
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) != 0
            {
                tv_list_set_ret(rettv, ::core::ptr::null_mut::<list_T>());
            }
            if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && tv_get_number_chk(
                    argvars.offset(4 as ::core::ffi::c_int as isize),
                    &raw mut error,
                ) != 0
            {
                flags |= WILD_ALLLINKS as ::core::ffi::c_int;
            }
        }
    }
    let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
    let file: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut buf1 as *mut ::core::ffi::c_char,
    );
    if !file.is_null() && !error {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            10 as ::core::ffi::c_int,
        );
        globpath(
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
                as *mut ::core::ffi::c_char,
            file as *mut ::core::ffi::c_char,
            &raw mut ga,
            flags,
            false_0 != 0,
        );
        if (*rettv).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*rettv).vval.v_string = ga_concat_strings(&raw mut ga, c"\n".as_ptr());
        } else {
            tv_list_alloc_ret(rettv, ga.ga_len as ptrdiff_t);
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < ga.ga_len {
                tv_list_append_string(
                    (*rettv).vval.v_list,
                    *(ga.ga_data as *mut *const ::core::ffi::c_char).offset(i as isize),
                    -1 as ssize_t,
                );
                i += 1;
            }
        }
        ga_clear_strings(&raw mut ga);
    } else {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    };
}
pub unsafe extern "C" fn f_glob2regpat(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let pat: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = if pat.is_null() {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        file_pat_to_reg_pat(
            pat,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0,
        )
    };
}
pub unsafe extern "C" fn f_haslocaldir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
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
        if scope_number[kCdScopeTabpage as ::core::ffi::c_int as usize] < 0 as ::core::ffi::c_int {
            emsg(gettext(
                c"E5001: Higher scope cannot be -1 if lower scope is >= 0.".as_ptr(),
            ));
            return;
        }
        if scope_number[kCdScopeWindow as ::core::ffi::c_int as usize] > 0 as ::core::ffi::c_int {
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
pub unsafe extern "C" fn f_isabsolutepath(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = path_is_absolute(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    )) as varnumber_T;
}
pub unsafe extern "C" fn f_isdirectory(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = os_isdir(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    )) as varnumber_T;
}
pub unsafe extern "C" fn f_mkdir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
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
                if defer as ::core::ffi::c_int != 0 || defer_recurse as ::core::ffi::c_int != 0 {
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
pub unsafe extern "C" fn f_pathshorten(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut trim_len: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        trim_len =
            tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
        if trim_len < 1 as ::core::ffi::c_int {
            trim_len = 1 as ::core::ffi::c_int;
        }
    }
    (*rettv).v_type = VAR_STRING;
    let mut p: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if p.is_null() {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        (*rettv).vval.v_string = xstrdup(p);
        shorten_dir_len((*rettv).vval.v_string, trim_len);
    };
}
unsafe extern "C" fn readdir_checkitem(
    mut context: *mut ::core::ffi::c_void,
    mut name: *const ::core::ffi::c_char,
) -> varnumber_T {
    let mut expr: *mut typval_T = context as *mut typval_T;
    let mut argv: [typval_T; 2] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 2];
    let mut retval: varnumber_T = 0 as varnumber_T;
    let mut error: bool = false_0 != 0;
    if (*expr).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 1 as varnumber_T;
    }
    let mut save_val: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    prepare_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
    set_vim_var_string(VV_VAL, name, -1 as ptrdiff_t);
    argv[0 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
    argv[0 as ::core::ffi::c_int as usize].vval.v_string = name as *mut ::core::ffi::c_char;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if eval_expr_typval(
        expr,
        false_0 != 0,
        &raw mut argv as *mut typval_T,
        1 as ::core::ffi::c_int,
        &raw mut rettv,
    ) != FAIL
    {
        retval = tv_get_number_chk(&raw mut rettv, &raw mut error);
        if error {
            retval = -1 as varnumber_T;
        }
        tv_clear(&raw mut rettv);
    }
    set_vim_var_string(
        VV_VAL,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ptrdiff_t,
    );
    restore_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
    return retval;
}
pub unsafe extern "C" fn f_readdir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    let mut path: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut expr: *mut typval_T = argvars.offset(1 as ::core::ffi::c_int as isize);
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut ret: ::core::ffi::c_int = readdir_core(
        &raw mut ga,
        path,
        expr as *mut ::core::ffi::c_void,
        Some(
            readdir_checkitem
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *const ::core::ffi::c_char,
                ) -> varnumber_T,
        ),
    );
    if ret == OK && ga.ga_len > 0 as ::core::ffi::c_int {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < ga.ga_len {
            let mut p: *const ::core::ffi::c_char =
                *(ga.ga_data as *mut *const ::core::ffi::c_char).offset(i as isize);
            tv_list_append_string((*rettv).vval.v_list, p, -1 as ssize_t);
            i += 1;
        }
    }
    ga_clear_strings(&raw mut ga);
}
unsafe extern "C" fn read_blob(
    fd: *mut FILE,
    mut rettv: *mut typval_T,
    mut offset: off_T,
    mut size_arg: off_T,
) -> ::core::ffi::c_int {
    let blob: *mut blob_T = (*rettv).vval.v_blob;
    let mut file_info: FileInfo = FileInfo {
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
    if !os_fileinfo_fd(fileno(fd), &raw mut file_info) {
        return FAIL;
    }
    let mut whence: ::core::ffi::c_int = 0;
    let mut size: off_T = size_arg;
    let file_size: off_T = os_fileinfo_size(&raw mut file_info) as off_T;
    if offset >= 0 as off_T {
        if size == -1 as off_T
            || size > file_size - offset
                && !(file_info.stat.st_mode & __S_IFMT as uint64_t == 0o20000 as uint64_t)
        {
            size = os_fileinfo_size(&raw mut file_info) as off_T - offset;
        }
        whence = SEEK_SET;
    } else {
        if -offset > file_size
            && !(file_info.stat.st_mode & __S_IFMT as uint64_t == 0o20000 as uint64_t)
        {
            offset = -file_size;
        }
        if size == -1 as off_T || size > -offset {
            size = -offset;
        }
        whence = SEEK_END;
    }
    if size <= 0 as off_T {
        return OK;
    }
    if offset != 0 as off_T && fseeko(fd, offset as __off_t, whence) != 0 as ::core::ffi::c_int {
        return OK;
    }
    ga_grow(&raw mut (*blob).bv_ga, size as ::core::ffi::c_int);
    (*blob).bv_ga.ga_len = size as ::core::ffi::c_int;
    if (fread(
        (*blob).bv_ga.ga_data,
        1 as size_t,
        (*blob).bv_ga.ga_len as size_t,
        fd,
    ) as size_t)
        < (*blob).bv_ga.ga_len as size_t
    {
        tv_blob_free((*rettv).vval.v_blob);
        (*rettv).vval.v_blob = ::core::ptr::null_mut::<blob_T>();
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn read_file_or_blob(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut always_blob: bool,
) {
    let mut binary: bool = false_0 != 0;
    let mut blob: bool = always_blob;
    let mut fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut buf: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut io_size: ::core::ffi::c_int =
        ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as ::core::ffi::c_int;
    let mut prev: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut prevlen: ptrdiff_t = 0 as ptrdiff_t;
    let mut prevsize: ptrdiff_t = 0 as ptrdiff_t;
    let mut maxline: int64_t = MAXLNUM as ::core::ffi::c_int as int64_t;
    let mut offset: off_T = 0 as off_T;
    let mut size: off_T = -1 as off_T;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if always_blob {
            offset = tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as off_T;
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                size = tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize)) as off_T;
            }
        } else {
            if strcmp(
                tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
                c"b".as_ptr(),
            ) == 0 as ::core::ffi::c_int
            {
                binary = true_0 != 0;
            } else if strcmp(
                tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
                c"B".as_ptr(),
            ) == 0 as ::core::ffi::c_int
            {
                blob = true_0 != 0;
            }
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                maxline =
                    tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize)) as int64_t;
            }
        }
    }
    if blob {
        tv_blob_alloc_ret(rettv);
    } else {
        tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    }
    let fname: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    if os_isdir(fname) {
        semsg_c!(
            gettext(&raw const e_isadir2 as *const ::core::ffi::c_char),
            fname,
        );
        return;
    }
    if *fname as ::core::ffi::c_int == NUL || {
        fd = os_fopen(fname, READBIN.as_ptr());
        fd.is_null()
    } {
        semsg_c!(
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            if *fname as ::core::ffi::c_int == NUL {
                gettext(c"<empty>".as_ptr()) as *const ::core::ffi::c_char
            } else {
                fname
            },
        );
        return;
    }
    if blob {
        if read_blob(fd, rettv, offset, size) == FAIL {
            semsg_c!(
                gettext(&raw const e_cant_read_file_str as *const ::core::ffi::c_char),
                fname,
            );
        }
        fclose(fd);
        return;
    }
    let l: *mut list_T = (*rettv).vval.v_list;
    while maxline < 0 as int64_t || (tv_list_len(l) as int64_t) < maxline {
        let mut readlen: ::core::ffi::c_int = fread(
            &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            1 as size_t,
            io_size as size_t,
            fd,
        ) as ::core::ffi::c_int;
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        p = &raw mut buf as *mut ::core::ffi::c_char;
        start = &raw mut buf as *mut ::core::ffi::c_char;
        while p < (&raw mut buf as *mut ::core::ffi::c_char).offset(readlen as isize)
            || readlen <= 0 as ::core::ffi::c_int
                && (prevlen > 0 as ptrdiff_t || binary as ::core::ffi::c_int != 0)
        {
            if readlen <= 0 as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
            {
                let mut s: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut len: size_t = p.offset_from(start) as size_t;
                if readlen > 0 as ::core::ffi::c_int && !binary {
                    while len > 0 as size_t
                        && *start.add(len.wrapping_sub(1 as size_t)) as ::core::ffi::c_int
                            == '\r' as ::core::ffi::c_int
                    {
                        len = len.wrapping_sub(1);
                    }
                    if len == 0 as size_t {
                        while prevlen > 0 as ptrdiff_t
                            && *prev.offset((prevlen - 1 as ptrdiff_t) as isize)
                                as ::core::ffi::c_int
                                == '\r' as ::core::ffi::c_int
                        {
                            prevlen -= 1;
                        }
                    }
                }
                if prevlen == 0 as ptrdiff_t {
                    debug_assert!(
                        len < 2147483647 as ::core::ffi::c_int as size_t,
                        "len < INT_MAX"
                    );
                    s = xmemdupz(start as *const ::core::ffi::c_void, len)
                        as *mut ::core::ffi::c_char;
                } else {
                    s = xrealloc(
                        prev as *mut ::core::ffi::c_void,
                        (prevlen as size_t)
                            .wrapping_add(len)
                            .wrapping_add(1 as size_t),
                    ) as *mut ::core::ffi::c_char;
                    memcpy(
                        s.offset(prevlen as isize) as *mut ::core::ffi::c_void,
                        start as *const ::core::ffi::c_void,
                        len,
                    );
                    *s.add((prevlen as size_t).wrapping_add(len)) = NUL as ::core::ffi::c_char;
                    prev = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    prevsize = 0 as ptrdiff_t;
                    prevlen = prevsize;
                }
                tv_list_append_owned_tv(
                    l,
                    typval_T {
                        v_type: VAR_STRING,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_string: s },
                    },
                );
                start = p.offset(1 as ::core::ffi::c_int as isize);
                if maxline < 0 as int64_t {
                    if tv_list_len(l) as int64_t > -maxline {
                        debug_assert!(
                            tv_list_len(l) as int64_t == 1 as int64_t + -maxline,
                            "tv_list_len(l) == 1 + (-maxline)"
                        );
                        tv_list_item_remove(l, tv_list_first(l));
                    }
                } else if tv_list_len(l) as int64_t >= maxline {
                    debug_assert!(
                        tv_list_len(l) as int64_t == maxline,
                        "tv_list_len(l) == maxline"
                    );
                    break;
                }
                if readlen <= 0 as ::core::ffi::c_int {
                    break;
                }
            } else if *p as ::core::ffi::c_int == NUL {
                *p = '\n' as ::core::ffi::c_char;
            } else if *p as uint8_t as ::core::ffi::c_int == 0xbf as ::core::ffi::c_int && !binary {
                let mut back1: ::core::ffi::c_char = (if p
                    >= (&raw mut buf as *mut ::core::ffi::c_char)
                        .offset(1 as ::core::ffi::c_int as isize)
                {
                    *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                } else if prevlen >= 1 as ptrdiff_t {
                    *prev.offset((prevlen - 1 as ptrdiff_t) as isize) as ::core::ffi::c_int
                } else {
                    NUL
                }) as ::core::ffi::c_char;
                let mut back2: ::core::ffi::c_char = (if p
                    >= (&raw mut buf as *mut ::core::ffi::c_char)
                        .offset(2 as ::core::ffi::c_int as isize)
                {
                    *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                } else if p
                    == (&raw mut buf as *mut ::core::ffi::c_char)
                        .offset(1 as ::core::ffi::c_int as isize)
                    && prevlen >= 1 as ptrdiff_t
                {
                    *prev.offset((prevlen - 1 as ptrdiff_t) as isize) as ::core::ffi::c_int
                } else if prevlen >= 2 as ptrdiff_t {
                    *prev.offset((prevlen - 2 as ptrdiff_t) as isize) as ::core::ffi::c_int
                } else {
                    NUL
                }) as ::core::ffi::c_char;
                if back2 as uint8_t as ::core::ffi::c_int == 0xef as ::core::ffi::c_int
                    && back1 as uint8_t as ::core::ffi::c_int == 0xbb as ::core::ffi::c_int
                {
                    let mut dest: *mut ::core::ffi::c_char =
                        p.offset(-(2 as ::core::ffi::c_int as isize));
                    if start == dest {
                        start = p.offset(1 as ::core::ffi::c_int as isize);
                    } else {
                        let mut adjust_prevlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if dest < &raw mut buf as *mut ::core::ffi::c_char {
                            adjust_prevlen = (&raw mut buf as *mut ::core::ffi::c_char)
                                .offset_from(dest)
                                as ::core::ffi::c_int;
                            dest = &raw mut buf as *mut ::core::ffi::c_char;
                        }
                        if readlen as isize
                            > p.offset_from(&raw mut buf as *mut ::core::ffi::c_char) + 1_isize
                        {
                            memmove(
                                dest as *mut ::core::ffi::c_void,
                                p.offset(1 as ::core::ffi::c_int as isize)
                                    as *const ::core::ffi::c_void,
                                (readlen as size_t)
                                    .wrapping_sub(
                                        p.offset_from(&raw mut buf as *mut ::core::ffi::c_char)
                                            as size_t,
                                    )
                                    .wrapping_sub(1 as size_t),
                            );
                        }
                        readlen -= 3 as ::core::ffi::c_int - adjust_prevlen;
                        prevlen -= adjust_prevlen as ptrdiff_t;
                        p = dest.offset(-(1 as ::core::ffi::c_int as isize));
                    }
                }
            }
            p = p.offset(1);
        }
        if maxline >= 0 as int64_t && tv_list_len(l) as int64_t >= maxline
            || readlen <= 0 as ::core::ffi::c_int
        {
            break;
        }
        if start < p {
            if p.offset_from(start) + prevlen as isize >= prevsize {
                if prevsize == 0 as ptrdiff_t {
                    prevsize = p.offset_from(start) as ptrdiff_t;
                } else {
                    let mut grow50pc: ptrdiff_t = prevsize * 3 as ptrdiff_t / 2 as ptrdiff_t;
                    let mut growmin: ptrdiff_t = p.offset_from(start) * 2 as ptrdiff_t + prevlen;
                    prevsize = if grow50pc > growmin {
                        grow50pc
                    } else {
                        growmin
                    };
                }
                prev = xrealloc(prev as *mut ::core::ffi::c_void, prevsize as size_t)
                    as *mut ::core::ffi::c_char;
            }
            memmove(
                prev.offset(prevlen as isize) as *mut ::core::ffi::c_void,
                start as *const ::core::ffi::c_void,
                p.offset_from(start) as size_t,
            );
            prevlen = (prevlen as ::core::ffi::c_long + p.offset_from(start) as ::core::ffi::c_long)
                as ptrdiff_t;
        }
    }
    xfree(prev as *mut ::core::ffi::c_void);
    fclose(fd);
}
pub unsafe extern "C" fn f_readblob(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    read_file_or_blob(argvars, rettv, true_0 != 0);
}
pub unsafe extern "C" fn f_readfile(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    read_file_or_blob(argvars, rettv, false_0 != 0);
}
pub unsafe extern "C" fn f_rename(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
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
pub unsafe extern "C" fn f_resolve(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    let mut fname: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut is_relative_to_current: bool = false_0 != 0;
    let mut has_trailing_pathsep: bool = false_0 != 0;
    let mut limit: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = xstrdup(fname);
    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '.' as ::core::ffi::c_int
        && (vim_ispathsep(*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && vim_ispathsep(*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0)
    {
        is_relative_to_current = true_0 != 0;
    }
    let mut len: ptrdiff_t = strlen(p) as ptrdiff_t;
    if len > 1 as ptrdiff_t && after_pathsep(p, p.offset(len as isize)) != 0 {
        has_trailing_pathsep = true_0 != 0;
        *p.offset((len - 1 as ptrdiff_t) as isize) = NUL as ::core::ffi::c_char;
    }
    let mut q: *mut ::core::ffi::c_char = path_next_component(p) as *mut ::core::ffi::c_char;
    let mut remain: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *q as ::core::ffi::c_int != NUL {
        remain = xstrdup(q.offset(-(1 as ::core::ffi::c_int as isize)));
        *q.offset(-1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
    }
    let buf: *mut ::core::ffi::c_char = xmallocz(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    let mut cpy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    loop {
        loop {
            len = readlink(p, buf, MAXPATHL as size_t) as ptrdiff_t;
            if len <= 0 as ptrdiff_t {
                break;
            }
            *buf.offset(len as isize) = NUL as ::core::ffi::c_char;
            let c2rust_fresh1 = limit;
            limit = limit - 1;
            if c2rust_fresh1 == 0 as ::core::ffi::c_int {
                xfree(p as *mut ::core::ffi::c_void);
                xfree(remain as *mut ::core::ffi::c_void);
                emsg(gettext(c"E655: Too many symbolic links (cycle?)".as_ptr()));
                (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
                xfree(buf as *mut ::core::ffi::c_void);
                return;
            }
            if remain.is_null() && has_trailing_pathsep as ::core::ffi::c_int != 0 {
                add_pathsep(buf);
            }
            q = path_next_component(
                if vim_ispathsep(*buf as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
                    buf.offset(1 as ::core::ffi::c_int as isize)
                } else {
                    buf
                },
            ) as *mut ::core::ffi::c_char;
            if *q as ::core::ffi::c_int != NUL {
                cpy = remain;
                remain = if !remain.is_null() {
                    concat_str(q.offset(-(1 as ::core::ffi::c_int as isize)), remain)
                } else {
                    xstrdup(q.offset(-(1 as ::core::ffi::c_int as isize)))
                };
                xfree(cpy as *mut ::core::ffi::c_void);
                *q.offset(-1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
            }
            q = path_tail(p);
            if q > p && *q as ::core::ffi::c_int == NUL {
                *p.offset(q.offset_from(p) - 1) = NUL as ::core::ffi::c_char;
                q = path_tail(p);
            }
            if q > p && !path_is_absolute(buf) {
                let p_len: size_t = strlen(p);
                let buf_len: size_t = strlen(buf);
                p = xrealloc(
                    p as *mut ::core::ffi::c_void,
                    p_len.wrapping_add(buf_len).wrapping_add(1 as size_t),
                ) as *mut ::core::ffi::c_char;
                memcpy(
                    path_tail(p) as *mut ::core::ffi::c_void,
                    buf as *const ::core::ffi::c_void,
                    buf_len.wrapping_add(1 as size_t),
                );
            } else {
                xfree(p as *mut ::core::ffi::c_void);
                p = xstrdup(buf);
            }
        }
        if remain.is_null() {
            break;
        }
        q = path_next_component(remain.offset(1 as ::core::ffi::c_int as isize))
            as *mut ::core::ffi::c_char;
        len = (q.offset_from(remain)
            - (*q as ::core::ffi::c_int != NUL) as ::core::ffi::c_int as isize)
            as ptrdiff_t;
        let p_len_0: size_t = strlen(p);
        cpy = xmallocz(p_len_0.wrapping_add(len as size_t)) as *mut ::core::ffi::c_char;
        memcpy(
            cpy as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            p_len_0.wrapping_add(1 as size_t),
        );
        xstrlcat(
            cpy.add(p_len_0),
            remain,
            (len as size_t).wrapping_add(1 as size_t),
        );
        xfree(p as *mut ::core::ffi::c_void);
        p = cpy;
        if *q as ::core::ffi::c_int != NUL {
            memmove(
                remain as *mut ::core::ffi::c_void,
                q.offset(-(1 as ::core::ffi::c_int as isize)) as *const ::core::ffi::c_void,
                strlen(q.offset(-(1 as ::core::ffi::c_int as isize))).wrapping_add(1 as size_t),
            );
        } else {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut remain as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
        }
    }
    if !vim_ispathsep(*p as ::core::ffi::c_int) {
        if is_relative_to_current as ::core::ffi::c_int != 0
            && *p as ::core::ffi::c_int != NUL
            && !(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || vim_ispathsep(
                        *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                    || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '.' as ::core::ffi::c_int
                        && (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == NUL
                            || vim_ispathsep(
                                *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            ) as ::core::ffi::c_int
                                != 0)))
        {
            cpy = concat_str(c"./".as_ptr(), p);
            xfree(p as *mut ::core::ffi::c_void);
            p = cpy;
        } else if !is_relative_to_current {
            q = p;
            while *q.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && vim_ispathsep(*q.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
            {
                q = q.offset(2 as ::core::ffi::c_int as isize);
            }
            if q > p {
                memmove(
                    p as *mut ::core::ffi::c_void,
                    p.offset(2 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    strlen(p.offset(2 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
                );
            }
        }
    }
    if !has_trailing_pathsep {
        q = p.add(strlen(p));
        if after_pathsep(p, q) != 0 {
            *path_tail_with_sep(p) = NUL as ::core::ffi::c_char;
        }
    }
    (*rettv).vval.v_string = p;
    xfree(buf as *mut ::core::ffi::c_void);
    simplify_filename((*rettv).vval.v_string);
}
pub unsafe extern "C" fn f_simplify(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let p: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).vval.v_string = xstrdup(p);
    simplify_filename((*rettv).vval.v_string);
    (*rettv).v_type = VAR_STRING;
}
pub unsafe extern "C" fn f_tempname(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = vim_tempname();
}
unsafe extern "C" fn write_list(
    fp: *mut FileDescriptor,
    list: *const list_T,
    binary: bool,
) -> bool {
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
                    let s: *const ::core::ffi::c_char = tv_get_string_chk(&raw const (*li).li_tv);
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
                                let written: ptrdiff_t =
                                    file_write(fp, hunk_start, p.offset_from(hunk_start) as size_t);
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
        gettext((e_error_while_writing_str.ptr() as *const _) as *const ::core::ffi::c_char),
        uv_strerror(error),
    );
    return false_0 != 0;
}
unsafe extern "C" fn write_data(
    fp: *mut FileDescriptor,
    data: *const ::core::ffi::c_char,
    len: size_t,
) -> bool {
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
        gettext((e_error_while_writing_str.ptr() as *const _) as *const ::core::ffi::c_char),
        uv_strerror(error),
    );
    return false_0 != 0;
}
unsafe extern "C" fn write_blob(fp: *mut FileDescriptor, blob: *const blob_T) -> bool {
    return write_data(
        fp,
        (*blob).bv_ga.ga_data as *const ::core::ffi::c_char,
        tv_blob_len(blob) as size_t,
    );
}
unsafe extern "C" fn write_string(
    fp: *mut FileDescriptor,
    data: *const ::core::ffi::c_char,
) -> bool {
    return write_data(fp, data, strlen(data));
}
pub unsafe extern "C" fn f_writefile(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
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
pub unsafe extern "C" fn f_browse(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*rettv).v_type = VAR_STRING;
}
pub unsafe extern "C" fn f_browsedir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    f_browse(argvars, rettv, fptr);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
pub const READBIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"rb\0") };
