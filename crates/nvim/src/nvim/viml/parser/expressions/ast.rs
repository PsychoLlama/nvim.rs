use super::*;

#[unsafe(no_mangle)]
pub static eltkn_cmp_type_tab: GlobalCell<[*const ::core::ffi::c_char; 5]> = GlobalCell::new([
    b"Equal\0".as_ptr() as *const ::core::ffi::c_char,
    b"Matches\0".as_ptr() as *const ::core::ffi::c_char,
    b"Greater\0".as_ptr() as *const ::core::ffi::c_char,
    b"GreaterOrEqual\0".as_ptr() as *const ::core::ffi::c_char,
    b"Identical\0".as_ptr() as *const ::core::ffi::c_char,
]);
#[unsafe(no_mangle)]
pub static expr_asgn_type_tab: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"Plain\0".as_ptr() as *const ::core::ffi::c_char,
    b"Add\0".as_ptr() as *const ::core::ffi::c_char,
    b"Subtract\0".as_ptr() as *const ::core::ffi::c_char,
    b"Concat\0".as_ptr() as *const ::core::ffi::c_char,
]);
#[unsafe(no_mangle)]
pub static ccs_tab: GlobalCell<[*const ::core::ffi::c_char; 64]> = GlobalCell::new([
    b"UseOption\0".as_ptr() as *const ::core::ffi::c_char,
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
    b"MatchCase\0".as_ptr() as *const ::core::ffi::c_char,
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
    b"IgnoreCase\0".as_ptr() as *const ::core::ffi::c_char,
]);
#[unsafe(no_mangle)]
pub static east_node_type_tab: GlobalCell<[*const ::core::ffi::c_char; 39]> = GlobalCell::new([
    b"Missing\0".as_ptr() as *const ::core::ffi::c_char,
    b"OpMissing\0".as_ptr() as *const ::core::ffi::c_char,
    b"Ternary\0".as_ptr() as *const ::core::ffi::c_char,
    b"TernaryValue\0".as_ptr() as *const ::core::ffi::c_char,
    b"Register\0".as_ptr() as *const ::core::ffi::c_char,
    b"Subscript\0".as_ptr() as *const ::core::ffi::c_char,
    b"ListLiteral\0".as_ptr() as *const ::core::ffi::c_char,
    b"UnaryPlus\0".as_ptr() as *const ::core::ffi::c_char,
    b"BinaryPlus\0".as_ptr() as *const ::core::ffi::c_char,
    b"Nested\0".as_ptr() as *const ::core::ffi::c_char,
    b"Call\0".as_ptr() as *const ::core::ffi::c_char,
    b"PlainIdentifier\0".as_ptr() as *const ::core::ffi::c_char,
    b"PlainKey\0".as_ptr() as *const ::core::ffi::c_char,
    b"ComplexIdentifier\0".as_ptr() as *const ::core::ffi::c_char,
    b"UnknownFigure\0".as_ptr() as *const ::core::ffi::c_char,
    b"Lambda\0".as_ptr() as *const ::core::ffi::c_char,
    b"DictLiteral\0".as_ptr() as *const ::core::ffi::c_char,
    b"CurlyBracesIdentifier\0".as_ptr() as *const ::core::ffi::c_char,
    b"Comma\0".as_ptr() as *const ::core::ffi::c_char,
    b"Colon\0".as_ptr() as *const ::core::ffi::c_char,
    b"Arrow\0".as_ptr() as *const ::core::ffi::c_char,
    b"Comparison\0".as_ptr() as *const ::core::ffi::c_char,
    b"Concat\0".as_ptr() as *const ::core::ffi::c_char,
    b"ConcatOrSubscript\0".as_ptr() as *const ::core::ffi::c_char,
    b"Integer\0".as_ptr() as *const ::core::ffi::c_char,
    b"Float\0".as_ptr() as *const ::core::ffi::c_char,
    b"SingleQuotedString\0".as_ptr() as *const ::core::ffi::c_char,
    b"DoubleQuotedString\0".as_ptr() as *const ::core::ffi::c_char,
    b"Or\0".as_ptr() as *const ::core::ffi::c_char,
    b"And\0".as_ptr() as *const ::core::ffi::c_char,
    b"UnaryMinus\0".as_ptr() as *const ::core::ffi::c_char,
    b"BinaryMinus\0".as_ptr() as *const ::core::ffi::c_char,
    b"Not\0".as_ptr() as *const ::core::ffi::c_char,
    b"Multiplication\0".as_ptr() as *const ::core::ffi::c_char,
    b"Division\0".as_ptr() as *const ::core::ffi::c_char,
    b"Mod\0".as_ptr() as *const ::core::ffi::c_char,
    b"Option\0".as_ptr() as *const ::core::ffi::c_char,
    b"Environment\0".as_ptr() as *const ::core::ffi::c_char,
    b"Assignment\0".as_ptr() as *const ::core::ffi::c_char,
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
        let mut i: size_t = 0 as size_t;
        while i < ast_stack.len().wrapping_sub(1 as size_t) {
            assert!(
                *ast_stack[i] != *cur_node,
                "*kv_A(ast_stack, i) != *cur_node"
            );
            i = i.wrapping_add(1);
        }
        if (*cur_node).is_null() {
            assert!(ast_stack.len() == 1 as size_t, "kv_size(ast_stack) == 1");
            ast_stack.truncate(ast_stack.len() - 1 as size_t);
        } else if !(**cur_node).children.is_null() {
            let maxchildren: uint8_t = node_maxchildren[(**cur_node).type_0 as usize];
            assert!(
                maxchildren as ::core::ffi::c_int > 0 as ::core::ffi::c_int,
                "maxchildren > 0"
            );
            assert!(
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
            ast_stack.truncate(ast_stack.len() - 1 as size_t);
            match (**cur_node).type_0 as ::core::ffi::c_uint {
                27 | 26 => {
                    xfree((**cur_node).data.str.value as *mut ::core::ffi::c_void);
                }
                0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17
                | 38 | 18 | 19 | 20 | 21 | 22 | 23 | 24 | 25 | 28 | 29 | 30 | 31 | 32 | 33 | 34
                | 35 | 36 | 37 | _ => {}
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
    let mut ret: bool = true_0 != 0;
    let mut top_node_p: *mut *mut ExprASTNode = ::core::ptr::null_mut::<*mut ExprASTNode>();
    let mut top_node: *mut ExprASTNode = ::core::ptr::null_mut::<ExprASTNode>();
    let mut top_node_lvl: ExprOpLvl = kEOpLvlInvalid;
    let mut top_node_ass: ExprOpAssociativity = 0 as ExprOpAssociativity;
    assert!(ast_stack.len() != 0, "kv_size(*ast_stack)");
    let bop_node_lvl: ExprOpLvl = (if (*bop_node).type_0 as ::core::ffi::c_uint
        == kExprNodeCall as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*bop_node).type_0 as ::core::ffi::c_uint
            == kExprNodeSubscript as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        kEOpLvlSubscript as ::core::ffi::c_int as ::core::ffi::c_uint
    } else {
        node_lvl(*bop_node) as ::core::ffi::c_uint
    }) as ExprOpLvl;
    loop {
        let mut new_top_node_p: *mut *mut ExprASTNode = stack_top(&ast_stack, 0);
        let mut new_top_node: *mut ExprASTNode = *new_top_node_p;
        assert!(!new_top_node.is_null(), "new_top_node != NULL");
        let new_top_node_lvl: ExprOpLvl = node_lvl(*new_top_node);
        let new_top_node_ass: ExprOpAssociativity = node_ass(*new_top_node);
        if !top_node_p.is_null()
            && (bop_node_lvl as ::core::ffi::c_uint > new_top_node_lvl as ::core::ffi::c_uint
                || bop_node_lvl as ::core::ffi::c_uint == new_top_node_lvl as ::core::ffi::c_uint
                    && new_top_node_ass as ::core::ffi::c_uint
                        == kEOpAssNo as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            break;
        }
        ast_stack.truncate(ast_stack.len() - 1 as size_t);
        top_node_p = new_top_node_p;
        top_node = new_top_node;
        top_node_lvl = new_top_node_lvl;
        top_node_ass = new_top_node_ass;
        if bop_node_lvl as ::core::ffi::c_uint == top_node_lvl as ::core::ffi::c_uint
            && top_node_ass as ::core::ffi::c_uint
                == kEOpAssRight as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            break;
        }
        if ast_stack.len() == 0 {
            break;
        }
    }
    if top_node_ass as ::core::ffi::c_uint
        == kEOpAssLeft as ::core::ffi::c_int as ::core::ffi::c_uint
        || top_node_lvl as ::core::ffi::c_uint != bop_node_lvl as ::core::ffi::c_uint
    {
        *top_node_p = bop_node;
        (*bop_node).children = top_node;
        assert!(
            (*(*bop_node).children).next.is_null(),
            "bop_node->children->next == NULL"
        );
        ast_stack.push(top_node_p);
        ast_stack.push(&raw mut (*(*bop_node).children).next);
    } else {
        assert!(
            top_node_lvl as ::core::ffi::c_uint == bop_node_lvl as ::core::ffi::c_uint
                && top_node_ass as ::core::ffi::c_uint
                    == kEOpAssRight as ::core::ffi::c_int as ::core::ffi::c_uint,
            "top_node_lvl == bop_node_lvl && top_node_ass == kEOpAssRight"
        );
        assert!(
            !(*top_node).children.is_null() && !(*(*top_node).children).next.is_null(),
            "top_node->children != NULL && top_node->children->next != NULL"
        );
        (*bop_node).children = (*(*top_node).children).next;
        (*(*top_node).children).next = bop_node;
        assert!(
            (*(*bop_node).children).next.is_null(),
            "bop_node->children->next == NULL"
        );
        ast_stack.push(top_node_p);
        ast_stack.push(&raw mut (*(*top_node).children).next);
        ast_stack.push(&raw mut (*(*bop_node).children).next);
        if (*bop_node).type_0 as ::core::ffi::c_uint
            == kExprNodeComparison as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            east_set_error(
                pstate,
                ast_err,
                gettext(b"E15: Operator is not associative: %.*s\0".as_ptr()
                    as *const ::core::ffi::c_char),
                (*bop_node).start,
            );
            ret = false_0 != 0;
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
    let pline: ParserLine = *(*pstate).reader.lines.items.offset(start.line as isize);
    (*ret_ast_err).msg = msg;
    (*ret_ast_err).arg_len = pline.size.wrapping_sub(start.col) as ::core::ffi::c_int;
    (*ret_ast_err).arg = if !pline.data.is_null() {
        pline.data.offset(start.col as isize)
    } else {
        ::core::ptr::null::<::core::ffi::c_char>()
    };
}
