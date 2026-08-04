//! `input()`, `inputsecret()` and the `:normal`-style script prompts.
//!
//! [`get_user_input`] is the shared implementation behind the `input*()`
//! family: it takes the prompt, default and completion out of the argument
//! (or the option dict), and drives a command line through
//! [`super::enter::getcmdline_prompt`].

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn script_get(
    eap: *mut exarg_T,
    lenp: *mut size_t,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut cmd: *mut ::core::ffi::c_char = (*eap).arg;
        if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != '<' as ::core::ffi::c_int
            || *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '<' as ::core::ffi::c_int
            || (*eap).ea_getline.is_none()
        {
            *lenp = strlen((*eap).arg);
            return (if (*eap).skip != 0 {
                NULL_0
            } else {
                xmemdupz((*eap).arg as *const ::core::ffi::c_void, *lenp)
            }) as *mut ::core::ffi::c_char;
        }
        cmd = cmd.offset(2 as ::core::ffi::c_int as isize);
        let mut ga: garray_T = garray_T {
            ga_len: 0 as ::core::ffi::c_int,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: NULL_0,
        };
        let l: *mut list_T = heredoc_get(eap, cmd, true_0 != 0);
        if l.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if (*eap).skip == 0 {
            ga_init(
                &raw mut ga,
                1 as ::core::ffi::c_int,
                0x400 as ::core::ffi::c_int,
            );
        }
        let l_: *const list_T = l;
        if !l_.is_null() {
            let mut li: *const listitem_T = (*l_).lv_first;
            while !li.is_null() {
                if (*eap).skip == 0 {
                    ga_concat(&raw mut ga, tv_get_string(&raw const (*li).li_tv));
                    ga_append(&raw mut ga, '\n' as uint8_t);
                }
                li = (*li).li_next;
            }
        }
        *lenp = ga.ga_len as size_t;
        if (*eap).skip == 0 {
            ga_append(&raw mut ga, NUL as uint8_t);
        }
        tv_list_free(l);
        return ga.ga_data as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn get_user_input(
    argvars: *const typval_T,
    rettv: *mut typval_T,
    inputdialog: bool,
    secret: bool,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if cmdpreview.get() {
            return;
        }
        let mut prompt: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut defstr: *const ::core::ffi::c_char = b"\0".as_ptr() as *const ::core::ffi::c_char;
        let mut cancelreturn: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
        let mut cancelreturn_strarg2: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut xp_name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut input_callback: Callback = Callback {
            data: C2Rust_Unnamed_5 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        };
        let mut prompt_buf: [::core::ffi::c_char; 65] = [0; 65];
        let mut defstr_buf: [::core::ffi::c_char; 65] = [0; 65];
        let mut cancelreturn_buf: [::core::ffi::c_char; 65] = [0; 65];
        let mut xp_name_buf: [::core::ffi::c_char; 65] = [0; 65];
        let mut def: [::core::ffi::c_char; 1] = [0 as ::core::ffi::c_char];
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                emsg(gettext(
                    b"E5050: {opts} must be the only argument\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                return;
            }
            let dict: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_dict;
            prompt = tv_dict_get_string_buf_chk(
                dict,
                b"prompt\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
                &raw mut prompt_buf as *mut ::core::ffi::c_char,
                b"\0".as_ptr() as *const ::core::ffi::c_char,
            );
            if prompt.is_null() {
                return;
            }
            defstr = tv_dict_get_string_buf_chk(
                dict,
                b"default\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
                &raw mut defstr_buf as *mut ::core::ffi::c_char,
                b"\0".as_ptr() as *const ::core::ffi::c_char,
            );
            if defstr.is_null() {
                return;
            }
            let mut cancelreturn_di: *mut dictitem_T = tv_dict_find(
                dict,
                b"cancelreturn\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 13]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            if !cancelreturn_di.is_null() {
                cancelreturn = &raw mut (*cancelreturn_di).di_tv;
            }
            xp_name = tv_dict_get_string_buf_chk(
                dict,
                b"completion\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
                &raw mut xp_name_buf as *mut ::core::ffi::c_char,
                &raw mut def as *mut ::core::ffi::c_char,
            );
            if xp_name.is_null() {
                return;
            }
            if xp_name == &raw mut def as *mut ::core::ffi::c_char as *const ::core::ffi::c_char {
                xp_name = ::core::ptr::null::<::core::ffi::c_char>();
            }
            if !tv_dict_get_callback(
                dict,
                b"highlight\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
                &raw mut input_callback,
            ) {
                return;
            }
        } else {
            prompt = tv_get_string_buf_chk(
                argvars.offset(0 as ::core::ffi::c_int as isize),
                &raw mut prompt_buf as *mut ::core::ffi::c_char,
            );
            if prompt.is_null() {
                return;
            }
            if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                defstr = tv_get_string_buf_chk(
                    argvars.offset(1 as ::core::ffi::c_int as isize),
                    &raw mut defstr_buf as *mut ::core::ffi::c_char,
                );
                if defstr.is_null() {
                    return;
                }
                if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let strarg2: *const ::core::ffi::c_char = tv_get_string_buf_chk(
                        argvars.offset(2 as ::core::ffi::c_int as isize),
                        &raw mut cancelreturn_buf as *mut ::core::ffi::c_char,
                    );
                    if strarg2.is_null() {
                        return;
                    }
                    if inputdialog {
                        cancelreturn_strarg2.v_type = VAR_STRING;
                        cancelreturn_strarg2.vval.v_string = strarg2 as *mut ::core::ffi::c_char;
                        cancelreturn = &raw mut cancelreturn_strarg2;
                    } else {
                        xp_name = strarg2;
                    }
                }
            }
        }
        let mut xp_type: ::core::ffi::c_int = EXPAND_NOTHING as ::core::ffi::c_int;
        let mut xp_arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if !xp_name.is_null() {
            let xp_namelen: ::core::ffi::c_int = strlen(xp_name) as ::core::ffi::c_int;
            let mut argt: uint32_t = 0 as uint32_t;
            if parse_compl_arg(
                xp_name,
                xp_namelen,
                &raw mut xp_type,
                &raw mut argt,
                &raw mut xp_arg,
            ) == FAIL
            {
                return;
            }
        }
        let mut p: *const ::core::ffi::c_char = prompt;
        if !ui_has(kUICmdline) {
            let mut lastnl: *const ::core::ffi::c_char =
                strrchr(prompt, '\n' as ::core::ffi::c_int);
            if !lastnl.is_null() {
                p = lastnl.offset(1 as ::core::ffi::c_int as isize);
                msg_start();
                msg_clr_eos();
                msg_puts_len(
                    prompt,
                    p.offset_from(prompt),
                    get_echo_hl_id(),
                    false_0 != 0,
                );
                msg_didout.set(false_0 != 0);
                msg_starthere();
            }
        }
        cmdline_row.set(msg_row.get());
        stuffReadbuffSpec(defstr);
        let save_ex_normal_busy: ::core::ffi::c_int = ex_normal_busy.get();
        ex_normal_busy.set(0 as ::core::ffi::c_int);
        (*rettv).vval.v_string = getcmdline_prompt(
            if secret as ::core::ffi::c_int != 0 {
                NUL
            } else {
                '@' as ::core::ffi::c_int
            },
            p,
            get_echo_hl_id(),
            xp_type,
            xp_arg,
            input_callback,
            false_0 != 0,
            ::core::ptr::null_mut::<bool>(),
        );
        ex_normal_busy.set(save_ex_normal_busy);
        callback_free(&raw mut input_callback);
        if (*rettv).vval.v_string.is_null() && !cancelreturn.is_null() {
            tv_copy(cancelreturn, rettv);
        }
        xfree(xp_arg as *mut ::core::ffi::c_void);
        need_wait_return.set(false_0 != 0);
        msg_didout.set(false_0 != 0);
    }
}
