//! Resolving a command name to a row of `cmdnames`.
//!
//! `find_ex_command` is the hot path: `cmdidxs1` and `cmdidxs2` — two
//! generated tables indexed by the first and second letter — let it start
//! the linear scan at the first command that could match, rather than at
//! the head of a 557-row table. Everything else here is a special case the
//! table cannot express.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::types::CmdIdx;
use core::ffi::{c_char, c_int};
use core::ptr;

use crate::ascii::ascii_isdigit;

use crate::eval::typval::NumBuf;
use crate::ex_docmd::address::skip_range;
use crate::ex_docmd::modifier::{CMDMODS, shared_prefix};
use crate::ex_docmd::{EXFLAG_LIST, EXFLAG_PRINT, cmdidxs1, cmdidxs2, cmdnames, command_count};
use crate::memory::xstrdup;
use crate::message::iemsg;
use crate::os::cshim::gettext;
use crate::startup::getout;

use crate::types::{CmdAddr, CmdLine, EvalFuncData, ExArg, ExArgt, Expand, NUL, TypVal, size_t};
use crate::usercmd::{expand_user_command_name, find_ucmd, get_user_command_name};

/// Is this index a *user* command rather than a row of `cmdnames`?
///
/// `find_ucmd` answers a negative index, whose magnitude has nothing to do
/// with `cmdnames` — `eap->useridx` says which user command it is. So a
/// negative `cmdidx` is the signal that the table must not be indexed.
pub fn is_user_cmd(cmdidx: CmdIdx) -> bool {
    matches!(cmdidx, CmdIdx::USER | CmdIdx::USER_BUF)
}

/// How many rows `cmdnames` has, as an index bound.
///
/// A `const` item rather than `CmdIdx::SIZE.code()` at each use: an
/// enum-to-integer conversion is a call at `-O0`, and this one bounds a
/// scan that runs per Ex command.
const ROWS: usize = command_count as usize;

/// Does `cursor` start with at least `len` characters of `cmd`, and end there?
///
/// **Advances `*pp` past the word on a match**, which is why the modifier
/// scan's arms are ordered the way they are: a failed `checkforcmd` leaves
/// the cursor alone, a successful one does not.
///
/// # Safety
///
/// `cursor` must point at a writable `*mut c_char` slot the caller owns for
/// the call. `cmd` must point at `len` readable bytes.
pub unsafe fn checkforcmd(cursor: *mut *mut c_char, cmd: *const c_char, len: c_int) -> bool {
    let p = unsafe { *cursor };
    let mut i = 0isize;
    while byte_at(cmd, i) != NUL && unsafe { *cmd.offset(i) } == unsafe { *p.offset(i) } {
        i += 1;
    }
    // A letter after the abbreviation means this is a longer word, not
    // this command: `:silentx` is not `:silent`.
    if i as c_int >= len && !(ubyte_at(p, i)).is_ascii_alphabetic() {
        unsafe { *cursor = skipwhite(p.offset(i)) };
        return true;
    }
    false
}

/// [`checkforcmd`] over a command line, answering where the word ended.
///
/// `None` is "not this command". `name` is the command spelled out and
/// `min` the shortest abbreviation of it the parse accepts, so `:sil`,
/// `:sile`, `:silen` and `:silent` are all `("silent", 3)` -- but `:silentx`
/// is not, because a letter after the abbreviation makes it a longer word.
pub(crate) fn check_for_word(line: &CmdLine, at: usize, name: &[u8], min: usize) -> Option<usize> {
    let matched = line.shared_prefix(at, name);
    let follows = line.byte_at(at + matched);
    (matched >= min && !follows.is_ascii_alphabetic()).then(|| line.skip_white(at + matched))
}

/// The two commands whose one-letter spelling the table cannot express,
/// because a longer command starts with the same letter.
///
/// `:k` is a mark, unless the word is `:ke…` (`:keepmarks` and friends).
/// `:s` is a substitute, unless the word is one of `:scriptnames`,
/// `:scriptencoding`, `:sign`, `:simalt`, `:sil…`, `:sre…` and the rest —
/// which is what the nest of tests below spells out. It is upstream's, byte
/// for byte, including the `p[3]`/`p[4]` asymmetry in the `:sc…` arm.
pub(crate) fn one_letter_cmd(at: impl Fn(usize) -> u8) -> Option<CmdIdx> {
    if at(0) == b'k' && (at(1) != b'e' || (at(1) == b'e' && at(2) != b'e')) {
        return Some(CmdIdx::k);
    }
    if at(0) == b's'
        && (at(1) == b'c'
            && (at(2) == 0
                || (at(2) != b's'
                    && at(2) != b'r'
                    && (at(3) == 0 || (at(3) != b'i' && at(4) != b'p'))))
            || at(1) == b'g'
            || at(1) == b'i' && at(2) != b'm' && at(2) != b'l' && at(2) != b'g'
            || at(1) == b'I'
            || at(1) == b'r' && at(2) != b'e')
    {
        return Some(CmdIdx::substitute);
    }
    None
}

/// Resolve `args.cmd` to a command index, and answer where the name ends.
///
/// `args.cmdidx` comes back as `CmdIdx::SIZE` for a name nothing matched, and
/// as a *negative* index for a user command. `full`, when given, is set
/// when the name was spelled out in full rather than abbreviated. `None` is
/// upstream's null answer: a user command the typed abbreviation cannot
/// choose between.
pub fn find_ex_command(excmd: &mut ExArg, full: Option<&mut bool>) -> Option<usize> {
    let cmd = excmd.line.cmd;
    if let Some(idx) = one_letter_cmd(|n| excmd.line.byte_at(cmd + n)) {
        excmd.cmdidx = idx;
        if let Some(full) = full {
            *full = true;
        }
        return Some(cmd + 1);
    }

    let mut at = cmd;
    while excmd.line.byte_at(at).is_ascii_alphabetic() {
        at += 1;
    }
    // `:py3`, `:python3` and `:py3file` are the only commands with a
    // digit in the name.
    if excmd.line.byte_at(cmd) == b'p' && excmd.line.byte_at(cmd + 1) == b'y' {
        while excmd.line.byte_at(at).is_ascii_alphanumeric() {
            at += 1;
        }
    }
    // A command that is punctuation rather than a word.
    if at == cmd && b"@!=><&~#".contains(&excmd.line.byte_at(at)) {
        at += 1;
    }

    let mut len = at - cmd;
    // `:dl` and `:dp` are `:delete` with a trailing `l`/`p` flag stuck
    // to it, and only when the rest really is an abbreviation of
    // "delete" — `:dj` is `:djump`.
    let last = excmd.line.byte_at(at.wrapping_sub(1));
    if excmd.line.byte_at(cmd) == b'd' && (last == b'l' || last == b'p') {
        // `with_nul`, not `to_bytes`: the walk is over the *typed*
        // word, which may be longer than "delete", and it is the
        // terminator that stops it — `:ddddddddl` would otherwise
        // index past the end.
        let delete = c"delete".to_bytes_with_nul();
        let mut i = 0;
        while i < len && excmd.line.byte_at(cmd + i) == delete[i] {
            i += 1;
        }
        if i + 1 == len {
            len -= 1;
            if last == b'l' {
                excmd.flags |= EXFLAG_LIST;
            } else {
                excmd.flags |= EXFLAG_PRINT;
            }
        }
    }

    excmd.cmdidx = CmdIdx::SIZE;
    // `:def` is Vim9 script's, which this editor does not have; it must
    // not resolve to `:defer`.
    if !(len == 3 && excmd.line.starts_with(cmd, b"def")) {
        // The word, read once: the scan below asks about it per row, and
        // measuring the line each time is what made this the parse's
        // hottest function when the cursors became offsets.
        let word = excmd.line.slice_at(cmd, len);
        // The scan walks rows rather than `CmdIdx`es and names what it
        // stopped at once: stepping an enum would be a conversion per row.
        let mut row = start_index(word);
        while row < ROWS {
            let name = cmdnames[row].cmd_name;
            if name_matches(name, word) {
                if let Some(full) = full
                    && byte_at(name, len as isize) == NUL
                {
                    *full = true;
                }
                excmd.cmdidx = CmdIdx::at_row(row);
                break;
            }
            row += 1;
        }
    }

    // Nothing in the table, and it starts with an upper-case letter:
    // it may be a user command, whose name may hold digits too.
    let mut end = Some(at);
    if excmd.cmdidx == CmdIdx::SIZE && excmd.line.byte_at(cmd).is_ascii_uppercase() {
        let mut at = at;
        while excmd.line.byte_at(at).is_ascii_alphanumeric() {
            at += 1;
        }
        end = unsafe { find_ucmd(excmd, at, None, ptr::null_mut(), ptr::null_mut()) };
    }
    if end == Some(cmd) {
        excmd.cmdidx = CmdIdx::SIZE;
    }
    end
}

/// Do the first `word.len()` bytes of the NUL-terminated `name` spell
/// `word`? `strncmp(name, word, word.len()) == 0`, with the typed side
/// already a slice.
fn name_matches(name: *const c_char, word: &[u8]) -> bool {
    // SAFETY: `name` is a table entry, NUL-terminated; the walk stops at
    // the first byte that differs, and the terminator differs from every
    // byte of `word`, which the caller counted out of a NUL-terminated
    // line.
    word.iter()
        .enumerate()
        .all(|(i, want)| unsafe { *name.add(i) } as u8 == *want)
}

/// Where the linear scan over `cmdnames` starts for this name.
///
/// `cmdidxs1[c1]` is the first command starting with `c1`, and
/// `cmdidxs2[c1][c2]` the offset from there to the first one starting with
/// `c1c2`. Both are generated *from the table's row order*, so a table that
/// has been reordered without regenerating them sends the scan to the wrong
/// place — hence the `command_count` check, which is upstream's own guard
/// against a stale generated header.
fn start_index(word: &[u8]) -> usize {
    let c1 = word.first().copied().unwrap_or(0);
    if !c1.is_ascii_lowercase() {
        return if c1.is_ascii_uppercase() {
            CmdIdx::Next.index()
        } else {
            CmdIdx::bang.index()
        };
    }
    if command_count != CmdIdx::SIZE.code() {
        iemsg(gettext(
            c"E943: Command table needs to be updated, run 'make'",
        ));
        getout(1);
    }
    let c2 = word.get(1).copied().unwrap_or(0);
    let mut idx = cmdidxs1[(c1 - b'a') as usize] as usize;
    if c2.is_ascii_lowercase() {
        idx += cmdidxs2[(c1 - b'a') as usize][(c2 - b'a') as usize] as usize;
    }
    idx
}

/// `exists(":cmd")`: 0 for no, 1 for an abbreviation, 2 for a full name,
/// 3 for a name that is ambiguous between user commands.
///
/// # Safety
///
/// `name` must point at a NUL-terminated string.
pub unsafe fn cmd_exists(name: *const c_char) -> c_int {
    // A modifier is a command as far as `exists()` is concerned.
    for md in &CMDMODS {
        let j = unsafe { shared_prefix(name, md.name) };
        if byte_at(name, j as isize) == NUL && j >= md.minlen {
            return if md.name.to_bytes().len() == j { 2 } else { 1 };
        }
    }
    // `:2match`/`:3match` carry their count in the name.
    let mut ea = blank_exarg();
    // SAFETY: the caller's promise -- a NUL-terminated name.
    ea.line = CmdLine::from_bytes(unsafe { cstr::bytes_at(name) });
    // `:2match`/`:3match` carry their count in the name.
    ea.line.cmd = usize::from(byte(name) == '2' as c_int || byte(name) == '3' as c_int);
    let mut full = false;
    let Some(at) = find_ex_command(&mut ea, Some(&mut full)) else {
        return 3;
    };
    // A leading digit is a range for every command but `:match`.
    if ascii_isdigit(byte(name)) && ea.cmdidx != CmdIdx::r#match {
        return 0;
    }
    if ea.line.byte_at(ea.line.skip_white(at)) != 0 {
        return 0;
    }
    if ea.cmdidx == CmdIdx::SIZE {
        0
    } else if full {
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
pub fn f_fullcommand(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut name = numbuf.string_ptr(&args[0]) as *mut c_char;
    result.write_string(ptr::null_mut());
    while byte(name) == ':' as c_int {
        name = unsafe { name.add(1) };
    }
    // SAFETY: `name` is NUL-terminated; a null context is "not completing".
    name = unsafe { name.add(skip_range(cstr::bytes_at(name), ptr::null_mut())) };
    let mut ea = blank_exarg();
    // SAFETY: `name` walks the NUL-terminated argument.
    ea.line = CmdLine::from_bytes(unsafe { cstr::bytes_at(name) });
    // `:2match`/`:3match` carry their count in the name.
    ea.line.cmd = usize::from(byte(name) == '2' as c_int || byte(name) == '3' as c_int);
    if find_ex_command(&mut ea, None).is_none() || ea.cmdidx == CmdIdx::SIZE {
        return;
    }
    unsafe {
        (*result).write_string(xstrdup(if is_user_cmd(ea.cmdidx) {
            get_user_command_name(ea.useridx, ea.cmdidx)
        } else {
            cmdnames[ea.cmdidx.index()].cmd_name
        }))
    };
}

/// A zeroed `ExArg` with the two fields a lookup needs set the way
/// `find_ex_command` expects: `CmdIdx::append` is index 0, the head of the
/// table, and no flags have been collected yet.
fn blank_exarg() -> ExArg {
    ExArg {
        cmdidx: CmdIdx::append,
        addr_type: CmdAddr::Lines,
        flags: 0,
        ..ExArg::default()
    }
}

/// The command index for a name of a known length, without the rest of
/// `find_ex_command`'s bookkeeping. Used by the API's command parser.
///
/// # Safety
///
/// `cmd` must point at `len` readable bytes.
pub unsafe fn excmd_get_cmdidx(cmd: *const c_char, len: size_t) -> CmdIdx {
    if len == 3 && prefix_eq(cmd, c"def".as_ptr(), 3) {
        return CmdIdx::SIZE;
    }
    // SAFETY: caller contract -- `cmd` is a NUL-terminated name, and the
    // walk stops at the terminator.
    if let Some(idx) = one_letter_cmd(|n| unsafe { *cmd.add(n) } as u8) {
        return idx;
    }
    // A linear scan from the head of the table, not the `cmdidxs`
    // shortcut: this entry point is not on the hot path.
    let mut row = 0;
    while row < ROWS {
        if prefix_eq(cmdnames[row].cmd_name, cmd, len) {
            break;
        }
        row += 1;
    }
    CmdIdx::at_row(row)
}

/// The `EX_*` flag set of a command.
pub fn excmd_get_argt(idx: CmdIdx) -> ExArgt {
    cmdnames[idx.index()].cmd_argt
}

/// The `idx`'th command name, for command-line completion. Indices past
/// the table are user commands.
///
/// Keeps the raw signature: cmdexpand's generator table holds it as an
/// `ItemGetter`.
///
/// # Safety
///
/// `_expand` must point at a live `Expand` context, unaliased for the call.
pub unsafe fn get_command_name(_expand: *mut Expand, idx: c_int) -> *mut c_char {
    if idx >= CmdIdx::SIZE.code() {
        return unsafe { expand_user_command_name(idx) };
    }
    cmdnames[idx as usize].cmd_name
}

/// Whether two NUL-terminated strings agree over their first `n` bytes --
/// `cstr::prefix_eq(a, b, n)` -- as checked code.
fn prefix_eq(a: *const c_char, b: *const c_char, n: usize) -> bool {
    // SAFETY: two NUL-terminated strings; each scan stops at its terminator.
    unsafe { cstr::prefix_eq(a, b, n) }
}

/// `skipwhite()` as checked code.
fn skipwhite(p: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated string.
    unsafe { crate::charset::skipwhite(p) }
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
