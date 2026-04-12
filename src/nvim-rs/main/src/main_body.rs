//! Main body extraction: server setup, post-startup, and event loop entry.
//!
//! Implements `rs_main_server_setup`, `rs_main_post_startup`, and
//! `rs_main_enter_loop`, which together replace the large body of C `main()`.

use crate::setup::MparmT;
use std::ffi::{c_char, c_int};

// =============================================================================
// Constants
// =============================================================================

// starting values (from globals.h)
const NO_BUFFERS: c_int = 1;

// =============================================================================
// C accessor function declarations
// =============================================================================

unsafe extern "C" {
    // === Server setup ===
    fn server_init(addr: *mut c_char) -> bool;
    fn win_init_size();
    fn nvim_diff_win_options_firstwin(); // diff_win_options(firstwin, false)
    fn nvim_setup_cmdline_row(); // sets cmdline_row/msg_row from Rows/p_ch
    fn default_grid_alloc();
    fn set_init_2(headless: bool);
    fn init_highlight(both: bool, reset: bool);
    fn ui_comp_syn_init();
    fn input_start();
    fn remote_ui_wait_for_attach(listen_and_embed: bool);
    fn nvim_sync_firstwin_height(); // firstwin->w_prev_height = firstwin->w_height
    fn screenclear();
    fn win_new_screensize();
    fn edit_stdin(parmp: *const MparmT) -> bool;
    fn open_scriptin(fname: *mut c_char) -> bool;
    fn nlua_init_defaults();
    // For inline nvim_open_scriptout
    fn os_fopen(path: *const c_char, mode: *const c_char) -> *mut std::ffi::c_void;
    fn nvim_vimrc_is_none(parmp: *const MparmT) -> bool; // strequal(use_vimrc, "NONE")
    fn nvim_set_p_lpl(val: bool); // p_lpl = val
    fn filetype_plugin_enable();
    fn filetype_maybe_enable();
    fn syn_maybe_enable();

    // === Post-startup ===
    fn nvim_set_vv_vim_did_init(); // set_vim_var_nr(VV_VIM_DID_INIT, 1)
    fn load_plugins();
    fn set_init_3();
    fn nvim_set_p_uc_zero(); // p_uc = 0
    fn nvim_set_p_ut_one(); // p_ut = 1
    fn nvim_p_shada_is_empty() -> bool; // *p_shada == NUL
    fn nvim_vv_oldfiles_is_null() -> bool;
    fn nvim_init_vv_oldfiles(); // set_vim_var_list(VV_OLDFILES, tv_list_alloc(0))
    fn setmouse();
    fn nvim_redraw_later_curwin(); // redraw_later(curwin, UPD_VALID)
    fn nvim_clear_vv_swapcommand(); // set_vim_var_string(VV_SWAPCOMMAND, NULL, -1)
    fn nvim_set_curwin_lnum_to_last_line(); // curwin->w_cursor.lnum = curbuf->b_ml.ml_line_count
    fn nvim_apply_event_bufenter(); // apply_autocmds(EVENT_BUFENTER, ...)
    fn setpcmark();
    fn nvim_diff_win_options_curtab(); // FOR_ALL_WINDOWS_IN_TAB(curtab) diff_win_options
    fn shorten_fnames(force: bool);
    fn nvim_clear_starting_state(); // starting=0, RedrawingDisabled=0
    fn redraw_all_later(type_: c_int);
    fn do_autochdir();

    // === Enter loop ===
    fn nvim_set_vv_vim_did_enter(); // set_vim_var_nr(VV_VIM_DID_ENTER, 1)
    fn nvim_apply_event_vimenter(); // apply_autocmds(EVENT_VIMENTER, ...)
    fn do_autocmd_uienter_all();
    fn nvim_set_reg_var_default(); // set_reg_var(get_default_register_name())
    fn nvim_curwin_needs_scrollbind_sync() -> bool; // curwin->w_p_diff && curwin->w_p_scb
    fn nvim_update_topline_curwin(); // update_topline(curwin)
    fn check_scrollbind(topline_diff: c_int, leftcol_diff: c_int);
    fn nvim_get_restart_edit() -> c_int;
    fn nvim_stuffchar_K_NOP(); // stuffcharReadbuff(K_NOP)
    fn nvim_has_clipboard_flags() -> bool; // cb_flags & (Unnamed|Unnamedplus)
    fn eval_has_provider(name: *const c_char, silent: bool) -> bool;
    fn nlua_exec_file(fname: *mut c_char) -> bool;
    fn msg_putchar(c: c_int);
    fn normal_enter(cmdwin: bool, noexmode: bool) -> !;
    fn getout(exitval: c_int) -> !;
    fn nvim_get_ioerr_ptr() -> *const c_char;

    // Already-accessible Rust externs (via C linker)
    fn rs_exe_pre_commands(parmp: *mut MparmT);
    fn rs_source_startup_scripts(parmp: *const MparmT);
    fn rs_set_window_layout(parmp: *mut MparmT);
    fn rs_recover_names(
        fname: *const c_char,
        do_list: c_int,
        ret_list: *mut std::ffi::c_void,
        nr: c_int,
        fname_out: *mut *mut c_char,
    ) -> c_int;
    fn rs_shada_read_everything(fname: *const c_char, forceit: bool, missing_ok: bool) -> c_int;
    fn rs_handle_quickfix(parmp: *mut MparmT);
    fn rs_read_stdin();
    fn rs_create_windows(parmp: *mut MparmT);
    fn rs_edit_buffers(parmp: *mut MparmT, cwd: *mut c_char);
    fn rs_handle_tag(tagname: *mut c_char);
    fn rs_exe_commands(parmp: *mut MparmT);
    fn rs_qf_jump_newwin(
        tv: *mut std::ffi::c_void,
        dir: c_int,
        errornr: c_int,
        forceit: bool,
        newwin: bool,
    );
    fn os_exit(r: c_int) -> !;
    fn rs_mainerr(msg1: *const c_char, msg2: *const c_char, msg3: *const c_char) -> !;
    fn xfree(ptr: *mut std::ffi::c_void);

    // TIME_MSG shim (no-op if time_fd == NULL)
    fn nvim_time_msg(msg: *const c_char);

    // Globals
    static mut silent_mode: bool;
    static mut full_screen: bool;
    static mut headless_mode: bool;
    static mut embedded_mode: bool;
    static mut exmode_active: bool;
    static mut stdin_isatty: bool;
    static mut starting: c_int;
    static mut msg_scroll: c_int;
    static mut no_wait_return: c_int;
    static mut recoverymode: bool;
    static mut msg_didout: bool;
    static mut RedrawingDisabled: c_int;
    static mut debug_break_level: c_int;
    // scriptout FILE* global (opaque)
    static mut scriptout: *mut std::ffi::c_void;
    static stderr: *mut std::ffi::c_void;
    fn fprintf(stream: *mut std::ffi::c_void, fmt: *const c_char, ...) -> c_int;
}

// =============================================================================
// Phase 1: rs_main_server_setup
// =============================================================================

/// Server setup: from `server_init` through `syn_maybe_enable`.
///
/// Extracted from C `main()` lines ~443-558.
///
/// # Safety
/// Must be called from main thread, after `nlua_init`, before post-startup.
#[no_mangle]
pub unsafe extern "C" fn rs_main_server_setup(parmp: *mut MparmT) {
    let p = &mut *parmp;

    if !server_init(p.listen_addr) {
        rs_mainerr(nvim_get_ioerr_ptr(), std::ptr::null(), std::ptr::null());
    }

    nvim_time_msg(c"expanding arguments".as_ptr());

    if p.diff_mode != 0 && p.window_count == -1 {
        p.window_count = 0; // open up to 3 windows
    }
    // Don't redraw until much later.
    RedrawingDisabled += 1;

    // setbuf(stdout, NULL) is a C-only call, handled by C main() before calling us.

    full_screen = !silent_mode;

    win_init_size();
    if p.diff_mode != 0 {
        nvim_diff_win_options_firstwin();
    }

    nvim_setup_cmdline_row(); // cmdline_row = Rows - p_ch; msg_row = cmdline_row

    default_grid_alloc();
    set_init_2(headless_mode);
    nvim_time_msg(c"inits 2".as_ptr());

    msg_scroll = 1; // true
    no_wait_return = 1; // true

    init_highlight(true, false);
    ui_comp_syn_init();
    nvim_time_msg(c"init highlight".as_ptr());

    debug_break_level = p.use_debug_break_level;

    // Read ex-commands if invoked with "-es".
    if !stdin_isatty && !p.input_istext && silent_mode && exmode_active {
        input_start();
    }

    // Wait for UIs.
    let use_remote_ui = embedded_mode && !headless_mode;
    let listen_and_embed = !p.listen_addr.is_null();
    if use_remote_ui {
        nvim_time_msg(c"waiting for UI".as_ptr());
        remote_ui_wait_for_attach(!listen_and_embed);
        nvim_time_msg(c"done waiting for UI".as_ptr());
        nvim_sync_firstwin_height();
    }

    // Prepare screen.
    starting = NO_BUFFERS;
    screenclear();
    win_new_screensize();
    nvim_time_msg(c"clear screen".as_ptr());

    // Handle "foo | nvim". EDIT_FILE may be overwritten now.
    if edit_stdin(parmp) {
        p.edit_type = EDIT_STDIN;
    }

    if !p.scriptin.is_null() && !open_scriptin(p.scriptin) {
        os_exit(2);
    }
    // Inline of nvim_open_scriptout: open the scriptout file.
    if !p.scriptout.is_null() {
        let mode = if p.scriptout_append { c"ab" } else { c"wb" };
        scriptout = os_fopen(p.scriptout, mode.as_ptr());
        if scriptout.is_null() {
            fprintf(
                stderr,
                c"Cannot open for script output: \"%s\"\n".as_ptr(),
                p.scriptout,
            );
            os_exit(2);
        }
    }

    nlua_init_defaults();
    nvim_time_msg(c"init default mappings & autocommands".as_ptr());

    let vimrc_none = nvim_vimrc_is_none(parmp);

    // Reset 'loadplugins' for "-u NONE" before "--cmd" arguments.
    if vimrc_none {
        nvim_set_p_lpl(p.clean);
    }
    // (if not vimrc_none, p_lpl remains as-is)

    rs_exe_pre_commands(parmp);
    nvim_time_msg(c"--cmd commands".as_ptr());

    if !vimrc_none || p.clean {
        filetype_plugin_enable();
    }

    rs_source_startup_scripts(parmp);
    nvim_time_msg(c"sourcing vimrc file(s)".as_ptr());

    if !vimrc_none || p.clean {
        filetype_maybe_enable();
        syn_maybe_enable();
    }
}

// EDIT_STDIN value (must match C enum in main.c)
const EDIT_STDIN: c_int = 2;

// =============================================================================
// Phase 1: rs_main_post_startup
// =============================================================================

/// Post-startup: from `set_vim_var_nr(VV_VIM_DID_INIT)` through `do_autochdir`.
///
/// Extracted from C `main()` lines ~560-699.
///
/// # Safety
/// Must be called after `rs_main_server_setup`, before `rs_main_enter_loop`.
/// `fname` must be a valid C string or null. `cwd` may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_main_post_startup(
    parmp: *mut MparmT,
    fname: *const c_char,
    cwd: *mut c_char,
    use_remote_ui: bool,
) {
    let _ = use_remote_ui; // used only for time_msg gate (TIME_MSG is no-op outside C)
    let p = &mut *parmp;

    nvim_set_vv_vim_did_init();

    load_plugins();

    rs_set_window_layout(parmp);

    // Recovery mode without a file name: list swap files.
    if recoverymode && fname.is_null() {
        rs_recover_names(
            std::ptr::null(),
            1,
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
        );
        os_exit(0);
    }

    set_init_3();
    nvim_time_msg(c"inits 3".as_ptr());

    if p.no_swap_file != 0 {
        nvim_set_p_uc_zero();
    }

    if silent_mode {
        nvim_set_p_ut_one();
    }

    // Read ShaDa file.
    if !nvim_p_shada_is_empty() {
        rs_shada_read_everything(std::ptr::null(), false, true);
        nvim_time_msg(c"reading ShaDa".as_ptr());
    }
    if nvim_vv_oldfiles_is_null() {
        nvim_init_vv_oldfiles();
    }

    // Load error file for -q.
    rs_handle_quickfix(parmp);
    if p.edit_type == EDIT_QF {
        nvim_time_msg(c"reading errorfile".as_ptr());
    }

    // Start putting things on screen.
    starting = NO_BUFFERS;
    no_wait_return = 0; // false
    if !exmode_active {
        msg_scroll = 0; // false
    }

    // Read file from stdin if needed.
    if p.edit_type == EDIT_STDIN && !recoverymode {
        rs_read_stdin();
    }

    setmouse();
    nvim_redraw_later_curwin();
    no_wait_return = 1; // true

    rs_create_windows(parmp);
    nvim_time_msg(c"opening buffers".as_ptr());

    nvim_clear_vv_swapcommand();

    if exmode_active {
        nvim_set_curwin_lnum_to_last_line();
    }

    nvim_apply_event_bufenter();
    nvim_time_msg(c"BufEnter autocommands".as_ptr());
    setpcmark();

    if p.edit_type == EDIT_QF {
        rs_qf_jump_newwin(std::ptr::null_mut(), 0, 0, false, false);
        nvim_time_msg(c"jump to first error".as_ptr());
    }

    rs_edit_buffers(parmp, cwd);
    xfree(cwd as *mut std::ffi::c_void);

    if p.diff_mode != 0 {
        nvim_diff_win_options_curtab();
    }

    shorten_fnames(false);

    rs_handle_tag(p.tagname);
    if !p.tagname.is_null() {
        nvim_time_msg(c"jumping to tag".as_ptr());
    }

    if p.n_commands > 0 {
        rs_exe_commands(parmp);
        nvim_time_msg(c"executing command arguments".as_ptr());
    }

    nvim_clear_starting_state(); // starting=0, RedrawingDisabled=0
    redraw_all_later(UPD_NOT_VALID);
    no_wait_return = 0; // false

    do_autochdir();
}

// UPD_NOT_VALID = 40 (from drawscreen.h)
const UPD_NOT_VALID: c_int = 40;

// EDIT_QF value (must match C enum in main.c)
const EDIT_QF: c_int = 4;

// =============================================================================
// Phase 1: rs_main_enter_loop
// =============================================================================

/// Main loop entry: from `set_vim_var_nr(VV_VIM_DID_ENTER)` through `normal_enter`.
///
/// Extracted from C `main()` lines ~701-764.
/// This function does not return.
///
/// # Safety
/// Must be called after `rs_main_post_startup`.
#[no_mangle]
pub unsafe extern "C" fn rs_main_enter_loop(parmp: *mut MparmT, use_remote_ui: bool) -> ! {
    let p = &mut *parmp;

    nvim_set_vv_vim_did_enter();
    nvim_apply_event_vimenter();
    nvim_time_msg(c"VimEnter autocommands".as_ptr());

    if use_remote_ui {
        do_autocmd_uienter_all();
        nvim_time_msg(c"UIEnter autocommands".as_ptr());
    }

    // MSWIN: os_icon_init / os_title_save omitted (platform-specific, handled in C).

    nvim_set_reg_var_default();

    if nvim_curwin_needs_scrollbind_sync() {
        nvim_update_topline_curwin();
        check_scrollbind(0, 0);
        nvim_time_msg(c"diff scrollbinding".as_ptr());
    }

    if nvim_get_restart_edit() != 0 {
        nvim_stuffchar_K_NOP();
    }

    if nvim_has_clipboard_flags() {
        eval_has_provider(c"clipboard".as_ptr(), false);
    }

    if !p.luaf.is_null() {
        msg_scroll = 1; // true
        let lua_ok = nlua_exec_file(p.luaf);
        nvim_time_msg(c"executing Lua -l script".as_ptr());
        if msg_didout {
            msg_putchar(b'\n' as c_int);
            msg_didout = false;
        }
        getout(if lua_ok { 0 } else { 1 });
    }

    nvim_time_msg(c"before starting main loop".as_ptr());

    // Main loop: never returns.
    normal_enter(false, false)
}
