#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::api::private::helpers::api_clear_error;
use crate::src::nvim::api::vim::nvim_paste;
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::charset::{ptr2cells, skipwhite};
use crate::src::nvim::cursor::get_cursor_line_ptr;
use crate::src::nvim::drawscreen::{setcursor, showmode, unshowmode, update_screen};
use crate::src::nvim::edit::{edit_putchar, edit_unputchar};
use crate::src::nvim::eval::garbage_collect;
use crate::src::nvim::eval::typval::{
    tv_check_for_opt_dict_arg, tv_dict_get_bool, tv_dict_get_string, tv_dict_has_key,
    tv_get_number_chk,
};
use crate::src::nvim::eval::vars::set_vim_var_nr;
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::event::multiqueue::multiqueue_empty;
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::ex_docmd::update_topline_cursor;
use crate::src::nvim::ex_getln::{
    get_cmdline_info, putcmdline, redrawcmd, redrawcmdline, unputcmdline,
};
use crate::src::nvim::garray::{ga_append, ga_clear, ga_concat_len, ga_grow};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::input::get_keystroke;
use crate::src::nvim::insexpand::{compl_status_local, ctrl_x_mode_not_default, vim_is_ctrl_x_key};
use crate::src::nvim::keycodes::{
    K_DOWN, K_END, K_HOME, K_HOR_SCROLLBAR, K_K0, K_K1, K_K2, K_K3, K_K4, K_K5, K_K6, K_K7, K_K8,
    K_K9, K_KCOMMA, K_KDIVIDE, K_KDOWN, K_KENTER, K_KEQUAL, K_KLEFT, K_KMINUS, K_KMULTIPLY,
    K_KPLUS, K_KPOINT, K_KRIGHT, K_KUP, K_LEFT, K_PASTE_END, K_PASTE_START, K_RIGHT, K_S_END,
    K_S_HOME, K_SPECIAL, K_UP, K_VER_SCROLLBAR, K_XDOWN, K_XEND, K_XHOME, K_XLEFT, K_XRIGHT, K_XUP,
    K_ZEND, K_ZERO, K_ZHOME, special_to_buf,
};
use crate::src::nvim::lua::executor::{nlua_call_ref, nlua_execute_on_key};
use crate::src::nvim::main::{
    KeyStuffed, KeyTyped, NameBuff, State, VIsual, VIsual_active, VIsual_reselect, VIsual_select,
    allow_keys, arrow_used, c_bytes, called_emsg, cmd_silent, cmdline_row, cmdline_star,
    cmdwin_type, ctrl_c_interrupts, curbuf, curwin, debug_did_msg, did_ai, did_emsg,
    did_outofmem_msg, did_swapwrite_msg, e_invarg2, e_invargNval, e_nesting, e_notopen_2,
    e_toocompl, emsg_silent, ex_normal_busy, exmode_active, finish_op, firstwin, got_int,
    ignore_script, langmap_mapchar, main_loop, mapped_ctrl_c, maptick, may_garbage_collect,
    mod_mask, mode_displayed, mouse_col, mouse_grid, mouse_row, msg_col, msg_didout, msg_row,
    msg_scroll, msg_silent, must_redraw, need_wait_return, no_mapping, no_zero_mapping, p_fs,
    p_langmap, p_lrm, p_lz, p_mmd, p_paste, p_sc, p_smd, p_timeout, p_tm, p_ttimeout, p_ttm, p_uc,
    pending_end_reg_executing, pending_exmode_active, redo_VIsual_busy, redraw_cmdline,
    reg_executing, reg_recording, repeat_luaref, restart_edit, scriptout, test_disable_char_avail,
    typebuf, typebuf_was_empty, typebuf_was_filled, vgetc_busy, vgetc_char, vgetc_mod_mask,
    want_garbage_collect,
};
use crate::src::nvim::mapping::{
    eval_map_expr, get_buf_maphash_list, get_maphash_list, langmap_adjust_mb,
};
use crate::src::nvim::mbyte::{
    mb_cptr2char_adv, mb_unescape, utf_char2bytes, utf_head_off, utf_ptr2StrCharInfo,
    utf_ptr2cells, utf_ptr2char, utf8len_tab, utfc_next, utfc_ptr2len,
};
use crate::src::nvim::memline::ml_sync_all;
use crate::src::nvim::memory::{
    ARENA_EMPTY, arena_finish, arena_mem_free, strequal, xfree, xmalloc, xmemcpyz, xmemdupz,
};
use crate::src::nvim::message::{emsg, iemsg, internal_error};
use crate::src::nvim::mouse::{MousePos, comp_pos, find_win_inner, is_mouse_key};
use crate::src::nvim::r#move::{validate_cursor, win_col_off};
use crate::src::nvim::normal::{add_to_showcmd, normal_cmd, pop_showcmd, push_showcmd};
use crate::src::nvim::ops::clear_oparg;
use crate::src::nvim::options::kOptBoFlagError;
use crate::src::nvim::os::env::expand_env;
use crate::src::nvim::os::fileio::{file_close, file_open, file_open_stdin, file_read};
use crate::src::nvim::os::input::{input_available, input_get, line_breakcheck, os_breakcheck};
use crate::src::nvim::os::libc::{atoi, fprintf, gettext, putc, stderr, strcmp, strlen, strncmp};
use crate::src::nvim::plines::{init_charsize_arg, win_charsize};
use crate::src::nvim::state::{
    MODE_ASKMORE, MODE_CMDLINE, MODE_HITRETURN, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL,
    MODE_SELECT, MODE_TERMINAL, MODE_VISUAL, get_real_state, state_handle_k_event,
    state_no_longer_safe,
};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    Arena, Array, CharsizeArg, Error, EvalFuncData, FileDescriptor, Integer, LuaRef, LuaRetMode,
    MotionType, MultiQueue, Object, OptInt, RemapValues, String_0, TriState, VV_MOUSE_COL,
    VV_MOUSE_LNUM, VV_MOUSE_WIN, VV_MOUSE_WINID, buffblock, buffblock_T, buffheader_T, colnr_T,
    flush_buffers_T, garray_T, kFalse, kNone, mapblock_T, oparg_T, ptrdiff_t, save_redo_T, size_t,
    tasave_T, typebuf_T, typval_T, uint8_t, uint64_t, varnumber_T,
};
use crate::src::nvim::ui::{ui_busy_start, ui_busy_stop, ui_cursor_goto, ui_flush, vim_beep};
use crate::src::nvim::undo::u_sync;

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
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const MAXMAPLEN: C2Rust_Unnamed_27 = 50;
pub const REMAP_SKIP: RemapValues = -3;
pub const REMAP_SCRIPT: RemapValues = -2;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub const FLUSH_INPUT: flush_buffers_T = 2;
pub const FLUSH_MINIMAL: flush_buffers_T = 0;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const NSCRIPT: C2Rust_Unnamed_30 = 15;
pub const RM_SCRIPT: C2Rust_Unnamed_36 = 2;
pub const RM_NONE: C2Rust_Unnamed_36 = 1;
pub const RM_YES: C2Rust_Unnamed_36 = 0;
pub const RM_ABBR: C2Rust_Unnamed_36 = 4;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct gotchars_state_T {
    pub buf: [uint8_t; 67],
    pub prev_c: ::core::ffi::c_int,
    pub buflen: size_t,
    pub pending_special: ::core::ffi::c_uint,
    pub pending_mbyte: ::core::ffi::c_uint,
}
pub const KEYLEN_PART_KEY: C2Rust_Unnamed_37 = -1;
pub const SHOWCMD_COLS: C2Rust_Unnamed_33 = 10;
pub const map_result_get: map_result_T = 1;
pub type map_result_T = ::core::ffi::c_uint;
pub const map_result_nomatch: map_result_T = 3;
pub const map_result_retry: map_result_T = 2;
pub const map_result_fail: map_result_T = 0;
pub const KEYLEN_PART_MAP: C2Rust_Unnamed_37 = -2;
pub const kMTCharWise: MotionType = 0;
pub const kFileReadOnly: C2Rust_Unnamed_34 = 1;
pub const kFileNonBlocking: C2Rust_Unnamed_34 = 128;
pub const kRetNilBool: LuaRetMode = 1;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const kFileTruncate: C2Rust_Unnamed_34 = 32;
pub const kFileCreateOnly: C2Rust_Unnamed_34 = 16;
pub const kFileNoSymlink: C2Rust_Unnamed_34 = 8;
pub const kFileWriteOnly: C2Rust_Unnamed_34 = 4;
pub const kFileCreate: C2Rust_Unnamed_34 = 2;
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_37 = ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
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
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
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
/// An empty `buffheader_T`, the state all five of them start in.
const EMPTY_BUFF: buffheader_T = buffheader_T {
    bh_first: buffblock {
        b_next: ::core::ptr::null_mut(),
        b_strlen: 0,
        b_str: [0],
    },
    bh_curr: ::core::ptr::null_mut(),
    bh_index: 0,
    bh_space: 0,
    bh_create_newblock: false,
};

/// The redo buffer: the keys `.` replays.
static redobuff: GlobalCell<buffheader_T> = GlobalCell::new(EMPTY_BUFF);
/// The redo buffer before this command, which `CTRL-O .` replays.
static old_redobuff: GlobalCell<buffheader_T> = GlobalCell::new(EMPTY_BUFF);
/// The register being recorded into by `q`.
static recordbuff: GlobalCell<buffheader_T> = GlobalCell::new(EMPTY_BUFF);
/// First read-ahead buffer, for translated commands.
static readbuf1: GlobalCell<buffheader_T> = GlobalCell::new(EMPTY_BUFF);
/// Second read-ahead buffer, for redo.
static readbuf2: GlobalCell<buffheader_T> = GlobalCell::new(EMPTY_BUFF);
/// The bytes of the key `vgetc` is assembling, for the `vim.on_key()`
/// callbacks. Upstream is a `kvec_withinit_t(char, MAXMAPLEN + 1)`; nothing
/// outside this module touches it, so it is an owned `Vec` here.
static on_key_buf: GlobalCell<Vec<u8>> = GlobalCell::new(Vec::new());
static on_key_ignore_len: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
static typeahead_char: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static block_redo: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static KeyNoremap: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static typebuf_init: GlobalCell<[uint8_t; 265]> = GlobalCell::new([0; 265]);
static noremapbuf_init: GlobalCell<[uint8_t; 265]> = GlobalCell::new([0; 265]);
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
/// The typeahead each `:source!` displaced, put back by `closescript`.
static saved_typebuf: GlobalCell<[typebuf_T; NSCRIPT as usize]> = GlobalCell::new(
    [typebuf_T {
        tb_buf: ::core::ptr::null_mut::<uint8_t>(),
        tb_noremap: ::core::ptr::null_mut::<uint8_t>(),
        tb_buflen: 0,
        tb_off: 0,
        tb_len: 0,
        tb_maplen: 0,
        tb_silent: 0,
        tb_no_abbr_cnt: 0,
        tb_change_cnt: 0,
    }; NSCRIPT as usize],
);
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
pub const K_SELECT_STRING: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"\x80\xF5X\0") };
pub const MOD_MASK_SHIFT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOD_MASK_CTRL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MOD_MASK_ALT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
