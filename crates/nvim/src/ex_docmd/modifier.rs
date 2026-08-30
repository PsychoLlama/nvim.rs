//! The command modifiers (`:silent`, `:verbose`, `:tab`,
//! `:keeppatterns`, the split direction, `:filter`, …): recognising them,
//! putting them in force around the command, and taking them out again.
//!
//! `parse_command_modifiers` only *fills in* a `cmdmod_T`; `apply_cmdmod`
//! is what puts it in force and `undo_cmdmod` what takes it back out, and
//! the two must stay a matched pair — `do_one_cmd` runs the second on
//! every exit path, including the ones an error takes.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::cstr;
use crate::ex_docmd::scan::ends_excmd;

use crate::winlayer::{Buf, Ea, Win};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;
use std::ffi::CString;

use crate::api::private::helpers::cstr_as_string;
use crate::ascii::{ascii_isdigit, ascii_iswhite};

use crate::buffer::BufFlags;
use crate::charset::skipdigits;

use crate::ex_docmd::address::{get_address, skip_range};

use crate::ex_docmd::onecmd::ex_func_is;

use crate::ex_docmd::source::getline_equal;
use crate::ex_docmd::window::current_tab_nr;
use crate::ex_docmd::{
    SID_NONE, cmdnames, e_invrange, ex_func_T, ex_msg, ex_pressedreturn, exmode_plus, getexline,
};
use crate::main::{
    cmdmod, curtab, did_emsg, emsg_silent, exmode_active, expr_map_lock, msg_col, msg_scroll,
    msg_silent, p_ei, p_verbose, sandbox,
};
use crate::mapping::{ex_abbreviate, ex_abclear, ex_map, ex_mapclear, ex_unmap};
use crate::memory::{xfree, xmemcpyz};

use crate::message::redirecting;
use crate::option::{kOptValTypeString, set_option_direct};
use crate::options::kOptEventignore;
use crate::optionstr::free_string_option;

use crate::pos::MAXLNUM;
use crate::regexp::{RE_MAGIC, vim_regcomp, vim_regexec, vim_regfree};
use crate::strings::vim_strchr;
use crate::types::{
    CMD_SIZE, CMD_echo, CMD_echoerr, CMD_echomsg, CMD_echon, CMD_execute, CmdAddr, CmdModFlags,
    Failed, NUL, OptInt, OptVal, OptValData, OptionSetFlags, String_0, cmdidx_T, cmdmod_T, exarg_T,
    size_t,
};
use crate::window::{WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_HOR, WSP_TOP, WSP_VERT, tabpage_index};
use ::libc::{atoi, strlen};

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
/// Answers `Err` when there is no command at all — a comment, a bare
/// newline, or an empty line — and `Ok` otherwise, including when the line
/// carried no modifier.
///
/// `skip_only` is `nvim_parse_cmd`'s mode: recognise everything, allocate
/// and evaluate nothing.
pub(crate) unsafe fn parse_command_modifiers(
    eap: *mut exarg_T,
    errormsg: &mut Option<CString>,
    cm: &mut cmdmod_T,
    skip_only: bool,
) -> Result<(), Failed> {
    let mut ea = unsafe { Ea::new(eap) };
    let orig_cmd = ea.cmd;
    let mut cmd_start: *mut c_char = ptr::null_mut();
    let mut use_plus_cmd = false;
    let mut has_visual_range = false;
    *cm = cmdmod_T::default();

    // A `'<,'>` typed by the user (which is what a Visual-mode `:` puts
    // there) is stepped over so a modifier after it is still seen, and
    // put back below — the range has to reach the command, not the
    // modifier scan.
    if unsafe { cstr::starts_with(ea.cmd, b"'<,'>") } {
        let p = unsafe { skipwhite(ea.cmd.add(5)) };
        if byte(p) != NUL && byte(p) != '|' as c_int {
            ea.cmd = unsafe { ea.cmd.add(5) };
            cmd_start = ea.cmd;
            has_visual_range = true;
        }
    }

    loop {
        while byte(ea.cmd) == ' ' as c_int
            || byte(ea.cmd) == '\t' as c_int
            || byte(ea.cmd) == ':' as c_int
        {
            ea.cmd = unsafe { ea.cmd.add(1) };
        }

        // In Ex mode an empty line means "print the next one", which is
        // spelled by substituting a `+` command.
        if byte(ea.cmd) == NUL
            && exmode_active.get()
            && unsafe { getline_equal(ea.ea_getline, ea.cookie, Some(getexline)) }
            && cur_win().w_cursor.lnum < cur_buf().b_ml.ml_line_count
        {
            ea.cmd = exmode_plus.as_ptr().cast_mut();
            use_plus_cmd = true;
            if !skip_only {
                ex_pressedreturn.set(true);
            }
            break;
        }
        if byte(ea.cmd) == '"' as c_int {
            ea.nextcmd = unsafe { vim_strchr(ea.cmd, '\n' as c_int) };
            if !ea.nextcmd.is_null() {
                ea.nextcmd = unsafe { ea.nextcmd.add(1) };
            }
            return Err(Failed);
        }
        if byte(ea.cmd) == '\n' as c_int {
            ea.nextcmd = unsafe { ea.cmd.add(1) };
            return Err(Failed);
        }
        if byte(ea.cmd) == NUL {
            if !skip_only {
                ex_pressedreturn.set(true);
            }
            return Err(Failed);
        }

        // A modifier may follow a range (`:1,2 silent print`), so the
        // name is looked for past one — but `eap->cmd` only moves for
        // the modifiers that accept that.
        let mut p = unsafe { skip_range(ea.cmd, ptr::null_mut()) };
        match ubyte(p) {
            b'a' => {
                if !checkforcmd(ea.cmd_ptr(), c"aboveleft".as_ptr(), 3) {
                    break;
                }
                cm.cmod_split |= WSP_ABOVE as c_int;
            }
            b'b' => {
                if checkforcmd(ea.cmd_ptr(), c"belowright".as_ptr(), 3) {
                    cm.cmod_split |= WSP_BELOW as c_int;
                } else if checkforcmd(ea.cmd_ptr(), c"browse".as_ptr(), 3) {
                    cm.cmod_flags |= CmdModFlags::BROWSE;
                } else if checkforcmd(ea.cmd_ptr(), c"botright".as_ptr(), 2) {
                    cm.cmod_split |= WSP_BOT as c_int;
                } else {
                    break;
                }
            }
            b'c' => {
                if !checkforcmd(ea.cmd_ptr(), c"confirm".as_ptr(), 4) {
                    break;
                }
                cm.cmod_flags |= CmdModFlags::CONFIRM;
            }
            b'k' => {
                if checkforcmd(ea.cmd_ptr(), c"keepmarks".as_ptr(), 3) {
                    cm.cmod_flags |= CmdModFlags::KEEPMARKS;
                } else if checkforcmd(ea.cmd_ptr(), c"keepalt".as_ptr(), 5) {
                    cm.cmod_flags |= CmdModFlags::KEEPALT;
                } else if checkforcmd(ea.cmd_ptr(), c"keeppatterns".as_ptr(), 5) {
                    cm.cmod_flags |= CmdModFlags::KEEPPATTERNS;
                } else if checkforcmd(ea.cmd_ptr(), c"keepjumps".as_ptr(), 5) {
                    cm.cmod_flags |= CmdModFlags::KEEPJUMPS;
                } else {
                    break;
                }
            }
            b'f' => {
                // `:filter` insists on a pattern *and* something after
                // it: the whole point is the command it wraps.
                let mut reg_pat: *mut c_char = ptr::null_mut();
                if !checkforcmd(&raw mut p, c"filter".as_ptr(), 4)
                    || byte(p) == NUL
                    || ends_excmd(byte(p)) != 0
                {
                    break;
                }
                if byte(p) == '!' as c_int {
                    cm.cmod_filter_force = true;
                    p = unsafe { skipwhite(p.add(1)) };
                    if byte(p) == NUL || ends_excmd(byte(p)) != 0 {
                        break;
                    }
                }
                p = if skip_only {
                    skip_vimgrep_pat(p, ptr::null_mut(), ptr::null_mut())
                } else {
                    skip_vimgrep_pat(p, &raw mut reg_pat, ptr::null_mut())
                };
                if p.is_null() || byte(p) == NUL {
                    break;
                }
                if !skip_only {
                    cm.cmod_filter_pat = xstrdup(reg_pat);
                    cm.cmod_filter_regmatch.regprog = unsafe { vim_regcomp(reg_pat, RE_MAGIC) };
                    if cm.cmod_filter_regmatch.regprog.is_null() {
                        break;
                    }
                }
                ea.cmd = p;
            }
            b'h' => {
                if checkforcmd(ea.cmd_ptr(), c"horizontal".as_ptr(), 3) {
                    cm.cmod_split |= WSP_HOR as c_int;
                } else if p == ea.cmd
                    && checkforcmd(&raw mut p, c"hide".as_ptr(), 3)
                    && byte(p) != NUL
                    && ends_excmd(byte(p)) == 0
                {
                    // `:hide` is a command in its own right, so it is
                    // only a modifier when a command follows it and no
                    // range precedes it.
                    ea.cmd = p;
                    cm.cmod_flags |= CmdModFlags::HIDE;
                } else {
                    break;
                }
            }
            b'l' => {
                if checkforcmd(ea.cmd_ptr(), c"lockmarks".as_ptr(), 3) {
                    cm.cmod_flags |= CmdModFlags::LOCKMARKS;
                } else if checkforcmd(ea.cmd_ptr(), c"leftabove".as_ptr(), 5) {
                    cm.cmod_split |= WSP_ABOVE as c_int;
                } else {
                    break;
                }
            }
            b'n' => {
                if checkforcmd(ea.cmd_ptr(), c"noautocmd".as_ptr(), 3) {
                    cm.cmod_flags |= CmdModFlags::NOAUTOCMD;
                } else if checkforcmd(ea.cmd_ptr(), c"noswapfile".as_ptr(), 3) {
                    cm.cmod_flags |= CmdModFlags::NOSWAPFILE;
                } else {
                    break;
                }
            }
            b'r' => {
                if !checkforcmd(ea.cmd_ptr(), c"rightbelow".as_ptr(), 6) {
                    break;
                }
                cm.cmod_split |= WSP_BELOW as c_int;
            }
            b's' => {
                if checkforcmd(ea.cmd_ptr(), c"sandbox".as_ptr(), 3) {
                    cm.cmod_flags |= CmdModFlags::SANDBOX;
                } else if checkforcmd(ea.cmd_ptr(), c"silent".as_ptr(), 3) {
                    cm.cmod_flags |= CmdModFlags::SILENT;
                    // `:silent!` only means "and silence errors" when
                    // the `!` is stuck to the word: `:silent !cmd` runs
                    // a shell command quietly.
                    if byte(ea.cmd) == '!' as c_int && !ascii_iswhite(byte_at(ea.cmd, -1)) {
                        ea.cmd = unsafe { skipwhite(ea.cmd.add(1)) };
                        cm.cmod_flags |= CmdModFlags::ERRSILENT;
                    }
                } else {
                    break;
                }
            }
            b't' => {
                if checkforcmd(&raw mut p, c"tab".as_ptr(), 3) {
                    if !skip_only {
                        let tabnr = unsafe {
                            get_address(
                                eap,
                                ea.cmd_ptr(),
                                CmdAddr::Tabs,
                                ea.skip != 0,
                                skip_only,
                                0,
                                1,
                                errormsg,
                            )
                        } as c_int;
                        if ea.cmd.is_null() {
                            return Err(Failed);
                        }
                        if tabnr == MAXLNUM as c_int {
                            cm.cmod_tab = tabpage_index(curtab.get()) + 1;
                        } else {
                            if tabnr < 0 || tabnr > current_tab_nr(ptr::null_mut()) {
                                *errormsg = Some(unsafe { ex_msg(e_invrange.as_ptr()) });
                                return Err(Failed);
                            }
                            cm.cmod_tab = tabnr + 1;
                        }
                    }
                    ea.cmd = p;
                } else if checkforcmd(ea.cmd_ptr(), c"topleft".as_ptr(), 2) {
                    cm.cmod_split |= WSP_TOP as c_int;
                } else {
                    break;
                }
            }
            b'u' => {
                if !checkforcmd(ea.cmd_ptr(), c"unsilent".as_ptr(), 3) {
                    break;
                }
                cm.cmod_flags |= CmdModFlags::UNSILENT;
            }
            b'v' => {
                if checkforcmd(ea.cmd_ptr(), c"vertical".as_ptr(), 4) {
                    cm.cmod_split |= WSP_VERT as c_int;
                } else if checkforcmd(&raw mut p, c"verbose".as_ptr(), 4) {
                    // The count is read from `eap->cmd`, which
                    // `checkforcmd` left *before* the word: `:5verbose`.
                    // Saturating: the count is whatever the user typed,
                    // so `:2147483647verbose set` would otherwise add one
                    // to `INT_MAX` and end the process.  C wraps here.
                    cm.cmod_verbose = if ascii_isdigit(byte(ea.cmd)) {
                        unsafe { atoi(ea.cmd) }.saturating_add(1)
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

    unsafe { restore_visual_range(ea, orig_cmd, cmd_start, has_visual_range, use_plus_cmd) };
    Ok(())
}

/// Put the `'<,'>` this scan stepped over back in front of the command.
///
/// The range has to end up immediately before the command word, so the
/// text between them — the modifiers — is shuffled left by the five bytes
/// the range occupies and the range is written into the gap. Ex mode's
/// substituted `+` command is not in the buffer at all, so it takes the
/// other branch and the range is prefixed rather than moved.
unsafe fn restore_visual_range(
    mut ea: Ea,
    orig_cmd: *mut c_char,
    cmd_start: *mut c_char,
    has_visual_range: bool,
    use_plus_cmd: bool,
) {
    if !has_visual_range {
        if use_plus_cmd {
            ea.cmd = exmode_plus.as_ptr().cast_mut();
        }
        return;
    }
    if ea.cmd > cmd_start {
        if use_plus_cmd {
            let len = unsafe { strlen(cmd_start) };
            memmove(orig_cmd as *mut c_void, cmd_start as *const c_void, len);
            unsafe { xmemcpyz(orig_cmd.add(len) as *mut c_void, c" *+".as_ptr().cast(), 3) };
        } else {
            unsafe {
                memmove(
                    cmd_start.offset(-5) as *mut c_void,
                    cmd_start as *const c_void,
                    ea.cmd.offset_from(cmd_start) as size_t,
                )
            };
            ea.cmd = unsafe { ea.cmd.offset(-5) };
            unsafe {
                memmove(
                    ea.cmd.offset(-1) as *mut c_void,
                    c":'<,'>".as_ptr() as *const c_void,
                    6,
                )
            };
        }
    } else if use_plus_cmd {
        ea.cmd = c"'<,'>+".as_ptr() as *mut c_char;
    } else {
        ea.cmd = orig_cmd;
    }
}

/// Whether the running command carries any of `flags` as a `:` modifier.
///
/// The question every caller used to ask by dereferencing the cell and
/// masking `cmod_flags` by hand, for a read that needs no pointer at all.
pub(crate) fn cmdmod_has(flags: CmdModFlags) -> bool {
    cmdmod_flags().has(flags)
}

/// The whole flag set the running command carries.
pub(crate) fn cmdmod_flags() -> CmdModFlags {
    cmdmod.with(|mods| mods.cmod_flags)
}

/// Replace the flag set — for the two scopes that save it, force a flag on
/// for a stretch of their own, and put the old set back by hand.
pub(crate) fn cmdmod_set_flags(flags: CmdModFlags) {
    cmdmod.with_mut(|mods| mods.cmod_flags = flags);
}

/// Force `flags` on for the rest of the command, as `:lockmarks` in front
/// of it would have.
pub(crate) fn cmdmod_add_flags(flags: CmdModFlags) {
    cmdmod.with_mut(|mods| mods.cmod_flags |= flags);
}

/// `:tab`'s argument: the 1-based tab page a new window goes to, or 0 for
/// "no `:tab` was used".
pub(crate) fn cmdmod_tab() -> c_int {
    cmdmod.with(|mods| mods.cmod_tab)
}

/// See [`cmdmod_tab`].
pub(crate) fn cmdmod_set_tab(tab: c_int) {
    cmdmod.with_mut(|mods| mods.cmod_tab = tab);
}

/// The `WSP_*` bits `:aboveleft`, `:vertical` and friends asked for.
pub(crate) fn cmdmod_split() -> c_int {
    cmdmod.with(|mods| mods.cmod_split)
}

/// Force `bits` on in the split direction, as a `:vertical` in front of the
/// command would have.
pub(crate) fn cmdmod_add_split(bits: c_int) {
    cmdmod.with_mut(|mods| mods.cmod_split |= bits);
}

/// See [`cmdmod_split`].
pub(crate) fn cmdmod_set_split(split: c_int) {
    cmdmod.with_mut(|mods| mods.cmod_split = split);
}

/// Everything `<mods>` and `smods` report, read out in one go: the four
/// scalars a caller rendering the modifier set needs.
pub(crate) fn cmdmod_report() -> (c_int, c_int, c_int, CmdModFlags) {
    cmdmod.with(|mods| {
        (
            mods.cmod_tab,
            mods.cmod_verbose,
            mods.cmod_split,
            mods.cmod_flags,
        )
    })
}

/// Whether `:filter pattern` is in force and `msg` does not match it.
///
/// The program is taken out of the cell and put back rather than matched in
/// place: the engines may *replace* it (the NFA one falls back to the
/// backtracking one and frees what it had), and a `\=` inside the pattern
/// re-enters the editor, so no borrow may be held across the match.
///
/// # Safety
/// `msg` is NUL-terminated, and this is a main-thread editor call.
pub(crate) unsafe fn cmdmod_filters_out(msg: *const c_char) -> bool {
    let mut regmatch = cmdmod.with(|mods| mods.cmod_filter_regmatch.clone());
    if regmatch.regprog.is_null() {
        return false;
    }
    // SAFETY: the caller's contract; `regmatch` holds this command's
    // `:filter` program.
    let matched = unsafe { vim_regexec(&raw mut regmatch, msg, 0) };
    cmdmod.with_mut(|mods| mods.cmod_filter_regmatch = regmatch);
    if cmdmod.with(|mods| mods.cmod_filter_force) {
        matched
    } else {
        !matched
    }
}

/// Put the modifiers now in the cell in force. Every field this writes is
/// saved *plus one*, so that zero can mean "not saved" — `undo_cmdmod`
/// relies on it, and so does the fact that this may run twice.
///
/// The writes go back into the cell one at a time rather than through a
/// borrow held across the body: the two calls out (`xstrdup` and
/// `set_option_direct`) re-enter the editor, and `set_option_direct` runs
/// with the new modifiers already in force, exactly as the C leaves them.
///
/// # Safety
/// Main-thread editor call. `set_option_direct` runs `'eventignore'`'s
/// side effects.
unsafe fn apply_cmdmod() {
    let mods = cmdmod.with(|cm| cm.cmod_flags);
    if mods.has(CmdModFlags::SANDBOX) && cmdmod.with(|cm| cm.cmod_did_sandbox) == 0 {
        sandbox.set(sandbox.get() + 1);
        cmdmod.with_mut(|cm| cm.cmod_did_sandbox = 1);
    }
    let verbose = cmdmod.with(|cm| cm.cmod_verbose);
    if verbose > 0 {
        if cmdmod.with(|cm| cm.cmod_verbose_save) == 0 {
            let save = p_verbose.get() + 1;
            cmdmod.with_mut(|cm| cm.cmod_verbose_save = save);
        }
        p_verbose.set((verbose - 1) as OptInt);
    }
    if mods.has(CmdModFlags::SILENT | CmdModFlags::UNSILENT)
        && cmdmod.with(|cm| cm.cmod_save_msg_silent) == 0
    {
        let (silent, scroll) = (msg_silent.get() + 1, msg_scroll.get());
        cmdmod.with_mut(|cm| {
            cm.cmod_save_msg_silent = silent;
            cm.cmod_save_msg_scroll = scroll;
        });
    }
    if mods.has(CmdModFlags::SILENT) {
        msg_silent.set(msg_silent.get() + 1);
    }
    if mods.has(CmdModFlags::UNSILENT) {
        msg_silent.set(0);
    }
    if mods.has(CmdModFlags::ERRSILENT) {
        emsg_silent.set(emsg_silent.get() + 1);
        cmdmod.with_mut(|cm| cm.cmod_did_esilent += 1);
    }
    if mods.has(CmdModFlags::NOAUTOCMD) && cmdmod.with(|cm| cm.cmod_save_ei).is_null() {
        // SAFETY: `p_ei` is the live `'eventignore'` string.
        let save_ei = xstrdup(p_ei.get());
        cmdmod.with_mut(|cm| cm.cmod_save_ei = save_ei);
        set_option_direct(
            kOptEventignore,
            eventignore_all(),
            OptionSetFlags::NONE,
            SID_NONE,
        );
    }
}

/// The 'eventignore' value `:noautocmd` installs.
fn eventignore_all() -> OptVal {
    OptVal {
        type_0: kOptValTypeString,
        data: OptValData {
            string: String_0::from_raw_parts(c"all".as_ptr() as *mut c_char, 3),
        },
    }
}

/// Take the modifiers back out of force.
pub(crate) unsafe fn undo_cmdmod(cm: &mut cmdmod_T) {
    if cm.cmod_verbose_save > 0 {
        p_verbose.set(cm.cmod_verbose_save - 1);
        cm.cmod_verbose_save = 0;
    }
    if cm.cmod_did_sandbox != 0 {
        sandbox.set(sandbox.get() - 1);
        cm.cmod_did_sandbox = 0;
    }
    if !cm.cmod_save_ei.is_null() {
        set_option_direct(
            kOptEventignore,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: unsafe { cstr_as_string(cm.cmod_save_ei) },
                },
            },
            OptionSetFlags::NONE,
            SID_NONE,
        );
        unsafe { free_string_option(cm.cmod_save_ei) };
        cm.cmod_save_ei = ptr::null_mut();
    }
    unsafe { xfree(cm.cmod_filter_pat as *mut c_void) };
    unsafe { vim_regfree(cm.cmod_filter_regmatch.regprog) };
    if cm.cmod_save_msg_silent > 0 {
        // A command that raised an error may have wanted the message
        // level it left behind; only restore over it when nothing did.
        if did_emsg.get() == 0 || msg_silent.get() > cm.cmod_save_msg_silent - 1 {
            msg_silent.set(cm.cmod_save_msg_silent - 1);
        }
        emsg_silent.set((emsg_silent.get() - cm.cmod_did_esilent).max(0));
        msg_scroll.set(cm.cmod_save_msg_scroll);
        if unsafe { redirecting() } {
            msg_col.set(0);
        }
        cm.cmod_save_msg_silent = 0;
        cm.cmod_did_esilent = 0;
    }
}

/// The modifiers in force for a scope, taken back out when it ends.
///
/// The hand-rolled shape this replaces is four statements far apart — copy
/// the cell, overwrite it, `apply_cmdmod`, and much later `undo_cmdmod`
/// plus copy the old set back — and `do_one_cmd` alone has a dozen ways out
/// between the halves. Here the second half is `Drop`'s, so an early return
/// or a panic can no longer leave a `:silent` or a `:noautocmd` switched on
/// for the rest of the session.
///
/// The restore is ordered: `undo_cmdmod` runs against the modifiers *still
/// in the cell* — a command may have changed them, `:tag` and `:help` both
/// do — and only then does the previous set go back.
#[must_use = "the modifiers are taken back out as soon as the guard is dropped"]
pub(crate) struct CmdModScope {
    saved: cmdmod_T,
}

impl CmdModScope {
    /// Save the modifiers in force and clear the cell, for the caller that
    /// parses the new set straight into it. This is what the C's copy plus
    /// `parse_command_modifiers`'s opening `CLEAR_FIELD` amount to.
    ///
    /// Clearing *here* rather than inside the parse is the one place this
    /// diverges from upstream, and it is a crash fix: `do_one_cmd` bails to
    /// its exit path on a `#!` shebang line before it ever parses, so
    /// upstream's `undo_cmdmod` there runs against the *enclosing* command's
    /// modifiers -- ending its `:silent` early, and freeing the `:filter`
    /// pattern and program that the enclosing frame goes on to free again.
    /// `:filter /x/ source file-starting-with-#!` is a double free upstream.
    /// See `1786212071-upstream-neovim-bugs`.
    pub(crate) fn cleared() -> Self {
        CmdModScope {
            saved: cmdmod.take(),
        }
    }

    /// Save the modifiers in force and put `mods` in force instead.
    ///
    /// # Safety
    /// Main-thread editor call: `apply_cmdmod` sets `'eventignore'`.
    pub(crate) unsafe fn enter(mods: cmdmod_T) -> Self {
        let scope = CmdModScope {
            saved: cmdmod.take(),
        };
        cmdmod.set(mods);
        // SAFETY: the caller's contract.
        unsafe { apply_cmdmod() };
        scope
    }

    /// Read the run of modifiers at the head of `eap`'s command line into
    /// the cell, which stays *cleared* for the whole parse — an error
    /// raised while the modifiers are still being read is no longer inside
    /// the enclosing `:silent` or `:filter`, which is what the C's opening
    /// `CLEAR_FIELD(cmdmod)` amounts to.
    ///
    /// # Safety
    /// As [`parse_command_modifiers`].
    pub(crate) unsafe fn parse(
        &self,
        eap: *mut exarg_T,
        errormsg: &mut Option<CString>,
    ) -> Result<(), Failed> {
        let mut parsed = cmdmod_T::default();
        // SAFETY: the caller's contract.
        let read = unsafe { parse_command_modifiers(eap, errormsg, &mut parsed, false) };
        cmdmod.set(parsed);
        read
    }

    /// Put the parsed modifiers in force, once [`CmdModScope::parse`] has
    /// stored them.
    ///
    /// # Safety
    /// As [`CmdModScope::enter`].
    pub(crate) unsafe fn apply(&self) {
        // SAFETY: the caller's contract.
        unsafe { apply_cmdmod() };
    }
}

impl Drop for CmdModScope {
    fn drop(&mut self) {
        // The clone shares the `:filter` pattern and program with the cell,
        // which is what lets `undo_cmdmod` free them while the cell still
        // answers `message_filtered` for anything it says on the way out.
        let mut live = cmdmod.with(Clone::clone);
        // SAFETY: `live` names the modifiers this guard put in force.
        unsafe { undo_cmdmod(&mut live) };
        cmdmod.set(core::mem::take(&mut self.saved));
    }
}

/// How many bytes of `cmd` are a command modifier, or 0 if none are.
///
/// Used by the command-line completion to decide what the word after a
/// modifier should complete as.
pub unsafe fn modifier_len(cmd: *mut c_char) -> c_int {
    // A count may precede a modifier, and only the two that accept one
    // match when it does.
    let p = if ascii_isdigit(byte(cmd)) {
        unsafe { skipwhite(skipdigits(cmd.add(1))) }
    } else {
        cmd
    };
    for md in &CMDMODS {
        let j = unsafe { shared_prefix(p, md.name) };
        let after = ubyte_at(p, j as isize);
        if j >= md.minlen && !after.is_ascii_alphabetic() && (p == cmd || md.has_count) {
            return j as c_int + unsafe { p.offset_from(cmd) } as c_int;
        }
    }
    0
}

/// How many bytes of the NUL-terminated `p` match the start of `name`.
///
/// The walk is over `p`, not over `name`: it stops at the end of the
/// *typed* word, so a full name and an abbreviation both come back with
/// the length that was typed.
pub(crate) unsafe fn shared_prefix(p: *const c_char, name: &CStr) -> usize {
    let name = name.to_bytes_with_nul();
    let mut j = 0usize;
    while byte_at(p, j as isize) != NUL && ubyte_at(p, j as isize) == name[j] {
        j += 1;
    }
    j
}

/// Is an expression-driven mapping running, in a buffer the user can see?
///
/// The dummy buffer an expression mapping is evaluated in is exempt: the
/// lock is about the *user's* text.
pub fn expr_map_locked() -> bool {
    // SAFETY: `curbuf` is a live buffer whenever a mapping can be running.
    expr_map_lock.get() > 0 && !cur_buf().b_flags.has(BufFlags::DUMMY)
}

/// Is this the location-list spelling of a quickfix command? Upstream tells
/// them apart by the leading `l` of the name and nothing else.
pub unsafe fn is_loclist_cmd(cmdidx: c_int) -> bool {
    if cmdidx < 0 || cmdidx >= CMD_SIZE as c_int {
        return false;
    }
    unsafe { *cmdnames[cmdidx as usize].cmd_name as c_int == 'l' as c_int }
}

/// Is this one of the mapping commands? Asked by the argument scan, which
/// must not treat a `<expr>` mapping's right-hand side as an expression.
pub unsafe fn is_map_cmd(cmdidx: cmdidx_T) -> bool {
    if (cmdidx as c_int) < 0 {
        return false;
    }
    let func: ex_func_T = cmdnames[cmdidx as usize].cmd_func;
    ex_func_is(func, ex_map)
        || ex_func_is(func, ex_unmap)
        || ex_func_is(func, ex_mapclear)
        || ex_func_is(func, ex_abbreviate)
        || ex_func_is(func, ex_abclear)
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// `checkforcmd()` as checked code.
fn checkforcmd(pp: *mut *mut c_char, cmd: *const c_char, len: c_int) -> bool {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::ex_docmd::lookup::checkforcmd(pp, cmd, len) }
}

/// `memmove()` as checked code.
fn memmove(
    __dest: *mut ::core::ffi::c_void,
    __src: *const ::core::ffi::c_void,
    __n: size_t,
) -> *mut ::core::ffi::c_void {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::os::cshim::memmove(__dest, __src, __n) }
}

/// `skip_vimgrep_pat()` as checked code.
fn skip_vimgrep_pat(
    p: *mut ::core::ffi::c_char,
    s: *mut *mut ::core::ffi::c_char,
    flags: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::ex_cmds::skip_vimgrep_pat(p, s, flags) }
}

/// `skipwhite()` as checked code.
fn skipwhite(p: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated string.
    unsafe { crate::charset::skipwhite(p) }
}

/// `xstrdup()` as checked code.
fn xstrdup(str: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated string.
    unsafe { crate::memory::xstrdup(str) }
}

/// The byte `p` points at, as the C's `*p` reads it.
fn byte(p: *const c_char) -> c_int {
    // SAFETY: a NUL-terminated string the command line owns.
    unsafe { *p as c_int }
}

/// The byte `p` points at, unsigned, as the C's `(uint8_t)*p` reads it.
fn ubyte(p: *const c_char) -> u8 {
    // SAFETY: a NUL-terminated string the command line owns.
    unsafe { *p as u8 }
}

/// The byte at `p[i]`, as the C's `*(p + i)` reads it.
fn byte_at(p: *const c_char, i: isize) -> c_int {
    // SAFETY: an offset within the NUL-terminated string `p` points into.
    unsafe { *p.offset(i) as c_int }
}

/// The byte at `p[i]`, unsigned, as the C's `(uint8_t)*(p + i)` reads it.
fn ubyte_at(p: *const c_char, i: isize) -> u8 {
    // SAFETY: an offset within the NUL-terminated string `p` points into.
    unsafe { *p.offset(i) as u8 }
}
