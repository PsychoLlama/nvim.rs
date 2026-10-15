//! Arguments that are not file names: `++opt=value`, `+cmd`, the tab page
//! argument, and opening the file a command will write to.
#![forbid(unsafe_code)]
use crate::cmdexpand::expand_generic;
use crate::ex_docmd::ex_msg;
use crate::ex_docmd::source::ex_errmsg;
use crate::memory::XString;
use crate::types::Candidate;
use crate::types::CmdIdx;
use crate::window::tab_index;
use crate::winlayer::TabPage;
use crate::winlayer::last_used_tab;

use crate::semsg;
use crate::tr_plural;
use crate::winlayer::{Buf, Win};

/// The completion context, whose caller has promised it outlives the value.
use core::ffi::{CStr, c_int, c_ulong};

use crate::ascii::{ascii_isdigit, ascii_isspace, ascii_iswhite};
use crate::charset::getdigits_at;

use crate::ex_docmd::lookup::check_for_word;

use crate::arglist::state::ARG_HAD_LAST;
use crate::ex_docmd::window::current_tab_nr;
use crate::ex_docmd::{
    BAD_DROP, BAD_KEEP, DIALOG_MSG_SIZE, FORCE_BIN, FORCE_NOBIN, VIM_QUESTION, VIM_YES, cmdmod_has,
    quitmore,
};
use crate::mbyte::{get_encoding_name, utf8len_tab};
use crate::message::vim_dialog_yesno;
use crate::message::{e_invarg2, e_invargval, e_invrange};
use crate::message_fmt::{emsg_text, msg_cstr};
use crate::option::vars::p_confirm;
use crate::optionstr::{check_ff_value, get_fileformat_name};
use crate::os::cshim::ngettext;

use crate::os::fs::{CFile, dir_exists, os_mkdir, os_strerror, path_exists};

use crate::types::regexp::RegMatch;
use crate::types::{
    CmdLine, CmdModFlags, CompleteListItemGetter, EcmdCmd, ExArg, Expand, FAIL, Failed, OK,
    int32_t, intmax_t,
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
pub(crate) fn get_bad_opt(value: &[u8]) -> Result<c_int, Failed> {
    if value.eq_ignore_ascii_case(b"keep") {
        Ok(BAD_KEEP)
    } else if value.eq_ignore_ascii_case(b"drop") {
        Ok(BAD_DROP)
    } else if let [byte] = value
        && utf8len_tab[usize::from(*byte)] == 1
    {
        Ok(c_int::from(*byte))
    } else {
        Err(Failed)
    }
}

/// The completion candidates for `++bad=`.
pub(crate) fn get_bad_name(_expand: &Expand, idx: usize) -> Option<Candidate> {
    const VALUES: [&CStr; 3] = [c"?", c"keep", c"drop"];
    VALUES.get(idx).map(|&v| Candidate::Borrowed(v))
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
            // The value, NUL-terminated by the write just above.
            if check_ff_value(excmd.line.cstr_from(at)) == FAIL {
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
            excmd.bad_char = get_bad_opt(excmd.line.rest_of(at))?;
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
    crate::ex_docmd::scan::skip_cmd_arg(line, at, rembs)
}

/// The completion candidates for `++`.
pub(crate) fn get_argopt_name(_expand: &Expand, idx: usize) -> Option<Candidate> {
    const VALUES: [&CStr; 7] = [
        c"fileformat=",
        c"encoding=",
        c"binary",
        c"nobinary",
        c"bad=",
        c"edit",
        c"p",
    ];
    VALUES.get(idx).map(|&v| Candidate::Borrowed(v))
}

/// Complete a `++opt` argument: the option names, or the values of the one
/// already typed.
pub fn expand_argopt(
    pat: &CStr,
    expand: &Expand,
    rmp: &mut RegMatch,
) -> Result<Vec<XString>, Failed> {
    // Past an `=`: complete the value, by whichever option name ends
    // right before it.
    let at = expand.pattern;
    if at > 0 && expand.line.get(at - 1) == Some(&b'=') {
        let name = &expand.line[..at - 1];
        let cb: CompleteListItemGetter = if name.ends_with(b"ff") || name.ends_with(b"fileformat") {
            get_fileformat_name
        } else if name.ends_with(b"enc") || name.ends_with(b"encoding") {
            get_encoding_name
        } else if name.ends_with(b"bad") {
            get_bad_name
        } else {
            return Err(Failed);
        };
        return Ok(expand_generic(pat, expand, rmp, cb, false));
    }
    // `++ff` is the only abbreviation worth finishing on its own.
    if expand.pattern_len == 2 && expand.pattern_starts_with(b"ff") {
        return Ok(vec![XString::from_cstr(c"fileformat=")]);
    }
    Ok(expand_generic(pat, expand, rmp, get_argopt_name, false))
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
        command.errmsg = Some(ex_errmsg(
            e_invarg2,
            command.line.cstr_from(command.line.arg),
        ));
    };

    'theend: {
        if excmd.line.byte_at(excmd.line.arg) != 0 {
            let mut p = excmd.line.arg;
            // `+N`/`-N` means N places to the right/left of here.
            let relative = match excmd.line.byte_at(p) {
                b'-' => {
                    p += 1;
                    -1
                }
                b'+' => {
                    p += 1;
                    1
                }
                _ => 0,
            };

            let p_save = p;
            let (number, past) = getdigits_at(excmd.line.tail(0), p, false, tab_number as intmax_t);
            tab_number = number as c_int;
            p = past;

            if relative == 0 {
                if excmd.line.rest_of(p) == b"$" {
                    tab_number = last_tab();
                } else if excmd.line.rest_of(p) == b"#" {
                    if last_used_tab().is_none() {
                        excmd.errmsg =
                            Some(ex_errmsg(e_invargval, excmd.line.cstr_from(excmd.line.arg)));
                        tab_number = 0;
                        break 'theend;
                    }
                    tab_number = tabpage_index(last_used_tab());
                } else if p == p_save
                    || excmd.line.byte_at(p_save) == b'-'
                    || excmd.line.byte_at(p) != 0
                    || tab_number > last_tab()
                {
                    // Not a number.
                    invarg2(excmd);
                    break 'theend;
                }
            } else {
                if excmd.line.byte_at(p_save) == 0 {
                    // A bare `+` or `-` is one place.
                    tab_number = 1;
                } else if p == p_save
                    || excmd.line.byte_at(p_save) == b'-'
                    || excmd.line.byte_at(p) != 0
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
                excmd.errmsg = Some(ex_msg(e_invrange));
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
                            excmd.errmsg = Some(ex_msg(e_invrange));
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
    let n = crate::arglist::argcount() - Win::current().w_arg_idx - 1;
    if forceit
        || !only_one_window()
        || crate::arglist::argcount() <= 1
        || ARG_HAD_LAST.get()
        || n <= 0
        || quitmore.get() != 0
    {
        return OK;
    }
    if !message {
        return FAIL;
    }
    if (p_confirm() || cmdmod_has(CmdModFlags::CONFIRM)) && !Buf::current().name.is_unnamed() {
        let fmt = ngettext(
            c"%d more file to edit.  Quit anyway?",
            c"%d more files to edit.  Quit anyway?",
            n as c_ulong,
        );
        let room = DIALOG_MSG_SIZE as usize;
        let question = XString::from_bytes(&crate::formatted!(room, fmt.to_bytes(), n));
        let answer = vim_dialog_yesno(VIM_QUESTION as c_int, None, question.as_cstr(), 1);
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
pub fn vim_mkdir_emsg(name: &CStr, prot: c_int) -> Result<(), Failed> {
    let ret = os_mkdir(name, prot as int32_t);
    if ret != 0 {
        let (name, arg1) = (msg_cstr(name), msg_cstr(os_strerror(ret)));
        semsg!("E739: Cannot create directory {name}: {arg1}");
        return Err(Failed);
    }
    Ok(())
}

/// Whether `:mkvimrc` and friends may write `fname`: not a directory, and
/// -- unless appending, or with the command's `!` -- not a file that exists.
/// Says why not.
fn exfile_writable(fname: &CStr, forceit: bool, appending: bool) -> bool {
    if dir_exists(fname) {
        let fname = msg_cstr(fname);
        semsg!("E17: \"{fname}\" is a directory");
        return false;
    }
    if !forceit && !appending && path_exists(fname) {
        let fname = msg_cstr(fname);
        semsg!("E189: \"{fname}\" exists (add ! to override)");
        return false;
    }
    true
}

/// "E190", for a file that would not open.
fn cannot_open_exfile(fname: &CStr) {
    let fname = msg_cstr(fname);
    semsg!("E190: Cannot open \"{fname}\" for writing");
}

/// Open the file `:redir` is about to write.
///
/// Appending is always allowed; creating over an existing file needs the
/// command's `!`.
pub fn open_exfile(fname: &CStr, forceit: c_int, mode: &CStr) -> Option<CFile> {
    if !exfile_writable(fname, forceit != 0, mode.to_bytes().first() == Some(&b'a')) {
        return None;
    }
    let file = CFile::open(fname, mode);
    if file.is_none() {
        cannot_open_exfile(fname);
    }
    file
}

/// Create the file `:mkvimrc`, `:mkview` and `:mksession` write, truncating
/// one that exists (which needs the command's `!`) -- upstream's
/// `open_exfile(.., "wb")`.
pub(crate) fn create_exfile(fname: &CStr, forceit: bool) -> Option<std::fs::File> {
    if !exfile_writable(fname, forceit, false) {
        return None;
    }
    use std::os::unix::ffi::OsStrExt;
    let file = std::fs::File::create(std::ffi::OsStr::from_bytes(fname.to_bytes())).ok();
    if file.is_none() {
        cannot_open_exfile(fname);
    }
    file
}

/// A dialog message: the one-`%s` `format` filled in with the file name it
/// is about, or `Untitled`, cut where upstream's `DIALOG_MSG_SIZE` buffer
/// cut it.
pub(crate) fn dialog_msg(format: &CStr, fname: Option<&CStr>) -> XString {
    let fname = fname.unwrap_or_else(|| crate::os::cshim::gettext(c"Untitled"));
    let room = DIALOG_MSG_SIZE as usize;
    XString::from_bytes(&crate::formatted!(room, format.to_bytes(), fname))
}
