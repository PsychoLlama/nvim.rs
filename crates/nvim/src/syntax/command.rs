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
use core::ffi::{CStr, c_char, c_int, c_void};

use super::*;
use crate::eval::typval::NumBuf;
use crate::types::NUL;

/// Is the word between `arg` and `next` exactly `name`, ignoring case?
///
/// Every mode command tests its arguments this way, and a failed test falls
/// through to the next candidate rather than claiming the argument.
unsafe fn word_is(arg: *const c_char, next: *const c_char, name: &CStr) -> bool {
    let len = name.count_bytes();
    unsafe { next.offset_from(arg) as usize == len && strncasecmp(arg, name.as_ptr(), len) == 0 }
}

/// Which of `names` the word between `arg` and `next` is.
unsafe fn word_index(arg: *const c_char, next: *const c_char, names: &[&CStr]) -> Option<usize> {
    unsafe { names.iter().position(|name| word_is(arg, next, name)) }
}

/// Common prologue: record the next command, and answer whether to go on.
unsafe fn mode_cmd_start(eap: *mut exarg_T) -> bool {
    unsafe { (*eap).nextcmd = find_nextcmd((*eap).arg) };
    unsafe { (*eap).skip == 0 }
}

/// `:syntax conceal [on|off]`.
pub(crate) unsafe fn syn_cmd_conceal(eap: *mut exarg_T, _syncing: c_int) {
    if !unsafe { mode_cmd_start(eap) } {
        return;
    }
    let arg = unsafe { (*eap).arg };
    let next = unsafe { skiptowhite(arg) };
    if unsafe { *arg } as c_int == NUL {
        let state = if cur_syn_block().b_syn_conceal != 0 {
            c"syntax conceal on"
        } else {
            c"syntax conceal off"
        };
        msg(state, 0);
    } else if let Some(i) = unsafe { word_index(arg, next, &[c"on", c"off"]) } {
        cur_syn_block().b_syn_conceal = if i == 0 { 1 } else { 0 };
    } else {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str(arg) };
        semsg!("E390: Illegal argument: {arg}");
    }
}

/// `:syntax case [match|ignore]`.
pub(crate) unsafe fn syn_cmd_case(eap: *mut exarg_T, _syncing: c_int) {
    if !unsafe { mode_cmd_start(eap) } {
        return;
    }
    let arg = unsafe { (*eap).arg };
    let next = unsafe { skiptowhite(arg) };
    if unsafe { *arg } as c_int == NUL {
        let state = if cur_syn_block().b_syn_ic != 0 {
            c"syntax case ignore"
        } else {
            c"syntax case match"
        };
        msg(state, 0);
    } else if let Some(i) = unsafe { word_index(arg, next, &[c"match", c"ignore"]) } {
        cur_syn_block().b_syn_ic = if i == 0 { 0 } else { 1 };
    } else {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str(arg) };
        semsg!("E390: Illegal argument: {arg}");
    }
}

/// `:syntax foldlevel [start|minimum]`.
pub(crate) unsafe fn syn_cmd_foldlevel(eap: *mut exarg_T, _syncing: c_int) {
    if !unsafe { mode_cmd_start(eap) } {
        return;
    }
    let arg = unsafe { (*eap).arg };
    if unsafe { *arg } as c_int == NUL {
        // A block whose foldlevel is neither of the two reports nothing.
        if cur_syn_block().b_syn_foldlevel == SYNFLD_START {
            msg(c"syntax foldlevel start", 0);
        } else if cur_syn_block().b_syn_foldlevel == SYNFLD_MINIMUM {
            msg(c"syntax foldlevel minimum", 0);
        }
        return;
    }

    let arg_end = unsafe { skiptowhite(arg) };
    match unsafe { word_index(arg, arg_end, &[c"start", c"minimum"]) } {
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
pub(crate) unsafe fn syn_cmd_spell(eap: *mut exarg_T, _syncing: c_int) {
    if !unsafe { mode_cmd_start(eap) } {
        return;
    }
    let arg = unsafe { (*eap).arg };
    let next = unsafe { skiptowhite(arg) };
    if unsafe { *arg } as c_int == NUL {
        let state = match cur_syn_block().b_syn_spell {
            SYNSPL_TOP => c"syntax spell toplevel",
            SYNSPL_NOTOP => c"syntax spell notoplevel",
            _ => c"syntax spell default",
        };
        msg(state, 0);
    } else if let Some(i) =
        unsafe { word_index(arg, next, &[c"toplevel", c"notoplevel", c"default"]) }
    {
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
pub(crate) unsafe fn syn_cmd_iskeyword(eap: *mut exarg_T, _syncing: c_int) {
    if unsafe { (*eap).skip } != 0 {
        return;
    }
    let arg = unsafe { skipwhite((*eap).arg) };
    if unsafe { *arg } as c_int == NUL {
        unsafe { msg_puts(c"\n".as_ptr()) };
        if !is_empty_option(cur_syn_block().b_syn_isk) {
            unsafe { msg_puts(c"syntax iskeyword ".as_ptr()) };
            unsafe { msg_outtrans(cur_syn_block().b_syn_isk, 0, false) };
        } else {
            unsafe { msg_outtrans(gettext(c"syntax iskeyword not set").as_ptr(), 0, false) };
        }
    } else if unsafe { strncasecmp(arg, c"clear".as_ptr(), 5) } == 0 {
        let chartab: *mut [uint8_t; 32] = syn_field!(cur_syn_block(), b_syn_chartab);
        // SAFETY: the editor's current buffer.
        let src = unsafe { &raw const (*curbuf.get()).b_chartab }.cast::<c_void>();
        // SAFETY: both tables are 32 bytes wide.
        unsafe { chartab.cast::<u8>().copy_from(src.cast(), 32) };
        unsafe { clear_string_option(syn_field!(cur_syn_block(), b_syn_isk)) };
    } else {
        let mut save_chartab: [c_char; 32] = [0; 32];
        let dst = (&raw mut save_chartab).cast::<c_void>();
        // SAFETY: the editor's current buffer.
        let src = unsafe { &raw const (*curbuf.get()).b_chartab }.cast::<c_void>();
        // SAFETY: both tables are 32 bytes wide.
        unsafe { dst.cast::<u8>().copy_from(src.cast(), 32) };
        let save_isk = unsafe { (*curbuf.get()).b_p_isk };
        unsafe { (*curbuf.get()).b_p_isk = xstrdup(arg) };

        unsafe { buf_init_chartab(curbuf.get(), false) };
        let dst: *mut [uint8_t; 32] = syn_field!(cur_syn_block(), b_syn_chartab);
        let dst = dst.cast::<c_void>();
        // SAFETY: the editor's current buffer.
        let src = unsafe { &raw const (*curbuf.get()).b_chartab }.cast::<c_void>();
        // SAFETY: both tables are 32 bytes wide.
        unsafe { dst.cast::<u8>().copy_from(src.cast(), 32) };
        // SAFETY: the editor's current buffer.
        let dst = unsafe { &raw mut (*curbuf.get()).b_chartab }.cast::<c_void>();
        let src = (&raw const save_chartab).cast::<c_void>();
        // SAFETY: both tables are 32 bytes wide.
        unsafe { dst.cast::<u8>().copy_from(src.cast(), 32) };
        unsafe { clear_string_option(syn_field!(cur_syn_block(), b_syn_isk)) };
        unsafe { cur_syn_block().b_syn_isk = (*curbuf.get()).b_p_isk };
        unsafe { (*curbuf.get()).b_p_isk = save_isk };
    }
    unsafe { redraw_later(curwin.get(), UPD_NOT_VALID) };
}

/// `:syntax on` / `:syntax enable`.
pub(crate) unsafe fn syn_cmd_on(eap: *mut exarg_T, _syncing: c_int) {
    unsafe { syn_cmd_onoff(eap, c"syntax") }
}

/// `:syntax reset`. It actually resets highlighting, not syntax.
pub(crate) unsafe fn syn_cmd_reset(eap: *mut exarg_T, _syncing: c_int) {
    unsafe { (*eap).nextcmd = check_nextcmd((*eap).arg) };
    if unsafe { (*eap).skip } == 0 {
        unsafe { init_highlight(true, true) };
    }
}

/// `:syntax manual`.
pub(crate) unsafe fn syn_cmd_manual(eap: *mut exarg_T, _syncing: c_int) {
    unsafe { syn_cmd_onoff(eap, c"manual") }
}

/// `:syntax off`.
pub(crate) unsafe fn syn_cmd_off(eap: *mut exarg_T, _syncing: c_int) {
    unsafe { syn_cmd_onoff(eap, c"nosyntax") }
}

/// Source `$VIMRUNTIME/syntax/{name}.vim`, which is what all four of the
/// on/off commands amount to.
unsafe fn syn_cmd_onoff(eap: *mut exarg_T, name: &CStr) {
    unsafe { (*eap).nextcmd = check_nextcmd((*eap).arg) };
    if unsafe { (*eap).skip } != 0 {
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
        unsafe { syn_cmd_on(&raw mut ea, 0) };
    }
}

/// One `:syntax` subcommand.
pub(crate) struct SubCommand {
    pub(crate) name: &'static CStr,
    func: unsafe fn(*mut exarg_T, c_int),
}

/// A `const fn` constructor keeps each entry on one line under rustfmt.
const fn sub(name: &'static CStr, func: unsafe fn(*mut exarg_T, c_int)) -> SubCommand {
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
    let arg = unsafe { (*eap).arg };
    syn_cmdlinep.set(unsafe { (*eap).cmdlinep });

    // Isolate the subcommand name.
    let mut subcmd_end = arg;
    while (unsafe { *subcmd_end } as u8).is_ascii_alphabetic() {
        subcmd_end = unsafe { subcmd_end.add(1) };
    }
    let subcmd_name = unsafe { xstrnsave(arg, subcmd_end.offset_from(arg) as size_t) };

    // Skip the error messages of every subcommand too.
    let _skipping = (unsafe { (*eap).skip } != 0).then(Suppress::emsg_skip);
    match SUBCOMMANDS
        .iter()
        .find(|sub| unsafe { cstr::eq(subcmd_name, sub.name.as_ptr()) })
    {
        Some(sub) => {
            unsafe { (*eap).arg = skipwhite(subcmd_end) };
            unsafe { (sub.func)(eap, 0) };
        }
        None => {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let subcmd_name = unsafe { c_str(subcmd_name) };
            semsg!("E410: Invalid :syntax subcommand: {subcmd_name}");
        }
    }
    unsafe { xfree(subcmd_name as *mut c_void) };
}

/// `:ownsyntax {name}` — give this window its own syntax block.
///
/// Upstream marks this `@deprecated`.
pub(crate) unsafe fn ex_ownsyntax(eap: *mut exarg_T) {
    let mut numbuf = NumBuf::new();
    if unsafe { (*curwin.get()).w_s } == unsafe { &raw mut (*(*curwin.get()).w_buffer).b_s } {
        unsafe {
            (*curwin.get()).w_s =
                xcalloc(1, ::core::mem::size_of::<synblock_T>()) as *mut synblock_T
        };
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

    // Save the value of b:current_syntax.
    let mut old_value = unsafe { get_var_value(c"b:current_syntax".as_ptr(), &mut numbuf) };
    if !old_value.is_null() {
        old_value = unsafe { xstrdup(old_value) };
    }

    // Apply the Syntax autocommand, which finds and loads the syntax file.
    let buf = curbuf.get();
    // SAFETY: the caller's command and the editor's current buffer.
    let (arg, fname) = unsafe { ((*eap).arg, (*buf).b_fname) };
    unsafe { apply_autocmds(AutoEvent::Syntax, arg, fname, true, buf) };

    // Move the value of b:current_syntax to w:current_syntax.
    let new_value = unsafe { get_var_value(c"b:current_syntax".as_ptr(), &mut numbuf) };
    if !new_value.is_null() {
        unsafe { set_internal_string_var(c"w:current_syntax".as_ptr(), new_value) };
    }

    // Restore the value of b:current_syntax.
    if old_value.is_null() {
        let _ = unsafe { do_unlet(c"b:current_syntax".as_ptr(), 16, true) };
    } else {
        unsafe { set_internal_string_var(c"b:current_syntax".as_ptr(), old_value) };
        unsafe { xfree(old_value as *mut c_void) };
    }
}
