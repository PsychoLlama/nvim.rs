//! Command-line completion inside an expression.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn set_context_for_expression(
    mut xp: *mut expand_T,
    mut arg: *mut c_char,
    mut cmdidx: cmdidx_T,
) {
    let mut got_eq: bool = false_0 != 0;
    if cmdidx as c_int == CMD_let as c_int || cmdidx as c_int == CMD_const as c_int {
        (*xp).xp_context = EXPAND_USER_VARS as c_int;
        if strpbrk(arg, b"\"'+-*/%.=!?~|&$([<>,#\0".as_ptr() as *const c_char).is_null() {
            let mut p: *mut c_char = arg.offset(strlen(arg) as isize);
            while p >= arg {
                (*xp).xp_pattern = p;
                p = p.offset(
                    -((utf_head_off(arg, p.offset(-(1 as c_int as isize))) + 1 as c_int) as isize),
                );
                if ascii_iswhite(*p as c_int) {
                    break;
                }
            }
            return;
        }
    } else {
        (*xp).xp_context = if cmdidx as c_int == CMD_call as c_int {
            EXPAND_FUNCTIONS as c_int
        } else {
            EXPAND_EXPRESSION as c_int
        };
    }
    loop {
        (*xp).xp_pattern = strpbrk(arg, b"\"'+-*/%.=!?~|&$([<>,#\0".as_ptr() as *const c_char);
        if (*xp).xp_pattern.is_null() {
            break;
        }
        let mut c: c_int = *(*xp).xp_pattern as uint8_t as c_int;
        if c == '&' as c_int {
            c = *(*xp).xp_pattern.offset(1 as c_int as isize) as uint8_t as c_int;
            if c == '&' as c_int {
                (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
                (*xp).xp_context = if cmdidx as c_int != CMD_let as c_int || got_eq as c_int != 0 {
                    EXPAND_EXPRESSION as c_int
                } else {
                    EXPAND_NOTHING as c_int
                };
            } else if c != ' ' as c_int {
                (*xp).xp_context = EXPAND_SETTINGS as c_int;
                if (c == 'l' as c_int || c == 'g' as c_int)
                    && *(*xp).xp_pattern.offset(2 as c_int as isize) as c_int == ':' as c_int
                {
                    (*xp).xp_pattern = (*xp).xp_pattern.offset(2 as c_int as isize);
                }
            }
        } else if c == '$' as c_int {
            (*xp).xp_context = EXPAND_ENV_VARS as c_int;
        } else if c == '=' as c_int {
            got_eq = true_0 != 0;
            (*xp).xp_context = EXPAND_EXPRESSION as c_int;
        } else {
            if c == '#' as c_int && (*xp).xp_context == EXPAND_EXPRESSION as c_int {
                break;
            }
            if (c == '<' as c_int || c == '#' as c_int)
                && (*xp).xp_context == EXPAND_FUNCTIONS as c_int
                && vim_strchr((*xp).xp_pattern, '(' as c_int).is_null()
            {
                break;
            }
            if cmdidx as c_int != CMD_let as c_int || got_eq as c_int != 0 {
                if c == '"' as c_int {
                    loop {
                        (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
                        c = *(*xp).xp_pattern as uint8_t as c_int;
                        if !(c != NUL && c != '"' as c_int) {
                            break;
                        }
                        if c == '\\' as c_int
                            && *(*xp).xp_pattern.offset(1 as c_int as isize) as c_int != NUL
                        {
                            (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
                        }
                    }
                    (*xp).xp_context = EXPAND_NOTHING as c_int;
                } else if c == '\'' as c_int {
                    loop {
                        (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
                        c = *(*xp).xp_pattern as uint8_t as c_int;
                        if !(c != NUL && c != '\'' as c_int) {
                            break;
                        }
                    }
                    (*xp).xp_context = EXPAND_NOTHING as c_int;
                } else if c == '|' as c_int {
                    if *(*xp).xp_pattern.offset(1 as c_int as isize) as c_int == '|' as c_int {
                        (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
                        (*xp).xp_context = EXPAND_EXPRESSION as c_int;
                    } else {
                        (*xp).xp_context = EXPAND_COMMANDS as c_int;
                    }
                } else {
                    (*xp).xp_context = EXPAND_EXPRESSION as c_int;
                }
            } else {
                (*xp).xp_context = EXPAND_EXPRESSION as c_int;
            }
        }
        arg = (*xp).xp_pattern;
        if *arg as c_int != NUL {
            loop {
                arg = arg.offset(1);
                c = *arg as uint8_t as c_int;
                if !(c != NUL && (c == ' ' as c_int || c == '\t' as c_int)) {
                    break;
                }
            }
        }
    }
    if cmd_has_expr_args(cmdidx) as c_int != 0 && (*xp).xp_context == EXPAND_EXPRESSION as c_int {
        loop {
            let n: *mut c_char = skiptowhite(arg);
            if n == arg || ascii_iswhite_or_nul(*skipwhite(n) as c_int) as c_int != 0 {
                break;
            }
            arg = skipwhite(n);
        }
    }
    (*xp).xp_pattern = arg;
}
