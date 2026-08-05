use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::charset::skipwhite;
use crate::src::nvim::debugger::dbg_check_skipped;
use crate::src::nvim::eval::typval::tv_list_ref;
use crate::src::nvim::eval::typval::{tv_clear, tv_free, tv_list_unref};
use crate::src::nvim::eval::userfunc::{do_return, get_return_cmd};
use crate::src::nvim::eval::vars::{set_vim_var_list, set_vim_var_string};
use crate::src::nvim::eval::{
    clear_evalarg, eval_for_line, eval_to_bool, eval_to_string_skip, eval0, fill_evalarg_from_eap,
    free_for_info, next_for_item,
};
use crate::src::nvim::ex_docmd::{ends_excmd, find_nextcmd, handle_did_throw, modifier_len};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    IObuff, caught_stack, cmdline_row, current_exception, debug_break_level, did_emsg, did_endif,
    did_throw, e_argreq, e_endfor, e_endif, e_endtry, e_endwhile, e_for, e_interr, e_invarg2,
    e_invexpr2, e_outofmem, e_str_not_inside_function, e_trailing_arg, e_while,
    empty_string_option, emsg_off, emsg_silent, force_abort, got_int, msg_list, msg_row,
    msg_scroll, msg_silent, need_rethrow, no_wait_return, p_cpo, p_verbose, suppress_errthrow,
    trylevel,
};
use crate::src::nvim::memory::{xfree, xmalloc, xrealloc, xstrdup, xstrlcpy};
use crate::src::nvim::message::{
    emsg, internal_error, msg_puts, semsg, smsg, verbose_enter, verbose_leave,
};
use crate::src::nvim::option::p_vfile;
use crate::src::nvim::os::libc::{
    __assert_fail, gettext, snprintf, strcat, strcpy, strlen, strncmp,
};
use crate::src::nvim::regexp::{RE_MAGIC, RE_STRING, skip_regexp_err};
use crate::src::nvim::runtime::{do_finish, estack_sfile, exestack, stacktrace_create};
use crate::src::nvim::strings::{concat_str, vim_snprintf, vim_snprintf_safelen, xstrnsave};
use crate::src::nvim::types::{
    BoolVarValue, CMD_index, OptInt, SpecialVarValue, VAR_UNKNOWN, VAR_UNLOCKED, VV_EXCEPTION,
    VV_STACKTRACE, VV_THROWPOINT, cleanup_T, colnr_T, cstack_T, eslist_T, estack_T, estack_arg_T,
    evalarg_T, exarg_T, except_T, except_type_T, exception_state_T, int64_t, linenr_T, list_T,
    msglist_T, ptrdiff_t, regmatch_T, regprog_T, size_t, typval_T, typval_vval_union,
};
unsafe extern "C" {
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec_nl(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T)
    -> bool;
}
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub type C2Rust_Unnamed_1 = ::core::ffi::c_uint;
pub const CSTACK_LEN: C2Rust_Unnamed_1 = 50;
pub type C2Rust_Unnamed_3 = ::core::ffi::c_uint;
pub const CSF_SILENT: C2Rust_Unnamed_3 = 16384;
pub const CSF_FINISHED: C2Rust_Unnamed_3 = 8192;
pub const CSF_CAUGHT: C2Rust_Unnamed_3 = 4096;
pub const CSF_THROWN: C2Rust_Unnamed_3 = 2048;
pub const CSF_FINALLY: C2Rust_Unnamed_3 = 512;
pub const CSF_TRY: C2Rust_Unnamed_3 = 256;
pub const CSF_FOR: C2Rust_Unnamed_3 = 16;
pub const CSF_WHILE: C2Rust_Unnamed_3 = 8;
pub const CSF_ELSE: C2Rust_Unnamed_3 = 4;
pub const CSF_ACTIVE: C2Rust_Unnamed_3 = 2;
pub const CSF_TRUE: C2Rust_Unnamed_3 = 1;
pub type C2Rust_Unnamed_4 = ::core::ffi::c_uint;
pub const CSTP_RETURN: C2Rust_Unnamed_4 = 24;
pub const CSTP_CONTINUE: C2Rust_Unnamed_4 = 16;
pub const CSTP_BREAK: C2Rust_Unnamed_4 = 8;
pub const CSTP_THROW: C2Rust_Unnamed_4 = 4;
pub const CSTP_INTERRUPT: C2Rust_Unnamed_4 = 2;
pub const CSTP_ERROR: C2Rust_Unnamed_4 = 1;
pub const CSTP_NONE: C2Rust_Unnamed_4 = 0;
pub type C2Rust_Unnamed_5 = ::core::ffi::c_uint;
pub const CSL_HAD_FINA: C2Rust_Unnamed_5 = 8;
pub const CSL_HAD_CONT: C2Rust_Unnamed_5 = 4;
pub const CSL_HAD_ENDLOOP: C2Rust_Unnamed_5 = 2;
pub const CSL_HAD_LOOP: C2Rust_Unnamed_5 = 1;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
pub const CMD_while: CMD_index = 525;
pub const CMD_snext: CMD_index = 414;
pub const CMD_endwhile: CMD_index = 147;
pub const CMD_elseif: CMD_index = 141;
pub const CMD_else: CMD_index = 140;
pub const CMD_drop: CMD_index = 130;
pub const CMD_arglocal: CMD_index = 14;
pub const CMD_argglobal: CMD_index = 13;
pub const CMD_argdo: CMD_index = 10;
pub const CMD_args: CMD_index = 7;
pub const ESTACK_NONE: estack_arg_T = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static e_multiple_else: GlobalCell<[::core::ffi::c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"E583: Multiple :else\0")
});
static e_multiple_finally: GlobalCell<[::core::ffi::c_char; 24]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E607: Multiple :finally\0")
});
pub const THROW_ON_ERROR: ::core::ffi::c_int = true_0;
unsafe extern "C" fn discard_pending_return(mut p: *mut typval_T) {
    tv_free(p);
}
static cause_abort: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub unsafe extern "C" fn aborting() -> bool {
    return did_emsg.get() != 0 && force_abort.get() as ::core::ffi::c_int != 0
        || got_int.get() as ::core::ffi::c_int != 0
        || did_throw.get() as ::core::ffi::c_int != 0;
}
pub unsafe extern "C" fn update_force_abort() {
    if cause_abort.get() {
        force_abort.set(true_0 != 0);
    }
}
pub unsafe extern "C" fn should_abort(mut retcode: ::core::ffi::c_int) -> bool {
    return retcode == FAIL && trylevel.get() != 0 as ::core::ffi::c_int && emsg_silent.get() == 0
        || aborting() as ::core::ffi::c_int != 0;
}
pub unsafe extern "C" fn aborted_in_try() -> bool {
    return force_abort.get();
}
pub unsafe extern "C" fn cause_errthrow(
    mut mesg: *const ::core::ffi::c_char,
    mut multiline: bool,
    mut concat: bool,
    mut severe: bool,
    mut ignore: *mut bool,
) -> bool {
    let mut elem: *mut msglist_T = ::core::ptr::null_mut::<msglist_T>();
    if suppress_errthrow.get() {
        return false_0 != 0;
    }
    if did_emsg.get() == 0 {
        cause_abort.set(force_abort.get());
        force_abort.set(false_0 != 0);
    }
    if (trylevel.get() == 0 as ::core::ffi::c_int && !cause_abort.get() || emsg_silent.get() != 0)
        && !did_throw.get()
    {
        return false_0 != 0;
    }
    if mesg
        == gettext(&raw const e_interr as *const ::core::ffi::c_char) as *const ::core::ffi::c_char
    {
        *ignore = true_0 != 0;
        return true_0 != 0;
    }
    cause_abort.set(true_0 != 0);
    if did_throw.get() {
        if (*current_exception.get()).type_0 as ::core::ffi::c_uint
            == ET_INTERRUPT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            got_int.set(false_0 != 0);
        }
        discard_current_exception();
    }
    if !(*msg_list.ptr()).is_null() {
        let mut plist: *mut *mut msglist_T = msg_list.get();
        while !(*plist).is_null() {
            if (**plist).next.is_null() && concat as ::core::ffi::c_int != 0 {
                (**plist).msg = xrealloc(
                    (**plist).msg as *mut ::core::ffi::c_void,
                    strlen((**plist).msg)
                        .wrapping_add(strlen(mesg))
                        .wrapping_add(1 as size_t),
                ) as *mut ::core::ffi::c_char;
                (**plist).throw_msg = strcat((**plist).msg, mesg);
                return true_0 != 0;
            }
            plist = &raw mut (**plist).next;
        }
        elem = xmalloc(::core::mem::size_of::<msglist_T>()) as *mut msglist_T;
        (*elem).msg = xstrdup(mesg);
        (*elem).multiline = multiline;
        (*elem).next = ::core::ptr::null_mut::<msglist_T>();
        (*elem).throw_msg = ::core::ptr::null_mut::<::core::ffi::c_char>();
        *plist = elem;
        if plist == msg_list.get() || severe as ::core::ffi::c_int != 0 {
            let mut tmsg: *mut ::core::ffi::c_char = (*elem).msg;
            if strncmp(
                tmsg,
                b"Vim E\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
                && ascii_isdigit(
                    *tmsg.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
                && ascii_isdigit(
                    *tmsg.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
                && ascii_isdigit(
                    *tmsg.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
                && *tmsg.offset(8 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ':' as ::core::ffi::c_int
                && *tmsg.offset(9 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ' ' as ::core::ffi::c_int
            {
                (**msg_list.get()).throw_msg = tmsg.offset(4 as ::core::ffi::c_int as isize);
            } else {
                (**msg_list.get()).throw_msg = tmsg;
            }
        }
        (*elem).sfile = estack_sfile(ESTACK_NONE);
        (*elem).slnum = (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum;
    }
    return true_0 != 0;
}
unsafe extern "C" fn free_msglist(mut l: *mut msglist_T) {
    let mut messages: *mut msglist_T = l;
    while !messages.is_null() {
        let mut next: *mut msglist_T = (*messages).next;
        xfree((*messages).msg as *mut ::core::ffi::c_void);
        xfree((*messages).sfile as *mut ::core::ffi::c_void);
        xfree(messages as *mut ::core::ffi::c_void);
        messages = next;
    }
}
pub unsafe extern "C" fn free_global_msglist() {
    free_msglist(*msg_list.get());
    *msg_list.get() = ::core::ptr::null_mut::<msglist_T>();
}
pub unsafe extern "C" fn do_errthrow(
    mut cstack: *mut cstack_T,
    mut cmdname: *mut ::core::ffi::c_char,
) {
    if cause_abort.get() {
        cause_abort.set(false_0 != 0);
        force_abort.set(true_0 != 0);
    }
    if (*msg_list.ptr()).is_null() || (*msg_list.get()).is_null() {
        return;
    }
    if throw_exception(
        *msg_list.get() as *mut ::core::ffi::c_void,
        ET_ERROR,
        cmdname,
    ) == FAIL
    {
        free_msglist(*msg_list.get());
    } else if !cstack.is_null() {
        do_throw(cstack);
    } else {
        need_rethrow.set(true_0 != 0);
    }
    *msg_list.get() = ::core::ptr::null_mut::<msglist_T>();
}
pub unsafe extern "C" fn do_intthrow(mut cstack: *mut cstack_T) -> bool {
    if !got_int.get() || trylevel.get() == 0 as ::core::ffi::c_int && !did_throw.get() {
        return false_0 != 0;
    }
    if did_throw.get() {
        if (*current_exception.get()).type_0 as ::core::ffi::c_uint
            == ET_INTERRUPT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return false_0 != 0;
        }
        discard_current_exception();
    }
    if throw_exception(
        b"Vim:Interrupt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_void,
        ET_INTERRUPT,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    ) != FAIL
    {
        do_throw(cstack);
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn get_exception_string(
    mut value: *mut ::core::ffi::c_void,
    mut type_0: except_type_T,
    mut cmdname: *mut ::core::ffi::c_char,
    mut should_free: *mut bool,
) -> *mut ::core::ffi::c_char {
    let mut ret: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if type_0 as ::core::ffi::c_uint == ET_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint {
        let mut val: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        *should_free = true_0 != 0;
        let mut mesg: *mut ::core::ffi::c_char = (*(value as *mut msglist_T)).throw_msg;
        if !cmdname.is_null() && *cmdname as ::core::ffi::c_int != NUL {
            let mut cmdlen: size_t = strlen(cmdname);
            ret = xstrnsave(
                b"Vim(\0".as_ptr() as *const ::core::ffi::c_char,
                (4 as size_t)
                    .wrapping_add(cmdlen)
                    .wrapping_add(2 as size_t)
                    .wrapping_add(strlen(mesg)),
            );
            strcpy(ret.offset(4 as ::core::ffi::c_int as isize), cmdname);
            strcpy(
                ret.offset((4 as size_t).wrapping_add(cmdlen) as isize),
                b"):\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            val = ret
                .offset(4 as ::core::ffi::c_int as isize)
                .offset(cmdlen as isize)
                .offset(2 as ::core::ffi::c_int as isize);
        } else {
            ret = xstrnsave(
                b"Vim:\0".as_ptr() as *const ::core::ffi::c_char,
                (4 as size_t).wrapping_add(strlen(mesg)),
            );
            val = ret.offset(4 as ::core::ffi::c_int as isize);
        }
        let mut p: *mut ::core::ffi::c_char = mesg;
        loop {
            if *p as ::core::ffi::c_int == NUL
                || *p as ::core::ffi::c_int == 'E' as ::core::ffi::c_int
                    && ascii_isdigit(
                        *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                    && (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int
                        || ascii_isdigit(
                            *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0
                            && (*p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == ':' as ::core::ffi::c_int
                                || ascii_isdigit(*p.offset(3 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0
                                    && *p.offset(4 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == ':' as ::core::ffi::c_int))
            {
                if *p as ::core::ffi::c_int == NUL || p == mesg {
                    strcat(val, mesg);
                    break;
                } else if !(*mesg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '"' as ::core::ffi::c_int
                    || p.offset(-(2 as ::core::ffi::c_int as isize))
                        < mesg.offset(1 as ::core::ffi::c_int as isize)
                    || *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != '"' as ::core::ffi::c_int
                    || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != ' ' as ::core::ffi::c_int)
                {
                    strcat(val, p);
                    *p.offset(-2 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
                    snprintf(
                        val.offset(strlen(p) as isize),
                        strlen(b" (%s)\0".as_ptr() as *const ::core::ffi::c_char),
                        b" (%s)\0".as_ptr() as *const ::core::ffi::c_char,
                        mesg.offset(1 as ::core::ffi::c_int as isize),
                    );
                    *p.offset(-2 as ::core::ffi::c_int as isize) = '"' as ::core::ffi::c_char;
                    break;
                }
            }
            p = p.offset(1);
        }
    } else {
        *should_free = false_0 != 0;
        ret = value as *mut ::core::ffi::c_char;
    }
    return ret;
}
unsafe extern "C" fn throw_exception(
    mut value: *mut ::core::ffi::c_void,
    mut type_0: except_type_T,
    mut cmdname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut excp: *mut except_T = ::core::ptr::null_mut::<except_T>();
    let mut should_free: bool = false;
    '_fail: {
        if type_0 as ::core::ffi::c_uint == ET_USER as ::core::ffi::c_int as ::core::ffi::c_uint {
            if strncmp(
                value as *const ::core::ffi::c_char,
                b"Vim\0".as_ptr() as *const ::core::ffi::c_char,
                3 as size_t,
            ) == 0 as ::core::ffi::c_int
                && (*(value as *mut ::core::ffi::c_char).offset(3 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == NUL
                    || *(value as *mut ::core::ffi::c_char).offset(3 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int
                    || *(value as *mut ::core::ffi::c_char).offset(3 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == '(' as ::core::ffi::c_int)
            {
                emsg(gettext(
                    b"E608: Cannot :throw exceptions with 'Vim' prefix\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                break '_fail;
            }
        }
        excp = xmalloc(::core::mem::size_of::<except_T>()) as *mut except_T;
        if type_0 as ::core::ffi::c_uint == ET_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint {
            (*excp).messages = value as *mut msglist_T;
        }
        should_free = false;
        (*excp).value = get_exception_string(value, type_0, cmdname, &raw mut should_free);
        if (*excp).value.is_null() && should_free as ::core::ffi::c_int != 0 {
            xfree(excp as *mut ::core::ffi::c_void);
            suppress_errthrow.set(true_0 != 0);
            emsg(gettext(&raw const e_outofmem as *const ::core::ffi::c_char));
        } else {
            (*excp).type_0 = type_0;
            if type_0 as ::core::ffi::c_uint
                == ET_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
                && !(*(value as *mut msglist_T)).sfile.is_null()
            {
                let mut entry: *mut msglist_T = value as *mut msglist_T;
                (*excp).throw_name = (*entry).sfile;
                (*entry).sfile = ::core::ptr::null_mut::<::core::ffi::c_char>();
                (*excp).throw_lnum = (*entry).slnum;
            } else {
                (*excp).throw_name = estack_sfile(ESTACK_NONE);
                if (*excp).throw_name.is_null() {
                    (*excp).throw_name = xstrdup(b"\0".as_ptr() as *const ::core::ffi::c_char);
                }
                (*excp).throw_lnum = (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum;
            }
            (*excp).stacktrace = stacktrace_create();
            tv_list_ref((*excp).stacktrace);
            if p_verbose.get() >= 13 as OptInt || debug_break_level.get() > 0 as ::core::ffi::c_int
            {
                let mut save_msg_silent: ::core::ffi::c_int = msg_silent.get();
                if debug_break_level.get() > 0 as ::core::ffi::c_int {
                    msg_silent.set(false_0);
                } else {
                    verbose_enter();
                }
                (*no_wait_return.ptr()) += 1;
                if debug_break_level.get() > 0 as ::core::ffi::c_int
                    || *p_vfile.get() as ::core::ffi::c_int == NUL
                {
                    msg_scroll.set(true_0);
                }
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"Exception thrown: %s\0".as_ptr() as *const ::core::ffi::c_char),
                    (*excp).value,
                );
                msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
                if debug_break_level.get() > 0 as ::core::ffi::c_int
                    || *p_vfile.get() as ::core::ffi::c_int == NUL
                {
                    cmdline_row.set(msg_row.get());
                }
                (*no_wait_return.ptr()) -= 1;
                if debug_break_level.get() > 0 as ::core::ffi::c_int {
                    msg_silent.set(save_msg_silent);
                } else {
                    verbose_leave();
                }
            }
            current_exception.set(excp);
            return OK;
        }
    }
    current_exception.set(::core::ptr::null_mut::<except_T>());
    return FAIL;
}
unsafe extern "C" fn discard_exception(mut excp: *mut except_T, mut was_finished: bool) {
    if current_exception.get() == excp {
        current_exception.set(::core::ptr::null_mut::<except_T>());
    }
    if excp.is_null() {
        internal_error(b"discard_exception()\0".as_ptr() as *const ::core::ffi::c_char);
        return;
    }
    if p_verbose.get() >= 13 as OptInt || debug_break_level.get() > 0 as ::core::ffi::c_int {
        let mut save_msg_silent: ::core::ffi::c_int = msg_silent.get();
        let mut saved_IObuff: *mut ::core::ffi::c_char =
            xstrdup(IObuff.ptr() as *mut ::core::ffi::c_char);
        if debug_break_level.get() > 0 as ::core::ffi::c_int {
            msg_silent.set(false_0);
        } else {
            verbose_enter();
        }
        (*no_wait_return.ptr()) += 1;
        if debug_break_level.get() > 0 as ::core::ffi::c_int
            || *p_vfile.get() as ::core::ffi::c_int == NUL
        {
            msg_scroll.set(true_0);
        }
        smsg(
            0 as ::core::ffi::c_int,
            if was_finished as ::core::ffi::c_int != 0 {
                gettext(b"Exception finished: %s\0".as_ptr() as *const ::core::ffi::c_char)
            } else {
                gettext(b"Exception discarded: %s\0".as_ptr() as *const ::core::ffi::c_char)
            },
            (*excp).value,
        );
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        if debug_break_level.get() > 0 as ::core::ffi::c_int
            || *p_vfile.get() as ::core::ffi::c_int == NUL
        {
            cmdline_row.set(msg_row.get());
        }
        (*no_wait_return.ptr()) -= 1;
        if debug_break_level.get() > 0 as ::core::ffi::c_int {
            msg_silent.set(save_msg_silent);
        } else {
            verbose_leave();
        }
        xstrlcpy(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            saved_IObuff,
            IOSIZE as size_t,
        );
        xfree(saved_IObuff as *mut ::core::ffi::c_void);
    }
    if (*excp).type_0 as ::core::ffi::c_uint
        != ET_INTERRUPT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        xfree((*excp).value as *mut ::core::ffi::c_void);
    }
    if (*excp).type_0 as ::core::ffi::c_uint
        == ET_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        free_msglist((*excp).messages);
    }
    xfree((*excp).throw_name as *mut ::core::ffi::c_void);
    tv_list_unref((*excp).stacktrace);
    xfree(excp as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn discard_current_exception() {
    if !(*current_exception.ptr()).is_null() {
        discard_exception(current_exception.get(), false_0 != 0);
    }
    did_throw.set(false_0 != 0);
    need_rethrow.set(false_0 != 0);
}
unsafe extern "C" fn catch_exception(mut excp: *mut except_T) {
    (*excp).caught = caught_stack.get();
    caught_stack.set(excp);
    set_vim_var_string(VV_EXCEPTION, (*excp).value, -1 as ptrdiff_t);
    set_vim_var_list(VV_STACKTRACE, (*excp).stacktrace);
    if *(*excp).throw_name as ::core::ffi::c_int != NUL {
        let mut IObufflen: size_t = 0;
        if (*excp).throw_lnum != 0 as linenr_T {
            IObufflen = vim_snprintf_safelen(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                gettext(b"%s, line %ld\0".as_ptr() as *const ::core::ffi::c_char),
                (*excp).throw_name,
                (*excp).throw_lnum as int64_t,
            );
        } else {
            IObufflen = vim_snprintf_safelen(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                (*excp).throw_name,
            );
        }
        set_vim_var_string(
            VV_THROWPOINT,
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IObufflen as ptrdiff_t,
        );
    } else {
        set_vim_var_string(
            VV_THROWPOINT,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
    }
    if p_verbose.get() >= 13 as OptInt || debug_break_level.get() > 0 as ::core::ffi::c_int {
        let mut save_msg_silent: ::core::ffi::c_int = msg_silent.get();
        if debug_break_level.get() > 0 as ::core::ffi::c_int {
            msg_silent.set(false_0);
        } else {
            verbose_enter();
        }
        (*no_wait_return.ptr()) += 1;
        if debug_break_level.get() > 0 as ::core::ffi::c_int
            || *p_vfile.get() as ::core::ffi::c_int == NUL
        {
            msg_scroll.set(true_0);
        }
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"Exception caught: %s\0".as_ptr() as *const ::core::ffi::c_char),
            (*excp).value,
        );
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        if debug_break_level.get() > 0 as ::core::ffi::c_int
            || *p_vfile.get() as ::core::ffi::c_int == NUL
        {
            cmdline_row.set(msg_row.get());
        }
        (*no_wait_return.ptr()) -= 1;
        if debug_break_level.get() > 0 as ::core::ffi::c_int {
            msg_silent.set(save_msg_silent);
        } else {
            verbose_leave();
        }
    }
}
unsafe extern "C" fn finish_exception(mut excp: *mut except_T) {
    if excp != caught_stack.get() {
        internal_error(b"finish_exception()\0".as_ptr() as *const ::core::ffi::c_char);
    }
    caught_stack.set((*caught_stack.get()).caught);
    if !(*caught_stack.ptr()).is_null() {
        set_vim_var_string(VV_EXCEPTION, (*caught_stack.get()).value, -1 as ptrdiff_t);
        set_vim_var_list(VV_STACKTRACE, (*caught_stack.get()).stacktrace);
        if *(*caught_stack.get()).throw_name as ::core::ffi::c_int != NUL {
            let mut IObufflen: size_t = 0;
            if (*caught_stack.get()).throw_lnum != 0 as linenr_T {
                IObufflen = vim_snprintf_safelen(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    gettext(b"%s, line %ld\0".as_ptr() as *const ::core::ffi::c_char),
                    (*caught_stack.get()).throw_name,
                    (*caught_stack.get()).throw_lnum as int64_t,
                );
            } else {
                IObufflen = vim_snprintf_safelen(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    (*caught_stack.get()).throw_name,
                );
            }
            set_vim_var_string(
                VV_THROWPOINT,
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IObufflen as ptrdiff_t,
            );
        } else {
            set_vim_var_string(
                VV_THROWPOINT,
                ::core::ptr::null::<::core::ffi::c_char>(),
                -1 as ptrdiff_t,
            );
        }
    } else {
        set_vim_var_string(
            VV_EXCEPTION,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        set_vim_var_string(
            VV_THROWPOINT,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        set_vim_var_list(VV_STACKTRACE, ::core::ptr::null_mut::<list_T>());
    }
    discard_exception(excp, true_0 != 0);
}
pub unsafe extern "C" fn exception_state_save(mut estate: *mut exception_state_T) {
    (*estate).estate_current_exception = current_exception.get();
    (*estate).estate_did_throw = did_throw.get();
    (*estate).estate_need_rethrow = need_rethrow.get();
    (*estate).estate_trylevel = trylevel.get();
    (*estate).estate_did_emsg = did_emsg.get();
}
pub unsafe extern "C" fn exception_state_restore(mut estate: *mut exception_state_T) {
    if did_throw.get() {
        handle_did_throw();
    }
    current_exception.set((*estate).estate_current_exception);
    did_throw.set((*estate).estate_did_throw);
    need_rethrow.set((*estate).estate_need_rethrow);
    trylevel.set((*estate).estate_trylevel);
    did_emsg.set((*estate).estate_did_emsg);
}
pub unsafe extern "C" fn exception_state_clear() {
    current_exception.set(::core::ptr::null_mut::<except_T>());
    did_throw.set(false_0 != 0);
    need_rethrow.set(false_0 != 0);
    trylevel.set(0 as ::core::ffi::c_int);
    did_emsg.set(0 as ::core::ffi::c_int);
}
pub const RP_MAKE: ::core::ffi::c_int = 0;
pub const RP_RESUME: ::core::ffi::c_int = 1;
pub const RP_DISCARD: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
unsafe extern "C" fn report_pending(
    mut action: ::core::ffi::c_int,
    mut pending: ::core::ffi::c_int,
    mut value: *mut ::core::ffi::c_void,
) {
    let mut mesg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    '_c2rust_label: {
        if !value.is_null() || pending & CSTP_THROW as ::core::ffi::c_int == 0 {
        } else {
            __assert_fail(
                b"value || !(pending & CSTP_THROW)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ex_eval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                723 as ::core::ffi::c_uint,
                b"void report_pending(int, int, void *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    match action {
        RP_MAKE => {
            mesg = gettext(b"%s made pending\0".as_ptr() as *const ::core::ffi::c_char);
        }
        RP_RESUME => {
            mesg = gettext(b"%s resumed\0".as_ptr() as *const ::core::ffi::c_char);
        }
        _ => {
            mesg = gettext(b"%s discarded\0".as_ptr() as *const ::core::ffi::c_char);
        }
    }
    match pending {
        0 => return,
        16 => {
            s = b":continue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        8 => {
            s = b":break\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        32 => {
            s = b":finish\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        24 => {
            s = get_return_cmd(value);
        }
        _ => {
            if pending & CSTP_THROW as ::core::ffi::c_int != 0 {
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    mesg,
                    gettext(b"Exception\0".as_ptr() as *const ::core::ffi::c_char),
                );
                mesg = concat_str(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    b": %s\0".as_ptr() as *const ::core::ffi::c_char,
                );
                s = (*(value as *mut except_T)).value;
            } else if pending & CSTP_ERROR as ::core::ffi::c_int != 0
                && pending & CSTP_INTERRUPT as ::core::ffi::c_int != 0
            {
                s = gettext(b"Error and interrupt\0".as_ptr() as *const ::core::ffi::c_char);
            } else if pending & CSTP_ERROR as ::core::ffi::c_int != 0 {
                s = gettext(b"Error\0".as_ptr() as *const ::core::ffi::c_char);
            } else {
                s = gettext(b"Interrupt\0".as_ptr() as *const ::core::ffi::c_char);
            }
        }
    }
    let mut save_msg_silent: ::core::ffi::c_int = msg_silent.get();
    if debug_break_level.get() > 0 as ::core::ffi::c_int {
        msg_silent.set(false_0);
    }
    (*no_wait_return.ptr()) += 1;
    msg_scroll.set(true_0);
    smsg(0 as ::core::ffi::c_int, mesg, s);
    msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
    cmdline_row.set(msg_row.get());
    (*no_wait_return.ptr()) -= 1;
    if debug_break_level.get() > 0 as ::core::ffi::c_int {
        msg_silent.set(save_msg_silent);
    }
    if pending == CSTP_RETURN as ::core::ffi::c_int {
        xfree(s as *mut ::core::ffi::c_void);
    } else if pending & CSTP_THROW as ::core::ffi::c_int != 0 {
        xfree(mesg as *mut ::core::ffi::c_void);
    }
}
pub unsafe extern "C" fn report_make_pending(
    mut pending: ::core::ffi::c_int,
    mut value: *mut ::core::ffi::c_void,
) {
    if p_verbose.get() >= 14 as OptInt || debug_break_level.get() > 0 as ::core::ffi::c_int {
        if debug_break_level.get() <= 0 as ::core::ffi::c_int {
            verbose_enter();
        }
        report_pending(RP_MAKE, pending, value);
        if debug_break_level.get() <= 0 as ::core::ffi::c_int {
            verbose_leave();
        }
    }
}
unsafe extern "C" fn report_resume_pending(
    mut pending: ::core::ffi::c_int,
    mut value: *mut ::core::ffi::c_void,
) {
    if p_verbose.get() >= 14 as OptInt || debug_break_level.get() > 0 as ::core::ffi::c_int {
        if debug_break_level.get() <= 0 as ::core::ffi::c_int {
            verbose_enter();
        }
        report_pending(RP_RESUME, pending, value);
        if debug_break_level.get() <= 0 as ::core::ffi::c_int {
            verbose_leave();
        }
    }
}
unsafe extern "C" fn report_discard_pending(
    mut pending: ::core::ffi::c_int,
    mut value: *mut ::core::ffi::c_void,
) {
    if p_verbose.get() >= 14 as OptInt || debug_break_level.get() > 0 as ::core::ffi::c_int {
        if debug_break_level.get() <= 0 as ::core::ffi::c_int {
            verbose_enter();
        }
        report_pending(RP_DISCARD, pending, value);
        if debug_break_level.get() <= 0 as ::core::ffi::c_int {
            verbose_leave();
        }
    }
}
pub unsafe fn ex_eval(mut eap: *mut exarg_T) {
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, (*eap).skip != 0);
    if eval0((*eap).arg, &raw mut tv, eap, &raw mut evalarg) == OK {
        tv_clear(&raw mut tv);
    }
    clear_evalarg(&raw mut evalarg, eap);
}
pub unsafe fn ex_if(mut eap: *mut exarg_T) {
    let cstack: *mut cstack_T = (*eap).cstack;
    if (*cstack).cs_idx == CSTACK_LEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
        (*eap).errmsg =
            gettext(b"E579: :if nesting too deep\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        (*cstack).cs_idx += 1;
        (*cstack).cs_flags[(*cstack).cs_idx as usize] = 0 as ::core::ffi::c_int;
        let mut skip: bool = did_emsg.get() != 0
            || got_int.get() as ::core::ffi::c_int != 0
            || did_throw.get() as ::core::ffi::c_int != 0
            || (*cstack).cs_idx > 0 as ::core::ffi::c_int
                && (*cstack).cs_flags[((*cstack).cs_idx - 1 as ::core::ffi::c_int) as usize]
                    & CSF_ACTIVE as ::core::ffi::c_int
                    == 0;
        let mut error: bool = false;
        let mut result: bool = eval_to_bool((*eap).arg, &raw mut error, eap, skip, false_0 != 0);
        if !skip && !error {
            if result {
                (*cstack).cs_flags[(*cstack).cs_idx as usize] =
                    CSF_ACTIVE as ::core::ffi::c_int | CSF_TRUE as ::core::ffi::c_int;
            }
        } else {
            (*cstack).cs_flags[(*cstack).cs_idx as usize] = CSF_TRUE as ::core::ffi::c_int;
        }
    };
}
pub unsafe fn ex_endif(mut eap: *mut exarg_T) {
    did_endif.set(true_0 != 0);
    if (*(*eap).cstack).cs_idx < 0 as ::core::ffi::c_int
        || (*(*eap).cstack).cs_flags[(*(*eap).cstack).cs_idx as usize]
            & (CSF_WHILE as ::core::ffi::c_int
                | CSF_FOR as ::core::ffi::c_int
                | CSF_TRY as ::core::ffi::c_int)
            != 0
    {
        (*eap).errmsg =
            gettext(b"E580: :endif without :if\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        if (*(*eap).cstack).cs_flags[(*(*eap).cstack).cs_idx as usize]
            & CSF_TRUE as ::core::ffi::c_int
            == 0
            && dbg_check_skipped(eap) as ::core::ffi::c_int != 0
        {
            do_intthrow((*eap).cstack);
        }
        (*(*eap).cstack).cs_idx -= 1;
    };
}
pub unsafe fn ex_else(mut eap: *mut exarg_T) {
    let cstack: *mut cstack_T = (*eap).cstack;
    let mut skip: bool = did_emsg.get() != 0
        || got_int.get() as ::core::ffi::c_int != 0
        || did_throw.get() as ::core::ffi::c_int != 0
        || (*cstack).cs_idx > 0 as ::core::ffi::c_int
            && (*cstack).cs_flags[((*cstack).cs_idx - 1 as ::core::ffi::c_int) as usize]
                & CSF_ACTIVE as ::core::ffi::c_int
                == 0;
    if (*cstack).cs_idx < 0 as ::core::ffi::c_int
        || (*cstack).cs_flags[(*cstack).cs_idx as usize]
            & (CSF_WHILE as ::core::ffi::c_int
                | CSF_FOR as ::core::ffi::c_int
                | CSF_TRY as ::core::ffi::c_int)
            != 0
    {
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_else as ::core::ffi::c_int {
            (*eap).errmsg =
                gettext(b"E581: :else without :if\0".as_ptr() as *const ::core::ffi::c_char);
            return;
        }
        (*eap).errmsg =
            gettext(b"E582: :elseif without :if\0".as_ptr() as *const ::core::ffi::c_char);
        skip = true_0 != 0;
    } else if (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_ELSE as ::core::ffi::c_int != 0 {
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_else as ::core::ffi::c_int {
            (*eap).errmsg =
                gettext((e_multiple_else.ptr() as *const _) as *const ::core::ffi::c_char);
            return;
        }
        (*eap).errmsg =
            gettext(b"E584: :elseif after :else\0".as_ptr() as *const ::core::ffi::c_char);
        skip = true_0 != 0;
    }
    if skip as ::core::ffi::c_int != 0
        || (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_TRUE as ::core::ffi::c_int != 0
    {
        if (*eap).errmsg.is_null() {
            (*cstack).cs_flags[(*cstack).cs_idx as usize] = CSF_TRUE as ::core::ffi::c_int;
        }
        skip = true_0 != 0;
    } else {
        (*cstack).cs_flags[(*cstack).cs_idx as usize] = CSF_ACTIVE as ::core::ffi::c_int;
    }
    if !skip
        && dbg_check_skipped(eap) as ::core::ffi::c_int != 0
        && got_int.get() as ::core::ffi::c_int != 0
    {
        do_intthrow(cstack);
        skip = true_0 != 0;
    }
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_elseif as ::core::ffi::c_int {
        let mut result: bool = false_0 != 0;
        let mut error: bool = false;
        if skip as ::core::ffi::c_int != 0
            && *(*eap).arg as ::core::ffi::c_int != '"' as ::core::ffi::c_int
            && ends_excmd(*(*eap).arg as ::core::ffi::c_int) != 0
        {
            semsg(
                gettext(&raw const e_invexpr2 as *const ::core::ffi::c_char),
                (*eap).arg,
            );
        } else {
            result = eval_to_bool((*eap).arg, &raw mut error, eap, skip, false_0 != 0);
        }
        if !skip && !error {
            if result {
                (*cstack).cs_flags[(*cstack).cs_idx as usize] =
                    CSF_ACTIVE as ::core::ffi::c_int | CSF_TRUE as ::core::ffi::c_int;
            } else {
                (*cstack).cs_flags[(*cstack).cs_idx as usize] = 0 as ::core::ffi::c_int;
            }
        } else if (*eap).errmsg.is_null() {
            (*cstack).cs_flags[(*cstack).cs_idx as usize] = CSF_TRUE as ::core::ffi::c_int;
        }
    } else {
        (*cstack).cs_flags[(*cstack).cs_idx as usize] |= CSF_ELSE as ::core::ffi::c_int;
    };
}
pub unsafe fn ex_while(mut eap: *mut exarg_T) {
    let mut error: bool = false;
    let cstack: *mut cstack_T = (*eap).cstack;
    if (*cstack).cs_idx == CSTACK_LEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
        (*eap).errmsg =
            gettext(b"E585: :while/:for nesting too deep\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        let mut result: bool = false;
        if (*cstack).cs_lflags & CSL_HAD_LOOP as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            (*cstack).cs_idx += 1;
            (*cstack).cs_looplevel += 1;
            (*cstack).cs_line[(*cstack).cs_idx as usize] = -1 as ::core::ffi::c_int;
        }
        (*cstack).cs_flags[(*cstack).cs_idx as usize] =
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_while as ::core::ffi::c_int {
                CSF_WHILE as ::core::ffi::c_int
            } else {
                CSF_FOR as ::core::ffi::c_int
            };
        let mut skip: ::core::ffi::c_int = (did_emsg.get() != 0
            || got_int.get() as ::core::ffi::c_int != 0
            || did_throw.get() as ::core::ffi::c_int != 0
            || (*cstack).cs_idx > 0 as ::core::ffi::c_int
                && (*cstack).cs_flags[((*cstack).cs_idx - 1 as ::core::ffi::c_int) as usize]
                    & CSF_ACTIVE as ::core::ffi::c_int
                    == 0) as ::core::ffi::c_int;
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_while as ::core::ffi::c_int {
            result = eval_to_bool((*eap).arg, &raw mut error, eap, skip != 0, false_0 != 0);
        } else {
            let mut evalarg: evalarg_T = evalarg_T {
                eval_flags: 0,
                eval_getline: None,
                eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
            fill_evalarg_from_eap(&raw mut evalarg, eap, skip != 0);
            let mut fi: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
            if (*cstack).cs_lflags & CSL_HAD_LOOP as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                fi = (*cstack).cs_forinfo[(*cstack).cs_idx as usize];
                error = false_0 != 0;
            } else {
                fi = eval_for_line((*eap).arg, &raw mut error, eap, &raw mut evalarg);
                (*cstack).cs_forinfo[(*cstack).cs_idx as usize] = fi;
            }
            if !error && !fi.is_null() && skip == 0 {
                result = next_for_item(fi, (*eap).arg);
            } else {
                result = false_0 != 0;
            }
            if !result {
                free_for_info(fi);
                (*cstack).cs_forinfo[(*cstack).cs_idx as usize] = NULL;
            }
            clear_evalarg(&raw mut evalarg, eap);
        }
        if skip == 0 && !error && result as ::core::ffi::c_int != 0 {
            (*cstack).cs_flags[(*cstack).cs_idx as usize] |=
                CSF_ACTIVE as ::core::ffi::c_int | CSF_TRUE as ::core::ffi::c_int;
            (*cstack).cs_lflags ^= CSL_HAD_LOOP as ::core::ffi::c_int;
        } else {
            (*cstack).cs_lflags &= !(CSL_HAD_LOOP as ::core::ffi::c_int);
            if skip == 0 && !error {
                (*cstack).cs_flags[(*cstack).cs_idx as usize] |= CSF_TRUE as ::core::ffi::c_int;
            }
        }
    };
}
pub unsafe fn ex_continue(mut eap: *mut exarg_T) {
    let cstack: *mut cstack_T = (*eap).cstack;
    if (*cstack).cs_looplevel <= 0 as ::core::ffi::c_int
        || (*cstack).cs_idx < 0 as ::core::ffi::c_int
    {
        (*eap).errmsg = gettext(
            b"E586: :continue without :while or :for\0".as_ptr() as *const ::core::ffi::c_char
        );
    } else {
        let mut idx: ::core::ffi::c_int = cleanup_conditionals(
            cstack,
            CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int,
            false_0,
        );
        '_c2rust_label: {
            if idx >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"idx >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ex_eval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1069 as ::core::ffi::c_uint,
                    b"void ex_continue(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if (*cstack).cs_flags[idx as usize]
            & (CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int)
            != 0
        {
            rewind_conditionals(
                cstack,
                idx,
                CSF_TRY as ::core::ffi::c_int,
                &raw mut (*cstack).cs_trylevel,
            );
            (*cstack).cs_lflags |= CSL_HAD_CONT as ::core::ffi::c_int;
        } else {
            (*cstack).cs_pending[idx as usize] =
                CSTP_CONTINUE as ::core::ffi::c_int as ::core::ffi::c_char;
            report_make_pending(CSTP_CONTINUE as ::core::ffi::c_int, NULL);
        }
    };
}
pub unsafe fn ex_break(mut eap: *mut exarg_T) {
    let cstack: *mut cstack_T = (*eap).cstack;
    if (*cstack).cs_looplevel <= 0 as ::core::ffi::c_int
        || (*cstack).cs_idx < 0 as ::core::ffi::c_int
    {
        (*eap).errmsg = gettext(
            b"E587: :break without :while or :for\0".as_ptr() as *const ::core::ffi::c_char
        );
    } else {
        let mut idx: ::core::ffi::c_int = cleanup_conditionals(
            cstack,
            CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int,
            true_0,
        );
        if idx >= 0 as ::core::ffi::c_int
            && (*cstack).cs_flags[idx as usize]
                & (CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int)
                == 0
        {
            (*cstack).cs_pending[idx as usize] =
                CSTP_BREAK as ::core::ffi::c_int as ::core::ffi::c_char;
            report_make_pending(CSTP_BREAK as ::core::ffi::c_int, NULL);
        }
    };
}
pub unsafe fn ex_endwhile(mut eap: *mut exarg_T) {
    let cstack: *mut cstack_T = (*eap).cstack;
    let mut err: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut csf: ::core::ffi::c_int = 0;
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_endwhile as ::core::ffi::c_int {
        err = &raw const e_while as *const ::core::ffi::c_char;
        csf = CSF_WHILE as ::core::ffi::c_int;
    } else {
        err = &raw const e_for as *const ::core::ffi::c_char;
        csf = CSF_FOR as ::core::ffi::c_int;
    }
    if (*cstack).cs_looplevel <= 0 as ::core::ffi::c_int
        || (*cstack).cs_idx < 0 as ::core::ffi::c_int
    {
        (*eap).errmsg = gettext(err);
    } else {
        let mut fl: ::core::ffi::c_int = (*cstack).cs_flags[(*cstack).cs_idx as usize];
        if fl & csf == 0 {
            if fl & CSF_WHILE as ::core::ffi::c_int != 0 {
                (*eap).errmsg = gettext(
                    b"E732: Using :endfor with :while\0".as_ptr() as *const ::core::ffi::c_char
                );
            } else if fl & CSF_FOR as ::core::ffi::c_int != 0 {
                (*eap).errmsg = gettext(
                    b"E733: Using :endwhile with :for\0".as_ptr() as *const ::core::ffi::c_char
                );
            }
        }
        if fl & (CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int) == 0 {
            if fl & CSF_TRY as ::core::ffi::c_int == 0 {
                (*eap).errmsg = gettext(&raw const e_endif as *const ::core::ffi::c_char);
            } else if fl & CSF_FINALLY as ::core::ffi::c_int != 0 {
                (*eap).errmsg = gettext(&raw const e_endtry as *const ::core::ffi::c_char);
            }
            let mut idx: ::core::ffi::c_int = 0;
            idx = (*cstack).cs_idx;
            while idx > 0 as ::core::ffi::c_int {
                fl = (*cstack).cs_flags[idx as usize];
                if fl & CSF_TRY as ::core::ffi::c_int != 0
                    && fl & CSF_FINALLY as ::core::ffi::c_int == 0
                {
                    (*eap).errmsg = gettext(err);
                    return;
                }
                if fl & csf != 0 {
                    break;
                }
                idx -= 1;
            }
            cleanup_conditionals(
                cstack,
                CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int,
                false_0,
            );
            rewind_conditionals(
                cstack,
                idx,
                CSF_TRY as ::core::ffi::c_int,
                &raw mut (*cstack).cs_trylevel,
            );
        } else if (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_TRUE as ::core::ffi::c_int
            != 0
            && (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_ACTIVE as ::core::ffi::c_int == 0
            && dbg_check_skipped(eap) as ::core::ffi::c_int != 0
        {
            do_intthrow(cstack);
        }
        (*cstack).cs_lflags |= CSL_HAD_ENDLOOP as ::core::ffi::c_int;
    };
}
pub unsafe fn ex_throw(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *arg as ::core::ffi::c_int != NUL
        && *arg as ::core::ffi::c_int != '|' as ::core::ffi::c_int
        && *arg as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
    {
        value = eval_to_string_skip(arg, eap, (*eap).skip != 0);
    } else {
        emsg(gettext(&raw const e_argreq as *const ::core::ffi::c_char));
        value = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if (*eap).skip == 0 && !value.is_null() {
        if throw_exception(
            value as *mut ::core::ffi::c_void,
            ET_USER,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ) == FAIL
        {
            xfree(value as *mut ::core::ffi::c_void);
        } else {
            do_throw((*eap).cstack);
        }
    }
}
pub unsafe extern "C" fn do_throw(mut cstack: *mut cstack_T) {
    let mut inactivate_try: bool = false_0 != 0;
    let mut idx: ::core::ffi::c_int = cleanup_conditionals(
        cstack,
        0 as ::core::ffi::c_int,
        inactivate_try as ::core::ffi::c_int,
    );
    if idx >= 0 as ::core::ffi::c_int {
        if (*cstack).cs_flags[idx as usize] & CSF_CAUGHT as ::core::ffi::c_int == 0 {
            if (*cstack).cs_flags[idx as usize] & CSF_ACTIVE as ::core::ffi::c_int != 0 {
                (*cstack).cs_flags[idx as usize] |= CSF_THROWN as ::core::ffi::c_int;
            } else {
                (*cstack).cs_flags[idx as usize] &= !(CSF_THROWN as ::core::ffi::c_int);
            }
        }
        (*cstack).cs_flags[idx as usize] &= !(CSF_ACTIVE as ::core::ffi::c_int);
        (*cstack).cs_pend.csp_ex[idx as usize] =
            current_exception.get() as *mut ::core::ffi::c_void;
    }
    did_throw.set(true_0 != 0);
}
pub unsafe fn ex_try(mut eap: *mut exarg_T) {
    let cstack: *mut cstack_T = (*eap).cstack;
    if (*cstack).cs_idx == CSTACK_LEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
        (*eap).errmsg =
            gettext(b"E601: :try nesting too deep\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        (*cstack).cs_idx += 1;
        (*cstack).cs_trylevel += 1;
        (*cstack).cs_flags[(*cstack).cs_idx as usize] = CSF_TRY as ::core::ffi::c_int;
        (*cstack).cs_pending[(*cstack).cs_idx as usize] =
            CSTP_NONE as ::core::ffi::c_int as ::core::ffi::c_char;
        let mut skip: ::core::ffi::c_int = (did_emsg.get() != 0
            || got_int.get() as ::core::ffi::c_int != 0
            || did_throw.get() as ::core::ffi::c_int != 0
            || (*cstack).cs_idx > 0 as ::core::ffi::c_int
                && (*cstack).cs_flags[((*cstack).cs_idx - 1 as ::core::ffi::c_int) as usize]
                    & CSF_ACTIVE as ::core::ffi::c_int
                    == 0) as ::core::ffi::c_int;
        if skip == 0 {
            (*cstack).cs_flags[(*cstack).cs_idx as usize] |=
                CSF_ACTIVE as ::core::ffi::c_int | CSF_TRUE as ::core::ffi::c_int;
            if emsg_silent.get() != 0 {
                let mut elem: *mut eslist_T =
                    xmalloc(::core::mem::size_of::<eslist_T>()) as *mut eslist_T;
                (*elem).saved_emsg_silent = emsg_silent.get();
                (*elem).next = (*cstack).cs_emsg_silent_list;
                (*cstack).cs_emsg_silent_list = elem;
                (*cstack).cs_flags[(*cstack).cs_idx as usize] |= CSF_SILENT as ::core::ffi::c_int;
                emsg_silent.set(0 as ::core::ffi::c_int);
            }
        }
    };
}
pub unsafe fn ex_catch(mut eap: *mut exarg_T) {
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut give_up: bool = false_0 != 0;
    let mut skip: bool = false_0 != 0;
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut save_cpo: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let cstack: *mut cstack_T = (*eap).cstack;
    let mut pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*cstack).cs_trylevel <= 0 as ::core::ffi::c_int
        || (*cstack).cs_idx < 0 as ::core::ffi::c_int
    {
        (*eap).errmsg =
            gettext(b"E603: :catch without :try\0".as_ptr() as *const ::core::ffi::c_char);
        give_up = true_0 != 0;
    } else {
        if (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_TRY as ::core::ffi::c_int == 0 {
            (*eap).errmsg = get_end_emsg(cstack);
            skip = true_0 != 0;
        }
        idx = (*cstack).cs_idx;
        while idx > 0 as ::core::ffi::c_int {
            if (*cstack).cs_flags[idx as usize] & CSF_TRY as ::core::ffi::c_int != 0 {
                break;
            }
            idx -= 1;
        }
        if (*cstack).cs_flags[idx as usize] & CSF_FINALLY as ::core::ffi::c_int != 0 {
            (*eap).errmsg =
                gettext(b"E604: :catch after :finally\0".as_ptr() as *const ::core::ffi::c_char);
            give_up = true_0 != 0;
        } else {
            rewind_conditionals(
                cstack,
                idx,
                CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int,
                &raw mut (*cstack).cs_looplevel,
            );
        }
    }
    if ends_excmd(*(*eap).arg as ::core::ffi::c_int) != 0 {
        pat = b".*\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        end = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*eap).nextcmd = find_nextcmd((*eap).arg);
    } else {
        pat = (*eap).arg.offset(1 as ::core::ffi::c_int as isize);
        end = skip_regexp_err(pat, *(*eap).arg as ::core::ffi::c_int, true_0);
        if end.is_null() {
            give_up = true_0 != 0;
        }
    }
    if !give_up {
        let mut caught: bool = false_0 != 0;
        if !did_throw.get()
            || (*cstack).cs_flags[idx as usize] & CSF_TRUE as ::core::ffi::c_int == 0
        {
            skip = true_0 != 0;
        }
        if !skip
            && (*cstack).cs_flags[idx as usize] & CSF_THROWN as ::core::ffi::c_int != 0
            && (*cstack).cs_flags[idx as usize] & CSF_CAUGHT as ::core::ffi::c_int == 0
        {
            if !end.is_null()
                && *end as ::core::ffi::c_int != NUL
                && ends_excmd(
                    *skipwhite(end.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                ) == 0
            {
                semsg(
                    gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                    end,
                );
                return;
            }
            if !dbg_check_skipped(eap) || !do_intthrow(cstack) {
                let mut save_char: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
                if !end.is_null() {
                    save_char = *end;
                    *end = NUL as ::core::ffi::c_char;
                }
                save_cpo = p_cpo.get();
                p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
                (*emsg_off.ptr()) += 1;
                regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
                (*emsg_off.ptr()) -= 1;
                regmatch.rm_ic = false_0 != 0;
                if !end.is_null() {
                    *end = save_char;
                }
                p_cpo.set(save_cpo);
                if regmatch.regprog.is_null() {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        pat,
                    );
                } else {
                    let mut prev_got_int: ::core::ffi::c_int = got_int.get() as ::core::ffi::c_int;
                    got_int.set(false_0 != 0);
                    caught = vim_regexec_nl(
                        &raw mut regmatch,
                        (*current_exception.get()).value,
                        0 as colnr_T,
                    );
                    got_int.set(got_int.get() as ::core::ffi::c_int | prev_got_int != 0);
                    vim_regfree(regmatch.regprog);
                }
            }
        }
        if caught {
            (*cstack).cs_flags[idx as usize] |=
                CSF_ACTIVE as ::core::ffi::c_int | CSF_CAUGHT as ::core::ffi::c_int;
            did_throw.set(false_0 != 0);
            got_int.set(did_throw.get());
            did_emsg.set(got_int.get() as ::core::ffi::c_int);
            catch_exception((*cstack).cs_pend.csp_ex[idx as usize] as *mut except_T);
            if (*cstack).cs_pend.csp_ex[(*cstack).cs_idx as usize]
                != current_exception.get() as *mut ::core::ffi::c_void
            {
                internal_error(b"ex_catch()\0".as_ptr() as *const ::core::ffi::c_char);
            }
        } else {
            cleanup_conditionals(cstack, CSF_TRY as ::core::ffi::c_int, true_0);
        }
    }
    if !end.is_null() {
        (*eap).nextcmd = find_nextcmd(end);
    }
}
pub unsafe fn ex_finally(mut eap: *mut exarg_T) {
    let mut idx: ::core::ffi::c_int = 0;
    let mut pending: ::core::ffi::c_int = CSTP_NONE as ::core::ffi::c_int;
    let cstack: *mut cstack_T = (*eap).cstack;
    idx = (*cstack).cs_idx;
    while idx >= 0 as ::core::ffi::c_int {
        if (*cstack).cs_flags[idx as usize] & CSF_TRY as ::core::ffi::c_int != 0 {
            break;
        }
        idx -= 1;
    }
    if (*cstack).cs_trylevel <= 0 as ::core::ffi::c_int || idx < 0 as ::core::ffi::c_int {
        (*eap).errmsg =
            gettext(b"E606: :finally without :try\0".as_ptr() as *const ::core::ffi::c_char);
        return;
    }
    if (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_TRY as ::core::ffi::c_int == 0 {
        (*eap).errmsg = get_end_emsg(cstack);
        pending = CSTP_ERROR as ::core::ffi::c_int;
    }
    if (*cstack).cs_flags[idx as usize] & CSF_FINALLY as ::core::ffi::c_int != 0 {
        (*eap).errmsg =
            gettext((e_multiple_finally.ptr() as *const _) as *const ::core::ffi::c_char);
        return;
    }
    rewind_conditionals(
        cstack,
        idx,
        CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int,
        &raw mut (*cstack).cs_looplevel,
    );
    let mut skip: ::core::ffi::c_int = ((*cstack).cs_flags[(*cstack).cs_idx as usize]
        & CSF_TRUE as ::core::ffi::c_int
        == 0) as ::core::ffi::c_int;
    if skip == 0 {
        if dbg_check_skipped(eap) {
            do_intthrow(cstack);
        }
        cleanup_conditionals(cstack, CSF_TRY as ::core::ffi::c_int, false_0);
        if pending == CSTP_ERROR as ::core::ffi::c_int
            || did_emsg.get() != 0
            || got_int.get() as ::core::ffi::c_int != 0
            || did_throw.get() as ::core::ffi::c_int != 0
        {
            if (*cstack).cs_pending[(*cstack).cs_idx as usize] as ::core::ffi::c_int
                == CSTP_RETURN as ::core::ffi::c_int
            {
                report_discard_pending(
                    CSTP_RETURN as ::core::ffi::c_int,
                    (*cstack).cs_pend.csp_rv[(*cstack).cs_idx as usize],
                );
                discard_pending_return(
                    (*cstack).cs_pend.csp_rv[(*cstack).cs_idx as usize] as *mut typval_T,
                );
            }
            if pending == CSTP_ERROR as ::core::ffi::c_int && did_emsg.get() == 0 {
                pending |= if THROW_ON_ERROR != 0 {
                    CSTP_THROW as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
            } else {
                pending |= if did_throw.get() as ::core::ffi::c_int != 0 {
                    CSTP_THROW as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
            }
            pending |= if did_emsg.get() != 0 {
                CSTP_ERROR as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
            pending |= if got_int.get() as ::core::ffi::c_int != 0 {
                CSTP_INTERRUPT as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
            '_c2rust_label: {
                if pending >= -127 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                    && pending <= 127 as ::core::ffi::c_int
                {
                } else {
                    __assert_fail(
                        b"pending >= CHAR_MIN && pending <= CHAR_MAX\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/ex_eval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1535 as ::core::ffi::c_uint,
                        b"void ex_finally(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            (*cstack).cs_pending[(*cstack).cs_idx as usize] = pending as ::core::ffi::c_char;
            if did_throw.get() as ::core::ffi::c_int != 0
                && (*cstack).cs_pend.csp_ex[(*cstack).cs_idx as usize]
                    != current_exception.get() as *mut ::core::ffi::c_void
            {
                internal_error(b"ex_finally()\0".as_ptr() as *const ::core::ffi::c_char);
            }
        }
        (*cstack).cs_lflags |= CSL_HAD_FINA as ::core::ffi::c_int;
    }
}
pub unsafe fn ex_endtry(mut eap: *mut exarg_T) {
    let mut idx: ::core::ffi::c_int = 0;
    let mut rethrow: bool = false_0 != 0;
    let mut pending: ::core::ffi::c_char = CSTP_NONE as ::core::ffi::c_int as ::core::ffi::c_char;
    let mut rettv: *mut ::core::ffi::c_void = NULL;
    let cstack: *mut cstack_T = (*eap).cstack;
    idx = (*cstack).cs_idx;
    while idx >= 0 as ::core::ffi::c_int {
        if (*cstack).cs_flags[idx as usize] & CSF_TRY as ::core::ffi::c_int != 0 {
            break;
        }
        idx -= 1;
    }
    if (*cstack).cs_trylevel <= 0 as ::core::ffi::c_int || idx < 0 as ::core::ffi::c_int {
        (*eap).errmsg =
            gettext(b"E602: :endtry without :try\0".as_ptr() as *const ::core::ffi::c_char);
        return;
    }
    let mut skip: bool = did_emsg.get() != 0
        || got_int.get() as ::core::ffi::c_int != 0
        || did_throw.get() as ::core::ffi::c_int != 0
        || (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_TRUE as ::core::ffi::c_int == 0;
    if (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_TRY as ::core::ffi::c_int == 0 {
        (*eap).errmsg = get_end_emsg(cstack);
        rewind_conditionals(
            cstack,
            idx,
            CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int,
            &raw mut (*cstack).cs_looplevel,
        );
        skip = true_0 != 0;
        if did_throw.get() {
            discard_current_exception();
        }
        did_emsg.set(false_0);
    } else {
        idx = (*cstack).cs_idx;
        if did_throw.get() as ::core::ffi::c_int != 0
            && (*cstack).cs_flags[idx as usize] & CSF_TRUE as ::core::ffi::c_int != 0
            && (*cstack).cs_flags[idx as usize] & CSF_FINALLY as ::core::ffi::c_int == 0
        {
            rethrow = true_0 != 0;
        }
    }
    if (rethrow as ::core::ffi::c_int != 0
        || !skip
            && (*cstack).cs_flags[idx as usize] & CSF_FINALLY as ::core::ffi::c_int == 0
            && (*cstack).cs_pending[idx as usize] == 0)
        && dbg_check_skipped(eap) as ::core::ffi::c_int != 0
    {
        if got_int.get() {
            skip = true_0 != 0;
            do_intthrow(cstack);
            rethrow = false_0 != 0;
            if did_throw.get() as ::core::ffi::c_int != 0
                && (*cstack).cs_flags[idx as usize] & CSF_FINALLY as ::core::ffi::c_int == 0
            {
                rethrow = true_0 != 0;
            }
        }
    }
    if !skip {
        pending = (*cstack).cs_pending[idx as usize];
        (*cstack).cs_pending[idx as usize] = CSTP_NONE as ::core::ffi::c_int as ::core::ffi::c_char;
        if pending as ::core::ffi::c_int == CSTP_RETURN as ::core::ffi::c_int {
            rettv = (*cstack).cs_pend.csp_rv[idx as usize];
        } else if pending as ::core::ffi::c_int & CSTP_THROW as ::core::ffi::c_int != 0 {
            current_exception.set((*cstack).cs_pend.csp_ex[idx as usize] as *mut except_T);
        }
    }
    cleanup_conditionals(
        cstack,
        CSF_TRY as ::core::ffi::c_int | CSF_SILENT as ::core::ffi::c_int,
        true_0,
    );
    if (*cstack).cs_idx >= 0 as ::core::ffi::c_int
        && (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_TRY as ::core::ffi::c_int != 0
    {
        (*cstack).cs_idx -= 1;
    }
    (*cstack).cs_trylevel -= 1;
    if !skip {
        report_resume_pending(
            pending as ::core::ffi::c_int,
            if pending as ::core::ffi::c_int == CSTP_RETURN as ::core::ffi::c_int {
                rettv
            } else if pending as ::core::ffi::c_int & CSTP_THROW as ::core::ffi::c_int != 0 {
                current_exception.get() as *mut ::core::ffi::c_void
            } else {
                NULL
            },
        );
        match pending as ::core::ffi::c_int {
            0 => {}
            16 => {
                ex_continue(eap);
            }
            8 => {
                ex_break(eap);
            }
            24 => {
                do_return(eap, false_0 != 0, false_0 != 0, rettv);
            }
            32 => {
                do_finish(eap, false_0 != 0);
            }
            _ => {
                if pending as ::core::ffi::c_int & CSTP_ERROR as ::core::ffi::c_int != 0 {
                    did_emsg.set(true_0);
                }
                if pending as ::core::ffi::c_int & CSTP_INTERRUPT as ::core::ffi::c_int != 0 {
                    got_int.set(true_0 != 0);
                }
                if pending as ::core::ffi::c_int & CSTP_THROW as ::core::ffi::c_int != 0 {
                    rethrow = true_0 != 0;
                }
            }
        }
    }
    if rethrow {
        do_throw(cstack);
    }
}
pub unsafe extern "C" fn enter_cleanup(mut csp: *mut cleanup_T) {
    let mut pending: ::core::ffi::c_int = CSTP_NONE as ::core::ffi::c_int;
    if did_emsg.get() != 0
        || got_int.get() as ::core::ffi::c_int != 0
        || did_throw.get() as ::core::ffi::c_int != 0
        || need_rethrow.get() as ::core::ffi::c_int != 0
    {
        (*csp).pending = (if did_emsg.get() != 0 {
            CSTP_ERROR as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | (if got_int.get() as ::core::ffi::c_int != 0 {
            CSTP_INTERRUPT as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | (if did_throw.get() as ::core::ffi::c_int != 0 {
            CSTP_THROW as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | (if need_rethrow.get() as ::core::ffi::c_int != 0 {
            CSTP_THROW as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
        if did_throw.get() as ::core::ffi::c_int != 0
            || need_rethrow.get() as ::core::ffi::c_int != 0
        {
            (*csp).exception = current_exception.get();
            current_exception.set(::core::ptr::null_mut::<except_T>());
        } else {
            (*csp).exception = ::core::ptr::null_mut::<except_T>();
            if did_emsg.get() != 0 {
                force_abort.set(
                    force_abort.get() as ::core::ffi::c_int
                        | cause_abort.get() as ::core::ffi::c_int
                        != 0,
                );
                cause_abort.set(false_0 != 0);
            }
        }
        need_rethrow.set(false_0 != 0);
        did_throw.set(need_rethrow.get());
        got_int.set(did_throw.get());
        did_emsg.set(got_int.get() as ::core::ffi::c_int);
        report_make_pending(pending, (*csp).exception as *mut ::core::ffi::c_void);
    } else {
        (*csp).pending = CSTP_NONE as ::core::ffi::c_int;
        (*csp).exception = ::core::ptr::null_mut::<except_T>();
    };
}
pub unsafe extern "C" fn leave_cleanup(mut csp: *mut cleanup_T) {
    let mut pending: ::core::ffi::c_int = (*csp).pending;
    if pending == CSTP_NONE as ::core::ffi::c_int {
        return;
    }
    if aborting() as ::core::ffi::c_int != 0 || need_rethrow.get() as ::core::ffi::c_int != 0 {
        if pending & CSTP_THROW as ::core::ffi::c_int != 0 {
            discard_exception((*csp).exception, false_0 != 0);
        } else {
            report_discard_pending(pending, NULL);
        }
        if !(*msg_list.ptr()).is_null() {
            free_global_msglist();
        }
    } else {
        if pending & CSTP_THROW as ::core::ffi::c_int != 0 {
            current_exception.set((*csp).exception);
        } else if pending & CSTP_ERROR as ::core::ffi::c_int != 0 {
            cause_abort.set(force_abort.get());
            force_abort.set(false_0 != 0);
        }
        if pending & CSTP_ERROR as ::core::ffi::c_int != 0 {
            did_emsg.set(true_0);
        }
        if pending & CSTP_INTERRUPT as ::core::ffi::c_int != 0 {
            got_int.set(true_0 != 0);
        }
        if pending & CSTP_THROW as ::core::ffi::c_int != 0 {
            need_rethrow.set(true_0 != 0);
        }
        report_resume_pending(
            pending,
            if pending & CSTP_THROW as ::core::ffi::c_int != 0 {
                current_exception.get() as *mut ::core::ffi::c_void
            } else {
                NULL
            },
        );
    };
}
pub unsafe extern "C" fn cleanup_conditionals(
    mut cstack: *mut cstack_T,
    mut searched_cond: ::core::ffi::c_int,
    mut inclusive: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut idx: ::core::ffi::c_int = 0;
    let mut stop: bool = false_0 != 0;
    idx = (*cstack).cs_idx;
    while idx >= 0 as ::core::ffi::c_int {
        if (*cstack).cs_flags[idx as usize] & CSF_TRY as ::core::ffi::c_int != 0 {
            if did_emsg.get() != 0
                || got_int.get() as ::core::ffi::c_int != 0
                || (*cstack).cs_flags[idx as usize] & CSF_FINALLY as ::core::ffi::c_int != 0
            {
                match (*cstack).cs_pending[idx as usize] as ::core::ffi::c_int {
                    0 => {}
                    16 | 8 | 32 => {
                        report_discard_pending(
                            (*cstack).cs_pending[idx as usize] as ::core::ffi::c_int,
                            NULL,
                        );
                        (*cstack).cs_pending[idx as usize] =
                            CSTP_NONE as ::core::ffi::c_int as ::core::ffi::c_char;
                    }
                    24 => {
                        report_discard_pending(
                            CSTP_RETURN as ::core::ffi::c_int,
                            (*cstack).cs_pend.csp_rv[idx as usize],
                        );
                        discard_pending_return(
                            (*cstack).cs_pend.csp_rv[idx as usize] as *mut typval_T,
                        );
                        (*cstack).cs_pending[idx as usize] =
                            CSTP_NONE as ::core::ffi::c_int as ::core::ffi::c_char;
                    }
                    _ => {
                        if (*cstack).cs_flags[idx as usize] & CSF_FINALLY as ::core::ffi::c_int != 0
                        {
                            if (*cstack).cs_pending[idx as usize] as ::core::ffi::c_int
                                & CSTP_THROW as ::core::ffi::c_int
                                != 0
                                && !(*cstack).cs_pend.csp_ex[idx as usize].is_null()
                            {
                                discard_exception(
                                    (*cstack).cs_pend.csp_ex[idx as usize] as *mut except_T,
                                    false_0 != 0,
                                );
                            } else {
                                report_discard_pending(
                                    (*cstack).cs_pending[idx as usize] as ::core::ffi::c_int,
                                    NULL,
                                );
                            }
                            (*cstack).cs_pending[idx as usize] =
                                CSTP_NONE as ::core::ffi::c_int as ::core::ffi::c_char;
                        }
                    }
                }
            }
            if (*cstack).cs_flags[idx as usize] & CSF_FINALLY as ::core::ffi::c_int == 0 {
                if (*cstack).cs_flags[idx as usize] & CSF_ACTIVE as ::core::ffi::c_int != 0
                    && (*cstack).cs_flags[idx as usize] & CSF_CAUGHT as ::core::ffi::c_int != 0
                    && (*cstack).cs_flags[idx as usize] & CSF_FINISHED as ::core::ffi::c_int == 0
                {
                    finish_exception((*cstack).cs_pend.csp_ex[idx as usize] as *mut except_T);
                    (*cstack).cs_flags[idx as usize] |= CSF_FINISHED as ::core::ffi::c_int;
                }
                if (*cstack).cs_flags[idx as usize] & CSF_TRUE as ::core::ffi::c_int != 0 {
                    if searched_cond == 0 as ::core::ffi::c_int && inclusive == 0 {
                        break;
                    }
                    stop = true_0 != 0;
                }
            }
        }
        if (*cstack).cs_flags[idx as usize] & searched_cond != 0 {
            if inclusive == 0 {
                break;
            }
            stop = true_0 != 0;
        }
        (*cstack).cs_flags[idx as usize] &= !(CSF_ACTIVE as ::core::ffi::c_int);
        if stop as ::core::ffi::c_int != 0
            && searched_cond != CSF_TRY as ::core::ffi::c_int | CSF_SILENT as ::core::ffi::c_int
        {
            break;
        }
        if (*cstack).cs_flags[idx as usize] & CSF_TRY as ::core::ffi::c_int != 0
            && (*cstack).cs_flags[idx as usize] & CSF_SILENT as ::core::ffi::c_int != 0
        {
            let mut elem: *mut eslist_T = ::core::ptr::null_mut::<eslist_T>();
            elem = (*cstack).cs_emsg_silent_list;
            (*cstack).cs_emsg_silent_list = (*elem).next;
            emsg_silent.set((*elem).saved_emsg_silent);
            xfree(elem as *mut ::core::ffi::c_void);
            (*cstack).cs_flags[idx as usize] &= !(CSF_SILENT as ::core::ffi::c_int);
        }
        if stop {
            break;
        }
        idx -= 1;
    }
    return idx;
}
unsafe extern "C" fn get_end_emsg(mut cstack: *mut cstack_T) -> *mut ::core::ffi::c_char {
    if (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_WHILE as ::core::ffi::c_int != 0 {
        return gettext(&raw const e_endwhile as *const ::core::ffi::c_char);
    }
    if (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_FOR as ::core::ffi::c_int != 0 {
        return gettext(&raw const e_endfor as *const ::core::ffi::c_char);
    }
    return gettext(&raw const e_endif as *const ::core::ffi::c_char);
}
pub unsafe extern "C" fn rewind_conditionals(
    mut cstack: *mut cstack_T,
    mut idx: ::core::ffi::c_int,
    mut cond_type: ::core::ffi::c_int,
    mut cond_level: *mut ::core::ffi::c_int,
) {
    while (*cstack).cs_idx > idx {
        if (*cstack).cs_flags[(*cstack).cs_idx as usize] & cond_type != 0 {
            *cond_level -= 1;
        }
        if (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_FOR as ::core::ffi::c_int != 0 {
            free_for_info((*cstack).cs_forinfo[(*cstack).cs_idx as usize]);
        }
        (*cstack).cs_idx -= 1;
    }
}
pub unsafe fn ex_endfunction(mut _eap: *mut exarg_T) {
    semsg(
        gettext(&raw const e_str_not_inside_function as *const ::core::ffi::c_char),
        b":endfunction\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
pub unsafe extern "C" fn has_loop_cmd(mut p: *mut ::core::ffi::c_char) -> bool {
    loop {
        while *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == '\t' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        let mut len: ::core::ffi::c_int = modifier_len(p);
        if len == 0 as ::core::ffi::c_int {
            break;
        }
        p = p.offset(len as isize);
    }
    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'w' as ::core::ffi::c_int
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'h' as ::core::ffi::c_int
        || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'f' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'o' as ::core::ffi::c_int
            && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'r' as ::core::ffi::c_int
    {
        return true_0 != 0;
    }
    return false_0 != 0;
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
