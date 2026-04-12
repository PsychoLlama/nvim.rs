// drawline.c: Functions for drawing window lines on the screen.
// This is the middle level, drawscreen.c is the top and grid.c the lower level.

#include <assert.h>
#include <limits.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/cursor_shape.h"
#include "nvim/decoration.h"
#include "nvim/decoration_defs.h"
#include "nvim/decoration_provider.h"
#include "nvim/diff.h"
#include "nvim/drawline.h"
#include "nvim/drawscreen.h"
#include "nvim/eval/vars.h"
#include "nvim/fold.h"
#include "nvim/fold_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/grid_defs.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/indent.h"
#include "nvim/insexpand.h"
#include "nvim/mark_defs.h"
#include "nvim/marktree_defs.h"
#include "nvim/match.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/os_defs.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/quickfix.h"
#include "nvim/sign_defs.h"
#include "nvim/spell.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/statusline.h"
#include "nvim/statusline_defs.h"
#include "nvim/strings.h"
#include "nvim/syntax.h"
#include "nvim/terminal.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/vim_defs.h"

#define MB_FILLER_CHAR '<'  // character used when a double-width character doesn't fit.

/// structure with variables passed between win_line() and other functions
typedef struct {
  const linenr_T lnum;       ///< line number to be drawn
  const foldinfo_T foldinfo;  ///< fold info for this line

  const int startrow;        ///< first row in the window to be drawn
  int row;                   ///< row in the window, excl w_winrow

  colnr_T vcol;              ///< virtual column, before wrapping
  int col;                   ///< visual column on screen, after wrapping
  int boguscols;             ///< nonexistent columns added to "col" to force wrapping
  int old_boguscols;         ///< bogus boguscols
  int vcol_off_co;           ///< offset for concealed characters

  int off;                   ///< offset relative start of line

  int cul_attr;              ///< set when 'cursorline' active
  int line_attr;             ///< attribute for the whole line
  int line_attr_lowprio;     ///< low-priority attribute for the line
  int sign_num_attr;         ///< line number attribute (sign numhl)
  int prev_num_attr;         ///< previous line's number attribute (sign numhl)
  int sign_cul_attr;         ///< cursorline sign attribute (sign culhl)

  int fromcol;               ///< start of inverting
  int tocol;                 ///< end of inverting

  colnr_T vcol_sbr;          ///< virtual column after showbreak
  bool need_showbreak;       ///< overlong line, skipping first x chars

  int char_attr;             ///< attributes for next character

  int n_extra;               ///< number of extra bytes
  int n_attr;                ///< chars with special attr
  char *p_extra;             ///< string of extra chars, plus NUL, only used
                             ///< when sc_extra and sc_final are NUL
  int extra_attr;            ///< attributes for p_extra
  schar_T sc_extra;          ///< extra chars, all the same
  schar_T sc_final;          ///< final char, mandatory if set

  bool extra_for_extmark;    ///< n_extra set for inline virtual text

  char extra[11];            ///< must be as large as transchar_charbuf[] in charset.c

  hlf_T diff_hlf;            ///< type of diff highlighting

  int n_virt_lines;          ///< nr of virtual lines
  int n_virt_below;          ///< nr of virtual lines belonging to previous line
  int filler_lines;          ///< nr of filler lines to be drawn
  int filler_todo;           ///< nr of filler lines still to do + 1
  SignTextAttrs sattrs[SIGN_SHOW_MAX];  ///< sign attributes for the sign column
  /// do consider wrapping in linebreak mode only after encountering
  /// a non whitespace char
  bool need_lbr;

  VirtText virt_inline;
  size_t virt_inline_i;
  HlMode virt_inline_hl_mode;

  bool reset_extra_attr;

  int skip_cells;            ///< nr of cells to skip for w_leftcol
                             ///< or w_skipcol or concealing
  int skipped_cells;         ///< nr of skipped cells for virtual text
                             ///< to be added to wlv.vcol later

  int *color_cols;           ///< if not NULL, highlight colorcolumn using according columns array
} winlinevars_T;

// Forward declarations for types used by c_win_line_pre_loop.
// Full definitions are below (after the generated header).
typedef struct WinLineState_s WinLineState;

/// Return type from c_win_line_pre_loop (forward-declared here for generated header).
typedef struct {
  int ptr_offset;
  colnr_T trailcol;
  colnr_T leadcol;
  bool lcs_eol_todo;
  schar_T lcs_eol;
  schar_T lcs_prec_todo;
  int start_vcol;
  bool may_have_inline_virt;
  int virt_line_index;
  int virt_line_flags;
  bool draw_cols;
  int leftcols_width;
  bool statuscol_draw;
  int statuscol_width;
  int statuscol_sign_cul_id;
} PreLoopResult;

/// Return type from rs_win_line_highlight_attrs (Phase 2 migration).
typedef struct {
  int extmark_attr;
  int has_match_conc;
} HighlightResult;

#include "drawline.c.generated.h"

// Rust implementations (rs_* names, called from win_line via generated extern decls)
extern int rs_ins_compl_win_active(win_T *wp);
extern int rs_ins_compl_lnum_in_range(int lnum);
extern const char *rs_get_showbreak_value(win_T *win);
extern int rs_diff_check_with_linestatus(win_T *wp, linenr_T lnum, int *linestatus);
extern bool rs_diff_find_change(win_T *wp, linenr_T lnum, diffline_T *diffline);
extern bool rs_diff_change_parse(diffline_T *diffline, diffline_change_T *change,
                                 int *change_start, int *change_end);
extern HighlightResult rs_win_line_highlight_attrs(win_T *wp, winlinevars_T *wlv,
                                                   WinLineState *state, colnr_T ptr_col,
                                                   bool lcs_eol_todo, bool may_have_inline_virt,
                                                   linenr_T lnum, void *screen_search_hl);

// Rust-exported functions (export_name = original C name, called directly from C)
extern schar_T get_lcs_ext(win_T *wp);
extern void advance_color_col(winlinevars_T *wlv, int vcol);
extern void margin_columns_win(win_T *wp, int *left_col, int *right_col);
extern int line_putchar(buf_T *buf, const char **pp, schar_T *dest, int maxcells, int vcol);
extern void draw_virt_text(win_T *wp, buf_T *buf, int col_off, int *end_col, int win_row);
extern int draw_virt_text_item(buf_T *buf, int col, VirtText vt, HlMode hl_mode, int max_col,
                               int vcol, int skip_cells);
extern void draw_col_buf(win_T *wp, winlinevars_T *wlv, const char *text, size_t len, int attr,
                         const colnr_T *fold_vcol, bool inc_vcol);
extern void draw_col_fill(winlinevars_T *wlv, schar_T fillchar, int width, int attr);
extern bool use_cursor_line_highlight(win_T *wp, linenr_T lnum);
extern void draw_foldcolumn(win_T *wp, winlinevars_T *wlv);
extern void fill_foldcolumn(win_T *wp, foldinfo_T foldinfo, linenr_T lnum, int attr, int fdc,
                            int *wlv_off, colnr_T *out_vcol, schar_T *out_buffer);
extern void draw_sign(bool nrcol, win_T *wp, winlinevars_T *wlv, int sign_idx);
extern bool use_cursor_line_nr(win_T *wp, winlinevars_T *wlv);
extern int get_line_number_attr(win_T *wp, winlinevars_T *wlv);
extern void draw_lnum_col(win_T *wp, winlinevars_T *wlv);
extern void handle_breakindent(win_T *wp, winlinevars_T *wlv);
extern void handle_showbreak_and_filler(win_T *wp, winlinevars_T *wlv);
extern void apply_cursorline_highlight(win_T *wp, winlinevars_T *wlv);
extern void set_line_attr_for_diff(win_T *wp, winlinevars_T *wlv);
extern bool has_more_inline_virt(winlinevars_T *wlv, ptrdiff_t v);
extern void handle_inline_virtual_text(win_T *wp, winlinevars_T *wlv, ptrdiff_t v, bool selected);
extern void win_line_start(win_T *wp, winlinevars_T *wlv);
extern void fix_for_boguscols(winlinevars_T *wlv);
extern int get_rightmost_vcol(win_T *wp, const int *color_cols);
extern void wlv_put_linebuf(win_T *wp, const winlinevars_T *wlv, int endcol, bool clear_end,
                            int bg_attr, int flags);

/// Return type of rs_get_foldtext (matches Rust FoldTextResult repr(C)).
typedef struct {
  char *text;            ///< pointer to fold text (may point into buf or be heap-allocated)
  bool text_is_allocated;  ///< true if text must be xfree'd
  bool has_virt_text;    ///< true if fold_vt was populated
} FoldTextResult;

extern FoldTextResult rs_get_foldtext(win_T *wp, linenr_T lnum, linenr_T lnume,
                                      int fi_level, char *buf, VirtText *vt_out);

// Rust-exported scratch buffer (replaces get_extra_buf).
extern char *rs_get_extra_buf(size_t size);

// Rust-exported wrappers for draw_statuscol, decor_providers_setup, invoke_range_next.
extern void rs_draw_statuscol(win_T *wp, winlinevars_T *wlv, int virtnum, int col_rows,
                              statuscol_T *stcp);
extern int rs_decor_providers_setup(int rows_to_draw, bool draw_from_line_start, linenr_T lnum,
                                    colnr_T col, win_T *wp);
extern int rs_invoke_range_next(win_T *wp, int lnum, colnr_T begin_col, colnr_T col_off);

/// State struct returned by rs_win_line_init. Mirrors Rust WinLineState (repr(C)).
/// Must match the Rust struct layout exactly.
typedef struct WinLineState_s {
  colnr_T vcol_prev;
  int fromcol_prev;
  bool noinvcur;
  bool lnum_in_visual_area;
  int char_attr_pri;
  int char_attr_base;
  bool area_highlighting;
  int vi_attr;
  int area_attr;
  int search_attr;
  int vcol_save_attr;
  int decor_attr;
  bool has_syntax;
  int folded_attr;
  int eol_hl_off;
  char nextline[300];   // SPWORDLEN*2
  int nextlinecol;
  int nextline_idx;
  int spell_attr;
  int word_end;
  int cur_checked_col;
  bool extra_check;
  int multi_attr;
  int mb_l;
  int mb_c;
  schar_T mb_schar;
  int change_start;
  int change_end;
  bool in_multispace;
  int multispace_pos;
  int n_extra_next;
  int extra_attr_next;
  bool search_attr_from_match;
  bool has_decor;
  int saved_search_attr;
  int saved_area_attr;
  int saved_decor_attr;
  bool saved_search_attr_from_match;
  int win_col_offset;
  bool area_active;
  bool decor_need_recheck;
  char buf_fold[51];  // FOLD_TEXT_LEN
  VirtText fold_vt;
  char *foldtext_free;
  bool cul_screenline;
  int left_curline_col;
  int right_curline_col;
  int match_conc;
  bool on_last_col;
  int syntax_flags;
  int syntax_seqnr;
  int prev_syntax_id;
  int conceal_attr;
  bool is_concealing;
  bool did_wcol;
  int bg_attr;
  bool draw_text;
  bool has_fold;
  bool has_foldtext;
  bool is_wrapped;
  bool in_curline;
  int view_width;
  int view_height;
  int line_attr_save;
  int line_attr_lowprio_save;
  bool check_decor_providers;
  int decor_provider_end_col;
  int linestatus;
  int change_index;
  diffline_T line_changes;
  VirtLines virt_lines;
  int saved_attr2;
  int n_attr3;
  int saved_attr3;
} WinLineState;  // struct tag: WinLineState_s

_Static_assert(sizeof(WinLineState) == 688, "WinLineState size mismatch with Rust");

/// Initialize line state variables for win_line() (lines 232-610).
extern void rs_win_line_init(win_T *wp, linenr_T lnum, int startrow, int col_rows,
                             bool concealed, spellvars_T *spv, foldinfo_T foldinfo,
                             winlinevars_T *wlv, WinLineState *state);

/// Pre-loop setup for win_line (Rust implementation of c_win_line_pre_loop).
extern PreLoopResult rs_c_win_line_pre_loop(win_T *wp, linenr_T lnum, winlinevars_T *wlv,
                                            WinLineState *wls, spellvars_T *spv,
                                            foldinfo_T foldinfo, int startrow, int endrow,
                                            int col_rows, int *term_attrs);

// Phase function declarations (Rust implementations)

/// Return type for rs_win_line_draw_cols (action codes + updated state).
typedef struct {
  int action;           ///< DRAW_COLS_ACTION_* constant
  bool draw_cols;       ///< updated draw_cols flag
  int leftcols_width;   ///< updated leftcols_width
  int virt_line_index;  ///< updated virt_line_index
  int virt_line_flags;  ///< updated virt_line_flags
  int win_col_offset;   ///< updated win_col_offset
  int ptr_offset;       ///< updated ptr offset
} DrawColsResult;

/// Action codes returned by rs_win_line_draw_cols.
#define DRAW_COLS_ACTION_FALLTHROUGH 0
#define DRAW_COLS_ACTION_BREAK       1
#define DRAW_COLS_ACTION_CONTINUE    2
#define DRAW_COLS_ACTION_GOTO_END_CHECK 3

/// Draw the columns (fold, sign, number, statuscolumn) for this row.
extern DrawColsResult rs_win_line_draw_cols(win_T *wp, linenr_T lnum, winlinevars_T *wlv,
                                            const WinLineState *wls, statuscol_T *statuscol,
                                            bool statuscol_draw, VirtLines *virt_lines,
                                            int ptr_col, int startrow, int endrow, int col_rows,
                                            int virt_line_index, int virt_line_flags,
                                            int leftcols_width, int win_col_offset,
                                            bool draw_text, bool has_decor, int bg_attr);

/// Return value for rs_win_line_process_n_extra.
typedef struct { schar_T mb_schar; int mb_c; int mb_l; } NExtraResult;

/// Return value for rs_win_line_process_char.
/// Must match Rust ProcessCharResult (repr(C)).
typedef struct {
  schar_T mb_schar;       ///< Display character (schar_T).
  int mb_c;               ///< Codepoint of the character.
  int mb_l;               ///< Byte length of the character.
  int ptr_col;            ///< Updated ptr byte offset (ptr - line).
  int prev_ptr_col;       ///< Updated prev_ptr byte offset.
  bool did_decrement_ptr; ///< Whether ptr was decremented.
  bool lcs_eol_todo;      ///< Whether EOL listchar is still to do.
  int search_attr;        ///< Updated search highlight attr.
  int decor_attr;         ///< Updated decoration attr.
} ProcessCharResult;

/// Phase 3: Process one character from the buffer line.
extern ProcessCharResult rs_win_line_process_char(win_T *wp, linenr_T lnum,
                                                  winlinevars_T *wlv, WinLineState *wls,
                                                  spellvars_T *spv, const char *line,
                                                  int ptr_col, colnr_T trailcol, colnr_T leadcol,
                                                  int extmark_attr, bool lcs_eol_todo,
                                                  int search_attr, int area_attr,
                                                  const int *term_attrs, int has_match_conc);

/// Full Rust implementation of win_line (Phase 4).
extern int rs_win_line(win_T *wp, linenr_T lnum, int startrow, int endrow, int col_rows,
                       bool concealed, spellvars_T *spv, foldinfo_T foldinfo);

// Phase 1: EOL highlight + fill + cursorcolumn
extern int rs_win_line_eol_highlight(win_T *wp, winlinevars_T *wlv, const WinLineState *state,
                                     bool lcs_eol_todo, int area_attr, colnr_T ptr_col,
                                     void *screen_search_hl);
extern bool rs_win_line_eol_fill(win_T *wp, winlinevars_T *wlv, WinLineState *state,
                                 int start_vcol, bool lcs_eol_todo, int eol_hl_off,
                                 const int *term_attrs, bool has_decor);
extern int rs_win_line_cursorcolumn(win_T *wp, winlinevars_T *wlv, bool lnum_in_visual_area,
                                    int search_attr, int area_attr);

// Phase 2: Store char + post-store
extern void rs_win_line_store_char(win_T *wp, winlinevars_T *wlv, const WinLineState *state,
                                   schar_T mb_schar, int multi_attr, bool is_concealing);
extern void rs_win_line_post_store(win_T *wp, winlinevars_T *wlv, WinLineState *state,
                                   int vcol_save_attr, colnr_T ptr_col);

// Phase 3: End-check / wrap
extern bool rs_win_line_end_check(win_T *wp, winlinevars_T *wlv, WinLineState *state,
                                  int endrow, int leftcols_width,
                                  int virt_line_index, int virt_line_flags,
                                  bool ptr_is_nul, bool lcs_eol_todo,
                                  void *virt_lines_ptr, int bg_attr,
                                  bool *statuscol_draw, bool *draw_cols_out,
                                  int *virt_line_index_out, int *virt_line_flags_out,
                                  schar_T *lcs_prec_todo_out);

// Phase 4: Extra attr restore + extends + cursor conceal
extern schar_T rs_win_line_extra_attr_restore(win_T *wp, winlinevars_T *wlv, WinLineState *state,
                                              schar_T lcs_prec_todo);
extern void rs_win_line_extends_char(win_T *wp, winlinevars_T *wlv, WinLineState *state,
                                     colnr_T ptr_col, bool ptr_is_nul,
                                     schar_T lcs_eol, bool lcs_eol_todo,
                                     bool may_have_inline_virt);
extern void rs_win_line_cursor_conceal_correct(win_T *wp, winlinevars_T *wlv, WinLineState *state,
                                               bool in_curline, colnr_T ptr_col);

// Phase 5: N_extra processing
extern NExtraResult rs_win_line_process_n_extra(win_T *wp, winlinevars_T *wlv, WinLineState *state,
                                                bool ptr_is_nul);

// Layout verification: PreLoopResult must match Rust PreLoopResult.
_Static_assert(sizeof(PreLoopResult) == 60, "PreLoopResult size mismatch with Rust");

// Layout verification: WinLineVars (Rust repr(C)) must match winlinevars_T (C).
// If any of these fail, update src/nvim-rs/drawline/src/lib.rs accordingly.
#include <stddef.h>
_Static_assert(sizeof(winlinevars_T) > 0, "winlinevars_T must be non-empty");
_Static_assert(offsetof(winlinevars_T, lnum) == 0, "lnum offset mismatch");
_Static_assert(offsetof(winlinevars_T, row) > offsetof(winlinevars_T, startrow), "row/startrow order");
_Static_assert(offsetof(winlinevars_T, off) > 0, "off field must not be first");
_Static_assert(offsetof(winlinevars_T, virt_inline) > offsetof(winlinevars_T, sattrs), "virt_inline after sattrs");
_Static_assert(sizeof(winlinevars_T) == sizeof(winlinevars_T), "size check placeholder");


// win_extmark_arr accessor functions for Rust

/// Push a WinExtmark to win_extmark_arr.
void nvim_win_extmark_push(uint64_t ns_id, uint64_t mark_id, int win_row, int win_col)
{
  WinExtmark m = { (NS)ns_id, mark_id, win_row, win_col };
  kv_push(win_extmark_arr, m);
}

/// Display line "lnum" of window "wp" on the screen.
/// wp->w_virtcol needs to be valid.
///
/// @param lnum         line to display
/// @param startrow     first row relative to window grid
/// @param endrow       last grid row to be redrawn
/// @param col_rows     set to the height of the line when only updating the columns,
///                     otherwise set to 0
/// @param concealed    only draw virtual lines belonging to the line above
/// @param spv          'spell' related variables kept between calls for "wp"
/// @param foldinfo     fold info for this line
/// @param[in, out] providers  decoration providers active this line
///                            items will be disables if they cause errors
///                            or explicitly return `false`.
///
/// @return             the number of last row the line occupies.
int win_line(win_T *wp, linenr_T lnum, int startrow, int endrow, int col_rows, bool concealed,
             spellvars_T *spv, foldinfo_T foldinfo)
{
  return rs_win_line(wp, lnum, startrow, endrow, col_rows, concealed, spv, foldinfo);
}

// DEAD CODE: the following is the old C implementation, kept for reference
// and deleted in Phase 5. This code is never compiled (guarded by #if 0).
#if 0
int win_line_old(win_T *wp, linenr_T lnum, int startrow, int endrow, int col_rows, bool concealed,
             spellvars_T *spv, foldinfo_T foldinfo)
{
  // Initialize winlinevars_T
  assert(startrow < endrow);
  winlinevars_T wlv = {
    .lnum = lnum,
    .foldinfo = foldinfo,
    .startrow = startrow,
    .row = startrow,
    .fromcol = -10,
    .tocol = MAXCOL,
    .vcol_sbr = -1,
    .old_boguscols = 0,
    .prev_num_attr = -1,
  };

  // Delegate initialization to Rust (lines 232-610 of original C)
  WinLineState wls;
  rs_win_line_init(wp, lnum, startrow, col_rows, concealed, spv, foldinfo, &wlv, &wls);

  // Unpack WinLineState into local C variables for the rest of the function
  GridView *grid = &wp->w_grid;       // grid specific to the window
  const int view_width = wls.view_width;
  const int view_height = wls.view_height;
  const bool in_curline = wls.in_curline;
  const bool has_fold = wls.has_fold;
  const bool has_foldtext = wls.has_foldtext;
  const bool is_wrapped = wls.is_wrapped;
  const bool draw_text = wls.draw_text;
  buf_T *buf = wp->w_buffer;  // still need buf for ml_get_buf etc.

#define SPWORDLEN 150
#define vcol_hlc(wlv) ((wlv).vcol - (wlv).vcol_off_co)

  colnr_T vcol_prev = wls.vcol_prev;
  int fromcol_prev = wls.fromcol_prev;
  bool noinvcur = wls.noinvcur;
  bool lnum_in_visual_area = wls.lnum_in_visual_area;
  int char_attr_pri = wls.char_attr_pri;
  int char_attr_base = wls.char_attr_base;
  bool area_highlighting = wls.area_highlighting;
  int vi_attr = wls.vi_attr;
  int area_attr = wls.area_attr;
  int search_attr = wls.search_attr;
  int vcol_save_attr = wls.vcol_save_attr;
  int decor_attr = wls.decor_attr;
  bool has_syntax = wls.has_syntax;
  int folded_attr = wls.folded_attr;
  int eol_hl_off = wls.eol_hl_off;
  // nextline[] lives in WinLineState; create a pointer alias for the remaining C code
  char *nextline = wls.nextline;
  int nextlinecol = wls.nextlinecol;
  int nextline_idx = wls.nextline_idx;
  int spell_attr = wls.spell_attr;
  int word_end = wls.word_end;
  int cur_checked_col = wls.cur_checked_col;
  bool extra_check = wls.extra_check;
  int multi_attr = wls.multi_attr;
  int mb_l = wls.mb_l;
  int mb_c = wls.mb_c;
  schar_T mb_schar = wls.mb_schar;
  int change_start = wls.change_start;
  int change_end = wls.change_end;
  bool in_multispace = wls.in_multispace;
  int multispace_pos = wls.multispace_pos;
  int n_extra_next = wls.n_extra_next;
  int extra_attr_next = wls.extra_attr_next;
  bool search_attr_from_match = wls.search_attr_from_match;
  bool has_decor = wls.has_decor;
  int saved_search_attr = wls.saved_search_attr;
  int saved_area_attr = wls.saved_area_attr;
  int saved_decor_attr = wls.saved_decor_attr;
  bool saved_search_attr_from_match = wls.saved_search_attr_from_match;
  int win_col_offset = wls.win_col_offset;
  bool area_active = wls.area_active;
  bool decor_need_recheck = wls.decor_need_recheck;
  // buf_fold[] lives in WinLineState; create a pointer alias for the remaining C code
  char *buf_fold = wls.buf_fold;
  VirtText fold_vt = wls.fold_vt;
  char *foldtext_free = wls.foldtext_free;
  bool cul_screenline = wls.cul_screenline;
  int left_curline_col = wls.left_curline_col;
  int right_curline_col = wls.right_curline_col;
  int match_conc = wls.match_conc;
  bool on_last_col = wls.on_last_col;
  int syntax_flags = wls.syntax_flags;
  int syntax_seqnr = wls.syntax_seqnr;
  int prev_syntax_id = wls.prev_syntax_id;
  int conceal_attr = wls.conceal_attr;
  bool is_concealing = wls.is_concealing;
  bool did_wcol = wls.did_wcol;
  int bg_attr = wls.bg_attr;
  int saved_attr2 = wls.saved_attr2;
  int n_attr3 = wls.n_attr3;
  int saved_attr3 = wls.saved_attr3;
  int line_attr_save = wls.line_attr_save;
  int line_attr_lowprio_save = wls.line_attr_lowprio_save;
  int linestatus = wls.linestatus;
  diffline_T line_changes = wls.line_changes;
  int change_index = wls.change_index;
  VirtLines virt_lines = wls.virt_lines;
  bool check_decor_providers = wls.check_decor_providers;
  int decor_provider_end_col = wls.decor_provider_end_col;

  // Pre-loop setup: delegate to Rust rs_c_win_line_pre_loop.
  // term_attrs is stack-allocated here so C controls its lifetime.
  int term_attrs[TERM_ATTRS_MAX] = { 0 };
  PreLoopResult plr = rs_c_win_line_pre_loop(wp, lnum, &wlv, &wls, spv, foldinfo,
                                             startrow, endrow, col_rows, term_attrs);

  // Reconstruct statuscol from PreLoopResult (sattrs points into wlv, foldinfo is local).
  statuscol_T statuscol = { 0 };
  if (plr.statuscol_draw) {
    statuscol.draw = true;
    statuscol.sattrs = wlv.sattrs;
    statuscol.foldinfo = foldinfo;
    statuscol.width = plr.statuscol_width;
    statuscol.sign_cul_id = plr.statuscol_sign_cul_id;
  }

  // Re-fetch line pointer (c_win_line_pre_loop may have called ml_get_buf internally).
  char *line = draw_text ? ml_get_buf(wp->w_buffer, lnum) : "";
  char *ptr = line + plr.ptr_offset;
  const int start_vcol = plr.start_vcol;
  colnr_T trailcol = plr.trailcol;
  colnr_T leadcol = plr.leadcol;
  bool lcs_eol_todo = plr.lcs_eol_todo;
  const schar_T lcs_eol = plr.lcs_eol;
  schar_T lcs_prec_todo = plr.lcs_prec_todo;
  const bool may_have_inline_virt = plr.may_have_inline_virt;
  int virt_line_index = plr.virt_line_index;
  int virt_line_flags = plr.virt_line_flags;
  bool draw_cols = plr.draw_cols;
  int leftcols_width = plr.leftcols_width;

  // Re-sync local variables that c_win_line_pre_loop may have modified in wls:
  extra_check = wls.extra_check;
  has_decor = wls.has_decor;
  area_highlighting = wls.area_highlighting;
  search_attr = wls.search_attr;
  search_attr_from_match = wls.search_attr_from_match;
  in_multispace = wls.in_multispace;
  multispace_pos = wls.multispace_pos;
  word_end = wls.word_end;
  spell_attr = wls.spell_attr;
  fromcol_prev = wls.fromcol_prev;
  check_decor_providers = wls.check_decor_providers;
  decor_provider_end_col = wls.decor_provider_end_col;

  // Repeat for the whole displayed line.
  while (true) {
    int has_match_conc = 0;  ///< match wants to conceal

    bool did_decrement_ptr = false;

    // Get next chunk of extmark highlights if previous approximation was smaller than needed.
    if (check_decor_providers && (int)(ptr - line) >= decor_provider_end_col) {
      int const col = (int)(ptr - line);
      decor_provider_end_col = rs_invoke_range_next(wp, lnum, col, 100);
      line = ml_get_buf(wp->w_buffer, lnum);
      ptr = line + col;
      if (!has_decor && decor_has_more_decorations(&decor_state, lnum - 1)) {
        has_decor = true;
        extra_check = true;
      }
    }

    // Delegate the draw_cols block to Rust.
    if (draw_cols) {
      DrawColsResult dcr = rs_win_line_draw_cols(
        wp, lnum, &wlv, &wls, &statuscol, statuscol.draw, &virt_lines,
        (int)(ptr - line), startrow, endrow, col_rows,
        virt_line_index, virt_line_flags, leftcols_width, win_col_offset,
        draw_text, has_decor, bg_attr);
      // Apply outputs
      draw_cols = dcr.draw_cols;
      leftcols_width = dcr.leftcols_width;
      virt_line_index = dcr.virt_line_index;
      virt_line_flags = dcr.virt_line_flags;
      win_col_offset = dcr.win_col_offset;
      // Re-fetch line pointer: rs_win_line_draw_cols may have called ml_get_buf.
      line = draw_text ? ml_get_buf(wp->w_buffer, lnum) : "";
      ptr = line + dcr.ptr_offset;
      // Dispatch control flow
      if (dcr.action == DRAW_COLS_ACTION_BREAK) {
        break;
      } else if (dcr.action == DRAW_COLS_ACTION_CONTINUE) {
        continue;
      } else if (dcr.action == DRAW_COLS_ACTION_GOTO_END_CHECK) {
        goto end_check;
      }
    }

    if (cul_screenline && wlv.filler_todo <= 0
        && wlv.vcol >= left_curline_col && wlv.vcol < right_curline_col) {
      apply_cursorline_highlight(wp, &wlv);
    }

    // When still displaying '$' of change command, stop at cursor.
    if (dollar_vcol >= 0 && in_curline && wlv.vcol >= wp->w_virtcol) {
      draw_virt_text(wp, buf, win_col_offset, &wlv.col, wlv.row);
      // don't clear anything after wlv.col
      wlv_put_linebuf(wp, &wlv, wlv.col, false, bg_attr, 0);
      // Pretend we have finished updating the window.  Except when
      // 'cursorcolumn' is set.
      if (wp->w_p_cuc) {
        wlv.row = wp->w_cline_row + wp->w_cline_height;
      } else {
        wlv.row = view_height;
      }
      break;
    }

    const bool draw_folded = has_fold && wlv.row == startrow + wlv.filler_lines;
    if (draw_folded && wlv.n_extra == 0) {
      wlv.char_attr = folded_attr = win_hl_attr(wp, HLF_FL);
      decor_attr = 0;
    }

    int extmark_attr = 0;
    if (wlv.filler_todo <= 0
        && (area_highlighting || spv->spv_has_spell || extra_check)) {
      // Sync C locals into wls so Rust can read/write them.
      wls.area_attr = area_attr;
      wls.search_attr = search_attr;
      wls.decor_attr = decor_attr;
      wls.char_attr_pri = char_attr_pri;
      wls.char_attr_base = char_attr_base;
      wls.search_attr_from_match = search_attr_from_match;
      wls.on_last_col = on_last_col;
      wls.area_active = area_active;
      wls.decor_need_recheck = decor_need_recheck;
      wls.saved_search_attr = saved_search_attr;
      wls.saved_area_attr = saved_area_attr;
      wls.saved_decor_attr = saved_decor_attr;
      wls.saved_search_attr_from_match = saved_search_attr_from_match;
      wls.match_conc = match_conc;
      wls.folded_attr = folded_attr;
      wls.change_index = change_index;
      wls.line_changes = line_changes;
      wls.change_start = change_start;
      wls.change_end = change_end;
      wls.vcol_prev = vcol_prev;
      wls.fromcol_prev = fromcol_prev;

      const colnr_T ptr_col = (colnr_T)(ptr - line);
      HighlightResult hlr = rs_win_line_highlight_attrs(wp, &wlv, &wls, ptr_col,
                                                        lcs_eol_todo, may_have_inline_virt,
                                                        lnum, &screen_search_hl);
      extmark_attr = hlr.extmark_attr;
      has_match_conc = hlr.has_match_conc;

      // Sync wls back to C locals.
      area_attr = wls.area_attr;
      search_attr = wls.search_attr;
      char_attr_pri = wls.char_attr_pri;
      char_attr_base = wls.char_attr_base;
      search_attr_from_match = wls.search_attr_from_match;
      on_last_col = wls.on_last_col;
      area_active = wls.area_active;
      decor_need_recheck = wls.decor_need_recheck;
      saved_search_attr = wls.saved_search_attr;
      saved_area_attr = wls.saved_area_attr;
      saved_decor_attr = wls.saved_decor_attr;
      saved_search_attr_from_match = wls.saved_search_attr_from_match;
      match_conc = wls.match_conc;
      change_index = wls.change_index;
      change_start = wls.change_start;
      change_end = wls.change_end;
      line_changes = wls.line_changes;

      // Re-fetch line pointer: rs_win_line_highlight_attrs calls update_search_hl
      // which may have reallocated the line buffer.
      line = draw_text ? ml_get_buf(wp->w_buffer, lnum) : "";
      ptr = line + ptr_col;
    }

    if (draw_folded && has_foldtext && wlv.n_extra == 0 && wlv.col == win_col_offset) {
      const int v = (int)(ptr - line);
      linenr_T lnume = lnum + foldinfo.fi_lines - 1;
      memset(buf_fold, ' ', FOLD_TEXT_LEN);
      FoldTextResult ftr = rs_get_foldtext(wp, lnum, lnume, foldinfo.fi_level, buf_fold, &fold_vt);
      wlv.p_extra = ftr.text;
      wlv.n_extra = (int)strlen(wlv.p_extra);

      if (ftr.text_is_allocated) {
        assert(foldtext_free == NULL);
        foldtext_free = wlv.p_extra;
      }
      wlv.sc_extra = NUL;
      wlv.sc_final = NUL;
      wlv.p_extra[wlv.n_extra] = NUL;

      // Get the line again as evaluating 'foldtext' may free it.
      line = ml_get_buf(wp->w_buffer, lnum);
      ptr = line + v;
    }

    // Draw 'fold' fillchar after 'foldtext', or after 'eol' listchar for transparent 'foldtext'.
    if (draw_folded && wlv.n_extra == 0 && wlv.col < view_width
        && (has_foldtext || (*ptr == NUL && (!wp->w_p_list || !lcs_eol_todo || lcs_eol == NUL)))) {
      // Fill rest of line with 'fold'.
      wlv.sc_extra = wp->w_p_fcs_chars.fold;
      wlv.sc_final = NUL;
      wlv.n_extra = view_width - wlv.col;
      // Don't continue search highlighting past the first filler char.
      search_attr = 0;
    }

    if (draw_folded && wlv.n_extra != 0 && wlv.col >= view_width) {
      // Truncate the folding.
      wlv.n_extra = 0;
    }

    // Get the next character to put on the screen.
    //
    // The "p_extra" points to the extra stuff that is inserted to
    // represent special characters (non-printable stuff) and other
    // things.  When all characters are the same, sc_extra is used.
    // If sc_final is set, it will compulsorily be used at the end.
    // "p_extra" must end in a NUL to avoid utfc_ptr2len() reads past
    // "p_extra[n_extra]".
    // For the '$' of the 'list' option, n_extra == 1, p_extra == "".
    if (wlv.n_extra > 0) {
      // Phase 5: delegate n_extra processing to Rust.
      wls.search_attr = search_attr;
      wls.saved_search_attr = saved_search_attr;
      wls.area_attr = area_attr;
      wls.saved_area_attr = saved_area_attr;
      wls.decor_attr = decor_attr;
      wls.saved_decor_attr = saved_decor_attr;
      wls.n_extra_next = n_extra_next;
      wls.extra_attr_next = extra_attr_next;
      wls.multi_attr = multi_attr;
      NExtraResult _ner = rs_win_line_process_n_extra(wp, &wlv, &wls, *ptr == NUL);
      mb_schar = _ner.mb_schar;
      mb_c = _ner.mb_c;
      mb_l = _ner.mb_l;
      search_attr = wls.search_attr;
      saved_search_attr = wls.saved_search_attr;
      area_attr = wls.area_attr;
      saved_area_attr = wls.saved_area_attr;
      decor_attr = wls.decor_attr;
      saved_decor_attr = wls.saved_decor_attr;
      n_extra_next = wls.n_extra_next;
      extra_attr_next = wls.extra_attr_next;
      multi_attr = wls.multi_attr;
    } else if (wlv.filler_todo > 0) {
      // Wait with reading text until filler lines are done. Still need to
      // initialize these.
      mb_c = ' ';
      mb_schar = schar_from_ascii(' ');
    } else if (has_foldtext || (has_fold && wlv.col >= view_width)) {
      // skip writing the buffer line itself
      mb_schar = NUL;
    } else {
      // Phase 3: delegate character processing to Rust.
      // Sync state that may have changed in C since last wls write.
      wls.extra_check = extra_check;
      wls.has_decor = has_decor;
      wls.char_attr_pri = char_attr_pri;
      ProcessCharResult pcr = rs_win_line_process_char(
        wp, lnum, &wlv, &wls, spv, line,
        (int)(ptr - line), trailcol, leadcol,
        extmark_attr, lcs_eol_todo, search_attr, area_attr,
        term_attrs, has_match_conc);
      // Apply results.
      mb_schar = pcr.mb_schar;
      mb_c = pcr.mb_c;
      mb_l = pcr.mb_l;
      ptr = line + pcr.ptr_col;
      lcs_eol_todo = pcr.lcs_eol_todo;
      search_attr = pcr.search_attr;
      decor_attr = pcr.decor_attr;
      did_decrement_ptr = pcr.did_decrement_ptr;
      // Sync back locals that Rust may have updated via wls.
      is_concealing = wls.is_concealing;
      saved_attr2 = wls.saved_attr2;
      prev_syntax_id = wls.prev_syntax_id;
      char_attr_pri = wls.char_attr_pri;
      // Sync has_syntax back (Rust writes it when syntax highlighting is active).
      has_syntax = wls.has_syntax;
    }  // end of printing from buffer content

    // In the cursor line and we may be concealing characters: correct
    // the cursor column when we reach its position.
    // Phase 4: delegate to Rust.
    wls.did_wcol = did_wcol;
    wls.mb_schar = mb_schar;
    rs_win_line_cursor_conceal_correct(wp, &wlv, &wls, in_curline, (colnr_T)(ptr - line));
    did_wcol = wls.did_wcol;

    // Phase 4: extra_attr restore + precedes listchar. Delegate to Rust.
    wls.search_attr_from_match = search_attr_from_match;
    wls.extra_attr_next = extra_attr_next;
    wls.n_attr3 = n_attr3;
    wls.saved_attr3 = saved_attr3;
    wls.mb_schar = mb_schar;
    wls.mb_c = mb_c;
    wls.n_extra_next = n_extra_next;
    lcs_prec_todo = rs_win_line_extra_attr_restore(wp, &wlv, &wls, lcs_prec_todo);
    search_attr_from_match = wls.search_attr_from_match;
    extra_attr_next = wls.extra_attr_next;
    n_attr3 = wls.n_attr3;
    saved_attr3 = wls.saved_attr3;
    mb_schar = wls.mb_schar;
    mb_c = wls.mb_c;
    n_extra_next = wls.n_extra_next;

    // At end of the text line or just after the last character.
    // Phase 1: EOL highlight. Delegate to Rust.
    if (mb_schar == NUL && eol_hl_off == 0) {
      eol_hl_off = rs_win_line_eol_highlight(wp, &wlv, &wls, lcs_eol_todo,
                                             area_attr, (colnr_T)(ptr - line),
                                             &screen_search_hl);
    }

    // At end of the text line.
    // Phase 1: EOL fill. Delegate to Rust.
    if (mb_schar == NUL) {
      wls.fold_vt = fold_vt;  // sync local fold_vt (may be populated by rs_get_foldtext)
      if (rs_win_line_eol_fill(wp, &wlv, &wls, start_vcol, lcs_eol_todo,
                               eol_hl_off, term_attrs, has_decor)) {
        break;
      }
    }

    // Phase 4c: extends char. Delegate to Rust.
    wls.has_decor = has_decor;
    wls.mb_schar = mb_schar;
    wls.mb_c = mb_c;
    rs_win_line_extends_char(wp, &wlv, &wls, (colnr_T)(ptr - line), *ptr == NUL,
                             lcs_eol, lcs_eol_todo, may_have_inline_virt);
    mb_schar = wls.mb_schar;
    mb_c = wls.mb_c;

    // Phase 1c: cursorcolumn highlight. Delegate to Rust.
    vcol_save_attr = rs_win_line_cursorcolumn(wp, &wlv, lnum_in_visual_area,
                                              search_attr, area_attr);

    if (wlv.filler_todo <= 0) {
      vcol_prev = wlv.vcol;
    }

    // Phase 2: store char. Delegate to Rust.
    rs_win_line_store_char(wp, &wlv, &wls, mb_schar, multi_attr, is_concealing);
    multi_attr = 0;  // consumed by store_char

    // Phase 2: post-store. Delegate to Rust.
    wls.has_decor = has_decor;
    wls.n_attr3 = n_attr3;
    wls.saved_attr3 = saved_attr3;
    wls.saved_attr2 = saved_attr2;
    wls.decor_need_recheck = decor_need_recheck;
    rs_win_line_post_store(wp, &wlv, &wls, vcol_save_attr, (colnr_T)(ptr - line));
    n_attr3 = wls.n_attr3;
    saved_attr3 = wls.saved_attr3;
    decor_need_recheck = wls.decor_need_recheck;

end_check:
    // Phase 3: end-check / wrap. Delegate to Rust.
    if (wlv.col >= view_width && (!has_foldtext || virt_line_index >= 0)
        && (wlv.col <= leftcols_width
            || *ptr != NUL
            || wlv.filler_todo > 0
            || (wp->w_p_list && wp->w_p_lcs_chars.eol != NUL && lcs_eol_todo)
            || (wlv.n_extra != 0 && (wlv.sc_extra != NUL || *wlv.p_extra != NUL))
            || (may_have_inline_virt && has_more_inline_virt(&wlv, ptr - line)))) {
      if (rs_win_line_end_check(wp, &wlv, &wls,
                                endrow, leftcols_width,
                                virt_line_index, virt_line_flags,
                                *ptr == NUL, lcs_eol_todo,
                                &wls.virt_lines, bg_attr,
                                &statuscol.draw, &draw_cols,
                                &virt_line_index, &virt_line_flags,
                                &lcs_prec_todo)) {
        break;
      }
    }
  }     // for every character in the line

  clear_virttext(&fold_vt);
  kv_destroy(virt_lines);
  xfree(foldtext_free);
  return wlv.row;
}
#endif  // 0 -- dead code (old C implementation)

