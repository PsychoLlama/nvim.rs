//! Menu execution: `execute_menu()` Rust implementation.
//!
//! This replaces the C `execute_menu()` function in `menu.c`.

use std::ffi::{c_char, c_int, c_void};

use crate::handle::VimMenuHandle;
use crate::menu_modes::{
    MENU_INDEX_CMDLINE, MENU_INDEX_INSERT, MENU_INDEX_INVALID, MENU_INDEX_NORMAL,
    MENU_INDEX_OP_PENDING, MENU_INDEX_SELECT, MENU_INDEX_TERMINAL, MENU_INDEX_VISUAL,
};

/// MAXCOL value.
const MAXCOL: i32 = 0x7fff;

/// NUL character.
const NUL: c_int = 0;

/// Opaque handle to `exarg_T*`.
type ExArgHandle = *mut c_void;

/// Cursor position matching C `pos_T`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct PosT {
    lnum: i32,
    col: i32,
    coladd: i32,
}

/// save_state_T opaque blob (4096 bytes is safe for any layout).
#[repr(C, align(8))]
struct SaveStateBlob {
    data: [u8; 4096],
}

impl SaveStateBlob {
    fn new() -> Self {
        Self { data: [0u8; 4096] }
    }
}

extern "C" {
    // ExArg accessors
    fn nvim_menu_eap_get_addr_count(eap: ExArgHandle) -> c_int;
    fn nvim_menu_eap_get_line1(eap: ExArgHandle) -> c_int;
    fn nvim_menu_eap_get_line2(eap: ExArgHandle) -> c_int;

    // Global state
    static State: c_int;
    static mut restart_edit: c_int;
    static mut ex_normal_busy: c_int;
    static mut VIsual_active: bool;
    static mut VIsual_reselect: bool;
    static mut VIsual_mode: c_int;
    static mut VIsual: PosT;
    static mut curwin: *mut c_void;
    static mut curbuf: *mut c_void;
    static p_sel: *const c_char;

    // Mode detection
    fn get_real_state() -> c_int;

    // Current script context
    fn nvim_get_current_sctx_sid() -> c_int;

    // Cursor operations
    fn check_cursor(win: *mut c_void);
    fn gchar_cursor() -> c_int;

    // Window cursor read/write
    fn nvim_win_get_cursor_ptr(wp: *mut c_void) -> *mut PosT;
    fn nvim_win_set_curswant(wp: *mut c_void, val: i32);

    // Buffer visual info accessors
    fn nvim_menu_buf_visual_start_lnum(buf: *mut c_void) -> i32;
    fn nvim_menu_buf_visual_end_lnum(buf: *mut c_void) -> i32;
    fn nvim_menu_buf_visual_end(buf: *mut c_void) -> PosT;
    fn nvim_menu_buf_visual_mode(buf: *mut c_void) -> c_int;
    fn nvim_menu_buf_visual_curswant(buf: *mut c_void) -> i32;
    fn nvim_menu_buf_visual_start(buf: *mut c_void) -> PosT;

    // State save/restore
    fn save_current_state(save: *mut c_void) -> bool;
    fn restore_current_state(save: *mut c_void);

    // Execution
    fn exec_normal_cmd(cmd: *mut c_char, remap: c_int, silent: bool);
    fn ins_typebuf(
        str: *const c_char,
        noremap: c_int,
        offset: c_int,
        nottyped: bool,
        silent: bool,
    ) -> c_int;

    // Error messaging
    fn semsg(s: *const c_char, ...) -> bool;
    fn nvim_gettext(s: *const c_char) -> *const c_char;
}

// Mode flag constants from state_defs.h
const MODE_INSERT: c_int = 0x10;
const MODE_CMDLINE: c_int = 0x08;
const MODE_TERMINAL: c_int = 0x80;
const MODE_VISUAL: c_int = 0x04;

// Error message
const E335: *const c_char = c"E335: Menu not defined for %s mode".as_ptr();

/// Execute "menu". Used by `:emenu` and the window toolbar.
///
/// `eap` may be null for the window toolbar.
/// `mode_idx` specifies a `MENU_INDEX_*` value, or `MENU_INDEX_INVALID` to
/// depend on the current state.
///
/// This is the Rust replacement for C `execute_menu()`.
///
/// # Safety
/// `menu` must be a valid `VimMenuHandle`.
/// `eap` must be a valid `exarg_T*` or null.
#[export_name = "execute_menu"]
pub unsafe extern "C" fn rs_execute_menu(eap: ExArgHandle, menu: VimMenuHandle, mode_idx: c_int) {
    let mut idx = mode_idx;

    if idx < 0 {
        let state = unsafe { State };
        let restart = unsafe { restart_edit };
        let sc_sid = unsafe { nvim_get_current_sctx_sid() };

        if ((state & MODE_INSERT) != 0 || restart != 0) && sc_sid == 0 {
            // Use Insert mode entry when returning to Insert mode.
            idx = MENU_INDEX_INSERT;
        } else if (state & MODE_CMDLINE) != 0 {
            idx = MENU_INDEX_CMDLINE;
        } else if (state & MODE_TERMINAL) != 0 {
            idx = MENU_INDEX_TERMINAL;
        } else if (unsafe { get_real_state() } & MODE_VISUAL) != 0 {
            // Detect real visual mode.
            idx = MENU_INDEX_VISUAL;
        } else if !eap.is_null() && unsafe { nvim_menu_eap_get_addr_count(eap) } != 0 {
            let line1 = unsafe { nvim_menu_eap_get_line1(eap) };
            let line2 = unsafe { nvim_menu_eap_get_line2(eap) };
            let tpos: PosT;

            idx = MENU_INDEX_VISUAL;

            // GEDDES: detect whether this is from a selection by checking if the
            // range matches up with the visual select start/end.
            let vi_start_lnum = unsafe { nvim_menu_buf_visual_start_lnum(curbuf) };
            let vi_end_lnum = unsafe { nvim_menu_buf_visual_end_lnum(curbuf) };

            if vi_start_lnum == line1 && vi_end_lnum == line2 {
                // Set up for visual mode - equivalent to gv.
                unsafe {
                    VIsual_mode = nvim_menu_buf_visual_mode(curbuf);
                    tpos = nvim_menu_buf_visual_end(curbuf);
                    *nvim_win_get_cursor_ptr(curwin) = nvim_menu_buf_visual_start(curbuf);
                    nvim_win_set_curswant(curwin, nvim_menu_buf_visual_curswant(curbuf));
                }
            } else {
                // Set up for line-wise visual mode.
                unsafe {
                    VIsual_mode = c_int::from(b'V');
                    (*nvim_win_get_cursor_ptr(curwin)).lnum = line1;
                    (*nvim_win_get_cursor_ptr(curwin)).col = 1;
                    tpos = PosT {
                        lnum: line2,
                        col: MAXCOL,
                        coladd: 0,
                    };
                }
            }

            // Activate visual mode.
            unsafe {
                VIsual_active = true;
                VIsual_reselect = true;
                check_cursor(curwin);
                VIsual = *nvim_win_get_cursor_ptr(curwin);
                *nvim_win_get_cursor_ptr(curwin) = tpos;
                check_cursor(curwin);

                // Adjust the cursor to make sure it is in the correct pos for exclusive mode.
                if *p_sel == b'e' as c_char && gchar_cursor() != NUL {
                    (*nvim_win_get_cursor_ptr(curwin)).col += 1;
                }
            }
        }
    }

    if idx == MENU_INDEX_INVALID || eap.is_null() {
        idx = MENU_INDEX_NORMAL;
    }

    let menu_ptr = menu.as_ptr();
    let strings_ptr = unsafe { (*menu_ptr).strings[idx as usize] };
    let modes = unsafe { (*menu_ptr).modes };

    if !strings_ptr.is_null() && (modes & (1 << idx)) != 0 {
        let sc_sid = unsafe { nvim_get_current_sctx_sid() };

        if eap.is_null() || sc_sid != 0 {
            // When executing a script/function, execute commands right now.
            // Also for the window toolbar.
            let mut save_state = SaveStateBlob::new();
            unsafe {
                ex_normal_busy += 1;
                if save_current_state(save_state.data.as_mut_ptr().cast()) {
                    exec_normal_cmd(
                        strings_ptr,
                        (*menu_ptr).noremap[idx as usize],
                        (*menu_ptr).silent[idx as usize],
                    );
                }
                restore_current_state(save_state.data.as_mut_ptr().cast());
                ex_normal_busy -= 1;
            }
        } else {
            // Otherwise put them in the typeahead buffer.
            unsafe {
                ins_typebuf(
                    strings_ptr,
                    (*menu_ptr).noremap[idx as usize],
                    0,
                    true,
                    (*menu_ptr).silent[idx as usize],
                );
            }
        }
    } else if !eap.is_null() {
        let mode_str: *const c_char = match idx {
            MENU_INDEX_VISUAL => c"Visual".as_ptr(),
            MENU_INDEX_SELECT => c"Select".as_ptr(),
            MENU_INDEX_OP_PENDING => c"Op-pending".as_ptr(),
            MENU_INDEX_TERMINAL => c"Terminal".as_ptr(),
            MENU_INDEX_INSERT => c"Insert".as_ptr(),
            MENU_INDEX_CMDLINE => c"Cmdline".as_ptr(),
            _ => c"Normal".as_ptr(),
        };
        unsafe { semsg(nvim_gettext(E335), mode_str) };
    }
}
