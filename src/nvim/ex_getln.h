#pragma once

#include <stddef.h>  // IWYU pragma: keep

#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/ex_getln_defs.h"  // IWYU pragma: keep
#include "nvim/option_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

/// flags used by vim_strsave_fnameescape()
enum {
  VSE_NONE   = 0,
  VSE_SHELL  = 1,  ///< escape for a shell command
  VSE_BUFFER = 2,  ///< escape for a ":buffer" command
};

#include "ex_getln.h.generated.h"

// Functions implemented in Rust (src/nvim-rs/) that replace C implementations:
#ifdef __cplusplus
extern "C" {
#endif

bool text_locked(void);
const char *get_text_locked_msg(void);
bool cmdline_overstrike(void);
bool cmdline_at_end(void);
bool is_in_cmdwin(void);
int cmdpreview_get_ns(void);
int get_cmdline_firstc(void);
bool cmdline_is_empty(void);
bool cmdline_is_search(void);
bool cmdline_is_ex_cmd(void);
int cmdline_level(void);
bool cmdline_at_max_level(void);
int cmdline_get_pos(void);
int cmdline_get_len(void);
bool cmdline_is_password(void);
int cmdline_parse_search_delim(const char *pattern, size_t len);
bool cmdline_is_literal_pattern(const char *pattern, size_t len);
bool cmdline_has_word_boundary(const char *pattern, size_t len);
int cmdline_check_bracket_balance(const char *expr, size_t len);
bool cmdline_is_expr_complete(const char *expr, size_t len);
int cmdline_find_last_token(const char *expr, size_t len);
bool cmdline_fname_needs_escape(const char *fname, size_t len);
bool cmdline_starts_with_tilde(const char *path, size_t len);
bool cmdline_expand_fuzzy_supported(const void *xp);
bool cmdline_expand_is_file_context(const void *xp);
bool cmdline_expand_uses_internal(const void *xp);

// Phase 2: Pure logic functions implemented in Rust
char *vim_strsave_fnameescape(const char *fname, int what);
void escape_fname(char **pp);
void tilde_replace(char *orig_pat, int num_files, char **files);

// Phase 3: Rendering/UI functions implemented in Rust
void compute_cmdrow(void);
void cursorcmd(void);
void gotocmdline(bool clr);
void rs_cmdline_ui_flush(void);

// Phase 4+5+6: Functions implemented in Rust
int get_list_range(char **str, int *num1, int *num2);
int cmd_screencol(int bytepos);
void f_getcmdline(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_getcmdpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_getcmdprompt(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_getcmdscreenpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_getcmdtype(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Phase 67 (this migration): Functions implemented in Rust
int check_opt_wim(void);
void redrawcmd(void);
void redrawcmdline(void);
void draw_cmdline(int start, int len);
void put_on_cmdline(const char *str, int len, bool redraw);
void cmdline_paste_str(const char *s, bool literally);

// Phase 67.5: CommandLineState helper functions implemented in Rust
int rs_command_line_handle_ctrl_bsl(int *c, bool *gotesc);
int rs_command_line_insert_reg(int *c, bool *gotesc);
int rs_command_line_erase_chars(int c, int indent, void *is_state);

// Phase 2 (ex_getln migration): cmdline_screen_cleared implemented in Rust
void cmdline_screen_cleared(void);

// Phase 4 (ex_getln migration): buffer management implemented in Rust
void alloc_cmdbuff(int len);
void dealloc_cmdbuff(void);
void realloc_cmdbuff(int len);

// Phase 1 (command_line_enter migration): implemented in Rust (entry_impl.rs)
uint8_t *rs_command_line_enter(int firstc, int count, int indent, int clear_ccline);

// Phase 2 (getcmdline_prompt migration): implemented in Rust (entry_impl.rs)
char *rs_getcmdline_prompt(int firstc, const char *prompt, int hl_id, int xp_context,
                           const char *xp_arg, int one_key, bool *mouse_used);

// Phase 2 helper: apply pending highlight callback from C static to ccline.
void nvim_apply_pending_hl_callback(void);

// Phase 4: Functions implemented in Rust (ex_getln migration)
void rs_do_autocmd_cmdlinechanged(int firstc);
const char *rs_did_set_cedit(optset_T *args);
char *getexline(int c, void *cookie, int indent, bool do_concat);

#ifdef __cplusplus
}
#endif
