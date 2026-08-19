//! `:registers` and `:display`.
//!
//! One command over two lists: the 39 real slots, then the computed registers
//! `". ": "% "# "/ "=`, which have no slot and so are printed by hand.
//! [`dis_msg`] is the escaping every one of the second list goes through --
//! `skip_esc` drops the trailing `<Esc>` an Insert-mode recording always ends
//! with, so that `".` reads as the text that was typed.
//!
//! Every preview is cut off at the window width rather than wrapped, and
//! `os_breakcheck` runs per register so that CTRL-C stops a long listing.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::*;
use crate::types::FAIL;

/// Print `p` as one line of `:registers`, stopping at the window width.
///
/// # Safety
/// `p` must be NUL-terminated.
unsafe fn dis_msg(mut p: *const c_char, skip_esc: bool) {
    unsafe {
        let mut n = Columns.get() - 6;
        while c_int::from(*p) != NUL
            // A recording's trailing <Esc> is noise, not content.
            && !(c_int::from(*p) == ESC && skip_esc && c_int::from(*p.add(1)) == NUL)
            && {
                n -= ptr2cells(p);
                n >= 0
            }
        {
            let l = utfc_ptr2len(p);
            if l > 1 {
                msg_outtrans_len(p, l, 0, false);
                p = p.offset(l as isize);
            } else {
                msg_outtrans_len(p, 1, 0, false);
                p = p.add(1);
            }
        }
        os_breakcheck();
    }
}

/// Print one computed register's line, if it has content and the `:registers`
/// argument asked for it.
///
/// # Safety
/// `text` must be null or NUL-terminated, and `label` a five-byte prefix.
unsafe fn dis_special(
    arg: *mut c_char,
    name: c_int,
    label: *const c_char,
    text: *mut c_char,
    skip_esc: bool,
) {
    unsafe {
        if text.is_null()
            || !(arg.is_null() || !vim_strchr(arg, name).is_null())
            || got_int.get()
            || message_filtered(text)
        {
            return;
        }
        msg_puts(label);
        dis_msg(text, skip_esc);
    }
}

/// Print one slot of `y_regs`.
///
/// # Safety
/// `yb` must point at a register whose `y_array` holds `y_size` strings.
unsafe fn dis_register(yb: *mut yankreg_T, name: c_int, type_0: c_int, hl_id: c_int) {
    unsafe {
        // 'msgfilter': show the register if any of its lines survives.
        let mut do_show = false;
        let mut j: size_t = 0;
        while !do_show && j < (*yb).y_size {
            do_show = !message_filtered((*(*yb).y_array.add(j)).data);
            j = j.wrapping_add(1);
        }
        if !do_show && (*yb).y_size != 0 {
            return;
        }

        msg_putchar('\n' as c_int);
        msg_puts(c"  ".as_ptr());
        msg_putchar(type_0);
        msg_puts(c"  ".as_ptr());
        msg_putchar('"' as c_int);
        msg_putchar(name);
        msg_puts(c"   ".as_ptr());

        // The content, cut off at the window width. A line break inside the
        // register shows as `^J`.
        let mut n = Columns.get() - 11;
        let mut j: size_t = 0;
        while j < (*yb).y_size && n > 1 {
            if j != 0 {
                msg_puts_hl(c"^J".as_ptr(), hl_id, false);
                n -= 2;
            }
            let mut p = (*(*yb).y_array.add(j)).data;
            while c_int::from(*p) != NUL && {
                n -= ptr2cells(p);
                n >= 0
            } {
                let clen = utfc_ptr2len(p);
                msg_outtrans_len(p, clen, 0, false);
                p = p.offset(clen as isize);
            }
            j = j.wrapping_add(1);
        }
        if n > 1 && (*yb).y_type == kMTLineWise {
            msg_puts_hl(c"^J".as_ptr(), hl_id, false);
        }
    }
}

/// `:registers` / `:display`.
///
/// The argument, if any, is a set of register names to restrict the listing
/// to.
///
/// # Safety
/// `eap` must be a live `exarg_T`. Queries the clipboard provider, and so
/// runs Lua.
pub unsafe fn ex_display(eap: *mut exarg_T) {
    unsafe {
        let mut arg = (*eap).arg;
        if !arg.is_null() && c_int::from(*arg) == NUL {
            arg = ::core::ptr::null_mut();
        }
        let hl_id = HLF_8;

        msg_ext_set_kind(c"list_cmd".as_ptr());
        msg_ext_skip_flush.set(true);
        msg_puts_title(gettext(c"\nType Name Content".as_ptr()));

        // -1 is the unnamed register, which aliases whichever slot was
        // written last.
        let mut i = -1;
        while i < NUM_REGISTERS && !got_int.get() {
            let name = get_register_name(i);
            if arg.is_null() || !vim_strchr(arg, name).is_null() {
                // Before `get_clipboard` below, because for `"*`/`"+` this
                // queries the provider itself, and upstream's order of the
                // two queries is what the messages depend on.
                let type_0 = match get_reg_type(name, ::core::ptr::null_mut()) {
                    kMTLineWise => 'l' as c_int,
                    kMTCharWise => 'c' as c_int,
                    _ => 'b' as c_int,
                };
                let mut yb = if i == -1 {
                    if y_previous.get().is_null() {
                        get_y_register(0)
                    } else {
                        y_previous.get()
                    }
                } else {
                    get_y_register(i)
                };
                clipboard::get_clipboard(name, &mut yb, true);

                // Don't show the register `:redir` is writing into: it would
                // record its own listing.
                let redir_target = name == mb_tolower(redir_reg.get())
                    || redir_reg.get() == '"' as c_int && yb == y_previous.get();
                if !redir_target && !(*yb).y_array.is_null() {
                    dis_register(yb, name, type_0, hl_id);
                    os_breakcheck();
                }
            }
            i += 1;
        }

        // The registers with no slot, in the order `:registers` documents.
        let insert = get_last_insert();
        dis_special(
            arg,
            '.' as c_int,
            c"\n  c  \".   ".as_ptr(),
            insert.data,
            true,
        );
        dis_special(
            arg,
            ':' as c_int,
            c"\n  c  \":   ".as_ptr(),
            last_cmdline.get(),
            false,
        );
        dis_special(
            arg,
            '%' as c_int,
            c"\n  c  \"%   ".as_ptr(),
            (*curbuf.get()).b_fname,
            false,
        );
        // `"#` is listed under the `%` argument, as upstream has it.
        if (arg.is_null() || !vim_strchr(arg, '%' as c_int).is_null()) && !got_int.get() {
            let mut fname: *mut c_char = ::core::ptr::null_mut();
            let mut dummy: linenr_T = 0;
            if buflist_name_nr(0, &raw mut fname, &raw mut dummy) != FAIL
                && !message_filtered(fname)
            {
                msg_puts(c"\n  c  \"#   ".as_ptr());
                dis_msg(fname, false);
            }
        }
        dis_special(
            arg,
            '/' as c_int,
            c"\n  c  \"/   ".as_ptr(),
            last_search_pat(),
            false,
        );
        dis_special(
            arg,
            '=' as c_int,
            c"\n  c  \"=   ".as_ptr(),
            expr_line.get(),
            false,
        );

        msg_ext_skip_flush.set(false);
    }
}
