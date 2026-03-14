//! Startup initialization functions
//!
//! Implements `rs_init_params`, `rs_init_startuptime`, `rs_check_and_set_isatty`,
//! `rs_init_path`, `rs_get_fname`, `rs_set_window_layout` replacing static C functions.

use std::ffi::{c_char, c_int};

/// Maximum number of commands from + or -c arguments (must match C MAX_ARG_CMDS).
const MAX_ARG_CMDS: usize = 10;

/// Mirror of `mparm_T` from main.h.
///
/// IMPORTANT: this must exactly match the C struct layout.
/// Fields mirror main.h exactly (same order, same types, same alignment).
#[repr(C)]
pub struct MparmT {
    pub argc: c_int,
    pub argv: *mut *mut c_char,

    pub use_vimrc: *mut c_char,
    pub clean: bool,

    pub n_commands: c_int,
    pub commands: [*mut c_char; MAX_ARG_CMDS],
    pub cmds_tofree: [c_char; MAX_ARG_CMDS],
    pub n_pre_commands: c_int,
    pub pre_commands: [*mut c_char; MAX_ARG_CMDS],
    pub luaf: *mut c_char,
    pub lua_arg0: c_int,

    pub edit_type: c_int,
    pub tagname: *mut c_char,
    pub use_ef: *mut c_char,

    pub input_istext: bool,

    pub no_swap_file: c_int,
    pub use_debug_break_level: c_int,
    pub window_count: c_int,
    pub window_layout: c_int,

    pub diff_mode: c_int,

    pub listen_addr: *mut c_char,
    pub remote: c_int,
    pub server_addr: *mut c_char,
    pub scriptin: *mut c_char,
    pub scriptout: *mut c_char,
    pub scriptout_append: bool,
    pub had_stdin_file: bool,
}

// Values for window_layout (must match C enums in main.c)
const WIN_HOR: c_int = 1; // "-o" horizontally split
const WIN_VER: c_int = 2; // "-O" vertically split

// v:variable indices (from eval_defs.h VvVars enum, 0-indexed)
// Verified by counting enum entries in eval_defs.h
const VV_PROGNAME: c_int = 27;
const VV_PROGPATH: c_int = 60;

// MAXPATHL value (matches C MAXPATHL on Linux/macOS)
const MAXPATHL: usize = 4096;

// Standard file descriptor numbers
const STDIN_FILENO: c_int = 0;
const STDOUT_FILENO: c_int = 1;
const STDERR_FILENO: c_int = 2;

// Extern C declarations needed
unsafe extern "C" {
    fn os_isatty(fd: c_int) -> bool;
    fn os_exepath(buf: *mut c_char, len: *mut usize) -> c_int;
    fn path_guess_exepath(exename: *const c_char, buf: *mut c_char, buflen: usize);
    fn set_vim_var_string(idx: c_int, val: *const c_char, len: c_int);
    fn path_tail(fname: *const c_char) -> *mut c_char;
    fn time_init(fname: *const c_char, process_name: *const c_char);
    fn time_start(message: *const c_char);
    fn rs_diffopt_horizontal() -> c_int;

    static mut stdin_isatty: bool;
    static mut stdout_isatty: bool;
    static mut stderr_isatty: bool;

    // For os_setenv_append_path on Windows
    #[cfg(windows)]
    fn os_setenv_append_path(exepath: *const c_char);
}

// GARGLIST access - thin C function exposed from main.c
unsafe extern "C" {
    fn nvim_garglist_name(idx: c_int) -> *mut c_char;
}

/// Initialize params struct with default values.
///
/// Zeroes the struct and sets defaults (debug_break_level=-1, window_count=-1, etc.)
///
/// # Safety
/// `paramp` must be a valid non-null pointer to an `mparm_T`-sized region.
/// `argv` must be a valid argv array with at least `argc` entries.
#[no_mangle]
pub unsafe extern "C" fn rs_init_params(paramp: *mut MparmT, argc: c_int, argv: *mut *mut c_char) {
    std::ptr::write_bytes(paramp, 0, 1);
    let p = &mut *paramp;
    p.argc = argc;
    p.argv = argv;
    p.use_debug_break_level = -1;
    p.window_count = -1;
    p.listen_addr = std::ptr::null_mut();
    p.server_addr = std::ptr::null_mut();
    p.remote = 0;
    p.luaf = std::ptr::null_mut();
    p.lua_arg0 = -1;
}

/// Initialize global startuptime file if "--startuptime" passed as an argument.
///
/// # Safety
/// `paramp` must be a valid pointer to an `mparm_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_init_startuptime(paramp: *const MparmT) {
    let p = &*paramp;
    let argc = p.argc as usize;
    let argv = p.argv;

    let mut is_embed = false;
    for i in 1..argc.saturating_sub(1) {
        let arg = *argv.add(i);
        if ascii_strcasecmp(arg, c"--embed".as_ptr()) == 0 {
            is_embed = true;
            break;
        }
    }

    for i in 1..argc.saturating_sub(1) {
        let arg = *argv.add(i);
        if ascii_strcasecmp(arg, c"--startuptime".as_ptr()) == 0 {
            let fname = *argv.add(i + 1);
            let process_name = if is_embed {
                c"Embedded".as_ptr()
            } else {
                c"Primary (or UI client)".as_ptr()
            };
            time_init(fname, process_name);
            time_start(c"--- NVIM STARTING ---".as_ptr());
            break;
        }
    }
}

/// Check stdin/stdout/stderr are ttys and set the corresponding globals.
///
/// # Safety
/// Reads/writes global isatty booleans.
#[no_mangle]
pub unsafe extern "C" fn rs_check_and_set_isatty(_paramp: *mut MparmT) {
    stdin_isatty = os_isatty(STDIN_FILENO);
    stdout_isatty = os_isatty(STDOUT_FILENO);
    stderr_isatty = os_isatty(STDERR_FILENO);
    // TIME_MSG("window checked") is a no-op here (macro only meaningful in C build)
}

/// Sets v:progname and v:progpath. Also modifies $PATH on Windows.
///
/// # Safety
/// `exename` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_init_path(exename: *const c_char) {
    let mut exepath = [0i8; MAXPATHL];
    let mut exepathlen = MAXPATHL;

    if os_exepath(exepath.as_mut_ptr(), &mut exepathlen) != 0 {
        // Fall back to argv[0] - missing procfs or similar issue
        path_guess_exepath(exename, exepath.as_mut_ptr(), MAXPATHL);
    }

    set_vim_var_string(VV_PROGPATH, exepath.as_ptr(), -1);
    set_vim_var_string(VV_PROGNAME, path_tail(exename), -1);

    #[cfg(windows)]
    os_setenv_append_path(exepath.as_ptr());
}

/// Get filename from command line (first entry in global argument list).
///
/// # Safety
/// The global argument list must be initialized.
#[export_name = "get_fname"]
#[allow(clippy::must_use_candidate)]
pub unsafe extern "C" fn rs_get_fname(_parmp: *mut MparmT, _cwd: *mut c_char) -> *mut c_char {
    nvim_garglist_name(0)
}

/// Decide about window layout for diff mode after reading vimrc.
///
/// # Safety
/// `paramp` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_set_window_layout(paramp: *mut MparmT) {
    let p = &mut *paramp;
    if p.diff_mode != 0 && p.window_layout == 0 {
        if rs_diffopt_horizontal() != 0 {
            p.window_layout = WIN_HOR;
        } else {
            p.window_layout = WIN_VER;
        }
    }
}

/// ASCII case-insensitive strcmp.
///
/// # Safety
/// Both pointers must be valid nul-terminated C strings.
unsafe fn ascii_strcasecmp(a: *const c_char, b: *const c_char) -> c_int {
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
