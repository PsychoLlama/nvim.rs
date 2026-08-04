#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::api::private::helpers::{
    api_free_array, copy_string, cstr_as_string, cstr_to_string, ga_take_string,
};
use crate::src::nvim::api::vim::nvim_echo;
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::autocmd::{AUGROUP_ALL, EVENT_PROGRESS, apply_autocmds_group, has_event};
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
use crate::src::nvim::mouse::{MOUSE_SETPOS, jump_to_mouse, setmouse};
use crate::src::nvim::option::{p_vfile, shortmess};
use crate::src::nvim::options::{
    kOptBoFlagMess, kOptBoFlagShell, kOptMoptFlagHistory, kOptMoptFlagHitEnter,
    kOptMoptFlagProgress, kOptMoptFlagWait, kOptRdbFlagNothrottle,
};
use crate::src::nvim::os::env::home_replace_save;
use crate::src::nvim::os::fs::os_fopen;
use crate::src::nvim::os::input::{input_available, os_breakcheck};
use crate::src::nvim::os::libc::{
    abort, abs, fclose, fprintf, fputs, gettext, memchr, ngettext, printf, putc, snprintf, stderr,
    strcmp, strlen, strnlen,
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
    Arena, Array, BoolVarValue, Dict, Error, Event, FILE, HlMessage, HlMessageChunk, Integer,
    KeyDict_echo_opts, MessageData, Object, OptInt, ScopeType, SpecialVarValue, String_0,
    VV_ERRMSG, VV_SCROLLSTART, VV_STATUSMSG, VV_WARNINGMSG, VarLockStatus, VarType, colnr_T,
    estack_T, estack_arg_T, exarg_T, flush_buffers_T, garray_T, int64_t, kObjectTypeInteger,
    kObjectTypeNil, object, object_data as C2Rust_Unnamed_11, ptrdiff_t, regmatch_T, sattr_T,
    schar_T, size_t, ssize_t, typval_T, typval_vval_union, uint64_t,
};
use crate::src::nvim::ui::{
    ui_active, ui_call_grid_destroy, ui_call_grid_resize, ui_call_grid_scroll,
    ui_call_msg_history_show, ui_call_msg_set_pos, ui_call_msg_show, ui_call_msg_showmode,
    ui_cursor_goto, ui_flush, ui_grid_cursor_goto, ui_has, ui_line, ui_refresh, vim_beep,
};
use crate::src::nvim::ui_compositor::{ui_comp_put_grid, ui_comp_remove_grid};
use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

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
/// Vimscript value tags and lock states. Kept as a family because ffigen
/// exports them to the unit specs' flat cdef namespace, where the LuaJIT
/// side names them; nothing in this module reads most of them.
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
/// The compositor layer messages float on.
pub const kZIndexMessages: c_uint = 200;
/// One entry of the message history. See [`self::history`].
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
/// `'shortmess'` flags: `T` truncates a long message in the middle, `t`
/// truncates a file message at the head.
pub const SHM_TRUNCALL: c_uint = 84;
pub const SHM_TRUNC: c_uint = 116;
/// The longest UTF-8 sequence, including composing characters.
pub const MB_MAXBYTES: c_uint = 21;
/// [`do_dialog`] answers, as `confirm()` reports them.
pub const VIM_DISCARDALL: c_uint = 6;
pub const VIM_ALL: c_uint = 5;
pub const VIM_CANCEL: c_uint = 4;
pub const VIM_NO: c_uint = 3;
pub const VIM_YES: c_uint = 2;
/// One run of displayed message text, for scrolling back over. See
/// [`self::scrollback`].
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
/// How much of the scrollback the next message should drop.
pub type sb_clear_T = ::core::ffi::c_uint;
pub const SB_CLEAR_CMDLINE_DONE: sb_clear_T = 3;
pub const SB_CLEAR_CMDLINE_BUSY: sb_clear_T = 2;
pub const SB_CLEAR_ALL: sb_clear_T = 1;
pub const SB_CLEAR_NONE: sb_clear_T = 0;
pub const ESTACK_NONE: estack_arg_T = 0;
pub const FLUSH_MINIMAL: flush_buffers_T = 0;
/// A dialog's button list: `&` marks the next character as its hotkey, and
/// a newline separates buttons.
pub const DLG_HOTKEY_CHAR: c_uint = 38;
pub const DLG_BUTTON_SEP: c_uint = 10;
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
pub const PROGRESS_TARGET_CMD: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
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
    ga_data: ::core::ptr::null_mut(),
});
static msg_ext_last_attr: GlobalCell<sattr_T> = GlobalCell::new(-1 as sattr_T);
static msg_ext_last_hl_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static msg_ext_history: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static msg_ext_append: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static msg_grid_pos_at_flush: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
static msg_id_next: GlobalCell<int64_t> = GlobalCell::new(1 as int64_t);

pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const MSG_BUF_LEN: ::core::ffi::c_int = 480 as ::core::ffi::c_int;
pub const KS_ZERO: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

/// The active attribute for highlight group `hlf`, i.e. C's `HL_ATTR`.
///
/// # Safety
/// `hlf` must be one of the `HLF_*` indices, and the highlight table must
/// have been built (it is, from the first redraw onwards).
unsafe fn hl_attr(hlf: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe { *hl_attr_active.get().add(hlf as usize) }
}

/// The innermost entry of the `:source`/function call stack, which is what
/// C's `SOURCING_NAME` and `SOURCING_LNUM` read.
///
/// # Safety
/// The exec stack must be non-empty, which it is whenever anything is
/// running -- the outermost entry is pushed before `main()` sources a thing.
unsafe fn sourcing_top() -> *mut estack_T {
    unsafe {
        let stack = (*exestack.ptr()).ga_data as *mut estack_T;
        stack.add(((*exestack.ptr()).ga_len - 1) as usize)
    }
}

/// An [`Array`] owning nothing, C's `ARRAY_DICT_INIT`.
pub(crate) const EMPTY_ARRAY: Array = Array {
    size: 0,
    capacity: 0,
    items: ::core::ptr::null_mut(),
};

/// A [`HlMessage`] owning nothing.
pub(crate) const EMPTY_HL_MESSAGE: HlMessage = HlMessage {
    size: 0,
    capacity: 0,
    items: ::core::ptr::null_mut(),
};

/// Append to a heap-allocated [`Array`], growing it the way C's `kv_push`
/// does: eight elements, then doubling.
///
/// [`crate::src::nvim::types::builders::ArrayBuf`] is the stack-allocated
/// form, and is what a callee that only reads the value wants. This is for
/// the arrays whose ownership outlives the frame that builds them -- which
/// is every array the message code hands to the UI or to the history.
///
/// # Safety
/// `array` must be [`EMPTY_ARRAY`] or the result of earlier `array_push`es,
/// and must not be borrowed elsewhere: a growth reallocates `items`.
unsafe fn array_push(array: &mut Array, value: Object) {
    unsafe {
        if array.size == array.capacity {
            array.capacity = if array.capacity != 0 {
                array.capacity * 2
            } else {
                8
            };
            array.items = xrealloc(
                array.items.cast(),
                ::core::mem::size_of::<Object>() * array.capacity,
            )
            .cast();
        }
        array.items.add(array.size).write(value);
        array.size += 1;
    }
}

/// [`array_push`] for a [`HlMessage`], which is the same shape over
/// [`HlMessageChunk`].
///
/// # Safety
/// As [`array_push`], with [`EMPTY_HL_MESSAGE`] as the empty value.
unsafe fn hl_msg_push(msg: &mut HlMessage, chunk: HlMessageChunk) {
    unsafe {
        if msg.size == msg.capacity {
            msg.capacity = if msg.capacity != 0 {
                msg.capacity * 2
            } else {
                8
            };
            msg.items = xrealloc(
                msg.items.cast(),
                ::core::mem::size_of::<HlMessageChunk>() * msg.capacity,
            )
            .cast();
        }
        msg.items.add(msg.size).write(chunk);
        msg.size += 1;
    }
}

/// Show a message. Exported for the unit specs.
///
/// Answers false when the message needed a hit-enter prompt that has not been
/// answered yet.
///
/// # Safety
/// `s` must be a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn msg(s: *const c_char, hl_id: c_int) -> bool {
    unsafe { msg_keep(s, hl_id, false, false) }
}

/// Show `str`, splitting it at the control characters that need handling of
/// their own: a newline, tab or carriage return is emitted as itself, and a
/// bell rings instead of being drawn.
///
/// `need_clear` is cleared once the rest of the line has been cleared.
///
/// # Safety
/// `str` must describe a readable range, and `need_clear` a writable `bool`.
pub unsafe fn msg_multiline(
    str: String_0,
    hl_id: c_int,
    check_int: bool,
    hist: bool,
    need_clear: *mut bool,
) {
    unsafe {
        let mut s = str.data.cast_const();
        let mut chunk = s;
        while (s.offset_from(str.data) as size_t) < str.size {
            if check_int && got_int.get() {
                return;
            }
            if matches!(*s as u8, b'\n' | b'\t' | b'\r' | 0x07) {
                msg_outtrans_len(chunk, s.offset_from(chunk) as c_int, hl_id, hist);
                if *s as c_int != TAB && *need_clear {
                    msg_clr_eos();
                    *need_clear = false;
                }
                if *s as c_int == BELL {
                    vim_beep(kOptBoFlagShell as c_uint);
                } else {
                    msg_putchar_hl(*s as u8 as c_int, hl_id);
                }
                chunk = s.add(1);
            }
            s = s.add(1);
        }
        // The tail, and the whole of an empty message: an empty `str` still
        // has to reach `msg_outtrans_len`, which is what clears the line.
        if *chunk != 0 || chunk == str.data.cast_const() {
            msg_outtrans_len(
                chunk,
                (str.size - chunk.offset_from(str.data) as size_t) as c_int,
                hl_id,
                hist,
            );
        }
    }
}

/// Nonzero while [`msg_multihl`] is emitting a chunk, so [`msg_keep`] knows
/// not to start or end a message of its own.
pub(crate) static is_multihl: GlobalCell<c_int> = GlobalCell::new(0);

/// Show a message made of chunks, each with its own highlight id.
///
/// Answers the message's id: `id` itself when it names one, and a freshly
/// allocated one when it is nil.
///
/// # Safety
/// `hl_msg` must be a valid message, `kind` null or a valid C string, and
/// `needs_msg_clear` a writable `bool`.
pub unsafe fn msg_multihl(
    id: Object,
    hl_msg: HlMessage,
    kind: *const c_char,
    history: bool,
    err: bool,
    msg_data: *mut MessageData,
    needs_msg_clear: *mut bool,
) -> Object {
    unsafe {
        let mut hl_msg = hl_msg;
        // Message `id`:
        // - Nil: generate a new Integer id.
        // - Integer: an existing id.
        // - String: a user-defined id, new or existing.
        let id = if id.type_0 == kObjectTypeNil {
            let next = msg_id_next.get();
            msg_id_next.set(next + 1);
            Object::integer(next)
        } else {
            if id.type_0 == kObjectTypeInteger && !msg_id_exists(id.data.integer) {
                abort();
            }
            id
        };

        let is_progress = strequal(kind, c"progress".as_ptr());
        // Don't display a progress message on the command line when the
        // target does not include it.
        if is_progress && progress_msg_target.get() & PROGRESS_TARGET_CMD == 0 {
            *needs_msg_clear = true;
            return id;
        }

        no_wait_return.set(no_wait_return.get() + 1);
        msg_start();
        msg_clr_eos();
        let mut need_clear = false;
        let mut hl_msg_updated = false;
        if !kind.is_null() {
            msg_ext_set_kind(kind);
        }
        msg_ext_skip_flush.set(true);
        msg_ext_id.set(id);

        // A progress message displays as "title: percent% msg".
        if is_progress && !msg_data.is_null() {
            let formatted = format_progress_message(hl_msg, msg_data);
            if formatted.items != hl_msg.items {
                *needs_msg_clear = true;
                hl_msg_updated = true;
                hl_msg = formatted;
            }
        }

        for i in 0..hl_msg.size {
            let chunk = *hl_msg.items.add(i);
            is_multihl.set(is_multihl.get() + 1);
            if err {
                emsg_multiline(chunk.text.data, kind, chunk.hl_id, true);
            } else {
                msg_multiline(chunk.text, chunk.hl_id, true, false, &raw mut need_clear);
            }
            debug_assert!(!ui_has(kUIMessages) || kind.is_null() || msg_ext_kind.get() == kind);
        }

        let kept = history && hl_msg.size != 0;
        if kept {
            msg_hist_add_multihl(hl_msg, false, msg_data);
        }

        msg_ext_skip_flush.set(false);
        is_multihl.set(0);
        no_wait_return.set(no_wait_return.get() - 1);
        msg_end();

        // The reformatted message is ours to free unless the history took it.
        if hl_msg_updated && !kept {
            hl_msg_free(hl_msg);
        }
        id
    }
}

/// Show a message, optionally keeping it displayed after a redraw.
///
/// `keep` sets `keep_msg` when the message fits without scrolling;
/// `multiline` sends it through [`msg_multiline`] rather than
/// [`msg_outtrans`].
///
/// # Safety
/// `s` must be a valid C string.
pub unsafe fn msg_keep(s: *const c_char, hl_id: c_int, keep: bool, multiline: bool) -> bool {
    unsafe {
        static entered: GlobalCell<c_int> = GlobalCell::new(0);

        if keep && multiline {
            // Not implemented. 'multiline' is only used by nvim-added
            // messages, which should avoid 'keep' behaviour -- they should
            // just show the message at the right time already.
            abort();
        }

        // Skip messages that do not match ":filter pattern", but never filter
        // when there is an error.
        if !emsg_on_display.get() && message_filtered(s) {
            return true;
        }

        if hl_id == 0 {
            set_vim_var_string(VV_STATUSMSG, s, -1);
        }

        // Displaying a message can cause a problem (e.g. when redrawing the
        // window), which causes another message, and so on. Break the loop by
        // limiting the recursion to three levels.
        if entered.get() >= 3 {
            return true;
        }
        entered.set(entered.get() + 1);

        // Add the message to the history unless it is a multihl, or a repeat
        // of the kept message, or a truncated one.
        if is_multihl.get() == 0
            && (s != keep_msg.get().cast_const()
                || (*s as u8 != b'<'
                    && !msg_hist_last.get().is_null()
                    && strcmp(s, (*(*msg_hist_last.get()).msg.items).text.data) != 0))
        {
            msg_hist_add(s, -1, hl_id);
        }

        if is_multihl.get() == 0 {
            msg_start();
        }

        // Truncate the message if needed.
        let buf = msg_strtrunc(s, false_0);
        let s = if buf.is_null() { s } else { buf.cast_const() };

        let mut need_clear = true;
        if multiline {
            msg_multiline(cstr_as_string(s), hl_id, false, false, &raw mut need_clear);
        } else {
            msg_outtrans(s, hl_id, false);
        }
        if need_clear {
            msg_clr_eos();
        }

        let mut retval = true;
        if is_multihl.get() == 0 {
            retval = msg_end();
        }

        if keep
            && retval
            && vim_strsize(s) < (Rows.get() - cmdline_row.get() - 1) * Columns.get() + sc_col.get()
        {
            set_keep_msg(s, 0);
        }

        need_fileinfo.set(false);

        xfree(buf.cast());
        entered.set(entered.get() - 1);
        retval
    }
}

/// Truncate `s` so it prints without causing a scroll.
///
/// Answers an allocated string, or null when no truncation was needed.
/// `force` truncates regardless of `'shortmess'`.
///
/// # Safety
/// `s` must be a valid C string.
pub unsafe fn msg_strtrunc(s: *const c_char, force: c_int) -> *mut c_char {
    unsafe {
        let mut buf: *mut c_char = ptr::null_mut();
        // May truncate the message to avoid a hit-return prompt.
        if (msg_scroll.get() == 0
            && !need_wait_return.get()
            && shortmess(SHM_TRUNCALL as c_int)
            && !exmode_active.get()
            && msg_silent.get() == 0
            && !ui_has(kUIMessages))
            || force != 0
        {
            let mut len = vim_strsize(s);
            let room = if msg_scrolled.get() != 0 {
                // Use all the columns.
                (Rows.get() - msg_row.get()) * Columns.get() - 1
            } else {
                // Use up to the 'showcmd' column.
                (Rows.get() - msg_row.get() - 1) * Columns.get() + sc_col.get() - 1
            };
            if len > room && room > 0 {
                // Up to 18 bytes per cell: six per character, and up to two
                // composing characters.
                len = (room + 2) * 18;
                buf = xmalloc(len as size_t).cast();
                trunc_string(s, buf, room, len);
            }
        }
        buf
    }
}

/// Truncate `s` into `buf` at cell width `room`, replacing the middle with
/// "...". `s` and `buf` may be the same pointer.
///
/// Exported for the unit specs.
///
/// # Safety
/// `s` must be a valid C string and `buf` must have room for `buflen` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn trunc_string(
    s: *const c_char,
    buf: *mut c_char,
    room_in: c_int,
    buflen: c_int,
) {
    unsafe {
        let mut room = room_in - 3; // "..." takes 3 chars
        let mut len = 0;
        let mut n;

        if *s == 0 {
            if buflen > 0 {
                *buf = 0;
            }
            return;
        }
        if room_in < 3 {
            room = 0;
        }
        let mut half = room / 2;

        // First part: the start of the string.
        let mut e = 0;
        while len < half && e < buflen {
            if *s.offset(e as isize) == 0 {
                // Text fits without truncating.
                *buf.offset(e as isize) = 0;
                return;
            }
            n = ptr2cells(s.offset(e as isize));
            if len + n > half {
                break;
            }
            len += n;
            *buf.offset(e as isize) = *s.offset(e as isize);
            // Copy the rest of a multibyte character one byte at a time; the
            // inner break leaves the outer step to run, as upstream's does.
            n = utfc_ptr2len(s.offset(e as isize));
            loop {
                n -= 1;
                if n <= 0 {
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

        // Last part: the end of the string.
        let mut i = strlen(s) as c_int;
        half = i;
        loop {
            half = half - utf_head_off(s, s.offset(half as isize).offset(-1)) - 1;
            n = ptr2cells(s.offset(half as isize));
            if len + n > room || half == 0 {
                break;
            }
            len += n;
            i = half;
        }

        if i <= e + 3 {
            // Text fits without truncating.
            if s != buf.cast_const() {
                len = strlen(s) as c_int;
                if len >= buflen {
                    len = buflen - 1;
                }
                len = len - e + 1;
                if len < 1 {
                    *buf.offset((e - 1) as isize) = 0;
                } else {
                    ptr::copy(s.offset(e as isize), buf.offset(e as isize), len as usize);
                }
            }
        } else if e + 3 < buflen {
            // Set the middle and copy the last part.
            ptr::copy_nonoverlapping(c"...".as_ptr(), buf.offset(e as isize), 3);
            len = strlen(s.offset(i as isize)) as c_int + 1;
            if len >= buflen - e - 3 {
                len = buflen - e - 3 - 1;
            }
            ptr::copy(
                s.offset(i as isize),
                buf.offset(e as isize).offset(3),
                len as usize,
            );
            *buf.offset((e + 3 + len - 1) as isize) = 0;
        } else {
            // Can't fit the "...", so just truncate.
            *buf.offset((buflen - 1) as isize) = 0;
        }
    }
}

/// [`msg`] with `printf` formatting. See the note on the variadics in
/// [`crate::src::nvim::message::errors`].
///
/// The caller must check the result is shorter than `IOSIZE`.
///
/// # Safety
/// `s` and the arguments must agree, as for `printf`.
pub unsafe extern "C" fn smsg(hl_id: c_int, s: *const c_char, mut c2rust_args: ...) -> c_int {
    unsafe {
        vim_vsnprintf(
            IObuff.ptr().cast(),
            IOSIZE as size_t,
            s,
            c2rust_args.clone(),
        );
        msg(IObuff.ptr().cast(), hl_id) as c_int
    }
}

/// [`msg_keep`] with `printf` formatting, keeping the message displayed.
///
/// # Safety
/// As [`smsg`].
pub unsafe extern "C" fn smsg_keep(hl_id: c_int, s: *const c_char, mut c2rust_args: ...) -> c_int {
    unsafe {
        vim_vsnprintf(
            IObuff.ptr().cast(),
            IOSIZE as size_t,
            s,
            c2rust_args.clone(),
        );
        msg_keep(IObuff.ptr().cast(), hl_id, true, false) as c_int
    }
}

/// Show `s`, truncated at the head if it does not fit the message area.
///
/// Answers the string that was shown, or null if it was not.
///
/// # Safety
/// `s` must be a valid, writable C string -- the truncation writes into it.
pub unsafe fn msg_trunc(s: *mut c_char, force: bool, hl_id: c_int) -> *mut c_char {
    unsafe {
        // The history gets the whole message; only the display is truncated.
        msg_hist_add(s, -1, hl_id);
        let ts = msg_may_trunc(force, s);
        msg_hist_off.set(true);
        let n = msg(ts, hl_id);
        msg_hist_off.set(false);
        if n { ts } else { ptr::null_mut() }
    }
}

/// Drop the *head* of `s` if it does not fit the message area, marking the
/// cut with a `<`.
///
/// Answers a pointer into `s`, which the marker is written through.
///
/// # Safety
/// `s` must be a valid, writable C string.
pub unsafe fn msg_may_trunc(force: bool, s: *mut c_char) -> *mut c_char {
    unsafe {
        // Under ext_messages the UI decides what fits. This guard changes no
        // answer -- nothing shrinks `cmdline_row` off `Rows` there, so `room`
        // is already negative -- but it says what the intent is; see docket
        // O-B13-6.
        if ui_has(kUIMessages) {
            return s;
        }
        let mut s = s;
        let room = (Rows.get() - cmdline_row.get() - 1) * Columns.get() + sc_col.get() - 1;
        if room > 0
            && (force || (shortmess(SHM_TRUNC as c_int) && !exmode_active.get()))
            && strlen(s) as c_int - room > 0
        {
            // May have up to 18 bytes per cell (6 per char, up to two
            // composing chars).
            let mut size = vim_strsize(s);
            if size <= room {
                return s;
            }
            // Find the last character that fits.
            let mut n = 0;
            while size >= room {
                size -= utf_ptr2cells(s.offset(n as isize));
                n += utfc_ptr2len(s.offset(n as isize));
            }
            n -= 1;
            s = s.offset(n as isize);
            *s = b'<' as c_char;
        }
        s
    }
}

/// Set `keep_msg` to `s`, freeing the old value.
///
/// # Safety
/// `s` must be null or a valid C string.
pub unsafe fn set_keep_msg(s: *const c_char, hl_id: c_int) {
    unsafe {
        // The kept message is not cleared and re-emitted with ext_messages:
        // neovim/neovim#20416.
        if ui_has(kUIMessages) {
            return;
        }
        xfree(keep_msg.get().cast());
        keep_msg.set(if !s.is_null() && msg_silent.get() == 0 {
            xstrdup(s)
        } else {
            ptr::null_mut()
        });
        keep_msg_more.set(false);
        keep_msg_hl_id.set(hl_id);
    }
}

/// Would a message be seen if it were shown now?
///
/// # Safety
/// Only that the typeahead is in a consistent state.
pub unsafe fn messaging() -> bool {
    unsafe {
        !(p_lz.get() != 0 && char_avail() && !KeyTyped.get())
            && (p_ch.get() > 0 || ui_has(kUIMessages))
    }
}

/// Report "N more lines" / "N fewer lines" after an edit, if `'report'`
/// allows it.
///
/// # Safety
/// Only that the message statics are consistent.
pub unsafe fn msgmore(n: c_int) {
    unsafe {
        if global_busy.get() != 0 || !messaging() {
            // Don't report when :global is executing.
            return;
        }
        // Keep the message from a previous msgmore(), but not another one.
        if !keep_msg.get().is_null() && !keep_msg_more.get() {
            return;
        }

        let pn = abs(n);
        if pn as OptInt <= p_report.get() {
            return;
        }
        let (one, many) = if n > 0 {
            (c"%d more line", c"%d more lines")
        } else {
            (c"%d line less", c"%d fewer lines")
        };
        vim_snprintf(
            msg_buf.ptr().cast(),
            MSG_BUF_LEN as size_t,
            ngettext(one.as_ptr(), many.as_ptr(), pn as ::core::ffi::c_ulong),
            pn,
        );
        if got_int.get() {
            xstrlcat(
                msg_buf.ptr().cast(),
                gettext(c" (Interrupted)".as_ptr()),
                MSG_BUF_LEN as size_t,
            );
        }
        if msg(msg_buf.ptr().cast(), 0) {
            set_keep_msg(msg_buf.ptr().cast(), 0);
            keep_msg_more.set(true);
        }
    }
}
