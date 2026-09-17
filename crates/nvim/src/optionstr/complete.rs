//! The callbacks for the completion, spelling and tag options.
//!
//! They are `pub` only so the generated option table can name them; see
//! [`super::frame`] for what they are handed.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::strings::has_char;
use core::ffi::{CStr, c_char, c_int, c_uint};

use crate::eval::userfunc::get_scriptlocal_funcname;
use crate::insexpand::set_cpt_callbacks;
use crate::option::copy_option_part;
use crate::option::vars::{P_CIA, P_COT, P_HLG, P_TC, cia_flags, cot_flags, spo_flags, tc_flags};
use crate::options::{opt_cot_values, opt_spo_values, opt_tc_values};
use crate::os::env::vim_unsetenv_ext;
use crate::os::state::{didset_vim, didset_vimruntime};
use crate::spell::{compile_cap_prog, did_set_spell_option, valid_spellfile, valid_spelllang};
use crate::spellfile::spell_check_msm;
use crate::spellsuggest::spell_check_sps;
use crate::types::{NUL, OptError, OptSet, OptionSetFlags};

use super::frame::{invalid, varp, win};
use super::{
    CPT_ABBR, CPT_KIND, CPT_MENU, LSIZE, free_string_option, illegal_char, illegal_char_after_chr,
    opt_strings_mask,
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
    // SAFETY: the frame's C string value, walked to its terminator.
    let mut p = unsafe { varp(args).get() };
    while unsafe { *p } != 0 {
        let mut part = [0u8; LSIZE as usize];
        let mut into = 0;
        let mut escaped = false;
        while unsafe { *p } != 0
            && (unsafe { *p } != b',' as c_char || escaped)
            && into < part.len() - 1
        {
            if unsafe { *p } == b'\\' as c_char && unsafe { *p.add(1) } == b',' as c_char {
                escaped = true;
                p = unsafe { p.add(1) };
            } else {
                escaped = false;
                part[into] = unsafe { *p } as u8;
                into += 1;
            }
            p = unsafe { p.add(1) };
        }
        let part = &part[..into];

        let source = part.first().copied().unwrap_or(0);
        if !has_char(
            unsafe { cstr::at(CPT_SOURCES.as_ptr()) },
            c_int::from(source),
        ) {
            return Err(illegal_char(c_int::from(source)));
        }

        // Anything after the source letter is either that source's
        // argument or a `^<count>`; anything else names a character the
        // source does not take.
        let takes_argument = has_char(
            unsafe { cstr::at(CPT_WITH_ARGUMENT.as_ptr()) },
            c_int::from(source),
        );
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

        while unsafe { *p } == b',' as c_char || unsafe { *p } == b' ' as c_char {
            p = unsafe { p.add(1) };
        }
    }

    // The "F" source names a function, which is resolved last because
    // it can fail for a reason the letter walk cannot see.
    if unsafe { set_cpt_callbacks(args) }.is_err() {
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
    const COLUMNS: [(&CStr, c_int); 3] = [
        (c"abbr", CPT_ABBR as c_int),
        (c"kind", CPT_KIND as c_int),
        (c"menu", CPT_MENU as c_int),
    ];
    let mut order: c_uint = 0;
    let mut seen = [false; 3];
    let mut count = 0;
    let mut buf = [0 as c_char; 10];

    // SAFETY: the option's own C string value, and a scratch buffer of the
    // size given.
    // A copy: the cursor below walks past the end of a projection's borrow.
    let cia = P_CIA.get();
    let mut p = cia.as_ptr().cast_mut();
    while unsafe { *p } != 0 {
        unsafe {
            copy_option_part(
                &raw mut p,
                buf.as_mut_ptr(),
                buf.len(),
                c",".as_ptr().cast_mut(),
            )
        };
        if count >= COLUMNS.len() {
            return invalid();
        }
        let column = unsafe { CStr::from_ptr(buf.as_ptr()) };
        let Some((_, which)) = COLUMNS.iter().find(|(name, _)| *name == column) else {
            return invalid();
        };
        if seen[*which as usize] {
            return invalid();
        }
        seen[*which as usize] = true;
        order = order * 10 + *which as c_uint;
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
    // A copy, so that the global value outlives the projection: `:set` rate.
    let global = P_COT.get();
    let value = if local {
        buf.b_p_cot
    } else {
        if !opt_flags.has(OptionSetFlags::GLOBAL) {
            // A plain `:set` drops the buffer's own answer.
            buf.b_cot_flags = 0 as c_uint;
        }
        global.as_ptr().cast_mut()
    };
    // SAFETY: a C string, against the table's own word list.
    let Some(mask) = (unsafe { opt_strings_mask(value, &opt_cot_values, true) }) else {
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
    // SAFETY: unsets this process's own environment variables.
    if didset_vim.get() {
        unsafe { vim_unsetenv_ext(c"VIM".as_ptr()) };
    }
    if didset_vimruntime.get() {
        unsafe { vim_unsetenv_ext(c"VIMRUNTIME".as_ptr()) };
    }
    Ok(())
}

/// 'helplang' is a comma-separated list of two-letter language codes, which
/// is checked by position rather than by parsing.
pub fn did_set_helplang(_args: &mut OptSet) -> Result<(), OptError> {
    // SAFETY: the option's own C string value; each test below is reached
    // only once the byte before it is known not to be the terminator.
    // A copy: the cursor below walks past the end of a projection's borrow.
    let hlg = P_HLG.get();
    let mut s = hlg.as_ptr().cast_mut();
    while c_int::from(unsafe { *s }) != NUL {
        if c_int::from(unsafe { *s.add(1) }) == NUL
            || ((unsafe { *s.add(2) } != b',' as c_char
                || c_int::from(unsafe { *s.add(3) }) == NUL)
                && c_int::from(unsafe { *s.add(2) }) != NUL)
        {
            return invalid();
        }
        if c_int::from(unsafe { *s.add(2) }) == NUL {
            break;
        }
        s = unsafe { s.add(3) };
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
    // SAFETY: the frame's own variable; `get_scriptlocal_funcname` returns
    // a fresh allocation or null, and the old value is freed here.
    let varp = varp(args);
    let resolved = unsafe { get_scriptlocal_funcname(varp.get()) };
    if !resolved.is_null() {
        // Replace and *then* free: a global value's string is the option
        // record's, so freeing it before the write would release it twice.
        let old = unsafe { varp.replace(resolved) };
        unsafe { free_string_option(old) };
    }
    Ok(())
}

pub fn did_set_spellcapcheck(args: &mut OptSet) -> Result<(), OptError> {
    // SAFETY: the frame's window and its syntax block.
    unsafe { compile_cap_prog(win(args).w_s) }
}

pub fn did_set_spellfile(args: &mut OptSet) -> Result<(), OptError> {
    // SAFETY: the frame's C string value.
    if !unsafe { valid_spellfile(varp(args).get()) } {
        return invalid();
    }
    did_set_spell_option()
}

pub fn did_set_spelllang(args: &mut OptSet) -> Result<(), OptError> {
    // SAFETY: the frame's C string value.
    if !valid_spelllang(unsafe { CStr::from_ptr(varp(args).get()) }) {
        return invalid();
    }
    did_set_spell_option()
}

/// 'spelloptions' keeps a mask at both scopes, and a `:set` writes both.
///
/// The window's mask lives in its *syntax block*, which a diff or preview
/// window may share with another window.
pub fn did_set_spelloptions(args: &mut OptSet) -> Result<(), OptError> {
    let (mut wp, opt_flags, new) = (win(args), args.os_flags, args.os_newval);
    let value = new
        .as_string()
        .expect("the table installs this callback on a string option only")
        .data();
    let global = !opt_flags.has(OptionSetFlags::LOCAL);
    let local = !opt_flags.has(OptionSetFlags::GLOBAL);
    if global || local {
        // SAFETY: a C string, against the table's own word list.
        let Some(mask) = (unsafe { opt_strings_mask(value, &opt_spo_values, true) }) else {
            return invalid();
        };
        if global {
            spo_flags.set(mask);
        }
        if local {
            // SAFETY: the frame's window and its syntax block.
            unsafe { (*wp.w_s).b_p_spo_flags = mask };
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
    // A copy, so that the global value outlives the projection: `:set` rate.
    let global = P_TC.get();
    let value = if local {
        buf.b_p_tc
    } else {
        global.as_ptr().cast_mut()
    };
    // An empty buffer-local value means "no override".
    // SAFETY: an option's value is a C string.
    let mask = if local && unsafe { c_int::from(*value) } == NUL {
        0 as c_uint
    } else {
        // SAFETY: a C string, against the table's own word list.
        match unsafe { opt_strings_mask(value, &opt_tc_values, false) } {
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
