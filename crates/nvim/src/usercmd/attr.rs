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
use super::{FAIL, OK, UC_BUFFER};
use crate::ascii::ascii_iswhite;
use crate::charset::getdigits_int;
use crate::message::emsg;
use crate::message_fmt::c_str;
use crate::os::cshim::{gettext, gettext_ptr};
use crate::semsg;
use crate::semsg_c;
use crate::strings::xstrnsave;
use crate::types::{CmdAddr, ExArgt, ExpandContext, NUL, size_t};
use core::ffi::{CStr, c_char, c_int};
use core::slice;

/// The address types `-addr=` names, and the short forms `:command` lists
/// them under.
///
/// Upstream terminates the table with an `CmdAddr::NoRange` row and walks until it
/// meets it; the sentinel is also what [`super::complete::
/// get_user_cmd_addr_type`] answers null for, ending completion. Here the
/// end of the slice is that sentinel.
pub(super) struct AddrType {
    pub(super) expand: CmdAddr,
    pub(super) name: &'static CStr,
    pub(super) shortname: &'static CStr,
}

/// Must stay alphabetical by `name`: it is offered for completion in order.
#[rustfmt::skip]
pub(super) static ADDR_TYPES: [AddrType; 8] = [
    AddrType { expand: CmdAddr::Arguments,      name: c"arguments",      shortname: c"arg" },
    AddrType { expand: CmdAddr::Lines,          name: c"lines",          shortname: c"line" },
    AddrType { expand: CmdAddr::LoadedBuffers, name: c"loaded_buffers", shortname: c"load" },
    AddrType { expand: CmdAddr::Tabs,           name: c"tabs",           shortname: c"tab" },
    AddrType { expand: CmdAddr::Buffers,        name: c"buffers",        shortname: c"buf" },
    AddrType { expand: CmdAddr::Windows,        name: c"windows",        shortname: c"win" },
    AddrType { expand: CmdAddr::Quickfix,       name: c"quickfix",       shortname: c"qf" },
    AddrType { expand: CmdAddr::Other,          name: c"other",          shortname: c"?" },
];

/// The row whose `expand` is `addr_type`, if it is one that has a name.
///
/// `CmdAddr::Lines` is deliberately excluded: it is the default, and neither
/// `:command`'s listing nor `nvim_get_commands()` names a default.
pub(super) fn named_addr_type(addr_type: CmdAddr) -> Option<&'static AddrType> {
    ADDR_TYPES
        .iter()
        .find(|row| row.expand != CmdAddr::Lines && row.expand == addr_type)
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
pub(crate) unsafe fn parse_addr_type_arg(
    value: *mut c_char,
    vallen: c_int,
    addr_type_arg: *mut CmdAddr,
) -> c_int {
    // SAFETY: caller contract.
    let typed = unsafe { slice::from_raw_parts(value.cast::<u8>(), vallen as usize) };
    if let Some(row) = ADDR_TYPES.iter().find(|row| row.name.to_bytes() == typed) {
        // SAFETY: caller contract.
        unsafe { *addr_type_arg = row.expand };
        return OK;
    }

    // SAFETY: caller contract; the walk stops at the NUL.
    let mut i = 0;
    while unsafe { *value.add(i) } != NUL as c_char
        && !ascii_iswhite(unsafe { *value.add(i) } as c_int)
    {
        i += 1;
    }
    unsafe { *value.add(i) = NUL as c_char };
    let untranslated = c"E180: Invalid address type value: %s".as_ptr();
    // SAFETY: caller contract; `value` is now NUL-terminated at the word, and
    // the one `%s` spends it.
    unsafe {
        let fmt = gettext_ptr(untranslated);
        semsg_c!(fmt, value)
    };
    FAIL
}

/// Parse a `-complete=` value: a name from [`COMMAND_COMPLETE`], optionally
/// followed by `,` and the function argument that `custom`/`customlist`
/// need.
///
/// Stores the `EXPAND_*` type in `complp`, may add `ExArgt::BUFNAME`/`ExArgt::XFILE`
/// to `argt`, and hands `compl_arg` a fresh copy of the argument part.
///
/// # Safety
/// `value` must be NUL-terminated with at least `vallen` readable bytes.
pub(crate) unsafe fn parse_compl_arg(
    value: *const c_char,
    vallen: c_int,
    complp: &mut ExpandContext,
    argt: &mut ExArgt,
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
        .filter_map(|i| ExpandContext::try_from(i).ok())
        .find(|&i| command_complete_name(i).is_some_and(|n| n.to_bytes() == name));
    let Some(expand) = found else {
        // SAFETY: caller contract.
        let value = unsafe { c_str(value) };
        semsg!("E180: Invalid complete value: {value}");
        return FAIL;
    };
    *complp = expand;
    if expand == ExpandContext::Buffers {
        *argt |= ExArgt::BUFNAME;
    } else if expand == ExpandContext::Directories
        || expand == ExpandContext::Files
        || expand == ExpandContext::ShellCmdLine
    {
        *argt |= ExArgt::XFILE;
    }

    let custom = expand == ExpandContext::UserDefined || expand == ExpandContext::UserList;
    // SAFETY: both messages are literals.
    if !custom && arg.is_some() {
        let msg = c"E468: Completion argument only allowed for custom completion".as_ptr();
        // SAFETY: the message is a static string.
        unsafe { emsg(gettext_ptr(msg)) };
        return FAIL;
    }
    if custom && arg.is_none() {
        let msg = c"E467: Custom completion requires a function argument".as_ptr();
        // SAFETY: as above.
        unsafe { emsg(gettext_ptr(msg)) };
        return FAIL;
    }
    if let Some(arg) = arg {
        // SAFETY: `arg` is a sub-slice of `value`, which is live.
        *compl_arg = unsafe { xstrnsave(arg.as_ptr().cast::<c_char>(), arg.len()) };
    }
    OK
}

/// Everything one attribute can change about a command being defined.
pub(super) struct Attributes<'a> {
    pub(super) argt: &'a mut ExArgt,
    pub(super) def: &'a mut c_int,
    pub(super) flags: &'a mut c_int,
    pub(super) complp: &'a mut ExpandContext,
    pub(super) compl_arg: &'a mut *mut c_char,
    pub(super) addr_type_arg: &'a mut CmdAddr,
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
        emsg(gettext(c"E175: No attribute specified"));
        return FAIL;
    }
    // SAFETY: caller contract.
    let typed = unsafe { slice::from_raw_parts(attr.cast::<u8>(), len) };

    // The flag attributes take no value.
    if abbreviates(typed, "bang") {
        *into.argt |= ExArgt::BANG;
        return OK;
    } else if abbreviates(typed, "buffer") {
        *into.flags |= UC_BUFFER;
        return OK;
    } else if abbreviates(typed, "register") {
        *into.argt |= ExArgt::REGSTR;
        return OK;
    } else if abbreviates(typed, "keepscript") {
        *into.argt |= ExArgt::KEEPSCRIPT;
        return OK;
    } else if abbreviates(typed, "bar") {
        *into.argt |= ExArgt::TRLBAR;
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
            Some(v) => {
                let (text, len) = (value_ptr(v), v.len() as c_int);
                // SAFETY: caller contract; the three out-parameters are the
                // caller's own.
                let got =
                    unsafe { parse_compl_arg(text, len, into.complp, into.argt, into.compl_arg) };
                match got {
                    FAIL => Err(Bad::Reported),
                    _ => Ok(()),
                }
            }
        }
    } else if abbreviates(name, "addr") {
        *into.argt |= ExArgt::RANGE;
        match value {
            None => Err(Bad::Missing(c"-addr")),
            // SAFETY: caller contract.
            Some(v) => match unsafe {
                parse_addr_type_arg(value_ptr(v), v.len() as c_int, into.addr_type_arg)
            } {
                FAIL => Err(Bad::Reported),
                _ => {
                    if *into.addr_type_arg != CmdAddr::Lines {
                        *into.argt |= ExArgt::ZEROR;
                    }
                    Ok(())
                }
            },
        }
    } else {
        // SAFETY: caller contract; the byte past the attribute is restored
        // as soon as the message has been formatted.
        let ch = unsafe { *attr.add(len) };
        unsafe { *attr.add(len) = NUL as c_char };
        // SAFETY: the byte past the attribute was just replaced by a NUL.
        let shown = unsafe { c_str(attr) };
        semsg!("E181: Invalid attribute: {shown}");
        unsafe { *attr.add(len) = ch };
        return FAIL;
    };

    let Err(bad) = outcome else { return OK };
    // SAFETY: every message and argument here is a literal.
    match bad {
        Bad::Nargs => {
            emsg(gettext(c"E176: Invalid number of arguments"));
        }
        Bad::TwoCount => {
            emsg(gettext(c"E177: Count cannot be specified twice"));
        }
        Bad::InvalidCount => {
            emsg(gettext(c"E178: Invalid default value for count"));
        }
        Bad::Missing(what) => {
            let untranslated = c"E179: Argument required for %s".as_ptr();
            // SAFETY: the one `%s` spends the attribute name.
            unsafe {
                let fmt = gettext_ptr(untranslated);
                semsg_c!(fmt, what.as_ptr())
            };
        }
        Bad::Reported => {}
    }
    FAIL
}

/// `-nargs=`: one of `0`, `1`, `*`, `?`, `+`.
fn scan_nargs(value: Option<&[u8]>, argt: &mut ExArgt) -> Result<(), Bad> {
    match value {
        Some([b'0']) => Ok(()),
        Some([b'1']) => {
            *argt |= ExArgt::EXTRA | ExArgt::NOSPC | ExArgt::NEEDARG;
            Ok(())
        }
        Some([b'*']) => {
            *argt |= ExArgt::EXTRA;
            Ok(())
        }
        Some([b'?']) => {
            *argt |= ExArgt::EXTRA | ExArgt::NOSPC;
            Ok(())
        }
        Some([b'+']) => {
            *argt |= ExArgt::EXTRA | ExArgt::NEEDARG;
            Ok(())
        }
        _ => Err(Bad::Nargs),
    }
}

/// `-range`, `-range=%` or `-range=N`.
fn scan_range(
    value: Option<&[u8]>,
    argt: &mut ExArgt,
    def: &mut c_int,
    addr_type_arg: &mut CmdAddr,
) -> Result<(), Bad> {
    *argt |= ExArgt::RANGE;
    match value {
        Some(b"%") => *argt |= ExArgt::DFLALL,
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
            *argt |= ExArgt::ZEROR;
        }
        None => {}
    }
    // The default for -range is to count buffer lines.
    if *addr_type_arg == CmdAddr::NoRange {
        *addr_type_arg = CmdAddr::Lines;
    }
    Ok(())
}

/// `-count` or `-count=N`.
fn scan_count(
    value: Option<&[u8]>,
    argt: &mut ExArgt,
    def: &mut c_int,
    addr_type_arg: &mut CmdAddr,
) -> Result<(), Bad> {
    *argt |= ExArgt::COUNT | ExArgt::ZEROR | ExArgt::RANGE;
    // The default for -count is to count anything.
    if *addr_type_arg == CmdAddr::NoRange {
        *addr_type_arg = CmdAddr::Other;
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
