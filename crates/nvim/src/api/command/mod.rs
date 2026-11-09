#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::api::private::helpers::{
    api_dict_to_keydict, api_set_sctx, api_typename, find_buffer_by_handle, try_enter, try_leave,
};
use crate::autocmd::defs::AutoEvent;
use crate::autocmd::{apply_autocmds, has_event};
use crate::ex_docmd::{
    builtin_command_cstr, excmd_get_argt, execute_cmd, get_cmd_default_range, getargcmd, getargopt,
    invalid_range, is_cmd_ni, is_map_cmd, parse_cmdline, replace_makeprg, set_cmd_count,
    set_cmd_dflall_range, undo_cmdmod,
};
use crate::ex_eval::aborting;
use crate::lua::ffi::LUA_NOREF;
use crate::types::api::ERROR_TYPE_EXCEPTION;

use crate::api::keysets::{
    KeyDict_cmd, KeyDict_cmd_magic, KeyDict_cmd_mods, KeyDict_cmd_mods_filter, KeyDict_cmd_opts,
    KeyDict_empty, KeyDict_get_commands, KeyDict_user_command,
};
use crate::api::private::helpers::TryState;
use crate::lua::executor::{api_new_luaref, release_luaref};
use crate::mbyte::mb_islower;
use crate::message::state::{MSG_COL, REDIR_OFF};
use crate::message::{capture_finish, capture_start};
use crate::os::libc::{int64_t, size_t, uint64_t};
use crate::regexp::RE_MAGIC;
use crate::types::{
    ApiDict, ApiString, Array, BufferHandle, CmdAddr, CmdMod, CmdModFlags, CmdParseInfo,
    ERROR_TYPE_VALIDATION, Error, ExArg, Integer, LuaRef, OBJECT_TYPE_LUA_REF, OBJECT_TYPE_STRING,
    Object,
};
use crate::usercmd::{
    Definition, Table, commands_array, parse_addr_type_arg, parse_compl_arg, uc_add_command,
    uc_del_named, uc_nargs_upper_bound, uc_split_args_iter, uc_validate_name,
};
use crate::window::{WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_HOR, WSP_TOP, WSP_VERT};

// The carve of the transpiled module; see each child's docs.
mod cmd;
mod cmdline;
mod parse;
mod user;

pub use self::cmd::*;
pub(crate) use self::cmdline::*;
pub use self::parse::*;
pub use self::user::*;
