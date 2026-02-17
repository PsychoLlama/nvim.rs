// syntax_bridge.c: thin C wrappers that forward public API calls to Rust.
// These functions maintain the C-visible API surface so that other C files
// (drawscreen.c, drawline.c, edit.c, spell.c, etc.) can call them without
// change, while the actual implementation lives in Rust.

#include <stdbool.h>

#include "nvim/buffer_defs.h"
#include "nvim/pos_defs.h"
#include "nvim/profile.h"
#include "nvim/syntax.h"
#include "nvim/types_defs.h"

// Rust FFI declarations
extern int rs_syntax_present(win_T *win);
extern void rs_syntax_start(win_T *wp, int lnum);
extern int rs_syntax_check_changed(int lnum);
extern void rs_syntax_end_parsing_impl(win_T *wp, int lnum);
extern int rs_get_syntax_attr(int col, int *can_spell, int keep_state);
extern int rs_syn_get_sub_char(void);
extern int rs_syn_get_foldlevel_impl(win_T *wp, int lnum);

#include "syntax_bridge.c.generated.h"

/// Start the syntax recognition for a line.
void syntax_start(win_T *wp, linenr_T lnum)
{
  rs_syntax_start(wp, (int)lnum);
}

/// We stop parsing syntax above line "lnum".
void syntax_end_parsing(win_T *wp, linenr_T lnum)
{
  rs_syntax_end_parsing_impl(wp, (int)lnum);
}

/// @return  true if the syntax at start of lnum changed since last time.
bool syntax_check_changed(linenr_T lnum)
{
  return rs_syntax_check_changed((int)lnum) != 0;
}

/// Gets highlight attributes for next character.
int get_syntax_attr(const colnr_T col, bool *const can_spell, const bool keep_state)
{
  int cs = 0;
  int attr = rs_get_syntax_attr((int)col, can_spell ? &cs : NULL, keep_state);
  if (can_spell) {
    *can_spell = cs != 0;
  }
  return attr;
}

/// @return  true if syntax highlighting is active for window "win".
bool syntax_present(win_T *win)
{
  return rs_syntax_present(win) != 0;
}

/// Return conceal substitution character.
int syn_get_sub_char(void)
{
  return rs_syn_get_sub_char();
}

/// Function called to get folding level for line "lnum" in window "wp".
int syn_get_foldlevel(win_T *wp, linenr_T lnum)
{
  return rs_syn_get_foldlevel_impl(wp, (int)lnum);
}
