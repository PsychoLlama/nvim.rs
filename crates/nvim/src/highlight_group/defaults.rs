//! The compiled-in default highlighting, and colour schemes.
//!
//! [`init_highlight`] sources `g:colors_name` if one is set
//! ([`load_colors`]) and otherwise feeds these tables — one `:highlight`
//! command per line — straight to [`do_highlight`]. They are compiled in so
//! that a build with no runtime files still has usable colours.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char};

use crate::autocmd::{EVENT_COLORSCHEME, EVENT_COLORSCHEMEPRE, apply_autocmds};
use crate::eval::vars::get_var_value;
use crate::global_cell::GlobalCell;
use crate::main::{
    cterm_normal_bg_color, cterm_normal_fg_color, curbuf, normal_bg, normal_fg, normal_sp, p_bg,
};
use crate::memory::{xfree, xstrdup};
use crate::runtime::{RuntimeOpts, source_runtime_vim_lua};
use crate::types::{Failed, RgbValue};

use super::do_highlight;
use crate::eval::typval::NumBuf;

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
static HIGHLIGHT_INIT_CMDLINE: [&CStr; 140] = [
    c"NvimInternalError ctermfg=Red ctermbg=Red guifg=Red guibg=Red",
    c"default link NvimAssignment Operator",
    c"default link NvimPlainAssignment NvimAssignment",
    c"default link NvimAugmentedAssignment NvimAssignment",
    c"default link NvimAssignmentWithAddition NvimAugmentedAssignment",
    c"default link NvimAssignmentWithSubtraction NvimAugmentedAssignment",
    c"default link NvimAssignmentWithConcatenation NvimAugmentedAssignment",
    c"default link NvimOperator Operator",
    c"default link NvimUnaryOperator NvimOperator",
    c"default link NvimUnaryPlus NvimUnaryOperator",
    c"default link NvimUnaryMinus NvimUnaryOperator",
    c"default link NvimNot NvimUnaryOperator",
    c"default link NvimBinaryOperator NvimOperator",
    c"default link NvimComparison NvimBinaryOperator",
    c"default link NvimComparisonModifier NvimComparison",
    c"default link NvimBinaryPlus NvimBinaryOperator",
    c"default link NvimBinaryMinus NvimBinaryOperator",
    c"default link NvimConcat NvimBinaryOperator",
    c"default link NvimConcatOrSubscript NvimConcat",
    c"default link NvimOr NvimBinaryOperator",
    c"default link NvimAnd NvimBinaryOperator",
    c"default link NvimMultiplication NvimBinaryOperator",
    c"default link NvimDivision NvimBinaryOperator",
    c"default link NvimMod NvimBinaryOperator",
    c"default link NvimTernary NvimOperator",
    c"default link NvimTernaryColon NvimTernary",
    c"default link NvimParenthesis Delimiter",
    c"default link NvimLambda NvimParenthesis",
    c"default link NvimNestingParenthesis NvimParenthesis",
    c"default link NvimCallingParenthesis NvimParenthesis",
    c"default link NvimSubscript NvimParenthesis",
    c"default link NvimSubscriptBracket NvimSubscript",
    c"default link NvimSubscriptColon NvimSubscript",
    c"default link NvimCurly NvimSubscript",
    c"default link NvimContainer NvimParenthesis",
    c"default link NvimDict NvimContainer",
    c"default link NvimList NvimContainer",
    c"default link NvimIdentifier Identifier",
    c"default link NvimIdentifierScope NvimIdentifier",
    c"default link NvimIdentifierScopeDelimiter NvimIdentifier",
    c"default link NvimIdentifierName NvimIdentifier",
    c"default link NvimIdentifierKey NvimIdentifier",
    c"default link NvimColon Delimiter",
    c"default link NvimComma Delimiter",
    c"default link NvimArrow Delimiter",
    c"default link NvimRegister SpecialChar",
    c"default link NvimNumber Number",
    c"default link NvimFloat NvimNumber",
    c"default link NvimNumberPrefix Type",
    c"default link NvimOptionSigil Type",
    c"default link NvimOptionName NvimIdentifier",
    c"default link NvimOptionScope NvimIdentifierScope",
    c"default link NvimOptionScopeDelimiter NvimIdentifierScopeDelimiter",
    c"default link NvimEnvironmentSigil NvimOptionSigil",
    c"default link NvimEnvironmentName NvimIdentifier",
    c"default link NvimString String",
    c"default link NvimStringBody NvimString",
    c"default link NvimStringQuote NvimString",
    c"default link NvimStringSpecial SpecialChar",
    c"default link NvimSingleQuote NvimStringQuote",
    c"default link NvimSingleQuotedBody NvimStringBody",
    c"default link NvimSingleQuotedQuote NvimStringSpecial",
    c"default link NvimDoubleQuote NvimStringQuote",
    c"default link NvimDoubleQuotedBody NvimStringBody",
    c"default link NvimDoubleQuotedEscape NvimStringSpecial",
    c"default link NvimFigureBrace NvimInternalError",
    c"default link NvimSingleQuotedUnknownEscape NvimInternalError",
    c"default link NvimSpacing Normal",
    c"default link NvimInvalidSingleQuotedUnknownEscape NvimInternalError",
    c"default link NvimInvalid Error",
    c"default link NvimInvalidAssignment NvimInvalid",
    c"default link NvimInvalidPlainAssignment NvimInvalidAssignment",
    c"default link NvimInvalidAugmentedAssignment NvimInvalidAssignment",
    c"default link NvimInvalidAssignmentWithAddition NvimInvalidAugmentedAssignment",
    c"default link NvimInvalidAssignmentWithSubtraction NvimInvalidAugmentedAssignment",
    c"default link NvimInvalidAssignmentWithConcatenation NvimInvalidAugmentedAssignment",
    c"default link NvimInvalidOperator NvimInvalid",
    c"default link NvimInvalidUnaryOperator NvimInvalidOperator",
    c"default link NvimInvalidUnaryPlus NvimInvalidUnaryOperator",
    c"default link NvimInvalidUnaryMinus NvimInvalidUnaryOperator",
    c"default link NvimInvalidNot NvimInvalidUnaryOperator",
    c"default link NvimInvalidBinaryOperator NvimInvalidOperator",
    c"default link NvimInvalidComparison NvimInvalidBinaryOperator",
    c"default link NvimInvalidComparisonModifier NvimInvalidComparison",
    c"default link NvimInvalidBinaryPlus NvimInvalidBinaryOperator",
    c"default link NvimInvalidBinaryMinus NvimInvalidBinaryOperator",
    c"default link NvimInvalidConcat NvimInvalidBinaryOperator",
    c"default link NvimInvalidConcatOrSubscript NvimInvalidConcat",
    c"default link NvimInvalidOr NvimInvalidBinaryOperator",
    c"default link NvimInvalidAnd NvimInvalidBinaryOperator",
    c"default link NvimInvalidMultiplication NvimInvalidBinaryOperator",
    c"default link NvimInvalidDivision NvimInvalidBinaryOperator",
    c"default link NvimInvalidMod NvimInvalidBinaryOperator",
    c"default link NvimInvalidTernary NvimInvalidOperator",
    c"default link NvimInvalidTernaryColon NvimInvalidTernary",
    c"default link NvimInvalidDelimiter NvimInvalid",
    c"default link NvimInvalidParenthesis NvimInvalidDelimiter",
    c"default link NvimInvalidLambda NvimInvalidParenthesis",
    c"default link NvimInvalidNestingParenthesis NvimInvalidParenthesis",
    c"default link NvimInvalidCallingParenthesis NvimInvalidParenthesis",
    c"default link NvimInvalidSubscript NvimInvalidParenthesis",
    c"default link NvimInvalidSubscriptBracket NvimInvalidSubscript",
    c"default link NvimInvalidSubscriptColon NvimInvalidSubscript",
    c"default link NvimInvalidCurly NvimInvalidSubscript",
    c"default link NvimInvalidContainer NvimInvalidParenthesis",
    c"default link NvimInvalidDict NvimInvalidContainer",
    c"default link NvimInvalidList NvimInvalidContainer",
    c"default link NvimInvalidValue NvimInvalid",
    c"default link NvimInvalidIdentifier NvimInvalidValue",
    c"default link NvimInvalidIdentifierScope NvimInvalidIdentifier",
    c"default link NvimInvalidIdentifierScopeDelimiter NvimInvalidIdentifier",
    c"default link NvimInvalidIdentifierName NvimInvalidIdentifier",
    c"default link NvimInvalidIdentifierKey NvimInvalidIdentifier",
    c"default link NvimInvalidColon NvimInvalidDelimiter",
    c"default link NvimInvalidComma NvimInvalidDelimiter",
    c"default link NvimInvalidArrow NvimInvalidDelimiter",
    c"default link NvimInvalidRegister NvimInvalidValue",
    c"default link NvimInvalidNumber NvimInvalidValue",
    c"default link NvimInvalidFloat NvimInvalidNumber",
    c"default link NvimInvalidNumberPrefix NvimInvalidNumber",
    c"default link NvimInvalidOptionSigil NvimInvalidIdentifier",
    c"default link NvimInvalidOptionName NvimInvalidIdentifier",
    c"default link NvimInvalidOptionScope NvimInvalidIdentifierScope",
    c"default link NvimInvalidOptionScopeDelimiter NvimInvalidIdentifierScopeDelimiter",
    c"default link NvimInvalidEnvironmentSigil NvimInvalidOptionSigil",
    c"default link NvimInvalidEnvironmentName NvimInvalidIdentifier",
    c"default link NvimInvalidString NvimInvalidValue",
    c"default link NvimInvalidStringBody NvimStringBody",
    c"default link NvimInvalidStringQuote NvimInvalidString",
    c"default link NvimInvalidStringSpecial NvimStringSpecial",
    c"default link NvimInvalidSingleQuote NvimInvalidStringQuote",
    c"default link NvimInvalidSingleQuotedBody NvimInvalidStringBody",
    c"default link NvimInvalidSingleQuotedQuote NvimInvalidStringSpecial",
    c"default link NvimInvalidDoubleQuote NvimInvalidStringQuote",
    c"default link NvimInvalidDoubleQuotedBody NvimInvalidStringBody",
    c"default link NvimInvalidDoubleQuotedEscape NvimInvalidStringSpecial",
    c"default link NvimInvalidDoubleQuotedUnknownEscape NvimInvalidValue",
    c"default link NvimInvalidFigureBrace NvimInvalidDelimiter",
    c"default link NvimInvalidSpacing ErrorMsg",
    c"default link NvimDoubleQuotedUnknownEscape NvimInvalidValue",
];

/// Defines the `Nvim*` command-line groups. `reset`/`init` mean what they do
/// for [`do_highlight`].
///
/// # Safety
/// Runs `:highlight` commands, which redraw; main thread only.
pub(crate) unsafe fn syn_init_cmdline_highlight(reset: bool, init: bool) {
    for line in &HIGHLIGHT_INIT_CMDLINE {
        // SAFETY: the caller's obligation; the strings are static.
        unsafe { do_highlight(line.as_ptr(), reset, init) };
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
pub(crate) unsafe fn init_highlight(both: bool, reset: bool) {
    let mut numbuf = NumBuf::new();
    /// Whether the `both == true` call from `main()` has happened. Before it
    /// has, nothing else is set up and its own run would overrule this one
    /// anyway, so a `both == false` call is dropped.
    static HAD_BOTH: GlobalCell<bool> = GlobalCell::new(false);

    // SAFETY: the editor's own state; every callee is a main-thread call.
    let name = unsafe { get_var_value(c"g:colors_name".as_ptr(), &mut numbuf) };
    if !name.is_null() {
        // `load_colors` can free the variable, and with it `name`.
        let copy = unsafe { xstrdup(name) };
        let okay = unsafe { load_colors(copy) }.is_ok();
        unsafe { xfree(copy.cast()) };
        if okay {
            return;
        }
    }

    if both {
        HAD_BOTH.set(true);
        for line in &HIGHLIGHT_INIT_BOTH {
            unsafe { do_highlight(line.as_ptr(), reset, true) };
        }
    } else if !HAD_BOTH.get() {
        return;
    }

    let table = if unsafe { *p_bg.get() } == b'l' as c_char {
        &HIGHLIGHT_INIT_LIGHT
    } else {
        &HIGHLIGHT_INIT_DARK
    };
    for line in table {
        unsafe { do_highlight(line.as_ptr(), reset, true) };
    }

    unsafe { syn_init_cmdline_highlight(false, false) };
}

/// Sources the colour scheme `name`, answering `OK` or `FAIL`.
///
/// A recursive call answers `OK` without doing anything: it means the scheme
/// being sourced set `'background'`, which reloaded the highlighting, which
/// is proof enough that it is working.
///
/// # Safety
/// Sources a script and fires autocommands; main thread only.
pub(crate) unsafe fn load_colors(name: *mut c_char) -> Result<(), Failed> {
    static RECURSIVE: GlobalCell<bool> = GlobalCell::new(false);

    // SAFETY: `name` is the caller's NUL-terminated scheme name.
    if RECURSIVE.get() {
        return Ok(());
    }
    RECURSIVE.set(true);

    let buf = curbuf.get();
    // SAFETY: the editor's current buffer.
    let fname = unsafe { (*buf).b_fname };
    unsafe { apply_autocmds(EVENT_COLORSCHEMEPRE, name, fname, false, buf) };
    let mut pattern = [
        b"colors/",
        unsafe { CStr::from_ptr(name) }.to_bytes(),
        b".*\0",
    ]
    .concat();
    let pattern = pattern.as_mut_ptr().cast();
    // SAFETY: a NUL-terminated pattern this frame owns.
    let retval = unsafe { source_runtime_vim_lua(pattern, RuntimeOpts::START | RuntimeOpts::OPT) };
    if retval.is_ok() {
        let buf = curbuf.get();
        // SAFETY: the editor's current buffer.
        let fname = unsafe { (*buf).b_fname };
        unsafe { apply_autocmds(EVENT_COLORSCHEME, name, fname, false, buf) };
    }

    RECURSIVE.set(false);
    retval
}

/// Forgets the `Normal` colours, so that the next `:highlight` starts from
/// the terminal's own.
pub(crate) fn restore_cterm_colors() {
    normal_fg.set(-1 as RgbValue);
    normal_bg.set(-1 as RgbValue);
    normal_sp.set(-1 as RgbValue);
    cterm_normal_fg_color.set(0);
    cterm_normal_bg_color.set(0);
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    /// The group a `:highlight` argument line defines: the new name of a
    /// `default link`, or the first word of a definition.
    fn defines(line: &CStr) -> &str {
        let text = line.to_str().expect("highlight lines are ASCII");
        text.strip_prefix("default link ")
            .unwrap_or(text)
            .split_whitespace()
            .next()
            .expect("a highlight line names a group")
    }

    /// `:highlight default link A B` **creates** `B` as a cleared group when
    /// `B` does not exist yet, which would silently un-highlight everything
    /// linked to it later. So the cmdline table has to be self-consistent in
    /// the order it is applied: every line names a group nothing has defined
    /// before, and every link points at one that is already there — either
    /// from the `'background'` tables above or from an earlier line here.
    ///
    /// This is the check `test/unit/viml/expressions/parser_spec.lua` ran
    /// over the FFI, and the reason `highlight_init_cmdline` was exported.
    /// It compares against the real tables rather than the spec's
    /// hand-maintained list of predefined groups.
    #[test]
    fn every_cmdline_group_is_defined_before_it_is_linked_to() {
        let mut defined: HashSet<&str> = HIGHLIGHT_INIT_BOTH
            .iter()
            .chain(&HIGHLIGHT_INIT_LIGHT)
            .chain(&HIGHLIGHT_INIT_DARK)
            .map(|line| defines(line))
            .collect();

        for (i, line) in HIGHLIGHT_INIT_CMDLINE.iter().enumerate() {
            let text = line.to_str().expect("highlight lines are ASCII");
            let mut words = text.split(' ');
            let group = match text.strip_prefix("default link ") {
                Some(_) => {
                    let (_, _, group, target) = (
                        words.next(),
                        words.next(),
                        words.next().expect("a link names a group"),
                        words.next().expect("a link names a target"),
                    );
                    assert_eq!(words.next(), None, "entry {i} ({text:?}) has extra words");
                    assert!(
                        defined.contains(target),
                        "entry {i} links {group} to {target}, which nothing has \
                         defined yet — the link would create it cleared"
                    );
                    group
                }
                None => {
                    let group = words.next().expect("a definition names a group");
                    assert!(
                        words.next().is_some(),
                        "entry {i} ({text:?}) defines {group} with no arguments"
                    );
                    group
                }
            };
            assert!(
                group.starts_with("Nvim"),
                "entry {i} defines {group}, which is not one of the parser's own groups"
            );
            assert!(defined.insert(group), "entry {i} redefines {group}");
        }
    }
}
