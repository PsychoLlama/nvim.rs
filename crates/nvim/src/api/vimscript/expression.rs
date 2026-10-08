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
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api_error;
use crate::message_fmt::msg_bytes;
use crate::types::NodeId;
use core::ffi::{CStr, c_char, c_int, c_uint};

pub fn nvim_parse_expression(
    expr: String_0,
    flags: String_0,
    hl: Boolean,
) -> Result<ApiDict, Error> {
    let error = Error::none();
    let pflags = parse_flags(&flags)?;

    // One line; a null string is no line at all, as a null `data` ended
    // upstream's reader before it began.
    let line = [expr.as_bytes()];
    let input: &[&[u8]] = if expr.is_null() { &[] } else { &line };
    let mut pstate = ParserState::new(input, hl);
    let east = viml_pexpr_parse(&mut pstate, pflags);

    // "len" and "ast", plus "error" and "highlight" when they apply.
    let ret_size = 2 + size_t::from(east.err.is_some()) + size_t::from(hl);
    let mut ret: ApiDict = ApiDict::with_capacity(ret_size);
    // A multi-line expression stops at the end of the first line.
    let consumed = if pstate.pos.line == 1 {
        expr.len()
    } else {
        pstate.pos.col
    };
    ret.insert(c"len", Object::integer(consumed as Integer));

    if let Some(err) = &east.err {
        let mut err_dict: ApiDict = ApiDict::with_capacity(2);
        let arg = err.arg.map_or(String_0::NULL, String_0::from_bytes);
        err_dict.insert(c"message", Object::string(String_0::from_cstr(err.msg)));
        err_dict.insert(c"arg", Object::string(arg));
        ret.insert(c"error", Object::dict(err_dict));
    }

    if hl {
        let colors = pstate.take_highlight();
        let mut hl_arr: Array = Array::with_capacity(colors.len());
        for chunk in &colors {
            let mut chunk_arr: Array = Array::with_capacity(4);
            chunk_arr.push(Object::integer(chunk.start.line as Integer));
            chunk_arr.push(Object::integer(chunk.start.col as Integer));
            chunk_arr.push(Object::integer(chunk.end_col as Integer));
            chunk_arr.push(Object::string(String_0::from_cstr(chunk.group)));
            hl_arr.push(Object::array(chunk_arr));
        }
        ret.insert(c"highlight", Object::array(hl_arr));
    }

    ret.insert(c"ast", convert_ast(&east));
    debug_assert!(ret.len() == ret.capacity(), "ret.len() == ret.capacity()");
    ret.reported(error)
}

/// The `flags` argument as `ExprParserFlags`, or which character was not one.
fn parse_flags(flags: &String_0) -> Result<c_int, Error> {
    let mut pflags: c_int = 0;
    for &ch in flags.as_bytes() {
        match ch {
            b'm' => pflags |= kExprFlagsMulti as c_int,
            b'E' => pflags |= kExprFlagsDisallowEOC as c_int,
            b'l' => pflags |= kExprFlagsParseLet as c_int,
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
                let code = ch as c_char as c_uint;
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
                let mut ret_node = ApiDict::with_capacity(node_dict_size(node));
                let num_children = east.children(id).count();
                if num_children != 0 {
                    let children = done.split_off(done.len() - num_children);
                    let mut children_array: Array = Array::with_capacity(num_children);
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
    let has_scope = type_0 == kExprNodeOption || type_0 == kExprNodePlainIdentifier;
    let has_ident = has_scope || type_0 == kExprNodePlainKey || type_0 == kExprNodeEnvironment;
    3 + size_t::from(node.children.is_some())
        + size_t::from(has_scope)
        + size_t::from(has_ident)
        + size_t::from(type_0 == kExprNodeRegister)
        // cmp_type, ccs_strategy and invert.
        + 3 * size_t::from(type_0 == kExprNodeComparison)
        + size_t::from(type_0 == kExprNodeInteger)
        + size_t::from(type_0 == kExprNodeFloat)
        + size_t::from(
            type_0 == kExprNodeDoubleQuotedString || type_0 == kExprNodeSingleQuotedString,
        )
        + size_t::from(type_0 == kExprNodeAssignment)
}

/// The pairs a node contributes once its children have been rendered: the
/// three every node has, then whatever its own variant carries. `ret_node`
/// was sized by [`node_dict_size`].
fn finish_node(node: &ExprASTNode<'_>, ret_node: &mut ApiDict) {
    let type_0 = node.type_0;
    let put = |dict: &mut ApiDict, key: &'static CStr, value: Object| {
        dict.insert(key, value);
    };
    // The three name tables hold static C strings.
    let table_name = |name: &CStr| Object::string(String_0::from_cstr(name));
    let bytes = |value: &[u8]| Object::string(String_0::from_bytes(value));

    let type_name = NODE_TYPE_NAMES[type_0 as usize];
    put(ret_node, c"type", table_name(type_name));

    let mut start_array: Array = Array::with_capacity(2);
    start_array.push(Object::integer(node.start.line as Integer));
    start_array.push(Object::integer(node.start.col as Integer));
    put(ret_node, c"start", Object::array(start_array));
    put(ret_node, c"len", Object::integer(node.len as Integer));

    let data = &node.data;
    match type_0 {
        kExprNodeDoubleQuotedString | kExprNodeSingleQuotedString => {
            // An unterminated or empty literal may have no body at all.
            let value = data.string().value.as_deref();
            let str = value.map_or(Object::string(String_0::NULL), bytes);
            put(ret_node, c"svalue", str);
        }
        kExprNodeOption => {
            put(
                ret_node,
                c"scope",
                Object::integer(data.option().scope as Integer),
            );
            put(ret_node, c"ident", bytes(data.option().ident));
        }
        kExprNodePlainIdentifier => {
            put(
                ret_node,
                c"scope",
                Object::integer(data.variable().scope as Integer),
            );
            put(ret_node, c"ident", bytes(data.variable().ident));
        }
        kExprNodePlainKey => {
            put(ret_node, c"ident", bytes(data.variable().ident));
        }
        kExprNodeEnvironment => {
            put(ret_node, c"ident", bytes(data.environment().ident));
        }
        kExprNodeRegister => {
            put(
                ret_node,
                c"name",
                Object::integer(data.register().name as Integer),
            );
        }
        kExprNodeComparison => {
            let cmp = COMPARISON_NAMES[data.comparison().type_0 as usize];
            put(ret_node, c"cmp_type", table_name(cmp));
            let ccs = CASE_STRATEGY_NAMES[data.comparison().ccs as usize];
            put(
                ret_node,
                c"ccs_strategy",
                ccs.map_or(Object::string(String_0::NULL), table_name),
            );
            put(ret_node, c"invert", Object::boolean(data.comparison().inv));
        }
        kExprNodeFloat => {
            put(ret_node, c"fvalue", Object::float(data.float().value));
        }
        kExprNodeInteger => {
            // The lexer's value is unsigned; the wire's is not.
            let value = data.integer().value.min(Integer::MAX as UVarNumber);
            put(ret_node, c"ivalue", Object::integer(value as Integer));
        }
        kExprNodeAssignment => {
            let asgn_type = data.assignment().type_0;
            // Plain "=" has no augmentation, and the table's slot for it is
            // the empty string rather than a name.
            let augmentation = if asgn_type == kExprAsgnPlain {
                Object::string(String_0::NULL)
            } else {
                table_name(ASSIGNMENT_NAMES[asgn_type as usize])
            };
            put(ret_node, c"augmentation", augmentation);
        }
        _ => {}
    }
}
