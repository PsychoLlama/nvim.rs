//! Reading a plain word list.
//!
//! The simplest input `:mkspell` accepts: one word per line, no affix file
//! beside it. Flags may follow a `/` — `=` for keep-case, `!` for a banned
//! word, `?` for a rare one, and digits naming the regions the word belongs
//! to. Lines starting with `/` are headers: `/encoding=` says what the file
//! is in, `/regions=` names the regions the digits refer to.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::{semsg_c, smsg_c};
use core::ffi::{CStr, c_char, c_int};

use crate::fileio::vim_fgets;
use crate::main::{e_notopen, got_int, p_enc};
use crate::mbyte::{convert_setup, enc_canonize, string_convert};
use crate::memory::xfree;
use crate::os::cshim::{gettext, strncmp};
use crate::os::fs::os_fopen;
use crate::os::input::line_breakcheck;
use crate::strings::{has_non_ascii, vim_strchr};
use crate::types::{CONV_NONE, NUL, linenr_T, size_t, uint8_t};
use ::libc::{fclose, strcpy, strlen};

use super::wordtree::store_word;
use super::{
    FAIL, MAXLINELEN, MAXREGIONS, OK, WF_BANNED, WF_FIXCAP, WF_KEEPCAP, WF_RARE, WF_REGION,
    spell_message_fmt, spellinfo_T,
};

/// Read a plain word list: one word per line, with optional `/` flags, and
/// `/encoding=` and `/regions=` header lines.
///
/// # Safety
///
/// `fname` must be a NUL-terminated path.
pub(super) unsafe fn spell_read_wordfile(spin: *mut spellinfo_T, fname: *mut c_char) -> c_int {
    // SAFETY: the caller promises the path; `rline` is MAXLINELEN, which is
    // the bound `vim_fgets` is given.
    let fd = unsafe { os_fopen(fname, c"r".as_ptr()) };
    if fd.is_null() {
        unsafe { semsg_c!(gettext(e_notopen.as_ptr()), fname) };
        return FAIL;
    }
    let name = unsafe { CStr::from_ptr(fname) }.to_string_lossy();
    spell_message_fmt(
        unsafe { &*spin },
        format_args!("Reading word file {name}..."),
    );

    let mut rline: [c_char; MAXLINELEN as usize] = [0; MAXLINELEN as usize];
    let mut pc: *mut c_char = core::ptr::null_mut();
    let mut lnum: linenr_T = 0;
    let mut retval = OK;
    let mut did_word = false;
    let mut non_ascii = 0;

    while !unsafe { vim_fgets(rline.as_mut_ptr(), MAXLINELEN, fd) } && !got_int.get() {
        line_breakcheck();
        lnum += 1;
        if rline[0] as c_int == b'#' as c_int {
            continue;
        }
        let mut l = unsafe { strlen(rline.as_ptr()) } as usize;
        while l > 0 && rline[l - 1] as uint8_t as c_int <= b' ' as c_int {
            l -= 1;
        }
        if l == 0 {
            continue;
        }
        rline[l] = NUL as c_char;

        unsafe { xfree(pc.cast()) };
        pc = core::ptr::null_mut();
        let mut line = if unsafe { (*spin).si_conv.vc_type } != CONV_NONE {
            let conv = unsafe { &raw mut (*spin).si_conv };
            pc = unsafe { string_convert(conv, rline.as_mut_ptr(), core::ptr::null_mut()) };
            if pc.is_null() {
                let fmt =
                    unsafe { gettext(c"Conversion failure for word in %s line %d: %s".as_ptr()) };
                unsafe { smsg_c!(0, fmt, fname, lnum, rline.as_mut_ptr()) };
                continue;
            }
            pc
        } else {
            rline.as_mut_ptr()
        };

        if unsafe { *line } as c_int == b'/' as c_int {
            line = unsafe { line.add(1) };
            unsafe { read_wordfile_header(spin, line, fname, lnum, did_word) };
            continue;
        }

        // A word, with optional flags after a "/".
        let mut flags = 0;
        let mut regionmask = unsafe { (*spin).si_region };
        let mut p = unsafe { vim_strchr(line, b'/' as c_int) };
        if !p.is_null() {
            unsafe { *p = NUL as c_char };
            p = unsafe { p.add(1) };
            while unsafe { *p } as c_int != NUL {
                match unsafe { *p } as u8 {
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
                        if n == 0 || n > unsafe { (*spin).si_region_count } {
                            let fmt =
                                unsafe { gettext(c"Invalid region nr in %s line %d: %s".as_ptr()) };
                            unsafe { smsg_c!(0, fmt, fname, lnum, p) };
                            break;
                        }
                        regionmask |= 1 << (n - 1);
                    }
                    _ => {
                        let fmt =
                            unsafe { gettext(c"Unrecognized flags in %s line %d: %s".as_ptr()) };
                        unsafe { smsg_c!(0, fmt, fname, lnum, p) };
                        break;
                    }
                }
                p = unsafe { p.add(1) };
            }
        }

        let none = core::ptr::null();
        if unsafe { (*spin).si_ascii } != 0 && unsafe { has_non_ascii(line) } {
            non_ascii += 1;
        } else if unsafe { store_word(&mut *spin, line, flags, regionmask, none, false) } == FAIL {
            retval = FAIL;
            break;
        } else {
            did_word = true;
        }
    }

    unsafe { xfree(pc.cast()) };
    unsafe { fclose(fd) };
    if unsafe { (*spin).si_ascii } != 0 && non_ascii > 0 {
        spell_message_fmt(
            unsafe { &*spin },
            format_args!("Ignored {non_ascii} words with non-ASCII characters"),
        );
    }
    retval
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
    if unsafe { strncmp(line, c"encoding=".as_ptr(), 9) } == 0 {
        if unsafe { (*spin).si_conv.vc_type } != CONV_NONE {
            let fmt =
                unsafe { gettext(c"Duplicate /encoding= line ignored in %s line %d: %s".as_ptr()) };
            unsafe { smsg_c!(0, fmt, fname, lnum, line.sub(1)) };
        } else if did_word {
            let fmt = unsafe {
                gettext(c"/encoding= line after word ignored in %s line %d: %s".as_ptr())
            };
            unsafe { smsg_c!(0, fmt, fname, lnum, line.sub(1)) };
        } else {
            line = unsafe { line.add(9) };
            let enc = unsafe { enc_canonize(line) };
            if unsafe { (*spin).si_ascii } == 0
                && unsafe { convert_setup(&raw mut (*spin).si_conv, enc, p_enc.get()) } == FAIL
            {
                let fmt =
                    unsafe { gettext(c"Conversion in %s not supported: from %s to %s".as_ptr()) };
                unsafe { smsg_c!(0, fmt, fname, line, p_enc.get()) };
            }
            unsafe { xfree(enc.cast()) };
            unsafe { (*spin).si_conv.vc_fail = true };
        }
    } else if unsafe { strncmp(line, c"regions=".as_ptr(), 8) } == 0 {
        if unsafe { (*spin).si_region_count } > 1 {
            let fmt =
                unsafe { gettext(c"Duplicate /regions= line ignored in %s line %d: %s".as_ptr()) };
            unsafe { smsg_c!(0, fmt, fname, lnum, line) };
        } else {
            line = unsafe { line.add(8) };
            if unsafe { strlen(line) } > (MAXREGIONS as c_int * 2) as size_t {
                let fmt = unsafe { gettext(c"Too many regions in %s line %d: %s".as_ptr()) };
                unsafe { smsg_c!(0, fmt, fname, lnum, line) };
            } else {
                unsafe { (*spin).si_region_count = strlen(line) as c_int / 2 };
                unsafe { strcpy((&raw mut (*spin).si_region_name).cast::<c_char>(), line) };
                unsafe { (*spin).si_region = (1 << (*spin).si_region_count) - 1 };
            }
        }
    } else {
        let fmt = unsafe { gettext(c"/ line ignored in %s line %d: %s".as_ptr()) };
        unsafe { smsg_c!(0, fmt, fname, lnum, line.sub(1)) };
    }
}
