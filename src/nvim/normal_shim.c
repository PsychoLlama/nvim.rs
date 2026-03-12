//
// normal.c:    Contains the main routine for processing characters in command
//              mode.  Communicates closely with the code in ops.c to handle
//              the operators.
//

#include <assert.h>
#include <ctype.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cmdhist.h"
#include "nvim/cursor.h"
#include "nvim/decoration.h"
#include "nvim/diff.h"
#include "nvim/digraph.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds2.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_getln.h"
#include "nvim/file_search.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/help.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/mapping.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/math.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/os/time.h"
#include "nvim/plines.h"
#include "nvim/profile.h"
#include "nvim/quickfix.h"
#include "nvim/register.h"
#include "nvim/search.h"
#include "nvim/spell.h"
#include "nvim/spell_defs.h"
#include "nvim/spellfile.h"
#include "nvim/spellsuggest.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/statusline.h"
#include "nvim/strings.h"
#include "nvim/syntax.h"
#include "nvim/tag.h"
#include "nvim/textformat.h"
#include "nvim/textobject.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

typedef struct {
  VimState state;
  bool command_finished;
  bool ctrl_w;
  bool need_flushbuf;
  bool set_prevcount;
  bool previous_got_int;             // `got_int` was true
  bool cmdwin;                       // command-line window normal mode
  bool noexmode;                     // true if the normal mode was pushed from
                                     // ex mode (:global or :visual for example)
  bool toplevel;                     // top-level normal mode
  oparg_T oa;                        // operator arguments
  cmdarg_T ca;                       // command arguments
  int mapped_len;
  int old_mapped_len;
  int idx;
  int c;
  int old_col;
  pos_T old_pos;
} NormalState;

#include "normal_shim.c.generated.h"

static const char e_changelist_is_empty[] = N_("E664: Changelist is empty");
static const char e_cmdline_window_already_open[]
  = N_("E1292: Command-line window is already open");

static inline void normal_state_init(NormalState *s) { memset(s, 0, sizeof(NormalState)); s->state.check = normal_check; s->state.execute = normal_execute; }

// nv_*(): functions called to handle Normal and Visual mode commands.
// n_*(): functions called to handle Normal mode commands.
// v_*(): functions called to handle Visual mode commands.

static const char *e_noident = N_("E349: No identifier under cursor");

/// Function to be called for a Normal or Visual mode command.
/// The argument is a cmdarg_T.
typedef void (*nv_func_T)(cmdarg_T *cap);

// Values for cmd_flags.
#define NV_NCH      0x01          // may need to get a second char
#define NV_NCH_NOP  (0x02|NV_NCH)  // get second char when no operator pending
#define NV_NCH_ALW  (0x04|NV_NCH)  // always get a second char
#define NV_LANG     0x08        // second char needs language adjustment

#define NV_SS       0x10        // may start selection
#define NV_SSS      0x20        // may start selection with shift modifier
#define NV_STS      0x40        // may stop selection without shift modif.
#define NV_RL       0x80        // 'rightleft' modifies command
#define NV_KEEPREG  0x100       // don't clear regname
#define NV_NCW      0x200       // not allowed in command-line window

// Generally speaking, every Normal mode command should either clear any
// pending operator (with *clearop*()), or set the motion type variable
// oap->motion_type.
//
// When a cursor motion command is made, it is marked as being a character or
// line oriented motion.  Then, if an operator is in effect, the operation
// becomes character or line oriented accordingly.

// Rust command handlers (forward declarations needed by dispatch table)
extern int rs_magic_isset(void);
extern void rs_nv_ignore(cmdarg_T *cap);
extern void rs_nv_nop(cmdarg_T *cap);
extern void rs_nv_error(cmdarg_T *cap);
extern void rs_nv_help(cmdarg_T *cap);
extern void rs_nv_suspend(cmdarg_T *cap);
extern void rs_nv_page(cmdarg_T *cap);
extern void rs_nv_halfpage(cmdarg_T *cap);
extern void rs_nv_ctrlg(cmdarg_T *cap);
extern void rs_nv_scroll_line(cmdarg_T *cap);
extern void rs_nv_kundo(cmdarg_T *cap);
extern void rs_nv_goto(cmdarg_T *cap);
extern void rs_nv_beginline(cmdarg_T *cap);
extern void rs_nv_dollar(cmdarg_T *cap);
extern void rs_nv_end(cmdarg_T *cap);
extern void rs_nv_home(cmdarg_T *cap);
extern void rs_nv_pipe(cmdarg_T *cap);
extern void rs_nv_wordcmd(cmdarg_T *cap);
extern void rs_nv_bck_word(cmdarg_T *cap);
extern void rs_nv_findpar(cmdarg_T *cap);
extern void rs_nv_brace(cmdarg_T *cap);
extern void rs_nv_csearch(cmdarg_T *cap);
extern void rs_nv_mark(cmdarg_T *cap);
extern void rs_nv_gomark(cmdarg_T *cap);
extern void rs_nv_pcmark(cmdarg_T *cap);
extern void rs_nv_regname(cmdarg_T *cap);
extern void rs_nv_put(cmdarg_T *cap);
extern void rs_nv_visual(cmdarg_T *cap);
extern void rs_nv_window(cmdarg_T *cap);
extern void rs_nv_clear(cmdarg_T *cap);
extern void rs_nv_ctrlo(cmdarg_T *cap);
extern void rs_nv_hat(cmdarg_T *cap);
extern void rs_nv_Zet(cmdarg_T *cap);
extern void rs_nv_esc(cmdarg_T *cap);
extern void rs_nv_edit(cmdarg_T *cap);
extern void rs_nv_search(cmdarg_T *cap);
extern void rs_nv_next(cmdarg_T *cap);
extern void rs_nv_ident(cmdarg_T *cap);
extern void rs_nv_operator(cmdarg_T *cap);
extern void rs_nv_optrans(cmdarg_T *cap);
extern void rs_nv_tilde(cmdarg_T *cap);
extern void rs_nv_subst(cmdarg_T *cap);
extern void rs_nv_select(cmdarg_T *cap);
extern void rs_nv_brackets(cmdarg_T *cap);
extern void rs_nv_undo(cmdarg_T *cap);
extern void rs_nv_Undo(cmdarg_T *cap);
extern void rs_nv_dot(cmdarg_T *cap);
extern void rs_nv_redo_or_register(cmdarg_T *cap);
extern void rs_nv_replace(cmdarg_T *cap);
extern void rs_nv_Replace(cmdarg_T *cap);
extern void rs_nv_zet(cmdarg_T *cap);
extern void rs_nv_scroll(cmdarg_T *cap);
extern void rs_nv_right(cmdarg_T *cap);
extern void rs_nv_left(cmdarg_T *cap);
extern void rs_nv_up(cmdarg_T *cap);
extern void rs_nv_down(cmdarg_T *cap);
extern void rs_nv_g_cmd(cmdarg_T *cap);
extern void rs_nv_at(cmdarg_T *cap);
extern void rs_nv_join(cmdarg_T *cap);
extern void rs_nv_open(cmdarg_T *cap);
extern void rs_nv_abbrev(cmdarg_T *cap);
extern void rs_nv_lineop(cmdarg_T *cap);
extern void rs_nv_normal(cmdarg_T *cap);
extern void rs_nv_percent(cmdarg_T *cap);
extern void rs_nv_tagpop(cmdarg_T *cap);
extern void rs_nv_regreplay(cmdarg_T *cap);
extern void rs_nv_ctrlh(cmdarg_T *cap);
extern void rs_nv_object(cmdarg_T *cap);
extern void rs_nv_vreplace(cmdarg_T *cap);
extern void rs_nv_g_underscore_cmd(cmdarg_T *cap);
extern void rs_nv_gi_cmd(cmdarg_T *cap);
extern void rs_nv_gv_cmd(cmdarg_T *cap);
extern void rs_n_swapchar(cmdarg_T *cap);
extern void rs_nv_addsub(cmdarg_T *cap);
extern void rs_nv_colon(cmdarg_T *cap);
extern void rs_nv_record(cmdarg_T *cap);
extern void rs_nv_paste(cmdarg_T *cap);
extern void rs_nv_event(cmdarg_T *cap);

/// This table contains one entry for every Normal or Visual mode command.
/// The order doesn't matter, init_normal_cmds() will create a sorted index.
/// It is faster when all keys from zero to '~' are present.
static const struct nv_cmd {
  int cmd_char;                 ///< (first) command character
  nv_func_T cmd_func;           ///< function for this command
  uint16_t cmd_flags;           ///< NV_ flags
  int16_t cmd_arg;              ///< value for ca.arg
} nv_cmds[] = {
  { NUL,       rs_nv_error,    0,                      0 },
  { Ctrl_A,    rs_nv_addsub,   0,                      0 },
  { Ctrl_B,    rs_nv_page,     NV_STS,                 BACKWARD },
  { Ctrl_C,    rs_nv_esc,      0,                      true },
  { Ctrl_D,    rs_nv_halfpage, 0,                      0 },
  { Ctrl_E,    rs_nv_scroll_line, 0,                   true },
  { Ctrl_F,    rs_nv_page,     NV_STS,                 FORWARD },
  { Ctrl_G,    rs_nv_ctrlg,    0,                      0 },
  { Ctrl_H,    rs_nv_ctrlh,    0,                      0 },
  { Ctrl_I,    rs_nv_pcmark,   0,                      0 },
  { NL,        rs_nv_down,     0,                      false },
  { Ctrl_K,    rs_nv_error,    0,                      0 },
  { Ctrl_L,    rs_nv_clear,    0,                      0 },
  { CAR,       rs_nv_down,     0,                      true },
  { Ctrl_N,    rs_nv_down,     NV_STS,                 false },
  { Ctrl_O,    rs_nv_ctrlo,    0,                      0 },
  { Ctrl_P,    rs_nv_up,       NV_STS,                 false },
  { Ctrl_Q,    rs_nv_visual,   0,                      false },
  { Ctrl_R,    rs_nv_redo_or_register, 0,              0 },
  { Ctrl_S,    rs_nv_ignore,   0,                      0 },
  { Ctrl_T,    rs_nv_tagpop,   NV_NCW,                 0 },
  { Ctrl_U,    rs_nv_halfpage, 0,                      0 },
  { Ctrl_V,    rs_nv_visual,   0,                      false },
  { 'V',       rs_nv_visual,   0,                      false },
  { 'v',       rs_nv_visual,   0,                      false },
  { Ctrl_W,    rs_nv_window,   0,                      0 },
  { Ctrl_X,    rs_nv_addsub,   0,                      0 },
  { Ctrl_Y,    rs_nv_scroll_line, 0,                   false },
  { Ctrl_Z,    rs_nv_suspend,  0,                      0 },
  { ESC,       rs_nv_esc,      0,                      false },
  { Ctrl_BSL,  rs_nv_normal,   NV_NCH_ALW,             0 },
  { Ctrl_RSB,  rs_nv_ident,    NV_NCW,                 0 },
  { Ctrl_HAT,  rs_nv_hat,      NV_NCW,                 0 },
  { Ctrl__,    rs_nv_error,    0,                      0 },
  { ' ',       rs_nv_right,    0,                      0 },
  { '!',       rs_nv_operator, 0,                      0 },
  { '"',       rs_nv_regname,  NV_NCH_NOP|NV_KEEPREG,  0 },
  { '#',       rs_nv_ident,    0,                      0 },
  { '$',       rs_nv_dollar,   0,                      0 },
  { '%',       rs_nv_percent,  0,                      0 },
  { '&',       rs_nv_optrans,  0,                      0 },
  { '\'',      rs_nv_gomark,   NV_NCH_ALW,             true },
  { '(',       rs_nv_brace,    0,                      BACKWARD },
  { ')',       rs_nv_brace,    0,                      FORWARD },
  { '*',       rs_nv_ident,    0,                      0 },
  { '+',       rs_nv_down,     0,                      true },
  { ',',       rs_nv_csearch,  0,                      true },
  { '-',       rs_nv_up,       0,                      true },
  { '.',       rs_nv_dot,      NV_KEEPREG,             0 },
  { '/',       rs_nv_search,   0,                      false },
  { '0',       rs_nv_beginline, 0,                     0 },
  { '1',       rs_nv_ignore,   0,                      0 },
  { '2',       rs_nv_ignore,   0,                      0 },
  { '3',       rs_nv_ignore,   0,                      0 },
  { '4',       rs_nv_ignore,   0,                      0 },
  { '5',       rs_nv_ignore,   0,                      0 },
  { '6',       rs_nv_ignore,   0,                      0 },
  { '7',       rs_nv_ignore,   0,                      0 },
  { '8',       rs_nv_ignore,   0,                      0 },
  { '9',       rs_nv_ignore,   0,                      0 },
  { ':',       rs_nv_colon,    0,                      0 },
  { ';',       rs_nv_csearch,  0,                      false },
  { '<',       rs_nv_operator, NV_RL,                  0 },
  { '=',       rs_nv_operator, 0,                      0 },
  { '>',       rs_nv_operator, NV_RL,                  0 },
  { '?',       rs_nv_search,   0,                      false },
  { '@',       rs_nv_at,       NV_NCH_NOP,             false },
  { 'A',       rs_nv_edit,     0,                      0 },
  { 'B',       rs_nv_bck_word, 0,                      1 },
  { 'C',       rs_nv_abbrev,   NV_KEEPREG,             0 },
  { 'D',       rs_nv_abbrev,   NV_KEEPREG,             0 },
  { 'E',       rs_nv_wordcmd,  0,                      true },
  { 'F',       rs_nv_csearch,  NV_NCH_ALW|NV_LANG,     BACKWARD },
  { 'G',       rs_nv_goto,     0,                      true },
  { 'H',       rs_nv_scroll,   0,                      0 },
  { 'I',       rs_nv_edit,     0,                      0 },
  { 'J',       rs_nv_join,     0,                      0 },
  { 'K',       rs_nv_ident,    0,                      0 },
  { 'L',       rs_nv_scroll,   0,                      0 },
  { 'M',       rs_nv_scroll,   0,                      0 },
  { 'N',       rs_nv_next,     0,                      SEARCH_REV },
  { 'O',       rs_nv_open,     0,                      0 },
  { 'P',       rs_nv_put,      0,                      0 },
  { 'Q',       rs_nv_regreplay, 0,                     0 },
  { 'R',       rs_nv_Replace,  0,                      false },
  { 'S',       rs_nv_subst,    NV_KEEPREG,             0 },
  { 'T',       rs_nv_csearch,  NV_NCH_ALW|NV_LANG,     BACKWARD },
  { 'U',       rs_nv_Undo,     0,                      0 },
  { 'W',       rs_nv_wordcmd,  0,                      true },
  { 'X',       rs_nv_abbrev,   NV_KEEPREG,             0 },
  { 'Y',       rs_nv_abbrev,   NV_KEEPREG,             0 },
  { 'Z',       rs_nv_Zet,      NV_NCH_NOP|NV_NCW,      0 },
  { '[',       rs_nv_brackets, NV_NCH_ALW,             BACKWARD },
  { '\\',      rs_nv_error,    0,                      0 },
  { ']',       rs_nv_brackets, NV_NCH_ALW,             FORWARD },
  { '^',       rs_nv_beginline, 0,                     BL_WHITE | BL_FIX },
  { '_',       rs_nv_lineop,   0,                      0 },
  { '`',       rs_nv_gomark,   NV_NCH_ALW,             false },
  { 'a',       rs_nv_edit,     NV_NCH,                 0 },
  { 'b',       rs_nv_bck_word, 0,                      0 },
  { 'c',       rs_nv_operator, 0,                      0 },
  { 'd',       rs_nv_operator, 0,                      0 },
  { 'e',       rs_nv_wordcmd,  0,                      false },
  { 'f',       rs_nv_csearch,  NV_NCH_ALW|NV_LANG,     FORWARD },
  { 'g',       rs_nv_g_cmd,    NV_NCH_ALW,             false },
  { 'h',       rs_nv_left,     NV_RL,                  0 },
  { 'i',       rs_nv_edit,     NV_NCH,                 0 },
  { 'j',       rs_nv_down,     0,                      false },
  { 'k',       rs_nv_up,       0,                      false },
  { 'l',       rs_nv_right,    NV_RL,                  0 },
  { 'm',       rs_nv_mark,     NV_NCH_NOP,             0 },
  { 'n',       rs_nv_next,     0,                      0 },
  { 'o',       rs_nv_open,     0,                      0 },
  { 'p',       rs_nv_put,      0,                      0 },
  { 'q',       rs_nv_record,   NV_NCH,                 0 },
  { 'r',       rs_nv_replace,  NV_NCH_NOP|NV_LANG,     0 },
  { 's',       rs_nv_subst,    NV_KEEPREG,             0 },
  { 't',       rs_nv_csearch,  NV_NCH_ALW|NV_LANG,     FORWARD },
  { 'u',       rs_nv_undo,     0,                      0 },
  { 'w',       rs_nv_wordcmd,  0,                      false },
  { 'x',       rs_nv_abbrev,   NV_KEEPREG,             0 },
  { 'y',       rs_nv_operator, 0,                      0 },
  { 'z',       rs_nv_zet,      NV_NCH_ALW,             0 },
  { '{',       rs_nv_findpar,  0,                      BACKWARD },
  { '|',       rs_nv_pipe,     0,                      0 },
  { '}',       rs_nv_findpar,  0,                      FORWARD },
  { '~',       rs_nv_tilde,    0,                      0 },

  // pound sign
  { POUND,     rs_nv_ident,    0,                      0 },
  { K_MOUSEUP, nv_mousescroll, 0,                      MSCR_UP },
  { K_MOUSEDOWN, nv_mousescroll, 0,                    MSCR_DOWN },
  { K_MOUSELEFT, nv_mousescroll, 0,                    MSCR_LEFT },
  { K_MOUSERIGHT, nv_mousescroll, 0,                   MSCR_RIGHT },
  { K_LEFTMOUSE, nv_mouse,     0,                      0 },
  { K_LEFTMOUSE_NM, nv_mouse,  0,                      0 },
  { K_LEFTDRAG, nv_mouse,      0,                      0 },
  { K_LEFTRELEASE, nv_mouse,   0,                      0 },
  { K_LEFTRELEASE_NM, nv_mouse, 0,                     0 },
  { K_MOUSEMOVE, nv_mouse,     0,                      0 },
  { K_MIDDLEMOUSE, nv_mouse,   0,                      0 },
  { K_MIDDLEDRAG, nv_mouse,    0,                      0 },
  { K_MIDDLERELEASE, nv_mouse, 0,                      0 },
  { K_RIGHTMOUSE, nv_mouse,    0,                      0 },
  { K_RIGHTDRAG, nv_mouse,     0,                      0 },
  { K_RIGHTRELEASE, nv_mouse,  0,                      0 },
  { K_X1MOUSE, nv_mouse,       0,                      0 },
  { K_X1DRAG, nv_mouse,        0,                      0 },
  { K_X1RELEASE, nv_mouse,     0,                      0 },
  { K_X2MOUSE, nv_mouse,       0,                      0 },
  { K_X2DRAG, nv_mouse,        0,                      0 },
  { K_X2RELEASE, nv_mouse,     0,                      0 },
  { K_IGNORE,  rs_nv_ignore,   NV_KEEPREG,             0 },
  { K_NOP,     rs_nv_nop,      0,                      0 },
  { K_INS,     rs_nv_edit,     0,                      0 },
  { K_KINS,    rs_nv_edit,     0,                      0 },
  { K_BS,      rs_nv_ctrlh,    0,                      0 },
  { K_UP,      rs_nv_up,       NV_SSS|NV_STS,          false },
  { K_S_UP,    rs_nv_page,     NV_SS,                  BACKWARD },
  { K_DOWN,    rs_nv_down,     NV_SSS|NV_STS,          false },
  { K_S_DOWN,  rs_nv_page,     NV_SS,                  FORWARD },
  { K_LEFT,    rs_nv_left,     NV_SSS|NV_STS|NV_RL,    0 },
  { K_S_LEFT,  rs_nv_bck_word, NV_SS|NV_RL,            0 },
  { K_C_LEFT,  rs_nv_bck_word, NV_SSS|NV_RL|NV_STS,    1 },
  { K_RIGHT,   rs_nv_right,    NV_SSS|NV_STS|NV_RL,    0 },
  { K_S_RIGHT, rs_nv_wordcmd,  NV_SS|NV_RL,            false },
  { K_C_RIGHT, rs_nv_wordcmd,  NV_SSS|NV_RL|NV_STS,    true },
  { K_PAGEUP,  rs_nv_page,     NV_SSS|NV_STS,          BACKWARD },
  { K_KPAGEUP, rs_nv_page,     NV_SSS|NV_STS,          BACKWARD },
  { K_PAGEDOWN, rs_nv_page,    NV_SSS|NV_STS,          FORWARD },
  { K_KPAGEDOWN, rs_nv_page,   NV_SSS|NV_STS,          FORWARD },
  { K_END,     rs_nv_end,      NV_SSS|NV_STS,          false },
  { K_KEND,    rs_nv_end,      NV_SSS|NV_STS,          false },
  { K_S_END,   rs_nv_end,      NV_SS,                  false },
  { K_C_END,   rs_nv_end,      NV_SSS|NV_STS,          true },
  { K_HOME,    rs_nv_home,     NV_SSS|NV_STS,          0 },
  { K_KHOME,   rs_nv_home,     NV_SSS|NV_STS,          0 },
  { K_S_HOME,  rs_nv_home,     NV_SS,                  0 },
  { K_C_HOME,  rs_nv_goto,     NV_SSS|NV_STS,          false },
  { K_DEL,     rs_nv_abbrev,   0,                      0 },
  { K_KDEL,    rs_nv_abbrev,   0,                      0 },
  { K_UNDO,    rs_nv_kundo,    0,                      0 },
  { K_HELP,    rs_nv_help,     NV_NCW,                 0 },
  { K_F1,      rs_nv_help,     NV_NCW,                 0 },
  { K_XF1,     rs_nv_help,     NV_NCW,                 0 },
  { K_SELECT,  rs_nv_select,   0,                      0 },
  { K_PASTE_START, rs_nv_paste, NV_KEEPREG,             0 },
  { K_EVENT,   rs_nv_event,    NV_KEEPREG,             0 },
  { K_COMMAND, rs_nv_colon,    0,                      0 },
  { K_LUA, rs_nv_colon,        0,                      0 },
};

// Number of commands in nv_cmds[].
#define NV_CMDS_SIZE ARRAY_SIZE(nv_cmds)

// Sorted index of commands in nv_cmds[].
static int16_t nv_cmd_idx[NV_CMDS_SIZE];

// The highest index for which
// nv_cmds[idx].cmd_char == nv_cmd_idx[nv_cmds[idx].cmd_char]
static int nv_max_linear;

// =============================================================================
// Rust FFI declarations (non-dispatch-table)
// =============================================================================

// memline crate
extern void rs_goto_byte(int cnt);

// Normal mode state machine
extern int rs_normal_check(void *s);
extern int rs_normal_execute(void *s, int key);
extern void rs_normal_prepare(void *s);
extern bool rs_normal_get_command_count(void *s);
extern bool rs_normal_handle_special_visual_command(void *s);
extern bool rs_normal_need_redraw_mode_message(void *s);
extern void rs_normal_redraw_mode_message(void *s);
extern void rs_normal_redraw(void *s);
extern bool rs_need_additional_char(int idx, int cmdchar, bool pending_op);

// Operator/command helpers
extern bool rs_op_pending(void);
extern int rs_find_command(int cmdchar);
extern int rs_invert_horizontal(int cmdchar);
extern int rs_unshift_special(int cmdchar, int *modp);
extern bool rs_is_ident(const char *line, int offset);
extern void rs_clearop(oparg_T *oap);
extern void rs_clearopbeep(oparg_T *oap);
extern bool rs_checkclearop(oparg_T *oap);
extern bool rs_checkclearopq(oparg_T *oap);
extern bool rs_check_text_locked(oparg_T *oap);
extern bool rs_check_text_or_curbuf_locked(oparg_T *oap);
extern void rs_prep_redo(int regname, int num, int cmd1, int cmd2, int cmd3, int cmd4, int cmd5);
extern void rs_prep_redo_cmd(cmdarg_T *cap);
extern void rs_set_vcount_ca(cmdarg_T *cap, bool *set_prevcount);
extern void rs_reset_VIsual_and_resel(void);
extern void rs_may_clear_cmdline(void);
extern void rs_v_visop(cmdarg_T *cap);
extern void rs_may_start_select(int c);
extern void rs_v_swap_corners(int cmdchar);
extern bool rs_unadjust_for_sel(void);
extern bool rs_get_visual_text(cmdarg_T *cap, char **pp, size_t *lenp);
extern int rs_nv_mark_move_to(cmdarg_T *cap, int flags, fmark_T *fm);
extern size_t rs_find_ident_at_pos(win_T *wp, linenr_T lnum, colnr_T startcol,
                                   char **text, int *textcol, int find_type);

// Window/fold/quickfix/diff helpers
extern void rs_set_fraction(win_T *wp);
extern void rs_win_setheight(int height);
extern void rs_qf_view_result(bool split);
extern int rs_hasAnyFolding(win_T *win);
extern void rs_foldOpenCursor(void);
extern void rs_foldCheckClose(void);
extern void rs_newFoldLevel(void);
extern int rs_foldManualAllowed(bool create);
extern void rs_clearFolding(win_T *win);
extern int rs_foldMoveTo(bool updown, int dir, int count);
extern void rs_foldAdjustVisual(void);
extern int rs_getDeepestNesting(win_T *wp);
extern void rs_deleteFold(win_T *wp, linenr_T start, linenr_T end, int recursive, bool had_visual);
extern void rs_foldUpdateAfterInsert(void);
extern void rs_setFoldRepeat(linenr_T lnum, int count, bool do_open);
extern linenr_T rs_setManualFold(linenr_T lnum, bool opening, bool recurse, int *donep);
extern void rs_do_ascii(exarg_T *eap);
extern void rs_diff_set_topline(win_T *fromwin, win_T *towin);
extern int rs_diff_move_to(int dir, int count);
extern int rs_get_vtopline(win_T *wp);
extern int rs_get_sidescrolloff_value(win_T *wp);
extern const char *rs_get_showbreak_value(win_T *win);
extern void rs_n_start_visual_mode(int c);
// rs_end_visual_mode deleted: now exported as end_visual_mode via #[export_name]
extern void rs_set_cursor_for_append_to_line(void);
extern void rs_set_op_var(int optype);
extern size_t rs_find_ident_under_cursor(char **text, int find_type);
// invoke_edit: now exported from Rust via #[export_name = "invoke_edit"]
extern void invoke_edit(cmdarg_T *cap, int repl, int cmd, int startln);
// del_from_showcmd: now exported from Rust via #[export_name = "del_from_showcmd"]
extern void del_from_showcmd(int len);

/// Compare functions for qsort() below, that checks the command character
/// through the index in nv_cmd_idx[].
static int nv_compare(const void *s1, const void *s2)
{
  // The commands are sorted on absolute value.
  int c1 = nv_cmds[*(const int16_t *)s1].cmd_char;
  int c2 = nv_cmds[*(const int16_t *)s2].cmd_char;
  if (c1 < 0) {
    c1 = -c1;
  }
  if (c2 < 0) {
    c2 = -c2;
  }
  return c1 == c2 ? 0 : c1 > c2 ? 1 : -1;
}

/// Initialize the nv_cmd_idx[] table.
void init_normal_cmds(void)
{
  assert(NV_CMDS_SIZE <= SHRT_MAX);

  // Fill the index table with a one to one relation.
  for (int16_t i = 0; i < (int16_t)NV_CMDS_SIZE; i++) {
    nv_cmd_idx[i] = i;
  }

  // Sort the commands by the command character.
  qsort(&nv_cmd_idx, NV_CMDS_SIZE, sizeof(int16_t), nv_compare);

  // Find the first entry that can't be indexed by the command character.
  int16_t i;
  for (i = 0; i < (int16_t)NV_CMDS_SIZE; i++) {
    if (i != nv_cmds[nv_cmd_idx[i]].cmd_char) {
      break;
    }
  }
  nv_max_linear = i - 1;
}

static oparg_T *current_oap = NULL;

// =============================================================================
// Accessor functions for Rust FFI
// =============================================================================

/// Check if current_oap is NULL.
int nvim_oap_is_null(void) { return current_oap == NULL; }

int nvim_oap_get_prev_opcount(void) { return current_oap ? current_oap->prev_opcount : 0; }

int nvim_oap_get_prev_count0(void) { return current_oap ? current_oap->prev_count0 : 0; }

int nvim_oap_get_op_type(void) { return current_oap ? current_oap->op_type : OP_NOP; }

int nvim_oap_get_regname(void) { return current_oap ? current_oap->regname : NUL; }

int nvim_get_opcount(void) { return opcount; }

void nvim_set_opcount(int val) { opcount = val; }

int nvim_get_nv_max_linear(void) { return nv_max_linear; }

int nvim_get_nv_cmd_char(int idx) { return (idx >= 0 && (size_t)idx < NV_CMDS_SIZE) ? nv_cmds[idx].cmd_char : 0; }

int nvim_get_nv_cmds_size(void) { return (int)NV_CMDS_SIZE; }

int16_t nvim_get_nv_cmd_idx(int idx) { return (idx >= 0 && (size_t)idx < NV_CMDS_SIZE) ? nv_cmd_idx[idx] : 0; }

unsigned int nvim_get_nv_cmd_flags(int idx) { return (idx >= 0 && (size_t)idx < NV_CMDS_SIZE) ? nv_cmds[idx].cmd_flags : 0; }

int nvim_get_nv_cmd_arg(int idx) { return (idx >= 0 && (size_t)idx < NV_CMDS_SIZE) ? nv_cmds[idx].cmd_arg : 0; }

// =============================================================================
// oparg_T pointer accessors for Rust FFI (takes explicit oap parameter)
// =============================================================================

int nvim_oap_get_op_type_ptr(oparg_T *oap) { return oap ? oap->op_type : OP_NOP; }

void nvim_oap_set_op_type(oparg_T *oap, int val) { if (oap) oap->op_type = val; }

int nvim_oap_get_regname_ptr(oparg_T *oap) { return oap ? oap->regname : NUL; }

void nvim_oap_set_regname(oparg_T *oap, int val) { if (oap) oap->regname = val; }

int nvim_oap_get_motion_force(oparg_T *oap) { return oap ? oap->motion_force : NUL; }

void nvim_oap_set_motion_force(oparg_T *oap, int val) { if (oap) oap->motion_force = val; }

void nvim_oap_set_use_reg_one(oparg_T *oap, bool val) { if (oap) oap->use_reg_one = val; }

int nvim_oap_get_motion_type(oparg_T *oap) { return oap ? oap->motion_type : kMTUnknown; }

void nvim_oap_set_motion_type(oparg_T *oap, int val) { if (oap) oap->motion_type = val; }

bool nvim_oap_get_inclusive(oparg_T *oap) { return oap ? oap->inclusive : false; }

void nvim_oap_set_inclusive(oparg_T *oap, bool val) { if (oap) oap->inclusive = val; }

// =============================================================================
// Additional oparg_T accessors for Rust ops crate
// =============================================================================

int nvim_oap_get_use_reg_one(oparg_T *oap) { return oap ? oap->use_reg_one : false; }

int nvim_oap_get_line_count(oparg_T *oap) { return oap ? oap->line_count : 0; }

void nvim_oap_set_line_count(oparg_T *oap, int val) { if (oap) oap->line_count = val; }

int nvim_oap_get_empty(oparg_T *oap) { return oap ? oap->empty : false; }

void nvim_oap_set_empty(oparg_T *oap, int val) { if (oap) oap->empty = val != 0; }

int nvim_oap_get_is_visual(oparg_T *oap) { return oap ? oap->is_VIsual : false; }

int nvim_oap_get_excl_tr_ws(oparg_T *oap) { return oap ? oap->excl_tr_ws : false; }

int nvim_oap_get_start_lnum(oparg_T *oap) { return oap ? oap->start.lnum : 0; }

int nvim_oap_get_start_col(oparg_T *oap) { return oap ? oap->start.col : 0; }

int nvim_oap_get_start_coladd(oparg_T *oap) { return oap ? oap->start.coladd : 0; }

int nvim_oap_get_end_lnum(oparg_T *oap) { return oap ? oap->end.lnum : 0; }

int nvim_oap_get_end_col(oparg_T *oap) { return oap ? oap->end.col : 0; }

int nvim_oap_get_end_coladd(oparg_T *oap) { return oap ? oap->end.coladd : 0; }

int nvim_oap_get_start_vcol(oparg_T *oap) { return oap ? oap->start_vcol : 0; }

int nvim_oap_get_end_vcol(oparg_T *oap) { return oap ? oap->end_vcol : 0; }

void nvim_set_motion_force(int val) { motion_force = val; }

void nvim_goto_tabpage(int n) { goto_tabpage(n); }

void nvim_pagescroll(int dir, int count, bool half) { pagescroll(dir, count, half); }

bool nvim_get_VIsual_select(void) { return VIsual_select; }

void nvim_set_VIsual_select(bool val) { VIsual_select = val; }

void nvim_may_trigger_modechanged(void) { may_trigger_modechanged(); }

void nvim_showmode(void) { showmode(); }

void nvim_fileinfo(int fullname, bool shorthelp, bool dont_truncate) { fileinfo(fullname, shorthelp, dont_truncate); }

void nvim_scroll_redraw(bool down, int count) { scroll_redraw(down, count); }

void nvim_u_undo(int count) { u_undo(count); }

void nvim_curwin_set_curswant(bool val) { curwin->w_set_curswant = val; }

linenr_T nvim_get_line_count(void) { return curbuf->b_ml.ml_line_count; }

linenr_T nvim_get_cursor_lnum(void) { return curwin->w_cursor.lnum; }

void nvim_set_cursor_lnum(linenr_T lnum) { curwin->w_cursor.lnum = lnum; }

void nvim_setpcmark(void) { setpcmark(); }

void nvim_beginline(int flags) { beginline(flags); }

bool nvim_cursor_down(int n, bool upd_topline) { return cursor_down(n, upd_topline); }

bool nvim_get_KeyTyped(void) { return KeyTyped; }

/// Get fdo_flags global.
unsigned int nvim_get_fdo_flags(void)
{
  // Guards: ensure Rust constants match C kOptFdoFlag* values
  _Static_assert(kOptFdoFlagHor == 0x04,
                 "kOptFdoFlagHor changed - update K_OPT_FDO_FLAG_HOR in normal/src/lib.rs");
  _Static_assert(kOptFdoFlagBlock == 0x02,
                 "kOptFdoFlagBlock changed - update K_OPT_FDO_FLAG_BLOCK in normal/src/lib.rs");
  _Static_assert(kOptFdoFlagJump == 0x400,
                 "kOptFdoFlagJump changed - update K_OPT_FDO_FLAG_JUMP in normal/src/lib.rs");
  _Static_assert(kOptFdoFlagMark == 0x08,
                 "kOptFdoFlagMark changed - update K_OPT_FDO_FLAG_MARK in normal/src/lib.rs");
  return fdo_flags;
}

void nvim_set_ins_at_eol(bool val) { ins_at_eol = val; }

void nvim_set_curswant(colnr_T val) { curwin->w_curswant = val; }

bool nvim_virtual_active(void) { return virtual_active(curwin); }

int nvim_gchar_cursor(void) { return utf_ptr2char(get_cursor_pos_ptr()); }

void nvim_coladvance(colnr_T col) { coladvance(curwin, col); }

// =============================================================================
// cmdarg_T accessors for Rust FFI
// =============================================================================

oparg_T *nvim_cap_get_oap(cmdarg_T *cap) { return cap ? cap->oap : NULL; }

int nvim_cap_get_retval(cmdarg_T *cap) { return cap ? cap->retval : 0; }

void nvim_cap_set_retval(cmdarg_T *cap, int val) { if (cap) cap->retval = val; }

void nvim_cap_or_retval(cmdarg_T *cap, int val) { if (cap) cap->retval |= val; }

int nvim_cap_get_cmdchar(cmdarg_T *cap) { return cap ? cap->cmdchar : 0; }

void nvim_cap_set_cmdchar(cmdarg_T *cap, int val) { if (cap) cap->cmdchar = val; }

int nvim_cap_get_nchar(cmdarg_T *cap) { return cap ? cap->nchar : 0; }

void nvim_cap_set_nchar(cmdarg_T *cap, int val) { if (cap) cap->nchar = val; }

int nvim_cap_get_extra_char(cmdarg_T *cap) { return cap ? cap->extra_char : 0; }

void nvim_cap_set_extra_char(cmdarg_T *cap, int val) { if (cap) cap->extra_char = val; }

int nvim_cap_get_count0(cmdarg_T *cap) { return cap ? cap->count0 : 0; }

void nvim_cap_set_count0(cmdarg_T *cap, int val) { if (cap) cap->count0 = val; }

int nvim_cap_get_count1(cmdarg_T *cap) { return cap ? cap->count1 : 0; }

void nvim_cap_set_count1(cmdarg_T *cap, int val) { if (cap) cap->count1 = val; }

int nvim_cap_get_opcount(cmdarg_T *cap) { return cap ? cap->opcount : 0; }

void nvim_cap_set_opcount(cmdarg_T *cap, int val) { if (cap) cap->opcount = val; }

int nvim_cap_get_arg(cmdarg_T *cap) { return cap ? cap->arg : 0; }

void nvim_cap_set_arg(cmdarg_T *cap, int val) { if (cap) cap->arg = val; }

int nvim_cap_get_prechar(cmdarg_T *cap) { return cap ? cap->prechar : 0; }

void nvim_cap_set_prechar(cmdarg_T *cap, int val) { if (cap) cap->prechar = val; }

// =============================================================================
// Word motion accessors for Rust FFI
// =============================================================================

int nvim_fwd_word(int count, bool bigword, bool eol) { return fwd_word(count, bigword, eol); }

int nvim_bck_word(int count, bool bigword, bool stop) { return bck_word(count, bigword, stop); }

int nvim_end_word(int count, bool bigword, bool stop, bool empty) { return end_word(count, bigword, stop, empty); }

int nvim_bckend_word(int count, bool bigword, bool eol) { return bckend_word(count, bigword, eol); }

int nvim_findsent(int dir, int count) { return findsent(dir, count); }

bool nvim_findpar(bool *pincl, int dir, int count, int what, bool both) { return findpar(pincl, dir, count, what, both); }

int nvim_get_cursor_col(void) { return curwin->w_cursor.col; }

void nvim_set_cursor_col(int col) { curwin->w_cursor.col = col; }

void nvim_set_cursor_coladd_zero(void) { curwin->w_cursor.coladd = 0; }

int nvim_inc_cursor(void) { return inc(&curwin->w_cursor); }

int nvim_dec_cursor(void) { return dec(&curwin->w_cursor); }

bool nvim_lt_VIsual_cursor(void) { return lt(VIsual, curwin->w_cursor); }

bool nvim_lt_pos_cursor(int lnum, int col) { pos_T startpos = { lnum, col, 0 }; return lt(startpos, curwin->w_cursor); }

void nvim_set_VIsual_select_exclu_adj(bool val) { VIsual_select_exclu_adj = val; }

unsigned int nvim_get_ve_flags(void) { return get_ve_flags(curwin); }

// =============================================================================
// Character search accessors for Rust FFI
// =============================================================================

int nvim_get_VIsual_mode(void) { return VIsual_mode; }

bool nvim_get_VIsual_select_exclu_adj(void) { return VIsual_select_exclu_adj; }

int nvim_searchc(cmdarg_T *cap, bool t_cmd) { return searchc(cap, t_cmd); }

void nvim_getvcol_cursor(int *scol, int *ecol) { getvcol(curwin, &curwin->w_cursor, scol, NULL, ecol); }

void nvim_set_cursor_coladd(int val) { curwin->w_cursor.coladd = val; }

_Static_assert(TAB == 0x09, "TAB changed");

// =============================================================================
// Mark command accessors for Rust FFI
// =============================================================================

bool nvim_setmark(int name) { return setmark(name); }

unsigned int nvim_get_jop_flags(void) { return jop_flags; }

fmark_T *nvim_mark_get(int name) { return mark_get(curbuf, curwin, NULL, kMarkAll, name); }


fmark_T *nvim_get_changelist(int count1) { return get_changelist(curbuf, curwin, count1); }

fmark_T *nvim_get_jumplist(int count1) { return get_jumplist(curwin, count1); }

bool nvim_goto_tabpage_lastused(void) { return goto_tabpage_lastused(); }

int nvim_get_changelistlen(void) { return curbuf->b_changelistlen; }

void nvim_emsg(const char *msg) { emsg(msg); }

const char *nvim_get_e_changelist_is_empty(void) { return _(e_changelist_is_empty); }

const char *nvim_get_e_start_of_changelist(void) { return _("E662: At start of changelist"); }

const char *nvim_get_e_end_of_changelist(void) { return _("E663: At end of changelist"); }

// =============================================================================
// Register command accessors for Rust FFI
// =============================================================================

int nvim_get_expr_register(void) { return get_expr_register(); }

bool nvim_valid_yank_reg(int regname, bool writing) { return valid_yank_reg(regname, writing); }

void nvim_set_reg_var(int regname) { set_reg_var(regname); }

// nv_put C accessors
bool nvim_put_get_save_fen(void) { return curwin->w_p_fen; }
int nvim_get_cb_flags(void) { return cb_flags; }
void *nvim_put_copy_register(int regname) { return copy_register(regname); }
void nvim_put_do_put(int regname, void *savereg, int dir, int count, int flags) { do_put(regname, (yankreg_T *)savereg, dir, count, flags); }
void nvim_put_free_register(void *savereg) { if (savereg != NULL) { free_register((yankreg_T *)savereg); xfree(savereg); } }
void nvim_auto_format_call(void) { auto_format(false, true); }

// =============================================================================
// Put/replace helper accessors for Rust FFI
// =============================================================================

// For nvim_put_check_prompt inlining
int nvim_get_b_prompt_start_lnum_put(void) { return curbuf->b_prompt_start.mark.lnum; }
void nvim_set_cursor_col_to_prompt_text_len(void) { curwin->w_cursor.col = (int)strlen(prompt_text()); }

// For nvim_put_visual_delete inlining
void nvim_set_w_p_fen(bool val) { curwin->w_p_fen = val; }
bool nvim_check_vd_condition(int regname) {
  return !VIsual_active || VIsual_mode == 'V' || regname != '.';
}
void nvim_inc_msg_silent(void) { msg_silent++; }
void nvim_dec_msg_silent(void) { msg_silent--; }
bool nvim_curbuf_ml_empty(void) { return (curbuf->b_ml.ml_flags & ML_EMPTY) != 0; }

// For nvim_put_visual_flags inlining
int nvim_get_cursor_col_vs_b_op_start_col(void) { return curwin->w_cursor.col - curbuf->b_op_start.col; }
int nvim_get_cursor_lnum_vs_b_op_start_lnum(void) { return (int)(curwin->w_cursor.lnum - curbuf->b_op_start.lnum); }

// For nvim_put_was_visual_cleanup inlining
void nvim_set_b_visual_from_op(void) {
  curbuf->b_visual.vi_start = curbuf->b_op_start;
  curbuf->b_visual.vi_end = curbuf->b_op_end;
}
void nvim_inc_b_visual_vi_end(void) { inc(&curbuf->b_visual.vi_end); }

// For nvim_put_delete_empty_line inlining
bool nvim_last_line_is_empty(void) {
  return *ml_get(curbuf->b_ml.ml_line_count) == NUL;
}
void nvim_ml_delete_last_line(void) {
  ml_delete_flags(curbuf->b_ml.ml_line_count, ML_DEL_MESSAGE);
  deleted_lines(curbuf->b_ml.ml_line_count + 1, 1);
}
bool nvim_cursor_lnum_gt_line_count(void) {
  return curwin->w_cursor.lnum > curbuf->b_ml.ml_line_count;
}
void nvim_cursor_lnum_set_to_line_count(void) {
  curwin->w_cursor.lnum = curbuf->b_ml.ml_line_count;
}
void nvim_coladvance_maxcol(void) { coladvance(curwin, MAXCOL); }

// For nvim_replace helpers inlining
void nvim_coladvance_force_val(colnr_T col) { coladvance_force(col); }
int nvim_get_cursor_pos_len_check(void) { return get_cursor_pos_len(); }
int nvim_mb_charlen_cursor(void) { return mb_charlen(get_cursor_pos_ptr()); }
bool nvim_curbuf_b_p_et(void) { return curbuf->b_p_et; }
void nvim_del_chars_call(int count, bool fixpos) { del_chars(count, fixpos); }
void nvim_ins_char_call(int c) { ins_char(c); }
void nvim_ins_char_bytes_from_cap(cmdarg_T *cap) { if (cap && cap->nchar_len > 0) { ins_char_bytes((char *)cap->nchar_composing, (size_t)cap->nchar_len); } }
void nvim_set_last_insert_call(int c) { set_last_insert(c); }
void nvim_set_b_op_start_cursor(void) { curbuf->b_op_start = curwin->w_cursor; }
void nvim_AppendToRedobuff_composing(cmdarg_T *cap) {
  if (cap && cap->nchar_len > 0) {
    AppendToRedobuff(cap->nchar_composing);
  }
}
int nvim_ins_copychar_val(int lnum) { return ins_copychar(lnum); }

// =============================================================================
// Visual mode accessors for Rust FFI
// =============================================================================

void nvim_set_finish_op(bool val) { finish_op = val; }


void nvim_set_VIsual_mode(int val) { VIsual_mode = val; }

void nvim_redraw_curbuf_inverted(void) { redraw_curbuf_later(UPD_INVERTED); }

int nvim_get_resel_VIsual_mode(void) { return resel_VIsual_mode; }

int nvim_get_resel_VIsual_line_count(void) { return resel_VIsual_line_count; }

int nvim_get_resel_VIsual_vcol(void) { return resel_VIsual_vcol; }

void nvim_set_VIsual_lnum(int lnum) { VIsual.lnum = lnum; }

void nvim_set_VIsual_col(int col) { VIsual.col = col; }

void nvim_set_VIsual_coladd(int coladd) { VIsual.coladd = coladd; }

void nvim_set_VIsual_active(bool val) { VIsual_active = val; }

int nvim_get_VIsual_reselect(void) { return VIsual_reselect; }

void nvim_set_VIsual_reselect(bool val) { VIsual_reselect = val; }

void nvim_setmouse(void) { setmouse(); }

int nvim_get_p_smd(void) { return p_smd; }

void nvim_check_cursor(void) { check_cursor(curwin); }

void nvim_update_curswant_force(void) { update_curswant_force(); }

int nvim_get_curswant(void) { return curwin->w_curswant; }

_Static_assert(MAXCOL == 0x7fffffff, "MAXCOL changed");

int nvim_cap_dec_count1(cmdarg_T *cap) { return cap ? --cap->count1 : 0; }

// =============================================================================
// Command handler accessors for Rust FFI
// =============================================================================

/// Clear b_syn_slow for all windows in current tab (for nv_clear).
void nvim_clear_b_syn_slow_all_windows(void) {
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    wp->w_s->b_syn_slow = false;
  }
}

/// syn_stack_free_all(curwin->w_s) wrapper.
void nvim_syn_stack_free_all_curwin(void) { syn_stack_free_all(curwin->w_s); }

int nvim_get_restart_VIsual_select(void) { return restart_VIsual_select; }

void nvim_set_restart_VIsual_select(int val) { restart_VIsual_select = val; }

void nvim_buflist_getfile(int n, int lnum, int flags, bool setpm) { buflist_getfile(n, lnum, flags, setpm); }

_Static_assert(GETF_SETMARK == 0x01, "GETF_SETMARK changed");
_Static_assert(GETF_ALT == 0x02, "GETF_ALT changed");

int nvim_get_curbuf_visual_vi_mode(void) { return curbuf->b_visual.vi_mode; }

void nvim_set_curbuf_visual_vi_mode(int val) { curbuf->b_visual.vi_mode = val; }

bool nvim_get_mode_displayed(void) { return mode_displayed; }

void nvim_set_clear_cmdline(bool val) { clear_cmdline = val; }

// =============================================================================
// Redo/count accessors for Rust FFI
// =============================================================================

int nvim_cap_get_nchar_len(cmdarg_T *cap) { return cap ? cap->nchar_len : 0; }

void nvim_cap_append_nchar_composing_to_redobuff(cmdarg_T *cap) { if (cap) AppendToRedobuff(cap->nchar_composing); }

void nvim_set_vcount_call(int64_t count, int64_t count1, bool set_prevcount) { set_vcount(count, count1, set_prevcount); }

bool nvim_do_execreg_recorded(void) { return do_execreg(reg_recorded, false, false, false) != false; }

bool nvim_normal_get_got_int(void) { return got_int; }

void nvim_normal_line_breakcheck(void) { line_breakcheck(); }

// =============================================================================
// Visual operator accessors for Rust FFI
// =============================================================================
_Static_assert(Ctrl_V == 22, "Ctrl_V mismatch");
_Static_assert(OP_DELETE == 1, "OP_DELETE mismatch");
_Static_assert(OP_YANK == 2, "OP_YANK mismatch");
_Static_assert(OP_LSHIFT == 4, "OP_LSHIFT mismatch");
_Static_assert(OP_RSHIFT == 5, "OP_RSHIFT mismatch");
_Static_assert(BL_WHITE == 1, "BL_WHITE mismatch");
_Static_assert(K_DEL == TERMCAP2KEY('k', 'D'), "K_DEL mismatch");
_Static_assert(K_KDEL == TERMCAP2KEY(KS_EXTRA, KE_KDEL), "K_KDEL mismatch");
_Static_assert(kMTLineWise == 1, "kMTLineWise mismatch");

// =============================================================================
// Selection/g-cmd accessors for Rust FFI
// =============================================================================
_Static_assert(Ctrl_N == 14, "Ctrl_N mismatch");
_Static_assert(Ctrl_G == 7, "Ctrl_G mismatch");
_Static_assert(Ctrl_C == 3, "Ctrl_C mismatch");
_Static_assert(kMTCharWise == 0, "kMTCharWise mismatch");

int nvim_get_cursor_line_byte_at_col(int col) { char *ptr = get_cursor_line_ptr(); return (uint8_t)ptr[col]; }

bool nvim_cursor_line_col_is_white(int col) { char *ptr = get_cursor_line_ptr(); return ascii_iswhite(ptr[col]); }

bool nvim_stuff_empty(void) { return stuff_empty(); }

bool nvim_typebuf_typed(void) { return typebuf_typed(); }

bool nvim_vim_strchr_p_slm(int c) { return vim_strchr(p_slm, c) != NULL; }

bool nvim_set_cursor_from_last_insert(void) { if (curbuf->b_last_insert.mark.lnum != 0) { curwin->w_cursor = curbuf->b_last_insert.mark; return true; } return false; }

void nvim_check_cursor_lnum_call(void) { check_cursor_lnum(curwin); }

int nvim_get_cursor_line_len(void) { return (int)get_cursor_line_len(); }

int nvim_get_cursor_coladd(void) { return curwin->w_cursor.coladd; }

int nvim_normal_get_cmdwin_type(void) { return cmdwin_type; }

void nvim_set_cmdwin_result(int val) { cmdwin_result = val; }

// =============================================================================
// Visual complex function accessors for Rust FFI
// =============================================================================

// Guards: ensure Rust constants match C values
_Static_assert(kOptFdoFlagPercent == 0x10,
               "kOptFdoFlagPercent changed - update K_OPT_FDO_FLAG_PERCENT in normal/src/lib.rs");
_Static_assert(BL_SOL == 2,
               "BL_SOL changed - update BL_SOL in normal/src/lib.rs");
_Static_assert(BL_FIX == 4,
               "BL_FIX changed - update BL_FIX in normal/src/lib.rs");
_Static_assert(UPD_INVERTED == 20,
               "UPD_INVERTED changed - update UPD_INVERTED in normal/src/lib.rs");

void nvim_set_VIsual_pos(int lnum, int col, int coladd) { VIsual.lnum = lnum; VIsual.col = col; VIsual.coladd = coladd; }

void nvim_set_cursor_pos(int lnum, int col, int coladd) { curwin->w_cursor.lnum = lnum; curwin->w_cursor.col = col; curwin->w_cursor.coladd = coladd; }

int nvim_get_b_visual_vi_start_lnum(void) { return curbuf->b_visual.vi_start.lnum; }

int nvim_get_b_visual_vi_start_col(void) { return curbuf->b_visual.vi_start.col; }

int nvim_get_b_visual_vi_start_coladd(void) { return curbuf->b_visual.vi_start.coladd; }

void nvim_set_b_visual_vi_start(int lnum, int col, int coladd) { curbuf->b_visual.vi_start.lnum = lnum; curbuf->b_visual.vi_start.col = col; curbuf->b_visual.vi_start.coladd = coladd; }

int nvim_get_b_visual_vi_end_lnum(void) { return curbuf->b_visual.vi_end.lnum; }

int nvim_get_b_visual_vi_end_col(void) { return curbuf->b_visual.vi_end.col; }

int nvim_get_b_visual_vi_end_coladd(void) { return curbuf->b_visual.vi_end.coladd; }

void nvim_set_b_visual_vi_end(int lnum, int col, int coladd) { curbuf->b_visual.vi_end.lnum = lnum; curbuf->b_visual.vi_end.col = col; curbuf->b_visual.vi_end.coladd = coladd; }

int nvim_get_b_visual_vi_curswant(void) { return curbuf->b_visual.vi_curswant; }

void nvim_set_b_visual_vi_curswant(int val) { curbuf->b_visual.vi_curswant = val; }

void nvim_set_curbuf_visual_mode_eval(int val) { curbuf->b_visual_mode_eval = val; }

void nvim_set_VIsual_select_reg(int val) { VIsual_select_reg = val; }
int nvim_get_VIsual_select_reg(void) { return VIsual_select_reg; }
int nvim_get_virtual_op(void) { return (int)virtual_op; }

void nvim_update_topline_call(void) { update_topline(curwin); }

bool nvim_p_sel_is_exclusive(void) { return *p_sel == 'e'; }

bool nvim_equalpos_VIsual_cursor(void) { return equalpos(VIsual, curwin->w_cursor); }

/// Wrapper for getvcols: takes two positions, returns left/right via out-params.
void nvim_getvcols_call(int lnum1, int col1, int coladd1,
                        int lnum2, int col2, int coladd2,
                        int *out_left, int *out_right)
{
  pos_T pos1 = { lnum1, col1, coladd1 };
  pos_T pos2 = { lnum2, col2, coladd2 };
  colnr_T left, right;
  getvcols(curwin, &pos1, &pos2, &left, &right);
  *out_left = left;
  *out_right = right;
}

/// findmatch wrapper: returns success and out-params for position.
bool nvim_findmatch_nul(oparg_T *oap, int *out_lnum, int *out_col, int *out_coladd)
{
  pos_T *pos = findmatch(oap, NUL);
  if (pos == NULL) {
    return false;
  }
  *out_lnum = pos->lnum;
  *out_col = pos->col;
  *out_coladd = pos->coladd;
  return true;
}

/// mark_mb_adjustpos for arbitrary pos (by lnum/col/coladd).
/// Updates *col_out after adjustment and returns new col.
int nvim_mark_mb_adjustpos_pos(int lnum, int col, int *col_out) {
  pos_T pp = { lnum, (colnr_T)col, 0 };
  mark_mb_adjustpos(curbuf, &pp);
  *col_out = pp.col;
  return pp.col;
}

/// getvcol coladd (ce - cs) for arbitrary pos.
int nvim_getvcol_pos_coladd(int lnum, int col, int coladd) {
  pos_T pp = { lnum, (colnr_T)col, (colnr_T)coladd };
  colnr_T cs, ce;
  getvcol(curwin, &pp, &cs, NULL, &ce);
  return (int)(ce - cs);
}

int nvim_ml_get_len_call(int lnum) { return (int)ml_get_len(lnum); }

// =============================================================================
// Search handler accessors for Rust FFI
// =============================================================================

/// Call getcmdline for search and set cap->searchbuf. Returns the searchbuf (or NULL).
char *nvim_getcmdline_for_search(cmdarg_T *cap)
{
  cap->searchbuf = getcmdline(cap->cmdchar, cap->count1, 0, true);
  return cap->searchbuf;
}

// nv_ident C wrappers

/// Initialize nv_ident: determine cmdchar/g_cmd, get visual text or ident under cursor.
/// Returns 0 on success, -1 to return early (clearop done), -2 to return early (clearopq done).
/// On success: *cmdchar_out, *g_cmd_out, *ptr_out, *n_out are set.

/// Wrapper for searchit using curwin/curbuf cursor (for find_decl pattern).
/// Returns 1 on success, 0 on failure.
int nvim_searchit_decl(const char *pat, size_t patlen, int searchflags) { return searchit(curwin, curbuf, &curwin->w_cursor, NULL, FORWARD, (char *)pat, patlen, 1, searchflags, RE_LAST, NULL); }
int nvim_findpar_decl(void) { bool incll; return findpar(&incll, BACKWARD, 1, '{', false) ? 1 : 0; }

/// Wrapper for vim_iswordp for the first char at ptr.
int nvim_vim_iswordp_char(const char *ptr) { return vim_iswordp(ptr) ? 1 : 0; }

/// Wrapper for get_leader_len on cursor line.
int nvim_get_leader_len_cursor_line(void) { return get_leader_len(get_cursor_line_ptr(), NULL, false, true); }

/// Check if first non-white char of cursor line is NUL (line is blank/whitespace).
int nvim_cursor_line_is_blank(void) { return *skipwhite(get_cursor_line_ptr()) == NUL ? 1 : 0; }

/// Wrapper for reset_search_dir().
void nvim_reset_search_dir(void) { reset_search_dir(); }

/// Get p_ws as int.
int nvim_get_p_ws_bool(void) { return p_ws ? 1 : 0; }

/// Set p_ws from int.
void nvim_set_p_ws_bool(int val) { p_ws = val != 0; }

/// Get p_scs as int.
int nvim_get_p_scs_bool(void) { return p_scs ? 1 : 0; }

/// Set p_scs from int.
void nvim_set_p_scs_bool(int val) { p_scs = val != 0; }

/// Set cursor col to 0.
void nvim_set_cursor_col_zero_val(void) { curwin->w_cursor.col = 0; }

/// Cursor lnum decrement.
void nvim_cursor_lnum_dec_val(void) { curwin->w_cursor.lnum--; }

/// findmatchlimit with NULL oap, FM_FORWARD, for block scope in find_decl.
bool nvim_findmatchlimit_forward(int64_t maxtravel,
                                  int *out_lnum, int *out_col, int *out_coladd)
{
  pos_T *pos = findmatchlimit(NULL, '}', FM_FORWARD, (long)maxtravel);
  if (pos == NULL) {
    return false;
  }
  *out_lnum = pos->lnum;
  *out_col = pos->col;
  *out_coladd = pos->coladd;
  return true;
}

// rs_find_decl extern removed: Rust exports as find_decl via #[export_name].

// =============================================================================
// Operator handler accessors for Rust FFI
// =============================================================================

// Accessors for operator Rust implementations
bool nvim_bt_prompt_curbuf(void) { return bt_prompt(curbuf); }
bool nvim_prompt_curpos_editable(void) { return prompt_curpos_editable(); }
bool nvim_op_is_change(int op_type) { return op_is_change(op_type); }
void nvim_oap_set_start_cursor(oparg_T *oap) { oap->start = curwin->w_cursor; }
void nvim_stuffnumReadbuff(int n) { stuffnumReadbuff(n); }
void nvim_stuffReadbuff(const char *s) { stuffReadbuff(s); }

// =============================================================================
// Text object handler accessors for Rust FFI
// =============================================================================

// nv_brackets_impl C accessors for Rust FFI
/// findmatchlimit wrapper that copies pos_T fields to output params.
bool nvim_findmatchlimit_call(oparg_T *oap, int findc, int flags, int64_t maxtravel,
                               int *out_lnum, int *out_col, int *out_coladd)
{
  pos_T *pos = findmatchlimit(oap, findc, flags, (long)maxtravel);
  if (pos == NULL) {
    return false;
  }
  *out_lnum = pos->lnum;
  *out_col = pos->col;
  *out_coladd = pos->coladd;
  return true;
}

/// find_pattern_in_path wrapper for bracket [i/]i/[d/]d commands.
/// Takes a copy of ptr (via xmemdupz) and frees it after the call,
/// matching the original nvim_bracket_find_ident behavior.
void nvim_find_pattern_in_path_call(char *ptr, size_t len, int count0, int nchar,
                                    int64_t count1, bool from_rbracket) {
  char *dup = xmemdupz(ptr, len);
  find_pattern_in_path(dup, 0, len, true,
                       count0 == 0 ? !isupper(nchar) : false,
                       (((nchar & 0xf) == ('d' & 0xf)) ? FIND_DEFINE : FIND_ANY),
                       (int)count1,
                       (isupper(nchar) ? ACTION_SHOW_ALL
                                       : islower(nchar) ? ACTION_SHOW : ACTION_GOTO),
                       (from_rbracket ? curwin->w_cursor.lnum + 1 : 1),
                       MAXLNUM, false, false);
  xfree(dup);
}

/// pos_to_mark(curbuf, NULL, curwin->w_cursor) -- returns fmark_T*.
fmark_T *nvim_pos_to_mark_cursor(void) { return pos_to_mark(curbuf, NULL, curwin->w_cursor); }

/// getnextmark wrapper: advance fm by one step in dir.
fmark_T *nvim_getnextmark_call(fmark_T *fm, int dir, int begin_line) {
  return getnextmark(&fm->mark, dir, begin_line);
}

/// do_mouse wrapper for bracket [<mouse>/]<mouse> commands.
void nvim_bracket_do_mouse_impl(oparg_T *oap, int nchar, int dir, int64_t count1) {
  do_mouse(oap, nchar, dir, (int)count1, PUT_FIXINDENT);
}

/// spell_move_to wrapper for bracket [s/]s/[r/]r/[S/]S commands.
size_t nvim_spell_move_to_cap_call(int dir, int smt_type) {
  return spell_move_to(curwin, dir, smt_type, false, NULL);
}

/// messaging() && !msg_silent && !shortmess(SHM_SEARCHCOUNT).
bool nvim_messaging_and_searchcount(void) {
  return messaging() && !msg_silent && !shortmess(SHM_SEARCHCOUNT);
}

// =============================================================================
// g-command and n_opencmd accessors for Rust FFI
// =============================================================================

int nvim_sms_marker_overlap_curwin(int width) { return sms_marker_overlap(curwin, width); }
void nvim_validate_cheight_curwin(void) { validate_cheight(curwin); }
int nvim_get_curwin_w_skipcol(void) { return (int)curwin->w_skipcol; }
int nvim_get_curwin_w_topline(void) { return (int)curwin->w_topline; }
bool nvim_get_curwin_w_cline_folded(void) { return curwin->w_cline_folded; }
void nvim_clear_curwin_w_valid_wcol(void) { curwin->w_valid &= ~VALID_WCOL; }
int nvim_utf_ptr2cells_cursor(void) { return utf_ptr2cells(get_cursor_pos_ptr()); }
int nvim_getvvcol_cursor_end(void) { colnr_T vcol; getvvcol(curwin, &curwin->w_cursor, NULL, NULL, &vcol); return (int)vcol; }
void nvim_hasFolding_cursor_set_lnum_up(void) { hasFolding(curwin, curwin->w_cursor.lnum, &curwin->w_cursor.lnum, NULL); }
void nvim_hasFolding_cursor_set_lnum_down(void) { hasFolding(curwin, curwin->w_cursor.lnum, NULL, &curwin->w_cursor.lnum); }
void nvim_set_curbuf_b_last_changedtick_i(void) { curbuf->b_last_changedtick_i = buf_get_changedtick(curbuf); }
bool nvim_u_save_for_opencmd(bool backward) { return u_save(curwin->w_cursor.lnum - (backward ? 1 : 0), curwin->w_cursor.lnum + (backward ? 0 : 1)) != 0; }
bool nvim_open_line_for_opencmd(bool backward, bool do_com) { return open_line(backward ? BACKWARD : FORWARD, do_com ? OPENLINE_DO_COM : 0, 0, NULL) != false; }
bool nvim_has_format_option_fo_open_coms(void) { return has_format_option(FO_OPEN_COMS); }
bool nvim_win_cursorline_standout_curwin(void) { return win_cursorline_standout(curwin); }
void nvim_clear_curwin_w_valid_crow(void) { curwin->w_valid &= ~VALID_CROW; }
/// mark_mb_adjustpos for cursor: adjusts curwin->w_cursor via curbuf,
/// returns new col.
int nvim_mark_mb_adjustpos_cursor_new(void) {
  mark_mb_adjustpos(curbuf, &curwin->w_cursor);
  return curwin->w_cursor.col;
}
/// getvcol for cursor pos after mark_mb_adjustpos_cursor: returns coladd = ce - cs.
int nvim_getvcol_cursor_coladd_after_adj(void) {
  colnr_T cs, ce;
  getvcol(curwin, &curwin->w_cursor, &cs, NULL, &ce);
  return (int)(ce - cs);
}
/// mark_mb_adjustpos for VIsual: adjusts VIsual via curbuf, returns new col.
int nvim_mark_mb_adjustpos_visual_new(void) {
  mark_mb_adjustpos(curbuf, &VIsual);
  return VIsual.col;
}
/// getvcol for VIsual pos: returns coladd = ce - cs.
int nvim_getvcol_visual_coladd_after_adj(void) {
  colnr_T cs, ce;
  getvcol(curwin, &VIsual, &cs, NULL, &ce);
  return (int)(ce - cs);
}
// =============================================================================
// Undo/Redo handler accessors for Rust FFI
// =============================================================================

// Accessors for undo/redo Rust implementations
bool nvim_start_redo(int count, bool restart) { return start_redo(count, restart); }
void nvim_u_redo_call(int count) { u_redo(count); }
void nvim_u_undoline_call(void) { u_undoline(); }
bool nvim_do_execreg_call(int regname) { return do_execreg(regname, false, false, false) != false; }
bool nvim_get_p_to(void) { return p_to; }

// =============================================================================
// Insert mode entry handler accessors for Rust FFI
// =============================================================================

// nv_replace C wrappers

/// Check if buffer is a prompt buffer and cursor is not in editable area.
int nvim_replace_check_prompt(void) {
  return (bt_prompt(curbuf) && !prompt_curpos_editable()) ? 1 : 0;
}

// =============================================================================
// Scroll and screen handler accessors for Rust FFI
// =============================================================================

void nvim_prompt_invoke_callback(void) { prompt_invoke_callback(); }
bool nvim_curbuf_modifiable(void) { return MODIFIABLE(curbuf); }
void nvim_coladvance_getviscol(void) { coladvance(curwin, getviscol()); }
void nvim_invoke_edit_R(cmdarg_T *cap, bool repl, int cmd) { invoke_edit(cap, repl, cmd, false); }
int nvim_get_literal_call(bool no_simplify) { return get_literal(no_simplify); }
void nvim_do_join_call(int count, bool insert_space) { do_join((size_t)count, insert_space, true, true, true); }
void nvim_nv_diffgetput_call(bool put, size_t count) { nv_diffgetput(put, count); }
int nvim_cursor_count0_max2(cmdarg_T *cap) { return MAX(cap->count0, 2); }
// z-command accessors for Rust FFI
int nvim_get_curwin_w_p_fdl(void) { return (int)curwin->w_p_fdl; }
void nvim_set_curwin_w_p_fdl(int val) { curwin->w_p_fdl = val; }
void nvim_set_curwin_w_foldinvalid(bool val) { curwin->w_foldinvalid = val; }
int nvim_get_curwin_w_view_width(void) { return curwin->w_view_width; }
int nvim_get_curwin_w_leftcol(void) { return curwin->w_leftcol; }
void nvim_set_curwin_w_leftcol(int val) { curwin->w_leftcol = val; }
void nvim_validate_botline_curwin(void) { validate_botline(curwin); }
int nvim_get_curwin_w_botline(void) { return curwin->w_botline; }
void nvim_check_cursor_col_call(void) { check_cursor_col(curwin); }
void nvim_scroll_cursor_top(int off, bool always) { scroll_cursor_top(curwin, off, always); }
void nvim_scroll_cursor_bot(int off, bool always) { scroll_cursor_bot(curwin, off, always); }
void nvim_scroll_cursor_halfway(bool atend, bool prefer_above) { scroll_cursor_halfway(curwin, atend, prefer_above); }
void nvim_redraw_later_curwin(int type) { redraw_later(curwin, type); }
void nvim_set_leftcol_call(int col) { set_leftcol((colnr_T)col); }
bool nvim_hasFolding_curwin(int lnum) { return hasFolding(curwin, lnum, NULL, NULL); }
void nvim_getvcol_curwin_cursor(int *vcol) { getvcol(curwin, &curwin->w_cursor, vcol, NULL, NULL); }
void nvim_getvcol_curwin_cursor_end(int *vcol) { getvcol(curwin, &curwin->w_cursor, NULL, NULL, vcol); }
int nvim_win_col_off_curwin(void) { return win_col_off(curwin); }
void nvim_changed_window_setting_curwin(void) { changed_window_setting(curwin); }
void nvim_spell_suggest_call(int count) { spell_suggest(count); }
bool nvim_get_curwin_w_p_wrap(void) { return curwin->w_p_wrap; }

/// Wrapper for spell_move_to(curwin, dir, SMT_ALL, true, NULL) for Rust FFI.
size_t nvim_spell_move_to_wrapper(int dir) { return spell_move_to(curwin, dir, SMT_ALL, true, NULL); }

/// Wrapper for ml_get_pos(&curwin->w_cursor) for Rust FFI.
char *nvim_ml_get_pos_cursor(void) { return ml_get_pos(&curwin->w_cursor); }

/// Sync w_p_fen in diff-synced windows for 'z' commands.
void nvim_sync_fen_in_diff_windows(void)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp != curwin && rs_foldmethodIsDiff(wp) && wp->w_p_scb) {
      wp->w_p_fen = curwin->w_p_fen;
      changed_window_setting(wp);
    }
  }
}

/// vim_strchr wrapper for a specific string
bool nvim_vim_strchr_str(const char *str, int c) { return vim_strchr(str, c) != NULL; }

/// Get translated E352 error message.
const char *nvim_get_e352_msg(void) { return _("E352: Cannot erase folds with current 'foldmethod'"); }

// =============================================================================
// Miscellaneous handler accessors for Rust FFI
// =============================================================================

// g-command C accessors for Rust FFI
bool nvim_current_search(int count, bool forward) { return current_search(count, forward); }
int nvim_cursor_up(int count, bool upd_topline) { return cursor_up(count, upd_topline); }
int nvim_linetabsize_curwin(int lnum) { return linetabsize(curwin, lnum); }
void nvim_cursor_pos_info_call(void) { cursor_pos_info(NULL); }
void nvim_invoke_edit_g(cmdarg_T *cap) { invoke_edit(cap, false, 'g', false); }
void nvim_set_mod_mask_ctrl(void) { mod_mask = MOD_MASK_CTRL; }
void nvim_do_mouse_g(oparg_T *oap, int nchar, int count1) { do_mouse(oap, nchar, BACKWARD, count1, 0); }
void nvim_undo_time_call(int count, bool sec, bool file, bool absolute) { undo_time(count, sec, file, absolute); }
void nvim_show_sb_text_call(void) { show_sb_text(); }
void nvim_show_utf8_call(void) { show_utf8(); }
void nvim_utf_find_illegal_call(void) { utf_find_illegal(); }
void nvim_set_oap_cursor_start(oparg_T *oap) { oap->cursor_start = curwin->w_cursor; }
// nv_screengo C accessors for Rust FFI
int nvim_get_curwin_w_virtcol(void) { return curwin->w_virtcol; }
int nvim_get_curwin_ml_line_count(void) { return curwin->w_buffer->b_ml.ml_line_count; }
int nvim_win_col_off2_curwin(void) { return win_col_off2(curwin); }
void nvim_cursor_up_inner_curwin(int n, bool skip_conceal) { cursor_up_inner(curwin, n, skip_conceal); }
void nvim_cursor_down_inner_curwin(int n, bool skip_conceal) { cursor_down_inner(curwin, n, skip_conceal); }
int nvim_oneright_call(void) { return oneright(); }
int nvim_vim_strsize_call(const char *s) { return vim_strsize((char *)s); }
void nvim_adjust_skipcol_call(void) { adjust_skipcol(); }
void nvim_dec_cursor_col(void) { curwin->w_cursor.col--; }

bool nvim_cursor_pos_ptr_is_nul(void) { return *get_cursor_pos_ptr() == NUL; }
bool nvim_lineempty_cursor(void) { return LINEEMPTY(curwin->w_cursor.lnum); }
bool nvim_vim_strchr_p_ww(int c) { return vim_strchr(p_ww, c) != NULL; }
int nvim_utfc_ptr2len_cursor(void) { return utfc_ptr2len(get_cursor_pos_ptr()); }
int nvim_oneleft_call(void) { return oneleft(); }
void nvim_cursor_col_inc_by_utfc(void) { curwin->w_cursor.col += (colnr_T)utfc_ptr2len(get_cursor_pos_ptr()); }
void nvim_set_cursor_col_zero(void) { curwin->w_cursor.col = 0; curwin->w_cursor.coladd = 0; }
static char *nvim_mps_save = NULL;
void nvim_save_and_set_mps(void) { nvim_mps_save = curbuf->b_p_mps; curbuf->b_p_mps = "(:),{:},[:],<:>"; }
void nvim_restore_mps(void) { curbuf->b_p_mps = nvim_mps_save; }
bool nvim_current_tagblock_call(oparg_T *oap, int count, bool include) { return current_tagblock(oap, count, include); }
bool nvim_current_quote_call(oparg_T *oap, int count, bool include, int quotechar) { return current_quote(oap, count, include, (char)quotechar); }

bool nvim_swapchar_call(int op_type, int lnum, int col) { pos_T pos = { .lnum = (linenr_T)lnum, .col = (colnr_T)col, .coladd = 0 }; return swapchar(op_type, &pos); }
bool nvim_u_savesub_call(int lnum) { return u_savesub((linenr_T)lnum); }
void nvim_u_clearline_curbuf(void) { u_clearline(curbuf); }
void nvim_changed_lines_call(int lnum, int col, int lnum_end, bool do_concealed) { changed_lines(curbuf, (linenr_T)lnum, (colnr_T)col, (linenr_T)lnum_end, 0, do_concealed); }
void nvim_set_b_op_start(int lnum, int col, int coladd) { curbuf->b_op_start.lnum = (linenr_T)lnum; curbuf->b_op_start.col = (colnr_T)col; curbuf->b_op_start.coladd = (colnr_T)coladd; }
void nvim_set_b_op_end_cursor(void) { curbuf->b_op_end = curwin->w_cursor; }
void nvim_dec_b_op_end_col(void) { if (curbuf->b_op_end.col > 0) curbuf->b_op_end.col--; }

// =============================================================================
// find_ident_at_pos accessors for Rust FFI
// =============================================================================

/// Constants for find_ident_at_pos (verified with _Static_assert).
_Static_assert(FIND_IDENT == 1, "FIND_IDENT changed");
_Static_assert(FIND_STRING == 2, "FIND_STRING changed");
_Static_assert(FIND_EVAL == 4, "FIND_EVAL changed");
_Static_assert(BACKWARD == -1, "BACKWARD changed");
_Static_assert(FORWARD == 1, "FORWARD changed");

char *nvim_ml_get_buf_wrapper(buf_T *buf, linenr_T lnum) { return ml_get_buf(buf, lnum); }

int nvim_mb_get_class_wrapper(const char *ptr) { return mb_get_class(ptr); }

int nvim_utfc_ptr2len_wrapper(const char *ptr) { return utfc_ptr2len(ptr); }

int nvim_utf_head_off_wrapper(const char *base, const char *ptr) { return utf_head_off(base, ptr); }

void nvim_emsg_no_string_under_cursor(void) { emsg(_("E348: No string under cursor")); }

void nvim_emsg_no_ident_under_cursor(void) { emsg(_(e_noident)); }

// =============================================================================
// NormalState field accessors for Rust FFI
// All take void* (opaque NormalState handle) and cast internally.
// =============================================================================

#define NS(p) ((NormalState *)(p))

int nvim_ns_get_c(void *s) { return NS(s)->c; }
void nvim_ns_set_c(void *s, int val) { NS(s)->c = val; }

bool nvim_ns_get_command_finished(void *s) { return NS(s)->command_finished; }
void nvim_ns_set_command_finished(void *s, bool val) { NS(s)->command_finished = val; }

bool nvim_ns_get_ctrl_w(void *s) { return NS(s)->ctrl_w; }
void nvim_ns_set_ctrl_w(void *s, bool val) { NS(s)->ctrl_w = val; }

bool nvim_ns_get_need_flushbuf(void *s) { return NS(s)->need_flushbuf; }
void nvim_ns_set_need_flushbuf(void *s, bool val) { NS(s)->need_flushbuf = val; }
void nvim_ns_set_need_flushbuf_or(void *s, bool val) { NS(s)->need_flushbuf |= val; }

bool nvim_ns_get_set_prevcount(void *s) { return NS(s)->set_prevcount; }
void nvim_ns_set_set_prevcount(void *s, bool val) { NS(s)->set_prevcount = val; }

int nvim_ns_get_old_mapped_len(void *s) { return NS(s)->old_mapped_len; }
void nvim_ns_set_old_mapped_len(void *s, int val) { NS(s)->old_mapped_len = val; }

int nvim_ns_get_mapped_len(void *s) { return NS(s)->mapped_len; }

int nvim_ns_get_idx(void *s) { return NS(s)->idx; }
void nvim_ns_set_idx(void *s, int val) { NS(s)->idx = val; }

int nvim_ns_get_old_col(void *s) { return NS(s)->old_col; }
void nvim_ns_set_old_col(void *s, int val) { NS(s)->old_col = val; }

bool nvim_ns_get_toplevel(void *s) { return NS(s)->toplevel; }
bool nvim_ns_get_cmdwin(void *s) { return NS(s)->cmdwin; }
bool nvim_ns_get_noexmode(void *s) { return NS(s)->noexmode; }

oparg_T *nvim_ns_get_oa_ptr(void *s) { return &NS(s)->oa; }
cmdarg_T *nvim_ns_get_ca_ptr(void *s) { return &NS(s)->ca; }

int nvim_ns_get_old_pos_lnum(void *s) { return NS(s)->old_pos.lnum; }
int nvim_ns_get_old_pos_col(void *s) { return NS(s)->old_pos.col; }
void nvim_ns_set_old_pos(void *s) { NS(s)->old_pos = curwin->w_cursor; }

/// Compound: CLEAR_FIELD(s->ca) and set s->ca.oap = &s->oa.
void nvim_ns_prepare_ca(void *s)
{
  CLEAR_FIELD(NS(s)->ca);
  NS(s)->ca.oap = &NS(s)->oa;
}

/// Set s->mapped_len.
void nvim_ns_set_mapped_len(void *s, int val) { NS(s)->mapped_len = val; }

/// Get s->oa.prev_opcount via oap handle.
int nvim_oap_get_prev_opcount_ptr(oparg_T *oap) { return oap ? oap->prev_opcount : 0; }

/// Get s->oa.prev_count0 via oap handle.
int nvim_oap_get_prev_count0_ptr(oparg_T *oap) { return oap ? oap->prev_count0 : 0; }

bool nvim_ns_get_previous_got_int(void *s) { return NS(s)->previous_got_int; }
void nvim_ns_set_previous_got_int(void *s, bool val) { NS(s)->previous_got_int = val; }

#undef NS

/// Normal state entry point. This is called on:
///
/// - Startup, In this case the function never returns.
/// - The command-line window is opened (`q:`). Returns when `cmdwin_result` != 0.
/// - The :visual command is called from :global in ex mode, `:global/PAT/visual`
///   for example. Returns when re-entering ex mode (because ex mode recursion is
///   not allowed)
///
/// This used to be called main_loop() on main.c
void normal_enter(bool cmdwin, bool noexmode)
{
  NormalState state;
  normal_state_init(&state);
  oparg_T *prev_oap = current_oap;
  current_oap = &state.oa;
  state.cmdwin = cmdwin;
  state.noexmode = noexmode;
  state.toplevel = (!cmdwin || cmdwin_result == 0) && !noexmode;
  state_enter(&state.state);
  current_oap = prev_oap;
}

// =============================================================================
// normal_get_additional_char accessors for Rust FFI
// =============================================================================

_Static_assert(MODE_REPLACE == 0x110, "MODE_REPLACE changed");
_Static_assert(MODE_LREPLACE == 0x120, "MODE_LREPLACE changed");
_Static_assert(MODE_LANGMAP == 0x20, "MODE_LANGMAP changed");
_Static_assert(MODE_NORMAL_BUSY == 0x1001, "MODE_NORMAL_BUSY changed");
_Static_assert(B_IMODE_LMAP == 1, "B_IMODE_LMAP changed");
_Static_assert(NV_LANG == 0x08, "NV_LANG changed");
_Static_assert(CPO_DIGRAPH == 'D', "CPO_DIGRAPH changed");

/// Wrapper for plain_vgetc.
int nvim_plain_vgetc_wrapper(void) { return plain_vgetc(); }

int nvim_langmap_adjust(int c, bool condition) { LANGMAP_ADJUST(c, condition); return c; }

/// Wrapper for add_to_showcmd.
bool nvim_add_to_showcmd_wrapper(int c) { return add_to_showcmd(c); }

/// Wrapper for del_from_showcmd (static function).
void nvim_del_from_showcmd_wrapper(int n) { del_from_showcmd(n); }

/// Increment no_mapping.
void nvim_inc_no_mapping(void) { no_mapping++; }
/// Decrement no_mapping.
void nvim_dec_no_mapping(void) { no_mapping--; }

/// Increment allow_keys.
void nvim_inc_allow_keys(void) { allow_keys++; }
/// Decrement allow_keys.
void nvim_dec_allow_keys(void) { allow_keys--; }

/// Set did_cursorhold.
void nvim_set_did_cursorhold(bool val) { did_cursorhold = val; }

/// Get curbuf->b_p_iminsert.
int nvim_get_curbuf_b_p_iminsert(void) { return curbuf->b_p_iminsert; }

// nvim_set_State and nvim_get_State are in window.c

/// Wrapper for ui_cursor_shape_no_check_conceal.
void nvim_ui_cursor_shape_no_check_conceal(void) { ui_cursor_shape_no_check_conceal(); }

/// Wrapper for get_digraph.
int nvim_get_digraph(bool flag) { return get_digraph(flag); }

/// Wrapper for vpeekc.
int nvim_vpeekc_wrapper(void) { return vpeekc(); }

/// Wrapper for do_sleep.
void nvim_do_sleep_wrapper(int ms, bool allow_int) { do_sleep(ms, allow_int); }

/// Check vim_strchr(p_cpo, c) != NULL.
bool nvim_vim_strchr_p_cpo(int c) { return vim_strchr(p_cpo, c) != NULL; }

/// Wrapper for vungetc.
void nvim_vungetc_wrapper(int c) { vungetc(c); }

/// Wrapper for get_op_type.
int nvim_get_op_type_wrapper(int c1, int c2) { return get_op_type(c1, c2); }

/// Get p_ttm.
long nvim_get_p_ttm(void) { return p_ttm; }
/// Get p_tm.
long nvim_get_p_tm(void) { return p_tm; }

/// Get MB_BYTE2LEN for a character.
int nvim_get_MB_BYTE2LEN(int c) { return MB_BYTE2LEN(c); }

void nvim_gotchars_ignore_wrapper(void) { no_u_sync++; gotchars_ignore(); no_u_sync--; }

/// Set cap->nchar_len.
void nvim_cap_set_nchar_len(cmdarg_T *cap, int val) { if (cap) { cap->nchar_len = val; } }

/// utf_iscomposing(prev, c, state_ptr) wrapper.
/// state_ptr points to a GraphemeState (int32_t) initialized to 0 (GRAPHEME_STATE_INIT).
bool nvim_utf_iscomposing_check(int prev, int c, int32_t *state_ptr)
{
  return utf_iscomposing(prev, c, (GraphemeState *)state_ptr);
}

/// utf_char2len(c) wrapper.
int nvim_utf_char2len_wrapper(int c) { return utf_char2len(c); }

// =============================================================================
// normal_finish_command accessors for Rust FFI
// =============================================================================

_Static_assert(K_IGNORE == -13821, "K_IGNORE changed");
_Static_assert(K_MOUSEMOVE == -25853, "K_MOUSEMOVE changed");
_Static_assert(K_EVENT == -26365, "K_EVENT changed");
_Static_assert(OP_NOP == 0, "OP_NOP changed");
_Static_assert(OP_COLON == 10, "OP_COLON changed");
_Static_assert(CA_COMMAND_BUSY == 1, "CA_COMMAND_BUSY changed");
_Static_assert(NV_KEEPREG == 0x100, "NV_KEEPREG changed");

/// set_reg_var(get_default_register_name()).
void nvim_set_reg_var_default(void) { set_reg_var(get_default_register_name()); }

/// typebuf_maplen() wrapper.
int nvim_typebuf_maplen_wrapper(void) { return typebuf_maplen(); }

void nvim_do_pending_operator_call(cmdarg_T *ca, int old_col, bool gui_yank) { do_pending_operator(ca, old_col, gui_yank); }

/// Get clear_cmdline global.
bool nvim_get_clear_cmdline(void) { return clear_cmdline; }

/// Get redraw_cmdline global.
bool nvim_get_redraw_cmdline(void) { return redraw_cmdline; }

/// Get msg_scroll global.
bool nvim_get_msg_scroll_val(void) { return msg_scroll; }

/// Set msg_scroll global.
void nvim_set_msg_scroll_val(bool val) { msg_scroll = val; }

/// Get msg_nowait global.
bool nvim_get_msg_nowait_val(void) { return msg_nowait; }


/// Get in_assert_fails global.
bool nvim_get_in_assert_fails(void) { return in_assert_fails; }

/// Get did_wait_return global.
bool nvim_get_did_wait_return_val(void) { return did_wait_return; }

/// Get keep_msg != NULL.
bool nvim_get_keep_msg_not_null(void) { return keep_msg != NULL; }

/// Copy keep_msg, display via msg(), free the copy.
/// Sets msg_hist_off true around the call to prevent history recording.
void nvim_keep_msg_display_and_free(void)
{
  char *p = xstrdup(keep_msg);
  msg_hist_off = true;
  msg(p, keep_msg_hl_id);
  msg_hist_off = false;
  xfree(p);
}

/// Check shortmess(SHM_FILEINFO).
bool nvim_shortmess_fileinfo(void) { return shortmess(SHM_FILEINFO); }

/// fileinfo(false, true, false) call.
void nvim_fileinfo_call(void) { fileinfo(false, true, false); }



/// may_clear_sb_text() call.
void nvim_may_clear_sb_text_call(void) { may_clear_sb_text(); }

/// show_cursor_info_later(false) call.
void nvim_show_cursor_info_later(void) { show_cursor_info_later(false); }

/// update_screen() call.
void nvim_update_screen_call(void) { update_screen(); }

/// redraw_statuslines() call.
void nvim_redraw_statuslines_call(void) { redraw_statuslines(); }

/// Set curbuf->b_last_used to time(NULL).
void nvim_curbuf_set_b_last_used(void) { curbuf->b_last_used = time(NULL); }

/// Compound: display keep_msg if must_redraw and not emsg_on_display.
/// This wraps the "if (must_redraw && keep_msg != NULL && !emsg_on_display)"
/// block in normal_redraw_mode_message.
void nvim_redraw_mode_msg_keep_msg(void)
{
  if (must_redraw && keep_msg != NULL && !emsg_on_display) {
    char *kmsg = keep_msg;
    keep_msg = NULL;
    setcursor();
    update_screen();
    keep_msg = kmsg;
    kmsg = xstrdup(keep_msg);
    msg(kmsg, keep_msg_hl_id);
    xfree(kmsg);
  }
}

/// os_delay(ms, can_interrupt) wrapper.
void nvim_os_delay_wrapper(int ms, bool can_interrupt) { os_delay(ms, can_interrupt); }

/// ui_cursor_shape() wrapper.
void nvim_ui_cursor_shape_wrapper(void) { ui_cursor_shape(); }

/// checkpcmark() wrapper.
void nvim_checkpcmark_wrapper(void) { checkpcmark(); }

/// Free ca->searchbuf and null it.
void nvim_xfree_cap_searchbuf(cmdarg_T *ca) { xfree(ca->searchbuf); ca->searchbuf = NULL; }

/// mb_check_adjust_col(curwin) wrapper.
void nvim_mb_check_adjust_col_wrapper(void) { mb_check_adjust_col(curwin); }

/// curwin->w_p_scb.
bool nvim_curwin_get_p_scb(void) { return curwin->w_p_scb; }

/// curwin->w_p_crb.
bool nvim_curwin_get_p_crb(void) { return curwin->w_p_crb; }

/// validate_cursor(curwin) wrapper.
void nvim_validate_cursor_curwin_wrapper(void) { validate_cursor(curwin); }

/// do_check_scrollbind(flag) wrapper.
void nvim_do_check_scrollbind_wrapper(bool flag) { do_check_scrollbind(flag); }

/// do_check_cursorbind() wrapper.
void nvim_do_check_cursorbind_wrapper(void) { do_check_cursorbind(); }

/// edit(cmd, startln, count) wrapper.
void nvim_edit_wrapper(int cmd, bool startln, int count) { edit(cmd, startln, count); }

// =============================================================================
// normal_execute accessors for Rust FFI
// =============================================================================

_Static_assert(K_KENTER == -16715, "K_KENTER changed");
_Static_assert(K_ZERO == -22783, "K_ZERO changed");
_Static_assert(ESC == 27, "ESC changed");
_Static_assert(NL == 10, "NL changed");
_Static_assert(CAR == 13, "CAR changed");
_Static_assert(Ctrl_W == 23, "Ctrl_W changed");
_Static_assert(MOD_MASK_SHIFT == 0x02, "MOD_MASK_SHIFT changed");
_Static_assert(MODE_NORMAL == 0x01, "MODE_NORMAL changed");
_Static_assert(MODE_SELECT == 0x40, "MODE_SELECT changed");
_Static_assert(NV_NCW == 0x200, "NV_NCW changed");
_Static_assert(NV_RL == 0x80, "NV_RL changed");
_Static_assert(NV_SS == 0x10, "NV_SS changed");
_Static_assert(NV_SSS == 0x20, "NV_SSS changed");
_Static_assert(NV_STS == 0x40, "NV_STS changed");

/// Get vgetc_char global.
int nvim_get_vgetc_char(void) { return vgetc_char; }

/// Get vgetc_mod_mask global.
int nvim_get_vgetc_mod_mask(void) { return vgetc_mod_mask; }

/// Get km_startsel global.
bool nvim_get_km_startsel(void) { return km_startsel; }

/// Get km_stopsel global.
bool nvim_get_km_stopsel(void) { return km_stopsel; }

/// Increment no_zero_mapping.
void nvim_inc_no_zero_mapping(void) { no_zero_mapping++; }
/// Decrement no_zero_mapping.
void nvim_dec_no_zero_mapping(void) { no_zero_mapping--; }

/// Get curwin->w_p_rl.
bool nvim_get_curwin_w_p_rl(void) { return curwin->w_p_rl; }

/// Set oa->prev_opcount via oap handle.
void nvim_oap_set_prev_opcount(oparg_T *oap, int val) { oap->prev_opcount = val; }

/// Set oa->prev_count0 via oap handle.
void nvim_oap_set_prev_count0(oparg_T *oap, int val) { oap->prev_count0 = val; }

/// ui_flush() wrapper.
void nvim_ui_flush_wrapper(void) { ui_flush(); }

/// Clear MOD_MASK_SHIFT from mod_mask.
void nvim_mod_mask_clear_shift(void) { mod_mask &= ~MOD_MASK_SHIFT; }

void nvim_execute_nv_cmd(int idx, cmdarg_T *ca) { ca->arg = nv_cmds[idx].cmd_arg; (nv_cmds[idx].cmd_func)(ca); }

static int normal_execute(VimState *state, int key) { return rs_normal_execute((NormalState *)state, key); }


// =============================================================================
// normal_check accessors for Rust FFI
// =============================================================================

/// Get did_throw global.
bool nvim_get_did_throw_direct(void) { return did_throw; }

/// Set quit_more global.
void nvim_set_quit_more(bool val) { quit_more = val; }

/// Get skip_redraw global.
bool nvim_get_skip_redraw(void) { return skip_redraw; }

/// Set skip_redraw global.
void nvim_set_skip_redraw(bool val) { skip_redraw = val; }

/// setcursor() wrapper.
void nvim_setcursor_wrapper(void) { setcursor(); }

bool nvim_curtab_needs_diff_update(void) { return curtab->tp_diff_update || curtab->tp_diff_invalid; }

/// Clear curtab diff update flag.
void nvim_curtab_clear_diff_update(void) { curtab->tp_diff_update = false; }

/// Get diff_need_scrollbind global.
bool nvim_get_diff_need_scrollbind(void) { return diff_need_scrollbind; }

/// Set diff_need_scrollbind global.
void nvim_set_diff_need_scrollbind(bool val) { diff_need_scrollbind = val; }

/// check_scrollbind(0, 0) wrapper.
void nvim_check_scrollbind_zero_wrapper(void) { check_scrollbind(0, 0); }

/// time_fd != NULL check.
bool nvim_get_time_fd_not_null(void) { return time_fd != NULL; }

void nvim_time_msg_first_screen_and_finish(void) { TIME_MSG("first screen update"); time_finish(); }

void nvim_may_make_initial_scroll_size_snapshot_wrapper(void) { may_make_initial_scroll_size_snapshot(); }

/// Set may_garbage_collect global.
void nvim_set_may_garbage_collect(bool val) { may_garbage_collect = val; }

/// update_curswant() wrapper.
void nvim_update_curswant_wrapper(void) { update_curswant(); }

/// Get cmdwin_result global.
int nvim_get_cmdwin_result(void) { return cmdwin_result; }

/// do_exmode() wrapper.
void nvim_do_exmode_wrapper(void) { do_exmode(); }

// =============================================================================
// normal_check and normal_redraw accessors for Rust FFI
// =============================================================================

// For normal_check_window_scrolled
void nvim_may_trigger_win_scrolled_resized_call(void) { may_trigger_win_scrolled_resized(); }

// For normal_check_safe_state
void nvim_may_trigger_safestate_call(bool safe) { may_trigger_safestate(safe); }

// For normal_check_folds
bool nvim_char_avail_call(void) { return char_avail(); }
bool nvim_fdo_has_all_flag(void) { return (fdo_flags & kOptFdoFlagAll) != 0; }

// For normal_check_stuff_buffer
void nvim_set_did_check_timestamps(bool val) { did_check_timestamps = val; }
bool nvim_get_need_check_timestamps(void) { return need_check_timestamps; }
void nvim_check_timestamps_call(bool focus) { check_timestamps(focus); }

// For normal_check_interrupt
bool nvim_get_quit_more(void) { return quit_more; }
void nvim_vgetc_and_discard(void) { (void)vgetc(); }
void nvim_set_exmode_active(bool val) { exmode_active = val; }

/// Check if last_cursormoved_win != curwin or cursor position differs.
bool nvim_last_cursormoved_check(void)
{
  return last_cursormoved_win != curwin || !equalpos(last_cursormoved, curwin->w_cursor);
}

/// Update last_cursormoved_win and last_cursormoved to curwin/cursor.
void nvim_update_last_cursormoved(void)
{
  last_cursormoved_win = curwin;
  last_cursormoved = curwin->w_cursor;
}

/// Check if EVENT_CURSORMOVED has listeners.
bool nvim_has_event_cursormoved(void) { return has_event(EVENT_CURSORMOVED); }

/// Fire EVENT_CURSORMOVED autocmds for curbuf.
void nvim_apply_autocmds_cursormoved(void)
{
  apply_autocmds(EVENT_CURSORMOVED, NULL, NULL, false, curbuf);
}

/// Check if EVENT_TEXTCHANGED has listeners.
bool nvim_has_event_textchanged(void) { return has_event(EVENT_TEXTCHANGED); }

/// Fire EVENT_TEXTCHANGED autocmds for curbuf.
void nvim_apply_autocmds_textchanged(void)
{
  apply_autocmds(EVENT_TEXTCHANGED, NULL, NULL, false, curbuf);
}

/// Check if curbuf changedtick has changed since b_last_changedtick.
bool nvim_curbuf_changedtick_changed(void)
{
  return curbuf->b_last_changedtick != buf_get_changedtick(curbuf);
}

/// Update curbuf->b_last_changedtick to the current changedtick.
void nvim_curbuf_update_last_changedtick(void)
{
  curbuf->b_last_changedtick = buf_get_changedtick(curbuf);
}

/// Check if EVENT_BUFMODIFIEDSET has listeners.
bool nvim_has_event_bufmodifiedset(void) { return has_event(EVENT_BUFMODIFIEDSET); }

/// Fire EVENT_BUFMODIFIEDSET autocmds for curbuf.
void nvim_apply_autocmds_bufmodifiedset(void)
{
  apply_autocmds(EVENT_BUFMODIFIEDSET, NULL, NULL, false, curbuf);
}

/// Get curbuf->b_changed_invalid.
bool nvim_curbuf_b_changed_invalid_get(void) { return curbuf->b_changed_invalid; }

/// Clear curbuf->b_changed_invalid.
void nvim_curbuf_b_changed_invalid_clear(void) { curbuf->b_changed_invalid = false; }

static int normal_check(VimState *state) { return rs_normal_check((NormalState *)state); }

// end_visual_mode deleted: now exported directly from Rust via #[export_name]

// =============================================================================
// showcmd accessors for Rust FFI
// =============================================================================

/// Constants for clear_showcmd (verified with _Static_assert).
_Static_assert(SHOWCMD_COLS == 10, "SHOWCMD_COLS changed");
_Static_assert(SHOWCMD_BUFLEN == SHOWCMD_COLS + 1 + 30, "SHOWCMD_BUFLEN changed");

char *nvim_normal_showcmd_buf_ptr(void) { return showcmd_buf; }

/// Returns the first character of p_sloc option.
int nvim_showcmd_get_p_sloc_first(void) { return (unsigned char)*p_sloc; }

/// Set w_redr_status for curwin (showcmdloc=statusline, clear path).
void nvim_showcmd_set_w_redr_status(void) { curwin->w_redr_status = true; }

/// Redraw statusline for curwin and restore cursor (showcmdloc=statusline, non-clear path).
void nvim_showcmd_win_redr_status(void) { win_redr_status(curwin); setcursor(); }

/// Set redraw_tabline flag (showcmdloc=tabline, clear path).
void nvim_showcmd_set_redraw_tabline(void) { redraw_tabline = true; }

/// Redraw tabline and restore cursor (showcmdloc=tabline, non-clear path).
void nvim_showcmd_draw_tabline(void) { draw_tabline(); setcursor(); }

/// Send showcmd via UI messages protocol.
/// If is_clear, sends an empty array; otherwise sends [{0, buf, 0}].
void nvim_showcmd_ui_msg_showcmd(const char *buf, bool is_clear)
{
  MAXSIZE_TEMP_ARRAY(content, 1);
  MAXSIZE_TEMP_ARRAY(chunk, 3);
  if (!is_clear) {
    ADD_C(chunk, INTEGER_OBJ(0));
    ADD_C(chunk, CSTR_AS_OBJ(buf));
    ADD_C(chunk, INTEGER_OBJ(0));
    ADD_C(content, ARRAY_OBJ(chunk));
  }
  ui_call_msg_showcmd(content);
}

/// Returns p_ch option value.
int nvim_showcmd_get_p_ch(void) { return (int)p_ch; }

/// Render the showcmd area on the grid last line.
/// buf is the current showcmd text (NULL or empty means clear).
/// len is the number of display cells already written (from grid_line_puts).
/// Performs msg_grid_validate, grid_line_start, grid_line_puts (content),
/// grid_line_puts (padding), grid_line_flush.
void nvim_showcmd_grid_render(const char *buf, bool is_clear)
{
  msg_grid_validate();
  int showcmd_row = Rows - 1;
  grid_line_start(&msg_grid_adj, showcmd_row);

  int len = 0;
  if (!is_clear) {
    len = grid_line_puts(sc_col, buf, -1, HL_ATTR(HLF_MSG));
  }

  // clear the rest of an old message by outputting up to SHOWCMD_COLS spaces
  grid_line_puts(sc_col + len, (char *)"          " + len, -1, HL_ATTR(HLF_MSG));

  grid_line_flush();
}

/// transchar(c) wrapper -- result is a static buffer valid until next call.
const char *nvim_transchar_wrapper(int c) { return transchar(c); }

/// utf_char2bytes(c, buf) -- writes UTF-8 encoding of c into buf, returns length.
int nvim_utf_char2bytes_wrapper(int c, char *buf) { return utf_char2bytes(c, buf); }

/// vim_isprintc(c) wrapper.
bool nvim_vim_isprintc_wrapper(int c) { return vim_isprintc(c); }

/// hasFolding for upward direction: hasFolding(curwin, lnum, out_lnum, NULL).
/// Returns true if there is a fold. Sets *out_lnum to the first line of the fold.
bool nvim_hasFolding_up(int lnum, int *out_lnum)
{
  linenr_T top = (linenr_T)lnum;
  bool r = hasFolding(curwin, top, &top, NULL);
  *out_lnum = (int)top;
  return r;
}

/// hasFolding for downward direction: hasFolding(curwin, lnum, NULL, out_lnum).
/// Returns true if there is a fold. Sets *out_lnum to the last line of the fold.
bool nvim_hasFolding_down(int lnum, int *out_lnum)
{
  linenr_T bot = (linenr_T)lnum;
  bool r = hasFolding(curwin, bot, NULL, &bot);
  *out_lnum = (int)bot;
  return r;
}

/// getvcols with p_sbr/w_p_sbr save-restore for block-Visual showcmd.
/// Saves p_sbr and curwin->w_p_sbr, sets them to empty, calls
/// getvcols(curwin, &w_cursor, &VIsual, out_left, out_right), then restores.
void nvim_getvcols_visual_sbr_save(int *out_left, int *out_right)
{
  char *const saved_sbr = p_sbr;
  char *const saved_w_sbr = curwin->w_p_sbr;
  p_sbr = empty_string_option;
  curwin->w_p_sbr = empty_string_option;
  colnr_T leftcol, rightcol;
  getvcols(curwin, &curwin->w_cursor, &VIsual, &leftcol, &rightcol);
  p_sbr = saved_sbr;
  curwin->w_p_sbr = saved_w_sbr;
  *out_left = (int)leftcol;
  *out_right = (int)rightcol;
}

// add_to_showcmd deleted: Rust showcmd.rs exports directly via #[export_name = "add_to_showcmd"].

void add_to_showcmd_c(int c) { add_to_showcmd(c); setcursor(); }

// del_from_showcmd deleted: Rust showcmd.rs exports directly via #[export_name = "del_from_showcmd"].

// push_showcmd deleted: Rust exports under the C name directly via #[export_name = "push_showcmd"].
// pop_showcmd deleted: Rust exports under the C name directly via #[export_name = "pop_showcmd"].

// =============================================================================
// Scrollbind C accessors for Rust FFI
// =============================================================================

/// Get did_syncbind global.
bool nvim_get_did_syncbind(void) { return did_syncbind; }

/// Set did_syncbind global.
void nvim_set_did_syncbind(bool val) { did_syncbind = val; }

/// Check curwin pointer equality with a saved handle.
bool nvim_curwin_eq(win_T *wp) { return curwin == wp; }

/// Check curwin->w_buffer pointer equality with a saved buffer handle.
bool nvim_curwin_buf_eq(buf_T *buf) { return curwin->w_buffer == buf; }

/// Get curwin->w_p_diff.
bool nvim_curwin_get_w_p_diff(void) { return curwin->w_p_diff; }

/// Get curwin->w_scbind_pos.
int nvim_curwin_get_w_scbind_pos(void) { return curwin->w_scbind_pos; }

/// Set curwin->w_scbind_pos.
void nvim_curwin_set_w_scbind_pos(int val) { curwin->w_scbind_pos = (linenr_T)val; }

/// Check if char c is in p_sbo (scrollopt) option string.
bool nvim_vim_strchr_p_sbo(int c) { return vim_strchr(p_sbo, c) != NULL; }

/// Compound: iterate all windows in current tab and sync scrollbind.
/// Handles curwin/curbuf swapping internally (unsafe to do in Rust).
/// old_curwin_arg: the original window before the loop.
/// vtopline_diff: vertical scroll diff.
/// tgt_leftcol: target left column for horizontal scroll.
/// want_ver: do vertical sync.
/// want_hor: do horizontal sync.
void nvim_scrollbind_sync_windows(win_T *old_curwin_arg, int vtopline_diff,
                                   int tgt_leftcol, bool want_ver, bool want_hor)
{
  win_T *old_curbuf_win = curwin;
  buf_T *old_curbuf_buf = curbuf;
  int old_VIsual_select = VIsual_select;
  int old_VIsual_active = VIsual_active;
  VIsual_select = VIsual_active = 0;

  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    curwin = wp;
    curbuf = curwin->w_buffer;
    if (curwin == old_curwin_arg || !curwin->w_p_scb) {
      continue;
    }

    if (want_ver) {
      if (old_curwin_arg->w_p_diff && curwin->w_p_diff) {
        rs_diff_set_topline(old_curwin_arg, curwin);
      } else {
        curwin->w_scbind_pos += vtopline_diff;
        int curr_vtopline = rs_get_vtopline(curwin);
        int max_vtopline = curr_vtopline + curwin->w_topfill
                           + plines_m_win_fill(curwin, curwin->w_topline + 1,
                                               curbuf->b_ml.ml_line_count);
        int new_vtopline = MAX(MIN((linenr_T)curwin->w_scbind_pos, max_vtopline), 1);
        int y = new_vtopline - curr_vtopline;
        if (y > 0) {
          scrollup(curwin, y, false);
        } else {
          scrolldown(curwin, -y, false);
        }
      }
      redraw_later(curwin, UPD_VALID);
      cursor_correct(curwin);
      curwin->w_redr_status = true;
    }

    if (want_hor) {
      set_leftcol((colnr_T)tgt_leftcol);
    }
  }

  VIsual_select = old_VIsual_select;
  VIsual_active = old_VIsual_active;
  curwin = old_curbuf_win;
  curbuf = old_curbuf_buf;
}

// do_check_scrollbind deleted: now exported directly from Rust via #[export_name]
// check_scrollbind deleted: now exported directly from Rust via #[export_name]

// find_decl deleted: Rust lib.rs exports directly via #[export_name = "find_decl"].
// nv_screengo deleted: dead code (no C or Rust callers; rs_nv_screengo called directly via move.c wrapper).

/// Initializes static oparg_T/cmdarg_T and returns cap pointer.
/// nvim is single-threaded so function-static storage is safe.
cmdarg_T *nvim_create_temp_cap_for_ident(int c1, int c2)
{
  static oparg_T oa;
  static cmdarg_T ca;
  clear_oparg(&oa);
  CLEAR_FIELD(ca);
  ca.oap = &oa;
  ca.cmdchar = c1;
  ca.nchar = c2;
  return &ca;
}

// do_nv_ident deleted: Rust exports under the C name directly via #[export_name = "do_nv_ident"].

// unadjust_for_sel_inner deleted: Rust lib.rs exports directly via #[export_name = "unadjust_for_sel_inner"].

// invoke_edit deleted: Rust lib.rs exports directly via #[export_name = "invoke_edit"].

void normal_cmd(oparg_T *oap, bool toplevel)
{
  NormalState s;
  normal_state_init(&s);
  s.toplevel = toplevel;
  s.oa = *oap;
  rs_normal_prepare(&s);
  normal_execute(&s.state, safe_vgetc());
  *oap = s.oa;
}

// nv_ident build accessors

/// Get the resolved keywordprg string (curbuf->b_p_kp or p_kp fallback).
char *nvim_ident_get_kp(void)
{
  return *curbuf->b_p_kp == NUL ? p_kp : curbuf->b_p_kp;
}

/// Return true if curbuf is a help buffer.
bool nvim_ident_curbuf_is_help(void) { return curbuf->b_help; }

/// Return curbuf's filetype string.
char *nvim_ident_get_curbuf_ft(void) { return curbuf->b_p_ft; }

/// Return get_cursor_line_ptr().
char *nvim_ident_get_cursor_line_ptr(void) { return get_cursor_line_ptr(); }

/// Return vim_iswordp(p).
bool nvim_ident_vim_iswordp(const char *p) { return vim_iswordp(p); }

/// Return mb_prevptr(line, p).
char *nvim_ident_mb_prevptr(char *line, char *p) { return mb_prevptr(line, p); }

/// Set g_tag_at_cursor.
void nvim_ident_set_g_tag_at_cursor(bool val) { g_tag_at_cursor = val; }


// =============================================================================
// Dispatch table handler accessors (nv_addsub, nv_colon, nv_record, nv_paste, nv_event)
// =============================================================================

/// Call op_addsub(oap, count1, arg).
void nvim_op_addsub_call(oparg_T *oap, int count1, int arg) { op_addsub(oap, count1, arg); }

/// Call do_record(nchar) and return FAIL/OK.
int nvim_do_record(int nchar) { return do_record(nchar); }

/// Return paste_repeat(count).
void nvim_paste_repeat(int count) { paste_repeat(count); }

/// Call state_handle_k_event().
void nvim_state_handle_k_event(void) { state_handle_k_event(); }

/// Call do_cmdline with appropriate function pointer for colon/cmdkey.
/// Returns false on failure (mirrors do_cmdline return).
bool nvim_do_cmdline_for_colon(cmdarg_T *cap, bool is_cmdkey) {
  return do_cmdline(NULL, is_cmdkey ? getcmdkeycmd : getexline, NULL,
                    cap->oap->op_type != OP_NOP ? DOCMD_KEEPLINE : 0);
}

/// Call map_execute_lua(true, false).
bool nvim_map_execute_lua_for_colon(void) { return map_execute_lua(true, false); }

/// Get cap->oap->start.lnum.
int nvim_get_oap_start_lnum(cmdarg_T *cap) { return (int)cap->oap->start.lnum; }

/// Get cap->oap->start.col.
int nvim_get_oap_start_col(cmdarg_T *cap) { return (int)cap->oap->start.col; }

/// Return did_emsg.
int nvim_did_emsg_check(void) { return did_emsg; }

/// Return the translated "E1292: Command-line window is already open" message.
const char *nvim_get_e_cmdline_window_already_open(void) { return _(e_cmdline_window_already_open); }

// =============================================================================
// Search, gotofile, visual text, and mark movement accessors for Rust FFI
// =============================================================================

/// Wrapper for do_search(). Sets searchit fields and returns sa_wrapped.
/// Returns the do_search() return value.
int nvim_do_search_call(oparg_T *oap, int dir, char *pat, size_t patlen,
                        int count1, int opt, int *wrapped)
{
  searchit_arg_T sia;
  CLEAR_FIELD(sia);
  int i = do_search(oap, dir, dir, pat, patlen, count1,
                    opt | SEARCH_OPT | SEARCH_ECHO | SEARCH_MSG, &sia);
  if (wrapped != NULL) {
    *wrapped = sia.sa_wrapped;
  }
  return i;
}

/// Returns true if cursor moved and highlights need refresh.
bool nvim_search_hls_needs_redraw(int prev_lnum, int prev_col, int prev_coladd)
{
  pos_T prev = { .lnum = prev_lnum, .col = (colnr_T)prev_col, .coladd = (colnr_T)prev_coladd };
  return !equalpos(curwin->w_cursor, prev) && p_hls && !no_hlsearch
         && win_hl_attr(curwin, HLF_LC) != win_hl_attr(curwin, HLF_L);
}

/// Wrapper for grab_file_name(count1, &lnum). Sets *lnum_out to lnum.
char *nvim_grab_file_name(int count1, int *lnum_out)
{
  linenr_T lnum = -1;
  char *result = grab_file_name(count1, &lnum);
  if (lnum_out != NULL) {
    *lnum_out = (int)lnum;
  }
  return result;
}

/// Check curbuf changed, b_nwindows, buf_hide.
bool nvim_curbuf_needs_autowrite(void) {
  return curbufIsChanged() && curbuf->b_nwindows <= 1 && !buf_hide(curbuf);
}

/// Call autowrite(curbuf, false).
void nvim_autowrite_curbuf(void) { autowrite(curbuf, false); }

/// Call check_can_set_curbuf_disabled().
bool nvim_check_can_set_curbuf_disabled(void) { return check_can_set_curbuf_disabled(); }

/// Call do_ecmd for gotofile. Returns OK/FAIL (1/0).
int nvim_do_ecmd_for_gotofile(char *ptr)
{
  return do_ecmd(0, ptr, NULL, NULL, ECMD_LAST,
                 buf_hide(curbuf) ? ECMD_HIDE : 0, curwin);
}

/// Call ml_get_pos(&VIsual).
char *nvim_ml_get_pos_visual(void) { return ml_get_pos(&VIsual); }

/// Call mark_move_to(fm, flags). Returns MarkMoveRes as int.
int nvim_mark_move_to_call(void *fm, int flags) { return (int)mark_move_to((fmark_T *)fm, (MarkMove)flags); }

// =============================================================================
// Visual mode, cursor adjustment, and ident accessors for Rust FFI
// =============================================================================

/// Call conceal_check_cursor_line().
void nvim_conceal_check_cursor_line(void) { conceal_check_cursor_line(); }

/// Set mouse_dragging to val.
void nvim_set_mouse_dragging(int val) { mouse_dragging = val; }

/// Call adjust_cursor_eol().
void nvim_adjust_cursor_eol(void) { adjust_cursor_eol(); }

/// Get get_op_char(optype).
int nvim_get_op_char(int optype) { return get_op_char(optype); }

/// Get get_extra_op_char(optype).
int nvim_get_extra_op_char(int optype) { return get_extra_op_char(optype); }

/// Set v:operator to opchars string of length len. If opchars is NULL, clear it.
void nvim_set_vim_var_string_vv_op(const char *opchars, int len)
{
  set_vim_var_string(VV_OP, opchars, len);
}

/// Call rs_find_ident_at_pos(curwin, cursor.lnum, cursor.col, text, NULL, find_type).
size_t rs_find_ident_under_cursor(char **text, int find_type)
{
  return rs_find_ident_at_pos(curwin, curwin->w_cursor.lnum,
                              curwin->w_cursor.col, text, NULL, find_type);
}

/// Coladvance wrapper: temporarily set State to MODE_INSERT for "A" command cursor positioning.
void nvim_coladvance_append_mode(void)
{
  const int save_State = State;
  State = MODE_INSERT;
  coladvance(curwin, MAXCOL);
  State = save_State;
}

/// Get length of cursor line suffix (strlen(get_cursor_pos_ptr())).
int nvim_get_cursor_pos_ptr_len(void) { return (int)strlen(get_cursor_pos_ptr()); }

/// Get curwin->w_redr_type.
int nvim_get_curwin_w_redr_type(void) { return curwin->w_redr_type; }

/// Set curwin->w_old_cursor_lnum and w_old_visual_lnum to cursor lnum.
void nvim_curwin_set_old_visual_lnums(void)
{
  curwin->w_old_cursor_lnum = curwin->w_cursor.lnum;
  curwin->w_old_visual_lnum = curwin->w_cursor.lnum;
}

/// Call redraw_curbuf_later(UPD_VALID).
void nvim_redraw_curbuf_later_valid(void) { redraw_curbuf_later(UPD_VALID); }

// =============================================================================
// nv_z and operator implementation accessors for Rust FFI
// =============================================================================

/// Return typebuf_was_empty global.
bool nvim_get_typebuf_was_empty(void) { return typebuf_was_empty; }

/// Call vim_beep(kOptBoFlagEsc).
void nvim_vim_beep_esc(void) { vim_beep(kOptBoFlagEsc); }

/// Return true if curbuf is a terminal buffer.
bool nvim_get_curbuf_terminal(void) { return curbuf->terminal != NULL; }

/// Call edit(cmd, startln, count) and return bool result.
bool nvim_edit_call(int cmd, bool startln, int count) { return edit(cmd, startln, count); }

/// Append digit to n (wraps vim_append_digit_int). Returns 0 (FAIL) or 1 (OK).
/// The updated n is written back through n_ptr.
int nvim_vim_append_digit_int(int *n_ptr, int digit)
{
  return vim_append_digit_int(n_ptr, digit);
}

/// Show "quit" or "abandon" hint message via msg() for ESC/CTRL-C.
void nvim_esc_show_msg(void)
{
  if (anyBufIsChanged()) {
    msg(_("Type  :qa!  and press <Enter> to abandon all changes"
          " and exit Nvim"), 0);
  } else {
    msg(_("Type  :qa  and press <Enter> to exit Nvim"), 0);
  }
}
