//! What the process says when it is asked, or when the command line is
//! wrong: `--help`, `--version`, and the argument errors.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};

use crate::lua::executor::nlua_init;
use crate::message::msg_putchar;
use crate::message::state::{info_message, msg_didout};
use crate::os::cshim::{gettext, gettext_owned, write_c_stderr, write_c_stdout};
use crate::os::signal::signal_stop;
use crate::path::path_tail;
use crate::startup::argv0;
use crate::startup::exit::os_exit;
use crate::types::IOSIZE;
use crate::version::list_version;

/// The `--help` text, one line per entry.
///
/// An empty line is printed literally; every other line goes through
/// `gettext`, which is why this is a list of strings and not one blob.
const USAGE: &[&CStr] = &[
    c"Usage:\n",
    c"  nvim [options] [file ...]\n",
    c"\nOptions:\n",
    c"  --cmd <cmd>           Execute <cmd> before any config\n",
    c"  +<cmd>, -c <cmd>      Execute <cmd> after config and first file\n",
    c"  -l <script> [args...] Execute Lua <script> (with optional args)\n",
    c"  -S <session>          Source <session> after loading the first file\n",
    c"  -s <scriptin>         Read Normal mode commands from <scriptin>\n",
    c"  -u <config>           Use this config file\n",
    c"",
    c"  -d                    Diff mode\n",
    c"  -es, -Es              Silent (batch) mode\n",
    c"  -h, --help            Print this help message\n",
    c"  -i <shada>            Use this shada file\n",
    c"  -n                    No swap file, use memory only\n",
    c"  -o[N]                 Open N windows (default: one per file)\n",
    c"  -O[N]                 Open N vertical windows (default: one per file)\n",
    c"  -p[N]                 Open N tab pages (default: one per file)\n",
    c"  -R                    Read-only (view) mode\n",
    c"  -v, --version         Print version information\n",
    c"  -V[N][file]           Verbose [level][file]\n",
    c"",
    c"  --                    Only file names after this\n",
    c"  --api-info            Write msgpack-encoded API metadata to stdout\n",
    c"  --clean               \"Factory defaults\" (skip user config and plugins, shada)\n",
    c"  --embed               Use stdin/stdout as a msgpack-rpc channel\n",
    c"  --headless            Don't start a user interface\n",
    c"  --listen <address>    Serve RPC API from this address\n",
    c"  --remote[-subcommand] Execute commands remotely on a server\n",
    c"  --server <address>    Connect to this Nvim server\n",
    c"  --startuptime <file>  Write startup timing messages to <file>\n",
    c"\nSee \":help startup-options\" for all options.\n",
];

/// Print the usage summary on stdout.
pub(crate) fn usage() {
    signal_stop();
    for line in USAGE {
        if line.is_empty() {
            write_c_stdout(b"\n");
        } else {
            write_c_stdout(gettext(line).to_bytes());
        }
    }
}

/// Print `--version`.
///
/// Lua has to be up first: the version list names the Lua runtime and the
/// features that depend on it.
pub(crate) fn version() {
    nlua_init(&[], -1);
    info_message.set(true);
    list_version();
    msg_putchar('\n' as c_int);
    msg_didout.set(false);
}

/// Report a command-line error on stderr, in the shape every other tool
/// uses: `nvim: <what>: "<offending argument>"`.
///
/// `msg1` is a msgid; `msg2` and `msg3` are optional and quoted when present.
pub(crate) fn print_mainerr(msg1: &CStr, msg2: Option<&CStr>, msg3: Option<&CStr>) {
    let prgname = argv0.get().map_or(c"", path_tail).to_bytes();
    let text = gettext_owned(msg1);
    // Nothing beyond this point should be interrupted by a handler that
    // expects a running editor.
    signal_stop();
    let mut out = [prgname, b": ", text.to_bytes()].concat();
    for arg in [msg2, msg3].into_iter().flatten() {
        out.extend_from_slice(b": \"");
        out.extend_from_slice(arg.to_bytes());
        out.push(b'"');
    }
    out.extend_from_slice(gettext(c"\nMore info with \"").to_bytes());
    out.extend_from_slice(prgname);
    out.extend_from_slice(b" -h\"\n");
    write_c_stderr(&out);
}

/// [`print_mainerr`] and then exit 1. Every argument error takes this path.
pub(crate) fn mainerr(msg1: &CStr, msg2: Option<&CStr>, msg3: Option<&CStr>) -> ! {
    print_mainerr(msg1, msg2, msg3);
    os_exit(1);
}

/// What upstream's `IObuff` holds, its NUL aside.
const IOBUFF_ROOM: usize = IOSIZE.unsigned_abs() as usize - 1;

/// `template` (a msgid) translated, with each `%s` in it replaced by the next
/// of `args`, cut to what upstream's `IObuff` holds -- the one `vim_snprintf`
/// shape the startup errors use, kept to the byte for argument words that are
/// not UTF-8.
pub(crate) fn fill_template(template: &'static CStr, args: &[&[u8]]) -> Vec<u8> {
    let template = gettext(template).to_bytes();
    let mut out = Vec::with_capacity(template.len());
    let mut args = args.iter();
    let mut rest = template;
    while let Some(at) = rest.windows(2).position(|pair| pair == b"%s") {
        out.extend_from_slice(&rest[..at]);
        out.extend_from_slice(args.next().copied().unwrap_or_default());
        rest = &rest[at + 2..];
    }
    out.extend_from_slice(rest);
    out.truncate(IOBUFF_ROOM);
    out
}
