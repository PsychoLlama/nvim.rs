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
/// The `:silent`, `:noautocmd`, `:keepmarks` … command modifiers, as the
/// bits `cmdmod_T::cmod_flags` carries.  The field is a `c_int` and every
/// transpiled test site casts, so these stay `c_uint` until the modules
/// reading them are rewritten and the casts go with them.
pub type CmdModFlags = ::core::ffi::c_uint;
pub const CMOD_SANDBOX: CmdModFlags = 1;
pub const CMOD_SILENT: CmdModFlags = 2;
pub const CMOD_ERRSILENT: CmdModFlags = 4;
pub const CMOD_UNSILENT: CmdModFlags = 8;
pub const CMOD_NOAUTOCMD: CmdModFlags = 16;
pub const CMOD_HIDE: CmdModFlags = 32;
pub const CMOD_BROWSE: CmdModFlags = 64;
pub const CMOD_CONFIRM: CmdModFlags = 128;
pub const CMOD_KEEPALT: CmdModFlags = 256;
pub const CMOD_KEEPMARKS: CmdModFlags = 512;
pub const CMOD_KEEPJUMPS: CmdModFlags = 1024;
pub const CMOD_LOCKMARKS: CmdModFlags = 2048;
pub const CMOD_KEEPPATTERNS: CmdModFlags = 4096;
pub const CMOD_NOSWAPFILE: CmdModFlags = 8192;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdmod_T {
    /// A set of [`CmdModFlags`] bits.
    pub cmod_flags: ::core::ffi::c_int,
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
