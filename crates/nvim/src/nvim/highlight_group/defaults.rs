//! The compiled-in default highlighting, and colour schemes.
//!
//! [`init_highlight`] sources `g:colors_name` if one is set
//! ([`load_colors`]) and otherwise feeds these tables — one `:highlight`
//! command per line — straight to [`do_highlight`]. They are compiled in so
//! that a build with no runtime files still has usable colours.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};

use crate::src::nvim::autocmd::{EVENT_COLORSCHEME, EVENT_COLORSCHEMEPRE, apply_autocmds};
use crate::src::nvim::eval::vars::get_var_value;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    cterm_normal_bg_color, cterm_normal_fg_color, curbuf, normal_bg, normal_fg, normal_sp, p_bg,
};
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::runtime::source_runtime_vim_lua;
use crate::src::nvim::types::RgbValue;

use super::{DIP_OPT, DIP_START, OK, do_highlight};

/// The groups whose definition does not depend on `'background'`.
static HIGHLIGHT_INIT_BOTH: [&CStr; 174] = [
    c"Cursor            guifg=bg      guibg=fg",
    c"CursorLineNr      gui=bold      cterm=bold",
    c"PmenuMatch        gui=bold      cterm=bold",
    c"PmenuMatchSel     gui=bold      cterm=bold",
    c"PmenuSel          gui=reverse   cterm=reverse,underline blend=0",
    c"RedrawDebugNormal gui=reverse   cterm=reverse",
    c"TabLineSel        gui=bold      cterm=NONE",
    c"TermCursor        gui=reverse   cterm=reverse",
    c"Underlined        gui=underline cterm=underline",
    c"lCursor           guifg=bg      guibg=fg",
    c"default link CursorIM         Cursor",
    c"default link CursorLineFold   FoldColumn",
    c"default link CursorLineSign   SignColumn",
    c"default link DiffTextAdd      DiffText",
    c"default link EndOfBuffer      NonText",
    c"default link FloatBorder      NormalFloat",
    c"default link FloatFooter      FloatTitle",
    c"default link FloatTitle       Title",
    c"default link FoldColumn       SignColumn",
    c"default link IncSearch        CurSearch",
    c"default link LineNrAbove      LineNr",
    c"default link LineNrBelow      LineNr",
    c"default link MsgSeparator     StatusLine",
    c"default link MsgArea          NONE",
    c"default link NormalNC         NONE",
    c"default link PmenuExtra       Pmenu",
    c"default link PmenuExtraSel    PmenuSel",
    c"default link PmenuKind        Pmenu",
    c"default link PmenuKindSel     PmenuSel",
    c"default link PmenuSbar        Pmenu",
    c"default link PmenuBorder        Pmenu",
    c"default link PmenuShadow        FloatShadow",
    c"default link PmenuShadowThrough FloatShadowThrough",
    c"default link PreInsert        Added",
    c"default link ComplMatchIns    NONE",
    c"default link ComplHint        NonText",
    c"default link ComplHintMore    MoreMsg",
    c"default link Substitute       Search",
    c"default link StatusLineTerm   StatusLine",
    c"default link StatusLineTermNC StatusLineNC",
    c"default link StderrMsg        ErrorMsg",
    c"default link StdoutMsg        NONE",
    c"default link TabLine          StatusLineNC",
    c"default link TabLineFill      TabLine",
    c"default link VertSplit        WinSeparator",
    c"default link VisualNOS        Visual",
    c"default link Whitespace       NonText",
    c"default link WildMenu         PmenuSel",
    c"default link WinSeparator     Normal",
    c"default link Character      Constant",
    c"default link Number         Constant",
    c"default link Boolean        Constant",
    c"default link Float          Number",
    c"default link Conditional    Statement",
    c"default link Repeat         Statement",
    c"default link Label          Statement",
    c"default link Keyword        Statement",
    c"default link Exception      Statement",
    c"default link Include        PreProc",
    c"default link Define         PreProc",
    c"default link Macro          PreProc",
    c"default link PreCondit      PreProc",
    c"default link StorageClass   Type",
    c"default link Structure      Type",
    c"default link Typedef        Type",
    c"default link Tag            Special",
    c"default link SpecialChar    Special",
    c"default link SpecialComment Special",
    c"default link Debug          Special",
    c"default link SpecialKey     Special",
    c"default link Ignore         Normal",
    c"default link LspCodeLens                 NonText",
    c"default link LspCodeLensSeparator        LspCodeLens",
    c"default link LspInlayHint                NonText",
    c"default link LspReferenceRead            LspReferenceText",
    c"default link LspReferenceText            Visual",
    c"default link LspReferenceWrite           LspReferenceText",
    c"default link LspReferenceTarget          LspReferenceText",
    c"default link LspSignatureActiveParameter Visual",
    c"default link SnippetTabstop              Visual",
    c"default link SnippetTabstopActive        SnippetTabstop",
    c"default link DiagnosticFloatingError    DiagnosticError",
    c"default link DiagnosticFloatingWarn     DiagnosticWarn",
    c"default link DiagnosticFloatingInfo     DiagnosticInfo",
    c"default link DiagnosticFloatingHint     DiagnosticHint",
    c"default link DiagnosticFloatingOk       DiagnosticOk",
    c"default link DiagnosticVirtualTextError DiagnosticError",
    c"default link DiagnosticVirtualTextWarn  DiagnosticWarn",
    c"default link DiagnosticVirtualTextInfo  DiagnosticInfo",
    c"default link DiagnosticVirtualTextHint  DiagnosticHint",
    c"default link DiagnosticVirtualTextOk    DiagnosticOk",
    c"default link DiagnosticVirtualLinesError DiagnosticError",
    c"default link DiagnosticVirtualLinesWarn  DiagnosticWarn",
    c"default link DiagnosticVirtualLinesInfo  DiagnosticInfo",
    c"default link DiagnosticVirtualLinesHint  DiagnosticHint",
    c"default link DiagnosticVirtualLinesOk    DiagnosticOk",
    c"default link DiagnosticSignError        DiagnosticError",
    c"default link DiagnosticSignWarn         DiagnosticWarn",
    c"default link DiagnosticSignInfo         DiagnosticInfo",
    c"default link DiagnosticSignHint         DiagnosticHint",
    c"default link DiagnosticSignOk           DiagnosticOk",
    c"default link DiagnosticUnnecessary      Comment",
    c"default link @variable.builtin           Special",
    c"default link @variable.parameter.builtin Special",
    c"default link @constant         Constant",
    c"default link @constant.builtin Special",
    c"default link @module         Structure",
    c"default link @module.builtin Special",
    c"default link @label          Label",
    c"default link @string             String",
    c"default link @string.regexp      @string.special",
    c"default link @string.escape      @string.special",
    c"default link @string.special     SpecialChar",
    c"default link @string.special.url Underlined",
    c"default link @character         Character",
    c"default link @character.special SpecialChar",
    c"default link @boolean      Boolean",
    c"default link @number       Number",
    c"default link @number.float Float",
    c"default link @type         Type",
    c"default link @type.builtin Special",
    c"default link @attribute         Macro",
    c"default link @attribute.builtin Special",
    c"default link @property          Identifier",
    c"default link @function         Function",
    c"default link @function.builtin Special",
    c"default link @constructor Special",
    c"default link @operator    Operator",
    c"default link @keyword Keyword",
    c"default link @punctuation         Delimiter",
    c"default link @punctuation.special Special",
    c"default link @comment Comment",
    c"default link @comment.error   DiagnosticError",
    c"default link @comment.warning DiagnosticWarn",
    c"default link @comment.note    DiagnosticInfo",
    c"default link @comment.todo    Todo",
    c"@markup.strong        gui=bold          cterm=bold",
    c"@markup.italic        gui=italic        cterm=italic",
    c"@markup.strikethrough gui=strikethrough cterm=strikethrough",
    c"@markup.underline     gui=underline     cterm=underline",
    c"default link @markup         Special",
    c"default link @markup.heading Title",
    c"default link @markup.link    Underlined",
    c"default link @diff.plus  Added",
    c"default link @diff.minus Removed",
    c"default link @diff.delta Changed",
    c"default link @tag         Tag",
    c"default link @tag.builtin Special",
    c"default @markup.heading.1.delimiter.vimdoc guibg=bg guifg=bg guisp=fg gui=underdouble,nocombine ctermbg=NONE ctermfg=NONE cterm=underdouble,nocombine",
    c"default @markup.heading.2.delimiter.vimdoc guibg=bg guifg=bg guisp=fg gui=underline,nocombine ctermbg=NONE ctermfg=NONE cterm=underline,nocombine",
    c"default link @lsp.type.class         @type",
    c"default link @lsp.type.comment       @comment",
    c"default link @lsp.type.decorator     @attribute",
    c"default link @lsp.type.enum          @type",
    c"default link @lsp.type.enumMember    @constant",
    c"default link @lsp.type.event         @type",
    c"default link @lsp.type.function      @function",
    c"default link @lsp.type.interface     @type",
    c"default link @lsp.type.keyword       @keyword",
    c"default link @lsp.type.macro         @constant.macro",
    c"default link @lsp.type.method        @function.method",
    c"default link @lsp.type.modifier      @type.qualifier",
    c"default link @lsp.type.namespace     @module",
    c"default link @lsp.type.number        @number",
    c"default link @lsp.type.operator      @operator",
    c"default link @lsp.type.parameter     @variable.parameter",
    c"default link @lsp.type.property      @property",
    c"default link @lsp.type.regexp        @string.regexp",
    c"default link @lsp.type.string        @string",
    c"default link @lsp.type.struct        @type",
    c"default link @lsp.type.type          @type",
    c"default link @lsp.type.typeParameter @type.definition",
    c"default link @lsp.type.variable      @variable",
    c"default link @lsp.mod.deprecated DiagnosticDeprecated",
];

/// `'background'` is `light`.
static HIGHLIGHT_INIT_LIGHT: [&CStr; 70] = [
    c"Normal guifg=NvimDarkGrey2 guibg=NvimLightGrey2 ctermfg=NONE ctermbg=NONE",
    c"Added                guifg=NvimDarkGreen                                  ctermfg=2",
    c"Changed              guifg=NvimDarkCyan                                   ctermfg=6",
    c"ColorColumn                               guibg=NvimLightGrey4            cterm=reverse",
    c"Conceal              guifg=NvimLightGrey4",
    c"CurSearch            guifg=NvimLightGrey1 guibg=NvimDarkYellow            ctermfg=15 ctermbg=3",
    c"CursorColumn                              guibg=NvimLightGrey3",
    c"CursorLine                                guibg=NvimLightGrey3",
    c"DiffAdd              guifg=NvimDarkGrey1  guibg=NvimLightGreen            ctermfg=15 ctermbg=2",
    c"DiffChange           guifg=NvimDarkGrey1  guibg=NvimLightGrey4",
    c"DiffDelete           guifg=NvimDarkRed                          gui=bold  ctermfg=1 cterm=bold",
    c"DiffText             guifg=NvimDarkGrey1  guibg=NvimLightCyan             ctermfg=15 ctermbg=6",
    c"Directory            guifg=NvimDarkCyan                                   ctermfg=6",
    c"ErrorMsg             guifg=NvimDarkRed                                    ctermfg=1",
    c"FloatShadow                               guibg=NvimLightGrey4            ctermbg=0 blend=80",
    c"FloatShadowThrough                        guibg=NvimLightGrey4            ctermbg=0 blend=100",
    c"Folded               guifg=NvimDarkGrey4  guibg=NvimLightGrey1",
    c"LineNr               guifg=NvimLightGrey4",
    c"MatchParen                                guibg=NvimLightGrey4  gui=bold  cterm=bold,underline",
    c"ModeMsg              guifg=NvimDarkGreen                                  ctermfg=2",
    c"MoreMsg              guifg=NvimDarkCyan                                   ctermfg=6",
    c"NonText              guifg=NvimLightGrey4",
    c"NormalFloat                               guibg=NvimLightGrey1",
    c"OkMsg                guifg=NvimDarkGreen                                  ctermfg=2",
    c"Pmenu                                     guibg=NvimLightGrey3            cterm=reverse",
    c"PmenuThumb                                guibg=NvimLightGrey4",
    c"Question             guifg=NvimDarkCyan                                   ctermfg=6",
    c"QuickFixLine         guifg=NvimDarkCyan                                   ctermfg=6",
    c"RedrawDebugClear                          guibg=NvimLightYellow           ctermfg=15 ctermbg=3",
    c"RedrawDebugComposed                       guibg=NvimLightGreen            ctermfg=15 ctermbg=2",
    c"RedrawDebugRecompose                      guibg=NvimLightRed              ctermfg=15 ctermbg=1",
    c"Removed              guifg=NvimDarkRed                                    ctermfg=1",
    c"Search               guifg=NvimDarkGrey1  guibg=NvimLightYellow           ctermfg=15 ctermbg=3",
    c"SignColumn           guifg=NvimLightGrey4",
    c"SpellBad             guisp=NvimDarkRed    gui=undercurl                   cterm=undercurl",
    c"SpellCap             guisp=NvimDarkYellow gui=undercurl                   cterm=undercurl",
    c"SpellLocal           guisp=NvimDarkGreen  gui=undercurl                   cterm=undercurl",
    c"SpellRare            guisp=NvimDarkCyan   gui=undercurl                   cterm=undercurl",
    c"StatusLine           guifg=NvimDarkGrey2  guibg=NvimLightGrey4            cterm=reverse",
    c"StatusLineNC         guifg=NvimDarkGrey3  guibg=NvimLightGrey3            cterm=bold,underline",
    c"Title                guifg=NvimDarkGrey2                        gui=bold  cterm=bold",
    c"Visual                                    guibg=NvimLightGrey4            ctermfg=15 ctermbg=0",
    c"WarningMsg           guifg=NvimDarkYellow                                 ctermfg=3",
    c"WinBar               guifg=NvimDarkGrey4  guibg=NvimLightGrey1  gui=bold  cterm=bold",
    c"WinBarNC             guifg=NvimDarkGrey4  guibg=NvimLightGrey1            cterm=bold",
    c"Constant   guifg=NvimDarkGrey2",
    c"Operator   guifg=NvimDarkGrey2",
    c"PreProc    guifg=NvimDarkGrey2",
    c"Type       guifg=NvimDarkGrey2",
    c"Delimiter  guifg=NvimDarkGrey2",
    c"Comment    guifg=NvimDarkGrey4",
    c"String     guifg=NvimDarkGreen                    ctermfg=2",
    c"Identifier guifg=NvimDarkBlue                     ctermfg=4",
    c"Function   guifg=NvimDarkCyan                     ctermfg=6",
    c"Statement  guifg=NvimDarkGrey2 gui=bold           cterm=bold",
    c"Special    guifg=NvimDarkCyan                     ctermfg=6",
    c"Error      guifg=NvimDarkGrey1 guibg=NvimLightRed ctermfg=15 ctermbg=1",
    c"Todo       guifg=NvimDarkGrey2 gui=bold           cterm=bold",
    c"DiagnosticError          guifg=NvimDarkRed                      ctermfg=1",
    c"DiagnosticWarn           guifg=NvimDarkYellow                   ctermfg=3",
    c"DiagnosticInfo           guifg=NvimDarkCyan                     ctermfg=6",
    c"DiagnosticHint           guifg=NvimDarkBlue                     ctermfg=4",
    c"DiagnosticOk             guifg=NvimDarkGreen                    ctermfg=2",
    c"DiagnosticUnderlineError guisp=NvimDarkRed    gui=underline     cterm=underline",
    c"DiagnosticUnderlineWarn  guisp=NvimDarkYellow gui=underline     cterm=underline",
    c"DiagnosticUnderlineInfo  guisp=NvimDarkCyan   gui=underline     cterm=underline",
    c"DiagnosticUnderlineHint  guisp=NvimDarkBlue   gui=underline     cterm=underline",
    c"DiagnosticUnderlineOk    guisp=NvimDarkGreen  gui=underline     cterm=underline",
    c"DiagnosticDeprecated     guisp=NvimDarkRed    gui=strikethrough cterm=strikethrough",
    c"@variable guifg=NvimDarkGrey2",
];

/// `'background'` is `dark`.
static HIGHLIGHT_INIT_DARK: [&CStr; 70] = [
    c"Normal guifg=NvimLightGrey2 guibg=NvimDarkGrey2 ctermfg=NONE ctermbg=NONE",
    c"Added                guifg=NvimLightGreen                                ctermfg=10",
    c"Changed              guifg=NvimLightCyan                                 ctermfg=14",
    c"ColorColumn                                guibg=NvimDarkGrey4           cterm=reverse",
    c"Conceal              guifg=NvimDarkGrey4",
    c"CurSearch            guifg=NvimDarkGrey1   guibg=NvimLightYellow         ctermfg=0 ctermbg=11",
    c"CursorColumn                               guibg=NvimDarkGrey3",
    c"CursorLine                                 guibg=NvimDarkGrey3",
    c"DiffAdd              guifg=NvimLightGrey1  guibg=NvimDarkGreen           ctermfg=0 ctermbg=10",
    c"DiffChange           guifg=NvimLightGrey1  guibg=NvimDarkGrey4",
    c"DiffDelete           guifg=NvimLightRed                         gui=bold ctermfg=9 cterm=bold",
    c"DiffText             guifg=NvimLightGrey1  guibg=NvimDarkCyan            ctermfg=0 ctermbg=14",
    c"Directory            guifg=NvimLightCyan                                 ctermfg=14",
    c"ErrorMsg             guifg=NvimLightRed                                  ctermfg=9",
    c"FloatShadow                                guibg=NvimDarkGrey4           ctermbg=0 blend=80",
    c"FloatShadowThrough                         guibg=NvimDarkGrey4           ctermbg=0 blend=100",
    c"Folded               guifg=NvimLightGrey4  guibg=NvimDarkGrey1",
    c"LineNr               guifg=NvimDarkGrey4",
    c"MatchParen                                 guibg=NvimDarkGrey4  gui=bold cterm=bold,underline",
    c"ModeMsg              guifg=NvimLightGreen                                ctermfg=10",
    c"MoreMsg              guifg=NvimLightCyan                                 ctermfg=14",
    c"NonText              guifg=NvimDarkGrey4",
    c"NormalFloat                                guibg=NvimDarkGrey1",
    c"OkMsg                guifg=NvimLightGreen                                ctermfg=10",
    c"Pmenu                                      guibg=NvimDarkGrey3           cterm=reverse",
    c"PmenuThumb                                 guibg=NvimDarkGrey4",
    c"Question             guifg=NvimLightCyan                                 ctermfg=14",
    c"QuickFixLine         guifg=NvimLightCyan                                 ctermfg=14",
    c"RedrawDebugClear                           guibg=NvimDarkYellow          ctermfg=0 ctermbg=11",
    c"RedrawDebugComposed                        guibg=NvimDarkGreen           ctermfg=0 ctermbg=10",
    c"RedrawDebugRecompose                       guibg=NvimDarkRed             ctermfg=0 ctermbg=9",
    c"Removed              guifg=NvimLightRed                                  ctermfg=9",
    c"Search               guifg=NvimLightGrey1  guibg=NvimDarkYellow          ctermfg=0 ctermbg=11",
    c"SignColumn           guifg=NvimDarkGrey4",
    c"SpellBad             guisp=NvimLightRed    gui=undercurl                 cterm=undercurl",
    c"SpellCap             guisp=NvimLightYellow gui=undercurl                 cterm=undercurl",
    c"SpellLocal           guisp=NvimLightGreen  gui=undercurl                 cterm=undercurl",
    c"SpellRare            guisp=NvimLightCyan   gui=undercurl                 cterm=undercurl",
    c"StatusLine           guifg=NvimLightGrey2  guibg=NvimDarkGrey4           cterm=reverse",
    c"StatusLineNC         guifg=NvimLightGrey3  guibg=NvimDarkGrey3           cterm=bold,underline",
    c"Title                guifg=NvimLightGrey2                       gui=bold cterm=bold",
    c"Visual                                     guibg=NvimDarkGrey4           ctermfg=0 ctermbg=15",
    c"WarningMsg           guifg=NvimLightYellow                               ctermfg=11",
    c"WinBar               guifg=NvimLightGrey4  guibg=NvimDarkGrey1  gui=bold cterm=bold",
    c"WinBarNC             guifg=NvimLightGrey4  guibg=NvimDarkGrey1           cterm=bold",
    c"Constant   guifg=NvimLightGrey2",
    c"Operator   guifg=NvimLightGrey2",
    c"PreProc    guifg=NvimLightGrey2",
    c"Type       guifg=NvimLightGrey2",
    c"Delimiter  guifg=NvimLightGrey2",
    c"Comment    guifg=NvimLightGrey4",
    c"String     guifg=NvimLightGreen                   ctermfg=10",
    c"Identifier guifg=NvimLightBlue                    ctermfg=12",
    c"Function   guifg=NvimLightCyan                    ctermfg=14",
    c"Statement  guifg=NvimLightGrey2 gui=bold          cterm=bold",
    c"Special    guifg=NvimLightCyan                    ctermfg=14",
    c"Error      guifg=NvimLightGrey1 guibg=NvimDarkRed ctermfg=0 ctermbg=9",
    c"Todo       guifg=NvimLightGrey2 gui=bold          cterm=bold",
    c"DiagnosticError          guifg=NvimLightRed                      ctermfg=9",
    c"DiagnosticWarn           guifg=NvimLightYellow                   ctermfg=11",
    c"DiagnosticInfo           guifg=NvimLightCyan                     ctermfg=14",
    c"DiagnosticHint           guifg=NvimLightBlue                     ctermfg=12",
    c"DiagnosticOk             guifg=NvimLightGreen                    ctermfg=10",
    c"DiagnosticUnderlineError guisp=NvimLightRed    gui=underline     cterm=underline",
    c"DiagnosticUnderlineWarn  guisp=NvimLightYellow gui=underline     cterm=underline",
    c"DiagnosticUnderlineInfo  guisp=NvimLightCyan   gui=underline     cterm=underline",
    c"DiagnosticUnderlineHint  guisp=NvimLightBlue   gui=underline     cterm=underline",
    c"DiagnosticUnderlineOk    guisp=NvimLightGreen  gui=underline     cterm=underline",
    c"DiagnosticDeprecated     guisp=NvimLightRed    gui=strikethrough cterm=strikethrough",
    c"@variable guifg=NvimLightGrey2",
];

/// The `Nvim*` groups the command-line highlighter uses.
///
/// Still a NUL-terminated array of raw pointers, and still exported: it is
/// the only one of the four with a reader outside this crate —
/// `test/unit/viml/expressions/parser_spec.lua` walks it over the FFI to
/// learn which groups the parser may name.
#[unsafe(no_mangle)]
pub static highlight_init_cmdline: GlobalCell<[*const c_char; 141]> = GlobalCell::new([
    c"NvimInternalError ctermfg=Red ctermbg=Red guifg=Red guibg=Red".as_ptr(),
    c"default link NvimAssignment Operator".as_ptr(),
    c"default link NvimPlainAssignment NvimAssignment".as_ptr(),
    c"default link NvimAugmentedAssignment NvimAssignment".as_ptr(),
    c"default link NvimAssignmentWithAddition NvimAugmentedAssignment".as_ptr(),
    c"default link NvimAssignmentWithSubtraction NvimAugmentedAssignment".as_ptr(),
    c"default link NvimAssignmentWithConcatenation NvimAugmentedAssignment".as_ptr(),
    c"default link NvimOperator Operator".as_ptr(),
    c"default link NvimUnaryOperator NvimOperator".as_ptr(),
    c"default link NvimUnaryPlus NvimUnaryOperator".as_ptr(),
    c"default link NvimUnaryMinus NvimUnaryOperator".as_ptr(),
    c"default link NvimNot NvimUnaryOperator".as_ptr(),
    c"default link NvimBinaryOperator NvimOperator".as_ptr(),
    c"default link NvimComparison NvimBinaryOperator".as_ptr(),
    c"default link NvimComparisonModifier NvimComparison".as_ptr(),
    c"default link NvimBinaryPlus NvimBinaryOperator".as_ptr(),
    c"default link NvimBinaryMinus NvimBinaryOperator".as_ptr(),
    c"default link NvimConcat NvimBinaryOperator".as_ptr(),
    c"default link NvimConcatOrSubscript NvimConcat".as_ptr(),
    c"default link NvimOr NvimBinaryOperator".as_ptr(),
    c"default link NvimAnd NvimBinaryOperator".as_ptr(),
    c"default link NvimMultiplication NvimBinaryOperator".as_ptr(),
    c"default link NvimDivision NvimBinaryOperator".as_ptr(),
    c"default link NvimMod NvimBinaryOperator".as_ptr(),
    c"default link NvimTernary NvimOperator".as_ptr(),
    c"default link NvimTernaryColon NvimTernary".as_ptr(),
    c"default link NvimParenthesis Delimiter".as_ptr(),
    c"default link NvimLambda NvimParenthesis".as_ptr(),
    c"default link NvimNestingParenthesis NvimParenthesis".as_ptr(),
    c"default link NvimCallingParenthesis NvimParenthesis".as_ptr(),
    c"default link NvimSubscript NvimParenthesis".as_ptr(),
    c"default link NvimSubscriptBracket NvimSubscript".as_ptr(),
    c"default link NvimSubscriptColon NvimSubscript".as_ptr(),
    c"default link NvimCurly NvimSubscript".as_ptr(),
    c"default link NvimContainer NvimParenthesis".as_ptr(),
    c"default link NvimDict NvimContainer".as_ptr(),
    c"default link NvimList NvimContainer".as_ptr(),
    c"default link NvimIdentifier Identifier".as_ptr(),
    c"default link NvimIdentifierScope NvimIdentifier".as_ptr(),
    c"default link NvimIdentifierScopeDelimiter NvimIdentifier".as_ptr(),
    c"default link NvimIdentifierName NvimIdentifier".as_ptr(),
    c"default link NvimIdentifierKey NvimIdentifier".as_ptr(),
    c"default link NvimColon Delimiter".as_ptr(),
    c"default link NvimComma Delimiter".as_ptr(),
    c"default link NvimArrow Delimiter".as_ptr(),
    c"default link NvimRegister SpecialChar".as_ptr(),
    c"default link NvimNumber Number".as_ptr(),
    c"default link NvimFloat NvimNumber".as_ptr(),
    c"default link NvimNumberPrefix Type".as_ptr(),
    c"default link NvimOptionSigil Type".as_ptr(),
    c"default link NvimOptionName NvimIdentifier".as_ptr(),
    c"default link NvimOptionScope NvimIdentifierScope".as_ptr(),
    c"default link NvimOptionScopeDelimiter NvimIdentifierScopeDelimiter".as_ptr(),
    c"default link NvimEnvironmentSigil NvimOptionSigil".as_ptr(),
    c"default link NvimEnvironmentName NvimIdentifier".as_ptr(),
    c"default link NvimString String".as_ptr(),
    c"default link NvimStringBody NvimString".as_ptr(),
    c"default link NvimStringQuote NvimString".as_ptr(),
    c"default link NvimStringSpecial SpecialChar".as_ptr(),
    c"default link NvimSingleQuote NvimStringQuote".as_ptr(),
    c"default link NvimSingleQuotedBody NvimStringBody".as_ptr(),
    c"default link NvimSingleQuotedQuote NvimStringSpecial".as_ptr(),
    c"default link NvimDoubleQuote NvimStringQuote".as_ptr(),
    c"default link NvimDoubleQuotedBody NvimStringBody".as_ptr(),
    c"default link NvimDoubleQuotedEscape NvimStringSpecial".as_ptr(),
    c"default link NvimFigureBrace NvimInternalError".as_ptr(),
    c"default link NvimSingleQuotedUnknownEscape NvimInternalError".as_ptr(),
    c"default link NvimSpacing Normal".as_ptr(),
    c"default link NvimInvalidSingleQuotedUnknownEscape NvimInternalError".as_ptr(),
    c"default link NvimInvalid Error".as_ptr(),
    c"default link NvimInvalidAssignment NvimInvalid".as_ptr(),
    c"default link NvimInvalidPlainAssignment NvimInvalidAssignment".as_ptr(),
    c"default link NvimInvalidAugmentedAssignment NvimInvalidAssignment".as_ptr(),
    c"default link NvimInvalidAssignmentWithAddition NvimInvalidAugmentedAssignment".as_ptr(),
    c"default link NvimInvalidAssignmentWithSubtraction NvimInvalidAugmentedAssignment".as_ptr(),
    c"default link NvimInvalidAssignmentWithConcatenation NvimInvalidAugmentedAssignment".as_ptr(),
    c"default link NvimInvalidOperator NvimInvalid".as_ptr(),
    c"default link NvimInvalidUnaryOperator NvimInvalidOperator".as_ptr(),
    c"default link NvimInvalidUnaryPlus NvimInvalidUnaryOperator".as_ptr(),
    c"default link NvimInvalidUnaryMinus NvimInvalidUnaryOperator".as_ptr(),
    c"default link NvimInvalidNot NvimInvalidUnaryOperator".as_ptr(),
    c"default link NvimInvalidBinaryOperator NvimInvalidOperator".as_ptr(),
    c"default link NvimInvalidComparison NvimInvalidBinaryOperator".as_ptr(),
    c"default link NvimInvalidComparisonModifier NvimInvalidComparison".as_ptr(),
    c"default link NvimInvalidBinaryPlus NvimInvalidBinaryOperator".as_ptr(),
    c"default link NvimInvalidBinaryMinus NvimInvalidBinaryOperator".as_ptr(),
    c"default link NvimInvalidConcat NvimInvalidBinaryOperator".as_ptr(),
    c"default link NvimInvalidConcatOrSubscript NvimInvalidConcat".as_ptr(),
    c"default link NvimInvalidOr NvimInvalidBinaryOperator".as_ptr(),
    c"default link NvimInvalidAnd NvimInvalidBinaryOperator".as_ptr(),
    c"default link NvimInvalidMultiplication NvimInvalidBinaryOperator".as_ptr(),
    c"default link NvimInvalidDivision NvimInvalidBinaryOperator".as_ptr(),
    c"default link NvimInvalidMod NvimInvalidBinaryOperator".as_ptr(),
    c"default link NvimInvalidTernary NvimInvalidOperator".as_ptr(),
    c"default link NvimInvalidTernaryColon NvimInvalidTernary".as_ptr(),
    c"default link NvimInvalidDelimiter NvimInvalid".as_ptr(),
    c"default link NvimInvalidParenthesis NvimInvalidDelimiter".as_ptr(),
    c"default link NvimInvalidLambda NvimInvalidParenthesis".as_ptr(),
    c"default link NvimInvalidNestingParenthesis NvimInvalidParenthesis".as_ptr(),
    c"default link NvimInvalidCallingParenthesis NvimInvalidParenthesis".as_ptr(),
    c"default link NvimInvalidSubscript NvimInvalidParenthesis".as_ptr(),
    c"default link NvimInvalidSubscriptBracket NvimInvalidSubscript".as_ptr(),
    c"default link NvimInvalidSubscriptColon NvimInvalidSubscript".as_ptr(),
    c"default link NvimInvalidCurly NvimInvalidSubscript".as_ptr(),
    c"default link NvimInvalidContainer NvimInvalidParenthesis".as_ptr(),
    c"default link NvimInvalidDict NvimInvalidContainer".as_ptr(),
    c"default link NvimInvalidList NvimInvalidContainer".as_ptr(),
    c"default link NvimInvalidValue NvimInvalid".as_ptr(),
    c"default link NvimInvalidIdentifier NvimInvalidValue".as_ptr(),
    c"default link NvimInvalidIdentifierScope NvimInvalidIdentifier".as_ptr(),
    c"default link NvimInvalidIdentifierScopeDelimiter NvimInvalidIdentifier".as_ptr(),
    c"default link NvimInvalidIdentifierName NvimInvalidIdentifier".as_ptr(),
    c"default link NvimInvalidIdentifierKey NvimInvalidIdentifier".as_ptr(),
    c"default link NvimInvalidColon NvimInvalidDelimiter".as_ptr(),
    c"default link NvimInvalidComma NvimInvalidDelimiter".as_ptr(),
    c"default link NvimInvalidArrow NvimInvalidDelimiter".as_ptr(),
    c"default link NvimInvalidRegister NvimInvalidValue".as_ptr(),
    c"default link NvimInvalidNumber NvimInvalidValue".as_ptr(),
    c"default link NvimInvalidFloat NvimInvalidNumber".as_ptr(),
    c"default link NvimInvalidNumberPrefix NvimInvalidNumber".as_ptr(),
    c"default link NvimInvalidOptionSigil NvimInvalidIdentifier".as_ptr(),
    c"default link NvimInvalidOptionName NvimInvalidIdentifier".as_ptr(),
    c"default link NvimInvalidOptionScope NvimInvalidIdentifierScope".as_ptr(),
    c"default link NvimInvalidOptionScopeDelimiter NvimInvalidIdentifierScopeDelimiter".as_ptr(),
    c"default link NvimInvalidEnvironmentSigil NvimInvalidOptionSigil".as_ptr(),
    c"default link NvimInvalidEnvironmentName NvimInvalidIdentifier".as_ptr(),
    c"default link NvimInvalidString NvimInvalidValue".as_ptr(),
    c"default link NvimInvalidStringBody NvimStringBody".as_ptr(),
    c"default link NvimInvalidStringQuote NvimInvalidString".as_ptr(),
    c"default link NvimInvalidStringSpecial NvimStringSpecial".as_ptr(),
    c"default link NvimInvalidSingleQuote NvimInvalidStringQuote".as_ptr(),
    c"default link NvimInvalidSingleQuotedBody NvimInvalidStringBody".as_ptr(),
    c"default link NvimInvalidSingleQuotedQuote NvimInvalidStringSpecial".as_ptr(),
    c"default link NvimInvalidDoubleQuote NvimInvalidStringQuote".as_ptr(),
    c"default link NvimInvalidDoubleQuotedBody NvimInvalidStringBody".as_ptr(),
    c"default link NvimInvalidDoubleQuotedEscape NvimInvalidStringSpecial".as_ptr(),
    c"default link NvimInvalidDoubleQuotedUnknownEscape NvimInvalidValue".as_ptr(),
    c"default link NvimInvalidFigureBrace NvimInvalidDelimiter".as_ptr(),
    c"default link NvimInvalidSpacing ErrorMsg".as_ptr(),
    c"default link NvimDoubleQuotedUnknownEscape NvimInvalidValue".as_ptr(),
    ::core::ptr::null(),
]);

/// Defines the `Nvim*` command-line groups. `reset`/`init` mean what they do
/// for [`do_highlight`].
///
/// # Safety
/// Runs `:highlight` commands, which redraw; main thread only.
pub unsafe fn syn_init_cmdline_highlight(reset: bool, init: bool) {
    // SAFETY: the table is NUL-terminated and its strings are static.
    unsafe {
        let table = *highlight_init_cmdline.ptr();
        for line in table.iter().take_while(|p| !p.is_null()) {
            do_highlight(*line, reset, init);
        }
    }
}

/// Applies `g:colors_name` if one is set, and otherwise the compiled-in
/// defaults.
///
/// `both` includes the groups `'background'` does not affect;
/// `reset` clears each group first.
///
/// # Safety
/// Sources a colour scheme and runs `:highlight`; main thread only.
pub unsafe fn init_highlight(both: bool, reset: bool) {
    /// Whether the `both == true` call from `main()` has happened. Before it
    /// has, nothing else is set up and its own run would overrule this one
    /// anyway, so a `both == false` call is dropped.
    static HAD_BOTH: GlobalCell<bool> = GlobalCell::new(false);

    // SAFETY: the editor's own state; every callee is a main-thread call.
    unsafe {
        let name = get_var_value(c"g:colors_name".as_ptr());
        if !name.is_null() {
            // `load_colors` can free the variable, and with it `name`.
            let copy = xstrdup(name);
            let okay = load_colors(copy) == OK;
            xfree(copy.cast());
            if okay {
                return;
            }
        }

        if both {
            HAD_BOTH.set(true);
            for line in &HIGHLIGHT_INIT_BOTH {
                do_highlight(line.as_ptr(), reset, true);
            }
        } else if !HAD_BOTH.get() {
            return;
        }

        let table = if *p_bg.get() == b'l' as c_char {
            &HIGHLIGHT_INIT_LIGHT
        } else {
            &HIGHLIGHT_INIT_DARK
        };
        for line in table {
            do_highlight(line.as_ptr(), reset, true);
        }

        syn_init_cmdline_highlight(false, false);
    }
}

/// Sources the colour scheme `name`, answering `OK` or `FAIL`.
///
/// A recursive call answers `OK` without doing anything: it means the scheme
/// being sourced set `'background'`, which reloaded the highlighting, which
/// is proof enough that it is working.
///
/// # Safety
/// Sources a script and fires autocommands; main thread only.
pub unsafe fn load_colors(name: *mut c_char) -> c_int {
    static RECURSIVE: GlobalCell<bool> = GlobalCell::new(false);

    // SAFETY: `name` is the caller's NUL-terminated scheme name.
    unsafe {
        if RECURSIVE.get() {
            return OK;
        }
        RECURSIVE.set(true);

        apply_autocmds(
            EVENT_COLORSCHEMEPRE,
            name,
            (*curbuf.get()).b_fname,
            false,
            curbuf.get(),
        );
        let mut pattern = [b"colors/", CStr::from_ptr(name).to_bytes(), b".*\0"].concat();
        let retval = source_runtime_vim_lua(
            pattern.as_mut_ptr().cast(),
            DIP_START as c_int + DIP_OPT as c_int,
        );
        if retval == OK {
            apply_autocmds(
                EVENT_COLORSCHEME,
                name,
                (*curbuf.get()).b_fname,
                false,
                curbuf.get(),
            );
        }

        RECURSIVE.set(false);
        retval
    }
}

/// Forgets the `Normal` colours, so that the next `:highlight` starts from
/// the terminal's own.
pub fn restore_cterm_colors() {
    normal_fg.set(-1 as RgbValue);
    normal_bg.set(-1 as RgbValue);
    normal_sp.set(-1 as RgbValue);
    cterm_normal_fg_color.set(0);
    cterm_normal_bg_color.set(0);
}
