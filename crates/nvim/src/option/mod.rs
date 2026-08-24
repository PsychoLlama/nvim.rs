//! Everything an option *does*: `:set` and its relatives, the validation a
//! new value has to pass, the `did_set_*` callbacks that react to one, and
//! the per-scope plumbing that decides which copy of a value a window or
//! buffer is looking at.
//!
//! Everything an option *is* — its name, type, scopes, flags, variable and
//! default — lives in the generated [`crate::options`] table.
//! The string options' own callbacks live in
//! [`crate::optionstr`].
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

use crate::global_cell::GlobalCell;
use crate::highlight::HlAttrFlags;
use crate::optionstr::set_chars_option;
use crate::types::{
    CharsOption, HlAttrs, OptScope, OptValType, RgbValue, int16_t, int32_t, xp_prefix_T,
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
mod state;
mod stropt;
mod validate;
mod value;

pub(crate) use self::check::*;
pub(crate) use self::context::*;
pub(crate) use self::copy::*;
pub(crate) use self::defaults::*;
pub(crate) use self::didset::*;
pub(crate) use self::expand::*;
pub(crate) use self::info::*;
pub(crate) use self::paste::*;
pub(crate) use self::query::*;
pub(crate) use self::scope::*;
pub(crate) use self::set::*;
pub(crate) use self::set_cmd::*;
pub(crate) use self::show::*;
pub(crate) use self::state::*;
pub(crate) use self::stropt::*;
pub(crate) use self::validate::*;
pub(crate) use self::value::*;
pub(crate) const NUMBUFLEN: c_uint = 65;
pub(crate) const XP_PREFIX_INV: xp_prefix_T = 2;
pub(crate) const XP_PREFIX_NO: xp_prefix_T = 1;
/// What the generated table's `flags` column can say about an option.
pub(crate) type OptFlags = c_uint;
pub(crate) const kOptFlagColon: OptFlags = 33554432;
pub(crate) const kOptFlagFunc: OptFlags = 16777216;
pub(crate) const kOptFlagMLE: OptFlags = 8388608;
pub(crate) const kOptFlagHLOnly: OptFlags = 4194304;
pub(crate) const kOptFlagNDname: OptFlags = 2097152;
pub(crate) const kOptFlagCurswant: OptFlags = 1048576;
pub(crate) const kOptFlagPriMkrc: OptFlags = 524288;
pub(crate) const kOptFlagInsecure: OptFlags = 262144;
pub(crate) const kOptFlagNFname: OptFlags = 131072;
pub(crate) const kOptFlagNoGlob: OptFlags = 65536;
pub(crate) const kOptFlagGettext: OptFlags = 32768;
pub(crate) const kOptFlagSecure: OptFlags = 16384;
pub(crate) const kOptFlagFlagList: OptFlags = 8192;
pub(crate) const kOptFlagNoDup: OptFlags = 4096;
pub(crate) const kOptFlagOneComma: OptFlags = 3072;
pub(crate) const kOptFlagComma: OptFlags = 1024;
pub(crate) const kOptFlagRedrAll: OptFlags = 768;
pub(crate) const kOptFlagRedrBuf: OptFlags = 512;
pub(crate) const kOptFlagRedrWin: OptFlags = 256;
pub(crate) const kOptFlagRedrStat: OptFlags = 128;
pub(crate) const kOptFlagRedrTabl: OptFlags = 64;
pub(crate) const kOptFlagUIOption: OptFlags = 32;
pub(crate) const kOptFlagNoMkrc: OptFlags = 16;
pub(crate) const kOptFlagWasSet: OptFlags = 8;
pub(crate) const kOptFlagNoDefault: OptFlags = 4;
pub(crate) const kOptFlagNoDefExp: OptFlags = 2;
pub(crate) const kOptFlagExpand: OptFlags = 1;
pub(crate) const kOptValTypeString: OptValType = 2;
pub(crate) const kOptValTypeNumber: OptValType = 1;
pub(crate) const kOptValTypeBoolean: OptValType = 0;
pub(crate) const kOptValTypeNil: OptValType = -1;
pub(crate) const kOptScopeBuf: OptScope = 2;
pub(crate) const kOptScopeWin: OptScope = 1;
pub(crate) const kOptScopeGlobal: OptScope = 0;
/// Which of `=`, `+=`, `^=` and `-=` a `:set` argument used.
pub(crate) type set_op_T = c_uint;
pub(crate) const OP_REMOVING: set_op_T = 3;
pub(crate) const OP_PREPENDING: set_op_T = 2;
pub(crate) const OP_ADDING: set_op_T = 1;
pub(crate) const OP_NONE: set_op_T = 0;
pub(crate) const STR2NR_ALL: c_uint = 15;
pub(crate) const FUZZY_SCORE_NONE: c_int = -2147483648;
pub(crate) const FSK_SIMPLIFY: c_uint = 8;
pub(crate) const FSK_KEEP_X_KEY: c_uint = 2;
pub(crate) const FSK_KEYCODE: c_uint = 1;
pub(crate) const BCO_NOHELP: c_uint = 4;
pub(crate) const BCO_ALWAYS: c_uint = 2;
pub(crate) const BCO_ENTER: c_uint = 1;
pub(crate) const STATUS_HEIGHT: c_uint = 1;
pub(crate) const MIN_COLUMNS: c_uint = 12;
pub(crate) const kListchars: CharsOption = 1;
pub(crate) const kFillchars: CharsOption = 0;
pub(crate) const ROOT_UID: c_int = 0 as c_int;
pub(crate) const B_IMODE_USE_INSERT: c_int = -1 as c_int;
pub(crate) const B_IMODE_NONE: c_int = 0 as c_int;
pub(crate) const KEYMAP_INIT: c_int = 1 as c_int;
pub(crate) const TAB: c_int = '\t' as c_int;
pub(crate) const CTRL_F_STR: &::core::ffi::CStr = c"\x06";
pub(crate) const FORCE_BIN: c_int = 1 as c_int;
pub(crate) const HLATTRS_INIT: HlAttrs = HlAttrs {
    rgb_ae_attr: HlAttrFlags::NONE,
    cterm_ae_attr: HlAttrFlags::NONE,
    rgb_fg_color: -1 as RgbValue,
    rgb_bg_color: -1 as RgbValue,
    rgb_sp_color: -1 as RgbValue,
    cterm_fg_color: 0 as int16_t,
    cterm_bg_color: 0 as int16_t,
    hl_blend: -1 as int32_t,
    url: -1 as int32_t,
};
pub(crate) const HIGHLIGHT_INIT: &::core::ffi::CStr = c"8:SpecialKey,~:EndOfBuffer,z:TermCursor,@:NonText,d:Directory,e:ErrorMsg,i:IncSearch,l:Search,y:CurSearch,m:MoreMsg,M:ModeMsg,n:LineNr,a:LineNrAbove,b:LineNrBelow,N:CursorLineNr,G:CursorLineSign,O:CursorLineFold,r:Question,s:StatusLine,S:StatusLineNC,c:VertSplit,t:Title,v:Visual,V:VisualNOS,w:WarningMsg,W:WildMenu,f:Folded,F:FoldColumn,A:DiffAdd,C:DiffChange,D:DiffDelete,T:DiffText,E:DiffTextAdd,>:SignColumn,-:Conceal,B:SpellBad,P:SpellCap,R:SpellRare,L:SpellLocal,+:Pmenu,=:PmenuSel,k:PmenuMatch,<:PmenuMatchSel,[:PmenuKind,]:PmenuKindSel,{:PmenuExtra,}:PmenuExtraSel,x:PmenuSbar,X:PmenuThumb,*:TabLine,#:TabLineSel,_:TabLineFill,!:CursorColumn,.:CursorLine,o:ColorColumn,q:QuickFixLine,z:StatusLineTerm,Z:StatusLineTermNC,g:MsgArea,h:ComplMatchIns,0:Whitespace,I:PreInsert";
pub(crate) const DFLT_EFM: &::core::ffi::CStr = c"%*[^\"]\"%f\"%*\\D%l: %m,\"%f\"%*\\D%l: %m,%-Gg%\\?make[%*\\d]: *** [%f:%l:%m,%-Gg%\\?make: *** [%f:%l:%m,%-G%f:%l: (Each undeclared identifier is reported only once,%-G%f:%l: for each function it appears in.),%-GIn file included from %f:%l:%c:,%-GIn file included from %f:%l:%c\\,,%-GIn file included from %f:%l:%c,%-GIn file included from %f:%l,%-G%*[ ]from %f:%l:%c,%-G%*[ ]from %f:%l:,%-G%*[ ]from %f:%l\\,,%-G%*[ ]from %f:%l,%f:%l:%c:%m,%f(%l):%m,%f:%l:%m,\"%f\"\\, line %l%*\\D%c%*[^ ] %m,%D%*\\a[%*\\d]: Entering directory %*[`']%f',%X%*\\a[%*\\d]: Leaving directory %*[`']%f',%D%*\\a: Entering directory %*[`']%f',%X%*\\a: Leaving directory %*[`']%f',%DMaking %*\\a in %f,%f|%l| %m";
pub(crate) const DFLT_GFN: &::core::ffi::CStr =
    c"Source Code Pro,DejaVu Sans Mono,Courier New,monospace";
pub(crate) const DFLT_GREPFORMAT: &::core::ffi::CStr = c"%f:%l:%m,%f:%l%m,%f  %l%m";
pub(crate) const ENC_DFLT: &::core::ffi::CStr = c"utf-8";
pub(crate) const EOL_UNIX: c_int = 0 as c_int;
pub(crate) const EOL_DOS: c_int = 1 as c_int;
pub(crate) const EOL_MAC: c_int = 2 as c_int;
pub(crate) const DFLT_FO_VIM: &::core::ffi::CStr = c"tcqj";
pub(crate) const CPO_VIM: &::core::ffi::CStr = c"aABceFs_";
pub(crate) const LISPWORD_VALUE: &::core::ffi::CStr = c"defun,define,defmacro,set!,lambda,if,case,let,flet,let*,letrec,do,do*,define-syntax,let-syntax,letrec-syntax,destructuring-bind,defpackage,defparameter,defstruct,deftype,defvar,do-all-symbols,do-external-symbols,do-symbols,dolist,dotimes,ecase,etypecase,eval-when,labels,macrolet,multiple-value-bind,multiple-value-call,multiple-value-prog1,multiple-value-setq,prog1,progv,typecase,unless,unwind-protect,when,with-input-from-string,with-open-file,with-open-stream,with-output-to-string,with-package-iterator,define-condition,handler-bind,handler-case,restart-bind,restart-case,with-simple-restart,store-value,use-value,muffle-warning,abort,continue,with-slots,with-slots*,with-accessors,with-accessors*,defclass,defmethod,print-unreadable-object";
pub(crate) static p_vfile: GlobalCell<*mut c_char> =
    GlobalCell::new(crate::optionstr::empty_option());
pub(crate) const NO_LOCAL_UNDOLEVEL: c_int = -123456 as c_int;
pub(crate) const SB_MAX: c_int = 1000000 as c_int;
pub(crate) const MAX_NUMBERWIDTH: c_int = 20 as c_int;
pub(crate) const TABSTOP_MAX: c_int = 9999 as c_int;
pub(crate) const DFLT_ERRORFILE: &::core::ffi::CStr = c"errors.err";
pub(crate) const DFLT_HELPFILE: &::core::ffi::CStr = c"$VIMRUNTIME/doc/help.txt";
pub(crate) const NO_SCREEN: c_int = 2 as c_int;
pub(crate) const DFLT_COLS: c_int = 80 as c_int;
pub(crate) const DFLT_ROWS: c_int = 24 as c_int;
pub(crate) const SID_NONE: c_int = -6 as c_int;
pub(crate) const INT_MIN: c_int = -INT_MAX - 1 as c_int;
pub(crate) const INT_MAX: c_int = __INT_MAX__;
pub(crate) const PROJECT_NAME: &::core::ffi::CStr = c"nvim";
pub(crate) const __INT_MAX__: c_int = 2147483647 as c_int;
