#![deny(unsafe_op_in_unsafe_fn)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::api::private::dispatch::{
    key_dict_cmd_magic_get_field, key_dict_cmd_mods_filter_get_field, key_dict_cmd_mods_get_field,
};
use crate::api::private::helpers::{
    api_dict_to_keydict, api_set_sctx, api_typename, cstr_to_string, cstrn_to_string,
    find_buffer_by_handle, string_to_cstr, try_enter, try_leave,
};
use crate::autocmd::{apply_autocmds, has_event};
use crate::charset::{skiptowhite, skipwhite};
use crate::ex_docmd::{
    excmd_get_argt, execute_cmd, find_ex_command, get_cmd_default_range, get_command_name,
    getargcmd, getargopt, invalid_range, is_cmd_ni, is_map_cmd, parse_cmdline, replace_makeprg,
    set_cmd_addr_type, set_cmd_count, set_cmd_dflall_range, undo_cmdmod,
};
use crate::ex_eval::aborting;
use crate::types::AutoEvent;

use crate::garray::{ga_clear, ga_init};
use crate::lua::executor::{api_free_luaref, api_new_luaref};
use crate::mbyte::mb_islower;
use crate::memory::{arena_alloc, xfree};
use crate::message::state::{capture_ga, msg_col, redir_off};
use crate::os::cshim::snprintf;
use crate::regexp::{RE_MAGIC, vim_regcomp};
use crate::register::valid_yank_reg;
use crate::types::{
    ApiDict, Arena, Array, BufferHandle, CmdAddr, CmdMod, CmdModFlags, CmdParseInfo, Direction,
    Error, ExArg, Expand, GArray, Integer, KeyDict_cmd, KeyDict_cmd_magic, KeyDict_cmd_mods,
    KeyDict_cmd_mods_filter, KeyDict_cmd_opts, KeyDict_empty, KeyDict_get_commands,
    KeyDict_user_command, LineNr, LuaRef, Object, String_0, TryState, UserCmd, int64_t,
    kErrorTypeException, kErrorTypeValidation, kObjectTypeLuaRef, kObjectTypeString, size_t,
    uint8_t, uint64_t,
};
use crate::usercmd::{
    Table, commands_array, get_user_command_name, parse_addr_type_arg, parse_compl_arg,
    uc_add_command, uc_del_command, uc_nargs_upper_bound, uc_split_args_iter, uc_validate_name,
};
use crate::window::{WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_HOR, WSP_TOP, WSP_VERT};
use ::libc::strtol;

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
