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

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported, array_add, dict_put};
use crate::kvec::InitVec;
use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

/// One frame of the AST walk: where the node pointer lives (so the walk can
/// free it and NULL it out) and where its rendered form goes.
#[derive(Copy, Clone)]
struct ConvFrame {
    node_p: *mut *mut ExprASTNode,
    ret_node_p: *mut Object,
}

pub unsafe fn nvim_parse_expression(
    expr: String_0,
    flags: String_0,
    hl: Boolean,
    arena: *mut Arena,
) -> Result<Dict, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let Some(pflags) = parse_flags(flags, err) else {
            return Dict::EMPTY.reported(error);
        };

        let mut parser_lines: [ParserLine; 2] = [
            ParserLine {
                data: expr.data(),
                size: expr.len(),
                allocated: false,
            },
            ParserLine {
                data: ptr::null::<c_char>(),
                size: 0,
                allocated: false,
            },
        ];
        let mut plines_p: *mut ParserLine = parser_lines.as_mut_ptr();
        let mut colors: ParserHighlight = ParserHighlight {
            size: 0,
            capacity: 0,
            items: ptr::null_mut::<ParserHighlightChunk>(),
            init_array: [ParserHighlightChunk {
                start: ParserPosition { line: 0, col: 0 },
                end_col: 0,
                group: ptr::null::<c_char>(),
            }; 16],
        };
        colors.capacity = colors.init_array.len();
        colors.items = colors.init_array.as_mut_ptr();
        let colors_p: *mut ParserHighlight = if hl {
            &raw mut colors
        } else {
            ptr::null_mut::<ParserHighlight>()
        };
        let mut pstate: ParserState = ::core::mem::zeroed();
        viml_parser_init(
            &raw mut pstate,
            Some(parser_simple_get_line),
            (&raw mut plines_p).cast(),
            colors_p,
        );
        let mut east: ExprAST = viml_pexpr_parse(&raw mut pstate, pflags);

        // "len" and "ast", plus "error" and "highlight" when they apply.
        let ret_size = 2 + size_t::from(!east.err.msg.is_null()) + size_t::from(hl);
        let mut ret: Dict = arena_dict(arena, ret_size);
        // A multi-line expression stops at the end of the first line.
        let consumed = if pstate.pos.line == 1 {
            parser_lines[0].size
        } else {
            pstate.pos.col
        };
        dict_put(&mut ret, c"len", Object::integer(consumed as Integer));

        if !east.err.msg.is_null() {
            let mut err_dict: Dict = arena_dict(arena, 2);
            dict_put(
                &mut err_dict,
                c"message",
                Object::string(arena_string(arena, cstr_as_string(east.err.msg))),
            );
            let arg = String_0::from_raw_parts(east.err.arg.cast_mut(), east.err.arg_len as size_t);
            dict_put(
                &mut err_dict,
                c"arg",
                Object::string(arena_string(arena, arg)),
            );
            dict_put(&mut ret, c"error", Object::dict(err_dict));
        }

        if hl {
            let mut hl_arr: Array = arena_array(arena, colors.size);
            for i in 0..colors.size {
                let chunk: ParserHighlightChunk = *colors.items.add(i);
                let mut chunk_arr: Array = arena_array(arena, 4);
                array_add(&mut chunk_arr, Object::integer(chunk.start.line as Integer));
                array_add(&mut chunk_arr, Object::integer(chunk.start.col as Integer));
                array_add(&mut chunk_arr, Object::integer(chunk.end_col as Integer));
                array_add(&mut chunk_arr, Object::string(cstr_as_string(chunk.group)));
                array_add(&mut hl_arr, Object::array(chunk_arr));
            }
            dict_put(&mut ret, c"highlight", Object::array(hl_arr));
        }
        let heap = InitVec::new(
            &mut colors.size,
            &mut colors.capacity,
            &mut colors.items,
            &mut colors.init_array,
        )
        .take_heap();
        xfree(heap);

        let mut ast = Object::NIL;
        convert_ast(arena, &raw mut east.root, &raw mut ast);
        dict_put(&mut ret, c"ast", ast);
        debug_assert!(ret.size == ret.capacity, "ret.size == ret.capacity");

        viml_pexpr_free_ast(east);
        viml_parser_destroy(&mut pstate);
        ret.reported(error)
    }
}

/// The `flags` argument as `ExprParserFlags`, or `None` after reporting which
/// character was not one.
unsafe fn parse_flags(flags: String_0, err: *mut Error) -> Option<c_int> {
    unsafe {
        let mut pflags: c_int = 0;
        for i in 0..flags.len() {
            let ch: c_char = *flags.data().add(i);
            match ch as u8 {
                b'm' => pflags |= kExprFlagsMulti as c_int,
                b'E' => pflags |= kExprFlagsDisallowEOC as c_int,
                b'l' => pflags |= kExprFlagsParseLet as c_int,
                // A NUL has no `%c` spelling worth printing.
                0 => {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        c"Invalid flag: '\\0' (%u)".as_ptr(),
                        ch as c_uint,
                    );
                    return None;
                }
                _ => {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        c"Invalid flag: '%c' (%u)".as_ptr(),
                        ch as c_int,
                        ch as c_uint,
                    );
                    return None;
                }
            }
        }
        Some(pflags)
    }
}

/// Render the tree at `*root_p` into `*out`, freeing each node as it is
/// finished.
///
/// Iterative because an expression nests as deep as the input says. A frame is
/// revisited once per child: the first visit allocates the node's dictionary
/// and descends into `children`, later visits walk the sibling chain through
/// `next`, and the visit that finds neither fills the dictionary in and pops.
unsafe fn convert_ast(arena: *mut Arena, root_p: *mut *mut ExprASTNode, out: *mut Object) {
    unsafe {
        let mut stack: Vec<ConvFrame> = Vec::with_capacity(16);
        stack.push(ConvFrame {
            node_p: root_p,
            ret_node_p: out,
        });
        while let Some(&frame) = stack.last() {
            let node: *mut ExprASTNode = *frame.node_p;
            if node.is_null() {
                // Only the root can be NULL, and only when the parse produced
                // nothing at all.
                debug_assert!(stack.len() == 1, "kv_size(ast_conv_stack) == 1");
                stack.pop();
                continue;
            }
            if (*frame.ret_node_p).type_0 == kObjectTypeNil {
                let ret_node = arena_dict(arena, node_dict_size(&*node));
                *frame.ret_node_p = Object::dict(ret_node);
            }
            let ret_node: *mut Dict = &raw mut (*frame.ret_node_p).data.dict;
            if !(*node).children.is_null() {
                // A node has at most two children, laid out as a `next` chain.
                let num_children = 1 + size_t::from(!(*(*node).children).next.is_null());
                let mut children_array: Array = arena_array(arena, num_children);
                for _ in 0..num_children {
                    array_add(&mut children_array, Object::NIL);
                }
                dict_put(&mut *ret_node, c"children", Object::array(children_array));
                stack.push(ConvFrame {
                    node_p: &raw mut (*node).children,
                    ret_node_p: children_array.items,
                });
            } else if !(*node).next.is_null() {
                stack.push(ConvFrame {
                    node_p: &raw mut (*node).next,
                    ret_node_p: frame.ret_node_p.add(1),
                });
            } else {
                stack.pop();
                finish_node(arena, node, &mut *ret_node);
                debug_assert!(
                    (*frame.ret_node_p).data.dict.size == (*frame.ret_node_p).data.dict.capacity,
                    "cur_item.ret_node_p->data.dict.size == cur_item.ret_node_p->data.dict.capacity"
                );
                xfree((*frame.node_p).cast());
                *frame.node_p = ptr::null_mut::<ExprASTNode>();
            }
        }
    }
}

/// How many pairs [`finish_node`] will put in a node's dictionary. The three
/// every node gets are "type", "start" and "len".
fn node_dict_size(node: &ExprASTNode) -> size_t {
    let type_0 = node.type_0;
    let has_scope = type_0 == kExprNodeOption || type_0 == kExprNodePlainIdentifier;
    let has_ident = has_scope || type_0 == kExprNodePlainKey || type_0 == kExprNodeEnvironment;
    3 + size_t::from(!node.children.is_null())
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
/// three every node has, then whatever its own variant carries.
unsafe fn finish_node(arena: *mut Arena, node: *mut ExprASTNode, ret_node: &mut Dict) {
    unsafe {
        let type_0 = (*node).type_0;
        let type_name = east_node_type_tab.with(|tab| tab[type_0 as usize]);
        dict_put(ret_node, c"type", Object::string(cstr_as_string(type_name)));

        let mut start_array: Array = arena_array(arena, 2);
        array_add(
            &mut start_array,
            Object::integer((*node).start.line as Integer),
        );
        array_add(
            &mut start_array,
            Object::integer((*node).start.col as Integer),
        );
        dict_put(ret_node, c"start", Object::array(start_array));
        dict_put(ret_node, c"len", Object::integer((*node).len as Integer));

        // The string body is owned by the node; hand the copy over and free it
        // here, since the node itself is about to go.
        let string_body = |value: *mut c_char, size: size_t| {
            Object::string(arena_string(arena, String_0::from_raw_parts(value, size)))
        };
        match type_0 {
            kExprNodeDoubleQuotedString | kExprNodeSingleQuotedString => {
                let str = string_body((*node).data.str.value, (*node).data.str.size);
                dict_put(ret_node, c"svalue", str);
                xfree((*node).data.str.value.cast());
            }
            kExprNodeOption => {
                dict_put(
                    ret_node,
                    c"scope",
                    Object::integer((*node).data.opt.scope as Integer),
                );
                let ident = string_body(
                    (*node).data.opt.ident.cast_mut(),
                    (*node).data.opt.ident_len,
                );
                dict_put(ret_node, c"ident", ident);
            }
            kExprNodePlainIdentifier => {
                dict_put(
                    ret_node,
                    c"scope",
                    Object::integer((*node).data.var.scope as Integer),
                );
                let ident = string_body(
                    (*node).data.var.ident.cast_mut(),
                    (*node).data.var.ident_len,
                );
                dict_put(ret_node, c"ident", ident);
            }
            kExprNodePlainKey => {
                let ident = string_body(
                    (*node).data.var.ident.cast_mut(),
                    (*node).data.var.ident_len,
                );
                dict_put(ret_node, c"ident", ident);
            }
            kExprNodeEnvironment => {
                let ident = string_body(
                    (*node).data.env.ident.cast_mut(),
                    (*node).data.env.ident_len,
                );
                dict_put(ret_node, c"ident", ident);
            }
            kExprNodeRegister => {
                dict_put(
                    ret_node,
                    c"name",
                    Object::integer((*node).data.reg.name as Integer),
                );
            }
            kExprNodeComparison => {
                let cmp = eltkn_cmp_type_tab.with(|tab| tab[(*node).data.cmp.type_0 as usize]);
                dict_put(ret_node, c"cmp_type", Object::string(cstr_as_string(cmp)));
                let ccs = ccs_tab.with(|tab| tab[(*node).data.cmp.ccs as usize]);
                dict_put(
                    ret_node,
                    c"ccs_strategy",
                    Object::string(cstr_as_string(ccs)),
                );
                dict_put(ret_node, c"invert", Object::boolean((*node).data.cmp.inv));
            }
            kExprNodeFloat => {
                dict_put(ret_node, c"fvalue", Object::float((*node).data.flt.value));
            }
            kExprNodeInteger => {
                // The lexer's value is unsigned; the wire's is not.
                let value = (*node).data.num.value.min(Integer::MAX as uvarnumber_T);
                dict_put(ret_node, c"ivalue", Object::integer(value as Integer));
            }
            kExprNodeAssignment => {
                let asgn_type = (*node).data.ass.type_0;
                // Plain "=" has no augmentation, and the table's slot for it is
                // the empty string rather than a name.
                let augmentation = if asgn_type == kExprAsgnPlain {
                    String_0::NULL
                } else {
                    cstr_as_string(expr_asgn_type_tab.with(|tab| tab[asgn_type as usize]))
                };
                dict_put(ret_node, c"augmentation", Object::string(augmentation));
            }
            _ => {}
        }
    }
}
