//! The fixture the lib tests that drive Ex commands share: a current window
//! and buffer, the evaluator brought up once per process, and helpers that
//! run a command line, a block of lines, or an expression.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::buffer::{bare_buffer, free_bare_buffer};
use crate::ex_docmd::do_cmdline_cmd;
use crate::getchar::state::got_int;
use crate::global_cell::{editor_state::Held, editor_state_lock};
use crate::memory::XString;
use crate::option::vars::{P_CPO, P_DEBUG, P_MFD};
use crate::window::{bare_window, free_bare_window};
use crate::winlayer::graph::{leave_curbuf, leave_curwin};
use crate::winlayer::{Buf, Win};
use std::ffi::CString;
use std::path::PathBuf;

/// A current window and buffer, the evaluator brought up once per process,
/// `'maxfuncdepth'` at its default, and no screen to draw messages on.
pub(crate) struct Fixture {
    window: Win,
    buffer: Buf,
    saved: Vec<(&'static crate::option::vars::StrOpt, Option<XString>)>,
    /// What [`check`](Self::check) and its siblings found, reported on drop.
    mismatches: std::cell::RefCell<Vec<String>>,
    _quiet: (crate::guard::MsgBump, crate::guard::MsgBump),
    _held: Held,
}

impl Fixture {
    pub(crate) fn new() -> Fixture {
        let held = editor_state_lock();
        got_int.set(false);
        let mut window = bare_window();
        window.make_current();
        let buffer = bare_buffer();
        buffer.make_current();
        // The current window shows the current buffer, as it always does in
        // the editor: a walk that reaches the buffer through the window
        // (`<buffer>`-local tables, `:command -buffer`) finds this one.
        window.w_buffer = buffer;
        P_MFD.set(100);
        // A message starts on a command line of its own rather than
        // scrolling a grid nobody sized.
        crate::option::vars::P_CH.set(1);
        // No highlight groups exist: a message must not try to make them.
        crate::highlight::state::need_highlight_changed.set(false);
        // Nothing ran the option defaults; the ones a call reads.
        let saved = [(&P_CPO, "aABceFs_"), (&P_DEBUG, "")]
            .into_iter()
            .map(|(option, value)| (option, option.swap(Some(XString::from(value)))))
            .collect();
        // Character classes and cell widths: without them every byte is zero
        // cells wide, and a listing padded by width depends on whether some
        // other test happened to fill the table first.
        crate::charset::init_chartab_for_tests();
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| {
            // A pattern compiled with 'cpoptions' cleared (`:catch`'s, a
            // `split()`'s) reads it as empty, which is what the defaults
            // being in means.
            crate::option::vars::mark_option_defaults_set();
            crate::eval::eval_init();
            crate::runtime::estack_init();
        });
        let fixture = Fixture {
            window,
            buffer,
            saved,
            mismatches: std::cell::RefCell::default(),
            // Errors still set `v:errmsg`; neither they nor messages are
            // drawn, since there is no screen.
            _quiet: (
                crate::guard::Suppress::emsg_silent(),
                crate::guard::Suppress::messages(),
            ),
            _held: held,
        };
        // `test_garbagecollect_now()` wants it, and it keeps the arguments
        // of the calls in progress markable.
        fixture.run("let v:testing = 1");
        fixture
    }

    /// Run one command line.
    pub(crate) fn run(&self, cmd: &str) {
        let cmd = CString::new(cmd).expect("no NUL");
        let _ = do_cmdline_cmd(&cmd);
    }

    /// Run `lines` as one block, through `execute()`, which hands a
    /// `:function` its body from the lines after it.
    pub(crate) fn block(&self, lines: &[&str]) {
        let quoted: Vec<String> = lines
            .iter()
            .map(|line| format!("'{}'", line.replace('\'', "''")))
            .collect();
        self.run(&format!("call execute([{}])", quoted.join(", ")));
    }

    /// `expr`'s value as `string()` renders it, or `<fail>`.
    pub(crate) fn eval(&self, expr: &str) -> String {
        let text = format!("string({expr})");
        crate::eval::eval_to_string(text.as_bytes(), false, false)
            .map(|s| String::from_utf8_lossy(s.as_cstr().to_bytes()).into_owned())
            .unwrap_or_else(|| "<fail>".into())
    }

    /// The error number a call of `expr` reports, or `none`.
    pub(crate) fn error_of(&self, expr: &str) -> String {
        self.run("let v:errmsg = ''");
        self.run(&format!("silent! call {expr}"));
        // Sliced, not split: a pattern would read 'cpoptions', which a
        // test has no defaults for.
        let err = self.eval("v:errmsg[: stridx(v:errmsg, ':') - 1]");
        if err == "''" {
            "none".into()
        } else {
            err.trim_matches('\'').into()
        }
    }

    /// What `cmd` prints, as `execute()` captures it.
    pub(crate) fn output(&self, cmd: &str) -> String {
        let quoted = cmd.replace('\'', "''");
        let text = format!("execute('{quoted}')");
        crate::eval::eval_to_string(text.as_bytes(), false, false)
            .map(|s| String::from_utf8_lossy(s.as_cstr().to_bytes()).into_owned())
            .unwrap_or_else(|| "<fail>".into())
    }

    /// The error `cmd` raises, as `v:errmsg` keeps it, or `none`.
    pub(crate) fn error_from(&self, cmd: &str) -> String {
        self.run("let v:errmsg = ''");
        self.run(&format!("silent! {cmd}"));
        let err = self.eval("v:errmsg");
        if err == "''" {
            "none".into()
        } else {
            err.trim_matches('\'').into()
        }
    }

    /// Book a mismatch: the fixture fails once, when it goes, with every
    /// row that moved written as the row that would make it pass -- a table
    /// of answers reads better than forty assertions, and one run shows all
    /// of them.
    #[track_caller]
    fn book(&self, kind: &str, what: &str, got: &str, want: &str) {
        if got != want {
            let at = std::panic::Location::caller();
            self.mismatches.borrow_mut().push(format!(
                "{}:{}: {kind}({what:?}, {got:?})",
                at.file(),
                at.line()
            ));
        }
    }

    /// `expr`'s value (as `string()` renders it) should be `want`.
    #[track_caller]
    pub(crate) fn check(&self, expr: &str, want: &str) {
        self.book("check", expr, &self.eval(expr), want);
    }

    /// What `cmd` prints should be `want`.
    #[track_caller]
    pub(crate) fn check_output(&self, cmd: &str, want: &str) {
        self.book("check_output", cmd, &self.output(cmd), want);
    }

    /// The error `cmd` raises should be `want` (`none` for none).
    #[track_caller]
    pub(crate) fn check_error(&self, cmd: &str, want: &str) {
        self.book("check_error", cmd, &self.error_from(cmd), want);
    }

    /// [`check`](Self::check) over a table of `(expr, want)` rows.
    #[track_caller]
    pub(crate) fn expect_all(&self, cases: &[(&str, &str)]) {
        for (expr, want) in cases {
            self.check(expr, want);
        }
    }

    pub(crate) fn delete(&self, names: &[&str]) {
        for name in names {
            self.run(&format!("silent! delfunction {name}"));
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        for (option, value) in self.saved.drain(..) {
            option.restore(value);
        }
        self.window.w_buffer = Buf::NULL;
        leave_curbuf();
        free_bare_buffer(self.buffer);
        leave_curwin();
        free_bare_window(self.window);
        let found = self.mismatches.take();
        if !found.is_empty() && !std::thread::panicking() {
            panic!("{} mismatches:\n{}", found.len(), found.join("\n"));
        }
    }
}

/// A scratch directory for the files a case sources or writes, removed when
/// it goes.
pub(crate) struct Scratch(pub(crate) PathBuf);

impl Scratch {
    pub(crate) fn new(name: &str) -> Scratch {
        let dir = std::env::temp_dir().join(format!("nvim-test-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        Scratch(dir)
    }

    pub(crate) fn file(&self, name: &str, text: &str) -> String {
        let path = self.0.join(name);
        std::fs::write(&path, text).expect("a scratch file");
        path.to_str().expect("a UTF-8 path").to_owned()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
