//! Everything an option *does*: `:set` and its relatives, the validation a
//! new value has to pass, the `did_set_*` callbacks that react to one, and
//! the per-scope plumbing that decides which copy of a value a window or
//! buffer is looking at.
//!
//! Everything an option *is* — its name, type, scopes, flags, variable and
//! default — lives in the generated [`crate::src::nvim::options`] table.
//! The string options' own callbacks live in
//! [`crate::src::nvim::optionstr`].
//!
//! The module is one file per kind of work; the parent holds no code, only
//! the vocabulary more than one child needs and the constants the
//! transpiled headers left behind.
//!
//! | child | what |
//! | --- | --- |
//! | [`value`] | the `OptVal` union: free, copy, compare, convert |
//! | [`scope`] | which variable a scope reaches |
//! | [`query`] | the accessors the rest of the editor reads through |
//! | [`validate`] | vetting a value before anything sees it |
//! | [`set`] | `set_option`, and the ordering it depends on |
//! | [`context`] | doing that for another window or buffer |
//! | [`set_cmd`] | the `:set` argument parser |
//! | [`stropt`] | the `+=`/`^=`/`-=` value assembly |
//! | [`didset`] | the boolean and numeric `did_set_*` callbacks |
//! | [`paste`] | 'paste', and everything it switches off |
//! | [`check`] | the sweeps that re-vet an option nothing set |
//! | [`defaults`] | where a default comes from, and the startup passes |
//! | [`copy`] | handing a new window or buffer its own values |
//! | [`show`] | `:set` listing, `:mkvimrc`, the UI broadcast |
//! | [`expand`] | command-line completion |
//! | [`info`] | `nvim_get_option_info` |

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::empty_string_option;
use crate::src::nvim::optionstr::set_chars_option;
use crate::src::nvim::types::{
    CMD_index, CallbackType, CharsOption, ErrorType, HlAttrs, OptScope, OptValType, RgbValue,
    String_0, TriState, VarType, VimVarIndex, auto_event, int16_t, int32_t, size_t, xp_prefix_T,
};
use core::ffi::{c_char, c_int, c_uint};

mod check;
mod context;
mod copy;
mod defaults;
mod didset;
mod expand;
mod info;
mod paste;
mod query;
mod scope;
mod set;
mod set_cmd;
mod show;
mod stropt;
mod validate;
mod value;

pub use self::check::*;
pub use self::context::*;
pub use self::copy::*;
pub use self::defaults::*;
pub use self::didset::*;
pub use self::expand::*;
pub use self::info::*;
pub use self::paste::*;
pub use self::query::*;
pub use self::scope::*;
pub use self::set::*;
pub use self::set_cmd::*;
pub use self::show::*;
pub(crate) use self::stropt::*;
pub(crate) use self::validate::*;
pub use self::value::*;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub const kCallbackNone: CallbackType = 0;
pub const VAR_STRING: VarType = 2;
pub const MAXCOL: c_uint = 2147483647;
pub const HL_GLOBAL: c_uint = 16384;
pub const HLF_W: c_uint = 26;
pub const NUMBUFLEN: c_uint = 65;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_BS_COMMA: c_uint = 4;
pub const XP_BS_THREE: c_uint = 2;
pub const XP_BS_ONE: c_uint = 1;
pub const EXPAND_KEYMAP: c_int = 55;
pub const EXPAND_SETTING_SUBTRACT: c_int = 53;
pub const EXPAND_STRING_SETTING: c_int = 52;
pub const EXPAND_OWNSYNTAX: c_int = 38;
pub const EXPAND_FILETYPE: c_int = 36;
pub const EXPAND_OLD_SETTING: c_int = 7;
pub const EXPAND_BOOL_SETTINGS: c_int = 5;
pub const EXPAND_SETTINGS: c_int = 4;
pub const EXPAND_DIRECTORIES: c_int = 3;
pub const EXPAND_FILES: c_int = 2;
pub const EXPAND_NOTHING: c_int = 0;
pub const EXPAND_UNSUCCESSFUL: c_int = -2;
/// What the generated table's `flags` column can say about an option.
pub type OptFlags = c_uint;
pub const kOptFlagColon: OptFlags = 33554432;
pub const kOptFlagFunc: OptFlags = 16777216;
pub const kOptFlagMLE: OptFlags = 8388608;
pub const kOptFlagHLOnly: OptFlags = 4194304;
pub const kOptFlagNDname: OptFlags = 2097152;
pub const kOptFlagCurswant: OptFlags = 1048576;
pub const kOptFlagPriMkrc: OptFlags = 524288;
pub const kOptFlagInsecure: OptFlags = 262144;
pub const kOptFlagNFname: OptFlags = 131072;
pub const kOptFlagNoGlob: OptFlags = 65536;
pub const kOptFlagGettext: OptFlags = 32768;
pub const kOptFlagSecure: OptFlags = 16384;
pub const kOptFlagFlagList: OptFlags = 8192;
pub const kOptFlagNoDup: OptFlags = 4096;
pub const kOptFlagOneComma: OptFlags = 3072;
pub const kOptFlagComma: OptFlags = 1024;
pub const kOptFlagRedrAll: OptFlags = 768;
pub const kOptFlagRedrBuf: OptFlags = 512;
pub const kOptFlagRedrWin: OptFlags = 256;
pub const kOptFlagRedrStat: OptFlags = 128;
pub const kOptFlagRedrTabl: OptFlags = 64;
pub const kOptFlagUIOption: OptFlags = 32;
pub const kOptFlagNoMkrc: OptFlags = 16;
pub const kOptFlagWasSet: OptFlags = 8;
pub const kOptFlagNoDefault: OptFlags = 4;
pub const kOptFlagNoDefExp: OptFlags = 2;
pub const kOptFlagExpand: OptFlags = 1;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
pub const kOptScopeBuf: OptScope = 2;
pub const kOptScopeWin: OptScope = 1;
pub const kOptScopeGlobal: OptScope = 0;
/// Which of `=`, `+=`, `^=` and `-=` a `:set` argument used.
pub type set_op_T = c_uint;
pub const OP_REMOVING: set_op_T = 3;
pub const OP_PREPENDING: set_op_T = 2;
pub const OP_ADDING: set_op_T = 1;
pub const OP_NONE: set_op_T = 0;
pub const CMD_setlocal: CMD_index = 402;
pub const CMD_setglobal: CMD_index = 401;
pub const CMOD_NOSWAPFILE: c_uint = 8192;
pub const EVENT_SYNTAX: auto_event = 110;
pub const EVENT_OPTIONSET: auto_event = 85;
pub const EVENT_BUFDELETE: auto_event = 2;
pub const EVENT_BUFADD: auto_event = 0;
pub const SHM_WRI: c_uint = 119;
pub const SHM_LINES: c_uint = 108;
pub const SHM_MOD: c_uint = 109;
pub const SHM_RO: c_uint = 114;
pub const STR2NR_ALL: c_uint = 15;
pub const UPD_CLEAR: c_uint = 50;
pub const UPD_NOT_VALID: c_uint = 40;
pub const UPD_SOME_VALID: c_uint = 35;
pub const VV_OPTION_TYPE: VimVarIndex = 67;
pub const VV_OPTION_COMMAND: VimVarIndex = 66;
pub const VV_OPTION_OLDGLOBAL: VimVarIndex = 65;
pub const VV_OPTION_OLDLOCAL: VimVarIndex = 64;
pub const VV_OPTION_OLD: VimVarIndex = 63;
pub const VV_OPTION_NEW: VimVarIndex = 62;
pub const VV_WARNINGMSG: VimVarIndex = 4;
pub const FUZZY_SCORE_NONE: c_int = -2147483648;
pub const MODE_TERMINAL: c_uint = 128;
pub const FSK_SIMPLIFY: c_uint = 8;
pub const FSK_KEEP_X_KEY: c_uint = 2;
pub const FSK_KEYCODE: c_uint = 1;
pub const BCO_NOHELP: c_uint = 4;
pub const BCO_ALWAYS: c_uint = 2;
pub const BCO_ENTER: c_uint = 1;
/// The scope and behaviour bits every `opt_flags` argument carries.
///
/// `OPT_LOCAL` and `OPT_GLOBAL` name a scope; neither means "both", which
/// is what a bare `:set` does. `OPT_MODELINE` says the value came from a
/// modeline and so must be treated as insecure; `OPT_WINONLY`/`OPT_NOWIN`
/// restrict a sweep to one kind of option; `OPT_ONECOLUMN` is `:set!`'s
/// one-per-line listing; `OPT_SKIPRTP` is `:mksession` leaving the runtime
/// paths alone.
pub const OPT_SKIPRTP: c_int = 128;
pub const OPT_ONECOLUMN: c_int = 32;
pub const OPT_NOWIN: c_int = 16;
pub const OPT_WINONLY: c_int = 8;
pub const OPT_MODELINE: c_int = 4;
pub const OPT_LOCAL: c_int = 2;
pub const OPT_GLOBAL: c_int = 1;
pub const STATUS_HEIGHT: c_uint = 1;
pub const DIP_ALL: c_uint = 1;
pub const MIN_COLUMNS: c_uint = 12;
pub const kListchars: CharsOption = 1;
pub const kFillchars: CharsOption = 0;
/// The longest path the option module will build or expand.
pub const MAXPATHL: c_int = 4096;
pub const ROOT_UID: c_int = 0 as c_int;
pub const BF_SYN_SET: c_int = 0x200 as c_int;
pub const B_IMODE_USE_INSERT: c_int = -1 as c_int;
pub const B_IMODE_NONE: c_int = 0 as c_int;
pub const KEYMAP_INIT: c_int = 1 as c_int;
pub const NULL_STRING: String_0 = String_0 {
    data: ::core::ptr::null_mut::<c_char>(),
    size: 0 as size_t,
};
pub const LOGLVL_INF: c_int = 2 as c_int;
pub const OK: c_int = 1 as c_int;
pub const FAIL: c_int = 0 as c_int;
pub const NUL: c_int = '\0' as c_int;
pub const TAB: c_int = '\t' as c_int;
pub const CTRL_F_STR: [c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [c_char; 2]>(*b"\x06\0") };
pub const Ctrl_C: c_int = 3 as c_int;
pub const PATHSEPSTR: [c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [c_char; 2]>(*b"/\0") };
pub const FORCE_BIN: c_int = 1 as c_int;
pub const HLATTRS_INIT: HlAttrs = HlAttrs {
    rgb_ae_attr: 0 as int32_t,
    cterm_ae_attr: 0 as int32_t,
    rgb_fg_color: -1 as RgbValue,
    rgb_bg_color: -1 as RgbValue,
    rgb_sp_color: -1 as RgbValue,
    cterm_fg_color: 0 as int16_t,
    cterm_bg_color: 0 as int16_t,
    hl_blend: -1 as int32_t,
    url: -1 as int32_t,
};
pub const HIGHLIGHT_INIT: [c_char; 779] = unsafe {
    ::core::mem::transmute::<
        [u8; 779],
        [c_char; 779],
    >(
        *b"8:SpecialKey,~:EndOfBuffer,z:TermCursor,@:NonText,d:Directory,e:ErrorMsg,i:IncSearch,l:Search,y:CurSearch,m:MoreMsg,M:ModeMsg,n:LineNr,a:LineNrAbove,b:LineNrBelow,N:CursorLineNr,G:CursorLineSign,O:CursorLineFold,r:Question,s:StatusLine,S:StatusLineNC,c:VertSplit,t:Title,v:Visual,V:VisualNOS,w:WarningMsg,W:WildMenu,f:Folded,F:FoldColumn,A:DiffAdd,C:DiffChange,D:DiffDelete,T:DiffText,E:DiffTextAdd,>:SignColumn,-:Conceal,B:SpellBad,P:SpellCap,R:SpellRare,L:SpellLocal,+:Pmenu,=:PmenuSel,k:PmenuMatch,<:PmenuMatchSel,[:PmenuKind,]:PmenuKindSel,{:PmenuExtra,}:PmenuExtraSel,x:PmenuSbar,X:PmenuThumb,*:TabLine,#:TabLineSel,_:TabLineFill,!:CursorColumn,.:CursorLine,o:ColorColumn,q:QuickFixLine,z:StatusLineTerm,Z:StatusLineTermNC,g:MsgArea,h:ComplMatchIns,0:Whitespace,I:PreInsert\0",
    )
};
pub const DFLT_EFM: [c_char; 667] = unsafe {
    ::core::mem::transmute::<
        [u8; 667],
        [c_char; 667],
    >(
        *b"%*[^\"]\"%f\"%*\\D%l: %m,\"%f\"%*\\D%l: %m,%-Gg%\\?make[%*\\d]: *** [%f:%l:%m,%-Gg%\\?make: *** [%f:%l:%m,%-G%f:%l: (Each undeclared identifier is reported only once,%-G%f:%l: for each function it appears in.),%-GIn file included from %f:%l:%c:,%-GIn file included from %f:%l:%c\\,,%-GIn file included from %f:%l:%c,%-GIn file included from %f:%l,%-G%*[ ]from %f:%l:%c,%-G%*[ ]from %f:%l:,%-G%*[ ]from %f:%l\\,,%-G%*[ ]from %f:%l,%f:%l:%c:%m,%f(%l):%m,%f:%l:%m,\"%f\"\\, line %l%*\\D%c%*[^ ] %m,%D%*\\a[%*\\d]: Entering directory %*[`']%f',%X%*\\a[%*\\d]: Leaving directory %*[`']%f',%D%*\\a: Entering directory %*[`']%f',%X%*\\a: Leaving directory %*[`']%f',%DMaking %*\\a in %f,%f|%l| %m\0",
    )
};
pub const DFLT_GFN: [c_char; 55] = unsafe {
    ::core::mem::transmute::<[u8; 55], [c_char; 55]>(
        *b"Source Code Pro,DejaVu Sans Mono,Courier New,monospace\0",
    )
};
pub const DFLT_GREPFORMAT: [c_char; 26] =
    unsafe { ::core::mem::transmute::<[u8; 26], [c_char; 26]>(*b"%f:%l:%m,%f:%l%m,%f  %l%m\0") };
pub const ENC_DFLT: [c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [c_char; 6]>(*b"utf-8\0") };
pub const EOL_UNIX: c_int = 0 as c_int;
pub const EOL_DOS: c_int = 1 as c_int;
pub const EOL_MAC: c_int = 2 as c_int;
pub const DFLT_FO_VIM: [c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [c_char; 5]>(*b"tcqj\0") };
pub const MAX_MCO: c_int = 6 as c_int;
pub const CPO_VIM: [c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [c_char; 9]>(*b"aABceFs_\0") };
pub const BS_START: c_int = 's' as c_int;
pub const BS_NOSTOP: c_int = 'p' as c_int;
pub const LISPWORD_VALUE: [c_char; 746] = unsafe {
    ::core::mem::transmute::<
        [u8; 746],
        [c_char; 746],
    >(
        *b"defun,define,defmacro,set!,lambda,if,case,let,flet,let*,letrec,do,do*,define-syntax,let-syntax,letrec-syntax,destructuring-bind,defpackage,defparameter,defstruct,deftype,defvar,do-all-symbols,do-external-symbols,do-symbols,dolist,dotimes,ecase,etypecase,eval-when,labels,macrolet,multiple-value-bind,multiple-value-call,multiple-value-prog1,multiple-value-setq,prog1,progv,typecase,unless,unwind-protect,when,with-input-from-string,with-open-file,with-open-stream,with-output-to-string,with-package-iterator,define-condition,handler-bind,handler-case,restart-bind,restart-case,with-simple-restart,store-value,use-value,muffle-warning,abort,continue,with-slots,with-slots*,with-accessors,with-accessors*,defclass,defmethod,print-unreadable-object\0",
    )
};
pub static p_vfile: GlobalCell<*mut c_char> =
    GlobalCell::new((empty_string_option.as_raw() as *const _) as *mut c_char);
pub const NO_LOCAL_UNDOLEVEL: c_int = -123456 as c_int;
pub const SB_MAX: c_int = 1000000 as c_int;
pub const MAX_NUMBERWIDTH: c_int = 20 as c_int;
pub const TABSTOP_MAX: c_int = 9999 as c_int;
pub const SHAPE_CURSOR: c_int = 2 as c_int;
pub const IOSIZE: c_int = 1024 as c_int + 1 as c_int;
pub const DFLT_ERRORFILE: [c_char; 11] =
    unsafe { ::core::mem::transmute::<[u8; 11], [c_char; 11]>(*b"errors.err\0") };
pub const DFLT_HELPFILE: [c_char; 25] =
    unsafe { ::core::mem::transmute::<[u8; 25], [c_char; 25]>(*b"$VIMRUNTIME/doc/help.txt\0") };
pub const NO_SCREEN: c_int = 2 as c_int;
pub const DFLT_COLS: c_int = 80 as c_int;
pub const DFLT_ROWS: c_int = 24 as c_int;
pub const SID_NONE: c_int = -6 as c_int;
pub const K_ZERO: c_int = -(255 as c_int + (('X' as c_int) << 8 as c_int));
pub const K_KENTER: c_int = -('K' as c_int + (('A' as c_int) << 8 as c_int));
pub const INT_MIN: c_int = -INT_MAX - 1 as c_int;
pub const INT_MAX: c_int = __INT_MAX__;
pub const PROJECT_NAME: [c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [c_char; 5]>(*b"nvim\0") };
pub const __INT_MAX__: c_int = 2147483647 as c_int;
