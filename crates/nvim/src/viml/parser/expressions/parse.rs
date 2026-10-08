//! The state machine behind `viml_pexpr_parse`.
//!
//! One token at a time is pulled from the lexer and handed to a handler
//! picked by its class; the handlers live in the sibling modules
//! (`operators`, `values`, `brackets`, `figure`) and speak to the parse
//! through [`ExprParser`], which owns everything the loop threads between
//! them, and [`Flow`], which is how a handler tells the loop what to do next.
//!
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use core::ffi::{CStr, c_int};

use super::ast::{east_set_error, translate};
use super::{brackets, figure, operators, values, *};

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
        if $p.is_invalid {
            const {
                match ::core::ffi::CStr::from_bytes_with_nul(
                    concat!("NvimInvalid", stringify!($group), "\0").as_bytes(),
                ) {
                    Ok(group) => group,
                    Err(_) => panic!("a group name holds no NUL"),
                }
            }
        } else {
            const {
                match ::core::ffi::CStr::from_bytes_with_nul(
                    concat!("Nvim", stringify!($group), "\0").as_bytes(),
                ) {
                    Ok(group) => group,
                    Err(_) => panic!("a group name holds no NUL"),
                }
            }
        }
    };
}
pub(super) use hl;

/// The payload of a token, read back as the arm its type selects.
///
/// The parser does ask for the *wrong* arm in two places, and both are
/// deliberate: an invalid option token is asked for its scope and an invalid
/// comparison for its case strategy, over a payload the lexer wrote as an
/// error. `values::option` and `operators::comparison` each have a
/// `kExprLexInvalid` arm that then reads on regardless, and the answer
/// reaches the highlight list. The C reads a union member the error did not
/// cover, which is the zeroes `blank_token` left -- except for the option
/// scope, which an error *does* leave standing and
/// [`LexExprTokenError::opt_scope`] therefore carries. So a mismatched arm
/// answers its zero here, which is the same value.
impl LexExprToken {
    /// `+=`, `-=`, `.=` or plain `=`.
    pub(super) fn assignment_type(&self) -> ExprAssignmentType {
        match self.data {
            LexExprTokenData::Assignment(ass) => ass.type_0,
            _ => kExprAsgnPlain,
        }
    }

    /// A number literal's value and the base its prefix named.
    pub(super) fn number(&self) -> LexExprTokenNumber {
        match self.data {
            LexExprTokenData::Number(num) => num,
            _ => LexExprTokenNumber {
                val: LexExprTokenNumberValue::Integer(0),
                base: 0,
            },
        }
    }

    /// What an invalid token was trying to be, and why it is not.
    pub(super) fn error(&self) -> LexExprTokenError {
        match self.data {
            LexExprTokenData::Error(err) => err,
            _ => LexExprTokenError {
                type_0: kExprLexInvalid,
                msg: c"",
                opt_scope: kExprOptScopeUnspecified,
            },
        }
    }

    /// An identifier's scope and whether it is an autoload name.
    pub(super) fn variable(&self) -> LexExprTokenVar {
        match self.data {
            LexExprTokenData::Var(var) => var,
            _ => LexExprTokenVar {
                scope: kExprVarScopeMissing,
                autoload: false,
            },
        }
    }

    /// An option's name, its length and its scope.
    ///
    /// An *invalid* option token answers the scope its `&g:` prefix named,
    /// with no name: see the note above.
    pub(super) fn option(&self) -> LexExprTokenOption {
        let scope = match self.data {
            LexExprTokenData::Option(opt) => return opt,
            LexExprTokenData::Error(err) => err.opt_scope,
            _ => kExprOptScopeUnspecified,
        };
        LexExprTokenOption {
            name_offset: 0,
            len: 0,
            scope,
        }
    }

    /// Whether a string literal reached its closing quote.
    pub(super) fn string_is_closed(&self) -> bool {
        match self.data {
            LexExprTokenData::Str(str) => str.closed,
            _ => false,
        }
    }

    /// A register token's register name.
    pub(super) fn register_name(&self) -> ::core::ffi::c_int {
        match self.data {
            LexExprTokenData::Register(reg) => reg.name,
            _ => 0,
        }
    }

    /// Whether a bracket, brace or parenthesis closes rather than opens.
    pub(super) fn is_closing(&self) -> bool {
        match self.data {
            LexExprTokenData::Brace(brc) => brc.closing,
            _ => false,
        }
    }

    /// Which of `*`, `/` and `%` this is.
    pub(super) fn multiplication_type(&self) -> ExprLexMulType {
        match self.data {
            LexExprTokenData::Multiplication(mul) => mul.type_0,
            _ => kExprLexMulMul,
        }
    }

    /// A comparison's operator, case-comparison strategy and inversion.
    pub(super) fn comparison(&self) -> LexExprTokenComparison {
        match self.data {
            LexExprTokenData::Comparison(cmp) => cmp,
            _ => LexExprTokenComparison {
                type_0: kExprCmpEqual,
                ccs: kCCStrategyUseOption,
                inv: false,
            },
        }
    }
}

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
pub(super) struct ExprParser<'a, 's> {
    /// Reader and highlight state, owned by the caller.
    pub(super) pstate: &'s mut ParserState<'a>,
    /// The AST being built, handed back when the parse ends.
    pub(super) ast: ExprAST<'a>,
    pub(super) flags: c_int,

    /// The current branch of the AST:
    ///
    /// - item 0 is the root slot;
    /// - item i is the previous item's last child.
    ///
    /// While the parser wants a value the last item is empty; otherwise it
    /// holds the last *finished* value, e.g. `1` or `+(1, 1)`.
    pub(super) ast_stack: Vec<Slot>,
    /// What is being parsed: a plain expression, an assignment lvalue, or a
    /// lambda's argument list.
    pub(super) pt_stack: Vec<ExprASTParseType>,
    pub(super) want_node: ExprASTWantedNode,
    pub(super) prev_token: LexExprToken,
    pub(super) highlighted_prev_spacing: bool,
    /// The figure brace node currently being read as a lambda's argument
    /// list; `None` at any other time.
    pub(super) lambda_node: Option<NodeId>,
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
    pub(super) pline: &'a [u8],
    /// The slot the token's value node goes into: the top of the AST stack.
    pub(super) top_node_p: Slot,
    /// Whether this token is a dictionary key rather than a value.
    pub(super) node_is_key: bool,
    /// The parse type in force for this token.
    pub(super) cur_pt: ExprASTParseType,
}

impl<'a, 's> ExprParser<'a, 's> {
    fn new(pstate: &'s mut ParserState<'a>, flags: c_int) -> Self {
        let mut pt_stack = Vec::new();
        pt_stack.push(kEPTExpr);
        if flags & kExprFlagsParseLet as c_int != 0 {
            pt_stack.push(kEPTAssignment);
        }
        let blank = LexExprToken {
            start: ParserPosition { line: 0, col: 0 },
            len: 0,
            type_0: kExprLexMissing,
            data: LexExprTokenData::Blank,
        };
        ExprParser {
            pstate,
            ast: ExprAST::new(),
            flags,
            ast_stack: vec![Slot::Root],
            pt_stack,
            want_node: kENodeValue,
            prev_token: blank,
            highlighted_prev_spacing: false,
            lambda_node: None,
            asgn_level: 0,
            cur_token: blank,
            tok_type: kExprLexMissing,
            is_invalid: false,
            lexer_flags: 0,
            is_concat_or_subscript: false,
            pline: &[],
            top_node_p: Slot::Root,
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
    fn next_token(&mut self) -> LexExprToken {
        viml_pexpr_next_token(
            self.pstate,
            want_node_to_lexer_flags[self.want_node as usize] | self.lexer_flags,
        )
    }

    /// How many highlight chunks have been recorded so far; `None` when the
    /// caller asked for no highlighting.
    pub(super) fn highlight_count(&self) -> Option<size_t> {
        self.pstate.highlight_count()
    }

    /// Rewrite the highlight group of a chunk already recorded, as the guess
    /// at what a figure brace is narrows. A no-op without highlighting.
    pub(super) fn recolour(&mut self, index: size_t, group: &'static CStr) {
        self.pstate.recolour(index, group);
    }

    /// `len` bytes of the line the current token came from, from `col`. The
    /// lines are the caller's for the whole parse, so a node may hold on to
    /// this.
    pub(super) fn line_slice(&self, col: size_t, len: size_t) -> &'a [u8] {
        &self.pline[col..col + len]
    }

    /// The byte at `col` of the line the current token came from.
    pub(super) fn line_byte(&self, col: size_t) -> u8 {
        self.pline[col]
    }

    /// Decode the current token's string literal into `node`, which becomes
    /// its owner.
    pub(super) fn decode_quoted_string(&mut self, node: NodeId) {
        let literal = parse_quoted_string(self.pstate, self.cur_token, self.is_invalid);
        self.ast.set_data(node, ExprNodeData::Str(literal));
    }

    /// `HL_CUR_TOKEN`: highlight the whole current token.
    pub(super) fn hl_token(&mut self, group: &'static CStr) {
        self.hl_at(self.cur_token.start, self.cur_token.len, group);
    }

    /// Highlight a slice of the current token.
    pub(super) fn hl_at(&mut self, pos: ParserPosition, len: size_t, group: &'static CStr) {
        self.pstate.highlight(pos, len, group);
    }

    /// `NEW_NODE_WITH_CUR_POS`: allocate a node spanning the current token,
    /// and the spacing before it if there was any.
    pub(super) fn new_node(&mut self, type_0: ExprASTNodeType) -> NodeId {
        let node = self.ast.new_node(type_0);
        if self.prev_token.type_0 == kExprLexSpacing {
            let len = self.cur_token.len.wrapping_add(self.prev_token.len);
            self.ast.set_span(node, self.prev_token.start, len);
        } else {
            self.ast
                .set_span(node, self.cur_token.start, self.cur_token.len);
        }
        node
    }

    /// `ERROR_FROM_TOKEN_AND_MSG`: reject the token and record `msg` as the
    /// parse error, unless an earlier error already stands.
    pub(super) fn error(&mut self, msg: &'static CStr) {
        self.error_at(translate(msg), self.cur_token.start);
    }

    /// `ERROR_FROM_TOKEN` / `ERROR_FROM_NODE_AND_MSG`: as [`Self::error`], for
    /// an already-translated message reported at an explicit position.
    pub(super) fn error_at(&mut self, msg: &'static CStr, at: ParserPosition) {
        self.is_invalid = true;
        east_set_error(self.pstate, &mut self.ast, msg, at);
    }

    /// `ADD_OP_NODE`: hand an operator node to the shunting yard.
    pub(super) fn add_op_node(&mut self, node: NodeId) {
        self.is_invalid |= !viml_pexpr_handle_bop(
            self.pstate,
            &mut self.ast,
            &mut self.ast_stack,
            node,
            &mut self.want_node,
        );
    }

    /// `ADD_VALUE_IF_MISSING`: stand a Missing node in for the value an
    /// operator was expecting, as in `* 5`.
    pub(super) fn add_value_if_missing(&mut self, msg: &'static CStr) {
        if self.want_node == kENodeValue {
            self.error(msg);
            let node = self.new_node(kExprNodeMissing);
            self.ast.set_len(node, 0);
            self.ast.set_slot(self.top_node_p, Some(node));
            self.want_node = kENodeOperator;
        }
    }

    /// `OP_MISSING`: two values in a row, as in `:echo @a @a`.
    ///
    /// With `kExprFlagsMulti` and nothing but the root on the stack the caller
    /// gets to start a second expression at this token; otherwise an OpMissing
    /// operator is spliced in and the token is dispatched again, this time in
    /// value position.
    pub(super) fn op_missing(&mut self) -> Flow {
        if self.flags & kExprFlagsMulti as c_int != 0 && self.may_have_next_expr() {
            return Flow::Stop;
        }
        debug_assert!(
            self.ast.slot(self.top_node_p).is_some(),
            "*top_node_p != NULL"
        );
        self.error(c"E15: Missing operator: %.*s");
        let node = self.new_node(kExprNodeOpMissing);
        self.ast.set_len(node, 0);
        self.add_op_node(node);
        Flow::Reprocess
    }

    /// `SELECT_FIGURE_BRACE_TYPE`: commit a figure brace node to a type now
    /// that it is known, and recolour its opening brace to match.
    ///
    pub(super) fn select_figure_brace_type(
        &mut self,
        node: NodeId,
        new_type: ExprASTNodeType,
        group: &'static CStr,
    ) {
        assert!(
            self.ast.kind(node) == kExprNodeUnknownFigure || self.ast.kind(node) == new_type,
            "the node is still an unknown figure brace, or already the new type"
        );
        self.ast.set_kind(node, new_type);
        self.recolour(self.ast.fig(node).opening_hl_idx, group);
    }

    /// `ADD_IDENT`'s prologue: open a complex identifier — `a{b}c` and
    /// friends — around the value already on the stack, and answer the slot
    /// the caller's new identifier node goes into.
    ///
    /// `None` means this cannot be a part of a complex identifier after all,
    /// and the caller must report a missing operator: either there is spacing
    /// before it, or what precedes it is not an identifier.
    pub(super) fn open_complex_identifier(&mut self) -> Option<Slot> {
        debug_assert!(
            self.want_node == kENodeOperator,
            "want_node == kENodeOperator"
        );
        if self.prev_token.type_0 == kExprLexSpacing {
            return None;
        }
        match self.ast.kind(self.ast.filled(self.top_node_p)) {
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
        self.ast.set_len(node, 0);
        let operand = self.ast.slot(self.top_node_p);
        self.ast.set_children(node, operand);
        self.ast.set_slot(self.top_node_p, Some(node));
        let slot = Slot::Next(operand.expect("an identifier precedes it"));
        self.ast_stack.push(slot);
        debug_assert!(self.ast.slot(slot).is_none(), "*new_top_node_p == NULL");
        Some(slot)
    }

    /// The whole parse: one iteration of the loop per token.
    fn run(&mut self) {
        loop {
            self.is_concat_or_subscript = self.want_node == kENodeValue
                && self.ast_stack.len() > 1
                && self
                    .ast
                    .kind(self.ast.filled(stack_top(&self.ast_stack, 1)))
                    == kExprNodeConcatOrSubscript;
            self.lexer_flags = kELFlagPeek as c_int
                | (if self.flags & kExprFlagsDisallowEOC as c_int != 0 {
                    kELFlagForbidEOC as c_int
                } else {
                    0
                })
                | (if self.want_node == kENodeValue
                    && (self.ast_stack.len() == 1
                        || !matches!(
                            self.ast
                                .kind(self.ast.filled(stack_top(&self.ast_stack, 1))),
                            kExprNodeConcat | kExprNodeConcatOrSubscript
                        ))
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
            self.pstate.advance(self.cur_token.len);
        }
        self.finish();
    }

    /// Refresh the per-token state and hand the token to its class handler.
    fn process_token(&mut self) -> Flow {
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
        self.pline = self
            .pstate
            .line(self.cur_token.start.line)
            .expect("a token comes from a line of the input");
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
                !self.cur_token.variable().autoload
                    && self.cur_token.variable().scope == kExprVarScopeMissing
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
            let enclosing = self.ast.filled(stack_top(&self.ast_stack, 1));
            self.ast.set_kind(enclosing, kExprNodeConcat);
        }
        if let Some(flow) = self.reconcile_parse_type() {
            return flow;
        }
        debug_assert!(!self.pt_stack.is_empty(), "kv_size(pt_stack)");
        self.cur_pt = self.pt_top();
        debug_assert!(
            self.lambda_node.is_none() || self.cur_pt == kEPTLambdaArguments,
            "lambda_node == NULL || cur_pt == kEPTLambdaArguments"
        );
        self.dispatch()
    }

    /// The stack invariants the C checked under `#ifndef NDEBUG`: item 0 is
    /// the root slot, and item i + 1 points at item i's *last* child.
    ///
    /// Debug-only, as upstream's is. The walk is linear in the depth of the
    /// stack and runs once per token, so leaving it on makes every parse
    /// quadratic in its nesting: at 8,000 nested parentheses it is **98% of
    /// the run** — 1,630 ms of 1,658.
    fn check_stack_invariants(&self) {
        if !cfg!(debug_assertions) {
            return;
        }
        let want_value = self.want_node == kENodeValue;
        let ast = &self.ast;
        debug_assert!(
            want_value == ast.slot(self.top_node_p).is_none(),
            "want_value == (*top_node_p == NULL)"
        );
        debug_assert!(
            self.ast_stack[0] == Slot::Root,
            "kv_A(ast_stack, 0) == &ast.root"
        );
        let last = self.ast_stack.len().saturating_sub(1);
        for (i, (&slot, &next)) in self.ast_stack.iter().zip(&self.ast_stack[1..]).enumerate() {
            let item_null = want_value && i + 1 == last;
            let node = ast.filled(slot);
            let first = ast.first_child(node);
            let second = first.and_then(|first| ast.next(first));
            debug_assert!(
                next == Slot::Children(node)
                    && (if item_null {
                        first.is_none()
                    } else {
                        first.is_some() && second.is_none()
                    })
                    || first.is_some_and(|first| next == Slot::Next(first))
                        && (if item_null {
                            second.is_none()
                        } else {
                            second.is_some_and(|second| ast.next(second).is_none())
                        }),
                "item {i} + 1 is the last child slot of item {i}"
            );
        }
    }

    /// Pop parse type stack items that this token proves wrong: an
    /// as-yet-undecided figure brace that cannot be a lambda after all, or an
    /// assignment lvalue that this token cannot be part of.
    fn reconcile_parse_type(&mut self) -> Option<Flow> {
        let is_single_assignment = self.pt_top() == kEPTSingleAssignment;
        match self.pt_top() {
            kEPTLambdaArguments => {
                if self.want_node == kENodeOperator
                    && self.tok_type != kExprLexComma
                    && self.tok_type != kExprLexArrow
                    || self.want_node == kENodeValue
                        && !(self.cur_token.type_0 == kExprLexPlainIdentifier
                            && self.cur_token.variable().scope == kExprVarScopeMissing
                            && !self.cur_token.variable().autoload)
                        && self.tok_type != kExprLexArrow
                {
                    let lambda_node = self.lambda_node.expect("a lambda's arguments are open");
                    let mut fig = self.ast.fig(lambda_node);
                    fig.type_guesses.allow_lambda = false;
                    self.ast.set_data(lambda_node, ExprNodeData::Figure(fig));
                    let first = self.ast.first_child(lambda_node);
                    if first.is_some_and(|first| self.ast.kind(first) == kExprNodeComma) {
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
                        self.lambda_node = None;
                        self.pt_stack.truncate(self.pt_stack.len() - 1);
                    }
                }
            }
            kEPTSingleAssignment | kEPTAssignment => {
                if self.want_node == kENodeValue
                    && self.tok_type != kExprLexBracket
                    && self.tok_type != kExprLexPlainIdentifier
                    && (self.tok_type != kExprLexFigureBrace || self.cur_token.is_closing())
                    && !(self.node_is_key && self.tok_type == kExprLexNumber)
                    && self.tok_type != kExprLexEnv
                    && self.tok_type != kExprLexOption
                    && self.tok_type != kExprLexRegister
                {
                    self.error(c"E15: Expected value part of assignment lvalue: %.*s");
                    self.pt_stack.truncate(self.pt_stack.len() - 1);
                } else if self.want_node == kENodeOperator
                    && self.tok_type != kExprLexBracket
                    && (self.tok_type != kExprLexFigureBrace || self.cur_token.is_closing())
                    && self.tok_type != kExprLexDot
                    && (self.tok_type != kExprLexComma || !is_single_assignment)
                    && self.tok_type != kExprLexAssignment
                    // Curly brace identifiers: these contain a plain identifier
                    // or another curly brace where an operator is wanted.
                    && !((self.tok_type == kExprLexPlainIdentifier
                        || self.tok_type == kExprLexFigureBrace && !self.cur_token.is_closing())
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
    fn dispatch(&mut self) -> Flow {
        match self.tok_type {
            kExprLexMissing | kExprLexSpacing | kExprLexEOC => {
                unreachable!("the token loop handles these itself")
            }
            kExprLexInvalid => {
                self.error_at(self.cur_token.error().msg, self.cur_token.start);
                // Dispatch it again as whatever it was trying to be.
                self.tok_type = self.cur_token.error().type_0;
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
    fn finish(&mut self) {
        debug_assert!(!self.pt_stack.is_empty(), "kv_size(pt_stack)");
        debug_assert!(!self.ast_stack.is_empty(), "kv_size(ast_stack)");
        // kEPTLambdaArguments is blacklisted because its presence means a
        // better error message comes out of the other branch.
        if self.want_node == kENodeValue && self.pt_top() != kEPTLambdaArguments {
            let pos = self.pstate.pos;
            self.error_at(translate(c"E15: Expected value, got EOC: %.*s"), pos);
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
        while !self.ast.has_error() && !self.ast_stack.is_empty() {
            let slot = self.ast_stack.pop().expect("the stack is not empty");
            // This should only happen when want_node == kENodeValue.
            let node = self.ast.filled(slot);
            // TODO(ZyX-I): Rehighlight as invalid?
            let msg: &'static CStr = match self.ast.kind(node) {
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
                | kExprNodePlainKey => unreachable!("a plain value is only ever on top"),
                // Actually Vim throws E109 in more cases.
                kExprNodeTernaryValue if !self.ast.got_colon(node) => {
                    c"E109: Missing ':' after '?': %.*s"
                }
                // Everything else is either only valid inside something that
                // has to be closed — and so is caught later — or is fine to
                // see in the stack.
                _ => continue,
            };
            let start = self.ast.start(node);
            self.error_at(translate(msg), start);
        }
    }
}

/// Parse one Vimscript expression out of `pstate`'s input.
///
/// The tree borrows the input lines; `pstate.pos` is left where the parse
/// stopped.
pub fn viml_pexpr_parse<'a>(pstate: &mut ParserState<'a>, flags: c_int) -> ExprAST<'a> {
    let mut parser = ExprParser::new(pstate, flags);
    parser.run();
    parser.ast.check_children();
    parser.ast
}
