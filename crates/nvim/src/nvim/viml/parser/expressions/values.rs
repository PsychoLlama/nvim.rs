//! Token handlers for the leaves of the tree: registers, options,
//! environment variables, numbers, identifiers and quoted strings.
//!
//! Every one of them is only valid where a value is wanted; in operator
//! position they either report a missing operator or, for a plain identifier
//! with no scope, join the preceding identifier into a curly-braces name.

use super::parse::{ExprParser, Flow, hl};
use super::*;

/// Number of characters to highlight as NumberPrefix, indexed by the base.
static base_to_prefix_length: [uint8_t; 17] = [
    0, 0, 2, // 0b
    0, 0, 0, 0, 0, 1, // 0
    0, 0, 0, 0, 0, 0, 0, 2, // 0x
];

/// `@a`.
pub(super) unsafe fn register(p: &mut ExprParser) -> Flow {
    if p.want_node == kENodeOperator {
        // Register in operator position: e.g. @a @a
        return p.op_missing();
    }
    let node = p.new_node(kExprNodeRegister);
    (*node).data.reg.name = p.cur_token.data.reg.name;
    *p.top_node_p = node;
    p.want_node = kENodeOperator;
    p.hl_token(hl!(p, Register));
    Flow::NextToken
}

/// `&opt`, `&g:opt`, `&l:opt`.
pub(super) unsafe fn option(p: &mut ExprParser) -> Flow {
    if p.want_node == kENodeOperator {
        return p.op_missing();
    }
    let node = p.new_node(kExprNodeOption);
    if p.cur_token.type_0 == kExprLexInvalid {
        assert!(
            p.cur_token.len == 1
                || p.cur_token.len == 3
                    && *p.pline.data.add(p.cur_token.start.col.wrapping_add(2))
                        as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int,
            "cur_token.len == 1 || (cur_token.len == 3 && pline.data[cur_token.start.col + 2] == ':')"
        );
        (*node).data.opt.ident = p.pline.data.add(p.cur_token.start.col).add(p.cur_token.len);
        (*node).data.opt.ident_len = 0;
        (*node).data.opt.scope = if p.cur_token.len == 3 {
            *p.pline.data.add(p.cur_token.start.col.wrapping_add(1)) as ExprOptScope
        } else {
            kExprOptScopeUnspecified
        };
    } else {
        (*node).data.opt.ident = p.cur_token.data.opt.name;
        (*node).data.opt.ident_len = p.cur_token.data.opt.len;
        (*node).data.opt.scope = p.cur_token.data.opt.scope;
    }
    *p.top_node_p = node;
    p.want_node = kENodeOperator;
    p.hl_at(p.cur_token.start, 1, hl!(p, OptionSigil));
    let scope_shift: size_t = if p.cur_token.data.opt.scope == kExprOptScopeUnspecified {
        0
    } else {
        2
    };
    if scope_shift != 0 {
        p.hl_at(shifted_pos(p.cur_token.start, 1), 1, hl!(p, OptionScope));
        p.hl_at(
            shifted_pos(p.cur_token.start, 2),
            1,
            hl!(p, OptionScopeDelimiter),
        );
    }
    p.hl_at(
        shifted_pos(p.cur_token.start, scope_shift.wrapping_add(1)),
        p.cur_token.len.wrapping_sub(scope_shift.wrapping_add(1)),
        hl!(p, OptionName),
    );
    Flow::NextToken
}

/// `$VAR`.
pub(super) unsafe fn environment(p: &mut ExprParser) -> Flow {
    if p.want_node == kENodeOperator {
        return p.op_missing();
    }
    let node = p.new_node(kExprNodeEnvironment);
    (*node).data.env.ident = p.pline.data.add(p.cur_token.start.col).offset(1);
    (*node).data.env.ident_len = p.cur_token.len.wrapping_sub(1);
    if (*node).data.env.ident_len == 0 {
        p.error(c"E15: Environment variable name missing");
    }
    *p.top_node_p = node;
    p.want_node = kENodeOperator;
    p.hl_at(p.cur_token.start, 1, hl!(p, EnvironmentSigil));
    p.hl_at(
        shifted_pos(p.cur_token.start, 1),
        p.cur_token.len.wrapping_sub(1),
        hl!(p, EnvironmentName),
    );
    Flow::NextToken
}

/// An integer or float literal — or a dictionary key, when it follows a dot.
pub(super) unsafe fn number(p: &mut ExprParser) -> Flow {
    if p.want_node != kENodeValue {
        return p.op_missing();
    }
    let node = if p.node_is_key {
        let node = p.new_node(kExprNodePlainKey);
        (*node).data.var.ident = p.pline.data.add(p.cur_token.start.col);
        (*node).data.var.ident_len = p.cur_token.len;
        p.hl_token(hl!(p, IdentifierKey));
        node
    } else if p.cur_token.data.num.is_float {
        let node = p.new_node(kExprNodeFloat);
        (*node).data.flt.value = p.cur_token.data.num.val.floating;
        p.hl_token(hl!(p, Float));
        node
    } else {
        let node = p.new_node(kExprNodeInteger);
        (*node).data.num.value = p.cur_token.data.num.val.integer;
        let prefix_length = base_to_prefix_length[p.cur_token.data.num.base as usize] as size_t;
        p.hl_at(p.cur_token.start, prefix_length, hl!(p, NumberPrefix));
        p.hl_at(
            shifted_pos(p.cur_token.start, prefix_length),
            p.cur_token.len.wrapping_sub(prefix_length),
            hl!(p, Number),
        );
        node
    };
    p.want_node = kENodeOperator;
    *p.top_node_p = node;
    Flow::NextToken
}

/// A bare or scoped identifier: `name`, `g:name`, or a dictionary key.
pub(super) unsafe fn plain_identifier(p: &mut ExprParser) -> Flow {
    let scope: ExprVarScope = if p.cur_token.type_0 == kExprLexInvalid {
        kExprVarScopeMissing
    } else {
        p.cur_token.data.var.scope
    };
    if p.want_node == kENodeValue {
        p.want_node = kENodeOperator;
        let node = p.new_node(if p.node_is_key {
            kExprNodePlainKey
        } else {
            kExprNodePlainIdentifier
        });
        (*node).data.var.scope = scope;
        let scope_shift: size_t = if scope == kExprVarScopeMissing { 0 } else { 2 };
        (*node).data.var.ident = p.pline.data.add(p.cur_token.start.col).add(scope_shift);
        (*node).data.var.ident_len = p.cur_token.len.wrapping_sub(scope_shift);
        *p.top_node_p = node;
        if scope_shift != 0 {
            debug_assert!(!p.node_is_key, "!node_is_key");
            p.hl_at(p.cur_token.start, 1, hl!(p, IdentifierScope));
            p.hl_at(
                shifted_pos(p.cur_token.start, 1),
                1,
                hl!(p, IdentifierScopeDelimiter),
            );
        }
        p.hl_at(
            shifted_pos(p.cur_token.start, scope_shift),
            p.cur_token.len.wrapping_sub(scope_shift),
            if p.node_is_key {
                hl!(p, IdentifierKey)
            } else {
                hl!(p, IdentifierName)
            },
        );
        return Flow::NextToken;
    }
    if scope != kExprVarScopeMissing {
        return p.op_missing();
    }
    // Operator position: this may only be another part of a curly braces
    // name, and only under the conditions `open_complex_identifier` checks.
    let Some(slot) = p.open_complex_identifier() else {
        return p.op_missing();
    };
    let node = p.new_node(kExprNodePlainIdentifier);
    (*node).data.var.scope = scope;
    (*node).data.var.ident = p.pline.data.add(p.cur_token.start.col);
    (*node).data.var.ident_len = p.cur_token.len;
    p.want_node = kENodeOperator;
    *slot = node;
    p.hl_token(hl!(p, IdentifierName));
    Flow::NextToken
}

/// A single- or double-quoted string literal.
pub(super) unsafe fn quoted_string(p: &mut ExprParser) -> Flow {
    let is_double = p.tok_type == kExprLexDoubleQuotedString;
    if !p.cur_token.data.str.closed {
        // It is weird, but Vim has two identical error messages with different
        // error numbers: "E114: Missing quote" and "E115: Missing quote".
        p.error(if is_double {
            c"E114: Missing double quote: %.*s"
        } else {
            c"E115: Missing single quote: %.*s"
        });
    }
    if p.want_node == kENodeOperator {
        return p.op_missing();
    }
    let node = p.new_node(if is_double {
        kExprNodeDoubleQuotedString
    } else {
        kExprNodeSingleQuotedString
    });
    *p.top_node_p = node;
    parse_quoted_string(p.pstate, node, p.cur_token, p.is_invalid);
    p.want_node = kENodeOperator;
    Flow::NextToken
}
