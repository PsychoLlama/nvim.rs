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
use crate::charset::skipwhite;
use crate::eval::typval::tv_get_string;
use crate::ex_docmd::address::skip_range;
use crate::ex_docmd::modifier::{CMDMODS, shared_prefix};
use crate::ex_docmd::{
    ADDR_LINES, EXFLAG_LIST, EXFLAG_PRINT, cmdidxs1, cmdidxs2, cmdnames, command_count,
};
use crate::main::getout;
use crate::memory::xstrdup;
use crate::message::iemsg;
use crate::os::cshim::{gettext, strncmp};
use crate::types::{
    CMD_Next, CMD_SIZE, CMD_append, CMD_bang, CMD_k, CMD_match, CMD_substitute, EvalFuncData, NUL,
    VAR_STRING, cmdidx_T, exarg_T, expand_T, size_t, typval_T, uint32_t,
};
use crate::usercmd::{expand_user_command_name, find_ucmd, get_user_command_name};

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
    unsafe {
        let p = *pp;
        let mut i = 0isize;
        while *cmd.offset(i) as c_int != NUL && *cmd.offset(i) == *p.offset(i) {
            i += 1;
        }
        // A letter after the abbreviation means this is a longer word, not
        // this command: `:silentx` is not `:silent`.
        if i as c_int >= len && !(*p.offset(i) as u8).is_ascii_alphabetic() {
            *pp = skipwhite(p.offset(i));
            return true;
        }
        false
    }
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
    unsafe {
        let at = |n: usize| *p.add(n) as c_int;
        if at(0) == 'k' as c_int
            && (at(1) != 'e' as c_int || (at(1) == 'e' as c_int && at(2) != 'e' as c_int))
        {
            *idx = CMD_k;
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
            *idx = CMD_substitute;
            return 1;
        }
        0
    }
}

/// Resolve `eap->cmd` to a command index, and answer where the name ends.
///
/// `eap->cmdidx` comes back as `CMD_SIZE` for a name nothing matched, and
/// as a *negative* index for a user command. `full`, when given, is set
/// when the name was spelled out in full rather than abbreviated.
pub unsafe fn find_ex_command(eap: *mut exarg_T, full: *mut c_int) -> *mut c_char {
    unsafe {
        let ea = &mut *eap;
        let mut p = ea.cmd;
        if one_letter_cmd(p, &raw mut ea.cmdidx) != 0 {
            if !full.is_null() {
                *full = 1;
            }
            return p.add(1);
        }

        while (*p as u8).is_ascii_alphabetic() {
            p = p.add(1);
        }
        // `:py3`, `:python3` and `:py3file` are the only commands with a
        // digit in the name.
        if *ea.cmd as c_int == 'p' as c_int && *ea.cmd.add(1) as c_int == 'y' as c_int {
            while (*p as u8).is_ascii_alphanumeric() {
                p = p.add(1);
            }
        }
        // A command that is punctuation rather than a word.
        if p == ea.cmd && c"@!=><&~#".to_bytes().contains(&(*p as u8)) {
            p = p.add(1);
        }

        let mut len = p.offset_from(ea.cmd) as c_int;
        // `:dl` and `:dp` are `:delete` with a trailing `l`/`p` flag stuck
        // to it, and only when the rest really is an abbreviation of
        // "delete" — `:dj` is `:djump`.
        if *ea.cmd as c_int == 'd' as c_int
            && (*p.offset(-1) as c_int == 'l' as c_int || *p.offset(-1) as c_int == 'p' as c_int)
        {
            // `with_nul`, not `to_bytes`: the walk is over the *typed*
            // word, which may be longer than "delete", and it is the
            // terminator that stops it — `:ddddddddl` would otherwise
            // index past the end.
            let delete = c"delete".to_bytes_with_nul();
            let mut i = 0;
            while i < len && *ea.cmd.offset(i as isize) as u8 == delete[i as usize] {
                i += 1;
            }
            if i == len - 1 {
                len -= 1;
                if *p.offset(-1) as c_int == 'l' as c_int {
                    ea.flags |= EXFLAG_LIST;
                } else {
                    ea.flags |= EXFLAG_PRINT;
                }
            }
        }

        ea.cmdidx = start_index(ea.cmd, len);
        debug_assert!(ea.cmdidx as c_int >= 0);
        // `:def` is Vim9 script's, which this editor does not have; it must
        // not resolve to `:defer`.
        if len == 3 && strncmp(c"def".as_ptr(), ea.cmd, 3) == 0 {
            ea.cmdidx = CMD_SIZE;
        }

        while (ea.cmdidx as c_int) < CMD_SIZE as c_int {
            let name = (*cmdnames.ptr())[ea.cmdidx as usize].cmd_name;
            if strncmp(name, ea.cmd, len as size_t) == 0 {
                if !full.is_null() && *name.offset(len as isize) as c_int == NUL {
                    *full = 1;
                }
                break;
            }
            ea.cmdidx = (ea.cmdidx as c_int + 1) as cmdidx_T;
        }

        // Nothing in the table, and it starts with an upper-case letter:
        // it may be a user command, whose name may hold digits too.
        if ea.cmdidx as c_int == CMD_SIZE as c_int && (*ea.cmd as u8).is_ascii_uppercase() {
            while (*p as u8).is_ascii_alphanumeric() {
                p = p.add(1);
            }
            p = find_ucmd(eap, p, full, ptr::null_mut(), ptr::null_mut());
        }
        if p == ea.cmd {
            ea.cmdidx = CMD_SIZE;
        }
        p
    }
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
    unsafe {
        let c1 = *cmd as u8;
        if !c1.is_ascii_lowercase() {
            return if c1.is_ascii_uppercase() {
                CMD_Next
            } else {
                CMD_bang
            };
        }
        if command_count.get() != CMD_SIZE as c_int {
            iemsg(gettext(
                c"E943: Command table needs to be updated, run 'make'".as_ptr(),
            ));
            getout(1);
        }
        let c2 = if len == 1 { 0u8 } else { *cmd.add(1) as u8 };
        let mut idx = (*cmdidxs1.ptr())[(c1 - b'a') as usize] as c_int;
        if c2.is_ascii_lowercase() {
            idx += (*cmdidxs2.ptr())[(c1 - b'a') as usize][(c2 - b'a') as usize] as c_int;
        }
        idx as cmdidx_T
    }
}

/// `exists(":cmd")`: 0 for no, 1 for an abbreviation, 2 for a full name,
/// 3 for a name that is ambiguous between user commands.
pub unsafe fn cmd_exists(name: *const c_char) -> c_int {
    unsafe {
        // A modifier is a command as far as `exists()` is concerned.
        for md in &CMDMODS {
            let j = shared_prefix(name, md.name);
            if *name.add(j) as c_int == NUL && j >= md.minlen {
                return if md.name.to_bytes().len() == j { 2 } else { 1 };
            }
        }
        // `:2match`/`:3match` carry their count in the name.
        let mut ea = blank_exarg();
        ea.cmd = if *name as c_int == '2' as c_int || *name as c_int == '3' as c_int {
            name.add(1)
        } else {
            name
        } as *mut c_char;
        let mut full: c_int = 0;
        let p = find_ex_command(&raw mut ea, &raw mut full);
        if p.is_null() {
            return 3;
        }
        // A leading digit is a range for every command but `:match`.
        if ascii_isdigit(*name as c_int) && ea.cmdidx as c_int != CMD_match as c_int {
            return 0;
        }
        if *skipwhite(p) as c_int != NUL {
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
}

/// `fullcommand()`: the full name of the command an abbreviation means.
///
/// The generated builtin-function table holds it as a `VimLFunc` fn
/// pointer, and apigen's line-based scan needs the declaration spelled out
/// literally.
pub unsafe fn f_fullcommand(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        let mut name = tv_get_string(argvars) as *mut c_char;
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ptr::null_mut();
        while *name as c_int == ':' as c_int {
            name = name.add(1);
        }
        name = skip_range(name, ptr::null_mut());
        let mut ea = blank_exarg();
        ea.cmd = if *name as c_int == '2' as c_int || *name as c_int == '3' as c_int {
            name.add(1)
        } else {
            name
        };
        let p = find_ex_command(&raw mut ea, ptr::null_mut());
        if p.is_null() || ea.cmdidx as c_int == CMD_SIZE as c_int {
            return;
        }
        (*rettv).vval.v_string = xstrdup(if (ea.cmdidx as c_int) < 0 {
            get_user_command_name(ea.useridx, ea.cmdidx as c_int)
        } else {
            (*cmdnames.ptr())[ea.cmdidx as usize].cmd_name
        });
    }
}

/// A zeroed `exarg_T` with the two fields a lookup needs set the way
/// `find_ex_command` expects: `CMD_append` is index 0, the head of the
/// table, and no flags have been collected yet.
fn blank_exarg() -> exarg_T {
    let mut ea: exarg_T = unsafe { core::mem::zeroed() };
    ea.cmdidx = CMD_append;
    ea.addr_type = ADDR_LINES;
    ea.flags = 0;
    ea
}

/// The command index for a name of a known length, without the rest of
/// `find_ex_command`'s bookkeeping. Used by the API's command parser.
pub unsafe fn excmd_get_cmdidx(cmd: *const c_char, len: size_t) -> cmdidx_T {
    unsafe {
        if len == 3 && strncmp(c"def".as_ptr(), cmd, 3) == 0 {
            return CMD_SIZE;
        }
        let mut idx: cmdidx_T = CMD_append;
        if one_letter_cmd(cmd, &raw mut idx) != 0 {
            return idx;
        }
        // A linear scan from the head of the table, not the `cmdidxs`
        // shortcut: this entry point is not on the hot path.
        let mut idx = CMD_append;
        while (idx as c_int) < CMD_SIZE as c_int {
            if strncmp((*cmdnames.ptr())[idx as usize].cmd_name, cmd, len) == 0 {
                break;
            }
            idx = (idx as c_int + 1) as cmdidx_T;
        }
        idx
    }
}

/// The `EX_*` flag set of a command.
pub unsafe fn excmd_get_argt(idx: cmdidx_T) -> uint32_t {
    unsafe { (*cmdnames.ptr())[idx as usize].cmd_argt }
}

/// The `idx`'th command name, for command-line completion. Indices past
/// the table are user commands.
///
/// Keeps the raw signature: cmdexpand's generator table holds it as an
/// `ItemGetter`.
pub unsafe fn get_command_name(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    unsafe {
        if idx >= CMD_SIZE as c_int {
            return expand_user_command_name(idx);
        }
        (*cmdnames.ptr())[idx as usize].cmd_name
    }
}
