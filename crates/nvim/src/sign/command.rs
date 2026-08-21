//! The `:sign` Ex command.
//!
//! [`ex_sign`] picks a subcommand out of [`CMDS`], [`parse_sign_cmd_args`]
//! turns the rest of the line into the `line=`/`name=`/`group=`/
//! `priority=`/`file=`/`buffer=` tuple the three placement subcommands
//! share, and the `sign_*_cmd` functions diagnose the combinations that do
//! not make sense before handing off to the placement primitives in the
//! parent. [`sign_list_placed`] and [`sign_list_defined`] are the
//! `:sign place` / `:sign list` reports.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::FAIL;
use crate::{semsg_c, smsg_c};

/// A `vim_snprintf` into a fresh [`MSG_BUF_LEN`] buffer, kept as bytes so the
/// caller can hand the result straight back to `msg_puts`.
///
/// Every message in this module is bounded that way upstream; `MSG_BUF_LEN`
/// is 480, and the format arguments (a file name, a group name, a sign name)
/// can all exceed it, so the truncation is load-bearing rather than
/// defensive.
macro_rules! msg_buf {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        let mut buf = [0 as c_char; MSG_BUF_LEN as usize];
        // SAFETY: the buffer is exactly the length passed, and the format
        // string and arguments are the caller's. Every expansion is inside
        // the caller's own `unsafe` block, so this carries none of its own.
        vim_snprintf(buf.as_mut_ptr(), MSG_BUF_LEN as size_t, gettext($fmt) $(, $arg)*);
        buf
    }};
}

/// The `:sign place` report for `rbuf`, or for every buffer when it is null.
///
/// # Safety
/// `rbuf` must be null or live; `group` must be null or NUL-terminated.
pub(crate) unsafe fn sign_list_placed(rbuf: *mut buf_T, group: *mut c_char) {
    // SAFETY: the caller's group name.
    let ns = unsafe { group_get_ns(group) };
    // SAFETY: a static title.
    unsafe { msg_puts_title(gettext(c"\n--- Signs ---".as_ptr())) };

    let mut buf = if rbuf.is_null() { firstbuf.get() } else { rbuf };
    while !buf.is_null() && !got_int.get() {
        // SAFETY: a live buffer, either the caller's or one off the list.
        let cbuf = unsafe { Buf::new(buf) };
        // SAFETY: as above.
        if unsafe { buf_has_signs(buf) } {
            // SAFETY: a live buffer's name is a NUL-terminated string.
            unsafe {
                msg_putchar('\n' as c_int);
                let lbuf = msg_buf!(c"Signs for %s:".as_ptr(), cbuf.b_fname);
                msg_puts_hl(lbuf.as_ptr(), HLF_D, false);
            };
        }

        // A group that names no namespace matches nothing, but still prints
        // the per-buffer header above.
        if ns >= 0 {
            let mut signs = placed_signs(cbuf, 0, ns, |_| Keep::Yes);
            if !signs.is_empty() {
                // SAFETY: every mark collected carries a live sign.
                unsafe { sort_signs(&mut signs) };
                // SAFETY: as above; each `sh` is that mark's own decoration.
                unsafe { report_signs(&signs) };
            }
        }

        if !rbuf.is_null() {
            return;
        }
        buf = cbuf.b_next;
    }
}

/// The `line=`/`id=`/`group=`/`name=`/`priority=` lines `:sign place`
/// prints for one buffer's signs, already sorted.
///
/// # Safety
/// Every mark must carry a live sign decoration.
unsafe fn report_signs(signs: &[MTKey]) {
    // SAFETY: a static newline, and the caller's marks.
    unsafe {
        msg_putchar('\n' as c_int);
        for (i, mark) in signs.iter().enumerate() {
            let sh = Sh::new(decor_find_sign(mt_decor(*mark)));
            let namebuf = if sh.sign_name.is_null() {
                [0; MSG_BUF_LEN as usize]
            } else {
                msg_buf!(c"  name=%s".as_ptr(), sign_get_name(sh.raw()))
            };
            let groupbuf = if mark.ns == 0 {
                [0; MSG_BUF_LEN as usize]
            } else {
                msg_buf!(
                    c"  group=%s".as_ptr(),
                    describe_ns(mark.ns as NS, c"".as_ptr()),
                )
            };
            let lbuf = msg_buf!(
                c"    line=%d  id=%u%s%s  priority=%d".as_ptr(),
                mark.pos.row + 1,
                mark.id,
                groupbuf.as_ptr(),
                namebuf.as_ptr(),
                c_int::from(sh.priority),
            );
            msg_puts(lbuf.as_ptr());
            if i + 1 < signs.len() {
                msg_putchar('\n' as c_int);
            }
        }
    };
}

/// The index of the `:sign` subcommand named between `begin_cmd` and
/// `end_cmd`, or [`SIGNCMD_LAST`] for one that is not a subcommand.
///
/// `end_cmd` is terminated in place for the comparison and put back, which
/// is why the command line has to be writable.
///
/// # Safety
/// `begin_cmd` must be NUL-terminated and `end_cmd` a writable position
/// within it.
pub(crate) unsafe fn sign_cmd_idx(begin_cmd: *mut c_char, end_cmd: *mut c_char) -> c_int {
    // SAFETY: the caller's command line.
    unsafe {
        let save = *end_cmd;
        *end_cmd = 0;
        let idx = CMDS
            .iter()
            .position(|cmd| strcmp(begin_cmd, cmd.as_ptr()) == 0)
            .map_or(SIGNCMD_LAST, |i| i as c_int);
        *end_cmd = save;
        idx
    }
}

/// The `:sign list` report for one definition.
///
/// # Safety
/// `sp` must be a live sign definition.
pub(crate) unsafe fn sign_list_defined(sp: Sign) {
    // SAFETY: a definition's name, icon and cells are its own.
    unsafe {
        smsg_c!(0, c"sign %s".as_ptr(), sp.sn_name);
        if !sp.sn_icon.is_null() {
            msg_puts(c" icon=".as_ptr());
            msg_outtrans(sp.sn_icon, 0, false);
            msg_puts(gettext(c" (not supported)".as_ptr()));
        }
        if sp.sn_text[0] != 0 {
            msg_puts(c" text=".as_ptr());
            let mut buf = [0 as c_char; SIGN_TEXT_BUF];
            describe_sign_text(buf.as_mut_ptr(), sp.cells());
            msg_outtrans(buf.as_ptr(), 0, false);
        }
        if sp.sn_priority > 0 {
            let lbuf = msg_buf!(c" priority=%d".as_ptr(), sp.sn_priority);
            msg_puts(lbuf.as_ptr());
        }
        let labels = [c" linehl=", c" texthl=", c" culhl=", c" numhl="];
        let ids = [sp.sn_line_hl, sp.sn_text_hl, sp.sn_cul_hl, sp.sn_num_hl];
        for (label, id) in labels.into_iter().zip(ids) {
            if id > 0 {
                msg_puts(label.as_ptr());
                let p = get_highlight_name_ext(::core::ptr::null_mut(), id - 1, false);
                msg_puts(if p.is_null() { c"NONE".as_ptr() } else { p });
            }
        }
    }
}

/// `:sign list {name}`.
///
/// # Safety
/// `name` must be a NUL-terminated string.
unsafe fn sign_list_by_name(name: *mut c_char) {
    // SAFETY: the caller's name.
    match unsafe { sign_find(name) } {
        // SAFETY: `sign_find` answered a live definition.
        Some(sp) => unsafe { sign_list_defined(sp) },
        None => {
            // SAFETY: the caller's name, and a format the message takes.
            unsafe { semsg_c!(gettext(c"E155: Unknown sign: %s".as_ptr()), name) };
        }
    }
}

/// `:sign define {name} {args}...`.
///
/// The arguments are `key=value` pairs in any order; an unrecognised key is
/// E474 and abandons the rest of the line, having already applied the ones
/// before it.
///
/// # Safety
/// `name` and `cmdline` must be writable NUL-terminated strings; this
/// terminates each argument in place.
unsafe fn sign_define_cmd(name: *mut c_char, cmdline: *mut c_char) {
    // SAFETY: the caller's command line.
    unsafe {
        let null = ::core::ptr::null_mut();
        let (mut icon, mut text) = (null, null);
        let (mut linehl, mut texthl, mut culhl, mut numhl) = (null, null, null, null);
        let mut prio = -1;

        let mut cmdline = cmdline;
        loop {
            let arg = skipwhite(cmdline);
            if *arg == 0 {
                break;
            }
            cmdline = skiptowhite_esc(arg);

            let after = |lit: &CStr| {
                if strncmp(arg, lit.as_ptr(), lit.count_bytes()) == 0 {
                    Some(arg.add(lit.count_bytes()))
                } else {
                    None
                }
            };
            if let Some(v) = after(c"icon=") {
                icon = v;
            } else if let Some(v) = after(c"text=") {
                text = v;
            } else if let Some(v) = after(c"linehl=") {
                linehl = v;
            } else if let Some(v) = after(c"texthl=") {
                texthl = v;
            } else if let Some(v) = after(c"culhl=") {
                culhl = v;
            } else if let Some(v) = after(c"numhl=") {
                numhl = v;
            } else if let Some(v) = after(c"priority=") {
                prio = atoi(v);
            } else {
                semsg_c!(gettext(&raw const e_invarg2 as *const c_char), arg);
                return;
            }

            if *cmdline == 0 {
                break;
            }
            // Terminate this argument's value; the next one starts after it.
            *cmdline = 0;
            cmdline = cmdline.add(1);
        }

        sign_define_by_name(name, icon, text, linehl, texthl, culhl, numhl, prio);
    }
}

/// `:sign place`, which both places a sign and — with no id — lists them.
///
/// # Safety
/// `buf` must be null or live; `name` and `group` must be null or
/// NUL-terminated.
unsafe fn sign_place_cmd(
    buf: *mut buf_T,
    lnum: linenr_T,
    name: *mut c_char,
    id: c_int,
    group: *mut c_char,
    prio: c_int,
) {
    // SAFETY: the caller's buffer, name and group.
    unsafe {
        let empty_group = !group.is_null() && *group == 0;
        if id <= 0 {
            // The listing forms: `:sign place [group=X] [file=Y|buffer=N]`.
            // A `line=` or a `name=` means a placement was intended.
            if lnum >= 0 || !name.is_null() || empty_group {
                emsg(gettext(&raw const e_invarg as *const c_char));
            } else {
                sign_list_placed(buf, group);
            }
            return;
        }
        if name.is_null() || buf.is_null() || empty_group {
            emsg(gettext(&raw const e_invarg as *const c_char));
            return;
        }
        let mut uid = id as uint32_t;
        sign_place(&raw mut uid, group, name, buf, lnum, prio);
    }
}

/// `:sign unplace`.
///
/// With no id at all it removes the highest-priority sign on the *cursor*
/// line of the current buffer — the only spelling that reaches
/// `buf_delete_signs`' single-line form, since an explicit `line=` is E474.
///
/// # Safety
/// `buf` must be null or live; `name` and `group` must be null or
/// NUL-terminated.
unsafe fn sign_unplace_cmd(
    buf: *mut buf_T,
    lnum: linenr_T,
    name: *const c_char,
    id: c_int,
    group: *mut c_char,
) {
    // SAFETY: the caller's buffer, name and group.
    unsafe {
        if lnum >= 0 || !name.is_null() || (!group.is_null() && *group == 0) {
            emsg(gettext(&raw const e_invarg as *const c_char));
            return;
        }

        let (buf, lnum) = if id == -1 {
            ((*curwin.get()).w_buffer, (*curwin.get()).w_cursor.lnum)
        } else {
            (buf, lnum)
        };

        if sign_unplace(buf, id.max(0), group, lnum) == FAIL && lnum > 0 {
            emsg(gettext(c"E159: Missing sign number".as_ptr()));
        }
    }
}

/// `:sign jump {id} [group={group}] file={fname}|buffer={nr}`.
///
/// # Safety
/// `buf` must be null or live; `name` and `group` must be null or
/// NUL-terminated.
unsafe fn sign_jump_cmd(
    buf: *mut buf_T,
    lnum: linenr_T,
    name: *const c_char,
    id: c_int,
    group: *mut c_char,
) {
    // SAFETY: the caller's buffer, name and group.
    unsafe {
        if name.is_null() && group.is_null() && id == -1 {
            emsg(gettext(&raw const e_argreq as *const c_char));
            return;
        }
        // No buffer, an empty group, or a `line=`/`name=` that jumping has
        // no use for.
        if buf.is_null() || (!group.is_null() && *group == 0) || lnum >= 0 || !name.is_null() {
            emsg(gettext(&raw const e_invarg as *const c_char));
            return;
        }
        sign_jump(id, group, buf);
    }
}

/// What [`parse_sign_cmd_args`] read off a `:sign place`/`unplace`/`jump`
/// line.
///
/// The "absent" values are what the three `sign_*_cmd` functions test
/// against, and they are not all zero: `id` and `lnum` are −1 so that an
/// explicit `0` can be told from no argument at all.
struct SignCmdArgs {
    name: *mut c_char,
    id: c_int,
    group: *mut c_char,
    prio: c_int,
    buf: *mut buf_T,
    lnum: linenr_T,
}

impl Default for SignCmdArgs {
    fn default() -> Self {
        Self {
            name: ::core::ptr::null_mut(),
            id: -1,
            group: ::core::ptr::null_mut(),
            prio: -1,
            buf: ::core::ptr::null_mut(),
            lnum: -1,
        }
    }
}

/// Parses the arguments `:sign place`, `:sign unplace` and `:sign jump`
/// share: an optional leading id, then `line=`, `name=`, `group=`,
/// `priority=` and one of `file=`/`buffer=`.
///
/// Answers `FAIL` after diagnosing; `OK` otherwise, including when nothing
/// was given.
///
/// # Safety
/// `arg` must be a writable NUL-terminated string; the `name=` and `group=`
/// values are terminated in place and pointed into.
unsafe fn parse_sign_cmd_args(cmd: c_int, arg: *mut c_char) -> Option<SignCmdArgs> {
    // SAFETY: the caller's command line.
    unsafe {
        let mut out = SignCmdArgs::default();
        let arg1 = arg;
        let mut arg = arg;
        let mut filename: *mut c_char = ::core::ptr::null_mut();
        let mut lnum_arg = false;

        // A leading number is the sign id — but only if a separator follows,
        // so that `:sign unplace 3name=x` is not read as id 3.
        if ascii_isdigit(*arg as c_int) {
            out.id = getdigits_int(&raw mut arg, true, 0);
            if !ascii_iswhite(*arg as c_int) && *arg != 0 {
                out.id = -1;
                arg = arg1;
            } else {
                arg = skipwhite(arg);
            }
        }

        while *arg != 0 {
            let after = |lit: &CStr| {
                if strncmp(arg, lit.as_ptr(), lit.count_bytes()) == 0 {
                    Some(arg.add(lit.count_bytes()))
                } else {
                    None
                }
            };
            if let Some(v) = after(c"line=") {
                out.lnum = atoi(v);
                arg = skiptowhite(v);
                lnum_arg = true;
            } else if cmd == SIGNCMD_UNPLACE && *arg == b'*' as c_char {
                // `:sign unplace *`: every sign, and not with an id too.
                if out.id != -1 {
                    emsg(gettext(&raw const e_invarg as *const c_char));
                    return None;
                }
                out.id = -2;
                arg = skiptowhite(arg.add(1));
            } else if let Some(v) = after(c"name=") {
                let mut namep = v;
                arg = skiptowhite(v);
                if *arg != 0 {
                    *arg = 0;
                    arg = arg.add(1);
                }
                // Leading zeroes are stripped, so "099" and "99" name the
                // same sign — but a bare "0" is kept.
                while *namep == b'0' as c_char && *namep.add(1) != 0 {
                    namep = namep.add(1);
                }
                out.name = namep;
            } else if let Some(v) = after(c"group=") {
                out.group = v;
                arg = skiptowhite(v);
                if *arg != 0 {
                    *arg = 0;
                    arg = arg.add(1);
                }
            } else if let Some(v) = after(c"priority=") {
                out.prio = atoi(v);
                arg = skiptowhite(v);
            } else if let Some(v) = after(c"file=") {
                filename = v;
                out.buf = buflist_findname_exp(v);
                break;
            } else if let Some(v) = after(c"buffer=") {
                filename = v;
                let mut p = v;
                out.buf = buflist_findnr(getdigits_int(&raw mut p, true, 0));
                // Diagnosed but not fatal, which is why this still breaks
                // out with whatever buffer it found.
                if *skipwhite(p) != 0 {
                    semsg_c!(gettext(&raw const e_trailing_arg as *const c_char), p);
                }
                break;
            } else {
                emsg(gettext(&raw const e_invarg as *const c_char));
                return None;
            }
            arg = skipwhite(arg);
        }

        if !filename.is_null() && out.buf.is_null() {
            semsg_c!(
                gettext(&raw const e_invalid_buffer_name_str as *const c_char),
                filename,
            );
            return None;
        }

        // `:sign place line=N` and `:sign jump` default to the current
        // buffer; `:sign unplace` deliberately does not.
        if filename.is_null() && ((cmd == SIGNCMD_PLACE && lnum_arg) || cmd == SIGNCMD_JUMP) {
            out.buf = (*curwin.get()).w_buffer;
        }
        Some(out)
    }
}

/// `:sign`.
///
/// # Safety
/// `eap` must be a live Ex-command argument block with a writable `arg`.
pub unsafe fn ex_sign(eap: *mut exarg_T) {
    // SAFETY: the caller's command.
    unsafe {
        let mut arg = (*eap).arg;

        let p = skiptowhite(arg);
        let idx = sign_cmd_idx(arg, p);
        if idx == SIGNCMD_LAST {
            semsg_c!(gettext(c"E160: Unknown sign command: %s".as_ptr()), arg);
            return;
        }
        arg = skipwhite(p);

        if idx > SIGNCMD_LIST {
            // Place, unplace or jump: a shared argument parser first.
            let Some(a) = parse_sign_cmd_args(idx, arg) else {
                return;
            };
            match idx {
                SIGNCMD_PLACE => sign_place_cmd(a.buf, a.lnum, a.name, a.id, a.group, a.prio),
                SIGNCMD_UNPLACE => sign_unplace_cmd(a.buf, a.lnum, a.name, a.id, a.group),
                SIGNCMD_JUMP => sign_jump_cmd(a.buf, a.lnum, a.name, a.id, a.group),
                _ => {}
            }
            return;
        }

        // Define, undefine or list.
        if idx == SIGNCMD_LIST && *arg == 0 {
            for sp in sign_defs() {
                sign_list_defined(sp);
            }
            return;
        }
        if *arg == 0 {
            emsg(gettext(c"E156: Missing sign name".as_ptr()));
            return;
        }

        // Isolate the sign name. Leading zeroes are stripped so "099" and
        // "99" are the same sign, but a bare "0" is kept.
        let mut p = skiptowhite(arg);
        if *p != 0 {
            *p = 0;
            p = p.add(1);
        }
        while *arg == b'0' as c_char && *arg.add(1) != 0 {
            arg = arg.add(1);
        }

        match idx {
            SIGNCMD_DEFINE => sign_define_cmd(arg, p),
            SIGNCMD_LIST => sign_list_by_name(arg),
            _ => {
                sign_undefine_by_name(arg);
            }
        }
    }
}
