#pragma once

#include "nvim/autocmd_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

/// Values for in_cinkeys()
enum {
  KEY_OPEN_FORW = 0x101,
  KEY_OPEN_BACK = 0x102,
  KEY_COMPLETE  = 0x103,  ///< end of completion
};

/// Values for change_indent()
enum {
  INDENT_SET = 1,  ///< set indent
  INDENT_INC = 2,  ///< increase indent
  INDENT_DEC = 3,  ///< decrease indent
};

/// flags for beginline()
enum {
  BL_WHITE = 1,  ///< cursor on first non-white in the line
  BL_SOL   = 2,  ///< use 'sol' option
  BL_FIX   = 4,  ///< don't leave cursor on a NUL
};

/// flags for insertchar()
enum {
  INSCHAR_FORMAT   = 1,   ///< force formatting
  INSCHAR_DO_COM   = 2,   ///< format comments
  INSCHAR_CTRLV    = 4,   ///< char typed just after CTRL-V
  INSCHAR_NO_FEX   = 8,   ///< don't use 'formatexpr'
  INSCHAR_COM_LIST = 16,  ///< format comments with list/2nd line indent
};

#include "edit.h.generated.h"

// Functions now implemented in Rust (src/nvim-rs/edit/)
// ins_tab: handles TAB insertion (tab.rs, export_name = "ins_tab")
bool ins_tab(void);
// ins_eol: handles CR/NL insertion (editing.rs, export_name = "ins_eol")
bool ins_eol(int c);
// ins_bs: handles backspace/delete in insert mode (backspace.rs)
bool ins_bs(int c, int mode, int *inserted_space_p);
// insertchar: character insertion with formatting (insertchar.rs, export_name = "insertchar")
void insertchar(int c, int flags, int second_indent);

// Phase 6 thin-wrapper eliminations: C body deleted, Rust provides the symbol directly.
bool ins_need_undo_get(void);
bool prompt_curpos_editable(void);
void truncate_spaces(char *line, size_t len);
void cursor_up_inner(win_T *wp, linenr_T n, bool skip_conceal);
void cursor_down_inner(win_T *wp, int n, bool skip_conceal);
void replace_push(char *str, size_t len);
bool get_can_cindent(void);

// Phase 3: functions now implemented in Rust (src/nvim-rs/edit/src/redraw.rs).
bool edit(int cmdchar, bool startln, int count);
void ins_redraw(bool ready);

// Phase 2: public thin-wrapper eliminations (Rust exports canonical names).
void undisplay_dollar(void);
void backspace_until_column(int col);
int get_literal(bool no_simplify);
void start_arrow(pos_T *end_insert_pos);
int stop_arrow(void);
void set_last_insert(int c);
void free_last_insert(void);
void beginline(int flags);
int oneright(void);
int oneleft(void);
int cursor_up(linenr_T n, bool upd_topline);
int cursor_down(int n, bool upd_topline);
int stuff_inserted(int c, int count, int no_esc);
char *get_last_insert_save(void);
void replace_push_nul(void);
int ins_copychar(linenr_T lnum);
colnr_T get_nolist_virtcol(void);
char *buf_prompt_text(const buf_T *buf);
char *prompt_text(void);
