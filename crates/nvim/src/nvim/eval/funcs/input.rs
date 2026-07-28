//! Asking the user: `input()`, `confirm()`, the prompt-buffer
//! accessors and `feedkeys()`.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub unsafe extern "C" fn f_confirm(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
    let mut buttons: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut def: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut type_0: ::core::ffi::c_int = VIM_GENERIC as ::core::ffi::c_int;
    let mut error: bool = false_0 != 0;
    let mut message: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if message.is_null() {
        error = true_0 != 0;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        buttons = tv_get_string_buf_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if buttons.is_null() {
            error = true_0 != 0;
        }
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            def = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
            if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut typestr: *const ::core::ffi::c_char = tv_get_string_buf_chk(
                    argvars.offset(3 as ::core::ffi::c_int as isize),
                    &raw mut buf2 as *mut ::core::ffi::c_char,
                );
                if typestr.is_null() {
                    error = true_0 != 0;
                } else {
                    match if (*typestr as ::core::ffi::c_int) < 'a' as ::core::ffi::c_int
                        || *typestr as ::core::ffi::c_int > 'z' as ::core::ffi::c_int
                    {
                        *typestr as ::core::ffi::c_int
                    } else {
                        *typestr as ::core::ffi::c_int
                            - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                    } {
                        69 => {
                            type_0 = VIM_ERROR as ::core::ffi::c_int;
                        }
                        81 => {
                            type_0 = VIM_QUESTION as ::core::ffi::c_int;
                        }
                        73 => {
                            type_0 = VIM_INFO as ::core::ffi::c_int;
                        }
                        87 => {
                            type_0 = VIM_WARNING as ::core::ffi::c_int;
                        }
                        71 => {
                            type_0 = VIM_GENERIC as ::core::ffi::c_int;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    if buttons.is_null() || *buttons as ::core::ffi::c_int == NUL {
        buttons = gettext(b"&Ok\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if !error {
        (*rettv).vval.v_number = do_dialog(
            type_0,
            ::core::ptr::null::<::core::ffi::c_char>(),
            message,
            buttons,
            def,
            ::core::ptr::null::<::core::ffi::c_char>(),
            false_0,
        ) as varnumber_T;
    }
}
pub unsafe extern "C" fn f_debugbreak(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = FAIL as varnumber_T;
    let mut pid: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    if pid == 0 as ::core::ffi::c_int {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    uv_kill(pid, SIGINT);
}
pub unsafe extern "C" fn f_feedkeys(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if check_secure() {
        return;
    }
    let keys: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut nbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut flags: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        flags = tv_get_string_buf(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut nbuf as *mut ::core::ffi::c_char,
        );
    }
    nvim_feedkeys(cstr_as_string(keys), cstr_as_string(flags), true_0 != 0);
}
static inputsecret_flag: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub unsafe extern "C" fn f_input(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_user_input(argvars, rettv, false_0 != 0, inputsecret_flag.get());
}
pub unsafe extern "C" fn f_inputdialog(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_user_input(argvars, rettv, true_0 != 0, inputsecret_flag.get());
}
pub unsafe extern "C" fn f_inputlist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_listarg as *const ::core::ffi::c_char),
            b"inputlist()\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    msg_ext_set_kind(b"confirm\0".as_ptr() as *const ::core::ffi::c_char);
    msg_start();
    msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
    lines_left.set(Rows.get());
    msg_scroll.set(true_0);
    msg_clr_eos();
    let mut l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    let l_: *const list_T = l;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            msg_puts(tv_get_string(&raw const (*li).li_tv));
            if !ui_has(kUIMessages) || !(*li).li_next.is_null() {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            li = (*li).li_next;
        }
    }
    let mut mouse_used: bool = false_0 != 0;
    let mut selected: ::core::ffi::c_int = prompt_for_input(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
        false_0 != 0,
        &raw mut mouse_used,
    );
    if mouse_used {
        selected = tv_list_len(l) - (cmdline_row.get() - mouse_row.get());
    }
    (*rettv).vval.v_number = selected as varnumber_T;
}
static ga_userinput: GlobalCell<garray_T> = GlobalCell::new(garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: ::core::mem::size_of::<tasave_T>() as ::core::ffi::c_int,
    ga_growsize: 4 as ::core::ffi::c_int,
    ga_data: NULL_0,
});
pub unsafe extern "C" fn f_inputrestore(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if !((*ga_userinput.ptr()).ga_len <= 0 as ::core::ffi::c_int) {
        (*ga_userinput.ptr()).ga_len -= 1;
        restore_typeahead(
            ((*ga_userinput.ptr()).ga_data as *mut tasave_T)
                .offset((*ga_userinput.ptr()).ga_len as isize),
        );
    } else if p_verbose.get() > 1 as OptInt {
        verb_msg(gettext(
            b"called inputrestore() more often than inputsave()\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        (*rettv).vval.v_number = 1 as varnumber_T;
    }
}
pub unsafe extern "C" fn f_inputsave(
    mut _argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut p: *mut tasave_T =
        ga_append_via_ptr(ga_userinput.ptr(), ::core::mem::size_of::<tasave_T>()) as *mut tasave_T;
    save_typeahead(p);
}
pub unsafe extern "C" fn f_inputsecret(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    (*cmdline_star.ptr()) += 1;
    inputsecret_flag.set(true_0 != 0);
    f_input(argvars, rettv, fptr);
    (*cmdline_star.ptr()) -= 1;
    inputsecret_flag.set(false_0 != 0);
}
pub unsafe extern "C" fn f_interrupt(
    mut _argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    got_int.set(true_0 != 0);
}
pub unsafe extern "C" fn f_prompt_getprompt(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let buf: *mut buf_T = tv_get_buf_from_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
    if buf.is_null() {
        return;
    }
    if !bt_prompt(buf) {
        return;
    }
    (*rettv).vval.v_string = xstrdup(buf_prompt_text(buf));
}
pub unsafe extern "C" fn f_prompt_getinput(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let buf: *mut buf_T = tv_get_buf_from_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
    if buf.is_null() {
        return;
    }
    if !bt_prompt(buf) {
        return;
    }
    (*rettv).vval.v_string = prompt_get_input(buf);
}
