use crate::semsg_c;
use crate::src::nvim::arglist::alist_name;
use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::autocmd::{EVENT_SESSIONWRITEPOST, apply_autocmds};
use crate::src::nvim::buffer::{bt_help, bt_nofilename, bt_normal, bt_terminal, buflist_findnr};
use crate::src::nvim::eval::typval::tv_get_string;
use crate::src::nvim::eval::var_flavour;
use crate::src::nvim::eval::vars::{get_globvar_dict, set_vim_var_string};
use crate::src::nvim::ex_docmd::{open_exfile, vim_mkdir_emsg};
use crate::src::nvim::ex_getln::vim_strsave_fnameescape;
use crate::src::nvim::file_search::vim_chdirfile;
use crate::src::nvim::fileio::shorten_fnames;
use crate::src::nvim::fold::put_folds;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::main::{
    Columns, Rows, curbuf, curtab, curwin, e_noname, e_notopen, e_prev_dir, e_write, first_tabpage,
    firstbuf, firstwin, global_alist, globaldir, no_hlsearch, p_acd, p_hls, p_shm, p_stal, p_vdir,
    p_wh, p_wiw, ssop_flags, topframe, vop_flags,
};
use crate::src::nvim::mapping::makemap;
use crate::src::nvim::mbyte::utfc_ptr2len;
use crate::src::nvim::memory::{xfree, xmalloc, xmemcpyz};
use crate::src::nvim::message::emsg;
use crate::src::nvim::option::{makefoldset, makeset};
use crate::src::nvim::options::{
    kOptSsopFlagBlank, kOptSsopFlagBuffers, kOptSsopFlagCurdir, kOptSsopFlagCursor,
    kOptSsopFlagFolds, kOptSsopFlagGlobals, kOptSsopFlagHelp, kOptSsopFlagLocaloptions,
    kOptSsopFlagOptions, kOptSsopFlagResize, kOptSsopFlagSesdir, kOptSsopFlagSkiprtp,
    kOptSsopFlagTabpages, kOptSsopFlagTerminal, kOptSsopFlagWinsize,
};
use crate::src::nvim::os::env::home_replace_save;
use crate::src::nvim::os::fs::{os_chdir, os_dirname, os_isdir};
use crate::src::nvim::os::libc::{fclose, fprintf, fputs, gettext, putc, strcpy, strlen};
use crate::src::nvim::path::{add_pathsep, vim_FullName, vim_ispathsep};
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::runtime::do_source;
use crate::src::nvim::strings::vim_strsave_escaped;
use crate::src::nvim::types::{
    CMD_mksession, CMD_mkview, CMD_mkvimrc, CdCause, FILE, OptInt, VAR_FLAVOUR_SESSION, VAR_FLOAT,
    VAR_NUMBER, VAR_STRING, VV_THIS_SESSION, aentry_T, buf_T, dictitem_T, exarg_T, float_T,
    frame_T, garray_T, hashitem_T, hashtab_T, int64_t, ptrdiff_t, size_t, tabpage_T, win_T,
};
use crate::src::nvim::window::tabpage_index;
pub const kCdCauseOther: CdCause = -1;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const VSE_NONE: C2Rust_Unnamed_14 = 0;
pub const DOSO_NONE: C2Rust_Unnamed_17 = 0;
pub const OPT_LOCAL: C2Rust_Unnamed_15 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_15 = 1;
pub const OPT_SKIPRTP: C2Rust_Unnamed_15 = 128;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const SESSION_FILE: [::core::ffi::c_char; 12] =
    unsafe { ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"Session.vim\0") };
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FR_LEAF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FR_COL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static did_lcd: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
unsafe extern "C" fn put_view_curpos(
    mut fd: *mut FILE,
    mut wp: *const win_T,
    mut spaces: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    if (*wp).w_curswant == MAXCOL as ::core::ffi::c_int {
        r = fprintf(
            fd,
            b"%snormal! $\n\0".as_ptr() as *const ::core::ffi::c_char,
            spaces,
        );
    } else {
        r = fprintf(
            fd,
            b"%snormal! 0%d|\n\0".as_ptr() as *const ::core::ffi::c_char,
            spaces,
            (*wp).w_virtcol as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
        );
    }
    return (r >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn ses_winsizes(
    mut fd: *mut FILE,
    mut restore_size: bool,
    mut tab_firstwin: *mut win_T,
) -> ::core::ffi::c_int {
    if restore_size as ::core::ffi::c_int != 0
        && ssop_flags.get() & kOptSsopFlagWinsize as ::core::ffi::c_int as ::core::ffi::c_uint != 0
    {
        let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut wp: *mut win_T = tab_firstwin;
        while !wp.is_null() {
            if ses_do_win(wp) != 0 {
                n += 1;
                if (*wp).w_height + (*wp).w_hsep_height + (*wp).w_status_height
                    < (*topframe.get()).fr_height
                    && fprintf(
                        fd,
                        b"exe '%dresize ' . ((&lines * %ld + %ld) / %ld)\n\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        n,
                        (*wp).w_height as int64_t,
                        Rows.get() as int64_t / 2 as int64_t,
                        Rows.get() as int64_t,
                    ) < 0 as ::core::ffi::c_int
                {
                    return FAIL;
                }
                if (*wp).w_width < Columns.get()
                    && fprintf(
                        fd,
                        b"exe 'vert %dresize ' . ((&columns * %ld + %ld) / %ld)\n\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        n,
                        (*wp).w_width as int64_t,
                        Columns.get() as int64_t / 2 as int64_t,
                        Columns.get() as int64_t,
                    ) < 0 as ::core::ffi::c_int
                {
                    return FAIL;
                }
            }
            wp = (*wp).w_next;
        }
    } else if FAIL
        == put_line(
            fd,
            b"wincmd =\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        )
    {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn ses_win_rec(mut fd: *mut FILE, mut fr: *mut frame_T) -> ::core::ffi::c_int {
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*fr).fr_layout as ::core::ffi::c_int == FR_LEAF {
        return OK;
    }
    let mut frc: *mut frame_T = ses_skipframe((*fr).fr_child);
    if !frc.is_null() {
        loop {
            frc = ses_skipframe((*frc).fr_next);
            if frc.is_null() {
                break;
            }
            if fprintf(
                fd,
                b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"wincmd _ | wincmd |\n\0".as_ptr() as *const ::core::ffi::c_char,
                if (*fr).fr_layout as ::core::ffi::c_int == FR_COL {
                    b"split\n\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"vsplit\n\0".as_ptr() as *const ::core::ffi::c_char
                },
            ) < 0 as ::core::ffi::c_int
            {
                return FAIL;
            }
            count += 1;
        }
    }
    if count > 0 as ::core::ffi::c_int
        && fprintf(
            fd,
            if (*fr).fr_layout as ::core::ffi::c_int == FR_COL {
                b"%dwincmd k\n\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"%dwincmd h\n\0".as_ptr() as *const ::core::ffi::c_char
            },
            count,
        ) < 0 as ::core::ffi::c_int
    {
        return FAIL;
    }
    frc = ses_skipframe((*fr).fr_child);
    while !frc.is_null() {
        ses_win_rec(fd, frc);
        frc = ses_skipframe((*frc).fr_next);
        if !frc.is_null()
            && put_line(
                fd,
                b"wincmd w\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) == FAIL
        {
            return FAIL;
        }
    }
    return OK;
}
unsafe extern "C" fn ses_skipframe(mut fr: *mut frame_T) -> *mut frame_T {
    let mut frc: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
    frc = fr;
    while !frc.is_null() {
        if ses_do_frame(frc) {
            break;
        }
        frc = (*frc).fr_next;
    }
    return frc;
}
unsafe extern "C" fn ses_do_frame(mut fr: *const frame_T) -> bool {
    let mut frc: *const frame_T = ::core::ptr::null::<frame_T>();
    if (*fr).fr_layout as ::core::ffi::c_int == FR_LEAF {
        return ses_do_win((*fr).fr_win) != 0;
    }
    frc = (*fr).fr_child;
    while !frc.is_null() {
        if ses_do_frame(frc) {
            return true_0 != 0;
        }
        frc = (*frc).fr_next;
    }
    return false_0 != 0;
}
unsafe extern "C" fn ses_do_win(mut wp: *mut win_T) -> ::core::ffi::c_int {
    if (*wp).w_floating {
        return false_0;
    }
    if (*(*wp).w_buffer).b_fname.is_null()
        || (*(*wp).w_buffer).terminal.is_null()
            && bt_nofilename((*wp).w_buffer) as ::core::ffi::c_int != 0
    {
        return (ssop_flags.get() & kOptSsopFlagBlank as ::core::ffi::c_int as ::core::ffi::c_uint)
            as ::core::ffi::c_int;
    }
    if bt_help((*wp).w_buffer) {
        return (ssop_flags.get() & kOptSsopFlagHelp as ::core::ffi::c_int as ::core::ffi::c_uint)
            as ::core::ffi::c_int;
    }
    if bt_terminal((*wp).w_buffer) {
        return (ssop_flags.get()
            & kOptSsopFlagTerminal as ::core::ffi::c_int as ::core::ffi::c_uint)
            as ::core::ffi::c_int;
    }
    return true_0;
}
unsafe extern "C" fn ses_arglist(
    mut fd: *mut FILE,
    mut cmd: *mut ::core::ffi::c_char,
    mut gap: *mut garray_T,
    mut fullname: bool,
    mut flagp: *mut ::core::ffi::c_uint,
) -> ::core::ffi::c_int {
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if fprintf(
        fd,
        b"%s\n%s\n\0".as_ptr() as *const ::core::ffi::c_char,
        cmd,
        b"%argdel\0".as_ptr() as *const ::core::ffi::c_char,
    ) < 0 as ::core::ffi::c_int
    {
        return FAIL;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        let mut s: *mut ::core::ffi::c_char =
            alist_name(((*gap).ga_data as *mut aentry_T).offset(i as isize));
        if !s.is_null() {
            if fullname {
                buf = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
                vim_FullName(s, buf, MAXPATHL as size_t, false_0 != 0);
                s = buf;
            }
            let mut fname_esc: *mut ::core::ffi::c_char = ses_escape_fname(s, flagp);
            if fprintf(
                fd,
                b"$argadd %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                fname_esc,
            ) < 0 as ::core::ffi::c_int
            {
                xfree(fname_esc as *mut ::core::ffi::c_void);
                xfree(buf as *mut ::core::ffi::c_void);
                return FAIL;
            }
            xfree(fname_esc as *mut ::core::ffi::c_void);
            xfree(buf as *mut ::core::ffi::c_void);
        }
        i += 1;
    }
    return OK;
}
unsafe extern "C" fn ses_get_fname(
    mut buf: *mut buf_T,
    mut flagp: *const ::core::ffi::c_uint,
) -> *mut ::core::ffi::c_char {
    if !(*buf).b_sfname.is_null()
        && flagp == ssop_flags.ptr() as *const ::core::ffi::c_uint
        && ssop_flags.get()
            & (kOptSsopFlagCurdir as ::core::ffi::c_int | kOptSsopFlagSesdir as ::core::ffi::c_int)
                as ::core::ffi::c_uint
            != 0
        && p_acd.get() == 0
        && did_lcd.get() == 0
    {
        return (*buf).b_sfname;
    }
    return (*buf).b_ffname;
}
unsafe extern "C" fn ses_fname(
    mut fd: *mut FILE,
    mut buf: *mut buf_T,
    mut flagp: *mut ::core::ffi::c_uint,
    mut add_eol: bool,
) -> ::core::ffi::c_int {
    let mut name: *mut ::core::ffi::c_char = ses_get_fname(buf, flagp);
    if ses_put_fname(fd, name, flagp) == FAIL
        || add_eol as ::core::ffi::c_int != 0
            && fprintf(fd, b"\n\0".as_ptr() as *const ::core::ffi::c_char) < 0 as ::core::ffi::c_int
    {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn ses_escape_fname(
    mut name: *mut ::core::ffi::c_char,
    mut _flagp: *mut ::core::ffi::c_uint,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut sname: *mut ::core::ffi::c_char =
        home_replace_save(::core::ptr::null_mut::<buf_T>(), name);
    p = sname;
    while *p as ::core::ffi::c_int != NUL {
        if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
            *p = '/' as ::core::ffi::c_char;
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    p = vim_strsave_fnameescape(sname, VSE_NONE as ::core::ffi::c_int);
    xfree(sname as *mut ::core::ffi::c_void);
    return p;
}
unsafe extern "C" fn ses_put_fname(
    mut fd: *mut FILE,
    mut name: *mut ::core::ffi::c_char,
    mut flagp: *mut ::core::ffi::c_uint,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = ses_escape_fname(name, flagp);
    let mut retval: bool = if fputs(p, fd) < 0 as ::core::ffi::c_int {
        FAIL
    } else {
        OK
    } != 0;
    xfree(p as *mut ::core::ffi::c_void);
    return retval as ::core::ffi::c_int;
}
unsafe extern "C" fn put_view(
    mut fd: *mut FILE,
    mut wp: *mut win_T,
    mut tp: *mut tabpage_T,
    mut add_edit: bool,
    mut flagp: *mut ::core::ffi::c_uint,
    mut current_arg_idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut f: ::core::ffi::c_int = 0;
    let mut did_next: bool = false_0 != 0;
    let mut do_cursor: bool = flagp == ssop_flags.ptr()
        || *flagp & kOptSsopFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint != 0;
    if (*wp).w_alist == global_alist.ptr() {
        if FAIL
            == put_line(
                fd,
                b"argglobal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            )
        {
            return FAIL;
        }
    } else if ses_arglist(
        fd,
        b"arglocal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        &raw mut (*(*wp).w_alist).al_ga,
        flagp == vop_flags.ptr()
            || *flagp & kOptSsopFlagCurdir as ::core::ffi::c_int as ::core::ffi::c_uint == 0
            || !(*tp).tp_localdir.is_null()
            || !(*wp).w_localdir.is_null(),
        flagp,
    ) == FAIL
    {
        return FAIL;
    }
    if (*wp).w_arg_idx != current_arg_idx
        && (*wp).w_arg_idx < (*(*wp).w_alist).al_ga.ga_len
        && flagp == ssop_flags.ptr()
    {
        if fprintf(
            fd,
            b"%ldargu\n\0".as_ptr() as *const ::core::ffi::c_char,
            (*wp).w_arg_idx as int64_t + 1 as int64_t,
        ) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
        did_next = true_0 != 0;
    }
    if add_edit as ::core::ffi::c_int != 0 && (!did_next || (*wp).w_arg_idx_invalid != 0) {
        let mut fname_esc: *mut ::core::ffi::c_char =
            ses_escape_fname(ses_get_fname((*wp).w_buffer, flagp), flagp);
        if bt_help((*wp).w_buffer) {
            let mut curtag: *mut ::core::ffi::c_char =
                b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            if (0 as ::core::ffi::c_int) < (*wp).w_tagstackidx
                && (*wp).w_tagstackidx <= (*wp).w_tagstacklen
            {
                curtag = (*wp).w_tagstack[((*wp).w_tagstackidx - 1 as ::core::ffi::c_int) as usize]
                    .tagname;
            }
            if put_line(
                fd,
                b"enew | setl bt=help\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            ) == FAIL
                || fprintf(
                    fd,
                    b"help %s\0".as_ptr() as *const ::core::ffi::c_char,
                    curtag,
                ) < 0 as ::core::ffi::c_int
                || put_eol(fd) == FAIL
            {
                xfree(fname_esc as *mut ::core::ffi::c_void);
                return FAIL;
            }
        } else if !(*(*wp).w_buffer).b_ffname.is_null()
            && (!bt_nofilename((*wp).w_buffer) || !(*(*wp).w_buffer).terminal.is_null())
        {
            if fprintf(
                fd,
                b"if bufexists(fnamemodify(\"%s\", \":p\")) | buffer %s | else | edit %s | endif\nif &buftype ==# 'terminal'\n  silent file %s\nendif\n\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                fname_esc,
                fname_esc,
                fname_esc,
                fname_esc,
            ) < 0 as ::core::ffi::c_int
            {
                xfree(fname_esc as *mut ::core::ffi::c_void);
                return FAIL;
            }
        } else {
            if FAIL
                == put_line(
                    fd,
                    b"enew\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                )
            {
                return FAIL;
            }
            if !(*(*wp).w_buffer).b_ffname.is_null() {
                if fprintf(
                    fd,
                    b"file %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                    fname_esc,
                ) < 0 as ::core::ffi::c_int
                {
                    xfree(fname_esc as *mut ::core::ffi::c_void);
                    return FAIL;
                }
            }
            do_cursor = false_0 != 0;
        }
        xfree(fname_esc as *mut ::core::ffi::c_void);
    }
    if (*wp).w_alt_fnum != 0 {
        let alt: *mut buf_T = buflist_findnr((*wp).w_alt_fnum);
        if flagp == ssop_flags.ptr()
            && !alt.is_null()
            && !(*alt).b_fname.is_null()
            && *(*alt).b_fname as ::core::ffi::c_int != NUL
            && (*alt).b_p_bl != 0
            && !(bt_terminal(alt) as ::core::ffi::c_int != 0
                && ssop_flags.get()
                    & kOptSsopFlagTerminal as ::core::ffi::c_int as ::core::ffi::c_uint
                    == 0)
            && (fputs(b"balt \0".as_ptr() as *const ::core::ffi::c_char, fd)
                < 0 as ::core::ffi::c_int
                || ses_fname(fd, alt, flagp, true_0 != 0) == FAIL)
        {
            return FAIL;
        }
    }
    if *flagp
        & (kOptSsopFlagOptions as ::core::ffi::c_int
            | kOptSsopFlagLocaloptions as ::core::ffi::c_int) as ::core::ffi::c_uint
        != 0
        && makemap(fd, (*wp).w_buffer) == FAIL
    {
        return FAIL;
    }
    let mut save_curwin: *mut win_T = curwin.get();
    curwin.set(wp);
    curbuf.set((*curwin.get()).w_buffer);
    if *flagp
        & (kOptSsopFlagOptions as ::core::ffi::c_int
            | kOptSsopFlagLocaloptions as ::core::ffi::c_int) as ::core::ffi::c_uint
        != 0
    {
        f = makeset(
            fd,
            OPT_LOCAL as ::core::ffi::c_int,
            (flagp == vop_flags.ptr()
                || *flagp & kOptSsopFlagOptions as ::core::ffi::c_int as ::core::ffi::c_uint == 0)
                as ::core::ffi::c_int,
        );
    } else if *flagp & kOptSsopFlagFolds as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        f = makefoldset(fd);
    } else {
        f = OK;
    }
    curwin.set(save_curwin);
    curbuf.set((*curwin.get()).w_buffer);
    if f == FAIL {
        return FAIL;
    }
    if *flagp & kOptSsopFlagFolds as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && !(*(*wp).w_buffer).b_ffname.is_null()
        && (bt_normal((*wp).w_buffer) as ::core::ffi::c_int != 0
            || bt_help((*wp).w_buffer) as ::core::ffi::c_int != 0)
    {
        if put_folds(fd, wp) == FAIL {
            return FAIL;
        }
    }
    if do_cursor {
        if (*wp).w_view_height <= 0 as ::core::ffi::c_int {
            if fprintf(
                fd,
                b"let s:l = %d\n\0".as_ptr() as *const ::core::ffi::c_char,
                (*wp).w_cursor.lnum,
            ) < 0 as ::core::ffi::c_int
            {
                return FAIL;
            }
        } else if fprintf(
            fd,
            b"let s:l = %d - ((%d * winheight(0) + %d) / %d)\n\0".as_ptr()
                as *const ::core::ffi::c_char,
            (*wp).w_cursor.lnum,
            (*wp).w_cursor.lnum - (*wp).w_topline,
            (*wp).w_view_height / 2 as ::core::ffi::c_int,
            (*wp).w_view_height,
        ) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
        if fprintf(
            fd,
            b"if s:l < 1 | let s:l = 1 | endif\nkeepjumps exe s:l\nnormal! zt\nkeepjumps %d\n\0"
                .as_ptr() as *const ::core::ffi::c_char,
            (*wp).w_cursor.lnum,
        ) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
        if (*wp).w_cursor.col == 0 as ::core::ffi::c_int {
            if FAIL
                == put_line(
                    fd,
                    b"normal! 0\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                )
            {
                return FAIL;
            }
        } else if (*wp).w_onebuf_opt.wo_wrap == 0
            && (*wp).w_leftcol > 0 as ::core::ffi::c_int
            && (*wp).w_width > 0 as ::core::ffi::c_int
        {
            if fprintf(
                fd,
                b"let s:c = %ld - ((%ld * winwidth(0) + %ld) / %ld)\nif s:c > 0\n  exe 'normal! ' . s:c . '|zs' . %ld . '|'\nelse\n\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                (*wp).w_virtcol as int64_t + 1 as int64_t,
                ((*wp).w_virtcol - (*wp).w_leftcol) as int64_t,
                ((*wp).w_width / 2 as ::core::ffi::c_int) as int64_t,
                (*wp).w_width as int64_t,
                (*wp).w_virtcol as int64_t + 1 as int64_t,
            ) < 0 as ::core::ffi::c_int
                || put_view_curpos(
                    fd,
                    wp,
                    b"  \0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                ) == FAIL
                || put_line(
                    fd,
                    b"endif\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                ) == FAIL
            {
                return FAIL;
            }
        } else if put_view_curpos(
            fd,
            wp,
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == FAIL
        {
            return FAIL;
        }
    }
    if !(*wp).w_localdir.is_null()
        && (flagp != vop_flags.ptr()
            || *flagp & kOptSsopFlagCurdir as ::core::ffi::c_int as ::core::ffi::c_uint != 0)
    {
        if fputs(b"lcd \0".as_ptr() as *const ::core::ffi::c_char, fd) < 0 as ::core::ffi::c_int
            || ses_put_fname(fd, (*wp).w_localdir, flagp) == FAIL
            || fprintf(fd, b"\n\0".as_ptr() as *const ::core::ffi::c_char) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
        did_lcd.set(true_0);
    }
    return OK;
}
unsafe extern "C" fn store_session_globals(mut fd: *mut FILE) -> ::core::ffi::c_int {
    let this_varhi_ht_: *mut hashtab_T = &raw mut (*get_globvar_dict()).dv_hashtab;
    let mut this_varhi_todo_: size_t = (*this_varhi_ht_).ht_used;
    let mut this_varhi_: *mut hashitem_T = (*this_varhi_ht_).ht_array;
    while this_varhi_todo_ != 0 {
        if !((*this_varhi_).hi_key.is_null()
            || (*this_varhi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            this_varhi_todo_ = this_varhi_todo_.wrapping_sub(1);
            let this_var: *mut dictitem_T = (*this_varhi_)
                .hi_key
                .offset(-(17 as ::core::ffi::c_ulong as isize))
                as *mut dictitem_T;
            if ((*this_var).di_tv.v_type as ::core::ffi::c_uint
                == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*this_var).di_tv.v_type as ::core::ffi::c_uint
                    == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
                && var_flavour(&raw mut (*this_var).di_key as *mut ::core::ffi::c_char)
                    as ::core::ffi::c_uint
                    == VAR_FLAVOUR_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let p: *mut ::core::ffi::c_char = vim_strsave_escaped(
                    tv_get_string(&raw mut (*this_var).di_tv),
                    b"\\\"\n\r\0".as_ptr() as *const ::core::ffi::c_char,
                );
                let mut t: *mut ::core::ffi::c_char = p;
                while *t as ::core::ffi::c_int != '\0' as ::core::ffi::c_int {
                    if *t as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                        *t = 'n' as ::core::ffi::c_char;
                    } else if *t as ::core::ffi::c_int == '\r' as ::core::ffi::c_int {
                        *t = 'r' as ::core::ffi::c_char;
                    }
                    t = t.offset(1);
                }
                if fprintf(
                    fd,
                    b"let %s = %c%s%c\0".as_ptr() as *const ::core::ffi::c_char,
                    &raw mut (*this_var).di_key as *mut ::core::ffi::c_char,
                    if (*this_var).di_tv.v_type as ::core::ffi::c_uint
                        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        '"' as ::core::ffi::c_int
                    } else {
                        ' ' as ::core::ffi::c_int
                    },
                    p,
                    if (*this_var).di_tv.v_type as ::core::ffi::c_uint
                        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        '"' as ::core::ffi::c_int
                    } else {
                        ' ' as ::core::ffi::c_int
                    },
                ) < 0 as ::core::ffi::c_int
                    || put_eol(fd) == 0 as ::core::ffi::c_int
                {
                    xfree(p as *mut ::core::ffi::c_void);
                    return 0 as ::core::ffi::c_int;
                }
                xfree(p as *mut ::core::ffi::c_void);
            } else if (*this_var).di_tv.v_type as ::core::ffi::c_uint
                == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
                && var_flavour(&raw mut (*this_var).di_key as *mut ::core::ffi::c_char)
                    as ::core::ffi::c_uint
                    == VAR_FLAVOUR_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut f: float_T = (*this_var).di_tv.vval.v_float;
                let mut sign: ::core::ffi::c_int = ' ' as ::core::ffi::c_int;
                if f < 0 as ::core::ffi::c_int as float_T {
                    f = -f;
                    sign = '-' as ::core::ffi::c_int;
                }
                if fprintf(
                    fd,
                    b"let %s = %c%f\0".as_ptr() as *const ::core::ffi::c_char,
                    &raw mut (*this_var).di_key as *mut ::core::ffi::c_char,
                    sign,
                    f,
                ) < 0 as ::core::ffi::c_int
                    || put_eol(fd) == 0 as ::core::ffi::c_int
                {
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
        this_varhi_ = this_varhi_.offset(1);
    }
    return OK;
}
unsafe extern "C" fn makeopens(
    mut fd: *mut FILE,
    mut dirnow: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut only_save_windows: bool = true_0 != 0;
    let mut restore_size: bool = true_0 != 0;
    let mut edited_win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut tab_firstwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut tab_topframe: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
    let mut cur_arg_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut next_arg_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if ssop_flags.get() & kOptSsopFlagBuffers as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        only_save_windows = false_0 != 0;
    }
    if FAIL
        == put_line(
            fd,
            b"let v:this_session=expand(\"<sfile>:p\")\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        )
    {
        return FAIL;
    }
    if FAIL
        == put_line(
            fd,
            b"doautoall SessionLoadPre\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        )
    {
        return FAIL;
    }
    if ssop_flags.get() & kOptSsopFlagGlobals as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        if store_session_globals(fd) == FAIL {
            return FAIL;
        }
    }
    if FAIL
        == put_line(
            fd,
            b"silent only\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        )
    {
        return FAIL;
    }
    if ssop_flags.get() & kOptSsopFlagTabpages as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && put_line(
            fd,
            b"silent tabonly\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == FAIL
    {
        return FAIL;
    }
    if ssop_flags.get() & kOptSsopFlagSesdir as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        if FAIL
            == put_line(
                fd,
                b"exe \"cd \" . escape(expand(\"<sfile>:p:h\"), ' ')\0".as_ptr()
                    as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            )
        {
            return FAIL;
        }
    } else if ssop_flags.get() & kOptSsopFlagCurdir as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0
    {
        let mut sname: *mut ::core::ffi::c_char = home_replace_save(
            ::core::ptr::null_mut::<buf_T>(),
            if !(*globaldir.ptr()).is_null() {
                globaldir.get()
            } else {
                dirnow
            },
        );
        let mut fname_esc: *mut ::core::ffi::c_char = ses_escape_fname(sname, ssop_flags.ptr());
        if fprintf(
            fd,
            b"cd %s\n\0".as_ptr() as *const ::core::ffi::c_char,
            fname_esc,
        ) < 0 as ::core::ffi::c_int
        {
            xfree(fname_esc as *mut ::core::ffi::c_void);
            xfree(sname as *mut ::core::ffi::c_void);
            return FAIL;
        }
        xfree(fname_esc as *mut ::core::ffi::c_void);
        xfree(sname as *mut ::core::ffi::c_void);
    }
    if fprintf(
        fd,
        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
        b"if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''\n  let s:wipebuf = bufnr('%')\nendif\n\0"
            .as_ptr() as *const ::core::ffi::c_char,
    ) < 0 as ::core::ffi::c_int
    {
        return FAIL;
    }
    if ssop_flags.get() & kOptSsopFlagOptions as ::core::ffi::c_int as ::core::ffi::c_uint
        == 0 as ::core::ffi::c_uint
    {
        if FAIL
            == put_line(
                fd,
                b"let s:shortmess_save = &shortmess\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            )
        {
            return FAIL;
        }
    }
    if FAIL
        == put_line(
            fd,
            b"set shortmess+=aoO\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        )
    {
        return FAIL;
    }
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if !(only_save_windows as ::core::ffi::c_int != 0
            && (*buf).b_nwindows == 0 as ::core::ffi::c_int)
            && !((*buf).b_help as ::core::ffi::c_int != 0
                && ssop_flags.get() & kOptSsopFlagHelp as ::core::ffi::c_int as ::core::ffi::c_uint
                    == 0)
            && !(bt_terminal(buf) as ::core::ffi::c_int != 0
                && ssop_flags.get()
                    & kOptSsopFlagTerminal as ::core::ffi::c_int as ::core::ffi::c_uint
                    == 0)
            && !(*buf).b_fname.is_null()
            && (*buf).b_p_bl != 0
        {
            if fprintf(
                fd,
                b"badd +%ld \0".as_ptr() as *const ::core::ffi::c_char,
                if (*buf).b_wininfo.size == 0 as size_t {
                    1 as int64_t
                } else {
                    (**(*buf)
                        .b_wininfo
                        .items
                        .offset(0 as ::core::ffi::c_int as isize))
                    .wi_mark
                    .mark
                    .lnum as int64_t
                },
            ) < 0 as ::core::ffi::c_int
                || ses_fname(fd, buf, ssop_flags.ptr(), true_0 != 0) == FAIL
            {
                return FAIL;
            }
        }
        buf = (*buf).b_next;
    }
    if ses_arglist(
        fd,
        b"argglobal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        &raw mut (*global_alist.ptr()).al_ga,
        ssop_flags.get() & kOptSsopFlagCurdir as ::core::ffi::c_int as ::core::ffi::c_uint == 0,
        ssop_flags.ptr(),
    ) == FAIL
    {
        return FAIL;
    }
    if ssop_flags.get() & kOptSsopFlagResize as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        if fprintf(
            fd,
            b"set lines=%ld columns=%ld\n\0".as_ptr() as *const ::core::ffi::c_char,
            Rows.get() as int64_t,
            Columns.get() as int64_t,
        ) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
    }
    let mut restore_stal: bool = false_0 != 0;
    if p_stal.get() == 1 as OptInt && !(*first_tabpage.get()).tp_next.is_null() {
        if FAIL
            == put_line(
                fd,
                b"set stal=2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            )
        {
            return FAIL;
        }
        restore_stal = true_0 != 0;
    }
    if ssop_flags.get() & kOptSsopFlagTabpages as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            if !(*tp).tp_next.is_null()
                && put_line(
                    fd,
                    b"tabnew +setlocal\\ bufhidden=wipe\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                ) == FAIL
            {
                return FAIL;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        if !(*first_tabpage.get()).tp_next.is_null()
            && put_line(
                fd,
                b"tabrewind\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) == FAIL
        {
            return FAIL;
        }
    }
    let mut restore_height_width: bool = false_0 != 0;
    let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp_0.is_null() {
        let mut need_tabnext: bool = false_0 != 0;
        let mut cnr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        if ssop_flags.get() & kOptSsopFlagTabpages as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        {
            if tp_0 == curtab.get() {
                tab_firstwin = firstwin.get();
                tab_topframe = topframe.get();
            } else {
                tab_firstwin = (*tp_0).tp_firstwin;
                tab_topframe = (*tp_0).tp_topframe;
            }
            if tp_0 != first_tabpage.get() {
                need_tabnext = true_0 != 0;
            }
        } else {
            tp_0 = curtab.get() as *mut tabpage_T;
            tab_firstwin = firstwin.get();
            tab_topframe = topframe.get();
        }
        let mut wp: *mut win_T = tab_firstwin;
        while !wp.is_null() {
            if ses_do_win(wp) != 0
                && !(*(*wp).w_buffer).b_ffname.is_null()
                && !bt_help((*wp).w_buffer)
                && !bt_nofilename((*wp).w_buffer)
            {
                if need_tabnext as ::core::ffi::c_int != 0
                    && put_line(
                        fd,
                        b"tabnext\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    ) == FAIL
                {
                    return FAIL;
                }
                need_tabnext = false_0 != 0;
                if fputs(b"edit \0".as_ptr() as *const ::core::ffi::c_char, fd)
                    < 0 as ::core::ffi::c_int
                    || ses_fname(fd, (*wp).w_buffer, ssop_flags.ptr(), true_0 != 0) == FAIL
                {
                    return FAIL;
                }
                if (*wp).w_arg_idx_invalid == 0 {
                    edited_win = wp;
                }
                break;
            } else {
                wp = (*wp).w_next;
            }
        }
        if need_tabnext as ::core::ffi::c_int != 0
            && put_line(
                fd,
                b"tabnext\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) == FAIL
        {
            return FAIL;
        }
        if (*tab_topframe).fr_layout as ::core::ffi::c_int != FR_LEAF {
            if FAIL
                == put_line(
                    fd,
                    b"let s:save_splitbelow = &splitbelow\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                )
            {
                return FAIL;
            }
            if FAIL
                == put_line(
                    fd,
                    b"let s:save_splitright = &splitright\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                )
            {
                return FAIL;
            }
            if FAIL
                == put_line(
                    fd,
                    b"set splitbelow splitright\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                )
            {
                return FAIL;
            }
            if ses_win_rec(fd, tab_topframe) == FAIL {
                return FAIL;
            }
            if FAIL
                == put_line(
                    fd,
                    b"let &splitbelow = s:save_splitbelow\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                )
            {
                return FAIL;
            }
            if FAIL
                == put_line(
                    fd,
                    b"let &splitright = s:save_splitright\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                )
            {
                return FAIL;
            }
        }
        let mut nr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut wp_0: *mut win_T = tab_firstwin;
        while !wp_0.is_null() {
            if ses_do_win(wp_0) != 0 {
                nr += 1;
            } else if !(*wp_0).w_floating {
                restore_size = false_0 != 0;
            }
            if curwin.get() == wp_0 {
                cnr = nr;
            }
            wp_0 = (*wp_0).w_next;
        }
        if !tab_firstwin.is_null() && !(*tab_firstwin).w_next.is_null() {
            if FAIL
                == put_line(
                    fd,
                    b"wincmd t\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                )
            {
                return FAIL;
            }
            if !restore_height_width {
                if FAIL
                    == put_line(
                        fd,
                        b"let s:save_winminheight = &winminheight\0".as_ptr()
                            as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    )
                {
                    return FAIL;
                }
                if FAIL
                    == put_line(
                        fd,
                        b"let s:save_winminwidth = &winminwidth\0".as_ptr()
                            as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    )
                {
                    return FAIL;
                }
            }
            if fprintf(
                fd,
                b"set winminheight=0\nset winheight=1\nset winminwidth=0\nset winwidth=1\n\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            ) < 0 as ::core::ffi::c_int
            {
                return FAIL;
            }
            restore_height_width = true_0 != 0;
        }
        if nr > 1 as ::core::ffi::c_int && ses_winsizes(fd, restore_size, tab_firstwin) == FAIL {
            return FAIL;
        }
        if ssop_flags.get() & kOptSsopFlagCurdir as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && !(*tp_0).tp_localdir.is_null()
        {
            if fputs(b"tcd \0".as_ptr() as *const ::core::ffi::c_char, fd) < 0 as ::core::ffi::c_int
                || ses_put_fname(fd, (*tp_0).tp_localdir, ssop_flags.ptr()) == FAIL
                || put_eol(fd) == FAIL
            {
                return FAIL;
            }
            did_lcd.set(true_0);
        }
        let mut wp_1: *mut win_T = tab_firstwin;
        while !wp_1.is_null() {
            if ses_do_win(wp_1) != 0 {
                if put_view(
                    fd,
                    wp_1,
                    tp_0 as *mut tabpage_T,
                    wp_1 != edited_win,
                    ssop_flags.ptr(),
                    cur_arg_idx,
                ) == FAIL
                {
                    return FAIL;
                }
                if nr > 1 as ::core::ffi::c_int
                    && put_line(
                        fd,
                        b"wincmd w\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    ) == FAIL
                {
                    return FAIL;
                }
                next_arg_idx = (*wp_1).w_arg_idx;
            }
            wp_1 = (*wp_1).w_next;
        }
        cur_arg_idx = next_arg_idx;
        if cnr > 1 as ::core::ffi::c_int
            && fprintf(
                fd,
                b"%dwincmd w\n\0".as_ptr() as *const ::core::ffi::c_char,
                cnr,
            ) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
        if nr > 1 as ::core::ffi::c_int && ses_winsizes(fd, restore_size, tab_firstwin) == FAIL {
            return FAIL;
        }
        if ssop_flags.get() & kOptSsopFlagTabpages as ::core::ffi::c_int as ::core::ffi::c_uint == 0
        {
            break;
        }
        tp_0 = (*tp_0).tp_next as *mut tabpage_T;
    }
    if ssop_flags.get() & kOptSsopFlagTabpages as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        if fprintf(
            fd,
            b"tabnext %d\n\0".as_ptr() as *const ::core::ffi::c_char,
            tabpage_index(curtab.get()),
        ) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
    }
    if restore_stal as ::core::ffi::c_int != 0
        && put_line(
            fd,
            b"set stal=1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == FAIL
    {
        return FAIL;
    }
    if fprintf(
        fd,
        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
        b"if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'\n  silent exe 'bwipe ' . s:wipebuf\nendif\nunlet! s:wipebuf\n\0"
            .as_ptr() as *const ::core::ffi::c_char,
    ) < 0 as ::core::ffi::c_int
    {
        return FAIL;
    }
    if fprintf(
        fd,
        b"set winheight=%ld winwidth=%ld\n\0".as_ptr() as *const ::core::ffi::c_char,
        p_wh.get(),
        p_wiw.get(),
    ) < 0 as ::core::ffi::c_int
    {
        return FAIL;
    }
    if ssop_flags.get() & kOptSsopFlagOptions as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        if fprintf(
            fd,
            b"set shortmess=%s\n\0".as_ptr() as *const ::core::ffi::c_char,
            p_shm.get(),
        ) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
    } else if FAIL
        == put_line(
            fd,
            b"let &shortmess = s:shortmess_save\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        )
    {
        return FAIL;
    }
    if restore_height_width {
        if FAIL
            == put_line(
                fd,
                b"let &winminheight = s:save_winminheight\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            )
        {
            return FAIL;
        }
        if FAIL
            == put_line(
                fd,
                b"let &winminwidth = s:save_winminwidth\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            )
        {
            return FAIL;
        }
    }
    if fprintf(
        fd,
        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
        b"let s:sx = expand(\"<sfile>:p:r\").\"x.vim\"\nif filereadable(s:sx)\n  exe \"source \" . fnameescape(s:sx)\nendif\n\0"
            .as_ptr() as *const ::core::ffi::c_char,
    ) < 0 as ::core::ffi::c_int
    {
        return FAIL;
    }
    return OK;
}
pub unsafe fn ex_loadview(mut eap: *mut exarg_T) {
    let mut fname: *mut ::core::ffi::c_char = get_view_file(*(*eap).arg);
    if fname.is_null() {
        return;
    }
    if do_source(
        fname,
        false_0 != 0,
        DOSO_NONE as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    ) == FAIL
    {
        semsg_c!(
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            fname,
        );
    }
    xfree(fname as *mut ::core::ffi::c_void);
}
pub unsafe fn ex_mkrc(mut eap: *mut exarg_T) {
    let mut view_session: bool = false_0 != 0;
    let mut using_vdir: ::core::ffi::c_int = false_0;
    let mut viewFile: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_mksession as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_mkview as ::core::ffi::c_int
    {
        view_session = true_0 != 0;
    }
    did_lcd.set(false_0);
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_mkview as ::core::ffi::c_int
        && (*(*eap).arg as ::core::ffi::c_int == NUL
            || ascii_isdigit(*(*eap).arg as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                && *(*eap).arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == NUL)
    {
        (*eap).forceit = true_0;
        fname = get_view_file(*(*eap).arg);
        if fname.is_null() {
            return;
        }
        viewFile = fname;
        using_vdir = true_0;
    } else if *(*eap).arg as ::core::ffi::c_int != NUL {
        fname = (*eap).arg;
    } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_mkvimrc as ::core::ffi::c_int {
        fname = VIMRC_FILE.as_ptr() as *mut ::core::ffi::c_char;
    } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_mksession as ::core::ffi::c_int {
        fname = SESSION_FILE.as_ptr() as *mut ::core::ffi::c_char;
    } else {
        fname = EXRC_FILE.as_ptr() as *mut ::core::ffi::c_char;
    }
    if using_vdir != 0 && !os_isdir(p_vdir.get()) {
        vim_mkdir_emsg(p_vdir.get(), 0o755 as ::core::ffi::c_int);
    }
    let mut fd: *mut FILE = open_exfile(
        fname,
        (*eap).forceit,
        WRITEBIN.as_ptr() as *mut ::core::ffi::c_char,
    );
    if !fd.is_null() {
        let mut failed: bool = false_0 != 0;
        let mut flagp: *mut ::core::ffi::c_uint = ::core::ptr::null_mut::<::core::ffi::c_uint>();
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_mkview as ::core::ffi::c_int {
            flagp = vop_flags.ptr();
        } else {
            flagp = ssop_flags.ptr();
        }
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_mkvimrc as ::core::ffi::c_int {
            put_line(
                fd,
                b"version 6.0\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
        }
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_mksession as ::core::ffi::c_int {
            if put_line(
                fd,
                b"let SessionLoad = 1\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            ) == FAIL
            {
                failed = true_0 != 0;
            }
        }
        if !view_session
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_mksession as ::core::ffi::c_int
                && *flagp & kOptSsopFlagOptions as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        {
            let mut flags: ::core::ffi::c_int = OPT_GLOBAL as ::core::ffi::c_int;
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_mksession as ::core::ffi::c_int
                && *flagp & kOptSsopFlagSkiprtp as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            {
                flags |= OPT_SKIPRTP as ::core::ffi::c_int;
            }
            failed = failed as ::core::ffi::c_int
                | (makemap(fd, ::core::ptr::null_mut::<buf_T>()) == FAIL
                    || makeset(fd, flags, false_0) == FAIL) as ::core::ffi::c_int
                != 0;
        }
        if !failed && view_session as ::core::ffi::c_int != 0 {
            if put_line(
                fd,
                b"let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1\0"
                    .as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) == FAIL
            {
                failed = true_0 != 0;
            }
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_mksession as ::core::ffi::c_int {
                let mut dirnow: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                dirnow = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
                if os_dirname(dirnow, MAXPATHL as size_t) == FAIL
                    || os_chdir(dirnow) != 0 as ::core::ffi::c_int
                {
                    *dirnow = NUL as ::core::ffi::c_char;
                }
                if *dirnow as ::core::ffi::c_int != NUL
                    && ssop_flags.get()
                        & kOptSsopFlagSesdir as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                {
                    if vim_chdirfile(fname, kCdCauseOther) == OK {
                        shorten_fnames(true_0);
                    }
                } else if *dirnow as ::core::ffi::c_int != NUL
                    && ssop_flags.get()
                        & kOptSsopFlagCurdir as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    && !(*globaldir.ptr()).is_null()
                {
                    if os_chdir(globaldir.get()) == 0 as ::core::ffi::c_int {
                        shorten_fnames(true_0);
                    }
                }
                failed = failed as ::core::ffi::c_int
                    | (makeopens(fd, dirnow) == FAIL) as ::core::ffi::c_int
                    != 0;
                if *dirnow as ::core::ffi::c_int != NUL
                    && (ssop_flags.get()
                        & kOptSsopFlagSesdir as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                        || ssop_flags.get()
                            & kOptSsopFlagCurdir as ::core::ffi::c_int as ::core::ffi::c_uint
                            != 0
                            && !(*globaldir.ptr()).is_null())
                {
                    if os_chdir(dirnow) != 0 as ::core::ffi::c_int {
                        emsg(gettext(&raw const e_prev_dir as *const ::core::ffi::c_char));
                    }
                    shorten_fnames(true_0);
                }
                xfree(dirnow as *mut ::core::ffi::c_void);
            } else {
                failed = failed as ::core::ffi::c_int
                    | (put_view(
                        fd,
                        curwin.get(),
                        curtab.get(),
                        using_vdir == 0,
                        flagp,
                        -1 as ::core::ffi::c_int,
                    ) == FAIL) as ::core::ffi::c_int
                    != 0;
            }
            if fprintf(
                fd,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"let &g:so = s:so_save | let &g:siso = s:siso_save\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ) < 0 as ::core::ffi::c_int
            {
                failed = true_0 != 0;
            }
            if p_hls.get() != 0
                && fprintf(
                    fd,
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    b"set hlsearch\n\0".as_ptr() as *const ::core::ffi::c_char,
                ) < 0 as ::core::ffi::c_int
            {
                failed = true_0 != 0;
            }
            if no_hlsearch.get() as ::core::ffi::c_int != 0
                && fprintf(
                    fd,
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    b"nohlsearch\n\0".as_ptr() as *const ::core::ffi::c_char,
                ) < 0 as ::core::ffi::c_int
            {
                failed = true_0 != 0;
            }
            if fprintf(
                fd,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"doautoall SessionLoadPost\n\0".as_ptr() as *const ::core::ffi::c_char,
            ) < 0 as ::core::ffi::c_int
            {
                failed = true_0 != 0;
            }
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_mksession as ::core::ffi::c_int {
                if fprintf(
                    fd,
                    b"unlet SessionLoad\n\0".as_ptr() as *const ::core::ffi::c_char,
                ) < 0 as ::core::ffi::c_int
                {
                    failed = true_0 != 0;
                }
            }
        }
        if put_line(
            fd,
            b"\" vim: set ft=vim :\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        ) == FAIL
        {
            failed = true_0 != 0;
        }
        failed = failed as ::core::ffi::c_int | fclose(fd) != 0;
        if failed {
            emsg(gettext(&raw const e_write as *const ::core::ffi::c_char));
        } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_mksession as ::core::ffi::c_int {
            let tbuf: *mut ::core::ffi::c_char =
                xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
            if vim_FullName(fname, tbuf, MAXPATHL as size_t, false_0 != 0) == OK {
                set_vim_var_string(VV_THIS_SESSION, tbuf, -1 as ptrdiff_t);
            }
            xfree(tbuf as *mut ::core::ffi::c_void);
        }
    }
    xfree(viewFile as *mut ::core::ffi::c_void);
    apply_autocmds(
        EVENT_SESSIONWRITEPOST,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
}
unsafe extern "C" fn get_view_file(mut c: ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    if (*curbuf.get()).b_ffname.is_null() {
        emsg(gettext(&raw const e_noname as *const ::core::ffi::c_char));
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut sname: *mut ::core::ffi::c_char =
        home_replace_save(::core::ptr::null_mut::<buf_T>(), (*curbuf.get()).b_ffname);
    let mut len: size_t = 0 as size_t;
    let mut p: *mut ::core::ffi::c_char = sname;
    while *p != 0 {
        if *p as ::core::ffi::c_int == '=' as ::core::ffi::c_int
            || vim_ispathsep(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        {
            len = len.wrapping_add(1);
        }
        p = p.offset(1);
    }
    let mut retval: *mut ::core::ffi::c_char = xmalloc(
        strlen(sname)
            .wrapping_add(len)
            .wrapping_add(strlen(p_vdir.get()))
            .wrapping_add(9 as size_t),
    ) as *mut ::core::ffi::c_char;
    strcpy(retval, p_vdir.get());
    add_pathsep(retval);
    let mut s: *mut ::core::ffi::c_char = retval.add(strlen(retval));
    let mut p_0: *mut ::core::ffi::c_char = sname;
    while *p_0 != 0 {
        if *p_0 as ::core::ffi::c_int == '=' as ::core::ffi::c_int {
            let c2rust_fresh0 = s;
            s = s.offset(1);
            *c2rust_fresh0 = '=' as ::core::ffi::c_char;
            let c2rust_fresh1 = s;
            s = s.offset(1);
            *c2rust_fresh1 = '=' as ::core::ffi::c_char;
        } else if vim_ispathsep(*p_0 as ::core::ffi::c_int) {
            let c2rust_fresh2 = s;
            s = s.offset(1);
            *c2rust_fresh2 = '=' as ::core::ffi::c_char;
            let c2rust_fresh3 = s;
            s = s.offset(1);
            *c2rust_fresh3 = '+' as ::core::ffi::c_char;
        } else {
            let c2rust_fresh4 = s;
            s = s.offset(1);
            *c2rust_fresh4 = *p_0;
        }
        p_0 = p_0.offset(1);
    }
    let c2rust_fresh5 = s;
    s = s.offset(1);
    *c2rust_fresh5 = '=' as ::core::ffi::c_char;
    let c2rust_fresh6 = s;
    s = s.offset(1);
    *c2rust_fresh6 = c;
    xmemcpyz(
        s as *mut ::core::ffi::c_void,
        b".vim\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
    );
    xfree(sname as *mut ::core::ffi::c_void);
    return retval;
}
pub unsafe extern "C" fn put_eol(mut fd: *mut FILE) -> ::core::ffi::c_int {
    if putc('\n' as ::core::ffi::c_int, fd) < 0 as ::core::ffi::c_int {
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn put_line(
    mut fd: *mut FILE,
    mut s: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if fprintf(fd, b"%s\n\0".as_ptr() as *const ::core::ffi::c_char, s) < 0 as ::core::ffi::c_int {
        return FAIL;
    }
    return OK;
}
pub const EXRC_FILE: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b".exrc\0") };
pub const VIMRC_FILE: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b".nvimrc\0") };
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const WRITEBIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"wb\0") };
