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
// Exception / msglist repr(C) structs (Phase 4)
// =============================================================================

/// Rust mirror of C `msglist_T` (struct msglist).
/// Fields accessed: next, msg, multiline.
#[repr(C)]
struct MsglistT {
    next: *mut MsglistT,
    msg: *mut c_char,
    throw_msg: *mut c_char,
    sfile: *mut c_char,
    slnum: i32, // linenr_T
    multiline: bool,
}

/// Rust mirror of C `except_T` (struct vim_exception).
/// Fields accessed: type, value, messages, throw_name, throw_lnum.
#[repr(C)]
struct ExceptT {
    type_: c_int, // except_type_T (int)
    value: *mut c_char,
    messages: *mut MsglistT,
    throw_name: *mut c_char,
    throw_lnum: i32, // linenr_T
    stacktrace: *mut c_void,
    caught: *mut ExceptT,
}

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

// redraw type constants
const UPD_VALID: c_int = 10;

// remap constants
const REMAP_NONE: c_int = -1;

// Event constants from auevents_enum.generated.h
const EVENT_QUITPRE: c_int = 90;
const EVENT_EXITPRE: c_int = 47;

// cmdmod flag constants (from ex_cmds_defs.h)
const CMOD_SANDBOX: c_int = 0x0001;
const CMOD_SILENT: c_int = 0x0002;
const CMOD_ERRSILENT: c_int = 0x0004;
const CMOD_UNSILENT: c_int = 0x0008;
const CMOD_NOAUTOCMD: c_int = 0x0010;

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

    // Window iteration (firstwin-based, for curtab)
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;

    // Window preview flag accessor (returns c_int, 0/1)
    fn nvim_win_get_p_pvw(wp: WinHandle) -> c_int;

    // Prepare / restore preview window
    fn nvim_docmd_prepare_preview_window();
    fn nvim_docmd_back_to_current_window(curwin_save: WinHandle);

    // find_pattern_in_path (C function in search_shim.c)
    #[link_name = "find_pattern_in_path"]
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

    // check_nextcmd (Rust export)
    fn check_nextcmd(p: *const c_char) -> *mut c_char;

    // getdigits
    fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int;
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;

    // syncbind window/buffer accessors
    fn nvim_win_get_w_p_scb(wp: WinHandle) -> bool;
    fn nvim_buf_get_ml_line_count(buf: BufHandle) -> LinenrT;
    fn nvim_win_set_scbind_pos(wp: WinHandle, val: c_int);
    fn nvim_win_set_redr_status(wp: WinHandle, val: c_int);
    fn nvim_curtab_first_win() -> WinHandle;
    fn nvim_win_get_next_in_tab(wp: WinHandle) -> WinHandle;
    #[link_name = "rs_get_vtopline"]
    fn nvim_docmd_get_vtopline(wp: WinHandle) -> c_int;
    #[link_name = "rs_get_scrolloff_value"]
    fn nvim_docmd_get_scrolloff_value(wp: WinHandle) -> c_int;
    fn plines_m_win_fill(wp: WinHandle, first: LinenrT, last: LinenrT) -> c_int;
    #[link_name = "cursor_correct"]
    fn nvim_docmd_cursor_correct(wp: WinHandle);
    fn setpcmark();
    fn checkpcmark();
    fn nvim_set_did_syncbind(val: bool);
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> LinenrT;
    // scrollup/scrolldown (Rust exports)
    #[link_name = "scrollup"]
    fn scrollup(wp: WinHandle, line_count: LinenrT, byfold: c_int) -> c_int;
    #[link_name = "scrolldown"]
    fn scrolldown(wp: WinHandle, line_count: LinenrT, byfold: c_int) -> c_int;
    // redraw_later
    #[link_name = "redraw_later"]
    fn nvim_redraw_later(wp: WinHandle, rtype: c_int);

    // These are implemented in Rust (impl_bodies.rs) but called via extern for local use
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
    fn nvim_docmd_is_only_tabpage() -> c_int;
    fn goto_tabpage(n: c_int);
    #[link_name = "rs_valid_tabpage"]
    fn rs_valid_tabpage(tp: *mut c_void) -> c_int;
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
    let line2 = (*eap).line2;
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

    let arg = (*eap).arg;
    let count = if (*eap).addr_count > 0 {
        (*eap).line2
    } else {
        1
    };
    let forceit = (*eap).forceit != 0;
    rs_do_tag(arg, final_cmd, count, forceit as c_int, true);
}

/// `:tag`, `:tselect`, `:tjump`, etc.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_tag"]
pub unsafe extern "C" fn rs_ex_tag(eap: ExArgHandle) {
    let cmdidx = (*eap).cmdidx;
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
    let cmdidx = (*eap).cmdidx;
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
    postponed_split_flags = crate::cmdmod.cmod_split;
    postponed_split_tab = crate::cmdmod.cmod_tab;
    let cmdidx = (*eap).cmdidx;
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
    let forceit = (*eap).forceit != 0;
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
    rs_do_exedit_impl(eap, std::ptr::null_mut());
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
    let cmdidx = (*eap).cmdidx;
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

    let mut eap_arg = (*eap).arg;
    let forceit = (*eap).forceit != 0;

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
                (*eap).errmsg = c"E488: Trailing characters: %s".as_ptr() as *mut c_char;
            } else {
                let nextcmd = check_nextcmd(p_after);
                (*eap).nextcmd = nextcmd;
            }
        }
    }

    // Only execute if not skipping
    if (*eap).skip == 0 {
        let len = strlen(eap_arg as *const c_char);
        let type_ = if !cmd_name.is_null() && *cmd_name == b'd' as c_char {
            FIND_DEFINE
        } else {
            FIND_ANY
        };
        let line1 = (*eap).line1;
        let line2 = (*eap).line2;
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
    msg_scroll = 1;

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
        if crate::got_int {
            break;
        }

        msg_putchar(b'\n' as c_int);
        let label = nvim_docmd_tab_page_fmt(tabcount);
        nvim_docmd_msg_outtrans_attr(label, HLF_T);
        tabcount += 1;
        os_breakcheck();

        let mut wp = nvim_tabpage_get_firstwin(tp);
        while !wp.is_null() {
            if crate::got_int {
                break;
            }
            if nvim_win_get_focusable(wp) == 0 || nvim_ex2_win_get_w_config_hide(wp) {
                wp = nvim_win_get_next(wp);
                continue;
            }

            msg_putchar(b'\n' as c_int);
            msg_putchar(if wp == curwin {
                b'>' as c_int
            } else if wp == lastused_win {
                b'#' as c_int
            } else {
                b' ' as c_int
            });
            msg_putchar(b' ' as c_int);
            let buf = nvim_win_get_buffer(wp);
            msg_putchar(if bufIsChanged(buf) != 0 {
                b'+' as c_int
            } else {
                b' ' as c_int
            });
            msg_putchar(b' ' as c_int);

            let spname = nvim_docmd_buf_spname(buf);
            if !spname.is_null() {
                let iobuff = std::ptr::addr_of_mut!(IObuff) as *mut c_char;
                nvim_xstrlcpy(iobuff, spname, nvim_iosize());
            } else {
                let fname = nvim_buf_get_b_fname(buf);
                nvim_docmd_home_replace(buf, fname);
            }
            let iobuff = std::ptr::addr_of_mut!(IObuff) as *mut c_char;
            nvim_docmd_msg_outtrans_attr(iobuff, 0);
            os_breakcheck();

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

/// `:syncbind` implementation.
///
/// # Safety
/// All window/buffer handles must be valid.
#[export_name = "nvim_docmd_ex_syncbind_impl"]
pub unsafe extern "C" fn rs_ex_syncbind_impl(_eap: ExArgHandle) {
    let curwin = nvim_get_curwin();
    let old_linenr = nvim_win_get_cursor_lnum(curwin);

    setpcmark();

    // Determine the minimum virtual topline across all scrollbind windows.
    let vtopline: LinenrT;
    if nvim_win_get_w_p_scb(curwin) {
        let mut vt = nvim_docmd_get_vtopline(curwin);
        let mut wp = nvim_curtab_first_win();
        while !wp.is_null() {
            if nvim_win_get_w_p_scb(wp) {
                let buf = nvim_win_get_buffer(wp);
                if !buf.is_null() {
                    let lcount = nvim_buf_get_ml_line_count(buf);
                    let y =
                        plines_m_win_fill(wp, 1, lcount) - nvim_docmd_get_scrolloff_value(curwin);
                    if y < vt {
                        vt = y;
                    }
                }
            }
            wp = nvim_win_get_next_in_tab(wp);
        }
        vtopline = if vt < 1 { 1 } else { vt };
    } else {
        vtopline = 1;
    }

    // Scroll all scrollbind windows to the target topline.
    let mut wp = nvim_curtab_first_win();
    while !wp.is_null() {
        if nvim_win_get_w_p_scb(wp) {
            let y = vtopline - nvim_docmd_get_vtopline(wp);
            if y > 0 {
                scrollup(wp, y, 1);
            } else {
                scrolldown(wp, -y, 1);
            }
            nvim_win_set_scbind_pos(wp, vtopline);
            nvim_redraw_later(wp, UPD_VALID);
            nvim_docmd_cursor_correct(wp);
            nvim_win_set_redr_status(wp, 1);
        }
        wp = nvim_win_get_next_in_tab(wp);
    }

    if nvim_win_get_w_p_scb(curwin) {
        nvim_set_did_syncbind(true);
        checkpcmark();
        if old_linenr != nvim_win_get_cursor_lnum(curwin) {
            let ctrl_o = [b'\x0f' as c_char, 0i8];
            nvim_docmd_ins_typebuf(ctrl_o.as_ptr(), REMAP_NONE, 0, true, false);
        }
    }
}

/// `:syncbind` - synchronize scrollbind windows.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_syncbind"]
pub unsafe extern "C" fn rs_ex_syncbind(eap: ExArgHandle) {
    rs_ex_syncbind_impl(eap);
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

    let tab_number = crate::address::rs_get_tabpage_arg(eap);
    if !(*eap).errmsg.is_null() {
        return;
    }

    goto_tabpage(tab_number);

    // Repeat up to 1000 times: autocommands may mess up the lists.
    let forceit = (*eap).forceit as c_int;
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
    let flags = (*eap).flags;
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
#[export_name = "update_topline_cursor"]
pub unsafe extern "C" fn rs_update_topline_cursor_impl() {
    let curwin = nvim_get_curwin();
    check_cursor(curwin);
    update_topline(curwin);
    if nvim_docmd_curwin_p_wrap() == 0 {
        validate_cursor(curwin);
    }
    update_curswant();
}

/// `nvim_docmd_vim_mkdir_emsg_impl` - create directory, emit error on failure.
///
/// # Safety
/// `name` must be a valid null-terminated C string.
#[export_name = "vim_mkdir_emsg"]
pub unsafe extern "C" fn rs_vim_mkdir_emsg_impl(name: *const c_char, prot: c_int) -> c_int {
    let ret = os_mkdir(name, prot);
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
#[export_name = "open_exfile"]
pub unsafe extern "C" fn rs_open_exfile_impl(
    fname: *mut c_char,
    forceit: c_int,
    mode: *mut c_char,
) -> *mut c_void {
    // On UNIX, check for directory.
    #[cfg(unix)]
    if os_isdir(fname) {
        nvim_docmd_semsg_isadir2(fname);
        return std::ptr::null_mut();
    }
    // Check if file exists when not appending and not forcing.
    if forceit == 0 && *mode != b'a' as c_char && os_path_exists(fname) {
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
    fn nvim_docmd_tabpage_new_impl();

    // ex_win_close_impl helpers
    #[link_name = "is_aucmd_win"]
    fn nvim_is_aucmd_win(wp: WinHandle) -> c_int;
    fn nvim_emsg_id(id: c_int);
    fn bufIsChanged(buf: BufHandle) -> c_int;
    fn nvim_get_p_confirm() -> c_int;
    fn nvim_get_cmdmod_confirm() -> c_int;
    fn nvim_get_p_write() -> c_int;
    fn nvim_docmd_dialog_changed_still_dirty(buf: BufHandle) -> bool;
    fn no_write_message();
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
    fn apply_autocmds(
        event: c_int,
        fname: *const c_char,
        fname_io: *const c_char,
        force: bool,
        buf: BufHandle,
    ) -> bool;

    // ex_tabs helpers
    #[link_name = "msg_start"]
    fn nvim_msg_start();
    fn nvim_docmd_tab_page_fmt(n: c_int) -> *mut c_char;
    fn nvim_docmd_msg_outtrans_attr(s: *const c_char, attr: c_int);
    #[link_name = "buf_spname"]
    fn nvim_docmd_buf_spname(buf: BufHandle) -> *mut c_char;
    fn nvim_docmd_home_replace(buf: BufHandle, src: *const c_char);
    static mut IObuff: [c_char; 1025];
    fn nvim_win_get_focusable(wp: WinHandle) -> c_int;
    fn nvim_ex2_win_get_w_config_hide(wp: WinHandle) -> bool;
    fn nvim_tabpage_get_curwin(tp: *mut c_void) -> WinHandle;
    fn nvim_get_lastused_tabpage() -> *mut c_void;
    fn nvim_tabpage_get_firstwin(tp: *mut c_void) -> WinHandle;
    fn nvim_buf_get_b_fname(buf: BufHandle) -> *const c_char;
    fn msg_putchar(c: c_int);
    fn os_breakcheck();
    fn nvim_iosize() -> usize;
    fn nvim_xstrlcpy(dst: *mut c_char, src: *const c_char, n: usize);

    // do_exbuffer helpers
    fn nvim_docmd_goto_buffer_current(eap: ExArgHandle);
    fn nvim_docmd_goto_buffer_first(eap: ExArgHandle, n: c_int);
    fn nvim_docmd_eap_get_do_ecmd_cmd(eap: ExArgHandle) -> *mut c_char;
    fn ex_errmsg(msg: *const c_char, arg: *const c_char) -> *mut c_char;
    static e_trailing_arg: [c_char; 1];

    // handle_did_throw globals (Phase 4)
    static mut current_exception: *mut ExceptT;
    static mut suppress_errthrow: bool;
    static mut force_abort: bool;
    static mut emsg_silent: c_int;
    fn nvim_docmd_fmt_exception_not_caught(value: *const c_char) -> *mut c_char;
    fn nvim_docmd_free_sourcing_name_and_pop();
    #[link_name = "discard_current_exception"]
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
    fn nvim_get_lastwin() -> WinHandle;
    fn nvim_docmd_ex_win_close_curwin(forceit: c_int);
    fn close_others(message: c_int, forceit: c_int);
    fn nvim_docmd_tabpage_get_lastwin(tp: *mut c_void) -> WinHandle;
    fn nvim_docmd_tabpage_lastwin_eq(tp: *mut c_void, wp: WinHandle) -> c_int;
    fn nvim_docmd_ex_win_close_in_tab(forceit: c_int, wp: WinHandle, tp: *mut c_void);

    // ex_find helpers
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
    fn check_can_set_curbuf_forceit(forceit: c_int) -> bool;

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

    // Wrappers that call C globals/functions - kept as thin C shims.
    fn nvim_cmod_capture_msg_scroll(cmod: CmodHandle);
    fn nvim_cmod_regfree_filter(cmod: CmodHandle);
    fn nvim_docmd_set_eventignore_all();
    fn nvim_docmd_set_eventignore_str(s: *mut c_char);
    static mut msg_silent: c_int;
    fn nvim_inc_msg_silent();
    fn nvim_get_p_verbose() -> c_int;
    fn nvim_set_p_verbose(val: c_int);
    fn nvim_get_p_ei() -> *const c_char;
    fn nvim_inc_sandbox();
    fn nvim_dec_sandbox();
    fn xstrdup(s: *const c_char) -> *mut c_char;
    #[link_name = "free_string_option"]
    fn nvim_free_string_option(p: *mut c_char);
    fn nvim_did_emsg_check() -> c_int;
    fn redirecting() -> c_int;
    static mut msg_col: c_int;
}

use crate::CmdMod;
type CmodHandle = *mut CmdMod;

// =============================================================================
// Phase N+15: save_current_state_impl / restore_current_state_impl
// =============================================================================

use crate::SaveState;
type SstHandle = *mut SaveState;

extern "C" {
    // Global accessors for save/restore state
    static mut msg_scroll: c_int;
    static mut msg_didout: bool;
    fn nvim_get_current_State() -> c_int;
    fn nvim_set_current_State(val: c_int);
    fn nvim_get_finish_op() -> c_int;
    fn nvim_get_opcount() -> c_int;
    fn nvim_set_opcount(val: c_int);
    fn nvim_get_reg_executing() -> c_int;
    static mut reg_executing: c_int;
    static mut pending_end_reg_executing: bool;
    fn nvim_get_force_restart_edit() -> c_int;
    fn nvim_set_force_restart_edit(val: c_int);
    // typeahead - kept as C wrappers since they call save_typeahead/restore_typeahead
    fn nvim_docmd_sst_save_typeahead(sst: SstHandle) -> c_int;
    fn nvim_docmd_sst_restore_typeahead(sst: SstHandle);
    // ui_cursor_shape
    fn ui_cursor_shape();
}

/// Save current editor state for :normal execution.
///
/// # Safety
/// `sst` must be a valid save_state_T pointer.
#[export_name = "save_current_state"]
pub unsafe extern "C" fn rs_save_current_state_impl(sst: SstHandle) -> bool {
    (*sst).save_msg_scroll = msg_scroll;
    (*sst).save_restart_edit = crate::restart_edit;
    (*sst).save_msg_didout = msg_didout;
    (*sst).save_state = nvim_get_current_State();
    (*sst).save_finish_op = nvim_get_finish_op() != 0;
    (*sst).save_opcount = nvim_get_opcount();
    (*sst).save_reg_executing = nvim_get_reg_executing();
    (*sst).save_pending_end_reg_executing = pending_end_reg_executing;

    msg_scroll = 0; // no msg scrolling in Normal mode
    crate::restart_edit = 0; // don't go to Insert mode

    // Save typeahead; return typebuf_valid.
    nvim_docmd_sst_save_typeahead(sst) != 0
}

/// Restore current editor state after :normal execution.
///
/// # Safety
/// `sst` must be a valid save_state_T pointer.
#[export_name = "restore_current_state"]
pub unsafe extern "C" fn rs_restore_current_state_impl(sst: SstHandle) {
    nvim_docmd_sst_restore_typeahead(sst);

    msg_scroll = (*sst).save_msg_scroll;
    if nvim_get_force_restart_edit() != 0 {
        nvim_set_force_restart_edit(0);
    } else {
        // Some function (terminal_enter()) may have overridden restart_edit.
        crate::restart_edit = (*sst).save_restart_edit;
    }
    nvim_set_finish_op((*sst).save_finish_op);
    nvim_set_opcount((*sst).save_opcount);
    reg_executing = (*sst).save_reg_executing;
    pending_end_reg_executing = (*sst).save_pending_end_reg_executing;

    // Don't reset msg_didout now; OR in the saved value.
    msg_didout = msg_didout || (*sst).save_msg_didout;

    // Restore the state (needed when called from a function executed for
    // 'indentexpr'). Update the mouse and cursor.
    nvim_set_current_State((*sst).save_state);
    ui_cursor_shape();
}

/// `do_exedit` - edit/badd/visual command dispatch.
///
/// # Safety
/// `eap` must be a valid ExArgHandle. `old_curwin` may be null.
#[no_mangle]
pub unsafe extern "C" fn do_exedit(eap: ExArgHandle, old_curwin: WinHandle) {
    rs_do_exedit_impl(eap, old_curwin);
}

/// `ex_splitview` - split/vsplit/tabedit dispatch.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_splitview"]
pub unsafe extern "C" fn rs_ex_splitview(eap: ExArgHandle) {
    rs_ex_splitview_impl(eap);
}

/// `:find` - find file in path and edit it.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "nvim_docmd_ex_find_impl"]
pub unsafe extern "C" fn rs_ex_find_impl(eap: ExArgHandle) {
    let forceit = (*eap).forceit as c_int;
    if !check_can_set_curbuf_forceit(forceit) {
        return;
    }

    let arg = (*eap).arg;
    let len = std::ffi::CStr::from_ptr(arg).to_bytes().len();

    let fname: *mut c_char = if nvim_docmd_get_findfunc_nonempty() {
        let count = if (*eap).addr_count > 0 {
            (*eap).line2 as c_int
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
        if (*eap).addr_count > 0 {
            let mut count = (*eap).line2 as c_int;
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

    (*eap).arg = fname;
    rs_do_exedit_impl(eap, std::ptr::null_mut());
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
#[export_name = "exec_normal"]
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
        if !cont || crate::got_int {
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
#[export_name = "exec_normal_cmd"]
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
    apply_autocmds(
        EVENT_QUITPRE,
        std::ptr::null(),
        std::ptr::null(),
        false,
        buf,
    );

    // Bail out when autocommands closed the window, or the buffer is locked.
    if !rs_win_valid(wp)
        || nvim_curbuf_locked() != 0
        || (nvim_buf_get_nwindows(buf) == 1 && nvim_buf_get_locked(buf) > 0)
    {
        return true;
    }

    let ok: c_int = 1; // OK
    if quit_all || (nvim_docmd_check_more(0, forceit as c_int) == ok && rs_only_one_window() != 0) {
        apply_autocmds(
            EVENT_EXITPRE,
            std::ptr::null(),
            std::ptr::null(),
            false,
            nvim_get_curbuf(),
        );
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

    if need_hide && !buf_hide(buf) && forceit == 0 {
        if (nvim_get_p_confirm() != 0 || nvim_get_cmdmod_confirm() != 0) && nvim_get_p_write() != 0
        {
            // Show dialog; if buffer still changed after, cancel close.
            if nvim_docmd_dialog_changed_still_dirty(buf) {
                return;
            }
            need_hide = false;
        } else {
            no_write_message();
            return;
        }
    }

    // Free buffer when not hiding it or when it's a scratch buffer.
    let free_buf = !need_hide && !buf_hide(buf);
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
    if nvim_get_firstwin() != nvim_get_lastwin() {
        close_others(1, (forceit != 0) as c_int);
    }
    // If only one window left, close it too (which closes the tab).
    if nvim_get_firstwin() == nvim_get_lastwin() {
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
    let arg = (*eap).arg;
    if !arg.is_null() && *arg != 0 {
        let errmsg = ex_errmsg(e_trailing_arg.as_ptr(), arg);
        (*eap).errmsg = errmsg;
    } else {
        if (*eap).addr_count == 0 {
            nvim_docmd_goto_buffer_current(eap);
        } else {
            nvim_docmd_goto_buffer_first(eap, (*eap).line2);
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

    let exc = current_exception;
    let etype = (*exc).type_;
    let p: *mut c_char;
    let messages: *mut MsglistT;

    match etype {
        ET_USER => {
            p = nvim_docmd_fmt_exception_not_caught((*exc).value);
            messages = std::ptr::null_mut();
        }
        ET_ERROR => {
            p = std::ptr::null_mut();
            messages = (*exc).messages;
            (*exc).messages = std::ptr::null_mut();
        }
        _ => {
            // ET_INTERRUPT: no message
            p = std::ptr::null_mut();
            messages = std::ptr::null_mut();
        }
    }

    // Push throw location onto execution stack.
    let throw_name = (*exc).throw_name;
    let throw_lnum = (*exc).throw_lnum;
    nvim_estack_push(5 /* ETYPE_EXCEPT */, throw_name, throw_lnum);
    (*exc).throw_name = std::ptr::null_mut();

    nvim_discard_current_exception(); // uses IObuff if 'verbose'

    // If "silent!" is not active, mark as fatal.
    if emsg_silent == 0 {
        suppress_errthrow = true;
        force_abort = true;
    }

    // Display the error message(s).
    if !messages.is_null() {
        let mut cur = messages;
        while !cur.is_null() {
            let next = (*cur).next;
            emsg_multiline(
                (*cur).msg,
                c"emsg".as_ptr(),
                HLF_E,
                c_int::from((*cur).multiline),
            );
            xfree((*cur).msg as *mut c_void);
            xfree((*cur).sfile as *mut c_void);
            xfree(cur as *mut c_void);
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

    // nvim_get_RedrawingDisabled + nvim_set_redrawing_disabled: deleted (Phase 1), use RedrawingDisabled directly
    #[link_name = "RedrawingDisabled"]
    static mut g_RedrawingDisabled: c_int;

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
    fn nvim_set_ex_pressedreturn(val: bool);

    // ex_no_reprint set/get
    fn nvim_set_ex_no_reprint(val: c_int);
    fn nvim_get_ex_no_reprint() -> c_int;

    // msg_row get/set
    static mut msg_row: c_int;
    fn nvim_ex2_set_msg_row(val: c_int);

    // curwin->w_cursor.lnum
    fn nvim_get_curwin_cursor_lnum() -> c_int;

    // cmdline_row set
    static mut cmdline_row: c_int;

    // lines_left / Rows
    static mut lines_left: c_int;
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

    // Helpers for vim_mkdir_emsg and open_exfile (migrated to Rust)
    fn nvim_docmd_semsg_mkdir_err(name: *const c_char, errcode: c_int);
    fn nvim_docmd_semsg_file_exists(fname: *const c_char);
    fn nvim_docmd_semsg_cant_open_write(fname: *const c_char);
    fn nvim_docmd_semsg_isadir2(fname: *const c_char);
    fn os_mkdir(name: *const c_char, prot: c_int) -> c_int;
    fn os_isdir(fname: *const c_char) -> bool;
    fn os_path_exists(fname: *const c_char) -> bool;
    fn nvim_docmd_os_fopen(fname: *const c_char, mode: *const c_char) -> *mut c_void;

    // Accessors for update_topline_cursor (migrated to Rust)
    fn nvim_docmd_curwin_p_wrap() -> c_int;
    fn check_cursor(win: WinHandle);
    fn update_topline(win: WinHandle);
    fn validate_cursor(win: WinHandle);
    fn update_curswant();

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
    const DIP_ALL: c_int = 0x01;
    if nvim_docmd_get_filetype_plugin() == K_NONE {
        source_runtime(c"ftplugin.vim".as_ptr(), DIP_ALL);
        nvim_docmd_set_filetype_plugin(K_TRUE);
    }
    if nvim_docmd_get_filetype_indent() == K_NONE {
        source_runtime(c"indent.vim".as_ptr(), DIP_ALL);
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
        source_runtime(c"filetype.lua filetype.vim".as_ptr(), 0x01); // DIP_ALL
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

    let save_msg_scroll = msg_scroll;
    g_RedrawingDisabled += 1; // don't redisplay the window
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
        msg_scroll = 1;
        need_wait_return = false;
        nvim_set_ex_pressedreturn(false);
        nvim_set_ex_no_reprint(0);
        let changedtick = nvim_docmd_curbuf_changedtick();
        let prev_msg_row = msg_row;
        let prev_line = nvim_get_curwin_cursor_lnum();
        cmdline_row = msg_row;
        nvim_docmd_do_cmdline_getexline_noflags();
        lines_left = nvim_ses_get_Rows() - 1;

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

    g_RedrawingDisabled -= 1;
    nvim_ex2_set_no_wait_return(nvim_ex2_get_no_wait_return() - 1);
    nvim_redraw_all_later(UPD_NOT_VALID);
    update_screen();
    need_wait_return = false;
    msg_scroll = save_msg_scroll;
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

// =============================================================================
// Phase N+14: apply_cmdmod_impl / undo_cmdmod_impl
// =============================================================================

/// Apply command modifiers: sandbox, verbose, silent, errsilent, noautocmd.
///
/// # Safety
/// `cmod` must be a valid cmdmod_T pointer.
#[export_name = "apply_cmdmod"]
pub unsafe extern "C" fn rs_apply_cmdmod_impl(cmod: CmodHandle) {
    let flags = (*cmod).cmod_flags;

    // :sandbox
    if (flags & CMOD_SANDBOX) != 0 && (*cmod).cmod_did_sandbox == 0 {
        nvim_inc_sandbox();
        (*cmod).cmod_did_sandbox = 1;
    }

    // :verbose
    let verbose = (*cmod).cmod_verbose;
    if verbose > 0 {
        if (*cmod).cmod_verbose_save == 0 {
            (*cmod).cmod_verbose_save = i64::from(nvim_get_p_verbose() + 1);
        }
        nvim_set_p_verbose(verbose - 1);
    }

    // :silent / :unsilent
    if (flags & (CMOD_SILENT | CMOD_UNSILENT)) != 0 && (*cmod).cmod_save_msg_silent == 0 {
        (*cmod).cmod_save_msg_silent = msg_silent + 1;
        nvim_cmod_capture_msg_scroll(cmod);
    }
    if (flags & CMOD_SILENT) != 0 {
        nvim_inc_msg_silent();
    }
    if (flags & CMOD_UNSILENT) != 0 {
        msg_silent = 0;
    }

    // :silent!
    if (flags & CMOD_ERRSILENT) != 0 {
        emsg_silent += 1;
        (*cmod).cmod_did_esilent += 1;
    }

    // :noautocmd
    if (flags & CMOD_NOAUTOCMD) != 0 && (*cmod).cmod_save_ei.is_null() {
        let p_ei = nvim_get_p_ei();
        (*cmod).cmod_save_ei = xstrdup(p_ei);
        nvim_docmd_set_eventignore_all();
    }
}

/// Undo and free command modifier state.
///
/// # Safety
/// `cmod` must be a valid cmdmod_T pointer.
#[export_name = "undo_cmdmod"]
pub unsafe extern "C" fn rs_undo_cmdmod_impl(cmod: CmodHandle) {
    // Restore verbose.
    let verbose_save = (*cmod).cmod_verbose_save;
    if verbose_save > 0 {
        nvim_set_p_verbose((verbose_save - 1) as c_int);
        (*cmod).cmod_verbose_save = 0;
    }

    // Undo sandbox.
    if (*cmod).cmod_did_sandbox != 0 {
        nvim_dec_sandbox();
        (*cmod).cmod_did_sandbox = 0;
    }

    // Restore 'eventignore'.
    let save_ei = (*cmod).cmod_save_ei;
    if !save_ei.is_null() {
        nvim_docmd_set_eventignore_str(save_ei);
        nvim_free_string_option(save_ei);
        (*cmod).cmod_save_ei = std::ptr::null_mut();
    }

    // Free filter pattern and regexp.
    xfree((*cmod).cmod_filter_pat.cast::<c_void>());
    (*cmod).cmod_filter_pat = std::ptr::null_mut();
    nvim_cmod_regfree_filter(cmod);

    // Restore msg_silent.
    let save_msg_silent = (*cmod).cmod_save_msg_silent;
    if save_msg_silent > 0 {
        // Prevent counters from going negative if a serious error enabled messages.
        let cur_msg_silent = msg_silent;
        if nvim_did_emsg_check() == 0 || cur_msg_silent > save_msg_silent - 1 {
            msg_silent = save_msg_silent - 1;
        }
        let did_esilent = (*cmod).cmod_did_esilent;
        let new_emsg_silent = emsg_silent - did_esilent;
        emsg_silent = if new_emsg_silent < 0 {
            0
        } else {
            new_emsg_silent
        };
        // Restore msg_scroll (set by file I/O commands even with no message).
        msg_scroll = (*cmod).cmod_save_msg_scroll;
        // Restore msg_col if redirecting.
        if redirecting() != 0 {
            msg_col = 0;
        }
        (*cmod).cmod_save_msg_silent = 0;
        (*cmod).cmod_did_esilent = 0;
    }
}

// =============================================================================
// Phase N+16: ex_splitview_impl
// =============================================================================

extern "C" {
    // eap cmdidx setter (for splitview quickfix adjustment)

    // CMD constants now use crate::cmd_idx::{CMD_*} from build.rs-generated file.

    // curbuf quickfix check
    fn nvim_bt_quickfix_curbuf() -> c_int;

    // global cmdmod accessors
    fn nvim_get_cmdmod_tab() -> c_int;
    fn nvim_docmd_get_global_cmdmod_flags() -> c_int;

    // win_new_tabpage (exported from Rust window crate)
    #[link_name = "win_new_tabpage"]
    fn nvim_docmd_win_new_tabpage(after: c_int, filename: *const u8) -> c_int;

    // EVENT_TABNEWENTERED autocmd
    fn nvim_apply_autocmds_tabnewentered();

    // win buffer comparison
    fn nvim_win_buf_is_curbuf(wp: WinHandle) -> c_int;

    // w_alt_fnum setter + curbuf b_fnum getter
    fn nvim_docmd_win_set_alt_fnum(wp: WinHandle, fnum: c_int);
    fn nvim_buf_get_fnum(buf: *mut c_void) -> c_int;

    // win_split: returns OK (1) on success, FAIL (0) on failure
    fn win_split(size: c_int, flags: c_int) -> c_int;

    // eap->cmd[0] accessor

    // scrollbind reset/check
    fn nvim_reset_binding_curwin();
    fn do_check_scrollbind(flag: bool);
}

// CMOD_KEEPALT = 0x0100 (from ex_cmds_defs.h)
const CMOD_KEEPALT: c_int = 0x0100;
// WSP_VERT = 0x02 (from window.h)
const WSP_VERT: c_int = 0x02;

/// `:split`, `:vsplit`, `:new`, `:vnew`, `:sfind`, `:tabedit`, `:tabnew`, `:tabfind`.
///
/// # Safety
/// `eap` must be a valid ExArgHandle.
#[export_name = "nvim_docmd_ex_splitview_impl"]
pub unsafe extern "C" fn rs_ex_splitview_impl(eap: ExArgHandle) {
    let old_curwin = nvim_get_curwin();

    let cmd_tabedit = crate::cmd_idx::CMD_tabedit;
    let cmd_tabfind = crate::cmd_idx::CMD_tabfind;
    let cmd_tabnew = crate::cmd_idx::CMD_tabnew;
    let cmd_split = crate::cmd_idx::CMD_split;
    let cmd_vsplit = crate::cmd_idx::CMD_vsplit;
    let cmd_new = crate::cmd_idx::CMD_new;
    let cmd_vnew = crate::cmd_idx::CMD_vnew;
    let cmd_sfind = crate::cmd_idx::CMD_sfind;

    let mut cmdidx = (*eap).cmdidx;
    let use_tab = cmdidx == cmd_tabedit || cmdidx == cmd_tabfind || cmdidx == cmd_tabnew;

    // A ":split" in the quickfix window works like ":new". Don't want two
    // quickfix windows. But it's OK when doing ":tab split".
    if nvim_bt_quickfix_curbuf() != 0 && nvim_get_cmdmod_tab() == 0 {
        if cmdidx == cmd_split {
            cmdidx = cmd_new;
            (*eap).cmdidx = cmdidx;
        }
        if cmdidx == cmd_vsplit {
            cmdidx = cmd_vnew;
            (*eap).cmdidx = cmdidx;
        }
    }

    let mut fname: *mut c_char = std::ptr::null_mut();
    let arg = (*eap).arg;

    if cmdidx == cmd_sfind || cmdidx == cmd_tabfind {
        let len = std::ffi::CStr::from_ptr(arg).to_bytes().len();
        if nvim_docmd_get_findfunc_nonempty() {
            let count = if (*eap).addr_count > 0 {
                (*eap).line2 as c_int
            } else {
                1
            };
            fname = nvim_docmd_findfunc_find_file(arg, len, count);
        } else {
            let mut file_to_find: *mut c_char = std::ptr::null_mut();
            let mut search_ctx: *mut c_void = std::ptr::null_mut();
            const FNAME_MESS: c_int = 1;
            let rel = nvim_docmd_curbuf_b_ffname();
            fname = nvim_docmd_find_file_in_path(
                arg,
                len,
                FNAME_MESS,
                1,
                rel,
                &mut file_to_find,
                &mut search_ctx,
            );
            xfree(file_to_find as *mut c_void);
            nvim_docmd_vim_findfile_cleanup(search_ctx);
        }
        if fname.is_null() {
            return;
        }
        (*eap).arg = fname;
    }

    // Either open new tab page or split the window.
    if use_tab {
        let cmod_tab = nvim_get_cmdmod_tab();
        let after = if cmod_tab != 0 {
            cmod_tab
        } else if (*eap).addr_count == 0 {
            0
        } else {
            (*eap).line2 as c_int + 1
        };
        if nvim_docmd_win_new_tabpage(after, (*eap).arg as *const u8) != FAIL {
            rs_do_exedit_impl(eap, old_curwin);
            nvim_apply_autocmds_tabnewentered();

            // Set the alternate buffer for the window we came from.
            let curwin = nvim_get_curwin();
            if curwin != old_curwin
                && rs_win_valid(old_curwin)
                && nvim_win_buf_is_curbuf(old_curwin) == 0
                && (nvim_docmd_get_global_cmdmod_flags() & CMOD_KEEPALT) == 0
            {
                nvim_docmd_win_set_alt_fnum(old_curwin, nvim_buf_get_fnum(nvim_get_curbuf()));
            }
        }
    } else {
        let size = if (*eap).addr_count > 0 {
            (*eap).line2 as c_int
        } else {
            0
        };
        let cmd_ptr = (*eap).cmd;
        let flags = if !cmd_ptr.is_null() && *cmd_ptr == b'v' as c_char {
            WSP_VERT
        } else {
            0
        };
        if win_split(size, flags) != FAIL {
            // Reset 'scrollbind' when editing another file, but keep it when
            // doing ":split" without arguments.
            let arg2 = (*eap).arg;
            if !arg2.is_null() && *arg2 != 0 {
                nvim_reset_binding_curwin();
            } else {
                do_check_scrollbind(false);
            }
            rs_do_exedit_impl(eap, old_curwin);
        }
    }

    xfree(fname as *mut c_void);
}

// =============================================================================
// Phase N+17: do_exedit_impl
// =============================================================================

extern "C" {
    // CMD_view/enew/sview/balt/badd use crate::cmd_idx::{CMD_*}.
    // readonlymode now via crate::readonlymode
    fn buf_hide(buf: BufHandle) -> bool;
    fn nvim_docmd_set_curbuf_b_p_ro(v: c_int);
    fn nvim_docmd_eap_get_do_ecmd_lnum(eap: ExArgHandle) -> LinenrT;
    fn nvim_docmd_do_exedit_handle_exmode(eap: ExArgHandle) -> c_int;
    fn nvim_docmd_do_exedit_split_fail_cleanup();
    fn nvim_docmd_do_exedit_split_fallback(eap: ExArgHandle);
    fn nvim_text_or_buf_locked() -> c_int;
    #[link_name = "do_ecmd"]
    fn nvim_docmd_do_ecmd(
        fnum: c_int,
        ffname: *mut c_char,
        sfname: *mut c_char,
        eap: ExArgHandle,
        newlnum: LinenrT,
        flags: c_int,
        oldwin: WinHandle,
    ) -> c_int;
}

const ECMD_HIDE: c_int = 0x01;
const ECMD_OLDBUF: c_int = 0x04;
const ECMD_FORCEIT: c_int = 0x08;
const ECMD_ADDBUF: c_int = 0x10;
const ECMD_ALTBUF: c_int = 0x20;
const ECMD_ONE: LinenrT = 1;

#[export_name = "nvim_docmd_do_exedit_impl"]
pub unsafe extern "C" fn rs_do_exedit_impl(eap: ExArgHandle, old_curwin: WinHandle) {
    let cmd_new = crate::cmd_idx::CMD_new;
    let cmd_tabnew = crate::cmd_idx::CMD_tabnew;
    let cmd_tabedit = crate::cmd_idx::CMD_tabedit;
    let cmd_vnew = crate::cmd_idx::CMD_vnew;
    let cmd_split = crate::cmd_idx::CMD_split;
    let cmd_vsplit = crate::cmd_idx::CMD_vsplit;
    let cmd_view = crate::cmd_idx::CMD_view;
    let cmd_enew = crate::cmd_idx::CMD_enew;
    let cmd_sview = crate::cmd_idx::CMD_sview;
    let cmd_balt = crate::cmd_idx::CMD_balt;
    let cmd_badd = crate::cmd_idx::CMD_badd;
    let cmdidx = (*eap).cmdidx;
    let arg = (*eap).arg;
    let forceit = (*eap).forceit != 0;
    if nvim_docmd_do_exedit_handle_exmode(eap) != 0 {
        return;
    }
    if (cmdidx == cmd_new || cmdidx == cmd_tabnew || cmdidx == cmd_tabedit || cmdidx == cmd_vnew)
        && (*arg == 0)
    {
        setpcmark();
        let oldwin = if old_curwin.is_null() {
            nvim_get_curwin()
        } else {
            std::ptr::null_mut()
        };
        nvim_docmd_do_ecmd(
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            eap,
            ECMD_ONE,
            ECMD_HIDE + if forceit { ECMD_FORCEIT } else { 0 },
            oldwin,
        );
    } else if (cmdidx != cmd_split && cmdidx != cmd_vsplit) || (*arg != 0) {
        if *arg != 0 && nvim_text_or_buf_locked() != 0 {
            return;
        }
        let n = crate::readonlymode;
        if cmdidx == cmd_view || cmdidx == cmd_sview {
            crate::readonlymode = true;
        } else if cmdidx == cmd_enew {
            crate::readonlymode = false;
        }
        if cmdidx != cmd_balt && cmdidx != cmd_badd {
            setpcmark();
        }
        let curbuf = nvim_get_curbuf();
        let flags = (if buf_hide(curbuf) { ECMD_HIDE } else { 0 })
            + (if forceit { ECMD_FORCEIT } else { 0 })
            + (if !old_curwin.is_null() {
                ECMD_OLDBUF
            } else {
                0
            })
            + (if cmdidx == cmd_badd { ECMD_ADDBUF } else { 0 })
            + (if cmdidx == cmd_balt { ECMD_ALTBUF } else { 0 });
        let ffname = if cmdidx == cmd_enew {
            std::ptr::null_mut()
        } else {
            arg
        };
        let oldwin = if old_curwin.is_null() {
            nvim_get_curwin()
        } else {
            std::ptr::null_mut()
        };
        let newlnum = nvim_docmd_eap_get_do_ecmd_lnum(eap);
        if nvim_docmd_do_ecmd(0, ffname, std::ptr::null_mut(), eap, newlnum, flags, oldwin) == FAIL
        {
            if !old_curwin.is_null() {
                nvim_docmd_do_exedit_split_fail_cleanup();
            }
        } else if crate::readonlymode && nvim_buf_get_nwindows(nvim_get_curbuf()) == 1 {
            nvim_docmd_set_curbuf_b_p_ro(1);
        }
        crate::readonlymode = n;
    } else {
        nvim_docmd_do_exedit_split_fallback(eap);
    }
    let curwin = nvim_get_curwin();
    if !old_curwin.is_null()
        && *arg != 0
        && curwin != old_curwin
        && rs_win_valid(old_curwin)
        && nvim_win_buf_is_curbuf(old_curwin) == 0
        && (nvim_docmd_get_global_cmdmod_flags() & CMOD_KEEPALT) == 0
    {
        nvim_docmd_win_set_alt_fnum(old_curwin, nvim_buf_get_fnum(nvim_get_curbuf()));
    }
    nvim_set_ex_no_reprint(1);
}
