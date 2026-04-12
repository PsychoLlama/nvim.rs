//! rs_command_line_enter: Rust implementation of command_line_enter and
//! rs_getcmdline_prompt: Rust implementation of getcmdline_prompt
//!
//! Migrated from ex_getln.c (Phase 1 + Phase 2 + Phase 3).

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::{c_char, c_int, c_void};

use crate::command_line_state::{CommandLineState, VimState};
use crate::entry::{rs_entry_should_add_to_history, rs_entry_should_save_last_cmdline};
use crate::history::hist_type;

// =============================================================================
// FFI declarations
// =============================================================================

unsafe extern "C" {
    // Level / cmdpreview
    fn nvim_cmdline_enter_level_inc() -> c_int;
    fn nvim_cmdline_enter_level_dec();
    fn nvim_cmdpreview_save_and_clear() -> bool;
    fn nvim_get_cmdpreview() -> bool;
    fn nvim_cmdpreview_restore(saved: bool, current: bool);

    // ccline save/restore
    fn nvim_ccline_save_and_clear(save_out: *mut c_void, clear_ccline_flag: bool) -> bool;
    fn nvim_ccline_restore(save_out: *const c_void);

    // ccline init
    fn nvim_ccline_enter_init(firstc: c_int, indent: c_int);
    fn nvim_ccline_set_level(level: c_int);
    fn nvim_ccline_apply_indent(indent: c_int);
    fn nvim_ccline_init_xpc(s: *mut c_void);

    // alloc_cmdbuff (already in Rust, exported as extern C)
    fn alloc_cmdbuff(len: c_int);
    fn dealloc_cmdbuff();

    // sb_text (direct)
    fn sb_text_start_cmdline();
    fn sb_text_end_cmdline();

    // cmdmsg_rl / msg_grid_validate / redir_off
    fn nvim_set_cmdmsg_rl(val: c_int);
    fn msg_grid_validate();
    fn nvim_set_redir_off(val: c_int);

    // gotocmdline / setmouse / may_trigger_modechanged
    fn nvim_gotocmdline();
    fn setmouse();
    fn may_trigger_modechanged();

    // State / msg_scroll / clear flags
    fn nvim_set_State(val: c_int);
    fn nvim_get_msg_scroll() -> c_int;
    fn nvim_set_msg_scroll(val: c_int);
    fn nvim_clear_did_emsg();
    fn nvim_clear_got_int();
    fn nvim_get_did_emsg_for_redraw() -> c_int;
    fn nvim_clear_need_wait_return_wrap();

    // p_icm
    fn nvim_get_p_icm_dup() -> *mut c_char;
    fn nvim_set_p_icm_option(val: *const c_char);

    // Langmap / RTL
    fn nvim_cmdline_setup_langmap(s: *mut c_void, firstc: c_int) -> c_int;
    fn rs_entry_should_use_cmdmsg_rl(
        firstc: c_int,
        win_p_rl: c_int,
        win_p_rlc_has_s: c_int,
    ) -> c_int;

    // Statuslines / redraw
    fn nvim_cmdline_redraw_statuslines();
    fn nvim_get_exmode_active() -> c_int;

    // rs_redrawcmdprompt, rs_cmd_startcol
    fn rs_redrawcmdprompt();
    fn rs_cmd_startcol() -> c_int;
    fn nvim_set_ccline_cmdspos(spos: c_int);

    // Autocmds
    fn nvim_cmdline_fire_enter_full(firstcbuf: *const c_char, level: c_int) -> c_int;
    fn nvim_cmdline_fire_leave_full(s: *mut c_void, c_val: c_int) -> c_int;
    fn set_vim_var_char(c: c_int);

    // History
    fn nvim_init_history_and_get_hislen() -> c_int;
    fn rs_entry_hist_char2type(firstc: c_int) -> c_int;

    // Digraph init (direct — do_digraph(-1))
    fn do_digraph(c: c_int) -> c_int;

    // state_enter (C function that runs the state machine)
    fn nvim_state_enter(state: *mut c_void);

    // command_line_check / command_line_execute are Rust functions already exported

    // emsg for recursion error
    fn nvim_emsg_command_too_recursive();

    // curwin accessors for RTL
    fn nvim_get_curwin_p_rl() -> c_int;
    fn nvim_get_curwin_p_rlc_has_s() -> c_int;

    // State accessor
    fn nvim_get_State() -> c_int;

    // Phase 3 cleanup — direct C functions
    fn cmdline_pum_active() -> c_int;
    fn cmdline_pum_remove(defer_redraw: bool);
    fn pum_check_clear();
    fn nvim_wildmenu_cleanup_ccline();
    fn ExpandCleanup(xpc: *mut c_void);
    fn nvim_ccline_clear_xpc_and_orig();
    fn nvim_add_to_history_ccline(histype: c_int, sep_char: c_int);
    fn nvim_save_last_cmdline();
    fn nvim_compute_cmdrow_if_not_scrolled();
    fn nvim_cmdline_gotesc_msg();
    fn nvim_cmdline_check_must_redraw();
    fn nvim_ccline_has_cmdbuff() -> c_int;
    fn nvim_ccline_get_one_key() -> c_int;
    fn nvim_ccline_free_last_colors();
    fn nvim_cmdline_ui_hide(gotesc: c_int);
    fn nvim_cmdline_status_redraw();
    fn nvim_ccline_restore_or_clear(did_save: bool, save_ccline_in: *const c_void);
    fn nvim_get_ccline_cmdlen() -> c_int;
    fn msg_check();
    fn xfree(ptr: *mut c_char);

    // msg_col / msg_silent — direct static access (Phase 2)
    static mut msg_col: c_int;
    static mut msg_silent: c_int;
    static mut cmd_silent: bool;

    // ccline field setters for prompt (Phase 2)
    fn nvim_set_ccline_cmdprompt(prompt: *mut c_char);
    fn nvim_set_ccline_hl_id(hl_id: c_int);
    fn nvim_set_ccline_xp_context(context: c_int);
    fn nvim_set_ccline_xp_arg(arg: *const c_char);
    fn nvim_set_ccline_input_fn(val: c_int);
    fn nvim_set_ccline_one_key(val: c_int);
    fn nvim_set_ccline_mouse_used_ptr(ptr: *mut bool);
    fn nvim_set_ccline_redraw_state(state: c_int);
    fn nvim_get_ccline_cmdbuff() -> *mut c_char;
    fn nvim_apply_pending_hl_callback();

    // getexline dependencies
    fn nvim_get_exec_from_reg() -> c_int;
    fn vpeekc() -> c_int;
    fn vgetc() -> c_int;
    fn getcmdline(firstc: c_int, count: c_int, indent: c_int, do_concat: bool) -> *mut c_char;
}

// MODE_CMDLINE constant
const MODE_CMDLINE: c_int = 0x0010;

// Aligned storage for CmdlineInfo C struct.
// CmdlineInfo contains pointer fields (alignment 8) and a Callback union.
// Conservative estimate: 512 bytes. Must be 8-byte aligned.
#[repr(C, align(8))]
struct CmdlineInfoStorage {
    bytes: [u8; 512],
}

impl CmdlineInfoStorage {
    fn new() -> Self {
        Self { bytes: [0u8; 512] }
    }
}

// =============================================================================
// rs_command_line_enter
// =============================================================================

/// Rust implementation of `command_line_enter`.
///
/// Main cmdline mode entry/exit orchestration: state setup, autocmd firing,
/// state machine invocation, exit cleanup.
///
/// # Safety
///
/// Calls into C code that manipulates global Neovim state. Must be called
/// from the main editor loop.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_command_line_enter(
    firstc: c_int,
    count: c_int,
    indent: c_int,
    clear_ccline: c_int,
) -> *mut u8 {
    // Increment recursion level (returns new level)
    let level = nvim_cmdline_enter_level_inc();

    // Save and clear cmdpreview
    let save_cmdpreview = nvim_cmdpreview_save_and_clear();

    // Save msg_scroll and State for teardown
    let save_msg_scroll = nvim_get_msg_scroll();
    let save_state = nvim_get_State();

    // Initialize CommandLineState on the stack (Rust-owned)
    let mut state = unsafe { CommandLineState::zeroed() };
    state.state = VimState {
        check: Some(command_line_check_trampoline),
        execute: Some(command_line_execute_trampoline),
    };
    state.firstc = firstc;
    state.count = count;
    state.indent = indent;
    state.save_msg_scroll = save_msg_scroll;
    state.save_state = save_state;
    state.prev_cmdpos = -1;
    state.ignore_drag_release = true;
    let s = std::ptr::addr_of_mut!(state).cast::<c_void>();

    // Get p_icm (must be freed at end)
    state.save_p_icm = nvim_get_p_icm_dup();

    // Initialize incsearch state
    crate::search::rs_init_incsearch_state(std::ptr::addr_of_mut!(state.is_state));

    // Save/clear ccline
    let mut save_ccline_buf = CmdlineInfoStorage::new();
    let save_ccline_ptr = std::ptr::addr_of_mut!(save_ccline_buf).cast::<c_void>();
    let did_save_ccline = nvim_ccline_save_and_clear(save_ccline_ptr, clear_ccline != 0);

    // Handle firstc == -1 special case
    if state.firstc == -1 {
        state.firstc = 0; // NUL
        state.break_ctrl_c = true;
    }

    // Alloc initial cmdbuff FIRST
    alloc_cmdbuff(indent + 50);
    nvim_ccline_enter_init(state.firstc, indent);

    sb_text_start_cmdline();

    // Apply autoindent for :insert/:append (firstc <= 0)
    if state.firstc <= 0 {
        nvim_ccline_apply_indent(indent);
    }

    // Set ccline.level
    nvim_ccline_set_level(level);

    // Check recursion limit
    if level == 50 {
        nvim_emsg_command_too_recursive();
        return do_theend(
            std::ptr::addr_of_mut!(state),
            did_save_ccline,
            save_ccline_ptr,
            save_msg_scroll,
            save_state,
            save_cmdpreview,
        );
    }

    // Initialize xpc
    nvim_ccline_init_xpc(s);

    // RTL: set cmdmsg_rl
    let use_rl = rs_entry_should_use_cmdmsg_rl(
        state.firstc,
        nvim_get_curwin_p_rl(),
        nvim_get_curwin_p_rlc_has_s(),
    );
    nvim_set_cmdmsg_rl(use_rl);

    msg_grid_validate();

    // Don't redirect the typed command
    nvim_set_redir_off(1);

    // Draw prompt if not silent
    if !cmd_silent {
        nvim_gotocmdline();
        rs_redrawcmdprompt();
        nvim_set_ccline_cmdspos(rs_cmd_startcol());
    }

    // Avoid scrolling when called by a recursive do_cmdline()
    nvim_set_msg_scroll(0);

    // Set Mode to MODE_CMDLINE
    nvim_set_State(MODE_CMDLINE);

    // Set langmap mode if needed
    nvim_cmdline_setup_langmap(s, state.firstc);

    setmouse();

    // Set cmdline_type for events
    let cmdline_type = crate::entry::EntryContext::new(firstc, count, indent).cmdline_type();
    state.cmdline_type = cmdline_type;

    // Build firstcbuf for autocmd
    let firstcbuf: [c_char; 2] = [cmdline_type as c_char, 0];

    // Fire CmdlineEnter autocmd
    nvim_cmdline_fire_enter_full(firstcbuf.as_ptr(), level);

    may_trigger_modechanged();

    // Initialize history
    state.hiscnt = nvim_init_history_and_get_hislen();
    state.histype = rs_entry_hist_char2type(state.firstc);

    // Init digraph typeahead
    do_digraph(-1);

    // If there was an error above, reset flags
    if nvim_get_did_emsg_for_redraw() != 0 {
        crate::screen::redrawcmd_rs();
    }

    // Redraw statuslines if not silent/not exmode
    if !cmd_silent && nvim_get_exmode_active() == 0 {
        nvim_cmdline_redraw_statuslines();
    }

    nvim_clear_did_emsg();
    nvim_clear_got_int();

    // Run the state machine
    nvim_state_enter(s);

    // Trigger CmdlineLeavePre if not already triggered
    rs_cmdline_fire_leavepre_autocmd(s, state.c);

    // Fire CmdlineLeave autocmd
    nvim_cmdline_fire_leave_full(s, state.c);

    // Reset cmdmsg_rl
    nvim_set_cmdmsg_rl(0);

    // Post-leave cleanup
    let result_ptr = leave_cleanup(std::ptr::addr_of_mut!(state));

    // Final teardown
    final_teardown(
        std::ptr::addr_of_mut!(state),
        did_save_ccline,
        save_ccline_ptr,
        save_msg_scroll,
        save_state,
        save_cmdpreview,
    );

    result_ptr
}

/// Shared teardown path for the `goto theend` case (recursion error).
///
/// # Safety
///
/// All pointers must be valid.
unsafe fn do_theend(
    state: *mut CommandLineState,
    did_save_ccline: bool,
    save_ccline_ptr: *mut c_void,
    save_msg_scroll: c_int,
    save_state: c_int,
    save_cmdpreview: bool,
) -> *mut u8 {
    final_teardown(
        state,
        did_save_ccline,
        save_ccline_ptr,
        save_msg_scroll,
        save_state,
        save_cmdpreview,
    );
    std::ptr::null_mut()
}

// =============================================================================
// Phase 3: Rust implementations of leave_cleanup and final_teardown
// =============================================================================

/// Post-leave cleanup: PUM, expand, incsearch, history.
/// Returns ccline.cmdbuff (the result string), or NULL.
///
/// # Safety
///
/// state must be a valid pointer to CommandLineState.
unsafe fn leave_cleanup(state: *mut CommandLineState) -> *mut u8 {
    let cs = &mut *state;

    // PUM cleanup
    if cmdline_pum_active() != 0 {
        cmdline_pum_remove(false);
    } else {
        pum_check_clear();
    }
    nvim_wildmenu_cleanup_ccline();
    cs.did_wild_list = false;
    cs.wim_index = 0;

    ExpandCleanup(std::ptr::addr_of_mut!(cs.xpc).cast::<c_void>());
    nvim_ccline_clear_xpc_and_orig();

    crate::search::rs_finish_incsearch_highlighting(
        c_int::from(cs.gotesc),
        std::ptr::addr_of_mut!(cs.is_state),
        0,
    );

    // History
    if nvim_ccline_has_cmdbuff() != 0 {
        let cmdlen = nvim_get_ccline_cmdlen();
        if rs_entry_should_add_to_history(
            cs.histype,
            cmdlen,
            cs.firstc,
            c_int::from(cs.some_key_typed),
        ) != 0
        {
            let sep_char = if cs.histype == hist_type::HIST_SEARCH {
                cs.firstc
            } else {
                0
            };
            nvim_add_to_history_ccline(cs.histype, sep_char);
            if rs_entry_should_save_last_cmdline(cs.firstc) != 0 {
                nvim_save_last_cmdline();
            }
        }

        if cs.gotesc {
            dealloc_cmdbuff();
            nvim_compute_cmdrow_if_not_scrolled();
            if nvim_ccline_get_one_key() == 0 {
                nvim_cmdline_gotesc_msg();
            }
        }
    }

    msg_check();
    nvim_cmdline_check_must_redraw();

    nvim_get_ccline_cmdbuff().cast::<u8>()
}

/// Final teardown: hide cmdline UI, restore ccline, free resources.
///
/// # Safety
///
/// All pointers must be valid.
unsafe fn final_teardown(
    state: *mut CommandLineState,
    did_save_ccline: bool,
    save_ccline_ptr: *mut c_void,
    save_msg_scroll: c_int,
    save_state: c_int,
    save_cmdpreview: bool,
) {
    let cs = &mut *state;

    nvim_set_msg_scroll(save_msg_scroll);
    nvim_set_redir_off(0);

    if cs.some_key_typed {
        nvim_clear_need_wait_return_wrap();
    }

    // Restore p_icm option (restore before free)
    nvim_set_p_icm_option(cs.save_p_icm);
    nvim_set_State(save_state);

    // Restore cmdpreview if it changed
    let current_cmdpreview = nvim_get_cmdpreview();
    nvim_cmdpreview_restore(save_cmdpreview, current_cmdpreview);

    may_trigger_modechanged();
    setmouse();
    sb_text_end_cmdline();

    // Free save_p_icm after restoring option
    xfree(cs.save_p_icm);
    cs.save_p_icm = std::ptr::null_mut();

    nvim_ccline_free_last_colors();

    nvim_cmdline_ui_hide(c_int::from(cs.gotesc));
    nvim_cmdline_status_redraw();

    nvim_cmdline_enter_level_dec();

    nvim_ccline_restore_or_clear(did_save_ccline, save_ccline_ptr);

    xfree(cs.prev_cmdbuff);
    cs.prev_cmdbuff = std::ptr::null_mut();
}

// =============================================================================
// Trampolines: C-callable wrappers for the Rust state machine callbacks
// These match the VimState function pointer signatures.
// command_line_execute and command_line_check are already exported from state.rs.
// =============================================================================

unsafe extern "C" fn command_line_check_trampoline(
    state: *mut crate::command_line_state::VimState,
) -> c_int {
    extern "C" {
        fn command_line_check(state: *mut crate::command_line_state::VimState) -> c_int;
    }
    command_line_check(state)
}

unsafe extern "C" fn command_line_execute_trampoline(
    state: *mut crate::command_line_state::VimState,
    key: c_int,
) -> c_int {
    extern "C" {
        fn command_line_execute(
            state: *mut crate::command_line_state::VimState,
            key: c_int,
        ) -> c_int;
    }
    command_line_execute(state, key)
}

// =============================================================================
// Phase 2: rs_getcmdline_prompt
// =============================================================================

/// Rust implementation of `getcmdline_prompt`.
///
/// Sets up ccline fields for prompt mode and calls rs_command_line_enter.
///
/// # Safety
///
/// Calls into C code that manipulates global Neovim state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_getcmdline_prompt(
    firstc: c_int,
    prompt: *const c_char,
    hl_id: c_int,
    xp_context: c_int,
    xp_arg: *const c_char,
    one_key: c_int,
    mouse_used: *mut bool,
) -> *mut c_char {
    let msg_col_save = msg_col;

    // Save ccline if in use (8-byte-aligned storage for CmdlineInfo)
    let mut save_ccline_buf = CmdlineInfoStorage::new();
    let save_ccline_ptr = std::ptr::addr_of_mut!(save_ccline_buf).cast::<c_void>();
    let did_save_ccline = nvim_ccline_save_and_clear(save_ccline_ptr, true);

    // Set ccline fields for prompt mode.
    nvim_set_ccline_cmdprompt(prompt.cast_mut());
    nvim_set_ccline_hl_id(hl_id);
    nvim_set_ccline_xp_context(xp_context);
    nvim_set_ccline_xp_arg(xp_arg);
    // input_fn = (firstc == '@')
    nvim_set_ccline_input_fn(c_int::from(firstc == b'@' as c_int));
    // highlight_callback = CALLBACK_NONE (not passed across FFI; handled by C wrapper)
    nvim_set_ccline_one_key(one_key);
    nvim_set_ccline_mouse_used_ptr(mouse_used);

    // Apply pending highlight callback (set by C getcmdline_prompt before calling us).
    nvim_apply_pending_hl_callback();

    let cmd_silent_saved = cmd_silent;
    let msg_silent_saved = msg_silent;
    msg_silent = 0;
    cmd_silent = false; // Want to see the prompt

    // Call command_line_enter with clear_ccline=false (0)
    let ret = rs_command_line_enter(firstc, 1, 0, 0);

    nvim_set_ccline_redraw_state(0); // kCmdRedrawNone

    if did_save_ccline {
        nvim_ccline_restore(save_ccline_ptr);
    }

    msg_silent = msg_silent_saved;
    cmd_silent = cmd_silent_saved;

    // Restore msg_col only if we're in a recursive cmdline
    if !nvim_get_ccline_cmdbuff().is_null() {
        msg_col = msg_col_save;
    }

    ret.cast::<c_char>()
}

// =============================================================================
// rs_cmdline_fire_leavepre_autocmd: migrated from ex_getln.c
// =============================================================================

const EVENT_CMDLINELEAVEPRE: c_int = 28;

unsafe extern "C" {
    fn rs_trigger_cmd_autocmd(typechar: c_int, evt: c_int);
}

/// Rust replacement for `nvim_cmdline_fire_leavepre_autocmd` in ex_getln.c.
///
/// Fires the CmdlineLeavePre autocmd if not already triggered.
/// Sets v:char to c_val first. Returns 1 if triggered, 0 if already done.
///
/// # Safety
///
/// `s` must be a valid pointer to `CommandLineState`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_fire_leavepre_autocmd(s: *mut c_void, c_val: c_int) -> c_int {
    let cs = &mut *s.cast::<crate::command_line_state::CommandLineState>();
    if cs.event_cmdlineleavepre_triggered {
        return 0;
    }
    set_vim_var_char(c_val);
    rs_trigger_cmd_autocmd(cs.cmdline_type, EVENT_CMDLINELEAVEPRE);
    cs.event_cmdlineleavepre_triggered = true;
    1
}

// =============================================================================
// getexline: migrated from ex_getln.c
// =============================================================================

/// Get an Ex command line for the ":" command.
///
/// When executing a register, removes the leading ':' from each line.
/// Then delegates to getcmdline().
///
/// # Safety
///
/// Calls C functions vpeekc, vgetc, getcmdline.
#[no_mangle]
pub unsafe extern "C" fn getexline(
    c: c_int,
    _cookie: *mut c_void,
    indent: c_int,
    do_concat: bool,
) -> *mut c_char {
    // When executing a register, remove ':' that's in front of each line.
    if nvim_get_exec_from_reg() != 0 && vpeekc() == b':' as c_int {
        vgetc();
    }
    getcmdline(c, 1, indent, do_concat)
}
