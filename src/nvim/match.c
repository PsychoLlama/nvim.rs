// match.c: functions for highlighting matches

#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval/funcs.h"
#include "nvim/ex_docmd.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/window.h"
#include "nvim/fold.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/macros_defs.h"
#include "nvim/match.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option_vars.h"
#include "nvim/pos_defs.h"
#include "nvim/profile.h"
#include "nvim/regexp.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

#include "match.c.generated.h"

// Static assertions: guard constants that Rust hardcodes in the match crate.
// If any of these fail, update the corresponding constant in src/nvim-rs/match/src/lib.rs.
_Static_assert(RE_MAGIC == 1, "RE_MAGIC changed: update Rust constant RE_MAGIC in match/src/lib.rs");
_Static_assert(UPD_SOME_VALID == 35, "UPD_SOME_VALID changed: update Rust constant in match/src/lib.rs");
_Static_assert(UPD_VALID == 10, "UPD_VALID changed: update Rust constant in match/src/lib.rs");
_Static_assert(HLF_L == 8, "HLF_L changed: update Rust constant in match/src/lib.rs");
_Static_assert(HLF_LC == 9, "HLF_LC changed: update Rust constant in match/src/lib.rs");

/// Get the head of the match list for a window.
matchitem_T *nvim_match_get_head(win_T *wp) { return wp->w_match_head; }

/// Set the head of the match list for a window.
void nvim_match_set_head(win_T *wp, matchitem_T *head) { wp->w_match_head = head; }

/// Get the next match ID for a window.
int nvim_match_get_next_id(win_T *wp) { return wp->w_next_match_id; }

/// Set the next match ID for a window.
void nvim_match_set_next_id(win_T *wp, int id) { wp->w_next_match_id = id; }

/// Get the next match item in the linked list.
matchitem_T *nvim_match_item_next(matchitem_T *m) { return m != NULL ? m->mit_next : NULL; }

/// Set the next pointer of a match item.
void nvim_match_item_set_next(matchitem_T *m, matchitem_T *next) { if (m != NULL) { m->mit_next = next; } }

int nvim_match_item_get_id(matchitem_T *m) { return m != NULL ? m->mit_id : 0; }

int nvim_match_item_get_priority(matchitem_T *m) { return m != NULL ? m->mit_priority : 0; }

/// Get the highlight group ID of a match item.
int nvim_match_item_get_hlg_id(matchitem_T *m) { return m != NULL ? m->mit_hlg_id : 0; }

/// Get the conceal character of a match item.
int nvim_match_item_get_conceal_char(matchitem_T *m) { return m != NULL ? m->mit_conceal_char : 0; }

/// Get the top line number for position matches.
linenr_T nvim_match_item_get_toplnum(matchitem_T *m) { return m != NULL ? m->mit_toplnum : 0; }

/// Get the bottom line number for position matches.
linenr_T nvim_match_item_get_botlnum(matchitem_T *m) { return m != NULL ? m->mit_botlnum : 0; }

/// Get the pattern of a match item (may be NULL for position matches).
const char *nvim_match_item_get_pattern(matchitem_T *m) { return m != NULL ? m->mit_pattern : NULL; }

/// Check if a match item has a pattern (vs positions).
bool nvim_match_item_has_pattern(matchitem_T *m) { return m != NULL && m->mit_pattern != NULL; }

/// Check if a match item has positions.
bool nvim_match_item_has_positions(matchitem_T *m) { return m != NULL && m->mit_pos_array != NULL && m->mit_pos_count > 0; }

/// Get the position count for a match item.
int nvim_match_item_get_pos_count(matchitem_T *m) { return m != NULL ? m->mit_pos_count : 0; }

/// Allocate a new match item.
matchitem_T *nvim_match_alloc(void) { return xcalloc(1, sizeof(matchitem_T)); }

/// Free a match item and all its resources.
void nvim_match_free(matchitem_T *m) { if (m != NULL) { vim_regfree(m->mit_match.regprog); xfree(m->mit_pattern); xfree(m->mit_pos_array); xfree(m); } }

/// Allocate position array for a match item.
llpos_T *nvim_match_alloc_positions(size_t count) { if (count == 0) { return NULL; } return xcalloc(count, sizeof(llpos_T)); }

/// Set the ID of a match item.
void nvim_match_item_set_id(matchitem_T *m, int id) { if (m != NULL) { m->mit_id = id; } }

/// Set the priority of a match item.
void nvim_match_item_set_priority(matchitem_T *m, int priority) { if (m != NULL) { m->mit_priority = priority; } }

/// Set the pattern of a match item (xstrdup).
void nvim_match_item_set_pattern(matchitem_T *m, const char *pat) { if (m != NULL) { m->mit_pattern = pat != NULL ? xstrdup(pat) : NULL; } }

/// Set the highlight group ID of a match item.
void nvim_match_item_set_hlg_id(matchitem_T *m, int hlg_id) { if (m != NULL) { m->mit_hlg_id = hlg_id; } }

/// Set the conceal character of a match item.
void nvim_match_item_set_conceal_char(matchitem_T *m, int ch) { if (m != NULL) { m->mit_conceal_char = ch; } }

/// Set the top line number for position matches.
void nvim_match_item_set_toplnum(matchitem_T *m, linenr_T lnum) { if (m != NULL) { m->mit_toplnum = lnum; } }

/// Set the bottom line number for position matches.
void nvim_match_item_set_botlnum(matchitem_T *m, linenr_T lnum) { if (m != NULL) { m->mit_botlnum = lnum; } }

/// Set the regprog of a match item.
void nvim_match_item_set_regprog(matchitem_T *m, regprog_T *regprog) { if (m != NULL) { m->mit_match.regprog = regprog; } }

/// Set rmm_ic of a match item.
void nvim_match_item_set_rmm_ic(matchitem_T *m, int ic) { if (m != NULL) { m->mit_match.rmm_ic = (bool)ic; } }

/// Set rmm_maxcol of a match item.
void nvim_match_item_set_rmm_maxcol(matchitem_T *m, colnr_T maxcol) { if (m != NULL) { m->mit_match.rmm_maxcol = maxcol; } }

/// Set position array and count for a match item.
void nvim_match_item_set_pos_array(matchitem_T *m, llpos_T *arr, int count) { if (m != NULL) { m->mit_pos_array = arr; m->mit_pos_count = count; } }

/// Set a single position entry in a position array.
void nvim_match_pos_set(llpos_T *arr, int idx, linenr_T lnum, colnr_T col, int len) { if (arr != NULL && idx >= 0) { arr[idx].lnum = lnum; arr[idx].col = col; arr[idx].len = len; } }

/// Get lnum field of a match_T.
linenr_T nvim_match_hl_get_lnum(match_T *shl) { return shl->lnum; }

/// Set has_cursor field of a match_T.
void nvim_match_hl_set_has_cursor(match_T *shl, int val) { shl->has_cursor = (bool)val; }

/// Get startcol field of a match_T.
colnr_T nvim_match_hl_get_startcol(match_T *shl) { return shl->startcol; }

/// Get endcol field of a match_T.
colnr_T nvim_match_hl_get_endcol(match_T *shl) { return shl->endcol; }

/// Get attr field of a match_T.
int nvim_match_hl_get_attr(match_T *shl) { return shl->attr; }

/// Get is_addpos field of a match_T.
int nvim_match_hl_get_is_addpos(match_T *shl) { return shl->is_addpos ? 1 : 0; }

/// Get rm.startpos[idx].lnum from a match_T.
linenr_T nvim_match_hl_rm_startpos_lnum(match_T *shl, int idx) { return shl->rm.startpos[idx].lnum; }

/// Get rm.startpos[idx].col from a match_T.
colnr_T nvim_match_hl_rm_startpos_col(match_T *shl, int idx) { return shl->rm.startpos[idx].col; }

/// Get rm.endpos[idx].lnum from a match_T.
linenr_T nvim_match_hl_rm_endpos_lnum(match_T *shl, int idx) { return shl->rm.endpos[idx].lnum; }

/// Get rm.endpos[idx].col from a match_T.
colnr_T nvim_match_hl_rm_endpos_col(match_T *shl, int idx) { return shl->rm.endpos[idx].col; }

/// Get pointer to mit_hl of a match item.
match_T *nvim_match_item_get_hl(matchitem_T *m) { return m != NULL ? &m->mit_hl : NULL; }

/// Set lnum field of a match_T.
void nvim_match_hl_set_lnum(match_T *shl, linenr_T lnum) { shl->lnum = lnum; }

/// Set is_addpos field of a match_T.
void nvim_match_hl_set_is_addpos(match_T *shl, int val) { shl->is_addpos = (bool)val; }

/// Set rm.startpos[idx] of a match_T.
void nvim_match_hl_rm_set_startpos(match_T *shl, int idx, linenr_T lnum, colnr_T col) { shl->rm.startpos[idx].lnum = lnum; shl->rm.startpos[idx].col = col; }

/// Set rm.endpos[idx] of a match_T.
void nvim_match_hl_rm_set_endpos(match_T *shl, int idx, linenr_T lnum, colnr_T col) { shl->rm.endpos[idx].lnum = lnum; shl->rm.endpos[idx].col = col; }

/// Get mit_pos_cur of a match item.
int nvim_match_item_get_pos_cur(matchitem_T *m) { return m != NULL ? m->mit_pos_cur : 0; }

/// Set mit_pos_cur of a match item.
void nvim_match_item_set_pos_cur(matchitem_T *m, int cur) { if (m != NULL) { m->mit_pos_cur = cur; } }

/// Get lnum from a position in the match item's position array.
linenr_T nvim_match_item_pos_get_lnum(matchitem_T *m, int idx) { if (m == NULL || m->mit_pos_array == NULL || idx < 0 || idx >= m->mit_pos_count) { return 0; } return m->mit_pos_array[idx].lnum; }

/// Get col from a position in the match item's position array.
colnr_T nvim_match_item_pos_get_col(matchitem_T *m, int idx) { if (m == NULL || m->mit_pos_array == NULL || idx < 0 || idx >= m->mit_pos_count) { return 0; } return m->mit_pos_array[idx].col; }

/// Get len from a position in the match item's position array.
int nvim_match_item_pos_get_len(matchitem_T *m, int idx) { if (m == NULL || m->mit_pos_array == NULL || idx < 0 || idx >= m->mit_pos_count) { return 0; } return m->mit_pos_array[idx].len; }

/// Swap two positions in the match item's position array.
void nvim_match_item_pos_swap(matchitem_T *m, int idx1, int idx2) { if (m == NULL || m->mit_pos_array == NULL) { return; } if (idx1 < 0 || idx1 >= m->mit_pos_count || idx2 < 0 || idx2 >= m->mit_pos_count) { return; } llpos_T tmp = m->mit_pos_array[idx1]; m->mit_pos_array[idx1] = m->mit_pos_array[idx2]; m->mit_pos_array[idx2] = tmp; }

/// Get buf field of a match_T.
buf_T *nvim_match_hl_get_buf(match_T *shl) { return shl->buf; }

/// Set buf field of a match_T.
void nvim_match_hl_set_buf(match_T *shl, buf_T *buf) { shl->buf = buf; }

/// Get first_lnum field of a match_T.
linenr_T nvim_match_hl_get_first_lnum(match_T *shl) { return shl->first_lnum; }

/// Set first_lnum field of a match_T.
void nvim_match_hl_set_first_lnum(match_T *shl, linenr_T lnum) { shl->first_lnum = lnum; }

/// Set attr field of a match_T.
void nvim_match_hl_set_attr(match_T *shl, int attr) { shl->attr = attr; }

/// Get the tm (proftime_T) from a match_T, cast to opaque pointer.
void *nvim_match_hl_get_tm_ptr(match_T *shl) { return &shl->tm; }

/// Set the tm (proftime_T) of a match_T from a profile_setlimit call.
void nvim_match_hl_set_tm(match_T *shl, int64_t msec) { shl->tm = profile_setlimit(msec); }

/// Get the regprog from a match_T.
regprog_T *nvim_match_hl_get_regprog(match_T *shl) { return shl->rm.regprog; }

/// Set the regprog of a match_T (does NOT free the old one).
void nvim_match_hl_set_regprog(match_T *shl, regprog_T *rp) { shl->rm.regprog = rp; }

/// Copy rm from match item to match_T: shl->rm = m->mit_match.
void nvim_match_hl_copy_rm_from_item(match_T *shl, matchitem_T *m) { if (m != NULL) { shl->rm = m->mit_match; } }

/// Sync regprog: m->mit_match.regprog = shl->rm.regprog.
void nvim_match_item_sync_regprog(matchitem_T *m, match_T *shl) { if (m != NULL) { m->mit_match.regprog = shl->rm.regprog; } }

/// Check if shl's regprog is a copy of the match item's.
int nvim_match_hl_regprog_is_copy(match_T *shl, matchitem_T *cur) { if (cur == NULL) { return 0; } return (shl == &cur->mit_hl && cur->mit_match.regprog == cur->mit_hl.rm.regprog) ? 1 : 0; }

/// Set startcol field of a match_T.
void nvim_match_hl_set_startcol(match_T *shl, colnr_T col) { shl->startcol = col; }

/// Set endcol field of a match_T.
void nvim_match_hl_set_endcol(match_T *shl, colnr_T col) { shl->endcol = col; }

/// Get attr_cur field of a match_T.
int nvim_match_hl_get_attr_cur(match_T *shl) { return shl->attr_cur; }

/// Set attr_cur field of a match_T.
void nvim_match_hl_set_attr_cur(match_T *shl, int attr) { shl->attr_cur = attr; }

/// Get has_cursor field of a match_T.
int nvim_match_hl_get_has_cursor(match_T *shl) { return shl->has_cursor ? 1 : 0; }

// C function wrappers for Rust to call (prefixed nvim_match_ to avoid symbol conflicts).

int nvim_match_syn_check_group(const char *grp, size_t len) { return syn_check_group(grp, len); }

regprog_T *nvim_match_vim_regcomp(const char *pat, int flags) { return vim_regcomp(pat, flags); }

int nvim_match_utf_ptr2char(const char *p) { return utf_ptr2char(p); }

void nvim_match_redraw_later(win_T *wp, int type) { redraw_later(wp, type); }

void nvim_match_redraw_win_range_later(win_T *wp, linenr_T top, linenr_T bot) { redraw_win_range_later(wp, top, bot); }

/// Wrapper for vim_regexec_multi with timer support.
int nvim_match_vim_regexec_multi(match_T *shl, win_T *win, linenr_T lnum, colnr_T col, int *timed_out) { return vim_regexec_multi(&shl->rm, win, shl->buf, lnum, col, &shl->tm, timed_out); }

void nvim_match_vim_regfree(regprog_T *rp) { vim_regfree(rp); }

void nvim_match_set_no_hlsearch(int flag) { set_no_hlsearch((bool)flag); }

/// Wrapper for re_multiline.
int nvim_match_re_multiline(regprog_T *rp) { if (rp == NULL) { return 0; } return re_multiline(rp); }

/// Wrapper for ml_get_buf returning a byte at position.
/// Returns 0 (NUL) if the position is at or past end of line.
int nvim_match_ml_get_byte(buf_T *buf, linenr_T lnum, colnr_T col) { char *line = ml_get_buf(buf, lnum); return (unsigned char)line[col]; }

/// Wrapper for utfc_ptr2len at a position in a buffer line.
int nvim_match_utfc_ptr2len(buf_T *buf, linenr_T lnum, colnr_T col) { char *line = ml_get_buf(buf, lnum); return utfc_ptr2len(line + col); }

/// Check if CPO_SEARCH is set in 'cpoptions'.
int nvim_match_has_cpo_search(void) { return vim_strchr(p_cpo, CPO_SEARCH) != NULL ? 1 : 0; }

/// Get search_first_line global.
linenr_T nvim_match_get_search_first_line(void) { return search_first_line; }

/// Get search_last_line global.
linenr_T nvim_match_get_search_last_line(void) { return search_last_line; }

/// Get p_rdt (redrawtime option).
int64_t nvim_match_get_p_rdt(void) { return p_rdt; }

// nvim_match_get_HLF_L() deleted — Rust uses constant HLF_L = 8 (with _Static_assert guard).

/// Get the window's buffer.
buf_T *nvim_match_win_get_buffer(win_T *wp) { return wp->w_buffer; }

/// Get the window's topline.
linenr_T nvim_match_win_get_topline(win_T *wp) { return wp->w_topline; }

/// Check if a line has folding (simplified: just checks existence).
int nvim_match_hasFolding(win_T *wp, linenr_T lnum) { return hasFolding(wp, lnum, NULL, NULL) ? 1 : 0; }

/// Refresh *line pointer by calling ml_get_buf.
void nvim_match_ml_get_buf_line(win_T *wp, linenr_T lnum, char **line) { *line = ml_get_buf(wp->w_buffer, lnum); }

/// Get byte at position col in line.
int nvim_match_line_byte_at(const char *line, colnr_T col) { return (unsigned char)line[col]; }

/// Get utfc_ptr2len at position col in line.
int nvim_match_utfc_ptr2len_at(const char *line, colnr_T col) { return utfc_ptr2len(line + col); }

/// Get syn_name2id for "Conceal".
int nvim_match_syn_name2id_conceal(void) { return syn_name2id("Conceal"); }

/// Set search_hl_has_cursor_lnum global.
void nvim_match_set_search_hl_has_cursor_lnum(linenr_T lnum) { search_hl_has_cursor_lnum = lnum; }

// nvim_match_get_HLF_LC() deleted — Rust uses constant HLF_LC = 9 (with _Static_assert guard).

// nvim_semsg_id_taken, nvim_semsg_invalid_id, nvim_semsg_invalid_delete_id,
// nvim_semsg_id_not_found, nvim_semsg_invarg2 migrated to Rust (match/src/core.rs).

// nvim_get_RE_MAGIC, nvim_get_UPD_SOME_VALID, nvim_get_UPD_VALID deleted —
// Rust uses constants RE_MAGIC=1, UPD_SOME_VALID=35, UPD_VALID=10 (with _Static_assert guards).

/// Wrapper for utf_char2bytes.
int nvim_match_utf_char2bytes(int c, char *buf) { return utf_char2bytes(c, buf); }

// f_getmatches, f_setmatches, f_matcharg migrated to Rust (see src/nvim-rs/match/src/viml.rs).

