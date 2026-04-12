//! rs_command_line_enter: Rust implementation of command_line_enter and
//! rs_getcmdline_prompt: Rust implementation of getcmdline_prompt
//!
//! Migrated from ex_getln.c (Phase 1 + Phase 2).

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::{c_char, c_int, c_void};

use crate::command_line_state::{CommandLineState, VimState};

// =============================================================================
// FFI declarations for Phase 1
// =============================================================================

unsafe extern "C" {
    // Level / cmdpreview
    fn nvim_cmdline_enter_level_inc() -> c_int;
    fn nvim_cmdpreview_save_and_clear() -> bool;

    // ccline save/restore
    // CmdlineInfo is opaque from Rust side; we use a byte array large enough.
    // Size from C: sizeof(CmdlineInfo). We pass a pointer to stack-allocated storage.
    fn nvim_ccline_save_and_clear(save_out: *mut c_void, clear_ccline_flag: bool) -> bool;
    fn nvim_ccline_restore(save_out: *const c_void);

    // ccline init
    fn nvim_ccline_enter_init(firstc: c_int, indent: c_int);
    fn nvim_ccline_set_level(level: c_int);
    fn nvim_ccline_apply_indent(indent: c_int);
    fn nvim_ccline_init_xpc(s: *mut c_void);

    // alloc_cmdbuff (already in Rust, exported as extern C)
    fn alloc_cmdbuff(len: c_int);

    // sb_text wrappers
    fn nvim_sb_text_start_cmdline();

    // cmdmsg_rl / msg_grid_validate / redir_off
    fn nvim_set_cmdmsg_rl(val: c_int);
    fn nvim_msg_grid_validate();
    fn nvim_set_redir_off(val: c_int);

    // gotocmdline / setmouse / may_trigger_modechanged
    fn nvim_gotocmdline();
    fn nvim_setmouse();
    fn nvim_may_trigger_modechanged();

    // State / msg_scroll / clear flags
    fn nvim_set_State(val: c_int);
    fn nvim_get_msg_scroll() -> c_int;
    fn nvim_set_msg_scroll(val: c_int);
    fn nvim_clear_did_emsg();
    fn nvim_clear_got_int();
    fn nvim_get_did_emsg_for_redraw() -> c_int;

    // p_icm
    fn nvim_get_p_icm_dup() -> *mut c_char;

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

    // rs_redrawcmdprompt, rs_cmd_startcol, nvim_get_cmd_silent
    fn rs_redrawcmdprompt();
    fn rs_cmd_startcol() -> c_int;
    fn nvim_set_ccline_cmdspos(spos: c_int);
    fn nvim_get_cmd_silent() -> c_int;

    // Autocmds
    fn nvim_cmdline_fire_enter_full(firstcbuf: *const c_char, level: c_int) -> c_int;
    fn nvim_cmdline_fire_leavepre_autocmd(s: *mut c_void, c_val: c_int) -> c_int;
    fn nvim_cmdline_fire_leave_full(s: *mut c_void, c_val: c_int) -> c_int;

    // History
    fn nvim_init_history_and_get_hislen() -> c_int;
    fn rs_entry_hist_char2type(firstc: c_int) -> c_int;

    // Digraph init
    fn nvim_do_digraph_init();

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

    // Cleanup helpers (still needed until leave_cleanup is fully replaced)
    fn nvim_cmdline_leave_cleanup(s: *mut c_void) -> *mut u8;
    fn nvim_cmdline_final_teardown(
        s: *mut c_void,
        did_save_ccline: bool,
        save_ccline: *mut c_void,
        save_msg_scroll: c_int,
        save_state: c_int,
        save_cmdpreview: bool,
        err: *mut c_void,
    );
}

// Phase 3 cleanup thin wrappers (declared for upcoming Rust migration; not yet used)
#[allow(dead_code)]
unsafe extern "C" {
    fn nvim_cmdline_pum_active_check() -> c_int;
    fn nvim_cmdline_pum_remove_noconfirm();
    fn nvim_pum_check_clear_wrap();
    fn nvim_wildmenu_cleanup_ccline();
    fn nvim_expand_cleanup_xpc(xpc: *mut c_void);
    fn nvim_ccline_clear_xpc_and_orig();
    fn nvim_add_to_history_ccline(histype: c_int, sep_char: c_int);
    fn nvim_save_last_cmdline();
    fn nvim_compute_cmdrow_if_not_scrolled();
    fn nvim_cmdline_gotesc_msg();
    fn nvim_cmdline_check_must_redraw();
    fn nvim_ccline_has_cmdbuff() -> c_int;
    fn nvim_ccline_get_one_key() -> c_int;
    fn nvim_get_msg_scrolled() -> c_int;
    fn nvim_clear_need_wait_return_wrap();
    fn nvim_ccline_free_last_colors();
    fn nvim_cmdline_ui_hide(gotesc: c_int);
    fn nvim_cmdline_status_redraw();
    fn nvim_ccline_restore_or_clear(did_save: bool, save_ccline_in: *const c_void);
    // Accessors for CommandLineState fields needed in cleanup
    fn nvim_cls_get_histype(s: *mut c_void) -> c_int;
    fn nvim_cls_get_some_key_typed(s: *mut c_void) -> c_int;
    fn nvim_cls_get_save_p_icm(s: *mut c_void) -> *mut c_char;
    fn nvim_cls_xfree_save_p_icm(s: *mut c_void);
    fn nvim_cls_xfree_prev_cmdbuff_phase3(s: *mut c_void);
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
    // clear_ccline controls whether to clear ccline if not in use (recursive entry).
    // - rs_getcmdline_prompt sets ccline fields before calling us (clear_ccline=false case)
    // - nvim_ccline_save_and_clear handles both cases: saves if in use, clears if not
    // The C assertion "assert(clear_ccline)" fires if cmdbuff!=NULL && clear_ccline==false,
    // which is an invariant maintained by the caller.

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

    // Save/clear ccline. When clear_ccline=false (getcmdline_prompt case),
    // caller pre-sets ccline fields and we must not clear them.
    let mut save_ccline_buf = CmdlineInfoStorage::new();
    let save_ccline_ptr = std::ptr::addr_of_mut!(save_ccline_buf).cast::<c_void>();
    let did_save_ccline = nvim_ccline_save_and_clear(save_ccline_ptr, clear_ccline != 0);

    // Handle firstc == -1 special case
    if state.firstc == -1 {
        state.firstc = 0; // NUL
        state.break_ctrl_c = true;
    }

    // Alloc initial cmdbuff FIRST (nvim_ccline_enter_init writes cmdbuff[0]=NUL)
    alloc_cmdbuff(indent + 50);
    // Initialize ccline fields (overstrike, cmdfirstc, cmdindent, cmdlen, cmdpos, etc.)
    // This writes cmdbuff[0]=NUL; cmdbuff must already be allocated.
    nvim_ccline_enter_init(state.firstc, indent);

    nvim_sb_text_start_cmdline();

    // Apply autoindent for :insert/:append (firstc <= 0)
    if state.firstc <= 0 {
        nvim_ccline_apply_indent(indent);
    }

    // Set ccline.level
    nvim_ccline_set_level(level);

    // Check recursion limit
    if level == 50 {
        nvim_emsg_command_too_recursive();
        // goto theend equivalent: jump to teardown
        return do_theend(
            s,
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

    nvim_msg_grid_validate();

    // Don't redirect the typed command
    nvim_set_redir_off(1);

    // Draw prompt if not silent
    if nvim_get_cmd_silent() == 0 {
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

    nvim_setmouse();

    // Set cmdline_type for events
    let cmdline_type = crate::entry::EntryContext::new(firstc, count, indent).cmdline_type();
    state.cmdline_type = cmdline_type;

    // Build firstcbuf for autocmd
    let firstcbuf: [c_char; 2] = [cmdline_type as c_char, 0];

    // Fire CmdlineEnter autocmd
    nvim_cmdline_fire_enter_full(firstcbuf.as_ptr(), level);

    nvim_may_trigger_modechanged();

    // Initialize history
    state.hiscnt = nvim_init_history_and_get_hislen();
    state.histype = rs_entry_hist_char2type(state.firstc);

    // Init digraph typeahead
    nvim_do_digraph_init();

    // If there was an error above, reset flags (we want to type and execute commands)
    if nvim_get_did_emsg_for_redraw() != 0 {
        crate::screen::redrawcmd_rs();
    }

    // Redraw statuslines if not silent/not exmode
    if nvim_get_cmd_silent() == 0 && nvim_get_exmode_active() == 0 {
        nvim_cmdline_redraw_statuslines();
    }

    nvim_clear_did_emsg();
    nvim_clear_got_int();

    // Run the state machine
    nvim_state_enter(s);

    // Trigger CmdlineLeavePre if not already triggered
    nvim_cmdline_fire_leavepre_autocmd(s, state.c);

    // Fire CmdlineLeave autocmd
    nvim_cmdline_fire_leave_full(s, state.c);

    // Reset cmdmsg_rl
    nvim_set_cmdmsg_rl(0);

    // Post-leave cleanup (PUM, expand, incsearch, history, gotesc handling)
    // Returns ccline.cmdbuff (the result string)
    let result_ptr = nvim_cmdline_leave_cleanup(s);

    // Final teardown (hide UI, restore ccline, free)
    nvim_cmdline_final_teardown(
        s,
        did_save_ccline,
        save_ccline_ptr,
        save_msg_scroll,
        save_state,
        save_cmdpreview,
        std::ptr::null_mut(), // no pending error from Rust side
    );

    result_ptr
}

/// Shared teardown path for the `goto theend` case (recursion error).
///
/// # Safety
///
/// All pointers must be valid.
unsafe fn do_theend(
    s: *mut c_void,
    did_save_ccline: bool,
    save_ccline_ptr: *mut c_void,
    save_msg_scroll: c_int,
    save_state: c_int,
    save_cmdpreview: bool,
) -> *mut u8 {
    // theend: xfree(s->save_p_icm), xfree ccline.last_colors, kv_destroy colors,
    // ui_call_cmdline_hide, status_redraw_all, cmdline_enter_level--
    // nvim_cmdline_final_teardown handles most of this.
    // But first we need to free save_p_icm which was allocated before entering theend.
    // nvim_cmdline_final_teardown will xfree cs->save_p_icm for us.
    nvim_cmdline_final_teardown(
        s,
        did_save_ccline,
        save_ccline_ptr,
        save_msg_scroll,
        save_state,
        save_cmdpreview,
        std::ptr::null_mut(),
    );
    std::ptr::null_mut()
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

unsafe extern "C" {
    // msg_col / msg_silent — direct static access
    static mut msg_col: c_int;
    static mut msg_silent: c_int;
    fn nvim_set_cmd_silent(val: c_int);

    // ccline field setters for prompt
    // Use *mut c_char to match other extern declarations in the codebase
    fn nvim_set_ccline_cmdprompt(prompt: *mut c_char);
    fn nvim_set_ccline_hl_id(hl_id: c_int);
    fn nvim_set_ccline_xp_context(context: c_int);
    // xp_arg setter
    fn nvim_set_ccline_xp_arg(arg: *const c_char);
    fn nvim_set_ccline_input_fn(val: c_int);
    fn nvim_set_ccline_one_key(val: c_int);
    fn nvim_set_ccline_mouse_used_ptr(ptr: *mut bool);
    fn nvim_set_ccline_redraw_state(state: c_int);
    fn nvim_get_ccline_cmdbuff() -> *mut c_char;
    fn nvim_apply_pending_hl_callback();
}

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
    // prompt_id will be overwritten by nvim_ccline_enter_init inside rs_command_line_enter.
    // The following fields survive (not touched by nvim_ccline_enter_init):
    // cmdprompt, hl_id, xp_context, xp_arg, input_fn, one_key, mouse_used.
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
    // The Callback union cannot cross the FFI boundary; C stores it in a static,
    // and we apply it here after ccline fields are initialized.
    nvim_apply_pending_hl_callback();

    let cmd_silent_saved = nvim_get_cmd_silent();
    let msg_silent_saved = msg_silent;
    msg_silent = 0;
    nvim_set_cmd_silent(0); // Want to see the prompt

    // Call command_line_enter with clear_ccline=false (0)
    let ret = rs_command_line_enter(firstc, 1, 0, 0);

    nvim_set_ccline_redraw_state(0); // kCmdRedrawNone

    if did_save_ccline {
        nvim_ccline_restore(save_ccline_ptr);
    }

    msg_silent = msg_silent_saved;
    nvim_set_cmd_silent(cmd_silent_saved);

    // Restore msg_col only if we're in a recursive cmdline
    if !nvim_get_ccline_cmdbuff().is_null() {
        msg_col = msg_col_save;
    }

    ret.cast::<c_char>()
}
