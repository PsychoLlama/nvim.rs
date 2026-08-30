#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::private::dispatch::{
    key_dict_cmd_magic_get_field, key_dict_cmd_mods_filter_get_field, key_dict_cmd_mods_get_field,
};
use crate::api::private::helpers::{
    api_dict_to_keydict, api_set_sctx, api_typename, arena_array, arena_dict, arena_string,
    cstr_as_string, cstrn_as_string, find_buffer_by_handle, string_to_cstr, try_enter, try_leave,
};
use crate::autocmd::{EVENT_CMDUNDEFINED, apply_autocmds, has_event};
use crate::charset::{skiptowhite, skipwhite};
use crate::ex_docmd::{
    excmd_get_argt, execute_cmd, find_ex_command, get_cmd_default_range, get_command_name,
    getargcmd, getargopt, invalid_range, is_cmd_ni, is_map_cmd, parse_cmdline, replace_makeprg,
    set_cmd_addr_type, set_cmd_count, set_cmd_dflall_range, undo_cmdmod,
};
use crate::ex_eval::aborting;

use crate::garray::{ga_clear, ga_init};
use crate::lua::executor::{api_free_luaref, api_new_luaref};
use crate::main::{capture_ga, curbuf, msg_col, redir_off};
use crate::mbyte::mb_islower;
use crate::memory::{arena_alloc, arena_memdupz, xcalloc, xfree, xrealloc};
use crate::os::cshim::snprintf;
use crate::regexp::{RE_MAGIC, vim_regcomp};
use crate::register::valid_yank_reg;
use crate::strings::kv_do_printf;
use crate::types::{
    Arena, Array, Buffer, CMD_SIZE, CMD_USER, CMD_USER_BUF, CMD_iput, CMD_put, CmdAddr,
    CmdModFlags, CmdParseInfo, Dict, Direction, Error, Integer, KeyDict_cmd, KeyDict_cmd_magic,
    KeyDict_cmd_mods, KeyDict_cmd_mods_filter, KeyDict_cmd_opts, KeyDict_empty,
    KeyDict_get_commands, KeyDict_user_command, LuaRef, Object, String_0, StringBuilder, TryState,
    buf_T, cmdmod_T, exarg_T, expand_T, garray_T, int64_t, kErrorTypeException, kErrorTypeNone,
    kErrorTypeValidation, kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeInteger,
    kObjectTypeLuaRef, kObjectTypeString, kObjectTypeTabpage, kObjectTypeWindow, linenr_T, size_t,
    ucmd_T, uint8_t, uint64_t,
};
use crate::usercmd::{
    Table, commands_array, get_user_command_name, parse_addr_type_arg, parse_compl_arg,
    uc_add_command, uc_del_command, uc_nargs_upper_bound, uc_split_args_iter, uc_validate_name,
};
use crate::window::{WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_HOR, WSP_TOP, WSP_VERT};
use ::libc::{memcpy, strcmp, strlen, strtol};

// The carve of the transpiled module; see each child's docs.
mod cmd;
mod cmdline;
mod parse;
mod user;

pub use self::cmd::*;
pub(crate) use self::cmdline::*;
pub use self::parse::*;
pub use self::user::*;
pub const kDirectionNotSet: Direction = 0;
pub const NUMBUFLEN: ::core::ffi::c_uint = 65;
pub const UC_BUFFER: ::core::ffi::c_uint = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__addr: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__count: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__force: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__nargs: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__range: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__preview: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__complete: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__cmd: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__reg: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__bang: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__addr: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__mods: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__args: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__count: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__magic: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__nargs: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__range: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__nextcmd: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_magic__bar: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_magic__file: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods__tab: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods__split: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods__filter: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods__verbose: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods_filter__pattern: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
