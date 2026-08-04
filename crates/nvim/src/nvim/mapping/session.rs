//! Writing mappings back out: `:mkexrc`, `:mksession`.
//!
//! [`makemap`] walks the table emitting a `:map` command per entry, splitting
//! one mapblock into up to three commands when its mode set is not one a
//! single command name can express, and [`put_escstr`] writes an LHS or RHS
//! with whatever escaping makes it read back identically.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::keycodes::key_unescape;
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
const CPO_FORCING: [c_char; 3] = [K_SPECIAL as u8 as c_char, NL as c_char, NUL as c_char];

/// Whether `mp` is a mapping `:mkexrc` can write out at all.
///
/// Script-local and Lua mappings are skipped, as is anything whose RHS
/// mentions `<SNR>`: none of the three would work when read back.
///
/// # Safety
/// `mp` must be a live mapblock.
unsafe fn is_writable_map(mp: *mut mapblock_T) -> bool {
    unsafe {
        if (*mp).m_noremap == REMAP_SCRIPT || (*mp).m_luaref != LUA_NOREF {
            return false;
        }
        let mut p = (*mp).m_str;
        while c_int::from(*p) != NUL {
            if c_int::from(*p as u8) == K_SPECIAL
                && c_int::from(*p.add(1) as u8) == KS_EXTRA
                && c_int::from(*p.add(2)) == KE_SNR as c_int
            {
                return false;
            }
            p = p.add(1);
        }
        true
    }
}

/// Write map commands for the current mappings to an `.exrc` file.
///
/// `buf` names the buffer whose local mappings to write, or is null for the
/// global ones.  Answers `FAIL` on a write error, `OK` otherwise.
///
/// # Safety
/// `fd` must be an open stream and `buf` null or a live buffer.
pub unsafe fn makemap(fd: *mut FILE, buf: *mut buf_T) -> c_int {
    unsafe {
        let mut did_cpo = false;
        let table = if buf.is_null() {
            MapTable::Global
        } else {
            MapTable::Buffer(buf)
        };

        // Do the loop twice: once for mappings, once for abbreviations.
        for abbr in [false, true] {
            let failed = map_walk(table, abbr, |mp| {
                if !is_writable_map(mp) {
                    return None;
                }

                // It is possible to create a mapping and then `:unmap`
                // certain modes; that is recreated here by mapping the
                // individual modes, which takes up to three commands.
                let bang = (*mp).m_mode == MODE_CMDLINE | MODE_INSERT;
                let cmd = if abbr {
                    c"abbr"
                } else if bang {
                    c"map!"
                } else {
                    c"map"
                };
                let Some(&(_, letters)) =
                    MODE_COMMANDS.iter().find(|(mode, _)| *mode == (*mp).m_mode)
                else {
                    iemsg(gettext(c"E228: makemap: Illegal mode".as_ptr()));
                    return Some(FAIL);
                };

                let put = |s: &CStr| fputs(s.as_ptr(), fd) >= 0;
                let mut letters = letters.iter().copied();
                let mut c1 = letters.next().unwrap_or(0);
                loop {
                    // When writing the <> form, 'cpo' has to be the Vim
                    // default; say so once, the first time it can matter.
                    if !did_cpo {
                        if c_int::from(*(*mp).m_str) == NUL {
                            did_cpo = true; // will use <Nop>
                        } else if !strpbrk((*mp).m_str, CPO_FORCING.as_ptr()).is_null()
                            || !strpbrk((*mp).m_keys, CPO_FORCING.as_ptr()).is_null()
                        {
                            did_cpo = true;
                        }
                        if did_cpo
                            && (fprintf(fd, c"let s:cpo_save=&cpo".as_ptr()) < 0
                                || put_eol(fd) < 0
                                || fprintf(fd, c"set cpo&vim".as_ptr()) < 0
                                || put_eol(fd) < 0)
                        {
                            return Some(FAIL);
                        }
                    }

                    if c1 != 0 && putc(c_int::from(c1), fd) < 0 {
                        return Some(FAIL);
                    }
                    if (*mp).m_noremap != REMAP_YES && fprintf(fd, c"nore".as_ptr()) < 0 {
                        return Some(FAIL);
                    }
                    if !put(cmd) {
                        return Some(FAIL);
                    }
                    if !buf.is_null() && !put(c" <buffer>") {
                        return Some(FAIL);
                    }
                    if (*mp).m_nowait != 0 && !put(c" <nowait>") {
                        return Some(FAIL);
                    }
                    if (*mp).m_silent != 0 && !put(c" <silent>") {
                        return Some(FAIL);
                    }
                    if (*mp).m_expr != 0 && !put(c" <expr>") {
                        return Some(FAIL);
                    }

                    if putc(c_int::from(b' '), fd) < 0
                        || put_escstr(fd, (*mp).m_keys, EscTarget::MapLhs) == FAIL
                        || putc(c_int::from(b' '), fd) < 0
                        || put_escstr(fd, (*mp).m_str, EscTarget::MapRhs) == FAIL
                        || put_eol(fd) < 0
                    {
                        return Some(FAIL);
                    }

                    c1 = letters.next().unwrap_or(0);
                    if c1 == 0 {
                        break;
                    }
                }
                None
            });
            if failed.is_some() {
                return FAIL;
            }
        }

        if did_cpo
            && (fprintf(fd, c"let &cpo=s:cpo_save".as_ptr()) < 0
                || put_eol(fd) < 0
                || fprintf(fd, c"unlet s:cpo_save".as_ptr()) < 0
                || put_eol(fd) < 0)
        {
            return FAIL;
        }
        OK
    }
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
/// Answers `FAIL` on a write error, `OK` otherwise.
///
/// # Safety
/// `fd` must be an open stream and `strstart` live and NUL-terminated.
pub unsafe fn put_escstr(fd: *mut FILE, strstart: *const c_char, what: EscTarget) -> c_int {
    unsafe {
        let mut str = strstart.cast::<u8>().cast_mut();

        // :map xx <Nop>
        if c_int::from(*str) == NUL && what == EscTarget::MapRhs {
            if fprintf(fd, c"<Nop>".as_ptr()) < 0 {
                return FAIL;
            }
            return OK;
        }

        while c_int::from(*str) != NUL {
            // Check for a multi-byte character, which may contain escaped
            // K_SPECIAL bytes.
            let mut p = mb_unescape((&raw mut str).cast());
            'next: {
                if !p.is_null() {
                    while c_int::from(*p) != NUL {
                        if fputc(c_int::from(*p), fd) < 0 {
                            return FAIL;
                        }
                        p = p.add(1);
                    }
                    str = str.sub(1);
                    break 'next;
                }

                let mut c = c_int::from(*str);
                // Special key codes have to be translated to make sense when
                // they are read back.
                if c == K_SPECIAL && what != EscTarget::SetValue {
                    let mut modifiers = 0;
                    if c_int::from(*str.add(1)) == KS_MODIFIER {
                        modifiers = c_int::from(*str.add(2));
                        str = str.add(3);

                        // Modifiers can apply to multi-byte characters too.
                        p = mb_unescape((&raw mut str).cast());
                        if p.is_null() {
                            c = c_int::from(*str);
                        } else {
                            // Retrieve the codepoint from the unescaped text.
                            c = utf_ptr2char(p);
                            str = str.sub(1);
                        }
                    }
                    if c == K_SPECIAL {
                        c = key_unescape(*str.add(1), *str.add(2));
                        str = str.add(2);
                    }
                    if c < 0 || modifiers != 0 {
                        // A special key.
                        if fputs(get_special_key_name(c, modifiers), fd) < 0 {
                            return FAIL;
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
                    if fprintf(fd, form.as_ptr()) < 0 {
                        return FAIL;
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
                    if putc(c_int::from(b'\\'), fd) < 0 {
                        return FAIL;
                    }
                } else if !(c_int::from(b' ')..=c_int::from(b'~')).contains(&c)
                    || c == c_int::from(b'|')
                    || (what == EscTarget::MapLhs && c == c_int::from(b' '))
                    || (what == EscTarget::MapRhs
                        && str == strstart.cast::<u8>().cast_mut()
                        && c == c_int::from(b' '))
                    || (what != EscTarget::SetValue && c == c_int::from(b'<'))
                {
                    if putc(Ctrl_V, fd) < 0 {
                        return FAIL;
                    }
                }
                if putc(c, fd) < 0 {
                    return FAIL;
                }
            }
            str = str.add(1);
        }
        OK
    }
}
