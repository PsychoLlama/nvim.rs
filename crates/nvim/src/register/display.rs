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

use crate::winlayer::Buf;
use core::ffi::{c_char, c_int};

use super::*;
use crate::types::NUL;

/// Print `p` as one line of `:registers`, stopping at the window width.
///
/// # Safety
/// `p` must be NUL-terminated.
unsafe fn dis_msg(mut p: *const c_char, skip_esc: bool) {
    let mut n = Columns.get() - 6;
    loop {
        // SAFETY: `p` starts at the caller's NUL-terminated string and only
        // ever steps over a whole character, so it stays inside it.
        let c = unsafe { c_int::from(*p) };
        // A recording's trailing <Esc> is noise, not content.  The `c == ESC`
        // test in front is what proves `p.add(1)` still inside the string.
        // SAFETY: as above.
        if c == NUL || (c == ESC && skip_esc && unsafe { c_int::from(*p.add(1)) } == NUL) {
            break;
        }
        // SAFETY: `p` points at a character of a NUL-terminated string.
        n -= unsafe { ptr2cells(p) };
        if n < 0 {
            break;
        }
        // SAFETY: as above; the answer covers one whole character, and is
        // never less than one byte.
        let l = unsafe { utfc_ptr2len(p) }.max(1);
        // SAFETY: `l` bytes from `p` are that character.
        unsafe { msg_outtrans_len(p, l, 0, false) };
        // SAFETY: stepping over a whole character stays within the string.
        p = unsafe { p.offset(l as isize) };
    }
    os_breakcheck();
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
    if text.is_null() {
        return;
    }
    // SAFETY: `arg`, tested non-null, is the NUL-terminated `:registers`
    // argument.
    if !arg.is_null() && unsafe { vim_strchr(arg, name) }.is_null() {
        return;
    }
    // SAFETY: `text` is NUL-terminated, tested non-null above.
    if got_int.get() || unsafe { message_filtered(text) } {
        return;
    }
    // SAFETY: `label` is the caller's NUL-terminated prefix.
    unsafe { msg_puts(label) };
    // SAFETY: `text` is NUL-terminated.
    unsafe { dis_msg(text, skip_esc) };
}

/// Print one slot of `y_regs`.
///
/// # Safety
/// `yb` must point at a register whose `y_array` holds `y_size` strings.
unsafe fn dis_register(yb: *mut yankreg_T, name: c_int, type_0: c_int, hl_id: c_int) {
    // Both of these re-read the register every time, as the transpiled code
    // did: printing can run `:redir`, which writes into a register.
    //
    // SAFETY (both): the caller promises `yb` is a live register whose
    // `y_array` holds `y_size` NUL-terminated strings, so any index below
    // `y_size` names one of them.
    let size = || unsafe { (*yb).y_size };
    let line = |j: size_t| unsafe { (*(*yb).y_array.add(j)).data() };

    // 'msgfilter': show the register if any of its lines survives.
    let mut do_show = false;
    let mut j: size_t = 0;
    while !do_show && j < size() {
        // SAFETY: `j` is below `y_size`, so the line is NUL-terminated.
        do_show = !unsafe { message_filtered(line(j)) };
        j = j.wrapping_add(1);
    }
    if !do_show && size() != 0 {
        return;
    }

    // SAFETY: main thread; every argument is a NUL-terminated literal or a
    // single character to print.
    unsafe { msg_putchar('\n' as c_int) };
    unsafe { msg_puts(c"  ".as_ptr()) };
    unsafe { msg_putchar(type_0) };
    unsafe { msg_puts(c"  ".as_ptr()) };
    unsafe { msg_putchar('"' as c_int) };
    unsafe { msg_putchar(name) };
    unsafe { msg_puts(c"   ".as_ptr()) };

    // The content, cut off at the window width. A line break inside the
    // register shows as `^J`.
    let mut n = Columns.get() - 11;
    let mut j: size_t = 0;
    while j < size() && n > 1 {
        if j != 0 {
            // SAFETY: a NUL-terminated literal.
            unsafe { msg_puts_hl(c"^J".as_ptr(), hl_id, false) };
            n -= 2;
        }
        let mut p = line(j);
        loop {
            // SAFETY: `p` starts at a NUL-terminated line of the register and
            // only ever steps over a whole character.
            if unsafe { c_int::from(*p) } == NUL {
                break;
            }
            // SAFETY: as above.
            n -= unsafe { ptr2cells(p) };
            if n < 0 {
                break;
            }
            // SAFETY: as above; the answer covers one whole character.
            let clen = unsafe { utfc_ptr2len(p) };
            // SAFETY: `clen` bytes from `p` are that character.
            unsafe { msg_outtrans_len(p, clen, 0, false) };
            // SAFETY: stepping over a whole character stays within the line.
            p = unsafe { p.offset(clen as isize) };
        }
        j = j.wrapping_add(1);
    }
    // SAFETY: `yb` is still the caller's live register.
    if n > 1 && unsafe { (*yb).y_type } == kMTLineWise {
        // SAFETY: a NUL-terminated literal.
        unsafe { msg_puts_hl(c"^J".as_ptr(), hl_id, false) };
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
    // SAFETY: the caller promises a live `exarg_T`, whose `arg` is null or a
    // NUL-terminated string.
    let mut arg = unsafe { (*eap).arg };
    // SAFETY: as above, and tested non-null.
    if !arg.is_null() && unsafe { c_int::from(*arg) } == NUL {
        arg = ::core::ptr::null_mut();
    }
    let hl_id = HLF_8;

    // SAFETY (both): main thread, printing NUL-terminated literals.
    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    msg_ext_skip_flush.set(true);
    unsafe { msg_puts_title(gettext(c"\nType Name Content").as_ptr()) };

    // -1 is the unnamed register, which aliases whichever slot was
    // written last.
    let mut i = -1;
    while i < NUM_REGISTERS && !got_int.get() {
        let name = get_register_name(i);
        // SAFETY: `arg` is null or the NUL-terminated `:registers` argument.
        if arg.is_null() || !unsafe { vim_strchr(arg, name) }.is_null() {
            // Before `get_clipboard` below, because for `"*`/`"+` this
            // queries the provider itself, and upstream's order of the
            // two queries is what the messages depend on.
            //
            // SAFETY: main thread; a null `expr` asks for no expression back.
            let type_0 = match unsafe { get_reg_type(name, ::core::ptr::null_mut()) } {
                kMTLineWise => 'l' as c_int,
                kMTCharWise => 'c' as c_int,
                _ => 'b' as c_int,
            };
            let mut yb = if i == -1 {
                if y_previous.get().is_null() {
                    // SAFETY: 0 is a register index.
                    unsafe { get_y_register(0) }
                } else {
                    y_previous.get()
                }
            } else {
                // SAFETY: `i` is below `NUM_REGISTERS`.
                unsafe { get_y_register(i) }
            };
            // SAFETY: main thread, with a live slot to fill in; this is the
            // call that may run the clipboard provider's Lua.
            unsafe { clipboard::get_clipboard(name, &mut yb, true) };

            // Don't show the register `:redir` is writing into: it would
            // record its own listing.
            let redir_target = name == mb_tolower(redir_reg.get())
                || redir_reg.get() == '"' as c_int && yb == y_previous.get();
            // SAFETY: `yb` is a live register slot.
            if !redir_target && !unsafe { (*yb).y_array }.is_null() {
                // SAFETY: a live slot whose `y_array` holds `y_size` strings.
                unsafe { dis_register(yb, name, type_0, hl_id) };
                os_breakcheck();
            }
        }
        i += 1;
    }

    // The registers with no slot, in the order `:registers` documents.
    //
    // SAFETY: every `text` below is null or a NUL-terminated string, and
    // every `label` one of this module's five-byte literals.
    let special = |name, label, text, skip_esc| unsafe {
        dis_special(arg, name, label, text, skip_esc);
    };
    // SAFETY: main thread; the answer owns its own copy of the text.
    let insert = unsafe { get_last_insert() };
    special('.' as c_int, c"\n  c  \".   ".as_ptr(), insert.data(), true);
    special(
        ':' as c_int,
        c"\n  c  \":   ".as_ptr(),
        last_cmdline.get(),
        false,
    );
    special(
        '%' as c_int,
        c"\n  c  \"%   ".as_ptr(),
        cur_buf().b_fname,
        false,
    );

    // `"#` is listed under the `%` argument, as upstream has it.
    // SAFETY: `arg` is null or the NUL-terminated `:registers` argument.
    let want_alt = arg.is_null() || !unsafe { vim_strchr(arg, '%' as c_int) }.is_null();
    if want_alt && !got_int.get() {
        let mut fname: *mut c_char = ::core::ptr::null_mut();
        let mut dummy: linenr_T = 0;
        // SAFETY: both out-parameters are writable locals.
        let named = unsafe { buflist_name_nr(0, &raw mut fname, &raw mut dummy) }.is_ok();
        // SAFETY: on success `fname` is the alternate file's name, which is
        // NUL-terminated.
        if named && !unsafe { message_filtered(fname) } {
            // SAFETY: a NUL-terminated literal, then the name.
            unsafe { msg_puts(c"\n  c  \"#   ".as_ptr()) };
            unsafe { dis_msg(fname, false) };
        }
    }

    special(
        '/' as c_int,
        c"\n  c  \"/   ".as_ptr(),
        last_search_pat(),
        false,
    );
    special(
        '=' as c_int,
        c"\n  c  \"=   ".as_ptr(),
        expr_line.get(),
        false,
    );

    msg_ext_skip_flush.set(false);
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
