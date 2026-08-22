//! Diff mode: the block list, the engines that build it, and the answers
//! the drawer reads off it.
//!
//! The thirteen children hold the code; this file holds the surface they
//! share -- the `diffin_T`/`diffout_T`/`diffio_T` triple a diff run is
//! carried in, the `DIFF_*` bits of `'diffopt'`, and the module's
//! **process-wide state**, which is seven cells and nothing else:
//!
//! | cell | written by | meaning |
//! | --- | --- | --- |
//! | `diff_flags` | `diffopt_changed` | the parsed `'diffopt'` bit set |
//! | `diff_algorithm` | `diffopt_changed` | the `XDF_*` bits for `algorithm:` |
//! | `linematch_lines` | `diffopt_changed` | `linematch:`, 0 when off |
//! | `diff_a_works` | `check_external_diff` | does the host's `diff(1)` take `-a`? |
//! | `diff_busy` | `diff_try_update` | a recompute is on the stack |
//! | `diff_need_update` | `diff_buf_*`, `ex_diffupdate` | one was asked for while busy |
//! | `simple_diffline_change` | `diff_find_change_simple` | the one-element answer buffer |
//!
//! The first four are option state and are only interesting because
//! `'diffopt'` is global; the last three are the ones that carry state
//! *across* calls. `diff_busy`/`diff_need_update` are a reentrancy guard:
//! `diff_try_update` runs autocommands, which can ask for another
//! recompute, and the pair defers it to the outer call's tail.
//! `simple_diffline_change` is a returned-pointer buffer, alive only until
//! the next `diff_find_change` -- the one cell whose lifetime is a real
//! obligation rather than a global option.
//!
//! Everything a *window* knows about its diff lives on `win_T`/`tabpage_T`
//! instead (`w_p_diff`, `tp_diffbuf`, `tp_first_diff`, `tp_diff_invalid`),
//! so there is no per-window state here to narrow.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::autocmd::{
    EVENT_DIFFUPDATED, apply_autocmds, aucmd_prepbuf, aucmd_restbuf, augroup_exists,
    block_autocmds, unblock_autocmds,
};
use crate::buffer::{
    bt_prompt, buf_get_changedtick, buf_is_empty, buf_valid, buflist_findnr, buflist_findpat,
    bufref_valid, set_bufref,
};
use crate::bufwrite::{WriteRequest, buf_write};
use crate::change::{change_warning, changed_lines};
use crate::charset::{getdigits_int, getdigits_int32};
use crate::cursor::check_cursor;
use crate::decoration::decor_conceal_line;
use crate::drawscreen::{UPD_NOT_VALID, UPD_SOME_VALID, UPD_VALID, redraw_later};
use crate::eval::typval::{tv_get_lnum, tv_get_number};
use crate::eval::vars::{eval_diff, eval_patch};
use crate::ex_cmds::{append_redir, ex_file};
use crate::ex_docmd::{do_cmdline_cmd, do_exedit, get_address};
use crate::extmark::extmark_adjust;
use crate::fileio::{buf_check_timestamp, shorten_fnames, vim_fgets, vim_gettempdir, vim_tempname};
use crate::fold::{
    fold_update, fold_update_all, foldmethod_is_diff, foldmethod_is_manual, has_folding,
    new_fold_level,
};
use crate::garray::{ga_clear, ga_grow, ga_init};
use crate::global_cell::GlobalCell;
use crate::highlight_group::{HLF_ADD, HLF_CHD, HLF_NONE, HLF_TXA, HLF_TXD};
use crate::linematch::linematch_nbuffers;
use crate::main::{
    KeyTyped, cmdmod, curbuf, curtab, curwin, diff_context, diff_foldcolumn, diff_need_scrollbind,
    e_cannot_have_more_than_nr_diff_anchors, e_diff_anchors_with_hidden_windows,
    e_failed_to_find_all_diff_anchors, e_invrange, e_prev_dir, e_problem_creating_internal_diff,
    first_tabpage, firstwin, need_diff_redraw, p_dex, p_dia, p_dip, p_pex, p_sbo, p_srr,
};
use crate::mark::{mark_adjust, setpcmark};
use crate::mbyte::{
    mb_get_class_tab, mb_stricmp, utf_char2bytes, utf_char2len, utf_fold, utf_head_off,
    utf_ptr2char, utfc_ptr2len,
};
use crate::memline::{ml_append, ml_delete, ml_get_buf, ml_get_buf_len};
use crate::memory::{memchrsub, xcalloc, xfree, xmalloc, xstrdup};
use crate::message::emsg;
use crate::r#move::{
    changed_line_abv_curs, changed_line_abv_curs_win, changed_window_setting, check_topfill,
    invalidate_botline_win, validate_cursor,
};
use crate::normal::check_scrollbind;
use crate::option::{set_option_direct_for, set_option_value_give_err};
use crate::options::{kOptBoFlagOperator, kOptDiff, kOptFoldmethod};
use crate::optionstr::free_string_option;
use crate::os::cshim::{gettext, snprintf, strncmp};
use crate::os::env::{os_env_exists, os_unsetenv};
use crate::os::fs::{os_chdir, os_dirname, os_fileinfo, os_fileinfo_size, os_fopen, os_remove};
use crate::os::shell::{ShellOpts, call_shell};
use crate::path::full_name_save;
use crate::pos::{MAXCOL, MAXLNUM};
use crate::search::{BACKWARD, FORWARD};
use crate::strings::{vim_snprintf, vim_strchr, vim_strsave_shellescape, xstrnsave};
use crate::types::{
    CMD_append, CMD_diffget, CMD_diffput, CMD_split, CmdAddr, CmdModFlags, EvalFuncData, ExtmarkOp,
    FILE, FileInfo, OptInt, OptScope, OptVal, OptValData, OptValType, String_0, aco_save_T, buf_T,
    bufref_T, colnr_T, diff_T, diffblock_S, diffline_S, diffline_T, diffline_change_T, exarg_T,
    garray_T, hlf_T, linenr_T, mmfile_t, scid_T, size_t, tabpage_T, typval_T, uint64_t,
    varnumber_T, win_T, xdemitcb_t, xdemitconf_t, xpparam_t,
};
use crate::ui::vim_beep;
use crate::undo::{u_save, u_sync};
use crate::window::{
    WSP_VERT, frames_locked, scroll_to_fraction, set_fraction, win_split, win_valid,
};
use crate::xdiff::ffi::xdl_diff;
use crate::xdiff::xtypes::{
    XDF_HISTOGRAM_DIFF, XDF_IGNORE_BLANK_LINES, XDF_IGNORE_WHITESPACE,
    XDF_IGNORE_WHITESPACE_AT_EOL, XDF_IGNORE_WHITESPACE_CHANGE, XDF_INDENT_HEURISTIC,
    XDF_NEED_MINIMAL, XDF_PATIENCE_DIFF,
};
use ::libc::{atol, fclose, fwrite, strcat, strcpy, strlen, tolower};

// The carve of the transpiled module; see each child's docs.
mod block;
mod compare;
mod engine;
mod excmd;
mod fold;
mod getput;
mod hunk;
mod inline;
mod lmatch;
mod opts;
mod refine;
mod scroll;
mod update;

pub use self::block::*;
pub(crate) use self::compare::*;
pub(crate) use self::engine::*;
pub use self::excmd::*;
pub use self::fold::*;
pub use self::getput::*;
pub(crate) use self::hunk::*;
pub use self::inline::*;
pub use self::lmatch::*;
pub use self::opts::*;
pub(crate) use self::refine::*;
pub use self::scroll::*;
pub use self::update::*;

pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptScopeWin: OptScope = 1;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
/// An empty growable array, the shape c2rust writes out at every site.
pub(crate) const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 0,
    ga_data: ::core::ptr::null_mut(),
};

/// An empty memory image: no bytes, no allocation.
pub(crate) const MMFILE_INIT: mmfile_t = mmfile_t {
    ptr: ::core::ptr::null_mut(),
    size: 0,
};

/// One side of a diff before either the file name or the memory image is
/// filled in; which one is used depends on `dio_internal`.
pub(crate) const DIFFIN_INIT: diffin_T = diffin_T {
    din_fname: ::core::ptr::null_mut(),
    din_mmfile: MMFILE_INIT,
};

pub struct diffio_T {
    pub dio_orig: diffin_T,
    pub dio_new: diffin_T,
    pub dio_diff: diffout_T,
    pub dio_internal: ::core::ffi::c_int,
}
pub struct diffout_T {
    pub dout_fname: *mut ::core::ffi::c_char,
    pub dout_ga: garray_T,
}
pub struct diffin_T {
    pub din_fname: *mut ::core::ffi::c_char,
    pub din_mmfile: mmfile_t,
}
#[derive(Copy, Clone)]
pub struct diffhunk_T {
    pub lnum_orig: linenr_T,
    pub count_orig: ::core::ffi::c_int,
    pub lnum_new: linenr_T,
    pub count_new: ::core::ffi::c_int,
}
pub const MAX_DIFF_ANCHORS: ::core::ffi::c_int = 20;
#[derive(Copy, Clone)]
pub struct linemap_entry_T {
    pub byte_start: colnr_T,
    pub num_bytes: colnr_T,
    pub lineoff: ::core::ffi::c_int,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const DB_COUNT: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
static diff_busy: GlobalCell<bool> = GlobalCell::new(false);
static diff_need_update: GlobalCell<bool> = GlobalCell::new(false);
pub const DIFF_FILLER: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const DIFF_IBLANK: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const DIFF_ICASE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const DIFF_IWHITE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const DIFF_IWHITEALL: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const DIFF_IWHITEEOL: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const DIFF_HORIZONTAL: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const DIFF_VERTICAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const DIFF_HIDDEN_OFF: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const DIFF_INTERNAL: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const DIFF_CLOSE_OFF: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const DIFF_FOLLOWWRAP: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const DIFF_LINEMATCH: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const DIFF_INLINE_NONE: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const DIFF_INLINE_SIMPLE: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const DIFF_INLINE_CHAR: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
pub const DIFF_INLINE_WORD: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const DIFF_ANCHOR: ::core::ffi::c_int = 0x20000 as ::core::ffi::c_int;
pub const ALL_WHITE_DIFF: ::core::ffi::c_int = DIFF_IWHITE | DIFF_IWHITEALL | DIFF_IWHITEEOL;
pub const ALL_INLINE: ::core::ffi::c_int =
    DIFF_INLINE_NONE | DIFF_INLINE_SIMPLE | DIFF_INLINE_CHAR | DIFF_INLINE_WORD;
pub const ALL_INLINE_DIFF: ::core::ffi::c_int = DIFF_INLINE_CHAR | DIFF_INLINE_WORD;
static diff_flags: GlobalCell<::core::ffi::c_int> = GlobalCell::new(
    DIFF_INTERNAL | DIFF_FILLER | DIFF_CLOSE_OFF | DIFF_LINEMATCH | DIFF_INLINE_CHAR,
);
static diff_algorithm: GlobalCell<u64> = GlobalCell::new(XDF_INDENT_HEURISTIC);
/// `inline:word`'s gap threshold, in bytes.
///
/// A `static int` upstream, with no setter: `'diffopt'` has no spelling that
/// reaches it, so it is a tuning constant rather than option state and does
/// not belong in a cell.
const DIFF_WORD_GAP: ::core::ffi::c_int = 5;
static linematch_lines: GlobalCell<::core::ffi::c_int> = GlobalCell::new(40 as ::core::ffi::c_int);
pub const LBUFLEN: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
pub const MAX_XDIFF_SIZE: ::core::ffi::c_long =
    1024 as ::core::ffi::c_long * 1024 as ::core::ffi::c_long * 1023 as ::core::ffi::c_long;
static diff_a_works: GlobalCell<Option<bool>> = GlobalCell::new(None);
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
