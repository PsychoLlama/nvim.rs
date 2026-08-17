//! `input()`, `inputsecret()` and the `:normal`-style script prompts.
//!
//! [`get_user_input`] is the shared implementation behind the `input*()`
//! family: it takes the prompt, default and completion out of the argument
//! (or the option dict), and drives a command line through
//! [`super::enter::getcmdline_prompt`].

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::{VAR_DICT, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED};

/// C's `NUMBUFLEN`: the size of the scratch buffer `tv_get_string_buf_chk`
/// and friends format a non-string value into.
const NUMBUFLEN: usize = 65;

/// Read the script body of a command that takes either `:command script` or a
/// heredoc:
///
/// ```text
/// :command << endmarker
///   script
/// endmarker
/// ```
///
/// `lenp` receives the length without the trailing NUL (zero while skipping).
/// Answers an allocated string, or NULL when skipping and on error; it shows
/// no messages of its own.
pub unsafe fn script_get(eap: *mut exarg_T, lenp: *mut size_t) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut cmd = (*eap).arg;
        if *cmd.offset(0) as ::core::ffi::c_int != '<' as ::core::ffi::c_int
            || *cmd.offset(1) as ::core::ffi::c_int != '<' as ::core::ffi::c_int
            || (*eap).ea_getline.is_none()
        {
            *lenp = strlen((*eap).arg);
            if (*eap).skip != 0 {
                return ::core::ptr::null_mut();
            }
            return xmemdupz((*eap).arg as *const ::core::ffi::c_void, *lenp)
                as *mut ::core::ffi::c_char;
        }
        cmd = cmd.offset(2);

        // C's `garray_T ga = { .ga_data = NULL, .ga_len = 0 }`: every other
        // field is left zeroed, growsize included, until `ga_init` below.
        let mut ga = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: NULL_0,
        };

        let l = heredoc_get(eap, cmd, true);
        if l.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }

        if (*eap).skip == 0 {
            ga_init(&raw mut ga, 1, 0x400);
        }

        let mut li: *const listitem_T = (*l).lv_first;
        while !li.is_null() {
            if (*eap).skip == 0 {
                ga_concat(&raw mut ga, tv_get_string(&raw const (*li).li_tv));
                ga_append(&raw mut ga, b'\n');
            }
            li = (*li).li_next;
        }

        // The length is the text without the terminator the next line adds.
        *lenp = ga.ga_len as size_t;
        if (*eap).skip == 0 {
            ga_append(&raw mut ga, NUL as uint8_t);
        }

        tv_list_free(l);
        ga.ga_data as *mut ::core::ffi::c_char
    }
}

/// Drive one `input()`-family prompt and leave its answer in `rettv`.
///
/// Shared by `input()`, `inputsecret()` and `inputdialog()`.  `argvars` is
/// either a single `{opts}` dict or up to three positional arguments, whose
/// third means completion for `input()` and the cancel value for
/// `inputdialog()`.
pub unsafe fn get_user_input(
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

        let prompt: *const ::core::ffi::c_char;
        let mut defstr: *const ::core::ffi::c_char = c"".as_ptr();
        let mut cancelreturn: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
        let mut cancelreturn_strarg2 = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut xp_name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut input_callback = Callback {
            data: C2Rust_Unnamed_5 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        };
        let mut prompt_buf: [::core::ffi::c_char; NUMBUFLEN] = [0; NUMBUFLEN];
        let mut defstr_buf: [::core::ffi::c_char; NUMBUFLEN] = [0; NUMBUFLEN];
        let mut cancelreturn_buf: [::core::ffi::c_char; NUMBUFLEN] = [0; NUMBUFLEN];
        let mut xp_name_buf: [::core::ffi::c_char; NUMBUFLEN] = [0; NUMBUFLEN];
        // Its *address* is the "argument absent" answer below, so it has to be
        // a distinct object from the `""` literal `defstr` starts as.
        let mut def: [::core::ffi::c_char; 1] = [0];

        if (*argvars.offset(0)).v_type == VAR_DICT {
            if (*argvars.offset(1)).v_type != VAR_UNKNOWN {
                emsg(gettext(c"E5050: {opts} must be the only argument".as_ptr()));
                return;
            }
            let dict = (*argvars.offset(0)).vval.v_dict;
            // C's `S_LEN(key)`: the key pointer and its length, spelled once.
            let dict_str = |key: &::core::ffi::CStr,
                            numbuf: *mut ::core::ffi::c_char,
                            def: *const ::core::ffi::c_char| {
                tv_dict_get_string_buf_chk(
                    dict,
                    key.as_ptr(),
                    key.count_bytes() as ptrdiff_t,
                    numbuf,
                    def,
                )
            };

            prompt = dict_str(c"prompt", prompt_buf.as_mut_ptr(), c"".as_ptr());
            if prompt.is_null() {
                return;
            }
            defstr = dict_str(c"default", defstr_buf.as_mut_ptr(), c"".as_ptr());
            if defstr.is_null() {
                return;
            }
            let cancelreturn_key = c"cancelreturn";
            let cancelreturn_di = tv_dict_find(
                dict,
                cancelreturn_key.as_ptr(),
                cancelreturn_key.count_bytes() as ptrdiff_t,
            );
            if !cancelreturn_di.is_null() {
                cancelreturn = &raw mut (*cancelreturn_di).di_tv;
            }
            xp_name = dict_str(c"completion", xp_name_buf.as_mut_ptr(), def.as_ptr());
            if xp_name.is_null() {
                // error
                return;
            }
            if xp_name == def.as_ptr() {
                // key absent: default to NULL
                xp_name = ::core::ptr::null::<::core::ffi::c_char>();
            }
            let highlight_key = c"highlight";
            if !tv_dict_get_callback(
                dict,
                highlight_key.as_ptr(),
                highlight_key.count_bytes() as ptrdiff_t,
                &raw mut input_callback,
            ) {
                return;
            }
        } else {
            prompt = tv_get_string_buf_chk(argvars.offset(0), prompt_buf.as_mut_ptr());
            if prompt.is_null() {
                return;
            }
            if (*argvars.offset(1)).v_type != VAR_UNKNOWN {
                defstr = tv_get_string_buf_chk(argvars.offset(1), defstr_buf.as_mut_ptr());
                if defstr.is_null() {
                    return;
                }
                if (*argvars.offset(2)).v_type != VAR_UNKNOWN {
                    let strarg2 =
                        tv_get_string_buf_chk(argvars.offset(2), cancelreturn_buf.as_mut_ptr());
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

        let mut xp_type = EXPAND_NOTHING;
        let mut xp_arg = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if !xp_name.is_null() {
            // input() with a third argument: completion
            let xp_namelen = strlen(xp_name) as ::core::ffi::c_int;
            let mut argt: uint32_t = 0;
            if parse_compl_arg(xp_name, xp_namelen, &mut xp_type, &mut argt, &mut xp_arg) == FAIL {
                return;
            }
        }

        // Only the part of the message after the last NL is the command
        // line's prompt, unless the command line is externalised.
        let mut p = prompt;
        if !ui_has(kUICmdline) {
            let lastnl = strrchr(prompt, '\n' as ::core::ffi::c_int);
            if !lastnl.is_null() {
                p = lastnl.offset(1);
                msg_start();
                msg_clr_eos();
                msg_puts_len(prompt, p.offset_from(prompt), get_echo_hl_id(), false);
                msg_didout.set(false);
                msg_starthere();
            }
        }
        cmdline_row.set(msg_row.get());

        stuffReadbuffSpec(defstr);

        let save_ex_normal_busy = ex_normal_busy.get();
        ex_normal_busy.set(0);
        (*rettv).vval.v_string = getcmdline_prompt(
            if secret {
                NUL
            } else {
                '@' as ::core::ffi::c_int
            },
            p,
            get_echo_hl_id(),
            xp_type,
            xp_arg,
            input_callback,
            false,
            ::core::ptr::null_mut::<bool>(),
        );
        ex_normal_busy.set(save_ex_normal_busy);
        callback_free(&raw mut input_callback);

        if (*rettv).vval.v_string.is_null() && !cancelreturn.is_null() {
            tv_copy(cancelreturn, rettv);
        }

        xfree(xp_arg as *mut ::core::ffi::c_void);
        // Since the user typed this, no need to wait for return.
        need_wait_return.set(false);
        msg_didout.set(false);
    }
}
