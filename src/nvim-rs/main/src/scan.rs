//! Command-line argument scanning
//!
//! Implements `rs_command_line_scan` replacing the static C function
//! `command_line_scan` in `src/nvim/main.c`.

use crate::setup::MparmT;
use std::ffi::{c_char, c_int};

// =============================================================================
// Constants
// =============================================================================

// Values for edit_type (must match C enums in main.c)
const EDIT_NONE: c_int = 0;
const EDIT_FILE: c_int = 1;
const EDIT_STDIN: c_int = 2;
const EDIT_TAG: c_int = 3;
const EDIT_QF: c_int = 4;

// Values for window_layout (must match C enums in main.c)
const WIN_HOR: c_int = 1;
const WIN_VER: c_int = 2;
const WIN_TABS: c_int = 3;

// MAX_ARG_CMDS (must match C MAX_ARG_CMDS)
const MAX_ARG_CMDS: c_int = 10;

// Option indices (auto-generated C enum kOpt* values, verified stable)
const K_OPT_ARABIC: c_int = 3;
const K_OPT_KEYMAP: c_int = 158;
const K_OPT_RIGHTLEFT: c_int = 238;
const K_OPT_SHADAFILE: c_int = 255;
const K_OPT_VERBOSEFILE: c_int = 340;
const K_OPT_WINDOW: c_int = 358;

// OptVal type enum values (must match kOptValType* in C)
const OPT_VAL_TYPE_BOOLEAN: c_int = 0;
const OPT_VAL_TYPE_NUMBER: c_int = 1;
const OPT_VAL_TYPE_STRING: c_int = 2;

// Boolean option values
const K_TRUE: c_int = 1;

// VV_SWAPCOMMAND index (from eval_defs.h VvVars)
const VV_SWAPCOMMAND: c_int = 35;

// SESSION_FILE default name
const SESSION_FILE: &[u8] = b"Session.vim\0";

// IOSIZE (must match C IOSIZE)
const IOSIZE: usize = 1025;

// =============================================================================
// OptVal types (duplicated from option crate to avoid dependency)
// =============================================================================

type OptInt = i64;

/// Nvim String type (matches api/private/defs.h)
#[repr(C)]
#[derive(Clone, Copy)]
struct NvimString {
    data: *mut c_char,
    size: usize,
}

/// Union data for OptVal
#[repr(C)]
#[derive(Clone, Copy)]
union OptValData {
    boolean: c_int,
    number: OptInt,
    string: NvimString,
}

/// Option value (matches OptVal in option_defs.h)
#[repr(C)]
#[derive(Clone, Copy)]
struct OptVal {
    type_: c_int,
    data: OptValData,
}

impl OptVal {
    fn boolean(val: bool) -> Self {
        Self {
            type_: OPT_VAL_TYPE_BOOLEAN,
            data: OptValData {
                boolean: if val { K_TRUE } else { 0 },
            },
        }
    }

    fn number(val: OptInt) -> Self {
        Self {
            type_: OPT_VAL_TYPE_NUMBER,
            data: OptValData { number: val },
        }
    }

    /// Create a string OptVal from a raw C string pointer (no ownership transfer).
    ///
    /// # Safety
    /// `s` must be a valid nul-terminated C string that lives long enough.
    unsafe fn cstr(s: *const c_char) -> Self {
        let len = c_strlen(s);
        Self {
            type_: OPT_VAL_TYPE_STRING,
            data: OptValData {
                string: NvimString {
                    data: s as *mut c_char,
                    size: len,
                },
            },
        }
    }
}

// =============================================================================
// Error message constants
// =============================================================================

static ERR_ARG_MISSING: &[u8] = b"Argument missing after\0";
static ERR_OPT_GARBAGE: &[u8] = b"Garbage after option argument\0";
static ERR_OPT_UNKNOWN: &[u8] = b"Unknown option argument\0";
static ERR_TOO_MANY_ARGS: &[u8] = b"Too many edit arguments\0";
static ERR_EXTRA_CMD: &[u8] =
    b"Too many \"+command\", \"-c command\" or \"--cmd command\" arguments\0";

// =============================================================================
// FFI declarations
// =============================================================================

unsafe extern "C" {
    // Option setting
    fn set_option_value_give_err(opt_idx: c_int, value: OptVal, opt_flags: c_int);
    fn set_options_bin(old_bin: c_int, new_bin: c_int, opt_flags: c_int);
    fn reset_modifiable();

    // Path utilities
    fn os_isdir(name: *const c_char) -> bool;
    fn concat_fnames(fname1: *const c_char, fname2: *const c_char, sep: bool) -> *mut c_char;
    fn path_tail(fname: *const c_char) -> *mut c_char;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xmalloc(size: usize) -> *mut std::ffi::c_void;
    fn xfree(p: *mut std::ffi::c_void);

    // API info output
    fn api_metadata_raw() -> NvimApiMetadata;
    fn os_write(fd: c_int, buf: *const c_char, size: usize, binary: bool) -> isize;

    // Error reporting
    fn semsg(fmt: *const c_char, ...);
    fn vim_snprintf(buf: *mut c_char, buflen: usize, fmt: *const c_char, ...) -> c_int;
    fn uv_strerror(error: c_int) -> *const c_char;

    // Exit
    fn os_exit(r: c_int) -> !;

    // Arglist
    fn nvim_al_get_global_alist() -> *mut std::ffi::c_void;
    fn nvim_al_ga_ptr(al: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn nvim_al_ga_grow(ga: *mut std::ffi::c_void, n: c_int);
    fn nvim_garglist_name(idx: c_int) -> *mut c_char;
    fn alist_add(al: *mut std::ffi::c_void, fname: *mut c_char, set_fnum: c_int);

    // Globals
    static mut exmode_active: bool;
    static mut silent_mode: bool;
    static mut headless_mode: bool;
    static mut embedded_mode: bool;
    static mut readonlymode: bool;
    static mut recoverymode: c_int;
    static mut p_lpl: bool;
    static mut p_verbose: c_int;
    static mut p_uc: c_int;
    static mut p_write: bool;
    static mut p_shadafile: *mut c_char;
    static mut nlua_disable_preload: bool;

    // Curbuf accessors
    fn nvim_buf_get_b_p_bin(buf: *mut std::ffi::c_void) -> c_int;
    fn nvim_buf_set_b_p_bin(buf: *mut std::ffi::c_void, val: c_int);
    fn nvim_buf_set_b_p_ro_true(buf: *mut std::ffi::c_void);
    fn nvim_get_curbuf() -> *mut std::ffi::c_void;

    // GARGCOUNT
    fn nvim_al_GARGCOUNT() -> c_int;

    // set_vim_var_string
    fn set_vim_var_string(idx: c_int, val: *const c_char, len: c_int);

    // From this crate
    fn rs_mainerr(msg1: *const c_char, msg2: *const c_char, msg3: *const c_char) -> !;
    fn rs_version();
    fn rs_usage();
    fn edit_stdin(parmp: *const MparmT) -> bool;

    // Windows-only
    #[cfg(windows)]
    fn os_homedir() -> *const c_char;
    #[cfg(windows)]
    fn path_fix_case(name: *mut c_char);
}

/// NvimApiMetadata return type for api_metadata_raw
#[repr(C)]
struct NvimApiMetadata {
    data: *const c_char,
    size: usize,
}

// =============================================================================
// Helpers
// =============================================================================

/// strlen for a raw C string pointer.
///
/// # Safety
/// `s` must be a valid nul-terminated C string.
unsafe fn c_strlen(s: *const c_char) -> usize {
    let mut i = 0usize;
    while *s.add(i) != 0 {
        i += 1;
    }
    i
}

/// ASCII case-insensitive strcmp (matches STRICMP).
///
/// # Safety
/// Both pointers must be valid nul-terminated C strings.
unsafe fn stricmp(a: *const c_char, b: *const c_char) -> c_int {
    let mut i = 0usize;
    loop {
        let ac = (*a.add(i) as u8).to_ascii_lowercase();
        let bc = (*b.add(i) as u8).to_ascii_lowercase();
        if ac != bc {
            return ac as c_int - bc as c_int;
        }
        if ac == 0 {
            return 0;
        }
        i += 1;
    }
}

/// ASCII case-insensitive strncmp (matches STRNICMP).
///
/// # Safety
/// Both pointers must be valid C strings with at least `n` readable bytes.
unsafe fn strnicmp(a: *const c_char, b: *const c_char, n: usize) -> c_int {
    for i in 0..n {
        let ac = (*a.add(i) as u8).to_ascii_lowercase();
        let bc = (*b.add(i) as u8).to_ascii_lowercase();
        if ac != bc {
            return ac as c_int - bc as c_int;
        }
        if ac == 0 {
            return 0;
        }
    }
    0
}

/// strequal: exact byte comparison (like C's `strcmp(a,b)==0`).
///
/// # Safety
/// Both pointers must be valid nul-terminated C strings.
unsafe fn strequal(a: *const c_char, b: *const c_char) -> bool {
    let mut i = 0usize;
    loop {
        let ac = *a.add(i);
        let bc = *b.add(i);
        if ac != bc {
            return false;
        }
        if ac == 0 {
            return true;
        }
        i += 1;
    }
}

/// Handle the `scripterror` goto case: duplicate script open attempt.
///
/// # Safety
/// argv must be valid: argv[-1] is the flag, argv[0] is the file.
unsafe fn scripterror(argv: *mut *mut c_char) -> ! {
    // Use a local stack buffer to avoid mutable-static-reference issues with IObuff.
    let mut buf = [0u8; IOSIZE];
    let buf_ptr = buf.as_mut_ptr() as *mut c_char;
    vim_snprintf(
        buf_ptr,
        IOSIZE,
        c"Attempt to open script file again: \"%s %s\"\n".as_ptr(),
        *argv.sub(1),
        *argv,
    );
    let _ = libc_write_stderr(buf_ptr as *const u8);
    os_exit(2);
}

/// Write to stderr (fd=2) using os_write.
///
/// # Safety
/// `msg` must be a valid nul-terminated string.
unsafe fn libc_write_stderr(msg: *const u8) -> isize {
    let len = {
        let mut i = 0usize;
        while *msg.add(i) != 0 {
            i += 1;
        }
        i
    };
    os_write(2, msg as *const c_char, len, false)
}

// =============================================================================
// Main implementation
// =============================================================================

/// Scan command-line arguments and populate `parmp`.
///
/// Direct replacement for the C `command_line_scan` function.
/// Called from the C `command_line_scan` thin wrapper (which also calls `TIME_MSG`).
///
/// # Safety
/// `parmp` must be a valid, non-null pointer to an initialized `MparmT`.
#[no_mangle]
pub unsafe extern "C" fn rs_command_line_scan(parmp: *mut MparmT) {
    let p = &mut *parmp;

    let total_argc = p.argc;
    let orig_argv = p.argv;

    // Local mutable copies of argc/argv pointers (like C locals)
    let mut argc = total_argc - 1;
    let mut argv: *mut *mut c_char = orig_argv.add(1);
    let mut argv_idx: c_int = 1; // index into argv[0][]
    let mut had_minmin = false;

    while argc > 0 {
        let arg0: *const c_char = *argv;

        if *arg0 == b'+' as c_char && !had_minmin {
            // "+" or "+{number}" or "+/{pat}" or "+{command}" argument
            if p.n_commands >= MAX_ARG_CMDS {
                rs_mainerr(
                    ERR_EXTRA_CMD.as_ptr() as *const c_char,
                    std::ptr::null(),
                    std::ptr::null(),
                );
            }
            argv_idx = -1;
            if *arg0.add(1) == 0 {
                p.commands[p.n_commands as usize] = c"$".as_ptr() as *mut c_char;
            } else {
                p.commands[p.n_commands as usize] = arg0.add(1) as *mut c_char;
            }
            p.n_commands += 1;
        } else if *arg0 == b'-' as c_char && !had_minmin {
            // Optional argument
            let mut want_argument = false;
            let c = *arg0.add(argv_idx as usize) as u8;
            argv_idx += 1;

            match c {
                0 => {
                    // "nvim -"  read from stdin
                    if exmode_active {
                        // "nvim -e -" silent mode
                        silent_mode = true;
                        p.no_swap_file = 1;
                    } else {
                        if p.edit_type > EDIT_STDIN {
                            rs_mainerr(
                                ERR_TOO_MANY_ARGS.as_ptr() as *const c_char,
                                arg0,
                                std::ptr::null(),
                            );
                        }
                        p.had_stdin_file = true;
                        p.edit_type = EDIT_STDIN;
                    }
                    argv_idx = -1;
                }

                b'-' => {
                    // "--" long options
                    let rest: *const c_char = arg0.add(argv_idx as usize);

                    if stricmp(rest, c"help".as_ptr()) == 0 {
                        rs_usage();
                        os_exit(0);
                    } else if stricmp(rest, c"version".as_ptr()) == 0 {
                        rs_version();
                        os_exit(0);
                    } else if stricmp(rest, c"api-info".as_ptr()) == 0 {
                        #[cfg(windows)]
                        {
                            // set stdout to binary to avoid crlf in --api-info output
                            extern "C" {
                                fn _setmode(fd: c_int, mode: c_int) -> c_int;
                            }
                            _setmode(1, 0x8000); // _O_BINARY
                        }
                        let data = api_metadata_raw();
                        let written = os_write(1, data.data, data.size, false);
                        if written < 0 {
                            semsg(
                                c"E5420: Failed to write to file: %s".as_ptr(),
                                uv_strerror(written as c_int),
                            );
                        }
                        os_exit(0);
                    } else if stricmp(rest, c"headless".as_ptr()) == 0 {
                        headless_mode = true;
                    } else if stricmp(rest, c"embed".as_ptr()) == 0 {
                        embedded_mode = true;
                    } else if strnicmp(rest, c"listen".as_ptr(), 6) == 0 {
                        want_argument = true;
                        argv_idx += 6;
                    } else if strnicmp(rest, c"literal".as_ptr(), 7) == 0 {
                        // Do nothing: file args are always literal. #7679
                    } else if strnicmp(rest, c"remote".as_ptr(), 6) == 0 {
                        p.remote = total_argc - argc;
                    } else if strnicmp(rest, c"server".as_ptr(), 6) == 0 {
                        want_argument = true;
                        argv_idx += 6;
                    } else if strnicmp(rest, c"noplugin".as_ptr(), 8) == 0 {
                        p_lpl = false;
                    } else if strnicmp(rest, c"cmd".as_ptr(), 3) == 0 {
                        want_argument = true;
                        argv_idx += 3;
                    } else if strnicmp(rest, c"startuptime".as_ptr(), 11) == 0 {
                        want_argument = true;
                        argv_idx += 11;
                    } else if strnicmp(rest, c"clean".as_ptr(), 5) == 0 {
                        p.use_vimrc = c"NONE".as_ptr() as *mut c_char;
                        p.clean = true;
                        set_option_value_give_err(
                            K_OPT_SHADAFILE,
                            OptVal::cstr(c"NONE".as_ptr()),
                            0,
                        );
                    } else if strnicmp(rest, c"luamod-dev".as_ptr(), 9) == 0 {
                        nlua_disable_preload = true;
                    } else {
                        if *arg0.add(argv_idx as usize) != 0 {
                            rs_mainerr(
                                ERR_OPT_UNKNOWN.as_ptr() as *const c_char,
                                arg0,
                                std::ptr::null(),
                            );
                        }
                        had_minmin = true;
                    }

                    if !want_argument {
                        argv_idx = -1;
                    }
                }

                b'A' => {
                    // "-A" start in Arabic mode.
                    set_option_value_give_err(K_OPT_ARABIC, OptVal::boolean(true), 0);
                }

                b'b' => {
                    // "-b" binary mode.
                    let curbuf = nvim_get_curbuf();
                    let old_bin = nvim_buf_get_b_p_bin(curbuf);
                    set_options_bin(old_bin, 1, 0);
                    nvim_buf_set_b_p_bin(curbuf, 1);
                }

                b'D' => {
                    // "-D" Debugging
                    p.use_debug_break_level = 9999;
                }

                b'd' => {
                    // "-d" diff mode
                    p.diff_mode = 1;
                }

                b'e' => {
                    // "-e" Ex mode
                    exmode_active = true;
                }

                b'E' => {
                    // "-E" Ex mode (with text input)
                    exmode_active = true;
                    p.input_istext = true;
                }

                b'f' => {
                    // "-f" GUI: run in foreground (no-op for nvim)
                }

                b'?' | b'h' => {
                    // "-?" or "-h" give help message
                    rs_usage();
                    os_exit(0);
                }

                b'H' => {
                    // "-H" Hebrew mode: rl + keymap=hebrew
                    set_option_value_give_err(K_OPT_KEYMAP, OptVal::cstr(c"hebrew".as_ptr()), 0);
                    set_option_value_give_err(K_OPT_RIGHTLEFT, OptVal::boolean(true), 0);
                }

                b'M' => {
                    // "-M" no changes or writing of files
                    reset_modifiable();
                    // FALLTHROUGH to 'm'
                    p_write = false;
                }

                b'm' => {
                    // "-m" no writing of files
                    p_write = false;
                }

                b'N' | b'X' => {
                    // "-N" Nocompatible  "-X" Do not connect to X server  (no-op)
                }

                b'n' => {
                    // "-n" no swap file
                    p.no_swap_file = 1;
                }

                b'p' => {
                    // "-p[N]" open N tab pages
                    p.window_count = get_number_arg_local(arg0, &mut argv_idx, 0);
                    p.window_layout = WIN_TABS;
                }

                b'o' => {
                    // "-o[N]" open N horizontal split windows
                    p.window_count = get_number_arg_local(arg0, &mut argv_idx, 0);
                    p.window_layout = WIN_HOR;
                }

                b'O' => {
                    // "-O[N]" open N vertical split windows
                    p.window_count = get_number_arg_local(arg0, &mut argv_idx, 0);
                    p.window_layout = WIN_VER;
                }

                b'q' => {
                    // "-q" QuickFix mode
                    if p.edit_type != EDIT_NONE {
                        rs_mainerr(
                            ERR_TOO_MANY_ARGS.as_ptr() as *const c_char,
                            arg0,
                            std::ptr::null(),
                        );
                    }
                    p.edit_type = EDIT_QF;
                    if *arg0.add(argv_idx as usize) != 0 {
                        // "-q{errorfile}"
                        p.use_ef = arg0.add(argv_idx as usize) as *mut c_char;
                        argv_idx = -1;
                    } else if argc > 1 {
                        // "-q {errorfile}"
                        want_argument = true;
                    }
                }

                b'R' => {
                    // "-R" readonly mode
                    readonlymode = true;
                    let curbuf = nvim_get_curbuf();
                    nvim_buf_set_b_p_ro_true(curbuf);
                    p_uc = 10000; // don't update very often
                }

                b'r' | b'L' => {
                    // "-r" / "-L" recovery mode
                    recoverymode = 1;
                }

                b's' => {
                    if exmode_active {
                        // "-es" silent (batch) Ex-mode
                        silent_mode = true;
                        p.no_swap_file = 1;
                        if p_shadafile.is_null() || *p_shadafile == 0 {
                            set_option_value_give_err(
                                K_OPT_SHADAFILE,
                                OptVal::cstr(c"NONE".as_ptr()),
                                0,
                            );
                        }
                    } else {
                        // "-s {scriptin}" read from script file
                        want_argument = true;
                    }
                }

                b't' => {
                    // "-t {tag}" or "-t{tag}" jump to tag
                    if p.edit_type != EDIT_NONE {
                        rs_mainerr(
                            ERR_TOO_MANY_ARGS.as_ptr() as *const c_char,
                            arg0,
                            std::ptr::null(),
                        );
                    }
                    p.edit_type = EDIT_TAG;
                    if *arg0.add(argv_idx as usize) != 0 {
                        // "-t{tag}"
                        p.tagname = arg0.add(argv_idx as usize) as *mut c_char;
                        argv_idx = -1;
                    } else {
                        // "-t {tag}"
                        want_argument = true;
                    }
                }

                b'v' => {
                    rs_version();
                    os_exit(0);
                }

                b'V' => {
                    // "-V{N}" Verbose level
                    p_verbose = get_number_arg_local(arg0, &mut argv_idx, 10);
                    if *arg0.add(argv_idx as usize) != 0 {
                        set_option_value_give_err(
                            K_OPT_VERBOSEFILE,
                            OptVal::cstr(arg0.add(argv_idx as usize)),
                            0,
                        );
                        argv_idx = c_strlen(arg0) as c_int;
                    }
                }

                b'w' => {
                    // "-w{number}" set window height  OR  "-w {scriptout}" write to script
                    if is_ascii_digit(*arg0.add(argv_idx as usize) as u8) {
                        let n = get_number_arg_local(arg0, &mut argv_idx, 10);
                        set_option_value_give_err(K_OPT_WINDOW, OptVal::number(n as OptInt), 0);
                        // break: argv_idx already advanced past digits
                    } else {
                        want_argument = true;
                    }
                }

                b'c' => {
                    // "-c{command}" or "-c {command}" exec command
                    if *arg0.add(argv_idx as usize) != 0 {
                        if p.n_commands >= MAX_ARG_CMDS {
                            rs_mainerr(
                                ERR_EXTRA_CMD.as_ptr() as *const c_char,
                                std::ptr::null(),
                                std::ptr::null(),
                            );
                        }
                        p.commands[p.n_commands as usize] =
                            arg0.add(argv_idx as usize) as *mut c_char;
                        p.n_commands += 1;
                        argv_idx = -1;
                        // no want_argument; continue loop
                    } else {
                        // FALLTHROUGH to 'S', 'i', 'l', 'u', 'U', 'W'
                        want_argument = true;
                    }
                }

                b'S' | b'i' | b'l' | b'u' | b'U' | b'W' => {
                    want_argument = true;
                }

                _ => {
                    rs_mainerr(
                        ERR_OPT_UNKNOWN.as_ptr() as *const c_char,
                        arg0,
                        std::ptr::null(),
                    );
                }
            }

            // Handle option arguments that need a second argument
            if want_argument {
                // Check for garbage immediately after the option letter
                if *arg0.add(argv_idx as usize) != 0 {
                    rs_mainerr(
                        ERR_OPT_GARBAGE.as_ptr() as *const c_char,
                        arg0,
                        std::ptr::null(),
                    );
                }

                argc -= 1;
                if argc < 1 && c != b'S' {
                    // -S has optional argument
                    rs_mainerr(
                        ERR_ARG_MISSING.as_ptr() as *const c_char,
                        arg0,
                        std::ptr::null(),
                    );
                }
                argv = argv.add(1);
                argv_idx = -1;

                let next_arg: *const c_char = *argv;

                match c {
                    b'c' => {
                        // "-c {command}" execute command
                        if p.n_commands >= MAX_ARG_CMDS {
                            rs_mainerr(
                                ERR_EXTRA_CMD.as_ptr() as *const c_char,
                                std::ptr::null(),
                                std::ptr::null(),
                            );
                        }
                        p.commands[p.n_commands as usize] = next_arg as *mut c_char;
                        p.n_commands += 1;
                    }

                    b'S' => {
                        // "-S {file}" execute Vim script
                        if p.n_commands >= MAX_ARG_CMDS {
                            rs_mainerr(
                                ERR_EXTRA_CMD.as_ptr() as *const c_char,
                                std::ptr::null(),
                                std::ptr::null(),
                            );
                        }
                        let a: *const c_char = if argc < 1 {
                            // "-S" without argument: use default session file name
                            SESSION_FILE.as_ptr() as *const c_char
                        } else if *next_arg == b'-' as c_char {
                            // "-S" followed by another option: use default session file
                            argc += 1;
                            argv = argv.sub(1);
                            SESSION_FILE.as_ptr() as *const c_char
                        } else {
                            next_arg
                        };

                        let a_len = c_strlen(a);
                        let s_size = a_len + 9; // "so " + a + NUL + some slack
                        let s = xmalloc(s_size) as *mut c_char;
                        vim_snprintf(s, s_size, c"so %s".as_ptr(), a);
                        p.cmds_tofree[p.n_commands as usize] = 1;
                        p.commands[p.n_commands as usize] = s;
                        p.n_commands += 1;
                    }

                    b'-' => {
                        // "--cmd {command}", "--listen {addr}", "--server {addr}", "--startuptime <file>"
                        let prev: *const c_char = *argv.sub(1);
                        if strequal(prev, c"--cmd".as_ptr()) {
                            if p.n_pre_commands >= MAX_ARG_CMDS {
                                rs_mainerr(
                                    ERR_EXTRA_CMD.as_ptr() as *const c_char,
                                    std::ptr::null(),
                                    std::ptr::null(),
                                );
                            }
                            p.pre_commands[p.n_pre_commands as usize] = next_arg as *mut c_char;
                            p.n_pre_commands += 1;
                        } else if strequal(prev, c"--listen".as_ptr()) {
                            p.listen_addr = next_arg as *mut c_char;
                        } else if strequal(prev, c"--server".as_ptr()) {
                            p.server_addr = next_arg as *mut c_char;
                        }
                        // "--startuptime <file>" already handled by init_startuptime
                    }

                    b'q' => {
                        // "-q {errorfile}" QuickFix mode
                        p.use_ef = next_arg as *mut c_char;
                    }

                    b'i' => {
                        // "-i {shada}" use for shada
                        set_option_value_give_err(K_OPT_SHADAFILE, OptVal::cstr(next_arg), 0);
                    }

                    b'l' => {
                        // "-l" Lua script: args after "-l"
                        headless_mode = true;
                        silent_mode = true;
                        p_verbose = 1;
                        p.no_swap_file = 1;
                        if p.use_vimrc.is_null() {
                            p.use_vimrc = c"NONE".as_ptr() as *mut c_char;
                        }
                        if p_shadafile.is_null() || *p_shadafile == 0 {
                            set_option_value_give_err(
                                K_OPT_SHADAFILE,
                                OptVal::cstr(c"NONE".as_ptr()),
                                0,
                            );
                        }
                        p.luaf = next_arg as *mut c_char;
                        argc -= 1;
                        if argc >= 0 {
                            // Lua args after "-l <file>"
                            p.lua_arg0 = total_argc - argc;
                            argc = 0;
                        }
                    }

                    b's' => {
                        // "-s {scriptin}" read from script file
                        if !p.scriptin.is_null() {
                            scripterror(argv);
                        }
                        p.scriptin = next_arg as *mut c_char;
                    }

                    b't' => {
                        // "-t {tag}"
                        p.tagname = next_arg as *mut c_char;
                    }

                    b'u' => {
                        // "-u {vimrc}" vim inits file
                        p.use_vimrc = next_arg as *mut c_char;
                    }

                    b'U' => {
                        // "-U {gvimrc}" gvim inits file (ignored)
                    }

                    b'w' => {
                        // "-w {nr}" window size OR "-w {scriptout}" append to script
                        if is_ascii_digit(*next_arg as u8) {
                            let mut tmp_idx: c_int = 0;
                            let n = get_number_arg_local(next_arg, &mut tmp_idx, 10);
                            set_option_value_give_err(K_OPT_WINDOW, OptVal::number(n as OptInt), 0);
                            argv_idx = -1;
                        } else {
                            // FALLTHROUGH to 'W'
                            if !p.scriptout.is_null() {
                                scripterror(argv);
                            }
                            p.scriptout = next_arg as *mut c_char;
                            p.scriptout_append = true;
                        }
                    }

                    b'W' => {
                        // "-W {scriptout}" overwrite script file
                        if !p.scriptout.is_null() {
                            scripterror(argv);
                        }
                        p.scriptout = next_arg as *mut c_char;
                        p.scriptout_append = false;
                    }

                    _ => {}
                }
            }
        } else {
            // File name argument
            argv_idx = -1;

            if p.edit_type > EDIT_STDIN {
                rs_mainerr(
                    ERR_TOO_MANY_ARGS.as_ptr() as *const c_char,
                    arg0,
                    std::ptr::null(),
                );
            }
            p.edit_type = EDIT_FILE;

            // Add the file to the global argument list
            let global_alist = nvim_al_get_global_alist();
            let ga = nvim_al_ga_ptr(global_alist);
            nvim_al_ga_grow(ga, 1);
            let mut fname_p = xstrdup(arg0);

            // On Windows expand "~\" or "~/" prefix to profile directory
            #[cfg(windows)]
            {
                if *fname_p == b'~' as c_char
                    && (*fname_p.add(1) == b'\\' as c_char || *fname_p.add(1) == b'/' as c_char)
                {
                    let home = os_homedir();
                    let home_len = c_strlen(home);
                    let fname_len = c_strlen(fname_p as *const c_char);
                    let size = home_len + fname_len + 1;
                    let expanded = xmalloc(size) as *mut c_char;
                    vim_snprintf(expanded, size, c"%s%s".as_ptr(), home, fname_p.add(1));
                    xfree(fname_p as *mut std::ffi::c_void);
                    fname_p = expanded;
                }
            }

            if p.diff_mode != 0
                && os_isdir(fname_p)
                && nvim_al_GARGCOUNT() > 0
                && !os_isdir(nvim_garglist_name(0))
            {
                let r = concat_fnames(fname_p, path_tail(nvim_garglist_name(0)), true);
                xfree(fname_p as *mut std::ffi::c_void);
                fname_p = r;
            }

            #[cfg(windows)]
            path_fix_case(fname_p);

            let alist_fnum_flag = if edit_stdin(parmp) {
                1 // add buffer nr after exp.
            } else {
                2 // add buffer number now and use curbuf
            };
            alist_add(global_alist, fname_p, alist_fnum_flag);
        }

        // If no more letters after current "-", advance to next argument.
        // argv_idx == -1 means skip to next argument.
        if argv_idx <= 0 || *(*argv).add(argv_idx as usize) == 0 {
            argc -= 1;
            argv = argv.add(1);
            argv_idx = 1;
        }
    }

    if embedded_mode && (silent_mode || !p.luaf.is_null()) {
        rs_mainerr(
            c"--embed conflicts with -es/-Es/-l".as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
        );
    }

    // If there is a "+123" or "-c" command, set v:swapcommand to the first one.
    if p.n_commands > 0 {
        let cmd0: *const c_char = p.commands[0];
        let cmd_len = c_strlen(cmd0);
        let swcmd_len = cmd_len + 3;
        let swcmd = xmalloc(swcmd_len) as *mut c_char;
        vim_snprintf(swcmd, swcmd_len, c":%s\r".as_ptr(), cmd0);
        set_vim_var_string(VV_SWAPCOMMAND, swcmd, -1);
        xfree(swcmd as *mut std::ffi::c_void);
    }
}

/// Inline version of `get_number_arg` for use within scan.rs.
///
/// # Safety
/// `p` must be a valid C string. `idx` must be a valid pointer.
unsafe fn get_number_arg_local(p: *const c_char, idx: &mut c_int, def: c_int) -> c_int {
    let i = *idx as usize;
    let byte = *p.add(i) as u8;
    if is_ascii_digit(byte) {
        let mut result: c_int = 0;
        let mut j = i;
        loop {
            let b = *p.add(j) as u8;
            if !is_ascii_digit(b) {
                break;
            }
            result = result
                .saturating_mul(10)
                .saturating_add((b - b'0') as c_int);
            j += 1;
        }
        *idx = j as c_int;
        result
    } else {
        def
    }
}

/// Check if a byte is an ASCII digit.
#[inline]
fn is_ascii_digit(b: u8) -> bool {
    b.is_ascii_digit()
}
