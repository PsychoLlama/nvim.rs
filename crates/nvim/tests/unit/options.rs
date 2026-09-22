//! The option layer: what a value round-trips as, and what a scope means.
//!
//! `option/` and `options/` had no in-crate test of any kind — the tree's
//! 311 option cells were pinned only by the functional and old suites,
//! which reach them through `:set` and read them back through `&opt`, and
//! so cannot tell "the option table holds the right bytes" from "the
//! Vimscript layer happens to print the right thing".
//!
//! That matters now because the storage is about to change under all of
//! it: the string options are raw `char *` cells today and become owned
//! strings in a struct, which is a rewrite with no visible output of its
//! own. These cases are the before-and-after.
//!
//! Everything here goes through the *public* surface, because `option/` is
//! `pub(crate)`: [`nvim_get_option_value`]/[`nvim_set_option_value`] for
//! the API path, and `do_cmdline_cmd` for the `:set` path. That is not a
//! compromise — those two are exactly the observers a rewrite has to keep
//! honest.

#![cfg(not(miri))]

use neovim::api::options::{nvim_get_option_value, nvim_set_option_value};
use neovim::ex_docmd::do_cmdline_cmd;
use neovim::types::{Error, KeyDict_option, Object, String_0};

use crate::support::{Editor, check_emsg, cstr, editor_lock};

/// The channel id an API call is attributed to. Zero is "the editor
/// itself", which is what `:set` would report too.
const SELF: u64 = 0;

/// The handle that stands for the current buffer or window.
const CURRENT: i32 = 0;

/// `opts` for a call that names no scope, buffer or window.
fn any() -> KeyDict_option {
    KeyDict_option::default()
}

/// `opts` naming an explicit scope: `"global"` or `"local"`.
fn scoped(scope: &str) -> KeyDict_option {
    KeyDict_option {
        scope: Some(String_0::from(scope)),
        ..KeyDict_option::default()
    }
}

/// `opts` naming the current buffer, which is how an API caller asks for a
/// buffer-local value without going through `:setlocal`. Handle 0 is the
/// API's spelling of "the current one".
fn on_current_buf() -> KeyDict_option {
    KeyDict_option {
        buf: Some(CURRENT),
        ..KeyDict_option::default()
    }
}

/// `opts` naming the current window. See [`on_current_buf`].
fn on_current_win() -> KeyDict_option {
    KeyDict_option {
        win: Some(CURRENT),
        ..KeyDict_option::default()
    }
}

/// Read option `name` at whatever scope `opts` names.
fn get(_editor: &Editor, name: &str, mut opts: KeyDict_option) -> Result<Object, Error> {
    // SAFETY: the name owns its bytes and `opts` is this frame's own.
    unsafe { nvim_get_option_value(String_0::from(name), &raw mut opts) }
}

/// Write option `name` at whatever scope `opts` names.
fn set(_editor: &Editor, name: &str, value: Object, mut opts: KeyDict_option) -> Result<(), Error> {
    // SAFETY: as `get`; `value` owns its payload.
    unsafe { nvim_set_option_value(SELF, String_0::from(name), value, &raw mut opts) }
}

/// The text of a string option, or a panic naming what came back instead.
#[track_caller]
fn text(got: Result<Object, Error>) -> String {
    let value = got.expect("the option exists");
    let bytes = value
        .as_string()
        .unwrap_or_else(|| panic!("expected a string, got {value:?}"))
        .as_bytes();
    String::from_utf8_lossy(bytes).into_owned()
}

/// The number of a numeric option.
#[track_caller]
fn number(got: Result<Object, Error>) -> i64 {
    let value = got.expect("the option exists");
    value
        .as_integer()
        .unwrap_or_else(|| panic!("expected a number, got {value:?}"))
}

/// The state of a boolean option.
#[track_caller]
fn flag(got: Result<Object, Error>) -> bool {
    let value = got.expect("the option exists");
    value
        .as_boolean()
        .unwrap_or_else(|| panic!("expected a boolean, got {value:?}"))
}

/// Run a command line, as a mapping would.
fn run(_editor: &Editor, line: &str) {
    let text = cstr(line);
    // SAFETY: `text` is NUL-terminated and outlives the call.
    let _ = do_cmdline_cmd(&text);
}

/// The options a case wrote, put back when it ends.
///
/// There is one editor per test process, so an option a case changes is
/// visible to every other case — including the `ex_docmd` and `statusline`
/// ones next door, which read `'statusline'` and `'shell'`. Each entry is
/// restored through the same API the case wrote it with, so the
/// restoration exercises no path the case did not.
struct Saved<'a> {
    editor: &'a Editor,
    /// Name, the value it had, and the scope to put it back at.
    entries: Vec<(String, Object, Option<&'static str>)>,
}

impl<'a> Saved<'a> {
    fn new(editor: &'a Editor) -> Saved<'a> {
        Saved {
            editor,
            entries: Vec::new(),
        }
    }

    /// Remember `name`'s current value at `scope` (`None` for the default
    /// scope), so the drop can write it back.
    fn keep(&mut self, name: &str, scope: Option<&'static str>) {
        let opts = match scope {
            None => any(),
            Some(scope) => scoped(scope),
        };
        let value = get(self.editor, name, opts).expect("the option exists");
        self.entries.push((name.to_string(), value, scope));
    }
}

impl Drop for Saved<'_> {
    fn drop(&mut self) {
        for (name, value, scope) in self.entries.drain(..).rev() {
            let opts = match scope {
                None => any(),
                Some(scope) => scoped(scope),
            };
            let _ = set(self.editor, &name, value, opts);
        }
    }
}

// ---------------------------------------------------------------------------
// Round trips, one per value kind and one per scope

#[test]
fn a_global_string_option_round_trips_through_the_api() {
    let editor = editor_lock();
    let mut saved = Saved::new(&editor);
    saved.keep("titlestring", None);

    set(
        &editor,
        "titlestring",
        Object::literal("nvim.rs — %f"),
        any(),
    )
    .expect("a free-form string is accepted");
    assert_eq!(text(get(&editor, "titlestring", any())), "nvim.rs — %f");

    // The empty string is a value, not an absence: it comes back as the
    // empty string rather than as nil or as the default.
    set(&editor, "titlestring", Object::literal(""), any()).expect("the empty string is a value");
    assert_eq!(text(get(&editor, "titlestring", any())), "");

    // And `:set` writes the same cell the API reads.
    run(&editor, "set titlestring=through-set");
    assert_eq!(text(get(&editor, "titlestring", any())), "through-set");
}

#[test]
fn a_buffer_local_string_option_is_read_off_the_buffer_it_names() {
    let editor = editor_lock();
    let mut saved = Saved::new(&editor);
    saved.keep("commentstring", None);

    set(&editor, "commentstring", Object::literal("# %s"), any()).expect("a buffer-local write");
    assert_eq!(text(get(&editor, "commentstring", any())), "# %s");
    // Naming the buffer explicitly reaches the same storage.
    assert_eq!(
        text(get(&editor, "commentstring", on_current_buf())),
        "# %s"
    );
    // `:setlocal` and the API write the same field.
    run(&editor, "setlocal commentstring=//\\ %s");
    assert_eq!(text(get(&editor, "commentstring", any())), "// %s");
}

#[test]
fn a_window_local_string_option_falls_back_to_the_global_until_it_is_set() {
    let editor = editor_lock();
    let mut saved = Saved::new(&editor);
    saved.keep("statusline", Some("global"));
    saved.keep("statusline", Some("local"));

    // 'statusline' is global-local: the local value is empty until it is
    // written, and reading without a scope answers the global one.
    set(&editor, "statusline", Object::literal(""), scoped("local"))
        .expect("clearing the local value");
    set(
        &editor,
        "statusline",
        Object::literal("global-line"),
        scoped("global"),
    )
    .expect("a global write");
    assert_eq!(text(get(&editor, "statusline", any())), "global-line");
    assert_eq!(text(get(&editor, "statusline", scoped("local"))), "");

    // A local write shadows it, for this window only, and the global value
    // is untouched underneath.
    set(
        &editor,
        "statusline",
        Object::literal("local-line"),
        scoped("local"),
    )
    .expect("a local write");
    assert_eq!(text(get(&editor, "statusline", any())), "local-line");
    assert_eq!(
        text(get(&editor, "statusline", on_current_win())),
        "local-line"
    );
    assert_eq!(
        text(get(&editor, "statusline", scoped("global"))),
        "global-line"
    );
}

#[test]
fn a_number_option_round_trips_at_both_scopes() {
    let editor = editor_lock();
    let mut saved = Saved::new(&editor);
    saved.keep("textwidth", None);
    saved.keep("scrolloff", Some("global"));
    saved.keep("scrolloff", Some("local"));

    // 'textwidth' is buffer-local and plain.
    set(&editor, "textwidth", Object::integer(72), any()).expect("a number write");
    assert_eq!(number(get(&editor, "textwidth", any())), 72);
    assert_eq!(number(get(&editor, "textwidth", on_current_buf())), 72);
    run(&editor, "setlocal textwidth=40");
    assert_eq!(number(get(&editor, "textwidth", any())), 40);

    // 'scrolloff' is global-local with -1 for "use the global one".
    set(&editor, "scrolloff", Object::integer(-1), scoped("local")).expect("unsetting the local");
    set(&editor, "scrolloff", Object::integer(7), scoped("global")).expect("a global write");
    assert_eq!(number(get(&editor, "scrolloff", any())), 7);
    set(&editor, "scrolloff", Object::integer(3), scoped("local")).expect("a local write");
    assert_eq!(number(get(&editor, "scrolloff", any())), 3);
    assert_eq!(number(get(&editor, "scrolloff", scoped("global"))), 7);
}

#[test]
fn a_boolean_option_round_trips_and_answers_a_boolean() {
    let editor = editor_lock();
    let mut saved = Saved::new(&editor);
    saved.keep("expandtab", None);
    saved.keep("wrap", None);
    saved.keep("hlsearch", None);

    // Buffer-local, window-local and global, one each — and the answer is
    // a `Boolean` object, not the 0/1 integer the option table holds.
    set(&editor, "expandtab", Object::boolean(true), any()).expect("a buffer-local flag");
    assert!(flag(get(&editor, "expandtab", on_current_buf())));
    run(&editor, "setlocal noexpandtab");
    assert!(!flag(get(&editor, "expandtab", any())));

    set(&editor, "wrap", Object::boolean(false), any()).expect("a window-local flag");
    assert!(!flag(get(&editor, "wrap", on_current_win())));

    set(&editor, "hlsearch", Object::boolean(true), any()).expect("a global flag");
    assert!(flag(get(&editor, "hlsearch", any())));
    run(&editor, "set nohlsearch");
    assert!(!flag(get(&editor, "hlsearch", any())));
}

// ---------------------------------------------------------------------------
// The shadow cells

#[test]
fn paste_saves_every_option_it_overrides_and_puts_them_back() {
    let editor = editor_lock();
    let mut saved = Saved::new(&editor);
    saved.keep("textwidth", None);
    saved.keep("expandtab", None);
    saved.keep("autoindent", None);
    saved.keep("paste", None);

    set(&editor, "textwidth", Object::integer(66), any()).expect("a number write");
    set(&editor, "expandtab", Object::boolean(true), any()).expect("a flag write");
    set(&editor, "autoindent", Object::boolean(true), any()).expect("a flag write");

    // `:set paste` forces its own values into the shadowed options...
    run(&editor, "set paste");
    assert!(flag(get(&editor, "paste", any())));
    assert_eq!(number(get(&editor, "textwidth", any())), 0);
    assert!(!flag(get(&editor, "expandtab", any())));
    assert!(!flag(get(&editor, "autoindent", any())));

    // ... and `:set nopaste` restores what the shadow cells kept, not the
    // options' defaults.
    run(&editor, "set nopaste");
    assert!(!flag(get(&editor, "paste", any())));
    assert_eq!(number(get(&editor, "textwidth", any())), 66);
    assert!(flag(get(&editor, "expandtab", any())));
    assert!(flag(get(&editor, "autoindent", any())));
}

#[test]
fn binary_shadows_the_four_options_it_overrides_per_buffer() {
    let editor = editor_lock();
    let mut saved = Saved::new(&editor);
    saved.keep("textwidth", None);
    saved.keep("wrapmargin", None);
    saved.keep("expandtab", None);
    saved.keep("binary", None);

    set(&editor, "textwidth", Object::integer(55), any()).expect("a number write");
    set(&editor, "wrapmargin", Object::integer(5), any()).expect("a number write");
    set(&editor, "expandtab", Object::boolean(true), any()).expect("a flag write");

    // 'binary' zeroes 'textwidth'/'wrapmargin' and clears 'expandtab'; the
    // `b_p_*_nobin` fields hold what they were.
    run(&editor, "setlocal binary");
    assert_eq!(number(get(&editor, "textwidth", any())), 0);
    assert_eq!(number(get(&editor, "wrapmargin", any())), 0);
    assert!(!flag(get(&editor, "expandtab", any())));

    run(&editor, "setlocal nobinary");
    assert_eq!(number(get(&editor, "textwidth", any())), 55);
    assert_eq!(number(get(&editor, "wrapmargin", any())), 5);
    assert!(flag(get(&editor, "expandtab", any())));
}

// ---------------------------------------------------------------------------
// Refusals

#[test]
fn a_did_set_callback_refuses_a_value_and_leaves_the_option_alone() {
    let editor = editor_lock();
    let mut saved = Saved::new(&editor);
    saved.keep("backspace", None);

    set(&editor, "backspace", Object::literal("indent,eol"), any()).expect("a valid value");

    // 'backspace' has a `did_set_*` that validates its word list, and a
    // refusal is an `Err` carrying the editor's own message — not a
    // silently accepted value.
    let refused = set(&editor, "backspace", Object::literal("bogus"), any())
        .expect_err("an invalid word is refused");
    assert!(
        refused.to_string().contains("E474"),
        "expected E474, got {refused}"
    );
    assert_eq!(text(get(&editor, "backspace", any())), "indent,eol");

    // The `:set` path reports the same refusal as a message.
    check_emsg(
        &editor,
        || run(&editor, "set backspace=bogus"),
        Some("E474: Invalid argument: backspace=bogus"),
    );
    assert_eq!(text(get(&editor, "backspace", any())), "indent,eol");
}

#[test]
fn an_unknown_option_is_a_refusal_and_not_a_default() {
    let editor = editor_lock();
    let unknown = get(&editor, "no_such_option_at_all", any())
        .expect_err("an option that does not exist is refused");
    assert!(
        unknown.to_string().contains("no_such_option_at_all"),
        "the message names the option: {unknown}"
    );

    let wrong_kind = set(&editor, "textwidth", Object::literal("not a number"), any())
        .expect_err("a string is not a number");
    assert!(
        wrong_kind.to_string().contains("expected number"),
        "the message says what was wanted: {wrong_kind}"
    );
}
