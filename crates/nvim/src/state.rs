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
    check_end_reg_executing, may_sync_undo, safe_vgetc, stuff_empty, typeahead, using_script,
    vpeekc,
};
use crate::global_cell::GlobalCell;
use crate::insexpand::{ctrl_x_mode_not_defined_yet, ins_compl_active};
use crate::keycodes::{Ctrl_V, K_EVENT, get_special_key_name};
use crate::log::{LOGLVL_DBG, logmsg};
use crate::main::{
    State, curbuf, debug_mode, exmode_active, finish_op, global_busy, got_int, last_mode, mod_mask,
    motion_force, must_redraw, need_wait_return, restart_VIsual_select, restart_edit, virtual_op,
};
use crate::normal::{visual_active, visual_mode, visual_select};
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
            let mut keyname_buf;
            let keyname = if key == K_EVENT {
                c"K_EVENT".as_ptr()
            } else {
                keyname_buf = get_special_key_name(key, mod_mask.get());
                keyname_buf.as_ptr()
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
            if vpeekc() != NUL || !typeahead().is_empty() {
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
                typeahead().change_cnt(),
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
    ve_flags_allow(flags, State.get(), visual_active(), visual_mode().raw())
}

/// The `'virtualedit'` half of [`virtual_active`], as a function of the
/// window's flags alone.
///
/// `ve=all` is the *whole* value, not one bit of it, exactly as the C spells
/// it -- which matters because upstream's `Block` (5) and `Insert` (6) both
/// *contain* `All` (4), so a masked test would answer differently for
/// `ve=all,onemore`.
fn ve_flags_allow(
    flags: OptVeFlags,
    state: ModeFlags,
    visual_active: bool,
    visual_mode: c_int,
) -> bool {
    let has = |flag: OptVeFlags| flags & flag != 0;
    flags == kOptVeFlagAll
        || has(kOptVeFlagBlock) && visual_active && visual_mode == Ctrl_V
        || has(kOptVeFlagInsert) && state & MODE_INSERT != 0
}

/// `State`, with the visual and operator-pending distinctions `State` alone
/// does not carry.
pub fn get_real_state() -> c_int {
    real_state(
        State.get(),
        visual_active(),
        visual_select(),
        finish_op.get(),
    )
}

/// [`get_real_state`] over a snapshot. Only normal mode is refined: the
/// distinctions live in globals `State` has no bit for.
fn real_state(
    state: ModeFlags,
    visual_active: bool,
    visual_select: bool,
    op_pending: bool,
) -> c_int {
    if state & MODE_NORMAL != 0 {
        if visual_active {
            if visual_select {
                return MODE_SELECT;
            }
            return MODE_VISUAL;
        } else if op_pending {
            return MODE_OP_PENDING;
        }
    }
    state
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

/// Everything [`mode_name`]'s decision tree reads, snapshotted from the
/// editor's globals in one place so the tree itself touches none of them.
#[derive(Clone, Copy)]
struct ModeInputs {
    /// The `State` bitmask.
    state: ModeFlags,
    /// The command line is prompting for a single keypress.
    cmdline_one_key: bool,
    /// The command line is overstriking rather than inserting.
    cmdline_overstrike: bool,
    /// Ex mode, i.e. `Q`.
    exmode_active: bool,
    /// A completion menu is up.
    ins_compl_active: bool,
    /// `CTRL-X` was typed and the sub-mode key has not been.
    ctrl_x_pending: bool,
    visual_active: bool,
    visual_select: bool,
    /// `v`, `V` or `CTRL-V`.
    visual_mode: c_int,
    /// Select mode resumes once the current operator finishes.
    restart_visual_select: bool,
    /// The current buffer is a terminal buffer.
    terminal_buffer: bool,
    /// An operator is waiting for its motion.
    finish_op: bool,
    /// The `v`/`V`/`CTRL-V` that forced the pending operator's motion kind.
    motion_force: c_int,
    /// The `:startinsert`-family letter insert mode will resume with, or 0.
    restart_edit: c_int,
}

/// The mode `m` describes, as `mode(1)` spells it.
///
/// Upstream fills a caller-provided `char[MODE_MAX_LENGTH]`; answering the
/// array is the same thing without the raw pointer, and the NUL padding it
/// carries makes it a C string wherever one is still wanted.
fn mode_name(m: &ModeInputs) -> ModeName {
    let state = m.state;
    let mut out = ModeWriter {
        name: [0; 4],
        len: 0,
    };

    if state == MODE_HITRETURN
        || state == MODE_ASKMORE
        || state == MODE_SETWSIZE
        || m.cmdline_one_key
    {
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
        if m.ins_compl_active {
            out.push(b'c');
        } else if m.ctrl_x_pending {
            out.push(b'x');
        }
    } else if state & MODE_CMDLINE != 0 || m.exmode_active {
        out.push(b'c');
        if m.exmode_active {
            out.push(b'v');
        }
        if state & MODE_CMDLINE != 0 && m.cmdline_overstrike {
            out.push(b'r');
        }
    } else if state & MODE_TERMINAL != 0 {
        out.push(b't');
    } else if m.visual_active {
        if m.visual_select {
            // `v`/`V`/`CTRL-V` shifted into the select-mode letters.
            out.push_key(m.visual_mode + c_int::from(b's') - c_int::from(b'v'));
        } else {
            out.push_key(m.visual_mode);
            if m.restart_visual_select {
                out.push(b's');
            }
        }
    } else {
        out.push(b'n');
        if m.finish_op {
            out.push(b'o');
            out.push_key(m.motion_force);
        } else if m.terminal_buffer {
            out.push(b't');
            if m.restart_edit == c_int::from(b'I') {
                out.push(b'T');
            }
        } else if is_restart_key(m.restart_edit) {
            out.push(b'i');
            out.push_key(m.restart_edit);
        }
    }
    out.name
}

/// The current mode as `mode(1)` spells it.
///
/// # Safety
/// The editor must be initialized: this reads the command line's state and
/// the current buffer.
pub unsafe fn get_mode() -> ModeName {
    let state = State.get();
    let in_cmdline = state & MODE_CMDLINE != 0;
    mode_name(&ModeInputs {
        state,
        // SAFETY: the editor is initialized, so the command-line state is live.
        cmdline_one_key: in_cmdline && unsafe { (*get_cmdline_info()).one_key },
        cmdline_overstrike: in_cmdline && cmdline_overstrike(),
        exmode_active: exmode_active.get(),
        ins_compl_active: ins_compl_active(),
        ctrl_x_pending: ctrl_x_mode_not_defined_yet(),
        visual_active: visual_active(),
        visual_select: visual_select(),
        visual_mode: visual_mode().raw(),
        restart_visual_select: restart_VIsual_select.get() != 0,
        // SAFETY: the editor is initialized, so `curbuf` is live.
        terminal_buffer: !unsafe { (*curbuf.get()).terminal }.is_null(),
        finish_op: finish_op.get(),
        motion_force: motion_force.get(),
        restart_edit: restart_edit.get(),
    })
}

/// The `ModeChanged` autocommand pattern: `old:new`, NUL-padded.
///
/// Both halves are at most three letters, so the eight bytes upstream sizes
/// this at are exactly enough and it never truncates.
fn modechanged_pattern(old: &ModeName, new: &ModeName) -> [c_char; 2 * size_of::<ModeName>()] {
    let mut pattern = [0 as c_char; 2 * size_of::<ModeName>()];
    let colon = [b':'.cast_signed()];
    let joined = letters(old).iter().chain(&colon).chain(letters(new));
    for (slot, &letter) in pattern.iter_mut().zip(joined) {
        *slot = letter;
    }
    pattern
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

    let mut pattern = modechanged_pattern(&old_mode, &curr_mode);

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
    stuff_empty()
        && typeahead().is_empty()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::kOptVeFlagOnemore;

    /// A [`ModeName`] from its letters, NUL-padded as the writer leaves it.
    fn name(letters: &str) -> ModeName {
        let mut out = [0 as c_char; 4];
        for (slot, byte) in out.iter_mut().zip(letters.bytes()) {
            *slot = byte.cast_signed();
        }
        out
    }

    /// The letters of a mode name, as text.
    fn text(mode: &ModeName) -> String {
        letters(mode)
            .iter()
            .map(|&c| char::from(c.cast_unsigned()))
            .collect()
    }

    /// A quiescent editor in `state`: nothing pending anywhere.
    fn quiet(state: ModeFlags) -> ModeInputs {
        ModeInputs {
            state,
            cmdline_one_key: false,
            cmdline_overstrike: false,
            exmode_active: false,
            ins_compl_active: false,
            ctrl_x_pending: false,
            visual_active: false,
            visual_select: false,
            visual_mode: 0,
            restart_visual_select: false,
            terminal_buffer: false,
            finish_op: false,
            motion_force: 0,
            restart_edit: 0,
        }
    }

    /// The name stops at the first NUL, and a name that fills the array has
    /// no NUL to stop at.
    #[test]
    fn letters_stop_at_the_padding() {
        assert_eq!(letters(&name("")), &[] as &[c_char]);
        assert_eq!(text(&name("no")), "no");
        assert_eq!(text(&[b'n'.cast_signed(); 4]), "nnnn");
    }

    /// `push_key` takes the low byte of a key, which is all a mode letter
    /// ever is.
    #[test]
    fn a_pushed_key_is_its_low_byte() {
        let mut out = ModeWriter {
            name: [0; 4],
            len: 0,
        };
        out.push(b'n');
        out.push_key(c_int::from(b'o'));
        // `CTRL-V` is 22, and `motion_force` may hold it.
        out.push_key(Ctrl_V);
        assert_eq!(out.name, [110, 111, 22, 0]);
    }

    /// Only the three `:startinsert`-family letters resume insert mode; the
    /// idle value 0 must not.
    #[test]
    fn only_irv_restart_insert_mode() {
        for key in *b"IRV" {
            assert!(is_restart_key(c_int::from(key)));
        }
        for key in [0, c_int::from(b'i'), c_int::from(b'n'), c_int::from(b'T')] {
            assert!(!is_restart_key(key));
        }
    }

    /// Normal mode is the only one the visual/operator distinctions refine;
    /// every other `State` is answered unchanged.
    #[test]
    fn real_state_refines_normal_mode_only() {
        assert_eq!(real_state(MODE_NORMAL, false, false, false), MODE_NORMAL);
        assert_eq!(real_state(MODE_NORMAL, true, false, false), MODE_VISUAL);
        assert_eq!(real_state(MODE_NORMAL, true, true, false), MODE_SELECT);
        assert_eq!(real_state(MODE_NORMAL, false, false, true), MODE_OP_PENDING);
        // Visual wins over a pending operator, as the nesting says.
        assert_eq!(real_state(MODE_NORMAL, true, false, true), MODE_VISUAL);
        for state in [MODE_INSERT, MODE_CMDLINE, MODE_TERMINAL, MODE_REPLACE] {
            assert_eq!(real_state(state, true, true, true), state);
        }
        // `MODE_NORMAL_BUSY` carries `MODE_NORMAL`'s bit, so it refines too.
        assert_eq!(
            real_state(MODE_NORMAL_BUSY, true, false, false),
            MODE_VISUAL
        );
    }

    /// The first arm is `==`, not a mask test, because upstream's flag
    /// values are not disjoint bits: `Block` (5) and `Insert` (6) both
    /// *contain* `All` (4). `ve=all,onemore` is the value that tells the two
    /// spellings apart -- it is not `All`, so nothing but block-visual or
    /// insert mode may make it true.
    #[test]
    fn ve_all_is_the_whole_value() {
        let all_onemore = kOptVeFlagAll | kOptVeFlagOnemore;
        assert!(ve_flags_allow(kOptVeFlagAll, MODE_NORMAL, false, 0));
        assert!(!ve_flags_allow(all_onemore, MODE_NORMAL, false, 0));
        assert!(!ve_flags_allow(all_onemore, MODE_NORMAL, true, Ctrl_V - 1));
        assert!(ve_flags_allow(all_onemore, MODE_NORMAL, true, Ctrl_V));
        assert!(ve_flags_allow(all_onemore, MODE_INSERT, false, 0));
        // `onemore` alone shares no bit with any of the three.
        assert!(!ve_flags_allow(
            kOptVeFlagOnemore,
            MODE_INSERT,
            true,
            Ctrl_V
        ));
    }

    /// `ve=block` only counts in block-visual, `ve=insert` only in insert.
    #[test]
    fn ve_block_and_insert_need_their_mode() {
        assert!(!ve_flags_allow(kOptVeFlagBlock, MODE_NORMAL, false, 0));
        assert!(!ve_flags_allow(kOptVeFlagBlock, MODE_NORMAL, true, 0));
        assert!(ve_flags_allow(kOptVeFlagBlock, MODE_NORMAL, true, Ctrl_V));
        assert!(!ve_flags_allow(kOptVeFlagInsert, MODE_NORMAL, false, 0));
        assert!(ve_flags_allow(kOptVeFlagInsert, MODE_INSERT, false, 0));
    }

    /// The prompting modes, which are whole `State` values rather than
    /// bits -- and `r?` is the command line asking for one key.
    #[test]
    fn the_prompt_modes_answer_r() {
        assert_eq!(text(&mode_name(&quiet(MODE_HITRETURN))), "r");
        assert_eq!(text(&mode_name(&quiet(MODE_ASKMORE))), "rm");
        assert_eq!(text(&mode_name(&quiet(MODE_SETWSIZE))), "r");
        assert_eq!(text(&mode_name(&quiet(MODE_EXTERNCMD))), "!");
        let one_key = ModeInputs {
            cmdline_one_key: true,
            ..quiet(MODE_CMDLINE)
        };
        assert_eq!(text(&mode_name(&one_key)), "r?");
    }

    /// Insert, replace and virtual replace, each with the completion
    /// suffixes that can follow.
    #[test]
    fn the_insert_family_and_its_suffixes() {
        assert_eq!(text(&mode_name(&quiet(MODE_INSERT))), "i");
        assert_eq!(text(&mode_name(&quiet(MODE_REPLACE))), "R");
        assert_eq!(text(&mode_name(&quiet(MODE_VREPLACE))), "Rv");
        for (state, expected) in [
            (MODE_INSERT, ("ic", "ix")),
            (MODE_REPLACE, ("Rc", "Rx")),
            (MODE_VREPLACE, ("Rvc", "Rvx")),
        ] {
            let completing = ModeInputs {
                ins_compl_active: true,
                ctrl_x_pending: true,
                ..quiet(state)
            };
            assert_eq!(text(&mode_name(&completing)), expected.0);
            let ctrl_x = ModeInputs {
                ctrl_x_pending: true,
                ..quiet(state)
            };
            assert_eq!(text(&mode_name(&ctrl_x)), expected.1);
        }
    }

    /// The command line, Ex mode, and the two together.
    #[test]
    fn the_cmdline_family() {
        assert_eq!(text(&mode_name(&quiet(MODE_CMDLINE))), "c");
        let overstrike = ModeInputs {
            cmdline_overstrike: true,
            ..quiet(MODE_CMDLINE)
        };
        assert_eq!(text(&mode_name(&overstrike)), "cr");
        let ex = ModeInputs {
            exmode_active: true,
            ..quiet(MODE_NORMAL)
        };
        assert_eq!(text(&mode_name(&ex)), "cv");
        // Overstrike is a command-line property: Ex mode alone cannot show
        // it, because there is no command line to overstrike.
        let ex_overstrike = ModeInputs {
            exmode_active: true,
            cmdline_overstrike: true,
            ..quiet(MODE_NORMAL)
        };
        assert_eq!(text(&mode_name(&ex_overstrike)), "cv");
        let both = ModeInputs {
            exmode_active: true,
            cmdline_overstrike: true,
            ..quiet(MODE_CMDLINE)
        };
        assert_eq!(text(&mode_name(&both)), "cvr");
    }

    /// Visual mode is named by its own key; select mode shifts that key by
    /// `s - v`, which is what turns `v`/`V`/`CTRL-V` into `s`/`S`/`CTRL-S`.
    #[test]
    fn visual_and_select_are_named_by_their_key() {
        for key in *b"vV" {
            let visual = ModeInputs {
                visual_active: true,
                visual_mode: c_int::from(key),
                ..quiet(MODE_NORMAL)
            };
            assert_eq!(mode_name(&visual), name(&String::from(char::from(key))));
            let select = ModeInputs {
                visual_select: true,
                ..visual
            };
            let shifted = c_int::from(key) - c_int::from(b'v') + c_int::from(b's');
            assert_eq!(c_int::from(select_letter(&select)), shifted);
        }
        // `CTRL-V` (22) becomes `CTRL-S` (19), neither of them printable.
        let blockwise = ModeInputs {
            visual_active: true,
            visual_select: true,
            visual_mode: Ctrl_V,
            ..quiet(MODE_NORMAL)
        };
        assert_eq!(mode_name(&blockwise), [19, 0, 0, 0]);
    }

    /// The single letter a select-mode name is.
    fn select_letter(inputs: &ModeInputs) -> u8 {
        let mode = mode_name(inputs);
        assert_eq!(letters(&mode).len(), 1);
        mode[0].cast_unsigned()
    }

    /// A visual mode the operator will return to appends `s`; select mode
    /// never does, because it is already there.
    #[test]
    fn a_resuming_select_mode_appends_s() {
        let resuming = ModeInputs {
            visual_active: true,
            visual_mode: c_int::from(b'V'),
            restart_visual_select: true,
            ..quiet(MODE_NORMAL)
        };
        assert_eq!(text(&mode_name(&resuming)), "Vs");
        let already = ModeInputs {
            visual_select: true,
            ..resuming
        };
        assert_eq!(text(&mode_name(&already)), "S");
    }

    /// Normal mode's four shapes: plain, operator-pending with its forced
    /// motion, a terminal buffer, and insert resuming after a `CTRL-O`.
    #[test]
    fn normal_mode_and_what_hangs_off_it() {
        assert_eq!(text(&mode_name(&quiet(MODE_NORMAL))), "n");
        let pending = ModeInputs {
            finish_op: true,
            motion_force: c_int::from(b'v'),
            ..quiet(MODE_NORMAL)
        };
        assert_eq!(text(&mode_name(&pending)), "nov");
        // An unforced motion pads with the NUL `motion_force` holds, so the
        // name is the two letters `no`.
        let unforced = ModeInputs {
            motion_force: 0,
            ..pending
        };
        assert_eq!(text(&mode_name(&unforced)), "no");
        assert_eq!(mode_name(&unforced), name("no"));

        let terminal = ModeInputs {
            terminal_buffer: true,
            ..quiet(MODE_NORMAL)
        };
        assert_eq!(text(&mode_name(&terminal)), "nt");
        let terminal_insert = ModeInputs {
            restart_edit: c_int::from(b'I'),
            ..terminal
        };
        assert_eq!(text(&mode_name(&terminal_insert)), "ntT");

        for key in *b"IRV" {
            let restarting = ModeInputs {
                restart_edit: c_int::from(key),
                ..quiet(MODE_NORMAL)
            };
            // `CTRL-O`'s mode: normal mode, inside the insert it resumes.
            let expected = format!("ni{}", char::from(key));
            assert_eq!(text(&mode_name(&restarting)), expected);
        }
        // Anything else in `restart_edit` is not a restart at all.
        let idle = ModeInputs {
            restart_edit: c_int::from(b'i'),
            ..quiet(MODE_NORMAL)
        };
        assert_eq!(text(&mode_name(&idle)), "n");
    }

    /// A terminal buffer in normal mode is `nt`; `MODE_TERMINAL` -- the
    /// terminal's *own* mode -- is `t`, and it is checked first.
    #[test]
    fn terminal_mode_outranks_a_terminal_buffer() {
        let inputs = ModeInputs {
            terminal_buffer: true,
            visual_active: true,
            ..quiet(MODE_TERMINAL)
        };
        assert_eq!(text(&mode_name(&inputs)), "t");
    }

    /// No mode name is longer than three letters, so none of them overruns
    /// the array `get_mode` answers -- upstream's `MODE_MAX_LENGTH`.
    #[test]
    fn no_name_fills_the_array() {
        let states = [
            MODE_NORMAL,
            MODE_VISUAL,
            MODE_INSERT,
            MODE_REPLACE,
            MODE_VREPLACE,
            MODE_CMDLINE,
            MODE_TERMINAL,
            MODE_HITRETURN,
            MODE_ASKMORE,
            MODE_SETWSIZE,
            MODE_EXTERNCMD,
            MODE_NORMAL_BUSY,
            MODE_SHOWMATCH,
            MODE_OP_PENDING,
            MODE_SELECT,
            MODE_LREPLACE,
        ];
        let flags = [false, true];
        let mut longest = 0;
        for state in states {
            for one_key in flags {
                for compl in flags {
                    for visual in flags {
                        for finish in flags {
                            let inputs = ModeInputs {
                                cmdline_one_key: one_key,
                                cmdline_overstrike: true,
                                exmode_active: one_key,
                                ins_compl_active: compl,
                                ctrl_x_pending: true,
                                visual_active: visual,
                                visual_select: compl,
                                visual_mode: c_int::from(b'V'),
                                restart_visual_select: true,
                                terminal_buffer: finish,
                                finish_op: finish,
                                motion_force: c_int::from(b'V'),
                                restart_edit: c_int::from(b'I'),
                                ..quiet(state)
                            };
                            let mode = mode_name(&inputs);
                            longest = longest.max(letters(&mode).len());
                            assert_eq!(mode[3], 0, "{mode:?} left no room for the NUL");
                        }
                    }
                }
            }
        }
        assert_eq!(longest, 3);
    }

    /// `old:new`, and the longest pair still fits the eight bytes upstream
    /// sizes the pattern at.
    #[test]
    fn the_modechanged_pattern_joins_with_a_colon() {
        assert_eq!(modechanged_pattern(&name("n"), &name("i")), name2("n:i"));
        assert_eq!(
            modechanged_pattern(&name("no"), &name("Rvc")),
            name2("no:Rvc")
        );
        // Three letters each side is 3 + 1 + 3 = 7 of the 8 bytes.
        let full = modechanged_pattern(&name("Rvc"), &name("ntT"));
        assert_eq!(full, name2("Rvc:ntT"));
        assert_eq!(full[7], 0);
        // An empty old mode is what the very first `ModeChanged` sees.
        assert_eq!(modechanged_pattern(&name(""), &name("n")), name2(":n"));
    }

    /// A pattern from its letters, NUL-padded.
    fn name2(letters: &str) -> [c_char; 2 * size_of::<ModeName>()] {
        let mut out = [0 as c_char; 2 * size_of::<ModeName>()];
        for (slot, byte) in out.iter_mut().zip(letters.bytes()) {
            *slot = byte.cast_signed();
        }
        out
    }
}
