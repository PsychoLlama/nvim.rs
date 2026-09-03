//! `:syntax` itself: the subcommand table, and the ones that set a mode.
//!
//! [`ex_syntax`] dispatches on the subcommand name; the commands here are the
//! per-block modes (`case`, `conceal`, `foldlevel`, `spell`, `iskeyword`) and
//! the on/off family, which just sources a runtime file. Everything that adds
//! or removes items lives in the sibling modules.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::guard::Suppress;
use crate::message_fmt::c_str;
use crate::optionstr::is_empty_option;
use crate::semsg;
use core::ffi::{CStr, c_char, c_int};

use super::*;
use crate::eval::typval::NumBuf;
use crate::types::NUL;

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
fn mode_cmd_start(eap: &mut exarg_T) -> bool {
    // SAFETY: `arg` is the caller's command line, a NUL-terminated string.
    eap.nextcmd = unsafe { find_nextcmd(eap.arg) };
    eap.skip == 0
}

/// `:syntax conceal [on|off]`.
pub(crate) fn syn_cmd_conceal(eap: &mut exarg_T, _syncing: c_int) {
    if !mode_cmd_start(eap) {
        return;
    }
    let arg = eap.arg;
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
pub(crate) fn syn_cmd_case(eap: &mut exarg_T, _syncing: c_int) {
    if !mode_cmd_start(eap) {
        return;
    }
    let arg = eap.arg;
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
pub(crate) fn syn_cmd_foldlevel(eap: &mut exarg_T, _syncing: c_int) {
    if !mode_cmd_start(eap) {
        return;
    }
    let arg = eap.arg;
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
pub(crate) fn syn_cmd_spell(eap: &mut exarg_T, _syncing: c_int) {
    if !mode_cmd_start(eap) {
        return;
    }
    let arg = eap.arg;
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
    unsafe { redraw_later(curwin.get(), UPD_NOT_VALID) };
}

/// `:syntax iskeyword [clear|{isk-value}]`.
///
/// The value is installed by running it through `'iskeyword'`'s own parser on
/// the current buffer and keeping the character table that produces, so the
/// buffer's own table has to be saved and put back around the call.
pub(crate) fn syn_cmd_iskeyword(eap: &mut exarg_T, _syncing: c_int) {
    if eap.skip != 0 {
        return;
    }
    let arg = unsafe { skipwhite(eap.arg) };
    if unsafe { *arg } as c_int == NUL {
        unsafe { msg_puts(c"\n".as_ptr()) };
        if !is_empty_option(cur_syn_block().b_syn_isk) {
            unsafe { msg_puts(c"syntax iskeyword ".as_ptr()) };
            unsafe { msg_outtrans(cur_syn_block().b_syn_isk, 0, false) };
        } else {
            unsafe { msg_outtrans(gettext(c"syntax iskeyword not set").as_ptr(), 0, false) };
        }
    } else if unsafe { strncasecmp(arg, c"clear".as_ptr(), 5) } == 0 {
        cur_syn_block().b_syn_chartab = buf_chartab();
        unsafe { clear_string_option(syn_field!(cur_syn_block(), b_syn_isk)) };
    } else {
        // Run the value through `'iskeyword'`'s own parser on the current
        // buffer and keep the table it produces, putting the buffer's own
        // option and table back afterwards.
        let saved = buf_chartab();
        let save_isk = unsafe { (*curbuf.get()).b_p_isk };
        unsafe { (*curbuf.get()).b_p_isk = xstrdup(arg) };

        unsafe { buf_init_chartab(curbuf.get(), false) };
        cur_syn_block().b_syn_chartab = buf_chartab();
        set_buf_chartab(saved);
        unsafe { clear_string_option(syn_field!(cur_syn_block(), b_syn_isk)) };
        unsafe { cur_syn_block().b_syn_isk = (*curbuf.get()).b_p_isk };
        unsafe { (*curbuf.get()).b_p_isk = save_isk };
    }
    unsafe { redraw_later(curwin.get(), UPD_NOT_VALID) };
}

/// The current buffer's character table, as the 32 bytes a syntax block
/// stores it in. The buffer declares it as four `uint64_t`s.
fn buf_chartab() -> [uint8_t; 32] {
    // SAFETY: the editor's current buffer.
    let words = unsafe { (*curbuf.get()).b_chartab };
    ::core::array::from_fn(|i| words[i / 8].to_ne_bytes()[i % 8])
}

/// Put `table` back as the current buffer's character table.
fn set_buf_chartab(table: [uint8_t; 32]) {
    // SAFETY: the editor's current buffer.
    unsafe {
        (*curbuf.get()).b_chartab = ::core::array::from_fn(|i| {
            uint64_t::from_ne_bytes(table[i * 8..i * 8 + 8].try_into().unwrap())
        });
    };
}

/// `:syntax on` / `:syntax enable`.
pub(crate) fn syn_cmd_on(eap: &mut exarg_T, _syncing: c_int) {
    syn_cmd_onoff(eap, c"syntax")
}

/// `:syntax reset`. It actually resets highlighting, not syntax.
pub(crate) fn syn_cmd_reset(eap: &mut exarg_T, _syncing: c_int) {
    eap.nextcmd = unsafe { check_nextcmd(eap.arg) };
    if eap.skip == 0 {
        unsafe { init_highlight(true, true) };
    }
}

/// `:syntax manual`.
pub(crate) fn syn_cmd_manual(eap: &mut exarg_T, _syncing: c_int) {
    syn_cmd_onoff(eap, c"manual")
}

/// `:syntax off`.
pub(crate) fn syn_cmd_off(eap: &mut exarg_T, _syncing: c_int) {
    syn_cmd_onoff(eap, c"nosyntax")
}

/// Source `$VIMRUNTIME/syntax/{name}.vim`, which is what all four of the
/// on/off commands amount to.
fn syn_cmd_onoff(eap: &mut exarg_T, name: &CStr) {
    eap.nextcmd = unsafe { check_nextcmd(eap.arg) };
    if eap.skip != 0 {
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
pub(crate) unsafe fn syn_maybe_enable() {
    if !did_syntax_onoff.get() {
        let mut ea = exarg_T {
            arg: c"".as_ptr().cast_mut(),
            skip: 0,
            ..Default::default()
        };
        syn_cmd_on(&mut ea, 0);
    }
}

/// A syntax block for `:ownsyntax`, all zero but for the fields a zeroed
/// block is not a valid value for.
///
/// Upstream's `xcalloc(1, sizeof(synblock_T))`; the block is released by
/// `reset_synblock`, which takes the `Box` back.
fn empty_synblock() -> Box<synblock_T> {
    let mut storage = Box::<synblock_T>::new_zeroed();
    // SAFETY: the block was just allocated and nothing has read it.
    unsafe { init_synblock(storage.as_mut_ptr()) };
    // SAFETY: all-zero bytes are otherwise what upstream hands a fresh block.
    unsafe { storage.assume_init() }
}

/// One `:syntax` subcommand.
pub(crate) struct SubCommand {
    pub(crate) name: &'static CStr,
    func: fn(&mut exarg_T, c_int),
}

/// A `const fn` constructor keeps each entry on one line under rustfmt.
const fn sub(name: &'static CStr, func: fn(&mut exarg_T, c_int)) -> SubCommand {
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
pub(crate) unsafe fn ex_syntax(eap: *mut exarg_T) {
    // SAFETY: the command table's promise -- the argument block of the
    // `:` command being run, which nothing else holds while it runs.
    let eap = unsafe { &mut *eap };
    let arg = eap.arg;
    syn_cmdlinep.set(eap.cmdlinep);

    // Isolate the subcommand name.
    let mut subcmd_end = arg;
    while (unsafe { *subcmd_end } as u8).is_ascii_alphabetic() {
        subcmd_end = unsafe { subcmd_end.add(1) };
    }
    // SAFETY: both pointers are into the command line, `arg` first.
    let subcmd_name = unsafe { name_at(arg, subcmd_end.offset_from(arg) as usize) };

    // Skip the error messages of every subcommand too.
    let _skipping = (eap.skip != 0).then(Suppress::emsg_skip);
    match SUBCOMMANDS.iter().find(|sub| *sub.name == *subcmd_name) {
        Some(sub) => {
            eap.arg = unsafe { skipwhite(subcmd_end) };
            (sub.func)(eap, 0);
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
pub(crate) unsafe fn ex_ownsyntax(eap: *mut exarg_T) {
    // SAFETY: the command table's promise, as `ex_syntax`'s.
    let eap = unsafe { &mut *eap };
    let mut numbuf = NumBuf::new();
    if unsafe { (*curwin.get()).w_s } == unsafe { &raw mut (*(*curwin.get()).w_buffer).b_s } {
        unsafe { (*curwin.get()).w_s = Box::into_raw(empty_synblock()) };
        unsafe { hash_init(syn_field!(cur_syn_block(), b_keywtab)) };
        unsafe { hash_init(syn_field!(cur_syn_block(), b_keywtab_ic)) };
        // TODO(vim): Keep the spell checking as it was.
        unsafe { (*curwin.get()).w_onebuf_opt.wo_spell = 0 }; // No spell checking
        // Make sure option values are "empty_string_option" instead of NULL.
        unsafe { clear_string_option(syn_field!(cur_syn_block(), b_p_spc)) };
        unsafe { clear_string_option(syn_field!(cur_syn_block(), b_p_spf)) };
        unsafe { clear_string_option(syn_field!(cur_syn_block(), b_p_spl)) };
        unsafe { clear_string_option(syn_field!(cur_syn_block(), b_p_spo)) };
        unsafe { clear_string_option(syn_field!(cur_syn_block(), b_syn_isk)) };
    }

    // Save the value of b:current_syntax; the autocommand below can change
    // it, so the bytes have to be copied out rather than borrowed.
    let old_value = unsafe { get_var_value(c"b:current_syntax".as_ptr(), &mut numbuf) };
    // SAFETY: a variable's value, a NUL-terminated string, live until the
    // autocommand runs.
    let old_value = unsafe { cstr::at_opt(old_value) }.map(CStr::to_owned);

    // Apply the Syntax autocommand, which finds and loads the syntax file.
    let buf = curbuf.get();
    // SAFETY: the editor's current buffer.
    let fname = unsafe { (*buf).b_fname };
    let arg = eap.arg;
    unsafe { apply_autocmds(AutoEvent::Syntax, arg, fname, true, buf) };

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
