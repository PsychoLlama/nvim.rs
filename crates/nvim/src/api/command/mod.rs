#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::private::dispatch::{
    KeyDict_cmd_magic_get_field, KeyDict_cmd_mods_filter_get_field, KeyDict_cmd_mods_get_field,
};
use crate::api::private::helpers::{
    api_dict_to_keydict, api_set_error, api_set_sctx, api_typename, arena_array, arena_dict,
    arena_string, cstr_as_string, cstrn_as_string, find_buffer_by_handle, string_to_cstr,
    try_enter, try_leave,
};
use crate::api::private::validate::{api_err_exp, api_err_invalid, api_err_required};
use crate::ascii::ascii_iswhite;
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
use crate::main::{capture_ga, curbuf, current_sctx, msg_col, msg_silent, redir_off};
use crate::mbyte::mb_islower;
use crate::memory::{arena_alloc, arena_memdupz, xcalloc, xfree, xrealloc};
use crate::os::cshim::{memmove, snprintf, strncmp};
use crate::regexp::{RE_MAGIC, vim_regcomp};
use crate::register::valid_yank_reg;
use crate::strings::kv_do_printf;
use crate::types::{
    Arena, Array, Buffer, CMD_SIZE, CMD_USER, CMD_USER_BUF, CMD_iput, CMD_put, CMOD_BROWSE,
    CMOD_CONFIRM, CMOD_ERRSILENT, CMOD_HIDE, CMOD_KEEPALT, CMOD_KEEPJUMPS, CMOD_KEEPMARKS,
    CMOD_KEEPPATTERNS, CMOD_LOCKMARKS, CMOD_NOAUTOCMD, CMOD_NOSWAPFILE, CMOD_SANDBOX, CMOD_SILENT,
    CMOD_UNSILENT, CmdParseInfo, Dict, Direction, Error, Integer, KeyDict_cmd, KeyDict_cmd_magic,
    KeyDict_cmd_mods, KeyDict_cmd_mods_filter, KeyDict_cmd_opts, KeyDict_empty,
    KeyDict_get_commands, KeyDict_user_command, KeyValuePair, LuaRef, Object, String_0,
    StringBuilder, TryState, buf_T, cmd_addr_T, cmdmod_T, exarg_T, expand_T, garray_T, int64_t,
    kErrorTypeException, kErrorTypeNone, kErrorTypeValidation, kObjectTypeBoolean,
    kObjectTypeBuffer, kObjectTypeInteger, kObjectTypeLuaRef, kObjectTypeString,
    kObjectTypeTabpage, kObjectTypeWindow, linenr_T, sctx_T, size_t, ucmd_T, uint8_t, uint32_t,
    uint64_t,
};
use crate::usercmd::{
    commands_array, free_ucmd, get_user_command_name, parse_addr_type_arg, parse_compl_arg,
    uc_add_command, uc_nargs_upper_bound, uc_split_args_iter, uc_validate_name, ucmds,
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
pub const ADDR_NONE: cmd_addr_T = 11;
pub const ADDR_OTHER: cmd_addr_T = 10;
pub const ADDR_TABS: cmd_addr_T = 5;
pub const ADDR_BUFFERS: cmd_addr_T = 4;
pub const ADDR_LOADED_BUFFERS: cmd_addr_T = 3;
pub const ADDR_ARGUMENTS: cmd_addr_T = 2;
pub const ADDR_WINDOWS: cmd_addr_T = 1;
pub const ADDR_QUICKFIX: cmd_addr_T = 8;
pub const ADDR_LINES: cmd_addr_T = 0;
pub const kDirectionNotSet: Direction = 0;
pub const NUMBUFLEN: C2Rust_Unnamed_15 = 65;
pub const EXPAND_NOTHING: C2Rust_Unnamed_16 = 0;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_16 = 32;
pub const UC_BUFFER: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const EXPAND_SIGN: C2Rust_Unnamed_16 = 34;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_16 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_16 = 12;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_16 = 9;
pub const EXPAND_FILES: C2Rust_Unnamed_16 = 2;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
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
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const EX_RANGE: ::core::ffi::c_uint = 0x1 as ::core::ffi::c_uint;
pub const EX_BANG: ::core::ffi::c_uint = 0x2 as ::core::ffi::c_uint;
pub const EX_EXTRA: ::core::ffi::c_uint = 0x4 as ::core::ffi::c_uint;
pub const EX_XFILE: ::core::ffi::c_uint = 0x8 as ::core::ffi::c_uint;
pub const EX_NOSPC: ::core::ffi::c_uint = 0x10 as ::core::ffi::c_uint;
pub const EX_DFLALL: ::core::ffi::c_uint = 0x20 as ::core::ffi::c_uint;
pub const EX_NEEDARG: ::core::ffi::c_uint = 0x80 as ::core::ffi::c_uint;
pub const EX_TRLBAR: ::core::ffi::c_uint = 0x100 as ::core::ffi::c_uint;
pub const EX_REGSTR: ::core::ffi::c_uint = 0x200 as ::core::ffi::c_uint;
pub const EX_COUNT: ::core::ffi::c_uint = 0x400 as ::core::ffi::c_uint;
pub const EX_ZEROR: ::core::ffi::c_uint = 0x1000 as ::core::ffi::c_uint;
pub const EX_CMDARG: ::core::ffi::c_uint = 0x4000 as ::core::ffi::c_uint;
pub const EX_ARGOPT: ::core::ffi::c_uint = 0x20000 as ::core::ffi::c_uint;
pub const EX_SBOXOK: ::core::ffi::c_uint = 0x40000 as ::core::ffi::c_uint;
pub const EX_KEEPSCRIPT: ::core::ffi::c_uint = 0x4000000 as ::core::ffi::c_uint;
pub const EX_PREVIEW: ::core::ffi::c_uint = 0x8000000 as ::core::ffi::c_uint;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
