use crate::src::nvim::autocmd::is_aucmd_win;
use crate::src::nvim::buffer::{
    buf_hide, buf_is_empty, buf_set_name, buflist_add, buflist_findnr, bufref_valid,
    curbuf_reusable, maketitle, otherfile, set_bufref,
};
use crate::src::nvim::eval::typval::{
    tv_get_number, tv_get_number_chk, tv_list_alloc_ret, tv_list_append_string,
};
use crate::src::nvim::eval::window::{find_tabwin, find_win_by_nr_or_id};
use crate::src::nvim::ex_cmds::do_ecmd;
use crate::src::nvim::ex_cmds2::{autowrite, check_changed};
use crate::src::nvim::ex_getln::gotocmdline;
use crate::src::nvim::fileio::file_pat_to_reg_pat;
use crate::src::nvim::garray::{ga_clear, ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    Columns, arg_had_last, autocmd_no_enter, autocmd_no_leave, cmdmod, cmdwin_type, curbuf, curtab,
    curwin, e_cannot_change_arglist_recursively, e_cmdwin, e_invarg, e_invrange, e_nomatch,
    e_nomatch2, first_tabpage, firstwin, global_alist, got_int, lastused_tabpage, lastwin,
    max_alist_id, p_ea, p_fic, p_tpm, tabpage_move_disallowed,
};
use crate::src::nvim::mark::{setmark, setpcmark};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xstrdup};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::normal::reset_VIsual_and_resel;
use crate::src::nvim::option::magic_isset;
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::libc::{__assert_fail, gettext, memmove, strlen};
use crate::src::nvim::path::{
    FullName_save, expand_wildcards, fix_fname, gen_expand_wildcards, path_fnamecmp,
    path_full_compare,
};
use crate::src::nvim::types::*;
use crate::src::nvim::undo::bufIsChanged;
use crate::src::nvim::version::list_in_columns;
use crate::src::nvim::window::{
    check_can_set_curbuf_forceit, goto_tabpage_tp, lastwin_nofloating, tabpage_index,
    valid_tabpage, win_close, win_enter, win_move_after, win_split, win_valid,
};
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;
unsafe extern "C" {
    fn vim_regcomp(expr_arg: *const c_char, re_flags: c_int) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const c_char, col: colnr_T) -> bool;
}

/// Constants the transpiler copied in from the headers this module includes.
mod flag {
    use super::{CMD_index, VarType, bln_values, c_int, c_uint, file_comparison};

    /// `alist_set`/`alist_add` "what to do with the name" selector.
    pub const AL_SET: c_uint = 1;
    pub const AL_ADD: c_uint = 2;
    pub const AL_DEL: c_uint = 3;

    /// `buflist_new` flags.
    pub const BLN_CURBUF: bln_values = 1;
    pub const BLN_LISTED: bln_values = 2;

    /// `check_changed` flags.
    pub const CCGD_AW: c_uint = 1;
    pub const CCGD_MULTWIN: c_uint = 2;
    pub const CCGD_FORCEIT: c_uint = 4;
    pub const CCGD_EXCMD: c_uint = 16;

    /// `do_ecmd` flags and its `lnum` sentinels.
    pub const ECMD_HIDE: c_uint = 1;
    pub const ECMD_OLDBUF: c_uint = 4;
    pub const ECMD_FORCEIT: c_uint = 8;
    pub const ECMD_ONE: c_int = 1;
    pub const ECMD_LAST: c_int = -1;

    /// `expand_wildcards`/`gen_expand_wildcards` flags.
    pub const EW_DIR: c_uint = 1;
    pub const EW_FILE: c_uint = 2;
    pub const EW_NOTFOUND: c_uint = 4;
    pub const EW_ADDSLASH: c_uint = 8;
    pub const EW_NOTWILD: c_uint = 1024;

    /// `win_split` flags.
    pub const WSP_ROOM: c_uint = 1;
    pub const WSP_BELOW: c_uint = 64;

    pub const kEqualFiles: file_comparison = 1;

    pub const VAR_UNKNOWN: VarType = 0;
    pub const VAR_NUMBER: VarType = 1;
    pub const VAR_STRING: VarType = 2;

    pub const CMD_args: CMD_index = 7;
    pub const CMD_argdo: CMD_index = 10;
    pub const CMD_argglobal: CMD_index = 13;
    pub const CMD_arglocal: CMD_index = 14;
    pub const CMD_drop: CMD_index = 130;
    pub const CMD_snext: CMD_index = 414;
}
use flag::*;

mod all;
mod eval;

pub use all::{arg_all, ex_all};
pub use eval::{f_argc, f_argidx, f_arglistid, f_argv};

/// `AARGLIST(al)` and `ALIST_COUNT(al)`: an argument list's entries and length.
/// The transpiler spelled these out at every use; the C had them as macros.
fn alist_entries(al: *mut alist_T) -> (*mut aentry_T, c_int) {
    let ga = unsafe { &(*al).al_ga };
    (ga.ga_data as *mut aentry_T, ga.ga_len)
}

fn alist_arg(al: *mut alist_T, n: c_int) -> *mut aentry_T {
    alist_entries(al).0.wrapping_add(n as usize)
}

fn alist_count(al: *mut alist_T) -> c_int {
    alist_entries(al).1
}

/// `WARGLIST(wp)[n]` and `WARGCOUNT(wp)`: a window's argument list.
fn win_alist(wp: *mut win_T) -> *mut alist_T {
    unsafe { (*wp).w_alist }
}

fn warg(wp: *mut win_T, n: c_int) -> *mut aentry_T {
    alist_arg(win_alist(wp), n)
}

fn wargcount(wp: *mut win_T) -> c_int {
    alist_count(win_alist(wp))
}

/// `ARGLIST[n]` and `ARGCOUNT`: the current window's argument list.
fn arg(n: c_int) -> *mut aentry_T {
    warg(curwin.get(), n)
}

fn argcount() -> c_int {
    wargcount(curwin.get())
}

/// `ARGCOUNT` as an assignable place.
fn argcount_mut<'a>() -> &'a mut c_int {
    unsafe { &mut (*win_alist(curwin.get())).al_ga.ga_len }
}

/// `alist_name(ARGLIST + n)`: the n-th argument's file name.
fn arg_name(n: c_int) -> *mut c_char {
    unsafe { alist_name(arg(n)) }
}

static arglist_locked: GlobalCell<bool> = GlobalCell::new(false);
unsafe extern "C" fn check_arglist_locked() -> c_int {
    if arglist_locked.get() {
        emsg(gettext(
            &raw const e_cannot_change_arglist_recursively as *const c_char,
        ));
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn alist_clear(mut al: *mut alist_T) {
    if check_arglist_locked() == FAIL {
        return;
    }
    let mut _gap: *mut garray_T = &raw mut (*al).al_ga;
    if !(*_gap).ga_data.is_null() {
        let mut i: c_int = 0;
        while i < (*_gap).ga_len {
            let mut _item: *mut aentry_T = ((*_gap).ga_data as *mut aentry_T).offset(i as isize);
            xfree((*_item).ae_fname as *mut c_void);
            i += 1;
        }
    }
    ga_clear(_gap);
}
pub unsafe extern "C" fn alist_init(mut al: *mut alist_T) {
    ga_init(&raw mut (*al).al_ga, size_of::<aentry_T>() as c_int, 5);
}
pub unsafe extern "C" fn alist_unlink(mut al: *mut alist_T) {
    if al != global_alist.ptr() && {
        (*al).al_refcount -= 1;
        (*al).al_refcount <= 0
    } {
        alist_clear(al);
        xfree(al as *mut c_void);
    }
}
unsafe extern "C" fn alist_new() {
    (*curwin.get()).w_alist = xmalloc(size_of::<alist_T>()) as *mut alist_T;
    (*(*curwin.get()).w_alist).al_refcount = 1;
    (*max_alist_id.ptr()) += 1;
    (*(*curwin.get()).w_alist).id = max_alist_id.get();
    alist_init((*curwin.get()).w_alist);
}
unsafe extern "C" fn alist_set(
    mut al: *mut alist_T,
    mut count: c_int,
    mut files: *mut *mut c_char,
    mut use_curbuf: c_int,
    mut fnum_list: *mut c_int,
    mut fnum_len: c_int,
) {
    if check_arglist_locked() == FAIL {
        return;
    }
    alist_clear(al);
    ga_grow(&raw mut (*al).al_ga, count);
    let mut i: c_int = 0;
    while i < count {
        if got_int.get() {
            while i < count {
                let c2rust_fresh0 = i;
                i = i + 1;
                xfree(*files.offset(c2rust_fresh0 as isize) as *mut c_void);
            }
            break;
        } else {
            if !fnum_list.is_null() && i < fnum_len {
                arglist_locked.set(true);
                buf_set_name(*fnum_list.offset(i as isize), *files.offset(i as isize));
                arglist_locked.set(false);
            }
            alist_add(
                al,
                *files.offset(i as isize),
                if use_curbuf != 0 { 2 } else { 1 },
            );
            os_breakcheck();
            i += 1;
        }
    }
    xfree(files as *mut c_void);
    if al == global_alist.ptr() {
        arg_had_last.set(false);
    }
}
pub unsafe extern "C" fn alist_add(
    mut al: *mut alist_T,
    mut fname: *mut c_char,
    mut set_fnum: c_int,
) {
    let mut wp: *mut win_T = curwin.get();
    if fname.is_null() {
        return;
    }
    if check_arglist_locked() == FAIL {
        return;
    }
    arglist_locked.set(true);
    (*wp).w_locked = true;
    (*((*al).al_ga.ga_data as *mut aentry_T).offset((*al).al_ga.ga_len as isize)).ae_fname = fname;
    if set_fnum > 0 {
        (*((*al).al_ga.ga_data as *mut aentry_T).offset((*al).al_ga.ga_len as isize)).ae_fnum =
            buflist_add(
                fname,
                BLN_LISTED as c_int
                    | (if set_fnum == 2 {
                        BLN_CURBUF as c_int
                    } else {
                        0
                    }),
            );
    }
    (*al).al_ga.ga_len += 1;
    arglist_locked.set(false);
    (*wp).w_locked = false;
}
/// Terminates the first argument in `s` and answers where the next one starts.
///
/// A backslash escapes the byte after it — both are kept, `alist_add`'s caller
/// strips them later — and a backtick suspends the "whitespace ends the
/// argument" rule, so `` `shell command` `` stays one argument.
///
/// The C walked a read and a write pointer in step and copied every byte over
/// itself; since neither branch ever consumes more bytes than it writes, the
/// only lasting change is the terminator, which lands on the whitespace that
/// ended the argument (or on the string's own NUL, where it is a no-op).
pub fn split_one_arg(s: &mut [u8]) -> usize {
    let mut inbacktick = false;
    let mut end = 0;
    while end < s.len() && s[end] != 0 {
        // `rem_backslash`: a backslash is only an escape if something follows.
        if s[end] == b'\\' && end + 1 < s.len() && s[end + 1] != 0 {
            end += 1;
        } else {
            if !inbacktick && ascii_isspace(s[end] as c_int) {
                break;
            }
            if s[end] == b'`' {
                inbacktick = !inbacktick;
            }
        }
        end += 1;
    }
    // `skipwhite`: space and tab only.
    let mut next = end;
    while next < s.len() && (s[next] == b' ' || s[next] == b'\t') {
        next += 1;
    }
    if end < s.len() {
        s[end] = 0;
    }
    next
}

unsafe extern "C" fn get_arglist(gap: *mut garray_T, str: *mut c_char, escaped: bool) {
    ga_init(gap, size_of::<*mut c_char>() as c_int, 20);
    // One `strlen` up front rather than one per argument: `split_one_arg` only
    // ever shortens what it is given, so the original length still bounds it.
    let total = strlen(str) as usize;
    let buf = core::slice::from_raw_parts_mut(str.cast::<u8>(), total + 1);
    let mut at = 0;
    while at < total && buf[at] != 0 {
        ga_grow(gap, 1);
        *((*gap).ga_data as *mut *mut c_char).offset((*gap).ga_len as isize) = str.add(at);
        (*gap).ga_len += 1;
        if !escaped {
            return;
        }
        at += split_one_arg(&mut buf[at..]);
    }
}
pub unsafe extern "C" fn get_arglist_exp(
    mut str: *mut c_char,
    mut fcountp: *mut c_int,
    mut fnamesp: *mut *mut *mut c_char,
    mut wig: bool,
) -> c_int {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ptr::null_mut(),
    };
    let mut i: c_int = 0;
    get_arglist(&raw mut ga, str, true);
    if wig {
        i = expand_wildcards(
            ga.ga_len,
            ga.ga_data as *mut *mut c_char,
            fcountp,
            fnamesp,
            EW_FILE as c_int | EW_NOTFOUND as c_int | EW_NOTWILD as c_int,
        );
    } else {
        i = gen_expand_wildcards(
            ga.ga_len,
            ga.ga_data as *mut *mut c_char,
            fcountp,
            fnamesp,
            EW_FILE as c_int | EW_NOTFOUND as c_int | EW_NOTWILD as c_int,
        );
    }
    ga_clear(&raw mut ga);
    return i;
}
unsafe extern "C" fn alist_check_arg_idx() {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut win: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !win.is_null() {
            if (*win).w_alist == (*curwin.get()).w_alist {
                check_arg_idx(win);
            }
            win = (*win).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
unsafe extern "C" fn alist_add_list(
    mut count: c_int,
    mut files: *mut *mut c_char,
    mut after: c_int,
    mut will_edit: bool,
) {
    let mut old_argcount: c_int = argcount();
    if check_arglist_locked() != FAIL {
        let mut wp: *mut win_T = curwin.get();
        ga_grow(&raw mut (*(*wp).w_alist).al_ga, count);
        after = if (if after > 0 { after } else { 0 }) < argcount() {
            if after > 0 { after } else { 0 }
        } else {
            argcount()
        };
        if after < argcount() {
            memmove(
                arg(after + count) as *mut c_void,
                arg(after) as *const c_void,
                ((argcount() - after) as size_t).wrapping_mul(size_of::<aentry_T>()),
            );
        }
        arglist_locked.set(true);
        (*wp).w_locked = true;
        let mut i: c_int = 0;
        while i < count {
            let flags: c_int =
                BLN_LISTED as c_int | (if will_edit { BLN_CURBUF as c_int } else { 0 });
            (*arg(after + i)).ae_fname = *files.offset(i as isize);
            (*arg(after + i)).ae_fnum = buflist_add(*files.offset(i as isize), flags);
            i += 1;
        }
        arglist_locked.set(false);
        (*wp).w_locked = false;
        (*(*wp).w_alist).al_ga.ga_len += count;
        if old_argcount > 0 && (*wp).w_arg_idx >= after {
            (*wp).w_arg_idx += count;
        }
        return;
    }
}
unsafe extern "C" fn arglist_del_files(mut alist_ga: *mut garray_T) {
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ptr::null_mut(),
        startp: [ptr::null_mut(); 10],
        endp: [ptr::null_mut(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    regmatch.rm_ic = p_fic.get() != 0;
    let mut i: c_int = 0;
    while i < (*alist_ga).ga_len && !got_int.get() {
        let mut p: *mut c_char = *((*alist_ga).ga_data as *mut *mut c_char).offset(i as isize);
        p = file_pat_to_reg_pat(p, ptr::null(), ptr::null_mut(), false_0);
        if p.is_null() {
            break;
        }
        regmatch.regprog = vim_regcomp(p, if magic_isset() { RE_MAGIC } else { 0 });
        if regmatch.regprog.is_null() {
            xfree(p as *mut c_void);
            break;
        } else {
            let mut didone: bool = false;
            let mut match_0: c_int = 0;
            while match_0 < argcount() {
                if vim_regexec(&raw mut regmatch, arg_name(match_0), 0 as colnr_T) {
                    didone = true;
                    xfree((*arg(match_0)).ae_fname as *mut c_void);
                    memmove(
                        arg(match_0) as *mut c_void,
                        arg(match_0).offset(1) as *const c_void,
                        ((argcount() - match_0 - 1) as size_t).wrapping_mul(size_of::<aentry_T>()),
                    );
                    *argcount_mut() -= 1;
                    if (*curwin.get()).w_arg_idx > match_0 {
                        (*curwin.get()).w_arg_idx -= 1;
                    }
                    match_0 -= 1;
                }
                match_0 += 1;
            }
            vim_regfree(regmatch.regprog);
            xfree(p as *mut c_void);
            if !didone {
                semsg(
                    gettext(&raw const e_nomatch2 as *const c_char),
                    *((*alist_ga).ga_data as *mut *mut c_char).offset(i as isize),
                );
            }
            i += 1;
        }
    }
    ga_clear(alist_ga);
}
unsafe extern "C" fn do_arglist(
    mut str: *mut c_char,
    mut what: c_int,
    mut after: c_int,
    mut will_edit: bool,
) -> c_int {
    let mut new_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ptr::null_mut(),
    };
    let mut exp_count: c_int = 0;
    let mut exp_files: *mut *mut c_char = ptr::null_mut();
    let mut arg_escaped: bool = true;
    if check_arglist_locked() == FAIL {
        return FAIL;
    }
    if what == AL_ADD as c_int && *str as c_int == NUL {
        if (*curbuf.get()).b_ffname.is_null() {
            return FAIL;
        }
        str = (*curbuf.get()).b_fname;
        arg_escaped = false;
    }
    get_arglist(&raw mut new_ga, str, arg_escaped);
    if what == AL_DEL as c_int {
        arglist_del_files(&raw mut new_ga);
    } else {
        let mut i: c_int = expand_wildcards(
            new_ga.ga_len,
            new_ga.ga_data as *mut *mut c_char,
            &raw mut exp_count,
            &raw mut exp_files,
            EW_DIR as c_int | EW_FILE as c_int | EW_ADDSLASH as c_int | EW_NOTFOUND as c_int,
        );
        ga_clear(&raw mut new_ga);
        if i == FAIL || exp_count == 0 {
            emsg(gettext(&raw const e_nomatch as *const c_char));
            return FAIL;
        }
        if what == AL_ADD as c_int {
            alist_add_list(exp_count, exp_files, after, will_edit);
            xfree(exp_files as *mut c_void);
        } else {
            '_c2rust_label: {
                if what == AL_SET as c_int {
                } else {
                    __assert_fail(
                        c"what == AL_SET".as_ptr(),
                        c"src/nvim/arglist.rs".as_ptr(),
                        471,
                        c"int do_arglist(char *, int, int, _Bool)".as_ptr(),
                    );
                }
            };
            alist_set(
                (*curwin.get()).w_alist,
                exp_count,
                exp_files,
                will_edit as c_int,
                ptr::null_mut(),
                0,
            );
        }
    }
    alist_check_arg_idx();
    return OK;
}
pub unsafe extern "C" fn set_arglist(mut str: *mut c_char) {
    do_arglist(str, AL_SET as c_int, 0, true);
}
pub unsafe extern "C" fn editing_arg_idx(mut win: *mut win_T) -> bool {
    return !((*win).w_arg_idx >= wargcount(win)
        || (*(*win).w_buffer).handle != (*warg(win, (*win).w_arg_idx)).ae_fnum
            && ((*(*win).w_buffer).b_ffname.is_null()
                || path_full_compare(
                    alist_name(warg(win, (*win).w_arg_idx)),
                    (*(*win).w_buffer).b_ffname,
                    true,
                    true,
                ) as c_uint
                    & kEqualFiles as c_int as c_uint
                    == 0));
}
pub unsafe extern "C" fn check_arg_idx(mut win: *mut win_T) {
    if wargcount(win) > 1 && !editing_arg_idx(win) {
        (*win).w_arg_idx_invalid = true_0;
        if (*win).w_arg_idx != wargcount(win) - 1
            && arg_had_last.get() as c_int == false_0
            && (*win).w_alist == global_alist.ptr()
            && alist_count(global_alist.ptr()) > 0
            && (*win).w_arg_idx < alist_count(global_alist.ptr())
            && ((*(*win).w_buffer).handle
                == (*alist_arg(global_alist.ptr(), alist_count(global_alist.ptr()) - 1)).ae_fnum
                || !(*(*win).w_buffer).b_ffname.is_null()
                    && path_full_compare(
                        alist_name(alist_arg(
                            global_alist.ptr(),
                            alist_count(global_alist.ptr()) - 1,
                        )),
                        (*(*win).w_buffer).b_ffname,
                        true,
                        true,
                    ) as c_uint
                        & kEqualFiles as c_int as c_uint
                        != 0)
        {
            arg_had_last.set(true);
        }
    } else {
        (*win).w_arg_idx_invalid = false_0;
        if (*win).w_arg_idx == wargcount(win) - 1 && (*win).w_alist == global_alist.ptr() {
            arg_had_last.set(true);
        }
    };
}
pub unsafe extern "C" fn ex_args(mut eap: *mut exarg_T) {
    if (*eap).cmdidx as c_int != CMD_args as c_int {
        if check_arglist_locked() == FAIL {
            return;
        }
        alist_unlink((*curwin.get()).w_alist);
        if (*eap).cmdidx as c_int == CMD_argglobal as c_int {
            (*curwin.get()).w_alist = global_alist.ptr();
        } else {
            alist_new();
        }
    }
    if *(*eap).arg as c_int != NUL {
        if check_arglist_locked() == FAIL {
            return;
        }
        ex_next(eap);
        return;
    }
    if (*eap).cmdidx as c_int == CMD_args as c_int {
        if argcount() <= 0 {
            return;
        }
        let mut items: *mut *mut c_char =
            xmalloc(size_of::<*mut c_char>().wrapping_mul(argcount() as size_t))
                as *mut *mut c_char;
        gotocmdline(true);
        let mut i: c_int = 0;
        while i < argcount() {
            *items.offset(i as isize) = arg_name(i);
            i += 1;
        }
        list_in_columns(items, argcount(), (*curwin.get()).w_arg_idx);
        xfree(items as *mut c_void);
        return;
    }
    if (*eap).cmdidx as c_int == CMD_arglocal as c_int {
        let mut gap: *mut garray_T = &raw mut (*(*curwin.get()).w_alist).al_ga;
        ga_grow(gap, alist_count(global_alist.ptr()));
        let mut i_0: c_int = 0;
        while i_0 < alist_count(global_alist.ptr()) {
            if !(*alist_arg(global_alist.ptr(), i_0)).ae_fname.is_null() {
                (*arg((*gap).ga_len)).ae_fname =
                    xstrdup((*alist_arg(global_alist.ptr(), i_0)).ae_fname);
                (*arg((*gap).ga_len)).ae_fnum = (*alist_arg(global_alist.ptr(), i_0)).ae_fnum;
                (*gap).ga_len += 1;
            }
            i_0 += 1;
        }
    }
}
pub unsafe extern "C" fn ex_previous(mut eap: *mut exarg_T) {
    if (*curwin.get()).w_arg_idx - (*eap).line2 as c_int >= argcount() {
        do_argfile(eap, argcount() - 1);
    } else {
        do_argfile(eap, (*curwin.get()).w_arg_idx - (*eap).line2 as c_int);
    };
}
pub unsafe extern "C" fn ex_rewind(mut eap: *mut exarg_T) {
    do_argfile(eap, 0);
}
pub unsafe extern "C" fn ex_last(mut eap: *mut exarg_T) {
    do_argfile(eap, argcount() - 1);
}
pub unsafe extern "C" fn ex_argument(mut eap: *mut exarg_T) {
    let mut i: c_int = 0;
    if (*eap).addr_count > 0 {
        i = (*eap).line2 as c_int - 1;
    } else {
        i = (*curwin.get()).w_arg_idx;
    }
    do_argfile(eap, i);
}
pub unsafe extern "C" fn do_argfile(mut eap: *mut exarg_T, mut argn: c_int) {
    let mut is_split_cmd: bool = *(*eap).cmd as c_int == 's' as c_int;
    let mut old_arg_idx: c_int = (*curwin.get()).w_arg_idx;
    if argn < 0 || argn >= argcount() {
        if argcount() <= 1 {
            emsg(gettext(c"E163: There is only one file to edit".as_ptr()));
        } else if argn < 0 {
            emsg(gettext(c"E164: Cannot go before first file".as_ptr()));
        } else {
            emsg(gettext(c"E165: Cannot go beyond last file".as_ptr()));
        }
        return;
    }
    if !is_split_cmd
        && (*arg(argn)).ae_fnum != (*curbuf.get()).handle
        && !check_can_set_curbuf_forceit((*eap).forceit)
    {
        return;
    }
    setpcmark();
    if is_split_cmd || (*cmdmod.ptr()).cmod_tab != 0 {
        if win_split(0, 0) == FAIL {
            return;
        }
        (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
        (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
    } else {
        let mut other: c_int = true_0;
        if buf_hide(curbuf.get()) {
            let mut p: *mut c_char = fix_fname(arg_name(argn));
            other = otherfile(p) as c_int;
            xfree(p as *mut c_void);
        }
        if (!buf_hide(curbuf.get()) || other == 0)
            && check_changed(
                curbuf.get(),
                CCGD_AW as c_int
                    | (if other != 0 { 0 } else { CCGD_MULTWIN as c_int })
                    | (if (*eap).forceit != 0 {
                        CCGD_FORCEIT as c_int
                    } else {
                        0
                    })
                    | CCGD_EXCMD as c_int,
            ) as c_int
                != 0
        {
            return;
        }
    }
    (*curwin.get()).w_arg_idx = argn;
    if argn == argcount() - 1 && (*curwin.get()).w_alist == global_alist.ptr() {
        arg_had_last.set(true);
    }
    if do_ecmd(
        0,
        arg_name((*curwin.get()).w_arg_idx),
        ptr::null_mut(),
        eap,
        ECMD_LAST as c_int as linenr_T,
        (if buf_hide((*curwin.get()).w_buffer) {
            ECMD_HIDE as c_int
        } else {
            0
        }) + (if (*eap).forceit != 0 {
            ECMD_FORCEIT as c_int
        } else {
            0
        }),
        curwin.get(),
    ) == FAIL
    {
        (*curwin.get()).w_arg_idx = old_arg_idx;
    } else if (*eap).cmdidx as c_int != CMD_argdo as c_int {
        setmark('\'' as c_int);
    }
}
pub unsafe extern "C" fn ex_next(mut eap: *mut exarg_T) {
    if buf_hide(curbuf.get())
        || (*eap).cmdidx as c_int == CMD_snext as c_int
        || !check_changed(
            curbuf.get(),
            CCGD_AW as c_int
                | (if (*eap).forceit != 0 {
                    CCGD_FORCEIT as c_int
                } else {
                    0
                })
                | CCGD_EXCMD as c_int,
        )
    {
        let mut i: c_int = 0;
        if *(*eap).arg as c_int != NUL {
            if do_arglist((*eap).arg, AL_SET as c_int, 0, true) == FAIL {
                return;
            }
            i = 0;
        } else {
            i = (*curwin.get()).w_arg_idx + (*eap).line2 as c_int;
        }
        do_argfile(eap, i);
    }
}
pub unsafe extern "C" fn ex_argdedupe(mut _eap: *mut exarg_T) {
    let mut i: c_int = 0;
    while i < argcount() {
        let mut firstFullname: *mut c_char = FullName_save((*arg(i)).ae_fname, false);
        let mut j: c_int = i + 1;
        while j < argcount() {
            let mut secondFullname: *mut c_char = FullName_save((*arg(j)).ae_fname, false);
            let mut areNamesDuplicate: bool = path_fnamecmp(firstFullname, secondFullname) == 0;
            xfree(secondFullname as *mut c_void);
            if areNamesDuplicate {
                xfree((*arg(j)).ae_fname as *mut c_void);
                memmove(
                    arg(j) as *mut c_void,
                    arg(j).offset(1) as *const c_void,
                    ((argcount() - j - 1) as size_t).wrapping_mul(size_of::<aentry_T>()),
                );
                *argcount_mut() -= 1;
                if (*curwin.get()).w_arg_idx == j {
                    (*curwin.get()).w_arg_idx = i;
                } else if (*curwin.get()).w_arg_idx > j {
                    (*curwin.get()).w_arg_idx -= 1;
                }
                j -= 1;
            }
            j += 1;
        }
        xfree(firstFullname as *mut c_void);
        i += 1;
    }
}
pub unsafe extern "C" fn ex_argedit(mut eap: *mut exarg_T) {
    let mut i: c_int = if (*eap).addr_count != 0 {
        (*eap).line2 as c_int
    } else {
        (*curwin.get()).w_arg_idx + 1
    };
    let mut curbuf_is_reusable: bool = curbuf_reusable();
    if do_arglist((*eap).arg, AL_ADD as c_int, i, true) == FAIL {
        return;
    }
    maketitle();
    if (*curwin.get()).w_arg_idx == 0
        && (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0
        && ((*curbuf.get()).b_ffname.is_null() || curbuf_is_reusable)
    {
        i = 0;
    }
    if i < argcount() {
        do_argfile(eap, i);
    }
}
pub unsafe extern "C" fn ex_argadd(mut eap: *mut exarg_T) {
    do_arglist(
        (*eap).arg,
        AL_ADD as c_int,
        if (*eap).addr_count > 0 {
            (*eap).line2 as c_int
        } else {
            (*curwin.get()).w_arg_idx + 1
        },
        false,
    );
    maketitle();
}
pub unsafe extern "C" fn ex_argdelete(mut eap: *mut exarg_T) {
    if check_arglist_locked() == FAIL {
        return;
    }
    if (*eap).addr_count > 0 || *(*eap).arg as c_int == NUL {
        if (*eap).addr_count == 0 {
            if (*curwin.get()).w_arg_idx >= argcount() {
                emsg(gettext(c"E610: No argument to delete".as_ptr()));
                return;
            }
            (*eap).line2 = ((*curwin.get()).w_arg_idx + 1) as linenr_T;
            (*eap).line1 = (*eap).line2;
        } else if (*eap).line2 > argcount() as linenr_T {
            (*eap).line2 = argcount() as linenr_T;
        }
        let mut n: linenr_T = (*eap).line2 - (*eap).line1 + 1 as linenr_T;
        if *(*eap).arg as c_int != NUL {
            emsg(gettext(&raw const e_invarg as *const c_char));
        } else if n <= 0 as linenr_T {
            if (*eap).line1 != 1 as linenr_T || (*eap).line2 != 0 as linenr_T {
                emsg(gettext(&raw const e_invrange as *const c_char));
            }
        } else {
            let mut i: linenr_T = (*eap).line1;
            while i <= (*eap).line2 {
                xfree((*arg(i - 1 as linenr_T)).ae_fname as *mut c_void);
                i += 1;
            }
            memmove(
                arg((*eap).line1).offset(-1) as *mut c_void,
                arg((*eap).line2) as *const c_void,
                ((argcount() as linenr_T - (*eap).line2) as size_t)
                    .wrapping_mul(size_of::<aentry_T>()),
            );
            *argcount_mut() -= n as c_int;
            if (*curwin.get()).w_arg_idx as linenr_T >= (*eap).line2 {
                (*curwin.get()).w_arg_idx -= n as c_int;
            } else if (*curwin.get()).w_arg_idx as linenr_T > (*eap).line1 {
                (*curwin.get()).w_arg_idx = (*eap).line1 as c_int;
            }
            if argcount() == 0 {
                (*curwin.get()).w_arg_idx = 0;
            } else if (*curwin.get()).w_arg_idx >= argcount() {
                (*curwin.get()).w_arg_idx = argcount() - 1;
            }
        }
    } else {
        do_arglist((*eap).arg, AL_DEL as c_int, 0, false);
    }
    maketitle();
}
pub unsafe extern "C" fn get_arglist_name(mut _xp: *mut expand_T, mut idx: c_int) -> *mut c_char {
    if idx >= argcount() {
        return ptr::null_mut();
    }
    return arg_name(idx);
}
pub unsafe extern "C" fn alist_name(mut aep: *mut aentry_T) -> *mut c_char {
    let mut bp: *mut buf_T = buflist_findnr((*aep).ae_fnum);
    if bp.is_null() || (*bp).b_fname.is_null() {
        return (*aep).ae_fname;
    }
    return (*bp).b_fname;
}
pub const NUL: c_int = 0;

/// `ascii_isspace`: space, or one of the five whitespace control characters.
fn ascii_isspace(c: c_int) -> bool {
    (9..=13).contains(&c) || c == b' ' as c_int
}
pub const ML_EMPTY: c_int = 0x1;
pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
pub const true_0: c_int = 1;
pub const false_0: c_int = 0;
pub const RE_MAGIC: c_int = 1;
