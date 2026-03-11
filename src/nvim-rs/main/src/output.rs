//! Output functions: usage, version, error printing
//!
//! Implements `rs_usage`, `rs_version`, `rs_print_mainerr`, `rs_mainerr`
//! replacing the static C functions in main.c.

use std::ffi::{c_char, c_int};
use std::io::Write as _;

// Extern C declarations needed by these functions
unsafe extern "C" {
    fn signal_stop();
    fn path_tail(fname: *const c_char) -> *mut c_char;
    fn nlua_init(argv: *mut *mut c_char, argc: c_int, lua_arg0: c_int);
    fn list_version();
    fn msg_putchar(c: c_int);
    static mut info_message: bool;
    static mut msg_didout: bool;
    /// argv0 exposed as a non-static global from main.c
    static mut nvim_argv0: *mut c_char;
    fn os_exit(r: c_int) -> !;
    fn gettext(msgid: *const c_char) -> *mut c_char;
}

/// Convert a C string pointer to a Rust `&str` (lossy, for printing).
///
/// # Safety
/// `ptr` must be a valid, nul-terminated C string.
unsafe fn c_str_to_str<'a>(ptr: *const c_char) -> &'a str {
    if ptr.is_null() {
        return "";
    }
    let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
    cstr.to_str().unwrap_or("")
}

/// Prints help message for "nvim -h" or "nvim --help".
///
/// # Safety
/// Calls C functions (`signal_stop`).
#[no_mangle]
pub unsafe extern "C" fn rs_usage() {
    signal_stop();

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let _ = writeln!(out, "Usage:");
    let _ = writeln!(out, "  nvim [options] [file ...]");
    let _ = writeln!(out, "\nOptions:");
    let _ = writeln!(
        out,
        "  --cmd <cmd>           Execute <cmd> before any config"
    );
    let _ = writeln!(
        out,
        "  +<cmd>, -c <cmd>      Execute <cmd> after config and first file"
    );
    let _ = writeln!(
        out,
        "  -l <script> [args...] Execute Lua <script> (with optional args)"
    );
    let _ = writeln!(
        out,
        "  -S <session>          Source <session> after loading the first file"
    );
    let _ = writeln!(
        out,
        "  -s <scriptin>         Read Normal mode commands from <scriptin>"
    );
    let _ = writeln!(out, "  -u <config>           Use this config file");
    let _ = writeln!(out);
    let _ = writeln!(out, "  -d                    Diff mode");
    let _ = writeln!(out, "  -es, -Es              Silent (batch) mode");
    let _ = writeln!(out, "  -h, --help            Print this help message");
    let _ = writeln!(out, "  -i <shada>            Use this shada file");
    let _ = writeln!(out, "  -n                    No swap file, use memory only");
    let _ = writeln!(
        out,
        "  -o[N]                 Open N windows (default: one per file)"
    );
    let _ = writeln!(
        out,
        "  -O[N]                 Open N vertical windows (default: one per file)"
    );
    let _ = writeln!(
        out,
        "  -p[N]                 Open N tab pages (default: one per file)"
    );
    let _ = writeln!(out, "  -R                    Read-only (view) mode");
    let _ = writeln!(out, "  -v, --version         Print version information");
    let _ = writeln!(out, "  -V[N][file]           Verbose [level][file]");
    let _ = writeln!(out);
    let _ = writeln!(out, "  --                    Only file names after this");
    let _ = writeln!(
        out,
        "  --api-info            Write msgpack-encoded API metadata to stdout"
    );
    let _ = writeln!(
        out,
        "  --clean               \"Factory defaults\" (skip user config and plugins, shada)"
    );
    let _ = writeln!(
        out,
        "  --embed               Use stdin/stdout as a msgpack-rpc channel"
    );
    let _ = writeln!(out, "  --headless            Don't start a user interface");
    let _ = writeln!(
        out,
        "  --listen <address>    Serve RPC API from this address"
    );
    let _ = writeln!(
        out,
        "  --remote[-subcommand] Execute commands remotely on a server"
    );
    let _ = writeln!(out, "  --server <address>    Connect to this Nvim server");
    let _ = writeln!(
        out,
        "  --startuptime <file>  Write startup timing messages to <file>"
    );
    let _ = writeln!(out, "\nSee \":help startup-options\" for all options.");
}

/// Prints version information for "nvim -v" or "nvim --version".
///
/// # Safety
/// Calls C functions (`nlua_init`, `list_version`, `msg_putchar`).
#[no_mangle]
pub unsafe extern "C" fn rs_version() {
    nlua_init(std::ptr::null_mut(), 0, -1);
    info_message = true; // use stdout, not stderr
    list_version();
    msg_putchar(b'\n' as c_int);
    msg_didout = false;
}

/// Prints a message of the form "{prog}: {msg1}: {msg2}: {msg3}" to stderr.
///
/// # Safety
/// `msg1` must be a valid C string. `msg2` and `msg3` may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_print_mainerr(
    msg1: *const c_char,
    msg2: *const c_char,
    msg3: *const c_char,
) {
    signal_stop();

    let prgname = path_tail(nvim_argv0);
    let prgname_str = c_str_to_str(prgname);
    // gettext translates msg1; fall back to raw string if gettext not available
    let msg1_raw = c_str_to_str(msg1);
    let msg1_tr = c_str_to_str(gettext(msg1));
    let msg1_out = if msg1_tr.is_empty() {
        msg1_raw
    } else {
        msg1_tr
    };

    let stderr = std::io::stderr();
    let mut err = stderr.lock();
    let _ = write!(err, "{prgname_str}: {msg1_out}");
    if !msg2.is_null() {
        let _ = write!(err, ": \"{}\"", c_str_to_str(msg2));
    }
    if !msg3.is_null() {
        let _ = write!(err, ": \"{}\"", c_str_to_str(msg3));
    }
    let _ = writeln!(err, "\nMore info with \"{prgname_str} -h\"");
}

/// Prints a message then exits with code 1.
///
/// # Safety
/// Pointer arguments must be valid C strings (msg2/msg3 may be null).
#[no_mangle]
pub unsafe extern "C" fn rs_mainerr(
    msg1: *const c_char,
    msg2: *const c_char,
    msg3: *const c_char,
) -> ! {
    rs_print_mainerr(msg1, msg2, msg3);
    os_exit(1);
}
