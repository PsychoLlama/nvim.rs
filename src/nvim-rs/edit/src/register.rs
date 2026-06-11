//! `ins_reg` -- CTRL-R register insertion in insert mode
//!
//! Ported from `edit.c` `ins_reg()`. Handles:
//! - Showing the `"` prompt
//! - Reading register name with literal mode (`CTRL-R`/`CTRL-O`/`CTRL-P`)
//! - Expression register (`=`)
//! - Register insertion (put, `insert_reg`, etc.)
//! - Showcmd/visualmode cleanup

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_int, c_uint};

// ============================================================================
// Constants
// ============================================================================

const NUL: c_int = 0;
const CTRL_R: c_int = 18; // ^R
const CTRL_O: c_int = 15; // ^O
const CTRL_P: c_int = 16; // ^P
const EQ_CHAR: c_int = b'=' as c_int; // '='

/// Direction constant matching C `BACKWARD = -1` from `vim_defs.h`.
const BACKWARD: c_int = -1;
const PUT_CURSEND: c_int = 2; // leave cursor after end of new text (register_defs.h)
const PUT_FIXINDENT: c_int = 1; // make indent look nice (register_defs.h)

const FAIL: c_int = 0;

/// `YREG_PASTE` from `register_defs.h` (first enum value = 0).
const YREG_PASTE: c_int = 0;

/// `kOptBoFlagRegister` from `option_defs.h`.
const K_OPT_BO_FLAG_REGISTER: c_int = 0x40;

// ============================================================================
// C accessors
// ============================================================================

extern "C" {
    fn nvim_redrawing() -> c_int;
    fn char_avail() -> bool;
    fn nvim_putchar(c: c_int, highlight: c_int);
    fn edit_unputchar();
    fn nvim_set_pc_status_unset();
    fn add_to_showcmd_c(c: c_int);
    fn rs_clear_showcmd();
    static mut no_mapping: c_int;
    static mut allow_keys: c_int;
    fn plain_vgetc() -> c_int;
    fn nvim_langmap_adjust(c: c_int, condition: bool) -> c_int;
    fn nvim_inc_no_u_sync();
    fn nvim_dec_no_u_sync();
    fn nvim_get_u_sync_once() -> c_int;
    fn nvim_set_u_sync_once(val: c_int);
    fn nvim_set_ins_need_undo(val: c_int);
    fn get_expr_register() -> c_int;
    fn nvim_ins_reg_restore_cursor_save();
    fn nvim_ins_reg_restore_cursor();
    fn valid_yank_reg(regname: c_int, writing: bool) -> bool;
    fn vim_beep(val: c_uint);
    #[link_name = "get_yank_register"]
    fn nvim_get_yank_register_paste(regname: c_int, mode: c_int) -> *mut std::ffi::c_void;
    fn nvim_reg_y_size(reg: *const std::ffi::c_void) -> usize;
    #[link_name = "rs_is_literal_register"]
    fn nvim_is_literal_register(regname: c_int) -> c_int;
    fn AppendCharToRedobuff(c: c_int);
    #[link_name = "do_put"]
    fn nvim_put_do_put(
        regname: c_int,
        savereg: *mut std::ffi::c_void,
        dir: c_int,
        count: c_int,
        flags: c_int,
    );
    fn insert_reg(regname: c_int, reg: *mut std::ffi::c_void, literally_arg: bool) -> c_int;
    fn nvim_get_stop_insert_mode() -> c_int;
    fn stuff_empty() -> bool;
    fn nvim_VIsual_active() -> c_int;
    fn end_visual_mode();
}

// ============================================================================
// Implementation
// ============================================================================

/// Handle CTRL-R in insert mode: insert register contents.
///
/// # Safety
/// Accesses global state via C accessor functions.
#[unsafe(export_name = "ins_reg")]
pub unsafe extern "C" fn rs_ins_reg() {
    let mut need_redraw = false;
    let mut literally: c_int = 0;
    let vis_active = nvim_VIsual_active();

    // If we are going to wait for a character, show a `"`.
    nvim_set_pc_status_unset();
    if nvim_redrawing() != 0 && !char_avail() {
        crate::redraw::ins_redraw_impl(false);
        nvim_putchar(c_int::from(b'"'), 1);
        add_to_showcmd_c(CTRL_R);
    }

    // Don't map the register name.
    no_mapping += 1;
    allow_keys += 1;
    let mut regname = plain_vgetc();
    regname = nvim_langmap_adjust(regname, true);
    if regname == CTRL_R || regname == CTRL_O || regname == CTRL_P {
        // Get a third key for literal register insertion.
        literally = regname;
        add_to_showcmd_c(literally);
        regname = plain_vgetc();
        regname = nvim_langmap_adjust(regname, true);
    }
    no_mapping -= 1;
    allow_keys -= 1;

    // Don't call `u_sync()` while typing the expression or giving error message.
    nvim_inc_no_u_sync();
    if regname == EQ_CHAR {
        nvim_ins_reg_restore_cursor_save();
        nvim_set_u_sync_once(2);
        regname = get_expr_register();
        nvim_ins_reg_restore_cursor();
    }

    if regname == NUL || !valid_yank_reg(regname, false) {
        vim_beep(K_OPT_BO_FLAG_REGISTER as c_uint);
        need_redraw = true;
    } else {
        let reg = nvim_get_yank_register_paste(regname, YREG_PASTE);

        if literally == CTRL_O || literally == CTRL_P {
            // Append the command to the redo buffer.
            AppendCharToRedobuff(CTRL_R);
            AppendCharToRedobuff(literally);
            AppendCharToRedobuff(regname);
            nvim_put_do_put(
                regname,
                std::ptr::null_mut(),
                BACKWARD,
                1,
                if literally == CTRL_P {
                    PUT_FIXINDENT | PUT_CURSEND
                } else {
                    PUT_CURSEND
                },
            );
        } else if nvim_reg_y_size(reg) > 1 && nvim_is_literal_register(regname) != 0 {
            AppendCharToRedobuff(CTRL_R);
            AppendCharToRedobuff(regname);
            nvim_put_do_put(regname, std::ptr::null_mut(), BACKWARD, 1, PUT_CURSEND);
        } else if insert_reg(regname, std::ptr::null_mut(), literally != 0) == FAIL {
            vim_beep(K_OPT_BO_FLAG_REGISTER as c_uint);
            need_redraw = true;
        } else if nvim_get_stop_insert_mode() != 0 {
            // `":stopinsert"` was invoked; nothing will be inserted.
            need_redraw = true;
        }
    }

    nvim_dec_no_u_sync();
    if nvim_get_u_sync_once() == 1 {
        nvim_set_ins_need_undo(1);
    }
    nvim_set_u_sync_once(0);

    // If the inserted register is empty, remove the `"`.
    if need_redraw || stuff_empty() {
        edit_unputchar();
    }
    rs_clear_showcmd();

    // Disallow starting Visual mode here.
    if vis_active == 0 && nvim_VIsual_active() != 0 {
        end_visual_mode();
    }
}
