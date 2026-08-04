//! Searching.
//!
//! This module is the root of the family and holds nothing but the
//! namespace its children share: [`pattern`] remembers the patterns,
//! [`find`] runs one over the buffer, [`command`] is `/` and `?`,
//! [`charsearch`] is `f`/`t`, [`select`] is `gn`, [`stat`] is the
//! `[1/15]` count, [`matchpair`] and [`comment`] are `%`, and
//! [`includes`] with [`incline`] is `[i`/`:checkpath`.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::autocmd::{EVENT_SEARCHWRAPPED, apply_autocmds};
use crate::src::nvim::buffer::buf_get_changedtick;
use crate::src::nvim::change::get_leader_len;
use crate::src::nvim::charset::{skipwhite, vim_isfilec, vim_iswordc, vim_iswordp};
use crate::src::nvim::cmdhist::add_to_history;
use crate::src::nvim::cursor::{
    check_cursor, dec_cursor, get_cursor_line_len, get_cursor_line_ptr, inc_cursor,
};
use crate::src::nvim::drawscreen::{
    UPD_INVERTED, UPD_SOME_VALID, UPD_VALID, redraw_all_later, redraw_curbuf_later, redraw_later,
    setcursor, show_cursor_info_later, showmode, update_screen,
};
use crate::src::nvim::eval::typval::tv_list_len;
use crate::src::nvim::eval::typval::{
    tv_check_for_nonnull_dict_arg, tv_dict_add_nr, tv_dict_alloc_ret, tv_dict_find,
    tv_get_number_chk, tv_get_string_chk, tv_list_find,
};
use crate::src::nvim::eval::vars::set_vim_var_nr;
use crate::src::nvim::ex_cmds::{getfile, prepare_tagpreview};
use crate::src::nvim::ex_docmd::set_no_hlsearch;
use crate::src::nvim::ex_getln::gotocmdline;
use crate::src::nvim::file_search::{file_name_in_line, find_file_name_in_path};
use crate::src::nvim::fileio::vim_fgets;
use crate::src::nvim::fold::{foldOpenCursor, hasFolding};
use crate::src::nvim::getchar::char_avail;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::indent_c::is_pos_in_string;
use crate::src::nvim::insexpand::{
    compl_status_adding, compl_status_sol, ctrl_x_mode_not_default, find_word_end, find_word_start,
    ins_compl_add_infercase, ins_compl_check_keys, ins_compl_interrupted, ins_compl_len,
};
use crate::src::nvim::main::{
    Columns, IObuff, KeyStuffed, KeyTyped, Rows, State, VIsual, VIsual_active, VIsual_mode,
    bot_top_msg, called_emsg, cmd_silent, cmdmod, curbuf, curwin, dollar_vcol, e_interr, e_invarg2,
    e_nopresub, e_noprevre, e_patnotf2, emsg_off, fdo_flags, g_do_tagpreview, got_int,
    msg_ext_overwrite, msg_hist_off, msg_nowait, msg_row, msg_scrolled, msg_silent, no_hlsearch,
    no_smartcase, p_cpo, p_def, p_hls, p_ic, p_inc, p_js, p_mat, p_msc, p_ri, p_scs, p_sel, p_siso,
    p_so, p_verbose, p_ws, rc_did_emsg, sc_col, search_match_endcol, search_match_lines,
    searchcmdlen, top_bot_msg,
};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::mbyte::{
    mb_isupper, mb_strcmp_ic, mb_strnicmp, utf_char2bytes, utf_head_off, utf_iscomposing_first,
    utf_ptr2char, utfc_ptr2len,
};
use crate::src::nvim::memline::{decl, inc, incl, ml_get, ml_get_buf, ml_get_buf_len, ml_get_len};
use crate::src::nvim::memory::{xfree, xmalloc, xmemdupz, xstrlcpy};
use crate::src::nvim::message::{
    emsg, give_warning, iemsg, messaging, msg, msg_check, msg_clr_eos, msg_end, msg_ext_set_kind,
    msg_home_replace, msg_outtrans, msg_prt_line, msg_putchar, msg_puts, msg_puts_hl,
    msg_puts_title, msg_start, msg_strtrunc, msg_trunc, semsg, smsg, verbose_enter, verbose_leave,
};
use crate::src::nvim::mouse::setmouse;
use crate::src::nvim::r#move::validate_cursor;
use crate::src::nvim::normal::may_start_select;
use crate::src::nvim::option::{magic_isset, shortmess};
use crate::src::nvim::options::{kOptBoFlagShowmatch, kOptFdoFlagSearch};
use crate::src::nvim::os::fs::os_fopen;
use crate::src::nvim::os::input::{fast_breakcheck, line_breakcheck};
use crate::src::nvim::os::libc::{
    atol, fclose, gettext, snprintf, strlen, strncmp, strpbrk, strstr,
};
use crate::src::nvim::os::time::{os_delay, os_time};
use crate::src::nvim::path::path_full_compare;
use crate::src::nvim::plines::getvcol;
use crate::src::nvim::pos::{clearpos, equalpos, lt, ltoreq};
use crate::src::nvim::profile::{profile_passed_limit, profile_setlimit};
use crate::src::nvim::regexp::skip_regexp_ex;
use crate::src::nvim::regexp::vim_regexec_multi;
use crate::src::nvim::state::MODE_SHOWMATCH;
use crate::src::nvim::strings::{reverse_text, vim_snprintf, vim_strchr, xstrnsave};
use crate::src::nvim::types::ui::kUIMessages;
use crate::src::nvim::types::{
    Direction, EvalFuncData, FILE, MotionType, OptInt, SearchOffset, SearchPattern, TriState,
    VarType, VimVarIndex, buf_T, cmdarg_T, colnr_T, dict_T, file_comparison, int64_t, linenr_T,
    list_T, lpos_T, magic_T, oparg_T, pos_T, proftime_T, ptrdiff_t, regmatch_T, regmmatch_T,
    regprog_T, searchit_arg_T, size_t, typval_T, varnumber_T, win_T,
};
use crate::src::nvim::ui::{
    ui_busy_start, ui_busy_stop, ui_cursor_shape, ui_flush, ui_has, vim_beep,
};
use crate::src::nvim::window::{win_enter, win_split, win_valid};

// The carve of the transpiled module; see each child's docs.
mod pattern;
pub use self::pattern::*;
mod find;
pub use self::find::*;
mod charsearch;
pub use self::charsearch::*;
mod command;
pub use self::command::*;
mod incline;
pub(crate) use self::incline::*;
mod includes;
pub use self::includes::*;
mod comment;
pub use self::comment::*;
mod select;
pub use self::select::*;
mod matchpair;
pub use self::matchpair::*;
mod stat;
pub use self::stat::*;
unsafe extern "C" {
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
pub const VAR_LIST: VarType = 4;
pub const VAR_UNKNOWN: VarType = 0;
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_12 = 2147483647;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const MAGIC_ALL: magic_T = 4;
pub const MAGIC_ON: magic_T = 3;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_14 = 4096;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_17 = 83;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_17 = 67;
pub const SHM_SEARCH: C2Rust_Unnamed_17 = 115;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_int;
pub const HIST_SEARCH: C2Rust_Unnamed_18 = 1;
pub const VV_SEARCHFORWARD: VimVarIndex = 56;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const FNAME_REL: C2Rust_Unnamed_20 = 16;
pub const FNAME_INCL: C2Rust_Unnamed_20 = 8;
pub const FNAME_EXP: C2Rust_Unnamed_20 = 2;
pub const kMTLineWise: MotionType = 1;
pub const kEqualFiles: file_comparison = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const CHECK_PATH: C2Rust_Unnamed_22 = 3;
pub const FIND_DEFINE: C2Rust_Unnamed_22 = 2;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const ACTION_EXPAND: C2Rust_Unnamed_23 = 5;
pub const ACTION_SHOW_ALL: C2Rust_Unnamed_23 = 4;
pub const ACTION_SPLIT: C2Rust_Unnamed_23 = 3;
pub const ACTION_SHOW: C2Rust_Unnamed_23 = 1;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const SEARCH_COL: C2Rust_Unnamed_24 = 4096;
pub const SEARCH_PEEK: C2Rust_Unnamed_24 = 2048;
pub const SEARCH_KEEP: C2Rust_Unnamed_24 = 1024;
pub const SEARCH_MARK: C2Rust_Unnamed_24 = 512;
pub const SEARCH_START: C2Rust_Unnamed_24 = 256;
pub const SEARCH_NOOF: C2Rust_Unnamed_24 = 128;
pub const SEARCH_END: C2Rust_Unnamed_24 = 64;
pub const SEARCH_HIS: C2Rust_Unnamed_24 = 32;
pub const SEARCH_OPT: C2Rust_Unnamed_24 = 16;
pub const SEARCH_MSG: C2Rust_Unnamed_24 = 12;
pub const SEARCH_ECHO: C2Rust_Unnamed_24 = 2;
pub const SEARCH_REV: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const FM_BLOCKSTOP: C2Rust_Unnamed_25 = 4;
pub const FM_FORWARD: C2Rust_Unnamed_25 = 2;
pub const FM_BACKWARD: C2Rust_Unnamed_25 = 1;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const RE_LAST: C2Rust_Unnamed_26 = 2;
pub const RE_BOTH: C2Rust_Unnamed_26 = 2;
pub const RE_SUBST: C2Rust_Unnamed_26 = 1;
pub const RE_SEARCH: C2Rust_Unnamed_26 = 0;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const SEARCH_STAT_DEF_TIMEOUT: C2Rust_Unnamed_27 = 40;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const SEARCH_STAT_BUF_LEN: C2Rust_Unnamed_28 = 16;
pub const LSIZE: C2Rust_Unnamed_29 = 512;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CPO_SEARCH: ::core::ffi::c_int = 'c' as ::core::ffi::c_int;
pub const CPO_SHOWMATCH: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const CPO_MATCHBSL: ::core::ffi::c_int = 'M' as ::core::ffi::c_int;
pub const CPO_LINEOFF: ::core::ffi::c_int = 'o' as ::core::ffi::c_int;
pub const CPO_MATCH: ::core::ffi::c_int = '%' as ::core::ffi::c_int;
pub const CPO_SCOLON: ::core::ffi::c_int = ';' as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
