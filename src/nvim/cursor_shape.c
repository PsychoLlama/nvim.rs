#include <stdbool.h>
#include <stdint.h>
#include <string.h>

#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/charset.h"
#include "nvim/cursor_shape.h"
#include "nvim/ex_getln.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_group.h"
#include "nvim/log.h"
#include "nvim/macros_defs.h"
#include "nvim/option_vars.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/ui.h"

#include "cursor_shape.c.generated.h"

extern int rs_cursor_is_block_during_visual(int exclusive);
extern int rs_cursor_mode_uses_syn_id(int syn_id);
extern int rs_cursor_get_mode_idx(void);
extern int rs_cursor_mode_str2int(const char *mode);
extern void rs_clear_shape_table(void);
extern const char *rs_parse_shape_opt(int what);

/// Handling of cursor and mouse pointer shapes in various modes.
cursorentry_T shape_table[SHAPE_IDX_COUNT] = {
  // Values are set by 'guicursor' and 'mouseshape'.
  // Adjust the SHAPE_IDX_ defines when changing this!
  { "normal", 0, 0, 0, 700, 400, 250, 0, 0, "n", SHAPE_CURSOR + SHAPE_MOUSE },
  { "visual", 0, 0, 0, 700, 400, 250, 0, 0, "v", SHAPE_CURSOR + SHAPE_MOUSE },
  { "insert", 0, 0, 0, 700, 400, 250, 0, 0, "i", SHAPE_CURSOR + SHAPE_MOUSE },
  { "replace", 0, 0, 0, 700, 400, 250, 0, 0, "r", SHAPE_CURSOR + SHAPE_MOUSE },
  { "cmdline_normal", 0, 0, 0, 700, 400, 250, 0, 0, "c", SHAPE_CURSOR + SHAPE_MOUSE },
  { "cmdline_insert", 0, 0, 0, 700, 400, 250, 0, 0, "ci", SHAPE_CURSOR + SHAPE_MOUSE },
  { "cmdline_replace", 0, 0, 0, 700, 400, 250, 0, 0, "cr", SHAPE_CURSOR + SHAPE_MOUSE },
  { "operator", 0, 0, 0, 700, 400, 250, 0, 0, "o", SHAPE_CURSOR + SHAPE_MOUSE },
  { "visual_select", 0, 0, 0, 700, 400, 250, 0, 0, "ve", SHAPE_CURSOR + SHAPE_MOUSE },
  { "cmdline_hover", 0, 0, 0,   0,   0,   0, 0, 0, "e", SHAPE_MOUSE },
  { "statusline_hover", 0, 0, 0,   0,   0,   0, 0, 0, "s", SHAPE_MOUSE },
  { "statusline_drag", 0, 0, 0,   0,   0,   0, 0, 0, "sd", SHAPE_MOUSE },
  { "vsep_hover", 0, 0, 0,   0,   0,   0, 0, 0, "vs", SHAPE_MOUSE },
  { "vsep_drag", 0, 0, 0,   0,   0,   0, 0, 0, "vd", SHAPE_MOUSE },
  { "more", 0, 0, 0,   0,   0,   0, 0, 0, "m", SHAPE_MOUSE },
  { "more_lastline", 0, 0, 0,   0,   0,   0, 0, 0, "ml", SHAPE_MOUSE },
  { "showmatch", 0, 0, 0, 100, 100, 100, 0, 0, "sm", SHAPE_CURSOR },
  { "terminal", 0, 0, 0, 0, 0, 0, 0, 0, "t", SHAPE_CURSOR },
};

/// Converts cursor_shapes into an Array of Dictionaries
/// @param arena initialized arena where memory will be allocated
///
/// @return Array of the form {[ "cursor_shape": ... ], ...}
Array mode_style_array(Arena *arena)
{
  Array all = arena_array(arena, SHAPE_IDX_COUNT);

  for (int i = 0; i < SHAPE_IDX_COUNT; i++) {
    cursorentry_T *cur = &shape_table[i];
    Dict dic = arena_dict(arena, 3 + ((cur->used_for & SHAPE_CURSOR) ? 9 : 0));
    PUT_C(dic, "name", CSTR_AS_OBJ(cur->full_name));
    PUT_C(dic, "short_name", CSTR_AS_OBJ(cur->name));
    if (cur->used_for & SHAPE_MOUSE) {
      PUT_C(dic, "mouse_shape", INTEGER_OBJ(cur->mshape));
    }
    if (cur->used_for & SHAPE_CURSOR) {
      String shape_str;
      switch (cur->shape) {
      case SHAPE_BLOCK:
        shape_str = cstr_as_string("block"); break;
      case SHAPE_VER:
        shape_str = cstr_as_string("vertical"); break;
      case SHAPE_HOR:
        shape_str = cstr_as_string("horizontal"); break;
      default:
        shape_str = cstr_as_string("unknown");
      }
      PUT_C(dic, "cursor_shape", STRING_OBJ(shape_str));
      PUT_C(dic, "cell_percentage", INTEGER_OBJ(cur->percentage));
      PUT_C(dic, "blinkwait", INTEGER_OBJ(cur->blinkwait));
      PUT_C(dic, "blinkon", INTEGER_OBJ(cur->blinkon));
      PUT_C(dic, "blinkoff", INTEGER_OBJ(cur->blinkoff));
      PUT_C(dic, "hl_id", INTEGER_OBJ(cur->id));
      PUT_C(dic, "id_lm", INTEGER_OBJ(cur->id_lm));
      PUT_C(dic, "attr_id", INTEGER_OBJ(cur->id ? syn_id2attr(cur->id) : 0));
      PUT_C(dic, "attr_id_lm", INTEGER_OBJ(cur->id_lm ? syn_id2attr(cur->id_lm) : 0));
    }

    ADD_C(all, DICT_OBJ(dic));
  }

  return all;
}

/// Parses the 'guicursor' option.
///
/// Clears `shape_table` if 'guicursor' is empty.
///
/// @param what SHAPE_CURSOR or SHAPE_MOUSE ('mouseshape')
///
/// @returns error message for an illegal option, NULL otherwise.
const char *parse_shape_opt(int what)
{
  return rs_parse_shape_opt(what);
}

/// Returns true if the cursor is non-blinking "block" shape during
/// visual selection.
///
/// @param exclusive If 'selection' option is "exclusive".
bool cursor_is_block_during_visual(bool exclusive)
  FUNC_ATTR_PURE
{
  return rs_cursor_is_block_during_visual(exclusive ? 1 : 0) != 0;
}

/// Map cursor mode from string to integer
///
/// @param mode Fullname of the mode whose id we are looking for
/// @return -1 in case of failure, else the matching SHAPE_ID* integer
int cursor_mode_str2int(const char *mode)
{
  int result = rs_cursor_mode_str2int(mode);
  if (result < 0) {
    WLOG("Unknown mode %s", mode);
  }
  return result;
}

/// Check if a syntax id is used as a cursor style.
bool cursor_mode_uses_syn_id(int syn_id)
  FUNC_ATTR_PURE
{
  return rs_cursor_mode_uses_syn_id(syn_id) != 0;
}

/// Return the index into shape_table[] for the current mode.
int cursor_get_mode_idx(void)
  FUNC_ATTR_PURE
{
  return rs_cursor_get_mode_idx();
}

/// Clears all entries in shape_table to block, blinkon0, and default color.
static void clear_shape_table(void)
{
  rs_clear_shape_table();
}

// Rust FFI accessor functions for shape_table

int nvim_get_shape_table_shape(int idx)
{
  return (int)shape_table[idx].shape;
}

int nvim_get_shape_table_blinkon(int idx)
{
  return shape_table[idx].blinkon;
}

int nvim_get_shape_table_id(int idx)
{
  return shape_table[idx].id;
}

int nvim_get_shape_table_id_lm(int idx)
{
  return shape_table[idx].id_lm;
}

int nvim_is_guicursor_empty(void)
{
  return *p_guicursor == NUL ? 1 : 0;
}

// Additional accessors for cursor_get_mode_idx

int nvim_get_state(void)
{
  return State;
}

int nvim_get_finish_op(void)
{
  return finish_op ? 1 : 0;
}

int nvim_get_visual_active(void)
{
  return VIsual_active ? 1 : 0;
}

int nvim_get_visual_select(void)
{
  return VIsual_select ? 1 : 0;
}

char nvim_get_p_sel_first(void)
{
  return *p_sel;
}

int nvim_get_restart_edit(void)
{
  return restart_edit;
}

void nvim_set_restart_edit(int val)
{
  restart_edit = val;
}

const char *nvim_get_shape_table_name(int idx)
{
  return shape_table[idx].full_name;
}

const char *nvim_get_shape_table_short_name(int idx)
{
  return shape_table[idx].name;
}

int nvim_get_shape_table_percentage(int idx)
{
  return shape_table[idx].percentage;
}

int nvim_get_shape_table_blinkwait(int idx)
{
  return shape_table[idx].blinkwait;
}

int nvim_get_shape_table_blinkoff(int idx)
{
  return shape_table[idx].blinkoff;
}

int nvim_get_shape_table_used_for(int idx)
{
  return shape_table[idx].used_for;
}

// Setter accessors for Rust FFI

void nvim_set_shape_table_shape(int idx, int shape)
{
  shape_table[idx].shape = (CursorShape)shape;
}

void nvim_set_shape_table_percentage(int idx, int pct)
{
  shape_table[idx].percentage = pct;
}

void nvim_set_shape_table_blinkwait(int idx, int val)
{
  shape_table[idx].blinkwait = val;
}

void nvim_set_shape_table_blinkon(int idx, int val)
{
  shape_table[idx].blinkon = val;
}

void nvim_set_shape_table_blinkoff(int idx, int val)
{
  shape_table[idx].blinkoff = val;
}

void nvim_set_shape_table_id(int idx, int id)
{
  shape_table[idx].id = id;
}

void nvim_set_shape_table_id_lm(int idx, int id)
{
  shape_table[idx].id_lm = id;
}

// Additional accessors for parse_shape_opt

const char *nvim_get_p_guicursor(void)
{
  return p_guicursor;
}

int nvim_syn_check_group(const char *name, size_t len)
{
  return syn_check_group(name, len);
}

void nvim_ui_mode_info_set(void)
{
  ui_mode_info_set();
}
