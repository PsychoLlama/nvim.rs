use crate::src::nvim::api::private::helpers::{
    cbuf_to_string, copy_string, cstr_as_string, cstr_to_string,
};
use crate::src::nvim::autocmd::{EVENT_DIRCHANGED, EVENT_DIRCHANGEDPRE, apply_autocmds, has_event};
use crate::src::nvim::charset::{getdigits_int32, getdigits_long, skipwhite, vim_isfilec};
use crate::src::nvim::cursor::get_cursor_line_ptr;
use crate::src::nvim::eval::typval::{
    tv_dict_add_bool, tv_dict_add_str, tv_dict_set_keys_readonly,
};
use crate::src::nvim::eval::vars::set_vim_var_string;
use crate::src::nvim::eval::{eval_to_string_safe, get_v_event, restore_v_event};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    NameBuff, VIsual_active, curbuf, current_sctx, curwin, e_cant_find_directory_str_in_cdpath,
    e_cant_find_file_str_in_path, e_no_more_directory_str_found_in_cdpath,
    e_no_more_file_str_found_in_path, got_int, line_msg, p_cdpath, p_cpo, p_fic, p_path,
};
use crate::src::nvim::mbyte::{mb_tolower, utf_head_off, utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xmemcpyz, xmemdupz, xrealloc, xstrlcpy};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::normal::get_visual_text;
use crate::src::nvim::option::{copy_option_part, was_set_insecurely};
use crate::src::nvim::options::kOptIncludeexpr;
use crate::src::nvim::os::env::expand_env_esc;
use crate::src::nvim::os::fs::{
    os_chdir, os_dirname, os_fileid, os_fileid_equal, os_isdir, os_path_exists,
};
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, __ctype_b_loc, abort, gettext, memmove, strcpy, strlen, strncmp, strtol,
};
use crate::src::nvim::path::{
    FreeWild, FullName_save, after_pathsep, expand_wildcards, path_fnamecmp, path_fnamencmp,
    path_has_drive_letter, path_is_url, path_shorten_fname, path_tail, path_tail_with_sep,
    path_with_url, pathcmp, simplify_filename, vim_isAbsName, vim_ispathsep,
};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr, xstrnsave};
use crate::src::nvim::types::{
    Arena, BoolVarValue, CdCause, CdScope, FileID, String_0, VimVarIndex, cmdarg_T, event_T,
    int32_t, int64_t, linenr_T, ptrdiff_t, save_v_event_T, sctx_T, size_t, uint8_t,
};

// The carve of the transpiled module; see each child's docs.
mod init;
pub use self::init::*;
mod visited;
pub(crate) use self::visited::*;
mod resolve;
pub use self::resolve::*;
mod cursor;
pub use self::cursor::*;
mod chdir;
pub use self::chdir::*;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const kCdScopeGlobal: CdScope = 2;
pub const kCdScopeTabpage: CdScope = 1;
pub const kCdScopeWindow: CdScope = 0;
pub const kCdScopeInvalid: CdScope = -1;
pub const kCdCauseAuto: CdCause = 2;
pub const kCdCauseWindow: CdCause = 1;
pub const kCdCauseOther: CdCause = -1;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_int;
pub const kBufOptIncludeexpr: C2Rust_Unnamed_13 = 46;
pub const VV_FNAME: VimVarIndex = 12;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const FINDFILE_BOTH: C2Rust_Unnamed_14 = 2;
pub const FINDFILE_DIR: C2Rust_Unnamed_14 = 1;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const FNAME_UNESC: C2Rust_Unnamed_15 = 32;
pub const FNAME_REL: C2Rust_Unnamed_15 = 16;
pub const FNAME_INCL: C2Rust_Unnamed_15 = 8;
pub const FNAME_HYP: C2Rust_Unnamed_15 = 4;
pub const FNAME_EXP: C2Rust_Unnamed_15 = 2;
pub const FNAME_MESS: C2Rust_Unnamed_15 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ff_search_ctx_T {
    pub ffsc_stack_ptr: *mut ff_stack_T,
    pub ffsc_visited_list: *mut ff_visited_list_hdr_T,
    pub ffsc_dir_visited_list: *mut ff_visited_list_hdr_T,
    pub ffsc_visited_lists_list: *mut ff_visited_list_hdr_T,
    pub ffsc_dir_visited_lists_list: *mut ff_visited_list_hdr_T,
    pub ffsc_file_to_search: String_0,
    pub ffsc_start_dir: String_0,
    pub ffsc_fix_path: String_0,
    pub ffsc_wc_path: String_0,
    pub ffsc_level: ::core::ffi::c_int,
    pub ffsc_stopdirs_v: *mut String_0,
    pub ffsc_find_what: ::core::ffi::c_int,
    pub ffsc_tagfile: ::core::ffi::c_int,
}
pub type ff_visited_list_hdr_T = ff_visited_list_hdr;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ff_visited_list_hdr {
    pub ffvl_next: *mut ff_visited_list_hdr,
    pub ffvl_filename: *mut ::core::ffi::c_char,
    pub ffvl_visited_list: *mut ff_visited_T,
}
pub type ff_visited_T = ff_visited;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ff_visited {
    pub ffv_next: *mut ff_visited,
    pub ffv_wc_path: *mut ::core::ffi::c_char,
    pub file_id_valid: bool,
    pub file_id: FileID,
    pub ffv_fname: [::core::ffi::c_char; 0],
}
pub type ff_stack_T = ff_stack;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ff_stack {
    pub ffs_prev: *mut ff_stack,
    pub ffs_fix_path: String_0,
    pub ffs_wc_path: String_0,
    pub ffs_filearray: *mut *mut ::core::ffi::c_char,
    pub ffs_filearray_size: ::core::ffi::c_int,
    pub ffs_filearray_cur: ::core::ffi::c_int,
    pub ffs_stage: ::core::ffi::c_int,
    pub ffs_level: ::core::ffi::c_int,
    pub ffs_star_star_empty: ::core::ffi::c_int,
}
pub const EW_NOTWILD: C2Rust_Unnamed_17 = 1024;
pub const EW_SILENT: C2Rust_Unnamed_17 = 32;
pub const EW_ADDSLASH: C2Rust_Unnamed_17 = 8;
pub const EW_DIR: C2Rust_Unnamed_17 = 1;
pub const OPT_LOCAL: C2Rust_Unnamed_16 = 2;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
/// The longest path name the searcher will build, buffers included.
pub const MAXPATHL: usize = 4096;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NULL_STRING: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
pub const PATHSEPSTR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"/\0") };
pub const CPO_DOTTAG: ::core::ffi::c_int = 'd' as ::core::ffi::c_int;
static ff_expand_buffer: GlobalCell<String_0> = GlobalCell::new(String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
});
pub const FF_MAX_STAR_STAR_EXPAND: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
static e_path_too_long_for_completion: GlobalCell<[::core::ffi::c_char; 35]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
            *b"E854: Path too long for completion\0",
        )
    });
pub unsafe extern "C" fn vim_findfile(
    mut search_ctx_arg: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_char {
    let mut rest_of_wildcards: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut path_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut stackp: *mut ff_stack_T = ::core::ptr::null_mut::<ff_stack_T>();
    if search_ctx_arg.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut search_ctx: *mut ff_search_ctx_T = search_ctx_arg as *mut ff_search_ctx_T;
    let mut file_path: String_0 = String_0 {
        data: xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char,
        size: 0,
    };
    if !(*search_ctx).ffsc_start_dir.data.is_null() {
        path_end = (*search_ctx)
            .ffsc_start_dir
            .data
            .offset((*search_ctx).ffsc_start_dir.size as isize);
    }
    '_fail: loop {
        os_breakcheck();
        if !got_int.get() {
            stackp = ff_pop(search_ctx);
            if !stackp.is_null() {
                if (*stackp).ffs_filearray.is_null()
                    && ff_check_visited(
                        &raw mut (*(*search_ctx).ffsc_dir_visited_list).ffvl_visited_list,
                        (*stackp).ffs_fix_path.data,
                        (*stackp).ffs_fix_path.size,
                        (*stackp).ffs_wc_path.data,
                        (*stackp).ffs_wc_path.size,
                    ) == FAIL
                {
                    ff_free_stack_element(stackp);
                    continue;
                } else if (*stackp).ffs_level <= 0 as ::core::ffi::c_int {
                    ff_free_stack_element(stackp);
                    continue;
                } else {
                    *file_path.data.offset(0 as ::core::ffi::c_int as isize) =
                        NUL as ::core::ffi::c_char;
                    file_path.size = 0 as size_t;
                    if (*stackp).ffs_filearray.is_null() {
                        let mut dirptrs: [*mut ::core::ffi::c_char; 2] =
                            [::core::ptr::null_mut::<::core::ffi::c_char>(); 2];
                        dirptrs[0 as ::core::ffi::c_int as usize] = file_path.data;
                        dirptrs[1 as ::core::ffi::c_int as usize] =
                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                        if !vim_isAbsName((*stackp).ffs_fix_path.data)
                            && !(*search_ctx).ffsc_start_dir.data.is_null()
                        {
                            if (*search_ctx).ffsc_start_dir.size.wrapping_add(1 as size_t)
                                >= MAXPATHL as size_t
                            {
                                ff_free_stack_element(stackp);
                                break;
                            } else {
                                let mut add_sep: bool = after_pathsep(
                                    (*search_ctx).ffsc_start_dir.data,
                                    (*search_ctx)
                                        .ffsc_start_dir
                                        .data
                                        .offset((*search_ctx).ffsc_start_dir.size as isize),
                                ) == 0;
                                file_path.size = vim_snprintf(
                                    file_path.data,
                                    MAXPATHL as size_t,
                                    b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    (*search_ctx).ffsc_start_dir.data,
                                    if add_sep as ::core::ffi::c_int != 0 {
                                        PATHSEPSTR.as_ptr()
                                    } else {
                                        b"\0".as_ptr() as *const ::core::ffi::c_char
                                    },
                                ) as size_t;
                                if file_path.size >= MAXPATHL as size_t {
                                    ff_free_stack_element(stackp);
                                    break;
                                }
                            }
                        }
                        if file_path
                            .size
                            .wrapping_add((*stackp).ffs_fix_path.size)
                            .wrapping_add(1 as size_t)
                            >= MAXPATHL as size_t
                        {
                            ff_free_stack_element(stackp);
                            break;
                        } else {
                            let mut add_sep_0: bool = after_pathsep(
                                (*stackp).ffs_fix_path.data,
                                (*stackp)
                                    .ffs_fix_path
                                    .data
                                    .offset((*stackp).ffs_fix_path.size as isize),
                            ) == 0;
                            file_path.size = file_path.size.wrapping_add(vim_snprintf(
                                file_path.data.offset(file_path.size as isize),
                                (MAXPATHL as size_t).wrapping_sub(file_path.size),
                                b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                                (*stackp).ffs_fix_path.data,
                                if add_sep_0 as ::core::ffi::c_int != 0 {
                                    PATHSEPSTR.as_ptr()
                                } else {
                                    b"\0".as_ptr() as *const ::core::ffi::c_char
                                },
                            )
                                as size_t);
                            if file_path.size >= MAXPATHL as size_t {
                                ff_free_stack_element(stackp);
                                break;
                            } else {
                                rest_of_wildcards = (*stackp).ffs_wc_path;
                                if *rest_of_wildcards.data as ::core::ffi::c_int != NUL {
                                    if strncmp(
                                        rest_of_wildcards.data,
                                        b"**\0".as_ptr() as *const ::core::ffi::c_char,
                                        2 as size_t,
                                    ) == 0 as ::core::ffi::c_int
                                    {
                                        let mut p: *mut ::core::ffi::c_char = rest_of_wildcards
                                            .data
                                            .offset(2 as ::core::ffi::c_int as isize);
                                        if *p as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
                                            *p -= 1;
                                            if file_path.size.wrapping_add(1 as size_t)
                                                >= MAXPATHL as size_t
                                            {
                                                ff_free_stack_element(stackp);
                                                break;
                                            } else {
                                                let c2rust_fresh11 = file_path.size;
                                                file_path.size = file_path.size.wrapping_add(1);
                                                *file_path.data.offset(c2rust_fresh11 as isize) =
                                                    '*' as ::core::ffi::c_char;
                                            }
                                        }
                                        if *p as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                                            memmove(
                                                rest_of_wildcards.data as *mut ::core::ffi::c_void,
                                                rest_of_wildcards
                                                    .data
                                                    .offset(3 as ::core::ffi::c_int as isize)
                                                    as *const ::core::ffi::c_void,
                                                rest_of_wildcards
                                                    .size
                                                    .wrapping_sub(3 as size_t)
                                                    .wrapping_add(1 as size_t),
                                            );
                                            rest_of_wildcards.size =
                                                rest_of_wildcards.size.wrapping_sub(3 as size_t);
                                            (*stackp).ffs_wc_path.size = rest_of_wildcards.size;
                                        } else {
                                            rest_of_wildcards.data = rest_of_wildcards
                                                .data
                                                .offset(3 as ::core::ffi::c_int as isize);
                                            rest_of_wildcards.size =
                                                rest_of_wildcards.size.wrapping_sub(3 as size_t);
                                        }
                                        if (*stackp).ffs_star_star_empty == 0 as ::core::ffi::c_int
                                        {
                                            (*stackp).ffs_star_star_empty = 1 as ::core::ffi::c_int;
                                            dirptrs[1 as ::core::ffi::c_int as usize] =
                                                (*stackp).ffs_fix_path.data;
                                        }
                                    }
                                    while *rest_of_wildcards.data as ::core::ffi::c_int != 0
                                        && !vim_ispathsep(
                                            *rest_of_wildcards.data as ::core::ffi::c_int,
                                        )
                                    {
                                        if file_path.size.wrapping_add(1 as size_t)
                                            >= MAXPATHL as size_t
                                        {
                                            ff_free_stack_element(stackp);
                                            break '_fail;
                                        } else {
                                            let c2rust_fresh12 = rest_of_wildcards.data;
                                            rest_of_wildcards.data =
                                                rest_of_wildcards.data.offset(1);
                                            let c2rust_fresh13 = file_path.size;
                                            file_path.size = file_path.size.wrapping_add(1);
                                            *file_path.data.offset(c2rust_fresh13 as isize) =
                                                *c2rust_fresh12;
                                            rest_of_wildcards.size =
                                                rest_of_wildcards.size.wrapping_sub(1);
                                        }
                                    }
                                    *file_path.data.offset(file_path.size as isize) =
                                        NUL as ::core::ffi::c_char;
                                    if vim_ispathsep(*rest_of_wildcards.data as ::core::ffi::c_int)
                                    {
                                        rest_of_wildcards.data = rest_of_wildcards.data.offset(1);
                                        rest_of_wildcards.size =
                                            rest_of_wildcards.size.wrapping_sub(1);
                                    }
                                }
                                if path_with_url(dirptrs[0 as ::core::ffi::c_int as usize]) != 0 {
                                    (*stackp).ffs_filearray =
                                        xmalloc(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                                            as *mut *mut ::core::ffi::c_char;
                                    *(*stackp)
                                        .ffs_filearray
                                        .offset(0 as ::core::ffi::c_int as isize) = xmemdupz(
                                        dirptrs[0 as ::core::ffi::c_int as usize]
                                            as *const ::core::ffi::c_void,
                                        file_path.size,
                                    )
                                        as *mut ::core::ffi::c_char;
                                    (*stackp).ffs_filearray_size = 1 as ::core::ffi::c_int;
                                } else {
                                    expand_wildcards(
                                        if dirptrs[1 as ::core::ffi::c_int as usize].is_null() {
                                            1 as ::core::ffi::c_int
                                        } else {
                                            2 as ::core::ffi::c_int
                                        },
                                        &raw mut dirptrs as *mut *mut ::core::ffi::c_char,
                                        &raw mut (*stackp).ffs_filearray_size,
                                        &raw mut (*stackp).ffs_filearray,
                                        EW_DIR as ::core::ffi::c_int
                                            | EW_ADDSLASH as ::core::ffi::c_int
                                            | EW_SILENT as ::core::ffi::c_int
                                            | EW_NOTWILD as ::core::ffi::c_int,
                                    );
                                }
                                (*stackp).ffs_filearray_cur = 0 as ::core::ffi::c_int;
                                (*stackp).ffs_stage = 0 as ::core::ffi::c_int;
                            }
                        }
                    } else {
                        rest_of_wildcards.data = (*stackp)
                            .ffs_wc_path
                            .data
                            .offset((*stackp).ffs_wc_path.size as isize);
                        rest_of_wildcards.size = 0 as size_t;
                    }
                    if (*stackp).ffs_stage == 0 as ::core::ffi::c_int {
                        's_500: {
                            if *rest_of_wildcards.data as ::core::ffi::c_int == NUL {
                                let mut i: ::core::ffi::c_int = (*stackp).ffs_filearray_cur;
                                loop {
                                    if i >= (*stackp).ffs_filearray_size {
                                        break 's_500;
                                    }
                                    if !(path_with_url(*(*stackp).ffs_filearray.offset(i as isize))
                                        == 0
                                        && !os_isdir(*(*stackp).ffs_filearray.offset(i as isize)))
                                    {
                                        let mut len: size_t =
                                            strlen(*(*stackp).ffs_filearray.offset(i as isize));
                                        if len
                                            .wrapping_add(1 as size_t)
                                            .wrapping_add((*search_ctx).ffsc_file_to_search.size)
                                            >= MAXPATHL as size_t
                                        {
                                            ff_free_stack_element(stackp);
                                            break '_fail;
                                        } else {
                                            let mut add_sep_1: bool = after_pathsep(
                                                *(*stackp).ffs_filearray.offset(i as isize),
                                                (*(*stackp).ffs_filearray.offset(i as isize))
                                                    .offset(len as isize),
                                            ) == 0;
                                            file_path.size = vim_snprintf(
                                                file_path.data,
                                                MAXPATHL as size_t,
                                                b"%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                                                *(*stackp).ffs_filearray.offset(i as isize),
                                                if add_sep_1 as ::core::ffi::c_int != 0 {
                                                    PATHSEPSTR.as_ptr()
                                                } else {
                                                    b"\0".as_ptr() as *const ::core::ffi::c_char
                                                },
                                                (*search_ctx).ffsc_file_to_search.data,
                                            )
                                                as size_t;
                                            if file_path.size >= MAXPATHL as size_t {
                                                ff_free_stack_element(stackp);
                                                break '_fail;
                                            } else {
                                                len = file_path.size;
                                                let mut suf: *mut ::core::ffi::c_char =
                                                    (if (*search_ctx).ffsc_tagfile != 0 {
                                                        b"\0".as_ptr() as *const ::core::ffi::c_char
                                                    } else {
                                                        (*curbuf.get()).b_p_sua
                                                            as *const ::core::ffi::c_char
                                                    })
                                                        as *mut ::core::ffi::c_char;
                                                loop {
                                                    if (path_with_url(file_path.data) != 0
                                                        || os_path_exists(file_path.data)
                                                            as ::core::ffi::c_int
                                                            != 0
                                                            && ((*search_ctx).ffsc_find_what
                                                                == FINDFILE_BOTH
                                                                    as ::core::ffi::c_int
                                                                || ((*search_ctx).ffsc_find_what
                                                                    == FINDFILE_DIR
                                                                        as ::core::ffi::c_int)
                                                                    as ::core::ffi::c_int
                                                                    == os_isdir(file_path.data)
                                                                        as ::core::ffi::c_int))
                                                        && ff_check_visited(
                                                            &raw mut (*(*search_ctx)
                                                                .ffsc_visited_list)
                                                                .ffvl_visited_list,
                                                            file_path.data,
                                                            file_path.size,
                                                            b"\0".as_ptr()
                                                                as *const ::core::ffi::c_char
                                                                as *mut ::core::ffi::c_char,
                                                            0 as size_t,
                                                        ) == OK
                                                    {
                                                        '_c2rust_label: {
                                                            if i < 2147483647 as ::core::ffi::c_int
                                                            {
                                                            } else {
                                                                __assert_fail(
                                                                    b"i < INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                                                                    b"src/nvim/file_search.rs\0"
                                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                                    875 as ::core::ffi::c_uint,
                                                                    b"char *vim_findfile(void *)\0".as_ptr()
                                                                        as *const ::core::ffi::c_char,
                                                                );
                                                            }
                                                        };
                                                        (*stackp).ffs_filearray_cur =
                                                            i + 1 as ::core::ffi::c_int;
                                                        ff_push(search_ctx, stackp);
                                                        if path_with_url(file_path.data) == 0 {
                                                            file_path.size =
                                                                simplify_filename(file_path.data);
                                                        }
                                                        if os_dirname(
                                                            (*ff_expand_buffer.ptr()).data,
                                                            MAXPATHL as size_t,
                                                        ) == OK
                                                        {
                                                            (*ff_expand_buffer.ptr()).size = strlen(
                                                                (*ff_expand_buffer.ptr()).data,
                                                            );
                                                            let mut p_0: *mut ::core::ffi::c_char =
                                                                path_shorten_fname(
                                                                    file_path.data,
                                                                    (*ff_expand_buffer.ptr()).data,
                                                                );
                                                            if !p_0.is_null() {
                                                                memmove(
                                                                    file_path.data as *mut ::core::ffi::c_void,
                                                                    p_0 as *const ::core::ffi::c_void,
                                                                    (file_path
                                                                        .data
                                                                        .offset(file_path.size as isize)
                                                                        .offset_from(p_0) as size_t)
                                                                        .wrapping_add(1 as size_t),
                                                                );
                                                                file_path.size =
                                                                    file_path.size.wrapping_sub(
                                                                        p_0.offset_from(
                                                                            file_path.data,
                                                                        )
                                                                            as size_t,
                                                                    );
                                                            }
                                                        }
                                                        return file_path.data;
                                                    }
                                                    if *suf as ::core::ffi::c_int == NUL {
                                                        break;
                                                    }
                                                    '_c2rust_label_0: {
                                                        if 4096 as size_t >= file_path.size {
                                                        } else {
                                                            __assert_fail(
                                                                b"MAXPATHL >= file_path.size\0"
                                                                    .as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                                b"src/nvim/file_search.rs\0"
                                                                    .as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                                907 as ::core::ffi::c_uint,
                                                                b"char *vim_findfile(void *)\0"
                                                                    .as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                            );
                                                        }
                                                    };
                                                    file_path.size =
                                                        len.wrapping_add(copy_option_part(
                                                            &raw mut suf,
                                                            file_path.data.offset(len as isize),
                                                            (MAXPATHL as size_t).wrapping_sub(len),
                                                            b",\0".as_ptr()
                                                                as *const ::core::ffi::c_char
                                                                as *mut ::core::ffi::c_char,
                                                        ));
                                                }
                                            }
                                        }
                                    }
                                    i += 1;
                                }
                            } else {
                                let mut i_0: ::core::ffi::c_int = (*stackp).ffs_filearray_cur;
                                while i_0 < (*stackp).ffs_filearray_size {
                                    if os_isdir(*(*stackp).ffs_filearray.offset(i_0 as isize)) {
                                        ff_push(
                                            search_ctx,
                                            ff_create_stack_element(
                                                *(*stackp).ffs_filearray.offset(i_0 as isize),
                                                strlen(
                                                    *(*stackp).ffs_filearray.offset(i_0 as isize),
                                                ),
                                                rest_of_wildcards.data,
                                                rest_of_wildcards.size,
                                                (*stackp).ffs_level - 1 as ::core::ffi::c_int,
                                                0 as ::core::ffi::c_int,
                                            ),
                                        );
                                    }
                                    i_0 += 1;
                                }
                            }
                        }
                        (*stackp).ffs_filearray_cur = 0 as ::core::ffi::c_int;
                        (*stackp).ffs_stage = 1 as ::core::ffi::c_int;
                    }
                    if strncmp(
                        (*stackp).ffs_wc_path.data,
                        b"**\0".as_ptr() as *const ::core::ffi::c_char,
                        2 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        let mut i_1: ::core::ffi::c_int = (*stackp).ffs_filearray_cur;
                        while i_1 < (*stackp).ffs_filearray_size {
                            if path_fnamecmp(
                                *(*stackp).ffs_filearray.offset(i_1 as isize),
                                (*stackp).ffs_fix_path.data,
                            ) != 0 as ::core::ffi::c_int
                            {
                                if os_isdir(*(*stackp).ffs_filearray.offset(i_1 as isize)) {
                                    ff_push(
                                        search_ctx,
                                        ff_create_stack_element(
                                            *(*stackp).ffs_filearray.offset(i_1 as isize),
                                            strlen(*(*stackp).ffs_filearray.offset(i_1 as isize)),
                                            (*stackp).ffs_wc_path.data,
                                            (*stackp).ffs_wc_path.size,
                                            (*stackp).ffs_level - 1 as ::core::ffi::c_int,
                                            1 as ::core::ffi::c_int,
                                        ),
                                    );
                                }
                            }
                            i_1 += 1;
                        }
                    }
                    ff_free_stack_element(stackp);
                    continue;
                }
            }
        }
        if !(!(*search_ctx).ffsc_start_dir.data.is_null()
            && !(*search_ctx).ffsc_stopdirs_v.is_null()
            && !got_int.get())
        {
            break;
        }
        let mut sptr: *mut ff_stack_T = ::core::ptr::null_mut::<ff_stack_T>();
        let mut plen: ptrdiff_t = path_end.offset_from((*search_ctx).ffsc_start_dir.data)
            + (*path_end as ::core::ffi::c_int != NUL) as ::core::ffi::c_int as ptrdiff_t;
        if ff_path_in_stoplist(
            (*search_ctx).ffsc_start_dir.data,
            plen as size_t,
            (*search_ctx).ffsc_stopdirs_v,
        ) {
            break;
        }
        while path_end > (*search_ctx).ffsc_start_dir.data
            && vim_ispathsep(*path_end as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        {
            path_end = path_end.offset(-1);
        }
        while path_end > (*search_ctx).ffsc_start_dir.data
            && !vim_ispathsep(
                *path_end.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            )
        {
            path_end = path_end.offset(-1);
        }
        *path_end = NUL as ::core::ffi::c_char;
        (*search_ctx).ffsc_start_dir.size =
            path_end.offset_from((*search_ctx).ffsc_start_dir.data) as size_t;
        path_end = path_end.offset(-1);
        if *(*search_ctx).ffsc_start_dir.data as ::core::ffi::c_int == NUL {
            break;
        }
        if (*search_ctx)
            .ffsc_start_dir
            .size
            .wrapping_add(1 as size_t)
            .wrapping_add((*search_ctx).ffsc_fix_path.size)
            >= MAXPATHL as size_t
        {
            break;
        }
        let mut add_sep_2: bool = after_pathsep(
            (*search_ctx).ffsc_start_dir.data,
            (*search_ctx)
                .ffsc_start_dir
                .data
                .offset((*search_ctx).ffsc_start_dir.size as isize),
        ) == 0;
        file_path.size = vim_snprintf(
            file_path.data,
            MAXPATHL as size_t,
            b"%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
            (*search_ctx).ffsc_start_dir.data,
            if add_sep_2 as ::core::ffi::c_int != 0 {
                PATHSEPSTR.as_ptr()
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            (*search_ctx).ffsc_fix_path.data,
        ) as size_t;
        if file_path.size >= MAXPATHL as size_t {
            break;
        }
        sptr = ff_create_stack_element(
            file_path.data,
            file_path.size,
            (*search_ctx).ffsc_wc_path.data,
            (*search_ctx).ffsc_wc_path.size,
            (*search_ctx).ffsc_level,
            0 as ::core::ffi::c_int,
        );
        ff_push(search_ctx, sptr);
    }
    xfree(file_path.data as *mut ::core::ffi::c_void);
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
