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

/// `viml_pexpr_next_token`, one token at a time — the port of
/// `test/unit/viml/expressions/lexer_spec.lua`.
///
/// Every case builds a parser over a single line, reads one token, and
/// compares all of it: the type, the span, the bytes that span covers *read
/// back out of the parser's own line* (so a length running past the line is
/// caught rather than believed), the per-type payload, and where the cursor
/// was left. The six `#[test]`s at the bottom are the spec's six flag
/// spellings; they share one body because the token stream is
/// flag-independent except where each of them says otherwise.
///
/// The enum constants keep the C's spelling, and matching on one is what
/// makes a forgotten import a compile error (an unimported name would bind
/// instead, and every arm after it become unreachable) — so the casing lint
/// is off here rather than the constants renamed.
#[allow(non_upper_case_globals)]
mod lexer {
    use std::ffi::{CStr, c_char, c_int, c_void};
    use std::{fmt, ptr, slice};

    use c2rust_neovim::src::nvim::types::{
        ExprAssignmentType, ExprCaseCompareStrategy, ExprComparisonType, ParserLine,
        ParserPosition, ParserState,
    };
    use c2rust_neovim::src::nvim::viml::parser::expressions::{
        LexExprToken, LexExprTokenType, ccs_tab, eltkn_cmp_type_tab, expr_asgn_type_tab,
        kELFlagAllowFloat, kELFlagForbidEOC, kELFlagForbidScope, kELFlagIsNotCmp, kELFlagPeek,
        kExprLexAnd, kExprLexArrow, kExprLexAssignment, kExprLexBracket, kExprLexColon,
        kExprLexComma, kExprLexComparison, kExprLexDot, kExprLexDoubleQuotedString, kExprLexEOC,
        kExprLexEnv, kExprLexFigureBrace, kExprLexInvalid, kExprLexMinus, kExprLexMissing,
        kExprLexMulDiv, kExprLexMulMod, kExprLexMulMul, kExprLexMultiplication, kExprLexNot,
        kExprLexNumber, kExprLexOption, kExprLexOr, kExprLexParenthesis, kExprLexPlainIdentifier,
        kExprLexPlus, kExprLexQuestion, kExprLexRegister, kExprLexSingleQuotedString,
        kExprLexSpacing, kExprOptScopeGlobal, kExprOptScopeLocal, kExprOptScopeUnspecified,
        viml_pexpr_next_token,
    };
    use c2rust_neovim::src::nvim::viml::parser::parser::{
        PARSER_STATE_INIT, parser_simple_get_line, reader_line, viml_parser_destroy,
        viml_parser_init,
    };

    use super::EMPTY_LINE;

    // -- the input ---------------------------------------------------------

    /// One line of input: the bytes the reader hands out, and the size it
    /// claims for them. The two differ where the spec truncates a line
    /// mid-token (`{ data = '009', size = 2 }`), which is how a scanner that
    /// reads past the end it was given gets caught: the bytes are there.
    #[derive(Clone)]
    struct Src {
        bytes: Vec<u8>,
        size: usize,
        present: bool,
    }

    /// A line the reader hands out whole.
    fn src(bytes: &[u8]) -> Src {
        Src {
            size: bytes.len(),
            bytes: bytes.to_vec(),
            present: true,
        }
    }

    /// A line whose claimed size is shorter than the bytes behind it.
    fn cut(bytes: &[u8], size: usize) -> Src {
        Src {
            bytes: bytes.to_vec(),
            size,
            present: true,
        }
    }

    /// "No line at all": a null `data`, which is not the same as an empty
    /// line — the reader stops on the former and scans the latter.
    fn absent() -> Src {
        Src {
            bytes: Vec::new(),
            size: 0,
            present: false,
        }
    }

    // -- the token, in the shape the spec compared -------------------------

    /// A byte string that stays readable when a comparison fails.
    #[derive(Clone, PartialEq)]
    struct Bytes(Vec<u8>);

    impl fmt::Debug for Bytes {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("\"")?;
            for &b in &self.0 {
                match b {
                    b'"' => f.write_str("\\\"")?,
                    b'\\' => f.write_str("\\\\")?,
                    0x20..=0x7e => write!(f, "{}", b as char)?,
                    _ => write!(f, "\\x{b:02x}")?,
                }
            }
            f.write_str("\"")
        }
    }

    /// `intchar2lua`: a character where one is printable, the raw number
    /// otherwise. Keeping the two apart is the point — a register named `\r`
    /// (13) and "no register name" (-1) must not compare equal to anything.
    #[derive(Clone, PartialEq, Debug)]
    enum Chr {
        Ch(char),
        Num(c_int),
    }

    fn intchar(ch: c_int) -> Chr {
        if (20..127).contains(&ch) {
            Chr::Ch(ch as u8 as char)
        } else {
            Chr::Num(ch)
        }
    }

    fn ch(c: char) -> Chr {
        Chr::Ch(c)
    }

    fn num(n: c_int) -> Chr {
        Chr::Num(n)
    }

    /// The token's per-type payload — the union member its type selects.
    #[derive(Clone, PartialEq, Debug)]
    enum Tkd {
        /// The types that carry nothing.
        None,
        Cmp {
            kind: String,
            ccs: String,
            inv: bool,
        },
        Mul(&'static str),
        Brc {
            closing: bool,
        },
        Reg {
            name: Chr,
        },
        Str {
            closed: bool,
        },
        Opt {
            scope: &'static str,
            name: Bytes,
        },
        Var {
            scope: Chr,
            autoload: bool,
        },
        Int {
            base: u8,
            val: u64,
        },
        Flt {
            base: u8,
            val: f64,
        },
        Asgn(String),
        Err(String),
    }

    fn cmp_data(kind: &str, inv: bool, ccs: &str) -> Tkd {
        Tkd::Cmp {
            kind: kind.to_owned(),
            ccs: ccs.to_owned(),
            inv,
        }
    }

    fn int(base: u8, val: u64) -> Tkd {
        Tkd::Int { base, val }
    }

    fn flt(base: u8, val: f64) -> Tkd {
        Tkd::Flt { base, val }
    }

    fn opt(scope: &'static str, name: &str) -> Tkd {
        Tkd::Opt {
            scope,
            name: Bytes(name.as_bytes().to_vec()),
        }
    }

    fn var(scope: Chr, autoload: bool) -> Tkd {
        Tkd::Var { scope, autoload }
    }

    fn reg(name: Chr) -> Tkd {
        Tkd::Reg { name }
    }

    fn quoted(closed: bool) -> Tkd {
        Tkd::Str { closed }
    }

    fn brc(closing: bool) -> Tkd {
        Tkd::Brc { closing }
    }

    fn asgn(kind: &str) -> Tkd {
        Tkd::Asgn(kind.to_owned())
    }

    fn err(msg: &str) -> Tkd {
        Tkd::Err(msg.to_owned())
    }

    #[derive(Clone, PartialEq, Debug)]
    struct Tok {
        kind: &'static str,
        start: (usize, usize),
        len: usize,
        text: Option<Bytes>,
        error: Option<String>,
        data: Tkd,
    }

    // -- reading the token back --------------------------------------------

    /// One of the parser's own `Nvim*` name tables, which is where the token
    /// types' spellings come from — the same tables `nvim_parse_expression`
    /// reports through.
    fn tab_name(entry: *const c_char) -> String {
        assert!(!entry.is_null(), "the name table has no entry for that");
        // SAFETY: a non-null entry of a table of static C strings.
        unsafe { CStr::from_ptr(entry) }
            .to_str()
            .expect("names are ASCII")
            .to_owned()
    }

    fn cmp_name(kind: ExprComparisonType) -> String {
        tab_name(eltkn_cmp_type_tab.with(|tab| tab[kind as usize]))
    }

    fn ccs_name(ccs: ExprCaseCompareStrategy) -> String {
        tab_name(ccs_tab.with(|tab| tab[ccs as usize]))
    }

    fn asgn_name(kind: ExprAssignmentType) -> String {
        tab_name(expr_asgn_type_tab.with(|tab| tab[kind as usize]))
    }

    fn kind_name(kind: LexExprTokenType) -> &'static str {
        match kind {
            kExprLexInvalid => "Invalid",
            kExprLexMissing => "Missing",
            kExprLexSpacing => "Spacing",
            kExprLexEOC => "EOC",
            kExprLexQuestion => "Question",
            kExprLexColon => "Colon",
            kExprLexOr => "Or",
            kExprLexAnd => "And",
            kExprLexComparison => "Comparison",
            kExprLexPlus => "Plus",
            kExprLexMinus => "Minus",
            kExprLexDot => "Dot",
            kExprLexMultiplication => "Multiplication",
            kExprLexNot => "Not",
            kExprLexNumber => "Number",
            kExprLexSingleQuotedString => "SingleQuotedString",
            kExprLexDoubleQuotedString => "DoubleQuotedString",
            kExprLexOption => "Option",
            kExprLexRegister => "Register",
            kExprLexEnv => "Env",
            kExprLexPlainIdentifier => "PlainIdentifier",
            kExprLexBracket => "Bracket",
            kExprLexFigureBrace => "FigureBrace",
            kExprLexParenthesis => "Parenthesis",
            kExprLexComma => "Comma",
            kExprLexArrow => "Arrow",
            kExprLexAssignment => "Assignment",
            other => panic!("unknown token type {other}"),
        }
    }

    /// `pstate_str`: the bytes the token's span covers, read back out of the
    /// line the parser kept rather than out of the token — which is what
    /// makes a `len` that overruns the line visible.
    ///
    /// # Safety
    /// `state` must point at a parser that has read at least one line.
    unsafe fn span(
        state: *mut ParserState,
        start: ParserPosition,
        len: usize,
    ) -> (Option<Bytes>, Option<String>) {
        // SAFETY: the caller's obligation. The reborrow is of the reader
        // alone, which is what `reader_line` reads.
        let reader = unsafe { &(*state).reader };
        if start.line >= reader.lines.size {
            return (
                None,
                Some("start.line >= pstate.reader.lines.size".to_owned()),
            );
        }
        let pline = reader_line(reader, start.line);
        let line: &[u8] = if pline.data.is_null() {
            &[]
        } else {
            // SAFETY: a `ParserLine` describes `size` readable bytes.
            unsafe { slice::from_raw_parts(pline.data.cast::<u8>(), pline.size) }
        };
        if start.col >= line.len() {
            return (None, Some("start.col >= #pstr".to_owned()));
        }
        let end = line.len().min(start.col + len);
        (Some(Bytes(line[start.col..end].to_vec())), None)
    }

    /// `eltkn2lua`'s payload half: the union member the token's type selects.
    ///
    /// # Safety
    /// `tkn` must be a token the lexer answered, still describing a live line.
    unsafe fn payload(kind: &str, tkn: &LexExprToken) -> Tkd {
        // SAFETY: the caller's obligation; each arm reads only the member its
        // own type selects, which is the invariant the lexer maintains.
        unsafe {
            match kind {
                "Comparison" => Tkd::Cmp {
                    kind: cmp_name(tkn.data.cmp.type_0),
                    ccs: ccs_name(tkn.data.cmp.ccs),
                    inv: tkn.data.cmp.inv,
                },
                "Multiplication" => Tkd::Mul(match tkn.data.mul.type_0 {
                    kExprLexMulMul => "Mul",
                    kExprLexMulDiv => "Div",
                    kExprLexMulMod => "Mod",
                    other => panic!("unknown multiplication type {other}"),
                }),
                "Bracket" | "FigureBrace" | "Parenthesis" => Tkd::Brc {
                    closing: tkn.data.brc.closing,
                },
                "Register" => Tkd::Reg {
                    name: intchar(tkn.data.reg.name),
                },
                "SingleQuotedString" | "DoubleQuotedString" => Tkd::Str {
                    closed: tkn.data.str.closed,
                },
                "Option" => Tkd::Opt {
                    scope: match tkn.data.opt.scope {
                        kExprOptScopeUnspecified => "Unspecified",
                        kExprOptScopeGlobal => "Global",
                        kExprOptScopeLocal => "Local",
                        other => panic!("unknown option scope {other}"),
                    },
                    name: Bytes(
                        slice::from_raw_parts(tkn.data.opt.name.cast::<u8>(), tkn.data.opt.len)
                            .to_vec(),
                    ),
                },
                "PlainIdentifier" => Tkd::Var {
                    scope: intchar(tkn.data.var.scope as c_int),
                    autoload: tkn.data.var.autoload,
                },
                "Number" => {
                    let base = tkn.data.num.base;
                    if tkn.data.num.is_float {
                        Tkd::Flt {
                            base,
                            val: tkn.data.num.val.floating,
                        }
                    } else {
                        Tkd::Int {
                            base,
                            val: tkn.data.num.val.integer,
                        }
                    }
                }
                "Assignment" => Tkd::Asgn(asgn_name(tkn.data.ass.type_0)),
                "Invalid" => Tkd::Err(
                    CStr::from_ptr(tkn.data.err.msg)
                        .to_str()
                        .expect("error messages are ASCII")
                        .to_owned(),
                ),
                _ => Tkd::None,
            }
        }
    }

    /// Read one token out of `lines`, starting at column `col`. Answers the
    /// token as the spec compared it, where the cursor was left, and the size
    /// of the first line — which is what decides whether the cursor wrapped.
    fn lex(lines: &[Src], col: usize, flags: c_int) -> (Tok, ParserPosition, usize) {
        let mut plines: Vec<ParserLine> = lines
            .iter()
            .map(|line| ParserLine {
                data: if line.present {
                    line.bytes.as_ptr().cast()
                } else {
                    ptr::null()
                },
                size: line.size,
                allocated: false,
            })
            .collect();
        plines.push(EMPTY_LINE);
        let mut cursor = plines.as_mut_ptr();
        let mut pstate = PARSER_STATE_INIT;
        let state = &raw mut pstate;
        // SAFETY: the state stays put for the whole call, the getter walks a
        // null-terminated array, and every line outlives the token read out
        // of it.
        unsafe {
            viml_parser_init(
                state,
                Some(parser_simple_get_line),
                &raw mut cursor as *mut c_void,
                ptr::null_mut(),
            );
            (*state).pos.col = col;
            let tkn = viml_pexpr_next_token(state, flags);
            let kind = kind_name(tkn.type_0);
            let (text, mut error) = span(state, tkn.start, tkn.len);
            if error.is_none() && text.as_ref().is_some_and(|got| got.0.len() != tkn.len) {
                error = Some("#str /= len".to_owned());
            }
            let tok = Tok {
                kind,
                start: (tkn.start.line, tkn.start.col),
                len: tkn.len,
                text,
                error,
                data: payload(kind, &tkn),
            };
            let pos = (*state).pos;
            let first = reader_line(&(*state).reader, 0).size;
            viml_parser_destroy(&mut *state);
            (tok, pos, first)
        }
    }

    // -- the spec's three case shapes --------------------------------------

    /// `singl_eltkn_test`: the same token read three ways — alone, with a
    /// space after it, and one byte into a longer line — each time checking
    /// where the cursor ended up.
    fn single(flags: c_int, advance: bool, kind: &'static str, text: &str, data: Tkd) {
        let bytes = text.as_bytes().to_vec();
        let want = |col: usize| Tok {
            kind,
            start: (0, col),
            len: bytes.len(),
            text: Some(Bytes(bytes.clone())),
            error: None,
            data: data.clone(),
        };

        one(flags, advance, &[src(&bytes)], 0, want(0), text);

        // A trailing space does not change where the token ends — except
        // where it would join the token (spacing), complete it (a bare `@`)
        // or be swallowed by it (an unterminated string).
        let absorbs_the_space = kind == "Spacing"
            || (kind == "Register" && text == "@")
            || (matches!(kind, "SingleQuotedString" | "DoubleQuotedString")
                && matches!(data, Tkd::Str { closed: false }));
        if !absorbs_the_space {
            let mut padded = bytes.clone();
            padded.push(b' ');
            one(flags, advance, &[src(&padded)], 0, want(0), text);
        }

        // And again one byte in, where nothing about the token may depend on
        // its being at the start of the line.
        let mut shifted = vec![b'x'];
        shifted.extend_from_slice(&bytes);
        one(flags, advance, &[src(&shifted)], 1, want(1), text);
    }

    /// One reading, plus `check_advance`: the cursor lands past the token,
    /// or at the start of the next line when the token ended this one, or
    /// exactly where it started when the caller only peeked.
    fn one(flags: c_int, advance: bool, lines: &[Src], col: usize, want: Tok, label: &str) {
        let (got, pos, first) = lex(lines, col, flags);
        assert_eq!(got, want, "{label:?} at column {col}, flags {flags}");
        let target = col + want.len;
        let expected = if !advance {
            (0, col)
        } else if first == target {
            (1, 0)
        } else {
            (0, target)
        };
        assert_eq!(
            (pos.line, pos.col),
            expected,
            "cursor after {label:?} at column {col}, flags {flags}"
        );
    }

    /// `simple_test`: one token out of a line the caller shapes, with no
    /// claim about the cursor.
    fn simple(flags: c_int, lines: &[Src], kind: &'static str, len: usize, text: &str, data: Tkd) {
        let (got, _, _) = lex(lines, 0, flags);
        assert_eq!(
            got,
            Tok {
                kind,
                start: (0, 0),
                len,
                text: Some(Bytes(text.as_bytes().to_vec())),
                error: None,
                data,
            },
            "{text:?}, flags {flags}"
        );
    }

    /// A line with nothing on it is end-of-command, and there is no text to
    /// read back for the empty span it answers.
    fn empty_is_eoc(flags: c_int, lines: &[Src]) {
        let (got, _, _) = lex(lines, 0, flags);
        assert_eq!(
            got,
            Tok {
                kind: "EOC",
                start: (0, 0),
                len: 0,
                text: None,
                error: Some("start.col >= #pstr".to_owned()),
                data: Tkd::None,
            },
            "an empty line, flags {flags}"
        );
    }

    /// `comparison_test`: an operator, its negation, and both under each of
    /// the two case-sensitivity suffixes.
    fn comparison(flags: c_int, advance: bool, op: &str, inv_op: &str, kind: &str) {
        for (text, inv, ccs) in [
            (op.to_owned(), false, "UseOption"),
            (inv_op.to_owned(), true, "UseOption"),
            (format!("{op}#"), false, "MatchCase"),
            (format!("{inv_op}#"), true, "MatchCase"),
            (format!("{op}?"), false, "IgnoreCase"),
            (format!("{inv_op}?"), true, "IgnoreCase"),
        ] {
            single(
                flags,
                advance,
                "Comparison",
                &text,
                cmp_data(kind, inv, ccs),
            );
        }
    }

    // -- the groups --------------------------------------------------------

    /// Everything the lexer reads the same way whatever the flags say.
    fn stable(flags: c_int, advance: bool) {
        let s = |kind, text, data| single(flags, advance, kind, text, data);
        s("Parenthesis", "(", brc(false));
        s("Parenthesis", ")", brc(true));
        s("Bracket", "[", brc(false));
        s("Bracket", "]", brc(true));
        s("FigureBrace", "{", brc(false));
        s("FigureBrace", "}", brc(true));
        s("Question", "?", Tkd::None);
        s("Colon", ":", Tkd::None);
        s("Dot", ".", Tkd::None);
        s("Assignment", ".=", asgn("Concat"));
        s("Plus", "+", Tkd::None);
        s("Assignment", "+=", asgn("Add"));
        s("Comma", ",", Tkd::None);
        s("Multiplication", "*", Tkd::Mul("Mul"));
        s("Multiplication", "/", Tkd::Mul("Div"));
        s("Multiplication", "%", Tkd::Mul("Mod"));
        s("Spacing", "  \t\t  \t\t", Tkd::None);
        s("Spacing", " ", Tkd::None);
        s("Spacing", "\t", Tkd::None);
        s(
            "Invalid",
            "\x01\x02\x03",
            err("E15: Invalid control character present in input: %.*s"),
        );
        s("Number", "0123", int(8, 83));
        s("Number", "01234567", int(8, 342391));
        s("Number", "012345678", int(10, 12345678));
        s("Number", "0x123", int(16, 291));
        s("Number", "0x56FF", int(16, 22271));
        s("Number", "0xabcdef", int(16, 11259375));
        s("Number", "0xABCDEF", int(16, 11259375));
        s("Number", "0x0", int(16, 0));
        s("Number", "00", int(8, 0));
        s("Number", "0b0", int(2, 0));
        s("Number", "0b010111", int(2, 23));
        s("Number", "0b100111", int(2, 39));
        s("Number", "0", int(10, 0));
        s("Number", "9", int(10, 9));
        s("Env", "$abc", Tkd::None);
        s("Env", "$", Tkd::None);
        s("PlainIdentifier", "test", var(num(0), false));
        s("PlainIdentifier", "_test", var(num(0), false));
        s("PlainIdentifier", "_test_foo", var(num(0), false));
        s("PlainIdentifier", "t", var(num(0), false));
        s("PlainIdentifier", "test5", var(num(0), false));
        s("PlainIdentifier", "t0", var(num(0), false));
        s("PlainIdentifier", "test#var", var(num(0), true));
        s("PlainIdentifier", "test#var#val###", var(num(0), true));
        s("PlainIdentifier", "t#####", var(num(0), true));
        s("And", "&&", Tkd::None);
        s("Or", "||", Tkd::None);
        s("Invalid", "&", err("E112: Option name missing: %.*s"));
        s("Option", "&opt", opt("Unspecified", "opt"));
        s("Option", "&t_xx", opt("Unspecified", "t_xx"));
        s("Option", "&t_\r\r", opt("Unspecified", "t_\r\r"));
        s("Option", "&t_\t\t", opt("Unspecified", "t_\t\t"));
        s("Option", "&t_  ", opt("Unspecified", "t_  "));
        s("Option", "&g:opt", opt("Global", "opt"));
        s("Option", "&l:opt", opt("Local", "opt"));
        s("Invalid", "&l:", err("E112: Option name missing: %.*s"));
        s("Invalid", "&g:", err("E112: Option name missing: %.*s"));
        s("Register", "@", reg(num(-1)));
        s("Register", "@a", reg(ch('a')));
        s("Register", "@\r", reg(num(13)));
        s("Register", "@ ", reg(ch(' ')));
        s("Register", "@\t", reg(num(9)));
        s("SingleQuotedString", "'test", quoted(false));
        s("SingleQuotedString", "'test'", quoted(true));
        s("SingleQuotedString", "''''", quoted(true));
        s("SingleQuotedString", "'x'''", quoted(true));
        s("SingleQuotedString", "'''x'", quoted(true));
        s("SingleQuotedString", "'''", quoted(false));
        s("SingleQuotedString", "'x''", quoted(false));
        s("SingleQuotedString", "'''x", quoted(false));
        s("DoubleQuotedString", "\"test", quoted(false));
        s("DoubleQuotedString", "\"test\"", quoted(true));
        s("DoubleQuotedString", r#""\"""#, quoted(true));
        s("DoubleQuotedString", r#""x\"""#, quoted(true));
        s("DoubleQuotedString", r#""\"x""#, quoted(true));
        s("DoubleQuotedString", r#""\""#, quoted(false));
        s("DoubleQuotedString", r#""x\""#, quoted(false));
        s("DoubleQuotedString", r#""\"x"#, quoted(false));
        s("Not", "!", Tkd::None);
        s("Assignment", "=", asgn("Plain"));
        comparison(flags, advance, "==", "!=", "Equal");
        comparison(flags, advance, "=~", "!~", "Matches");
        comparison(flags, advance, ">", "<=", "Greater");
        comparison(flags, advance, ">=", "<", "GreaterOrEqual");
        s("Minus", "-", Tkd::None);
        s("Assignment", "-=", asgn("Subtract"));
        s("Arrow", "->", Tkd::None);
        s("Invalid", "~", err("E15: Unidentified character: %.*s"));

        empty_is_eoc(flags, &[absent()]);
        empty_is_eoc(flags, &[src(b"")]);

        // A float needs `kELFlagAllowFloat`; without it the scan stops at the
        // dot, whatever follows it.
        for text in [
            "2.", "2e5", "2.x", "2.2.", "2.0x", "2.0e", "2.0e+", "2.0e-", "2.0e+x", "2.0e-x",
            "2.0e+1a", "2.0e-1a",
        ] {
            simple(flags, &[src(text.as_bytes())], "Number", 1, "2", int(10, 2));
        }
        simple(flags, &[src(b"0b102")], "Number", 4, "0b10", int(2, 2));
        simple(flags, &[src(b"10F")], "Number", 2, "10", int(10, 10));
        simple(
            flags,
            &[src(b"0x0123456789ABCDEFG")],
            "Number",
            18,
            "0x0123456789ABCDEF",
            int(16, 81985529216486895),
        );
        // A line the reader cut short: the digits past `size` are readable,
        // and must still not be read.
        simple(flags, &[cut(b"00", 2)], "Number", 2, "00", int(8, 0));
        simple(flags, &[cut(b"009", 2)], "Number", 2, "00", int(8, 0));
        simple(flags, &[cut(b"01", 1)], "Number", 1, "0", int(10, 0));
    }

    /// A leading `x:` is a scope, and a `#` anywhere after it makes the name
    /// an autoload one.
    fn scopes(flags: c_int, advance: bool) {
        for scope in ['s', 'g', 'v', 'b', 'w', 't', 'l', 'a'] {
            single(
                flags,
                advance,
                "PlainIdentifier",
                &format!("{scope}:test#var"),
                var(ch(scope), true),
            );
            single(
                flags,
                advance,
                "PlainIdentifier",
                &format!("{scope}:"),
                var(ch(scope), false),
            );
        }
        simple(
            flags,
            &[src(b"g:")],
            "PlainIdentifier",
            2,
            "g:",
            var(ch('g'), false),
        );
        simple(
            flags,
            &[src(b"g:is#foo")],
            "PlainIdentifier",
            8,
            "g:is#foo",
            var(ch('g'), true),
        );
        simple(
            flags,
            &[src(b"g:isnot#foo")],
            "PlainIdentifier",
            11,
            "g:isnot#foo",
            var(ch('g'), true),
        );
    }

    /// `is` and `isnot` are comparison operators, and the suffix that follows
    /// them is part of the operator rather than the start of a name.
    fn is_comparison(flags: c_int, advance: bool) {
        comparison(flags, advance, "is", "isnot", "Identical");
        for (text, len, kept, inv, ccs) in [
            ("is", 2, "is", false, "UseOption"),
            ("isnot", 5, "isnot", true, "UseOption"),
            ("is?", 3, "is?", false, "IgnoreCase"),
            ("isnot?", 6, "isnot?", true, "IgnoreCase"),
            ("is#", 3, "is#", false, "MatchCase"),
            ("isnot#", 6, "isnot#", true, "MatchCase"),
            ("is#foo", 3, "is#", false, "MatchCase"),
            ("isnot#foo", 6, "isnot#", true, "MatchCase"),
        ] {
            simple(
                flags,
                &[src(text.as_bytes())],
                "Comparison",
                len,
                kept,
                cmp_data("Identical", inv, ccs),
            );
        }
    }

    /// Without `kELFlagAllowFloat` a fractional part is not part of the
    /// number at all.
    fn numbers(flags: c_int) {
        for text in ["2.0", "2.0e5", "2.0e+5", "2.0e-5"] {
            simple(flags, &[src(text.as_bytes())], "Number", 1, "2", int(10, 2));
        }
    }

    /// The three characters that end a command.
    fn eoc(flags: c_int, advance: bool) {
        for text in ["|", "\0", "\n"] {
            single(flags, advance, "EOC", text, Tkd::None);
        }
    }

    // -- the six flag spellings --------------------------------------------

    #[test]
    fn scans_one_token_at_a_time() {
        let flags = 0;
        stable(flags, true);
        eoc(flags, true);
        scopes(flags, true);
        is_comparison(flags, true);
        numbers(flags);
    }

    #[test]
    fn peeking_reads_the_same_token_without_moving_the_cursor() {
        let flags = kELFlagPeek as c_int;
        stable(flags, false);
        eoc(flags, false);
        scopes(flags, false);
        is_comparison(flags, false);
        numbers(flags);
    }

    #[test]
    fn forbidding_scope_stops_the_name_before_the_colon() {
        let flags = kELFlagForbidScope as c_int;
        stable(flags, true);
        eoc(flags, true);
        is_comparison(flags, true);
        numbers(flags);

        simple(
            flags,
            &[src(b"g:")],
            "PlainIdentifier",
            1,
            "g",
            var(num(0), false),
        );
    }

    #[test]
    fn allowing_floats_takes_the_fractional_part() {
        let flags = kELFlagAllowFloat as c_int;
        stable(flags, true);
        eoc(flags, true);
        scopes(flags, true);
        is_comparison(flags, true);

        for (text, len, val) in [
            ("2.2", 3, 2.2),
            ("2.0e5", 5, 2e5),
            ("2.0e+5", 6, 2e5),
            ("2.0e-5", 6, 2e-5),
            ("2.500000e-5", 11, 2.5e-5),
            ("2.5555e2", 8, 2.5555e2),
            ("2.5555e+2", 9, 2.5555e2),
            ("2.5555e-2", 9, 2.5555e-2),
        ] {
            simple(
                flags,
                &[src(text.as_bytes())],
                "Number",
                len,
                text,
                flt(10, val),
            );
        }
        // Where the reader cut the line short, the exponent that is not on it
        // is not part of the number — and a cut that lands between `e` and its
        // digits leaves no float at all.
        simple(
            flags,
            &[cut(b"2.5e-5", 3)],
            "Number",
            3,
            "2.5",
            flt(10, 2.5),
        );
        simple(flags, &[cut(b"2.5e5", 4)], "Number", 1, "2", int(10, 2));
        simple(
            flags,
            &[cut(b"2.5e-50", 6)],
            "Number",
            6,
            "2.5e-5",
            flt(10, 2.5e-5),
        );
    }

    #[test]
    fn is_can_be_read_as_an_identifier_instead() {
        let flags = kELFlagIsNotCmp as c_int;
        stable(flags, true);
        eoc(flags, true);
        scopes(flags, true);
        numbers(flags);

        for (text, len, kept, autoload) in [
            ("is", 2, "is", false),
            ("isnot", 5, "isnot", false),
            ("is?", 2, "is", false),
            ("isnot?", 5, "isnot", false),
            ("is#", 3, "is#", true),
            ("isnot#", 6, "isnot#", true),
            ("is#foo", 6, "is#foo", true),
            ("isnot#foo", 9, "isnot#foo", true),
        ] {
            simple(
                flags,
                &[src(text.as_bytes())],
                "PlainIdentifier",
                len,
                kept,
                var(num(0), autoload),
            );
        }
    }

    #[test]
    fn forbidding_eoc_makes_the_three_end_characters_invalid() {
        let flags = kELFlagForbidEOC as c_int;
        stable(flags, true);
        scopes(flags, true);
        is_comparison(flags, true);
        numbers(flags);

        for text in ["|", "\0", "\n"] {
            single(
                flags,
                true,
                "Invalid",
                text,
                err("E15: Unexpected EOC character: %.*s"),
            );
        }
    }
}
