#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::private::helpers::{
    arena_array, arena_dict, arena_string, buf_get_text, cstr_as_string, dict_get_value,
    dict_set_var, find_buffer_by_handle, normalize_index, set_mark, try_enter, try_leave,
};
use crate::api::private::validate::check_string_array;
use crate::autocmd::{aucmd_prepbuf, aucmd_restbuf};
use crate::buffer::{buf_ensure_loaded, buf_get_changedtick, buf_meta_total, do_buffer};
use crate::buffer_updates::{buf_updates_register, buf_updates_unregister};
use crate::change::changed_lines;
use crate::cursor::{check_cursor_col, check_cursor_lnum, check_visual_pos};
use crate::ex_cmds::rename_buffer;
use crate::extmark::extmark_splice;

use crate::decoration::kMTMetaLines;
use crate::lua::executor::{kRetLuaref, nlua_call_ref};
use crate::lua::ffi::{lua_createtable, lua_pushlstring, lua_rawseti};
use crate::main::{State, curbuf, curwin, p_acd};
use crate::mapping::{keymap_array, modify_keymap};
use crate::mark::{mark_adjust_buf, mark_get};
use crate::memline::{
    ml_append_buf, ml_delete_buf, ml_find_line_or_offset, ml_get_buf, ml_get_buf_len,
    ml_replace_buf,
};
use crate::memory::{
    arena_alloc, arena_allocz, arena_memdupz, memchrsub, strchrsub, xfree, xmemdupz,
};
use crate::r#move::{changed_cline_bef_curs, invalidate_botline_win, update_topline};
use crate::ops::get_region_bytecount;
use crate::os::cshim::strchr;
use crate::pos::{MAXCOL, MAXLNUM};
use crate::search::FORWARD;
use crate::state::MODE_INSERT;
use crate::types::{
    AlignTextPos, Arena, Array, Boolean, BufUpdateCallbacks, Buffer, Dict, Error, ExtmarkOp,
    Integer, KeyDict_buf_attach, KeyDict_buf_delete, KeyDict_empty, KeyDict_keymap, KeyValuePair,
    LuaRef, MarkAdjustMode, MarkGet, Object, String_0, TryState, UndoObjectType, WinSplit,
    WinStyle, aco_save_T, bcount_t, buf_T, colnr_T, dobuf_action_values, dobuf_start_values,
    except_T, fmark_T, int64_t, kErrorTypeNone, linenr_T, lua_State, msglist_T, pos_T, ptrdiff_t,
    size_t, uint64_t, win_T,
};
use crate::undo::u_save_buf;
use ::libc::memcpy;

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
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitLeft: WinSplit = 0;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const kMarkAdjustTerm: MarkAdjustMode = 2;
pub const kMarkAdjustApi: MarkAdjustMode = 1;
pub const kMarkAdjustNormal: MarkAdjustMode = 0;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const kMarkAllNoResolve: MarkGet = 2;
pub const kMarkBufLocal: MarkGet = 0;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const INTERNAL_CALL_MASK: uint64_t = (1 as ::core::ffi::c_int as uint64_t)
    << ::core::mem::size_of::<uint64_t>()
        .wrapping_mul(8_usize)
        .wrapping_sub(1_usize);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL.wrapping_add(1 as uint64_t);
pub const KEYSET_OPTIDX_buf_attach__on_bytes: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_buf_attach__on_lines: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_buf_attach__on_detach: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_buf_attach__on_reload: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_buf_attach__on_changedtick: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const BUF_UPDATE_CALLBACKS_INIT: BufUpdateCallbacks = BufUpdateCallbacks {
    on_lines: LUA_NOREF,
    on_bytes: LUA_NOREF,
    on_changedtick: LUA_NOREF,
    on_detach: LUA_NOREF,
    on_reload: LUA_NOREF,
    utf_sizes: false,
    preview: false,
};
