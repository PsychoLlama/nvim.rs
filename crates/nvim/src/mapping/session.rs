//! Writing mappings back out: `:mkexrc`, `:mksession`.
//!
//! [`makemap`] walks the table emitting a `:map` command per entry, splitting
//! one mapblock into up to three commands when its mode set is not one a
//! single command name can express, and [`put_escstr`] writes an LHS or RHS
//! with whatever escaping makes it read back identically.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::keycodes::ModMask;
use crate::keycodes::{Ctrl_V, KE_SNR, key_unescape};
use crate::types::MB_MAXCHAR;
use crate::types::{Failed, NUL};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_char, c_int};

/// The command-name prefix letters that reproduce each mode set a `:map`
/// command can express.
///
/// A mapping's modes can be whittled down by `:unmap`ping some of them, which
/// leaves combinations no single command name covers; those need up to three
/// commands, one per letter here.  An empty entry is the bare command --
/// plain `:map` for the four Normal-side modes, and `:map!` for the Insert
/// and Cmdline pair, which [`makemap`] tells apart by the mode itself.
/// Anything not listed is an error.
const MODE_COMMANDS: [(c_int, &[u8]); 20] = [
    (
        MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING,
        b"",
    ),
    (MODE_NORMAL, b"n"),
    (MODE_VISUAL, b"x"),
    (MODE_SELECT, b"s"),
    (MODE_OP_PENDING, b"o"),
    (MODE_NORMAL | MODE_VISUAL, b"nx"),
    (MODE_NORMAL | MODE_SELECT, b"ns"),
    (MODE_NORMAL | MODE_OP_PENDING, b"no"),
    (MODE_VISUAL | MODE_SELECT, b"v"),
    (MODE_VISUAL | MODE_OP_PENDING, b"xo"),
    (MODE_SELECT | MODE_OP_PENDING, b"so"),
    (MODE_NORMAL | MODE_VISUAL | MODE_SELECT, b"nv"),
    (MODE_NORMAL | MODE_VISUAL | MODE_OP_PENDING, b"nxo"),
    (MODE_NORMAL | MODE_SELECT | MODE_OP_PENDING, b"nso"),
    (MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING, b"vo"),
    (MODE_CMDLINE | MODE_INSERT, b""),
    (MODE_CMDLINE, b"c"),
    (MODE_INSERT, b"i"),
    (MODE_LANGMAP, b"l"),
    (MODE_TERMINAL, b"t"),
];

/// The bytes that force `'cpoptions'` to be reset around the written
/// mappings: `K_SPECIAL` and a newline both read back differently under a
/// non-default `'cpo'`.
const CPO_FORCING: [u8; 2] = [K_SPECIAL as u8, NL as u8];

/// Whether `mp` is a mapping `:mkexrc` can write out at all.
///
/// Script-local and Lua mappings are skipped, as is anything whose RHS
/// mentions `<SNR>`: none of the three would work when read back.
fn is_writable_map(mp: Mb) -> bool {
    if mp.m_noremap == REMAP_SCRIPT || mp.luaref() != LUA_NOREF {
        return false;
    }
    // A `<SNR>` in the RHS is the three-byte escape `K_SPECIAL KS_EXTRA
    // KE_SNR`; the pointer form read past the end of a two-byte tail, which
    // the windows below simply do not produce.
    !mp.rhs().windows(3).any(|at| {
        c_int::from(at[0]) == K_SPECIAL
            && c_int::from(at[1]) == KS_EXTRA
            && c_int::from(at[2] as c_char) == KE_SNR as c_int
    })
}

/// The session or `.exrc` file being written, whose caller has promised the
/// stream stays open for as long as the value is used.
///
/// Every method answers `false` on a write error, which is what upstream's
/// `< 0` tests come to.  The promise is discharged by the `:mksession` /
/// `:mkexrc` frame that opened the file: it outlives every write made out of
/// it.  Construction is the unsafe step, once; every write after it is
/// ordinary checked code, which is what makes this worth a newtype.
#[derive(Copy, Clone)]
struct Out(*mut FILE);

impl Out {
    /// # Safety
    /// `fd` must stay an open stream for as long as the value is used.
    unsafe fn new(fd: *mut FILE) -> Self {
        Self(fd)
    }

    /// Write a NUL-terminated string.
    fn puts(self, s: &CStr) -> bool {
        // SAFETY: the constructor's promise — an open stream — and `s` is
        // NUL-terminated by its type.
        unsafe { fputs(s.as_ptr(), self.0) >= 0 }
    }

    /// Write a literal through `fprintf`, as upstream does.
    fn printf(self, s: &CStr) -> bool {
        // SAFETY: as [`Out::puts`]; every caller passes a literal with no
        // conversion in it, so there are no arguments to check.
        unsafe { fprintf(self.0, s.as_ptr()) >= 0 }
    }

    /// Write one byte through `putc`.
    fn putc(self, c: c_int) -> bool {
        // SAFETY: as [`Out::puts`].
        unsafe { putc(c, self.0) >= 0 }
    }

    /// Write one byte through `fputc`.
    fn fputc(self, c: c_int) -> bool {
        // SAFETY: as [`Out::puts`].
        unsafe { fputc(c, self.0) >= 0 }
    }

    /// End the line, honouring `'sessionoptions'`'s line-ending choice.
    ///
    /// The one method that never answers `false`: see
    /// [`put_eol_unchecked`] for the upstream check this preserves.
    fn eol(self) -> bool {
        // SAFETY: as [`Out::puts`].
        unsafe { put_eol_unchecked(self.0) }
    }
}

/// Write map commands for the current mappings to an `.exrc` file.
///
/// `buf` names the buffer whose local mappings to write, or is `None` for
/// the global ones.  Answers `Err` on a write error.
///
/// # Safety
/// `fd` must be an open stream.
pub unsafe fn makemap(fd: *mut FILE, buf: Option<Buf>) -> Result<(), Failed> {
    // SAFETY: the caller's promise — `fd` is an open stream for the whole of
    // this body.
    let out = unsafe { Out::new(fd) };
    let mut did_cpo = false;
    let table = match buf {
        Some(buf) => MapTable::Buffer(buf),
        None => MapTable::Global,
    };

    // Do the loop twice: once for mappings, once for abbreviations.
    for abbr in [false, true] {
        let write = |mp: Mb| {
            if !is_writable_map(mp) {
                return None;
            }

            // It is possible to create a mapping and then `:unmap`
            // certain modes; that is recreated here by mapping the
            // individual modes, which takes up to three commands.
            let bang = mp.m_mode == MODE_CMDLINE | MODE_INSERT;
            let cmd = if abbr {
                c"abbr"
            } else if bang {
                c"map!"
            } else {
                c"map"
            };
            let Some(&(_, letters)) = MODE_COMMANDS.iter().find(|(mode, _)| *mode == mp.m_mode)
            else {
                iemsg(gettext(c"E228: makemap: Illegal mode"));
                return Some(Failed);
            };

            let mut letters = letters.iter().copied();
            let mut c1 = letters.next().unwrap_or(0);
            loop {
                // When writing the <> form, 'cpo' has to be the Vim
                // default; say so once, the first time it can matter.
                if !did_cpo {
                    let forcing = |bytes: &[u8]| bytes.iter().any(|b| CPO_FORCING.contains(b));
                    if mp.rhs().is_empty() {
                        did_cpo = true; // will use <Nop>
                    } else if forcing(mp.rhs()) || forcing(mp.keys()) {
                        did_cpo = true;
                    }
                    if did_cpo
                        && !(out.printf(c"let s:cpo_save=&cpo")
                            && out.eol()
                            && out.printf(c"set cpo&vim")
                            && out.eol())
                    {
                        return Some(Failed);
                    }
                }

                if c1 != 0 && !out.putc(c_int::from(c1)) {
                    return Some(Failed);
                }
                if mp.m_noremap != REMAP_YES && !out.printf(c"nore") {
                    return Some(Failed);
                }
                if !out.puts(cmd) {
                    return Some(Failed);
                }
                if buf.is_some() && !out.puts(c" <buffer>") {
                    return Some(Failed);
                }
                if mp.m_nowait && !out.puts(c" <nowait>") {
                    return Some(Failed);
                }
                if mp.m_silent && !out.puts(c" <silent>") {
                    return Some(Failed);
                }
                if mp.m_expr && !out.puts(c" <expr>") {
                    return Some(Failed);
                }

                // SAFETY: `m_keys` and `m_str` are the mapping's own
                // NUL-terminated strings and `out` an open stream.
                let wrote = unsafe {
                    out.putc(c_int::from(b' '))
                        && put_escstr(fd, mp.m_keys.as_ptr(), EscTarget::MapLhs).is_ok()
                        && out.putc(c_int::from(b' '))
                        && put_escstr(fd, mp.m_rhs.str.as_ptr(), EscTarget::MapRhs).is_ok()
                        && out.eol()
                };
                if !wrote {
                    return Some(Failed);
                }

                c1 = letters.next().unwrap_or(0);
                if c1 == 0 {
                    break;
                }
            }
            None
        };
        // SAFETY: the tables are live, and `write` neither unlinks nor frees
        // an entry.
        if let Some(failed) = unsafe { map_walk(table, abbr, write) } {
            return Err(failed);
        }
    }

    if did_cpo
        && !(out.printf(c"let &cpo=s:cpo_save")
            && out.eol()
            && out.printf(c"unlet s:cpo_save")
            && out.eol())
    {
        return Err(Failed);
    }
    Ok(())
}

/// What [`put_escstr`] is writing, which decides how much escaping it needs.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EscTarget {
    /// The `{lhs}` of a `:map` command, where a space needs a CTRL-V.
    MapLhs,
    /// The `{rhs}` of a `:map` command, where only a *leading* space does.
    MapRhs,
    /// A `:set` value, which escapes with backslashes rather than CTRL-Vs and
    /// leaves `K_SPECIAL` bytes alone.
    SetValue,
}

/// Write `strstart` to `fd` with the escaping `what` calls for.
///
/// Answers `Err` on a write error.
///
/// # Safety
/// `fd` must be an open stream and `strstart` live and NUL-terminated.
pub unsafe fn put_escstr(
    fd: *mut FILE,
    strstart: *const c_char,
    what: EscTarget,
) -> Result<(), Failed> {
    let mut ch = [0 as c_char; MB_MAXCHAR];
    // SAFETY: the caller's promise -- `fd` is an open stream.
    let out = unsafe { Out::new(fd) };
    let start = strstart.cast::<u8>().cast_mut();
    let mut str = start;

    // :map xx <Nop>
    // SAFETY: the caller's promise -- `strstart` is live and NUL-terminated.
    if unsafe { c_int::from(*str) } == NUL && what == EscTarget::MapRhs {
        if !out.printf(c"<Nop>") {
            return Err(Failed);
        }
        return Ok(());
    }

    loop {
        // SAFETY: as above; the walk stops at the NUL.
        if unsafe { c_int::from(*str) } == NUL {
            break;
        }
        // Check for a multi-byte character, which may contain escaped
        // K_SPECIAL bytes.
        // SAFETY: `mb_unescape` reads `str`'s NUL-terminated tail and steps
        // `str` past whatever it consumed.
        let mut p = unsafe { mb_unescape((&raw mut str).cast(), &mut ch) };
        'next: {
            if !p.is_null() {
                loop {
                    // SAFETY: `p` is `mb_unescape`'s NUL-terminated answer.
                    let c = unsafe { c_int::from(*p) };
                    if c == NUL {
                        break;
                    }
                    if !out.fputc(c) {
                        return Err(Failed);
                    }
                    // SAFETY: as above.
                    p = unsafe { p.add(1) };
                }
                // SAFETY: `mb_unescape` stepped `str` past at least one byte.
                str = unsafe { str.sub(1) };
                break 'next;
            }

            // SAFETY: `str` is on a non-NUL byte of the caller's string.
            let mut c = unsafe { c_int::from(*str) };
            // Special key codes have to be translated to make sense when
            // they are read back.
            if c == K_SPECIAL && what != EscTarget::SetValue {
                let mut modifiers = ModMask::NONE;
                // SAFETY: a `K_SPECIAL` escape is three bytes long, so
                // `str[1]` and `str[2]` are inside the same string.
                if unsafe { c_int::from(*str.add(1)) } == KS_MODIFIER {
                    // SAFETY: as above.
                    unsafe {
                        modifiers = ModMask::from_bits(c_int::from(*str.add(2)));
                        str = str.add(3);
                    }

                    // Modifiers can apply to multi-byte characters too.
                    // SAFETY: as the first `mb_unescape` above.
                    p = unsafe { mb_unescape((&raw mut str).cast(), &mut ch) };
                    if p.is_null() {
                        // SAFETY: `str` is still inside the caller's string.
                        c = unsafe { c_int::from(*str) };
                    } else {
                        // Retrieve the codepoint from the unescaped text.
                        // SAFETY: `p` is `mb_unescape`'s NUL-terminated answer.
                        c = unsafe { utf_ptr2char(p) };
                        // SAFETY: as above.
                        str = unsafe { str.sub(1) };
                    }
                }
                if c == K_SPECIAL {
                    // SAFETY: as above -- the escape's two trailing bytes.
                    unsafe {
                        c = key_unescape(*str.add(1), *str.add(2));
                        str = str.add(2);
                    }
                }
                if c < 0 || !modifiers.is_empty() {
                    // A special key.
                    let name = get_special_key_name(c, modifiers);
                    // SAFETY: a NUL-terminated rendering that outlives the
                    // call, written to the open stream.
                    if unsafe { fputs(name.as_ptr(), fd) } < 0 {
                        return Err(Failed);
                    }
                    break 'next;
                }
            }

            // A '\n' in a map command is written as <NL>; in a set
            // command as \^V^J.
            if c == NL {
                let form = if what == EscTarget::SetValue {
                    c"\\\x16\n"
                } else {
                    c"<NL>"
                };
                if !out.printf(form) {
                    return Err(Failed);
                }
                break 'next;
            }

            // Some characters have to be escaped with CTRL-V to keep
            // DoOneCmd() from misreading them; a space, Tab or '"' needs
            // a backslash to keep do_set() from misreading it. A '<'
            // needs a CTRL-V or it starts a special key name, a space
            // needs one in a :map lhs, and in a :map rhs one does only at
            // the very start.
            if what == EscTarget::SetValue
                && (ascii_iswhite(c) || c == c_int::from(b'"') || c == c_int::from(b'\\'))
            {
                if !out.putc(c_int::from(b'\\')) {
                    return Err(Failed);
                }
            } else if (!(c_int::from(b' ')..=c_int::from(b'~')).contains(&c)
                || c == c_int::from(b'|')
                || (what == EscTarget::MapLhs && c == c_int::from(b' '))
                || (what == EscTarget::MapRhs && str == start && c == c_int::from(b' '))
                || (what != EscTarget::SetValue && c == c_int::from(b'<')))
                && !out.putc(Ctrl_V)
            {
                return Err(Failed);
            }
            if !out.putc(c) {
                return Err(Failed);
            }
        }
        // SAFETY: `str` is on a non-NUL byte, so the next one is in the
        // caller's string.
        str = unsafe { str.add(1) };
    }
    Ok(())
}
