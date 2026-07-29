//! What the process says when it is asked, or when the command line
//! is wrong: `--help`, `--version`, and the argument errors.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn mainerr(
    mut msg1: *const c_char,
    mut msg2: *const c_char,
    mut msg3: *const c_char,
) -> ! {
    print_mainerr(msg1, msg2, msg3);
    os_exit(1 as c_int);
}

pub(crate) unsafe extern "C" fn print_mainerr(
    mut msg1: *const c_char,
    mut msg2: *const c_char,
    mut msg3: *const c_char,
) {
    let mut prgname: *mut c_char = path_tail(argv0.get());
    signal_stop();
    fprintf(
        stderr,
        b"%s: %s\0".as_ptr() as *const c_char,
        prgname,
        gettext(msg1),
    );
    if !msg2.is_null() {
        fprintf(stderr, b": \"%s\"\0".as_ptr() as *const c_char, msg2);
    }
    if !msg3.is_null() {
        fprintf(stderr, b": \"%s\"\0".as_ptr() as *const c_char, msg3);
    }
    fprintf(
        stderr,
        gettext(b"\nMore info with \"\0".as_ptr() as *const c_char),
    );
    fprintf(stderr, b"%s -h\"\n\0".as_ptr() as *const c_char, prgname);
}

pub(crate) unsafe extern "C" fn version() {
    nlua_init(
        ::core::ptr::null_mut::<*mut c_char>(),
        0 as c_int,
        -1 as c_int,
    );
    info_message.set(true_0 != 0);
    list_version();
    msg_putchar('\n' as c_int);
    msg_didout.set(false_0 != 0);
}

pub(crate) unsafe extern "C" fn usage() {
    signal_stop();
    printf(gettext(b"Usage:\n\0".as_ptr() as *const c_char));
    printf(gettext(
        b"  nvim [options] [file ...]\n\0".as_ptr() as *const c_char
    ));
    printf(gettext(b"\nOptions:\n\0".as_ptr() as *const c_char));
    printf(gettext(
        b"  --cmd <cmd>           Execute <cmd> before any config\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  +<cmd>, -c <cmd>      Execute <cmd> after config and first file\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  -l <script> [args...] Execute Lua <script> (with optional args)\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  -S <session>          Source <session> after loading the first file\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  -s <scriptin>         Read Normal mode commands from <scriptin>\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  -u <config>           Use this config file\n\0".as_ptr() as *const c_char,
    ));
    printf(b"\n\0".as_ptr() as *const c_char);
    printf(gettext(
        b"  -d                    Diff mode\n\0".as_ptr() as *const c_char
    ));
    printf(gettext(
        b"  -es, -Es              Silent (batch) mode\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  -h, --help            Print this help message\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  -i <shada>            Use this shada file\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  -n                    No swap file, use memory only\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  -o[N]                 Open N windows (default: one per file)\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  -O[N]                 Open N vertical windows (default: one per file)\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  -p[N]                 Open N tab pages (default: one per file)\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  -R                    Read-only (view) mode\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  -v, --version         Print version information\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  -V[N][file]           Verbose [level][file]\n\0".as_ptr() as *const c_char,
    ));
    printf(b"\n\0".as_ptr() as *const c_char);
    printf(gettext(
        b"  --                    Only file names after this\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  --api-info            Write msgpack-encoded API metadata to stdout\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  --clean               \"Factory defaults\" (skip user config and plugins, shada)\n\0"
            .as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  --embed               Use stdin/stdout as a msgpack-rpc channel\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  --headless            Don't start a user interface\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  --listen <address>    Serve RPC API from this address\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  --remote[-subcommand] Execute commands remotely on a server\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  --server <address>    Connect to this Nvim server\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  --startuptime <file>  Write startup timing messages to <file>\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"\nSee \":help startup-options\" for all options.\n\0".as_ptr() as *const c_char,
    ));
}
