//! `nvim_parse_expression()`: an expression as an AST.
//!
//! The parser's whole output rendered as data: the length consumed, the error
//! if the parse failed, the highlight chunks when `hl` is set, and the AST
//! itself -- walked iteratively with an explicit stack rather than recursively,
//! because an expression nests arbitrarily deep.
//!
//! Every container here is sized *exactly* before it is filled: `arena_dict`
//! and `arena_array` take a capacity and the pushes must add up to it, which
//! is what [`node_dict_size`] is for and what the assertions at the two exits
//! check.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use super::*;
use crate::api_error;
use crate::message_fmt::msg_bytes;
use crate::narrow::len_as_integer;
use crate::types::DictBuilder;
use crate::viml::parser::expressions::EXPR_ASGN_PLAIN;
use crate::viml::parser::expressions::EXPR_FLAGS_DISALLOW_EOC;
use crate::viml::parser::expressions::EXPR_FLAGS_MULTI;
use crate::viml::parser::expressions::EXPR_FLAGS_PARSE_LET;
use crate::viml::parser::expressions::EXPR_NODE_ASSIGNMENT;
use crate::viml::parser::expressions::EXPR_NODE_COMPARISON;
use crate::viml::parser::expressions::EXPR_NODE_DOUBLE_QUOTED_STRING;
use crate::viml::parser::expressions::EXPR_NODE_ENVIRONMENT;
use crate::viml::parser::expressions::EXPR_NODE_FLOAT;
use crate::viml::parser::expressions::EXPR_NODE_INTEGER;
use crate::viml::parser::expressions::EXPR_NODE_OPTION;
use crate::viml::parser::expressions::EXPR_NODE_PLAIN_IDENTIFIER;
use crate::viml::parser::expressions::EXPR_NODE_PLAIN_KEY;
use crate::viml::parser::expressions::EXPR_NODE_REGISTER;
use crate::viml::parser::expressions::EXPR_NODE_SINGLE_QUOTED_STRING;
use crate::viml::parser::expressions::NodeId;
use core::ffi::{CStr, c_int, c_uint};

pub fn nvim_parse_expression(
    expr: ApiString,
    flags: ApiString,
    hl: Boolean,
) -> Result<ApiDict, Error> {
    let pflags = parse_flags(&flags)?;

    // One line; a null string is no line at all, as a null `data` ended
    // upstream's reader before it began.
    let line = [expr.as_bytes()];
    let input: &[&[u8]] = if expr.is_null() { &[] } else { &line };
    let mut pstate = ParserState::new(input, hl);
    let east = viml_pexpr_parse(&mut pstate, pflags);

    // "len" and "ast", plus "error" and "highlight" when they apply.
    let ret_size = 2 + size_t::from(east.err.is_some()) + size_t::from(hl);
    let mut ret = DictBuilder::with_capacity(ret_size);
    // A multi-line expression stops at the end of the first line.
    let consumed = if pstate.pos.line == 1 {
        expr.len()
    } else {
        pstate.pos.col
    };
    ret.insert(c"len", Object::integer(len_as_integer(consumed)));

    if let Some(err) = &east.err {
        let mut err_dict = DictBuilder::with_capacity(2);
        let arg = err.arg.map_or(ApiString::NULL, ApiString::from_bytes);
        err_dict.insert(c"message", Object::string(ApiString::from_cstr(err.msg)));
        err_dict.insert(c"arg", Object::string(arg));
        ret.insert(c"error", Object::dict(err_dict));
    }

    if hl {
        let colors = pstate.take_highlight();
        let mut hl_arr = Vec::with_capacity(colors.len());
        for chunk in &colors {
            let chunk_arr = vec![
                Object::integer(len_as_integer(chunk.start.line)),
                Object::integer(len_as_integer(chunk.start.col)),
                Object::integer(len_as_integer(chunk.end_col)),
                Object::string(ApiString::from_cstr(chunk.group)),
            ];
            hl_arr.push(Object::array(chunk_arr));
        }
        ret.insert(c"highlight", Object::array(hl_arr));
    }

    ret.insert(c"ast", convert_ast(&east));
    debug_assert!(ret.len() == ret.capacity(), "ret.len() == ret.capacity()");
    Ok(ret.into())
}

/// The `flags` argument as `ExprParserFlags`, or which character was not one.
fn parse_flags(flags: &ApiString) -> Result<c_int, Error> {
    let mut pflags: c_int = 0;
    for &ch in flags.as_bytes() {
        match ch {
            b'm' => pflags |= EXPR_FLAGS_MULTI.cast_signed(),
            b'E' => pflags |= EXPR_FLAGS_DISALLOW_EOC.cast_signed(),
            b'l' => pflags |= EXPR_FLAGS_PARSE_LET.cast_signed(),
            // A NUL has no `%c` spelling worth printing.
            0 => {
                let code = c_uint::from(ch);
                return Err(api_error!(
                    kErrorTypeValidation,
                    "Invalid flag: '\\0' ({code})"
                ));
            }
            _ => {
                // The C prints the flag as a `char`'s `%u`.
                let code = c_int::from(ch.cast_signed()).cast_unsigned();
                let shown = msg_bytes(core::slice::from_ref(&ch));
                return Err(api_error!(
                    kErrorTypeValidation,
                    "Invalid flag: '{shown}' ({code})"
                ));
            }
        }
    }
    Ok(pflags)
}

/// The tree rendered as nested dictionaries, `nil` for an empty parse.
///
/// Iterative, because an expression nests as deep as the input says: each
/// node is entered once, its children rendered first, and its dictionary
/// built when it is left from the rendered children on top of `done`.
fn convert_ast(east: &ExprAST<'_>) -> Object {
    #[derive(Debug)]
    enum Step {
        Enter(NodeId),
        Leave(NodeId),
    }
    let Some(root) = east.root else {
        return Object::Nil;
    };
    let mut steps = vec![Step::Enter(root)];
    let mut done: Vec<Object> = Vec::new();
    while let Some(step) = steps.pop() {
        match step {
            Step::Enter(id) => {
                steps.push(Step::Leave(id));
                // Pushed last to first, so the first child is rendered first.
                let children: Vec<NodeId> = east.children(id).collect();
                steps.extend(children.into_iter().rev().map(Step::Enter));
            }
            Step::Leave(id) => {
                let node = east.node(id);
                let mut ret_node = DictBuilder::with_capacity(node_dict_size(node));
                let num_children = east.children(id).count();
                if num_children != 0 {
                    let children = done.split_off(done.len() - num_children);
                    let mut children_array = Vec::with_capacity(num_children);
                    for child in children {
                        children_array.push(child);
                    }
                    ret_node.insert(c"children", Object::array(children_array));
                }
                finish_node(node, &mut ret_node);
                debug_assert!(
                    ret_node.len() == ret_node.capacity(),
                    "the node dictionary was sized for exactly the keys it holds"
                );
                done.push(Object::dict(ret_node));
            }
        }
    }
    debug_assert!(done.len() == 1, "the walk renders the root last");
    done.pop().unwrap_or(Object::Nil)
}

/// How many pairs [`finish_node`] will put in a node's dictionary. The three
/// every node gets are "type", "start" and "len".
fn node_dict_size(node: &ExprASTNode) -> size_t {
    let type_0 = node.type_0;
    let has_scope = type_0 == EXPR_NODE_OPTION || type_0 == EXPR_NODE_PLAIN_IDENTIFIER;
    let has_ident = has_scope || type_0 == EXPR_NODE_PLAIN_KEY || type_0 == EXPR_NODE_ENVIRONMENT;
    3 + size_t::from(node.children.is_some())
        + size_t::from(has_scope)
        + size_t::from(has_ident)
        + size_t::from(type_0 == EXPR_NODE_REGISTER)
        // cmp_type, ccs_strategy and invert.
        + 3 * size_t::from(type_0 == EXPR_NODE_COMPARISON)
        + size_t::from(type_0 == EXPR_NODE_INTEGER)
        + size_t::from(type_0 == EXPR_NODE_FLOAT)
        + size_t::from(
            type_0 == EXPR_NODE_DOUBLE_QUOTED_STRING || type_0 == EXPR_NODE_SINGLE_QUOTED_STRING,
        )
        + size_t::from(type_0 == EXPR_NODE_ASSIGNMENT)
}

/// The pairs a node contributes once its children have been rendered: the
/// three every node has, then whatever its own variant carries. `ret_node`
/// was sized by [`node_dict_size`].
fn finish_node(node: &ExprASTNode<'_>, ret_node: &mut DictBuilder) {
    let type_0 = node.type_0;
    let put = |dict: &mut DictBuilder, key: &'static CStr, value: Object| {
        dict.insert(key, value);
    };
    // The three name tables hold static C strings.
    let table_name = |name: &CStr| Object::string(ApiString::from_cstr(name));
    let bytes = |value: &[u8]| Object::string(ApiString::from_bytes(value));

    let type_name = NODE_TYPE_NAMES[type_0 as usize];
    put(ret_node, c"type", table_name(type_name));

    let start_array = vec![
        Object::integer(len_as_integer(node.start.line)),
        Object::integer(len_as_integer(node.start.col)),
    ];
    put(ret_node, c"start", Object::array(start_array));
    put(ret_node, c"len", Object::integer(len_as_integer(node.len)));

    let data = &node.data;
    match type_0 {
        EXPR_NODE_DOUBLE_QUOTED_STRING | EXPR_NODE_SINGLE_QUOTED_STRING => {
            // An unterminated or empty literal may have no body at all.
            let value = data.string().value.as_deref();
            let str = value.map_or(Object::string(ApiString::NULL), bytes);
            put(ret_node, c"svalue", str);
        }
        EXPR_NODE_OPTION => {
            put(
                ret_node,
                c"scope",
                Object::integer(Integer::from(data.option().scope)),
            );
            put(ret_node, c"ident", bytes(data.option().ident));
        }
        EXPR_NODE_PLAIN_IDENTIFIER => {
            put(
                ret_node,
                c"scope",
                Object::integer(Integer::from(data.variable().scope)),
            );
            put(ret_node, c"ident", bytes(data.variable().ident));
        }
        EXPR_NODE_PLAIN_KEY => {
            put(ret_node, c"ident", bytes(data.variable().ident));
        }
        EXPR_NODE_ENVIRONMENT => {
            put(ret_node, c"ident", bytes(data.environment().ident));
        }
        EXPR_NODE_REGISTER => {
            put(
                ret_node,
                c"name",
                Object::integer(Integer::from(data.register().name)),
            );
        }
        EXPR_NODE_COMPARISON => {
            let cmp = COMPARISON_NAMES[data.comparison().type_0 as usize];
            put(ret_node, c"cmp_type", table_name(cmp));
            let ccs = CASE_STRATEGY_NAMES[data.comparison().ccs as usize];
            put(
                ret_node,
                c"ccs_strategy",
                ccs.map_or(Object::string(ApiString::NULL), table_name),
            );
            put(ret_node, c"invert", Object::boolean(data.comparison().inv));
        }
        EXPR_NODE_FLOAT => {
            put(ret_node, c"fvalue", Object::float(data.float().value));
        }
        EXPR_NODE_INTEGER => {
            // The lexer's value is unsigned; the wire's is not.
            let value = Integer::try_from(data.integer().value).unwrap_or(Integer::MAX);
            put(ret_node, c"ivalue", Object::integer(value));
        }
        EXPR_NODE_ASSIGNMENT => {
            let asgn_type = data.assignment().type_0;
            // Plain "=" has no augmentation, and the table's slot for it is
            // the empty string rather than a name.
            let augmentation = if asgn_type == EXPR_ASGN_PLAIN {
                Object::string(ApiString::NULL)
            } else {
                table_name(ASSIGNMENT_NAMES[asgn_type as usize])
            };
            put(ret_node, c"augmentation", augmentation);
        }
        _ => {}
    }
}
