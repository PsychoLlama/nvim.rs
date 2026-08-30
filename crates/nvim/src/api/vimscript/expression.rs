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
use crate::api::private::helpers::{Reported, array_add, dict_put};
use crate::api_error;
use crate::kvec::InitVec;
use crate::message_fmt::msg_bytes;
use core::ffi::{CStr, c_char, c_int, c_uint};
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
    let mut error = Error::none();
    // SAFETY: `flags` is the caller's string and `error` this frame's slot.
    let Some(pflags) = (unsafe { parse_flags(flags, &mut error) }) else {
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
    // SAFETY: `ParserState` is a plain-data struct, so all-zero is a valid
    // value for it; `viml_parser_init` fills it in before it is read.
    let mut pstate: ParserState = unsafe { ::core::mem::zeroed() };
    let state = &raw mut pstate;
    let lines = (&raw mut plines_p).cast();
    // SAFETY: `state`, `lines` and `colors_p` all name this frame's locals,
    // which outlive the parse below.
    unsafe { viml_parser_init(state, Some(parser_simple_get_line), lines, colors_p) };
    // SAFETY: as above -- the parser reads the lines it was just given.
    let mut east: ExprAST = unsafe { viml_pexpr_parse(state, pflags) };

    // "len" and "ast", plus "error" and "highlight" when they apply.
    let ret_size = 2 + size_t::from(!east.err.msg.is_null()) + size_t::from(hl);
    let mut ret: Dict = arena_dict(arena, ret_size);
    // A multi-line expression stops at the end of the first line.
    let consumed = if pstate.pos.line == 1 {
        parser_lines[0].size
    } else {
        pstate.pos.col
    };
    // Every container here is sized for exactly the pairs that follow, so
    // the one promise `dict_put`/`array_add` ask for is this function's own
    // invariant -- stated here once rather than at every call site.
    // SAFETY: as above.
    unsafe { dict_put(&mut ret, c"len", Object::integer(consumed as Integer)) };

    if !east.err.msg.is_null() {
        let mut err_dict: Dict = arena_dict(arena, 2);
        let arg = String_0::from_raw_parts(east.err.arg.cast_mut(), east.err.arg_len as size_t);
        // SAFETY: the parser's message is NUL-terminated and `arg` names
        // `arg_len` bytes of the expression; both dictionaries are sized.
        unsafe {
            let msg = arena_string(arena, cstr_as_string(east.err.msg));
            dict_put(&mut err_dict, c"message", Object::string(msg));
            dict_put(
                &mut err_dict,
                c"arg",
                Object::string(arena_string(arena, arg)),
            );
            dict_put(&mut ret, c"error", Object::dict(err_dict));
        }
    }

    if hl {
        let mut hl_arr: Array = arena_array(arena, colors.size);
        for i in 0..colors.size {
            // SAFETY: `i` is below `size`, so the chunk is inside `items`.
            let chunk: ParserHighlightChunk = unsafe { *colors.items.add(i) };
            let mut chunk_arr: Array = arena_array(arena, 4);
            // SAFETY: as above -- both arrays are sized for these pushes,
            // and `group` is a static highlight-group name.
            unsafe {
                array_add(&mut chunk_arr, Object::integer(chunk.start.line as Integer));
                array_add(&mut chunk_arr, Object::integer(chunk.start.col as Integer));
                array_add(&mut chunk_arr, Object::integer(chunk.end_col as Integer));
                array_add(&mut chunk_arr, Object::string(cstr_as_string(chunk.group)));
                array_add(&mut hl_arr, Object::array(chunk_arr));
            }
        }
        // SAFETY: as above.
        unsafe { dict_put(&mut ret, c"highlight", Object::array(hl_arr)) };
    }
    // The vector `colors` describes is either its inline array or one heap
    // block; only the second has anything to free.
    let heap = InitVec::new(
        &mut colors.size,
        &mut colors.capacity,
        &mut colors.items,
        &mut colors.init_array,
    )
    .take_heap();
    // SAFETY: `heap` is null or that block.
    unsafe { xfree(heap) };

    let mut ast = Object::NIL;
    // SAFETY: `east.root` and `ast` are this frame's, and `arena` the
    // caller's.
    unsafe { convert_ast(arena, &raw mut east.root, &raw mut ast) };
    // SAFETY: as above.
    unsafe { dict_put(&mut ret, c"ast", ast) };
    debug_assert!(ret.size == ret.capacity, "ret.size == ret.capacity");

    // SAFETY: the walk freed every node it rendered and NULLed its slot, so
    // this frees only what is left; `pstate` is this frame's.
    unsafe { viml_pexpr_free_ast(east) };
    viml_parser_destroy(&mut pstate);
    ret.reported(error)
}

/// The `flags` argument as `ExprParserFlags`, or `None` after reporting which
/// character was not one.
///
/// # Safety
/// `flags` must name its own bytes.
unsafe fn parse_flags(flags: String_0, err: &mut Error) -> Option<c_int> {
    let mut pflags: c_int = 0;
    for i in 0..flags.len() {
        // SAFETY: `i` is below `len`, so the byte is inside the string.
        let ch: c_char = unsafe { *flags.data().add(i) };
        match ch as u8 {
            b'm' => pflags |= kExprFlagsMulti as c_int,
            b'E' => pflags |= kExprFlagsDisallowEOC as c_int,
            b'l' => pflags |= kExprFlagsParseLet as c_int,
            // A NUL has no `%c` spelling worth printing.
            0 => {
                let code = ch as c_uint;
                *err = api_error!(kErrorTypeValidation, "Invalid flag: '\\0' ({code})");
                return None;
            }
            _ => {
                let code = ch as c_uint;
                let raw = ch as u8;
                let shown = msg_bytes(core::slice::from_ref(&raw));
                *err = api_error!(kErrorTypeValidation, "Invalid flag: '{shown}' ({code})");
                return None;
            }
        }
    }
    Some(pflags)
}

/// Render the tree at `*root_p` into `*out`, freeing each node as it is
/// finished.
///
/// Iterative because an expression nests as deep as the input says. A frame is
/// revisited once per child: the first visit allocates the node's dictionary
/// and descends into `children`, later visits walk the sibling chain through
/// `next`, and the visit that finds neither fills the dictionary in and pops.
///
/// # Safety
/// `root_p` and `out` must name the caller's slots, and the tree below
/// `*root_p` must be the parser's own.
unsafe fn convert_ast(arena: *mut Arena, root_p: *mut *mut ExprASTNode, out: *mut Object) {
    let mut stack: Vec<ConvFrame> = Vec::with_capacity(16);
    stack.push(ConvFrame {
        node_p: root_p,
        ret_node_p: out,
    });
    while let Some(&frame) = stack.last() {
        // SAFETY: every frame names a slot of the caller's or of a node the
        // walk has not freed yet.
        let node: *mut ExprASTNode = unsafe { *frame.node_p };
        if node.is_null() {
            // Only the root can be NULL, and only when the parse produced
            // nothing at all.
            debug_assert!(stack.len() == 1, "kv_size(ast_conv_stack) == 1");
            stack.pop();
            continue;
        }
        // SAFETY: as above.
        if unsafe { (*frame.ret_node_p).type_0 } == kObjectTypeNil {
            // SAFETY: `node` is a live node of the parser's tree.
            let ret_node = arena_dict(arena, unsafe { node_dict_size(&*node) });
            // SAFETY: as above.
            unsafe { *frame.ret_node_p = Object::dict(ret_node) };
        }
        // SAFETY: the slot now holds a dictionary, so its union arm is the
        // one this addresses.
        let ret_node: *mut Dict = unsafe { &raw mut (*frame.ret_node_p).data.dict };
        // SAFETY: `node` is live.
        let children = unsafe { (*node).children };
        if !children.is_null() {
            // A node has at most two children, laid out as a `next` chain.
            // SAFETY: `children` is the first of them.
            let num_children = 1 + size_t::from(!unsafe { (*children).next }.is_null());
            let mut children_array: Array = arena_array(arena, num_children);
            // SAFETY: the array was sized for exactly these pushes, and the
            // dictionary for this pair.
            unsafe {
                for _ in 0..num_children {
                    array_add(&mut children_array, Object::NIL);
                }
                dict_put(&mut *ret_node, c"children", Object::array(children_array));
            }
            stack.push(ConvFrame {
                // SAFETY: `node` is live for as long as the frame is.
                node_p: unsafe { &raw mut (*node).children },
                ret_node_p: children_array.items,
            });
        // SAFETY: `node` is live.
        } else if !unsafe { (*node).next }.is_null() {
            stack.push(ConvFrame {
                // SAFETY: as above.
                node_p: unsafe { &raw mut (*node).next },
                // SAFETY: the parent sized its array for both siblings.
                ret_node_p: unsafe { frame.ret_node_p.add(1) },
            });
        } else {
            stack.pop();
            // SAFETY: `node` is live and `ret_node` is its dictionary,
            // sized by `node_dict_size` for exactly what this adds.
            unsafe { finish_node(arena, node, &mut *ret_node) };
            // SAFETY: the slot holds the dictionary just filled in.
            let filled = unsafe { (*frame.ret_node_p).data.dict };
            debug_assert!(
                filled.size == filled.capacity,
                "cur_item.ret_node_p->data.dict.size == cur_item.ret_node_p->data.dict.capacity"
            );
            // SAFETY: the node has been rendered, so nothing names it any
            // more, and the slot it hung off is the caller's.
            unsafe {
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
///
/// # Safety
/// `node` must be a live node of the parser's tree, and `ret_node` its
/// dictionary, sized by [`node_dict_size`].
unsafe fn finish_node(arena: *mut Arena, node: *mut ExprASTNode, ret_node: &mut Dict) {
    // SAFETY: the caller's promise -- `node` is live, and nothing below
    // writes through it.
    let node = unsafe { &*node };
    let type_0 = node.type_0;

    // `ret_node` was sized by `node_dict_size` for exactly the pairs added
    // here, which is the one promise `dict_put` asks for -- stated once
    // rather than at each of the fifteen call sites below.
    let put = |dict: &mut Dict, key: &'static CStr, value: Object| {
        // SAFETY: as above.
        unsafe { dict_put(dict, key, value) };
    };
    // The three name tables hold static C strings.
    let table_name = |name: *const c_char| {
        // SAFETY: as above.
        Object::string(unsafe { cstr_as_string(name) })
    };
    // The string body is owned by the node; hand the copy over and free it
    // here, since the node itself is about to go.
    let string_body = |value: *mut c_char, size: size_t| {
        let borrowed = String_0::from_raw_parts(value, size);
        // SAFETY: the node owns `size` readable bytes at `value`.
        Object::string(unsafe { arena_string(arena, borrowed) })
    };

    let type_name = east_node_type_tab.with(|tab| tab[type_0 as usize]);
    put(ret_node, c"type", table_name(type_name));

    let mut start_array: Array = arena_array(arena, 2);
    // SAFETY: the array was sized for exactly these two pushes.
    unsafe {
        array_add(
            &mut start_array,
            Object::integer(node.start.line as Integer),
        );
        array_add(&mut start_array, Object::integer(node.start.col as Integer));
    }
    put(ret_node, c"start", Object::array(start_array));
    put(ret_node, c"len", Object::integer(node.len as Integer));

    // The payload, once, so each arm below reads as the field list its
    // node type carries rather than as a chain through the pointer.
    let data = node.data;
    match type_0 {
        kExprNodeDoubleQuotedString | kExprNodeSingleQuotedString => {
            let str = string_body(data.string().value, data.string().size);
            put(ret_node, c"svalue", str);
            // SAFETY: the body was just copied into the arena.
            unsafe { xfree(data.string().value.cast()) };
        }
        kExprNodeOption => {
            put(
                ret_node,
                c"scope",
                Object::integer(data.option().scope as Integer),
            );
            let ident = string_body(data.option().ident.cast_mut(), data.option().ident_len);
            put(ret_node, c"ident", ident);
        }
        kExprNodePlainIdentifier => {
            put(
                ret_node,
                c"scope",
                Object::integer(data.variable().scope as Integer),
            );
            let ident = string_body(data.variable().ident.cast_mut(), data.variable().ident_len);
            put(ret_node, c"ident", ident);
        }
        kExprNodePlainKey => {
            let ident = string_body(data.variable().ident.cast_mut(), data.variable().ident_len);
            put(ret_node, c"ident", ident);
        }
        kExprNodeEnvironment => {
            let ident = string_body(
                data.environment().ident.cast_mut(),
                data.environment().ident_len,
            );
            put(ret_node, c"ident", ident);
        }
        kExprNodeRegister => {
            put(
                ret_node,
                c"name",
                Object::integer(data.register().name as Integer),
            );
        }
        kExprNodeComparison => {
            let cmp = eltkn_cmp_type_tab.with(|tab| tab[data.comparison().type_0 as usize]);
            put(ret_node, c"cmp_type", table_name(cmp));
            let ccs = ccs_tab.with(|tab| tab[data.comparison().ccs as usize]);
            put(ret_node, c"ccs_strategy", table_name(ccs));
            put(ret_node, c"invert", Object::boolean(data.comparison().inv));
        }
        kExprNodeFloat => {
            put(ret_node, c"fvalue", Object::float(data.float().value));
        }
        kExprNodeInteger => {
            // The lexer's value is unsigned; the wire's is not.
            let value = data.integer().value.min(Integer::MAX as uvarnumber_T);
            put(ret_node, c"ivalue", Object::integer(value as Integer));
        }
        kExprNodeAssignment => {
            let asgn_type = data.assignment().type_0;
            // Plain "=" has no augmentation, and the table's slot for it is
            // the empty string rather than a name.
            let augmentation = if asgn_type == kExprAsgnPlain {
                Object::string(String_0::NULL)
            } else {
                table_name(expr_asgn_type_tab.with(|tab| tab[asgn_type as usize]))
            };
            put(ret_node, c"augmentation", augmentation);
        }
        _ => {}
    }
}
