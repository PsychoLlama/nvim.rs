//! Startup: what the argument scan makes of an argv, run as real processes.
//!
//! `init_params` and `command_line_scan` read `argc`/`argv` into
//! `MainParams` once, before anything else exists, and the rest of
//! `main` — the window layout, the order `--cmd`, the user config, `-c` and
//! `+cmd` run in, the read-only and binary flags, stdin, the error exits —
//! follows from what they recorded. None of it can be driven inside the
//! shared editor this suite runs in, so each case starts the `nvim` this
//! package builds with an argv table and reads back what it reports.
//!
//! Every child is `--headless -u NONE -i NONE` (unless the case is about
//! one of those), runs in the case's own directory with `$HOME` and the XDG
//! directories pointed into it, and reports through a file it writes as its
//! last `-c`, so nothing depends on how the terminal-less child prints.
//!
//! Written against the behaviour before `MainParams` owns its arguments and
//! `command_line_scan` reads a slice; a row that moves is a behaviour
//! change.

#![cfg(not(miri))]

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::support::Sandbox;

/// What every case asks the child for, one `key=value` per field in a fixed
/// order. `line` reads 0 when no `+` command moved the cursor: `-c`
/// commands run with the cursor on line 0, so that `+/pat` can match line 1.
const REPORT: &str = "call writefile([join(map([\
['argv', argv()], ['buf', bufname()], ['line', line('.')], \
['wins', winnr('$')], ['tabs', tabpagenr('$')], ['ro', &readonly], \
['ma', &modifiable], ['bin', &binary], ['diff', &diff], ['uc', &updatecount], \
['order', get(g:, 'order', [])], ['lines', getline(1, '$')]], \
{_, kv -> kv[0] .. '=' .. string(kv[1])}))], 'report')";

/// The child's exit code, its stderr, and its report (`<none>` when it
/// never wrote one).
struct Run {
    code: Option<i32>,
    stderr: String,
    report: String,
}

/// Start `nvim` in `dir` with `args`, `stdin` on its standard input, and
/// wait at most ten seconds for it.
fn nvim<A: AsRef<std::ffi::OsStr> + std::fmt::Debug>(
    dir: &Path,
    args: &[A],
    stdin: Option<&[u8]>,
) -> Run {
    let _ = std::fs::remove_file(dir.join("report"));
    let runtime = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../runtime");
    let mut child = Command::new(env!("CARGO_BIN_EXE_nvim"))
        .args(args)
        .current_dir(dir)
        .env_clear()
        .env("HOME", dir)
        .env("PATH", "/usr/bin:/bin")
        .env("VIMRUNTIME", runtime)
        .env("XDG_CONFIG_HOME", dir.join("config"))
        .env("XDG_DATA_HOME", dir.join("data"))
        .env("XDG_STATE_HOME", dir.join("state"))
        .env("XDG_CACHE_HOME", dir.join("cache"))
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("nvim starts");
    if let Some(bytes) = stdin {
        use std::io::Write;
        let mut pipe = child.stdin.take().expect("a stdin pipe");
        pipe.write_all(bytes).expect("stdin written");
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.try_wait().expect("the child") {
            break status;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!("nvim {args:?} did not exit");
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let mut stderr = String::new();
    {
        use std::io::Read;
        let _ = child
            .stderr
            .take()
            .expect("a stderr pipe")
            .read_to_string(&mut stderr);
    }
    Run {
        code: status.code(),
        stderr,
        report: std::fs::read_to_string(dir.join("report"))
            .map_or_else(|_| "<none>".into(), |text| text.trim_end().to_owned()),
    }
}

/// `nvim --headless -u NONE -i NONE <args> -c REPORT -c 'qa!'`.
fn report(dir: &Path, args: &[&str]) -> String {
    let mut argv = vec!["--headless", "-u", "NONE", "-i", "NONE"];
    argv.extend_from_slice(args);
    argv.extend_from_slice(&["-c", REPORT, "-c", "qa!"]);
    let run = nvim(dir, &argv, None);
    assert_eq!(run.code, Some(0), "{args:?}: {}", run.stderr);
    run.report
}

/// Mismatches booked rather than asserted, so one run shows every row that
/// moved; the drop fails once, listing them.
#[derive(Default)]
struct Book(std::cell::RefCell<Vec<String>>);

impl Book {
    #[track_caller]
    fn eq(&self, got: impl std::fmt::Debug, want: impl std::fmt::Debug) {
        let (got, want) = (format!("{got:?}"), format!("{want:?}"));
        if got != want {
            let at = std::panic::Location::caller();
            self.0
                .borrow_mut()
                .push(format!("{}: got {got}", at.line()));
        }
    }
}

impl Drop for Book {
    fn drop(&mut self) {
        let found = self.0.take();
        if !found.is_empty() && !std::thread::panicking() {
            panic!("{} mismatches:\n{}", found.len(), found.join("\n"));
        }
    }
}

/// One scalar field of a report.
fn field(report: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    report
        .split(' ')
        .find_map(|part| part.strip_prefix(&prefix))
        .map(str::to_owned)
}

fn files(name: &str) -> Sandbox {
    let sandbox = Sandbox::dir(name);
    sandbox.write("a.txt", b"a1\na2\na3\nneedle\na5\n");
    sandbox.write("b.txt", b"b1\n");
    sandbox.write("c.txt", b"c1\n");
    sandbox
}

#[test]
fn file_arguments_and_the_end_of_options() {
    let book = Book::default();
    let sandbox = files("startup_files");
    let dir = sandbox.root();
    book.eq(report(dir, &["a.txt", "b.txt"]), "argv=['a.txt', 'b.txt'] buf='a.txt' line=0 wins=1 tabs=1 ro=0 ma=1 bin=0 diff=0 uc=200 order=[] lines=['a1', 'a2', 'a3', 'needle', 'a5']");
    // After `--` a leading `-` is a file name.
    // (So the report's own `-c`s go first here.)
    let run = nvim(
        dir,
        &[
            "--headless",
            "-u",
            "NONE",
            "-i",
            "NONE",
            "-c",
            REPORT,
            "-c",
            "qa!",
            "--",
            "-b.txt",
            "a.txt",
        ],
        None,
    );
    book.eq((run.code, run.report), (Some(0), "argv=['-b.txt', 'a.txt'] buf='-b.txt' line=0 wins=1 tabs=1 ro=0 ma=1 bin=0 diff=0 uc=200 order=[] lines=['']".to_owned()));
    book.eq(
        report(dir, &[]),
        "argv=[] buf='' line=0 wins=1 tabs=1 ro=0 ma=1 bin=0 diff=0 uc=200 order=[] lines=['']",
    );
}

#[test]
fn plus_commands_place_the_cursor() {
    let book = Book::default();
    let sandbox = files("startup_plus");
    let dir = sandbox.root();
    let line = |args: &[&str]| {
        let got = report(dir, args);
        field(&got, "line")
    };
    book.eq(line(&["+3", "a.txt"]).as_deref(), Some("3"));
    book.eq(line(&["+", "a.txt"]).as_deref(), Some("5"));
    book.eq(line(&["+/needle", "a.txt"]).as_deref(), Some("4"));
}

#[test]
fn commands_run_in_their_order() {
    let book = Book::default();
    let sandbox = files("startup_order");
    let dir = sandbox.root();
    let got = report(
        dir,
        &[
            "--cmd",
            "let g:order = ['cmd:' .. bufname()]",
            "-c",
            "call add(g:order, 'c1:' .. bufname())",
            "+call add(g:order, 'plus')",
            "--cmd",
            "call add(g:order, 'cmd2')",
            "-c",
            "call add(g:order, 'c2')",
            "a.txt",
        ],
    );
    book.eq(got, "argv=['a.txt'] buf='a.txt' line=1 wins=1 tabs=1 ro=0 ma=1 bin=0 diff=0 uc=200 order=['cmd:a.txt', 'cmd2', 'c1:a.txt', 'plus', 'c2'] lines=['a1', 'a2', 'a3', 'needle', 'a5']");
}

#[test]
fn window_and_tab_layouts() {
    let book = Book::default();
    let sandbox = files("startup_layout");
    let dir = sandbox.root();
    let shape = |args: &[&str]| {
        let got = report(dir, args);
        format!(
            "{}/{}",
            field(&got, "wins").unwrap_or_default(),
            field(&got, "tabs").unwrap_or_default()
        )
    };
    book.eq(shape(&["-o", "a.txt", "b.txt", "c.txt"]), "3/1");
    book.eq(shape(&["-O2", "a.txt", "b.txt", "c.txt"]), "2/1");
    book.eq(shape(&["-p", "a.txt", "b.txt"]), "1/2");
    book.eq(shape(&["-p5", "a.txt"]), "1/5");
}

#[test]
fn flags_that_set_options() {
    let book = Book::default();
    let sandbox = files("startup_flags");
    let dir = sandbox.root();
    book.eq(report(dir, &["-R", "a.txt"]), "argv=['a.txt'] buf='a.txt' line=0 wins=1 tabs=1 ro=1 ma=1 bin=0 diff=0 uc=10000 order=[] lines=['a1', 'a2', 'a3', 'needle', 'a5']");
    book.eq(report(dir, &["-M", "b.txt"]), "argv=['b.txt'] buf='b.txt' line=0 wins=1 tabs=1 ro=0 ma=0 bin=0 diff=0 uc=200 order=[] lines=['b1']");
    book.eq(report(dir, &["-b", "-n", "b.txt"]), "argv=['b.txt'] buf='b.txt' line=0 wins=1 tabs=1 ro=0 ma=1 bin=1 diff=0 uc=0 order=[] lines=['b1']");
    book.eq(report(dir, &["-d", "b.txt", "c.txt"]), "argv=['b.txt', 'c.txt'] buf='b.txt' line=0 wins=2 tabs=1 ro=0 ma=1 bin=0 diff=1 uc=200 order=[] lines=['b1']");
}

#[test]
fn bad_arguments_exit_with_a_message() {
    let book = Book::default();
    let sandbox = files("startup_errors");
    let dir = sandbox.root();
    let fail = |args: &[&str]| {
        let run = nvim(dir, args, None);
        format!("{:?} {}", run.code, run.stderr.trim_end())
    };
    book.eq(
        fail(&["--headless", "-x"]),
        "Some(1) nvim: Unknown option argument: \"-x\"\nMore info with \"nvim -h\"",
    );
    book.eq(
        fail(&["--headless", "--bogus"]),
        "Some(1) nvim: Unknown option argument: \"--bogus\"\nMore info with \"nvim -h\"",
    );
    book.eq(
        fail(&["--headless", "-u"]),
        "Some(1) nvim: Argument missing after: \"-u\"\nMore info with \"nvim -h\"",
    );
    let many: Vec<&str> = std::iter::repeat_n(["-c", "echo"], 11).flatten().collect();
    book.eq(fail(&[&["--headless"][..], &many].concat()), "Some(1) nvim: Too many \"+command\", \"-c command\" or \"--cmd command\" arguments\nMore info with \"nvim -h\"");
}

#[test]
fn stdin_is_read_as_text_with_a_dash() {
    let book = Book::default();
    let sandbox = files("startup_stdin");
    let dir = sandbox.root();
    let mut argv = vec!["--headless", "-u", "NONE", "-i", "NONE", "-"];
    argv.extend_from_slice(&["-c", REPORT, "-c", "qa!"]);
    let run = nvim(dir, &argv, Some(b"from\nstdin\n"));
    book.eq((run.code, run.report), (Some(0), "argv=[] buf='' line=0 wins=1 tabs=1 ro=0 ma=1 bin=0 diff=0 uc=200 order=[] lines=['from', 'stdin']".to_owned()));
}

#[test]
fn silent_ex_mode_reads_commands_from_stdin() {
    let book = Book::default();
    let sandbox = files("startup_ex");
    let dir = sandbox.root();
    let run = nvim(
        dir,
        &["-es", "-u", "NONE", "-i", "NONE", "a.txt"],
        Some(b"2,3delete\nwrite! out.txt\nqa!\n"),
    );
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    book.eq(
        std::fs::read_to_string(dir.join("out.txt")).ok().as_deref(),
        Some("a1\nneedle\na5\n"),
    );
}

#[test]
fn a_lua_script_gets_its_arguments_and_sets_the_status() {
    let book = Book::default();
    let sandbox = files("startup_lua");
    let dir = sandbox.root();
    sandbox.write(
        "s.lua",
        b"vim.fn.writefile({vim.inspect(_G.arg, {newline = ' ', indent = ''})}, 'report')\nos.exit(3)\n",
    );
    let run = nvim(dir, &["-l", "s.lua", "one", "-x", "--", "two"], None);
    book.eq(
        (run.code, run.report),
        (
            Some(3),
            "{ \"one\", \"-x\", \"--\", \"two\", [0] = \"s.lua\" }".to_owned(),
        ),
    );
}

#[test]
fn startuptime_logs_the_phases() {
    let book = Book::default();
    let sandbox = files("startup_time");
    let dir = sandbox.root();
    let _ = report(dir, &["--startuptime", "time.log"]);
    let log = std::fs::read_to_string(dir.join("time.log")).unwrap_or_default();
    // The times move; which phases are logged does not.
    // So does the set of Lua modules required on the way, which is the
    // runtime's business: those lines are left out.
    let phases: Vec<&str> = log
        .lines()
        .skip_while(|line| !line.ends_with("--- NVIM STARTING ---"))
        .filter_map(|line| line.split(": ").nth(1))
        .filter(|phase| !phase.starts_with("require("))
        .collect();
    book.eq(
        phases,
        [
            "--- NVIM STARTING ---",
            "event init",
            "early init",
            "locale set",
            "init first window",
            "inits 1",
            "window checked",
            "parsing arguments",
            "init lua interpreter",
            "expanding arguments",
            "inits 2",
            "init highlight",
            "clear screen",
            "sourcing nvim_exec2()",
            "init default mappings & autocommands",
            "sourcing vimrc file(s)",
            "inits 3",
            "reading ShaDa",
            "opening buffers",
            "BufEnter autocommands",
            "editing files in windows",
            "--- NVIM STARTED ---",
        ],
    );
}

#[test]
fn script_files_and_sessions() {
    let book = Book::default();
    let sandbox = files("startup_scripts");
    let dir = sandbox.root();
    sandbox.write("Session.vim", b"let g:order = ['default']\n");
    sandbox.write("s.vim", b"let g:order = ['named']\n");
    sandbox.write("keys.vim", b"");
    let order = |args: &[&str]| field(&report(dir, args), "order");
    // `-S` before another option, or last, is the default session.
    book.eq(order(&["-S"]).as_deref(), Some("['default']"));
    book.eq(order(&["-S", "s.vim"]).as_deref(), Some("['named']"));
    let fail = |args: &[&str]| {
        let run = nvim(dir, args, None);
        format!("{:?} {}", run.code, run.stderr.trim_end())
    };
    book.eq(
        fail(&["--headless", "-s", "keys.vim", "-s", "keys.vim", "+q"]),
        "Some(2) Attempt to open script file again: \"-s keys.vim\"",
    );
    book.eq(
        fail(&["--headless", "-w", "out.log", "-W", "out2.log", "+q"]),
        "Some(2) Attempt to open script file again: \"-W out2.log\"",
    );
    // `-w` with digits is a 'window' height, in the word or the next one.
    let window = |args: &[&str]| {
        let mut argv = vec!["--headless", "-u", "NONE", "-i", "NONE"];
        argv.extend_from_slice(args);
        argv.extend_from_slice(&["-c", "call writefile([&window], 'report')", "-c", "qa!"]);
        nvim(dir, &argv, None).report
    };
    book.eq(window(&["-w12"]), "12");
    book.eq(window(&["-w", "7"]), "7");
    // An unwritable `-w` file stops the startup.
    book.eq(
        fail(&["--headless", "-u", "NONE", "-W", "nodir/out.log", "+q"]),
        "Some(2) Cannot open for script output: \"nodir/out.log\"",
    );
}

#[test]
fn attached_values_and_conflicts() {
    let book = Book::default();
    let sandbox = files("startup_attached");
    let dir = sandbox.root();
    sandbox.write("errors.txt", b"a.txt:4:needle here\n");
    sandbox.write("tags", b"needle\ta.txt\t4\n");
    let qf = |args: &[&str]| {
        let got = report(dir, args);
        format!("{:?} {:?}", field(&got, "buf"), field(&got, "line"))
    };
    book.eq(qf(&["-qerrors.txt"]), "Some(\"'a.txt'\") Some(\"4\")");
    book.eq(qf(&["-q", "errors.txt"]), "Some(\"'a.txt'\") Some(\"4\")");
    book.eq(qf(&["-tneedle"]), "Some(\"'a.txt'\") Some(\"4\")");
    book.eq(qf(&["-t", "needle"]), "Some(\"'a.txt'\") Some(\"4\")");
    let fail = |args: &[&str]| {
        let run = nvim(dir, args, None);
        format!("{:?} {}", run.code, run.stderr.trim_end())
    };
    book.eq(
        fail(&["--embed", "-es"]),
        "Some(1) nvim: --embed conflicts with -es/-Es/-l\nMore info with \"nvim -h\"",
    );
    book.eq(
        fail(&["--headless", "-t", "needle", "-q", "errors.txt"]),
        "Some(1) nvim: Too many edit arguments: \"-q\"\nMore info with \"nvim -h\"",
    );
    // `-ll` is only the bare interpreter as the first argument.
    book.eq(
        fail(&["-ll"]),
        "Some(1) nvim: Argument missing after: \"-ll\"\nMore info with \"nvim -h\"",
    );
    book.eq(
        fail(&["--headless", "-u", "NONE", "-c", "qa!", "-ll"]),
        "Some(1) nvim: Garbage after option argument: \"-ll\"\nMore info with \"nvim -h\"",
    );
    // `-d file dir` diffs `file` against `dir/file`.
    std::fs::create_dir_all(dir.join("sub")).expect("a directory");
    sandbox.write("sub/b.txt", b"b1\nb2\n");
    let argv = [
        "--headless",
        "-u",
        "NONE",
        "-i",
        "NONE",
        "-d",
        "b.txt",
        "sub",
    ];
    let write = "call writefile([join(argv(), '|')], 'report')";
    let run = nvim(
        dir,
        &[&argv[..], &["-c", write, "-c", "qa!"]].concat(),
        None,
    );
    book.eq(run.report, "b.txt|sub/b.txt");
}

#[test]
fn a_bare_lua_interpreter_and_odd_bytes() {
    let book = Book::default();
    let sandbox = files("startup_ll");
    let dir = sandbox.root();
    sandbox.write(
        "s.lua",
        b"io.open('report', 'w'):write(table.concat(arg, ',', 0) .. ' ' .. tostring(vim == nil))\nos.exit(4)\n",
    );
    let run = nvim(dir, &["-ll", "s.lua", "x", "--y"], None);
    book.eq(
        (run.code, run.report),
        (Some(4), "s.lua,x,--y false".to_owned()),
    );
    // A file name that is not UTF-8 is still a file name.
    use std::os::unix::ffi::OsStrExt;
    let odd = std::ffi::OsStr::from_bytes(b"odd\xff.txt");
    let args: Vec<&std::ffi::OsStr> = ["--headless", "-u", "NONE", "-i", "NONE", "-c"]
        .iter()
        .map(std::ffi::OsStr::new)
        .chain([
            std::ffi::OsStr::new("call writefile([len(bufname())], 'report')"),
            std::ffi::OsStr::new("-c"),
            std::ffi::OsStr::new("qa!"),
            odd,
        ])
        .collect();
    let run = nvim(dir, &args, None);
    book.eq((run.code, run.report), (Some(0), "8".to_owned()));
}
