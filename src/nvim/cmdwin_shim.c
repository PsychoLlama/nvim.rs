// cmdwin_shim.c: C accessor functions for nvim_open_cmdwin Rust migration.
// These bridge the gap between the Rust implementation and C globals/functions.

#include <stdbool.h>
#include <stddef.h>

#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/clipboard.h"
#include "nvim/cmdhist.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_getln.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/globals.h"
#include "nvim/mapping.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/memory_defs.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/popupmenu.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"
#include "nvim/cmdwin.h"

#include "cmdwin_shim.c.generated.h"

// Rust helpers used by this shim
extern int rs_win_valid(win_T *win);
extern int rs_last_window(win_T *win);

// =============================================================================
// Global state: cmdwin_* variables
// =============================================================================

void nvim_set_cmdwin_type(int val) { cmdwin_type = val; }
void nvim_set_cmdwin_level(int val) { cmdwin_level = val; }
void nvim_set_cmdwin_win(win_T *wp) { cmdwin_win = wp; }
void nvim_set_cmdwin_old_curwin(win_T *wp) { cmdwin_old_curwin = wp; }
void nvim_set_cmdwin_buf(buf_T *buf) { cmdwin_buf = buf; }
int nvim_get_cmdwin_result(void) { return cmdwin_result; }

// =============================================================================
// Global state: misc global variables
// =============================================================================

int nvim_get_cmdmsg_rl(void) { return cmdmsg_rl ? 1 : 0; }
void nvim_set_need_wait_return(int val) { need_wait_return = (val != 0); }
void nvim_set_KeyTyped(int val) { KeyTyped = (val != 0); }
void nvim_set_skip_win_fix_cursor(int val) { skip_win_fix_cursor = (val != 0); }
void nvim_set_exmode_active(int val) { exmode_active = (val != 0); }

// =============================================================================
// curwin / curbuf field accessors
// =============================================================================

void nvim_curwin_set_w_p_rl(int val) { curwin->w_p_rl = (val != 0); }
void nvim_curwin_set_w_p_cole(int val) { curwin->w_p_cole = (OptInt)val; }
void nvim_curbuf_inc_ro_locked(void) { curbuf->b_ro_locked++; }
void nvim_curbuf_dec_ro_locked(void) { curbuf->b_ro_locked--; }

// =============================================================================
// cmdmod accessors
// =============================================================================

void nvim_set_cmdmod_tab_zero(void) { cmdmod.cmod_tab = 0; }
void nvim_set_cmdmod_noswapfile(void) { cmdmod.cmod_flags |= CMOD_NOSWAPFILE; }

// =============================================================================
// Window / buffer management wrappers
// =============================================================================

/// Open a split window at the bottom for cmdwin. Returns FAIL or OK.
int nvim_win_split_bot(int height) { return win_split(height, WSP_BOT); }

/// Open a scratch buffer for cmdwin. Returns OK or FAIL.
int nvim_buf_open_scratch_cmdwin(void) { return buf_open_scratch(0, NULL); }

/// Close a cmdwin window if valid and not the last.
/// Returns 1 if win_close was called, 0 otherwise.
int nvim_win_close_if_valid_not_last(win_T *wp)
{
  if (rs_win_valid(wp) && !rs_last_window(wp)) {
    win_close(wp, true, false);
    return 1;
  }
  return 0;
}

/// win_close wrapper (force=true, free_buf=false)
void nvim_win_close_cmdwin(win_T *wp) { win_close(wp, true, false); }

/// close_buffer with DOBUF_WIPE if bufref is valid and not curbuf.
/// curbuf_ptr is the current buffer to compare against.
void nvim_close_buffer_wipe_if_valid(void *bufref_void, buf_T *curbuf_ptr)
{
  bufref_T *bufref = (bufref_T *)bufref_void;
  if (bufref_valid(bufref) && bufref->br_buf != curbuf_ptr) {
    close_buffer(NULL, bufref->br_buf, DOBUF_WIPE, false, false);
  }
}


/// normal_enter(true, false) for cmdwin main loop.
void nvim_normal_enter_cmdwin(void) { normal_enter(true, false); }

// =============================================================================
// beep / pum
// =============================================================================

void nvim_pum_undisplay_true(void) { pum_undisplay(true); }

// =============================================================================
// Autocommand triggers for cmdwin
// =============================================================================

void nvim_trigger_cmdwinenter(void)
{
  char typestr[2] = { (char)cmdwin_type, NUL };
  apply_autocmds(EVENT_CMDWINENTER, typestr, typestr, false, curbuf);
}

void nvim_trigger_cmdwinleave(void)
{
  char typestr[2] = { (char)cmdwin_type, NUL };
  apply_autocmds(EVENT_CMDWINLEAVE, typestr, typestr, false, curbuf);
}

// =============================================================================
// Option setting wrappers
// =============================================================================

void nvim_set_opt_bufhidden_wipe(void)
{
  set_option_value_give_err(kOptBufhidden, STATIC_CSTR_AS_OPTVAL("wipe"), OPT_LOCAL);
}

void nvim_set_opt_filetype_vim(void)
{
  set_option_value_give_err(kOptFiletype, STATIC_CSTR_AS_OPTVAL("vim"), OPT_LOCAL);
}

// =============================================================================
// Mapping wrappers
// =============================================================================

void nvim_add_tab_map_insert(void) { add_map("<Tab>", "<C-X><C-V>", MODE_INSERT, true); }
void nvim_add_tab_map_normal(void) { add_map("<Tab>", "a<C-X><C-V>", MODE_NORMAL, true); }

// =============================================================================
// History accessors
// =============================================================================

int nvim_get_hisidx(int histtype) { return *get_hisidx(histtype); }

/// Get history entry string for histtype at index i. Returns NULL if none.
const char *nvim_get_histentry_str(int histtype, int i)
{
  return get_histentry(histtype)[i].hisstr;
}

// =============================================================================
// Buffer/line read/write
// =============================================================================

/// ml_replace the last line with nvim_get_ccline_cmdbuff().
void nvim_ml_replace_last_with_cmdbuff(void)
{
  ml_replace(curbuf->b_ml.ml_line_count, nvim_get_ccline_cmdbuff(), true);
}

/// ml_append wrapper for history filling. lnum is 0-based line to append after.
int nvim_ml_append_cmdwin(int lnum, const char *line) { return ml_append((linenr_T)lnum, (char *)line, 0, false); }

// =============================================================================
// UI / redraw
// =============================================================================

void nvim_invalidate_botline_curwin(void) { invalidate_botline(curwin); }
void nvim_redraw_later_curwin_some_valid(void) { redraw_later(curwin, UPD_SOME_VALID); }

int nvim_ui_has_cmdline(void) { return ui_has(kUICmdline) ? 1 : 0; }

void nvim_ui_call_cmdline_hide_ccline(int do_flush)
{
  ui_call_cmdline_hide(nvim_get_ccline_level(), (bool)do_flush);
}

int nvim_cmd_screencol_cmdpos(void) { return cmd_screencol(nvim_get_ccline_cmdpos()); }
// =============================================================================
// Getchar / typeahead wrappers
// =============================================================================



// =============================================================================
// Clipboard batch count
// =============================================================================



// =============================================================================
// Command-line buffer management
// =============================================================================


// =============================================================================
// Misc wrappers
// =============================================================================

/// Set ccline cmdpos to curwin->w_cursor.col.
void nvim_set_ccline_cmdpos_from_cursor(void) { nvim_set_ccline_cmdpos((int)curwin->w_cursor.col); }

/// Set the error message for cmdwin state corruption.
void nvim_emsg_cmdwin_changed(void)
{
  emsg(_("E199: Active window or buffer changed or deleted"));
}

/// Perform full cleanup after cmdwin result extraction:
/// restore restart_edit, cmdmsg_rl, State; call may_trigger_modechanged/setmouse/setcursor.
/// This is handled inline in Rust using individual accessors, not here.

/// Set ccline cmdbuff to xmemdupz of a literal string.
/// Used for :qa / :qa! shortcuts.
void nvim_set_ccline_cmdbuff_qa(int with_bang)
{
  const char *p = with_bang ? "qa!" : "qa";
  size_t plen = with_bang ? 3 : 2;
  nvim_set_ccline_cmdbuff(xmemdupz(p, plen));
  nvim_set_ccline_cmdlen((int)plen);
  nvim_set_ccline_cmdbufflen((int)plen + 1);
}

/// Set ccline cmdbuff to empty string (xmemdupz("", 0)).
void nvim_set_ccline_cmdbuff_empty(void)
{
  nvim_set_ccline_cmdbuff(xmemdupz("", 0));
  nvim_set_ccline_cmdlen(0);
  nvim_set_ccline_cmdbufflen(1);
  nvim_set_ccline_cmdpos(0);
}

/// Set ccline cmdbuff to xstrnsave of cursor line.
void nvim_set_ccline_cmdbuff_from_cursor(void)
{
  nvim_set_ccline_cmdlen((int)get_cursor_line_len());
  nvim_set_ccline_cmdbufflen(nvim_get_ccline_cmdlen() + 1);
  nvim_set_ccline_cmdbuff(xstrnsave(get_cursor_line_ptr(), (size_t)nvim_get_ccline_cmdlen()));
}

/// Stuff ':qa\n' or ':qa!\n' into readbuff.
void nvim_stuff_qa_into_readbuff(int with_bang)
{
  stuffcharReadbuff(':');
  stuffReadbuff(with_bang ? "qa!" : "qa");
  stuffcharReadbuff(CAR);
}

/// Allocate and return a zeroed garray_T on the heap.
void *nvim_alloc_garray(void) { return xcalloc(1, sizeof(garray_T)); }

/// Free a heap-allocated garray_T and clear it.
void nvim_free_garray(void *gap_void)
{
  if (gap_void) {
    ga_clear((garray_T *)gap_void);
    xfree(gap_void);
  }
}

/// Allocate a zeroed bufref_T on the heap.
void *nvim_alloc_bufref(void) { return xcalloc(1, sizeof(bufref_T)); }

/// Free a heap-allocated bufref_T.
void nvim_free_bufref(void *br) { xfree(br); }

/// set_bufref wrapper using heap-allocated bufref (opaque handle).

/// bufref_valid wrapper using heap-allocated bufref (opaque handle).
int nvim_bufref_valid_cmdwin(void *br) { return bufref_valid((bufref_T *)br) ? 1 : 0; }

/// Get br_buf from heap-allocated bufref (opaque handle).
buf_T *nvim_bufref_get_buf_cmdwin(void *br) { return ((bufref_T *)br)->br_buf; }
