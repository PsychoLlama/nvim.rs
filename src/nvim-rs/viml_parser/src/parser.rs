// VimL expression parser — Rust replacement for viml_pexpr_parse().
//
// Mechanical translation of 1,152-line C function with heavy macro usage.
// goto labels become labeled loops and early returns.
//
// Key goto → control-flow mapping:
//   viml_pexpr_parse_process_token  → 'process_token loop with continue
//   viml_pexpr_parse_cycle_end      → continue 'main_loop
//   viml_pexpr_parse_end            → break 'main_loop

#![allow(clippy::too_many_lines)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::nonminimal_bool)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::redundant_else)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::len_zero)]
#![allow(unused_assignments)]
#![allow(dead_code)]
#![allow(unused_unsafe)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::if_not_else)]

use std::ffi::{c_char, c_int, c_void};

use crate::expr_types::{
    AstFigTypeGuesses, AstNodeDataAss, AstNodeDataCmp, AstNodeDataEnv, AstNodeDataFig,
    AstNodeDataFlt, AstNodeDataNum, AstNodeDataOpt, AstNodeDataReg, AstNodeDataTer, ExprAST,
    ExprASTError, ExprASTNode, ExprASTNodeType, ExprAssignmentType, ExprCaseCompareStrategy,
    ExprComparisonType, ExprOptScope, ExprVarScope, LexExprToken, LexExprTokenType, ParserPosition,
};
use crate::lexer::ParserState;
use crate::quoted_string::parse_quoted_string;

// ---------------------------------------------------------------------------
// Lexer flags (matching expressions.h kELFlag*)
// ---------------------------------------------------------------------------

const KELFLAG_PEEK: c_int = 1 << 0;
const KELFLAG_FORBID_SCOPE: c_int = 1 << 1;
const KELFLAG_ALLOW_FLOAT: c_int = 1 << 2;
const KELFLAG_IS_NOT_CMP: c_int = 1 << 3;
const KELFLAG_FORBID_EOC: c_int = 1 << 4;

// Parser flags (matching expressions.h kExprFlags*)
const KEXPR_FLAGS_MULTI: c_int = 1 << 0;
const KEXPR_FLAGS_DISALLOW_EOC: c_int = 1 << 1;
const KEXPR_FLAGS_PARSE_LET: c_int = 1 << 2;

// ---------------------------------------------------------------------------
// Internal enums (not ABI-exposed to C)
// ---------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum WantedNode {
    Operator,
    Value,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum ParseType {
    Expr = 0,
    LambdaArguments,
    Assignment,
    SingleAssignment,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum OpLvl {
    Invalid = 0,
    ComplexIdentifier,
    Parens,
    Assignment,
    Arrow,
    Comma,
    Colon,
    TernaryValue,
    Ternary,
    Or,
    And,
    Comparison,
    Addition,
    Multiplication,
    Unary,
    Subscript,
    Value,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum OpAss {
    No,
    Left,
    Right,
}

// ---------------------------------------------------------------------------
// Operator precedence/associativity table
// ---------------------------------------------------------------------------

struct NodeProps {
    lvl: OpLvl,
    ass: OpAss,
}

const fn node_props(typ: ExprASTNodeType) -> NodeProps {
    use ExprASTNodeType as T;
    match typ {
        T::Missing => NodeProps {
            lvl: OpLvl::Invalid,
            ass: OpAss::No,
        },
        T::OpMissing => NodeProps {
            lvl: OpLvl::Multiplication,
            ass: OpAss::No,
        },
        T::Nested => NodeProps {
            lvl: OpLvl::Parens,
            ass: OpAss::No,
        },
        T::Call => NodeProps {
            lvl: OpLvl::Parens,
            ass: OpAss::No,
        },
        T::Subscript => NodeProps {
            lvl: OpLvl::Parens,
            ass: OpAss::No,
        },
        T::UnknownFigure => NodeProps {
            lvl: OpLvl::Parens,
            ass: OpAss::Left,
        },
        T::Lambda => NodeProps {
            lvl: OpLvl::Parens,
            ass: OpAss::No,
        },
        T::DictLiteral => NodeProps {
            lvl: OpLvl::Parens,
            ass: OpAss::No,
        },
        T::ListLiteral => NodeProps {
            lvl: OpLvl::Parens,
            ass: OpAss::No,
        },
        T::Arrow => NodeProps {
            lvl: OpLvl::Arrow,
            ass: OpAss::No,
        },
        T::Comma => NodeProps {
            lvl: OpLvl::Comma,
            ass: OpAss::Right,
        },
        T::Colon => NodeProps {
            lvl: OpLvl::Colon,
            ass: OpAss::No,
        },
        T::Ternary => NodeProps {
            lvl: OpLvl::Ternary,
            ass: OpAss::Right,
        },
        T::Or => NodeProps {
            lvl: OpLvl::Or,
            ass: OpAss::Left,
        },
        T::And => NodeProps {
            lvl: OpLvl::And,
            ass: OpAss::Left,
        },
        T::TernaryValue => NodeProps {
            lvl: OpLvl::TernaryValue,
            ass: OpAss::Right,
        },
        T::Comparison => NodeProps {
            lvl: OpLvl::Comparison,
            ass: OpAss::Right,
        },
        T::BinaryPlus => NodeProps {
            lvl: OpLvl::Addition,
            ass: OpAss::Left,
        },
        T::BinaryMinus => NodeProps {
            lvl: OpLvl::Addition,
            ass: OpAss::Left,
        },
        T::Concat => NodeProps {
            lvl: OpLvl::Addition,
            ass: OpAss::Left,
        },
        T::Multiplication => NodeProps {
            lvl: OpLvl::Multiplication,
            ass: OpAss::Left,
        },
        T::Division => NodeProps {
            lvl: OpLvl::Multiplication,
            ass: OpAss::Left,
        },
        T::Mod => NodeProps {
            lvl: OpLvl::Multiplication,
            ass: OpAss::Left,
        },
        T::UnaryPlus => NodeProps {
            lvl: OpLvl::Unary,
            ass: OpAss::No,
        },
        T::UnaryMinus => NodeProps {
            lvl: OpLvl::Unary,
            ass: OpAss::No,
        },
        T::Not => NodeProps {
            lvl: OpLvl::Unary,
            ass: OpAss::No,
        },
        T::ConcatOrSubscript => NodeProps {
            lvl: OpLvl::Subscript,
            ass: OpAss::Left,
        },
        T::CurlyBracesIdentifier => NodeProps {
            lvl: OpLvl::ComplexIdentifier,
            ass: OpAss::Left,
        },
        T::Assignment => NodeProps {
            lvl: OpLvl::Assignment,
            ass: OpAss::Left,
        },
        T::ComplexIdentifier => NodeProps {
            lvl: OpLvl::Value,
            ass: OpAss::Left,
        },
        T::PlainIdentifier => NodeProps {
            lvl: OpLvl::Value,
            ass: OpAss::No,
        },
        T::PlainKey => NodeProps {
            lvl: OpLvl::Value,
            ass: OpAss::No,
        },
        T::Register => NodeProps {
            lvl: OpLvl::Value,
            ass: OpAss::No,
        },
        T::Integer => NodeProps {
            lvl: OpLvl::Value,
            ass: OpAss::No,
        },
        T::Float => NodeProps {
            lvl: OpLvl::Value,
            ass: OpAss::No,
        },
        T::DoubleQuotedString => NodeProps {
            lvl: OpLvl::Value,
            ass: OpAss::No,
        },
        T::SingleQuotedString => NodeProps {
            lvl: OpLvl::Value,
            ass: OpAss::No,
        },
        T::Option => NodeProps {
            lvl: OpLvl::Value,
            ass: OpAss::No,
        },
        T::Environment => NodeProps {
            lvl: OpLvl::Value,
            ass: OpAss::No,
        },
    }
}

#[inline]
unsafe fn node_lvl(node: &ExprASTNode) -> OpLvl {
    node_props(node.typ).lvl
}

#[inline]
unsafe fn node_ass(node: &ExprASTNode) -> OpAss {
    node_props(node.typ).ass
}

// ---------------------------------------------------------------------------
// Extern "C" functions
// ---------------------------------------------------------------------------

extern "C" {
    fn viml_pexpr_next_token(pstate: *mut ParserState, flags: c_int) -> LexExprToken;
    fn nvim_viml_parser_advance(pstate: *mut ParserState, len: usize);
    fn nvim_viml_parser_highlight(
        pstate: *mut ParserState,
        start: ParserPosition,
        len: usize,
        group: *const c_char,
    );
    fn nvim_parser_get_line_data(
        pstate: *const ParserState,
        line_idx: usize,
        data_out: *mut *const c_char,
        size_out: *mut usize,
    );
    fn nvim_parser_get_pos(pstate: *const ParserState) -> ParserPosition;
    fn nvim_parser_get_colors(pstate: *const ParserState) -> *mut c_void;
    fn nvim_parser_get_colors_size(pstate: *const ParserState) -> usize;
    fn nvim_parser_set_color_group(pstate: *const ParserState, idx: usize, group: *const c_char);

    fn xmalloc(size: usize) -> *mut c_void;
}

// ---------------------------------------------------------------------------
// gettext — wrap error message literals
// ---------------------------------------------------------------------------

/// Pass an error message through gettext.
macro_rules! gettext_msg {
    ($s:literal) => {
        concat!($s, "\0").as_ptr().cast::<c_char>()
    };
}

// ---------------------------------------------------------------------------
// HL macro — produce highlight group name
// ---------------------------------------------------------------------------

macro_rules! hl {
    ($is_invalid:expr, $suffix:literal) => {{
        if $is_invalid {
            concat!("NvimInvalid", $suffix, "\0")
                .as_ptr()
                .cast::<c_char>()
        } else {
            concat!("Nvim", $suffix, "\0").as_ptr().cast::<c_char>()
        }
    }};
}

// ---------------------------------------------------------------------------
// base_to_prefix_length lookup
// ---------------------------------------------------------------------------

const BASE_TO_PREFIX_LEN: [u8; 17] = {
    let mut arr = [0u8; 17];
    arr[2] = 2;
    arr[8] = 1;
    arr[10] = 0;
    arr[16] = 2;
    arr
};

// ---------------------------------------------------------------------------
// Helpers: node allocation and manipulation
// ---------------------------------------------------------------------------

/// Allocate a new ExprASTNode with `xmalloc`, initialise type, children=NULL, next=NULL.
unsafe fn new_node(typ: ExprASTNodeType) -> *mut ExprASTNode {
    let ret = unsafe { xmalloc(std::mem::size_of::<ExprASTNode>()) }.cast::<ExprASTNode>();
    unsafe {
        (*ret).typ = typ;
        (*ret).children = std::ptr::null_mut();
        (*ret).next = std::ptr::null_mut();
    }
    ret
}

/// Set error in `ast_err` if not already set.
unsafe fn east_set_error(
    pstate: *const ParserState,
    ast_err: &mut ExprASTError,
    msg: *const c_char,
    start: ParserPosition,
) {
    if !ast_err.msg.is_null() {
        return;
    }
    let mut line_data: *const c_char = std::ptr::null();
    let mut line_size: usize = 0;
    unsafe {
        nvim_parser_get_line_data(pstate, start.line, &raw mut line_data, &raw mut line_size);
    }
    ast_err.msg = msg;
    if line_data.is_null() {
        ast_err.arg_len = 0;
        ast_err.arg = std::ptr::null();
    } else {
        ast_err.arg_len = (line_size - start.col) as c_int;
        ast_err.arg = unsafe { line_data.add(start.col) };
    }
}

/// Get line data pointer for the current token's line.
unsafe fn get_line_data(pstate: *const ParserState, line_idx: usize) -> *const c_char {
    let mut data: *const c_char = std::ptr::null();
    let mut _size: usize = 0;
    unsafe { nvim_parser_get_line_data(pstate, line_idx, &raw mut data, &raw mut _size) };
    data
}

/// Shifted ParserPosition.
#[inline]
fn shifted_pos(pos: ParserPosition, shift: usize) -> ParserPosition {
    ParserPosition {
        line: pos.line,
        col: pos.col + shift,
    }
}

/// ParserPosition with a new column.
#[inline]
fn recol_pos(pos: ParserPosition, new_col: usize) -> ParserPosition {
    ParserPosition {
        line: pos.line,
        col: new_col,
    }
}

/// Is the parse type one of the assignment kinds?
#[inline]
fn pt_is_assignment(pt: ParseType) -> bool {
    pt == ParseType::Assignment || pt == ParseType::SingleAssignment
}

// ---------------------------------------------------------------------------
// want_node_to_lexer_flags table
// ---------------------------------------------------------------------------

fn want_node_to_lexer_flags(want: WantedNode) -> c_int {
    match want {
        WantedNode::Value => KELFLAG_IS_NOT_CMP,
        WantedNode::Operator => KELFLAG_FORBID_SCOPE,
    }
}

// ---------------------------------------------------------------------------
// viml_pexpr_handle_bop
// ---------------------------------------------------------------------------

/// Handle binary operator: adjust AST stack for precedence.
/// Returns false if an error was set.
unsafe fn handle_bop(
    pstate: *const ParserState,
    ast_stack: &mut Vec<*mut *mut ExprASTNode>,
    bop_node: *mut ExprASTNode,
    want_node: &mut WantedNode,
    ast_err: &mut ExprASTError,
) -> bool {
    let mut ret = true;
    let mut top_node_p: *mut *mut ExprASTNode = std::ptr::null_mut();
    let mut top_node: *mut ExprASTNode = std::ptr::null_mut();
    let mut top_node_lvl = OpLvl::Invalid;
    let mut top_node_ass = OpAss::No;

    assert!(!ast_stack.is_empty());
    let bop_node_lvl = {
        let t = unsafe { (*bop_node).typ };
        if t == ExprASTNodeType::Call || t == ExprASTNodeType::Subscript {
            OpLvl::Subscript
        } else {
            unsafe { node_lvl(&*bop_node) }
        }
    };

    loop {
        let new_top_node_p: *mut *mut ExprASTNode = *ast_stack.last().unwrap();
        let new_top_node: *mut ExprASTNode = unsafe { *new_top_node_p };
        assert!(!new_top_node.is_null());
        let new_top_node_lvl = unsafe { node_lvl(&*new_top_node) };
        let new_top_node_ass = unsafe { node_ass(&*new_top_node) };
        if !top_node_p.is_null()
            && (bop_node_lvl > new_top_node_lvl
                || (bop_node_lvl == new_top_node_lvl && new_top_node_ass == OpAss::No))
        {
            break;
        }
        ast_stack.pop();
        top_node_p = new_top_node_p;
        top_node = new_top_node;
        top_node_lvl = new_top_node_lvl;
        top_node_ass = new_top_node_ass;
        if bop_node_lvl == top_node_lvl && top_node_ass == OpAss::Right {
            break;
        }
        if ast_stack.is_empty() {
            break;
        }
    }

    if top_node_ass == OpAss::Left || top_node_lvl != bop_node_lvl {
        // outer(op(x,y)) -> outer(new_op(op(x,y),*))
        unsafe { *top_node_p = bop_node };
        unsafe { (*bop_node).children = top_node };
        assert!(unsafe { (*(*bop_node).children).next }.is_null());
        ast_stack.push(top_node_p);
        ast_stack.push(unsafe { &raw mut (*(*bop_node).children).next });
    } else {
        assert!(top_node_lvl == bop_node_lvl && top_node_ass == OpAss::Right);
        assert!(
            !unsafe { (*top_node).children }.is_null()
                && !unsafe { (*(*top_node).children).next }.is_null()
        );
        // outer(op(x,y)) -> outer(op(x,new_op(y,*)))
        unsafe { (*bop_node).children = (*(*top_node).children).next };
        unsafe { (*(*top_node).children).next = bop_node };
        assert!(unsafe { (*(*bop_node).children).next }.is_null());
        ast_stack.push(top_node_p);
        ast_stack.push(unsafe { std::ptr::addr_of_mut!((*(*top_node).children).next) });
        ast_stack.push(unsafe { &raw mut (*(*bop_node).children).next });
        // Chained comparison is not associative
        if unsafe { (*bop_node).typ } == ExprASTNodeType::Comparison {
            unsafe {
                east_set_error(
                    pstate,
                    ast_err,
                    gettext_msg!("E15: Operator is not associative: %.*s"),
                    (*bop_node).start,
                );
            }
            ret = false;
        }
    }
    *want_node = WantedNode::Value;
    ret
}

// ---------------------------------------------------------------------------
// viml_pexpr_parse
// ---------------------------------------------------------------------------

/// Parse one VimScript expression.
///
/// # Safety
/// `pstate` must point to a valid C `ParserState`.
#[unsafe(export_name = "viml_pexpr_parse")]
pub unsafe extern "C" fn viml_pexpr_parse(pstate: *mut ParserState, flags: c_int) -> ExprAST {
    let mut ast = ExprAST {
        err: ExprASTError {
            msg: std::ptr::null(),
            arg_len: 0,
            arg: std::ptr::null(),
        },
        root: std::ptr::null_mut(),
    };

    // ast_stack: each element is a `*mut *mut ExprASTNode` into the tree.
    let mut ast_stack: Vec<*mut *mut ExprASTNode> = Vec::with_capacity(16);
    ast_stack.push(std::ptr::addr_of_mut!(ast.root));

    let mut want_node = WantedNode::Value;

    // pt_stack: parse-type stack
    let mut pt_stack: Vec<ParseType> = Vec::with_capacity(4);
    pt_stack.push(ParseType::Expr);
    if flags & KEXPR_FLAGS_PARSE_LET != 0 {
        pt_stack.push(ParseType::Assignment);
    }

    let mut prev_token = LexExprToken {
        start: ParserPosition { line: 0, col: 0 },
        len: 0,
        typ: LexExprTokenType::Missing,
        data: crate::expr_types::LexTknData {
            err: crate::expr_types::LexTknDataErr {
                typ: LexExprTokenType::Missing,
                msg: std::ptr::null(),
            },
        },
    };
    let mut highlighted_prev_spacing = false;
    let mut lambda_node: *mut ExprASTNode = std::ptr::null_mut();
    let mut asgn_level: usize = 0;

    'main_loop: loop {
        let is_concat_or_subscript = want_node == WantedNode::Value
            && ast_stack.len() > 1
            && unsafe { (**ast_stack[ast_stack.len() - 2]).typ }
                == ExprASTNodeType::ConcatOrSubscript;

        let lexer_additional_flags = KELFLAG_PEEK
            | (if flags & KEXPR_FLAGS_DISALLOW_EOC != 0 {
                KELFLAG_FORBID_EOC
            } else {
                0
            })
            | (if want_node == WantedNode::Value
                && (ast_stack.len() == 1
                    || (unsafe { (**ast_stack[ast_stack.len() - 2]).typ }
                        != ExprASTNodeType::Concat
                        && unsafe { (**ast_stack[ast_stack.len() - 2]).typ }
                            != ExprASTNodeType::ConcatOrSubscript))
            {
                KELFLAG_ALLOW_FLOAT
            } else {
                0
            });

        let cur_token_peeked = unsafe {
            viml_pexpr_next_token(
                pstate,
                want_node_to_lexer_flags(want_node) | lexer_additional_flags,
            )
        };
        if cur_token_peeked.typ == LexExprTokenType::EOC {
            break 'main_loop;
        }

        let mut tok_type = cur_token_peeked.typ;
        let token_invalid = tok_type == LexExprTokenType::Invalid;
        let mut is_invalid = token_invalid;

        // 'process_token: re-fetch the token (now without PEEK) and process it.
        // In C this corresponds to the label viml_pexpr_parse_process_token.
        'process_token: loop {
            let cur_token = unsafe {
                viml_pexpr_next_token(
                    pstate,
                    want_node_to_lexer_flags(want_node) | lexer_additional_flags,
                )
            };

            if tok_type == LexExprTokenType::Spacing {
                if is_invalid {
                    unsafe {
                        nvim_viml_parser_highlight(
                            pstate,
                            cur_token.start,
                            cur_token.len,
                            hl!(true, "Spacing"),
                        );
                    }
                }
                // goto cycle_end
                prev_token = cur_token;
                highlighted_prev_spacing = false;
                unsafe { nvim_viml_parser_advance(pstate, cur_token.len) };
                continue 'main_loop;
            } else if is_invalid
                && prev_token.typ == LexExprTokenType::Spacing
                && !highlighted_prev_spacing
            {
                unsafe {
                    nvim_viml_parser_highlight(
                        pstate,
                        prev_token.start,
                        prev_token.len,
                        hl!(false, "Spacing"),
                    );
                }
                is_invalid = false;
                highlighted_prev_spacing = true;
            }

            // Get current line data for token position.
            let pline_data = unsafe { get_line_data(pstate, cur_token.start.line) };

            let top_node_p: *mut *mut ExprASTNode = *ast_stack.last().unwrap();
            assert!(ast_stack.len() >= 1);
            let mut cur_node: *mut ExprASTNode = std::ptr::null_mut();

            // is_concat_or_subscript was computed before the loop iteration.
            let node_is_key = is_concat_or_subscript
                && (if cur_token.typ == LexExprTokenType::PlainIdentifier {
                    let var = unsafe { cur_token.data.var };
                    !var.autoload && var.scope == ExprVarScope::Missing
                } else {
                    cur_token.typ == LexExprTokenType::Number
                })
                && prev_token.typ != LexExprTokenType::Spacing;

            if is_concat_or_subscript && !node_is_key {
                // Turn ConcatOrSubscript into Concat.
                let idx = ast_stack.len() - 2;
                unsafe { (*(*ast_stack[idx])).typ = ExprASTNodeType::Concat };
            }

            // Pop parse-type entries for misplaced nodes.
            let is_single_assignment = *pt_stack.last().unwrap() == ParseType::SingleAssignment;
            match *pt_stack.last().unwrap() {
                ParseType::Expr => {}
                ParseType::LambdaArguments => {
                    if (want_node == WantedNode::Operator
                        && tok_type != LexExprTokenType::Comma
                        && tok_type != LexExprTokenType::Arrow)
                        || (want_node == WantedNode::Value
                            && !(cur_token.typ == LexExprTokenType::PlainIdentifier
                                && unsafe { cur_token.data.var }.scope == ExprVarScope::Missing
                                && !unsafe { cur_token.data.var }.autoload)
                            && tok_type != LexExprTokenType::Arrow)
                    {
                        unsafe { (*lambda_node).data.fig.type_guesses.allow_lambda = false };
                        if !unsafe { (*lambda_node).children }.is_null()
                            && unsafe { (*(*lambda_node).children).typ } == ExprASTNodeType::Comma
                        {
                            is_invalid = true;
                            unsafe {
                                east_set_error(
                                    pstate,
                                    &mut ast.err,
                                    gettext_msg!(
                                        "E15: Expected lambda arguments list or arrow: %.*s"
                                    ),
                                    cur_token.start,
                                );
                            }
                        } else {
                            lambda_node = std::ptr::null_mut();
                            pt_stack.pop();
                        }
                    }
                }
                ParseType::SingleAssignment | ParseType::Assignment => {
                    if want_node == WantedNode::Value
                        && tok_type != LexExprTokenType::Bracket
                        && tok_type != LexExprTokenType::PlainIdentifier
                        && !(tok_type == LexExprTokenType::FigureBrace
                            && !unsafe { cur_token.data.brc }.closing)
                        && !(node_is_key && tok_type == LexExprTokenType::Number)
                        && tok_type != LexExprTokenType::Env
                        && tok_type != LexExprTokenType::Option
                        && tok_type != LexExprTokenType::Register
                    {
                        is_invalid = true;
                        unsafe {
                            east_set_error(
                                pstate,
                                &mut ast.err,
                                gettext_msg!("E15: Expected value part of assignment lvalue: %.*s"),
                                cur_token.start,
                            );
                        }
                        pt_stack.pop();
                    } else if want_node == WantedNode::Operator
                        && tok_type != LexExprTokenType::Bracket
                        && !(tok_type == LexExprTokenType::FigureBrace
                            && !unsafe { cur_token.data.brc }.closing)
                        && tok_type != LexExprTokenType::Dot
                        && !(tok_type == LexExprTokenType::Comma && !is_single_assignment)
                        && tok_type != LexExprTokenType::Assignment
                        && !((tok_type == LexExprTokenType::PlainIdentifier
                            || (tok_type == LexExprTokenType::FigureBrace
                                && !unsafe { cur_token.data.brc }.closing))
                            && prev_token.typ != LexExprTokenType::Spacing)
                    {
                        if flags & KEXPR_FLAGS_MULTI != 0 && ast_stack.len() == 1 {
                            break 'main_loop;
                        }
                        is_invalid = true;
                        unsafe {
                            east_set_error(
                                pstate,
                                &mut ast.err,
                                gettext_msg!(
                                    "E15: Expected assignment operator or subscript: %.*s"
                                ),
                                cur_token.start,
                            );
                        }
                        pt_stack.pop();
                    }
                    assert!(!pt_stack.is_empty());
                }
            }
            assert!(!pt_stack.is_empty());
            let cur_pt = *pt_stack.last().unwrap();
            assert!(lambda_node.is_null() || cur_pt == ParseType::LambdaArguments);

            // Macro helpers (inlined).
            // NEW_NODE_WITH_CUR_POS: allocate node and set position from cur_token,
            // including preceding spacing.
            macro_rules! new_node_with_cur_pos {
                ($typ:expr) => {{
                    let n = unsafe { new_node($typ) };
                    unsafe {
                        (*n).start = cur_token.start;
                        (*n).len = cur_token.len;
                    }
                    if prev_token.typ == LexExprTokenType::Spacing {
                        unsafe {
                            (*n).start = prev_token.start;
                            (*n).len += prev_token.len;
                        }
                    }
                    n
                }};
            }

            macro_rules! hl_cur_token {
                ($suffix:literal) => {
                    unsafe {
                        nvim_viml_parser_highlight(
                            pstate,
                            cur_token.start,
                            cur_token.len,
                            hl!(is_invalid, $suffix),
                        )
                    }
                };
            }

            macro_rules! add_op_node {
                ($node:expr) => {{
                    let ok = unsafe {
                        handle_bop(pstate, &mut ast_stack, $node, &mut want_node, &mut ast.err)
                    };
                    is_invalid |= !ok;
                }};
            }

            macro_rules! add_value_if_missing {
                ($msg:literal) => {
                    if want_node == WantedNode::Value {
                        is_invalid = true;
                        unsafe {
                            east_set_error(
                                pstate,
                                &mut ast.err,
                                gettext_msg!($msg),
                                cur_token.start,
                            );
                        }
                        let n = new_node_with_cur_pos!(ExprASTNodeType::Missing);
                        unsafe { (*n).len = 0 };
                        unsafe { *top_node_p = n };
                        want_node = WantedNode::Operator;
                    }
                };
            }

            macro_rules! error_from_token_and_msg {
                ($msg:literal) => {{
                    is_invalid = true;
                    unsafe {
                        east_set_error(pstate, &mut ast.err, gettext_msg!($msg), cur_token.start);
                    }
                }};
            }

            macro_rules! error_from_node_and_msg {
                ($node:expr, $msg:literal) => {{
                    is_invalid = true;
                    unsafe {
                        east_set_error(pstate, &mut ast.err, gettext_msg!($msg), (*$node).start);
                    }
                }};
            }

            // OP_MISSING: record a missing operator and re-process.
            // Returns (via break/continue in loop) to simulate goto.
            // We use a special enum to communicate control flow.
            // In this translation: OP_MISSING either jumps to parse_end (multi
            // expression) or inserts OpMissing node and jumps to process_token.
            macro_rules! op_missing {
                () => {{
                    if flags & KEXPR_FLAGS_MULTI != 0 && ast_stack.len() == 1 {
                        break 'main_loop;
                    } else {
                        assert!(!unsafe { *top_node_p }.is_null());
                        is_invalid = true;
                        unsafe {
                            east_set_error(
                                pstate,
                                &mut ast.err,
                                gettext_msg!("E15: Missing operator: %.*s"),
                                cur_token.start,
                            );
                        }
                        let n = new_node_with_cur_pos!(ExprASTNodeType::OpMissing);
                        unsafe { (*n).len = 0 };
                        add_op_node!(n);
                        // goto process_token: restart the inner loop
                        tok_type = cur_token.typ;
                        continue 'process_token;
                    }
                }};
            }

            // SELECT_FIGURE_BRACE_TYPE
            macro_rules! select_figure_brace_type {
                ($node:expr, $new_type:expr, $hl:literal) => {{
                    let n_ = $node;
                    assert!(
                        unsafe { (*n_).typ } == ExprASTNodeType::UnknownFigure
                            || unsafe { (*n_).typ } == $new_type
                    );
                    unsafe { (*n_).typ = $new_type };
                    if !unsafe { nvim_parser_get_colors(pstate.cast_const()) }.is_null() {
                        let idx = unsafe { (*n_).data.fig.opening_hl_idx };
                        unsafe {
                            nvim_parser_set_color_group(
                                pstate.cast_const(),
                                idx,
                                hl!(is_invalid, $hl),
                            )
                        };
                    }
                }};
            }

            // ADD_IDENT macro (handles complex identifier building).
            // Returns true if OP_MISSING was triggered (re-process needed).
            macro_rules! add_ident {
                ($new_ident_code:block, $hl:literal) => {{
                    assert!(want_node == WantedNode::Operator);
                    if prev_token.typ == LexExprTokenType::Spacing {
                        op_missing!();
                    }
                    match unsafe { (*top_node_p).read().typ } {
                        ExprASTNodeType::ComplexIdentifier
                        | ExprASTNodeType::PlainIdentifier
                        | ExprASTNodeType::CurlyBracesIdentifier => {
                            let ci = new_node_with_cur_pos!(ExprASTNodeType::ComplexIdentifier);
                            unsafe { (*ci).len = 0 };
                            unsafe { (*ci).children = *top_node_p };
                            unsafe { *top_node_p = ci };
                            ast_stack.push(unsafe { &raw mut (*(*top_node_p).read().children).next });
                            let new_top_node_p: *mut *mut ExprASTNode = *ast_stack.last().unwrap();
                            assert!(unsafe { *new_top_node_p }.is_null());
                            $new_ident_code
                            unsafe { *new_top_node_p = cur_node };
                            unsafe {
                                nvim_viml_parser_highlight(
                                    pstate,
                                    cur_token.start,
                                    cur_token.len,
                                    hl!(is_invalid, $hl),
                                )
                            };
                        }
                        _ => {
                            op_missing!();
                        }
                    }
                }};
            }

            // Main token dispatch.
            match tok_type {
                LexExprTokenType::Missing | LexExprTokenType::Spacing | LexExprTokenType::EOC => {
                    panic!("viml_pexpr_parse: unexpected token type in main dispatch");
                }

                LexExprTokenType::Invalid => {
                    error_from_token_and_msg!("E15: Invalid token: %.*s");
                    tok_type = unsafe { cur_token.data.err.typ };
                    // goto process_token
                    continue 'process_token;
                }

                LexExprTokenType::Register => {
                    if want_node == WantedNode::Operator {
                        op_missing!();
                    }
                    cur_node = new_node_with_cur_pos!(ExprASTNodeType::Register);
                    unsafe {
                        (*cur_node).data.reg = AstNodeDataReg {
                            name: cur_token.data.reg.name,
                        }
                    };
                    unsafe { *top_node_p = cur_node };
                    want_node = WantedNode::Operator;
                    hl_cur_token!("Register");
                }

                LexExprTokenType::Plus => {
                    if want_node == WantedNode::Value {
                        cur_node = new_node_with_cur_pos!(ExprASTNodeType::UnaryPlus);
                        unsafe { *top_node_p = cur_node };
                        ast_stack.push(unsafe { &raw mut (*cur_node).children });
                        hl_cur_token!("UnaryPlus");
                    } else {
                        cur_node = new_node_with_cur_pos!(ExprASTNodeType::BinaryPlus);
                        add_op_node!(cur_node);
                        hl_cur_token!("BinaryPlus");
                    }
                    want_node = WantedNode::Value;
                }

                LexExprTokenType::Minus => {
                    if want_node == WantedNode::Value {
                        cur_node = new_node_with_cur_pos!(ExprASTNodeType::UnaryMinus);
                        unsafe { *top_node_p = cur_node };
                        ast_stack.push(unsafe { &raw mut (*cur_node).children });
                        hl_cur_token!("UnaryMinus");
                    } else {
                        cur_node = new_node_with_cur_pos!(ExprASTNodeType::BinaryMinus);
                        add_op_node!(cur_node);
                        hl_cur_token!("BinaryMinus");
                    }
                    want_node = WantedNode::Value;
                }

                LexExprTokenType::Or => {
                    add_value_if_missing!("E15: Unexpected or operator: %.*s");
                    cur_node = new_node_with_cur_pos!(ExprASTNodeType::Or);
                    hl_cur_token!("Or");
                    add_op_node!(cur_node);
                }

                LexExprTokenType::And => {
                    add_value_if_missing!("E15: Unexpected and operator: %.*s");
                    cur_node = new_node_with_cur_pos!(ExprASTNodeType::And);
                    hl_cur_token!("And");
                    add_op_node!(cur_node);
                }

                LexExprTokenType::Multiplication => {
                    add_value_if_missing!("E15: Unexpected multiplication-like operator: %.*s");
                    let mul = unsafe { cur_token.data.mul };
                    cur_node = match mul.typ {
                        crate::expr_types::LexExprMulType::Mul => {
                            let n = new_node_with_cur_pos!(ExprASTNodeType::Multiplication);
                            hl_cur_token!("Multiplication");
                            n
                        }
                        crate::expr_types::LexExprMulType::Div => {
                            let n = new_node_with_cur_pos!(ExprASTNodeType::Division);
                            hl_cur_token!("Division");
                            n
                        }
                        crate::expr_types::LexExprMulType::Mod => {
                            let n = new_node_with_cur_pos!(ExprASTNodeType::Mod);
                            hl_cur_token!("Mod");
                            n
                        }
                    };
                    add_op_node!(cur_node);
                }

                LexExprTokenType::Option => {
                    if want_node == WantedNode::Operator {
                        op_missing!();
                    }
                    cur_node = new_node_with_cur_pos!(ExprASTNodeType::Option);
                    if cur_token.typ == LexExprTokenType::Invalid {
                        assert!(
                            cur_token.len == 1
                                || (cur_token.len == 3
                                    && unsafe { *pline_data.add(cur_token.start.col + 2) }
                                        == b':' as c_char)
                        );
                        unsafe {
                            (*cur_node).data.opt = AstNodeDataOpt {
                                ident: pline_data.add(cur_token.start.col + cur_token.len),
                                ident_len: 0,
                                scope: if cur_token.len == 3 {
                                    ExprOptScope::try_from(
                                        *pline_data.add(cur_token.start.col + 1) as u8
                                    )
                                    .unwrap_or(ExprOptScope::Unspecified)
                                } else {
                                    ExprOptScope::Unspecified
                                },
                            }
                        };
                    } else {
                        let opt = unsafe { cur_token.data.opt };
                        unsafe {
                            (*cur_node).data.opt = AstNodeDataOpt {
                                ident: opt.name,
                                ident_len: opt.len,
                                scope: opt.scope,
                            }
                        };
                    }
                    unsafe { *top_node_p = cur_node };
                    want_node = WantedNode::Operator;
                    unsafe {
                        nvim_viml_parser_highlight(
                            pstate,
                            cur_token.start,
                            1,
                            hl!(is_invalid, "OptionSigil"),
                        )
                    };
                    let scope = unsafe { (*cur_node).data.opt.scope };
                    let scope_shift: usize = if scope == ExprOptScope::Unspecified {
                        0
                    } else {
                        2
                    };
                    if scope_shift != 0 {
                        unsafe {
                            nvim_viml_parser_highlight(
                                pstate,
                                shifted_pos(cur_token.start, 1),
                                1,
                                hl!(is_invalid, "OptionScope"),
                            );
                            nvim_viml_parser_highlight(
                                pstate,
                                shifted_pos(cur_token.start, 2),
                                1,
                                hl!(is_invalid, "OptionScopeDelimiter"),
                            );
                        }
                    }
                    unsafe {
                        nvim_viml_parser_highlight(
                            pstate,
                            shifted_pos(cur_token.start, scope_shift + 1),
                            cur_token.len - (scope_shift + 1),
                            hl!(is_invalid, "OptionName"),
                        );
                    }
                }

                LexExprTokenType::Env => {
                    if want_node == WantedNode::Operator {
                        op_missing!();
                    }
                    cur_node = new_node_with_cur_pos!(ExprASTNodeType::Environment);
                    unsafe {
                        (*cur_node).data.env = AstNodeDataEnv {
                            ident: pline_data.add(cur_token.start.col + 1),
                            ident_len: cur_token.len - 1,
                        }
                    };
                    if unsafe { (*cur_node).data.env.ident_len } == 0 {
                        error_from_token_and_msg!("E15: Environment variable name missing");
                    }
                    unsafe { *top_node_p = cur_node };
                    want_node = WantedNode::Operator;
                    unsafe {
                        nvim_viml_parser_highlight(
                            pstate,
                            cur_token.start,
                            1,
                            hl!(is_invalid, "EnvironmentSigil"),
                        );
                        nvim_viml_parser_highlight(
                            pstate,
                            shifted_pos(cur_token.start, 1),
                            cur_token.len - 1,
                            hl!(is_invalid, "EnvironmentName"),
                        );
                    }
                }

                LexExprTokenType::Not => {
                    if want_node == WantedNode::Operator {
                        op_missing!();
                    }
                    cur_node = new_node_with_cur_pos!(ExprASTNodeType::Not);
                    unsafe { *top_node_p = cur_node };
                    ast_stack.push(unsafe { &raw mut (*cur_node).children });
                    hl_cur_token!("Not");
                }

                LexExprTokenType::Comparison => {
                    add_value_if_missing!("E15: Expected value, got comparison operator: %.*s");
                    cur_node = new_node_with_cur_pos!(ExprASTNodeType::Comparison);
                    if cur_token.typ == LexExprTokenType::Invalid {
                        unsafe {
                            (*cur_node).data.cmp = AstNodeDataCmp {
                                ccs: ExprCaseCompareStrategy::UseOption,
                                typ: ExprComparisonType::Equal,
                                inv: false,
                            }
                        };
                    } else {
                        let cmp = unsafe { cur_token.data.cmp };
                        unsafe {
                            (*cur_node).data.cmp = AstNodeDataCmp {
                                ccs: cmp.ccs,
                                typ: cmp.typ,
                                inv: cmp.inv,
                            }
                        };
                    }
                    add_op_node!(cur_node);
                    let ccs = unsafe { cur_token.data.cmp.ccs };
                    if ccs != ExprCaseCompareStrategy::UseOption {
                        unsafe {
                            nvim_viml_parser_highlight(
                                pstate,
                                cur_token.start,
                                cur_token.len - 1,
                                hl!(is_invalid, "Comparison"),
                            );
                            nvim_viml_parser_highlight(
                                pstate,
                                shifted_pos(cur_token.start, cur_token.len - 1),
                                1,
                                hl!(is_invalid, "ComparisonModifier"),
                            );
                        }
                    } else {
                        hl_cur_token!("Comparison");
                    }
                    want_node = WantedNode::Value;
                }

                LexExprTokenType::Comma => {
                    assert!(
                        !(want_node == WantedNode::Value && cur_pt == ParseType::LambdaArguments)
                    );
                    if want_node == WantedNode::Value {
                        error_from_token_and_msg!("E15: Expected value, got comma: %.*s");
                        let n = new_node_with_cur_pos!(ExprASTNodeType::Missing);
                        unsafe { (*n).len = 0 };
                        unsafe { *top_node_p = n };
                        want_node = WantedNode::Operator;
                    }
                    if cur_pt == ParseType::LambdaArguments {
                        assert!(!lambda_node.is_null());
                        assert!(unsafe { (*lambda_node).data.fig.type_guesses.allow_lambda });
                        select_figure_brace_type!(lambda_node, ExprASTNodeType::Lambda, "Lambda");
                    }
                    // Validate comma placement.
                    let comma_valid = if ast_stack.len() < 2 {
                        false
                    } else {
                        let mut valid = false;
                        for i in 1..ast_stack.len() {
                            let eastnode_p = ast_stack[ast_stack.len() - 1 - i];
                            let eastnode: *mut ExprASTNode = unsafe { *eastnode_p };
                            let eastnode_type = unsafe { (*eastnode).typ };
                            let eastnode_lvl = unsafe { node_lvl(&*eastnode) };
                            if eastnode_type == ExprASTNodeType::Lambda {
                                assert!(
                                    cur_pt == ParseType::LambdaArguments
                                        && want_node == WantedNode::Operator
                                );
                                valid = true;
                                break;
                            } else if eastnode_type == ExprASTNodeType::DictLiteral
                                || eastnode_type == ExprASTNodeType::ListLiteral
                                || eastnode_type == ExprASTNodeType::Call
                            {
                                valid = true;
                                break;
                            } else if eastnode_type == ExprASTNodeType::Comma
                                || eastnode_type == ExprASTNodeType::Colon
                                || eastnode_lvl > OpLvl::Comma
                            {
                                // continue
                            } else {
                                break;
                            }
                            if i == ast_stack.len() - 1 {
                                break;
                            }
                        }
                        valid
                    };
                    if !comma_valid {
                        error_from_token_and_msg!(
                            "E15: Comma outside of call, lambda or literal: %.*s"
                        );
                    }
                    cur_node = new_node_with_cur_pos!(ExprASTNodeType::Comma);
                    add_op_node!(cur_node);
                    hl_cur_token!("Comma");
                }

                LexExprTokenType::Colon => {
                    // Translated from the complex goto-heavy colon handling.
                    let mut is_ternary = false;
                    let mut is_subscript = false;
                    let mut colon_valid = false;

                    'colon_search: {
                        if ast_stack.len() < 2 {
                            break 'colon_search;
                        }
                        let mut can_be_ternary = true;
                        for i in 1..ast_stack.len() {
                            let eastnode_p = ast_stack[ast_stack.len() - 1 - i];
                            let eastnode: *mut ExprASTNode = unsafe { *eastnode_p };
                            let eastnode_type = unsafe { (*eastnode).typ };
                            let eastnode_lvl = unsafe { node_lvl(&*eastnode) };
                            // kEOpLvlTernary > kEOpLvlComma
                            assert!(OpLvl::Ternary > OpLvl::Comma);
                            if can_be_ternary
                                && eastnode_type == ExprASTNodeType::TernaryValue
                                && !unsafe { (*eastnode).data.ter.got_colon }
                            {
                                // Drop i items from the end of ast_stack.
                                let new_len = ast_stack.len() - i;
                                ast_stack.truncate(new_len);
                                unsafe {
                                    (*eastnode).start = cur_token.start;
                                    (*eastnode).len = cur_token.len;
                                }
                                if prev_token.typ == LexExprTokenType::Spacing {
                                    unsafe {
                                        (*eastnode).start = prev_token.start;
                                        (*eastnode).len += prev_token.len;
                                    }
                                }
                                is_ternary = true;
                                unsafe { (*eastnode).data.ter.got_colon = true };
                                // ADD_VALUE_IF_MISSING for ternary colon
                                if want_node == WantedNode::Value {
                                    is_invalid = true;
                                    unsafe {
                                        east_set_error(
                                            pstate,
                                            &mut ast.err,
                                            gettext_msg!("E15: Expected value, got colon: %.*s"),
                                            cur_token.start,
                                        );
                                    }
                                    let n = new_node_with_cur_pos!(ExprASTNodeType::Missing);
                                    unsafe { (*n).len = 0 };
                                    unsafe { *top_node_p = n };
                                    want_node = WantedNode::Operator;
                                }
                                assert!(!unsafe { (*eastnode).children }.is_null());
                                assert!(unsafe { (*(*eastnode).children).next }.is_null());
                                ast_stack.push(unsafe { &raw mut (*(*eastnode).children).next });
                                colon_valid = true;
                                break 'colon_search;
                            } else if eastnode_type == ExprASTNodeType::UnknownFigure {
                                select_figure_brace_type!(
                                    eastnode,
                                    ExprASTNodeType::DictLiteral,
                                    "Dict"
                                );
                                colon_valid = true;
                                break 'colon_search;
                            } else if eastnode_type == ExprASTNodeType::DictLiteral {
                                colon_valid = true;
                                break 'colon_search;
                            } else if eastnode_type == ExprASTNodeType::Subscript {
                                is_subscript = true;
                                assert!(!is_ternary);
                                colon_valid = true;
                                break 'colon_search;
                            } else if eastnode_type == ExprASTNodeType::Colon {
                                break 'colon_search; // invalid
                            } else if eastnode_lvl >= OpLvl::TernaryValue {
                                // continue searching
                            } else if eastnode_lvl >= OpLvl::Comma {
                                can_be_ternary = false;
                            } else {
                                break 'colon_search; // invalid
                            }
                            if i == ast_stack.len() - 1 {
                                break 'colon_search;
                            }
                        }
                    }

                    if is_subscript {
                        assert!(ast_stack.len() > 1);
                        if want_node == WantedNode::Value
                            && unsafe { (**ast_stack[ast_stack.len() - 2]).typ }
                                == ExprASTNodeType::Subscript
                        {
                            let n = new_node_with_cur_pos!(ExprASTNodeType::Missing);
                            unsafe { (*n).len = 0 };
                            unsafe { *top_node_p = n };
                            want_node = WantedNode::Operator;
                        } else {
                            add_value_if_missing!("E15: Expected value, got colon: %.*s");
                        }
                        cur_node = new_node_with_cur_pos!(ExprASTNodeType::Colon);
                        add_op_node!(cur_node);
                        hl_cur_token!("SubscriptColon");
                    } else {
                        if !colon_valid {
                            error_from_token_and_msg!(
                                "E15: Colon outside of dictionary or ternary operator: %.*s"
                            );
                        }
                        add_value_if_missing!("E15: Expected value, got colon: %.*s");
                        if is_ternary {
                            hl_cur_token!("TernaryColon");
                        } else {
                            cur_node = new_node_with_cur_pos!(ExprASTNodeType::Colon);
                            add_op_node!(cur_node);
                            hl_cur_token!("Colon");
                        }
                    }
                    want_node = WantedNode::Value;
                }

                LexExprTokenType::Bracket => {
                    if unsafe { cur_token.data.brc.closing } {
                        let mut new_top_node_p: *mut *mut ExprASTNode = std::ptr::null_mut();
                        // Drop topmost value.
                        ast_stack.pop();
                        let bracket_error = if ast_stack.is_empty() {
                            // Stack empty: error
                            cur_node = new_node_with_cur_pos!(ExprASTNodeType::ListLiteral);
                            unsafe { (*cur_node).len = 0 };
                            if want_node != WantedNode::Value {
                                unsafe { (*cur_node).children = *top_node_p };
                            }
                            unsafe { *top_node_p = cur_node };
                            true
                        } else {
                            if want_node == WantedNode::Value {
                                let last = *ast_stack.last().unwrap();
                                let last_typ = unsafe { (**last).typ };
                                if last_typ != ExprASTNodeType::ListLiteral
                                    && last_typ != ExprASTNodeType::Comma
                                    && last_typ != ExprASTNodeType::Colon
                                {
                                    error_from_token_and_msg!(
                                        "E15: Expected value, got closing bracket: %.*s"
                                    );
                                }
                            }
                            // Pop until ListLiteral or Subscript.
                            loop {
                                new_top_node_p = ast_stack.pop().unwrap();
                                if new_top_node_p.is_null() {
                                    continue;
                                }
                                let t = unsafe { (*new_top_node_p).read().typ };
                                if t == ExprASTNodeType::ListLiteral
                                    || t == ExprASTNodeType::Subscript
                                {
                                    break;
                                }
                                if ast_stack.is_empty() {
                                    break;
                                }
                            }
                            false
                        };

                        if bracket_error
                            || unsafe { (*new_top_node_p).read().typ }
                                != ExprASTNodeType::ListLiteral
                                && unsafe { (*new_top_node_p).read().typ }
                                    != ExprASTNodeType::Subscript
                        {
                            // bracket closing error
                            assert!(ast_stack.is_empty());
                            error_from_token_and_msg!("E15: Unexpected closing figure brace: %.*s");
                            hl_cur_token!("List");
                        } else {
                            match unsafe { (*new_top_node_p).read().typ } {
                                ExprASTNodeType::ListLiteral => {
                                    if pt_is_assignment(cur_pt)
                                        && unsafe { (*new_top_node_p).read().children }.is_null()
                                    {
                                        error_from_token_and_msg!(
                                            "E475: Unable to assign to empty list: %.*s"
                                        );
                                    }
                                    hl_cur_token!("List");
                                }
                                ExprASTNodeType::Subscript => {
                                    hl_cur_token!("SubscriptBracket");
                                }
                                _ => unreachable!(),
                            }
                        }
                        if !new_top_node_p.is_null() {
                            ast_stack.push(new_top_node_p);
                        }
                        want_node = WantedNode::Operator;
                        if ast_stack.len() <= asgn_level {
                            assert!(ast_stack.len() == asgn_level);
                            asgn_level = 0;
                            if cur_pt == ParseType::Assignment {
                                assert!(!ast.err.msg.is_null());
                            } else if cur_pt == ParseType::Expr
                                && pt_stack.len() > 1
                                && pt_is_assignment(pt_stack[pt_stack.len() - 2])
                            {
                                pt_stack.pop();
                            }
                        }
                        if cur_pt == ParseType::SingleAssignment && ast_stack.len() == 1 {
                            pt_stack.pop();
                        }
                    } else {
                        if want_node == WantedNode::Value {
                            cur_node = new_node_with_cur_pos!(ExprASTNodeType::ListLiteral);
                            unsafe { *top_node_p = cur_node };
                            ast_stack.push(unsafe { &raw mut (*cur_node).children });
                            if cur_pt == ParseType::Assignment {
                                pt_stack.push(ParseType::SingleAssignment);
                            } else if cur_pt == ParseType::SingleAssignment {
                                error_from_token_and_msg!(
                                    "E475: Nested lists not allowed when assigning: %.*s"
                                );
                            }
                            hl_cur_token!("List");
                        } else {
                            if prev_token.typ == LexExprTokenType::Spacing {
                                op_missing!();
                            }
                            cur_node = new_node_with_cur_pos!(ExprASTNodeType::Subscript);
                            add_op_node!(cur_node);
                            hl_cur_token!("SubscriptBracket");
                            if pt_is_assignment(cur_pt) {
                                assert!(want_node == WantedNode::Value);
                                asgn_level = ast_stack.len() - 1;
                                pt_stack.push(ParseType::Expr);
                            }
                        }
                    }
                }

                LexExprTokenType::FigureBrace => {
                    if unsafe { cur_token.data.brc.closing } {
                        let mut new_top_node_p: *mut *mut ExprASTNode = std::ptr::null_mut();
                        ast_stack.pop();
                        let figure_error = if ast_stack.is_empty() {
                            cur_node = new_node_with_cur_pos!(ExprASTNodeType::UnknownFigure);
                            unsafe {
                                (*cur_node).data.fig = AstNodeDataFig {
                                    type_guesses: AstFigTypeGuesses {
                                        allow_lambda: false,
                                        allow_dict: false,
                                        allow_ident: false,
                                    },
                                    opening_hl_idx: 0,
                                };
                                (*cur_node).len = 0;
                            }
                            if want_node != WantedNode::Value {
                                unsafe { (*cur_node).children = *top_node_p };
                            }
                            unsafe { *top_node_p = cur_node };
                            new_top_node_p = top_node_p;
                            true
                        } else {
                            if want_node == WantedNode::Value {
                                let last = *ast_stack.last().unwrap();
                                let last_typ = unsafe { (**last).typ };
                                if last_typ != ExprASTNodeType::UnknownFigure
                                    && last_typ != ExprASTNodeType::Comma
                                {
                                    error_from_token_and_msg!(
                                        "E15: Expected value, got closing figure brace: %.*s"
                                    );
                                }
                            }
                            // Pop until figure brace type node.
                            loop {
                                new_top_node_p = ast_stack.pop().unwrap();
                                if new_top_node_p.is_null() {
                                    if ast_stack.is_empty() {
                                        break;
                                    }
                                    continue;
                                }
                                let t = unsafe { (*new_top_node_p).read().typ };
                                if t == ExprASTNodeType::UnknownFigure
                                    || t == ExprASTNodeType::DictLiteral
                                    || t == ExprASTNodeType::CurlyBracesIdentifier
                                    || t == ExprASTNodeType::Lambda
                                {
                                    break;
                                }
                                if ast_stack.is_empty() {
                                    break;
                                }
                            }
                            false
                        };

                        if figure_error
                            || (!new_top_node_p.is_null() && {
                                let t = unsafe { (*new_top_node_p).read().typ };
                                t != ExprASTNodeType::UnknownFigure
                                    && t != ExprASTNodeType::DictLiteral
                                    && t != ExprASTNodeType::CurlyBracesIdentifier
                                    && t != ExprASTNodeType::Lambda
                            })
                        {
                            assert!(ast_stack.is_empty());
                            error_from_token_and_msg!("E15: Unexpected closing figure brace: %.*s");
                            hl_cur_token!("FigureBrace");
                        } else {
                            let new_top_node = unsafe { *new_top_node_p };
                            match unsafe { (*new_top_node).typ } {
                                ExprASTNodeType::UnknownFigure => {
                                    if unsafe { (*new_top_node).children }.is_null() {
                                        assert!(want_node == WantedNode::Value);
                                        assert!(unsafe {
                                            (*new_top_node).data.fig.type_guesses.allow_dict
                                        });
                                        select_figure_brace_type!(
                                            new_top_node,
                                            ExprASTNodeType::DictLiteral,
                                            "Dict"
                                        );
                                        hl_cur_token!("Dict");
                                    } else if unsafe {
                                        (*new_top_node).data.fig.type_guesses.allow_ident
                                    } {
                                        select_figure_brace_type!(
                                            new_top_node,
                                            ExprASTNodeType::CurlyBracesIdentifier,
                                            "Curly"
                                        );
                                        hl_cur_token!("Curly");
                                    } else {
                                        error_from_node_and_msg!(
                                            new_top_node,
                                            "E15: Don't know what figure brace means: %.*s"
                                        );
                                        if !unsafe { nvim_parser_get_colors(pstate.cast_const()) }
                                            .is_null()
                                        {
                                            let idx =
                                                unsafe { (*new_top_node).data.fig.opening_hl_idx };
                                            unsafe {
                                                nvim_parser_set_color_group(
                                                    pstate.cast_const(),
                                                    idx,
                                                    hl!(is_invalid, "FigureBrace"),
                                                )
                                            };
                                        }
                                        hl_cur_token!("FigureBrace");
                                    }
                                }
                                ExprASTNodeType::DictLiteral => hl_cur_token!("Dict"),
                                ExprASTNodeType::CurlyBracesIdentifier => hl_cur_token!("Curly"),
                                ExprASTNodeType::Lambda => hl_cur_token!("Lambda"),
                                _ => unreachable!(),
                            }
                        }
                        if !new_top_node_p.is_null() {
                            ast_stack.push(new_top_node_p);
                        }
                        want_node = WantedNode::Operator;
                        if ast_stack.len() <= asgn_level {
                            assert!(ast_stack.len() == asgn_level);
                            if cur_pt == ParseType::Expr
                                && pt_stack.len() > 1
                                && pt_is_assignment(pt_stack[pt_stack.len() - 2])
                            {
                                pt_stack.pop();
                                asgn_level = 0;
                            }
                        }
                    } else {
                        if want_node == WantedNode::Value {
                            hl_cur_token!("FigureBrace");
                            if pt_is_assignment(cur_pt) {
                                cur_node =
                                    new_node_with_cur_pos!(ExprASTNodeType::CurlyBracesIdentifier);
                                unsafe {
                                    (*cur_node).data.fig = AstNodeDataFig {
                                        type_guesses: AstFigTypeGuesses {
                                            allow_lambda: false,
                                            allow_dict: false,
                                            allow_ident: true,
                                        },
                                        opening_hl_idx: 0,
                                    }
                                };
                                pt_stack.push(ParseType::Expr);
                            } else {
                                cur_node = new_node_with_cur_pos!(ExprASTNodeType::UnknownFigure);
                                unsafe {
                                    (*cur_node).data.fig = AstNodeDataFig {
                                        type_guesses: AstFigTypeGuesses {
                                            allow_lambda: true,
                                            allow_dict: true,
                                            allow_ident: true,
                                        },
                                        opening_hl_idx: 0,
                                    }
                                };
                            }
                            if !unsafe { nvim_parser_get_colors(pstate.cast_const()) }.is_null() {
                                unsafe {
                                    (*cur_node).data.fig.opening_hl_idx =
                                        nvim_parser_get_colors_size(pstate.cast_const()) - 1;
                                }
                            }
                            unsafe { *top_node_p = cur_node };
                            ast_stack.push(unsafe { &raw mut (*cur_node).children });
                            pt_stack.push(ParseType::LambdaArguments);
                            lambda_node = cur_node;
                        } else {
                            // Operator position: add ident.
                            add_ident!(
                                {
                                    cur_node = new_node_with_cur_pos!(
                                        ExprASTNodeType::CurlyBracesIdentifier
                                    );
                                    unsafe {
                                        (*cur_node).data.fig.opening_hl_idx =
                                            nvim_parser_get_colors_size(pstate.cast_const());
                                        (*cur_node).data.fig.type_guesses = AstFigTypeGuesses {
                                            allow_lambda: false,
                                            allow_dict: false,
                                            allow_ident: true,
                                        };
                                    }
                                    ast_stack.push(unsafe { &raw mut (*cur_node).children });
                                    if pt_is_assignment(cur_pt) {
                                        pt_stack.push(ParseType::Expr);
                                    }
                                    want_node = WantedNode::Value;
                                },
                                "Curly"
                            );
                        }
                        if pt_is_assignment(cur_pt) && !pt_is_assignment(*pt_stack.last().unwrap())
                        {
                            assert!(want_node == WantedNode::Value);
                            asgn_level = ast_stack.len() - 1;
                        }
                    }
                }

                LexExprTokenType::Arrow => {
                    if cur_pt == ParseType::LambdaArguments {
                        pt_stack.pop();
                        assert!(!pt_stack.is_empty());
                        if want_node == WantedNode::Value {
                            ast_stack.pop();
                        }
                        assert!(ast_stack.len() >= 1);
                        while unsafe { (*(**ast_stack.last().unwrap())).typ }
                            != ExprASTNodeType::Lambda
                            && unsafe { (*(**ast_stack.last().unwrap())).typ }
                                != ExprASTNodeType::UnknownFigure
                        {
                            ast_stack.pop();
                        }
                        assert!(
                            unsafe { **ast_stack.last().unwrap() == lambda_node }
                                || unsafe { (**ast_stack.last().unwrap()) == lambda_node }
                        );
                        select_figure_brace_type!(lambda_node, ExprASTNodeType::Lambda, "Lambda");
                        cur_node = new_node_with_cur_pos!(ExprASTNodeType::Arrow);
                        if unsafe { (*lambda_node).children }.is_null() {
                            assert!(want_node == WantedNode::Value);
                            unsafe { (*lambda_node).children = cur_node };
                            ast_stack.push(unsafe { &raw mut (*lambda_node).children });
                        } else {
                            assert!(unsafe { (*(*lambda_node).children).next }.is_null());
                            unsafe { (*(*lambda_node).children).next = cur_node };
                            ast_stack.push(unsafe { &raw mut (*(*lambda_node).children).next });
                        }
                        ast_stack.push(unsafe { &raw mut (*cur_node).children });
                        lambda_node = std::ptr::null_mut();
                    } else {
                        add_value_if_missing!("E15: Unexpected arrow: %.*s");
                        error_from_token_and_msg!("E15: Arrow outside of lambda: %.*s");
                        cur_node = new_node_with_cur_pos!(ExprASTNodeType::Arrow);
                        add_op_node!(cur_node);
                    }
                    want_node = WantedNode::Value;
                    hl_cur_token!("Arrow");
                }

                LexExprTokenType::PlainIdentifier => {
                    let scope = if cur_token.typ == LexExprTokenType::Invalid {
                        ExprVarScope::Missing
                    } else {
                        unsafe { cur_token.data.var.scope }
                    };
                    if want_node == WantedNode::Value {
                        want_node = WantedNode::Operator;
                        cur_node = new_node_with_cur_pos!(if node_is_key {
                            ExprASTNodeType::PlainKey
                        } else {
                            ExprASTNodeType::PlainIdentifier
                        });
                        unsafe { (*cur_node).data.var.scope = scope };
                        let scope_shift: usize = if scope == ExprVarScope::Missing { 0 } else { 2 };
                        unsafe {
                            (*cur_node).data.var.ident =
                                pline_data.add(cur_token.start.col + scope_shift);
                            (*cur_node).data.var.ident_len = cur_token.len - scope_shift;
                        }
                        unsafe { *top_node_p = cur_node };
                        if scope_shift != 0 {
                            assert!(!node_is_key);
                            unsafe {
                                nvim_viml_parser_highlight(
                                    pstate,
                                    cur_token.start,
                                    1,
                                    hl!(is_invalid, "IdentifierScope"),
                                );
                                nvim_viml_parser_highlight(
                                    pstate,
                                    shifted_pos(cur_token.start, 1),
                                    1,
                                    hl!(is_invalid, "IdentifierScopeDelimiter"),
                                );
                            }
                        }
                        unsafe {
                            nvim_viml_parser_highlight(
                                pstate,
                                shifted_pos(cur_token.start, scope_shift),
                                cur_token.len - scope_shift,
                                if node_is_key {
                                    hl!(is_invalid, "IdentifierKey")
                                } else {
                                    hl!(is_invalid, "IdentifierName")
                                },
                            );
                        }
                    } else {
                        if scope == ExprVarScope::Missing {
                            add_ident!(
                                {
                                    cur_node =
                                        new_node_with_cur_pos!(ExprASTNodeType::PlainIdentifier);
                                    unsafe {
                                        (*cur_node).data.var.scope = scope;
                                        (*cur_node).data.var.ident =
                                            pline_data.add(cur_token.start.col);
                                        (*cur_node).data.var.ident_len = cur_token.len;
                                    }
                                    want_node = WantedNode::Operator;
                                },
                                "IdentifierName"
                            );
                        } else {
                            op_missing!();
                        }
                    }
                }

                LexExprTokenType::Number => {
                    if want_node != WantedNode::Value {
                        op_missing!();
                    }
                    if node_is_key {
                        cur_node = new_node_with_cur_pos!(ExprASTNodeType::PlainKey);
                        unsafe {
                            (*cur_node).data.var.ident = pline_data.add(cur_token.start.col);
                            (*cur_node).data.var.ident_len = cur_token.len;
                        }
                        hl_cur_token!("IdentifierKey");
                    } else if unsafe { cur_token.data.num.is_float } {
                        cur_node = new_node_with_cur_pos!(ExprASTNodeType::Float);
                        unsafe {
                            (*cur_node).data.flt = AstNodeDataFlt {
                                value: cur_token.data.num.val.floating,
                            }
                        };
                        hl_cur_token!("Float");
                    } else {
                        cur_node = new_node_with_cur_pos!(ExprASTNodeType::Integer);
                        unsafe {
                            (*cur_node).data.num = AstNodeDataNum {
                                value: cur_token.data.num.val.integer,
                            }
                        };
                        let prefix_len = BASE_TO_PREFIX_LEN
                            [unsafe { cur_token.data.num.base } as usize]
                            as usize;
                        unsafe {
                            nvim_viml_parser_highlight(
                                pstate,
                                cur_token.start,
                                prefix_len,
                                hl!(is_invalid, "NumberPrefix"),
                            );
                            nvim_viml_parser_highlight(
                                pstate,
                                shifted_pos(cur_token.start, prefix_len),
                                cur_token.len - prefix_len,
                                hl!(is_invalid, "Number"),
                            );
                        }
                    }
                    want_node = WantedNode::Operator;
                    unsafe { *top_node_p = cur_node };
                }

                LexExprTokenType::Dot => {
                    add_value_if_missing!("E15: Unexpected dot: %.*s");
                    if prev_token.typ == LexExprTokenType::Spacing {
                        if cur_pt == ParseType::Assignment {
                            error_from_token_and_msg!(
                                "E15: Cannot concatenate in assignments: %.*s"
                            );
                        }
                        cur_node = new_node_with_cur_pos!(ExprASTNodeType::Concat);
                        hl_cur_token!("Concat");
                    } else {
                        cur_node = new_node_with_cur_pos!(ExprASTNodeType::ConcatOrSubscript);
                        hl_cur_token!("ConcatOrSubscript");
                    }
                    add_op_node!(cur_node);
                }

                LexExprTokenType::Parenthesis => {
                    if unsafe { cur_token.data.brc.closing } {
                        let mut no_paren_error = false;
                        if want_node == WantedNode::Value {
                            if ast_stack.len() > 1 {
                                let idx = ast_stack.len() - 2;
                                let prev_top_typ = unsafe { (*(*ast_stack[idx])).typ };
                                if prev_top_typ == ExprASTNodeType::Call {
                                    // Empty function call — no error.
                                    ast_stack.pop();
                                    no_paren_error = true;
                                }
                            }
                            if !no_paren_error {
                                error_from_token_and_msg!(
                                    "E15: Expected value, got parenthesis: %.*s"
                                );
                                let n = new_node_with_cur_pos!(ExprASTNodeType::Missing);
                                unsafe { (*n).len = 0 };
                                unsafe { *top_node_p = n };
                            }
                        } else {
                            ast_stack.pop();
                        }
                        let mut new_top_node_p: *mut *mut ExprASTNode = std::ptr::null_mut();
                        loop {
                            if ast_stack.is_empty() {
                                break;
                            }
                            new_top_node_p = ast_stack.pop().unwrap();
                            if new_top_node_p.is_null() {
                                continue;
                            }
                            let t = unsafe { (*new_top_node_p).read().typ };
                            if t == ExprASTNodeType::Nested || t == ExprASTNodeType::Call {
                                break;
                            }
                        }

                        if !new_top_node_p.is_null()
                            && (unsafe { (*new_top_node_p).read().typ } == ExprASTNodeType::Nested
                                || unsafe { (*new_top_node_p).read().typ } == ExprASTNodeType::Call)
                        {
                            if unsafe { (*new_top_node_p).read().typ } == ExprASTNodeType::Nested {
                                hl_cur_token!("NestingParenthesis");
                            } else {
                                hl_cur_token!("CallingParenthesis");
                            }
                        } else {
                            if new_top_node_p.is_null() {
                                new_top_node_p = top_node_p;
                            }
                            error_from_token_and_msg!("E15: Unexpected closing parenthesis: %.*s");
                            hl_cur_token!("NestingParenthesis");
                            cur_node = unsafe { new_node(ExprASTNodeType::Nested) };
                            unsafe {
                                (*cur_node).start = cur_token.start;
                                (*cur_node).len = 0;
                                (*cur_node).children = *new_top_node_p;
                                *new_top_node_p = cur_node;
                            }
                            assert!(unsafe { (*cur_node).next }.is_null());
                        }
                        ast_stack.push(new_top_node_p);
                        want_node = WantedNode::Operator;
                    } else {
                        match want_node {
                            WantedNode::Value => {
                                cur_node = new_node_with_cur_pos!(ExprASTNodeType::Nested);
                                unsafe { *top_node_p = cur_node };
                                ast_stack.push(unsafe { &raw mut (*cur_node).children });
                                hl_cur_token!("NestingParenthesis");
                            }
                            WantedNode::Operator => {
                                if prev_token.typ == LexExprTokenType::Spacing {
                                    let top_typ = unsafe { (*top_node_p).read().typ };
                                    if top_typ != ExprASTNodeType::PlainIdentifier
                                        && top_typ != ExprASTNodeType::ComplexIdentifier
                                        && top_typ != ExprASTNodeType::CurlyBracesIdentifier
                                    {
                                        op_missing!();
                                    }
                                }
                                cur_node = new_node_with_cur_pos!(ExprASTNodeType::Call);
                                add_op_node!(cur_node);
                                hl_cur_token!("CallingParenthesis");
                            }
                        }
                        want_node = WantedNode::Value;
                    }
                }

                LexExprTokenType::Question => {
                    add_value_if_missing!("E15: Expected value, got question mark: %.*s");
                    cur_node = new_node_with_cur_pos!(ExprASTNodeType::Ternary);
                    add_op_node!(cur_node);
                    hl_cur_token!("Ternary");
                    let ter_val_node = unsafe { new_node(ExprASTNodeType::TernaryValue) };
                    unsafe {
                        (*ter_val_node).start = cur_token.start;
                        (*ter_val_node).len = cur_token.len;
                        (*ter_val_node).data.ter = AstNodeDataTer { got_colon: false };
                    }
                    assert!(!unsafe { (*cur_node).children }.is_null());
                    assert!(unsafe { (*(*cur_node).children).next }.is_null());
                    assert_eq!(unsafe { *ast_stack.last().unwrap() }, unsafe {
                        &raw mut (*(*cur_node).children).next
                    });
                    unsafe { **ast_stack.last().unwrap() = ter_val_node };
                    ast_stack.push(unsafe { &raw mut (*ter_val_node).children });
                }

                LexExprTokenType::SingleQuotedString | LexExprTokenType::DoubleQuotedString => {
                    let is_double = tok_type == LexExprTokenType::DoubleQuotedString;
                    if !unsafe { cur_token.data.str_.closed } {
                        if is_double {
                            error_from_token_and_msg!("E114: Missing double quote: %.*s");
                        } else {
                            error_from_token_and_msg!("E115: Missing single quote: %.*s");
                        }
                    }
                    if want_node == WantedNode::Operator {
                        op_missing!();
                    }
                    cur_node = new_node_with_cur_pos!(if is_double {
                        ExprASTNodeType::DoubleQuotedString
                    } else {
                        ExprASTNodeType::SingleQuotedString
                    });
                    unsafe { *top_node_p = cur_node };
                    unsafe { parse_quoted_string(pstate, cur_node, cur_token, is_invalid) };
                    want_node = WantedNode::Operator;
                }

                LexExprTokenType::Assignment => {
                    if cur_pt == ParseType::Assignment {
                        pt_stack.pop();
                    } else if cur_pt == ParseType::SingleAssignment {
                        pt_stack.pop();
                        pt_stack.pop();
                        error_from_token_and_msg!(
                            "E475: Expected closing bracket to end list assignment lvalue: %.*s"
                        );
                    } else {
                        error_from_token_and_msg!("E15: Misplaced assignment: %.*s");
                    }
                    assert!(!pt_stack.is_empty());
                    assert!(*pt_stack.last().unwrap() == ParseType::Expr);
                    add_value_if_missing!("E15: Unexpected assignment: %.*s");
                    cur_node = new_node_with_cur_pos!(ExprASTNodeType::Assignment);
                    unsafe {
                        (*cur_node).data.ass = AstNodeDataAss {
                            typ: cur_token.data.ass.typ,
                        }
                    };
                    match unsafe { cur_token.data.ass.typ } {
                        ExprAssignmentType::Plain => hl_cur_token!("PlainAssignment"),
                        ExprAssignmentType::Add => hl_cur_token!("AssignmentWithAddition"),
                        ExprAssignmentType::Subtract => hl_cur_token!("AssignmentWithSubtraction"),
                        ExprAssignmentType::Concat => hl_cur_token!("AssignmentWithConcatenation"),
                    }
                    add_op_node!(cur_node);
                }
            }

            // viml_pexpr_parse_cycle_end:
            prev_token = cur_token;
            highlighted_prev_spacing = false;
            unsafe { nvim_viml_parser_advance(pstate, cur_token.len) };
            break 'process_token; // Don't re-loop; proceed to next main_loop iteration.
        } // end 'process_token
    } // end 'main_loop

    // viml_pexpr_parse_end:
    assert!(!pt_stack.is_empty());
    assert!(!ast_stack.is_empty());

    if want_node == WantedNode::Value && *pt_stack.last().unwrap() != ParseType::LambdaArguments {
        let pos = unsafe { nvim_parser_get_pos(pstate.cast_const()) };
        unsafe {
            east_set_error(
                pstate,
                &mut ast.err,
                gettext_msg!("E15: Expected value, got EOC: %.*s"),
                pos,
            );
        }
    } else if ast_stack.len() != 1 {
        assert!(!ast_stack.is_empty());
        ast_stack.pop(); // Pop topmost finished value.
        while ast.err.msg.is_null() && !ast_stack.is_empty() {
            let cur_node = unsafe { *ast_stack.pop().unwrap() };
            assert!(!cur_node.is_null());
            match unsafe { (*cur_node).typ } {
                ExprASTNodeType::OpMissing | ExprASTNodeType::Missing => {}
                ExprASTNodeType::Call => unsafe {
                    east_set_error(
                        pstate,
                        &mut ast.err,
                        gettext_msg!("E116: Missing closing parenthesis for function call: %.*s"),
                        (*cur_node).start,
                    );
                },
                ExprASTNodeType::Nested => unsafe {
                    east_set_error(
                        pstate,
                        &mut ast.err,
                        gettext_msg!(
                            "E110: Missing closing parenthesis for nested expression: %.*s"
                        ),
                        (*cur_node).start,
                    );
                },
                ExprASTNodeType::ListLiteral => unsafe {
                    east_set_error(
                        pstate,
                        &mut ast.err,
                        gettext_msg!("E697: Missing end of List ']': %.*s"),
                        (*cur_node).start,
                    );
                },
                ExprASTNodeType::DictLiteral => unsafe {
                    east_set_error(
                        pstate,
                        &mut ast.err,
                        gettext_msg!("E723: Missing end of Dictionary '}': %.*s"),
                        (*cur_node).start,
                    );
                },
                ExprASTNodeType::UnknownFigure => unsafe {
                    east_set_error(
                        pstate,
                        &mut ast.err,
                        gettext_msg!("E15: Missing closing figure brace: %.*s"),
                        (*cur_node).start,
                    );
                },
                ExprASTNodeType::Lambda => unsafe {
                    east_set_error(
                        pstate,
                        &mut ast.err,
                        gettext_msg!("E15: Missing closing figure brace for lambda: %.*s"),
                        (*cur_node).start,
                    );
                },
                ExprASTNodeType::CurlyBracesIdentifier => {
                    // Should not appear in stack like this.
                    panic!("viml_pexpr_parse: CurlyBracesIdentifier in stack at end");
                }
                ExprASTNodeType::Integer
                | ExprASTNodeType::Float
                | ExprASTNodeType::SingleQuotedString
                | ExprASTNodeType::DoubleQuotedString
                | ExprASTNodeType::Option
                | ExprASTNodeType::Environment
                | ExprASTNodeType::Register
                | ExprASTNodeType::PlainIdentifier
                | ExprASTNodeType::PlainKey => {
                    // Plain values can't be in non-topmost stack position.
                    panic!("viml_pexpr_parse: plain value in stack at end");
                }
                ExprASTNodeType::Comma | ExprASTNodeType::Colon | ExprASTNodeType::Arrow => {}
                ExprASTNodeType::Subscript
                | ExprASTNodeType::ConcatOrSubscript
                | ExprASTNodeType::ComplexIdentifier
                | ExprASTNodeType::Assignment
                | ExprASTNodeType::Mod
                | ExprASTNodeType::Division
                | ExprASTNodeType::Multiplication
                | ExprASTNodeType::Not
                | ExprASTNodeType::And
                | ExprASTNodeType::Or
                | ExprASTNodeType::Concat
                | ExprASTNodeType::Comparison
                | ExprASTNodeType::UnaryMinus
                | ExprASTNodeType::UnaryPlus
                | ExprASTNodeType::BinaryMinus
                | ExprASTNodeType::Ternary
                | ExprASTNodeType::BinaryPlus => {}
                ExprASTNodeType::TernaryValue => {
                    if !unsafe { (*cur_node).data.ter.got_colon } {
                        unsafe {
                            east_set_error(
                                pstate,
                                &mut ast.err,
                                gettext_msg!("E109: Missing ':' after '?': %.*s"),
                                (*cur_node).start,
                            );
                        }
                    }
                }
            }
        }
    }

    ast
}
