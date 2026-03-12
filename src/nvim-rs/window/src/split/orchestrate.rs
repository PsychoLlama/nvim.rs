//! win_split_ins orchestrator — migrated from window.c lines 2974-3491.
//!
//! This module implements the core logic of win_split_ins: validation,
//! size calculation, window allocation, frame reorganization, dimension
//! assignment, equalization, and scroll fixup.
//!
//! Autocmd triggers (win_enter_ext) and option restore (p_wiw/p_wh) remain
//! in the C wrapper.

// Option values and window dimensions may need truncation when converting
// between i64 and c_int, but values are guaranteed to be in valid range.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_lines)]

use std::ffi::c_int;

use crate::frame::constants::{
    STATUS_HEIGHT, WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_HELP, WSP_NOENTER, WSP_ROOM, WSP_TOP,
    WSP_VERT,
};
use crate::{Frame, TabpageHandle, WinHandle, FR_COL, FR_ROW};

// =============================================================================
// FFI constants
// =============================================================================

/// WEE_TRIGGER_NEW_AUTOCMDS flag for win_enter_ext.
const WEE_TRIGGER_NEW_AUTOCMDS: c_int = 0x04;
/// WEE_TRIGGER_ENTER_AUTOCMDS flag for win_enter_ext.
const WEE_TRIGGER_ENTER_AUTOCMDS: c_int = 0x08;
/// WEE_TRIGGER_LEAVE_AUTOCMDS flag for win_enter_ext.
const WEE_TRIGGER_LEAVE_AUTOCMDS: c_int = 0x10;

/// UPD_NOT_VALID constant from drawscreen.h.
const UPD_NOT_VALID: c_int = 40;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // --- Global state ---
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_get_topframe() -> *mut Frame;
    fn nvim_get_Rows() -> c_int;
    fn nvim_get_Columns() -> c_int;
    fn nvim_get_sc_col() -> c_int;

    // --- Options ---
    fn nvim_get_p_wmw() -> i64;
    fn nvim_get_p_wmh() -> i64;
    fn nvim_get_p_wiw() -> i64;
    fn nvim_get_p_wh() -> i64;
    fn nvim_get_p_ea() -> c_int;
    fn nvim_get_p_ead_char() -> c_int;
    fn nvim_get_p_ls() -> i64;
    fn nvim_get_p_sb() -> c_int;
    fn nvim_get_p_spr() -> c_int;
    fn nvim_get_p_ch() -> i64;
    fn nvim_set_p_wiw(val: i64);
    fn nvim_set_p_wh(val: i64);

    // --- Window field accessors ---
    fn nvim_win_get_w_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;
    fn nvim_win_get_prev(wp: WinHandle) -> WinHandle;
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;
    fn nvim_win_get_wfh(wp: WinHandle) -> c_int;
    fn nvim_win_get_wfw(wp: WinHandle) -> c_int;
    fn nvim_win_get_status_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_hsep_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_vsep_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_winrow(wp: WinHandle) -> c_int;
    fn nvim_win_get_wincol(wp: WinHandle) -> c_int;
    fn nvim_win_get_winbar_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_fraction(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_scr(wp: WinHandle) -> i64;
    fn nvim_win_get_config_external_int(wp: WinHandle) -> c_int;

    // --- Window field setters ---
    fn nvim_win_set_winrow(wp: WinHandle, val: c_int);
    fn nvim_win_set_wincol(wp: WinHandle, val: c_int);
    fn nvim_win_set_status_height(wp: WinHandle, val: c_int);
    fn nvim_win_set_hsep_height(wp: WinHandle, val: c_int);
    fn nvim_win_set_vsep_width(wp: WinHandle, val: c_int);
    fn nvim_win_set_pos_changed(wp: WinHandle, val: c_int);
    fn nvim_win_set_fraction(wp: WinHandle, val: c_int);
    fn nvim_win_set_p_scr(wp: WinHandle, val: i64);
    fn nvim_win_set_floating(wp: WinHandle, val: c_int);

    // --- Wrappers for complex operations ---
    fn rs_win_alloc(after: WinHandle, hidden: c_int) -> WinHandle;
    fn rs_new_frame(wp: WinHandle);
    fn rs_win_init(wp: WinHandle, oldwin: WinHandle, flags: c_int);
    fn rs_frame_flatten(frp: *mut Frame);
    fn nvim_xcalloc_frame() -> *mut Frame;
    fn nvim_ui_comp_remove_grid_win(wp: WinHandle);
    fn nvim_ui_has_multigrid() -> c_int;
    fn nvim_ui_call_win_hide_win(wp: WinHandle);
    fn rs_win_free_grid(wp: WinHandle, reinit: c_int);
    fn nvim_merge_win_config_init(wp: WinHandle);
    fn nvim_redraw_later_wrapper(wp: WinHandle, r#type: c_int);
    fn nvim_status_redraw_all_wrapper();
    fn nvim_msg_clr_eos_force();
    fn nvim_comp_col();
    fn nvim_win_float_anchor_laststatus();
    #[link_name = "rs_set_fraction"]
    fn nvim_set_fraction_wrapper(wp: WinHandle);
    #[link_name = "rs_win_setheight_win"]
    fn nvim_win_setheight_win_wrapper(height: c_int, wp: WinHandle);
    #[link_name = "rs_win_setwidth_win"]
    fn nvim_win_setwidth_win_wrapper(width: c_int, wp: WinHandle);
    #[link_name = "rs_one_window_in_tab"]
    fn nvim_one_window_firstwin(firstwin: WinHandle, tp: TabpageHandle) -> c_int;
    fn nvim_is_aucmd_win(wp: WinHandle) -> c_int;
    // nvim_fixup_external_curwin removed: replaced by crate::close::win_close::fixup_external_curwin (Phase 8)
    fn nvim_set_msg_row_val(val: c_int);
    fn nvim_set_msg_col_val(val: c_int);

    // --- Already-migrated Rust functions (called via extern C) ---
    fn rs_frame_minheight(topfrp: *const Frame, next_curwin: WinHandle) -> c_int;
    fn rs_frame_minwidth(topfrp: *const Frame, next_curwin: WinHandle) -> c_int;
    fn rs_win_new_height(wp: WinHandle, height: c_int);
    fn rs_win_new_width(wp: WinHandle, width: c_int);
    #[link_name = "frame_new_height"]
    fn rs_frame_new_height(
        topfrp: *mut Frame,
        height: c_int,
        topfirst: c_int,
        wfh: c_int,
        set_ch: c_int,
    );
    fn rs_frame_new_width(topfrp: *mut Frame, width: c_int, leftfirst: c_int, wfw: c_int);
    fn rs_frame_fix_height(wp: WinHandle);
    fn rs_frame_fix_width(wp: WinHandle);
    fn rs_frame_set_vsep(frp: *const Frame, add: c_int);
    fn rs_frame_add_hsep(frp: *const Frame);
    fn rs_frame_add_statusline(frp: *mut Frame);
    fn rs_frame_insert(before: *mut Frame, frp: *mut Frame);
    fn rs_frame_append(after: *mut Frame, frp: *mut Frame);
    fn rs_win_append(after: WinHandle, wp: WinHandle, tp: WinHandle);
    fn rs_win_comp_pos() -> c_int;
    fn rs_win_equal(next_curwin: WinHandle, current: c_int, dir: c_int);
    fn rs_set_fraction(wp: WinHandle);
    fn rs_tabline_height() -> c_int;
    fn rs_global_stl_height() -> c_int;

    // --- win_fix_scroll (stays in C) ---
    fn win_fix_scroll(size_changed: c_int);

    // --- lastwin_nofloating (called from C too) ---
    #[link_name = "rs_lastwin_nofloating"]
    fn lastwin_nofloating() -> WinHandle;
}

// NOWIN sentinel — matches C code: #define NOWIN ((win_T *)-1)
const NOWIN: WinHandle = unsafe { WinHandle::from_ptr((-1isize) as *mut std::ffi::c_void) };

// =============================================================================
// EMSG IDs
// =============================================================================

/// E442: Can't split topleft and botright at the same time
const EMSG_E442: c_int = 3;
/// e_noroom: Not enough room
const EMSG_NOROOM: c_int = 13;

// =============================================================================
// Result type
// =============================================================================

/// Result returned to the C wrapper.
#[repr(C)]
pub struct SplitInsResult {
    pub wp: WinHandle,
    pub do_enter: c_int,
    pub enter_flags: c_int,
    pub vertical: c_int,
    pub saved_option: c_int,
}

// =============================================================================
// Implementation
// =============================================================================

/// Core logic of win_split_ins.
///
/// # Safety
/// Called from C with valid pointers or null.
unsafe fn win_split_ins_impl(
    size: c_int,
    flags: c_int,
    new_wp: WinHandle,
    dir: c_int,
    to_flatten: *mut Frame,
) -> SplitInsResult {
    let fail = SplitInsResult {
        wp: WinHandle::null(),
        do_enter: 0,
        enter_flags: 0,
        vertical: 0,
        saved_option: 0,
    };

    // aucmd_win[] should always remain floating
    if !new_wp.is_null() && nvim_is_aucmd_win(new_wp) != 0 {
        return fail;
    }

    let oldwin = get_oldwin(flags);
    let vertical = (flags & WSP_VERT) != 0;
    let toplevel = (flags & (WSP_TOP | WSP_BOT)) != 0;

    // add a status line when p_ls == 1 and splitting the first window
    let need_status = calc_need_status(oldwin);

    // --- Size validation and calculation ---
    let calc = if vertical {
        calc_vertical_split(flags, oldwin, size, toplevel)
    } else {
        calc_horizontal_split(flags, oldwin, size, toplevel, need_status)
    };
    let Some(calc) = calc else {
        return fail;
    };

    let new_size = calc.new_size;
    let do_equal = calc.do_equal;
    let oldwin_height = calc.oldwin_height;
    let did_set_fraction = calc.did_set_fraction;

    // --- Allocate new window and link into list ---
    let wp = alloc_and_link(flags, oldwin, new_wp, vertical);
    if wp.is_null() {
        return fail;
    }

    // --- Initialize new window ---
    init_new_window(wp, new_wp, flags);

    // --- Flatten frames if needed ---
    if !to_flatten.is_null() {
        rs_frame_flatten(to_flatten);
    }

    // --- Reorganize frame tree ---
    let layout = if vertical { FR_ROW } else { FR_COL };
    let (curfrp, before) = reorganize_frame_tree(flags, oldwin, toplevel, vertical, layout);

    // --- Insert frame ---
    let frp = if new_wp.is_null() {
        nvim_win_get_frame(wp)
    } else {
        nvim_win_get_frame(new_wp)
    };
    (*frp).fr_parent = (*curfrp).fr_parent;

    if before {
        rs_frame_insert(curfrp, frp);
    } else {
        rs_frame_append(curfrp, frp);
    }

    // --- Set fractions ---
    if !did_set_fraction {
        rs_set_fraction(oldwin);
    }
    let old_fraction = nvim_win_get_fraction(oldwin);
    nvim_win_set_fraction(wp, old_fraction);

    // --- Assign dimensions ---
    if vertical {
        assign_vertical_dimensions(
            wp,
            oldwin,
            curfrp,
            frp,
            flags,
            new_size,
            need_status,
            toplevel,
            before,
        );
    } else {
        assign_horizontal_dimensions(
            wp,
            oldwin,
            curfrp,
            frp,
            flags,
            new_size,
            oldwin_height,
            toplevel,
            before,
        );
    }

    if toplevel {
        rs_win_comp_pos();
    }

    // --- Redraw ---
    nvim_redraw_later_wrapper(wp, UPD_NOT_VALID);
    nvim_redraw_later_wrapper(oldwin, UPD_NOT_VALID);
    nvim_status_redraw_all_wrapper();

    if need_status != 0 {
        nvim_set_msg_row_val(nvim_get_Rows() - 1);
        nvim_set_msg_col_val(nvim_get_sc_col());
        nvim_msg_clr_eos_force();
        nvim_comp_col();
        nvim_set_msg_row_val(nvim_get_Rows() - 1);
        nvim_set_msg_col_val(0);
    }

    // --- Equalize ---
    if do_equal || dir != 0 {
        let eq_dir = if vertical {
            if dir == i32::from(b'v') {
                i32::from(b'b')
            } else {
                i32::from(b'h')
            }
        } else if dir == i32::from(b'h') {
            i32::from(b'b')
        } else {
            i32::from(b'v')
        };
        rs_win_equal(wp, 1, eq_dir);
    } else if nvim_is_aucmd_win(wp) == 0 {
        win_fix_scroll(0);
    }

    // --- Save option and prepare enter flags ---
    let saved_option;
    if flags & WSP_VERT != 0 {
        saved_option = nvim_get_p_wiw() as c_int;
        if size != 0 {
            nvim_set_p_wiw(i64::from(size));
        }
    } else {
        saved_option = nvim_get_p_wh() as c_int;
        if size != 0 {
            nvim_set_p_wh(i64::from(size));
        }
    }

    let do_enter = (flags & WSP_NOENTER) == 0;
    let enter_flags = if new_wp.is_null() {
        WEE_TRIGGER_NEW_AUTOCMDS
    } else {
        0
    } | WEE_TRIGGER_ENTER_AUTOCMDS
        | WEE_TRIGGER_LEAVE_AUTOCMDS;

    SplitInsResult {
        wp,
        do_enter: c_int::from(do_enter),
        enter_flags,
        vertical: c_int::from(vertical),
        saved_option,
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// Determine oldwin based on flags.
unsafe fn get_oldwin(flags: c_int) -> WinHandle {
    if (flags & WSP_TOP) != 0 {
        nvim_get_firstwin()
    } else if (flags & WSP_BOT) != 0 || nvim_win_get_floating(nvim_get_curwin()) != 0 {
        lastwin_nofloating()
    } else {
        nvim_get_curwin()
    }
}

/// Calculate need_status: 1 if we need to add a status line, 0 otherwise.
unsafe fn calc_need_status(oldwin: WinHandle) -> c_int {
    if nvim_one_window_firstwin(nvim_get_firstwin(), TabpageHandle::null()) != 0
        && nvim_get_p_ls() == 1
        && nvim_win_get_status_height(oldwin) == 0
    {
        if nvim_win_get_w_height(oldwin) <= nvim_get_p_wmh() as c_int {
            // will fail with noroom — handled by caller
            return -1; // signal error
        }
        nvim_win_float_anchor_laststatus();
        STATUS_HEIGHT
    } else {
        0
    }
}

/// Result of size calculations.
struct SplitCalc {
    new_size: c_int,
    do_equal: bool,
    oldwin_height: c_int,
    did_set_fraction: bool,
}

/// Calculate size for vertical split. Returns None on "no room".
unsafe fn calc_vertical_split(
    flags: c_int,
    oldwin: WinHandle,
    size: c_int,
    toplevel: bool,
) -> Option<SplitCalc> {
    let p_wmw = nvim_get_p_wmw() as c_int;
    let wmw1 = if p_wmw == 0 { 1 } else { p_wmw };
    let mut needed = wmw1 + 1;
    if (flags & WSP_ROOM) != 0 {
        needed += nvim_get_p_wiw() as c_int - wmw1;
    }

    let oldwin_frame = nvim_win_get_frame(oldwin);
    let topframe = nvim_get_topframe();
    let (minwidth, available);

    if toplevel {
        minwidth = rs_frame_minwidth(topframe, NOWIN);
        available = (*topframe).fr_width;
    } else if nvim_get_p_ea() != 0 {
        let mut mw = rs_frame_minwidth(oldwin_frame, NOWIN);
        let mut prevfrp = oldwin_frame;
        let mut frp = (*oldwin_frame).fr_parent;
        while !frp.is_null() {
            if (*frp).fr_layout == FR_ROW {
                let mut frp2 = (*frp).fr_child;
                while !frp2.is_null() {
                    if frp2 != prevfrp {
                        mw += rs_frame_minwidth(frp2, NOWIN);
                    }
                    frp2 = (*frp2).fr_next;
                }
            }
            prevfrp = frp;
            frp = (*frp).fr_parent;
        }
        minwidth = mw;
        available = (*topframe).fr_width;
    } else {
        minwidth = rs_frame_minwidth(oldwin_frame, NOWIN);
        available = (*oldwin_frame).fr_width;
    }
    needed += minwidth;

    if available < needed {
        nvim_emsg_id(EMSG_NOROOM);
        return None;
    }

    let mut new_size = if size == 0 {
        nvim_win_get_w_width(oldwin) / 2
    } else {
        size
    };
    new_size = new_size.min(available - minwidth - 1).max(wmw1);

    let mut do_equal = nvim_win_get_w_width(oldwin) - new_size - 1 < p_wmw;

    // winfixwidth: try to expand oldwin
    if nvim_win_get_wfw(oldwin) != 0 {
        nvim_win_setwidth_win_wrapper(nvim_win_get_w_width(oldwin) + new_size + 1, oldwin);
    }

    // Check siblings for equalization
    if !do_equal && nvim_get_p_ea() != 0 && size == 0 && nvim_get_p_ead_char() != i32::from(b'v') {
        let parent = (*oldwin_frame).fr_parent;
        if !parent.is_null() {
            let mut frp = (*parent).fr_child;
            while !frp.is_null() {
                let fw = (*frp).fr_win;
                if fw != oldwin
                    && !fw.is_null()
                    && (nvim_win_get_w_width(fw) > new_size
                        || nvim_win_get_w_width(fw) > nvim_win_get_w_width(oldwin) - new_size - 1)
                {
                    do_equal = true;
                    break;
                }
                frp = (*frp).fr_next;
            }
        }
    }

    Some(SplitCalc {
        new_size,
        do_equal,
        oldwin_height: 0,
        did_set_fraction: false,
    })
}

/// Calculate size for horizontal split. Returns None on "no room".
unsafe fn calc_horizontal_split(
    flags: c_int,
    oldwin: WinHandle,
    size: c_int,
    toplevel: bool,
    need_status: c_int,
) -> Option<SplitCalc> {
    if need_status < 0 {
        // error already indicated by calc_need_status
        nvim_emsg_id(EMSG_NOROOM);
        return None;
    }

    let p_wmh = nvim_get_p_wmh() as c_int;
    let wmh1 = p_wmh.max(1) + nvim_win_get_winbar_height(oldwin);
    let mut needed = wmh1 + STATUS_HEIGHT;
    if (flags & WSP_ROOM) != 0 {
        needed += nvim_get_p_wh() as c_int - wmh1 + nvim_win_get_winbar_height(oldwin);
    }
    if nvim_get_p_ch() < 1 {
        needed += 1; // cmdheight=0 adjustment
    }

    let oldwin_frame = nvim_win_get_frame(oldwin);
    let topframe = nvim_get_topframe();
    let (minheight, available);

    if toplevel {
        minheight = rs_frame_minheight(topframe, NOWIN) + need_status;
        available = (*topframe).fr_height;
    } else if nvim_get_p_ea() != 0 {
        let mut mh = rs_frame_minheight(oldwin_frame, NOWIN) + need_status;
        let mut prevfrp = oldwin_frame;
        let mut frp = (*oldwin_frame).fr_parent;
        while !frp.is_null() {
            if (*frp).fr_layout == FR_COL {
                let mut frp2 = (*frp).fr_child;
                while !frp2.is_null() {
                    if frp2 != prevfrp {
                        mh += rs_frame_minheight(frp2, NOWIN);
                    }
                    frp2 = (*frp2).fr_next;
                }
            }
            prevfrp = frp;
            frp = (*frp).fr_parent;
        }
        minheight = mh;
        available = (*topframe).fr_height;
    } else {
        minheight = rs_frame_minheight(oldwin_frame, NOWIN) + need_status;
        available = (*oldwin_frame).fr_height;
    }
    needed += minheight;

    if available < needed {
        nvim_emsg_id(EMSG_NOROOM);
        return None;
    }

    let mut oldwin_height = nvim_win_get_w_height(oldwin);
    if need_status != 0 {
        nvim_win_set_status_height(oldwin, STATUS_HEIGHT);
        oldwin_height -= STATUS_HEIGHT;
    }

    let mut new_size = if size == 0 { oldwin_height / 2 } else { size };
    new_size = new_size
        .min(available - minheight - STATUS_HEIGHT)
        .max(wmh1);

    let mut do_equal = oldwin_height - new_size - STATUS_HEIGHT < p_wmh;

    let mut did_set_fraction = false;

    // winfixheight
    if nvim_win_get_wfh(oldwin) != 0 {
        rs_set_fraction(oldwin);
        did_set_fraction = true;
        nvim_win_setheight_win_wrapper(
            nvim_win_get_w_height(oldwin) + new_size + STATUS_HEIGHT,
            oldwin,
        );
        oldwin_height = nvim_win_get_w_height(oldwin);
        if need_status != 0 {
            oldwin_height -= STATUS_HEIGHT;
        }
    }

    // Check siblings for equalization
    if !do_equal && nvim_get_p_ea() != 0 && size == 0 && nvim_get_p_ead_char() != i32::from(b'h') {
        let parent = (*oldwin_frame).fr_parent;
        if !parent.is_null() {
            let mut frp = (*parent).fr_child;
            while !frp.is_null() {
                let fw = (*frp).fr_win;
                if fw != oldwin
                    && !fw.is_null()
                    && (nvim_win_get_w_height(fw) > new_size
                        || nvim_win_get_w_height(fw) > oldwin_height - new_size - STATUS_HEIGHT)
                {
                    do_equal = true;
                    break;
                }
                frp = (*frp).fr_next;
            }
        }
    }

    Some(SplitCalc {
        new_size,
        do_equal,
        oldwin_height,
        did_set_fraction,
    })
}

/// Allocate new window and link it into the window list.
unsafe fn alloc_and_link(
    flags: c_int,
    oldwin: WinHandle,
    new_wp: WinHandle,
    vertical: bool,
) -> WinHandle {
    let place_after = if (flags & WSP_TOP) == 0
        && ((flags & WSP_BOT) != 0
            || (flags & WSP_BELOW) != 0
            || ((flags & WSP_ABOVE) == 0
                && (if vertical {
                    nvim_get_p_spr() != 0
                } else {
                    nvim_get_p_sb() != 0
                }))) {
        // new window below/right
        oldwin
    } else {
        // new window above/left
        nvim_win_get_prev(oldwin)
    };

    if new_wp.is_null() {
        rs_win_alloc(place_after, 0)
    } else {
        rs_win_append(place_after, new_wp, WinHandle::null());
        new_wp
    }
}

/// Initialize a newly created or moved window.
unsafe fn init_new_window(wp: WinHandle, new_wp: WinHandle, flags: c_int) {
    if new_wp.is_null() {
        // Fresh allocation
        rs_new_frame(wp);
        rs_win_init(wp, nvim_get_curwin(), flags);
    } else if nvim_win_get_floating(wp) != 0 {
        // Moving a floating window into the layout
        nvim_ui_comp_remove_grid_win(wp);
        if nvim_ui_has_multigrid() != 0 {
            nvim_win_set_pos_changed(wp, 1);
        } else {
            nvim_ui_call_win_hide_win(wp);
            rs_win_free_grid(wp, 1);
        }

        if nvim_win_get_config_external_int(wp) != 0 {
            // nvim_fixup_external_curwin migrated to Rust (Phase 8)
            crate::close::win_close::fixup_external_curwin(wp);
        }

        nvim_win_set_floating(wp, 0);
        rs_new_frame(wp);
        nvim_merge_win_config_init(wp);
    }
}

/// Reorganize the frame tree and return (curfrp, before).
unsafe fn reorganize_frame_tree(
    flags: c_int,
    oldwin: WinHandle,
    toplevel: bool,
    vertical: bool,
    layout: i8,
) -> (*mut Frame, bool) {
    let topframe = nvim_get_topframe();
    let mut curfrp;
    let before;

    if toplevel {
        if ((*topframe).fr_layout == FR_COL && !vertical)
            || ((*topframe).fr_layout == FR_ROW && vertical)
        {
            curfrp = (*topframe).fr_child;
            if (flags & WSP_BOT) != 0 {
                while !curfrp.is_null() && !(*curfrp).fr_next.is_null() {
                    curfrp = (*curfrp).fr_next;
                }
            }
        } else {
            curfrp = topframe;
        }
        before = (flags & WSP_TOP) != 0;
    } else {
        curfrp = nvim_win_get_frame(oldwin);
        if (flags & WSP_BELOW) != 0 {
            before = false;
        } else if (flags & WSP_ABOVE) != 0 {
            before = true;
        } else if vertical {
            before = nvim_get_p_spr() == 0;
        } else {
            before = nvim_get_p_sb() == 0;
        }
    }

    // Create a new branch in the frame tree if needed
    if (*curfrp).fr_parent.is_null() || (*(*curfrp).fr_parent).fr_layout != layout {
        let new_frp = nvim_xcalloc_frame();
        *new_frp = *curfrp;
        (*curfrp).fr_layout = layout;
        (*new_frp).fr_parent = curfrp;
        (*new_frp).fr_next = std::ptr::null_mut();
        (*new_frp).fr_prev = std::ptr::null_mut();
        (*curfrp).fr_child = new_frp;
        (*curfrp).fr_win = WinHandle::null();
        curfrp = new_frp;
        if (*new_frp).fr_win.is_null() {
            // Container frame — reparent children
            let mut child = (*new_frp).fr_child;
            while !child.is_null() {
                (*child).fr_parent = curfrp;
                child = (*child).fr_next;
            }
        } else {
            // Leaf frame — update oldwin's frame pointer
            set_win_frame(oldwin, new_frp);
        }
    }

    (curfrp, before)
}

/// Set w_frame for a window via its accessor.
unsafe fn set_win_frame(wp: WinHandle, frp: *mut Frame) {
    extern "C" {
        fn nvim_win_set_frame(wp: WinHandle, frp: *mut Frame);
    }
    nvim_win_set_frame(wp, frp);
}

/// Assign dimensions for a vertical split.
#[allow(clippy::too_many_arguments)]
unsafe fn assign_vertical_dimensions(
    wp: WinHandle,
    oldwin: WinHandle,
    curfrp: *mut Frame,
    frp: *mut Frame,
    flags: c_int,
    new_size: c_int,
    need_status: c_int,
    toplevel: bool,
    before: bool,
) {
    let curwin = nvim_get_curwin();
    nvim_win_set_p_scr(wp, nvim_win_get_p_scr(curwin));

    if need_status != 0 {
        rs_win_new_height(oldwin, nvim_win_get_w_height(oldwin) - 1);
        nvim_win_set_status_height(oldwin, need_status);
    }

    let p_ls = nvim_get_p_ls();
    if toplevel {
        nvim_win_set_winrow(wp, rs_tabline_height());
        rs_win_new_height(
            wp,
            (*curfrp).fr_height - c_int::from(p_ls == 1 || p_ls == 2),
        );
        nvim_win_set_status_height(wp, c_int::from(p_ls == 1 || p_ls == 2));
        nvim_win_set_hsep_height(wp, 0);
    } else {
        nvim_win_set_winrow(wp, nvim_win_get_winrow(oldwin));
        rs_win_new_height(wp, nvim_win_get_w_height(oldwin));
        nvim_win_set_status_height(wp, nvim_win_get_status_height(oldwin));
        nvim_win_set_hsep_height(wp, nvim_win_get_hsep_height(oldwin));
    }
    (*frp).fr_height = (*curfrp).fr_height;

    rs_win_new_width(wp, new_size);
    if before {
        nvim_win_set_vsep_width(wp, 1);
    } else {
        nvim_win_set_vsep_width(wp, nvim_win_get_vsep_width(oldwin));
        nvim_win_set_vsep_width(oldwin, 1);
    }

    if toplevel {
        if (flags & WSP_BOT) != 0 {
            rs_frame_set_vsep(curfrp, 1);
        }
        rs_frame_new_width(
            curfrp,
            (*curfrp).fr_width - (new_size + c_int::from((flags & WSP_TOP) != 0)),
            flags & WSP_TOP,
            0,
        );
    } else {
        rs_win_new_width(oldwin, nvim_win_get_w_width(oldwin) - (new_size + 1));
    }

    if before {
        nvim_win_set_wincol(wp, nvim_win_get_wincol(oldwin));
        nvim_win_set_wincol(oldwin, nvim_win_get_wincol(oldwin) + new_size + 1);
    } else {
        nvim_win_set_wincol(
            wp,
            nvim_win_get_wincol(oldwin) + nvim_win_get_w_width(oldwin) + 1,
        );
    }

    rs_frame_fix_width(oldwin);
    rs_frame_fix_width(wp);
}

/// Assign dimensions for a horizontal split.
#[allow(clippy::too_many_arguments)]
unsafe fn assign_horizontal_dimensions(
    wp: WinHandle,
    oldwin: WinHandle,
    curfrp: *mut Frame,
    frp: *mut Frame,
    flags: c_int,
    new_size: c_int,
    oldwin_height: c_int,
    toplevel: bool,
    before: bool,
) {
    let is_stl_global = rs_global_stl_height() > 0;

    if toplevel {
        nvim_win_set_wincol(wp, 0);
        rs_win_new_width(wp, nvim_get_Columns());
        nvim_win_set_vsep_width(wp, 0);
    } else {
        nvim_win_set_wincol(wp, nvim_win_get_wincol(oldwin));
        rs_win_new_width(wp, nvim_win_get_w_width(oldwin));
        nvim_win_set_vsep_width(wp, nvim_win_get_vsep_width(oldwin));
    }
    (*frp).fr_width = (*curfrp).fr_width;

    rs_win_new_height(wp, new_size);
    let old_status_height = nvim_win_get_status_height(oldwin);
    if before {
        nvim_win_set_hsep_height(wp, c_int::from(is_stl_global));
    } else {
        nvim_win_set_hsep_height(wp, nvim_win_get_hsep_height(oldwin));
        nvim_win_set_hsep_height(oldwin, c_int::from(is_stl_global));
    }

    if toplevel {
        let mut new_fr_height = (*curfrp).fr_height - new_size;
        if is_stl_global {
            if (flags & WSP_BOT) != 0 {
                rs_frame_add_hsep(curfrp);
            } else {
                new_fr_height -= 1;
            }
        } else {
            let p_ls = nvim_get_p_ls();
            if !((flags & WSP_BOT) != 0 && p_ls == 0) {
                new_fr_height -= STATUS_HEIGHT;
            }
            if (flags & WSP_BOT) != 0 {
                rs_frame_add_statusline(curfrp);
            }
        }
        rs_frame_new_height(curfrp, new_fr_height, flags & WSP_TOP, 0, 0);
    } else {
        rs_win_new_height(oldwin, oldwin_height - (new_size + STATUS_HEIGHT));
    }

    if before {
        // new window above current one
        nvim_win_set_winrow(wp, nvim_win_get_winrow(oldwin));
        if is_stl_global {
            nvim_win_set_status_height(wp, 0);
            nvim_win_set_winrow(
                oldwin,
                nvim_win_get_winrow(oldwin) + nvim_win_get_w_height(wp) + 1,
            );
        } else {
            nvim_win_set_status_height(wp, STATUS_HEIGHT);
            nvim_win_set_winrow(
                oldwin,
                nvim_win_get_winrow(oldwin) + nvim_win_get_w_height(wp) + STATUS_HEIGHT,
            );
        }
    } else {
        // new window below current one
        if is_stl_global {
            nvim_win_set_winrow(
                wp,
                nvim_win_get_winrow(oldwin) + nvim_win_get_w_height(oldwin) + 1,
            );
            nvim_win_set_status_height(wp, 0);
        } else {
            nvim_win_set_winrow(
                wp,
                nvim_win_get_winrow(oldwin) + nvim_win_get_w_height(oldwin) + STATUS_HEIGHT,
            );
            nvim_win_set_status_height(wp, old_status_height);
            if (flags & WSP_BOT) == 0 {
                nvim_win_set_status_height(oldwin, STATUS_HEIGHT);
            }
        }
    }

    rs_frame_fix_height(wp);
    rs_frame_fix_height(oldwin);
}

// =============================================================================
// FFI export
// =============================================================================

/// FFI: Main entry point called from C wrapper.
///
/// # Safety
/// Caller must ensure all pointer arguments are valid or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_split_ins(
    size: c_int,
    flags: c_int,
    new_wp: WinHandle,
    dir: c_int,
    to_flatten: *mut Frame,
) -> SplitInsResult {
    win_split_ins_impl(size, flags, new_wp, dir, to_flatten)
}

extern "C" {
    fn rs_win_enter_ext(wp: WinHandle, flags: c_int);
}

/// Internal: full win_split_ins with post-processing.
///
/// Computes oldwin, calls the core implementation, then:
/// - calls `rs_win_enter_ext` if needed,
/// - restores `p_wiw` or `p_wh`,
/// - sets `oldwin->w_pos_changed`.
///
/// Returns the new window handle, or null on failure.
///
/// # Safety
/// Caller must ensure all pointer arguments are valid or null.
pub(crate) unsafe fn win_split_ins_full_impl(
    size: c_int,
    flags: c_int,
    new_wp: WinHandle,
    dir: c_int,
    to_flatten: *mut Frame,
) -> WinHandle {
    let oldwin = get_oldwin(flags);
    let res = win_split_ins_impl(size, flags, new_wp, dir, to_flatten);
    if res.wp.is_null() {
        return WinHandle::null();
    }

    if res.do_enter != 0 {
        rs_win_enter_ext(res.wp, res.enter_flags);
    }

    // Restore p_wiw or p_wh.
    if res.vertical != 0 {
        nvim_set_p_wiw(i64::from(res.saved_option));
    } else {
        nvim_set_p_wh(i64::from(res.saved_option));
    }

    if rs_win_valid(oldwin) != 0 {
        nvim_win_set_pos_changed(oldwin, 1);
    }

    res.wp
}

/// FFI: Full win_split_ins entry point — absorbs post-processing from C wrapper.
///
/// # Safety
/// Caller must ensure all pointer arguments are valid or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_split_ins_full(
    size: c_int,
    flags: c_int,
    new_wp: WinHandle,
    dir: c_int,
    to_flatten: *mut Frame,
) -> WinHandle {
    win_split_ins_full_impl(size, flags, new_wp, dir, to_flatten)
}

/// C export: `win_split_ins` — eliminates the C thin wrapper.
///
/// # Safety
/// Caller must ensure all pointer arguments are valid or null.
#[unsafe(export_name = "win_split_ins")]
pub unsafe extern "C" fn win_split_ins(
    size: c_int,
    flags: c_int,
    new_wp: WinHandle,
    dir: c_int,
    to_flatten: *mut Frame,
) -> WinHandle {
    win_split_ins_full_impl(size, flags, new_wp, dir, to_flatten)
}

// =============================================================================
// Phase 2: win_split and win_splitmove orchestration
// =============================================================================

// Additional external C functions needed for Phase 2 that are not already
// declared in the extern block above.
extern "C" {
    /// nvim_may_open_tabpage: opens a new tab page when :tab modifier was used.
    #[link_name = "rs_may_open_tabpage"]
    fn nvim_may_open_tabpage() -> c_int;

    /// nvim_get_cmdmod_split: get cmdmod.cmod_split flags.
    fn nvim_get_cmdmod_split() -> c_int;

    /// nvim_emsg_id: generic error message dispatcher.
    fn nvim_emsg_id(id: c_int);

    /// nvim_get_curtab: get current tabpage.
    fn nvim_get_curtab() -> TabpageHandle;

    /// rs_check_split_disallowed: check split_disallowed counter and buffer lock.
    fn rs_check_split_disallowed(wp: WinHandle) -> c_int;

    /// rs_make_snapshot: create snapshot for help window.
    fn rs_make_snapshot(idx: c_int);

    /// rs_clear_snapshot: clear snapshot.
    fn rs_clear_snapshot(tp: TabpageHandle, idx: c_int);

    /// rs_winframe_remove: remove window from frame tree.
    fn rs_winframe_remove(
        win: WinHandle,
        dirp: *mut c_int,
        tp: TabpageHandle,
        unflat_altfr: *mut *mut Frame,
    ) -> WinHandle;

    /// rs_winframe_restore: undo winframe_remove.
    fn rs_winframe_restore(wp: WinHandle, dir: c_int, unflat_altfr: *mut Frame);

    /// rs_win_remove: remove window from window list.
    fn rs_win_remove(wp: WinHandle, tp: TabpageHandle);

    /// rs_last_status: add/remove last status line.
    fn rs_last_status(morewin: c_int);

    /// rs_win_valid: check if window is valid.
    fn rs_win_valid(wp: WinHandle) -> c_int;

    /// nvim_win_get_floating_win: get w_floating from a win_T* (avoids conflict with nvim_win_get_floating).
    fn nvim_win_get_floating_win(wp: WinHandle) -> c_int;
}

// rs_one_window_in_tab is already declared as nvim_one_window_firstwin (link_name) above.
// rs_win_append, rs_win_comp_pos, rs_win_equal, nvim_get_p_ea are in the existing extern block.
// nvim_win_get_w_height, nvim_win_get_prev, nvim_win_get_floating, nvim_is_aucmd_win are
// declared in the existing extern block.
// rs_win_setheight_win is nvim_win_setheight_win_wrapper (link_name) in existing block.

/// SNAP_HELP_IDX constant.
const SNAP_HELP_IDX: c_int = 0;

/// OK/FAIL constants.
const OK: c_int = 1;
const FAIL: c_int = 0;

/// win_split implementation.
///
/// Implements `:split`, `:vsplit`, CTRL-W s, etc.
/// Returns OK (1) or FAIL (0).
///
/// # Safety
/// Calls C accessor functions.
unsafe fn win_split_impl(size: c_int, flags: c_int) -> c_int {
    let curwin = nvim_get_curwin();

    // Check if split is allowed.
    if rs_check_split_disallowed(curwin) == 0 {
        return FAIL;
    }

    // When the :tab modifier was used, open a new tab page instead.
    if nvim_may_open_tabpage() == OK {
        return OK;
    }

    // Add flags from :vertical, :topleft, :botright.
    let flags = flags | nvim_get_cmdmod_split();
    if (flags & WSP_TOP) != 0 && (flags & WSP_BOT) != 0 {
        nvim_emsg_id(EMSG_E442);
        return FAIL;
    }

    // When creating the help window, make a snapshot of the window layout.
    // Otherwise clear the snapshot — it's now invalid.
    if (flags & WSP_HELP) != 0 {
        rs_make_snapshot(SNAP_HELP_IDX);
    } else {
        rs_clear_snapshot(nvim_get_curtab(), SNAP_HELP_IDX);
    }

    if win_split_ins_full_impl(size, flags, WinHandle::null(), 0, std::ptr::null_mut()).is_null() {
        FAIL
    } else {
        OK
    }
}

/// win_splitmove implementation.
///
/// Moves window `wp` into a new split position.
/// Returns OK (1) or FAIL (0).
///
/// # Safety
/// Calls C accessor functions.
unsafe fn win_splitmove_impl(wp: WinHandle, size: c_int, flags: c_int) -> c_int {
    let mut dir: c_int = 0;
    let height = nvim_win_get_w_height(wp);

    // Nothing to do if wp is the only window.
    if nvim_one_window_firstwin(wp, TabpageHandle::null()) != 0 {
        return OK;
    }

    // Validate: not aucmd_win and split is allowed.
    if nvim_is_aucmd_win(wp) != 0 || rs_check_split_disallowed(wp) == 0 {
        return FAIL;
    }

    let mut unflat_altfr: *mut Frame = std::ptr::null_mut();
    if nvim_win_get_floating_win(wp) != 0 {
        rs_win_remove(wp, TabpageHandle::null());
    } else {
        // Remove the window and frame from the tree of frames. Don't flatten
        // any frames yet so we can restore things if win_split_ins fails.
        rs_winframe_remove(
            wp,
            std::ptr::addr_of_mut!(dir),
            TabpageHandle::null(),
            std::ptr::addr_of_mut!(unflat_altfr),
        );
        rs_win_remove(wp, TabpageHandle::null());
        rs_last_status(0); // may need to remove last status line
        rs_win_comp_pos(); // recompute window positions
    }

    // Split on the desired side and put wp there.
    if win_split_ins_full_impl(size, flags, wp, dir, unflat_altfr).is_null() {
        if nvim_win_get_floating_win(wp) == 0 {
            // win_split_ins doesn't change sizes or layout if it fails to insert
            // an existing window, so just undo winframe_remove.
            rs_winframe_restore(wp, dir, unflat_altfr);
        }
        rs_win_append(nvim_win_get_prev(wp), wp, WinHandle::null());
        return FAIL;
    }

    // If splitting horizontally, try to preserve height.
    // Note: win_split_ins autocommands may have closed wp or made it floating!
    if size == 0
        && (flags & WSP_VERT) == 0
        && rs_win_valid(wp) != 0
        && nvim_win_get_floating_win(wp) == 0
    {
        nvim_win_setheight_win_wrapper(height, wp);
        if nvim_get_p_ea() != 0 {
            // Equalize windows.
            let curwin = nvim_get_curwin();
            rs_win_equal(curwin, c_int::from(curwin == wp), i32::from(b'v'));
        }
    }

    OK
}

/// FFI: win_split — top-level split function.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_split(size: c_int, flags: c_int) -> c_int {
    win_split_impl(size, flags)
}

/// C export: `win_split` — eliminates the C thin wrapper.
///
/// # Safety
/// Calls C accessor functions.
#[must_use]
#[unsafe(export_name = "win_split")]
pub unsafe extern "C" fn win_split(size: c_int, flags: c_int) -> c_int {
    win_split_impl(size, flags)
}

/// FFI: win_splitmove — move window into a new split position.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_splitmove(wp: WinHandle, size: c_int, flags: c_int) -> c_int {
    win_splitmove_impl(wp, size, flags)
}

/// C export: `win_splitmove` — eliminates the C thin wrapper.
///
/// # Safety
/// Calls C accessor functions.
#[must_use]
#[unsafe(export_name = "win_splitmove")]
pub unsafe extern "C" fn win_splitmove(wp: WinHandle, size: c_int, flags: c_int) -> c_int {
    win_splitmove_impl(wp, size, flags)
}
