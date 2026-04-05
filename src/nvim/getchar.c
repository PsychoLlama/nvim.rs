// getchar.c: Code related to getting a character from the user or a script
// file, manipulations with redo buffer and stuff buffer.

#include <assert.h>
#include <limits.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/vim.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_getln.h"
#include "nvim/ex_getln_defs.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/input.h"
#include "nvim/insexpand.h"
#include "nvim/keycodes.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/main.h"
#include "nvim/mapping.h"
#include "nvim/mapping_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/normal_defs.h"
#include "nvim/ops.h"
#include "nvim/option_vars.h"
#include "nvim/os/fileio.h"
#include "nvim/os/fileio_defs.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"

// Rust implementation in nvim-event crate
extern int rs_multiqueue_empty(MultiQueue *mq);
extern MultiQueue *rs_loop_get_events(Loop *loop);
#define multiqueue_empty(mq) rs_multiqueue_empty(mq)
#define loop_get_events(l) rs_loop_get_events(l)

// Buffer FFI functions (buffers owned by Rust)
extern int rs_read_readbuffers(int advance);
extern void rs_start_stuff(void);

// Typebuf lifecycle (owned by Rust: rs_init/alloc/free/save/close_typebuf)
extern void rs_init_typebuf(void);
extern void rs_alloc_typebuf(void);
extern void rs_free_typebuf(void);
extern void rs_save_typebuf(void);
extern void rs_close_typebuf(void);

// Recording/gotchars operations (full functions in Rust)
extern void rs_gotchars(const uint8_t *chars, size_t len);
extern void rs_add_byte_to_showcmd(uint8_t byte);

// no_reduce_keys is now owned by Rust; getchar_common calls these
extern void rs_inc_no_reduce_keys(void);
extern void rs_dec_no_reduce_keys(void);

/// Index in scriptin (non-static: accessed by Rust via extern)
int curscript = -1;
/// Streams to read script from (non-static: accessed by Rust via extern)
FileDescriptor scriptin[NSCRIPT] = { 0 };

int typeahead_char = 0;  ///< typeahead char that's not flushed (non-static: accessed by Rust)

int KeyNoremap = 0;  ///< remapping flags (non-static: accessed by Rust)

/// Variables used by vgetorpeek() and flush_buffers()
///
/// typebuf.tb_buf[] contains all characters that are not consumed yet.
/// typebuf.tb_buf[typebuf.tb_off] is the first valid character.
/// typebuf.tb_buf[typebuf.tb_off + typebuf.tb_len - 1] is the last valid char.
/// typebuf.tb_buf[typebuf.tb_off + typebuf.tb_len] must be NUL.
/// The head of the buffer may contain the result of mappings, abbreviations
/// and @a commands.  The length of this part is typebuf.tb_maplen.
/// typebuf.tb_silent is the part where <silent> applies.
/// After the head are characters that come from the terminal.
/// typebuf.tb_no_abbr_cnt is the number of characters in typebuf.tb_buf that
/// should not be considered for abbreviations.
/// Some parts of typebuf.tb_buf may not be mapped. These parts are remembered
/// in typebuf.tb_noremap[], which is the same length as typebuf.tb_buf and
/// contains RM_NONE for the characters that are not to be remapped.
/// typebuf.tb_noremap[typebuf.tb_off] is the first valid flag.
enum {
  RM_YES    = 0,  ///< tb_noremap: remap
  RM_NONE   = 1,  ///< tb_noremap: don't remap
  RM_SCRIPT = 2,  ///< tb_noremap: remap local script mappings
  RM_ABBR   = 4,  ///< tb_noremap: don't remap, do abbrev.
};

// last_recorded_len moved to Rust (macro_recording.rs); export as #[no_mangle]
extern size_t last_recorded_len;

enum {
  KEYLEN_PART_KEY = -1,  ///< keylen value for incomplete key-code
  KEYLEN_PART_MAP = -2,  ///< keylen value for incomplete mapping
};

#include "getchar.c.generated.h"

static const char e_recursive_mapping[] = N_("E223: Recursive mapping");

// Rust replacements: rs_init_typebuf, rs_alloc_typebuf, rs_free_typebuf,
// rs_save_typebuf, rs_close_typebuf (in nvim-getchar crate)

// save_typeahead / restore_typeahead moved to Rust (typebuf.rs) as #[export_name] fns

/// Open a new script file for the ":source!" command.
///
/// @param directly  when true execute directly
void openscript(char *name, bool directly)
{
  if (curscript + 1 == NSCRIPT) {
    emsg(_(e_nesting));
    return;
  }

  // Disallow sourcing a file in the sandbox, the commands would be executed
  // later, possibly outside of the sandbox.
  if (rs_check_secure()) {
    return;
  }

  if (ignore_script) {
    // Not reading from script, also don't open one.  Warning message?
    return;
  }

  curscript++;
  // use NameBuff for expanded name
  expand_env(name, NameBuff, MAXPATHL);
  int error = file_open(&scriptin[curscript], NameBuff, kFileReadOnly, 0);
  if (error) {
    semsg(_(e_notopen_2), name, os_strerror(error));
    curscript--;
    return;
  }
  rs_save_typebuf();

  // Execute the commands from the file right now when using ":source!"
  // after ":global" or ":argdo" or in a loop.  Also when another command
  // follows.  This means the display won't be updated.  Don't do this
  // always, "make test" would fail.
  if (directly) {
    oparg_T oa;
    int save_State = State;
    int save_restart_edit = restart_edit;
    int save_finish_op = finish_op;
    int save_msg_scroll = msg_scroll;

    State = MODE_NORMAL;
    msg_scroll = false;         // no msg scrolling in Normal mode
    restart_edit = 0;           // don't go to Insert mode
    clear_oparg(&oa);
    finish_op = false;

    int oldcurscript = curscript;
    do {
      update_topline_cursor();          // update cursor position and topline
      normal_cmd(&oa, false);           // execute one command
      vpeekc();                   // check for end of file
    } while (curscript >= oldcurscript);

    State = save_State;
    msg_scroll = save_msg_scroll;
    restart_edit = save_restart_edit;
    finish_op = save_finish_op;
  }
}

// closescript, close_all_scripts, open_scriptin moved to Rust (typebuf.rs, Phase 5)
// rs_closescript is the Rust implementation; close_all_scripts and open_scriptin
// are exported with #[export_name] from Rust.

// no_reduce_keys moved to Rust (typebuf.rs); getchar_common uses rs_inc/dec_no_reduce_keys()

/// "getchar()" and "getcharstr()" functions
static void getchar_common(typval_T *argvars, typval_T *rettv, bool allow_number)
  FUNC_ATTR_NONNULL_ALL
{
  varnumber_T n = 0;
  const int called_emsg_start = called_emsg;
  bool error = false;
  bool simplify = true;
  char cursor_flag = NUL;

  if (argvars[0].v_type != VAR_UNKNOWN
      && tv_check_for_opt_dict_arg(argvars, 1) == FAIL) {
    return;
  }

  if (argvars[0].v_type != VAR_UNKNOWN && argvars[1].v_type == VAR_DICT) {
    dict_T *d = argvars[1].vval.v_dict;

    if (allow_number) {
      allow_number = tv_dict_get_bool(d, "number", true);
    } else if (tv_dict_has_key(d, "number")) {
      semsg(_(e_invarg2), "number");
    }

    simplify = tv_dict_get_bool(d, "simplify", true);

    const char *cursor_str = tv_dict_get_string(d, "cursor", false);
    if (cursor_str != NULL) {
      if (strcmp(cursor_str, "hide") != 0
          && strcmp(cursor_str, "keep") != 0
          && strcmp(cursor_str, "msg") != 0) {
        semsg(_(e_invargNval), "cursor", cursor_str);
      } else {
        cursor_flag = cursor_str[0];
      }
    }
  }

  if (called_emsg != called_emsg_start) {
    return;
  }

  if (cursor_flag == 'h') {
    ui_busy_start();
  }

  no_mapping++;
  allow_keys++;
  if (!simplify) {
    rs_inc_no_reduce_keys();
  }
  while (true) {
    if (cursor_flag == 'm' || (cursor_flag == NUL && msg_col > 0)) {
      ui_cursor_goto(msg_row, msg_col);
    }

    if (argvars[0].v_type == VAR_UNKNOWN
        || (argvars[0].v_type == VAR_NUMBER && argvars[0].vval.v_number == -1)) {
      // getchar(): blocking wait.
      // TODO(bfredl): deduplicate shared logic with state_enter ?
      if (!char_avail()) {
        // Flush screen updates before blocking.
        ui_flush();
        input_get(NULL, 0, -1, typebuf.tb_change_cnt, loop_get_events(&main_loop));
        if (!input_available() && !multiqueue_empty(loop_get_events(&main_loop))) {
          state_handle_k_event();
          continue;
        }
      }
      n = safe_vgetc();
    } else if (tv_get_number_chk(&argvars[0], &error) == 1) {
      // getchar(1): only check if char avail
      n = vpeekc_any();
    } else if (error || vpeekc_any() == NUL) {
      // illegal argument or getchar(0) and no char avail: return zero
      n = 0;
    } else {
      // getchar(0) and char avail() != NUL: get a character.
      // Note that vpeekc_any() returns K_SPECIAL for K_IGNORE.
      n = safe_vgetc();
    }

    if (n == K_IGNORE
        || n == K_MOUSEMOVE
        || n == K_VER_SCROLLBAR
        || n == K_HOR_SCROLLBAR) {
      continue;
    }
    break;
  }
  no_mapping--;
  allow_keys--;
  if (!simplify) {
    rs_dec_no_reduce_keys();
  }

  if (cursor_flag == 'h') {
    ui_busy_stop();
  }

  set_vim_var_nr(VV_MOUSE_WIN, 0);
  set_vim_var_nr(VV_MOUSE_WINID, 0);
  set_vim_var_nr(VV_MOUSE_LNUM, 0);
  set_vim_var_nr(VV_MOUSE_COL, 0);

  if (n != 0 && (!allow_number || IS_SPECIAL(n) || mod_mask != 0)) {
    char temp[10];                // modifier: 3, mbyte-char: 6, NUL: 1
    int i = 0;

    // Turn a special key into three bytes, plus modifier.
    if (mod_mask != 0) {
      temp[i++] = (char)K_SPECIAL;
      temp[i++] = (char)KS_MODIFIER;
      temp[i++] = (char)mod_mask;
    }
    if (IS_SPECIAL(n)) {
      temp[i++] = (char)K_SPECIAL;
      temp[i++] = (char)K_SECOND(n);
      temp[i++] = (char)K_THIRD(n);
    } else {
      i += utf_char2bytes((int)n, temp + i);
    }
    assert(i < 10);
    temp[i] = NUL;
    rettv->v_type = VAR_STRING;
    rettv->vval.v_string = xmemdupz(temp, (size_t)i);

    if (is_mouse_key((int)n)) {
      int row = mouse_row;
      int col = mouse_col;
      int grid = mouse_grid;
      linenr_T lnum;
      win_T *wp;

      if (row >= 0 && col >= 0) {
        int winnr = 1;
        // Find the window at the mouse coordinates and compute the
        // text position.
        win_T *const win = mouse_find_win_inner(&grid, &row, &col);
        if (win == NULL) {
          return;
        }
        mouse_comp_pos(win, &row, &col, &lnum);
        for (wp = firstwin; wp != win; wp = wp->w_next) {
          winnr++;
        }
        set_vim_var_nr(VV_MOUSE_WIN, winnr);
        set_vim_var_nr(VV_MOUSE_WINID, wp->handle);
        set_vim_var_nr(VV_MOUSE_LNUM, lnum);
        set_vim_var_nr(VV_MOUSE_COL, col + 1);
      }
    }
  } else if (!allow_number) {
    rettv->v_type = VAR_STRING;
  } else {
    rettv->vval.v_number = n;
  }
}

/// "getchar()" function
void f_getchar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { getchar_common(argvars, rettv, true); }

/// "getcharstr()" function
void f_getcharstr(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { getchar_common(argvars, rettv, false); }

/// "getcharmod()" function
void f_getcharmod(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { rettv->vval.v_number = mod_mask; }

// map_result_T, handle_mapping, vgetorpeek migrated to Rust (Phases 1+2)
int put_string_in_typebuf(int offset, int slen, uint8_t *string, int new_slen);
bool at_ins_compl_key(void);

// inchar moved to Rust (typebuf.rs, Phase 5) as #[no_mangle] pub unsafe extern "C" fn inchar

/// Handle a Lua mapping: get its LuaRef from typeahead and execute it.
///
/// @param may_repeat  save the LuaRef for redoing with "." later
/// @param discard     discard the keys instead of executing the LuaRef
///
/// @return  false if getting the LuaRef was aborted, true otherwise
bool map_execute_lua(bool may_repeat, bool discard)
{
  garray_T line_ga;
  int c1 = -1;
  bool aborted = false;

  ga_init(&line_ga, 1, 32);

  no_mapping++;

  got_int = false;
  while (c1 != NUL && !aborted) {
    ga_grow(&line_ga, 32);
    // Get one character at a time.
    c1 = vgetorpeek(true);
    if (got_int) {
      aborted = true;
    } else if (c1 == '\r' || c1 == '\n') {
      c1 = NUL;  // end the line
    } else {
      ga_append(&line_ga, (uint8_t)c1);
    }
  }

  no_mapping--;

  if (aborted || discard) {
    ga_clear(&line_ga);
    return !aborted;
  }

  LuaRef ref = (LuaRef)atoi(line_ga.ga_data);
  if (may_repeat) {
    repeat_luaref = ref;
  }

  Error err = ERROR_INIT;
  Array args = ARRAY_DICT_INIT;
  nlua_call_ref(ref, NULL, args, kRetNilBool, NULL, &err);
  if (ERROR_SET(&err)) {
    semsg_multiline("emsg", "E5108: %s", err.msg);
    api_clear_error(&err);
  }

  ga_clear(&line_ga);
  return true;
}

/// Gets a paste stored by paste_store() from typeahead and repeats it.
void paste_repeat(int count)
{
  garray_T ga = GA_INIT(1, 32);
  bool aborted = false;

  no_mapping++;

  got_int = false;
  while (!aborted) {
    ga_grow(&ga, 32);
    uint8_t c1 = (uint8_t)vgetorpeek(true);
    if (c1 == K_SPECIAL) {
      c1 = (uint8_t)vgetorpeek(true);
      uint8_t c2 = (uint8_t)vgetorpeek(true);
      int c = TO_SPECIAL(c1, c2);
      if (c == K_PASTE_END) {
        break;
      } else if (c == K_ZERO) {
        ga_append(&ga, NUL);
      } else if (c == K_SPECIAL) {
        ga_append(&ga, K_SPECIAL);
      } else {
        ga_append(&ga, K_SPECIAL);
        ga_append(&ga, c1);
        ga_append(&ga, c2);
      }
    } else {
      ga_append(&ga, c1);
    }
    aborted = got_int;
  }

  no_mapping--;

  String str = cbuf_as_string(ga.ga_data, (size_t)ga.ga_len);
  Arena arena = ARENA_EMPTY;
  Error err = ERROR_INIT;
  for (int i = 0; !aborted && i < count; i++) {
    nvim_paste(LUA_INTERNAL_CALL, str, false, -1, &arena, &err);
    aborted = ERROR_SET(&err);
  }
  api_clear_error(&err);
  arena_mem_free(arena_finish(&arena));
  ga_clear(&ga);
}

int nvim_get_typebuf_change_cnt(void) { return typebuf.tb_change_cnt; }
int nvim_get_typebuf_was_filled(void) { return typebuf_was_filled ? 1 : 0; }
int nvim_get_typebuf_maplen(void) { return typebuf.tb_maplen; }
int nvim_get_typebuf_len(void) { return typebuf.tb_len; }
// nvim_get_curscript, nvim_get_keynoremap deleted: curscript, KeyNoremap now non-static
// nvim_get_rm_none, nvim_get_rm_script, nvim_get_maxmaplen deleted: Rust uses constants directly
uint8_t *nvim_get_typebuf_buf(void) { return typebuf.tb_buf; }
uint8_t *nvim_get_typebuf_noremap(void) { return typebuf.tb_noremap; }
int nvim_get_typebuf_buflen(void) { return typebuf.tb_buflen; }
int nvim_get_typebuf_off(void) { return typebuf.tb_off; }
void nvim_set_typebuf_off(int val) { typebuf.tb_off = val; }
void nvim_set_typebuf_len(int val) { typebuf.tb_len = val; }
void nvim_set_typebuf_maplen(int val) { typebuf.tb_maplen = val; }
void nvim_set_typebuf_silent(int val) { typebuf.tb_silent = val; }
void nvim_set_typebuf_no_abbr_cnt(int val) { typebuf.tb_no_abbr_cnt = val; }
void nvim_set_typebuf_change_cnt(int val) { typebuf.tb_change_cnt = val; }
void nvim_set_typebuf_was_filled(int val) { typebuf_was_filled = val != 0; }
// nvim_get/set_old_char, nvim_get/set_old_mod_mask deleted: moved to Rust (input.rs)
int nvim_get_typebuf_silent(void) { return typebuf.tb_silent; }
int nvim_get_typebuf_no_abbr_cnt(void) { return typebuf.tb_no_abbr_cnt; }
void nvim_init_typebuf(void) { rs_init_typebuf(); }

// nvim_get/set/add_last_recorded_len deleted: moved to Rust (macro_recording.rs)
void nvim_set_keynoremap(int val) { KeyNoremap = val; }
int nvim_get_no_mapping(void) { return no_mapping; }
void nvim_set_no_mapping(int val) { no_mapping = val; }
int nvim_get_allow_keys(void) { return allow_keys; }
void nvim_set_allow_keys(int val) { allow_keys = val; }
int nvim_get_mapped_ctrl_c(void) { return mapped_ctrl_c; }
void nvim_set_mapped_ctrl_c(int val) { mapped_ctrl_c = val; }
int nvim_get_keytyped(void) { return KeyTyped ? 1 : 0; }
void nvim_set_keytyped(int val) { KeyTyped = val != 0; }
int nvim_get_keystuffed(void) { return KeyStuffed; }
void nvim_set_keystuffed(int val) { KeyStuffed = val; }
int nvim_get_vgetc_busy(void) { return vgetc_busy; }
void nvim_inc_vgetc_busy(void) { vgetc_busy++; }
void nvim_dec_vgetc_busy(void)
{
  if (vgetc_busy > 0) {
    vgetc_busy--;
  }
}
int nvim_get_ex_normal_busy(void) { return ex_normal_busy; }
int nvim_get_maptick(void) { return maptick; }
void nvim_inc_maptick(void) { maptick++; }
int nvim_get_mod_mask(void) { return mod_mask; }
void nvim_set_mod_mask(int val) { mod_mask = val; }
int nvim_get_cmd_silent(void) { return cmd_silent ? 1 : 0; }
void nvim_set_cmd_silent(int val) { cmd_silent = val != 0; }
int nvim_get_mouse_grid(void) { return mouse_grid; }
void nvim_set_mouse_grid(int val) { mouse_grid = val; }
int nvim_get_mouse_row(void) { return mouse_row; }
void nvim_set_mouse_row(int val) { mouse_row = val; }
int nvim_get_mouse_col(void) { return mouse_col; }
void nvim_set_mouse_col(int val) { mouse_col = val; }
int nvim_char_avail(void) { return char_avail() ? 1 : 0; }
void nvim_set_reg_executing(int val) { reg_executing = val; }
int nvim_get_pending_end_reg_executing(void) { return pending_end_reg_executing ? 1 : 0; }
void nvim_set_pending_end_reg_executing(int val) { pending_end_reg_executing = val != 0; }
// nvim_mb_byte2len_check deleted: no Rust callers
void nvim_state_no_longer_safe(void) { state_no_longer_safe("rs_ins_typebuf()"); }
int nvim_get_key_stuffed(void) { return KeyStuffed ? 1 : 0; }
// nvim_get/set_typeahead_char deleted: typeahead_char now non-static
// nvim_get/set_old_keystuffed deleted: moved to Rust (input.rs)

void nvim_set_visual_from_cursor(void)
{
  VIsual = curwin->w_cursor;
  VIsual_active = true;
  VIsual_select = false;
  VIsual_reselect = true;
  redo_VIsual_busy = true;
}

// nvim_map_execute_lua_discard, nvim_paste_repeat_discard deleted:
// Rust calls map_execute_lua/paste_repeat directly
