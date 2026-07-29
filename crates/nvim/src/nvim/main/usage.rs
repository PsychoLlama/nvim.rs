//! What the process says when it is asked, or when the command line is
//! wrong: `--help`, `--version`, and the argument errors.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use crate::src::nvim::lua::executor::nlua_init;
use crate::src::nvim::main::exit::os_exit;
use crate::src::nvim::main::{argv0, info_message, msg_didout};
use crate::src::nvim::message::msg_putchar;
use crate::src::nvim::os::libc::{fprintf, gettext, printf, stderr};
use crate::src::nvim::os::signal::signal_stop;
use crate::src::nvim::path::path_tail;
use crate::src::nvim::version::list_version;

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
pub(crate) unsafe fn usage() {
    // SAFETY: stops the signal handlers and writes to stdout.
    unsafe {
        signal_stop();
        for line in USAGE {
            if line.is_empty() {
                printf(c"\n".as_ptr());
            } else {
                printf(gettext(line.as_ptr()));
            }
        }
    }
}

/// Print `--version`.
///
/// Lua has to be up first: the version list names the Lua runtime and the
/// features that depend on it.
pub(crate) unsafe fn version() {
    // SAFETY: initialises the Lua state with no argv and writes a message.
    unsafe {
        nlua_init(ptr::null_mut(), 0, -1);
        info_message.set(true);
        list_version();
        msg_putchar('\n' as c_int);
        msg_didout.set(false);
    }
}

/// Report a command-line error on stderr, in the shape every other tool
/// uses: `nvim: <what>: "<offending argument>"`.
///
/// `msg2` and `msg3` are optional and quoted when present.
pub(crate) unsafe fn print_mainerr(msg1: *const c_char, msg2: *const c_char, msg3: *const c_char) {
    // SAFETY: the three messages are NUL-terminated or null, and `argv0` is
    // set before any caller can reach this.
    unsafe {
        let prgname = path_tail(argv0.get());
        // Nothing beyond this point should be interrupted by a handler that
        // expects a running editor.
        signal_stop();
        fprintf(stderr, c"%s: %s".as_ptr(), prgname, gettext(msg1));
        if !msg2.is_null() {
            fprintf(stderr, c": \"%s\"".as_ptr(), msg2);
        }
        if !msg3.is_null() {
            fprintf(stderr, c": \"%s\"".as_ptr(), msg3);
        }
        fprintf(stderr, gettext(c"\nMore info with \"".as_ptr()));
        fprintf(stderr, c"%s -h\"\n".as_ptr(), prgname);
    }
}

/// [`print_mainerr`] and then exit 1. Every argument error takes this path.
pub(crate) unsafe fn mainerr(msg1: *const c_char, msg2: *const c_char, msg3: *const c_char) -> ! {
    // SAFETY: as `print_mainerr`; `os_exit` does not return.
    unsafe {
        print_mainerr(msg1, msg2, msg3);
        os_exit(1);
    }
}
