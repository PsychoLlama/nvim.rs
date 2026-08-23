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

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};

use super::{brackets, figure, operators, values, *};
use crate::types::ParserHighlight;
use crate::viml::parser::parser::reader_line;

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

/// The payload of a token, read back as the member its type selects.
///
/// The parser does read the *wrong* member in two places, and both are
/// deliberate: an invalid option token is asked for `opt.scope` and an
/// invalid comparison for `cmp.ccs`, over bytes the lexer wrote as `err`.
/// The C does the same — `values::option` and `operators::comparison` both
/// have a `kExprLexInvalid` arm that then reads on regardless — so these
/// answer whatever the union happens to hold, exactly as it did.
impl LexExprToken {
    /// `+=`, `-=`, `.=` or plain `=`.
    pub(super) fn assignment_type(&self) -> ExprAssignmentType {
        // SAFETY: reading a `Copy` member of a `repr(C)` union of `Copy`
        // members is a reinterpretation of initialised bytes, never a
        // dereference. Every accessor below carries the same reasoning.
        unsafe { self.data.ass.type_0 }
    }

    /// A number literal's base and whether it is a float.
    pub(super) fn number(&self) -> LexExprTokenNumber {
        unsafe { self.data.num }
    }

    /// A float literal's value. Only meaningful when `number().is_float`.
    pub(super) fn number_float(&self) -> float_T {
        unsafe { self.data.num.val.floating }
    }

    /// An integer literal's value. Only meaningful when `!number().is_float`.
    pub(super) fn number_integer(&self) -> uvarnumber_T {
        unsafe { self.data.num.val.integer }
    }

    /// What an invalid token was trying to be, and why it is not.
    pub(super) fn error(&self) -> LexExprTokenError {
        unsafe { self.data.err }
    }

    /// An identifier's scope and whether it is an autoload name.
    pub(super) fn variable(&self) -> LexExprTokenVar {
        unsafe { self.data.var }
    }

    /// An option's name, its length and its scope.
    pub(super) fn option(&self) -> LexExprTokenOption {
        unsafe { self.data.opt }
    }

    /// Whether a string literal reached its closing quote.
    pub(super) fn string_is_closed(&self) -> bool {
        unsafe { self.data.str.closed }
    }

    /// A register token's register name.
    pub(super) fn register_name(&self) -> ::core::ffi::c_int {
        unsafe { self.data.reg.name }
    }

    /// Whether a bracket, brace or parenthesis closes rather than opens.
    pub(super) fn is_closing(&self) -> bool {
        unsafe { self.data.brc.closing }
    }

    /// Which of `*`, `/` and `%` this is.
    pub(super) fn multiplication_type(&self) -> ExprLexMulType {
        unsafe { self.data.mul.type_0 }
    }

    /// A comparison's operator, case-comparison strategy and inversion.
    pub(super) fn comparison(&self) -> LexExprTokenComparison {
        unsafe { self.data.cmp }
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
    /// # Safety
    /// `pstate` must point at a parser initialised by `viml_parser_init` and
    /// `ast` at the AST being built, both of which must outlive the parse.
    /// Every method below relies on that and is safe because of it.
    unsafe fn new(pstate: *mut ParserState, ast: *mut ExprAST, flags: c_int) -> Self {
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
                data: LexExprTokenData {
                    cmp: LexExprTokenComparison {
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
                data: LexExprTokenData {
                    cmp: LexExprTokenComparison {
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
    fn next_token(&mut self) -> LexExprToken {
        // SAFETY: the parser holds `pstate` for the whole parse.
        unsafe {
            viml_pexpr_next_token(
                self.pstate,
                want_node_to_lexer_flags[self.want_node as usize] | self.lexer_flags,
            )
        }
    }

    /// The line a position falls on.
    fn line_at(&self, at: ParserPosition) -> ParserLine {
        // SAFETY: as above; the reborrow reaches only the reader.
        reader_line(unsafe { &(*self.pstate).reader }, at.line)
    }

    /// The caller's highlight log, or null when they wanted none.
    fn colors(&self) -> *mut ParserHighlight {
        // SAFETY: as above.
        unsafe { (*self.pstate).colors }
    }

    /// How many highlight chunks have been recorded so far; `None` when the
    /// caller asked for no highlighting.
    pub(super) fn highlight_count(&self) -> Option<size_t> {
        let colors = self.colors();
        // SAFETY: non-null, and the caller owns it for the whole parse.
        (!colors.is_null()).then(|| unsafe { (*colors).size })
    }

    /// Rewrite the highlight group of a chunk already recorded, as the guess
    /// at what a figure brace is narrows. A no-op without highlighting.
    ///
    /// This goes through `highlight_vec`, never the collection's own `items`:
    /// that pointer is stale while the log is still inline.
    pub(super) fn recolour(&self, index: size_t, group: *const c_char) {
        let colors = self.colors();
        if colors.is_null() {
            return;
        }
        // SAFETY: non-null, and the caller owns it for the whole parse.
        highlight_vec(unsafe { &mut *colors }).as_mut_slice()[index].group = group;
    }

    /// A pointer into the line the current token came from. The reader keeps
    /// every line it has read for the whole parse, so a node may hold on to
    /// this. `wrapping_add` because the C did: `col` is a position within the
    /// line, so the arithmetic is exact.
    pub(super) fn line_ptr(&self, col: size_t) -> *const c_char {
        self.pline.data.wrapping_add(col)
    }

    /// The byte at `col` of the line the current token came from.
    pub(super) fn line_byte(&self, col: size_t) -> u8 {
        // SAFETY: `pline` spans the whole line, and callers index within the
        // current token, which the lexer cut out of that line.
        unsafe { *self.line_ptr(col) as u8 }
    }

    /// Decode the current token's string literal into `node`, which becomes
    /// its owner.
    pub(super) fn decode_quoted_string(&self, node: *mut ExprASTNode) {
        // SAFETY: the parser holds `pstate` for the whole parse, and `node` is
        // the string node just allocated for this token.
        unsafe { parse_quoted_string(self.pstate, node, self.cur_token, self.is_invalid) };
    }

    /// `HL_CUR_TOKEN`: highlight the whole current token.
    pub(super) fn hl_token(&self, group: *const c_char) {
        self.hl_at(self.cur_token.start, self.cur_token.len, group);
    }

    /// Highlight a slice of the current token.
    pub(super) fn hl_at(&self, pos: ParserPosition, len: size_t, group: *const c_char) {
        // SAFETY: the parser holds `pstate` for the whole parse, and every
        // group named here is a `'static` string.
        unsafe { viml_parser_highlight(self.pstate, pos, len, group) };
    }

    /// `NEW_NODE_WITH_CUR_POS`: allocate a node spanning the current token,
    /// and the spacing before it if there was any.
    pub(super) fn new_node(&self, type_0: ExprASTNodeType) -> *mut ExprASTNode {
        let node = viml_pexpr_new_node(type_0);
        if self.prev_token.type_0 == kExprLexSpacing {
            let len = self.cur_token.len.wrapping_add(self.prev_token.len);
            set_node_span(node, self.prev_token.start, len);
        } else {
            set_node_span(node, self.cur_token.start, self.cur_token.len);
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
    pub(super) fn error_at(&mut self, msg: *const c_char, at: ParserPosition) {
        self.is_invalid = true;
        east_set_error(self.pstate, self.ast, msg, at);
    }

    /// `ADD_OP_NODE`: hand an operator node to the shunting yard.
    pub(super) fn add_op_node(&mut self, node: *mut ExprASTNode) {
        self.is_invalid |= !viml_pexpr_handle_bop(
            self.pstate,
            &mut self.ast_stack,
            node,
            &mut self.want_node,
            self.ast,
        );
    }

    /// `ADD_VALUE_IF_MISSING`: stand a Missing node in for the value an
    /// operator was expecting, as in `* 5`.
    pub(super) fn add_value_if_missing(&mut self, msg: &'static CStr) {
        if self.want_node == kENodeValue {
            self.error(msg);
            let node = self.new_node(kExprNodeMissing);
            set_node_len(node, 0);
            set_slot_node(self.top_node_p, node);
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
        debug_assert!(!slot_node(self.top_node_p).is_null(), "*top_node_p != NULL");
        self.error(c"E15: Missing operator: %.*s");
        let node = self.new_node(kExprNodeOpMissing);
        set_node_len(node, 0);
        self.add_op_node(node);
        Flow::Reprocess
    }

    /// `SELECT_FIGURE_BRACE_TYPE`: commit a figure brace node to a type now
    /// that it is known, and recolour its opening brace to match.
    ///
    pub(super) fn select_figure_brace_type(
        &mut self,
        node: *mut ExprASTNode,
        new_type: ExprASTNodeType,
        group: *const c_char,
    ) {
        assert!(
            node_type(node) == kExprNodeUnknownFigure || node_type(node) == new_type,
            "the node is still an unknown figure brace, or already the new type"
        );
        set_node_type(node, new_type);
        self.recolour(node_fig(node).opening_hl_idx, group);
    }

    /// `ADD_IDENT`'s prologue: open a complex identifier — `a{b}c` and
    /// friends — around the value already on the stack, and answer the slot
    /// the caller's new identifier node goes into.
    ///
    /// `None` means this cannot be a part of a complex identifier after all,
    /// and the caller must report a missing operator: either there is spacing
    /// before it, or what precedes it is not an identifier.
    pub(super) fn open_complex_identifier(&mut self) -> Option<*mut *mut ExprASTNode> {
        debug_assert!(
            self.want_node == kENodeOperator,
            "want_node == kENodeOperator"
        );
        if self.prev_token.type_0 == kExprLexSpacing {
            return None;
        }
        match node_type(slot_node(self.top_node_p)) {
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
        set_node_len(node, 0);
        set_node_children(node, slot_node(self.top_node_p));
        set_slot_node(self.top_node_p, node);
        self.ast_stack.push(next_slot(node_children(node)));
        let slot = stack_top(&self.ast_stack, 0);
        debug_assert!(slot_node(slot).is_null(), "*new_top_node_p == NULL");
        Some(slot)
    }

    /// The whole parse: one iteration of the loop per token.
    fn run(&mut self) {
        loop {
            self.is_concat_or_subscript = self.want_node == kENodeValue
                && self.ast_stack.len() > 1
                && node_type(slot_node(stack_top(&self.ast_stack, 1)))
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
                            node_type(slot_node(stack_top(&self.ast_stack, 1))),
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
            // SAFETY: the parser holds `pstate` for the whole parse; the two
            // reborrows are of disjoint fields and reach neither the AST stack
            // nor the highlight log.
            let (pos, reader) = unsafe { (&mut (*self.pstate).pos, &mut (*self.pstate).reader) };
            viml_parser_advance(pos, reader, self.cur_token.len);
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
        self.pline = self.line_at(self.cur_token.start);
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
            set_node_type(slot_node(stack_top(&self.ast_stack, 1)), kExprNodeConcat);
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
        debug_assert!(
            want_value == slot_node(self.top_node_p).is_null(),
            "want_value == (*top_node_p == NULL)"
        );
        debug_assert!(
            self.ast_stack[0] == ast_root_slot(self.ast),
            "kv_A(ast_stack, 0) == &ast.root"
        );
        let last = self.ast_stack.len().saturating_sub(1);
        for (i, (&slot, &next)) in self.ast_stack.iter().zip(&self.ast_stack[1..]).enumerate() {
            let item_null = want_value && i + 1 == last;
            let node = slot_node(slot);
            debug_assert!(
                children_slot(node) == next
                    && (if item_null {
                        node_children(node).is_null()
                    } else {
                        node_next(node_children(node)).is_null()
                    })
                    || next_slot(node_children(node)) == next
                        && (if item_null {
                            node_next(node_children(node)).is_null()
                        } else {
                            node_next(node_next(node_children(node))).is_null()
                        }),
                "(&(*kv_A(ast_stack, i))->children == kv_A(ast_stack, i + 1) && (item_null ? (*kv_A(ast_stack, i))->children == NULL : (*kv_A(ast_stack, i))->children->next == NULL)) || ((&(*kv_A(ast_stack, i))->children->next == kv_A(ast_stack, i + 1)) && (item_null ? (*kv_A(ast_stack, i))->children->next == NULL : (*kv_A(ast_stack, i))->children->next->next == NULL))"
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
                    let mut fig = node_fig(self.lambda_node);
                    fig.type_guesses.allow_lambda = false;
                    set_node_data(self.lambda_node, ExprNodeData::Figure(fig));
                    let first = node_children(self.lambda_node);
                    if !first.is_null() && node_type(first) == kExprNodeComma {
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
            // SAFETY: `abort` only ever ends the process.
            kExprLexMissing | kExprLexSpacing | kExprLexEOC => unsafe { abort() },
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
            // SAFETY: the parser holds `pstate` for the whole parse.
            let pos = unsafe { (*self.pstate).pos };
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
        while !ast_has_error(self.ast) && !self.ast_stack.is_empty() {
            let node = slot_node(self.ast_stack.pop().expect("the stack is not empty"));
            // This should only happen when want_node == kENodeValue.
            debug_assert!(!node.is_null(), "cur_node != NULL");
            // TODO(ZyX-I): Rehighlight as invalid?
            let msg: &'static CStr = match node_type(node) {
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
                // SAFETY: `abort` only ever ends the process.
                | kExprNodePlainKey => unsafe { abort() },
                // Actually Vim throws E109 in more cases.
                kExprNodeTernaryValue if !node_got_colon(node) => {
                    c"E109: Missing ':' after '?': %.*s"
                }
                // Everything else is either only valid inside something that
                // has to be closed — and so is caught later — or is fine to
                // see in the stack.
                _ => continue,
            };
            self.error_at(translate(msg), node_start(node));
        }
    }
}

/// Parse one Vimscript expression.
pub unsafe fn viml_pexpr_parse(pstate: *mut ParserState, flags: c_int) -> ExprAST {
    let mut ast = ExprAST {
        err: ExprASTError {
            msg: ::core::ptr::null::<c_char>(),
            arg: ::core::ptr::null::<c_char>(),
            arg_len: 0,
        },
        root: ::core::ptr::null_mut::<ExprASTNode>(),
    };
    // SAFETY: the caller's obligation, and `ast` lives to the end of this
    // frame — past the parse, which is all `ExprParser` needs.
    let mut parser = unsafe { ExprParser::new(pstate, &raw mut ast, flags) };
    parser.ast_stack.push(&raw mut ast.root);
    parser.run();
    ast
}
