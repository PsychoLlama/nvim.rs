//! The `$NVIM_LOG_FILE` log: the shape of a line, and how the file is
//! chosen.
//!
//! `log.rs` had no test. `test/functional/core/log_spec.lua` starts a child
//! Nvim and greps its log for a name and a source location, which covers the
//! two fields the spec was written for and nothing else: the level tag, the
//! timestamp, the padding, the tagged form with no source location, the
//! unterminated form the RPC trace appends to, the level cutoff, and every
//! step of the fallback from `$NVIM_LOG_FILE` down to `./nvim.log` were all
//! unobserved.
//!
//! These cases drive `log_init` in-process instead. That is process-wide
//! state — one path, one "did we initialise", one recursion flag — so every
//! case takes the editor lock through its [`Sandbox`], and [`Logging`] puts
//! the log somewhere that still exists on the way out, because the case's own
//! directory is deleted before the next `logmsg!` anywhere in this binary
//! runs.

#![cfg(not(miri))]

use std::ffi::c_int;
use std::path::{Path, PathBuf};

use neovim::log::{LOGLVL_DBG, LOGLVL_ERR, LOGLVL_INF, LOGLVL_WRN, log_init, logmsg_line};
use neovim::main::g_min_log_level;

use crate::support::Sandbox;

/// Points the editor's log at a path for the rest of the case.
///
/// The drop matters more than the constructor: `log_init` has no undo, so
/// once a case has run, every later log line in this test binary goes
/// wherever this left it. A deleted sandbox directory would make each of
/// them fail to open and complain on stderr, so the way out is a file in the
/// system temp directory that outlives every case.
struct Logging;

impl Logging {
    /// `$NVIM_LOG_FILE` = `path`, then re-decide the log file.
    fn to(sandbox: &mut Sandbox, path: &Path) -> Logging {
        sandbox.set_env("NVIM_LOG_FILE", path.to_str().expect("a temp path is text"));
        sandbox.remember_env("__NVIM_LOG_FILE_WANT");
        log_init();
        Logging
    }
}

impl Drop for Logging {
    fn drop(&mut self) {
        let parking = std::env::temp_dir().join("nvim-unit-log-parking.log");
        // SAFETY-free: this is the test's own process environment, and the
        // editor lock is still held by the case's sandbox.
        unsafe { std::env::set_var("NVIM_LOG_FILE", &parking) };
        log_init();
    }
}

/// Write one line and answer what landed in `at`.
fn logged(at: &Path, line: impl FnOnce()) -> String {
    let before = std::fs::read_to_string(at).unwrap_or_default();
    line();
    let after = std::fs::read_to_string(at).expect("the log file");
    assert!(
        after.starts_with(&before),
        "the log was rewritten, not appended"
    );
    after[before.len()..].to_string()
}

/// One log line taken apart: the level tag, the timestamp, the instance
/// name, and everything from the source location on.
struct Line<'a> {
    level: &'a str,
    stamp: &'a str,
    name: &'a str,
    /// The width the name was padded to, `%-10s` in the format.
    name_width: usize,
    rest: &'a str,
}

impl<'a> Line<'a> {
    fn parse(line: &'a str) -> Line<'a> {
        let (level, tail) = line.split_once(' ').expect("a level and a space");
        let (stamp, tail) = tail.split_once(' ').expect("a timestamp and a space");
        let (name, padded) = tail.split_once(' ').expect("a name and a space");
        let rest = padded.trim_start_matches(' ');
        Line {
            level,
            stamp,
            name,
            name_width: name.len() + 1 + (padded.len() - rest.len()),
            rest,
        }
    }

    /// Whether the timestamp is `%Y-%m-%dT%H:%M:%S` plus three digits of
    /// milliseconds — checked by shape, since the value is the clock's.
    fn stamp_is_well_formed(&self) -> bool {
        let digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
        let punctuation: Vec<(usize, char)> = self
            .stamp
            .char_indices()
            .filter(|(_, c)| !c.is_ascii_digit())
            .collect();
        self.stamp.len() == 23
            && punctuation
                == [
                    (4, '-'),
                    (7, '-'),
                    (10, 'T'),
                    (13, ':'),
                    (16, ':'),
                    (19, '.'),
                ]
            && digits(&self.stamp[20..])
    }
}

#[test]
fn a_line_carries_its_level_time_name_and_source_location() {
    let mut sandbox = Sandbox::dir("log-line-shape");
    let at = sandbox.path("nvim.log");
    let _logging = Logging::to(&mut sandbox, &at);

    let text = logged(&at, || {
        assert!(logmsg_line(
            LOGLVL_ERR,
            None,
            Some(c"server_init"),
            58,
            true,
            || "test log message".to_string(),
        ));
    });
    assert!(
        text.ends_with('\n'),
        "an `eol` line is terminated: {text:?}"
    );
    let line = Line::parse(text.trim_end_matches('\n'));
    assert_eq!(line.level, "ERR");
    assert!(line.stamp_is_well_formed(), "timestamp {:?}", line.stamp);
    assert!(!line.name.is_empty(), "the instance name is never blank");
    assert!(line.name_width >= 11, "`%-10s` plus its separator");
    assert_eq!(line.rest, "server_init:58: test log message");
}

#[test]
fn each_level_prints_its_own_three_letter_tag() {
    let mut sandbox = Sandbox::dir("log-levels");
    let at = sandbox.path("nvim.log");
    let _logging = Logging::to(&mut sandbox, &at);

    let levels = [
        (LOGLVL_DBG, "DBG"),
        (LOGLVL_INF, "INF"),
        (LOGLVL_WRN, "WRN"),
        (LOGLVL_ERR, "ERR"),
    ];
    for (level, tag) in levels {
        let text = logged(&at, || {
            assert!(logmsg_line(level, None, Some(c"who"), 1, true, || "x".into()));
        });
        assert_eq!(Line::parse(&text).level, tag);
    }
}

#[test]
fn a_tagged_line_carries_no_source_location() {
    let mut sandbox = Sandbox::dir("log-tagged");
    let at = sandbox.path("nvim.log");
    let _logging = Logging::to(&mut sandbox, &at);

    // What `logmsg_tagged!` passes: a context instead of a function, and no
    // line number. The tag is written flush against the payload, which is
    // why every caller's tag ends in its own separator.
    let text = logged(&at, || {
        assert!(logmsg_line(
            LOGLVL_INF,
            Some(c"RPC: "),
            None,
            -1,
            true,
            || "nvim_get_mode".to_string(),
        ));
    });
    assert_eq!(
        Line::parse(text.trim_end_matches('\n')).rest,
        "RPC: nvim_get_mode"
    );
}

#[test]
fn a_line_with_neither_context_nor_location_is_marked_unknown() {
    let mut sandbox = Sandbox::dir("log-unknown");
    let at = sandbox.path("nvim.log");
    let _logging = Logging::to(&mut sandbox, &at);

    let text = logged(&at, || {
        assert!(logmsg_line(LOGLVL_WRN, None, None, -1, true, || "orphan".into()));
    });
    assert_eq!(Line::parse(text.trim_end_matches('\n')).rest, "?:orphan");
}

#[test]
fn an_eol_free_line_supplies_its_own_terminator() {
    let mut sandbox = Sandbox::dir("log-eol");
    let at = sandbox.path("nvim.log");
    let _logging = Logging::to(&mut sandbox, &at);

    // What the RPC trace does: `eol` off, and a payload that ends in its own
    // newline, so that the line is terminated exactly once.
    let text = logged(&at, || {
        assert!(logmsg_line(
            LOGLVL_DBG,
            Some(c"RPC: "),
            None,
            -1,
            false,
            || { "<- 1: [request] id=2: nvim_get_mode\n".to_string() }
        ));
    });
    assert!(text.ends_with(": nvim_get_mode\n"), "{text:?}");
    assert_eq!(text.matches('\n').count(), 1, "no second terminator");

    // With no terminator anywhere the line simply stays open, and the next
    // line's prefix runs straight into it — each call writes a whole prefix
    // of its own, so this is a run-on rather than a continuation.
    let text = logged(&at, || {
        assert!(logmsg_line(
            LOGLVL_DBG,
            Some(c"RPC: "),
            None,
            -1,
            false,
            || "head ".into()
        ));
        assert!(logmsg_line(
            LOGLVL_DBG,
            Some(c"RPC: "),
            None,
            -1,
            true,
            || "tail".into()
        ));
    });
    let lines: Vec<&str> = text.trim_end_matches('\n').split('\n').collect();
    assert_eq!(lines.len(), 1, "two calls, one physical line: {text:?}");
    assert!(lines[0].starts_with("DBG "), "{:?}", lines[0]);
    assert!(lines[0].contains("RPC: head DBG "), "{:?}", lines[0]);
    assert!(lines[0].ends_with("RPC: tail"), "{:?}", lines[0]);
}

#[test]
fn a_payload_keeps_the_newlines_it_carries() {
    let mut sandbox = Sandbox::dir("log-multiline");
    let at = sandbox.path("nvim.log");
    let _logging = Logging::to(&mut sandbox, &at);

    // Nothing escapes or splits the payload: a message with newlines in it
    // lands as several lines of which only the first carries a prefix.
    let text = logged(&at, || {
        assert!(logmsg_line(LOGLVL_ERR, None, Some(c"f"), 7, true, || {
            "first\nsecond\nthird".to_string()
        }));
    });
    let lines: Vec<&str> = text.trim_end_matches('\n').split('\n').collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(Line::parse(lines[0]).rest, "f:7: first");
    assert_eq!(&lines[1..], ["second", "third"]);
}

#[test]
fn a_line_below_the_minimum_level_is_not_written() {
    let mut sandbox = Sandbox::dir("log-min-level");
    let at = sandbox.path("nvim.log");
    let _logging = Logging::to(&mut sandbox, &at);

    let saved = g_min_log_level.get();
    g_min_log_level.set(LOGLVL_WRN);
    let text = logged(&at, || {
        assert!(!logmsg_line(LOGLVL_INF, None, Some(c"f"), 1, true, || {
            panic!("the payload of a filtered line is never rendered")
        }));
        assert!(logmsg_line(LOGLVL_WRN, None, Some(c"f"), 1, true, || {
            "kept".into()
        }));
    });
    g_min_log_level.set(saved);
    assert_eq!(Line::parse(text.trim_end_matches('\n')).rest, "f:1: kept");
}

/// Where the log ended up, and what `$NVIM_LOG_FILE` and
/// `$__NVIM_LOG_FILE_WANT` say about it, after one `log_init`.
fn resolve(_sandbox: &mut Sandbox) -> (Option<PathBuf>, Option<PathBuf>) {
    log_init();
    logmsg_line(LOGLVL_ERR, None, Some(c"resolve"), 1, true, || {
        "probe".into()
    });
    let var = |name: &str| std::env::var_os(name).map(PathBuf::from);
    (var("NVIM_LOG_FILE"), var("__NVIM_LOG_FILE_WANT"))
}

#[test]
fn the_log_file_falls_back_one_step_at_a_time() {
    let mut sandbox = Sandbox::dir("log-fallback");
    let _logging = Logging {};
    let root = sandbox.root().to_path_buf();
    sandbox.remember_env("NVIM_LOG_FILE");
    sandbox.remember_env("__NVIM_LOG_FILE_WANT");
    sandbox.remember_env("XDG_STATE_HOME");
    let state = root.join("state");
    sandbox.set_env("XDG_STATE_HOME", state.to_str().unwrap());

    // 1. A writable `$NVIM_LOG_FILE` is used as it stands, and left alone:
    //    it already holds the value the editor decided on.
    let wanted = root.join("wanted.log");
    sandbox.set_env("NVIM_LOG_FILE", wanted.to_str().unwrap());
    sandbox.unset_env("__NVIM_LOG_FILE_WANT");
    let (chosen, want) = resolve(&mut sandbox);
    assert_eq!(chosen.as_deref(), Some(wanted.as_path()));
    assert_eq!(want, None, "nothing was abandoned");
    assert!(
        wanted.exists(),
        "the line landed in the file that was asked for"
    );

    // 2. A `$NVIM_LOG_FILE` naming a *directory* cannot be appended to. The
    //    wanted path is remembered for `_core/log.lua` to complain about and
    //    the log drops to `$XDG_STATE_HOME/nvim/nvim.log`.
    let as_dir = sandbox.mkdir("not-a-file");
    sandbox.set_env("NVIM_LOG_FILE", as_dir.to_str().unwrap());
    sandbox.unset_env("__NVIM_LOG_FILE_WANT");
    let (chosen, want) = resolve(&mut sandbox);
    assert_eq!(want.as_deref(), Some(as_dir.as_path()));
    assert_eq!(
        chosen.as_deref(),
        Some(state.join("nvim/nvim.log").as_path())
    );
    assert!(state.join("nvim/nvim.log").exists());

    // 3. With no `$NVIM_LOG_FILE` and a state directory that cannot be made,
    //    the last resort is `./nvim.log` in the working directory — and the
    //    state path is what gets remembered as wanted.
    sandbox.unset_env("NVIM_LOG_FILE");
    sandbox.unset_env("__NVIM_LOG_FILE_WANT");
    let blocked = sandbox.touch("blocked");
    sandbox.set_env("XDG_STATE_HOME", blocked.to_str().unwrap());
    let (chosen, want) = resolve(&mut sandbox);
    // Relative, as upstream leaves it: the last resort is `nvim.log` in
    // whatever directory the editor is standing in.
    assert_eq!(chosen.as_deref(), Some(Path::new("nvim.log")));
    assert_eq!(
        want.as_deref(),
        Some(blocked.join("nvim/nvim.log").as_path())
    );
    assert!(root.join("nvim.log").exists());
}

/// The level constants are the numbers `'verbose'` and `$NVIM_LOG_LEVEL`
/// compare against, so their values are interface, not an enumeration.
#[test]
fn the_levels_are_the_numbers_they_have_always_been() {
    let levels: [c_int; 4] = [LOGLVL_DBG, LOGLVL_INF, LOGLVL_WRN, LOGLVL_ERR];
    assert_eq!(levels, [1, 2, 3, 4]);
}
