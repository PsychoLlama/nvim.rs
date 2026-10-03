//! `:vimgrep`, the built-in grep.
//!
//! [`ex_vimgrep`] loads each file named on the command line — into a real
//! buffer if one is already open, otherwise into a throwaway one
//! ([`load_dummy_buffer`], in the sibling `dummy` module) — and
//! [`match_buflines`] runs the pattern over its lines, recording every match
//! as an entry.
//!
//! Loading a file fires autocommands, and an autocommand can replace the
//! quickfix list, close windows or change directory. So the list is held as
//! a view under a [`QuickfixBusy`], re-checked by id after every file
//! ([`list_still_usable`]), and the directory is restored around every dummy
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
use crate::arglist::expand_file_args;
use crate::autocmd::AucmdBuf;
use crate::buffer::{BufFlags, find_buffer_by_name};
use crate::ex_cmds::split_vimgrep_pat;
use crate::ex_docmd::cmdmod_has;
use crate::file_search::Name;
use crate::memory::XString;
use crate::message::msg_strtrunc_text;
use crate::message_fmt::msg_bytes;
use crate::optionstr::OptString;
use crate::os::fs::current_dir;
use crate::path::try_shorten_fname;
use crate::regexp::{OwnedProg, RE_MAGIC};
use crate::search::last_pattern_owned;
use crate::semsg;
use crate::smsg;
use crate::types::CmdIdx;
use crate::types::{CmdLine, CmdModFlags, OptionSetFlags};
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_int, c_uint};
use std::time::{SystemTime, UNIX_EPOCH};

/// The autocommand name of a `:vimgrep`-family command. `:grep` is here
/// too, because `'grepprg'` set to `internal` sends it this way.
fn vgr_get_auname(cmdidx: CmdIdx) -> Option<&'static CStr> {
    Some(match cmdidx {
        CmdIdx::vimgrep => c"vimgrep",
        CmdIdx::lvimgrep => c"lvimgrep",
        CmdIdx::vimgrepadd => c"vimgrepadd",
        CmdIdx::lvimgrepadd => c"lvimgrepadd",
        CmdIdx::grep => c"grep",
        CmdIdx::lgrep => c"lgrep",
        CmdIdx::grepadd => c"grepadd",
        CmdIdx::lgrepadd => c"lgrepadd",
        _ => return None,
    })
}

/// The command line of one `:vimgrep`, parsed.
struct Search {
    /// The pattern as the user wrote it, for the fuzzy matcher and for the
    /// "no match" message.
    spat: XString,
    /// `VGR_GLOBAL`, `VGR_NOJUMP` and `VGR_FUZZY`.
    flags: c_int,
    /// How many more matches to record before stopping.
    tomatch: c_int,
    prog: OwnedProg,
    ignore_case: bool,
    /// The title the list gets, which outlives the command line.
    qf_title: Name,
}

impl Search {
    /// Parse `:vimgrep`'s arguments: the pattern, its flags, the match limit
    /// and the files to search. Reports the error itself.
    fn parse(excmd: &mut ExArg) -> Option<(Search, Vec<XString>)> {
        let title = qf_cmdtitle(excmd.line.line());
        let tomatch = if excmd.addr_count > 0 {
            excmd.line2 as c_int
        } else {
            MAXLNUM
        };

        let arg = excmd.line.arg();
        let Some((spat, flags, rest)) = split_vimgrep_pat(arg) else {
            qf_emsg(e_invalpat);
            return None;
        };
        let spat = XString::from_bytes(&spat);

        let prog = compile_pattern(&spat)?;

        let files = skip_white(&arg[rest..]);
        if files.is_empty() {
            qf_emsg(c"E683: File name missing or invalid pattern");
            return None;
        }
        let files = match expand_file_args(XString::from_bytes(files).as_cstr()) {
            Some(files) if !files.is_empty() => files,
            _ => {
                qf_emsg(e_nomatch);
                return None;
            }
        };
        let search = Search {
            spat,
            flags,
            tomatch,
            prog,
            ignore_case: p_ic(),
            qf_title: Name::from_bytes(title.to_bytes()),
        };
        Some((search, files))
    }
}

/// Compile the search pattern, falling back on the last search pattern when
/// `:vimgrep //` left it empty. Answers `None` after reporting the error.
fn compile_pattern(spat: &XString) -> Option<OwnedProg> {
    if !spat.is_empty() {
        return OwnedProg::compile(spat.as_cstr(), RE_MAGIC);
    }
    let Some(last) = last_pattern_owned() else {
        qf_emsg(e_noprevre);
        return None;
    };
    OwnedProg::compile(last.as_cstr(), RE_MAGIC)
}

/// Show which file is being searched, on the command line and without
/// waiting for a keypress.
fn display_fname(fname: &CStr) {
    msg_start();
    match msg_strtrunc_text(fname, 1) {
        None => msg_display(fname, 0, false),
        Some(truncated) => msg_display(truncated.as_cstr(), 0, false),
    };
    msg_clr_eos();
    msg_didout.set(false);
    msg_nowait.set(true);
    msg_col.set(0);
    ui_flush();
}

/// Load a file into a dummy buffer with `'modelines'` and the `FileType`
/// autocommand turned off, so that reading it stays cheap. `dirname_now` is
/// set to where the read left the editor.
fn load_quietly(
    fname: &CStr,
    dirname_start: &CStr,
    dirname_now: &mut Option<XString>,
) -> Option<Buf> {
    let save_ei = au_event_disable(c",Filetype");
    let save_mls = p_mls();
    P_MLS.set(0);
    let buf = load_dummy_buffer(fname, dirname_start, dirname_now);
    P_MLS.set(save_mls);
    au_event_restore(Some(save_ei));
    buf
}

/// Whether the list with id `qfid` can still be added to after an
/// autocommand ran. A quickfix list that went away is replaced by a fresh
/// one; a location list that went away ends the command.
fn list_still_usable(window: Option<Win>, qi: Qi, qfid: c_uint, title: &CStr) -> bool {
    if !qflist_valid(window, qfid) {
        if window.is_some() {
            qf_emsg(E_LOCATION_LIST_CHANGED);
            return false;
        }
        qf_new_list(qi, Some(title));
        return true;
    }
    qf_restore_list(qi, qfid).is_ok()
}

/// Search one buffer's lines and add an entry for every match. Answers
/// whether anything matched at all.
///
/// `duplicate_name` says the file is open in a buffer that has no memfile,
/// in which case the entry names the file rather than that buffer.
fn match_buflines(
    qfl: Qfl,
    fname: &CStr,
    buffer: Buf,
    search: &mut Search,
    duplicate_name: bool,
) -> bool {
    let bufnum = if duplicate_name { 0 } else { buffer.handle };
    let global = search.flags & VGR_GLOBAL.cast_signed() != 0;
    let mut found_match = false;

    let mut lnum: LineNr = 1;
    while lnum <= buffer.b_ml.ml_line_count && search.tomatch > 0 {
        if search.flags & VGR_FUZZY.cast_signed() == 0 {
            let mut col: ColNr = 0;
            let mut regmatch = RegMMatch {
                rmm_ic: c_int::from(search.ignore_case),
                rmm_maxcol: 0,
                ..RegMMatch::default()
            };
            while search
                .prog
                .exec_multi(&mut regmatch, Some(Win::current()), buffer, lnum, col)
                > 0
            {
                let start = regmatch.startpos[0];
                let end = regmatch.endpos[0];
                // A copy: adding the entry lists the file, which runs
                // autocommands, and those can change the buffer.
                let text = XString::from_cstr(buffer.lines().line_cstr(start.lnum + lnum, 0));
                qf_add_entry(
                    qfl,
                    &NewEntry {
                        fname: Some(fname),
                        bufnum,
                        lnum: start.lnum + lnum,
                        end_lnum: end.lnum + lnum,
                        col: start.col + 1,
                        end_col: end.col + 1,
                        ..NewEntry::new(text.as_cstr())
                    },
                );
                found_match = true;

                search.tomatch -= 1;
                if search.tomatch == 0 {
                    break;
                }
                // Without `g` only the first match of a line counts, and
                // a match that ran into the next line has consumed this
                // one either way.
                if !global || end.lnum > 0 {
                    break;
                }
                // Move past the match, and past one more column when the
                // match was empty, so that the scan makes progress.
                col = end.col + ColNr::from(col == end.col);
                if col > ml_get_buf_len(buffer, lnum) {
                    break;
                }
            }
        } else {
            let line = XString::from_cstr(buffer.lines().line_cstr(lnum, 0));
            let linelen = ml_get_buf_len(buffer, lnum);
            // The pattern length is in bytes while the matcher fills one
            // position per *character*, so for a multibyte pattern the
            // position read below is one the matcher never wrote. It has
            // to be zero, which is why the array is cleared every line.
            let pat_len = search.spat.len().min(FUZZY_MATCH_MAX_LEN);
            let mut col: ColNr = 0;
            // Cleared once per line, not once per match: a second match
            // on the same line reads whatever the first one left in the
            // positions past its own length, which is what upstream does.
            let mut positions = [0u32; FUZZY_MATCH_MAX_LEN];
            loop {
                let from = usize::try_from(col).expect("a column is never negative");
                let rest = CStr::from_bytes_until_nul(&line.as_cstr().to_bytes_with_nul()[from..])
                    .expect("the line is terminated");
                let (_, filled) = fuzzy_match(rest, search.spat.as_cstr(), false, &mut positions);
                if filled == 0 {
                    break;
                }

                let first = c_int::try_from(positions[0]).unwrap_or(c_int::MAX);
                qf_add_entry(
                    qfl,
                    &NewEntry {
                        fname: Some(fname),
                        bufnum,
                        lnum,
                        col: first + col + 1,
                        ..NewEntry::new(line.as_cstr())
                    },
                );
                found_match = true;

                search.tomatch -= 1;
                if search.tomatch == 0 || !global {
                    break;
                }
                // `pat_len` is at least 1 here: an empty pattern fills
                // no position and so never passes the test above.
                col = ColNr::try_from(positions[pat_len - 1]).unwrap_or(ColNr::MAX) + col + 1;
                if col > linelen {
                    break;
                }
            }
        }

        line_breakcheck();
        if got_int.get() {
            break;
        }
        lnum += 1;
    }

    found_match
}

/// What searching the files left behind for [`ex_vimgrep`]'s tail.
#[derive(Default)]
struct Outcome {
    /// A dummy buffer was loaded, so folds have to be rebuilt afterwards.
    redraw_for_dummy: bool,
    /// The buffer holding the first match, which is kept loaded so that the
    /// jump lands in it.
    first_match_buf: Option<Buf>,
    /// Where an autocommand left the directory, if the first match's buffer
    /// is to be entered with it.
    target_dir: Option<XString>,
}

/// Whether the swap file the buffer has is one that already existed, i.e.
/// not the `.swp` this load made — in which case the dummy buffer is
/// unloaded rather than kept, so that the swap file is not left behind.
fn existing_swapfile(buffer: Buf) -> bool {
    buffer
        .swap_file_name()
        .is_some_and(|name| !name.to_bytes().ends_with(b"wp"))
}

/// The time now, in seconds, as `time(NULL)` counts it.
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

/// Search every file named on the command line. Answers false when an
/// autocommand made the list unusable, in which case the caller stops.
fn process_files(
    window: Option<Win>,
    qi: Qi,
    search: &mut Search,
    files: &[XString],
    out: &mut Outcome,
) -> bool {
    let mut save_qfid = qi.current_list().id;
    let dirname_start = current_dir().unwrap_or_default();
    let mut dirname_now = None;

    // Upstream never resets this in the "buffer already loaded" arm, so
    // a file that is open keeps whatever the last dummy load decided.
    let mut duplicate_name = false;
    let mut seconds = 0;
    for file in files {
        if got_int.get() || search.tomatch <= 0 {
            break;
        }
        let fname = XString::from_cstr(try_shorten_fname(file.as_cstr()));
        // Print the file name every second, so that a slow search shows
        // progress without flooding the message area.
        if now() > seconds {
            seconds = now();
            display_fname(fname.as_cstr());
        }

        // Load the file into a buffer, unless it is already loaded.
        let mut buf = find_buffer_by_name(file.as_cstr());
        let using_dummy = buf.is_none_or(|buf| buf.b_ml.ml_mfp.is_null());
        if using_dummy {
            duplicate_name = buf.is_some();
            out.redraw_for_dummy = true;
            buf = load_quietly(fname.as_cstr(), dirname_start.as_cstr(), &mut dirname_now);
        }

        // Autocommands may have changed the list under us.
        let title = XString::from_bytes(search.qf_title.bytes());
        if !list_still_usable(window, qi, save_qfid, title.as_cstr()) {
            return false;
        }
        save_qfid = qi.current_list().id;

        if let Some(buf) = buf {
            let found_match = match_buflines(
                qi.current_slot(),
                fname.as_cstr(),
                buf,
                search,
                duplicate_name,
            );
            if using_dummy {
                keep_or_drop_dummy(
                    buf,
                    found_match,
                    duplicate_name,
                    search,
                    dirname_start.as_cstr(),
                    dirname_now.as_ref(),
                    out,
                );
            }
        } else if !got_int.get() {
            let fname = msg_bytes(&fname);
            smsg!(0, "Cannot open file \"{fname}\"");
        }
    }
    true
}

/// Decide what becomes of the dummy buffer a file was loaded into: wipe it,
/// unload it, or keep it because it holds the first match and the jump will
/// land there.
fn keep_or_drop_dummy(
    mut buffer: Buf,
    found_match: bool,
    duplicate_name: bool,
    search: &Search,
    dirname_start: &CStr,
    dirname_now: Option<&XString>,
    out: &mut Outcome,
) {
    if found_match && out.first_match_buf.is_none() {
        out.first_match_buf = Some(buffer);
    }

    // Never keep a dummy buffer when another buffer has the same name.
    if duplicate_name {
        wipe_dummy_buffer(buffer, Some(dirname_start));
        return;
    }

    // `:hide` keeps the buffer loaded — unless 'bufhidden' says the
    // buffer goes away as soon as it is hidden, which wins.
    let bufhidden = buffer.b_p_bh.first_byte();
    let hidden_stays = cmdmod_has(CmdModFlags::HIDE) && !matches!(bufhidden, b'u' | b'w' | b'd');
    if !hidden_stays {
        if !found_match {
            // Do not keep a buffer that was not loaded before.
            wipe_dummy_buffer(buffer, Some(dirname_start));
            return;
        }
        if out.first_match_buf != Some(buffer)
            || search.flags & VGR_NOJUMP.cast_signed() != 0
            || existing_swapfile(buffer)
        {
            unload_dummy_buffer(buffer, dirname_start);
            // Keeping the buffer, remove the dummy flag.
            buffer.b_flags.clear(BufFlags::DUMMY);
            return;
        }
    }

    // Keeping the buffer, remove the dummy flag.
    buffer.b_flags.clear(BufFlags::DUMMY);

    // The buffer is still loaded, so the jump below has to go to the
    // directory the search left it in.
    if out.first_match_buf == Some(buffer)
        && out.target_dir.is_none()
        && let Some(now) = dirname_now.filter(|now| now.as_cstr() != dirname_start)
    {
        out.target_dir = Some(now.clone());
    }

    // The Filetype autocommands and the modelines need to run now, in
    // that buffer — but not the window-local options.
    let aco = AucmdBuf::enter(buffer);
    let filetype = buffer.b_p_ft.get();
    let name = buffer.name.shown().map(CStr::to_owned);
    fire_autocmds_for(
        AutoEvent::FileType,
        Some(filetype.as_cstr()),
        name.as_deref(),
        true,
        Some(buffer),
    );
    do_modelines(OptionSetFlags::NOWIN);
    drop(aco);
}

/// Jump to the first match, and change to the directory the first match's
/// file was found in when the search left the editor somewhere else.
fn jump_to_match(qi: Qi, forceit: bool, out: &mut Outcome) {
    let buf = Buf::current_or_none();
    qf_jump(qi, 0, 0, forceit);
    if Buf::current_or_none() != buf {
        out.redraw_for_dummy = false;
    }

    // The buffer of the first match is the one the search left the
    // directory in; put the window back there.
    if let Some(target_dir) = &out.target_dir
        && out.first_match_buf.is_some()
        && out.first_match_buf == Buf::current_or_none()
    {
        let mut ea = ExArg {
            line: CmdLine::from_bytes(target_dir),
            cmdidx: CmdIdx::lcd,
            ..Default::default()
        };
        ex_cd(&mut ea);
    }
}

/// `:vimgrep`, `:lvimgrep`, `:vimgrepadd`, `:lvimgrepadd`, and `:grep` and
/// friends when `'grepprg'` is `internal`.
pub fn ex_vimgrep(excmd: &mut ExArg) {
    if !check_can_set_curbuf_forceit(c_int::from(excmd.forceit)) {
        return;
    }

    let au_name = vgr_get_auname(excmd.cmdidx);
    if let Some(name) = au_name {
        let claimed = fire_qf_autocmd(AutoEvent::QuickFixCmdPre, name, true);
        if claimed && aborting() {
            return;
        }
    }

    let (qi, wp) = stack_or_new_for_cmd(excmd);

    let parsed = Search::parse(excmd);
    let Some((mut search, files)) = parsed else {
        return;
    };

    let adding = matches!(
        excmd.cmdidx,
        CmdIdx::grepadd | CmdIdx::lgrepadd | CmdIdx::vimgrepadd | CmdIdx::lvimgrepadd
    );
    if !adding || qi.is_empty() {
        // Make a new list.
        let title = XString::from_bytes(search.qf_title.bytes());
        qf_new_list(qi, Some(title.as_cstr()));
    }

    let busy = QuickfixBusy::hold();
    let mut out = Outcome::default();
    let searched = process_files(wp, qi, &mut search, &files, &mut out);
    drop(files);
    if !searched {
        drop(busy);
        return;
    }

    let mut qfl = qi.current_slot();
    qfl.no_valid = false;
    qfl.cursor = 0;
    qfl.index = 1;
    qfl.changed();

    qf_update_buffer(qi, None);

    // Remember the current list, so that an autocommand replacing it is
    // noticed before the jump.
    let save_qfid = qi.current_list().id;
    if let Some(name) = au_name {
        fire_qf_autocmd(AutoEvent::QuickFixCmdPost, name, true);
    }
    if !qflist_valid(wp, save_qfid) || qf_restore_list(qi, save_qfid).is_err() {
        drop(busy);
        return;
    }

    if qi.current_list().is_empty() {
        let spat = msg_bytes(&search.spat);
        semsg!("E480: No match: {spat}");
    } else if search.flags & VGR_NOJUMP.cast_signed() == 0 {
        jump_to_match(qi, excmd.forceit, &mut out);
    }

    drop(busy);

    // Reading the files may have messed up the folds of the window the
    // command was given in.
    if out.redraw_for_dummy {
        fold_update_all(Win::current());
    }
}
