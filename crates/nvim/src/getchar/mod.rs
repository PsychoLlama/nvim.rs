#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::private::helpers::api_clear_error;
use crate::api::vim::nvim_paste;
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::{ptr2cells, skipwhite};
use crate::cursor::get_cursor_line_ptr;
use crate::drawscreen::{setcursor, showmode, unshowmode, update_screen};
use crate::edit::{edit_putchar, edit_unputchar};
use crate::eval::garbage_collect;
use crate::eval::typval::{
    tv_check_for_opt_dict_arg, tv_dict_get_bool, tv_dict_has_key, tv_get_number_chk,
};
use crate::eval::vars::set_vim_var_nr;
use crate::event::libuv::uv_strerror;
use crate::event::multiqueue::multiqueue_empty;
use crate::ex_cmds::check_secure;
use crate::ex_docmd::update_topline_cursor;
use crate::ex_getln::{cmdline_in_use, putcmdline, redrawcmd, redrawcmdline, unputcmdline};
use crate::garray::{ga_append, ga_clear, ga_concat_len, ga_grow};
use crate::global_cell::GlobalCell;
use crate::input::get_keystroke;
use crate::insexpand::{compl_status_local, ctrl_x_mode_not_default, vim_is_ctrl_x_key};
use crate::keycodes::{
    K_DOWN, K_END, K_HOME, K_HOR_SCROLLBAR, K_K0, K_K1, K_K2, K_K3, K_K4, K_K5, K_K6, K_K7, K_K8,
    K_K9, K_KCOMMA, K_KDIVIDE, K_KDOWN, K_KENTER, K_KEQUAL, K_KLEFT, K_KMINUS, K_KMULTIPLY,
    K_KPLUS, K_KPOINT, K_KRIGHT, K_KUP, K_LEFT, K_PASTE_END, K_PASTE_START, K_RIGHT, K_S_END,
    K_S_HOME, K_SPECIAL, K_UP, K_VER_SCROLLBAR, K_XDOWN, K_XEND, K_XHOME, K_XLEFT, K_XRIGHT, K_XUP,
    K_ZEND, K_ZERO, K_ZHOME, special_to_buf,
};
use crate::lua::executor::{nlua_call_ref, nlua_execute_on_key};
use crate::main::{
    KeyStuffed, KeyTyped, State, VIsual_reselect, allow_keys, arrow_used, c_bytes, called_emsg,
    cmd_silent, cmdline_row, cmdline_star, cmdwin_type, ctrl_c_interrupts, curbuf, curwin,
    debug_did_msg, did_ai, did_emsg, did_outofmem_msg, did_swapwrite_msg, e_invarg2, e_invargNval,
    e_nesting, e_notopen_2, e_toocompl, emsg_silent, ex_normal_busy, exmode_active, finish_op,
    got_int, ignore_script, langmap_mapchar, main_loop, mapped_ctrl_c, maptick,
    may_garbage_collect, mod_mask, mode_displayed, mouse_col, mouse_grid, mouse_row, msg_col,
    msg_didout, msg_row, msg_scroll, msg_silent, must_redraw, need_wait_return, no_mapping,
    no_zero_mapping, p_fs, p_langmap, p_lrm, p_lz, p_mmd, p_paste, p_sc, p_smd, p_timeout, p_tm,
    p_ttimeout, p_ttm, p_uc, pending_end_reg_executing, pending_exmode_active, redo_VIsual_busy,
    redraw_cmdline, reg_executing, reg_recording, repeat_luaref, restart_edit, scriptout,
    test_disable_char_avail, typebuf_was_empty, typebuf_was_filled, vgetc_busy, vgetc_char,
    vgetc_mod_mask, want_garbage_collect,
};
use crate::mapping::{eval_map_expr, get_buf_maphash_list, get_maphash_list, langmap_adjust_mb};
use crate::mbyte::{
    mb_cptr2char_adv, mb_unescape, utf_char2bytes, utf_head_off, utf_ptr2cells, utf_ptr2char,
    utf_ptr2str_char_info, utf8len_tab, utfc_next, utfc_ptr2len,
};
use crate::memline::ml_sync_all;
use crate::memory::{
    ARENA_EMPTY, arena_finish, arena_mem_free, strequal, xfree, xmalloc, xmemcpyz, xmemdupz,
};
use crate::message::{emsg, iemsg, internal_error};
use crate::mouse::{MousePos, comp_pos, find_win_inner, is_mouse_key};
use crate::r#move::{validate_cursor, win_col_off};
use crate::normal::{add_to_showcmd, normal_cmd, pop_showcmd, push_showcmd};
use crate::ops::clear_oparg;
use crate::options::kOptBoFlagError;
use crate::os::cshim::{gettext, putc, stderr, strncmp};
use crate::os::env::expand_env;
use crate::os::fileio::{FileOpenFlags, file_close, file_open, file_open_stdin, file_read};
use crate::os::input::{input_available, input_get, line_breakcheck, os_breakcheck};
use crate::plines::{init_charsize_arg, win_charsize};
use crate::state::{
    MODE_ASKMORE, MODE_CMDLINE, MODE_HITRETURN, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL,
    MODE_SELECT, MODE_TERMINAL, MODE_VISUAL, get_real_state, state_handle_k_event,
    state_no_longer_safe,
};
use crate::strings::vim_strchr;
use crate::types::{
    Arena, Array, CharsizeArg, Error, EvalFuncData, FileDescriptor, Integer, LuaRef, LuaRetMode,
    MotionType, MultiQueue, Object, OptInt, RemapValues, String_0, Vv, colnr_T, flush_buffers_T,
    garray_T, mapblock_T, oparg_T, ptrdiff_t, save_redo_T, size_t, tasave_T, typval_T, uint8_t,
    uint64_t, varnumber_T,
};
use crate::ui::{ui_busy_start, ui_busy_stop, ui_cursor_goto, ui_flush, vim_beep};
use crate::undo::u_sync;
use ::libc::{atoi, fprintf, strcmp, strlen};

// The carve of the transpiled module; see each child's docs.
mod buffers;
pub use self::buffers::*;
mod redo;
pub use self::redo::*;
mod typeahead;
pub use self::typeahead::*;
mod record;
pub use self::record::*;
mod scriptin;
pub use self::scriptin::*;
mod mapmatch;
pub(crate) use self::mapmatch::*;
mod key;
pub use self::key::*;
mod peek;
pub use self::peek::*;
mod funcs;
pub use self::funcs::*;
mod cmdkey;
pub use self::cmdkey::*;
mod paste;
pub use self::paste::*;
pub const MAXMAPLEN: ::core::ffi::c_uint = 50;
pub const REMAP_SKIP: RemapValues = -3;
pub const REMAP_SCRIPT: RemapValues = -2;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub const FLUSH_INPUT: flush_buffers_T = 2;
pub const FLUSH_MINIMAL: flush_buffers_T = 0;
pub const NSCRIPT: ::core::ffi::c_uint = 15;
pub const RM_SCRIPT: ::core::ffi::c_uint = 2;
pub const RM_NONE: ::core::ffi::c_uint = 1;
pub const RM_YES: ::core::ffi::c_uint = 0;
pub const RM_ABBR: ::core::ffi::c_uint = 4;
#[derive(Copy, Clone)]
pub struct gotchars_state_T {
    pub buf: [uint8_t; 67],
    pub prev_c: ::core::ffi::c_int,
    pub buflen: size_t,
    pub pending_special: ::core::ffi::c_uint,
    pub pending_mbyte: ::core::ffi::c_uint,
}
pub const KEYLEN_PART_KEY: ::core::ffi::c_int = -1;
pub const SHOWCMD_COLS: ::core::ffi::c_uint = 10;
pub const map_result_get: map_result_T = 1;
pub type map_result_T = ::core::ffi::c_uint;
pub const map_result_nomatch: map_result_T = 3;
pub const map_result_retry: map_result_T = 2;
pub const map_result_fail: map_result_T = 0;
pub const KEYLEN_PART_MAP: ::core::ffi::c_int = -2;
pub const kMTCharWise: MotionType = 0;
pub const kFileReadOnly: FileOpenFlags = 1;
pub const kFileNonBlocking: FileOpenFlags = 128;
pub const kRetNilBool: LuaRetMode = 1;
pub const kFileTruncate: FileOpenFlags = 32;
pub const kFileCreateOnly: FileOpenFlags = 16;
pub const kFileNoSymlink: FileOpenFlags = 8;
pub const kFileWriteOnly: FileOpenFlags = 4;
pub const kFileCreate: FileOpenFlags = 2;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
/// The top bit of a channel id, which no real channel has: a call that
/// carries it came from inside nvim rather than over a channel.
pub const INTERNAL_CALL_MASK: uint64_t = 1 << (uint64_t::BITS - 1);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL + 1;

/// Whether `channel_id` names an internal caller rather than a channel.
fn is_internal_call(channel_id: uint64_t) -> bool {
    channel_id & INTERNAL_CALL_MASK != 0
}
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const DEL: ::core::ffi::c_int = 0x7f as ::core::ffi::c_int;
static curscript: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
/// Streams to read script (`-s` / `:source!`) input from, innermost last.
static scriptin: GlobalCell<[FileDescriptor; NSCRIPT as usize]> =
    GlobalCell::new([EMPTY_FILE; NSCRIPT as usize]);

/// A `FileDescriptor` that has never been opened. Upstream's array is a
/// zero-initialised C static; this spells the same bytes once instead of
/// fifteen times.
const EMPTY_FILE: FileDescriptor = FileDescriptor {
    fd: 0,
    buffer: ::core::ptr::null_mut(),
    read_pos: ::core::ptr::null_mut(),
    write_pos: ::core::ptr::null_mut(),
    wr: false,
    eof: false,
    non_blocking: false,
    bytes_read: 0,
};
// The five key buffers. Each is reached through the `KeyBufferRef` its
// accessor answers (`redobuff()`, ...), never by name: the accessor is what
// keeps every operation to one short borrow of the cell.
static REDOBUFF: GlobalCell<KeyBuffer> = GlobalCell::new(KeyBuffer::EMPTY);
static OLD_REDOBUFF: GlobalCell<KeyBuffer> = GlobalCell::new(KeyBuffer::EMPTY);
static RECORDBUFF: GlobalCell<KeyBuffer> = GlobalCell::new(KeyBuffer::EMPTY);
static READBUF1: GlobalCell<KeyBuffer> = GlobalCell::new(KeyBuffer::EMPTY);
static READBUF2: GlobalCell<KeyBuffer> = GlobalCell::new(KeyBuffer::EMPTY);
/// The bytes of the key `vgetc` is assembling, for the `vim.on_key()`
/// callbacks. Upstream is a `kvec_withinit_t(char, MAXMAPLEN + 1)`; nothing
/// outside this module touches it, so it is an owned `Vec` here.
static on_key_buf: GlobalCell<Vec<u8>> = GlobalCell::new(Vec::new());
static on_key_ignore_len: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
static typeahead_char: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static block_redo: GlobalCell<bool> = GlobalCell::new(false);
static KeyNoremap: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
/// How many bytes the last `gotchars` recorded, so that `get_recorded` can
/// drop the keys that stopped the recording.
///
/// Every arithmetic on this counter is **wrapping**, as the C's `size_t` is:
/// `vgetc` subtracts what the previous call recorded and `ungetchars`
/// subtracts what it took back, and either can take it below zero. A huge
/// value then makes `get_recorded`'s `len >= last_recorded_len` fail and
/// nothing is trimmed, which is what upstream does. `test_registers`'
/// Test_recording_with_select_mode reaches it.
static last_recorded_len: GlobalCell<size_t> = GlobalCell::new(0);
static e_recursive_mapping: [::core::ffi::c_char; 24] = c_bytes(b"E223: Recursive mapping\0");
static e_cmd_mapping_must_end_with_cr: [::core::ffi::c_char; 40] =
    c_bytes(b"E1255: <Cmd> mapping must end with <CR>\0");
static e_cmd_mapping_must_end_with_cr_before_second_cmd: [::core::ffi::c_char; 60] =
    c_bytes(b"E1136: <Cmd> mapping must end with <CR> before second <Cmd>\0");
static old_char: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
static old_mod_mask: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static old_mouse_grid: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static old_mouse_row: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static old_mouse_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static old_KeyStuffed: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static no_reduce_keys: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
pub const K_SELECT_STRING: &::core::ffi::CStr = c"\x80\xF5X";
pub const MOD_MASK_SHIFT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOD_MASK_CTRL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MOD_MASK_ALT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
