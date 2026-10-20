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

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::charset::byte2cells;
use crate::cstr;
use crate::mbyte::{c_cells_at, c_cluster_len};
use crate::memory::XString;
use crate::strings::has_char;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};

use super::*;

/// Print `text`'s characters, up to its first NUL, for as long as they fit in
/// the `n` cells left, which it counts down -- below zero when a character
/// did not fit.
fn put_cut(text: &[u8], n: &mut c_int, skip_esc: bool) {
    let text = cstr::until_nul(text);
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        // A recording's trailing <Esc> is noise, not content.
        if skip_esc && rest.len() == 1 && c_int::from(rest[0]) == ESC {
            break;
        }
        // As `ptr2cells` measures the C string `text` is the bytes of.
        *n -= if rest[0] >= 0x80 {
            c_cells_at(rest)
        } else {
            byte2cells(c_int::from(rest[0]))
        };
        if *n < 0 {
            break;
        }
        // The character's bytes, never less than one.
        let len = c_cluster_len(rest).max(1);
        msg_display_bytes(&rest[..len], 0, false);
        at += len;
    }
}

/// Print `p` as one line of `:registers`, stopping at the window width.
fn dis_msg(p: &[u8], skip_esc: bool) {
    put_cut(p, &mut (Columns.get() - 6), skip_esc);
    os_breakcheck();
}

/// Print one computed register's line, if it has content and the `:registers`
/// argument asked for it.
fn dis_special(arg: Option<&CStr>, name: c_int, label: &CStr, text: Option<&CStr>, skip_esc: bool) {
    let Some(text) = text else {
        return;
    };
    if arg.is_some_and(|arg| !has_char(arg, name)) {
        return;
    }
    if got_int.get() || message_filtered(text) {
        return;
    }
    msg_str(label);
    dis_msg(text.to_bytes(), skip_esc);
}

/// Print one slot of `y_regs`, as it was when the listing reached it.
fn dis_register(yb: &YankReg, name: c_int, type_0: c_int, hl_id: c_int) {
    let lines = yb.lines.as_slice();

    // 'msgfilter': show the register if any of its lines survives.
    let do_show = lines.iter().any(|line| !message_filtered(line.as_cstr()));
    if !do_show && !lines.is_empty() {
        return;
    }

    msg_putchar('\n' as c_int);
    msg_str(c"  ");
    msg_putchar(type_0);
    msg_str(c"  ");
    msg_putchar('"' as c_int);
    msg_putchar(name);
    msg_str(c"   ");

    // The content, cut off at the window width. A line break inside the
    // register shows as `^J`.
    let mut n = Columns.get() - 11;
    for (j, line) in lines.iter().enumerate() {
        if n <= 1 {
            break;
        }
        if j != 0 {
            msg_str_hl(c"^J", hl_id, false);
            n -= 2;
        }
        put_cut(line, &mut n, false);
    }
    if n > 1 && yb.kind == kMTLineWise {
        msg_str_hl(c"^J", hl_id, false);
    }
}

/// `:registers` / `:display`.
///
/// The argument, if any, is a set of register names to restrict the listing
/// to.
pub fn ex_display(excmd: &mut ExArg) {
    // A copy of the filter: listing a register may run the clipboard
    // provider.
    let filter = XString::from_bytes(excmd.line.arg());
    let arg = (!filter.is_empty()).then(|| filter.as_cstr());
    let hl_id = HLF_8;

    msg_ext_set_kind(c"list_cmd");
    msg_ext_skip_flush.set(true);
    msg_title(gettext(c"\nType Name Content"));

    // -1 is the unnamed register, which aliases whichever slot was
    // written last.
    let mut i = -1;
    while i < NUM_REGISTERS && !got_int.get() {
        let name = get_register_name(i);
        if arg.is_none_or(|arg| has_char(arg, name)) {
            // Before `get_clipboard` below, because for `"*`/`"+` this
            // queries the provider itself, and upstream's order of the
            // two queries is what the messages depend on.
            let type_0 = match get_reg_type(name).0 {
                kMTLineWise => 'l' as c_int,
                kMTCharWise => 'c' as c_int,
                _ => 'b' as c_int,
            };
            let mut yb = if i == -1 {
                get_y_register(y_previous.get().unwrap_or(0))
            } else {
                get_y_register(i)
            };
            // This is the call that may run the clipboard provider's Lua.
            if let Some((slot, _)) = clipboard::get_clipboard(name, true) {
                yb = Reg::Slot(slot);
            }

            // Don't show the register `:redir` is writing into: it would
            // record its own listing.
            let redir_target = name == mb_tolower(redir_reg.get())
                || redir_reg.get() == '"' as c_int && Some(yb) == get_y_previous();
            // A share of the lines: printing can run `:redir`, which writes
            // a register.
            let reg = yb.snapshot();
            if !redir_target && reg.lines.is_set() {
                dis_register(&reg, name, type_0, hl_id);
                os_breakcheck();
            }
        }
        i += 1;
    }

    // The registers with no slot, in the order `:registers` documents.
    let special = |name, label, text: Option<&CStr>, skip_esc| {
        dis_special(arg, name, label, text, skip_esc);
    };
    let insert = get_last_insert();
    let insert =
        (!insert.is_null()).then(|| XString::from_bytes(cstr::until_nul(insert.as_bytes())));
    special(
        '.' as c_int,
        c"\n  c  \".   ",
        insert.as_ref().map(XString::as_cstr),
        true,
    );
    // A copy: listing it runs the message machinery.
    let last = last_cmdline.with(Clone::clone);
    special(
        ':' as c_int,
        c"\n  c  \":   ",
        last.as_ref().map(XString::as_cstr),
        false,
    );
    let current = Buf::current().name.shown().map(XString::from_cstr);
    special(
        '%' as c_int,
        c"\n  c  \"%   ",
        current.as_ref().map(XString::as_cstr),
        false,
    );

    // `"#` is listed under the `%` argument, as upstream has it.
    let want_alt = arg.is_none_or(|arg| has_char(arg, '%' as c_int));
    if want_alt && !got_int.get() {
        // A copy: the filter runs a regexp and the display the message
        // machinery.
        let alt = buflist_name_nr(0).and_then(|(alt, _)| alt.name.shown().map(XString::from_cstr));
        if let Some(fname) = alt
            && !message_filtered(fname.as_cstr())
        {
            msg_str(c"\n  c  \"#   ");
            dis_msg(&fname, false);
        }
    }

    // A copy, as the expression below.
    let last = last_search_pat();
    special(
        '/' as c_int,
        c"\n  c  \"/   ",
        last.as_ref().map(XString::as_cstr),
        false,
    );
    // A copy: listing a register may run a message callback that sets it.
    let expr = expr_line.with(Clone::clone);
    special(
        '=' as c_int,
        c"\n  c  \"=   ",
        expr.as_ref().map(XString::as_cstr),
        false,
    );

    msg_ext_skip_flush.set(false);
}
