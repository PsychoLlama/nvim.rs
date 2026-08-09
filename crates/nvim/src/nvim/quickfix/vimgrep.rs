//! `:vimgrep`, the built-in grep.
//!
//! [`ex_vimgrep`] loads each file named on the command line — into a real
//! buffer if one is already open, otherwise into a throwaway one
//! ([`load_dummy_buffer`], in the sibling `dummy` module) — and
//! [`match_buflines`] runs the pattern over its lines, recording every match
//! as an entry.
//!
//! Loading a file fires autocommands, and an autocommand can replace the
//! quickfix list, close windows or change directory. So the buffer, window
//! and list pointers stay raw here, the list is re-checked by id after every
//! file ([`list_still_usable`]), and the directory is restored around every
//! dummy buffer.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::src::nvim::file_search::Name;
use crate::src::nvim::regexp::RE_MAGIC;
use crate::src::nvim::types::{
    CMD_grep, CMD_grepadd, CMD_lcd, CMD_lgrep, CMD_lgrepadd, CMD_lvimgrep, CMD_lvimgrepadd,
    CMD_vimgrep, CMD_vimgrepadd, CMOD_HIDE,
};
#[allow(unused_imports)]
use crate::{semsg_c, smsg_c};
use core::ffi::{CStr, c_char, c_int, c_uint};
use core::ptr;

/// The autocommand name of a `:vimgrep`-family command. `:grep` is here
/// too, because `'grepprg'` set to `internal` sends it this way.
fn vgr_get_auname(cmdidx: cmdidx_T) -> Option<&'static CStr> {
    Some(match cmdidx {
        CMD_vimgrep => c"vimgrep",
        CMD_lvimgrep => c"lvimgrep",
        CMD_vimgrepadd => c"vimgrepadd",
        CMD_lvimgrepadd => c"lvimgrepadd",
        CMD_grep => c"grep",
        CMD_lgrep => c"lgrep",
        CMD_grepadd => c"grepadd",
        CMD_lgrepadd => c"lgrepadd",
        _ => return None,
    })
}

/// The files named on the command line, as `get_arglist_exp` expanded them.
/// Owns the array, which `FreeWild` is the only way to give back.
struct Files {
    names: *mut *mut c_char,
    count: c_int,
}

impl Files {
    /// Expand the file names in `arg`, reporting E480 when the pattern
    /// matches nothing.
    ///
    /// # Safety
    ///
    /// `arg` must be NUL-terminated.
    unsafe fn expand(arg: *mut c_char) -> Option<Files> {
        let mut files = Files {
            names: ptr::null_mut(),
            count: 0,
        };
        // SAFETY: the caller's string, and two writable out-parameters.
        unsafe {
            let ok = get_arglist_exp(arg, &raw mut files.count, &raw mut files.names, true) == OK;
            if !ok || files.count == 0 {
                emsg(gettext(&raw const e_nomatch as *const c_char));
                return None;
            }
        }
        Some(files)
    }

    /// # Safety
    ///
    /// `at` must be below `count`.
    unsafe fn get(&self, at: c_int) -> *mut c_char {
        // SAFETY: the caller's promise.
        unsafe { *self.names.offset(at as isize) }
    }
}

impl Drop for Files {
    fn drop(&mut self) {
        // SAFETY: the array and its entries are ours, and nothing else holds
        // them.
        unsafe { FreeWild(self.count, self.names) };
    }
}

/// The command line of one `:vimgrep`, parsed.
///
/// Owns the compiled pattern and the list title; `spat` points into the
/// command line itself, which `skip_vimgrep_pat` terminated in place.
struct Search {
    /// The pattern as the user wrote it, for the fuzzy matcher and for the
    /// "no match" message.
    spat: *mut c_char,
    /// `VGR_GLOBAL`, `VGR_NOJUMP` and `VGR_FUZZY`.
    flags: c_int,
    /// How many more matches to record before stopping.
    tomatch: c_int,
    regmatch: regmmatch_T,
    /// The title the list gets, which outlives the command line.
    qf_title: Name,
}

impl Drop for Search {
    fn drop(&mut self) {
        // SAFETY: the program is ours and nothing else holds it.
        unsafe { vim_regfree(self.regmatch.regprog) };
    }
}

impl Search {
    /// Parse `:vimgrep`'s arguments: the pattern, its flags, the match limit
    /// and the files to search. Reports the error itself.
    ///
    /// # Safety
    ///
    /// `eap` must be a live command.
    unsafe fn parse(eap: *mut exarg_T) -> Option<(Search, Files)> {
        // SAFETY: forwarded from the caller.
        unsafe {
            let mut search = Search {
                spat: ptr::null_mut(),
                flags: 0,
                tomatch: if (*eap).addr_count > 0 {
                    (*eap).line2 as c_int
                } else {
                    MAXLNUM as c_int
                },
                regmatch: regmmatch_T::default(),
                qf_title: Name::from_ptr(qf_cmdtitle(*(*eap).cmdlinep)),
            };

            let p = skip_vimgrep_pat((*eap).arg, &raw mut search.spat, &raw mut search.flags);
            if p.is_null() {
                emsg(gettext(&raw const e_invalpat as *const c_char));
                return None;
            }

            search.regmatch.regprog = compile_pattern(search.spat);
            if search.regmatch.regprog.is_null() {
                return None;
            }
            search.regmatch.rmm_ic = p_ic.get();
            search.regmatch.rmm_maxcol = 0;

            let p = skipwhite(p);
            if *p as c_int == NUL {
                emsg(gettext(
                    c"E683: File name missing or invalid pattern".as_ptr(),
                ));
                return None;
            }

            let files = Files::expand(p)?;
            Some((search, files))
        }
    }
}

/// Compile the search pattern, falling back on the last search pattern when
/// `:vimgrep //` left it empty. Answers null after reporting the error.
///
/// # Safety
///
/// `spat` must be null or NUL-terminated.
unsafe fn compile_pattern(spat: *mut c_char) -> *mut regprog_T {
    // SAFETY: forwarded from the caller.
    unsafe {
        if !spat.is_null() && *spat as c_int != NUL {
            return vim_regcomp(spat, RE_MAGIC);
        }
        if last_search_pat().is_null() {
            emsg(gettext(&raw const e_noprevre as *const c_char));
            return ptr::null_mut();
        }
        vim_regcomp(last_search_pat(), RE_MAGIC)
    }
}

/// Show which file is being searched, on the command line and without
/// waiting for a keypress.
///
/// # Safety
///
/// `fname` must be NUL-terminated.
unsafe fn display_fname(fname: *mut c_char) {
    // SAFETY: forwarded from the caller.
    unsafe {
        msg_start();
        let truncated = msg_strtrunc(fname, 1);
        if truncated.is_null() {
            msg_outtrans(fname, 0, false);
        } else {
            msg_outtrans(truncated, 0, false);
            xfree(truncated.cast());
        }
        msg_clr_eos();
        msg_didout.set(false);
        msg_nowait.set(true);
        msg_col.set(0);
        ui_flush();
    }
}

/// Load a file into a dummy buffer with `'modelines'` and the `FileType`
/// autocommand turned off, so that reading it stays cheap.
///
/// # Safety
///
/// The strings must be NUL-terminated and `dirname_now` have room for
/// MAXPATHL bytes.
unsafe fn load_quietly(
    fname: *mut c_char,
    dirname_start: *const c_char,
    dirname_now: *mut c_char,
) -> *mut buf_T {
    // SAFETY: forwarded from the caller.
    unsafe {
        let save_ei = au_event_disable(c",Filetype".as_ptr().cast_mut());
        let save_mls = p_mls.get();
        p_mls.set(0);
        let buf = load_dummy_buffer(fname, dirname_start, dirname_now);
        p_mls.set(save_mls);
        au_event_restore(save_ei);
        buf
    }
}

/// Whether the list with id `qfid` can still be added to after an
/// autocommand ran. A quickfix list that went away is replaced by a fresh
/// one; a location list that went away ends the command.
///
/// # Safety
///
/// `qi` must be a live stack and `title` NUL-terminated.
unsafe fn list_still_usable(
    wp: *mut win_T,
    qi: *mut qf_info_T,
    qfid: c_uint,
    title: *const c_char,
) -> bool {
    // SAFETY: forwarded from the caller.
    unsafe {
        if !qflist_valid(wp, qfid) {
            if !wp.is_null() {
                emsg(gettext(E_LOCATION_LIST_CHANGED.as_ptr()));
                return false;
            }
            qf_new_list(qi, title);
            return true;
        }
        qf_restore_list(qi, qfid) != FAIL
    }
}

/// Search one buffer's lines and add an entry for every match. Answers
/// whether anything matched at all.
///
/// `duplicate_name` says the file is open in a buffer that has no memfile,
/// in which case the entry names the file rather than that buffer.
///
/// # Safety
///
/// `qfl` must be a live list, `buf` a loaded buffer and `fname`
/// NUL-terminated.
unsafe fn match_buflines(
    qfl: *mut qf_list_T,
    fname: *mut c_char,
    buf: *mut buf_T,
    search: &mut Search,
    duplicate_name: bool,
) -> bool {
    // SAFETY: forwarded from the caller.
    unsafe {
        let bufnum = if duplicate_name {
            0
        } else {
            (*buf).handle as c_int
        };
        let global = search.flags & VGR_GLOBAL as c_int != 0;
        let mut found_match = false;

        let mut lnum: linenr_T = 1;
        while lnum <= (*buf).b_ml.ml_line_count && search.tomatch > 0 {
            if search.flags & VGR_FUZZY as c_int == 0 {
                let mut col: colnr_T = 0;
                while vim_regexec_multi(
                    &raw mut search.regmatch,
                    curwin.get(),
                    buf,
                    lnum,
                    col,
                    ptr::null_mut(),
                    ptr::null_mut(),
                ) > 0
                {
                    let start = search.regmatch.startpos[0];
                    let end = search.regmatch.endpos[0];
                    qf_add_entry(
                        qfl,
                        &NewEntry {
                            fname,
                            bufnum,
                            lnum: start.lnum + lnum,
                            end_lnum: end.lnum + lnum,
                            col: start.col as c_int + 1,
                            end_col: end.col as c_int + 1,
                            ..NewEntry::new(ml_get_buf(buf, start.lnum + lnum))
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
                    col = end.col + colnr_T::from(col == end.col);
                    if col > ml_get_buf_len(buf, lnum) {
                        break;
                    }
                }
            } else {
                let line = ml_get_buf(buf, lnum);
                let linelen = ml_get_buf_len(buf, lnum);
                // The pattern length is in bytes while the matcher fills one
                // position per *character*, so for a multibyte pattern the
                // position read below is one the matcher never wrote. It has
                // to be zero, which is why the array is cleared every line.
                let pat_len = strlen(search.spat).min(FUZZY_MATCH_MAX_LEN as size_t);
                let mut col: colnr_T = 0;
                // Cleared once per line, not once per match: a second match
                // on the same line reads whatever the first one left in the
                // positions past its own length, which is what upstream does.
                let mut positions = [0u32; FUZZY_MATCH_MAX_LEN as usize];
                loop {
                    let mut score: c_int = 0;
                    let matched = fuzzy_match(
                        line.offset(col as isize),
                        search.spat,
                        false,
                        &raw mut score,
                        positions.as_mut_ptr(),
                        positions.len() as c_int,
                    );
                    if !matched {
                        break;
                    }

                    qf_add_entry(
                        qfl,
                        &NewEntry {
                            fname,
                            bufnum,
                            lnum,
                            col: positions[0] as c_int + col as c_int + 1,
                            ..NewEntry::new(line)
                        },
                    );
                    found_match = true;

                    search.tomatch -= 1;
                    if search.tomatch == 0 || !global {
                        break;
                    }
                    // `pat_len` is at least 1 here: an empty pattern fills
                    // no position and so never passes the test above.
                    col = positions[pat_len as usize - 1] as colnr_T + col + 1;
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
}

/// What searching the files left behind for [`ex_vimgrep`]'s tail.
struct Outcome {
    /// A dummy buffer was loaded, so folds have to be rebuilt afterwards.
    redraw_for_dummy: bool,
    /// The buffer holding the first match, which is kept loaded so that the
    /// jump lands in it.
    first_match_buf: *mut buf_T,
    /// Where an autocommand left the directory, if the first match's buffer
    /// is to be entered with it.
    target_dir: Option<Name>,
}

impl Default for Outcome {
    fn default() -> Self {
        Outcome {
            redraw_for_dummy: false,
            first_match_buf: ptr::null_mut(),
            target_dir: None,
        }
    }
}

/// Whether the swap file the buffer has is one that already existed, i.e.
/// not the `.swp` this load made — in which case the dummy buffer is
/// unloaded rather than kept, so that the swap file is not left behind.
///
/// # Safety
///
/// `buf` must be a live buffer.
unsafe fn existing_swapfile(buf: *const buf_T) -> bool {
    // SAFETY: forwarded from the caller.
    unsafe {
        if (*buf).b_ml.ml_mfp.is_null() {
            return false;
        }
        let fname = mf_fname((*buf).b_ml.ml_mfp);
        if fname.is_null() {
            return false;
        }
        !CStr::from_ptr(fname).to_bytes().ends_with(b"wp")
    }
}

/// Search every file named on the command line. Answers false when an
/// autocommand made the list unusable, in which case the caller stops.
///
/// # Safety
///
/// `qi` must be a live stack and `wp` null or a live window.
unsafe fn process_files(
    wp: *mut win_T,
    qi: *mut qf_info_T,
    search: &mut Search,
    files: &Files,
    out: &mut Outcome,
) -> bool {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut save_qfid = (*qf_get_curlist(qi)).qf_id;
        let mut dirname_start = vec![0 as c_char; MAXPATHL as usize];
        let mut dirname_now = vec![0 as c_char; MAXPATHL as usize];
        os_dirname(dirname_start.as_mut_ptr(), MAXPATHL as size_t);
        let start = dirname_start.as_ptr();

        // Upstream never resets this in the "buffer already loaded" arm, so
        // a file that is open keeps whatever the last dummy load decided.
        let mut duplicate_name = false;
        let mut seconds: time_t = 0;
        let mut fi = 0;
        while fi < files.count && !got_int.get() && search.tomatch > 0 {
            let fname = path_try_shorten_fname(files.get(fi));
            // Print the file name every second, so that a slow search shows
            // progress without flooding the message area.
            if time(ptr::null_mut()) > seconds {
                seconds = time(ptr::null_mut());
                display_fname(fname);
            }

            // Load the file into a buffer, unless it is already loaded.
            let mut buf = buflist_findname_exp(files.get(fi));
            let using_dummy = buf.is_null() || (*buf).b_ml.ml_mfp.is_null();
            if using_dummy {
                duplicate_name = !buf.is_null();
                out.redraw_for_dummy = true;
                buf = load_quietly(fname, start, dirname_now.as_mut_ptr());
            }

            // Autocommands may have changed the list under us.
            if !list_still_usable(wp, qi, save_qfid, search.qf_title.as_ptr()) {
                return false;
            }
            save_qfid = (*qf_get_curlist(qi)).qf_id;

            if buf.is_null() {
                if !got_int.get() {
                    smsg_c!(0, gettext(c"Cannot open file \"%s\"".as_ptr()), fname);
                }
            } else {
                let found_match =
                    match_buflines(qf_get_curlist(qi), fname, buf, search, duplicate_name);
                if using_dummy {
                    keep_or_drop_dummy(
                        buf,
                        found_match,
                        duplicate_name,
                        search,
                        start,
                        dirname_now.as_ptr(),
                        out,
                    );
                }
            }
            fi += 1;
        }
        true
    }
}

/// Decide what becomes of the dummy buffer a file was loaded into: wipe it,
/// unload it, or keep it because it holds the first match and the jump will
/// land there.
///
/// # Safety
///
/// `buf` must be the dummy buffer just searched, and the two directory names
/// NUL-terminated.
unsafe fn keep_or_drop_dummy(
    buf: *mut buf_T,
    found_match: bool,
    duplicate_name: bool,
    search: &Search,
    dirname_start: *const c_char,
    dirname_now: *const c_char,
    out: &mut Outcome,
) {
    // SAFETY: forwarded from the caller.
    unsafe {
        if found_match && out.first_match_buf.is_null() {
            out.first_match_buf = buf;
        }

        // Never keep a dummy buffer when another buffer has the same name.
        if duplicate_name {
            wipe_dummy_buffer(buf, dirname_start);
            return;
        }

        // `:hide` keeps the buffer loaded — unless 'bufhidden' says the
        // buffer goes away as soon as it is hidden, which wins.
        let bufhidden = *(*buf).b_p_bh as u8;
        let hidden_stays = (*cmdmod.ptr()).cmod_flags & CMOD_HIDE as c_int != 0
            && !matches!(bufhidden, b'u' | b'w' | b'd');
        if !hidden_stays {
            if !found_match {
                // Do not keep a buffer that was not loaded before.
                wipe_dummy_buffer(buf, dirname_start);
                return;
            }
            if !ptr::eq(buf, out.first_match_buf)
                || search.flags & VGR_NOJUMP as c_int != 0
                || existing_swapfile(buf)
            {
                unload_dummy_buffer(buf, dirname_start);
                // Keeping the buffer, remove the dummy flag.
                (*buf).b_flags &= !BF_DUMMY;
                return;
            }
        }

        // Keeping the buffer, remove the dummy flag.
        (*buf).b_flags &= !BF_DUMMY;

        // The buffer is still loaded, so the jump below has to go to the
        // directory the search left it in.
        if ptr::eq(buf, out.first_match_buf)
            && out.target_dir.is_none()
            && strcmp(dirname_start, dirname_now) != 0
        {
            out.target_dir = Some(Name::from_ptr(dirname_now));
        }

        // The Filetype autocommands and the modelines need to run now, in
        // that buffer — but not the window-local options.
        let mut aco = aco_save_T::default();
        aucmd_prepbuf(&raw mut aco, buf);
        apply_autocmds(EVENT_FILETYPE, (*buf).b_p_ft, (*buf).b_fname, true, buf);
        do_modelines(OPT_NOWIN as c_int);
        aucmd_restbuf(&raw mut aco);
    }
}

/// Jump to the first match, and change to the directory the first match's
/// file was found in when the search left the editor somewhere else.
///
/// # Safety
///
/// `qi` must be a live stack.
unsafe fn jump_to_match(qi: *mut qf_info_T, forceit: c_int, out: &mut Outcome) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let buf = curbuf.get();
        qf_jump(qi, 0, 0, forceit);
        if !ptr::eq(buf, curbuf.get()) {
            out.redraw_for_dummy = false;
        }

        // The buffer of the first match is the one the search left the
        // directory in; put the window back there.
        if let Some(target_dir) = &out.target_dir
            && ptr::eq(curbuf.get(), out.first_match_buf)
        {
            let mut ea = exarg_T {
                arg: target_dir.as_ptr().cast_mut(),
                cmdidx: CMD_lcd,
                ..Default::default()
            };
            ex_cd(&raw mut ea);
        }
    }
}

/// `:vimgrep`, `:lvimgrep`, `:vimgrepadd`, `:lvimgrepadd`, and `:grep` and
/// friends when `'grepprg'` is `internal`.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn ex_vimgrep(eap: *mut exarg_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        if !check_can_set_curbuf_forceit((*eap).forceit) {
            return;
        }

        let au_name = vgr_get_auname((*eap).cmdidx);
        if let Some(name) = au_name {
            let claimed = apply_autocmds(
                EVENT_QUICKFIXCMDPRE,
                name.as_ptr().cast_mut(),
                (*curbuf.get()).b_fname,
                true,
                curbuf.get(),
            );
            if claimed && aborting() {
                return;
            }
        }

        let mut wp: *mut win_T = ptr::null_mut();
        let qi = qf_cmd_get_or_alloc_stack(eap, &raw mut wp);

        let Some((mut search, files)) = Search::parse(eap) else {
            return;
        };

        let adding = matches!(
            (*eap).cmdidx,
            CMD_grepadd | CMD_lgrepadd | CMD_vimgrepadd | CMD_lvimgrepadd
        );
        if !adding || qf_stack_empty(qi) {
            // Make a new list.
            qf_new_list(qi, search.qf_title.as_ptr());
        }

        incr_quickfix_busy();
        let mut out = Outcome::default();
        let searched = process_files(wp, qi, &mut search, &files, &mut out);
        drop(files);
        if !searched {
            decr_quickfix_busy();
            return;
        }

        let qfl = qf_get_curlist(qi);
        (*qfl).qf_nonevalid = false;
        (*qfl).qf_ptr = (*qfl).qf_start;
        (*qfl).qf_index = 1;
        qf_list_changed(qfl);

        qf_update_buffer(qi, ptr::null_mut());

        // Remember the current list, so that an autocommand replacing it is
        // noticed before the jump.
        let save_qfid = (*qf_get_curlist(qi)).qf_id;
        if let Some(name) = au_name {
            apply_autocmds(
                EVENT_QUICKFIXCMDPOST,
                name.as_ptr().cast_mut(),
                (*curbuf.get()).b_fname,
                true,
                curbuf.get(),
            );
        }
        if !qflist_valid(wp, save_qfid) || qf_restore_list(qi, save_qfid) == FAIL {
            decr_quickfix_busy();
            return;
        }

        if qf_list_empty(qf_get_curlist(qi)) {
            semsg_c!(gettext(&raw const e_nomatch2 as *const c_char), search.spat);
        } else if search.flags & VGR_NOJUMP as c_int == 0 {
            jump_to_match(qi, (*eap).forceit, &mut out);
        }

        decr_quickfix_busy();

        // Reading the files may have messed up the folds of the window the
        // command was given in.
        if out.redraw_for_dummy {
            foldUpdateAll(curwin.get());
        }
    }
}
