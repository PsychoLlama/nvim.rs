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

pub type MessagePackType = ::core::ffi::c_uint;
/// A `v:` variable, by its slot in the `vimvars` table.
///
/// `Vv::Count` is `v:count`, and the variant names are the `v:` names rather
/// than upstream's `VV_*` spellings, which drifted from them
/// (`Vv::Servername` is `v:servername`, `Vv::Operator` is `v:operator`, `Vv::Register` is
/// `v:register`). Eleven slots -- the `beval_*` six, `v:windowid` and the
/// four `v:_null_*` -- had no `VV_*` constant in the port at all and are
/// named here for the first time.
///
/// **The order is the table's**: `eval::vars`'s `vimvars` is indexed by this,
/// so a variant's discriminant is its row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Vv {
    /// `v:count`
    Count = 0,
    /// `v:count1`
    Count1 = 1,
    /// `v:prevcount`
    Prevcount = 2,
    /// `v:errmsg`
    Errmsg = 3,
    /// `v:warningmsg`
    Warningmsg = 4,
    /// `v:statusmsg`
    Statusmsg = 5,
    /// `v:shell_error`
    ShellError = 6,
    /// `v:this_session`
    ThisSession = 7,
    /// `v:version`
    Version = 8,
    /// `v:lnum`
    Lnum = 9,
    /// `v:termrequest`
    Termrequest = 10,
    /// `v:termresponse`
    Termresponse = 11,
    /// `v:fname`
    Fname = 12,
    /// `v:lang`
    Lang = 13,
    /// `v:lc_time`
    LcTime = 14,
    /// `v:ctype`
    Ctype = 15,
    /// `v:charconvert_from`
    CharconvertFrom = 16,
    /// `v:charconvert_to`
    CharconvertTo = 17,
    /// `v:fname_in`
    FnameIn = 18,
    /// `v:fname_out`
    FnameOut = 19,
    /// `v:fname_new`
    FnameNew = 20,
    /// `v:fname_diff`
    FnameDiff = 21,
    /// `v:cmdarg`
    Cmdarg = 22,
    /// `v:foldstart`
    Foldstart = 23,
    /// `v:foldend`
    Foldend = 24,
    /// `v:folddashes`
    Folddashes = 25,
    /// `v:foldlevel`
    Foldlevel = 26,
    /// `v:progname`
    Progname = 27,
    /// `v:servername`
    Servername = 28,
    /// `v:dying`
    Dying = 29,
    /// `v:exception`
    Exception = 30,
    /// `v:throwpoint`
    Throwpoint = 31,
    /// `v:register`
    Register = 32,
    /// `v:cmdbang`
    Cmdbang = 33,
    /// `v:insertmode`
    Insertmode = 34,
    /// `v:val`
    Val = 35,
    /// `v:key`
    Key = 36,
    /// `v:profiling`
    Profiling = 37,
    /// `v:fcs_reason`
    FcsReason = 38,
    /// `v:fcs_choice`
    FcsChoice = 39,
    /// `v:beval_bufnr`
    BevalBufnr = 40,
    /// `v:beval_winnr`
    BevalWinnr = 41,
    /// `v:beval_winid`
    BevalWinid = 42,
    /// `v:beval_lnum`
    BevalLnum = 43,
    /// `v:beval_col`
    BevalCol = 44,
    /// `v:beval_text`
    BevalText = 45,
    /// `v:scrollstart`
    Scrollstart = 46,
    /// `v:swapname`
    Swapname = 47,
    /// `v:swapchoice`
    Swapchoice = 48,
    /// `v:swapcommand`
    Swapcommand = 49,
    /// `v:char`
    Char = 50,
    /// `v:mouse_win`
    MouseWin = 51,
    /// `v:mouse_winid`
    MouseWinid = 52,
    /// `v:mouse_lnum`
    MouseLnum = 53,
    /// `v:mouse_col`
    MouseCol = 54,
    /// `v:operator`
    Operator = 55,
    /// `v:searchforward`
    Searchforward = 56,
    /// `v:hlsearch`
    Hlsearch = 57,
    /// `v:oldfiles`
    Oldfiles = 58,
    /// `v:windowid`
    Windowid = 59,
    /// `v:progpath`
    Progpath = 60,
    /// `v:completed_item`
    CompletedItem = 61,
    /// `v:option_new`
    OptionNew = 62,
    /// `v:option_old`
    OptionOld = 63,
    /// `v:option_oldlocal`
    OptionOldlocal = 64,
    /// `v:option_oldglobal`
    OptionOldglobal = 65,
    /// `v:option_command`
    OptionCommand = 66,
    /// `v:option_type`
    OptionType = 67,
    /// `v:errors`
    Errors = 68,
    /// `v:false`
    False = 69,
    /// `v:true`
    True = 70,
    /// `v:null`
    Null = 71,
    /// `v:numbermax`
    Numbermax = 72,
    /// `v:numbermin`
    Numbermin = 73,
    /// `v:numbersize`
    Numbersize = 74,
    /// `v:vim_did_enter`
    VimDidEnter = 75,
    /// `v:testing`
    Testing = 76,
    /// `v:t_number`
    TNumber = 77,
    /// `v:t_string`
    TString = 78,
    /// `v:t_func`
    TFunc = 79,
    /// `v:t_list`
    TList = 80,
    /// `v:t_dict`
    TDict = 81,
    /// `v:t_float`
    TFloat = 82,
    /// `v:t_bool`
    TBool = 83,
    /// `v:t_blob`
    TBlob = 84,
    /// `v:event`
    Event = 85,
    /// `v:versionlong`
    Versionlong = 86,
    /// `v:echospace`
    Echospace = 87,
    /// `v:argf`
    Argf = 88,
    /// `v:argv`
    Argv = 89,
    /// `v:collate`
    Collate = 90,
    /// `v:exiting`
    Exiting = 91,
    /// `v:maxcol`
    Maxcol = 92,
    /// `v:stacktrace`
    Stacktrace = 93,
    /// `v:vim_did_init`
    VimDidInit = 94,
    /// `v:stderr`
    Stderr = 95,
    /// `v:msgpack_types`
    MsgpackTypes = 96,
    /// `v:_null_string`
    NullString = 97,
    /// `v:_null_list`
    NullList = 98,
    /// `v:_null_dict`
    NullDict = 99,
    /// `v:_null_blob`
    NullBlob = 100,
    /// `v:lua`
    Lua = 101,
    /// `v:relnum`
    Relnum = 102,
    /// `v:virtnum`
    Virtnum = 103,
    /// `v:starttime`
    Starttime = 104,
    /// `v:exitreason`
    Exitreason = 105,
}

/// A number that is not a `v:` variable's slot.
#[derive(Clone, Copy, Debug)]
pub struct NotAVimVar;

impl TryFrom<usize> for Vv {
    type Error = NotAVimVar;

    fn try_from(value: usize) -> Result<Self, NotAVimVar> {
        // SAFETY-free: the table is dense from 0 to `COUNT`, so the cast is
        // total over that range and rejected outside it.
        Vv::ALL.get(value).copied().ok_or(NotAVimVar)
    }
}

impl Vv {
    /// How many `v:` variables there are; the length of the `vimvars` table.
    pub const COUNT: usize = 106;

    /// Every `v:` variable, in table order.
    pub const ALL: [Vv; Self::COUNT] = ALL_VIM_VARS;

    /// This variable's row in the `vimvars` table.
    pub const fn index(self) -> usize {
        self as usize
    }
}

const ALL_VIM_VARS: [Vv; Vv::COUNT] = [
    Vv::Count,
    Vv::Count1,
    Vv::Prevcount,
    Vv::Errmsg,
    Vv::Warningmsg,
    Vv::Statusmsg,
    Vv::ShellError,
    Vv::ThisSession,
    Vv::Version,
    Vv::Lnum,
    Vv::Termrequest,
    Vv::Termresponse,
    Vv::Fname,
    Vv::Lang,
    Vv::LcTime,
    Vv::Ctype,
    Vv::CharconvertFrom,
    Vv::CharconvertTo,
    Vv::FnameIn,
    Vv::FnameOut,
    Vv::FnameNew,
    Vv::FnameDiff,
    Vv::Cmdarg,
    Vv::Foldstart,
    Vv::Foldend,
    Vv::Folddashes,
    Vv::Foldlevel,
    Vv::Progname,
    Vv::Servername,
    Vv::Dying,
    Vv::Exception,
    Vv::Throwpoint,
    Vv::Register,
    Vv::Cmdbang,
    Vv::Insertmode,
    Vv::Val,
    Vv::Key,
    Vv::Profiling,
    Vv::FcsReason,
    Vv::FcsChoice,
    Vv::BevalBufnr,
    Vv::BevalWinnr,
    Vv::BevalWinid,
    Vv::BevalLnum,
    Vv::BevalCol,
    Vv::BevalText,
    Vv::Scrollstart,
    Vv::Swapname,
    Vv::Swapchoice,
    Vv::Swapcommand,
    Vv::Char,
    Vv::MouseWin,
    Vv::MouseWinid,
    Vv::MouseLnum,
    Vv::MouseCol,
    Vv::Operator,
    Vv::Searchforward,
    Vv::Hlsearch,
    Vv::Oldfiles,
    Vv::Windowid,
    Vv::Progpath,
    Vv::CompletedItem,
    Vv::OptionNew,
    Vv::OptionOld,
    Vv::OptionOldlocal,
    Vv::OptionOldglobal,
    Vv::OptionCommand,
    Vv::OptionType,
    Vv::Errors,
    Vv::False,
    Vv::True,
    Vv::Null,
    Vv::Numbermax,
    Vv::Numbermin,
    Vv::Numbersize,
    Vv::VimDidEnter,
    Vv::Testing,
    Vv::TNumber,
    Vv::TString,
    Vv::TFunc,
    Vv::TList,
    Vv::TDict,
    Vv::TFloat,
    Vv::TBool,
    Vv::TBlob,
    Vv::Event,
    Vv::Versionlong,
    Vv::Echospace,
    Vv::Argf,
    Vv::Argv,
    Vv::Collate,
    Vv::Exiting,
    Vv::Maxcol,
    Vv::Stacktrace,
    Vv::VimDidInit,
    Vv::Stderr,
    Vv::MsgpackTypes,
    Vv::NullString,
    Vv::NullList,
    Vv::NullDict,
    Vv::NullBlob,
    Vv::Lua,
    Vv::Relnum,
    Vv::Virtnum,
    Vv::Starttime,
    Vv::Exitreason,
];

crate::flag_set! {
    /// What kind of `v:` variable a `vimvars` row is -- upstream's `VV_*`
    /// *flag* bits, which share the prefix with the slot names above and are
    /// a different family entirely.
    pub struct VimVarFlags;

    /// Also readable without the `v:` prefix, from `compat_hashtab`.
    const COMPAT = 1;
    /// Read-only.
    const RO = 2;
    /// Read-only inside a `:sandbox`.
    const RO_SBX = 4;
}
/// How one `eval*()` call reads its continuation lines.
///
/// Not `Copy`: `eval_tofree` is the joined line the evaluator allocated and
/// must free once.
#[derive(Clone)]
pub struct evalarg_T {
    pub eval_flags: ::core::ffi::c_int,
    pub eval_getline: LineGetter,
    pub eval_cookie: *mut ::core::ffi::c_void,
    pub eval_tofree: *mut ::core::ffi::c_char,
}
pub type exprtype_T = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
pub struct lval_T {
    pub ll_name: *const ::core::ffi::c_char,
    pub ll_name_len: size_t,
    pub ll_exp_name: *mut ::core::ffi::c_char,
    pub ll_tv: *mut typval_T,
    pub ll_li: *mut listitem_T,
    pub ll_list: *mut list_T,
    pub ll_range: bool,
    pub ll_empty2: bool,
    pub ll_n1: ::core::ffi::c_int,
    pub ll_n2: ::core::ffi::c_int,
    pub ll_dict: *mut dict_T,
    pub ll_di: *mut dictitem_T,
    pub ll_newkey: *mut ::core::ffi::c_char,
    pub ll_blob: *mut blob_T,
}
#[derive(Clone, Default)]
pub struct save_v_event_T {
    pub sve_did_save: bool,
    pub sve_hashtab: hashtab_T,
}
#[derive(Clone)]
pub struct timer_T {
    pub tw: TimeWatcher,
    pub timer_id: ::core::ffi::c_int,
    pub repeat_count: ::core::ffi::c_int,
    pub refcount: ::core::ffi::c_int,
    pub emsg_count: ::core::ffi::c_int,
    pub timeout: int64_t,
    pub stopped: bool,
    pub paused: bool,
    pub callback: Callback,
}
pub type var_flavour_T = ::core::ffi::c_uint;
/// Which persistence a global variable qualifies for, from the case of its
/// name: all-lowercase is session-only, `Mixed` reaches ShaDa, ALLCAPS
/// neither.
pub const VAR_FLAVOUR_DEFAULT: var_flavour_T = 1;
pub const VAR_FLAVOUR_SESSION: var_flavour_T = 2;
pub const VAR_FLAVOUR_SHADA: var_flavour_T = 4;
