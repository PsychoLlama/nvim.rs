use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::autocmd::{
    EVENT_DIFFUPDATED, apply_autocmds, aucmd_prepbuf, aucmd_restbuf, augroup_exists,
    block_autocmds, unblock_autocmds,
};
use crate::src::nvim::buffer::buf_get_changedtick;
use crate::src::nvim::buffer::{
    bt_prompt, buf_is_empty, buf_valid, buflist_findnr, buflist_findpat, bufref_valid, set_bufref,
};
use crate::src::nvim::bufwrite::{WriteRequest, buf_write};
use crate::src::nvim::change::{change_warning, changed_lines};
use crate::src::nvim::charset::{getdigits_int, getdigits_int32, skipwhite};
use crate::src::nvim::cursor::check_cursor;
use crate::src::nvim::decoration::decor_conceal_line;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, UPD_SOME_VALID, UPD_VALID, redraw_later};
use crate::src::nvim::eval::typval::{tv_get_lnum, tv_get_number};
use crate::src::nvim::eval::vars::{eval_diff, eval_patch};
use crate::src::nvim::ex_cmds::{append_redir, ex_file};
use crate::src::nvim::ex_docmd::{do_cmdline_cmd, do_exedit, get_address};
use crate::src::nvim::extmark::extmark_adjust;
use crate::src::nvim::fileio::{
    buf_check_timestamp, shorten_fnames, vim_fgets, vim_gettempdir, vim_tempname,
};
use crate::src::nvim::fold::{
    foldUpdate, foldUpdateAll, foldmethodIsDiff, foldmethodIsManual, hasFolding, newFoldLevel,
};
use crate::src::nvim::garray::{ga_append, ga_clear, ga_concat_len, ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight_group::{HLF_ADD, HLF_CHD, HLF_NONE, HLF_TXA, HLF_TXD};
use crate::src::nvim::linematch::linematch_nbuffers;
use crate::src::nvim::main::{
    KeyTyped, cmdmod, curbuf, curtab, curwin, diff_context, diff_foldcolumn, diff_need_scrollbind,
    e_cannot_have_more_than_nr_diff_anchors, e_diff_anchors_with_hidden_windows,
    e_failed_to_find_all_diff_anchors, e_invrange, e_prev_dir, e_problem_creating_internal_diff,
    first_tabpage, firstwin, need_diff_redraw, p_dex, p_dia, p_dip, p_pex, p_sbo, p_srr,
};
use crate::src::nvim::mark::{mark_adjust, setpcmark};
use crate::src::nvim::mbyte::{
    mb_get_class_tab, mb_stricmp, utf_char2bytes, utf_char2len, utf_fold, utf_head_off,
    utf_ptr2char, utfc_ptr2len,
};
use crate::src::nvim::memline::{ml_append, ml_delete, ml_get_buf, ml_get_buf_len};
use crate::src::nvim::memory::{memchrsub, xcalloc, xfree, xmalloc, xstrdup};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::r#move::{
    changed_line_abv_curs, changed_line_abv_curs_win, changed_window_setting, check_topfill,
    invalidate_botline_win, validate_cursor,
};
use crate::src::nvim::normal::check_scrollbind;
use crate::src::nvim::option::{set_option_direct_for, set_option_value_give_err};
use crate::src::nvim::options::{kOptBoFlagOperator, kOptDiff, kOptFoldmethod};
use crate::src::nvim::optionstr::free_string_option;
use crate::src::nvim::os::env::{os_env_exists, os_unsetenv};
use crate::src::nvim::os::fs::{
    os_chdir, os_dirname, os_fileinfo, os_fileinfo_size, os_fopen, os_remove,
};
use crate::src::nvim::os::libc::{
    __assert_fail, __ctype_b_loc, atol, fclose, fwrite, gettext, memcpy, memmove, memset, qsort,
    snprintf, strcat, strcmp, strcpy, strlen, strncmp, tolower,
};
use crate::src::nvim::os::shell::call_shell;
use crate::src::nvim::path::FullName_save;
use crate::src::nvim::pos::{MAXCOL, MAXLNUM};
use crate::src::nvim::search::{BACKWARD, FORWARD};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr, vim_strsave_shellescape, xstrnsave};
use crate::src::nvim::types::{
    BoolVarValue, CMD_index, EvalFuncData, ExtmarkOp, FILE, FileInfo, OptInt, OptScope, OptVal,
    OptValData, OptValType, ScopeType, SpecialVarValue, String_0, TriState, VarLockStatus, VarType,
    aco_save_T, buf_T, bufref_T, cmd_addr_T, colnr_T, cstack_T, diff_T, diffblock_S, diffline_S,
    diffline_T, diffline_change_S, diffline_change_T, exarg_T, garray_T, hlf_T, int32_t, kFalse,
    kNone, kTrue, linenr_T, mmfile_t, scid_T, size_t, tabpage_T, typval_T, uint8_t, uint64_t,
    uv_stat_t, uv_timespec_t, varnumber_T, win_T, xdemitcb_t, xdemitconf_t,
    xdl_emit_hunk_consume_func_t, xpparam_t,
};
use crate::src::nvim::ui::vim_beep;
use crate::src::nvim::undo::{u_save, u_sync};
use crate::src::nvim::window::{
    frames_locked, scroll_to_fraction, set_fraction, win_split, win_valid,
};
use crate::src::xdiff::xdiffi::xdl_diff;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub const VAR_PARTIAL: VarType = 9;
pub const VAR_SPECIAL: VarType = 8;
pub const VAR_BOOL: VarType = 7;
pub const VAR_FLOAT: VarType = 6;
pub const VAR_DICT: VarType = 5;
pub const VAR_LIST: VarType = 4;
pub const VAR_FUNC: VarType = 3;
pub const VAR_STRING: VarType = 2;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_UNKNOWN: VarType = 0;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptScopeWin: OptScope = 1;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const CMD_split: CMD_index = 420;
pub const CMD_diffput: CMD_index = 122;
pub const CMD_diffget: CMD_index = 119;
pub const CMD_append: CMD_index = 0;
pub const ADDR_LINES: cmd_addr_T = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_18 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_18 = 2048;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct diffio_T {
    pub dio_orig: diffin_T,
    pub dio_new: diffin_T,
    pub dio_diff: diffout_T,
    pub dio_internal: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct diffout_T {
    pub dout_fname: *mut ::core::ffi::c_char,
    pub dout_ga: garray_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct diffin_T {
    pub din_fname: *mut ::core::ffi::c_char,
    pub din_mmfile: mmfile_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct diffhunk_T {
    pub lnum_orig: linenr_T,
    pub count_orig: ::core::ffi::c_int,
    pub lnum_new: linenr_T,
    pub count_new: ::core::ffi::c_int,
}
pub type diffstyle_T = ::core::ffi::c_uint;
pub const DIFF_NONE: diffstyle_T = 2;
pub const DIFF_UNIFIED: diffstyle_T = 1;
pub const DIFF_ED: diffstyle_T = 0;
pub const kShellOptDoOut: C2Rust_Unnamed_22 = 4;
pub const kShellOptSilent: C2Rust_Unnamed_22 = 8;
pub const kShellOptFilter: C2Rust_Unnamed_22 = 1;
pub const MAX_DIFF_ANCHORS: C2Rust_Unnamed_24 = 20;
pub const OPT_LOCAL: C2Rust_Unnamed_21 = 2;
pub const WSP_VERT: C2Rust_Unnamed_23 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct linemap_entry_T {
    pub byte_start: colnr_T,
    pub num_bytes: colnr_T,
    pub lineoff: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const DB_COUNT: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static diff_busy: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static diff_need_update: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
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
static diff_algorithm: GlobalCell<::core::ffi::c_int> = GlobalCell::new(XDF_INDENT_HEURISTIC);
static diff_word_gap: GlobalCell<::core::ffi::c_int> = GlobalCell::new(5 as ::core::ffi::c_int);
static linematch_lines: GlobalCell<::core::ffi::c_int> = GlobalCell::new(40 as ::core::ffi::c_int);
pub const LBUFLEN: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
pub const MAX_XDIFF_SIZE: ::core::ffi::c_long =
    1024 as ::core::ffi::c_long * 1024 as ::core::ffi::c_long * 1023 as ::core::ffi::c_long;
static diff_a_works: GlobalCell<TriState> = GlobalCell::new(kNone);
unsafe extern "C" fn clear_diffblock(mut dp: *mut diff_T) {
    ga_clear(&raw mut (*dp).df_changes);
    xfree(dp as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn diff_buf_delete(mut buf: *mut buf_T) {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut i: ::core::ffi::c_int = diff_buf_idx(buf, tp as *mut tabpage_T);
        if i != DB_COUNT {
            (*tp).tp_diffbuf[i as usize] = ::core::ptr::null_mut::<buf_T>();
            (*tp).tp_diff_invalid = true_0;
            if tp == curtab.get() {
                need_diff_redraw.set(true_0 != 0);
                redraw_later(curwin.get(), UPD_VALID);
            }
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
pub unsafe extern "C" fn diff_buf_adjust(mut win: *mut win_T) {
    if (*win).w_onebuf_opt.wo_diff == 0 {
        let mut found_win: bool = false_0 != 0;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == (*win).w_buffer && (*wp).w_onebuf_opt.wo_diff != 0 {
                found_win = true_0 != 0;
            }
            wp = (*wp).w_next;
        }
        if !found_win {
            let mut i: ::core::ffi::c_int = diff_buf_idx((*win).w_buffer, curtab.get());
            if i != DB_COUNT {
                (*curtab.get()).tp_diffbuf[i as usize] = ::core::ptr::null_mut::<buf_T>();
                (*curtab.get()).tp_diff_invalid = true_0;
                diff_redraw(true_0 != 0);
            }
        }
    } else {
        diff_buf_add((*win).w_buffer);
    };
}
pub unsafe extern "C" fn diff_buf_add(mut buf: *mut buf_T) {
    if diff_buf_idx(buf, curtab.get()) != DB_COUNT {
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if (*curtab.get()).tp_diffbuf[i as usize].is_null() {
            (*curtab.get()).tp_diffbuf[i as usize] = buf as *mut buf_T;
            (*curtab.get()).tp_diff_invalid = true_0;
            diff_redraw(true_0 != 0);
            return;
        }
        i += 1;
    }
    semsg(
        gettext(b"E96: Cannot diff more than %d buffers\0".as_ptr() as *const ::core::ffi::c_char),
        DB_COUNT,
    );
}
unsafe extern "C" fn diff_buf_clear() {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
            (*curtab.get()).tp_diffbuf[i as usize] = ::core::ptr::null_mut::<buf_T>();
            (*curtab.get()).tp_diff_invalid = true_0;
            diff_redraw(true_0 != 0);
        }
        i += 1;
    }
}
unsafe extern "C" fn diff_buf_idx(
    mut buf: *mut buf_T,
    mut tp: *mut tabpage_T,
) -> ::core::ffi::c_int {
    let mut idx: ::core::ffi::c_int = 0;
    idx = 0 as ::core::ffi::c_int;
    while idx < DB_COUNT {
        if (*tp).tp_diffbuf[idx as usize] == buf {
            break;
        }
        idx += 1;
    }
    return idx;
}
pub unsafe extern "C" fn diff_invalidate(mut buf: *mut buf_T) {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut i: ::core::ffi::c_int = diff_buf_idx(buf, tp as *mut tabpage_T);
        if i != DB_COUNT {
            (*tp).tp_diff_invalid = true_0;
            if tp == curtab.get() {
                diff_redraw(true_0 != 0);
            }
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
pub unsafe extern "C" fn diff_mark_adjust(
    mut buf: *mut buf_T,
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
) {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut idx: ::core::ffi::c_int = diff_buf_idx(buf, tp as *mut tabpage_T);
        if idx != DB_COUNT {
            diff_mark_adjust_tp(
                tp as *mut tabpage_T,
                idx,
                line1,
                line2,
                amount,
                amount_after,
            );
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
unsafe extern "C" fn diff_mark_adjust_tp(
    mut tp: *mut tabpage_T,
    mut idx: ::core::ffi::c_int,
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
) {
    if diff_internal() != 0 {
        (*tp).tp_diff_invalid = true_0;
        (*tp).tp_diff_update = true_0;
    }
    let mut inserted: linenr_T = 0;
    let mut deleted: linenr_T = 0;
    if line2 == MAXLNUM as ::core::ffi::c_int as linenr_T {
        inserted = amount;
        deleted = 0 as ::core::ffi::c_int as linenr_T;
    } else if amount_after > 0 as linenr_T {
        inserted = amount_after;
        deleted = 0 as ::core::ffi::c_int as linenr_T;
    } else {
        inserted = 0 as ::core::ffi::c_int as linenr_T;
        deleted = -amount_after;
    }
    let mut dprev: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut dp: *mut diff_T = (*tp).tp_first_diff;
    let mut lnum_deleted: linenr_T = line1;
    loop {
        if (dp.is_null()
            || (*dp).df_lnum[idx as usize] - 1 as linenr_T > line2
            || line2 == MAXLNUM as ::core::ffi::c_int as linenr_T
                && (*dp).df_lnum[idx as usize] > line1)
            && (dprev.is_null()
                || (*dprev).df_lnum[idx as usize] + (*dprev).df_count[idx as usize] < line1)
            && !diff_busy.get()
        {
            let mut dnext: *mut diff_T = diff_alloc_new(tp, dprev, dp);
            (*dnext).df_lnum[idx as usize] = line1;
            (*dnext).df_count[idx as usize] = inserted;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < DB_COUNT {
                if !(*tp).tp_diffbuf[i as usize].is_null() && i != idx {
                    if dprev.is_null() {
                        (*dnext).df_lnum[i as usize] = line1;
                    } else {
                        (*dnext).df_lnum[i as usize] = line1
                            + ((*dprev).df_lnum[i as usize] + (*dprev).df_count[i as usize])
                            - ((*dprev).df_lnum[idx as usize] + (*dprev).df_count[idx as usize]);
                    }
                    (*dnext).df_count[i as usize] = deleted;
                }
                i += 1;
            }
        }
        if dp.is_null() {
            break;
        }
        let mut last: linenr_T =
            (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] - 1 as linenr_T;
        if last >= line1 - 1 as linenr_T {
            if diff_busy.get() {
                if (*dp).df_lnum[idx as usize] > line2 {
                    (*dp).df_lnum[idx as usize] += amount_after;
                }
                dprev = dp;
                dp = (*dp).df_next;
                continue;
            } else if (*dp).df_lnum[idx as usize]
                - (deleted + inserted != 0 as linenr_T) as ::core::ffi::c_int
                > line2
            {
                if amount_after == 0 as linenr_T {
                    break;
                }
                (*dp).df_lnum[idx as usize] += amount_after;
            } else {
                let mut check_unchanged: bool = false_0 != 0;
                if deleted > 0 as linenr_T {
                    let mut n: linenr_T = 0;
                    let mut off: linenr_T = 0 as linenr_T;
                    if (*dp).df_lnum[idx as usize] >= line1 {
                        if last <= line2 {
                            if !(*dp).df_next.is_null()
                                && (*(*dp).df_next).df_lnum[idx as usize] - 1 as linenr_T <= line2
                            {
                                n = (*(*dp).df_next).df_lnum[idx as usize] - lnum_deleted;
                                deleted -= n;
                                n -= (*dp).df_count[idx as usize];
                                lnum_deleted = (*(*dp).df_next).df_lnum[idx as usize];
                            } else {
                                n = deleted - (*dp).df_count[idx as usize];
                            }
                            (*dp).df_count[idx as usize] = 0 as ::core::ffi::c_int as linenr_T;
                        } else {
                            off = (*dp).df_lnum[idx as usize] - lnum_deleted;
                            n = off;
                            (*dp).df_count[idx as usize] = ((*dp).df_count[idx as usize]
                                as ::core::ffi::c_int
                                - (line2 - (*dp).df_lnum[idx as usize] + 1 as linenr_T)
                                    as ::core::ffi::c_int)
                                as linenr_T;
                            check_unchanged = true_0 != 0;
                        }
                        (*dp).df_lnum[idx as usize] = line1;
                    } else if last < line2 {
                        (*dp).df_count[idx as usize] = ((*dp).df_count[idx as usize]
                            as ::core::ffi::c_int
                            - (last - lnum_deleted + 1 as linenr_T) as ::core::ffi::c_int)
                            as linenr_T;
                        if !(*dp).df_next.is_null()
                            && (*(*dp).df_next).df_lnum[idx as usize] - 1 as linenr_T <= line2
                        {
                            n = (*(*dp).df_next).df_lnum[idx as usize] - 1 as linenr_T - last;
                            deleted -= (*(*dp).df_next).df_lnum[idx as usize] - lnum_deleted;
                            lnum_deleted = (*(*dp).df_next).df_lnum[idx as usize];
                        } else {
                            n = line2 - last;
                        }
                        check_unchanged = true_0 != 0;
                    } else {
                        n = 0 as ::core::ffi::c_int as linenr_T;
                        (*dp).df_count[idx as usize] -= deleted;
                    }
                    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i_0 < DB_COUNT {
                        if !(*tp).tp_diffbuf[i_0 as usize].is_null() && i_0 != idx {
                            if (*dp).df_lnum[i_0 as usize] > off {
                                (*dp).df_lnum[i_0 as usize] -= off;
                            } else {
                                (*dp).df_lnum[i_0 as usize] = 1 as ::core::ffi::c_int as linenr_T;
                            }
                            (*dp).df_count[i_0 as usize] += n;
                        }
                        i_0 += 1;
                    }
                } else if (*dp).df_lnum[idx as usize] <= line1 {
                    (*dp).df_count[idx as usize] += inserted;
                    check_unchanged = true_0 != 0;
                } else {
                    (*dp).df_lnum[idx as usize] += inserted;
                }
                if check_unchanged {
                    diff_check_unchanged(tp, dp);
                }
            }
        }
        if !dprev.is_null()
            && !(*dp).is_linematched
            && !diff_busy.get()
            && (*dprev).df_lnum[idx as usize] + (*dprev).df_count[idx as usize]
                == (*dp).df_lnum[idx as usize]
        {
            let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_1 < DB_COUNT {
                if !(*tp).tp_diffbuf[i_1 as usize].is_null() {
                    (*dprev).df_count[i_1 as usize] += (*dp).df_count[i_1 as usize];
                }
                i_1 += 1;
            }
            dp = diff_free(tp, dprev, dp);
        } else {
            dprev = dp;
            dp = (*dp).df_next;
        }
    }
    dprev = ::core::ptr::null_mut::<diff_T>();
    dp = (*tp).tp_first_diff;
    while !dp.is_null() {
        let mut i_2: ::core::ffi::c_int = 0;
        i_2 = 0 as ::core::ffi::c_int;
        while i_2 < DB_COUNT {
            if !(*tp).tp_diffbuf[i_2 as usize].is_null()
                && (*dp).df_count[i_2 as usize] != 0 as linenr_T
            {
                break;
            }
            i_2 += 1;
        }
        if i_2 == DB_COUNT {
            dp = diff_free(tp, dprev, dp);
        } else {
            dprev = dp;
            dp = (*dp).df_next;
        }
    }
    if tp == curtab.get() {
        need_diff_redraw.set(true_0 != 0);
        diff_need_scrollbind.set(true_0 != 0);
    }
}
unsafe extern "C" fn diff_alloc_new(
    mut tp: *mut tabpage_T,
    mut dprev: *mut diff_T,
    mut dp: *mut diff_T,
) -> *mut diff_T {
    let mut dnew: *mut diff_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<diff_T>()) as *mut diff_T;
    (*dnew).is_linematched = false_0 != 0;
    (*dnew).df_next = dp;
    if dprev.is_null() {
        (*tp).tp_first_diff = dnew;
    } else {
        (*dprev).df_next = dnew;
    }
    (*dnew).has_changes = false_0 != 0;
    ga_init(
        &raw mut (*dnew).df_changes,
        ::core::mem::size_of::<diffline_change_T>() as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
    );
    return dnew;
}
unsafe extern "C" fn diff_free(
    mut tp: *mut tabpage_T,
    mut dprev: *mut diff_T,
    mut dp: *mut diff_T,
) -> *mut diff_T {
    let mut ret: *mut diff_T = (*dp).df_next;
    clear_diffblock(dp);
    if dprev.is_null() {
        (*tp).tp_first_diff = ret;
    } else {
        (*dprev).df_next = ret;
    }
    return ret;
}
unsafe extern "C" fn diff_check_unchanged(mut tp: *mut tabpage_T, mut dp: *mut diff_T) {
    let mut i_org: ::core::ffi::c_int = 0;
    i_org = 0 as ::core::ffi::c_int;
    while i_org < DB_COUNT {
        if !(*tp).tp_diffbuf[i_org as usize].is_null() {
            break;
        }
        i_org += 1;
    }
    if i_org == DB_COUNT {
        return;
    }
    if diff_check_sanity(tp, dp) == FAIL {
        return;
    }
    let mut off_org: linenr_T = 0 as linenr_T;
    let mut off_new: linenr_T = 0 as linenr_T;
    let mut dir: ::core::ffi::c_int = FORWARD as ::core::ffi::c_int;
    loop {
        while (*dp).df_count[i_org as usize] > 0 as linenr_T {
            if dir == BACKWARD as ::core::ffi::c_int {
                off_org = (*dp).df_count[i_org as usize] - 1 as linenr_T;
            }
            let mut line_org: *mut ::core::ffi::c_char = xstrdup(ml_get_buf(
                (*tp).tp_diffbuf[i_org as usize] as *mut buf_T,
                (*dp).df_lnum[i_org as usize] + off_org,
            ));
            let mut i_new: ::core::ffi::c_int = 0;
            i_new = i_org + 1 as ::core::ffi::c_int;
            while i_new < DB_COUNT {
                if !(*tp).tp_diffbuf[i_new as usize].is_null() {
                    if dir == BACKWARD as ::core::ffi::c_int {
                        off_new = (*dp).df_count[i_new as usize] - 1 as linenr_T;
                    }
                    if off_new < 0 as linenr_T || off_new >= (*dp).df_count[i_new as usize] {
                        break;
                    }
                    if diff_cmp(
                        line_org,
                        ml_get_buf(
                            (*tp).tp_diffbuf[i_new as usize] as *mut buf_T,
                            (*dp).df_lnum[i_new as usize] + off_new,
                        ),
                    ) != 0 as ::core::ffi::c_int
                    {
                        break;
                    }
                }
                i_new += 1;
            }
            xfree(line_org as *mut ::core::ffi::c_void);
            if i_new != DB_COUNT {
                break;
            }
            i_new = i_org;
            while i_new < DB_COUNT {
                if !(*tp).tp_diffbuf[i_new as usize].is_null() {
                    if dir == FORWARD as ::core::ffi::c_int {
                        (*dp).df_lnum[i_new as usize] += 1;
                    }
                    (*dp).df_count[i_new as usize] -= 1;
                }
                i_new += 1;
            }
        }
        if dir == BACKWARD as ::core::ffi::c_int {
            break;
        }
        dir = BACKWARD as ::core::ffi::c_int;
    }
}
unsafe extern "C" fn diff_check_sanity(
    mut tp: *mut tabpage_T,
    mut dp: *mut diff_T,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if !(*tp).tp_diffbuf[i as usize].is_null() {
            if (*dp).df_lnum[i as usize] + (*dp).df_count[i as usize] - 1 as linenr_T
                > (*(*tp).tp_diffbuf[i as usize]).b_ml.ml_line_count
            {
                return FAIL;
            }
        }
        i += 1;
    }
    return OK;
}
pub unsafe extern "C" fn diff_redraw(mut dofold: bool) {
    let mut wp_other: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut used_max_fill_other: bool = false_0 != 0;
    let mut used_max_fill_curwin: bool = false_0 != 0;
    need_diff_redraw.set(false_0 != 0);
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if !((*wp).w_onebuf_opt.wo_diff == 0 || !buf_valid((*wp).w_buffer)) {
            redraw_later(wp, UPD_SOME_VALID);
            if wp != curwin.get() {
                wp_other = wp;
            }
            if dofold as ::core::ffi::c_int != 0 && foldmethodIsDiff(wp) as ::core::ffi::c_int != 0
            {
                foldUpdateAll(wp);
            }
            let mut n: ::core::ffi::c_int = diff_check_fill(wp, (*wp).w_topline);
            if wp != curwin.get() && (*wp).w_topfill > 0 as ::core::ffi::c_int
                || n > 0 as ::core::ffi::c_int
            {
                if (*wp).w_topfill > n {
                    (*wp).w_topfill = if n > 0 as ::core::ffi::c_int {
                        n
                    } else {
                        0 as ::core::ffi::c_int
                    };
                } else if n > 0 as ::core::ffi::c_int && n > (*wp).w_topfill {
                    (*wp).w_topfill = n;
                    if wp == curwin.get() {
                        used_max_fill_curwin = true_0 != 0;
                    } else if !wp_other.is_null() {
                        used_max_fill_other = true_0 != 0;
                    }
                }
                check_topfill(wp, false_0 != 0);
            }
        }
        wp = (*wp).w_next;
    }
    if !wp_other.is_null() && (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
        if used_max_fill_curwin {
            diff_set_topline(wp_other, curwin.get());
        } else if used_max_fill_other {
            diff_set_topline(curwin.get(), wp_other);
        }
    }
}
unsafe extern "C" fn clear_diffin(mut din: *mut diffin_T) {
    if (*din).din_fname.is_null() {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*din).din_mmfile.ptr as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
    } else {
        os_remove((*din).din_fname);
    };
}
unsafe extern "C" fn clear_diffout(mut dout: *mut diffout_T) {
    if (*dout).dout_fname.is_null() {
        ga_clear(&raw mut (*dout).dout_ga);
    } else {
        os_remove((*dout).dout_fname);
    };
}
unsafe extern "C" fn diff_write_buffer(
    mut buf: *mut buf_T,
    mut m: *mut mmfile_t,
    mut start: linenr_T,
    mut end: linenr_T,
) -> ::core::ffi::c_int {
    if end < 0 as linenr_T {
        end = (*buf).b_ml.ml_line_count;
    }
    if (*buf).b_ml.ml_flags & ML_EMPTY != 0 || end < start {
        (*m).ptr = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*m).size = 0 as ::core::ffi::c_int;
        return OK;
    }
    let mut len: size_t = 0 as size_t;
    let mut lnum: linenr_T = start;
    while lnum <= end {
        len = len.wrapping_add((ml_get_buf_len(buf, lnum) as size_t).wrapping_add(1 as size_t));
        lnum += 1;
    }
    let mut ptr: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    (*m).ptr = ptr;
    (*m).size = len as ::core::ffi::c_int;
    len = 0 as size_t;
    let mut lnum_0: linenr_T = start;
    while lnum_0 <= end {
        let mut s: *mut ::core::ffi::c_char = ml_get_buf(buf, lnum_0);
        if diff_flags.get() & DIFF_ICASE != 0 {
            while *s as ::core::ffi::c_int != NUL {
                let mut c: ::core::ffi::c_int = 0;
                let mut c_len: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                let mut cbuf: [::core::ffi::c_char; 22] = [0; 22];
                if *s as ::core::ffi::c_int == NL {
                    c = NUL;
                } else {
                    c = utf_ptr2char(s);
                    c_len = utf_char2len(c);
                    c = utf_fold(c);
                }
                let orig_len: ::core::ffi::c_int = utfc_ptr2len(s);
                if utf_char2bytes(c, &raw mut cbuf as *mut ::core::ffi::c_char) != c_len {
                    memmove(
                        ptr.offset(len as isize) as *mut ::core::ffi::c_void,
                        s as *const ::core::ffi::c_void,
                        orig_len as size_t,
                    );
                } else {
                    memmove(
                        ptr.offset(len as isize) as *mut ::core::ffi::c_void,
                        &raw mut cbuf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                        c_len as size_t,
                    );
                    if orig_len > c_len {
                        memmove(
                            ptr.offset(len as isize).offset(c_len as isize)
                                as *mut ::core::ffi::c_void,
                            s.offset(c_len as isize) as *const ::core::ffi::c_void,
                            (orig_len - c_len) as size_t,
                        );
                    }
                }
                s = s.offset(orig_len as isize);
                len = len.wrapping_add(orig_len as size_t);
            }
        } else {
            let mut slen: size_t = strlen(s);
            memmove(
                ptr.offset(len as isize) as *mut ::core::ffi::c_void,
                s as *const ::core::ffi::c_void,
                slen,
            );
            memchrsub(
                ptr.offset(len as isize) as *mut ::core::ffi::c_void,
                NL as ::core::ffi::c_char,
                NUL as ::core::ffi::c_char,
                slen,
            );
            len = len.wrapping_add(slen);
        }
        let c2rust_fresh8 = len;
        len = len.wrapping_add(1);
        *ptr.offset(c2rust_fresh8 as isize) = NL as ::core::ffi::c_char;
        lnum_0 += 1;
    }
    return OK;
}
unsafe extern "C" fn diff_write(
    mut buf: *mut buf_T,
    mut din: *mut diffin_T,
    mut start: linenr_T,
    mut end: linenr_T,
) -> ::core::ffi::c_int {
    if (*din).din_fname.is_null() {
        return diff_write_buffer(buf, &raw mut (*din).din_mmfile, start, end);
    }
    if frames_locked() {
        return FAIL;
    }
    if end < 0 as linenr_T {
        end = (*buf).b_ml.ml_line_count;
    }
    let mut save_ml_flags: ::core::ffi::c_int = (*buf).b_ml.ml_flags;
    let mut save_ff: *mut ::core::ffi::c_char = (*buf).b_p_ff;
    (*buf).b_p_ff = xstrdup(b"unix\0".as_ptr() as *const ::core::ffi::c_char);
    let save_cmod_flags: bool = (*cmdmod.ptr()).cmod_flags != 0;
    (*cmdmod.ptr()).cmod_flags |= CMOD_LOCKMARKS as ::core::ffi::c_int;
    if end < start {
        end = start;
        (*buf).b_ml.ml_flags |= ML_EMPTY;
    }
    let mut r: ::core::ffi::c_int = buf_write(
        buf,
        (*din).din_fname,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        start,
        end,
        ::core::ptr::null_mut::<exarg_T>(),
        WriteRequest::filter(),
    );
    (*cmdmod.ptr()).cmod_flags = save_cmod_flags as ::core::ffi::c_int;
    free_string_option((*buf).b_p_ff);
    (*buf).b_p_ff = save_ff;
    (*buf).b_ml.ml_flags = (*buf).b_ml.ml_flags & !ML_EMPTY | save_ml_flags & ML_EMPTY;
    return r;
}
unsafe extern "C" fn lnum_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut lnum1: linenr_T = *(s1 as *mut linenr_T);
    let mut lnum2: linenr_T = *(s2 as *mut linenr_T);
    if lnum1 < lnum2 {
        return -1 as ::core::ffi::c_int;
    }
    if lnum1 > lnum2 {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn diff_try_update(
    mut dio: *mut diffio_T,
    mut idx_orig: ::core::ffi::c_int,
    mut eap: *mut exarg_T,
) {
    let mut num_anchors: ::core::ffi::c_int = 0;
    let mut anchors: [[linenr_T; 20]; 8] = [[0; 20]; 8];
    '_theend: {
        if (*dio).dio_internal != 0 {
            ga_init(
                &raw mut (*dio).dio_diff.dout_ga,
                ::core::mem::size_of::<diffhunk_T>() as ::core::ffi::c_int,
                100 as ::core::ffi::c_int,
            );
        } else {
            (*dio).dio_orig.din_fname = vim_tempname();
            (*dio).dio_new.din_fname = vim_tempname();
            (*dio).dio_diff.dout_fname = vim_tempname();
            if (*dio).dio_orig.din_fname.is_null()
                || (*dio).dio_new.din_fname.is_null()
                || (*dio).dio_diff.dout_fname.is_null()
            {
                break '_theend;
            } else if check_external_diff(dio) == FAIL {
                break '_theend;
            }
        }
        if !eap.is_null() && (*eap).forceit != 0 {
            let mut idx_new: ::core::ffi::c_int = idx_orig;
            while idx_new < DB_COUNT {
                let mut buf: *mut buf_T =
                    (*curtab.get()).tp_diffbuf[idx_new as usize] as *mut buf_T;
                if buf_valid(buf) {
                    buf_check_timestamp(buf);
                }
                idx_new += 1;
            }
        }
        num_anchors = INT_MAX;
        anchors = [[0; 20]; 8];
        memset(
            &raw mut anchors as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<[[linenr_T; 20]; 8]>(),
        );
        if diff_flags.get() & DIFF_ANCHOR != 0 {
            let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while idx < DB_COUNT {
                if !(*curtab.get()).tp_diffbuf[idx as usize].is_null() {
                    let mut buf_num_anchors: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if parse_diffanchors(
                        false_0 != 0,
                        (*curtab.get()).tp_diffbuf[idx as usize] as *mut buf_T,
                        &raw mut *(&raw mut anchors as *mut [linenr_T; 20]).offset(idx as isize)
                            as *mut linenr_T,
                        &raw mut buf_num_anchors,
                    ) != OK
                    {
                        emsg(gettext(
                            &raw const e_failed_to_find_all_diff_anchors
                                as *const ::core::ffi::c_char,
                        ));
                        num_anchors = 0 as ::core::ffi::c_int;
                        memset(
                            &raw mut anchors as *mut ::core::ffi::c_void,
                            0 as ::core::ffi::c_int,
                            ::core::mem::size_of::<[[linenr_T; 20]; 8]>(),
                        );
                        break;
                    } else {
                        if buf_num_anchors < num_anchors {
                            num_anchors = buf_num_anchors;
                        }
                        if buf_num_anchors > 0 as ::core::ffi::c_int {
                            qsort(
                                &raw mut *(&raw mut anchors as *mut [linenr_T; 20])
                                    .offset(idx as isize)
                                    as *mut linenr_T
                                    as *mut ::core::ffi::c_void,
                                buf_num_anchors as size_t,
                                ::core::mem::size_of::<linenr_T>(),
                                Some(
                                    lnum_compare
                                        as unsafe extern "C" fn(
                                            *const ::core::ffi::c_void,
                                            *const ::core::ffi::c_void,
                                        )
                                            -> ::core::ffi::c_int,
                                ),
                            );
                        }
                    }
                }
                idx += 1;
            }
        }
        if num_anchors == INT_MAX {
            num_anchors = 0 as ::core::ffi::c_int;
        }
        let mut anchor_i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            if anchor_i > num_anchors {
                break '_theend;
            }
            let mut orig_diff: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
            if anchor_i != 0 as ::core::ffi::c_int {
                orig_diff = (*curtab.get()).tp_first_diff;
                (*curtab.get()).tp_first_diff = ::core::ptr::null_mut::<diff_T>();
            }
            let mut lnum_start: linenr_T = if anchor_i == 0 as ::core::ffi::c_int {
                1 as linenr_T
            } else {
                anchors[idx_orig as usize][(anchor_i - 1 as ::core::ffi::c_int) as usize]
            };
            let mut lnum_end: linenr_T = if anchor_i == num_anchors {
                -1 as linenr_T
            } else {
                anchors[idx_orig as usize][anchor_i as usize] - 1 as linenr_T
            };
            let mut buf_0: *mut buf_T = (*curtab.get()).tp_diffbuf[idx_orig as usize] as *mut buf_T;
            if diff_write(buf_0, &raw mut (*dio).dio_orig, lnum_start, lnum_end) == FAIL {
                if !orig_diff.is_null() {
                    (*curtab.get()).tp_first_diff = orig_diff;
                    diff_clear(curtab.get());
                }
                break '_theend;
            } else {
                let mut idx_new_0: ::core::ffi::c_int = idx_orig + 1 as ::core::ffi::c_int;
                while idx_new_0 < DB_COUNT {
                    buf_0 = (*curtab.get()).tp_diffbuf[idx_new_0 as usize] as *mut buf_T;
                    if !(buf_0.is_null() || (*buf_0).b_ml.ml_mfp.is_null()) {
                        lnum_start = if anchor_i == 0 as ::core::ffi::c_int {
                            1 as linenr_T
                        } else {
                            anchors[idx_new_0 as usize]
                                [(anchor_i - 1 as ::core::ffi::c_int) as usize]
                        };
                        lnum_end = if anchor_i == num_anchors {
                            -1 as linenr_T
                        } else {
                            anchors[idx_new_0 as usize][anchor_i as usize] - 1 as linenr_T
                        };
                        if diff_write(buf_0, &raw mut (*dio).dio_new, lnum_start, lnum_end) != FAIL
                        {
                            if diff_file(dio) != FAIL {
                                diff_read(idx_orig, idx_new_0, dio);
                                clear_diffin(&raw mut (*dio).dio_new);
                                clear_diffout(&raw mut (*dio).dio_diff);
                            }
                        }
                    }
                    idx_new_0 += 1;
                }
                clear_diffin(&raw mut (*dio).dio_orig);
                if anchor_i != 0 as ::core::ffi::c_int {
                    let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
                    while !dp.is_null() {
                        let mut idx_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while idx_0 < DB_COUNT {
                            if anchors[idx_0 as usize]
                                [(anchor_i - 1 as ::core::ffi::c_int) as usize]
                                > 0 as linenr_T
                            {
                                (*dp).df_lnum[idx_0 as usize] = ((*dp).df_lnum[idx_0 as usize]
                                    as ::core::ffi::c_int
                                    + (anchors[idx_0 as usize]
                                        [(anchor_i - 1 as ::core::ffi::c_int) as usize]
                                        - 1 as linenr_T)
                                        as ::core::ffi::c_int)
                                    as linenr_T;
                            }
                            idx_0 += 1;
                        }
                        dp = (*dp).df_next;
                    }
                    if !orig_diff.is_null() {
                        let mut last_diff: *mut diff_T = orig_diff;
                        while !(*last_diff).df_next.is_null() {
                            last_diff = (*last_diff).df_next;
                        }
                        (*last_diff).df_next = (*curtab.get()).tp_first_diff;
                        (*curtab.get()).tp_first_diff = orig_diff;
                    }
                }
                anchor_i += 1;
            }
        }
    }
    xfree((*dio).dio_orig.din_fname as *mut ::core::ffi::c_void);
    xfree((*dio).dio_new.din_fname as *mut ::core::ffi::c_void);
    xfree((*dio).dio_diff.dout_fname as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn diff_internal() -> ::core::ffi::c_int {
    return (diff_flags.get() & DIFF_INTERNAL != 0 as ::core::ffi::c_int
        && *p_dex.get() as ::core::ffi::c_int == NUL) as ::core::ffi::c_int;
}
pub unsafe fn ex_diffupdate(mut eap: *mut exarg_T) {
    let mut idx_new: ::core::ffi::c_int = 0;
    let mut diffio: diffio_T = diffio_T {
        dio_orig: diffin_T {
            din_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            din_mmfile: mmfile_t {
                ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        },
        dio_new: diffin_T {
            din_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            din_mmfile: mmfile_t {
                ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        },
        dio_diff: diffout_T {
            dout_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            dout_ga: garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
        },
        dio_internal: 0,
    };
    if diff_busy.get() {
        diff_need_update.set(true_0 != 0);
        return;
    }
    let mut had_diffs: ::core::ffi::c_int =
        !(*curtab.get()).tp_first_diff.is_null() as ::core::ffi::c_int;
    diff_clear(curtab.get());
    (*curtab.get()).tp_diff_invalid = false_0;
    let mut idx_orig: ::core::ffi::c_int = 0;
    idx_orig = 0 as ::core::ffi::c_int;
    while idx_orig < DB_COUNT {
        if !(*curtab.get()).tp_diffbuf[idx_orig as usize].is_null() {
            break;
        }
        idx_orig += 1;
    }
    if idx_orig != DB_COUNT {
        idx_new = 0;
        idx_new = idx_orig + 1 as ::core::ffi::c_int;
        while idx_new < DB_COUNT {
            if !(*curtab.get()).tp_diffbuf[idx_new as usize].is_null() {
                break;
            }
            idx_new += 1;
        }
        if idx_new != DB_COUNT {
            diffio = diffio_T {
                dio_orig: diffin_T {
                    din_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    din_mmfile: mmfile_t {
                        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        size: 0,
                    },
                },
                dio_new: diffin_T {
                    din_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    din_mmfile: mmfile_t {
                        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        size: 0,
                    },
                },
                dio_diff: diffout_T {
                    dout_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    dout_ga: garray_T {
                        ga_len: 0,
                        ga_maxlen: 0,
                        ga_itemsize: 0,
                        ga_growsize: 0,
                        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    },
                },
                dio_internal: 0,
            };
            diffio.dio_internal = diff_internal();
            diff_try_update(&raw mut diffio, idx_orig, eap);
            (*curwin.get()).w_valid_cursor.lnum = 0 as ::core::ffi::c_int as linenr_T;
        }
    }
    if had_diffs != 0 || !(*curtab.get()).tp_first_diff.is_null() {
        diff_redraw(true_0 != 0);
        apply_autocmds(
            EVENT_DIFFUPDATED,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    }
}
unsafe extern "C" fn check_external_diff(mut diffio: *mut diffio_T) -> ::core::ffi::c_int {
    let mut io_error: bool = false_0 != 0;
    let mut ok: TriState = kFalse;
    loop {
        ok = kFalse;
        let mut fd: *mut FILE = os_fopen(
            (*diffio).dio_orig.din_fname,
            b"w\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if fd.is_null() {
            io_error = true_0 != 0;
        } else {
            if fwrite(
                b"line1\n\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                6 as size_t,
                1 as size_t,
                fd,
            ) != 1 as ::core::ffi::c_ulong
            {
                io_error = true_0 != 0;
            }
            fclose(fd);
            fd = os_fopen(
                (*diffio).dio_new.din_fname,
                b"w\0".as_ptr() as *const ::core::ffi::c_char,
            );
            if fd.is_null() {
                io_error = true_0 != 0;
            } else {
                if fwrite(
                    b"line2\n\0".as_ptr() as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    6 as size_t,
                    1 as size_t,
                    fd,
                ) != 1 as ::core::ffi::c_ulong
                {
                    io_error = true_0 != 0;
                }
                fclose(fd);
                fd = if diff_file(diffio) == OK {
                    os_fopen(
                        (*diffio).dio_diff.dout_fname,
                        b"r\0".as_ptr() as *const ::core::ffi::c_char,
                    )
                } else {
                    ::core::ptr::null_mut::<FILE>()
                };
                if fd.is_null() {
                    io_error = true_0 != 0;
                } else {
                    let mut linebuf: [::core::ffi::c_char; 50] = [0; 50];
                    while !vim_fgets(&raw mut linebuf as *mut ::core::ffi::c_char, LBUFLEN, fd) {
                        if strncmp(
                            &raw mut linebuf as *mut ::core::ffi::c_char,
                            b"1c1\0".as_ptr() as *const ::core::ffi::c_char,
                            3 as size_t,
                        ) == 0 as ::core::ffi::c_int
                            || strncmp(
                                &raw mut linebuf as *mut ::core::ffi::c_char,
                                b"@@ -1 +1 @@\0".as_ptr() as *const ::core::ffi::c_char,
                                11 as size_t,
                            ) == 0 as ::core::ffi::c_int
                        {
                            ok = kTrue;
                        }
                    }
                    fclose(fd);
                }
                os_remove((*diffio).dio_diff.dout_fname);
                os_remove((*diffio).dio_new.din_fname);
            }
            os_remove((*diffio).dio_orig.din_fname);
        }
        if *p_dex.get() as ::core::ffi::c_int != NUL {
            break;
        }
        if diff_a_works.get() as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
            break;
        }
        diff_a_works.set(ok);
        if ok as u64 != 0 {
            break;
        }
    }
    if ok as u64 == 0 {
        if io_error {
            emsg(gettext(
                b"E810: Cannot read or write temp files\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        emsg(gettext(
            b"E97: Cannot create diffs\0".as_ptr() as *const ::core::ffi::c_char
        ));
        diff_a_works.set(kNone);
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn diff_file_internal(mut diffio: *mut diffio_T) -> ::core::ffi::c_int {
    let mut param: xpparam_t = xpparam_t {
        flags: 0,
        anchors: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        anchors_nr: 0,
    };
    let mut emit_cfg: xdemitconf_t = xdemitconf_t {
        ctxlen: 0,
        interhunkctxlen: 0,
        flags: 0,
        find_func: None,
        find_func_priv: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        hunk_func: None,
    };
    let mut emit_cb: xdemitcb_t = xdemitcb_t {
        priv_0: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        out_hunk: None,
        out_line: None,
    };
    memset(
        &raw mut param as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<xpparam_t>(),
    );
    memset(
        &raw mut emit_cfg as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<xdemitconf_t>(),
    );
    memset(
        &raw mut emit_cb as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<xdemitcb_t>(),
    );
    param.flags = diff_algorithm.get() as ::core::ffi::c_ulong;
    if diff_flags.get() & DIFF_IWHITE != 0 {
        param.flags |= XDF_IGNORE_WHITESPACE_CHANGE as ::core::ffi::c_ulong;
    }
    if diff_flags.get() & DIFF_IWHITEALL != 0 {
        param.flags |= XDF_IGNORE_WHITESPACE as ::core::ffi::c_ulong;
    }
    if diff_flags.get() & DIFF_IWHITEEOL != 0 {
        param.flags |= XDF_IGNORE_WHITESPACE_AT_EOL as ::core::ffi::c_ulong;
    }
    if diff_flags.get() & DIFF_IBLANK != 0 {
        param.flags |= XDF_IGNORE_BLANK_LINES as ::core::ffi::c_ulong;
    }
    emit_cfg.ctxlen = 0 as ::core::ffi::c_long;
    emit_cb.priv_0 = &raw mut (*diffio).dio_diff as *mut ::core::ffi::c_void;
    emit_cfg.hunk_func = Some(
        xdiff_out
            as unsafe extern "C" fn(
                ::core::ffi::c_int,
                ::core::ffi::c_int,
                ::core::ffi::c_int,
                ::core::ffi::c_int,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ) as xdl_emit_hunk_consume_func_t;
    if (*diffio).dio_orig.din_mmfile.size as ::core::ffi::c_long > MAX_XDIFF_SIZE
        || (*diffio).dio_new.din_mmfile.size as ::core::ffi::c_long > MAX_XDIFF_SIZE
    {
        emsg(gettext(
            &raw const e_problem_creating_internal_diff as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if xdl_diff(
        &raw mut (*diffio).dio_orig.din_mmfile,
        &raw mut (*diffio).dio_new.din_mmfile,
        &raw mut param,
        &raw mut emit_cfg,
        &raw mut emit_cb,
    ) < 0 as ::core::ffi::c_int
    {
        emsg(gettext(
            &raw const e_problem_creating_internal_diff as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn diff_file(mut dio: *mut diffio_T) -> ::core::ffi::c_int {
    let mut tmp_orig: *mut ::core::ffi::c_char = (*dio).dio_orig.din_fname;
    let mut tmp_new: *mut ::core::ffi::c_char = (*dio).dio_new.din_fname;
    let mut tmp_diff: *mut ::core::ffi::c_char = (*dio).dio_diff.dout_fname;
    if *p_dex.get() as ::core::ffi::c_int != NUL {
        eval_diff(tmp_orig, tmp_new, tmp_diff);
        return OK;
    }
    if (*dio).dio_internal != 0 {
        return diff_file_internal(dio);
    }
    let len: size_t = strlen(tmp_orig)
        .wrapping_add(strlen(tmp_new))
        .wrapping_add(strlen(tmp_diff))
        .wrapping_add(strlen(p_srr.get()))
        .wrapping_add(27 as size_t);
    let cmd: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    if os_env_exists(
        b"DIFF_OPTIONS\0".as_ptr() as *const ::core::ffi::c_char,
        true_0 != 0,
    ) {
        os_unsetenv(b"DIFF_OPTIONS\0".as_ptr() as *const ::core::ffi::c_char);
    }
    vim_snprintf(
        cmd,
        len,
        b"diff %s%s%s%s%s%s%s%s %s\0".as_ptr() as *const ::core::ffi::c_char,
        if diff_a_works.get() as ::core::ffi::c_int == kFalse as ::core::ffi::c_int {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"-a \0".as_ptr() as *const ::core::ffi::c_char
        },
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        if diff_flags.get() & DIFF_IWHITE != 0 {
            b"-b \0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        if diff_flags.get() & DIFF_IWHITEALL != 0 {
            b"-w \0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        if diff_flags.get() & DIFF_IWHITEEOL != 0 {
            b"-Z \0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        if diff_flags.get() & DIFF_IBLANK != 0 {
            b"-B \0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        if diff_flags.get() & DIFF_ICASE != 0 {
            b"-i \0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        tmp_orig,
        tmp_new,
    );
    append_redir(cmd, len, p_srr.get(), tmp_diff);
    block_autocmds();
    call_shell(
        cmd,
        kShellOptFilter as ::core::ffi::c_int
            | kShellOptSilent as ::core::ffi::c_int
            | kShellOptDoOut as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
    unblock_autocmds();
    xfree(cmd as *mut ::core::ffi::c_void);
    return OK;
}
pub unsafe fn ex_diffpatch(mut eap: *mut exarg_T) {
    let mut buflen: size_t = 0;
    let mut dirbuf: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut file_info: FileInfo = FileInfo {
        stat: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
    };
    let mut info_ok: bool = false;
    let mut filesize: uint64_t = 0;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut old_curwin: *mut win_T = curwin.get();
    let mut newname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut esc_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fullname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tmp_orig: *mut ::core::ffi::c_char = vim_tempname();
    let mut tmp_new: *mut ::core::ffi::c_char = vim_tempname();
    if !(tmp_orig.is_null() || tmp_new.is_null()) {
        if buf_write(
            curbuf.get(),
            tmp_orig,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            1 as linenr_T,
            (*curbuf.get()).b_ml.ml_line_count,
            ::core::ptr::null_mut::<exarg_T>(),
            WriteRequest::filter(),
        ) != FAIL
        {
            fullname = FullName_save((*eap).arg, false_0 != 0);
            esc_name = vim_strsave_shellescape(
                if !fullname.is_null() {
                    fullname
                } else {
                    (*eap).arg
                },
                true_0 != 0,
                true_0 != 0,
            );
            buflen = strlen(tmp_orig)
                .wrapping_add(strlen(esc_name))
                .wrapping_add(strlen(tmp_new))
                .wrapping_add(16 as size_t);
            buf = xmalloc(buflen) as *mut ::core::ffi::c_char;
            dirbuf = [0; 4096];
            if os_dirname(
                &raw mut dirbuf as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
            ) != OK
                || os_chdir(&raw mut dirbuf as *mut ::core::ffi::c_char) != 0 as ::core::ffi::c_int
            {
                dirbuf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
            } else {
                let mut tempdir: *mut ::core::ffi::c_char = vim_gettempdir();
                if tempdir.is_null() {
                    tempdir = b"/tmp\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                }
                os_chdir(tempdir);
                shorten_fnames(true_0);
            }
            if *p_pex.get() as ::core::ffi::c_int != NUL {
                eval_patch(
                    tmp_orig,
                    if !fullname.is_null() {
                        fullname
                    } else {
                        (*eap).arg
                    },
                    tmp_new,
                );
            } else {
                vim_snprintf(
                    buf,
                    buflen,
                    b"patch -o %s %s < %s\0".as_ptr() as *const ::core::ffi::c_char,
                    tmp_new,
                    tmp_orig,
                    esc_name,
                );
                block_autocmds();
                call_shell(
                    buf,
                    kShellOptFilter as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                );
                unblock_autocmds();
            }
            if dirbuf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != NUL {
                if os_chdir(&raw mut dirbuf as *mut ::core::ffi::c_char) != 0 as ::core::ffi::c_int
                {
                    emsg(gettext(&raw const e_prev_dir as *const ::core::ffi::c_char));
                }
                shorten_fnames(true_0);
            }
            strcpy(buf, tmp_new);
            strcat(buf, b".orig\0".as_ptr() as *const ::core::ffi::c_char);
            os_remove(buf);
            strcpy(buf, tmp_new);
            strcat(buf, b".rej\0".as_ptr() as *const ::core::ffi::c_char);
            os_remove(buf);
            file_info = FileInfo {
                stat: uv_stat_t {
                    st_dev: 0,
                    st_mode: 0,
                    st_nlink: 0,
                    st_uid: 0,
                    st_gid: 0,
                    st_rdev: 0,
                    st_ino: 0,
                    st_size: 0,
                    st_blksize: 0,
                    st_blocks: 0,
                    st_flags: 0,
                    st_gen: 0,
                    st_atim: uv_timespec_t {
                        tv_sec: 0,
                        tv_nsec: 0,
                    },
                    st_mtim: uv_timespec_t {
                        tv_sec: 0,
                        tv_nsec: 0,
                    },
                    st_ctim: uv_timespec_t {
                        tv_sec: 0,
                        tv_nsec: 0,
                    },
                    st_birthtim: uv_timespec_t {
                        tv_sec: 0,
                        tv_nsec: 0,
                    },
                },
            };
            info_ok = os_fileinfo(tmp_new, &raw mut file_info);
            filesize = os_fileinfo_size(&raw mut file_info);
            if !info_ok || filesize == 0 as uint64_t {
                emsg(gettext(
                    b"E816: Cannot read patch output\0".as_ptr() as *const ::core::ffi::c_char
                ));
            } else {
                if !(*curbuf.get()).b_fname.is_null() {
                    newname = xstrnsave(
                        (*curbuf.get()).b_fname,
                        strlen((*curbuf.get()).b_fname).wrapping_add(4 as size_t),
                    );
                    strcat(newname, b".new\0".as_ptr() as *const ::core::ffi::c_char);
                }
                (*cmdmod.ptr()).cmod_tab = 0 as ::core::ffi::c_int;
                if win_split(
                    0 as ::core::ffi::c_int,
                    if diff_flags.get() & DIFF_VERTICAL != 0 {
                        WSP_VERT as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    },
                ) != FAIL
                {
                    (*eap).cmdidx = CMD_split;
                    (*eap).arg = tmp_new;
                    do_exedit(eap, old_curwin);
                    if curwin.get() != old_curwin
                        && win_valid(old_curwin) as ::core::ffi::c_int != 0
                    {
                        diff_win_options(curwin.get(), true_0 != 0);
                        diff_win_options(old_curwin, true_0 != 0);
                        if !newname.is_null() {
                            (*eap).arg = newname;
                            ex_file(eap);
                            if augroup_exists(
                                b"filetypedetect\0".as_ptr() as *const ::core::ffi::c_char
                            ) {
                                do_cmdline_cmd(b":doau filetypedetect BufRead\0".as_ptr()
                                    as *const ::core::ffi::c_char);
                            }
                        }
                    }
                }
            }
        }
    }
    if !tmp_orig.is_null() {
        os_remove(tmp_orig);
    }
    xfree(tmp_orig as *mut ::core::ffi::c_void);
    if !tmp_new.is_null() {
        os_remove(tmp_new);
    }
    xfree(tmp_new as *mut ::core::ffi::c_void);
    xfree(newname as *mut ::core::ffi::c_void);
    xfree(buf as *mut ::core::ffi::c_void);
    xfree(fullname as *mut ::core::ffi::c_void);
    xfree(esc_name as *mut ::core::ffi::c_void);
}
pub unsafe fn ex_diffsplit(mut eap: *mut exarg_T) {
    let mut old_curwin: *mut win_T = curwin.get();
    let mut old_curbuf: bufref_T = bufref_T::default();
    set_bufref(&raw mut old_curbuf, curbuf.get());
    validate_cursor(curwin.get());
    set_fraction(curwin.get());
    (*cmdmod.ptr()).cmod_tab = 0 as ::core::ffi::c_int;
    if win_split(
        0 as ::core::ffi::c_int,
        if diff_flags.get() & DIFF_VERTICAL != 0 {
            WSP_VERT as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
    ) == FAIL
    {
        return;
    }
    (*eap).cmdidx = CMD_split;
    (*curwin.get()).w_onebuf_opt.wo_diff = true_0;
    do_exedit(eap, old_curwin);
    if curwin.get() == old_curwin {
        return;
    }
    diff_win_options(curwin.get(), true_0 != 0);
    if win_valid(old_curwin) {
        diff_win_options(old_curwin, true_0 != 0);
        if bufref_valid(&raw mut old_curbuf) {
            (*curwin.get()).w_cursor.lnum =
                diff_get_corresponding_line(old_curbuf.br_buf, (*old_curwin).w_cursor.lnum);
        }
    }
    scroll_to_fraction(curwin.get(), (*curwin.get()).w_height);
}
pub unsafe fn ex_diffthis(mut _eap: *mut exarg_T) {
    diff_win_options(curwin.get(), true_0 != 0);
}
unsafe extern "C" fn set_diff_option(mut wp: *mut win_T, mut value: bool) {
    let mut old_curwin: *mut win_T = curwin.get();
    curwin.set(wp);
    curbuf.set((*curwin.get()).w_buffer);
    (*curbuf.get()).b_ro_locked += 1;
    set_option_value_give_err(
        kOptDiff,
        OptVal {
            type_0: kOptValTypeBoolean,
            data: OptValData {
                boolean: value as TriState,
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
    );
    (*curbuf.get()).b_ro_locked -= 1;
    curwin.set(old_curwin);
    curbuf.set((*curwin.get()).w_buffer);
}
pub unsafe extern "C" fn diff_win_options(mut wp: *mut win_T, mut addbuf: bool) {
    let mut old_curwin: *mut win_T = curwin.get();
    curwin.set(wp);
    newFoldLevel();
    curwin.set(old_curwin);
    if (*wp).w_onebuf_opt.wo_diff == 0 {
        (*wp).w_onebuf_opt.wo_scb_save = (*wp).w_onebuf_opt.wo_scb;
    }
    (*wp).w_onebuf_opt.wo_scb = true_0;
    if (*wp).w_onebuf_opt.wo_diff == 0 {
        (*wp).w_onebuf_opt.wo_crb_save = (*wp).w_onebuf_opt.wo_crb;
    }
    (*wp).w_onebuf_opt.wo_crb = true_0;
    if diff_flags.get() & DIFF_FOLLOWWRAP == 0 {
        if (*wp).w_onebuf_opt.wo_diff == 0 {
            (*wp).w_onebuf_opt.wo_wrap_save = (*wp).w_onebuf_opt.wo_wrap;
        }
        (*wp).w_onebuf_opt.wo_wrap = false_0;
        (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
    }
    if (*wp).w_onebuf_opt.wo_diff == 0 {
        if (*wp).w_onebuf_opt.wo_diff_saved != 0 {
            free_string_option((*wp).w_onebuf_opt.wo_fdm_save);
        }
        (*wp).w_onebuf_opt.wo_fdm_save = xstrdup((*wp).w_onebuf_opt.wo_fdm);
    }
    set_option_direct_for(
        kOptFoldmethod,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"diff\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
        0 as scid_T,
        kOptScopeWin,
        wp as *mut ::core::ffi::c_void,
    );
    if (*wp).w_onebuf_opt.wo_diff == 0 {
        (*wp).w_onebuf_opt.wo_fen_save = (*wp).w_onebuf_opt.wo_fen;
        (*wp).w_onebuf_opt.wo_fdl_save = (*wp).w_onebuf_opt.wo_fdl;
        if (*wp).w_onebuf_opt.wo_diff_saved != 0 {
            free_string_option((*wp).w_onebuf_opt.wo_fdc_save);
        }
        (*wp).w_onebuf_opt.wo_fdc_save = xstrdup((*wp).w_onebuf_opt.wo_fdc);
    }
    free_string_option((*wp).w_onebuf_opt.wo_fdc);
    (*wp).w_onebuf_opt.wo_fdc = xstrdup(b"2\0".as_ptr() as *const ::core::ffi::c_char);
    '_c2rust_label: {
        if diff_foldcolumn.get() >= 0 as ::core::ffi::c_int
            && diff_foldcolumn.get() <= 9 as ::core::ffi::c_int
        {
        } else {
            __assert_fail(
                b"diff_foldcolumn >= 0 && diff_foldcolumn <= 9\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/diff.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1573 as ::core::ffi::c_uint,
                b"void diff_win_options(win_T *, _Bool)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    snprintf(
        (*wp).w_onebuf_opt.wo_fdc,
        strlen((*wp).w_onebuf_opt.wo_fdc).wrapping_add(1 as size_t),
        b"%d\0".as_ptr() as *const ::core::ffi::c_char,
        diff_foldcolumn.get(),
    );
    (*wp).w_onebuf_opt.wo_fen = true_0;
    (*wp).w_onebuf_opt.wo_fdl = 0 as OptInt;
    foldUpdateAll(wp);
    changed_window_setting(wp);
    if vim_strchr(p_sbo.get(), 'h' as ::core::ffi::c_int).is_null() {
        do_cmdline_cmd(b"set sbo+=hor\0".as_ptr() as *const ::core::ffi::c_char);
    }
    (*wp).w_onebuf_opt.wo_diff_saved = true_0;
    set_diff_option(wp, true_0 != 0);
    if addbuf {
        diff_buf_add((*wp).w_buffer);
    }
    redraw_later(wp, UPD_NOT_VALID);
}
pub unsafe fn ex_diffoff(mut eap: *mut exarg_T) {
    let mut diffwin: bool = false_0 != 0;
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if if (*eap).forceit != 0 {
            (*wp).w_onebuf_opt.wo_diff
        } else {
            (wp == curwin.get()) as ::core::ffi::c_int
        } != 0
        {
            set_diff_option(wp, false_0 != 0);
            if (*wp).w_onebuf_opt.wo_diff_saved != 0 {
                if (*wp).w_onebuf_opt.wo_scb != 0 {
                    (*wp).w_onebuf_opt.wo_scb = (*wp).w_onebuf_opt.wo_scb_save;
                }
                if (*wp).w_onebuf_opt.wo_crb != 0 {
                    (*wp).w_onebuf_opt.wo_crb = (*wp).w_onebuf_opt.wo_crb_save;
                }
                if diff_flags.get() & DIFF_FOLLOWWRAP == 0 {
                    if (*wp).w_onebuf_opt.wo_wrap == 0 && (*wp).w_onebuf_opt.wo_wrap_save != 0 {
                        (*wp).w_onebuf_opt.wo_wrap = true_0;
                        (*wp).w_leftcol = 0 as ::core::ffi::c_int as colnr_T;
                    }
                }
                free_string_option((*wp).w_onebuf_opt.wo_fdm);
                (*wp).w_onebuf_opt.wo_fdm = xstrdup(
                    if *(*wp).w_onebuf_opt.wo_fdm_save as ::core::ffi::c_int != 0 {
                        (*wp).w_onebuf_opt.wo_fdm_save as *const ::core::ffi::c_char
                    } else {
                        b"manual\0".as_ptr() as *const ::core::ffi::c_char
                    },
                );
                free_string_option((*wp).w_onebuf_opt.wo_fdc);
                (*wp).w_onebuf_opt.wo_fdc = xstrdup(
                    if *(*wp).w_onebuf_opt.wo_fdc_save as ::core::ffi::c_int != 0 {
                        (*wp).w_onebuf_opt.wo_fdc_save as *const ::core::ffi::c_char
                    } else {
                        b"0\0".as_ptr() as *const ::core::ffi::c_char
                    },
                );
                if (*wp).w_onebuf_opt.wo_fdl == 0 as OptInt {
                    (*wp).w_onebuf_opt.wo_fdl = (*wp).w_onebuf_opt.wo_fdl_save;
                }
                if (*wp).w_onebuf_opt.wo_fen != 0 {
                    (*wp).w_onebuf_opt.wo_fen = if foldmethodIsManual(wp) as ::core::ffi::c_int != 0
                    {
                        false_0
                    } else {
                        (*wp).w_onebuf_opt.wo_fen_save
                    };
                }
                foldUpdateAll(wp);
            }
            (*wp).w_topfill = 0 as ::core::ffi::c_int;
            changed_window_setting(wp);
            diff_buf_adjust(wp);
        }
        diffwin = diffwin as ::core::ffi::c_int | (*wp).w_onebuf_opt.wo_diff != 0;
        wp = (*wp).w_next;
    }
    if (*eap).forceit != 0 {
        diff_buf_clear();
    }
    if !diffwin {
        diff_need_update.set(false_0 != 0);
        (*curtab.get()).tp_diff_invalid = false_0;
        (*curtab.get()).tp_diff_update = false_0;
        diff_clear(curtab.get());
    }
    if !diffwin && !vim_strchr(p_sbo.get(), 'h' as ::core::ffi::c_int).is_null() {
        do_cmdline_cmd(b"set sbo-=hor\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
unsafe extern "C" fn extract_hunk_internal(
    mut dout: *mut diffout_T,
    mut hunk: *mut diffhunk_T,
    mut line_idx: *mut ::core::ffi::c_int,
) -> bool {
    let mut eof: bool = *line_idx >= (*dout).dout_ga.ga_len;
    if !eof {
        let c2rust_fresh7 = *line_idx;
        *line_idx = *line_idx + 1;
        *hunk = *((*dout).dout_ga.ga_data as *mut diffhunk_T).offset(c2rust_fresh7 as isize);
    }
    return eof;
}
unsafe extern "C" fn extract_hunk(
    mut fd: *mut FILE,
    mut hunk: *mut diffhunk_T,
    mut diffstyle: *mut diffstyle_T,
) -> bool {
    loop {
        let mut line: [::core::ffi::c_char; 50] = [0; 50];
        if vim_fgets(&raw mut line as *mut ::core::ffi::c_char, LBUFLEN, fd) {
            return true_0 != 0;
        }
        if *diffstyle as ::core::ffi::c_uint
            == DIFF_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if *(*__ctype_b_loc()).offset(
                *(&raw mut line as *mut ::core::ffi::c_char) as uint8_t as ::core::ffi::c_int
                    as isize,
            ) as ::core::ffi::c_int
                & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
            {
                *diffstyle = DIFF_ED;
            } else if strncmp(
                &raw mut line as *mut ::core::ffi::c_char,
                b"@@ \0".as_ptr() as *const ::core::ffi::c_char,
                3 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                *diffstyle = DIFF_UNIFIED;
            } else {
                if !(strncmp(
                    &raw mut line as *mut ::core::ffi::c_char,
                    b"--- \0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
                    && vim_fgets(&raw mut line as *mut ::core::ffi::c_char, LBUFLEN, fd)
                        as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    && strncmp(
                        &raw mut line as *mut ::core::ffi::c_char,
                        b"+++ \0".as_ptr() as *const ::core::ffi::c_char,
                        4 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    && vim_fgets(&raw mut line as *mut ::core::ffi::c_char, LBUFLEN, fd)
                        as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    && strncmp(
                        &raw mut line as *mut ::core::ffi::c_char,
                        b"@@ \0".as_ptr() as *const ::core::ffi::c_char,
                        3 as size_t,
                    ) == 0 as ::core::ffi::c_int)
                {
                    continue;
                }
                *diffstyle = DIFF_UNIFIED;
            }
        }
        if *diffstyle as ::core::ffi::c_uint == DIFF_ED as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if *(*__ctype_b_loc()).offset(
                *(&raw mut line as *mut ::core::ffi::c_char) as uint8_t as ::core::ffi::c_int
                    as isize,
            ) as ::core::ffi::c_int
                & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                == 0
            {
                continue;
            }
            if parse_diff_ed(&raw mut line as *mut ::core::ffi::c_char, hunk) == FAIL {
                continue;
            }
        } else {
            '_c2rust_label: {
                if *diffstyle as ::core::ffi::c_uint
                    == DIFF_UNIFIED as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                } else {
                    __assert_fail(
                        b"*diffstyle == DIFF_UNIFIED\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/diff.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1726 as ::core::ffi::c_uint,
                        b"_Bool extract_hunk(FILE *, diffhunk_T *, diffstyle_T *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            if strncmp(
                &raw mut line as *mut ::core::ffi::c_char,
                b"@@ \0".as_ptr() as *const ::core::ffi::c_char,
                3 as size_t,
            ) != 0 as ::core::ffi::c_int
            {
                continue;
            }
            if parse_diff_unified(&raw mut line as *mut ::core::ffi::c_char, hunk) == FAIL {
                continue;
            }
        }
        return false_0 != 0;
    }
}
unsafe extern "C" fn process_hunk(
    mut dpp: *mut *mut diff_T,
    mut dprevp: *mut *mut diff_T,
    mut idx_orig: ::core::ffi::c_int,
    mut idx_new: ::core::ffi::c_int,
    mut hunk: *mut diffhunk_T,
    mut notsetp: *mut bool,
) {
    let mut dp: *mut diff_T = *dpp;
    let mut dprev: *mut diff_T = *dprevp;
    while !dp.is_null()
        && (*hunk).lnum_orig > (*dp).df_lnum[idx_orig as usize] + (*dp).df_count[idx_orig as usize]
    {
        if *notsetp {
            diff_copy_entry(dprev, dp, idx_orig, idx_new);
        }
        dprev = dp;
        dp = (*dp).df_next;
        *notsetp = true_0 != 0;
    }
    if !dp.is_null()
        && (*hunk).lnum_orig <= (*dp).df_lnum[idx_orig as usize] + (*dp).df_count[idx_orig as usize]
        && (*hunk).lnum_orig + (*hunk).count_orig as linenr_T >= (*dp).df_lnum[idx_orig as usize]
    {
        let mut dpl: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        dpl = dp;
        while !(*dpl).df_next.is_null() {
            if ((*hunk).lnum_orig + (*hunk).count_orig as linenr_T)
                < (*(*dpl).df_next).df_lnum[idx_orig as usize]
            {
                break;
            }
            dpl = (*dpl).df_next;
        }
        let mut off: linenr_T = (*dp).df_lnum[idx_orig as usize] - (*hunk).lnum_orig;
        if off > 0 as linenr_T {
            let mut i: ::core::ffi::c_int = idx_orig;
            while i < idx_new {
                if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
                    (*dp).df_lnum[i as usize] -= off;
                    (*dp).df_count[i as usize] += off;
                }
                i += 1;
            }
            (*dp).df_lnum[idx_new as usize] = (*hunk).lnum_new;
            (*dp).df_count[idx_new as usize] = (*hunk).count_new as linenr_T;
        } else if *notsetp {
            (*dp).df_lnum[idx_new as usize] = (*hunk).lnum_new + off;
            (*dp).df_count[idx_new as usize] = (*hunk).count_new as linenr_T - off;
        } else {
            let mut orig_size_in_dp: ::core::ffi::c_int = if ((*hunk).count_orig as linenr_T)
                < (*dp).df_lnum[idx_orig as usize] + (*dp).df_count[idx_orig as usize]
                    - (*hunk).lnum_orig
            {
                (*hunk).count_orig
            } else {
                (*dp).df_lnum[idx_orig as usize] as ::core::ffi::c_int
                    + (*dp).df_count[idx_orig as usize] as ::core::ffi::c_int
                    - (*hunk).lnum_orig as ::core::ffi::c_int
            };
            let mut size_diff: ::core::ffi::c_int = (*hunk).count_new - orig_size_in_dp;
            (*dp).df_count[idx_new as usize] =
                ((*dp).df_count[idx_new as usize] as ::core::ffi::c_int + size_diff) as linenr_T;
            off = (*hunk).lnum_new + (*hunk).count_new as linenr_T
                - ((*dp).df_lnum[idx_new as usize] + (*dp).df_count[idx_new as usize]);
            if off > 0 as linenr_T {
                (*dp).df_count[idx_new as usize] += off;
            }
        }
        off = (*hunk).lnum_orig + (*hunk).count_orig as linenr_T
            - ((*dpl).df_lnum[idx_orig as usize] + (*dpl).df_count[idx_orig as usize]);
        if off < 0 as linenr_T {
            if *notsetp as ::core::ffi::c_int != 0 || dp != dpl {
                (*dp).df_count[idx_new as usize] += -off;
            }
            off = 0 as ::core::ffi::c_int as linenr_T;
        }
        let mut i_0: ::core::ffi::c_int = idx_orig;
        while i_0 < idx_new {
            if !(*curtab.get()).tp_diffbuf[i_0 as usize].is_null() {
                (*dp).df_count[i_0 as usize] = (*dpl).df_lnum[i_0 as usize]
                    + (*dpl).df_count[i_0 as usize]
                    - (*dp).df_lnum[i_0 as usize]
                    + off;
            }
            i_0 += 1;
        }
        let mut dn: *mut diff_T = (*dp).df_next;
        (*dp).df_next = (*dpl).df_next;
        while dn != (*dp).df_next {
            dpl = (*dn).df_next;
            clear_diffblock(dn);
            dn = dpl;
        }
    } else {
        dp = diff_alloc_new(curtab.get(), dprev, dp);
        (*dp).df_lnum[idx_orig as usize] = (*hunk).lnum_orig;
        (*dp).df_count[idx_orig as usize] = (*hunk).count_orig as linenr_T;
        (*dp).df_lnum[idx_new as usize] = (*hunk).lnum_new;
        (*dp).df_count[idx_new as usize] = (*hunk).count_new as linenr_T;
        let mut i_1: ::core::ffi::c_int = idx_orig + 1 as ::core::ffi::c_int;
        while i_1 < idx_new {
            if !(*curtab.get()).tp_diffbuf[i_1 as usize].is_null() {
                diff_copy_entry(dprev, dp, idx_orig, i_1);
            }
            i_1 += 1;
        }
    }
    *notsetp = false_0 != 0;
    *dpp = dp;
    *dprevp = dprev;
}
unsafe extern "C" fn diff_read(
    mut idx_orig: ::core::ffi::c_int,
    mut idx_new: ::core::ffi::c_int,
    mut dio: *mut diffio_T,
) {
    let mut fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut line_hunk_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut dprev: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
    let mut dout: *mut diffout_T = &raw mut (*dio).dio_diff;
    let mut notset: bool = true_0 != 0;
    let mut diffstyle: diffstyle_T = DIFF_NONE;
    if (*dio).dio_internal == 0 {
        fd = os_fopen(
            (*dout).dout_fname,
            b"r\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if fd.is_null() {
            emsg(gettext(
                b"E98: Cannot read diff output\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return;
        }
    }
    loop {
        let mut hunk: diffhunk_T = diffhunk_T {
            lnum_orig: 0 as linenr_T,
            count_orig: 0,
            lnum_new: 0,
            count_new: 0,
        };
        let mut eof: bool = if (*dio).dio_internal != 0 {
            extract_hunk_internal(dout, &raw mut hunk, &raw mut line_hunk_idx) as ::core::ffi::c_int
        } else {
            extract_hunk(fd, &raw mut hunk, &raw mut diffstyle) as ::core::ffi::c_int
        } != 0;
        if eof {
            break;
        }
        process_hunk(
            &raw mut dp,
            &raw mut dprev,
            idx_orig,
            idx_new,
            &raw mut hunk,
            &raw mut notset,
        );
    }
    while !dp.is_null() {
        if notset {
            diff_copy_entry(dprev, dp, idx_orig, idx_new);
        }
        dprev = dp;
        dp = (*dp).df_next;
        notset = true_0 != 0;
    }
    if !fd.is_null() {
        fclose(fd);
    }
}
unsafe extern "C" fn diff_copy_entry(
    mut dprev: *mut diff_T,
    mut dp: *mut diff_T,
    mut idx_orig: ::core::ffi::c_int,
    mut idx_new: ::core::ffi::c_int,
) {
    let mut off: linenr_T = 0;
    if dprev.is_null() {
        off = 0 as ::core::ffi::c_int as linenr_T;
    } else {
        off = (*dprev).df_lnum[idx_orig as usize] + (*dprev).df_count[idx_orig as usize]
            - ((*dprev).df_lnum[idx_new as usize] + (*dprev).df_count[idx_new as usize]);
    }
    (*dp).df_lnum[idx_new as usize] = (*dp).df_lnum[idx_orig as usize] - off;
    (*dp).df_count[idx_new as usize] = (*dp).df_count[idx_orig as usize];
}
pub unsafe extern "C" fn diff_clear(mut tp: *mut tabpage_T) {
    let mut next_p: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut p: *mut diff_T = (*tp).tp_first_diff;
    while !p.is_null() {
        next_p = (*p).df_next;
        clear_diffblock(p);
        p = next_p;
    }
    (*tp).tp_first_diff = ::core::ptr::null_mut::<diff_T>();
}
pub unsafe extern "C" fn diff_linematch(mut dp: *mut diff_T) -> bool {
    if diff_flags.get() & DIFF_LINEMATCH == 0 {
        return false_0 != 0;
    }
    let mut tsize: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
            if (*dp).df_count[i as usize] < 0 as linenr_T {
                return false_0 != 0;
            }
            tsize += (*dp).df_count[i as usize] as ::core::ffi::c_int;
        }
        i += 1;
    }
    return tsize <= linematch_lines.get();
}
unsafe extern "C" fn get_max_diff_length(mut dp: *const diff_T) -> ::core::ffi::c_int {
    let mut maxlength: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while k < DB_COUNT {
        if !(*curtab.get()).tp_diffbuf[k as usize].is_null() {
            if (*dp).df_count[k as usize] > maxlength as linenr_T {
                maxlength = (*dp).df_count[k as usize] as ::core::ffi::c_int;
            }
        }
        k += 1;
    }
    return maxlength;
}
unsafe extern "C" fn find_top_diff_block(
    mut thistopdiff: *mut *mut diff_T,
    mut next_adjacent_blocks: *mut *mut diff_T,
    mut fromidx: ::core::ffi::c_int,
    mut topline: ::core::ffi::c_int,
) {
    let mut topdiff: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut localtopdiff: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut topdiffchange: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    topdiff = (*curtab.get()).tp_first_diff;
    while !topdiff.is_null() {
        if localtopdiff.is_null() || topdiffchange != 0 {
            localtopdiff = topdiff;
            topdiffchange = 0 as ::core::ffi::c_int;
        }
        if topline as linenr_T >= (*topdiff).df_lnum[fromidx as usize]
            && topline as linenr_T
                <= (*topdiff).df_lnum[fromidx as usize] + (*topdiff).df_count[fromidx as usize]
        {
            if (*thistopdiff).is_null() {
                *thistopdiff = localtopdiff;
            }
        }
        if !(!(*topdiff).df_next.is_null()
            && (*(*topdiff).df_next).df_lnum[fromidx as usize]
                == (*topdiff).df_lnum[fromidx as usize] + (*topdiff).df_count[fromidx as usize])
        {
            topdiffchange = 1 as ::core::ffi::c_int;
            if !(*thistopdiff).is_null() {
                *next_adjacent_blocks = (*topdiff).df_next;
                break;
            }
        }
        topdiff = (*topdiff).df_next;
    }
}
unsafe extern "C" fn calculate_topfill_and_topline(
    fromidx: ::core::ffi::c_int,
    toidx: ::core::ffi::c_int,
    from_topline: ::core::ffi::c_int,
    from_topfill: ::core::ffi::c_int,
    mut topfill: *mut ::core::ffi::c_int,
    mut topline: *mut linenr_T,
) {
    let mut thistopdiff: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut next_adjacent_blocks: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut virtual_lines_passed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    find_top_diff_block(
        &raw mut thistopdiff,
        &raw mut next_adjacent_blocks,
        fromidx,
        from_topline,
    );
    let mut curdif: *mut diff_T = thistopdiff;
    while !curdif.is_null()
        && (*curdif).df_lnum[fromidx as usize] + (*curdif).df_count[fromidx as usize]
            <= from_topline as linenr_T
    {
        virtual_lines_passed += get_max_diff_length(curdif);
        curdif = (*curdif).df_next;
    }
    if curdif != next_adjacent_blocks {
        virtual_lines_passed +=
            (from_topline as linenr_T - (*curdif).df_lnum[fromidx as usize]) as ::core::ffi::c_int;
    }
    virtual_lines_passed -= from_topfill;
    if virtual_lines_passed < 0 as ::core::ffi::c_int {
        virtual_lines_passed = 0 as ::core::ffi::c_int;
    }
    let mut curlinenum_to: ::core::ffi::c_int = if !thistopdiff.is_null() {
        (*thistopdiff).df_lnum[toidx as usize] as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
    let mut virt_lines_left: ::core::ffi::c_int = virtual_lines_passed;
    curdif = thistopdiff;
    while virt_lines_left > 0 as ::core::ffi::c_int
        && !curdif.is_null()
        && curdif != next_adjacent_blocks
    {
        curlinenum_to += (if (virt_lines_left as linenr_T) < (*curdif).df_count[toidx as usize] {
            virt_lines_left as linenr_T
        } else {
            (*curdif).df_count[toidx as usize]
        }) as ::core::ffi::c_int;
        virt_lines_left -= if virt_lines_left < get_max_diff_length(curdif) {
            virt_lines_left
        } else {
            get_max_diff_length(curdif)
        };
        curdif = (*curdif).df_next;
    }
    let mut max_virt_lines: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut dp: *mut diff_T = thistopdiff;
    while !dp.is_null() {
        if (*dp).df_lnum[toidx as usize] + (*dp).df_count[toidx as usize]
            <= curlinenum_to as linenr_T
        {
            max_virt_lines += get_max_diff_length(dp);
            dp = (*dp).df_next;
        } else {
            if (*dp).df_lnum[toidx as usize] <= curlinenum_to as linenr_T {
                max_virt_lines += (curlinenum_to as linenr_T - (*dp).df_lnum[toidx as usize])
                    as ::core::ffi::c_int;
            }
            break;
        }
    }
    if diff_flags.get() & DIFF_FILLER != 0 {
        *topfill = max_virt_lines - virtual_lines_passed;
    }
    *topline = curlinenum_to as linenr_T;
}
unsafe extern "C" fn apply_linematch_results(
    mut dp: *mut diff_T,
    mut decisions_length: size_t,
    mut decisions: *const ::core::ffi::c_int,
) {
    let mut line_numbers: [::core::ffi::c_int; 8] = [0; 8];
    let mut outputmap: [::core::ffi::c_int; 8] = [0; 8];
    let mut ndiffs: size_t = 0 as size_t;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
            line_numbers[i as usize] = (*dp).df_lnum[i as usize] as ::core::ffi::c_int;
            (*dp).df_count[i as usize] = 0 as ::core::ffi::c_int as linenr_T;
            outputmap[ndiffs as usize] = i;
            ndiffs = ndiffs.wrapping_add(1);
        }
        i += 1;
    }
    let mut dp_s: *mut diff_T = dp;
    let mut i_0: size_t = 0 as size_t;
    while i_0 < decisions_length {
        if i_0 != 0 as size_t
            && *decisions.offset(i_0.wrapping_sub(1 as size_t) as isize)
                != *decisions.offset(i_0 as isize)
        {
            dp_s = diff_alloc_new(curtab.get(), dp_s, (*dp_s).df_next);
            (*dp_s).is_linematched = true_0 != 0;
            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while j < DB_COUNT {
                if !(*curtab.get()).tp_diffbuf[j as usize].is_null() {
                    (*dp_s).df_lnum[j as usize] = line_numbers[j as usize] as linenr_T;
                    (*dp_s).df_count[j as usize] = 0 as ::core::ffi::c_int as linenr_T;
                }
                j += 1;
            }
        }
        let mut j_0: size_t = 0 as size_t;
        while j_0 < ndiffs {
            if *decisions.offset(i_0 as isize) & (1 as ::core::ffi::c_int) << j_0 != 0 {
                (*dp_s).df_count[outputmap[j_0 as usize] as usize] += 1;
                line_numbers[outputmap[j_0 as usize] as usize] += 1;
            }
            j_0 = j_0.wrapping_add(1);
        }
        i_0 = i_0.wrapping_add(1);
    }
    (*dp).is_linematched = true_0 != 0;
}
unsafe extern "C" fn run_linematch_algorithm(mut dp: *mut diff_T) {
    let mut diffbufs_mm: [mmfile_t; 8] = [mmfile_t {
        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    }; 8];
    let mut diff_length: [::core::ffi::c_int; 8] = [0; 8];
    let mut ndiffs: size_t = 0 as size_t;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
            if (*dp).df_count[i as usize] > 0 as linenr_T {
                diff_write_buffer(
                    (*curtab.get()).tp_diffbuf[i as usize] as *mut buf_T,
                    (&raw mut diffbufs_mm as *mut mmfile_t).offset(ndiffs as isize),
                    (*dp).df_lnum[i as usize],
                    (*dp).df_lnum[i as usize] + (*dp).df_count[i as usize] - 1 as linenr_T,
                );
            } else {
                diffbufs_mm[ndiffs as usize].size = 0 as ::core::ffi::c_int;
                diffbufs_mm[ndiffs as usize].ptr = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            diff_length[ndiffs as usize] = (*dp).df_count[i as usize] as ::core::ffi::c_int;
            ndiffs = ndiffs.wrapping_add(1);
        }
        i += 1;
    }
    let iwhite: bool = diff_flags.get() & (DIFF_IWHITEALL | DIFF_IWHITE) > 0 as ::core::ffi::c_int;
    // `diff_write_buffer` leaves an empty block as a NULL pointer, which
    // `from_raw_parts` may not see; those axes have zero length anyway.
    let mut blocks: [&[u8]; 8] = [&[]; 8];
    for (block, mm) in blocks.iter_mut().zip(&diffbufs_mm[..ndiffs]) {
        if !mm.ptr.is_null() {
            *block = ::core::slice::from_raw_parts(mm.ptr as *const u8, mm.size as usize);
        }
    }
    let decisions = linematch_nbuffers(&blocks[..ndiffs], &diff_length[..ndiffs], iwhite);
    let mut i_0: size_t = 0 as size_t;
    while i_0 < ndiffs {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*(&raw mut diffbufs_mm as *mut mmfile_t).offset(i_0 as isize)).ptr
                as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        i_0 = i_0.wrapping_add(1);
    }
    apply_linematch_results(dp, decisions.len(), decisions.as_ptr());
}
pub unsafe extern "C" fn diff_check_with_linestatus(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut linestatus: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut buf: *mut buf_T = (*wp).w_buffer;
    if !linestatus.is_null() {
        *linestatus = 0 as ::core::ffi::c_int;
    }
    if (*curtab.get()).tp_diff_invalid != 0 {
        ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
    }
    if (*curtab.get()).tp_first_diff.is_null() || (*wp).w_onebuf_opt.wo_diff == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if lnum < 1 as linenr_T || lnum > (*buf).b_ml.ml_line_count + 1 as linenr_T {
        return 0 as ::core::ffi::c_int;
    }
    let mut idx: ::core::ffi::c_int = diff_buf_idx(buf, curtab.get());
    if idx == DB_COUNT {
        return 0 as ::core::ffi::c_int;
    }
    if hasFolding(
        wp,
        lnum,
        ::core::ptr::null_mut::<linenr_T>(),
        ::core::ptr::null_mut::<linenr_T>(),
    ) as ::core::ffi::c_int
        != 0
        || decor_conceal_line(
            wp,
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            false_0 != 0,
        ) as ::core::ffi::c_int
            != 0
    {
        return 0 as ::core::ffi::c_int;
    }
    let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    dp = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if lnum <= (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
            break;
        }
        dp = (*dp).df_next;
    }
    if dp.is_null() || lnum < (*dp).df_lnum[idx as usize] {
        return 0 as ::core::ffi::c_int;
    }
    if lnum >= (*wp).w_topline
        && lnum < (*wp).w_botline
        && !(*dp).is_linematched
        && diff_linematch(dp) as ::core::ffi::c_int != 0
        && diff_check_sanity(curtab.get(), dp) != 0
    {
        run_linematch_algorithm(dp);
    }
    let mut num_fill: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lnum == (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
        if diff_flags.get() & DIFF_FILLER != 0 {
            let mut maxcount: ::core::ffi::c_int = get_max_diff_length(dp);
            num_fill += (maxcount as linenr_T - (*dp).df_count[idx as usize]) as ::core::ffi::c_int;
        }
        if !(!(*dp).df_next.is_null()
            && lnum >= (*(*dp).df_next).df_lnum[idx as usize]
            && lnum
                <= (*(*dp).df_next).df_lnum[idx as usize] + (*(*dp).df_next).df_count[idx as usize])
        {
            break;
        }
        dp = (*dp).df_next;
    }
    if lnum < (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
        let mut zero: bool = false_0 != 0;
        let mut cmp: bool = false_0 != 0;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < DB_COUNT {
            if i != idx && !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
                if (*dp).df_count[i as usize] == 0 as linenr_T {
                    zero = true_0 != 0;
                } else {
                    if (*dp).df_count[i as usize] != (*dp).df_count[idx as usize] {
                        if !linestatus.is_null() {
                            *linestatus = -1 as ::core::ffi::c_int;
                        }
                        return num_fill;
                    }
                    cmp = true_0 != 0;
                }
            }
            i += 1;
        }
        if cmp {
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < DB_COUNT {
                if i_0 != idx
                    && !(*curtab.get()).tp_diffbuf[i_0 as usize].is_null()
                    && (*dp).df_count[i_0 as usize] != 0 as linenr_T
                {
                    if !diff_equal_entry(dp, idx, i_0) {
                        if !linestatus.is_null() {
                            *linestatus = -1 as ::core::ffi::c_int;
                        }
                        return num_fill;
                    }
                }
                i_0 += 1;
            }
        }
        if !zero {
            return num_fill;
        }
        if !linestatus.is_null() {
            *linestatus = -2 as ::core::ffi::c_int;
        }
        return num_fill;
    }
    return num_fill;
}
pub unsafe extern "C" fn diff_check_fill(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
) -> ::core::ffi::c_int {
    if diff_flags.get() & DIFF_FILLER == 0 {
        return 0 as ::core::ffi::c_int;
    }
    let mut n: ::core::ffi::c_int =
        diff_check_with_linestatus(wp, lnum, ::core::ptr::null_mut::<::core::ffi::c_int>());
    return if n > 0 as ::core::ffi::c_int {
        n
    } else {
        0 as ::core::ffi::c_int
    };
}
unsafe extern "C" fn diff_equal_entry(
    mut dp: *mut diff_T,
    mut idx1: ::core::ffi::c_int,
    mut idx2: ::core::ffi::c_int,
) -> bool {
    if (*dp).df_count[idx1 as usize] != (*dp).df_count[idx2 as usize] {
        return false_0 != 0;
    }
    if diff_check_sanity(curtab.get(), dp) == FAIL {
        return false_0 != 0;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (i as linenr_T) < (*dp).df_count[idx1 as usize] {
        let mut line: *mut ::core::ffi::c_char = xstrdup(ml_get_buf(
            (*curtab.get()).tp_diffbuf[idx1 as usize] as *mut buf_T,
            (*dp).df_lnum[idx1 as usize] + i as linenr_T,
        ));
        let mut cmp: ::core::ffi::c_int = diff_cmp(
            line,
            ml_get_buf(
                (*curtab.get()).tp_diffbuf[idx2 as usize] as *mut buf_T,
                (*dp).df_lnum[idx2 as usize] + i as linenr_T,
            ),
        );
        xfree(line as *mut ::core::ffi::c_void);
        if cmp != 0 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        i += 1;
    }
    return true_0 != 0;
}
unsafe extern "C" fn diff_equal_char(
    p1: *const ::core::ffi::c_char,
    p2: *const ::core::ffi::c_char,
    len: *mut ::core::ffi::c_int,
) -> bool {
    let l: ::core::ffi::c_int = utfc_ptr2len(p1);
    if l != utfc_ptr2len(p2) {
        return false_0 != 0;
    }
    if l > 1 as ::core::ffi::c_int {
        if strncmp(p1, p2, l as size_t) != 0 as ::core::ffi::c_int
            && (diff_flags.get() & DIFF_ICASE == 0
                || utf_fold(utf_ptr2char(p1)) != utf_fold(utf_ptr2char(p2)))
        {
            return false_0 != 0;
        }
        *len = l;
    } else {
        if *p1 as ::core::ffi::c_int != *p2 as ::core::ffi::c_int
            && (diff_flags.get() & DIFF_ICASE == 0
                || tolower(*p1 as uint8_t as ::core::ffi::c_int)
                    != tolower(*p2 as uint8_t as ::core::ffi::c_int))
        {
            return false_0 != 0;
        }
        *len = 1 as ::core::ffi::c_int;
    }
    return true_0 != 0;
}
unsafe extern "C" fn diff_cmp(
    mut s1: *mut ::core::ffi::c_char,
    mut s2: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if diff_flags.get() & DIFF_IBLANK != 0
        && (*skipwhite(s1) as ::core::ffi::c_int == NUL
            || *skipwhite(s2) as ::core::ffi::c_int == NUL)
    {
        return 0 as ::core::ffi::c_int;
    }
    if diff_flags.get() & (DIFF_ICASE | ALL_WHITE_DIFF) == 0 as ::core::ffi::c_int {
        return strcmp(s1, s2);
    }
    if diff_flags.get() & DIFF_ICASE != 0 && diff_flags.get() & ALL_WHITE_DIFF == 0 {
        return mb_stricmp(s1, s2);
    }
    let mut p1: *mut ::core::ffi::c_char = s1;
    let mut p2: *mut ::core::ffi::c_char = s2;
    while *p1 as ::core::ffi::c_int != NUL && *p2 as ::core::ffi::c_int != NUL {
        if diff_flags.get() & DIFF_IWHITE != 0
            && ascii_iswhite(*p1 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            && ascii_iswhite(*p2 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            || diff_flags.get() & DIFF_IWHITEALL != 0
                && (ascii_iswhite(*p1 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                    || ascii_iswhite(*p2 as ::core::ffi::c_int) as ::core::ffi::c_int != 0)
        {
            p1 = skipwhite(p1);
            p2 = skipwhite(p2);
        } else {
            let mut l: ::core::ffi::c_int = 0;
            if !diff_equal_char(p1, p2, &raw mut l) {
                break;
            }
            p1 = p1.offset(l as isize);
            p2 = p2.offset(l as isize);
        }
    }
    p1 = skipwhite(p1);
    p2 = skipwhite(p2);
    if *p1 as ::core::ffi::c_int != NUL || *p2 as ::core::ffi::c_int != NUL {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn diff_set_topline(mut fromwin: *mut win_T, mut towin: *mut win_T) {
    let mut frombuf: *mut buf_T = (*fromwin).w_buffer;
    let mut fromidx: ::core::ffi::c_int = diff_buf_idx(frombuf, curtab.get());
    if fromidx == DB_COUNT {
        return;
    }
    if (*curtab.get()).tp_diff_invalid != 0 {
        ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
    }
    let mut lnum: linenr_T = (*fromwin).w_topline;
    (*towin).w_topfill = 0 as ::core::ffi::c_int;
    let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    dp = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if lnum <= (*dp).df_lnum[fromidx as usize] + (*dp).df_count[fromidx as usize] {
            break;
        }
        dp = (*dp).df_next;
    }
    if dp.is_null() {
        (*towin).w_topline =
            (*(*towin).w_buffer).b_ml.ml_line_count - ((*frombuf).b_ml.ml_line_count - lnum);
    } else {
        let mut toidx: ::core::ffi::c_int = diff_buf_idx((*towin).w_buffer, curtab.get());
        if toidx == DB_COUNT {
            return;
        }
        (*towin).w_topline =
            lnum + ((*dp).df_lnum[toidx as usize] - (*dp).df_lnum[fromidx as usize]);
        if lnum >= (*dp).df_lnum[fromidx as usize] {
            calculate_topfill_and_topline(
                fromidx,
                toidx,
                (*fromwin).w_topline as ::core::ffi::c_int,
                (*fromwin).w_topfill,
                &raw mut (*towin).w_topfill,
                &raw mut (*towin).w_topline,
            );
        }
    }
    (*towin).w_botfill = false_0 != 0;
    if (*towin).w_topline > (*(*towin).w_buffer).b_ml.ml_line_count {
        (*towin).w_topline = (*(*towin).w_buffer).b_ml.ml_line_count;
        (*towin).w_botfill = true_0 != 0;
    }
    if (*towin).w_topline < 1 as linenr_T {
        (*towin).w_topline = 1 as ::core::ffi::c_int as linenr_T;
        (*towin).w_topfill = 0 as ::core::ffi::c_int;
    }
    invalidate_botline_win(towin);
    changed_line_abv_curs_win(towin);
    check_topfill(towin, false_0 != 0);
    hasFolding(
        towin,
        (*towin).w_topline,
        &raw mut (*towin).w_topline,
        ::core::ptr::null_mut::<linenr_T>(),
    );
}
unsafe extern "C" fn parse_diffanchors(
    mut check_only: bool,
    mut buf: *mut buf_T,
    mut anchors: *mut linenr_T,
    mut num_anchors: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut dia: *mut ::core::ffi::c_char = if *(*buf).b_p_dia as ::core::ffi::c_int == NUL {
        p_dia.get()
    } else {
        (*buf).b_p_dia
    };
    let mut orig_curbuf: *mut buf_T = curbuf.get();
    let mut orig_curwin: *mut win_T = curwin.get();
    let mut bufwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if check_only {
        bufwin = curwin.get();
    } else {
        bufwin = firstwin.get();
        while !bufwin.is_null() {
            if (*bufwin).w_buffer == buf && (*bufwin).w_onebuf_opt.wo_diff != 0 {
                break;
            }
            bufwin = (*bufwin).w_next;
        }
        if bufwin.is_null() && *dia as ::core::ffi::c_int != NUL {
            emsg(gettext(
                &raw const e_diff_anchors_with_hidden_windows as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
    }
    i = 0 as ::core::ffi::c_int;
    while i < MAX_DIFF_ANCHORS as ::core::ffi::c_int && *dia as ::core::ffi::c_int != NUL {
        if *dia as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            return FAIL;
        }
        curbuf.set(buf);
        curwin.set(bufwin);
        let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut lnum: linenr_T = get_address(
            ::core::ptr::null_mut::<exarg_T>(),
            &raw mut dia,
            ADDR_LINES,
            check_only,
            true_0 != 0,
            false_0,
            1 as ::core::ffi::c_int,
            &raw mut errormsg,
        );
        curbuf.set(orig_curbuf);
        curwin.set(orig_curwin);
        if !errormsg.is_null() {
            emsg(errormsg);
        }
        if dia.is_null() {
            return FAIL;
        }
        if *dia as ::core::ffi::c_int != ',' as ::core::ffi::c_int
            && *dia as ::core::ffi::c_int != NUL
        {
            return FAIL;
        }
        if !check_only
            && (lnum == MAXLNUM as ::core::ffi::c_int as linenr_T
                || lnum <= 0 as linenr_T
                || lnum > (*buf).b_ml.ml_line_count + 1 as linenr_T)
        {
            emsg(gettext(&raw const e_invrange as *const ::core::ffi::c_char));
            return FAIL;
        }
        if !anchors.is_null() {
            *anchors.offset(i as isize) = lnum;
        }
        if *dia as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            dia = dia.offset(1);
        }
        i += 1;
    }
    if i == MAX_DIFF_ANCHORS as ::core::ffi::c_int && *dia as ::core::ffi::c_int != NUL {
        semsg(
            gettext(
                &raw const e_cannot_have_more_than_nr_diff_anchors as *const ::core::ffi::c_char,
            ),
            MAX_DIFF_ANCHORS as ::core::ffi::c_int,
        );
        return FAIL;
    }
    if !num_anchors.is_null() {
        *num_anchors = i;
    }
    return OK;
}
pub unsafe extern "C" fn diffanchors_changed(mut buflocal: bool) -> ::core::ffi::c_int {
    let mut result: ::core::ffi::c_int = parse_diffanchors(
        true_0 != 0,
        curbuf.get(),
        ::core::ptr::null_mut::<linenr_T>(),
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    );
    if result == OK && diff_flags.get() & DIFF_ANCHOR != 0 {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            if !buflocal {
                (*tp).tp_diff_invalid = true_0;
            } else {
                let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while idx < DB_COUNT {
                    if (*tp).tp_diffbuf[idx as usize] == curbuf.get() {
                        (*tp).tp_diff_invalid = true_0;
                        break;
                    } else {
                        idx += 1;
                    }
                }
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
    }
    return result;
}
pub unsafe extern "C" fn diffopt_changed() -> ::core::ffi::c_int {
    let mut diff_context_new: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
    let mut linematch_lines_new: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut diff_flags_new: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut diff_foldcolumn_new: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    let mut diff_algorithm_new: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut diff_indent_heuristic: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = p_dip.get();
    while *p as ::core::ffi::c_int != NUL {
        if strncmp(
            p,
            b"filler\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(6 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_FILLER;
        } else if strncmp(
            p,
            b"anchor\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(6 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_ANCHOR;
        } else if strncmp(
            p,
            b"context:\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
            && ascii_isdigit(*p.offset(8 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            p = p.offset(8 as ::core::ffi::c_int as isize);
            diff_context_new = getdigits_int(&raw mut p, false_0 != 0, diff_context_new);
        } else if strncmp(
            p,
            b"iblank\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(6 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_IBLANK;
        } else if strncmp(
            p,
            b"icase\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(5 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_ICASE;
        } else if strncmp(
            p,
            b"iwhiteall\0".as_ptr() as *const ::core::ffi::c_char,
            9 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(9 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_IWHITEALL;
        } else if strncmp(
            p,
            b"iwhiteeol\0".as_ptr() as *const ::core::ffi::c_char,
            9 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(9 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_IWHITEEOL;
        } else if strncmp(
            p,
            b"iwhite\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(6 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_IWHITE;
        } else if strncmp(
            p,
            b"horizontal\0".as_ptr() as *const ::core::ffi::c_char,
            10 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(10 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_HORIZONTAL;
        } else if strncmp(
            p,
            b"vertical\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(8 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_VERTICAL;
        } else if strncmp(
            p,
            b"foldcolumn:\0".as_ptr() as *const ::core::ffi::c_char,
            11 as size_t,
        ) == 0 as ::core::ffi::c_int
            && ascii_isdigit(*p.offset(11 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            p = p.offset(11 as ::core::ffi::c_int as isize);
            diff_foldcolumn_new = getdigits_int(&raw mut p, false_0 != 0, diff_foldcolumn_new);
        } else if strncmp(
            p,
            b"hiddenoff\0".as_ptr() as *const ::core::ffi::c_char,
            9 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(9 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_HIDDEN_OFF;
        } else if strncmp(
            p,
            b"closeoff\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(8 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_CLOSE_OFF;
        } else if strncmp(
            p,
            b"followwrap\0".as_ptr() as *const ::core::ffi::c_char,
            10 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(10 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_FOLLOWWRAP;
        } else if strncmp(
            p,
            b"indent-heuristic\0".as_ptr() as *const ::core::ffi::c_char,
            16 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(16 as ::core::ffi::c_int as isize);
            diff_indent_heuristic = XDF_INDENT_HEURISTIC;
        } else if strncmp(
            p,
            b"internal\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(8 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_INTERNAL;
        } else if strncmp(
            p,
            b"algorithm:\0".as_ptr() as *const ::core::ffi::c_char,
            10 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(10 as ::core::ffi::c_int as isize);
            if strncmp(
                p,
                b"myers\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(5 as ::core::ffi::c_int as isize);
                diff_algorithm_new = 0 as ::core::ffi::c_int;
            } else if strncmp(
                p,
                b"minimal\0".as_ptr() as *const ::core::ffi::c_char,
                7 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(7 as ::core::ffi::c_int as isize);
                diff_algorithm_new = XDF_NEED_MINIMAL;
            } else if strncmp(
                p,
                b"patience\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(8 as ::core::ffi::c_int as isize);
                diff_algorithm_new = XDF_PATIENCE_DIFF;
            } else if strncmp(
                p,
                b"histogram\0".as_ptr() as *const ::core::ffi::c_char,
                9 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(9 as ::core::ffi::c_int as isize);
                diff_algorithm_new = XDF_HISTOGRAM_DIFF;
            } else {
                return FAIL;
            }
        } else if strncmp(
            p,
            b"inline:\0".as_ptr() as *const ::core::ffi::c_char,
            7 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(7 as ::core::ffi::c_int as isize);
            if strncmp(
                p,
                b"none\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(4 as ::core::ffi::c_int as isize);
                diff_flags_new &= !ALL_INLINE;
                diff_flags_new |= DIFF_INLINE_NONE;
            } else if strncmp(
                p,
                b"simple\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(6 as ::core::ffi::c_int as isize);
                diff_flags_new &= !ALL_INLINE;
                diff_flags_new |= DIFF_INLINE_SIMPLE;
            } else if strncmp(
                p,
                b"char\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(4 as ::core::ffi::c_int as isize);
                diff_flags_new &= !ALL_INLINE;
                diff_flags_new |= DIFF_INLINE_CHAR;
            } else if strncmp(
                p,
                b"word\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(4 as ::core::ffi::c_int as isize);
                diff_flags_new &= !ALL_INLINE;
                diff_flags_new |= DIFF_INLINE_WORD;
            } else {
                return FAIL;
            }
        } else if strncmp(
            p,
            b"linematch:\0".as_ptr() as *const ::core::ffi::c_char,
            10 as size_t,
        ) == 0 as ::core::ffi::c_int
            && ascii_isdigit(*p.offset(10 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            p = p.offset(10 as ::core::ffi::c_int as isize);
            linematch_lines_new = getdigits_int(&raw mut p, false_0 != 0, linematch_lines_new);
            diff_flags_new |= DIFF_LINEMATCH;
            diff_flags_new |= DIFF_FILLER;
        }
        if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int && *p as ::core::ffi::c_int != NUL
        {
            return FAIL;
        }
        if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            p = p.offset(1);
        }
    }
    diff_algorithm_new |= diff_indent_heuristic;
    if diff_flags_new & DIFF_HORIZONTAL != 0 && diff_flags_new & DIFF_VERTICAL != 0 {
        return FAIL;
    }
    if diff_flags.get() != diff_flags_new || diff_algorithm.get() != diff_algorithm_new {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            (*tp).tp_diff_invalid = true_0;
            tp = (*tp).tp_next as *mut tabpage_T;
        }
    }
    diff_flags.set(diff_flags_new);
    diff_context.set(if diff_context_new == 0 as ::core::ffi::c_int {
        1 as ::core::ffi::c_int
    } else {
        diff_context_new
    });
    linematch_lines.set(linematch_lines_new);
    diff_foldcolumn.set(diff_foldcolumn_new);
    diff_algorithm.set(diff_algorithm_new);
    diff_redraw(true_0 != 0);
    check_scrollbind(0 as linenr_T, 0 as ::core::ffi::c_int);
    return OK;
}
pub unsafe extern "C" fn diffopt_horizontal() -> bool {
    return diff_flags.get() & DIFF_HORIZONTAL != 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn diffopt_hiddenoff() -> bool {
    return diff_flags.get() & DIFF_HIDDEN_OFF != 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn diffopt_closeoff() -> bool {
    return diff_flags.get() & DIFF_CLOSE_OFF != 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn diffopt_filler() -> bool {
    return diff_flags.get() & DIFF_FILLER != 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn diff_update_line(mut lnum: linenr_T) {
    if diff_flags.get() & ALL_INLINE_DIFF == 0 {
        return;
    }
    let mut idx: ::core::ffi::c_int = diff_buf_idx(curbuf.get(), curtab.get());
    if idx == DB_COUNT {
        return;
    }
    let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    dp = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if lnum <= (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
            break;
        }
        dp = (*dp).df_next;
    }
    if !dp.is_null() {
        (*dp).has_changes = false_0 != 0;
        (*dp).df_changes.ga_len = 0 as ::core::ffi::c_int;
    }
}
static simple_diffline_change: GlobalCell<diffline_change_T> = GlobalCell::new(diffline_change_T {
    dc_start: [0; 8],
    dc_end: [0; 8],
    dc_start_lnum_off: [0; 8],
    dc_end_lnum_off: [0; 8],
});
pub unsafe extern "C" fn diff_change_parse(
    mut diffline: *mut diffline_T,
    mut change: *mut diffline_change_T,
    mut change_start: *mut ::core::ffi::c_int,
    mut change_end: *mut ::core::ffi::c_int,
) -> bool {
    if (*change).dc_start_lnum_off[(*diffline).bufidx as usize] < (*diffline).lineoff {
        *change_start = 0 as ::core::ffi::c_int;
    } else {
        *change_start = (*change).dc_start[(*diffline).bufidx as usize] as ::core::ffi::c_int;
    }
    if (*change).dc_end_lnum_off[(*diffline).bufidx as usize] > (*diffline).lineoff {
        *change_end = INT_MAX;
    } else {
        *change_end = (*change).dc_end[(*diffline).bufidx as usize] as ::core::ffi::c_int;
    }
    if change == simple_diffline_change.ptr() {
        return false_0 != 0;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if i != (*diffline).bufidx {
            if (*change).dc_start[i as usize] != (*change).dc_end[i as usize]
                || (*change).dc_end_lnum_off[i as usize] != (*change).dc_start_lnum_off[i as usize]
            {
                return false_0 != 0;
            }
        }
        i += 1;
    }
    return true_0 != 0;
}
unsafe extern "C" fn diff_find_change_simple(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut dp: *const diff_T,
    mut idx: ::core::ffi::c_int,
    mut startp: *mut ::core::ffi::c_int,
    mut endp: *mut ::core::ffi::c_int,
) -> bool {
    let mut line_org: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if diff_flags.get() & DIFF_INLINE_NONE != 0 {
        line_org = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        line_org = xstrdup(ml_get_buf((*wp).w_buffer, lnum));
    }
    let mut si_org: ::core::ffi::c_int = 0;
    let mut si_new: ::core::ffi::c_int = 0;
    let mut ei_org: ::core::ffi::c_int = 0;
    let mut ei_new: ::core::ffi::c_int = 0;
    let mut added: bool = true_0 != 0;
    let mut off: linenr_T = lnum - (*dp).df_lnum[idx as usize];
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if !(*curtab.get()).tp_diffbuf[i as usize].is_null() && i != idx {
            if off < (*dp).df_count[i as usize] {
                added = false_0 != 0;
                if diff_flags.get() & DIFF_INLINE_NONE != 0 {
                    break;
                }
                let mut line_new: *mut ::core::ffi::c_char = ml_get_buf(
                    (*curtab.get()).tp_diffbuf[i as usize] as *mut buf_T,
                    (*dp).df_lnum[i as usize] + off,
                );
                si_new = 0 as ::core::ffi::c_int;
                si_org = si_new;
                while *line_org.offset(si_org as isize) as ::core::ffi::c_int != NUL {
                    if diff_flags.get() & DIFF_IWHITE != 0
                        && ascii_iswhite(*line_org.offset(si_org as isize) as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                            != 0
                        && ascii_iswhite(*line_new.offset(si_new as isize) as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                            != 0
                        || diff_flags.get() & DIFF_IWHITEALL != 0
                            && (ascii_iswhite(
                                *line_org.offset(si_org as isize) as ::core::ffi::c_int
                            ) as ::core::ffi::c_int
                                != 0
                                || ascii_iswhite(
                                    *line_new.offset(si_new as isize) as ::core::ffi::c_int
                                ) as ::core::ffi::c_int
                                    != 0)
                    {
                        si_org = skipwhite(line_org.offset(si_org as isize)).offset_from(line_org)
                            as ::core::ffi::c_int;
                        si_new = skipwhite(line_new.offset(si_new as isize)).offset_from(line_new)
                            as ::core::ffi::c_int;
                    } else {
                        let mut l: ::core::ffi::c_int = 0;
                        if !diff_equal_char(
                            line_org.offset(si_org as isize),
                            line_new.offset(si_new as isize),
                            &raw mut l,
                        ) {
                            break;
                        }
                        si_org += l;
                        si_new += l;
                    }
                }
                si_org -= utf_head_off(line_org, line_org.offset(si_org as isize));
                si_new -= utf_head_off(line_new, line_new.offset(si_new as isize));
                *startp = if *startp < si_org { *startp } else { si_org };
                if *line_org.offset(si_org as isize) as ::core::ffi::c_int != NUL
                    || *line_new.offset(si_new as isize) as ::core::ffi::c_int != NUL
                {
                    ei_org = strlen(line_org) as ::core::ffi::c_int;
                    ei_new = strlen(line_new) as ::core::ffi::c_int;
                    while ei_org >= *startp
                        && ei_new >= si_new
                        && ei_org >= 0 as ::core::ffi::c_int
                        && ei_new >= 0 as ::core::ffi::c_int
                    {
                        if diff_flags.get() & DIFF_IWHITE != 0
                            && ascii_iswhite(*line_org.offset(ei_org as isize) as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                                != 0
                            && ascii_iswhite(*line_new.offset(ei_new as isize) as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                                != 0
                            || diff_flags.get() & DIFF_IWHITEALL != 0
                                && (ascii_iswhite(
                                    *line_org.offset(ei_org as isize) as ::core::ffi::c_int
                                ) as ::core::ffi::c_int
                                    != 0
                                    || ascii_iswhite(
                                        *line_new.offset(ei_new as isize) as ::core::ffi::c_int
                                    ) as ::core::ffi::c_int
                                        != 0)
                        {
                            while ei_org >= *startp
                                && ascii_iswhite(
                                    *line_org.offset(ei_org as isize) as ::core::ffi::c_int
                                ) as ::core::ffi::c_int
                                    != 0
                            {
                                ei_org -= 1;
                            }
                            while ei_new >= si_new
                                && ascii_iswhite(
                                    *line_new.offset(ei_new as isize) as ::core::ffi::c_int
                                ) as ::core::ffi::c_int
                                    != 0
                            {
                                ei_new -= 1;
                            }
                        } else {
                            let mut p1: *const ::core::ffi::c_char =
                                line_org.offset(ei_org as isize);
                            let mut p2: *const ::core::ffi::c_char =
                                line_new.offset(ei_new as isize);
                            p1 = p1.offset(-(utf_head_off(line_org, p1) as isize));
                            p2 = p2.offset(-(utf_head_off(line_new, p2) as isize));
                            let mut l_0: ::core::ffi::c_int = 0;
                            if !diff_equal_char(p1, p2, &raw mut l_0) {
                                break;
                            }
                            ei_org -= l_0;
                            ei_new -= l_0;
                        }
                    }
                    *endp = if *endp > ei_org { *endp } else { ei_org };
                }
            }
        }
        i += 1;
    }
    xfree(line_org as *mut ::core::ffi::c_void);
    return added;
}
unsafe extern "C" fn diff_refine_inline_char_highlight(
    mut dp_orig: *mut diff_T,
    mut linemap: *mut garray_T,
    mut idx1: ::core::ffi::c_int,
) {
    let mut pass: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    loop {
        let mut has_unmerged_gaps: bool = false_0 != 0;
        let mut has_merged_gaps: bool = false_0 != 0;
        let mut dp: *mut diff_T = dp_orig;
        while !dp.is_null() && !(*dp).df_next.is_null() {
            if (*dp).df_lnum[idx1 as usize] + (*dp).df_count[idx1 as usize] - 1 as linenr_T
                >= (*linemap.offset(idx1 as isize)).ga_len as linenr_T
                || (*(*dp).df_next).df_lnum[idx1 as usize] - 1 as linenr_T
                    >= (*linemap.offset(idx1 as isize)).ga_len as linenr_T
            {
                dp = (*dp).df_next;
            } else {
                let mut entry1: *mut linemap_entry_T =
                    ((*linemap.offset(idx1 as isize)).ga_data as *mut linemap_entry_T).offset(
                        (*(&raw mut (*dp).df_lnum as *mut linenr_T).offset(idx1 as isize)
                            + *(&raw mut (*dp).df_count as *mut linenr_T).offset(idx1 as isize)
                            - 1 as linenr_T) as isize,
                    );
                let mut entry2: *mut linemap_entry_T =
                    ((*linemap.offset(idx1 as isize)).ga_data as *mut linemap_entry_T).offset(
                        (*(&raw mut (*(*dp).df_next).df_lnum as *mut linenr_T)
                            .offset(idx1 as isize)
                            - 1 as linenr_T) as isize,
                    );
                if (*entry1).lineoff != (*entry2).lineoff {
                    dp = (*dp).df_next;
                } else {
                    let mut gap: linenr_T = (*(*dp).df_next).df_lnum[idx1 as usize]
                        - ((*dp).df_lnum[idx1 as usize] + (*dp).df_count[idx1 as usize]);
                    if gap <= 3 as linenr_T {
                        let mut max_df_count: linenr_T = 0 as linenr_T;
                        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while i < DB_COUNT {
                            max_df_count = if max_df_count
                                > (*dp).df_count[i as usize] + (*(*dp).df_next).df_count[i as usize]
                            {
                                max_df_count
                            } else {
                                (*dp).df_count[i as usize] + (*(*dp).df_next).df_count[i as usize]
                            };
                            i += 1;
                        }
                        if max_df_count >= gap * 4 as linenr_T {
                            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while i_0 < DB_COUNT {
                                (*dp).df_count[i_0 as usize] = (*(*dp).df_next).df_lnum
                                    [i_0 as usize]
                                    + (*(*dp).df_next).df_count[i_0 as usize]
                                    - (*dp).df_lnum[i_0 as usize];
                                i_0 += 1;
                            }
                            let mut dp_next: *mut diff_T = (*dp).df_next;
                            (*dp).df_next = (*dp_next).df_next;
                            clear_diffblock(dp_next);
                            has_merged_gaps = true_0 != 0;
                            continue;
                        } else {
                            has_unmerged_gaps = true_0 != 0;
                        }
                    }
                    dp = (*dp).df_next;
                }
            }
        }
        if !has_unmerged_gaps || !has_merged_gaps {
            break;
        }
        let c2rust_fresh9 = pass;
        pass = pass + 1;
        if c2rust_fresh9 >= 4 as ::core::ffi::c_int {
            break;
        }
    }
}
unsafe extern "C" fn diff_refine_inline_word_highlight(
    mut dp_orig: *mut diff_T,
    mut linemap: *mut garray_T,
    mut idx1: ::core::ffi::c_int,
    mut start_lnum: linenr_T,
) {
    let mut pass: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    loop {
        let mut dp: *mut diff_T = dp_orig;
        while !dp.is_null() && !(*dp).df_next.is_null() {
            if (*dp).df_lnum[idx1 as usize] + (*dp).df_count[idx1 as usize] - 1 as linenr_T
                >= (*linemap.offset(idx1 as isize)).ga_len as linenr_T
                || (*(*dp).df_next).df_lnum[idx1 as usize] - 1 as linenr_T
                    >= (*linemap.offset(idx1 as isize)).ga_len as linenr_T
            {
                dp = (*dp).df_next;
            } else {
                let mut entry1: *mut linemap_entry_T =
                    ((*linemap.offset(idx1 as isize)).ga_data as *mut linemap_entry_T).offset(
                        (*(&raw mut (*dp).df_lnum as *mut linenr_T).offset(idx1 as isize)
                            + *(&raw mut (*dp).df_count as *mut linenr_T).offset(idx1 as isize)
                            - 2 as linenr_T) as isize,
                    );
                let mut entry2: *mut linemap_entry_T =
                    ((*linemap.offset(idx1 as isize)).ga_data as *mut linemap_entry_T).offset(
                        (*(&raw mut (*(*dp).df_next).df_lnum as *mut linenr_T)
                            .offset(idx1 as isize)
                            - 1 as linenr_T) as isize,
                    );
                if (*entry1).lineoff != (*entry2).lineoff {
                    dp = (*dp).df_next;
                } else {
                    let mut gap_start: ::core::ffi::c_int = (*entry1).byte_start
                        as ::core::ffi::c_int
                        + (*entry1).num_bytes as ::core::ffi::c_int;
                    let mut gap_end: ::core::ffi::c_int =
                        (*entry2).byte_start as ::core::ffi::c_int;
                    let mut gap_size: ::core::ffi::c_int = gap_end - gap_start;
                    if gap_size <= 0 as ::core::ffi::c_int || gap_size > diff_word_gap.get() {
                        dp = (*dp).df_next;
                    } else {
                        let mut line: *mut ::core::ffi::c_char = ml_get_buf(
                            (*curtab.get()).tp_diffbuf[idx1 as usize] as *mut buf_T,
                            start_lnum + (*entry1).lineoff as linenr_T,
                        );
                        let mut gap_text: *mut ::core::ffi::c_char =
                            line.offset(gap_start as isize);
                        let mut only_non_word: bool = true_0 != 0;
                        let mut has_content: bool = false_0 != 0;
                        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while i < gap_size
                            && *gap_text.offset(i as isize) as ::core::ffi::c_int != NUL
                        {
                            has_content = true_0 != 0;
                            let mut char_class: ::core::ffi::c_int = mb_get_class_tab(
                                gap_text.offset(i as isize),
                                &raw mut (**(&raw mut (*curtab.get()).tp_diffbuf
                                    as *mut *mut buf_T)
                                    .offset(idx1 as isize))
                                .b_chartab as *mut uint64_t,
                            );
                            if char_class == 2 as ::core::ffi::c_int {
                                only_non_word = false_0 != 0;
                                break;
                            } else {
                                i += 1;
                            }
                        }
                        if has_content as ::core::ffi::c_int != 0
                            && only_non_word as ::core::ffi::c_int != 0
                        {
                            let mut total_change_bytes: ::core::ffi::c_long =
                                0 as ::core::ffi::c_long;
                            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while i_0 < DB_COUNT {
                                if !(*curtab.get()).tp_diffbuf[i_0 as usize].is_null() {
                                    let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    while (k as linenr_T) < (*dp).df_count[i_0 as usize] {
                                        let mut idx: ::core::ffi::c_int =
                                            (*dp).df_lnum[i_0 as usize] as ::core::ffi::c_int + k
                                                - 1 as ::core::ffi::c_int;
                                        if idx < (*linemap.offset(i_0 as isize)).ga_len {
                                            total_change_bytes +=
                                                (*((*linemap.offset(i_0 as isize)).ga_data
                                                    as *mut linemap_entry_T)
                                                    .offset(idx as isize))
                                                .num_bytes
                                                    as ::core::ffi::c_long;
                                        }
                                        k += 1;
                                    }
                                    let mut k_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    while (k_0 as linenr_T)
                                        < (*(*dp).df_next).df_count[i_0 as usize]
                                    {
                                        let mut idx_0: ::core::ffi::c_int = (*(*dp).df_next).df_lnum
                                            [i_0 as usize]
                                            as ::core::ffi::c_int
                                            + k_0
                                            - 1 as ::core::ffi::c_int;
                                        if idx_0 < (*linemap.offset(i_0 as isize)).ga_len {
                                            total_change_bytes +=
                                                (*((*linemap.offset(i_0 as isize)).ga_data
                                                    as *mut linemap_entry_T)
                                                    .offset(idx_0 as isize))
                                                .num_bytes
                                                    as ::core::ffi::c_long;
                                        }
                                        k_0 += 1;
                                    }
                                }
                                i_0 += 1;
                            }
                            if total_change_bytes
                                >= (gap_size * 2 as ::core::ffi::c_int) as ::core::ffi::c_long
                            {
                                let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                while i_1 < DB_COUNT {
                                    if !(*curtab.get()).tp_diffbuf[i_1 as usize].is_null() {
                                        (*dp).df_count[i_1 as usize] = (*(*dp).df_next).df_lnum
                                            [i_1 as usize]
                                            + (*(*dp).df_next).df_count[i_1 as usize]
                                            - (*dp).df_lnum[i_1 as usize];
                                    }
                                    i_1 += 1;
                                }
                                let mut dp_next: *mut diff_T = (*dp).df_next;
                                (*dp).df_next = (*dp_next).df_next;
                                clear_diffblock(dp_next);
                                continue;
                            }
                        }
                        dp = (*dp).df_next;
                    }
                }
            }
        }
        let c2rust_fresh10 = pass;
        pass = pass + 1;
        if c2rust_fresh10 >= 4 as ::core::ffi::c_int {
            break;
        }
    }
}
unsafe extern "C" fn diff_find_change_inline_diff(mut dp: *mut diff_T) {
    let mut new_diff: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let save_diff_algorithm: ::core::ffi::c_int = diff_algorithm.get();
    let mut dio: diffio_T = diffio_T {
        dio_orig: diffin_T {
            din_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            din_mmfile: mmfile_t {
                ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        },
        dio_new: diffin_T {
            din_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            din_mmfile: mmfile_t {
                ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        },
        dio_diff: diffout_T {
            dout_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            dout_ga: garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
        },
        dio_internal: 0,
    };
    ga_init(
        &raw mut dio.dio_diff.dout_ga,
        ::core::mem::size_of::<diffhunk_T>() as ::core::ffi::c_int,
        1000 as ::core::ffi::c_int,
    );
    dio.dio_internal = true_0;
    (*diff_algorithm.ptr()) |= XDF_INDENT_HEURISTIC;
    let mut orig_diff: *mut diff_T = (*curtab.get()).tp_first_diff;
    (*curtab.get()).tp_first_diff = ::core::ptr::null_mut::<diff_T>();
    let mut orig_diffbuf: [*mut buf_T; 8] = [::core::ptr::null_mut::<buf_T>(); 8];
    memcpy(
        &raw mut orig_diffbuf as *mut *mut buf_T as *mut ::core::ffi::c_void,
        &raw mut (*curtab.get()).tp_diffbuf as *mut *mut buf_T as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[*mut buf_T; 8]>(),
    );
    let mut linemap: [garray_T; 8] = [garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    }; 8];
    let mut file1_str: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut file2_str: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut file1_str,
        1 as ::core::ffi::c_int,
        1024 as ::core::ffi::c_int,
    );
    ga_init(
        &raw mut file2_str,
        1 as ::core::ffi::c_int,
        1024 as ::core::ffi::c_int,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        ga_init(
            (&raw mut linemap as *mut garray_T).offset(i as isize),
            ::core::mem::size_of::<linemap_entry_T>() as ::core::ffi::c_int,
            128 as ::core::ffi::c_int,
        );
        i += 1;
    }
    let mut file1_idx: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    '_done: {
        while i_0 < DB_COUNT {
            dio.dio_diff.dout_ga.ga_len = 0 as ::core::ffi::c_int;
            let mut buf: *mut buf_T = (*curtab.get()).tp_diffbuf[i_0 as usize] as *mut buf_T;
            if !(buf.is_null() || (*buf).b_ml.ml_mfp.is_null()) {
                if (*dp).df_count[i_0 as usize] == 0 as linenr_T {
                    (*curtab.get()).tp_diffbuf[i_0 as usize] = ::core::ptr::null_mut::<buf_T>();
                } else {
                    if file1_idx == -1 as ::core::ffi::c_int {
                        file1_idx = i_0;
                    }
                    let mut curstr: *mut garray_T = if file1_idx != i_0 {
                        &raw mut file2_str
                    } else {
                        &raw mut file1_str
                    };
                    let mut numlines: linenr_T = 0 as linenr_T;
                    (*curstr).ga_len = 0 as ::core::ffi::c_int;
                    let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while (off as linenr_T) < (*dp).df_count[i_0 as usize] {
                        let mut curline: *mut ::core::ffi::c_char = ml_get_buf(
                            (*curtab.get()).tp_diffbuf[i_0 as usize] as *mut buf_T,
                            (*dp).df_lnum[i_0 as usize] + off as linenr_T,
                        );
                        let mut in_keyword: bool = false_0 != 0;
                        let mut last_white: bool = false_0 != 0;
                        let mut eol_ga_len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                        let mut eol_linemap_len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                        let mut eol_numlines: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                        let mut s: *mut ::core::ffi::c_char = curline;
                        while *s as ::core::ffi::c_int != NUL {
                            let mut new_in_keyword: bool = false_0 != 0;
                            if diff_flags.get() & DIFF_INLINE_WORD != 0 {
                                new_in_keyword = mb_get_class_tab(
                                    s,
                                    &raw mut (**(&raw mut (*curtab.get()).tp_diffbuf
                                        as *mut *mut buf_T)
                                        .offset(file1_idx as isize))
                                    .b_chartab as *mut uint64_t,
                                ) == 2 as ::core::ffi::c_int;
                            }
                            if in_keyword as ::core::ffi::c_int != 0 && !new_in_keyword {
                                ga_append(curstr, NL as uint8_t);
                                numlines += 1;
                            }
                            if ascii_iswhite(*s as ::core::ffi::c_int) {
                                if diff_flags.get() & DIFF_IWHITEALL != 0 {
                                    in_keyword = false_0 != 0;
                                    s = skipwhite(s);
                                    continue;
                                } else if diff_flags.get() & DIFF_IWHITEEOL != 0
                                    || diff_flags.get() & DIFF_IWHITE != 0
                                {
                                    if !last_white {
                                        eol_ga_len = (*curstr).ga_len;
                                        eol_linemap_len = linemap[i_0 as usize].ga_len;
                                        eol_numlines = numlines as ::core::ffi::c_int;
                                        last_white = true_0 != 0;
                                    }
                                }
                            } else if diff_flags.get() & DIFF_IWHITEEOL != 0
                                || diff_flags.get() & DIFF_IWHITE != 0
                            {
                                last_white = false_0 != 0;
                                eol_ga_len = -1 as ::core::ffi::c_int;
                                eol_linemap_len = -1 as ::core::ffi::c_int;
                                eol_numlines = -1 as ::core::ffi::c_int;
                            }
                            let mut char_len: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                            if *s as ::core::ffi::c_int == NL {
                                ga_append(curstr, NUL as uint8_t);
                            } else {
                                char_len = utfc_ptr2len(s);
                                if ascii_iswhite(*s as ::core::ffi::c_int) as ::core::ffi::c_int
                                    != 0
                                    && diff_flags.get() & DIFF_IWHITE != 0
                                {
                                    char_len = skipwhite(s).offset_from(s) as ::core::ffi::c_int;
                                }
                                if diff_flags.get() & DIFF_ICASE != 0 {
                                    let mut c: ::core::ffi::c_int = utf_ptr2char(s);
                                    let mut c_len: ::core::ffi::c_int = utf_char2len(c);
                                    c = utf_fold(c);
                                    let mut cbuf: [::core::ffi::c_char; 22] = [0; 22];
                                    let mut c_fold_len: ::core::ffi::c_int = utf_char2bytes(
                                        c,
                                        &raw mut cbuf as *mut ::core::ffi::c_char,
                                    );
                                    ga_concat_len(
                                        curstr,
                                        &raw mut cbuf as *mut ::core::ffi::c_char,
                                        c_fold_len as size_t,
                                    );
                                    if char_len > c_len {
                                        ga_concat_len(
                                            curstr,
                                            s.offset(c_len as isize),
                                            (char_len - c_len) as size_t,
                                        );
                                    }
                                } else {
                                    ga_concat_len(curstr, s, char_len as size_t);
                                }
                            }
                            if !new_in_keyword {
                                ga_append(curstr, NL as uint8_t);
                                numlines += 1;
                            }
                            if !new_in_keyword
                                || new_in_keyword as ::core::ffi::c_int != 0 && !in_keyword
                            {
                                let mut linemap_entry: linemap_entry_T = linemap_entry_T {
                                    byte_start: s.offset_from(curline) as colnr_T,
                                    num_bytes: char_len as colnr_T,
                                    lineoff: off,
                                };
                                ga_grow(
                                    (&raw mut linemap as *mut garray_T).offset(i_0 as isize),
                                    1 as ::core::ffi::c_int,
                                );
                                *(linemap[i_0 as usize].ga_data as *mut linemap_entry_T)
                                    .offset(linemap[i_0 as usize].ga_len as isize) = linemap_entry;
                                linemap[i_0 as usize].ga_len += 1;
                            } else {
                                (*(linemap[i_0 as usize].ga_data as *mut linemap_entry_T)
                                    .offset(
                                        (linemap[i_0 as usize].ga_len - 1 as ::core::ffi::c_int)
                                            as isize,
                                    ))
                                .num_bytes += char_len;
                            }
                            in_keyword = new_in_keyword;
                            s = s.offset(char_len as isize);
                        }
                        if in_keyword {
                            ga_append(curstr, NL as uint8_t);
                            numlines += 1;
                        }
                        if diff_flags.get() & DIFF_IWHITEEOL != 0
                            || diff_flags.get() & DIFF_IWHITE != 0
                        {
                            if eol_ga_len != -1 as ::core::ffi::c_int {
                                (*curstr).ga_len = eol_ga_len;
                                linemap[i_0 as usize].ga_len = eol_linemap_len;
                                numlines = eol_numlines as linenr_T;
                            }
                        }
                        if diff_flags.get() & DIFF_IWHITEALL == 0 {
                            ga_append(curstr, NL as uint8_t);
                            numlines += 1;
                            let mut linemap_entry_0: linemap_entry_T = linemap_entry_T {
                                byte_start: s.offset_from(curline) as colnr_T,
                                num_bytes: ::core::mem::size_of::<::core::ffi::c_int>() as colnr_T,
                                lineoff: off,
                            };
                            ga_grow(
                                (&raw mut linemap as *mut garray_T).offset(i_0 as isize),
                                1 as ::core::ffi::c_int,
                            );
                            *(linemap[i_0 as usize].ga_data as *mut linemap_entry_T)
                                .offset(linemap[i_0 as usize].ga_len as isize) = linemap_entry_0;
                            linemap[i_0 as usize].ga_len += 1;
                        }
                        off += 1;
                    }
                    if file1_idx != i_0 {
                        dio.dio_new.din_mmfile.ptr = (*curstr).ga_data as *mut ::core::ffi::c_char;
                        dio.dio_new.din_mmfile.size = (*curstr).ga_len;
                    } else {
                        dio.dio_orig.din_mmfile.ptr = (*curstr).ga_data as *mut ::core::ffi::c_char;
                        dio.dio_orig.din_mmfile.size = (*curstr).ga_len;
                    }
                    if file1_idx != i_0 {
                        let mut diff_status: ::core::ffi::c_int = diff_file_internal(&raw mut dio);
                        if diff_status == FAIL {
                            break '_done;
                        }
                        diff_read(0 as ::core::ffi::c_int, i_0, &raw mut dio);
                        clear_diffout(&raw mut dio.dio_diff);
                    }
                }
            }
            i_0 += 1;
        }
        new_diff = (*curtab.get()).tp_first_diff;
        if diff_flags.get() & DIFF_INLINE_WORD != 0 && file1_idx != -1 as ::core::ffi::c_int {
            diff_refine_inline_word_highlight(
                new_diff,
                &raw mut linemap as *mut garray_T,
                file1_idx,
                (*dp).df_lnum[file1_idx as usize],
            );
        } else if diff_flags.get() & DIFF_INLINE_CHAR != 0 && file1_idx != -1 as ::core::ffi::c_int
        {
            diff_refine_inline_char_highlight(
                new_diff,
                &raw mut linemap as *mut garray_T,
                file1_idx,
            );
        }
        (*dp).df_changes.ga_len = 0 as ::core::ffi::c_int;
        while !new_diff.is_null() {
            let mut change: diffline_change_T = diffline_change_S {
                dc_start: [0 as colnr_T; 8],
                dc_end: [0; 8],
                dc_start_lnum_off: [0; 8],
                dc_end_lnum_off: [0; 8],
            };
            let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_1 < DB_COUNT {
                if (*new_diff).df_lnum[i_1 as usize] > 0 as linenr_T {
                    let mut diff_lnum: linenr_T = (*new_diff).df_lnum[i_1 as usize] - 1 as linenr_T;
                    let mut diff_lnum_end: linenr_T =
                        diff_lnum + (*new_diff).df_count[i_1 as usize];
                    if diff_lnum >= linemap[i_1 as usize].ga_len as linenr_T {
                        change.dc_start[i_1 as usize] = MAXCOL as ::core::ffi::c_int as colnr_T;
                        change.dc_start_lnum_off[i_1 as usize] = INT_MAX;
                    } else {
                        change.dc_start[i_1 as usize] = (*(linemap[i_1 as usize].ga_data
                            as *mut linemap_entry_T)
                            .offset(diff_lnum as isize))
                        .byte_start;
                        change.dc_start_lnum_off[i_1 as usize] = (*(linemap[i_1 as usize].ga_data
                            as *mut linemap_entry_T)
                            .offset(diff_lnum as isize))
                        .lineoff;
                    }
                    if diff_lnum == diff_lnum_end {
                        change.dc_end[i_1 as usize] = change.dc_start[i_1 as usize];
                        change.dc_end_lnum_off[i_1 as usize] =
                            change.dc_start_lnum_off[i_1 as usize];
                    } else if diff_lnum_end - 1 as linenr_T
                        >= linemap[i_1 as usize].ga_len as linenr_T
                    {
                        change.dc_end[i_1 as usize] = MAXCOL as ::core::ffi::c_int as colnr_T;
                        change.dc_end_lnum_off[i_1 as usize] = INT_MAX;
                    } else {
                        change.dc_end[i_1 as usize] = (*(linemap[i_1 as usize].ga_data
                            as *mut linemap_entry_T)
                            .offset((diff_lnum_end - 1 as linenr_T) as isize))
                        .byte_start
                            + (*(linemap[i_1 as usize].ga_data as *mut linemap_entry_T)
                                .offset((diff_lnum_end - 1 as linenr_T) as isize))
                            .num_bytes;
                        change.dc_end_lnum_off[i_1 as usize] = (*(linemap[i_1 as usize].ga_data
                            as *mut linemap_entry_T)
                            .offset((diff_lnum_end - 1 as linenr_T) as isize))
                        .lineoff;
                    }
                }
                i_1 += 1;
            }
            ga_grow(&raw mut (*dp).df_changes, 1 as ::core::ffi::c_int);
            *((*dp).df_changes.ga_data as *mut diffline_change_T)
                .offset((*dp).df_changes.ga_len as isize) = change;
            (*dp).df_changes.ga_len += 1;
            new_diff = (*new_diff).df_next;
        }
    }
    diff_algorithm.set(save_diff_algorithm);
    (*dp).has_changes = true_0 != 0;
    diff_clear(curtab.get());
    (*curtab.get()).tp_first_diff = orig_diff;
    memcpy(
        &raw mut (*curtab.get()).tp_diffbuf as *mut *mut buf_T as *mut ::core::ffi::c_void,
        &raw mut orig_diffbuf as *mut *mut buf_T as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[*mut buf_T; 8]>(),
    );
    ga_clear(&raw mut file1_str);
    ga_clear(&raw mut file2_str);
    clear_diffout(&raw mut dio.dio_diff);
    let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_2 < DB_COUNT {
        ga_clear((&raw mut linemap as *mut garray_T).offset(i_2 as isize));
        i_2 += 1;
    }
}
pub unsafe extern "C" fn diff_find_change(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut diffline: *mut diffline_T,
) -> bool {
    let mut idx: ::core::ffi::c_int = diff_buf_idx((*wp).w_buffer, curtab.get());
    if idx == DB_COUNT {
        return false_0 != 0;
    }
    let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    dp = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if lnum < (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
            break;
        }
        dp = (*dp).df_next;
    }
    if dp.is_null() || diff_check_sanity(curtab.get(), dp) == FAIL {
        return false_0 != 0;
    }
    let mut off: ::core::ffi::c_int =
        lnum as ::core::ffi::c_int - (*dp).df_lnum[idx as usize] as ::core::ffi::c_int;
    if diff_flags.get() & ALL_INLINE_DIFF == 0 {
        let mut change_start: ::core::ffi::c_int = MAXCOL as ::core::ffi::c_int;
        let mut change_end: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut ret: ::core::ffi::c_int = diff_find_change_simple(
            wp,
            lnum,
            dp,
            idx,
            &raw mut change_start,
            &raw mut change_end,
        ) as ::core::ffi::c_int;
        change_end += 1 as ::core::ffi::c_int;
        memset(
            simple_diffline_change.ptr() as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<diffline_change_T>(),
        );
        (*diffline).changes = simple_diffline_change.ptr();
        (*diffline).num_changes = 1 as ::core::ffi::c_int;
        (*diffline).bufidx = idx;
        (*diffline).lineoff = (lnum - (*dp).df_lnum[idx as usize]) as ::core::ffi::c_int;
        (*simple_diffline_change.ptr()).dc_start[idx as usize] = change_start as colnr_T;
        (*simple_diffline_change.ptr()).dc_end[idx as usize] = change_end as colnr_T;
        (*simple_diffline_change.ptr()).dc_start_lnum_off[idx as usize] = off;
        (*simple_diffline_change.ptr()).dc_end_lnum_off[idx as usize] = off;
        return ret != 0;
    }
    if !(*dp).has_changes {
        diff_find_change_inline_diff(dp);
    }
    let mut changes: *mut garray_T = &raw mut (*dp).df_changes;
    let mut num_changes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut change_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    (*diffline).changes = ::core::ptr::null_mut::<diffline_change_T>();
    change_idx = 0 as ::core::ffi::c_int;
    while change_idx < (*changes).ga_len {
        let mut change: *mut diffline_change_T =
            ((*dp).df_changes.ga_data as *mut diffline_change_T).offset(change_idx as isize);
        if (*change).dc_end_lnum_off[idx as usize] >= off {
            if (*change).dc_start_lnum_off[idx as usize] > off {
                break;
            }
            if (*diffline).changes.is_null() {
                (*diffline).changes = change;
            }
            num_changes += 1;
        }
        change_idx += 1;
    }
    (*diffline).num_changes = num_changes;
    (*diffline).bufidx = idx;
    (*diffline).lineoff = off;
    let mut added: bool = false_0 != 0;
    if num_changes == 1 as ::core::ffi::c_int && change_idx == (*dp).df_changes.ga_len {
        added = true_0 != 0;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < DB_COUNT {
            if idx != i {
                if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
                    let mut change_0: *mut diffline_change_T = ((*dp).df_changes.ga_data
                        as *mut diffline_change_T)
                        .offset(((*dp).df_changes.ga_len - 1 as ::core::ffi::c_int) as isize);
                    if (*change_0).dc_start_lnum_off[i as usize] != INT_MAX {
                        added = false_0 != 0;
                        break;
                    }
                }
            }
            i += 1;
        }
    }
    return added;
}
pub unsafe extern "C" fn diff_infold(mut wp: *mut win_T, mut lnum: linenr_T) -> bool {
    if (*wp).w_onebuf_opt.wo_diff == 0 {
        return false_0 != 0;
    }
    let mut idx: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut other: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if (*curtab.get()).tp_diffbuf[i as usize] == (*wp).w_buffer {
            idx = i;
        } else if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
            other = true_0 != 0;
        }
        i += 1;
    }
    if idx == -1 as ::core::ffi::c_int || !other {
        return false_0 != 0;
    }
    if (*curtab.get()).tp_diff_invalid != 0 {
        ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
    }
    if (*curtab.get()).tp_first_diff.is_null() {
        return true_0 != 0;
    }
    let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if (*dp).df_lnum[idx as usize] - diff_context.get() as linenr_T > lnum {
            break;
        }
        if (*dp).df_lnum[idx as usize]
            + (*dp).df_count[idx as usize]
            + diff_context.get() as linenr_T
            > lnum
        {
            return false_0 != 0;
        }
        dp = (*dp).df_next;
    }
    return true_0 != 0;
}
pub unsafe fn nv_diffgetput(mut put: bool, mut count: size_t) {
    if bt_prompt(curbuf.get()) {
        vim_beep(kOptBoFlagOperator as ::core::ffi::c_int as ::core::ffi::c_uint);
        return;
    }
    let mut ea: exarg_T = exarg_T {
        arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdidx: CMD_append,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
        usefilter: 0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    let mut buf: [::core::ffi::c_char; 30] = [0; 30];
    if count == 0 as size_t {
        ea.arg = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else {
        vim_snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
            b"%zu\0".as_ptr() as *const ::core::ffi::c_char,
            count,
        );
        ea.arg = &raw mut buf as *mut ::core::ffi::c_char;
    }
    if put {
        ea.cmdidx = CMD_diffput;
    } else {
        ea.cmdidx = CMD_diffget;
    }
    ea.addr_count = 0 as ::core::ffi::c_int;
    ea.line1 = (*curwin.get()).w_cursor.lnum;
    ea.line2 = (*curwin.get()).w_cursor.lnum;
    ex_diffgetput(&raw mut ea);
}
unsafe extern "C" fn valid_diff(mut diff: *mut diff_T) -> bool {
    let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if dp == diff {
            return true_0 != 0;
        }
        dp = (*dp).df_next;
    }
    return false_0 != 0;
}
pub unsafe fn ex_diffgetput(mut eap: *mut exarg_T) {
    let mut idx_other: ::core::ffi::c_int = 0;
    let mut idx_cur: ::core::ffi::c_int = diff_buf_idx(curbuf.get(), curtab.get());
    if idx_cur == DB_COUNT {
        emsg(gettext(
            b"E99: Current buffer is not in diff mode\0".as_ptr() as *const ::core::ffi::c_char,
        ));
        return;
    }
    if *(*eap).arg as ::core::ffi::c_int == NUL {
        let mut found_not_ma: bool = false_0 != 0;
        idx_other = 0 as ::core::ffi::c_int;
        while idx_other < DB_COUNT {
            if (*curtab.get()).tp_diffbuf[idx_other as usize] != curbuf.get()
                && !(*curtab.get()).tp_diffbuf[idx_other as usize].is_null()
            {
                if (*eap).cmdidx as ::core::ffi::c_int != CMD_diffput as ::core::ffi::c_int
                    || (*(*curtab.get()).tp_diffbuf[idx_other as usize]).b_p_ma != 0
                {
                    break;
                }
                found_not_ma = true_0 != 0;
            }
            idx_other += 1;
        }
        if idx_other == DB_COUNT {
            if found_not_ma {
                emsg(gettext(
                    b"E793: No other buffer in diff mode is modifiable\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
            } else {
                emsg(gettext(
                    b"E100: No other buffer in diff mode\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
            return;
        }
        let mut i: ::core::ffi::c_int = idx_other + 1 as ::core::ffi::c_int;
        while i < DB_COUNT {
            if (*curtab.get()).tp_diffbuf[i as usize] != curbuf.get()
                && !(*curtab.get()).tp_diffbuf[i as usize].is_null()
                && ((*eap).cmdidx as ::core::ffi::c_int != CMD_diffput as ::core::ffi::c_int
                    || (*(*curtab.get()).tp_diffbuf[i as usize]).b_p_ma != 0)
            {
                emsg(gettext(
                    b"E101: More than two buffers in diff mode, don't know which one to use\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                ));
                return;
            }
            i += 1;
        }
    } else {
        let mut p: *mut ::core::ffi::c_char = (*eap).arg.offset(strlen((*eap).arg) as isize);
        while p > (*eap).arg
            && ascii_iswhite(*p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            p = p.offset(-1);
        }
        let mut i_0: ::core::ffi::c_int = 0;
        i_0 = 0 as ::core::ffi::c_int;
        while ascii_isdigit(*(*eap).arg.offset(i_0 as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
            && (*eap).arg.offset(i_0 as isize) < p
        {
            i_0 += 1;
        }
        if (*eap).arg.offset(i_0 as isize) == p {
            i_0 = atol((*eap).arg) as ::core::ffi::c_int;
        } else {
            i_0 = buflist_findpat((*eap).arg, p, false_0 != 0, true_0 != 0, false_0 != 0);
            if i_0 < 0 as ::core::ffi::c_int {
                return;
            }
        }
        let mut buf: *mut buf_T = buflist_findnr(i_0);
        if buf.is_null() {
            semsg(
                gettext(b"E102: Can't find buffer \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
                (*eap).arg,
            );
            return;
        }
        if buf == curbuf.get() {
            return;
        }
        idx_other = diff_buf_idx(buf, curtab.get());
        if idx_other == DB_COUNT {
            semsg(
                gettext(b"E103: Buffer \"%s\" is not in diff mode\0".as_ptr()
                    as *const ::core::ffi::c_char),
                (*eap).arg,
            );
            return;
        }
    }
    diff_busy.set(true_0 != 0);
    if (*eap).addr_count == 0 as ::core::ffi::c_int {
        let mut linestatus: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*eap).line1 == (*curbuf.get()).b_ml.ml_line_count
            && (diff_check_with_linestatus(curwin.get(), (*eap).line1, &raw mut linestatus)
                == 0 as ::core::ffi::c_int
                && linestatus == 0 as ::core::ffi::c_int)
            && ((*eap).line1 == 1 as linenr_T
                || diff_check_with_linestatus(
                    curwin.get(),
                    (*eap).line1 - 1 as linenr_T,
                    &raw mut linestatus,
                ) >= 0 as ::core::ffi::c_int
                    && linestatus == 0 as ::core::ffi::c_int)
        {
            (*eap).line2 += 1;
        } else if (*eap).line1 > 0 as linenr_T {
            (*eap).line1 -= 1;
        }
    }
    let mut aco: aco_save_T = aco_save_T::default();
    if (*eap).cmdidx as ::core::ffi::c_int != CMD_diffget as ::core::ffi::c_int {
        aucmd_prepbuf(
            &raw mut aco,
            (*curtab.get()).tp_diffbuf[idx_other as usize] as *mut buf_T,
        );
    }
    let idx_from: ::core::ffi::c_int =
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_diffget as ::core::ffi::c_int {
            idx_other
        } else {
            idx_cur
        };
    let idx_to: ::core::ffi::c_int =
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_diffget as ::core::ffi::c_int {
            idx_cur
        } else {
            idx_other
        };
    '_theend: {
        if (*curbuf.get()).b_changed == 0 {
            change_warning(curbuf.get(), 0 as ::core::ffi::c_int);
            if diff_buf_idx(curbuf.get(), curtab.get()) != idx_to {
                emsg(gettext(
                    b"E787: Buffer changed unexpectedly\0".as_ptr() as *const ::core::ffi::c_char
                ));
                break '_theend;
            }
        }
        diffgetput(
            (*eap).addr_count,
            idx_cur,
            idx_from,
            idx_to,
            (*eap).line1,
            (*eap).line2,
        );
        if (*eap).cmdidx as ::core::ffi::c_int != CMD_diffget as ::core::ffi::c_int {
            if KeyTyped.get() {
                u_sync(false_0 != 0);
            }
            aucmd_restbuf(&raw mut aco);
        }
    }
    diff_busy.set(false_0 != 0);
    if diff_need_update.get() {
        ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
    }
    check_cursor(curwin.get());
    changed_line_abv_curs();
    if (*curtab.get()).tp_first_diff.is_null() {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_onebuf_opt.wo_diff != 0
                && *(*wp)
                    .w_onebuf_opt
                    .wo_fdm
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == 'd' as ::core::ffi::c_int
                && (*wp).w_onebuf_opt.wo_fen != 0
            {
                foldUpdateAll(wp);
            }
            wp = (*wp).w_next;
        }
    }
    if diff_need_update.get() {
        diff_need_update.set(false_0 != 0);
    } else {
        diff_redraw(false_0 != 0);
        apply_autocmds(
            EVENT_DIFFUPDATED,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    };
}
unsafe extern "C" fn diffgetput(
    addr_count: ::core::ffi::c_int,
    idx_cur: ::core::ffi::c_int,
    idx_from: ::core::ffi::c_int,
    idx_to: ::core::ffi::c_int,
    line1: linenr_T,
    line2: linenr_T,
) {
    let mut off: linenr_T = 0 as linenr_T;
    let mut dprev: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if addr_count == 0 {
            while !(*dp).df_next.is_null()
                && (*(*dp).df_next).df_lnum[idx_cur as usize]
                    == (*dp).df_lnum[idx_cur as usize] + (*dp).df_count[idx_cur as usize]
                && (*(*dp).df_next).df_lnum[idx_cur as usize] == line1 + off + 1 as linenr_T
            {
                dprev = dp;
                dp = (*dp).df_next;
            }
        }
        if (*dp).df_lnum[idx_cur as usize] > line2 + off {
            break;
        }
        let mut dfree: diff_T = diffblock_S {
            df_next: ::core::ptr::null_mut::<diff_T>(),
            df_lnum: [0; 8],
            df_count: [0; 8],
            is_linematched: false,
            has_changes: false,
            df_changes: garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
        };
        let mut did_free: bool = false_0 != 0;
        let mut lnum: linenr_T = (*dp).df_lnum[idx_to as usize];
        let mut count: linenr_T = (*dp).df_count[idx_to as usize];
        if (*dp).df_lnum[idx_cur as usize] + (*dp).df_count[idx_cur as usize] > line1 + off
            && u_save(lnum - 1 as linenr_T, lnum + count) != FAIL
        {
            let mut start_skip: linenr_T = 0 as linenr_T;
            let mut end_skip: linenr_T = 0 as linenr_T;
            if addr_count > 0 as ::core::ffi::c_int {
                start_skip = line1 + off - (*dp).df_lnum[idx_cur as usize];
                if start_skip > 0 as linenr_T {
                    if start_skip > count {
                        lnum += count;
                        count = 0 as ::core::ffi::c_int as linenr_T;
                    } else {
                        count -= start_skip;
                        lnum += start_skip;
                    }
                } else {
                    start_skip = 0 as ::core::ffi::c_int as linenr_T;
                }
                end_skip = (*dp).df_lnum[idx_cur as usize] + (*dp).df_count[idx_cur as usize]
                    - 1 as linenr_T
                    - (line2 + off);
                if end_skip > 0 as linenr_T {
                    if idx_cur == idx_from {
                        count = if count < (*dp).df_count[idx_cur as usize] - start_skip - end_skip
                        {
                            count
                        } else {
                            (*dp).df_count[idx_cur as usize] - start_skip - end_skip
                        };
                    } else {
                        count -= end_skip;
                        end_skip = if (*dp).df_count[idx_from as usize] - start_skip - count
                            > 0 as linenr_T
                        {
                            (*dp).df_count[idx_from as usize] - start_skip - count
                        } else {
                            0 as linenr_T
                        };
                    }
                } else {
                    end_skip = 0 as ::core::ffi::c_int as linenr_T;
                }
            }
            let mut buf_empty: bool = buf_is_empty(curbuf.get());
            let mut added: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while (i as linenr_T) < count {
                buf_empty = (*curbuf.get()).b_ml.ml_line_count == 1 as linenr_T;
                if ml_delete(lnum) == OK {
                    added -= 1;
                }
                i += 1;
            }
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while (i_0 as linenr_T) < (*dp).df_count[idx_from as usize] - start_skip - end_skip {
                let mut nr: linenr_T =
                    (*dp).df_lnum[idx_from as usize] + start_skip + i_0 as linenr_T;
                if nr
                    > (*(*curtab.get()).tp_diffbuf[idx_from as usize])
                        .b_ml
                        .ml_line_count
                {
                    break;
                }
                let mut p: *mut ::core::ffi::c_char = xstrdup(ml_get_buf(
                    (*curtab.get()).tp_diffbuf[idx_from as usize] as *mut buf_T,
                    nr,
                ));
                ml_append(
                    lnum + i_0 as linenr_T - 1 as linenr_T,
                    p,
                    0 as colnr_T,
                    false_0 != 0,
                );
                xfree(p as *mut ::core::ffi::c_void);
                added += 1;
                if buf_empty as ::core::ffi::c_int != 0
                    && (*curbuf.get()).b_ml.ml_line_count == 2 as linenr_T
                {
                    buf_empty = false_0 != 0;
                    ml_delete(2 as linenr_T);
                }
                i_0 += 1;
            }
            let mut new_count: linenr_T = (*dp).df_count[idx_to as usize] + added as linenr_T;
            (*dp).df_count[idx_to as usize] = new_count;
            if start_skip == 0 as linenr_T && end_skip == 0 as linenr_T {
                let mut i_1: ::core::ffi::c_int = 0;
                i_1 = 0 as ::core::ffi::c_int;
                while i_1 < DB_COUNT {
                    if !(*curtab.get()).tp_diffbuf[i_1 as usize].is_null()
                        && i_1 != idx_from
                        && i_1 != idx_to
                        && !diff_equal_entry(dp, idx_from, i_1)
                    {
                        break;
                    }
                    i_1 += 1;
                }
                if i_1 == DB_COUNT {
                    dfree = *dp;
                    did_free = true_0 != 0;
                    dp = diff_free(curtab.get(), dprev, dp);
                }
            }
            if added != 0 as ::core::ffi::c_int {
                mark_adjust(
                    lnum,
                    lnum + count - 1 as linenr_T,
                    MAXLNUM as ::core::ffi::c_int as linenr_T,
                    added as linenr_T,
                    kExtmarkNOOP,
                );
                if (*curwin.get()).w_cursor.lnum >= lnum {
                    if (*curwin.get()).w_cursor.lnum >= lnum + count {
                        (*curwin.get()).w_cursor.lnum =
                            ((*curwin.get()).w_cursor.lnum as ::core::ffi::c_int + added)
                                as linenr_T;
                        (*curwin.get()).w_cursor.lnum =
                            if (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count {
                                (*curwin.get()).w_cursor.lnum
                            } else {
                                (*curbuf.get()).b_ml.ml_line_count
                            };
                    } else if added < 0 as ::core::ffi::c_int {
                        (*curwin.get()).w_cursor.lnum = lnum;
                    }
                }
            }
            extmark_adjust(
                curbuf.get(),
                lnum,
                lnum + count - 1 as linenr_T,
                MAXLNUM as ::core::ffi::c_int as linenr_T,
                added as linenr_T,
                kExtmarkUndo,
            );
            changed_lines(
                curbuf.get(),
                lnum,
                0 as colnr_T,
                lnum + count,
                added as linenr_T,
                true_0 != 0,
            );
            if did_free {
                diff_fold_update(&raw mut dfree, idx_to);
            }
            if added != 0 as ::core::ffi::c_int && !valid_diff(dp) {
                break;
            }
            if !did_free {
                (*dp).df_count[idx_to as usize] = new_count;
            }
            if idx_cur == idx_to {
                off = (off as ::core::ffi::c_int + added) as linenr_T;
            }
        }
        if !did_free {
            dprev = dp;
            dp = (*dp).df_next;
        }
    }
}
unsafe extern "C" fn diff_fold_update(mut dp: *mut diff_T, mut skip_idx: ::core::ffi::c_int) {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < DB_COUNT {
            if (*curtab.get()).tp_diffbuf[i as usize] == (*wp).w_buffer && i != skip_idx {
                foldUpdate(
                    wp,
                    (*dp).df_lnum[i as usize],
                    (*dp).df_lnum[i as usize] + (*dp).df_count[i as usize],
                );
            }
            i += 1;
        }
        wp = (*wp).w_next;
    }
}
pub unsafe extern "C" fn diff_mode_buf(mut buf: *mut buf_T) -> bool {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        if diff_buf_idx(buf, tp as *mut tabpage_T) != DB_COUNT {
            return true_0 != 0;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn diff_move_to(
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    let mut idx: ::core::ffi::c_int = diff_buf_idx(curbuf.get(), curtab.get());
    if idx == DB_COUNT || (*curtab.get()).tp_first_diff.is_null() {
        return FAIL;
    }
    if (*curtab.get()).tp_diff_invalid != 0 {
        ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
    }
    if (*curtab.get()).tp_first_diff.is_null() {
        return FAIL;
    }
    loop {
        count -= 1;
        if count < 0 as ::core::ffi::c_int {
            break;
        }
        if dir == BACKWARD as ::core::ffi::c_int
            && lnum <= (*(*curtab.get()).tp_first_diff).df_lnum[idx as usize]
        {
            break;
        }
        let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        dp = (*curtab.get()).tp_first_diff;
        while !dp.is_null() {
            if dir == FORWARD as ::core::ffi::c_int && lnum < (*dp).df_lnum[idx as usize]
                || dir == BACKWARD as ::core::ffi::c_int
                    && ((*dp).df_next.is_null() || lnum <= (*(*dp).df_next).df_lnum[idx as usize])
            {
                lnum = (*dp).df_lnum[idx as usize];
                break;
            } else {
                dp = (*dp).df_next;
            }
        }
    }
    lnum = if lnum < (*curbuf.get()).b_ml.ml_line_count {
        lnum
    } else {
        (*curbuf.get()).b_ml.ml_line_count
    };
    if lnum == (*curwin.get()).w_cursor.lnum {
        return FAIL;
    }
    setpcmark();
    (*curwin.get()).w_cursor.lnum = lnum;
    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    return OK;
}
unsafe extern "C" fn diff_get_corresponding_line_int(
    mut buf1: *mut buf_T,
    mut lnum1: linenr_T,
) -> linenr_T {
    let mut baseline: linenr_T = 0 as linenr_T;
    let mut idx1: ::core::ffi::c_int = diff_buf_idx(buf1, curtab.get());
    let mut idx2: ::core::ffi::c_int = diff_buf_idx(curbuf.get(), curtab.get());
    if idx1 == DB_COUNT || idx2 == DB_COUNT || (*curtab.get()).tp_first_diff.is_null() {
        return lnum1;
    }
    if (*curtab.get()).tp_diff_invalid != 0 {
        ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
    }
    if (*curtab.get()).tp_first_diff.is_null() {
        return lnum1;
    }
    let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if (*dp).df_lnum[idx1 as usize] > lnum1 {
            return lnum1 - baseline;
        }
        if (*dp).df_lnum[idx1 as usize] + (*dp).df_count[idx1 as usize] > lnum1 {
            baseline = lnum1 - (*dp).df_lnum[idx1 as usize];
            baseline = if baseline < (*dp).df_count[idx2 as usize] {
                baseline
            } else {
                (*dp).df_count[idx2 as usize]
            };
            return (*dp).df_lnum[idx2 as usize] + baseline;
        }
        if (*dp).df_lnum[idx1 as usize] == lnum1
            && (*dp).df_count[idx1 as usize] == 0 as linenr_T
            && (*dp).df_lnum[idx2 as usize] <= (*curwin.get()).w_cursor.lnum
            && (*dp).df_lnum[idx2 as usize] + (*dp).df_count[idx2 as usize]
                > (*curwin.get()).w_cursor.lnum
        {
            return (*curwin.get()).w_cursor.lnum;
        }
        baseline = (*dp).df_lnum[idx1 as usize] + (*dp).df_count[idx1 as usize]
            - ((*dp).df_lnum[idx2 as usize] + (*dp).df_count[idx2 as usize]);
        dp = (*dp).df_next;
    }
    return lnum1 - baseline;
}
pub unsafe extern "C" fn diff_get_corresponding_line(
    mut buf1: *mut buf_T,
    mut lnum1: linenr_T,
) -> linenr_T {
    let mut lnum: linenr_T = diff_get_corresponding_line_int(buf1, lnum1);
    return if lnum < (*curbuf.get()).b_ml.ml_line_count {
        lnum
    } else {
        (*curbuf.get()).b_ml.ml_line_count
    };
}
pub unsafe extern "C" fn diff_lnum_win(mut lnum: linenr_T, mut wp: *mut win_T) -> linenr_T {
    let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut idx: ::core::ffi::c_int = diff_buf_idx(curbuf.get(), curtab.get());
    if idx == DB_COUNT {
        return 0 as linenr_T;
    }
    if (*curtab.get()).tp_diff_invalid != 0 {
        ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
    }
    dp = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if lnum <= (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
            break;
        }
        dp = (*dp).df_next;
    }
    if dp.is_null() {
        return (*(*wp).w_buffer).b_ml.ml_line_count - ((*curbuf.get()).b_ml.ml_line_count - lnum);
    }
    let mut i: ::core::ffi::c_int = diff_buf_idx((*wp).w_buffer, curtab.get());
    if i == DB_COUNT {
        return 0 as linenr_T;
    }
    let mut n: linenr_T = lnum + ((*dp).df_lnum[i as usize] - (*dp).df_lnum[idx as usize]);
    return if n < (*dp).df_lnum[i as usize] + (*dp).df_count[i as usize] {
        n
    } else {
        (*dp).df_lnum[i as usize] + (*dp).df_count[i as usize]
    };
}
unsafe extern "C" fn parse_diff_ed(
    mut line: *mut ::core::ffi::c_char,
    mut hunk: *mut diffhunk_T,
) -> ::core::ffi::c_int {
    let mut l1: ::core::ffi::c_int = 0;
    let mut l2: ::core::ffi::c_int = 0;
    let mut p: *mut ::core::ffi::c_char = line;
    let mut f1: linenr_T = getdigits_int32(&raw mut p, true_0 != 0, 0 as int32_t);
    if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
        p = p.offset(1);
        l1 = getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
    } else {
        l1 = f1 as ::core::ffi::c_int;
    }
    if *p as ::core::ffi::c_int != 'a' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != 'c' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != 'd' as ::core::ffi::c_int
    {
        return FAIL;
    }
    let c2rust_fresh6 = p;
    p = p.offset(1);
    let mut difftype: ::core::ffi::c_int = *c2rust_fresh6 as uint8_t as ::core::ffi::c_int;
    let mut f2: ::core::ffi::c_int =
        getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
    if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
        p = p.offset(1);
        l2 = getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
    } else {
        l2 = f2;
    }
    if (l1 as linenr_T) < f1 || l2 < f2 {
        return FAIL;
    }
    if difftype == 'a' as ::core::ffi::c_int {
        (*hunk).lnum_orig = f1 + 1 as linenr_T;
        (*hunk).count_orig = 0 as ::core::ffi::c_int;
    } else {
        (*hunk).lnum_orig = f1;
        (*hunk).count_orig = (l1 as linenr_T - f1 + 1 as linenr_T) as ::core::ffi::c_int;
    }
    if difftype == 'd' as ::core::ffi::c_int {
        (*hunk).lnum_new = f2 as linenr_T + 1 as linenr_T;
        (*hunk).count_new = 0 as ::core::ffi::c_int;
    } else {
        (*hunk).lnum_new = f2 as linenr_T;
        (*hunk).count_new = l2 - f2 + 1 as ::core::ffi::c_int;
    }
    return OK;
}
unsafe extern "C" fn parse_diff_unified(
    mut line: *mut ::core::ffi::c_char,
    mut hunk: *mut diffhunk_T,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = line;
    let c2rust_fresh0 = p;
    p = p.offset(1);
    if *c2rust_fresh0 as ::core::ffi::c_int == '@' as ::core::ffi::c_int
        && {
            let c2rust_fresh1 = p;
            p = p.offset(1);
            *c2rust_fresh1 as ::core::ffi::c_int == '@' as ::core::ffi::c_int
        }
        && {
            let c2rust_fresh2 = p;
            p = p.offset(1);
            *c2rust_fresh2 as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
        }
        && {
            let c2rust_fresh3 = p;
            p = p.offset(1);
            *c2rust_fresh3 as ::core::ffi::c_int == '-' as ::core::ffi::c_int
        }
    {
        let mut oldcount: ::core::ffi::c_int = 0;
        let mut newline: linenr_T = 0;
        let mut newcount: ::core::ffi::c_int = 0;
        let mut oldline: linenr_T = getdigits_int32(&raw mut p, true_0 != 0, 0 as int32_t);
        if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            p = p.offset(1);
            oldcount = getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
        } else {
            oldcount = 1 as ::core::ffi::c_int;
        }
        let c2rust_fresh4 = p;
        p = p.offset(1);
        if *c2rust_fresh4 as ::core::ffi::c_int == ' ' as ::core::ffi::c_int && {
            let c2rust_fresh5 = p;
            p = p.offset(1);
            *c2rust_fresh5 as ::core::ffi::c_int == '+' as ::core::ffi::c_int
        } {
            newline = getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int) as linenr_T;
            if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                p = p.offset(1);
                newcount = getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
            } else {
                newcount = 1 as ::core::ffi::c_int;
            }
        } else {
            return FAIL;
        }
        if oldcount == 0 as ::core::ffi::c_int {
            oldline = (oldline as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as linenr_T;
        }
        if newcount == 0 as ::core::ffi::c_int {
            newline = (newline as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as linenr_T;
        }
        if newline == 0 as linenr_T {
            newline = 1 as ::core::ffi::c_int as linenr_T;
        }
        (*hunk).lnum_orig = oldline;
        (*hunk).count_orig = oldcount;
        (*hunk).lnum_new = newline;
        (*hunk).count_new = newcount;
        return OK;
    }
    return FAIL;
}
unsafe extern "C" fn xdiff_out(
    mut start_a: ::core::ffi::c_int,
    mut count_a: ::core::ffi::c_int,
    mut start_b: ::core::ffi::c_int,
    mut count_b: ::core::ffi::c_int,
    mut priv_0: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut dout: *mut diffout_T = priv_0 as *mut diffout_T;
    ga_grow(&raw mut (*dout).dout_ga, 1 as ::core::ffi::c_int);
    *((*dout).dout_ga.ga_data as *mut diffhunk_T).offset((*dout).dout_ga.ga_len as isize) =
        diffhunk_T {
            lnum_orig: start_a as linenr_T + 1 as linenr_T,
            count_orig: count_a,
            lnum_new: start_b as linenr_T + 1 as linenr_T,
            count_new: count_b,
        };
    (*dout).dout_ga.ga_len += 1;
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn f_diff_filler(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number =
        (if 0 as ::core::ffi::c_int > diff_check_fill(curwin.get(), tv_get_lnum(argvars)) {
            0 as ::core::ffi::c_int
        } else {
            diff_check_fill(curwin.get(), tv_get_lnum(argvars))
        }) as varnumber_T;
}
pub unsafe extern "C" fn f_diff_hlID(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    static prev_lnum: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
    static changedtick: GlobalCell<varnumber_T> = GlobalCell::new(0 as varnumber_T);
    static fnum: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    static prev_diff_flags: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    static change_start: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    static change_end: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    static hlID: GlobalCell<hlf_T> = GlobalCell::new(HLF_NONE);
    let mut diffline: diffline_T = diffline_S {
        changes: ::core::ptr::null_mut::<diffline_change_T>(),
        num_changes: 0,
        bufidx: 0,
        lineoff: 0,
    };
    let cache_results: bool = diff_flags.get() & ALL_INLINE_DIFF == 0;
    let mut lnum: linenr_T = tv_get_lnum(argvars);
    if lnum < 0 as linenr_T {
        lnum = 0 as ::core::ffi::c_int as linenr_T;
    }
    if !cache_results
        || lnum != prev_lnum.get()
        || changedtick.get() != buf_get_changedtick(curbuf.get())
        || fnum.get() != (*curbuf.get()).handle
        || diff_flags.get() != prev_diff_flags.get()
    {
        let mut linestatus: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        diff_check_with_linestatus(curwin.get(), lnum, &raw mut linestatus);
        if linestatus < 0 as ::core::ffi::c_int {
            if linestatus == -1 as ::core::ffi::c_int {
                change_start.set(MAXCOL as ::core::ffi::c_int);
                change_end.set(-1 as ::core::ffi::c_int);
                if diff_find_change(curwin.get(), lnum, &raw mut diffline) {
                    hlID.set(HLF_ADD);
                } else {
                    hlID.set(HLF_CHD);
                    if diffline.num_changes > 0 as ::core::ffi::c_int
                        && cache_results as ::core::ffi::c_int != 0
                    {
                        change_start.set(
                            (*diffline.changes.offset(0 as ::core::ffi::c_int as isize)).dc_start
                                [diffline.bufidx as usize]
                                as ::core::ffi::c_int,
                        );
                        change_end.set(
                            (*diffline.changes.offset(0 as ::core::ffi::c_int as isize)).dc_end
                                [diffline.bufidx as usize]
                                as ::core::ffi::c_int,
                        );
                    }
                }
            } else {
                hlID.set(HLF_ADD);
            }
        } else {
            hlID.set(HLF_NONE);
        }
        if cache_results {
            prev_lnum.set(lnum);
            changedtick.set(buf_get_changedtick(curbuf.get()));
            fnum.set((*curbuf.get()).handle as ::core::ffi::c_int);
            prev_diff_flags.set(diff_flags.get());
        }
    }
    if hlID.get() as ::core::ffi::c_uint == HLF_CHD as ::core::ffi::c_uint
        || hlID.get() as ::core::ffi::c_uint == HLF_TXD as ::core::ffi::c_uint
    {
        let mut col: ::core::ffi::c_int =
            tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int;
        if cache_results {
            if col >= change_start.get() && col < change_end.get() {
                hlID.set(HLF_TXD);
            } else {
                hlID.set(HLF_CHD);
            }
        } else {
            hlID.set(HLF_CHD);
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < diffline.num_changes {
                let mut added: bool = diff_change_parse(
                    &raw mut diffline,
                    diffline.changes.offset(i as isize),
                    change_start.ptr(),
                    change_end.ptr(),
                );
                if col >= change_start.get() && col < change_end.get() {
                    hlID.set(
                        (if added as ::core::ffi::c_int != 0 {
                            HLF_TXA
                        } else {
                            HLF_TXD
                        }) as hlf_T,
                    );
                    break;
                } else {
                    if col < change_start.get() {
                        break;
                    }
                    i += 1;
                }
            }
        }
    }
    (*rettv).vval.v_number = hlID.get() as varnumber_T;
}
pub const XDF_NEED_MINIMAL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int;
pub const XDF_IGNORE_WHITESPACE: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
pub const XDF_IGNORE_WHITESPACE_CHANGE: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int;
pub const XDF_IGNORE_WHITESPACE_AT_EOL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int;
pub const XDF_IGNORE_BLANK_LINES: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int;
pub const XDF_PATIENCE_DIFF: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 14 as ::core::ffi::c_int;
pub const XDF_HISTOGRAM_DIFF: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int;
pub const XDF_INDENT_HEURISTIC: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 23 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
