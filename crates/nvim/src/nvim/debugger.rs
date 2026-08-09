use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::charset::{getdigits_int32, skipwhite};
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_all_later};
use crate::src::nvim::eval::typval::{kCallbackNone, tv_free};
use crate::src::nvim::eval::{eval_expr, typval_compare, typval_tostring};
use crate::src::nvim::ex_docmd::{do_cmdline, do_cmdline_cmd};
use crate::src::nvim::ex_getln::{getcmdline_prompt, getexline};
use crate::src::nvim::fileio::file_pat_to_reg_pat;
use crate::src::nvim::garray::{ga_clear, ga_grow};
use crate::src::nvim::getchar::{restore_typeahead, save_typeahead};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::keycodes::{K_SPECIAL, KE_SNR};
use crate::src::nvim::main::{
    NameBuff, RedrawingDisabled, Rows, State, cmd_silent, cmdline_row, curbuf, curwin,
    debug_backtrace_level, debug_break_level, debug_did_msg, debug_mode, debug_tick, did_emsg,
    e_invarg2, e_noname, emsg_off, emsg_silent, ex_nesting_level, ex_normal_busy, got_int,
    ignore_script, lines_left, msg_row, msg_scroll, msg_silent, need_wait_return, no_wait_return,
    redir_off,
};
use crate::src::nvim::memory::{xfree, xmalloc, xstrdup};
use crate::src::nvim::message::{emsg, msg, msg_starthere};
use crate::src::nvim::os::env::{expand_env_save, home_replace};
use crate::src::nvim::os::libc::{atoi, gettext, memmove, strcmp, strcpy, strlen, strncmp, strstr};
use crate::src::nvim::path::fix_fname;
use crate::src::nvim::regexp::{RE_MAGIC, RE_STRING, vim_regcomp, vim_regexec_prog, vim_regfree};
use crate::src::nvim::runtime::{estack_sfile, exestack};
use crate::src::nvim::state::MODE_NORMAL;
use crate::src::nvim::types::{
    CMD_breakdel, CMD_profdel, CMD_profile, Callback, Callback_data as C2Rust_Unnamed_5, String_0,
    buf_T, buffblock, buffblock_T, buffheader_T, colnr_T, estack_T, estack_arg_T, exarg_T,
    exprtype_T, garray_T, int32_t, int64_t, linenr_T, regprog_T, size_t, tasave_T, typebuf_T,
    typval_T, uint8_t, varnumber_T,
};
use crate::{semsg_c, smsg_c};
pub type C2Rust_Unnamed_13 = ::core::ffi::c_int;
pub const EXPAND_NOTHING: C2Rust_Unnamed_13 = 0;
pub const DOCMD_EXCRESET: C2Rust_Unnamed_17 = 16;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_17 = 1;
pub const ESTACK_NONE: estack_arg_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct debuggy {
    pub dbg_nr: ::core::ffi::c_int,
    pub dbg_type: ::core::ffi::c_int,
    pub dbg_name: *mut ::core::ffi::c_char,
    pub dbg_prog: *mut regprog_T,
    pub dbg_lnum: linenr_T,
    pub dbg_forceit: ::core::ffi::c_int,
    pub dbg_val: *mut typval_T,
    pub dbg_level: ::core::ffi::c_int,
}
pub const EXPR_IS: exprtype_T = 9;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static debug_greedy: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static debug_oldval: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static debug_newval: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub unsafe extern "C" fn do_debug(mut cmd: *mut ::core::ffi::c_char) {
    let mut save_msg_scroll: ::core::ffi::c_int = msg_scroll.get();
    let mut save_State: ::core::ffi::c_int = State.get();
    let mut save_did_emsg: ::core::ffi::c_int = did_emsg.get();
    let save_cmd_silent: bool = cmd_silent.get();
    let mut save_msg_silent: ::core::ffi::c_int = msg_silent.get();
    let mut save_emsg_silent: ::core::ffi::c_int = emsg_silent.get();
    let mut save_redir_off: bool = redir_off.get();
    let mut typeaheadbuf: tasave_T = tasave_T {
        save_typebuf: typebuf_T {
            tb_buf: ::core::ptr::null_mut::<uint8_t>(),
            tb_noremap: ::core::ptr::null_mut::<uint8_t>(),
            tb_buflen: 0,
            tb_off: 0,
            tb_len: 0,
            tb_maplen: 0,
            tb_silent: 0,
            tb_no_abbr_cnt: 0,
            tb_change_cnt: 0,
        },
        typebuf_valid: false,
        old_char: 0,
        old_mod_mask: 0,
        save_readbuf1: buffheader_T {
            bh_first: buffblock_T {
                b_next: ::core::ptr::null_mut::<buffblock>(),
                b_strlen: 0,
                b_str: [0; 1],
            },
            bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
            bh_index: 0,
            bh_space: 0,
            bh_create_newblock: false,
        },
        save_readbuf2: buffheader_T {
            bh_first: buffblock_T {
                b_next: ::core::ptr::null_mut::<buffblock>(),
                b_strlen: 0,
                b_str: [0; 1],
            },
            bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
            bh_index: 0,
            bh_space: 0,
            bh_create_newblock: false,
        },
        save_inputbuf: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut typeahead_saved: bool = false_0 != 0;
    let mut save_ignore_script: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cmdline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static last_cmd: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    (*RedrawingDisabled.ptr()) += 1;
    (*no_wait_return.ptr()) += 1;
    did_emsg.set(false_0);
    cmd_silent.set(false_0 != 0);
    msg_silent.set(false_0);
    emsg_silent.set(false_0);
    redir_off.set(true_0 != 0);
    State.set(MODE_NORMAL);
    debug_mode.set(true_0 != 0);
    if !debug_did_msg.get() {
        msg(
            gettext(
                b"Entering Debug mode.  Type \"cont\" to continue.\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            0 as ::core::ffi::c_int,
        );
    }
    if !(*debug_oldval.ptr()).is_null() {
        smsg_c!(
            0 as ::core::ffi::c_int,
            gettext(b"Oldval = \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
            debug_oldval.get(),
        );
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            debug_oldval.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
    }
    if !(*debug_newval.ptr()).is_null() {
        smsg_c!(
            0 as ::core::ffi::c_int,
            gettext(b"Newval = \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
            debug_newval.get(),
        );
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            debug_newval.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL;
        let _ = *ptr__0;
    }
    let mut sname: *mut ::core::ffi::c_char = estack_sfile(ESTACK_NONE);
    if !sname.is_null() {
        msg(sname, 0 as ::core::ffi::c_int);
    }
    xfree(sname as *mut ::core::ffi::c_void);
    if (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_lnum
        != 0 as linenr_T
    {
        smsg_c!(
            0 as ::core::ffi::c_int,
            gettext(b"line %ld: %s\0".as_ptr() as *const ::core::ffi::c_char),
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum as int64_t,
            cmd,
        );
    } else {
        smsg_c!(
            0 as ::core::ffi::c_int,
            gettext(b"cmd: %s\0".as_ptr() as *const ::core::ffi::c_char),
            cmd,
        );
    }
    loop {
        msg_scroll.set(true_0);
        need_wait_return.set(false_0 != 0);
        let mut save_ex_normal_busy: ::core::ffi::c_int = ex_normal_busy.get();
        ex_normal_busy.set(0 as ::core::ffi::c_int);
        if !debug_greedy.get() {
            save_typeahead(&raw mut typeaheadbuf);
            typeahead_saved = true_0 != 0;
            save_ignore_script = ignore_script.get() as ::core::ffi::c_int;
            ignore_script.set(true_0 != 0);
        }
        let mut n: ::core::ffi::c_int = debug_break_level.get();
        debug_break_level.set(-1 as ::core::ffi::c_int);
        xfree(cmdline as *mut ::core::ffi::c_void);
        cmdline = getcmdline_prompt(
            '>' as ::core::ffi::c_int,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            EXPAND_NOTHING as ::core::ffi::c_int,
            ::core::ptr::null::<::core::ffi::c_char>(),
            Callback {
                data: C2Rust_Unnamed_5 {
                    funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                type_0: kCallbackNone,
            },
            false_0 != 0,
            ::core::ptr::null_mut::<bool>(),
        );
        debug_break_level.set(n);
        if typeahead_saved {
            restore_typeahead(&raw mut typeaheadbuf);
            ignore_script.set(save_ignore_script != 0);
        }
        ex_normal_busy.set(save_ex_normal_busy);
        cmdline_row.set(msg_row.get());
        msg_starthere();
        if !cmdline.is_null() {
            p = skipwhite(cmdline);
            if *p as ::core::ffi::c_int != NUL {
                match *p as ::core::ffi::c_int {
                    99 => {
                        last_cmd.set(CMD_CONT);
                        tail = b"ont\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    110 => {
                        last_cmd.set(CMD_NEXT);
                        tail = b"ext\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    115 => {
                        last_cmd.set(CMD_STEP);
                        tail = b"tep\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    102 => {
                        last_cmd.set(0 as ::core::ffi::c_int);
                        if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'r' as ::core::ffi::c_int
                        {
                            last_cmd.set(CMD_FRAME);
                            tail = b"rame\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            last_cmd.set(CMD_FINISH);
                            tail = b"inish\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                    }
                    113 => {
                        last_cmd.set(CMD_QUIT);
                        tail = b"uit\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    105 => {
                        last_cmd.set(CMD_INTERRUPT);
                        tail = b"nterrupt\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    98 => {
                        last_cmd.set(CMD_BACKTRACE);
                        if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 't' as ::core::ffi::c_int
                        {
                            tail = b"t\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            tail = b"acktrace\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                    }
                    119 => {
                        last_cmd.set(CMD_BACKTRACE);
                        tail = b"here\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    117 => {
                        last_cmd.set(CMD_UP);
                        tail = b"p\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    100 => {
                        last_cmd.set(CMD_DOWN);
                        tail = b"own\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    _ => {
                        last_cmd.set(0 as ::core::ffi::c_int);
                    }
                }
                if last_cmd.get() != 0 as ::core::ffi::c_int {
                    p = p.offset(1);
                    while *p as ::core::ffi::c_int != NUL
                        && *p as ::core::ffi::c_int == *tail as ::core::ffi::c_int
                    {
                        p = p.offset(1);
                        tail = tail.offset(1);
                    }
                    if (*p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                        && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                        || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                            && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
                        && last_cmd.get() != CMD_FRAME
                    {
                        last_cmd.set(0 as ::core::ffi::c_int);
                    }
                }
            }
            if last_cmd.get() != 0 as ::core::ffi::c_int {
                match last_cmd.get() {
                    CMD_CONT => {
                        debug_break_level.set(-1 as ::core::ffi::c_int);
                    }
                    CMD_NEXT => {
                        debug_break_level.set(ex_nesting_level.get());
                    }
                    CMD_STEP => {
                        debug_break_level.set(9999 as ::core::ffi::c_int);
                    }
                    CMD_FINISH => {
                        debug_break_level.set(ex_nesting_level.get() - 1 as ::core::ffi::c_int);
                    }
                    CMD_QUIT => {
                        got_int.set(true_0 != 0);
                        debug_break_level.set(-1 as ::core::ffi::c_int);
                    }
                    CMD_INTERRUPT => {
                        got_int.set(true_0 != 0);
                        debug_break_level.set(9999 as ::core::ffi::c_int);
                        last_cmd.set(CMD_STEP);
                    }
                    CMD_BACKTRACE => {
                        do_showbacktrace(cmd);
                        continue;
                    }
                    CMD_FRAME => {
                        if *p as ::core::ffi::c_int == NUL {
                            do_showbacktrace(cmd);
                        } else {
                            p = skipwhite(p);
                            do_setdebugtracelevel(p);
                        }
                        continue;
                    }
                    CMD_UP => {
                        (*debug_backtrace_level.ptr()) += 1;
                        do_checkbacktracelevel();
                        continue;
                    }
                    CMD_DOWN => {
                        (*debug_backtrace_level.ptr()) -= 1;
                        do_checkbacktracelevel();
                        continue;
                    }
                    _ => {}
                }
                debug_backtrace_level.set(0 as ::core::ffi::c_int);
                break;
            } else {
                n = debug_break_level.get();
                debug_break_level.set(-1 as ::core::ffi::c_int);
                do_cmdline(
                    cmdline,
                    Some(
                        getexline
                            as unsafe extern "C" fn(
                                ::core::ffi::c_int,
                                *mut ::core::ffi::c_void,
                                ::core::ffi::c_int,
                                bool,
                            )
                                -> *mut ::core::ffi::c_char,
                    ),
                    NULL,
                    DOCMD_VERBOSE as ::core::ffi::c_int | DOCMD_EXCRESET as ::core::ffi::c_int,
                );
                debug_break_level.set(n);
            }
        }
        lines_left.set(Rows.get() - 1 as ::core::ffi::c_int);
    }
    xfree(cmdline as *mut ::core::ffi::c_void);
    (*RedrawingDisabled.ptr()) -= 1;
    (*no_wait_return.ptr()) -= 1;
    redraw_all_later(UPD_NOT_VALID);
    need_wait_return.set(false_0 != 0);
    msg_scroll.set(save_msg_scroll);
    lines_left.set(Rows.get() - 1 as ::core::ffi::c_int);
    State.set(save_State);
    debug_mode.set(false_0 != 0);
    did_emsg.set(save_did_emsg);
    cmd_silent.set(save_cmd_silent);
    msg_silent.set(save_msg_silent);
    emsg_silent.set(save_emsg_silent);
    redir_off.set(save_redir_off);
    debug_did_msg.set(true_0 != 0);
}
pub const CMD_CONT: ::core::ffi::c_int = 1;
pub const CMD_NEXT: ::core::ffi::c_int = 2;
pub const CMD_STEP: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const CMD_FINISH: ::core::ffi::c_int = 4;
pub const CMD_QUIT: ::core::ffi::c_int = 5;
pub const CMD_INTERRUPT: ::core::ffi::c_int = 6;
pub const CMD_BACKTRACE: ::core::ffi::c_int = 7;
pub const CMD_FRAME: ::core::ffi::c_int = 8;
pub const CMD_UP: ::core::ffi::c_int = 9;
pub const CMD_DOWN: ::core::ffi::c_int = 10;
unsafe extern "C" fn get_maxbacktrace_level(
    mut sname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut maxbacktrace: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if sname.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let mut p: *mut ::core::ffi::c_char = sname;
    let mut q: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    loop {
        q = strstr(p, b"..\0".as_ptr() as *const ::core::ffi::c_char);
        if q.is_null() {
            break;
        }
        p = q.offset(2 as ::core::ffi::c_int as isize);
        maxbacktrace += 1;
    }
    return maxbacktrace;
}
unsafe extern "C" fn do_setdebugtracelevel(mut arg: *mut ::core::ffi::c_char) {
    let mut level: ::core::ffi::c_int = atoi(arg);
    if *arg as ::core::ffi::c_int == '+' as ::core::ffi::c_int || level < 0 as ::core::ffi::c_int {
        (*debug_backtrace_level.ptr()) += level;
    } else {
        debug_backtrace_level.set(level);
    }
    do_checkbacktracelevel();
}
unsafe extern "C" fn do_checkbacktracelevel() {
    if debug_backtrace_level.get() < 0 as ::core::ffi::c_int {
        debug_backtrace_level.set(0 as ::core::ffi::c_int);
        msg(
            gettext(b"frame is zero\0".as_ptr() as *const ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
    } else {
        let mut sname: *mut ::core::ffi::c_char = estack_sfile(ESTACK_NONE);
        let mut max: ::core::ffi::c_int = get_maxbacktrace_level(sname);
        if debug_backtrace_level.get() > max {
            debug_backtrace_level.set(max);
            smsg_c!(
                0 as ::core::ffi::c_int,
                gettext(b"frame at highest level: %d\0".as_ptr() as *const ::core::ffi::c_char),
                max,
            );
        }
        xfree(sname as *mut ::core::ffi::c_void);
    };
}
unsafe extern "C" fn do_showbacktrace(mut cmd: *mut ::core::ffi::c_char) {
    let mut sname: *mut ::core::ffi::c_char = estack_sfile(ESTACK_NONE);
    let mut max: ::core::ffi::c_int = get_maxbacktrace_level(sname);
    if !sname.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut cur: *mut ::core::ffi::c_char = sname;
        while !got_int.get() {
            let mut next: *mut ::core::ffi::c_char =
                strstr(cur, b"..\0".as_ptr() as *const ::core::ffi::c_char);
            if !next.is_null() {
                *next = NUL as ::core::ffi::c_char;
            }
            if i == max - debug_backtrace_level.get() {
                smsg_c!(
                    0 as ::core::ffi::c_int,
                    b"->%d %s\0".as_ptr() as *const ::core::ffi::c_char,
                    max - i,
                    cur,
                );
            } else {
                smsg_c!(
                    0 as ::core::ffi::c_int,
                    b"  %d %s\0".as_ptr() as *const ::core::ffi::c_char,
                    max - i,
                    cur,
                );
            }
            i += 1;
            if next.is_null() {
                break;
            }
            *next = '.' as ::core::ffi::c_char;
            cur = next.offset(2 as ::core::ffi::c_int as isize);
        }
        xfree(sname as *mut ::core::ffi::c_void);
    }
    if (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_lnum
        != 0 as linenr_T
    {
        smsg_c!(
            0 as ::core::ffi::c_int,
            gettext(b"line %ld: %s\0".as_ptr() as *const ::core::ffi::c_char),
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum as int64_t,
            cmd,
        );
    } else {
        smsg_c!(
            0 as ::core::ffi::c_int,
            gettext(b"cmd: %s\0".as_ptr() as *const ::core::ffi::c_char),
            cmd,
        );
    };
}
pub unsafe fn ex_debug(mut eap: *mut exarg_T) {
    let mut debug_break_level_save: ::core::ffi::c_int = debug_break_level.get();
    debug_break_level.set(9999 as ::core::ffi::c_int);
    do_cmdline_cmd((*eap).arg);
    debug_break_level.set(debug_break_level_save);
}
static debug_breakpoint_name: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static debug_breakpoint_lnum: GlobalCell<linenr_T> = GlobalCell::new(0);
static debug_skipped: GlobalCell<bool> = GlobalCell::new(false);
static debug_skipped_name: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub unsafe extern "C" fn dbg_check_breakpoint(mut eap: *mut exarg_T) {
    debug_skipped.set(false_0 != 0);
    if !(*debug_breakpoint_name.ptr()).is_null() {
        if (*eap).skip == 0 {
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if *(*debug_breakpoint_name.ptr()).offset(0 as ::core::ffi::c_int as isize) as uint8_t
                as ::core::ffi::c_int
                == K_SPECIAL
                && *(*debug_breakpoint_name.ptr()).offset(1 as ::core::ffi::c_int as isize)
                    as uint8_t as ::core::ffi::c_int
                    == KS_EXTRA
                && *(*debug_breakpoint_name.ptr()).offset(2 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == KE_SNR as ::core::ffi::c_int
            {
                p = b"<SNR>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                p = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            smsg_c!(
                0 as ::core::ffi::c_int,
                gettext(b"Breakpoint in \"%s%s\" line %ld\0".as_ptr() as *const ::core::ffi::c_char),
                p,
                (*debug_breakpoint_name.ptr()).offset(
                    (if *p as ::core::ffi::c_int == NUL {
                        0 as ::core::ffi::c_int
                    } else {
                        3 as ::core::ffi::c_int
                    }) as isize,
                ),
                debug_breakpoint_lnum.get() as int64_t,
            );
            debug_breakpoint_name.set(::core::ptr::null_mut::<::core::ffi::c_char>());
            do_debug((*eap).cmd);
        } else {
            debug_skipped.set(true_0 != 0);
            debug_skipped_name.set(debug_breakpoint_name.get());
            debug_breakpoint_name.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        }
    } else if ex_nesting_level.get() <= debug_break_level.get() {
        if (*eap).skip == 0 {
            do_debug((*eap).cmd);
        } else {
            debug_skipped.set(true_0 != 0);
            debug_skipped_name.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        }
    }
}
pub unsafe extern "C" fn dbg_check_skipped(mut eap: *mut exarg_T) -> bool {
    if !debug_skipped.get() {
        return false_0 != 0;
    }
    let mut prev_got_int: bool = got_int.get();
    got_int.set(false_0 != 0);
    debug_breakpoint_name.set(debug_skipped_name.get());
    (*eap).skip = false_0;
    dbg_check_breakpoint(eap);
    (*eap).skip = true_0;
    got_int.set(got_int.get() as ::core::ffi::c_int | prev_got_int as ::core::ffi::c_int != 0);
    return true_0 != 0;
}
static dbg_breakp: GlobalCell<garray_T> = GlobalCell::new(garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: ::core::mem::size_of::<debuggy>() as ::core::ffi::c_int,
    ga_growsize: 4 as ::core::ffi::c_int,
    ga_data: NULL,
});
static last_breakp: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static has_expr_breakpoint: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static prof_ga: GlobalCell<garray_T> = GlobalCell::new(garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: ::core::mem::size_of::<debuggy>() as ::core::ffi::c_int,
    ga_growsize: 4 as ::core::ffi::c_int,
    ga_data: NULL,
});
pub const DBG_FUNC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DBG_FILE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DBG_EXPR: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
unsafe extern "C" fn eval_expr_no_emsg(bp: *mut debuggy) -> *mut typval_T {
    (*emsg_off.ptr()) += 1;
    let tv: *mut typval_T = eval_expr((*bp).dbg_name, ::core::ptr::null_mut::<exarg_T>());
    (*emsg_off.ptr()) -= 1;
    return tv;
}
unsafe extern "C" fn dbg_parsearg(
    mut arg: *mut ::core::ffi::c_char,
    mut gap: *mut garray_T,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = arg;
    let mut here: bool = false_0 != 0;
    ga_grow(gap, 1 as ::core::ffi::c_int);
    let mut bp: *mut debuggy = ((*gap).ga_data as *mut debuggy).offset((*gap).ga_len as isize);
    if strncmp(
        p,
        b"func\0".as_ptr() as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        (*bp).dbg_type = DBG_FUNC;
    } else if strncmp(
        p,
        b"file\0".as_ptr() as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        (*bp).dbg_type = DBG_FILE;
    } else if gap != prof_ga.ptr()
        && strncmp(
            p,
            b"here\0".as_ptr() as *const ::core::ffi::c_char,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        if (*curbuf.get()).b_ffname.is_null() {
            emsg(gettext(&raw const e_noname as *const ::core::ffi::c_char));
            return FAIL;
        }
        (*bp).dbg_type = DBG_FILE;
        here = true_0 != 0;
    } else if gap != prof_ga.ptr()
        && strncmp(
            p,
            b"expr\0".as_ptr() as *const ::core::ffi::c_char,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        (*bp).dbg_type = DBG_EXPR;
    } else {
        semsg_c!(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            p,
        );
        return FAIL;
    }
    p = skipwhite(p.offset(4 as ::core::ffi::c_int as isize));
    if here {
        (*bp).dbg_lnum = (*curwin.get()).w_cursor.lnum;
    } else if gap != prof_ga.ptr()
        && ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
    {
        (*bp).dbg_lnum = getdigits_int32(&raw mut p, true_0 != 0, 0 as int32_t) as linenr_T;
        p = skipwhite(p);
    } else {
        (*bp).dbg_lnum = 0 as ::core::ffi::c_int as linenr_T;
    }
    if !here && *p as ::core::ffi::c_int == NUL
        || here as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != NUL
        || (*bp).dbg_type == DBG_FUNC
            && !strstr(p, b"()\0".as_ptr() as *const ::core::ffi::c_char).is_null()
    {
        semsg_c!(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            arg,
        );
        return FAIL;
    }
    if (*bp).dbg_type == DBG_FUNC {
        (*bp).dbg_name = xstrdup(
            if strncmp(
                p,
                b"g:\0".as_ptr() as *const ::core::ffi::c_char,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p.offset(2 as ::core::ffi::c_int as isize)
            } else {
                p
            },
        );
    } else if here {
        (*bp).dbg_name = xstrdup((*curbuf.get()).b_ffname);
    } else if (*bp).dbg_type == DBG_EXPR {
        (*bp).dbg_name = xstrdup(p);
        (*bp).dbg_val = eval_expr_no_emsg(bp);
    } else {
        let mut q: *mut ::core::ffi::c_char = expand_env_save(p);
        if q.is_null() {
            return FAIL;
        }
        p = expand_env_save(q);
        xfree(q as *mut ::core::ffi::c_void);
        if p.is_null() {
            return FAIL;
        }
        if *p as ::core::ffi::c_int != '*' as ::core::ffi::c_int {
            (*bp).dbg_name = fix_fname(p);
            xfree(p as *mut ::core::ffi::c_void);
        } else {
            (*bp).dbg_name = p;
        }
    }
    if (*bp).dbg_name.is_null() {
        return FAIL;
    }
    return OK;
}
pub unsafe fn ex_breakadd(mut eap: *mut exarg_T) {
    let mut gap: *mut garray_T = dbg_breakp.ptr();
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_profile as ::core::ffi::c_int {
        gap = prof_ga.ptr();
    }
    if dbg_parsearg((*eap).arg, gap) != OK {
        return;
    }
    let mut bp: *mut debuggy = ((*gap).ga_data as *mut debuggy).offset((*gap).ga_len as isize);
    (*bp).dbg_forceit = (*eap).forceit;
    if (*bp).dbg_type != DBG_EXPR {
        let mut pat: *mut ::core::ffi::c_char = file_pat_to_reg_pat(
            (*bp).dbg_name,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0,
        );
        if !pat.is_null() {
            (*bp).dbg_prog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
            xfree(pat as *mut ::core::ffi::c_void);
        }
        if pat.is_null() || (*bp).dbg_prog.is_null() {
            xfree((*bp).dbg_name as *mut ::core::ffi::c_void);
        } else {
            if (*bp).dbg_lnum == 0 as linenr_T {
                (*bp).dbg_lnum = 1 as ::core::ffi::c_int as linenr_T;
            }
            if (*eap).cmdidx as ::core::ffi::c_int != CMD_profile as ::core::ffi::c_int {
                (*last_breakp.ptr()) += 1;
                (*((*gap).ga_data as *mut debuggy).offset((*gap).ga_len as isize)).dbg_nr =
                    last_breakp.get();
                (*debug_tick.ptr()) += 1;
            }
            (*gap).ga_len += 1;
        }
    } else {
        (*last_breakp.ptr()) += 1;
        let c2rust_fresh0 = (*gap).ga_len;
        (*gap).ga_len = (*gap).ga_len + 1;
        (*((*gap).ga_data as *mut debuggy).offset(c2rust_fresh0 as isize)).dbg_nr =
            last_breakp.get();
        (*debug_tick.ptr()) += 1;
        if gap == dbg_breakp.ptr() {
            has_expr_breakpoint.set(true_0 != 0);
        }
    };
}
pub unsafe fn ex_debuggreedy(mut eap: *mut exarg_T) {
    if (*eap).addr_count == 0 as ::core::ffi::c_int || (*eap).line2 != 0 as linenr_T {
        debug_greedy.set(true_0 != 0);
    } else {
        debug_greedy.set(false_0 != 0);
    };
}
unsafe extern "C" fn update_has_expr_breakpoint() {
    has_expr_breakpoint.set(false_0 != 0);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*dbg_breakp.ptr()).ga_len {
        if (*((*dbg_breakp.ptr()).ga_data as *mut debuggy).offset(i as isize)).dbg_type == DBG_EXPR
        {
            has_expr_breakpoint.set(true_0 != 0);
            break;
        } else {
            i += 1;
        }
    }
}
pub unsafe fn ex_breakdel(mut eap: *mut exarg_T) {
    let mut todel: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut del_all: bool = false_0 != 0;
    let mut best_lnum: linenr_T = 0 as linenr_T;
    let mut gap: *mut garray_T = dbg_breakp.ptr();
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_profdel as ::core::ffi::c_int {
        gap = prof_ga.ptr();
    }
    if ascii_isdigit(*(*eap).arg as ::core::ffi::c_int) {
        let mut nr: ::core::ffi::c_int = atoi((*eap).arg);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*gap).ga_len {
            if (*((*gap).ga_data as *mut debuggy).offset(i as isize)).dbg_nr == nr {
                todel = i;
                break;
            } else {
                i += 1;
            }
        }
    } else if *(*eap).arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
        todel = 0 as ::core::ffi::c_int;
        del_all = true_0 != 0;
    } else {
        if dbg_parsearg((*eap).arg, gap) == FAIL {
            return;
        }
        let mut bp: *mut debuggy = ((*gap).ga_data as *mut debuggy).offset((*gap).ga_len as isize);
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*gap).ga_len {
            let mut bpi: *mut debuggy = ((*gap).ga_data as *mut debuggy).offset(i_0 as isize);
            if (*bp).dbg_type == (*bpi).dbg_type
                && strcmp((*bp).dbg_name, (*bpi).dbg_name) == 0 as ::core::ffi::c_int
                && ((*bp).dbg_lnum == (*bpi).dbg_lnum
                    || (*bp).dbg_lnum == 0 as linenr_T
                        && (best_lnum == 0 as linenr_T || (*bpi).dbg_lnum < best_lnum))
            {
                todel = i_0;
                best_lnum = (*bpi).dbg_lnum;
            }
            i_0 += 1;
        }
        xfree((*bp).dbg_name as *mut ::core::ffi::c_void);
    }
    if todel < 0 as ::core::ffi::c_int {
        semsg_c!(
            gettext(b"E161: Breakpoint not found: %s\0".as_ptr() as *const ::core::ffi::c_char),
            (*eap).arg,
        );
        return;
    }
    while (*gap).ga_len > 0 as ::core::ffi::c_int {
        xfree(
            (*((*gap).ga_data as *mut debuggy).offset(todel as isize)).dbg_name
                as *mut ::core::ffi::c_void,
        );
        if (*((*gap).ga_data as *mut debuggy).offset(todel as isize)).dbg_type == DBG_EXPR
            && !(*((*gap).ga_data as *mut debuggy).offset(todel as isize))
                .dbg_val
                .is_null()
        {
            tv_free((*((*gap).ga_data as *mut debuggy).offset(todel as isize)).dbg_val);
        }
        vim_regfree((*((*gap).ga_data as *mut debuggy).offset(todel as isize)).dbg_prog);
        (*gap).ga_len -= 1;
        if todel < (*gap).ga_len {
            memmove(
                ((*gap).ga_data as *mut debuggy).offset(todel as isize) as *mut ::core::ffi::c_void,
                ((*gap).ga_data as *mut debuggy).offset((todel + 1 as ::core::ffi::c_int) as isize)
                    as *const ::core::ffi::c_void,
                (((*gap).ga_len - todel) as size_t).wrapping_mul(::core::mem::size_of::<debuggy>()),
            );
        }
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_breakdel as ::core::ffi::c_int {
            (*debug_tick.ptr()) += 1;
        }
        if !del_all {
            break;
        }
    }
    if (*gap).ga_len <= 0 as ::core::ffi::c_int {
        ga_clear(gap);
    }
    if gap == dbg_breakp.ptr() {
        update_has_expr_breakpoint();
    }
}
pub unsafe fn ex_breaklist(mut _eap: *mut exarg_T) {
    if (*dbg_breakp.ptr()).ga_len <= 0 as ::core::ffi::c_int {
        msg(
            gettext(b"No breakpoints defined\0".as_ptr() as *const ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*dbg_breakp.ptr()).ga_len {
        let mut bp: *mut debuggy = ((*dbg_breakp.ptr()).ga_data as *mut debuggy).offset(i as isize);
        if (*bp).dbg_type == DBG_FILE {
            home_replace(
                ::core::ptr::null::<buf_T>(),
                (*bp).dbg_name,
                NameBuff.ptr() as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                true_0 != 0,
            );
        }
        if (*bp).dbg_type != DBG_EXPR {
            smsg_c!(
                0 as ::core::ffi::c_int,
                gettext(b"%3d  %s %s  line %ld\0".as_ptr() as *const ::core::ffi::c_char),
                (*bp).dbg_nr,
                if (*bp).dbg_type == DBG_FUNC {
                    b"func\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"file\0".as_ptr() as *const ::core::ffi::c_char
                },
                if (*bp).dbg_type == DBG_FUNC {
                    (*bp).dbg_name
                } else {
                    NameBuff.ptr() as *mut ::core::ffi::c_char
                },
                (*bp).dbg_lnum as int64_t,
            );
        } else {
            smsg_c!(
                0 as ::core::ffi::c_int,
                gettext(b"%3d  expr %s\0".as_ptr() as *const ::core::ffi::c_char),
                (*bp).dbg_nr,
                (*bp).dbg_name,
            );
        }
        i += 1;
    }
}
pub unsafe extern "C" fn dbg_find_breakpoint(
    mut file: bool,
    mut fname: *mut ::core::ffi::c_char,
    mut after: linenr_T,
) -> linenr_T {
    return debuggy_find(
        file,
        fname,
        after,
        dbg_breakp.ptr(),
        ::core::ptr::null_mut::<bool>(),
    );
}
pub unsafe extern "C" fn has_profiling(
    mut file: bool,
    mut fname: *mut ::core::ffi::c_char,
    mut fp: *mut bool,
) -> bool {
    return debuggy_find(file, fname, 0 as linenr_T, prof_ga.ptr(), fp) != 0 as linenr_T;
}
unsafe extern "C" fn debuggy_find(
    mut file: bool,
    mut fname: *mut ::core::ffi::c_char,
    mut after: linenr_T,
    mut gap: *mut garray_T,
    mut fp: *mut bool,
) -> linenr_T {
    let mut bp: *mut debuggy = ::core::ptr::null_mut::<debuggy>();
    let mut lnum: linenr_T = 0 as linenr_T;
    let mut name: *mut ::core::ffi::c_char = fname;
    if (*gap).ga_len <= 0 as ::core::ffi::c_int {
        return 0 as linenr_T;
    }
    if !file
        && *fname.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            == K_SPECIAL
    {
        name = xmalloc(strlen(fname).wrapping_add(3 as size_t)) as *mut ::core::ffi::c_char;
        strcpy(
            name,
            b"<SNR>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        strcpy(
            name.offset(5 as ::core::ffi::c_int as isize),
            fname.offset(3 as ::core::ffi::c_int as isize),
        );
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        bp = ((*gap).ga_data as *mut debuggy).offset(i as isize);
        if ((*bp).dbg_type == DBG_FILE) as ::core::ffi::c_int == file as ::core::ffi::c_int
            && (*bp).dbg_type != DBG_EXPR
            && (gap == prof_ga.ptr()
                || (*bp).dbg_lnum > after && (lnum == 0 as linenr_T || (*bp).dbg_lnum < lnum))
        {
            let mut prev_got_int: bool = got_int.get();
            got_int.set(false_0 != 0);
            if vim_regexec_prog(&raw mut (*bp).dbg_prog, false_0 != 0, name, 0 as colnr_T) {
                lnum = (*bp).dbg_lnum;
                if !fp.is_null() {
                    *fp = (*bp).dbg_forceit != 0;
                }
            }
            got_int
                .set(got_int.get() as ::core::ffi::c_int | prev_got_int as ::core::ffi::c_int != 0);
        } else if (*bp).dbg_type == DBG_EXPR {
            let mut line: bool = false_0 != 0;
            let tv: *mut typval_T = eval_expr_no_emsg(bp);
            if !tv.is_null() {
                if (*bp).dbg_val.is_null() {
                    xfree(debug_oldval.get() as *mut ::core::ffi::c_void);
                    debug_oldval.set(typval_tostring(
                        ::core::ptr::null_mut::<typval_T>(),
                        true_0 != 0,
                    ));
                    (*bp).dbg_val = tv;
                    xfree(debug_newval.get() as *mut ::core::ffi::c_void);
                    debug_newval.set(typval_tostring((*bp).dbg_val, true_0 != 0));
                    line = true_0 != 0;
                } else {
                    if typval_compare(tv, (*bp).dbg_val, EXPR_IS, false_0 != 0) == OK
                        && (*tv).vval.v_number == false_0 as varnumber_T
                    {
                        line = true_0 != 0;
                        xfree(debug_oldval.get() as *mut ::core::ffi::c_void);
                        debug_oldval.set(typval_tostring((*bp).dbg_val, true_0 != 0));
                        let v: *mut typval_T = eval_expr_no_emsg(bp);
                        xfree(debug_newval.get() as *mut ::core::ffi::c_void);
                        debug_newval.set(typval_tostring(v, true_0 != 0));
                        tv_free((*bp).dbg_val);
                        (*bp).dbg_val = v;
                    }
                    tv_free(tv);
                }
            } else if !(*bp).dbg_val.is_null() {
                xfree(debug_oldval.get() as *mut ::core::ffi::c_void);
                debug_oldval.set(typval_tostring((*bp).dbg_val, true_0 != 0));
                xfree(debug_newval.get() as *mut ::core::ffi::c_void);
                debug_newval.set(typval_tostring(
                    ::core::ptr::null_mut::<typval_T>(),
                    true_0 != 0,
                ));
                tv_free((*bp).dbg_val);
                (*bp).dbg_val = ::core::ptr::null_mut::<typval_T>();
                line = true_0 != 0;
            }
            if line {
                lnum = if after > 0 as linenr_T {
                    after
                } else {
                    1 as linenr_T
                };
                break;
            }
        }
        i += 1;
    }
    if name != fname {
        xfree(name as *mut ::core::ffi::c_void);
    }
    return lnum;
}
pub unsafe extern "C" fn dbg_breakpoint(mut name: *mut ::core::ffi::c_char, mut lnum: linenr_T) {
    debug_breakpoint_name.set(name);
    debug_breakpoint_lnum.set(lnum);
}
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
