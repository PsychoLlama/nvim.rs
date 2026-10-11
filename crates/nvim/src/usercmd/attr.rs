//! `:command`'s attributes: `-bang`, `-bar`, `-buffer`, `-register`,
//! `-keepscript`, `-nargs=`, `-range=`, `-count=`, `-addr=` and
//! `-complete=`.
//!
//! [`uc_scan_attr`] takes one attribute and folds it into the four things a
//! command definition is made of: the `EX_*` flag word `argt`, the default
//! count `def`, whether it is buffer-local, and the completion pair
//! (`complp`/`compl_arg`). The two `=`-valued parsers that need a table of
//! their own are separate, because the API takes the same strings for
//! `nvim_create_user_command()` and calls them directly.
//!
//! Attribute names are matched as *prefixes* -- `-ba` names `-bang` --
//! which is what [`abbreviates`] spells out. `-complete=` values are not:
//! they match whole.
//!
//! Original: `src/nvim/usercmd.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use super::complete::{COMMAND_COMPLETE, command_complete_name};
use crate::ascii::ascii_iswhite;
use crate::charset::getdigits_int_at;
use crate::cstr;
use crate::message::emsg;
use crate::message_fmt::msg_bytes;
use crate::os::cshim::gettext;
use crate::semsg;
use crate::types::{CmdAddr, ExArgt, ExpandContext, Failed};
use core::ffi::{CStr, c_int};
use std::ffi::CString;

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

/// Parse an `-addr=` value: the type it names.
///
/// `value` is the value; `shown` is the C string it starts (the rest of a
/// command line, or the API's string). On failure the answer is the word
/// the message named -- `shown` up to its first whitespace, where upstream
/// cut it in place -- which the API's own refusal quotes again.
pub(crate) fn parse_addr_type_arg<'a>(value: &[u8], shown: &'a [u8]) -> Result<CmdAddr, &'a [u8]> {
    if let Some(row) = ADDR_TYPES.iter().find(|row| row.name.to_bytes() == value) {
        return Ok(row.expand);
    }
    let word = shown
        .iter()
        .position(|&byte| byte == 0 || ascii_iswhite(c_int::from(byte)))
        .map_or(shown, |end| &shown[..end]);
    let shown_word = msg_bytes(word);
    semsg!("E180: Invalid address type value: {shown_word}");
    Err(word)
}

/// Parse a `-complete=` value: a name from [`COMMAND_COMPLETE`], optionally
/// followed by `,` and the function argument that `custom`/`customlist`
/// need.
///
/// Stores the `EXPAND_*` type in `complp`, may add `ExArgt::BUFNAME`/`ExArgt::XFILE`
/// to `argt`, and hands `compl_arg` a copy of the argument part. `shown` is
/// the C string `value` starts, which an unknown name is reported as.
pub(crate) fn parse_compl_arg(
    value: &[u8],
    shown: &[u8],
    complp: &mut ExpandContext,
    argt: &mut ExArgt,
    compl_arg: &mut Option<CString>,
) -> Result<(), Failed> {
    // The argument part is whatever follows the first comma.
    let (name, arg) = match value.iter().position(|&b| b == b',') {
        Some(comma) => (&value[..comma], Some(&value[comma + 1..])),
        None => (value, None),
    };

    let found = (0..COMMAND_COMPLETE.len())
        .filter_map(|i| c_int::try_from(i).ok())
        .filter_map(|i| ExpandContext::try_from(i).ok())
        .find(|&i| command_complete_name(i).is_some_and(|n| n.to_bytes() == name));
    let Some(expand) = found else {
        let value = msg_bytes(cstr::until_nul(shown));
        semsg!("E180: Invalid complete value: {value}");
        return Err(Failed);
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
    if !custom && arg.is_some() {
        emsg(gettext(
            c"E468: Completion argument only allowed for custom completion",
        ));
        return Err(Failed);
    }
    if custom && arg.is_none() {
        emsg(gettext(
            c"E467: Custom completion requires a function argument",
        ));
        return Err(Failed);
    }
    if let Some(arg) = arg {
        *compl_arg = Some(cstr::owned(cstr::until_nul(arg)));
    }
    Ok(())
}

/// Everything one attribute can change about a command being defined.
pub(super) struct Attributes<'a> {
    pub(super) argt: &'a mut ExArgt,
    pub(super) def: &'a mut c_int,
    pub(super) buffer_local: &'a mut bool,
    pub(super) complp: &'a mut ExpandContext,
    pub(super) compl_arg: &'a mut Option<CString>,
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
/// The attribute is the first `len` bytes of `attr`, which runs on to the
/// end of the command line: a bad `-complete=` value is reported with the
/// rest of the line after it, as upstream's `%s` of a pointer into the line
/// shows it.
pub(super) fn uc_scan_attr(attr: &[u8], len: usize, into: Attributes) -> Result<(), Failed> {
    if len == 0 {
        emsg(gettext(c"E175: No attribute specified"));
        return Err(Failed);
    }
    let typed = &attr[..len];

    // The flag attributes take no value.
    if abbreviates(typed, "bang") {
        *into.argt |= ExArgt::BANG;
        return Ok(());
    } else if abbreviates(typed, "buffer") {
        *into.buffer_local = true;
        return Ok(());
    } else if abbreviates(typed, "register") {
        *into.argt |= ExArgt::REGSTR;
        return Ok(());
    } else if abbreviates(typed, "keepscript") {
        *into.argt |= ExArgt::KEEPSCRIPT;
        return Ok(());
    } else if abbreviates(typed, "bar") {
        *into.argt |= ExArgt::TRLBAR;
        return Ok(());
    }

    // Everything else is `name=value`; a bare name leaves `value` unset,
    // which `-range` and `-count` accept and the other two reject.
    let (name, value) = match typed.iter().position(|&b| b == b'=') {
        Some(eq) => (&typed[..eq], Some(eq + 1)),
        None => (typed, None),
    };
    // The value as a slice, and as the C string it starts.
    let value_of = |at: usize| (&typed[at..], &attr[at..]);

    let outcome = if abbreviates(name, "nargs") {
        scan_nargs(value.map(|at| value_of(at).0), into.argt)
    } else if abbreviates(name, "range") {
        scan_range(
            value.map(|at| value_of(at).0),
            into.argt,
            into.def,
            into.addr_type_arg,
        )
    } else if abbreviates(name, "count") {
        scan_count(
            value.map(|at| value_of(at).0),
            into.argt,
            into.def,
            into.addr_type_arg,
        )
    } else if abbreviates(name, "complete") {
        match value.map(value_of) {
            None => Err(Bad::Missing(c"-complete")),
            Some((v, shown)) => parse_compl_arg(v, shown, into.complp, into.argt, into.compl_arg)
                .map_err(|Failed| Bad::Reported),
        }
    } else if abbreviates(name, "addr") {
        *into.argt |= ExArgt::RANGE;
        match value.map(value_of) {
            None => Err(Bad::Missing(c"-addr")),
            Some((v, shown)) => match parse_addr_type_arg(v, shown) {
                Err(_) => Err(Bad::Reported),
                Ok(addr_type) => {
                    *into.addr_type_arg = addr_type;
                    if addr_type != CmdAddr::Lines {
                        *into.argt |= ExArgt::ZEROR;
                    }
                    Ok(())
                }
            },
        }
    } else {
        let shown = msg_bytes(typed);
        semsg!("E181: Invalid attribute: {shown}");
        return Err(Failed);
    };

    let Err(bad) = outcome else { return Ok(()) };
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
            let what = what.to_string_lossy();
            semsg!("E179: Argument required for {what}");
        }
        Bad::Reported => {}
    }
    Err(Failed)
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
    // The walk stops at the end of `value`, a slice of the attribute: the
    // pointer walk it replaced read past it into the rest of the command
    // line (Miri).
    let (n, past) = getdigits_int_at(value, 0, true, 0);
    (past == value.len()).then_some(n)
}
