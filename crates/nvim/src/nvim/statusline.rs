//! The status line, the tab line, the winbar, the ruler and the status
//! column.
//!
//! Carved by what is being drawn:
//!
//! | child | what |
//! | --- | --- |
//! | [`status`] | the default status line and the click-definition arenas |
//! | [`custom`] | `'statusline'`/`'winbar'`/`'rulerformat'` rendering |
//! | [`tabline`] | `draw_tabline()` and its `ext_tabline` form |
//! | [`stl`] | `build_stl_str_hl()`, the `%` format language itself |
//!
//! What stays here is the `STL_*` item alphabet the format language is written
//! in, the `stl_item_t` kinds, `build_statuscol_str()` (`'statuscolumn'`, which
//! is the same language with a different item set), and the two small entry
//! points -- `fillchar_status()` and `redraw_custom_statusline()` -- the
//! drawing layer calls in.
//!
//! Original: `src/nvim/statusline.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::eval::vars::set_vim_var_nr;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight_group::{HLF_S, HLF_SNC};
use crate::src::nvim::main::curwin;
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::options::kOptStatuscolumn;
use crate::src::nvim::types::{
    AlignTextPos, Array, Object, OptValType, StlClickDefinition_type_0 as C2Rust_Unnamed_13,
    StlClickRecord, StlFlag, VV_LNUM, VV_RELNUM, WinSplit, WinStyle, hlf_T, linenr_T, schar_T,
    size_t, statuscol_T, varnumber_T, win_T,
};

// The carve of the transpiled module; see each child's docs.
mod custom;
mod status;
mod stl;
mod tabline;

pub use self::custom::*;
pub use self::status::*;
pub use self::stl::*;
pub use self::tabline::*;

pub const kStlClickFuncRun: C2Rust_Unnamed_13 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_13 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_13 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_13 = 0;
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitLeft: WinSplit = 0;
pub const kOptValTypeString: OptValType = 2;
/// Most sign columns `'signcolumn'` will ever ask for.
pub const SIGN_SHOW_MAX: ::core::ffi::c_int = 9;
pub const STL_CLICK_FUNC: StlFlag = 64;
pub const STL_TABCLOSENR: StlFlag = 88;
pub const STL_TABPAGENR: StlFlag = 84;
pub const STL_HIGHLIGHT_COMB: StlFlag = 36;
pub const STL_HIGHLIGHT: StlFlag = 35;
pub const STL_USER_HL: StlFlag = 42;
pub const STL_TRUNCMARK: StlFlag = 60;
pub const STL_SEPARATE: StlFlag = 61;
pub const STL_VIM_EXPR: StlFlag = 123;
pub const STL_SIGNCOL: StlFlag = 115;
pub const STL_FOLDCOL: StlFlag = 67;
pub const STL_SHOWCMD: StlFlag = 83;
pub const STL_PAGENUM: StlFlag = 78;
pub const STL_ARGLISTSTAT: StlFlag = 97;
pub const STL_ALTPERCENT: StlFlag = 80;
pub const STL_PERCENTAGE: StlFlag = 112;
pub const STL_QUICKFIX: StlFlag = 113;
pub const STL_MODIFIED_ALT: StlFlag = 77;
pub const STL_MODIFIED: StlFlag = 109;
pub const STL_PREVIEWFLAG_ALT: StlFlag = 87;
pub const STL_PREVIEWFLAG: StlFlag = 119;
pub const STL_FILETYPE_ALT: StlFlag = 89;
pub const STL_FILETYPE: StlFlag = 121;
pub const STL_HELPFLAG_ALT: StlFlag = 72;
pub const STL_HELPFLAG: StlFlag = 104;
pub const STL_ROFLAG_ALT: StlFlag = 82;
pub const STL_ROFLAG: StlFlag = 114;
pub const STL_BYTEVAL_X: StlFlag = 66;
pub const STL_BYTEVAL: StlFlag = 98;
pub const STL_OFFSET_X: StlFlag = 79;
pub const STL_OFFSET: StlFlag = 111;
pub const STL_KEYMAP: StlFlag = 107;
pub const STL_BUFNO: StlFlag = 110;
pub const STL_NUMLINES: StlFlag = 76;
pub const STL_LINE: StlFlag = 108;
pub const STL_VIRTCOL_ALT: StlFlag = 86;
pub const STL_VIRTCOL: StlFlag = 118;
pub const STL_COLUMN: StlFlag = 99;
pub const STL_FILENAME: StlFlag = 116;
pub const STL_FULLPATH: StlFlag = 70;
pub const STL_FILEPATH: StlFlag = 102;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stl_item {
    pub start: *mut ::core::ffi::c_char,
    pub cmd: *mut ::core::ffi::c_char,
    pub minwid: ::core::ffi::c_int,
    pub maxwid: ::core::ffi::c_int,
    pub type_0: C2Rust_Unnamed_15,
}
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const Trunc: C2Rust_Unnamed_15 = 10;
pub const ClickFunc: C2Rust_Unnamed_15 = 9;
pub const TabPage: C2Rust_Unnamed_15 = 8;
pub const HighlightFold: C2Rust_Unnamed_15 = 7;
pub const HighlightSign: C2Rust_Unnamed_15 = 6;
pub const HighlightCombining: C2Rust_Unnamed_15 = 5;
pub const Highlight: C2Rust_Unnamed_15 = 4;
pub const Separate: C2Rust_Unnamed_15 = 3;
pub const Group: C2Rust_Unnamed_15 = 2;
pub const Empty: C2Rust_Unnamed_15 = 1;
pub const Normal: C2Rust_Unnamed_15 = 0;
pub type stl_item_t = stl_item;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const OPT_LOCAL: C2Rust_Unnamed_17 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_17 = 1;
pub type NumberBase = ::core::ffi::c_uint;
pub const kNumBaseHexadecimal: NumberBase = 16;
pub const kNumBaseDecimal: NumberBase = 10;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const FR_COL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const EOL_MAC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const MAX_NUMBERWIDTH: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const SID_ERROR: ::core::ffi::c_int = -5 as ::core::ffi::c_int;
pub const MAX_STL_EVAL_DEPTH: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
pub const RULER_BUF_LEN: ::core::ffi::c_int = 70 as ::core::ffi::c_int;
pub unsafe extern "C" fn fillchar_status(mut group: *mut hlf_T, mut wp: *mut win_T) -> schar_T {
    unsafe {
        if wp == curwin.get() {
            *group = HLF_S;
            return (*wp).w_p_fcs_chars.stl;
        } else {
            *group = HLF_SNC;
            return (*wp).w_p_fcs_chars.stlnc;
        };
    }
}
pub unsafe extern "C" fn redraw_custom_statusline(mut wp: *mut win_T) {
    unsafe {
        static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        if entered.get() {
            return;
        }
        entered.set(true_0 != 0);
        win_redr_custom(wp, false_0 != 0, false_0 != 0, false_0 != 0);
        entered.set(false_0 != 0);
    }
}
pub unsafe extern "C" fn build_statuscol_str(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut relnum: linenr_T,
    mut buf: *mut ::core::ffi::c_char,
    mut stcp: *mut statuscol_T,
) -> ::core::ffi::c_int {
    unsafe {
        let fillclick: bool = relnum >= 0 as linenr_T
            && (*stcp).width > 0 as ::core::ffi::c_int
            && lnum == (*wp).w_topline;
        if relnum >= 0 as linenr_T {
            set_vim_var_nr(VV_LNUM, lnum as varnumber_T);
            set_vim_var_nr(VV_RELNUM, relnum as varnumber_T);
        }
        let mut clickrec: *mut StlClickRecord = ::core::ptr::null_mut::<StlClickRecord>();
        let mut stc: *mut ::core::ffi::c_char = xstrdup((*wp).w_onebuf_opt.wo_stc);
        let mut width: ::core::ffi::c_int = build_stl_str_hl(
            wp,
            buf,
            MAXPATHL as size_t,
            stc,
            kOptStatuscolumn,
            OPT_LOCAL as ::core::ffi::c_int,
            0 as schar_T,
            (*stcp).width,
            &raw mut (*stcp).hlrec,
            ::core::ptr::null_mut::<size_t>(),
            if fillclick as ::core::ffi::c_int != 0 {
                &raw mut clickrec
            } else {
                ::core::ptr::null_mut::<*mut StlClickRecord>()
            },
            stcp,
        );
        xfree(stc as *mut ::core::ffi::c_void);
        if fillclick {
            stl_clear_click_defs(
                (*wp).w_statuscol_click_defs,
                (*wp).w_statuscol_click_defs_size,
            );
            (*wp).w_statuscol_click_defs = stl_alloc_click_defs(
                (*wp).w_statuscol_click_defs,
                width,
                &raw mut (*wp).w_statuscol_click_defs_size,
            );
            stl_fill_click_defs(
                (*wp).w_statuscol_click_defs,
                clickrec,
                buf,
                width,
                false_0 != 0,
            );
        }
        return width;
    }
}
pub const TMPLEN: ::core::ffi::c_int = 70 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
