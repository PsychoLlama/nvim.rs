//! Resolving a command name to a row of `cmdnames`.
//!
//! `find_ex_command` is the hot path — `cmdidxs1`/`cmdidxs2` let it skip
//! straight to the first command with the right first and second letter — and it
//! carries all the special cases the table cannot express.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn checkforcmd(
    mut pp: *mut *mut c_char,
    mut cmd: *const c_char,
    mut len: c_int,
) -> bool {
    let mut i: c_int = 0;
    i = 0 as c_int;
    while *cmd.offset(i as isize) as c_int != NUL {
        if *cmd.offset(i as isize) as c_int != *(*pp).offset(i as isize) as c_int {
            break;
        }
        i += 1;
    }
    if i >= len
        && !(*(*pp).offset(i as isize) as c_uint >= 'A' as c_uint
            && *(*pp).offset(i as isize) as c_uint <= 'Z' as c_uint
            || *(*pp).offset(i as isize) as c_uint >= 'a' as c_uint
                && *(*pp).offset(i as isize) as c_uint <= 'z' as c_uint)
    {
        *pp = skipwhite((*pp).offset(i as isize));
        return true_0 != 0;
    }
    return false_0 != 0;
}

pub(crate) unsafe extern "C" fn one_letter_cmd(
    mut p: *const c_char,
    mut idx: *mut cmdidx_T,
) -> c_int {
    if *p.offset(0 as c_int as isize) as c_int == 'k' as c_int
        && (*p.offset(1 as c_int as isize) as c_int != 'e' as c_int
            || *p.offset(1 as c_int as isize) as c_int == 'e' as c_int
                && *p.offset(2 as c_int as isize) as c_int != 'e' as c_int)
    {
        *idx = CMD_k;
        return true_0;
    }
    if *p.offset(0 as c_int as isize) as c_int == 's' as c_int
        && (*p.offset(1 as c_int as isize) as c_int == 'c' as c_int
            && (*p.offset(2 as c_int as isize) as c_int == NUL
                || *p.offset(2 as c_int as isize) as c_int != 's' as c_int
                    && *p.offset(2 as c_int as isize) as c_int != 'r' as c_int
                    && (*p.offset(3 as c_int as isize) as c_int == NUL
                        || *p.offset(3 as c_int as isize) as c_int != 'i' as c_int
                            && *p.offset(4 as c_int as isize) as c_int != 'p' as c_int))
            || *p.offset(1 as c_int as isize) as c_int == 'g' as c_int
            || *p.offset(1 as c_int as isize) as c_int == 'i' as c_int
                && *p.offset(2 as c_int as isize) as c_int != 'm' as c_int
                && *p.offset(2 as c_int as isize) as c_int != 'l' as c_int
                && *p.offset(2 as c_int as isize) as c_int != 'g' as c_int
            || *p.offset(1 as c_int as isize) as c_int == 'I' as c_int
            || *p.offset(1 as c_int as isize) as c_int == 'r' as c_int
                && *p.offset(2 as c_int as isize) as c_int != 'e' as c_int)
    {
        *idx = CMD_substitute;
        return true_0;
    }
    return false_0;
}

pub unsafe extern "C" fn find_ex_command(
    mut eap: *mut exarg_T,
    mut full: *mut c_int,
) -> *mut c_char {
    let mut p: *mut c_char = (*eap).cmd;
    if one_letter_cmd(p, &raw mut (*eap).cmdidx) != 0 {
        p = p.offset(1);
        if !full.is_null() {
            *full = true_0;
        }
    } else {
        while *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
            || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
        {
            p = p.offset(1);
        }
        if *(*eap).cmd.offset(0 as c_int as isize) as c_int == 'p' as c_int
            && *(*eap).cmd.offset(1 as c_int as isize) as c_int == 'y' as c_int
        {
            while *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
                || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
                || ascii_isdigit(*p as c_int) as c_int != 0
            {
                p = p.offset(1);
            }
        }
        if p == (*eap).cmd
            && !vim_strchr(
                b"@!=><&~#\0".as_ptr() as *const c_char,
                *p as uint8_t as c_int,
            )
            .is_null()
        {
            p = p.offset(1);
        }
        let mut len: c_int = p.offset_from((*eap).cmd) as c_int;
        if *(*eap).cmd as c_int == 'd' as c_int
            && (*p.offset(-1 as c_int as isize) as c_int == 'l' as c_int
                || *p.offset(-1 as c_int as isize) as c_int == 'p' as c_int)
        {
            let mut i: c_int = 0;
            i = 0 as c_int;
            while i < len {
                if *(*eap).cmd.offset(i as isize) as c_int
                    != c_bytes(b"delete\0")[i as usize] as c_int
                {
                    break;
                }
                i += 1;
            }
            if i == len - 1 as c_int {
                len -= 1;
                if *p.offset(-1 as c_int as isize) as c_int == 'l' as c_int {
                    (*eap).flags |= EXFLAG_LIST;
                } else {
                    (*eap).flags |= EXFLAG_PRINT;
                }
            }
        }
        if *(*eap).cmd.offset(0 as c_int as isize) as c_uint >= 'a' as c_uint
            && *(*eap).cmd.offset(0 as c_int as isize) as c_uint <= 'z' as c_uint
        {
            let c1: c_int = *(*eap).cmd.offset(0 as c_int as isize) as uint8_t as c_int;
            let c2: c_int = if len == 1 as c_int {
                NUL
            } else {
                *(*eap).cmd.offset(1 as c_int as isize) as c_int
            };
            if command_count.get() != CMD_SIZE as c_int {
                iemsg(gettext(
                    b"E943: Command table needs to be updated, run 'make'\0".as_ptr()
                        as *const c_char,
                ));
                getout(1 as c_int);
            }
            (*eap).cmdidx =
                (*cmdidxs1.ptr())[(c1 as uint8_t as c_int - 'a' as c_int) as usize] as cmdidx_T;
            if c2 as c_uint >= 'a' as c_uint && c2 as c_uint <= 'z' as c_uint {
                (*eap).cmdidx = ((*eap).cmdidx as c_int
                    + (*cmdidxs2.ptr())[(c1 as uint8_t as c_int - 'a' as c_int) as usize]
                        [(c2 as uint8_t as c_int - 'a' as c_int) as usize]
                        as c_int) as cmdidx_T;
            }
        } else if *(*eap).cmd.offset(0 as c_int as isize) as c_uint >= 'A' as c_uint
            && *(*eap).cmd.offset(0 as c_int as isize) as c_uint <= 'Z' as c_uint
        {
            (*eap).cmdidx = CMD_Next;
        } else {
            (*eap).cmdidx = CMD_bang;
        }
        '_c2rust_label: {
            if (*eap).cmdidx as c_int >= 0 as c_int {
            } else {
                __assert_fail(
                    b"eap->cmdidx >= 0\0".as_ptr() as *const c_char,
                    b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                    3236 as c_uint,
                    b"char *find_ex_command(exarg_T *, int *)\0".as_ptr() as *const c_char,
                );
            }
        };
        if len == 3 as c_int
            && strncmp(b"def\0".as_ptr() as *const c_char, (*eap).cmd, 3 as size_t) == 0 as c_int
        {
            (*eap).cmdidx = CMD_SIZE;
        }
        while ((*eap).cmdidx as c_int) < CMD_SIZE as c_int {
            if strncmp(
                (*cmdnames.ptr())[(*eap).cmdidx as c_int as usize].cmd_name,
                (*eap).cmd,
                len as size_t,
            ) == 0 as c_int
            {
                if !full.is_null()
                    && *(*cmdnames.ptr())[(*eap).cmdidx as c_int as usize]
                        .cmd_name
                        .offset(len as isize) as c_int
                        == NUL
                {
                    *full = true_0;
                }
                break;
            } else {
                (*eap).cmdidx = ((*eap).cmdidx as c_int + 1 as c_int) as cmdidx_T;
            }
        }
        if (*eap).cmdidx as c_int == CMD_SIZE as c_int
            && *(*eap).cmd as c_int >= 'A' as c_int
            && *(*eap).cmd as c_int <= 'Z' as c_int
        {
            while *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
                || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
                || ascii_isdigit(*p as c_int) as c_int != 0
            {
                p = p.offset(1);
            }
            p = find_ucmd(
                eap,
                p,
                full,
                ::core::ptr::null_mut::<expand_T>(),
                ::core::ptr::null_mut::<c_int>(),
            );
        }
        if p == (*eap).cmd {
            (*eap).cmdidx = CMD_SIZE;
        }
    }
    return p;
}

pub unsafe extern "C" fn cmd_exists(name: *const c_char) -> c_int {
    for md in &CMDMODS {
        let j = shared_prefix(name, md.name);
        if *name.add(j) as c_int == NUL && j >= md.minlen {
            return if md.name.to_bytes().len() == j { 2 } else { 1 };
        }
    }
    let mut ea: exarg_T = exarg_T {
        arg: ::core::ptr::null_mut::<c_char>(),
        args: ::core::ptr::null_mut::<*mut c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<c_char>(),
        cmd: ::core::ptr::null_mut::<c_char>(),
        cmdlinep: ::core::ptr::null_mut::<*mut c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<c_char>(),
        cmdidx: CMD_append,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
        usefilter: 0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    ea.cmd = (if *name as c_int == '2' as c_int || *name as c_int == '3' as c_int {
        name.offset(1 as c_int as isize)
    } else {
        name
    }) as *mut c_char;
    ea.cmdidx = CMD_append;
    ea.flags = 0 as c_int;
    let mut full: c_int = false_0;
    let mut p: *mut c_char = find_ex_command(&raw mut ea, &raw mut full);
    if p.is_null() {
        return 3 as c_int;
    }
    if ascii_isdigit(*name as c_int) as c_int != 0 && ea.cmdidx as c_int != CMD_match as c_int {
        return 0 as c_int;
    }
    if *skipwhite(p) as c_int != NUL {
        return 0 as c_int;
    }
    return if ea.cmdidx as c_int == CMD_SIZE as c_int {
        0 as c_int
    } else if full != 0 {
        2 as c_int
    } else {
        1 as c_int
    };
}

pub unsafe extern "C" fn f_fullcommand(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut name: *mut c_char = tv_get_string(argvars.offset(0 as c_int as isize)) as *mut c_char;
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<c_char>();
    while *name as c_int == ':' as c_int {
        name = name.offset(1);
    }
    name = skip_range(name, ::core::ptr::null_mut::<c_int>());
    let mut ea: exarg_T = exarg_T {
        arg: ::core::ptr::null_mut::<c_char>(),
        args: ::core::ptr::null_mut::<*mut c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<c_char>(),
        cmd: ::core::ptr::null_mut::<c_char>(),
        cmdlinep: ::core::ptr::null_mut::<*mut c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<c_char>(),
        cmdidx: CMD_append,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
        usefilter: 0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    ea.cmd = if *name as c_int == '2' as c_int || *name as c_int == '3' as c_int {
        name.offset(1 as c_int as isize)
    } else {
        name
    };
    ea.cmdidx = CMD_append;
    ea.flags = 0 as c_int;
    let mut p: *mut c_char = find_ex_command(&raw mut ea, ::core::ptr::null_mut::<c_int>());
    if p.is_null() || ea.cmdidx as c_int == CMD_SIZE as c_int {
        return;
    }
    (*rettv).vval.v_string = xstrdup(if (ea.cmdidx as c_int) < 0 as c_int {
        get_user_command_name(ea.useridx, ea.cmdidx as c_int)
    } else {
        (*cmdnames.ptr())[ea.cmdidx as usize].cmd_name
    });
}

pub unsafe extern "C" fn excmd_get_cmdidx(mut cmd: *const c_char, mut len: size_t) -> cmdidx_T {
    if len == 3 as size_t
        && strncmp(b"def\0".as_ptr() as *const c_char, cmd, 3 as size_t) == 0 as c_int
    {
        return CMD_SIZE;
    }
    let mut idx: cmdidx_T = CMD_append;
    if one_letter_cmd(cmd, &raw mut idx) == 0 {
        idx = CMD_append;
        while (idx as c_int) < CMD_SIZE as c_int {
            if strncmp((*cmdnames.ptr())[idx as c_int as usize].cmd_name, cmd, len) == 0 as c_int {
                break;
            }
            idx = (idx as c_int + 1 as c_int) as cmdidx_T;
        }
    }
    return idx;
}

pub unsafe extern "C" fn excmd_get_argt(mut idx: cmdidx_T) -> uint32_t {
    return (*cmdnames.ptr())[idx as c_int as usize].cmd_argt;
}

pub unsafe extern "C" fn get_command_name(mut _xp: *mut expand_T, mut idx: c_int) -> *mut c_char {
    if idx >= CMD_SIZE as c_int {
        return expand_user_command_name(idx);
    }
    return (*cmdnames.ptr())[idx as usize].cmd_name;
}
