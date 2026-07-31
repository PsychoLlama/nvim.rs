//! Reading a plain word list.
//!
//! The simplest input `:mkspell` accepts: one word per line, no affix file
//! beside it. Flags may follow a `/` — `=` for keep-case, `!` for a banned
//! word, `?` for a rare one, and digits naming the regions the word belongs
//! to. Lines starting with `/` are headers: `/encoding=` says what the file
//! is in, `/regions=` names the regions the digits refer to.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use crate::src::nvim::fileio::vim_fgets;
use crate::src::nvim::main::{IObuff, e_notopen, got_int, p_enc};
use crate::src::nvim::mbyte::{convert_setup, enc_canonize, string_convert};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::message::{semsg, smsg};
use crate::src::nvim::os::fs::os_fopen;
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{fclose, gettext, strcpy, strlen, strncmp};
use crate::src::nvim::strings::{has_non_ascii, vim_snprintf, vim_strchr};
use crate::src::nvim::types::{linenr_T, size_t, uint8_t};

use super::wordtree::store_word;
use super::{
    CONV_NONE, FAIL, IOSIZE, MAXLINELEN, MAXREGIONS, NUL, OK, WF_BANNED, WF_FIXCAP, WF_KEEPCAP,
    WF_RARE, WF_REGION, spell_message, spellinfo_T,
};

/// Read a plain word list: one word per line, with optional `/` flags, and
/// `/encoding=` and `/regions=` header lines.
///
/// # Safety
///
/// `fname` must be a NUL-terminated path.
pub unsafe fn spell_read_wordfile(spin: *mut spellinfo_T, fname: *mut c_char) -> c_int {
    // SAFETY: the caller promises the path; `rline` is MAXLINELEN, which is
    // the bound `vim_fgets` is given.
    unsafe {
        let fd = os_fopen(fname, c"r".as_ptr());
        if fd.is_null() {
            semsg(gettext((&raw const e_notopen).cast()), fname);
            return FAIL;
        }
        vim_snprintf(
            IObuff.ptr().cast::<c_char>(),
            IOSIZE as size_t,
            gettext(c"Reading word file %s...".as_ptr()),
            fname,
        );
        spell_message(&*spin, IObuff.ptr().cast::<c_char>());

        let mut rline: [c_char; MAXLINELEN as usize] = [0; MAXLINELEN as usize];
        let mut pc: *mut c_char = core::ptr::null_mut();
        let mut lnum: linenr_T = 0;
        let mut retval = OK;
        let mut did_word = false;
        let mut non_ascii = 0;

        while !vim_fgets(rline.as_mut_ptr(), MAXLINELEN, fd) && !got_int.get() {
            line_breakcheck();
            lnum += 1;
            if rline[0] as c_int == b'#' as c_int {
                continue;
            }
            let mut l = strlen(rline.as_ptr()) as usize;
            while l > 0 && rline[l - 1] as uint8_t as c_int <= b' ' as c_int {
                l -= 1;
            }
            if l == 0 {
                continue;
            }
            rline[l] = NUL as c_char;

            xfree(pc.cast());
            pc = core::ptr::null_mut();
            let mut line = if (*spin).si_conv.vc_type != CONV_NONE as c_int {
                pc = string_convert(
                    &raw mut (*spin).si_conv,
                    rline.as_mut_ptr(),
                    core::ptr::null_mut(),
                );
                if pc.is_null() {
                    smsg(
                        0,
                        gettext(c"Conversion failure for word in %s line %d: %s".as_ptr()),
                        fname,
                        lnum,
                        rline.as_mut_ptr(),
                    );
                    continue;
                }
                pc
            } else {
                rline.as_mut_ptr()
            };

            if *line as c_int == b'/' as c_int {
                line = line.add(1);
                read_wordfile_header(spin, line, fname, lnum, did_word);
                continue;
            }

            // A word, with optional flags after a "/".
            let mut flags = 0;
            let mut regionmask = (*spin).si_region;
            let mut p = vim_strchr(line, b'/' as c_int);
            if !p.is_null() {
                *p = NUL as c_char;
                p = p.add(1);
                while *p as c_int != NUL {
                    match *p as u8 {
                        b'=' => flags |= WF_KEEPCAP as c_int | WF_FIXCAP as c_int,
                        b'!' => flags |= WF_BANNED as c_int,
                        b'?' => flags |= WF_RARE as c_int,
                        d if d.is_ascii_digit() => {
                            // The first digit replaces the default set of
                            // regions, the rest add to it.
                            if flags & WF_REGION as c_int == 0 {
                                regionmask = 0;
                            }
                            flags |= WF_REGION as c_int;
                            let n = (d - b'0') as c_int;
                            if n == 0 || n > (*spin).si_region_count {
                                smsg(
                                    0,
                                    gettext(c"Invalid region nr in %s line %d: %s".as_ptr()),
                                    fname,
                                    lnum,
                                    p,
                                );
                                break;
                            }
                            regionmask |= 1 << (n - 1);
                        }
                        _ => {
                            smsg(
                                0,
                                gettext(c"Unrecognized flags in %s line %d: %s".as_ptr()),
                                fname,
                                lnum,
                                p,
                            );
                            break;
                        }
                    }
                    p = p.add(1);
                }
            }

            if (*spin).si_ascii != 0 && has_non_ascii(line) {
                non_ascii += 1;
            } else if store_word(
                &mut *spin,
                line,
                flags,
                regionmask,
                core::ptr::null(),
                false,
            ) == FAIL
            {
                retval = FAIL;
                break;
            } else {
                did_word = true;
            }
        }

        xfree(pc.cast());
        fclose(fd);
        if (*spin).si_ascii != 0 && non_ascii > 0 {
            vim_snprintf(
                IObuff.ptr().cast::<c_char>(),
                IOSIZE as size_t,
                gettext(c"Ignored %d words with non-ASCII characters".as_ptr()),
                non_ascii,
            );
            spell_message(&*spin, IObuff.ptr().cast::<c_char>());
        }
        retval
    }
}

/// Handle a `/`-prefixed header line of a word file.
///
/// `line` points just past the `/`.
///
/// # Safety
///
/// `line` and `fname` must be NUL-terminated.
unsafe fn read_wordfile_header(
    spin: *mut spellinfo_T,
    mut line: *mut c_char,
    fname: *mut c_char,
    lnum: linenr_T,
    did_word: bool,
) {
    // SAFETY: the caller promises the strings; the region name is copied
    // only after its length has been checked against the array.
    unsafe {
        if strncmp(line, c"encoding=".as_ptr(), 9) == 0 {
            if (*spin).si_conv.vc_type != CONV_NONE as c_int {
                smsg(
                    0,
                    gettext(c"Duplicate /encoding= line ignored in %s line %d: %s".as_ptr()),
                    fname,
                    lnum,
                    line.sub(1),
                );
            } else if did_word {
                smsg(
                    0,
                    gettext(c"/encoding= line after word ignored in %s line %d: %s".as_ptr()),
                    fname,
                    lnum,
                    line.sub(1),
                );
            } else {
                line = line.add(9);
                let enc = enc_canonize(line);
                if (*spin).si_ascii == 0
                    && convert_setup(&raw mut (*spin).si_conv, enc, p_enc.get()) == FAIL
                {
                    smsg(
                        0,
                        gettext(c"Conversion in %s not supported: from %s to %s".as_ptr()),
                        fname,
                        line,
                        p_enc.get(),
                    );
                }
                xfree(enc.cast());
                (*spin).si_conv.vc_fail = true;
            }
        } else if strncmp(line, c"regions=".as_ptr(), 8) == 0 {
            if (*spin).si_region_count > 1 {
                smsg(
                    0,
                    gettext(c"Duplicate /regions= line ignored in %s line %d: %s".as_ptr()),
                    fname,
                    lnum,
                    line,
                );
            } else {
                line = line.add(8);
                if strlen(line) > (MAXREGIONS as c_int * 2) as size_t {
                    smsg(
                        0,
                        gettext(c"Too many regions in %s line %d: %s".as_ptr()),
                        fname,
                        lnum,
                        line,
                    );
                } else {
                    (*spin).si_region_count = strlen(line) as c_int / 2;
                    strcpy((&raw mut (*spin).si_region_name).cast::<c_char>(), line);
                    (*spin).si_region = (1 << (*spin).si_region_count) - 1;
                }
            }
        } else {
            smsg(
                0,
                gettext(c"/ line ignored in %s line %d: %s".as_ptr()),
                fname,
                lnum,
                line.sub(1),
            );
        }
    }
}
