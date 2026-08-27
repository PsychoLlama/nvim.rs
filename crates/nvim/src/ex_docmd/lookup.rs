//! Resolving a command name to a row of `cmdnames`.
//!
//! `find_ex_command` is the hot path: `cmdidxs1` and `cmdidxs2` — two
//! generated tables indexed by the first and second letter — let it start
//! the linear scan at the first command that could match, rather than at
//! the head of a 557-row table. Everything else here is a special case the
//! table cannot express.
#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr;

use crate::ascii::ascii_isdigit;

use crate::eval::typval::NumBuf;
use crate::ex_docmd::address::skip_range;
use crate::ex_docmd::modifier::{CMDMODS, shared_prefix};
use crate::ex_docmd::{EXFLAG_LIST, EXFLAG_PRINT, cmdidxs1, cmdidxs2, cmdnames, command_count};
use crate::main::getout;
use crate::memory::xstrdup;
use crate::message::iemsg;
use crate::os::cshim::gettext;

use crate::types::{
    CMD_Next, CMD_SIZE, CMD_append, CMD_bang, CMD_k, CMD_match, CMD_substitute, CmdAddr,
    EvalFuncData, ExArgt, NUL, VAR_STRING, cmdidx_T, exarg_T, expand_T, size_t, typval_T,
};
use crate::usercmd::{expand_user_command_name, find_ucmd, get_user_command_name};
use crate::winlayer::Ea;

/// Is this index a *user* command rather than a row of `cmdnames`?
///
/// `find_ucmd` answers a negative index, whose magnitude has nothing to do
/// with `cmdnames` — `eap->useridx` says which user command it is. So a
/// negative `cmdidx` is the signal that the table must not be indexed.
pub fn is_user_cmd(cmdidx: cmdidx_T) -> bool {
    (cmdidx as c_int) < 0
}

/// Does `pp` start with at least `len` characters of `cmd`, and end there?
///
/// **Advances `*pp` past the word on a match**, which is why the modifier
/// scan's arms are ordered the way they are: a failed `checkforcmd` leaves
/// the cursor alone, a successful one does not.
pub unsafe fn checkforcmd(pp: *mut *mut c_char, cmd: *const c_char, len: c_int) -> bool {
    let p = unsafe { *pp };
    let mut i = 0isize;
    while unsafe { *cmd.offset(i) } as c_int != NUL
        && unsafe { *cmd.offset(i) } == unsafe { *p.offset(i) }
    {
        i += 1;
    }
    // A letter after the abbreviation means this is a longer word, not
    // this command: `:silentx` is not `:silent`.
    if i as c_int >= len && !(unsafe { *p.offset(i) } as u8).is_ascii_alphabetic() {
        unsafe { *pp = skipwhite(p.offset(i)) };
        return true;
    }
    false
}

/// The two commands whose one-letter spelling the table cannot express,
/// because a longer command starts with the same letter.
///
/// `:k` is a mark, unless the word is `:ke…` (`:keepmarks` and friends).
/// `:s` is a substitute, unless the word is one of `:scriptnames`,
/// `:scriptencoding`, `:sign`, `:simalt`, `:sil…`, `:sre…` and the rest —
/// which is what the nest of tests below spells out. It is upstream's, byte
/// for byte, including the `p[3]`/`p[4]` asymmetry in the `:sc…` arm.
pub(crate) unsafe fn one_letter_cmd(p: *const c_char, idx: *mut cmdidx_T) -> c_int {
    let at = |n: usize| unsafe { *p.add(n) } as c_int;
    if at(0) == 'k' as c_int
        && (at(1) != 'e' as c_int || (at(1) == 'e' as c_int && at(2) != 'e' as c_int))
    {
        unsafe { *idx = CMD_k };
        return 1;
    }
    if at(0) == 's' as c_int
        && (at(1) == 'c' as c_int
            && (at(2) == NUL
                || (at(2) != 's' as c_int
                    && at(2) != 'r' as c_int
                    && (at(3) == NUL || (at(3) != 'i' as c_int && at(4) != 'p' as c_int))))
            || at(1) == 'g' as c_int
            || at(1) == 'i' as c_int
                && at(2) != 'm' as c_int
                && at(2) != 'l' as c_int
                && at(2) != 'g' as c_int
            || at(1) == 'I' as c_int
            || at(1) == 'r' as c_int && at(2) != 'e' as c_int)
    {
        unsafe { *idx = CMD_substitute };
        return 1;
    }
    0
}

/// Resolve `eap->cmd` to a command index, and answer where the name ends.
///
/// `eap->cmdidx` comes back as `CMD_SIZE` for a name nothing matched, and
/// as a *negative* index for a user command. `full`, when given, is set
/// when the name was spelled out in full rather than abbreviated.
pub unsafe fn find_ex_command(eap: *mut exarg_T, full: *mut c_int) -> *mut c_char {
    let mut ea = unsafe { Ea::new(eap) };
    let mut p = ea.cmd;
    if unsafe { one_letter_cmd(p, ea.cmdidx_ptr()) } != 0 {
        if !full.is_null() {
            unsafe { *full = 1 };
        }
        return unsafe { p.add(1) };
    }

    while (unsafe { *p } as u8).is_ascii_alphabetic() {
        p = unsafe { p.add(1) };
    }
    // `:py3`, `:python3` and `:py3file` are the only commands with a
    // digit in the name.
    if unsafe { *ea.cmd } as c_int == 'p' as c_int
        && unsafe { *ea.cmd.add(1) } as c_int == 'y' as c_int
    {
        while (unsafe { *p } as u8).is_ascii_alphanumeric() {
            p = unsafe { p.add(1) };
        }
    }
    // A command that is punctuation rather than a word.
    if p == ea.cmd && c"@!=><&~#".to_bytes().contains(&(unsafe { *p } as u8)) {
        p = unsafe { p.add(1) };
    }

    let mut len = unsafe { p.offset_from(ea.cmd) } as c_int;
    // `:dl` and `:dp` are `:delete` with a trailing `l`/`p` flag stuck
    // to it, and only when the rest really is an abbreviation of
    // "delete" — `:dj` is `:djump`.
    if unsafe { *ea.cmd } as c_int == 'd' as c_int
        && (unsafe { *p.offset(-1) } as c_int == 'l' as c_int
            || unsafe { *p.offset(-1) } as c_int == 'p' as c_int)
    {
        // `with_nul`, not `to_bytes`: the walk is over the *typed*
        // word, which may be longer than "delete", and it is the
        // terminator that stops it — `:ddddddddl` would otherwise
        // index past the end.
        let delete = c"delete".to_bytes_with_nul();
        let mut i = 0;
        while i < len && unsafe { *ea.cmd.offset(i as isize) } as u8 == delete[i as usize] {
            i += 1;
        }
        if i == len - 1 {
            len -= 1;
            if unsafe { *p.offset(-1) } as c_int == 'l' as c_int {
                ea.flags |= EXFLAG_LIST;
            } else {
                ea.flags |= EXFLAG_PRINT;
            }
        }
    }

    ea.cmdidx = unsafe { start_index(ea.cmd, len) };
    debug_assert!(ea.cmdidx as c_int >= 0);
    // `:def` is Vim9 script's, which this editor does not have; it must
    // not resolve to `:defer`.
    if len == 3 && strncmp(c"def".as_ptr(), ea.cmd, 3) == 0 {
        ea.cmdidx = CMD_SIZE;
    }

    while (ea.cmdidx as c_int) < CMD_SIZE as c_int {
        let name = cmdnames[ea.cmdidx as usize].cmd_name;
        if strncmp(name, ea.cmd, len as size_t) == 0 {
            if !full.is_null() && unsafe { *name.offset(len as isize) } as c_int == NUL {
                unsafe { *full = 1 };
            }
            break;
        }
        ea.cmdidx = (ea.cmdidx as c_int + 1) as cmdidx_T;
    }

    // Nothing in the table, and it starts with an upper-case letter:
    // it may be a user command, whose name may hold digits too.
    if ea.cmdidx as c_int == CMD_SIZE as c_int && (unsafe { *ea.cmd } as u8).is_ascii_uppercase() {
        while (unsafe { *p } as u8).is_ascii_alphanumeric() {
            p = unsafe { p.add(1) };
        }
        p = unsafe { find_ucmd(eap, p, full, ptr::null_mut(), ptr::null_mut()) };
    }
    if p == ea.cmd {
        ea.cmdidx = CMD_SIZE;
    }
    p
}

/// Where the linear scan over `cmdnames` starts for this name.
///
/// `cmdidxs1[c1]` is the first command starting with `c1`, and
/// `cmdidxs2[c1][c2]` the offset from there to the first one starting with
/// `c1c2`. Both are generated *from the table's row order*, so a table that
/// has been reordered without regenerating them sends the scan to the wrong
/// place — hence the `command_count` check, which is upstream's own guard
/// against a stale generated header.
unsafe fn start_index(cmd: *const c_char, len: c_int) -> cmdidx_T {
    let c1 = unsafe { *cmd } as u8;
    if !c1.is_ascii_lowercase() {
        return if c1.is_ascii_uppercase() {
            CMD_Next
        } else {
            CMD_bang
        };
    }
    if command_count != CMD_SIZE as c_int {
        unsafe {
            iemsg(gettext(
                c"E943: Command table needs to be updated, run 'make'".as_ptr(),
            ))
        };
        unsafe { getout(1) };
    }
    let c2 = if len == 1 {
        0u8
    } else {
        unsafe { *cmd.add(1) as u8 }
    };
    let mut idx = cmdidxs1[(c1 - b'a') as usize] as c_int;
    if c2.is_ascii_lowercase() {
        idx += cmdidxs2[(c1 - b'a') as usize][(c2 - b'a') as usize] as c_int;
    }
    idx as cmdidx_T
}

/// `exists(":cmd")`: 0 for no, 1 for an abbreviation, 2 for a full name,
/// 3 for a name that is ambiguous between user commands.
pub unsafe fn cmd_exists(name: *const c_char) -> c_int {
    // A modifier is a command as far as `exists()` is concerned.
    for md in &CMDMODS {
        let j = unsafe { shared_prefix(name, md.name) };
        if unsafe { *name.add(j) } as c_int == NUL && j >= md.minlen {
            return if md.name.to_bytes().len() == j { 2 } else { 1 };
        }
    }
    // `:2match`/`:3match` carry their count in the name.
    let mut ea = blank_exarg();
    ea.cmd =
        if unsafe { *name } as c_int == '2' as c_int || unsafe { *name } as c_int == '3' as c_int {
            unsafe { name.add(1) }
        } else {
            name
        } as *mut c_char;
    let mut full: c_int = 0;
    let p = unsafe { find_ex_command(&raw mut ea, &raw mut full) };
    if p.is_null() {
        return 3;
    }
    // A leading digit is a range for every command but `:match`.
    if ascii_isdigit(unsafe { *name } as c_int) && ea.cmdidx as c_int != CMD_match as c_int {
        return 0;
    }
    if unsafe { *skipwhite(p) } as c_int != NUL {
        return 0;
    }
    if ea.cmdidx as c_int == CMD_SIZE as c_int {
        0
    } else if full != 0 {
        2
    } else {
        1
    }
}

/// `fullcommand()`: the full name of the command an abbreviation means.
///
/// The generated builtin-function table holds it as a `VimLFunc` fn
/// pointer, and apigen's line-based scan needs the declaration spelled out
/// literally.
pub unsafe fn f_fullcommand(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut name = unsafe { numbuf.string(argvars) } as *mut c_char;
    unsafe { (*rettv).v_type = VAR_STRING };
    unsafe { (*rettv).vval.v_string = ptr::null_mut() };
    while unsafe { *name } as c_int == ':' as c_int {
        name = unsafe { name.add(1) };
    }
    name = unsafe { skip_range(name, ptr::null_mut()) };
    let mut ea = blank_exarg();
    ea.cmd =
        if unsafe { *name } as c_int == '2' as c_int || unsafe { *name } as c_int == '3' as c_int {
            unsafe { name.add(1) }
        } else {
            name
        };
    let p = unsafe { find_ex_command(&raw mut ea, ptr::null_mut()) };
    if p.is_null() || ea.cmdidx as c_int == CMD_SIZE as c_int {
        return;
    }
    unsafe {
        (*rettv).vval.v_string = xstrdup(if (ea.cmdidx as c_int) < 0 {
            get_user_command_name(ea.useridx, ea.cmdidx as c_int)
        } else {
            cmdnames[ea.cmdidx as usize].cmd_name
        })
    };
}

/// A zeroed `exarg_T` with the two fields a lookup needs set the way
/// `find_ex_command` expects: `CMD_append` is index 0, the head of the
/// table, and no flags have been collected yet.
fn blank_exarg() -> exarg_T {
    let mut ea: exarg_T = unsafe { core::mem::zeroed() };
    ea.cmdidx = CMD_append;
    ea.addr_type = CmdAddr::Lines;
    ea.flags = 0;
    ea
}

/// The command index for a name of a known length, without the rest of
/// `find_ex_command`'s bookkeeping. Used by the API's command parser.
pub unsafe fn excmd_get_cmdidx(cmd: *const c_char, len: size_t) -> cmdidx_T {
    if len == 3 && strncmp(c"def".as_ptr(), cmd, 3) == 0 {
        return CMD_SIZE;
    }
    let mut idx: cmdidx_T = CMD_append;
    if unsafe { one_letter_cmd(cmd, &raw mut idx) } != 0 {
        return idx;
    }
    // A linear scan from the head of the table, not the `cmdidxs`
    // shortcut: this entry point is not on the hot path.
    let mut idx = CMD_append;
    while (idx as c_int) < CMD_SIZE as c_int {
        if strncmp(cmdnames[idx as usize].cmd_name, cmd, len) == 0 {
            break;
        }
        idx = (idx as c_int + 1) as cmdidx_T;
    }
    idx
}

/// The `EX_*` flag set of a command.
pub unsafe fn excmd_get_argt(idx: cmdidx_T) -> ExArgt {
    cmdnames[idx as usize].cmd_argt
}

/// The `idx`'th command name, for command-line completion. Indices past
/// the table are user commands.
///
/// Keeps the raw signature: cmdexpand's generator table holds it as an
/// `ItemGetter`.
pub unsafe fn get_command_name(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    if idx >= CMD_SIZE as c_int {
        return unsafe { expand_user_command_name(idx) };
    }
    cmdnames[idx as usize].cmd_name
}

/// `skipwhite()` as checked code.
fn skipwhite(p: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated string.
    unsafe { crate::charset::skipwhite(p) }
}

/// `strncmp()` as checked code.
fn strncmp(
    __s1: *const ::core::ffi::c_char,
    __s2: *const ::core::ffi::c_char,
    __n: size_t,
) -> ::core::ffi::c_int {
    // SAFETY: two NUL-terminated strings, and a length within both.
    unsafe { crate::os::cshim::strncmp(__s1, __s2, __n) }
}
