#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;
use crate::memory::XString;
use crate::r#move::WinValid;
use crate::registry::IdSet;
use crate::types::Failed;
use crate::winlayer::{FrameId, TabId, WinId};

pub struct SwitchWin {
    /// The window and tab page to go back to. Handles: `switch_win` runs
    /// user code between the save and the restore, and that code can close
    /// either of them.
    pub(crate) sw_curwin: Option<WinId>,
    pub(crate) sw_curtab: Option<TabId>,
    pub sw_same_win: bool,
    pub sw_visual_active: bool,
}
pub struct WinExecute {
    /// The window the command ran in, as a handle: it may be gone by the
    /// time `win_execute_after` looks.
    pub(crate) wp: Option<WinId>,
    pub curpos: Pos,
    pub cwd: [::core::ffi::c_char; 4096],
    pub cwd_status: Result<(), Failed>,
    pub apply_acd: bool,
    pub save_sfname: *mut ::core::ffi::c_char,
    pub switchwin: SwitchWin,
}

impl Default for SwitchWin {
    /// The zeroed state a caller declares before handing it to `switch_win`,
    /// which fills every field. Nothing reads one of these before that.
    fn default() -> Self {
        SwitchWin {
            sw_curwin: None,
            sw_curtab: None,
            sw_same_win: false,
            sw_visual_active: false,
        }
    }
}

impl Default for WinExecute {
    /// The zeroed state a caller declares before handing it to
    /// `win_execute_before`, which fills what it needs and leaves the rest --
    /// `cwd` in particular is only written when 'autochdir' is on.
    fn default() -> Self {
        WinExecute {
            wp: None,
            curpos: Pos {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            cwd: [0; 4096],
            cwd_status: Err(Failed),
            apply_acd: false,
            save_sfname: ::core::ptr::null_mut(),
            switchwin: SwitchWin::default(),
        }
    }
}

/// Neither `Copy` nor `Clone`: a frame is a *node* of the window layout
/// tree, and duplicating one would put a second node into a structure the
/// editor walks. They are allocated one at a time by `winlayer::new_frame`
/// and freed by `free_frame`; the absence of the derives is what says so.
pub struct Frame {
    /// This frame's key in the frame registry; `winlayer::FrameId` names it.
    pub handle: Handle,
    pub fr_layout: ::core::ffi::c_char,
    pub fr_width: ::core::ffi::c_int,
    pub fr_newwidth: ::core::ffi::c_int,
    pub fr_height: ::core::ffi::c_int,
    pub fr_newheight: ::core::ffi::c_int,
    /// The four edges of the layout tree, as identities: the frame this one
    /// hangs off, its neighbours in that row or column, and its first child.
    /// `winlayer::FrameRef` resolves them; a freed frame's link reads `None`.
    pub(crate) fr_parent: Option<FrameId>,
    pub(crate) fr_next: Option<FrameId>,
    pub(crate) fr_prev: Option<FrameId>,
    pub(crate) fr_child: Option<FrameId>,
    /// The window a leaf frame holds, `None` for a row or a column. An
    /// identity: a layout snapshot outlives the windows it remembers.
    pub(crate) fr_win: Option<WinId>,
}
/// Deliberately neither `Copy` nor `Clone`: a tab page owns three
/// allocations (`tp_vars`, `tp_localdir`, `tp_diffbuf`'s memlines) and a
/// window list, so a bitwise copy would be a second owner of all of them.
/// Tab pages are reached through `winlayer::TabPage`, which is the `Copy`
/// handle naming this one.
pub struct Tabpage {
    pub handle: Handle,
    /// The tab page list off `first_tabpage`. A handle, as the buffer
    /// list's links are — `winlayer::TabPage::next` and `winlayer::tabs`
    /// are how it is walked.
    pub(crate) tp_next: Option<TabId>,
    pub(crate) tp_topframe: Option<FrameId>,
    /// The window this tab page was last working in, and the one before it.
    /// Handles, as its window list's ends are: both are read *after* a call
    /// that can close a window. **Stale while the tab page is current**,
    /// exactly as `tp_firstwin` is.
    pub(crate) tp_curwin: Option<WinId>,
    pub(crate) tp_prevwin: Option<WinId>,
    /// This tab page's window list, its two ends. Handles, as the links
    /// between them are. **Stale while the tab page is the current one** —
    /// the `firstwin`/`lastwin` globals are then the truth, which is what
    /// `winlayer::windows_in_tab` encodes.
    ///
    pub(crate) tp_firstwin: Option<WinId>,
    pub(crate) tp_lastwin: Option<WinId>,
    pub tp_old_rows_avail: int64_t,
    pub tp_old_columns: int64_t,
    pub tp_ch_used: OptInt,
    pub tp_did_tabclosedpre: bool,
    pub tp_first_diff: *mut DiffBlock,
    pub tp_diffbuf: [*mut Buffer; 8],
    pub tp_diff_invalid: ::core::ffi::c_int,
    pub tp_diff_update: ::core::ffi::c_int,
    pub(crate) tp_snapshot: [Option<FrameId>; 3],
    pub tp_winvar: ScopeDictItem,
    pub tp_vars: *mut Dict,
    pub tp_localdir: *mut ::core::ffi::c_char,
    pub tp_prevdir: *mut ::core::ffi::c_char,
}
/// Neither `Copy` nor `Clone`. A window owns its buffer list entry, its
/// grid, its option strings, its tag stack and its jump list; nothing in
/// the tree may duplicate one, and the absence of the derives is what says
/// so.
pub struct Window {
    pub handle: Handle,
    pub w_buffer: *mut Buffer,
    pub w_s: *mut SynBlock,
    pub w_ns_hl: ::core::ffi::c_int,
    pub w_ns_hl_winhl: ::core::ffi::c_int,
    pub w_ns_hl_active: ::core::ffi::c_int,
    pub w_ns_hl_attr: *mut ::core::ffi::c_int,
    /// The namespaces this window shows, when they are window-local.
    /// Membership only.
    pub(crate) w_ns_set: IdSet<uint32_t>,
    pub w_hl_id_normal: ::core::ffi::c_int,
    pub w_hl_attr_normal: ::core::ffi::c_int,
    pub w_hl_attr_normalnc: ::core::ffi::c_int,
    /// Whether the window's highlight namespace has to be re-resolved
    /// before it is next drawn.
    pub w_hl_needs_update: bool,
    /// This tab page's window list. Handles rather than addresses, as the
    /// buffer and tab page lists' links are: `winlayer::Win::next`/`prev`
    /// and `winlayer::windows`/`windows_back` are how they are walked.
    ///
    /// A window is registered from `win_alloc` to `win_free`, so a link can
    /// always be resolved -- with one hole the tree keeps: the autocommand
    /// window is *unregistered while idle*, and `aucmd_prepbuf` therefore
    /// puts it back in the registry before `win_append` files it here.
    pub(crate) w_prev: Option<WinId>,
    pub(crate) w_next: Option<WinId>,
    pub w_locked: bool,
    /// The leaf frame this window sits in. An identity: `winframe_remove`
    /// frees a frame under whoever held it. `winlayer::Win::frame` resolves it.
    pub(crate) w_frame: Option<FrameId>,
    pub w_cursor: Pos,
    pub w_curswant: ColNr,
    /// Whether the next cursor move should recompute `w_curswant` — the
    /// column a vertical move aims for — rather than keep the one the last
    /// horizontal move set.
    pub w_set_curswant: bool,
    pub w_cursorline: LineNr,
    pub w_last_cursorline: LineNr,
    pub w_old_visual_mode: ::core::ffi::c_char,
    pub w_old_cursor_lnum: LineNr,
    pub w_old_cursor_fcol: ColNr,
    pub w_old_cursor_lcol: ColNr,
    pub w_old_visual_lnum: LineNr,
    pub w_old_visual_col: ColNr,
    pub w_old_curswant: ColNr,
    pub w_last_cursor_lnum_rnu: LineNr,
    pub w_p_lcs_chars: LcsChars,
    pub w_p_fcs_chars: FcsChars,
    pub w_topline: LineNr,
    /// Whether `w_topline` was set on purpose rather than left at its
    /// default, which decides whether entering the buffer may move it.
    pub w_topline_was_set: bool,
    pub w_topfill: ::core::ffi::c_int,
    pub w_old_topfill: ::core::ffi::c_int,
    pub w_botfill: bool,
    pub w_old_botfill: bool,
    pub w_leftcol: ColNr,
    pub w_skipcol: ColNr,
    pub w_last_topline: LineNr,
    pub w_last_topfill: ::core::ffi::c_int,
    pub w_last_leftcol: ColNr,
    pub w_last_skipcol: ColNr,
    pub w_last_width: ::core::ffi::c_int,
    pub w_last_height: ::core::ffi::c_int,
    pub w_winrow: ::core::ffi::c_int,
    pub w_height: ::core::ffi::c_int,
    pub w_prev_winrow: ::core::ffi::c_int,
    pub w_prev_height: ::core::ffi::c_int,
    pub w_status_height: ::core::ffi::c_int,
    pub w_winbar_height: ::core::ffi::c_int,
    pub w_wincol: ::core::ffi::c_int,
    pub w_width: ::core::ffi::c_int,
    pub w_hsep_height: ::core::ffi::c_int,
    pub w_vsep_width: ::core::ffi::c_int,
    pub w_save_cursor: PosSave,
    pub w_do_win_fix_cursor: bool,
    pub w_winrow_off: ::core::ffi::c_int,
    pub w_wincol_off: ::core::ffi::c_int,
    pub w_view_height: ::core::ffi::c_int,
    pub w_view_width: ::core::ffi::c_int,
    pub w_height_request: ::core::ffi::c_int,
    pub w_width_request: ::core::ffi::c_int,
    pub w_border_adj: [::core::ffi::c_int; 4],
    pub w_height_outer: ::core::ffi::c_int,
    pub w_width_outer: ::core::ffi::c_int,
    pub w_valid: WinValid,
    pub w_valid_cursor: Pos,
    pub w_valid_leftcol: ColNr,
    pub w_valid_skipcol: ColNr,
    pub w_viewport_invalid: bool,
    pub w_viewport_last_topline: LineNr,
    pub w_viewport_last_botline: LineNr,
    pub w_viewport_last_topfill: LineNr,
    pub w_viewport_last_skipcol: LineNr,
    pub w_cline_height: ::core::ffi::c_int,
    pub w_cline_folded: bool,
    pub w_cline_row: ::core::ffi::c_int,
    pub w_virtcol: ColNr,
    pub w_wrow: ::core::ffi::c_int,
    pub w_wcol: ::core::ffi::c_int,
    pub w_botline: LineNr,
    pub w_empty_rows: ::core::ffi::c_int,
    pub w_filler_rows: ::core::ffi::c_int,
    pub w_lines_valid: ::core::ffi::c_int,
    pub w_lines: *mut WLine,
    pub w_lines_size: ::core::ffi::c_int,
    pub w_folds: GArray,
    pub w_fold_manual: bool,
    pub w_foldinvalid: bool,
    pub w_nrwidth: ::core::ffi::c_int,
    pub w_scwidth: ::core::ffi::c_int,
    pub w_minscwidth: ::core::ffi::c_int,
    pub w_maxscwidth: ::core::ffi::c_int,
    pub w_redr_type: ::core::ffi::c_int,
    pub w_upd_rows: ::core::ffi::c_int,
    pub w_redraw_top: LineNr,
    pub w_redraw_bot: LineNr,
    pub w_redr_status: bool,
    pub w_redr_border: bool,
    pub w_redr_statuscol: bool,
    pub w_display_tick: DispTick,
    pub w_stl_cursor: Pos,
    pub w_stl_virtcol: ColNr,
    pub w_stl_topline: LineNr,
    pub w_stl_line_count: LineNr,
    pub w_stl_topfill: ::core::ffi::c_int,
    /// Whether the last line drawn in the window was empty, as the status
    /// line last saw it.
    pub w_stl_empty: bool,
    pub w_stl_recording: ::core::ffi::c_int,
    pub w_stl_state: ::core::ffi::c_int,
    pub w_stl_visual_mode: ::core::ffi::c_int,
    pub w_stl_visual_pos: Pos,
    pub w_alt_fnum: ::core::ffi::c_int,
    pub w_alist: *mut ArgList,
    pub w_arg_idx: ::core::ffi::c_int,
    /// Whether `w_arg_idx` no longer names the argument the window shows.
    pub w_arg_idx_invalid: bool,
    pub w_localdir: *mut ::core::ffi::c_char,
    pub w_prevdir: *mut ::core::ffi::c_char,
    pub w_onebuf_opt: WinOpt,
    pub w_allbuf_opt: WinOpt,
    pub w_p_cc_cols: *mut ::core::ffi::c_int,
    pub w_p_culopt_flags: uint8_t,
    pub w_briopt_min: ::core::ffi::c_int,
    pub w_briopt_shift: ::core::ffi::c_int,
    pub w_briopt_sbr: bool,
    pub w_briopt_list: ::core::ffi::c_int,
    pub w_briopt_vcol: ::core::ffi::c_int,
    pub w_scbind_pos: ::core::ffi::c_int,
    pub w_winvar: ScopeDictItem,
    pub w_vars: *mut Dict,
    pub w_pcmark: Pos,
    pub w_prev_pcmark: Pos,
    pub w_jumplist: [XFileMark; 100],
    pub w_jumplistlen: ::core::ffi::c_int,
    pub w_jumplistidx: ::core::ffi::c_int,
    pub w_changelistidx: ::core::ffi::c_int,
    pub w_match_head: *mut MatchItem,
    pub w_next_match_id: ::core::ffi::c_int,
    pub w_tagstack: [Taggy; 20],
    pub w_tagstackidx: ::core::ffi::c_int,
    pub w_tagstacklen: ::core::ffi::c_int,
    pub w_grid: GridView,
    pub w_grid_alloc: ScreenGrid,
    pub w_pos_changed: bool,
    pub w_floating: bool,
    pub w_float_is_info: bool,
    pub w_config: WinConfig,
    pub w_fraction: ::core::ffi::c_int,
    pub w_prev_fraction_row: ::core::ffi::c_int,
    pub w_nrwidth_line_count: LineNr,
    pub w_statuscol_line_count: LineNr,
    pub w_nrwidth_width: ::core::ffi::c_int,
    pub w_llist: *mut QfInfo,
    pub w_llist_ref: *mut QfInfo,
    pub w_status_click_defs: *mut StlClickDefinition,
    pub w_status_click_defs_size: size_t,
    pub w_winbar_click_defs: *mut StlClickDefinition,
    pub w_winbar_click_defs_size: size_t,
    pub w_statuscol_click_defs: *mut StlClickDefinition,
    pub w_statuscol_click_defs_size: size_t,
}
#[derive(Clone)]
pub struct WinInfo {
    pub wi_win: *mut Window,
    pub wi_mark: FileMark,
    pub wi_optset: bool,
    pub wi_opt: WinOpt,
    pub wi_fold_manual: bool,
    pub wi_folds: GArray,
    pub wi_changelistidx: ::core::ffi::c_int,
}
/// Not `Copy`: the string options in here own their bytes, and
/// `copy_winopt` exists precisely to duplicate them -- the derived `Clone`
/// is a deep copy of exactly those, which is why `copy_winopt` can be
/// field-by-field rather than a `write`.
///
/// `None` in a string field is upstream's shared empty string: the window
/// has no value of its own there. A window is allocated zeroed and freed
/// with `xfree`, so `window::alloc`'s `clear_options` is what releases these
/// -- there is no destructor to lean on.
#[derive(Clone)]
pub struct WinOpt {
    pub wo_arab: ::core::ffi::c_int,
    pub wo_bri: ::core::ffi::c_int,
    pub wo_briopt: Option<XString>,
    pub wo_diff: ::core::ffi::c_int,
    pub wo_fdc: Option<XString>,
    pub wo_eiw: Option<XString>,
    pub wo_fdc_save: Option<XString>,
    pub wo_fen: ::core::ffi::c_int,
    pub wo_fen_save: ::core::ffi::c_int,
    pub wo_fdi: Option<XString>,
    pub wo_fdl: OptInt,
    pub wo_fdl_save: OptInt,
    pub wo_fdm: Option<XString>,
    pub wo_fdm_save: Option<XString>,
    pub wo_fml: OptInt,
    pub wo_fdn: OptInt,
    pub wo_fde: Option<XString>,
    pub wo_fdt: Option<XString>,
    pub wo_fmr: Option<XString>,
    pub wo_lbr: ::core::ffi::c_int,
    pub wo_list: ::core::ffi::c_int,
    pub wo_nu: ::core::ffi::c_int,
    pub wo_rnu: ::core::ffi::c_int,
    pub wo_ve: Option<XString>,
    pub wo_ve_flags: ::core::ffi::c_uint,
    pub wo_nuw: OptInt,
    pub wo_wfb: ::core::ffi::c_int,
    pub wo_wfh: ::core::ffi::c_int,
    pub wo_wfw: ::core::ffi::c_int,
    pub wo_pvw: ::core::ffi::c_int,
    pub wo_lhi: OptInt,
    pub wo_rl: ::core::ffi::c_int,
    pub wo_rlc: Option<XString>,
    pub wo_scr: OptInt,
    pub wo_sms: ::core::ffi::c_int,
    pub wo_spell: ::core::ffi::c_int,
    pub wo_cuc: ::core::ffi::c_int,
    pub wo_cul: ::core::ffi::c_int,
    pub wo_culopt: Option<XString>,
    pub wo_cc: Option<XString>,
    pub wo_sbr: Option<XString>,
    pub wo_stc: Option<XString>,
    pub wo_stl: Option<XString>,
    pub wo_wbr: Option<XString>,
    pub wo_scb: ::core::ffi::c_int,
    pub wo_diff_saved: ::core::ffi::c_int,
    pub wo_scb_save: ::core::ffi::c_int,
    pub wo_wrap: ::core::ffi::c_int,
    pub wo_wrap_save: ::core::ffi::c_int,
    pub wo_cocu: Option<XString>,
    pub wo_cole: OptInt,
    pub wo_crb: ::core::ffi::c_int,
    pub wo_crb_save: ::core::ffi::c_int,
    pub wo_scl: Option<XString>,
    pub wo_siso: OptInt,
    pub wo_so: OptInt,
    pub wo_winhl: Option<XString>,
    pub wo_lcs: Option<XString>,
    pub wo_fcs: Option<XString>,
    pub wo_winbl: OptInt,
    pub wo_wrap_flags: uint32_t,
    pub wo_stl_flags: uint32_t,
    pub wo_wbr_flags: uint32_t,
    pub wo_fde_flags: uint32_t,
    pub wo_fdt_flags: uint32_t,
    pub wo_script_ctx: [ScriptCtx; 51],
}
#[derive(Copy, Clone)]
pub struct WLine {
    pub wl_lnum: LineNr,
    pub wl_size: uint16_t,
    pub wl_valid: bool,
    pub wl_folded: bool,
    pub wl_foldend: LineNr,
    pub wl_lastlnum: LineNr,
}
