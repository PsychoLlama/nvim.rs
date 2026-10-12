//! Writing mappings back out: `:mkexrc`, `:mksession`.
//!
//! [`makemap`] walks the table emitting a `:map` command per entry, splitting
//! one mapblock into up to three commands when its mode set is not one a
//! single command name can express, and [`put_escstr`] writes an LHS or RHS
//! with whatever escaping makes it read back identically.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cstr;
use crate::keycodes::ModMask;
use crate::keycodes::{Ctrl_V, KE_SNR, key_unescape};
use crate::mbyte::{char_at, unescape_at};
use crate::types::Failed;
use crate::winlayer::Buf;
use core::ffi::c_int;
use std::io::Write;

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

/// Whether `b` forces `'cpoptions'` to be reset around the written
/// mappings: `K_SPECIAL` and a newline both read back differently under a
/// non-default `'cpo'`.
fn is_cpo_forcing(b: u8) -> bool {
    c_int::from(b) == K_SPECIAL || c_int::from(b) == NL
}

/// Whether `mp` is a mapping `:mkexrc` can write out at all.
///
/// Script-local and Lua mappings are skipped, as is anything whose RHS
/// mentions `<SNR>`: none of the three would work when read back.
fn is_writable_map(mp: &MapBlock) -> bool {
    if mp.m_noremap == REMAP_SCRIPT || mp.luaref() != LUA_NOREF {
        return false;
    }
    // A `<SNR>` in the RHS is the three-byte escape `K_SPECIAL KS_EXTRA
    // KE_SNR`; the pointer form read past the end of a two-byte tail, which
    // the windows below simply do not produce.
    !mp.rhs().windows(3).any(|at| {
        c_int::from(at[0]) == K_SPECIAL
            && c_int::from(at[1]) == KS_EXTRA
            && c_int::from(at[2].cast_signed()) == KE_SNR.cast_signed()
    })
}

/// The `:map` commands that recreate `mp`, appended to `out` a line each.
///
/// It is possible to create a mapping and then `:unmap` certain modes; that
/// is recreated here by mapping the individual modes, which takes up to
/// three commands.  `did_cpo` says whether the `'cpo'` reset has been written
/// yet.  Answers `Err` for a mode set no command can express.
fn put_map(
    out: &mut Vec<Vec<u8>>,
    mp: &MapBlock,
    abbr: bool,
    local: bool,
    did_cpo: &mut bool,
) -> Result<(), Failed> {
    let bang = mp.m_mode == MODE_CMDLINE | MODE_INSERT;
    let cmd: &[u8] = if abbr {
        b"abbr"
    } else if bang {
        b"map!"
    } else {
        b"map"
    };
    let Some(&(_, letters)) = MODE_COMMANDS.iter().find(|(mode, _)| *mode == mp.m_mode) else {
        return Err(Failed);
    };

    let mut letters = letters.iter().copied();
    let mut c1 = letters.next().unwrap_or(0);
    loop {
        // When writing the <> form, 'cpo' has to be the Vim default; say so
        // once, the first time it can matter.
        if !*did_cpo {
            let forcing = |bytes: &[u8]| bytes.iter().copied().any(is_cpo_forcing);
            if mp.rhs().is_empty() || forcing(mp.rhs()) || forcing(mp.keys()) {
                *did_cpo = true; // an empty RHS will use <Nop>
                out.push(b"let s:cpo_save=&cpo".to_vec());
                out.push(b"set cpo&vim".to_vec());
            }
        }

        let mut line = Vec::new();
        if c1 != 0 {
            line.push(c1);
        }
        if mp.m_noremap != REMAP_YES {
            line.extend_from_slice(b"nore");
        }
        line.extend_from_slice(cmd);
        if local {
            line.extend_from_slice(b" <buffer>");
        }
        if mp.m_nowait {
            line.extend_from_slice(b" <nowait>");
        }
        if mp.m_silent {
            line.extend_from_slice(b" <silent>");
        }
        if mp.m_expr {
            line.extend_from_slice(b" <expr>");
        }

        line.push(b' ');
        put_escstr(&mut line, mp.keys(), EscTarget::MapLhs);
        line.push(b' ');
        put_escstr(&mut line, mp.rhs(), EscTarget::MapRhs);
        out.push(line);

        c1 = letters.next().unwrap_or(0);
        if c1 == 0 {
            break;
        }
    }
    Ok(())
}

/// Write map commands for the current mappings to an `.exrc` file.
///
/// `buf` names the buffer whose local mappings to write, or is `None` for
/// the global ones.  Answers `Err` on a write error.
///
/// The commands are rendered first, out of the table's borrow, and then
/// written a line at a time.  A newline's write error is not reported:
/// upstream's `put_eol(fd) < 0` never fires.
pub fn makemap(out: &mut dyn Write, buffer: Option<Buf>) -> Result<(), Failed> {
    let mut text = Vec::new();
    let mut did_cpo = false;
    let table = match buffer {
        Some(buf) => MapTable::Buffer(buf),
        None => MapTable::Global,
    };

    // Do the loop twice: once for mappings, once for abbreviations.
    let mut illegal = false;
    for abbr in [false, true] {
        illegal = table.with(|tables| {
            tables
                .walk(abbr)
                .filter(|mp| is_writable_map(mp))
                .any(|mp| put_map(&mut text, mp, abbr, buffer.is_some(), &mut did_cpo).is_err())
        });
        if illegal {
            break;
        }
    }

    if !illegal && did_cpo {
        text.push(b"let &cpo=s:cpo_save".to_vec());
        text.push(b"unlet s:cpo_save".to_vec());
    }
    for line in &text {
        out.write_all(line).map_err(|_| Failed)?;
        let _ = out.write_all(b"\n");
    }
    if illegal {
        iemsg(gettext(c"E228: makemap: Illegal mode"));
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

/// Append `text` to `out` with the escaping `what` calls for.
pub(crate) fn put_escstr(out: &mut Vec<u8>, text: &[u8], what: EscTarget) {
    // :map xx <Nop>
    if text.is_empty() && what == EscTarget::MapRhs {
        out.extend_from_slice(b"<Nop>");
        return;
    }

    let at = |i: usize| cstr::byte_at(text, i);
    let mut i = 0;
    while i < text.len() {
        // Check for a multi-byte character, which may contain escaped
        // K_SPECIAL bytes.
        if let Some((bytes, used)) = unescape_at(&text[i..]) {
            out.extend_from_slice(cstr::in_bytes(&bytes).to_bytes());
            i += used;
            continue;
        }

        let mut c = c_int::from(text[i]);
        // Special key codes have to be translated to make sense when they
        // are read back.
        if c == K_SPECIAL && what != EscTarget::SetValue {
            let mut modifiers = ModMask::NONE;
            // A `K_SPECIAL` escape is three bytes long.
            if c_int::from(at(i + 1)) == KS_MODIFIER {
                modifiers = ModMask::from_bits(c_int::from(at(i + 2)));
                i += 3;

                // Modifiers can apply to multi-byte characters too.
                match unescape_at(&text[i.min(text.len())..]) {
                    None => c = c_int::from(at(i)),
                    Some((bytes, used)) => {
                        // Retrieve the codepoint from the unescaped text.
                        c = char_at(cstr::in_bytes(&bytes).to_bytes());
                        i += used - 1;
                    }
                }
            }
            if c == K_SPECIAL {
                // The escape's two trailing bytes.
                c = key_unescape(at(i + 1), at(i + 2));
                i += 2;
            }
            if c < 0 || !modifiers.is_empty() {
                // A special key.
                let name = get_special_key_name(c, modifiers);
                out.extend_from_slice(cstr::in_chars(&name).to_bytes());
                i += 1;
                continue;
            }
        }

        // A '\n' in a map command is written as <NL>; in a set command as
        // \^V^J.
        if c == NL {
            out.extend_from_slice(if what == EscTarget::SetValue {
                b"\\\x16\n"
            } else {
                b"<NL>"
            });
            i += 1;
            continue;
        }

        // Some characters have to be escaped with CTRL-V to keep
        // DoOneCmd() from misreading them; a space, Tab or '"' needs a
        // backslash to keep do_set() from misreading it. A '<' needs a
        // CTRL-V or it starts a special key name, a space needs one in a
        // :map lhs, and in a :map rhs one does only at the very start.
        if what == EscTarget::SetValue
            && (ascii_iswhite(c) || c == c_int::from(b'"') || c == c_int::from(b'\\'))
        {
            out.push(b'\\');
        } else if !(c_int::from(b' ')..=c_int::from(b'~')).contains(&c)
            || c == c_int::from(b'|')
            || (what == EscTarget::MapLhs && c == c_int::from(b' '))
            || (what == EscTarget::MapRhs && i == 0 && c == c_int::from(b' '))
            || (what != EscTarget::SetValue && c == c_int::from(b'<'))
        {
            out.push(Ctrl_V.to_le_bytes()[0]);
        }
        // `putc` writes the low byte.
        out.push(c.to_le_bytes()[0]);
        i += 1;
    }
}
