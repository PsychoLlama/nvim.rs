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

#include "drawline.c.generated.h"

// Rust implementations (rs_* names, called from win_line via generated extern decls)
extern int rs_ins_compl_win_active(win_T *wp);
extern int rs_ins_compl_lnum_in_range(int lnum);
extern const char *rs_get_showbreak_value(win_T *win);
extern int rs_diff_check_with_linestatus(win_T *wp, linenr_T lnum, int *linestatus);
extern bool rs_diff_find_change(win_T *wp, linenr_T lnum, diffline_T *diffline);
extern bool rs_diff_change_parse(diffline_T *diffline, diffline_change_T *change,
                                 int *change_start, int *change_end);

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

// Phase function declarations (Rust implementations)

/// Return value for rs_win_line_process_n_extra.
typedef struct { schar_T mb_schar; int mb_c; int mb_l; } NExtraResult;

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


// ============================================================================
// Pre-loop setup helper (called from rs_win_line_pre_loop via extern "C")
// ============================================================================

/// Pre-loop setup for win_line: encapsulates lines 468-714 of the original C.
/// Mutates wlv (vcol, skip_cells, fromcol, need_showbreak) and wls
/// (extra_check, has_decor, area_highlighting, search_attr, search_attr_from_match,
///  in_multispace, multispace_pos, word_end, spell_attr, cur_checked_col).
/// Populates term_attrs (must be zeroed by caller, size TERM_ATTRS_MAX).
/// Returns fields C still needs as local variables in the main loop.
static PreLoopResult c_win_line_pre_loop(win_T *wp, linenr_T lnum, winlinevars_T *wlv,
                                         WinLineState *wls, spellvars_T *spv,
                                         foldinfo_T foldinfo, int startrow, int endrow,
                                         int col_rows, int *term_attrs)
{
  PreLoopResult res = {
    .ptr_offset = 0,
    .trailcol = MAXCOL,
    .leadcol = 0,
    .lcs_eol_todo = true,
    .lcs_eol = wp->w_p_lcs_chars.eol,
    .lcs_prec_todo = wp->w_p_lcs_chars.prec,
    .start_vcol = 0,
    .may_have_inline_virt = false,
    .virt_line_index = -1,
    .virt_line_flags = 0,
    .draw_cols = true,
    .leftcols_width = 0,
    .statuscol_draw = false,
    .statuscol_width = 0,
    .statuscol_sign_cul_id = 0,
  };

  bool draw_text = wls->draw_text;
  bool has_foldtext = wls->has_foldtext;
  bool has_fold = wls->has_fold;

  // statuscol initialization
  if (*wp->w_p_stc != NUL) {
    res.statuscol_draw = true;
    res.statuscol_width = win_col_off(wp) - (wp == cmdwin_win);
    res.statuscol_sign_cul_id = use_cursor_line_highlight(wp, lnum) ? wlv->sign_cul_attr : 0;
  }

  // current line
  char *line = draw_text ? ml_get_buf(wp->w_buffer, lnum) : "";
  char *ptr = line;

  if (wp->w_p_list && !has_foldtext && draw_text) {
    if (wp->w_p_lcs_chars.space
        || wp->w_p_lcs_chars.multispace != NULL
        || wp->w_p_lcs_chars.leadmultispace != NULL
        || wp->w_p_lcs_chars.trail
        || wp->w_p_lcs_chars.lead
        || wp->w_p_lcs_chars.nbsp) {
      wls->extra_check = true;
    }
    // find start of trailing whitespace
    if (wp->w_p_lcs_chars.trail) {
      res.trailcol = ml_get_buf_len(wp->w_buffer, lnum);
      while (res.trailcol > 0 && ascii_iswhite(ptr[res.trailcol - 1])) {
        res.trailcol--;
      }
      res.trailcol += (colnr_T)(ptr - line);
    }
    // find end of leading whitespace
    if (wp->w_p_lcs_chars.lead || wp->w_p_lcs_chars.leadmultispace != NULL) {
      res.leadcol = 0;
      while (ascii_iswhite(ptr[res.leadcol])) {
        res.leadcol++;
      }
      if (ptr[res.leadcol] == NUL) {
        // in a line full of spaces all of them are treated as trailing
        res.leadcol = 0;
      } else {
        // keep track of the first column not filled with spaces
        res.leadcol += (colnr_T)(ptr - line + 1);
      }
    }
  }

  // 'nowrap' or 'wrap' and a single line that doesn't fit: Advance to the
  // first character to be displayed.
  res.start_vcol = wp->w_p_wrap
                   ? (startrow == 0 ? wp->w_skipcol : 0)
                   : wp->w_leftcol;

  if (has_foldtext) {
    wlv->vcol = res.start_vcol;
  } else if (res.start_vcol > 0 && col_rows == 0) {
    char *prev_ptr = ptr;
    CharSize cs = { 0 };

    CharsizeArg csarg;
    CSType cstype = init_charsize_arg(&csarg, wp, lnum, line);
    csarg.max_head_vcol = res.start_vcol;
    int vcol = wlv->vcol;
    StrCharInfo ci = utf_ptr2StrCharInfo(ptr);
    while (vcol < res.start_vcol) {
      cs = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &csarg);
      vcol += cs.width;
      prev_ptr = ci.ptr;
      if (*prev_ptr == NUL) {
        break;
      }
      ci = utfc_next(ci);
      if (wp->w_p_list) {
        wls->in_multispace = *prev_ptr == ' ' && (*ci.ptr == ' '
                                                   || (prev_ptr > line && prev_ptr[-1] == ' '));
        if (!wls->in_multispace) {
          wls->multispace_pos = 0;
        } else if (ci.ptr >= line + res.leadcol
                   && wp->w_p_lcs_chars.multispace != NULL) {
          wls->multispace_pos++;
          if (wp->w_p_lcs_chars.multispace[wls->multispace_pos] == NUL) {
            wls->multispace_pos = 0;
          }
        } else if (ci.ptr < line + res.leadcol
                   && wp->w_p_lcs_chars.leadmultispace != NULL) {
          wls->multispace_pos++;
          if (wp->w_p_lcs_chars.leadmultispace[wls->multispace_pos] == NUL) {
            wls->multispace_pos = 0;
          }
        }
      }
    }
    wlv->vcol = vcol;
    ptr = ci.ptr;
    int charsize = cs.width;
    int head = cs.head;

    // When 'cuc', 'colorcolumn', 'virtualedit', visual mode active, or
    // drawing a fold, the end of the line may be before the start of the
    // displayed part.
    if (wlv->vcol < res.start_vcol && (wp->w_p_cuc
                                       || wlv->color_cols
                                       || virtual_active(wp)
                                       || (VIsual_active && wp->w_buffer == curwin->w_buffer)
                                       || has_fold)) {
      wlv->vcol = res.start_vcol;
    }

    // Handle a character that's not completely on the screen.
    if (wlv->vcol > res.start_vcol) {
      wlv->vcol -= charsize;
      ptr = prev_ptr;
    }

    if (res.start_vcol > wlv->vcol) {
      wlv->skip_cells = res.start_vcol - wlv->vcol - head;
    }

    // Adjust inverted text relative to start of screen
    if (wlv->tocol <= wlv->vcol) {
      wlv->fromcol = 0;
    } else if (wlv->fromcol >= 0 && wlv->fromcol < wlv->vcol) {
      wlv->fromcol = wlv->vcol;
    }

    // When w_skipcol is non-zero, first line needs 'showbreak'
    if (wp->w_p_wrap) {
      wlv->need_showbreak = true;
    }
    // When spell checking a word we need to figure out the start of the
    // word and if it's badly spelled or not.
    if (spv->spv_has_spell) {
      colnr_T linecol = (colnr_T)(ptr - line);
      hlf_T spell_hlf = HLF_COUNT;

      pos_T pos = wp->w_cursor;
      wp->w_cursor.lnum = lnum;
      wp->w_cursor.col = linecol;
      size_t len = spell_move_to(wp, FORWARD, SMT_ALL, true, &spell_hlf);

      // spell_move_to() may call ml_get() and make "line" invalid
      line = ml_get_buf(wp->w_buffer, lnum);
      ptr = line + linecol;

      if (len == 0 || wp->w_cursor.col > linecol) {
        // no bad word found at line start, don't check until end of a word
        spell_hlf = HLF_COUNT;
        wls->word_end = (int)(spell_to_word_end(ptr, wp) - line + 1);
      } else {
        // bad word found, use attributes until end of word
        assert(len <= INT_MAX);
        wls->word_end = wp->w_cursor.col + (int)len + 1;
        // Turn index into actual attributes.
        if (spell_hlf != HLF_COUNT) {
          wls->spell_attr = highlight_attr[spell_hlf];
        }
      }
      wp->w_cursor = pos;

      // Need to restart syntax highlighting for this line.
      if (wls->has_syntax) {
        syntax_start(wp, lnum);
      }
    }
  }

  if (wls->check_decor_providers) {
    int const col = (int)(ptr - line);
    wls->decor_provider_end_col = rs_decor_providers_setup(endrow - startrow,
                                                           res.start_vcol == 0,
                                                           lnum,
                                                           col,
                                                           wp);
    line = ml_get_buf(wp->w_buffer, lnum);
    ptr = line + col;
  }

  decor_redraw_line(wp, lnum - 1, &decor_state);
  if (!wls->has_decor && decor_has_more_decorations(&decor_state, lnum - 1)) {
    wls->has_decor = true;
    wls->extra_check = true;
  }

  // Correct highlighting for cursor that can't be disabled.
  if (wlv->fromcol >= 0) {
    if (wls->noinvcur) {
      if ((colnr_T)wlv->fromcol == wp->w_virtcol) {
        wls->fromcol_prev = wlv->fromcol;
        wlv->fromcol = -1;
      } else if ((colnr_T)wlv->fromcol < wp->w_virtcol) {
        wls->fromcol_prev = wp->w_virtcol;
      }
    }
    if (wlv->fromcol >= wlv->tocol) {
      wlv->fromcol = -1;
    }
  }

  if (col_rows == 0 && draw_text && !has_foldtext) {
    const int v = (int)(ptr - line);
    wls->area_highlighting |= prepare_search_hl_line(wp, lnum, v,
                                                     &line, &screen_search_hl, &wls->search_attr,
                                                     &wls->search_attr_from_match);
    ptr = line + v;  // "line" may have been updated
  }

  if ((State & MODE_INSERT) && rs_ins_compl_win_active(wp)
      && (wls->in_curline || rs_ins_compl_lnum_in_range((int)lnum))) {
    wls->area_highlighting = true;
  }

  win_line_start(wp, wlv);

  if (wp->w_buffer->terminal) {
    terminal_get_line_attributes(wp->w_buffer->terminal, wp, lnum, term_attrs);
    wls->extra_check = true;
  }

  res.may_have_inline_virt = !has_foldtext && buf_meta_total(wp->w_buffer, kMTMetaInline) > 0;
  res.ptr_offset = (int)(ptr - line);
  return res;
}

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

  // Pre-loop setup: delegate lines 468-714 to c_win_line_pre_loop.
  // term_attrs is stack-allocated here so C controls its lifetime.
  int term_attrs[TERM_ATTRS_MAX] = { 0 };
  PreLoopResult plr = c_win_line_pre_loop(wp, lnum, &wlv, &wls, spv, foldinfo,
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
    int decor_conceal = 0;

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

    // Skip this quickly when working on the text.
    if (draw_cols) {
      if (cul_screenline) {
        wlv.cul_attr = 0;
        wlv.line_attr = line_attr_save;
        wlv.line_attr_lowprio = line_attr_lowprio_save;
      }

      assert(wlv.off == 0);

      if (wp == cmdwin_win) {
        // Draw the cmdline character.
        draw_col_fill(&wlv, schar_from_ascii(cmdwin_type), 1, win_hl_attr(wp, HLF_AT));
      }

      if (wlv.filler_todo > 0) {
        int index = wlv.filler_todo - (wlv.filler_lines - wlv.n_virt_lines);
        if (index > 0) {
          virt_line_index = (int)kv_size(virt_lines) - index;
          assert(virt_line_index >= 0);
          virt_line_flags = kv_A(virt_lines, virt_line_index).flags;
        }
      }

      if (virt_line_index >= 0 && (virt_line_flags & kVLLeftcol)) {
        // skip columns
      } else if (statuscol.draw) {
        // Draw 'statuscolumn' if it is set.
        const int v = (int)(ptr - line);
        rs_draw_statuscol(wp, &wlv, wlv.row - startrow - wlv.filler_lines, col_rows, &statuscol);
        if (wp->w_redr_statuscol) {
          break;
        }
        if (draw_text) {
          // Get the line again as evaluating 'statuscolumn' may free it.
          line = ml_get_buf(wp->w_buffer, lnum);
          ptr = line + v;
        }
      } else {
        // draw builtin info columns: fold, sign, number
        draw_foldcolumn(wp, &wlv);

        // wp->w_scwidth is zero if signcol=number is used
        for (int sign_idx = 0; sign_idx < wp->w_scwidth; sign_idx++) {
          draw_sign(false, wp, &wlv, sign_idx);
        }

        draw_lnum_col(wp, &wlv);
      }

      win_col_offset = wlv.off;

      // When only updating the columns and that's done, stop here.
      if (col_rows > 0) {
        wlv_put_linebuf(wp, &wlv, MIN(wlv.off, view_width), false, bg_attr, 0);
        // Need to update more screen lines if:
        // - 'statuscolumn' needs to be drawn, or
        // - LineNrAbove or LineNrBelow is used, or
        // - still drawing filler lines.
        if ((wlv.row + 1 - wlv.startrow < col_rows
             && (statuscol.draw
                 || win_hl_attr(wp, HLF_LNA) != win_hl_attr(wp, HLF_N)
                 || win_hl_attr(wp, HLF_LNB) != win_hl_attr(wp, HLF_N)))
            || wlv.filler_todo > 0) {
          wlv.row++;
          if (wlv.row == endrow) {
            break;
          }
          wlv.filler_todo--;
          virt_line_index = -1;
          if (wlv.filler_todo == 0 && (wp->w_botfill || !draw_text)) {
            break;
          }
          // win_line_start(wp, &wlv);
          wlv.col = 0;
          wlv.off = 0;
          continue;
        } else {
          break;
        }
      }

      // Check if 'breakindent' applies and show it.
      if (!wp->w_briopt_sbr) {
        handle_breakindent(wp, &wlv);
      }
      handle_showbreak_and_filler(wp, &wlv);
      if (wp->w_briopt_sbr) {
        handle_breakindent(wp, &wlv);
      }

      wlv.col = wlv.off;
      draw_cols = false;
      if (wlv.filler_todo <= 0) {
        leftcols_width = wlv.off;
      }
      if (has_decor && wlv.row == startrow + wlv.filler_lines) {
        // hide virt_text on text hidden by 'nowrap' or 'smoothscroll'
        decor_redraw_col(wp, (colnr_T)(ptr - line) - 1, wlv.off, true, &decor_state);
      }
      if (wlv.col >= view_width) {
        wlv.col = wlv.off = view_width;
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
      if (wlv.n_extra == 0 || !wlv.extra_for_extmark) {
        wlv.reset_extra_attr = false;
      }

      if (has_decor && wlv.n_extra == 0) {
        // Duplicate the Visual area check after this block,
        // but don't check inside p_extra here.
        if (wlv.vcol == wlv.fromcol
            || (wlv.vcol + 1 == wlv.fromcol
                && (wlv.n_extra == 0 && utf_ptr2cells(ptr) > 1))
            || (vcol_prev == fromcol_prev
                && vcol_prev < wlv.vcol
                && wlv.vcol < wlv.tocol)) {
          area_active = true;
        } else if (area_active
                   && (wlv.vcol == wlv.tocol
                       || (noinvcur && wlv.vcol == wp->w_virtcol))) {
          area_active = false;
        }

        bool selected = (area_active || (area_highlighting && noinvcur
                                         && wlv.vcol == wp->w_virtcol));
        // When there may be inline virtual text, position of non-inline virtual text
        // can only be decided after drawing inline virtual text with lower priority.
        if (decor_need_recheck) {
          if (!may_have_inline_virt) {
            decor_recheck_draw_col(wlv.off, selected, &decor_state);
          }
          decor_need_recheck = false;
        }
        extmark_attr = decor_redraw_col(wp, (colnr_T)(ptr - line),
                                        may_have_inline_virt ? -3 : wlv.off,
                                        selected, &decor_state);
        if (may_have_inline_virt) {
          handle_inline_virtual_text(wp, &wlv, ptr - line, selected);
          if (wlv.n_extra > 0 && wlv.virt_inline_hl_mode <= kHlModeReplace) {
            // restore search_attr and area_attr when n_extra is down to zero
            // TODO(bfredl): this is ugly as fuck. look if we can do this some other way.
            saved_search_attr = search_attr;
            saved_area_attr = area_attr;
            saved_decor_attr = decor_attr;
            saved_search_attr_from_match = search_attr_from_match;
            search_attr = 0;
            area_attr = 0;
            decor_attr = 0;
            search_attr_from_match = false;
          }
        }
      }

      int *area_attr_p = wlv.extra_for_extmark && wlv.virt_inline_hl_mode <= kHlModeReplace
                         ? &saved_area_attr : &area_attr;

      // handle Visual or match highlighting in this line
      if (wlv.vcol == wlv.fromcol
          || (wlv.vcol + 1 == wlv.fromcol
              && ((wlv.n_extra == 0 && utf_ptr2cells(ptr) > 1)
                  || (wlv.n_extra > 0 && wlv.p_extra != NULL
                      && utf_ptr2cells(wlv.p_extra) > 1)))
          || (vcol_prev == fromcol_prev
              && vcol_prev < wlv.vcol               // not at margin
              && wlv.vcol < wlv.tocol)) {
        *area_attr_p = vi_attr;                     // start highlighting
        area_active = true;
      } else if (*area_attr_p != 0
                 && (wlv.vcol == wlv.tocol
                     || (noinvcur && wlv.vcol == wp->w_virtcol))) {
        *area_attr_p = 0;                           // stop highlighting
        area_active = false;
      }

      if (!has_foldtext && wlv.n_extra == 0) {
        // Check for start/end of 'hlsearch' and other matches.
        // After end, check for start/end of next match.
        // When another match, have to check for start again.
        const int v = (int)(ptr - line);
        search_attr = update_search_hl(wp, lnum, v, &line, &screen_search_hl,
                                       &has_match_conc, &match_conc, lcs_eol_todo,
                                       &on_last_col, &search_attr_from_match);
        ptr = line + v;  // "line" may have been changed

        // Do not allow a conceal over EOL otherwise EOL will be missed
        // and bad things happen.
        if (*ptr == NUL) {
          has_match_conc = 0;
        }

        // Check if ComplMatchIns highlight is needed.
        if ((State & MODE_INSERT) && rs_ins_compl_win_active(wp)
            && (in_curline || rs_ins_compl_lnum_in_range((int)lnum))) {
          int ins_match_attr = rs_ins_compl_col_range_attr((int)lnum, (int)(ptr - line));
          if (ins_match_attr > 0) {
            search_attr = hl_combine_attr(search_attr, ins_match_attr);
          }
        }
      }

      if (wlv.diff_hlf != (hlf_T)0) {
        if (line_changes.num_changes > 0
            && change_index >= 0
            && change_index < line_changes.num_changes - 1) {
          if (ptr - line
              >= line_changes.changes[change_index + 1].dc_start[line_changes.bufidx]) {
            change_index += 1;
          }
        }
        bool added = false;
        if (line_changes.num_changes > 0 && change_index >= 0
            && change_index < line_changes.num_changes) {
          added = rs_diff_change_parse(&line_changes, &line_changes.changes[change_index],
                                    &change_start, &change_end);
        }
        // When there is extra text (eg: virtual text) it gets the
        // diff highlighting for the line, but not for changed text.
        if (wlv.diff_hlf == HLF_CHD && ptr - line >= change_start
            && wlv.n_extra == 0) {
          wlv.diff_hlf = added ? HLF_TXA : HLF_TXD;   // added/changed text
        }
        if ((wlv.diff_hlf == HLF_TXD || wlv.diff_hlf == HLF_TXA)
            && ((ptr - line >= change_end && wlv.n_extra == 0)
                || (wlv.n_extra > 0 && wlv.extra_for_extmark))) {
          wlv.diff_hlf = HLF_CHD;                     // changed line
        }
        set_line_attr_for_diff(wp, &wlv);
      }

      // Decide which of the highlight attributes to use.
      if (area_attr != 0) {
        char_attr_pri = hl_combine_attr(wlv.line_attr, area_attr);
        if (!highlight_match) {
          // let search highlight show in Visual area if possible
          char_attr_pri = hl_combine_attr(search_attr, char_attr_pri);
        }
      } else if (search_attr != 0) {
        char_attr_pri = hl_combine_attr(wlv.line_attr, search_attr);
      } else if (wlv.line_attr != 0
                 && ((wlv.fromcol == -10 && wlv.tocol == MAXCOL)
                     || wlv.vcol < wlv.fromcol
                     || vcol_prev < fromcol_prev
                     || wlv.vcol >= wlv.tocol)) {
        // Use wlv.line_attr when not in the Visual or 'incsearch' area
        // (area_attr may be 0 when "noinvcur" is set).
        char_attr_pri = wlv.line_attr;
      } else {
        char_attr_pri = 0;
      }
      char_attr_base = hl_combine_attr(folded_attr, decor_attr);
      wlv.char_attr = hl_combine_attr(char_attr_base, char_attr_pri);
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
      const char *prev_ptr = ptr;

      // first byte of next char
      int c0 = (uint8_t)(*ptr);
      if (c0 == NUL) {
        // no more cells to skip
        wlv.skip_cells = 0;
      }

      // Get a character from the line itself.
      mb_l = utfc_ptr2len(ptr);
      mb_schar = utfc_ptr2schar(ptr, &mb_c);

      // Overlong encoded ASCII or ASCII with composing char
      // is displayed normally, except a NUL.
      if (mb_l > 1 && mb_c < 0x80) {
        c0 = mb_c;
      }

      if ((mb_l == 1 && c0 >= 0x80)
          || (mb_l >= 1 && mb_c == 0)
          || (mb_l > 1 && (!vim_isprintc(mb_c)))) {
        // Illegal UTF-8 byte: display as <xx>.
        // Non-printable character : display as ? or fullwidth ?.
        transchar_hex(wlv.extra, mb_c);
        if (wp->w_p_rl) {  // reverse
          rl_mirror_ascii(wlv.extra, NULL);
        }

        wlv.p_extra = wlv.extra;
        mb_c = mb_ptr2char_adv((const char **)&wlv.p_extra);
        mb_schar = schar_from_char(mb_c);
        wlv.n_extra = (int)strlen(wlv.p_extra);
        wlv.sc_extra = NUL;
        wlv.sc_final = NUL;
        if (area_attr == 0 && search_attr == 0) {
          wlv.n_attr = wlv.n_extra + 1;
          wlv.extra_attr = win_hl_attr(wp, HLF_8);
          saved_attr2 = wlv.char_attr;               // save current attr
        }
      } else if (mb_l == 0) {        // at the NUL at end-of-line
        mb_l = 1;
      }
      // If a double-width char doesn't fit display a '>' in the
      // last column; the character is displayed at the start of the
      // next line.
      if (wlv.col >= view_width - 1 && schar_cells(mb_schar) == 2) {
        mb_schar = schar_from_ascii('>');
        mb_c = '>';
        mb_l = 1;
        multi_attr = win_hl_attr(wp, HLF_AT);
        // Put pointer back so that the character will be
        // displayed at the start of the next line.
        ptr--;
        did_decrement_ptr = true;
      } else if (*ptr != NUL) {
        ptr += mb_l - 1;
      }

      // If a double-width char doesn't fit at the left side display a '<' in
      // the first column.  Don't do this for unprintable characters.
      if (wlv.skip_cells > 0 && mb_l > 1 && wlv.n_extra == 0) {
        wlv.n_extra = 1;
        wlv.sc_extra = schar_from_ascii(MB_FILLER_CHAR);
        wlv.sc_final = NUL;
        mb_schar = schar_from_ascii(' ');
        mb_c = ' ';
        mb_l = 1;
        if (area_attr == 0 && search_attr == 0) {
          wlv.n_attr = wlv.n_extra + 1;
          wlv.extra_attr = win_hl_attr(wp, HLF_AT);
          saved_attr2 = wlv.char_attr;             // save current attr
        }
      }
      ptr++;

      decor_attr = 0;
      if (extra_check) {
        const bool no_plain_buffer = (wp->w_s->b_p_spo_flags & kOptSpoFlagNoplainbuffer) != 0;
        bool can_spell = !no_plain_buffer;

        // Get extmark and syntax attributes, unless still at the start of the line
        // (double-wide char that doesn't fit).
        const int v = (int)(ptr - line);
        const ptrdiff_t prev_v = prev_ptr - line;
        if (has_syntax && v > 0) {
          // Get the syntax attribute for the character.  If there
          // is an error, disable syntax highlighting.
          int save_did_emsg = did_emsg;
          did_emsg = false;

          decor_attr = get_syntax_attr(v - 1, spv->spv_has_spell ? &can_spell : NULL, false);

          if (did_emsg) {
            wp->w_s->b_syn_error = true;
            has_syntax = false;
          } else {
            did_emsg = save_did_emsg;
          }

          if (wp->w_s->b_syn_slow) {
            has_syntax = false;
          }

          // Need to get the line again, a multi-line regexp may
          // have made it invalid.
          line = ml_get_buf(wp->w_buffer, lnum);
          ptr = line + v;
          prev_ptr = line + prev_v;

          // no concealing past the end of the line, it interferes
          // with line highlighting.
          syntax_flags = (mb_schar == 0) ? 0 : get_syntax_info(&syntax_seqnr);
        }

        if (has_decor && v > 0) {
          // extmarks take preceedence over syntax.c
          decor_attr = hl_combine_attr(decor_attr, extmark_attr);
          decor_conceal = decor_state.conceal;
          can_spell = TRISTATE_TO_BOOL(decor_state.spell, can_spell);
        }

        char_attr_base = hl_combine_attr(folded_attr, decor_attr);
        wlv.char_attr = hl_combine_attr(char_attr_base, char_attr_pri);

        // Check spelling (unless at the end of the line).
        // Only do this when there is no syntax highlighting, the
        // @Spell cluster is not used or the current syntax item
        // contains the @Spell cluster.
        int v1 = (int)(ptr - line);
        if (spv->spv_has_spell && v1 >= word_end && v1 > cur_checked_col) {
          spell_attr = 0;
          // do not calculate cap_col at the end of the line or when
          // only white space is following
          if (mb_schar != 0 && (*skipwhite(prev_ptr) != NUL) && can_spell) {
            char *p;
            hlf_T spell_hlf = HLF_COUNT;
            v1 -= mb_l - 1;

            // Use nextline[] if possible, it has the start of the
            // next line concatenated.
            if ((prev_ptr - line) - nextlinecol >= 0) {
              p = nextline + ((prev_ptr - line) - nextlinecol);
            } else {
              p = (char *)prev_ptr;
            }
            spv->spv_cap_col -= (int)(prev_ptr - line);
            size_t tmplen = spell_check(wp, p, &spell_hlf, &spv->spv_cap_col, spv->spv_unchanged);
            assert(tmplen <= INT_MAX);
            int len = (int)tmplen;
            word_end = v1 + len;

            // In Insert mode only highlight a word that
            // doesn't touch the cursor.
            if (spell_hlf != HLF_COUNT
                && (State & MODE_INSERT)
                && wp->w_cursor.lnum == lnum
                && wp->w_cursor.col >= (colnr_T)(prev_ptr - line)
                && wp->w_cursor.col < (colnr_T)word_end) {
              spell_hlf = HLF_COUNT;
              spell_redraw_lnum = lnum;
            }

            if (spell_hlf == HLF_COUNT && p != prev_ptr
                && (p - nextline) + len > nextline_idx) {
              // Remember that the good word continues at the
              // start of the next line.
              spv->spv_checked_lnum = lnum + 1;
              spv->spv_checked_col = (int)((p - nextline) + len - nextline_idx);
            }

            // Turn index into actual attributes.
            if (spell_hlf != HLF_COUNT) {
              spell_attr = highlight_attr[spell_hlf];
            }

            if (spv->spv_cap_col > 0) {
              if (p != prev_ptr && (p - nextline) + spv->spv_cap_col >= nextline_idx) {
                // Remember that the word in the next line
                // must start with a capital.
                spv->spv_capcol_lnum = lnum + 1;
                spv->spv_cap_col = (int)((p - nextline) + spv->spv_cap_col - nextline_idx);
              } else {
                // Compute the actual column.
                spv->spv_cap_col += (int)(prev_ptr - line);
              }
            }
          }
        }
        if (spell_attr != 0) {
          char_attr_base = hl_combine_attr(char_attr_base, spell_attr);
          wlv.char_attr = hl_combine_attr(char_attr_base, char_attr_pri);
        }

        if (wp->w_buffer->terminal) {
          wlv.char_attr = hl_combine_attr(wlv.vcol < TERM_ATTRS_MAX ? term_attrs[wlv.vcol] : 0,
                                          wlv.char_attr);
        }

        // we don't want linebreak to apply for lines that start with
        // leading spaces, followed by long letters (since it would add
        // a break at the beginning of a line and this might be unexpected)
        //
        // So only allow to linebreak, once we have found chars not in
        // 'breakat' in the line.
        if (wp->w_p_lbr && !wlv.need_lbr && mb_schar != NUL
            && !vim_isbreak((uint8_t)(*ptr))) {
          wlv.need_lbr = true;
        }
        // Found last space before word: check for line break.
        if (wp->w_p_lbr && c0 == mb_c && mb_c < 128 && wlv.need_lbr
            && vim_isbreak(mb_c) && !vim_isbreak((uint8_t)(*ptr))) {
          int mb_off = utf_head_off(line, ptr - 1);
          char *p = ptr - (mb_off + 1);

          CharsizeArg csarg;
          // lnum == 0, do not want virtual text to be counted here
          CSType cstype = init_charsize_arg(&csarg, wp, 0, line);
          wlv.n_extra = win_charsize(cstype, wlv.vcol, p, utf_ptr2CharInfo(p).value,
                                     &csarg).width - 1;

          if (on_last_col && mb_c != TAB) {
            // Do not continue search/match highlighting over the
            // line break, but for TABs the highlighting should
            // include the complete width of the character
            search_attr = 0;
          }

          if (mb_c == TAB && wlv.n_extra + wlv.col > view_width) {
            wlv.n_extra = tabstop_padding(wlv.vcol, wp->w_buffer->b_p_ts,
                                          wp->w_buffer->b_p_vts_array) - 1;
          }
          wlv.sc_extra = schar_from_ascii(mb_off > 0 ? MB_FILLER_CHAR : ' ');
          wlv.sc_final = NUL;
          if (mb_c < 128 && ascii_iswhite(mb_c)) {
            if (mb_c == TAB) {
              // See "Tab alignment" below.
              fix_for_boguscols(&wlv);
            }
            if (!wp->w_p_list) {
              mb_c = ' ';
              mb_schar = schar_from_ascii(mb_c);
            }
          }
        }

        if (wp->w_p_list) {
          in_multispace = mb_c == ' ' && (*ptr == ' ' || (prev_ptr > line && prev_ptr[-1] == ' '));
          if (!in_multispace) {
            multispace_pos = 0;
          }
        }

        // 'list': Change char 160 to 'nbsp' and space to 'space'.
        // But not when the character is followed by a composing
        // character (use mb_l to check that).
        if (wp->w_p_list
            && ((((mb_c == 160 && mb_l == 2) || (mb_c == 0x202f && mb_l == 3))
                 && wp->w_p_lcs_chars.nbsp)
                || (mb_c == ' '
                    && mb_l == 1
                    && (wp->w_p_lcs_chars.space
                        || (in_multispace && wp->w_p_lcs_chars.multispace != NULL))
                    && ptr - line >= leadcol
                    && ptr - line <= trailcol))) {
          if (in_multispace && wp->w_p_lcs_chars.multispace != NULL) {
            mb_schar = wp->w_p_lcs_chars.multispace[multispace_pos++];
            if (wp->w_p_lcs_chars.multispace[multispace_pos] == NUL) {
              multispace_pos = 0;
            }
          } else {
            mb_schar = (mb_c == ' ') ? wp->w_p_lcs_chars.space : wp->w_p_lcs_chars.nbsp;
          }
          wlv.n_attr = 1;
          wlv.extra_attr = win_hl_attr(wp, HLF_0);
          saved_attr2 = wlv.char_attr;  // save current attr
          mb_c = schar_get_first_codepoint(mb_schar);
        }

        if (mb_c == ' ' && mb_l == 1 && ((trailcol != MAXCOL && ptr > line + trailcol)
                                         || (leadcol != 0 && ptr < line + leadcol))) {
          if (leadcol != 0 && in_multispace && ptr < line + leadcol
              && wp->w_p_lcs_chars.leadmultispace != NULL) {
            mb_schar = wp->w_p_lcs_chars.leadmultispace[multispace_pos++];
            if (wp->w_p_lcs_chars.leadmultispace[multispace_pos] == NUL) {
              multispace_pos = 0;
            }
          } else if (ptr > line + trailcol && wp->w_p_lcs_chars.trail) {
            mb_schar = wp->w_p_lcs_chars.trail;
          } else if (ptr < line + leadcol && wp->w_p_lcs_chars.lead) {
            mb_schar = wp->w_p_lcs_chars.lead;
          } else if (leadcol != 0 && wp->w_p_lcs_chars.space) {
            mb_schar = wp->w_p_lcs_chars.space;
          }

          wlv.n_attr = 1;
          wlv.extra_attr = win_hl_attr(wp, HLF_0);
          saved_attr2 = wlv.char_attr;  // save current attr
          mb_c = schar_get_first_codepoint(mb_schar);
        }
      }

      // Handling of non-printable characters.
      if (!vim_isprintc(mb_c)) {
        // when getting a character from the file, we may have to
        // turn it into something else on the way to putting it on the screen.
        if (mb_c == TAB && (!wp->w_p_list || wp->w_p_lcs_chars.tab1)) {
          int tab_len = 0;
          colnr_T vcol_adjusted = wlv.vcol;  // removed showbreak length
          char *const sbr = (char *)rs_get_showbreak_value(wp);

          // Only adjust the tab_len, when at the first column after the
          // showbreak value was drawn.
          if (*sbr != NUL && wlv.vcol == wlv.vcol_sbr && wp->w_p_wrap) {
            vcol_adjusted = wlv.vcol - mb_charlen(sbr);
          }
          // tab amount depends on current column
          tab_len = tabstop_padding(vcol_adjusted,
                                    wp->w_buffer->b_p_ts,
                                    wp->w_buffer->b_p_vts_array) - 1;

          if (!wp->w_p_lbr || !wp->w_p_list) {
            wlv.n_extra = tab_len;
          } else {
            int saved_nextra = wlv.n_extra;

            if (wlv.vcol_off_co > 0) {
              // there are characters to conceal
              tab_len += wlv.vcol_off_co;
            }
            // boguscols before fix_for_boguscols() from above.
            if (wp->w_p_lcs_chars.tab1 && wlv.old_boguscols > 0
                && wlv.n_extra > tab_len) {
              tab_len += wlv.n_extra - tab_len;
            }

            if (tab_len > 0) {
              // If wlv.n_extra > 0, it gives the number of chars
              // to use for a tab, else we need to calculate the
              // width for a tab.
              size_t tab2_len = schar_len(wp->w_p_lcs_chars.tab2);
              size_t len = (size_t)tab_len * tab2_len;
              if (wp->w_p_lcs_chars.tab3) {
                len += schar_len(wp->w_p_lcs_chars.tab3) - tab2_len;
              }
              if (wlv.n_extra > 0) {
                len += (size_t)(wlv.n_extra - tab_len);
              }
              mb_schar = wp->w_p_lcs_chars.tab1;
              mb_c = schar_get_first_codepoint(mb_schar);
              char *p = rs_get_extra_buf(len + 1);
              memset(p, ' ', len);
              p[len] = NUL;
              wlv.p_extra = p;
              for (int i = 0; i < tab_len; i++) {
                if (*p == NUL) {
                  tab_len = i;
                  break;
                }
                schar_T lcs = wp->w_p_lcs_chars.tab2;

                // if tab3 is given, use it for the last char
                if (wp->w_p_lcs_chars.tab3 && i == tab_len - 1) {
                  lcs = wp->w_p_lcs_chars.tab3;
                }
                size_t slen = schar_get_adv(&p, lcs);
                wlv.n_extra += (int)slen - (saved_nextra > 0 ? 1 : 0);
              }

              // n_extra will be increased by fix_for_boguscols()
              // below, so need to adjust for that here
              if (wlv.vcol_off_co > 0) {
                wlv.n_extra -= wlv.vcol_off_co;
              }
            }
          }

          {
            int vc_saved = wlv.vcol_off_co;

            // Tab alignment should be identical regardless of
            // 'conceallevel' value. So tab compensates of all
            // previous concealed characters, and thus resets
            // vcol_off_co and boguscols accumulated so far in the
            // line. Note that the tab can be longer than
            // 'tabstop' when there are concealed characters.
            fix_for_boguscols(&wlv);

            // Make sure, the highlighting for the tab char will be
            // correctly set further below (effectively reverts the
            // fix_for_boguscols() call).
            if (wlv.n_extra == tab_len + vc_saved && wp->w_p_list
                && wp->w_p_lcs_chars.tab1) {
              tab_len += vc_saved;
            }
          }

          if (wp->w_p_list) {
            mb_schar = (wlv.n_extra == 0 && wp->w_p_lcs_chars.tab3)
                       ? wp->w_p_lcs_chars.tab3 : wp->w_p_lcs_chars.tab1;
            if (wp->w_p_lbr && wlv.p_extra != NULL && *wlv.p_extra != NUL) {
              wlv.sc_extra = NUL;  // using p_extra from above
            } else {
              wlv.sc_extra = wp->w_p_lcs_chars.tab2;
            }
            wlv.sc_final = wp->w_p_lcs_chars.tab3;
            wlv.n_attr = tab_len + 1;
            wlv.extra_attr = win_hl_attr(wp, HLF_0);
            saved_attr2 = wlv.char_attr;  // save current attr
          } else {
            wlv.sc_final = NUL;
            wlv.sc_extra = schar_from_ascii(' ');
            mb_schar = schar_from_ascii(' ');
          }
          mb_c = schar_get_first_codepoint(mb_schar);
        } else if (mb_schar == NUL
                   && (wp->w_p_list
                       || ((wlv.fromcol >= 0 || fromcol_prev >= 0)
                           && wlv.tocol > wlv.vcol
                           && VIsual_mode != Ctrl_V
                           && wlv.col < view_width
                           && !(noinvcur
                                && lnum == wp->w_cursor.lnum
                                && wlv.vcol == wp->w_virtcol)))
                   && lcs_eol_todo && lcs_eol != NUL) {
          // Display a '$' after the line or highlight an extra
          // character if the line break is included.
          // For a diff line the highlighting continues after the "$".
          if (wlv.diff_hlf == (hlf_T)0
              && wlv.line_attr == 0
              && wlv.line_attr_lowprio == 0) {
            // In virtualedit, visual selections may extend beyond end of line
            if (!(area_highlighting && virtual_active(wp)
                  && wlv.tocol != MAXCOL && wlv.vcol < wlv.tocol)) {
              wlv.p_extra = "";
            }
            wlv.n_extra = 0;
          }
          if (wp->w_p_list && wp->w_p_lcs_chars.eol > 0) {
            mb_schar = wp->w_p_lcs_chars.eol;
          } else {
            mb_schar = schar_from_ascii(' ');
          }
          lcs_eol_todo = false;
          ptr--;  // put it back at the NUL
          wlv.extra_attr = win_hl_attr(wp, HLF_AT);
          wlv.n_attr = 1;
          mb_c = schar_get_first_codepoint(mb_schar);
        } else if (mb_schar != NUL) {
          wlv.p_extra = transchar_buf(wp->w_buffer, mb_c);
          if (wlv.n_extra == 0) {
            wlv.n_extra = byte2cells(mb_c) - 1;
          }
          if ((dy_flags & kOptDyFlagUhex) && wp->w_p_rl) {
            rl_mirror_ascii(wlv.p_extra, NULL);   // reverse "<12>"
          }
          wlv.sc_extra = NUL;
          wlv.sc_final = NUL;
          if (wp->w_p_lbr) {
            mb_c = (uint8_t)(*wlv.p_extra);
            char *p = rs_get_extra_buf((size_t)wlv.n_extra + 1);
            memset(p, ' ', (size_t)wlv.n_extra);
            memcpy(p, wlv.p_extra + 1, strlen(wlv.p_extra) - 1);
            p[wlv.n_extra] = NUL;
            wlv.p_extra = p;
          } else {
            wlv.n_extra = byte2cells(mb_c) - 1;
            mb_c = (uint8_t)(*wlv.p_extra++);
          }
          wlv.n_attr = wlv.n_extra + 1;
          wlv.extra_attr = win_hl_attr(wp, HLF_8);
          saved_attr2 = wlv.char_attr;  // save current attr
          mb_schar = schar_from_ascii(mb_c);
        } else if (VIsual_active
                   && (VIsual_mode == Ctrl_V || VIsual_mode == 'v')
                   && virtual_active(wp)
                   && wlv.tocol != MAXCOL
                   && wlv.vcol < wlv.tocol
                   && wlv.col < view_width) {
          mb_c = ' ';
          mb_schar = schar_from_char(mb_c);
          ptr--;  // put it back at the NUL
        }
      }

      if (wp->w_p_cole > 0
          && (wp != curwin || lnum != wp->w_cursor.lnum || conceal_cursor_line(wp))
          && ((syntax_flags & HL_CONCEAL) != 0 || has_match_conc > 0 || decor_conceal > 0)
          && !(lnum_in_visual_area && vim_strchr(wp->w_p_cocu, 'v') == NULL)) {
        wlv.char_attr = conceal_attr;
        if (((prev_syntax_id != syntax_seqnr && (syntax_flags & HL_CONCEAL) != 0)
             || has_match_conc > 1 || decor_conceal > 1)
            && (syn_get_sub_char() != NUL
                || (has_match_conc && match_conc)
                || (decor_conceal && decor_state.conceal_char)
                || wp->w_p_cole == 1)
            && wp->w_p_cole != 3) {
          if (schar_cells(mb_schar) > 1) {
            // When the first char to be concealed is double-width,
            // need to advance one more virtual column.
            wlv.n_extra++;
          }

          // First time at this concealed item: display one
          // character.
          if (has_match_conc && match_conc) {
            mb_schar = schar_from_char(match_conc);
          } else if (decor_conceal && decor_state.conceal_char) {
            mb_schar = decor_state.conceal_char;
            if (decor_state.conceal_attr) {
              wlv.char_attr = decor_state.conceal_attr;
            }
          } else if (syn_get_sub_char() != NUL) {
            mb_schar = schar_from_char(syn_get_sub_char());
          } else if (wp->w_p_lcs_chars.conceal != NUL) {
            mb_schar = wp->w_p_lcs_chars.conceal;
          } else {
            mb_schar = schar_from_ascii(' ');
          }

          mb_c = schar_get_first_codepoint(mb_schar);

          prev_syntax_id = syntax_seqnr;

          if (wlv.n_extra > 0) {
            wlv.vcol_off_co += wlv.n_extra;
          }
          wlv.vcol += wlv.n_extra;
          if (is_wrapped && wlv.n_extra > 0) {
            wlv.boguscols += wlv.n_extra;
            wlv.col += wlv.n_extra;
          }
          wlv.n_extra = 0;
          wlv.n_attr = 0;
        } else if (wlv.skip_cells == 0) {
          is_concealing = true;
          wlv.skip_cells = 1;
        }
      } else {
        prev_syntax_id = 0;
        is_concealing = false;
      }

      if (wlv.skip_cells > 0 && did_decrement_ptr) {
        // not showing the '>', put pointer back to avoid getting stuck
        ptr++;
      }
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

