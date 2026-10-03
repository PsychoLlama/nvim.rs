//! `:helpgrep`, which searches the help files.
//!
//! [`ex_helpgrep`] walks every `doc/` directory in `'runtimepath'`
//! ([`hgr_search_in_rtp`]) and matches the pattern against each help file's
//! lines ([`hgr_search_file`]), building a list without ever loading a
//! buffer.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::option::SavedCpo;
use crate::option::next_option_part;
use crate::option::vars::P_RTP;
use crate::path::{ExpandFlags, expand_wildcards_list, vim_ispathsep};
use crate::regexp::{OwnedProg, RE_MAGIC, RE_STRING};
use crate::semsg;
use crate::types::CmdIdx;
use crate::types::{IOSIZE, MAXPATHL};
use core::ffi::{CStr, c_int};
use std::fs::File;
use std::os::unix::ffi::OsStrExt;

/// The wildcard `:helpgrep` expands in each `'runtimepath'` entry. It is a
/// `\(…\)` alternation because `gen_expand_wildcards` matches it as a
/// regular expression once the shell-style parts are translated.
const HELP_FILES: &[u8] = br"doc/*.\(txt\|??x\)";

/// The location list `:lhelpgrep` adds to: the one of the help window, if
/// there is one, and otherwise a fresh stack — which the caller is told
/// about through `new_ll`, because it has to free it again if nothing ends
/// up pointing at it.
fn hgr_get_ll(new_ll: &mut bool) -> Qi {
    let wp = if is_help_buffer(Win::current()) {
        Some(Win::current())
    } else {
        qf_find_help_win()
    };
    if let Some(existing) = wp.and_then(|wp| wp.w_llist) {
        return existing.stack();
    }
    *new_ll = true;
    new_location_stack(QFLT_LOCATION, 1).stack()
}

/// `vim_fgets`: one line of at most `IOSIZE - 1` bytes, the rest of a longer
/// one read and thrown away. `false` at the end of the file.
fn vim_fgets(file: &mut Fgets, line: &mut Vec<u8>) -> bool {
    let size = IOSIZE as usize;
    if !file.fgets(line, size) {
        return false;
    }
    // The last-but-one byte tells whether the line fitted: `fgets` leaves
    // it alone when the line was shorter than the buffer.
    let filled = |line: &[u8], at: usize| line.get(at).is_some_and(|&c| c != 0 && c != b'\n');
    if filled(line, size - 2) {
        // Throw away the rest of the line.
        let mut rest = Vec::new();
        while file.fgets(&mut rest, 200) && filled(&rest, 200 - 2) {}
    }
    true
}

/// Add an entry for every line of one help file that the pattern matches.
fn hgr_search_file(qfl: Qfl, fname: &CStr, prog: &mut OwnedProg) {
    let Ok(file) = File::open(std::ffi::OsStr::from_bytes(fname.to_bytes())) else {
        return;
    };
    let mut file = Fgets::new(file);
    let mut line = Vec::new();
    let mut lnum: LineNr = 1;
    while !got_int.get() && vim_fgets(&mut file, &mut line) {
        // As C reads the line: up to its first NUL.
        let end = line.iter().position(|&b| b == 0).unwrap_or(line.len());
        line.truncate(end);
        line.push(0);
        let text = CStr::from_bytes_until_nul(&line).expect("terminated above");
        if let Some(matched) = prog.exec(text, 0, false) {
            // Remove the trailing CR, LF, spaces, etc.
            let mut l = text.to_bytes().len();
            while l > 0 && line[l - 1] <= b' ' {
                l -= 1;
            }
            let text = XString::from_bytes(&line[..l]);
            let col =
                |at: Option<usize>| c_int::try_from(at.unwrap_or(0)).unwrap_or(c_int::MAX) + 1;
            qf_add_entry(
                qfl,
                &NewEntry {
                    fname: Some(fname),
                    lnum,
                    col: col(matched.starts[0]),
                    end_col: col(matched.ends[0]),
                    // A help entry, which `qf_jump` opens as help.
                    kind: 1,
                    ..NewEntry::new(text.as_cstr())
                },
            );
        }
        lnum += 1;
        line_breakcheck();
    }
}

/// Search every help file in `dir`'s `doc/` directory, skipping the ones
/// written in another language than `lang`.
fn hgr_search_files_in_dir(qfl: Qfl, dir: &[u8], prog: &mut OwnedProg, lang: Option<&[u8]>) {
    // Find all "*.txt" and "*.??x" files in the "doc" directory.
    // Upstream builds this in `NameBuff` with `add_pathsep` and `strcat`,
    // which a 'runtimepath' entry close to MAXPATHL overruns; the pattern is
    // owned here instead.
    let mut pattern: Vec<u8> = dir.to_vec();
    if dir.last().is_some_and(|&c| !vim_ispathsep(c_int::from(c))) {
        pattern.push(u8::try_from(PATHSEP).unwrap_or(b'/'));
    }
    pattern.extend_from_slice(HELP_FILES);
    let pattern = XString::from_bytes(&pattern);

    let Some(fnames) =
        expand_wildcards_list(pattern.as_cstr(), ExpandFlags::FILE | ExpandFlags::SILENT)
    else {
        return;
    };
    for fname in &fnames {
        if got_int.get() {
            break;
        }
        if lang.is_none_or(|lang| wanted_language(lang, fname)) {
            hgr_search_file(qfl, fname.as_cstr(), prog);
        }
    }
}

/// Whether a help file is one `lang` asked for. The language is the two
/// characters before the extension's last one, so `foo.frx` is French —
/// except that `en` also claims every plain `.txt` file.
fn wanted_language(lang: &[u8], fname: &[u8]) -> bool {
    let ext = &fname[fname.len().saturating_sub(3)..];
    let same = |a: &[u8], b: &[u8], n: usize| {
        a.len() >= n && b.len() >= n && a[..n].eq_ignore_ascii_case(&b[..n])
    };
    same(lang, ext, 2) || (same(lang, b"en", 2) && same(b"txt", ext, 3))
}

/// Search the help files of every `'runtimepath'` entry.
fn hgr_search_in_rtp(qfl: Qfl, prog: &mut OwnedProg, lang: Option<&[u8]>) {
    // A copy: the walk is over the value as it was when the search began.
    let rtp = P_RTP.get();
    let mut rest: &[u8] = &rtp;
    let mut dir = Vec::new();
    while !rest.is_empty() && !got_int.get() {
        rest = next_option_part(rest, &mut dir);
        // Upstream copies each entry into a `MAXPATHL` buffer.
        dir.truncate(MAXPATHL as usize - 1);
        hgr_search_files_in_dir(qfl, &dir, prog, lang);
    }
}

/// The `@xx` language specifier at the end of a `:helpgrep` argument: the
/// pattern without it, and the two letters — upstream's `check_help_lang`,
/// minus the cut, which the caller makes.
fn split_help_lang(arg: &[u8]) -> (&[u8], Option<&[u8]>) {
    let len = arg.len();
    if len >= 3
        && arg[len - 3] == b'@'
        && arg[len - 2].is_ascii_alphabetic()
        && arg[len - 1].is_ascii_alphabetic()
    {
        return (&arg[..len - 3], Some(&arg[len - 2..]));
    }
    (arg, None)
}

/// `:helpgrep` and `:lhelpgrep`.
pub fn ex_helpgrep(excmd: &mut ExArg) {
    let mut qi = Qi::global();

    let au_name = match excmd.cmdidx {
        CmdIdx::helpgrep => Some(c"helpgrep"),
        CmdIdx::lhelpgrep => Some(c"lhelpgrep"),
        _ => None,
    };
    if let Some(name) = au_name {
        let claimed = fire_qf_autocmd(AutoEvent::QuickFixCmdPre, name, true);
        if claimed && aborting() {
            return;
        }
    }

    // Make 'cpoptions' empty, the 'l' flag should not be used here: a
    // plugin the search sources may set the option itself, which is what
    // the guard's restore is for.
    let cpo = SavedCpo::empty_under_user_code();

    let mut new_qi = false;
    if is_loclist_cmd(excmd.cmdidx) {
        qi = hgr_get_ll(&mut new_qi);
    }

    let busy = QuickfixBusy::hold();

    // Check for a specified language. Upstream cuts it off the command
    // line itself, which is why the list's title, made from the line below,
    // does not show it; the cut is made here too.
    let (pattern, lang) = split_help_lang(excmd.line.arg());
    let pattern = XString::from_bytes(pattern);
    let lang = lang.map(<[u8]>::to_vec);
    if lang.is_some() {
        let at = excmd.line.arg + pattern.len();
        excmd.line.buffer_mut()[at] = 0;
    }
    let prog = OwnedProg::compile(pattern.as_cstr(), RE_MAGIC + RE_STRING);
    let updated = prog.is_some();
    if let Some(mut prog) = prog {
        // Create a new quickfix list.
        let title = qf_cmdtitle(excmd.line.line());
        qf_new_list(qi, Some(&title));
        let mut qfl = qi.current_slot();

        hgr_search_in_rtp(qfl, &mut prog, lang.as_deref());
        drop(prog);

        qfl.no_valid = false;
        qfl.cursor = 0;
        qfl.index = 1;
        qfl.changed();
    }

    drop(cpo);

    if updated {
        // This may open a window and source scripts, so it waits until
        // 'cpo' has been restored.
        qf_update_buffer(qi, None);
    }

    if let Some(name) = au_name {
        fire_qf_autocmd(AutoEvent::QuickFixCmdPost, name, true);
        // When adding to an existing location list stack, an autocommand
        // may have made that stack invalid, in which case there is
        // nothing left to jump to.
        if !new_qi && qi.kind == QFLT_LOCATION && qf_find_win_with_loclist(qi.id()).is_none() {
            drop(busy);
            return;
        }
    }

    // Jump to the first match.
    if !qi.current_list().is_empty() {
        qf_jump(qi, 0, 0, false);
    } else {
        let arg = msg_bytes(excmd.line.arg());
        semsg!("E480: No match: {arg}");
    }

    drop(busy);

    if excmd.cmdidx == CmdIdx::lhelpgrep && new_qi {
        let mut win = Win::current();
        if !buf_is_help(win.buffer_or_none()) || win.w_llist == Some(qi.id()) {
            // The help window was not opened, or it already points at
            // the right location list: the new one is not wanted.
            drop_stack_ref(Some(qi.id()));
        } else if win.w_llist.is_none() {
            // The current window had no location list before, so it
            // takes the new one.
            win.w_llist = Some(qi.id());
        }
    }
}
