#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::api::private::helpers::{
    buf_get_text, dict_get_value, dict_set_var, find_buffer_by_handle, normalize_index, set_mark,
    try_enter, try_leave,
};
use crate::api::private::validate::check_string_array;
use crate::autocmd::{aucmd_prepbuf, aucmd_restbuf};
use crate::buffer::{buf_ensure_loaded, buf_get_changedtick, buf_meta_total, do_buffer};
use crate::buffer_updates::{buf_updates_register, buf_updates_unregister};
use crate::change::changed_lines;
use crate::cursor::{check_cursor_col, check_cursor_lnum, check_visual_pos};
use crate::ex_cmds::rename_buffer;
use crate::extmark::extmark_splice;

use crate::decoration::MT_META_LINES;
use crate::lua::executor::{RET_LUAREF, nlua_call_ref_reporting};
use crate::mapping::{keymap_array, modify_keymap};
use crate::mark::{mark_adjust_buf, mark_get};
use crate::memline::{ml_append_buf_terminated, ml_delete_buf, ml_get_buf_len, ml_replace_buf_now};

use crate::api::keysets::{KeyDict_buf_attach, KeyDict_buf_delete, KeyDict_empty, KeyDict_keymap};
use crate::api::private::helpers::TryState;
use crate::autocmd::defs::AcoSave;
use crate::extmark::BCount;
use crate::r#move::{changed_cline_bef_curs, invalidate_botline_win, update_topline};
use crate::ops::get_region_bytecount;
use crate::option::vars::{P_ACD, p_acd};
use crate::os::libc::{size_t, uint64_t};
use crate::pos::{MAXCOL, MAXLNUM};
use crate::search::FORWARD;
use crate::state::MODE_INSERT;
use crate::state::mode::STATE;
use crate::types::{
    ApiDict, ApiString, Array, Boolean, BufUpdateCallbacks, BufferHandle, ColNr, Error, Integer,
    LineNr, LuaRef, Object, Pos,
};
use crate::undo::u_save_buf;

// The carve of the transpiled module; see each child's docs.
mod attach;
mod lines;
mod marks;
mod props;
mod text;

pub use self::attach::*;
pub use self::lines::*;
pub use self::marks::*;
pub use self::props::*;
pub use self::text::*;
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const INTERNAL_CALL_MASK: uint64_t = 1_u64
    << ::core::mem::size_of::<uint64_t>()
        .wrapping_mul(8_usize)
        .wrapping_sub(1_usize);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL.wrapping_add(1 as uint64_t);
pub const BUF_UPDATE_CALLBACKS_INIT: BufUpdateCallbacks = BufUpdateCallbacks {
    on_lines: LUA_NOREF,
    on_bytes: LUA_NOREF,
    on_changedtick: LUA_NOREF,
    on_detach: LUA_NOREF,
    on_reload: LUA_NOREF,
    utf_sizes: false,
    preview: false,
};
