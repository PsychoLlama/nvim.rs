#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::api::buffer::{nvim_buf_get_lines, nvim_buf_set_lines};
use crate::api::extmark::{nvim_buf_clear_namespace, nvim_create_namespace, parse_virt_text};
use crate::api::keysets::{KeyDict_empty, KeyDict_exec_opts};
use crate::api::private::dispatch::msgpack_rpc_get_handler_for;
use crate::api::private::helpers::{api_set_sctx, api_typename, dict_set_var};
use crate::decoration::defs::VirtText;
use crate::decoration::defs::{
    DecorExt, DecorInline, DecorPriority, DecorVirtText, DecorVirtText_data,
};
use crate::decoration::{
    HL_MODE_UNKNOWN, V_POS_END_OF_LINE, decor_find_virttext, decor_inline_hl, decor_put_vt,
    with_decor_virt_mut,
};
use crate::eval::vars::globvar_dict;
use crate::getchar::state::GOT_INT;
use crate::global_cell::GlobalCell;
use crate::highlight::hl_get_attr_by_id;
use crate::highlight_group::{syn_check_group, syn_get_final_id, syn_id2attr, syn_name2id};
use crate::lua::executor::nlua_exec;
use crate::message::msg_end;
use crate::message::state::{MSG_DIDOUT, MSG_SILENT};
use crate::msgpack_rpc::handler::MsgpackRpcRequestHandler;
use crate::option::defs::{OptIndex, OptVal};
use crate::option::{
    find_option, get_option_value_for, get_vimoption, object_as_optval, option_has_scope,
    set_option_value_for,
};
use crate::options::OPT_INVALID;
use crate::os::libc::{int64_t, size_t, uint8_t, uint32_t, uint64_t};
use crate::pos::{MAXCOL, MAXLNUM};
use crate::types::{
    ApiDict, ApiString, Array, Boolean, BufferHandle, ColNr, Error, Integer, OBJECT_TYPE_ARRAY,
    OBJECT_TYPE_STRING, Object, TabpageHandle, WindowHandle,
};

// The carve of the transpiled module; see each child's docs.
mod bufhl;
mod eval;
mod highlight;
mod lines;
mod options;
mod vars;
mod write;

pub use self::bufhl::*;
pub use self::eval::*;
pub use self::highlight::*;
pub use self::lines::*;
pub use self::options::*;
pub use self::vars::*;
pub use self::write::*;
pub const LINE_BUFFER_MIN_SIZE: ::core::ffi::c_uint = 4096;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub(crate) use crate::decoration::DECOR_ID_INVALID;
