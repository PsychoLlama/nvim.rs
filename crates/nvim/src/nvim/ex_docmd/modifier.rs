//! The command modifiers (`:silent`, `:verbose`, `:tab`,
//! `:keeppatterns`, the split direction, `:filter`, …): recognising them,
//! putting them in force around the command, and taking them out again.
//!
//! `parse_command_modifiers` only *fills in* a `cmdmod_T`; `apply_cmdmod`
//! is what puts it in force and `undo_cmdmod` what takes it back out, and
//! the two must stay a matched pair — `do_one_cmd` runs the second on
//! every exit path, including the ones an error takes.
#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::charset::{skipdigits, skipwhite};
use crate::src::nvim::ex_cmds::skip_vimgrep_pat;
use crate::src::nvim::ex_docmd::address::{get_address, skip_range};
use crate::src::nvim::ex_docmd::ex_func_T;
use crate::src::nvim::ex_docmd::lookup::checkforcmd;
use crate::src::nvim::ex_docmd::onecmd::ex_func_is;
use crate::src::nvim::ex_docmd::scan::ends_excmd;
use crate::src::nvim::ex_docmd::source::getline_equal;
use crate::src::nvim::ex_docmd::window::current_tab_nr;
use crate::src::nvim::ex_docmd::{
    ADDR_TABS, BF_DUMMY, CMD_SIZE, CMD_echo, CMD_echoerr, CMD_echomsg, CMD_echon, CMD_execute,
    CMOD_BROWSE, CMOD_CONFIRM, CMOD_ERRSILENT, CMOD_HIDE, CMOD_KEEPALT, CMOD_KEEPJUMPS,
    CMOD_KEEPMARKS, CMOD_KEEPPATTERNS, CMOD_LOCKMARKS, CMOD_NOAUTOCMD, CMOD_NOSWAPFILE,
    CMOD_SANDBOX, CMOD_SILENT, CMOD_UNSILENT, FAIL, NUL, OK, SID_NONE, cmdnames, e_invrange,
    ex_pressedreturn, exmode_plus, getexline,
};
use crate::src::nvim::main::{
    curbuf, curtab, curwin, did_emsg, emsg_silent, exmode_active, expr_map_lock, msg_silent, p_ei,
    p_verbose, sandbox,
};
use crate::src::nvim::main::{msg_col, msg_scroll};
use crate::src::nvim::mapping::{ex_abbreviate, ex_abclear, ex_map, ex_mapclear, ex_unmap};
use crate::src::nvim::memory::{xfree, xmemcpyz, xstrdup};
use crate::src::nvim::message::redirecting;
use crate::src::nvim::option::kOptValTypeString;
use crate::src::nvim::option::set_option_direct;
use crate::src::nvim::options::kOptEventignore;
use crate::src::nvim::optionstr::free_string_option;
use crate::src::nvim::os::libc::{atoi, gettext, memmove, memset, strlen, strncmp};
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::regexp::{RE_MAGIC, vim_regcomp, vim_regfree};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::String_0;
use crate::src::nvim::types::{OptInt, size_t};
use crate::src::nvim::types::{OptVal, OptValData};
use crate::src::nvim::types::{cmdidx_T, cmdmod_T, exarg_T};
use crate::src::nvim::window::{
    WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_HOR, WSP_TOP, WSP_VERT, tabpage_index,
};

/// One recognised modifier name, for the two callers that only need to know
/// *whether* a word is one: `modifier_len` and `cmd_exists`.
pub(crate) struct CmdMod {
    pub(crate) name: &'static CStr,
    pub(crate) minlen: usize,
    /// Whether a count may precede this modifier (`:3tab`, `:5verbose`).
    pub(crate) has_count: bool,
}

const fn m(name: &'static CStr, minlen: usize, has_count: bool) -> CmdMod {
    CmdMod {
        name,
        minlen,
        has_count,
    }
}

/// Every command modifier, in the order upstream lists them.
///
/// This table is *not* what `parse_command_modifiers` dispatches through —
/// that has a hand-written `match` on the first byte, because several
/// modifiers need more than a name match. This is the answer to "is this
/// word a modifier at all".
#[rustfmt::skip]
pub(crate) static CMDMODS: [CmdMod; 24] = [
    m(c"aboveleft", 3, false),   m(c"belowright", 3, false),
    m(c"botright", 2, false),    m(c"browse", 3, false),
    m(c"confirm", 4, false),     m(c"filter", 4, false),
    m(c"hide", 3, false),        m(c"horizontal", 3, false),
    m(c"keepalt", 5, false),     m(c"keepjumps", 5, false),
    m(c"keepmarks", 3, false),   m(c"keeppatterns", 5, false),
    m(c"leftabove", 5, false),   m(c"lockmarks", 3, false),
    m(c"noautocmd", 3, false),   m(c"noswapfile", 3, false),
    m(c"rightbelow", 6, false),  m(c"sandbox", 3, false),
    m(c"silent", 3, false),      m(c"tab", 3, true),
    m(c"topleft", 2, false),     m(c"unsilent", 3, false),
    m(c"verbose", 4, true),      m(c"vertical", 4, false),
];

/// Commands whose argument is an expression, so a `|` inside it belongs to
/// the expression rather than separating commands.
pub fn cmd_has_expr_args(cmdidx: cmdidx_T) -> bool {
    let i = cmdidx as c_int;
    i == CMD_execute as c_int
        || i == CMD_echo as c_int
        || i == CMD_echon as c_int
        || i == CMD_echomsg as c_int
        || i == CMD_echoerr as c_int
}

/// Read the run of modifiers at the head of the command line into `cmod`,
/// advancing `eap->cmd` past them.
///
/// Returns `FAIL` when there is no command at all — a comment, a bare
/// newline, or an empty line — and `OK` otherwise, including when the line
/// carried no modifier.
///
/// `skip_only` is `nvim_parse_cmd`'s mode: recognise everything, allocate
/// and evaluate nothing.
pub unsafe fn parse_command_modifiers(
    eap: *mut exarg_T,
    errormsg: *mut *const c_char,
    cmod: *mut cmdmod_T,
    skip_only: bool,
) -> c_int {
    unsafe {
        let ea = &mut *eap;
        let cm = &mut *cmod;
        let orig_cmd = ea.cmd;
        let mut cmd_start: *mut c_char = ptr::null_mut();
        let mut use_plus_cmd = false;
        let mut has_visual_range = false;
        memset(cmod as *mut c_void, 0, size_of::<cmdmod_T>());

        // A `'<,'>` typed by the user (which is what a Visual-mode `:` puts
        // there) is stepped over so a modifier after it is still seen, and
        // put back below — the range has to reach the command, not the
        // modifier scan.
        if strncmp(ea.cmd, c"'<,'>".as_ptr(), 5) == 0 {
            let p = skipwhite(ea.cmd.add(5));
            if *p as c_int != NUL && *p as c_int != '|' as c_int {
                ea.cmd = ea.cmd.add(5);
                cmd_start = ea.cmd;
                has_visual_range = true;
            }
        }

        loop {
            while *ea.cmd as c_int == ' ' as c_int
                || *ea.cmd as c_int == '\t' as c_int
                || *ea.cmd as c_int == ':' as c_int
            {
                ea.cmd = ea.cmd.add(1);
            }

            // In Ex mode an empty line means "print the next one", which is
            // spelled by substituting a `+` command.
            if *ea.cmd as c_int == NUL
                && exmode_active.get()
                && getline_equal(ea.ea_getline, ea.cookie, Some(getexline))
                && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
            {
                ea.cmd = exmode_plus.ptr() as *mut c_char;
                use_plus_cmd = true;
                if !skip_only {
                    ex_pressedreturn.set(true);
                }
                break;
            }
            if *ea.cmd as c_int == '"' as c_int {
                ea.nextcmd = vim_strchr(ea.cmd, '\n' as c_int);
                if !ea.nextcmd.is_null() {
                    ea.nextcmd = ea.nextcmd.add(1);
                }
                return FAIL;
            }
            if *ea.cmd as c_int == '\n' as c_int {
                ea.nextcmd = ea.cmd.add(1);
                return FAIL;
            }
            if *ea.cmd as c_int == NUL {
                if !skip_only {
                    ex_pressedreturn.set(true);
                }
                return FAIL;
            }

            // A modifier may follow a range (`:1,2 silent print`), so the
            // name is looked for past one — but `eap->cmd` only moves for
            // the modifiers that accept that.
            let mut p = skip_range(ea.cmd, ptr::null_mut());
            match *p as u8 {
                b'a' => {
                    if !checkforcmd(&raw mut ea.cmd, c"aboveleft".as_ptr(), 3) {
                        break;
                    }
                    cm.cmod_split |= WSP_ABOVE as c_int;
                }
                b'b' => {
                    if checkforcmd(&raw mut ea.cmd, c"belowright".as_ptr(), 3) {
                        cm.cmod_split |= WSP_BELOW as c_int;
                    } else if checkforcmd(&raw mut ea.cmd, c"browse".as_ptr(), 3) {
                        cm.cmod_flags |= CMOD_BROWSE as c_int;
                    } else if checkforcmd(&raw mut ea.cmd, c"botright".as_ptr(), 2) {
                        cm.cmod_split |= WSP_BOT as c_int;
                    } else {
                        break;
                    }
                }
                b'c' => {
                    if !checkforcmd(&raw mut ea.cmd, c"confirm".as_ptr(), 4) {
                        break;
                    }
                    cm.cmod_flags |= CMOD_CONFIRM as c_int;
                }
                b'k' => {
                    if checkforcmd(&raw mut ea.cmd, c"keepmarks".as_ptr(), 3) {
                        cm.cmod_flags |= CMOD_KEEPMARKS as c_int;
                    } else if checkforcmd(&raw mut ea.cmd, c"keepalt".as_ptr(), 5) {
                        cm.cmod_flags |= CMOD_KEEPALT as c_int;
                    } else if checkforcmd(&raw mut ea.cmd, c"keeppatterns".as_ptr(), 5) {
                        cm.cmod_flags |= CMOD_KEEPPATTERNS as c_int;
                    } else if checkforcmd(&raw mut ea.cmd, c"keepjumps".as_ptr(), 5) {
                        cm.cmod_flags |= CMOD_KEEPJUMPS as c_int;
                    } else {
                        break;
                    }
                }
                b'f' => {
                    // `:filter` insists on a pattern *and* something after
                    // it: the whole point is the command it wraps.
                    let mut reg_pat: *mut c_char = ptr::null_mut();
                    if !checkforcmd(&raw mut p, c"filter".as_ptr(), 4)
                        || *p as c_int == NUL
                        || ends_excmd(*p as c_int) != 0
                    {
                        break;
                    }
                    if *p as c_int == '!' as c_int {
                        cm.cmod_filter_force = true;
                        p = skipwhite(p.add(1));
                        if *p as c_int == NUL || ends_excmd(*p as c_int) != 0 {
                            break;
                        }
                    }
                    p = if skip_only {
                        skip_vimgrep_pat(p, ptr::null_mut(), ptr::null_mut())
                    } else {
                        skip_vimgrep_pat(p, &raw mut reg_pat, ptr::null_mut())
                    };
                    if p.is_null() || *p as c_int == NUL {
                        break;
                    }
                    if !skip_only {
                        cm.cmod_filter_pat = xstrdup(reg_pat);
                        cm.cmod_filter_regmatch.regprog = vim_regcomp(reg_pat, RE_MAGIC);
                        if cm.cmod_filter_regmatch.regprog.is_null() {
                            break;
                        }
                    }
                    ea.cmd = p;
                }
                b'h' => {
                    if checkforcmd(&raw mut ea.cmd, c"horizontal".as_ptr(), 3) {
                        cm.cmod_split |= WSP_HOR as c_int;
                    } else if p == ea.cmd
                        && checkforcmd(&raw mut p, c"hide".as_ptr(), 3)
                        && *p as c_int != NUL
                        && ends_excmd(*p as c_int) == 0
                    {
                        // `:hide` is a command in its own right, so it is
                        // only a modifier when a command follows it and no
                        // range precedes it.
                        ea.cmd = p;
                        cm.cmod_flags |= CMOD_HIDE as c_int;
                    } else {
                        break;
                    }
                }
                b'l' => {
                    if checkforcmd(&raw mut ea.cmd, c"lockmarks".as_ptr(), 3) {
                        cm.cmod_flags |= CMOD_LOCKMARKS as c_int;
                    } else if checkforcmd(&raw mut ea.cmd, c"leftabove".as_ptr(), 5) {
                        cm.cmod_split |= WSP_ABOVE as c_int;
                    } else {
                        break;
                    }
                }
                b'n' => {
                    if checkforcmd(&raw mut ea.cmd, c"noautocmd".as_ptr(), 3) {
                        cm.cmod_flags |= CMOD_NOAUTOCMD as c_int;
                    } else if checkforcmd(&raw mut ea.cmd, c"noswapfile".as_ptr(), 3) {
                        cm.cmod_flags |= CMOD_NOSWAPFILE as c_int;
                    } else {
                        break;
                    }
                }
                b'r' => {
                    if !checkforcmd(&raw mut ea.cmd, c"rightbelow".as_ptr(), 6) {
                        break;
                    }
                    cm.cmod_split |= WSP_BELOW as c_int;
                }
                b's' => {
                    if checkforcmd(&raw mut ea.cmd, c"sandbox".as_ptr(), 3) {
                        cm.cmod_flags |= CMOD_SANDBOX as c_int;
                    } else if checkforcmd(&raw mut ea.cmd, c"silent".as_ptr(), 3) {
                        cm.cmod_flags |= CMOD_SILENT as c_int;
                        // `:silent!` only means "and silence errors" when
                        // the `!` is stuck to the word: `:silent !cmd` runs
                        // a shell command quietly.
                        if *ea.cmd as c_int == '!' as c_int
                            && !ascii_iswhite(*ea.cmd.offset(-1) as c_int)
                        {
                            ea.cmd = skipwhite(ea.cmd.add(1));
                            cm.cmod_flags |= CMOD_ERRSILENT as c_int;
                        }
                    } else {
                        break;
                    }
                }
                b't' => {
                    if checkforcmd(&raw mut p, c"tab".as_ptr(), 3) {
                        if !skip_only {
                            let tabnr = get_address(
                                eap,
                                &raw mut ea.cmd,
                                ADDR_TABS,
                                ea.skip != 0,
                                skip_only,
                                0,
                                1,
                                errormsg,
                            ) as c_int;
                            if ea.cmd.is_null() {
                                return FAIL;
                            }
                            if tabnr == MAXLNUM as c_int {
                                cm.cmod_tab = tabpage_index(curtab.get()) + 1;
                            } else {
                                if tabnr < 0 || tabnr > current_tab_nr(ptr::null_mut()) {
                                    *errormsg = gettext(&raw const e_invrange as *const c_char);
                                    return FAIL;
                                }
                                cm.cmod_tab = tabnr + 1;
                            }
                        }
                        ea.cmd = p;
                    } else if checkforcmd(&raw mut ea.cmd, c"topleft".as_ptr(), 2) {
                        cm.cmod_split |= WSP_TOP as c_int;
                    } else {
                        break;
                    }
                }
                b'u' => {
                    if !checkforcmd(&raw mut ea.cmd, c"unsilent".as_ptr(), 3) {
                        break;
                    }
                    cm.cmod_flags |= CMOD_UNSILENT as c_int;
                }
                b'v' => {
                    if checkforcmd(&raw mut ea.cmd, c"vertical".as_ptr(), 4) {
                        cm.cmod_split |= WSP_VERT as c_int;
                    } else if checkforcmd(&raw mut p, c"verbose".as_ptr(), 4) {
                        // The count is read from `eap->cmd`, which
                        // `checkforcmd` left *before* the word: `:5verbose`.
                        cm.cmod_verbose = if ascii_isdigit(*ea.cmd as c_int) {
                            atoi(ea.cmd) + 1
                        } else {
                            2
                        };
                        ea.cmd = p;
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }

        restore_visual_range(ea, orig_cmd, cmd_start, has_visual_range, use_plus_cmd);
        OK
    }
}

/// Put the `'<,'>` this scan stepped over back in front of the command.
///
/// The range has to end up immediately before the command word, so the
/// text between them — the modifiers — is shuffled left by the five bytes
/// the range occupies and the range is written into the gap. Ex mode's
/// substituted `+` command is not in the buffer at all, so it takes the
/// other branch and the range is prefixed rather than moved.
unsafe fn restore_visual_range(
    ea: &mut exarg_T,
    orig_cmd: *mut c_char,
    cmd_start: *mut c_char,
    has_visual_range: bool,
    use_plus_cmd: bool,
) {
    unsafe {
        if !has_visual_range {
            if use_plus_cmd {
                ea.cmd = exmode_plus.ptr() as *mut c_char;
            }
            return;
        }
        if ea.cmd > cmd_start {
            if use_plus_cmd {
                let len = strlen(cmd_start);
                memmove(orig_cmd as *mut c_void, cmd_start as *const c_void, len);
                xmemcpyz(orig_cmd.add(len) as *mut c_void, c" *+".as_ptr().cast(), 3);
            } else {
                memmove(
                    cmd_start.offset(-5) as *mut c_void,
                    cmd_start as *const c_void,
                    ea.cmd.offset_from(cmd_start) as size_t,
                );
                ea.cmd = ea.cmd.offset(-5);
                memmove(
                    ea.cmd.offset(-1) as *mut c_void,
                    c":'<,'>".as_ptr() as *const c_void,
                    6,
                );
            }
        } else if use_plus_cmd {
            ea.cmd = c"'<,'>+".as_ptr() as *mut c_char;
        } else {
            ea.cmd = orig_cmd;
        }
    }
}

/// Put the parsed modifiers in force. Every field this writes is saved
/// *plus one*, so that zero can mean "not saved" — `undo_cmdmod` relies on
/// it, and so does the fact that `apply_cmdmod` may run twice.
pub unsafe fn apply_cmdmod(cmod: *mut cmdmod_T) {
    unsafe {
        let cm = &mut *cmod;
        if cm.cmod_flags & CMOD_SANDBOX as c_int != 0 && cm.cmod_did_sandbox == 0 {
            *sandbox.ptr() += 1;
            cm.cmod_did_sandbox = 1;
        }
        if cm.cmod_verbose > 0 {
            if cm.cmod_verbose_save == 0 {
                cm.cmod_verbose_save = p_verbose.get() + 1;
            }
            p_verbose.set((cm.cmod_verbose - 1) as OptInt);
        }
        if cm.cmod_flags & (CMOD_SILENT as c_int | CMOD_UNSILENT as c_int) != 0
            && cm.cmod_save_msg_silent == 0
        {
            cm.cmod_save_msg_silent = msg_silent.get() + 1;
            cm.cmod_save_msg_scroll = msg_scroll.get();
        }
        if cm.cmod_flags & CMOD_SILENT as c_int != 0 {
            *msg_silent.ptr() += 1;
        }
        if cm.cmod_flags & CMOD_UNSILENT as c_int != 0 {
            msg_silent.set(0);
        }
        if cm.cmod_flags & CMOD_ERRSILENT as c_int != 0 {
            *emsg_silent.ptr() += 1;
            cm.cmod_did_esilent += 1;
        }
        if cm.cmod_flags & CMOD_NOAUTOCMD as c_int != 0 && cm.cmod_save_ei.is_null() {
            cm.cmod_save_ei = xstrdup(p_ei.get());
            set_option_direct(kOptEventignore, eventignore_all(), 0, SID_NONE);
        }
    }
}

/// The 'eventignore' value `:noautocmd` installs.
fn eventignore_all() -> OptVal {
    OptVal {
        type_0: kOptValTypeString,
        data: OptValData {
            string: String_0 {
                data: c"all".as_ptr() as *mut c_char,
                size: 3,
            },
        },
    }
}

/// Take the modifiers back out of force.
pub unsafe fn undo_cmdmod(cmod: *mut cmdmod_T) {
    unsafe {
        let cm = &mut *cmod;
        if cm.cmod_verbose_save > 0 {
            p_verbose.set(cm.cmod_verbose_save - 1);
            cm.cmod_verbose_save = 0;
        }
        if cm.cmod_did_sandbox != 0 {
            *sandbox.ptr() -= 1;
            cm.cmod_did_sandbox = 0;
        }
        if !cm.cmod_save_ei.is_null() {
            set_option_direct(
                kOptEventignore,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string(cm.cmod_save_ei),
                    },
                },
                0,
                SID_NONE,
            );
            free_string_option(cm.cmod_save_ei);
            cm.cmod_save_ei = ptr::null_mut();
        }
        xfree(cm.cmod_filter_pat as *mut c_void);
        vim_regfree(cm.cmod_filter_regmatch.regprog);
        if cm.cmod_save_msg_silent > 0 {
            // A command that raised an error may have wanted the message
            // level it left behind; only restore over it when nothing did.
            if did_emsg.get() == 0 || msg_silent.get() > cm.cmod_save_msg_silent - 1 {
                msg_silent.set(cm.cmod_save_msg_silent - 1);
            }
            *emsg_silent.ptr() -= cm.cmod_did_esilent;
            emsg_silent.set(emsg_silent.get().max(0));
            msg_scroll.set(cm.cmod_save_msg_scroll);
            if redirecting() {
                msg_col.set(0);
            }
            cm.cmod_save_msg_silent = 0;
            cm.cmod_did_esilent = 0;
        }
    }
}

/// How many bytes of `cmd` are a command modifier, or 0 if none are.
///
/// Used by the command-line completion to decide what the word after a
/// modifier should complete as.
pub unsafe fn modifier_len(cmd: *mut c_char) -> c_int {
    unsafe {
        // A count may precede a modifier, and only the two that accept one
        // match when it does.
        let p = if ascii_isdigit(*cmd as c_int) {
            skipwhite(skipdigits(cmd.add(1)))
        } else {
            cmd
        };
        for md in &CMDMODS {
            let j = shared_prefix(p, md.name);
            let after = *p.add(j) as u8;
            if j >= md.minlen && !after.is_ascii_alphabetic() && (p == cmd || md.has_count) {
                return j as c_int + p.offset_from(cmd) as c_int;
            }
        }
        0
    }
}

/// How many bytes of the NUL-terminated `p` match the start of `name`.
///
/// The walk is over `p`, not over `name`: it stops at the end of the
/// *typed* word, so a full name and an abbreviation both come back with
/// the length that was typed.
pub(crate) unsafe fn shared_prefix(p: *const c_char, name: &CStr) -> usize {
    unsafe {
        let name = name.to_bytes_with_nul();
        let mut j = 0usize;
        while *p.add(j) as c_int != NUL && *p.add(j) as u8 == name[j] {
            j += 1;
        }
        j
    }
}

/// Is an expression-driven mapping running, in a buffer the user can see?
///
/// The dummy buffer an expression mapping is evaluated in is exempt: the
/// lock is about the *user's* text.
pub unsafe fn expr_map_locked() -> bool {
    unsafe { expr_map_lock.get() > 0 && (*curbuf.get()).b_flags & BF_DUMMY == 0 }
}

/// Is this the location-list spelling of a quickfix command? Upstream tells
/// them apart by the leading `l` of the name and nothing else.
pub unsafe fn is_loclist_cmd(cmdidx: c_int) -> bool {
    if cmdidx < 0 || cmdidx >= CMD_SIZE as c_int {
        return false;
    }
    unsafe { *(*cmdnames.ptr())[cmdidx as usize].cmd_name as c_int == 'l' as c_int }
}

/// Is this one of the mapping commands? Asked by the argument scan, which
/// must not treat a `<expr>` mapping's right-hand side as an expression.
pub unsafe fn is_map_cmd(cmdidx: cmdidx_T) -> bool {
    if (cmdidx as c_int) < 0 {
        return false;
    }
    let func: ex_func_T = unsafe { (*cmdnames.ptr())[cmdidx as usize].cmd_func };
    ex_func_is(func, ex_map)
        || ex_func_is(func, ex_unmap)
        || ex_func_is(func, ex_mapclear)
        || ex_func_is(func, ex_abbreviate)
        || ex_func_is(func, ex_abclear)
}
