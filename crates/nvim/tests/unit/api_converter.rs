//! `test/unit/api/private_helpers_spec.lua`, `describe('vim_to_object')`.
//!
//! The last spec built on `test/unit/eval/testutil.lua`, and the reason that
//! harness outlived `typval_spec`. It is a conversion table between two
//! value types — [`Tv`] going in, [`Obj`] coming out — and the whole of what
//! it says is where the two disagree:
//!
//! - the API has **no NULL container**: a NULL list becomes an empty array
//!   and a NULL dict an empty dict;
//! - the API has **no funcref**: a `VAR_FUNC` or `VAR_PARTIAL` becomes nil;
//! - and a **cycle** becomes nil at the point it closes, one level down,
//!   rather than looping.
//!
//! Every case needs a live editor, which Miri cannot start.

#![cfg(not(miri))]

use neovim::eval::typval::tv_clear;
use neovim::types::Object;

use crate::support::alloc::AllocLog;
use crate::support::tv::{Obj, Pt, Tv};

/// Convert `value` and read the answer back, releasing both.
///
/// # Safety
/// The editor must be up.
unsafe fn convert(value: Tv) -> Obj {
    // SAFETY: the value and the object are this call's own.
    unsafe {
        let mut tv = value.build();
        let object = Object::from(&tv);
        let read = crate::support::tv::read_object(&raw const object);
        drop(object);
        tv_clear(&mut tv);
        read
    }
}

/// The rows that convert to the same shape they went in as.
#[test]
fn every_scalar_and_container_converts_to_its_own_shape() {
    let _log = AllocLog::start();
    // SAFETY: `convert` owns everything it makes.
    unsafe {
        assert_eq!(convert(Tv::Bool(true)), Obj::Bool(true));
        assert_eq!(convert(Tv::Bool(false)), Obj::Bool(false));
        assert_eq!(convert(Tv::Nil), Obj::Nil);
        // A `VAR_FLOAT` stays a float; only `int(n)` is an integer.
        assert_eq!(convert(Tv::Float(1.0)), Obj::Float(1.0));
        assert_eq!(convert(Tv::Float(-1.5)), Obj::Float(-1.5));
        assert_eq!(convert(Tv::Int(10)), Obj::Int(10));
        assert_eq!(convert(Tv::s("")), Obj::s(""));
        assert_eq!(convert(Tv::s("foobar")), Obj::s("foobar"));

        assert_eq!(convert(Tv::Dict(vec![])), Obj::Dict(vec![]));
        assert_eq!(
            convert(Tv::dict([
                ("test", Tv::Float(10.0)),
                ("test2", Tv::Bool(true)),
                ("test3", Tv::s("test")),
            ])),
            Obj::dict([
                ("test", Obj::Float(10.0)),
                ("test2", Obj::Bool(true)),
                ("test3", Obj::s("test")),
            ])
        );
        assert_eq!(
            convert(Tv::dict([
                ("test", Tv::Dict(vec![])),
                ("test2", Tv::List(vec![Tv::Float(1.0), Tv::Float(2.0)])),
            ])),
            Obj::dict([
                ("test", Obj::Dict(vec![])),
                ("test2", Obj::Array(vec![Obj::Float(1.0), Obj::Float(2.0)])),
            ])
        );

        assert_eq!(convert(Tv::List(vec![])), Obj::Array(vec![]));
        assert_eq!(
            convert(Tv::List(vec![
                Tv::Float(1.0),
                Tv::Float(2.0),
                Tv::s("test"),
                Tv::s("foo"),
            ])),
            Obj::Array(vec![
                Obj::Float(1.0),
                Obj::Float(2.0),
                Obj::s("test"),
                Obj::s("foo"),
            ])
        );
        assert_eq!(
            convert(Tv::List(vec![
                Tv::Dict(vec![]),
                Tv::dict([
                    ("test", Tv::Dict(vec![])),
                    ("test3", Tv::dict([("test4", Tv::Bool(true))])),
                ]),
            ])),
            Obj::Array(vec![
                Obj::Dict(vec![]),
                Obj::dict([
                    ("test", Obj::Dict(vec![])),
                    ("test3", Obj::dict([("test4", Obj::Bool(true))])),
                ]),
            ])
        );
    }
}

/// A container that reaches itself converts to nil at the point the cycle
/// closes — the API's value type has no way to hold one, and the walk must
/// not follow it.
#[test]
fn a_cycle_converts_to_nil_where_it_closes() {
    let _log = AllocLog::start();
    // SAFETY: `convert` owns everything it makes, and `Tv::Cycle` builds a
    // real reference that `tv_clear` releases.
    unsafe {
        // One level: `d.dct = d`.
        assert_eq!(
            convert(Tv::Dict(vec![(b"dct".to_vec(), Tv::Cycle(0))])),
            Obj::dict([("dct", Obj::Nil)])
        );
        // And `l[1] = l`.
        assert_eq!(
            convert(Tv::List(vec![Tv::Cycle(0)])),
            Obj::Array(vec![Obj::Nil])
        );

        // Two levels, through a list: `d.dct = [d]`.
        assert_eq!(
            convert(Tv::Dict(vec![
                (b"test".to_vec(), Tv::Bool(true)),
                (b"dict".to_vec(), Tv::Nil),
                (b"dct".to_vec(), Tv::List(vec![Tv::Cycle(0)])),
            ])),
            Obj::dict([
                ("dct", Obj::Array(vec![Obj::Nil])),
                ("dict", Obj::Nil),
                ("test", Obj::Bool(true)),
            ])
        );
        // Two levels, through a dict: `d.dct = {dctin = d}`.
        assert_eq!(
            convert(Tv::Dict(vec![
                (b"test".to_vec(), Tv::Bool(true)),
                (b"dict".to_vec(), Tv::Nil),
                (
                    b"dct".to_vec(),
                    Tv::Dict(vec![(b"dctin".to_vec(), Tv::Cycle(0))])
                ),
            ])),
            Obj::dict([
                ("dct", Obj::dict([("dctin", Obj::Nil)])),
                ("dict", Obj::Nil),
                ("test", Obj::Bool(true)),
            ])
        );
        // Two levels, list in list: `l[1] = [l]`.
        assert_eq!(
            convert(Tv::List(vec![Tv::List(vec![Tv::Cycle(0)])])),
            Obj::Array(vec![Obj::Array(vec![Obj::Nil])])
        );
        // Two levels, dict in list: `l[1] = {lst = l}`.
        assert_eq!(
            convert(Tv::List(vec![
                Tv::Dict(vec![(b"lst".to_vec(), Tv::Cycle(0))]),
                Tv::Bool(true),
                Tv::Bool(false),
                Tv::s("ttest"),
            ])),
            Obj::Array(vec![
                Obj::dict([("lst", Obj::Nil)]),
                Obj::Bool(true),
                Obj::Bool(false),
                Obj::s("ttest"),
            ])
        );
    }
}

/// A NULL container is not a distinct value on the API side: it converts to
/// the empty one.
#[test]
fn a_null_container_converts_to_an_empty_one() {
    let _log = AllocLog::start();
    // SAFETY: neither value owns anything.
    unsafe {
        assert_eq!(convert(Tv::NullList), Obj::Array(vec![]));
        assert_eq!(convert(Tv::NullDict), Obj::Dict(vec![]));
    }
}

/// `itp('regression: partials in a list')`, spec line 105.
///
/// A partial has no API representation, so it converts to nil — and the
/// regression was that the walk stopped there instead of carrying on to the
/// item beside it.
#[test]
fn a_partial_in_a_list_converts_to_nil_without_ending_the_walk() {
    let _log = AllocLog::start();
    // SAFETY: `convert` owns everything it makes.
    unsafe {
        let partial = Tv::Partial(Box::new(Pt {
            value: b"printf".to_vec(),
            auto: false,
            args: vec![Tv::s("%s")],
            dict: Some(Tv::dict([("v", Tv::Float(1.0))])),
        }));
        assert_eq!(
            convert(Tv::List(vec![partial, Tv::Dict(vec![])])),
            Obj::Array(vec![Obj::Nil, Obj::Dict(vec![])])
        );
    }
}

// ---------------------------------------------------------------------------
// The Lua direction: `lua/converter/` over the typed `Stack`

mod lua {
    use core::ffi::c_int;

    use neovim::eval::typval::tv_clear;
    use neovim::lua::converter::{
        pop_dict, pop_integer, pop_object, pop_string, pop_typval, push_object, push_typval,
    };
    use neovim::lua::executor::get_global_lstate;
    use neovim::lua::stack::{At, Kind, Raise, Stack};
    use neovim::types::{Object, lua_State};

    use crate::support::tv::{Obj, Tv, read, read_object};
    use crate::support::{Editor, check_emsg, editor_lock};

    /// The editor's Lua stack. The token is the lock that makes this the
    /// only `Stack` over the main state's frame.
    fn lua(_editor: &Editor) -> Stack {
        // SAFETY: the editor is up (the lock brought it up), its state lives
        // as long as the process, and the lock serialises every case.
        unsafe { Stack::from_raw(get_global_lstate()) }
    }

    /// Run `chunk` and leave its one result on top.
    fn eval(stack: &mut Stack, chunk: &str) {
        stack
            .load(chunk.as_bytes(), c"=converter")
            .expect("the chunk compiles");
        stack.pcall(0, 1, None).expect("the chunk runs");
    }

    /// The Lua value `chunk` answers, as an api object.
    fn object_of(stack: &mut Stack, chunk: &str) -> Result<Obj, String> {
        eval(stack, chunk);
        match pop_object(stack, false) {
            // SAFETY: the object is this call's own.
            Ok(object) => Ok(unsafe { read_object(&raw const object) }),
            Err(e) => Err(e.to_string()),
        }
    }

    #[test]
    fn a_lua_value_converts_to_an_object_and_back() {
        let editor = editor_lock();
        let mut stack = lua(&editor);
        let top = stack.top();
        let chunk = "return { s = 'a\\0b', n = 3, f = 1.5, t = true, \
                     l = { 1, 'two', { x = vim.NIL } }, e = vim.empty_dict(), a = {} }";
        let expected = Obj::dict([
            ("a", Obj::Array(vec![])),
            ("e", Obj::Dict(vec![])),
            ("f", Obj::Float(1.5)),
            (
                "l",
                Obj::Array(vec![
                    Obj::Int(1),
                    Obj::s("two"),
                    Obj::dict([("x", Obj::Nil)]),
                ]),
            ),
            ("n", Obj::Int(3)),
            ("s", Obj::s(b"a\0b")),
            ("t", Obj::Bool(true)),
        ]);
        assert_eq!(object_of(&mut stack, chunk), Ok(expected.clone()));
        assert_eq!(stack.top(), top);

        // And back: the push leaves one value, which converts to the same.
        eval(&mut stack, chunk);
        let object = pop_object(&mut stack, false).expect("it converted once");
        push_object(&mut stack, &object, false);
        assert_eq!(stack.top(), At::arg(top.index() + 1));
        // An empty dictionary keeps its marker through the push.
        stack.get_field(At::TOP, c"e");
        assert!(stack.get_metatable(At::TOP));
        stack.pop(2);
        let again = pop_object(&mut stack, false).expect("the push converts back");
        // SAFETY: both objects are this case's own.
        assert_eq!(unsafe { read_object(&raw const again) }, expected);
        assert_eq!(stack.top(), top);
    }

    #[test]
    fn a_vimscript_value_converts_to_lua_and_back() {
        let editor = editor_lock();
        let mut stack = lua(&editor);
        let top = stack.top();
        let value = Tv::dict([
            (
                "list",
                Tv::List(vec![Tv::Int(1), Tv::s("x"), Tv::Float(2.5)]),
            ),
            ("dict", Tv::Dict(vec![])),
            ("yes", Tv::Bool(true)),
        ]);
        // SAFETY: the value is this case's own and released below.
        unsafe {
            let mut tv = value.build();
            assert!(push_typval(&mut stack, &tv, false));
            assert_eq!(stack.kind(At::TOP), Kind::Table);
            let mut back = pop_typval(&mut stack).expect("the push converts back");
            // Compared as api objects, whose dictionaries read back sorted.
            let (sent, got) = (Object::from(&tv), Object::from(&back));
            assert_eq!(read_object(&raw const got), read_object(&raw const sent));
            assert_eq!(stack.top(), top);
            tv_clear(&mut tv);
            tv_clear(&mut back);
        }
    }

    /// A table that contains itself becomes a list that contains itself, one
    /// shared container rather than a descent that never ends.
    #[test]
    fn a_cyclic_table_converts_to_a_shared_list() {
        let editor = editor_lock();
        let mut stack = lua(&editor);
        let top = stack.top();
        eval(&mut stack, "local t = { 1 }; t[2] = t; return t");
        let mut tv = pop_typval(&mut stack).expect("a cycle converts");
        // SAFETY: the value is this case's own.
        unsafe {
            assert_eq!(
                read(&raw const tv),
                Tv::List(vec![Tv::Int(1), Tv::Cycle(0)])
            );
            tv_clear(&mut tv);
        }
        assert_eq!(stack.top(), top);
    }

    /// Nesting past what the Lua stack can hold is refused, and the refusal
    /// leaves the stack as balanced as a success does.
    #[test]
    fn a_table_nested_past_the_stack_is_refused() {
        let editor = editor_lock();
        let mut stack = lua(&editor);
        let top = stack.top();
        // Upstream's texts: the classifier is what runs out of room first.
        let deep = "local t = {} for _ = 1, 20000 do t = { t } end return t";
        let refused = check_emsg(
            &editor,
            || object_of(&mut stack, deep),
            Some("E1502: Lua failed to grow stack to 4001"),
        );
        assert_eq!(refused, Err("Cannot convert given Lua table".to_string()));
        assert_eq!(stack.top(), top);
        eval(&mut stack, deep);
        let refused = check_emsg(
            &editor,
            || pop_typval(&mut stack),
            Some(
                "E5100: Cannot convert given Lua table: table should contain either only \
                 integer keys or only string keys",
            ),
        );
        assert!(refused.is_none());
        assert_eq!(stack.top(), top);
    }

    /// Every leaf pop takes exactly its one value, refused or not -- the
    /// dictionary's refusal of a value inside it included.
    #[test]
    fn a_refused_pop_takes_exactly_its_value() {
        let editor = editor_lock();
        let mut stack = lua(&editor);
        stack.push_integer(7);
        let top = stack.top();
        stack.push_number(1.5);
        let refused = pop_integer(&mut stack).expect_err("not integral");
        assert_eq!(refused.to_string(), "Number is not integral");
        assert_eq!(stack.top(), top);
        stack.push_number(1.0);
        assert!(pop_string(&mut stack).is_err());
        assert_eq!(stack.top(), top);
        eval(&mut stack, "return { a = function() end }");
        let refused = pop_dict(&mut stack, false).expect_err("a function has no api image");
        assert_eq!(refused.to_string(), "Cannot convert given Lua type");
        assert_eq!(stack.top(), top);
        // The value below is untouched.
        assert_eq!(stack.to_integer(At::TOP), 7);
        stack.pop(1);
    }

    /// A C function raising through `Stack::entry`.
    unsafe extern "C-unwind" fn raises(state: *mut lua_State) -> c_int {
        // SAFETY: the state Lua called this function on.
        unsafe {
            Stack::entry(state, |stack| {
                let owned = String::from("owned by the body");
                match stack.kind(At::arg(1)) {
                    Kind::Nil => Err(Raise::message(owned)),
                    Kind::Boolean => Err(Raise::argument(1, "invalid buffer handle: 999")),
                    _ => stack.check_integer(At::arg(1)).map(|_| 0),
                }
            })
        }
    }

    /// The error each way of raising leaves, through `pcall`.
    #[test]
    fn a_raise_reaches_lua_with_its_text() {
        let editor = editor_lock();
        let mut stack = lua(&editor);
        let top = stack.top();
        let mut raised = |arg: &str| {
            stack.push_function(raises);
            stack.set_global(c"_converter_raises");
            eval(
                &mut stack,
                &format!("return select(2, pcall(_converter_raises, {arg}))"),
            );
            let text = stack.string(At::TOP).map(<[u8]>::to_vec);
            stack.pop(1);
            text.map(|t| String::from_utf8(t).expect("UTF-8"))
        };
        assert_eq!(raised("nil").as_deref(), Some("owned by the body"));
        assert_eq!(
            raised("true").as_deref(),
            Some("bad argument #1 to '?' (invalid buffer handle: 999)")
        );
        assert_eq!(
            raised("'x'").as_deref(),
            Some("bad argument #1 to '?' (number expected, got string)")
        );
        assert_eq!(raised("1"), None);
        stack.push_nil();
        stack.set_global(c"_converter_raises");
        assert_eq!(stack.top(), top);
    }
}
