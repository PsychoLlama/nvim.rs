//! Reading a plain word list.
//!
//! The simplest input `:mkspell` accepts: one word per line, no affix file
//! beside it. Flags may follow a `/` — `=` for keep-case, `!` for a banned
//! word, `?` for a rare one, and digits naming the regions the word belongs
//! to. Lines starting with `/` are headers: `/encoding=` says what the file
//! is in, `/regions=` names the regions the digits refer to.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::message_fmt::c_str;
use crate::semsg;
use crate::smsg;
use core::ffi::{CStr, c_char, c_int};

use crate::fileio::vim_fgets;
use crate::main::{got_int, p_enc};
use crate::mbyte::{convert_setup, enc_canonize, string_convert};
use crate::memory::xfree;
use crate::os::cshim::strncmp;
use crate::os::fs::os_fopen;
use crate::os::input::line_breakcheck;
use crate::strings::{has_non_ascii, vim_strchr};
use crate::types::{CONV_NONE, Failed, NUL, linenr_T, size_t, uint8_t};
use ::libc::{fclose, strcpy, strlen};

use super::wordtree::store_word;
use super::{
    MAXLINELEN, MAXREGIONS, WF_BANNED, WF_FIXCAP, WF_KEEPCAP, WF_RARE, WF_REGION,
    spell_message_fmt, spellinfo_T,
};

/// Read a plain word list: one word per line, with optional `/` flags, and
/// `/encoding=` and `/regions=` header lines.
///
/// # Safety
///
/// `fname` must be a NUL-terminated path.
pub(super) unsafe fn spell_read_wordfile(
    spin: *mut spellinfo_T,
    fname: *mut c_char,
) -> Result<(), Failed> {
    // SAFETY: the caller promises the path; `rline` is MAXLINELEN, which is
    // the bound `vim_fgets` is given.
    let fd = unsafe { os_fopen(fname, c"r".as_ptr()) };
    if fd.is_null() {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let fname = unsafe { c_str(fname) };
        semsg!("E484: Can't open file {fname}");
        return Err(Failed);
    }
    let name = unsafe { CStr::from_ptr(fname) }.to_string_lossy();
    spell_message_fmt(
        unsafe { &*spin },
        format_args!("Reading word file {name}..."),
    );

    let mut rline: [c_char; MAXLINELEN as usize] = [0; MAXLINELEN as usize];
    let mut pc: *mut c_char = core::ptr::null_mut();
    let mut lnum: linenr_T = 0;
    let mut retval = Ok(());
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
                // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
                let (fname, rline) = unsafe { (c_str(fname), c_str(rline.as_mut_ptr())) };
                smsg!(
                    0,
                    "Conversion failure for word in {fname} line {}: {rline}",
                    lnum
                );
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
                            // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
                            let (fname, p) = unsafe { (c_str(fname), c_str(p)) };
                            smsg!(0, "Invalid region nr in {fname} line {}: {p}", lnum);
                            break;
                        }
                        regionmask |= 1 << (n - 1);
                    }
                    _ => {
                        // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
                        let (fname, p) = unsafe { (c_str(fname), c_str(p)) };
                        smsg!(0, "Unrecognized flags in {fname} line {}: {p}", lnum);
                        break;
                    }
                }
                p = unsafe { p.add(1) };
            }
        }

        let none = core::ptr::null();
        if unsafe { (*spin).si_ascii } != 0 && unsafe { has_non_ascii(line) } {
            non_ascii += 1;
        } else if unsafe { store_word(&mut *spin, line, flags, regionmask, none, false) }.is_err() {
            retval = Err(Failed);
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
            // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
            let (fname, arg2) = unsafe { (c_str(fname), c_str(line.sub(1))) };
            smsg!(
                0,
                "Duplicate /encoding= line ignored in {fname} line {}: {arg2}",
                lnum
            );
        } else if did_word {
            // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
            let (fname, arg2) = unsafe { (c_str(fname), c_str(line.sub(1))) };
            smsg!(
                0,
                "/encoding= line after word ignored in {fname} line {}: {arg2}",
                lnum
            );
        } else {
            line = unsafe { line.add(9) };
            let enc = unsafe { enc_canonize(line) };
            if unsafe { (*spin).si_ascii } == 0
                && unsafe { convert_setup(&raw mut (*spin).si_conv, enc, p_enc.get()) }.is_err()
            {
                // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
                let (fname, line, arg2) =
                    unsafe { (c_str(fname), c_str(line), c_str(p_enc.get())) };
                smsg!(
                    0,
                    "Conversion in {fname} not supported: from {line} to {arg2}"
                );
            }
            unsafe { xfree(enc.cast()) };
            unsafe { (*spin).si_conv.vc_fail = true };
        }
    } else if unsafe { strncmp(line, c"regions=".as_ptr(), 8) } == 0 {
        if unsafe { (*spin).si_region_count } > 1 {
            // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
            let (fname, line) = unsafe { (c_str(fname), c_str(line)) };
            smsg!(
                0,
                "Duplicate /regions= line ignored in {fname} line {}: {line}",
                lnum
            );
        } else {
            line = unsafe { line.add(8) };
            if unsafe { strlen(line) } > (MAXREGIONS as c_int * 2) as size_t {
                // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
                let (fname, line) = unsafe { (c_str(fname), c_str(line)) };
                smsg!(0, "Too many regions in {fname} line {}: {line}", lnum);
            } else {
                unsafe { (*spin).si_region_count = strlen(line) as c_int / 2 };
                unsafe { strcpy((&raw mut (*spin).si_region_name).cast::<c_char>(), line) };
                unsafe { (*spin).si_region = (1 << (*spin).si_region_count) - 1 };
            }
        }
    } else {
        // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
        let (fname, arg2) = unsafe { (c_str(fname), c_str(line.sub(1))) };
        smsg!(0, "/ line ignored in {fname} line {}: {arg2}", lnum);
    }
}
