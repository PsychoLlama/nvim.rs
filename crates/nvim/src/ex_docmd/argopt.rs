//! Arguments that are not file names: `++opt=value`, `+cmd`, the tab page
//! argument, and opening the file a command will write to.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int, c_ulong};
use core::ptr;

use crate::ascii::{ascii_isdigit, ascii_isspace, ascii_iswhite};
use crate::charset::{getdigits, skipwhite};
use crate::cmdexpand::ExpandGeneric;
use crate::event::libuv::uv_strerror;
use crate::ex_docmd::lookup::checkforcmd;
use crate::ex_docmd::scan::skip_cmd_arg;
use crate::ex_docmd::source::ex_errmsg;
use crate::ex_docmd::window::current_tab_nr;
use crate::ex_docmd::{
    BAD_DROP, BAD_KEEP, DIALOG_MSG_SIZE, FORCE_BIN, FORCE_NOBIN, VIM_QUESTION, VIM_YES, cmdmod_has,
    dollar_command, quitmore,
};
use crate::main::{
    arg_had_last, curbuf, curtab, curwin, e_invarg2, e_invargval, e_invrange, e_isadir2, e_mkdir,
    lastused_tabpage, p_confirm,
};
use crate::mbyte::{get_encoding_name, utf8len_tab};
use crate::memory::{xmalloc, xstrdup};
use crate::message::vim_dialog_yesno;
use crate::optionstr::{check_ff_value, get_fileformat_name};
use crate::os::cshim::{gettext, ngettext, strncmp};
use crate::os::fs::{os_fopen, os_isdir, os_mkdir, os_path_exists};
use crate::strings::vim_snprintf;
use crate::types::regexp::regmatch_T;
use crate::types::{
    CMD_tabmove, CMD_tabnext, CmdModFlags, CompleteListItemGetter, FAIL, FILE, NUL, OK, exarg_T,
    expand_T, int32_t, intmax_t, size_t, uint8_t,
};
use crate::window::{only_one_window, tabpage_index, valid_tabpage};
use ::libc::{strcasecmp, strcmp};

/// Take a `+cmd` argument, and answer the command it names.
///
/// `+` alone means `$`, the last line. The command runs to the end of the
/// argument unless a space that is not backslash-escaped ends it, which is
/// what `skip_cmd_arg` finds; the byte after it is overwritten with a
/// terminator, so the answer borrows the command line.
pub unsafe fn getargcmd(argp: *mut *mut c_char) -> *mut c_char {
    unsafe {
        let mut arg = *argp;
        if *arg as c_int != '+' as c_int {
            return ptr::null_mut();
        }
        arg = arg.add(1);
        let command;
        if ascii_isspace(*arg as c_int) || *arg as c_int == NUL {
            command = dollar_command.ptr() as *mut c_char;
        } else {
            command = arg;
            arg = skip_cmd_arg(command, true);
            if *arg as c_int != NUL {
                *arg = NUL as c_char;
                arg = arg.add(1);
            }
        }
        *argp = skipwhite(arg);
        command
    }
}

/// Read the value of `++bad=`: `keep`, `drop`, or one single-byte
/// replacement character.
pub unsafe fn get_bad_opt(p: *const c_char, eap: *mut exarg_T) -> c_int {
    unsafe {
        if strcasecmp(p as *mut c_char, c"keep".as_ptr() as *mut c_char) == 0 {
            (*eap).bad_char = BAD_KEEP;
        } else if strcasecmp(p as *mut c_char, c"drop".as_ptr() as *mut c_char) == 0 {
            (*eap).bad_char = BAD_DROP;
        } else if utf8len_tab[*p as uint8_t as usize] == 1 && *p.add(1) as c_int == NUL {
            (*eap).bad_char = *p as uint8_t as c_int;
        } else {
            return FAIL;
        }
        OK
    }
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
pub unsafe fn getargopt(eap: *mut exarg_T) -> c_int {
    unsafe {
        let ea = &mut *eap;
        let mut arg = ea.arg.add(2);
        let mut bad_char_idx: c_int = 0;

        // `++bin`/`++nobin` and `++binary`/`++nobinary`.
        if strncmp(arg, c"bin".as_ptr(), 3) == 0 || strncmp(arg, c"nobin".as_ptr(), 5) == 0 {
            if *arg as c_int == 'n' as c_int {
                arg = arg.add(2);
                ea.force_bin = FORCE_NOBIN;
            } else {
                ea.force_bin = FORCE_BIN;
            }
            if !checkforcmd(&raw mut arg, c"binary".as_ptr(), 3) {
                return FAIL;
            }
            ea.arg = skipwhite(arg);
            return OK;
        }

        // `++edit`, and not `++editsomething`.
        if strncmp(arg, c"edit".as_ptr(), 4) == 0 && !(*arg.add(4) as u8).is_ascii_alphabetic() {
            ea.read_edit = 1;
            ea.arg = skipwhite(arg.add(4));
            return OK;
        }

        // `++p`, and not `++psomething`.
        if *arg as c_int == 'p' as c_int && !(*arg.add(1) as u8).is_ascii_alphabetic() {
            ea.mkdir_p = 1;
            ea.arg = skipwhite(arg.add(1));
            return OK;
        }

        let pp: *mut c_int = if strncmp(arg, c"ff".as_ptr(), 2) == 0 {
            arg = arg.add(2);
            &raw mut ea.force_ff
        } else if strncmp(arg, c"fileformat".as_ptr(), 10) == 0 {
            arg = arg.add(10);
            &raw mut ea.force_ff
        } else if strncmp(arg, c"enc".as_ptr(), 3) == 0 {
            arg = arg.add(if strncmp(arg, c"encoding".as_ptr(), 8) == 0 {
                8
            } else {
                3
            });
            &raw mut ea.force_enc
        } else if strncmp(arg, c"bad".as_ptr(), 3) == 0 {
            arg = arg.add(3);
            &raw mut bad_char_idx
        } else {
            ptr::null_mut()
        };

        if pp.is_null() || *arg as c_int != '=' as c_int {
            return FAIL;
        }
        arg = arg.add(1);
        *pp = arg.offset_from(ea.cmd) as c_int;
        arg = skip_cmd_arg(arg, false);
        ea.arg = skipwhite(arg);
        *arg = NUL as c_char;

        if pp == &raw mut ea.force_ff {
            if check_ff_value(ea.cmd.offset(ea.force_ff as isize)) == FAIL {
                return FAIL;
            }
            // Only the first letter is kept: 'u', 'd' or 'm'.
            ea.force_ff = *ea.cmd.offset(ea.force_ff as isize) as uint8_t as c_int;
        } else if pp == &raw mut ea.force_enc {
            let mut p = ea.cmd.offset(ea.force_enc as isize);
            while *p as c_int != NUL {
                *p = (*p as u8).to_ascii_lowercase() as c_char;
                p = p.add(1);
            }
        } else if get_bad_opt(ea.cmd.offset(bad_char_idx as isize), eap) == FAIL {
            return FAIL;
        }
        OK
    }
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
) -> c_int {
    unsafe {
        let x = &mut *xp;
        // Past an `=`: complete the value, by whichever option name ends
        // right before it.
        if x.xp_pattern > x.xp_line && *x.xp_pattern.offset(-1) as c_int == '=' as c_int {
            let name_end = x.xp_pattern.offset(-1);
            let ends_with = |word: &CStr| {
                let n = word.to_bytes().len() as isize;
                name_end.offset_from(x.xp_line) >= n
                    && strncmp(name_end.offset(-n), word.as_ptr(), n as size_t) == 0
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
                return FAIL;
            }
            ExpandGeneric(pat, xp, rmp, matches, num_matches, cb, false);
            return OK;
        }
        // `++ff` is the only abbreviation worth finishing on its own.
        if x.xp_pattern_len == 2 && strncmp(x.xp_pattern, c"ff".as_ptr(), x.xp_pattern_len) == 0 {
            *matches = xmalloc(size_of::<*mut c_char>()) as *mut *mut c_char;
            *num_matches = 1;
            **matches = xstrdup(c"fileformat=".as_ptr());
            return OK;
        }
        ExpandGeneric(
            pat,
            xp,
            rmp,
            matches,
            num_matches,
            Some(get_argopt_name),
            false,
        );
        OK
    }
}

/// Which tab page a `:tab…` command means.
///
/// Four spellings, and they do not agree on what counts as tab 0:
/// `:tabmove 0` moves before the first tab and is legal, every other
/// command refuses it — that is `unaccept_arg0`. An argument may be
/// absolute (`3`, `$`, `#`), relative (`+2`, `-1`), a range before the
/// command (`:2tabnext`), or absent.
pub(crate) unsafe fn get_tabpage_arg(eap: *mut exarg_T) -> c_int {
    unsafe {
        let ea = &mut *eap;
        let mut tab_number: c_int = 0;
        let unaccept_arg0 = if ea.cmdidx as c_int == CMD_tabmove as c_int {
            0
        } else {
            1
        };
        let last_tab = || current_tab_nr(ptr::null_mut());
        let invarg2 = |ea: &mut exarg_T| {
            ea.errmsg = ex_errmsg(&raw const e_invarg2 as *const c_char, ea.arg);
        };

        'theend: {
            if !ea.arg.is_null() && *ea.arg as c_int != NUL {
                let mut p = ea.arg;
                // `+N`/`-N` means N places to the right/left of here.
                let relative = match *p as c_int {
                    c if c == '-' as c_int => {
                        p = p.add(1);
                        -1
                    }
                    c if c == '+' as c_int => {
                        p = p.add(1);
                        1
                    }
                    _ => 0,
                };

                let p_save = p;
                tab_number = getdigits(&raw mut p, false, tab_number as intmax_t) as c_int;

                if relative == 0 {
                    if strcmp(p, c"$".as_ptr()) == 0 {
                        tab_number = last_tab();
                    } else if strcmp(p, c"#".as_ptr()) == 0 {
                        if !valid_tabpage(lastused_tabpage.get()) {
                            ea.errmsg = ex_errmsg(&raw const e_invargval as *const c_char, ea.arg);
                            tab_number = 0;
                            break 'theend;
                        }
                        tab_number = tabpage_index(lastused_tabpage.get());
                    } else if p == p_save
                        || *p_save as c_int == '-' as c_int
                        || *p as c_int != NUL
                        || tab_number > last_tab()
                    {
                        // Not a number.
                        invarg2(ea);
                        break 'theend;
                    }
                } else {
                    if *p_save as c_int == NUL {
                        // A bare `+` or `-` is one place.
                        tab_number = 1;
                    } else if p == p_save
                        || *p_save as c_int == '-' as c_int
                        || *p as c_int != NUL
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
                    ea.errmsg = gettext(&raw const e_invrange as *const c_char);
                    tab_number = 0;
                } else {
                    tab_number = ea.line2 as c_int;
                    if unaccept_arg0 == 0 {
                        // `:-tabmove` is spelled as a range, so the sign has
                        // to be read back off the command line — the range
                        // parser has already turned it into a number.
                        let mut cmdp = ea.cmd;
                        loop {
                            cmdp = cmdp.offset(-1);
                            if !(cmdp > *ea.cmdlinep
                                && (ascii_iswhite(*cmdp as c_int) || ascii_isdigit(*cmdp as c_int)))
                            {
                                break;
                            }
                        }
                        if *cmdp as c_int == '-' as c_int {
                            tab_number = tab_number.wrapping_sub(1);
                            if tab_number < unaccept_arg0 {
                                ea.errmsg = gettext(&raw const e_invrange as *const c_char);
                            }
                        }
                    }
                }
            } else {
                // No argument at all.
                tab_number = if ea.cmdidx as c_int == CMD_tabnext as c_int {
                    let next = tabpage_index(curtab.get()) + 1;
                    if next > last_tab() { 1 } else { next }
                } else if ea.cmdidx as c_int == CMD_tabmove as c_int {
                    last_tab()
                } else {
                    tabpage_index(curtab.get())
                };
            }
        }
        tab_number
    }
}

/// Refuse to leave when the argument list has files nobody has edited yet.
///
/// Answers `OK` when quitting is allowed. `quitmore` is what makes the
/// second `:q` work: the refusal sets it, and `do_one_cmd` counts it down.
pub(crate) unsafe fn check_more(message: bool, forceit: bool) -> c_int {
    unsafe {
        let n = (*(*curwin.get()).w_alist).al_ga.ga_len - (*curwin.get()).w_arg_idx - 1;
        if forceit
            || !only_one_window()
            || (*(*curwin.get()).w_alist).al_ga.ga_len <= 1
            || arg_had_last.get()
            || n <= 0
            || quitmore.get() != 0
        {
            return OK;
        }
        if !message {
            return FAIL;
        }
        if (p_confirm.get() != 0 || cmdmod_has(CmdModFlags::CONFIRM))
            && !(*curbuf.get()).b_fname.is_null()
        {
            let mut buff: [c_char; 1000] = [0; 1000];
            vim_snprintf(
                &raw mut buff as *mut c_char,
                DIALOG_MSG_SIZE as size_t,
                ngettext(
                    c"%d more file to edit.  Quit anyway?".as_ptr(),
                    c"%d more files to edit.  Quit anyway?".as_ptr(),
                    n as c_ulong,
                ),
                n,
            );
            let answer = vim_dialog_yesno(
                VIM_QUESTION as c_int,
                ptr::null_mut(),
                &raw mut buff as *mut c_char,
                1,
            );
            return if answer == VIM_YES as c_int { OK } else { FAIL };
        }
        semsg_c!(
            ngettext(
                c"E173: %d more file to edit".as_ptr(),
                c"E173: %d more files to edit".as_ptr(),
                n as c_ulong,
            ),
            n,
        );
        quitmore.set(2);
        FAIL
    }
}

/// `mkdir`, reporting the reason it failed.
pub unsafe fn vim_mkdir_emsg(name: *const c_char, prot: c_int) -> c_int {
    unsafe {
        let ret = os_mkdir(name, prot as int32_t);
        if ret != 0 {
            semsg_c!(
                gettext(&raw const e_mkdir as *const c_char),
                name,
                uv_strerror(ret),
            );
            return FAIL;
        }
        OK
    }
}

/// Open the file `:mkvimrc` and friends are about to write.
///
/// Appending is always allowed; creating over an existing file needs the
/// command's `!`.
pub unsafe fn open_exfile(fname: *mut c_char, forceit: c_int, mode: *mut c_char) -> *mut FILE {
    unsafe {
        if os_isdir(fname) {
            semsg_c!(gettext(&raw const e_isadir2 as *const c_char), fname);
            return ptr::null_mut();
        }
        if forceit == 0 && *mode as c_int != 'a' as c_int && os_path_exists(fname) {
            semsg_c!(
                gettext(c"E189: \"%s\" exists (add ! to override)".as_ptr()),
                fname,
            );
            return ptr::null_mut();
        }
        let fd = os_fopen(fname, mode);
        if fd.is_null() {
            semsg_c!(
                gettext(c"E190: Cannot open \"%s\" for writing".as_ptr()),
                fname,
            );
        }
        fd
    }
}

/// Fill in a dialog message with the file name it is about, or `Untitled`.
pub unsafe fn dialog_msg(buff: *mut c_char, format: *mut c_char, fname: *mut c_char) {
    unsafe {
        let fname = if fname.is_null() {
            gettext(c"Untitled".as_ptr())
        } else {
            fname
        };
        vim_snprintf(buff, DIALOG_MSG_SIZE as size_t, format, fname);
    }
}
