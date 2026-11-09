#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::api::keysets::KeyDict_exec_opts;
use crate::api::private::helpers::api_set_sctx;
use crate::eval::eval0;
use crate::eval::typval::tv_clear;
use crate::eval::userfunc::MAX_FUNC_ARGS;
use crate::ex_docmd::do_cmdline_cmd;
use crate::ex_eval::state::{DID_THROW, FORCE_ABORT, SUPPRESS_ERRTHROW};
use crate::global_cell::GlobalCell;
use crate::message::state::{DID_EMSG, MSG_COL, REDIR_OFF};
use crate::message::{capture_finish, capture_start};
use crate::os::libc::{size_t, uint64_t};
use crate::runtime::do_source_str;
use crate::types::api::ERROR_TYPE_EXCEPTION;
use crate::types::{
    ApiDict, ApiString, Array, Boolean, ERROR_TYPE_VALIDATION, Error, Integer, Object, TypVal,
    VAR_FUNC, VAR_PARTIAL,
};
use crate::viml::parser::expressions::{
    ASSIGNMENT_NAMES, CASE_STRATEGY_NAMES, COMPARISON_NAMES, NODE_TYPE_NAMES, viml_pexpr_parse,
};
use crate::viml::parser::expressions::{ExprAST, ExprASTNode};
use crate::viml::parser::parser::ParserState;

// The carve of the transpiled module; see each child's docs.
mod eval;
mod exec;
mod expression;

pub use self::eval::*;
pub use self::exec::*;
pub use self::expression::*;
