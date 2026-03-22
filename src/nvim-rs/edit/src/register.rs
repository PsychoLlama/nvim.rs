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

const BACKWARD: c_int = 0;
const PUT_CURSEND: c_int = 0x04;
const PUT_FIXINDENT: c_int = 0x08;

const FAIL: c_int = 0;

/// `kOptBoFlagRegister` from `option_defs.h`.
const K_OPT_BO_FLAG_REGISTER: c_int = 0x40;

// ============================================================================
// C accessors
// ============================================================================

extern "C" {
    fn nvim_redrawing() -> c_int;
    fn char_avail() -> bool;
    fn ins_redraw_false();
    fn nvim_edit_putchar(c: c_int, highlight: c_int);
    fn edit_unputchar();
    fn nvim_edit_set_pc_status_unset();
    fn add_to_showcmd_c(c: c_int);
    fn rs_clear_showcmd();
    fn nvim_inc_no_mapping();
    fn nvim_dec_no_mapping();
    fn nvim_inc_allow_keys();
    fn nvim_dec_allow_keys();
    fn nvim_plain_vgetc_wrapper() -> c_int;
    fn nvim_langmap_adjust(c: c_int, condition: bool) -> c_int;
    fn nvim_inc_no_u_sync();
    fn nvim_dec_no_u_sync();
    fn nvim_get_u_sync_once() -> c_int;
    fn nvim_set_u_sync_once(val: c_int);
    fn nvim_set_ins_need_undo(val: c_int);
    fn nvim_get_expr_register() -> c_int;
    fn nvim_edit_ins_reg_restore_cursor_save();
    fn nvim_edit_ins_reg_restore_cursor();
    fn nvim_valid_yank_reg(regname: c_int, writing: bool) -> bool;
    fn vim_beep(val: c_uint);
    fn nvim_edit_get_yank_register(regname: c_int) -> *mut std::ffi::c_void;
    fn nvim_edit_reg_y_size(reg: *const std::ffi::c_void) -> usize;
    fn nvim_edit_is_literal_register(regname: c_int) -> c_int;
    fn nvim_edit_append_char_to_redobuff(c: c_int);
    fn nvim_put_do_put(
        regname: c_int,
        savereg: *mut std::ffi::c_void,
        dir: c_int,
        count: c_int,
        flags: c_int,
    );
    fn nvim_edit_insert_reg(regname: c_int, literally: c_int) -> c_int;
    fn nvim_edit_get_stop_insert_mode() -> c_int;
    fn nvim_stuff_empty() -> bool;
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
    nvim_edit_set_pc_status_unset();
    if nvim_redrawing() != 0 && !char_avail() {
        ins_redraw_false();
        nvim_edit_putchar(c_int::from(b'"'), 1);
        add_to_showcmd_c(CTRL_R);
    }

    // Don't map the register name.
    nvim_inc_no_mapping();
    nvim_inc_allow_keys();
    let mut regname = nvim_plain_vgetc_wrapper();
    regname = nvim_langmap_adjust(regname, true);
    if regname == CTRL_R || regname == CTRL_O || regname == CTRL_P {
        // Get a third key for literal register insertion.
        literally = regname;
        add_to_showcmd_c(literally);
        regname = nvim_plain_vgetc_wrapper();
        regname = nvim_langmap_adjust(regname, true);
    }
    nvim_dec_no_mapping();
    nvim_dec_allow_keys();

    // Don't call `u_sync()` while typing the expression or giving error message.
    nvim_inc_no_u_sync();
    if regname == EQ_CHAR {
        nvim_edit_ins_reg_restore_cursor_save();
        nvim_set_u_sync_once(2);
        regname = nvim_get_expr_register();
        nvim_edit_ins_reg_restore_cursor();
    }

    if regname == NUL || !nvim_valid_yank_reg(regname, false) {
        vim_beep(K_OPT_BO_FLAG_REGISTER as c_uint);
        need_redraw = true;
    } else {
        let reg = nvim_edit_get_yank_register(regname);

        if literally == CTRL_O || literally == CTRL_P {
            // Append the command to the redo buffer.
            nvim_edit_append_char_to_redobuff(CTRL_R);
            nvim_edit_append_char_to_redobuff(literally);
            nvim_edit_append_char_to_redobuff(regname);
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
        } else if nvim_edit_reg_y_size(reg) > 1 && nvim_edit_is_literal_register(regname) != 0 {
            nvim_edit_append_char_to_redobuff(CTRL_R);
            nvim_edit_append_char_to_redobuff(regname);
            nvim_put_do_put(regname, std::ptr::null_mut(), BACKWARD, 1, PUT_CURSEND);
        } else if nvim_edit_insert_reg(regname, literally) == FAIL {
            vim_beep(K_OPT_BO_FLAG_REGISTER as c_uint);
            need_redraw = true;
        } else if nvim_edit_get_stop_insert_mode() != 0 {
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
    if need_redraw || nvim_stuff_empty() {
        edit_unputchar();
    }
    rs_clear_showcmd();

    // Disallow starting Visual mode here.
    if vis_active == 0 && nvim_VIsual_active() != 0 {
        end_visual_mode();
    }
}
