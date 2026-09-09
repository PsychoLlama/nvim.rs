//! What every kind of Vimscript value looks like through the four sinks that
//! render one: `:echo`, `string()`, `json_encode()` and `msgpackdump()`.
//!
//! The walk (`eval/typval_encode/walk.rs`) and its six `TypvalSink`s are the
//! largest reader of `TypVal`'s shape in the tree — `tv_clear` is one of
//! them — and until now the only thing pinning three of them was the
//! functional suite, which drives them through `eval()` and so cannot say
//! which sink answered. These cases call the entry points directly, over one
//! corpus, so a change to how a value is *stored* is caught by every
//! renderer at once and the failure names the value rather than the script
//! that built it.
//!
//! The corpus is one row per `VarType` plus the rows where the four disagree,
//! and the disagreements are the interesting part:
//!
//! - `:echo` prints a string's own bytes; `string()` quotes them, doubling
//!   the single quotes. At the *top level* `:echo` does the same for a
//!   funcref, so `echo function('tr')` is `tr` while `string()` is
//!   `function('tr')` — but a funcref nested in a list is `function('tr')`
//!   in both, because the special case lives in `encode_tv2echo` and not in
//!   the sink.
//! - a NULL string, list, dict and blob render exactly as their empty
//!   counterparts. Nothing downstream of a sink can tell the two apart,
//!   which is what lets so much of `eval/` hand out either.
//! - `json_encode()` refuses a funcref, a partial, an infinity and a NaN;
//!   `msgpackdump()` refuses the first two and packs the last two as
//!   ordinary IEEE doubles.
//! - a *self-referencing* container is where all four part company: `:echo`
//!   prints a back-reference marker and succeeds silently, `string()` prints
//!   a different marker, reports `E724` once and still succeeds, and the two
//!   binary encoders refuse outright with different messages.
//! - msgpack writes a Vimscript string as **bin** (`0xc4`) and a *dict key*
//!   as **str** (`0xa1`), and a blob as bin as well — so a round trip
//!   through msgpack cannot tell a string from a blob either.
//!
//! Every case needs a live editor, which Miri cannot start.

#![cfg(not(miri))]

use std::ffi::CStr;
use std::ptr;

use neovim::eval::encode::{
    encode_tv2echo, encode_tv2json, encode_tv2string, encode_vim_to_msgpack,
};
use neovim::eval::typval::{tv_clear, tv_list_free};
use neovim::memory::xfree;
use neovim::msgpack_rpc::packer::{packer_string_buffer, packer_take_string};
use neovim::types::{PackerBuffer, String_0, TypVal};

use crate::support::tv::{Payload, Pt, Tv};
use crate::support::{Editor, check_emsg, editor_lock, internalize};

/// The name the two refusing encoders put in their complaints. `json`'s is
/// fixed inside `encode_tv2json`; msgpack's is the caller's, so this is the
/// label `f_msgpackdump` builds for its first item — which makes the
/// messages below the exact text a user sees.
const MSGPACK_OBJNAME: &CStr = c"msgpackdump() argument, index 0";

/// `E724`, which `string()` reports once per dump and both binary encoders
/// report instead of an answer.
const E724: &str = "E724: unable to correctly dump variable with self-referencing container";

/// One corpus row: a value, and what each sink makes of it.
///
/// Each rendering is a pair of "what came back" and "what was reported on
/// the way", because two of the four can do both at once.
struct Row {
    /// What the value is, in the harness's model.
    tv: Tv,
    /// `encode_tv2echo`. It never reports anything, so there is no message.
    echo: &'static str,
    /// `encode_tv2string`.
    string: (&'static str, Option<String>),
    /// `encode_tv2json`. A refusal answers the *empty* string and reports.
    json: (&'static str, Option<String>),
    /// `encode_vim_to_msgpack`: the bytes, or the message of its refusal.
    msgpack: Result<Vec<u8>, String>,
}

/// A row whose four renderings are all silent successes.
fn row(tv: Tv, echo: &'static str, string: &'static str, json: &'static str, mp: &[u8]) -> Row {
    Row {
        tv,
        echo,
        string: (string, None),
        json: (json, None),
        msgpack: Ok(mp.to_vec()),
    }
}

/// A row both text sinks render and both binary ones refuse over a funcref,
/// which is every funcref and every partial. `path` is what the encoders
/// call the place they met it: `itself` at the top, `index 0` one level in.
fn funcref_row(tv: Tv, echo: &'static str, string: &'static str, path: &str) -> Row {
    let tail = "attempt to dump function reference";
    Row {
        tv,
        echo,
        string: (string, None),
        json: (
            "",
            Some(format!(
                "E474: Error while dumping encode_tv2json() argument, {path}: {tail}"
            )),
        ),
        msgpack: Err(format!(
            "E5004: Error while dumping {objname}, {path}: {tail}",
            objname = MSGPACK_OBJNAME.to_str().unwrap(),
        )),
    }
}

/// The corpus: one row per `VarType`, then the rows where the sinks part
/// company.
fn corpus() -> Vec<Row> {
    let self_ref = |what: &str| {
        Err(format!(
            "E5005: Unable to dump {objname}: container references itself in {what}",
            objname = MSGPACK_OBJNAME.to_str().unwrap(),
        ))
    };
    vec![
        // ------------------------------------------------------- scalars
        row(Tv::Int(0), "0", "0", "0", &[0x00]),
        row(Tv::Int(-1), "-1", "-1", "-1", &[0xff]),
        row(
            Tv::Int(i64::MAX),
            "9223372036854775807",
            "9223372036854775807",
            "9223372036854775807",
            &[0xcf, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
        ),
        // A float always renders with a fractional part, which is why
        // `string(1.0)` is not `1`.
        row(
            Tv::Float(0.0),
            "0.0",
            "0.0",
            "0.0",
            &[0xcb, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
        row(
            Tv::Float(1.5),
            "1.5",
            "1.5",
            "1.5",
            &[0xcb, 0x3f, 0xf8, 0, 0, 0, 0, 0, 0],
        ),
        row(Tv::Bool(true), "v:true", "v:true", "true", &[0xc3]),
        row(Tv::Bool(false), "v:false", "v:false", "false", &[0xc2]),
        row(Tv::Nil, "v:null", "v:null", "null", &[0xc0]),
        // ------------------------------------------------------- strings
        row(
            Tv::s("ab"),
            "ab",
            "'ab'",
            "\"ab\"",
            &[0xc4, 0x02, b'a', b'b'],
        ),
        // A quote doubles inside `string()`'s quotes and is ordinary to the
        // other three.
        row(
            Tv::s("a'b"),
            "a'b",
            "'a''b'",
            "\"a'b\"",
            &[0xc4, 0x03, b'a', b'\'', b'b'],
        ),
        // A newline survives `string()` literally and is escaped only by
        // JSON.
        row(
            Tv::s("a\nb"),
            "a\nb",
            "'a\nb'",
            "\"a\\nb\"",
            &[0xc4, 0x03, b'a', b'\n', b'b'],
        ),
        // A NULL string is an empty one to every sink.
        row(Tv::NullStr, "", "''", "\"\"", &[0xc4, 0x00]),
        row(Tv::s(""), "", "''", "\"\"", &[0xc4, 0x00]),
        // --------------------------------------------------- containers
        row(Tv::List(vec![]), "[]", "[]", "[]", &[0x90]),
        row(Tv::NullList, "[]", "[]", "[]", &[0x90]),
        row(
            Tv::List(vec![Tv::Int(1), Tv::Float(2.5), Tv::s("x")]),
            "[1, 2.5, 'x']",
            "[1, 2.5, 'x']",
            "[1, 2.5, \"x\"]",
            &[
                0x93, 0x01, 0xcb, 0x40, 0x04, 0, 0, 0, 0, 0, 0, 0xc4, 0x01, b'x',
            ],
        ),
        row(Tv::Dict(vec![]), "{}", "{}", "{}", &[0x80]),
        row(Tv::NullDict, "{}", "{}", "{}", &[0x80]),
        // A dict key is msgpack **str** where the value beside it is bin.
        row(
            Tv::dict([("a", Tv::Int(1))]),
            "{'a': 1}",
            "{'a': 1}",
            "{\"a\": 1}",
            &[0x81, 0xa1, b'a', 0x01],
        ),
        // Nesting: the two text sinks agree below the top level, and a
        // container inside a container is rendered by the same sink.
        row(
            Tv::List(vec![
                Tv::List(vec![Tv::Int(1)]),
                Tv::dict([("a", Tv::List(vec![Tv::Int(2)]))]),
            ]),
            "[[1], {'a': [2]}]",
            "[[1], {'a': [2]}]",
            "[[1], {\"a\": [2]}]",
            &[0x92, 0x91, 0x01, 0x81, 0xa1, b'a', 0x91, 0x02],
        ),
        // ---------------------------------------------------------- blobs
        row(
            Tv::Blob(vec![0x00, 0xff, 0x1a]),
            "0z00FF1A",
            "0z00FF1A",
            "[0, 255, 26]",
            &[0xc4, 0x03, 0x00, 0xff, 0x1a],
        ),
        row(Tv::Blob(vec![]), "0z", "0z", "[]", &[0xc4, 0x00]),
        row(Tv::NullBlob, "0z", "0z", "[]", &[0xc4, 0x00]),
        // --------------------------------------------- funcrefs, refused
        // The top-level echo special case: a funcref echoes its bare name.
        funcref_row(Tv::Func(b"tr".to_vec()), "tr", "function('tr')", "itself"),
        // Nested, the special case does not apply and both text sinks say
        // `function('tr')`.
        funcref_row(
            Tv::List(vec![Tv::Func(b"tr".to_vec())]),
            "[function('tr')]",
            "[function('tr')]",
            "index 0",
        ),
        funcref_row(
            Tv::Partial(Box::new(Pt {
                value: b"tr".to_vec(),
                auto: false,
                args: vec![Tv::Int(1), Tv::Int(2)],
                dict: None,
            })),
            "function('tr', [1, 2])",
            "function('tr', [1, 2])",
            "itself",
        ),
        // A partial's bound dict renders after its name, in the place its
        // arguments would take.
        funcref_row(
            Tv::Partial(Box::new(Pt {
                value: b"tr".to_vec(),
                auto: false,
                args: vec![],
                dict: Some(Tv::dict([("a", Tv::Int(1))])),
            })),
            "function('tr', {'a': 1})",
            "function('tr', {'a': 1})",
            "itself",
        ),
        // ------------------------------------------ the two JSON refusals
        Row {
            tv: Tv::Float(f64::INFINITY),
            echo: "str2float('inf')",
            string: ("str2float('inf')", None),
            json: (
                "",
                Some("E474: Unable to represent infinity in JSON".into()),
            ),
            // msgpack has no opinion: it is an IEEE double either way.
            msgpack: Ok(vec![0xcb, 0x7f, 0xf0, 0, 0, 0, 0, 0, 0]),
        },
        Row {
            tv: Tv::Float(f64::NAN),
            echo: "str2float('nan')",
            string: ("str2float('nan')", None),
            json: (
                "",
                Some("E474: Unable to represent NaN value in JSON".into()),
            ),
            msgpack: Ok(vec![0xcb, 0x7f, 0xf8, 0, 0, 0, 0, 0, 0]),
        },
        // ----------------------------------------- self-reference, where
        // all four sinks differ and nowhere else.
        Row {
            tv: Tv::List(vec![Tv::Int(1), Tv::Cycle(0)]),
            echo: "[1, [...@0]]",
            string: ("[1, {E724@0}]", Some(E724.into())),
            // Not a typo and not valid JSON: the JSON sink reports the
            // cycle and then *omits* the value, leaving the separator it
            // had already written. Upstream does the same.
            json: ("[1, ]", Some(E724.into())),
            msgpack: self_ref("index 1"),
        },
        Row {
            tv: Tv::Dict(vec![
                (b"a".to_vec(), Tv::Int(1)),
                (b"self".to_vec(), Tv::Cycle(0)),
            ]),
            echo: "{'a': 1, 'self': {...@0}}",
            string: ("{'a': 1, 'self': {E724@0}}", Some(E724.into())),
            json: ("{\"a\": 1, \"self\": }", Some(E724.into())),
            msgpack: self_ref("key 'self'"),
        },
    ]
}

// ------------------------------------------------------------ the sinks

/// `encode_tv2echo`, with the length it reports checked against the bytes.
///
/// # Safety
/// `tv` is live.
unsafe fn echo(tv: *mut TypVal) -> String {
    let mut len = 0;
    let text = unsafe { internalize(encode_tv2echo(tv, &raw mut len)) };
    assert_eq!(len, text.len(), "the length reported is the length written");
    text
}

/// `encode_tv2string`.
///
/// # Safety
/// As [`echo`].
unsafe fn string(tv: *mut TypVal) -> String {
    let mut len = 0;
    let text = unsafe { internalize(encode_tv2string(tv, &raw mut len)) };
    assert_eq!(len, text.len());
    text
}

/// `encode_tv2json`. A refusal answers an empty string, not a null one.
///
/// # Safety
/// As [`echo`].
unsafe fn json(tv: *mut TypVal) -> String {
    let mut len = 0;
    let text = unsafe { internalize(encode_tv2json(tv, &raw mut len)) };
    assert_eq!(len, text.len());
    text
}

/// `encode_vim_to_msgpack` through the packer `msgpackdump()` uses, or
/// `None` when it refused.
///
/// # Safety
/// As [`echo`].
unsafe fn msgpack(tv: *mut TypVal) -> Option<Vec<u8>> {
    let mut buffer: PackerBuffer = packer_string_buffer();
    let ok = unsafe { encode_vim_to_msgpack(&raw mut buffer, tv, MSGPACK_OBJNAME.as_ptr()) };
    let packed: String_0 = packer_take_string(&buffer);
    let bytes = unsafe { packed.as_bytes() }.to_vec();
    unsafe { xfree(packed.data().cast()) };
    (ok != 0).then_some(bytes)
}

// ------------------------------------------------------------- the cases

/// Run `f` over each row's built value, naming the row in any failure.
///
/// The editor lock is held for the whole sweep: every sink writes into the
/// message history, and `string()`'s `E724` latch (`did_echo_string_emsg`)
/// is a global only one case at a time may move.
fn each_row(mut f: impl FnMut(&Editor, &Row, *mut TypVal)) {
    let editor = editor_lock();
    for row in corpus() {
        // SAFETY: each value is this iteration's own and is cleared before
        // the next is built. A self-referencing row holds the reference the
        // clear releases, so the container behind it stays alive with the
        // one it holds itself -- as it does for the editor, whose garbage
        // collector is what reclaims those.
        unsafe {
            let mut tv = row.tv.build();
            f(&editor, &row, &raw mut tv);
            tv_clear(&raw mut tv);
        }
    }
}

/// `:echo`'s rendering of every row, and the message history not moving:
/// echoing a value reports nothing, a self-reference included.
#[test]
fn echo_renders_every_kind_of_value_without_complaining() {
    each_row(|editor, row, tv| {
        let got = check_emsg(editor, || unsafe { echo(tv) }, None);
        assert_eq!(got, row.echo, "echoing {:?}", row.tv);
    });
}

/// `string()`'s rendering, and the one message it reports.
#[test]
fn string_quotes_every_kind_of_value_and_marks_a_cycle() {
    each_row(|editor, row, tv| {
        let (want, msg) = (row.string.0, row.string.1.as_deref());
        let got = check_emsg(editor, || unsafe { string(tv) }, msg);
        assert_eq!(got, want, "stringifying {:?}", row.tv);
    });
}

/// `json_encode()`'s rendering, and what it refuses.
#[test]
fn json_encodes_what_it_can_and_names_what_it_cannot() {
    each_row(|editor, row, tv| {
        let (want, msg) = (row.json.0, row.json.1.as_deref());
        let got = check_emsg(editor, || unsafe { json(tv) }, msg);
        assert_eq!(got, want, "encoding {:?}", row.tv);
    });
}

/// `msgpackdump()`'s bytes, and what it refuses.
#[test]
fn msgpack_packs_every_kind_of_value_it_accepts() {
    each_row(|editor, row, tv| match &row.msgpack {
        Ok(want) => {
            let got = check_emsg(editor, || unsafe { msgpack(tv) }, None);
            assert_eq!(
                got.as_deref(),
                Some(want.as_slice()),
                "packing {:?}: {:02x?}",
                row.tv,
                got
            );
        }
        Err(msg) => {
            let got = check_emsg(editor, || unsafe { msgpack(tv) }, Some(msg));
            assert_eq!(got, None, "packing {:?} should have refused", row.tv);
        }
    });
}

/// The one thing no row can say: `string()` reports `E724` **once** per dump
/// however many times the cycle is met — the whole point of the
/// `did_echo_string_emsg` latch — and the latch is reset afterwards, so the
/// *next* `string()` reports it again.
#[test]
fn a_cycle_met_twice_is_reported_once_per_dump() {
    let editor = editor_lock();
    // SAFETY: the list is this case's own. It holds two references to
    // itself, so it is taken apart by `tv_list_free` rather than by
    // releasing the outside one.
    unsafe {
        let tv = Tv::List(vec![Tv::Cycle(0), Tv::Cycle(0)]).build();
        let at = &raw const tv as *mut TypVal;

        let got = check_emsg(&editor, || string(at), Some(E724));
        assert_eq!(got, "[{E724@0}, {E724@0}]", "both cycles were marked");

        // The second dump reports it again: the latch is per dump, not per
        // value.
        let got = check_emsg(&editor, || string(at), Some(E724));
        assert_eq!(got, "[{E724@0}, {E724@0}]");

        // `:echo` never reports it at all, however many times it is met.
        let got = check_emsg(&editor, || echo(at), None);
        assert_eq!(got, "[[...@0], [...@0]]");

        tv_list_free(tv.list());
    }
}

/// A NULL length pointer is accepted by all three text entry points — the
/// shape most callers use, and the one a `len`-carrying rewrite has to keep.
#[test]
fn the_length_is_optional() {
    let _editor = editor_lock();
    // SAFETY: the value is this case's own and is cleared.
    unsafe {
        let mut tv = Tv::List(vec![Tv::Int(1), Tv::s("x")]).build();
        let at = &raw mut tv;
        assert_eq!(internalize(encode_tv2echo(at, ptr::null_mut())), "[1, 'x']");
        assert_eq!(
            internalize(encode_tv2string(at, ptr::null_mut())),
            "[1, 'x']"
        );
        assert_eq!(
            internalize(encode_tv2json(at, ptr::null_mut())),
            "[1, \"x\"]"
        );
        tv_clear(at);
    }
}
