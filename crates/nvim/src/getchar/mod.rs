#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

pub mod state;
use crate::api::vim::nvim_paste;
use crate::ascii::ascii_iswhite;
use crate::debugger::state::DEBUG_DID_MSG;
use crate::drawscreen::state::{CMDLINE_ROW, MODE_DISPLAYED, MUST_REDRAW, REDRAW_CMDLINE};
use crate::drawscreen::{setcursor, showmode, unshowmode, update_screen};
use crate::edit::{edit_putchar, edit_unputchar};
use crate::eval::funcs::builtin::EvalFuncData;
use crate::eval::garbage_collect;
use crate::eval::gc::{MAY_GARBAGE_COLLECT, WANT_GARBAGE_COLLECT};
use crate::eval::typval::{
    dict_get_bool, dict_has_key, tv_check_for_opt_dict_arg, tv_get_number_chk,
};
use crate::eval::vars::set_vim_var_nr;
use crate::ex_cmds::check_secure;
use crate::ex_docmd::state::EX_NORMAL_BUSY;
use crate::ex_docmd::update_topline_cursor;
use crate::ex_getln::state::CMDLINE_STAR;
use crate::ex_getln::{cmdline_in_use, putcmdline, redrawcmd, redrawcmdline, unputcmdline};
use crate::fileio::buffered::FileOpenFlags;
use crate::getchar::state::{
    ALLOW_KEYS, BLOCK_REDO, CTRL_C_INTERRUPTS, CURSCRIPT, GOT_INT, GOTCHARS_PENDING, IGNORE_SCRIPT,
    KEY_NOREMAP, KEY_STUFFED, KEY_TYPED, LANGMAP_MAPCHAR, LAST_RECORDED_LEN,
    LAST_VGETC_RECORDED_LEN, MAPPED_CTRL_C, MAPTICK, MOD_MASK, NO_MAPPING, NO_REDUCE_KEYS,
    NO_ZERO_MAPPING, NORMAL_BUSY_LAST_KEY, ON_KEY_IGNORE_LEN, PENDING_END_REG_EXECUTING,
    REG_EXECUTING, REG_RECORDING, REPEAT_LUAREF, SCRIPTOUT, SHOWCMD_PENDING, TYPEAHEAD_CHAR,
    TYPEBUF_WAS_EMPTY, TYPEBUF_WAS_FILLED, TYPED_SINCE_SYNC, UNGOT, UngotKey, VGETC_BUSY,
    VGETC_CHAR, VGETC_MOD_MASK, test_disable_char_avail,
};
use crate::getchar::state::{FlushBuffers, SaveRedo, TypeaheadSave};
use crate::global_cell::GlobalCell;
use crate::input::get_keystroke;
use crate::insexpand::{compl_status_local, ctrl_x_mode_not_default, vim_is_ctrl_x_key};
use crate::keycodes::{K_SPECIAL, KS_EXTRA, KS_MODIFIER, KS_SPECIAL, special_key_bytes};
use crate::lua::executor::nlua_execute_on_key;
use crate::mapping::{MapAt, MapExpr, eval_map_expr, with_maphash_lists};
use crate::mbyte::utf8len_tab;
use crate::memfile::DID_SWAPWRITE_MSG;
use crate::memline::ml_sync_all;
use crate::memory::DID_OUTOFMEM_MSG;
use crate::message::state::{
    CALLED_EMSG, CMD_SILENT, DID_EMSG, EMSG_SILENT, MSG_COL, MSG_DIDOUT, MSG_ROW, MSG_SCROLL,
    MSG_SILENT, NEED_WAIT_RETURN,
};
use crate::message::{E_NESTING, E_TOOCOMPL};
use crate::message::{emsg, iemsg};
use crate::mouse::state::{MOUSE_COL, MOUSE_GRID, MOUSE_ROW};
use crate::mouse::{MousePos, comp_pos, find_win_inner, is_mouse_key};
use crate::r#move::validate_cursor;
use crate::normal::defs::OpArg;
use crate::normal::{add_to_showcmd, normal_cmd, pop_showcmd, push_showcmd};
use crate::option::vars::{
    p_fs, p_lrm, p_lz, p_mmd, p_paste, p_sc, p_smd, p_timeout, p_tm, p_ttimeout, p_ttm, p_uc,
};
use crate::options::OPT_BO_FLAG_ERROR;
use crate::os::cshim::gettext;
use crate::os::input::{input_available, input_get, line_breakcheck, os_breakcheck};
use crate::os::libc::{ptrdiff_t, size_t, uint8_t, uint64_t};
use crate::state::mode::{
    ARROW_USED, DID_AI, EXMODE_ACTIVE, FINISH_OP, PENDING_EXMODE_ACTIVE, REDO_V_ISUAL_BUSY,
    RESTART_EDIT, STATE, V_ISUAL_RESELECT,
};
use crate::state::{
    MODE_ASKMORE, MODE_CMDLINE, MODE_HITRETURN, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL,
    MODE_SELECT, MODE_TERMINAL, MODE_VISUAL, get_real_state, state_handle_k_event,
    state_no_longer_safe,
};
use crate::types::{
    ApiString, Array, ColNr, Error, Integer, LuaRef, OptInt, TypVal, VarNumber, Vv,
};
use crate::ui::{ui_busy_start, ui_busy_stop, ui_cursor_goto, ui_flush, vim_beep};
use crate::undo::u_sync;
use crate::winlayer::graph::CMDWIN_TYPE;

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
/// The longest LHS a mapping can have, in bytes.
pub const MAXMAPLEN: usize = 50;
/// How much of the text going into the typeahead may be mapped again
/// (upstream's `REMAP_*` values and the positive count beside them).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Remap {
    /// All of it: `Remap::Yes`.
    Yes,
    /// None of it: `Remap::No`.
    No,
    /// Only through script-local mappings (`<script>`): `Remap::Script`.
    Script,
    /// Not the first byte, but abbreviations still apply: `Remap::Skip`.
    Skip,
    /// Not the first `n` bytes; the rest may be.
    First(usize),
}

impl Remap {
    /// The flag the bytes that may not be remapped are marked with, and how
    /// many of `len` inserted bytes get it.
    fn marks(self, len: usize) -> (u8, usize) {
        match self {
            Remap::Yes => (RM_NONE, 0),
            Remap::No => (RM_NONE, len),
            Remap::Script => (RM_SCRIPT, len),
            Remap::Skip => (RM_ABBR, 1),
            Remap::First(n) => (RM_NONE, n),
        }
    }
}
pub const FLUSH_INPUT: FlushBuffers = 2;
pub const FLUSH_MINIMAL: FlushBuffers = 0;
pub const NSCRIPT: ::core::ffi::c_uint = 15;
/// What each typeahead byte may still be remapped through (its `noremap`
/// flags, and `KEY_NOREMAP` for the key last read).
pub const RM_YES: u8 = 0;
pub const RM_NONE: u8 = 1;
pub const RM_SCRIPT: u8 = 2;
pub const RM_ABBR: u8 = 4;
#[derive(Copy, Clone, Debug)]
pub(crate) struct GotcharsState {
    pub buf: [uint8_t; 67],
    /// The byte before this one: inside a key escape, its second byte.
    pub prev_byte: u8,
    pub buflen: size_t,
    pub pending_special: usize,
    pub pending_mbyte: usize,
}
pub const KEYLEN_PART_KEY: ::core::ffi::c_int = -1;
pub const SHOWCMD_COLS: ::core::ffi::c_uint = 10;
/// What [`handle_mapping`] did with the front of the typeahead.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum MapResult {
    /// It failed; the caller starts over.
    Fail,
    /// Nothing mapped: the typeahead has a byte to hand out.
    Get,
    /// Something was mapped or simplified; match the result again.
    Retry,
    /// More input is needed to decide.
    NoMatch,
}
pub const KEYLEN_PART_MAP: ::core::ffi::c_int = -2;
pub const FILE_READ_ONLY: FileOpenFlags = 1;
pub const FILE_NON_BLOCKING: FileOpenFlags = 128;
pub const FILE_TRUNCATE: FileOpenFlags = 32;
pub const FILE_CREATE_ONLY: FileOpenFlags = 16;
pub const FILE_NO_SYMLINK: FileOpenFlags = 8;
pub const FILE_WRITE_ONLY: FileOpenFlags = 4;
pub const FILE_CREATE: FileOpenFlags = 2;
/// The top bit of a channel id, which no real channel has: a call that
/// carries it came from inside nvim rather than over a channel.
pub const INTERNAL_CALL_MASK: uint64_t = 1 << (uint64_t::BITS - 1);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL + 1;

/// Whether `channel_id` names an internal caller rather than a channel.
fn is_internal_call(channel_id: uint64_t) -> bool {
    channel_id & INTERNAL_CALL_MASK != 0
}
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
static ON_KEY_BUF: GlobalCell<Vec<u8>> = GlobalCell::new(Vec::new());
static E_RECURSIVE_MAPPING: &::core::ffi::CStr = c"E223: Recursive mapping";
static E_CMD_MAPPING_MUST_END_WITH_CR: &::core::ffi::CStr =
    c"E1255: <Cmd> mapping must end with <CR>";
static E_CMD_MAPPING_MUST_END_WITH_CR_BEFORE_SECOND_CMD: &::core::ffi::CStr =
    c"E1136: <Cmd> mapping must end with <CR> before second <Cmd>";
pub const K_SELECT_STRING: &::core::ffi::CStr = c"\x80\xF5X";
