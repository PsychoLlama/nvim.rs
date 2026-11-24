//! `Win`'s field getters and setters: one per `Copy` field, and one per
//! `Copy` window-local option of the window's own set (`w_onebuf_opt`), each
//! checked against the window's slot as `slot_fields!` says.
#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `winlayer/` row in docs/perimeter.md.
#![allow(unsafe_code)]

use core::ffi::{c_int, c_uint};

use crate::arglist::ArgListId;
use crate::grid::defs::GridView;
use crate::r#move::WinValid;
use crate::os::libc::{uint8_t, uint32_t};
use crate::quickfix::model::QfId;
use crate::slot;
use crate::types::*;
use crate::winlayer::{Buf, FrameId, Win, WinId};

slot_fields! {
    pub Win => Window {
        w_buffer, set_w_buffer: Buf;
        w_s, set_w_s: *mut SynBlock;
        w_ns_hl, set_w_ns_hl: NS;
        w_ns_hl_winhl, set_w_ns_hl_winhl: c_int;
        w_ns_hl_active, set_w_ns_hl_active: NS;
        w_ns_hl_attr, set_w_ns_hl_attr: Option<&'static crate::highlight::state::HlAttrTable>;
        w_hl_id_normal, set_w_hl_id_normal: c_int;
        w_hl_attr_normal, set_w_hl_attr_normal: c_int;
        w_hl_attr_normalnc, set_w_hl_attr_normalnc: c_int;
        w_hl_needs_update, set_w_hl_needs_update: bool;
        w_locked, set_w_locked: bool;
        w_cursor, set_w_cursor: Pos;
        w_curswant, set_w_curswant: ColNr;
        w_set_curswant, set_w_set_curswant: bool;
        w_cursorline, set_w_cursorline: LineNr;
        w_last_cursorline, set_w_last_cursorline: LineNr;
        w_old_visual_mode, set_w_old_visual_mode: c_int;
        w_old_cursor_lnum, set_w_old_cursor_lnum: LineNr;
        w_old_cursor_fcol, set_w_old_cursor_fcol: ColNr;
        w_old_cursor_lcol, set_w_old_cursor_lcol: ColNr;
        w_old_visual_lnum, set_w_old_visual_lnum: LineNr;
        w_old_visual_col, set_w_old_visual_col: ColNr;
        w_old_curswant, set_w_old_curswant: ColNr;
        w_last_cursor_lnum_rnu, set_w_last_cursor_lnum_rnu: LineNr;
        w_topline, set_w_topline: LineNr;
        w_topline_was_set, set_w_topline_was_set: bool;
        w_topfill, set_w_topfill: c_int;
        w_old_topfill, set_w_old_topfill: c_int;
        w_botfill, set_w_botfill: bool;
        w_old_botfill, set_w_old_botfill: bool;
        w_leftcol, set_w_leftcol: ColNr;
        w_skipcol, set_w_skipcol: ColNr;
        w_last_topline, set_w_last_topline: LineNr;
        w_last_topfill, set_w_last_topfill: c_int;
        w_last_leftcol, set_w_last_leftcol: ColNr;
        w_last_skipcol, set_w_last_skipcol: ColNr;
        w_last_width, set_w_last_width: c_int;
        w_last_height, set_w_last_height: c_int;
        w_winrow, set_w_winrow: c_int;
        w_height, set_w_height: c_int;
        w_prev_winrow, set_w_prev_winrow: c_int;
        w_prev_height, set_w_prev_height: c_int;
        w_status_height, set_w_status_height: c_int;
        w_winbar_height, set_w_winbar_height: c_int;
        w_wincol, set_w_wincol: c_int;
        w_width, set_w_width: c_int;
        w_hsep_height, set_w_hsep_height: c_int;
        w_vsep_width, set_w_vsep_width: c_int;
        w_do_win_fix_cursor, set_w_do_win_fix_cursor: bool;
        w_winrow_off, set_w_winrow_off: c_int;
        w_wincol_off, set_w_wincol_off: c_int;
        w_view_height, set_w_view_height: c_int;
        w_view_width, set_w_view_width: c_int;
        w_height_request, set_w_height_request: c_int;
        w_width_request, set_w_width_request: c_int;
        w_border_adj, set_w_border_adj: [c_int; 4];
        w_height_outer, set_w_height_outer: c_int;
        w_width_outer, set_w_width_outer: c_int;
        w_valid, set_w_valid: WinValid;
        w_valid_cursor, set_w_valid_cursor: Pos;
        w_valid_leftcol, set_w_valid_leftcol: ColNr;
        w_valid_skipcol, set_w_valid_skipcol: ColNr;
        w_viewport_invalid, set_w_viewport_invalid: bool;
        w_viewport_last_topline, set_w_viewport_last_topline: LineNr;
        w_viewport_last_botline, set_w_viewport_last_botline: LineNr;
        w_viewport_last_topfill, set_w_viewport_last_topfill: i32;
        w_viewport_last_skipcol, set_w_viewport_last_skipcol: ColNr;
        w_cline_height, set_w_cline_height: c_int;
        w_cline_folded, set_w_cline_folded: bool;
        w_cline_row, set_w_cline_row: c_int;
        w_virtcol, set_w_virtcol: ColNr;
        w_wrow, set_w_wrow: c_int;
        w_wcol, set_w_wcol: c_int;
        w_botline, set_w_botline: LineNr;
        w_empty_rows, set_w_empty_rows: c_int;
        w_filler_rows, set_w_filler_rows: c_int;
        w_lines_valid, set_w_lines_valid: c_int;
        w_fold_manual, set_w_fold_manual: bool;
        w_foldinvalid, set_w_foldinvalid: bool;
        w_nrwidth, set_w_nrwidth: c_int;
        w_scwidth, set_w_scwidth: c_int;
        w_minscwidth, set_w_minscwidth: c_int;
        w_maxscwidth, set_w_maxscwidth: c_int;
        w_redr_type, set_w_redr_type: c_int;
        w_upd_rows, set_w_upd_rows: c_int;
        w_redraw_top, set_w_redraw_top: LineNr;
        w_redraw_bot, set_w_redraw_bot: LineNr;
        w_redr_status, set_w_redr_status: bool;
        w_redr_border, set_w_redr_border: bool;
        w_redr_statuscol, set_w_redr_statuscol: bool;
        w_display_tick, set_w_display_tick: DispTick;
        w_stl_cursor, set_w_stl_cursor: Pos;
        w_stl_virtcol, set_w_stl_virtcol: ColNr;
        w_stl_topline, set_w_stl_topline: LineNr;
        w_stl_line_count, set_w_stl_line_count: LineNr;
        w_stl_topfill, set_w_stl_topfill: c_int;
        w_stl_empty, set_w_stl_empty: bool;
        w_stl_recording, set_w_stl_recording: u8;
        w_stl_state, set_w_stl_state: c_int;
        w_stl_visual_mode, set_w_stl_visual_mode: c_int;
        w_stl_visual_pos, set_w_stl_visual_pos: Pos;
        w_alt_fnum, set_w_alt_fnum: Option<Handle>;
        w_alist, set_w_alist: ArgListId;
        w_arg_idx, set_w_arg_idx: c_int;
        w_arg_idx_invalid, set_w_arg_idx_invalid: bool;
        w_p_culopt_flags, set_w_p_culopt_flags: uint8_t;
        w_briopt_min, set_w_briopt_min: c_int;
        w_briopt_shift, set_w_briopt_shift: c_int;
        w_briopt_sbr, set_w_briopt_sbr: bool;
        w_briopt_list, set_w_briopt_list: c_int;
        w_briopt_vcol, set_w_briopt_vcol: c_int;
        w_scbind_pos, set_w_scbind_pos: c_int;
        w_pcmark, set_w_pcmark: Pos;
        w_prev_pcmark, set_w_prev_pcmark: Pos;
        w_jumplistlen, set_w_jumplistlen: c_int;
        w_jumplistidx, set_w_jumplistidx: c_int;
        w_changelistidx, set_w_changelistidx: c_int;
        w_next_match_id, set_w_next_match_id: c_int;
        w_tagstackidx, set_w_tagstackidx: c_int;
        w_tagstacklen, set_w_tagstacklen: c_int;
        w_grid, set_w_grid: GridView;
        w_pos_changed, set_w_pos_changed: bool;
        w_floating, set_w_floating: bool;
        w_float_is_info, set_w_float_is_info: bool;
        w_fraction, set_w_fraction: c_int;
        w_prev_fraction_row, set_w_prev_fraction_row: c_int;
        w_nrwidth_line_count, set_w_nrwidth_line_count: LineNr;
        w_statuscol_line_count, set_w_statuscol_line_count: LineNr;
        w_nrwidth_width, set_w_nrwidth_width: c_int;
        // '*' window-local options: the window's own values (`w_onebuf_opt`).
        wo_arab, set_wo_arab: c_int = w_onebuf_opt.wo_arab;
        wo_bri, set_wo_bri: c_int = w_onebuf_opt.wo_bri;
        wo_diff, set_wo_diff: c_int = w_onebuf_opt.wo_diff;
        wo_fen, set_wo_fen: c_int = w_onebuf_opt.wo_fen;
        wo_fen_save, set_wo_fen_save: c_int = w_onebuf_opt.wo_fen_save;
        wo_fdl, set_wo_fdl: OptInt = w_onebuf_opt.wo_fdl;
        wo_fdl_save, set_wo_fdl_save: OptInt = w_onebuf_opt.wo_fdl_save;
        wo_fml, set_wo_fml: OptInt = w_onebuf_opt.wo_fml;
        wo_fdn, set_wo_fdn: OptInt = w_onebuf_opt.wo_fdn;
        wo_lbr, set_wo_lbr: c_int = w_onebuf_opt.wo_lbr;
        wo_list, set_wo_list: c_int = w_onebuf_opt.wo_list;
        wo_nu, set_wo_nu: c_int = w_onebuf_opt.wo_nu;
        wo_rnu, set_wo_rnu: c_int = w_onebuf_opt.wo_rnu;
        wo_ve_flags, set_wo_ve_flags: c_uint = w_onebuf_opt.wo_ve_flags;
        wo_nuw, set_wo_nuw: OptInt = w_onebuf_opt.wo_nuw;
        wo_wfb, set_wo_wfb: c_int = w_onebuf_opt.wo_wfb;
        wo_wfh, set_wo_wfh: c_int = w_onebuf_opt.wo_wfh;
        wo_wfw, set_wo_wfw: c_int = w_onebuf_opt.wo_wfw;
        wo_pvw, set_wo_pvw: c_int = w_onebuf_opt.wo_pvw;
        wo_lhi, set_wo_lhi: OptInt = w_onebuf_opt.wo_lhi;
        wo_rl, set_wo_rl: c_int = w_onebuf_opt.wo_rl;
        wo_scr, set_wo_scr: OptInt = w_onebuf_opt.wo_scr;
        wo_sms, set_wo_sms: c_int = w_onebuf_opt.wo_sms;
        wo_spell, set_wo_spell: c_int = w_onebuf_opt.wo_spell;
        wo_cuc, set_wo_cuc: c_int = w_onebuf_opt.wo_cuc;
        wo_cul, set_wo_cul: c_int = w_onebuf_opt.wo_cul;
        wo_scb, set_wo_scb: c_int = w_onebuf_opt.wo_scb;
        wo_diff_saved, set_wo_diff_saved: c_int = w_onebuf_opt.wo_diff_saved;
        wo_scb_save, set_wo_scb_save: c_int = w_onebuf_opt.wo_scb_save;
        wo_wrap, set_wo_wrap: c_int = w_onebuf_opt.wo_wrap;
        wo_wrap_save, set_wo_wrap_save: c_int = w_onebuf_opt.wo_wrap_save;
        wo_cole, set_wo_cole: OptInt = w_onebuf_opt.wo_cole;
        wo_crb, set_wo_crb: c_int = w_onebuf_opt.wo_crb;
        wo_crb_save, set_wo_crb_save: c_int = w_onebuf_opt.wo_crb_save;
        wo_siso, set_wo_siso: OptInt = w_onebuf_opt.wo_siso;
        wo_so, set_wo_so: OptInt = w_onebuf_opt.wo_so;
        wo_winbl, set_wo_winbl: OptInt = w_onebuf_opt.wo_winbl;
        wo_wrap_flags, set_wo_wrap_flags: uint32_t = w_onebuf_opt.wo_wrap_flags;
        wo_stl_flags, set_wo_stl_flags: uint32_t = w_onebuf_opt.wo_stl_flags;
        wo_wbr_flags, set_wo_wbr_flags: uint32_t = w_onebuf_opt.wo_wbr_flags;
        wo_fde_flags, set_wo_fde_flags: uint32_t = w_onebuf_opt.wo_fde_flags;
        wo_fdt_flags, set_wo_fdt_flags: uint32_t = w_onebuf_opt.wo_fdt_flags;
    }
}

slot_fields! {
    pub(crate) Win => Window {
        w_prev, set_w_prev: Option<WinId>;
        w_next, set_w_next: Option<WinId>;
        w_frame, set_w_frame: Option<FrameId>;
        w_llist, set_w_llist: Option<QfId>;
        w_llist_ref, set_w_llist_ref: Option<QfId>;
    }
}

// Leaves of the fields that are not `Copy` themselves: a float's
// configuration, the saved view, and the 'listchars'/'fillchars' glyphs.
slot_fields! {
    pub Win => Window {
        config_hide, set_config_hide: bool = w_config.hide;
        config_focusable: bool = w_config.focusable;
        config_mouse: bool = w_config.mouse;
        config_external: bool = w_config.external;
        config_border: bool = w_config.border;
        config_shadow, set_config_shadow: bool = w_config.shadow;
        config_zindex: c_int = w_config.zindex;
        config_width, set_config_width: c_int = w_config.width;
        config_height, set_config_height: c_int = w_config.height;
        config_row, set_config_row: f64 = w_config.row;
        config_col, set_config_col: f64 = w_config.col;
        config_anchor, set_config_anchor: FloatAnchor = w_config.anchor;
        config_style: WinStyle = w_config.style;
        config_relative: FloatRelative = w_config.relative;
        config_window: WindowHandle = w_config.window;
        config_bufpos: LPos = w_config.bufpos;
        w_topline_save, set_w_topline_save: c_int = w_save_cursor.w_topline_save;
        w_topline_corr, set_w_topline_corr: c_int = w_save_cursor.w_topline_corr;
        w_cursor_save, set_w_cursor_save: Pos = w_save_cursor.w_cursor_save;
        w_cursor_corr, set_w_cursor_corr: Pos = w_save_cursor.w_cursor_corr;
        lcs_eol: ScreenChar = w_p_lcs_chars.eol;
        lcs_ext: ScreenChar = w_p_lcs_chars.ext;
        lcs_prec: ScreenChar = w_p_lcs_chars.prec;
        lcs_nbsp: ScreenChar = w_p_lcs_chars.nbsp;
        lcs_space: ScreenChar = w_p_lcs_chars.space;
        lcs_tab1: ScreenChar = w_p_lcs_chars.tab1;
        lcs_tab2: ScreenChar = w_p_lcs_chars.tab2;
        lcs_tab3: ScreenChar = w_p_lcs_chars.tab3;
        lcs_leadtab1: ScreenChar = w_p_lcs_chars.leadtab1;
        lcs_leadtab2: ScreenChar = w_p_lcs_chars.leadtab2;
        lcs_leadtab3: ScreenChar = w_p_lcs_chars.leadtab3;
        lcs_lead: ScreenChar = w_p_lcs_chars.lead;
        lcs_trail: ScreenChar = w_p_lcs_chars.trail;
        lcs_conceal: ScreenChar = w_p_lcs_chars.conceal;
        fcs_stl: ScreenChar = w_p_fcs_chars.stl;
        fcs_stlnc: ScreenChar = w_p_fcs_chars.stlnc;
        fcs_wbr: ScreenChar = w_p_fcs_chars.wbr;
        fcs_horiz: ScreenChar = w_p_fcs_chars.horiz;
        fcs_horizup: ScreenChar = w_p_fcs_chars.horizup;
        fcs_horizdown: ScreenChar = w_p_fcs_chars.horizdown;
        fcs_vert: ScreenChar = w_p_fcs_chars.vert;
        fcs_vertleft: ScreenChar = w_p_fcs_chars.vertleft;
        fcs_vertright: ScreenChar = w_p_fcs_chars.vertright;
        fcs_verthoriz: ScreenChar = w_p_fcs_chars.verthoriz;
        fcs_fold: ScreenChar = w_p_fcs_chars.fold;
        fcs_foldopen: ScreenChar = w_p_fcs_chars.foldopen;
        fcs_foldclosed: ScreenChar = w_p_fcs_chars.foldclosed;
        fcs_foldsep: ScreenChar = w_p_fcs_chars.foldsep;
        fcs_foldinner: ScreenChar = w_p_fcs_chars.foldinner;
        fcs_diff: ScreenChar = w_p_fcs_chars.diff;
        fcs_msgsep: ScreenChar = w_p_fcs_chars.msgsep;
        fcs_eob: ScreenChar = w_p_fcs_chars.eob;
        fcs_lastline: ScreenChar = w_p_fcs_chars.lastline;
        fcs_trunc: ScreenChar = w_p_fcs_chars.trunc;
        fcs_truncrl: ScreenChar = w_p_fcs_chars.truncrl;
    }
}

impl Win {
    /// A float's configuration, copied out.
    #[inline]
    #[track_caller]
    pub(crate) fn config(self) -> WinConfig {
        self.read().w_config.clone()
    }

    /// The screen line entry `i` (`w_lines[i]`), copied out.
    #[inline]
    #[track_caller]
    pub(crate) fn wline(self, i: usize) -> WLine {
        self.read().w_lines[i].clone()
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use crate::global_cell::editor_state_lock;
    use crate::types::LineNr;
    use crate::window::{bare_window, free_bare_window};

    /// A getter, a setter and a nested leaf reach the field in place; under
    /// a `read()` the setter refuses and the getter answers, under an
    /// `edit()` both refuse; once freed, both refuse.
    #[test]
    fn a_window_field_is_read_and_written_in_place_and_checked() {
        let _held = editor_state_lock();
        let win = bare_window();
        win.set_w_topline(LineNr::new(3));
        win.set_wo_wrap(1);
        win.set_config_width(7);
        assert_eq!(win.w_topline(), LineNr::new(3));
        assert_eq!((win.wo_wrap(), win.config_width()), (1, 7));
        let read = win.read();
        assert!(catch_unwind(AssertUnwindSafe(|| win.set_w_topline(LineNr::ONE))).is_err());
        assert_eq!(win.w_topline(), LineNr::new(3));
        drop(read);
        let edit = win.edit();
        assert!(catch_unwind(AssertUnwindSafe(|| win.wo_wrap())).is_err());
        drop(edit);
        free_bare_window(win);
        assert!(catch_unwind(AssertUnwindSafe(|| win.w_topline())).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| win.set_wo_wrap(0))).is_err());
    }
}
