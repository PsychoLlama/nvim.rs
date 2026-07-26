use super::*;

#[inline(always)]
pub(super) fn pt_is_assignment(pt: ExprASTParseType) -> bool {
    return pt as ::core::ffi::c_uint
        == kEPTAssignment as ::core::ffi::c_int as ::core::ffi::c_uint
        || pt as ::core::ffi::c_uint
            == kEPTSingleAssignment as ::core::ffi::c_int as ::core::ffi::c_uint;
}

static want_node_to_lexer_flags: [::core::ffi::c_int; 2] = [
    kELFlagForbidScope as ::core::ffi::c_int,
    kELFlagIsNotCmp as ::core::ffi::c_int,
];
static base_to_prefix_length: [uint8_t; 17] = [
    0,
    0,
    2 as uint8_t,
    0,
    0,
    0,
    0,
    0,
    1 as uint8_t,
    0,
    0 as uint8_t,
    0,
    0,
    0,
    0,
    0,
    2 as uint8_t,
];
#[unsafe(no_mangle)]
pub unsafe extern "C" fn viml_pexpr_parse(
    pstate: *mut ParserState,
    flags: ::core::ffi::c_int,
) -> ExprAST {
    let mut can_be_ternary: bool = false;
    let mut is_subscript: bool = false;
    let mut i_0: size_t = 0;
    let mut eastnode_p: *const *mut ExprASTNode = ::core::ptr::null::<*mut ExprASTNode>();
    let mut eastnode_type: ExprASTNodeType = kExprNodeMissing;
    let mut eastnode_lvl: ExprOpLvl = kEOpLvlInvalid;
    let mut pline: ParserLine = ParserLine {
        data: ::core::ptr::null::<::core::ffi::c_char>(),
        size: 0,
        allocated: false,
    };
    let mut top_node_p: *mut *mut ExprASTNode = ::core::ptr::null_mut::<*mut ExprASTNode>();
    let mut cur_node: *mut ExprASTNode = ::core::ptr::null_mut::<ExprASTNode>();
    let mut want_value: bool = false;
    let mut node_is_key: bool = false;
    let mut is_single_assignment: bool = false;
    let mut cur_pt: ExprASTParseType = kEPTExpr;
    let mut ast: ExprAST = ExprAST {
        err: ExprASTError {
            msg: ::core::ptr::null::<::core::ffi::c_char>(),
            arg: ::core::ptr::null::<::core::ffi::c_char>(),
            arg_len: 0 as ::core::ffi::c_int,
        },
        root: ::core::ptr::null_mut::<ExprASTNode>(),
    };
    let mut ast_stack: Vec<*mut *mut ExprASTNode> = Vec::new();
    ast_stack.push(&raw mut ast.root);
    let mut want_node: ExprASTWantedNode = kENodeValue;
    let mut pt_stack: Vec<ExprASTParseType> = Vec::new();
    pt_stack.push(kEPTExpr);
    if flags & kExprFlagsParseLet as ::core::ffi::c_int != 0 {
        pt_stack.push(kEPTAssignment);
    }
    let mut prev_token: LexExprToken = LexExprToken {
        start: ParserPosition { line: 0, col: 0 },
        len: 0,
        type_0: kExprLexMissing,
        data: C2Rust_Unnamed_7 {
            cmp: C2Rust_Unnamed_19 {
                type_0: kExprCmpEqual,
                ccs: kCCStrategyUseOption,
                inv: false,
            },
        },
    };
    let mut highlighted_prev_spacing: bool = false_0 != 0;
    let mut lambda_node: *mut ExprASTNode = ::core::ptr::null_mut::<ExprASTNode>();
    let mut asgn_level: size_t = 0 as size_t;
    '_viml_pexpr_parse_end: loop {
        let is_concat_or_subscript: bool = want_node as ::core::ffi::c_uint
            == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
            && ast_stack.len() > 1 as size_t
            && (**stack_top(&ast_stack, 1)).type_0 as ::core::ffi::c_uint
                == kExprNodeConcatOrSubscript as ::core::ffi::c_int as ::core::ffi::c_uint;
        let lexer_additional_flags: ::core::ffi::c_int = kELFlagPeek as ::core::ffi::c_int
            | (if flags & kExprFlagsDisallowEOC as ::core::ffi::c_int != 0 {
                kELFlagForbidEOC as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })
            | (if want_node as ::core::ffi::c_uint
                == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                && (ast_stack.len() == 1 as size_t
                    || (**stack_top(&ast_stack, 1)).type_0 as ::core::ffi::c_uint
                        != kExprNodeConcat as ::core::ffi::c_int as ::core::ffi::c_uint
                        && (**stack_top(&ast_stack, 1)).type_0 as ::core::ffi::c_uint
                            != kExprNodeConcatOrSubscript as ::core::ffi::c_int
                                as ::core::ffi::c_uint)
            {
                kELFlagAllowFloat as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            });
        let mut cur_token: LexExprToken = viml_pexpr_next_token(
            pstate,
            want_node_to_lexer_flags[want_node as usize] | lexer_additional_flags,
        );
        if cur_token.type_0 as ::core::ffi::c_uint
            == kExprLexEOC as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            break;
        }
        let mut tok_type: LexExprTokenType = cur_token.type_0;
        let token_invalid: bool = tok_type as ::core::ffi::c_uint
            == kExprLexInvalid as ::core::ffi::c_int as ::core::ffi::c_uint;
        let mut is_invalid: bool = token_invalid;
        '_viml_pexpr_parse_cycle_end: {
            's_6212: {
                's_4376: {
                    loop {
                        cur_token = viml_pexpr_next_token(
                            pstate,
                            want_node_to_lexer_flags[want_node as usize] | lexer_additional_flags,
                        );
                        if tok_type as ::core::ffi::c_uint
                            == kExprLexSpacing as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            if is_invalid {
                                viml_parser_highlight(
                                    pstate,
                                    cur_token.start,
                                    cur_token.len,
                                    if is_invalid as ::core::ffi::c_int != 0 {
                                        b"NvimInvalidSpacing\0".as_ptr()
                                            as *const ::core::ffi::c_char
                                    } else {
                                        b"NvimSpacing\0".as_ptr() as *const ::core::ffi::c_char
                                    },
                                );
                            }
                            break '_viml_pexpr_parse_cycle_end;
                        } else {
                            if is_invalid as ::core::ffi::c_int != 0
                                && prev_token.type_0 as ::core::ffi::c_uint
                                    == kExprLexSpacing as ::core::ffi::c_int as ::core::ffi::c_uint
                                && !highlighted_prev_spacing
                            {
                                viml_parser_highlight(
                                    pstate,
                                    prev_token.start,
                                    prev_token.len,
                                    if is_invalid as ::core::ffi::c_int != 0 {
                                        b"NvimInvalidSpacing\0".as_ptr()
                                            as *const ::core::ffi::c_char
                                    } else {
                                        b"NvimSpacing\0".as_ptr() as *const ::core::ffi::c_char
                                    },
                                );
                                is_invalid = false_0 != 0;
                                highlighted_prev_spacing = true_0 != 0;
                            }
                            pline = *(*pstate)
                                .reader
                                .lines
                                .items
                                .offset(cur_token.start.line as isize);
                            top_node_p = stack_top(&ast_stack, 0);
                            assert!(ast_stack.len() >= 1 as size_t, "kv_size(ast_stack) >= 1");
                            cur_node = ::core::ptr::null_mut::<ExprASTNode>();
                            want_value = want_node as ::core::ffi::c_uint
                                == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint;
                            assert!(
                                want_value as ::core::ffi::c_int
                                    == (*top_node_p).is_null() as ::core::ffi::c_int,
                                "want_value == (*top_node_p == NULL)"
                            );
                            assert!(
                                ast_stack[0] == &raw mut ast.root,
                                "kv_A(ast_stack, 0) == &ast.root"
                            );
                            let mut i: size_t = 0 as size_t;
                            while i.wrapping_add(1 as size_t) < ast_stack.len() {
                                let item_null: bool = want_value as ::core::ffi::c_int != 0
                                    && i.wrapping_add(2 as size_t) == ast_stack.len();
                                assert!(
                                    &raw mut (**ast_stack[i]).children
                                        == ast_stack[i.wrapping_add(1)]
                                        && (if item_null as ::core::ffi::c_int != 0 {
                                            (**ast_stack[i]).children.is_null()
                                                as ::core::ffi::c_int
                                        } else {
                                            (*(**ast_stack[i]).children).next.is_null()
                                                as ::core::ffi::c_int
                                        }) != 0
                                        || &raw mut (*(**ast_stack[i]).children).next
                                            == ast_stack[i.wrapping_add(1)]
                                            && (if item_null as ::core::ffi::c_int != 0 {
                                                (*(**ast_stack[i]).children).next.is_null()
                                                    as ::core::ffi::c_int
                                            } else {
                                                (*(*(**ast_stack[i]).children).next).next.is_null()
                                                    as ::core::ffi::c_int
                                            }) != 0,
                                    "(&(*kv_A(ast_stack, i))->children == kv_A(ast_stack, i + 1) && (item_null ? (*kv_A(ast_stack, i))->children == NULL : (*kv_A(ast_stack, i))->children->next == NULL)) || ((&(*kv_A(ast_stack, i))->children->next == kv_A(ast_stack, i + 1)) && (item_null ? (*kv_A(ast_stack, i))->children->next == NULL : (*kv_A(ast_stack, i))->children->next->next == NULL))"
                                );
                                i = i.wrapping_add(1);
                            }
                            node_is_key = is_concat_or_subscript as ::core::ffi::c_int != 0
                                && (if cur_token.type_0 as ::core::ffi::c_uint
                                    == kExprLexPlainIdentifier as ::core::ffi::c_int
                                        as ::core::ffi::c_uint
                                {
                                    (!cur_token.data.var.autoload
                                        && cur_token.data.var.scope as ::core::ffi::c_uint
                                            == kExprVarScopeMissing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint)
                                        as ::core::ffi::c_int
                                } else {
                                    (cur_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexNumber as ::core::ffi::c_int
                                            as ::core::ffi::c_uint)
                                        as ::core::ffi::c_int
                                }) != 0
                                && prev_token.type_0 as ::core::ffi::c_uint
                                    != kExprLexSpacing as ::core::ffi::c_int as ::core::ffi::c_uint;
                            if is_concat_or_subscript as ::core::ffi::c_int != 0 && !node_is_key {
                                (**stack_top(&ast_stack, 1)).type_0 = kExprNodeConcat;
                            }
                            is_single_assignment = pt_stack[pt_stack.len() - 1]
                                as ::core::ffi::c_uint
                                == kEPTSingleAssignment as ::core::ffi::c_int
                                    as ::core::ffi::c_uint;
                            match pt_stack[pt_stack.len() - 1] as ::core::ffi::c_uint {
                                1 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeOperator as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexComma as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexArrow as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || want_node as ::core::ffi::c_uint
                                            == kENodeValue as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            && !(cur_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexPlainIdentifier as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                && cur_token.data.var.scope as ::core::ffi::c_uint
                                                    == kExprVarScopeMissing as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                && !cur_token.data.var.autoload)
                                            && tok_type as ::core::ffi::c_uint
                                                != kExprLexArrow as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                    {
                                        (*lambda_node).data.fig.type_guesses.allow_lambda =
                                            false_0 != 0;
                                        if !(*lambda_node).children.is_null()
                                            && (*(*lambda_node).children).type_0
                                                as ::core::ffi::c_uint
                                                == kExprNodeComma as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                        {
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E15: Expected lambda arguments list or arrow: %.*s\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                        } else {
                                            lambda_node = ::core::ptr::null_mut::<ExprASTNode>();
                                            pt_stack.truncate(pt_stack.len() - 1 as size_t);
                                        }
                                    }
                                }
                                3 | 2 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexBracket as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexPlainIdentifier as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        && (tok_type as ::core::ffi::c_uint
                                            != kExprLexFigureBrace as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || cur_token.data.brc.closing as ::core::ffi::c_int
                                                != 0)
                                        && !(node_is_key as ::core::ffi::c_int != 0
                                            && tok_type as ::core::ffi::c_uint
                                                == kExprLexNumber as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint)
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexEnv as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexOption as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexRegister as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Expected value part of assignment lvalue: %.*s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        pt_stack.truncate(pt_stack.len() - 1 as size_t);
                                    } else if want_node as ::core::ffi::c_uint
                                        == kENodeOperator as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexBracket as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        && (tok_type as ::core::ffi::c_uint
                                            != kExprLexFigureBrace as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || cur_token.data.brc.closing as ::core::ffi::c_int
                                                != 0)
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexDot as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        && (tok_type as ::core::ffi::c_uint
                                            != kExprLexComma as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || !is_single_assignment)
                                        && tok_type as ::core::ffi::c_uint
                                            != kExprLexAssignment as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        && !((tok_type as ::core::ffi::c_uint
                                            == kExprLexPlainIdentifier as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || tok_type as ::core::ffi::c_uint
                                                == kExprLexFigureBrace as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                && !cur_token.data.brc.closing)
                                            && prev_token.type_0 as ::core::ffi::c_uint
                                                != kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint)
                                    {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.len() == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Expected assignment operator or subscript: %.*s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        pt_stack.truncate(pt_stack.len() - 1 as size_t);
                                    }
                                    assert!(pt_stack.len() != 0, "kv_size(pt_stack)");
                                }
                                0 | _ => {}
                            }
                            assert!(pt_stack.len() != 0, "kv_size(pt_stack)");
                            cur_pt = pt_stack[pt_stack.len() - 1];
                            assert!(
                                lambda_node.is_null()
                                    || cur_pt as ::core::ffi::c_uint
                                        == kEPTLambdaArguments as ::core::ffi::c_int
                                            as ::core::ffi::c_uint,
                                "lambda_node == NULL || cur_pt == kEPTLambdaArguments"
                            );
                            match tok_type as ::core::ffi::c_uint {
                                1 | 2 | 3 => {
                                    abort();
                                }
                                0 => {
                                    is_invalid = true_0 != 0;
                                    east_set_error(
                                        pstate,
                                        &raw mut ast.err,
                                        cur_token.data.err.msg,
                                        cur_token.start,
                                    );
                                    tok_type = cur_token.data.err.type_0;
                                }
                                18 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeOperator as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.len() == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        assert!(!(*top_node_p).is_null(), "*top_node_p != NULL");
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    } else {
                                        cur_node = viml_pexpr_new_node(kExprNodeRegister);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).data.reg.name = cur_token.data.reg.name;
                                        *top_node_p = cur_node;
                                        want_node = kENodeOperator;
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidRegister\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimRegister\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        break '_viml_pexpr_parse_cycle_end;
                                    }
                                }
                                9 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        cur_node = viml_pexpr_new_node(kExprNodeUnaryPlus);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        *top_node_p = cur_node;
                                        ast_stack.push(&raw mut (*cur_node).children);
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidUnaryPlus\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimUnaryPlus\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                    } else {
                                        cur_node = viml_pexpr_new_node(kExprNodeBinaryPlus);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidBinaryPlus\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimBinaryPlus\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                    }
                                    want_node = kENodeValue;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                10 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        cur_node = viml_pexpr_new_node(kExprNodeUnaryMinus);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        *top_node_p = cur_node;
                                        ast_stack.push(&raw mut (*cur_node).children);
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidUnaryMinus\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimUnaryMinus\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                    } else {
                                        cur_node = viml_pexpr_new_node(kExprNodeBinaryMinus);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidBinaryMinus\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimBinaryMinus\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                    }
                                    want_node = kENodeValue;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                6 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Unexpected or operator: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                        (**top_node_p).start = cur_token.start;
                                        (**top_node_p).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (**top_node_p).start = prev_token.start;
                                            (**top_node_p).len =
                                                (**top_node_p).len.wrapping_add(prev_token.len);
                                        }
                                        (**top_node_p).len = 0 as size_t;
                                        want_node = kENodeOperator;
                                    }
                                    cur_node = viml_pexpr_new_node(kExprNodeOr);
                                    (*cur_node).start = cur_token.start;
                                    (*cur_node).len = cur_token.len;
                                    if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*cur_node).start = prev_token.start;
                                        (*cur_node).len =
                                            (*cur_node).len.wrapping_add(prev_token.len);
                                    }
                                    viml_parser_highlight(
                                        pstate,
                                        cur_token.start,
                                        cur_token.len,
                                        if is_invalid as ::core::ffi::c_int != 0 {
                                            b"NvimInvalidOr\0".as_ptr()
                                                as *const ::core::ffi::c_char
                                        } else {
                                            b"NvimOr\0".as_ptr() as *const ::core::ffi::c_char
                                        },
                                    );
                                    is_invalid = is_invalid as ::core::ffi::c_int
                                        | !viml_pexpr_handle_bop(
                                            pstate,
                                            &mut ast_stack,
                                            cur_node,
                                            &raw mut want_node,
                                            &raw mut ast.err,
                                        )
                                            as ::core::ffi::c_int
                                        != 0;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                7 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Unexpected and operator: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                        (**top_node_p).start = cur_token.start;
                                        (**top_node_p).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (**top_node_p).start = prev_token.start;
                                            (**top_node_p).len =
                                                (**top_node_p).len.wrapping_add(prev_token.len);
                                        }
                                        (**top_node_p).len = 0 as size_t;
                                        want_node = kENodeOperator;
                                    }
                                    cur_node = viml_pexpr_new_node(kExprNodeAnd);
                                    (*cur_node).start = cur_token.start;
                                    (*cur_node).len = cur_token.len;
                                    if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*cur_node).start = prev_token.start;
                                        (*cur_node).len =
                                            (*cur_node).len.wrapping_add(prev_token.len);
                                    }
                                    viml_parser_highlight(
                                        pstate,
                                        cur_token.start,
                                        cur_token.len,
                                        if is_invalid as ::core::ffi::c_int != 0 {
                                            b"NvimInvalidAnd\0".as_ptr()
                                                as *const ::core::ffi::c_char
                                        } else {
                                            b"NvimAnd\0".as_ptr() as *const ::core::ffi::c_char
                                        },
                                    );
                                    is_invalid = is_invalid as ::core::ffi::c_int
                                        | !viml_pexpr_handle_bop(
                                            pstate,
                                            &mut ast_stack,
                                            cur_node,
                                            &raw mut want_node,
                                            &raw mut ast.err,
                                        )
                                            as ::core::ffi::c_int
                                        != 0;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                12 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Unexpected multiplication-like operator: %.*s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                        (**top_node_p).start = cur_token.start;
                                        (**top_node_p).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (**top_node_p).start = prev_token.start;
                                            (**top_node_p).len =
                                                (**top_node_p).len.wrapping_add(prev_token.len);
                                        }
                                        (**top_node_p).len = 0 as size_t;
                                        want_node = kENodeOperator;
                                    }
                                    match cur_token.data.mul.type_0 as ::core::ffi::c_uint {
                                        0 => {
                                            cur_node = viml_pexpr_new_node(kExprNodeMultiplication);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidMultiplication\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimMultiplication\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        1 => {
                                            cur_node = viml_pexpr_new_node(kExprNodeDivision);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidDivision\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimDivision\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        2 => {
                                            cur_node = viml_pexpr_new_node(kExprNodeMod);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidMod\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimMod\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        _ => {}
                                    }
                                    is_invalid = is_invalid as ::core::ffi::c_int
                                        | !viml_pexpr_handle_bop(
                                            pstate,
                                            &mut ast_stack,
                                            cur_node,
                                            &raw mut want_node,
                                            &raw mut ast.err,
                                        )
                                            as ::core::ffi::c_int
                                        != 0;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                17 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeOperator as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.len() == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        assert!(!(*top_node_p).is_null(), "*top_node_p != NULL");
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    } else {
                                        cur_node = viml_pexpr_new_node(kExprNodeOption);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        if cur_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexInvalid as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            assert!(
                                                cur_token.len == 1 as size_t
                                                    || cur_token.len == 3 as size_t
                                                        && *pline.data.offset(
                                                            cur_token
                                                                .start
                                                                .col
                                                                .wrapping_add(2 as size_t)
                                                                as isize,
                                                        )
                                                            as ::core::ffi::c_int
                                                            == ':' as ::core::ffi::c_int,
                                                "cur_token.len == 1 || (cur_token.len == 3 && pline.data[cur_token.start.col + 2] == ':')"
                                            );
                                            (*cur_node).data.opt.ident = pline
                                                .data
                                                .offset(cur_token.start.col as isize)
                                                .offset(cur_token.len as isize);
                                            (*cur_node).data.opt.ident_len = 0 as size_t;
                                            (*cur_node).data.opt.scope = (if cur_token.len
                                                == 3 as size_t
                                            {
                                                *pline.data.offset(
                                                    cur_token.start.col.wrapping_add(1 as size_t)
                                                        as isize,
                                                )
                                                    as ExprOptScope
                                                    as ::core::ffi::c_uint
                                            } else {
                                                kExprOptScopeUnspecified as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            })
                                                as ExprOptScope;
                                        } else {
                                            (*cur_node).data.opt.ident = cur_token.data.opt.name;
                                            (*cur_node).data.opt.ident_len = cur_token.data.opt.len;
                                            (*cur_node).data.opt.scope = cur_token.data.opt.scope;
                                        }
                                        *top_node_p = cur_node;
                                        want_node = kENodeOperator;
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            1 as size_t,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidOptionSigil\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimOptionSigil\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        let scope_shift: size_t = (if cur_token.data.opt.scope
                                            as ::core::ffi::c_uint
                                            == kExprOptScopeUnspecified as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            0 as ::core::ffi::c_int
                                        } else {
                                            2 as ::core::ffi::c_int
                                        })
                                            as size_t;
                                        if scope_shift != 0 {
                                            viml_parser_highlight(
                                                pstate,
                                                shifted_pos(cur_token.start, 1 as size_t),
                                                1 as size_t,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidOptionScope\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimOptionScope\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                            viml_parser_highlight(
                                                pstate,
                                                shifted_pos(cur_token.start, 2 as size_t),
                                                1 as size_t,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidOptionScopeDelimiter\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimOptionScopeDelimiter\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        viml_parser_highlight(
                                            pstate,
                                            shifted_pos(
                                                cur_token.start,
                                                scope_shift.wrapping_add(1 as size_t),
                                            ),
                                            cur_token.len.wrapping_sub(
                                                scope_shift.wrapping_add(1 as size_t),
                                            ),
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidOptionName\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimOptionName\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        break '_viml_pexpr_parse_cycle_end;
                                    }
                                }
                                19 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeOperator as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.len() == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        assert!(!(*top_node_p).is_null(), "*top_node_p != NULL");
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    } else {
                                        cur_node = viml_pexpr_new_node(kExprNodeEnvironment);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).data.env.ident = pline
                                            .data
                                            .offset(cur_token.start.col as isize)
                                            .offset(1 as ::core::ffi::c_int as isize);
                                        (*cur_node).data.env.ident_len =
                                            cur_token.len.wrapping_sub(1 as size_t);
                                        if (*cur_node).data.env.ident_len == 0 as size_t {
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E15: Environment variable name missing\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                        }
                                        *top_node_p = cur_node;
                                        want_node = kENodeOperator;
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            1 as size_t,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidEnvironmentSigil\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimEnvironmentSigil\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        viml_parser_highlight(
                                            pstate,
                                            shifted_pos(cur_token.start, 1 as size_t),
                                            cur_token.len.wrapping_sub(1 as size_t),
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidEnvironmentName\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimEnvironmentName\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        break '_viml_pexpr_parse_cycle_end;
                                    }
                                }
                                13 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeOperator as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.len() == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        assert!(!(*top_node_p).is_null(), "*top_node_p != NULL");
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    } else {
                                        cur_node = viml_pexpr_new_node(kExprNodeNot);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        *top_node_p = cur_node;
                                        ast_stack.push(&raw mut (*cur_node).children);
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidNot\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimNot\0".as_ptr() as *const ::core::ffi::c_char
                                            },
                                        );
                                        break '_viml_pexpr_parse_cycle_end;
                                    }
                                }
                                8 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Expected value, got comparison operator: %.*s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                        (**top_node_p).start = cur_token.start;
                                        (**top_node_p).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (**top_node_p).start = prev_token.start;
                                            (**top_node_p).len =
                                                (**top_node_p).len.wrapping_add(prev_token.len);
                                        }
                                        (**top_node_p).len = 0 as size_t;
                                        want_node = kENodeOperator;
                                    }
                                    cur_node = viml_pexpr_new_node(kExprNodeComparison);
                                    (*cur_node).start = cur_token.start;
                                    (*cur_node).len = cur_token.len;
                                    if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*cur_node).start = prev_token.start;
                                        (*cur_node).len =
                                            (*cur_node).len.wrapping_add(prev_token.len);
                                    }
                                    if cur_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexInvalid as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*cur_node).data.cmp.ccs = kCCStrategyUseOption;
                                        (*cur_node).data.cmp.type_0 = kExprCmpEqual;
                                        (*cur_node).data.cmp.inv = false_0 != 0;
                                    } else {
                                        (*cur_node).data.cmp.ccs = cur_token.data.cmp.ccs;
                                        (*cur_node).data.cmp.type_0 = cur_token.data.cmp.type_0;
                                        (*cur_node).data.cmp.inv = cur_token.data.cmp.inv;
                                    }
                                    is_invalid = is_invalid as ::core::ffi::c_int
                                        | !viml_pexpr_handle_bop(
                                            pstate,
                                            &mut ast_stack,
                                            cur_node,
                                            &raw mut want_node,
                                            &raw mut ast.err,
                                        )
                                            as ::core::ffi::c_int
                                        != 0;
                                    if cur_token.data.cmp.ccs as ::core::ffi::c_uint
                                        != kCCStrategyUseOption as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len.wrapping_sub(1 as size_t),
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidComparison\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimComparison\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        viml_parser_highlight(
                                            pstate,
                                            shifted_pos(
                                                cur_token.start,
                                                cur_token.len.wrapping_sub(1 as size_t),
                                            ),
                                            1 as size_t,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidComparisonModifier\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimComparisonModifier\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                    } else {
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidComparison\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimComparison\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                    }
                                    want_node = kENodeValue;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                24 => {
                                    assert!(
                                        !(want_node as ::core::ffi::c_uint
                                            == kENodeValue as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            && cur_pt as ::core::ffi::c_uint
                                                == kEPTLambdaArguments as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint),
                                        "!(want_node == kENodeValue && cur_pt == kEPTLambdaArguments)"
                                    );
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Expected value, got comma: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        *top_node_p = cur_node;
                                        want_node = kENodeOperator;
                                    }
                                    if cur_pt as ::core::ffi::c_uint
                                        == kEPTLambdaArguments as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        assert!(!lambda_node.is_null(), "lambda_node != NULL");
                                        assert!(
                                            (*lambda_node).data.fig.type_guesses.allow_lambda,
                                            "lambda_node->data.fig.type_guesses.allow_lambda"
                                        );
                                        let node_: *mut ExprASTNode = lambda_node;
                                        assert!(
                                            (*node_).type_0 as ::core::ffi::c_uint
                                                == kExprNodeUnknownFigure as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                || (*node_).type_0 as ::core::ffi::c_uint
                                                    == kExprNodeLambda as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint,
                                            "node_->type == kExprNodeUnknownFigure || node_->type == kExprNodeLambda"
                                        );
                                        (*node_).type_0 = kExprNodeLambda;
                                        if !(*pstate).colors.is_null() {
                                            (*(*(*pstate).colors).items.offset(
                                                (*node_).data.fig.opening_hl_idx as isize,
                                            ))
                                            .group = if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidLambda\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimLambda\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            };
                                        }
                                    }
                                    's_2222: {
                                        '_viml_pexpr_parse_invalid_comma: {
                                            if ast_stack.len() >= 2 as size_t {
                                                i_0 = 1 as size_t;
                                                loop {
                                                    if i_0 >= ast_stack.len() {
                                                        break 's_2222;
                                                    }
                                                    eastnode_p = ast_stack
                                                        [ast_stack.len() - i_0 - 1]
                                                        as *const *mut ExprASTNode;
                                                    eastnode_type = (**eastnode_p).type_0;
                                                    eastnode_lvl = node_lvl(**eastnode_p);
                                                    if eastnode_type as ::core::ffi::c_uint
                                                        == kExprNodeLambda as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    {
                                                        assert!(
                                                            cur_pt as ::core::ffi::c_uint
                                                                == kEPTLambdaArguments
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                                && want_node as ::core::ffi::c_uint
                                                                    == kENodeOperator
                                                                        as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint,
                                                            "cur_pt == kEPTLambdaArguments && want_node == kENodeOperator"
                                                        );
                                                        break 's_2222;
                                                    } else {
                                                        if eastnode_type as ::core::ffi::c_uint
                                                            == kExprNodeDictLiteral
                                                                as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                            || eastnode_type as ::core::ffi::c_uint
                                                                == kExprNodeListLiteral
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            || eastnode_type as ::core::ffi::c_uint
                                                                == kExprNodeCall
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                        {
                                                            break 's_2222;
                                                        }
                                                        if !(eastnode_type as ::core::ffi::c_uint
                                                            == kExprNodeComma as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                            || eastnode_type as ::core::ffi::c_uint
                                                                == kExprNodeColon
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            || eastnode_lvl as ::core::ffi::c_uint
                                                                > kEOpLvlComma as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint)
                                                        {
                                                            break '_viml_pexpr_parse_invalid_comma;
                                                        }
                                                        if i_0
                                                            == ast_stack
                                                                .len()
                                                                .wrapping_sub(1 as size_t)
                                                        {
                                                            break '_viml_pexpr_parse_invalid_comma;
                                                        }
                                                        i_0 = i_0.wrapping_add(1);
                                                    }
                                                }
                                            }
                                        }
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Comma outside of call, lambda or literal: %.*s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                    }
                                    cur_node = viml_pexpr_new_node(kExprNodeComma);
                                    (*cur_node).start = cur_token.start;
                                    (*cur_node).len = cur_token.len;
                                    if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*cur_node).start = prev_token.start;
                                        (*cur_node).len =
                                            (*cur_node).len.wrapping_add(prev_token.len);
                                    }
                                    is_invalid = is_invalid as ::core::ffi::c_int
                                        | !viml_pexpr_handle_bop(
                                            pstate,
                                            &mut ast_stack,
                                            cur_node,
                                            &raw mut want_node,
                                            &raw mut ast.err,
                                        )
                                            as ::core::ffi::c_int
                                        != 0;
                                    viml_parser_highlight(
                                        pstate,
                                        cur_token.start,
                                        cur_token.len,
                                        if is_invalid as ::core::ffi::c_int != 0 {
                                            b"NvimInvalidComma\0".as_ptr()
                                                as *const ::core::ffi::c_char
                                        } else {
                                            b"NvimComma\0".as_ptr() as *const ::core::ffi::c_char
                                        },
                                    );
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                5 => {
                                    let mut is_ternary: bool = false_0 != 0;
                                    's_2937: {
                                        '_viml_pexpr_parse_valid_colon: {
                                            '_viml_pexpr_parse_invalid_colon: {
                                                if ast_stack.len() >= 2 as size_t {
                                                    can_be_ternary = true_0 != 0;
                                                    is_subscript = false_0 != 0;
                                                    let mut i_1: size_t = 1 as size_t;
                                                    while i_1 < ast_stack.len() {
                                                        let eastnode_p_0: *const *mut ExprASTNode =
                                                            ast_stack[ast_stack.len() - i_1 - 1]
                                                                as *const *mut ExprASTNode;
                                                        let eastnode_type_0: ExprASTNodeType =
                                                            (**eastnode_p_0).type_0;
                                                        let eastnode_lvl_0: ExprOpLvl =
                                                            node_lvl(**eastnode_p_0);
                                                        if can_be_ternary as ::core::ffi::c_int != 0
                                                            && eastnode_type_0
                                                                as ::core::ffi::c_uint
                                                                == kExprNodeTernaryValue
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            && !(**eastnode_p_0).data.ter.got_colon
                                                        {
                                                            ast_stack
                                                                .truncate(ast_stack.len() - i_1);
                                                            (**eastnode_p_0).start =
                                                                cur_token.start;
                                                            (**eastnode_p_0).len = cur_token.len;
                                                            if prev_token.type_0
                                                                as ::core::ffi::c_uint
                                                                == kExprLexSpacing
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            {
                                                                (**eastnode_p_0).start =
                                                                    prev_token.start;
                                                                (**eastnode_p_0).len =
                                                                    (**eastnode_p_0)
                                                                        .len
                                                                        .wrapping_add(
                                                                            prev_token.len,
                                                                        );
                                                            }
                                                            is_ternary = true_0 != 0;
                                                            (**eastnode_p_0).data.ter.got_colon =
                                                                true_0 != 0;
                                                            if want_node as ::core::ffi::c_uint
                                                                == kENodeValue as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            {
                                                                is_invalid = true_0 != 0;
                                                                east_set_error(
                                                                    pstate,
                                                                    &raw mut ast.err,
                                                                    gettext(
                                                                        b"E15: Expected value, got colon: %.*s\0".as_ptr()
                                                                            as *const ::core::ffi::c_char,
                                                                    ),
                                                                    cur_token.start,
                                                                );
                                                                *top_node_p = viml_pexpr_new_node(
                                                                    kExprNodeMissing,
                                                                );
                                                                (**top_node_p).start =
                                                                    cur_token.start;
                                                                (**top_node_p).len = cur_token.len;
                                                                if prev_token.type_0
                                                                    as ::core::ffi::c_uint
                                                                    == kExprLexSpacing
                                                                        as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                                {
                                                                    (**top_node_p).start =
                                                                        prev_token.start;
                                                                    (**top_node_p).len =
                                                                        (**top_node_p)
                                                                            .len
                                                                            .wrapping_add(
                                                                                prev_token.len,
                                                                            );
                                                                }
                                                                (**top_node_p).len = 0 as size_t;
                                                                want_node = kENodeOperator;
                                                            }
                                                            assert!(
                                                                !(**eastnode_p_0)
                                                                    .children
                                                                    .is_null(),
                                                                "(*eastnode_p)->children != NULL"
                                                            );
                                                            assert!(
                                                                (*(**eastnode_p_0).children)
                                                                    .next
                                                                    .is_null(),
                                                                "(*eastnode_p)->children->next == NULL"
                                                            );
                                                            ast_stack.push(
                                                                &raw mut (*(**eastnode_p_0)
                                                                    .children)
                                                                    .next,
                                                            );
                                                            break;
                                                        } else if eastnode_type_0
                                                            as ::core::ffi::c_uint
                                                            == kExprNodeUnknownFigure
                                                                as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                        {
                                                            let node__0: *mut ExprASTNode =
                                                                *eastnode_p_0;
                                                            assert!(
                                                                (*node__0).type_0
                                                                    as ::core::ffi::c_uint
                                                                    == kExprNodeUnknownFigure
                                                                        as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                                    || (*node__0).type_0
                                                                        as ::core::ffi::c_uint
                                                                        == kExprNodeDictLiteral
                                                                            as ::core::ffi::c_int
                                                                            as ::core::ffi::c_uint,
                                                                "node_->type == kExprNodeUnknownFigure || node_->type == kExprNodeDictLiteral"
                                                            );
                                                            (*node__0).type_0 =
                                                                kExprNodeDictLiteral;
                                                            if !(*pstate).colors.is_null() {
                                                                (*(*(*pstate).colors)
                                                                    .items
                                                                    .offset(
                                                                        (*node__0)
                                                                            .data
                                                                            .fig
                                                                            .opening_hl_idx
                                                                            as isize,
                                                                    ))
                                                                .group = if is_invalid
                                                                    as ::core::ffi::c_int
                                                                    != 0
                                                                {
                                                                    b"NvimInvalidDict\0".as_ptr() as *const ::core::ffi::c_char
                                                                } else {
                                                                    b"NvimDict\0".as_ptr() as *const ::core::ffi::c_char
                                                                };
                                                            }
                                                            break;
                                                        } else {
                                                            if eastnode_type_0
                                                                as ::core::ffi::c_uint
                                                                == kExprNodeDictLiteral
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            {
                                                                break;
                                                            }
                                                            if eastnode_type_0
                                                                as ::core::ffi::c_uint
                                                                == kExprNodeSubscript
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            {
                                                                is_subscript = true_0 != 0;
                                                                assert!(!is_ternary, "!is_ternary");
                                                                break;
                                                            } else {
                                                                if eastnode_type_0
                                                                    as ::core::ffi::c_uint
                                                                    == kExprNodeColon
                                                                        as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                                {
                                                                    break '_viml_pexpr_parse_invalid_colon;
                                                                }
                                                                if (eastnode_lvl_0
                                                                    as ::core::ffi::c_uint)
                                                                    < kEOpLvlTernaryValue
                                                                        as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                                {
                                                                    if (eastnode_lvl_0
                                                                        as ::core::ffi::c_uint)
                                                                        < kEOpLvlComma
                                                                            as ::core::ffi::c_int
                                                                            as ::core::ffi::c_uint
                                                                    {
                                                                        break '_viml_pexpr_parse_invalid_colon;
                                                                    }
                                                                    can_be_ternary = false_0 != 0;
                                                                }
                                                                if i_1
                                                                    == ast_stack
                                                                        .len()
                                                                        .wrapping_sub(1 as size_t)
                                                                {
                                                                    break '_viml_pexpr_parse_invalid_colon;
                                                                }
                                                                i_1 = i_1.wrapping_add(1);
                                                            }
                                                        }
                                                    }
                                                    if is_subscript {
                                                        assert!(
                                                            ast_stack.len() > 1 as size_t,
                                                            "kv_size(ast_stack) > 1"
                                                        );
                                                        if want_node as ::core::ffi::c_uint
                                                            == kENodeValue as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                            && (**stack_top(&ast_stack, 1)).type_0
                                                                as ::core::ffi::c_uint
                                                                == kExprNodeSubscript
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                        {
                                                            *top_node_p = viml_pexpr_new_node(
                                                                kExprNodeMissing,
                                                            );
                                                            (**top_node_p).start = cur_token.start;
                                                            (**top_node_p).len = cur_token.len;
                                                            if prev_token.type_0
                                                                as ::core::ffi::c_uint
                                                                == kExprLexSpacing
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            {
                                                                (**top_node_p).start =
                                                                    prev_token.start;
                                                                (**top_node_p).len = (**top_node_p)
                                                                    .len
                                                                    .wrapping_add(prev_token.len);
                                                            }
                                                            (**top_node_p).len = 0 as size_t;
                                                            want_node = kENodeOperator;
                                                        } else if want_node as ::core::ffi::c_uint
                                                            == kENodeValue as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                        {
                                                            is_invalid = true_0 != 0;
                                                            east_set_error(
                                                                pstate,
                                                                &raw mut ast.err,
                                                                gettext(
                                                                    b"E15: Expected value, got colon: %.*s\0".as_ptr()
                                                                        as *const ::core::ffi::c_char,
                                                                ),
                                                                cur_token.start,
                                                            );
                                                            *top_node_p = viml_pexpr_new_node(
                                                                kExprNodeMissing,
                                                            );
                                                            (**top_node_p).start = cur_token.start;
                                                            (**top_node_p).len = cur_token.len;
                                                            if prev_token.type_0
                                                                as ::core::ffi::c_uint
                                                                == kExprLexSpacing
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                            {
                                                                (**top_node_p).start =
                                                                    prev_token.start;
                                                                (**top_node_p).len = (**top_node_p)
                                                                    .len
                                                                    .wrapping_add(prev_token.len);
                                                            }
                                                            (**top_node_p).len = 0 as size_t;
                                                            want_node = kENodeOperator;
                                                        }
                                                        cur_node =
                                                            viml_pexpr_new_node(kExprNodeColon);
                                                        (*cur_node).start = cur_token.start;
                                                        (*cur_node).len = cur_token.len;
                                                        if prev_token.type_0 as ::core::ffi::c_uint
                                                            == kExprLexSpacing as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                        {
                                                            (*cur_node).start = prev_token.start;
                                                            (*cur_node).len = (*cur_node)
                                                                .len
                                                                .wrapping_add(prev_token.len);
                                                        }
                                                        is_invalid = is_invalid
                                                            as ::core::ffi::c_int
                                                            | !viml_pexpr_handle_bop(
                                                                pstate,
                                                                &mut ast_stack,
                                                                cur_node,
                                                                &raw mut want_node,
                                                                &raw mut ast.err,
                                                            )
                                                                as ::core::ffi::c_int
                                                            != 0;
                                                        viml_parser_highlight(
                                                            pstate,
                                                            cur_token.start,
                                                            cur_token.len,
                                                            if is_invalid as ::core::ffi::c_int != 0
                                                            {
                                                                b"NvimInvalidSubscriptColon\0"
                                                                    .as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            } else {
                                                                b"NvimSubscriptColon\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            },
                                                        );
                                                        break 's_2937;
                                                    } else {
                                                        break '_viml_pexpr_parse_valid_colon;
                                                    }
                                                }
                                            }
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E15: Colon outside of dictionary or ternary operator: %.*s\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                        }
                                        if want_node as ::core::ffi::c_uint
                                            == kENodeValue as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E15: Expected value, got colon: %.*s\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                            *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                            (**top_node_p).start = cur_token.start;
                                            (**top_node_p).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (**top_node_p).start = prev_token.start;
                                                (**top_node_p).len =
                                                    (**top_node_p).len.wrapping_add(prev_token.len);
                                            }
                                            (**top_node_p).len = 0 as size_t;
                                            want_node = kENodeOperator;
                                        }
                                        if is_ternary {
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidTernaryColon\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimTernaryColon\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        } else {
                                            cur_node = viml_pexpr_new_node(kExprNodeColon);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            is_invalid = is_invalid as ::core::ffi::c_int
                                                | !viml_pexpr_handle_bop(
                                                    pstate,
                                                    &mut ast_stack,
                                                    cur_node,
                                                    &raw mut want_node,
                                                    &raw mut ast.err,
                                                )
                                                    as ::core::ffi::c_int
                                                != 0;
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidColon\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimColon\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                    }
                                    want_node = kENodeValue;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                21 => {
                                    if cur_token.data.brc.closing {
                                        let mut new_top_node: *mut ExprASTNode =
                                            ::core::ptr::null_mut::<ExprASTNode>();
                                        let mut new_top_node_p: *mut *mut ExprASTNode =
                                            ::core::ptr::null_mut::<*mut ExprASTNode>();
                                        ast_stack.truncate(ast_stack.len() - 1 as size_t);
                                        's_3146: {
                                            if ast_stack.len() == 0 {
                                                cur_node =
                                                    viml_pexpr_new_node(kExprNodeListLiteral);
                                                (*cur_node).start = cur_token.start;
                                                (*cur_node).len = cur_token.len;
                                                if prev_token.type_0 as ::core::ffi::c_uint
                                                    == kExprLexSpacing as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    (*cur_node).start = prev_token.start;
                                                    (*cur_node).len = (*cur_node)
                                                        .len
                                                        .wrapping_add(prev_token.len);
                                                }
                                                (*cur_node).len = 0 as size_t;
                                                if want_node as ::core::ffi::c_uint
                                                    != kENodeValue as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    (*cur_node).children = *top_node_p;
                                                }
                                                *top_node_p = cur_node;
                                                new_top_node_p = top_node_p;
                                            } else {
                                                if want_node as ::core::ffi::c_uint
                                                    == kENodeValue as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    if (**stack_top(&ast_stack, 0)).type_0
                                                        as ::core::ffi::c_uint
                                                        != kExprNodeListLiteral
                                                            as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                        && (**stack_top(&ast_stack, 0)).type_0
                                                            as ::core::ffi::c_uint
                                                            != kExprNodeComma as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                        && (**stack_top(&ast_stack, 0)).type_0
                                                            as ::core::ffi::c_uint
                                                            != kExprNodeColon as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                    {
                                                        is_invalid = true_0 != 0;
                                                        east_set_error(
                                                            pstate,
                                                            &raw mut ast.err,
                                                            gettext(
                                                                b"E15: Expected value, got closing bracket: %.*s\0".as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                            ),
                                                            cur_token.start,
                                                        );
                                                    }
                                                }
                                                loop {
                                                    new_top_node_p = ast_stack
                                                        .pop()
                                                        .expect("the stack is not empty");
                                                    if !(ast_stack.len() != 0
                                                        && (new_top_node_p.is_null()
                                                            || (**new_top_node_p).type_0
                                                                as ::core::ffi::c_uint
                                                                != kExprNodeListLiteral
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                                && (**new_top_node_p).type_0
                                                                    as ::core::ffi::c_uint
                                                                    != kExprNodeSubscript
                                                                        as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint))
                                                    {
                                                        break;
                                                    }
                                                }
                                                new_top_node = *new_top_node_p;
                                                match (*new_top_node).type_0 as ::core::ffi::c_uint
                                                {
                                                    6 => {
                                                        if pt_is_assignment(cur_pt)
                                                            as ::core::ffi::c_int
                                                            != 0
                                                            && (*new_top_node).children.is_null()
                                                        {
                                                            is_invalid = true_0 != 0;
                                                            east_set_error(
                                                                pstate,
                                                                &raw mut ast.err,
                                                                gettext(
                                                                    b"E475: Unable to assign to empty list: %.*s\0".as_ptr()
                                                                        as *const ::core::ffi::c_char,
                                                                ),
                                                                cur_token.start,
                                                            );
                                                        }
                                                        viml_parser_highlight(
                                                            pstate,
                                                            cur_token.start,
                                                            cur_token.len,
                                                            if is_invalid as ::core::ffi::c_int != 0
                                                            {
                                                                b"NvimInvalidList\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            } else {
                                                                b"NvimList\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            },
                                                        );
                                                        break 's_3146;
                                                    }
                                                    5 => {
                                                        viml_parser_highlight(
                                                            pstate,
                                                            cur_token.start,
                                                            cur_token.len,
                                                            if is_invalid as ::core::ffi::c_int != 0
                                                            {
                                                                b"NvimInvalidSubscriptBracket\0"
                                                                    .as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            } else {
                                                                b"NvimSubscriptBracket\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            },
                                                        );
                                                        break 's_3146;
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            assert!(ast_stack.len() == 0, "!kv_size(ast_stack)");
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E15: Unexpected closing figure brace: %.*s\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidList\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimList\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        ast_stack.push(new_top_node_p);
                                        want_node = kENodeOperator;
                                        if ast_stack.len() <= asgn_level {
                                            assert!(
                                                ast_stack.len() == asgn_level,
                                                "kv_size(ast_stack) == asgn_level"
                                            );
                                            asgn_level = 0 as size_t;
                                            if cur_pt as ::core::ffi::c_uint
                                                == kEPTAssignment as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                assert!(!ast.err.msg.is_null(), "ast.err.msg");
                                            } else if cur_pt as ::core::ffi::c_uint
                                                == kEPTExpr as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                && pt_stack.len() > 1 as size_t
                                                && pt_is_assignment(
                                                    pt_stack[pt_stack.len() - 1 - 1],
                                                )
                                                    as ::core::ffi::c_int
                                                    != 0
                                            {
                                                pt_stack.truncate(pt_stack.len() - 1 as size_t);
                                            }
                                        }
                                        if cur_pt as ::core::ffi::c_uint
                                            == kEPTSingleAssignment as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            && ast_stack.len() == 1 as size_t
                                        {
                                            pt_stack.truncate(pt_stack.len() - 1 as size_t);
                                        }
                                        break '_viml_pexpr_parse_cycle_end;
                                    } else if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        cur_node = viml_pexpr_new_node(kExprNodeListLiteral);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        *top_node_p = cur_node;
                                        ast_stack.push(&raw mut (*cur_node).children);
                                        if cur_pt as ::core::ffi::c_uint
                                            == kEPTAssignment as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            pt_stack.push(kEPTSingleAssignment);
                                        } else if cur_pt as ::core::ffi::c_uint
                                            == kEPTSingleAssignment as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E475: Nested lists not allowed when assigning: %.*s\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                        }
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidList\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimList\0".as_ptr() as *const ::core::ffi::c_char
                                            },
                                        );
                                        break '_viml_pexpr_parse_cycle_end;
                                    } else if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.len() == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        assert!(!(*top_node_p).is_null(), "*top_node_p != NULL");
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    } else {
                                        cur_node = viml_pexpr_new_node(kExprNodeSubscript);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidSubscriptBracket\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimSubscriptBracket\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        if pt_is_assignment(cur_pt) {
                                            assert!(
                                                want_node as ::core::ffi::c_uint
                                                    == kENodeValue as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint,
                                                "want_node == kENodeValue"
                                            );
                                            asgn_level = ast_stack.len().wrapping_sub(1 as size_t);
                                            pt_stack.push(kEPTExpr);
                                        }
                                        break '_viml_pexpr_parse_cycle_end;
                                    }
                                }
                                22 => {
                                    if cur_token.data.brc.closing {
                                        let mut new_top_node_0: *mut ExprASTNode =
                                            ::core::ptr::null_mut::<ExprASTNode>();
                                        let mut new_top_node_p_0: *mut *mut ExprASTNode =
                                            ::core::ptr::null_mut::<*mut ExprASTNode>();
                                        ast_stack.truncate(ast_stack.len() - 1 as size_t);
                                        's_3806: {
                                            if ast_stack.len() == 0 {
                                                cur_node =
                                                    viml_pexpr_new_node(kExprNodeUnknownFigure);
                                                (*cur_node).start = cur_token.start;
                                                (*cur_node).len = cur_token.len;
                                                if prev_token.type_0 as ::core::ffi::c_uint
                                                    == kExprLexSpacing as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    (*cur_node).start = prev_token.start;
                                                    (*cur_node).len = (*cur_node)
                                                        .len
                                                        .wrapping_add(prev_token.len);
                                                }
                                                (*cur_node).data.fig.type_guesses.allow_lambda =
                                                    false_0 != 0;
                                                (*cur_node).data.fig.type_guesses.allow_dict =
                                                    false_0 != 0;
                                                (*cur_node).data.fig.type_guesses.allow_ident =
                                                    false_0 != 0;
                                                (*cur_node).len = 0 as size_t;
                                                if want_node as ::core::ffi::c_uint
                                                    != kENodeValue as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    (*cur_node).children = *top_node_p;
                                                }
                                                *top_node_p = cur_node;
                                                new_top_node_p_0 = top_node_p;
                                            } else {
                                                if want_node as ::core::ffi::c_uint
                                                    == kENodeValue as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    if (**stack_top(&ast_stack, 0)).type_0
                                                        as ::core::ffi::c_uint
                                                        != kExprNodeUnknownFigure
                                                            as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                        && (**stack_top(&ast_stack, 0)).type_0
                                                            as ::core::ffi::c_uint
                                                            != kExprNodeComma as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                    {
                                                        is_invalid = true_0 != 0;
                                                        east_set_error(
                                                            pstate,
                                                            &raw mut ast.err,
                                                            gettext(
                                                                b"E15: Expected value, got closing figure brace: %.*s\0"
                                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                            ),
                                                            cur_token.start,
                                                        );
                                                    }
                                                }
                                                loop {
                                                    new_top_node_p_0 = ast_stack
                                                        .pop()
                                                        .expect("the stack is not empty");
                                                    if !(ast_stack.len() != 0
                                                        && (new_top_node_p_0.is_null()
                                                            || (**new_top_node_p_0).type_0 as ::core::ffi::c_uint
                                                                != kExprNodeUnknownFigure as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                                && (**new_top_node_p_0).type_0 as ::core::ffi::c_uint
                                                                    != kExprNodeDictLiteral as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                                && (**new_top_node_p_0).type_0 as ::core::ffi::c_uint
                                                                    != kExprNodeCurlyBracesIdentifier as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                                && (**new_top_node_p_0).type_0 as ::core::ffi::c_uint
                                                                    != kExprNodeLambda as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint))
                                                    {
                                                        break;
                                                    }
                                                }
                                                new_top_node_0 = *new_top_node_p_0;
                                                match (*new_top_node_0).type_0
                                                    as ::core::ffi::c_uint
                                                {
                                                    14 => {
                                                        if (*new_top_node_0).children.is_null() {
                                                            assert!(
                                                                want_node as ::core::ffi::c_uint
                                                                    == kENodeValue
                                                                        as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint,
                                                                "want_node == kENodeValue"
                                                            );
                                                            assert!(
                                                                (*new_top_node_0)
                                                                    .data
                                                                    .fig
                                                                    .type_guesses
                                                                    .allow_dict,
                                                                "new_top_node->data.fig.type_guesses.allow_dict"
                                                            );
                                                            let node__1: *mut ExprASTNode =
                                                                new_top_node_0;
                                                            assert!(
                                                                (*node__1).type_0
                                                                    as ::core::ffi::c_uint
                                                                    == kExprNodeUnknownFigure
                                                                        as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                                    || (*node__1).type_0
                                                                        as ::core::ffi::c_uint
                                                                        == kExprNodeDictLiteral
                                                                            as ::core::ffi::c_int
                                                                            as ::core::ffi::c_uint,
                                                                "node_->type == kExprNodeUnknownFigure || node_->type == kExprNodeDictLiteral"
                                                            );
                                                            (*node__1).type_0 =
                                                                kExprNodeDictLiteral;
                                                            if !(*pstate).colors.is_null() {
                                                                (*(*(*pstate).colors)
                                                                    .items
                                                                    .offset(
                                                                        (*node__1)
                                                                            .data
                                                                            .fig
                                                                            .opening_hl_idx
                                                                            as isize,
                                                                    ))
                                                                .group = if is_invalid
                                                                    as ::core::ffi::c_int
                                                                    != 0
                                                                {
                                                                    b"NvimInvalidDict\0".as_ptr() as *const ::core::ffi::c_char
                                                                } else {
                                                                    b"NvimDict\0".as_ptr() as *const ::core::ffi::c_char
                                                                };
                                                            }
                                                            viml_parser_highlight(
                                                                pstate,
                                                                cur_token.start,
                                                                cur_token.len,
                                                                if is_invalid as ::core::ffi::c_int
                                                                    != 0
                                                                {
                                                                    b"NvimInvalidDict\0".as_ptr() as *const ::core::ffi::c_char
                                                                } else {
                                                                    b"NvimDict\0".as_ptr() as *const ::core::ffi::c_char
                                                                },
                                                            );
                                                        } else if (*new_top_node_0)
                                                            .data
                                                            .fig
                                                            .type_guesses
                                                            .allow_ident
                                                        {
                                                            let node__2: *mut ExprASTNode =
                                                                new_top_node_0;
                                                            assert!((*node__2).type_0 as ::core::ffi::c_uint == kExprNodeUnknownFigure as ::core::ffi::c_int as ::core::ffi::c_uint || (*node__2).type_0 as ::core::ffi::c_uint == kExprNodeCurlyBracesIdentifier as ::core::ffi::c_int as ::core::ffi::c_uint, "node_->type == kExprNodeUnknownFigure || node_->type == kExprNodeCurlyBracesIdentifier");
                                                            (*node__2).type_0 =
                                                                kExprNodeCurlyBracesIdentifier;
                                                            if !(*pstate).colors.is_null() {
                                                                (*(*(*pstate).colors)
                                                                    .items
                                                                    .offset(
                                                                        (*node__2)
                                                                            .data
                                                                            .fig
                                                                            .opening_hl_idx
                                                                            as isize,
                                                                    ))
                                                                .group = if is_invalid
                                                                    as ::core::ffi::c_int
                                                                    != 0
                                                                {
                                                                    b"NvimInvalidCurly\0".as_ptr() as *const ::core::ffi::c_char
                                                                } else {
                                                                    b"NvimCurly\0".as_ptr() as *const ::core::ffi::c_char
                                                                };
                                                            }
                                                            viml_parser_highlight(
                                                                pstate,
                                                                cur_token.start,
                                                                cur_token.len,
                                                                if is_invalid as ::core::ffi::c_int
                                                                    != 0
                                                                {
                                                                    b"NvimInvalidCurly\0".as_ptr() as *const ::core::ffi::c_char
                                                                } else {
                                                                    b"NvimCurly\0".as_ptr() as *const ::core::ffi::c_char
                                                                },
                                                            );
                                                        } else {
                                                            is_invalid = true_0 != 0;
                                                            east_set_error(
                                                                pstate,
                                                                &raw mut ast.err,
                                                                gettext(
                                                                    b"E15: Don't know what figure brace means: %.*s\0".as_ptr()
                                                                        as *const ::core::ffi::c_char,
                                                                ),
                                                                (*new_top_node_0).start,
                                                            );
                                                            if !(*pstate).colors.is_null() {
                                                                (*(*(*pstate).colors)
                                                                    .items
                                                                    .offset(
                                                                        (*new_top_node_0)
                                                                            .data
                                                                            .fig
                                                                            .opening_hl_idx
                                                                            as isize,
                                                                    ))
                                                                .group = if is_invalid
                                                                    as ::core::ffi::c_int
                                                                    != 0
                                                                {
                                                                    b"NvimInvalidFigureBrace\0".as_ptr()
                                                                        as *const ::core::ffi::c_char
                                                                } else {
                                                                    b"NvimFigureBrace\0".as_ptr() as *const ::core::ffi::c_char
                                                                };
                                                            }
                                                            viml_parser_highlight(
                                                                pstate,
                                                                cur_token.start,
                                                                cur_token.len,
                                                                if is_invalid as ::core::ffi::c_int
                                                                    != 0
                                                                {
                                                                    b"NvimInvalidFigureBrace\0".as_ptr()
                                                                        as *const ::core::ffi::c_char
                                                                } else {
                                                                    b"NvimFigureBrace\0".as_ptr() as *const ::core::ffi::c_char
                                                                },
                                                            );
                                                        }
                                                        break 's_3806;
                                                    }
                                                    16 => {
                                                        viml_parser_highlight(
                                                            pstate,
                                                            cur_token.start,
                                                            cur_token.len,
                                                            if is_invalid as ::core::ffi::c_int != 0
                                                            {
                                                                b"NvimInvalidDict\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            } else {
                                                                b"NvimDict\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            },
                                                        );
                                                        break 's_3806;
                                                    }
                                                    17 => {
                                                        viml_parser_highlight(
                                                            pstate,
                                                            cur_token.start,
                                                            cur_token.len,
                                                            if is_invalid as ::core::ffi::c_int != 0
                                                            {
                                                                b"NvimInvalidCurly\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            } else {
                                                                b"NvimCurly\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            },
                                                        );
                                                        break 's_3806;
                                                    }
                                                    15 => {
                                                        viml_parser_highlight(
                                                            pstate,
                                                            cur_token.start,
                                                            cur_token.len,
                                                            if is_invalid as ::core::ffi::c_int != 0
                                                            {
                                                                b"NvimInvalidLambda\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            } else {
                                                                b"NvimLambda\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                            },
                                                        );
                                                        break 's_3806;
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            assert!(ast_stack.len() == 0, "!kv_size(ast_stack)");
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E15: Unexpected closing figure brace: %.*s\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidFigureBrace\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimFigureBrace\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        ast_stack.push(new_top_node_p_0);
                                        want_node = kENodeOperator;
                                        if ast_stack.len() <= asgn_level {
                                            assert!(
                                                ast_stack.len() == asgn_level,
                                                "kv_size(ast_stack) == asgn_level"
                                            );
                                            if cur_pt as ::core::ffi::c_uint
                                                == kEPTExpr as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                && pt_stack.len() > 1 as size_t
                                                && pt_is_assignment(
                                                    pt_stack[pt_stack.len() - 1 - 1],
                                                )
                                                    as ::core::ffi::c_int
                                                    != 0
                                            {
                                                pt_stack.truncate(pt_stack.len() - 1 as size_t);
                                                asgn_level = 0 as size_t;
                                            }
                                        }
                                        break '_viml_pexpr_parse_cycle_end;
                                    } else if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidFigureBrace\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimFigureBrace\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        if pt_is_assignment(cur_pt) {
                                            cur_node =
                                                viml_pexpr_new_node(kExprNodeCurlyBracesIdentifier);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            (*cur_node).data.fig.type_guesses.allow_lambda =
                                                false_0 != 0;
                                            (*cur_node).data.fig.type_guesses.allow_dict =
                                                false_0 != 0;
                                            (*cur_node).data.fig.type_guesses.allow_ident =
                                                true_0 != 0;
                                            pt_stack.push(kEPTExpr);
                                        } else {
                                            cur_node = viml_pexpr_new_node(kExprNodeUnknownFigure);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            (*cur_node).data.fig.type_guesses.allow_lambda =
                                                true_0 != 0;
                                            (*cur_node).data.fig.type_guesses.allow_dict =
                                                true_0 != 0;
                                            (*cur_node).data.fig.type_guesses.allow_ident =
                                                true_0 != 0;
                                        }
                                        if !(*pstate).colors.is_null() {
                                            (*cur_node).data.fig.opening_hl_idx =
                                                (*(*pstate).colors).size.wrapping_sub(1 as size_t);
                                        }
                                        *top_node_p = cur_node;
                                        ast_stack.push(&raw mut (*cur_node).children);
                                        pt_stack.push(kEPTLambdaArguments);
                                        lambda_node = cur_node;
                                        break 's_4376;
                                    } else {
                                        assert!(
                                            want_node as ::core::ffi::c_uint
                                                == kENodeOperator as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint,
                                            "want_node == kENodeOperator"
                                        );
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                                && ast_stack.len() == 1 as size_t
                                            {
                                                break '_viml_pexpr_parse_end;
                                            }
                                            assert!(
                                                !(*top_node_p).is_null(),
                                                "*top_node_p != NULL"
                                            );
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                cur_token.start,
                                            );
                                            cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            (*cur_node).len = 0 as size_t;
                                            is_invalid = is_invalid as ::core::ffi::c_int
                                                | !viml_pexpr_handle_bop(
                                                    pstate,
                                                    &mut ast_stack,
                                                    cur_node,
                                                    &raw mut want_node,
                                                    &raw mut ast.err,
                                                )
                                                    as ::core::ffi::c_int
                                                != 0;
                                        } else {
                                            match (**top_node_p).type_0 as ::core::ffi::c_uint {
                                                13 | 11 | 17 => {
                                                    cur_node = viml_pexpr_new_node(
                                                        kExprNodeComplexIdentifier,
                                                    );
                                                    (*cur_node).start = cur_token.start;
                                                    (*cur_node).len = cur_token.len;
                                                    if prev_token.type_0 as ::core::ffi::c_uint
                                                        == kExprLexSpacing as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    {
                                                        (*cur_node).start = prev_token.start;
                                                        (*cur_node).len = (*cur_node)
                                                            .len
                                                            .wrapping_add(prev_token.len);
                                                    }
                                                    (*cur_node).len = 0 as size_t;
                                                    (*cur_node).children = *top_node_p;
                                                    *top_node_p = cur_node;
                                                    ast_stack.push(
                                                        &raw mut (*(*cur_node).children).next,
                                                    );
                                                    let new_top_node_p_1: *mut *mut ExprASTNode =
                                                        stack_top(&ast_stack, 0);
                                                    assert!(
                                                        (*new_top_node_p_1).is_null(),
                                                        "*new_top_node_p == NULL"
                                                    );
                                                    cur_node = viml_pexpr_new_node(
                                                        kExprNodeCurlyBracesIdentifier,
                                                    );
                                                    (*cur_node).start = cur_token.start;
                                                    (*cur_node).len = cur_token.len;
                                                    if prev_token.type_0 as ::core::ffi::c_uint
                                                        == kExprLexSpacing as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    {
                                                        (*cur_node).start = prev_token.start;
                                                        (*cur_node).len = (*cur_node)
                                                            .len
                                                            .wrapping_add(prev_token.len);
                                                    }
                                                    if !(*pstate).colors.is_null() {
                                                        (*cur_node).data.fig.opening_hl_idx =
                                                            (*(*pstate).colors).size;
                                                    }
                                                    (*cur_node)
                                                        .data
                                                        .fig
                                                        .type_guesses
                                                        .allow_lambda = false;
                                                    (*cur_node).data.fig.type_guesses.allow_dict =
                                                        false;
                                                    (*cur_node).data.fig.type_guesses.allow_ident =
                                                        true;
                                                    ast_stack.push(&raw mut (*cur_node).children);
                                                    if pt_is_assignment(cur_pt) {
                                                        pt_stack.push(kEPTExpr);
                                                    }
                                                    want_node = kENodeValue;
                                                    *new_top_node_p_1 = cur_node;
                                                    viml_parser_highlight(
                                                        pstate,
                                                        cur_token.start,
                                                        cur_token.len,
                                                        if is_invalid as ::core::ffi::c_int != 0 {
                                                            b"NvimInvalidCurly\0".as_ptr()
                                                                as *const ::core::ffi::c_char
                                                        } else {
                                                            b"NvimCurly\0".as_ptr()
                                                                as *const ::core::ffi::c_char
                                                        },
                                                    );
                                                    break 's_4376;
                                                }
                                                _ => {
                                                    if flags & kExprFlagsMulti as ::core::ffi::c_int
                                                        != 0
                                                        && ast_stack.len() == 1 as size_t
                                                    {
                                                        break '_viml_pexpr_parse_end;
                                                    }
                                                    assert!(
                                                        !(*top_node_p).is_null(),
                                                        "*top_node_p != NULL"
                                                    );
                                                    is_invalid = true_0 != 0;
                                                    east_set_error(
                                                        pstate,
                                                        &raw mut ast.err,
                                                        gettext(
                                                            b"E15: Missing operator: %.*s\0"
                                                                .as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                        ),
                                                        cur_token.start,
                                                    );
                                                    cur_node =
                                                        viml_pexpr_new_node(kExprNodeOpMissing);
                                                    (*cur_node).start = cur_token.start;
                                                    (*cur_node).len = cur_token.len;
                                                    if prev_token.type_0 as ::core::ffi::c_uint
                                                        == kExprLexSpacing as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    {
                                                        (*cur_node).start = prev_token.start;
                                                        (*cur_node).len = (*cur_node)
                                                            .len
                                                            .wrapping_add(prev_token.len);
                                                    }
                                                    (*cur_node).len = 0 as size_t;
                                                    is_invalid = is_invalid as ::core::ffi::c_int
                                                        | !viml_pexpr_handle_bop(
                                                            pstate,
                                                            &mut ast_stack,
                                                            cur_node,
                                                            &raw mut want_node,
                                                            &raw mut ast.err,
                                                        )
                                                            as ::core::ffi::c_int
                                                        != 0;
                                                }
                                            }
                                        }
                                    }
                                }
                                25 => {
                                    if cur_pt as ::core::ffi::c_uint
                                        == kEPTLambdaArguments as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        pt_stack.truncate(pt_stack.len() - 1 as size_t);
                                        assert!(pt_stack.len() != 0, "kv_size(pt_stack)");
                                        if want_node as ::core::ffi::c_uint
                                            == kENodeValue as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            ast_stack.truncate(ast_stack.len() - 1 as size_t);
                                        }
                                        assert!(
                                            ast_stack.len() >= 1 as size_t,
                                            "kv_size(ast_stack) >= 1"
                                        );
                                        while (**stack_top(&ast_stack, 0)).type_0
                                            as ::core::ffi::c_uint
                                            != kExprNodeLambda as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            && (**stack_top(&ast_stack, 0)).type_0
                                                as ::core::ffi::c_uint
                                                != kExprNodeUnknownFigure as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                        {
                                            ast_stack.truncate(ast_stack.len() - 1 as size_t);
                                        }
                                        assert!(
                                            *stack_top(&ast_stack, 0) == lambda_node,
                                            "(*kv_last(ast_stack)) == lambda_node"
                                        );
                                        let node__3: *mut ExprASTNode = lambda_node;
                                        assert!(
                                            (*node__3).type_0 as ::core::ffi::c_uint
                                                == kExprNodeUnknownFigure as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                || (*node__3).type_0 as ::core::ffi::c_uint
                                                    == kExprNodeLambda as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint,
                                            "node_->type == kExprNodeUnknownFigure || node_->type == kExprNodeLambda"
                                        );
                                        (*node__3).type_0 = kExprNodeLambda;
                                        if !(*pstate).colors.is_null() {
                                            (*(*(*pstate).colors).items.offset(
                                                (*node__3).data.fig.opening_hl_idx as isize,
                                            ))
                                            .group = if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidLambda\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimLambda\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            };
                                        }
                                        cur_node = viml_pexpr_new_node(kExprNodeArrow);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        if (*lambda_node).children.is_null() {
                                            assert!(
                                                want_node as ::core::ffi::c_uint
                                                    == kENodeValue as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint,
                                                "want_node == kENodeValue"
                                            );
                                            (*lambda_node).children = cur_node;
                                            ast_stack.push(&raw mut (*lambda_node).children);
                                        } else {
                                            assert!(
                                                (*(*lambda_node).children).next.is_null(),
                                                "lambda_node->children->next == NULL"
                                            );
                                            (*(*lambda_node).children).next = cur_node;
                                            ast_stack
                                                .push(&raw mut (*(*lambda_node).children).next);
                                        }
                                        ast_stack.push(&raw mut (*cur_node).children);
                                        lambda_node = ::core::ptr::null_mut::<ExprASTNode>();
                                    } else {
                                        if want_node as ::core::ffi::c_uint
                                            == kENodeValue as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(b"E15: Unexpected arrow: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                cur_token.start,
                                            );
                                            *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                            (**top_node_p).start = cur_token.start;
                                            (**top_node_p).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (**top_node_p).start = prev_token.start;
                                                (**top_node_p).len =
                                                    (**top_node_p).len.wrapping_add(prev_token.len);
                                            }
                                            (**top_node_p).len = 0 as size_t;
                                            want_node = kENodeOperator;
                                        }
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Arrow outside of lambda: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeArrow);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    }
                                    want_node = kENodeValue;
                                    viml_parser_highlight(
                                        pstate,
                                        cur_token.start,
                                        cur_token.len,
                                        if is_invalid as ::core::ffi::c_int != 0 {
                                            b"NvimInvalidArrow\0".as_ptr()
                                                as *const ::core::ffi::c_char
                                        } else {
                                            b"NvimArrow\0".as_ptr() as *const ::core::ffi::c_char
                                        },
                                    );
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                20 => {
                                    let scope: ExprVarScope =
                                        (if cur_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexInvalid as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            kExprVarScopeMissing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        } else {
                                            cur_token.data.var.scope as ::core::ffi::c_uint
                                        }) as ExprVarScope;
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        want_node = kENodeOperator;
                                        cur_node = viml_pexpr_new_node(
                                            (if node_is_key as ::core::ffi::c_int != 0 {
                                                kExprNodePlainKey as ::core::ffi::c_int
                                            } else {
                                                kExprNodePlainIdentifier as ::core::ffi::c_int
                                            })
                                                as ExprASTNodeType,
                                        );
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).data.var.scope = scope;
                                        let scope_shift_0: size_t = (if scope as ::core::ffi::c_uint
                                            == kExprVarScopeMissing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            0 as ::core::ffi::c_int
                                        } else {
                                            2 as ::core::ffi::c_int
                                        })
                                            as size_t;
                                        (*cur_node).data.var.ident = pline
                                            .data
                                            .offset(cur_token.start.col as isize)
                                            .offset(scope_shift_0 as isize);
                                        (*cur_node).data.var.ident_len =
                                            cur_token.len.wrapping_sub(scope_shift_0);
                                        *top_node_p = cur_node;
                                        if scope_shift_0 != 0 {
                                            assert!(!node_is_key, "!node_is_key");
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                1 as size_t,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidIdentifierScope\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimIdentifierScope\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                            viml_parser_highlight(
                                                pstate,
                                                shifted_pos(cur_token.start, 1 as size_t),
                                                1 as size_t,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidIdentifierScopeDelimiter\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimIdentifierScopeDelimiter\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        viml_parser_highlight(
                                            pstate,
                                            shifted_pos(cur_token.start, scope_shift_0),
                                            cur_token.len.wrapping_sub(scope_shift_0),
                                            if node_is_key as ::core::ffi::c_int != 0 {
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidIdentifierKey\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimIdentifierKey\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                }
                                            } else if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidIdentifierName\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimIdentifierName\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                        break '_viml_pexpr_parse_cycle_end;
                                    } else if scope as ::core::ffi::c_uint
                                        == kExprVarScopeMissing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        assert!(
                                            want_node as ::core::ffi::c_uint
                                                == kENodeOperator as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint,
                                            "want_node == kENodeOperator"
                                        );
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                                && ast_stack.len() == 1 as size_t
                                            {
                                                break '_viml_pexpr_parse_end;
                                            }
                                            assert!(
                                                !(*top_node_p).is_null(),
                                                "*top_node_p != NULL"
                                            );
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                cur_token.start,
                                            );
                                            cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            (*cur_node).len = 0 as size_t;
                                            is_invalid = is_invalid as ::core::ffi::c_int
                                                | !viml_pexpr_handle_bop(
                                                    pstate,
                                                    &mut ast_stack,
                                                    cur_node,
                                                    &raw mut want_node,
                                                    &raw mut ast.err,
                                                )
                                                    as ::core::ffi::c_int
                                                != 0;
                                        } else {
                                            match (**top_node_p).type_0 as ::core::ffi::c_uint {
                                                13 | 11 | 17 => {
                                                    cur_node = viml_pexpr_new_node(
                                                        kExprNodeComplexIdentifier,
                                                    );
                                                    (*cur_node).start = cur_token.start;
                                                    (*cur_node).len = cur_token.len;
                                                    if prev_token.type_0 as ::core::ffi::c_uint
                                                        == kExprLexSpacing as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    {
                                                        (*cur_node).start = prev_token.start;
                                                        (*cur_node).len = (*cur_node)
                                                            .len
                                                            .wrapping_add(prev_token.len);
                                                    }
                                                    (*cur_node).len = 0 as size_t;
                                                    (*cur_node).children = *top_node_p;
                                                    *top_node_p = cur_node;
                                                    ast_stack.push(
                                                        &raw mut (*(*cur_node).children).next,
                                                    );
                                                    let new_top_node_p_2: *mut *mut ExprASTNode =
                                                        stack_top(&ast_stack, 0);
                                                    assert!(
                                                        (*new_top_node_p_2).is_null(),
                                                        "*new_top_node_p == NULL"
                                                    );
                                                    cur_node = viml_pexpr_new_node(
                                                        kExprNodePlainIdentifier,
                                                    );
                                                    (*cur_node).start = cur_token.start;
                                                    (*cur_node).len = cur_token.len;
                                                    if prev_token.type_0 as ::core::ffi::c_uint
                                                        == kExprLexSpacing as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    {
                                                        (*cur_node).start = prev_token.start;
                                                        (*cur_node).len = (*cur_node)
                                                            .len
                                                            .wrapping_add(prev_token.len);
                                                    }
                                                    (*cur_node).data.var.scope = scope;
                                                    (*cur_node).data.var.ident = pline
                                                        .data
                                                        .offset(cur_token.start.col as isize);
                                                    (*cur_node).data.var.ident_len = cur_token.len;
                                                    want_node = kENodeOperator;
                                                    *new_top_node_p_2 = cur_node;
                                                    viml_parser_highlight(
                                                        pstate,
                                                        cur_token.start,
                                                        cur_token.len,
                                                        if is_invalid as ::core::ffi::c_int != 0 {
                                                            b"NvimInvalidIdentifierName\0".as_ptr()
                                                                as *const ::core::ffi::c_char
                                                        } else {
                                                            b"NvimIdentifierName\0".as_ptr()
                                                                as *const ::core::ffi::c_char
                                                        },
                                                    );
                                                    break '_viml_pexpr_parse_cycle_end;
                                                }
                                                _ => {
                                                    if flags & kExprFlagsMulti as ::core::ffi::c_int
                                                        != 0
                                                        && ast_stack.len() == 1 as size_t
                                                    {
                                                        break '_viml_pexpr_parse_end;
                                                    }
                                                    assert!(
                                                        !(*top_node_p).is_null(),
                                                        "*top_node_p != NULL"
                                                    );
                                                    is_invalid = true_0 != 0;
                                                    east_set_error(
                                                        pstate,
                                                        &raw mut ast.err,
                                                        gettext(
                                                            b"E15: Missing operator: %.*s\0"
                                                                .as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                        ),
                                                        cur_token.start,
                                                    );
                                                    cur_node =
                                                        viml_pexpr_new_node(kExprNodeOpMissing);
                                                    (*cur_node).start = cur_token.start;
                                                    (*cur_node).len = cur_token.len;
                                                    if prev_token.type_0 as ::core::ffi::c_uint
                                                        == kExprLexSpacing as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    {
                                                        (*cur_node).start = prev_token.start;
                                                        (*cur_node).len = (*cur_node)
                                                            .len
                                                            .wrapping_add(prev_token.len);
                                                    }
                                                    (*cur_node).len = 0 as size_t;
                                                    is_invalid = is_invalid as ::core::ffi::c_int
                                                        | !viml_pexpr_handle_bop(
                                                            pstate,
                                                            &mut ast_stack,
                                                            cur_node,
                                                            &raw mut want_node,
                                                            &raw mut ast.err,
                                                        )
                                                            as ::core::ffi::c_int
                                                        != 0;
                                                }
                                            }
                                        }
                                    } else {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.len() == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        assert!(!(*top_node_p).is_null(), "*top_node_p != NULL");
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    }
                                }
                                14 => {
                                    if want_node as ::core::ffi::c_uint
                                        != kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.len() == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        assert!(!(*top_node_p).is_null(), "*top_node_p != NULL");
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    } else {
                                        if node_is_key {
                                            cur_node = viml_pexpr_new_node(kExprNodePlainKey);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            (*cur_node).data.var.ident =
                                                pline.data.offset(cur_token.start.col as isize);
                                            (*cur_node).data.var.ident_len = cur_token.len;
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidIdentifierKey\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimIdentifierKey\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        } else if cur_token.data.num.is_float {
                                            cur_node = viml_pexpr_new_node(kExprNodeFloat);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            (*cur_node).data.flt.value =
                                                cur_token.data.num.val.floating;
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidFloat\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimFloat\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        } else {
                                            cur_node = viml_pexpr_new_node(kExprNodeInteger);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = cur_token.len;
                                            if prev_token.type_0 as ::core::ffi::c_uint
                                                == kExprLexSpacing as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*cur_node).start = prev_token.start;
                                                (*cur_node).len =
                                                    (*cur_node).len.wrapping_add(prev_token.len);
                                            }
                                            (*cur_node).data.num.value =
                                                cur_token.data.num.val.integer;
                                            let prefix_length: uint8_t = base_to_prefix_length
                                                [cur_token.data.num.base as usize];
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                prefix_length as size_t,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidNumberPrefix\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimNumberPrefix\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                            viml_parser_highlight(
                                                pstate,
                                                shifted_pos(
                                                    cur_token.start,
                                                    prefix_length as size_t,
                                                ),
                                                cur_token.len.wrapping_sub(prefix_length as size_t),
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidNumber\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimNumber\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        want_node = kENodeOperator;
                                        *top_node_p = cur_node;
                                        break '_viml_pexpr_parse_cycle_end;
                                    }
                                }
                                11 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Unexpected dot: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                        (**top_node_p).start = cur_token.start;
                                        (**top_node_p).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (**top_node_p).start = prev_token.start;
                                            (**top_node_p).len =
                                                (**top_node_p).len.wrapping_add(prev_token.len);
                                        }
                                        (**top_node_p).len = 0 as size_t;
                                        want_node = kENodeOperator;
                                    }
                                    if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        if cur_pt as ::core::ffi::c_uint
                                            == kEPTAssignment as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E15: Cannot concatenate in assignments: %.*s\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                        }
                                        cur_node = viml_pexpr_new_node(kExprNodeConcat);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidConcat\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimConcat\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                    } else {
                                        cur_node = viml_pexpr_new_node(kExprNodeConcatOrSubscript);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        viml_parser_highlight(
                                            pstate,
                                            cur_token.start,
                                            cur_token.len,
                                            if is_invalid as ::core::ffi::c_int != 0 {
                                                b"NvimInvalidConcatOrSubscript\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            } else {
                                                b"NvimConcatOrSubscript\0".as_ptr()
                                                    as *const ::core::ffi::c_char
                                            },
                                        );
                                    }
                                    is_invalid = is_invalid as ::core::ffi::c_int
                                        | !viml_pexpr_handle_bop(
                                            pstate,
                                            &mut ast_stack,
                                            cur_node,
                                            &raw mut want_node,
                                            &raw mut ast.err,
                                        )
                                            as ::core::ffi::c_int
                                        != 0;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                23 => {
                                    if cur_token.data.brc.closing {
                                        's_5886: {
                                            if want_node as ::core::ffi::c_uint
                                                == kENodeValue as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                if ast_stack.len() > 1 as size_t {
                                                    let prev_top_node: *const ExprASTNode =
                                                        *stack_top(&ast_stack, 1);
                                                    if (*prev_top_node).type_0
                                                        as ::core::ffi::c_uint
                                                        == kExprNodeCall as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    {
                                                        ast_stack.truncate(ast_stack.len() - 1);
                                                        break 's_5886;
                                                    }
                                                }
                                                is_invalid = true_0 != 0;
                                                east_set_error(
                                                    pstate,
                                                    &raw mut ast.err,
                                                    gettext(
                                                        b"E15: Expected value, got parenthesis: %.*s\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ),
                                                    cur_token.start,
                                                );
                                                cur_node = viml_pexpr_new_node(kExprNodeMissing);
                                                (*cur_node).start = cur_token.start;
                                                (*cur_node).len = cur_token.len;
                                                if prev_token.type_0 as ::core::ffi::c_uint
                                                    == kExprLexSpacing as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    (*cur_node).start = prev_token.start;
                                                    (*cur_node).len = (*cur_node)
                                                        .len
                                                        .wrapping_add(prev_token.len);
                                                }
                                                (*cur_node).len = 0 as size_t;
                                                *top_node_p = cur_node;
                                            } else {
                                                ast_stack.truncate(ast_stack.len() - 1 as size_t);
                                            }
                                        }
                                        let mut new_top_node_p_3: *mut *mut ExprASTNode =
                                            ::core::ptr::null_mut::<*mut ExprASTNode>();
                                        while ast_stack.len() != 0
                                            && (new_top_node_p_3.is_null()
                                                || (**new_top_node_p_3).type_0
                                                    as ::core::ffi::c_uint
                                                    != kExprNodeNested as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                    && (**new_top_node_p_3).type_0
                                                        as ::core::ffi::c_uint
                                                        != kExprNodeCall as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint)
                                        {
                                            new_top_node_p_3 =
                                                ast_stack.pop().expect("the stack is not empty");
                                        }
                                        if !new_top_node_p_3.is_null()
                                            && ((**new_top_node_p_3).type_0 as ::core::ffi::c_uint
                                                == kExprNodeNested as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                || (**new_top_node_p_3).type_0
                                                    as ::core::ffi::c_uint
                                                    == kExprNodeCall as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint)
                                        {
                                            if (**new_top_node_p_3).type_0 as ::core::ffi::c_uint
                                                == kExprNodeNested as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                viml_parser_highlight(
                                                    pstate,
                                                    cur_token.start,
                                                    cur_token.len,
                                                    if is_invalid as ::core::ffi::c_int != 0 {
                                                        b"NvimInvalidNestingParenthesis\0".as_ptr()
                                                            as *const ::core::ffi::c_char
                                                    } else {
                                                        b"NvimNestingParenthesis\0".as_ptr()
                                                            as *const ::core::ffi::c_char
                                                    },
                                                );
                                            } else {
                                                viml_parser_highlight(
                                                    pstate,
                                                    cur_token.start,
                                                    cur_token.len,
                                                    if is_invalid as ::core::ffi::c_int != 0 {
                                                        b"NvimInvalidCallingParenthesis\0".as_ptr()
                                                            as *const ::core::ffi::c_char
                                                    } else {
                                                        b"NvimCallingParenthesis\0".as_ptr()
                                                            as *const ::core::ffi::c_char
                                                    },
                                                );
                                            }
                                        } else {
                                            if new_top_node_p_3.is_null() {
                                                new_top_node_p_3 = top_node_p;
                                            }
                                            is_invalid = true_0 != 0;
                                            east_set_error(
                                                pstate,
                                                &raw mut ast.err,
                                                gettext(
                                                    b"E15: Unexpected closing parenthesis: %.*s\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                cur_token.start,
                                            );
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidNestingParenthesis\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimNestingParenthesis\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                            cur_node = viml_pexpr_new_node(kExprNodeNested);
                                            (*cur_node).start = cur_token.start;
                                            (*cur_node).len = 0 as size_t;
                                            (*cur_node).children = *new_top_node_p_3;
                                            *new_top_node_p_3 = cur_node;
                                            assert!(
                                                (*cur_node).next.is_null(),
                                                "cur_node->next == NULL"
                                            );
                                        }
                                        ast_stack.push(new_top_node_p_3);
                                        want_node = kENodeOperator;
                                        break '_viml_pexpr_parse_cycle_end;
                                    } else {
                                        match want_node as ::core::ffi::c_uint {
                                            1 => {
                                                cur_node = viml_pexpr_new_node(kExprNodeNested);
                                                (*cur_node).start = cur_token.start;
                                                (*cur_node).len = cur_token.len;
                                                if prev_token.type_0 as ::core::ffi::c_uint
                                                    == kExprLexSpacing as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    (*cur_node).start = prev_token.start;
                                                    (*cur_node).len = (*cur_node)
                                                        .len
                                                        .wrapping_add(prev_token.len);
                                                }
                                                *top_node_p = cur_node;
                                                ast_stack.push(&raw mut (*cur_node).children);
                                                viml_parser_highlight(
                                                    pstate,
                                                    cur_token.start,
                                                    cur_token.len,
                                                    if is_invalid as ::core::ffi::c_int != 0 {
                                                        b"NvimInvalidNestingParenthesis\0".as_ptr()
                                                            as *const ::core::ffi::c_char
                                                    } else {
                                                        b"NvimNestingParenthesis\0".as_ptr()
                                                            as *const ::core::ffi::c_char
                                                    },
                                                );
                                                break 's_6212;
                                            }
                                            0 => {
                                                if prev_token.type_0 as ::core::ffi::c_uint
                                                    != kExprLexSpacing as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    break;
                                                }
                                                if !((**top_node_p).type_0 as ::core::ffi::c_uint
                                                    != kExprNodePlainIdentifier
                                                        as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                    && (**top_node_p).type_0 as ::core::ffi::c_uint
                                                        != kExprNodeComplexIdentifier
                                                            as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    && (**top_node_p).type_0 as ::core::ffi::c_uint
                                                        != kExprNodeCurlyBracesIdentifier
                                                            as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint)
                                                {
                                                    break;
                                                }
                                                if flags & kExprFlagsMulti as ::core::ffi::c_int
                                                    != 0
                                                    && ast_stack.len() == 1 as size_t
                                                {
                                                    break '_viml_pexpr_parse_end;
                                                }
                                                assert!(
                                                    !(*top_node_p).is_null(),
                                                    "*top_node_p != NULL"
                                                );
                                                is_invalid = true_0 != 0;
                                                east_set_error(
                                                    pstate,
                                                    &raw mut ast.err,
                                                    gettext(
                                                        b"E15: Missing operator: %.*s\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ),
                                                    cur_token.start,
                                                );
                                                cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                                (*cur_node).start = cur_token.start;
                                                (*cur_node).len = cur_token.len;
                                                if prev_token.type_0 as ::core::ffi::c_uint
                                                    == kExprLexSpacing as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    (*cur_node).start = prev_token.start;
                                                    (*cur_node).len = (*cur_node)
                                                        .len
                                                        .wrapping_add(prev_token.len);
                                                }
                                                (*cur_node).len = 0 as size_t;
                                                is_invalid = is_invalid as ::core::ffi::c_int
                                                    | !viml_pexpr_handle_bop(
                                                        pstate,
                                                        &mut ast_stack,
                                                        cur_node,
                                                        &raw mut want_node,
                                                        &raw mut ast.err,
                                                    )
                                                        as ::core::ffi::c_int
                                                    != 0;
                                            }
                                            _ => {
                                                break 's_6212;
                                            }
                                        }
                                    }
                                }
                                4 => {
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E15: Expected value, got question mark: %.*s\0"
                                                    .as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                        *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                        (**top_node_p).start = cur_token.start;
                                        (**top_node_p).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (**top_node_p).start = prev_token.start;
                                            (**top_node_p).len =
                                                (**top_node_p).len.wrapping_add(prev_token.len);
                                        }
                                        (**top_node_p).len = 0 as size_t;
                                        want_node = kENodeOperator;
                                    }
                                    cur_node = viml_pexpr_new_node(kExprNodeTernary);
                                    (*cur_node).start = cur_token.start;
                                    (*cur_node).len = cur_token.len;
                                    if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*cur_node).start = prev_token.start;
                                        (*cur_node).len =
                                            (*cur_node).len.wrapping_add(prev_token.len);
                                    }
                                    is_invalid = is_invalid as ::core::ffi::c_int
                                        | !viml_pexpr_handle_bop(
                                            pstate,
                                            &mut ast_stack,
                                            cur_node,
                                            &raw mut want_node,
                                            &raw mut ast.err,
                                        )
                                            as ::core::ffi::c_int
                                        != 0;
                                    viml_parser_highlight(
                                        pstate,
                                        cur_token.start,
                                        cur_token.len,
                                        if is_invalid as ::core::ffi::c_int != 0 {
                                            b"NvimInvalidTernary\0".as_ptr()
                                                as *const ::core::ffi::c_char
                                        } else {
                                            b"NvimTernary\0".as_ptr() as *const ::core::ffi::c_char
                                        },
                                    );
                                    let mut ter_val_node: *mut ExprASTNode =
                                        ::core::ptr::null_mut::<ExprASTNode>();
                                    ter_val_node = viml_pexpr_new_node(kExprNodeTernaryValue);
                                    (*ter_val_node).start = cur_token.start;
                                    (*ter_val_node).len = cur_token.len;
                                    if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*ter_val_node).start = prev_token.start;
                                        (*ter_val_node).len =
                                            (*ter_val_node).len.wrapping_add(prev_token.len);
                                    }
                                    (*ter_val_node).data.ter.got_colon = false_0 != 0;
                                    assert!(
                                        !(*cur_node).children.is_null(),
                                        "cur_node->children != NULL"
                                    );
                                    assert!(
                                        (*(*cur_node).children).next.is_null(),
                                        "cur_node->children->next == NULL"
                                    );
                                    assert!(
                                        stack_top(&ast_stack, 0)
                                            == &raw mut (*(*cur_node).children).next,
                                        "kv_last(ast_stack) == &cur_node->children->next"
                                    );
                                    *stack_top(&ast_stack, 0) = ter_val_node;
                                    ast_stack.push(&raw mut (*ter_val_node).children);
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                16 | 15 => {
                                    let is_double: bool = tok_type as ::core::ffi::c_uint
                                        == kExprLexDoubleQuotedString as ::core::ffi::c_int
                                            as ::core::ffi::c_uint;
                                    if !cur_token.data.str.closed {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            if is_double as ::core::ffi::c_int != 0 {
                                                gettext(
                                                    b"E114: Missing double quote: %.*s\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                )
                                            } else {
                                                gettext(
                                                    b"E115: Missing single quote: %.*s\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                )
                                            },
                                            cur_token.start,
                                        );
                                    }
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeOperator as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        if flags & kExprFlagsMulti as ::core::ffi::c_int != 0
                                            && ast_stack.len() == 1 as size_t
                                        {
                                            break '_viml_pexpr_parse_end;
                                        }
                                        assert!(!(*top_node_p).is_null(), "*top_node_p != NULL");
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Missing operator: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        cur_node = viml_pexpr_new_node(kExprNodeOpMissing);
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        (*cur_node).len = 0 as size_t;
                                        is_invalid = is_invalid as ::core::ffi::c_int
                                            | !viml_pexpr_handle_bop(
                                                pstate,
                                                &mut ast_stack,
                                                cur_node,
                                                &raw mut want_node,
                                                &raw mut ast.err,
                                            )
                                                as ::core::ffi::c_int
                                            != 0;
                                    } else {
                                        cur_node = viml_pexpr_new_node(
                                            (if is_double as ::core::ffi::c_int != 0 {
                                                kExprNodeDoubleQuotedString as ::core::ffi::c_int
                                            } else {
                                                kExprNodeSingleQuotedString as ::core::ffi::c_int
                                            })
                                                as ExprASTNodeType,
                                        );
                                        (*cur_node).start = cur_token.start;
                                        (*cur_node).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*cur_node).start = prev_token.start;
                                            (*cur_node).len =
                                                (*cur_node).len.wrapping_add(prev_token.len);
                                        }
                                        *top_node_p = cur_node;
                                        parse_quoted_string(
                                            pstate, cur_node, cur_token, is_invalid,
                                        );
                                        want_node = kENodeOperator;
                                        break '_viml_pexpr_parse_cycle_end;
                                    }
                                }
                                26 => {
                                    if cur_pt as ::core::ffi::c_uint
                                        == kEPTAssignment as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        pt_stack.truncate(pt_stack.len() - 1 as size_t);
                                    } else if cur_pt as ::core::ffi::c_uint
                                        == kEPTSingleAssignment as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        pt_stack.truncate(pt_stack.len() - 2 as size_t);
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(
                                                b"E475: Expected closing bracket to end list assignment lvalue: %.*s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            cur_token.start,
                                        );
                                    } else {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Misplaced assignment: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                    }
                                    assert!(pt_stack.len() != 0, "kv_size(pt_stack)");
                                    assert!(
                                        pt_stack[pt_stack.len() - 1] as ::core::ffi::c_uint
                                            == kEPTExpr as ::core::ffi::c_int
                                                as ::core::ffi::c_uint,
                                        "kv_last(pt_stack) == kEPTExpr"
                                    );
                                    if want_node as ::core::ffi::c_uint
                                        == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        is_invalid = true_0 != 0;
                                        east_set_error(
                                            pstate,
                                            &raw mut ast.err,
                                            gettext(b"E15: Unexpected assignment: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            cur_token.start,
                                        );
                                        *top_node_p = viml_pexpr_new_node(kExprNodeMissing);
                                        (**top_node_p).start = cur_token.start;
                                        (**top_node_p).len = cur_token.len;
                                        if prev_token.type_0 as ::core::ffi::c_uint
                                            == kExprLexSpacing as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (**top_node_p).start = prev_token.start;
                                            (**top_node_p).len =
                                                (**top_node_p).len.wrapping_add(prev_token.len);
                                        }
                                        (**top_node_p).len = 0 as size_t;
                                        want_node = kENodeOperator;
                                    }
                                    cur_node = viml_pexpr_new_node(kExprNodeAssignment);
                                    (*cur_node).start = cur_token.start;
                                    (*cur_node).len = cur_token.len;
                                    if prev_token.type_0 as ::core::ffi::c_uint
                                        == kExprLexSpacing as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*cur_node).start = prev_token.start;
                                        (*cur_node).len =
                                            (*cur_node).len.wrapping_add(prev_token.len);
                                    }
                                    (*cur_node).data.ass.type_0 = cur_token.data.ass.type_0;
                                    match cur_token.data.ass.type_0 as ::core::ffi::c_uint {
                                        0 => {
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidPlainAssignment\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimPlainAssignment\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        1 => {
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidAssignmentWithAddition\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimAssignmentWithAddition\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        2 => {
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidAssignmentWithSubtraction\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimAssignmentWithSubtraction\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        3 => {
                                            viml_parser_highlight(
                                                pstate,
                                                cur_token.start,
                                                cur_token.len,
                                                if is_invalid as ::core::ffi::c_int != 0 {
                                                    b"NvimInvalidAssignmentWithConcatenation\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char
                                                } else {
                                                    b"NvimAssignmentWithConcatenation\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                },
                                            );
                                        }
                                        _ => {}
                                    }
                                    is_invalid = is_invalid as ::core::ffi::c_int
                                        | !viml_pexpr_handle_bop(
                                            pstate,
                                            &mut ast_stack,
                                            cur_node,
                                            &raw mut want_node,
                                            &raw mut ast.err,
                                        )
                                            as ::core::ffi::c_int
                                        != 0;
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                                _ => {
                                    break '_viml_pexpr_parse_cycle_end;
                                }
                            }
                        }
                    }
                    cur_node = viml_pexpr_new_node(kExprNodeCall);
                    (*cur_node).start = cur_token.start;
                    (*cur_node).len = cur_token.len;
                    if prev_token.type_0 as ::core::ffi::c_uint
                        == kExprLexSpacing as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        (*cur_node).start = prev_token.start;
                        (*cur_node).len = (*cur_node).len.wrapping_add(prev_token.len);
                    }
                    is_invalid = is_invalid as ::core::ffi::c_int
                        | !viml_pexpr_handle_bop(
                            pstate,
                            &mut ast_stack,
                            cur_node,
                            &raw mut want_node,
                            &raw mut ast.err,
                        ) as ::core::ffi::c_int
                        != 0;
                    viml_parser_highlight(
                        pstate,
                        cur_token.start,
                        cur_token.len,
                        if is_invalid as ::core::ffi::c_int != 0 {
                            b"NvimInvalidCallingParenthesis\0".as_ptr()
                                as *const ::core::ffi::c_char
                        } else {
                            b"NvimCallingParenthesis\0".as_ptr() as *const ::core::ffi::c_char
                        },
                    );
                    break 's_6212;
                }
                if pt_is_assignment(cur_pt) as ::core::ffi::c_int != 0
                    && !pt_is_assignment(pt_stack[pt_stack.len() - 1])
                {
                    assert!(
                        want_node as ::core::ffi::c_uint
                            == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint,
                        "want_node == kENodeValue"
                    );
                    asgn_level = ast_stack.len().wrapping_sub(1 as size_t);
                }
                break '_viml_pexpr_parse_cycle_end;
            }
            want_node = kENodeValue;
        }
        prev_token = cur_token;
        highlighted_prev_spacing = false_0 != 0;
        viml_parser_advance(&mut (*pstate).pos, &mut (*pstate).reader, cur_token.len);
    }
    assert!(pt_stack.len() != 0, "kv_size(pt_stack)");
    assert!(ast_stack.len() != 0, "kv_size(ast_stack)");
    if want_node as ::core::ffi::c_uint == kENodeValue as ::core::ffi::c_int as ::core::ffi::c_uint
        && pt_stack[pt_stack.len() - 1] as ::core::ffi::c_uint
            != kEPTLambdaArguments as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        east_set_error(
            pstate,
            &raw mut ast.err,
            gettext(b"E15: Expected value, got EOC: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            (*pstate).pos,
        );
    } else if ast_stack.len() != 1 as size_t {
        assert!(ast_stack.len() != 0, "kv_size(ast_stack)");
        ast_stack.truncate(ast_stack.len() - 1 as size_t);
        while ast.err.msg.is_null() && ast_stack.len() != 0 {
            let cur_node_0: *const ExprASTNode = *ast_stack.pop().expect("the stack is not empty");
            assert!(!cur_node_0.is_null(), "cur_node != NULL");
            match (*cur_node_0).type_0 as ::core::ffi::c_uint {
                10 => {
                    east_set_error(
                        pstate,
                        &raw mut ast.err,
                        gettext(
                            b"E116: Missing closing parenthesis for function call: %.*s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        (*cur_node_0).start,
                    );
                }
                9 => {
                    east_set_error(
                        pstate,
                        &raw mut ast.err,
                        gettext(
                            b"E110: Missing closing parenthesis for nested expression: %.*s\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        (*cur_node_0).start,
                    );
                }
                6 => {
                    east_set_error(
                        pstate,
                        &raw mut ast.err,
                        gettext(b"E697: Missing end of List ']': %.*s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        (*cur_node_0).start,
                    );
                }
                16 => {
                    east_set_error(
                        pstate,
                        &raw mut ast.err,
                        gettext(b"E723: Missing end of Dictionary '}': %.*s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        (*cur_node_0).start,
                    );
                }
                14 => {
                    east_set_error(
                        pstate,
                        &raw mut ast.err,
                        gettext(b"E15: Missing closing figure brace: %.*s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        (*cur_node_0).start,
                    );
                }
                15 => {
                    east_set_error(
                        pstate,
                        &raw mut ast.err,
                        gettext(
                            b"E15: Missing closing figure brace for lambda: %.*s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        (*cur_node_0).start,
                    );
                }
                17 => {
                    abort();
                }
                24 | 25 | 26 | 27 | 36 | 37 | 4 | 11 | 12 => {
                    abort();
                }
                18 | 19 | 20 => {}
                5 | 23 | 13 | 38 | 35 | 34 | 33 | 32 | 29 | 28 | 22 | 21 | 30 | 7 | 31 | 2 | 8 => {}
                3 => {
                    if !(*cur_node_0).data.ter.got_colon {
                        east_set_error(
                            pstate,
                            &raw mut ast.err,
                            gettext(b"E109: Missing ':' after '?': %.*s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            (*cur_node_0).start,
                        );
                    }
                }
                1 | 0 | _ => {}
            }
        }
    }
    return ast;
}
