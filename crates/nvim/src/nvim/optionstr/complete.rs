//! The callbacks for the completion, spelling and tag options.
//!
//! They are `pub` only so the generated option table can name them; see
//! [`super::frame`] for what they are handed.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_uint};
use core::ptr;

use crate::src::nvim::eval::userfunc::get_scriptlocal_funcname;
use crate::src::nvim::insexpand::set_cpt_callbacks;
use crate::src::nvim::main::{
    cia_flags, cot_flags, didset_vim, didset_vimruntime, p_cia, p_cot, p_hlg, p_tc, spo_flags,
    tc_flags,
};
use crate::src::nvim::option::copy_option_part;
use crate::src::nvim::options::{opt_cot_values, opt_spo_values, opt_tc_values};
use crate::src::nvim::os::env::vim_unsetenv_ext;
use crate::src::nvim::spell::{
    compile_cap_prog, did_set_spell_option, valid_spellfile, valid_spelllang,
};
use crate::src::nvim::spellfile::spell_check_msm;
use crate::src::nvim::spellsuggest::spell_check_sps;
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{buf_T, optset_T};

use super::frame::{errbuf, invalid, varp, win};
use super::{
    CPT_ABBR, CPT_KIND, CPT_MENU, LSIZE, NUL, OK, OPT_GLOBAL, OPT_LOCAL, free_string_option,
    illegal_char, illegal_char_after_chr, opt_strings_flags,
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
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_complete(args: *mut optset_T) -> *const c_char {
    let (buf, buflen) = unsafe { errbuf(args) };
    // SAFETY: the frame's C string value, walked to its terminator.
    unsafe {
        let mut p = *varp(args);
        while *p != 0 {
            let mut part = [0u8; LSIZE as usize];
            let mut into = 0;
            let mut escaped = false;
            while *p != 0 && (*p != b',' as c_char || escaped) && into < part.len() - 1 {
                if *p == b'\\' as c_char && *p.add(1) == b',' as c_char {
                    escaped = true;
                    p = p.add(1);
                } else {
                    escaped = false;
                    part[into] = *p as u8;
                    into += 1;
                }
                p = p.add(1);
            }
            let part = &part[..into];

            let source = part.first().copied().unwrap_or(0);
            if vim_strchr(CPT_SOURCES.as_ptr(), c_int::from(source)).is_null() {
                return illegal_char(buf, buflen, c_int::from(source));
            }

            // Anything after the source letter is either that source's
            // argument or a `^<count>`; anything else names a character the
            // source does not take.
            let takes_argument =
                !vim_strchr(CPT_WITH_ARGUMENT.as_ptr(), c_int::from(source)).is_null();
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
                if buf.is_null() {
                    return ptr::null();
                }
                return illegal_char_after_chr(buf, buflen, c_int::from(char_before));
            }

            while *p == b',' as c_char || *p == b' ' as c_char {
                p = p.add(1);
            }
        }

        // The "F" source names a function, which is resolved last because
        // it can fail for a reason the letter walk cannot see.
        if set_cpt_callbacks(args) != OK {
            return illegal_char_after_chr(buf, buflen, c_int::from(b'F'));
        }
    }
    ptr::null()
}

/// 'completeitemalign' is the three completion-menu columns in the order
/// they are drawn: each exactly once, none of them missing.
///
/// The order is kept as a base-10 number, one digit per column, which is
/// what the menu drawing code reads.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_completeitemalign(_args: *mut optset_T) -> *const c_char {
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
    unsafe {
        let mut p = p_cia.get();
        while *p != 0 {
            copy_option_part(
                &raw mut p,
                buf.as_mut_ptr(),
                buf.len(),
                c",".as_ptr().cast_mut(),
            );
            if count >= COLUMNS.len() {
                return invalid();
            }
            let column = CStr::from_ptr(buf.as_ptr());
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
    }
    // "abbr" alone would leave the order at 0, which is also "nothing was
    // named"; the count is what rules that out.
    if order == 0 || count != COLUMNS.len() {
        return invalid();
    }
    cia_flags.set(order);
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_completeopt(args: *mut optset_T) -> *const c_char {
    // SAFETY: the caller's frame and buffer.
    let (buf, opt_flags) = unsafe { ((*args).os_buf.cast::<buf_T>(), (*args).os_flags) };
    // SAFETY: the frame's buffer.
    let (value, flags) = unsafe {
        if opt_flags & OPT_LOCAL as c_int != 0 {
            ((*buf).b_p_cot, &raw mut (*buf).b_cot_flags)
        } else {
            if opt_flags & OPT_GLOBAL as c_int == 0 {
                // A plain `:set` drops the buffer's own answer.
                (*buf).b_cot_flags = 0 as c_uint;
            }
            (p_cot.get(), cot_flags.ptr())
        }
    };
    // SAFETY: a C string, the table's own word list and the mask beside it.
    if unsafe {
        opt_strings_flags(
            value,
            opt_cot_values.ptr().cast::<*const c_char>(),
            flags,
            true,
        )
    } != OK
    {
        return invalid();
    }
    ptr::null()
}

/// A 'helpfile' the user chose overrides `$VIM`/`$VIMRUNTIME`, so the ones
/// nvim derived for itself are dropped and re-derived from it.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_helpfile(_args: *mut optset_T) -> *const c_char {
    // SAFETY: unsets this process's own environment variables.
    unsafe {
        if didset_vim.get() {
            vim_unsetenv_ext(c"VIM".as_ptr());
        }
        if didset_vimruntime.get() {
            vim_unsetenv_ext(c"VIMRUNTIME".as_ptr());
        }
    }
    ptr::null()
}

/// 'helplang' is a comma-separated list of two-letter language codes, which
/// is checked by position rather than by parsing.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_helplang(_args: *mut optset_T) -> *const c_char {
    // SAFETY: the option's own C string value; each test below is reached
    // only once the byte before it is known not to be the terminator.
    unsafe {
        let mut s = p_hlg.get();
        while c_int::from(*s) != NUL {
            if c_int::from(*s.add(1)) == NUL
                || ((*s.add(2) != b',' as c_char || c_int::from(*s.add(3)) == NUL)
                    && c_int::from(*s.add(2)) != NUL)
            {
                return invalid();
            }
            if c_int::from(*s.add(2)) == NUL {
                break;
            }
            s = s.add(3);
        }
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_mkspellmem(_args: *mut optset_T) -> *const c_char {
    // SAFETY: re-reads the option's own value.
    if unsafe { spell_check_msm() } != OK {
        return invalid();
    }
    ptr::null()
}

/// The callback for every option holding an expression or a function name
/// ('foldexpr', 'formatexpr', 'completefunc', …).
///
/// A `s:`-prefixed name is resolved to its script-local spelling now, while
/// the script that set the option is still on the stack; the option's value
/// is rewritten in place with the answer.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_optexpr(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame's own variable; `get_scriptlocal_funcname` returns
    // a fresh allocation or null, and the old value is freed here.
    unsafe {
        let varp = varp(args);
        let resolved = get_scriptlocal_funcname(*varp);
        if !resolved.is_null() {
            free_string_option(*varp);
            *varp = resolved;
        }
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_spellcapcheck(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame's window and its syntax block.
    unsafe { compile_cap_prog((*win(args)).w_s) }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_spellfile(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame's C string value.
    if !unsafe { valid_spellfile(*varp(args)) } {
        return invalid();
    }
    // SAFETY: re-reads the spelling options.
    unsafe { did_set_spell_option() }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_spelllang(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame's C string value.
    if !unsafe { valid_spelllang(*varp(args)) } {
        return invalid();
    }
    // SAFETY: re-reads the spelling options.
    unsafe { did_set_spell_option() }
}

/// 'spelloptions' keeps a mask at both scopes, and a `:set` writes both.
///
/// The window's mask lives in its *syntax block*, which a diff or preview
/// window may share with another window.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_spelloptions(args: *mut optset_T) -> *const c_char {
    // SAFETY: the caller's frame, window and new value.
    let (wp, opt_flags, value) =
        unsafe { (win(args), (*args).os_flags, (*args).os_newval.string.data) };
    let words = opt_spo_values.ptr().cast::<*const c_char>();
    // SAFETY: a C string, the table's own word list, and each scope's mask.
    unsafe {
        if opt_flags & OPT_LOCAL as c_int == 0
            && opt_strings_flags(value, words, spo_flags.ptr(), true) != OK
        {
            return invalid();
        }
        if opt_flags & OPT_GLOBAL as c_int == 0
            && opt_strings_flags(value, words, &raw mut (*(*wp).w_s).b_p_spo_flags, true) != OK
        {
            return invalid();
        }
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_spellsuggest(_args: *mut optset_T) -> *const c_char {
    // SAFETY: re-reads the option's own value.
    if unsafe { spell_check_sps() } != OK {
        return invalid();
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_tagcase(args: *mut optset_T) -> *const c_char {
    // SAFETY: the caller's frame and buffer.
    let (buf, opt_flags) = unsafe { ((*args).os_buf.cast::<buf_T>(), (*args).os_flags) };
    let local = opt_flags & OPT_LOCAL as c_int != 0;
    // SAFETY: the frame's buffer.
    let (value, flags) = unsafe {
        if local {
            ((*buf).b_p_tc, &raw mut (*buf).b_tc_flags)
        } else {
            (p_tc.get(), tc_flags.ptr())
        }
    };
    // SAFETY: a C string, the table's own word list and the mask beside it.
    unsafe {
        // An empty buffer-local value means "no override".
        if local && c_int::from(*value) == NUL {
            *flags = 0 as c_uint;
        } else if opt_strings_flags(
            value,
            opt_tc_values.ptr().cast::<*const c_char>(),
            flags,
            false,
        ) != OK
        {
            return invalid();
        }
    }
    ptr::null()
}
