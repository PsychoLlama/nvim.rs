//! Reading a plain word list.
//!
//! The simplest input `:mkspell` accepts: one word per line, no affix file
//! beside it. Flags may follow a `/` — `=` for keep-case, `!` for a banned
//! word, `?` for a rare one, and digits naming the regions the word belongs
//! to. Lines starting with `/` are headers: `/encoding=` says what the file
//! is in, `/regions=` names the regions the digits refer to.

#![forbid(unsafe_code)]

use crate::message_fmt::{msg_bytes, msg_cstr};
use crate::semsg;
use crate::smsg;
use crate::spell::WordFlags;
use core::ffi::{CStr, c_int};
use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;

use crate::cstr;
use crate::fileio::Fgets;
use crate::getchar::state::got_int;
use crate::mbyte::canonize;
use crate::option::vars::p_enc;
use crate::os::input::line_breakcheck;
use crate::strings::{any_non_ascii, find_char};
use crate::types::{CONV_NONE, Failed, LineNr};

use super::wordtree::store_word;
use super::{MAXLINELEN, MAXREGIONS, SpellInfo, spell_message_fmt};

/// Read a plain word list: one word per line, with optional `/` flags, and
/// `/encoding=` and `/regions=` header lines.
pub(super) fn spell_read_wordfile(spin: &mut SpellInfo, fname: &CStr) -> Result<(), Failed> {
    let Ok(file) = File::open(OsStr::from_bytes(fname.to_bytes())) else {
        let fname = msg_cstr(fname);
        semsg!("E484: Can't open file {fname}");
        return Err(Failed);
    };
    let mut fd = Fgets::new(file);
    let name = fname.to_string_lossy();
    spell_message_fmt(&*spin, format_args!("Reading word file {name}..."));

    let mut rline: Vec<u8> = Vec::new();
    let mut lnum: LineNr = 0;
    let mut retval = Ok(());
    let mut did_word = false;
    let mut non_ascii = 0;

    while fd.line_capped(&mut rline, MAXLINELEN as usize) && !got_int.get() {
        line_breakcheck();
        lnum += 1;
        let read = cstr::until_nul(&rline);
        if read.first() == Some(&b'#') {
            continue;
        }
        // Trim trailing white space, and skip the line if nothing is left.
        let l = read.iter().rposition(|&b| b > b' ').map_or(0, |at| at + 1);
        if l == 0 {
            continue;
        }
        let read = &read[..l];

        let converted;
        let mut line: &[u8] = if spin.si_conv.vc_type != CONV_NONE {
            let Some(text) = spin.si_conv.convert(read) else {
                let (fname, rline) = (msg_cstr(fname), msg_bytes(read));
                smsg!(
                    0,
                    "Conversion failure for word in {fname} line {}: {rline}",
                    lnum
                );
                continue;
            };
            converted = text;
            cstr::until_nul(&converted)
        } else {
            read
        };

        if let Some(header) = line.strip_prefix(b"/") {
            read_wordfile_header(spin, header, fname, lnum, did_word);
            continue;
        }

        // A word, with optional flags after a "/".
        let mut flags = WordFlags::NONE;
        let mut regionmask = spin.si_region;
        if let Some(slash) = find_char(line, c_int::from(b'/')) {
            let flag_text = &line[slash + 1..];
            line = &line[..slash];
            for (i, &f) in flag_text.iter().enumerate() {
                match f {
                    b'=' => flags |= WordFlags::KEEPCAP | WordFlags::FIXCAP,
                    b'!' => flags |= WordFlags::BANNED,
                    b'?' => flags |= WordFlags::RARE,
                    d if d.is_ascii_digit() => {
                        // The first digit replaces the default set of
                        // regions, the rest add to it.
                        if !flags.has(WordFlags::REGION) {
                            regionmask = 0;
                        }
                        flags |= WordFlags::REGION;
                        let n = c_int::from(d - b'0');
                        if n == 0 || n > spin.si_region_count {
                            let (fname, p) = (msg_cstr(fname), msg_bytes(&flag_text[i..]));
                            smsg!(0, "Invalid region nr in {fname} line {}: {p}", lnum);
                            break;
                        }
                        regionmask |= 1 << (n - 1);
                    }
                    _ => {
                        let (fname, p) = (msg_cstr(fname), msg_bytes(&flag_text[i..]));
                        smsg!(0, "Unrecognized flags in {fname} line {}: {p}", lnum);
                        break;
                    }
                }
            }
        }

        if spin.si_ascii != 0 && any_non_ascii(line) {
            non_ascii += 1;
        } else if store_word(&mut *spin, line, flags, regionmask, b"", false).is_err() {
            retval = Err(Failed);
            break;
        } else {
            did_word = true;
        }
    }

    if spin.si_ascii != 0 && non_ascii > 0 {
        spell_message_fmt(
            &*spin,
            format_args!("Ignored {non_ascii} words with non-ASCII characters"),
        );
    }
    retval
}

/// Handle a `/`-prefixed header line of a word file.
///
/// `line` is what follows the `/`.
fn read_wordfile_header(
    spin: &mut SpellInfo,
    line: &[u8],
    fname: &CStr,
    lnum: LineNr,
    did_word: bool,
) {
    // The whole line, `/` included, as the messages show it.
    let whole = || {
        let mut text = b"/".to_vec();
        text.extend_from_slice(line);
        text
    };
    if let Some(enc) = line.strip_prefix(b"encoding=") {
        if spin.si_conv.vc_type != CONV_NONE {
            let whole = whole();
            let (fname, arg2) = (msg_cstr(fname), msg_bytes(&whole));
            smsg!(
                0,
                "Duplicate /encoding= line ignored in {fname} line {}: {arg2}",
                lnum
            );
        } else if did_word {
            let whole = whole();
            let (fname, arg2) = (msg_cstr(fname), msg_bytes(&whole));
            smsg!(
                0,
                "/encoding= line after word ignored in {fname} line {}: {arg2}",
                lnum
            );
        } else {
            let canon = canonize(enc);
            if spin.si_ascii == 0
                && p_enc(|value| spin.si_conv.setup(Some(canon.as_cstr()), Some(value))).is_err()
            {
                let to = p_enc(|value| value.to_bytes().to_vec());
                let (fname, line, arg2) = (msg_cstr(fname), msg_bytes(enc), msg_bytes(&to));
                smsg!(
                    0,
                    "Conversion in {fname} not supported: from {line} to {arg2}"
                );
            }
            spin.si_conv.vc_fail = true;
        }
    } else if let Some(regions) = line.strip_prefix(b"regions=") {
        if spin.si_region_count > 1 {
            let (fname, line) = (msg_cstr(fname), msg_bytes(line));
            smsg!(
                0,
                "Duplicate /regions= line ignored in {fname} line {}: {line}",
                lnum
            );
        } else if regions.len() > MAXREGIONS as usize * 2 {
            let (fname, line) = (msg_cstr(fname), msg_bytes(regions));
            smsg!(0, "Too many regions in {fname} line {}: {line}", lnum);
        } else {
            spin.si_region_count = regions.len() as c_int / 2;
            spin.si_region_name[..regions.len()].copy_from_slice(regions);
            spin.si_region_name[regions.len()] = 0;
            spin.si_region = (1 << spin.si_region_count) - 1;
        }
    } else {
        let whole = whole();
        let (fname, arg2) = (msg_cstr(fname), msg_bytes(&whole));
        smsg!(0, "/ line ignored in {fname} line {}: {arg2}", lnum);
    }
}
