//! `:command`'s attributes: `-bang`, `-bar`, `-buffer`, `-register`,
//! `-keepscript`, `-nargs=`, `-range=`, `-count=`, `-addr=` and
//! `-complete=`.
//!
//! [`uc_scan_attr`] takes one attribute and folds it into the four things a
//! command definition is made of: the `EX_*` flag word `argt`, the default
//! count `def`, the `UC_BUFFER` flag, and the completion pair
//! (`complp`/`compl_arg`). The two `=`-valued parsers that need a table of
//! their own are separate, because the API takes the same strings for
//! `nvim_create_user_command()` and calls them directly.
//!
//! Attribute names are matched as *prefixes* -- `-ba` names `-bang` --
//! which is what [`abbreviates`] spells out. `-complete=` values are not:
//! they match whole.
//!
//! Original: `src/nvim/usercmd.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::complete::{COMMAND_COMPLETE, command_complete_name};
use super::{
    ADDR_ARGUMENTS, ADDR_BUFFERS, ADDR_LINES, ADDR_LOADED_BUFFERS, ADDR_NONE, ADDR_OTHER,
    ADDR_QUICKFIX, ADDR_TABS, ADDR_WINDOWS, EX_BANG, EX_BUFNAME, EX_COUNT, EX_DFLALL, EX_EXTRA,
    EX_KEEPSCRIPT, EX_NEEDARG, EX_NOSPC, EX_RANGE, EX_REGSTR, EX_TRLBAR, EX_XFILE, EX_ZEROR,
    EXPAND_BUFFERS, EXPAND_DIRECTORIES, EXPAND_FILES, EXPAND_SHELLCMDLINE, EXPAND_USER_DEFINED,
    EXPAND_USER_LIST, FAIL, NUL, OK, UC_BUFFER,
};
use crate::semsg_c;
use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::charset::getdigits_int;
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::strings::xstrnsave;
use crate::src::nvim::types::{cmd_addr_T, size_t, uint32_t};
use core::ffi::{CStr, c_char, c_int};
use core::slice;

/// The address types `-addr=` names, and the short forms `:command` lists
/// them under.
///
/// Upstream terminates the table with an `ADDR_NONE` row and walks until it
/// meets it; the sentinel is also what [`super::complete::
/// get_user_cmd_addr_type`] answers null for, ending completion. Here the
/// end of the slice is that sentinel.
pub(super) struct AddrType {
    pub(super) expand: cmd_addr_T,
    pub(super) name: &'static CStr,
    pub(super) shortname: &'static CStr,
}

/// Must stay alphabetical by `name`: it is offered for completion in order.
#[rustfmt::skip]
pub(super) static ADDR_TYPES: [AddrType; 8] = [
    AddrType { expand: ADDR_ARGUMENTS,      name: c"arguments",      shortname: c"arg" },
    AddrType { expand: ADDR_LINES,          name: c"lines",          shortname: c"line" },
    AddrType { expand: ADDR_LOADED_BUFFERS, name: c"loaded_buffers", shortname: c"load" },
    AddrType { expand: ADDR_TABS,           name: c"tabs",           shortname: c"tab" },
    AddrType { expand: ADDR_BUFFERS,        name: c"buffers",        shortname: c"buf" },
    AddrType { expand: ADDR_WINDOWS,        name: c"windows",        shortname: c"win" },
    AddrType { expand: ADDR_QUICKFIX,       name: c"quickfix",       shortname: c"qf" },
    AddrType { expand: ADDR_OTHER,          name: c"other",          shortname: c"?" },
];

/// The row whose `expand` is `addr_type`, if it is one that has a name.
///
/// `ADDR_LINES` is deliberately excluded: it is the default, and neither
/// `:command`'s listing nor `nvim_get_commands()` names a default.
pub(super) fn named_addr_type(addr_type: cmd_addr_T) -> Option<&'static AddrType> {
    ADDR_TYPES
        .iter()
        .find(|row| row.expand != ADDR_LINES && row.expand == addr_type)
}

/// C's `STRNICMP(attr, name, len) == 0` over `attr`'s `len` bytes.
///
/// A *prefix* test, not an equality: upstream compares only as far as the
/// user typed, so `-ba` names `-bang` and `-nar=1` names `-nargs=1`. An
/// `attr` longer than `name` runs into `name`'s NUL and cannot match.
fn abbreviates(attr: &[u8], name: &str) -> bool {
    attr.len() <= name.len() && name.as_bytes()[..attr.len()].eq_ignore_ascii_case(attr)
}

/// Parse an `-addr=` value, storing the type it names.
///
/// On failure the value is truncated at the first whitespace *in place*, as
/// upstream does, so that the message names only the offending word.
///
/// # Safety
/// `value` must be writable and NUL-terminated, with at least `vallen`
/// readable bytes.
pub unsafe fn parse_addr_type_arg(
    value: *mut c_char,
    vallen: c_int,
    addr_type_arg: *mut cmd_addr_T,
) -> c_int {
    // SAFETY: caller contract.
    let typed = unsafe { slice::from_raw_parts(value.cast::<u8>(), vallen as usize) };
    if let Some(row) = ADDR_TYPES.iter().find(|row| row.name.to_bytes() == typed) {
        // SAFETY: caller contract.
        unsafe { *addr_type_arg = row.expand };
        return OK;
    }

    // SAFETY: caller contract; the walk stops at the NUL.
    unsafe {
        let mut i = 0;
        while *value.add(i) != NUL && !ascii_iswhite(*value.add(i) as c_int) {
            i += 1;
        }
        *value.add(i) = NUL;
    }
    // SAFETY: caller contract; `value` is now NUL-terminated at the word.
    unsafe {
        semsg_c!(
            gettext(c"E180: Invalid address type value: %s".as_ptr()),
            value,
        );
    }
    FAIL
}

/// Parse a `-complete=` value: a name from [`COMMAND_COMPLETE`], optionally
/// followed by `,` and the function argument that `custom`/`customlist`
/// need.
///
/// Stores the `EXPAND_*` type in `complp`, may add `EX_BUFNAME`/`EX_XFILE`
/// to `argt`, and hands `compl_arg` a fresh copy of the argument part.
///
/// # Safety
/// `value` must be NUL-terminated with at least `vallen` readable bytes.
pub unsafe fn parse_compl_arg(
    value: *const c_char,
    vallen: c_int,
    complp: &mut c_int,
    argt: &mut uint32_t,
    compl_arg: &mut *mut c_char,
) -> c_int {
    // SAFETY: caller contract.
    let typed = unsafe { slice::from_raw_parts(value.cast::<u8>(), vallen as usize) };
    // The argument part is whatever follows the first comma.
    let (name, arg) = match typed.iter().position(|&b| b == b',') {
        Some(comma) => (&typed[..comma], Some(&typed[comma + 1..])),
        None => (typed, None),
    };

    let found = (0..COMMAND_COMPLETE.len() as c_int)
        .find(|&i| command_complete_name(i).is_some_and(|n| n.to_bytes() == name));
    let Some(expand) = found else {
        // SAFETY: caller contract.
        unsafe {
            semsg_c!(gettext(c"E180: Invalid complete value: %s".as_ptr()), value,);
        }
        return FAIL;
    };
    *complp = expand;
    if expand == EXPAND_BUFFERS {
        *argt |= EX_BUFNAME;
    } else if expand == EXPAND_DIRECTORIES
        || expand == EXPAND_FILES
        || expand == EXPAND_SHELLCMDLINE
    {
        *argt |= EX_XFILE;
    }

    let custom = expand == EXPAND_USER_DEFINED || expand == EXPAND_USER_LIST;
    // SAFETY: both messages are literals.
    unsafe {
        if !custom && arg.is_some() {
            emsg(gettext(
                c"E468: Completion argument only allowed for custom completion".as_ptr(),
            ));
            return FAIL;
        }
        if custom && arg.is_none() {
            emsg(gettext(
                c"E467: Custom completion requires a function argument".as_ptr(),
            ));
            return FAIL;
        }
    }
    if let Some(arg) = arg {
        // SAFETY: `arg` is a sub-slice of `value`, which is live.
        *compl_arg = unsafe { xstrnsave(arg.as_ptr().cast::<c_char>(), arg.len()) };
    }
    OK
}

/// Everything one attribute can change about a command being defined.
pub(super) struct Attributes<'a> {
    pub(super) argt: &'a mut uint32_t,
    pub(super) def: &'a mut c_int,
    pub(super) flags: &'a mut c_int,
    pub(super) complp: &'a mut c_int,
    pub(super) compl_arg: &'a mut *mut c_char,
    pub(super) addr_type_arg: &'a mut cmd_addr_T,
}

/// What went wrong with an attribute, as the message still to report.
enum Bad {
    Nargs,
    TwoCount,
    InvalidCount,
    /// `-complete` and `-addr` are the two that insist on a value.
    Missing(&'static CStr),
    /// The value parser has reported already.
    Reported,
}

/// Fold one `-attribute[=value]` into `into`.
///
/// `attr` is `len` bytes of the command line, not NUL-terminated there --
/// which is why the "invalid attribute" message writes a NUL over
/// `attr[len]` and puts the byte back afterwards.
///
/// # Safety
/// `attr` must be writable for `len + 1` bytes.
pub(super) unsafe fn uc_scan_attr(attr: *mut c_char, len: size_t, into: Attributes) -> c_int {
    if len == 0 {
        // SAFETY: the message is a literal.
        unsafe { emsg(gettext(c"E175: No attribute specified".as_ptr())) };
        return FAIL;
    }
    // SAFETY: caller contract.
    let typed = unsafe { slice::from_raw_parts(attr.cast::<u8>(), len) };

    // The flag attributes take no value.
    if abbreviates(typed, "bang") {
        *into.argt |= EX_BANG;
        return OK;
    } else if abbreviates(typed, "buffer") {
        *into.flags |= UC_BUFFER;
        return OK;
    } else if abbreviates(typed, "register") {
        *into.argt |= EX_REGSTR;
        return OK;
    } else if abbreviates(typed, "keepscript") {
        *into.argt |= EX_KEEPSCRIPT;
        return OK;
    } else if abbreviates(typed, "bar") {
        *into.argt |= EX_TRLBAR;
        return OK;
    }

    // Everything else is `name=value`; a bare name leaves `value` unset,
    // which `-range` and `-count` accept and the other two reject.
    let (name, value) = match typed.iter().position(|&b| b == b'=') {
        Some(eq) => (&typed[..eq], Some(&typed[eq + 1..])),
        None => (typed, None),
    };
    // The value is a sub-slice of `attr`, so this is where it starts.
    // SAFETY: caller contract.
    let value_ptr = |v: &[u8]| unsafe { attr.add(len - v.len()) };

    let outcome = if abbreviates(name, "nargs") {
        scan_nargs(value, into.argt)
    } else if abbreviates(name, "range") {
        scan_range(value, into.argt, into.def, into.addr_type_arg)
    } else if abbreviates(name, "count") {
        scan_count(value, into.argt, into.def, into.addr_type_arg)
    } else if abbreviates(name, "complete") {
        match value {
            None => Err(Bad::Missing(c"-complete")),
            // SAFETY: caller contract.
            Some(v) => match unsafe {
                parse_compl_arg(
                    value_ptr(v),
                    v.len() as c_int,
                    into.complp,
                    into.argt,
                    into.compl_arg,
                )
            } {
                FAIL => Err(Bad::Reported),
                _ => Ok(()),
            },
        }
    } else if abbreviates(name, "addr") {
        *into.argt |= EX_RANGE;
        match value {
            None => Err(Bad::Missing(c"-addr")),
            // SAFETY: caller contract.
            Some(v) => match unsafe {
                parse_addr_type_arg(value_ptr(v), v.len() as c_int, into.addr_type_arg)
            } {
                FAIL => Err(Bad::Reported),
                _ => {
                    if *into.addr_type_arg != ADDR_LINES {
                        *into.argt |= EX_ZEROR;
                    }
                    Ok(())
                }
            },
        }
    } else {
        // SAFETY: caller contract; the byte past the attribute is restored
        // as soon as the message has been formatted.
        unsafe {
            let ch = *attr.add(len);
            *attr.add(len) = NUL;
            semsg_c!(gettext(c"E181: Invalid attribute: %s".as_ptr()), attr,);
            *attr.add(len) = ch;
        }
        return FAIL;
    };

    let Err(bad) = outcome else { return OK };
    // SAFETY: every message and argument here is a literal.
    unsafe {
        match bad {
            Bad::Nargs => {
                emsg(gettext(c"E176: Invalid number of arguments".as_ptr()));
            }
            Bad::TwoCount => {
                emsg(gettext(c"E177: Count cannot be specified twice".as_ptr()));
            }
            Bad::InvalidCount => {
                emsg(gettext(c"E178: Invalid default value for count".as_ptr()));
            }
            Bad::Missing(what) => {
                semsg_c!(
                    gettext(c"E179: Argument required for %s".as_ptr()),
                    what.as_ptr(),
                );
            }
            Bad::Reported => {}
        }
    }
    FAIL
}

/// `-nargs=`: one of `0`, `1`, `*`, `?`, `+`.
fn scan_nargs(value: Option<&[u8]>, argt: &mut uint32_t) -> Result<(), Bad> {
    match value {
        Some([b'0']) => Ok(()),
        Some([b'1']) => {
            *argt |= EX_EXTRA | EX_NOSPC | EX_NEEDARG;
            Ok(())
        }
        Some([b'*']) => {
            *argt |= EX_EXTRA;
            Ok(())
        }
        Some([b'?']) => {
            *argt |= EX_EXTRA | EX_NOSPC;
            Ok(())
        }
        Some([b'+']) => {
            *argt |= EX_EXTRA | EX_NEEDARG;
            Ok(())
        }
        _ => Err(Bad::Nargs),
    }
}

/// `-range`, `-range=%` or `-range=N`.
fn scan_range(
    value: Option<&[u8]>,
    argt: &mut uint32_t,
    def: &mut c_int,
    addr_type_arg: &mut cmd_addr_T,
) -> Result<(), Bad> {
    *argt |= EX_RANGE;
    match value {
        Some(b"%") => *argt |= EX_DFLALL,
        Some(v) => {
            if *def >= 0 {
                return Err(Bad::TwoCount);
            }
            // `-range=` with nothing after it is rejected here, where
            // `-count=` accepts it: upstream's emptiness test is on this
            // branch only.
            if v.is_empty() {
                return Err(Bad::InvalidCount);
            }
            *def = digits(v).ok_or(Bad::InvalidCount)?;
            *argt |= EX_ZEROR;
        }
        None => {}
    }
    // The default for -range is to count buffer lines.
    if *addr_type_arg == ADDR_NONE {
        *addr_type_arg = ADDR_LINES;
    }
    Ok(())
}

/// `-count` or `-count=N`.
fn scan_count(
    value: Option<&[u8]>,
    argt: &mut uint32_t,
    def: &mut c_int,
    addr_type_arg: &mut cmd_addr_T,
) -> Result<(), Bad> {
    *argt |= EX_COUNT | EX_ZEROR | EX_RANGE;
    // The default for -count is to count anything.
    if *addr_type_arg == ADDR_NONE {
        *addr_type_arg = ADDR_OTHER;
    }
    if let Some(v) = value {
        if *def >= 0 {
            return Err(Bad::TwoCount);
        }
        // Unlike -range, an empty value is accepted here: `getdigits_int`
        // answers 0 and the emptiness check below is on -range only.
        *def = digits(v).ok_or(Bad::InvalidCount)?;
    }
    *def = (*def).max(0);
    Ok(())
}

/// `value` as a whole number, or `None` when it is not all digits.
///
/// Upstream calls `getdigits_int(&p, true, 0)` and then insists that `p`
/// landed exactly on the end of the value; the strictness is the point,
/// since `-count=3x` must not quietly mean three.
fn digits(value: &[u8]) -> Option<c_int> {
    let mut p = value.as_ptr().cast::<c_char>().cast_mut();
    // SAFETY: `p` walks `value`, which is live; `getdigits_int` stops at
    // the first byte that is not a digit, and the value is followed by the
    // rest of the command line rather than by the end of an allocation.
    let n = unsafe { getdigits_int(&raw mut p, true, 0) };
    // SAFETY: as above.
    (unsafe { p.offset_from(value.as_ptr().cast::<c_char>()) } as usize == value.len()).then_some(n)
}
