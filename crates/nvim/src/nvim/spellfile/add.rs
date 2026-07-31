//! Adding a word to a user's own spell file.
//!
//! `zg` and friends append to a `.add` file — a plain word list beside the
//! user's `'spellfile'` — and then rebuild the `.add.spl` from it, so the
//! word takes effect without a full `:mkspell`.
//!
//! [`init_spellfile`] picks the path the first time one is needed: the
//! first writable `spell` directory on 'runtimepath', named after the
//! language and encoding the buffer is actually using.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::buffer::buflist_findname_exp;
use crate::src::nvim::drawscreen::{UPD_SOME_VALID, redraw_all_later};
use crate::src::nvim::fileio::{buf_reload, vim_fgets, vim_tempname};
use crate::src::nvim::main::{NameBuff, curbuf, curwin, e_bufloaded, e_notopen, e_notset};
use crate::src::nvim::memory::{xfree, xmalloc, xmemcpyz, xstrlcat, xstrlcpy};
use crate::src::nvim::message::{emsg, semsg, smsg};
use crate::src::nvim::option::{copy_option_part, set_option_value_give_err};
use crate::src::nvim::options::kOptSpellfile;
use crate::src::nvim::os::env::home_replace;
use crate::src::nvim::os::fs::{os_fopen, os_mkdir, os_mkdir_recurse};
use crate::src::nvim::os::libc::{
    __errno_location, fclose, fprintf, fputc, fseek, ftell, gettext, strerror, strlen, strncmp,
    strstr,
};
use crate::src::nvim::os::stdpaths::get_xdg_home;
use crate::src::nvim::path::{dir_of_file_exists, path_tail, path_tail_with_sep, vim_ispathsep};
use crate::src::nvim::spell::{int_wordlist, spell_enc};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr};
use crate::src::nvim::types::{
    FILE, OptVal, OptValData, SpellAddType, buf_T, int32_t, langp_T, size_t, uint8_t,
};
use crate::src::nvim::undo::bufIsChanged;

use super::wordtree::valid_spell_word;
use super::{
    MAXPATHL, MAXWLEN, NUL, OPT_LOCAL, SEEK_SET, SPELL_ADD_BAD, SPELL_ADD_RARE,
    e_illegal_character_in_word, false_0, kOptValTypeString, kXDGDataHome, mkspell, true_0,
};

pub unsafe fn spell_add_word(
    mut word: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut what: SpellAddType,
    mut idx: ::core::ffi::c_int,
    mut undo: bool,
) {
    // SAFETY: a straight move of the transpiled body; the
    // preconditions are unchanged and stated on the caller side.
    unsafe {
        let mut fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut new_spf: bool = false_0 != 0;
        let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut fnamebuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut line: [::core::ffi::c_char; 508] = [0; 508];
        let mut spf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if !valid_spell_word(word, word.offset(len as isize)) {
            emsg(gettext(e_illegal_character_in_word.get()));
            return;
        }
        if idx == 0 as ::core::ffi::c_int {
            if (*int_wordlist.ptr()).is_null() {
                int_wordlist.set(vim_tempname());
                if (*int_wordlist.ptr()).is_null() {
                    return;
                }
            }
            fname = int_wordlist.get();
        } else {
            let mut i: ::core::ffi::c_int = 0;
            if *(*(*curwin.get()).w_s).b_p_spf as ::core::ffi::c_int == NUL {
                init_spellfile();
                new_spf = true_0 != 0;
            }
            if *(*(*curwin.get()).w_s).b_p_spf as ::core::ffi::c_int == NUL {
                semsg(
                    gettext(&raw const e_notset as *const ::core::ffi::c_char),
                    b"spellfile\0".as_ptr() as *const ::core::ffi::c_char,
                );
                return;
            }
            fnamebuf = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
            spf = (*(*curwin.get()).w_s).b_p_spf;
            i = 1 as ::core::ffi::c_int;
            while *spf as ::core::ffi::c_int != NUL {
                copy_option_part(
                    &raw mut spf,
                    fnamebuf,
                    MAXPATHL as size_t,
                    b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                if i == idx {
                    break;
                }
                if *spf as ::core::ffi::c_int == NUL {
                    semsg(
                        gettext(b"E765: 'spellfile' does not have %d entries\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        idx,
                    );
                    xfree(fnamebuf as *mut ::core::ffi::c_void);
                    return;
                }
                i += 1;
            }
            buf = buflist_findname_exp(fnamebuf);
            if !buf.is_null() && (*buf).b_ml.ml_mfp.is_null() {
                buf = ::core::ptr::null_mut::<buf_T>();
            }
            if !buf.is_null() && bufIsChanged(buf) as ::core::ffi::c_int != 0 {
                emsg(gettext(
                    &raw const e_bufloaded as *const ::core::ffi::c_char,
                ));
                xfree(fnamebuf as *mut ::core::ffi::c_void);
                return;
            }
            fname = fnamebuf;
        }
        if what as ::core::ffi::c_uint == SPELL_ADD_BAD as ::core::ffi::c_int as ::core::ffi::c_uint
            || undo as ::core::ffi::c_int != 0
        {
            let mut fpos_next: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut fpos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            fd = os_fopen(fname, b"r\0".as_ptr() as *const ::core::ffi::c_char);
            if !fd.is_null() {
                while !vim_fgets(
                    &raw mut line as *mut ::core::ffi::c_char,
                    MAXWLEN as ::core::ffi::c_int * 2 as ::core::ffi::c_int,
                    fd,
                ) {
                    fpos = fpos_next;
                    fpos_next = ftell(fd) as ::core::ffi::c_int;
                    if fpos_next < 0 as ::core::ffi::c_int {
                        break;
                    }
                    if !(strncmp(
                        word,
                        &raw mut line as *mut ::core::ffi::c_char,
                        len as size_t,
                    ) == 0 as ::core::ffi::c_int
                        && (line[len as usize] as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                            || (line[len as usize] as uint8_t as ::core::ffi::c_int)
                                < ' ' as ::core::ffi::c_int))
                    {
                        continue;
                    }
                    fclose(fd);
                    fd = os_fopen(fname, b"r+\0".as_ptr() as *const ::core::ffi::c_char);
                    if fd.is_null() {
                        break;
                    }
                    if fseek(fd, fpos as ::core::ffi::c_long, SEEK_SET) == 0 as ::core::ffi::c_int {
                        fputc('#' as ::core::ffi::c_int, fd);
                        if undo {
                            home_replace(
                                ::core::ptr::null::<buf_T>(),
                                fname,
                                NameBuff.ptr() as *mut ::core::ffi::c_char,
                                MAXPATHL as size_t,
                                true_0 != 0,
                            );
                            smsg(
                                0 as ::core::ffi::c_int,
                                gettext(b"Word '%.*s' removed from %s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                len,
                                word,
                                NameBuff.ptr() as *mut ::core::ffi::c_char,
                            );
                        }
                    }
                    if fseek(fd, fpos_next as ::core::ffi::c_long, SEEK_SET)
                        == 0 as ::core::ffi::c_int
                    {
                        continue;
                    }
                    semsg(
                        b"%s: %s\0".as_ptr() as *const ::core::ffi::c_char,
                        gettext(b"Seek error in spellfile\0".as_ptr() as *const ::core::ffi::c_char),
                        strerror(*__errno_location()),
                    );
                    break;
                }
                if !fd.is_null() {
                    fclose(fd);
                }
            }
        }
        if !undo {
            fd = os_fopen(fname, b"a\0".as_ptr() as *const ::core::ffi::c_char);
            if fd.is_null() && new_spf as ::core::ffi::c_int != 0 {
                let mut p: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                if !dir_of_file_exists(fname) && {
                    p = path_tail_with_sep(fname);
                    p != fname
                } {
                    let mut c: ::core::ffi::c_char = *p;
                    *p = NUL as ::core::ffi::c_char;
                    os_mkdir(fname, 0o755 as int32_t);
                    *p = c;
                    fd = os_fopen(fname, b"a\0".as_ptr() as *const ::core::ffi::c_char);
                }
            }
            if fd.is_null() {
                semsg(
                    gettext(&raw const e_notopen as *const ::core::ffi::c_char),
                    fname,
                );
            } else {
                if what as ::core::ffi::c_uint
                    == SPELL_ADD_BAD as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    fprintf(
                        fd,
                        b"%.*s/!\n\0".as_ptr() as *const ::core::ffi::c_char,
                        len,
                        word,
                    );
                } else if what as ::core::ffi::c_uint
                    == SPELL_ADD_RARE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    fprintf(
                        fd,
                        b"%.*s/?\n\0".as_ptr() as *const ::core::ffi::c_char,
                        len,
                        word,
                    );
                } else {
                    fprintf(
                        fd,
                        b"%.*s\n\0".as_ptr() as *const ::core::ffi::c_char,
                        len,
                        word,
                    );
                }
                fclose(fd);
                home_replace(
                    ::core::ptr::null::<buf_T>(),
                    fname,
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    MAXPATHL as size_t,
                    true_0 != 0,
                );
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"Word '%.*s' added to %s\0".as_ptr() as *const ::core::ffi::c_char),
                    len,
                    word,
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                );
            }
        }
        if !fd.is_null() {
            mkspell(
                1 as ::core::ffi::c_int,
                &raw mut fname,
                false_0 != 0,
                true_0 != 0,
                true_0 != 0,
            );
            if !buf.is_null() {
                buf_reload(buf, (*buf).b_orig_mode, false_0 != 0);
            }
            redraw_all_later(UPD_SOME_VALID);
        }
        xfree(fnamebuf as *mut ::core::ffi::c_void);
    }
}
unsafe fn init_spellfile() {
    // SAFETY: a straight move of the transpiled body; the
    // preconditions are unchanged and stated on the caller side.
    unsafe {
        let mut lend: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut aspath: bool = false_0 != 0;
        let mut lstart: *mut ::core::ffi::c_char = (*curbuf.get()).b_s.b_p_spl;
        if *(*(*curwin.get()).w_s).b_p_spl as ::core::ffi::c_int == NUL
            || (*(*curwin.get()).w_s).b_langp.ga_len <= 0 as ::core::ffi::c_int
        {
            return;
        }
        lend = (*(*curwin.get()).w_s).b_p_spl;
        while *lend as ::core::ffi::c_int != NUL
            && vim_strchr(
                b",._\0".as_ptr() as *const ::core::ffi::c_char,
                *lend as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
        {
            if vim_ispathsep(*lend as ::core::ffi::c_int) {
                aspath = true_0 != 0;
                lstart = lend.offset(1 as ::core::ffi::c_int as isize);
            }
            lend = lend.offset(1);
        }
        let mut buf: *mut ::core::ffi::c_char =
            xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        let mut buf_len: size_t = MAXPATHL as size_t;
        if !aspath {
            let mut xdg_path: *mut ::core::ffi::c_char = get_xdg_home(kXDGDataHome);
            xstrlcpy(buf, xdg_path, buf_len);
            xfree(xdg_path as *mut ::core::ffi::c_void);
            xstrlcat(
                buf,
                b"/site/spell\0".as_ptr() as *const ::core::ffi::c_char,
                buf_len,
            );
            let mut failed_dir: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            if os_mkdir_recurse(
                buf,
                0o755 as int32_t,
                &raw mut failed_dir,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            ) != 0 as ::core::ffi::c_int
            {
                xfree(buf as *mut ::core::ffi::c_void);
                xfree(failed_dir as *mut ::core::ffi::c_void);
                return;
            }
        } else {
            if lend.offset_from((*curbuf.get()).b_s.b_p_spl) as size_t >= buf_len {
                xfree(buf as *mut ::core::ffi::c_void);
                return;
            }
            xmemcpyz(
                buf as *mut ::core::ffi::c_void,
                (*curbuf.get()).b_s.b_p_spl as *const ::core::ffi::c_void,
                lend.offset_from((*curbuf.get()).b_s.b_p_spl) as size_t,
            );
        }
        vim_snprintf(
            buf.offset(strlen(buf) as isize),
            buf_len.wrapping_sub(strlen(buf)),
            b"/%.*s\0".as_ptr() as *const ::core::ffi::c_char,
            lend.offset_from(lstart) as ::core::ffi::c_int,
            lstart,
        );
        let mut fname: *mut ::core::ffi::c_char = (*(*((*(*curwin.get()).w_s).b_langp.ga_data
            as *mut langp_T)
            .offset(0 as ::core::ffi::c_int as isize))
        .lp_slang)
            .sl_fname;
        let mut enc_suffix: *const ::core::ffi::c_char = if !fname.is_null()
            && !strstr(
                path_tail(fname),
                b".ascii.\0".as_ptr() as *const ::core::ffi::c_char,
            )
            .is_null()
        {
            b"ascii\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            spell_enc() as *const ::core::ffi::c_char
        };
        vim_snprintf(
            buf.offset(strlen(buf) as isize),
            buf_len.wrapping_sub(strlen(buf)),
            b".%s.add\0".as_ptr() as *const ::core::ffi::c_char,
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
            OPT_LOCAL as ::core::ffi::c_int,
        );
        xfree(buf as *mut ::core::ffi::c_void);
    }
}
