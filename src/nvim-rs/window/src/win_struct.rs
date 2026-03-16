//! Repr(C) mirror of `win_T` from `buffer_defs.h`.
//!
//! This struct provides direct field access to `win_T` from Rust, eliminating
//! the need for C accessor functions. Layout validated by `_Static_assert`
//! checks in `src/nvim/window_struct_check.c`.
//!
//! # Safety
//! This struct MUST match the C `win_T` layout exactly. All offsets are
//! validated at compile time via C static assertions.

#![allow(dead_code)]
#![allow(clippy::struct_field_names)]

use std::ffi::{c_char, c_int, c_void};

use crate::{Frame, WinHandle};

/// Neovim `linenr_T` (int32_t)
pub type LineNr = i32;
/// Neovim `colnr_T` (int/i32)
pub type ColNr = i32;
/// Neovim `handle_T` (int)
pub type HandleT = c_int;
/// Neovim `OptInt` (int64_t)
pub type OptInt = i64;

/// Mirror of C `pos_T` from `pos_defs.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PosT {
    pub lnum: LineNr,
    pub col: ColNr,
    pub coladd: ColNr,
}

/// Mirror of C `pos_save_T` from `buffer_defs.h`.
/// Contains saved cursor and topline for check_lnums/reset_lnums.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PosSaveT {
    /// w_topline_save
    pub w_topline_save: LineNr,
    /// w_topline_corr
    pub w_topline_corr: LineNr,
    /// w_cursor_save (pos_T = 12 bytes)
    pub w_cursor_save: PosT,
    /// w_cursor_corr (pos_T = 12 bytes)
    pub w_cursor_corr: PosT,
}

/// Repr(C) mirror of C `win_T` (`struct window_S`).
///
/// Fields are laid out exactly as in `buffer_defs.h`.
/// Complex nested types that are not accessed directly from Rust use
/// opaque `[u8; N]` padding preserving correct offsets.
///
/// All field offsets are validated by `_Static_assert` in
/// `src/nvim/window_struct_check.c`.
#[repr(C)]
pub struct WinStruct {
    // offset 0
    pub handle: HandleT,
    _pad0: [u8; 4], // alignment padding to reach offset 8

    // offset 8
    pub w_buffer: *mut c_void, // buf_T*

    // offset 16
    pub w_s: *mut c_void, // synblock_T*

    // offset 24
    pub w_ns_hl: c_int,
    // offset 28
    pub w_ns_hl_winhl: c_int,
    // offset 32
    pub w_ns_hl_active: c_int,
    _pad1: [u8; 4], // alignment for pointer at 40

    // offset 40
    pub w_ns_hl_attr: *mut c_int,

    // offset 48: Set(uint32_t) w_ns_set = { MapHash h; uint32_t* keys }
    // MapHash = 6*u32 + ptr = 32 bytes; keys ptr = 8 bytes -> total 40
    _ns_set: [u8; 40],

    // offset 88
    pub w_hl_id_normal: c_int,
    // offset 92
    pub w_hl_attr_normal: c_int,
    // offset 96
    pub w_hl_attr_normalnc: c_int,
    // offset 100 - C type is 'int w_hl_needs_update' (not bool!)
    pub w_hl_needs_update: c_int,

    // offset 104
    pub w_prev: WinHandle, // win_T*
    // offset 112
    pub w_next: WinHandle, // win_T*

    // offset 120 - bool w_locked
    pub w_locked: bool,
    _pad2: [u8; 7], // to reach offset 128

    // offset 128
    pub w_frame: *mut Frame, // frame_T*

    // offset 136: pos_T w_cursor (12 bytes)
    pub w_cursor: PosT,

    // offset 148
    pub w_curswant: ColNr,
    // offset 152
    pub w_set_curswant: c_int,
    // offset 156
    pub w_cursorline: LineNr,
    // offset 160
    pub w_last_cursorline: LineNr,

    // offset 164 - char w_old_visual_mode
    pub w_old_visual_mode: c_char,
    _pad3: [u8; 3], // to reach 168

    // offset 168
    pub w_old_cursor_lnum: LineNr,
    // offset 172
    pub w_old_cursor_fcol: ColNr,
    // offset 176
    pub w_old_cursor_lcol: ColNr,
    // offset 180
    pub w_old_visual_lnum: LineNr,
    // offset 184
    pub w_old_visual_col: ColNr,
    // offset 188
    pub w_old_curswant: ColNr,
    // offset 192
    pub w_last_cursor_lnum_rnu: LineNr,
    _pad4: [u8; 4], // to reach 200

    // offset 200: lcs_chars_T w_p_lcs_chars (64 bytes)
    pub w_p_lcs_chars: [u8; 64],

    // offset 264: fcs_chars_T w_p_fcs_chars (84 bytes)
    pub w_p_fcs_chars: [u8; 84],

    // offset 348
    pub w_topline: LineNr,
    // offset 352 - char w_topline_was_set
    pub w_topline_was_set: u8,
    _pad6: [u8; 3], // to reach 356

    // offset 356
    pub w_topfill: c_int,
    // offset 360
    pub w_old_topfill: c_int,
    // offset 364 - bool w_botfill
    pub w_botfill: bool,
    // offset 365 - bool w_old_botfill
    pub w_old_botfill: bool,
    _pad7: [u8; 2], // to reach 368

    // offset 368
    pub w_leftcol: ColNr,
    // offset 372
    pub w_skipcol: ColNr,

    // offset 376
    pub w_last_topline: LineNr,
    // offset 380
    pub w_last_topfill: c_int,
    // offset 384
    pub w_last_leftcol: ColNr,
    // offset 388
    pub w_last_skipcol: ColNr,
    // offset 392
    pub w_last_width: c_int,
    // offset 396
    pub w_last_height: c_int,

    // offset 400
    pub w_winrow: c_int,
    // offset 404
    pub w_height: c_int,
    // offset 408
    pub w_prev_winrow: c_int,
    // offset 412
    pub w_prev_height: c_int,
    // offset 416
    pub w_status_height: c_int,
    // offset 420
    pub w_winbar_height: c_int,
    // offset 424
    pub w_wincol: c_int,
    // offset 428
    pub w_width: c_int,
    // offset 432
    pub w_hsep_height: c_int,
    // offset 436
    pub w_vsep_width: c_int,

    // offset 440: pos_save_T w_save_cursor (32 bytes)
    pub w_save_cursor: PosSaveT,

    // offset 472 - bool w_do_win_fix_cursor
    pub w_do_win_fix_cursor: bool,
    _pad8: [u8; 3], // to reach 476

    // offset 476
    pub w_winrow_off: c_int,
    // offset 480
    pub w_wincol_off: c_int,
    // offset 484
    pub w_view_height: c_int,
    // offset 488
    pub w_view_width: c_int,
    // offset 492
    pub w_height_request: c_int,
    // offset 496
    pub w_width_request: c_int,

    // offset 500: int w_border_adj[4]
    pub w_border_adj: [c_int; 4],

    // offset 516
    pub w_height_outer: c_int,
    // offset 520
    pub w_width_outer: c_int,

    // offset 524
    pub w_valid: c_int,
    // offset 528: pos_T w_valid_cursor (12 bytes)
    pub w_valid_cursor: PosT,
    // offset 540
    pub w_valid_leftcol: ColNr,
    // offset 544
    pub w_valid_skipcol: ColNr,

    // offset 548 - bool w_viewport_invalid
    pub w_viewport_invalid: bool,
    _pad9: [u8; 3], // to reach 552

    // offset 552
    pub w_viewport_last_topline: LineNr,
    // offset 556
    pub w_viewport_last_botline: LineNr,
    // offset 560
    pub w_viewport_last_topfill: c_int,
    // offset 564
    pub w_viewport_last_skipcol: ColNr,

    // offset 568
    pub w_cline_height: c_int,
    // offset 572 - bool w_cline_folded
    pub w_cline_folded: bool,
    _pad10: [u8; 3], // to reach 576

    // offset 576
    pub w_cline_row: c_int,

    // offset 580
    pub w_virtcol: ColNr,

    // offset 584
    pub w_wrow: c_int,
    // offset 588
    pub w_wcol: c_int,

    // offset 592
    pub w_botline: LineNr,
    // offset 596
    pub w_empty_rows: c_int,
    // offset 600
    pub w_filler_rows: c_int,

    // offset 604
    pub w_lines_valid: c_int,
    // offset 608: wline_T* w_lines (8 bytes)
    pub w_lines: *mut c_void,
    // offset 616
    pub w_lines_size: c_int,
    _pad11: [u8; 4], // to reach 624

    // offset 624: garray_T w_folds (24 bytes: ptr + 4 ints)
    _w_folds: [u8; 24],
    // offset 648: bool w_fold_manual, bool w_foldinvalid
    pub w_fold_manual: bool,
    pub w_foldinvalid: bool,
    _pad12: [u8; 2], // to reach 652

    // offset 652
    pub w_nrwidth: c_int,
    // offset 656
    pub w_scwidth: c_int,
    // offset 660
    pub w_minscwidth: c_int,
    // offset 664
    pub w_maxscwidth: c_int,

    // offset 668
    pub w_redr_type: c_int,
    // offset 672
    pub w_upd_rows: c_int,
    // offset 676
    pub w_redraw_top: LineNr,
    // offset 680
    pub w_redraw_bot: LineNr,
    // offset 684 - bool w_redr_status
    pub w_redr_status: bool,
    // offset 685 - bool w_redr_border
    pub w_redr_border: bool,
    // offset 686 - bool w_redr_statuscol
    pub w_redr_statuscol: bool,
    _pad13: [u8; 1], // to reach 688

    // offset 688: pos_T w_stl_cursor (12 bytes)
    pub w_stl_cursor: PosT,
    // offset 700
    pub w_stl_virtcol: ColNr,
    // offset 704
    pub w_stl_topline: LineNr,
    // offset 708
    pub w_stl_line_count: LineNr,
    // offset 712
    pub w_stl_topfill: c_int,
    // offset 716 - char w_stl_empty
    pub w_stl_empty: c_char,
    _pad14: [u8; 3], // to reach 720
    // offset 720
    pub w_stl_recording: c_int,
    // offset 724
    pub w_stl_state: c_int,
    // offset 728
    pub w_stl_visual_mode: c_int,
    // offset 732: pos_T w_stl_visual_pos (12 bytes)
    pub w_stl_visual_pos: PosT,
    // 732+12=744, already aligned

    // offset 744
    pub w_alt_fnum: c_int,
    _pad16: [u8; 4], // to reach 752

    // offset 752: alist_T* w_alist (8 bytes)
    pub w_alist: *mut c_void,
    // offset 760
    pub w_arg_idx: c_int,
    // offset 764
    pub w_arg_idx_invalid: c_int,

    // offset 768: char* w_localdir (8 bytes)
    pub w_localdir: *mut c_char,
    // offset 776: char* w_prevdir (8 bytes)
    pub w_prevdir: *mut c_char,

    // offset 784: winopt_T w_onebuf_opt (1640 bytes)
    _w_onebuf_opt: [u8; 1640],

    // offset 2424: winopt_T w_allbuf_opt (1640 bytes)
    _w_allbuf_opt: [u8; 1640],

    // offset 4064: int* w_p_cc_cols (8 bytes)
    pub w_p_cc_cols: *mut c_int,
    // offset 4072 - uint8_t w_p_culopt_flags
    pub w_p_culopt_flags: u8,
    _pad17: [u8; 3], // to reach 4076

    // offset 4076
    pub w_briopt_min: c_int,
    // offset 4080
    pub w_briopt_shift: c_int,
    // offset 4084 - bool w_briopt_sbr
    pub w_briopt_sbr: bool,
    _pad18: [u8; 3], // to reach 4088
    // offset 4088
    pub w_briopt_list: c_int,
    // offset 4092
    pub w_briopt_vcol: c_int,

    // offset 4096
    pub w_scbind_pos: c_int,
    _pad19: [u8; 4], // to reach 4104 (ScopeDictDictItem alignment)

    // offset 4104: ScopeDictDictItem w_winvar (24 bytes)
    _w_winvar: [u8; 24],

    // offset 4128: dict_T* w_vars (8 bytes)
    pub w_vars: *mut c_void,

    // offset 4136: pos_T w_pcmark (12 bytes)
    pub w_pcmark: PosT,
    // offset 4148: pos_T w_prev_pcmark (12 bytes)
    pub w_prev_pcmark: PosT,

    // offset 4160: xfmark_T w_jumplist[200] (4800 bytes)
    _w_jumplist: [u8; 4800],

    // offset 8960
    pub w_jumplistlen: c_int,
    // offset 8964
    pub w_jumplistidx: c_int,
    // offset 8968
    pub w_changelistidx: c_int,
    _pad20: [u8; 4], // to reach 8976 (pointer alignment)

    // offset 8976: matchitem_T* w_match_head (8 bytes)
    pub w_match_head: *mut c_void,
    // offset 8984
    pub w_next_match_id: c_int,
    _pad21: [u8; 4], // to reach 8992

    // offset 8992: taggy_T w_tagstack[20] (1280 bytes)
    _w_tagstack: [u8; 1280],

    // offset 10272
    pub w_tagstackidx: c_int,
    // offset 10276
    pub w_tagstacklen: c_int,
    // 10276+4=10280 = w_grid offset

    // offset 10280: GridView w_grid (16 bytes)
    _w_grid: [u8; 16],

    // offset 10296: ScreenGrid w_grid_alloc (96 bytes)
    _w_grid_alloc: [u8; 96],

    // offset 10392 - bool w_pos_changed
    pub w_pos_changed: bool,
    // offset 10393 - bool w_floating
    pub w_floating: bool,
    // offset 10394 - bool w_float_is_info
    pub w_float_is_info: bool,
    _pad23: [u8; 5], // to reach 10400 (WinConfig 8-byte alignment)

    // offset 10400: WinConfig w_config (480 bytes)
    _w_config: [u8; 480],

    // offset 10880
    pub w_fraction: c_int,
    // offset 10884
    pub w_prev_fraction_row: c_int,

    // offset 10888
    pub w_nrwidth_line_count: LineNr,
    // offset 10892
    pub w_statuscol_line_count: LineNr,
    // offset 10896
    pub w_nrwidth_width: c_int,
    _pad24: [u8; 4], // to reach 10904 (pointer alignment)

    // offset 10904: qf_info_T* w_llist (8 bytes)
    pub w_llist: *mut c_void,
    // offset 10912: qf_info_T* w_llist_ref (8 bytes)
    pub w_llist_ref: *mut c_void,

    // offset 10920: StlClickDefinition* w_status_click_defs (8 bytes)
    pub w_status_click_defs: *mut c_void,
    // offset 10928: size_t w_status_click_defs_size (8 bytes)
    pub w_status_click_defs_size: usize,
    // offset 10936: StlClickDefinition* w_winbar_click_defs (8 bytes)
    pub w_winbar_click_defs: *mut c_void,
    // offset 10944: size_t w_winbar_click_defs_size (8 bytes)
    pub w_winbar_click_defs_size: usize,
    // offset 10952: StlClickDefinition* w_statuscol_click_defs (8 bytes)
    pub w_statuscol_click_defs: *mut c_void,
    // offset 10960: size_t w_statuscol_click_defs_size (8 bytes)
    pub w_statuscol_click_defs_size: usize,
    // sizeof(win_T) = 10968
}

// =============================================================================
// Accessor methods for winopt_T fields accessed via raw byte offsets.
//
// These fields live inside _w_onebuf_opt (starting at offset 784) via the C
// `#define w_p_X w_onebuf_opt.wo_X` macros. We use raw pointer arithmetic with
// ABSOLUTE offsets (as measured in the full win_T struct) to read/write them.
//
// All offsets are validated in window_struct_check.c.
// =============================================================================

/// Read a c_int at `abs_offset` bytes from the start of a WinStruct.
///
/// # Safety
/// The caller must ensure `abs_offset` is a valid field offset for the
/// `win_T` struct, as validated by `_Static_assert` in window_struct_check.c.
#[inline]
#[allow(clippy::cast_ptr_alignment)]
const unsafe fn read_int_at(ws: &WinStruct, abs_offset: usize) -> c_int {
    let ptr = std::ptr::addr_of!(*ws)
        .cast::<u8>()
        .add(abs_offset)
        .cast::<c_int>();
    ptr.read_unaligned()
}

/// Write a c_int at `abs_offset` bytes from the start of a WinStruct.
///
/// # Safety
/// The caller must ensure `abs_offset` is a valid field offset for the
/// `win_T` struct, as validated by `_Static_assert` in window_struct_check.c.
#[inline]
#[allow(clippy::cast_ptr_alignment)]
const unsafe fn write_int_at(ws: &mut WinStruct, abs_offset: usize, val: c_int) {
    let ptr = std::ptr::addr_of_mut!(*ws)
        .cast::<u8>()
        .add(abs_offset)
        .cast::<c_int>();
    ptr.write_unaligned(val);
}

/// Read an OptInt (i64) at `abs_offset` bytes from the start of a WinStruct.
///
/// # Safety
/// The caller must ensure `abs_offset` is a valid field offset for the
/// `win_T` struct, as validated by `_Static_assert` in window_struct_check.c.
#[inline]
#[allow(clippy::cast_ptr_alignment)]
const unsafe fn read_optint_at(ws: &WinStruct, abs_offset: usize) -> OptInt {
    let ptr = std::ptr::addr_of!(*ws)
        .cast::<u8>()
        .add(abs_offset)
        .cast::<OptInt>();
    ptr.read_unaligned()
}

impl WinStruct {
    /// `w_p_pvw` (preview window) - at absolute offset 972
    #[must_use]
    #[inline]
    pub const fn w_p_pvw(&self) -> c_int {
        unsafe { read_int_at(self, 972) }
    }

    /// `w_p_wfh` (winfixheight) - at absolute offset 964
    #[must_use]
    #[inline]
    pub const fn w_p_wfh(&self) -> c_int {
        unsafe { read_int_at(self, 964) }
    }

    /// `w_p_wfw` (winfixwidth) - at absolute offset 968
    #[must_use]
    #[inline]
    pub const fn w_p_wfw(&self) -> c_int {
        unsafe { read_int_at(self, 968) }
    }

    /// `w_p_diff` - at absolute offset 800
    #[must_use]
    #[inline]
    pub const fn w_p_diff(&self) -> c_int {
        unsafe { read_int_at(self, 800) }
    }

    /// `w_p_crb` (scrollbind option) - at absolute offset 1112
    #[must_use]
    #[inline]
    pub const fn w_p_crb(&self) -> c_int {
        unsafe { read_int_at(self, 1112) }
    }

    /// Set `w_p_crb` - at absolute offset 1112
    ///
    /// # Safety
    /// Caller must hold exclusive access to this window.
    #[inline]
    pub const unsafe fn set_w_p_crb(&mut self, val: c_int) {
        write_int_at(self, 1112, val);
    }

    /// `w_p_fen` (foldenable) - at absolute offset 832
    #[must_use]
    #[inline]
    pub const fn w_p_fen(&self) -> c_int {
        unsafe { read_int_at(self, 832) }
    }

    /// `w_p_rnu` (relativenumber) - at absolute offset 932
    #[must_use]
    #[inline]
    pub const fn w_p_rnu(&self) -> c_int {
        unsafe { read_int_at(self, 932) }
    }

    /// `w_p_nu` (number) - at absolute offset 928
    #[must_use]
    #[inline]
    pub const fn w_p_nu(&self) -> c_int {
        unsafe { read_int_at(self, 928) }
    }

    /// `w_p_list` - at absolute offset 924
    #[must_use]
    #[inline]
    pub const fn w_p_list(&self) -> c_int {
        unsafe { read_int_at(self, 924) }
    }

    /// `w_p_cul` (cursorline) - at absolute offset 1020
    #[must_use]
    #[inline]
    pub const fn w_p_cul(&self) -> c_int {
        unsafe { read_int_at(self, 1020) }
    }

    /// `w_p_cole` (conceallevel) - at absolute offset 1104 (OptInt = i64)
    #[must_use]
    #[inline]
    pub const fn w_p_cole(&self) -> OptInt {
        unsafe { read_optint_at(self, 1104) }
    }

    /// `w_p_so` (scrolloff) - at absolute offset 1136 (OptInt = i64)
    #[must_use]
    #[inline]
    pub const fn w_p_so(&self) -> OptInt {
        unsafe { read_optint_at(self, 1136) }
    }

    /// `w_p_siso` (sidescrolloff) - at absolute offset 1128 (OptInt = i64)
    #[must_use]
    #[inline]
    pub const fn w_p_siso(&self) -> OptInt {
        unsafe { read_optint_at(self, 1128) }
    }

    /// `w_p_wrap` - at absolute offset 1084
    #[must_use]
    #[inline]
    pub const fn w_p_wrap(&self) -> c_int {
        unsafe { read_int_at(self, 1084) }
    }

    /// `w_p_cuc` (cursorcolumn) - at absolute offset 1016
    #[must_use]
    #[inline]
    pub const fn w_p_cuc(&self) -> c_int {
        unsafe { read_int_at(self, 1016) }
    }

    /// `w_p_bri` (breakindent) - at absolute offset 788
    #[must_use]
    #[inline]
    pub const fn w_p_bri(&self) -> c_int {
        unsafe { read_int_at(self, 788) }
    }

    /// `w_p_rl` (rightleft) - at absolute offset 984
    #[must_use]
    #[inline]
    pub const fn w_p_rl(&self) -> c_int {
        unsafe { read_int_at(self, 984) }
    }

    /// Set `w_p_rl` (rightleft) - at absolute offset 984
    ///
    /// # Safety
    /// Caller must hold exclusive access to this window.
    #[inline]
    pub const unsafe fn set_w_p_rl(&mut self, val: c_int) {
        write_int_at(self, 984, val);
    }

    /// `w_p_arab` (arabic) - at absolute offset 784
    #[must_use]
    #[inline]
    pub const fn w_p_arab(&self) -> c_int {
        unsafe { read_int_at(self, 784) }
    }

    /// `w_p_scb` (scrollbind) - at absolute offset 1072
    #[must_use]
    #[inline]
    pub const fn w_p_scb(&self) -> c_int {
        unsafe { read_int_at(self, 1072) }
    }

    /// `w_p_sms` (smoothscroll) - at absolute offset 1008
    #[must_use]
    #[inline]
    pub const fn w_p_sms(&self) -> c_int {
        unsafe { read_int_at(self, 1008) }
    }

    /// Set `w_p_sms` (smoothscroll) - at absolute offset 1008
    ///
    /// # Safety
    /// Caller must hold exclusive access to this window.
    #[inline]
    pub const unsafe fn set_w_p_sms(&mut self, val: c_int) {
        write_int_at(self, 1008, val);
    }

    /// `w_p_nuw` (numberwidth) - at absolute offset 952 (OptInt = i64)
    #[must_use]
    #[inline]
    pub const fn w_p_nuw(&self) -> OptInt {
        unsafe { read_optint_at(self, 952) }
    }

    /// `w_p_wfb` (winfixbuf) - at absolute offset 960
    #[must_use]
    #[inline]
    pub const fn w_p_wfb(&self) -> c_int {
        unsafe { read_int_at(self, 960) }
    }

    /// `w_p_winbl` (winblend) - at absolute offset 1168 (OptInt = i64)
    #[must_use]
    #[inline]
    pub const fn w_p_winbl(&self) -> OptInt {
        unsafe { read_optint_at(self, 1168) }
    }

    /// `w_p_wrap_flags` - at absolute offset 1176
    #[must_use]
    #[inline]
    pub const fn w_p_wrap_flags(&self) -> c_int {
        unsafe { read_int_at(self, 1176) }
    }
}

/// Get a reference to `WinStruct` from a `WinHandle`.
///
/// # Safety
/// The handle must be a valid non-null `win_T*` with a lifetime at least as
/// long as `'a`.
#[inline]
#[must_use]
pub const unsafe fn win_ref<'a>(wp: WinHandle) -> &'a WinStruct {
    &*(wp.as_ptr().cast::<WinStruct>())
}

/// Get a mutable reference to `WinStruct` from a `WinHandle`.
///
/// # Safety
/// The handle must be a valid non-null `win_T*` with a lifetime at least as
/// long as `'a`, and the caller must guarantee exclusive access.
#[inline]
#[must_use]
pub unsafe fn win_mut<'a>(wp: WinHandle) -> &'a mut WinStruct {
    &mut *(wp.as_ptr().cast::<WinStruct>())
}

// =============================================================================
// Phase 2: Simple getter #[export_name] functions replacing C accessors.
//
// Each function matches the C signature in window_shim.c exactly.
// The corresponding C functions are deleted from window_shim.c.
// =============================================================================

/// Returns `wp->w_locked`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_locked"]
#[must_use]
pub const unsafe extern "C" fn win_get_locked(wp: WinHandle) -> bool {
    win_ref(wp).w_locked
}

/// Returns `wp->w_floating`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_floating"]
#[must_use]
pub const unsafe extern "C" fn win_get_floating(wp: WinHandle) -> bool {
    win_ref(wp).w_floating
}

/// Returns `wp->w_p_pvw` (preview window option).
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_pvw"]
#[must_use]
pub const unsafe extern "C" fn win_get_pvw(wp: WinHandle) -> c_int {
    win_ref(wp).w_p_pvw()
}

/// Returns `wp->w_next`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_next"]
#[must_use]
pub const unsafe extern "C" fn win_get_next(wp: WinHandle) -> WinHandle {
    win_ref(wp).w_next
}

/// Returns `wp->w_prev`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_prev"]
#[must_use]
pub const unsafe extern "C" fn win_get_prev(wp: WinHandle) -> WinHandle {
    win_ref(wp).w_prev
}

/// Returns `wp->w_frame`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_frame"]
#[must_use]
pub const unsafe extern "C" fn win_get_frame(wp: WinHandle) -> *mut Frame {
    win_ref(wp).w_frame
}

/// Returns `wp->handle`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_handle"]
#[must_use]
pub const unsafe extern "C" fn win_get_handle(wp: WinHandle) -> HandleT {
    win_ref(wp).handle
}

/// Returns `wp->w_width`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_w_width"]
#[must_use]
pub const unsafe extern "C" fn win_get_w_width(wp: WinHandle) -> c_int {
    win_ref(wp).w_width
}

/// Returns `wp->w_height`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_w_height"]
#[must_use]
pub const unsafe extern "C" fn win_get_w_height(wp: WinHandle) -> c_int {
    win_ref(wp).w_height
}

/// Returns `wp->w_winrow`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_winrow"]
#[must_use]
pub const unsafe extern "C" fn win_get_winrow(wp: WinHandle) -> c_int {
    win_ref(wp).w_winrow
}

/// Returns `wp->w_wincol`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_wincol"]
#[must_use]
pub const unsafe extern "C" fn win_get_wincol(wp: WinHandle) -> c_int {
    win_ref(wp).w_wincol
}

/// Returns `wp->w_winrow_off`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_winrow_off"]
#[must_use]
pub const unsafe extern "C" fn win_get_winrow_off(wp: WinHandle) -> c_int {
    win_ref(wp).w_winrow_off
}

/// Returns `wp->w_wincol_off`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_wincol_off"]
#[must_use]
pub const unsafe extern "C" fn win_get_wincol_off(wp: WinHandle) -> c_int {
    win_ref(wp).w_wincol_off
}

/// Returns `wp->w_hsep_height`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_hsep_height"]
#[must_use]
pub const unsafe extern "C" fn win_get_hsep_height(wp: WinHandle) -> c_int {
    win_ref(wp).w_hsep_height
}

/// Returns `wp->w_vsep_width`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_vsep_width"]
#[must_use]
pub const unsafe extern "C" fn win_get_vsep_width(wp: WinHandle) -> c_int {
    win_ref(wp).w_vsep_width
}

/// Returns `wp->w_status_height`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_status_height"]
#[must_use]
pub const unsafe extern "C" fn win_get_status_height(wp: WinHandle) -> c_int {
    win_ref(wp).w_status_height
}

/// Returns `wp->w_winbar_height`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_winbar_height"]
#[must_use]
pub const unsafe extern "C" fn win_get_winbar_height(wp: WinHandle) -> c_int {
    win_ref(wp).w_winbar_height
}

/// Returns `wp->w_view_width`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_view_width"]
#[must_use]
pub const unsafe extern "C" fn win_get_view_width(wp: WinHandle) -> c_int {
    win_ref(wp).w_view_width
}

/// Returns `wp->w_view_height`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_view_height"]
#[must_use]
pub const unsafe extern "C" fn win_get_view_height(wp: WinHandle) -> c_int {
    win_ref(wp).w_view_height
}

/// Returns `wp->w_buffer`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_w_buffer"]
#[must_use]
pub const unsafe extern "C" fn win_get_w_buffer(wp: WinHandle) -> *mut c_void {
    win_ref(wp).w_buffer
}

/// Returns `wp->w_valid`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_valid"]
#[must_use]
pub const unsafe extern "C" fn win_get_valid(wp: WinHandle) -> c_int {
    win_ref(wp).w_valid
}

/// Returns `wp->w_p_wfh` (winfixheight).
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_wfh"]
#[must_use]
pub const unsafe extern "C" fn win_get_wfh(wp: WinHandle) -> c_int {
    win_ref(wp).w_p_wfh()
}

/// Returns `wp->w_p_wfw` (winfixwidth).
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_wfw"]
#[must_use]
pub const unsafe extern "C" fn win_get_wfw(wp: WinHandle) -> c_int {
    win_ref(wp).w_p_wfw()
}

/// Returns `wp->w_p_diff`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_diff"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_diff(wp: WinHandle) -> c_int {
    win_ref(wp).w_p_diff()
}

/// Returns `wp->w_p_crb` (scrollbind).
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_crb"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_crb(wp: WinHandle) -> c_int {
    win_ref(wp).w_p_crb()
}

/// Returns `wp->w_p_fen` (foldenable).
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_fen"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_fen(wp: WinHandle) -> c_int {
    win_ref(wp).w_p_fen()
}

/// Returns `wp->w_p_rnu` (relativenumber).
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_rnu"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_rnu(wp: WinHandle) -> c_int {
    win_ref(wp).w_p_rnu()
}

/// Returns `wp->w_p_nu` (number).
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_nu"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_nu(wp: WinHandle) -> c_int {
    win_ref(wp).w_p_nu()
}

/// Returns `wp->w_p_nuw` (numberwidth).
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_nuw"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_nuw(wp: WinHandle) -> OptInt {
    win_ref(wp).w_p_nuw()
}

/// Returns `wp->w_p_list`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_list"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_list(wp: WinHandle) -> c_int {
    win_ref(wp).w_p_list()
}

/// Returns `wp->w_p_cul` (cursorline).
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_cul"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_cul(wp: WinHandle) -> c_int {
    win_ref(wp).w_p_cul()
}

/// Returns `wp->w_p_cole` (conceallevel).
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_cole"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_cole(wp: WinHandle) -> OptInt {
    win_ref(wp).w_p_cole()
}

/// Returns `wp->w_p_so` (scrolloff).
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_so"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_so(wp: WinHandle) -> OptInt {
    win_ref(wp).w_p_so()
}

/// Returns `wp->w_p_siso` (sidescrolloff).
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_siso"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_siso(wp: WinHandle) -> OptInt {
    win_ref(wp).w_p_siso()
}

/// Returns `wp->w_p_wrap`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_wrap"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_wrap(wp: WinHandle) -> c_int {
    win_ref(wp).w_p_wrap()
}

/// Returns `wp->w_p_cuc` (cursorcolumn).
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_cuc"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_cuc(wp: WinHandle) -> c_int {
    win_ref(wp).w_p_cuc()
}

/// Returns `wp->w_p_bri` (breakindent).
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_bri"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_bri(wp: WinHandle) -> c_int {
    win_ref(wp).w_p_bri()
}

/// Returns `wp->w_p_rl` (rightleft).
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_rl"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_rl(wp: WinHandle) -> c_int {
    win_ref(wp).w_p_rl()
}

/// Returns `wp->w_p_arab` (arabic).
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_arab"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_arab(wp: WinHandle) -> c_int {
    win_ref(wp).w_p_arab()
}

/// Returns `wp->w_p_scb` (scrollbind).
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_scb"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_scb(wp: WinHandle) -> bool {
    win_ref(wp).w_p_scb() != 0
}

/// Returns `wp->w_p_sms` (smoothscroll). Returns 0 if wp is null.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_p_sms"]
#[must_use]
pub const unsafe extern "C" fn win_get_p_sms(wp: WinHandle) -> c_int {
    if wp.as_ptr().is_null() {
        return 0;
    }
    win_ref(wp).w_p_sms()
}

/// Returns `wp->w_ns_hl`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_ns_hl"]
#[must_use]
pub const unsafe extern "C" fn win_get_ns_hl(wp: WinHandle) -> c_int {
    win_ref(wp).w_ns_hl
}

/// Returns `wp->w_ns_hl_active`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_ns_hl_active"]
#[must_use]
pub const unsafe extern "C" fn win_get_ns_hl_active(wp: WinHandle) -> c_int {
    win_ref(wp).w_ns_hl_active
}

/// Returns `wp->w_hl_attr_normal`, or 0 if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_get_hl_attr_normal"]
#[must_use]
pub const unsafe extern "C" fn win_get_hl_attr_normal(wp: WinHandle) -> c_int {
    if wp.as_ptr().is_null() {
        return 0;
    }
    win_ref(wp).w_hl_attr_normal
}

/// Returns `wp->w_hl_attr_normalnc`, or 0 if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_get_hl_attr_normalnc"]
#[must_use]
pub const unsafe extern "C" fn win_get_hl_attr_normalnc(wp: WinHandle) -> c_int {
    if wp.as_ptr().is_null() {
        return 0;
    }
    win_ref(wp).w_hl_attr_normalnc
}

/// Returns `wp->w_hl_needs_update`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_hl_needs_update"]
#[must_use]
pub const unsafe extern "C" fn win_get_hl_needs_update(wp: WinHandle) -> bool {
    win_ref(wp).w_hl_needs_update != 0
}

/// Returns `wp->w_cursor.lnum`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_cursor_lnum"]
#[must_use]
pub const unsafe extern "C" fn win_get_cursor_lnum(wp: WinHandle) -> LineNr {
    win_ref(wp).w_cursor.lnum
}

/// Returns `wp->w_cursor.col`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_cursor_col"]
#[must_use]
pub const unsafe extern "C" fn win_get_cursor_col(wp: WinHandle) -> ColNr {
    win_ref(wp).w_cursor.col
}

/// Returns `wp->w_cursor.coladd`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_cursor_coladd"]
#[must_use]
pub const unsafe extern "C" fn win_get_cursor_coladd(wp: WinHandle) -> ColNr {
    win_ref(wp).w_cursor.coladd
}

/// Returns `wp->w_topline`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_topline"]
#[must_use]
pub const unsafe extern "C" fn win_get_topline(wp: WinHandle) -> LineNr {
    win_ref(wp).w_topline
}

/// Returns `wp->w_botline`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_botline"]
#[must_use]
pub const unsafe extern "C" fn win_get_botline(wp: WinHandle) -> LineNr {
    win_ref(wp).w_botline
}

/// Returns `wp->w_topfill`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_topfill"]
#[must_use]
pub const unsafe extern "C" fn win_get_topfill(wp: WinHandle) -> c_int {
    win_ref(wp).w_topfill
}

/// Returns `wp->w_leftcol`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_leftcol"]
#[must_use]
pub const unsafe extern "C" fn win_get_leftcol(wp: WinHandle) -> ColNr {
    win_ref(wp).w_leftcol
}

/// Returns `wp->w_skipcol`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_skipcol"]
#[must_use]
pub const unsafe extern "C" fn win_get_skipcol(wp: WinHandle) -> ColNr {
    win_ref(wp).w_skipcol
}

/// Returns `wp->w_virtcol`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_virtcol"]
#[must_use]
pub const unsafe extern "C" fn win_get_virtcol(wp: WinHandle) -> ColNr {
    win_ref(wp).w_virtcol
}

/// Returns `wp->w_wcol`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_wcol"]
#[must_use]
pub const unsafe extern "C" fn win_get_wcol(wp: WinHandle) -> c_int {
    win_ref(wp).w_wcol
}

/// Returns `wp->w_wrow`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_wrow"]
#[must_use]
pub const unsafe extern "C" fn win_get_wrow(wp: WinHandle) -> c_int {
    win_ref(wp).w_wrow
}

/// Returns `wp->w_curswant`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_curswant"]
#[must_use]
pub const unsafe extern "C" fn win_get_curswant(wp: WinHandle) -> ColNr {
    win_ref(wp).w_curswant
}

/// Returns `wp->w_set_curswant != 0`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_set_curswant"]
#[must_use]
pub const unsafe extern "C" fn win_get_set_curswant(wp: WinHandle) -> c_int {
    if win_ref(wp).w_set_curswant != 0 {
        1
    } else {
        0
    }
}

/// Returns `wp->w_cursorline`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_cursorline"]
#[must_use]
pub const unsafe extern "C" fn win_get_cursorline(wp: WinHandle) -> LineNr {
    win_ref(wp).w_cursorline
}

/// Returns `wp->w_empty_rows` or 0 if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_get_empty_rows"]
#[must_use]
pub const unsafe extern "C" fn win_get_empty_rows(wp: WinHandle) -> c_int {
    if wp.as_ptr().is_null() {
        return 0;
    }
    win_ref(wp).w_empty_rows
}

/// Returns `wp->w_arg_idx`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_arg_idx"]
#[must_use]
pub const unsafe extern "C" fn win_get_arg_idx(wp: WinHandle) -> c_int {
    win_ref(wp).w_arg_idx
}

/// Returns `wp->w_arg_idx_invalid`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_arg_idx_invalid"]
#[must_use]
pub const unsafe extern "C" fn win_get_arg_idx_invalid(wp: WinHandle) -> c_int {
    win_ref(wp).w_arg_idx_invalid
}

/// Returns `wp->w_redr_type` or 0 if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_get_redr_type"]
#[must_use]
pub const unsafe extern "C" fn win_get_redr_type(wp: WinHandle) -> c_int {
    if wp.as_ptr().is_null() {
        return 0;
    }
    win_ref(wp).w_redr_type
}

/// Returns `wp->w_redr_status` or 0 if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_get_redr_status"]
#[must_use]
pub const unsafe extern "C" fn win_get_redr_status(wp: WinHandle) -> c_int {
    if wp.as_ptr().is_null() {
        return 0;
    }
    if win_ref(wp).w_redr_status {
        1
    } else {
        0
    }
}

/// Returns `wp->w_redr_border` or 0 if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_get_redr_border"]
#[must_use]
pub const unsafe extern "C" fn win_get_redr_border(wp: WinHandle) -> c_int {
    if wp.as_ptr().is_null() {
        return 0;
    }
    if win_ref(wp).w_redr_border {
        1
    } else {
        0
    }
}

/// Returns `wp->w_lines_valid` or 0 if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_get_lines_valid"]
#[must_use]
pub const unsafe extern "C" fn win_get_lines_valid(wp: WinHandle) -> c_int {
    if wp.as_ptr().is_null() {
        return 0;
    }
    win_ref(wp).w_lines_valid
}

/// Returns `wp->w_redraw_top` or 0 if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_get_redraw_top"]
#[must_use]
pub const unsafe extern "C" fn win_get_redraw_top(wp: WinHandle) -> LineNr {
    if wp.as_ptr().is_null() {
        return 0;
    }
    win_ref(wp).w_redraw_top
}

/// Returns `wp->w_redraw_bot` or 0 if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_get_redraw_bot"]
#[must_use]
pub const unsafe extern "C" fn win_get_redraw_bot(wp: WinHandle) -> LineNr {
    if wp.as_ptr().is_null() {
        return 0;
    }
    win_ref(wp).w_redraw_bot
}

/// Returns `wp->w_cline_row`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_cline_row"]
#[must_use]
pub const unsafe extern "C" fn win_get_cline_row(wp: WinHandle) -> c_int {
    win_ref(wp).w_cline_row
}

/// Returns `wp->w_cline_height`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_cline_height"]
#[must_use]
pub const unsafe extern "C" fn win_get_cline_height(wp: WinHandle) -> c_int {
    win_ref(wp).w_cline_height
}

/// Returns `wp->w_cline_folded ? 1 : 0`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_cline_folded"]
#[must_use]
pub const unsafe extern "C" fn win_get_cline_folded(wp: WinHandle) -> c_int {
    if win_ref(wp).w_cline_folded {
        1
    } else {
        0
    }
}

/// Returns `wp->w_scwidth`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_scwidth"]
#[must_use]
pub const unsafe extern "C" fn win_get_scwidth(wp: WinHandle) -> c_int {
    win_ref(wp).w_scwidth
}

/// Returns `wp->w_minscwidth`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_minscwidth"]
#[must_use]
pub const unsafe extern "C" fn win_get_minscwidth(wp: WinHandle) -> c_int {
    win_ref(wp).w_minscwidth
}

/// Returns `wp->w_nrwidth_width`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_nrwidth_width"]
#[must_use]
pub const unsafe extern "C" fn win_get_nrwidth_width(wp: WinHandle) -> c_int {
    win_ref(wp).w_nrwidth_width
}

/// Returns `wp->w_nrwidth_line_count`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_get_nrwidth_line_count"]
#[must_use]
pub const unsafe extern "C" fn win_get_nrwidth_line_count(wp: WinHandle) -> LineNr {
    win_ref(wp).w_nrwidth_line_count
}

/// Returns `wp->w_alt_fnum` or 0 if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_get_alt_fnum"]
#[must_use]
pub const unsafe extern "C" fn win_get_alt_fnum(wp: WinHandle) -> c_int {
    if wp.as_ptr().is_null() {
        return 0;
    }
    win_ref(wp).w_alt_fnum
}

// =============================================================================
// Phase 3: Simple setter #[export_name] functions replacing C setters.
//
// Each function matches the C signature in window_shim.c exactly.
// =============================================================================

/// Sets `wp->w_next`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_next"]
pub unsafe extern "C" fn win_set_next(wp: WinHandle, next: WinHandle) {
    win_mut(wp).w_next = next;
}

/// Sets `wp->w_prev`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_prev"]
pub unsafe extern "C" fn win_set_prev(wp: WinHandle, prev: WinHandle) {
    win_mut(wp).w_prev = prev;
}

/// Sets `wp->w_ns_hl_active`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_ns_hl_active"]
pub unsafe extern "C" fn win_set_ns_hl_active(wp: WinHandle, val: c_int) {
    win_mut(wp).w_ns_hl_active = val;
}

/// Sets `wp->w_ns_hl_attr`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_ns_hl_attr"]
pub unsafe extern "C" fn win_set_ns_hl_attr(wp: WinHandle, val: *mut c_int) {
    win_mut(wp).w_ns_hl_attr = val;
}

/// Sets `wp->w_hl_needs_update`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_hl_needs_update"]
pub unsafe extern "C" fn win_set_hl_needs_update(wp: WinHandle, val: bool) {
    win_mut(wp).w_hl_needs_update = c_int::from(val);
}

/// Sets `wp->w_hl_attr_normal`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_hl_attr_normal"]
pub unsafe extern "C" fn win_set_hl_attr_normal(wp: WinHandle, val: c_int) {
    win_mut(wp).w_hl_attr_normal = val;
}

/// Sets `wp->w_hl_attr_normalnc`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_hl_attr_normalnc"]
pub unsafe extern "C" fn win_set_hl_attr_normalnc(wp: WinHandle, val: c_int) {
    win_mut(wp).w_hl_attr_normalnc = val;
}

/// Sets `wp->w_valid`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_valid"]
pub unsafe extern "C" fn win_set_valid(wp: WinHandle, val: c_int) {
    win_mut(wp).w_valid = val;
}

/// Sets `wp->w_valid |= bits`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_valid_bits"]
pub unsafe extern "C" fn win_set_valid_bits(wp: WinHandle, bits: c_int) {
    win_mut(wp).w_valid |= bits;
}

/// Clears valid bits: `wp->w_valid &= ~bits`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_clear_valid_bits"]
pub unsafe extern "C" fn win_clear_valid_bits(wp: WinHandle, bits: c_int) {
    win_mut(wp).w_valid &= !bits;
}

/// Sets `wp->w_lines_valid`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_lines_valid"]
pub unsafe extern "C" fn win_set_lines_valid(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_lines_valid = val;
}

/// Sets `wp->w_pos_changed`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_pos_changed"]
pub unsafe extern "C" fn win_set_pos_changed(wp: WinHandle, val: c_int) {
    win_mut(wp).w_pos_changed = val != 0;
}

/// Sets `wp->w_redr_status`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_redr_status"]
pub unsafe extern "C" fn win_set_redr_status(wp: WinHandle, val: c_int) {
    win_mut(wp).w_redr_status = val != 0;
}

/// Sets `wp->w_redr_type`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_redr_type"]
pub unsafe extern "C" fn win_set_redr_type(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_redr_type = val;
}

/// Sets `wp->w_redr_statuscol`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_redr_statuscol"]
pub unsafe extern "C" fn win_set_redr_statuscol(wp: WinHandle, val: bool) {
    win_mut(wp).w_redr_statuscol = val;
}

/// Sets `wp->w_redraw_top`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_redraw_top"]
pub unsafe extern "C" fn win_set_redraw_top(wp: WinHandle, val: LineNr) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_redraw_top = val;
}

/// Sets `wp->w_redraw_bot`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_redraw_bot"]
pub unsafe extern "C" fn win_set_redraw_bot(wp: WinHandle, val: LineNr) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_redraw_bot = val;
}

/// Sets `wp->w_cursor.lnum`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_cursor_lnum"]
pub unsafe extern "C" fn win_set_cursor_lnum(wp: WinHandle, lnum: LineNr) {
    win_mut(wp).w_cursor.lnum = lnum;
}

/// Sets `wp->w_cursor.col`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_cursor_col"]
pub unsafe extern "C" fn win_set_cursor_col(wp: WinHandle, col: ColNr) {
    win_mut(wp).w_cursor.col = col;
}

/// Sets `wp->w_topline`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_topline"]
pub unsafe extern "C" fn win_set_topline(wp: WinHandle, val: LineNr) {
    win_mut(wp).w_topline = val;
}

/// Sets `wp->w_topfill`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_topfill"]
pub unsafe extern "C" fn win_set_topfill(wp: WinHandle, val: c_int) {
    win_mut(wp).w_topfill = val;
}

/// Sets `wp->w_botline`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_botline"]
pub unsafe extern "C" fn win_set_botline(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_botline = val as LineNr;
}

/// Sets `wp->w_leftcol`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_leftcol"]
pub unsafe extern "C" fn win_set_leftcol(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_leftcol = val as ColNr;
}

/// Sets `wp->w_skipcol`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_skipcol"]
pub unsafe extern "C" fn win_set_skipcol(wp: WinHandle, val: ColNr) {
    win_mut(wp).w_skipcol = val;
}

/// Sets `wp->w_topline_was_set`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_topline_was_set"]
pub unsafe extern "C" fn win_set_topline_was_set(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_topline_was_set = u8::from(val != 0);
}

/// Sets `wp->w_virtcol`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_virtcol"]
pub unsafe extern "C" fn win_set_virtcol(wp: WinHandle, val: ColNr) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_virtcol = val;
}

/// Sets `wp->w_wcol`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_wcol"]
pub unsafe extern "C" fn win_set_wcol(wp: WinHandle, val: c_int) {
    win_mut(wp).w_wcol = val;
}

/// Sets `wp->w_wrow`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_wrow"]
pub unsafe extern "C" fn win_set_wrow(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_wrow = val;
}

/// Sets `wp->w_curswant`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_curswant"]
pub unsafe extern "C" fn win_set_curswant(wp: WinHandle, val: ColNr) {
    win_mut(wp).w_curswant = val;
}

/// Sets `wp->w_set_curswant`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_set_curswant"]
pub unsafe extern "C" fn win_set_set_curswant(wp: WinHandle, val: c_int) {
    win_mut(wp).w_set_curswant = val;
}

/// Sets `wp->w_cline_row`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_cline_row"]
pub unsafe extern "C" fn win_set_cline_row(wp: WinHandle, val: c_int) {
    win_mut(wp).w_cline_row = val;
}

/// Sets `wp->w_cline_height`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_cline_height"]
pub unsafe extern "C" fn win_set_cline_height(wp: WinHandle, val: c_int) {
    win_mut(wp).w_cline_height = val;
}

/// Sets `wp->w_cline_folded`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_cline_folded"]
pub unsafe extern "C" fn win_set_cline_folded(wp: WinHandle, val: c_int) {
    win_mut(wp).w_cline_folded = val != 0;
}

/// Sets `wp->w_viewport_invalid`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_viewport_invalid"]
pub unsafe extern "C" fn win_set_viewport_invalid(wp: WinHandle, val: c_int) {
    win_mut(wp).w_viewport_invalid = val != 0;
}

/// Sets `wp->w_valid_cursor` fields.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_valid_cursor"]
pub unsafe extern "C" fn win_set_valid_cursor(
    wp: WinHandle,
    lnum: LineNr,
    col: ColNr,
    coladd: ColNr,
) {
    let ws = win_mut(wp);
    ws.w_valid_cursor.lnum = lnum;
    ws.w_valid_cursor.col = col;
    ws.w_valid_cursor.coladd = coladd;
}

/// Sets `wp->w_valid_cursor.col`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_valid_cursor_col"]
pub unsafe extern "C" fn win_set_valid_cursor_col(wp: WinHandle, col: ColNr) {
    win_mut(wp).w_valid_cursor.col = col;
}

/// Sets `wp->w_valid_cursor.coladd`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_valid_cursor_coladd"]
pub unsafe extern "C" fn win_set_valid_cursor_coladd(wp: WinHandle, coladd: ColNr) {
    win_mut(wp).w_valid_cursor.coladd = coladd;
}

/// Sets `wp->w_valid_leftcol`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_valid_leftcol"]
pub unsafe extern "C" fn win_set_valid_leftcol(wp: WinHandle, val: ColNr) {
    win_mut(wp).w_valid_leftcol = val;
}

/// Sets `wp->w_valid_skipcol`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_valid_skipcol"]
pub unsafe extern "C" fn win_set_valid_skipcol(wp: WinHandle, val: ColNr) {
    win_mut(wp).w_valid_skipcol = val;
}

/// Sets `wp->w_winrow`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_winrow"]
pub unsafe extern "C" fn win_set_winrow(wp: WinHandle, val: c_int) {
    win_mut(wp).w_winrow = val;
}

/// Sets `wp->w_wincol`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_wincol"]
pub unsafe extern "C" fn win_set_wincol(wp: WinHandle, val: c_int) {
    win_mut(wp).w_wincol = val;
}

/// Sets `wp->w_hsep_height`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_hsep_height"]
pub unsafe extern "C" fn win_set_hsep_height(wp: WinHandle, val: c_int) {
    win_mut(wp).w_hsep_height = val;
}

/// Sets `wp->w_status_height`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_status_height"]
pub unsafe extern "C" fn win_set_status_height(wp: WinHandle, val: c_int) {
    win_mut(wp).w_status_height = val;
}

/// Sets `wp->w_vsep_width`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_vsep_width"]
pub unsafe extern "C" fn win_set_vsep_width(wp: WinHandle, val: c_int) {
    win_mut(wp).w_vsep_width = val;
}

/// Sets `wp->w_winbar_height`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_winbar_height"]
pub unsafe extern "C" fn win_set_winbar_height(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_winbar_height = val;
}

/// Sets `wp->w_height`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_field_height"]
pub unsafe extern "C" fn win_set_field_height(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_height = val;
}

/// Sets `wp->w_width`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_field_width"]
pub unsafe extern "C" fn win_set_field_width(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_width = val;
}

/// Sets `wp->w_view_height`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_view_height"]
pub unsafe extern "C" fn win_set_view_height(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_view_height = val;
}

/// Sets `wp->w_view_width`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_view_width"]
pub unsafe extern "C" fn win_set_view_width(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_view_width = val;
}

/// Sets `wp->w_height_outer`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_height_outer"]
pub unsafe extern "C" fn win_set_height_outer(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_height_outer = val;
}

/// Sets `wp->w_width_outer`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_width_outer"]
pub unsafe extern "C" fn win_set_width_outer(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_width_outer = val;
}

/// Sets `wp->w_winrow_off`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_winrow_off"]
pub unsafe extern "C" fn win_set_winrow_off(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_winrow_off = val;
}

/// Sets `wp->w_wincol_off`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_wincol_off"]
pub unsafe extern "C" fn win_set_wincol_off(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_wincol_off = val;
}

/// Sets `wp->w_empty_rows`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_empty_rows"]
pub unsafe extern "C" fn win_set_empty_rows(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_empty_rows = val;
}

/// Sets `wp->w_nrwidth_width`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_nrwidth_width"]
pub unsafe extern "C" fn win_set_nrwidth_width(wp: WinHandle, val: c_int) {
    win_mut(wp).w_nrwidth_width = val;
}

/// Sets `wp->w_nrwidth_line_count`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_nrwidth_line_count"]
pub unsafe extern "C" fn win_set_nrwidth_line_count(wp: WinHandle, val: LineNr) {
    win_mut(wp).w_nrwidth_line_count = val;
}

/// Sets `wp->w_statuscol_line_count`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_statuscol_line_count"]
pub unsafe extern "C" fn win_set_statuscol_line_count(wp: WinHandle, val: LineNr) {
    win_mut(wp).w_statuscol_line_count = val;
}

/// Sets `wp->w_p_crb`. Calls through to winopt_T setter.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_p_crb"]
pub unsafe extern "C" fn win_set_p_crb(wp: WinHandle, val: c_int) {
    win_mut(wp).set_w_p_crb(val);
}

/// Sets `wp->w_p_rl`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_p_rl"]
pub unsafe extern "C" fn win_set_p_rl(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).set_w_p_rl(val);
}

/// Sets `wp->w_p_sms`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_p_sms"]
pub unsafe extern "C" fn win_set_p_sms(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).set_w_p_sms(val);
}

/// Sets `wp->w_floating`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_floating"]
pub unsafe extern "C" fn win_set_floating(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_floating = val != 0;
}

/// Sets `wp->w_fraction`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_fraction"]
pub unsafe extern "C" fn win_set_fraction(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_fraction = val;
}

/// Sets `wp->w_prev_fraction_row`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_prev_fraction_row"]
pub unsafe extern "C" fn win_set_prev_fraction_row(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_prev_fraction_row = val;
}

/// Sets `wp->w_prev_height`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_prev_height"]
pub unsafe extern "C" fn win_set_prev_height(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_prev_height = val;
}

/// Sets `wp->w_prev_winrow`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_prev_winrow"]
pub unsafe extern "C" fn win_set_prev_winrow(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_prev_winrow = val;
}

/// Sets `wp->w_do_win_fix_cursor`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_do_win_fix_cursor"]
pub unsafe extern "C" fn win_set_do_win_fix_cursor(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_do_win_fix_cursor = val != 0;
}

/// Sets `wp->w_scbind_pos`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_scbind_pos"]
pub unsafe extern "C" fn win_set_scbind_pos(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_scbind_pos = val;
}

/// Sets `wp->w_filler_rows`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_filler_rows"]
pub unsafe extern "C" fn win_set_filler_rows(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_filler_rows = val;
}

/// Sets `wp->w_botfill`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_botfill"]
pub unsafe extern "C" fn win_set_botfill(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_botfill = val != 0;
}

/// Sets `wp->w_locked`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_locked"]
pub unsafe extern "C" fn win_set_locked(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_locked = val != 0;
}

/// Sets `wp->w_old_visual_mode`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_old_visual_mode"]
pub unsafe extern "C" fn win_set_old_visual_mode(wp: WinHandle, val: c_int) {
    win_mut(wp).w_old_visual_mode = (val & 0xff) as i8;
}

/// Sets `wp->w_old_cursor_lnum`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_old_cursor_lnum"]
pub unsafe extern "C" fn win_set_old_cursor_lnum(wp: WinHandle, val: LineNr) {
    win_mut(wp).w_old_cursor_lnum = val;
}

/// Sets `wp->w_old_visual_lnum`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_old_visual_lnum"]
pub unsafe extern "C" fn win_set_old_visual_lnum(wp: WinHandle, val: LineNr) {
    win_mut(wp).w_old_visual_lnum = val;
}

/// Sets `wp->w_old_visual_col`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_old_visual_col"]
pub unsafe extern "C" fn win_set_old_visual_col(wp: WinHandle, val: ColNr) {
    win_mut(wp).w_old_visual_col = val;
}

/// Sets `wp->w_cursorline`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_w_cursorline"]
pub unsafe extern "C" fn win_set_w_cursorline(wp: WinHandle, val: LineNr) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_cursorline = val;
}

/// Sets `wp->w_alt_fnum`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_alt_fnum"]
pub unsafe extern "C" fn win_set_alt_fnum(wp: WinHandle, val: c_int) {
    win_mut(wp).w_alt_fnum = val;
}

/// Sets `wp->w_changelistidx`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_changelistidx"]
pub unsafe extern "C" fn win_set_changelistidx(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_changelistidx = val;
}

/// Sets `wp->w_nrwidth`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_nrwidth"]
pub unsafe extern "C" fn win_set_nrwidth(wp: WinHandle, val: c_int) {
    win_mut(wp).w_nrwidth = val;
}

/// Sets `wp->w_frame`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_frame"]
pub unsafe extern "C" fn win_set_frame(wp: WinHandle, frp: *mut Frame) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_frame = frp;
}

/// Sets `wp->w_buffer`.
///
/// # Safety
/// `wp` must be a valid non-null `win_T*`.
#[export_name = "nvim_win_set_buffer_raw"]
pub unsafe extern "C" fn win_set_buffer_raw(wp: WinHandle, buf: *mut c_void) {
    win_mut(wp).w_buffer = buf;
}

/// Sets `wp->w_redr_border`. Does nothing if wp is null.
///
/// # Safety
/// `wp` must be a valid `win_T*` (may be null).
#[export_name = "nvim_win_set_redr_border"]
pub unsafe extern "C" fn win_set_redr_border(wp: WinHandle, val: c_int) {
    if wp.as_ptr().is_null() {
        return;
    }
    win_mut(wp).w_redr_border = val != 0;
}
