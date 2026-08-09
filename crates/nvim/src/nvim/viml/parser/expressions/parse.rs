//! The state machine behind `viml_pexpr_parse`.
//!
//! One token at a time is pulled from the lexer and handed to a handler
//! picked by its class; the handlers live in the sibling modules
//! (`operators`, `values`, `brackets`, `figure`) and speak to the parse
//! through [`ExprParser`], which owns everything the loop threads between
//! them, and [`Flow`], which is how a handler tells the loop what to do next.
//!
//! The AST itself deliberately stays *outside* `ExprParser`: the bottom of
//! the AST stack points at `ExprAST::root`, and a struct holding a pointer
//! into itself cannot be passed around as `&mut` without invalidating it.

use super::*;
use super::{brackets, figure, operators, values};
use core::ffi::{CStr, c_char, c_int};

/// Additional flags to pass to the lexer, indexed by the wanted node.
static want_node_to_lexer_flags: [c_int; 2] = [
    kELFlagForbidScope as c_int, // kENodeOperator
    kELFlagIsNotCmp as c_int,    // kENodeValue
];

/// Determine whether the given parse type is an assignment.
#[inline(always)]
pub(super) fn pt_is_assignment(pt: ExprASTParseType) -> bool {
    pt == kEPTAssignment || pt == kEPTSingleAssignment
}

/// The highlight group for the current token: `Nvim<group>`, or
/// `NvimInvalid<group>` once anything about the token has been rejected.
///
/// The C spelled this `HL()`, and like it this reads `is_invalid` where it is
/// written — handlers flip that flag mid-token and expect later highlights to
/// follow.
macro_rules! hl {
    ($p:expr, $group:ident) => {
        (if $p.is_invalid {
            concat!("NvimInvalid", stringify!($group), "\0")
        } else {
            concat!("Nvim", stringify!($group), "\0")
        })
        .as_ptr()
        .cast::<::core::ffi::c_char>()
    };
}
pub(super) use hl;

/// What a token handler wants the driver to do next.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Flow {
    /// The token is consumed: record it as the previous one and advance the
    /// reader past it.
    NextToken,
    /// Run the token through the dispatcher again. Something the dispatch
    /// depends on changed — the token's own type (an invalid token reports
    /// what it was meant to be), or the wanted node after an operator was
    /// spliced in ahead of it.
    Reprocess,
    /// Stop parsing and return what has been built. Deliberately *without*
    /// consuming the token: with `kExprFlagsMulti` the caller resumes here
    /// with a second expression.
    Stop,
}

/// Everything `viml_pexpr_parse`'s token loop threads between its stages.
///
/// The first block lives for the whole parse; the second is refreshed for
/// each token before the handlers see it.
pub(super) struct ExprParser {
    /// Reader and highlight state, owned by the caller. Stays a raw pointer:
    /// it embeds kvecs whose `items` point back inside it, which a `&mut`
    /// retag would invalidate.
    pub(super) pstate: *mut ParserState,
    /// The AST being built, owned by `viml_pexpr_parse`'s frame. `ast_stack`
    /// holds a pointer to its `root` field, so it cannot live in here.
    pub(super) ast: *mut ExprAST,
    pub(super) flags: c_int,

    /// The current branch of the AST:
    ///
    /// - item 0 holds the root of the tree, i.e. `&ast.root`;
    /// - item i points to the previous item's last child.
    ///
    /// While the parser wants a value the last item points at NULL; otherwise
    /// it holds the last *finished* value, e.g. `1` or `+(1, 1)`.
    pub(super) ast_stack: Vec<*mut *mut ExprASTNode>,
    /// What is being parsed: a plain expression, an assignment lvalue, or a
    /// lambda's argument list.
    pub(super) pt_stack: Vec<ExprASTParseType>,
    pub(super) want_node: ExprASTWantedNode,
    pub(super) prev_token: LexExprToken,
    pub(super) highlighted_prev_spacing: bool,
    /// The figure brace node currently being read as a lambda's argument
    /// list; NULL at any other time.
    pub(super) lambda_node: *mut ExprASTNode,
    /// Stack depth at which the assignment lvalue started, so that closing
    /// its last bracket can pop the assignment parse type again.
    pub(super) asgn_level: size_t,

    /// The token being processed.
    pub(super) cur_token: LexExprToken,
    /// Its class. Not always `cur_token.type_0`: an invalid token is
    /// re-dispatched as whatever it was trying to be.
    pub(super) tok_type: LexExprTokenType,
    /// Whether anything about this token has been rejected. Drives the choice
    /// of highlight group, see [`hl!`].
    pub(super) is_invalid: bool,
    /// Lexer flags derived from the parse state, fixed for this token even
    /// when it is dispatched more than once.
    pub(super) lexer_flags: c_int,
    /// Whether the enclosing node is an as-yet-undecided `d.key`.
    pub(super) is_concat_or_subscript: bool,
    /// The line the token was read from.
    pub(super) pline: ParserLine,
    /// The slot the token's value node goes into: the top of the AST stack.
    pub(super) top_node_p: *mut *mut ExprASTNode,
    /// Whether this token is a dictionary key rather than a value.
    pub(super) node_is_key: bool,
    /// The parse type in force for this token.
    pub(super) cur_pt: ExprASTParseType,
}

impl ExprParser {
    fn new(pstate: *mut ParserState, ast: *mut ExprAST, flags: c_int) -> Self {
        let mut pt_stack = Vec::new();
        pt_stack.push(kEPTExpr);
        if flags & kExprFlagsParseLet as c_int != 0 {
            pt_stack.push(kEPTAssignment);
        }
        ExprParser {
            pstate,
            ast,
            flags,
            ast_stack: Vec::new(),
            pt_stack,
            want_node: kENodeValue,
            prev_token: LexExprToken {
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
            },
            highlighted_prev_spacing: false,
            lambda_node: ::core::ptr::null_mut::<ExprASTNode>(),
            asgn_level: 0,
            cur_token: LexExprToken {
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
            },
            tok_type: kExprLexMissing,
            is_invalid: false,
            lexer_flags: 0,
            is_concat_or_subscript: false,
            pline: ParserLine {
                data: ::core::ptr::null::<c_char>(),
                size: 0,
                allocated: false,
            },
            top_node_p: ::core::ptr::null_mut::<*mut ExprASTNode>(),
            node_is_key: false,
            cur_pt: kEPTExpr,
        }
    }

    /// `kv_last(pt_stack)`.
    pub(super) fn pt_top(&self) -> ExprASTParseType {
        self.pt_stack[self.pt_stack.len() - 1]
    }

    /// `MAY_HAVE_NEXT_EXPR`: whether another expression could follow this one.
    /// `:echo @a @a` is valid; `:echo (@a @a)` is not.
    pub(super) fn may_have_next_expr(&self) -> bool {
        self.ast_stack.len() == 1
    }

    /// Peek at the next token with the flags this parse state calls for.
    unsafe fn next_token(&mut self) -> LexExprToken {
        viml_pexpr_next_token(
            self.pstate,
            want_node_to_lexer_flags[self.want_node as usize] | self.lexer_flags,
        )
    }

    /// `HL_CUR_TOKEN`: highlight the whole current token.
    pub(super) unsafe fn hl_token(&self, group: *const c_char) {
        viml_parser_highlight(self.pstate, self.cur_token.start, self.cur_token.len, group);
    }

    /// Highlight a slice of the current token.
    pub(super) unsafe fn hl_at(&self, pos: ParserPosition, len: size_t, group: *const c_char) {
        viml_parser_highlight(self.pstate, pos, len, group);
    }

    /// `NEW_NODE_WITH_CUR_POS`: allocate a node spanning the current token,
    /// and the spacing before it if there was any.
    pub(super) unsafe fn new_node(&self, type_0: ExprASTNodeType) -> *mut ExprASTNode {
        let node = viml_pexpr_new_node(type_0);
        (*node).start = self.cur_token.start;
        (*node).len = self.cur_token.len;
        if self.prev_token.type_0 == kExprLexSpacing {
            (*node).start = self.prev_token.start;
            (*node).len = (*node).len.wrapping_add(self.prev_token.len);
        }
        node
    }

    /// `ERROR_FROM_TOKEN_AND_MSG`: reject the token and record `msg` as the
    /// parse error, unless an earlier error already stands.
    pub(super) unsafe fn error(&mut self, msg: &CStr) {
        self.error_at(gettext(msg.as_ptr()), self.cur_token.start);
    }

    /// `ERROR_FROM_TOKEN` / `ERROR_FROM_NODE_AND_MSG`: as [`Self::error`], for
    /// an already-translated message reported at an explicit position.
    pub(super) unsafe fn error_at(&mut self, msg: *const c_char, at: ParserPosition) {
        self.is_invalid = true;
        east_set_error(self.pstate, &raw mut (*self.ast).err, msg, at);
    }

    /// `ADD_OP_NODE`: hand an operator node to the shunting yard.
    pub(super) unsafe fn add_op_node(&mut self, node: *mut ExprASTNode) {
        self.is_invalid = self.is_invalid
            | !viml_pexpr_handle_bop(
                self.pstate,
                &mut self.ast_stack,
                node,
                &raw mut self.want_node,
                &raw mut (*self.ast).err,
            );
    }

    /// `ADD_VALUE_IF_MISSING`: stand a Missing node in for the value an
    /// operator was expecting, as in `* 5`.
    pub(super) unsafe fn add_value_if_missing(&mut self, msg: &CStr) {
        if self.want_node == kENodeValue {
            self.error(msg);
            let node = self.new_node(kExprNodeMissing);
            (*node).len = 0;
            *self.top_node_p = node;
            self.want_node = kENodeOperator;
        }
    }

    /// `OP_MISSING`: two values in a row, as in `:echo @a @a`.
    ///
    /// With `kExprFlagsMulti` and nothing but the root on the stack the caller
    /// gets to start a second expression at this token; otherwise an OpMissing
    /// operator is spliced in and the token is dispatched again, this time in
    /// value position.
    pub(super) unsafe fn op_missing(&mut self) -> Flow {
        if self.flags & kExprFlagsMulti as c_int != 0 && self.may_have_next_expr() {
            return Flow::Stop;
        }
        debug_assert!(!(*self.top_node_p).is_null(), "*top_node_p != NULL");
        self.error(c"E15: Missing operator: %.*s");
        let node = self.new_node(kExprNodeOpMissing);
        (*node).len = 0;
        self.add_op_node(node);
        Flow::Reprocess
    }

    /// `SELECT_FIGURE_BRACE_TYPE`: commit a figure brace node to a type now
    /// that it is known, and recolour its opening brace to match.
    ///
    /// The recolouring goes through `highlight_vec`, never the collection's
    /// own `items`: that pointer is stale while the log is still inline.
    pub(super) unsafe fn select_figure_brace_type(
        &mut self,
        node: *mut ExprASTNode,
        new_type: ExprASTNodeType,
        group: *const c_char,
    ) {
        assert!(
            (*node).type_0 == kExprNodeUnknownFigure || (*node).type_0 == new_type,
            "the node is still an unknown figure brace, or already the new type"
        );
        (*node).type_0 = new_type;
        if !(*self.pstate).colors.is_null() {
            highlight_vec(&mut *(*self.pstate).colors).as_mut_slice()
                [(*node).data.fig.opening_hl_idx]
                .group = group;
        }
    }

    /// `ADD_IDENT`'s prologue: open a complex identifier — `a{b}c` and
    /// friends — around the value already on the stack, and answer the slot
    /// the caller's new identifier node goes into.
    ///
    /// `None` means this cannot be a part of a complex identifier after all,
    /// and the caller must report a missing operator: either there is spacing
    /// before it, or what precedes it is not an identifier.
    pub(super) unsafe fn open_complex_identifier(&mut self) -> Option<*mut *mut ExprASTNode> {
        debug_assert!(
            self.want_node == kENodeOperator,
            "want_node == kENodeOperator"
        );
        if self.prev_token.type_0 == kExprLexSpacing {
            return None;
        }
        match (**self.top_node_p).type_0 {
            // TODO(ZyX-I): Extend syntax to allow ${expr}. This is needed to
            // handle environment variables like those bash uses for
            // `export -f`: their names consist not only of alphanumeric
            // characters.
            kExprNodeComplexIdentifier
            | kExprNodePlainIdentifier
            | kExprNodeCurlyBracesIdentifier => {}
            _ => return None,
        }
        let node = self.new_node(kExprNodeComplexIdentifier);
        (*node).len = 0;
        (*node).children = *self.top_node_p;
        *self.top_node_p = node;
        self.ast_stack.push(&raw mut (*(*node).children).next);
        let slot = stack_top(&self.ast_stack, 0);
        debug_assert!((*slot).is_null(), "*new_top_node_p == NULL");
        Some(slot)
    }

    /// The whole parse: one iteration of the loop per token.
    unsafe fn run(&mut self) {
        loop {
            self.is_concat_or_subscript = self.want_node == kENodeValue
                && self.ast_stack.len() > 1
                && (**stack_top(&self.ast_stack, 1)).type_0 == kExprNodeConcatOrSubscript;
            self.lexer_flags = kELFlagPeek as c_int
                | (if self.flags & kExprFlagsDisallowEOC as c_int != 0 {
                    kELFlagForbidEOC as c_int
                } else {
                    0
                })
                | (if self.want_node == kENodeValue
                    && (self.ast_stack.len() == 1
                        || (**stack_top(&self.ast_stack, 1)).type_0 != kExprNodeConcat
                            && (**stack_top(&self.ast_stack, 1)).type_0
                                != kExprNodeConcatOrSubscript)
                {
                    kELFlagAllowFloat as c_int
                } else {
                    0
                });
            self.cur_token = self.next_token();
            if self.cur_token.type_0 == kExprLexEOC {
                break;
            }
            self.tok_type = self.cur_token.type_0;
            self.is_invalid = self.tok_type == kExprLexInvalid;
            let flow = loop {
                match self.process_token() {
                    Flow::Reprocess => {}
                    flow => break flow,
                }
            };
            if flow == Flow::Stop {
                break;
            }
            self.prev_token = self.cur_token;
            self.highlighted_prev_spacing = false;
            viml_parser_advance(
                &mut (*self.pstate).pos,
                &mut (*self.pstate).reader,
                self.cur_token.len,
            );
        }
        self.finish();
    }

    /// Refresh the per-token state and hand the token to its class handler.
    unsafe fn process_token(&mut self) -> Flow {
        // May use different flags this time.
        self.cur_token = self.next_token();
        if self.tok_type == kExprLexSpacing {
            if self.is_invalid {
                self.hl_token(hl!(self, Spacing));
            }
            // Otherwise do not do anything: let regular spacing be highlighted
            // as normal. This also allows later to highlight spacing as
            // invalid.
            return Flow::NextToken;
        } else if self.is_invalid
            && self.prev_token.type_0 == kExprLexSpacing
            && !self.highlighted_prev_spacing
        {
            self.hl_at(
                self.prev_token.start,
                self.prev_token.len,
                hl!(self, Spacing),
            );
            self.is_invalid = false;
            self.highlighted_prev_spacing = true;
        }
        self.pline = *(*self.pstate)
            .reader
            .lines
            .items
            .add(self.cur_token.start.line);
        self.top_node_p = stack_top(&self.ast_stack, 0);
        debug_assert!(!self.ast_stack.is_empty(), "kv_size(ast_stack) >= 1");
        self.check_stack_invariants();

        // Note: in Vim whether expression "cond?d.a:2" is valid depends both
        // on "cond" and whether "d" is a dictionary: the expression is valid
        // if the condition is true and "d" is a dictionary. This parser does
        // not allow such ambiguity, especially because it simply can't:
        // whether "d" is a dictionary is not known at parsing time.
        //
        // Here the example will always contain a concat with "a:2" sucking the
        // colon, making the expression invalid both because there is no longer
        // a spare colon for the ternary and because concatenating a dictionary
        // with anything is not valid.
        self.node_is_key = self.is_concat_or_subscript
            && (if self.cur_token.type_0 == kExprLexPlainIdentifier {
                !self.cur_token.data.var.autoload
                    && self.cur_token.data.var.scope == kExprVarScopeMissing
            } else {
                self.cur_token.type_0 == kExprLexNumber
            })
            && self.prev_token.type_0 != kExprLexSpacing;
        if self.is_concat_or_subscript && !self.node_is_key {
            // Note: in Vim "d. a" (this is the reason behind the
            // `prev_token.type != kExprLexSpacing` part of the condition) as
            // well as any other "d.{expr}" where "{expr}" does not look like a
            // key is invalid whenever "d" happens to be a dictionary. Since the
            // parser has no idea whether the preceding expression is actually a
            // dictionary it can't outright reject anything, so it turns
            // kExprNodeConcatOrSubscript into kExprNodeConcat instead.
            (**stack_top(&self.ast_stack, 1)).type_0 = kExprNodeConcat;
        }
        if let Some(flow) = self.reconcile_parse_type() {
            return flow;
        }
        debug_assert!(!self.pt_stack.is_empty(), "kv_size(pt_stack)");
        self.cur_pt = self.pt_top();
        debug_assert!(
            self.lambda_node.is_null() || self.cur_pt == kEPTLambdaArguments,
            "lambda_node == NULL || cur_pt == kEPTLambdaArguments"
        );
        self.dispatch()
    }

    /// The stack invariants the C checked under `#ifndef NDEBUG`: item 0 is
    /// the root slot, and item i + 1 points at item i's *last* child.
    unsafe fn check_stack_invariants(&self) {
        let want_value = self.want_node == kENodeValue;
        debug_assert!(
            want_value == (*self.top_node_p).is_null(),
            "want_value == (*top_node_p == NULL)"
        );
        debug_assert!(
            self.ast_stack[0] == &raw mut (*self.ast).root,
            "kv_A(ast_stack, 0) == &ast.root"
        );
        for i in 0..self.ast_stack.len().saturating_sub(1) {
            let item_null = want_value && i + 2 == self.ast_stack.len();
            let node = *self.ast_stack[i];
            let next = self.ast_stack[i + 1];
            assert!(
                &raw mut (*node).children == next
                    && (if item_null {
                        (*node).children.is_null()
                    } else {
                        (*(*node).children).next.is_null()
                    })
                    || &raw mut (*(*node).children).next == next
                        && (if item_null {
                            (*(*node).children).next.is_null()
                        } else {
                            (*(*(*node).children).next).next.is_null()
                        }),
                "(&(*kv_A(ast_stack, i))->children == kv_A(ast_stack, i + 1) && (item_null ? (*kv_A(ast_stack, i))->children == NULL : (*kv_A(ast_stack, i))->children->next == NULL)) || ((&(*kv_A(ast_stack, i))->children->next == kv_A(ast_stack, i + 1)) && (item_null ? (*kv_A(ast_stack, i))->children->next == NULL : (*kv_A(ast_stack, i))->children->next->next == NULL))"
            );
        }
    }

    /// Pop parse type stack items that this token proves wrong: an
    /// as-yet-undecided figure brace that cannot be a lambda after all, or an
    /// assignment lvalue that this token cannot be part of.
    unsafe fn reconcile_parse_type(&mut self) -> Option<Flow> {
        let is_single_assignment = self.pt_top() == kEPTSingleAssignment;
        match self.pt_top() {
            kEPTLambdaArguments => {
                if self.want_node == kENodeOperator
                    && self.tok_type != kExprLexComma
                    && self.tok_type != kExprLexArrow
                    || self.want_node == kENodeValue
                        && !(self.cur_token.type_0 == kExprLexPlainIdentifier
                            && self.cur_token.data.var.scope == kExprVarScopeMissing
                            && !self.cur_token.data.var.autoload)
                        && self.tok_type != kExprLexArrow
                {
                    (*self.lambda_node).data.fig.type_guesses.allow_lambda = false;
                    if !(*self.lambda_node).children.is_null()
                        && (*(*self.lambda_node).children).type_0 == kExprNodeComma
                    {
                        // A comma child means the parser has already seen at
                        // least "{arg1,", so the node cannot possibly be
                        // anything but a lambda.
                        //
                        // Vim may give E121 or E720 here, but neither looks
                        // right: both are results of reevaluating a
                        // possibly-lambda node as a dictionary, and that is not
                        // going to happen.
                        self.error(c"E15: Expected lambda arguments list or arrow: %.*s");
                    } else {
                        // Else it may appear that the possibly-lambda node is
                        // actually a dictionary or a curly-braces-name
                        // identifier.
                        self.lambda_node = ::core::ptr::null_mut::<ExprASTNode>();
                        self.pt_stack.truncate(self.pt_stack.len() - 1);
                    }
                }
            }
            kEPTSingleAssignment | kEPTAssignment => {
                if self.want_node == kENodeValue
                    && self.tok_type != kExprLexBracket
                    && self.tok_type != kExprLexPlainIdentifier
                    && (self.tok_type != kExprLexFigureBrace || self.cur_token.data.brc.closing)
                    && !(self.node_is_key && self.tok_type == kExprLexNumber)
                    && self.tok_type != kExprLexEnv
                    && self.tok_type != kExprLexOption
                    && self.tok_type != kExprLexRegister
                {
                    self.error(c"E15: Expected value part of assignment lvalue: %.*s");
                    self.pt_stack.truncate(self.pt_stack.len() - 1);
                } else if self.want_node == kENodeOperator
                    && self.tok_type != kExprLexBracket
                    && (self.tok_type != kExprLexFigureBrace || self.cur_token.data.brc.closing)
                    && self.tok_type != kExprLexDot
                    && (self.tok_type != kExprLexComma || !is_single_assignment)
                    && self.tok_type != kExprLexAssignment
                    // Curly brace identifiers: these contain a plain identifier
                    // or another curly brace where an operator is wanted.
                    && !((self.tok_type == kExprLexPlainIdentifier
                        || self.tok_type == kExprLexFigureBrace
                            && !self.cur_token.data.brc.closing)
                        && self.prev_token.type_0 != kExprLexSpacing)
                {
                    if self.flags & kExprFlagsMulti as c_int != 0 && self.may_have_next_expr() {
                        return Some(Flow::Stop);
                    }
                    self.error(c"E15: Expected assignment operator or subscript: %.*s");
                    self.pt_stack.truncate(self.pt_stack.len() - 1);
                }
                debug_assert!(!self.pt_stack.is_empty(), "kv_size(pt_stack)");
            }
            _ => {}
        }
        None
    }

    /// Hand the token to the handler for its class.
    unsafe fn dispatch(&mut self) -> Flow {
        match self.tok_type {
            kExprLexMissing | kExprLexSpacing | kExprLexEOC => abort(),
            kExprLexInvalid => {
                self.error_at(self.cur_token.data.err.msg, self.cur_token.start);
                // Dispatch it again as whatever it was trying to be.
                self.tok_type = self.cur_token.data.err.type_0;
                Flow::Reprocess
            }
            kExprLexRegister => values::register(self),
            kExprLexOption => values::option(self),
            kExprLexEnv => values::environment(self),
            kExprLexNumber => values::number(self),
            kExprLexPlainIdentifier => values::plain_identifier(self),
            kExprLexDoubleQuotedString | kExprLexSingleQuotedString => values::quoted_string(self),
            kExprLexPlus => operators::plus(self),
            kExprLexMinus => operators::minus(self),
            kExprLexOr => operators::or(self),
            kExprLexAnd => operators::and(self),
            kExprLexMultiplication => operators::multiplication(self),
            kExprLexNot => operators::not(self),
            kExprLexComparison => operators::comparison(self),
            kExprLexDot => operators::dot(self),
            kExprLexQuestion => operators::question(self),
            kExprLexArrow => operators::arrow(self),
            kExprLexAssignment => operators::assignment(self),
            kExprLexComma => brackets::comma(self),
            kExprLexColon => brackets::colon(self),
            kExprLexBracket => brackets::bracket(self),
            kExprLexParenthesis => brackets::parenthesis(self),
            kExprLexFigureBrace => figure::figure_brace(self),
            _ => Flow::NextToken,
        }
    }

    /// End of the expression: report whatever the stack was still waiting for.
    unsafe fn finish(&mut self) {
        debug_assert!(!self.pt_stack.is_empty(), "kv_size(pt_stack)");
        debug_assert!(!self.ast_stack.is_empty(), "kv_size(ast_stack)");
        // kEPTLambdaArguments is blacklisted because its presence means a
        // better error message comes out of the other branch.
        if self.want_node == kENodeValue && self.pt_top() != kEPTLambdaArguments {
            self.error_at(
                gettext(c"E15: Expected value, got EOC: %.*s".as_ptr()),
                (*self.pstate).pos,
            );
            return;
        }
        if self.ast_stack.len() == 1 {
            return;
        }
        // Something may be wrong, check whether it really is. The pointer to
        // ast.root must never be dropped, so "!= 1" is the same as "> 1".
        //
        // The topmost item is a *finished* value — it may hold an already
        // finished nested expression — so it must not be analyzed.
        self.ast_stack.truncate(self.ast_stack.len() - 1);
        while (*self.ast).err.msg.is_null() && !self.ast_stack.is_empty() {
            let node: *const ExprASTNode = *self.ast_stack.pop().expect("the stack is not empty");
            // This should only happen when want_node == kENodeValue.
            debug_assert!(!node.is_null(), "cur_node != NULL");
            // TODO(ZyX-I): Rehighlight as invalid?
            let msg: &CStr = match (*node).type_0 {
                // The error should've been already reported.
                kExprNodeOpMissing | kExprNodeMissing => continue,
                kExprNodeCall => c"E116: Missing closing parenthesis for function call: %.*s",
                kExprNodeNested => c"E110: Missing closing parenthesis for nested expression: %.*s",
                // For whatever reason "[1" yields "E696: Missing comma in
                // list" in Vim while "[1," yields E697.
                kExprNodeListLiteral => c"E697: Missing end of List ']': %.*s",
                // The same problem as with the list literal, E722 (missing
                // comma) vs E723, but additionally just "{" yields only E15.
                kExprNodeDictLiteral => c"E723: Missing end of Dictionary '}': %.*s",
                kExprNodeUnknownFigure => c"E15: Missing closing figure brace: %.*s",
                kExprNodeLambda => c"E15: Missing closing figure brace for lambda: %.*s",
                // Upstream `abort()`s here, on the premise that until the
                // trailing "}" a curly braces identifier cannot be told from a
                // Dict and so can never be left unfinished on the stack. The
                // premise is false: a `{` in *operator* position is a curly
                // braces name from the moment it is lexed (see
                // `figure::figure_brace`'s else arm), so any unterminated one
                // reaches this loop. `nvim_parse_expression('a{b')` — no flags
                // — killed the process. It is an unclosed figure brace like any
                // other; say so.
                kExprNodeCurlyBracesIdentifier => c"E15: Missing closing figure brace: %.*s",
                // These are plain values and not containers; they can only
                // show up in the topmost stack element, which was
                // unconditionally popped above.
                kExprNodeInteger
                | kExprNodeFloat
                | kExprNodeSingleQuotedString
                | kExprNodeDoubleQuotedString
                | kExprNodeOption
                | kExprNodeEnvironment
                | kExprNodeRegister
                | kExprNodePlainIdentifier
                | kExprNodePlainKey => abort(),
                // Actually Vim throws E109 in more cases.
                kExprNodeTernaryValue if !(*node).data.ter.got_colon => {
                    c"E109: Missing ':' after '?': %.*s"
                }
                // Everything else is either only valid inside something that
                // has to be closed — and so is caught later — or is fine to
                // see in the stack.
                _ => continue,
            };
            self.error_at(gettext(msg.as_ptr()), (*node).start);
        }
    }
}

/// Parse one Vimscript expression.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn viml_pexpr_parse(pstate: *mut ParserState, flags: c_int) -> ExprAST {
    let mut ast = ExprAST {
        err: ExprASTError {
            msg: ::core::ptr::null::<c_char>(),
            arg: ::core::ptr::null::<c_char>(),
            arg_len: 0,
        },
        root: ::core::ptr::null_mut::<ExprASTNode>(),
    };
    let mut parser = ExprParser::new(pstate, &raw mut ast, flags);
    parser.ast_stack.push(&raw mut ast.root);
    parser.run();
    ast
}
