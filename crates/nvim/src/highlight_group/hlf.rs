//! The `HLF_*` family: indices into the builtin UI highlight table.
//!
//! One home for what used to be re-declared in 44 modules.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_char, c_int};

use crate::global_cell::ConstTable;

/// Index into `highlight_attr[]`: the builtin UI highlight groups.
///
/// The names the indices resolve to are in `hlf_names`; the two must stay
/// in step. `HLF_COUNT` is the length of both.
/// no UI highlight active
pub(crate) const HLF_NONE: c_int = 0;
/// `SpecialKey`: Meta & special keys listed with ":map", text that is displayed different from what it is
pub(crate) const HLF_8: c_int = 1;
/// `EndOfBuffer`: after the last line in the buffer
pub(crate) const HLF_EOB: c_int = 2;
/// `NonText`: @ characters at end of screen, characters that don't really exist in the text
pub(crate) const HLF_AT: c_int = 4;
/// `Directory`: directories in CTRL-D listing
pub(crate) const HLF_D: c_int = 5;
/// `ErrorMsg`: error messages
pub(crate) const HLF_E: c_int = 6;
/// `IncSearch`: incremental search
pub(crate) const HLF_I: c_int = 7;
/// `Search`: last search string
pub(crate) const HLF_L: c_int = 8;
/// `CurSearch`: current search match
pub(crate) const HLF_LC: c_int = 9;
/// `MoreMsg`: "--More--" message
pub(crate) const HLF_M: c_int = 10;
/// `ModeMsg`: Mode (e.g., "-- INSERT --")
pub(crate) const HLF_CM: c_int = 11;
/// `LineNr`: line number for ":number" and ":#" commands
pub(crate) const HLF_N: c_int = 12;
/// `LineNrAbove`: LineNrAbove
pub(crate) const HLF_LNA: c_int = 13;
/// `LineNrBelow`: LineNrBelow
pub(crate) const HLF_LNB: c_int = 14;
/// `CursorLineNr`: current line number when 'cursorline' is set
pub(crate) const HLF_CLN: c_int = 15;
/// `CursorLineSign`: current line sign column
pub(crate) const HLF_CLS: c_int = 16;
/// `CursorLineFold`: current line fold
pub(crate) const HLF_CLF: c_int = 17;
/// `Question`: return to continue message and yes/no questions
pub(crate) const HLF_R: c_int = 18;
/// `StatusLine`: status lines
pub(crate) const HLF_S: c_int = 19;
/// `StatusLineNC`: status lines of not-current windows
pub(crate) const HLF_SNC: c_int = 20;
/// `WinSeparator`: window split separators
pub(crate) const HLF_C: c_int = 21;
/// `Title`: Titles for output from ":set all", ":autocmd" etc.
pub(crate) const HLF_T: c_int = 23;
/// `Visual`: Visual mode
pub(crate) const HLF_V: c_int = 24;
/// `WarningMsg`: warning messages
pub(crate) const HLF_W: c_int = 26;
/// `WildMenu`: Wildmenu highlight
pub(crate) const HLF_WM: c_int = 27;
/// `Folded`: Folded line
pub(crate) const HLF_FL: c_int = 28;
/// `FoldColumn`: Fold column
pub(crate) const HLF_FC: c_int = 29;
/// `DiffAdd`: Added diff line
pub(crate) const HLF_ADD: c_int = 30;
/// `DiffChange`: Changed diff line
pub(crate) const HLF_CHD: c_int = 31;
/// `DiffDelete`: Deleted diff line
pub(crate) const HLF_DED: c_int = 32;
/// `DiffText`: Text Changed in diff line
pub(crate) const HLF_TXD: c_int = 33;
/// `DiffTextAdd`: Text Added in changed diff line
pub(crate) const HLF_TXA: c_int = 34;
/// `SignColumn`: Sign column
pub(crate) const HLF_SC: c_int = 35;
/// Concealed text
pub(crate) const HLF_CONCEAL: c_int = 36;
/// `SpellBad`: SpellBad
pub(crate) const HLF_SPB: c_int = 37;
/// `SpellCap`: SpellCap
pub(crate) const HLF_SPC: c_int = 38;
/// `SpellRare`: SpellRare
pub(crate) const HLF_SPR: c_int = 39;
/// `SpellLocal`: SpellLocal
pub(crate) const HLF_SPL: c_int = 40;
/// `Pmenu`: popup menu normal item
pub(crate) const HLF_PNI: c_int = 41;
/// `PmenuSel`: popup menu selected item
pub(crate) const HLF_PSI: c_int = 42;
/// `PmenuMatch`: popup menu matched text in normal item
pub(crate) const HLF_PMNI: c_int = 43;
/// `PmenuMatchSel`: popup menu matched text in selected item
pub(crate) const HLF_PMSI: c_int = 44;
/// `PmenuKind`: popup menu normal item "kind"
pub(crate) const HLF_PNK: c_int = 45;
/// `PmenuKindSel`: popup menu selected item "kind"
pub(crate) const HLF_PSK: c_int = 46;
/// `PmenuExtra`: popup menu normal item "menu" (extra text)
pub(crate) const HLF_PNX: c_int = 47;
/// `PmenuExtraSel`: popup menu selected item "menu" (extra text)
pub(crate) const HLF_PSX: c_int = 48;
/// `PmenuSbar`: popup menu scrollbar
pub(crate) const HLF_PSB: c_int = 49;
/// `PmenuThumb`: popup menu scrollbar thumb
pub(crate) const HLF_PST: c_int = 50;
/// `PmenuBorder`: popup menu border
pub(crate) const HLF_PBR: c_int = 51;
/// `TabLine`: tabpage line
pub(crate) const HLF_TP: c_int = 52;
/// `TabLineSel`: tabpage line selected
pub(crate) const HLF_TPS: c_int = 53;
/// `TabLineFill`: tabpage line filler
pub(crate) const HLF_TPF: c_int = 54;
/// `CursorColumn`: 'cursorcolumn'
pub(crate) const HLF_CUC: c_int = 55;
/// `CursorLine`: 'cursorline'
pub(crate) const HLF_CUL: c_int = 56;
/// `ColorColumn`: 'colorcolumn'
pub(crate) const HLF_MC: c_int = 57;
/// `QuickFixLine`: selected quickfix line
pub(crate) const HLF_QFL: c_int = 58;
/// `Whitespace`: Whitespace
pub(crate) const HLF_0: c_int = 59;
/// `NormalNC`: NormalNC: Normal text in non-current windows
pub(crate) const HLF_INACTIVE: c_int = 60;
/// `MsgSeparator`: message separator line
pub(crate) const HLF_MSGSEP: c_int = 61;
/// `NormalFloat`: Floating window
pub(crate) const HLF_NFLOAT: c_int = 62;
/// `MsgArea`: Message area
pub(crate) const HLF_MSG: c_int = 63;
/// `FloatBorder`: Floating window border
pub(crate) const HLF_BORDER: c_int = 64;
/// `WinBar`: Window bars
pub(crate) const HLF_WBR: c_int = 65;
/// `WinBarNC`: Window bars of not-current windows
pub(crate) const HLF_WBRNC: c_int = 66;
/// `FloatTitle`: Float Border Title
pub(crate) const HLF_BTITLE: c_int = 68;
/// `FloatFooter`: Float Border Footer
pub(crate) const HLF_BFOOTER: c_int = 69;
/// `StderrMsg`: stderr messages (from shell)
pub(crate) const HLF_SE: c_int = 72;
/// `StdoutMsg`: stdout messages (from shell)
pub(crate) const HLF_SO: c_int = 73;
/// One past the last group.
pub(crate) const HLF_COUNT: c_int = 76;

/// The group each `HLF_*` index names.
///
/// Index 0 (`HLF_NONE`) is a NULL, as upstream has it: the sentinel is not a
/// group and the two readers that walk from 0 hand it straight to
/// `cstr_as_string`, which answers an empty string for a NULL.
pub(crate) static hlf_names: ConstTable<[*const c_char; HLF_COUNT as usize]> = ConstTable::new([
    ::core::ptr::null::<c_char>(),
    c"SpecialKey".as_ptr(),
    c"EndOfBuffer".as_ptr(),
    c"TermCursor".as_ptr(),
    c"NonText".as_ptr(),
    c"Directory".as_ptr(),
    c"ErrorMsg".as_ptr(),
    c"IncSearch".as_ptr(),
    c"Search".as_ptr(),
    c"CurSearch".as_ptr(),
    c"MoreMsg".as_ptr(),
    c"ModeMsg".as_ptr(),
    c"LineNr".as_ptr(),
    c"LineNrAbove".as_ptr(),
    c"LineNrBelow".as_ptr(),
    c"CursorLineNr".as_ptr(),
    c"CursorLineSign".as_ptr(),
    c"CursorLineFold".as_ptr(),
    c"Question".as_ptr(),
    c"StatusLine".as_ptr(),
    c"StatusLineNC".as_ptr(),
    c"WinSeparator".as_ptr(),
    c"VertSplit".as_ptr(),
    c"Title".as_ptr(),
    c"Visual".as_ptr(),
    c"VisualNC".as_ptr(),
    c"WarningMsg".as_ptr(),
    c"WildMenu".as_ptr(),
    c"Folded".as_ptr(),
    c"FoldColumn".as_ptr(),
    c"DiffAdd".as_ptr(),
    c"DiffChange".as_ptr(),
    c"DiffDelete".as_ptr(),
    c"DiffText".as_ptr(),
    c"DiffTextAdd".as_ptr(),
    c"SignColumn".as_ptr(),
    c"Conceal".as_ptr(),
    c"SpellBad".as_ptr(),
    c"SpellCap".as_ptr(),
    c"SpellRare".as_ptr(),
    c"SpellLocal".as_ptr(),
    c"Pmenu".as_ptr(),
    c"PmenuSel".as_ptr(),
    c"PmenuMatch".as_ptr(),
    c"PmenuMatchSel".as_ptr(),
    c"PmenuKind".as_ptr(),
    c"PmenuKindSel".as_ptr(),
    c"PmenuExtra".as_ptr(),
    c"PmenuExtraSel".as_ptr(),
    c"PmenuSbar".as_ptr(),
    c"PmenuThumb".as_ptr(),
    c"PmenuBorder".as_ptr(),
    c"TabLine".as_ptr(),
    c"TabLineSel".as_ptr(),
    c"TabLineFill".as_ptr(),
    c"CursorColumn".as_ptr(),
    c"CursorLine".as_ptr(),
    c"ColorColumn".as_ptr(),
    c"QuickFixLine".as_ptr(),
    c"Whitespace".as_ptr(),
    c"NormalNC".as_ptr(),
    c"MsgSeparator".as_ptr(),
    c"NormalFloat".as_ptr(),
    c"MsgArea".as_ptr(),
    c"FloatBorder".as_ptr(),
    c"WinBar".as_ptr(),
    c"WinBarNC".as_ptr(),
    c"Cursor".as_ptr(),
    c"FloatTitle".as_ptr(),
    c"FloatFooter".as_ptr(),
    c"StatusLineTerm".as_ptr(),
    c"StatusLineTermNC".as_ptr(),
    c"StderrMsg".as_ptr(),
    c"StdoutMsg".as_ptr(),
    c"OkMsg".as_ptr(),
    c"PreInsert".as_ptr(),
]);
