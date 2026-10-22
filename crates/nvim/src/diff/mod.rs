//! Diff mode: the block list, the engines that build it, and the answers
//! the drawer reads off it.
//!
//! The thirteen children hold the code; this file holds the surface they
//! share -- the owned `DiffIn`/`DiffOut`/`DiffIo` triple a diff run is
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
//! Everything a *window* knows about its diff lives on `Window`/`Tabpage`
//! instead (`w_p_diff`, `tp_diffbuf`, `tp_first_diff`, `tp_diff_invalid`),
//! so there is no per-window state here to narrow.

#![deny(unsafe_op_in_unsafe_fn)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

pub(crate) mod state;
use crate::ascii::ascii_iswhite;
use crate::autocmd::{
    apply_autocmds, aucmd_prepbuf, aucmd_restbuf, augroup_exists, block_autocmds, unblock_autocmds,
};
use crate::buffer::{buf_get_changedtick, buf_is_empty, buf_is_prompt, current_buf, find_buf};
use crate::bufwrite::{WriteName, WriteRequest, buf_write};
use crate::change::{change_warning, changed_lines};
use crate::charset::getdigits_int;
use crate::cursor::check_cursor;
use crate::decoration::decor_conceal_line;
use crate::drawscreen::{UPD_NOT_VALID, UPD_SOME_VALID, UPD_VALID, redraw_later};
use crate::eval::typval::{tv_get_lnum, tv_get_number};
use crate::eval::vars::{eval_diff, eval_patch};
use crate::ex_cmds::{append_redir, ex_file};
use crate::ex_docmd::{do_cmdline_cmd, do_exedit, get_address};
use crate::extmark::extmark_adjust;
use crate::fileio::{buf_check_timestamp, shorten_fnames, temp_name, vim_gettempdir};
use crate::fold::{
    fold_update, fold_update_all, foldmethod_is_diff, foldmethod_is_manual, new_fold_level,
};

use crate::diff::state::{diff_context, diff_foldcolumn, diff_need_scrollbind, need_diff_redraw};
use crate::getchar::state::KeyTyped;
use crate::global_cell::GlobalCell;
use crate::highlight_group::{HLF_ADD, HLF_CHD, HLF_NONE, HLF_TXA, HLF_TXD};
use crate::linematch::linematch_nbuffers;
use crate::mark::{mark_adjust, setpcmark};
use crate::mbyte::{
    head_off, mb_get_class_tab, utf_char2bytes, utf_char2len, utf_fold, utf_ptr2char, utfc_ptr2len,
};
use crate::memline::{Lines, ml_append_cstr, ml_delete};
use crate::memory::{XString, xfree};
use crate::message::emsg;
use crate::message::{
    e_diff_anchors_with_hidden_windows, e_failed_to_find_all_diff_anchors, e_invrange, e_prev_dir,
    e_problem_creating_internal_diff,
};
use crate::r#move::{
    changed_line_abv_curs, changed_line_abv_curs_win, changed_window_setting,
    invalidate_botline_win, validate_cursor,
};
use crate::normal::check_scrollbind;
use crate::option::vars::{p_dex, p_sbo, p_srr};
use crate::option::{OptionTarget, set_option_direct_for, set_option_value_give_err};
use crate::options::{kOptBoFlagOperator, kOptDiff, kOptFoldmethod};
use crate::os::cshim::gettext;
use crate::os::env::{os_env_exists, os_unsetenv};
use crate::os::fs::{os_chdir, os_dirname, os_fileinfo, os_fileinfo_size, os_remove};
use crate::os::shell::{ShellOpts, call_shell};
use crate::pos::{MAXCOL, MAXLNUM};
use crate::search::{BACKWARD, FORWARD};
use crate::strings::shellescape;
use crate::types::AutoEvent;
use crate::types::NL;
use crate::types::{
    AcoSave, CmdAddr, CmdModFlags, ColNr, DiffBlock, DiffLine, DiffLineChange, EvalFuncData, ExArg,
    ExtmarkOp, FileInfo, Hlf, LineNr, OptInt, OptScope, OptVal, ScriptId, String_0, TypVal,
    VarNumber, size_t, uint64_t,
};
use crate::ui::vim_beep;
use crate::undo::{u_save, u_sync};
use crate::window::{
    WSP_VERT, frames_locked, scroll_to_fraction, set_fraction, win_split, win_valid,
};
use crate::xdiff::xtypes::{
    XDF_HISTOGRAM_DIFF, XDF_IGNORE_BLANK_LINES, XDF_IGNORE_WHITESPACE,
    XDF_IGNORE_WHITESPACE_AT_EOL, XDF_IGNORE_WHITESPACE_CHANGE, XDF_INDENT_HEURISTIC,
    XDF_NEED_MINIMAL, XDF_PATIENCE_DIFF,
};
use ::libc::tolower;

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
pub const _ISdigit: ::core::ffi::c_uint = 2048;
pub const kOptScopeWin: OptScope = 1;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
/// One diff run: the two sides and the answer.
///
/// `internal` picks the engine: the built-in one reads each side's `text`
/// and fills `diff.hunks`; the external one (or `'diffexpr'`) reads the temp
/// files named by each side's `fname` and writes the one `diff.fname` names.
#[derive(Default)]
pub struct DiffIo {
    pub orig: DiffIn,
    pub new: DiffIn,
    pub diff: DiffOut,
    pub internal: bool,
}
/// An external run's temp files: the two sides, then the output.
pub(crate) type TempNames<'a> = [&'a core::ffi::CStr; 3];

impl DiffIo {
    /// The three temp file names, once all three are made.
    pub(crate) fn temp_names(&self) -> Option<TempNames<'_>> {
        Some([
            self.orig.fname.as_ref()?.as_cstr(),
            self.new.fname.as_ref()?.as_cstr(),
            self.diff.fname.as_ref()?.as_cstr(),
        ])
    }
}
#[derive(Default)]
pub struct DiffOut {
    /// The temp file the external diff writes; `None` for the internal one.
    pub fname: Option<XString>,
    /// The hunks the internal engine produced; empty when the run went
    /// through a temp file named by `fname` instead.
    pub hunks: Vec<DiffHunk>,
}
#[derive(Default)]
pub struct DiffIn {
    /// The temp file the buffer is written to; `None` for the internal
    /// engine, which reads `text`.
    pub fname: Option<XString>,
    /// The buffer's lines as one NL-terminated memory image.
    pub text: Vec<u8>,
}
#[derive(Copy, Clone)]
pub struct DiffHunk {
    pub lnum_orig: LineNr,
    pub count_orig: ::core::ffi::c_int,
    pub lnum_new: LineNr,
    pub count_new: ::core::ffi::c_int,
}
pub const MAX_DIFF_ANCHORS: ::core::ffi::c_int = 20;
pub struct LinemapEntry {
    pub byte_start: ColNr,
    pub num_bytes: ColNr,
    pub lineoff: ::core::ffi::c_int,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DB_COUNT: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
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
