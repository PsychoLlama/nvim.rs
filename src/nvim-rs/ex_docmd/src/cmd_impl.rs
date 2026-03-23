//! Phase 1 command implementations migrated from ex_docmd.c.
//!
//! Contains Rust implementations of static command handlers:
//! ex_terminal, ex_checkhealth, ex_syncbind, ex_read, ex_detach, ex_connect,
//! ex_tabs, ex_restart, ex_tabonly, ex_pclose, ex_pedit, ex_pbuffer,
//! ex_stag, ex_tag, ex_ptag, ex_goto, ex_findpat.

use std::ffi::{c_char, c_int, c_void};

use crate::ExArgHandle;

// =============================================================================
// Type aliases
// =============================================================================

type LinenrT = i32;
type WinHandle = *mut c_void;

// =============================================================================
// Constants
// =============================================================================

// find_pattern_in_path type constants
const FIND_ANY: c_int = 1;
const FIND_DEFINE: c_int = 2;

// find_pattern_in_path action constants
const ACTION_SHOW: c_int = 1;
const ACTION_GOTO: c_int = 2;
const ACTION_SPLIT: c_int = 3;
const ACTION_SHOW_ALL: c_int = 4;

// Tag type constants
const DT_TAG: c_int = 1;
const DT_POP: c_int = 2;
const DT_NEXT: c_int = 3;
const DT_PREV: c_int = 4;
const DT_FIRST: c_int = 5;
const DT_SELECT: c_int = 7;
const DT_JUMP: c_int = 9;
const DT_LTAG: c_int = 11;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // eap accessors (already in commands.rs but we need them locally)
    fn nvim_eap_get_arg(eap: ExArgHandle) -> *mut c_char;
    fn nvim_eap_get_cmdidx(eap: ExArgHandle) -> c_int;
    fn nvim_eap_get_forceit(eap: ExArgHandle) -> bool;
    fn nvim_eap_get_line1(eap: ExArgHandle) -> LinenrT;
    fn nvim_eap_get_line2(eap: ExArgHandle) -> LinenrT;
    fn nvim_eap_get_addr_count(eap: ExArgHandle) -> c_int;
    fn nvim_eap_get_skip(eap: ExArgHandle) -> c_int;
    fn nvim_eap_set_errmsg_const(eap: ExArgHandle, msg: *const c_char);
    fn nvim_eap_set_nextcmd(eap: ExArgHandle, p: *mut c_char);

    // cmdnames accessor
    fn nvim_docmd_cmdnames_name(idx: c_int) -> *mut c_char;

    // tag functions
    fn rs_do_tag(tag: *mut c_char, typ: c_int, count: c_int, forceit: c_int, verbose: bool);

    // goto_byte (Rust, callable from Rust via no_mangle)
    #[link_name = "rs_goto_byte"]
    fn nvim_cmd_goto_byte(cnt: c_int);

    // preview window globals
    static mut g_do_tagpreview: c_int;
    static p_pvh: std::ffi::c_long;

    // postponed_split globals
    static mut postponed_split: c_int;
    static mut postponed_split_flags: c_int;
    static mut postponed_split_tab: c_int;

    // cmdmod accessors
    fn nvim_docmd_get_cmdmod_cmod_split() -> c_int;
    fn nvim_docmd_get_cmdmod_cmod_tab() -> c_int;

    // Window iteration (firstwin-based, for curtab)
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;

    // Window preview flag accessor (returns c_int, 0/1)
    fn nvim_win_get_p_pvw(wp: WinHandle) -> c_int;

    // ex_win_close accessor (with NULL tabpage)
    fn nvim_docmd_ex_win_close(forceit: bool, win: WinHandle);

    // Prepare / restore preview window
    fn nvim_docmd_prepare_preview_window();
    fn nvim_docmd_back_to_current_window(curwin_save: WinHandle);

    // do_exedit wrapper
    fn nvim_docmd_do_exedit(eap: ExArgHandle);

    // do_exbuffer wrapper
    fn nvim_docmd_do_exbuffer(eap: ExArgHandle);

    // find_pattern_in_path (via rs_find_pattern_in_path)
    #[link_name = "rs_find_pattern_in_path"]
    fn nvim_cmd_find_pattern_in_path(
        ptr: *const c_char,
        dir: c_int,
        len: usize,
        whole: c_int,
        skip_comments: c_int,
        typ: c_int,
        count: c_int,
        action: c_int,
        start_lnum: LinenrT,
        end_lnum: LinenrT,
        forceit: c_int,
        silent: c_int,
    );

    // skip_regexp
    fn rs_skip_regexp(startp: *mut c_char, delim: c_int, magic: c_int) -> *mut c_char;

    // magic check
    #[link_name = "rs_magic_isset"]
    fn nvim_cmd_magic_isset() -> c_int;

    // check_nextcmd
    fn nvim_check_nextcmd(p: *const c_char) -> *mut c_char;

    // getdigits
    fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int;
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;

    // Complex implementation wrappers (C delegates for complex functions)
    fn nvim_docmd_ex_tabs_impl(eap: ExArgHandle);
    fn nvim_docmd_ex_syncbind_impl(eap: ExArgHandle);
    fn nvim_docmd_ex_read_impl(eap: ExArgHandle);
    fn nvim_docmd_ex_detach_impl(eap: ExArgHandle);
    fn nvim_docmd_ex_connect_impl(eap: ExArgHandle);
    fn nvim_docmd_ex_checkhealth_impl(eap: ExArgHandle);
    fn nvim_docmd_ex_terminal_impl(eap: ExArgHandle);
    fn nvim_docmd_ex_restart_impl(eap: ExArgHandle);
    fn nvim_docmd_ex_tabonly_impl(eap: ExArgHandle);

    // curwin accessor
    fn nvim_get_curwin() -> WinHandle;
}

/// Ends-excmd check (inline, same as in lib.rs)
#[inline]
fn ends_excmd(c: i32) -> bool {
    c == 0 || c == b'|' as i32 || c == b'"' as i32 || c == b'\n' as i32
}

// =============================================================================
// Phase 1: Simple command implementations
// =============================================================================

/// `:goto` - jump to byte offset.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_goto"]
pub unsafe extern "C" fn rs_ex_goto(eap: ExArgHandle) {
    let line2 = nvim_eap_get_line2(eap);
    nvim_cmd_goto_byte(line2);
}

/// Internal: implement ex_tag_cmd logic in Rust.
///
/// # Safety
/// `eap` must be valid; `name` must be a valid C string pointer.
unsafe fn rs_ex_tag_cmd(eap: ExArgHandle, name: *const c_char) {
    // Determine DT_* command from second character of command name.
    let c1 = if name.is_null() {
        0u8
    } else {
        *name.add(1) as u8
    };
    let cmd = match c1 as char {
        'j' => DT_JUMP,        // ":tjump"
        's' => DT_SELECT,      // ":tselect"
        'p' | 'N' => DT_PREV,  // ":tprevious", ":tNext"
        'n' => DT_NEXT,        // ":tnext"
        'o' => DT_POP,         // ":pop"
        'f' | 'r' => DT_FIRST, // ":tfirst", ":trewind"
        'l' => {
            // ":tlast" - but also check first char for 'l' meaning DT_LTAG
            // The DT_LTAG check uses name[0] == 'l'
            let c0 = if name.is_null() { 0u8 } else { *name as u8 };
            if c0 == b'l' {
                DT_LTAG
            } else {
                6 // DT_LAST
            }
        }
        _ => DT_TAG, // ":tag"
    };

    // Re-check: if name[0] == 'l' this overrides to DT_LTAG
    let c0 = if name.is_null() { 0u8 } else { *name as u8 };
    let final_cmd = if c0 == b'l' { DT_LTAG } else { cmd };

    let arg = nvim_eap_get_arg(eap);
    let count = if nvim_eap_get_addr_count(eap) > 0 {
        nvim_eap_get_line2(eap)
    } else {
        1
    };
    let forceit = nvim_eap_get_forceit(eap);
    rs_do_tag(arg, final_cmd, count, forceit as c_int, true);
}

/// `:tag`, `:tselect`, `:tjump`, etc.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_tag"]
pub unsafe extern "C" fn rs_ex_tag(eap: ExArgHandle) {
    let cmdidx = nvim_eap_get_cmdidx(eap);
    let name = nvim_docmd_cmdnames_name(cmdidx);
    rs_ex_tag_cmd(eap, name);
}

/// `:ptag`, `:ptselect`, `:ptjump`, etc.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_ptag"]
pub unsafe extern "C" fn rs_ex_ptag(eap: ExArgHandle) {
    g_do_tagpreview = p_pvh as c_int; // will be reset in ex_tag_cmd()
    let cmdidx = nvim_eap_get_cmdidx(eap);
    let name = nvim_docmd_cmdnames_name(cmdidx);
    // Use name + 1 (skip leading 'p')
    rs_ex_tag_cmd(eap, name.add(1));
}

/// `:stag`, `:stselect`, `:stjump`.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_stag"]
pub unsafe extern "C" fn rs_ex_stag(eap: ExArgHandle) {
    postponed_split = -1;
    postponed_split_flags = nvim_docmd_get_cmdmod_cmod_split();
    postponed_split_tab = nvim_docmd_get_cmdmod_cmod_tab();
    let cmdidx = nvim_eap_get_cmdidx(eap);
    let name = nvim_docmd_cmdnames_name(cmdidx);
    // Use name + 1 (skip leading 's')
    rs_ex_tag_cmd(eap, name.add(1));
    postponed_split_flags = 0;
    postponed_split_tab = 0;
}

/// `:pclose` - close any preview window.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_pclose"]
pub unsafe extern "C" fn rs_ex_pclose(eap: ExArgHandle) {
    let forceit = nvim_eap_get_forceit(eap);
    let mut wp = nvim_get_firstwin();
    while !wp.is_null() {
        if nvim_win_get_p_pvw(wp) != 0 {
            nvim_docmd_ex_win_close(forceit, wp);
            break;
        }
        wp = nvim_win_get_next(wp);
    }
}

/// `:pedit` - preview edit.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_pedit"]
pub unsafe extern "C" fn rs_ex_pedit(eap: ExArgHandle) {
    let curwin_save = nvim_get_curwin();
    nvim_docmd_prepare_preview_window();
    nvim_docmd_do_exedit(eap);
    nvim_docmd_back_to_current_window(curwin_save);
}

/// `:pbuffer` - preview buffer.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_pbuffer"]
pub unsafe extern "C" fn rs_ex_pbuffer(eap: ExArgHandle) {
    let curwin_save = nvim_get_curwin();
    nvim_docmd_prepare_preview_window();
    nvim_docmd_do_exbuffer(eap);
    nvim_docmd_back_to_current_window(curwin_save);
}

/// `:isearch`, `:dsearch`, `:ilist`, `:dlist`, `:ijump`, `:djump`, `:isplit`, `:dsplit`.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_findpat"]
pub unsafe extern "C" fn rs_ex_findpat(eap: ExArgHandle) {
    let cmdidx = nvim_eap_get_cmdidx(eap);
    let cmd_name = nvim_docmd_cmdnames_name(cmdidx);

    let mut whole = true;

    // Determine action from 3rd character of command name.
    let c2 = if cmd_name.is_null() {
        0u8
    } else {
        *cmd_name.add(2) as u8
    };
    let action = match c2 as char {
        'e' => {
            // ":psearch", ":isearch", ":dsearch"
            let c0 = if cmd_name.is_null() {
                0u8
            } else {
                *cmd_name as u8
            };
            if c0 == b'p' {
                ACTION_GOTO
            } else {
                ACTION_SHOW
            }
        }
        'i' => ACTION_SHOW_ALL, // ":ilist", ":dlist"
        'u' => ACTION_GOTO,     // ":ijump", ":djump"
        _ => ACTION_SPLIT,      // ":isplit", ":dsplit"
    };

    let mut eap_arg = nvim_eap_get_arg(eap);
    let forceit = nvim_eap_get_forceit(eap);

    let mut n = 1;
    if !eap_arg.is_null() && *eap_arg != 0 && {
        let c = *eap_arg as u8;
        // ascii_isdigit
        c.is_ascii_digit()
    } {
        n = getdigits_int(&mut eap_arg, false, 0);
        eap_arg = skipwhite(eap_arg);
    }

    if !eap_arg.is_null() && *eap_arg == b'/' as c_char {
        // Match regexp, not just whole words
        whole = false;
        eap_arg = eap_arg.add(1);
        let magic = nvim_cmd_magic_isset();
        let p = rs_skip_regexp(eap_arg, b'/' as c_int, magic);
        if !p.is_null() && *p != 0 {
            *p = 0; // NUL-terminate
            let p_after = p.add(1);
            let p_after = skipwhite(p_after);
            // Check for trailing illegal characters
            if !ends_excmd(*p_after as i32) {
                nvim_eap_set_errmsg_const(eap, c"E488: Trailing characters: %s".as_ptr());
            } else {
                let nextcmd = nvim_check_nextcmd(p_after);
                nvim_eap_set_nextcmd(eap, nextcmd);
            }
        }
    }

    // Only execute if not skipping
    if nvim_eap_get_skip(eap) == 0 {
        let len = strlen(eap_arg as *const c_char);
        let type_ = if !cmd_name.is_null() && *cmd_name == b'd' as c_char {
            FIND_DEFINE
        } else {
            FIND_ANY
        };
        let line1 = nvim_eap_get_line1(eap);
        let line2 = nvim_eap_get_line2(eap);
        nvim_cmd_find_pattern_in_path(
            eap_arg as *const c_char,
            0,
            len,
            whole as c_int,
            (!forceit) as c_int,
            type_,
            n,
            action,
            line1,
            line2,
            forceit as c_int,
            0,
        );
    }
}

// =============================================================================
// Phase 1: Complex command implementations (delegate to C wrappers)
// =============================================================================

/// `:tabs` - list all tabs and their windows.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_tabs"]
pub unsafe extern "C" fn rs_ex_tabs(eap: ExArgHandle) {
    nvim_docmd_ex_tabs_impl(eap);
}

/// `:syncbind` - synchronize scrollbind windows.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_syncbind"]
pub unsafe extern "C" fn rs_ex_syncbind(eap: ExArgHandle) {
    nvim_docmd_ex_syncbind_impl(eap);
}

/// `:read` - read file or filter output.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_read"]
pub unsafe extern "C" fn rs_ex_read(eap: ExArgHandle) {
    nvim_docmd_ex_read_impl(eap);
}

/// `:detach` - detach the current UI.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_detach"]
pub unsafe extern "C" fn rs_ex_detach(eap: ExArgHandle) {
    nvim_docmd_ex_detach_impl(eap);
}

/// `:connect` - connect UI to a different server.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_connect"]
pub unsafe extern "C" fn rs_ex_connect(eap: ExArgHandle) {
    nvim_docmd_ex_connect_impl(eap);
}

/// `:checkhealth` - run health checks.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_checkhealth"]
pub unsafe extern "C" fn rs_ex_checkhealth(eap: ExArgHandle) {
    nvim_docmd_ex_checkhealth_impl(eap);
}

/// `:terminal` - open terminal buffer.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_terminal"]
pub unsafe extern "C" fn rs_ex_terminal(eap: ExArgHandle) {
    nvim_docmd_ex_terminal_impl(eap);
}

/// `:restart` - restart the Nvim server.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_restart"]
pub unsafe extern "C" fn rs_ex_restart(eap: ExArgHandle) {
    nvim_docmd_ex_restart_impl(eap);
}

/// `:tabonly` - close all tab pages except the current one.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_tabonly"]
pub unsafe extern "C" fn rs_ex_tabonly(eap: ExArgHandle) {
    nvim_docmd_ex_tabonly_impl(eap);
}

// =============================================================================
// Phase 2: Public utility functions (delegate to C implementations)
// =============================================================================

extern "C" {
    // Phase 2 C implementation wrappers
    fn nvim_docmd_do_exedit_impl(eap: ExArgHandle, old_curwin: WinHandle);
    fn nvim_docmd_ex_splitview_impl(eap: ExArgHandle);
    fn nvim_docmd_ex_find_impl(eap: ExArgHandle);
    fn nvim_docmd_before_quit_autocmds_impl(wp: WinHandle, quit_all: bool, forceit: bool) -> bool;
    fn nvim_docmd_ex_win_close_impl(forceit: c_int, win: WinHandle, tp: *mut c_void);
    fn nvim_docmd_tabpage_close_impl(forceit: c_int);
    fn nvim_docmd_tabpage_close_other_impl(tp: *mut c_void, forceit: c_int);
    fn nvim_docmd_tabpage_new_impl();
    fn nvim_docmd_do_exbuffer_impl(eap: ExArgHandle);
    fn nvim_docmd_handle_did_throw_impl();
}

/// `do_exedit` - edit/badd/visual command dispatch.
///
/// # Safety
/// `eap` must be a valid ExArgHandle. `old_curwin` may be null.
#[no_mangle]
pub unsafe extern "C" fn do_exedit(eap: ExArgHandle, old_curwin: WinHandle) {
    nvim_docmd_do_exedit_impl(eap, old_curwin);
}

/// `ex_splitview` - split/vsplit/tabedit dispatch.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_splitview"]
pub unsafe extern "C" fn rs_ex_splitview(eap: ExArgHandle) {
    nvim_docmd_ex_splitview_impl(eap);
}

/// `:find` command.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_find"]
pub unsafe extern "C" fn rs_ex_find(eap: ExArgHandle) {
    nvim_docmd_ex_find_impl(eap);
}

/// `before_quit_autocmds` - fire pre-quit autocmds.
///
/// # Safety
/// `wp` must be a valid WinHandle.
#[no_mangle]
pub unsafe extern "C" fn before_quit_autocmds(
    wp: WinHandle,
    quit_all: bool,
    forceit: bool,
) -> bool {
    nvim_docmd_before_quit_autocmds_impl(wp, quit_all, forceit)
}

/// `ex_win_close` - close a window, handling modified buffers.
///
/// # Safety
/// `win` must be a valid WinHandle. `tp` may be null.
#[no_mangle]
pub unsafe extern "C" fn ex_win_close(forceit: c_int, win: WinHandle, tp: *mut c_void) {
    nvim_docmd_ex_win_close_impl(forceit, win, tp);
}

/// `tabpage_close` - close the current tab page.
///
/// # Safety
/// This function is safe to call.
#[no_mangle]
pub unsafe extern "C" fn tabpage_close(forceit: c_int) {
    nvim_docmd_tabpage_close_impl(forceit);
}

/// `tabpage_close_other` - close another tab page.
///
/// # Safety
/// `tp` must be a valid tabpage handle.
#[no_mangle]
pub unsafe extern "C" fn tabpage_close_other(tp: *mut c_void, forceit: c_int) {
    nvim_docmd_tabpage_close_other_impl(tp, forceit);
}

/// `tabpage_new` - open a new tab page.
///
/// # Safety
/// This function is safe to call.
#[no_mangle]
pub unsafe extern "C" fn tabpage_new() {
    nvim_docmd_tabpage_new_impl();
}

/// `do_exbuffer` - execute buffer command.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "do_exbuffer"]
pub unsafe extern "C" fn rs_do_exbuffer(eap: ExArgHandle) {
    nvim_docmd_do_exbuffer_impl(eap);
}

/// `handle_did_throw` - report uncaught exception.
///
/// # Safety
/// This function accesses global exception state.
#[no_mangle]
pub unsafe extern "C" fn handle_did_throw() {
    nvim_docmd_handle_did_throw_impl();
}
