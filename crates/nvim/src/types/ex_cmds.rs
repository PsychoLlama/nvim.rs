#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
pub struct CmdParseInfo {
    pub cmdmod: cmdmod_T,
    pub magic: CmdParseInfo_magic,
}
#[derive(Copy, Clone)]
pub struct CmdParseInfo_magic {
    pub file: bool,
    pub bar: bool,
}
/// A reader `do_cmdline` pulls its lines from, given a `(c, cookie, indent,
/// do_concat)` and returning allocated memory or null at the end of input.
///
/// Spelled separately from [`LineGetter`] so a caller can name the bare
/// function type -- comparing two readers means `ptr::fn_addr_eq` on it.
pub type LineGetterFn = unsafe fn(
    ::core::ffi::c_int,
    *mut ::core::ffi::c_void,
    ::core::ffi::c_int,
    bool,
) -> *mut ::core::ffi::c_char;
pub type LineGetter = Option<LineGetterFn>;
#[derive(Copy, Clone)]
pub struct SubReplacementString {
    pub sub: *mut ::core::ffi::c_char,
    pub timestamp: Timestamp,
    pub additional_data: *mut AdditionalData,
}
pub type cmd_addr_T = ::core::ffi::c_uint;
crate::flag_set! {
    /// The `:silent`, `:noautocmd`, `:keepmarks` … command modifiers, as the
    /// bits [`cmdmod_T::cmod_flags`] carries.
    pub struct CmdModFlags;

    /// `:sandbox` -- the command runs with `sandbox` raised.
    const SANDBOX = 1;
    /// `:silent` -- do not echo what the command says.
    const SILENT = 2;
    /// `:silent!` -- do not show its errors either.
    const ERRSILENT = 4;
    /// `:unsilent` -- say it even inside a `:silent`.
    const UNSILENT = 8;
    /// `:noautocmd` -- fire no autocommands for the duration.
    const NOAUTOCMD = 16;
    /// `:hide` -- a buffer left behind becomes hidden rather than unloaded.
    const HIDE = 32;
    /// `:browse` -- ask for a file name. There is no file dialog, so this
    /// only ever reaches the "not supported" arm.
    const BROWSE = 64;
    /// `:confirm` -- prompt before a destructive step.
    const CONFIRM = 128;
    /// `:keepalt` -- leave the alternate file alone.
    const KEEPALT = 256;
    /// `:keepmarks` -- leave the marks alone.
    const KEEPMARKS = 512;
    /// `:keepjumps` -- leave the jump list and the `\'` mark alone.
    const KEEPJUMPS = 1024;
    /// `:lockmarks` -- do not move the marks for lines the command adds or
    /// removes.
    const LOCKMARKS = 2048;
    /// `:keeppatterns` -- leave the search history alone.
    const KEEPPATTERNS = 4096;
    /// `:noswapfile` -- a buffer the command opens gets no swap file.
    const NOSWAPFILE = 8192;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdmod_T {
    pub cmod_flags: CmdModFlags,
    pub cmod_split: ::core::ffi::c_int,
    pub cmod_tab: ::core::ffi::c_int,
    pub cmod_filter_pat: *mut ::core::ffi::c_char,
    pub cmod_filter_regmatch: regmatch_T,
    pub cmod_filter_force: bool,
    pub cmod_verbose: ::core::ffi::c_int,
    pub cmod_save_ei: *mut ::core::ffi::c_char,
    pub cmod_did_sandbox: ::core::ffi::c_int,
    pub cmod_verbose_save: OptInt,
    pub cmod_save_msg_silent: ::core::ffi::c_int,
    pub cmod_save_msg_scroll: ::core::ffi::c_int,
    pub cmod_did_esilent: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct exarg {
    pub arg: *mut ::core::ffi::c_char,
    pub args: *mut *mut ::core::ffi::c_char,
    pub arglens: *mut size_t,
    pub argc: size_t,
    pub nextcmd: *mut ::core::ffi::c_char,
    pub cmd: *mut ::core::ffi::c_char,
    pub cmdlinep: *mut *mut ::core::ffi::c_char,
    pub cmdline_tofree: *mut ::core::ffi::c_char,
    pub cmdidx: cmdidx_T,
    pub argt: uint32_t,
    pub skip: ::core::ffi::c_int,
    pub forceit: ::core::ffi::c_int,
    pub addr_count: ::core::ffi::c_int,
    pub line1: linenr_T,
    pub line2: linenr_T,
    pub addr_type: cmd_addr_T,
    pub flags: ::core::ffi::c_int,
    pub do_ecmd_cmd: *mut ::core::ffi::c_char,
    pub do_ecmd_lnum: linenr_T,
    pub append: ::core::ffi::c_int,
    pub usefilter: ::core::ffi::c_int,
    pub amount: ::core::ffi::c_int,
    pub regname: ::core::ffi::c_int,
    pub force_bin: ::core::ffi::c_int,
    pub read_edit: ::core::ffi::c_int,
    pub mkdir_p: ::core::ffi::c_int,
    pub force_ff: ::core::ffi::c_int,
    pub force_enc: ::core::ffi::c_int,
    pub bad_char: ::core::ffi::c_int,
    pub useridx: ::core::ffi::c_int,
    pub errmsg: *mut ::core::ffi::c_char,
    pub ea_getline: LineGetter,
    pub cookie: *mut ::core::ffi::c_void,
    pub cstack: *mut cstack_T,
}
pub type exarg_T = exarg;

impl Default for exarg {
    /// The all-zero `exarg_T` that `CLEAR_FIELD(ea)` produces upstream.
    fn default() -> Self {
        exarg {
            arg: ::core::ptr::null_mut(),
            args: ::core::ptr::null_mut(),
            arglens: ::core::ptr::null_mut(),
            argc: 0,
            nextcmd: ::core::ptr::null_mut(),
            cmd: ::core::ptr::null_mut(),
            cmdlinep: ::core::ptr::null_mut(),
            cmdline_tofree: ::core::ptr::null_mut(),
            cmdidx: 0,
            argt: 0,
            skip: 0,
            forceit: 0,
            addr_count: 0,
            line1: 0,
            line2: 0,
            addr_type: 0,
            flags: 0,
            do_ecmd_cmd: ::core::ptr::null_mut(),
            do_ecmd_lnum: 0,
            append: 0,
            usefilter: 0,
            amount: 0,
            regname: 0,
            force_bin: 0,
            read_edit: 0,
            mkdir_p: 0,
            force_ff: 0,
            force_enc: 0,
            bad_char: 0,
            useridx: 0,
            errmsg: ::core::ptr::null_mut(),
            ea_getline: None,
            cookie: ::core::ptr::null_mut(),
            cstack: ::core::ptr::null_mut(),
        }
    }
}
