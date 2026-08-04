//! The Vimscript expression parser, driven end to end the way
//! `nvim_parse_expression` drives it.
//!
//! These exist as much for Miri as for the assertions. The parser builds
//! three `Vec`s while `ParserState`'s own collections point at arrays inside
//! the state, and it writes the tree through raw `ExprASTNode **` slots held
//! on the AST stack; a borrow-stack mistake anywhere in that traffic shows up
//! here and nowhere in the LuaJIT specs. Each case therefore also frees its
//! tree and tears the state down.

use std::ffi::{CStr, c_int, c_void};
use std::fmt::Write as _;
use std::ptr;

use c2rust_neovim::src::nvim::memory::xfree;
use c2rust_neovim::src::nvim::types::{
    ExprAST, ExprASTNode, ParserHighlight, ParserHighlightChunk, ParserLine, ParserPosition,
    ParserState,
};
use c2rust_neovim::src::nvim::viml::parser::expressions::{
    east_node_type_tab, viml_pexpr_free_ast, viml_pexpr_parse,
};
use c2rust_neovim::src::nvim::viml::parser::parser::{
    PARSER_STATE_INIT, highlight_vec, parser_simple_get_line, viml_parser_destroy, viml_parser_init,
};

const EMPTY_LINE: ParserLine = ParserLine {
    data: ptr::null(),
    size: 0,
    allocated: false,
};

/// `kExprFlagsMulti`, the flag `nvim_parse_expression` passes when it is
/// allowed to stop at the first thing that cannot continue the expression.
const MULTI: c_int = 1;

/// `kExprFlagsParseLet`: the token stream is an assignment lvalue, which has
/// its own grammar and its own parse-type stack.
const PARSE_LET: c_int = 4;

fn node_name(node: *const ExprASTNode) -> &'static str {
    unsafe {
        let name = east_node_type_tab.ptr().cast::<*const i8>();
        CStr::from_ptr(*name.add((*node).type_0 as usize))
            .to_str()
            .expect("node type names are ASCII")
    }
}

/// A parenthesised dump of the tree: `Type(child, child)`. This is the same
/// information `nvim_parse_expression` reports, in the same order, so it
/// pins the shape without depending on the RPC encoding.
fn dump(node: *const ExprASTNode, out: &mut String) {
    out.push_str(node_name(node));
    let mut child = unsafe { (*node).children };
    if child.is_null() {
        return;
    }
    out.push('(');
    let mut first = true;
    while !child.is_null() {
        if !first {
            out.push_str(", ");
        }
        first = false;
        dump(child, out);
        child = unsafe { (*child).next };
    }
    out.push(')');
}

struct Parsed {
    tree: String,
    error: Option<String>,
    groups: Vec<String>,
}

/// Parse `expr` and answer its tree dump, its error message and the
/// highlight groups it logged, then release everything it allocated.
fn parse_with_flags(expr: &str, flags: c_int) -> Parsed {
    let source = format!("{expr}\0");
    let mut input = [
        ParserLine {
            data: source.as_ptr().cast(),
            size: expr.len(),
            allocated: false,
        },
        EMPTY_LINE,
    ];
    let mut cursor = input.as_mut_ptr();

    let mut colors = ParserHighlight {
        size: 0,
        capacity: 0,
        items: ptr::null_mut(),
        init_array: [ParserHighlightChunk {
            start: ParserPosition { line: 0, col: 0 },
            end_col: 0,
            group: ptr::null(),
        }; 16],
    };
    colors.capacity = colors.init_array.len();
    colors.items = colors.init_array.as_mut_ptr();

    let mut pstate: ParserState = PARSER_STATE_INIT;
    let state = &raw mut pstate;
    unsafe {
        viml_parser_init(
            state,
            Some(parser_simple_get_line),
            &raw mut cursor as *mut c_void,
            &raw mut colors,
        );
        let ast: ExprAST = viml_pexpr_parse(state, flags);

        let mut tree = String::new();
        if ast.root.is_null() {
            tree.push_str("<empty>");
        } else {
            dump(ast.root, &mut tree);
        }
        let error = if ast.err.msg.is_null() {
            None
        } else {
            let mut msg = CStr::from_ptr(ast.err.msg)
                .to_str()
                .expect("error messages are ASCII")
                .to_owned();
            if ast.err.arg_len != 0 {
                let arg =
                    std::slice::from_raw_parts(ast.err.arg.cast::<u8>(), ast.err.arg_len as usize);
                let mut rendered = String::new();
                write!(rendered, "{}", String::from_utf8_lossy(arg)).unwrap();
                msg = msg.replace("%.*s", &rendered);
            }
            Some(msg)
        };

        // Read the log back through `highlight_vec`, not through the
        // pointer `colors.items` holds: the parser's own borrow of the
        // collection invalidated that one, and only the view re-derives it.
        let mut groups = Vec::new();
        for chunk in highlight_vec(&mut colors).as_slice() {
            groups.push(
                CStr::from_ptr(chunk.group)
                    .to_str()
                    .expect("group names are ASCII")
                    .to_owned(),
            );
        }

        viml_pexpr_free_ast(ast);
        viml_parser_destroy(&mut *state);
        // The chunk log belongs to the caller, not to the parser state: a
        // long expression pushes it off its inline array and onto the heap.
        xfree(highlight_vec(&mut colors).take_heap());
        Parsed {
            tree,
            error,
            groups,
        }
    }
}

fn parse(expr: &str) -> Parsed {
    parse_with_flags(expr, 0)
}

fn tree(expr: &str) -> String {
    let parsed = parse(expr);
    assert_eq!(parsed.error, None, "{expr} parsed with an error");
    parsed.tree
}

#[test]
fn arithmetic_binds_tighter_than_comparison() {
    assert_eq!(tree("1"), "Integer");
    assert_eq!(tree("1 + 2"), "BinaryPlus(Integer, Integer)");
    assert_eq!(
        tree("1 + 2 * 3"),
        "BinaryPlus(Integer, Multiplication(Integer, Integer))"
    );
    assert_eq!(
        tree("1 * 2 + 3"),
        "BinaryPlus(Multiplication(Integer, Integer), Integer)"
    );
    assert_eq!(
        tree("a == b + 1"),
        "Comparison(PlainIdentifier, BinaryPlus(PlainIdentifier, Integer))"
    );
    // Unary minus is above multiplication and below subscripting.
    assert_eq!(
        tree("-a[0]"),
        "UnaryMinus(Subscript(PlainIdentifier, Integer))"
    );
}

#[test]
fn ternary_is_right_associative() {
    assert_eq!(
        tree("a ? b : c ? d : e"),
        "Ternary(PlainIdentifier, TernaryValue(PlainIdentifier, \
         Ternary(PlainIdentifier, TernaryValue(PlainIdentifier, PlainIdentifier))))"
    );
}

#[test]
fn calls_lists_and_dictionaries() {
    assert_eq!(
        tree("abs(-3)"),
        "Call(PlainIdentifier, UnaryMinus(Integer))"
    );
    assert_eq!(tree("f()"), "Call(PlainIdentifier)");
    assert_eq!(
        tree("[1, 2, 3]"),
        "ListLiteral(Comma(Integer, Comma(Integer, Integer)))"
    );
    assert_eq!(tree("[]"), "ListLiteral");
    assert_eq!(
        tree("{'a': 1}"),
        "DictLiteral(Colon(SingleQuotedString, Integer))"
    );
    assert_eq!(
        tree("{-> 1}"),
        "Lambda(Arrow(Integer))",
        "a lambda with no arguments has only its arrow"
    );
    assert_eq!(
        tree("{a, b -> a}"),
        "Lambda(Comma(PlainIdentifier, PlainIdentifier), Arrow(PlainIdentifier))"
    );
}

/// `d.key` is a subscript only when what follows looks like a key and no
/// space intervenes; otherwise the parser rewrites the node to a plain
/// concatenation, because it cannot know whether `d` is a dictionary.
#[test]
fn dot_is_a_subscript_only_when_a_key_follows_immediately() {
    assert_eq!(tree("d.a"), "ConcatOrSubscript(PlainIdentifier, PlainKey)");
    assert_eq!(
        tree("d. a"),
        "Concat(PlainIdentifier, PlainIdentifier)",
        "a space after the dot forces concatenation"
    );
    assert_eq!(
        tree("d.g:a"),
        "Concat(PlainIdentifier, PlainIdentifier)",
        "a scoped name cannot be a key"
    );
}

#[test]
fn options_registers_and_environment_variables() {
    assert_eq!(tree("&option"), "Option");
    assert_eq!(tree("&l:option"), "Option");
    assert_eq!(tree("@a"), "Register");
    assert_eq!(tree("$HOME"), "Environment");
    assert_eq!(tree("!a"), "Not(PlainIdentifier)");
}

#[test]
fn strings_are_leaves_whatever_they_contain() {
    // The double-quoted decoder rewrites escapes in place, which is what
    // moves its shift log off the inline array further down.
    assert_eq!(tree(r#""a\nb""#), "DoubleQuotedString");
    assert_eq!(tree(r"'it''s'"), "SingleQuotedString");
    assert_eq!(
        tree(r#""a" . "b""#),
        "Concat(DoubleQuotedString, DoubleQuotedString)"
    );
}

/// The AST stack starts at zero capacity and every nested value pushes onto
/// it, so a deeply nested expression is what exercises its growth. The C's
/// inline array held sixteen.
#[test]
fn deep_nesting_grows_the_ast_stack() {
    let depth = 40;
    let expr = format!("{}1{}", "(".repeat(depth), ")".repeat(depth));
    let mut expected = String::new();
    for _ in 0..depth {
        expected.push_str("Nested(");
    }
    expected.push_str("Integer");
    expected.push_str(&")".repeat(depth));
    assert_eq!(tree(&expr), expected);
}

/// Same for the parse-type stack, which upstream never freed: each `[` in an
/// assignment lvalue pushes onto it.
#[test]
fn deep_assignment_lvalues_grow_the_parse_type_stack() {
    let expr = format!("a{}", "[b]".repeat(20));
    let parsed = parse_with_flags(&expr, PARSE_LET);
    assert_eq!(parsed.error, None);
    assert!(parsed.tree.starts_with("Subscript("));
}

/// A string long enough to need more than sixteen escape shifts, which is
/// where `parse_quoted_string`'s own vector leaves its inline array.
#[test]
fn many_escapes_grow_the_shift_log() {
    let expr = format!("\"{}\"", r"\n".repeat(40));
    assert_eq!(tree(&expr), "DoubleQuotedString");
}

#[test]
fn errors_carry_the_offending_text() {
    let parsed = parse("1 +");
    assert_eq!(
        parsed.error.as_deref(),
        Some("E15: Expected value, got EOC: %.*s".to_owned()).as_deref(),
        "an empty remainder leaves the placeholder unfilled"
    );
    assert_eq!(parsed.tree, "BinaryPlus(Integer)");

    let parsed = parse("(1");
    assert!(
        parsed
            .error
            .as_deref()
            .is_some_and(|m| m.starts_with("E110:")),
        "unclosed parenthesis: {:?}",
        parsed.error
    );

    let parsed = parse("a ? b");
    assert!(
        parsed
            .error
            .as_deref()
            .is_some_and(|m| m.starts_with("E109:")),
        "ternary without a colon: {:?}",
        parsed.error
    );
}

/// With `kExprFlagsMulti` the parser stops at the first token that cannot
/// continue the expression rather than reporting an error.
#[test]
fn multi_stops_instead_of_failing() {
    let parsed = parse_with_flags("1 2", MULTI);
    assert_eq!(parsed.error, None);
    assert_eq!(parsed.tree, "Integer");
}

/// Highlight chunks are part of the public surface: `nvim_parse_expression`
/// reports them and the cmdline highlighter paints them.
#[test]
fn highlighting_names_every_token() {
    assert_eq!(
        parse("1 + 2").groups,
        ["NvimNumber", "NvimBinaryPlus", "NvimNumber"],
        "spacing between valid tokens is left uncoloured"
    );
    assert_eq!(parse("[1]").groups, ["NvimList", "NvimNumber", "NvimList"]);
    assert_eq!(
        parse("1 +").groups,
        ["NvimNumber", "NvimBinaryPlus"],
        "the missing operand is reported through the error, not a chunk"
    );
}

/// A figure brace is coloured when it opens and renamed once the parser knows
/// what the node turned out to be. The rename reaches back into a chunk
/// already in the log, which is the one place the parser writes to a recorded
/// chunk rather than appending.
#[test]
fn figure_braces_are_recoloured_once_their_node_is_known() {
    assert_eq!(parse("{}").groups, ["NvimDict", "NvimDict"]);
    assert_eq!(
        parse("{-> 1}").groups,
        ["NvimLambda", "NvimArrow", "NvimNumber", "NvimLambda"]
    );
    assert_eq!(
        parse("{a}").groups,
        ["NvimCurly", "NvimIdentifierName", "NvimCurly"]
    );
}

/// Two values in a row splice an OpMissing operator in and hand the same
/// token to the dispatcher a second time, now in value position. That
/// re-dispatch is the parser's only backward jump, and every value token
/// class reaches it.
#[test]
fn a_value_in_operator_position_is_dispatched_twice() {
    for (expr, expected) in [
        ("1 2", "OpMissing(Integer, Integer)"),
        ("@a @a", "OpMissing(Register, Register)"),
        ("a $B", "OpMissing(PlainIdentifier, Environment)"),
        ("a &o", "OpMissing(PlainIdentifier, Option)"),
        ("1 'a'", "OpMissing(Integer, SingleQuotedString)"),
        ("1 !2", "OpMissing(Integer, Not(Integer))"),
        (
            "a [b]",
            "OpMissing(PlainIdentifier, ListLiteral(PlainIdentifier))",
        ),
        (
            "1 {a}",
            "OpMissing(Integer, CurlyBracesIdentifier(PlainIdentifier))",
        ),
        (
            "(a) (b)",
            "OpMissing(Nested(PlainIdentifier), Nested(PlainIdentifier))",
        ),
    ] {
        let parsed = parse(expr);
        assert_eq!(parsed.tree, expected, "{expr}");
        assert!(
            parsed
                .error
                .as_deref()
                .is_some_and(|m| m.starts_with("E15: Missing operator")),
            "{expr}: {:?}",
            parsed.error
        );
    }
    // An identifier before the spacing is the documented exception: Vim reads
    // "function (args)" as a call but "(funcref) (args)" as two values.
    assert_eq!(
        tree("a (b)"),
        "Call(PlainIdentifier, PlainIdentifier)",
        "spacing before a call's parenthesis is allowed after an identifier"
    );
}

/// `a{b}c` is one name, not three values: in operator position a figure brace
/// or a bare identifier opens a ComplexIdentifier around what came before.
#[test]
fn curly_braces_names_join_what_precedes_them() {
    assert_eq!(
        tree("a{b}c"),
        "ComplexIdentifier(PlainIdentifier, \
         ComplexIdentifier(CurlyBracesIdentifier(PlainIdentifier), PlainIdentifier))"
    );
    assert_eq!(
        parse("a{b}c").groups,
        [
            "NvimIdentifierName",
            "NvimCurly",
            "NvimIdentifierName",
            "NvimCurly",
            "NvimIdentifierName"
        ]
    );
}

/// A colon inside a subscript is a slice, and either side of it may be
/// missing.
#[test]
fn subscript_slices_may_omit_either_bound() {
    assert_eq!(
        tree("a[:2]"),
        "Subscript(PlainIdentifier, Colon(Missing, Integer))"
    );
    assert_eq!(tree("a[1:]"), "Subscript(PlainIdentifier, Colon(Integer))");
    assert_eq!(tree("a[:]"), "Subscript(PlainIdentifier, Colon(Missing))");
    assert_eq!(
        parse("a[:2]").groups,
        [
            "NvimIdentifierName",
            "NvimSubscriptBracket",
            "NvimSubscriptColon",
            "NvimNumber",
            "NvimSubscriptBracket"
        ]
    );
}

/// The assignment lvalue grammar: only subscripts, curly braces names and
/// list literals may appear to the left of the operator, and only one level
/// of list.
#[test]
fn assignment_lvalues_have_their_own_grammar() {
    for (expr, expected) in [
        ("a = 1", "Assignment(PlainIdentifier, Integer)"),
        (
            "[a] = 1",
            "Assignment(ListLiteral(PlainIdentifier), Integer)",
        ),
        (
            "{a} = 1",
            "Assignment(CurlyBracesIdentifier(PlainIdentifier), Integer)",
        ),
        (
            "a[1] = 2",
            "Assignment(Subscript(PlainIdentifier, Integer), Integer)",
        ),
    ] {
        let parsed = parse_with_flags(expr, PARSE_LET);
        assert_eq!(parsed.error, None, "{expr}");
        assert_eq!(parsed.tree, expected, "{expr}");
    }

    let parsed = parse_with_flags("[[a]] = 1", PARSE_LET);
    assert!(
        parsed
            .error
            .as_deref()
            .is_some_and(|m| m.starts_with("E475: Nested lists")),
        "{:?}",
        parsed.error
    );
    let parsed = parse_with_flags("a . b = 1", PARSE_LET);
    assert!(
        parsed
            .error
            .as_deref()
            .is_some_and(|m| m.starts_with("E15: Cannot concatenate")),
        "{:?}",
        parsed.error
    );
}

/// A separator or closing bracket that has nothing to belong to still
/// produces the node it would have made, wrapped around whatever the stack
/// held.
#[test]
fn a_bracket_or_separator_with_nothing_open_is_reported() {
    for (expr, expected, message) in [
        ("]", "ListLiteral", "E15: Unexpected closing figure brace"),
        ("}", "UnknownFigure", "E15: Unexpected closing figure brace"),
        (
            ")",
            "Nested(Missing)",
            "E15: Expected value, got parenthesis",
        ),
        (
            "1,2",
            "Comma(Integer, Integer)",
            "E15: Comma outside of call, lambda or literal",
        ),
        (
            "1:2",
            "Colon(Integer, Integer)",
            "E15: Colon outside of dictionary or ternary operator",
        ),
        (
            "a->b",
            "Arrow(PlainIdentifier, PlainIdentifier)",
            "E15: Arrow outside of lambda",
        ),
    ] {
        let parsed = parse(expr);
        assert_eq!(parsed.tree, expected, "{expr}");
        assert!(
            parsed
                .error
                .as_deref()
                .is_some_and(|m| m.starts_with(message)),
            "{expr}: {:?}",
            parsed.error
        );
    }
}

/// An invalid token reports what it was trying to be, and the parser
/// dispatches it again as that — so a lone sigil still yields its node.
#[test]
fn an_invalid_token_is_dispatched_as_what_it_meant_to_be() {
    let parsed = parse("&");
    assert_eq!(parsed.tree, "Option");
    assert!(
        parsed
            .error
            .as_deref()
            .is_some_and(|m| m.starts_with("E112:")),
        "{:?}",
        parsed.error
    );
    assert_eq!(parsed.groups, ["NvimInvalidOptionSigil"]);

    let parsed = parse("$");
    assert_eq!(parsed.tree, "Environment");
    assert_eq!(
        parsed.error.as_deref(),
        Some("E15: Environment variable name missing")
    );
}
