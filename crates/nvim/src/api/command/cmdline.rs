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
use crate::types::ExArgt;
use crate::winlayer::Ea;
use core::ffi::{CStr, c_char, c_int, c_void};
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

/// Append `len` bytes to a [`StringBuilder`], growing it to the next power
/// of two when they do not fit: upstream's `kv_concat_len(cmdline, src,
/// len)`.  c2rust expanded that macro at all twenty-four of
/// [`build_cmdline_str`]'s call sites, ~40 lines apiece.
///
/// # Safety
/// `src` must point at `len` readable bytes.
unsafe fn cmdline_concat(cmdline: &mut StringBuilder, src: *const c_char, len: size_t) {
    if len == 0 {
        return;
    }
    if cmdline.capacity < cmdline.size + len {
        let mut capacity = cmdline.size + len - 1;
        capacity |= capacity >> 1;
        capacity |= capacity >> 2;
        capacity |= capacity >> 4;
        capacity |= capacity >> 8;
        capacity |= capacity >> 16;
        cmdline.capacity = capacity + 1;
        // SAFETY: `items` is null with a zero capacity, or the allocation
        // this function made last time.
        cmdline.items = unsafe { xrealloc(cmdline.items.cast(), cmdline.capacity) }.cast();
    }
    debug_assert!(!cmdline.items.is_null());
    let dest = cmdline.items;
    let size = cmdline.size;
    // SAFETY: the grow above left room for `len` bytes past `size`, and the
    // caller promised `src` holds that many.
    unsafe { memcpy(dest.add(size).cast(), src.cast::<c_void>(), len) };
    cmdline.size += len;
}

/// [`cmdline_concat`] for a string literal: upstream's `kv_concat`.
fn cmdline_concat_str(cmdline: &mut StringBuilder, s: &CStr) {
    // SAFETY: a `CStr` holds exactly `count_bytes` readable bytes.
    unsafe { cmdline_concat(cmdline, s.as_ptr(), s.count_bytes()) };
}

/// Write out the `:silent`/`:vertical`/... prefixes in the order upstream
/// parses them back.
fn concat_cmdmods(cmdline: &mut StringBuilder, cmdmod: &cmdmod_T) {
    if cmdmod.cmod_tab != 0 {
        let tab = cmdmod.cmod_tab - 1;
        // SAFETY: `cmdline` is the caller's builder; the format takes the
        // one integer it is given.
        unsafe { kv_do_printf(cmdline, c"%dtab ".as_ptr(), tab) };
    }
    if cmdmod.cmod_verbose > 0 {
        let verbose = cmdmod.cmod_verbose - 1;
        // SAFETY: as above.
        unsafe { kv_do_printf(cmdline, c"%dverbose ".as_ptr(), verbose) };
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
    // SAFETY: a null pointer with a zero size asks `xrealloc` for a fresh
    // allocation.
    let items = unsafe { xrealloc(ptr::null_mut(), 32) }.cast::<c_char>();
    let mut cmdline: StringBuilder = StringBuilder {
        size: 0,
        capacity: 32,
        items,
    };
    // SAFETY: `cmdinfo` is the caller's, live for the call.
    let cmdmod = unsafe { &(*cmdinfo).cmdmod };
    concat_cmdmods(&mut cmdline, cmdmod);

    if eap.argt.has(ExArgt::RANGE) {
        if eap.addr_count == 1 {
            let line2 = eap.line2;
            // SAFETY: the format takes the one integer it is given.
            unsafe { kv_do_printf(&mut cmdline, c"%d".as_ptr(), line2) };
        } else if eap.addr_count > 1 {
            let (line1, line2) = (eap.line1, eap.line2);
            // SAFETY: the format takes the two integers it is given.
            unsafe { kv_do_printf(&mut cmdline, c"%d,%d".as_ptr(), line1, line2) };
            // Only two of them made it into the string.
            eap.addr_count = 2;
        }
    }
    let cmdname_idx: size_t = cmdline.size;
    let cmd = eap.cmd;
    // SAFETY: `eap.cmd` is the command name, NUL-terminated.
    unsafe { cmdline_concat(&mut cmdline, cmd, cstr::bytes_at(cmd).len()) };
    if eap.argt.has(ExArgt::BANG) && eap.forceit != 0 {
        cmdline_concat_str(&mut cmdline, c"!");
    }
    if eap.argt.has(ExArgt::REGSTR) && eap.regname != 0 {
        let regname = eap.regname;
        // SAFETY: the format takes the one character it is given.
        unsafe { kv_do_printf(&mut cmdline, c" %c".as_ptr(), regname) };
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
    let argstart_idx: size_t = cmdline.size;
    let arglens = eap.arglens;
    for i in 0..argc {
        // SAFETY: `i` is below `size`, so the object is inside `items`, and
        // the tag says its payload is the string.
        let s: String_0 = unsafe { (*args.items.add(i)).data.string };
        // SAFETY: `arglens` was allocated with `argc` slots.
        unsafe { *arglens.add(i) = s.len() };
        cmdline_concat_str(&mut cmdline, c" ");
        // SAFETY: `s` names its own bytes.
        unsafe { cmdline_concat(&mut cmdline, s.data(), s.len()) };
    }
    // The NUL is part of `size`, so that `arg` below can point at it.
    // SAFETY: the literal's terminator is the one byte being copied.
    unsafe { cmdline_concat(&mut cmdline, c"".as_ptr(), 1) };

    // SAFETY: `cmdname_idx` is an offset into the buffer just built.
    eap.cmd = unsafe { cmdline.items.add(cmdname_idx) };
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
            *eap_args.add(i) = cmdline.items.add(offset);
            offset += *arglens.add(i);
        }
    }
    eap.arg = if argc > 0 {
        // SAFETY: `args` has at least one slot, filled in above.
        unsafe { *eap_args }
    } else {
        // SAFETY: `size` counts the terminator written above.
        unsafe { cmdline.items.add(cmdline.size - 1) }
    };
    // SAFETY: `cmdlinep` is the caller's slot, which takes the buffer over.
    unsafe { *cmdlinep = cmdline.items };

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
