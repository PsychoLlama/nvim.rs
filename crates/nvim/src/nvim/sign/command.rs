//! The `:sign` Ex command.
//!
//! [`ex_sign`] picks a subcommand out of `cmds`, [`parse_sign_cmd_args`]
//! turns the rest of the line into the `line=`/`name=`/`group=`/
//! `priority=`/`file=`/`buffer=` tuple the three placement subcommands
//! share, and the `sign_*_cmd` functions diagnose the combinations that do
//! not make sense before handing off to the placement primitives in the
//! parent. [`sign_list_placed`] and [`sign_list_defined`] are the
//! `:sign place` / `:sign list` reports.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn sign_list_placed(
    mut rbuf: *mut buf_T,
    mut group: *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut lbuf: [::core::ffi::c_char; 480] = [0; 480];
        let mut namebuf: [::core::ffi::c_char; 480] = [0; 480];
        let mut groupbuf: [::core::ffi::c_char; 480] = [0; 480];
        let mut buf: *mut buf_T = if !rbuf.is_null() {
            rbuf
        } else {
            firstbuf.get()
        };
        let mut ns: int64_t = group_get_ns(group);
        msg_puts_title(gettext(
            b"\n--- Signs ---\0".as_ptr() as *const ::core::ffi::c_char
        ));
        while !buf.is_null() && !got_int.get() {
            if buf_has_signs(buf) {
                msg_putchar('\n' as ::core::ffi::c_int);
                vim_snprintf(
                    &raw mut lbuf as *mut ::core::ffi::c_char,
                    MSG_BUF_LEN as size_t,
                    gettext(b"Signs for %s:\0".as_ptr() as *const ::core::ffi::c_char),
                    (*buf).b_fname,
                );
                msg_puts_hl(&raw mut lbuf as *mut ::core::ffi::c_char, HLF_D, false);
            }
            if ns >= 0 as int64_t {
                let mut itr = MarkTreeIter::default();
                let mut signs: Vec<MTKey> = Vec::new();
                marktree_itr_get(
                    &raw mut (*buf).b_marktree as *mut MarkTree,
                    0,
                    0 as ::core::ffi::c_int,
                    &raw mut itr,
                );
                while !itr.x.is_null() {
                    let mut mark: MTKey = marktree_itr_current(&raw mut itr);
                    if !mt_end(mark)
                        && mt_decor_sign(mark) as ::core::ffi::c_int != 0
                        && (ns == ALL_GROUPS || ns == mark.ns as int64_t)
                    {
                        signs.push(mark);
                    }
                    marktree_itr_next(&raw mut (*buf).b_marktree as *mut MarkTree, &raw mut itr);
                }
                if !signs.is_empty() {
                    sort_signs(&mut signs);
                    let signs_size = signs.len() as size_t;
                    msg_putchar('\n' as ::core::ffi::c_int);
                    let mut i: size_t = 0 as size_t;
                    while i < signs_size {
                        namebuf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                        groupbuf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                        let mut mark_0: MTKey = signs[i as usize];
                        let mut sh: *mut DecorSignHighlight = decor_find_sign(mt_decor(mark_0));
                        if !(*sh).sign_name.is_null() {
                            vim_snprintf(
                                &raw mut namebuf as *mut ::core::ffi::c_char,
                                MSG_BUF_LEN as size_t,
                                gettext(b"  name=%s\0".as_ptr() as *const ::core::ffi::c_char),
                                sign_get_name(sh),
                            );
                        }
                        if mark_0.ns != 0 as uint32_t {
                            vim_snprintf(
                                &raw mut groupbuf as *mut ::core::ffi::c_char,
                                MSG_BUF_LEN as size_t,
                                gettext(b"  group=%s\0".as_ptr() as *const ::core::ffi::c_char),
                                describe_ns(
                                    mark_0.ns as NS,
                                    b"\0".as_ptr() as *const ::core::ffi::c_char,
                                ),
                            );
                        }
                        vim_snprintf(
                            &raw mut lbuf as *mut ::core::ffi::c_char,
                            MSG_BUF_LEN as size_t,
                            gettext(b"    line=%d  id=%u%s%s  priority=%d\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            mark_0.pos.row + 1,
                            mark_0.id,
                            &raw mut groupbuf as *mut ::core::ffi::c_char,
                            &raw mut namebuf as *mut ::core::ffi::c_char,
                            (*sh).priority as ::core::ffi::c_int,
                        );
                        msg_puts(&raw mut lbuf as *mut ::core::ffi::c_char);
                        if i < signs_size.wrapping_sub(1 as size_t) {
                            msg_putchar('\n' as ::core::ffi::c_int);
                        }
                        i = i.wrapping_add(1);
                    }
                }
            }
            if !rbuf.is_null() {
                return;
            }
            buf = (*buf).b_next;
        }
    }
}

pub(crate) unsafe extern "C" fn sign_cmd_idx(
    mut begin_cmd: *mut ::core::ffi::c_char,
    mut end_cmd: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let save: ::core::ffi::c_char = *end_cmd;
        *end_cmd = NUL as ::core::ffi::c_char;
        let idx = CMDS
            .iter()
            .position(|cmd| strcmp(begin_cmd, cmd.as_ptr()) == 0)
            .map_or(SIGNCMD_LAST, |i| i as ::core::ffi::c_int);
        *end_cmd = save;
        return idx;
    }
}

pub(crate) unsafe extern "C" fn sign_list_defined(mut sp: *mut sign_T) {
    unsafe {
        smsg(
            0 as ::core::ffi::c_int,
            b"sign %s\0".as_ptr() as *const ::core::ffi::c_char,
            (*sp).sn_name,
        );
        if !(*sp).sn_icon.is_null() {
            msg_puts(b" icon=\0".as_ptr() as *const ::core::ffi::c_char);
            msg_outtrans((*sp).sn_icon, 0 as ::core::ffi::c_int, false);
            msg_puts(gettext(
                b" (not supported)\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        if (*sp).sn_text[0 as ::core::ffi::c_int as usize] != 0 {
            msg_puts(b" text=\0".as_ptr() as *const ::core::ffi::c_char);
            let mut buf: [::core::ffi::c_char; 64] = [0; 64];
            describe_sign_text(
                &raw mut buf as *mut ::core::ffi::c_char,
                &raw mut (*sp).sn_text as *mut schar_T,
            );
            msg_outtrans(
                &raw mut buf as *mut ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
                false,
            );
        }
        if (*sp).sn_priority > 0 as ::core::ffi::c_int {
            let mut lbuf: [::core::ffi::c_char; 480] = [0; 480];
            vim_snprintf(
                &raw mut lbuf as *mut ::core::ffi::c_char,
                MSG_BUF_LEN as size_t,
                b" priority=%d\0".as_ptr() as *const ::core::ffi::c_char,
                (*sp).sn_priority,
            );
            msg_puts(&raw mut lbuf as *mut ::core::ffi::c_char);
        }
        static arg: GlobalCell<[*mut ::core::ffi::c_char; 4]> = GlobalCell::new([
            b" linehl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b" texthl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b" culhl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b" numhl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ]);
        let mut hl: [::core::ffi::c_int; 4] = [
            (*sp).sn_line_hl,
            (*sp).sn_text_hl,
            (*sp).sn_cul_hl,
            (*sp).sn_num_hl,
        ];
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 4 as ::core::ffi::c_int {
            if hl[i as usize] > 0 as ::core::ffi::c_int {
                msg_puts((*arg.ptr())[i as usize]);
                let mut p: *const ::core::ffi::c_char = get_highlight_name_ext(
                    ::core::ptr::null_mut::<expand_T>(),
                    hl[i as usize] - 1 as ::core::ffi::c_int,
                    false,
                );
                msg_puts(if !p.is_null() {
                    p
                } else {
                    b"NONE\0".as_ptr() as *const ::core::ffi::c_char
                });
            }
            i += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn sign_list_by_name(mut name: *mut ::core::ffi::c_char) {
    unsafe {
        let sp: *mut sign_T = sign_find(name);
        if !sp.is_null() {
            sign_list_defined(sp);
        } else {
            semsg(
                gettext(b"E155: Unknown sign: %s\0".as_ptr() as *const ::core::ffi::c_char),
                name,
            );
        };
    }
}

pub(crate) unsafe extern "C" fn sign_define_cmd(
    mut name: *mut ::core::ffi::c_char,
    mut cmdline: *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut icon: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut linehl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut texthl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut culhl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut numhl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut prio: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        loop {
            let mut arg: *mut ::core::ffi::c_char = skipwhite(cmdline);
            if *arg as ::core::ffi::c_int == NUL {
                break;
            }
            cmdline = skiptowhite_esc(arg);
            if strncmp(
                arg,
                b"icon=\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                icon = arg.offset(5 as ::core::ffi::c_int as isize);
            } else if strncmp(
                arg,
                b"text=\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                text = arg.offset(5 as ::core::ffi::c_int as isize);
            } else if strncmp(
                arg,
                b"linehl=\0".as_ptr() as *const ::core::ffi::c_char,
                7 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                linehl = arg.offset(7 as ::core::ffi::c_int as isize);
            } else if strncmp(
                arg,
                b"texthl=\0".as_ptr() as *const ::core::ffi::c_char,
                7 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                texthl = arg.offset(7 as ::core::ffi::c_int as isize);
            } else if strncmp(
                arg,
                b"culhl=\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                culhl = arg.offset(6 as ::core::ffi::c_int as isize);
            } else if strncmp(
                arg,
                b"numhl=\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                numhl = arg.offset(6 as ::core::ffi::c_int as isize);
            } else if strncmp(
                arg,
                b"priority=\0".as_ptr() as *const ::core::ffi::c_char,
                9 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                prio = atoi(arg.offset(9 as ::core::ffi::c_int as isize));
            } else {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    arg,
                );
                return;
            }
            if *cmdline as ::core::ffi::c_int == NUL {
                break;
            }
            let c2rust_fresh7 = cmdline;
            cmdline = cmdline.offset(1);
            *c2rust_fresh7 = NUL as ::core::ffi::c_char;
        }
        sign_define_by_name(name, icon, text, linehl, texthl, culhl, numhl, prio);
    }
}

pub(crate) unsafe extern "C" fn sign_place_cmd(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut name: *mut ::core::ffi::c_char,
    mut id: ::core::ffi::c_int,
    mut group: *mut ::core::ffi::c_char,
    mut prio: ::core::ffi::c_int,
) {
    unsafe {
        if id <= 0 as ::core::ffi::c_int {
            if lnum >= 0 as linenr_T
                || !name.is_null()
                || !group.is_null() && *group as ::core::ffi::c_int == NUL
            {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            } else {
                sign_list_placed(buf, group);
            }
        } else {
            if name.is_null()
                || buf.is_null()
                || !group.is_null() && *group as ::core::ffi::c_int == NUL
            {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return;
            }
            let mut uid: uint32_t = id as uint32_t;
            sign_place(&raw mut uid, group, name, buf, lnum, prio);
        };
    }
}

pub(crate) unsafe extern "C" fn sign_unplace_cmd(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut name: *const ::core::ffi::c_char,
    mut id: ::core::ffi::c_int,
    mut group: *mut ::core::ffi::c_char,
) {
    unsafe {
        if lnum >= 0 as linenr_T
            || !name.is_null()
            || !group.is_null() && *group as ::core::ffi::c_int == NUL
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        if id == -1 as ::core::ffi::c_int {
            lnum = (*curwin.get()).w_cursor.lnum;
            buf = (*curwin.get()).w_buffer;
        }
        if sign_unplace(
            buf,
            if 0 as ::core::ffi::c_int > id {
                0 as ::core::ffi::c_int
            } else {
                id
            },
            group,
            lnum,
        ) == 0
            && lnum > 0 as linenr_T
        {
            emsg(gettext(
                b"E159: Missing sign number\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
    }
}

pub(crate) unsafe extern "C" fn sign_jump_cmd(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut name: *const ::core::ffi::c_char,
    mut id: ::core::ffi::c_int,
    mut group: *mut ::core::ffi::c_char,
) {
    unsafe {
        if name.is_null() && group.is_null() && id == -1 as ::core::ffi::c_int {
            emsg(gettext(&raw const e_argreq as *const ::core::ffi::c_char));
            return;
        }
        if buf.is_null()
            || !group.is_null() && *group as ::core::ffi::c_int == NUL
            || lnum >= 0 as linenr_T
            || !name.is_null()
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        sign_jump(id, group, buf);
    }
}

pub(crate) unsafe extern "C" fn parse_sign_cmd_args(
    mut cmd: ::core::ffi::c_int,
    mut arg: *mut ::core::ffi::c_char,
    mut name: *mut *mut ::core::ffi::c_char,
    mut id: *mut ::core::ffi::c_int,
    mut group: *mut *mut ::core::ffi::c_char,
    mut prio: *mut ::core::ffi::c_int,
    mut buf: *mut *mut buf_T,
    mut lnum: *mut linenr_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut arg1: *mut ::core::ffi::c_char = arg;
        let mut filename: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut lnum_arg: bool = false;
        if ascii_isdigit(*arg as ::core::ffi::c_int) {
            *id = getdigits_int(&raw mut arg, true, 0 as ::core::ffi::c_int);
            if !ascii_iswhite(*arg as ::core::ffi::c_int) && *arg as ::core::ffi::c_int != NUL {
                *id = -1 as ::core::ffi::c_int;
                arg = arg1;
            } else {
                arg = skipwhite(arg);
            }
        }
        while *arg as ::core::ffi::c_int != NUL {
            if strncmp(
                arg,
                b"line=\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                arg = arg.offset(5 as ::core::ffi::c_int as isize);
                *lnum = atoi(arg) as linenr_T;
                arg = skiptowhite(arg);
                lnum_arg = true;
            } else if strncmp(
                arg,
                b"*\0".as_ptr() as *const ::core::ffi::c_char,
                1 as size_t,
            ) == 0 as ::core::ffi::c_int
                && cmd == SIGNCMD_UNPLACE
            {
                if *id != -1 as ::core::ffi::c_int {
                    emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                    return FAIL;
                }
                *id = -2 as ::core::ffi::c_int;
                arg = skiptowhite(arg.offset(1 as ::core::ffi::c_int as isize));
            } else if strncmp(
                arg,
                b"name=\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                arg = arg.offset(5 as ::core::ffi::c_int as isize);
                let mut namep: *mut ::core::ffi::c_char = arg;
                arg = skiptowhite(arg);
                if *arg as ::core::ffi::c_int != NUL {
                    let c2rust_fresh5 = arg;
                    arg = arg.offset(1);
                    *c2rust_fresh5 = NUL as ::core::ffi::c_char;
                }
                while *namep.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '0' as ::core::ffi::c_int
                    && *namep.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    namep = namep.offset(1);
                }
                *name = namep;
            } else if strncmp(
                arg,
                b"group=\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                arg = arg.offset(6 as ::core::ffi::c_int as isize);
                *group = arg;
                arg = skiptowhite(arg);
                if *arg as ::core::ffi::c_int != NUL {
                    let c2rust_fresh6 = arg;
                    arg = arg.offset(1);
                    *c2rust_fresh6 = NUL as ::core::ffi::c_char;
                }
            } else if strncmp(
                arg,
                b"priority=\0".as_ptr() as *const ::core::ffi::c_char,
                9 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                arg = arg.offset(9 as ::core::ffi::c_int as isize);
                *prio = atoi(arg);
                arg = skiptowhite(arg);
            } else if strncmp(
                arg,
                b"file=\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                arg = arg.offset(5 as ::core::ffi::c_int as isize);
                filename = arg;
                *buf = buflist_findname_exp(arg);
                break;
            } else if strncmp(
                arg,
                b"buffer=\0".as_ptr() as *const ::core::ffi::c_char,
                7 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                arg = arg.offset(7 as ::core::ffi::c_int as isize);
                filename = arg;
                *buf = buflist_findnr(getdigits_int(&raw mut arg, true, 0 as ::core::ffi::c_int));
                if *skipwhite(arg) as ::core::ffi::c_int != NUL {
                    semsg(
                        gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                        arg,
                    );
                }
                break;
            } else {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return FAIL;
            }
            arg = skipwhite(arg);
        }
        if !filename.is_null() && (*buf).is_null() {
            semsg(
                gettext(&raw const e_invalid_buffer_name_str as *const ::core::ffi::c_char),
                filename,
            );
            return FAIL;
        }
        if filename.is_null()
            && (cmd == SIGNCMD_PLACE && lnum_arg as ::core::ffi::c_int != 0 || cmd == SIGNCMD_JUMP)
        {
            *buf = (*curwin.get()).w_buffer;
        }
        return OK;
    }
}

pub unsafe fn ex_sign(mut eap: *mut exarg_T) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut p: *mut ::core::ffi::c_char = skiptowhite(arg);
        let mut idx: ::core::ffi::c_int = sign_cmd_idx(arg, p);
        if idx == SIGNCMD_LAST {
            semsg(
                gettext(b"E160: Unknown sign command: %s\0".as_ptr() as *const ::core::ffi::c_char),
                arg,
            );
            return;
        }
        arg = skipwhite(p);
        if idx <= SIGNCMD_LIST {
            if idx == SIGNCMD_LIST && *arg as ::core::ffi::c_int == NUL {
                for sp in sign_defs() {
                    sign_list_defined(sp);
                }
            } else if *arg as ::core::ffi::c_int == NUL {
                emsg(gettext(
                    b"E156: Missing sign name\0".as_ptr() as *const ::core::ffi::c_char
                ));
            } else {
                p = skiptowhite(arg);
                if *p as ::core::ffi::c_int != NUL {
                    let c2rust_fresh0 = p;
                    p = p.offset(1);
                    *c2rust_fresh0 = NUL as ::core::ffi::c_char;
                }
                while *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '0' as ::core::ffi::c_int
                    && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    arg = arg.offset(1);
                }
                if idx == SIGNCMD_DEFINE {
                    sign_define_cmd(arg, p);
                } else if idx == SIGNCMD_LIST {
                    sign_list_by_name(arg);
                } else {
                    sign_undefine_by_name(arg);
                }
                return;
            }
        } else {
            let mut id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            let mut lnum: linenr_T = -1 as linenr_T;
            let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut group: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut prio: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
            if parse_sign_cmd_args(
                idx,
                arg,
                &raw mut name,
                &raw mut id,
                &raw mut group,
                &raw mut prio,
                &raw mut buf,
                &raw mut lnum,
            ) == FAIL
            {
                return;
            }
            if idx == SIGNCMD_PLACE {
                sign_place_cmd(buf, lnum, name, id, group, prio);
            } else if idx == SIGNCMD_UNPLACE {
                sign_unplace_cmd(buf, lnum, name, id, group);
            } else if idx == SIGNCMD_JUMP {
                sign_jump_cmd(buf, lnum, name, id, group);
            }
        };
    }
}
