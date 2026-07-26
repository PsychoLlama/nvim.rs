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
};
use c2rust_neovim::src::nvim::viml::parser::expressions::{
    east_node_type_tab, viml_pexpr_free_ast, viml_pexpr_parse,
};
use c2rust_neovim::src::nvim::viml::parser::parser::{
    PARSER_STATE_INIT, ParserState, highlight_vec, parser_simple_get_line, viml_parser_destroy,
    viml_parser_init,
};

const EMPTY_LINE: ParserLine = ParserLine {
    data: ptr::null(),
    size: 0,
    allocated: false,
};

/// `kExprFlagsMulti`, the flag `nvim_parse_expression` passes when it is
/// allowed to stop at the first thing that cannot continue the expression.
const MULTI: c_int = 1;

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
    // kExprFlagsParseLet; the lvalue grammar is what uses the type stack.
    let expr = format!("a{}", "[b]".repeat(20));
    let parsed = parse_with_flags(&expr, 4);
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
