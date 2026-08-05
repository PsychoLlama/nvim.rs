use crate::src::nvim::api::private::helpers::{api_clear_error, api_free_object, cstr_as_string};
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::buffer::{bt_help, buflist_findnr, set_buflisted, wipe_buffer};
use crate::src::nvim::charset::{buf_init_chartab, skipwhite};
use crate::src::nvim::cmdexpand::{ExpandInit, ExpandOne};
use crate::src::nvim::ex_cmds::do_ecmd;
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::fileio::vim_fgets;
use crate::src::nvim::options::{kOptBuftype, kOptFoldmethod, kOptIskeyword};

use crate::src::nvim::garray::{ga_clear, ga_grow, ga_init};
use crate::src::nvim::highlight_group::HLF_E;
use crate::src::nvim::lua::executor::nlua_exec;
use crate::src::nvim::main::{
    Columns, IObuff, KeyTyped, NameBuff, cmdmod, curbuf, curtab, curwin, e_fnametoolong, e_noident,
    firstwin, got_int, p_hf, p_hh, p_hlg, p_rtp, p_sb, restart_edit,
};
use crate::src::nvim::memory::{xfree, xmalloc, xstrdup, xstrlcat, xstrlcpy};
use crate::src::nvim::message::{emsg, emsg_multiline, semsg, smsg};
use crate::src::nvim::option::set_option_direct;
use crate::src::nvim::optionstr::check_buf_options;
use crate::src::nvim::os::fs::{os_fopen, os_isdir};
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, fclose, fprintf, fputs, gettext, memcpy, putc, qsort, snprintf, strcasecmp,
    strchr, strcmp, strcpy, strlen, strncmp,
};
use crate::src::nvim::path::{FreeWild, add_pathsep, gen_expand_wildcards, path_full_compare};
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::runtime::{DIP_ALL, DIP_DIR, do_in_path};
use crate::src::nvim::strings::{sort_strings, vim_snprintf, vim_strchr};
use crate::src::nvim::tag::{do_tag, find_tags};
use crate::src::nvim::types::{
    Arena, Array, CMOD_KEEPALT, Direction, Error, FILE, LuaRetMode, Object, OptInt, OptVal,
    OptValData, OptValType, String_0, buf_T, exarg_T, expand_T, file_comparison, garray_T,
    kErrorTypeNone, kObjectTypeNil, kObjectTypeString, linenr_T, object,
    object_data as C2Rust_Unnamed_13, pos_T, scid_T, sctx_T, size_t, uint8_t, win_T, xp_prefix_T,
};
use crate::src::nvim::window::{
    WSP_BOT, WSP_HELP, WSP_TOP, win_close, win_enter, win_setheight, win_split,
};
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_int;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_15 = 3;
pub const kOptValTypeString: OptValType = 2;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const WILD_EXPAND_FREE: C2Rust_Unnamed_18 = 2;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const WILD_SILENT: C2Rust_Unnamed_19 = 64;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const ECMD_SET_HELP: C2Rust_Unnamed_20 = 2;
pub const ECMD_HIDE: C2Rust_Unnamed_20 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_int;
pub const ECMD_LASTL: C2Rust_Unnamed_21 = 0;
pub const DT_HELP: C2Rust_Unnamed_25 = 8;
pub const TAG_MANY: C2Rust_Unnamed_26 = 300;
pub const TAG_NO_TAGFUNC: C2Rust_Unnamed_26 = 256;
pub const TAG_VERBOSE: C2Rust_Unnamed_26 = 32;
pub const TAG_NAMES: C2Rust_Unnamed_26 = 2;
pub const TAG_REGEXP: C2Rust_Unnamed_26 = 4;
pub const TAG_HELP: C2Rust_Unnamed_26 = 1;
pub const TAG_KEEP_LANG: C2Rust_Unnamed_26 = 128;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub const OPT_LOCAL: C2Rust_Unnamed_22 = 2;
pub const kEqualFiles: file_comparison = 1;
pub const kEqualFileNames: file_comparison = 7;
pub const kOneFileMissing: file_comparison = 6;
pub const kBothFilesMissing: file_comparison = 4;
pub const kDifferentFiles: file_comparison = 2;
pub const EW_SILENT: C2Rust_Unnamed_23 = 32;
pub const EW_FILE: C2Rust_Unnamed_23 = 2;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 57] = unsafe {
    ::core::mem::transmute::<[u8; 57], [::core::ffi::c_char; 57]>(
        *b"int find_help_tags(const char *, int *, char ***, _Bool)\0",
    )
};
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub unsafe fn ex_help(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut helpfd: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut num_matches: ::core::ffi::c_int = 0;
    let mut matches: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut empty_fnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut alt_fnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let old_KeyTyped: bool = KeyTyped.get();
    if !eap.is_null() {
        arg = (*eap).arg;
        while *arg != 0 {
            if *arg as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                || *arg as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
                || *arg as ::core::ffi::c_int == '|' as ::core::ffi::c_int
                    && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                    && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != '|' as ::core::ffi::c_int
            {
                let c2rust_fresh0 = arg;
                arg = arg.offset(1);
                *c2rust_fresh0 = NUL as ::core::ffi::c_char;
                (*eap).nextcmd = arg;
                break;
            } else {
                arg = arg.offset(1);
            }
        }
        arg = (*eap).arg;
        if (*eap).skip != 0 {
            return;
        }
    } else {
        arg = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    let mut p: *mut ::core::ffi::c_char = arg
        .offset(strlen(arg) as isize)
        .offset(-(1 as ::core::ffi::c_int as isize));
    while p > arg
        && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != '\\' as ::core::ffi::c_int
    {
        let c2rust_fresh1 = p;
        p = p.offset(-1);
        *c2rust_fresh1 = NUL as ::core::ffi::c_char;
    }
    let mut lang: *mut ::core::ffi::c_char = check_help_lang(arg);
    let mut helpbang: bool =
        !eap.is_null() && (*eap).forceit != 0 && *arg as ::core::ffi::c_int == NUL;
    if *arg as ::core::ffi::c_int == NUL && !helpbang {
        arg = b"help.txt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    let mut allocated_arg: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    if helpbang {
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut res: Object = nlua_exec(
            String_0 {
                data: b"return require'vim._core.help'.resolve_tag()\0".as_ptr()
                    as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 45]>().wrapping_sub(1 as size_t),
            },
            ::core::ptr::null::<::core::ffi::c_char>(),
            Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            },
            kRetObject,
            ::core::ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
            && res.type_0 as ::core::ffi::c_uint
                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            && res.data.string.size > 0 as size_t
        {
            allocated_arg = xstrdup(res.data.string.data);
            arg = allocated_arg;
        }
        api_free_object(res);
        api_clear_error(&raw mut err);
        if allocated_arg.is_null() {
            emsg(gettext(&raw const e_noident as *const ::core::ffi::c_char));
            return;
        }
    }
    let mut n: ::core::ffi::c_int = find_help_tags(
        arg,
        &raw mut num_matches,
        &raw mut matches,
        !eap.is_null() && (*eap).forceit != 0,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if n != FAIL && !lang.is_null() {
        i = 0 as ::core::ffi::c_int;
        while i < num_matches {
            let mut len: ::core::ffi::c_int =
                strlen(*matches.offset(i as isize)) as ::core::ffi::c_int;
            if len > 3 as ::core::ffi::c_int
                && *(*matches.offset(i as isize)).offset((len - 3 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    == '@' as ::core::ffi::c_int
                && strcasecmp(
                    (*matches.offset(i as isize))
                        .offset(len as isize)
                        .offset(-(2 as ::core::ffi::c_int as isize)),
                    lang,
                ) == 0 as ::core::ffi::c_int
            {
                break;
            }
            i += 1;
        }
    }
    if i >= num_matches || n == FAIL {
        if !lang.is_null() {
            semsg(
                gettext(b"E661: No '%s' help for %s\0".as_ptr() as *const ::core::ffi::c_char),
                lang,
                arg,
            );
        } else {
            semsg(
                gettext(b"E149: No help for %s\0".as_ptr() as *const ::core::ffi::c_char),
                arg,
            );
        }
        if n != FAIL {
            FreeWild(num_matches, matches);
        }
        xfree(allocated_arg as *mut ::core::ffi::c_void);
        return;
    }
    let mut tag: *mut ::core::ffi::c_char = xstrdup(*matches.offset(i as isize));
    FreeWild(num_matches, matches);
    '_erret: {
        if !bt_help((*curwin.get()).w_buffer) || (*cmdmod.ptr()).cmod_tab != 0 as ::core::ffi::c_int
        {
            if (*cmdmod.ptr()).cmod_tab != 0 as ::core::ffi::c_int {
                wp = ::core::ptr::null_mut::<win_T>();
            } else {
                wp = ::core::ptr::null_mut::<win_T>();
                let mut wp2: *mut win_T = if curtab.get() == curtab.get() {
                    firstwin.get()
                } else {
                    (*curtab.get()).tp_firstwin
                };
                while !wp2.is_null() {
                    if bt_help((*wp2).w_buffer) as ::core::ffi::c_int != 0
                        && !(*wp2).w_config.hide
                        && (*wp2).w_config.focusable as ::core::ffi::c_int != 0
                    {
                        wp = wp2;
                        break;
                    } else {
                        wp2 = (*wp2).w_next;
                    }
                }
            }
            if !wp.is_null() && (*(*wp).w_buffer).b_nwindows > 0 as ::core::ffi::c_int {
                win_enter(wp, true_0 != 0);
            } else {
                helpfd = os_fopen(p_hf.get(), READBIN.as_ptr());
                if helpfd.is_null() {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(
                            b"Help file \"%s\" not found\0".as_ptr() as *const ::core::ffi::c_char
                        ),
                        p_hf.get(),
                    );
                    break '_erret;
                } else {
                    fclose(helpfd);
                    n = WSP_HELP as ::core::ffi::c_int;
                    if (*cmdmod.ptr()).cmod_split == 0 as ::core::ffi::c_int
                        && (*curwin.get()).w_width != Columns.get()
                        && (*curwin.get()).w_width < 80 as ::core::ffi::c_int
                    {
                        n |= if p_sb.get() != 0 {
                            WSP_BOT as ::core::ffi::c_int
                        } else {
                            WSP_TOP as ::core::ffi::c_int
                        };
                    }
                    if win_split(0 as ::core::ffi::c_int, n) == FAIL {
                        break '_erret;
                    } else {
                        if ((*curwin.get()).w_height as OptInt) < p_hh.get() {
                            win_setheight(p_hh.get() as ::core::ffi::c_int);
                        }
                        alt_fnum = (*curbuf.get()).handle as ::core::ffi::c_int;
                        do_ecmd(
                            0 as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            ::core::ptr::null_mut::<exarg_T>(),
                            ECMD_LASTL as ::core::ffi::c_int as linenr_T,
                            ECMD_HIDE as ::core::ffi::c_int + ECMD_SET_HELP as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<win_T>(),
                        );
                        if (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int
                            == 0 as ::core::ffi::c_int
                        {
                            (*curwin.get()).w_alt_fnum = alt_fnum;
                        }
                        empty_fnum = (*curbuf.get()).handle as ::core::ffi::c_int;
                    }
                }
            }
        }
        restart_edit.set(0 as ::core::ffi::c_int);
        KeyTyped.set(old_KeyTyped);
        do_tag(
            tag,
            DT_HELP as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
            false_0,
            true_0 != 0,
        );
        if empty_fnum != 0 as ::core::ffi::c_int && (*curbuf.get()).handle != empty_fnum {
            let mut buf: *mut buf_T = buflist_findnr(empty_fnum);
            if !buf.is_null() && (*buf).b_nwindows == 0 as ::core::ffi::c_int {
                wipe_buffer(buf, true_0 != 0);
            }
        }
        if alt_fnum != 0 as ::core::ffi::c_int
            && (*curwin.get()).w_alt_fnum == empty_fnum
            && (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            (*curwin.get()).w_alt_fnum = alt_fnum;
        }
    }
    xfree(tag as *mut ::core::ffi::c_void);
    xfree(allocated_arg as *mut ::core::ffi::c_void);
}
pub unsafe fn ex_helpclose(mut eap: *mut exarg_T) {
    let mut win: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !win.is_null() {
        if bt_help((*win).w_buffer) {
            win_close(win, false_0 != 0, (*eap).forceit != 0);
            return;
        }
        win = (*win).w_next;
    }
}
pub unsafe extern "C" fn check_help_lang(
    mut arg: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut len: ::core::ffi::c_int = strlen(arg) as ::core::ffi::c_int;
    if len >= 3 as ::core::ffi::c_int
        && *arg.offset((len - 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            == '@' as ::core::ffi::c_int
        && (*arg.offset((len - 2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >= 'A' as ::core::ffi::c_uint
            && *arg.offset((len - 2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                <= 'Z' as ::core::ffi::c_uint
            || *arg.offset((len - 2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >= 'a' as ::core::ffi::c_uint
                && *arg.offset((len - 2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                    <= 'z' as ::core::ffi::c_uint)
        && (*arg.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >= 'A' as ::core::ffi::c_uint
            && *arg.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                <= 'Z' as ::core::ffi::c_uint
            || *arg.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >= 'a' as ::core::ffi::c_uint
                && *arg.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                    <= 'z' as ::core::ffi::c_uint)
    {
        *arg.offset((len - 3 as ::core::ffi::c_int) as isize) = NUL as ::core::ffi::c_char;
        return arg
            .offset(len as isize)
            .offset(-(2 as ::core::ffi::c_int as isize));
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn help_heuristic(
    mut matched_string: *mut ::core::ffi::c_char,
    mut offset: ::core::ffi::c_int,
    mut wrong_case: bool,
) -> ::core::ffi::c_int {
    let mut num_letters: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = matched_string;
    while *p != 0 {
        if *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        {
            num_letters += 1;
        }
        p = p.offset(1);
    }
    if offset > 0 as ::core::ffi::c_int
        && (*matched_string.offset(offset as isize) as ::core::ffi::c_uint
            >= 'A' as ::core::ffi::c_uint
            && *matched_string.offset(offset as isize) as ::core::ffi::c_uint
                <= 'Z' as ::core::ffi::c_uint
            || *matched_string.offset(offset as isize) as ::core::ffi::c_uint
                >= 'a' as ::core::ffi::c_uint
                && *matched_string.offset(offset as isize) as ::core::ffi::c_uint
                    <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(*matched_string.offset(offset as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0)
        && (*matched_string.offset((offset - 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_uint
            >= 'A' as ::core::ffi::c_uint
            && *matched_string.offset((offset - 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_uint
                <= 'Z' as ::core::ffi::c_uint
            || *matched_string.offset((offset - 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_uint
                >= 'a' as ::core::ffi::c_uint
                && *matched_string.offset((offset - 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_uint
                    <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(
                *matched_string.offset((offset - 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0)
    {
        offset += 10000 as ::core::ffi::c_int;
    } else if offset > 2 as ::core::ffi::c_int {
        offset *= 200 as ::core::ffi::c_int;
    }
    if wrong_case {
        offset += 5000 as ::core::ffi::c_int;
    }
    if *matched_string.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '+' as ::core::ffi::c_int
        && *matched_string.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
    {
        offset += 100 as ::core::ffi::c_int;
    }
    return 100 as ::core::ffi::c_int * num_letters
        + strlen(matched_string) as ::core::ffi::c_int
        + offset;
}
unsafe extern "C" fn help_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut p1: *mut ::core::ffi::c_char = (*(s1 as *mut *mut ::core::ffi::c_char))
        .offset(strlen(*(s1 as *mut *mut ::core::ffi::c_char)) as isize)
        .offset(1 as ::core::ffi::c_int as isize);
    let mut p2: *mut ::core::ffi::c_char = (*(s2 as *mut *mut ::core::ffi::c_char))
        .offset(strlen(*(s2 as *mut *mut ::core::ffi::c_char)) as isize)
        .offset(1 as ::core::ffi::c_int as isize);
    let mut cmp: ::core::ffi::c_int = strcmp(p1, p2);
    if cmp != 0 as ::core::ffi::c_int {
        return cmp;
    }
    return strcmp(
        *(s1 as *mut *mut ::core::ffi::c_char),
        *(s2 as *mut *mut ::core::ffi::c_char),
    );
}
pub unsafe extern "C" fn find_help_tags(
    mut arg: *const ::core::ffi::c_char,
    mut num_matches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut keep_lang: bool,
) -> ::core::ffi::c_int {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_13 { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh2 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh2 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_13 {
            string: cstr_as_string(arg),
        },
    };
    let mut res: Object = nlua_exec(
        String_0 {
            data: b"return require'vim._core.help'.escape_subject(...)\0".as_ptr()
                as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: ::core::mem::size_of::<[::core::ffi::c_char; 51]>().wrapping_sub(1 as size_t),
        },
        ::core::ptr::null::<::core::ffi::c_char>(),
        args,
        kRetObject,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        emsg_multiline(
            err.msg,
            b"lua_error\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_E,
            true_0 != 0,
        );
        api_clear_error(&raw mut err);
        return FAIL;
    }
    api_clear_error(&raw mut err);
    '_c2rust_label: {
        if res.type_0 as ::core::ffi::c_uint
            == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"res.type == kObjectTypeString\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/help.rs\0".as_ptr() as *const ::core::ffi::c_char,
                353 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    xstrlcpy(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        res.data.string.data,
        ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
    );
    api_free_object(res);
    *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    *num_matches = 0 as ::core::ffi::c_int;
    let mut flags: ::core::ffi::c_int = TAG_HELP as ::core::ffi::c_int
        | TAG_REGEXP as ::core::ffi::c_int
        | TAG_NAMES as ::core::ffi::c_int
        | TAG_VERBOSE as ::core::ffi::c_int
        | TAG_NO_TAGFUNC as ::core::ffi::c_int;
    if keep_lang {
        flags |= TAG_KEEP_LANG as ::core::ffi::c_int;
    }
    if find_tags(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        num_matches,
        matches,
        flags,
        MAXCOL as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    ) == OK
        && *num_matches > 0 as ::core::ffi::c_int
    {
        qsort(
            *matches as *mut ::core::ffi::c_void,
            *num_matches as size_t,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>(),
            Some(
                help_compare
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        while *num_matches > TAG_MANY as ::core::ffi::c_int {
            *num_matches -= 1;
            xfree(*(*matches).offset(*num_matches as isize) as *mut ::core::ffi::c_void);
        }
    }
    return OK;
}
pub unsafe extern "C" fn cleanup_help_tags(
    mut num_file: ::core::ffi::c_int,
    mut file: *mut *mut ::core::ffi::c_char,
) {
    let mut buf: [::core::ffi::c_char; 4] = [0; 4];
    let mut p: *mut ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
    if *(*p_hlg.ptr()).offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        && (*(*p_hlg.ptr()).offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 'e' as ::core::ffi::c_int
            || *(*p_hlg.ptr()).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != 'n' as ::core::ffi::c_int)
    {
        let c2rust_fresh3 = p;
        p = p.offset(1);
        *c2rust_fresh3 = '@' as ::core::ffi::c_char;
        let c2rust_fresh4 = p;
        p = p.offset(1);
        *c2rust_fresh4 = *(*p_hlg.ptr()).offset(0 as ::core::ffi::c_int as isize);
        let c2rust_fresh5 = p;
        p = p.offset(1);
        *c2rust_fresh5 = *(*p_hlg.ptr()).offset(1 as ::core::ffi::c_int as isize);
    }
    *p = NUL as ::core::ffi::c_char;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_file {
        let mut len: ::core::ffi::c_int =
            strlen(*file.offset(i as isize)) as ::core::ffi::c_int - 3 as ::core::ffi::c_int;
        if len > 0 as ::core::ffi::c_int {
            if strcmp(
                (*file.offset(i as isize)).offset(len as isize),
                b"@en\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                let mut j: ::core::ffi::c_int = 0;
                j = 0 as ::core::ffi::c_int;
                while j < num_file {
                    if j != i
                        && strlen(*file.offset(j as isize)) as ::core::ffi::c_int
                            == len + 3 as ::core::ffi::c_int
                        && strncmp(
                            *file.offset(i as isize),
                            *file.offset(j as isize),
                            (len as size_t).wrapping_add(1 as size_t),
                        ) == 0 as ::core::ffi::c_int
                    {
                        break;
                    }
                    j += 1;
                }
                if j == num_file {
                    *(*file.offset(i as isize)).offset(len as isize) = NUL as ::core::ffi::c_char;
                }
            }
        }
        i += 1;
    }
    if *(&raw mut buf as *mut ::core::ffi::c_char) as ::core::ffi::c_int != NUL {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < num_file {
            let mut len_0: ::core::ffi::c_int =
                strlen(*file.offset(i_0 as isize)) as ::core::ffi::c_int - 3 as ::core::ffi::c_int;
            if len_0 > 0 as ::core::ffi::c_int {
                if strcmp(
                    (*file.offset(i_0 as isize)).offset(len_0 as isize),
                    &raw mut buf as *mut ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    *(*file.offset(i_0 as isize)).offset(len_0 as isize) =
                        NUL as ::core::ffi::c_char;
                }
            }
            i_0 += 1;
        }
    }
}
pub unsafe extern "C" fn prepare_help_buffer() {
    (*curbuf.get()).b_help = true_0 != 0;
    set_option_direct(
        kOptBuftype,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"help\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
        0 as scid_T,
    );
    let mut p: *mut ::core::ffi::c_char = b"!-~,^*,^|,^\",192-255\0".as_ptr()
        as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char;
    if strcmp((*curbuf.get()).b_p_isk, p) != 0 as ::core::ffi::c_int {
        set_option_direct(
            kOptIskeyword,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(p),
                },
            },
            OPT_LOCAL as ::core::ffi::c_int,
            0 as scid_T,
        );
        check_buf_options(curbuf.get());
        buf_init_chartab(curbuf.get(), false);
    }
    set_option_direct(
        kOptFoldmethod,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"manual\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
        0 as scid_T,
    );
    (*curbuf.get()).b_p_ts = 8 as OptInt;
    (*curwin.get()).w_onebuf_opt.wo_list = false_0;
    (*curbuf.get()).b_p_ma = false_0;
    (*curbuf.get()).b_p_bin = false_0;
    (*curwin.get()).w_onebuf_opt.wo_nu = 0 as ::core::ffi::c_int;
    (*curwin.get()).w_onebuf_opt.wo_rnu = 0 as ::core::ffi::c_int;
    (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
    (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
    (*curwin.get()).w_onebuf_opt.wo_arab = false_0;
    (*curwin.get()).w_onebuf_opt.wo_rl = false_0;
    (*curwin.get()).w_onebuf_opt.wo_fen = false_0;
    (*curwin.get()).w_onebuf_opt.wo_diff = false_0;
    (*curwin.get()).w_onebuf_opt.wo_spell = false_0;
    set_buflisted(false_0);
}
pub unsafe extern "C" fn get_local_additions() {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut res: Object = nlua_exec(
        String_0 {
            data: b"return require'vim._core.help'.local_additions()\0".as_ptr()
                as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: ::core::mem::size_of::<[::core::ffi::c_char; 49]>().wrapping_sub(1 as size_t),
        },
        ::core::ptr::null::<::core::ffi::c_char>(),
        Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        },
        kRetNilBool,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        emsg_multiline(
            err.msg,
            b"lua_error\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_E,
            true_0 != 0,
        );
    }
    api_free_object(res);
    api_clear_error(&raw mut err);
}
pub unsafe fn ex_exusage(mut _eap: *mut exarg_T) {
    do_cmdline_cmd(b"help ex-cmd-index\0".as_ptr() as *const ::core::ffi::c_char);
}
pub unsafe fn ex_viusage(mut _eap: *mut exarg_T) {
    do_cmdline_cmd(b"help normal-index\0".as_ptr() as *const ::core::ffi::c_char);
}
unsafe extern "C" fn helptags_one(
    mut dir: *mut ::core::ffi::c_char,
    mut ext: *const ::core::ffi::c_char,
    mut tagfname: *const ::core::ffi::c_char,
    mut add_help_tags: bool,
    mut ignore_writeerr: bool,
) {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut filecount: ::core::ffi::c_int = 0;
    let mut files: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dirlen: size_t = xstrlcpy(
        NameBuff.ptr() as *mut ::core::ffi::c_char,
        dir,
        ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
    );
    if dirlen >= MAXPATHL as size_t
        || xstrlcat(
            NameBuff.ptr() as *mut ::core::ffi::c_char,
            b"/**/*\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        ) >= MAXPATHL as size_t
        || xstrlcat(
            NameBuff.ptr() as *mut ::core::ffi::c_char,
            ext,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        ) >= MAXPATHL as size_t
    {
        emsg(gettext(
            &raw const e_fnametoolong as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut buff_list: [*mut ::core::ffi::c_char; 1] = [NameBuff.ptr() as *mut ::core::ffi::c_char];
    let res: ::core::ffi::c_int = gen_expand_wildcards(
        1 as ::core::ffi::c_int,
        &raw mut buff_list as *mut *mut ::core::ffi::c_char,
        &raw mut filecount,
        &raw mut files,
        EW_FILE as ::core::ffi::c_int | EW_SILENT as ::core::ffi::c_int,
    );
    if res == FAIL || filecount == 0 as ::core::ffi::c_int {
        if !got_int.get() {
            semsg(
                gettext(b"E151: No match: %s\0".as_ptr() as *const ::core::ffi::c_char),
                NameBuff.ptr() as *mut ::core::ffi::c_char,
            );
        }
        if res != FAIL {
            FreeWild(filecount, files);
        }
        return;
    }
    memcpy(
        NameBuff.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        dir as *const ::core::ffi::c_void,
        dirlen.wrapping_add(1 as size_t),
    );
    if !add_pathsep(NameBuff.ptr() as *mut ::core::ffi::c_char)
        || xstrlcat(
            NameBuff.ptr() as *mut ::core::ffi::c_char,
            tagfname,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        ) >= MAXPATHL as size_t
    {
        emsg(gettext(
            &raw const e_fnametoolong as *const ::core::ffi::c_char,
        ));
        return;
    }
    let fd_tags: *mut FILE = os_fopen(
        NameBuff.ptr() as *mut ::core::ffi::c_char,
        b"w\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if fd_tags.is_null() {
        if !ignore_writeerr {
            semsg(
                gettext(
                    b"E152: Cannot open %s for writing\0".as_ptr() as *const ::core::ffi::c_char
                ),
                NameBuff.ptr() as *mut ::core::ffi::c_char,
            );
        }
        FreeWild(filecount, files);
        return;
    }
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        100 as ::core::ffi::c_int,
    );
    if add_help_tags as ::core::ffi::c_int != 0
        || path_full_compare(
            b"$VIMRUNTIME/doc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            dir,
            false_0 != 0,
            true_0 != 0,
        ) as ::core::ffi::c_uint
            == kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut s_len: size_t = (18 as size_t).wrapping_add(strlen(tagfname));
        s = xmalloc(s_len) as *mut ::core::ffi::c_char;
        snprintf(
            s,
            s_len,
            b"help-tags\t%s\t1\n\0".as_ptr() as *const ::core::ffi::c_char,
            tagfname,
        );
        ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
        *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(ga.ga_len as isize) = s;
        ga.ga_len += 1;
    }
    let mut fi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while fi < filecount && !got_int.get() {
        let fd: *mut FILE = os_fopen(
            *files.offset(fi as isize),
            b"r\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if fd.is_null() {
            semsg(
                gettext(
                    b"E153: Unable to open %s for reading\0".as_ptr() as *const ::core::ffi::c_char
                ),
                *files.offset(fi as isize),
            );
        } else {
            let fname: *const ::core::ffi::c_char = (*files.offset(fi as isize))
                .offset(dirlen as isize)
                .offset(1 as ::core::ffi::c_int as isize);
            let mut in_example: bool = false_0 != 0;
            while !vim_fgets(IObuff.ptr() as *mut ::core::ffi::c_char, IOSIZE, fd) && !got_int.get()
            {
                if in_example {
                    if !vim_strchr(
                        b" \t\n\r\0".as_ptr() as *const ::core::ffi::c_char,
                        (*IObuff.ptr())[0 as ::core::ffi::c_int as usize] as uint8_t
                            as ::core::ffi::c_int,
                    )
                    .is_null()
                    {
                        continue;
                    }
                    in_example = false_0 != 0;
                }
                let mut p1: *mut ::core::ffi::c_char = vim_strchr(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    '*' as ::core::ffi::c_int,
                );
                while !p1.is_null() {
                    let mut p2: *mut ::core::ffi::c_char = strchr(
                        p1.offset(1 as ::core::ffi::c_int as isize),
                        '*' as ::core::ffi::c_int,
                    );
                    if !p2.is_null() && p2 > p1.offset(1 as ::core::ffi::c_int as isize) {
                        s = p1.offset(1 as ::core::ffi::c_int as isize);
                        while s < p2 {
                            if *s as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                                || *s as ::core::ffi::c_int == '\t' as ::core::ffi::c_int
                                || *s as ::core::ffi::c_int == '|' as ::core::ffi::c_int
                            {
                                break;
                            }
                            s = s.offset(1);
                        }
                        if s == p2
                            && (p1 == IObuff.ptr() as *mut ::core::ffi::c_char
                                || *p1.offset(-1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == ' ' as ::core::ffi::c_int
                                || *p1.offset(-1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == '\t' as ::core::ffi::c_int)
                            && (!vim_strchr(
                                b" \t\n\r\0".as_ptr() as *const ::core::ffi::c_char,
                                *s.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                                    as ::core::ffi::c_int,
                            )
                            .is_null()
                                || *s.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == NUL)
                        {
                            *p2 = NUL as ::core::ffi::c_char;
                            p1 = p1.offset(1);
                            let mut s_len_0: size_t = (p2.offset_from(p1) as size_t)
                                .wrapping_add(strlen(fname))
                                .wrapping_add(2 as size_t);
                            s = xmalloc(s_len_0) as *mut ::core::ffi::c_char;
                            ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                            *(ga.ga_data as *mut *mut ::core::ffi::c_char)
                                .offset(ga.ga_len as isize) = s;
                            ga.ga_len += 1;
                            snprintf(
                                s,
                                s_len_0,
                                b"%s\t%s\0".as_ptr() as *const ::core::ffi::c_char,
                                p1,
                                fname,
                            );
                            p2 = vim_strchr(
                                p2.offset(1 as ::core::ffi::c_int as isize),
                                '*' as ::core::ffi::c_int,
                            );
                        }
                    }
                    p1 = p2;
                }
                let mut off: size_t = strlen(IObuff.ptr() as *mut ::core::ffi::c_char);
                if off >= 2 as size_t
                    && (*IObuff.ptr())[off.wrapping_sub(1 as size_t) as usize] as ::core::ffi::c_int
                        == '\n' as ::core::ffi::c_int
                {
                    off = off.wrapping_sub(2 as size_t);
                    while off > 0 as size_t
                        && ((*IObuff.ptr())[off as usize] as ::core::ffi::c_uint
                            >= 'a' as ::core::ffi::c_uint
                            && (*IObuff.ptr())[off as usize] as ::core::ffi::c_uint
                                <= 'z' as ::core::ffi::c_uint
                            || ascii_isdigit((*IObuff.ptr())[off as usize] as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                                != 0)
                    {
                        off = off.wrapping_sub(1);
                    }
                    if (*IObuff.ptr())[off as usize] as ::core::ffi::c_int
                        == '>' as ::core::ffi::c_int
                        && (off == 0 as size_t
                            || (*IObuff.ptr())[off.wrapping_sub(1 as size_t) as usize]
                                as ::core::ffi::c_int
                                == ' ' as ::core::ffi::c_int)
                    {
                        in_example = true_0 != 0;
                    }
                }
                line_breakcheck();
            }
            fclose(fd);
        }
        fi += 1;
    }
    FreeWild(filecount, files);
    if !got_int.get() && !ga.ga_data.is_null() {
        sort_strings(ga.ga_data as *mut *mut ::core::ffi::c_char, ga.ga_len);
        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while i < ga.ga_len {
            let mut p1_0: *mut ::core::ffi::c_char = *(ga.ga_data as *mut *mut ::core::ffi::c_char)
                .offset((i - 1 as ::core::ffi::c_int) as isize);
            let mut p2_0: *mut ::core::ffi::c_char =
                *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize);
            while *p1_0 as ::core::ffi::c_int == *p2_0 as ::core::ffi::c_int {
                if *p2_0 as ::core::ffi::c_int == '\t' as ::core::ffi::c_int {
                    *p2_0 = NUL as ::core::ffi::c_char;
                    vim_snprintf(
                        NameBuff.ptr() as *mut ::core::ffi::c_char,
                        MAXPATHL as size_t,
                        gettext(b"E154: Duplicate tag \"%s\" in file %s/%s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize),
                        dir,
                        p2_0.offset(1 as ::core::ffi::c_int as isize),
                    );
                    emsg(NameBuff.ptr() as *mut ::core::ffi::c_char);
                    *p2_0 = '\t' as ::core::ffi::c_char;
                    break;
                } else {
                    p1_0 = p1_0.offset(1);
                    p2_0 = p2_0.offset(1);
                }
            }
            i += 1;
        }
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < ga.ga_len {
            s = *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(i_0 as isize);
            if strncmp(
                s,
                b"help-tags\t\0".as_ptr() as *const ::core::ffi::c_char,
                10 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                fputs(s, fd_tags);
            } else {
                fprintf(
                    fd_tags,
                    b"%s\t/*\0".as_ptr() as *const ::core::ffi::c_char,
                    s,
                );
                let mut p1_1: *mut ::core::ffi::c_char = s;
                while *p1_1 as ::core::ffi::c_int != '\t' as ::core::ffi::c_int {
                    if *p1_1 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                        || *p1_1 as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                    {
                        putc('\\' as ::core::ffi::c_int, fd_tags);
                    }
                    putc(*p1_1 as ::core::ffi::c_int, fd_tags);
                    p1_1 = p1_1.offset(1);
                }
                fprintf(fd_tags, b"*\n\0".as_ptr() as *const ::core::ffi::c_char);
            }
            i_0 += 1;
        }
    }
    let mut _gap: *mut garray_T = &raw mut ga;
    if !(*_gap).ga_data.is_null() {
        let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_1 < (*_gap).ga_len {
            let mut _item: *mut *mut ::core::ffi::c_void =
                ((*_gap).ga_data as *mut *mut ::core::ffi::c_void).offset(i_1 as isize);
            xfree(*_item);
            i_1 += 1;
        }
    }
    ga_clear(_gap);
    fclose(fd_tags);
}
unsafe extern "C" fn do_helptags(
    mut dirname: *mut ::core::ffi::c_char,
    mut add_help_tags: bool,
    mut ignore_writeerr: bool,
) {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut lang: [::core::ffi::c_char; 2] = [0; 2];
    let mut ext: [::core::ffi::c_char; 5] = [0; 5];
    let mut fname: [::core::ffi::c_char; 8] = [0; 8];
    let mut filecount: ::core::ffi::c_int = 0;
    let mut files: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    xstrlcpy(
        NameBuff.ptr() as *mut ::core::ffi::c_char,
        dirname,
        ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
    );
    if !add_pathsep(NameBuff.ptr() as *mut ::core::ffi::c_char)
        || xstrlcat(
            NameBuff.ptr() as *mut ::core::ffi::c_char,
            b"**\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        ) >= MAXPATHL as size_t
    {
        emsg(gettext(
            &raw const e_fnametoolong as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut buff_list: [*mut ::core::ffi::c_char; 1] = [NameBuff.ptr() as *mut ::core::ffi::c_char];
    if gen_expand_wildcards(
        1 as ::core::ffi::c_int,
        &raw mut buff_list as *mut *mut ::core::ffi::c_char,
        &raw mut filecount,
        &raw mut files,
        EW_FILE as ::core::ffi::c_int | EW_SILENT as ::core::ffi::c_int,
    ) == FAIL
        || filecount == 0 as ::core::ffi::c_int
    {
        semsg(
            gettext(b"E151: No match: %s\0".as_ptr() as *const ::core::ffi::c_char),
            NameBuff.ptr() as *mut ::core::ffi::c_char,
        );
        return;
    }
    let mut j: ::core::ffi::c_int = 0;
    ga_init(
        &raw mut ga,
        1 as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < filecount {
        let mut len: ::core::ffi::c_int = strlen(*files.offset(i as isize)) as ::core::ffi::c_int;
        's_52: {
            if len > 4 as ::core::ffi::c_int {
                if strcasecmp(
                    (*files.offset(i as isize))
                        .offset(len as isize)
                        .offset(-(4 as ::core::ffi::c_int as isize)),
                    b".txt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    lang[0 as ::core::ffi::c_int as usize] = 'e' as ::core::ffi::c_char;
                    lang[1 as ::core::ffi::c_int as usize] = 'n' as ::core::ffi::c_char;
                } else if *(*files.offset(i as isize))
                    .offset((len - 4 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                    && (*(*files.offset(i as isize))
                        .offset((len - 3 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        >= 'A' as ::core::ffi::c_uint
                        && *(*files.offset(i as isize))
                            .offset((len - 3 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            <= 'Z' as ::core::ffi::c_uint
                        || *(*files.offset(i as isize))
                            .offset((len - 3 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            >= 'a' as ::core::ffi::c_uint
                            && *(*files.offset(i as isize))
                                .offset((len - 3 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_uint
                                <= 'z' as ::core::ffi::c_uint)
                    && (*(*files.offset(i as isize))
                        .offset((len - 2 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        >= 'A' as ::core::ffi::c_uint
                        && *(*files.offset(i as isize))
                            .offset((len - 2 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            <= 'Z' as ::core::ffi::c_uint
                        || *(*files.offset(i as isize))
                            .offset((len - 2 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            >= 'a' as ::core::ffi::c_uint
                            && *(*files.offset(i as isize))
                                .offset((len - 2 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_uint
                                <= 'z' as ::core::ffi::c_uint)
                    && (if (*(*files.offset(i as isize))
                        .offset((len - 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int)
                        < 'A' as ::core::ffi::c_int
                        || *(*files.offset(i as isize))
                            .offset((len - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            > 'Z' as ::core::ffi::c_int
                    {
                        *(*files.offset(i as isize))
                            .offset((len - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                    } else {
                        *(*files.offset(i as isize))
                            .offset((len - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                    }) == 'x' as ::core::ffi::c_int
                {
                    lang[0 as ::core::ffi::c_int as usize] = (if (*(*files.offset(i as isize))
                        .offset((len - 3 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int)
                        < 'A' as ::core::ffi::c_int
                        || *(*files.offset(i as isize))
                            .offset((len - 3 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            > 'Z' as ::core::ffi::c_int
                    {
                        *(*files.offset(i as isize))
                            .offset((len - 3 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                    } else {
                        *(*files.offset(i as isize))
                            .offset((len - 3 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                    })
                        as ::core::ffi::c_char;
                    lang[1 as ::core::ffi::c_int as usize] = (if (*(*files.offset(i as isize))
                        .offset((len - 2 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int)
                        < 'A' as ::core::ffi::c_int
                        || *(*files.offset(i as isize))
                            .offset((len - 2 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            > 'Z' as ::core::ffi::c_int
                    {
                        *(*files.offset(i as isize))
                            .offset((len - 2 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                    } else {
                        *(*files.offset(i as isize))
                            .offset((len - 2 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                    })
                        as ::core::ffi::c_char;
                } else {
                    break 's_52;
                }
                j = 0 as ::core::ffi::c_int;
                while j < ga.ga_len {
                    if strncmp(
                        &raw mut lang as *mut ::core::ffi::c_char,
                        (ga.ga_data as *mut ::core::ffi::c_char).offset(j as isize),
                        2 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        break;
                    }
                    j += 2 as ::core::ffi::c_int;
                }
                if j == ga.ga_len {
                    ga_grow(&raw mut ga, 2 as ::core::ffi::c_int);
                    let c2rust_fresh6 = ga.ga_len;
                    ga.ga_len = ga.ga_len + 1;
                    *(ga.ga_data as *mut ::core::ffi::c_char).offset(c2rust_fresh6 as isize) =
                        lang[0 as ::core::ffi::c_int as usize];
                    let c2rust_fresh7 = ga.ga_len;
                    ga.ga_len = ga.ga_len + 1;
                    *(ga.ga_data as *mut ::core::ffi::c_char).offset(c2rust_fresh7 as isize) =
                        lang[1 as ::core::ffi::c_int as usize];
                }
            }
        }
        i += 1;
    }
    j = 0 as ::core::ffi::c_int;
    while j < ga.ga_len {
        strcpy(
            &raw mut fname as *mut ::core::ffi::c_char,
            b"tags-xx\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        fname[5 as ::core::ffi::c_int as usize] =
            *(ga.ga_data as *mut ::core::ffi::c_char).offset(j as isize);
        fname[6 as ::core::ffi::c_int as usize] = *(ga.ga_data as *mut ::core::ffi::c_char)
            .offset((j + 1 as ::core::ffi::c_int) as isize);
        if fname[5 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            == 'e' as ::core::ffi::c_int
            && fname[6 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                == 'n' as ::core::ffi::c_int
        {
            fname[4 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
            strcpy(
                &raw mut ext as *mut ::core::ffi::c_char,
                b".txt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
        } else {
            strcpy(
                &raw mut ext as *mut ::core::ffi::c_char,
                b".xxx\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            ext[1 as ::core::ffi::c_int as usize] = fname[5 as ::core::ffi::c_int as usize];
            ext[2 as ::core::ffi::c_int as usize] = fname[6 as ::core::ffi::c_int as usize];
        }
        helptags_one(
            dirname,
            &raw mut ext as *mut ::core::ffi::c_char,
            &raw mut fname as *mut ::core::ffi::c_char,
            add_help_tags,
            ignore_writeerr,
        );
        j += 2 as ::core::ffi::c_int;
    }
    ga_clear(&raw mut ga);
    FreeWild(filecount, files);
}
unsafe extern "C" fn helptags_cb(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut cookie: *mut ::core::ffi::c_void,
) -> bool {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_fnames {
        do_helptags(
            *fnames.offset(i as isize),
            *(cookie as *mut bool),
            true_0 != 0,
        );
        if !all {
            return true_0 != 0;
        }
        i += 1;
    }
    return num_fnames > 0 as ::core::ffi::c_int;
}
pub unsafe fn ex_helptags(mut eap: *mut exarg_T) {
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
    let mut add_help_tags: bool = false_0 != 0;
    if strncmp(
        (*eap).arg,
        b"++t\0".as_ptr() as *const ::core::ffi::c_char,
        3 as size_t,
    ) == 0 as ::core::ffi::c_int
        && ascii_iswhite(*(*eap).arg.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
    {
        add_help_tags = true_0 != 0;
        (*eap).arg = skipwhite((*eap).arg.offset(3 as ::core::ffi::c_int as isize));
    }
    if strcmp((*eap).arg, b"ALL\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        do_in_path(
            p_rtp.get(),
            b"\0".as_ptr() as *const ::core::ffi::c_char,
            b"doc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            DIP_ALL as ::core::ffi::c_int + DIP_DIR as ::core::ffi::c_int,
            Some(
                helptags_cb
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            &raw mut add_help_tags as *mut ::core::ffi::c_void,
        );
    } else {
        ExpandInit(&raw mut xpc);
        xpc.xp_context = EXPAND_DIRECTORIES as ::core::ffi::c_int;
        let mut dirname: *mut ::core::ffi::c_char = ExpandOne(
            &raw mut xpc,
            (*eap).arg,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            WILD_LIST_NOTFOUND as ::core::ffi::c_int | WILD_SILENT as ::core::ffi::c_int,
            WILD_EXPAND_FREE as ::core::ffi::c_int,
        );
        if dirname.is_null() || !os_isdir(dirname) {
            semsg(
                gettext(b"E150: Not a directory: %s\0".as_ptr() as *const ::core::ffi::c_char),
                (*eap).arg,
            );
        } else {
            do_helptags(dirname, add_help_tags, false_0 != 0);
        }
        xfree(dirname as *mut ::core::ffi::c_void);
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const READBIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"rb\0") };
