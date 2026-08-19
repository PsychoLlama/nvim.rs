//! Adding a word to a user's own spell file.
//!
//! `zg`, `zw`, `zG`, `zW` and their `u`-prefixed undo forms all land in
//! [`spell_add_word`]. It appends a line to a plain word list — the user's
//! `'spellfile'`, or an internal temporary file for the `G`/`W` forms that
//! only last the session — and then compiles that list into the `.add.spl`
//! beside it, so the word takes effect without a full `:mkspell`.
//!
//! The word list is line-oriented and the flags a word carries are written
//! after a `/`, exactly as `:mkspell` reads them:
//!
//! ```text
//! word            good
//! word/!          bad ("zw")
//! word/?          rare ("zG" with 'spellsuggest' ... rare forms)
//! #word           removed; a line commented out by an undo
//! ```
//!
//! Undoing does not rewrite the file — it writes a `#` over the first
//! character of the line, which costs one seek instead of a rewrite. Adding
//! a *bad* word does the same to any existing good entry for it first,
//! because a good entry sorts ahead of the banned one and would win.
//!
//! [`init_spellfile`] picks the path the first time one is needed: the
//! first writable `spell` directory on `'runtimepath'`, named after the
//! language and encoding the buffer is actually using.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::{semsg_c, smsg_c};
use core::ffi::{c_char, c_int, c_long, c_void};

use crate::api::private::helpers::cstr_as_string;
use crate::buffer::buflist_findname_exp;
use crate::drawscreen::{UPD_SOME_VALID, redraw_all_later};
use crate::fileio::{buf_reload, vim_fgets, vim_tempname};
use crate::main::{NameBuff, curbuf, curwin, e_bufloaded, e_notopen, e_notset};
use crate::memory::{xfree, xmalloc, xmemcpyz, xstrlcat, xstrlcpy};
use crate::message::emsg;
use crate::option::{copy_option_part, set_option_value_give_err};
use crate::options::kOptSpellfile;
use crate::os::cshim::{gettext, strncmp, strstr};
use crate::os::env::home_replace;
use crate::os::fs::{os_fopen, os_mkdir, os_mkdir_recurse};
use crate::os::stdpaths::get_xdg_home;
use crate::path::{dir_of_file_exists, path_tail, path_tail_with_sep, vim_ispathsep};
use crate::spell::{int_wordlist, spell_enc};
use crate::strings::{vim_snprintf, vim_strchr};
use crate::types::{
    FILE, MAXPATHL, NUL, OptVal, OptValData, SpellAddType, buf_T, int32_t, langp_T, size_t, uint8_t,
};
use crate::undo::bufIsChanged;
use ::libc::{__errno_location, fclose, fprintf, fputc, fseek, ftell, strerror, strlen};

use super::wordtree::valid_spell_word;
use super::{
    MAXWLEN, OPT_LOCAL, SEEK_SET, SPELL_ADD_BAD, SPELL_ADD_RARE, e_illegal_character_in_word,
    kOptValTypeString, kXDGDataHome, mkspell,
};

/// Add `word[..len]` to a spell file, or take it back out again.
///
/// `what` says whether the word is good, bad (`zw`) or rare; `undo` asks
/// for the matching entry to be removed instead. `idx` selects which file:
/// zero is the session-only internal word list, and anything else is the
/// n-th entry of `'spellfile'`.
pub unsafe fn spell_add_word(
    word: *mut c_char,
    len: c_int,
    what: SpellAddType,
    idx: c_int,
    undo: bool,
) {
    unsafe {
        if !valid_spell_word(word, word.offset(len as isize)) {
            emsg(gettext(e_illegal_character_in_word.get()));
            return;
        }

        // "fnamebuf" owns the name when it came from 'spellfile'; the
        // internal word list's name is owned by the global.
        let mut fnamebuf: *mut c_char = core::ptr::null_mut();
        // The buffer the file is open in, if the user is editing it.
        let mut buf: *mut buf_T = core::ptr::null_mut();
        let mut new_spf = false;

        let fname = if idx == 0 {
            if (*int_wordlist.ptr()).is_null() {
                int_wordlist.set(vim_tempname());
                if (*int_wordlist.ptr()).is_null() {
                    return;
                }
            }
            int_wordlist.get()
        } else {
            // Give 'spellfile' a sensible default if it has none.
            if *(*(*curwin.get()).w_s).b_p_spf == 0 {
                init_spellfile();
                new_spf = true;
            }
            if *(*(*curwin.get()).w_s).b_p_spf == 0 {
                semsg_c!(
                    gettext(&raw const e_notset as *const c_char),
                    c"spellfile".as_ptr(),
                );
                return;
            }

            fnamebuf = xmalloc(MAXPATHL as size_t) as *mut c_char;
            let mut spf = (*(*curwin.get()).w_s).b_p_spf;
            let mut i = 1;
            while *spf != 0 {
                copy_option_part(
                    &raw mut spf,
                    fnamebuf,
                    MAXPATHL as size_t,
                    c",".as_ptr() as *mut c_char,
                );
                if i == idx {
                    break;
                }
                if *spf == 0 {
                    semsg_c!(
                        gettext(c"E765: 'spellfile' does not have %d entries".as_ptr()),
                        idx,
                    );
                    xfree(fnamebuf as *mut c_void);
                    return;
                }
                i += 1;
            }

            // Refuse to write the file behind the user's back if they are
            // editing it and have unsaved changes.
            buf = buflist_findname_exp(fnamebuf);
            if !buf.is_null() && (*buf).b_ml.ml_mfp.is_null() {
                buf = core::ptr::null_mut();
            }
            if !buf.is_null() && bufIsChanged(buf) {
                emsg(gettext(&raw const e_bufloaded as *const c_char));
                xfree(fnamebuf as *mut c_void);
                return;
            }

            fnamebuf
        };

        // Whether the last attempt to open the file succeeded. What happens
        // at the end hangs on this, and in C it was a test on the (already
        // closed) FILE pointer.
        let mut opened = false;

        if what == SPELL_ADD_BAD as SpellAddType || undo {
            // A good entry for the word sorts ahead of the banned one and
            // would win, so it has to go first.
            opened = comment_out_word(fname, word, len, undo);
        }

        if !undo {
            let mut fd = os_fopen(fname, c"a".as_ptr());
            if fd.is_null() && new_spf {
                // 'spellfile' was just given its default and the file will
                // not open: the "spell" directory may not exist yet.
                // init_spellfile() already checked the parent is writable.
                let p = path_tail_with_sep(fname);
                if !dir_of_file_exists(fname) && p != fname {
                    let c = *p;
                    *p = NUL as c_char;
                    os_mkdir(fname, 0o755 as int32_t);
                    *p = c;
                    fd = os_fopen(fname, c"a".as_ptr());
                }
            }
            opened = !fd.is_null();

            if fd.is_null() {
                semsg_c!(gettext(&raw const e_notopen as *const c_char), fname);
            } else {
                let format = if what == SPELL_ADD_BAD as SpellAddType {
                    c"%.*s/!\n".as_ptr()
                } else if what == SPELL_ADD_RARE as SpellAddType {
                    c"%.*s/?\n".as_ptr()
                } else {
                    c"%.*s\n".as_ptr()
                };
                fprintf(fd, format, len, word);
                fclose(fd);

                home_replace(
                    core::ptr::null(),
                    fname,
                    NameBuff.ptr() as *mut c_char,
                    MAXPATHL as size_t,
                    true,
                );
                smsg_c!(
                    0,
                    gettext(c"Word '%.*s' added to %s".as_ptr()),
                    len,
                    word,
                    NameBuff.ptr() as *mut c_char,
                );
            }
        }

        if opened {
            // Compile the word list into the .add.spl beside it, so the
            // change takes effect without a full :mkspell.
            let mut fname = fname;
            mkspell(1, &raw mut fname, false, true, true);
            if !buf.is_null() {
                buf_reload(buf, (*buf).b_orig_mode, false);
            }
            redraw_all_later(UPD_SOME_VALID);
        }

        xfree(fnamebuf as *mut c_void);
    }
}

/// Comment out every line of `fname` holding `word[..len]`, by writing a
/// `#` over its first character.
///
/// Returns whether the file could be opened at all — which is what decides
/// whether the caller recompiles it.
///
/// Reading and writing the same handle is not portable, so each hit closes
/// the file and reopens it for update; the scan then resumes from the
/// position it had reached.
unsafe fn comment_out_word(fname: *mut c_char, word: *mut c_char, len: c_int, undo: bool) -> bool {
    unsafe {
        let mut line = [0 as c_char; MAXWLEN * 2];
        let mut fd: *mut FILE = os_fopen(fname, c"r".as_ptr());
        if fd.is_null() {
            return false;
        }

        // The offsets of the line just read and of the one after it.
        let mut fpos: c_int = 0;
        let mut fpos_next: c_int = 0;
        while !vim_fgets(line.as_mut_ptr(), MAXWLEN as c_int * 2, fd) {
            fpos = fpos_next;
            fpos_next = ftell(fd) as c_int;
            if fpos_next < 0 {
                break; // should never happen
            }

            // The line holds the word when the flags or the line end
            // follow it directly.
            let matched = strncmp(word, line.as_ptr(), len as size_t) == 0
                && (line[len as usize] == b'/' as c_char
                    || (line[len as usize] as uint8_t as c_int) < ' ' as c_int);
            if !matched {
                continue;
            }

            fclose(fd);
            fd = os_fopen(fname, c"r+".as_ptr());
            if fd.is_null() {
                break;
            }
            if fseek(fd, fpos as c_long, SEEK_SET) == 0 {
                fputc('#' as c_int, fd);
                if undo {
                    home_replace(
                        core::ptr::null(),
                        fname,
                        NameBuff.ptr() as *mut c_char,
                        MAXPATHL as size_t,
                        true,
                    );
                    smsg_c!(
                        0,
                        gettext(c"Word '%.*s' removed from %s".as_ptr()),
                        len,
                        word,
                        NameBuff.ptr() as *mut c_char,
                    );
                }
            }
            if fseek(fd, fpos_next as c_long, SEEK_SET) != 0 {
                semsg_c!(
                    c"%s: %s".as_ptr(),
                    gettext(c"Seek error in spellfile".as_ptr()),
                    strerror(*__errno_location()),
                );
                break;
            }
        }

        if !fd.is_null() {
            fclose(fd);
            return true;
        }
        false
    }
}

/// Give `'spellfile'` a default: the user's own `spell` directory, or the
/// directory `'spelllang'` named if it named one by path, holding a
/// `.add` file named after the language and encoding in use.
unsafe fn init_spellfile() {
    unsafe {
        if *(*(*curwin.get()).w_s).b_p_spl == 0 || (*(*curwin.get()).w_s).b_langp.ga_len <= 0 {
            return;
        }

        // Take the first 'spelllang' entry up to a separator. When it is a
        // path, the file goes beside it and "lstart" is its last component.
        let mut lstart = (*curbuf.get()).b_s.b_p_spl;
        let mut lend = (*(*curwin.get()).w_s).b_p_spl;
        let mut aspath = false;
        while *lend != 0 && vim_strchr(c",._".as_ptr(), *lend as uint8_t as c_int).is_null() {
            if vim_ispathsep(*lend as c_int) {
                aspath = true;
                lstart = lend.offset(1);
            }
            lend = lend.offset(1);
        }

        let buf_len = MAXPATHL as size_t;
        let buf = xmalloc(buf_len) as *mut c_char;
        if aspath {
            // Use the directory 'spelllang' pointed at.
            if lend.offset_from((*curbuf.get()).b_s.b_p_spl) as size_t >= buf_len {
                xfree(buf as *mut c_void);
                return;
            }
            xmemcpyz(
                buf as *mut c_void,
                (*curbuf.get()).b_s.b_p_spl as *const c_void,
                lend.offset_from((*curbuf.get()).b_s.b_p_spl) as size_t,
            );
        } else {
            // Otherwise the user's own site directory, created if need be.
            let xdg_path = get_xdg_home(kXDGDataHome);
            xstrlcpy(buf, xdg_path, buf_len);
            xfree(xdg_path as *mut c_void);
            xstrlcat(buf, c"/site/spell".as_ptr(), buf_len);

            let mut failed_dir: *mut c_char = core::ptr::null_mut();
            if os_mkdir_recurse(
                buf,
                0o755 as int32_t,
                &raw mut failed_dir,
                core::ptr::null_mut(),
            ) != 0
            {
                xfree(buf as *mut c_void);
                xfree(failed_dir as *mut c_void);
                return;
            }
        }

        // "<dir>/<lang>"
        vim_snprintf(
            buf.add(strlen(buf)),
            buf_len - strlen(buf),
            c"/%.*s".as_ptr(),
            lend.offset_from(lstart) as c_int,
            lstart,
        );

        // The suffix has to match the file actually loaded, which may be
        // the ASCII build of the language rather than the current encoding.
        let fname =
            (*(*((*(*curwin.get()).w_s).b_langp.ga_data as *mut langp_T)).lp_slang).sl_fname;
        let enc_suffix =
            if !fname.is_null() && !strstr(path_tail(fname), c".ascii.".as_ptr()).is_null() {
                c"ascii".as_ptr()
            } else {
                spell_enc() as *const c_char
            };
        vim_snprintf(
            buf.add(strlen(buf)),
            buf_len - strlen(buf),
            c".%s.add".as_ptr(),
            enc_suffix,
        );

        set_option_value_give_err(
            kOptSpellfile,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(buf),
                },
            },
            OPT_LOCAL as c_int,
        );
        xfree(buf as *mut c_void);
    }
}
