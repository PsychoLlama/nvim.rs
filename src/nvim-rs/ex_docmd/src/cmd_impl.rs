//! Phase 1 command implementations migrated from ex_docmd.c.
//!
//! Contains Rust implementations of static command handlers:
//! ex_terminal, ex_checkhealth, ex_syncbind, ex_read, ex_detach, ex_connect,
//! ex_tabs, ex_restart, ex_tabonly, ex_pclose, ex_pedit, ex_pbuffer,
//! ex_stag, ex_tag, ex_ptag, ex_goto, ex_findpat.

use std::ffi::{c_char, c_int, c_void};

use crate::ExArgHandle;
use nvim_normal::types::OpargT;

// =============================================================================
// Type aliases
// =============================================================================

type LinenrT = i32;
type WinHandle = *mut c_void;
type BufHandle = *mut c_void;

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

    // Prepare / restore preview window
    fn nvim_docmd_prepare_preview_window();
    fn nvim_docmd_back_to_current_window(curwin_save: WinHandle);

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
    fn nvim_docmd_ex_syncbind_impl(eap: ExArgHandle);
    fn nvim_docmd_ex_read_impl(eap: ExArgHandle);
    fn nvim_docmd_ex_detach_impl(eap: ExArgHandle);
    fn nvim_docmd_ex_connect_impl(eap: ExArgHandle);
    fn nvim_docmd_ex_checkhealth_impl(eap: ExArgHandle);
    fn nvim_docmd_ex_terminal_impl(eap: ExArgHandle);
    fn nvim_docmd_ex_restart_impl(eap: ExArgHandle);

    // curwin accessor
    fn nvim_get_curwin() -> WinHandle;

    // cmdwin_type global (for ex_tabonly and others)
    static cmdwin_type: c_int;

    // Tabpage iteration (for ex_tabonly)
    fn nvim_get_first_tabpage() -> *mut c_void;
    fn nvim_tabpage_get_next(tp: *mut c_void) -> *mut c_void;
    fn nvim_docmd_tabpage_is_curtopframe(tp: *mut c_void) -> c_int;
    fn nvim_docmd_get_tabpage_arg(eap: ExArgHandle) -> c_int;
    fn nvim_docmd_is_only_tabpage() -> c_int;
    fn goto_tabpage(n: c_int);
    #[link_name = "rs_valid_tabpage"]
    fn rs_valid_tabpage(tp: *mut c_void) -> c_int;
    fn nvim_eap_get_errmsg(eap: ExArgHandle) -> *mut c_char;
    fn nvim_set_cmdwin_result(val: c_int);
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
            rs_ex_win_close_impl(c_int::from(forceit), wp, std::ptr::null_mut());
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
    nvim_docmd_do_exedit_impl(eap, std::ptr::null_mut());
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
    rs_do_exbuffer_impl(eap);
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

/// `nvim_docmd_ex_tabs_impl` - list all tabs and their windows.
///
/// # Safety
/// Accesses global curwin, tabpage list, window list.
#[export_name = "nvim_docmd_ex_tabs_impl"]
pub unsafe extern "C" fn rs_ex_tabs_impl(_eap: ExArgHandle) {
    // HLF_T = 23 (highlight group for tab-page header)
    const HLF_T: c_int = 23;

    nvim_msg_start();
    nvim_set_msg_scroll(1);

    let lastused_tp = nvim_get_lastused_tabpage();
    let lastused_win = if rs_valid_tabpage(lastused_tp) != 0 {
        nvim_tabpage_get_curwin(lastused_tp)
    } else {
        std::ptr::null_mut()
    };

    let curwin = nvim_get_curwin();
    let mut tabcount: c_int = 1;
    let mut tp = nvim_get_first_tabpage();
    while !tp.is_null() {
        if nvim_docmd_get_got_int() != 0 {
            break;
        }

        nvim_msg_putchar(b'\n' as c_int);
        let label = nvim_docmd_tab_page_fmt(tabcount);
        nvim_docmd_msg_outtrans_attr(label, HLF_T);
        tabcount += 1;
        nvim_docmd_os_breakcheck();

        let mut wp = nvim_tabpage_get_firstwin(tp);
        while !wp.is_null() {
            if nvim_docmd_get_got_int() != 0 {
                break;
            }
            if nvim_win_get_focusable(wp) == 0 || nvim_ex2_win_get_w_config_hide(wp) {
                wp = nvim_win_get_next(wp);
                continue;
            }

            nvim_msg_putchar(b'\n' as c_int);
            nvim_msg_putchar(if wp == curwin {
                b'>' as c_int
            } else if wp == lastused_win {
                b'#' as c_int
            } else {
                b' ' as c_int
            });
            nvim_msg_putchar(b' ' as c_int);
            let buf = nvim_win_get_buffer(wp);
            nvim_msg_putchar(if bufIsChanged(buf) != 0 {
                b'+' as c_int
            } else {
                b' ' as c_int
            });
            nvim_msg_putchar(b' ' as c_int);

            let spname = nvim_docmd_buf_spname(buf);
            if !spname.is_null() {
                let iobuff = nvim_docmd_get_iobuff();
                nvim_xstrlcpy(iobuff, spname, nvim_iosize());
            } else {
                let fname = nvim_buf_get_b_fname(buf);
                nvim_docmd_home_replace(buf, fname);
            }
            let iobuff = nvim_docmd_get_iobuff();
            nvim_docmd_msg_outtrans_attr(iobuff, 0);
            nvim_docmd_os_breakcheck();

            wp = nvim_win_get_next(wp);
        }

        tp = nvim_tabpage_get_next(tp);
    }
}

/// `:tabs` - list all tabs and their windows.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_tabs"]
pub unsafe extern "C" fn rs_ex_tabs(eap: ExArgHandle) {
    rs_ex_tabs_impl(eap);
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
    const K_IGNORE: c_int = -13821;

    if cmdwin_type != 0 {
        nvim_set_cmdwin_result(K_IGNORE);
        return;
    }

    if nvim_docmd_is_only_tabpage() != 0 {
        msg(c"Already only one tab page".as_ptr(), 0);
        return;
    }

    let tab_number = nvim_docmd_get_tabpage_arg(eap);
    if !nvim_eap_get_errmsg(eap).is_null() {
        return;
    }

    goto_tabpage(tab_number);

    // Repeat up to 1000 times: autocommands may mess up the lists.
    let forceit = nvim_eap_get_forceit(eap) as c_int;
    'outer: for _ in 0..1000 {
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            if nvim_docmd_tabpage_is_curtopframe(tp) == 0 {
                rs_tabpage_close_other_impl(tp, forceit);
                // If we failed to close it, quit
                if rs_valid_tabpage(tp) != 0 {
                    break 'outer;
                }
                // Start over: tp is now invalid
                break;
            }
            tp = nvim_tabpage_get_next(tp);
        }
        // Check if done
        if nvim_docmd_is_only_tabpage() != 0 {
            break;
        }
    }
}

// EXFLAG constants (matching ex_cmds.h)
const EXFLAG_NR: c_int = 0x02;
const EXFLAG_LIST: c_int = 0x01;

/// `nvim_docmd_ex_may_print_impl` - print current line if flags set.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "nvim_docmd_ex_may_print_impl"]
pub unsafe extern "C" fn rs_nvim_docmd_ex_may_print_impl(eap: ExArgHandle) {
    let flags = nvim_eap_get_flags(eap);
    if flags != 0 {
        rs_print_line(
            nvim_get_curwin_cursor_lnum(),
            flags & EXFLAG_NR,
            flags & EXFLAG_LIST,
            1,
        );
        nvim_set_ex_no_reprint(1);
    }
}

const FAIL: c_int = 0;
const OK: c_int = 1;

/// `nvim_docmd_update_topline_cursor_impl` - update topline, leftcol, cursor.
///
/// # Safety
/// Accesses global curwin.
#[export_name = "nvim_docmd_update_topline_cursor_impl"]
pub unsafe extern "C" fn rs_update_topline_cursor_impl() {
    nvim_docmd_check_cursor();
    nvim_docmd_update_topline();
    if nvim_docmd_curwin_p_wrap() == 0 {
        nvim_docmd_validate_cursor();
    }
    nvim_docmd_update_curswant();
}

/// `nvim_docmd_vim_mkdir_emsg_impl` - create directory, emit error on failure.
///
/// # Safety
/// `name` must be a valid null-terminated C string.
#[export_name = "nvim_docmd_vim_mkdir_emsg_impl"]
pub unsafe extern "C" fn rs_vim_mkdir_emsg_impl(name: *const c_char, prot: c_int) -> c_int {
    let ret = nvim_docmd_os_mkdir(name, prot);
    if ret != 0 {
        nvim_docmd_semsg_mkdir_err(name, ret);
        return FAIL;
    }
    OK
}

/// `nvim_docmd_open_exfile_impl` - open file for writing, with checks.
///
/// # Safety
/// `fname`, `mode` must be valid null-terminated C strings.
#[export_name = "nvim_docmd_open_exfile_impl"]
pub unsafe extern "C" fn rs_open_exfile_impl(
    fname: *mut c_char,
    forceit: c_int,
    mode: *mut c_char,
) -> *mut c_void {
    // On UNIX, check for directory.
    #[cfg(unix)]
    if nvim_docmd_os_isdir(fname) != 0 {
        nvim_docmd_semsg_isadir2(fname);
        return std::ptr::null_mut();
    }
    // Check if file exists when not appending and not forcing.
    if forceit == 0 && *mode != b'a' as c_char && nvim_docmd_os_path_exists(fname) != 0 {
        nvim_docmd_semsg_file_exists(fname);
        return std::ptr::null_mut();
    }
    let fd = nvim_docmd_os_fopen(fname, mode as *const c_char);
    if fd.is_null() {
        nvim_docmd_semsg_cant_open_write(fname);
    }
    fd
}

// =============================================================================
// Phase 2: Public utility functions (delegate to C implementations)
// =============================================================================

extern "C" {
    // Phase 2 C implementation wrappers
    fn nvim_docmd_do_exedit_impl(eap: ExArgHandle, old_curwin: WinHandle);
    fn nvim_docmd_ex_splitview_impl(eap: ExArgHandle);
    fn nvim_docmd_tabpage_new_impl();

    // ex_win_close_impl helpers
    #[link_name = "is_aucmd_win"]
    fn nvim_is_aucmd_win(wp: WinHandle) -> c_int;
    fn nvim_emsg_id(id: c_int);
    fn bufIsChanged(buf: BufHandle) -> c_int;
    fn buf_hide(buf: BufHandle) -> c_int;
    fn nvim_get_p_confirm() -> c_int;
    fn nvim_get_cmdmod_confirm() -> c_int;
    fn nvim_get_p_write() -> c_int;
    fn nvim_docmd_dialog_changed_still_dirty(buf: BufHandle) -> bool;
    fn nvim_docmd_no_write_message();
    #[link_name = "win_close"]
    fn nvim_docmd_win_close(win: WinHandle, free_buf: bool, force: bool) -> c_int;
    #[link_name = "rs_win_close_othertab"]
    fn nvim_docmd_win_close_othertab(
        wp: WinHandle,
        free_buf: c_int,
        tp: *mut c_void,
        force: c_int,
    ) -> c_int;

    // before_quit_autocmds helpers
    fn rs_win_valid(wp: WinHandle) -> bool;
    fn nvim_curbuf_locked() -> c_int;
    fn nvim_buf_get_nwindows(buf: BufHandle) -> c_int;
    fn nvim_buf_get_locked(buf: BufHandle) -> c_int;
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_get_curbuf() -> BufHandle;
    fn rs_only_one_window() -> c_int;
    fn nvim_docmd_check_more(message: c_int, forceit: c_int) -> c_int;
    fn nvim_docmd_apply_autocmds_quitpre(buf: BufHandle);
    fn nvim_docmd_apply_autocmds_exitpre();

    // ex_tabs helpers
    fn nvim_msg_start();
    fn nvim_docmd_tab_page_fmt(n: c_int) -> *mut c_char;
    fn nvim_docmd_msg_outtrans_attr(s: *const c_char, attr: c_int);
    #[link_name = "buf_spname"]
    fn nvim_docmd_buf_spname(buf: BufHandle) -> *mut c_char;
    fn nvim_docmd_home_replace(buf: BufHandle, src: *const c_char);
    fn nvim_docmd_get_iobuff() -> *mut c_char;
    fn nvim_win_get_focusable(wp: WinHandle) -> c_int;
    fn nvim_ex2_win_get_w_config_hide(wp: WinHandle) -> bool;
    fn nvim_tabpage_get_curwin(tp: *mut c_void) -> WinHandle;
    fn nvim_get_lastused_tabpage() -> *mut c_void;
    fn nvim_tabpage_get_firstwin(tp: *mut c_void) -> WinHandle;
    fn nvim_buf_get_b_fname(buf: BufHandle) -> *const c_char;
    fn nvim_msg_putchar(c: c_int);
    fn nvim_docmd_get_got_int() -> c_int;
    fn nvim_docmd_os_breakcheck();
    fn nvim_iosize() -> usize;
    fn nvim_xstrlcpy(dst: *mut c_char, src: *const c_char, n: usize);

    // do_exbuffer helpers
    fn nvim_docmd_goto_buffer_current(eap: ExArgHandle);
    fn nvim_docmd_goto_buffer_first(eap: ExArgHandle, n: c_int);
    fn nvim_docmd_eap_get_do_ecmd_cmd(eap: ExArgHandle) -> *mut c_char;
    fn nvim_docmd_errmsg_trailing_arg(arg: *const c_char) -> *mut c_char;
    fn nvim_eap_set_errmsg(eap: ExArgHandle, msg: *mut c_char);

    // handle_did_throw helpers
    fn nvim_exception_get_type() -> c_int;
    fn nvim_exception_get_value() -> *mut c_char;
    fn nvim_exception_take_messages() -> *mut c_void;
    fn nvim_exception_get_throw_name() -> *mut c_char;
    fn nvim_exception_get_throw_lnum() -> c_int;
    fn nvim_exception_set_throw_name_null();
    fn nvim_msglist_get_next(m: *mut c_void) -> *mut c_void;
    fn nvim_msglist_get_msg(m: *mut c_void) -> *mut c_char;
    fn nvim_msglist_is_multiline(m: *mut c_void) -> c_int;
    fn nvim_msglist_free_item(m: *mut c_void);
    fn nvim_get_emsg_silent() -> c_int;
    fn nvim_set_suppress_errthrow(val: bool);
    fn nvim_set_force_abort(val: bool);
    fn nvim_docmd_fmt_exception_not_caught(value: *const c_char) -> *mut c_char;
    fn nvim_docmd_free_sourcing_name_and_pop();
    fn nvim_discard_current_exception();
    fn xfree(p: *mut c_void);
    // emsg_multiline(s, kind, hl_id, multiline) -> int
    fn emsg_multiline(
        s: *const c_char,
        kind: *const c_char,
        hl_id: c_int,
        multiline: c_int,
    ) -> c_int;
    // estack_push
    #[link_name = "estack_push"]
    fn nvim_estack_push(etype: c_int, name: *mut c_char, lnum: c_int) -> *mut c_void;

    // tabpage_close helpers (for Rust implementations)
    fn nvim_docmd_curwin_is_floating() -> c_int;
    fn nvim_docmd_is_one_window() -> c_int;
    fn nvim_docmd_ex_win_close_curwin(forceit: c_int);
    fn nvim_docmd_close_others(message: bool, forceit: bool);
    fn nvim_docmd_tabpage_get_lastwin(tp: *mut c_void) -> WinHandle;
    fn nvim_docmd_tabpage_lastwin_eq(tp: *mut c_void, wp: WinHandle) -> c_int;
    fn nvim_docmd_ex_win_close_in_tab(forceit: c_int, wp: WinHandle, tp: *mut c_void);

    // ex_find helpers
    fn nvim_eap_set_arg(eap: ExArgHandle, arg: *mut c_char);
    fn nvim_docmd_get_findfunc_nonempty() -> bool;
    fn nvim_docmd_findfunc_find_file(arg: *mut c_char, len: usize, count: c_int) -> *mut c_char;
    fn nvim_docmd_curbuf_b_ffname() -> *const c_char;
    #[link_name = "find_file_in_path"]
    fn nvim_docmd_find_file_in_path(
        ptr: *mut c_char,
        len: usize,
        options: c_int,
        first: c_int,
        rel_fname: *const c_char,
        file_to_find: *mut *mut c_char,
        search_ctx: *mut *mut c_void,
    ) -> *mut c_char;
    #[link_name = "nvim_vim_findfile_cleanup"]
    fn nvim_docmd_vim_findfile_cleanup(search_ctx: *mut c_void);
    #[link_name = "check_can_set_curbuf_forceit"]
    fn nvim_docmd_check_can_set_curbuf_forceit(forceit: c_int) -> bool;

    // exec_normal helpers
    #[link_name = "ins_typebuf"]
    fn nvim_docmd_ins_typebuf(
        str: *const c_char,
        noremap: c_int,
        offset: c_int,
        nottyped: bool,
        silent: bool,
    ) -> c_int;
    fn nvim_set_finish_op(val: bool);
    #[link_name = "stuff_empty"]
    fn nvim_docmd_stuff_empty() -> c_int;
    #[link_name = "typebuf_typed"]
    fn nvim_docmd_typebuf_typed() -> c_int;
    fn nvim_docmd_typebuf_tb_len() -> c_int;
    #[link_name = "vpeekc"]
    fn nvim_docmd_vpeekc() -> c_int;
    fn normal_cmd(oap: *mut OpargT, toplevel: bool);
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

/// `:find` - find file in path and edit it.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "nvim_docmd_ex_find_impl"]
pub unsafe extern "C" fn rs_ex_find_impl(eap: ExArgHandle) {
    let forceit = nvim_eap_get_forceit(eap) as c_int;
    if !nvim_docmd_check_can_set_curbuf_forceit(forceit) {
        return;
    }

    let arg = nvim_eap_get_arg(eap);
    let len = std::ffi::CStr::from_ptr(arg).to_bytes().len();

    let fname: *mut c_char = if nvim_docmd_get_findfunc_nonempty() {
        let count = if nvim_eap_get_addr_count(eap) > 0 {
            nvim_eap_get_line2(eap) as c_int
        } else {
            1
        };
        nvim_docmd_findfunc_find_file(arg, len, count)
    } else {
        let mut file_to_find: *mut c_char = std::ptr::null_mut();
        let mut search_ctx: *mut c_void = std::ptr::null_mut();
        const FNAME_MESS: c_int = 1;
        let rel = nvim_docmd_curbuf_b_ffname();
        let mut fname = nvim_docmd_find_file_in_path(
            arg,
            len,
            FNAME_MESS,
            1,
            rel,
            &mut file_to_find,
            &mut search_ctx,
        );
        if nvim_eap_get_addr_count(eap) > 0 {
            let mut count = nvim_eap_get_line2(eap) as c_int;
            while !fname.is_null() && {
                count -= 1;
                count > 0
            } {
                xfree(fname as *mut c_void);
                fname = nvim_docmd_find_file_in_path(
                    std::ptr::null_mut(),
                    0,
                    FNAME_MESS,
                    0,
                    rel,
                    &mut file_to_find,
                    &mut search_ctx,
                );
            }
        }
        xfree(file_to_find as *mut c_void);
        nvim_docmd_vim_findfile_cleanup(search_ctx);
        fname
    };

    if fname.is_null() {
        return;
    }

    nvim_eap_set_arg(eap, fname);
    nvim_docmd_do_exedit_impl(eap, std::ptr::null_mut());
    xfree(fname as *mut c_void);
}

/// `:find` command - public entry point.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_find"]
pub unsafe extern "C" fn rs_ex_find(eap: ExArgHandle) {
    rs_ex_find_impl(eap);
}

/// Execute normal mode commands from typeahead until buffer is empty.
///
/// # Safety
/// Must be called with valid global state.
#[export_name = "nvim_docmd_exec_normal_impl"]
pub unsafe extern "C" fn rs_exec_normal_impl(was_typed: bool, use_vpeekc: bool) {
    let mut oa = OpargT::default();
    nvim_set_finish_op(false);
    // Ctrl_C = 3
    const CTRL_C: c_int = 3;
    loop {
        // C: (!stuff_empty() || typebuf_has_data || vpeekc_avail) && !got_int
        // stuff_empty() returns 1=empty, 0=not-empty
        let cont = if nvim_docmd_stuff_empty() != 0 {
            // stuff buffer IS empty; check typeahead and vpeekc
            let typed_or_not = was_typed || nvim_docmd_typebuf_typed() == 0;
            let has_typebuf = typed_or_not && nvim_docmd_typebuf_tb_len() > 0;
            if has_typebuf {
                true
            } else if use_vpeekc {
                let c = nvim_docmd_vpeekc();
                c != 0 && c != CTRL_C
            } else {
                false
            }
        } else {
            // stuff buffer NOT empty → continue
            true
        };
        if !cont || nvim_docmd_get_got_int() != 0 {
            break;
        }
        rs_update_topline_cursor_impl();
        normal_cmd(&raw mut oa, true);
    }
}

/// Execute normal mode command "cmd".
/// "remap" can be REMAP_NONE (= -1) or REMAP_YES (= 0).
///
/// # Safety
/// `cmd` must be a valid null-terminated string.
#[export_name = "nvim_docmd_exec_normal_cmd_impl"]
pub unsafe extern "C" fn rs_exec_normal_cmd_impl(cmd: *mut c_char, remap: c_int, silent: bool) {
    nvim_docmd_ins_typebuf(cmd, remap, 0, true, silent);
    rs_exec_normal_impl(false, false);
}

/// `nvim_docmd_before_quit_autocmds_impl` - fire pre-quit autocmds.
///
/// Fires QUITPRE, checks window validity and buffer locks, and optionally
/// fires EXITPRE if this is the last window. Returns true if the quit should
/// be cancelled.
///
/// # Safety
/// `wp` must be a valid WinHandle.
#[export_name = "nvim_docmd_before_quit_autocmds_impl"]
pub unsafe extern "C" fn rs_before_quit_autocmds_impl(
    wp: WinHandle,
    quit_all: bool,
    forceit: bool,
) -> bool {
    let buf = nvim_win_get_buffer(wp);
    nvim_docmd_apply_autocmds_quitpre(buf);

    // Bail out when autocommands closed the window, or the buffer is locked.
    if !rs_win_valid(wp)
        || nvim_curbuf_locked() != 0
        || (nvim_buf_get_nwindows(buf) == 1 && nvim_buf_get_locked(buf) > 0)
    {
        return true;
    }

    let ok: c_int = 1; // OK
    if quit_all || (nvim_docmd_check_more(0, forceit as c_int) == ok && rs_only_one_window() != 0) {
        nvim_docmd_apply_autocmds_exitpre();
        // Refuse to quit when locked or the window was closed.
        let curbuf = nvim_get_curbuf();
        if !rs_win_valid(wp)
            || nvim_curbuf_locked() != 0
            || (nvim_buf_get_nwindows(curbuf) == 1 && nvim_buf_get_locked(curbuf) > 0)
        {
            return true;
        }
    }

    false
}

/// `before_quit_autocmds` - public C-callable name.
///
/// # Safety
/// `wp` must be a valid WinHandle.
#[no_mangle]
pub unsafe extern "C" fn before_quit_autocmds(
    wp: WinHandle,
    quit_all: bool,
    forceit: bool,
) -> bool {
    rs_before_quit_autocmds_impl(wp, quit_all, forceit)
}

/// `nvim_docmd_ex_win_close_impl` - close a window, handling modified buffers.
///
/// If `tp` is non-null, close the window in that tab page.
///
/// # Safety
/// `win` must be a valid WinHandle. `tp` may be null.
#[export_name = "nvim_docmd_ex_win_close_impl"]
pub unsafe extern "C" fn rs_ex_win_close_impl(forceit: c_int, win: WinHandle, tp: *mut c_void) {
    const EMSG_E_AUTOCMD_CLOSE: c_int = 10;

    // Never close the autocommand window.
    if nvim_is_aucmd_win(win) != 0 {
        nvim_emsg_id(EMSG_E_AUTOCMD_CLOSE);
        return;
    }

    let buf = nvim_win_get_buffer(win);
    let need_hide = bufIsChanged(buf) != 0 && nvim_buf_get_nwindows(buf) <= 1;
    let mut need_hide = need_hide;

    if need_hide && buf_hide(buf) == 0 && forceit == 0 {
        if (nvim_get_p_confirm() != 0 || nvim_get_cmdmod_confirm() != 0) && nvim_get_p_write() != 0
        {
            // Show dialog; if buffer still changed after, cancel close.
            if nvim_docmd_dialog_changed_still_dirty(buf) {
                return;
            }
            need_hide = false;
        } else {
            nvim_docmd_no_write_message();
            return;
        }
    }

    // Free buffer when not hiding it or when it's a scratch buffer.
    let free_buf = !need_hide && buf_hide(buf) == 0;
    if tp.is_null() {
        nvim_docmd_win_close(win, free_buf, forceit != 0);
    } else {
        nvim_docmd_win_close_othertab(win, c_int::from(free_buf), tp, forceit);
    }
}

/// `ex_win_close` - public C-callable wrapper.
///
/// # Safety
/// `win` must be a valid WinHandle. `tp` may be null.
#[no_mangle]
pub unsafe extern "C" fn ex_win_close(forceit: c_int, win: WinHandle, tp: *mut c_void) {
    rs_ex_win_close_impl(forceit, win, tp);
}

/// `nvim_docmd_tabpage_close_impl` - close the current tab page.
///
/// Closes floating windows, then all non-current windows, then the last window.
///
/// # Safety
/// Accesses C globals (curwin). Must only be called from C context.
#[export_name = "nvim_docmd_tabpage_close_impl"]
pub unsafe extern "C" fn rs_tabpage_close_impl(forceit: c_int) {
    // First close all floating windows in this tab.
    while nvim_docmd_curwin_is_floating() != 0 {
        nvim_docmd_ex_win_close_curwin(forceit);
    }
    // Close all other non-floating windows.
    if nvim_docmd_is_one_window() == 0 {
        nvim_docmd_close_others(true, forceit != 0);
    }
    // If only one window left, close it too (which closes the tab).
    if nvim_docmd_is_one_window() != 0 {
        nvim_docmd_ex_win_close_curwin(forceit);
    }
}

/// `tabpage_close` - C ABI entry point for closing current tab page.
///
/// # Safety
/// Accesses C globals (curwin). Must only be called from C context.
#[no_mangle]
pub unsafe extern "C" fn tabpage_close(forceit: c_int) {
    rs_tabpage_close_impl(forceit);
}

/// `nvim_docmd_tabpage_close_other_impl` - close another tab page.
///
/// Iterates windows in `tp` (up to 1000) closing each one.
///
/// # Safety
/// `tp` must be a valid tabpage handle. Must only be called from C context.
#[export_name = "nvim_docmd_tabpage_close_other_impl"]
pub unsafe extern "C" fn rs_tabpage_close_other_impl(tp: *mut c_void, forceit: c_int) {
    let mut done = 0;
    while done < 1000 {
        done += 1;
        let wp = nvim_docmd_tabpage_get_lastwin(tp);
        nvim_docmd_ex_win_close_in_tab(forceit, wp, tp);
        // Stop if the tabpage is gone or the last window didn't change.
        if rs_valid_tabpage(tp) == 0 || nvim_docmd_tabpage_lastwin_eq(tp, wp) != 0 {
            break;
        }
    }
}

/// `tabpage_close_other` - C ABI entry point for closing another tab page.
///
/// # Safety
/// `tp` must be a valid tabpage handle. Must only be called from C context.
#[no_mangle]
pub unsafe extern "C" fn tabpage_close_other(tp: *mut c_void, forceit: c_int) {
    rs_tabpage_close_other_impl(tp, forceit);
}

/// `tabpage_new` - open a new tab page.
///
/// # Safety
/// This function is safe to call.
#[no_mangle]
pub unsafe extern "C" fn tabpage_new() {
    nvim_docmd_tabpage_new_impl();
}

/// `nvim_docmd_do_exbuffer_impl` - execute `:buffer` and related commands.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "nvim_docmd_do_exbuffer_impl"]
pub unsafe extern "C" fn rs_do_exbuffer_impl(eap: ExArgHandle) {
    let arg = nvim_eap_get_arg(eap);
    if !arg.is_null() && *arg != 0 {
        let errmsg = nvim_docmd_errmsg_trailing_arg(arg);
        nvim_eap_set_errmsg(eap, errmsg);
    } else {
        if nvim_eap_get_addr_count(eap) == 0 {
            nvim_docmd_goto_buffer_current(eap);
        } else {
            nvim_docmd_goto_buffer_first(eap, nvim_eap_get_line2(eap));
        }
        let do_ecmd_cmd = nvim_docmd_eap_get_do_ecmd_cmd(eap);
        if !do_ecmd_cmd.is_null() {
            do_cmdline_cmd(do_ecmd_cmd);
        }
    }
}

/// `do_exbuffer` - public C entry point.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "do_exbuffer"]
pub unsafe extern "C" fn rs_do_exbuffer(eap: ExArgHandle) {
    rs_do_exbuffer_impl(eap);
}

/// `nvim_docmd_handle_did_throw_impl` - report an uncaught exception.
///
/// If the exception is a user exception (ET_USER), formats E605 and calls
/// `emsg`. For error exceptions (ET_ERROR), replays the saved message list
/// via `emsg_multiline`. Interrupt exceptions are silently discarded.
///
/// # Safety
/// Accesses global `current_exception`. Must only be called when
/// `current_exception != NULL`.
#[export_name = "nvim_docmd_handle_did_throw_impl"]
pub unsafe extern "C" fn rs_handle_did_throw_impl() {
    // Exception type constants matching except_type_T
    const ET_USER: c_int = 0;
    const ET_ERROR: c_int = 1;
    // ET_INTERRUPT = 2 (fall-through: no message)
    const HLF_E: c_int = 6; // ErrorMsg highlight group

    let etype = nvim_exception_get_type();
    let p: *mut c_char;
    let messages: *mut c_void;

    match etype {
        ET_USER => {
            p = nvim_docmd_fmt_exception_not_caught(nvim_exception_get_value());
            messages = std::ptr::null_mut();
        }
        ET_ERROR => {
            p = std::ptr::null_mut();
            messages = nvim_exception_take_messages();
        }
        _ => {
            // ET_INTERRUPT: no message
            p = std::ptr::null_mut();
            messages = std::ptr::null_mut();
        }
    }

    // Push throw location onto execution stack.
    let throw_name = nvim_exception_get_throw_name();
    let throw_lnum = nvim_exception_get_throw_lnum();
    nvim_estack_push(5 /* ETYPE_EXCEPT */, throw_name, throw_lnum);
    nvim_exception_set_throw_name_null();

    nvim_discard_current_exception(); // uses IObuff if 'verbose'

    // If "silent!" is not active, mark as fatal.
    if nvim_get_emsg_silent() == 0 {
        nvim_set_suppress_errthrow(true);
        nvim_set_force_abort(true);
    }

    // Display the error message(s).
    if !messages.is_null() {
        let mut cur = messages;
        while !cur.is_null() {
            let next = nvim_msglist_get_next(cur);
            emsg_multiline(
                nvim_msglist_get_msg(cur),
                c"emsg".as_ptr(),
                HLF_E,
                nvim_msglist_is_multiline(cur),
            );
            nvim_msglist_free_item(cur);
            cur = next;
        }
    } else if !p.is_null() {
        emsg(p);
        xfree(p as *mut c_void);
    }

    nvim_docmd_free_sourcing_name_and_pop();
}

/// `handle_did_throw` - public C entry point.
///
/// # Safety
/// This function accesses global exception state.
#[no_mangle]
pub unsafe extern "C" fn handle_did_throw() {
    rs_handle_did_throw_impl();
}

// =============================================================================
// Phase 6: Loop infrastructure and small functions
// =============================================================================

extern "C" {
    // do_cmdline wrapper (for do_cmdline_cmd implementation)
    #[link_name = "do_cmdline"]
    fn nvim_p6_do_cmdline(
        cmd: *mut c_char,
        fgetline: *mut c_void,
        cookie: *mut c_void,
        flags: c_int,
    ) -> c_int;

    // getexline wrapper with no flags (do_exmode calls do_cmdline(NULL, getexline, NULL, 0))
    fn nvim_docmd_do_cmdline_getexline_noflags();

    // curbuf changedtick as i64
    fn nvim_docmd_curbuf_changedtick() -> i64;

    // msg_scroll_flush wrapper
    fn nvim_docmd_msg_scroll_flush();

    // State / mode
    fn nvim_set_state(state: c_int);
    static mut exmode_active: bool;
    static mut need_wait_return: bool;

    // may_trigger_modechanged
    fn may_trigger_modechanged();

    // global_busy
    fn nvim_get_global_busy() -> bool;

    // msg_scroll get/set
    fn nvim_get_msg_scroll() -> c_int;
    fn nvim_set_msg_scroll(val: c_int);

    // RedrawingDisabled get/set
    fn nvim_get_RedrawingDisabled() -> c_int;
    fn nvim_set_redrawing_disabled(val: c_int);

    // no_wait_return
    fn nvim_ex2_get_no_wait_return() -> c_int;
    fn nvim_ex2_set_no_wait_return(val: c_int);

    // msg
    fn msg(s: *const c_char, hl_id: c_int);

    // ex_normal_busy
    #[link_name = "ex_normal_busy"]
    static mut nvim_ex_normal_busy: c_int;

    // typebuf.tb_len
    fn nvim_get_typebuf_len() -> c_int;

    // ex_pressedreturn get/set
    fn nvim_get_ex_pressedreturn() -> c_int;
    fn nvim_docmd_set_pressedreturn(val: bool);

    // ex_no_reprint set/get
    fn nvim_set_ex_no_reprint(val: c_int);
    fn nvim_get_ex_no_reprint() -> c_int;

    // msg_row get/set
    fn nvim_get_msg_row() -> c_int;
    fn nvim_ex2_set_msg_row(val: c_int);

    // curwin->w_cursor.lnum
    fn nvim_get_curwin_cursor_lnum() -> c_int;

    // cmdline_row set
    fn nvim_set_cmdline_row(val: c_int);

    // lines_left / Rows
    fn nvim_set_lines_left(val: c_int);
    fn nvim_ses_get_Rows() -> c_int;

    // curbuf->b_ml.ml_flags & ML_EMPTY
    fn nvim_al_curbuf_ml_empty() -> c_int;

    // emsg
    fn emsg(s: *const c_char);

    // e_empty_buffer string (translated)
    fn nvim_get_e_empty_buffer() -> *const c_char;

    // msg_col set
    fn nvim_ex2_set_msg_col(val: c_int);

    // rs_print_line_no_prefix (already no_mangle in Rust)
    #[link_name = "rs_print_line_no_prefix"]
    fn nvim_p6_print_line_no_prefix(lnum: i32, print_marks: bool, list: bool);

    // msg_clr_eos
    fn msg_clr_eos();

    // need_wait_return

    // redraw_all_later / update_screen
    fn nvim_redraw_all_later(redraw_type: c_int);
    fn update_screen();

    // filetype accessor functions
    fn nvim_docmd_get_filetype_plugin() -> c_int;
    fn nvim_docmd_set_filetype_plugin(val: c_int);
    fn nvim_docmd_get_filetype_indent() -> c_int;
    fn nvim_docmd_set_filetype_indent(val: c_int);
    fn nvim_docmd_get_filetype_detect() -> c_int;
    fn nvim_docmd_set_filetype_detect(val: c_int);

    // source_runtime (Rust function, already exported as "source_runtime")
    fn source_runtime(name: *const c_char, flags: c_int) -> c_int;

    // rs_print_line (for ex_may_print)
    fn rs_print_line(lnum: c_int, use_number: c_int, list: c_int, first: c_int);
    // nvim_eap_get_flags (for ex_may_print)
    fn nvim_eap_get_flags(eap: ExArgHandle) -> c_int;

    // Helpers for vim_mkdir_emsg and open_exfile (migrated to Rust)
    fn nvim_docmd_semsg_mkdir_err(name: *const c_char, errcode: c_int);
    fn nvim_docmd_semsg_file_exists(fname: *const c_char);
    fn nvim_docmd_semsg_cant_open_write(fname: *const c_char);
    fn nvim_docmd_semsg_isadir2(fname: *const c_char);
    fn nvim_docmd_os_mkdir(name: *const c_char, prot: c_int) -> c_int;
    fn nvim_docmd_os_isdir(fname: *const c_char) -> c_int;
    fn nvim_docmd_os_path_exists(fname: *const c_char) -> c_int;
    fn nvim_docmd_os_fopen(fname: *const c_char, mode: *const c_char) -> *mut c_void;

    // Accessors for update_topline_cursor (migrated to Rust)
    fn nvim_docmd_curwin_p_wrap() -> c_int;
    fn nvim_docmd_check_cursor();
    fn nvim_docmd_update_topline();
    fn nvim_docmd_validate_cursor();
    fn nvim_docmd_update_curswant();

    // filetype constants
    fn nvim_docmd_get_ftplugin_file() -> *const c_char;
    fn nvim_docmd_get_indent_file() -> *const c_char;
    fn nvim_docmd_get_filetype_file() -> *const c_char;
    fn nvim_docmd_get_dip_all() -> c_int;
}

/// DOCMD flags for do_cmdline_cmd.
const DOCMD_VERBOSE: c_int = 0x01;
const DOCMD_NOWAIT: c_int = 0x02;
const DOCMD_KEYTYPED: c_int = 0x40;

// TriState values for filetype flags.
const K_NONE: c_int = 0;
const K_TRUE: c_int = 1;

// MODE_NORMAL constant.
const MODE_NORMAL: c_int = 0x01;

// UPD_NOT_VALID constant.
const UPD_NOT_VALID: c_int = 40;

/// `do_cmdline_cmd` - execute a simple Ex command string.
///
/// Used for translated commands like "*".
///
/// # Safety
/// `cmd` must be a valid C string pointer or NULL.
#[no_mangle]
pub unsafe extern "C" fn do_cmdline_cmd(cmd: *const c_char) -> c_int {
    nvim_p6_do_cmdline(
        cmd as *mut c_char,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        DOCMD_VERBOSE | DOCMD_NOWAIT | DOCMD_KEYTYPED,
    )
}

/// `filetype_plugin_enable` - enable ftplugin and indent autocommands.
///
/// Called from main.c during startup.
///
/// # Safety
/// Calls C functions that source runtime files.
#[no_mangle]
pub unsafe extern "C" fn filetype_plugin_enable() {
    if nvim_docmd_get_filetype_plugin() == K_NONE {
        source_runtime(nvim_docmd_get_ftplugin_file(), nvim_docmd_get_dip_all());
        nvim_docmd_set_filetype_plugin(K_TRUE);
    }
    if nvim_docmd_get_filetype_indent() == K_NONE {
        source_runtime(nvim_docmd_get_indent_file(), nvim_docmd_get_dip_all());
        nvim_docmd_set_filetype_indent(K_TRUE);
    }
}

/// `filetype_maybe_enable` - enable filetype detection if not disabled.
///
/// Called from main.c during startup.
///
/// # Safety
/// Calls C functions that source runtime files.
#[no_mangle]
pub unsafe extern "C" fn filetype_maybe_enable() {
    if nvim_docmd_get_filetype_detect() == K_NONE {
        // Normally .vim files are sourced before .lua files when both are
        // supported, but we reverse the order here because we want the Lua
        // autocommand to be defined first so that it runs first
        source_runtime(nvim_docmd_get_filetype_file(), nvim_docmd_get_dip_all());
        nvim_docmd_set_filetype_detect(K_TRUE);
    }
}

/// `do_exmode` - repeatedly get commands for Ex mode until `:vi`.
///
/// # Safety
/// Accesses many C globals. Must only be called from C context.
#[no_mangle]
pub unsafe extern "C" fn do_exmode() {
    exmode_active = true;
    nvim_set_state(MODE_NORMAL);
    may_trigger_modechanged();

    // When using ":global /pat/ visual" and then "Q" we return to continue
    // the :global command.
    if nvim_get_global_busy() {
        return;
    }

    let save_msg_scroll = nvim_get_msg_scroll();
    nvim_set_redrawing_disabled(nvim_get_RedrawingDisabled() + 1); // don't redisplay the window
    nvim_ex2_set_no_wait_return(nvim_ex2_get_no_wait_return() + 1); // don't wait for return

    msg(
        c"Entering Ex mode.  Type \"visual\" to go to Normal mode.".as_ptr(),
        0,
    );
    while exmode_active {
        // Check for a ":normal" command and no more characters left.
        if nvim_ex_normal_busy > 0 && nvim_get_typebuf_len() == 0 {
            exmode_active = false;
            break;
        }
        nvim_set_msg_scroll(1);
        need_wait_return = false;
        nvim_docmd_set_pressedreturn(false);
        nvim_set_ex_no_reprint(0);
        let changedtick = nvim_docmd_curbuf_changedtick();
        let prev_msg_row = nvim_get_msg_row();
        let prev_line = nvim_get_curwin_cursor_lnum();
        nvim_set_cmdline_row(nvim_get_msg_row());
        nvim_docmd_do_cmdline_getexline_noflags();
        nvim_set_lines_left(nvim_ses_get_Rows() - 1);

        if (prev_line != nvim_get_curwin_cursor_lnum()
            || changedtick != nvim_docmd_curbuf_changedtick())
            && nvim_get_ex_no_reprint() == 0
        {
            if nvim_al_curbuf_ml_empty() != 0 {
                emsg(nvim_get_e_empty_buffer());
            } else {
                if nvim_get_ex_pressedreturn() != 0 {
                    // Make sure the message overwrites the right line and isn't throttled.
                    nvim_docmd_msg_scroll_flush();
                    // go up one line, to overwrite the ":<CR>" line, so the
                    // output doesn't contain empty lines.
                    let mut mr = prev_msg_row;
                    if prev_msg_row == nvim_ses_get_Rows() - 1 {
                        mr -= 1;
                    }
                    nvim_ex2_set_msg_row(mr);
                }
                nvim_ex2_set_msg_col(0);
                nvim_p6_print_line_no_prefix(nvim_get_curwin_cursor_lnum(), false, false);
                msg_clr_eos();
            }
        } else if nvim_get_ex_pressedreturn() != 0 && nvim_get_ex_no_reprint() == 0 {
            // must be at EOF
            if nvim_al_curbuf_ml_empty() != 0 {
                emsg(nvim_get_e_empty_buffer());
            } else {
                emsg(c"E501: At end-of-file".as_ptr());
            }
        }
    }

    nvim_set_redrawing_disabled(nvim_get_RedrawingDisabled() - 1);
    nvim_ex2_set_no_wait_return(nvim_ex2_get_no_wait_return() - 1);
    nvim_redraw_all_later(UPD_NOT_VALID);
    update_screen();
    need_wait_return = false;
    nvim_set_msg_scroll(save_msg_scroll);
}

// =============================================================================
// Phase N: post_chdir_impl helpers
// =============================================================================

extern "C" {
    fn nvim_docmd_curwin_clear_localdir();
    fn nvim_docmd_curtab_clear_localdir();
    fn nvim_docmd_get_globaldir() -> *const c_char;
    fn nvim_docmd_set_globaldir_strdup(pdir: *const c_char);
    fn nvim_docmd_clear_globaldir();
    fn nvim_docmd_os_dirname_cwd(buf: *mut c_char, len: usize) -> c_int;
    fn nvim_docmd_curtab_set_localdir(cwd: *const c_char);
    fn nvim_docmd_curwin_set_localdir(cwd: *const c_char);
    fn nvim_docmd_set_last_chdir_reason_null();
    fn nvim_docmd_shorten_fnames_nosymlinks();
    fn nvim_docmd_do_autocmd_dirchanged_manual_post(cwd: *const c_char, scope: c_int);
}

/// CdScope enum values (from vim_defs.h).
const CD_SCOPE_INVALID: c_int = -1;
const CD_SCOPE_WINDOW: c_int = 0;
const CD_SCOPE_TABPAGE: c_int = 1;
const CD_SCOPE_GLOBAL: c_int = 2;

/// MAXPATHL - maximum path length (PATH_MAX on Linux = 4096).
const MAXPATHL: usize = 4096;

/// OS return code OK = 1.
const OS_OK: c_int = 1;

/// `nvim_docmd_post_chdir_impl` - update directory state after a chdir.
///
/// Mirrors the C `nvim_docmd_post_chdir_impl` function. Clears local directory
/// fields, sets globaldir if needed, stores the new cwd, and fires DirChanged.
///
/// # Safety
/// Accesses C globals (curwin, curtab, globaldir, last_chdir_reason).
#[export_name = "nvim_docmd_post_chdir_impl"]
pub unsafe extern "C" fn rs_post_chdir_impl(scope: c_int, trigger_dirchanged: bool) {
    // Always overwrite the window-local CWD.
    nvim_docmd_curwin_clear_localdir();

    // Overwrite the tab-local CWD for :cd, :tcd.
    if scope >= CD_SCOPE_TABPAGE {
        nvim_docmd_curtab_clear_localdir();
    }

    if scope < CD_SCOPE_GLOBAL {
        // nvim_get_prevdir is already declared in commands.rs; use our own call via the C shim
        // (prevdir is handled in changedir_func before calling us, and we use globaldir here).
        // If still in global directory, set CWD as the global directory.
        // We need prevdir here -- call the C accessor from commands.rs.
        // Since we're in cmd_impl.rs, declare a local extern for it.
        extern "C" {
            fn nvim_get_prevdir(scope: c_int) -> *mut c_char;
        }
        let pdir = nvim_get_prevdir(scope);
        let globaldir = nvim_docmd_get_globaldir();
        if globaldir.is_null() && !pdir.is_null() {
            nvim_docmd_set_globaldir_strdup(pdir as *const c_char);
        }
    }

    let mut cwd = [0u8; MAXPATHL];
    let cwd_ptr = cwd.as_mut_ptr() as *mut c_char;
    if nvim_docmd_os_dirname_cwd(cwd_ptr, MAXPATHL) != OS_OK {
        return;
    }

    match scope {
        CD_SCOPE_GLOBAL => {
            nvim_docmd_clear_globaldir();
        }
        CD_SCOPE_TABPAGE => {
            nvim_docmd_curtab_set_localdir(cwd_ptr);
        }
        CD_SCOPE_WINDOW => {
            nvim_docmd_curwin_set_localdir(cwd_ptr);
        }
        CD_SCOPE_INVALID => {
            // Should never happen; abort() in C. Panic in Rust.
            panic!("nvim_docmd_post_chdir_impl: invalid CdScope");
        }
        _ => {}
    }

    nvim_docmd_set_last_chdir_reason_null();
    nvim_docmd_shorten_fnames_nosymlinks();

    if trigger_dirchanged {
        nvim_docmd_do_autocmd_dirchanged_manual_post(cwd_ptr, scope);
    }
}
