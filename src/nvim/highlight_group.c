// highlight_group.c: code for managing highlight groups

#include <assert.h>
#include <ctype.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/keysets_defs.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/dispatch.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/private/validate.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/cursor_shape.h"
#include "nvim/decoration_provider.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_docmd.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid_defs.h"
#include "nvim/highlight.h"
#include "nvim/highlight_group.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/map_defs.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/time.h"
#include "nvim/runtime.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/vim_defs.h"

// Accessors for Rust FFI
int nvim_get_t_colors(void)
{
  return t_colors;
}

int nvim_get_normal_fg(void)
{
  return normal_fg;
}

int nvim_get_normal_bg(void)
{
  return normal_bg;
}

int nvim_get_normal_sp(void)
{
  return normal_sp;
}

void nvim_set_normal_fg(int val)
{
  normal_fg = val;
}

void nvim_set_normal_bg(int val)
{
  normal_bg = val;
}

void nvim_set_normal_sp(int val)
{
  normal_sp = val;
}

int nvim_get_cterm_normal_fg_color(void)
{
  return cterm_normal_fg_color;
}

int nvim_get_cterm_normal_bg_color(void)
{
  return cterm_normal_bg_color;
}

void nvim_set_cterm_normal_fg_color(int val)
{
  cterm_normal_fg_color = val;
}

void nvim_set_cterm_normal_bg_color(int val)
{
  cterm_normal_bg_color = val;
}

/// Get current window's active highlight namespace
int c_curwin_ns_hl_active(void)
{
  return curwin->w_ns_hl_active;
}

char nvim_get_p_bg(void)
{
  return *p_bg;
}

/// \addtogroup SG_SET
/// @{
enum {
  SG_CTERM = 2,  // cterm has been set
  SG_GUI = 4,    // gui has been set
  SG_LINK = 8,   // link has been set
};
/// @}

#define MAX_SYN_NAME 200

// builtin |highlight-groups|
static garray_T highlight_ga = GA_EMPTY_INIT_VALUE;

// C accessor for highlight_ga.ga_len (used by Rust)
int nvim_get_highlight_ga_len(void) { return highlight_ga.ga_len; }

// arena for object with same lifetime as highlight_ga (aka hl_table)
Arena highlight_arena = ARENA_EMPTY;

Map(cstr_t, int) highlight_unames = MAP_INIT;

/// The "term", "cterm" and "gui" arguments can be any combination of the
/// following names, separated by commas (but no spaces!).
static char *(hl_name_table[]) =
{ "bold", "standout", "underline",
  "undercurl", "underdouble", "underdotted", "underdashed",
  "italic", "reverse", "inverse", "strikethrough", "altfont",
  "nocombine", "NONE" };
static int hl_attr_table[] =
{ HL_BOLD, HL_STANDOUT, HL_UNDERLINE,
  HL_UNDERCURL, HL_UNDERDOUBLE, HL_UNDERDOTTED, HL_UNDERDASHED,
  HL_ITALIC, HL_INVERSE, HL_INVERSE, HL_STRIKETHROUGH, HL_ALTFONT,
  HL_NOCOMBINE, 0 };

/// Structure that stores information about a highlight group.
/// The ID of a highlight group is also called group ID.  It is the index in
/// the highlight_ga array PLUS ONE.
typedef struct {
  char *sg_name;                ///< highlight group name
  char *sg_name_u;              ///< uppercase of sg_name
  bool sg_cleared;              ///< "hi clear" was used
  int sg_attr;                  ///< Screen attr @see ATTR_ENTRY
  int sg_link;                  ///< link to this highlight group ID
  int sg_deflink;               ///< default link; restored in highlight_clear()
  int sg_set;                   ///< combination of flags in \ref SG_SET
  sctx_T sg_deflink_sctx;       ///< script where the default link was set
  sctx_T sg_script_ctx;         ///< script in which the group was last set for terminal UIs
  int sg_cterm;                 ///< "cterm=" highlighting attr
                                ///< (combination of \ref HlAttrFlags)
  int sg_cterm_fg;              ///< terminal fg color number + 1
  int sg_cterm_bg;              ///< terminal bg color number + 1
  bool sg_cterm_bold;           ///< bold attr was set for light color for RGB UIs
  int sg_gui;                   ///< "gui=" highlighting attributes
                                ///< (combination of \ref HlAttrFlags)
  RgbValue sg_rgb_fg;           ///< RGB foreground color
  RgbValue sg_rgb_bg;           ///< RGB background color
  RgbValue sg_rgb_sp;           ///< RGB special color
  int sg_rgb_fg_idx;            ///< RGB foreground color index
  int sg_rgb_bg_idx;            ///< RGB background color index
  int sg_rgb_sp_idx;            ///< RGB special color index

  int sg_blend;                 ///< blend level (0-100 inclusive), -1 if unset

  int sg_parent;                ///< parent of @nested.group
} HlGroup;

// C accessors for hl_table fields (used by Rust)
int nvim_hl_table_get_sg_gui(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_gui; }
int nvim_hl_table_get_sg_cterm(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_cterm; }

// Additional C accessors for Rust FFI
char *nvim_hl_table_get_sg_name(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_name; }
char *nvim_hl_table_get_sg_name_u(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_name_u; }
bool nvim_hl_table_get_sg_cleared(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_cleared; }
int nvim_hl_table_get_sg_attr(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_attr; }
int nvim_hl_table_get_sg_link(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_link; }
int nvim_hl_table_get_sg_deflink(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_deflink; }
int nvim_hl_table_get_sg_set(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_set; }
int nvim_hl_table_get_sg_cterm_fg(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_cterm_fg; }
int nvim_hl_table_get_sg_cterm_bg(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_cterm_bg; }
bool nvim_hl_table_get_sg_cterm_bold(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_cterm_bold; }
RgbValue nvim_hl_table_get_sg_rgb_fg(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_rgb_fg; }
RgbValue nvim_hl_table_get_sg_rgb_bg(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_rgb_bg; }
RgbValue nvim_hl_table_get_sg_rgb_sp(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_rgb_sp; }
int nvim_hl_table_get_sg_rgb_fg_idx(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_rgb_fg_idx; }
int nvim_hl_table_get_sg_rgb_bg_idx(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_rgb_bg_idx; }
int nvim_hl_table_get_sg_rgb_sp_idx(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_rgb_sp_idx; }
int nvim_hl_table_get_sg_blend(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_blend; }
int nvim_hl_table_get_sg_parent(int idx) { return ((HlGroup *)highlight_ga.ga_data)[idx].sg_parent; }

// Setters for hl_table fields (used by Rust)
void nvim_hl_table_set_sg_cleared(int idx, bool val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_cleared = val; }
void nvim_hl_table_set_sg_attr(int idx, int val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_attr = val; }
void nvim_hl_table_set_sg_link(int idx, int val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_link = val; }
void nvim_hl_table_set_sg_deflink(int idx, int val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_deflink = val; }
void nvim_hl_table_set_sg_set(int idx, int val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_set = val; }
void nvim_hl_table_set_sg_cterm(int idx, int val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_cterm = val; }
void nvim_hl_table_set_sg_cterm_fg(int idx, int val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_cterm_fg = val; }
void nvim_hl_table_set_sg_cterm_bg(int idx, int val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_cterm_bg = val; }
void nvim_hl_table_set_sg_cterm_bold(int idx, bool val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_cterm_bold = val; }
void nvim_hl_table_set_sg_gui(int idx, int val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_gui = val; }
void nvim_hl_table_set_sg_rgb_fg(int idx, RgbValue val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_rgb_fg = val; }
void nvim_hl_table_set_sg_rgb_bg(int idx, RgbValue val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_rgb_bg = val; }
void nvim_hl_table_set_sg_rgb_sp(int idx, RgbValue val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_rgb_sp = val; }
void nvim_hl_table_set_sg_rgb_fg_idx(int idx, int val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_rgb_fg_idx = val; }
void nvim_hl_table_set_sg_rgb_bg_idx(int idx, int val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_rgb_bg_idx = val; }
void nvim_hl_table_set_sg_rgb_sp_idx(int idx, int val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_rgb_sp_idx = val; }
void nvim_hl_table_set_sg_blend(int idx, int val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_blend = val; }
void nvim_hl_table_set_sg_parent(int idx, int val) { ((HlGroup *)highlight_ga.ga_data)[idx].sg_parent = val; }

enum {
  kColorIdxNone = -1,
  kColorIdxHex = -2,
  kColorIdxFg = -3,
  kColorIdxBg = -4,
};

#include "highlight_group.c.generated.h"

static const char e_highlight_group_name_not_found_str[]
  = N_("E411: Highlight group not found: %s");
static const char e_group_has_settings_highlight_link_ignored[]
  = N_("E414: Group has settings, highlight link ignored");
static const char e_unexpected_equal_sign_str[]
  = N_("E415: Unexpected equal sign: %s");
static const char e_missing_equal_sign_str_2[]
  = N_("E416: Missing equal sign: %s");
static const char e_missing_argument_str[]
  = N_("E417: Missing argument: %s");

#define hl_table ((HlGroup *)((highlight_ga.ga_data)))

// The default highlight groups.  These are compiled-in for fast startup and
// they still work when the runtime files can't be found.

static const char *highlight_init_both[] = {
  "Cursor            guifg=bg      guibg=fg",
  "CursorLineNr      gui=bold      cterm=bold",
  "PmenuMatch        gui=bold      cterm=bold",
  "PmenuMatchSel     gui=bold      cterm=bold",
  "PmenuSel          gui=reverse   cterm=reverse,underline blend=0",
  "RedrawDebugNormal gui=reverse   cterm=reverse",
  "TabLineSel        gui=bold      cterm=NONE",
  "TermCursor        gui=reverse   cterm=reverse",
  "Underlined        gui=underline cterm=underline",
  "lCursor           guifg=bg      guibg=fg",

  // UI
  "default link CursorIM         Cursor",
  "default link CursorLineFold   FoldColumn",
  "default link CursorLineSign   SignColumn",
  "default link DiffTextAdd      DiffText",
  "default link EndOfBuffer      NonText",
  "default link FloatBorder      NormalFloat",
  "default link FloatFooter      FloatTitle",
  "default link FloatTitle       Title",
  "default link FoldColumn       SignColumn",
  "default link IncSearch        CurSearch",
  "default link LineNrAbove      LineNr",
  "default link LineNrBelow      LineNr",
  "default link MsgSeparator     StatusLine",
  "default link MsgArea          NONE",
  "default link NormalNC         NONE",
  "default link PmenuExtra       Pmenu",
  "default link PmenuExtraSel    PmenuSel",
  "default link PmenuKind        Pmenu",
  "default link PmenuKindSel     PmenuSel",
  "default link PmenuSbar        Pmenu",
  "default link PmenuBorder        Pmenu",
  "default link PmenuShadowThrough FloatShadowThrough",
  "default link PreInsert        Added",
  "default link ComplMatchIns    NONE",
  "default link ComplHint        NonText",
  "default link ComplHintMore    MoreMsg",
  "default link Substitute       Search",
  "default link StatusLineTerm   StatusLine",
  "default link StatusLineTermNC StatusLineNC",
  "default link StderrMsg        ErrorMsg",
  "default link StdoutMsg        NONE",
  "default link TabLine          StatusLineNC",
  "default link TabLineFill      TabLine",
  "default link VertSplit        WinSeparator",
  "default link VisualNOS        Visual",
  "default link Whitespace       NonText",
  "default link WildMenu         PmenuSel",
  "default link WinSeparator     Normal",

  // Syntax
  "default link Character      Constant",
  "default link Number         Constant",
  "default link Boolean        Constant",
  "default link Float          Number",
  "default link Conditional    Statement",
  "default link Repeat         Statement",
  "default link Label          Statement",
  "default link Keyword        Statement",
  "default link Exception      Statement",
  "default link Include        PreProc",
  "default link Define         PreProc",
  "default link Macro          PreProc",
  "default link PreCondit      PreProc",
  "default link StorageClass   Type",
  "default link Structure      Type",
  "default link Typedef        Type",
  "default link Tag            Special",
  "default link SpecialChar    Special",
  "default link SpecialComment Special",
  "default link Debug          Special",
  "default link Ignore         Normal",

  // Built-in LSP
  "default link LspCodeLens                 NonText",
  "default link LspCodeLensSeparator        LspCodeLens",
  "default link LspInlayHint                NonText",
  "default link LspReferenceRead            LspReferenceText",
  "default link LspReferenceText            Visual",
  "default link LspReferenceWrite           LspReferenceText",
  "default link LspReferenceTarget          LspReferenceText",
  "default link LspSignatureActiveParameter Visual",
  "default link SnippetTabstop              Visual",
  "default link SnippetTabstopActive        SnippetTabstop",

  // Diagnostic
  "default link DiagnosticFloatingError    DiagnosticError",
  "default link DiagnosticFloatingWarn     DiagnosticWarn",
  "default link DiagnosticFloatingInfo     DiagnosticInfo",
  "default link DiagnosticFloatingHint     DiagnosticHint",
  "default link DiagnosticFloatingOk       DiagnosticOk",
  "default link DiagnosticVirtualTextError DiagnosticError",
  "default link DiagnosticVirtualTextWarn  DiagnosticWarn",
  "default link DiagnosticVirtualTextInfo  DiagnosticInfo",
  "default link DiagnosticVirtualTextHint  DiagnosticHint",
  "default link DiagnosticVirtualTextOk    DiagnosticOk",
  "default link DiagnosticVirtualLinesError DiagnosticError",
  "default link DiagnosticVirtualLinesWarn  DiagnosticWarn",
  "default link DiagnosticVirtualLinesInfo  DiagnosticInfo",
  "default link DiagnosticVirtualLinesHint  DiagnosticHint",
  "default link DiagnosticVirtualLinesOk    DiagnosticOk",
  "default link DiagnosticSignError        DiagnosticError",
  "default link DiagnosticSignWarn         DiagnosticWarn",
  "default link DiagnosticSignInfo         DiagnosticInfo",
  "default link DiagnosticSignHint         DiagnosticHint",
  "default link DiagnosticSignOk           DiagnosticOk",
  "default link DiagnosticUnnecessary      Comment",

  // Treesitter standard groups
  "default link @variable.builtin           Special",
  "default link @variable.parameter.builtin Special",

  "default link @constant         Constant",
  "default link @constant.builtin Special",

  "default link @module         Structure",
  "default link @module.builtin Special",
  "default link @label          Label",

  "default link @string             String",
  "default link @string.regexp      @string.special",
  "default link @string.escape      @string.special",
  "default link @string.special     SpecialChar",
  "default link @string.special.url Underlined",

  "default link @character         Character",
  "default link @character.special SpecialChar",

  "default link @boolean      Boolean",
  "default link @number       Number",
  "default link @number.float Float",

  "default link @type         Type",
  "default link @type.builtin Special",

  "default link @attribute         Macro",
  "default link @attribute.builtin Special",
  "default link @property          Identifier",

  "default link @function         Function",
  "default link @function.builtin Special",

  "default link @constructor Special",
  "default link @operator    Operator",

  "default link @keyword Keyword",

  "default link @punctuation         Delimiter",  // fallback for subgroups; never used itself
  "default link @punctuation.special Special",

  "default link @comment Comment",

  "default link @comment.error   DiagnosticError",
  "default link @comment.warning DiagnosticWarn",
  "default link @comment.note    DiagnosticInfo",
  "default link @comment.todo    Todo",

  "@markup.strong        gui=bold          cterm=bold",
  "@markup.italic        gui=italic        cterm=italic",
  "@markup.strikethrough gui=strikethrough cterm=strikethrough",
  "@markup.underline     gui=underline     cterm=underline",

  "default link @markup         Special",  // fallback for subgroups; never used itself
  "default link @markup.heading Title",
  "default link @markup.link    Underlined",

  "default link @diff.plus  Added",
  "default link @diff.minus Removed",
  "default link @diff.delta Changed",

  "default link @tag         Tag",
  "default link @tag.builtin Special",

  // :help
  // Highlight "===" and "---" heading delimiters specially.
  "default @markup.heading.1.delimiter.vimdoc guibg=bg guifg=bg guisp=fg gui=underdouble,nocombine ctermbg=NONE ctermfg=NONE cterm=underdouble,nocombine",
  "default @markup.heading.2.delimiter.vimdoc guibg=bg guifg=bg guisp=fg gui=underline,nocombine ctermbg=NONE ctermfg=NONE cterm=underline,nocombine",

  // LSP semantic tokens
  "default link @lsp.type.class         @type",
  "default link @lsp.type.comment       @comment",
  "default link @lsp.type.decorator     @attribute",
  "default link @lsp.type.enum          @type",
  "default link @lsp.type.enumMember    @constant",
  "default link @lsp.type.event         @type",
  "default link @lsp.type.function      @function",
  "default link @lsp.type.interface     @type",
  "default link @lsp.type.keyword       @keyword",
  "default link @lsp.type.macro         @constant.macro",
  "default link @lsp.type.method        @function.method",
  "default link @lsp.type.modifier      @type.qualifier",
  "default link @lsp.type.namespace     @module",
  "default link @lsp.type.number        @number",
  "default link @lsp.type.operator      @operator",
  "default link @lsp.type.parameter     @variable.parameter",
  "default link @lsp.type.property      @property",
  "default link @lsp.type.regexp        @string.regexp",
  "default link @lsp.type.string        @string",
  "default link @lsp.type.struct        @type",
  "default link @lsp.type.type          @type",
  "default link @lsp.type.typeParameter @type.definition",
  "default link @lsp.type.variable      @variable",

  "default link @lsp.mod.deprecated DiagnosticDeprecated",

  NULL
};

// Default colors only used with a light background.
static const char *highlight_init_light[] = {
  "Normal guifg=NvimDarkGrey2 guibg=NvimLightGrey2 ctermfg=NONE ctermbg=NONE",

  // UI
  "Added                guifg=NvimDarkGreen                                  ctermfg=2",
  "Changed              guifg=NvimDarkCyan                                   ctermfg=6",
  "ColorColumn                               guibg=NvimLightGrey4            cterm=reverse",
  "Conceal              guifg=NvimLightGrey4",
  "CurSearch            guifg=NvimLightGrey1 guibg=NvimDarkYellow            ctermfg=15 ctermbg=3",
  "CursorColumn                              guibg=NvimLightGrey3",
  "CursorLine                                guibg=NvimLightGrey3",
  "DiffAdd              guifg=NvimDarkGrey1  guibg=NvimLightGreen            ctermfg=15 ctermbg=2",
  "DiffChange           guifg=NvimDarkGrey1  guibg=NvimLightGrey4",
  "DiffDelete           guifg=NvimDarkRed                          gui=bold  ctermfg=1 cterm=bold",
  "DiffText             guifg=NvimDarkGrey1  guibg=NvimLightCyan             ctermfg=15 ctermbg=6",
  "Directory            guifg=NvimDarkCyan                                   ctermfg=6",
  "ErrorMsg             guifg=NvimDarkRed                                    ctermfg=1",
  "FloatShadow                               guibg=NvimLightGrey4            ctermbg=0 blend=80",
  "FloatShadowThrough                        guibg=NvimLightGrey4            ctermbg=0 blend=100",
  "Folded               guifg=NvimDarkGrey4  guibg=NvimLightGrey1",
  "LineNr               guifg=NvimLightGrey4",
  "MatchParen                                guibg=NvimLightGrey4  gui=bold  cterm=bold,underline",
  "ModeMsg              guifg=NvimDarkGreen                                  ctermfg=2",
  "MoreMsg              guifg=NvimDarkCyan                                   ctermfg=6",
  "NonText              guifg=NvimLightGrey4",
  "NormalFloat                               guibg=NvimLightGrey1",
  "OkMsg                guifg=NvimDarkGreen                                  ctermfg=2",
  "Pmenu                                     guibg=NvimLightGrey3            cterm=reverse",
  "PmenuThumb                                guibg=NvimLightGrey4",
  "PmenuShadow                               guibg=NvimLightGrey4            ctermbg=0 blend=100",
  "Question             guifg=NvimDarkCyan                                   ctermfg=6",
  "QuickFixLine         guifg=NvimDarkCyan                                   ctermfg=6",
  "RedrawDebugClear                          guibg=NvimLightYellow           ctermfg=15 ctermbg=3",
  "RedrawDebugComposed                       guibg=NvimLightGreen            ctermfg=15 ctermbg=2",
  "RedrawDebugRecompose                      guibg=NvimLightRed              ctermfg=15 ctermbg=1",
  "Removed              guifg=NvimDarkRed                                    ctermfg=1",
  "Search               guifg=NvimDarkGrey1  guibg=NvimLightYellow           ctermfg=15 ctermbg=3",
  "SignColumn           guifg=NvimLightGrey4",
  "SpecialKey           guifg=NvimLightGrey4",
  "SpellBad             guisp=NvimDarkRed    gui=undercurl                   cterm=undercurl",
  "SpellCap             guisp=NvimDarkYellow gui=undercurl                   cterm=undercurl",
  "SpellLocal           guisp=NvimDarkGreen  gui=undercurl                   cterm=undercurl",
  "SpellRare            guisp=NvimDarkCyan   gui=undercurl                   cterm=undercurl",
  "StatusLine           guifg=NvimDarkGrey2  guibg=NvimLightGrey4            cterm=reverse",
  "StatusLineNC         guifg=NvimDarkGrey3  guibg=NvimLightGrey3            cterm=bold,underline",
  "Title                guifg=NvimDarkGrey2                        gui=bold  cterm=bold",
  "Visual                                    guibg=NvimLightGrey4            ctermfg=15 ctermbg=0",
  "WarningMsg           guifg=NvimDarkYellow                                 ctermfg=3",
  "WinBar               guifg=NvimDarkGrey4  guibg=NvimLightGrey1  gui=bold  cterm=bold",
  "WinBarNC             guifg=NvimDarkGrey4  guibg=NvimLightGrey1            cterm=bold",

  // Syntax
  "Constant   guifg=NvimDarkGrey2",  // Use only `Normal` foreground to be usable on different background
  "Operator   guifg=NvimDarkGrey2",
  "PreProc    guifg=NvimDarkGrey2",
  "Type       guifg=NvimDarkGrey2",
  "Delimiter  guifg=NvimDarkGrey2",

  "Comment    guifg=NvimDarkGrey4",
  "String     guifg=NvimDarkGreen                    ctermfg=2",
  "Identifier guifg=NvimDarkBlue                     ctermfg=4",
  "Function   guifg=NvimDarkCyan                     ctermfg=6",
  "Statement  guifg=NvimDarkGrey2 gui=bold           cterm=bold",
  "Special    guifg=NvimDarkCyan                     ctermfg=6",
  "Error      guifg=NvimDarkGrey1 guibg=NvimLightRed ctermfg=15 ctermbg=1",
  "Todo       guifg=NvimDarkGrey2 gui=bold           cterm=bold",

  // Diagnostic
  "DiagnosticError          guifg=NvimDarkRed                      ctermfg=1",
  "DiagnosticWarn           guifg=NvimDarkYellow                   ctermfg=3",
  "DiagnosticInfo           guifg=NvimDarkCyan                     ctermfg=6",
  "DiagnosticHint           guifg=NvimDarkBlue                     ctermfg=4",
  "DiagnosticOk             guifg=NvimDarkGreen                    ctermfg=2",
  "DiagnosticUnderlineError guisp=NvimDarkRed    gui=underline     cterm=underline",
  "DiagnosticUnderlineWarn  guisp=NvimDarkYellow gui=underline     cterm=underline",
  "DiagnosticUnderlineInfo  guisp=NvimDarkCyan   gui=underline     cterm=underline",
  "DiagnosticUnderlineHint  guisp=NvimDarkBlue   gui=underline     cterm=underline",
  "DiagnosticUnderlineOk    guisp=NvimDarkGreen  gui=underline     cterm=underline",
  "DiagnosticDeprecated     guisp=NvimDarkRed    gui=strikethrough cterm=strikethrough",

  // Treesitter standard groups
  "@variable guifg=NvimDarkGrey2",
  NULL
};

// Default colors only used with a dark background.
static const char *highlight_init_dark[] = {
  "Normal guifg=NvimLightGrey2 guibg=NvimDarkGrey2 ctermfg=NONE ctermbg=NONE",

  // UI
  "Added                guifg=NvimLightGreen                                ctermfg=10",
  "Changed              guifg=NvimLightCyan                                 ctermfg=14",
  "ColorColumn                                guibg=NvimDarkGrey4           cterm=reverse",
  "Conceal              guifg=NvimDarkGrey4",
  "CurSearch            guifg=NvimDarkGrey1   guibg=NvimLightYellow         ctermfg=0 ctermbg=11",
  "CursorColumn                               guibg=NvimDarkGrey3",
  "CursorLine                                 guibg=NvimDarkGrey3",
  "DiffAdd              guifg=NvimLightGrey1  guibg=NvimDarkGreen           ctermfg=0 ctermbg=10",
  "DiffChange           guifg=NvimLightGrey1  guibg=NvimDarkGrey4",
  "DiffDelete           guifg=NvimLightRed                         gui=bold ctermfg=9 cterm=bold",
  "DiffText             guifg=NvimLightGrey1  guibg=NvimDarkCyan            ctermfg=0 ctermbg=14",
  "Directory            guifg=NvimLightCyan                                 ctermfg=14",
  "ErrorMsg             guifg=NvimLightRed                                  ctermfg=9",
  "FloatShadow                                guibg=NvimDarkGrey4           ctermbg=0 blend=80",
  "FloatShadowThrough                         guibg=NvimDarkGrey4           ctermbg=0 blend=100",
  "Folded               guifg=NvimLightGrey4  guibg=NvimDarkGrey1",
  "LineNr               guifg=NvimDarkGrey4",
  "MatchParen                                 guibg=NvimDarkGrey4  gui=bold cterm=bold,underline",
  "ModeMsg              guifg=NvimLightGreen                                ctermfg=10",
  "MoreMsg              guifg=NvimLightCyan                                 ctermfg=14",
  "NonText              guifg=NvimDarkGrey4",
  "NormalFloat                                guibg=NvimDarkGrey1",
  "OkMsg                guifg=NvimLightGreen                                ctermfg=10",
  "Pmenu                                      guibg=NvimDarkGrey3           cterm=reverse",
  "PmenuThumb                                 guibg=NvimDarkGrey4",
  "PmenuShadow                                guibg=NvimDarkGrey4           ctermbg=0 blend=100",
  "Question             guifg=NvimLightCyan                                 ctermfg=14",
  "QuickFixLine         guifg=NvimLightCyan                                 ctermfg=14",
  "RedrawDebugClear                           guibg=NvimDarkYellow          ctermfg=0 ctermbg=11",
  "RedrawDebugComposed                        guibg=NvimDarkGreen           ctermfg=0 ctermbg=10",
  "RedrawDebugRecompose                       guibg=NvimDarkRed             ctermfg=0 ctermbg=9",
  "Removed              guifg=NvimLightRed                                  ctermfg=9",
  "Search               guifg=NvimLightGrey1  guibg=NvimDarkYellow          ctermfg=0 ctermbg=11",
  "SignColumn           guifg=NvimDarkGrey4",
  "SpecialKey           guifg=NvimDarkGrey4",
  "SpellBad             guisp=NvimLightRed    gui=undercurl                 cterm=undercurl",
  "SpellCap             guisp=NvimLightYellow gui=undercurl                 cterm=undercurl",
  "SpellLocal           guisp=NvimLightGreen  gui=undercurl                 cterm=undercurl",
  "SpellRare            guisp=NvimLightCyan   gui=undercurl                 cterm=undercurl",
  "StatusLine           guifg=NvimLightGrey2  guibg=NvimDarkGrey4           cterm=reverse",
  "StatusLineNC         guifg=NvimLightGrey3  guibg=NvimDarkGrey3           cterm=bold,underline",
  "Title                guifg=NvimLightGrey2                       gui=bold cterm=bold",
  "Visual                                     guibg=NvimDarkGrey4           ctermfg=0 ctermbg=15",
  "WarningMsg           guifg=NvimLightYellow                               ctermfg=11",
  "WinBar               guifg=NvimLightGrey4  guibg=NvimDarkGrey1  gui=bold cterm=bold",
  "WinBarNC             guifg=NvimLightGrey4  guibg=NvimDarkGrey1           cterm=bold",

  // Syntax
  "Constant   guifg=NvimLightGrey2",  // Use only `Normal` foreground to be usable on different background
  "Operator   guifg=NvimLightGrey2",
  "PreProc    guifg=NvimLightGrey2",
  "Type       guifg=NvimLightGrey2",
  "Delimiter  guifg=NvimLightGrey2",

  "Comment    guifg=NvimLightGrey4",
  "String     guifg=NvimLightGreen                   ctermfg=10",
  "Identifier guifg=NvimLightBlue                    ctermfg=12",
  "Function   guifg=NvimLightCyan                    ctermfg=14",
  "Statement  guifg=NvimLightGrey2 gui=bold          cterm=bold",
  "Special    guifg=NvimLightCyan                    ctermfg=14",
  "Error      guifg=NvimLightGrey1 guibg=NvimDarkRed ctermfg=0 ctermbg=9",
  "Todo       guifg=NvimLightGrey2 gui=bold          cterm=bold",

  // Diagnostic
  "DiagnosticError          guifg=NvimLightRed                      ctermfg=9",
  "DiagnosticWarn           guifg=NvimLightYellow                   ctermfg=11",
  "DiagnosticInfo           guifg=NvimLightCyan                     ctermfg=14",
  "DiagnosticHint           guifg=NvimLightBlue                     ctermfg=12",
  "DiagnosticOk             guifg=NvimLightGreen                    ctermfg=10",
  "DiagnosticUnderlineError guisp=NvimLightRed    gui=underline     cterm=underline",
  "DiagnosticUnderlineWarn  guisp=NvimLightYellow gui=underline     cterm=underline",
  "DiagnosticUnderlineInfo  guisp=NvimLightCyan   gui=underline     cterm=underline",
  "DiagnosticUnderlineHint  guisp=NvimLightBlue   gui=underline     cterm=underline",
  "DiagnosticUnderlineOk    guisp=NvimLightGreen  gui=underline     cterm=underline",
  "DiagnosticDeprecated     guisp=NvimLightRed    gui=strikethrough cterm=strikethrough",

  // Treesitter standard groups
  "@variable guifg=NvimLightGrey2",
  NULL
};

const char *const highlight_init_cmdline[] = {
  // XXX When modifying a list modify it in both valid and invalid halves.
  // TODO(ZyX-I): merge valid and invalid groups via a macros.

  // NvimInternalError should appear only when highlighter has a bug.
  "NvimInternalError ctermfg=Red ctermbg=Red guifg=Red guibg=Red",

  // Highlight groups (links) used by parser:

  "default link NvimAssignment Operator",
  "default link NvimPlainAssignment NvimAssignment",
  "default link NvimAugmentedAssignment NvimAssignment",
  "default link NvimAssignmentWithAddition NvimAugmentedAssignment",
  "default link NvimAssignmentWithSubtraction NvimAugmentedAssignment",
  "default link NvimAssignmentWithConcatenation NvimAugmentedAssignment",

  "default link NvimOperator Operator",

  "default link NvimUnaryOperator NvimOperator",
  "default link NvimUnaryPlus NvimUnaryOperator",
  "default link NvimUnaryMinus NvimUnaryOperator",
  "default link NvimNot NvimUnaryOperator",

  "default link NvimBinaryOperator NvimOperator",
  "default link NvimComparison NvimBinaryOperator",
  "default link NvimComparisonModifier NvimComparison",
  "default link NvimBinaryPlus NvimBinaryOperator",
  "default link NvimBinaryMinus NvimBinaryOperator",
  "default link NvimConcat NvimBinaryOperator",
  "default link NvimConcatOrSubscript NvimConcat",
  "default link NvimOr NvimBinaryOperator",
  "default link NvimAnd NvimBinaryOperator",
  "default link NvimMultiplication NvimBinaryOperator",
  "default link NvimDivision NvimBinaryOperator",
  "default link NvimMod NvimBinaryOperator",

  "default link NvimTernary NvimOperator",
  "default link NvimTernaryColon NvimTernary",

  "default link NvimParenthesis Delimiter",
  "default link NvimLambda NvimParenthesis",
  "default link NvimNestingParenthesis NvimParenthesis",
  "default link NvimCallingParenthesis NvimParenthesis",

  "default link NvimSubscript NvimParenthesis",
  "default link NvimSubscriptBracket NvimSubscript",
  "default link NvimSubscriptColon NvimSubscript",
  "default link NvimCurly NvimSubscript",

  "default link NvimContainer NvimParenthesis",
  "default link NvimDict NvimContainer",
  "default link NvimList NvimContainer",

  "default link NvimIdentifier Identifier",
  "default link NvimIdentifierScope NvimIdentifier",
  "default link NvimIdentifierScopeDelimiter NvimIdentifier",
  "default link NvimIdentifierName NvimIdentifier",
  "default link NvimIdentifierKey NvimIdentifier",

  "default link NvimColon Delimiter",
  "default link NvimComma Delimiter",
  "default link NvimArrow Delimiter",

  "default link NvimRegister SpecialChar",
  "default link NvimNumber Number",
  "default link NvimFloat NvimNumber",
  "default link NvimNumberPrefix Type",

  "default link NvimOptionSigil Type",
  "default link NvimOptionName NvimIdentifier",
  "default link NvimOptionScope NvimIdentifierScope",
  "default link NvimOptionScopeDelimiter NvimIdentifierScopeDelimiter",

  "default link NvimEnvironmentSigil NvimOptionSigil",
  "default link NvimEnvironmentName NvimIdentifier",

  "default link NvimString String",
  "default link NvimStringBody NvimString",
  "default link NvimStringQuote NvimString",
  "default link NvimStringSpecial SpecialChar",

  "default link NvimSingleQuote NvimStringQuote",
  "default link NvimSingleQuotedBody NvimStringBody",
  "default link NvimSingleQuotedQuote NvimStringSpecial",

  "default link NvimDoubleQuote NvimStringQuote",
  "default link NvimDoubleQuotedBody NvimStringBody",
  "default link NvimDoubleQuotedEscape NvimStringSpecial",

  "default link NvimFigureBrace NvimInternalError",
  "default link NvimSingleQuotedUnknownEscape NvimInternalError",

  "default link NvimSpacing Normal",

  // NvimInvalid groups:

  "default link NvimInvalidSingleQuotedUnknownEscape NvimInternalError",

  "default link NvimInvalid Error",

  "default link NvimInvalidAssignment NvimInvalid",
  "default link NvimInvalidPlainAssignment NvimInvalidAssignment",
  "default link NvimInvalidAugmentedAssignment NvimInvalidAssignment",
  "default link NvimInvalidAssignmentWithAddition NvimInvalidAugmentedAssignment",
  "default link NvimInvalidAssignmentWithSubtraction NvimInvalidAugmentedAssignment",
  "default link NvimInvalidAssignmentWithConcatenation NvimInvalidAugmentedAssignment",

  "default link NvimInvalidOperator NvimInvalid",

  "default link NvimInvalidUnaryOperator NvimInvalidOperator",
  "default link NvimInvalidUnaryPlus NvimInvalidUnaryOperator",
  "default link NvimInvalidUnaryMinus NvimInvalidUnaryOperator",
  "default link NvimInvalidNot NvimInvalidUnaryOperator",

  "default link NvimInvalidBinaryOperator NvimInvalidOperator",
  "default link NvimInvalidComparison NvimInvalidBinaryOperator",
  "default link NvimInvalidComparisonModifier NvimInvalidComparison",
  "default link NvimInvalidBinaryPlus NvimInvalidBinaryOperator",
  "default link NvimInvalidBinaryMinus NvimInvalidBinaryOperator",
  "default link NvimInvalidConcat NvimInvalidBinaryOperator",
  "default link NvimInvalidConcatOrSubscript NvimInvalidConcat",
  "default link NvimInvalidOr NvimInvalidBinaryOperator",
  "default link NvimInvalidAnd NvimInvalidBinaryOperator",
  "default link NvimInvalidMultiplication NvimInvalidBinaryOperator",
  "default link NvimInvalidDivision NvimInvalidBinaryOperator",
  "default link NvimInvalidMod NvimInvalidBinaryOperator",

  "default link NvimInvalidTernary NvimInvalidOperator",
  "default link NvimInvalidTernaryColon NvimInvalidTernary",

  "default link NvimInvalidDelimiter NvimInvalid",

  "default link NvimInvalidParenthesis NvimInvalidDelimiter",
  "default link NvimInvalidLambda NvimInvalidParenthesis",
  "default link NvimInvalidNestingParenthesis NvimInvalidParenthesis",
  "default link NvimInvalidCallingParenthesis NvimInvalidParenthesis",

  "default link NvimInvalidSubscript NvimInvalidParenthesis",
  "default link NvimInvalidSubscriptBracket NvimInvalidSubscript",
  "default link NvimInvalidSubscriptColon NvimInvalidSubscript",
  "default link NvimInvalidCurly NvimInvalidSubscript",

  "default link NvimInvalidContainer NvimInvalidParenthesis",
  "default link NvimInvalidDict NvimInvalidContainer",
  "default link NvimInvalidList NvimInvalidContainer",

  "default link NvimInvalidValue NvimInvalid",

  "default link NvimInvalidIdentifier NvimInvalidValue",
  "default link NvimInvalidIdentifierScope NvimInvalidIdentifier",
  "default link NvimInvalidIdentifierScopeDelimiter NvimInvalidIdentifier",
  "default link NvimInvalidIdentifierName NvimInvalidIdentifier",
  "default link NvimInvalidIdentifierKey NvimInvalidIdentifier",

  "default link NvimInvalidColon NvimInvalidDelimiter",
  "default link NvimInvalidComma NvimInvalidDelimiter",
  "default link NvimInvalidArrow NvimInvalidDelimiter",

  "default link NvimInvalidRegister NvimInvalidValue",
  "default link NvimInvalidNumber NvimInvalidValue",
  "default link NvimInvalidFloat NvimInvalidNumber",
  "default link NvimInvalidNumberPrefix NvimInvalidNumber",

  "default link NvimInvalidOptionSigil NvimInvalidIdentifier",
  "default link NvimInvalidOptionName NvimInvalidIdentifier",
  "default link NvimInvalidOptionScope NvimInvalidIdentifierScope",
  "default link NvimInvalidOptionScopeDelimiter NvimInvalidIdentifierScopeDelimiter",

  "default link NvimInvalidEnvironmentSigil NvimInvalidOptionSigil",
  "default link NvimInvalidEnvironmentName NvimInvalidIdentifier",

  // Invalid string bodies and specials are still highlighted as valid ones to
  // minimize the red area.
  "default link NvimInvalidString NvimInvalidValue",
  "default link NvimInvalidStringBody NvimStringBody",
  "default link NvimInvalidStringQuote NvimInvalidString",
  "default link NvimInvalidStringSpecial NvimStringSpecial",

  "default link NvimInvalidSingleQuote NvimInvalidStringQuote",
  "default link NvimInvalidSingleQuotedBody NvimInvalidStringBody",
  "default link NvimInvalidSingleQuotedQuote NvimInvalidStringSpecial",

  "default link NvimInvalidDoubleQuote NvimInvalidStringQuote",
  "default link NvimInvalidDoubleQuotedBody NvimInvalidStringBody",
  "default link NvimInvalidDoubleQuotedEscape NvimInvalidStringSpecial",
  "default link NvimInvalidDoubleQuotedUnknownEscape NvimInvalidValue",

  "default link NvimInvalidFigureBrace NvimInvalidDelimiter",

  "default link NvimInvalidSpacing ErrorMsg",

  // Not actually invalid, but we show the user that they are doing something
  // wrong.
  "default link NvimDoubleQuotedUnknownEscape NvimInvalidValue",
  NULL,
};

/// Returns the name of a highlight group.
char *highlight_group_name(int id)
{
  return hl_table[id].sg_name;
}

/// Returns the ID of the link to a highlight group.
int highlight_link_id(int id)
{
  return hl_table[id].sg_link;
}

/// Returns the attribute ID (screen attr) of a highlight group.
/// @param id Highlight group index (0-based)
/// @return The sg_attr field, or 0 if id is out of bounds
int highlight_group_attr(int id)
{
  if (id < 0 || id >= highlight_ga.ga_len) {
    return 0;
  }
  return hl_table[id].sg_attr;
}

/// Returns whether a highlight group has been cleared.
/// @param id Highlight group index (0-based)
/// @return true if cleared, false otherwise (including out-of-bounds)
bool highlight_group_cleared(int id)
{
  if (id < 0 || id >= highlight_ga.ga_len) {
    return false;
  }
  return hl_table[id].sg_cleared;
}

/// Returns the sg_set flags of a highlight group.
/// @param id Highlight group index (0-based)
/// @return The sg_set field (combination of SG_* flags), or 0 if out of bounds
int highlight_group_set(int id)
{
  if (id < 0 || id >= highlight_ga.ga_len) {
    return 0;
  }
  return hl_table[id].sg_set;
}

/// Returns the parent ID of a highlight group (for @nested.groups).
/// @param id Highlight group index (0-based)
/// @return The sg_parent field, or 0 if out of bounds
int highlight_group_parent(int id)
{
  if (id < 0 || id >= highlight_ga.ga_len) {
    return 0;
  }
  return hl_table[id].sg_parent;
}

/// Lookup a highlight group by uppercase name.
/// @param name_u Uppercase name to look up (must be null-terminated)
/// @return The highlight group ID (1-based), or 0 if not found
int nvim_highlight_name_lookup(const char *name_u)
{
  return map_get(cstr_t, int)(&highlight_unames, name_u);
}

/// Create default links for Nvim* highlight groups used for cmdline coloring
void syn_init_cmdline_highlight(bool reset, bool init)
{
  for (size_t i = 0; highlight_init_cmdline[i] != NULL; i++) {
    do_highlight(highlight_init_cmdline[i], reset, init);
  }
}

/// Load colors from a file if "g:colors_name" is set, otherwise load builtin
/// colors
///
/// @param both include groups where 'bg' doesn't matter
/// @param reset clear groups first
void init_highlight(bool both, bool reset)
{
  static bool had_both = false;

  // Try finding the color scheme file.  Used when a color file was loaded
  // and 'background' or 't_Co' is changed.
  char *p = get_var_value("g:colors_name");
  if (p != NULL) {
    // Value of g:colors_name could be freed in load_colors() and make
    // p invalid, so copy it.
    char *copy_p = xstrdup(p);
    bool okay = load_colors(copy_p);
    xfree(copy_p);
    if (okay) {
      return;
    }
  }

  // Didn't use a color file, use the compiled-in colors.
  if (both) {
    had_both = true;
    const char *const *const pp = highlight_init_both;
    for (size_t i = 0; pp[i] != NULL; i++) {
      do_highlight(pp[i], reset, true);
    }
  } else if (!had_both) {
    // Don't do anything before the call with both == true from main().
    // Not everything has been setup then, and that call will overrule
    // everything anyway.
    return;
  }

  const char *const *const pp = ((*p_bg == 'l')
                                 ? highlight_init_light
                                 : highlight_init_dark);
  for (size_t i = 0; pp[i] != NULL; i++) {
    do_highlight(pp[i], reset, true);
  }

  syn_init_cmdline_highlight(false, false);
}

/// Load color file "name".
///
/// @return  OK for success, FAIL for failure.
int load_colors(char *name)
{
  static bool recursive = false;

  // When being called recursively, this is probably because setting
  // 'background' caused the highlighting to be reloaded.  This means it is
  // working, thus we should return OK.
  if (recursive) {
    return OK;
  }

  recursive = true;
  size_t buflen = strlen(name) + 12;
  char *buf = xmalloc(buflen);
  apply_autocmds(EVENT_COLORSCHEMEPRE, name, curbuf->b_fname, false, curbuf);
  snprintf(buf, buflen, "colors/%s.*", name);
  int retval = source_runtime_vim_lua(buf, DIP_START + DIP_OPT);
  xfree(buf);
  if (retval == OK) {
    apply_autocmds(EVENT_COLORSCHEME, name, curbuf->b_fname, false, curbuf);
  }

  recursive = false;

  return retval;
}

static char *(color_names[28]) = {
  "Black", "DarkBlue", "DarkGreen", "DarkCyan",
  "DarkRed", "DarkMagenta", "Brown", "DarkYellow",
  "Gray", "Grey", "LightGray", "LightGrey",
  "DarkGray", "DarkGrey",
  "Blue", "LightBlue", "Green", "LightGreen",
  "Cyan", "LightCyan", "Red", "LightRed", "Magenta",
  "LightMagenta", "Yellow", "LightYellow", "White", "NONE"
};
// Lookup the "cterm" value to be used for color with index "idx" in
// color_names[].
// "boldp" will be set to kTrue or kFalse for a foreground color when using 8
// colors, otherwise it will be unchanged.
typedef struct {
  int color;
  int bold;  // -1 = unchanged, 0 = kFalse, 1 = kTrue
} LookupColorResult;
extern LookupColorResult rs_lookup_color(int idx, bool foreground);

static int lookup_color(const int idx, const bool foreground, TriState *const boldp)
{
  LookupColorResult result = rs_lookup_color(idx, foreground);
  if (result.bold == 0) {
    *boldp = kFalse;
  } else if (result.bold == 1) {
    *boldp = kTrue;
  }
  // bold == -1 means unchanged, leave boldp as-is
  return result.color;
}


void set_hl_group(int id, HlAttrs attrs, Dict(highlight) *dict, int link_id)
{
  int idx = id - 1;  // Index is ID minus one.
  bool is_default = attrs.rgb_ae_attr & HL_DEFAULT;

  // Return if "default" was used and the group already has settings
  if (is_default && hl_has_settings(idx, true) && !dict->force) {
    return;
  }

  HlGroup *g = &hl_table[idx];
  g->sg_cleared = false;

  if (link_id > 0) {
    g->sg_link = link_id;
    g->sg_script_ctx = current_sctx;
    g->sg_script_ctx.sc_lnum += SOURCING_LNUM;
    nlua_set_sctx(&g->sg_script_ctx);
    g->sg_set |= SG_LINK;
    if (is_default) {
      g->sg_deflink = link_id;
      g->sg_deflink_sctx = current_sctx;
      g->sg_deflink_sctx.sc_lnum += SOURCING_LNUM;
      nlua_set_sctx(&g->sg_deflink_sctx);
    }
  } else {
    g->sg_link = 0;
  }

  g->sg_gui = attrs.rgb_ae_attr &~HL_DEFAULT;

  g->sg_rgb_fg = attrs.rgb_fg_color;
  g->sg_rgb_bg = attrs.rgb_bg_color;
  g->sg_rgb_sp = attrs.rgb_sp_color;

  struct {
    int *dest; RgbValue val; Object name;
  } cattrs[] = {
    { &g->sg_rgb_fg_idx, g->sg_rgb_fg,
      HAS_KEY(dict, highlight, fg) ? dict->fg : dict->foreground },
    { &g->sg_rgb_bg_idx, g->sg_rgb_bg,
      HAS_KEY(dict, highlight, bg) ? dict->bg : dict->background },
    { &g->sg_rgb_sp_idx, g->sg_rgb_sp, HAS_KEY(dict, highlight, sp) ? dict->sp : dict->special },
    { NULL, -1, NIL },
  };

  for (int j = 0; cattrs[j].dest; j++) {
    if (cattrs[j].val < 0) {
      *cattrs[j].dest = kColorIdxNone;
    } else if (cattrs[j].name.type == kObjectTypeString && cattrs[j].name.data.string.size) {
      name_to_color(cattrs[j].name.data.string.data, cattrs[j].dest);
    } else {
      *cattrs[j].dest = kColorIdxHex;
    }
  }

  g->sg_cterm = attrs.cterm_ae_attr &~HL_DEFAULT;
  g->sg_cterm_bg = attrs.cterm_bg_color;
  g->sg_cterm_fg = attrs.cterm_fg_color;
  g->sg_cterm_bold = g->sg_cterm & HL_BOLD;
  g->sg_blend = attrs.hl_blend;

  g->sg_script_ctx = current_sctx;
  g->sg_script_ctx.sc_lnum += SOURCING_LNUM;
  nlua_set_sctx(&g->sg_script_ctx);

  g->sg_attr = hl_get_syn_attr(0, id, attrs);

  // 'Normal' is special
  if (strcmp(g->sg_name_u, "NORMAL") == 0) {
    cterm_normal_fg_color = g->sg_cterm_fg;
    cterm_normal_bg_color = g->sg_cterm_bg;
    bool did_changed = false;
    if (normal_bg != g->sg_rgb_bg || normal_fg != g->sg_rgb_fg || normal_sp != g->sg_rgb_sp) {
      did_changed = true;
    }
    normal_fg = g->sg_rgb_fg;
    normal_bg = g->sg_rgb_bg;
    normal_sp = g->sg_rgb_sp;

    if (did_changed) {
      highlight_attr_set_all();
    }
    ui_default_colors_set();
  } else {
    // a cursor style uses this syn_id, make sure its attribute is updated.
    if (cursor_mode_uses_syn_id(id)) {
      ui_mode_info_set();
    }
  }

  if (!updating_screen) {
    redraw_all_later(UPD_NOT_VALID);
  }
  need_highlight_changed = true;
}

/// Handle ":highlight" command
///
/// When using ":highlight clear" this is called recursively for each group with
/// forceit and init being both true.
///
/// @param[in]  line  Command arguments.
/// @param[in]  forceit  True when bang is given, allows to link group even if
///                      it has its own settings.
/// @param[in]  init  True when initializing.
void do_highlight(const char *line, const bool forceit, const bool init)
  FUNC_ATTR_NONNULL_ALL
{
  // If no argument, list current highlighting.
  if (!init && ends_excmd((uint8_t)(*line))) {
    msg_ext_set_kind("list_cmd");
    for (int i = 1; i <= highlight_ga.ga_len && !got_int; i++) {
      // TODO(brammool): only call when the group has attributes set
      highlight_list_one(i);
    }
    return;
  }

  bool dodefault = false;

  // Isolate the name.
  const char *name_end = skiptowhite(line);
  const char *linep = skipwhite(name_end);

  // Check for "default" argument.
  if (strncmp(line, "default", (size_t)(name_end - line)) == 0) {
    dodefault = true;
    line = linep;
    name_end = skiptowhite(line);
    linep = skipwhite(name_end);
  }

  bool doclear = false;
  bool dolink = false;

  // Check for "clear" or "link" argument.
  if (strncmp(line, "clear", (size_t)(name_end - line)) == 0) {
    doclear = true;
  } else if (strncmp(line, "link", (size_t)(name_end - line)) == 0) {
    dolink = true;
  }

  // ":highlight {group-name}": list highlighting for one group.
  if (!doclear && !dolink && ends_excmd((uint8_t)(*linep))) {
    int id = syn_name2id_len(line, (size_t)(name_end - line));
    if (id == 0) {
      semsg(_(e_highlight_group_name_not_found_str), line);
    } else {
      msg_ext_set_kind("list_cmd");
      highlight_list_one(id);
    }
    return;
  }

  // Handle ":highlight link {from} {to}" command.
  if (dolink) {
    const char *from_start = linep;
    int to_id;
    HlGroup *hlgroup = NULL;

    const char *from_end = skiptowhite(from_start);
    const char *to_start = skipwhite(from_end);
    const char *to_end = skiptowhite(to_start);

    if (ends_excmd((uint8_t)(*from_start))
        || ends_excmd((uint8_t)(*to_start))) {
      semsg(_("E412: Not enough arguments: \":highlight link %s\""),
            from_start);
      return;
    }

    if (!ends_excmd(*skipwhite(to_end))) {
      semsg(_("E413: Too many arguments: \":highlight link %s\""), from_start);
      return;
    }

    int from_id = syn_check_group(from_start, (size_t)(from_end - from_start));
    if (strncmp(to_start, "NONE", 4) == 0) {
      to_id = 0;
    } else {
      to_id = syn_check_group(to_start, (size_t)(to_end - to_start));
    }

    if (from_id > 0) {
      hlgroup = &hl_table[from_id - 1];
      if (dodefault && (forceit || hlgroup->sg_deflink == 0)) {
        hlgroup->sg_deflink = to_id;
        hlgroup->sg_deflink_sctx = current_sctx;
        hlgroup->sg_deflink_sctx.sc_lnum += SOURCING_LNUM;
        nlua_set_sctx(&hlgroup->sg_deflink_sctx);
      }
    }

    if (from_id > 0 && (!init || hlgroup->sg_set == 0)) {
      // Don't allow a link when there already is some highlighting
      // for the group, unless '!' is used
      if (to_id > 0 && !forceit && !init
          && hl_has_settings(from_id - 1, dodefault)) {
        if (SOURCING_NAME == NULL && !dodefault) {
          emsg(_(e_group_has_settings_highlight_link_ignored));
        }
      } else if (hlgroup->sg_link != to_id
                 || hlgroup->sg_script_ctx.sc_sid != current_sctx.sc_sid
                 || hlgroup->sg_cleared) {
        if (!init) {
          hlgroup->sg_set |= SG_LINK;
        }
        hlgroup->sg_link = to_id;
        hlgroup->sg_script_ctx = current_sctx;
        hlgroup->sg_script_ctx.sc_lnum += SOURCING_LNUM;
        nlua_set_sctx(&hlgroup->sg_script_ctx);
        hlgroup->sg_cleared = false;
        redraw_all_later(UPD_SOME_VALID);

        // Only call highlight changed() once after multiple changes
        need_highlight_changed = true;
      }
    }

    return;
  }

  if (doclear) {
    // ":highlight clear [group]" command.
    line = linep;
    if (ends_excmd((uint8_t)(*line))) {
      do_unlet(S_LEN("g:colors_name"), true);
      restore_cterm_colors();

      // Clear all default highlight groups and load the defaults.
      for (int j = 0; j < highlight_ga.ga_len; j++) {
        highlight_clear(j);
      }
      init_highlight(true, true);
      highlight_changed();
      redraw_all_later(UPD_NOT_VALID);
      return;
    }
    name_end = skiptowhite(line);
    linep = skipwhite(name_end);
  }

  // Find the group name in the table.  If it does not exist yet, add it.
  int id = syn_check_group(line, (size_t)(name_end - line));
  if (id == 0) {  // Failed (out of memory).
    return;
  }
  int idx = id - 1;  // Index is ID minus one.

  // Return if "default" was used and the group already has settings
  if (dodefault && hl_has_settings(idx, true)) {
    return;
  }

  // Make a copy so we can check if any attribute actually changed
  HlGroup item_before = hl_table[idx];
  bool is_normal_group = (strcmp(hl_table[idx].sg_name_u, "NORMAL") == 0);

  // Clear the highlighting for ":hi clear {group}" and ":hi clear".
  if (doclear || (forceit && init)) {
    highlight_clear(idx);
    if (!doclear) {
      hl_table[idx].sg_set = 0;
    }
  }

  bool did_change = false;
  bool error = false;

  char key[64];
  char arg[512];
  if (!doclear) {
    const char *arg_start;

    while (!ends_excmd((uint8_t)(*linep))) {
      const char *key_start = linep;
      if (*linep == '=') {
        semsg(_(e_unexpected_equal_sign_str), key_start);
        error = true;
        break;
      }

      // Isolate the key ("term", "ctermfg", "ctermbg", "font", "guifg",
      // "guibg" or "guisp").
      while (*linep && !ascii_iswhite(*linep) && *linep != '=') {
        linep++;
      }
      size_t key_len = (size_t)(linep - key_start);
      if (key_len > sizeof(key) - 1) {
        emsg(_("E423: Illegal argument"));
        error = true;
        break;
      }
      vim_memcpy_up(key, key_start, key_len);
      key[key_len] = NUL;
      linep = skipwhite(linep);

      if (strcmp(key, "NONE") == 0) {
        if (!init || hl_table[idx].sg_set == 0) {
          if (!init) {
            hl_table[idx].sg_set |= SG_CTERM + SG_GUI;
          }
          highlight_clear(idx);
        }
        continue;
      }

      // Check for the equal sign.
      if (*linep != '=') {
        semsg(_(e_missing_equal_sign_str_2), key_start);
        error = true;
        break;
      }
      linep++;

      // Isolate the argument.
      linep = skipwhite(linep);
      if (*linep == '\'') {  // guifg='color name'
        arg_start = ++linep;
        linep = strchr(linep, '\'');
        if (linep == NULL) {
          semsg(_(e_invarg2), key_start);
          error = true;
          break;
        }
      } else {
        arg_start = linep;
        linep = skiptowhite(linep);
      }
      if (linep == arg_start) {
        semsg(_(e_missing_argument_str), key_start);
        error = true;
        break;
      }
      size_t arg_len = (size_t)(linep - arg_start);
      if (arg_len > sizeof(arg) - 1) {
        emsg(_("E423: Illegal argument"));
        error = true;
        break;
      }
      memcpy(arg, arg_start, arg_len);
      arg[arg_len] = NUL;

      if (*linep == '\'') {
        linep++;
      }

      // Store the argument.
      if (strcmp(key, "TERM") == 0
          || strcmp(key, "CTERM") == 0
          || strcmp(key, "GUI") == 0) {
        int attr = 0;
        int off = 0;
        int i;
        while (arg[off] != NUL) {
          for (i = ARRAY_SIZE(hl_attr_table); --i >= 0;) {
            int len = (int)strlen(hl_name_table[i]);
            if (STRNICMP(arg + off, hl_name_table[i], len) == 0) {
              if (hl_attr_table[i] & HL_UNDERLINE_MASK) {
                attr &= ~HL_UNDERLINE_MASK;
              }
              attr |= hl_attr_table[i];
              off += len;
              break;
            }
          }
          if (i < 0) {
            semsg(_("E418: Illegal value: %s"), arg);
            error = true;
            break;
          }
          if (arg[off] == ',') {  // Another one follows.
            off++;
          }
        }
        if (error) {
          break;
        }
        if (*key == 'C') {
          if (!init || !(hl_table[idx].sg_set & SG_CTERM)) {
            if (!init) {
              hl_table[idx].sg_set |= SG_CTERM;
            }
            hl_table[idx].sg_cterm = attr;
            hl_table[idx].sg_cterm_bold = false;
          }
        } else if (*key == 'G') {
          if (!init || !(hl_table[idx].sg_set & SG_GUI)) {
            if (!init) {
              hl_table[idx].sg_set |= SG_GUI;
            }
            hl_table[idx].sg_gui = attr;
          }
        }
      } else if (strcmp(key, "FONT") == 0) {
        // in non-GUI fonts are simply ignored
      } else if (strcmp(key, "CTERMFG") == 0 || strcmp(key, "CTERMBG") == 0) {
        if (!init || !(hl_table[idx].sg_set & SG_CTERM)) {
          if (!init) {
            hl_table[idx].sg_set |= SG_CTERM;
          }

          // When setting the foreground color, and previously the "bold"
          // flag was set for a light color, reset it now
          if (key[5] == 'F' && hl_table[idx].sg_cterm_bold) {
            hl_table[idx].sg_cterm &= ~HL_BOLD;
            hl_table[idx].sg_cterm_bold = false;
          }

          int color;
          if (ascii_isdigit(*arg)) {
            color = atoi(arg);
          } else if (STRICMP(arg, "fg") == 0) {
            if (cterm_normal_fg_color) {
              color = cterm_normal_fg_color - 1;
            } else {
              emsg(_("E419: FG color unknown"));
              error = true;
              break;
            }
          } else if (STRICMP(arg, "bg") == 0) {
            if (cterm_normal_bg_color > 0) {
              color = cterm_normal_bg_color - 1;
            } else {
              emsg(_("E420: BG color unknown"));
              error = true;
              break;
            }
          } else {
            // Reduce calls to STRICMP a bit, it can be slow.
            int off = TOUPPER_ASC(*arg);
            int i;
            for (i = ARRAY_SIZE(color_names); --i >= 0;) {
              if (off == color_names[i][0]
                  && STRICMP(arg + 1, color_names[i] + 1) == 0) {
                break;
              }
            }
            if (i < 0) {
              semsg(_("E421: Color name or number not recognized: %s"),
                    key_start);
              error = true;
              break;
            }

            TriState bold = kNone;
            color = lookup_color(i, key[5] == 'F', &bold);

            // set/reset bold attribute to get light foreground
            // colors (on some terminals, e.g. "linux")
            if (bold == kTrue) {
              hl_table[idx].sg_cterm |= HL_BOLD;
              hl_table[idx].sg_cterm_bold = true;
            } else if (bold == kFalse) {
              hl_table[idx].sg_cterm &= ~HL_BOLD;
            }
          }
          // Add one to the argument, to avoid zero.  Zero is used for
          // "NONE", then "color" is -1.
          if (key[5] == 'F') {
            hl_table[idx].sg_cterm_fg = color + 1;
            if (is_normal_group) {
              cterm_normal_fg_color = color + 1;
            }
          } else {
            hl_table[idx].sg_cterm_bg = color + 1;
            if (is_normal_group) {
              cterm_normal_bg_color = color + 1;
              if (!ui_rgb_attached()) {
                if (color >= 0) {
                  int dark = -1;

                  if (t_colors < 16) {
                    dark = (color == 0 || color == 4);
                  } else if (color < 16) {
                    // Limit the heuristic to the standard 16 colors
                    dark = (color < 7 || color == 8);
                  }
                  // Set the 'background' option if the value is
                  // wrong.
                  if (dark != -1
                      && dark != (*p_bg == 'd')
                      && !option_was_set(kOptBackground)) {
                    set_option_value_give_err(kOptBackground,
                                              CSTR_AS_OPTVAL(dark ? "dark" : "light"), 0);
                    reset_option_was_set(kOptBackground);
                  }
                }
              }
            }
          }
        }
      } else if (strcmp(key, "GUIFG") == 0) {
        int *indexp = &hl_table[idx].sg_rgb_fg_idx;

        if (!init || !(hl_table[idx].sg_set & SG_GUI)) {
          if (!init) {
            hl_table[idx].sg_set |= SG_GUI;
          }

          RgbValue old_color = hl_table[idx].sg_rgb_fg;
          int old_idx = hl_table[idx].sg_rgb_fg_idx;

          if (strcmp(arg, "NONE") != 0) {
            hl_table[idx].sg_rgb_fg = name_to_color(arg, indexp);
          } else {
            hl_table[idx].sg_rgb_fg = -1;
            hl_table[idx].sg_rgb_fg_idx = kColorIdxNone;
          }

          did_change = hl_table[idx].sg_rgb_fg != old_color || hl_table[idx].sg_rgb_fg != old_idx;
        }

        if (is_normal_group) {
          normal_fg = hl_table[idx].sg_rgb_fg;
        }
      } else if (strcmp(key, "GUIBG") == 0) {
        int *indexp = &hl_table[idx].sg_rgb_bg_idx;

        if (!init || !(hl_table[idx].sg_set & SG_GUI)) {
          if (!init) {
            hl_table[idx].sg_set |= SG_GUI;
          }

          RgbValue old_color = hl_table[idx].sg_rgb_bg;
          int old_idx = hl_table[idx].sg_rgb_bg_idx;

          if (strcmp(arg, "NONE") != 0) {
            hl_table[idx].sg_rgb_bg = name_to_color(arg, indexp);
          } else {
            hl_table[idx].sg_rgb_bg = -1;
            hl_table[idx].sg_rgb_bg_idx = kColorIdxNone;
          }

          did_change = hl_table[idx].sg_rgb_bg != old_color || hl_table[idx].sg_rgb_bg != old_idx;
        }

        if (is_normal_group) {
          normal_bg = hl_table[idx].sg_rgb_bg;
        }
      } else if (strcmp(key, "GUISP") == 0) {
        int *indexp = &hl_table[idx].sg_rgb_sp_idx;

        if (!init || !(hl_table[idx].sg_set & SG_GUI)) {
          if (!init) {
            hl_table[idx].sg_set |= SG_GUI;
          }

          RgbValue old_color = hl_table[idx].sg_rgb_sp;
          int old_idx = hl_table[idx].sg_rgb_sp_idx;

          if (strcmp(arg, "NONE") != 0) {
            hl_table[idx].sg_rgb_sp = name_to_color(arg, indexp);
          } else {
            hl_table[idx].sg_rgb_sp = -1;
          }

          did_change = hl_table[idx].sg_rgb_sp != old_color || hl_table[idx].sg_rgb_sp != old_idx;
        }

        if (is_normal_group) {
          normal_sp = hl_table[idx].sg_rgb_sp;
        }
      } else if (strcmp(key, "START") == 0 || strcmp(key, "STOP") == 0) {
        // Ignored for now
      } else if (strcmp(key, "BLEND") == 0) {
        if (strcmp(arg, "NONE") != 0) {
          hl_table[idx].sg_blend = (int)strtol(arg, NULL, 10);
        } else {
          hl_table[idx].sg_blend = -1;
        }
      } else {
        semsg(_("E423: Illegal argument: %s"), key_start);
        error = true;
        break;
      }
      hl_table[idx].sg_cleared = false;

      // When highlighting has been given for a group, don't link it.
      if (!init || !(hl_table[idx].sg_set & SG_LINK)) {
        hl_table[idx].sg_link = 0;
      }

      // Continue with next argument.
      linep = skipwhite(linep);
    }
  }

  bool did_highlight_changed = false;

  if (!error && is_normal_group) {
    // Need to update all groups, because they might be using "bg" and/or
    // "fg", which have been changed now.
    highlight_attr_set_all();

    if (!ui_has(kUILinegrid) && starting == 0) {
      // Older UIs assume that we clear the screen after normal group is
      // changed
      ui_refresh();
    } else {
      // TUI and newer UIs will repaint the screen themselves. UPD_NOT_VALID
      // redraw below will still handle usages of guibg=fg etc.
      ui_default_colors_set();
    }
    did_highlight_changed = true;
    redraw_all_later(UPD_NOT_VALID);
  } else {
    set_hl_attr(idx);
  }
  hl_table[idx].sg_script_ctx = current_sctx;
  hl_table[idx].sg_script_ctx.sc_lnum += SOURCING_LNUM;
  nlua_set_sctx(&hl_table[idx].sg_script_ctx);

  // Only call highlight_changed() once, after a sequence of highlight
  // commands, and only if an attribute actually changed
  if ((did_change
       || memcmp(&hl_table[idx], &item_before, sizeof(item_before)) != 0)
      && !did_highlight_changed) {
    // Do not trigger a redraw when highlighting is changed while
    // redrawing.  This may happen when evaluating 'statusline' changes the
    // StatusLine group.
    if (!updating_screen) {
      redraw_all_later(UPD_NOT_VALID);
    }
    need_highlight_changed = true;
  }
}

#if defined(EXITFREE)
void free_highlight(void)
{
  ga_clear(&highlight_ga);
  map_destroy(cstr_t, &highlight_unames);
  arena_mem_free(arena_finish(&highlight_arena));
}

#endif

/// @param check_link  if true also check for an existing link.
///
/// @return true if highlight group "idx" has any settings.
static bool hl_has_settings(int idx, bool check_link)
{
  return hl_table[idx].sg_cleared == 0
         && (hl_table[idx].sg_attr != 0
             || hl_table[idx].sg_cterm_fg != 0
             || hl_table[idx].sg_cterm_bg != 0
             || hl_table[idx].sg_rgb_fg_idx != kColorIdxNone
             || hl_table[idx].sg_rgb_bg_idx != kColorIdxNone
             || hl_table[idx].sg_rgb_sp_idx != kColorIdxNone
             || (check_link && (hl_table[idx].sg_set & SG_LINK)));
}

/// Clear highlighting for one group.
static void highlight_clear(int idx)
{
  hl_table[idx].sg_cleared = true;

  hl_table[idx].sg_attr = 0;
  hl_table[idx].sg_cterm = 0;
  hl_table[idx].sg_cterm_bold = false;
  hl_table[idx].sg_cterm_fg = 0;
  hl_table[idx].sg_cterm_bg = 0;
  hl_table[idx].sg_gui = 0;
  hl_table[idx].sg_rgb_fg = -1;
  hl_table[idx].sg_rgb_bg = -1;
  hl_table[idx].sg_rgb_sp = -1;
  hl_table[idx].sg_rgb_fg_idx = kColorIdxNone;
  hl_table[idx].sg_rgb_bg_idx = kColorIdxNone;
  hl_table[idx].sg_rgb_sp_idx = kColorIdxNone;
  hl_table[idx].sg_blend = -1;
  // Restore default link and context if they exist. Otherwise clears.
  hl_table[idx].sg_link = hl_table[idx].sg_deflink;
  // Since we set the default link, set the location to where the default
  // link was set.
  hl_table[idx].sg_script_ctx = hl_table[idx].sg_deflink_sctx;
}

/// \addtogroup LIST_XXX
/// @{
#define LIST_ATTR   1
#define LIST_STRING 2
#define LIST_INT    3
/// @}

static void highlight_list_one(const int id)
{
  const HlGroup *sgp = &hl_table[id - 1];  // index is ID minus one
  bool didh = false;

  if (message_filtered(sgp->sg_name)) {
    return;
  }

  // don't list specialized groups if a parent is used instead
  if (sgp->sg_parent && sgp->sg_cleared) {
    return;
  }

  didh = highlight_list_arg(id, didh, LIST_ATTR,
                            sgp->sg_cterm, NULL, "cterm");
  didh = highlight_list_arg(id, didh, LIST_INT,
                            sgp->sg_cterm_fg, NULL, "ctermfg");
  didh = highlight_list_arg(id, didh, LIST_INT,
                            sgp->sg_cterm_bg, NULL, "ctermbg");

  didh = highlight_list_arg(id, didh, LIST_ATTR,
                            sgp->sg_gui, NULL, "gui");
  char hexbuf[8];
  didh = highlight_list_arg(id, didh, LIST_STRING, 0,
                            coloridx_to_name(sgp->sg_rgb_fg_idx, sgp->sg_rgb_fg, hexbuf), "guifg");
  didh = highlight_list_arg(id, didh, LIST_STRING, 0,
                            coloridx_to_name(sgp->sg_rgb_bg_idx, sgp->sg_rgb_bg, hexbuf), "guibg");
  didh = highlight_list_arg(id, didh, LIST_STRING, 0,
                            coloridx_to_name(sgp->sg_rgb_sp_idx, sgp->sg_rgb_sp, hexbuf), "guisp");

  didh = highlight_list_arg(id, didh, LIST_INT,
                            sgp->sg_blend + 1, NULL, "blend");

  if (sgp->sg_link && !got_int) {
    syn_list_header(didh, 0, id, true);
    didh = true;
    msg_puts_hl("links to", HLF_D, false);
    msg_putchar(' ');
    msg_outtrans(hl_table[hl_table[id - 1].sg_link - 1].sg_name, 0, false);
  }

  if (!didh) {
    highlight_list_arg(id, didh, LIST_STRING, 0, "cleared", "");
  }
  if (p_verbose > 0) {
    last_set_msg(sgp->sg_script_ctx);
  }
}

static bool hlgroup2dict(Dict *hl, NS ns_id, int hl_id, Arena *arena)
{
  HlGroup *sgp = &hl_table[hl_id - 1];
  int link = ns_id == 0 ? sgp->sg_link : ns_get_hl(&ns_id, hl_id, true, sgp->sg_set);
  if (link == -1) {
    return false;
  }
  if (ns_id == 0 && sgp->sg_cleared && sgp->sg_set == 0) {
    // table entry was created but not ever set
    return false;
  }
  HlAttrs attr =
    syn_attr2entry(ns_id == 0 ? sgp->sg_attr : ns_get_hl(&ns_id, hl_id, false, sgp->sg_set));
  *hl = arena_dict(arena, HLATTRS_DICT_SIZE + 1);
  if (attr.rgb_ae_attr & HL_DEFAULT) {
    PUT_C(*hl, "default", BOOLEAN_OBJ(true));
  }
  if (link > 0) {
    assert(1 <= link && link <= highlight_ga.ga_len);
    PUT_C(*hl, "link", CSTR_AS_OBJ(hl_table[link - 1].sg_name));
  }
  Dict hl_cterm = arena_dict(arena, HLATTRS_DICT_SIZE);
  hlattrs2dict(hl, NULL, attr, true, true);
  hlattrs2dict(hl, &hl_cterm, attr, false, true);
  if (kv_size(hl_cterm)) {
    PUT_C(*hl, "cterm", DICT_OBJ(hl_cterm));
  }
  return true;
}

Dict ns_get_hl_defs(NS ns_id, Dict(get_highlight) *opts, Arena *arena, Error *err)
{
  Boolean link = GET_BOOL_OR_TRUE(opts, get_highlight, link);
  int id = -1;
  if (HAS_KEY(opts, get_highlight, name)) {
    Boolean create = GET_BOOL_OR_TRUE(opts, get_highlight, create);
    id = create ? syn_check_group(opts->name.data, opts->name.size)
                : syn_name2id_len(opts->name.data, opts->name.size);
    if (id == 0 && !create) {
      Dict attrs = ARRAY_DICT_INIT;
      return attrs;
    }
  } else if (HAS_KEY(opts, get_highlight, id)) {
    id = (int)opts->id;
  }

  if (id != -1) {
    VALIDATE(1 <= id && id <= highlight_ga.ga_len, "%s", "Highlight id out of bounds", {
      goto cleanup;
    });
    Dict attrs = ARRAY_DICT_INIT;
    hlgroup2dict(&attrs, ns_id, link ? id : syn_get_final_id(id), arena);
    return attrs;
  }
  if (ERROR_SET(err)) {
    goto cleanup;
  }

  Dict rv = arena_dict(arena, (size_t)highlight_ga.ga_len);
  for (int i = 1; i <= highlight_ga.ga_len; i++) {
    Dict attrs = ARRAY_DICT_INIT;
    if (!hlgroup2dict(&attrs, ns_id, i, arena)) {
      continue;
    }
    PUT_C(rv, hl_table[(link ? i : syn_get_final_id(i)) - 1].sg_name, DICT_OBJ(attrs));
  }

  return rv;

cleanup:
  return (Dict)ARRAY_DICT_INIT;
}

/// Outputs a highlight when doing ":hi MyHighlight"
///
/// @param type one of \ref LIST_XXX
/// @param iarg integer argument used if \p type == LIST_INT
/// @param sarg string used if \p type == LIST_STRING
static bool highlight_list_arg(const int id, bool didh, const int type, int iarg, const char *sarg,
                               const char *const name)
{
  if (got_int) {
    return false;
  }

  if (type == LIST_STRING ? (sarg == NULL) : (iarg == 0)) {
    return didh;
  }

  char buf[100];
  const char *ts = buf;
  if (type == LIST_INT) {
    snprintf(buf, sizeof(buf), "%d", iarg - 1);
  } else if (type == LIST_STRING) {
    ts = sarg;
  } else {    // type == LIST_ATTR
    buf[0] = NUL;
    for (int i = 0; hl_attr_table[i] != 0; i++) {
      if (((hl_attr_table[i] & HL_UNDERLINE_MASK)
           && ((iarg & HL_UNDERLINE_MASK) == hl_attr_table[i]))
          || (!(hl_attr_table[i] & HL_UNDERLINE_MASK)
              && (iarg & hl_attr_table[i]))) {
        if (buf[0] != NUL) {
          xstrlcat(buf, ",", 100);
        }
        xstrlcat(buf, hl_name_table[i], 100);
        if (!(hl_attr_table[i] & HL_UNDERLINE_MASK)) {
          iarg &= ~hl_attr_table[i];  // don't want "inverse"
        }
      }
    }
  }

  syn_list_header(didh, vim_strsize(ts) + (int)strlen(name) + 1, id, false);
  didh = true;
  if (!got_int) {
    if (*name != NUL) {
      msg_puts_hl(name, HLF_D, false);
      msg_puts_hl("=", HLF_D, false);
    }
    msg_outtrans(ts, 0, false);
  }
  return didh;
}

/// Return color name of the given highlight group
///
/// @param[in]  id  Highlight group to work with.
/// @param[in]  what  What to return: one of "font", "fg", "bg", "sp", "fg#",
///                   "bg#" or "sp#".
/// @param[in]  modec  'g' for GUI, 'c' for cterm and 't' for term.
///
/// @return color name, possibly in a static buffer. Buffer will be overwritten
///         on next highlight_color() call. May return NULL.
const char *highlight_color(const int id, const char *const what, const int modec)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  static char name[20];
  bool fg = false;
  bool sp = false;
  bool font = false;

  if (id <= 0 || id > highlight_ga.ga_len) {
    return NULL;
  }

  if (TOLOWER_ASC(what[0]) == 'f' && TOLOWER_ASC(what[1]) == 'g') {
    fg = true;
  } else if (TOLOWER_ASC(what[0]) == 'f' && TOLOWER_ASC(what[1]) == 'o'
             && TOLOWER_ASC(what[2]) == 'n' && TOLOWER_ASC(what[3]) == 't') {
    font = true;
  } else if (TOLOWER_ASC(what[0]) == 's' && TOLOWER_ASC(what[1]) == 'p') {
    sp = true;
  } else if (!(TOLOWER_ASC(what[0]) == 'b' && TOLOWER_ASC(what[1]) == 'g')) {
    return NULL;
  }

  int n;

  if (modec == 'g') {
    if (what[2] == '#' && ui_rgb_attached()) {
      if (fg) {
        n = hl_table[id - 1].sg_rgb_fg;
      } else if (sp) {
        n = hl_table[id - 1].sg_rgb_sp;
      } else {
        n = hl_table[id - 1].sg_rgb_bg;
      }
      if (n < 0 || n > 0xffffff) {
        return NULL;
      }
      snprintf(name, sizeof(name), "#%06x", n);
      return name;
    }
    if (fg) {
      return coloridx_to_name(hl_table[id - 1].sg_rgb_fg_idx, hl_table[id - 1].sg_rgb_fg, name);
    } else if (sp) {
      return coloridx_to_name(hl_table[id - 1].sg_rgb_sp_idx, hl_table[id - 1].sg_rgb_sp, name);
    } else {
      return coloridx_to_name(hl_table[id - 1].sg_rgb_bg_idx, hl_table[id - 1].sg_rgb_bg, name);
    }
  }
  if (font || sp) {
    return NULL;
  }
  if (modec == 'c') {
    if (fg) {
      n = hl_table[id - 1].sg_cterm_fg - 1;
    } else {
      n = hl_table[id - 1].sg_cterm_bg - 1;
    }
    if (n < 0) {
      return NULL;
    }
    snprintf(name, sizeof(name), "%d", n);
    return name;
  }
  // term doesn't have color.
  return NULL;
}

/// Output the syntax list header.
///
/// @param did_header did header already
/// @param outlen length of string that comes
/// @param id highlight group id
/// @param force_newline always start a new line
/// @return true when started a new line.
bool syn_list_header(const bool did_header, const int outlen, const int id, bool force_newline)
{
  int endcol = 19;
  bool newline = true;
  int name_col = 0;
  bool adjust = true;

  if (!did_header) {
    msg_putchar('\n');
    if (got_int) {
      return true;
    }
    msg_col = name_col = msg_outtrans(hl_table[id - 1].sg_name, 0, false);
    endcol = 15;
  } else if ((ui_has(kUIMessages) || msg_silent) && !force_newline) {
    msg_putchar(' ');
    adjust = false;
  } else if (msg_col + outlen + 1 >= Columns || force_newline) {
    msg_putchar('\n');
    if (got_int) {
      return true;
    }
  } else {
    if (msg_col >= endcol) {    // wrap around is like starting a new line
      newline = false;
    }
  }

  if (adjust) {
    if (msg_col >= endcol) {
      // output at least one space
      endcol = msg_col + 1;
    }

    msg_advance(endcol);
  }

  // Show "xxx" with the attributes.
  if (!did_header) {
    if (endcol == Columns - 1 && endcol <= name_col) {
      msg_putchar(' ');
    }
    msg_puts_hl("xxx", id, false);
    msg_putchar(' ');
  }

  return newline;
}

/// Set the attribute numbers for a highlight group.
/// Called after one of the attributes has changed.
/// @param idx corrected highlight index
static void set_hl_attr(int idx)
{
  HlAttrs at_en = HLATTRS_INIT;
  HlGroup *sgp = hl_table + idx;

  at_en.cterm_ae_attr = (int16_t)sgp->sg_cterm;
  at_en.cterm_fg_color = (int16_t)sgp->sg_cterm_fg;
  at_en.cterm_bg_color = (int16_t)sgp->sg_cterm_bg;
  at_en.rgb_ae_attr = (int16_t)sgp->sg_gui;
  // FIXME(tarruda): The "unset value" for rgb is -1, but since hlgroup is
  // initialized with 0 (by garray functions), check for sg_rgb_{f,b}g_name
  // before setting attr_entry->{f,g}g_color to a other than -1
  at_en.rgb_fg_color = sgp->sg_rgb_fg_idx != kColorIdxNone ? sgp->sg_rgb_fg : -1;
  at_en.rgb_bg_color = sgp->sg_rgb_bg_idx != kColorIdxNone ? sgp->sg_rgb_bg : -1;
  at_en.rgb_sp_color = sgp->sg_rgb_sp_idx != kColorIdxNone ? sgp->sg_rgb_sp : -1;
  at_en.hl_blend = sgp->sg_blend;

  sgp->sg_attr = hl_get_syn_attr(0, idx + 1, at_en);

  // a cursor style uses this syn_id, make sure its attribute is updated.
  if (cursor_mode_uses_syn_id(idx + 1)) {
    ui_mode_info_set();
  }
}

int syn_name2id(const char *name)
  FUNC_ATTR_NONNULL_ALL
{
  if (name[0] == '@') {
    // if we look up @aaa.bbb, we have to consider @aaa as well
    return syn_check_group(name, strlen(name));
  }
  return syn_name2id_len(name, strlen(name));
}

/// Find highlight group name in the table and return its ID.
/// If it doesn't exist yet, a new entry is created.
///
/// @param pp Highlight group name
/// @param len length of \p pp
///
/// @return 0 for failure else the id of the group
// Forward declaration of syn_add_group for Rust to call back
static int syn_add_group(const char *name, size_t len);

// Exposed to Rust for creating new highlight groups
int c_syn_add_group(const char *name, size_t len)
{
  return syn_add_group(name, len);
}

/// Add new highlight group and return its ID.
///
/// @param name must be an allocated string, it will be consumed.
/// @return 0 for failure, else the allocated group id
/// @see syn_check_group
static int syn_add_group(const char *name, size_t len)
{
  // Check that the name is valid (ASCII letters, digits, '_', '.', '@', '-').
  for (size_t i = 0; i < len; i++) {
    int c = (uint8_t)name[i];
    if (!vim_isprintc(c)) {
      emsg(_("E669: Unprintable character in group name"));
      return 0;
    } else if (!ASCII_ISALNUM(c) && c != '_' && c != '.' && c != '@' && c != '-') {
      // '.' and '@' are allowed characters for use with treesitter capture names.
      msg_source(HLF_W);
      emsg(_(e_highlight_group_name_invalid_char));
      return 0;
    }
  }

  int scoped_parent = 0;
  if (len > 1 && name[0] == '@') {
    char *delim = xmemrchr(name, '.', len);
    if (delim) {
      scoped_parent = syn_check_group(name, (size_t)(delim - name));
    }
  }

  // First call for this growarray: init growing array.
  if (highlight_ga.ga_data == NULL) {
    highlight_ga.ga_itemsize = sizeof(HlGroup);
    ga_set_growsize(&highlight_ga, 10);
    // 265 builtin groups, will always be used, plus some space
    ga_grow(&highlight_ga, 300);
  }

  if (highlight_ga.ga_len >= MAX_HL_ID) {
    emsg(_("E849: Too many highlight and syntax groups"));
    return 0;
  }

  // Append another syntax_highlight entry.
  HlGroup *hlgp = GA_APPEND_VIA_PTR(HlGroup, &highlight_ga);
  CLEAR_POINTER(hlgp);
  hlgp->sg_name = arena_memdupz(&highlight_arena, name, len);
  hlgp->sg_rgb_bg = -1;
  hlgp->sg_rgb_fg = -1;
  hlgp->sg_rgb_sp = -1;
  hlgp->sg_rgb_bg_idx = kColorIdxNone;
  hlgp->sg_rgb_fg_idx = kColorIdxNone;
  hlgp->sg_rgb_sp_idx = kColorIdxNone;
  hlgp->sg_blend = -1;
  hlgp->sg_name_u = arena_memdupz(&highlight_arena, name, len);
  hlgp->sg_parent = scoped_parent;
  // will get set to false by caller if settings are added
  hlgp->sg_cleared = true;
  vim_strup(hlgp->sg_name_u);

  int id = highlight_ga.ga_len;  // ID is index plus one

  map_put(cstr_t, int)(&highlight_unames, hlgp->sg_name_u, id);

  return id;
}

/// Refresh the color attributes of all highlight groups.
void highlight_attr_set_all(void)
{
  for (int idx = 0; idx < highlight_ga.ga_len; idx++) {
    HlGroup *sgp = &hl_table[idx];
    if (sgp->sg_rgb_bg_idx == kColorIdxFg) {
      sgp->sg_rgb_bg = normal_fg;
    } else if (sgp->sg_rgb_bg_idx == kColorIdxBg) {
      sgp->sg_rgb_bg = normal_bg;
    }
    if (sgp->sg_rgb_fg_idx == kColorIdxFg) {
      sgp->sg_rgb_fg = normal_fg;
    } else if (sgp->sg_rgb_fg_idx == kColorIdxBg) {
      sgp->sg_rgb_fg = normal_bg;
    }
    if (sgp->sg_rgb_sp_idx == kColorIdxFg) {
      sgp->sg_rgb_sp = normal_fg;
    } else if (sgp->sg_rgb_sp_idx == kColorIdxBg) {
      sgp->sg_rgb_sp = normal_bg;
    }
    set_hl_attr(idx);
  }
}

// Apply difference between User[1-9] and HLF_S to HLF_SNC.
static void combine_stl_hlt(int id, int id_S, int id_alt, int hlcnt, int i, int hlf, int *table)
  FUNC_ATTR_NONNULL_ALL
{
  HlGroup *const hlt = hl_table;

  if (id_alt == 0) {
    CLEAR_POINTER(&hlt[hlcnt + i]);
    hlt[hlcnt + i].sg_cterm = highlight_attr[hlf];
    hlt[hlcnt + i].sg_gui = highlight_attr[hlf];
  } else {
    memmove(&hlt[hlcnt + i], &hlt[id_alt - 1], sizeof(HlGroup));
  }
  hlt[hlcnt + i].sg_link = 0;

  hlt[hlcnt + i].sg_cterm ^= hlt[id - 1].sg_cterm ^ hlt[id_S - 1].sg_cterm;
  if (hlt[id - 1].sg_cterm_fg != hlt[id_S - 1].sg_cterm_fg) {
    hlt[hlcnt + i].sg_cterm_fg = hlt[id - 1].sg_cterm_fg;
  }
  if (hlt[id - 1].sg_cterm_bg != hlt[id_S - 1].sg_cterm_bg) {
    hlt[hlcnt + i].sg_cterm_bg = hlt[id - 1].sg_cterm_bg;
  }
  hlt[hlcnt + i].sg_gui ^= hlt[id - 1].sg_gui ^ hlt[id_S - 1].sg_gui;
  if (hlt[id - 1].sg_rgb_fg != hlt[id_S - 1].sg_rgb_fg) {
    hlt[hlcnt + i].sg_rgb_fg = hlt[id - 1].sg_rgb_fg;
  }
  if (hlt[id - 1].sg_rgb_bg != hlt[id_S - 1].sg_rgb_bg) {
    hlt[hlcnt + i].sg_rgb_bg = hlt[id - 1].sg_rgb_bg;
  }
  if (hlt[id - 1].sg_rgb_sp != hlt[id_S - 1].sg_rgb_sp) {
    hlt[hlcnt + i].sg_rgb_sp = hlt[id - 1].sg_rgb_sp;
  }
  highlight_ga.ga_len = hlcnt + i + 1;
  set_hl_attr(hlcnt + i);  // At long last we can apply
  table[i] = syn_id2attr(hlcnt + i + 1);
}

/// Translate highlight groups into attributes in highlight_attr[] and set up
/// the user highlights User1..9. A set of corresponding highlights to use on
/// top of HLF_SNC is computed.  Called only when nvim starts and upon first
/// screen redraw after any :highlight command.
void highlight_changed(void)
{
  char userhl[30];  // use 30 to avoid compiler warning
  int id_S = -1;
  int id_SNC = 0;

  need_highlight_changed = false;

  // sentinel value. used when no highlight is active
  highlight_attr[HLF_NONE] = 0;

  /// Translate builtin highlight groups into attributes for quick lookup.
  for (int hlf = 1; hlf < HLF_COUNT; hlf++) {
    int id = syn_check_group(hlf_names[hlf], strlen(hlf_names[hlf]));
    if (id == 0) {
      abort();
    }
    int ns_id = -1;
    int final_id = id;
    syn_ns_get_final_id(&ns_id, &final_id);
    if (hlf == HLF_SNC) {
      id_SNC = final_id;
    } else if (hlf == HLF_S) {
      id_S = final_id;
    }

    highlight_attr[hlf] = hl_get_ui_attr(ns_id, hlf, final_id, hlf == HLF_INACTIVE);

    if (highlight_attr[hlf] != highlight_attr_last[hlf]) {
      if (hlf == HLF_MSG) {
        clear_cmdline = true;
        HlAttrs attrs = syn_attr2entry(highlight_attr[hlf]);
        msg_grid.blending = attrs.hl_blend > -1;
      }
      ui_call_hl_group_set(cstr_as_string(hlf_names[hlf]),
                           highlight_attr[hlf]);
      highlight_attr_last[hlf] = highlight_attr[hlf];
    }
  }

  // Setup the user highlights
  //
  // Temporarily utilize 10 more hl entries:
  // 9 for User1-User9 combined with StatusLineNC
  // 1 for StatusLine default
  // Must to be in there simultaneously in case of table overflows in
  // get_attr_entry()
  ga_grow(&highlight_ga, 10);
  int hlcnt = highlight_ga.ga_len;
  if (id_S == -1) {
    // Make sure id_S is always valid to simplify code below. Use the last entry
    CLEAR_POINTER(&hl_table[hlcnt + 9]);
    id_S = hlcnt + 10;
  }
  for (int i = 0; i < 9; i++) {
    snprintf(userhl, sizeof(userhl), "User%d", i + 1);
    int id = syn_name2id(userhl);
    if (id == 0) {
      highlight_user[i] = 0;
      highlight_stlnc[i] = 0;
    } else {
      highlight_user[i] = syn_id2attr(id);
      combine_stl_hlt(id, id_S, id_SNC, hlcnt, i, HLF_SNC, highlight_stlnc);
    }
  }
  highlight_ga.ga_len = hlcnt;

  decor_provider_invalidate_hl();
}

/// Handle command line completion for :highlight command.
void set_context_in_highlight_cmd(expand_T *xp, const char *arg)
{
  // Default: expand group names.
  xp->xp_context = EXPAND_HIGHLIGHT;
  xp->xp_pattern = (char *)arg;
  include_link = 2;
  include_default = 1;

  if (*arg == NUL) {
    return;
  }

  // (part of) subcommand already typed
  const char *p = skiptowhite(arg);
  if (*p == NUL) {
    return;
  }

  // past "default" or group name
  include_default = 0;
  if (strncmp("default", arg, (unsigned)(p - arg)) == 0) {
    arg = skipwhite(p);
    xp->xp_pattern = (char *)arg;
    p = skiptowhite(arg);
  }
  if (*p == NUL) {
    return;
  }

  // past group name
  include_link = 0;
  if (arg[1] == 'i' && arg[0] == 'N') {
    highlight_list();
  }
  if (strncmp("link", arg, (unsigned)(p - arg)) == 0
      || strncmp("clear", arg, (unsigned)(p - arg)) == 0) {
    xp->xp_pattern = skipwhite(p);
    p = skiptowhite(xp->xp_pattern);
    if (*p != NUL) {  // past first group name
      xp->xp_pattern = skipwhite(p);
      p = skiptowhite(xp->xp_pattern);
    }
  }
  if (*p != NUL) {  // past group name(s)
    xp->xp_context = EXPAND_NOTHING;
  }
}

/// List highlighting matches in a nice way.
static void highlight_list(void)
{
  for (int i = 10; --i >= 0;) {
    highlight_list_two(i, HLF_D);
  }
  for (int i = 40; --i >= 0;) {
    highlight_list_two(99, 0);
  }
}

static void highlight_list_two(int cnt, int id)
{
  msg_puts_hl(&("N \bI \b!  \b"[cnt / 11]), id, false);
  msg_clr_eos();
  ui_flush();
  os_delay(cnt == 99 ? 40 : (uint64_t)cnt * 50, false);
}

/// Function given to ExpandGeneric() to obtain the list of group names.
char *get_highlight_name(expand_T *const xp, int idx)
  FUNC_ATTR_WARN_UNUSED_RESULT
{
  return (char *)get_highlight_name_ext(xp, idx, true);
}

/// Obtain a highlight group name.
///
/// @param skip_cleared  if true don't return a cleared entry.
const char *get_highlight_name_ext(expand_T *xp, int idx, bool skip_cleared)
  FUNC_ATTR_WARN_UNUSED_RESULT
{
  if (idx < 0) {
    return NULL;
  }

  // Items are never removed from the table, skip the ones that were cleared.
  if (skip_cleared && idx < highlight_ga.ga_len && hl_table[idx].sg_cleared) {
    return "";
  }

  if (idx == highlight_ga.ga_len && include_none != 0) {
    return "none";
  } else if (idx == highlight_ga.ga_len + include_none
             && include_default != 0) {
    return "default";
  } else if (idx == highlight_ga.ga_len + include_none + include_default
             && include_link != 0) {
    return "link";
  } else if (idx == highlight_ga.ga_len + include_none + include_default + 1
             && include_link != 0) {
    return "clear";
  } else if (idx >= highlight_ga.ga_len) {
    return NULL;
  }
  return hl_table[idx].sg_name;
}

