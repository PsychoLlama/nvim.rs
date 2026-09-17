//! `:syntax` itself: the subcommand table, and the ones that set a mode.
//!
//! [`ex_syntax`] dispatches on the subcommand name; the commands here are the
//! per-block modes (`case`, `conceal`, `foldlevel`, `spell`, `iskeyword`) and
//! the on/off family, which just sources a runtime file. Everything that adds
//! or removes items lives in the sibling modules.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::guard::Suppress;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_char, c_int};

use super::*;
use crate::eval::typval::NumBuf;
use crate::memory::XString;
use crate::optionstr::LocalOptStr;
use crate::types::{CmdLine, NUL};

/// Which of `names` the argument word is, ignoring case.
///
/// A failed test falls through to the next candidate rather than claiming
/// the argument, which is what makes an unknown word E390 rather than a
/// half-applied setting.
fn word_index(word: &[u8], names: &[&CStr]) -> Option<usize> {
    names
        .iter()
        .position(|name| word.eq_ignore_ascii_case(name.to_bytes()))
}

/// Common prologue: record the next command, and answer whether to go on.
fn mode_cmd_start(args: &mut ExArg) -> bool {
    // SAFETY: `arg` is the caller's command line, a NUL-terminated string.
    let arg_start = args.arg_ptr();
    args.set_nextcmd_ptr(unsafe { find_nextcmd(arg_start) });
    !args.skip
}

/// `:syntax conceal [on|off]`.
pub(crate) fn syn_cmd_conceal(args: &mut ExArg, _syncing: c_int) {
    if !mode_cmd_start(args) {
        return;
    }
    let arg = args.arg_ptr();
    // SAFETY: the caller's command line.
    let (word, _) = unsafe { word_at(arg) };
    if word.is_empty() {
        let state = if cur_syn_block().b_syn_conceal != 0 {
            c"syntax conceal on"
        } else {
            c"syntax conceal off"
        };
        msg(state, 0);
    } else if let Some(i) = word_index(word, &[c"on", c"off"]) {
        cur_syn_block().b_syn_conceal = if i == 0 { 1 } else { 0 };
    } else {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str(arg) };
        semsg!("E390: Illegal argument: {arg}");
    }
}

/// `:syntax case [match|ignore]`.
pub(crate) fn syn_cmd_case(args: &mut ExArg, _syncing: c_int) {
    if !mode_cmd_start(args) {
        return;
    }
    let arg = args.arg_ptr();
    // SAFETY: the caller's command line.
    let (word, _) = unsafe { word_at(arg) };
    if word.is_empty() {
        let state = if cur_syn_block().b_syn_ic != 0 {
            c"syntax case ignore"
        } else {
            c"syntax case match"
        };
        msg(state, 0);
    } else if let Some(i) = word_index(word, &[c"match", c"ignore"]) {
        cur_syn_block().b_syn_ic = if i == 0 { 0 } else { 1 };
    } else {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str(arg) };
        semsg!("E390: Illegal argument: {arg}");
    }
}

/// `:syntax foldlevel [start|minimum]`.
pub(crate) fn syn_cmd_foldlevel(args: &mut ExArg, _syncing: c_int) {
    if !mode_cmd_start(args) {
        return;
    }
    let arg = args.arg_ptr();
    // SAFETY: the caller's command line.
    let (word, arg_end) = unsafe { word_at(arg) };
    if word.is_empty() {
        // A block whose foldlevel is neither of the two reports nothing.
        if cur_syn_block().b_syn_foldlevel == SYNFLD_START {
            msg(c"syntax foldlevel start", 0);
        } else if cur_syn_block().b_syn_foldlevel == SYNFLD_MINIMUM {
            msg(c"syntax foldlevel minimum", 0);
        }
        return;
    }

    match word_index(word, &[c"start", c"minimum"]) {
        Some(0) => cur_syn_block().b_syn_foldlevel = SYNFLD_START,
        Some(_) => cur_syn_block().b_syn_foldlevel = SYNFLD_MINIMUM,
        None => {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = unsafe { c_str(arg) };
            semsg!("E390: Illegal argument: {arg}");
            return;
        }
    }

    // Unlike the other mode commands, this one diagnoses trailing text.
    let arg = unsafe { skipwhite(arg_end) };
    if unsafe { *arg } as c_int != NUL {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str(arg) };
        semsg!("E390: Illegal argument: {arg}");
    }
}

/// `:syntax spell [toplevel|notoplevel|default]`.
pub(crate) fn syn_cmd_spell(args: &mut ExArg, _syncing: c_int) {
    if !mode_cmd_start(args) {
        return;
    }
    let arg = args.arg_ptr();
    // SAFETY: the caller's command line.
    let (word, _) = unsafe { word_at(arg) };
    if word.is_empty() {
        let state = match cur_syn_block().b_syn_spell {
            SYNSPL_TOP => c"syntax spell toplevel",
            SYNSPL_NOTOP => c"syntax spell notoplevel",
            _ => c"syntax spell default",
        };
        msg(state, 0);
    } else if let Some(i) = word_index(word, &[c"toplevel", c"notoplevel", c"default"]) {
        cur_syn_block().b_syn_spell = match i {
            0 => SYNSPL_TOP,
            1 => SYNSPL_NOTOP,
            _ => SYNSPL_DEFAULT,
        };
    } else {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str(arg) };
        semsg!("E390: Illegal argument: {arg}");
        return;
    }

    // Assume spell checking changed, force a redraw.
    redraw_later(Win::current(), UPD_NOT_VALID);
}

/// `:syntax iskeyword [clear|{isk-value}]`.
///
/// The value is installed by running it through `'iskeyword'`'s own parser on
/// the current buffer and keeping the character table that produces, so the
/// buffer's own table has to be saved and put back around the call.
pub(crate) fn syn_cmd_iskeyword(args: &mut ExArg, _syncing: c_int) {
    if args.skip {
        return;
    }
    let arg = unsafe { skipwhite(args.arg_ptr()) };
    if unsafe { *arg } as c_int == NUL {
        msg_str(c"\n");
        if !cur_syn_block().b_syn_isk.is_unset() {
            msg_str(c"syntax iskeyword ");
            let value = cur_syn_block().b_syn_isk.clone().unwrap_or_default();
            msg_display(value.as_cstr(), 0, false);
        } else {
            msg_display(gettext(c"syntax iskeyword not set"), 0, false);
        }
    } else if unsafe { strncasecmp(arg, c"clear".as_ptr(), 5) } == 0 {
        cur_syn_block().b_syn_chartab = buf_chartab();
        cur_syn_block().b_syn_isk = None;
    } else {
        // Run the value through `'iskeyword'`'s own parser on the current
        // buffer and keep the table it produces, putting the buffer's own
        // option and table back afterwards. The parsed value then *moves*
        // from the buffer's option into the syntax block's own copy.
        let saved = buf_chartab();
        // SAFETY: the command's argument is NUL-terminated.
        let value = XString::from_cstr(unsafe { cstr::at(arg) });
        let save_isk = Buf::current().b_p_isk.replace(value);

        buf_init_chartab(Buf::current(), false);
        cur_syn_block().b_syn_chartab = buf_chartab();
        set_buf_chartab(saved);
        cur_syn_block().b_syn_isk = Buf::current().b_p_isk.take();
        Buf::current().b_p_isk = save_isk;
    }
    redraw_later(Win::current(), UPD_NOT_VALID);
}

/// The current buffer's character table, as the 32 bytes a syntax block
/// stores it in. The buffer declares it as four `uint64_t`s.
fn buf_chartab() -> [uint8_t; 32] {
    let words = Buf::current().b_chartab;
    ::core::array::from_fn(|i| words[i / 8].to_ne_bytes()[i % 8])
}

/// Put `table` back as the current buffer's character table.
fn set_buf_chartab(table: [uint8_t; 32]) {
    Buf::current().b_chartab = ::core::array::from_fn(|i| {
        uint64_t::from_ne_bytes(table[i * 8..i * 8 + 8].try_into().unwrap())
    });
}

/// `:syntax on` / `:syntax enable`.
pub(crate) fn syn_cmd_on(args: &mut ExArg, _syncing: c_int) {
    syn_cmd_onoff(args, c"syntax")
}

/// `:syntax reset`. It actually resets highlighting, not syntax.
pub(crate) fn syn_cmd_reset(args: &mut ExArg, _syncing: c_int) {
    let arg_start = args.arg_ptr();
    args.set_nextcmd_ptr(unsafe { check_nextcmd(arg_start) });
    if !args.skip {
        init_highlight(true, true);
    }
}

/// `:syntax manual`.
pub(crate) fn syn_cmd_manual(args: &mut ExArg, _syncing: c_int) {
    syn_cmd_onoff(args, c"manual")
}

/// `:syntax off`.
pub(crate) fn syn_cmd_off(args: &mut ExArg, _syncing: c_int) {
    syn_cmd_onoff(args, c"nosyntax")
}

/// Source `$VIMRUNTIME/syntax/{name}.vim`, which is what all four of the
/// on/off commands amount to.
fn syn_cmd_onoff(args: &mut ExArg, name: &CStr) {
    let arg_start = args.arg_ptr();
    args.set_nextcmd_ptr(unsafe { check_nextcmd(arg_start) });
    if args.skip {
        return;
    }
    did_syntax_onoff.set(true);
    let mut buf: [c_char; 100] = [0; 100];
    buf[0] = b's' as c_char;
    buf[1] = b'o' as c_char;
    buf[2] = b' ' as c_char;
    let (at, room) = (unsafe { buf.as_mut_ptr().add(3) }, buf.len() - 3);
    // SAFETY: `at` is three bytes into a buffer with `room` left.
    unsafe { vim_snprintf(at, room, SYNTAX_FNAME.as_ptr(), name.as_ptr()) };
    let _ = unsafe { do_cmdline_cmd(buf.as_ptr()) };
}

/// Turn syntax highlighting on unless `:syntax` has already been used one way
/// or the other.
pub(crate) fn syn_maybe_enable() {
    if !did_syntax_onoff.get() {
        let mut ea = ExArg {
            line: CmdLine::from_bytes(b""),
            skip: false,
            ..Default::default()
        };
        syn_cmd_on(&mut ea, 0);
    }
}

/// A syntax block for `:ownsyntax`, all zero but for the fields a zeroed
/// block is not a valid value for.
///
/// Upstream's `xcalloc(1, sizeof(SynBlock))`; the block is released by
/// `reset_synblock`, which takes the `Box` back.
fn empty_synblock() -> Box<SynBlock> {
    let mut storage = Box::<SynBlock>::new_zeroed();
    // SAFETY: the block was just allocated and nothing has read it.
    unsafe { init_synblock(storage.as_mut_ptr()) };
    // SAFETY: all-zero bytes are otherwise what upstream hands a fresh block.
    unsafe { storage.assume_init() }
}

/// One `:syntax` subcommand.
pub(crate) struct SubCommand {
    pub(crate) name: &'static CStr,
    func: fn(&mut ExArg, c_int),
}

/// A `const fn` constructor keeps each entry on one line under rustfmt.
const fn sub(name: &'static CStr, func: fn(&mut ExArg, c_int)) -> SubCommand {
    SubCommand { name, func }
}

/// Every `:syntax` subcommand. The empty name is the fallthrough — a bare
/// `:syntax` lists the items — and it is also what completion offers last.
pub(crate) static SUBCOMMANDS: [SubCommand; 19] = [
    sub(c"case", syn_cmd_case),
    sub(c"clear", syn_cmd_clear),
    sub(c"cluster", syn_cmd_cluster),
    sub(c"conceal", syn_cmd_conceal),
    sub(c"enable", syn_cmd_on),
    sub(c"foldlevel", syn_cmd_foldlevel),
    sub(c"include", syn_cmd_include),
    sub(c"iskeyword", syn_cmd_iskeyword),
    sub(c"keyword", syn_cmd_keyword),
    sub(c"list", syn_cmd_list),
    sub(c"manual", syn_cmd_manual),
    sub(c"match", syn_cmd_match),
    sub(c"on", syn_cmd_on),
    sub(c"off", syn_cmd_off),
    sub(c"region", syn_cmd_region),
    sub(c"reset", syn_cmd_reset),
    sub(c"spell", syn_cmd_spell),
    sub(c"sync", syn_cmd_sync),
    sub(c"", syn_cmd_list),
];

/// `:syntax`. Finds the subcommand name in [`SUBCOMMANDS`] and calls it.
pub(crate) fn ex_syntax(excmd: &mut ExArg) {
    // SAFETY: the command table's promise -- the argument block of the
    let arg = excmd.arg_ptr();

    // Isolate the subcommand name.
    let mut subcmd_end = arg;
    while (unsafe { *subcmd_end } as u8).is_ascii_alphabetic() {
        subcmd_end = unsafe { subcmd_end.add(1) };
    }
    // SAFETY: both pointers are into the command line, `arg` first.
    let subcmd_name = unsafe { name_at(arg, subcmd_end.offset_from(arg) as usize) };

    // Skip the error messages of every subcommand too.
    let _skipping = (excmd.skip).then(Suppress::emsg_skip);
    match SUBCOMMANDS.iter().find(|sub| *sub.name == *subcmd_name) {
        Some(sub) => {
            excmd.set_arg_ptr(unsafe { skipwhite(subcmd_end) });
            (sub.func)(excmd, 0);
        }
        None => {
            // SAFETY: `subcmd_name` is live for the whole message.
            let subcmd_name = unsafe { c_str(subcmd_name.as_ptr()) };
            semsg!("E410: Invalid :syntax subcommand: {subcmd_name}");
        }
    }
}

/// `:ownsyntax {name}` — give this window its own syntax block.
///
/// Upstream marks this `@deprecated`.
pub(crate) fn ex_ownsyntax(excmd: &mut ExArg) {
    let mut numbuf = NumBuf::new();
    if Win::current().w_s == unsafe { &raw mut (*Win::current().w_buffer).b_s } {
        Win::current().w_s = Box::into_raw(empty_synblock());
        unsafe { hash_init::<*mut c_char>(syn_field!(cur_syn_block(), b_keywtab)) };
        unsafe { hash_init::<*mut c_char>(syn_field!(cur_syn_block(), b_keywtab_ic)) };
        // TODO(vim): Keep the spell checking as it was.
        Win::current().w_onebuf_opt.wo_spell = 0; // No spell checking
        // Upstream replaces the block's five NULL option values with the
        // shared empty string here; a fresh block's are already `None`,
        // which *is* that value.
    }

    // Save the value of b:current_syntax; the autocommand below can change
    // it, so the bytes have to be copied out rather than borrowed.
    let old_value = unsafe { get_var_value(c"b:current_syntax".as_ptr(), &mut numbuf) };
    // SAFETY: a variable's value, a NUL-terminated string, live until the
    // autocommand runs.
    let old_value = unsafe { cstr::at_opt(old_value) }.map(CStr::to_owned);

    // Apply the Syntax autocommand, which finds and loads the syntax file.
    let buffer = Buf::current();
    let (fname, arg) = (buffer.name.shown_ptr(), excmd.arg_ptr());
    // SAFETY: a live buffer, and the command's own NUL-terminated argument.
    unsafe { apply_autocmds(AutoEvent::Syntax, arg, fname, true, Some(buffer)) };

    // Move the value of b:current_syntax to w:current_syntax.
    let new_value = unsafe { get_var_value(c"b:current_syntax".as_ptr(), &mut numbuf) };
    if !new_value.is_null() {
        unsafe { set_internal_string_var(c"w:current_syntax".as_ptr(), new_value) };
    }

    // Restore the value of b:current_syntax.
    match &old_value {
        None => {
            let _ = unsafe { do_unlet(c"b:current_syntax".as_ptr(), 16, true) };
        }
        Some(value) => unsafe {
            set_internal_string_var(c"b:current_syntax".as_ptr(), value.as_ptr().cast_mut());
        },
    }
}
