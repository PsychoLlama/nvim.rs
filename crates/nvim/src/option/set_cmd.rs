//! The `:set` family: parsing one argument into an option, a prefix, an
//! operator and a new value.
//!
//! An argument is `[no|inv]name[?|!|&|<|[+^-]=value]`, and the parse is
//! left to right: the prefix, the name, an optional operator, then the
//! character after it decides everything else. [`do_one_set_option`] is
//! that decision tree; [`get_option_newval`] turns what is left into a
//! value of the option's own type.
//!
//! Error reporting is deferred: a rejected argument is reported by
//! [`do_set`], with the text of the argument appended, which is why the
//! parse hands messages back through an out-parameter rather than
//! reporting them where they are found.
//!
//! The command line is a slice, walked by offset; its end reads as the
//! terminator the C walked to.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::charset::Str2NrBases;
use crate::charset::{str2nr_in, trans_characters_in};
use crate::cstr;
use crate::keycodes::Ctrl_V;
use crate::keycodes::Key;
use crate::keycodes::special_key_at;
use crate::optionstr::OptString;
use crate::types::CmdIdx;
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_int};

use super::{OptSlot, SetOp, boolean_optval, option_last_set, ui_refresh_options};
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::drawscreen::{UPD_CLEAR, redraw_all_later};
use crate::eval::last_set_msg;
use crate::ex_getln::gotocmdline;
use crate::guard::Suppress;
use crate::guard::sandbox;
use crate::memory::XString;
use crate::message::state::info_message;
use crate::message::{e_invarg, e_sandbox, e_trailing};
use crate::message::{emsg, msg_ext_set_kind, msg_putchar};
use crate::option::vars::{p_mle, p_verbose};
use crate::options::{kOptFoldmethod, kOptInvalid, kOptWildchar, kOptWildcharm, kOptWrap};
use crate::os::cshim::gettext_owned;
use crate::startup::silent_mode;
use crate::types::{
    ExArg, Failed, IOSIZE, NUL, OptError, OptIndex, OptInt, OptVal, OptionSetFlags, ScriptId,
    uint32_t,
};

use super::{
    FSK_KEEP_X_KEY, FSK_KEYCODE, FSK_SIMPLIFY, OP_ADDING, OP_NONE, OP_PREPENDING, OP_REMOVING,
    didset_options, didset_options2, get_option, get_option_default, get_option_value, get_varp,
    get_varp_scope, is_tty_option, kOptFlagMLE, kOptFlagSecure, kOptScopeBuf, kOptScopeWin,
    kOptValTypeBoolean, kOptValTypeNumber, option_has_scope, option_has_type,
    option_is_global_local, option_is_window_local, option_scope_idx, option_var, set_option,
    set_options_default, showoneopt, showoptions, slot_value, stropt_get_newval,
    unset_option_local_value,
};

/// The messages the parse reports. They go back to [`do_set`] rather than
/// being emitted here, so that the offending argument can be appended.
const E_UNKNOWN_OPTION: &CStr = c"E518: Unknown option";
const E_NOT_ALLOWED_IN_MODELINE: &CStr = c"E520: Not allowed in a modeline";
const E_MODELINE_NEEDS_MODELINEEXPR: &CStr =
    c"E992: Not allowed in a modeline when 'modelineexpr' is off";
const E_NUMBER_REQUIRED_AFTER_EQUAL: &CStr = c"E521: Number required after =";

/// The room upstream's `errbuf` gave a message a `set_option` rejection
/// came back with, terminator included: the report carries no more of it.
const ERRBUF_LEN: usize = 80;

/// What `no` or `inv` in front of a boolean option's name asks for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Prefix {
    None,
    No,
    Inv,
}

/// The byte at `i` of the command line, its end reading as the terminator.
fn at(arg: &[u8], i: usize) -> u8 {
    cstr::byte_at(arg, i)
}

/// Whether the byte at `i` is a space or a tab.
fn white_at(arg: &[u8], i: usize) -> bool {
    ascii_iswhite(c_int::from(at(arg, i)))
}

/// `:set`, `:setlocal` and `:setglobal`.
pub(crate) fn ex_set(excmd: &mut ExArg) {
    let mut flags = match excmd.cmdidx {
        CmdIdx::setlocal => OptionSetFlags::LOCAL,
        CmdIdx::setglobal => OptionSetFlags::GLOBAL,
        _ => OptionSetFlags::NONE,
    };
    // `:set!` lists one option per line.
    if excmd.forceit {
        flags |= OptionSetFlags::ONECOLUMN;
    }
    let _ = do_set(excmd.line.arg(), flags);
}

/// The operator at `arg[p..]`, if the two characters there are one.
fn get_op(arg: &[u8], p: usize) -> SetOp {
    if at(arg, p) == NUL as u8 || at(arg, p + 1) != b'=' {
        return OP_NONE;
    }
    match at(arg, p) {
        b'+' => OP_ADDING,
        b'^' => OP_PREPENDING,
        b'-' => OP_REMOVING,
        _ => OP_NONE,
    }
}

/// The prefix at `arg[*p..]`, advancing past it.
fn get_option_prefix(arg: &[u8], p: &mut usize) -> Prefix {
    for (spelling, prefix) in [(&b"no"[..], Prefix::No), (b"inv", Prefix::Inv)] {
        if arg[*p..].starts_with(spelling) {
            *p += spelling.len();
            return prefix;
        }
    }
    Prefix::None
}

/// Whether this option may be set at all, here and now: some are refused in
/// a modeline or in the sandbox, and `:setlocal`/`:setglobal` on a modeline
/// only reach the scopes the caller asked for.
fn validate_opt_idx(
    win: Win,
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
    flags: uint32_t,
    prefix: Prefix,
    errmsg: &mut Option<OptError>,
) -> Result<(), Failed> {
    if prefix != Prefix::None && !option_has_type(opt_idx, kOptValTypeBoolean) {
        *errmsg = Some(e_invarg.into());
        return Err(Failed);
    }
    // A `:set` sweeping over windows or buffers only wants its own kind.
    if opt_flags.has(OptionSetFlags::WINONLY) && !option_is_window_local(opt_idx) {
        return Err(Failed);
    }
    if opt_flags.has(OptionSetFlags::NOWIN) && option_is_window_local(opt_idx) {
        return Err(Failed);
    }
    if opt_flags.has(OptionSetFlags::MODELINE) {
        if flags & kOptFlagSecure as uint32_t != 0 {
            *errmsg = Some(E_NOT_ALLOWED_IN_MODELINE.into());
            return Err(Failed);
        }
        if flags & kOptFlagMLE as uint32_t != 0 && !p_mle() {
            *errmsg = Some(E_MODELINE_NEEDS_MODELINEEXPR.into());
            return Err(Failed);
        }
        // A modeline must not undo what `:diffthis` set up.
        if win.w_onebuf_opt.wo_diff != 0 && (opt_idx == kOptFoldmethod || opt_idx == kOptWrap) {
            return Err(Failed);
        }
    }
    if sandbox.get() != 0 && flags & kOptFlagSecure as uint32_t != 0 {
        *errmsg = Some(e_sandbox.into());
        return Err(Failed);
    }
    Ok(())
}

/// How long the terminal option's name at the start of `text` is, or `None`
/// when `text` does not start with one. `term` and `ttytype` are spelled
/// out and must be all of `text`; the rest are `t_xx`, optionally wrapped in
/// `<>`.
///
/// The end of the slice reads as the terminator, as it did when this walked
/// a C string.
pub(crate) fn tty_option_end(text: &[u8]) -> Option<usize> {
    if text == b"term" || text == b"ttytype" {
        return Some(text.len());
    }
    let at = |i: usize| cstr::byte_at(text, i);
    let delimit = at(0) == b'<';
    let mut p = usize::from(delimit);
    if at(p) == b't' && at(p + 1) == b'_' && at(p + 2) != NUL as u8 && at(p + 3) != NUL as u8 {
        p += 4;
    } else if delimit {
        while at(p) != NUL as u8 && at(p) != b'>' {
            p += 1;
        }
    }
    if delimit {
        if at(p) != b'>' {
            return None;
        }
        p += 1;
    }
    (p != 0).then_some(p)
}

/// How long the option name at the start of `text` is, and the option it
/// names. A terminal option ends where it ends but resolves to
/// `kOptInvalid`; anything that does not start with a letter is not a name
/// at all, and answers `None` with `kOptInvalid`.
pub(crate) fn option_end(text: &[u8]) -> (OptIndex, Option<usize>) {
    if let Some(end) = tty_option_end(text) {
        return (kOptInvalid, Some(end));
    }
    let len = text.iter().take_while(|b| b.is_ascii_alphabetic()).count();
    if len == 0 {
        return (kOptInvalid, None);
    }
    (super::find_option_len(&text[..len]), Some(len))
}

/// The value the character at `arg[*p]` -- `nextchar` -- and what follows
/// it ask for, in the option's own type, with `*p` past the value of a
/// string option.
///
/// `:set opt&` and `:set opt<` short-circuit; everything else is a value of
/// the option's declared type, with the operator applied to the old one.
#[allow(clippy::too_many_arguments)]
fn get_option_newval(
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
    prefix: Prefix,
    arg: &[u8],
    p: &mut usize,
    nextchar: u8,
    op: SetOp,
    flags: uint32_t,
    varp: OptSlot,
    errmsg: &mut Option<OptError>,
) -> OptVal {
    debug_assert!(!varp.is_none());

    // Setting the local value of a global-local option amends whatever
    // it is currently reading through, which may be the global value.
    let oldval_is_global = option_is_global_local(opt_idx) && opt_flags.has(OptionSetFlags::LOCAL);
    let from = if oldval_is_global {
        get_varp(opt_idx)
    } else {
        varp
    };

    // `:set opt&`. Deliberately `OptionSetFlags::GLOBAL` rather than
    // `opt_flags`, so that a `:setlocal opt&` on a global-local option gets
    // the real default rather than the unset marker.
    if nextchar == b'&' {
        return get_option_default(opt_idx, OptionSetFlags::GLOBAL);
    }
    // `:set opt<` resets to the global value; `:setlocal opt<` copies it
    // into the local one.
    if nextchar == b'<' {
        if option_is_global_local(opt_idx) && !opt_flags.has(OptionSetFlags::LOCAL) {
            let _ = unset_option_local_value(opt_idx);
        }
        return get_option_value(opt_idx, OptionSetFlags::GLOBAL);
    }

    if option_has_type(opt_idx, kOptValTypeBoolean) {
        let boolean = if nextchar == b'!' {
            // `:set opt!` inverts, leaving an unset global-local value
            // unset.
            slot_value(opt_idx, from).as_boolean().map(|b| !b)
        } else if prefix == Prefix::Inv {
            Some(varp.boolean_var().get() == 0)
        } else {
            Some(prefix != Prefix::No)
        };
        boolean_optval(boolean)
    } else if option_has_type(opt_idx, kOptValTypeNumber) {
        let oldval_num = from.number_var().get();
        let Some(newval_num) = take_number(opt_idx, &arg[(*p + 1).min(arg.len())..], errmsg) else {
            return OptVal::Nil;
        };
        let number = match op {
            OP_ADDING => oldval_num + newval_num,
            // `^=` on a number multiplies; there is nothing to prepend to.
            OP_PREPENDING => oldval_num * newval_num,
            OP_REMOVING => oldval_num - newval_num,
            _ => newval_num,
        };
        OptVal::Number(number)
    } else {
        // A copy: assembling the value expands the environment, which may
        // read other options.
        let origval = from.string_var().get();
        let mut op = op;
        let value_at = (*p + 1).min(arg.len());
        let (newval, taken) =
            stropt_get_newval(opt_idx, &arg[value_at..], varp, &origval, &mut op, flags);
        *p = value_at + taken;
        OptVal::from_xstring(newval)
    }
}

/// The number at the start of `arg`, or `None` with `errmsg` set.
///
/// 'wildchar' and 'wildcharm' hold a key rather than a count, so for those
/// two a `<xx>` name, a `^x` control spelling or any single non-digit is a
/// key; everything else is `[-]0-9` in any of the bases `vim_str2nr` reads.
fn take_number(opt_idx: OptIndex, arg: &[u8], errmsg: &mut Option<OptError>) -> Option<OptInt> {
    let is_key_option = matches!(opt_idx, kOptWildchar | kOptWildcharm);
    let first = c_int::from(at(arg, 0));
    let looks_like_key = first == '<' as c_int
        || first == '^' as c_int
        || (first != NUL && (at(arg, 1) == 0 || white_at(arg, 1)) && !ascii_isdigit(first));
    if is_key_option && looks_like_key {
        let key = OptInt::from(string_to_key(arg));
        if key == 0 {
            *errmsg = Some(e_invarg.into());
            return None;
        }
        return Some(key);
    }

    if first != '-' as c_int && !ascii_isdigit(first) {
        *errmsg = Some(E_NUMBER_REQUIRED_AFTER_EQUAL.into());
        return None;
    }
    let parsed = str2nr_in(arg, Str2NrBases::ALL, true);
    let len = parsed.len;
    if len == 0 || (at(arg, len) != 0 && !white_at(arg, len)) {
        *errmsg = Some(E_NUMBER_REQUIRED_AFTER_EQUAL.into());
        return None;
    }
    Some(parsed.value)
}

/// Parse and apply one `:set` argument, moving `*p` past what it took.
fn do_one_set_option(
    opt_flags: OptionSetFlags,
    arg: &[u8],
    p: &mut usize,
    did_show: &mut bool,
    errmsg: &mut Option<OptError>,
) {
    let prefix = get_option_prefix(arg, p);
    let start = *p;

    let text = &arg[start..];
    let (opt_idx, name_len) = option_end(text);
    if opt_idx == kOptInvalid {
        // A terminal option is accepted and discarded.
        if !is_tty_option(text) {
            *errmsg = Some(E_UNKNOWN_OPTION.into());
        }
        return;
    }
    // A known option always has a name.
    let option_end = start + name_len.unwrap_or(0);

    // What ends the name decides whether a trailing character is an
    // error; `:set ai  ?` is allowed, `:set ai?x` is not.
    let after_white = white_at(arg, option_end);
    let mut q = option_end;
    while white_at(arg, q) {
        q += 1;
    }
    let op = get_op(arg, q);
    if op != OP_NONE {
        q += 1;
    }
    let nextchar = at(arg, q);

    let flags = get_option(opt_idx).flags;
    let varp = get_varp_scope(opt_idx, opt_flags);

    let win = Win::current();
    if validate_opt_idx(win, opt_idx, opt_flags, flags, prefix, errmsg).is_err() {
        return;
    }

    let one_of = |set: &[u8]| nextchar != 0 && set.contains(&nextchar);
    if one_of(b"?=:!&<") {
        *p = q;
        // `:set opt&vi` and `:set opt&vim` both mean `:set opt&` here;
        // nvim has no separate Vi default.
        if nextchar == b'&' && at(arg, *p + 1) == b'v' && at(arg, *p + 2) == b'i' {
            *p += if at(arg, *p + 3) == b'm' { 3 } else { 2 };
        }
        // Nothing may follow the ones that take no value.
        if one_of(b"?!&<") && at(arg, *p + 1) != 0 && !white_at(arg, *p + 1) {
            *errmsg = Some(e_trailing.into());
            return;
        }
    }

    // `:set opt?`, and a bare `:set opt` for anything but a boolean,
    // shows the value rather than setting it.
    let showing = nextchar == b'?'
        || (prefix == Prefix::None
            && !one_of(b"=:&<")
            && !option_has_type(opt_idx, kOptValTypeBoolean));
    if showing {
        show_one(opt_idx, opt_flags, varp, did_show);
        if nextchar != b'?' && nextchar != 0 && !after_white {
            *errmsg = Some(e_trailing.into());
        }
        return;
    }

    if option_has_type(opt_idx, kOptValTypeBoolean) {
        // A boolean takes no value, and nothing may follow it.
        if one_of(b"=:") {
            *errmsg = Some(e_invarg.into());
            return;
        }
        if !one_of(b"!&<") && nextchar != 0 && !after_white {
            *errmsg = Some(e_trailing.into());
            return;
        }
    } else if !one_of(b"=:&<") {
        *errmsg = Some(e_invarg.into());
        return;
    }

    let newval = get_option_newval(
        opt_idx, opt_flags, prefix, arg, p, nextchar, op, flags, varp, errmsg,
    );
    if newval.is_nil() || errmsg.is_some() {
        return;
    }
    // `+=`/`^=`/`-=` amend the value; only a plain assignment replaces it,
    // which is what clears the insecure mark.
    let replaced = op == OP_NONE;
    if let Err(err) = set_option(opt_idx, newval, opt_flags, 0 as ScriptId, false, replaced) {
        // The report carries what upstream's `errbuf` held: the message cut
        // to the room it had.
        let message = err.as_cstr().to_bytes();
        let kept = &message[..message.len().min(ERRBUF_LEN - 1)];
        *errmsg = Some(OptError::Owned(XString::from_bytes(kept)));
    }
}

/// Show one option's value, on its own line, opening the message area the
/// first time. `varp` is the variable the value is read from, which says
/// which script context 'verbose' reports.
fn show_one(opt_idx: OptIndex, opt_flags: OptionSetFlags, varp: OptSlot, did_show: &mut bool) {
    if *did_show {
        msg_putchar('\n' as c_int);
    } else {
        msg_ext_set_kind(c"list_cmd");
        gotocmdline(true);
        *did_show = true;
    }
    showoneopt(opt_idx, opt_flags);

    // With 'verbose' set, say where the value came from — from the
    // script context of the scope the value is being read from.
    if p_verbose() <= 0 {
        return;
    }
    if varp == option_var(opt_idx) {
        last_set_msg(option_last_set(opt_idx));
    } else if option_has_scope(opt_idx, kOptScopeWin) {
        let at = option_scope_idx(opt_idx, kOptScopeWin) as usize;
        last_set_msg(Win::current().w_onebuf_opt.wo_script_ctx[at]);
    } else if option_has_scope(opt_idx, kOptScopeBuf) {
        let at = option_scope_idx(opt_idx, kOptScopeBuf) as usize;
        last_set_msg(Buf::current().b_p_script_ctx[at]);
    }
}

/// The whole of a `:set` command line: a list of arguments, `all`, `all&`,
/// or nothing at all.
///
/// Answers `Err` on the first argument that is rejected, having reported
/// it; the rest of the line is not looked at.
pub(crate) fn do_set(arg: &[u8], opt_flags: OptionSetFlags) -> Result<(), Failed> {
    let mut did_show = false;
    let mut p = 0;

    if at(arg, 0) == 0 {
        showoptions(false, opt_flags);
        did_show = true;
    }
    while at(arg, p) != 0 {
        // "all" is only the keyword when it is a whole word, and never
        // in a modeline.
        let is_all = arg[p..].starts_with(b"all")
            && !at(arg, p + 3).is_ascii_alphabetic()
            && !opt_flags.has(OptionSetFlags::MODELINE);
        if is_all {
            p += 3;
            if at(arg, p) == b'&' {
                p += 1;
                set_options_default(opt_flags);
                didset_options();
                didset_options2();
                ui_refresh_options();
                redraw_all_later(UPD_CLEAR);
            } else {
                showoptions(true, opt_flags);
                did_show = true;
            }
        } else {
            let startarg = p;
            let mut errmsg = None;
            do_one_set_option(opt_flags, arg, &mut p, &mut did_show, &mut errmsg);
            // Step over the value, and over an `=` that starts another
            // one — twice at most, which is what `:set opt=a=b` needs.
            for _ in 0..2 {
                p = skip_to_white_escaped(arg, p);
                p = skip_white(arg, p);
                if at(arg, p) != b'=' {
                    break;
                }
            }
            if let Some(errmsg) = errmsg {
                report(errmsg.as_cstr(), &arg[startarg..p.min(arg.len())]);
                return Err(Failed);
            }
        }
        p = skip_white(arg, p);
    }

    // `-s` suppresses messages, but a `:set` that was asked to show
    // something still ends its listing with a newline.
    if silent_mode.get() && did_show {
        silent_mode.set(false);
        info_message.set(true);
        msg_putchar('\n' as c_int);
        silent_mode.set(true);
        info_message.set(false);
    }
    Ok(())
}

/// Past the spaces and tabs at `arg[p..]`.
fn skip_white(arg: &[u8], mut p: usize) -> usize {
    while white_at(arg, p) {
        p += 1;
    }
    p
}

/// To the next space or tab at `arg[p..]`, or its end; a backslash or
/// CTRL-V hides the byte after it -- upstream's `skiptowhite_esc`.
fn skip_to_white_escaped(arg: &[u8], mut p: usize) -> usize {
    loop {
        let byte = at(arg, p);
        if byte == 0 || ascii_iswhite(c_int::from(byte)) {
            return p;
        }
        let escapes = (byte == b'\\' || c_int::from(byte) == Ctrl_V) && at(arg, p + 1) != 0;
        p += 1 + usize::from(escapes);
    }
}

/// Report a rejected argument as "message: argument".
///
/// The argument is appended only when both fit in the report buffer; a
/// message that long stands on its own.
fn report(errmsg: &CStr, arg: &[u8]) {
    // Upstream builds the report in `IObuff`, and translates its unprintable
    // characters in place within that room: the buffer is the room.
    let room = IOSIZE as usize;
    let mut report = vec![0u8; room];
    let msg = gettext_owned(errmsg);
    let msg = msg.to_bytes();
    let kept = msg.len().min(room - 1);
    report[..kept].copy_from_slice(&msg[..kept]);
    // Two past the message, leaving room for the ": " written back over
    // its terminator -- measured on the whole message, as the formatter's
    // answer was.
    let at = msg.len() + 2;
    if at + arg.len() < room {
        report[at - 2..at].copy_from_slice(b": ");
        report[at..at + arg.len()].copy_from_slice(arg);
        report[at + arg.len()] = NUL as u8;
    }
    trans_characters_in(&mut report);
    // The message is the whole report; do not make the user acknowledge
    // the half of it that has already scrolled past.
    let _no_prompt = Suppress::wait_return();
    emsg(cstr::in_bytes(&report));
}

/// The key `arg` names, given that it is `len` bytes long and, with
/// `has_lt`, arrived inside `<>` -- in which case `len` counts the `<` too,
/// as the C's `strlen` from it did, and `text` is everything from that `<`
/// to the end of the line, terminator included, which is what the key name
/// is read out of. Zero for anything that is not a key.
fn find_key_len(arg: &[u8], len: usize, text: &[u8], has_lt: bool) -> c_int {
    if len >= 4 && at(arg, 0) == b't' && at(arg, 1) == b'_' {
        // A `t_xx` termcap name, which is two bytes packed into one
        // negative key code.
        if !has_lt || at(arg, 4) == b'>' {
            return -(c_int::from(at(arg, 2)) + (c_int::from(at(arg, 3)) << 8));
        }
        return 0;
    }
    if !has_lt {
        return 0;
    }
    let how = FSK_KEYCODE as c_int | FSK_KEEP_X_KEY as c_int | FSK_SIMPLIFY as c_int;
    match special_key_at(text, how, None) {
        // A key with a modifier left over does not fit in one option value.
        Some((key, modifiers, _)) if modifiers.is_empty() => key,
        _ => 0,
    }
}

/// The key a 'wildchar'-like option's value names: `<xx>`, `^x`, or the
/// first byte taken literally.
pub(crate) fn string_to_key(arg: &[u8]) -> c_int {
    if at(arg, 0) == b'<' && at(arg, 1) != 0 {
        // From the `<` on, terminator and all: the C measured the rest of
        // the line from the `<` and read one byte past it.
        let mut text = arg.to_vec();
        text.push(0);
        return find_key_len(&arg[1..], arg.len(), &text, true);
    }
    if at(arg, 0) == b'^' && at(arg, 1) != 0 {
        // CTRL-x, where NUL would be ambiguous with "no key".
        let key = c_int::from(at(arg, 1).to_ascii_uppercase()) ^ 0x40;
        return if key == 0 { Key::Zero.code() } else { key };
    }
    c_int::from(at(arg, 0))
}
