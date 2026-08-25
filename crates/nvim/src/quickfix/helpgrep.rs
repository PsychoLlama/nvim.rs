//! `:helpgrep`, which searches the help files.
//!
//! [`ex_helpgrep`] walks every `doc/` directory in `'runtimepath'`
//! ([`hgr_search_in_rtp`]) and matches the pattern against each help file's
//! lines ([`hgr_search_file`]), building a list without ever loading a
//! buffer.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::optionstr::{empty_option, is_empty_option};
use crate::path::ExpandFlags;
use crate::regexp::{RE_MAGIC, RE_STRING};
use crate::semsg_c;
use crate::types::{CMD_helpgrep, CMD_lhelpgrep, IOSIZE, MAXPATHL, NUL, OK, OptionSetFlags};
use core::ffi::{c_char, c_int};
use core::ptr;

/// The wildcard `:helpgrep` expands in each `'runtimepath'` entry. It is a
/// `\(…\)` alternation because `gen_expand_wildcards` matches it as a
/// regular expression once the shell-style parts are translated.
const HELP_FILES: &[u8] = br"doc/*.\(txt\|??x\)";

/// The location list `:lhelpgrep` adds to: the one of the help window, if
/// there is one, and otherwise a fresh stack — which the caller is told
/// about through `new_ll`, because it has to free it again if nothing ends
/// up pointing at it.
///
/// # Safety
///
/// There must be a current window.
unsafe fn hgr_get_ll(new_ll: &mut bool) -> *mut qf_info_T {
    // SAFETY: the caller's promise.
    unsafe {
        let wp = if bt_help((*curwin.get()).w_buffer) {
            curwin.get()
        } else {
            qf_find_help_win()
        };
        let qi = if wp.is_null() {
            ptr::null_mut()
        } else {
            (*wp).w_llist
        };
        if !qi.is_null() {
            return qi;
        }
        *new_ll = true;
        qf_alloc_stack(QFLT_LOCATION, 1)
    }
}

/// Add an entry for every line of one help file that the pattern matches.
///
/// # Safety
///
/// `qfl` must be a live list, `fname` NUL-terminated and `p_regmatch` a
/// compiled pattern.
unsafe fn hgr_search_file(qfl: *mut qf_list_T, fname: *mut c_char, p_regmatch: *mut regmatch_T) {
    // Where each line is read. Upstream shares `IObuff`, which the entry
    // it builds and the messages it may raise both write.
    let mut read = [0 as c_char; IOSIZE as usize];
    // SAFETY: forwarded from the caller.
    unsafe {
        let fd = os_fopen(fname, c"r".as_ptr());
        if fd.is_null() {
            return;
        }

        let line = read.as_mut_ptr();
        let mut lnum: linenr_T = 1;
        while !vim_fgets(line, IOSIZE, fd) && !got_int.get() {
            if vim_regexec(p_regmatch, line, 0) {
                // Remove the trailing CR, LF, spaces, etc.
                let mut l = strlen(line);
                while l > 0 && *line.add(l - 1) as c_int <= ' ' as c_int {
                    l -= 1;
                    *line.add(l) = NUL as c_char;
                }

                qf_add_entry(
                    qfl,
                    &NewEntry {
                        fname,
                        lnum,
                        col: (*p_regmatch).startp[0].offset_from(line) as c_int + 1,
                        end_col: (*p_regmatch).endp[0].offset_from(line) as c_int + 1,
                        // A help entry, which `qf_jump` opens as help.
                        kind: 1,
                        ..NewEntry::new(line)
                    },
                );
            }
            lnum += 1;
            line_breakcheck();
        }
        fclose(fd);
    }
}

/// Search every help file in `dir`'s `doc/` directory, skipping the ones
/// written in another language than `lang`.
///
/// # Safety
///
/// `qfl` must be a live list, `p_regmatch` a compiled pattern and `lang`
/// null or NUL-terminated.
unsafe fn hgr_search_files_in_dir(
    qfl: *mut qf_list_T,
    dir: &[u8],
    p_regmatch: *mut regmatch_T,
    lang: *const c_char,
) {
    // SAFETY: the caller's list, pattern and language, plus one owned file
    // pattern that stays alive for the whole call.
    unsafe {
        // Find all "*.txt" and "*.??x" files in the "doc" directory.
        // Upstream builds this in `NameBuff` with `add_pathsep` and
        // `strcat`, which a 'runtimepath' entry close to MAXPATHL overruns;
        // the pattern is owned here instead.
        let mut pattern: Vec<u8> = dir.to_vec();
        pattern.push(0);
        let base: *const c_char = pattern.as_ptr().cast();
        if !dir.is_empty() && after_pathsep(base, base.add(dir.len())) == 0 {
            pattern[dir.len()] = PATHSEP as u8;
            pattern.push(0);
        }
        pattern.pop();
        pattern.extend_from_slice(HELP_FILES);
        pattern.push(0);

        let mut fcount: c_int = 0;
        let mut fnames: *mut *mut c_char = ptr::null_mut();
        let mut arg: *mut c_char = pattern.as_mut_ptr().cast();
        if gen_expand_wildcards(
            1,
            &raw mut arg,
            &raw mut fcount,
            &raw mut fnames,
            ExpandFlags::FILE | ExpandFlags::SILENT,
        ) != OK
            || fcount <= 0
        {
            return;
        }

        let mut fi = 0;
        while fi < fcount && !got_int.get() {
            let fname = *fnames.offset(fi as isize);
            if lang.is_null() || wanted_language(lang, fname) {
                hgr_search_file(qfl, fname, p_regmatch);
            }
            fi += 1;
        }
        free_wild(fcount, fnames);
    }
}

/// Whether a help file is one `lang` asked for. The language is the two
/// characters before the extension's last one, so `foo.frx` is French —
/// except that `en` also claims every plain `.txt` file.
///
/// # Safety
///
/// Both strings must be NUL-terminated, and `fname` at least three bytes
/// long, which every name the wildcard produced is.
unsafe fn wanted_language(lang: *const c_char, fname: *const c_char) -> bool {
    // SAFETY: the caller's promise.
    unsafe {
        let ext = fname.add(strlen(fname)).offset(-3);
        strncasecmp(lang, ext, 2) == 0
            || (strncasecmp(lang, c"en".as_ptr(), 2) == 0
                && strncasecmp(c"txt".as_ptr(), ext, 3) == 0)
    }
}

/// Search the help files of every `'runtimepath'` entry.
///
/// # Safety
///
/// `qfl` must be a live list and `p_regmatch` a compiled pattern.
unsafe fn hgr_search_in_rtp(qfl: *mut qf_list_T, p_regmatch: *mut regmatch_T, lang: *const c_char) {
    let mut dir = [0 as c_char; MAXPATHL as usize];
    // SAFETY: forwarded from the caller; `dir` holds MAXPATHL bytes.
    unsafe {
        let mut p = p_rtp.get();
        while *p as c_int != NUL && !got_int.get() {
            let len = copy_option_part(
                &raw mut p,
                dir.as_mut_ptr(),
                MAXPATHL as size_t,
                c",".as_ptr().cast_mut(),
            );
            let entry = core::slice::from_raw_parts(dir.as_ptr().cast::<u8>(), len);
            hgr_search_files_in_dir(qfl, entry, p_regmatch, lang);
        }
    }
}

/// `:helpgrep` and `:lhelpgrep`.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn ex_helpgrep(eap: *mut exarg_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut qi = ql_info.get();
        debug_assert!(!qi.is_null());

        let au_name = match (*eap).cmdidx {
            CMD_helpgrep => Some(c"helpgrep"),
            CMD_lhelpgrep => Some(c"lhelpgrep"),
            _ => None,
        };
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

        // Make 'cpoptions' empty, the 'l' flag should not be used here.
        let save_cpo = p_cpo.get();
        p_cpo.set(empty_option());

        let mut new_qi = false;
        if is_loclist_cmd((*eap).cmdidx as c_int) {
            qi = hgr_get_ll(&mut new_qi);
        }

        incr_quickfix_busy();

        // Check for a specified language.
        let lang = check_help_lang((*eap).arg);
        let mut regmatch = regmatch_T {
            regprog: vim_regcomp((*eap).arg, RE_MAGIC + RE_STRING),
            rm_ic: false,
            ..regmatch_T::default()
        };
        let updated = !regmatch.regprog.is_null();
        if updated {
            // Create a new quickfix list.
            qf_new_list(qi, qf_cmdtitle(*(*eap).cmdlinep));
            let qfl = qf_get_curlist(qi);

            hgr_search_in_rtp(qfl, &raw mut regmatch, lang);
            vim_regfree(regmatch.regprog);

            (*qfl).qf_nonevalid = false;
            (*qfl).qf_ptr = (*qfl).qf_start;
            (*qfl).qf_index = 1;
            qf_list_changed(qfl);
        }

        if is_empty_option(p_cpo.get()) {
            p_cpo.set(save_cpo);
        } else {
            // Darn, some plugin changed the value. If it's still empty it
            // was changed and restored, need to restore the complicated way.
            if *p_cpo.get() as c_int == NUL {
                set_option_value_give_err(
                    kOptCpoptions,
                    OptVal {
                        type_0: kOptValTypeString,
                        data: OptValData {
                            string: cstr_as_string(save_cpo),
                        },
                    },
                    OptionSetFlags::NONE,
                );
            }
            free_string_option(save_cpo);
        }

        if updated {
            // This may open a window and source scripts, so it waits until
            // 'cpo' has been restored.
            qf_update_buffer(qi, ptr::null_mut());
        }

        if let Some(name) = au_name {
            apply_autocmds(
                EVENT_QUICKFIXCMDPOST,
                name.as_ptr().cast_mut(),
                (*curbuf.get()).b_fname,
                true,
                curbuf.get(),
            );
            // When adding to an existing location list stack, an autocommand
            // may have made that stack invalid, in which case there is
            // nothing left to jump to.
            if !new_qi
                && (*qi).qfl_type == QFLT_LOCATION as qfltype_T
                && qf_find_win_with_loclist(qi).is_null()
            {
                decr_quickfix_busy();
                return;
            }
        }

        // Jump to the first match.
        if !qf_list_empty(qf_get_curlist(qi)) {
            qf_jump(qi, 0, 0, false as c_int);
        } else {
            semsg_c!(gettext(&raw const e_nomatch2 as *const c_char), (*eap).arg);
        }

        decr_quickfix_busy();

        if (*eap).cmdidx == CMD_lhelpgrep && new_qi {
            if !bt_help((*curwin.get()).w_buffer) || (*curwin.get()).w_llist == qi {
                // The help window was not opened, or it already points at
                // the right location list: the new one is not wanted.
                ll_free_all(&raw mut qi);
            } else if (*curwin.get()).w_llist.is_null() {
                // The current window had no location list before, so it
                // takes the new one.
                (*curwin.get()).w_llist = qi;
            }
        }
    }
}
