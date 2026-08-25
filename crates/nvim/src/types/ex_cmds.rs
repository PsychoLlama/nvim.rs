#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Clone)]
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
/// What an Ex command's range counts -- upstream's `ADDR_*`, the value
/// `exarg_T::addr_type`, `CommandDefinition::cmd_addr_type` and
/// `ucmd_T::uc_addr_type` carry.
///
/// A range is `1,5` whatever it addresses; this is what those numbers *are*.
/// `:1,5delete` is line numbers, `:1,5bdelete` buffer numbers and `:1,5close`
/// window numbers, and the address parser reads `.`, `$` and `'m` differently
/// for each. c2rust gave the family a bare `c_uint`, so every one of the ~70
/// `match` sites over it needed a catch-all arm for values that cannot exist.
///
/// `#[repr(u32)]` with the upstream discriminants: `exarg_T` and `ucmd_T` are
/// `repr(C)`, and the discriminants are what `ex_cmds.lua` and the
/// `nvim_parse_cmd` API answer with.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum CmdAddr {
    /// Buffer lines -- the default, and what `:1,5d` means.
    Lines = 0,
    /// Window numbers, as `:1,5close` counts them.
    Windows = 1,
    /// Argument-list indices.
    Arguments = 2,
    /// Buffer numbers, loaded buffers only.
    LoadedBuffers = 3,
    /// Buffer numbers, listed or not.
    Buffers = 4,
    /// Tab-page numbers.
    Tabs = 5,
    /// Tab pages counted from the current one (`:tabmove`).
    TabsRelative = 6,
    /// Quickfix entries, valid ones only.
    QuickfixValid = 7,
    /// Quickfix entry numbers.
    Quickfix = 8,
    /// A plain non-negative number the command reads itself.
    Unsigned = 9,
    /// Something only the command knows how to count.
    Other = 10,
    /// The command takes no range at all.
    NoRange = 11,
}

crate::flag_set! {
    /// What syntax an Ex command accepts -- upstream's `EX_*`, the bits
    /// `exarg_T::argt` and the command table's `cmd_argt` carry. The whole
    /// of `:` is described by this one word: which of a range, a `!`, an
    /// argument, a register and a count the command takes, and where it is
    /// allowed to run.
    pub struct ExArgt;

    /// The command takes a range.
    const RANGE = 0x001;
    /// The command takes a `!` after its name.
    const BANG = 0x002;
    /// The command takes an argument.
    const EXTRA = 0x004;
    /// Expand `%`, `#` and the other wildcards in the argument.
    const XFILE = 0x008;
    /// The argument is one word: no spaces allowed.
    const NOSPC = 0x010;
    /// A missing range means the whole file, not the current line.
    const DFLALL = 0x020;
    /// Extend the range to whole closed folds.
    const WHOLEFOLD = 0x040;
    /// An argument is required.
    const NEEDARG = 0x080;
    /// A `|` ends the command, and so does a `"` comment.
    const TRLBAR = 0x100;
    /// A register name may follow the command.
    const REGSTR = 0x200;
    /// A count may follow the command.
    const COUNT = 0x400;
    /// A `"` does *not* start a comment: the command's own argument may
    /// contain one.
    const NOTRLCOM = 0x800;
    /// Line number zero is allowed in the range.
    const ZEROR = 0x1000;
    /// `CTRL-V` quotes the next character in the argument.
    const CTRLV = 0x2000;
    /// `++opt=arg` file options are copied into `eap->cmd`.
    const CMDARG = 0x4000;
    /// The argument names a buffer, for `:buffer`-style completion.
    const BUFNAME = 0x8000;
    /// The named buffer may be an unlisted one.
    const BUFUNL = 0x10000;
    /// `++opt=arg` file options are allowed.
    const ARGOPT = 0x20000;
    /// The command is allowed inside the `'sandbox'`.
    const SBOXOK = 0x40000;
    /// The command is allowed in the command-line window.
    const CMDWIN = 0x80000;
    /// The command changes the buffer, so `'modifiable'` and the text lock
    /// are checked first.
    const MODIFY = 0x100000;
    /// Trailing `l`, `#` or `p` flags are allowed.
    const FLAGS = 0x200000;
    /// The command is allowed when the buffer is locked against changes.
    const LOCK_OK = 0x1000000;
    /// A user command that keeps the script context of its definition.
    const KEEPSCRIPT = 0x4000000;
    /// The command has an `'inccommand'` preview implementation.
    const PREVIEW = 0x8000000;
}
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
/// The `:silent`/`:noautocmd`/`:tab`/… run in front of one Ex command.
///
/// Not `Copy`: `cmod_filter_pat` and `cmod_filter_regmatch.regprog` are
/// allocations the modifier set owns, and the trailing `cmod_*_save`
/// fields are what `apply_cmdmod` put aside so `undo_cmdmod` can put it
/// back — a duplicate of those would undo the same suppression twice.
#[derive(Clone)]
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

impl cmdmod_T {
    /// No modifiers at all: the all-zero set a command starts from, and
    /// what the C reaches with `CLEAR_FIELD(cmdmod)`. A `const` because
    /// two other all-zero initialisers embed it.
    pub const NONE: cmdmod_T = cmdmod_T {
        cmod_flags: CmdModFlags::NONE,
        cmod_split: 0,
        cmod_tab: 0,
        cmod_filter_pat: ::core::ptr::null_mut(),
        cmod_filter_regmatch: regmatch_T {
            regprog: ::core::ptr::null_mut(),
            startp: [::core::ptr::null_mut(); 10],
            endp: [::core::ptr::null_mut(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        },
        cmod_filter_force: false,
        cmod_verbose: 0,
        cmod_save_ei: ::core::ptr::null_mut(),
        cmod_did_sandbox: 0,
        cmod_verbose_save: 0,
        cmod_save_msg_silent: 0,
        cmod_save_msg_scroll: 0,
        cmod_did_esilent: 0,
    };
}

impl Default for cmdmod_T {
    fn default() -> Self {
        Self::NONE
    }
}
/// One parsed Ex command line.
///
/// Not `Copy`: `args`/`arglens` and `cmdline_tofree` are allocations the
/// command owns for as long as it runs.
#[derive(Clone)]
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
    pub argt: ExArgt,
    pub skip: ::core::ffi::c_int,
    pub forceit: ::core::ffi::c_int,
    pub addr_count: ::core::ffi::c_int,
    pub line1: linenr_T,
    pub line2: linenr_T,
    pub addr_type: CmdAddr,
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
            argt: ExArgt::NONE,
            skip: 0,
            forceit: 0,
            addr_count: 0,
            line1: 0,
            line2: 0,
            addr_type: CmdAddr::Lines,
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
