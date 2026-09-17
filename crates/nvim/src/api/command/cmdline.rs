//! Rebuilding a command *string* from the parsed pieces.
//!
//! [`build_cmdline_str`] is what `nvim_cmd` hands `execute_cmd` for the paths
//! that still want text: it writes the modifiers back in their canonical
//! order, then the range, the command name, the bang, the register and each
//! argument, recording where each one landed so `eap->args` can point into the
//! finished buffer.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::ascii::ascii_iswhite;
use crate::types::{CmdLine, ExArgt};
use core::ffi::{CStr, c_char, c_int};

/// Nothing but spaces and tabs.
///
/// Upstream also breaks out of the scan on a NUL, which cannot happen:
/// `ascii_iswhite` has already answered false for one and returned.
///
/// # Safety
///
/// `str` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`.
pub(crate) fn string_iswhite(str: &String_0) -> bool {
    str.as_bytes()
        .iter()
        .all(|&byte| ascii_iswhite(c_int::from(byte)))
}

/// Append `len` bytes: upstream's `kv_concat_len(cmdline, src, len)`, which
/// c2rust expanded at all twenty-four of [`build_cmdline_str`]'s call sites,
/// ~40 lines apiece.
///
/// # Safety
/// `src` must point at `len` readable bytes.
unsafe fn cmdline_concat(cmdline: &mut Vec<u8>, src: *const c_char, len: size_t) {
    if len == 0 {
        return;
    }
    // SAFETY: the caller's bytes, which are not part of `cmdline`.
    cmdline.extend_from_slice(unsafe { core::slice::from_raw_parts(src.cast::<u8>(), len) });
}

/// [`cmdline_concat`] for a string literal: upstream's `kv_concat`.
fn cmdline_concat_str(cmdline: &mut Vec<u8>, s: &CStr) {
    cmdline.extend_from_slice(s.to_bytes());
}

/// Write out the `:silent`/`:vertical`/... prefixes in the order upstream
/// parses them back.
fn concat_cmdmods(cmdline: &mut Vec<u8>, cmdmod: &CmdMod) {
    if cmdmod.cmod_tab != 0 {
        let tab = cmdmod.cmod_tab - 1;
        cmdline.extend_from_slice(format!("{tab}tab ").as_bytes());
    }
    if cmdmod.cmod_verbose > 0 {
        let verbose = cmdmod.cmod_verbose - 1;
        cmdline.extend_from_slice(format!("{verbose}verbose ").as_bytes());
    }
    if cmdmod.cmod_flags.has(CmdModFlags::ERRSILENT) {
        cmdline_concat_str(cmdline, c"silent! ");
    } else if cmdmod.cmod_flags.has(CmdModFlags::SILENT) {
        cmdline_concat_str(cmdline, c"silent ");
    }
    if cmdmod.cmod_flags.has(CmdModFlags::UNSILENT) {
        cmdline_concat_str(cmdline, c"unsilent ");
    }
    // A switch over the *masked* value, so two placement bits at once
    // spell no modifier at all rather than the first of them.
    const ABOVE: c_int = WSP_ABOVE as c_int;
    const BELOW: c_int = WSP_BELOW as c_int;
    const TOP: c_int = WSP_TOP as c_int;
    const BOT: c_int = WSP_BOT as c_int;
    match cmdmod.cmod_split & (ABOVE | BELOW | TOP | BOT) {
        ABOVE => cmdline_concat_str(cmdline, c"aboveleft "),
        BELOW => cmdline_concat_str(cmdline, c"belowright "),
        TOP => cmdline_concat_str(cmdline, c"topleft "),
        BOT => cmdline_concat_str(cmdline, c"botright "),
        _ => {}
    }
    if cmdmod.cmod_split & WSP_VERT as c_int != 0 {
        cmdline_concat_str(cmdline, c"vertical ");
    }
    if cmdmod.cmod_split & WSP_HOR as c_int != 0 {
        cmdline_concat_str(cmdline, c"horizontal ");
    }
    for (mask, text) in [
        (CmdModFlags::SANDBOX, c"sandbox "),
        (CmdModFlags::NOAUTOCMD, c"noautocmd "),
        (CmdModFlags::BROWSE, c"browse "),
        (CmdModFlags::CONFIRM, c"confirm "),
        (CmdModFlags::HIDE, c"hide "),
        (CmdModFlags::KEEPALT, c"keepalt "),
        (CmdModFlags::KEEPJUMPS, c"keepjumps "),
        (CmdModFlags::KEEPMARKS, c"keepmarks "),
        (CmdModFlags::KEEPPATTERNS, c"keeppatterns "),
        (CmdModFlags::LOCKMARKS, c"lockmarks "),
        (CmdModFlags::NOSWAPFILE, c"noswapfile "),
    ] {
        if cmdmod.cmod_flags.has(mask) {
            cmdline_concat_str(cmdline, text);
        }
    }
}

/// Render the Dict back into a command line, and leave it in `excmd`.
///
/// Every argument is preceded by one space, which is what lets the offsets
/// the argument vector records be recovered from the lengths alone.
///
/// # Safety
///
/// `cmdinfo` must point at the command's `CmdParseInfo`, live and unaliased
/// for the call.
pub(crate) unsafe fn build_cmdline_str(excmd: &mut ExArg, cmdinfo: *mut CmdParseInfo, args: Array) {
    let argc: size_t = args.len();
    // Upstream's `kv_resize(cmdline, 32)`: a size hint, nothing more.
    let mut cmdline: Vec<u8> = Vec::with_capacity(32);
    // SAFETY: `cmdinfo` is the caller's, live for the call.
    let cmdmod = unsafe { &(*cmdinfo).cmdmod };
    concat_cmdmods(&mut cmdline, cmdmod);

    if excmd.argt.has(ExArgt::RANGE) {
        if excmd.addr_count == 1 {
            let line2 = excmd.line2;
            cmdline.extend_from_slice(format!("{line2}").as_bytes());
        } else if excmd.addr_count > 1 {
            let (line1, line2) = (excmd.line1, excmd.line2);
            cmdline.extend_from_slice(format!("{line1},{line2}").as_bytes());
            // Only two of them made it into the string.
            excmd.addr_count = 2;
        }
    }
    let cmdname_idx: size_t = cmdline.len();
    let name = excmd.line.cmd;
    cmdline.extend_from_slice(excmd.line.rest_of(name));
    if excmd.argt.has(ExArgt::BANG) && excmd.forceit {
        cmdline_concat_str(&mut cmdline, c"!");
    }
    if excmd.argt.has(ExArgt::REGSTR) && excmd.regname != 0 {
        // `%c`: the low byte of the register name, not its UTF-8 encoding.
        cmdline.push(b' ');
        cmdline.push(excmd.regname as u8);
    }

    let mut spans: Vec<(usize, usize)> = Vec::with_capacity(argc);
    for item in args.iter().take(argc) {
        let s = item
            .as_string()
            .expect("collect_args puts only Strings in the array");
        cmdline.push(b' ');
        spans.push((cmdline.len(), s.len()));
        // SAFETY: `s` names its own bytes.
        unsafe { cmdline_concat(&mut cmdline, s.data(), s.len()) };
    }
    // The terminator is where `arg` points when there are no arguments.
    let end_idx = cmdline.len();
    cmdline.push(0);

    excmd.line = CmdLine::from_vec(cmdline);
    excmd.line.cmd = cmdname_idx;
    excmd.line.arg = spans.first().map_or(end_idx, |&(at, _)| at);
    excmd.line.args = spans;

    // `:make`/`:grep` rewrite their own argument, and the rewrite has no
    // relation to the argument vector that was just built.
    if replace_makeprg(excmd) {
        excmd.line.args.clear();
    }
}
