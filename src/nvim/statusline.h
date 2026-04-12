#pragma once

#include <stdbool.h>
#include <stddef.h>

#include "nvim/highlight_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/option_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"     // IWYU pragma: keep
#include "nvim/statusline_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

/// Click defs type selector for nvim_stl_win_get/set_click_defs.
#define STL_CLICK_DEFS_STATUS    0
#define STL_CLICK_DEFS_WINBAR    1
#define STL_CLICK_DEFS_STATUSCOL 2

/// Batch click defs result for nvim_stl_win_get_click_defs.
typedef struct {
  void *ptr;   ///< pointer to click defs array
  size_t size; ///< number of elements in the array
} StlClickDefsResult;

/// Batch sign info returned by nvim_stl_stcp_get_sign_info.
typedef struct {
  int has_text;     ///< 1 if sattrs[idx].text[0] != 0
  int hl_id;        ///< sattrs[idx].hl_id
  int sign_cul_id;  ///< stcp->sign_cul_id (same for all indices)
} StlSignInfo;

/// Batch cursor info returned by nvim_stl_get_win_cursor_info.
typedef struct {
  int clamped_lnum;   ///< cursor lnum clamped to ml_line_count
  int byte_value;     ///< byte at cursor position (0 if empty)
  int byte_offset;    ///< byte offset at cursor (from rs_ml_find_line_or_offset)
  int ml_empty;       ///< 1 if buffer ML_EMPTY flag is set
  int empty_line;     ///< 1 if cursor line is empty (first char is NUL)
  int cursor_invalid; ///< 1 if cursor lnum > line count
  int first_char;     ///< first char of cursor line (uint8_t cast to int)
} StlCursorInfo;

/// Array defining what should be done when tabline is clicked
EXTERN StlClickDefinition *tab_page_click_defs INIT( = NULL);
/// Size of the tab_page_click_defs array
EXTERN size_t tab_page_click_defs_size INIT( = 0);

// Declarations for functions implemented in Rust (via #[export_name]).
// These replace the auto-generated declarations for the deleted C thin wrappers.
#ifndef DLLEXPORT
#  ifdef MSWIN
#    define DLLEXPORT __declspec(dllexport)
#  else
#    define DLLEXPORT
#  endif
#endif

DLLEXPORT void win_redr_status(win_T *wp);
DLLEXPORT void get_trans_bufname(buf_T *buf);
DLLEXPORT bool stl_connected(win_T *wp);
DLLEXPORT void stl_clear_click_defs(StlClickDefinition *const click_defs, const size_t click_defs_size);
DLLEXPORT StlClickDefinition *stl_alloc_click_defs(StlClickDefinition *cdp, int width, size_t *size);
DLLEXPORT void stl_fill_click_defs(StlClickDefinition *click_defs, StlClickRecord *click_recs, const char *buf, int width, bool tabline);
DLLEXPORT void win_redr_winbar(win_T *wp);
DLLEXPORT void redraw_ruler(void);
DLLEXPORT schar_T fillchar_status(hlf_T *group, win_T *wp);
DLLEXPORT void redraw_custom_statusline(win_T *wp);
DLLEXPORT void draw_tabline(void);
DLLEXPORT int build_statuscol_str(win_T *wp, linenr_T lnum, linenr_T relnum, char *buf, statuscol_T *stcp);
DLLEXPORT int build_stl_str_hl(win_T *wp, char *out, size_t outlen, char *fmt, OptIndex opt_idx, int opt_scope, schar_T fillchar, int maxwidth, stl_hlrec_t **hltab, size_t *hltab_len, StlClickRecord **tabtab, statuscol_T *stcp);

#include "statusline.h.generated.h"
