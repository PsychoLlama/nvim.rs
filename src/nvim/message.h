#pragma once

#include <errno.h>
#include <stdbool.h>
#include <stddef.h>  // IWYU pragma: keep

#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/grid_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/message_defs.h"  // IWYU pragma: keep

/// Types of dialogs passed to do_dialog().
enum {
  VIM_GENERIC   = 0,
  VIM_ERROR     = 1,
  VIM_WARNING   = 2,
  VIM_INFO      = 3,
  VIM_QUESTION  = 4,
  VIM_LAST_TYPE = 4,  ///< sentinel value
};

/// Return values for functions like vim_dialogyesno()
enum {
  VIM_YES        = 2,
  VIM_NO         = 3,
  VIM_CANCEL     = 4,
  VIM_ALL        = 5,
  VIM_DISCARDALL = 6,
};

extern MessageHistoryEntry *msg_hist_last;

EXTERN bool msg_ext_need_clear INIT( = false);
/// Set to true to force grouping a set of message chunks into a single `cmdline_show` event.
EXTERN bool msg_ext_skip_flush INIT( = false);
/// Set to true when message should be appended to previous message line.
EXTERN bool msg_ext_append INIT( = false);
/// Set to true when previous message should be overwritten.
EXTERN bool msg_ext_overwrite INIT( = false);

/// allocated grid for messages. Used unless ext_messages is active.
/// See also the description at msg_scroll_flush()
EXTERN ScreenGrid msg_grid INIT( = SCREEN_GRID_INIT);
EXTERN int msg_grid_pos INIT( = 0);

/// "adjusted" message grid. This grid accepts positions relative to
/// default_grid. Internally it will be translated to a position on msg_grid
/// relative to the start of the message area, or directly mapped to default_grid
/// for legacy (display-=msgsep) message scroll behavior.
/// TODO(bfredl): refactor "internal" message logic, msg_row etc
/// to use the correct positions already.
EXTERN GridView msg_grid_adj INIT( = { 0 });

/// value of msg_scrolled at latest msg_scroll_flush.
EXTERN int msg_scrolled_at_flush INIT( = 0);

EXTERN int msg_grid_scroll_discount INIT( = 0);

EXTERN int msg_listdo_overwrite INIT( = 0);

// Rust-implemented functions exported under C names via #[export_name]
bool msg_use_grid(void);
int msg_scrollsize(void);
bool msg_do_throttle(void);
int msg_use_printf(void);
int redirecting(void);

// Phase 1 (message migration): Rust-implemented replacements
void reset_last_sourcing(void);
bool message_filtered(const char *msg);
void msg_ui_refresh(void);
void msg_ui_flush(void);

// Phase 2 (message migration): Rust-implemented replacements
void msg_start(void);

// Phase 3 (message migration): Rust-implemented replacements
bool msg_keep(const char *s, int hl_id, bool keep, bool multiline);

// Phase 4 (message migration): Rust-implemented replacements
void clear_sb_text(bool all);
void msg_scroll_up(bool may_throttle, bool zerocmd);
void msg_clr_eos_force(void);
void msg_reset_scroll(void);
void msg_scroll_flush(void);
bool emsg_multiline(const char *s, const char *kind, int hl_id, bool multiline);
void msg_line_flush(void);
void msg_source(int hl_id);

// Phase 1: Simple leaf functions migrated to Rust
bool msg(const char *s, int hl_id);
int verb_msg(const char *s);
void msg_puts(const char *s);
void msg_puts_title(const char *s);
void msg_putchar(int c);
void msg_outnum(int n);
void msg_starthere(void);
void msg_clr_eos(void);
void msg_end_prompt(void);
bool emsg(const char *s);
void iemsg(const char *s);

// Phase 2: Truncation functions migrated to Rust
char *msg_strtrunc(const char *s, int force);
void trunc_string(const char *s, char *buf, int room_in, int buflen);

// Phase 76: Filename truncation functions migrated to Rust
char *msg_may_trunc(bool force, char *s);
char *msg_trunc(char *s, bool force, int hl_id);

// Phase 77: Home replace display functions migrated to Rust
void msg_home_replace(const char *fname);

// Phase 78: msg_outtrans migrated to Rust
int msg_outtrans(const char *str, int hl_id, bool hist);

// Phase 79: msg_outtrans_one migrated to Rust
const char *msg_outtrans_one(const char *p, int hl_id, bool hist);

// Phase 80: msg_outtrans_long migrated to Rust
void msg_outtrans_long(const char *longstr, int hl_id);

// Phase 81: msg_puts_hl migrated to Rust
void msg_puts_hl(const char *s, int hl_id, bool hist);

// Phase 5 (output): msg_puts_len migrated to Rust
void msg_puts_len(const char *str, ptrdiff_t len, int hl_id, bool hist);

// Phase 2 (additional): Formatting functions migrated to Rust
void msg_advance(int col);

// Phase 5: Scrollback management functions migrated to Rust
void may_clear_sb_text(void);
void sb_text_start_cmdline(void);
void sb_text_restart_cmdline(void);
void sb_text_end_cmdline(void);
void msg_sb_eol(void);

// Phase 68: keep_msg and delay functions migrated to Rust
void set_keep_msg(const char *s, int hl_id);
void msg_check_for_delay(bool check_msg_scroll);
void msg_clr_cmdline(void);
void msg_check(void);
bool messaging(void);
bool msg_end(void);
void give_warning(const char *message, bool hl);

// Phase 73: verbose functions migrated to Rust
void verbose_enter(void);
void verbose_leave(void);
void verbose_enter_scroll(void);
void verbose_leave_scroll(void);
void verbose_stop(void);
int verbose_open(void);
void msg_make(const char *arg);
void msg_putchar_hl(int c, int hl_id);

// Phase 82: msg_hist_clear and msg_hist_clear_temp migrated to Rust
void msg_hist_clear(int keep);
void msg_hist_clear_temp(void);

// Phase 83: msg_outtrans_len migrated to Rust
int msg_outtrans_len(const char *msgstr, int len, int hl_id, bool hist);

// Phase 84: msg_multiline migrated to Rust
void msg_multiline(String str, int hl_id, bool check_int, bool hist, bool *need_clear);

// Phase 87: msg_ext_set_kind migrated to Rust
void msg_ext_set_kind(const char *msg_kind);

// Phase 86: messagesopt_changed migrated to Rust
int messagesopt_changed(void);

// Phase 85: str2special family and msg_outtrans_special migrated to Rust
int msg_outtrans_special(const char *strstart, bool from, int maxlen);
const char *str2special(const char **sp, bool replace_spaces, bool replace_lt);
void str2specialbuf(const char *sp, char *buf, size_t len);
char *str2special_save(const char *str, bool replace_spaces, bool replace_lt);

// Phase 6: str2special_arena migrated to Rust
char *str2special_arena(const char *str, bool replace_spaces, bool replace_lt, Arena *arena);

// Phase 6: msgmore migrated to Rust
void msgmore(int n);

#include "message.h.generated.h"

// Prefer using semsg(), because perror() may send the output to the wrong
// destination and mess up the screen.
#define PERROR(msg) (void)semsg("%s: %s", (msg), strerror(errno))
