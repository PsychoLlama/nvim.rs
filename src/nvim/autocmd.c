// autocmd.c: Autocommand related functions

#include <assert.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/cursor.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/fileio.h"
#include "nvim/getchar.h"
#include "nvim/getchar_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/grid_defs.h"
#include "nvim/hashtab.h"
#include "nvim/highlight_defs.h"
#include "nvim/insexpand.h"
#include "nvim/lua/executor.h"
#include "nvim/map_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/time.h"
#include "nvim/os/time_defs.h"
#include "nvim/path.h"
#include "nvim/profile.h"
#include "nvim/regexp.h"
#include "nvim/runtime.h"
#include "nvim/runtime_defs.h"
#include "nvim/search.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_compositor.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"
#include "nvim/winfloat.h"

#include "auevents_name_map.generated.h"

#include "autocmd.c.generated.h"

// Rust FFI declarations
extern void rs_win_append(win_T *after, win_T *wp, tabpage_T *tp);
extern void rs_win_remove(win_T *wp, tabpage_T *tp);
extern int rs_ins_compl_active(void);
extern void rs_check_lnums(int do_curwin);
extern void rs_check_lnums_nested(int do_curwin);
extern void rs_reset_lnums(void);
extern int rs_valid_tabpage_win(tabpage_T *tpc);
extern win_T *rs_win_find_by_handle(int handle);


extern const char *rs_event_nr2name(int event, int num_events);
extern void rs_aubuflocal_remove(int bufnr);
extern int arg_augroup_get(char **argp);

// C accessor for event_names array (used by Rust)
const char *nvim_get_event_name(int event)
{
  if (event >= 0 && event < NUM_EVENTS) {
    return event_names[event].name;
  }
  return NULL;
}

/// Get the number of autocommands registered for an event.
size_t nvim_get_autocmds_count(int event)
{
  if (event >= 0 && event < NUM_EVENTS) {
    return kv_size(autocmds[event]);
  }
  return 0;
}

// Autocommands are stored in a contiguous vector per event, in definition order.
// Patterns are reference-counted and reused for consecutive autocommands.
static AutoPatCmd *active_apc_list = NULL;  // stack of active autocommands
int current_augroup = AUGROUP_DEFAULT;
// au_need_clean, autocmd_nested, autocmd_nesting, filechangeshell_busy are
// now EXTERN globals declared in autocmd.h (promoted from file-static).
int autocmd_blocked = 0;  // block all autocmds
char *old_termresponse = NULL;

static void aucmd_del(AutoCmd *ac)
{
  if (ac->pat != NULL && --ac->pat->refcount == 0) {
    XFREE_CLEAR(ac->pat->pat);
    vim_regfree(ac->pat->reg_prog);
    xfree(ac->pat);
  }
  ac->pat = NULL;
  if (ac->handler_cmd) {
    XFREE_CLEAR(ac->handler_cmd);
  } else {
    callback_free(&ac->handler_fn);
  }
  XFREE_CLEAR(ac->desc);

  au_need_clean = true;
}

AutoCmdVec *au_get_autocmds_for_event(event_T event)
  FUNC_ATTR_PURE
{
  return &autocmds[(int)event];
}

// free_all_autocmds is now implemented in Rust (rs_free_all_autocmds in lib.rs),
// exported directly under the name "free_all_autocmds" via #[unsafe(export_name)].
// The C declaration in autocmd.h still covers external callers gated by #if EXITFREE.

/// Registers an autocmd. The handler may be a Ex command or callback function, decided by
/// the `handler_cmd` or `handler_fn` args.
///
/// @param handler_cmd Handler Ex command, or NULL if handler is a function (`handler_fn`).
/// @param handler_fn Handler function, ignored if `handler_cmd` is not NULL.
int autocmd_register(int64_t id, event_T event, const char *pat, int patlen, int group, bool once,
                     bool nested, char *desc, const char *handler_cmd, Callback *handler_fn)
{
  // 0 is not a valid group.
  assert(group != 0);

  if (patlen > (int)strlen(pat)) {
    return FAIL;
  }

  const int findgroup = group == AUGROUP_ALL ? current_augroup : group;

  // detect special <buffer[=X]> buffer-local patterns
  const bool is_buflocal = aupat_is_buflocal(pat, patlen);
  int buflocal_nr = 0;

  char buflocal_pat[BUFLOCAL_PAT_LEN];  // for "<buffer=X>"
  if (is_buflocal) {
    buflocal_nr = aupat_get_buflocal_nr(pat, patlen);

    // normalize pat into standard "<buffer>#N" form
    aupat_normalize_buflocal_pat(buflocal_pat, pat, patlen, buflocal_nr);

    pat = buflocal_pat;
    patlen = (int)strlen(buflocal_pat);
  }

  // Try to reuse pattern from the last existing autocommand.
  AutoPat *ap = NULL;
  AutoCmdVec *const acs = &autocmds[(int)event];
  for (ptrdiff_t i = (ptrdiff_t)kv_size(*acs) - 1; i >= 0; i--) {
    ap = kv_A(*acs, i).pat;
    if (ap == NULL) {
      continue;  // Skip deleted autocommands.
    }
    // Set result back to NULL if the last pattern doesn't match.
    if (ap->group != findgroup || ap->patlen != patlen
        || strncmp(pat, ap->pat, (size_t)patlen) != 0) {
      ap = NULL;
    }
    break;
  }

  // No matching pattern found, allocate a new one.
  if (ap == NULL) {
    // refuse to add buffer-local ap if buffer number is invalid
    if (is_buflocal && (buflocal_nr == 0 || buflist_findnr(buflocal_nr) == NULL)) {
      semsg(_("E680: <buffer=%d>: invalid buffer number "), buflocal_nr);
      return FAIL;
    }

    ap = xmalloc(sizeof(AutoPat));

    if (is_buflocal) {
      ap->buflocal_nr = buflocal_nr;
      ap->reg_prog = NULL;
    } else {
      ap->buflocal_nr = 0;
      char *reg_pat = file_pat_to_reg_pat(pat, pat + patlen, &ap->allow_dirs, true);
      if (reg_pat != NULL) {
        ap->reg_prog = vim_regcomp(reg_pat, RE_MAGIC);
      }
      xfree(reg_pat);
      if (reg_pat == NULL || ap->reg_prog == NULL) {
        xfree(ap);
        return FAIL;
      }
    }

    ap->refcount = 0;
    ap->pat = xmemdupz(pat, (size_t)patlen);
    ap->patlen = patlen;

    // need to initialize last_mode for the first ModeChanged autocmd
    if (event == EVENT_MODECHANGED && !has_event(EVENT_MODECHANGED)) {
      get_mode(last_mode);
    }

    // If the event is CursorMoved or CursorMovedI, update the last cursor position
    // position to avoid immediately triggering the autocommand
    if ((event == EVENT_CURSORMOVED && !has_event(EVENT_CURSORMOVED))
        || (event == EVENT_CURSORMOVEDI && !has_event(EVENT_CURSORMOVEDI))) {
      last_cursormoved_win = curwin;
      last_cursormoved = curwin->w_cursor;
    }

    // Initialize the fields checked by the WinScrolled and
    // WinResized trigger to prevent them from firing right after
    // the first autocmd is defined.
    if ((event == EVENT_WINSCROLLED || event == EVENT_WINRESIZED)
        && !(has_event(EVENT_WINSCROLLED) || has_event(EVENT_WINRESIZED))) {
      tabpage_T *save_curtab = curtab;
      FOR_ALL_TABS(tp) {
        unuse_tabpage(curtab);
        use_tabpage(tp);
        snapshot_windows_scroll_size();
      }
      unuse_tabpage(curtab);
      use_tabpage(save_curtab);
    }

    ap->group = group == AUGROUP_ALL ? current_augroup : group;
  }

  ap->refcount++;

  // Add the autocmd at the end of the AutoCmd vector.
  AutoCmd *ac = kv_pushp(autocmds[(int)event]);
  ac->pat = ap;
  ac->id = id;
  if (handler_cmd) {
    ac->handler_cmd = xstrdup(handler_cmd);
  } else {
    ac->handler_cmd = NULL;
    callback_copy(&ac->handler_fn, handler_fn);
  }
  ac->script_ctx = current_sctx;
  ac->script_ctx.sc_lnum += SOURCING_LNUM;
  nlua_set_sctx(&ac->script_ctx);
  ac->once = once;
  ac->nested = nested;
  ac->desc = desc == NULL ? NULL : xstrdup(desc);

  return OK;
}

/// Prepare for executing autocommands for (hidden) buffer `buf`.
/// If the current buffer is not in any visible window, put it in a temporary
/// floating window using an entry in `aucmd_win[]`.
/// Set `curbuf` and `curwin` to match `buf`.
///
/// @param aco  structure to save values in
/// @param buf  new curbuf
void aucmd_prepbuf(aco_save_T *aco, buf_T *buf)
{
  win_T *win;
  bool need_append = true;  // Append `aucmd_win` to the window list.
  const bool same_buffer = buf == curbuf;

  // Find a window that is for the new buffer
  if (same_buffer) {  // be quick when buf is curbuf
    win = curwin;
  } else {
    win = NULL;
    FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
      if (wp->w_buffer == buf) {
        win = wp;
        break;
      }
    }
  }

  // Allocate a window when needed.
  win_T *auc_win = NULL;
  int auc_idx = AUCMD_WIN_COUNT;
  if (win == NULL) {
    for (auc_idx = 0; auc_idx < AUCMD_WIN_COUNT; auc_idx++) {
      if (!aucmd_win[auc_idx].auc_win_used) {
        break;
      }
    }

    if (auc_idx == AUCMD_WIN_COUNT) {
      kv_push(aucmd_win_vec, ((aucmdwin_T){
        .auc_win = NULL,
        .auc_win_used = false,
      }));
    }

    if (aucmd_win[auc_idx].auc_win == NULL) {
      win_alloc_aucmd_win(auc_idx);
      need_append = false;
    }
    auc_win = aucmd_win[auc_idx].auc_win;
    aucmd_win[auc_idx].auc_win_used = true;
  }

  aco->save_curwin_handle = curwin->handle;
  aco->save_prevwin_handle = prevwin == NULL ? 0 : prevwin->handle;
  aco->save_State = State;
  if (bt_prompt(curbuf)) {
    aco->save_prompt_insert = curbuf->b_prompt_insert;
  }

  if (win != NULL) {
    // There is a window for "buf" in the current tab page, make it the
    // curwin.  This is preferred, it has the least side effects (esp. if
    // "buf" is curbuf).
    aco->use_aucmd_win_idx = -1;
    curwin = win;
  } else {
    // There is no window for "buf", use "auc_win".  To minimize the side
    // effects, insert it in the current tab page.
    // Anything related to a window (e.g., setting folds) may have
    // unexpected results.
    aco->use_aucmd_win_idx = auc_idx;
    auc_win->w_buffer = buf;
    auc_win->w_s = &buf->b_s;
    buf->b_nwindows++;
    win_init_empty(auc_win);  // set cursor and topline to safe values

    // Make sure w_localdir, tp_localdir and globaldir are NULL to avoid a
    // chdir() in win_enter_ext().
    XFREE_CLEAR(auc_win->w_localdir);
    aco->tp_localdir = curtab->tp_localdir;
    curtab->tp_localdir = NULL;
    aco->globaldir = globaldir;
    globaldir = NULL;

    block_autocmds();  // We don't want BufEnter/WinEnter autocommands.
    if (need_append) {
      rs_win_append(lastwin, auc_win, NULL);
      pmap_put(int)(&window_handles, auc_win->handle, auc_win);
      win_config_float(auc_win, auc_win->w_config);
    }
    // Prevent chdir() call in win_enter_ext(), through do_autochdir()
    const int save_acd = p_acd;
    p_acd = false;
    // no redrawing and don't set the window title
    RedrawingDisabled++;
    win_enter(auc_win, false);
    RedrawingDisabled--;
    p_acd = save_acd;
    unblock_autocmds();
    curwin = auc_win;
  }
  curbuf = buf;
  aco->new_curwin_handle = curwin->handle;
  set_bufref(&aco->new_curbuf, curbuf);

  aco->save_VIsual_active = VIsual_active;
  if (!same_buffer) {
    // disable the Visual area, position may be invalid in another buffer
    VIsual_active = false;
  }
}

/// Cleanup after executing autocommands for a (hidden) buffer.
/// Restore the window as it was (if possible).
///
/// @param aco  structure holding saved values
void aucmd_restbuf(aco_save_T *aco)
{
  if (aco->use_aucmd_win_idx >= 0) {
    win_T *awp = aucmd_win[aco->use_aucmd_win_idx].auc_win;

    // Find "awp", it can't be closed, but it may be in another tab page.
    // Do not trigger autocommands here.
    block_autocmds();
    if (curwin != awp) {
      FOR_ALL_TAB_WINDOWS(tp, wp) {
        if (wp == awp) {
          if (tp != curtab) {
            goto_tabpage_tp(tp, true, true);
          }
          win_goto(awp);
          goto win_found;
        }
      }
    }
win_found:
    curbuf->b_nwindows--;
    const bool save_stop_insert_mode = stop_insert_mode;
    // May need to stop Insert mode if we were in a prompt buffer.
    leaving_window(curwin);
    // Do not stop Insert mode when already in Insert mode before.
    if (aco->save_State & MODE_INSERT) {
      stop_insert_mode = save_stop_insert_mode;
    }
    // Remove the window.
    rs_win_remove(curwin, NULL);
    pmap_del(int)(&window_handles, curwin->handle, NULL);
    if (curwin->w_grid_alloc.chars != NULL) {
      ui_comp_remove_grid(&curwin->w_grid_alloc);
      ui_call_win_hide(curwin->w_grid_alloc.handle);
      grid_free(&curwin->w_grid_alloc);
    }

    // The window is marked as not used, but it is not freed, it can be
    // used again.
    aucmd_win[aco->use_aucmd_win_idx].auc_win_used = false;

    if (!rs_valid_tabpage_win(curtab)) {
      // no valid window in current tabpage
      close_tabpage(curtab);
    }

    unblock_autocmds();

    win_T *const save_curwin = rs_win_find_by_handle(aco->save_curwin_handle);
    if (save_curwin != NULL) {
      curwin = save_curwin;
    } else {
      // Hmm, original window disappeared.  Just use the first one.
      curwin = firstwin;
    }
    curbuf = curwin->w_buffer;
    // May need to restore insert mode for a prompt buffer.
    entering_window(curwin);
    if (bt_prompt(curbuf)) {
      curbuf->b_prompt_insert = aco->save_prompt_insert;
    }

    prevwin = rs_win_find_by_handle(aco->save_prevwin_handle);
    vars_clear(&awp->w_vars->dv_hashtab);         // free all w: variables
    hash_init(&awp->w_vars->dv_hashtab);          // re-use the hashtab

    // If :lcd has been used in the autocommand window, correct current
    // directory before restoring tp_localdir and globaldir.
    if (awp->w_localdir != NULL) {
      win_fix_current_dir();
    }
    xfree(curtab->tp_localdir);
    curtab->tp_localdir = aco->tp_localdir;
    xfree(globaldir);
    globaldir = aco->globaldir;

    // the buffer contents may have changed
    VIsual_active = aco->save_VIsual_active;
    check_cursor(curwin);
    if (curwin->w_topline > curbuf->b_ml.ml_line_count) {
      curwin->w_topline = curbuf->b_ml.ml_line_count;
      curwin->w_topfill = 0;
    }
  } else {
    // Restore curwin.  Use the window ID, a window may have been closed
    // and the memory re-used for another one.
    win_T *const save_curwin = rs_win_find_by_handle(aco->save_curwin_handle);
    if (save_curwin != NULL) {
      // Restore the buffer which was previously edited by curwin, if it was
      // changed, we are still the same window and the buffer is valid.
      if (curwin->handle == aco->new_curwin_handle
          && curbuf != aco->new_curbuf.br_buf
          && bufref_valid(&aco->new_curbuf)
          && aco->new_curbuf.br_buf->b_ml.ml_mfp != NULL) {
        if (curwin->w_s == &curbuf->b_s) {
          curwin->w_s = &aco->new_curbuf.br_buf->b_s;
        }
        curbuf->b_nwindows--;
        curbuf = aco->new_curbuf.br_buf;
        curwin->w_buffer = curbuf;
        curbuf->b_nwindows++;
      }

      curwin = save_curwin;
      curbuf = curwin->w_buffer;
      prevwin = rs_win_find_by_handle(aco->save_prevwin_handle);

      // In case the autocommand moves the cursor to a position that does not
      // exist in curbuf
      VIsual_active = aco->save_VIsual_active;
      check_cursor(curwin);
    }
  }

  VIsual_active = aco->save_VIsual_active;
  check_cursor(curwin);  // just in case lines got deleted
  if (VIsual_active) {
    check_pos(curbuf, &VIsual);
  }
}


// apply_autocmds_group is implemented in Rust (rs_apply_autocmds_group in autocmd/src/lib.rs)
// and exported directly under the name "apply_autocmds_group" via #[unsafe(export_name)].
// The C declaration in autocmd.h still covers external callers.

/// Executes an autocmd callback function (as opposed to an Ex command).
static bool au_callback(const AutoCmd *ac, const AutoPatCmd *apc)
{
  Callback callback = ac->handler_fn;
  if (callback.type == kCallbackLua) {
    MAXSIZE_TEMP_DICT(data, 7);
    PUT_C(data, "id", INTEGER_OBJ(ac->id));
    PUT_C(data, "event", CSTR_AS_OBJ(rs_event_nr2name((int)apc->event, NUM_EVENTS)));
    PUT_C(data, "file", CSTR_AS_OBJ(apc->afile_orig));
    PUT_C(data, "match", CSTR_AS_OBJ(autocmd_match));
    PUT_C(data, "buf", INTEGER_OBJ(autocmd_bufnr));

    if (apc->data) {
      PUT_C(data, "data", *apc->data);
    }

    int group = ac->pat->group;
    switch (group) {
    case AUGROUP_ERROR:
      abort();  // unreachable
    case AUGROUP_DEFAULT:
    case AUGROUP_ALL:
    case AUGROUP_DELETED:
      // omit group in these cases
      break;
    default:
      PUT_C(data, "group", INTEGER_OBJ(group));
      break;
    }

    MAXSIZE_TEMP_ARRAY(args, 1);
    ADD_C(args, DICT_OBJ(data));

    Object result = nlua_call_ref(callback.data.luaref, NULL, args, kRetNilBool, NULL, NULL);
    return LUARET_TRUTHY(result);
  } else {
    typval_T argsin = TV_INITIAL_VALUE;
    typval_T rettv = TV_INITIAL_VALUE;
    callback_call(&callback, 0, &argsin, &rettv);
    return false;
  }
}

// getnextac is implemented in Rust (rs_getnextac in autocmd/src/lib.rs)
// and exported directly under the name "getnextac" via #[unsafe(export_name)].

/// Gets an (allocated) string representation of an autocmd command/callback.
static char *aucmd_handler_to_string(AutoCmd *ac)
  FUNC_ATTR_PURE
{
  if (ac->handler_cmd) {
    return xstrdup(ac->handler_cmd);
  }
  return callback_to_string(&ac->handler_fn, NULL);
}

// do_filetype_autocmd is implemented in Rust (rs_do_filetype_autocmd in autocmd/src/lib.rs)
// and exported directly under the name "do_filetype_autocmd" via #[unsafe(export_name)].

// nvim_get_autocmd_blocked() removed: Rust accesses autocmd_blocked directly.
int nvim_get_aucmd_win_count(void) { return AUCMD_WIN_COUNT; }

/// Check if aucmd_win at index is used (used by Rust FFI).
int nvim_aucmd_win_used(int idx)
{
  if (idx < 0 || idx >= AUCMD_WIN_COUNT) {
    return 0;
  }
  return aucmd_win[idx].auc_win_used ? 1 : 0;
}

/// Get the window pointer at aucmd_win index (used by Rust FFI).
win_T *nvim_aucmd_win_get_win(int idx)
{
  if (idx < 0 || idx >= AUCMD_WIN_COUNT) {
    return NULL;
  }
  return aucmd_win[idx].auc_win;
}

// nvim_get_did_cursorhold removed: autocmd crate accesses did_cursorhold directly.
// The following are kept because other Rust crates or the autocmd crate still call them:
int nvim_get_reg_recording(void) { return reg_recording; }
int nvim_get_reg_executing(void) { return reg_executing; }
int nvim_get_autocmd_bufnr(void) { return autocmd_bufnr; }
int nvim_get_curbuf_fnum(void) { return curbuf->b_fnum; }
int nvim_get_curbuf_handle(void) { return curbuf->handle; }

/// Returns NUM_EVENTS if not found.
int nvim_event_name2nr(const char *start, size_t len)
{
  int hash_idx = event_name2nr_hash(start, len);
  if (hash_idx < 0) {
    return NUM_EVENTS;
  }
  return abs(event_names[event_hash[hash_idx]].event);
}

/// Negative means window-level event, positive means global-only.
int nvim_get_event_sign(int event)
{
  if (event < 0 || event >= NUM_EVENTS) {
    return 0;
  }
  return event_names[event].event;
}

// nvim_get_p_ei is kept because event.rs and ex_docmd still call it.
const char *nvim_get_p_ei(void) { return p_ei; }
void nvim_autocmd_set_option_eventignore(const char *val) { set_option_direct(kOptEventignore, CSTR_AS_OPTVAL(val), 0, SID_NONE); }

/// Delete the autocmd at index `idx` in event `event` (refcount + free).
void nvim_autocmd_del_at(int event, size_t idx)
{
  AutoCmdVec *const acs = &autocmds[event];
  if (idx < kv_size(*acs)) {
    aucmd_del(&kv_A(*acs, idx));
  }
}

/// Destroy the AutoCmd vector for an event (kv_destroy wrapper for Rust).
/// Used by free_all_autocmds in Rust.
void nvim_autocmd_destroy_event_vec(int event)
{
  kv_destroy(autocmds[event]);
}

/// Get all pattern info for autocmd at (event, idx) in one call.
AutoPatInfo nvim_autocmd_get_pat_info(int event, size_t idx)
{
  AutoCmdVec *const acs = &autocmds[event];
  if (idx >= kv_size(*acs) || kv_A(*acs, idx).pat == NULL) {
    return (AutoPatInfo){ .is_null = 1 };
  }
  AutoCmd *const ac = &kv_A(*acs, idx);
  AutoPat *const ap = ac->pat;
  return (AutoPatInfo){
    .is_null = 0,
    .group = ap->group,
    .buflocal_nr = ap->buflocal_nr,
    .pat = ap->pat,
    .patlen = ap->patlen,
    .pat_id = (uintptr_t)ap,
    .id = ac->id,
  };
}

/// Get handler info for autocmd at (event, idx) in one call.
AutoHandlerInfo nvim_autocmd_get_handler_info(int event, size_t idx)
{
  AutoCmdVec *const acs = &autocmds[event];
  if (idx >= kv_size(*acs)) {
    return (AutoHandlerInfo){ 0 };
  }
  AutoCmd *const ac = &kv_A(*acs, idx);
  return (AutoHandlerInfo){
    .handler_str = aucmd_handler_to_string(ac),
    .desc = ac->desc,
    .has_handler_cmd = ac->handler_cmd != NULL ? 1 : 0,
  };
}

/// Compact the autocmd vector for a given event (remove NULL-pat entries).
void nvim_autocmd_compact_event(int event)
{
  AutoCmdVec *const acs = &autocmds[event];
  size_t nsize = 0;
  for (size_t i = 0; i < kv_size(*acs); i++) {
    AutoCmd *const ac = &kv_A(*acs, i);
    if (nsize != i) {
      kv_A(*acs, nsize) = *ac;
    }
    if (ac->pat != NULL) {
      nsize++;
    }
  }
  if (nsize == 0) {
    kv_destroy(*acs);
  } else {
    acs->size = nsize;
  }
}

// nvim_get_au_need_clean/nvim_set_au_need_clean removed: Rust accesses au_need_clean directly.

/// Walk active_apc_list and invalidate matching arg_bufnr.
void nvim_apc_invalidate_bufnr(int bufnr)
{
  for (AutoPatCmd *apc = active_apc_list; apc != NULL; apc = apc->next) {
    if (bufnr == apc->arg_bufnr) {
      apc->arg_bufnr = 0;
    }
  }
}

/// Verbose message for aubuflocal_remove.
void nvim_verbose_buflocal_remove(int event, int bufnr)
{
  if (p_verbose >= 6) {
    verbose_enter();
    smsg(0, _("auto-removing autocommand: %s <buffer=%d>"), rs_event_nr2name(event, NUM_EVENTS), bufnr);
    verbose_leave();
  }
}

// nvim_skipwhite exists in fold.c
_Static_assert(HLF_8 == 1, "HLF_8 value changed");
_Static_assert(HLF_E == 6, "HLF_E value changed");
_Static_assert(HLF_T == 23, "HLF_T value changed");
_Static_assert(EVENT_TERMRESPONSE == 120, "EVENT_TERMRESPONSE value changed");
_Static_assert(VV_TERMRESPONSE == 11, "VV_TERMRESPONSE value changed");
_Static_assert(FAIL == 0, "FAIL value changed");
_Static_assert(OK == 1, "OK value changed");

/// Check if autocmd at (event, idx) matches file for has_autocmd.
/// This consolidates the match_file_pat + buflocal check.
bool nvim_autocmd_match_file(int event, size_t idx,
                             const char *fname, const char *sfname,
                             const char *tail, int buf_fnum)
{
  AutoCmdVec *const acs = &autocmds[event];
  if (idx >= kv_size(*acs)) {
    return false;
  }
  AutoPat *const ap = kv_A(*acs, idx).pat;
  if (ap == NULL) {
    return false;
  }
  if (ap->buflocal_nr == 0) {
    return match_file_pat(NULL, &ap->reg_prog,
                          (char *)fname, (char *)sfname, (char *)tail,
                          ap->allow_dirs);
  }
  return buf_fnum != 0 && ap->buflocal_nr == buf_fnum;
}

/// Call last_set_msg for the script context of autocmd at (event, idx).
void nvim_autocmd_show_last_set(int event, size_t idx)
{
  AutoCmdVec *const acs = &autocmds[event];
  if (idx < kv_size(*acs)) {
    last_set_msg(kv_A(*acs, idx).script_ctx);
  }
}

void nvim_autocmd_eap_set_nextcmd(void *eap, char *val) { ((exarg_T *)eap)->nextcmd = val; }

/// Delete matching autocmds for a pattern at (event, idx) if they match findgroup/pat/patlen.
void nvim_autocmd_del_matching(int event, int findgroup, const char *pat, int patlen)
{
  AutoCmdVec *const acs = &autocmds[event];
  for (size_t i = 0; i < kv_size(*acs); i++) {
    AutoCmd *const ac = &kv_A(*acs, i);
    AutoPat *const ap = ac->pat;
    if (ap != NULL && ap->group == findgroup && ap->patlen == patlen
        && strncmp(pat, ap->pat, (size_t)patlen) == 0) {
      aucmd_del(ac);
    }
  }
}

/// Register an autocmd with a command string handler. Wraps autocmd_register.
int nvim_autocmd_register_cmd(int event, const char *pat, int patlen, int group,
                              bool once, bool nested, const char *cmd)
{
  Callback handler_fn = CALLBACK_INIT;
  return autocmd_register(0, (event_T)event, pat, patlen, group, once, nested,
                          NULL, cmd, &handler_fn);
}

void *nvim_autocmd_get_curbuf_ptr(void) { return curbuf; }
void nvim_autocmd_semsg_str(const char *fmt, const char *arg) { semsg(fmt, arg); }
void nvim_autocmd_smsg_no_matching(const char *arg_start) { smsg(0, _("No matching autocommands: %s"), arg_start); }

// =============================================================================
// Phase 1: AutoPatCmd field accessors for aucmd_next / getnextac migration
// =============================================================================

int nvim_apc_get_event(const void *apc_raw) { return (int)((const AutoPatCmd *)apc_raw)->event; }
size_t nvim_apc_get_ausize(const void *apc_raw) { return ((const AutoPatCmd *)apc_raw)->ausize; }
size_t nvim_apc_get_auidx(const void *apc_raw) { return ((const AutoPatCmd *)apc_raw)->auidx; }
int nvim_apc_get_group(const void *apc_raw) { return ((const AutoPatCmd *)apc_raw)->group; }
void *nvim_apc_get_lastpat(const void *apc_raw) { return ((const AutoPatCmd *)apc_raw)->lastpat; }
void nvim_apc_set_auidx(void *apc_raw, size_t idx) { ((AutoPatCmd *)apc_raw)->auidx = idx; }

/// Returns the AutoPat* for autocmd at (event, idx), or NULL if deleted.
void *nvim_autocmd_get_ac_pat(int event, size_t idx)
{
  AutoCmdVec *const acs = &autocmds[event];
  if (idx >= kv_size(*acs)) {
    return NULL;
  }
  return kv_A(*acs, idx).pat;
}


/// Get ac->nested for autocmd at (event, idx).
bool nvim_autocmd_get_ac_nested(int event, size_t idx)
{
  AutoCmdVec *const acs = &autocmds[event];
  if (idx >= kv_size(*acs)) {
    return false;
  }
  return kv_A(*acs, idx).nested;
}

/// Get ac->once for autocmd at (event, idx).
bool nvim_autocmd_get_ac_once(int event, size_t idx)
{
  AutoCmdVec *const acs = &autocmds[event];
  if (idx >= kv_size(*acs)) {
    return false;
  }
  return kv_A(*acs, idx).once;
}

/// Set current_sctx from ac->script_ctx for autocmd at (event, idx).
/// Also sets apc->script_ctx = current_sctx.
void nvim_autocmd_set_script_ctx(int event, size_t idx, void *apc_raw)
{
  AutoCmdVec *const acs = &autocmds[event];
  if (idx < kv_size(*acs)) {
    current_sctx = kv_A(*acs, idx).script_ctx;
    if (apc_raw) {
      ((AutoPatCmd *)apc_raw)->script_ctx = current_sctx;
    }
  }
}

/// Return xstrdup of ac->handler_cmd for autocmd at (event, idx), or NULL if handler is function.
char *nvim_autocmd_get_ac_handler_cmd(int event, size_t idx)
{
  AutoCmdVec *const acs = &autocmds[event];
  if (idx >= kv_size(*acs) || kv_A(*acs, idx).handler_cmd == NULL) {
    return NULL;
  }
  return xstrdup(kv_A(*acs, idx).handler_cmd);
}

/// Get the verbose handler string for a autocmd at (event, idx) for verbose output.
/// Returns an allocated string (caller frees).
char *nvim_autocmd_get_handler_str_verbose(int event, size_t idx)
{
  AutoCmdVec *const acs = &autocmds[event];
  if (idx >= kv_size(*acs)) {
    return NULL;
  }
  return aucmd_handler_to_string(&kv_A(*acs, idx));
}

/// Execute the callback from an AutoCmd copy (for safe oneshot handling).
/// ac_copy_raw is a pointer to an AutoCmd copy (stack allocated by Rust via xcalloc or similar).
bool nvim_autocmd_execute_callback_copy(const void *ac_copy_raw, const void *apc_raw)
{
  return au_callback((const AutoCmd *)ac_copy_raw, (const AutoPatCmd *)apc_raw);
}

/// Get the size of AutoCmd struct (for Rust allocation).
size_t nvim_sizeof_autocmd(void) { return sizeof(AutoCmd); }

/// Copy the AutoCmd at (event, idx) into the buffer pointed to by dst.
void nvim_autocmd_copy_ac(int event, size_t idx, void *dst)
{
  AutoCmdVec *const acs = &autocmds[event];
  if (idx < kv_size(*acs)) {
    memcpy(dst, &kv_A(*acs, idx), sizeof(AutoCmd));
  }
}

/// Restore the pat pointer for autocmd at (event, idx) from a saved copy.
/// Used to undo the oneshot pat=NULL trick after callback may have reallocated acs.
void nvim_autocmd_restore_pat(int event, size_t idx, void *ac_copy_raw)
{
  AutoCmdVec *const acs = &autocmds[event];
  if (idx < kv_size(*acs)) {
    kv_A(*acs, idx).pat = ((AutoCmd *)ac_copy_raw)->pat;
  }
}

// nvim_set_autocmd_nested removed: Rust accesses autocmd_nested directly.

/// Temporarily set ac->pat = NULL without freeing (for oneshot hide-during-callback trick).
void nvim_autocmd_set_pat_null(int event, size_t idx)
{
  AutoCmdVec *const acs = &autocmds[event];
  if (idx < kv_size(*acs)) {
    kv_A(*acs, idx).pat = NULL;
  }
}

/// Check whether autocmd at (event, i) should be skipped, returns new pat match status.
/// Returns 0 for skip, 1 for same-pat (already matched), 2 for new match.
/// On new match, also updates the exestack entry and emits verbose message.
int nvim_aucmd_try_match(int event, size_t i, void *apc_raw)
{
  AutoPatCmd *apc = (AutoPatCmd *)apc_raw;
  AutoCmdVec *const acs = &autocmds[event];
  if (i >= kv_size(*acs)) {
    return 0;
  }
  AutoPat *const ap = kv_A(*acs, i).pat;
  if (ap == NULL) {
    return 0;
  }
  if (ap == apc->lastpat) {
    return 1;  // Same pattern, already matched
  }
  if (apc->group != AUGROUP_ALL && apc->group != ap->group) {
    return 0;
  }
  if (ap->buflocal_nr == 0
      ? !match_file_pat(NULL, &ap->reg_prog, apc->fname, apc->sfname, apc->tail, ap->allow_dirs)
      : ap->buflocal_nr != apc->arg_bufnr) {
    return 0;
  }
  return 2;  // New match
}

/// Update exestack entry for a new match at (event, i) in apc, and emit verbose message.
void nvim_aucmd_set_entry(int event, size_t i, void *apc_raw)
{
  AutoPatCmd *apc = (AutoPatCmd *)apc_raw;
  AutoCmdVec *const acs = &autocmds[event];
  AutoPat *const ap = kv_A(*acs, i).pat;
  estack_T *const entry = ((estack_T *)exestack.ga_data) + exestack.ga_len - 1;
  const char *const name = rs_event_nr2name(event, NUM_EVENTS);
  const char *const s = _("%s Autocommands for \"%s\"");
  const size_t sourcing_name_len = strlen(s) + strlen(name) + (size_t)ap->patlen + 1;
  char *const namep = xmalloc(sourcing_name_len);
  snprintf(namep, sourcing_name_len, s, name, ap->pat);
  if (p_verbose >= 8) {
    verbose_enter();
    smsg(0, _("Executing %s"), namep);
    verbose_leave();
  }
  XFREE_CLEAR(entry->es_name);
  entry->es_name = namep;
  entry->es_info.aucmd = apc;
}

/// Clear the exestack entry when no more autocmds remain.
void nvim_aucmd_clear_entry(void)
{
  estack_T *const entry = ((estack_T *)exestack.ga_data) + exestack.ga_len - 1;
  XFREE_CLEAR(entry->es_name);
  entry->es_info.aucmd = NULL;
}

/// Set apc->lastpat from the AutoPat* at (event, i).
void nvim_apc_set_lastpat_from_ac(int event, size_t i, void *apc_raw)
{
  AutoCmdVec *const acs = &autocmds[event];
  ((AutoPatCmd *)apc_raw)->lastpat = kv_A(*acs, i).pat;
}

/// Set apc->lastpat to NULL (signal no more autocmds).
void nvim_apc_set_lastpat_null(void *apc_raw)
{
  ((AutoPatCmd *)apc_raw)->lastpat = NULL;
}

// =============================================================================
// Phase 2: apply_autocmds_group migration accessors
// =============================================================================

// --- Global variable save/restore ---

/// Saved autocmd context (for save/restore around apply_autocmds_group).
typedef struct {
  char *save_autocmd_fname;
  bool save_autocmd_fname_full;
  int save_autocmd_bufnr;
  char *save_autocmd_match;
  int save_autocmd_busy;
  bool save_autocmd_nested;
  bool save_changed;
  buf_T *old_curbuf;
} AutocmdSaveCtx;

/// Save autocmd global variables. Returns heap-allocated context.
void *nvim_autocmd_save_ctx(void)
{
  AutocmdSaveCtx *ctx = xmalloc(sizeof(*ctx));
  ctx->save_autocmd_fname = autocmd_fname;
  ctx->save_autocmd_fname_full = autocmd_fname_full;
  ctx->save_autocmd_bufnr = autocmd_bufnr;
  ctx->save_autocmd_match = autocmd_match;
  ctx->save_autocmd_busy = autocmd_busy;
  ctx->save_autocmd_nested = autocmd_nested;
  ctx->save_changed = curbuf->b_changed;
  ctx->old_curbuf = curbuf;
  return ctx;
}

/// Restore autocmd global variables from saved context. Frees context.
/// Also frees the current autocmd_fname (MAXPATHL copy) before restoring.
void nvim_autocmd_restore_ctx(void *ctx_raw)
{
  AutocmdSaveCtx *ctx = (AutocmdSaveCtx *)ctx_raw;
  autocmd_busy = ctx->save_autocmd_busy;
  autocmd_nested = ctx->save_autocmd_nested;
  xfree(autocmd_fname);  // free MAXPATHL copy set by nvim_autocmd_setup_afile
  autocmd_fname = ctx->save_autocmd_fname;
  autocmd_fname_full = ctx->save_autocmd_fname_full;
  autocmd_bufnr = ctx->save_autocmd_bufnr;
  autocmd_match = ctx->save_autocmd_match;
  xfree(ctx);
}

/// Get the saved old_curbuf from context.
void *nvim_autocmd_ctx_get_old_curbuf(const void *ctx_raw)
{
  return ((const AutocmdSaveCtx *)ctx_raw)->old_curbuf;
}

/// Get the saved b_changed from context.
bool nvim_autocmd_ctx_get_save_changed(const void *ctx_raw)
{
  return ((const AutocmdSaveCtx *)ctx_raw)->save_changed;
}

// --- Set individual autocmd globals ---

// nvim_set_autocmd_fname, nvim_set_autocmd_fname_full, nvim_set_autocmd_bufnr2,
// nvim_set_autocmd_match, nvim_set_autocmd_busy removed: Rust accesses globals directly.
// nvim_get_autocmd_no_enter, nvim_get_autocmd_no_leave removed: Rust accesses globals directly.

// --- Current buffer field accessors ---

bool nvim_get_curbuf_b_did_filetype(void) { return curbuf->b_did_filetype; }
void nvim_set_curbuf_b_did_filetype(bool v) { curbuf->b_did_filetype = v; }
bool nvim_get_curbuf_b_keep_filetype(void) { return curbuf->b_keep_filetype; }
void nvim_set_curbuf_b_au_did_filetype(bool v) { curbuf->b_au_did_filetype = v; }

// --- Buffer field accessors (autocmd-specific variants) ---

int nvim_autocmd_buf_get_fnum(const void *buf) { return ((const buf_T *)buf)->b_fnum; }

// --- Win ignore / eventignorewin check (helpers for Rust migration) ---

/// Return the sign of event_names[event].event (positive = true, non-positive = false).
bool nvim_autocmd_event_is_positive(int event) { return event_names[event].event > 0; }

/// Return true if buf == curbuf.
bool nvim_autocmd_buf_is_curbuf(const void *buf) { return (const buf_T *)buf == curbuf; }

/// Return curwin->w_p_eiw.
const char *nvim_get_curwin_p_eiw(void) { return curwin->w_p_eiw; }

/// Return buf->b_nwindows.
int nvim_autocmd_buf_get_nwindows(const void *buf)
{
  return ((const buf_T *)buf)->b_nwindows;
}

/// Iterate all tab windows; return true if ALL windows showing buf ignore event.
/// Returns false immediately if any window showing buf does NOT ignore the event.
bool nvim_autocmd_all_wins_ignore_event(int event, const void *buf_raw)
{
  const buf_T *buf = (const buf_T *)buf_raw;
  bool all_ignore = true;
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    if (wp->w_buffer == buf && !event_ignored(event, wp->w_p_eiw)) {
      all_ignore = false;
      break;
    }
  }
  return all_ignore;
}

// --- AutoPatCmd lifecycle ---

/// Returns the size of AutoPatCmd (for heap allocation in Rust).
size_t nvim_sizeof_autopatcmd(void) { return sizeof(AutoPatCmd); }

/// Initialize a heap-allocated AutoPatCmd with the given fields.
void nvim_apc_init(void *apc_raw, int event, int group, char *fname, char *sfname,
                   const char *tail, int arg_bufnr, char *afile_orig, void *data)
{
  AutoPatCmd *apc = (AutoPatCmd *)apc_raw;
  memset(apc, 0, sizeof(*apc));
  apc->lastpat = NULL;
  apc->auidx = 0;
  apc->ausize = kv_size(autocmds[event]);
  apc->afile_orig = afile_orig;
  apc->fname = fname;
  apc->sfname = sfname;
  apc->tail = (char *)tail;
  apc->group = group;
  apc->event = (event_T)event;
  apc->arg_bufnr = arg_bufnr;
  apc->data = (Object *)data;
  apc->next = NULL;
}

/// Push apc onto active_apc_list.
void nvim_apc_push_active(void *apc_raw)
{
  AutoPatCmd *apc = (AutoPatCmd *)apc_raw;
  apc->next = active_apc_list;
  active_apc_list = apc;
}

/// Pop apc from active_apc_list (if it's at the top).
void nvim_apc_pop_active(void *apc_raw)
{
  if (active_apc_list == (AutoPatCmd *)apc_raw) {
    active_apc_list = ((AutoPatCmd *)apc_raw)->next;
  }
}

/// Get apc->lastpat != NULL.
bool nvim_apc_has_match(const void *apc_raw) { return ((const AutoPatCmd *)apc_raw)->lastpat != NULL; }

// --- Search pattern / redo save/restore ---

/// Saved search + redo state for apply_autocmds_group.
typedef struct {
  bool did_save_redobuff;
  save_redo_T save_redo;
} AutocmdExecSave;

/// Save search patterns and optionally redo buffer. Returns heap-allocated save struct.
void *nvim_autocmd_save_exec(void)
{
  AutocmdExecSave *s = xmalloc(sizeof(*s));
  save_search_patterns();
  s->did_save_redobuff = false;
  if (!rs_ins_compl_active()) {
    saveRedobuff(&s->save_redo);
    s->did_save_redobuff = true;
  }
  return s;
}

/// Restore search patterns and optionally redo buffer. Frees save struct.
void nvim_autocmd_restore_exec(void *save_raw)
{
  AutocmdExecSave *s = (AutocmdExecSave *)save_raw;
  restore_search_patterns();
  if (s->did_save_redobuff) {
    restoreRedobuff(&s->save_redo);
  }
  xfree(s);
}

// --- funccal save/restore ---

typedef struct {
  funccal_entry_T entry;
} AutocmdFunccalSave;

/// Save funccal state. Returns heap-allocated save struct.
void *nvim_autocmd_save_funccal(void)
{
  AutocmdFunccalSave *s = xmalloc(sizeof(*s));
  save_funccal(&s->entry);
  return s;
}

/// Restore funccal state. Frees save struct.
void nvim_autocmd_restore_funccal(void *save_raw)
{
  (void)save_raw;
  restore_funccal();
  xfree(save_raw);
}

// --- Profiling ---

/// Call prof_child_enter if profiling is active. Returns heap-allocated proftime_T or NULL.
void *nvim_autocmd_prof_enter(void)
{
  if (do_profiling != PROF_YES) {
    return NULL;
  }
  proftime_T *wt = xmalloc(sizeof(*wt));
  prof_child_enter(wt);
  return wt;
}

/// Call prof_child_exit if profiling is active. Frees the proftime_T.
void nvim_autocmd_prof_exit(void *wt)
{
  if (wt == NULL) {
    return;
  }
  prof_child_exit((proftime_T *)wt);
  xfree(wt);
}

// --- estack ---

void nvim_autocmd_estack_push(void) { estack_push(ETYPE_AUCMD, NULL, 0); }
void nvim_autocmd_estack_pop(void) { xfree(SOURCING_NAME); estack_pop(); }

// filechangeshell_busy, autocmd_nesting, autocmd_nested, au_need_clean are now
// EXTERN globals in autocmd.h; their accessor functions have been removed.
// Rust accesses these directly via extern "C" { static mut ...; }.

// --- v:cmdbang ---

int64_t nvim_get_vim_var_cmdbang(void) { return get_vim_var_nr(VV_CMDBANG); }
void nvim_set_vim_var_cmdbang(int64_t v) { set_vim_var_nr(VV_CMDBANG, v); }

// --- eap.forceit accessor ---

bool nvim_autocmd_eap_get_forceit(const void *eap) { return ((const exarg_T *)eap)->forceit; }

// --- set_cmdarg wrapper ---
// Returns old cmdarg (saved value). If eap is NULL, restores from old_arg.

char *nvim_set_cmdarg(void *eap, char *old_arg) { return set_cmdarg((exarg_T *)eap, old_arg); }

// --- get/set pressedreturn (autocmd variant) ---

bool nvim_autocmd_get_pressedreturn(void) { return get_pressedreturn(); }
void nvim_autocmd_set_pressedreturn(bool v) { set_pressedreturn(v); }

// nvim_get_keytd, nvim_set_keytd removed: Rust accesses KeyTyped directly.
// nvim_inc_redrawing_disabled, nvim_dec_redrawing_disabled removed: Rust accesses RedrawingDisabled directly.

// --- Free pending bufs/wins ---

void nvim_autocmd_free_pending(void)
{
  while (au_pending_free_buf != NULL) {
    buf_T *b = au_pending_free_buf->b_next;
    xfree(au_pending_free_buf);
    au_pending_free_buf = b;
  }
  while (au_pending_free_win != NULL) {
    win_T *w = au_pending_free_win->w_next;
    xfree(au_pending_free_win);
    au_pending_free_win = w;
  }
}


/// Variant of check_changed that takes explicit values (for Rust after ctx has been freed).
void nvim_autocmd_check_changed_ex(void *old_curbuf_raw, bool save_changed)
{
  buf_T *old_curbuf = (buf_T *)old_curbuf_raw;
  if (curbuf == old_curbuf) {
    if (curbuf->b_changed != save_changed) {
      need_maketitle = true;
    }
    curbuf->b_changed = save_changed;
  }
}

/// The event constants for events that check b_changed in apply_autocmds_group.
bool nvim_autocmd_event_resets_changed(int event)
{
  return event == EVENT_BUFREADPOST || event == EVENT_BUFWRITEPOST
      || event == EVENT_FILEAPPENDPOST || event == EVENT_VIMLEAVE
      || event == EVENT_VIMLEAVEPRE;
}

/// event_ignored wrapper.
bool nvim_event_ignored(int event, const char *pat) { return event_ignored((event_T)event, pat); }

/// did_emsg accessor.
int nvim_autocmd_get_did_emsg(void) { return did_emsg; }
void nvim_autocmd_add_did_emsg(int v) { did_emsg += v; }

// nvim_get_autocmd_nested removed: Rust accesses autocmd_nested directly.
// nvim_get_autocmd_busy is already defined in change_ffi.c

/// Save/restore current_sctx (for apply_autocmds_group migration).
typedef struct {
  sctx_T sctx;
} AutocmdSctxSave;

void *nvim_autocmd_save_sctx(void)
{
  AutocmdSctxSave *s = xmalloc(sizeof(*s));
  s->sctx = current_sctx;
  return s;
}

void nvim_autocmd_restore_sctx(void *s_raw)
{
  AutocmdSctxSave *s = (AutocmdSctxSave *)s_raw;
  current_sctx = s->sctx;
  xfree(s);
}
