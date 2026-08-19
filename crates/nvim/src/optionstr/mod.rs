//! Setting a string option: the checks, the completion, and the callback
//! the option table names for each one.
//!
//! The module is one file per kind of work; the parent holds only what
//! more than one of them needs — the message texts, the two character
//! structs the screen draws from, and the constants the transpiled
//! headers left behind.
//!
//! | child | what |
//! | --- | --- |
//! | [`frame`] | what a callback is handed |
//! | [`check`] | vetting a value, and the sweeps that re-vet every buffer |
//! | [`flags`] | flag-letter sets and fixed word lists |
//! | [`expand`] | command-line completion of a value |
//! | [`chars`] | 'fillchars'/'listchars' field lists |
//! | [`display`] | the callbacks for what the screen looks like |
//! | [`buffer`] | the callbacks for how a buffer's text is handled |
//! | [`complete`] | the callbacks for completion, spelling and tags |
//! | [`statusline`] | the callbacks for format strings and session specs |

#![deny(unsafe_op_in_unsafe_fn)]

use crate::global_cell::GlobalCell;
use crate::types::{
    AlignTextPos, CharsOption, OptValType, WinSplit, WinStyle, fcs_chars_T, lcs_chars_T, schar_T,
};
use core::ffi::{CStr, c_int, c_uint};

mod check;
mod frame;
pub use self::check::*;
mod flags;
pub use self::flags::*;
mod expand;
pub use self::expand::*;
mod chars;
pub use self::chars::*;
mod display;
pub use self::display::*;
mod buffer;
pub use self::buffer::*;
mod complete;
pub use self::complete::*;
mod statusline;
pub use self::statusline::*;
use crate::regexp::vim_regexec;
use crate::terminal::terminal_notify_theme;
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitLeft: WinSplit = 0;
pub const kZIndexFloatDefault: c_uint = 50;
pub const kOptValTypeString: OptValType = 2;
pub const CPT_MENU: c_uint = 2;
pub const CPT_KIND: c_uint = 1;
pub const CPT_ABBR: c_uint = 0;
pub const kListchars: CharsOption = 1;
pub const kFillchars: CharsOption = 0;
pub const LSIZE: c_uint = 512;
/// 'imsearch' at -1 follows 'iminsert' rather than answering for itself.
pub const B_IMODE_USE_INSERT: c_int = -1;
pub const B_IMODE_NONE: c_int = 0;
pub const B_IMODE_LMAP: c_int = 1;
/// The only value nvim accepts for 'highlight': Vim's highlight-group
/// mapping is not implemented, so the option can only be left alone.
pub const HIGHLIGHT_INIT: &CStr = c"8:SpecialKey,~:EndOfBuffer,z:TermCursor,@:NonText,d:Directory,e:ErrorMsg,i:IncSearch,l:Search,y:CurSearch,m:MoreMsg,M:ModeMsg,n:LineNr,a:LineNrAbove,b:LineNrBelow,N:CursorLineNr,G:CursorLineSign,O:CursorLineFold,r:Question,s:StatusLine,S:StatusLineNC,c:VertSplit,t:Title,v:Visual,V:VisualNOS,w:WarningMsg,W:WildMenu,f:Folded,F:FoldColumn,A:DiffAdd,C:DiffChange,D:DiffDelete,T:DiffText,E:DiffTextAdd,>:SignColumn,-:Conceal,B:SpellBad,P:SpellCap,R:SpellRare,L:SpellLocal,+:Pmenu,=:PmenuSel,k:PmenuMatch,<:PmenuMatchSel,[:PmenuKind,]:PmenuKindSel,{:PmenuExtra,}:PmenuExtraSel,x:PmenuSbar,X:PmenuThumb,*:TabLine,#:TabLineSel,_:TabLineFill,!:CursorColumn,.:CursorLine,o:ColorColumn,q:QuickFixLine,z:StatusLineTerm,Z:StatusLineTermNC,g:MsgArea,h:ComplMatchIns,0:Whitespace,I:PreInsert";
pub const EOL_MAC: c_int = 2;
/// The letters 'formatoptions' accepts.
pub const FO_ALL: &CStr = c"tcro/q2vlb1mMBn,aw]jp";
/// The letters 'cpoptions' accepts.
pub const CPO_VI: &CStr = c"aAbBcCdDeEfFiIJKlLmMnoOpPqrRsStuvWxXyZ$!%+>;~_";
/// The letters 'whichwrap' accepts.
pub const WW_ALL: &CStr = c"bshl<>[]~";
/// The letters 'mouse' accepts.
pub const MOUSE_ALL: &CStr = c"anvichr";
pub const MOUSESCROLL_VERT_DFLT: c_int = 3;
pub const MOUSESCROLL_HOR_DFLT: c_int = 6;
/// The letters 'concealcursor' accepts.
pub const COCU_ALL: &CStr = c"nvic";
/// The flag letters a 'comments' part may carry before its own separator.
pub const COM_ALL: &CStr = c"nbsmexflrO";
pub const SCL_NO: c_int = -1;
pub const SID_NONE: c_int = -6;
pub const STL_IN_ICON: c_int = 1;
pub const STL_IN_TITLE: c_int = 2;
/// "E535: Illegal character after <%c>", for the options that spell a
/// field as a character followed by a value.
pub(crate) const e_illegal_character_after_chr: &CStr = c"E535: Illegal character after <%c>";
/// "E536: Comma required", for 'foldmarker'.
pub(crate) const e_comma_required: &CStr = c"E536: Comma required";
/// "E540: Unclosed expression sequence", for a `%{` with no `}`.
pub(crate) const e_unclosed_expression_sequence: &CStr = c"E540: Unclosed expression sequence";
/// "E542: Unbalanced groups", for a `%(` with no `%)`.
pub(crate) const e_unbalanced_groups: &CStr = c"E542: Unbalanced groups";
/// "E589: 'backupext' and 'patchmode' are equal".
pub(crate) const e_backupext_and_patchmode_are_equal: &CStr =
    c"E589: 'backupext' and 'patchmode' are equal";
/// "E595: 'showbreak' contains unprintable or wide character".
pub(crate) const e_showbreak_contains_unprintable_or_wide_character: &CStr =
    c"E595: 'showbreak' contains unprintable or wide character";
/// "E1511: Wrong number of characters for field", for a 'fillchars' or
/// 'listchars' field given too few or too many characters.
pub(crate) const e_wrong_number_of_characters_for_field_str: &CStr =
    c"E1511: Wrong number of characters for field \"%s\"";
/// "E1512: Wrong character width for field", for one that does not fit
/// in a single screen cell.
pub(crate) const e_wrong_character_width_for_field_str: &CStr =
    c"E1512: Wrong character width for field \"%s\"";
/// The letters 'shortmess' accepts. The trailing "nfxi" were removed
/// as flags and are now silently ignored.
pub const SHM_ALL: &CStr = c"rmlwaWtToOsAIcCqFSnfxi";
/// "E834: Conflicts with value of 'listchars'", reported when something
/// other than `:set` made the current value unrenderable.
pub(crate) const e_conflicts_with_value_of_listchars: &CStr =
    c"E834: Conflicts with value of 'listchars'";
/// "E835: Conflicts with value of 'fillchars'", as above.
pub(crate) const e_conflicts_with_value_of_fillchars: &CStr =
    c"E835: Conflicts with value of 'fillchars'";
static fcs_chars: GlobalCell<fcs_chars_T> = GlobalCell::new(fcs_chars_T {
    stl: 0,
    stlnc: 0,
    wbr: 0,
    horiz: 0,
    horizup: 0,
    horizdown: 0,
    vert: 0,
    vertleft: 0,
    vertright: 0,
    verthoriz: 0,
    fold: 0,
    foldopen: 0,
    foldclosed: 0,
    foldsep: 0,
    foldinner: 0,
    diff: 0,
    msgsep: 0,
    eob: 0,
    lastline: 0,
    trunc: 0,
    truncrl: 0,
});
static lcs_chars: GlobalCell<lcs_chars_T> = GlobalCell::new(lcs_chars_T {
    eol: 0,
    ext: 0,
    prec: 0,
    nbsp: 0,
    space: 0,
    tab1: 0,
    tab2: 0,
    tab3: 0,
    leadtab1: 0,
    leadtab2: 0,
    leadtab3: 0,
    lead: 0,
    trail: 0,
    multispace: ::core::ptr::null_mut::<schar_T>(),
    leadmultispace: ::core::ptr::null_mut::<schar_T>(),
    conceal: 0,
});
pub const INT_MAX: c_int = 2147483647;
