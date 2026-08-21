#![deny(unsafe_op_in_unsafe_fn)]

//! The editor's mode: the loop every modal state machine runs on, and the
//! `mode()` string that names where it currently is.
//!
//! A `VimState` is a pair of function pointers — `check` before each key,
//! `execute` for the key — and [`state_enter`] is the loop that drives them.
//! Normal, insert, terminal and command-line mode are all one of these; the
//! loop is what makes `K_EVENT` (an event-queue wakeup rather than a key)
//! look like input to all of them.

use core::ffi::{c_char, c_int};
use core::ptr;

use crate::autocmd::{EVENT_MODECHANGED, EVENT_SAFESTATE, apply_autocmds, has_event};
use crate::channel::main_loop_events;
use crate::drawscreen::{setcursor, update_screen};
use crate::eval::typval::{tv_dict_add_str, tv_dict_set_keys_readonly};
use crate::eval::{get_v_event, restore_v_event};
use crate::event::multiqueue::{multiqueue_empty, multiqueue_get};
use crate::ex_getln::{cmdline_overstrike, get_cmdline_info};
use crate::getchar::{
    check_end_reg_executing, may_sync_undo, safe_vgetc, stuff_empty, using_script, vpeekc,
};
use crate::global_cell::GlobalCell;
use crate::insexpand::{ctrl_x_mode_not_defined_yet, ins_compl_active};
use crate::keycodes::{Ctrl_V, K_EVENT, get_special_key_name};
use crate::log::{LOGLVL_DBG, logmsg};
use crate::main::{
    State, VIsual_active, VIsual_mode, VIsual_select, curbuf, debug_mode, exmode_active, finish_op,
    global_busy, got_int, last_mode, mod_mask, motion_force, must_redraw, need_wait_return,
    restart_VIsual_select, restart_edit, typebuf, virtual_op,
};
use crate::option::get_ve_flags;
use crate::options::{OptVeFlags, kOptVeFlagAll, kOptVeFlagBlock, kOptVeFlagInsert};
use crate::os::input::{input_available, input_get, os_breakcheck};
use crate::types::{
    Direction, NUL, ProcType, VimState, hashitem_T, hashtab_T, save_v_event_T, uint8_t, win_T,
};
use crate::ui::ui_flush;

pub const kProcTypePty: ProcType = 1;
pub const kDirectionNotSet: Direction = 0;

/// The editor-mode bitmask `State` carries, and the masks that read it.
pub type ModeFlags = c_int;
pub const MODE_SHOWMATCH: ModeFlags = 24592;
pub const MODE_EXTERNCMD: ModeFlags = 20480;
pub const MODE_SETWSIZE: ModeFlags = 16384;
pub const MODE_ASKMORE: ModeFlags = 12288;
pub const MODE_HITRETURN: ModeFlags = 8193;
pub const MODE_NORMAL_BUSY: ModeFlags = 4097;
pub const MODE_LREPLACE: ModeFlags = 288;
pub const MODE_VREPLACE: ModeFlags = 784;
pub const VREPLACE_FLAG: ModeFlags = 512;
pub const MODE_REPLACE: ModeFlags = 272;
pub const REPLACE_FLAG: ModeFlags = 256;
pub const MAP_ALL_MODES: ModeFlags = 255;
pub const MODE_TERMINAL: ModeFlags = 128;
pub const MODE_SELECT: ModeFlags = 64;
pub const MODE_LANGMAP: ModeFlags = 32;
pub const MODE_INSERT: ModeFlags = 16;
pub const MODE_CMDLINE: ModeFlags = 8;
pub const MODE_OP_PENDING: ModeFlags = 4;
pub const MODE_VISUAL: ModeFlags = 2;
pub const MODE_NORMAL: ModeFlags = 1;

/// Run `s` until its `execute` says to stop.
///
/// `check` runs before every key: 0 ends the state, -1 asks for it to be run
/// again (something it did invalidated what came after), anything else goes
/// on to read a key. `execute` answers the same three ways for the key.
///
/// # Safety
/// `s` must point at a live `VimState` whose two callbacks are safe to run,
/// and the editor must be initialized.
pub unsafe fn state_enter(s: *mut VimState) {
    'state: loop {
        // SAFETY: the caller's `VimState`, live for the whole call.
        let check_result = match unsafe { (*s).check } {
            // SAFETY: as above — the callback is the state's own.
            Some(check) => unsafe { check(s) },
            None => 1,
        };
        if check_result == 0 {
            break;
        }
        if check_result == -1 {
            continue;
        }
        loop {
            let key = unsafe { next_key() };
            if key == K_EVENT {
                // The queue is about to run arbitrary code, so anything the
                // key-reading side was in the middle of has to be settled.
                // SAFETY: the editor is initialized.
                unsafe {
                    check_end_reg_executing(true);
                    may_sync_undo();
                }
            }
            let keyname = if key == K_EVENT {
                c"K_EVENT".as_ptr()
            } else {
                get_special_key_name(key, mod_mask.get()).cast_const()
            };
            // SAFETY: `keyname` is NUL-terminated and outlives the call.
            unsafe { logmsg!(LOGLVL_DBG, c"state_enter", 97, c"input: %s", keyname) };
            // SAFETY: the caller's `VimState`; `execute` is the state's own.
            let execute_result =
                unsafe { (*s).execute.expect("non-null function pointer")(s, key) };
            if execute_result == 0 {
                break 'state;
            }
            if execute_result != -1 {
                break;
            }
        }
    }
}

/// The next key for [`state_enter`], which may be the pseudo-key `K_EVENT`:
/// either the queue already has work, or waiting for input woke us with
/// work and no key.
///
/// # Safety
/// The editor must be initialized.
unsafe fn next_key() -> c_int {
    loop {
        // SAFETY: the editor is initialized, so the typeahead buffer and the
        // main loop's queue are live.
        unsafe {
            if vpeekc() != NUL || typebuf.with(|t| t.tb_len) > 0 {
                return safe_vgetc();
            }
            if !multiqueue_empty(main_loop_events()) {
                ui_flush();
                return K_EVENT;
            }
            // Nothing to do but wait, so show what has been decided first.
            if must_redraw.get() != 0 && !need_wait_return.get() && State.get() & MODE_CMDLINE == 0
            {
                update_screen();
                setcursor();
            }
            ui_flush();
            input_get(
                ptr::null_mut::<uint8_t>(),
                0,
                -1,
                typebuf.with(|t| t.tb_change_cnt),
                main_loop_events(),
            );
            // A wakeup with neither input nor queued work is spurious.
            if input_available() != 0 || multiqueue_empty(main_loop_events()) {
                continue;
            }
            return K_EVENT;
        }
    }
}

/// Run everything the event queue is holding, unless input or an interrupt
/// arrives first. What [`state_enter`]'s `K_EVENT` dispatches to.
///
/// # Safety
/// The editor must be initialized.
pub unsafe fn state_handle_k_event() {
    loop {
        // SAFETY: the main loop's queue is live, and an `Event` it answers
        // owns its own `argv`.
        unsafe {
            let mut event = multiqueue_get(main_loop_events());
            if let Some(handler) = event.handler {
                handler(&raw mut event.argv as *mut *mut core::ffi::c_void);
            }
            if multiqueue_empty(main_loop_events()) {
                return;
            }
            os_breakcheck();
        }
        if input_available() != 0 || got_int.get() {
            return;
        }
    }
}

/// Whether the cursor may sit where there is no character, in `wp`.
///
/// # Safety
/// `wp` must point at a live window.
pub unsafe fn virtual_active(wp: *mut win_T) -> bool {
    // Inside an operator, the operator's own answer stands.
    if let Some(active) = virtual_op.get() {
        return active;
    }
    if State.get() & MODE_TERMINAL != 0 {
        return true;
    }
    // SAFETY: the caller's window.
    let flags = unsafe { get_ve_flags(wp) };
    // `ve=all` is the *whole* value, not one bit of it, exactly as the C
    // spells it -- which matters because upstream's `Block` (5) and `Insert`
    // (6) both *contain* `All` (4), so a masked test would answer differently
    // for `ve=all,onemore`.
    let has = |flag: OptVeFlags| flags & flag != 0;
    flags == kOptVeFlagAll
        || has(kOptVeFlagBlock) && VIsual_active.get() && VIsual_mode.get() == Ctrl_V
        || has(kOptVeFlagInsert) && State.get() & MODE_INSERT != 0
}

/// `State`, with the visual and operator-pending distinctions `State` alone
/// does not carry.
pub fn get_real_state() -> c_int {
    if State.get() & MODE_NORMAL != 0 {
        if VIsual_active.get() {
            if VIsual_select.get() {
                return MODE_SELECT;
            }
            return MODE_VISUAL;
        } else if finish_op.get() {
            return MODE_OP_PENDING;
        }
    }
    State.get()
}

/// One to three mode letters, NUL-padded — upstream's `char[MODE_MAX_LENGTH]`.
type ModeName = [c_char; 4];

/// The letters of a mode name, without the NUL padding.
fn letters(mode: &ModeName) -> &[c_char] {
    let len = mode.iter().position(|&c| c == 0).unwrap_or(mode.len());
    &mode[..len]
}

/// Appends to a [`ModeName`] as upstream's `buf[i++] = ...` did.
struct ModeWriter {
    name: ModeName,
    len: usize,
}

impl ModeWriter {
    fn push(&mut self, letter: u8) {
        self.name[self.len] = letter.cast_signed();
        self.len += 1;
    }

    /// A letter upstream computes rather than spells: the visual mode's own
    /// key, or `motion_force`. Both are single bytes by construction.
    fn push_key(&mut self, key: c_int) {
        let [byte, ..] = key.to_le_bytes();
        self.push(byte);
    }
}

/// The `restart_edit` values that report as insert mode: the three
/// `:startinsert`-family letters.
fn is_restart_key(key: c_int) -> bool {
    b"IRV".map(c_int::from).contains(&key)
}

/// The current mode as `mode(1)` spells it.
///
/// Upstream fills a caller-provided `char[MODE_MAX_LENGTH]`; answering the
/// array is the same thing without the raw pointer, and the NUL padding it
/// carries makes it a C string wherever one is still wanted.
///
/// # Safety
/// The editor must be initialized: this reads the command line's state and
/// the current buffer.
pub unsafe fn get_mode() -> ModeName {
    let state = State.get();
    let mut out = ModeWriter {
        name: [0; 4],
        len: 0,
    };
    // SAFETY: the editor is initialized, so the command-line state is live.
    let one_key = state & MODE_CMDLINE != 0 && unsafe { (*get_cmdline_info()).one_key };

    if state == MODE_HITRETURN || state == MODE_ASKMORE || state == MODE_SETWSIZE || one_key {
        out.push(b'r');
        if state == MODE_ASKMORE {
            out.push(b'm');
        } else if state & MODE_CMDLINE != 0 {
            out.push(b'?');
        }
    } else if state == MODE_EXTERNCMD {
        out.push(b'!');
    } else if state & MODE_INSERT != 0 {
        if state & VREPLACE_FLAG != 0 {
            out.push(b'R');
            out.push(b'v');
        } else if state & REPLACE_FLAG != 0 {
            out.push(b'R');
        } else {
            out.push(b'i');
        }
        if ins_compl_active() {
            out.push(b'c');
        } else if ctrl_x_mode_not_defined_yet() {
            out.push(b'x');
        }
    } else if state & MODE_CMDLINE != 0 || exmode_active.get() {
        out.push(b'c');
        if exmode_active.get() {
            out.push(b'v');
        }
        if state & MODE_CMDLINE != 0 && cmdline_overstrike() {
            out.push(b'r');
        }
    } else if state & MODE_TERMINAL != 0 {
        out.push(b't');
    } else if VIsual_active.get() {
        if VIsual_select.get() {
            // `v`/`V`/`CTRL-V` shifted into the select-mode letters.
            out.push_key(VIsual_mode.get() + c_int::from(b's') - c_int::from(b'v'));
        } else {
            out.push_key(VIsual_mode.get());
            if restart_VIsual_select.get() != 0 {
                out.push(b's');
            }
        }
    } else {
        out.push(b'n');
        if finish_op.get() {
            out.push(b'o');
            out.push_key(motion_force.get());
        // SAFETY: the editor is initialized, so `curbuf` is live.
        } else if !unsafe { (*curbuf.get()).terminal }.is_null() {
            out.push(b't');
            if restart_edit.get() == c_int::from(b'I') {
                out.push(b'T');
            }
        } else if is_restart_key(restart_edit.get()) {
            out.push(b'i');
            out.push_key(restart_edit.get());
        }
    }
    out.name
}

/// Fire `ModeChanged` if the mode string has changed since the last time
/// this ran. Called from wherever a mode transition finishes.
///
/// # Safety
/// The editor must be initialized.
pub unsafe fn may_trigger_modechanged() {
    // SAFETY: the editor is initialized.
    if !unsafe { has_event(EVENT_MODECHANGED) } || got_int.get() {
        return;
    }
    let mut old_mode = last_mode.get();
    // SAFETY: as above.
    let mut curr_mode = unsafe { get_mode() };
    if letters(&old_mode) == letters(&curr_mode) {
        return;
    }

    // `old:new`. Both halves are at most three letters, so the eight bytes
    // upstream sizes this at are exactly enough and it never truncates.
    let mut pattern = [0 as c_char; 2 * size_of::<ModeName>()];
    let colon = [b':'.cast_signed()];
    let joined = letters(&old_mode)
        .iter()
        .chain(&colon)
        .chain(letters(&curr_mode));
    for (slot, &letter) in pattern.iter_mut().zip(joined) {
        *slot = letter;
    }

    let mut save_v_event = save_v_event_T {
        sve_did_save: false,
        sve_hashtab: hashtab_T {
            ht_mask: 0,
            ht_used: 0,
            ht_filled: 0,
            ht_changed: 0,
            ht_locked: 0,
            ht_array: ptr::null_mut::<hashitem_T>(),
            ht_smallarray: [hashitem_T {
                hi_hash: 0,
                hi_key: ptr::null_mut::<c_char>(),
            }; 16],
        },
    };
    // SAFETY: the editor is initialized; `v_event` is borrowed from
    // `save_v_event`, which outlives the `restore_v_event` that ends it, and
    // both mode names outlive the autocommand that reads them.
    unsafe {
        let v_event = get_v_event(&raw mut save_v_event);
        tv_dict_add_str(
            v_event,
            c"new_mode".as_ptr(),
            c"new_mode".count_bytes(),
            curr_mode.as_mut_ptr(),
        );
        tv_dict_add_str(
            v_event,
            c"old_mode".as_ptr(),
            c"old_mode".count_bytes(),
            old_mode.as_mut_ptr(),
        );
        tv_dict_set_keys_readonly(v_event);
        apply_autocmds(
            EVENT_MODECHANGED,
            pattern.as_mut_ptr(),
            ptr::null_mut::<c_char>(),
            false,
            curbuf.get(),
        );
        last_mode.set(curr_mode);
        restore_v_event(v_event, &raw mut save_v_event);
    }
}

/// Whether `SafeState` has been announced and not yet withdrawn.
static was_safe: GlobalCell<bool> = GlobalCell::new(false);

/// Nothing is pending that would make running arbitrary code surprising.
///
/// # Safety
/// The editor must be initialized.
unsafe fn is_safe_now() -> bool {
    // SAFETY: the editor is initialized.
    let stuff_empty = unsafe { stuff_empty() };
    stuff_empty
        && typebuf.with(|t| t.tb_len) == 0
        && using_script() == 0
        && global_busy.get() == 0
        && !debug_mode.get()
}

/// Fire `SafeState` when the editor has come to rest. `safe` is the caller's
/// half of the answer: it knows whether *it* is in the middle of something.
///
/// # Safety
/// The editor must be initialized.
pub unsafe fn may_trigger_safestate(safe: bool) {
    // SAFETY: the editor is initialized.
    let is_safe = safe && unsafe { is_safe_now() };
    if was_safe.get() != is_safe {
        let what = if is_safe {
            c"SafeState: Start triggering"
        } else {
            c"SafeState: Stop triggering"
        };
        // SAFETY: `what` is a literal, and carries no format directives.
        unsafe { logmsg!(LOGLVL_DBG, c"may_trigger_safestate", 305, what) };
    }
    if is_safe {
        // SAFETY: the editor is initialized, so `curbuf` is live.
        unsafe {
            apply_autocmds(
                EVENT_SAFESTATE,
                ptr::null_mut::<c_char>(),
                ptr::null_mut::<c_char>(),
                false,
                curbuf.get(),
            );
        }
    }
    was_safe.set(is_safe);
}

/// Withdraw `SafeState`: something is pending again. `reason` is logged.
///
/// # Safety
/// `reason` must be null or NUL-terminated.
pub unsafe fn state_no_longer_safe(reason: *const c_char) {
    if was_safe.get() && !reason.is_null() {
        // SAFETY: the caller's NUL-terminated string.
        unsafe {
            logmsg!(
                LOGLVL_DBG,
                c"state_no_longer_safe",
                319,
                c"SafeState reset: %s",
                reason
            );
        }
    }
    was_safe.set(false);
}

/// Whether `SafeState` currently stands. Read by `state()`.
pub fn get_was_safe_state() -> bool {
    was_safe.get()
}
