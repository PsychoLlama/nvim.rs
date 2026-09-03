//! Arguments that are not file names: `++opt=value`, `+cmd`, the tab page
//! argument, and opening the file a command will write to.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::cstr;
use crate::strings::vim_snprintf;
use crate::types::CmdIdx;

use std::ffi::CString;

use crate::semsg;
use crate::tr_plural;
use crate::winlayer::{Buf, Ea, Live, Win};

/// The completion context, whose caller has promised it outlives the value.
type Xp = Live<expand_T>;
use core::ffi::{CStr, c_char, c_int, c_ulong};
use core::ptr;

use crate::ascii::{ascii_isdigit, ascii_isspace, ascii_iswhite};
use crate::charset::getdigits;

use crate::event::libuv::uv_strerror;

use crate::ex_docmd::lookup::checkforcmd;

use crate::ex_docmd::window::current_tab_nr;
use crate::ex_docmd::{
    BAD_DROP, BAD_KEEP, DIALOG_MSG_SIZE, FORCE_BIN, FORCE_NOBIN, VIM_QUESTION, VIM_YES, cmdmod_has,
    dollar_command, quitmore,
};
use crate::main::{
    arg_had_last, curtab, e_invarg2, e_invargval, e_invrange, lastused_tabpage, p_confirm,
};
use crate::mbyte::{get_encoding_name, utf8len_tab};
use crate::memory::{xmalloc, xstrdup};
use crate::message::vim_dialog_yesno;
use crate::message_fmt::{c_str, emsg_text};
use crate::optionstr::{check_ff_value, get_fileformat_name};
use crate::os::cshim::ngettext;

use crate::os::fs::{os_fopen, os_isdir, os_mkdir, os_path_exists};

use crate::types::regexp::regmatch_T;
use crate::types::{
    CmdModFlags, CompleteListItemGetter, FAIL, FILE, Failed, NUL, OK, exarg_T, expand_T, int32_t,
    intmax_t, size_t,
};
use crate::window::{only_one_window, tabpage_index, valid_tabpage};

/// Take a `+cmd` argument, and answer the command it names.
///
/// `+` alone means `$`, the last line. The command runs to the end of the
/// argument unless a space that is not backslash-escaped ends it, which is
/// what `skip_cmd_arg` finds; the byte after it is overwritten with a
/// terminator, so the answer borrows the command line.
pub unsafe fn getargcmd(argp: *mut *mut c_char) -> *mut c_char {
    let mut arg = unsafe { *argp };
    if byte(arg) != '+' as c_int {
        return ptr::null_mut();
    }
    arg = unsafe { arg.add(1) };
    let command;
    if ascii_isspace(byte(arg)) || byte(arg) == NUL {
        command = dollar_command.as_ptr().cast_mut();
    } else {
        command = arg;
        arg = skip_cmd_arg(command, true);
        if byte(arg) != NUL {
            unsafe { *arg = NUL as c_char };
            arg = unsafe { arg.add(1) };
        }
    }
    unsafe { *argp = skipwhite(arg) };
    command
}

/// Read the value of `++bad=`: `keep`, `drop`, or one single-byte
/// replacement character.
pub(crate) unsafe fn get_bad_opt(p: *const c_char, mut eap: Ea) -> Result<(), Failed> {
    if strcasecmp(p as *mut c_char, c"keep".as_ptr() as *mut c_char) == 0 {
        eap.bad_char = BAD_KEEP;
    } else if strcasecmp(p as *mut c_char, c"drop".as_ptr() as *mut c_char) == 0 {
        eap.bad_char = BAD_DROP;
    } else if utf8len_tab[ubyte(p) as usize] == 1 && byte_at(p, 1) == NUL {
        eap.bad_char = ubyte(p) as c_int;
    } else {
        return Err(Failed);
    }
    Ok(())
}

/// The completion candidates for `++bad=`.
///
/// Keeps the raw signature: installed as a `CompleteListItemGetter`.
pub(crate) fn get_bad_name(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    const VALUES: [&CStr; 3] = [c"?", c"keep", c"drop"];
    match VALUES.get(idx as usize) {
        Some(v) => v.as_ptr() as *mut c_char,
        None => ptr::null_mut(),
    }
}

/// Read one `++opt` or `++opt=value` argument off the front of `eap->arg`.
///
/// The three that take a value are stored as *offsets* into `eap->cmd`
/// rather than as pointers, because the command line is reallocated by the
/// `%`/`#` expansion that runs later; `do_ecmd` and the write path resolve
/// them against the line they end up with.
pub unsafe fn getargopt(eap: *mut exarg_T) -> Result<(), Failed> {
    let mut ea = unsafe { Ea::new(eap) };
    let mut arg = unsafe { ea.arg.add(2) };
    let mut bad_char_idx: c_int = 0;

    // `++bin`/`++nobin` and `++binary`/`++nobinary`.
    if starts_with(arg, b"bin") || starts_with(arg, b"nobin") {
        if byte(arg) == 'n' as c_int {
            arg = unsafe { arg.add(2) };
            ea.force_bin = FORCE_NOBIN;
        } else {
            ea.force_bin = FORCE_BIN;
        }
        if !unsafe { checkforcmd(&raw mut arg, c"binary".as_ptr(), 3) } {
            return Err(Failed);
        }
        ea.arg = skipwhite(arg);
        return Ok(());
    }

    // `++edit`, and not `++editsomething`.
    if starts_with(arg, b"edit") && !(ubyte_at(arg, 4)).is_ascii_alphabetic() {
        ea.read_edit = 1;
        ea.arg = unsafe { skipwhite(arg.add(4)) };
        return Ok(());
    }

    // `++p`, and not `++psomething`.
    if byte(arg) == 'p' as c_int && !(ubyte_at(arg, 1)).is_ascii_alphabetic() {
        ea.mkdir_p = 1;
        ea.arg = unsafe { skipwhite(arg.add(1)) };
        return Ok(());
    }

    let pp: *mut c_int = if starts_with(arg, b"ff") {
        arg = unsafe { arg.add(2) };
        ea.force_ff_ptr()
    } else if starts_with(arg, b"fileformat") {
        arg = unsafe { arg.add(10) };
        ea.force_ff_ptr()
    } else if starts_with(arg, b"enc") {
        arg = unsafe { arg.add(if starts_with(arg, b"encoding") { 8 } else { 3 }) };
        ea.force_enc_ptr()
    } else if starts_with(arg, b"bad") {
        arg = unsafe { arg.add(3) };
        &raw mut bad_char_idx
    } else {
        ptr::null_mut()
    };

    if pp.is_null() || byte(arg) != '=' as c_int {
        return Err(Failed);
    }
    arg = unsafe { arg.add(1) };
    unsafe { *pp = arg.offset_from(ea.cmd) as c_int };
    arg = skip_cmd_arg(arg, false);
    ea.arg = skipwhite(arg);
    unsafe { *arg = NUL as c_char };

    if pp == ea.force_ff_ptr() {
        if unsafe { check_ff_value(ea.cmd.offset(ea.force_ff as isize)) } == FAIL {
            return Err(Failed);
        }
        // Only the first letter is kept: 'u', 'd' or 'm'.
        ea.force_ff = ubyte_at(ea.cmd, ea.force_ff as isize) as c_int;
    } else if pp == ea.force_enc_ptr() {
        let mut p = unsafe { ea.cmd.offset(ea.force_enc as isize) };
        while byte(p) != NUL {
            unsafe { *p = (*p as u8).to_ascii_lowercase() as c_char };
            p = unsafe { p.add(1) };
        }
    } else if unsafe { get_bad_opt(ea.cmd.offset(bad_char_idx as isize), ea) }.is_err() {
        return Err(Failed);
    }
    Ok(())
}

/// The completion candidates for `++`.
///
/// Keeps the raw signature: installed as a `CompleteListItemGetter`.
pub(crate) fn get_argopt_name(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
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
pub unsafe fn expand_argopt(
    pat: *mut c_char,
    xp: *mut expand_T,
    rmp: *mut regmatch_T,
    matches: *mut *mut *mut c_char,
    num_matches: *mut c_int,
) -> Result<(), Failed> {
    // SAFETY: the completion context is the caller's, live for the call.
    let mut x = unsafe { Xp::new(xp) };
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
        expand_generic(pat, xp, rmp, matches, num_matches, cb, false);
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
        xp,
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
pub(crate) fn get_tabpage_arg(mut ea: Ea) -> c_int {
    let mut tab_number: c_int = 0;
    let unaccept_arg0 = if ea.cmdidx == CmdIdx::tabmove { 0 } else { 1 };
    let last_tab = || current_tab_nr(ptr::null_mut());
    let invarg2 = |mut ea: Ea| {
        ea.errmsg = Some(ex_errmsg(e_invarg2.as_ptr(), ea.arg));
    };

    'theend: {
        if !ea.arg.is_null() && byte(ea.arg) != NUL {
            let mut p = ea.arg;
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
                    if !valid_tabpage(lastused_tabpage.get()) {
                        ea.errmsg = Some(ex_errmsg(e_invargval.as_ptr(), ea.arg));
                        tab_number = 0;
                        break 'theend;
                    }
                    tab_number = tabpage_index(lastused_tabpage.get());
                } else if p == p_save
                    || byte(p_save) == '-' as c_int
                    || byte(p) != NUL
                    || tab_number > last_tab()
                {
                    // Not a number.
                    invarg2(ea);
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
                    invarg2(ea);
                    break 'theend;
                }
                // `int` arithmetic on a number the user typed: the C
                // wraps, and the range check below is what refuses
                // whatever comes out. `:tabmove -2147483648` is the
                // case that reaches it.
                tab_number = tab_number
                    .wrapping_mul(relative)
                    .wrapping_add(tabpage_index(curtab.get()));
                // `:tabmove -1` moves *before* the tab to the left,
                // which is one place further than counting says.
                if unaccept_arg0 == 0 && relative == -1 {
                    tab_number = tab_number.wrapping_sub(1);
                }
            }
            if tab_number < unaccept_arg0 || tab_number > last_tab() {
                invarg2(ea);
            }
        } else if ea.addr_count > 0 {
            if unaccept_arg0 != 0 && ea.line2 == 0 {
                ea.errmsg = Some(ex_msg(e_invrange.as_ptr()));
                tab_number = 0;
            } else {
                tab_number = ea.line2 as c_int;
                if unaccept_arg0 == 0 {
                    // `:-tabmove` is spelled as a range, so the sign has
                    // to be read back off the command line — the range
                    // parser has already turned it into a number.
                    let mut cmdp = ea.cmd;
                    loop {
                        cmdp = unsafe { cmdp.offset(-1) };
                        if !(cmdp > unsafe { *ea.cmdlinep }
                            && (ascii_iswhite(byte(cmdp)) || ascii_isdigit(byte(cmdp))))
                        {
                            break;
                        }
                    }
                    if byte(cmdp) == '-' as c_int {
                        tab_number = tab_number.wrapping_sub(1);
                        if tab_number < unaccept_arg0 {
                            ea.errmsg = Some(ex_msg(e_invrange.as_ptr()));
                        }
                    }
                }
            }
        } else {
            // No argument at all.
            tab_number = if ea.cmdidx == CmdIdx::tabnext {
                let next = tabpage_index(curtab.get()) + 1;
                if next > last_tab() { 1 } else { next }
            } else if ea.cmdidx == CmdIdx::tabmove {
                last_tab()
            } else {
                tabpage_index(curtab.get())
            };
        }
    }
    tab_number
}

/// Refuse to leave when the argument list has files nobody has edited yet.
///
/// Answers `OK` when quitting is allowed. `quitmore` is what makes the
/// second `:q` work: the refusal sets it, and `do_one_cmd` counts it down.
pub(crate) unsafe fn check_more(message: bool, forceit: bool) -> c_int {
    let n = unsafe { (*cur_win().w_alist).al_ga.len() as c_int } - cur_win().w_arg_idx - 1;
    if forceit
        || !unsafe { only_one_window() }
        || unsafe { (*cur_win().w_alist).al_ga.len() as c_int } <= 1
        || arg_had_last.get()
        || n <= 0
        || quitmore.get() != 0
    {
        return OK;
    }
    if !message {
        return FAIL;
    }
    if (p_confirm.get() != 0 || cmdmod_has(CmdModFlags::CONFIRM)) && !cur_buf().b_fname.is_null() {
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
pub unsafe fn vim_mkdir_emsg(name: *const c_char, prot: c_int) -> Result<(), Failed> {
    let ret = unsafe { os_mkdir(name, prot as int32_t) };
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
pub unsafe fn dialog_msg(buff: *mut c_char, format: *mut c_char, fname: *mut c_char) {
    let fname = if fname.is_null() {
        gettext(c"Untitled".as_ptr())
    } else {
        fname
    };
    unsafe { vim_snprintf(buff, DIALOG_MSG_SIZE as size_t, format, fname) };
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
    xp: *mut expand_T,
    regmatch: *mut regmatch_T,
    matches: *mut *mut *mut c_char,
    numMatches: *mut c_int,
    func: CompleteListItemGetter,
    escaped: bool,
) {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe {
        crate::cmdexpand::expand_generic(pat, xp, regmatch, matches, numMatches, func, escaped)
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
