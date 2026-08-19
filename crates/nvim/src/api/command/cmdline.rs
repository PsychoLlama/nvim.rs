//! Rebuilding a command *string* from the parsed pieces.
//!
//! [`build_cmdline_str`] is what `nvim_cmd` hands `execute_cmd` for the paths
//! that still want text: it writes the modifiers back in their canonical
//! order, then the range, the command name, the bang, the register and each
//! argument, recording where each one landed so `eap->args` can point into the
//! finished buffer.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::ExArgt;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::{mem, ptr};

/// Nothing but spaces and tabs.
///
/// Upstream also breaks out of the scan on a NUL, which cannot happen:
/// `ascii_iswhite` has already answered false for one and returned.
pub(crate) unsafe fn string_iswhite(str: String_0) -> bool {
    unsafe {
        for i in 0..str.size {
            if !ascii_iswhite(*str.data.add(i) as c_int) {
                return false;
            }
        }
        true
    }
}

/// Append `len` bytes to a [`StringBuilder`], growing it to the next power
/// of two when they do not fit: upstream's `kv_concat_len(cmdline, src,
/// len)`.  c2rust expanded that macro at all twenty-four of
/// [`build_cmdline_str`]'s call sites, ~40 lines apiece.
///
/// # Safety
/// `cmdline` points at a live builder and `src` at `len` readable bytes.
unsafe fn cmdline_concat(cmdline: *mut StringBuilder, src: *const c_char, len: size_t) {
    unsafe {
        if len == 0 {
            return;
        }
        if (*cmdline).capacity < (*cmdline).size + len {
            let mut capacity = (*cmdline).size + len - 1;
            capacity |= capacity >> 1;
            capacity |= capacity >> 2;
            capacity |= capacity >> 4;
            capacity |= capacity >> 8;
            capacity |= capacity >> 16;
            (*cmdline).capacity = capacity + 1;
            (*cmdline).items =
                xrealloc((*cmdline).items.cast(), (*cmdline).capacity).cast::<c_char>();
        }
        debug_assert!(!(*cmdline).items.is_null());
        memcpy(
            (*cmdline).items.add((*cmdline).size).cast(),
            src.cast::<c_void>(),
            len,
        );
        (*cmdline).size += len;
    }
}

/// [`cmdline_concat`] for a string literal: upstream's `kv_concat`.
///
/// # Safety
/// `cmdline` points at a live builder.
unsafe fn cmdline_concat_str(cmdline: *mut StringBuilder, s: &CStr) {
    unsafe { cmdline_concat(cmdline, s.as_ptr(), s.count_bytes()) }
}

/// Write out the `:silent`/`:vertical`/... prefixes in the order upstream
/// parses them back.
unsafe fn concat_cmdmods(cmdline: *mut StringBuilder, cmdmod: &cmdmod_T) {
    unsafe {
        if cmdmod.cmod_tab != 0 {
            kv_do_printf(cmdline, c"%dtab ".as_ptr(), cmdmod.cmod_tab - 1);
        }
        if cmdmod.cmod_verbose > 0 {
            kv_do_printf(cmdline, c"%dverbose ".as_ptr(), cmdmod.cmod_verbose - 1);
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
}

pub(crate) unsafe fn build_cmdline_str(
    cmdlinep: *mut *mut c_char,
    eap: *mut exarg_T,
    cmdinfo: *mut CmdParseInfo,
    args: Array,
) {
    unsafe {
        let argc: size_t = args.size;
        let mut cmdline: StringBuilder = StringBuilder {
            size: 0,
            capacity: 32,
            items: xrealloc(ptr::null_mut(), 32).cast::<c_char>(),
        };
        concat_cmdmods(&raw mut cmdline, &(*cmdinfo).cmdmod);

        if (*eap).argt.has(ExArgt::RANGE) {
            if (*eap).addr_count == 1 {
                kv_do_printf(&raw mut cmdline, c"%d".as_ptr(), (*eap).line2);
            } else if (*eap).addr_count > 1 {
                kv_do_printf(
                    &raw mut cmdline,
                    c"%d,%d".as_ptr(),
                    (*eap).line1,
                    (*eap).line2,
                );
                // Only two of them made it into the string.
                (*eap).addr_count = 2;
            }
        }
        let cmdname_idx: size_t = cmdline.size;
        cmdline_concat(&raw mut cmdline, (*eap).cmd, strlen((*eap).cmd));
        if (*eap).argt.has(ExArgt::BANG) && (*eap).forceit != 0 {
            cmdline_concat_str(&raw mut cmdline, c"!");
        }
        if (*eap).argt.has(ExArgt::REGSTR) && (*eap).regname != 0 {
            kv_do_printf(&raw mut cmdline, c" %c".as_ptr(), (*eap).regname);
        }

        // Each argument is preceded by one space, which is what lets the
        // offsets below be recovered from the lengths alone.
        (*eap).argc = argc;
        (*eap).arglens = if argc > 0 {
            xcalloc(argc, mem::size_of::<size_t>()).cast::<size_t>()
        } else {
            ptr::null_mut::<size_t>()
        };
        let argstart_idx: size_t = cmdline.size;
        for i in 0..argc {
            let s: String_0 = (*args.items.add(i)).data.string;
            *(*eap).arglens.add(i) = s.size;
            cmdline_concat_str(&raw mut cmdline, c" ");
            cmdline_concat(&raw mut cmdline, s.data, s.size);
        }
        // The NUL is part of `size`, so that `arg` below can point at it.
        cmdline_concat(&raw mut cmdline, c"".as_ptr(), 1);

        (*eap).cmd = cmdline.items.add(cmdname_idx);
        (*eap).args = if argc > 0 {
            xcalloc(argc, mem::size_of::<*mut c_char>()).cast::<*mut c_char>()
        } else {
            ptr::null_mut::<*mut c_char>()
        };
        let mut offset: size_t = argstart_idx;
        for i in 0..argc {
            offset += 1;
            *(*eap).args.add(i) = cmdline.items.add(offset);
            offset += *(*eap).arglens.add(i);
        }
        (*eap).arg = if argc > 0 {
            *(*eap).args
        } else {
            cmdline.items.add(cmdline.size - 1)
        };
        *cmdlinep = cmdline.items;

        // `:make`/`:grep` rewrite their own argument, and the rewrite has no
        // relation to the `args` array that was just built.
        let p: *mut c_char = replace_makeprg(eap, (*eap).arg, cmdlinep);
        if p != (*eap).arg {
            (*eap).arg = p;
            xfree((*eap).args.cast());
            (*eap).args = ptr::null_mut::<*mut c_char>();
            xfree((*eap).arglens.cast());
            (*eap).arglens = ptr::null_mut::<size_t>();
            (*eap).argc = 0;
        }
    }
}
