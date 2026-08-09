use super::*;

#[unsafe(no_mangle)]
pub static eltkn_cmp_type_tab: GlobalCell<[*const ::core::ffi::c_char; 5]> = GlobalCell::new([
    c"Equal".as_ptr(),
    c"Matches".as_ptr(),
    c"Greater".as_ptr(),
    c"GreaterOrEqual".as_ptr(),
    c"Identical".as_ptr(),
]);
#[unsafe(no_mangle)]
pub static expr_asgn_type_tab: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    c"Plain".as_ptr(),
    c"Add".as_ptr(),
    c"Subtract".as_ptr(),
    c"Concat".as_ptr(),
]);
#[unsafe(no_mangle)]
pub static ccs_tab: GlobalCell<[*const ::core::ffi::c_char; 64]> = GlobalCell::new([
    c"UseOption".as_ptr(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    c"MatchCase".as_ptr(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    c"IgnoreCase".as_ptr(),
]);
#[unsafe(no_mangle)]
pub static east_node_type_tab: GlobalCell<[*const ::core::ffi::c_char; 39]> = GlobalCell::new([
    c"Missing".as_ptr(),
    c"OpMissing".as_ptr(),
    c"Ternary".as_ptr(),
    c"TernaryValue".as_ptr(),
    c"Register".as_ptr(),
    c"Subscript".as_ptr(),
    c"ListLiteral".as_ptr(),
    c"UnaryPlus".as_ptr(),
    c"BinaryPlus".as_ptr(),
    c"Nested".as_ptr(),
    c"Call".as_ptr(),
    c"PlainIdentifier".as_ptr(),
    c"PlainKey".as_ptr(),
    c"ComplexIdentifier".as_ptr(),
    c"UnknownFigure".as_ptr(),
    c"Lambda".as_ptr(),
    c"DictLiteral".as_ptr(),
    c"CurlyBracesIdentifier".as_ptr(),
    c"Comma".as_ptr(),
    c"Colon".as_ptr(),
    c"Arrow".as_ptr(),
    c"Comparison".as_ptr(),
    c"Concat".as_ptr(),
    c"ConcatOrSubscript".as_ptr(),
    c"Integer".as_ptr(),
    c"Float".as_ptr(),
    c"SingleQuotedString".as_ptr(),
    c"DoubleQuotedString".as_ptr(),
    c"Or".as_ptr(),
    c"And".as_ptr(),
    c"UnaryMinus".as_ptr(),
    c"BinaryMinus".as_ptr(),
    c"Not".as_ptr(),
    c"Multiplication".as_ptr(),
    c"Division".as_ptr(),
    c"Mod".as_ptr(),
    c"Option".as_ptr(),
    c"Environment".as_ptr(),
    c"Assignment".as_ptr(),
]);
pub(super) static node_maxchildren: [uint8_t; 39] = [
    0 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    0 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    2 as uint8_t,
];
#[unsafe(no_mangle)]
pub unsafe extern "C" fn viml_pexpr_free_ast(mut ast: ExprAST) {
    let mut ast_stack: Vec<*mut *mut ExprASTNode> = Vec::new();
    ast_stack.push(&raw mut ast.root);
    while ast_stack.len() != 0 {
        let cur_node: *mut *mut ExprASTNode = stack_top(&ast_stack, 0);
        let mut i: size_t = 0;
        while i < ast_stack.len().wrapping_sub(1) {
            debug_assert!(
                *ast_stack[i] != *cur_node,
                "*kv_A(ast_stack, i) != *cur_node"
            );
            i = i.wrapping_add(1);
        }
        if (*cur_node).is_null() {
            debug_assert!(ast_stack.len() == 1, "kv_size(ast_stack) == 1");
            ast_stack.truncate(ast_stack.len() - 1);
        } else if !(**cur_node).children.is_null() {
            let maxchildren: uint8_t = node_maxchildren[(**cur_node).type_0 as usize];
            debug_assert!(
                maxchildren as ::core::ffi::c_int > 0 as ::core::ffi::c_int,
                "maxchildren > 0"
            );
            debug_assert!(
                maxchildren as ::core::ffi::c_int <= 2 as ::core::ffi::c_int,
                "maxchildren <= 2"
            );
            assert!(
                (if maxchildren as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
                    (*(**cur_node).children).next.is_null() as ::core::ffi::c_int
                } else {
                    ((*(**cur_node).children).next.is_null()
                        || (*(*(**cur_node).children).next).next.is_null())
                        as ::core::ffi::c_int
                }) != 0,
                "maxchildren == 1 ? (*cur_node)->children->next == NULL : ((*cur_node)->children->next == NULL || (*cur_node)->children->next->next == NULL)"
            );
            ast_stack.push(&raw mut (**cur_node).children);
        } else if !(**cur_node).next.is_null() {
            ast_stack.push(&raw mut (**cur_node).next);
        } else if !(*cur_node).is_null() {
            ast_stack.truncate(ast_stack.len() - 1);
            match (**cur_node).type_0 {
                kExprNodeDoubleQuotedString | kExprNodeSingleQuotedString => {
                    xfree((**cur_node).data.str.value as *mut ::core::ffi::c_void);
                }
                _ => {}
            }
            xfree(*cur_node as *mut ::core::ffi::c_void);
            *cur_node = ::core::ptr::null_mut::<ExprASTNode>();
        }
    }
}
#[inline]
pub(super) unsafe extern "C" fn viml_pexpr_new_node(type_0: ExprASTNodeType) -> *mut ExprASTNode {
    let mut ret: *mut ExprASTNode =
        xmalloc(::core::mem::size_of::<ExprASTNode>()) as *mut ExprASTNode;
    (*ret).type_0 = type_0;
    (*ret).children = ::core::ptr::null_mut::<ExprASTNode>();
    (*ret).next = ::core::ptr::null_mut::<ExprASTNode>();
    return ret;
}
pub(super) static node_type_to_node_props: [C2Rust_Unnamed_34; 39] = [
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlInvalid,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlMultiplication,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlTernary,
        ass: kEOpAssRight,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlTernaryValue,
        ass: kEOpAssRight,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlUnary,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlAddition,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlComplexIdentifier,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlComma,
        ass: kEOpAssRight,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlColon,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlArrow,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlComparison,
        ass: kEOpAssRight,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlAddition,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlSubscript,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlOr,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlAnd,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlUnary,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlAddition,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlUnary,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlMultiplication,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlMultiplication,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlMultiplication,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlAssignment,
        ass: kEOpAssLeft,
    },
];
#[inline(always)]
pub(super) fn node_lvl(node: ExprASTNode) -> ExprOpLvl {
    return node_type_to_node_props[node.type_0 as usize].lvl;
}
#[inline(always)]
pub(super) fn node_ass(node: ExprASTNode) -> ExprOpAssociativity {
    return node_type_to_node_props[node.type_0 as usize].ass;
}
pub(super) unsafe extern "C" fn viml_pexpr_handle_bop(
    pstate: *const ParserState,
    ast_stack: &mut Vec<*mut *mut ExprASTNode>,
    bop_node: *mut ExprASTNode,
    want_node_p: *mut ExprASTWantedNode,
    ast_err: *mut ExprASTError,
) -> bool {
    let mut ret: bool = true;
    let mut top_node_p: *mut *mut ExprASTNode = ::core::ptr::null_mut::<*mut ExprASTNode>();
    let mut top_node: *mut ExprASTNode = ::core::ptr::null_mut::<ExprASTNode>();
    let mut top_node_lvl: ExprOpLvl = kEOpLvlInvalid;
    let mut top_node_ass: ExprOpAssociativity = 0 as ExprOpAssociativity;
    debug_assert!(ast_stack.len() != 0, "kv_size(*ast_stack)");
    let bop_node_lvl: ExprOpLvl =
        (if (*bop_node).type_0 == kExprNodeCall || (*bop_node).type_0 == kExprNodeSubscript {
            kEOpLvlSubscript
        } else {
            node_lvl(*bop_node) as ::core::ffi::c_uint
        }) as ExprOpLvl;
    loop {
        let mut new_top_node_p: *mut *mut ExprASTNode = stack_top(&ast_stack, 0);
        let mut new_top_node: *mut ExprASTNode = *new_top_node_p;
        debug_assert!(!new_top_node.is_null(), "new_top_node != NULL");
        let new_top_node_lvl: ExprOpLvl = node_lvl(*new_top_node);
        let new_top_node_ass: ExprOpAssociativity = node_ass(*new_top_node);
        if !top_node_p.is_null()
            && (bop_node_lvl as ::core::ffi::c_uint > new_top_node_lvl as ::core::ffi::c_uint
                || bop_node_lvl as ::core::ffi::c_uint == new_top_node_lvl as ::core::ffi::c_uint
                    && new_top_node_ass == kEOpAssNo)
        {
            break;
        }
        ast_stack.truncate(ast_stack.len() - 1);
        top_node_p = new_top_node_p;
        top_node = new_top_node;
        top_node_lvl = new_top_node_lvl;
        top_node_ass = new_top_node_ass;
        if bop_node_lvl as ::core::ffi::c_uint == top_node_lvl as ::core::ffi::c_uint
            && top_node_ass == kEOpAssRight
        {
            break;
        }
        if ast_stack.len() == 0 {
            break;
        }
    }
    if top_node_ass == kEOpAssLeft
        || top_node_lvl as ::core::ffi::c_uint != bop_node_lvl as ::core::ffi::c_uint
    {
        *top_node_p = bop_node;
        (*bop_node).children = top_node;
        debug_assert!(
            (*(*bop_node).children).next.is_null(),
            "bop_node->children->next == NULL"
        );
        ast_stack.push(top_node_p);
        ast_stack.push(&raw mut (*(*bop_node).children).next);
    } else {
        assert!(
            top_node_lvl as ::core::ffi::c_uint == bop_node_lvl as ::core::ffi::c_uint
                && top_node_ass == kEOpAssRight,
            "top_node_lvl == bop_node_lvl && top_node_ass == kEOpAssRight"
        );
        debug_assert!(
            !(*top_node).children.is_null() && !(*(*top_node).children).next.is_null(),
            "top_node->children != NULL && top_node->children->next != NULL"
        );
        (*bop_node).children = (*(*top_node).children).next;
        (*(*top_node).children).next = bop_node;
        debug_assert!(
            (*(*bop_node).children).next.is_null(),
            "bop_node->children->next == NULL"
        );
        ast_stack.push(top_node_p);
        ast_stack.push(&raw mut (*(*top_node).children).next);
        ast_stack.push(&raw mut (*(*bop_node).children).next);
        if (*bop_node).type_0 == kExprNodeComparison {
            east_set_error(
                pstate,
                ast_err,
                gettext(c"E15: Operator is not associative: %.*s".as_ptr()),
                (*bop_node).start,
            );
            ret = false;
        }
    }
    *want_node_p = kENodeValue;
    return ret;
}
#[inline(always)]
pub(super) unsafe extern "C" fn east_set_error(
    pstate: *const ParserState,
    ret_ast_err: *mut ExprASTError,
    msg: *const ::core::ffi::c_char,
    start: ParserPosition,
) {
    if !(*ret_ast_err).msg.is_null() {
        return;
    }
    let pline: ParserLine = *(*pstate).reader.lines.items.add(start.line);
    (*ret_ast_err).msg = msg;
    (*ret_ast_err).arg_len = pline.size.wrapping_sub(start.col) as ::core::ffi::c_int;
    (*ret_ast_err).arg = if !pline.data.is_null() {
        pline.data.add(start.col)
    } else {
        ::core::ptr::null::<::core::ffi::c_char>()
    };
}
