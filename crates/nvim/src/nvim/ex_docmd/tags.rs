//! `:tag` and the identifier searches that share its argument
//! handling.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ex_findpat(mut eap: *mut exarg_T) {
    let mut whole: bool = true_0 != 0;
    let mut action: c_int = 0;
    match *(*cmdnames.ptr())[(*eap).cmdidx as usize]
        .cmd_name
        .offset(2 as c_int as isize) as c_int
    {
        101 => {
            if *(*cmdnames.ptr())[(*eap).cmdidx as usize]
                .cmd_name
                .offset(0 as c_int as isize) as c_int
                == 'p' as c_int
            {
                action = ACTION_GOTO as c_int;
            } else {
                action = ACTION_SHOW as c_int;
            }
        }
        105 => {
            action = ACTION_SHOW_ALL as c_int;
        }
        117 => {
            action = ACTION_GOTO as c_int;
        }
        _ => {
            action = ACTION_SPLIT as c_int;
        }
    }
    let mut n: c_int = 1 as c_int;
    if ascii_isdigit(*(*eap).arg as c_int) {
        n = getdigits_int(&raw mut (*eap).arg, false_0 != 0, 0 as c_int);
        (*eap).arg = skipwhite((*eap).arg);
    }
    if *(*eap).arg as c_int == '/' as c_int {
        whole = false_0 != 0;
        (*eap).arg = (*eap).arg.offset(1);
        let mut p: *mut c_char = skip_regexp((*eap).arg, '/' as c_int, magic_isset() as c_int);
        if *p != 0 {
            let c2rust_fresh16 = p;
            p = p.offset(1);
            *c2rust_fresh16 = NUL as c_char;
            p = skipwhite(p);
            if ends_excmd(*p as c_int) == 0 {
                (*eap).errmsg = ex_errmsg(&raw const e_trailing_arg as *const c_char, p);
            } else {
                (*eap).nextcmd = check_nextcmd(p);
            }
        }
    }
    if (*eap).skip == 0 {
        find_pattern_in_path(
            (*eap).arg,
            kDirectionNotSet,
            strlen((*eap).arg),
            whole,
            (*eap).forceit == 0,
            if *(*eap).cmd as c_int == 'd' as c_int {
                FIND_DEFINE as c_int
            } else {
                FIND_ANY as c_int
            },
            n,
            action,
            (*eap).line1,
            (*eap).line2,
            (*eap).forceit != 0,
            false_0 != 0,
        );
    }
}

pub(crate) unsafe extern "C" fn ex_ptag(mut eap: *mut exarg_T) {
    g_do_tagpreview.set(p_pvh.get() as c_int);
    ex_tag_cmd(
        eap,
        (*cmdnames.ptr())[(*eap).cmdidx as usize]
            .cmd_name
            .offset(1 as c_int as isize),
    );
}

pub(crate) unsafe extern "C" fn ex_stag(mut eap: *mut exarg_T) {
    postponed_split.set(-1 as c_int);
    postponed_split_flags.set((*cmdmod.ptr()).cmod_split);
    postponed_split_tab.set((*cmdmod.ptr()).cmod_tab);
    ex_tag_cmd(
        eap,
        (*cmdnames.ptr())[(*eap).cmdidx as usize]
            .cmd_name
            .offset(1 as c_int as isize),
    );
    postponed_split_flags.set(0 as c_int);
    postponed_split_tab.set(0 as c_int);
}

pub(crate) unsafe extern "C" fn ex_tag(mut eap: *mut exarg_T) {
    ex_tag_cmd(eap, (*cmdnames.ptr())[(*eap).cmdidx as usize].cmd_name);
}

pub(crate) unsafe extern "C" fn ex_tag_cmd(mut eap: *mut exarg_T, mut name: *const c_char) {
    let mut cmd: c_int = 0;
    match *name.offset(1 as c_int as isize) as c_int {
        106 => {
            cmd = DT_JUMP as c_int;
        }
        115 => {
            cmd = DT_SELECT as c_int;
        }
        112 | 78 => {
            cmd = DT_PREV as c_int;
        }
        110 => {
            cmd = DT_NEXT as c_int;
        }
        111 => {
            cmd = DT_POP as c_int;
        }
        102 | 114 => {
            cmd = DT_FIRST as c_int;
        }
        108 => {
            cmd = DT_LAST as c_int;
        }
        _ => {
            cmd = DT_TAG as c_int;
        }
    }
    if *name.offset(0 as c_int as isize) as c_int == 'l' as c_int {
        cmd = DT_LTAG as c_int;
    }
    do_tag(
        (*eap).arg,
        cmd,
        if (*eap).addr_count > 0 as c_int {
            (*eap).line2 as c_int
        } else {
            1 as c_int
        },
        (*eap).forceit,
        true_0 != 0,
    );
}
