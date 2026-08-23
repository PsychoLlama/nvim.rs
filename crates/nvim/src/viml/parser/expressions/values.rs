//! Token handlers for the leaves of the tree: registers, options,
//! environment variables, numbers, identifiers and quoted strings.
//!
//! Every one of them is only valid where a value is wanted; in operator
//! position they either report a missing operator or, for a plain identifier
//! with no scope, join the preceding identifier into a curly-braces name.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::parse::{ExprParser, Flow, hl};
use super::*;

/// Number of characters to highlight as NumberPrefix, indexed by the base.
static base_to_prefix_length: [uint8_t; 17] = [
    0, 0, 2, // 0b
    0, 0, 0, 0, 0, 1, // 0
    0, 0, 0, 0, 0, 0, 0, 2, // 0x
];

/// `@a`.
pub(super) fn register(p: &mut ExprParser) -> Flow {
    if p.want_node == kENodeOperator {
        // Register in operator position: e.g. @a @a
        return p.op_missing();
    }
    let node = p.new_node(kExprNodeRegister);
    set_node_data(
        node,
        ExprNodeData::Register(ExprNodeRegister {
            name: p.cur_token.register_name(),
        }),
    );
    set_slot_node(p.top_node_p, node);
    p.want_node = kENodeOperator;
    p.hl_token(hl!(p, Register));
    Flow::NextToken
}

/// `&opt`, `&g:opt`, `&l:opt`.
pub(super) fn option(p: &mut ExprParser) -> Flow {
    if p.want_node == kENodeOperator {
        return p.op_missing();
    }
    let node = p.new_node(kExprNodeOption);
    let opt = if p.cur_token.type_0 == kExprLexInvalid {
        // `&`, or `&x:` with an unknown scope letter: there is no name, so the
        // node points just past the sigil and spans nothing.
        let at = p.cur_token.start.col;
        assert!(
            p.cur_token.len == 1 || p.cur_token.len == 3 && p.line_byte(at.wrapping_add(2)) == b':',
            "cur_token.len == 1 || (cur_token.len == 3 && pline.data[cur_token.start.col + 2] == ':')"
        );
        ExprNodeOption {
            ident: p.line_ptr(at.wrapping_add(p.cur_token.len)),
            ident_len: 0,
            scope: if p.cur_token.len == 3 {
                ExprOptScope::from(p.line_byte(at.wrapping_add(1)))
            } else {
                kExprOptScopeUnspecified
            },
        }
    } else {
        ExprNodeOption {
            ident: p.cur_token.option().name,
            ident_len: p.cur_token.option().len,
            scope: p.cur_token.option().scope,
        }
    };
    set_node_data(node, ExprNodeData::Opt(opt));
    set_slot_node(p.top_node_p, node);
    p.want_node = kENodeOperator;
    p.hl_at(p.cur_token.start, 1, hl!(p, OptionSigil));
    // Note: the scope read here is the *token's*, which for an invalid token
    // is whatever the lexer left in `err`. The C reads the same bytes.
    let scope_shift: size_t = if p.cur_token.option().scope == kExprOptScopeUnspecified {
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
pub(super) fn environment(p: &mut ExprParser) -> Flow {
    if p.want_node == kENodeOperator {
        return p.op_missing();
    }
    let node = p.new_node(kExprNodeEnvironment);
    let env = ExprNodeEnvironment {
        ident: p.line_ptr(p.cur_token.start.col.wrapping_add(1)),
        ident_len: p.cur_token.len.wrapping_sub(1),
    };
    set_node_data(node, ExprNodeData::Environment(env));
    if env.ident_len == 0 {
        p.error(c"E15: Environment variable name missing");
    }
    set_slot_node(p.top_node_p, node);
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
pub(super) fn number(p: &mut ExprParser) -> Flow {
    if p.want_node != kENodeValue {
        return p.op_missing();
    }
    let node = if p.node_is_key {
        let node = p.new_node(kExprNodePlainKey);
        set_node_data(
            node,
            ExprNodeData::Variable(ExprNodeVariable {
                scope: kExprVarScopeMissing,
                ident: p.line_ptr(p.cur_token.start.col),
                ident_len: p.cur_token.len,
            }),
        );
        p.hl_token(hl!(p, IdentifierKey));
        node
    } else if p.cur_token.number().is_float {
        let node = p.new_node(kExprNodeFloat);
        set_node_data(
            node,
            ExprNodeData::Float(ExprNodeFloat {
                value: p.cur_token.number_float(),
            }),
        );
        p.hl_token(hl!(p, Float));
        node
    } else {
        let node = p.new_node(kExprNodeInteger);
        set_node_data(
            node,
            ExprNodeData::Integer(ExprNodeInteger {
                value: p.cur_token.number_integer(),
            }),
        );
        let prefix_length = base_to_prefix_length[p.cur_token.number().base as usize] as size_t;
        p.hl_at(p.cur_token.start, prefix_length, hl!(p, NumberPrefix));
        p.hl_at(
            shifted_pos(p.cur_token.start, prefix_length),
            p.cur_token.len.wrapping_sub(prefix_length),
            hl!(p, Number),
        );
        node
    };
    p.want_node = kENodeOperator;
    set_slot_node(p.top_node_p, node);
    Flow::NextToken
}

/// A bare or scoped identifier: `name`, `g:name`, or a dictionary key.
pub(super) fn plain_identifier(p: &mut ExprParser) -> Flow {
    let scope: ExprVarScope = if p.cur_token.type_0 == kExprLexInvalid {
        kExprVarScopeMissing
    } else {
        p.cur_token.variable().scope
    };
    if p.want_node == kENodeValue {
        p.want_node = kENodeOperator;
        let node = p.new_node(if p.node_is_key {
            kExprNodePlainKey
        } else {
            kExprNodePlainIdentifier
        });
        let scope_shift: size_t = if scope == kExprVarScopeMissing { 0 } else { 2 };
        set_node_data(
            node,
            ExprNodeData::Variable(ExprNodeVariable {
                scope,
                ident: p.line_ptr(p.cur_token.start.col.wrapping_add(scope_shift)),
                ident_len: p.cur_token.len.wrapping_sub(scope_shift),
            }),
        );
        set_slot_node(p.top_node_p, node);
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
    set_node_data(
        node,
        ExprNodeData::Variable(ExprNodeVariable {
            scope,
            ident: p.line_ptr(p.cur_token.start.col),
            ident_len: p.cur_token.len,
        }),
    );
    p.want_node = kENodeOperator;
    set_slot_node(slot, node);
    p.hl_token(hl!(p, IdentifierName));
    Flow::NextToken
}

/// A single- or double-quoted string literal.
pub(super) fn quoted_string(p: &mut ExprParser) -> Flow {
    let is_double = p.tok_type == kExprLexDoubleQuotedString;
    if !p.cur_token.string_is_closed() {
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
    set_slot_node(p.top_node_p, node);
    p.decode_quoted_string(node);
    p.want_node = kENodeOperator;
    Flow::NextToken
}
