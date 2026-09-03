//! Rebuilding a command *string* from the parsed pieces.
//!
//! [`build_cmdline_str`] is what `nvim_cmd` hands `execute_cmd` for the paths
//! that still want text: it writes the modifiers back in their canonical
//! order, then the range, the command name, the bang, the register and each
//! argument, recording where each one landed so `eap->args` can point into the
//! finished buffer.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::ascii::ascii_iswhite;
use crate::cstr;
use crate::memory::handoff::owned_cstr;
use crate::types::ExArgt;
use crate::winlayer::Ea;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// Nothing but spaces and tabs.
///
/// Upstream also breaks out of the scan on a NUL, which cannot happen:
/// `ascii_iswhite` has already answered false for one and returned.
pub(crate) unsafe fn string_iswhite(str: String_0) -> bool {
    for i in 0..str.len() {
        // SAFETY: `i` is below `len`, so the byte is inside the string.
        let byte = unsafe { *str.data().add(i) };
        if !ascii_iswhite(byte as c_int) {
            return false;
        }
    }
    true
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
fn concat_cmdmods(cmdline: &mut Vec<u8>, cmdmod: &cmdmod_T) {
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

pub(crate) unsafe fn build_cmdline_str(
    cmdlinep: *mut *mut c_char,
    eap: *mut exarg_T,
    cmdinfo: *mut CmdParseInfo,
    args: Array,
) {
    // SAFETY: the caller's promise -- `eap` is the command being built and
    // is live for the call.
    let mut eap = unsafe { Ea::new(eap) };
    let argc: size_t = args.size;
    // Upstream's `kv_resize(cmdline, 32)`: a size hint, nothing more.
    let mut cmdline: Vec<u8> = Vec::with_capacity(32);
    // SAFETY: `cmdinfo` is the caller's, live for the call.
    let cmdmod = unsafe { &(*cmdinfo).cmdmod };
    concat_cmdmods(&mut cmdline, cmdmod);

    if eap.argt.has(ExArgt::RANGE) {
        if eap.addr_count == 1 {
            let line2 = eap.line2;
            cmdline.extend_from_slice(format!("{line2}").as_bytes());
        } else if eap.addr_count > 1 {
            let (line1, line2) = (eap.line1, eap.line2);
            cmdline.extend_from_slice(format!("{line1},{line2}").as_bytes());
            // Only two of them made it into the string.
            eap.addr_count = 2;
        }
    }
    let cmdname_idx: size_t = cmdline.len();
    let cmd = eap.cmd;
    // SAFETY: `eap.cmd` is the command name, NUL-terminated.
    unsafe { cmdline_concat(&mut cmdline, cmd, cstr::bytes_at(cmd).len()) };
    if eap.argt.has(ExArgt::BANG) && eap.forceit != 0 {
        cmdline_concat_str(&mut cmdline, c"!");
    }
    if eap.argt.has(ExArgt::REGSTR) && eap.regname != 0 {
        // `%c`: the low byte of the register name, not its UTF-8 encoding.
        cmdline.push(b' ');
        cmdline.push(eap.regname as u8);
    }

    // Each argument is preceded by one space, which is what lets the
    // offsets below be recovered from the lengths alone.
    eap.argc = argc;
    eap.arglens = if argc > 0 {
        // SAFETY: `xcalloc` answers `argc` zeroed slots.
        unsafe { xcalloc(argc, size_of::<size_t>()) }.cast::<size_t>()
    } else {
        ptr::null_mut::<size_t>()
    };
    let argstart_idx: size_t = cmdline.len();
    let arglens = eap.arglens;
    for i in 0..argc {
        // SAFETY: `i` is below `size`, so the object is inside `items`.
        let s: String_0 = unsafe { *args.items.add(i) }
            .as_string()
            .expect("collect_args puts only Strings in the array");
        // SAFETY: `arglens` was allocated with `argc` slots.
        unsafe { *arglens.add(i) = s.len() };
        cmdline_concat_str(&mut cmdline, c" ");
        // SAFETY: `s` names its own bytes.
        unsafe { cmdline_concat(&mut cmdline, s.data(), s.len()) };
    }
    // Handed to the caller, who releases it with `xfree`; every pointer
    // below is into it, so it is taken over before any of them are made.
    // The terminator `owned_cstr` appends is where `arg` points when there
    // are no arguments.
    let end_idx = cmdline.len();
    let items = owned_cstr(cmdline);

    // SAFETY: `cmdname_idx` is an offset into the buffer just built.
    eap.cmd = unsafe { items.add(cmdname_idx) };
    eap.args = if argc > 0 {
        // SAFETY: `xcalloc` answers `argc` zeroed slots.
        unsafe { xcalloc(argc, size_of::<*mut c_char>()) }.cast::<*mut c_char>()
    } else {
        ptr::null_mut::<*mut c_char>()
    };
    let eap_args = eap.args;
    let mut offset: size_t = argstart_idx;
    for i in 0..argc {
        offset += 1;
        // SAFETY: both arrays have `argc` slots, and `offset` is inside the
        // buffer the arguments were written into.
        unsafe {
            *eap_args.add(i) = items.add(offset);
            offset += *arglens.add(i);
        }
    }
    eap.arg = if argc > 0 {
        // SAFETY: `args` has at least one slot, filled in above.
        unsafe { *eap_args }
    } else {
        // SAFETY: `end_idx` is where the terminator went.
        unsafe { items.add(end_idx) }
    };
    // SAFETY: `cmdlinep` is the caller's slot, which takes the buffer over.
    unsafe { *cmdlinep = items };

    // `:make`/`:grep` rewrite their own argument, and the rewrite has no
    // relation to the `args` array that was just built.
    let arg = eap.arg;
    // SAFETY: `eap` is the command being built and `cmdlinep` the caller's
    // slot, which `replace_makeprg` may reallocate.
    let p: *mut c_char = unsafe { replace_makeprg(eap.raw(), arg, cmdlinep) };
    if p != arg {
        eap.arg = p;
        // SAFETY: both arrays are this function's own allocations.
        unsafe {
            xfree(eap_args.cast());
            xfree(arglens.cast());
        }
        eap.args = ptr::null_mut::<*mut c_char>();
        eap.arglens = ptr::null_mut::<size_t>();
        eap.argc = 0;
    }
}
