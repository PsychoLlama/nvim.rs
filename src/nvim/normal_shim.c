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

static int VIsual_mode_orig = NUL;              // saved Visual mode

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
  { Ctrl_A,    nv_addsub,      0,                      0 },
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
  { Ctrl_X,    nv_addsub,      0,                      0 },
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
  { ':',       nv_colon,       0,                      0 },
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
  { 'q',       nv_record,      NV_NCH,                 0 },
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
  { K_PASTE_START, nv_paste,   NV_KEEPREG,             0 },
  { K_EVENT,   nv_event,       NV_KEEPREG,             0 },
  { K_COMMAND, nv_colon,       0,                      0 },
  { K_LUA, nv_colon,           0,                      0 },
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

// Normal mode state machine
extern int rs_normal_check(void *s);
extern int rs_normal_execute(void *s, int key);
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

/// Get the command character at index in nv_cmds.
int nvim_get_nv_cmd_char(int idx)
{
  if (idx < 0 || (size_t)idx >= NV_CMDS_SIZE) {
    return 0;
  }
  return nv_cmds[idx].cmd_char;
}

int nvim_get_nv_cmds_size(void) { return (int)NV_CMDS_SIZE; }

/// Get the nv_cmd_idx value at position.
int16_t nvim_get_nv_cmd_idx(int idx)
{
  if (idx < 0 || (size_t)idx >= NV_CMDS_SIZE) {
    return 0;
  }
  return nv_cmd_idx[idx];
}

/// Get the command flags at index in nv_cmds.
unsigned int nvim_get_nv_cmd_flags(int idx)
{
  if (idx < 0 || (size_t)idx >= NV_CMDS_SIZE) {
    return 0;
  }
  return nv_cmds[idx].cmd_flags;
}

/// Get the command arg at index in nv_cmds.
int nvim_get_nv_cmd_arg(int idx)
{
  if (idx < 0 || (size_t)idx >= NV_CMDS_SIZE) {
    return 0;
  }
  return nv_cmds[idx].cmd_arg;
}

// =============================================================================
// oparg_T pointer accessors for Rust FFI (takes explicit oap parameter)
// =============================================================================

int nvim_oap_get_op_type_ptr(oparg_T *oap) { return oap ? oap->op_type : OP_NOP; }

/// Set oap->op_type.
void nvim_oap_set_op_type(oparg_T *oap, int val)
{
  if (oap) {
    oap->op_type = val;
  }
}

int nvim_oap_get_regname_ptr(oparg_T *oap) { return oap ? oap->regname : NUL; }

/// Set oap->regname.
void nvim_oap_set_regname(oparg_T *oap, int val)
{
  if (oap) {
    oap->regname = val;
  }
}

int nvim_oap_get_motion_force(oparg_T *oap) { return oap ? oap->motion_force : NUL; }

/// Set oap->motion_force.
void nvim_oap_set_motion_force(oparg_T *oap, int val)
{
  if (oap) {
    oap->motion_force = val;
  }
}

/// Set oap->use_reg_one.
void nvim_oap_set_use_reg_one(oparg_T *oap, bool val)
{
  if (oap) {
    oap->use_reg_one = val;
  }
}

int nvim_oap_get_motion_type(oparg_T *oap) { return oap ? oap->motion_type : kMTUnknown; }

/// Set oap->motion_type.
void nvim_oap_set_motion_type(oparg_T *oap, int val)
{
  if (oap) {
    oap->motion_type = val;
  }
}

bool nvim_oap_get_inclusive(oparg_T *oap) { return oap ? oap->inclusive : false; }

/// Set oap->inclusive.
void nvim_oap_set_inclusive(oparg_T *oap, bool val)
{
  if (oap) {
    oap->inclusive = val;
  }
}

// =============================================================================
// Additional oparg_T accessors for Rust ops crate
// =============================================================================

int nvim_oap_get_op_type_raw(oparg_T *oap) { return oap ? oap->op_type : OP_NOP; }

int nvim_oap_get_regname_raw(oparg_T *oap) { return oap ? oap->regname : NUL; }

int nvim_oap_get_motion_type_raw(oparg_T *oap) { return oap ? oap->motion_type : kMTUnknown; }

int nvim_oap_get_use_reg_one(oparg_T *oap) { return oap ? oap->use_reg_one : false; }

int nvim_oap_get_line_count(oparg_T *oap) { return oap ? oap->line_count : 0; }

/// Set oap->line_count.
void nvim_oap_set_line_count(oparg_T *oap, int val)
{
  if (oap) {
    oap->line_count = val;
  }
}

int nvim_oap_get_empty(oparg_T *oap) { return oap ? oap->empty : false; }

/// Set oap->empty.
void nvim_oap_set_empty(oparg_T *oap, int val)
{
  if (oap) {
    oap->empty = val != 0;
  }
}

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

/// Set cap->retval.
void nvim_cap_set_retval(cmdarg_T *cap, int val)
{
  if (cap) {
    cap->retval = val;
  }
}

/// OR val into cap->retval.
void nvim_cap_or_retval(cmdarg_T *cap, int val)
{
  if (cap) {
    cap->retval |= val;
  }
}

int nvim_cap_get_cmdchar(cmdarg_T *cap) { return cap ? cap->cmdchar : 0; }

/// Set cap->cmdchar.
void nvim_cap_set_cmdchar(cmdarg_T *cap, int val)
{
  if (cap) {
    cap->cmdchar = val;
  }
}

int nvim_cap_get_nchar(cmdarg_T *cap) { return cap ? cap->nchar : 0; }

/// Set cap->nchar.
void nvim_cap_set_nchar(cmdarg_T *cap, int val)
{
  if (cap) {
    cap->nchar = val;
  }
}

int nvim_cap_get_extra_char(cmdarg_T *cap) { return cap ? cap->extra_char : 0; }

/// Set cap->extra_char.
void nvim_cap_set_extra_char(cmdarg_T *cap, int val)
{
  if (cap) {
    cap->extra_char = val;
  }
}

int nvim_cap_get_count0(cmdarg_T *cap) { return cap ? cap->count0 : 0; }

/// Set cap->count0.
void nvim_cap_set_count0(cmdarg_T *cap, int val)
{
  if (cap) {
    cap->count0 = val;
  }
}

int nvim_cap_get_count1(cmdarg_T *cap) { return cap ? cap->count1 : 0; }

/// Set cap->count1.
void nvim_cap_set_count1(cmdarg_T *cap, int val)
{
  if (cap) {
    cap->count1 = val;
  }
}

int nvim_cap_get_opcount(cmdarg_T *cap) { return cap ? cap->opcount : 0; }

/// Set cap->opcount.
void nvim_cap_set_opcount(cmdarg_T *cap, int val)
{
  if (cap) {
    cap->opcount = val;
  }
}

int nvim_cap_get_arg(cmdarg_T *cap) { return cap ? cap->arg : 0; }

/// Set cap->arg.
void nvim_cap_set_arg(cmdarg_T *cap, int val)
{
  if (cap) {
    cap->arg = val;
  }
}

int nvim_cap_get_prechar(cmdarg_T *cap) { return cap ? cap->prechar : 0; }

/// Set cap->prechar.
void nvim_cap_set_prechar(cmdarg_T *cap, int val)
{
  if (cap) {
    cap->prechar = val;
  }
}

// =============================================================================
// Word motion accessors for Rust FFI
// =============================================================================

void nvim_curwin_set_set_curswant(bool val) { curwin->w_set_curswant = val; }

int nvim_fwd_word(int count, bool bigword, bool eol) { return fwd_word(count, bigword, eol); }

int nvim_bck_word(int count, bool bigword, bool stop) { return bck_word(count, bigword, stop); }

int nvim_end_word(int count, bool bigword, bool stop, bool empty) { return end_word(count, bigword, stop, empty); }

int nvim_bckend_word(int count, bool bigword, bool eol) { return bckend_word(count, bigword, eol); }

int nvim_findsent(int dir, int count) { return findsent(dir, count); }

bool nvim_findpar(bool *pincl, int dir, int count, int what, bool both) { return findpar(pincl, dir, count, what, both); }

int nvim_get_cursor_col(void) { return curwin->w_cursor.col; }

void nvim_set_cursor_col(int col) { curwin->w_cursor.col = col; }

void nvim_set_cursor_coladd_zero(void) { curwin->w_cursor.coladd = 0; }

int nvim_gchar_cursor_call(void) { return gchar_cursor(); }

int nvim_inc_cursor(void) { return inc(&curwin->w_cursor); }

int nvim_dec_cursor(void) { return dec(&curwin->w_cursor); }

bool nvim_cpo_has_changew(void) { return vim_strchr(p_cpo, CPO_CHANGEW) != NULL; }

bool nvim_ascii_iswhite(int c) { return ascii_iswhite(c); }

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

bool nvim_is_special(int key) { return IS_SPECIAL(key); }

void nvim_getvcol_cursor(int *scol, int *ecol) { getvcol(curwin, &curwin->w_cursor, scol, NULL, ecol); }

void nvim_set_cursor_coladd(int val) { curwin->w_cursor.coladd = val; }

int nvim_get_TAB(void) { return TAB; }

// =============================================================================
// Mark command accessors for Rust FFI
// =============================================================================

bool nvim_setmark(int name) { return setmark(name); }

int nvim_cap_get_nchar_call(cmdarg_T *cap) { return cap ? cap->nchar : 0; }

int nvim_cap_get_extra_char_call(cmdarg_T *cap) { return cap ? cap->extra_char : 0; }

unsigned int nvim_get_jop_flags(void) { return jop_flags; }

fmark_T *nvim_mark_get(int name) { return mark_get(curbuf, curwin, NULL, kMarkAll, name); }

int nvim_nv_mark_move_to(cmdarg_T *cap, int flags, fmark_T *fm) { return nv_mark_move_to(cap, flags, fm); }

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

void nvim_nv_put_opt(cmdarg_T *cap, bool fix_indent) { nv_put_opt(cap, fix_indent); }

// =============================================================================
// Visual mode accessors for Rust FFI
// =============================================================================

int nvim_get_Ctrl_Q(void) { return Ctrl_Q; }

int nvim_get_Ctrl_V(void) { return Ctrl_V; }

/// Set cap->cmdchar.
void nvim_cap_set_cmdchar_call(cmdarg_T *cap, int val)
{
  if (cap) {
    cap->cmdchar = val;
  }
}

int nvim_get_motion_force(void) { return motion_force; }

void nvim_set_finish_op(bool val) { finish_op = val; }

void nvim_end_visual_mode(void) { end_visual_mode(); }

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

int nvim_get_MAXCOL(void) { return MAXCOL; }

void nvim_n_start_visual_mode(int cmdchar) { n_start_visual_mode(cmdchar); }

/// Cap count1 decrement and access.
int nvim_cap_dec_count1(cmdarg_T *cap)
{
  if (cap) {
    return --cap->count1;
  }
  return 0;
}

/// Internal nv_visual implementation (full C logic).
static void nv_visual_impl(cmdarg_T *cap);

void nvim_nv_visual_impl(cmdarg_T *cap) { nv_visual_impl(cap); }

// =============================================================================
// Command handler accessors for Rust FFI
// =============================================================================

/// Clear all syntax states and redraw for nv_clear.
void nvim_nv_clear_impl(void)
{
  syn_stack_free_all(curwin->w_s);
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    wp->w_s->b_syn_slow = false;
  }
  redraw_later(curwin, UPD_CLEAR);
}

int nvim_get_restart_VIsual_select(void) { return restart_VIsual_select; }

void nvim_set_restart_VIsual_select(int val) { restart_VIsual_select = val; }

void nvim_buflist_getfile(int n, int lnum, int flags, bool setpm) { buflist_getfile(n, lnum, flags, setpm); }

int nvim_get_GETF_SETMARK(void) { return GETF_SETMARK; }

int nvim_get_GETF_ALT(void) { return GETF_ALT; }

// =============================================================================
// Visual mode accessors for Rust FFI
// =============================================================================

int nvim_get_VIsual_mode_orig(void) { return VIsual_mode_orig; }

void nvim_set_VIsual_mode_orig(int val) { VIsual_mode_orig = val; }

int nvim_get_curbuf_visual_vi_mode(void) { return curbuf->b_visual.vi_mode; }

void nvim_set_curbuf_visual_vi_mode(int val) { curbuf->b_visual.vi_mode = val; }

bool nvim_get_mode_displayed(void) { return mode_displayed; }

void nvim_set_clear_cmdline(bool val) { clear_cmdline = val; }

// =============================================================================
// Redo/count accessors for Rust FFI
// =============================================================================

int nvim_cap_get_nchar_len(cmdarg_T *cap) { return cap ? cap->nchar_len : 0; }

/// Append cap->nchar_composing to the redo buffer.
void nvim_cap_append_nchar_composing_to_redobuff(cmdarg_T *cap)
{
  if (cap) {
    AppendToRedobuff(cap->nchar_composing);
  }
}

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
_Static_assert(BL_SOL == 2, "BL_SOL mismatch");
_Static_assert(BL_FIX == 4, "BL_FIX mismatch");
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

/// Set curwin->w_cursor from curbuf->b_last_insert.mark.
/// Returns true if b_last_insert.mark.lnum != 0.
bool nvim_set_cursor_from_last_insert(void)
{
  if (curbuf->b_last_insert.mark.lnum != 0) {
    curwin->w_cursor = curbuf->b_last_insert.mark;
    return true;
  }
  return false;
}

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

void nvim_update_topline_call(void) { update_topline(curwin); }

bool nvim_p_sel_is_exclusive(void) { return *p_sel == 'e'; }

bool nvim_equalpos_VIsual_cursor(void) { return equalpos(VIsual, curwin->w_cursor); }

bool nvim_get_w_set_curswant(void) { return curwin->w_set_curswant; }

void nvim_set_w_set_curswant(bool val) { curwin->w_set_curswant = val; }

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

void nvim_coladvance_call(int col) { coladvance(curwin, col); }

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

bool nvim_unadjust_for_sel_inner_cursor(void) { return unadjust_for_sel_inner(&curwin->w_cursor); }

bool nvim_unadjust_for_sel_inner_visual(void) { return unadjust_for_sel_inner(&VIsual); }

int nvim_mark_mb_adjustpos_cursor(void) { mark_mb_adjustpos(curbuf, &curwin->w_cursor); return curwin->w_cursor.col; }

int nvim_mark_mb_adjustpos_visual(void) { mark_mb_adjustpos(curbuf, &VIsual); return VIsual.col; }

/// Wrapper for getvcol on a position, returns ce (end virtual column).
int nvim_getvcol_ce(int lnum, int col, int coladd)
{
  pos_T pp = { lnum, col, coladd };
  colnr_T cs, ce;
  getvcol(curwin, &pp, &cs, NULL, &ce);
  return ce - cs;
}

int nvim_ml_get_len_call(int lnum) { return (int)ml_get_len(lnum); }

/// Wrapper for nv_Zet C implementation.
void nvim_nv_Zet_impl(cmdarg_T *cap)
{
  if (rs_checkclearopq(cap->oap)) {
    return;
  }

  switch (cap->nchar) {
  // "ZZ": equivalent to ":x".
  case 'Z':
    do_cmdline_cmd("x");
    break;

  // "ZQ": equivalent to ":q!" (Elvis compatible).
  case 'Q':
    do_cmdline_cmd("q!");
    break;

  default:
    rs_clearopbeep(cap->oap);
  }
}

/// Wrapper for nv_esc C implementation.
void nvim_nv_esc_impl(cmdarg_T *cap)
{
  bool no_reason = (cap->oap->op_type == OP_NOP
                    && cap->opcount == 0
                    && cap->count0 == 0
                    && cap->oap->regname == 0);

  if (cap->arg) {               // true for CTRL-C
    if (restart_edit == 0 && cmdwin_type == 0 && !VIsual_active && no_reason) {
      if (anyBufIsChanged()) {
        msg(_("Type  :qa!  and press <Enter> to abandon all changes"
              " and exit Nvim"), 0);
      } else {
        msg(_("Type  :qa  and press <Enter> to exit Nvim"), 0);
      }
    }

    if (restart_edit != 0) {
      redraw_mode = true;  // remove "-- (insert) --"
    }

    restart_edit = 0;

    if (cmdwin_type != 0) {
      cmdwin_result = K_IGNORE;
      got_int = false;          // don't stop executing autocommands et al.
      return;
    }
  } else if (cmdwin_type != 0 && ex_normal_busy && typebuf_was_empty) {
    // When :normal runs out of characters while in the command line window
    // vgetorpeek() will repeatedly return ESC.  Exit the cmdline window to
    // break the loop.
    cmdwin_result = K_IGNORE;
    return;
  }

  if (VIsual_active) {
    end_visual_mode();          // stop Visual
    check_cursor_col(curwin);         // make sure cursor is not beyond EOL
    curwin->w_set_curswant = true;
    redraw_curbuf_later(UPD_INVERTED);
  } else if (no_reason) {
    vim_beep(kOptBoFlagEsc);
  }
  rs_clearop(cap->oap);
}

/// Wrapper for nv_edit C implementation.
void nvim_nv_edit_impl(cmdarg_T *cap)
{
  // <Insert> is equal to "i"
  if (cap->cmdchar == K_INS || cap->cmdchar == K_KINS) {
    cap->cmdchar = 'i';
  }

  // in Visual mode "A" and "I" are an operator
  if (VIsual_active && (cap->cmdchar == 'A' || cap->cmdchar == 'I')) {
    rs_v_visop(cap);
    // in Visual mode and after an operator "a" and "i" are for text objects
  } else if ((cap->cmdchar == 'a' || cap->cmdchar == 'i')
             && (cap->oap->op_type != OP_NOP || VIsual_active)) {
    rs_nv_object(cap);
  } else if (!curbuf->b_p_ma && !curbuf->terminal) {
    emsg(_(e_modifiable));
    rs_clearop(cap->oap);
  } else if (!rs_checkclearopq(cap->oap)) {
    switch (cap->cmdchar) {
    case 'A':           // "A"ppend after the line
      set_cursor_for_append_to_line();
      break;

    case 'I':           // "I"nsert before the first non-blank
      beginline(BL_WHITE);
      break;

    case 'a':           // "a"ppend is like "i"nsert on the next character.
      // increment coladd when in virtual space, increment the
      // column otherwise, also to append after an unprintable char
      if (virtual_active(curwin)
          && (curwin->w_cursor.coladd > 0
              || *get_cursor_pos_ptr() == NUL
              || *get_cursor_pos_ptr() == TAB)) {
        curwin->w_cursor.coladd++;
      } else if (*get_cursor_pos_ptr() != NUL) {
        inc_cursor();
      }
      break;
    }

    if (curwin->w_cursor.coladd && cap->cmdchar != 'A') {
      int save_State = State;

      // Pretend Insert mode here to allow the cursor on the
      // character past the end of the line
      State = MODE_INSERT;
      coladvance(curwin, getviscol());
      State = save_State;
    }

    invoke_edit(cap, false, cap->cmdchar, false);
  }
}

// =============================================================================
// Search handler accessors for Rust FFI
// =============================================================================

// Forward declarations for search handlers
static void nv_search_impl(cmdarg_T *cap);
static void nv_next_impl(cmdarg_T *cap);
static void nv_ident_impl(cmdarg_T *cap);

void nvim_nv_search_impl(cmdarg_T *cap) { nv_search_impl(cap); }

void nvim_nv_next_impl(cmdarg_T *cap) { nv_next_impl(cap); }

void nvim_nv_ident_impl(cmdarg_T *cap) { nv_ident_impl(cap); }

/// Wrapper for nv_gd C implementation (go to definition).
void nvim_nv_gd_impl(oparg_T *oap, int nchar, int thisblock)
{
  size_t len;
  char *ptr;
  if ((len = find_ident_under_cursor(&ptr, FIND_IDENT)) == 0
      || !find_decl(ptr, len, nchar == 'd', thisblock, SEARCH_START)) {
    rs_clearopbeep(oap);
    return;
  }

  if ((fdo_flags & kOptFdoFlagSearch) && KeyTyped && oap->op_type == OP_NOP) {
    rs_foldOpenCursor();
  }
  // clear any search statistics
  if (messaging() && !msg_silent && !shortmess(SHM_SEARCHCOUNT)) {
    clear_cmdline = true;
  }
}

// =============================================================================
// Operator handler accessors for Rust FFI
// =============================================================================

// Forward declarations for operator handlers
static void nv_operator_impl(cmdarg_T *cap);
static void nv_optrans_impl(cmdarg_T *cap);
static void nv_tilde_impl(cmdarg_T *cap);
static void nv_subst_impl(cmdarg_T *cap);

void nvim_nv_operator_impl(cmdarg_T *cap) { nv_operator_impl(cap); }

void nvim_nv_optrans_impl(cmdarg_T *cap) { nv_optrans_impl(cap); }

void nvim_nv_tilde_impl(cmdarg_T *cap) { nv_tilde_impl(cap); }

void nvim_nv_subst_impl(cmdarg_T *cap) { nv_subst_impl(cap); }

// =============================================================================
// Text object handler accessors for Rust FFI
// =============================================================================

// Forward declarations for text object handlers
static void nv_object_impl(cmdarg_T *cap);
static void nv_select_impl(cmdarg_T *cap);
static void nv_brackets_impl(cmdarg_T *cap);

void nvim_nv_object_impl(cmdarg_T *cap) { nv_object_impl(cap); }

void nvim_nv_select_impl(cmdarg_T *cap) { nv_select_impl(cap); }

void nvim_nv_brackets_impl(cmdarg_T *cap) { nv_brackets_impl(cap); }

// =============================================================================
// Undo/Redo handler accessors for Rust FFI
// =============================================================================

// Forward declarations for undo/redo handlers
static void nv_undo_impl(cmdarg_T *cap);
static void nv_Undo_impl(cmdarg_T *cap);
static void nv_dot_impl(cmdarg_T *cap);
static void nv_redo_or_register_impl(cmdarg_T *cap);

void nvim_nv_undo_impl(cmdarg_T *cap) { nv_undo_impl(cap); }

void nvim_nv_Undo_impl(cmdarg_T *cap) { nv_Undo_impl(cap); }

void nvim_nv_dot_impl(cmdarg_T *cap) { nv_dot_impl(cap); }

void nvim_nv_redo_or_register_impl(cmdarg_T *cap) { nv_redo_or_register_impl(cap); }

// =============================================================================
// Insert mode entry handler accessors for Rust FFI
// =============================================================================

// Forward declarations for insert mode entry handlers
static void nv_replace_impl(cmdarg_T *cap);
static void nv_Replace_impl(cmdarg_T *cap);
static void nv_vreplace_impl(cmdarg_T *cap);

void nvim_nv_replace_impl(cmdarg_T *cap) { nv_replace_impl(cap); }

void nvim_nv_Replace_impl(cmdarg_T *cap) { nv_Replace_impl(cap); }

void nvim_nv_vreplace_impl(cmdarg_T *cap) { nv_vreplace_impl(cap); }

// =============================================================================
// Scroll and screen handler accessors for Rust FFI
// =============================================================================

// Forward declarations for scroll/screen handlers
static void nv_zet_impl(cmdarg_T *cap);
static void nv_scroll_impl(cmdarg_T *cap);
static void nv_right_impl(cmdarg_T *cap);
static void nv_left_impl(cmdarg_T *cap);
static void nv_up_impl(cmdarg_T *cap);
static void nv_down_impl(cmdarg_T *cap);

void nvim_nv_zet_impl(cmdarg_T *cap) { nv_zet_impl(cap); }

void nvim_nv_scroll_impl(cmdarg_T *cap) { nv_scroll_impl(cap); }

void nvim_nv_right_impl(cmdarg_T *cap) { nv_right_impl(cap); }

void nvim_nv_left_impl(cmdarg_T *cap) { nv_left_impl(cap); }

void nvim_nv_up_impl(cmdarg_T *cap) { nv_up_impl(cap); }

void nvim_nv_down_impl(cmdarg_T *cap) { nv_down_impl(cap); }

// =============================================================================
// Miscellaneous handler accessors for Rust FFI
// =============================================================================

// Forward declarations for miscellaneous handlers
static void nv_g_cmd_impl(cmdarg_T *cap);
static void nv_at_impl(cmdarg_T *cap);
static void nv_join_impl(cmdarg_T *cap);
static void nv_open_impl(cmdarg_T *cap);

void nvim_nv_g_cmd_impl(cmdarg_T *cap) { nv_g_cmd_impl(cap); }

void nvim_nv_at_impl(cmdarg_T *cap) { nv_at_impl(cap); }

void nvim_nv_join_impl(cmdarg_T *cap) { nv_join_impl(cap); }

void nvim_nv_open_impl(cmdarg_T *cap) { nv_open_impl(cap); }

// =============================================================================
// Window command accessors for Rust FFI
// =============================================================================

void nvim_nv_colon(cmdarg_T *cap) { nv_colon(cap); }

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
void nvim_ns_set_old_pos(void *s) { NS(s)->old_pos = curwin->w_cursor; }

void nvim_ns_save_opcount(void *s) { NS(s)->oa.prev_opcount = NS(s)->ca.opcount; }

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

static void normal_prepare(NormalState *s)
{
  CLEAR_FIELD(s->ca);  // also resets s->ca.retval
  s->ca.oap = &s->oa;

  // Use a count remembered from before entering an operator. After typing "3d"
  // we return from normal_cmd() and come back here, the "3" is remembered in
  // "opcount".
  s->ca.opcount = opcount;

  // If there is an operator pending, then the command we take this time will
  // terminate it. Finish_op tells us to finish the operation before returning
  // this time (unless the operation was cancelled).
  int c = finish_op;
  finish_op = (s->oa.op_type != OP_NOP);
  if (finish_op != c) {
    ui_cursor_shape();  // may show different cursor shape
  }
  may_trigger_modechanged();

  s->set_prevcount = false;
  // When not finishing an operator and no register name typed, reset the count.
  if (!finish_op && !s->oa.regname) {
    s->ca.opcount = 0;
    s->set_prevcount = true;
  }

  // Restore counts from before receiving K_EVENT.  This means after
  // typing "3", handling K_EVENT and then typing "2" we get "32", not
  // "3 * 2".
  if (s->oa.prev_opcount > 0 || s->oa.prev_count0 > 0) {
    s->ca.opcount = s->oa.prev_opcount;
    s->ca.count0 = s->oa.prev_count0;
    s->oa.prev_opcount = 0;
    s->oa.prev_count0 = 0;
  }

  s->mapped_len = typebuf_maplen();
  State = MODE_NORMAL_BUSY;

  // Set v:count here, when called from main() and not a stuffed command, so
  // that v:count can be used in an expression mapping when there is no count.
  // Do set it for redo
  if (s->toplevel && readbuf1_empty()) {
    rs_set_vcount_ca(&s->ca, &s->set_prevcount);
  }
}

static bool normal_handle_special_visual_command(NormalState *s)
{
  // when 'keymodel' contains "stopsel" may stop Select/Visual mode
  if (km_stopsel
      && (nv_cmds[s->idx].cmd_flags & NV_STS)
      && !(mod_mask & MOD_MASK_SHIFT)) {
    end_visual_mode();
    redraw_curbuf_later(UPD_INVERTED);
  }

  // Keys that work different when 'keymodel' contains "startsel"
  if (km_startsel) {
    if (nv_cmds[s->idx].cmd_flags & NV_SS) {
      s->ca.cmdchar = rs_unshift_special(s->ca.cmdchar, &mod_mask);
      s->idx = rs_find_command(s->ca.cmdchar);
      if (s->idx < 0) {
        // Just in case
        rs_clearopbeep(&s->oa);
        return true;
      }
    } else if ((nv_cmds[s->idx].cmd_flags & NV_SSS)
               && (mod_mask & MOD_MASK_SHIFT)) {
      mod_mask &= ~MOD_MASK_SHIFT;
    }
  }
  return false;
}

static bool normal_need_additional_char(NormalState *s) { bool pending_op = s->oa.op_type != OP_NOP; return rs_need_additional_char(s->idx, s->ca.cmdchar, pending_op); }

static bool normal_need_redraw_mode_message(NormalState *s)
{
  // In Visual mode and with "^O" in Insert mode, a short message will be
  // overwritten by the mode message.  Wait a bit, until a key is hit.
  // In Visual mode, it's more important to keep the Visual area updated
  // than keeping a message (e.g. from a /pat search).
  // Only do this if the command was typed, not from a mapping.
  // Don't wait when emsg_silent is non-zero.
  // Also wait a bit after an error message, e.g. for "^O:".
  // Don't redraw the screen, it would remove the message.
  return (
          // 'showmode' is set and messages can be printed
          ((p_smd && msg_silent == 0
            // must restart insert mode (ctrl+o or ctrl+l) or just entered visual mode
            && (restart_edit != 0 || (VIsual_active
                                      && s->old_pos.lnum == curwin->w_cursor.lnum
                                      && s->old_pos.col == curwin->w_cursor.col))
            // command-line must be cleared or redrawn
            && (clear_cmdline || redraw_cmdline)
            // some message was printed or scrolled
            && (msg_didout || (msg_didany && msg_scroll))
            // it is fine to remove the current message
            && !msg_nowait
            // the command was the result of direct user input and not a mapping
            && KeyTyped)
           // must restart insert mode, not in visual mode and error message is
           // being shown
           || (restart_edit != 0 && !VIsual_active && msg_scroll
               && emsg_on_display))
          // no register was used
          && s->oa.regname == 0
          && !(s->ca.retval & CA_COMMAND_BUSY)
          && stuff_empty()
          && typebuf_typed()
          && emsg_silent == 0
          && !in_assert_fails
          && !did_wait_return
          && s->oa.op_type == OP_NOP);
}

static void normal_redraw_mode_message(NormalState *s)
{
  int save_State = State;

  // Draw the cursor with the right shape here
  if (restart_edit != 0) {
    State = MODE_INSERT;
  }

  // If need to redraw, and there is a "keep_msg", redraw before the
  // delay
  if (must_redraw && keep_msg != NULL && !emsg_on_display) {
    char *kmsg;

    kmsg = keep_msg;
    keep_msg = NULL;
    // Showmode() will clear keep_msg, but we want to use it anyway.
    // First update w_topline.
    setcursor();
    update_screen();
    // now reset it, otherwise it's put in the history again
    keep_msg = kmsg;

    kmsg = xstrdup(keep_msg);
    msg(kmsg, keep_msg_hl_id);
    xfree(kmsg);
  }
  setcursor();
  ui_cursor_shape();                  // show different cursor shape
  ui_flush();
  if (!ui_has(kUIMessages) && (msg_scroll || emsg_on_display)) {
    os_delay(1003, true);            // wait at least one second
  }
  if (ui_has(kUIMessages)) {
    os_delay(3003, false);           // wait up to three seconds
  }
  State = save_State;

  msg_scroll = false;
  emsg_on_display = false;
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

/// Handle the composing character loop in normal_get_additional_char.
/// This is the "lang" section that deals with multi-byte composing chars.
/// Takes NormalState* (as void*).
void nvim_normal_handle_composing_chars(void *sp)
{
  NormalState *s = (NormalState *)sp;
  no_mapping--;
  GraphemeState state = GRAPHEME_STATE_INIT;
  int prev_code = s->ca.nchar;

  while ((s->c = vpeekc()) > 0
         && (s->c >= 0x100 || MB_BYTE2LEN(vpeekc()) > 1)) {
    s->c = plain_vgetc();

    if (!utf_iscomposing(prev_code, s->c, &state)) {
      vungetc(s->c);
      break;
    }

    if (s->ca.nchar_len == 0) {
      s->ca.nchar_len = utf_char2bytes(s->ca.nchar, s->ca.nchar_composing);
    }

    if (s->ca.nchar_len + utf_char2len(s->c) < (int)sizeof(s->ca.nchar_composing)) {
      s->ca.nchar_len += utf_char2bytes(s->c, s->ca.nchar_composing + s->ca.nchar_len);
    }
    prev_code = s->c;
  }
  s->ca.nchar_composing[s->ca.nchar_len] = NUL;
  no_mapping++;
  no_u_sync++;
  gotchars_ignore();
  no_u_sync--;
}

static void normal_invert_horizontal(NormalState *s) { s->ca.cmdchar = rs_invert_horizontal(s->ca.cmdchar); s->idx = rs_find_command(s->ca.cmdchar); }

static bool normal_get_command_count(NormalState *s)
{
  if (VIsual_active && VIsual_select) {
    return false;
  }
  // Handle a count before a command and compute ca.count0.
  // Note that '0' is a command and not the start of a count, but it's
  // part of a count after other digits.
  while ((s->c >= '1' && s->c <= '9')
         || (s->ca.count0 != 0 && (s->c == K_DEL || s->c == K_KDEL || s->c == '0'))) {
    if (s->c == K_DEL || s->c == K_KDEL) {
      s->ca.count0 /= 10;
      del_from_showcmd(4);            // delete the digit and ~@%
    } else if (s->ca.count0 > 99999999) {
      s->ca.count0 = 999999999;
    } else {
      s->ca.count0 = s->ca.count0 * 10 + (s->c - '0');
    }

    // Set v:count here, when called from main() and not a stuffed
    // command, so that v:count can be used in an expression mapping
    // right after the count. Do set it for redo.
    if (s->toplevel && readbuf1_empty()) {
      rs_set_vcount_ca(&s->ca, &s->set_prevcount);
    }

    if (s->ctrl_w) {
      no_mapping++;
      allow_keys++;                   // no mapping for nchar, but keys
    }

    no_zero_mapping++;                // don't map zero here
    s->c = plain_vgetc();
    LANGMAP_ADJUST(s->c, true);
    no_zero_mapping--;
    if (s->ctrl_w) {
      no_mapping--;
      allow_keys--;
    }
    s->need_flushbuf |= add_to_showcmd(s->c);
  }

  // If we got CTRL-W there may be a/another count
  if (s->c == Ctrl_W && !s->ctrl_w && s->oa.op_type == OP_NOP) {
    s->ctrl_w = true;
    s->ca.opcount = s->ca.count0;           // remember first count
    s->ca.count0 = 0;
    no_mapping++;
    allow_keys++;                        // no mapping for nchar, but keys
    s->c = plain_vgetc();                // get next character
    LANGMAP_ADJUST(s->c, true);
    no_mapping--;
    allow_keys--;
    s->need_flushbuf |= add_to_showcmd(s->c);
    return true;
  }

  return false;
}

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

bool nvim_normal_need_redraw_mode_message_wrapper(void *sp) { return normal_need_redraw_mode_message((NormalState *)sp); }

void nvim_normal_redraw_mode_message_wrapper(void *sp) { normal_redraw_mode_message((NormalState *)sp); }

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

/// showmode() wrapper.
void nvim_showmode_wrapper(void) { showmode(); }

// =============================================================================
// normal_execute accessors for Rust FFI
// =============================================================================

_Static_assert(K_IGNORE == -13821, "K_IGNORE changed");
_Static_assert(K_MOUSEMOVE == -25853, "K_MOUSEMOVE changed");
_Static_assert(K_EVENT == -26365, "K_EVENT changed");
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

/// Get curwin->w_curswant.
int nvim_get_curwin_w_curswant(void) { return curwin->w_curswant; }

/// Get vgetc_char global.
int nvim_get_vgetc_char(void) { return vgetc_char; }

/// Get vgetc_mod_mask global.
int nvim_get_vgetc_mod_mask(void) { return vgetc_mod_mask; }

/// Get km_startsel global.
bool nvim_get_km_startsel(void) { return km_startsel; }

/// Get curwin->w_p_rl.
bool nvim_get_curwin_w_p_rl(void) { return curwin->w_p_rl; }

/// Set oa->prev_opcount via oap handle.
void nvim_oap_set_prev_opcount(oparg_T *oap, int val) { oap->prev_opcount = val; }

/// Set oa->prev_count0 via oap handle.
void nvim_oap_set_prev_count0(oparg_T *oap, int val) { oap->prev_count0 = val; }

void nvim_normal_get_command_count_loop(void *sp) { while (normal_get_command_count((NormalState *)sp)) {} }

bool nvim_normal_handle_special_visual_command_wrapper(void *sp) { return normal_handle_special_visual_command((NormalState *)sp); }

void nvim_normal_invert_horizontal_wrapper(void *sp) { normal_invert_horizontal((NormalState *)sp); }

bool nvim_normal_need_additional_char_wrapper(void *sp) { return normal_need_additional_char((NormalState *)sp); }

/// ui_flush() wrapper.
void nvim_ui_flush_wrapper(void) { ui_flush(); }

/// unshift_special(&ca) wrapper.
void nvim_unshift_special_wrapper(cmdarg_T *ca) { ca->cmdchar = rs_unshift_special(ca->cmdchar, &mod_mask); }

/// Clear MOD_MASK_SHIFT from mod_mask.
void nvim_mod_mask_clear_shift(void) { mod_mask &= ~MOD_MASK_SHIFT; }

void nvim_execute_nv_cmd(int idx, cmdarg_T *ca) { ca->arg = nv_cmds[idx].cmd_arg; (nv_cmds[idx].cmd_func)(ca); }

static int normal_execute(VimState *state, int key) { return rs_normal_execute((NormalState *)state, key); }

static void normal_check_stuff_buffer(NormalState *s)
{
  if (stuff_empty()) {
    did_check_timestamps = false;

    if (need_check_timestamps) {
      check_timestamps(false);
    }

    if (need_wait_return) {
      // if wait_return still needed call it now
      wait_return(false);
    }
  }
}

static void normal_check_interrupt(NormalState *s)
{
  // Reset "got_int" now that we got back to the main loop.  Except when
  // inside a ":g/pat/cmd" command, then the "got_int" needs to abort
  // the ":g" command.
  // For ":g/pat/vi" we reset "got_int" when used once.  When used
  // a second time we go back to Ex mode and abort the ":g" command.
  if (got_int) {
    if (s->noexmode && global_busy && !exmode_active
        && s->previous_got_int) {
      // Typed two CTRL-C in a row: go back to ex mode as if "Q" was
      // used and keep "got_int" set, so that it aborts ":g".
      exmode_active = true;
      State = MODE_NORMAL;
    } else if (!global_busy || !exmode_active) {
      if (!quit_more) {
        // flush all buffers
        vgetc();
      }
      got_int = false;
    }
    s->previous_got_int = true;
  } else {
    s->previous_got_int = false;
  }
}

static void normal_check_window_scrolled(NormalState *s)
{
  if (!finish_op) {
    may_trigger_win_scrolled_resized();
  }
}

static void normal_check_cursor_moved(NormalState *s)
{
  // Trigger CursorMoved if the cursor moved.
  if (!finish_op && has_event(EVENT_CURSORMOVED)
      && (last_cursormoved_win != curwin
          || !equalpos(last_cursormoved, curwin->w_cursor))) {
    apply_autocmds(EVENT_CURSORMOVED, NULL, NULL, false, curbuf);
    last_cursormoved_win = curwin;
    last_cursormoved = curwin->w_cursor;
  }
}

static void normal_check_text_changed(NormalState *s)
{
  // Trigger TextChanged if changedtick differs.
  if (!finish_op && has_event(EVENT_TEXTCHANGED)
      && curbuf->b_last_changedtick != buf_get_changedtick(curbuf)) {
    apply_autocmds(EVENT_TEXTCHANGED, NULL, NULL, false, curbuf);
    curbuf->b_last_changedtick = buf_get_changedtick(curbuf);
  }
}

static void normal_check_buffer_modified(NormalState *s)
{
  // Trigger BufModified if b_modified changed
  if (!finish_op && has_event(EVENT_BUFMODIFIEDSET)
      && curbuf->b_changed_invalid == true) {
    apply_autocmds(EVENT_BUFMODIFIEDSET, NULL, NULL, false, curbuf);
    curbuf->b_changed_invalid = false;
  }
}

static void normal_check_safe_state(NormalState *s) { may_trigger_safestate(!rs_op_pending() && restart_edit == 0); }

static void normal_check_folds(NormalState *s)
{
  // Include a closed fold completely in the Visual area.
  rs_foldAdjustVisual();

  // When 'foldclose' is set, apply 'foldlevel' to folds that don't
  // contain the cursor.
  // When 'foldopen' is "all", open the fold(s) under the cursor.
  // This may mark the window for redrawing.
  if (rs_hasAnyFolding(curwin) && !char_avail()) {
    rs_foldCheckClose();

    if (fdo_flags & kOptFdoFlagAll) {
      rs_foldOpenCursor();
    }
  }
}

static void normal_redraw(NormalState *s)
{
  // Before redrawing, make sure w_topline is correct, and w_leftcol
  // if lines don't wrap, and w_skipcol if lines wrap.
  update_topline(curwin);
  validate_cursor(curwin);

  show_cursor_info_later(false);

  if (must_redraw) {
    update_screen();
  } else {
    redraw_statuslines();
    if (redraw_cmdline || clear_cmdline || redraw_mode) {
      showmode();
    }
  }

  curbuf->b_last_used = time(NULL);

  // Display message after redraw.
  if (keep_msg != NULL) {
    char *const p = xstrdup(keep_msg);

    // msg_start() will set keep_msg to NULL, make a copy
    // first.  Don't reset keep_msg, msg_attr_keep() uses it to
    // check for duplicates.  Never put this message in
    // history.
    msg_hist_off = true;
    msg(p, keep_msg_hl_id);
    msg_hist_off = false;
    xfree(p);
  }

  // show fileinfo after redraw
  if (need_fileinfo && !shortmess(SHM_FILEINFO)) {
    fileinfo(false, true, false);
    need_fileinfo = false;
  }

  emsg_on_display = false;  // can delete error message now
  did_emsg = false;
  msg_didany = false;  // reset lines_left in msg_start()
  may_clear_sb_text();  // clear scroll-back text on next msg

  setcursor();
}

// =============================================================================
// normal_check accessors for Rust FFI
// =============================================================================

void nvim_normal_check_stuff_buffer_wrapper(void *sp) { normal_check_stuff_buffer((NormalState *)sp); }

void nvim_normal_check_interrupt_wrapper(void *sp) { normal_check_interrupt((NormalState *)sp); }

/// Get did_throw global.
bool nvim_get_did_throw_direct(void) { return did_throw; }

/// discard_current_exception wrapper.
void nvim_discard_current_exception_wrapper(void) { discard_current_exception(); }

/// Set quit_more global.
void nvim_set_quit_more(bool val) { quit_more = val; }

/// Get skip_redraw global.
bool nvim_get_skip_redraw(void) { return skip_redraw; }

/// Set skip_redraw global.
void nvim_set_skip_redraw(bool val) { skip_redraw = val; }

/// Set do_redraw global.
void nvim_set_do_redraw(bool val) { do_redraw = val; }

/// setcursor() wrapper.
void nvim_setcursor_wrapper(void) { setcursor(); }

/// update_topline(curwin) wrapper.
void nvim_update_topline_curwin_wrapper(void) { update_topline(curwin); }

void nvim_normal_check_cursor_moved_wrapper(void *sp) { normal_check_cursor_moved((NormalState *)sp); }

void nvim_normal_check_text_changed_wrapper(void *sp) { normal_check_text_changed((NormalState *)sp); }

void nvim_normal_check_window_scrolled_wrapper(void *sp) { normal_check_window_scrolled((NormalState *)sp); }

void nvim_normal_check_buffer_modified_wrapper(void *sp) { normal_check_buffer_modified((NormalState *)sp); }

void nvim_normal_check_safe_state_wrapper(void *sp) { normal_check_safe_state((NormalState *)sp); }

bool nvim_curtab_needs_diff_update(void) { return curtab->tp_diff_update || curtab->tp_diff_invalid; }

/// Clear curtab diff update flag.
void nvim_curtab_clear_diff_update(void) { curtab->tp_diff_update = false; }

/// Get diff_need_scrollbind global.
bool nvim_get_diff_need_scrollbind(void) { return diff_need_scrollbind; }

/// Set diff_need_scrollbind global.
void nvim_set_diff_need_scrollbind(bool val) { diff_need_scrollbind = val; }

/// check_scrollbind(0, 0) wrapper.
void nvim_check_scrollbind_zero_wrapper(void) { check_scrollbind(0, 0); }

void nvim_normal_check_folds_wrapper(void *sp) { normal_check_folds((NormalState *)sp); }

void nvim_normal_redraw_wrapper(void *sp) { normal_redraw((NormalState *)sp); }

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

void nvim_normal_prepare_wrapper(void *sp) { normal_prepare((NormalState *)sp); }

static int normal_check(VimState *state) { return rs_normal_check((NormalState *)state); }

/// End Visual mode.
/// This function should ALWAYS be called to end Visual mode, except from
/// do_pending_operator().
void end_visual_mode(void)
{
  VIsual_select_exclu_adj = false;
  VIsual_active = false;
  setmouse();
  mouse_dragging = 0;

  // Save the current VIsual area for '< and '> marks, and "gv"
  curbuf->b_visual.vi_mode = VIsual_mode;
  curbuf->b_visual.vi_start = VIsual;
  curbuf->b_visual.vi_end = curwin->w_cursor;
  curbuf->b_visual.vi_curswant = curwin->w_curswant;
  curbuf->b_visual_mode_eval = VIsual_mode;
  if (!virtual_active(curwin)) {
    curwin->w_cursor.coladd = 0;
  }

  rs_may_clear_cmdline();

  adjust_cursor_eol();
  may_trigger_modechanged();
}

/// Find the identifier under or to the right of the cursor.
/// "find_type" can have one of three values:
/// FIND_IDENT:   find an identifier (keyword)
/// FIND_STRING:  find any non-white text
/// FIND_IDENT + FIND_STRING: find any non-white text, identifier preferred.
/// FIND_EVAL:  find text useful for C program debugging
///
/// There are three steps:
/// 1. Search forward for the start of an identifier/text.  Doesn't move if
///    already on one.
/// 2. Search backward for the start of this identifier/text.
///    This doesn't match the real Vi but I like it a little better and it
///    shouldn't bother anyone.
/// 3. Search forward to the end of this identifier/text.
///    When FIND_IDENT isn't defined, we backup until a blank.
///
/// @return  the length of the text, or zero if no text is found.
///
/// If text is found, a pointer to the text is put in "*text".  This
/// points into the current buffer line and is not always NUL terminated.
size_t find_ident_under_cursor(char **text, int find_type)
  FUNC_ATTR_NONNULL_ARG(1)
{
  return rs_find_ident_at_pos(curwin, curwin->w_cursor.lnum,
                              curwin->w_cursor.col, text, NULL, find_type);
}

// Routines for displaying a partly typed command
static char old_showcmd_buf[SHOWCMD_BUFLEN];    // For push_showcmd()
static bool showcmd_is_clear = true;
static bool showcmd_visual = false;

// =============================================================================
// showcmd accessors for Rust FFI
// =============================================================================

/// Constants for clear_showcmd (verified with _Static_assert).
_Static_assert(SHOWCMD_COLS == 10, "SHOWCMD_COLS changed");
_Static_assert(SHOWCMD_BUFLEN == SHOWCMD_COLS + 1 + 30, "SHOWCMD_BUFLEN changed");

bool nvim_get_showcmd_is_clear(void) { return showcmd_is_clear; }

void nvim_set_showcmd_is_clear(bool val) { showcmd_is_clear = val; }

bool nvim_get_showcmd_visual(void) { return showcmd_visual; }

void nvim_set_showcmd_visual(bool val) { showcmd_visual = val; }

char *nvim_normal_showcmd_buf_ptr(void) { return showcmd_buf; }

void nvim_normal_display_showcmd(void) { display_showcmd(); }

/// Compute Visual area info and write result into showcmd_buf.
/// Returns true if in Visual mode and char_avail() is false.
bool nvim_clear_showcmd_visual_info(void)
{
  if (!VIsual_active || char_avail()) {
    return false;
  }

  bool cursor_bot = lt(VIsual, curwin->w_cursor);
  int lines;
  colnr_T leftcol, rightcol;
  linenr_T top, bot;

  if (cursor_bot) {
    top = VIsual.lnum;
    bot = curwin->w_cursor.lnum;
  } else {
    top = curwin->w_cursor.lnum;
    bot = VIsual.lnum;
  }
  hasFolding(curwin, top, &top, NULL);
  hasFolding(curwin, bot, NULL, &bot);
  lines = bot - top + 1;

  if (VIsual_mode == Ctrl_V) {
    char *const saved_sbr = p_sbr;
    char *const saved_w_sbr = curwin->w_p_sbr;
    p_sbr = empty_string_option;
    curwin->w_p_sbr = empty_string_option;
    getvcols(curwin, &curwin->w_cursor, &VIsual, &leftcol, &rightcol);
    p_sbr = saved_sbr;
    curwin->w_p_sbr = saved_w_sbr;
    snprintf(showcmd_buf, SHOWCMD_BUFLEN, "%" PRId64 "x%" PRId64,
             (int64_t)lines, (int64_t)rightcol - leftcol + 1);
  } else if (VIsual_mode == 'V' || VIsual.lnum != curwin->w_cursor.lnum) {
    snprintf(showcmd_buf, SHOWCMD_BUFLEN, "%" PRId64, (int64_t)lines);
  } else {
    char *s, *e;
    int bytes = 0;
    int chars = 0;

    if (cursor_bot) {
      s = ml_get_pos(&VIsual);
      e = get_cursor_pos_ptr();
    } else {
      s = get_cursor_pos_ptr();
      e = ml_get_pos(&VIsual);
    }
    while ((*p_sel != 'e') ? s <= e : s < e) {
      int l = utfc_ptr2len(s);
      if (l == 0) {
        bytes++;
        chars++;
        break;
      }
      bytes += l;
      chars++;
      s += l;
    }
    if (bytes == chars) {
      snprintf(showcmd_buf, SHOWCMD_BUFLEN, "%d", chars);
    } else {
      snprintf(showcmd_buf, SHOWCMD_BUFLEN, "%d-%d", chars, bytes);
    }
  }
  int limit = ui_has(kUIMessages) ? SHOWCMD_BUFLEN - 1 : SHOWCMD_COLS;
  showcmd_buf[limit] = NUL;
  return true;
}

/// Add 'c' to string of shown command chars.
///
/// @return  true if output has been written (and setcursor() has been called).
bool add_to_showcmd(int c)
{
  static int ignore[] = {
    K_IGNORE,
    K_LEFTMOUSE, K_LEFTDRAG, K_LEFTRELEASE, K_MOUSEMOVE,
    K_MIDDLEMOUSE, K_MIDDLEDRAG, K_MIDDLERELEASE,
    K_RIGHTMOUSE, K_RIGHTDRAG, K_RIGHTRELEASE,
    K_MOUSEDOWN, K_MOUSEUP, K_MOUSELEFT, K_MOUSERIGHT,
    K_X1MOUSE, K_X1DRAG, K_X1RELEASE, K_X2MOUSE, K_X2DRAG, K_X2RELEASE,
    K_EVENT,
    0
  };

  if (!p_sc || msg_silent != 0) {
    return false;
  }

  if (showcmd_visual) {
    showcmd_buf[0] = NUL;
    showcmd_visual = false;
  }

  // Ignore keys that are scrollbar updates and mouse clicks
  if (IS_SPECIAL(c)) {
    for (int i = 0; ignore[i] != 0; i++) {
      if (ignore[i] == c) {
        return false;
      }
    }
  }

  char *p;
  char mbyte_buf[MB_MAXCHAR + 1];
  if (c <= 0x7f || !vim_isprintc(c)) {
    p = transchar(c);
    if (*p == ' ') {
      STRCPY(p, "<20>");
    }
  } else {
    mbyte_buf[utf_char2bytes(c, mbyte_buf)] = NUL;
    p = mbyte_buf;
  }
  size_t old_len = strlen(showcmd_buf);
  size_t extra_len = strlen(p);
  size_t limit = ui_has(kUIMessages) ? SHOWCMD_BUFLEN - 1 : SHOWCMD_COLS;
  if (old_len + extra_len > limit) {
    size_t overflow = old_len + extra_len - limit;
    memmove(showcmd_buf, showcmd_buf + overflow, old_len - overflow + 1);
  }
  strcat(showcmd_buf, p);

  if (char_avail()) {
    return false;
  }

  display_showcmd();

  return true;
}

void add_to_showcmd_c(int c) { add_to_showcmd(c); setcursor(); }

/// Delete 'len' characters from the end of the shown command.
static void del_from_showcmd(int len)
{
  if (!p_sc) {
    return;
  }

  int old_len = (int)strlen(showcmd_buf);
  len = MIN(len, old_len);
  showcmd_buf[old_len - len] = NUL;

  if (!char_avail()) {
    display_showcmd();
  }
}

/// push_showcmd() and pop_showcmd() are used when waiting for the user to type
/// something and there is a partial mapping.
void push_showcmd(void)
{
  if (p_sc) {
    STRCPY(old_showcmd_buf, showcmd_buf);
  }
}

void pop_showcmd(void)
{
  if (!p_sc) {
    return;
  }

  STRCPY(showcmd_buf, old_showcmd_buf);

  display_showcmd();
}

static void display_showcmd(void)
{
  showcmd_is_clear = (showcmd_buf[0] == NUL);

  if (*p_sloc == 's') {
    if (showcmd_is_clear) {
      curwin->w_redr_status = true;
    } else {
      win_redr_status(curwin);
      setcursor();  // put cursor back where it belongs
    }
    return;
  }
  if (*p_sloc == 't') {
    if (showcmd_is_clear) {
      redraw_tabline = true;
    } else {
      draw_tabline();
      setcursor();  // put cursor back where it belongs
    }
    return;
  }
  // 'showcmdloc' is "last" or empty

  if (ui_has(kUIMessages)) {
    MAXSIZE_TEMP_ARRAY(content, 1);
    MAXSIZE_TEMP_ARRAY(chunk, 3);
    if (!showcmd_is_clear) {
      // placeholder for future highlight support
      ADD_C(chunk, INTEGER_OBJ(0));
      ADD_C(chunk, CSTR_AS_OBJ(showcmd_buf));
      ADD_C(chunk, INTEGER_OBJ(0));
      ADD_C(content, ARRAY_OBJ(chunk));
    }
    ui_call_msg_showcmd(content);
    return;
  }
  if (p_ch == 0) {
    return;
  }

  msg_grid_validate();
  int showcmd_row = Rows - 1;
  grid_line_start(&msg_grid_adj, showcmd_row);

  int len = 0;
  if (!showcmd_is_clear) {
    len = grid_line_puts(sc_col, showcmd_buf, -1, HL_ATTR(HLF_MSG));
  }

  // clear the rest of an old message by outputting up to SHOWCMD_COLS spaces
  grid_line_puts(sc_col + len, (char *)"          " + len, -1, HL_ATTR(HLF_MSG));

  grid_line_flush();
}

/// When "check" is false, prepare for commands that scroll the window.
/// When "check" is true, take care of scroll-binding after the window has
/// scrolled.  Called from normal_cmd() and edit().
void do_check_scrollbind(bool check)
{
  static win_T *old_curwin = NULL;
  static linenr_T old_vtopline = 0;
  static buf_T *old_buf = NULL;
  static colnr_T old_leftcol = 0;

  int vtopline = rs_get_vtopline(curwin);

  if (check && curwin->w_p_scb) {
    // If a ":syncbind" command was just used, don't scroll, only reset
    // the values.
    if (did_syncbind) {
      did_syncbind = false;
    } else if (curwin == old_curwin) {
      // Synchronize other windows, as necessary according to
      // 'scrollbind'.  Don't do this after an ":edit" command, except
      // when 'diff' is set.
      if ((curwin->w_buffer == old_buf
           || curwin->w_p_diff
           )
          && (vtopline != old_vtopline
              || curwin->w_leftcol != old_leftcol)) {
        check_scrollbind(vtopline - old_vtopline, curwin->w_leftcol - old_leftcol);
      }
    } else if (vim_strchr(p_sbo, 'j')) {  // jump flag set in 'scrollopt'
      // When switching between windows, make sure that the relative
      // vertical offset is valid for the new window.  The relative
      // offset is invalid whenever another 'scrollbind' window has
      // scrolled to a point that would force the current window to
      // scroll past the beginning or end of its buffer.  When the
      // resync is performed, some of the other 'scrollbind' windows may
      // need to jump so that the current window's relative position is
      // visible on-screen.
      check_scrollbind(vtopline - curwin->w_scbind_pos, 0);
    }
    curwin->w_scbind_pos = vtopline;
  }

  old_curwin = curwin;
  old_vtopline = vtopline;
  old_buf = curwin->w_buffer;
  old_leftcol = curwin->w_leftcol;
}

/// Synchronize any windows that have "scrollbind" set, based on the
/// number of rows by which the current window has changed
/// (1998-11-02 16:21:01  R. Edward Ralston <eralston@computer.org>)
void check_scrollbind(linenr_T vtopline_diff, int leftcol_diff)
{
  win_T *old_curwin = curwin;
  buf_T *old_curbuf = curbuf;
  int old_VIsual_select = VIsual_select;
  int old_VIsual_active = VIsual_active;
  colnr_T tgt_leftcol = curwin->w_leftcol;

  // check 'scrollopt' string for vertical and horizontal scroll options
  bool want_ver = old_curwin->w_p_diff
                  || (vim_strchr(p_sbo, 'v') && vtopline_diff != 0);
  bool want_hor = (vim_strchr(p_sbo, 'h') && (leftcol_diff || vtopline_diff != 0));

  // loop through the scrollbound windows and scroll accordingly
  VIsual_select = VIsual_active = 0;
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    curwin = wp;
    curbuf = curwin->w_buffer;
    // skip original window and windows with 'noscrollbind'
    if (curwin == old_curwin || !curwin->w_p_scb) {
      continue;
    }

    // do the vertical scroll
    if (want_ver) {
      if (old_curwin->w_p_diff && curwin->w_p_diff) {
        rs_diff_set_topline(old_curwin, curwin);
      } else {
        curwin->w_scbind_pos += vtopline_diff;
        int curr_vtopline = rs_get_vtopline(curwin);

        // Perf: reuse curr_vtopline to reduce the time in plines_m_win_fill().
        // Equivalent to:
        //   int max_vtopline = plines_m_win_fill(curwin, 1, curbuf->b_ml.ml_line_count);
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

    // do the horizontal scroll
    if (want_hor) {
      set_leftcol(tgt_leftcol);
    }
  }

  // reset current-window
  VIsual_select = old_VIsual_select;
  VIsual_active = old_VIsual_active;
  curwin = old_curwin;
  curbuf = old_curbuf;
}

/// CTRL-A and CTRL-X: Add or subtract from letter or number under cursor.
static void nv_addsub(cmdarg_T *cap)
{
  if (bt_prompt(curbuf) && !prompt_curpos_editable()) {
    rs_clearopbeep(cap->oap);
  } else if (!VIsual_active && cap->oap->op_type == OP_NOP) {
    rs_prep_redo_cmd(cap);
    cap->oap->op_type = cap->cmdchar == Ctrl_A ? OP_NR_ADD : OP_NR_SUB;
    op_addsub(cap->oap, cap->count1, cap->arg);
    cap->oap->op_type = OP_NOP;
  } else if (VIsual_active) {
    rs_nv_operator(cap);
  } else {
    rs_clearop(cap->oap);
  }
}

/// Implementation of "gd" and "gD" command.
///
/// @param thisblock  1 for "1gd" and "1gD"
static void nv_gd(oparg_T *oap, int nchar, int thisblock)
{
  size_t len;
  char *ptr;
  if ((len = find_ident_under_cursor(&ptr, FIND_IDENT)) == 0
      || !find_decl(ptr, len, nchar == 'd', thisblock, SEARCH_START)) {
    rs_clearopbeep(oap);
    return;
  }

  if ((fdo_flags & kOptFdoFlagSearch) && KeyTyped && oap->op_type == OP_NOP) {
    rs_foldOpenCursor();
  }
  // clear any search statistics
  if (messaging() && !msg_silent && !shortmess(SHM_SEARCHCOUNT)) {
    clear_cmdline = true;
  }
}

/// Search for variable declaration of "ptr[len]".
/// When "locally" is true in the current function ("gd"), otherwise in the
/// current file ("gD").
///
/// @param thisblock  when true check the {} block scope.
/// @param flags_arg  flags passed to searchit()
///
/// @return           fail when not found.
bool find_decl(char *ptr, size_t len, bool locally, bool thisblock, int flags_arg)
{
  pos_T par_pos;
  pos_T found_pos;
  bool t;
  bool retval = true;
  bool incll;
  int searchflags = flags_arg;

  size_t patsize = len + 7;
  char *pat = xmalloc(patsize);

  // Put "\V" before the pattern to avoid that the special meaning of "."
  // and "~" causes trouble.
  assert(patsize <= INT_MAX);
  size_t patlen = (size_t)snprintf(pat, patsize,
                                   vim_iswordp(ptr) ? "\\V\\<%.*s\\>" : "\\V%.*s",
                                   (int)len, ptr);
  pos_T old_pos = curwin->w_cursor;
  bool save_p_ws = p_ws;
  bool save_p_scs = p_scs;
  p_ws = false;         // don't wrap around end of file now
  p_scs = false;        // don't switch ignorecase off now

  // With "gD" go to line 1.
  // With "gd" Search back for the start of the current function, then go
  // back until a blank line.  If this fails go to line 1.
  if (!locally || !findpar(&incll, BACKWARD, 1, '{', false)) {
    setpcmark();                        // Set in findpar() otherwise
    curwin->w_cursor.lnum = 1;
    par_pos = curwin->w_cursor;
  } else {
    par_pos = curwin->w_cursor;
    while (curwin->w_cursor.lnum > 1
           && *skipwhite(get_cursor_line_ptr()) != NUL) {
      curwin->w_cursor.lnum--;
    }
  }
  curwin->w_cursor.col = 0;

  // Search forward for the identifier, ignore comment lines.
  clearpos(&found_pos);
  while (true) {
    t = searchit(curwin, curbuf, &curwin->w_cursor, NULL, FORWARD,
                 pat, patlen, 1, searchflags, RE_LAST, NULL);
    if (curwin->w_cursor.lnum >= old_pos.lnum) {
      t = false;         // match after start is failure too
    }

    if (thisblock && t != false) {
      const int64_t maxtravel = old_pos.lnum - curwin->w_cursor.lnum + 1;
      const pos_T *pos = findmatchlimit(NULL, '}', FM_FORWARD, maxtravel);

      // Check that the block the match is in doesn't end before the
      // position where we started the search from.
      if (pos != NULL && pos->lnum < old_pos.lnum) {
        // There can't be a useful match before the end of this block.
        // Skip to the end
        curwin->w_cursor = *pos;
        continue;
      }
    }

    if (t == false) {
      // If we previously found a valid position, use it.
      if (found_pos.lnum != 0) {
        curwin->w_cursor = found_pos;
        t = true;
      }
      break;
    }
    if (get_leader_len(get_cursor_line_ptr(), NULL, false, true) > 0) {
      // Ignore this line, continue at start of next line.
      curwin->w_cursor.lnum++;
      curwin->w_cursor.col = 0;
      continue;
    }
    bool valid = rs_is_ident(get_cursor_line_ptr(), curwin->w_cursor.col);

    // If the current position is not a valid identifier and a previous match is
    // present, favor that one instead.
    if (!valid && found_pos.lnum != 0) {
      curwin->w_cursor = found_pos;
      break;
    }
    // global search: use first match found
    if (valid && !locally) {
      break;
    }
    if (valid && curwin->w_cursor.lnum >= par_pos.lnum) {
      // If we previously found a valid position, use it.
      if (found_pos.lnum != 0) {
        curwin->w_cursor = found_pos;
      }
      break;
    }

    // For finding a local variable and the match is before the "{" or
    // inside a comment, continue searching.  For K&R style function
    // declarations this skips the function header without types.
    if (!valid) {
      clearpos(&found_pos);
    } else {
      found_pos = curwin->w_cursor;
    }
    // Remove SEARCH_START from flags to avoid getting stuck at one position.
    searchflags &= ~SEARCH_START;
  }

  if (t == false) {
    retval = false;
    curwin->w_cursor = old_pos;
  } else {
    curwin->w_set_curswant = true;
    // "n" searches forward now
    reset_search_dir();
  }

  xfree(pat);
  p_ws = save_p_ws;
  p_scs = save_p_scs;

  return retval;
}

/// Move 'dist' lines in direction 'dir', counting lines by *screen*
/// lines rather than lines in the file.
/// 'dist' must be positive.
///
/// @return  true if able to move cursor, false otherwise.
bool nv_screengo(oparg_T *oap, int dir, int dist, bool skip_conceal)
{
  int linelen = linetabsize(curwin, curwin->w_cursor.lnum);
  bool retval = true;
  bool atend = false;
  int col_off1;                 // margin offset for first screen line
  int col_off2;                 // margin offset for wrapped screen line
  int width1;                   // text width for first screen line
  int width2;                   // text width for wrapped screen line

  oap->motion_type = kMTCharWise;
  oap->inclusive = (curwin->w_curswant == MAXCOL);

  col_off1 = win_col_off(curwin);
  col_off2 = col_off1 - win_col_off2(curwin);
  width1 = curwin->w_view_width - col_off1;
  width2 = curwin->w_view_width - col_off2;

  if (width2 == 0) {
    width2 = 1;  // Avoid divide by zero.
  }

  if (curwin->w_view_width != 0) {
    int n;
    // Instead of sticking at the last character of the buffer line we
    // try to stick in the last column of the screen.
    if (curwin->w_curswant == MAXCOL) {
      atend = true;
      validate_virtcol(curwin);
      if (width1 <= 0) {
        curwin->w_curswant = 0;
      } else {
        curwin->w_curswant = width1 - 1;
        if (curwin->w_virtcol > curwin->w_curswant) {
          curwin->w_curswant += ((curwin->w_virtcol
                                  - curwin->w_curswant -
                                  1) / width2 + 1) * width2;
        }
      }
    } else {
      if (linelen > width1) {
        n = ((linelen - width1 - 1) / width2 + 1) * width2 + width1;
      } else {
        n = width1;
      }
      curwin->w_curswant = MIN(curwin->w_curswant, n - 1);
    }

    while (dist--) {
      if (dir == BACKWARD) {
        if (curwin->w_curswant >= width1
            && !hasFolding(curwin, curwin->w_cursor.lnum, NULL, NULL)) {
          // Move back within the line. This can give a negative value
          // for w_curswant if width1 < width2 (with cpoptions+=n),
          // which will get clipped to column 0.
          curwin->w_curswant -= width2;
        } else {
          // to previous line
          if (curwin->w_cursor.lnum <= 1) {
            retval = false;
            break;
          }
          cursor_up_inner(curwin, 1, skip_conceal);

          linelen = linetabsize(curwin, curwin->w_cursor.lnum);
          if (linelen > width1) {
            int w = (((linelen - width1 - 1) / width2) + 1) * width2;
            assert(curwin->w_curswant <= INT_MAX - w);
            curwin->w_curswant += w;
          }
        }
      } else {  // dir == FORWARD
        if (linelen > width1) {
          n = ((linelen - width1 - 1) / width2 + 1) * width2 + width1;
        } else {
          n = width1;
        }
        if (curwin->w_curswant + width2 < (colnr_T)n
            && !hasFolding(curwin, curwin->w_cursor.lnum, NULL, NULL)) {
          // move forward within line
          curwin->w_curswant += width2;
        } else {
          // to next line
          if (curwin->w_cursor.lnum >= curwin->w_buffer->b_ml.ml_line_count) {
            retval = false;
            break;
          }
          cursor_down_inner(curwin, 1, skip_conceal);
          curwin->w_curswant %= width2;

          // Check if the cursor has moved below the number display
          // when width1 < width2 (with cpoptions+=n). Subtract width2
          // to get a negative value for w_curswant, which will get
          // clipped to column 0.
          if (curwin->w_curswant >= width1) {
            curwin->w_curswant -= width2;
          }
          linelen = linetabsize(curwin, curwin->w_cursor.lnum);
        }
      }
    }
  }

  if (virtual_active(curwin) && atend) {
    coladvance(curwin, MAXCOL);
  } else {
    coladvance(curwin, curwin->w_curswant);
  }

  if (curwin->w_cursor.col > 0 && curwin->w_p_wrap) {
    // Check for landing on a character that got split at the end of the
    // last line.  We want to advance a screenline, not end up in the same
    // screenline or move two screenlines.
    validate_virtcol(curwin);
    colnr_T virtcol = curwin->w_virtcol;
    if (virtcol > (colnr_T)width1 && *rs_get_showbreak_value(curwin) != NUL) {
      virtcol -= vim_strsize((char *)rs_get_showbreak_value(curwin));
    }

    int c = utf_ptr2char(get_cursor_pos_ptr());
    if (dir == FORWARD && virtcol < curwin->w_curswant
        && (curwin->w_curswant <= (colnr_T)width1)
        && !vim_isprintc(c) && c > 255) {
      oneright();
    }

    if (virtcol > curwin->w_curswant
        && (curwin->w_curswant < (colnr_T)width1
            ? (curwin->w_curswant > (colnr_T)width1 / 2)
            : ((curwin->w_curswant - width1) % width2
               > (colnr_T)width2 / 2))) {
      curwin->w_cursor.col--;
    }
  }

  if (atend) {
    curwin->w_curswant = MAXCOL;            // stick in the last column
  }
  adjust_skipcol();

  return retval;
}

/// Get the count specified after a 'z' command. Only the 'z<CR>', 'zl', 'zh',
/// 'z<Left>', and 'z<Right>' commands accept a count after 'z'.
/// @return  true to process the 'z' command and false to skip it.
static bool nv_z_get_count(cmdarg_T *cap, int *nchar_arg)
{
  int nchar = *nchar_arg;

  // "z123{nchar}": edit the count before obtaining {nchar}
  if (rs_checkclearop(cap->oap)) {
    return false;
  }
  int n = nchar - '0';

  while (true) {
    no_mapping++;
    allow_keys++;         // no mapping for nchar, but allow key codes
    nchar = plain_vgetc();
    LANGMAP_ADJUST(nchar, true);
    no_mapping--;
    allow_keys--;
    add_to_showcmd(nchar);

    if (nchar == K_DEL || nchar == K_KDEL) {
      n /= 10;
    } else if (ascii_isdigit(nchar)) {
      if (vim_append_digit_int(&n, nchar - '0') == FAIL) {
        rs_clearopbeep(cap->oap);
        break;
      }
    } else if (nchar == CAR) {
      rs_win_setheight(n);
      break;
    } else if (nchar == 'l'
               || nchar == 'h'
               || nchar == K_LEFT
               || nchar == K_RIGHT) {
      cap->count1 = n ? n * cap->count1 : cap->count1;
      *nchar_arg = nchar;
      return true;
    } else {
      rs_clearopbeep(cap->oap);
      break;
    }
  }
  cap->oap->op_type = OP_NOP;
  return false;
}

/// "zug" and "zuw": undo "zg" and "zw"
/// "zg": add good word to word list
/// "zw": add wrong word to word list
/// "zG": add good word to temp word list
/// "zW": add wrong word to temp word list
static int nv_zg_zw(cmdarg_T *cap, int nchar)
{
  bool undo = false;

  if (nchar == 'u') {
    no_mapping++;
    allow_keys++;               // no mapping for nchar, but allow key codes
    nchar = plain_vgetc();
    LANGMAP_ADJUST(nchar, true);
    no_mapping--;
    allow_keys--;
    add_to_showcmd(nchar);

    if (vim_strchr("gGwW", nchar) == NULL) {
      rs_clearopbeep(cap->oap);
      return OK;
    }
    undo = true;
  }

  if (rs_checkclearop(cap->oap)) {
    return OK;
  }
  char *ptr = NULL;
  size_t len;
  if (VIsual_active && !get_visual_text(cap, &ptr, &len)) {
    return FAIL;
  }
  if (ptr == NULL) {
    pos_T pos = curwin->w_cursor;

    // Find bad word under the cursor.  When 'spell' is
    // off this fails and find_ident_under_cursor() is
    // used below.
    emsg_off++;
    len = spell_move_to(curwin, FORWARD, SMT_ALL, true, NULL);
    emsg_off--;
    if (len != 0 && curwin->w_cursor.col <= pos.col) {
      ptr = ml_get_pos(&curwin->w_cursor);
    }
    curwin->w_cursor = pos;
  }

  if (ptr == NULL && (len = find_ident_under_cursor(&ptr, FIND_IDENT)) == 0) {
    return FAIL;
  }
  assert(len <= INT_MAX);
  spell_add_word(ptr, (int)len,
                 nchar == 'w' || nchar == 'W' ? SPELL_ADD_BAD : SPELL_ADD_GOOD,
                 (nchar == 'G' || nchar == 'W') ? 0 : cap->count1,
                 undo);

  return OK;
}

/// Commands that start with "z" (implementation).
static void nv_zet_impl(cmdarg_T *cap)
{
  colnr_T col;
  int nchar = cap->nchar;
  int old_fdl = (int)curwin->w_p_fdl;
  int old_fen = curwin->w_p_fen;

  int siso = rs_get_sidescrolloff_value(curwin);

  if (ascii_isdigit(nchar) && !nv_z_get_count(cap, &nchar)) {
    return;
  }

  // "zf" and "zF" are always an operator, "zd", "zo", "zO", "zc"
  // and "zC" only in Visual mode.  "zj" and "zk" are motion
  // commands.
  if (cap->nchar != 'f' && cap->nchar != 'F'
      && !(VIsual_active && vim_strchr("dcCoO", cap->nchar))
      && cap->nchar != 'j' && cap->nchar != 'k'
      && rs_checkclearop(cap->oap)) {
    return;
  }

  // For "z+", "z<CR>", "zt", "z.", "zz", "z^", "z-", "zb":
  // If line number given, set cursor.
  if ((vim_strchr("+\r\nt.z^-b", nchar) != NULL)
      && cap->count0
      && cap->count0 != curwin->w_cursor.lnum) {
    setpcmark();
    if (cap->count0 > curbuf->b_ml.ml_line_count) {
      curwin->w_cursor.lnum = curbuf->b_ml.ml_line_count;
    } else {
      curwin->w_cursor.lnum = cap->count0;
    }
    check_cursor_col(curwin);
  }

  switch (nchar) {
  // "z+", "z<CR>" and "zt": put cursor at top of screen
  case '+':
    if (cap->count0 == 0) {
      // No count given: put cursor at the line below screen
      validate_botline(curwin);               // make sure w_botline is valid
      curwin->w_cursor.lnum = MIN(curwin->w_botline, curbuf->b_ml.ml_line_count);
    }
    FALLTHROUGH;
  case NL:
  case CAR:
  case K_KENTER:
    beginline(BL_WHITE | BL_FIX);
    FALLTHROUGH;

  case 't':
    scroll_cursor_top(curwin, 0, true);
    redraw_later(curwin, UPD_VALID);
    rs_set_fraction(curwin);
    break;

  // "z." and "zz": put cursor in middle of screen
  case '.':
    beginline(BL_WHITE | BL_FIX);
    FALLTHROUGH;

  case 'z':
    scroll_cursor_halfway(curwin, true, false);
    redraw_later(curwin, UPD_VALID);
    rs_set_fraction(curwin);
    break;

  // "z^", "z-" and "zb": put cursor at bottom of screen
  case '^':     // Strange Vi behavior: <count>z^ finds line at top of window
                // when <count> is at bottom of window, and puts that one at
                // bottom of window.
    if (cap->count0 != 0) {
      scroll_cursor_bot(curwin, 0, true);
      curwin->w_cursor.lnum = curwin->w_topline;
    } else if (curwin->w_topline == 1) {
      curwin->w_cursor.lnum = 1;
    } else {
      curwin->w_cursor.lnum = curwin->w_topline - 1;
    }
    FALLTHROUGH;
  case '-':
    beginline(BL_WHITE | BL_FIX);
    FALLTHROUGH;

  case 'b':
    scroll_cursor_bot(curwin, 0, true);
    redraw_later(curwin, UPD_VALID);
    rs_set_fraction(curwin);
    break;

  // "zH" - scroll screen right half-page
  case 'H':
    cap->count1 *= curwin->w_view_width / 2;
    FALLTHROUGH;

  // "zh" - scroll screen to the right
  case 'h':
  case K_LEFT:
    if (!curwin->w_p_wrap) {
      set_leftcol((colnr_T)cap->count1 > curwin->w_leftcol
                  ? 0 : curwin->w_leftcol - (colnr_T)cap->count1);
    }
    break;

  // "zL" - scroll window left half-page
  case 'L':
    cap->count1 *= curwin->w_view_width / 2;
    FALLTHROUGH;

  // "zl" - scroll window to the left if not wrapping
  case 'l':
  case K_RIGHT:
    if (!curwin->w_p_wrap) {
      set_leftcol(curwin->w_leftcol + (colnr_T)cap->count1);
    }
    break;

  // "zs" - scroll screen, cursor at the start
  case 's':
    if (!curwin->w_p_wrap) {
      if (hasFolding(curwin, curwin->w_cursor.lnum, NULL, NULL)) {
        col = 0;                        // like the cursor is in col 0
      } else {
        getvcol(curwin, &curwin->w_cursor, &col, NULL, NULL);
      }
      if (col > siso) {
        col -= siso;
      } else {
        col = 0;
      }
      if (curwin->w_leftcol != col) {
        curwin->w_leftcol = col;
        redraw_later(curwin, UPD_NOT_VALID);
      }
    }
    break;

  // "ze" - scroll screen, cursor at the end
  case 'e':
    if (!curwin->w_p_wrap) {
      if (hasFolding(curwin, curwin->w_cursor.lnum, NULL, NULL)) {
        col = 0;                        // like the cursor is in col 0
      } else {
        getvcol(curwin, &curwin->w_cursor, NULL, NULL, &col);
      }
      int n = curwin->w_view_width - win_col_off(curwin);
      if (col + siso < n) {
        col = 0;
      } else {
        col = col + siso - n + 1;
      }
      if (curwin->w_leftcol != col) {
        curwin->w_leftcol = col;
        redraw_later(curwin, UPD_NOT_VALID);
      }
    }
    break;

  // "zp", "zP" in block mode put without adding trailing spaces
  case 'P':
  case 'p':
    rs_nv_put(cap);
    break;
  // "zy" Yank without trailing spaces
  case 'y':
    rs_nv_operator(cap);
    break;

  // "zF": create fold command
  // "zf": create fold operator
  case 'F':
  case 'f':
    if (rs_foldManualAllowed(true)) {
      cap->nchar = 'f';
      rs_nv_operator(cap);
      curwin->w_p_fen = true;

      // "zF" is like "zfzf"
      if (nchar == 'F' && cap->oap->op_type == OP_FOLD) {
        rs_nv_operator(cap);
        finish_op = true;
      }
    } else {
      rs_clearopbeep(cap->oap);
    }
    break;

  // "zd": delete fold at cursor
  // "zD": delete fold at cursor recursively
  case 'd':
  case 'D':
    if (rs_foldManualAllowed(false)) {
      if (VIsual_active) {
        rs_nv_operator(cap);
      } else {
        rs_deleteFold(curwin, curwin->w_cursor.lnum,
                   curwin->w_cursor.lnum, nchar == 'D', false);
      }
    }
    break;

  // "zE": erase all folds
  case 'E':
    if (rs_foldmethodIsManual(curwin)) {
      rs_clearFolding(curwin);
      changed_window_setting(curwin);
    } else if (rs_foldmethodIsMarker(curwin)) {
      rs_deleteFold(curwin, 1, curbuf->b_ml.ml_line_count, true, false);
    } else {
      emsg(_("E352: Cannot erase folds with current 'foldmethod'"));
    }
    break;

  // "zn": fold none: reset 'foldenable'
  case 'n':
    curwin->w_p_fen = false;
    break;

  // "zN": fold Normal: set 'foldenable'
  case 'N':
    curwin->w_p_fen = true;
    break;

  // "zi": invert folding: toggle 'foldenable'
  case 'i':
    curwin->w_p_fen = !curwin->w_p_fen;
    break;

  // "za": open closed fold or close open fold at cursor
  case 'a':
    if (hasFolding(curwin, curwin->w_cursor.lnum, NULL, NULL)) {
      rs_setFoldRepeat(curwin->w_cursor.lnum, cap->count1, true);
    } else {
      rs_setFoldRepeat(curwin->w_cursor.lnum, cap->count1, false);
      curwin->w_p_fen = true;
    }
    break;

  // "zA": open fold at cursor recursively
  case 'A':
    if (hasFolding(curwin, curwin->w_cursor.lnum, NULL, NULL)) {
      rs_setManualFold(curwin->w_cursor.lnum, true, true, NULL);
    } else {
      rs_setManualFold(curwin->w_cursor.lnum, false, true, NULL);
      curwin->w_p_fen = true;
    }
    break;

  // "zo": open fold at cursor or Visual area
  case 'o':
    if (VIsual_active) {
      rs_nv_operator(cap);
    } else {
      rs_setFoldRepeat(curwin->w_cursor.lnum, cap->count1, true);
    }
    break;

  // "zO": open fold recursively
  case 'O':
    if (VIsual_active) {
      rs_nv_operator(cap);
    } else {
      rs_setManualFold(curwin->w_cursor.lnum, true, true, NULL);
    }
    break;

  // "zc": close fold at cursor or Visual area
  case 'c':
    if (VIsual_active) {
      rs_nv_operator(cap);
    } else {
      rs_setFoldRepeat(curwin->w_cursor.lnum, cap->count1, false);
    }
    curwin->w_p_fen = true;
    break;

  // "zC": close fold recursively
  case 'C':
    if (VIsual_active) {
      rs_nv_operator(cap);
    } else {
      rs_setManualFold(curwin->w_cursor.lnum, false, true, NULL);
    }
    curwin->w_p_fen = true;
    break;

  // "zv": open folds at the cursor
  case 'v':
    rs_foldOpenCursor();
    break;

  // "zx": re-apply 'foldlevel' and open folds at the cursor
  case 'x':
    curwin->w_p_fen = true;
    curwin->w_foldinvalid = true;               // recompute folds
    rs_newFoldLevel();                             // update right now
    rs_foldOpenCursor();
    break;

  // "zX": undo manual opens/closes, re-apply 'foldlevel'
  case 'X':
    curwin->w_p_fen = true;
    curwin->w_foldinvalid = true;               // recompute folds
    old_fdl = -1;                               // force an update
    break;

  // "zm": fold more
  case 'm':
    if (curwin->w_p_fdl > 0) {
      curwin->w_p_fdl -= cap->count1;
      curwin->w_p_fdl = MAX(curwin->w_p_fdl, 0);
    }
    old_fdl = -1;                       // force an update
    curwin->w_p_fen = true;
    break;

  // "zM": close all folds
  case 'M':
    curwin->w_p_fdl = 0;
    old_fdl = -1;                       // force an update
    curwin->w_p_fen = true;
    break;

  // "zr": reduce folding
  case 'r':
    curwin->w_p_fdl += cap->count1;
    {
      int d = rs_getDeepestNesting(curwin);
      curwin->w_p_fdl = MIN(curwin->w_p_fdl, d);
    }
    break;

  case 'R':     //  "zR": open all folds
    curwin->w_p_fdl = rs_getDeepestNesting(curwin);
    old_fdl = -1;                       // force an update
    break;

  case 'j':     // "zj" move to next fold downwards
  case 'k':     // "zk" move to next fold upwards
    if (rs_foldMoveTo(true, nchar == 'j' ? FORWARD : BACKWARD,
                   cap->count1) == false) {
      rs_clearopbeep(cap->oap);
    }
    break;

  case 'u':     // "zug" and "zuw": undo "zg" and "zw"
  case 'g':     // "zg": add good word to word list
  case 'w':     // "zw": add wrong word to word list
  case 'G':     // "zG": add good word to temp word list
  case 'W':     // "zW": add wrong word to temp word list
    if (nv_zg_zw(cap, nchar) == FAIL) {
      return;
    }
    break;

  case '=':     // "z=": suggestions for a badly spelled word
    if (!rs_checkclearop(cap->oap)) {
      spell_suggest(cap->count0);
    }
    break;

  default:
    rs_clearopbeep(cap->oap);
  }

  // Redraw when 'foldenable' changed
  if (old_fen != curwin->w_p_fen) {
    if (rs_foldmethodIsDiff(curwin) && curwin->w_p_scb) {
      // Adjust 'foldenable' in diff-synced windows.
      FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
        if (wp != curwin && rs_foldmethodIsDiff(wp) && wp->w_p_scb) {
          wp->w_p_fen = curwin->w_p_fen;
          changed_window_setting(wp);
        }
      }
    }
    changed_window_setting(curwin);
  }

  // Redraw when 'foldlevel' changed.
  if (old_fdl != curwin->w_p_fdl) {
    rs_newFoldLevel();
  }
}

/// Handle a ":" command and <Cmd> or Lua mappings.
static void nv_colon(cmdarg_T *cap)
{
  bool cmd_result;
  bool is_cmdkey = cap->cmdchar == K_COMMAND;
  bool is_lua = cap->cmdchar == K_LUA;

  if (VIsual_active && !is_cmdkey && !is_lua) {
    rs_nv_operator(cap);
    return;
  }

  if (cap->oap->op_type != OP_NOP) {
    // Using ":" as a movement is charwise exclusive.
    cap->oap->motion_type = kMTCharWise;
    cap->oap->inclusive = false;
  } else if (cap->count0 && !is_cmdkey && !is_lua) {
    // translate "count:" into ":.,.+(count - 1)"
    stuffcharReadbuff('.');
    if (cap->count0 > 1) {
      stuffReadbuff(",.+");
      stuffnumReadbuff(cap->count0 - 1);
    }
  }

  // When typing, don't type below an old message
  if (KeyTyped) {
    compute_cmdrow();
  }

  if (is_lua) {
    cmd_result = map_execute_lua(true, false);
  } else {
    // get a command line and execute it
    cmd_result = do_cmdline(NULL, is_cmdkey ? getcmdkeycmd : getexline, NULL,
                            cap->oap->op_type != OP_NOP ? DOCMD_KEEPLINE : 0);
  }

  if (cmd_result == false) {
    // The Ex command failed, do not execute the operator.
    rs_clearop(cap->oap);
  } else if (cap->oap->op_type != OP_NOP
             && (cap->oap->start.lnum > curbuf->b_ml.ml_line_count
                 || cap->oap->start.col > ml_get_len(cap->oap->start.lnum)
                 || did_emsg)) {
    // The start of the operator has become invalid by the Ex command.
    rs_clearopbeep(cap->oap);
  }
}

/// Call nv_ident() as if "c1" was used, with "c2" as next character.
void do_nv_ident(int c1, int c2)
{
  oparg_T oa;
  cmdarg_T ca;

  clear_oparg(&oa);
  CLEAR_FIELD(ca);
  ca.oap = &oa;
  ca.cmdchar = c1;
  ca.nchar = c2;
  rs_nv_ident(&ca);
}

/// 'K' normal-mode command. Get the command to lookup the keyword under the
/// cursor.
static size_t nv_K_getcmd(cmdarg_T *cap, char *kp, bool kp_help, bool kp_ex, char **ptr_arg,
                          size_t n, char *buf, size_t bufsize, size_t *buflen)
{
  if (kp_help) {
    // in the help buffer
    STRCPY(buf, "he! ");
    *buflen = STRLEN_LITERAL("he! ");
    return n;
  }

  if (kp_ex) {
    *buflen = 0;
    // 'keywordprg' is an ex command
    if (cap->count0 != 0) {  // Send the count to the ex command.
      *buflen = (size_t)snprintf(buf, bufsize, "%" PRId64, (int64_t)(cap->count0));
    }
    *buflen += (size_t)snprintf(buf + *buflen, bufsize - *buflen, "%s ", kp);
    return n;
  }

  char *ptr = *ptr_arg;

  // An external command will probably use an argument starting
  // with "-" as an option.  To avoid trouble we skip the "-".
  while (*ptr == '-' && n > 0) {
    ptr++;
    n--;
  }
  if (n == 0) {
    // found dashes only
    emsg(_(e_noident));
    xfree(buf);
    *ptr_arg = ptr;
    return 0;
  }

  // When a count is given, turn it into a range.  Is this
  // really what we want?
  bool isman = (strcmp(kp, "man") == 0);
  bool isman_s = (strcmp(kp, "man -s") == 0);
  if (cap->count0 != 0 && !(isman || isman_s)) {
    *buflen = (size_t)snprintf(buf, bufsize, ".,.+%" PRId64, (int64_t)(cap->count0 - 1));
  }

  do_cmdline_cmd("tabnew");
  *buflen += (size_t)snprintf(buf + *buflen, bufsize - *buflen, "terminal ");
  if (cap->count0 == 0 && isman_s) {
    *buflen += (size_t)snprintf(buf + *buflen, bufsize - *buflen, "man ");
  } else {
    *buflen += (size_t)snprintf(buf + *buflen, bufsize - *buflen, "%s ", kp);
  }
  if (cap->count0 != 0 && (isman || isman_s)) {
    *buflen += (size_t)snprintf(buf + *buflen, bufsize - *buflen,
                                "%" PRId64 " ", (int64_t)cap->count0);
  }

  *ptr_arg = ptr;
  return n;
}

/// Implementation of nv_ident.
static void nv_ident_impl(cmdarg_T *cap)
{
  char *ptr = NULL;
  char *p;
  size_t n = 0;                 // init for GCC
  int cmdchar;
  bool g_cmd;                   // "g" command
  bool tag_cmd = false;

  if (cap->cmdchar == 'g') {    // "g*", "g#", "g]" and "gCTRL-]"
    cmdchar = cap->nchar;
    g_cmd = true;
  } else {
    cmdchar = cap->cmdchar;
    g_cmd = false;
  }

  if (cmdchar == POUND) {       // the pound sign, '#' for English keyboards
    cmdchar = '#';
  }

  // The "]", "CTRL-]" and "K" commands accept an argument in Visual mode.
  if (cmdchar == ']' || cmdchar == Ctrl_RSB || cmdchar == 'K') {
    if (VIsual_active && get_visual_text(cap, &ptr, &n) == false) {
      return;
    }
    if (rs_checkclearopq(cap->oap)) {
      return;
    }
  }

  if (ptr == NULL && (n = find_ident_under_cursor(&ptr,
                                                  ((cmdchar == '*'
                                                    || cmdchar == '#')
                                                   ? FIND_IDENT|FIND_STRING
                                                   : FIND_IDENT))) == 0) {
    rs_clearop(cap->oap);
    return;
  }

  // Allocate buffer to put the command in.  Inserting backslashes can
  // double the length of the word.  p_kp / curbuf->b_p_kp could be added
  // and some numbers.
  char *kp = *curbuf->b_p_kp == NUL ? p_kp : curbuf->b_p_kp;  // 'keywordprg'
  bool kp_help = (*kp == NUL || strcmp(kp, ":he") == 0 || strcmp(kp, ":help") == 0);
  if (kp_help && *skipwhite(ptr) == NUL) {
    emsg(_(e_noident));   // found white space only
    return;
  }
  bool kp_ex = (*kp == ':');  // 'keywordprg' is an ex command
  size_t bufsize = n * 2 + 30 + strlen(kp);
  char *buf = xmalloc(bufsize);
  buf[0] = NUL;
  size_t buflen = 0;

  switch (cmdchar) {
  case '*':
  case '#':
    // Put cursor at start of word, makes search skip the word
    // under the cursor.
    // Call setpcmark() first, so "*``" puts the cursor back where
    // it was.
    setpcmark();
    curwin->w_cursor.col = (colnr_T)(ptr - get_cursor_line_ptr());

    if (!g_cmd && vim_iswordp(ptr)) {
      STRCPY(buf, "\\<");
      buflen = STRLEN_LITERAL("\\<");
    }
    no_smartcase = true;                // don't use 'smartcase' now
    break;

  case 'K':
    n = nv_K_getcmd(cap, kp, kp_help, kp_ex, &ptr, n, buf, bufsize, &buflen);
    if (n == 0) {
      return;
    }
    break;

  case ']':
    tag_cmd = true;
    STRCPY(buf, "ts ");
    buflen = STRLEN_LITERAL("ts ");
    break;

  default:
    tag_cmd = true;
    if (curbuf->b_help) {
      STRCPY(buf, "he! ");
      buflen = STRLEN_LITERAL("he! ");
    } else {
      if (g_cmd) {
        STRCPY(buf, "tj ");
        buflen = STRLEN_LITERAL("tj ");
      } else if (cap->count0 == 0) {
        STRCPY(buf, "ta ");
        buflen = STRLEN_LITERAL("ta ");
      } else {
        buflen = (size_t)snprintf(buf, bufsize, ":%" PRId64 "ta ", (int64_t)cap->count0);
      }
    }
  }

  // Now grab the chars in the identifier
  if (cmdchar == 'K' && !kp_help) {
    ptr = xstrnsave(ptr, n);
    if (kp_ex) {
      // Escape the argument properly for an Ex command
      p = vim_strsave_fnameescape(ptr, VSE_NONE);
    } else {
      // Escape the argument properly for a shell command
      p = vim_strsave_shellescape(ptr, true, true);
    }
    xfree(ptr);
    size_t plen = strlen(p);
    char *newbuf = xrealloc(buf, buflen + plen + 1);
    buf = newbuf;
    STRCPY(buf + buflen, p);
    buflen += plen;
    xfree(p);
  } else {
    char *aux_ptr;
    if (cmdchar == '*') {
      aux_ptr = (rs_magic_isset() ? "/.*~[^$\\" : "/^$\\");
    } else if (cmdchar == '#') {
      aux_ptr = (rs_magic_isset() ? "/?.*~[^$\\" : "/?^$\\");
    } else if (tag_cmd) {
      if (strcmp(curbuf->b_p_ft, "help") == 0) {
        // ":help" handles unescaped argument
        aux_ptr = "";
      } else {
        aux_ptr = "\\|\"\n[";
      }
    } else {
      aux_ptr = "\\|\"\n*?[";
    }

    p = buf + buflen;
    while (n-- > 0) {
      // put a backslash before \ and some others
      if (vim_strchr(aux_ptr, (uint8_t)(*ptr)) != NULL) {
        *p++ = '\\';
      }

      // When current byte is a part of multibyte character, copy all
      // bytes of that character.
      const size_t len = (size_t)(utfc_ptr2len(ptr) - 1);
      for (size_t i = 0; i < len && n > 0; i++, n--) {
        *p++ = *ptr++;
      }
      *p++ = *ptr++;
    }
    *p = NUL;
    buflen = (size_t)(p - buf);
  }

  // Execute the command.
  if (cmdchar == '*' || cmdchar == '#') {
    if (!g_cmd && vim_iswordp(mb_prevptr(get_cursor_line_ptr(), ptr))) {
      STRCPY(buf + buflen, "\\>");
      buflen += STRLEN_LITERAL("\\>");
    }

    // put pattern in search history
    init_history();
    add_to_history(HIST_SEARCH, buf, buflen, true, NUL);

    normal_search(cap, cmdchar == '*' ? '/' : '?', buf, buflen, 0, NULL);
  } else {
    g_tag_at_cursor = true;
    do_cmdline_cmd(buf);
    g_tag_at_cursor = false;

    if (cmdchar == 'K' && !kp_ex && !kp_help) {
      // Start insert mode in terminal buffer
      restart_edit = 'i';

      add_map("<esc>", "<Cmd>bdelete!<CR>", MODE_TERMINAL, true);
    }
  }

  xfree(buf);
}

/// Get visually selected text, within one line only.
///
/// @param pp    return: start of selected text
/// @param lenp  return: length of selected text
///
/// @return      false if more than one line selected.
bool get_visual_text(cmdarg_T *cap, char **pp, size_t *lenp)
{
  if (VIsual_mode != 'V') {
    rs_unadjust_for_sel();
  }
  if (VIsual.lnum != curwin->w_cursor.lnum) {
    if (cap != NULL) {
      rs_clearopbeep(cap->oap);
    }
    return false;
  }
  if (VIsual_mode == 'V') {
    *pp = get_cursor_line_ptr();
    *lenp = (size_t)get_cursor_line_len();
  } else {
    if (lt(curwin->w_cursor, VIsual)) {
      *pp = ml_get_pos(&curwin->w_cursor);
      *lenp = (size_t)VIsual.col - (size_t)curwin->w_cursor.col + 1;
    } else {
      *pp = ml_get_pos(&VIsual);
      *lenp = (size_t)curwin->w_cursor.col - (size_t)VIsual.col + 1;
    }
    if (**pp == NUL) {
      *lenp = 0;
    }
    if (*lenp > 0) {
      // Correct the length to include all bytes of the last character.
      *lenp += (size_t)(utfc_ptr2len(*pp + (*lenp - 1)) - 1);
    }
  }
  rs_reset_VIsual_and_resel();
  return true;
}

/// Handle scrolling command 'H', 'L' and 'M' (implementation).
static void nv_scroll_impl(cmdarg_T *cap)
{
  int n;
  linenr_T lnum;

  cap->oap->motion_type = kMTLineWise;
  setpcmark();

  if (cap->cmdchar == 'L') {
    validate_botline(curwin);          // make sure curwin->w_botline is valid
    curwin->w_cursor.lnum = curwin->w_botline - 1;
    if (cap->count1 - 1 >= curwin->w_cursor.lnum) {
      curwin->w_cursor.lnum = 1;
    } else {
      if (win_lines_concealed(curwin)) {
        // Count a fold for one screen line.
        for (n = cap->count1 - 1; n > 0 && curwin->w_cursor.lnum > curwin->w_topline; n--) {
          hasFolding(curwin, curwin->w_cursor.lnum, &curwin->w_cursor.lnum, NULL);
          n += decor_conceal_line(curwin, curwin->w_cursor.lnum, true);
          if (curwin->w_cursor.lnum > curwin->w_topline) {
            curwin->w_cursor.lnum--;
          }
        }
      } else {
        curwin->w_cursor.lnum -= cap->count1 - 1;
      }
    }
  } else {
    if (cap->cmdchar == 'M') {
      int used = 0;
      // Don't count filler lines above the window.
      used -= win_get_fill(curwin, curwin->w_topline) - curwin->w_topfill;
      validate_botline(curwin);  // make sure w_empty_rows is valid
      int half = (curwin->w_view_height - curwin->w_empty_rows + 1) / 2;
      for (n = 0; curwin->w_topline + n < curbuf->b_ml.ml_line_count; n++) {
        // Count half the number of filler lines to be "below this
        // line" and half to be "above the next line".
        if (n > 0 && used + win_get_fill(curwin, curwin->w_topline + n) / 2 >= half) {
          n--;
          break;
        }
        used += plines_win(curwin, curwin->w_topline + n, true);
        if (used >= half) {
          break;
        }
        if (hasFolding(curwin, curwin->w_topline + n, NULL, &lnum)) {
          n = lnum - curwin->w_topline;
        }
      }
      if (n > 0 && used > curwin->w_view_height) {
        n--;
      }
    } else {  // (cap->cmdchar == 'H')
      n = cap->count1 - 1;
      if (win_lines_concealed(curwin)) {
        // Count a fold for one screen line.
        lnum = curwin->w_topline;
        while ((decor_conceal_line(curwin, lnum - 1, true) || n-- > 0)
               && lnum < curwin->w_botline - 1) {
          hasFolding(curwin, lnum, NULL, &lnum);
          lnum++;
        }
        n = lnum - curwin->w_topline;
      }
    }
    curwin->w_cursor.lnum = MIN(curwin->w_topline + n, curbuf->b_ml.ml_line_count);
  }

  // Correct for 'so', except when an operator is pending.
  if (cap->oap->op_type == OP_NOP) {
    cursor_correct(curwin);
  }
  beginline(BL_SOL | BL_FIX);
}

/// Cursor right commands (implementation).
static void nv_right_impl(cmdarg_T *cap)
{
  int n;

  if (mod_mask & (MOD_MASK_SHIFT | MOD_MASK_CTRL)) {
    // <C-Right> and <S-Right> move a word or WORD right
    if (mod_mask & MOD_MASK_CTRL) {
      cap->arg = true;
    }
    rs_nv_wordcmd(cap);
    return;
  }

  cap->oap->motion_type = kMTCharWise;
  cap->oap->inclusive = false;
  bool past_line = (VIsual_active && *p_sel != 'o');

  // In virtual edit mode, there's no such thing as "past_line", as lines
  // are (theoretically) infinitely long.
  if (virtual_active(curwin)) {
    past_line = false;
  }

  for (n = cap->count1; n > 0; n--) {
    if ((!past_line && oneright() == false)
        || (past_line && *get_cursor_pos_ptr() == NUL)) {
      //    <Space> wraps to next line if 'whichwrap' has 's'.
      //        'l' wraps to next line if 'whichwrap' has 'l'.
      // CURS_RIGHT wraps to next line if 'whichwrap' has '>'.
      if (((cap->cmdchar == ' ' && vim_strchr(p_ww, 's') != NULL)
           || (cap->cmdchar == 'l' && vim_strchr(p_ww, 'l') != NULL)
           || (cap->cmdchar == K_RIGHT && vim_strchr(p_ww, '>') != NULL))
          && curwin->w_cursor.lnum < curbuf->b_ml.ml_line_count) {
        // When deleting we also count the NL as a character.
        // Set cap->oap->inclusive when last char in the line is
        // included, move to next line after that
        if (cap->oap->op_type != OP_NOP
            && !cap->oap->inclusive
            && !LINEEMPTY(curwin->w_cursor.lnum)) {
          cap->oap->inclusive = true;
        } else {
          curwin->w_cursor.lnum++;
          curwin->w_cursor.col = 0;
          curwin->w_cursor.coladd = 0;
          curwin->w_set_curswant = true;
          cap->oap->inclusive = false;
        }
        continue;
      }
      if (cap->oap->op_type == OP_NOP) {
        // Only beep and flush if not moved at all
        if (n == cap->count1) {
          beep_flush();
        }
      } else {
        if (!LINEEMPTY(curwin->w_cursor.lnum)) {
          cap->oap->inclusive = true;
        }
      }
      break;
    } else if (past_line) {
      curwin->w_set_curswant = true;
      if (virtual_active(curwin)) {
        oneright();
      } else {
        curwin->w_cursor.col += utfc_ptr2len(get_cursor_pos_ptr());
      }
    }
  }
  if (n != cap->count1 && (fdo_flags & kOptFdoFlagHor) && KeyTyped
      && cap->oap->op_type == OP_NOP) {
    rs_foldOpenCursor();
  }
}

/// Cursor left commands (implementation).
///
/// @return  true when operator end should not be adjusted.
static void nv_left_impl(cmdarg_T *cap)
{
  int n;

  if (mod_mask & (MOD_MASK_SHIFT | MOD_MASK_CTRL)) {
    // <C-Left> and <S-Left> move a word or WORD left
    if (mod_mask & MOD_MASK_CTRL) {
      cap->arg = 1;
    }
    rs_nv_bck_word(cap);
    return;
  }

  cap->oap->motion_type = kMTCharWise;
  cap->oap->inclusive = false;
  for (n = cap->count1; n > 0; n--) {
    if (oneleft() == false) {
      // <BS> and <Del> wrap to previous line if 'whichwrap' has 'b'.
      //                 'h' wraps to previous line if 'whichwrap' has 'h'.
      //           CURS_LEFT wraps to previous line if 'whichwrap' has '<'.
      if ((((cap->cmdchar == K_BS || cap->cmdchar == Ctrl_H)
            && vim_strchr(p_ww, 'b') != NULL)
           || (cap->cmdchar == 'h' && vim_strchr(p_ww, 'h') != NULL)
           || (cap->cmdchar == K_LEFT && vim_strchr(p_ww, '<') != NULL))
          && curwin->w_cursor.lnum > 1) {
        curwin->w_cursor.lnum--;
        coladvance(curwin, MAXCOL);
        curwin->w_set_curswant = true;

        // When the NL before the first char has to be deleted we
        // put the cursor on the NUL after the previous line.
        // This is a very special case, be careful!
        // Don't adjust op_end now, otherwise it won't work.
        if ((cap->oap->op_type == OP_DELETE || cap->oap->op_type == OP_CHANGE)
            && !LINEEMPTY(curwin->w_cursor.lnum)) {
          char *cp = get_cursor_pos_ptr();

          if (*cp != NUL) {
            curwin->w_cursor.col += utfc_ptr2len(cp);
          }
          cap->retval |= CA_NO_ADJ_OP_END;
        }
        continue;
      } else if (cap->oap->op_type == OP_NOP && n == cap->count1) {
        // Only beep and flush if not moved at all
        beep_flush();
      }
      break;
    }
  }
  if (n != cap->count1 && (fdo_flags & kOptFdoFlagHor) && KeyTyped
      && cap->oap->op_type == OP_NOP) {
    rs_foldOpenCursor();
  }
}

/// Cursor up commands (implementation).
/// cap->arg is true for "-": Move cursor to first non-blank.
static void nv_up_impl(cmdarg_T *cap)
{
  if (mod_mask & MOD_MASK_SHIFT) {
    // <S-Up> is page up
    cap->arg = BACKWARD;
    rs_nv_page(cap);
    return;
  }

  cap->oap->motion_type = kMTLineWise;
  if (cursor_up(cap->count1, cap->oap->op_type == OP_NOP) == false) {
    rs_clearopbeep(cap->oap);
  } else if (cap->arg) {
    beginline(BL_WHITE | BL_FIX);
  }
}

/// Cursor down commands (implementation).
/// cap->arg is true for CR and "+": Move cursor to first non-blank.
static void nv_down_impl(cmdarg_T *cap)
{
  if (mod_mask & MOD_MASK_SHIFT) {
    // <S-Down> is page down
    cap->arg = FORWARD;
    rs_nv_page(cap);
  } else if (bt_quickfix(curbuf) && cap->cmdchar == CAR) {
    // Quickfix window only: view the result under the cursor.
    rs_qf_view_result(false);
  } else {
    // In the cmdline window a <CR> executes the command.
    if (cmdwin_type != 0 && cap->cmdchar == CAR) {
      cmdwin_result = CAR;
    } else if (bt_prompt(curbuf) && cap->cmdchar == CAR
               && curwin->w_cursor.lnum == curbuf->b_ml.ml_line_count) {
      // In a prompt buffer a <CR> in the last line invokes the callback.
      prompt_invoke_callback();
      if (restart_edit == 0) {
        restart_edit = 'a';
      }
    } else {
      cap->oap->motion_type = kMTLineWise;
      if (cursor_down(cap->count1, cap->oap->op_type == OP_NOP) == false) {
        rs_clearopbeep(cap->oap);
      } else if (cap->arg) {
        beginline(BL_WHITE | BL_FIX);
      }
    }
  }
}

/// Grab the file name under the cursor and edit it.
static void nv_gotofile(cmdarg_T *cap)
{
  linenr_T lnum = -1;

  if (rs_check_text_or_curbuf_locked(cap->oap)) {
    return;
  }

  if (!check_can_set_curbuf_disabled()) {
    return;
  }

  char *ptr = grab_file_name(cap->count1, &lnum);

  if (ptr != NULL) {
    // do autowrite if necessary
    if (curbufIsChanged() && curbuf->b_nwindows <= 1 && !buf_hide(curbuf)) {
      autowrite(curbuf, false);
    }
    setpcmark();
    if (do_ecmd(0, ptr, NULL, NULL, ECMD_LAST,
                buf_hide(curbuf) ? ECMD_HIDE : 0, curwin) == OK
        && cap->nchar == 'F' && lnum >= 0) {
      curwin->w_cursor.lnum = lnum;
      check_cursor_lnum(curwin);
      beginline(BL_SOL | BL_FIX);
    }
    xfree(ptr);
  } else {
    rs_clearop(cap->oap);
  }
}

/// Implementation of '?' and '/' commands.
/// If cap->arg is true don't set PC mark.
static void nv_search_impl(cmdarg_T *cap)
{
  oparg_T *oap = cap->oap;
  pos_T save_cursor = curwin->w_cursor;

  if (cap->cmdchar == '?' && cap->oap->op_type == OP_ROT13) {
    // Translate "g??" to "g?g?"
    cap->cmdchar = 'g';
    cap->nchar = '?';
    rs_nv_operator(cap);
    return;
  }

  // When using 'incsearch' the cursor may be moved to set a different search
  // start position.
  cap->searchbuf = getcmdline(cap->cmdchar, cap->count1, 0, true);

  if (cap->searchbuf == NULL) {
    rs_clearop(oap);
    return;
  }

  normal_search(cap, cap->cmdchar, cap->searchbuf, strlen(cap->searchbuf),
                (cap->arg || !equalpos(save_cursor, curwin->w_cursor))
                ? 0 : SEARCH_MARK, NULL);
}

/// Implementation of "N" and "n" commands.
/// cap->arg is SEARCH_REV for "N", 0 for "n".
static void nv_next_impl(cmdarg_T *cap)
{
  pos_T old = curwin->w_cursor;
  int wrapped = false;
  int i = normal_search(cap, 0, NULL, 0, SEARCH_MARK | cap->arg, &wrapped);

  if (i == 1 && !wrapped && equalpos(old, curwin->w_cursor)) {
    // Avoid getting stuck on the current cursor position, which can happen when
    // an offset is given and the cursor is on the last char in the buffer:
    // Repeat with count + 1.
    cap->count1 += 1;
    normal_search(cap, 0, NULL, 0, SEARCH_MARK | cap->arg, NULL);
    cap->count1 -= 1;
  }

  // Redraw the window to refresh the highlighted matches.
  if (i > 0 && p_hls && !no_hlsearch
      && win_hl_attr(curwin, HLF_LC) != win_hl_attr(curwin, HLF_L)) {
    redraw_later(curwin, UPD_SOME_VALID);
  }
}

/// Search for "pat" in direction "dir" ('/' or '?', 0 for repeat).
/// Uses only cap->count1 and cap->oap from "cap".
///
/// @param opt  extra flags for do_search()
///
/// @return 0 for failure, 1 for found, 2 for found and line offset added.
static int normal_search(cmdarg_T *cap, int dir, char *pat, size_t patlen, int opt, int *wrapped)
{
  searchit_arg_T sia;
  pos_T const prev_cursor = curwin->w_cursor;

  cap->oap->motion_type = kMTCharWise;
  cap->oap->inclusive = false;
  cap->oap->use_reg_one = true;
  curwin->w_set_curswant = true;

  CLEAR_FIELD(sia);
  int i = do_search(cap->oap, dir, dir, pat, patlen, cap->count1,
                    opt | SEARCH_OPT | SEARCH_ECHO | SEARCH_MSG, &sia);
  if (wrapped != NULL) {
    *wrapped = sia.sa_wrapped;
  }
  if (i == 0) {
    rs_clearop(cap->oap);
  } else {
    if (i == 2) {
      cap->oap->motion_type = kMTLineWise;
    }
    curwin->w_cursor.coladd = 0;
    if (cap->oap->op_type == OP_NOP && (fdo_flags & kOptFdoFlagSearch) && KeyTyped) {
      rs_foldOpenCursor();
    }
  }
  // Redraw the window to refresh the highlighted matches.
  if (!equalpos(curwin->w_cursor, prev_cursor) && p_hls && !no_hlsearch
      && win_hl_attr(curwin, HLF_LC) != win_hl_attr(curwin, HLF_L)) {
    redraw_later(curwin, UPD_SOME_VALID);
  }

  // "/$" will put the cursor after the end of the line, may need to
  // correct that here
  check_cursor(curwin);

  return i;
}

/// "[{", "[(", "]}" or "])": go to Nth unclosed '{', '(', '}' or ')'
/// "[#", "]#": go to start/end of Nth innermost #if..#endif construct.
/// "[/", "[*", "]/", "]*": go to Nth comment start/end.
/// "[m" or "]m" search for prev/next start of (Java) method.
/// "[M" or "]M" search for prev/next end of (Java) method.
static void nv_bracket_block(cmdarg_T *cap, const pos_T *old_pos)
{
  pos_T new_pos = { 0, 0, 0 };
  pos_T *pos = NULL;  // init for GCC
  pos_T prev_pos;
  int n;
  int findc;

  if (cap->nchar == '*') {
    cap->nchar = '/';
  }
  prev_pos.lnum = 0;
  if (cap->nchar == 'm' || cap->nchar == 'M') {
    if (cap->cmdchar == '[') {
      findc = '{';
    } else {
      findc = '}';
    }
    n = 9999;
  } else {
    findc = cap->nchar;
    n = cap->count1;
  }
  for (; n > 0; n--) {
    if ((pos = findmatchlimit(cap->oap, findc,
                              (cap->cmdchar == '[') ? FM_BACKWARD : FM_FORWARD, 0)) == NULL) {
      if (new_pos.lnum == 0) {        // nothing found
        if (cap->nchar != 'm' && cap->nchar != 'M') {
          rs_clearopbeep(cap->oap);
        }
      } else {
        pos = &new_pos;               // use last one found
      }
      break;
    }
    prev_pos = new_pos;
    curwin->w_cursor = *pos;
    new_pos = *pos;
  }
  curwin->w_cursor = *old_pos;

  // Handle "[m", "]m", "[M" and "[M".  The findmatchlimit() only
  // brought us to the match for "[m" and "]M" when inside a method.
  // Try finding the '{' or '}' we want to be at.
  // Also repeat for the given count.
  if (cap->nchar == 'm' || cap->nchar == 'M') {
    int c;
    // norm is true for "]M" and "[m"
    bool norm = ((findc == '{') == (cap->nchar == 'm'));

    n = cap->count1;
    // found a match: we were inside a method
    if (prev_pos.lnum != 0) {
      pos = &prev_pos;
      curwin->w_cursor = prev_pos;
      if (norm) {
        n--;
      }
    } else {
      pos = NULL;
    }
    while (n > 0) {
      while (true) {
        if ((findc == '{' ? dec_cursor() : inc_cursor()) < 0) {
          // if not found anything, that's an error
          if (pos == NULL) {
            rs_clearopbeep(cap->oap);
          }
          n = 0;
          break;
        }
        c = gchar_cursor();
        if (c == '{' || c == '}') {
          // Must have found end/start of class: use it.
          // Or found the place to be at.
          if ((c == findc && norm) || (n == 1 && !norm)) {
            new_pos = curwin->w_cursor;
            pos = &new_pos;
            n = 0;
          } else if (new_pos.lnum == 0) {
            // if no match found at all, we started outside of the
            // class and we're inside now.  Just go on.
            new_pos = curwin->w_cursor;
            pos = &new_pos;
          } else if ((pos = findmatchlimit(cap->oap, findc,
                                           (cap->cmdchar == '[') ? FM_BACKWARD : FM_FORWARD,
                                           0)) == NULL) {
            // found start/end of other method: go to match
            n = 0;
          } else {
            curwin->w_cursor = *pos;
          }
          break;
        }
      }
      n--;
    }
    curwin->w_cursor = *old_pos;
    if (pos == NULL && new_pos.lnum != 0) {
      rs_clearopbeep(cap->oap);
    }
  }
  if (pos != NULL) {
    setpcmark();
    curwin->w_cursor = *pos;
    curwin->w_set_curswant = true;
    if ((fdo_flags & kOptFdoFlagBlock) && KeyTyped
        && cap->oap->op_type == OP_NOP) {
      rs_foldOpenCursor();
    }
  }
}

/// "[" and "]" commands implementation.
/// cap->arg is BACKWARD for "[" and FORWARD for "]".
static void nv_brackets_impl(cmdarg_T *cap)
{
  int flag;
  int n;

  cap->oap->motion_type = kMTCharWise;
  cap->oap->inclusive = false;
  pos_T old_pos = curwin->w_cursor;         // cursor position before command
  curwin->w_cursor.coladd = 0;              // TODO(Unknown): don't do this for an error.

  // "[f" or "]f" : Edit file under the cursor (same as "gf")
  if (cap->nchar == 'f') {
    nv_gotofile(cap);
  } else if (vim_strchr("iI\011dD\004", cap->nchar) != NULL) {
    // Find the occurrence(s) of the identifier or define under cursor
    // in current and included files or jump to the first occurrence.
    //
    //                    search       list           jump
    //                  fwd   bwd    fwd   bwd     fwd    bwd
    // identifier       "]i"  "[i"   "]I"  "[I"   "]^I"  "[^I"
    // define           "]d"  "[d"   "]D"  "[D"   "]^D"  "[^D"
    char *ptr;
    size_t len;

    if ((len = find_ident_under_cursor(&ptr, FIND_IDENT)) == 0) {
      rs_clearop(cap->oap);
    } else {
      // Make a copy, if the line was changed it will be freed.
      ptr = xmemdupz(ptr, len);
      find_pattern_in_path(ptr, 0, len, true,
                           cap->count0 == 0 ? !isupper(cap->nchar) : false,
                           (((cap->nchar & 0xf) == ('d' & 0xf))
                            ? FIND_DEFINE
                            : FIND_ANY),
                           cap->count1,
                           (isupper(cap->nchar) ? ACTION_SHOW_ALL
                                                : islower(cap->nchar) ? ACTION_SHOW
                                                                      : ACTION_GOTO),
                           (cap->cmdchar == ']'
                            ? curwin->w_cursor.lnum + 1
                            : 1),
                           MAXLNUM,
                           false, false);
      xfree(ptr);
      curwin->w_set_curswant = true;
    }
  } else if ((cap->cmdchar == '[' && vim_strchr("{(*/#mM", cap->nchar) != NULL)
             || (cap->cmdchar == ']' && vim_strchr("})*/#mM", cap->nchar) != NULL)) {
    // "[{", "[(", "]}" or "])": go to Nth unclosed '{', '(', '}' or ')'
    // "[#", "]#": go to start/end of Nth innermost #if..#endif construct.
    // "[/", "[*", "]/", "]*": go to Nth comment start/end.
    // "[m" or "]m" search for prev/next start of (Java) method.
    // "[M" or "]M" search for prev/next end of (Java) method.
    nv_bracket_block(cap, &old_pos);
  } else if (cap->nchar == '[' || cap->nchar == ']') {
    // "[[", "[]", "]]" and "][": move to start or end of function
    if (cap->nchar == cap->cmdchar) {               // "]]" or "[["
      flag = '{';
    } else {
      flag = '}';                   // "][" or "[]"
    }
    curwin->w_set_curswant = true;
    // Imitate strange Vi behaviour: When using "]]" with an operator we also stop at '}'.
    if (!findpar(&cap->oap->inclusive, cap->arg, cap->count1, flag,
                 (cap->oap->op_type != OP_NOP
                  && cap->arg == FORWARD && flag == '{'))) {
      rs_clearopbeep(cap->oap);
    } else {
      if (cap->oap->op_type == OP_NOP) {
        beginline(BL_WHITE | BL_FIX);
      }
      if ((fdo_flags & kOptFdoFlagBlock) && KeyTyped && cap->oap->op_type == OP_NOP) {
        rs_foldOpenCursor();
      }
    }
  } else if (cap->nchar == 'p' || cap->nchar == 'P') {
    // "[p", "[P", "]P" and "]p": put with indent adjustment
    nv_put_opt(cap, true);
  } else if (cap->nchar == '\'' || cap->nchar == '`') {
    // "['", "[`", "]'" and "]`": jump to next mark
    fmark_T *fm = pos_to_mark(curbuf, NULL, curwin->w_cursor);
    assert(fm != NULL);
    fmark_T *prev_fm;
    for (n = cap->count1; n > 0; n--) {
      prev_fm = fm;
      fm = getnextmark(&fm->mark, cap->cmdchar == '[' ? BACKWARD : FORWARD,
                       cap->nchar == '\'');
      if (fm == NULL) {
        break;
      }
    }
    if (fm == NULL) {
      fm = prev_fm;
    }
    MarkMove flags = kMarkContext;
    flags |= cap->nchar == '\'' ? kMarkBeginLine : 0;
    nv_mark_move_to(cap, flags, fm);
  } else if (cap->nchar >= K_RIGHTRELEASE && cap->nchar <= K_LEFTMOUSE) {
    // [ or ] followed by a middle mouse click: put selected text with
    // indent adjustment.  Any other button just does as usual.
    do_mouse(cap->oap, cap->nchar,
             (cap->cmdchar == ']') ? FORWARD : BACKWARD,
             cap->count1, PUT_FIXINDENT);
  } else if (cap->nchar == 'z') {
    // "[z" and "]z": move to start or end of open fold.
    if (rs_foldMoveTo(false, cap->cmdchar == ']' ? FORWARD : BACKWARD,
                   cap->count1) == false) {
      rs_clearopbeep(cap->oap);
    }
  } else if (cap->nchar == 'c') {
    // "[c" and "]c": move to next or previous diff-change.
    if (rs_diff_move_to(cap->cmdchar == ']' ? FORWARD : BACKWARD,
                     cap->count1) == false) {
      rs_clearopbeep(cap->oap);
    }
  } else if (cap->nchar == 'r' || cap->nchar == 's' || cap->nchar == 'S') {
    // "[r", "[s", "[S", "]r", "]s" and "]S": move to next spell error.
    setpcmark();
    for (n = 0; n < cap->count1; n++) {
      if (spell_move_to(curwin, cap->cmdchar == ']' ? FORWARD : BACKWARD,
                        cap->nchar == 's'
                        ? SMT_ALL
                        : cap->nchar == 'r' ? SMT_RARE : SMT_BAD,
                        false, NULL) == 0) {
        rs_clearopbeep(cap->oap);
        break;
      }
      curwin->w_set_curswant = true;
    }
    if (cap->oap->op_type == OP_NOP && (fdo_flags & kOptFdoFlagSearch) && KeyTyped) {
      rs_foldOpenCursor();
    }
  } else {
    // Not a valid cap->nchar.
    rs_clearopbeep(cap->oap);
  }
}

/// "u" command: Undo or make lower case (implementation).
static void nv_undo_impl(cmdarg_T *cap)
{
  if (cap->oap->op_type == OP_LOWER
      || VIsual_active) {
    // translate "<Visual>u" to "<Visual>gu" and "guu" to "gugu"
    cap->cmdchar = 'g';
    cap->nchar = 'u';
    rs_nv_operator(cap);
  } else {
    rs_nv_kundo(cap);
  }
}

/// Handle the "r" command (implementation).
static void nv_replace_impl(cmdarg_T *cap)
{
  int had_ctrl_v;

  if (rs_checkclearop(cap->oap)) {
    return;
  }
  if (bt_prompt(curbuf) && !prompt_curpos_editable()) {
    rs_clearopbeep(cap->oap);
    return;
  }

  // get another character
  if (cap->nchar == Ctrl_V || cap->nchar == Ctrl_Q) {
    had_ctrl_v = Ctrl_V;
    cap->nchar = get_literal(false);
    // Don't redo a multibyte character with CTRL-V.
    if (cap->nchar > DEL) {
      had_ctrl_v = NUL;
    }
  } else {
    had_ctrl_v = NUL;
  }

  // Abort if the character is a special key.
  if (IS_SPECIAL(cap->nchar)) {
    rs_clearopbeep(cap->oap);
    return;
  }

  // Visual mode "r"
  if (VIsual_active) {
    if (got_int) {
      got_int = false;
    }
    if (had_ctrl_v) {
      // Use a special (negative) number to make a difference between a
      // literal CR or NL and a line break.
      if (cap->nchar == CAR) {
        cap->nchar = REPLACE_CR_NCHAR;
      } else if (cap->nchar == NL) {
        cap->nchar = REPLACE_NL_NCHAR;
      }
    }
    rs_nv_operator(cap);
    return;
  }

  // Break tabs, etc.
  if (virtual_active(curwin)) {
    if (u_save_cursor() == false) {
      return;
    }
    if (gchar_cursor() == NUL) {
      // Add extra space and put the cursor on the first one.
      coladvance_force((colnr_T)(getviscol() + cap->count1));
      assert(cap->count1 <= INT_MAX);
      curwin->w_cursor.col -= (colnr_T)cap->count1;
    } else if (gchar_cursor() == TAB) {
      coladvance_force(getviscol());
    }
  }

  // Abort if not enough characters to replace.
  if ((size_t)get_cursor_pos_len() < (unsigned)cap->count1
      || (mb_charlen(get_cursor_pos_ptr()) < cap->count1)) {
    rs_clearopbeep(cap->oap);
    return;
  }

  // Replacing with a TAB is done by edit() when it is complicated because
  // 'expandtab' or 'smarttab' is set.  CTRL-V TAB inserts a literal TAB.
  // Other characters are done below to avoid problems with things like
  // CTRL-V 048 (for edit() this would be R CTRL-V 0 ESC).
  if (had_ctrl_v != Ctrl_V && cap->nchar == '\t' && (curbuf->b_p_et || p_sta)) {
    stuffnumReadbuff(cap->count1);
    stuffcharReadbuff('R');
    stuffcharReadbuff('\t');
    stuffcharReadbuff(ESC);
    return;
  }

  // save line for undo
  if (u_save_cursor() == false) {
    return;
  }

  if (had_ctrl_v != Ctrl_V && (cap->nchar == '\r' || cap->nchar == '\n')) {
    // Replace character(s) by a single newline.
    // Strange vi behaviour: Only one newline is inserted.
    // Delete the characters here.
    // Insert the newline with an insert command, takes care of
    // autoindent.      The insert command depends on being on the last
    // character of a line or not.
    del_chars(cap->count1, false);        // delete the characters
    stuffcharReadbuff('\r');
    stuffcharReadbuff(ESC);

    // Give 'r' to edit(), to get the redo command right.
    invoke_edit(cap, true, 'r', false);
  } else {
    rs_prep_redo(cap->oap->regname, cap->count1, NUL, 'r', NUL, had_ctrl_v, 0);

    curbuf->b_op_start = curwin->w_cursor;
    const int old_State = State;

    if (cap->nchar_len > 0) {
      AppendToRedobuff(cap->nchar_composing);
    } else {
      AppendCharToRedobuff(cap->nchar);
    }

    // This is slow, but it handles replacing a single-byte with a
    // multi-byte and the other way around.  Also handles adding
    // composing characters for utf-8.
    for (int n = cap->count1; n > 0; n--) {
      State = MODE_REPLACE;
      if (cap->nchar == Ctrl_E || cap->nchar == Ctrl_Y) {
        int c = ins_copychar(curwin->w_cursor.lnum
                             + (cap->nchar == Ctrl_Y ? -1 : 1));
        if (c != NUL) {
          ins_char(c);
        } else {
          // will be decremented further down
          curwin->w_cursor.col++;
        }
      } else {
        if (cap->nchar_len) {
          ins_char_bytes(cap->nchar_composing, (size_t)cap->nchar_len);
        } else {
          ins_char(cap->nchar);
        }
      }
      State = old_State;
    }
    curwin->w_cursor.col--;         // cursor on the last replaced char
    // if the character on the left of the current cursor is a multi-byte
    // character, move two characters left
    mb_adjust_cursor();
    curbuf->b_op_end = curwin->w_cursor;
    curwin->w_set_curswant = true;
    set_last_insert(cap->nchar);
  }

  rs_foldUpdateAfterInsert();
}

/// "R" (cap->arg is false) and "gR" (cap->arg is true) (implementation).
static void nv_Replace_impl(cmdarg_T *cap)
{
  if (VIsual_active) {          // "R" is replace lines
    cap->cmdchar = 'c';
    cap->nchar = NUL;
    VIsual_mode_orig = VIsual_mode;     // remember original area for gv
    VIsual_mode = 'V';
    rs_nv_operator(cap);
    return;
  }

  if (rs_checkclearopq(cap->oap)) {
    return;
  }

  if (!MODIFIABLE(curbuf)) {
    emsg(_(e_modifiable));
  } else {
    if (virtual_active(curwin)) {
      coladvance(curwin, getviscol());
    }
    invoke_edit(cap, false, cap->arg ? 'V' : 'R', false);
  }
}

/// "gr" (implementation).
static void nv_vreplace_impl(cmdarg_T *cap)
{
  if (VIsual_active) {
    cap->cmdchar = 'r';
    cap->nchar = cap->extra_char;
    rs_nv_replace(cap);         // Do same as "r" in Visual mode for now
    return;
  }

  if (rs_checkclearopq(cap->oap)) {
    return;
  }

  if (!MODIFIABLE(curbuf)) {
    emsg(_(e_modifiable));
  } else {
    if (cap->extra_char == Ctrl_V || cap->extra_char == Ctrl_Q) {
      // get another character
      cap->extra_char = get_literal(false);
    }
    if (cap->extra_char < ' ') {
      // Prefix a control character with CTRL-V to avoid it being used as
      // a command.
      stuffcharReadbuff(Ctrl_V);
    }
    stuffcharReadbuff(cap->extra_char);
    stuffcharReadbuff(ESC);
    if (virtual_active(curwin)) {
      coladvance(curwin, getviscol());
    }
    invoke_edit(cap, true, 'v', false);
  }
}

/// Swap case for "~" command, when it does not work like an operator.
static void n_swapchar(cmdarg_T *cap)
{
  bool did_change = false;

  if (rs_checkclearopq(cap->oap)) {
    return;
  }

  if (LINEEMPTY(curwin->w_cursor.lnum) && vim_strchr(p_ww, '~') == NULL) {
    rs_clearopbeep(cap->oap);
    return;
  }

  rs_prep_redo_cmd(cap);

  if (u_save_cursor() == false) {
    return;
  }

  pos_T startpos = curwin->w_cursor;
  for (int n = cap->count1; n > 0; n--) {
    did_change |= swapchar(cap->oap->op_type, &curwin->w_cursor);
    inc_cursor();
    if (gchar_cursor() == NUL) {
      if (vim_strchr(p_ww, '~') != NULL
          && curwin->w_cursor.lnum < curbuf->b_ml.ml_line_count) {
        curwin->w_cursor.lnum++;
        curwin->w_cursor.col = 0;
        if (n > 1) {
          if (u_savesub(curwin->w_cursor.lnum) == false) {
            break;
          }
          u_clearline(curbuf);
        }
      } else {
        break;
      }
    }
  }

  check_cursor(curwin);
  curwin->w_set_curswant = true;
  if (did_change) {
    changed_lines(curbuf, startpos.lnum, startpos.col, curwin->w_cursor.lnum + 1,
                  0, true);
    curbuf->b_op_start = startpos;
    curbuf->b_op_end = curwin->w_cursor;
    if (curbuf->b_op_end.col > 0) {
      curbuf->b_op_end.col--;
    }
  }
}

/// Move the cursor to the mark position
///
/// Wrapper to mark_move_to() that also handles normal mode command arguments.
/// @note  It will switch the buffer if neccesarry, move the cursor and set the
/// view depending on the given flags.
/// @param cap  command line arguments
/// @param flags for mark_move_to()
/// @param mark  mark
/// @return  The result of calling mark_move_to()
static MarkMoveRes nv_mark_move_to(cmdarg_T *cap, MarkMove flags, fmark_T *fm)
{
  MarkMoveRes res = mark_move_to(fm, flags);
  if (res & kMarkMoveFailed) {
    rs_clearop(cap->oap);
  }
  cap->oap->motion_type = flags & kMarkBeginLine ? kMTLineWise : kMTCharWise;
  if (cap->cmdchar == '`') {
    cap->oap->use_reg_one = true;
  }
  cap->oap->inclusive = false;  // ignored if not kMTCharWise
  curwin->w_set_curswant = true;
  return res;
}

/// Implementation of "s" and "S" commands.
static void nv_subst_impl(cmdarg_T *cap)
{
  if (bt_prompt(curbuf) && !prompt_curpos_editable()) {
    rs_clearopbeep(cap->oap);
    return;
  }
  if (VIsual_active) {  // "vs" and "vS" are the same as "vc"
    if (cap->cmdchar == 'S') {
      VIsual_mode_orig = VIsual_mode;
      VIsual_mode = 'V';
    }
    cap->cmdchar = 'c';
    nv_operator_impl(cap);
  } else {
    nv_optrans_impl(cap);
  }
}

/// Implementation of command translation.
static void nv_optrans_impl(cmdarg_T *cap)
{
  static const char *(ar[]) = { "dl", "dh", "d$", "c$", "cl", "cc", "yy",
                                ":s\r" };
  static const char *str = "xXDCsSY&";

  if (!rs_checkclearopq(cap->oap)) {
    if (cap->count0) {
      stuffnumReadbuff(cap->count0);
    }
    stuffReadbuff(ar[strchr(str, (char)cap->cmdchar) - str]);
  }
  cap->opcount = 0;
}

/// Internal implementation of nv_visual.
static void nv_visual_impl(cmdarg_T *cap)
{
  if (cap->cmdchar == Ctrl_Q) {
    cap->cmdchar = Ctrl_V;
  }

  // 'v', 'V' and CTRL-V can be used while an operator is pending to make it
  // charwise, linewise, or blockwise.
  if (cap->oap->op_type != OP_NOP) {
    motion_force = cap->oap->motion_force = cap->cmdchar;
    finish_op = false;          // operator doesn't finish now but later
    return;
  }

  VIsual_select = cap->arg;
  if (VIsual_active) {      // change Visual mode
    if (VIsual_mode == cap->cmdchar) {      // stop visual mode
      end_visual_mode();
    } else {                                  // toggle char/block mode
                                              //           or char/line mode
      VIsual_mode = cap->cmdchar;
      showmode();
      may_trigger_modechanged();
    }
    redraw_curbuf_later(UPD_INVERTED);  // update the inversion
  } else {                // start Visual mode
    if (cap->count0 > 0 && resel_VIsual_mode != NUL) {
      // use previously selected part
      VIsual = curwin->w_cursor;

      VIsual_active = true;
      VIsual_reselect = true;
      if (!cap->arg) {
        // start Select mode when 'selectmode' contains "cmd"
        rs_may_start_select('c');
      }
      setmouse();
      if (p_smd && msg_silent == 0) {
        redraw_cmdline = true;              // show visual mode later
      }
      // For V and ^V, we multiply the number of lines even if there
      // was only one -- webb
      if (resel_VIsual_mode != 'v' || resel_VIsual_line_count > 1) {
        curwin->w_cursor.lnum += resel_VIsual_line_count * cap->count0 - 1;
        check_cursor(curwin);
      }
      VIsual_mode = resel_VIsual_mode;
      if (VIsual_mode == 'v') {
        if (resel_VIsual_line_count <= 1) {
          update_curswant_force();
          assert(cap->count0 >= INT_MIN && cap->count0 <= INT_MAX);
          curwin->w_curswant += resel_VIsual_vcol * cap->count0;
          if (*p_sel != 'e') {
            curwin->w_curswant--;
          }
        } else {
          curwin->w_curswant = resel_VIsual_vcol;
        }
        coladvance(curwin, curwin->w_curswant);
      }
      if (resel_VIsual_vcol == MAXCOL) {
        curwin->w_curswant = MAXCOL;
        coladvance(curwin, MAXCOL);
      } else if (VIsual_mode == Ctrl_V) {
        // Update curswant on the original line, that is where "col" is valid.
        linenr_T lnum = curwin->w_cursor.lnum;
        curwin->w_cursor.lnum = VIsual.lnum;
        update_curswant_force();
        assert(cap->count0 >= INT_MIN && cap->count0 <= INT_MAX);
        curwin->w_curswant += resel_VIsual_vcol * cap->count0 - 1;
        curwin->w_cursor.lnum = lnum;
        if (*p_sel == 'e') {
          curwin->w_curswant++;
        }
        coladvance(curwin, curwin->w_curswant);
      } else {
        curwin->w_set_curswant = true;
      }
      redraw_curbuf_later(UPD_INVERTED);  // show the inversion
    } else {
      if (!cap->arg) {
        // start Select mode when 'selectmode' contains "cmd"
        rs_may_start_select('c');
      }
      n_start_visual_mode(cap->cmdchar);
      if (VIsual_mode != 'V' && *p_sel == 'e') {
        cap->count1++;          // include one more char
      } else {
        VIsual_select_exclu_adj = false;
      }
      if (cap->count0 > 0 && --cap->count1 > 0) {
        // With a count select that many characters or lines.
        if (VIsual_mode == 'v' || VIsual_mode == Ctrl_V) {
          rs_nv_right(cap);
        } else if (VIsual_mode == 'V') {
          rs_nv_down(cap);
        }
      }
    }
  }
}

/// Start Visual mode "c".
/// Should set VIsual_select before calling this.
static void n_start_visual_mode(int c)
{
  VIsual_mode = c;
  VIsual_active = true;
  VIsual_reselect = true;
  // Corner case: the 0 position in a tab may change when going into
  // virtualedit.  Recalculate curwin->w_cursor to avoid bad highlighting.
  //
  if (c == Ctrl_V && (get_ve_flags(curwin) & kOptVeFlagBlock) && gchar_cursor() == TAB) {
    validate_virtcol(curwin);
    coladvance(curwin, curwin->w_virtcol);
  }
  VIsual = curwin->w_cursor;

  rs_foldAdjustVisual();

  may_trigger_modechanged();
  setmouse();
  // Check for redraw after changing the state.
  conceal_check_cursor_line();

  if (p_smd && msg_silent == 0) {
    redraw_cmdline = true;      // show visual mode later
  }
  // Only need to redraw this line, unless still need to redraw an old
  // Visual area (when 'lazyredraw' is set).
  if (curwin->w_redr_type < UPD_INVERTED) {
    curwin->w_old_cursor_lnum = curwin->w_cursor.lnum;
    curwin->w_old_visual_lnum = curwin->w_cursor.lnum;
  }
  redraw_curbuf_later(UPD_VALID);
}

/// "g0", "g^" : Like "0" and "^" but for screen lines.
/// "gm": middle of "g0" and "g$".
void nv_g_home_m_cmd(cmdarg_T *cap)
{
  int i;
  const bool flag = cap->nchar == '^';

  cap->oap->motion_type = kMTCharWise;
  cap->oap->inclusive = false;
  if (curwin->w_p_wrap && curwin->w_view_width != 0) {
    int width1 = curwin->w_view_width - win_col_off(curwin);
    int width2 = width1 + win_col_off2(curwin);

    validate_virtcol(curwin);
    i = 0;
    if (curwin->w_virtcol >= (colnr_T)width1 && width2 > 0) {
      i = (curwin->w_virtcol - width1) / width2 * width2 + width1;
    }

    // When ending up below 'smoothscroll' marker, move just beyond it so
    // that skipcol is not adjusted later.
    if (curwin->w_skipcol > 0 && curwin->w_cursor.lnum == curwin->w_topline) {
      int overlap = sms_marker_overlap(curwin, curwin->w_view_width - width2);
      if (overlap > 0 && i == curwin->w_skipcol) {
        i += overlap;
      }
    }
  } else {
    i = curwin->w_leftcol;
  }
  // Go to the middle of the screen line.  When 'number' or
  // 'relativenumber' is on and lines are wrapping the middle can be more
  // to the left.
  if (cap->nchar == 'm') {
    i += (curwin->w_view_width - win_col_off(curwin)
          + ((curwin->w_p_wrap && i > 0) ? win_col_off2(curwin) : 0)) / 2;
  }
  coladvance(curwin, (colnr_T)i);
  if (flag) {
    do {
      i = gchar_cursor();
    } while (ascii_iswhite(i) && oneright() == OK);
    curwin->w_valid &= ~VALID_WCOL;
  }
  curwin->w_set_curswant = true;
  if (rs_hasAnyFolding(curwin)) {
    validate_cheight(curwin);
    if (curwin->w_cline_folded) {
      update_curswant_force();
    }
  }
  adjust_skipcol();
}

/// "g$" : Like "$" but for screen lines.
static void nv_g_dollar_cmd(cmdarg_T *cap)
{
  oparg_T *oap = cap->oap;
  int i;
  int col_off = win_col_off(curwin);
  const bool flag = cap->nchar == K_END || cap->nchar == K_KEND;

  oap->motion_type = kMTCharWise;
  oap->inclusive = true;
  if (curwin->w_p_wrap && curwin->w_view_width != 0) {
    curwin->w_curswant = MAXCOL;              // so we stay at the end
    if (cap->count1 == 1) {
      int width1 = curwin->w_view_width - col_off;
      int width2 = width1 + win_col_off2(curwin);

      validate_virtcol(curwin);
      i = width1 - 1;
      if (curwin->w_virtcol >= (colnr_T)width1) {
        i += ((curwin->w_virtcol - width1) / width2 + 1) * width2;
      }
      coladvance(curwin, (colnr_T)i);

      // Make sure we stick in this column.
      update_curswant_force();
      if (curwin->w_cursor.col > 0 && curwin->w_p_wrap) {
        // Check for landing on a character that got split at
        // the end of the line.  We do not want to advance to
        // the next screen line.
        if (curwin->w_virtcol > (colnr_T)i) {
          curwin->w_cursor.col--;
        }
      }
    } else if (nv_screengo(oap, FORWARD, cap->count1 - 1, false) == false) {
      rs_clearopbeep(oap);
    }
  } else {
    if (cap->count1 > 1) {
      // if it fails, let the cursor still move to the last char
      cursor_down(cap->count1 - 1, false);
    }
    i = curwin->w_leftcol + curwin->w_view_width - col_off - 1;
    coladvance(curwin, (colnr_T)i);

    // if the character doesn't fit move one back
    if (curwin->w_cursor.col > 0 && utf_ptr2cells(get_cursor_pos_ptr()) > 1) {
      colnr_T vcol;

      getvvcol(curwin, &curwin->w_cursor, NULL, NULL, &vcol);
      if (vcol >= curwin->w_leftcol + curwin->w_view_width - col_off) {
        curwin->w_cursor.col--;
      }
    }

    // Make sure we stick in this column.
    update_curswant_force();
  }
  if (flag) {
    do {
      i = gchar_cursor();
    } while (ascii_iswhite_or_nul(i) && oneleft() == OK);
    curwin->w_valid &= ~VALID_WCOL;
  }
}

/// Commands starting with "g" (implementation).
static void nv_g_cmd_impl(cmdarg_T *cap)
{
  oparg_T *oap = cap->oap;
  int i;

  switch (cap->nchar) {
  // "g^A/g^X": Sequentially increment visually selected region.
  case Ctrl_A:
  case Ctrl_X:
    if (VIsual_active) {
      cap->arg = true;
      cap->cmdchar = cap->nchar;
      cap->nchar = NUL;
      nv_addsub(cap);
    } else {
      rs_clearopbeep(oap);
    }
    break;

  // "gR": Enter virtual replace mode.
  case 'R':
    cap->arg = true;
    rs_nv_Replace(cap);
    break;

  case 'r':
    rs_nv_vreplace(cap);
    break;

  case '&':
    do_cmdline_cmd("%s//~/&");
    break;

  // "gv": Reselect the previous Visual area.  If Visual already active,
  //       exchange previous and current Visual area.
  case 'v':
    rs_nv_gv_cmd(cap);
    break;
  // "gV": Don't reselect the previous Visual area after a Select mode mapping of menu.
  case 'V':
    VIsual_reselect = false;
    break;

  // "gh":  start Select mode.
  // "gH":  start Select line mode.
  // "g^H": start Select block mode.
  case K_BS:
    cap->nchar = Ctrl_H;
    FALLTHROUGH;
  case 'h':
  case 'H':
  case Ctrl_H:
    cap->cmdchar = cap->nchar + ('v' - 'h');
    cap->arg = true;
    rs_nv_visual(cap);
    break;

  // "gn", "gN" visually select next/previous search match
  // "gn" selects next match
  // "gN" selects previous match
  case 'N':
  case 'n':
    if (!current_search(cap->count1, cap->nchar == 'n')) {
      rs_clearopbeep(oap);
    }
    break;

  // "gj" and "gk" two new funny movement keys -- up and down
  // movement based on *screen* line rather than *file* line.
  case 'j':
  case K_DOWN:
    // with 'nowrap' it works just like the normal "j" command.
    if (!curwin->w_p_wrap) {
      oap->motion_type = kMTLineWise;
      i = cursor_down(cap->count1, oap->op_type == OP_NOP);
    } else {
      i = nv_screengo(oap, FORWARD, cap->count1, false);
    }
    if (!i) {
      rs_clearopbeep(oap);
    }
    break;

  case 'k':
  case K_UP:
    // with 'nowrap' it works just like the normal "k" command.
    if (!curwin->w_p_wrap) {
      oap->motion_type = kMTLineWise;
      i = cursor_up(cap->count1, oap->op_type == OP_NOP);
    } else {
      i = nv_screengo(oap, BACKWARD, cap->count1, false);
    }
    if (!i) {
      rs_clearopbeep(oap);
    }
    break;

  // "gJ": join two lines without inserting a space.
  case 'J':
    rs_nv_join(cap);
    break;

  // "g0", "g^" : Like "0" and "^" but for screen lines.
  // "gm": middle of "g0" and "g$".
  case '^':
  case '0':
  case 'm':
  case K_HOME:
  case K_KHOME:
    nv_g_home_m_cmd(cap);
    break;

  case 'M':
    oap->motion_type = kMTCharWise;
    oap->inclusive = false;
    i = linetabsize(curwin, curwin->w_cursor.lnum);
    if (cap->count0 > 0 && cap->count0 <= 100) {
      coladvance(curwin, (colnr_T)(i * cap->count0 / 100));
    } else {
      coladvance(curwin, (colnr_T)(i / 2));
    }
    curwin->w_set_curswant = true;
    break;

  // "g_": to the last non-blank character in the line or <count> lines downward.
  case '_':
    rs_nv_g_underscore_cmd(cap);
    break;

  // "g$" : Like "$" but for screen lines.
  case '$':
  case K_END:
  case K_KEND:
    nv_g_dollar_cmd(cap);
    break;

  // "g*" and "g#", like "*" and "#" but without using "\<" and "\>"
  case '*':
  case '#':
#if POUND != '#'
  case POUND:           // pound sign (sometimes equal to '#')
#endif
  case Ctrl_RSB:                // :tag or :tselect for current identifier
  case ']':                     // :tselect for current identifier
    rs_nv_ident(cap);
    break;

  // ge and gE: go back to end of word
  case 'e':
  case 'E':
    oap->motion_type = kMTCharWise;
    curwin->w_set_curswant = true;
    oap->inclusive = true;
    if (bckend_word(cap->count1, cap->nchar == 'E', false) == false) {
      rs_clearopbeep(oap);
    }
    break;

  // "g CTRL-G": display info about cursor position
  case Ctrl_G:
    cursor_pos_info(NULL);
    break;

  // "gi": start Insert at the last position.
  case 'i':
    rs_nv_gi_cmd(cap);
    break;

  // "gI": Start insert in column 1.
  case 'I':
    beginline(0);
    if (!rs_checkclearopq(oap)) {
      invoke_edit(cap, false, 'g', false);
    }
    break;

  // "gf": goto file, edit file under cursor
  // "]f" and "[f": can also be used.
  case 'f':
  case 'F':
    nv_gotofile(cap);
    break;

  // "g'm" and "g`m": jump to mark without setting pcmark
  case '\'':
    cap->arg = true;
    FALLTHROUGH;
  case '`':
    rs_nv_gomark(cap);
    break;

  // "gs": Goto sleep.
  case 's':
    do_sleep(cap->count1 * 1000, false);
    break;

  // "ga": Display the ascii value of the character under the
  // cursor.    It is displayed in decimal, hex, and octal. -- webb
  case 'a':
    rs_do_ascii(NULL);
    break;

  // "g8": Display the bytes used for the UTF-8 character under the
  // cursor.    It is displayed in hex.
  // "8g8" finds illegal byte sequence.
  case '8':
    if (cap->count0 == 8) {
      utf_find_illegal();
    } else {
      show_utf8();
    }
    break;
  // "g<": show scrollback text
  case '<':
    show_sb_text();
    break;

  // "gg": Goto the first line in file.  With a count it goes to
  // that line number like for "G". -- webb
  case 'g':
    cap->arg = false;
    rs_nv_goto(cap);
    break;

  //  Two-character operators:
  //  "gq"       Format text
  //  "gw"       Format text and keep cursor position
  //  "g~"       Toggle the case of the text.
  //  "gu"       Change text to lower case.
  //  "gU"       Change text to upper case.
  //  "g?"       rot13 encoding
  //  "g@"       call 'operatorfunc'
  case 'q':
  case 'w':
    oap->cursor_start = curwin->w_cursor;
    FALLTHROUGH;
  case '~':
  case 'u':
  case 'U':
  case '?':
  case '@':
    rs_nv_operator(cap);
    break;

  // "gd": Find first occurrence of pattern under the cursor in the current function
  // "gD": idem, but in the current file.
  case 'd':
  case 'D':
    nv_gd(oap, cap->nchar, cap->count0);
    break;

  // g<*Mouse> : <C-*mouse>
  case K_MIDDLEMOUSE:
  case K_MIDDLEDRAG:
  case K_MIDDLERELEASE:
  case K_LEFTMOUSE:
  case K_LEFTDRAG:
  case K_LEFTRELEASE:
  case K_MOUSEMOVE:
  case K_RIGHTMOUSE:
  case K_RIGHTDRAG:
  case K_RIGHTRELEASE:
  case K_X1MOUSE:
  case K_X1DRAG:
  case K_X1RELEASE:
  case K_X2MOUSE:
  case K_X2DRAG:
  case K_X2RELEASE:
    mod_mask = MOD_MASK_CTRL;
    do_mouse(oap, cap->nchar, BACKWARD, cap->count1, 0);
    break;

  case K_IGNORE:
    break;

  // "gP" and "gp": same as "P" and "p" but leave cursor just after new text
  case 'p':
  case 'P':
    rs_nv_put(cap);
    break;

  // "go": goto byte count from start of buffer
  case 'o':
    oap->inclusive = false;
    goto_byte(cap->count0);
    break;

  // "gQ": improved Ex mode
  case 'Q':
    if (!rs_check_text_locked(cap->oap) && !rs_checkclearopq(oap)) {
      do_exmode();
    }
    break;

  case ',':
    rs_nv_pcmark(cap);
    break;

  case ';':
    cap->count1 = -cap->count1;
    rs_nv_pcmark(cap);
    break;

  case 't':
    if (!rs_checkclearop(oap)) {
      goto_tabpage(cap->count0);
    }
    break;
  case 'T':
    if (!rs_checkclearop(oap)) {
      goto_tabpage(-cap->count1);
    }
    break;

  case TAB:
    if (!rs_checkclearop(oap) && !goto_tabpage_lastused()) {
      rs_clearopbeep(oap);
    }
    break;

  case '+':
  case '-':   // "g+" and "g-": undo or redo along the timeline
    if (!rs_checkclearopq(oap)) {
      undo_time(cap->nchar == '-' ? -cap->count1 : cap->count1,
                false, false, false);
    }
    break;

  default:
    rs_clearopbeep(oap);
    break;
  }
}

/// Handle "o" and "O" commands.
static void n_opencmd(cmdarg_T *cap)
{
  if (rs_checkclearopq(cap->oap)) {
    return;
  }

  if (cap->cmdchar == 'O') {
    // Open above the first line of a folded sequence of lines
    hasFolding(curwin, curwin->w_cursor.lnum,
               &curwin->w_cursor.lnum, NULL);
  } else {
    // Open below the last line of a folded sequence of lines
    hasFolding(curwin, curwin->w_cursor.lnum,
               NULL, &curwin->w_cursor.lnum);
  }
  // trigger TextChangedI for the 'o/O' command
  curbuf->b_last_changedtick_i = buf_get_changedtick(curbuf);
  if (u_save(curwin->w_cursor.lnum - (cap->cmdchar == 'O' ? 1 : 0),
             curwin->w_cursor.lnum + (cap->cmdchar == 'o' ? 1 : 0))
      && open_line(cap->cmdchar == 'O' ? BACKWARD : FORWARD,
                   has_format_option(FO_OPEN_COMS) ? OPENLINE_DO_COM : 0,
                   0, NULL)) {
    if (win_cursorline_standout(curwin)) {
      // force redraw of cursorline
      curwin->w_valid &= ~VALID_CROW;
    }
    invoke_edit(cap, false, cap->cmdchar, true);
  }
}

/// "." command: redo last change (implementation).
static void nv_dot_impl(cmdarg_T *cap)
{
  if (rs_checkclearopq(cap->oap)) {
    return;
  }

  // If "restart_edit" is true, the last but one command is repeated
  // instead of the last command (inserting text). This is used for
  // CTRL-O <.> in insert mode.
  if (start_redo(cap->count0, restart_edit != 0 && !arrow_used) == false) {
    rs_clearopbeep(cap->oap);
  }
}

/// CTRL-R: undo undo or specify register in select mode (implementation)
static void nv_redo_or_register_impl(cmdarg_T *cap)
{
  if (VIsual_select && VIsual_active) {
    // Get register name
    no_mapping++;
    int reg = plain_vgetc();
    LANGMAP_ADJUST(reg, true);
    no_mapping--;

    if (reg == '"') {
      // the unnamed register is 0
      reg = 0;
    }

    VIsual_select_reg = valid_yank_reg(reg, true) ? reg : 0;
    return;
  }

  if (rs_checkclearopq(cap->oap)) {
    return;
  }

  u_redo(cap->count1);
  curwin->w_set_curswant = true;
}

/// Handle "U" command (implementation).
static void nv_Undo_impl(cmdarg_T *cap)
{
  // In Visual mode and typing "gUU" triggers an operator
  if (cap->oap->op_type == OP_UPPER || VIsual_active) {
    // translate "gUU" to "gUgU"
    cap->cmdchar = 'g';
    cap->nchar = 'U';
    rs_nv_operator(cap);
    return;
  }

  if (rs_checkclearopq(cap->oap)) {
    return;
  }

  u_undoline();
  curwin->w_set_curswant = true;
}

/// Implementation of '~' command.
static void nv_tilde_impl(cmdarg_T *cap)
{
  if (!p_to && !VIsual_active && cap->oap->op_type != OP_TILDE) {
    if (bt_prompt(curbuf) && !prompt_curpos_editable()) {
      rs_clearopbeep(cap->oap);
      return;
    }
    n_swapchar(cap);
  } else {
    nv_operator_impl(cap);
  }
}

/// Implementation of operator command.
static void nv_operator_impl(cmdarg_T *cap)
{
  int op_type = get_op_type(cap->cmdchar, cap->nchar);

  if (bt_prompt(curbuf) && op_is_change(op_type)
      && !prompt_curpos_editable()) {
    rs_clearopbeep(cap->oap);
    return;
  }

  if (op_type == cap->oap->op_type) {       // double operator works on lines
    rs_nv_lineop(cap);
  } else if (!rs_checkclearop(cap->oap)) {
    cap->oap->start = curwin->w_cursor;
    cap->oap->op_type = op_type;
    set_op_var(op_type);
  }
}

/// Set v:operator to the characters for "optype".
static void set_op_var(int optype)
{
  if (optype == OP_NOP) {
    set_vim_var_string(VV_OP, NULL, 0);
  } else {
    char opchars[3];
    int opchar0 = get_op_char(optype);
    assert(opchar0 >= 0 && opchar0 <= UCHAR_MAX);
    opchars[0] = (char)opchar0;

    int opchar1 = get_extra_op_char(optype);
    assert(opchar1 >= 0 && opchar1 <= UCHAR_MAX);
    opchars[1] = (char)opchar1;

    opchars[2] = NUL;
    set_vim_var_string(VV_OP, opchars, -1);
  }
}

/// Handle linewise operator "dd", "yy", etc.
///
/// Used after a movement command: If the cursor ends up on the NUL after the
/// end of the line, may move it back to the last character and make the motion
/// inclusive.
static void adjust_cursor(oparg_T *oap)
{
  // The cursor cannot remain on the NUL when:
  // - the column is > 0
  // - not in Visual mode or 'selection' is "o"
  // - 'virtualedit' is not "all" and not "onemore".
  if (curwin->w_cursor.col > 0 && gchar_cursor() == NUL
      && (!VIsual_active || *p_sel == 'o')
      && !virtual_active(curwin)
      && (get_ve_flags(curwin) & kOptVeFlagOnemore) == 0) {
    curwin->w_cursor.col--;
    // prevent cursor from moving on the trail byte
    mb_adjust_cursor();
    oap->inclusive = true;
  }
}

/// In exclusive Visual mode, may include the last character.
static void adjust_for_sel(cmdarg_T *cap)
{
  if (VIsual_active && cap->oap->inclusive && *p_sel == 'e'
      && gchar_cursor() != NUL && lt(VIsual, curwin->w_cursor)) {
    inc_cursor();
    cap->oap->inclusive = false;
    VIsual_select_exclu_adj = true;
  }
}

/// Move position "*pp" back one character for 'selection' == "exclusive".
///
/// @return  true when backed up to the previous line.
bool unadjust_for_sel_inner(pos_T *pp)
{
  VIsual_select_exclu_adj = false;

  if (pp->coladd > 0) {
    pp->coladd--;
  } else if (pp->col > 0) {
    pp->col--;
    mark_mb_adjustpos(curbuf, pp);
    if (virtual_active(curwin)) {
      colnr_T cs, ce;
      getvcol(curwin, pp, &cs, NULL, &ce);
      pp->coladd = ce - cs;
    }
  } else if (pp->lnum > 1) {
    pp->lnum--;
    pp->col = ml_get_len(pp->lnum);
    return true;
  }

  return false;
}

/// SELECT key in Normal or Visual mode: end of Select mode mapping (implementation).
static void nv_select_impl(cmdarg_T *cap)
{
  if (VIsual_active) {
    VIsual_select = true;
    VIsual_select_reg = 0;
  } else if (VIsual_reselect) {
    cap->nchar = 'v';               // fake "gv" command
    cap->arg = true;
    rs_nv_g_cmd(cap);
  }
}

/// Move the cursor for the "A" command.
void set_cursor_for_append_to_line(void)
{
  curwin->w_set_curswant = true;
  if (get_ve_flags(curwin) == kOptVeFlagAll) {
    const int save_State = State;
    // Pretend Insert mode here to allow the cursor on the
    // character past the end of the line
    State = MODE_INSERT;
    coladvance(curwin, MAXCOL);
    State = save_State;
  } else {
    curwin->w_cursor.col += (colnr_T)strlen(get_cursor_pos_ptr());
  }
}

/// Invoke edit() and take care of "restart_edit" and the return value.
///
/// @param repl  "r" or "gr" command
static void invoke_edit(cmdarg_T *cap, int repl, int cmd, int startln)
{
  int restart_edit_save = 0;

  // Complicated: When the user types "a<C-O>a" we don't want to do Insert
  // mode recursively.  But when doing "a<C-O>." or "a<C-O>rx" we do allow
  // it.
  if (repl || !stuff_empty()) {
    restart_edit_save = restart_edit;
  } else {
    restart_edit_save = 0;
  }

  // Always reset "restart_edit", this is not a restarted edit.
  restart_edit = 0;

  // Reset Changedtick_i, so that TextChangedI will only be triggered for stuff
  // from insert mode, for 'o/O' this has already been done in n_opencmd
  if (cap->cmdchar != 'O' && cap->cmdchar != 'o') {
    curbuf->b_last_changedtick_i = buf_get_changedtick(curbuf);
  }
  if (edit(cmd, startln, cap->count1)) {
    cap->retval |= CA_COMMAND_BUSY;
  }

  if (restart_edit == 0) {
    restart_edit = restart_edit_save;
  }
}

/// "a" or "i" while an operator is pending or in Visual mode: object motion (implementation).
static void nv_object_impl(cmdarg_T *cap)
{
  bool flag;
  bool include;

  if (cap->cmdchar == 'i') {
    include = false;        // "ix" = inner object: exclude white space
  } else {
    include = true;         // "ax" = an object: include white space
  }
  // Make sure (), [], {} and <> are in 'matchpairs'
  char *mps_save = curbuf->b_p_mps;
  curbuf->b_p_mps = "(:),{:},[:],<:>";

  switch (cap->nchar) {
  case 'w':       // "aw" = a word
    flag = current_word(cap->oap, cap->count1, include, false);
    break;
  case 'W':       // "aW" = a WORD
    flag = current_word(cap->oap, cap->count1, include, true);
    break;
  case 'b':       // "ab" = a braces block
  case '(':
  case ')':
    flag = current_block(cap->oap, cap->count1, include, '(', ')');
    break;
  case 'B':       // "aB" = a Brackets block
  case '{':
  case '}':
    flag = current_block(cap->oap, cap->count1, include, '{', '}');
    break;
  case '[':       // "a[" = a [] block
  case ']':
    flag = current_block(cap->oap, cap->count1, include, '[', ']');
    break;
  case '<':       // "a<" = a <> block
  case '>':
    flag = current_block(cap->oap, cap->count1, include, '<', '>');
    break;
  case 't':       // "at" = a tag block (xml and html)
    // Do not adjust oap->end in do_pending_operator()
    // otherwise there are different results for 'dit'
    // (note leading whitespace in last line):
    // 1) <b>      2) <b>
    //    foobar      foobar
    //    </b>            </b>
    cap->retval |= CA_NO_ADJ_OP_END;
    flag = current_tagblock(cap->oap, cap->count1, include);
    break;
  case 'p':       // "ap" = a paragraph
    flag = current_par(cap->oap, cap->count1, include, 'p');
    break;
  case 's':       // "as" = a sentence
    flag = current_sent(cap->oap, cap->count1, include);
    break;
  case '"':       // "a"" = a double quoted string
  case '\'':       // "a'" = a single quoted string
  case '`':       // "a`" = a backtick quoted string
    flag = current_quote(cap->oap, cap->count1, include,
                         cap->nchar);
    break;
  default:
    flag = false;
    break;
  }

  curbuf->b_p_mps = mps_save;
  if (!flag) {
    rs_clearopbeep(cap->oap);
  }
  adjust_cursor_col();
  curwin->w_set_curswant = true;
}

/// "q" command: Start/stop recording.
/// "q:", "q/", "q?": edit command-line in command-line window.
static void nv_record(cmdarg_T *cap)
{
  if (cap->oap->op_type == OP_FORMAT) {
    // "gqq" is the same as "gqgq": format line
    cap->cmdchar = 'g';
    cap->nchar = 'q';
    rs_nv_operator(cap);
    return;
  }

  if (rs_checkclearop(cap->oap)) {
    return;
  }

  if (cap->nchar == ':' || cap->nchar == '/' || cap->nchar == '?') {
    if (cmdwin_type != 0) {
      emsg(_(e_cmdline_window_already_open));
      return;
    }
    stuffcharReadbuff(cap->nchar);
    stuffcharReadbuff(K_CMDWIN);
  } else {
    // (stop) recording into a named register, unless executing a
    // register.
    if (reg_executing == 0 && do_record(cap->nchar) == FAIL) {
      rs_clearopbeep(cap->oap);
    }
  }
}

/// Handle the "@r" command (implementation).
static void nv_at_impl(cmdarg_T *cap)
{
  if (rs_checkclearop(cap->oap)) {
    return;
  }
  if (cap->nchar == '=') {
    if (get_expr_register() == NUL) {
      return;
    }
  }
  while (cap->count1-- && !got_int) {
    if (do_execreg(cap->nchar, false, false, false) == false) {
      rs_clearopbeep(cap->oap);
      break;
    }
    line_breakcheck();
  }
}

/// Handle "J" or "gJ" command (implementation).
static void nv_join_impl(cmdarg_T *cap)
{
  if (VIsual_active) {  // join the visual lines
    rs_nv_operator(cap);
    return;
  }

  if (rs_checkclearop(cap->oap)) {
    return;
  }

  cap->count0 = MAX(cap->count0, 2);  // default for join is two lines!

  if (curwin->w_cursor.lnum + cap->count0 - 1 >
      curbuf->b_ml.ml_line_count) {
    // can't join when on the last line
    if (cap->count0 <= 2) {
      rs_clearopbeep(cap->oap);
      return;
    }
    cap->count0 = curbuf->b_ml.ml_line_count - curwin->w_cursor.lnum + 1;
  }

  rs_prep_redo(cap->oap->regname, cap->count0,
            NUL, cap->cmdchar, NUL, NUL, cap->nchar);
  do_join((size_t)cap->count0, cap->nchar == NUL, true, true, true);
}

/// "P", "gP", "p" and "gp" commands.
///
/// @param fix_indent  true for "[p", "[P", "]p" and "]P".
static void nv_put_opt(cmdarg_T *cap, bool fix_indent)
{
  yankreg_T *savereg = NULL;
  bool empty = false;
  bool was_visual = false;
  int dir;
  int flags = 0;
  const int save_fen = curwin->w_p_fen;

  if (cap->oap->op_type != OP_NOP) {
    // "dp" is ":diffput"
    if (cap->oap->op_type == OP_DELETE && cap->cmdchar == 'p') {
      rs_clearop(cap->oap);
      assert(cap->opcount >= 0);
      nv_diffgetput(true, (size_t)cap->opcount);
    } else {
      rs_clearopbeep(cap->oap);
    }
    return;
  }

  if (bt_prompt(curbuf) && !prompt_curpos_editable()) {
    if (curwin->w_cursor.lnum == curbuf->b_prompt_start.mark.lnum) {
      curwin->w_cursor.col = (int)strlen(prompt_text());
      // Since we've shifted the cursor to the first editable char. We want to
      // paste before that.
      cap->cmdchar = 'P';
    } else {
      rs_clearopbeep(cap->oap);
      return;
    }
  }

  if (fix_indent) {
    dir = (cap->cmdchar == ']' && cap->nchar == 'p')
          ? FORWARD : BACKWARD;
    flags |= PUT_FIXINDENT;
  } else {
    dir = (cap->cmdchar == 'P'
           || ((cap->cmdchar == 'g' || cap->cmdchar == 'z')
               && cap->nchar == 'P')) ? BACKWARD : FORWARD;
  }
  rs_prep_redo_cmd(cap);
  if (cap->cmdchar == 'g') {
    flags |= PUT_CURSEND;
  } else if (cap->cmdchar == 'z') {
    flags |= PUT_BLOCK_INNER;
  }

  if (VIsual_active) {
    // Putting in Visual mode: The put text replaces the selected
    // text.  First delete the selected text, then put the new text.
    // Need to save and restore the registers that the delete
    // overwrites if the old contents is being put.
    was_visual = true;
    int regname = cap->oap->regname;
    bool keep_registers = cap->cmdchar == 'P';
    // '+' and '*' could be the same selection
    bool clipoverwrite = (regname == '+' || regname == '*')
                         && (cb_flags & (kOptCbFlagUnnamed | kOptCbFlagUnnamedplus));
    if (regname == 0 || regname == '"' || clipoverwrite
        || ascii_isdigit(regname) || regname == '-') {
      // The delete might overwrite the register we want to put, save it first
      savereg = copy_register(regname);
    }

    // Temporarily disable folding, as deleting a fold marker may cause
    // the cursor to be included in a fold.
    curwin->w_p_fen = false;

    // To place the cursor correctly after a blockwise put, and to leave the
    // text in the correct position when putting over a selection with
    // 'virtualedit' and past the end of the line, we use the 'c' operator in
    // do_put(), which requires the visual selection to still be active.
    if (!VIsual_active || VIsual_mode == 'V' || regname != '.') {
      // Now delete the selected text. Avoid messages here.
      cap->cmdchar = 'd';
      cap->nchar = NUL;
      cap->oap->regname = keep_registers ? '_' : NUL;
      msg_silent++;
      rs_nv_operator(cap);
      do_pending_operator(cap, 0, false);
      empty = (curbuf->b_ml.ml_flags & ML_EMPTY);
      msg_silent--;

      // delete PUT_LINE_BACKWARD;
      cap->oap->regname = regname;
    }

    // When deleted a linewise Visual area, put the register as
    // lines to avoid it joined with the next line.  When deletion was
    // charwise, split a line when putting lines.
    if (VIsual_mode == 'V') {
      flags |= PUT_LINE;
    } else if (VIsual_mode == 'v') {
      flags |= PUT_LINE_SPLIT;
    }
    if (VIsual_mode == Ctrl_V && dir == FORWARD) {
      flags |= PUT_LINE_FORWARD;
    }
    dir = BACKWARD;
    if ((VIsual_mode != 'V'
         && curwin->w_cursor.col < curbuf->b_op_start.col)
        || (VIsual_mode == 'V'
            && curwin->w_cursor.lnum < curbuf->b_op_start.lnum)) {
      // cursor is at the end of the line or end of file, put
      // forward.
      dir = FORWARD;
    }
    // May have been reset in do_put().
    VIsual_active = true;
  }
  do_put(cap->oap->regname, savereg, dir, cap->count1, flags);

  // If a register was saved, free it
  if (savereg != NULL) {
    free_register(savereg);
    xfree(savereg);
  }

  if (was_visual) {
    if (save_fen) {
      curwin->w_p_fen = true;
    }
    // What to reselect with "gv"?  Selecting the just put text seems to
    // be the most useful, since the original text was removed.
    curbuf->b_visual.vi_start = curbuf->b_op_start;
    curbuf->b_visual.vi_end = curbuf->b_op_end;
    // need to adjust cursor position
    if (*p_sel == 'e') {
      inc(&curbuf->b_visual.vi_end);
    }
  }

  // When all lines were selected and deleted do_put() leaves an empty
  // line that needs to be deleted now.
  if (empty && *ml_get(curbuf->b_ml.ml_line_count) == NUL) {
    ml_delete_flags(curbuf->b_ml.ml_line_count, ML_DEL_MESSAGE);
    deleted_lines(curbuf->b_ml.ml_line_count + 1, 1);

    // If the cursor was in that line, move it to the end of the last
    // line.
    if (curwin->w_cursor.lnum > curbuf->b_ml.ml_line_count) {
      curwin->w_cursor.lnum = curbuf->b_ml.ml_line_count;
      coladvance(curwin, MAXCOL);
    }
  }
  auto_format(false, true);
}

/// "o" and "O" commands (implementation).
static void nv_open_impl(cmdarg_T *cap)
{
  // "do" is ":diffget"
  if (cap->oap->op_type == OP_DELETE && cap->cmdchar == 'o') {
    rs_clearop(cap->oap);
    assert(cap->opcount >= 0);
    nv_diffgetput(false, (size_t)cap->opcount);
  } else if (VIsual_active) {
    // switch start and end of visual/
    rs_v_swap_corners(cap->cmdchar);
  } else if (bt_prompt(curbuf) && curwin->w_cursor.lnum < curbuf->b_prompt_start.mark.lnum) {
    rs_clearopbeep(cap->oap);
  } else {
    n_opencmd(cap);
  }
}

static void nv_paste(cmdarg_T *cap) { paste_repeat(cap->count1); }

/// Handle an arbitrary event in normal mode
static void nv_event(cmdarg_T *cap)
{
  // Garbage collection should have been executed before blocking for events in
  // the `input_get` in `state_enter`, but we also disable it here in case the
  // `input_get` branch was not executed (!multiqueue_empty(loop.events), which
  // could have `may_garbage_collect` set to true in `normal_check`).
  //
  // That is because here we may run code that calls `input_get` later
  // (`f_confirm` or `get_keystroke` for example), but in these cases it is
  // not safe to perform garbage collection because there could be unreferenced
  // lists or dicts being used.
  may_garbage_collect = false;
  bool may_restart = (restart_edit != 0 || restart_VIsual_select != 0);
  state_handle_k_event();
  finish_op = false;
  if (may_restart) {
    // Tricky: if restart_edit was set before the handler we are in ctrl-o mode,
    // but if not, the event should be allowed to trigger :startinsert.
    cap->retval |= CA_COMMAND_BUSY;  // don't call edit() or restart Select now
  }
}

void normal_cmd(oparg_T *oap, bool toplevel)
{
  NormalState s;
  normal_state_init(&s);
  s.toplevel = toplevel;
  s.oa = *oap;
  normal_prepare(&s);
  normal_execute(&s.state, safe_vgetc());
  *oap = s.oa;
}
