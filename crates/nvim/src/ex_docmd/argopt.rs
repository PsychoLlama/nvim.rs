//! Arguments that are not file names: `++opt=value`, `+cmd`, the tab page
//! argument, and opening the file a command will write to.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
use crate::cstr;
use crate::strings::vim_snprintf;
use crate::types::CmdIdx;
use crate::window::tab_index;
use crate::winlayer::TabPage;
use crate::winlayer::last_used_tab;

use std::ffi::CString;

use crate::semsg;
use crate::tr_plural;
use crate::winlayer::{Buf, Live, Win};

/// The completion context, whose caller has promised it outlives the value.
type Xp = Live<Expand>;
use core::ffi::{CStr, c_char, c_int, c_ulong};
use core::ptr;

use crate::ascii::{ascii_isdigit, ascii_isspace, ascii_iswhite};
use crate::charset::getdigits;

use crate::event::libuv::uv_strerror;

use crate::ex_docmd::lookup::check_for_word;

use crate::arglist::state::arg_had_last;
use crate::ex_docmd::window::current_tab_nr;
use crate::ex_docmd::{
    BAD_DROP, BAD_KEEP, DIALOG_MSG_SIZE, FORCE_BIN, FORCE_NOBIN, VIM_QUESTION, VIM_YES, cmdmod_has,
    quitmore,
};
use crate::mbyte::{get_encoding_name, utf8len_tab};
use crate::memory::{xmalloc, xstrdup};
use crate::message::vim_dialog_yesno;
use crate::message::{e_invarg2, e_invargval, e_invrange};
use crate::message_fmt::{c_str, emsg_text};
use crate::option::vars::p_confirm;
use crate::optionstr::{check_ff_value, get_fileformat_name};
use crate::os::cshim::ngettext;

use crate::os::fs::{os_fopen, os_isdir, os_mkdir, os_path_exists};

use crate::types::regexp::RegMatch;
use crate::types::{
    CmdLine, CmdModFlags, CompleteListItemGetter, EcmdCmd, ExArg, Expand, FAIL, FILE, Failed, NUL,
    OK, int32_t, intmax_t, size_t,
};
use crate::window::{only_one_window, tabpage_index};

/// Take a `+cmd` argument, and answer the command it names.
///
/// `+` alone means `$`, the last line. The command runs to the end of the
/// argument unless a space that is not backslash-escaped ends it, which is
/// what `skip_cmd_arg` finds; that byte is overwritten with a terminator,
/// so the answer is an offset into the command line's own buffer.
///
/// Leaves `args.arg` past the `+cmd`.
///
pub fn getargcmd(excmd: &mut ExArg) -> EcmdCmd {
    let mut at = excmd.line.arg;
    if excmd.line.byte_at(at) != b'+' {
        return EcmdCmd::None;
    }
    at += 1;
    let command =
        if ascii_isspace(c_int::from(excmd.line.byte_at(at))) || excmd.line.byte_at(at) == 0 {
            EcmdCmd::Dollar
        } else {
            let command = EcmdCmd::At(at);
            at = skip_arg_at(&mut excmd.line, at, true);
            // The command is handed on as a NUL-terminated string of its own,
            // so the byte that ended it becomes its terminator.
            if excmd.line.byte_at(at) != 0 {
                excmd.line.terminate_at(at);
                at += 1;
            }
            command
        };
    excmd.line.arg = excmd.line.skip_white(at);
    command
}

/// Read the value of `++bad=`: `keep`, `drop`, or one single-byte
/// replacement character.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
pub(crate) unsafe fn get_bad_opt(p: *const c_char, excmd: &mut ExArg) -> Result<(), Failed> {
    if strcasecmp(p as *mut c_char, c"keep".as_ptr() as *mut c_char) == 0 {
        excmd.bad_char = BAD_KEEP;
    } else if strcasecmp(p as *mut c_char, c"drop".as_ptr() as *mut c_char) == 0 {
        excmd.bad_char = BAD_DROP;
    } else if utf8len_tab[ubyte(p) as usize] == 1 && byte_at(p, 1) == NUL {
        excmd.bad_char = ubyte(p) as c_int;
    } else {
        return Err(Failed);
    }
    Ok(())
}

/// The completion candidates for `++bad=`.
///
/// Keeps the raw signature: installed as a `CompleteListItemGetter`.
pub(crate) fn get_bad_name(_expand: *mut Expand, idx: c_int) -> *mut c_char {
    const VALUES: [&CStr; 3] = [c"?", c"keep", c"drop"];
    match VALUES.get(idx as usize) {
        Some(v) => v.as_ptr() as *mut c_char,
        None => ptr::null_mut(),
    }
}

/// Read one `++opt` or `++opt=value` argument off the front of `args.arg`.
///
/// The three that take a value are stored as *offsets* into `args.cmd`
/// rather than as pointers, because the command line is reallocated by the
/// `%`/`#` expansion that runs later; `do_ecmd` and the write path resolve
/// them against the line they end up with.
pub fn getargopt(excmd: &mut ExArg) -> Result<(), Failed> {
    let mut at = excmd.line.arg + 2;

    // `++bin`/`++nobin` and `++binary`/`++nobinary`.
    if excmd.line.starts_with(at, b"bin") || excmd.line.starts_with(at, b"nobin") {
        if excmd.line.byte_at(at) == b'n' {
            at += 2;
            excmd.force_bin = FORCE_NOBIN;
        } else {
            excmd.force_bin = FORCE_BIN;
        }
        let Some(after) = check_for_word(&excmd.line, at, b"binary", 3) else {
            return Err(Failed);
        };
        excmd.line.arg = after;
        return Ok(());
    }

    // `++edit`, and not `++editsomething`.
    if excmd.line.starts_with(at, b"edit") && !excmd.line.byte_at(at + 4).is_ascii_alphabetic() {
        excmd.read_edit = true;
        excmd.line.arg = excmd.line.skip_white(at + 4);
        return Ok(());
    }

    // `++p`, and not `++psomething`.
    if excmd.line.byte_at(at) == b'p' && !excmd.line.byte_at(at + 1).is_ascii_alphabetic() {
        excmd.mkdir_p = true;
        excmd.line.arg = excmd.line.skip_white(at + 1);
        return Ok(());
    }

    // Which of the three offsets the value is recorded in, and how long the
    // option's own name was.
    let starts = |word: &[u8]| excmd.line.starts_with(at, word);
    let opt = if starts(b"fileformat") {
        Some((Opt::FileFormat, 10))
    } else if starts(b"ff") {
        Some((Opt::FileFormat, 2))
    } else if starts(b"encoding") {
        Some((Opt::Encoding, 8))
    } else if starts(b"enc") {
        Some((Opt::Encoding, 3))
    } else if starts(b"bad") {
        Some((Opt::BadChar, 3))
    } else {
        None
    };
    let Some((opt, name_len)) = opt else {
        return Err(Failed);
    };
    at += name_len;
    if excmd.line.byte_at(at) != b'=' {
        return Err(Failed);
    }
    at += 1;

    // The three that take a value are stored as *offsets from the command
    // word* rather than as cursors, because the command line is
    // reallocated by the `%`/`#` expansion that runs later.
    let value = c_int::try_from(at - excmd.line.cmd).unwrap_or(0);
    match opt {
        Opt::FileFormat => excmd.force_ff = value,
        Opt::Encoding => excmd.force_enc = value,
        // `get_bad_opt` reads the value itself, so nothing records it.
        Opt::BadChar => {}
    }
    let end = skip_arg_at(&mut excmd.line, at, false);
    excmd.line.arg = excmd.line.skip_white(end);
    excmd.line.terminate_at(end);

    match opt {
        Opt::FileFormat => {
            let ff = excmd.line.ptr_at(at);
            // SAFETY: the value, NUL-terminated by the write just above.
            if unsafe { check_ff_value(ff) } == FAIL {
                return Err(Failed);
            }
            // Only the first letter is kept: 'u', 'd' or 'm'.
            excmd.force_ff = c_int::from(excmd.line.byte_at(at));
        }
        Opt::Encoding => {
            for off in at..excmd.line.end_of(at) {
                let lower = excmd.line.byte_at(off).to_ascii_lowercase();
                excmd.line.set_byte(off, lower);
            }
        }
        Opt::BadChar => {
            let bad = excmd.line.ptr_at(at);
            // SAFETY: as above.
            if unsafe { get_bad_opt(bad, excmd) }.is_err() {
                return Err(Failed);
            }
        }
    }
    Ok(())
}

/// Which `++opt=value` was given: the three that take a value, and so the
/// three `ExArg` offsets one can be recorded in.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Opt {
    FileFormat,
    Encoding,
    BadChar,
}

/// [`skip_cmd_arg`] over a command line, answering where the argument ends.
///
/// `rembs` deletes the backslash of each escaped byte, in place, which
/// shortens the line -- so the answer is an offset into what the line is
/// *after* the walk.
fn skip_arg_at(line: &mut CmdLine, at: usize, rembs: bool) -> usize {
    let start = line.ptr_at(at);
    let end = skip_cmd_arg(start, rembs);
    line.offset_of(end)
}

/// The completion candidates for `++`.
///
/// Keeps the raw signature: installed as a `CompleteListItemGetter`.
pub(crate) fn get_argopt_name(_expand: *mut Expand, idx: c_int) -> *mut c_char {
    const VALUES: [&CStr; 7] = [
        c"fileformat=",
        c"encoding=",
        c"binary",
        c"nobinary",
        c"bad=",
        c"edit",
        c"p",
    ];
    match VALUES.get(idx as usize) {
        Some(v) => v.as_ptr() as *mut c_char,
        None => ptr::null_mut(),
    }
}

/// Complete a `++opt` argument: the option names, or the values of the one
/// already typed.
///
/// # Safety
///
/// `pat` must point at a NUL-terminated string, unaliased for the call.
/// `expand` must point at a live `Expand` context, unaliased for the call.
/// `rmp` must point at a live `RegMatch`, unaliased for the call. `matches`
/// must point at a writable `*mut *mut c_char` slot the caller owns for the
/// call. `num_matches` must point at a writable `int` the caller owns.
pub unsafe fn expand_argopt(
    pat: *mut c_char,
    expand: *mut Expand,
    rmp: *mut RegMatch,
    matches: *mut *mut *mut c_char,
    num_matches: *mut c_int,
) -> Result<(), Failed> {
    // SAFETY: the completion context is the caller's, live for the call.
    let x = unsafe { Xp::new(expand) };
    // Past an `=`: complete the value, by whichever option name ends
    // right before it.
    if x.xp_pattern > x.xp_line && byte_at(x.xp_pattern, -1) == '=' as c_int {
        let name_end = unsafe { x.xp_pattern.offset(-1) };
        let ends_with = |word: &CStr| {
            let n = word.to_bytes().len() as isize;
            unsafe {
                name_end.offset_from(x.xp_line) >= n
                    && prefix_eq(name_end.offset(-n), word.as_ptr(), n as size_t)
            }
        };
        let cb: CompleteListItemGetter = if ends_with(c"ff") || ends_with(c"fileformat") {
            Some(get_fileformat_name)
        } else if ends_with(c"enc") || ends_with(c"encoding") {
            Some(get_encoding_name)
        } else if ends_with(c"bad") {
            Some(get_bad_name)
        } else {
            None
        };
        if cb.is_none() {
            return Err(Failed);
        }
        expand_generic(pat, expand, rmp, matches, num_matches, cb, false);
        return Ok(());
    }
    // `++ff` is the only abbreviation worth finishing on its own.
    if x.xp_pattern_len == 2 && starts_with(x.xp_pattern, b"ff") {
        unsafe { *matches = xmalloc(size_of::<*mut c_char>()) as *mut *mut c_char };
        unsafe { *num_matches = 1 };
        unsafe { **matches = xstrdup(c"fileformat=".as_ptr()) };
        return Ok(());
    }
    expand_generic(
        pat,
        expand,
        rmp,
        matches,
        num_matches,
        Some(get_argopt_name),
        false,
    );
    Ok(())
}

/// Which tab page a `:tab…` command means.
///
/// Four spellings, and they do not agree on what counts as tab 0:
/// `:tabmove 0` moves before the first tab and is legal, every other
/// command refuses it — that is `unaccept_arg0`. An argument may be
/// absolute (`3`, `$`, `#`), relative (`+2`, `-1`), a range before the
/// command (`:2tabnext`), or absent.
pub(crate) fn get_tabpage_arg(excmd: &mut ExArg) -> c_int {
    let mut tab_number: c_int = 0;
    let unaccept_arg0 = if excmd.cmdidx == CmdIdx::tabmove {
        0
    } else {
        1
    };
    let last_tab = || current_tab_nr(None);
    let invarg2 = |command: &mut ExArg| {
        // SAFETY: the argument is a tail of the command line.
        command.errmsg = Some(ex_errmsg(e_invarg2.as_ptr(), command.arg_ptr()));
    };

    'theend: {
        if !excmd.arg_ptr().is_null() && excmd.line.byte_at(excmd.line.arg) != 0 {
            let mut p = excmd.arg_ptr();
            // `+N`/`-N` means N places to the right/left of here.
            let relative = match byte(p) {
                c if c == '-' as c_int => {
                    p = unsafe { p.add(1) };
                    -1
                }
                c if c == '+' as c_int => {
                    p = unsafe { p.add(1) };
                    1
                }
                _ => 0,
            };

            let p_save = p;
            tab_number = unsafe { getdigits(&raw mut p, false, tab_number as intmax_t) } as c_int;

            if relative == 0 {
                if equals(p, b"$") {
                    tab_number = last_tab();
                } else if equals(p, b"#") {
                    if last_used_tab().is_none() {
                        excmd.errmsg = Some(ex_errmsg(e_invargval.as_ptr(), excmd.arg_ptr()));
                        tab_number = 0;
                        break 'theend;
                    }
                    tab_number = tabpage_index(last_used_tab());
                } else if p == p_save
                    || byte(p_save) == '-' as c_int
                    || byte(p) != NUL
                    || tab_number > last_tab()
                {
                    // Not a number.
                    invarg2(excmd);
                    break 'theend;
                }
            } else {
                if byte(p_save) == NUL {
                    // A bare `+` or `-` is one place.
                    tab_number = 1;
                } else if p == p_save
                    || byte(p_save) == '-' as c_int
                    || byte(p) != NUL
                    || tab_number == 0
                {
                    invarg2(excmd);
                    break 'theend;
                }
                // `int` arithmetic on a number the user typed: the C
                // wraps, and the range check below is what refuses
                // whatever comes out. `:tabmove -2147483648` is the
                // case that reaches it.
                tab_number = tab_number
                    .wrapping_mul(relative)
                    .wrapping_add(tab_index(TabPage::current()));
                // `:tabmove -1` moves *before* the tab to the left,
                // which is one place further than counting says.
                if unaccept_arg0 == 0 && relative == -1 {
                    tab_number = tab_number.wrapping_sub(1);
                }
            }
            if tab_number < unaccept_arg0 || tab_number > last_tab() {
                invarg2(excmd);
            }
        } else if excmd.addr_count > 0 {
            if unaccept_arg0 != 0 && excmd.line2 == 0 {
                excmd.errmsg = Some(ex_msg(e_invrange.as_ptr()));
                tab_number = 0;
            } else {
                tab_number = excmd.line2 as c_int;
                if unaccept_arg0 == 0 {
                    // `:-tabmove` is spelled as a range, so the sign has
                    // to be read back off the command line — the range
                    // parser has already turned it into a number.
                    let mut at = excmd.line.cmd;
                    loop {
                        at -= 1;
                        if !(at > 0
                            && (ascii_iswhite(c_int::from(excmd.line.byte_at(at)))
                                || ascii_isdigit(c_int::from(excmd.line.byte_at(at)))))
                        {
                            break;
                        }
                    }
                    if excmd.line.byte_at(at) == b'-' {
                        tab_number = tab_number.wrapping_sub(1);
                        if tab_number < unaccept_arg0 {
                            excmd.errmsg = Some(ex_msg(e_invrange.as_ptr()));
                        }
                    }
                }
            }
        } else {
            // No argument at all.
            tab_number = if excmd.cmdidx == CmdIdx::tabnext {
                let next = tab_index(TabPage::current()) + 1;
                if next > last_tab() { 1 } else { next }
            } else if excmd.cmdidx == CmdIdx::tabmove {
                last_tab()
            } else {
                tab_index(TabPage::current())
            };
        }
    }
    tab_number
}

/// Refuse to leave when the argument list has files nobody has edited yet.
///
/// Answers `OK` when quitting is allowed. `quitmore` is what makes the
/// second `:q` work: the refusal sets it, and `do_one_cmd` counts it down.
pub(crate) fn check_more(message: bool, forceit: bool) -> c_int {
    let n =
        unsafe { (*Win::current().w_alist).al_ga.len() as c_int } - Win::current().w_arg_idx - 1;
    if forceit
        || !only_one_window()
        || unsafe { (*Win::current().w_alist).al_ga.len() as c_int } <= 1
        || arg_had_last.get()
        || n <= 0
        || quitmore.get() != 0
    {
        return OK;
    }
    if !message {
        return FAIL;
    }
    if (p_confirm() || cmdmod_has(CmdModFlags::CONFIRM)) && !Buf::current().name.is_unnamed() {
        let mut buff: [c_char; 1000] = [0; 1000];
        let fmt = ngettext(
            c"%d more file to edit.  Quit anyway?",
            c"%d more files to edit.  Quit anyway?",
            n as c_ulong,
        )
        .as_ptr();
        unsafe {
            vim_snprintf(
                &raw mut buff as *mut c_char,
                DIALOG_MSG_SIZE as size_t,
                fmt,
                n,
            )
        };
        let answer = unsafe {
            vim_dialog_yesno(
                VIM_QUESTION as c_int,
                ptr::null_mut(),
                &raw mut buff as *mut c_char,
                1,
            )
        };
        return if answer == VIM_YES as c_int { OK } else { FAIL };
    }
    let fmt = ngettext(
        c"E173: %d more file to edit",
        c"E173: %d more files to edit",
        n as c_ulong,
    );
    emsg_text(tr_plural!(fmt, n));
    quitmore.set(2);
    FAIL
}

/// `mkdir`, reporting the reason it failed.
///
/// # Safety
///
/// `name` must point at a NUL-terminated string.
pub unsafe fn vim_mkdir_emsg(name: *const c_char, prot: c_int) -> Result<(), Failed> {
    let ret = unsafe { os_mkdir(cstr::at(name), prot as int32_t) };
    if ret != 0 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
        let (name, arg1) = unsafe { (c_str(name), c_str(uv_strerror(ret))) };
        semsg!("E739: Cannot create directory {name}: {arg1}");
        return Err(Failed);
    }
    Ok(())
}

/// Open the file `:mkvimrc` and friends are about to write.
///
/// Appending is always allowed; creating over an existing file needs the
/// command's `!`.
///
/// # Safety
///
/// `fname` must point at a NUL-terminated string, unaliased for the call.
/// `mode` must point at a NUL-terminated string, unaliased for the call.
pub unsafe fn open_exfile(fname: *mut c_char, forceit: c_int, mode: *mut c_char) -> *mut FILE {
    if unsafe { os_isdir(fname) } {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let fname = unsafe { c_str(fname) };
        semsg!("E17: \"{fname}\" is a directory");
        return ptr::null_mut();
    }
    if forceit == 0 && byte(mode) != 'a' as c_int && unsafe { os_path_exists(fname) } {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let fname = unsafe { c_str(fname) };
        semsg!("E189: \"{fname}\" exists (add ! to override)");
        return ptr::null_mut();
    }
    let fd = unsafe { os_fopen(fname, mode) };
    if fd.is_null() {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let fname = unsafe { c_str(fname) };
        semsg!("E190: Cannot open \"{fname}\" for writing");
    }
    fd
}

/// Fill in a dialog message with the file name it is about, or `Untitled`.
///
/// # Safety
///
/// `buff` must point at a NUL-terminated string, unaliased for the call.
/// `format` must point at a NUL-terminated string, unaliased for the call.
/// `fname` must point at a NUL-terminated string, unaliased for the call.
pub unsafe fn dialog_msg(buff: *mut c_char, format: *mut c_char, fname: *mut c_char) {
    let fname = if fname.is_null() {
        gettext(c"Untitled".as_ptr())
    } else {
        fname
    };
    unsafe { vim_snprintf(buff, DIALOG_MSG_SIZE as size_t, format, fname) };
}

/// Whether two NUL-terminated strings agree over their first `n` bytes --
/// `cstr::prefix_eq(a, b, n)` -- as checked code.
fn prefix_eq(a: *const c_char, b: *const c_char, n: usize) -> bool {
    // SAFETY: two NUL-terminated strings; each scan stops at its terminator.
    unsafe { cstr::prefix_eq(a, b, n) }
}

/// `strncmp()`'s prefix test as checked code.
fn starts_with(p: *const c_char, prefix: &[u8]) -> bool {
    // SAFETY: a NUL-terminated string; the scan stops at its terminator.
    unsafe { cstr::starts_with(p, prefix) }
}

/// `ex_errmsg()` as checked code.
fn ex_errmsg(msg_0: *const c_char, arg: *const c_char) -> CString {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::ex_docmd::source::ex_errmsg(msg_0, arg) }
}

/// `ex_msg()` as checked code.
fn ex_msg(msg: *const c_char) -> CString {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::ex_docmd::ex_msg(msg) }
}

/// `expand_generic()` as checked code.
#[allow(clippy::too_many_arguments)]
fn expand_generic(
    pat: *const c_char,
    expand: *mut Expand,
    regmatch: *mut RegMatch,
    matches: *mut *mut *mut c_char,
    num_matches: *mut c_int,
    func: CompleteListItemGetter,
    escaped: bool,
) {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe {
        crate::cmdexpand::expand_generic(pat, expand, regmatch, matches, num_matches, func, escaped)
    }
}

/// `gettext()` as checked code.
fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    // SAFETY: a NUL-terminated message; `gettext` answers one too.
    unsafe { crate::os::cshim::gettext_ptr(__msgid).as_ptr().cast_mut() }
}

/// `skip_cmd_arg()` as checked code.
fn skip_cmd_arg(p: *mut c_char, rembs: bool) -> *mut c_char {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::ex_docmd::scan::skip_cmd_arg(p, rembs) }
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

/// Whether the string at `p` is exactly `lit` -- `strcmp(p, lit) == 0` --
/// as checked code.
fn equals(p: *const c_char, lit: &[u8]) -> bool {
    // SAFETY: a NUL-terminated string.
    unsafe { cstr::eq_bytes(p, lit) }
}

/// `strcasecmp()` as checked code.
fn strcasecmp(a: *const c_char, b: *const c_char) -> c_int {
    // SAFETY: two NUL-terminated strings.
    unsafe { ::libc::strcasecmp(a, b) }
}
