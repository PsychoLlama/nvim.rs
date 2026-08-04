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
    __assert_fail, abort, abs, fclose, fprintf, fputs, gettext, memchr, memmove, ngettext, printf,
    putc, snprintf, stderr, strcmp, strlen, strnlen,
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
    ScopeType, ScreenGrid, SpecialVarValue, String_0, VV_ERRMSG, VV_SCROLLSTART, VV_STATUSMSG,
    VV_WARNINGMSG, VarLockStatus, VarType, buf_T, cmd_addr_T, colnr_T, estack_T, estack_arg_T,
    exarg_T, flush_buffers_T, garray_T, int64_t, kObjectTypeArray, kObjectTypeBoolean,
    kObjectTypeDict, kObjectTypeInteger, kObjectTypeNil, kObjectTypeString, key_extra,
    key_value_pair, linenr_T, object, object_data as C2Rust_Unnamed_11, ptrdiff_t, regmatch_T,
    sattr_T, schar_T, size_t, ssize_t, typval_T, typval_vval_union, uint8_t, uint32_t, uint64_t,
};
use crate::src::nvim::ui::{
    ui_active, ui_call_grid_destroy, ui_call_grid_resize, ui_call_grid_scroll,
    ui_call_msg_history_show, ui_call_msg_set_pos, ui_call_msg_show, ui_call_msg_showmode,
    ui_cursor_goto, ui_flush, ui_grid_cursor_goto, ui_has, ui_line, ui_refresh, vim_beep,
};
use crate::src::nvim::ui_compositor::{ui_comp_put_grid, ui_comp_remove_grid};

// The carve of the transpiled module; see each child's docs.
mod grid;
pub use self::grid::*;
mod scrollback;
pub use self::scrollback::*;
mod outtrans;
pub use self::outtrans::*;
mod prtline;
pub use self::prtline::*;
mod puts;
pub use self::puts::*;
mod ext;
pub use self::ext::*;
mod history;
pub use self::history::*;
mod prompt;
pub use self::prompt::*;
mod dialog;
pub use self::dialog::*;
mod redir;
pub use self::redir::*;
mod progress;
pub use self::progress::*;
mod errors;
pub use self::errors::*;
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
static do_clear_hist_temp: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
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
static last_msgchunk: GlobalCell<*mut msgchunk_T> =
    GlobalCell::new(::core::ptr::null_mut::<msgchunk_T>());
static do_clear_sb_text: GlobalCell<sb_clear_T> = GlobalCell::new(SB_CLEAR_NONE);
static pre_verbose_kind: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null::<::core::ffi::c_char>());
static verbose_kind: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"verbose\0".as_ptr() as *const ::core::ffi::c_char);
pub const HAS_HOTKEY_LEN: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
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
