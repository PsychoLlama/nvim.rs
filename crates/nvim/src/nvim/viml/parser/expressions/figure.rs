//! The token handler for `{` and `}`.
//!
//! A figure brace is the parser's one genuinely ambiguous token: at `{` it
//! could still turn out to be a dictionary literal, a lambda, or a
//! curly-braces name, so the node is created as UnknownFigure and narrowed
//! later — by the arrow, by a colon, or at the closing brace. The opening
//! brace's highlight chunk is rewritten each time the guess narrows, which is
//! what `opening_hl_idx` is for.

use super::parse::{ExprParser, Flow, hl, pt_is_assignment};
use super::*;

pub(super) unsafe fn figure_brace(p: &mut ExprParser) -> Flow {
    if p.cur_token.data.brc.closing {
        return closing_figure_brace(p);
    }
    if p.want_node == kENodeValue {
        p.hl_token(hl!(p, FigureBrace));
        // Value: may be any of a lambda, a dictionary literal and a curly
        // braces name. Though if this is an assignment it may only be a curly
        // braces name.
        let in_assignment = pt_is_assignment(p.cur_pt);
        let node = if in_assignment {
            let node = p.new_node(kExprNodeCurlyBracesIdentifier);
            (*node).data.fig.type_guesses.allow_lambda = false;
            (*node).data.fig.type_guesses.allow_dict = false;
            (*node).data.fig.type_guesses.allow_ident = true;
            p.pt_stack.push(kEPTExpr);
            node
        } else {
            let node = p.new_node(kExprNodeUnknownFigure);
            (*node).data.fig.type_guesses.allow_lambda = true;
            (*node).data.fig.type_guesses.allow_dict = true;
            (*node).data.fig.type_guesses.allow_ident = true;
            node
        };
        if !(*p.pstate).colors.is_null() {
            (*node).data.fig.opening_hl_idx = (*(*p.pstate).colors).size.wrapping_sub(1);
        }
        *p.top_node_p = node;
        p.ast_stack.push(&raw mut (*node).children);
        if !in_assignment {
            // Upstream pushes kEPTLambdaArguments and arms `lambda_node`
            // unconditionally, but the assignment arm above has already decided
            // the node is a curly braces name (`allow_lambda == false`) — a
            // `:let` lvalue cannot be a lambda. Every consumer of
            // kEPTLambdaArguments then assumes the figure node is still a
            // *candidate* lambda, and on that path it is not: `,` trips
            // `assert(allow_lambda)` (compiled out upstream, so a curly braces
            // name silently became a Lambda) and `->` walks the AST stack for a
            // Lambda/UnknownFigure that is not there, popping past the bottom —
            // `kv_last` of an empty kvec. Both were reachable from a plain
            // `nvim_parse_expression(.., 'l', ..)`; see O-B14-15.
            //
            // Not arming it is the identity on every input that parsed before:
            // the kEPTLambdaArguments normalisation at the top of the loop
            // drops the entry (and clears `lambda_node`) for every token except
            // a bare identifier, `,` and `->` — and `,`/`->` are exactly the two
            // that misbehaved.
            p.pt_stack.push(kEPTLambdaArguments);
            p.lambda_node = node;
        }
    } else {
        // Operator: this may only be a part of a curly braces name.
        let Some(slot) = p.open_complex_identifier() else {
            return p.op_missing();
        };
        let node = p.new_node(kExprNodeCurlyBracesIdentifier);
        if !(*p.pstate).colors.is_null() {
            (*node).data.fig.opening_hl_idx = (*(*p.pstate).colors).size;
        }
        (*node).data.fig.type_guesses.allow_lambda = false;
        (*node).data.fig.type_guesses.allow_dict = false;
        (*node).data.fig.type_guesses.allow_ident = true;
        p.ast_stack.push(&raw mut (*node).children);
        if pt_is_assignment(p.cur_pt) {
            p.pt_stack.push(kEPTExpr);
        }
        p.want_node = kENodeValue;
        *slot = node;
        p.hl_token(hl!(p, Curly));
    }
    if pt_is_assignment(p.cur_pt) && !pt_is_assignment(p.pt_top()) {
        // Subtract 1 for the NULL at the top.
        debug_assert!(p.want_node == kENodeValue, "want_node == kENodeValue");
        p.asgn_level = p.ast_stack.len().wrapping_sub(1);
    }
    Flow::NextToken
}

unsafe fn closing_figure_brace(p: &mut ExprParser) -> Flow {
    // Always drop the topmost value:
    //
    // 1. When want_node != kENodeValue the topmost item is a *finished* left
    //    operand, which may as well be "{@a}" and needs not be finished again.
    // 2. Otherwise it points at NULL, which nobody wants.
    p.ast_stack.truncate(p.ast_stack.len() - 1);
    let new_top_node_p: *mut *mut ExprASTNode;
    let mut unexpected = false;
    if p.ast_stack.is_empty() {
        let node = p.new_node(kExprNodeUnknownFigure);
        (*node).data.fig.type_guesses.allow_lambda = false;
        (*node).data.fig.type_guesses.allow_dict = false;
        (*node).data.fig.type_guesses.allow_ident = false;
        (*node).len = 0;
        if p.want_node != kENodeValue {
            (*node).children = *p.top_node_p;
        }
        *p.top_node_p = node;
        new_top_node_p = p.top_node_p;
        unexpected = true;
    } else {
        if p.want_node == kENodeValue
            && (**stack_top(&p.ast_stack, 0)).type_0 != kExprNodeUnknownFigure
            && (**stack_top(&p.ast_stack, 0)).type_0 != kExprNodeComma
        {
            // The top being UnknownFigure may occur for an empty dictionary
            // literal, while Comma is expected in a non-empty one.
            p.error(c"E15: Expected value, got closing figure brace: %.*s");
        }
        let mut slot;
        loop {
            slot = p.ast_stack.pop().expect("the stack is not empty");
            if !(!p.ast_stack.is_empty()
                && (slot.is_null()
                    || (**slot).type_0 != kExprNodeUnknownFigure
                        && (**slot).type_0 != kExprNodeDictLiteral
                        && (**slot).type_0 != kExprNodeCurlyBracesIdentifier
                        && (**slot).type_0 != kExprNodeLambda))
            {
                break;
            }
        }
        new_top_node_p = slot;
        let new_top_node = *new_top_node_p;
        match (*new_top_node).type_0 {
            kExprNodeUnknownFigure => {
                if (*new_top_node).children.is_null() {
                    // No children of a curly braces node indicates an empty
                    // dictionary.
                    debug_assert!(p.want_node == kENodeValue, "want_node == kENodeValue");
                    debug_assert!(
                        (*new_top_node).data.fig.type_guesses.allow_dict,
                        "new_top_node->data.fig.type_guesses.allow_dict"
                    );
                    p.select_figure_brace_type(new_top_node, kExprNodeDictLiteral, hl!(p, Dict));
                    p.hl_token(hl!(p, Dict));
                } else if (*new_top_node).data.fig.type_guesses.allow_ident {
                    p.select_figure_brace_type(
                        new_top_node,
                        kExprNodeCurlyBracesIdentifier,
                        hl!(p, Curly),
                    );
                    p.hl_token(hl!(p, Curly));
                } else {
                    // If by this time the type of the node has not already been
                    // guessed, but it definitely is not a curly braces name,
                    // then it is invalid for sure.
                    p.error_at(
                        gettext(c"E15: Don't know what figure brace means: %.*s".as_ptr()),
                        (*new_top_node).start,
                    );
                    if !(*p.pstate).colors.is_null() {
                        // Reset the opening brace to NvimInvalidFigureBrace.
                        highlight_vec(&mut *(*p.pstate).colors).as_mut_slice()
                            [(*new_top_node).data.fig.opening_hl_idx]
                            .group = hl!(p, FigureBrace);
                    }
                    p.hl_token(hl!(p, FigureBrace));
                }
            }
            kExprNodeDictLiteral => p.hl_token(hl!(p, Dict)),
            kExprNodeCurlyBracesIdentifier => p.hl_token(hl!(p, Curly)),
            kExprNodeLambda => p.hl_token(hl!(p, Lambda)),
            _ => unexpected = true,
        }
    }
    if unexpected {
        debug_assert!(p.ast_stack.is_empty(), "!kv_size(ast_stack)");
        p.error(c"E15: Unexpected closing figure brace: %.*s");
        p.hl_token(hl!(p, FigureBrace));
    }
    p.ast_stack.push(new_top_node_p);
    p.want_node = kENodeOperator;
    if p.ast_stack.len() <= p.asgn_level {
        debug_assert!(
            p.ast_stack.len() == p.asgn_level,
            "kv_size(ast_stack) == asgn_level"
        );
        if p.cur_pt == kEPTExpr
            && p.pt_stack.len() > 1
            && pt_is_assignment(p.pt_stack[p.pt_stack.len() - 2])
        {
            p.pt_stack.truncate(p.pt_stack.len() - 1);
            p.asgn_level = 0;
        }
    }
    Flow::NextToken
}
