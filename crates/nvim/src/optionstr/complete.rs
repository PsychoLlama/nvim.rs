//! The callbacks for the completion, spelling and tag options.
//!
//! They are `pub` only so the generated option table can name them; see
//! [`super::frame`] for what they are handed.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int, c_uint};

use crate::eval::userfunc::scriptlocal_funcname;
use crate::insexpand::set_cpt_callbacks;
use crate::option::split_option_part;
use crate::option::vars::{P_CIA, P_HLG, cia_flags, cot_flags, spo_flags, tc_flags};
use crate::options::{opt_cot_values, opt_spo_values, opt_tc_values};
use crate::os::env::vim_unsetenv_named;
use crate::os::state::{didset_vim, didset_vimruntime};
use crate::spell::{compile_cap_prog, did_set_spell_option, valid_spellfile, valid_spelllang};
use crate::spellfile::spell_check_msm;
use crate::spellsuggest::spell_check_sps;
use crate::types::{OptError, OptSet, OptionSetFlags};

use super::frame::{invalid, new_bytes, new_value, varp, win};
use super::{
    CPT_ABBR, CPT_KIND, CPT_MENU, LSIZE, illegal_char, illegal_char_after_chr, opt_strings_mask,
};

/// The sources 'complete' accepts, one letter each.
const CPT_SOURCES: &CStr = c".wbuksid]tUfFo";

/// The three sources that take an argument after their letter, so that a
/// second character is part of the value rather than a mistake.
const CPT_WITH_ARGUMENT: &CStr = c"ksF";

/// Check 'complete', a comma-separated list of one-letter sources, some of
/// them followed by an argument and any of them by `^<count>`.
///
/// The parts are copied out one at a time because a comma may be escaped
/// with a backslash — and because a part longer than the scratch buffer is
/// simply cut there and the remainder checked as if it were the next part,
/// which is upstream's behaviour and is preserved.
pub fn did_set_complete(args: &mut OptSet) -> Result<(), OptError> {
    let value = new_bytes(args);
    // Reading past the end answers the terminator, as walking the C string
    // does.
    let at = |i: usize| value.get(i).copied().unwrap_or(0);
    let mut p = 0;
    while at(p) != 0 {
        let mut part = Vec::with_capacity(LSIZE as usize);
        let mut escaped = false;
        while at(p) != 0 && (at(p) != b',' || escaped) && part.len() < LSIZE as usize - 1 {
            if at(p) == b'\\' && at(p + 1) == b',' {
                escaped = true;
                p += 1;
            } else {
                escaped = false;
                part.push(at(p));
            }
            p += 1;
        }

        let source = part.first().copied().unwrap_or(0);
        if source == 0 || !CPT_SOURCES.to_bytes().contains(&source) {
            return Err(illegal_char(c_int::from(source)));
        }

        // Anything after the source letter is either that source's
        // argument or a `^<count>`; anything else names a character the
        // source does not take.
        let takes_argument = CPT_WITH_ARGUMENT.to_bytes().contains(&source);
        let char_before = if !takes_argument && part.len() > 1 && part[1] != b'^' {
            Some(source)
        } else {
            match part.iter().position(|&b| b == b'^') {
                // A `^` has to be followed by a count, and by nothing
                // else.
                Some(at) => {
                    let count = &part[at + 1..];
                    (count.is_empty() || !count.iter().all(u8::is_ascii_digit)).then_some(b'^')
                }
                None => None,
            }
        };
        if let Some(char_before) = char_before {
            return Err(illegal_char_after_chr(c_int::from(char_before)));
        }

        while at(p) == b',' || at(p) == b' ' {
            p += 1;
        }
    }

    // The "F" source names a function, which is resolved last because
    // it can fail for a reason the letter walk cannot see.
    let local = args.os_flags.has(OptionSetFlags::LOCAL);
    if set_cpt_callbacks(local).is_err() {
        return Err(illegal_char_after_chr(c_int::from(b'F')));
    }
    Ok(())
}

/// 'completeitemalign' is the three completion-menu columns in the order
/// they are drawn: each exactly once, none of them missing.
///
/// The order is kept as a base-10 number, one digit per column, which is
/// what the menu drawing code reads.
pub fn did_set_completeitemalign(_args: &mut OptSet) -> Result<(), OptError> {
    // Each column's position here is its digit.
    const COLUMNS: [(&CStr, c_uint); 3] = [
        (c"abbr", CPT_ABBR),
        (c"kind", CPT_KIND),
        (c"menu", CPT_MENU),
    ];
    const _: () = assert!(CPT_ABBR == 0 && CPT_KIND == 1 && CPT_MENU == 2);
    // Upstream copies each part into a ten-byte buffer, so a longer one is
    // cut to nine bytes before it is compared.
    const PART_ROOM: usize = 10;
    let mut order: c_uint = 0;
    let mut seen = [false; 3];
    let mut count = 0;

    // A copy: the walk below outlives a projection's borrow.
    let cia = P_CIA.get();
    let mut rest: &[u8] = &cia;
    while !rest.is_empty() {
        let (column, after) = split_option_part(rest, PART_ROOM, b",");
        rest = after;
        if count >= COLUMNS.len() {
            return invalid();
        }
        let Some(which) = COLUMNS
            .iter()
            .position(|(name, _)| name.to_bytes() == column.as_slice())
        else {
            return invalid();
        };
        if seen[which] {
            return invalid();
        }
        seen[which] = true;
        order = order * 10 + COLUMNS[which].1;
        count += 1;
    }
    // "abbr" alone would leave the order at 0, which is also "nothing was
    // named"; the count is what rules that out.
    if order == 0 || count != COLUMNS.len() {
        return invalid();
    }
    cia_flags.set(order);
    Ok(())
}

pub fn did_set_completeopt(args: &mut OptSet) -> Result<(), OptError> {
    let (mut buf, opt_flags) = (args.os_buf, args.os_flags);
    let local = opt_flags.has(OptionSetFlags::LOCAL);
    // The variable being set is the buffer's own copy for `:setlocal` and
    // the global one otherwise -- which is the value the frame holds.
    if !local && !opt_flags.has(OptionSetFlags::GLOBAL) {
        // A plain `:set` drops the buffer's own answer.
        buf.b_cot_flags = 0 as c_uint;
    }
    let Some(mask) = opt_strings_mask(new_bytes(args), &opt_cot_values, true) else {
        return invalid();
    };
    if local {
        buf.b_cot_flags = mask;
    } else {
        cot_flags.set(mask);
    }
    Ok(())
}

/// A 'helpfile' the user chose overrides `$VIM`/`$VIMRUNTIME`, so the ones
/// nvim derived for itself are dropped and re-derived from it.
pub fn did_set_helpfile(_args: &mut OptSet) -> Result<(), OptError> {
    if didset_vim.get() {
        vim_unsetenv_named(c"VIM");
    }
    if didset_vimruntime.get() {
        vim_unsetenv_named(c"VIMRUNTIME");
    }
    Ok(())
}

/// 'helplang' is a comma-separated list of two-letter language codes, which
/// is checked by position rather than by parsing.
pub fn did_set_helplang(_args: &mut OptSet) -> Result<(), OptError> {
    // A copy: the walk below outlives a projection's borrow.
    let hlg = P_HLG.get();
    // Reading past the end answers the terminator; each test below is
    // reached only once the byte before it is known not to be it, as
    // walking the C string does.
    let at = |i: usize| hlg.get(i).copied().unwrap_or(0);
    let mut s = 0;
    while at(s) != 0 {
        if at(s + 1) == 0 || ((at(s + 2) != b',' || at(s + 3) == 0) && at(s + 2) != 0) {
            return invalid();
        }
        if at(s + 2) == 0 {
            break;
        }
        s += 3;
    }
    Ok(())
}

pub fn did_set_mkspellmem(_args: &mut OptSet) -> Result<(), OptError> {
    if spell_check_msm().is_err() {
        return invalid();
    }
    Ok(())
}

/// The callback for every option holding an expression or a function name
/// ('foldexpr', 'formatexpr', 'completefunc', …).
///
/// A `s:`-prefixed name is resolved to its script-local spelling now, while
/// the script that set the option is still on the stack; the option's value
/// is rewritten in place with the answer.
pub fn did_set_optexpr(args: &mut OptSet) -> Result<(), OptError> {
    if let Some(resolved) = scriptlocal_funcname(new_bytes(args)) {
        varp(args).set(resolved);
    }
    Ok(())
}

pub fn did_set_spellcapcheck(args: &mut OptSet) -> Result<(), OptError> {
    compile_cap_prog(&mut win(args).syntax())
}

pub fn did_set_spellfile(args: &mut OptSet) -> Result<(), OptError> {
    if !valid_spellfile(new_bytes(args)) {
        return invalid();
    }
    did_set_spell_option()
}

pub fn did_set_spelllang(args: &mut OptSet) -> Result<(), OptError> {
    if !valid_spelllang(new_value(args).as_cstr()) {
        return invalid();
    }
    did_set_spell_option()
}

/// 'spelloptions' keeps a mask at both scopes, and a `:set` writes both.
///
/// The window's mask lives in its *syntax block*, which a diff or preview
/// window may share with another window.
pub fn did_set_spelloptions(args: &mut OptSet) -> Result<(), OptError> {
    let (wp, opt_flags) = (win(args), args.os_flags);
    let global = !opt_flags.has(OptionSetFlags::LOCAL);
    let local = !opt_flags.has(OptionSetFlags::GLOBAL);
    if global || local {
        let Some(mask) = opt_strings_mask(new_bytes(args), &opt_spo_values, true) else {
            return invalid();
        };
        if global {
            spo_flags.set(mask);
        }
        if local {
            wp.syntax().b_p_spo_flags = mask;
        }
    }
    Ok(())
}

pub fn did_set_spellsuggest(_args: &mut OptSet) -> Result<(), OptError> {
    if spell_check_sps().is_err() {
        return invalid();
    }
    Ok(())
}

pub fn did_set_tagcase(args: &mut OptSet) -> Result<(), OptError> {
    let (mut buf, opt_flags) = (args.os_buf, args.os_flags);
    let local = opt_flags.has(OptionSetFlags::LOCAL);
    // The variable being set is the buffer's own copy for `:setlocal` and
    // the global one otherwise -- which is the value the frame holds.
    let value = new_bytes(args);
    // An empty buffer-local value means "no override".
    let mask = if local && value.is_empty() {
        0 as c_uint
    } else {
        match opt_strings_mask(value, &opt_tc_values, false) {
            Some(mask) => mask,
            None => return invalid(),
        }
    };
    if local {
        buf.b_tc_flags = mask;
    } else {
        tc_flags.set(mask);
    }
    Ok(())
}
