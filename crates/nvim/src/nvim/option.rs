//! Everything an option *does*: `:set` and its relatives, the validation a
//! new value has to pass, the `did_set_*` callbacks that react to one, and
//! the per-scope plumbing that decides which copy of a value a window or
//! buffer is looking at.
//!
//! Everything an option *is* — its name, type, scopes, flags, variable and
//! default — lives in the generated [`crate::src::nvim::options`] table.

use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::charset::{transchar, vim_strsize};
use crate::src::nvim::cmdexpand::cmdline_fuzzy_complete;
use crate::src::nvim::ex_session::{put_eol, put_line};
use crate::src::nvim::fuzzy::{fuzzy_match_str, fuzzymatches_to_strmatches};
use crate::src::nvim::garray::{ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::keycodes::{
    find_special_key_in_table, get_special_key_code, get_special_key_name,
};
use crate::src::nvim::main::{
    Columns, NameBuff, curbuf, curwin, empty_string_option, escape_chars, got_int, info_message,
    p_bdir, p_cdpath, p_dir, p_ft, p_keymap, p_mouse, p_path, p_pp, p_rtp, p_sps, p_syn, p_tags,
    p_vdir, p_wc, p_wcm, silent_mode,
};
use crate::src::nvim::mapping::put_escstr;
use crate::src::nvim::memory::{xfree, xmalloc, xmemdupz, xstrdup, xstrlcpy};
use crate::src::nvim::message::{
    message_filtered, msg_advance, msg_ext_set_kind, msg_outtrans, msg_putchar, msg_puts,
    msg_puts_title,
};
use crate::src::nvim::mouse::setmouse;
use crate::src::nvim::options::*;
use crate::src::nvim::optionstr::set_chars_option;
use crate::src::nvim::os::env::{expand_env_esc, home_replace};
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, abort, fprintf, fputs, gettext, snprintf, strcmp, strlen, strncmp,
};
use crate::src::nvim::strings::{vim_strchr, vim_strsave_escaped};
use crate::src::nvim::types::{
    CMD_index, CallbackType, CharsOption, ErrorType, FILE, HlAttrs, Object, ObjectType, OptIndex,
    OptInt, OptScope, OptVal, OptValType, RgbValue, String_0, Terminal, TriState, VarType,
    VimVarIndex, auto_event, buf_T, colnr_T, expand_T, fuzmatch_str_T, garray_T, int16_t, int32_t,
    optexpand_T, regmatch_T, size_t, uint8_t, uint32_t, uint64_t, vimoption_T, win_T, xp_prefix_T,
};
use crate::src::nvim::ui::ui_call_option_set;
use crate::src::nvim::undo::curbufIsChanged;
use core::ffi::{c_char, c_int, c_uint, c_void};

// The carve of a 9,000-line transpiled module; see the child docs.
mod defaults;
pub use self::defaults::*;
mod stropt;
pub(crate) use self::stropt::*;
mod set_cmd;
pub use self::set_cmd::*;
mod didset;
pub use self::didset::*;
mod paste;
pub use self::paste::*;
mod validate;
pub(crate) use self::validate::*;
mod value;
pub use self::value::*;
mod scope;
pub use self::scope::*;
mod set;
pub use self::set::*;
mod context;
pub use self::context::*;
mod show;
pub use self::show::*;
mod expand;
pub use self::expand::*;
mod copy;
pub use self::copy::*;
mod info;
pub use self::info::*;
mod query;
pub use self::query::*;
mod check;
pub use self::check::*;
unsafe extern "C" {
    fn vim_regexec(rmp: *mut regmatch_T, line: *const c_char, col: colnr_T) -> bool;
    fn on_scrollback_option_changed(term: *mut Terminal);
    fn ll_resize_stack(wp: *mut win_T, n: c_int);
}
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
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
pub const OPT_SKIPRTP: c_uint = 128;
pub const OPT_ONECOLUMN: c_uint = 32;
pub const OPT_NOWIN: c_uint = 16;
pub const OPT_WINONLY: c_uint = 8;
pub const OPT_MODELINE: c_uint = 4;
pub const OPT_LOCAL: c_uint = 2;
pub const OPT_GLOBAL: c_uint = 1;
pub const STATUS_HEIGHT: c_uint = 1;
pub const DIP_ALL: c_uint = 1;
pub const MIN_COLUMNS: c_uint = 12;
pub const MAX_SEARCH_COUNT: c_uint = 9999;
pub const kListchars: CharsOption = 1;
pub const kFillchars: CharsOption = 0;
pub type set_prefix_T = c_uint;
pub const PREFIX_INV: set_prefix_T = 2;
pub const PREFIX_NONE: set_prefix_T = 1;
pub const PREFIX_NO: set_prefix_T = 0;
pub const NULL: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub const DEFAULT_MAXPATHL: c_int = 4096 as c_int;
pub const MAXPATHL: c_int = DEFAULT_MAXPATHL;
pub const ROOT_UID: c_int = 0 as c_int;
pub const BF_SYN_SET: c_int = 0x200 as c_int;
pub const B_IMODE_USE_INSERT: c_int = -1 as c_int;
pub const B_IMODE_NONE: c_int = 0 as c_int;
pub const B_IMODE_LAST: c_int = 1 as c_int;
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
pub const CPO_BUFOPT: c_int = 's' as c_int;
pub const CPO_BUFOPTGLOB: c_int = 'S' as c_int;
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
static e_unknown_option: GlobalCell<[c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [c_char; 21]>(*b"E518: Unknown option\0")
});
static e_not_allowed_in_modeline: GlobalCell<[c_char; 32]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 32], [c_char; 32]>(*b"E520: Not allowed in a modeline\0")
});
static e_not_allowed_in_modeline_when_modelineexpr_is_off: GlobalCell<[c_char; 59]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 59], [c_char; 59]>(
            *b"E992: Not allowed in a modeline when 'modelineexpr' is off\0",
        )
    });
static e_number_required_after_equal: GlobalCell<[c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [c_char; 30]>(*b"E521: Number required after =\0")
});
static e_preview_window_already_exists: GlobalCell<[c_char; 38]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 38], [c_char; 38]>(*b"E590: A preview window already exists\0")
});
static e_cannot_have_negative_or_zero_number_of_quickfix: GlobalCell<[c_char; 72]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 72], [c_char; 72]>(
            *b"E1542: Cannot have a negative or zero number of quickfix/location lists\0",
        )
    });
static e_cannot_have_more_than_hundred_quickfix: GlobalCell<[c_char; 63]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 63], [c_char; 63]>(
            *b"E1543: Cannot have more than a hundred quickfix/location lists\0",
        )
    });
static p_term: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
static p_ttytype: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
static p_et_nobin: GlobalCell<c_int> = GlobalCell::new(0);
static p_ml_nobin: GlobalCell<c_int> = GlobalCell::new(0);
static p_tw_nobin: GlobalCell<OptInt> = GlobalCell::new(0);
static p_wm_nobin: GlobalCell<OptInt> = GlobalCell::new(0);
static p_ai_nopaste: GlobalCell<c_int> = GlobalCell::new(0);
static p_et_nopaste: GlobalCell<c_int> = GlobalCell::new(0);
static p_sts_nopaste: GlobalCell<OptInt> = GlobalCell::new(0);
static p_tw_nopaste: GlobalCell<OptInt> = GlobalCell::new(0);
static p_wm_nopaste: GlobalCell<OptInt> = GlobalCell::new(0);
static p_vsts_nopaste: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub const OPTION_COUNT: usize = ::core::mem::size_of::<[vimoption_T; 374]>()
    .wrapping_div(::core::mem::size_of::<vimoption_T>())
    .wrapping_div(
        (::core::mem::size_of::<[vimoption_T; 374]>()
            .wrapping_rem(::core::mem::size_of::<vimoption_T>())
            == 0) as c_int as usize,
    );
pub const INC: c_int = 20 as c_int;
pub const GAP: c_int = 3 as c_int;
static expand_option_idx: GlobalCell<OptIndex> = GlobalCell::new(kOptInvalid);
static expand_option_start_col: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
static expand_option_name: GlobalCell<[c_char; 5]> = GlobalCell::new([
    't' as c_char,
    '_' as c_char,
    NUL as c_char,
    NUL as c_char,
    NUL as c_char,
]);
static expand_option_flags: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
static expand_option_append: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub const INT_MIN: c_int = -INT_MAX - 1 as c_int;
pub const INT_MAX: c_int = __INT_MAX__;
pub const true_0: c_int = 1 as c_int;
pub const false_0: c_int = 0 as c_int;
pub const PROJECT_NAME: [c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [c_char; 5]>(*b"nvim\0") };
pub const __INT_MAX__: c_int = 2147483647 as c_int;
