//! The command modifiers (`:silent`, `:verbose`, `:tab`,
//! `:keeppatterns`, the split direction, `:filter`, …): recognising them,
//! putting them in force around the command, and taking them out again.
//!
//! `parse_command_modifiers` only *fills in* a `CmdMod`; `apply_cmdmod`
//! is what puts it in force and `undo_cmdmod` what takes it back out, and
//! the two must stay a matched pair — `do_one_cmd` runs the second on
//! every exit path, including the ones an error takes.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
use crate::cstr;
use crate::ex_docmd::is_user_cmd;
use crate::ex_docmd::lookup::check_for_word;
use crate::ex_docmd::scan::ends_excmd;
use crate::types::CmdIdx;
use crate::types::OptStr;
use crate::window::tab_index;
use crate::winlayer::TabPage;

use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;
use std::ffi::CString;

use crate::ascii::{ascii_isdigit, ascii_iswhite};

use crate::buffer::BufFlags;
use crate::charset::skipdigits;

use crate::ex_docmd::address::{get_address, skip_range};

use crate::ex_docmd::onecmd::ex_func_is;

use crate::ex_docmd::source::getline_equal;
use crate::ex_docmd::state::cmdmod;
use crate::ex_docmd::window::current_tab_nr;
use crate::ex_docmd::{
    ExFunc, SID_NONE, cmdnames, e_invrange, ex_msg, ex_pressedreturn, getexline,
};
use crate::getchar::state::expr_map_lock;
use crate::guard::sandbox;
use crate::mapping::{ex_abbreviate, ex_abclear, ex_map, ex_mapclear, ex_unmap};
use crate::memory::{xfree, xmemcpyz};
use crate::message::state::{did_emsg, emsg_silent, msg_col, msg_scroll, msg_silent};
use crate::option::vars::{P_EI, P_VERBOSE, p_verbose};
use crate::state::mode::exmode_active;

use crate::message::redirecting;
use crate::option::set_option_direct;
use crate::options::kOptEventignore;
use crate::optionstr::free_string_option;

use crate::pos::MAXLNUM;
use crate::regexp::{RE_MAGIC, vim_regcomp, vim_regexec, vim_regfree};
use crate::types::{
    CmdAddr, CmdMod, CmdModFlags, ExArg, Failed, NUL, OptInt, OptVal, OptionSetFlags, size_t,
};
use crate::window::{WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_HOR, WSP_TOP, WSP_VERT};
use ::libc::atoi;

/// One recognised modifier name, for the two callers that only need to know
/// *whether* a word is one: `modifier_len` and `cmd_exists`.
pub(crate) struct CmdModSpec {
    pub(crate) name: &'static CStr,
    pub(crate) minlen: usize,
    /// Whether a count may precede this modifier (`:3tab`, `:5verbose`).
    pub(crate) has_count: bool,
}

const fn m(name: &'static CStr, minlen: usize, has_count: bool) -> CmdModSpec {
    CmdModSpec {
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
pub(crate) static CMDMODS: [CmdModSpec; 24] = [
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
pub fn cmd_has_expr_args(cmdidx: CmdIdx) -> bool {
    matches!(
        cmdidx,
        CmdIdx::execute | CmdIdx::echo | CmdIdx::echon | CmdIdx::echomsg | CmdIdx::echoerr
    )
}

/// Read the run of modifiers at the head of the command line into `cmod`,
/// advancing `args.cmd` past them.
///
/// Answers `Err` when there is no command at all — a comment, a bare
/// newline, or an empty line — and `Ok` otherwise, including when the line
/// carried no modifier.
///
/// `skip_only` is `nvim_parse_cmd`'s mode: recognise everything, allocate
/// and evaluate nothing.
pub(crate) fn parse_command_modifiers(
    excmd: &mut ExArg,
    errormsg: &mut Option<CString>,
    cm: &mut CmdMod,
    skip_only: bool,
) -> Result<(), Failed> {
    let orig_cmd = excmd.cmd_ptr();
    let mut cmd_start: *mut c_char = ptr::null_mut();
    let mut use_plus_cmd = false;
    let mut has_visual_range = false;
    *cm = CmdMod::default();

    // A `'<,'>` typed by the user (which is what a Visual-mode `:` puts
    // there) is stepped over so a modifier after it is still seen, and
    // put back below — the range has to reach the command, not the
    // modifier scan.
    if excmd.line.cmd().starts_with(b"'<,'>") {
        let after = excmd.line.skip_white(excmd.line.cmd + 5);
        if excmd.line.byte_at(after) != 0 && excmd.line.byte_at(after) != b'|' {
            excmd.line.cmd += 5;
            cmd_start = excmd.cmd_ptr();
            has_visual_range = true;
        }
    }

    loop {
        while matches!(excmd.line.byte_at(excmd.line.cmd), b' ' | b'\t' | b':') {
            excmd.line.cmd += 1;
        }

        // In Ex mode an empty line means "print the next one", which is
        // spelled by substituting a `+` command.
        if excmd.line.byte_at(excmd.line.cmd) == 0
            && exmode_active.get()
            && unsafe { getline_equal(excmd.ea_getline, excmd.cookie, Some(getexline)) }
            && Win::current().w_cursor.lnum < Buf::current().b_ml.ml_line_count
        {
            // An Ex-mode empty line *becomes* a `+` command: the line the
            // user did not type is retired and this one takes its place.
            excmd.line.take_over(b"+\0".to_vec());
            excmd.line.cmd = 0;
            excmd.line.substituted = true;
            use_plus_cmd = true;
            if !skip_only {
                ex_pressedreturn.set(true);
            }
            break;
        }
        if excmd.line.byte_at(excmd.line.cmd) == b'"' {
            // The comment runs to the newline; whatever follows it is the
            // next command.
            let cmd = excmd.line.cmd;
            excmd.line.next = excmd.line.rest_of(cmd).iter().position(|b| *b == b'\n');
            excmd.line.next = excmd.line.next.map(|at| cmd + at + 1);
            return Err(Failed);
        }
        if excmd.line.byte_at(excmd.line.cmd) == b'\n' {
            excmd.line.next = Some(excmd.line.cmd + 1);
            return Err(Failed);
        }
        if excmd.line.byte_at(excmd.line.cmd) == 0 {
            if !skip_only {
                ex_pressedreturn.set(true);
            }
            return Err(Failed);
        }

        // A modifier may follow a range (`:1,2 silent print`), so the
        // name is looked for past one — but `args.cmd` only moves for
        // the modifiers that accept that.
        let cmd = excmd.cmd_ptr();
        // SAFETY: the command's own NUL-terminated line, and a null context
        // is "not completing".
        let skipped = unsafe { skip_range(cmd, ptr::null_mut()) };
        let mut at = excmd.line.offset_of(skipped);
        match excmd.line.byte_at(at) {
            b'a' => {
                if !takes(excmd, b"aboveleft", 3) {
                    break;
                }
                cm.cmod_split |= WSP_ABOVE as c_int;
            }
            b'b' => {
                if takes(excmd, b"belowright", 3) {
                    cm.cmod_split |= WSP_BELOW as c_int;
                } else if takes(excmd, b"browse", 3) {
                    cm.cmod_flags |= CmdModFlags::BROWSE;
                } else if takes(excmd, b"botright", 2) {
                    cm.cmod_split |= WSP_BOT as c_int;
                } else {
                    break;
                }
            }
            b'c' => {
                if !takes(excmd, b"confirm", 4) {
                    break;
                }
                cm.cmod_flags |= CmdModFlags::CONFIRM;
            }
            b'k' => {
                if takes(excmd, b"keepmarks", 3) {
                    cm.cmod_flags |= CmdModFlags::KEEPMARKS;
                } else if takes(excmd, b"keepalt", 5) {
                    cm.cmod_flags |= CmdModFlags::KEEPALT;
                } else if takes(excmd, b"keeppatterns", 5) {
                    cm.cmod_flags |= CmdModFlags::KEEPPATTERNS;
                } else if takes(excmd, b"keepjumps", 5) {
                    cm.cmod_flags |= CmdModFlags::KEEPJUMPS;
                } else {
                    break;
                }
            }
            b'f' => {
                // `:filter` insists on a pattern *and* something after
                // it: the whole point is the command it wraps.
                let mut reg_pat: *mut c_char = ptr::null_mut();
                let Some(after) = check_for_word(&excmd.line, at, b"filter", 4) else {
                    break;
                };
                at = after;
                if excmd.line.byte_at(at) == 0
                    || ends_excmd(c_int::from(excmd.line.byte_at(at))) != 0
                {
                    break;
                }
                if excmd.line.byte_at(at) == b'!' {
                    cm.cmod_filter_force = true;
                    at = excmd.line.skip_white(at + 1);
                    if excmd.line.byte_at(at) == 0
                        || ends_excmd(c_int::from(excmd.line.byte_at(at))) != 0
                    {
                        break;
                    }
                }
                let start = excmd.line.ptr_at(at);
                let past = if skip_only {
                    skip_vimgrep_pat(start, ptr::null_mut(), ptr::null_mut())
                } else {
                    skip_vimgrep_pat(start, &raw mut reg_pat, ptr::null_mut())
                };
                if past.is_null() {
                    break;
                }
                at = excmd.line.offset_of(past);
                if excmd.line.byte_at(at) == 0 {
                    break;
                }
                if !skip_only {
                    cm.cmod_filter_pat = xstrdup(reg_pat);
                    cm.cmod_filter_regmatch.regprog = unsafe { vim_regcomp(reg_pat, RE_MAGIC) };
                    if cm.cmod_filter_regmatch.regprog.is_null() {
                        break;
                    }
                }
                excmd.line.cmd = at;
            }
            b'h' => {
                if takes(excmd, b"horizontal", 3) {
                    cm.cmod_split |= WSP_HOR as c_int;
                } else if at == excmd.line.cmd
                    && let Some(after) = check_for_word(&excmd.line, at, b"hide", 3)
                    && excmd.line.byte_at(after) != 0
                    && ends_excmd(c_int::from(excmd.line.byte_at(after))) == 0
                {
                    // `:hide` is a command in its own right, so it is
                    // only a modifier when a command follows it and no
                    // range precedes it.
                    excmd.line.cmd = after;
                    cm.cmod_flags |= CmdModFlags::HIDE;
                } else {
                    break;
                }
            }
            b'l' => {
                if takes(excmd, b"lockmarks", 3) {
                    cm.cmod_flags |= CmdModFlags::LOCKMARKS;
                } else if takes(excmd, b"leftabove", 5) {
                    cm.cmod_split |= WSP_ABOVE as c_int;
                } else {
                    break;
                }
            }
            b'n' => {
                if takes(excmd, b"noautocmd", 3) {
                    cm.cmod_flags |= CmdModFlags::NOAUTOCMD;
                } else if takes(excmd, b"noswapfile", 3) {
                    cm.cmod_flags |= CmdModFlags::NOSWAPFILE;
                } else {
                    break;
                }
            }
            b'r' => {
                if !takes(excmd, b"rightbelow", 6) {
                    break;
                }
                cm.cmod_split |= WSP_BELOW as c_int;
            }
            b's' => {
                if takes(excmd, b"sandbox", 3) {
                    cm.cmod_flags |= CmdModFlags::SANDBOX;
                } else if takes(excmd, b"silent", 3) {
                    cm.cmod_flags |= CmdModFlags::SILENT;
                    // `:silent!` only means "and silence errors" when
                    // the `!` is stuck to the word: `:silent !cmd` runs
                    // a shell command quietly.
                    let cmd = excmd.line.cmd;
                    if excmd.line.byte_at(cmd) == b'!'
                        && cmd > 0
                        && !ascii_iswhite(c_int::from(excmd.line.byte_at(cmd - 1)))
                    {
                        excmd.line.cmd = excmd.line.skip_white(cmd + 1);
                        cm.cmod_flags |= CmdModFlags::ERRSILENT;
                    }
                } else {
                    break;
                }
            }
            b't' => {
                if let Some(after) = check_for_word(&excmd.line, at, b"tab", 3) {
                    at = after;
                    if !skip_only {
                        // The scan advances a cursor of its own; see
                        // `parse_cmd_address`.
                        let mut cursor = excmd.cmd_ptr();
                        let skip = excmd.skip;
                        let tabnr = unsafe {
                            get_address(
                                Some(excmd),
                                &raw mut cursor,
                                CmdAddr::Tabs,
                                skip,
                                skip_only,
                                0,
                                1,
                                errormsg,
                            )
                        } as c_int;
                        if cursor.is_null() {
                            return Err(Failed);
                        }
                        excmd.set_cmd_ptr(cursor);
                        if tabnr == MAXLNUM {
                            cm.cmod_tab = tab_index(TabPage::current()) + 1;
                        } else {
                            if tabnr < 0 || tabnr > current_tab_nr(None) {
                                *errormsg = Some(unsafe { ex_msg(e_invrange.as_ptr()) });
                                return Err(Failed);
                            }
                            cm.cmod_tab = tabnr + 1;
                        }
                    }
                    excmd.line.cmd = at;
                } else if takes(excmd, b"topleft", 2) {
                    cm.cmod_split |= WSP_TOP as c_int;
                } else {
                    break;
                }
            }
            b'u' => {
                if !takes(excmd, b"unsilent", 3) {
                    break;
                }
                cm.cmod_flags |= CmdModFlags::UNSILENT;
            }
            b'v' => {
                if takes(excmd, b"vertical", 4) {
                    cm.cmod_split |= WSP_VERT as c_int;
                } else if let Some(after) = check_for_word(&excmd.line, at, b"verbose", 4) {
                    at = after;
                    // The count is read from `args.cmd`, which
                    // `checkforcmd` left *before* the word: `:5verbose`.
                    // Saturating: the count is whatever the user typed,
                    // so `:2147483647verbose set` would otherwise add one
                    // to `INT_MAX` and end the process.  C wraps here.
                    let count = excmd.line.ptr_at(excmd.line.cmd);
                    cm.cmod_verbose =
                        if ascii_isdigit(c_int::from(excmd.line.byte_at(excmd.line.cmd))) {
                            // SAFETY: the command word, NUL-terminated.
                            unsafe { atoi(count) }.saturating_add(1)
                        } else {
                            2
                        };
                    excmd.line.cmd = at;
                } else {
                    break;
                }
            }
            _ => break,
        }
    }

    unsafe { restore_visual_range(excmd, orig_cmd, cmd_start, has_visual_range, use_plus_cmd) };
    Ok(())
}

/// Put the `'<,'>` this scan stepped over back in front of the command.
///
/// The range has to end up immediately before the command word, so the
/// text between them — the modifiers — is shuffled left by the five bytes
/// the range occupies and the range is written into the gap. Ex mode's
/// substituted `+` command is not in the buffer at all, so it takes the
/// other branch and the range is prefixed rather than moved.
///
/// # Safety
///
/// `orig_cmd` must point at a NUL-terminated string, unaliased for the call.
/// `cmd_start` must point at a NUL-terminated string, unaliased for the call.
unsafe fn restore_visual_range(
    excmd: &mut ExArg,
    orig_cmd: *mut c_char,
    cmd_start: *mut c_char,
    has_visual_range: bool,
    use_plus_cmd: bool,
) {
    if !has_visual_range {
        if use_plus_cmd {
            // The `+` the scan substituted for the empty line is the whole
            // of it, so the command word is its first byte.
            excmd.line.cmd = 0;
        }
        return;
    }
    if excmd.cmd_ptr() > cmd_start {
        if use_plus_cmd {
            let len = unsafe { cstr::bytes_at(cmd_start) }.len();
            move_bytes(orig_cmd, cmd_start, len);
            unsafe { xmemcpyz(orig_cmd.add(len) as *mut c_void, c" *+".as_ptr().cast(), 3) };
        } else {
            // SAFETY: the five bytes before `cmd_start` are the `:'<,'>` this
            // is making room for, and both ends are inside the command line.
            let (into, kept) =
                unsafe { (cmd_start.offset(-5), excmd.cmd_ptr().offset_from(cmd_start)) };
            move_bytes(into, cmd_start, kept as size_t);
            let cmd_start = excmd.cmd_ptr();
            excmd.set_cmd_ptr(unsafe { cmd_start.offset(-5) });
            let at = unsafe { excmd.cmd_ptr().offset(-1) };
            move_bytes(at, c":'<,'>".as_ptr(), 6);
        }
    } else if use_plus_cmd {
        // The Visual range is put back in front of the `+` this run
        // substituted for an empty Ex-mode line.
        excmd.line.take_over(b"'<,'>+\0".to_vec());
        excmd.line.cmd = 0;
        // Not the bare substitution any more: the range is the user's.
        excmd.line.substituted = false;
    } else {
        excmd.set_cmd_ptr(orig_cmd);
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
fn apply_cmdmod() {
    let mods = cmdmod.with(|cm| cm.cmod_flags);
    if mods.has(CmdModFlags::SANDBOX) && cmdmod.with(|cm| cm.cmod_did_sandbox) == 0 {
        sandbox.set(sandbox.get() + 1);
        cmdmod.with_mut(|cm| cm.cmod_did_sandbox = 1);
    }
    let verbose = cmdmod.with(|cm| cm.cmod_verbose);
    if verbose > 0 {
        if cmdmod.with(|cm| cm.cmod_verbose_save) == 0 {
            let save = p_verbose() + 1;
            cmdmod.with_mut(|cm| cm.cmod_verbose_save = save);
        }
        P_VERBOSE.set((verbose - 1) as OptInt);
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
        let save_ei = P_EI.get().into_raw();
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
    // A `.rodata` borrow: `set_option_direct`, its only consumer, copies.
    OptVal::static_string(c"all")
}

/// Take the modifiers back out of force.
pub(crate) fn undo_cmdmod(cm: &mut CmdMod) {
    if cm.cmod_verbose_save > 0 {
        P_VERBOSE.set(cm.cmod_verbose_save - 1);
        cm.cmod_verbose_save = 0;
    }
    if cm.cmod_did_sandbox != 0 {
        sandbox.set(sandbox.get() - 1);
        cm.cmod_did_sandbox = 0;
    }
    if !cm.cmod_save_ei.is_null() {
        set_option_direct(
            kOptEventignore,
            // SAFETY: the saved value is NUL-terminated and freed just
            // below, after `set_option_direct` has copied it.
            OptVal::String(unsafe { OptStr::borrowing(cm.cmod_save_ei) }),
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
        if redirecting() {
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
    saved: CmdMod,
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
    pub(crate) fn enter(mods: CmdMod) -> Self {
        let scope = CmdModScope {
            saved: cmdmod.take(),
        };
        cmdmod.set(mods);
        apply_cmdmod();
        scope
    }

    /// Read the run of modifiers at the head of `excmd`'s command line into
    /// the cell, which stays *cleared* for the whole parse — an error
    /// raised while the modifiers are still being read is no longer inside
    /// the enclosing `:silent` or `:filter`, which is what the C's opening
    /// `CLEAR_FIELD(cmdmod)` amounts to.
    pub(crate) fn parse(
        &self,
        excmd: &mut ExArg,
        errormsg: &mut Option<CString>,
    ) -> Result<(), Failed> {
        let mut parsed = CmdMod::default();
        // SAFETY: the caller's contract.
        let read = parse_command_modifiers(excmd, errormsg, &mut parsed, false);
        cmdmod.set(parsed);
        read
    }

    /// Put the parsed modifiers in force, once [`CmdModScope::parse`] has
    /// stored them.
    pub(crate) fn apply(&self) {
        apply_cmdmod();
    }
}

impl Drop for CmdModScope {
    fn drop(&mut self) {
        // The clone shares the `:filter` pattern and program with the cell,
        // which is what lets `undo_cmdmod` free them while the cell still
        // answers `message_filtered` for anything it says on the way out.
        let mut live = cmdmod.with(Clone::clone);
        undo_cmdmod(&mut live);
        cmdmod.set(core::mem::take(&mut self.saved));
    }
}

/// How many bytes of `cmd` are a command modifier, or 0 if none are.
///
/// Used by the command-line completion to decide what the word after a
/// modifier should complete as.
///
/// # Safety
///
/// `cmd` must point at a NUL-terminated string, unaliased for the call.
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
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
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
    expr_map_lock.get() > 0 && !Buf::current().b_flags.has(BufFlags::DUMMY)
}

/// Is this the location-list spelling of a quickfix command? Upstream tells
/// them apart by the leading `l` of the name and nothing else.
pub fn is_loclist_cmd(cmdidx: CmdIdx) -> bool {
    if is_user_cmd(cmdidx) || cmdidx == CmdIdx::SIZE {
        return false;
    }
    unsafe { *cmdnames[cmdidx.index()].cmd_name as c_int == 'l' as c_int }
}

/// Is this one of the mapping commands? Asked by the argument scan, which
/// must not treat a `<expr>` mapping's right-hand side as an expression.
pub fn is_map_cmd(cmdidx: CmdIdx) -> bool {
    if is_user_cmd(cmdidx) {
        return false;
    }
    let func: ExFunc = cmdnames[cmdidx.index()].cmd_func;
    ex_func_is(func, ex_map)
        || ex_func_is(func, ex_unmap)
        || ex_func_is(func, ex_mapclear)
        || ex_func_is(func, ex_abbreviate)
        || ex_func_is(func, ex_abclear)
}

/// Take the modifier `name` off the front of the command word if it is
/// there, `min` letters being enough to spell it.
fn takes(excmd: &mut ExArg, name: &[u8], min: usize) -> bool {
    match check_for_word(&excmd.line, excmd.line.cmd, name, min) {
        Some(at) => {
            excmd.line.cmd = at;
            true
        }
        None => false,
    }
}

/// `memmove()`'s byte copy as checked code: `n` bytes, overlap allowed.
fn move_bytes(dest: *mut c_char, src: *const c_char, n: size_t) {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { dest.cast::<u8>().copy_from(src.cast(), n) };
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
