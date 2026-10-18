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

#![forbid(unsafe_code)]

use crate::cstr;
use crate::semsg;
use crate::smsg;
use crate::strings::has_bytes;
use crate::strings::has_char;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int};
use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::{Cursor, Seek, SeekFrom, Write};
use std::os::unix::ffi::OsStrExt;

use crate::buffer::BufRef;
use crate::buffer::find_buffer_by_name;
use crate::drawscreen::{UPD_SOME_VALID, redraw_all_later};
use crate::fileio::Fgets;
use crate::fileio::buf_reload;
use crate::fileio::temp_name;
use crate::memory::XString;
use crate::message::e_bufloaded;
use crate::message::emsg;
use crate::message_fmt::msg_bytes;
use crate::option::set_option_value_give_err;
use crate::option::split_option_part;
use crate::options::kOptSpellfile;
use crate::os::cshim::gettext;
use crate::os::env::home_replace_in;
use crate::os::fs::{mkdir_recurse, os_mkdir};
use crate::os::stdpaths::xdg_home;
use crate::path::{dir_of_file_exists, path_tail, tail_with_sep_index, vim_ispathsep};
use crate::spell::{int_wordlist, spell_enc, with_lang};
use crate::types::{MAXPATHL, OptVal, OptionSetFlags, SpellAddType};
use crate::undo::buf_is_changed;

use super::wordtree::valid_spell_word;
use super::{
    MAXWLEN, SPELL_ADD_BAD, SPELL_ADD_RARE, e_illegal_character_in_word, kXDGDataHome, mkspell,
};
use crate::optionstr::OptString;

/// Add `word` to a spell file, or take it back out again.
///
/// `what` says whether the word is good, bad (`zw`) or rare; `undo` asks
/// for the matching entry to be removed instead. `idx` selects which file:
/// zero is the session-only internal word list, and anything else is the
/// n-th entry of `'spellfile'`.
pub fn spell_add_word(word: &[u8], what: SpellAddType, idx: c_int, undo: bool) {
    if !valid_spell_word(word) {
        emsg(gettext(e_illegal_character_in_word));
        return;
    }

    // The buffer the file is open in, if the user is editing it. Held as a
    // reference: `mkspell` below runs between finding it and reloading it.
    let mut bufref = BufRef::NONE;
    let mut new_spf = false;

    // The file's name, held for the call: `mkspell` below may reload the
    // spelling state, and 'spellfile' may change under it.
    let fname: XString = if idx == 0 {
        if int_wordlist.with(Option::is_none) {
            let Some(name) = temp_name() else {
                return;
            };
            int_wordlist.set(Some(name));
        }
        int_wordlist.with(Clone::clone).expect("just set")
    } else {
        // Give 'spellfile' a sensible default if it has none.
        if Win::current().syn_block().b_p_spf.first_byte() == 0 {
            init_spellfile();
            new_spf = true;
        }
        if Win::current().syn_block().b_p_spf.first_byte() == 0 {
            semsg!("E764: Option '{}' is not set", "spellfile");
            return;
        }

        let spf = Win::current().syn_block().b_p_spf.get();
        let mut rest: &[u8] = &spf;
        let mut i = 1;
        let name = loop {
            let (part, after) = split_option_part(rest, MAXPATHL as usize, b",");
            rest = after;
            if i == idx {
                break part;
            }
            if rest.is_empty() {
                semsg!("E765: 'spellfile' does not have {} entries", idx);
                return;
            }
            i += 1;
        };
        let name = XString::from_bytes(&name);

        // Refuse to write the file behind the user's back if they are
        // editing it and have unsaved changes.
        let open = find_buffer_by_name(name.as_cstr()).filter(|b| b.b_ml.ml_mfp.is_some());
        if open.is_some_and(buf_is_changed) {
            emsg(gettext(e_bufloaded));
            return;
        }
        bufref = BufRef::of_opt(open);
        name
    };

    // Whether the last attempt to open the file succeeded. What happens
    // at the end hangs on this, and in C it was a test on the (already
    // closed) FILE pointer.
    let mut opened = false;

    if what == SPELL_ADD_BAD as SpellAddType || undo {
        // A good entry for the word sorts ahead of the banned one and
        // would win, so it has to go first.
        opened = comment_out_word(fname.as_cstr(), word, undo);
    }

    if !undo {
        let path = OsStr::from_bytes(&fname);
        let open_append = || OpenOptions::new().append(true).create(true).open(path);
        let mut fd = open_append();
        if fd.is_err() && new_spf {
            // 'spellfile' was just given its default and the file will
            // not open: the "spell" directory may not exist yet.
            // init_spellfile() already checked the parent is writable.
            let p = tail_with_sep_index(&fname);
            if !dir_of_file_exists(&fname) && p != 0 {
                os_mkdir(&cstr::owned(&fname[..p]), 0o755);
                fd = open_append();
            }
        }
        opened = fd.is_ok();

        match fd {
            Err(_) => {
                let fname = msg_bytes(&fname);
                semsg!("E484: Can't open file {fname}");
            }
            Ok(mut fd) => {
                let suffix: &[u8] = if what == SPELL_ADD_BAD as SpellAddType {
                    b"/!\n"
                } else if what == SPELL_ADD_RARE as SpellAddType {
                    b"/?\n"
                } else {
                    b"\n"
                };
                let mut entry = word.to_vec();
                entry.extend_from_slice(suffix);
                let _ = fd.write_all(&entry);
                drop(fd);

                let shown = home_replace_in(None, fname.as_cstr(), MAXPATHL as usize, true);
                let (word, shown) = (msg_bytes(word), msg_bytes(&shown));
                smsg!(0, "Word '{word}' added to {shown}");
            }
        }
    }

    if opened {
        // Compile the word list into the .add.spl beside it, so the
        // change takes effect without a full :mkspell.
        mkspell(&[&fname[..]], false, true, true);
        if let Some(buf) = bufref.get() {
            buf_reload(buf, buf.b_orig_mode, false);
        }
        redraw_all_later(UPD_SOME_VALID);
    }
}

/// Comment out every line of `fname` holding `word`, by writing a `#` over
/// its first character.
///
/// Returns whether the file could be opened at all — which is what decides
/// whether the caller recompiles it.
///
/// The lines are read as `vim_fgets` reads them, from a copy of the file;
/// each hit opens the file for update and writes the one byte, which is all
/// the reopened stream upstream wrote before it read on.
fn comment_out_word(fname: &CStr, word: &[u8], undo: bool) -> bool {
    let path = OsStr::from_bytes(fname.to_bytes());
    let Ok(text) = std::fs::read(path) else {
        return false;
    };
    let mut reader = Cursor::new(&text[..]);
    let mut line = Vec::new();

    // The offsets of the line just read and of the one after it.
    let mut fpos_next = 0;
    while Fgets::over(&mut reader).line_capped(&mut line, MAXWLEN * 2) {
        let fpos = fpos_next;
        fpos_next = reader.position();

        // The line holds the word when the flags or the line end follow it
        // directly.
        let read = cstr::until_nul(&line);
        let after = cstr::byte_at(read, word.len());
        let matched = read.starts_with(word) && (after == b'/' || after < b' ');
        if !matched {
            continue;
        }

        let Ok(mut fd) = OpenOptions::new().read(true).write(true).open(path) else {
            return false;
        };
        if fd.seek(SeekFrom::Start(fpos)).is_ok() {
            let _ = fd.write_all(b"#");
            if undo {
                let shown = home_replace_in(None, fname, MAXPATHL as usize, true);
                let (word, shown) = (msg_bytes(word), msg_bytes(&shown));
                smsg!(0, "Word '{word}' removed from {shown}");
            }
        }
    }
    true
}

/// Give `'spellfile'` a default: the user's own `spell` directory, or the
/// directory `'spelllang'` named if it named one by path, holding a
/// `.add` file named after the language and encoding in use.
fn init_spellfile() {
    if Win::current().syn_block().b_p_spl.first_byte() == 0
        || Win::current().syn_block().b_langp.is_empty()
    {
        return;
    }

    // Take the first 'spelllang' entry up to a separator. When it is a
    // path, the file goes beside it and "lstart" is its last component.
    // Upstream started "lstart" in the buffer's value and walked the
    // window's; they are one value unless the window has its own syntax.
    let spl = Win::current().syn_block().b_p_spl.get();
    let mut lstart = 0;
    let mut lend = 0;
    let mut aspath = false;
    while lend < spl.len() && !has_char(c",._", c_int::from(spl[lend])) {
        if vim_ispathsep(c_int::from(spl[lend])) {
            aspath = true;
            lstart = lend + 1;
        }
        lend += 1;
    }

    let room = MAXPATHL as usize - 1;
    let mut buf: Vec<u8> = if aspath {
        // Use the directory 'spelllang' pointed at.
        if lend >= MAXPATHL as usize {
            return;
        }
        spl[..lend].to_vec()
    } else {
        // Otherwise the user's own site directory, created if need be.
        let mut buf = xdg_home(kXDGDataHome)
            .map(|home| home.to_vec())
            .unwrap_or_default();
        buf.truncate(room);
        buf.extend_from_slice(b"/site/spell");
        buf.truncate(room);
        if mkdir_recurse(&cstr::owned(&buf), 0o755).is_err() {
            return;
        }
        buf
    };

    // "<dir>/<lang>"
    buf.push(b'/');
    buf.extend_from_slice(&spl[lstart..lend]);
    buf.truncate(room);

    // The suffix has to match the file actually loaded, which may be
    // the ASCII build of the language rather than the current encoding.
    let first = Win::current().syn_block().b_langp[0].slang;
    let ascii = with_lang(first, |slang| {
        has_bytes(path_tail(slang.sl_fname.as_cstr()), b".ascii.")
    });
    let encoding = spell_enc();
    let enc_suffix: &[u8] = if ascii == Some(true) {
        b"ascii"
    } else {
        &encoding
    };
    buf.push(b'.');
    buf.extend_from_slice(enc_suffix);
    buf.extend_from_slice(b".add");
    buf.truncate(room);

    set_option_value_give_err(
        kOptSpellfile,
        OptVal::from_cstr(&cstr::owned(&buf)),
        OptionSetFlags::LOCAL,
    );
}
