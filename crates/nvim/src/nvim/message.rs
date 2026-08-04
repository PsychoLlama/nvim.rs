use crate::src::nvim::api::private::helpers::{
    api_free_array, copy_string, cstr_as_string, cstr_to_string, ga_take_string,
};
use crate::src::nvim::api::vim::nvim_echo;
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::autocmd::{EVENT_PROGRESS, apply_autocmds_group, has_event};
use crate::src::nvim::charset::{
    byte2cells, char2cells, getdigits_int, ptr2cells, skipwhite, transchar_buf, transchar_byte_buf,
    vim_isprintc, vim_strsize,
};
use crate::src::nvim::drawscreen::{
    UPD_NOT_VALID, UPD_VALID, redraw_all_later, redraw_later, set_must_redraw,
};
use crate::src::nvim::eval::callback_call;
use crate::src::nvim::eval::typval::{kCallbackNone, tv_clear};
use crate::src::nvim::eval::vars::{get_vim_var_str, set_vim_var_string, var_redir_str};
use crate::src::nvim::event::r#loop::loop_schedule_deferred;
use crate::src::nvim::event::multiqueue::multiqueue_process_events;
use crate::src::nvim::ex_docmd::do_sleep;
use crate::src::nvim::ex_eval::cause_errthrow;
use crate::src::nvim::fileio::check_timestamps;
use crate::src::nvim::garray::{ga_append, ga_concat, ga_concat_len, ga_init};
use crate::src::nvim::getchar::{
    beep_flush, char_avail, flush_buffers, ins_char_typebuf, safe_vgetc, stuff_empty,
    typeahead_noflush,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{
    grid_adjust, grid_alloc, grid_assign_handle, grid_clear, grid_clear_line, grid_del_lines,
    grid_free, grid_ins_lines, grid_line_cursor_goto, grid_line_flush,
    grid_line_flush_if_valid_row, grid_line_mirror, grid_line_puts, grid_line_start, schar_get,
};
use crate::src::nvim::highlight::hl_combine_attr;
use crate::src::nvim::highlight_group::{
    HLF_0, HLF_8, HLF_AT, HLF_E, HLF_M, HLF_MSG, HLF_N, HLF_R, HLF_T, HLF_W, highlight_changed,
    syn_check_group, syn_id2attr,
};
use crate::src::nvim::indent::tabstop_padding;
use crate::src::nvim::input::{get_keystroke, prompt_for_input};
use crate::src::nvim::keycodes::{
    K_BS, K_DOWN, K_EVENT, K_PAGEDOWN, K_PAGEUP, K_SPECIAL, K_UP, K_ZERO, get_special_key_name,
};
use crate::src::nvim::log::{LOGLVL_DBG, LOGLVL_INF, logmsg};
use crate::src::nvim::main::{
    Columns, IObuff, KeyTyped, Rows, State, allow_keys, called_emsg, capture_ga, clear_cmdline,
    cmd_silent, cmdline_row, cmdline_was_last_drawn, cmdmod, cmdmsg_rl, curbuf, curwin,
    default_grid, did_emsg, did_wait_return, do_redraw, e_intern2, e_invarg, e_notopen,
    embedded_mode, emsg_assert_fails_context, emsg_assert_fails_lnum, emsg_assert_fails_msg,
    emsg_noredir, emsg_off, emsg_on_display, emsg_severe, emsg_silent, emsg_skip, ex_exitval,
    exiting, exmode_active, full_screen, global_busy, got_int, headless_mode, hl_attr_active,
    in_assert_fails, info_message, keep_msg, keep_msg_hl_id, lines_left, main_loop, mode_displayed,
    msg_buf, msg_col, msg_did_scroll, msg_didany, msg_didout, msg_ext_overwrite,
    msg_ext_skip_flush, msg_ext_skip_verbose, msg_grid, msg_grid_adj, msg_grid_pos,
    msg_grid_scroll_discount, msg_hist_off, msg_no_more, msg_nowait, msg_row, msg_scroll,
    msg_scrolled, msg_scrolled_at_flush, msg_scrolled_ign, msg_silent, need_check_timestamps,
    need_clr_eos, need_fileinfo, need_highlight_changed, need_wait_return, no_mapping,
    no_wait_return, nvim_testing, on_print, p_ch, p_debug, p_eb, p_lz, p_mopt, p_more, p_report,
    p_verbose, quit_more, rdb_flags, redir_fd, redir_off, redir_reg, redir_vname, redraw_cmdline,
    redrawing_cmdline, reg_recording, resize_events, sc_col, scriptout, silent_mode, skip_redraw,
    vgetc_busy, vgetc_char, vgetc_mod_mask,
};
use crate::src::nvim::mbyte::{
    mb_string2cells, mb_string2cells_len, mb_tolower, mb_unescape, utf_char2bytes, utf_char2cells,
    utf_head_off, utf_ptr2cells, utf_ptr2char, utf_ptr2len, utf8len_tab, utfc_ptr2len,
    utfc_ptr2len_len,
};
use crate::src::nvim::memory::{
    arena_alloc, strequal, strnequal, xcalloc, xfree, xmalloc, xmemdupz, xmemrchr, xrealloc,
    xstrdup, xstrlcat, xstrlcpy,
};
use crate::src::nvim::mouse::{jump_to_mouse, setmouse};
use crate::src::nvim::option::{p_vfile, shortmess};
use crate::src::nvim::options::{
    kOptBoFlagMess, kOptBoFlagShell, kOptMoptFlagHistory, kOptMoptFlagHitEnter,
    kOptMoptFlagProgress, kOptMoptFlagWait, kOptRdbFlagNothrottle,
};
use crate::src::nvim::os::env::home_replace_save;
use crate::src::nvim::os::fs::os_fopen;
use crate::src::nvim::os::input::{input_available, os_breakcheck};
use crate::src::nvim::os::libc::{
    __assert_fail, abort, abs, fclose, fprintf, fputs, gettext, memchr, memcpy, memmove, ngettext,
    printf, putc, snprintf, stderr, strcmp, strlen, strnlen,
};
use crate::src::nvim::os::time::os_delay;
use crate::src::nvim::register::write_reg_contents;
use crate::src::nvim::runtime::{estack_sfile, exestack};
use crate::src::nvim::state::{
    MODE_ASKMORE, MODE_CMDLINE, MODE_EXTERNCMD, MODE_HITRETURN, MODE_SETWSIZE,
};
use crate::src::nvim::strings::{vim_snprintf, vim_snprintf_safelen, vim_strchr, vim_vsnprintf};
use crate::src::nvim::types::api::kErrorTypeNone;
use crate::src::nvim::types::ui::{kUIMessages, kUIMultigrid};
use crate::src::nvim::types::{
    Arena, Array, BoolVarValue, Boolean, CMD_index, Dict, Error, Event, FILE, HlMessage,
    HlMessageChunk, Integer, KeyDict_echo_opts, KeyValuePair, MessageData, Object, OptInt,
    ScopeType, ScreenGrid, SpecialVarValue, String_0, VarLockStatus, VarType, VimVarIndex, buf_T,
    cmd_addr_T, colnr_T, cstack_T, estack_T, estack_arg_T, exarg, exarg_T, flush_buffers_T,
    garray_T, int64_t, kObjectTypeArray, kObjectTypeBoolean, kObjectTypeDict, kObjectTypeInteger,
    kObjectTypeNil, kObjectTypeString, key_extra, key_value_pair, linenr_T, object,
    object_data as C2Rust_Unnamed_11, ptrdiff_t, regmatch_T, sattr_T, schar_T, size_t, ssize_t,
    typval_T, typval_vval_union, uint8_t, uint32_t, uint64_t,
};
use crate::src::nvim::ui::{
    ui_active, ui_call_grid_destroy, ui_call_grid_resize, ui_call_grid_scroll,
    ui_call_msg_history_show, ui_call_msg_set_pos, ui_call_msg_show, ui_call_msg_showmode,
    ui_cursor_goto, ui_flush, ui_grid_cursor_goto, ui_has, ui_line, ui_refresh, vim_beep,
};
use crate::src::nvim::ui_compositor::{ui_comp_put_grid, ui_comp_remove_grid};
unsafe extern "C" {
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
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
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const kZIndexMessages: C2Rust_Unnamed_27 = 200;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_hist {
    pub next: *mut msg_hist,
    pub prev: *mut msg_hist,
    pub msg: HlMessage,
    pub kind: *mut ::core::ffi::c_char,
    pub temp: bool,
    pub append: bool,
}
pub type MessageHistoryEntry = msg_hist;
pub const CMD_snext: CMD_index = 414;
pub const CMD_drop: CMD_index = 130;
pub const CMD_arglocal: CMD_index = 14;
pub const CMD_argglobal: CMD_index = 13;
pub const CMD_argdo: CMD_index = 10;
pub const CMD_args: CMD_index = 7;
pub const CMD_append: CMD_index = 0;
pub const ADDR_LINES: cmd_addr_T = 0;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_int;
pub const AUGROUP_ALL: C2Rust_Unnamed_30 = -3;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const SHM_TRUNCALL: C2Rust_Unnamed_34 = 84;
pub const SHM_TRUNC: C2Rust_Unnamed_34 = 116;
pub const VV_SCROLLSTART: VimVarIndex = 46;
pub const VV_FOLDLEVEL: VimVarIndex = 26;
pub const VV_FOLDDASHES: VimVarIndex = 25;
pub const VV_FOLDEND: VimVarIndex = 24;
pub const VV_FOLDSTART: VimVarIndex = 23;
pub const VV_LNUM: VimVarIndex = 9;
pub const VV_STATUSMSG: VimVarIndex = 5;
pub const VV_WARNINGMSG: VimVarIndex = 4;
pub const VV_ERRMSG: VimVarIndex = 3;
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub const MB_MAXBYTES: C2Rust_Unnamed_36 = 21;
pub type C2Rust_Unnamed_37 = ::core::ffi::c_uint;
pub const VIM_DISCARDALL: C2Rust_Unnamed_37 = 6;
pub const VIM_ALL: C2Rust_Unnamed_37 = 5;
pub const VIM_CANCEL: C2Rust_Unnamed_37 = 4;
pub const VIM_NO: C2Rust_Unnamed_37 = 3;
pub const VIM_YES: C2Rust_Unnamed_37 = 2;
pub const MOUSE_SETPOS: C2Rust_Unnamed_40 = 8;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const KE_LEFTMOUSE: key_extra = 44;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_RIGHTDRAG: key_extra = 51;
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_MIDDLEDRAG: key_extra = 48;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_IGNORE: key_extra = 53;
pub type msgchunk_T = msgchunk_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msgchunk_S {
    pub sb_next: *mut msgchunk_T,
    pub sb_prev: *mut msgchunk_T,
    pub sb_eol: ::core::ffi::c_char,
    pub sb_msg_col: ::core::ffi::c_int,
    pub sb_hl_id: ::core::ffi::c_int,
    pub sb_text: [::core::ffi::c_char; 0],
}
pub type sb_clear_T = ::core::ffi::c_uint;
pub const SB_CLEAR_CMDLINE_DONE: sb_clear_T = 3;
pub const SB_CLEAR_CMDLINE_BUSY: sb_clear_T = 2;
pub const SB_CLEAR_ALL: sb_clear_T = 1;
pub const SB_CLEAR_NONE: sb_clear_T = 0;
pub const ESTACK_NONE: estack_arg_T = 0;
pub const FLUSH_MINIMAL: flush_buffers_T = 0;
pub const DLG_HOTKEY_CHAR: C2Rust_Unnamed_41 = 38;
pub const DLG_BUTTON_SEP: C2Rust_Unnamed_41 = 10;
pub type C2Rust_Unnamed_40 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_41 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const BELL: ::core::ffi::c_int = '\u{7}' as ::core::ffi::c_int;
pub const BS: ::core::ffi::c_int = 8;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = 10;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = 27;
pub const Ctrl_B: ::core::ffi::c_int = 2;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const Ctrl_F: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
static confirm_msg_used: GlobalCell<::core::ffi::c_int> = GlobalCell::new(false_0);
static confirm_msg: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static confirm_buttons: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
#[unsafe(no_mangle)]
pub static msg_hist_last: GlobalCell<*mut MessageHistoryEntry> =
    GlobalCell::new(::core::ptr::null_mut::<MessageHistoryEntry>());
static msg_hist_first: GlobalCell<*mut MessageHistoryEntry> =
    GlobalCell::new(::core::ptr::null_mut::<MessageHistoryEntry>());
static msg_hist_temp: GlobalCell<*mut MessageHistoryEntry> =
    GlobalCell::new(::core::ptr::null_mut::<MessageHistoryEntry>());
static msg_hist_len: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static msg_hist_max: GlobalCell<::core::ffi::c_int> = GlobalCell::new(500 as ::core::ffi::c_int);
pub const PROGRESS_TARGET_CMD: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
static msg_flags: GlobalCell<::core::ffi::c_int> = GlobalCell::new(
    kOptMoptFlagHitEnter as ::core::ffi::c_int
        | kOptMoptFlagHistory as ::core::ffi::c_int
        | kOptMoptFlagProgress as ::core::ffi::c_int,
);
static msg_wait: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static progress_msg_target: GlobalCell<::core::ffi::c_int> = GlobalCell::new(PROGRESS_TARGET_CMD);
static verbose_fd: GlobalCell<*mut FILE> = GlobalCell::new(::core::ptr::null_mut::<FILE>());
static verbose_did_open: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static keep_msg_more: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static msg_ext_kind: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null::<::core::ffi::c_char>());
static msg_ext_trigger: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null::<::core::ffi::c_char>());
static msg_ext_id: GlobalCell<Object> = GlobalCell::new(object {
    type_0: kObjectTypeInteger,
    data: C2Rust_Unnamed_11 {
        integer: 1 as Integer,
    },
});
static msg_ext_chunks: GlobalCell<*mut Array> = GlobalCell::new(::core::ptr::null_mut::<Array>());
static msg_ext_last_chunk: GlobalCell<garray_T> = GlobalCell::new(garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
    ga_growsize: 40 as ::core::ffi::c_int,
    ga_data: NULL,
});
static msg_ext_last_attr: GlobalCell<sattr_T> = GlobalCell::new(-1 as sattr_T);
static msg_ext_last_hl_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static msg_ext_history: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static msg_ext_append: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static msg_grid_pos_at_flush: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
static msg_id_next: GlobalCell<int64_t> = GlobalCell::new(1 as int64_t);
pub unsafe extern "C" fn msg_id_exists(mut id: int64_t) -> bool {
    return id > 0 as int64_t && id < msg_id_next.get();
}
unsafe extern "C" fn ui_ext_msg_set_pos(mut row: ::core::ffi::c_int, mut scrolled: bool) {
    let mut buf: [::core::ffi::c_char; 32] = [0; 32];
    let mut size: size_t = schar_get(
        &raw mut buf as *mut ::core::ffi::c_char,
        (*curwin.get()).w_p_fcs_chars.msgsep,
    );
    ui_call_msg_set_pos(
        (*msg_grid.ptr()).handle as Integer,
        row as Integer,
        scrolled as Boolean,
        String_0 {
            data: &raw mut buf as *mut ::core::ffi::c_char,
            size: size,
        },
        (*msg_grid.ptr()).zindex as Integer,
        (*msg_grid.ptr()).comp_index as ::core::ffi::c_int as Integer,
    );
    (*msg_grid.ptr()).pending_comp_index_update = false_0 != 0;
}
pub unsafe extern "C" fn msg_grid_set_pos(mut row: ::core::ffi::c_int, mut scrolled: bool) {
    if !(*msg_grid.ptr()).throttled {
        ui_ext_msg_set_pos(row, scrolled);
        msg_grid_pos_at_flush.set(row);
    }
    msg_grid_pos.set(row);
    if !(*msg_grid.ptr()).chars.is_null() {
        (*msg_grid_adj.ptr()).row_offset = -row;
    }
}
pub unsafe extern "C" fn msg_use_grid() -> bool {
    return !(*default_grid.ptr()).chars.is_null() && !ui_has(kUIMessages);
}
pub unsafe extern "C" fn msg_grid_validate() {
    grid_assign_handle(msg_grid.ptr());
    let mut should_alloc: bool = msg_use_grid();
    let mut max_rows: ::core::ffi::c_int = Rows.get() - p_ch.get() as ::core::ffi::c_int;
    if should_alloc as ::core::ffi::c_int != 0
        && ((*msg_grid.ptr()).rows != Rows.get()
            || (*msg_grid.ptr()).cols != Columns.get()
            || (*msg_grid.ptr()).chars.is_null())
    {
        grid_alloc(
            msg_grid.ptr(),
            Rows.get(),
            Columns.get(),
            false_0 != 0,
            true_0 != 0,
        );
        (*msg_grid.ptr()).zindex = kZIndexMessages as ::core::ffi::c_int;
        xfree((*msg_grid.ptr()).dirty_col as *mut ::core::ffi::c_void);
        (*msg_grid.ptr()).dirty_col = xcalloc(
            Rows.get() as size_t,
            ::core::mem::size_of::<::core::ffi::c_int>(),
        ) as *mut ::core::ffi::c_int;
        let mut pos: ::core::ffi::c_int = if State.get() & MODE_ASKMORE != 0 {
            0 as ::core::ffi::c_int
        } else if max_rows - msg_scrolled.get() > 0 as ::core::ffi::c_int {
            max_rows - msg_scrolled.get()
        } else {
            0 as ::core::ffi::c_int
        };
        (*msg_grid.ptr()).throttled = false_0 != 0;
        msg_grid_set_pos(pos, msg_scrolled.get() != 0);
        ui_comp_put_grid(
            msg_grid.ptr(),
            pos,
            0 as ::core::ffi::c_int,
            (*msg_grid.ptr()).rows,
            (*msg_grid.ptr()).cols,
            false_0 != 0,
            true_0 != 0,
        );
        ui_call_grid_resize(
            (*msg_grid.ptr()).handle as Integer,
            (*msg_grid.ptr()).cols as Integer,
            (*msg_grid.ptr()).rows as Integer,
        );
        msg_scrolled_at_flush.set(msg_scrolled.get());
        (*msg_grid.ptr()).mouse_enabled = false_0 != 0;
        (*msg_grid_adj.ptr()).target = msg_grid.ptr();
    } else if !should_alloc && !(*msg_grid.ptr()).chars.is_null() {
        ui_comp_remove_grid(msg_grid.ptr());
        grid_free(msg_grid.ptr());
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*msg_grid.ptr()).dirty_col as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        ui_call_grid_destroy((*msg_grid.ptr()).handle as Integer);
        (*msg_grid.ptr()).throttled = false_0 != 0;
        (*msg_grid_adj.ptr()).row_offset = 0 as ::core::ffi::c_int;
        (*msg_grid_adj.ptr()).target = default_grid.ptr();
        redraw_cmdline.set(true_0 != 0);
    } else if !(*msg_grid.ptr()).chars.is_null()
        && msg_scrolled.get() == 0
        && msg_grid_pos.get() != max_rows
    {
        let mut diff: ::core::ffi::c_int = msg_grid_pos.get() - max_rows;
        msg_grid_set_pos(max_rows, false_0 != 0);
        if diff > 0 as ::core::ffi::c_int {
            grid_clear(
                msg_grid_adj.ptr(),
                Rows.get() - diff,
                Rows.get(),
                0 as ::core::ffi::c_int,
                Columns.get(),
                *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
            );
        }
    }
    if !(*msg_grid.ptr()).chars.is_null()
        && msg_scrolled.get() == 0
        && cmdline_row.get() < msg_grid_pos.get()
    {
        cmdline_row.set(msg_grid_pos.get());
    }
}
pub unsafe extern "C" fn verb_msg(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    verbose_enter();
    let mut n: ::core::ffi::c_int =
        msg_keep(s, 0 as ::core::ffi::c_int, false_0 != 0, false_0 != 0) as ::core::ffi::c_int;
    verbose_leave();
    return n;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn msg(mut s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool {
    return msg_keep(s, hl_id, false_0 != 0, false_0 != 0);
}
pub unsafe extern "C" fn msg_multiline(
    mut str: String_0,
    mut hl_id: ::core::ffi::c_int,
    mut check_int: bool,
    mut hist: bool,
    mut need_clear: *mut bool,
) {
    let mut s: *const ::core::ffi::c_char = str.data;
    let mut chunk: *const ::core::ffi::c_char = s;
    while (s.offset_from(str.data) as size_t) < str.size {
        if check_int as ::core::ffi::c_int != 0 && got_int.get() as ::core::ffi::c_int != 0 {
            return;
        }
        if *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
            || *s as ::core::ffi::c_int == TAB
            || *s as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
            || *s as ::core::ffi::c_int == BELL
        {
            msg_outtrans_len(
                chunk,
                s.offset_from(chunk) as ::core::ffi::c_int,
                hl_id,
                hist,
            );
            if *s as ::core::ffi::c_int != TAB && *need_clear as ::core::ffi::c_int != 0 {
                msg_clr_eos();
                *need_clear = false_0 != 0;
            }
            if *s as ::core::ffi::c_int == BELL {
                vim_beep(kOptBoFlagShell as ::core::ffi::c_int as ::core::ffi::c_uint);
            } else {
                msg_putchar_hl(*s as uint8_t as ::core::ffi::c_int, hl_id);
            }
            chunk = s.offset(1 as ::core::ffi::c_int as isize);
        }
        s = s.offset(1);
    }
    if *chunk as ::core::ffi::c_int != NUL || chunk == str.data as *const ::core::ffi::c_char {
        msg_outtrans_len(
            chunk,
            str.size.wrapping_sub(chunk.offset_from(str.data) as size_t) as ::core::ffi::c_int,
            hl_id,
            hist,
        );
    }
}
static is_multihl: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
unsafe extern "C" fn format_progress_message(
    mut hl_msg: HlMessage,
    mut msg_data: *mut MessageData,
) -> HlMessage {
    let mut updated_msg: HlMessage = HlMessage {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<HlMessageChunk>(),
    };
    if (*msg_data).title.size != 0 as size_t {
        let mut hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*msg_data).status.data.is_null() {
            hl_id = 0 as ::core::ffi::c_int;
        } else if strequal(
            (*msg_data).status.data,
            b"success\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            hl_id = syn_check_group(
                b"OkMsg\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            );
        } else if strequal(
            (*msg_data).status.data,
            b"failed\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            hl_id = syn_check_group(
                b"ErrorMsg\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            );
        } else if strequal(
            (*msg_data).status.data,
            b"running\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            hl_id = syn_check_group(
                b"MoreMsg\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            );
        } else if strequal(
            (*msg_data).status.data,
            b"cancel\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            hl_id = syn_check_group(
                b"WarningMsg\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
            );
        }
        if updated_msg.size == updated_msg.capacity {
            updated_msg.capacity = if updated_msg.capacity != 0 {
                updated_msg.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            updated_msg.items = xrealloc(
                updated_msg.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(updated_msg.capacity),
            ) as *mut HlMessageChunk;
        } else {
        };
        let c2rust_fresh9 = updated_msg.size;
        updated_msg.size = updated_msg.size.wrapping_add(1);
        *updated_msg.items.offset(c2rust_fresh9 as isize) = HlMessageChunk {
            text: copy_string((*msg_data).title, ::core::ptr::null_mut::<Arena>()),
            hl_id: hl_id,
        };
        if updated_msg.size == updated_msg.capacity {
            updated_msg.capacity = if updated_msg.capacity != 0 {
                updated_msg.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            updated_msg.items = xrealloc(
                updated_msg.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(updated_msg.capacity),
            ) as *mut HlMessageChunk;
        } else {
        };
        let c2rust_fresh10 = updated_msg.size;
        updated_msg.size = updated_msg.size.wrapping_add(1);
        *updated_msg.items.offset(c2rust_fresh10 as isize) = HlMessageChunk {
            text: cstr_to_string(b": \0".as_ptr() as *const ::core::ffi::c_char),
            hl_id: 0 as ::core::ffi::c_int,
        };
    }
    if (*msg_data).percent > 0 as Integer {
        let mut percent_buf: [::core::ffi::c_char; 10] = [0; 10];
        vim_snprintf(
            &raw mut percent_buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>(),
            b"%3ld%% \0".as_ptr() as *const ::core::ffi::c_char,
            (*msg_data).percent as ::core::ffi::c_long,
        );
        let mut percent: String_0 =
            cstr_to_string(&raw mut percent_buf as *mut ::core::ffi::c_char);
        let mut hl_id_0: ::core::ffi::c_int = syn_check_group(
            b"WarningMsg\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
        );
        if updated_msg.size == updated_msg.capacity {
            updated_msg.capacity = if updated_msg.capacity != 0 {
                updated_msg.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            updated_msg.items = xrealloc(
                updated_msg.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(updated_msg.capacity),
            ) as *mut HlMessageChunk;
        } else {
        };
        let c2rust_fresh11 = updated_msg.size;
        updated_msg.size = updated_msg.size.wrapping_add(1);
        *updated_msg.items.offset(c2rust_fresh11 as isize) = HlMessageChunk {
            text: percent,
            hl_id: hl_id_0,
        };
    }
    if updated_msg.size != 0 as size_t {
        let mut i: uint32_t = 0 as uint32_t;
        while (i as size_t) < hl_msg.size {
            if updated_msg.size == updated_msg.capacity {
                updated_msg.capacity = if updated_msg.capacity != 0 {
                    updated_msg.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                updated_msg.items = xrealloc(
                    updated_msg.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(updated_msg.capacity),
                ) as *mut HlMessageChunk;
            } else {
            };
            let c2rust_fresh12 = updated_msg.size;
            updated_msg.size = updated_msg.size.wrapping_add(1);
            *updated_msg.items.offset(c2rust_fresh12 as isize) = HlMessageChunk {
                text: copy_string(
                    (*hl_msg.items.offset(i as isize)).text,
                    ::core::ptr::null_mut::<Arena>(),
                ),
                hl_id: (*hl_msg.items.offset(i as isize)).hl_id,
            };
            i = i.wrapping_add(1);
        }
        return updated_msg;
    } else {
        return hl_msg;
    };
}
pub unsafe extern "C" fn msg_multihl(
    mut id: Object,
    mut hl_msg: HlMessage,
    mut kind: *const ::core::ffi::c_char,
    mut history: bool,
    mut err: bool,
    mut msg_data: *mut MessageData,
    mut needs_msg_clear: *mut bool,
) -> Object {
    if id.type_0 as ::core::ffi::c_uint
        == kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let c2rust_fresh8 = msg_id_next.get();
        msg_id_next.set(msg_id_next.get() + 1);
        id = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed_11 {
                integer: c2rust_fresh8,
            },
        };
    } else if id.type_0 as ::core::ffi::c_uint
        == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        && !msg_id_exists(id.data.integer as int64_t)
    {
        abort();
    }
    if strequal(kind, b"progress\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int
        != 0
        && progress_msg_target.get() & PROGRESS_TARGET_CMD == 0 as ::core::ffi::c_int
    {
        *needs_msg_clear = true_0 != 0;
        return id;
    }
    (*no_wait_return.ptr()) += 1;
    msg_start();
    msg_clr_eos();
    let mut need_clear: bool = false_0 != 0;
    let mut hl_msg_updated: bool = false_0 != 0;
    if !kind.is_null() {
        msg_ext_set_kind(kind);
    }
    msg_ext_skip_flush.set(true_0 != 0);
    msg_ext_id.set(id);
    if strequal(kind, b"progress\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int
        != 0
        && !msg_data.is_null()
    {
        let mut formated_message: HlMessage = format_progress_message(hl_msg, msg_data);
        if formated_message.items != hl_msg.items {
            *needs_msg_clear = true_0 != 0;
            hl_msg_updated = true_0 != 0;
            hl_msg = formated_message;
        }
    }
    let mut i: uint32_t = 0 as uint32_t;
    while (i as size_t) < hl_msg.size {
        let mut chunk: HlMessageChunk = *hl_msg.items.offset(i as isize);
        (*is_multihl.ptr()) += 1;
        if err {
            emsg_multiline(chunk.text.data, kind, chunk.hl_id, true_0 != 0);
        } else {
            msg_multiline(
                chunk.text,
                chunk.hl_id,
                true_0 != 0,
                false_0 != 0,
                &raw mut need_clear,
            );
        }
        '_c2rust_label: {
            if !ui_has(kUIMessages) || kind.is_null() || msg_ext_kind.get() == kind {
            } else {
                __assert_fail(
                    b"!ui_has(kUIMessages) || kind == NULL || msg_ext_kind == kind\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/message.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    416 as ::core::ffi::c_uint,
                    b"Object msg_multihl(Object, HlMessage, const char *, _Bool, _Bool, MessageData *, _Bool *)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        i = i.wrapping_add(1);
    }
    if history as ::core::ffi::c_int != 0 && hl_msg.size != 0 {
        msg_hist_add_multihl(hl_msg, false_0 != 0, msg_data);
    }
    msg_ext_skip_flush.set(false_0 != 0);
    is_multihl.set(0 as ::core::ffi::c_int);
    (*no_wait_return.ptr()) -= 1;
    msg_end();
    if hl_msg_updated as ::core::ffi::c_int != 0
        && !(history as ::core::ffi::c_int != 0 && hl_msg.size != 0)
    {
        hl_msg_free(hl_msg);
    }
    return id;
}
pub unsafe extern "C" fn msg_keep(
    mut s: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut keep: bool,
    mut multiline: bool,
) -> bool {
    static entered: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    if keep as ::core::ffi::c_int != 0 && multiline as ::core::ffi::c_int != 0 {
        abort();
    }
    if !emsg_on_display.get() && message_filtered(s) as ::core::ffi::c_int != 0 {
        return true_0 != 0;
    }
    if hl_id == 0 as ::core::ffi::c_int {
        set_vim_var_string(VV_STATUSMSG, s, -1 as ptrdiff_t);
    }
    if entered.get() >= 3 as ::core::ffi::c_int {
        return true_0 != 0;
    }
    (*entered.ptr()) += 1;
    if is_multihl.get() == 0
        && (s != keep_msg.get() as *const ::core::ffi::c_char
            || *s as ::core::ffi::c_int != '<' as ::core::ffi::c_int
                && !(*msg_hist_last.ptr()).is_null()
                && strcmp(
                    s,
                    (*(*msg_hist_last.get())
                        .msg
                        .items
                        .offset(0 as ::core::ffi::c_int as isize))
                    .text
                    .data,
                ) != 0 as ::core::ffi::c_int)
    {
        msg_hist_add(s, -1 as ::core::ffi::c_int, hl_id);
    }
    if is_multihl.get() == 0 {
        msg_start();
    }
    let mut buf: *mut ::core::ffi::c_char = msg_strtrunc(s, false_0);
    if !buf.is_null() {
        s = buf;
    }
    let mut need_clear: bool = true_0 != 0;
    if multiline {
        msg_multiline(
            cstr_as_string(s),
            hl_id,
            false_0 != 0,
            false_0 != 0,
            &raw mut need_clear,
        );
    } else {
        msg_outtrans(s, hl_id, false_0 != 0);
    }
    if need_clear {
        msg_clr_eos();
    }
    let mut retval: bool = true_0 != 0;
    if is_multihl.get() == 0 {
        retval = msg_end();
    }
    if keep as ::core::ffi::c_int != 0
        && retval as ::core::ffi::c_int != 0
        && vim_strsize(s)
            < (Rows.get() - cmdline_row.get() - 1 as ::core::ffi::c_int) * Columns.get()
                + sc_col.get()
    {
        set_keep_msg(s, 0 as ::core::ffi::c_int);
    }
    need_fileinfo.set(false_0 != 0);
    xfree(buf as *mut ::core::ffi::c_void);
    (*entered.ptr()) -= 1;
    return retval;
}
pub unsafe extern "C" fn msg_strtrunc(
    mut s: *const ::core::ffi::c_char,
    mut force: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if msg_scroll.get() == 0
        && !need_wait_return.get()
        && shortmess(SHM_TRUNCALL as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        && !exmode_active.get()
        && msg_silent.get() == 0 as ::core::ffi::c_int
        && !ui_has(kUIMessages)
        || force != 0
    {
        let mut room: ::core::ffi::c_int = 0;
        let mut len: ::core::ffi::c_int = vim_strsize(s);
        if msg_scrolled.get() != 0 as ::core::ffi::c_int {
            room = (Rows.get() - msg_row.get()) * Columns.get() - 1 as ::core::ffi::c_int;
        } else {
            room = (Rows.get() - msg_row.get() - 1 as ::core::ffi::c_int) * Columns.get()
                + sc_col.get()
                - 1 as ::core::ffi::c_int;
        }
        if len > room && room > 0 as ::core::ffi::c_int {
            len = (room + 2 as ::core::ffi::c_int) * 18 as ::core::ffi::c_int;
            buf = xmalloc(len as size_t) as *mut ::core::ffi::c_char;
            trunc_string(s, buf, room, len);
        }
    }
    return buf;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn trunc_string(
    mut s: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut room_in: ::core::ffi::c_int,
    mut buflen: ::core::ffi::c_int,
) {
    let mut room: ::core::ffi::c_int = room_in - 3 as ::core::ffi::c_int;
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut e: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    if *s as ::core::ffi::c_int == NUL {
        if buflen > 0 as ::core::ffi::c_int {
            *buf = NUL as ::core::ffi::c_char;
        }
        return;
    }
    if room_in < 3 as ::core::ffi::c_int {
        room = 0 as ::core::ffi::c_int;
    }
    let mut half: ::core::ffi::c_int = room / 2 as ::core::ffi::c_int;
    e = 0 as ::core::ffi::c_int;
    while len < half && e < buflen {
        if *s.offset(e as isize) as ::core::ffi::c_int == NUL {
            *buf.offset(e as isize) = NUL as ::core::ffi::c_char;
            return;
        }
        n = ptr2cells(s.offset(e as isize));
        if len + n > half {
            break;
        }
        len += n;
        *buf.offset(e as isize) = *s.offset(e as isize);
        n = utfc_ptr2len(s.offset(e as isize));
        loop {
            n -= 1;
            if n <= 0 as ::core::ffi::c_int {
                break;
            }
            e += 1;
            if e == buflen {
                break;
            }
            *buf.offset(e as isize) = *s.offset(e as isize);
        }
        e += 1;
    }
    i = strlen(s) as ::core::ffi::c_int;
    half = i;
    loop {
        half =
            half - utf_head_off(
                s,
                s.offset(half as isize)
                    .offset(-(1 as ::core::ffi::c_int as isize)),
            ) - 1 as ::core::ffi::c_int;
        n = ptr2cells(s.offset(half as isize));
        if len + n > room || half == 0 as ::core::ffi::c_int {
            break;
        }
        len += n;
        i = half;
    }
    if i <= e + 3 as ::core::ffi::c_int {
        if s != buf as *const ::core::ffi::c_char {
            len = strlen(s) as ::core::ffi::c_int;
            if len >= buflen {
                len = buflen - 1 as ::core::ffi::c_int;
            }
            len = len - e + 1 as ::core::ffi::c_int;
            if len < 1 as ::core::ffi::c_int {
                *buf.offset((e - 1 as ::core::ffi::c_int) as isize) = NUL as ::core::ffi::c_char;
            } else {
                memmove(
                    buf.offset(e as isize) as *mut ::core::ffi::c_void,
                    s.offset(e as isize) as *const ::core::ffi::c_void,
                    len as size_t,
                );
            }
        }
    } else if (e + 3 as ::core::ffi::c_int) < buflen {
        memmove(
            buf.offset(e as isize) as *mut ::core::ffi::c_void,
            b"...\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            3 as size_t,
        );
        len = strlen(s.offset(i as isize)) as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
        if len >= buflen - e - 3 as ::core::ffi::c_int {
            len = buflen - e - 3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        }
        memmove(
            buf.offset(e as isize)
                .offset(3 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            s.offset(i as isize) as *const ::core::ffi::c_void,
            len as size_t,
        );
        *buf.offset((e + 3 as ::core::ffi::c_int + len - 1 as ::core::ffi::c_int) as isize) =
            NUL as ::core::ffi::c_char;
    } else {
        *buf.offset((buflen - 1 as ::core::ffi::c_int) as isize) = NUL as ::core::ffi::c_char;
    };
}
pub unsafe extern "C" fn smsg(
    mut hl_id: ::core::ffi::c_int,
    mut s: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> ::core::ffi::c_int {
    let mut arglist: ::core::ffi::VaList;
    arglist = c2rust_args.clone();
    vim_vsnprintf(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        s,
        arglist,
    );
    return msg(IObuff.ptr() as *mut ::core::ffi::c_char, hl_id) as ::core::ffi::c_int;
}
pub unsafe extern "C" fn smsg_keep(
    mut hl_id: ::core::ffi::c_int,
    mut s: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> ::core::ffi::c_int {
    let mut arglist: ::core::ffi::VaList;
    arglist = c2rust_args.clone();
    vim_vsnprintf(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        s,
        arglist,
    );
    return msg_keep(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        hl_id,
        true_0 != 0,
        false_0 != 0,
    ) as ::core::ffi::c_int;
}
static last_sourcing_lnum: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
static last_sourcing_name: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub unsafe extern "C" fn reset_last_sourcing() {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        last_sourcing_name.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    last_sourcing_lnum.set(0 as ::core::ffi::c_int);
}
unsafe extern "C" fn other_sourcing_name() -> bool {
    if !(*exestack.ptr()).ga_data.is_null()
        && (*exestack.ptr()).ga_len > 0 as ::core::ffi::c_int
        && !(*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_name
        .is_null()
    {
        if !(*last_sourcing_name.ptr()).is_null() {
            return strcmp(
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name,
                last_sourcing_name.get(),
            ) != 0 as ::core::ffi::c_int;
        }
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn get_emsg_source() -> *mut ::core::ffi::c_char {
    if !(*exestack.ptr()).ga_data.is_null()
        && (*exestack.ptr()).ga_len > 0 as ::core::ffi::c_int
        && !(*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_name
        .is_null()
        && other_sourcing_name() as ::core::ffi::c_int != 0
    {
        let mut sname: *mut ::core::ffi::c_char = estack_sfile(ESTACK_NONE);
        let mut tofree: *mut ::core::ffi::c_char = sname;
        if sname.is_null() {
            sname = (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_name;
        }
        let p: *const ::core::ffi::c_char =
            gettext(b"Error in %s:\0".as_ptr() as *const ::core::ffi::c_char);
        let buf_len: size_t = strlen(sname)
            .wrapping_add(strlen(p))
            .wrapping_add(1 as size_t);
        let buf: *mut ::core::ffi::c_char = xmalloc(buf_len) as *mut ::core::ffi::c_char;
        snprintf(buf, buf_len, p, sname);
        xfree(tofree as *mut ::core::ffi::c_void);
        return buf;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn get_emsg_lnum() -> *mut ::core::ffi::c_char {
    if !(*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_name
    .is_null()
        && (other_sourcing_name() as ::core::ffi::c_int != 0
            || (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum
                != last_sourcing_lnum.get() as linenr_T)
        && (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum
            != 0 as linenr_T
    {
        let p: *const ::core::ffi::c_char =
            gettext(b"line %4d:\0".as_ptr() as *const ::core::ffi::c_char);
        let buf_len: size_t = (20 as size_t).wrapping_add(strlen(p));
        let buf: *mut ::core::ffi::c_char = xmalloc(buf_len) as *mut ::core::ffi::c_char;
        snprintf(
            buf,
            buf_len,
            p,
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum,
        );
        return buf;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn msg_source(mut hl_id: ::core::ffi::c_int) {
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if recursive.get() {
        return;
    }
    recursive.set(true_0 != 0);
    (*no_wait_return.ptr()) += 1;
    let mut p: *mut ::core::ffi::c_char = get_emsg_source();
    if !p.is_null() {
        msg_scroll.set(true_0);
        msg(p, hl_id);
        xfree(p as *mut ::core::ffi::c_void);
    }
    p = get_emsg_lnum();
    if !p.is_null() {
        msg(p, HLF_N);
        xfree(p as *mut ::core::ffi::c_void);
        last_sourcing_lnum.set(
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum as ::core::ffi::c_int,
        );
    }
    if (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_name
    .is_null()
        || other_sourcing_name() as ::core::ffi::c_int != 0
    {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            last_sourcing_name.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        if !(*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_name
        .is_null()
        {
            last_sourcing_name.set(xstrdup(
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name,
            ));
            if redirecting() == 0 {
                msg_putchar_hl('\n' as ::core::ffi::c_int, hl_id);
            }
        }
    }
    (*no_wait_return.ptr()) -= 1;
    recursive.set(false_0 != 0);
}
pub(crate) unsafe extern "C" fn emsg_not_now() -> ::core::ffi::c_int {
    if emsg_off.get() > 0 as ::core::ffi::c_int
        && vim_strchr(p_debug.get(), 'm' as ::core::ffi::c_int).is_null()
        && vim_strchr(p_debug.get(), 't' as ::core::ffi::c_int).is_null()
        || emsg_skip.get() > 0 as ::core::ffi::c_int
    {
        return true_0;
    }
    return false_0;
}
pub unsafe extern "C" fn emsg_multiline(
    mut s: *const ::core::ffi::c_char,
    mut kind: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut multiline: bool,
) -> bool {
    let mut ignore: bool = false_0 != 0;
    if emsg_not_now() != 0 {
        return true_0 != 0;
    }
    (*called_emsg.ptr()) += 1;
    let mut severe: bool = emsg_severe.get();
    emsg_severe.set(false_0 != 0);
    if emsg_off.get() == 0 || !vim_strchr(p_debug.get(), 't' as ::core::ffi::c_int).is_null() {
        if cause_errthrow(
            s,
            multiline,
            is_multihl.get() > 1 as ::core::ffi::c_int,
            severe,
            &raw mut ignore,
        ) {
            if !ignore {
                (*did_emsg.ptr()) += 1;
            }
            return true_0 != 0;
        }
        if in_assert_fails.get() as ::core::ffi::c_int != 0
            && (*emsg_assert_fails_msg.ptr()).is_null()
        {
            emsg_assert_fails_msg.set(xstrdup(s));
            emsg_assert_fails_lnum.set(
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum as ::core::ffi::c_long,
            );
            xfree(emsg_assert_fails_context.get() as *mut ::core::ffi::c_void);
            emsg_assert_fails_context.set(xstrdup(
                if (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name
                .is_null()
                {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_name as *const ::core::ffi::c_char
                },
            ));
        }
        set_vim_var_string(VV_ERRMSG, s, -1 as ptrdiff_t);
        if emsg_silent.get() != 0 as ::core::ffi::c_int {
            if !emsg_noredir.get() {
                msg_start();
                let mut p: *mut ::core::ffi::c_char = get_emsg_source();
                if !p.is_null() {
                    let p_len: size_t = strlen(p);
                    *p.offset(p_len as isize) = '\n' as ::core::ffi::c_char;
                    redir_write(p, p_len as ptrdiff_t + 1 as ptrdiff_t);
                    xfree(p as *mut ::core::ffi::c_void);
                }
                p = get_emsg_lnum();
                if !p.is_null() {
                    let p_len_0: size_t = strlen(p);
                    *p.offset(p_len_0 as isize) = '\n' as ::core::ffi::c_char;
                    redir_write(p, p_len_0 as ptrdiff_t + 1 as ptrdiff_t);
                    xfree(p as *mut ::core::ffi::c_void);
                }
                redir_write(s, strlen(s) as ptrdiff_t);
            }
            if !(*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_name
            .is_null()
                && (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum
                    != 0 as linenr_T
            {
                logmsg(
                    LOGLVL_DBG,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    b"emsg_multiline\0".as_ptr() as *const ::core::ffi::c_char,
                    845 as ::core::ffi::c_int,
                    true_0 != 0,
                    b"(:silent) %s (%s (line %d))\0".as_ptr() as *const ::core::ffi::c_char,
                    s,
                    (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_name,
                    (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_lnum,
                );
            } else {
                logmsg(
                    LOGLVL_DBG,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    b"emsg_multiline\0".as_ptr() as *const ::core::ffi::c_char,
                    847 as ::core::ffi::c_int,
                    true_0 != 0,
                    b"(:silent) %s\0".as_ptr() as *const ::core::ffi::c_char,
                    s,
                );
            }
            return true_0 != 0;
        }
        if !(*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_name
        .is_null()
            && (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum
                != 0 as linenr_T
        {
            logmsg(
                LOGLVL_INF,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"emsg_multiline\0".as_ptr() as *const ::core::ffi::c_char,
                855 as ::core::ffi::c_int,
                true_0 != 0,
                b"%s (%s (line %d))\0".as_ptr() as *const ::core::ffi::c_char,
                s,
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name,
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum,
            );
        } else {
            logmsg(
                LOGLVL_INF,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"emsg_multiline\0".as_ptr() as *const ::core::ffi::c_char,
                857 as ::core::ffi::c_int,
                true_0 != 0,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                s,
            );
        }
        ex_exitval.set(1 as ::core::ffi::c_int);
        msg_silent.set(0 as ::core::ffi::c_int);
        cmd_silent.set(false_0 != 0);
        if global_busy.get() != 0 {
            (*global_busy.ptr()) += 1;
        }
        if p_eb.get() != 0 {
            beep_flush();
        } else {
            flush_buffers(FLUSH_MINIMAL);
        }
        (*did_emsg.ptr()) += 1;
    }
    emsg_on_display.set(true_0 != 0);
    if msg_scrolled.get() != 0 as ::core::ffi::c_int {
        need_wait_return.set(true_0 != 0);
    }
    msg_ext_set_kind(kind);
    msg_scroll.set(true_0);
    let mut save_msg_skip_flush: bool = msg_ext_skip_flush.get();
    msg_ext_skip_flush.set(true_0 != 0);
    msg_source(hl_id);
    msg_nowait.set(false_0 != 0);
    let mut rv: ::core::ffi::c_int =
        msg_keep(s, hl_id, false_0 != 0, multiline) as ::core::ffi::c_int;
    msg_ext_skip_flush.set(save_msg_skip_flush);
    return rv != 0;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emsg(mut s: *const ::core::ffi::c_char) -> bool {
    return emsg_multiline(
        s,
        b"emsg\0".as_ptr() as *const ::core::ffi::c_char,
        HLF_E,
        false_0 != 0,
    );
}
pub unsafe extern "C" fn emsg_invreg(mut name: ::core::ffi::c_int) {
    semsg(
        gettext(b"E354: Invalid register name: '%s'\0".as_ptr() as *const ::core::ffi::c_char),
        transchar_buf(::core::ptr::null::<buf_T>(), name),
    );
}
pub unsafe extern "C" fn semsg(fmt: *const ::core::ffi::c_char, mut c2rust_args: ...) -> bool {
    let mut ret: bool = false;
    let mut ap: ::core::ffi::VaList;
    ap = c2rust_args.clone();
    ret = semsgv(fmt, ap);
    return ret;
}
pub unsafe extern "C" fn semsg_multiline(
    mut kind: *const ::core::ffi::c_char,
    fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> bool {
    let mut ret: bool = false;
    let mut ap: ::core::ffi::VaList;
    static errbuf: GlobalCell<[::core::ffi::c_char; 8192]> = GlobalCell::new([0; 8192]);
    if emsg_not_now() != 0 {
        return true_0 != 0;
    }
    ap = c2rust_args.clone();
    vim_vsnprintf(
        errbuf.ptr() as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8192]>(),
        fmt,
        ap,
    );
    ret = emsg_multiline(
        errbuf.ptr() as *mut ::core::ffi::c_char,
        kind,
        HLF_E,
        true_0 != 0,
    );
    return ret;
}
unsafe extern "C" fn semsgv(
    mut fmt: *const ::core::ffi::c_char,
    mut ap: ::core::ffi::VaList,
) -> bool {
    static errbuf: GlobalCell<[::core::ffi::c_char; 1025]> = GlobalCell::new([0; 1025]);
    if emsg_not_now() != 0 {
        return true_0 != 0;
    }
    vim_vsnprintf(
        errbuf.ptr() as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
        fmt,
        ap,
    );
    return emsg(errbuf.ptr() as *mut ::core::ffi::c_char);
}
pub unsafe extern "C" fn iemsg(mut s: *const ::core::ffi::c_char) {
    if emsg_not_now() != 0 {
        return;
    }
    emsg(s);
}
pub unsafe extern "C" fn siemsg(mut s: *const ::core::ffi::c_char, mut c2rust_args: ...) {
    if emsg_not_now() != 0 {
        return;
    }
    let mut ap: ::core::ffi::VaList;
    ap = c2rust_args.clone();
    semsgv(s, ap);
}
pub unsafe extern "C" fn internal_error(mut where_0: *const ::core::ffi::c_char) {
    siemsg(
        gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
        where_0,
    );
}
unsafe extern "C" fn msg_semsg_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut s: *mut ::core::ffi::c_char =
        *argv.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
    emsg(s);
    xfree(s as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn msg_schedule_semsg(fmt: *const ::core::ffi::c_char, mut c2rust_args: ...) {
    let mut ap: ::core::ffi::VaList;
    ap = c2rust_args.clone();
    vim_vsnprintf(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        fmt,
        ap,
    );
    let mut s: *mut ::core::ffi::c_char = xstrdup(IObuff.ptr() as *mut ::core::ffi::c_char);
    loop_schedule_deferred(
        main_loop.ptr(),
        Event {
            handler: Some(
                msg_semsg_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                s as *mut ::core::ffi::c_void,
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ],
        },
    );
}
unsafe extern "C" fn msg_semsg_multiline_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut s: *mut ::core::ffi::c_char =
        *argv.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
    emsg_multiline(
        s,
        b"emsg\0".as_ptr() as *const ::core::ffi::c_char,
        HLF_E,
        true_0 != 0,
    );
    xfree(s as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn msg_schedule_semsg_multiline(
    fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) {
    let mut ap: ::core::ffi::VaList;
    ap = c2rust_args.clone();
    vim_vsnprintf(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        fmt,
        ap,
    );
    let mut s: *mut ::core::ffi::c_char = xstrdup(IObuff.ptr() as *mut ::core::ffi::c_char);
    loop_schedule_deferred(
        main_loop.ptr(),
        Event {
            handler: Some(
                msg_semsg_multiline_event
                    as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                s as *mut ::core::ffi::c_void,
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ],
        },
    );
}
pub unsafe extern "C" fn msg_trunc(
    mut s: *mut ::core::ffi::c_char,
    mut force: bool,
    mut hl_id: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    msg_hist_add(s, -1 as ::core::ffi::c_int, hl_id);
    let mut ts: *mut ::core::ffi::c_char = msg_may_trunc(force, s);
    msg_hist_off.set(true_0 != 0);
    let mut n: bool = msg(ts, hl_id);
    msg_hist_off.set(false_0 != 0);
    if n {
        return ts;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn msg_may_trunc(
    mut force: bool,
    mut s: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if ui_has(kUIMessages) {
        return s;
    }
    let mut room: ::core::ffi::c_int =
        (Rows.get() - cmdline_row.get() - 1 as ::core::ffi::c_int) * Columns.get() + sc_col.get()
            - 1 as ::core::ffi::c_int;
    if room > 0 as ::core::ffi::c_int
        && (force as ::core::ffi::c_int != 0
            || shortmess(SHM_TRUNC as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                && !exmode_active.get())
        && strlen(s) as ::core::ffi::c_int - room > 0 as ::core::ffi::c_int
    {
        let mut size: ::core::ffi::c_int = vim_strsize(s);
        if size <= room {
            return s;
        }
        let mut n: ::core::ffi::c_int = 0;
        n = 0 as ::core::ffi::c_int;
        while size >= room {
            size -= utf_ptr2cells(s.offset(n as isize));
            n += utfc_ptr2len(s.offset(n as isize));
        }
        n -= 1;
        s = s.offset(n as isize);
        *s = '<' as ::core::ffi::c_char;
    }
    return s;
}
pub unsafe extern "C" fn msg_progress(
    mut s: *mut ::core::ffi::c_char,
    mut id: *mut ::core::ffi::c_char,
    mut status: *mut ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut hist: bool,
    mut trunc: bool,
) -> *mut ::core::ffi::c_char {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut opts: KeyDict_echo_opts = KeyDict_echo_opts {
        is_set__echo_opts_: 0,
        err: false,
        verbose: false,
        _truncate: false,
        kind: cstr_as_string(b"progress\0".as_ptr() as *const ::core::ffi::c_char),
        id: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_11 {
                string: cstr_as_string(id),
            },
        },
        title: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        status: cstr_as_string(status),
        percent: 0,
        source: cstr_as_string(b"nvim\0".as_ptr() as *const ::core::ffi::c_char),
        data: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
    };
    if hist as ::core::ffi::c_int != 0 && (!trunc || ui_has(kUIMessages) as ::core::ffi::c_int != 0)
    {
        msg_hist_add(s, -1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    }
    if trunc {
        s = msg_may_trunc(false_0 != 0, s);
    }
    let mut chunk: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut chunk__items: [Object; 2] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_11 { boolean: false },
    }; 2];
    chunk.capacity = 2 as size_t;
    chunk.items = &raw mut chunk__items as *mut Object;
    let mut chunks: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut chunks__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_11 { boolean: false },
    }; 1];
    chunks.capacity = 1 as size_t;
    chunks.items = &raw mut chunks__items as *mut Object;
    let c2rust_fresh13 = chunk.size;
    chunk.size = chunk.size.wrapping_add(1);
    *chunk.items.offset(c2rust_fresh13 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_11 {
            string: cstr_as_string(s),
        },
    };
    let c2rust_fresh14 = chunk.size;
    chunk.size = chunk.size.wrapping_add(1);
    *chunk.items.offset(c2rust_fresh14 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_11 {
            integer: hl_id as Integer,
        },
    };
    let c2rust_fresh15 = chunks.size;
    chunks.size = chunks.size.wrapping_add(1);
    *chunks.items.offset(c2rust_fresh15 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_11 { array: chunk },
    };
    nvim_echo(chunks, false_0 != 0, &raw mut opts, &raw mut err);
    ui_flush();
    return s;
}
pub unsafe extern "C" fn hl_msg_free(mut hl_msg: HlMessage) {
    let mut i: size_t = 0 as size_t;
    while i < hl_msg.size {
        xfree((*hl_msg.items.offset(i as isize)).text.data as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    xfree(hl_msg.items as *mut ::core::ffi::c_void);
    hl_msg.capacity = 0 as size_t;
    hl_msg.size = hl_msg.capacity;
    hl_msg.items = ::core::ptr::null_mut::<HlMessageChunk>();
}
unsafe extern "C" fn msg_hist_add(
    mut s: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
) {
    let mut text: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: if len < 0 as ::core::ffi::c_int {
            strlen(s)
        } else {
            len as size_t
        },
    };
    while text.size > 0 as size_t && *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
        text.size = text.size.wrapping_sub(1);
        s = s.offset(1);
    }
    while text.size > 0 as size_t
        && *s.offset(text.size.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            == '\n' as ::core::ffi::c_int
    {
        text.size = text.size.wrapping_sub(1);
    }
    if text.size == 0 as size_t {
        return;
    }
    text.data = xmemdupz(s as *const ::core::ffi::c_void, text.size) as *mut ::core::ffi::c_char;
    let mut msg_0: HlMessage = HlMessage {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<HlMessageChunk>(),
    };
    if msg_0.size == msg_0.capacity {
        msg_0.capacity = if msg_0.capacity != 0 {
            msg_0.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        msg_0.items = xrealloc(
            msg_0.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(msg_0.capacity),
        ) as *mut HlMessageChunk;
    } else {
    };
    let c2rust_fresh7 = msg_0.size;
    msg_0.size = msg_0.size.wrapping_add(1);
    *msg_0.items.offset(c2rust_fresh7 as isize) = HlMessageChunk {
        text: text,
        hl_id: hl_id,
    };
    msg_hist_add_multihl(msg_0, false_0 != 0, ::core::ptr::null_mut::<MessageData>());
}
static do_clear_hist_temp: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
pub unsafe extern "C" fn do_autocmd_progress(
    mut msg_id: Object,
    mut msg_0: HlMessage,
    mut msg_data: *mut MessageData,
) {
    if !has_event(EVENT_PROGRESS) {
        return;
    }
    let mut data: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut data__items: [KeyValuePair; 7] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_11 { boolean: false },
        },
    }; 7];
    data.capacity = 7 as size_t;
    data.items = &raw mut data__items as *mut KeyValuePair;
    let mut messages: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut i: size_t = 0 as size_t;
    while i < msg_0.size {
        if messages.size == messages.capacity {
            messages.capacity = if messages.capacity != 0 {
                messages.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            messages.items = xrealloc(
                messages.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Object>().wrapping_mul(messages.capacity),
            ) as *mut Object;
        } else {
        };
        let c2rust_fresh16 = messages.size;
        messages.size = messages.size.wrapping_add(1);
        *messages.items.offset(c2rust_fresh16 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_11 {
                string: (*msg_0.items.offset(i as isize)).text,
            },
        };
        i = i.wrapping_add(1);
    }
    let c2rust_fresh17 = data.size;
    data.size = data.size.wrapping_add(1);
    *data.items.offset(c2rust_fresh17 as isize) = key_value_pair {
        key: cstr_as_string(b"id\0".as_ptr() as *const ::core::ffi::c_char),
        value: msg_id,
    };
    let c2rust_fresh18 = data.size;
    data.size = data.size.wrapping_add(1);
    *data.items.offset(c2rust_fresh18 as isize) = key_value_pair {
        key: cstr_as_string(b"text\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_11 { array: messages },
        },
    };
    if !msg_data.is_null() {
        let c2rust_fresh19 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh19 as isize) = key_value_pair {
            key: cstr_as_string(b"percent\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_11 {
                    integer: (*msg_data).percent,
                },
            },
        };
        let c2rust_fresh20 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh20 as isize) = key_value_pair {
            key: cstr_as_string(b"source\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_11 {
                    string: (*msg_data).source,
                },
            },
        };
        let c2rust_fresh21 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh21 as isize) = key_value_pair {
            key: cstr_as_string(b"status\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_11 {
                    string: (*msg_data).status,
                },
            },
        };
        let c2rust_fresh22 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh22 as isize) = key_value_pair {
            key: cstr_as_string(b"title\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_11 {
                    string: (*msg_data).title,
                },
            },
        };
        let c2rust_fresh23 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh23 as isize) = key_value_pair {
            key: cstr_as_string(b"data\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed_11 {
                    dict: (*msg_data).data,
                },
            },
        };
    }
    let mut c2rust_lvalue: Object = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed_11 { dict: data },
    };
    apply_autocmds_group(
        EVENT_PROGRESS,
        (if !msg_data.is_null() && (*msg_data).source.size > 0 as size_t {
            (*msg_data).source.data as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0 != 0,
        AUGROUP_ALL as ::core::ffi::c_int,
        ::core::ptr::null_mut::<buf_T>(),
        ::core::ptr::null_mut::<exarg_T>(),
        &raw mut c2rust_lvalue,
    );
    xfree(messages.items as *mut ::core::ffi::c_void);
    messages.capacity = 0 as size_t;
    messages.size = messages.capacity;
    messages.items = ::core::ptr::null_mut::<Object>();
}
unsafe extern "C" fn msg_hist_add_multihl(
    mut msg_0: HlMessage,
    mut temp: bool,
    mut _msg_data: *mut MessageData,
) {
    if do_clear_hist_temp.get() {
        msg_hist_clear_temp();
        do_clear_hist_temp.set(false_0 != 0);
    }
    if msg_hist_off.get() as ::core::ffi::c_int != 0 || msg_silent.get() != 0 as ::core::ffi::c_int
    {
        hl_msg_free(msg_0);
        return;
    }
    let mut entry: *mut MessageHistoryEntry =
        xmalloc(::core::mem::size_of::<MessageHistoryEntry>()) as *mut MessageHistoryEntry;
    (*entry).msg = msg_0;
    (*entry).temp = temp;
    (*entry).kind = if !(*msg_ext_kind.ptr()).is_null() {
        xstrdup(msg_ext_kind.get())
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    (*entry).prev = msg_hist_last.get() as *mut msg_hist;
    (*entry).next = ::core::ptr::null_mut::<msg_hist>();
    (*entry).append = msg_ext_append.get();
    if (*msg_hist_first.ptr()).is_null() {
        msg_hist_first.set(entry);
    }
    if !(*msg_hist_last.ptr()).is_null() {
        (*msg_hist_last.get()).next = entry as *mut msg_hist;
    }
    if (*msg_hist_temp.ptr()).is_null() {
        msg_hist_temp.set(entry);
    }
    (*msg_hist_len.ptr()) += !temp as ::core::ffi::c_int;
    msg_hist_last.set(entry);
    msg_ext_history.set(true_0 != 0);
    msg_hist_clear(msg_hist_max.get());
}
unsafe extern "C" fn msg_hist_free_msg(mut entry: *mut MessageHistoryEntry) {
    if (*entry).next.is_null() {
        msg_hist_last.set((*entry).prev as *mut MessageHistoryEntry);
    } else {
        (*(*entry).next).prev = (*entry).prev;
    }
    if (*entry).prev.is_null() {
        msg_hist_first.set((*entry).next as *mut MessageHistoryEntry);
    } else {
        (*(*entry).prev).next = (*entry).next;
    }
    if entry == msg_hist_temp.get() {
        msg_hist_temp.set((*entry).next as *mut MessageHistoryEntry);
    }
    hl_msg_free((*entry).msg);
    xfree((*entry).kind as *mut ::core::ffi::c_void);
    xfree(entry as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn msg_hist_clear(mut keep: ::core::ffi::c_int) {
    while msg_hist_len.get() > keep
        || keep == 0 as ::core::ffi::c_int && !(*msg_hist_first.ptr()).is_null()
    {
        (*msg_hist_len.ptr()) -= !(*msg_hist_first.get()).temp as ::core::ffi::c_int;
        msg_hist_free_msg(msg_hist_first.get());
    }
}
pub unsafe extern "C" fn msg_hist_clear_temp() {
    while !(*msg_hist_temp.ptr()).is_null() {
        let mut next: *mut MessageHistoryEntry =
            (*msg_hist_temp.get()).next as *mut MessageHistoryEntry;
        if (*msg_hist_temp.get()).temp {
            msg_hist_free_msg(msg_hist_temp.get());
        }
        msg_hist_temp.set(next);
    }
}
pub unsafe extern "C" fn messagesopt_changed() -> ::core::ffi::c_int {
    let mut messages_flags_new: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut messages_wait_new: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut messages_history_new: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut progress_target_flag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = p_mopt.get();
    while *p as ::core::ffi::c_int != NUL {
        if strnequal(
            p,
            b"hit-enter\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        ) {
            p = p.offset(
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize)
                    as isize,
            );
            messages_flags_new |= kOptMoptFlagHitEnter as ::core::ffi::c_int;
        } else if strnequal(
            p,
            b"wait:\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        ) as ::core::ffi::c_int
            != 0
            && ascii_isdigit(*p.offset(
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                    as isize,
            ) as ::core::ffi::c_int) as ::core::ffi::c_int
                != 0
        {
            p = p.offset(
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                    as isize,
            );
            messages_wait_new = getdigits_int(&raw mut p, false_0 != 0, INT_MAX);
            messages_flags_new |= kOptMoptFlagWait as ::core::ffi::c_int;
        } else if strnequal(
            p,
            b"history:\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        ) as ::core::ffi::c_int
            != 0
            && ascii_isdigit(*p.offset(
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize)
                    as isize,
            ) as ::core::ffi::c_int) as ::core::ffi::c_int
                != 0
        {
            p = p.offset(
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize)
                    as isize,
            );
            messages_history_new = getdigits_int(&raw mut p, false_0 != 0, INT_MAX);
            messages_flags_new |= kOptMoptFlagHistory as ::core::ffi::c_int;
        } else if strnequal(
            p,
            b"progress:\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        ) {
            p = p.offset(
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize)
                    as isize,
            );
            messages_flags_new |= kOptMoptFlagProgress as ::core::ffi::c_int;
            if *p as ::core::ffi::c_int == 'c' as ::core::ffi::c_int {
                progress_target_flag |= PROGRESS_TARGET_CMD;
                p = p.offset(1);
            }
        }
        if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int && *p as ::core::ffi::c_int != NUL
        {
            return FAIL;
        }
        if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            p = p.offset(1);
        }
    }
    if messages_flags_new
        & (kOptMoptFlagHitEnter as ::core::ffi::c_int | kOptMoptFlagWait as ::core::ffi::c_int)
        == 0
    {
        return FAIL;
    }
    if messages_flags_new & kOptMoptFlagHistory as ::core::ffi::c_int == 0 {
        return FAIL;
    }
    '_c2rust_label: {
        if messages_history_new >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"messages_history_new >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1322 as ::core::ffi::c_uint,
                b"int messagesopt_changed(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if messages_history_new > 10000 as ::core::ffi::c_int {
        return FAIL;
    }
    '_c2rust_label_0: {
        if messages_wait_new >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"messages_wait_new >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1328 as ::core::ffi::c_uint,
                b"int messagesopt_changed(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if messages_wait_new > 10000 as ::core::ffi::c_int {
        return FAIL;
    }
    msg_flags.set(messages_flags_new);
    msg_wait.set(messages_wait_new);
    progress_msg_target.set(progress_target_flag);
    msg_hist_max.set(messages_history_new);
    msg_hist_clear(msg_hist_max.get());
    return OK;
}
pub unsafe fn ex_messages(mut eap: *mut exarg_T) {
    if strcmp(
        (*eap).arg,
        b"clear\0".as_ptr() as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        msg_hist_clear(if (*eap).addr_count != 0 {
            (*eap).line2 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
        return;
    }
    if *(*eap).arg as ::core::ffi::c_int != NUL {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut entries: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut p: *mut MessageHistoryEntry = if (*eap).skip != 0 {
        msg_hist_temp.get()
    } else {
        msg_hist_first.get()
    };
    let mut skip: ::core::ffi::c_int = if (*eap).addr_count != 0 {
        msg_hist_len.get() - (*eap).line2 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    while !p.is_null() {
        if !((*p).temp as ::core::ffi::c_int != 0 && (*eap).skip == 0 || {
            let c2rust_fresh24 = skip;
            skip = skip - 1;
            c2rust_fresh24 > 0 as ::core::ffi::c_int
        }) {
            if ui_has(kUIMessages) as ::core::ffi::c_int != 0 && msg_silent.get() == 0 {
                let mut entry: Array = Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
                if entry.size == entry.capacity {
                    entry.capacity = if entry.capacity != 0 {
                        entry.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    entry.items = xrealloc(
                        entry.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<Object>().wrapping_mul(entry.capacity),
                    ) as *mut Object;
                } else {
                };
                let c2rust_fresh25 = entry.size;
                entry.size = entry.size.wrapping_add(1);
                *entry.items.offset(c2rust_fresh25 as isize) = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_11 {
                        string: cstr_to_string((*p).kind),
                    },
                };
                let mut content: Array = Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
                let mut i: uint32_t = 0 as uint32_t;
                while (i as size_t) < (*p).msg.size {
                    let mut chunk: HlMessageChunk = *(*p).msg.items.offset(i as isize);
                    let mut content_entry: Array = Array {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<Object>(),
                    };
                    if content_entry.size == content_entry.capacity {
                        content_entry.capacity = if content_entry.capacity != 0 {
                            content_entry.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        content_entry.items = xrealloc(
                            content_entry.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul(content_entry.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh26 = content_entry.size;
                    content_entry.size = content_entry.size.wrapping_add(1);
                    *content_entry.items.offset(c2rust_fresh26 as isize) = object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed_11 {
                            integer: (if chunk.hl_id != 0 {
                                syn_id2attr(chunk.hl_id)
                            } else {
                                0 as ::core::ffi::c_int
                            }) as Integer,
                        },
                    };
                    if content_entry.size == content_entry.capacity {
                        content_entry.capacity = if content_entry.capacity != 0 {
                            content_entry.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        content_entry.items = xrealloc(
                            content_entry.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul(content_entry.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh27 = content_entry.size;
                    content_entry.size = content_entry.size.wrapping_add(1);
                    *content_entry.items.offset(c2rust_fresh27 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_11 {
                            string: copy_string(chunk.text, ::core::ptr::null_mut::<Arena>()),
                        },
                    };
                    if content_entry.size == content_entry.capacity {
                        content_entry.capacity = if content_entry.capacity != 0 {
                            content_entry.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        content_entry.items = xrealloc(
                            content_entry.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul(content_entry.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh28 = content_entry.size;
                    content_entry.size = content_entry.size.wrapping_add(1);
                    *content_entry.items.offset(c2rust_fresh28 as isize) = object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed_11 {
                            integer: chunk.hl_id as Integer,
                        },
                    };
                    if content.size == content.capacity {
                        content.capacity = if content.capacity != 0 {
                            content.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        content.items = xrealloc(
                            content.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul(content.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh29 = content.size;
                    content.size = content.size.wrapping_add(1);
                    *content.items.offset(c2rust_fresh29 as isize) = object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed_11 {
                            array: content_entry,
                        },
                    };
                    i = i.wrapping_add(1);
                }
                if entry.size == entry.capacity {
                    entry.capacity = if entry.capacity != 0 {
                        entry.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    entry.items = xrealloc(
                        entry.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<Object>().wrapping_mul(entry.capacity),
                    ) as *mut Object;
                } else {
                };
                let c2rust_fresh30 = entry.size;
                entry.size = entry.size.wrapping_add(1);
                *entry.items.offset(c2rust_fresh30 as isize) = object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed_11 { array: content },
                };
                if entry.size == entry.capacity {
                    entry.capacity = if entry.capacity != 0 {
                        entry.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    entry.items = xrealloc(
                        entry.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<Object>().wrapping_mul(entry.capacity),
                    ) as *mut Object;
                } else {
                };
                let c2rust_fresh31 = entry.size;
                entry.size = entry.size.wrapping_add(1);
                *entry.items.offset(c2rust_fresh31 as isize) = object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed_11 {
                        boolean: (*p).append,
                    },
                };
                if entries.size == entries.capacity {
                    entries.capacity = if entries.capacity != 0 {
                        entries.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    entries.items = xrealloc(
                        entries.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<Object>().wrapping_mul(entries.capacity),
                    ) as *mut Object;
                } else {
                };
                let c2rust_fresh32 = entries.size;
                entries.size = entries.size.wrapping_add(1);
                *entries.items.offset(c2rust_fresh32 as isize) = object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed_11 { array: entry },
                };
            }
            if redirecting() != 0 || !ui_has(kUIMessages) {
                (*msg_silent.ptr()) += ui_has(kUIMessages) as ::core::ffi::c_int;
                let mut needs_clear: bool = false_0 != 0;
                msg_multihl(
                    object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed_11 { boolean: false },
                    },
                    (*p).msg,
                    (*p).kind,
                    false_0 != 0,
                    false_0 != 0,
                    ::core::ptr::null_mut::<MessageData>(),
                    &raw mut needs_clear,
                );
                (*msg_silent.ptr()) -= ui_has(kUIMessages) as ::core::ffi::c_int;
            }
        }
        p = (*p).next as *mut MessageHistoryEntry;
    }
    if entries.size > 0 as size_t {
        ui_call_msg_history_show(entries, (*eap).skip != 0 as ::core::ffi::c_int);
        api_free_array(entries);
    }
}
pub unsafe extern "C" fn msg_end_prompt() {
    need_wait_return.set(false_0 != 0);
    emsg_on_display.set(false_0 != 0);
    cmdline_row.set(msg_row.get());
    msg_col.set(0 as ::core::ffi::c_int);
    msg_clr_eos();
    lines_left.set(-1 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn wait_return(mut redraw: ::core::ffi::c_int) {
    let mut c: ::core::ffi::c_int = 0;
    let mut had_got_int: ::core::ffi::c_int = 0;
    let mut save_scriptout: *mut FILE = ::core::ptr::null_mut::<FILE>();
    if redraw == true_0 {
        redraw_all_later(UPD_NOT_VALID);
    }
    if ui_has(kUIMessages) {
        prompt_for_input(
            b"Press any key to continue\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            HLF_M,
            true_0 != 0,
            ::core::ptr::null_mut::<bool>(),
        );
        return;
    }
    if msg_silent.get() != 0 as ::core::ffi::c_int {
        return;
    }
    if headless_mode.get() as ::core::ffi::c_int != 0 && ui_active() == 0 {
        return;
    }
    if vgetc_busy.get() > 0 as ::core::ffi::c_int {
        return;
    }
    need_wait_return.set(true_0 != 0);
    if no_wait_return.get() != 0 {
        if !exmode_active.get() {
            cmdline_row.set(msg_row.get());
        }
        return;
    }
    redir_off.set(true_0 != 0);
    let mut oldState: ::core::ffi::c_int = State.get();
    if quit_more.get() {
        c = CAR;
        quit_more.set(false_0 != 0);
        got_int.set(false_0 != 0);
    } else if exmode_active.get() {
        msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        c = CAR;
        got_int.set(false_0 != 0);
    } else if !stuff_empty() {
        c = CAR;
    } else {
        State.set(MODE_HITRETURN);
        setmouse();
        cmdline_row.set(msg_row.get());
        if need_check_timestamps.get() {
            check_timestamps(false_0);
        }
        if p_ch.get() == 0 as OptInt && !ui_has(kUIMessages) && msg_scrolled.get() == 0 {
            msg_grid_validate();
            msg_scroll_up(false_0 != 0, true_0 != 0);
            (*msg_scrolled.ptr()) += 1;
            cmdline_row.set(Rows.get() - 1 as ::core::ffi::c_int);
        }
        if msg_flags.get() & kOptMoptFlagHitEnter as ::core::ffi::c_int != 0 {
            hit_return_msg(true_0 != 0);
            loop {
                had_got_int = got_int.get() as ::core::ffi::c_int;
                (*no_mapping.ptr()) += 1;
                (*allow_keys.ptr()) += 1;
                let save_reg_recording: ::core::ffi::c_int = reg_recording.get();
                save_scriptout = scriptout.get();
                reg_recording.set(0 as ::core::ffi::c_int);
                scriptout.set(::core::ptr::null_mut::<FILE>());
                c = safe_vgetc();
                if had_got_int != 0 && global_busy.get() == 0 {
                    got_int.set(false_0 != 0);
                }
                (*no_mapping.ptr()) -= 1;
                (*allow_keys.ptr()) -= 1;
                reg_recording.set(save_reg_recording);
                scriptout.set(save_scriptout);
                if p_more.get() != 0 {
                    if c == 'b' as ::core::ffi::c_int
                        || c == Ctrl_B
                        || c == 'k' as ::core::ffi::c_int
                        || c == 'u' as ::core::ffi::c_int
                        || c == 'g' as ::core::ffi::c_int
                        || c == K_UP
                        || c == K_PAGEUP
                    {
                        if msg_scrolled.get() > Rows.get() {
                            do_more_prompt(c);
                        } else {
                            msg_didout.set(false_0 != 0);
                            c = -(253 as ::core::ffi::c_int
                                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
                            msg_col.set(0 as ::core::ffi::c_int);
                        }
                        if quit_more.get() {
                            c = CAR;
                            quit_more.set(false_0 != 0);
                            got_int.set(false_0 != 0);
                        } else if c
                            != -(253 as ::core::ffi::c_int
                                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                        {
                            c = -(253 as ::core::ffi::c_int
                                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
                            hit_return_msg(false_0 != 0);
                        }
                    } else if msg_scrolled.get() > Rows.get() - 2 as ::core::ffi::c_int
                        && (c == 'j' as ::core::ffi::c_int
                            || c == 'd' as ::core::ffi::c_int
                            || c == 'f' as ::core::ffi::c_int
                            || c == Ctrl_F
                            || c == K_DOWN
                            || c == K_PAGEDOWN)
                    {
                        c = -(253 as ::core::ffi::c_int
                            + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
                    }
                }
                if !(had_got_int != 0 && c == Ctrl_C
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_LEFTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_LEFTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_MIDDLEDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_MIDDLERELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_RIGHTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_RIGHTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_MOUSELEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_MOUSERIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)))
                {
                    break;
                }
            }
            os_breakcheck();
            if c == -(253 as ::core::ffi::c_int
                + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || c == -(253 as ::core::ffi::c_int
                    + ((KE_MIDDLEMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || c == -(253 as ::core::ffi::c_int
                    + ((KE_RIGHTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || c == -(253 as ::core::ffi::c_int
                    + ((KE_X1MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || c == -(253 as ::core::ffi::c_int
                    + ((KE_X2MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                jump_to_mouse(
                    MOUSE_SETPOS as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<bool>(),
                    0 as ::core::ffi::c_int,
                );
            } else if vim_strchr(b"\r\n \0".as_ptr() as *const ::core::ffi::c_char, c).is_null()
                && c != Ctrl_C
                && c != 'q' as ::core::ffi::c_int
            {
                ins_char_typebuf(vgetc_char.get(), vgetc_mod_mask.get(), true_0 != 0);
                do_redraw.set(true_0 != 0);
            }
        } else {
            c = CAR;
            do_sleep(msg_wait.get() as int64_t, true_0 != 0);
        }
    }
    redir_off.set(false_0 != 0);
    if c == ':' as ::core::ffi::c_int
        || c == '?' as ::core::ffi::c_int
        || c == '/' as ::core::ffi::c_int
    {
        if !exmode_active.get() {
            cmdline_row.set(msg_row.get());
        }
        skip_redraw.set(true_0 != 0);
        do_redraw.set(false_0 != 0);
    }
    let mut tmpState: ::core::ffi::c_int = State.get();
    State.set(oldState);
    setmouse();
    msg_check();
    need_wait_return.set(false_0 != 0);
    did_wait_return.set(true_0 != 0);
    emsg_on_display.set(false_0 != 0);
    lines_left.set(-1 as ::core::ffi::c_int);
    reset_last_sourcing();
    if !(*keep_msg.ptr()).is_null()
        && vim_strsize(keep_msg.get())
            >= (Rows.get() - cmdline_row.get() - 1 as ::core::ffi::c_int) * Columns.get()
                + sc_col.get()
    {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            keep_msg.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
    }
    if tmpState == MODE_SETWSIZE {
        ui_refresh();
    } else if !skip_redraw.get() {
        if redraw == true_0
            || msg_scrolled.get() != 0 as ::core::ffi::c_int && redraw != -1 as ::core::ffi::c_int
        {
            redraw_later(curwin.get(), UPD_VALID);
        }
    }
}
unsafe extern "C" fn hit_return_msg(mut newline_sb: bool) {
    let mut save_p_more: ::core::ffi::c_int = p_more.get();
    if !newline_sb {
        p_more.set(false_0);
    }
    if msg_didout.get() {
        msg_putchar('\n' as ::core::ffi::c_int);
    }
    p_more.set(false_0);
    if got_int.get() {
        msg_puts(gettext(
            b"Interrupt: \0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
    msg_puts_hl(
        gettext(b"Press ENTER or type command to continue\0".as_ptr() as *const ::core::ffi::c_char),
        HLF_R,
        false_0 != 0,
    );
    if msg_use_printf() == 0 {
        msg_clr_eos();
    }
    p_more.set(save_p_more);
}
pub unsafe extern "C" fn set_keep_msg(
    mut s: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
) {
    if ui_has(kUIMessages) {
        return;
    }
    xfree(keep_msg.get() as *mut ::core::ffi::c_void);
    if !s.is_null() && msg_silent.get() == 0 as ::core::ffi::c_int {
        keep_msg.set(xstrdup(s));
    } else {
        keep_msg.set(::core::ptr::null_mut::<::core::ffi::c_char>());
    }
    keep_msg_more.set(false_0 != 0);
    keep_msg_hl_id.set(hl_id);
}
pub unsafe extern "C" fn messaging() -> bool {
    return !(p_lz.get() != 0 && char_avail() as ::core::ffi::c_int != 0 && !KeyTyped.get())
        && (p_ch.get() > 0 as OptInt || ui_has(kUIMessages) as ::core::ffi::c_int != 0);
}
pub unsafe extern "C" fn msgmore(mut n: ::core::ffi::c_int) {
    let mut pn: ::core::ffi::c_int = 0;
    if global_busy.get() != 0 || !messaging() {
        return;
    }
    if !(*keep_msg.ptr()).is_null() && !keep_msg_more.get() {
        return;
    }
    pn = abs(n);
    if pn as OptInt > p_report.get() {
        if n > 0 as ::core::ffi::c_int {
            vim_snprintf(
                msg_buf.ptr() as *mut ::core::ffi::c_char,
                MSG_BUF_LEN as size_t,
                ngettext(
                    b"%d more line\0".as_ptr() as *const ::core::ffi::c_char,
                    b"%d more lines\0".as_ptr() as *const ::core::ffi::c_char,
                    pn as ::core::ffi::c_ulong,
                ),
                pn,
            );
        } else {
            vim_snprintf(
                msg_buf.ptr() as *mut ::core::ffi::c_char,
                MSG_BUF_LEN as size_t,
                ngettext(
                    b"%d line less\0".as_ptr() as *const ::core::ffi::c_char,
                    b"%d fewer lines\0".as_ptr() as *const ::core::ffi::c_char,
                    pn as ::core::ffi::c_ulong,
                ),
                pn,
            );
        }
        if got_int.get() {
            xstrlcat(
                msg_buf.ptr() as *mut ::core::ffi::c_char,
                gettext(b" (Interrupted)\0".as_ptr() as *const ::core::ffi::c_char),
                MSG_BUF_LEN as size_t,
            );
        }
        if msg(
            msg_buf.ptr() as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        ) {
            set_keep_msg(
                msg_buf.ptr() as *mut ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
            keep_msg_more.set(true_0 != 0);
        }
    }
}
static redir_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub unsafe extern "C" fn msg_ext_set_kind(mut msg_kind: *const ::core::ffi::c_char) {
    msg_ext_ui_flush();
    msg_ext_kind.set(msg_kind);
    redir_col.set(if msg_ext_append.get() as ::core::ffi::c_int != 0 {
        redir_col.get()
    } else {
        0 as ::core::ffi::c_int
    });
}
pub unsafe extern "C" fn msg_ext_set_append(mut append: bool) {
    msg_ext_ui_flush();
    msg_ext_append.set(append);
}
pub unsafe extern "C" fn msg_ext_set_trigger(mut trigger: *const ::core::ffi::c_char) {
    msg_ext_ui_flush();
    msg_ext_trigger.set(trigger);
}
pub unsafe extern "C" fn msg_start() {
    let mut did_return: bool = false_0 != 0;
    msg_row.set(if msg_row.get() > cmdline_row.get() {
        msg_row.get()
    } else {
        cmdline_row.get()
    });
    if msg_silent.get() == 0 {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            keep_msg.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        need_fileinfo.set(false_0 != 0);
    }
    if need_highlight_changed.get() {
        highlight_changed();
    }
    if need_clr_eos.get() as ::core::ffi::c_int != 0
        || p_ch.get() == 0 as OptInt && redrawing_cmdline.get() as ::core::ffi::c_int != 0
    {
        need_clr_eos.set(false_0 != 0);
        msg_clr_eos();
    }
    if p_ch.get() == 0 as OptInt && !ui_has(kUIMessages) && msg_scrolled.get() == 0 {
        msg_grid_validate();
        msg_scroll_up(false_0 != 0, true_0 != 0);
        (*msg_scrolled.ptr()) += 1;
        cmdline_row.set(Rows.get() - 1 as ::core::ffi::c_int);
    }
    if msg_scroll.get() == 0 && full_screen.get() as ::core::ffi::c_int != 0 {
        msg_row.set(cmdline_row.get());
        msg_col.set(0 as ::core::ffi::c_int);
    } else if (msg_didout.get() as ::core::ffi::c_int != 0 || p_ch.get() == 0 as OptInt)
        && !ui_has(kUIMessages)
    {
        if p_ch.get() == 0 as OptInt && !msg_didout.get() && msg_use_printf() != 0 {
            msg_puts_display(
                b"\n\0".as_ptr() as *const ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                false_0,
            );
        } else {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        did_return = true_0 != 0;
        cmdline_row.set(msg_row.get());
    }
    if !msg_didany.get() || lines_left.get() < 0 as ::core::ffi::c_int {
        msg_starthere();
    }
    if msg_silent.get() == 0 as ::core::ffi::c_int {
        msg_didout.set(false_0 != 0);
    }
    if ui_has(kUIMessages) {
        msg_ext_ui_flush();
    }
    if !did_return {
        redir_write(
            b"\n\0".as_ptr() as *const ::core::ffi::c_char,
            1 as ptrdiff_t,
        );
    }
}
pub unsafe extern "C" fn msg_starthere() {
    lines_left.set(cmdline_row.get());
    msg_didany.set(false_0 != 0);
}
pub unsafe extern "C" fn msg_putchar(mut c: ::core::ffi::c_int) {
    msg_putchar_hl(c, 0 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn msg_putchar_hl(mut c: ::core::ffi::c_int, mut hl_id: ::core::ffi::c_int) {
    let mut buf: [::core::ffi::c_char; 7] = [0; 7];
    if c < 0 as ::core::ffi::c_int {
        buf[0 as ::core::ffi::c_int as usize] = K_SPECIAL as ::core::ffi::c_char;
        buf[1 as ::core::ffi::c_int as usize] = (if c == K_SPECIAL {
            KS_SPECIAL
        } else if c == NUL {
            KS_ZERO
        } else {
            -c & 0xff as ::core::ffi::c_int
        }) as ::core::ffi::c_char;
        buf[2 as ::core::ffi::c_int as usize] = (if c == K_SPECIAL || c == NUL {
            KE_FILLER as ::core::ffi::c_uint
        } else {
            -c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint
        }) as ::core::ffi::c_char;
        buf[3 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    } else {
        buf[utf_char2bytes(c, &raw mut buf as *mut ::core::ffi::c_char) as usize] =
            NUL as ::core::ffi::c_char;
    }
    msg_puts_hl(
        &raw mut buf as *mut ::core::ffi::c_char,
        hl_id,
        false_0 != 0,
    );
}
pub unsafe extern "C" fn msg_outnum(mut n: ::core::ffi::c_int) {
    let mut buf: [::core::ffi::c_char; 20] = [0; 20];
    snprintf(
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
        b"%d\0".as_ptr() as *const ::core::ffi::c_char,
        n,
    );
    msg_puts(&raw mut buf as *mut ::core::ffi::c_char);
}
pub unsafe extern "C" fn msg_home_replace(mut fname: *const ::core::ffi::c_char) {
    msg_home_replace_hl(fname, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn msg_home_replace_hl(
    mut fname: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
) {
    let mut name: *mut ::core::ffi::c_char =
        home_replace_save(::core::ptr::null_mut::<buf_T>(), fname);
    msg_outtrans(name, hl_id, false_0 != 0);
    xfree(name as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn msg_outtrans(
    mut str: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut hist: bool,
) -> ::core::ffi::c_int {
    return msg_outtrans_len(str, strlen(str) as ::core::ffi::c_int, hl_id, hist);
}
pub unsafe extern "C" fn msg_outtrans_one(
    mut p: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut hist: bool,
) -> *const ::core::ffi::c_char {
    let mut l: ::core::ffi::c_int = 0;
    l = utfc_ptr2len(p);
    if l > 1 as ::core::ffi::c_int {
        msg_outtrans_len(p, l, hl_id, hist);
        return p.offset(l as isize);
    }
    msg_puts_hl(
        transchar_byte_buf(
            ::core::ptr::null::<buf_T>(),
            *p as uint8_t as ::core::ffi::c_int,
        ),
        hl_id,
        hist,
    );
    return p.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe extern "C" fn msg_outtrans_len(
    mut msgstr: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
    mut hist: bool,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut str: *const ::core::ffi::c_char = msgstr;
    let mut plain_start: *const ::core::ffi::c_char = msgstr;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut c: ::core::ffi::c_int = 0;
    let mut save_got_int: ::core::ffi::c_int = got_int.get() as ::core::ffi::c_int;
    got_int.set(false_0 != 0);
    if hist {
        msg_hist_add(str, len, hl_id);
    }
    if msg_silent.get() == 0 as ::core::ffi::c_int
        && len > 0 as ::core::ffi::c_int
        && msg_row.get() >= cmdline_row.get()
        && msg_col.get() == 0 as ::core::ffi::c_int
    {
        clear_cmdline.set(false_0 != 0);
        mode_displayed.set(false_0 != 0);
    }
    loop {
        len -= 1;
        if !(len >= 0 as ::core::ffi::c_int && !got_int.get()) {
            break;
        }
        let mut mb_l: ::core::ffi::c_int = utfc_ptr2len_len(str, len + 1 as ::core::ffi::c_int);
        if mb_l > 1 as ::core::ffi::c_int {
            c = utf_ptr2char(str);
            if vim_isprintc(c) {
                retval += utf_ptr2cells(str);
            } else {
                if str > plain_start {
                    msg_puts_len(plain_start, str.offset_from(plain_start), hl_id, hist);
                }
                plain_start = str.offset(mb_l as isize);
                msg_puts_hl(
                    transchar_buf(::core::ptr::null::<buf_T>(), c),
                    if hl_id == 0 as ::core::ffi::c_int {
                        HLF_8
                    } else {
                        hl_id
                    },
                    false_0 != 0,
                );
                retval += char2cells(c);
            }
            len -= mb_l - 1 as ::core::ffi::c_int;
            str = str.offset(mb_l as isize);
        } else {
            s = transchar_byte_buf(
                ::core::ptr::null::<buf_T>(),
                *str as uint8_t as ::core::ffi::c_int,
            );
            if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                if str > plain_start {
                    msg_puts_len(plain_start, str.offset_from(plain_start), hl_id, hist);
                }
                plain_start = str.offset(1 as ::core::ffi::c_int as isize);
                msg_puts_hl(
                    s,
                    if hl_id == 0 as ::core::ffi::c_int {
                        HLF_8
                    } else {
                        hl_id
                    },
                    false_0 != 0,
                );
                retval += strlen(s) as ::core::ffi::c_int;
            } else {
                retval += 1;
            }
            str = str.offset(1);
        }
    }
    if (str > plain_start || plain_start == msgstr) && !got_int.get() {
        msg_puts_len(plain_start, str.offset_from(plain_start), hl_id, hist);
    }
    got_int.set(got_int.get() as ::core::ffi::c_int | save_got_int != 0);
    return retval;
}
pub unsafe extern "C" fn msg_make(mut arg: *const ::core::ffi::c_char) {
    let mut i: ::core::ffi::c_int = 0;
    static str: GlobalCell<*const ::core::ffi::c_char> =
        GlobalCell::new(b"eeffoc\0".as_ptr() as *const ::core::ffi::c_char);
    static rs: GlobalCell<*const ::core::ffi::c_char> =
        GlobalCell::new(b"Plon#dqg#vxjduB\0".as_ptr() as *const ::core::ffi::c_char);
    arg = skipwhite(arg);
    i = 5 as ::core::ffi::c_int;
    while *arg as ::core::ffi::c_int != 0 && i >= 0 as ::core::ffi::c_int {
        let c2rust_fresh33 = arg;
        arg = arg.offset(1);
        if *c2rust_fresh33 as ::core::ffi::c_int
            != *(*str.ptr()).offset(i as isize) as ::core::ffi::c_int
        {
            break;
        }
        i -= 1;
    }
    if i < 0 as ::core::ffi::c_int {
        msg_putchar('\n' as ::core::ffi::c_int);
        i = 0 as ::core::ffi::c_int;
        while *(*rs.ptr()).offset(i as isize) != 0 {
            msg_putchar(
                *(*rs.ptr()).offset(i as isize) as ::core::ffi::c_int - 3 as ::core::ffi::c_int,
            );
            i += 1;
        }
    }
}
pub unsafe extern "C" fn msg_outtrans_special(
    mut strstart: *const ::core::ffi::c_char,
    mut from: bool,
    mut maxlen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if strstart.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let mut str: *const ::core::ffi::c_char = strstart;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut hl_id: ::core::ffi::c_int = HLF_8;
    while *str as ::core::ffi::c_int != NUL {
        let mut text: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if (str == strstart
            || *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
            && *str as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
        {
            text = b"<Space>\0".as_ptr() as *const ::core::ffi::c_char;
            str = str.offset(1);
        } else {
            text = str2special(&raw mut str, from, false_0 != 0);
        }
        if *text.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *text.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            text = transchar_byte_buf(
                ::core::ptr::null::<buf_T>(),
                *text.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int,
            );
        }
        let len: ::core::ffi::c_int = vim_strsize(text);
        if maxlen > 0 as ::core::ffi::c_int && retval + len >= maxlen {
            break;
        }
        msg_puts_hl(
            text,
            if len > 1 as ::core::ffi::c_int && utfc_ptr2len(text) <= 1 as ::core::ffi::c_int {
                hl_id
            } else {
                0 as ::core::ffi::c_int
            },
            false_0 != 0,
        );
        retval += len;
    }
    return retval;
}
pub unsafe extern "C" fn str2special_save(
    str: *const ::core::ffi::c_char,
    replace_spaces: bool,
    replace_lt: bool,
) -> *mut ::core::ffi::c_char {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        1 as ::core::ffi::c_int,
        40 as ::core::ffi::c_int,
    );
    let mut p: *const ::core::ffi::c_char = str;
    while *p as ::core::ffi::c_int != NUL {
        ga_concat(
            &raw mut ga,
            str2special(&raw mut p, replace_spaces, replace_lt),
        );
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    return ga.ga_data as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn str2special_arena(
    mut str: *const ::core::ffi::c_char,
    mut replace_spaces: bool,
    mut replace_lt: bool,
    mut arena: *mut Arena,
) -> *mut ::core::ffi::c_char {
    let mut p: *const ::core::ffi::c_char = str;
    let mut len: size_t = 0 as size_t;
    while *p != 0 {
        len = len.wrapping_add(strlen(str2special(&raw mut p, replace_spaces, replace_lt)));
    }
    let mut buf: *mut ::core::ffi::c_char =
        arena_alloc(arena, len.wrapping_add(1 as size_t), false_0 != 0) as *mut ::core::ffi::c_char;
    let mut pos: size_t = 0 as size_t;
    p = str;
    while *p != 0 {
        let mut s: *const ::core::ffi::c_char = str2special(&raw mut p, replace_spaces, replace_lt);
        let mut s_len: size_t = strlen(s);
        memcpy(
            buf.offset(pos as isize) as *mut ::core::ffi::c_void,
            s as *const ::core::ffi::c_void,
            s_len,
        );
        pos = pos.wrapping_add(s_len);
    }
    *buf.offset(pos as isize) = NUL as ::core::ffi::c_char;
    return buf;
}
pub unsafe extern "C" fn str2special(
    sp: *mut *const ::core::ffi::c_char,
    replace_spaces: bool,
    replace_lt: bool,
) -> *const ::core::ffi::c_char {
    static buf: GlobalCell<[::core::ffi::c_char; 7]> = GlobalCell::new([0; 7]);
    let p: *const ::core::ffi::c_char = mb_unescape(sp);
    if !p.is_null() {
        return p;
    }
    let mut str: *const ::core::ffi::c_char = *sp;
    let mut c: ::core::ffi::c_int = *str as uint8_t as ::core::ffi::c_int;
    let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut special: bool = false_0 != 0;
    if c == K_SPECIAL
        && *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        && *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
    {
        if *str.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            == KS_MODIFIER
        {
            modifiers =
                *str.offset(2 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int;
            str = str.offset(3 as ::core::ffi::c_int as isize);
            c = *str as uint8_t as ::core::ffi::c_int;
        }
        if c == K_SPECIAL
            && *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            c = if *str.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                == KS_SPECIAL
            {
                K_SPECIAL
            } else if *str.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                == KS_ZERO
            {
                K_ZERO
            } else {
                -(*str.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                    + ((*str.offset(2 as ::core::ffi::c_int as isize) as uint8_t
                        as ::core::ffi::c_int)
                        << 8 as ::core::ffi::c_int))
            };
            str = str.offset(2 as ::core::ffi::c_int as isize);
        }
        if c < 0 as ::core::ffi::c_int || modifiers != 0 {
            special = true_0 != 0;
        }
    }
    if !(c < 0 as ::core::ffi::c_int)
        && (*utf8len_tab.ptr())[c as usize] as ::core::ffi::c_int > 1 as ::core::ffi::c_int
    {
        *sp = str;
        let mut p_0: *const ::core::ffi::c_char = mb_unescape(sp);
        if !p_0.is_null() {
            c = utf_ptr2char(p_0);
        } else {
            *sp = str.offset(1 as ::core::ffi::c_int as isize);
        }
    } else {
        *sp = str.offset(
            (if *str as ::core::ffi::c_int == NUL {
                0 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            }) as isize,
        );
    }
    if special as ::core::ffi::c_int != 0
        || c < ' ' as ::core::ffi::c_int
        || replace_spaces as ::core::ffi::c_int != 0 && c == ' ' as ::core::ffi::c_int
        || replace_lt as ::core::ffi::c_int != 0 && c == '<' as ::core::ffi::c_int
    {
        return get_special_key_name(c, modifiers);
    }
    (*buf.ptr())[0 as ::core::ffi::c_int as usize] = c as ::core::ffi::c_char;
    (*buf.ptr())[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    return buf.ptr() as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn msg_prt_line(mut s: *const ::core::ffi::c_char, mut list: bool) {
    let mut sc: schar_T = 0;
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut n_extra: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut sc_extra: schar_T = 0 as schar_T;
    let mut sc_final: schar_T = 0 as schar_T;
    let mut p_extra: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: ::core::ffi::c_int = 0;
    let mut hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut lead: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut in_multispace: bool = false_0 != 0;
    let mut multispace_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut trail: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut l: ::core::ffi::c_int = 0;
    if (*curwin.get()).w_onebuf_opt.wo_list != 0 {
        list = true_0 != 0;
    }
    if list {
        if (*curwin.get()).w_p_lcs_chars.trail != 0 {
            trail = s.offset(strlen(s) as isize);
            while trail > s
                && ascii_iswhite(
                    *trail.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
            {
                trail = trail.offset(-1);
            }
        }
        if (*curwin.get()).w_p_lcs_chars.lead != 0
            || !(*curwin.get()).w_p_lcs_chars.leadmultispace.is_null()
            || (*curwin.get()).w_p_lcs_chars.leadtab1 != NUL as schar_T
        {
            lead = s;
            while ascii_iswhite(*lead.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            {
                lead = lead.offset(1);
            }
            if *lead as ::core::ffi::c_int == NUL {
                lead = ::core::ptr::null::<::core::ffi::c_char>();
            }
        }
    }
    if *s as ::core::ffi::c_int == NUL
        && !(list as ::core::ffi::c_int != 0 && (*curwin.get()).w_p_lcs_chars.eol != NUL as schar_T)
    {
        msg_putchar(' ' as ::core::ffi::c_int);
    }
    while !got_int.get() {
        if n_extra > 0 as ::core::ffi::c_int {
            n_extra -= 1;
            if n_extra == 0 as ::core::ffi::c_int && sc_final != 0 {
                sc = sc_final;
            } else if sc_extra != 0 {
                sc = sc_extra;
            } else {
                '_c2rust_label: {
                    if !p_extra.is_null() {
                    } else {
                        __assert_fail(
                            b"p_extra != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            2209 as ::core::ffi::c_uint,
                            b"void msg_prt_line(const char *, _Bool)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                let c2rust_fresh34 = p_extra;
                p_extra = p_extra.offset(1);
                sc = *c2rust_fresh34 as ::core::ffi::c_uchar as schar_T;
            }
        } else {
            l = utfc_ptr2len(s);
            if l > 1 as ::core::ffi::c_int {
                col += utf_ptr2cells(s);
                let mut buf: [::core::ffi::c_char; 22] = [0; 22];
                if l >= MB_MAXBYTES as ::core::ffi::c_int {
                    xstrlcpy(
                        &raw mut buf as *mut ::core::ffi::c_char,
                        b"?\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 22]>(),
                    );
                } else if (*curwin.get()).w_p_lcs_chars.nbsp != NUL as schar_T
                    && list as ::core::ffi::c_int != 0
                    && (utf_ptr2char(s) == 160 as ::core::ffi::c_int
                        || utf_ptr2char(s) == 0x202f as ::core::ffi::c_int)
                {
                    schar_get(
                        &raw mut buf as *mut ::core::ffi::c_char,
                        (*curwin.get()).w_p_lcs_chars.nbsp,
                    );
                } else {
                    memmove(
                        &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                        s as *const ::core::ffi::c_void,
                        l as size_t,
                    );
                    buf[l as usize] = NUL as ::core::ffi::c_char;
                }
                msg_puts(&raw mut buf as *mut ::core::ffi::c_char);
                s = s.offset(l as isize);
                continue;
            } else {
                hl_id = 0 as ::core::ffi::c_int;
                let c2rust_fresh35 = s;
                s = s.offset(1);
                let mut c: ::core::ffi::c_int = *c2rust_fresh35 as uint8_t as ::core::ffi::c_int;
                if c >= 0x80 as ::core::ffi::c_int {
                    col += utf_char2cells(c);
                    msg_putchar(c);
                    continue;
                } else {
                    sc_extra = NUL as schar_T;
                    sc_final = NUL as schar_T;
                    if list {
                        in_multispace = c == ' ' as ::core::ffi::c_int
                            && (*s as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                                || col > 0 as ::core::ffi::c_int
                                    && *s.offset(-2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == ' ' as ::core::ffi::c_int);
                        if !in_multispace {
                            multispace_pos = 0 as ::core::ffi::c_int;
                        }
                    }
                    if c == TAB && (!list || (*curwin.get()).w_p_lcs_chars.tab1 != 0) {
                        n_extra = tabstop_padding(
                            col as colnr_T,
                            (*curbuf.get()).b_p_ts,
                            (*curbuf.get()).b_p_vts_array,
                        ) - 1 as ::core::ffi::c_int;
                        if !list {
                            sc = ' ' as ::core::ffi::c_int as schar_T;
                            sc_extra = ' ' as ::core::ffi::c_int as schar_T;
                        } else {
                            let mut lcs_tab1: schar_T = (*curwin.get()).w_p_lcs_chars.tab1;
                            let mut lcs_tab2: schar_T = (*curwin.get()).w_p_lcs_chars.tab2;
                            let mut lcs_tab3: schar_T = (*curwin.get()).w_p_lcs_chars.tab3;
                            if !lead.is_null()
                                && s <= lead
                                && (*curwin.get()).w_p_lcs_chars.leadtab1 != NUL as schar_T
                            {
                                lcs_tab1 = (*curwin.get()).w_p_lcs_chars.leadtab1;
                                lcs_tab2 = (*curwin.get()).w_p_lcs_chars.leadtab2;
                                lcs_tab3 = (*curwin.get()).w_p_lcs_chars.leadtab3;
                            }
                            sc = if n_extra == 0 as ::core::ffi::c_int && lcs_tab3 != 0 {
                                lcs_tab3
                            } else {
                                lcs_tab1
                            };
                            sc_extra = lcs_tab2;
                            sc_final = lcs_tab3;
                            hl_id = HLF_0;
                        }
                    } else if c == NUL
                        && list as ::core::ffi::c_int != 0
                        && (*curwin.get()).w_p_lcs_chars.eol != NUL as schar_T
                    {
                        p_extra = b"\0".as_ptr() as *const ::core::ffi::c_char;
                        n_extra = 1 as ::core::ffi::c_int;
                        sc = (*curwin.get()).w_p_lcs_chars.eol;
                        hl_id = HLF_AT;
                        s = s.offset(-1);
                    } else if c != NUL && {
                        n = byte2cells(c);
                        n > 1 as ::core::ffi::c_int
                    } {
                        n_extra = n - 1 as ::core::ffi::c_int;
                        p_extra = transchar_byte_buf(::core::ptr::null::<buf_T>(), c);
                        let c2rust_fresh36 = p_extra;
                        p_extra = p_extra.offset(1);
                        sc = *c2rust_fresh36 as schar_T;
                        hl_id = HLF_0;
                    } else if c == ' ' as ::core::ffi::c_int {
                        if !lead.is_null()
                            && s <= lead
                            && in_multispace as ::core::ffi::c_int != 0
                            && !(*curwin.get()).w_p_lcs_chars.leadmultispace.is_null()
                        {
                            let c2rust_fresh37 = multispace_pos;
                            multispace_pos = multispace_pos + 1;
                            sc = *(*curwin.get())
                                .w_p_lcs_chars
                                .leadmultispace
                                .offset(c2rust_fresh37 as isize);
                            if *(*curwin.get())
                                .w_p_lcs_chars
                                .leadmultispace
                                .offset(multispace_pos as isize)
                                == NUL as schar_T
                            {
                                multispace_pos = 0 as ::core::ffi::c_int;
                            }
                            hl_id = HLF_0;
                        } else if !lead.is_null()
                            && s <= lead
                            && (*curwin.get()).w_p_lcs_chars.lead != NUL as schar_T
                        {
                            sc = (*curwin.get()).w_p_lcs_chars.lead;
                            hl_id = HLF_0;
                        } else if !trail.is_null() && s > trail {
                            sc = (*curwin.get()).w_p_lcs_chars.trail;
                            hl_id = HLF_0;
                        } else if in_multispace as ::core::ffi::c_int != 0
                            && !(*curwin.get()).w_p_lcs_chars.multispace.is_null()
                        {
                            let c2rust_fresh38 = multispace_pos;
                            multispace_pos = multispace_pos + 1;
                            sc = *(*curwin.get())
                                .w_p_lcs_chars
                                .multispace
                                .offset(c2rust_fresh38 as isize);
                            if *(*curwin.get())
                                .w_p_lcs_chars
                                .multispace
                                .offset(multispace_pos as isize)
                                == NUL as schar_T
                            {
                                multispace_pos = 0 as ::core::ffi::c_int;
                            }
                            hl_id = HLF_0;
                        } else if list as ::core::ffi::c_int != 0
                            && (*curwin.get()).w_p_lcs_chars.space != NUL as schar_T
                        {
                            sc = (*curwin.get()).w_p_lcs_chars.space;
                            hl_id = HLF_0;
                        } else {
                            sc = ' ' as ::core::ffi::c_int as schar_T;
                        }
                    } else {
                        sc = c as schar_T;
                    }
                }
            }
        }
        if sc == NUL as schar_T {
            break;
        }
        let mut buf_0: [::core::ffi::c_char; 32] = [0; 32];
        schar_get(&raw mut buf_0 as *mut ::core::ffi::c_char, sc);
        msg_puts_hl(
            &raw mut buf_0 as *mut ::core::ffi::c_char,
            hl_id,
            false_0 != 0,
        );
        col += 1;
    }
    msg_clr_eos();
}
pub unsafe extern "C" fn msg_puts(mut s: *const ::core::ffi::c_char) {
    msg_puts_hl(s, 0 as ::core::ffi::c_int, false_0 != 0);
}
pub unsafe extern "C" fn msg_puts_title(mut s: *const ::core::ffi::c_char) {
    s = s.offset(
        (ui_has(kUIMessages) as ::core::ffi::c_int != 0
            && *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int) as ::core::ffi::c_int
            as isize,
    );
    msg_puts_hl(s, HLF_T, false_0 != 0);
}
pub unsafe extern "C" fn msg_outtrans_long(
    mut longstr: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
) {
    let mut len: ::core::ffi::c_int = strlen(longstr) as ::core::ffi::c_int;
    let mut slen: ::core::ffi::c_int = len;
    let mut room: ::core::ffi::c_int = Columns.get() - msg_col.get();
    if !ui_has(kUIMessages) && len > room && room >= 20 as ::core::ffi::c_int {
        slen = (room - 3 as ::core::ffi::c_int) / 2 as ::core::ffi::c_int;
        msg_outtrans_len(longstr, slen, hl_id, false_0 != 0);
        msg_puts_hl(
            b"...\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_8,
            false_0 != 0,
        );
    }
    msg_outtrans_len(
        longstr.offset(len as isize).offset(-(slen as isize)),
        slen,
        hl_id,
        false_0 != 0,
    );
}
pub unsafe extern "C" fn msg_puts_hl(
    s: *const ::core::ffi::c_char,
    hl_id: ::core::ffi::c_int,
    hist: bool,
) {
    msg_puts_len(s, -1 as ptrdiff_t, hl_id, hist);
}
pub unsafe extern "C" fn msg_puts_len(
    str: *const ::core::ffi::c_char,
    len: ptrdiff_t,
    mut hl_id: ::core::ffi::c_int,
    mut hist: bool,
) {
    '_c2rust_label: {
        if len < 0 as ptrdiff_t
            || memchr(
                str as *const ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                len as size_t,
            )
            .is_null()
        {
        } else {
            __assert_fail(
                b"len < 0 || memchr(str, 0, (size_t)len) == NULL\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2367 as ::core::ffi::c_uint,
                b"void msg_puts_len(const char *const, const ptrdiff_t, int, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    redir_write(str, len);
    if msg_silent.get() != 0 as ::core::ffi::c_int || *str as ::core::ffi::c_int == NUL {
        if *str as ::core::ffi::c_int == NUL && ui_has(kUIMessages) as ::core::ffi::c_int != 0 {
            msg_ext_ui_flush();
            ui_call_msg_show(
                cstr_as_string(b"empty\0".as_ptr() as *const ::core::ffi::c_char),
                Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                },
                false_0 != 0,
                false_0 != 0,
                false_0 != 0,
                object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed_11 {
                        integer: -1 as Integer,
                    },
                },
                String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0 as size_t,
                },
            );
            cmdline_was_last_drawn.set(false_0 != 0);
        }
        return;
    }
    if hist {
        msg_hist_add(str, len as ::core::ffi::c_int, hl_id);
    }
    let mut overflow: bool = !ui_has(kUIMessages)
        && msg_scrolled.get()
            > (if p_ch.get() == 0 as OptInt {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            });
    if overflow as ::core::ffi::c_int != 0
        && !msg_scrolled_ign.get()
        && strcmp(str, b"\r\0".as_ptr() as *const ::core::ffi::c_char) != 0 as ::core::ffi::c_int
    {
        need_wait_return.set(true_0 != 0);
    }
    msg_didany.set(true_0 != 0);
    if msg_use_printf() != 0 {
        let mut saved_msg_col: ::core::ffi::c_int = msg_col.get();
        msg_puts_printf(str, len);
        if headless_mode.get() {
            msg_col.set(saved_msg_col);
        }
    }
    if msg_use_printf() == 0
        || headless_mode.get() as ::core::ffi::c_int != 0 && !(*default_grid.ptr()).chars.is_null()
    {
        msg_puts_display(str, len as ::core::ffi::c_int, hl_id, false_0);
    }
    need_fileinfo.set(false_0 != 0);
}
unsafe extern "C" fn msg_ext_emit_chunk() {
    if (*msg_ext_chunks.ptr()).is_null() {
        msg_ext_init_chunks();
    }
    if msg_ext_last_attr.get() == -1 as sattr_T {
        return;
    }
    let mut chunk: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    if chunk.size == chunk.capacity {
        chunk.capacity = if chunk.capacity != 0 {
            chunk.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        chunk.items = xrealloc(
            chunk.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<Object>().wrapping_mul(chunk.capacity),
        ) as *mut Object;
    } else {
    };
    let c2rust_fresh1 = chunk.size;
    chunk.size = chunk.size.wrapping_add(1);
    *chunk.items.offset(c2rust_fresh1 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_11 {
            integer: msg_ext_last_attr.get() as Integer,
        },
    };
    msg_ext_last_attr.set(-1 as ::core::ffi::c_int as sattr_T);
    let mut text: String_0 = ga_take_string(msg_ext_last_chunk.ptr());
    if chunk.size == chunk.capacity {
        chunk.capacity = if chunk.capacity != 0 {
            chunk.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        chunk.items = xrealloc(
            chunk.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<Object>().wrapping_mul(chunk.capacity),
        ) as *mut Object;
    } else {
    };
    let c2rust_fresh2 = chunk.size;
    chunk.size = chunk.size.wrapping_add(1);
    *chunk.items.offset(c2rust_fresh2 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_11 { string: text },
    };
    if chunk.size == chunk.capacity {
        chunk.capacity = if chunk.capacity != 0 {
            chunk.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        chunk.items = xrealloc(
            chunk.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<Object>().wrapping_mul(chunk.capacity),
        ) as *mut Object;
    } else {
    };
    let c2rust_fresh3 = chunk.size;
    chunk.size = chunk.size.wrapping_add(1);
    *chunk.items.offset(c2rust_fresh3 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_11 {
            integer: msg_ext_last_hl_id.get() as Integer,
        },
    };
    if (*msg_ext_chunks.get()).size == (*msg_ext_chunks.get()).capacity {
        (*msg_ext_chunks.get()).capacity = if (*msg_ext_chunks.get()).capacity != 0 {
            (*msg_ext_chunks.get()).capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        (*msg_ext_chunks.get()).items = xrealloc(
            (*msg_ext_chunks.get()).items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<Object>().wrapping_mul((*msg_ext_chunks.get()).capacity),
        ) as *mut Object;
    } else {
    };
    let c2rust_fresh4 = (*msg_ext_chunks.get()).size;
    (*msg_ext_chunks.get()).size = (*msg_ext_chunks.get()).size.wrapping_add(1);
    *(*msg_ext_chunks.get()).items.offset(c2rust_fresh4 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_11 { array: chunk },
    };
}
unsafe extern "C" fn msg_puts_display(
    mut str: *const ::core::ffi::c_char,
    mut maxlen: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
    mut recurse: ::core::ffi::c_int,
) {
    let mut s: *const ::core::ffi::c_char = str;
    let mut sb_str: *const ::core::ffi::c_char = str;
    let mut sb_col: ::core::ffi::c_int = msg_col.get();
    let mut attr: ::core::ffi::c_int = if hl_id != 0 {
        syn_id2attr(hl_id)
    } else {
        0 as ::core::ffi::c_int
    };
    did_wait_return.set(false_0 != 0);
    if ui_has(kUIMessages) {
        if attr as sattr_T != msg_ext_last_attr.get() {
            msg_ext_emit_chunk();
            msg_ext_last_attr.set(attr as sattr_T);
            msg_ext_last_hl_id.set(hl_id);
        }
        let mut len: size_t = if maxlen < 0 as ::core::ffi::c_int {
            strlen(str)
        } else {
            strnlen(str, maxlen as size_t)
        };
        ga_concat_len(msg_ext_last_chunk.ptr(), str, len);
        let mut lastline: *const ::core::ffi::c_char =
            xmemrchr(str as *const ::core::ffi::c_void, '\n' as uint8_t, len)
                as *const ::core::ffi::c_char;
        maxlen -= (if !lastline.is_null() {
            lastline.offset_from(str)
        } else {
            0 as isize
        }) as ::core::ffi::c_int;
        let mut p: *const ::core::ffi::c_char = if !lastline.is_null() {
            lastline.offset(1 as ::core::ffi::c_int as isize)
        } else {
            str
        };
        let mut col: ::core::ffi::c_int = (if maxlen < 0 as ::core::ffi::c_int {
            mb_string2cells(p)
        } else {
            mb_string2cells_len(p, maxlen as size_t)
        }) as ::core::ffi::c_int;
        msg_col.set(
            (if !lastline.is_null() {
                0 as ::core::ffi::c_int
            } else {
                msg_col.get()
            }) + col,
        );
        return;
    }
    let mut print_attr: ::core::ffi::c_int =
        hl_combine_attr(*(*hl_attr_active.ptr()).offset(HLF_MSG as isize), attr);
    msg_grid_validate();
    cmdline_was_last_drawn.set(redrawing_cmdline.get());
    let mut msg_row_pending: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    loop {
        if msg_col.get() >= Columns.get() {
            if p_more.get() != 0 && recurse == 0 {
                store_sb_text(&raw mut sb_str, s, hl_id, &raw mut sb_col, true_0);
            }
            if msg_no_more.get() as ::core::ffi::c_int != 0
                && lines_left.get() == 0 as ::core::ffi::c_int
            {
                break;
            }
            msg_col.set(0 as ::core::ffi::c_int);
            (*msg_row.ptr()) += 1;
            msg_didout.set(false_0 != 0);
        }
        if msg_row.get() >= Rows.get() {
            msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
            if msg_no_more.get() as ::core::ffi::c_int != 0
                && lines_left.get() == 0 as ::core::ffi::c_int
            {
                break;
            }
            if recurse == 0 {
                if msg_row_pending >= 0 as ::core::ffi::c_int {
                    msg_line_flush();
                    msg_row_pending = -1 as ::core::ffi::c_int;
                }
                msg_scroll_up(true_0 != 0, false_0 != 0);
                inc_msg_scrolled();
                need_wait_return.set(true_0 != 0);
                redraw_cmdline.set(true_0 != 0);
                if cmdline_row.get() > 0 as ::core::ffi::c_int && !exmode_active.get() {
                    (*cmdline_row.ptr()) -= 1;
                }
                if lines_left.get() > 0 as ::core::ffi::c_int {
                    (*lines_left.ptr()) -= 1;
                }
                if p_more.get() != 0
                    && lines_left.get() == 0 as ::core::ffi::c_int
                    && State.get() != MODE_HITRETURN
                    && !msg_no_more.get()
                    && !exmode_active.get()
                {
                    if do_more_prompt(NUL) {
                        s = confirm_buttons.get();
                    }
                    if quit_more.get() {
                        return;
                    }
                }
            }
        }
        if !((maxlen < 0 as ::core::ffi::c_int
            || (s.offset_from(str) as ::core::ffi::c_int) < maxlen)
            && *s as ::core::ffi::c_int != NUL)
        {
            break;
        }
        if msg_row.get() != msg_row_pending
            && (*s as uint8_t as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
                || *s as ::core::ffi::c_int == TAB)
        {
            if msg_row_pending >= 0 as ::core::ffi::c_int {
                msg_line_flush();
            }
            grid_line_start(msg_grid_adj.ptr(), msg_row.get());
            msg_row_pending = msg_row.get();
        }
        if *s as uint8_t as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int {
            let mut cw: ::core::ffi::c_int = utf_ptr2cells(s);
            let mut l: ::core::ffi::c_int = if maxlen >= 0 as ::core::ffi::c_int {
                utfc_ptr2len_len(
                    s,
                    str.offset(maxlen as isize).offset_from(s) as ::core::ffi::c_int,
                )
            } else {
                utfc_ptr2len(s)
            };
            if cw > 1 as ::core::ffi::c_int
                && msg_col.get() == Columns.get() - 1 as ::core::ffi::c_int
            {
                grid_line_puts(
                    msg_col.get(),
                    b">\0".as_ptr() as *const ::core::ffi::c_char,
                    1 as ::core::ffi::c_int,
                    *(*hl_attr_active.ptr()).offset(HLF_AT as isize),
                );
                cw = 1 as ::core::ffi::c_int;
            } else {
                grid_line_puts(msg_col.get(), s, l, print_attr);
                s = s.offset(l as isize);
            }
            msg_didout.set(true_0 != 0);
            (*msg_col.ptr()) += cw;
        } else {
            let c2rust_fresh5 = s;
            s = s.offset(1);
            let mut c: ::core::ffi::c_char = *c2rust_fresh5;
            if c as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                msg_didout.set(false_0 != 0);
                msg_col.set(0 as ::core::ffi::c_int);
                (*msg_row.ptr()) += 1;
                if p_more.get() != 0 && recurse == 0 {
                    store_sb_text(&raw mut sb_str, s, hl_id, &raw mut sb_col, true_0);
                }
            } else if c as ::core::ffi::c_int == '\r' as ::core::ffi::c_int {
                msg_col.set(0 as ::core::ffi::c_int);
            } else if c as ::core::ffi::c_int == '\u{8}' as ::core::ffi::c_int {
                if msg_col.get() != 0 {
                    (*msg_col.ptr()) -= 1;
                }
            } else if c as ::core::ffi::c_int == TAB {
                loop {
                    grid_line_puts(
                        msg_col.get(),
                        b" \0".as_ptr() as *const ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                        print_attr,
                    );
                    (*msg_col.ptr()) += 1 as ::core::ffi::c_int;
                    if msg_col.get() == Columns.get() {
                        break;
                    }
                    if msg_col.get() & 7 as ::core::ffi::c_int == 0 {
                        break;
                    }
                }
            } else if c as ::core::ffi::c_int == BELL {
                vim_beep(kOptBoFlagShell as ::core::ffi::c_int as ::core::ffi::c_uint);
            }
        }
    }
    if msg_row_pending >= 0 as ::core::ffi::c_int {
        msg_line_flush();
    }
    msg_cursor_goto(msg_row.get(), msg_col.get());
    if p_more.get() != 0 && recurse == 0 {
        store_sb_text(&raw mut sb_str, s, hl_id, &raw mut sb_col, false_0);
    }
    msg_check();
}
pub unsafe extern "C" fn msg_line_flush() {
    if cmdmsg_rl.get() {
        grid_line_mirror((*msg_grid.ptr()).cols);
    }
    grid_line_flush_if_valid_row();
}
pub unsafe extern "C" fn msg_cursor_goto(mut row: ::core::ffi::c_int, mut col: ::core::ffi::c_int) {
    if cmdmsg_rl.get() {
        col = Columns.get() - 1 as ::core::ffi::c_int - col;
    }
    let mut grid: *mut ScreenGrid = grid_adjust(msg_grid_adj.ptr(), &raw mut row, &raw mut col);
    ui_grid_cursor_goto((*grid).handle, row, col);
}
pub unsafe extern "C" fn message_filtered(mut msg_0: *const ::core::ffi::c_char) -> bool {
    if (*cmdmod.ptr()).cmod_filter_regmatch.regprog.is_null() {
        return false_0 != 0;
    }
    let mut match_0: bool = vim_regexec(
        &raw mut (*cmdmod.ptr()).cmod_filter_regmatch,
        msg_0,
        0 as colnr_T,
    );
    return if (*cmdmod.ptr()).cmod_filter_force as ::core::ffi::c_int != 0 {
        match_0 as ::core::ffi::c_int
    } else {
        !match_0 as ::core::ffi::c_int
    } != 0;
}
pub unsafe extern "C" fn msg_scrollsize() -> ::core::ffi::c_int {
    return msg_scrolled.get()
        + p_ch.get() as ::core::ffi::c_int
        + (if p_ch.get() > 0 as OptInt || msg_scrolled.get() > 1 as ::core::ffi::c_int {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
}
pub unsafe extern "C" fn msg_do_throttle() -> bool {
    return msg_use_grid() as ::core::ffi::c_int != 0
        && rdb_flags.get() & kOptRdbFlagNothrottle as ::core::ffi::c_int as ::core::ffi::c_uint
            == 0;
}
pub unsafe extern "C" fn msg_scroll_up(mut may_throttle: bool, mut zerocmd: bool) {
    if may_throttle as ::core::ffi::c_int != 0 && msg_do_throttle() as ::core::ffi::c_int != 0 {
        (*msg_grid.ptr()).throttled = true_0 != 0;
    }
    msg_did_scroll.set(true_0 != 0);
    if msg_grid_pos.get() > 0 as ::core::ffi::c_int {
        msg_grid_set_pos(msg_grid_pos.get() - 1 as ::core::ffi::c_int, !zerocmd);
        if zerocmd as ::core::ffi::c_int != 0 && !(*msg_grid.ptr()).chars.is_null() {
            grid_clear_line(
                msg_grid.ptr(),
                *(*msg_grid.ptr())
                    .line_offset
                    .offset(0 as ::core::ffi::c_int as isize),
                (*msg_grid.ptr()).cols,
                false_0 != 0,
            );
        }
    } else {
        grid_del_lines(
            msg_grid.ptr(),
            0 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
            (*msg_grid.ptr()).rows,
            0 as ::core::ffi::c_int,
            (*msg_grid.ptr()).cols,
        );
        memmove(
            (*msg_grid.ptr()).dirty_col as *mut ::core::ffi::c_void,
            (*msg_grid.ptr())
                .dirty_col
                .offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            (((*msg_grid.ptr()).rows - 1 as ::core::ffi::c_int) as size_t)
                .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
        );
        *(*msg_grid.ptr())
            .dirty_col
            .offset(((*msg_grid.ptr()).rows - 1 as ::core::ffi::c_int) as isize) =
            0 as ::core::ffi::c_int;
    }
    grid_clear(
        msg_grid_adj.ptr(),
        Rows.get() - 1 as ::core::ffi::c_int,
        Rows.get(),
        0 as ::core::ffi::c_int,
        Columns.get(),
        *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
    );
}
pub unsafe extern "C" fn msg_scroll_flush() {
    if (*msg_grid.ptr()).throttled {
        (*msg_grid.ptr()).throttled = false_0 != 0;
        let mut pos_delta: ::core::ffi::c_int = msg_grid_pos_at_flush.get() - msg_grid_pos.get();
        '_c2rust_label: {
            if pos_delta >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"pos_delta >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2689 as ::core::ffi::c_uint,
                    b"void msg_scroll_flush(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut delta: ::core::ffi::c_int =
            if msg_scrolled.get() - msg_scrolled_at_flush.get() < (*msg_grid.ptr()).rows {
                msg_scrolled.get() - msg_scrolled_at_flush.get()
            } else {
                (*msg_grid.ptr()).rows
            };
        if pos_delta > 0 as ::core::ffi::c_int {
            ui_ext_msg_set_pos(msg_grid_pos.get(), true_0 != 0);
        }
        let mut to_scroll: ::core::ffi::c_int = delta - pos_delta - msg_grid_scroll_discount.get();
        '_c2rust_label_0: {
            if to_scroll >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"to_scroll >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2697 as ::core::ffi::c_uint,
                    b"void msg_scroll_flush(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if to_scroll > 0 as ::core::ffi::c_int && msg_grid_pos.get() == 0 as ::core::ffi::c_int {
            ui_call_grid_scroll(
                (*msg_grid.ptr()).handle as Integer,
                0 as Integer,
                Rows.get() as Integer,
                0 as Integer,
                Columns.get() as Integer,
                to_scroll as Integer,
                0 as Integer,
            );
        }
        let mut i: ::core::ffi::c_int = if Rows.get()
            - (if delta > 1 as ::core::ffi::c_int {
                delta
            } else {
                1 as ::core::ffi::c_int
            })
            > 0 as ::core::ffi::c_int
        {
            Rows.get()
                - (if delta > 1 as ::core::ffi::c_int {
                    delta
                } else {
                    1 as ::core::ffi::c_int
                })
        } else {
            0 as ::core::ffi::c_int
        };
        while i < Rows.get() {
            let mut row: ::core::ffi::c_int = i - msg_grid_pos.get();
            '_c2rust_label_1: {
                if row >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"row >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2707 as ::core::ffi::c_uint,
                        b"void msg_scroll_flush(void)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            ui_line(
                msg_grid.ptr(),
                row,
                false_0 != 0,
                0 as ::core::ffi::c_int,
                *(*msg_grid.ptr()).dirty_col.offset(row as isize),
                (*msg_grid.ptr()).cols,
                *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
                false_0 != 0,
            );
            *(*msg_grid.ptr()).dirty_col.offset(row as isize) = 0 as ::core::ffi::c_int;
            i += 1;
        }
    }
    msg_scrolled_at_flush.set(msg_scrolled.get());
    msg_grid_scroll_discount.set(0 as ::core::ffi::c_int);
    msg_grid_pos_at_flush.set(msg_grid_pos.get());
}
pub unsafe extern "C" fn msg_reset_scroll() {
    if ui_has(kUIMessages) {
        return;
    }
    (*msg_grid.ptr()).throttled = false_0 != 0;
    msg_grid_set_pos(Rows.get() - p_ch.get() as ::core::ffi::c_int, false_0 != 0);
    clear_cmdline.set(true_0 != 0);
    if !(*msg_grid.ptr()).chars.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i
            < (if msg_scrollsize() < (*msg_grid.ptr()).rows {
                msg_scrollsize()
            } else {
                (*msg_grid.ptr()).rows
            })
        {
            grid_clear_line(
                msg_grid.ptr(),
                *(*msg_grid.ptr()).line_offset.offset(i as isize),
                (*msg_grid.ptr()).cols,
                false_0 != 0,
            );
            i += 1;
        }
    }
    msg_scrolled.set(0 as ::core::ffi::c_int);
    msg_scrolled_at_flush.set(0 as ::core::ffi::c_int);
    msg_grid_scroll_discount.set(0 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn msg_ui_refresh() {
    if ui_has(kUIMultigrid) as ::core::ffi::c_int != 0 && !(*msg_grid.ptr()).chars.is_null() {
        ui_call_grid_resize(
            (*msg_grid.ptr()).handle as Integer,
            (*msg_grid.ptr()).cols as Integer,
            (*msg_grid.ptr()).rows as Integer,
        );
        ui_ext_msg_set_pos(msg_grid_pos.get(), msg_scrolled.get() != 0);
    }
}
pub unsafe extern "C" fn msg_ui_flush() {
    if ui_has(kUIMultigrid) as ::core::ffi::c_int != 0
        && !(*msg_grid.ptr()).chars.is_null()
        && (*msg_grid.ptr()).pending_comp_index_update as ::core::ffi::c_int != 0
    {
        ui_ext_msg_set_pos(msg_grid_pos.get(), msg_scrolled.get() != 0);
    }
}
unsafe extern "C" fn inc_msg_scrolled() {
    if *get_vim_var_str(VV_SCROLLSTART) as ::core::ffi::c_int == NUL {
        let mut p: String_0 = String_0 {
            data: (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_name,
            size: 0,
        };
        let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if p.data.is_null() {
            p = cstr_as_string(gettext(b"Unknown\0".as_ptr() as *const ::core::ffi::c_char));
        } else {
            let mut tofreesize: size_t = strlen(p.data).wrapping_add(40 as size_t);
            tofree = xmalloc(tofreesize) as *mut ::core::ffi::c_char;
            p.size = vim_snprintf_safelen(
                tofree,
                tofreesize,
                gettext(b"%s line %ld\0".as_ptr() as *const ::core::ffi::c_char),
                p.data,
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum as int64_t,
            );
            p.data = tofree;
        }
        set_vim_var_string(VV_SCROLLSTART, p.data, p.size as ptrdiff_t);
        xfree(tofree as *mut ::core::ffi::c_void);
    }
    (*msg_scrolled.ptr()) += 1;
    set_must_redraw(UPD_VALID);
}
static last_msgchunk: GlobalCell<*mut msgchunk_T> =
    GlobalCell::new(::core::ptr::null_mut::<msgchunk_T>());
static do_clear_sb_text: GlobalCell<sb_clear_T> = GlobalCell::new(SB_CLEAR_NONE);
unsafe extern "C" fn store_sb_text(
    mut sb_str: *mut *const ::core::ffi::c_char,
    mut s: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut sb_col: *mut ::core::ffi::c_int,
    mut finish: ::core::ffi::c_int,
) {
    let mut mp: *mut msgchunk_T = ::core::ptr::null_mut::<msgchunk_T>();
    if do_clear_sb_text.get() as ::core::ffi::c_uint
        == SB_CLEAR_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
        || do_clear_sb_text.get() as ::core::ffi::c_uint
            == SB_CLEAR_CMDLINE_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        clear_sb_text(
            do_clear_sb_text.get() as ::core::ffi::c_uint
                == SB_CLEAR_ALL as ::core::ffi::c_int as ::core::ffi::c_uint,
        );
        msg_sb_eol();
        if do_clear_sb_text.get() as ::core::ffi::c_uint
            == SB_CLEAR_CMDLINE_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            && s > *sb_str
            && **sb_str as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
        {
            *sb_str = (*sb_str).offset(1);
        }
        do_clear_sb_text.set(SB_CLEAR_NONE);
    }
    if s > *sb_str {
        mp = xmalloc(
            (28 as size_t)
                .wrapping_add(s.offset_from(*sb_str) as size_t)
                .wrapping_add(1 as size_t),
        ) as *mut msgchunk_T;
        (*mp).sb_eol = finish as ::core::ffi::c_char;
        (*mp).sb_msg_col = *sb_col;
        (*mp).sb_hl_id = hl_id;
        memcpy(
            &raw mut (*mp).sb_text as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            *sb_str as *const ::core::ffi::c_void,
            s.offset_from(*sb_str) as size_t,
        );
        *(&raw mut (*mp).sb_text as *mut ::core::ffi::c_char)
            .offset(s.offset_from(*sb_str) as isize) = NUL as ::core::ffi::c_char;
        if (*last_msgchunk.ptr()).is_null() {
            last_msgchunk.set(mp);
            (*mp).sb_prev = ::core::ptr::null_mut::<msgchunk_T>();
        } else {
            (*mp).sb_prev = last_msgchunk.get();
            (*last_msgchunk.get()).sb_next = mp;
            last_msgchunk.set(mp);
        }
        (*mp).sb_next = ::core::ptr::null_mut::<msgchunk_T>();
    } else if finish != 0 && !(*last_msgchunk.ptr()).is_null() {
        (*last_msgchunk.get()).sb_eol = true_0 as ::core::ffi::c_char;
    }
    *sb_str = s;
    *sb_col = 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn may_clear_sb_text() {
    msg_ext_ui_flush();
    do_clear_sb_text.set(SB_CLEAR_ALL);
    do_clear_hist_temp.set(true_0 != 0);
}
pub unsafe extern "C" fn sb_text_start_cmdline() {
    if do_clear_sb_text.get() as ::core::ffi::c_uint
        == SB_CLEAR_CMDLINE_BUSY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        sb_text_restart_cmdline();
    } else {
        msg_sb_eol();
        do_clear_sb_text.set(SB_CLEAR_CMDLINE_BUSY);
    };
}
pub unsafe extern "C" fn sb_text_restart_cmdline() {
    do_clear_sb_text.set(SB_CLEAR_CMDLINE_BUSY);
    if (*last_msgchunk.ptr()).is_null() || (*last_msgchunk.get()).sb_eol as ::core::ffi::c_int != 0
    {
        return;
    }
    let mut tofree: *mut msgchunk_T = msg_sb_start(last_msgchunk.get());
    last_msgchunk.set((*tofree).sb_prev);
    if !(*last_msgchunk.ptr()).is_null() {
        (*last_msgchunk.get()).sb_next = ::core::ptr::null_mut::<msgchunk_T>();
    }
    while !tofree.is_null() {
        let mut tofree_next: *mut msgchunk_T = (*tofree).sb_next;
        xfree(tofree as *mut ::core::ffi::c_void);
        tofree = tofree_next;
    }
}
pub unsafe extern "C" fn sb_text_end_cmdline() {
    do_clear_sb_text.set(SB_CLEAR_CMDLINE_DONE);
}
pub unsafe extern "C" fn clear_sb_text(mut all: bool) {
    let mut mp: *mut msgchunk_T = ::core::ptr::null_mut::<msgchunk_T>();
    let mut lastp: *mut *mut msgchunk_T = ::core::ptr::null_mut::<*mut msgchunk_T>();
    if all {
        lastp = last_msgchunk.ptr();
    } else {
        if (*last_msgchunk.ptr()).is_null() {
            return;
        }
        lastp = &raw mut (*(msg_sb_start
            as unsafe extern "C" fn(*mut msgchunk_T) -> *mut msgchunk_T)(
            last_msgchunk.get()
        ))
        .sb_prev;
    }
    while !(*lastp).is_null() {
        mp = (**lastp).sb_prev;
        xfree(*lastp as *mut ::core::ffi::c_void);
        *lastp = mp;
    }
}
pub unsafe extern "C" fn show_sb_text() {
    if ui_has(kUIMessages) {
        let mut ea: exarg_T = exarg {
            arg: b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            arglens: ::core::ptr::null_mut::<size_t>(),
            argc: 0,
            nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdidx: CMD_append,
            argt: 0,
            skip: true_0,
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
        ex_messages(&raw mut ea);
        return;
    }
    let mut mp: *mut msgchunk_T = msg_sb_start(last_msgchunk.get());
    if mp.is_null() || (*mp).sb_prev.is_null() {
        vim_beep(kOptBoFlagMess as ::core::ffi::c_int as ::core::ffi::c_uint);
    } else {
        do_more_prompt('G' as ::core::ffi::c_int);
        wait_return(false_0);
    };
}
unsafe extern "C" fn msg_sb_start(mut mps: *mut msgchunk_T) -> *mut msgchunk_T {
    let mut mp: *mut msgchunk_T = mps;
    while !mp.is_null() && !(*mp).sb_prev.is_null() && (*(*mp).sb_prev).sb_eol == 0 {
        mp = (*mp).sb_prev;
    }
    return mp;
}
pub unsafe extern "C" fn msg_sb_eol() {
    if !(*last_msgchunk.ptr()).is_null() {
        (*last_msgchunk.get()).sb_eol = true_0 as ::core::ffi::c_char;
    }
}
unsafe extern "C" fn disp_sb_line(
    mut row: ::core::ffi::c_int,
    mut smp: *mut msgchunk_T,
) -> *mut msgchunk_T {
    let mut mp: *mut msgchunk_T = smp;
    loop {
        msg_row.set(row);
        msg_col.set((*mp).sb_msg_col);
        let mut p: *mut ::core::ffi::c_char = &raw mut (*mp).sb_text as *mut ::core::ffi::c_char;
        msg_puts_display(p, -1 as ::core::ffi::c_int, (*mp).sb_hl_id, true_0);
        if (*mp).sb_eol as ::core::ffi::c_int != 0 || (*mp).sb_next.is_null() {
            break;
        }
        mp = (*mp).sb_next;
    }
    return (*mp).sb_next;
}
pub unsafe extern "C" fn msg_use_printf() -> ::core::ffi::c_int {
    return (!embedded_mode.get() && ui_active() == 0 && !ui_has(kUIMessages))
        as ::core::ffi::c_int;
}
unsafe extern "C" fn msg_puts_printf(mut str: *const ::core::ffi::c_char, maxlen: ptrdiff_t) {
    let mut s: *const ::core::ffi::c_char = str;
    let mut buf: [::core::ffi::c_char; 7] = [0; 7];
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*on_print.ptr()).type_0 as ::core::ffi::c_uint
        != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut argv: [typval_T; 1] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 1];
        argv[0 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
        argv[0 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
        argv[0 as ::core::ffi::c_int as usize].vval.v_string = str as *mut ::core::ffi::c_char;
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        callback_call(
            on_print.ptr(),
            1 as ::core::ffi::c_int,
            &raw mut argv as *mut typval_T,
            &raw mut rettv,
        );
        tv_clear(&raw mut rettv);
        return;
    }
    while (maxlen < 0 as ptrdiff_t || s.offset_from(str) < maxlen)
        && *s as ::core::ffi::c_int != NUL
    {
        let mut len: ::core::ffi::c_int = utf_ptr2len(s);
        if !(silent_mode.get() as ::core::ffi::c_int != 0 && p_verbose.get() == 0 as OptInt) {
            p = (&raw mut buf as *mut ::core::ffi::c_char).offset(0 as ::core::ffi::c_int as isize);
            if *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                && !info_message.get()
                && !silent_mode.get()
                && !headless_mode.get()
            {
                let c2rust_fresh6 = p;
                p = p.offset(1);
                *c2rust_fresh6 = '\r' as ::core::ffi::c_char;
            }
            memcpy(
                p as *mut ::core::ffi::c_void,
                s as *const ::core::ffi::c_void,
                len as size_t,
            );
            *p.offset(len as isize) = NUL as ::core::ffi::c_char;
            if info_message.get() {
                printf(
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    &raw mut buf as *mut ::core::ffi::c_char,
                );
            } else {
                fprintf(
                    stderr,
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    &raw mut buf as *mut ::core::ffi::c_char,
                );
            }
        }
        let mut cw: ::core::ffi::c_int = utf_char2cells(utf_ptr2char(s));
        if *s as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
            || *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
        {
            msg_col.set(0 as ::core::ffi::c_int);
            msg_didout.set(false_0 != 0);
        } else {
            (*msg_col.ptr()) += cw;
            msg_didout.set(true_0 != 0);
        }
        s = s.offset(len as isize);
    }
}
unsafe extern "C" fn do_more_prompt(mut typed_char: ::core::ffi::c_int) -> bool {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut used_typed_char: ::core::ffi::c_int = typed_char;
    let mut oldState: ::core::ffi::c_int = State.get();
    let mut c: ::core::ffi::c_int = 0;
    let mut retval: bool = false_0 != 0;
    let mut to_redraw: bool = false_0 != 0;
    let mut mp_last: *mut msgchunk_T = ::core::ptr::null_mut::<msgchunk_T>();
    let mut mp: *mut msgchunk_T = ::core::ptr::null_mut::<msgchunk_T>();
    let mut no_need_more: bool =
        headless_mode.get() as ::core::ffi::c_int != 0 && !embedded_mode.get() && ui_active() == 0;
    if no_need_more as ::core::ffi::c_int != 0
        || entered.get() as ::core::ffi::c_int != 0
        || State.get() == MODE_HITRETURN && typed_char == 0 as ::core::ffi::c_int
    {
        return false_0 != 0;
    }
    entered.set(true_0 != 0);
    if typed_char == 'G' as ::core::ffi::c_int {
        mp_last = msg_sb_start(last_msgchunk.get());
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < Rows.get() - 2 as ::core::ffi::c_int
            && !mp_last.is_null()
            && !(*mp_last).sb_prev.is_null()
        {
            mp_last = msg_sb_start((*mp_last).sb_prev);
            i += 1;
        }
    }
    State.set(MODE_ASKMORE);
    setmouse();
    if typed_char == NUL {
        msg_moremsg(false_0 != 0);
    }
    's_528: loop {
        if used_typed_char != NUL {
            c = used_typed_char;
            used_typed_char = NUL;
        } else {
            c = get_keystroke(resize_events.get());
        }
        let mut toscroll: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        's_276: {
            match c {
                BS | K_BS | 107 | K_UP => {
                    toscroll = -1 as ::core::ffi::c_int;
                    break 's_276;
                }
                CAR | NL | 106 | K_DOWN => {
                    toscroll = 1 as ::core::ffi::c_int;
                    break 's_276;
                }
                117 => {
                    toscroll = -(Rows.get() / 2 as ::core::ffi::c_int);
                    break 's_276;
                }
                100 => {
                    toscroll = Rows.get() / 2 as ::core::ffi::c_int;
                    break 's_276;
                }
                98 | Ctrl_B | K_PAGEUP => {
                    toscroll = -(Rows.get() - 1 as ::core::ffi::c_int);
                    break 's_276;
                }
                32 | 102 | Ctrl_F | K_PAGEDOWN | -11517 => {
                    toscroll = Rows.get() - 1 as ::core::ffi::c_int;
                    break 's_276;
                }
                103 => {
                    toscroll = -999999 as ::core::ffi::c_int;
                    break 's_276;
                }
                71 => {
                    toscroll = 999999 as ::core::ffi::c_int;
                    lines_left.set(999999 as ::core::ffi::c_int);
                    break 's_276;
                }
                58 => {
                    if confirm_msg_used.get() == 0 {
                        typeahead_noflush(':' as ::core::ffi::c_int);
                        cmdline_row.set(Rows.get() - 1 as ::core::ffi::c_int);
                        skip_redraw.set(true_0 != 0);
                        need_wait_return.set(false_0 != 0);
                    }
                }
                113 | Ctrl_C | ESC => {}
                K_EVENT => {
                    multiqueue_process_events(resize_events.get());
                    to_redraw = true_0 != 0;
                    break 's_276;
                }
                _ => {
                    msg_moremsg(true_0 != 0);
                    continue 's_528;
                }
            }
            if confirm_msg_used.get() != 0 {
                retval = true_0 != 0;
            } else {
                got_int.set(true_0 != 0);
                quit_more.set(true_0 != 0);
            }
            lines_left.set(Rows.get() - 1 as ::core::ffi::c_int);
        }
        '_c2rust_label: {
            if toscroll == 0 as ::core::ffi::c_int || !to_redraw {
            } else {
                __assert_fail(
                    b"(toscroll == 0) || !to_redraw\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3168 as ::core::ffi::c_uint,
                    b"_Bool do_more_prompt(int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if !(toscroll != 0 as ::core::ffi::c_int || to_redraw as ::core::ffi::c_int != 0) {
            break;
        }
        if toscroll < 0 as ::core::ffi::c_int || to_redraw as ::core::ffi::c_int != 0 {
            if mp_last.is_null() {
                mp = msg_sb_start(last_msgchunk.get());
            } else if !(*mp_last).sb_prev.is_null() {
                mp = msg_sb_start((*mp_last).sb_prev);
            } else {
                mp = ::core::ptr::null_mut::<msgchunk_T>();
            }
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < Rows.get() - 2 as ::core::ffi::c_int
                && !mp.is_null()
                && !(*mp).sb_prev.is_null()
            {
                mp = msg_sb_start((*mp).sb_prev);
                i_0 += 1;
            }
            if !mp.is_null() && (!(*mp).sb_prev.is_null() || to_redraw as ::core::ffi::c_int != 0) {
                let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_1 > toscroll {
                    if mp.is_null() || (*mp).sb_prev.is_null() {
                        break;
                    }
                    mp = msg_sb_start((*mp).sb_prev);
                    if mp_last.is_null() {
                        mp_last = msg_sb_start(last_msgchunk.get());
                    } else {
                        mp_last = msg_sb_start((*mp_last).sb_prev);
                    }
                    i_1 -= 1;
                }
                if toscroll == -1 as ::core::ffi::c_int && !to_redraw {
                    grid_ins_lines(
                        msg_grid.ptr(),
                        0 as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                        Rows.get(),
                        0 as ::core::ffi::c_int,
                        Columns.get(),
                    );
                    grid_clear(
                        msg_grid_adj.ptr(),
                        0 as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        Columns.get(),
                        *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
                    );
                    disp_sb_line(0 as ::core::ffi::c_int, mp);
                } else {
                    grid_clear(
                        msg_grid_adj.ptr(),
                        0 as ::core::ffi::c_int,
                        Rows.get(),
                        0 as ::core::ffi::c_int,
                        Columns.get(),
                        *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
                    );
                    let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while !mp.is_null() && i_2 < Rows.get() - 1 as ::core::ffi::c_int {
                        mp = disp_sb_line(i_2, mp);
                        (*msg_scrolled.ptr()) += 1;
                        i_2 += 1;
                    }
                    to_redraw = false_0 != 0;
                }
                toscroll = 0 as ::core::ffi::c_int;
            }
        } else {
            if cmdline_row.get() >= Rows.get() && !ui_has(kUIMessages) {
                msg_scroll_up(true_0 != 0, false_0 != 0);
                (*msg_scrolled.ptr()) += 1;
            }
            while toscroll > 0 as ::core::ffi::c_int && !mp_last.is_null() {
                if msg_do_throttle() as ::core::ffi::c_int != 0 && !(*msg_grid.ptr()).throttled {
                    (*msg_scrolled_at_flush.ptr()) -= 1;
                    (*msg_grid_scroll_discount.ptr()) += 1;
                }
                msg_scroll_up(true_0 != 0, false_0 != 0);
                inc_msg_scrolled();
                grid_clear(
                    msg_grid_adj.ptr(),
                    Rows.get() - 2 as ::core::ffi::c_int,
                    Rows.get() - 1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    Columns.get(),
                    *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
                );
                mp_last = disp_sb_line(Rows.get() - 2 as ::core::ffi::c_int, mp_last);
                toscroll -= 1;
            }
        }
        if toscroll <= 0 as ::core::ffi::c_int {
            grid_clear(
                msg_grid_adj.ptr(),
                Rows.get() - 1 as ::core::ffi::c_int,
                Rows.get(),
                0 as ::core::ffi::c_int,
                Columns.get(),
                *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
            );
            msg_moremsg(false_0 != 0);
        } else {
            lines_left.set(toscroll);
            break;
        }
    }
    grid_clear(
        msg_grid_adj.ptr(),
        Rows.get() - 1 as ::core::ffi::c_int,
        Rows.get(),
        0 as ::core::ffi::c_int,
        Columns.get(),
        *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
    );
    redraw_cmdline.set(true_0 != 0);
    clear_cmdline.set(false_0 != 0);
    mode_displayed.set(false_0 != 0);
    State.set(oldState);
    setmouse();
    if quit_more.get() {
        msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
        msg_col.set(0 as ::core::ffi::c_int);
    }
    entered.set(false_0 != 0);
    return retval;
}
unsafe extern "C" fn msg_moremsg(mut full: bool) {
    let mut attr: ::core::ffi::c_int = hl_combine_attr(
        *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
        *(*hl_attr_active.ptr()).offset(HLF_M as isize),
    );
    grid_line_start(msg_grid_adj.ptr(), Rows.get() - 1 as ::core::ffi::c_int);
    let mut len: ::core::ffi::c_int = grid_line_puts(
        0 as ::core::ffi::c_int,
        gettext(b"-- More --\0".as_ptr() as *const ::core::ffi::c_char),
        -1 as ::core::ffi::c_int,
        attr,
    );
    if full {
        len += grid_line_puts(
            len,
            gettext(
                b" SPACE/d/j: screen/page/line down, b/u/k: up, q: quit \0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            -1 as ::core::ffi::c_int,
            attr,
        );
    }
    grid_line_cursor_goto(len);
    grid_line_flush();
}
pub unsafe extern "C" fn repeat_message() {
    if ui_has(kUIMessages) {
        return;
    }
    if State.get() == MODE_ASKMORE {
        msg_moremsg(true_0 != 0);
        msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
    } else if State.get() & MODE_CMDLINE != 0 && !(*confirm_msg.ptr()).is_null() {
        display_confirm_msg();
        msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
    } else if State.get() == MODE_EXTERNCMD {
        ui_cursor_goto(msg_row.get(), msg_col.get());
    } else if State.get() == MODE_HITRETURN || State.get() == MODE_SETWSIZE {
        if msg_row.get() == Rows.get() - 1 as ::core::ffi::c_int {
            msg_didout.set(false_0 != 0);
            msg_col.set(0 as ::core::ffi::c_int);
            msg_clr_eos();
        }
        hit_return_msg(false_0 != 0);
        msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
    }
}
pub unsafe extern "C" fn msg_clr_eos() {
    if msg_silent.get() == 0 as ::core::ffi::c_int {
        msg_clr_eos_force();
    }
}
pub unsafe extern "C" fn msg_clr_eos_force() {
    if ui_has(kUIMessages) {
        return;
    }
    let mut msg_startcol: ::core::ffi::c_int = if cmdmsg_rl.get() as ::core::ffi::c_int != 0 {
        0 as ::core::ffi::c_int
    } else {
        msg_col.get()
    };
    let mut msg_endcol: ::core::ffi::c_int = if cmdmsg_rl.get() as ::core::ffi::c_int != 0 {
        Columns.get() - msg_col.get()
    } else {
        Columns.get()
    };
    if !(*msg_grid.ptr()).chars.is_null() && msg_row.get() < msg_grid_pos.get() {
        msg_grid_validate();
        if msg_row.get() < msg_grid_pos.get() {
            msg_row.set(msg_grid_pos.get());
        }
    }
    grid_clear(
        msg_grid_adj.ptr(),
        msg_row.get(),
        msg_row.get() + 1 as ::core::ffi::c_int,
        msg_startcol,
        msg_endcol,
        *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
    );
    grid_clear(
        msg_grid_adj.ptr(),
        msg_row.get() + 1 as ::core::ffi::c_int,
        Rows.get(),
        0 as ::core::ffi::c_int,
        Columns.get(),
        *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
    );
    redraw_cmdline.set(true_0 != 0);
    if msg_row.get() < Rows.get() - 1 as ::core::ffi::c_int
        || msg_col.get() == 0 as ::core::ffi::c_int
    {
        clear_cmdline.set(false_0 != 0);
        mode_displayed.set(false_0 != 0);
        cmdline_was_last_drawn.set(false_0 != 0);
    }
}
pub unsafe extern "C" fn msg_clr_cmdline() {
    msg_row.set(cmdline_row.get());
    msg_col.set(0 as ::core::ffi::c_int);
    msg_clr_eos_force();
}
pub unsafe extern "C" fn msg_end() -> bool {
    if !exiting.get()
        && need_wait_return.get() as ::core::ffi::c_int != 0
        && State.get() & MODE_CMDLINE == 0
    {
        wait_return(false_0);
        return false_0 != 0;
    }
    msg_ext_ui_flush();
    return true_0 != 0;
}
unsafe extern "C" fn msg_ext_init_chunks() -> *mut Array {
    let mut tofree: *mut Array = msg_ext_chunks.get();
    msg_ext_chunks.set(xcalloc(1 as size_t, ::core::mem::size_of::<Array>()) as *mut Array);
    msg_col.set(0 as ::core::ffi::c_int);
    return tofree;
}
pub unsafe extern "C" fn msg_ext_ui_flush() {
    if !ui_has(kUIMessages) {
        msg_ext_kind.set(::core::ptr::null::<::core::ffi::c_char>());
        return;
    } else if msg_ext_skip_flush.get() {
        return;
    }
    msg_ext_emit_chunk();
    if (*msg_ext_chunks.get()).size > 0 as size_t {
        let mut tofree: *mut Array = msg_ext_init_chunks();
        ui_call_msg_show(
            cstr_as_string(msg_ext_kind.get()),
            *tofree,
            msg_ext_overwrite.get() as Boolean,
            msg_ext_history.get() as Boolean,
            msg_ext_append.get() as Boolean,
            msg_ext_id.get(),
            cstr_as_string(msg_ext_trigger.get()),
        );
        if msg_ext_history.get() {
            api_free_array(*tofree);
        } else {
            let mut msg_0: HlMessage = HlMessage {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<HlMessageChunk>(),
            };
            let mut i: size_t = 0 as size_t;
            while i < (*tofree).size {
                let mut chunk: *mut Object = (*(*tofree).items.offset(i as isize)).data.array.items;
                if msg_0.size == msg_0.capacity {
                    msg_0.capacity = if msg_0.capacity != 0 {
                        msg_0.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    msg_0.items = xrealloc(
                        msg_0.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(msg_0.capacity),
                    ) as *mut HlMessageChunk;
                } else {
                };
                let c2rust_fresh0 = msg_0.size;
                msg_0.size = msg_0.size.wrapping_add(1);
                *msg_0.items.offset(c2rust_fresh0 as isize) = HlMessageChunk {
                    text: (*chunk.offset(1 as ::core::ffi::c_int as isize))
                        .data
                        .string,
                    hl_id: (*chunk.offset(2 as ::core::ffi::c_int as isize))
                        .data
                        .integer as ::core::ffi::c_int,
                };
                xfree(chunk as *mut ::core::ffi::c_void);
                i = i.wrapping_add(1);
            }
            xfree((*tofree).items as *mut ::core::ffi::c_void);
            msg_hist_add_multihl(msg_0, true_0 != 0, ::core::ptr::null_mut::<MessageData>());
        }
        xfree(tofree as *mut ::core::ffi::c_void);
        msg_ext_overwrite.set(false_0 != 0);
        msg_ext_history.set(false_0 != 0);
        msg_ext_append.set(false_0 != 0);
        msg_ext_kind.set(::core::ptr::null::<::core::ffi::c_char>());
        (*msg_id_next.ptr()) += ((*msg_ext_id.ptr()).data.integer == msg_id_next.get())
            as ::core::ffi::c_int as int64_t;
        msg_ext_id.set(object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed_11 {
                integer: msg_id_next.get(),
            },
        });
    }
}
pub unsafe extern "C" fn msg_ext_flush_showmode() {
    static clear: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if ui_has(kUIMessages) as ::core::ffi::c_int != 0
        && (msg_ext_last_attr.get() != -1 as sattr_T || clear.get() as ::core::ffi::c_int != 0)
    {
        clear.set(msg_ext_last_attr.get() != -1 as sattr_T);
        msg_ext_emit_chunk();
        let mut tofree: *mut Array = msg_ext_init_chunks();
        ui_call_msg_showmode(*tofree);
        api_free_array(*tofree);
        xfree(tofree as *mut ::core::ffi::c_void);
    }
}
pub unsafe extern "C" fn msg_check() {
    if ui_has(kUIMessages) {
        return;
    }
    if msg_row.get() == Rows.get() - 1 as ::core::ffi::c_int && msg_col.get() >= sc_col.get() {
        need_wait_return.set(true_0 != 0);
        redraw_cmdline.set(true_0 != 0);
    }
}
unsafe extern "C" fn redir_write(str: *const ::core::ffi::c_char, maxlen: ptrdiff_t) {
    let mut s: *const ::core::ffi::c_char = str;
    if maxlen == 0 as ptrdiff_t {
        return;
    }
    if redir_off.get() {
        return;
    }
    if *p_vfile.get() as ::core::ffi::c_int != NUL && (*verbose_fd.ptr()).is_null() {
        verbose_open();
    }
    if redirecting() != 0 {
        if *s as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
            && *s as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
        {
            while redir_col.get() < msg_col.get() {
                if !(*capture_ga.ptr()).is_null() {
                    ga_concat_len(
                        capture_ga.get(),
                        b" \0".as_ptr() as *const ::core::ffi::c_char,
                        1 as size_t,
                    );
                }
                if redir_reg.get() != 0 {
                    write_reg_contents(
                        redir_reg.get(),
                        b" \0".as_ptr() as *const ::core::ffi::c_char,
                        1 as ssize_t,
                        true_0,
                    );
                } else if redir_vname.get() {
                    var_redir_str(
                        b" \0".as_ptr() as *const ::core::ffi::c_char,
                        -1 as ::core::ffi::c_int,
                    );
                } else if !(*redir_fd.ptr()).is_null() {
                    fputs(
                        b" \0".as_ptr() as *const ::core::ffi::c_char,
                        redir_fd.get(),
                    );
                }
                if !(*verbose_fd.ptr()).is_null() {
                    fputs(
                        b" \0".as_ptr() as *const ::core::ffi::c_char,
                        verbose_fd.get(),
                    );
                }
                (*redir_col.ptr()) += 1;
            }
        }
        let mut len: size_t = if maxlen == -1 as ptrdiff_t {
            strlen(s)
        } else {
            maxlen as size_t
        };
        if !(*capture_ga.ptr()).is_null() {
            ga_concat_len(capture_ga.get(), str, len);
        }
        if redir_reg.get() != 0 {
            write_reg_contents(redir_reg.get(), s, len as ssize_t, true_0);
        }
        if redir_vname.get() {
            var_redir_str(s, maxlen as ::core::ffi::c_int);
        }
        while *s as ::core::ffi::c_int != NUL
            && (maxlen < 0 as ptrdiff_t
                || (s.offset_from(str) as ::core::ffi::c_int as ptrdiff_t) < maxlen)
        {
            if redir_reg.get() == 0 && !redir_vname.get() && (*capture_ga.ptr()).is_null() {
                if !(*redir_fd.ptr()).is_null() {
                    putc(*s as ::core::ffi::c_int, redir_fd.get());
                }
            }
            if !(*verbose_fd.ptr()).is_null() {
                putc(*s as ::core::ffi::c_int, verbose_fd.get());
            }
            if *s as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
                || *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
            {
                redir_col.set(0 as ::core::ffi::c_int);
            } else if *s as ::core::ffi::c_int == '\t' as ::core::ffi::c_int {
                (*redir_col.ptr()) +=
                    8 as ::core::ffi::c_int - redir_col.get() % 8 as ::core::ffi::c_int;
            } else {
                (*redir_col.ptr()) += 1;
            }
            s = s.offset(1);
        }
        if msg_silent.get() != 0 as ::core::ffi::c_int {
            msg_col.set(redir_col.get());
        }
    }
}
pub unsafe extern "C" fn redirecting() -> ::core::ffi::c_int {
    return (!(*redir_fd.ptr()).is_null()
        || *p_vfile.get() as ::core::ffi::c_int != NUL
        || redir_reg.get() != 0
        || redir_vname.get() as ::core::ffi::c_int != 0
        || !(*capture_ga.ptr()).is_null()) as ::core::ffi::c_int;
}
static pre_verbose_kind: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null::<::core::ffi::c_char>());
static verbose_kind: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"verbose\0".as_ptr() as *const ::core::ffi::c_char);
pub unsafe extern "C" fn verbose_enter() {
    if *p_vfile.get() as ::core::ffi::c_int != NUL {
        (*msg_silent.ptr()) += 1;
    }
    if !msg_ext_skip_verbose.get() {
        if msg_ext_kind.get() != verbose_kind.get() {
            pre_verbose_kind.set(msg_ext_kind.get());
        }
        msg_ext_set_kind(b"verbose\0".as_ptr() as *const ::core::ffi::c_char);
    }
    msg_ext_skip_verbose.set(false_0 != 0);
}
pub unsafe extern "C" fn verbose_leave() {
    if *p_vfile.get() as ::core::ffi::c_int != NUL {
        (*msg_silent.ptr()) -= 1;
        if msg_silent.get() < 0 as ::core::ffi::c_int {
            msg_silent.set(0 as ::core::ffi::c_int);
        }
    }
    if !(*pre_verbose_kind.ptr()).is_null() {
        msg_ext_set_kind(pre_verbose_kind.get());
        pre_verbose_kind.set(::core::ptr::null::<::core::ffi::c_char>());
    }
}
pub unsafe extern "C" fn verbose_enter_scroll() {
    verbose_enter();
    if *p_vfile.get() as ::core::ffi::c_int == NUL {
        msg_scroll.set(true_0);
    }
}
pub unsafe extern "C" fn verbose_leave_scroll() {
    verbose_leave();
    if *p_vfile.get() as ::core::ffi::c_int == NUL {
        cmdline_row.set(msg_row.get());
    }
}
pub unsafe extern "C" fn verbose_stop() {
    if !(*verbose_fd.ptr()).is_null() {
        fclose(verbose_fd.get());
        verbose_fd.set(::core::ptr::null_mut::<FILE>());
    }
    verbose_did_open.set(false_0 != 0);
}
pub unsafe extern "C" fn verbose_open() -> ::core::ffi::c_int {
    if (*verbose_fd.ptr()).is_null() && !verbose_did_open.get() {
        verbose_did_open.set(true_0 != 0);
        verbose_fd.set(os_fopen(
            p_vfile.get(),
            b"a\0".as_ptr() as *const ::core::ffi::c_char,
        ));
        if (*verbose_fd.ptr()).is_null() {
            semsg(
                gettext(&raw const e_notopen as *const ::core::ffi::c_char),
                p_vfile.get(),
            );
            return FAIL;
        }
    }
    return OK;
}
pub unsafe extern "C" fn give_warning(
    mut message: *const ::core::ffi::c_char,
    mut hl: bool,
    mut hist: bool,
) {
    if msg_silent.get() != 0 as ::core::ffi::c_int {
        return;
    }
    let mut save_msg_hist_off: bool = msg_hist_off.get();
    msg_hist_off.set(!hist);
    (*no_wait_return.ptr()) += 1;
    set_vim_var_string(VV_WARNINGMSG, message, -1 as ptrdiff_t);
    let mut ptr_: *mut *mut ::core::ffi::c_void = keep_msg.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    if hl {
        keep_msg_hl_id.set(HLF_W);
    } else {
        keep_msg_hl_id.set(0 as ::core::ffi::c_int);
    }
    if (*msg_ext_kind.ptr()).is_null() {
        msg_ext_set_kind(b"wmsg\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if msg(message, keep_msg_hl_id.get()) as ::core::ffi::c_int != 0
        && msg_scrolled.get() == 0 as ::core::ffi::c_int
    {
        set_keep_msg(message, keep_msg_hl_id.get());
    }
    msg_didout.set(false_0 != 0);
    msg_nowait.set(true_0 != 0);
    msg_col.set(0 as ::core::ffi::c_int);
    (*no_wait_return.ptr()) -= 1;
    msg_hist_off.set(save_msg_hist_off);
}
pub unsafe extern "C" fn swmsg(
    mut hl: bool,
    fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) {
    let mut args: ::core::ffi::VaList;
    args = c2rust_args.clone();
    vim_vsnprintf(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        fmt,
        args,
    );
    give_warning(IObuff.ptr() as *mut ::core::ffi::c_char, hl, true_0 != 0);
}
pub unsafe extern "C" fn msg_advance(mut col: ::core::ffi::c_int) {
    if msg_silent.get() != 0 as ::core::ffi::c_int {
        msg_col.set(col);
        return;
    }
    col = if col < Columns.get() - 1 as ::core::ffi::c_int {
        col
    } else {
        Columns.get() - 1 as ::core::ffi::c_int
    };
    while msg_col.get() < col {
        msg_putchar(' ' as ::core::ffi::c_int);
    }
}
pub unsafe extern "C" fn do_dialog(
    mut _type_0: ::core::ffi::c_int,
    mut _title: *const ::core::ffi::c_char,
    mut message: *const ::core::ffi::c_char,
    mut buttons: *const ::core::ffi::c_char,
    mut dfltbutton: ::core::ffi::c_int,
    mut _textfield: *const ::core::ffi::c_char,
    mut ex_cmd: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0;
    if silent_mode.get() {
        return dfltbutton;
    }
    let mut save_msg_silent: ::core::ffi::c_int = msg_silent.get();
    let mut oldState: ::core::ffi::c_int = State.get();
    msg_silent.set(0 as ::core::ffi::c_int);
    (*no_wait_return.ptr()) += 1;
    let mut hotkeys: *mut ::core::ffi::c_char =
        msg_show_console_dialog(message, buttons, dfltbutton);
    loop {
        if ui_active() == 0 && input_available() == 0 {
            retval = dfltbutton;
            break;
        } else {
            let mut c: ::core::ffi::c_int = prompt_for_input(
                confirm_buttons.get(),
                HLF_M,
                true_0 != 0,
                ::core::ptr::null_mut::<bool>(),
            );
            match c {
                CAR | NUL => {
                    retval = dfltbutton;
                    break;
                }
                Ctrl_C | ESC => {
                    retval = 0 as ::core::ffi::c_int;
                    break;
                }
                _ => {
                    if c < 0 as ::core::ffi::c_int {
                        msg_didany.set(false_0 != 0);
                        msg_didout.set(msg_didany.get());
                    } else if c == ':' as ::core::ffi::c_int && ex_cmd != 0 {
                        retval = dfltbutton;
                        ins_char_typebuf(
                            ':' as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                            false_0 != 0,
                        );
                        break;
                    } else {
                        c = mb_tolower(c);
                        retval = 1 as ::core::ffi::c_int;
                        i = 0 as ::core::ffi::c_int;
                        while *hotkeys.offset(i as isize) != 0 {
                            if utf_ptr2char(hotkeys.offset(i as isize)) == c {
                                break;
                            }
                            i += utfc_ptr2len(hotkeys.offset(i as isize)) - 1 as ::core::ffi::c_int;
                            retval += 1;
                            i += 1;
                        }
                        if *hotkeys.offset(i as isize) != 0 {
                            break;
                        }
                        msg_didany.set(false_0 != 0);
                        msg_didout.set(msg_didany.get());
                    }
                }
            }
        }
    }
    xfree(hotkeys as *mut ::core::ffi::c_void);
    xfree(confirm_msg.get() as *mut ::core::ffi::c_void);
    confirm_msg.set(::core::ptr::null_mut::<::core::ffi::c_char>());
    msg_silent.set(save_msg_silent);
    State.set(oldState);
    setmouse();
    (*no_wait_return.ptr()) -= 1;
    msg_end_prompt();
    return retval;
}
unsafe extern "C" fn copy_char(
    mut from: *const ::core::ffi::c_char,
    mut to: *mut ::core::ffi::c_char,
    mut lowercase: bool,
) -> ::core::ffi::c_int {
    if lowercase {
        let mut c: ::core::ffi::c_int = mb_tolower(utf_ptr2char(from));
        return utf_char2bytes(c, to);
    }
    let mut len: ::core::ffi::c_int = utfc_ptr2len(from);
    memmove(
        to as *mut ::core::ffi::c_void,
        from as *const ::core::ffi::c_void,
        len as size_t,
    );
    return len;
}
pub const HAS_HOTKEY_LEN: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
unsafe extern "C" fn console_dialog_alloc(
    mut message: *const ::core::ffi::c_char,
    mut buttons: *const ::core::ffi::c_char,
    mut has_hotkey: *mut bool,
) -> *mut ::core::ffi::c_char {
    let mut lenhotkey: ::core::ffi::c_int = MB_MAXBYTES as ::core::ffi::c_int;
    *has_hotkey.offset(0 as ::core::ffi::c_int as isize) = false_0 != 0;
    let mut msg_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut button_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut r: *const ::core::ffi::c_char = buttons;
    while *r != 0 {
        if *r as ::core::ffi::c_int == DLG_BUTTON_SEP as ::core::ffi::c_int {
            button_len += 3 as ::core::ffi::c_int;
            lenhotkey += MB_MAXBYTES as ::core::ffi::c_int;
            if idx < HAS_HOTKEY_LEN - 1 as ::core::ffi::c_int {
                idx += 1;
                *has_hotkey.offset(idx as isize) = false_0 != 0;
            }
        } else if *r as ::core::ffi::c_int == DLG_HOTKEY_CHAR as ::core::ffi::c_int {
            r = r.offset(1);
            button_len += 1;
            if idx < HAS_HOTKEY_LEN - 1 as ::core::ffi::c_int {
                *has_hotkey.offset(idx as isize) = true_0 != 0;
            }
        }
        r = r.offset(utfc_ptr2len(r as *mut ::core::ffi::c_char) as isize);
    }
    msg_len += strlen(message) as ::core::ffi::c_int + 3 as ::core::ffi::c_int;
    button_len += strlen(buttons) as ::core::ffi::c_int + 3 as ::core::ffi::c_int;
    lenhotkey += 1;
    if !*has_hotkey.offset(0 as ::core::ffi::c_int as isize) {
        button_len += 2 as ::core::ffi::c_int;
    }
    confirm_msg.set(xmalloc(msg_len as size_t) as *mut ::core::ffi::c_char);
    snprintf(
        confirm_msg.get(),
        msg_len as size_t,
        if ui_has(kUIMessages) as ::core::ffi::c_int != 0 {
            b"%s\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\n%s\n\0".as_ptr() as *const ::core::ffi::c_char
        },
        message,
    );
    xfree(confirm_buttons.get() as *mut ::core::ffi::c_void);
    confirm_buttons.set(xmalloc(button_len as size_t) as *mut ::core::ffi::c_char);
    return xmalloc(lenhotkey as size_t) as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn msg_show_console_dialog(
    mut message: *const ::core::ffi::c_char,
    mut buttons: *const ::core::ffi::c_char,
    mut dfltbutton: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut has_hotkey: [bool; 30] = [
        false_0 != 0,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
    ];
    let mut hotk: *mut ::core::ffi::c_char =
        console_dialog_alloc(message, buttons, &raw mut has_hotkey as *mut bool);
    copy_confirm_hotkeys(
        buttons,
        dfltbutton,
        &raw mut has_hotkey as *mut bool as *const bool,
        hotk,
    );
    display_confirm_msg();
    return hotk;
}
unsafe extern "C" fn copy_confirm_hotkeys(
    mut buttons: *const ::core::ffi::c_char,
    mut default_button_idx: ::core::ffi::c_int,
    mut has_hotkey: *const bool,
    mut hotkeys_ptr: *mut ::core::ffi::c_char,
) {
    *hotkeys_ptr.offset(copy_char(buttons, hotkeys_ptr, true_0 != 0) as isize) =
        NUL as ::core::ffi::c_char;
    let mut first_hotkey: bool = false_0 != 0;
    if !*has_hotkey.offset(0 as ::core::ffi::c_int as isize) {
        first_hotkey = true_0 != 0;
    }
    let mut msgp: *mut ::core::ffi::c_char = confirm_buttons.get();
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut r: *const ::core::ffi::c_char = buttons;
    while *r != 0 {
        if *r as ::core::ffi::c_int == DLG_BUTTON_SEP as ::core::ffi::c_int {
            let c2rust_fresh39 = msgp;
            msgp = msgp.offset(1);
            *c2rust_fresh39 = ',' as ::core::ffi::c_char;
            let c2rust_fresh40 = msgp;
            msgp = msgp.offset(1);
            *c2rust_fresh40 = ' ' as ::core::ffi::c_char;
            hotkeys_ptr = hotkeys_ptr.offset(strlen(hotkeys_ptr) as isize);
            *hotkeys_ptr.offset(copy_char(
                r.offset(1 as ::core::ffi::c_int as isize),
                hotkeys_ptr,
                true_0 != 0,
            ) as isize) = NUL as ::core::ffi::c_char;
            if default_button_idx != 0 {
                default_button_idx -= 1;
            }
            if idx < HAS_HOTKEY_LEN - 1 as ::core::ffi::c_int && {
                idx += 1;
                !*has_hotkey.offset(idx as isize)
            } {
                first_hotkey = true_0 != 0;
            }
        } else if *r as ::core::ffi::c_int == DLG_HOTKEY_CHAR as ::core::ffi::c_int
            || first_hotkey as ::core::ffi::c_int != 0
        {
            if *r as ::core::ffi::c_int == DLG_HOTKEY_CHAR as ::core::ffi::c_int {
                r = r.offset(1);
            }
            first_hotkey = false_0 != 0;
            if *r as ::core::ffi::c_int == DLG_HOTKEY_CHAR as ::core::ffi::c_int {
                let c2rust_fresh41 = msgp;
                msgp = msgp.offset(1);
                *c2rust_fresh41 = *r;
            } else {
                let c2rust_fresh42 = msgp;
                msgp = msgp.offset(1);
                *c2rust_fresh42 = (if default_button_idx == 1 as ::core::ffi::c_int {
                    '[' as ::core::ffi::c_int
                } else {
                    '(' as ::core::ffi::c_int
                }) as ::core::ffi::c_char;
                msgp = msgp.offset(copy_char(r, msgp, false_0 != 0) as isize);
                let c2rust_fresh43 = msgp;
                msgp = msgp.offset(1);
                *c2rust_fresh43 = (if default_button_idx == 1 as ::core::ffi::c_int {
                    ']' as ::core::ffi::c_int
                } else {
                    ')' as ::core::ffi::c_int
                }) as ::core::ffi::c_char;
                *hotkeys_ptr.offset(copy_char(r, hotkeys_ptr, true_0 != 0) as isize) =
                    NUL as ::core::ffi::c_char;
            }
        } else {
            msgp = msgp.offset(copy_char(r, msgp, false_0 != 0) as isize);
        }
        r = r.offset(utfc_ptr2len(r as *mut ::core::ffi::c_char) as isize);
    }
    let c2rust_fresh44 = msgp;
    msgp = msgp.offset(1);
    *c2rust_fresh44 = ':' as ::core::ffi::c_char;
    let c2rust_fresh45 = msgp;
    msgp = msgp.offset(1);
    *c2rust_fresh45 = ' ' as ::core::ffi::c_char;
    *msgp = NUL as ::core::ffi::c_char;
}
unsafe extern "C" fn display_confirm_msg() {
    (*confirm_msg_used.ptr()) += 1;
    if !(*confirm_msg.ptr()).is_null() {
        msg_ext_set_kind(b"confirm\0".as_ptr() as *const ::core::ffi::c_char);
        msg_puts_hl(confirm_msg.get(), HLF_M, false_0 != 0);
    }
    (*confirm_msg_used.ptr()) -= 1;
}
pub unsafe extern "C" fn vim_dialog_yesno(
    mut type_0: ::core::ffi::c_int,
    mut title: *mut ::core::ffi::c_char,
    mut message: *mut ::core::ffi::c_char,
    mut dflt: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if do_dialog(
        type_0,
        if title.is_null() {
            gettext(b"Question\0".as_ptr() as *const ::core::ffi::c_char)
        } else {
            title
        },
        message,
        gettext(b"&Yes\n&No\0".as_ptr() as *const ::core::ffi::c_char),
        dflt,
        ::core::ptr::null::<::core::ffi::c_char>(),
        false_0,
    ) == 1 as ::core::ffi::c_int
    {
        return VIM_YES as ::core::ffi::c_int;
    }
    return VIM_NO as ::core::ffi::c_int;
}
pub unsafe extern "C" fn vim_dialog_yesnocancel(
    mut type_0: ::core::ffi::c_int,
    mut title: *mut ::core::ffi::c_char,
    mut message: *mut ::core::ffi::c_char,
    mut dflt: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    match do_dialog(
        type_0,
        if title.is_null() {
            gettext(b"Question\0".as_ptr() as *const ::core::ffi::c_char)
        } else {
            title
        },
        message,
        gettext(b"&Yes\n&No\n&Cancel\0".as_ptr() as *const ::core::ffi::c_char),
        dflt,
        ::core::ptr::null::<::core::ffi::c_char>(),
        false_0,
    ) {
        1 => return VIM_YES as ::core::ffi::c_int,
        2 => return VIM_NO as ::core::ffi::c_int,
        _ => {}
    }
    return VIM_CANCEL as ::core::ffi::c_int;
}
pub unsafe extern "C" fn vim_dialog_yesnoallcancel(
    mut type_0: ::core::ffi::c_int,
    mut title: *mut ::core::ffi::c_char,
    mut message: *mut ::core::ffi::c_char,
    mut dflt: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    match do_dialog(
        type_0,
        if title.is_null() {
            b"Question\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            title as *const ::core::ffi::c_char
        },
        message,
        gettext(
            b"&Yes\n&No\nSave &All\n&Discard All\n&Cancel\0".as_ptr() as *const ::core::ffi::c_char
        ),
        dflt,
        ::core::ptr::null::<::core::ffi::c_char>(),
        false_0,
    ) {
        1 => return VIM_YES as ::core::ffi::c_int,
        2 => return VIM_NO as ::core::ffi::c_int,
        3 => return VIM_ALL as ::core::ffi::c_int,
        4 => return VIM_DISCARDALL as ::core::ffi::c_int,
        _ => {}
    }
    return VIM_CANCEL as ::core::ffi::c_int;
}
pub unsafe extern "C" fn msg_delay(mut ms: uint64_t, mut ignoreinput: bool) {
    if ui_has(kUIMessages) {
        return;
    }
    if nvim_testing.get() {
        ms = 100 as uint64_t;
    }
    logmsg(
        LOGLVL_DBG,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"msg_delay\0".as_ptr() as *const ::core::ffi::c_char,
        4047 as ::core::ffi::c_int,
        true_0 != 0,
        b"%lu ms%s\0".as_ptr() as *const ::core::ffi::c_char,
        ms,
        if nvim_testing.get() as ::core::ffi::c_int != 0 {
            b" (skipped by NVIM_TEST)\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
    );
    ui_flush();
    os_delay(ms, ignoreinput);
}
pub unsafe extern "C" fn msg_check_for_delay(mut check_msg_scroll: bool) {
    if (emsg_on_display.get() as ::core::ffi::c_int != 0
        || check_msg_scroll as ::core::ffi::c_int != 0 && msg_scroll.get() != 0)
        && !did_wait_return.get()
        && emsg_silent.get() == 0 as ::core::ffi::c_int
        && !in_assert_fails.get()
        && !ui_has(kUIMessages)
    {
        msg_delay(1006 as uint64_t, true_0 != 0);
        emsg_on_display.set(false_0 != 0);
        if check_msg_scroll {
            msg_scroll.set(false_0);
        }
    }
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const MSG_BUF_LEN: ::core::ffi::c_int = 480 as ::core::ffi::c_int;
pub const KS_ZERO: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
pub const KE_FILLER: ::core::ffi::c_int = 'X' as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
