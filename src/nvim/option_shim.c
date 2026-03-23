// User-settable options. Checklist for adding a new option:
// - Put it in options.lua
// - For a global option: Add a variable for it in option_vars.h.
// - For a buffer or window local option:
//   - Add a variable to the window or buffer struct in buffer_defs.h.
//   - For a window option, add some code to copy_winopt().
//   - For a window string option, add code to check_winopt()
//     and clear_winopt(). If setting the option needs parsing,
//     add some code to didset_window_options().
//   - For a buffer option, add some code to buf_copy_options().
//   - For a buffer string option, add code to check_buf_options().
// - If it's a numeric option, add any necessary bounds checks to check_num_option_bounds().
// - If it's a list of flags, add some code in do_set(), search for WW_ALL.
// - Add documentation! "desc" in options.lua, and any other related places.
// - Add an entry in runtime/scripts/optwin.lua.

#define IN_OPTION_C
#include <assert.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>

#include "auto/config.h"
#include "klib/kvec.h"
#include "nvim/api/extmark.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/private/validate.h"
#include "nvim/ascii_defs.h"
#include "nvim/assert_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/cursor_shape.h"
#include "nvim/decoration_defs.h"
#include "nvim/decoration_provider.h"
#include "nvim/diff.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/eval/window.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_getln.h"
#include "nvim/ex_session.h"
#include "nvim/fold.h"
#include "nvim/fuzzy.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid_defs.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/indent.h"
#include "nvim/indent_c.h"
#include "nvim/insexpand.h"
#include "nvim/keycodes.h"
#include "nvim/log.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/mapping.h"
#include "nvim/math.h"
#include "nvim/mbyte.h"
#include "nvim/memfile.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/os/input.h"
#include "nvim/os/lang.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/popupmenu.h"
#include "nvim/pos_defs.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/runtime.h"
#include "nvim/spell.h"
#include "nvim/spellfile.h"
#include "nvim/spellsuggest.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/tag.h"
#include "nvim/terminal.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/undo_defs.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"
#include "nvim/winfloat.h"

#ifdef BACKSLASH_IN_FILENAME
# include "nvim/arglist.h"
#endif

_Static_assert(sizeof(vimoption_T) == 160,
               "sizeof(vimoption_T) changed - update Rust VIMOPTION_SIZE in option/src/accessors.rs");

// Rust FFI declarations (used by internal code)
extern bool rs_callback_from_typval(Callback *callback, const typval_T *arg);

// Rust metadata query functions (option pass 8 phase 1)
// is_option_hidden, option_has_type, option_has_scope now exported directly from Rust via #[export_name]
extern int rs_option_is_global_local(int opt_idx);
// New functions added in Phase 8 index.rs
extern int rs_option_get_type(int opt_idx);

extern void rs_option_value2string(OptIndex opt_idx, int opt_flags);

// Static assertions for constants shared with Rust (see callbacks/mod.rs UpdateType)
_Static_assert(UPD_VALID == 10, "UPD_VALID mismatch with Rust UpdateType::Valid");
_Static_assert(UPD_INVERTED == 20, "UPD_INVERTED mismatch with Rust UpdateType::Inverted");
_Static_assert(UPD_INVERTED_ALL == 25, "UPD_INVERTED_ALL mismatch with Rust UpdateType::InvertedAll");
_Static_assert(UPD_REDRAW_TOP == 30, "UPD_REDRAW_TOP mismatch with Rust UpdateType::RedrawTop");
_Static_assert(UPD_SOME_VALID == 35, "UPD_SOME_VALID mismatch with Rust UpdateType::SomeValid");
_Static_assert(UPD_NOT_VALID == 40, "UPD_NOT_VALID mismatch with Rust UpdateType::NotValid");
_Static_assert(UPD_CLEAR == 50, "UPD_CLEAR mismatch with Rust UpdateType::Clear");
_Static_assert(NO_SCREEN == 2, "NO_SCREEN mismatch with Rust NO_SCREEN constant");
_Static_assert(Ctrl_C == 3, "Ctrl_C mismatch with Rust CTRL_C constant");
_Static_assert(K_KENTER == -16715, "K_KENTER mismatch with Rust K_KENTER constant");
// Constant accessors replaced by Rust constants + static_assert validation
_Static_assert(BCO_ENTER == 1, "BCO_ENTER mismatch with Rust BCO_ENTER constant");
_Static_assert(BCO_ALWAYS == 2, "BCO_ALWAYS mismatch with Rust BCO_ALWAYS constant");
_Static_assert(BCO_NOHELP == 4, "BCO_NOHELP mismatch with Rust BCO_NOHELP constant");
_Static_assert(CPO_BUFOPTGLOB == 'S', "CPO_BUFOPTGLOB mismatch with Rust CPO_BUFOPTGLOB constant");
_Static_assert(CPO_BUFOPT == 's', "CPO_BUFOPT mismatch with Rust CPO_BUFOPT constant");
_Static_assert(CMOD_NOSWAPFILE == 0x2000, "CMOD_NOSWAPFILE mismatch with Rust CMOD_NOSWAPFILE constant");
_Static_assert(SID_NONE == -6, "SID_NONE mismatch with Rust SID_NONE constant");
_Static_assert(kOptFlagUIOption == (1 << 5), "kOptFlagUIOption mismatch with Rust K_OPT_FLAG_UI_OPTION constant");
_Static_assert(kOptFlagRedrWin == (1 << 8), "kOptFlagRedrWin mismatch with Rust K_OPT_FLAG_REDR_WIN constant");
_Static_assert(kOptFlagRedrBuf == (1 << 9), "kOptFlagRedrBuf mismatch with Rust K_OPT_FLAG_REDR_BUF constant");
_Static_assert(kOptFlagSecure == (1 << 14), "kOptFlagSecure mismatch with Rust K_OPT_FLAG_SECURE constant");
_Static_assert(kOptFlagInsecure == (1 << 18), "kOptFlagInsecure mismatch with Rust K_OPT_FLAG_INSECURE constant");
_Static_assert(kOptFlagCurswant == (1 << 21), "kOptFlagCurswant mismatch with Rust K_OPT_FLAG_CURSWANT constant");
_Static_assert(kOptFlagHLOnly == (1 << 23), "kOptFlagHLOnly mismatch with Rust K_OPT_FLAG_HL_ONLY constant");
_Static_assert(OPT_MODELINE == 0x04, "OPT_MODELINE mismatch with Rust OPT_MODELINE constant");

// Rust FFI callback declarations (referenced by options.generated.h)
extern const char *rs_did_set_hlsearch(optset_T *args);
extern const char *rs_did_set_ignorecase(optset_T *args);
extern const char *rs_did_set_title_icon(optset_T *args);
extern const char *rs_did_set_titlelen(optset_T *args);
extern const char *rs_did_set_iminsert(optset_T *args);
extern const char *rs_did_set_langnoremap(optset_T *args);
extern const char *rs_did_set_langremap(optset_T *args);
extern const char *rs_did_set_foldlevel(optset_T *args);
extern const char *rs_did_set_textwidth(optset_T *args);
extern const char *rs_did_set_pumblend(optset_T *args);
extern const char *rs_did_set_winblend(optset_T *args);
extern const char *rs_did_set_smoothscroll_full(optset_T *args);
extern const char *rs_did_set_showtabline_full(optset_T *args);
extern const char *rs_did_set_numberwidth(optset_T *args);
extern const char *rs_did_set_number_relativenumber(optset_T *args);
extern const char *rs_did_set_diff(optset_T *args);
extern const char *rs_did_set_eof_eol_fixeol_bomb(optset_T *args);
extern const char *rs_did_set_equalalways(optset_T *args);
extern const char *rs_did_set_winminheight(optset_T *args);
extern const char *rs_did_set_winminwidth(optset_T *args);
extern const char *rs_did_set_foldminlines(optset_T *args);
extern const char *rs_did_set_foldnestmax(optset_T *args);
extern const char *rs_did_set_helpheight(optset_T *args);
extern const char *rs_did_set_swapfile(optset_T *args);
extern const char *rs_did_set_modifiable(optset_T *args);
extern const char *rs_did_set_updatecount(optset_T *args);
extern const char *rs_did_set_lisp(optset_T *args);
extern const char *rs_did_set_wildchar(optset_T *args);
extern const char *rs_did_set_window(optset_T *args);
extern const char *rs_did_set_scrollbind(optset_T *args);
extern const char *rs_did_set_autochdir(optset_T *args);
extern const char *rs_did_set_wrap(optset_T *args);
extern const char *rs_did_set_winheight(optset_T *args);
extern const char *rs_did_set_winwidth(optset_T *args);
extern const char *rs_did_set_binary_full(optset_T *args);
extern const char *rs_did_set_modified(optset_T *args);
extern const char *rs_did_set_readonly(optset_T *args);
extern const char *rs_did_set_scrollback(optset_T *args);
extern const char *rs_did_set_undolevels_full(optset_T *args);
extern const char *rs_did_set_arabic_cb(optset_T *args);
extern const char *rs_did_set_cmdheight_cb(optset_T *args);
extern const char *rs_did_set_laststatus_cb(optset_T *args);
extern const char *rs_did_set_undofile_cb(optset_T *args);
extern const char *rs_did_set_buflisted(optset_T *args);
extern const char *rs_did_set_previewwindow(optset_T *args);
extern const char *rs_did_set_spell_full(optset_T *args);
extern const char *rs_did_set_shiftwidth_tabstop(optset_T *args);
extern const char *rs_did_set_xhistory(optset_T *args);
extern const char *rs_did_set_lines_or_columns(optset_T *args);
extern const char *rs_did_set_paste_full(optset_T *args);
#ifdef BACKSLASH_IN_FILENAME
extern const char *rs_did_set_shellslash(optset_T *args);
#endif

// Phase 3: winhighlight callback (from Rust winhl.rs)
extern const char *rs_did_set_winhighlight(optset_T *args);

// Phase 2: Medium-complexity string callbacks (from Rust string_simple.rs)
extern const char *rs_did_set_backupcopy(optset_T *args);
extern const char *rs_did_set_commentstring(optset_T *args);
extern const char *rs_did_set_comments(optset_T *args);
extern const char *rs_did_set_matchpairs(optset_T *args);
extern const char *rs_did_set_sessionoptions(optset_T *args);
extern const char *rs_did_set_spelloptions(optset_T *args);
extern const char *rs_did_set_diffanchors(optset_T *args);
extern const char *rs_did_set_messagesopt(optset_T *args);
extern const char *rs_did_set_diffopt(optset_T *args);
extern const char *rs_did_set_langmap(optset_T *args);
extern const char *rs_did_set_runtimepackpath(optset_T *args);
extern const char *rs_did_set_foldignore(optset_T *args);
extern const char *rs_did_set_foldmarker(optset_T *args);
extern const char *rs_did_set_foldmethod(optset_T *args);
extern const char *rs_did_set_cinoptions(optset_T *args);
extern const char *rs_did_set_spellfile(optset_T *args);
extern const char *rs_did_set_spelllang(optset_T *args);
extern const char *rs_did_set_spellcapcheck(optset_T *args);
extern const char *rs_did_set_keymodel(optset_T *args);
extern const char *rs_did_set_eventignore(optset_T *args);
extern const char *rs_did_set_spellsuggest(optset_T *args);
extern const char *rs_did_set_mkspellmem(optset_T *args);
extern const char *rs_did_set_winborder(optset_T *args);
extern const char *rs_did_set_pumborder(optset_T *args);
extern const char *rs_did_set_filetype_or_syntax(optset_T *args);
extern const char *rs_did_set_verbosefile(optset_T *args);
extern const char *rs_did_set_helpfile(optset_T *args);
extern const char *rs_did_set_optexpr(optset_T *args);
extern const char *rs_did_set_foldexpr(optset_T *args);
extern const char *rs_did_set_rulerformat(optset_T *args);
extern const char *rs_did_set_statusline(optset_T *args);
extern const char *rs_did_set_statuscolumn(optset_T *args);
extern const char *rs_did_set_tabline(optset_T *args);
extern const char *rs_did_set_winbar(optset_T *args);
extern const char *rs_did_set_highlight(optset_T *args);
extern const char *rs_did_set_iconstring(optset_T *args);
extern const char *rs_did_set_titlestring(optset_T *args);
extern const char *rs_did_set_isopt(optset_T *args);
extern const char *rs_did_set_iskeyword(optset_T *args);
extern const char *rs_did_set_signcolumn(optset_T *args);
extern const char *rs_did_set_tagcase(optset_T *args);
extern const char *rs_did_set_virtualedit_full(optset_T *args);
extern const char *rs_did_set_guicursor(optset_T *args);
extern const char *rs_did_set_ambiwidth(optset_T *args);
extern const char *rs_did_set_emoji(optset_T *args);
extern const char *rs_did_set_showbreak(optset_T *args);
extern const char *rs_did_set_cursorlineopt(optset_T *args);
// Phase 1: moved to direct C calls (no more nvim_did_set_* wrappers needed)
extern const char *rs_did_set_background(optset_T *args);
extern const char *rs_did_set_buftype(optset_T *args);
extern const char *rs_did_set_chars_option(optset_T *args);
extern const char *rs_did_set_colorcolumn(optset_T *args);
extern const char *rs_did_set_complete(optset_T *args);
extern const char *rs_did_set_encoding(optset_T *args);
extern const char *rs_did_set_fileformat(optset_T *args);
extern const char *rs_did_set_keymap(optset_T *args);
extern const char *rs_did_set_shada(optset_T *args);
extern const char *rs_did_set_str_generic(optset_T *args);
extern const char *rs_did_set_completeopt(optset_T *args);
extern const char *rs_did_set_varsofttabstop(optset_T *args);
extern const char *rs_did_set_vartabstop(optset_T *args);
extern const char *rs_did_set_cedit(optset_T *args);
extern const char *rs_did_set_operatorfunc(optset_T *args);
extern const char *rs_did_set_findfunc(optset_T *args);
extern const char *rs_did_set_completeitemalign(optset_T *args);

// Phase 109: rs_* aliases for already-Rust callbacks (didset.rs, userfunc.rs, tag, quickfix)
extern const char *rs_did_set_helplang(optset_T *args);
extern const char *rs_did_set_breakat(optset_T *args);
extern const char *rs_did_set_backupext_or_patchmode(optset_T *args);
extern const char *rs_did_set_mousescroll(optset_T *args);
extern const char *rs_did_set_completefunc(optset_T *args);
extern const char *rs_did_set_omnifunc(optset_T *args);
extern const char *rs_did_set_thesaurusfunc(optset_T *args);
extern const char *rs_did_set_tagfunc(optset_T *args);
extern const char *rs_did_set_quickfixtextfunc(optset_T *args);

// Phase 1: Simple string validation callbacks (from Rust string_simple.rs and display.rs)
extern const char *rs_did_set_concealcursor(optset_T *args);
extern const char *rs_did_set_cpoptions(optset_T *args);
extern const char *rs_did_set_formatoptions(optset_T *args);
extern const char *rs_did_set_mouse(optset_T *args);
extern const char *rs_did_set_shortmess(optset_T *args);
extern const char *rs_did_set_whichwrap(optset_T *args);
extern const char *rs_did_set_backspace(optset_T *args);
extern const char *rs_did_set_bufhidden(optset_T *args);
extern const char *rs_did_set_inccommand(optset_T *args);
extern const char *rs_did_set_lispoptions(optset_T *args);
extern const char *rs_did_set_wildmode(optset_T *args);
extern const char *rs_did_set_breakindentopt(optset_T *args);
extern const char *rs_did_set_display(optset_T *args);
extern const char *rs_did_set_showcmdloc(optset_T *args);
extern const char *rs_did_set_selection(optset_T *args);

// OptVal helpers (from Rust value.rs)
extern OptVal rs_optval_from_varp(OptIndex opt_idx, void *varp);
extern void rs_set_option_varp(OptIndex opt_idx, void *varp, OptVal value, int free_oldval);
extern char *rs_optval_to_cstr(OptVal o);

// Rust FFI declarations (window/layout module)
extern tabpage_T *rs_win_find_tabpage(win_T *win);

// =============================================================================
// Accessor functions for Rust code
// =============================================================================

// optset_T field accessors for Rust callbacks
void *nvim_optset_get_win(const void *args) { return (void *)((const optset_T *)args)->os_win; }
void *nvim_optset_get_buf(const void *args) { return (void *)((const optset_T *)args)->os_buf; }
int nvim_optset_get_idx(const void *args) { return (int)((const optset_T *)args)->os_idx; }
int nvim_optset_get_oldval_boolean(const void *args) { return (int)((const optset_T *)args)->os_oldval.boolean; }
int64_t nvim_optset_get_oldval_number(const void *args) { return ((const optset_T *)args)->os_oldval.number; }
int64_t nvim_optset_get_newval_number(const void *args) { return ((const optset_T *)args)->os_newval.number; }
void *nvim_optset_get_varp(const void *args) { return ((const optset_T *)args)->os_varp; }
int nvim_optset_get_newval_boolean(const void *args) { return (int)((const optset_T *)args)->os_newval.boolean; }
int nvim_optset_get_flags(const void *args) { return ((const optset_T *)args)->os_flags; }
void nvim_optset_set_value_changed(void *args, int val) { ((optset_T *)args)->os_value_changed = val != 0; }
void nvim_optset_set_value_checked(void *args, int val) { ((optset_T *)args)->os_value_checked = val != 0; }
const char *nvim_optset_get_oldval_str(const void *args) { return ((const optset_T *)args)->os_oldval.string.data; }

// Phase 99: verbosefile / helpfile accessors
int nvim_verbose_check_and_open(void) {
  verbose_stop();
  if (*p_vfile != NUL && verbose_open() == FAIL) {
    return 0;  // FAIL
  }
  return 1;  // OK
}


const char *nvim_get_curbuf_sua(void) { return curbuf->b_p_sua; }

// =============================================================================
// Setter functions for Rust code
// =============================================================================

// =============================================================================
// Callback accessor functions for Rust callbacks module
// =============================================================================

// Window diff accessor
int nvim_win_get_diff(win_T *win) { return win ? win->w_p_diff : 0; }

// Window accessor for view height (option module specific)
int nvim_option_win_get_view_height(win_T *win) { return win ? win->w_view_height : 0; }

// Window variable pointer accessors
const char *nvim_win_get_p_wbr(win_T *win) { return win ? (const char *)win->w_p_wbr : NULL; }

// Window briopt accessors and setters
int nvim_win_get_briopt_list(win_T *win) { return win ? win->w_briopt_list : 0; }
const char *nvim_win_get_p_briopt(win_T *win) { return win ? win->w_p_briopt : NULL; }
void nvim_win_set_briopt_shift(win_T *win, int val) { if (win) { win->w_briopt_shift = val; } }
void nvim_win_set_briopt_min(win_T *win, int val) { if (win) { win->w_briopt_min = val; } }
void nvim_win_set_briopt_sbr(win_T *win, int val) { if (win) { win->w_briopt_sbr = (val != 0); } }
void nvim_win_set_briopt_list(win_T *win, int val) { if (win) { win->w_briopt_list = val; } }
void nvim_win_set_briopt_vcol(win_T *win, int val) { if (win) { win->w_briopt_vcol = val; } }

// Window accessors for display callbacks
const char *nvim_option_win_get_stc(win_T *win) { return win ? (const char *)win->w_p_stc : NULL; }
void nvim_option_win_set_nrwidth(win_T *win, int value) { if (win) win->w_nrwidth_line_count = value; }
int nvim_option_win_get_sms(win_T *win) { return win ? win->w_p_sms : 0; }
void nvim_option_win_set_skipcol(win_T *win, int value) { if (win) win->w_skipcol = value; }

// Buffer accessors for behavior callbacks
int nvim_buf_get_p_swf(buf_T *buf) { return buf ? buf->b_p_swf : 0; }
int nvim_buf_get_p_udf(buf_T *buf) { return buf ? buf->b_p_udf : 0; }
void nvim_option_buf_set_modified_was_set(buf_T *buf, int val) { if (buf) buf->b_modified_was_set = val; }
int nvim_option_buf_get_b_p_ro(buf_T *buf) { return buf ? buf->b_p_ro : 0; }
void nvim_option_buf_set_b_did_warn(buf_T *buf, int val) { if (buf) buf->b_did_warn = val != 0; }
void *nvim_option_buf_get_terminal_ptr(buf_T *buf) { return buf ? buf->terminal : NULL; }
int nvim_option_buf_get_b_p_bin(buf_T *buf) { return buf ? buf->b_p_bin : 0; }
// Phase 88: undolevels accessors
void nvim_buf_set_b_p_ul(buf_T *buf, OptInt val) { buf->b_p_ul = val; }

// Phase 96: spellcapcheck and keymodel accessors
const char *nvim_compile_cap_prog_win(win_T *win) { return compile_cap_prog(win->w_s); }

// Phase 104: guicursor / ambiwidth / emoji / showbreak accessors

// Phase 98: spell / border option accessors
bool parse_border_opt(char *border_opt);  // defined in optionstr.c

// Iterate callback for all tab windows
void nvim_callback_for_all_tab_windows(void (*callback)(win_T *)) {
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    callback(wp);
  }
}

// Iterate callback for all buffers (for undofile and paste callbacks)
void nvim_for_all_buffers(void (*callback)(buf_T *)) {
  FOR_ALL_BUFFERS(bp) {
    callback(bp);
  }
}

// Buffer field accessors for undofile callback
int nvim_buf_is_changed(buf_T *buf) { return buf ? bufIsChanged(buf) : 0; }
int nvim_buf_has_memfile(buf_T *buf) { return buf && buf->b_ml.ml_mfp != NULL; }

// Buflisted callback accessors
int nvim_buf_get_p_bl(buf_T *buf) { return buf ? buf->b_p_bl : 0; }
void nvim_apply_autocmds_buf_event(int event, buf_T *buf) {
  apply_autocmds((event_T)event, NULL, NULL, true, buf);
}

// Previewwindow callback accessors and helpers
int nvim_win_get_p_pvw(win_T *win) { return win ? win->w_p_pvw : 0; }
void nvim_win_set_p_pvw(win_T *win, int val) { if (win) win->w_p_pvw = val != 0; }
void nvim_for_all_windows_in_curtab(void (*callback)(win_T *, void *), void *ud) {
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    callback(wp, ud);
  }
}
// Spell callback accessor
int nvim_win_get_p_spell(win_T *win) { return win ? win->w_p_spell : 0; }

// Shiftwidth/tabstop callback accessors
void *nvim_buf_get_b_p_sw_addr(buf_T *buf) { return buf ? (void *)&buf->b_p_sw : NULL; }
OptInt nvim_buf_get_b_p_sw(buf_T *buf) { return buf ? buf->b_p_sw : 0; }

// Pumblend accessors
void nvim_callback_set_pum_grid_blending(int value) { pum_grid.blending = (value != 0); }

// Winblend accessors
void nvim_callback_win_clamp_winbl(win_T *win) {
  if (win) {
    if (win->w_p_winbl > 100) win->w_p_winbl = 100;
    if (win->w_p_winbl < 0) win->w_p_winbl = 0;
  }
}
void nvim_callback_win_set_hl_needs_update(win_T *win, int value) {
  if (win) win->w_hl_needs_update = (value != 0);
}

// Scrollbind accessor
void nvim_callback_win_set_scbind_pos(win_T *win, int value) {
  if (win) win->w_scbind_pos = value;
}

// =============================================================================
// Phase 1 accessors for string callback migration
// =============================================================================

// Dereference varp as char** to get the current string value
const char *nvim_optset_get_varp_str(const void *args)
{
  char **varp = (char **)((const optset_T *)args)->os_varp;
  return varp ? *varp : NULL;
}
// Return os_errbuf for listflag error formatting
char *nvim_optset_get_errbuf(const void *args) { return ((const optset_T *)args)->os_errbuf; }
// Return os_errbuflen for listflag error formatting
size_t nvim_optset_get_errbuflen(const void *args) { return ((const optset_T *)args)->os_errbuflen; }
// Wrappers for side-effect functions
int nvim_call_briopt_check_win(const char *val, win_T *win)
{
  return briopt_check(val, win) == OK ? 1 : 0;
}
// Return address of win->w_p_briopt for varp comparison
const void *nvim_win_get_p_briopt_addr(win_T *win) { return win ? (const void *)&win->w_p_briopt : NULL; }
// Return varp from optset_T (as void*)
const void *nvim_optset_get_varp_ptr(const void *args) { return ((const optset_T *)args)->os_varp; }

// =============================================================================
// Phase 2 accessors for medium-complexity string callback migration
// =============================================================================

// Return os_newval.string.data from optset_T
const char *nvim_optset_get_newval_str(const void *args)
{
  return ((const optset_T *)args)->os_newval.string.data;
}
// Buffer-local bkc accessors
unsigned nvim_buf_get_bkc_flags(buf_T *buf) { return buf->b_bkc_flags; }
void nvim_buf_set_bkc_flags(buf_T *buf, unsigned val) { buf->b_bkc_flags = val; }
const char *nvim_buf_get_p_bkc(buf_T *buf) { return buf->b_p_bkc; }
// Window-local spo_flags accessors
unsigned nvim_win_get_spo_flags(win_T *win) { return win->w_s->b_p_spo_flags; }
void nvim_win_set_spo_flags(win_T *win, unsigned val) { win->w_s->b_p_spo_flags = val; }
// Pointer accessors for option value arrays (return pointer to NULL-terminated array)

// =============================================================================
// Phase 3 accessors for parse_winhl_opt migration
// =============================================================================

// Window ns_hl_winhl accessors
int nvim_win_get_ns_hl_winhl(win_T *win) { return win->w_ns_hl_winhl; }
void nvim_win_set_ns_hl_winhl(win_T *win, int val) { win->w_ns_hl_winhl = val; }
void nvim_win_set_ns_hl(win_T *win, int val) { win->w_ns_hl = val; }
// Return win->w_p_winhl (current winhighlight string)
const char *nvim_win_get_p_winhl(win_T *win) { return win ? win->w_p_winhl : NULL; }
// Return address of win->w_p_winhl for varp comparison
const void *nvim_win_get_p_winhl_addr(win_T *win) { return win ? (const void *)&win->w_p_winhl : NULL; }
// Prepare namespace for winhighlight: create if absent, bump hl_valid if existing.
// Returns the namespace id.
int nvim_winhl_ns_prepare(win_T *wp)
{
  if (wp->w_ns_hl_winhl == 0) {
    wp->w_ns_hl_winhl = (int)nvim_create_namespace(NULL_STRING);
  } else {
    DecorProvider *dp = get_decor_provider(wp->w_ns_hl_winhl, true);
    dp->hl_valid++;
  }
  return wp->w_ns_hl_winhl;
}
// Apply winhighlight namespace highlight definition (HL_GLOBAL flag set).
void nvim_winhl_ns_hl_def(int ns_hl, int hl_id_link, int hl_id)
{
  HlAttrs attrs = HLATTRS_INIT;
  attrs.rgb_ae_attr |= HL_GLOBAL;
  ns_hl_def(ns_hl, hl_id_link, attrs, hl_id, NULL);
}

// =============================================================================
// Simple accessor functions for Rust (don't require options array)
// =============================================================================

// fill_culopt_flags accessors
const char *nvim_win_get_p_culopt(win_T *wp) { return wp ? wp->w_p_culopt : NULL; }
void nvim_win_set_p_culopt_flags(win_T *wp, uint8_t flags) { if (wp) wp->w_p_culopt_flags = flags; }


#define OPTION_COUNT ARRAY_SIZE(options)

#include "option_shim.c.generated.h"

// options[] is initialized in options.generated.h.
// The options with a NULL variable are 'hidden': a set command for them is
// ignored and they are not printed.

#include "options.generated.h"
#include "options_map.generated.h"

// Rust behavior.rs now uses #[link_name] to call qf_resize_stack/ll_resize_stack directly.

// lines_or_columns callback: restore option varp to its old number value
void nvim_optset_restore_oldval_number(const void *args)
{
  const optset_T *a = (const optset_T *)args;
  OptVal oldval = (OptVal){ .type = kOptValTypeNumber, .data = a->os_oldval };
  rs_set_option_varp(a->os_idx, a->os_varp, oldval, 0);
}

// =============================================================================
// Accessor functions for Rust setcmd module (require options array)
// =============================================================================

// Options array accessor
vimoption_T *nvim_get_options_array(void) { return options; }

// Option flags accessor
uint32_t nvim_get_option_flags(OptIndex opt_idx) {
  if (opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) {
    return 0;
  }
  return options[opt_idx].flags;
}

// Option variable pointer accessor
void *nvim_get_option_var(OptIndex opt_idx) {
  if (opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) {
    return NULL;
  }
  return options[opt_idx].var;
}

// =============================================================================
// Accessor functions for rs_get_varp_from / rs_get_varp_scope_from
// =============================================================================

// Get p->var from a vimoption_T pointer
void *nvim_vimoption_get_var(vimoption_T *p) { return p->var; }
// Get p->flags_var from a vimoption_T pointer (NULL if not set)
unsigned *nvim_vimoption_get_flags_var_ptr(vimoption_T *p) { return p->flags_var; }

// Get the OptIndex of a vimoption_T by pointer arithmetic against the options array
OptIndex nvim_get_opt_idx_from_ptr(vimoption_T *p) { return (OptIndex)(p - options); }

// Get sizeof(winopt_T) at runtime (for GLOBAL_WO replication in Rust)
int nvim_get_sizeof_winopt_T(void) { return (int)sizeof(winopt_T); }

// =============================================================================
// Phase 8 metadata query accessors (Phase 1)
// =============================================================================
// Returns options[opt_idx].type as c_int for Rust metadata.rs
int nvim_get_option_type(OptIndex opt_idx) { return (int)options[opt_idx].type; }
// Returns options[opt_idx].scope_flags as c_int for Rust metadata.rs
int nvim_get_option_scope_flags(OptIndex opt_idx) { return (int)options[opt_idx].scope_flags; }
// Returns options[opt_idx].scope_idx[scope] as c_int for Rust metadata.rs
int nvim_get_option_scope_idx(OptIndex opt_idx, int scope) { return (int)options[opt_idx].scope_idx[scope]; }
// Returns options[opt_idx].immutable as c_int for Rust metadata.rs
int nvim_get_option_immutable(OptIndex opt_idx) { return (int)options[opt_idx].immutable; }
// Returns &options[opt_idx].def_val.data as const void* for Rust metadata.rs
const void *nvim_get_option_def_val_data_ptr(OptIndex opt_idx) { return &options[opt_idx].def_val.data; }
// Returns &options[opt_idx].script_ctx as void* for Rust metadata.rs
void *nvim_get_option_script_ctx_ptr(OptIndex opt_idx) { return &options[opt_idx].script_ctx; }

// =============================================================================
// Phase 8 default value management accessors (Phase 2)
// =============================================================================
// Set options[opt_idx].def_val (write accessor for Rust)
void nvim_set_option_def_val(OptIndex opt_idx, OptVal val) { options[opt_idx].def_val = val; }
// Returns get_varp(&options[opt_idx]) for check_options loop in Rust
void *nvim_get_option_varp_for_check(OptIndex opt_idx) { return get_varp(&options[opt_idx]); }
// Get the cmdheight default value as a number for set_init_tablocal
int64_t nvim_get_cmdheight_def_number(void) { return options[kOptCmdheight].def_val.data.number; }

// =============================================================================
// Phase 8 validate_option_value cluster accessors (Phase 3)
// =============================================================================
// Returns 1 if the current user is root (getuid() == ROOT_UID), 0 otherwise.
int nvim_is_root_user(void)
{
#ifdef UNIX
  return getuid() == ROOT_UID ? 1 : 0;
#else
  return 0;
#endif
}

// nvim_buf_opt_field_offsets: moved to buffer_shim.c
// nvim_win_get_opt_field_addr: moved to window_shim.c


// Option script context accessor
sctx_T nvim_get_option_script_ctx(OptIndex opt_idx) {
  if (opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) {
    return (sctx_T){ 0 };
  }
  return options[opt_idx].script_ctx;
}

// Window script context accessor
sctx_T nvim_get_win_p_script_ctx(win_T *win, OptIndex opt_idx) {
  if (!win || opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) {
    return (sctx_T){ 0 };
  }
  return win->w_p_script_ctx[opt_idx];
}

// Buffer script context accessor
sctx_T nvim_get_buf_p_script_ctx(buf_T *buf, OptIndex opt_idx) {
  if (!buf || opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) {
    return (sctx_T){ 0 };
  }
  return buf->b_p_script_ctx[opt_idx];
}

// Wrapper functions to expose static functions to Rust
int nvim_validate_opt_idx(win_T *win, OptIndex opt_idx, int opt_flags, uint32_t flags,
                          int prefix, const char **errmsg);


// set_options_bin helpers: accessors for curbuf binary-save fields
int nvim_curbuf_get_b_p_tw_nobin(void) { return (int)curbuf->b_p_tw_nobin; }
void nvim_curbuf_set_b_p_tw_nobin(OptInt v) { curbuf->b_p_tw_nobin = v; }
int nvim_curbuf_get_b_p_wm_nobin(void) { return (int)curbuf->b_p_wm_nobin; }
void nvim_curbuf_set_b_p_wm_nobin(OptInt v) { curbuf->b_p_wm_nobin = v; }
void nvim_curbuf_set_b_p_ml_nobin(int v) { curbuf->b_p_ml_nobin = v != 0; }
void nvim_curbuf_set_b_p_et_nobin(int v) { curbuf->b_p_et_nobin = v != 0; }
void nvim_curbuf_set_b_p_tw(OptInt v) { curbuf->b_p_tw = v; }
void nvim_curbuf_set_b_p_wm(OptInt v) { curbuf->b_p_wm = v; }
int nvim_curbuf_get_b_p_ml(void) { return curbuf->b_p_ml; }
void nvim_curbuf_set_b_p_ml(int v) { curbuf->b_p_ml = v != 0; }
int nvim_curbuf_get_b_p_et(void) { return curbuf->b_p_et; }
void nvim_curbuf_set_b_p_et(int v) { curbuf->b_p_et = v != 0; }
// =============================================================================
// Phase 4 (option pass 4) accessors for query.rs migration
// =============================================================================

// b_p_ep / p_ep accessors (for get_equalprg)
const char *nvim_curbuf_get_b_p_ep(void) { return curbuf->b_p_ep; }
// b_p_ffu / p_ffu accessors (for get_findfunc)
const char *nvim_curbuf_get_b_p_ffu(void) { return curbuf->b_p_ffu; }
// p_flp / b_p_flp accessors (for get_flp_value)
const char *nvim_buf_get_p_flp(buf_T *buf) { return buf->b_p_flp; }
unsigned nvim_win_get_ve_flags(win_T *wp) { return wp->w_ve_flags; }
// buf b_p_iminsert/imsearch accessors (for set_iminsert/imsearch_global)
OptInt nvim_buf_get_b_p_iminsert(buf_T *buf) { return buf->b_p_iminsert; }
OptInt nvim_buf_get_b_p_imsearch(buf_T *buf) { return buf->b_p_imsearch; }
// p_ma / b_p_ma accessors (for reset_modifiable)
int nvim_curbuf_get_b_p_ma(void) { return curbuf->b_p_ma; }
void nvim_curbuf_set_b_p_ma(int v) { curbuf->b_p_ma = v != 0; }
// insecure_flag pointer accessors
uint32_t *nvim_win_get_p_wrap_flags_ptr(win_T *wp) { return &wp->w_p_wrap_flags; }
uint32_t *nvim_win_get_p_stl_flags_ptr(win_T *wp) { return &wp->w_p_stl_flags; }
uint32_t *nvim_win_get_p_wbr_flags_ptr(win_T *wp) { return &wp->w_p_wbr_flags; }
uint32_t *nvim_win_get_p_fde_flags_ptr(win_T *wp) { return &wp->w_p_fde_flags; }
uint32_t *nvim_win_get_p_fdt_flags_ptr(win_T *wp) { return &wp->w_p_fdt_flags; }
uint32_t *nvim_win_get_buf_p_inde_flags_ptr(win_T *wp) { return &wp->w_buffer->b_p_inde_flags; }
uint32_t *nvim_win_get_buf_p_fex_flags_ptr(win_T *wp) { return &wp->w_buffer->b_p_fex_flags; }
uint32_t *nvim_win_get_buf_p_inex_flags_ptr(win_T *wp) { return &wp->w_buffer->b_p_inex_flags; }
uint32_t *nvim_win_allbuf_p_wrap_flags_ptr(win_T *wp) { return &wp->w_allbuf_opt.wo_wrap_flags; }
uint32_t *nvim_win_allbuf_p_fde_flags_ptr(win_T *wp) { return &wp->w_allbuf_opt.wo_fde_flags; }
uint32_t *nvim_win_allbuf_p_fdt_flags_ptr(win_T *wp) { return &wp->w_allbuf_opt.wo_fdt_flags; }
uint32_t *nvim_option_get_flags_ptr(OptIndex opt_idx) { return &options[opt_idx].flags; }

void nvim_curbuf_set_p_script_ctx(int idx, sctx_T sctx) { curbuf->b_p_script_ctx[idx] = sctx; }
void nvim_curwin_set_p_script_ctx(int idx, sctx_T sctx) { curwin->w_p_script_ctx[idx] = sctx; }
void nvim_curwin_set_allbuf_opt_script_ctx(int idx, sctx_T sctx) { curwin->w_allbuf_opt.wo_script_ctx[idx] = sctx; }
void nvim_option_set_script_ctx(OptIndex opt_idx, sctx_T sctx) { options[opt_idx].script_ctx = sctx; }

// Phase 4 (session.rs) accessors
void *nvim_get_varp_scope_by_idx(OptIndex opt_idx, int opt_flags)
{
  return get_varp_scope(&options[opt_idx], opt_flags);
}
// nvim_get_namebuff is defined in buffer.c
void *nvim_option_get_var_ptr(OptIndex opt_idx) { return options[opt_idx].var; }
OptVal nvim_option_get_def_val(OptIndex opt_idx) { return options[opt_idx].def_val; }

// Accessors for rs_set_init_2 and rs_set_init_3 (option pass 7 phase 2)
void nvim_option_ilog_rtp(void) { ILOG("startup runtimepath/packpath value: %s", p_rtp); }
// Accessors for rs_ex_set and rs_validate_opt_idx (option pass 7 phase 3)
int nvim_get_cmd_idx_setlocal(void) { return (int)CMD_setlocal; }
int nvim_get_cmd_idx_setglobal(void) { return (int)CMD_setglobal; }

// Get the option fullname for writing to session files
const char *nvim_option_get_fullname(OptIndex opt_idx) { return options[opt_idx].fullname; }
// Get kOptSyntax and kOptFiletype indices (already exist as enum values in opt_index.rs)

// Phase 3 winopt accessors
// Return pointer to the i-th string field of a winopt_T (for clear/check loops in Rust).
// String fields in the order used by check_winopt / clear_winopt.
char **nvim_winopt_string_field_ptr(winopt_T *wop, int idx)
{
  switch (idx) {
  case 0:  return &wop->wo_fdc;
  case 1:  return &wop->wo_fdc_save;
  case 2:  return &wop->wo_fdi;
  case 3:  return &wop->wo_fdm;
  case 4:  return &wop->wo_fdm_save;
  case 5:  return &wop->wo_fde;
  case 6:  return &wop->wo_fdt;
  case 7:  return &wop->wo_fmr;
  case 8:  return &wop->wo_eiw;
  case 9:  return &wop->wo_scl;
  case 10: return &wop->wo_rlc;
  case 11: return &wop->wo_sbr;
  case 12: return &wop->wo_stl;
  case 13: return &wop->wo_culopt;
  case 14: return &wop->wo_cc;
  case 15: return &wop->wo_cocu;
  case 16: return &wop->wo_briopt;
  case 17: return &wop->wo_winhl;
  case 18: return &wop->wo_lcs;
  case 19: return &wop->wo_fcs;
  case 20: return &wop->wo_ve;
  case 21: return &wop->wo_wbr;
  case 22: return &wop->wo_stc;
  default: return NULL;
  }
}

// Copy all scalar (bool/int/flags) fields from one winopt_T to another.
// Does NOT copy string fields or wo_script_ctx.
void nvim_copy_winopt_scalars(winopt_T *from, winopt_T *to)
{
  to->wo_arab = from->wo_arab;
  to->wo_list = from->wo_list;
  to->wo_nu = from->wo_nu;
  to->wo_rnu = from->wo_rnu;
  to->wo_ve_flags = from->wo_ve_flags;
  to->wo_nuw = from->wo_nuw;
  to->wo_rl = from->wo_rl;
  to->wo_wrap = from->wo_wrap;
  to->wo_wrap_save = from->wo_wrap_save;
  to->wo_lbr = from->wo_lbr;
  to->wo_bri = from->wo_bri;
  to->wo_scb = from->wo_scb;
  to->wo_scb_save = from->wo_scb_save;
  to->wo_sms = from->wo_sms;
  to->wo_crb = from->wo_crb;
  to->wo_crb_save = from->wo_crb_save;
  to->wo_siso = from->wo_siso;
  to->wo_so = from->wo_so;
  to->wo_spell = from->wo_spell;
  to->wo_cuc = from->wo_cuc;
  to->wo_cul = from->wo_cul;
  to->wo_diff = from->wo_diff;
  to->wo_diff_saved = from->wo_diff_saved;
  to->wo_cole = from->wo_cole;
  to->wo_fen = from->wo_fen;
  to->wo_fen_save = from->wo_fen_save;
  to->wo_fml = from->wo_fml;
  to->wo_fdl = from->wo_fdl;
  to->wo_fdl_save = from->wo_fdl_save;
  to->wo_fdn = from->wo_fdn;
  to->wo_lhi = from->wo_lhi;
  to->wo_winbl = from->wo_winbl;
  to->wo_wrap_flags = from->wo_wrap_flags;
  to->wo_stl_flags = from->wo_stl_flags;
  to->wo_wbr_flags = from->wo_wbr_flags;
  to->wo_fde_flags = from->wo_fde_flags;
  to->wo_fdt_flags = from->wo_fdt_flags;
}

// Copy the diff-mode save string fields with conditional xstrdup.
void nvim_copy_winopt_save_strs(winopt_T *from, winopt_T *to)
{
  to->wo_fdc_save = from->wo_diff_saved ? xstrdup(from->wo_fdc_save) : empty_string_option;
  to->wo_fdm_save = from->wo_diff_saved ? xstrdup(from->wo_fdm_save) : empty_string_option;
}

// Copy the wo_script_ctx array via memmove.
void nvim_copy_winopt_script_ctx(winopt_T *from, winopt_T *to)
{
  memmove(to->wo_script_ctx, from->wo_script_ctx, sizeof(to->wo_script_ctx));
}

// Update w_grid_alloc.blending based on current w_p_winbl value (different from window_shim's nvim_win_set_grid_blending which takes explicit bool).
void nvim_win_update_grid_blending(win_T *wp) { wp->w_grid_alloc.blending = wp->w_p_winbl > 0; }

/// 'title' and 'icon' only default to true if they have not been set or reset
/// in .vimrc and we can read the old value.
/// When 'title' and 'icon' have been reset in .vimrc, we won't even check if
/// they can be reset.  This reduces startup time when using X on a remote
/// machine.

// validate_opt_idx: broken circular wrapper deleted; setcmd.rs now calls rs_validate_opt_idx directly.
// find_option_end: broken circular wrapper deleted; setcmd.rs now calls rs_find_option_end directly.

/// Parse 'arg' for option settings.
///
/// 'arg' may be IObuff, but only when no errors can be present and option
/// does not need to be expanded with option_expand().
/// "opt_flags":
/// 0 for ":set"
/// OPT_GLOBAL   for ":setglobal"
/// OPT_LOCAL    for ":setlocal" and a modeline
/// OPT_MODELINE for a modeline
/// OPT_WINONLY  to only set window-local options
/// OPT_NOWIN    to skip setting window-local options
///
/// @param arg  option string (may be written to!)
///
/// @return  FAIL if an error is detected, OK otherwise

// When changing 'title', 'titlestring', 'icon' or 'iconstring', call
// maketitle() to create and display it.
/// set_options_bin -  called when 'bin' changes value.
///
/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).

/// Get the address of p_kp (keywordprg global), as void*.
/// Used by Rust stropt_get_newval to detect keywordprg option.
void *nvim_option_get_p_kp_ptr(void)
{
  return (void *)&p_kp;
}

/// Handle setting `winhighlight' in window "wp"
///
/// @param winhl  when NULL: use "wp->w_p_winhl"
/// @param wp     when NULL: only parse "winhl"
///
/// @return  whether the option value is valid.

/// Process the new global 'undolevels' option value.

void check_redraw(uint32_t flags)
{
  check_redraw_for(curbuf, curwin, flags);
}

/// Find index for an option. Don't go beyond `len` length.
///
/// @param[in]  name  Option name.
/// @param      len   Option name length.
///
/// @return Option index or kOptInvalid if option was not found.

/// Direct hash-based option lookup for use by Rust (avoids circular delegation).
///
/// Called from rs_find_option_len / rs_find_option in index.rs.
OptIndex nvim_find_option_len_hash(const char *name, size_t len)
{
  int index = find_option_hash(name, len);
  return index >= 0 ? option_hash_elems[index].opt_idx : kOptInvalid;
}

/// Create OptVal from var pointer.
///
/// @param       opt_idx  Option index in options[] table.
/// @param[out]  varp     Pointer to option variable.
///
/// @return Option value stored in varp.
OptVal optval_from_varp(OptIndex opt_idx, void *varp)
  FUNC_ATTR_NONNULL_ARG(2)
{
  return rs_optval_from_varp(opt_idx, varp);
}

/// Set option var pointer value from OptVal.
///
/// @param       opt_idx      Option index in options[] table.
/// @param[out]  varp         Pointer to option variable.
/// @param[in]   value        New option value.
/// @param       free_oldval  Free old value.

/// Convert an OptVal to an API Object.
Object optval_as_object(OptVal o)
{
  switch (o.type) {
  case kOptValTypeNil:
    return NIL;
  case kOptValTypeBoolean:
    switch (o.data.boolean) {
    case kFalse:
    case kTrue:
      return BOOLEAN_OBJ(o.data.boolean);
    case kNone:
      return NIL;
    }
    UNREACHABLE;
  case kOptValTypeNumber:
    return INTEGER_OBJ(o.data.number);
  case kOptValTypeString:
    return STRING_OBJ(o.data.string);
  }
  UNREACHABLE;
}

/// Convert an API Object to an OptVal.
OptVal object_as_optval(Object o, bool *error)
{
  switch (o.type) {
  case kObjectTypeNil:
    return NIL_OPTVAL;
  case kObjectTypeBoolean:
    return BOOLEAN_OPTVAL(o.data.boolean);
  case kObjectTypeInteger:
    return NUMBER_OPTVAL((OptInt)o.data.integer);
  case kObjectTypeString:
    return STRING_OPTVAL(o.data.string);
  default:
    *error = true;
    return NIL_OPTVAL;
  }
  UNREACHABLE;
}

/// Check if option is global-local.
static inline bool option_is_global_local(OptIndex opt_idx)
{
  return rs_option_is_global_local(opt_idx);
}

// Rust callers now use #[link_name] to call the rs_ functions directly.

/// Set option value directly, without processing any side effects.
///
/// @param  opt_idx    Option index in options[] table.
/// @param  value      Option value.
/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
/// @param  set_sid    Script ID. Special values:
///                      0: Use current script ID.
///                      SID_NONE: Don't set script ID.

/// Switch current context to get/set option value for window/buffer.
///
/// @param[out]  ctx        Current context. switchwin_T for window and aco_save_T for buffer.
/// @param       scope      Option scope. See OptScope in option.h.
/// @param[in]   from       Target buffer/window.
/// @param[out]  err        Error message, if any.
///
/// @return  true if context was switched, false otherwise.
static bool switch_option_context(void *const ctx, OptScope scope, void *const from, Error *err)
{
  switch (scope) {
  case kOptScopeGlobal:
    return false;
  case kOptScopeWin: {
    win_T *const win = (win_T *)from;
    switchwin_T *const switchwin = (switchwin_T *)ctx;

    if (win == curwin) {
      return false;
    }

    if (switch_win_noblock(switchwin, win, rs_win_find_tabpage(win), true)
        == FAIL) {
      restore_win_noblock(switchwin, true);

      if (ERROR_SET(err)) {
        return false;
      }
      api_set_error(err, kErrorTypeException, "Problem while switching windows");
      return false;
    }
    return true;
  }
  case kOptScopeBuf: {
    buf_T *const buf = (buf_T *)from;
    aco_save_T *const aco = (aco_save_T *)ctx;

    if (buf == curbuf) {
      return false;
    }
    aucmd_prepbuf(aco, buf);
    return true;
  }
  }
  UNREACHABLE;
}

/// Restore context after getting/setting option for window/buffer. See switch_option_context() for
/// params.
static void restore_option_context(void *const ctx, OptScope scope)
{
  switch (scope) {
  case kOptScopeGlobal:
    break;
  case kOptScopeWin:
    restore_win_noblock((switchwin_T *)ctx, true);
    break;
  case kOptScopeBuf:
    aucmd_restbuf((aco_save_T *)ctx);
    break;
  }
}

/// Get option value for buffer / window.
///
/// @param       opt_idx    Option index in options[] table.
/// @param[out]  flagsp     Set to the option flags (see OptFlags) (if not NULL).
/// @param[in]   scope      Option scope (can be OPT_LOCAL, OPT_GLOBAL or a combination).
/// @param[out]  hidden     Whether option is hidden.
/// @param       scope      Option scope. See OptScope in option.h.
/// @param[in]   from       Target buffer/window.
/// @param[out]  err        Error message, if any.
///
/// @return  Option value. Must be freed by caller.
OptVal get_option_value_for(OptIndex opt_idx, int opt_flags, const OptScope scope, void *const from,
                            Error *err)
{
  switchwin_T switchwin;
  aco_save_T aco;
  void *ctx = scope == kOptScopeWin ? (void *)&switchwin
                                    : (scope == kOptScopeBuf ? (void *)&aco : NULL);

  bool switched = switch_option_context(ctx, scope, from, err);
  if (ERROR_SET(err)) {
    return NIL_OPTVAL;
  }

  OptVal retv = get_option_value(opt_idx, opt_flags);

  if (switched) {
    restore_option_context(ctx, scope);
  }

  return retv;
}

/// Set option value for buffer / window.
///
/// @param       name        Option name.
/// @param       opt_idx     Option index in options[] table.
/// @param[in]   value       Option value.
/// @param[in]   opt_flags   Flags: OPT_LOCAL, OPT_GLOBAL, or 0 (both).
/// @param       scope       Option scope. See OptScope in option.h.
/// @param[in]   from        Target buffer/window.
/// @param[out]  err         Error message, if any.
void set_option_value_for(const char *name, OptIndex opt_idx, OptVal value, const int opt_flags,
                          const OptScope scope, void *const from, Error *err)
  FUNC_ATTR_NONNULL_ARG(1)
{
  switchwin_T switchwin;
  aco_save_T aco;
  void *ctx = scope == kOptScopeWin ? (void *)&switchwin
                                    : (scope == kOptScopeBuf ? (void *)&aco : NULL);

  bool switched = switch_option_context(ctx, scope, from, err);
  if (ERROR_SET(err)) {
    return;
  }

  const char *const errmsg = set_option_value_handle_tty(name, opt_idx, value, opt_flags);
  if (errmsg) {
    api_set_error(err, kErrorTypeException, "%s", errmsg);
  }

  if (switched) {
    restore_option_context(ctx, scope);
  }
}

/// Send update to UIs with values of UI relevant options
void ui_refresh_options(void)
{
  for (OptIndex opt_idx = 0; opt_idx < kOptCount; opt_idx++) {
    uint32_t flags = options[opt_idx].flags;
    if (!(flags & kOptFlagUIOption)) {
      continue;
    }
    String name = cstr_as_string(options[opt_idx].fullname);
    Object value = optval_as_object(optval_from_varp(opt_idx, options[opt_idx].var));
    ui_call_option_set(name, value);
  }
  if (p_mouse != NULL) {
    setmouse();
  }
}

/// Write modified options as ":set" commands to a file.
///
/// There are three values for "opt_flags":
/// OPT_GLOBAL:         Write global option values and fresh values of
///             buffer-local options (used for start of a session
///             file).
/// OPT_GLOBAL + OPT_LOCAL: Idem, add fresh values of window-local options for
///             curwin (used for a vimrc file).
/// OPT_LOCAL:          Write buffer-local option values for curbuf, fresh
///             and local values for window-local options of
///             curwin.  Local values are also written when at the
///             default value, because a modeline or autocommand
///             may have set them when doing ":edit file" and the
///             user has set them back at the default or fresh
///             value.
///             When "local_only" is true, don't write fresh
///             values, only local values (for ":mkview").

/// Get pointer to option variable, depending on local or global scope.
///
/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
void *get_varp_scope(vimoption_T *p, int opt_flags)
{
  return get_varp_scope_from(p, opt_flags, curbuf, curwin);
}

/// Get pointer to option variable at 'opt_idx', depending on local or global
/// scope.
void *get_option_varp_scope_from(OptIndex opt_idx, int opt_flags, buf_T *buf, win_T *win)
{
  return get_varp_scope_from(&(options[opt_idx]), opt_flags, buf, win);
}

/// Get option index from option pointer
static inline OptIndex get_opt_idx(vimoption_T *opt)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_PURE
{
  return (OptIndex)(opt - options);
}

/// Get pointer to option variable.
static inline void *get_varp(vimoption_T *p)
{
  return get_varp_from(p, curbuf, curwin);
}

/// Copy options from one window to another.
/// Used when splitting a window.
void win_copy_options(win_T *wp_from, win_T *wp_to)
{
  copy_winopt(&wp_from->w_onebuf_opt, &wp_to->w_onebuf_opt);
  copy_winopt(&wp_from->w_allbuf_opt, &wp_to->w_allbuf_opt);
  didset_window_options(wp_to, true);
}

// expand_option_start_col and expand_option_append live in Rust (setcmd.rs).
extern int expand_option_start_col;
extern bool expand_option_append;

// =============================================================================
// Accessors for rs_set_context_in_set_cmd (Rust Phase 3)
// =============================================================================

// expand_T field accessors
int nvim_xp_get_context(expand_T *xp) { return xp->xp_context; }
void nvim_xp_set_context(expand_T *xp, int val) { xp->xp_context = val; }
char *nvim_xp_get_pattern(expand_T *xp) { return xp->xp_pattern; }
void nvim_xp_set_pattern(expand_T *xp, char *val) { xp->xp_pattern = val; }
void nvim_xp_set_prefix(expand_T *xp, int val) { xp->xp_prefix = (xp_prefix_T)val; }
char *nvim_xp_get_line(expand_T *xp) { return xp->xp_line; }
int nvim_xp_get_backslash(expand_T *xp) { return xp->xp_backslash; }
void nvim_xp_set_backslash(expand_T *xp, int val) { xp->xp_backslash = val; }
/// Return xp->xp_buf (fixed-size buffer, EXPAND_BUF_LEN bytes).
char *nvim_xp_get_buf(expand_T *xp) { return xp->xp_buf; }

// options[opt_idx] field accessors for set_context logic
int nvim_option_has_expand_cb(OptIndex opt_idx)
{
  if (opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) {
    return 0;
  }
  return options[opt_idx].opt_expand_cb != NULL ? 1 : 0;
}

// Pointer-comparison accessors: check if options[opt_idx].var == &p_xxx
// Returns 1 if equal, 0 otherwise.
int nvim_opt_var_is_p_syn(OptIndex opt_idx) {
  if (opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) return 0;
  return options[opt_idx].var == &p_syn ? 1 : 0;
}
int nvim_opt_var_is_p_ft(OptIndex opt_idx) {
  if (opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) return 0;
  return options[opt_idx].var == &p_ft ? 1 : 0;
}
int nvim_opt_var_is_p_keymap(OptIndex opt_idx) {
  if (opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) return 0;
  return options[opt_idx].var == &p_keymap ? 1 : 0;
}
int nvim_opt_var_is_p_sps(OptIndex opt_idx) {
  if (opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) return 0;
  return options[opt_idx].var == &p_sps ? 1 : 0;
}
// For the expand dir/file option var comparisons - returns an enum:
// 0 = not a special path option, 1 = directory (XP_BS_THREE), 2 = directory (XP_BS_ONE), 3 = files (XP_BS_THREE), 4 = files (XP_BS_ONE)
int nvim_opt_var_expand_type(OptIndex opt_idx) {
  if (opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) return 0;
  char *p = options[opt_idx].var;
  if (p == (char *)&p_bdir || p == (char *)&p_dir || p == (char *)&p_pp
      || p == (char *)&p_rtp || p == (char *)&p_vdir) {
    return 2;  // EXPAND_DIRECTORIES + XP_BS_ONE
  }
  if (p == (char *)&p_path || p == (char *)&p_cdpath) {
    return 1;  // EXPAND_DIRECTORIES + XP_BS_THREE
  }
  if (p == (char *)&p_tags) {
    return 3;  // EXPAND_FILES + XP_BS_THREE
  }
  return 4;  // EXPAND_FILES + XP_BS_ONE
}

/// Get the value for the numeric or string option///opp in a nice format into
/// NameBuff[].  Must not be called with a hidden option!
///
/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).

/// Set the callback function value for an option that accepts a function name,
/// lambda, et al. (e.g. 'operatorfunc', 'tagfunc', etc.)
/// @return  OK if the option is successfully set to a function, otherwise FAIL
int option_set_callback_func(char *optval, Callback *optcb)
{
  if (optval == NULL || *optval == NUL) {
    callback_free(optcb);
    return OK;
  }

  typval_T *tv;
  if (*optval == '{'
      || (strncmp(optval, "function(", 9) == 0)
      || (strncmp(optval, "funcref(", 8) == 0)) {
    // Lambda expression or a funcref
    tv = eval_expr(optval, NULL);
    if (tv == NULL) {
      return FAIL;
    }
  } else {
    // treat everything else as a function name string
    tv = xcalloc(1, sizeof(*tv));
    tv->v_type = VAR_STRING;
    tv->vval.v_string = xstrdup(optval);
  }

  Callback cb;
  if (!rs_callback_from_typval(&cb, tv) || cb.type == kCallbackNone) {
    tv_free(tv);
    return FAIL;
  }

  callback_free(optcb);
  *optcb = cb;
  tv_free(tv);
  return OK;
}

/// Get window or buffer local options
dict_T *get_winbuf_options(const int bufopt)
  FUNC_ATTR_WARN_UNUSED_RESULT
{
  dict_T *const d = tv_dict_alloc();

  for (OptIndex opt_idx = 0; opt_idx < kOptCount; opt_idx++) {
    vimoption_T *opt = &options[opt_idx];

    if ((bufopt && (option_has_scope(opt_idx, kOptScopeBuf)))
        || (!bufopt && (option_has_scope(opt_idx, kOptScopeWin)))) {
      void *varp = get_varp(opt);

      if (varp != NULL) {
        typval_T opt_tv = optval_as_tv(optval_from_varp(opt_idx, varp), true);
        tv_dict_add_tv(d, opt->fullname, strlen(opt->fullname), &opt_tv);
      }
    }
  }

  return d;
}

Dict get_vimoption(String name, int opt_flags, buf_T *buf, win_T *win, Arena *arena, Error *err)
{
  OptIndex opt_idx = find_option_len(name.data, name.size);
  VALIDATE_S(opt_idx != kOptInvalid, "option (not found)", name.data, {
    return (Dict)ARRAY_DICT_INIT;
  });

  return vimoption2dict(&options[opt_idx], opt_flags, buf, win, arena);
}

Dict get_all_vimoptions(Arena *arena)
{
  Dict retval = arena_dict(arena, kOptCount);
  for (OptIndex opt_idx = 0; opt_idx < kOptCount; opt_idx++) {
    Dict opt_dict = vimoption2dict(&options[opt_idx], OPT_GLOBAL, curbuf, curwin, arena);
    PUT_C(retval, options[opt_idx].fullname, DICT_OBJ(opt_dict));
  }
  return retval;
}

static Dict vimoption2dict(vimoption_T *opt, int opt_flags, buf_T *buf, win_T *win, Arena *arena)
{
  OptIndex opt_idx = get_opt_idx(opt);
  Dict dict = arena_dict(arena, 13);

  PUT_C(dict, "name", CSTR_AS_OBJ(opt->fullname));
  PUT_C(dict, "shortname", CSTR_AS_OBJ(opt->shortname));

  const char *scope;
  if (option_has_scope(opt_idx, kOptScopeBuf)) {
    scope = "buf";
  } else if (option_has_scope(opt_idx, kOptScopeWin)) {
    scope = "win";
  } else {
    scope = "global";
  }

  PUT_C(dict, "scope", CSTR_AS_OBJ(scope));

  // welcome to the jungle
  PUT_C(dict, "global_local", BOOLEAN_OBJ(option_is_global_local(opt_idx)));
  PUT_C(dict, "commalist", BOOLEAN_OBJ(opt->flags & kOptFlagComma));
  PUT_C(dict, "flaglist", BOOLEAN_OBJ(opt->flags & kOptFlagFlagList));

  PUT_C(dict, "was_set", BOOLEAN_OBJ(opt->flags & kOptFlagWasSet));

  sctx_T script_ctx = { .sc_sid = 0 };
  if (opt_flags == OPT_GLOBAL) {
    script_ctx = opt->script_ctx;
  } else {
    // Scope is either OPT_LOCAL or a fallback mode was requested.
    if (option_has_scope(opt_idx, kOptScopeBuf)) {
      script_ctx = buf->b_p_script_ctx[opt->scope_idx[kOptScopeBuf]];
    }
    if (option_has_scope(opt_idx, kOptScopeWin)) {
      script_ctx = win->w_p_script_ctx[opt->scope_idx[kOptScopeWin]];
    }
    if (opt_flags != OPT_LOCAL && script_ctx.sc_sid == 0) {
      script_ctx = opt->script_ctx;
    }
  }

  PUT_C(dict, "last_set_sid", INTEGER_OBJ(script_ctx.sc_sid));
  PUT_C(dict, "last_set_linenr", INTEGER_OBJ(script_ctx.sc_lnum));
  PUT_C(dict, "last_set_chan", INTEGER_OBJ((int64_t)script_ctx.sc_chan));

  PUT_C(dict, "type", CSTR_AS_OBJ(optval_type_get_name((OptValType)rs_option_get_type(get_opt_idx(opt)))));
  PUT_C(dict, "default", optval_as_object(opt->def_val));
  PUT_C(dict, "allows_duplicates", BOOLEAN_OBJ(!(opt->flags & kOptFlagNoDup)));

  return dict;
}

// nvim_validate_opt_idx: broken circular wrapper deleted; setcmd.rs now calls rs_validate_opt_idx directly.

// =============================================================================
// Phase 6 accessors: do_syntax_autocmd, do_spelllang_source, get_fileformat_force
// =============================================================================

/// Apply EVENT_SYNTAX autocmds for the given buffer.
/// @param force  whether to force the autocmd (value_changed || syn_recursive == 1)
void nvim_apply_syntax_autocmd(buf_T *buf, bool force)
{
  apply_autocmds(EVENT_SYNTAX, buf->b_p_syn, buf->b_fname, force, buf);
}

/// Get win->w_s->b_p_spl (spelllang option for this window's wordlist).
const char *nvim_win_get_b_p_spl(win_T *win)
{
  return (win && win->w_s) ? win->w_s->b_p_spl : NULL;
}

/// Get first character of buf->b_p_ff as an unsigned char.
int nvim_buf_get_b_p_ff_first(const buf_T *buf)
{
  return (buf && buf->b_p_ff) ? (unsigned char)(*buf->b_p_ff) : 0;
}

// =============================================================================
// Phase 6 accessors: showoptions / showoneopt
// =============================================================================

/// Get the varp for an option by index (using get_varp for the current/global value).
void *nvim_get_varp_by_idx(OptIndex opt_idx)
{
  return get_varp(&options[opt_idx]);
}

/// Check if varp points to curbuf->b_changed.
/// Used by showoneopt to detect the 'modified' pseudo-boolean option.
int nvim_varp_is_curbuf_b_changed(const void *varp)
{
  return (const int *)varp == &curbuf->b_changed ? 1 : 0;
}

// =============================================================================
// Phase 6 accessors: ExpandSettings / match_str / escape_option_str_cmdline
// =============================================================================

extern char *rs_escape_option_str_cmdline(const char *var);

/// Invoke the expand callback for an option (constructs optexpand_T in C).
int nvim_option_invoke_expand_cb(OptIndex opt_idx, int opt_flags,
                                 void *xp, void *regmatch,
                                 int *num_matches, char ***matches)
{
  if (opt_idx == kOptInvalid || options[opt_idx].opt_expand_cb == NULL) {
    return FAIL;
  }
  optexpand_T args = {
    .oe_varp = get_varp_scope(&options[opt_idx], opt_flags),
    .oe_idx = opt_idx,
    .oe_append = (bool)expand_option_append,
    .oe_regmatch = (regmatch_T *)regmatch,
    .oe_xp = (expand_T *)xp,
    .oe_set_arg = ((expand_T *)xp)->xp_line + expand_option_start_col,
  };
  args.oe_include_orig_val = !args.oe_append && (*args.oe_set_arg == NUL);

  rs_option_value2string(opt_idx, opt_flags);
  char *var = NameBuff;
  char *buf = rs_escape_option_str_cmdline(var);
  args.oe_opt_value = buf;

  int result = options[opt_idx].opt_expand_cb(&args, num_matches, matches);
  xfree(buf);
  return result;
}

/// Get the shortname of an option by index.
const char *nvim_option_get_shortname(OptIndex opt_idx)
{
  return options[opt_idx].shortname;
}

/// Get rm_ic field from a regmatch_T pointer (opaque).
int nvim_regmatch_get_rm_ic(const void *regmatch)
{
  return ((const regmatch_T *)regmatch)->rm_ic;
}

/// Set rm_ic field on a regmatch_T pointer (opaque).
void nvim_regmatch_set_rm_ic(void *regmatch, int val)
{
  ((regmatch_T *)regmatch)->rm_ic = val;
}

/// Get fuzmatch_str_T size.
size_t nvim_option_get_fuzmatch_size(void)
{
  return sizeof(fuzmatch_str_T);
}

/// Set a fuzmatch_str_T entry at index in an opaque array.
void nvim_option_fuzmatch_set(void *fuzmatch, int idx, const char *str, int score)
{
  fuzmatch_str_T *fm = (fuzmatch_str_T *)fuzmatch;
  fm[idx].idx = idx;
  fm[idx].str = xstrdup(str);
  fm[idx].score = score;
}

// =============================================================================
// Phase 9 (pass 9) accessors: did_set_option / set_option / apply_optionset_autocmd
// =============================================================================

/// Invoke the did_set_cb for an option. Constructs optset_T in C and calls the callback.
/// Returns NULL on success, error message on failure.
/// Output fields are written back to *value_changed_out, *value_checked_out, *restore_chartab_out.
const char *nvim_invoke_did_set_cb(OptIndex opt_idx, void *varp, OptVal old_value,
                                   OptVal new_value, int opt_flags,
                                   char *errbuf, size_t errbuflen,
                                   int *value_changed_out, int *value_checked_out,
                                   int *restore_chartab_out)
{
  vimoption_T *opt = &options[opt_idx];
  if (opt->opt_did_set_cb == NULL) {
    return NULL;
  }
  optset_T args = {
    .os_varp = varp,
    .os_idx = opt_idx,
    .os_flags = opt_flags,
    .os_oldval = old_value.data,
    .os_newval = new_value.data,
    .os_value_checked = false,
    .os_value_changed = false,
    .os_restore_chartab = false,
    .os_errbuf = errbuf,
    .os_errbuflen = errbuflen,
    .os_buf = curbuf,
    .os_win = curwin,
  };
  const char *errmsg = opt->opt_did_set_cb(&args);
  *value_changed_out = args.os_value_changed ? 1 : 0;
  *value_checked_out = args.os_value_checked ? 1 : 0;
  *restore_chartab_out = args.os_restore_chartab ? 1 : 0;
  return errmsg;
}

/// Set the script context for an option from a SID.
/// Uses C to construct sctx_T (avoids FFI layout issues with sc_chan field).
void nvim_set_option_sctx_from_sid(OptIndex opt_idx, int opt_flags, int set_sid)
{
  sctx_T script_ctx = set_sid == 0 ? current_sctx : (sctx_T){ .sc_sid = set_sid };
  set_option_sctx(opt_idx, opt_flags, script_ctx);
}

/// Returns 1 if opt->opt_did_set_cb is non-NULL for opt_idx.
int nvim_option_has_did_set_cb(OptIndex opt_idx) { return options[opt_idx].opt_did_set_cb != NULL ? 1 : 0; }

// nvim_curwin_get_w_curswant and nvim_curwin_set_w_set_curswant are defined in indent_ffi.c

/// Get `secure` global variable
// nvim_get_secure, nvim_set_secure are defined in ex_docmd.c
// nvim_get_sandbox is defined in undo.c



/// Error message strings for did_set_option

/// Call emsg(_(msg)) -- translates and shows error message

/// Call get_varp_scope(&options[opt_idx], opt_flags)
void *nvim_get_varp_scope_opt(OptIndex opt_idx, int opt_flags)
{
  return get_varp_scope(&options[opt_idx], opt_flags);
}

/// Call get_varp(&options[opt_idx])
void *nvim_get_varp_opt(OptIndex opt_idx)
{
  return get_varp(&options[opt_idx]);
}

/// Return pointer to options[opt_idx] (vimoption_T *).
vimoption_T *nvim_get_option_ptr_by_idx(OptIndex opt_idx)
{
  return &options[opt_idx];
}

// =============================================================================
// Phase 9 (pass 9) trampolines: set_option / apply_optionset_autocmd
// =============================================================================

/// Apply the OptionSet autocommand: called from rs_set_option_impl in Rust.
/// Keeps VimL type system interactions (optval_as_tv, set_vim_var_tv, etc.) in C.
void nvim_apply_optionset_autocmd(OptIndex opt_idx, int opt_flags, OptVal oldval,
                                  OptVal oldval_g, OptVal oldval_l, OptVal newval,
                                  const char *errmsg)
{
  // Don't do this while starting up, failure or recursively.
  if (starting || errmsg != NULL || *get_vim_var_str(VV_OPTION_TYPE) != NUL) {
    return;
  }

  char buf_type[7];
  typval_T oldval_tv = optval_as_tv(oldval, false);
  typval_T oldval_g_tv = optval_as_tv(oldval_g, false);
  typval_T oldval_l_tv = optval_as_tv(oldval_l, false);
  typval_T newval_tv = optval_as_tv(newval, false);

  vim_snprintf(buf_type, sizeof(buf_type), "%s", (opt_flags & OPT_LOCAL) ? "local" : "global");
  set_vim_var_tv(VV_OPTION_NEW, &newval_tv);
  set_vim_var_tv(VV_OPTION_OLD, &oldval_tv);
  set_vim_var_string(VV_OPTION_TYPE, buf_type, -1);
  if (opt_flags & OPT_LOCAL) {
    set_vim_var_string(VV_OPTION_COMMAND, "setlocal", -1);
    set_vim_var_tv(VV_OPTION_OLDLOCAL, &oldval_tv);
  }
  if (opt_flags & OPT_GLOBAL) {
    set_vim_var_string(VV_OPTION_COMMAND, "setglobal", -1);
    set_vim_var_tv(VV_OPTION_OLDGLOBAL, &oldval_tv);
  }
  if ((opt_flags & (OPT_LOCAL | OPT_GLOBAL)) == 0) {
    set_vim_var_string(VV_OPTION_COMMAND, "set", -1);
    set_vim_var_tv(VV_OPTION_OLDLOCAL, &oldval_l_tv);
    set_vim_var_tv(VV_OPTION_OLDGLOBAL, &oldval_g_tv);
  }
  if (opt_flags & OPT_MODELINE) {
    set_vim_var_string(VV_OPTION_COMMAND, "modeline", -1);
    set_vim_var_tv(VV_OPTION_OLDLOCAL, &oldval_tv);
  }
  apply_autocmds(EVENT_OPTIONSET, options[opt_idx].fullname, NULL, false, NULL);
  reset_v_option_vars();
}

/// Call ui_call_option_set for a specific option with a saved new value.
void nvim_ui_call_option_set(OptIndex opt_idx, OptVal saved_new_value)
{
  ui_call_option_set(cstr_as_string(options[opt_idx].fullname), optval_as_object(saved_new_value));
}

// rs_set_option_impl is declared at the top of the extern declarations section above.

// =============================================================================
// Phase 11 (pass 11) accessors: buf_copy_options
// =============================================================================

/// Returns buf->b_p_initialized.
int nvim_buf_get_b_p_initialized(buf_T *buf) { return buf->b_p_initialized ? 1 : 0; }
/// Sets buf->b_p_initialized.
void nvim_buf_set_b_p_initialized(buf_T *buf, int val) { buf->b_p_initialized = val != 0; }

/// Returns buf->b_help.
int nvim_buf_get_b_help(buf_T *buf) { return buf->b_help ? 1 : 0; }
/// Sets buf->b_help.
void nvim_buf_set_b_help(buf_T *buf, int val) { buf->b_help = val != 0; }

/// CLEAR_FIELD(buf->b_p_script_ctx) -- zeroes the script_ctx array for the buffer.
void nvim_buf_clear_b_p_script_ctx(buf_T *buf) { CLEAR_FIELD(buf->b_p_script_ctx); }

/// Returns 1 if buf->b_p_bt[0] == 'h' (help buftype), else 0.
int nvim_buf_get_b_p_bt_is_help(buf_T *buf)
{
  return (buf->b_p_bt && buf->b_p_bt[0] == 'h') ? 1 : 0;
}

/// Saves b_p_isk pointer and NULLs the field.
/// Returns the saved pointer.
char *nvim_buf_save_and_clear_b_p_isk(buf_T *buf)
{
  char *saved = buf->b_p_isk;
  buf->b_p_isk = NULL;
  return saved;
}

/// Restores b_p_isk from a previously saved pointer.
void nvim_buf_restore_b_p_isk(buf_T *buf, char *saved) { buf->b_p_isk = saved; }

/// buf->b_p_ro = false
void nvim_buf_clear_b_p_ro(buf_T *buf) { buf->b_p_ro = false; }

/// compile_cap_prog(&buf->b_s)
void nvim_call_compile_cap_prog_buf(buf_T *buf) { compile_cap_prog(&buf->b_s); }

/// tabstop_set(str, &buf->b_p_vsts_array)
void nvim_call_tabstop_set_vsts(buf_T *buf, const char *str) { tabstop_set(str, &buf->b_p_vsts_array); }
/// tabstop_set(str, &buf->b_p_vts_array)
void nvim_call_tabstop_set_vts(buf_T *buf, const char *str) { tabstop_set(str, &buf->b_p_vts_array); }

/// Returns buf->b_p_vts_array (for null check)
int nvim_buf_get_b_p_vts_array_is_null(buf_T *buf) { return buf->b_p_vts_array == NULL ? 1 : 0; }

/// buf->b_kmap_state |= KEYMAP_INIT
void nvim_buf_kmap_state_set_init(buf_T *buf) { buf->b_kmap_state |= KEYMAP_INIT; }


// Generic helpers for offset-based buf_T field writes (used by bufcopy.rs):

/// Set a buf_T char* field at the given byte offset to xstrdup(s).
void nvim_buf_set_string_field(buf_T *buf, ptrdiff_t offset, const char *s)
{
  char **field = (char **)(((char *)buf) + offset);
  *field = xstrdup(s);
}

/// Set a buf_T char* field at the given byte offset to empty_string_option.
void nvim_buf_empty_string_field(buf_T *buf, ptrdiff_t offset)
{
  char **field = (char **)(((char *)buf) + offset);
  *field = empty_string_option;
}

/// Generic buf_T bool field setter: writes 0 or 1 via byte offset.
void nvim_buf_set_bool_field(buf_T *buf, ptrdiff_t offset, int val)
{
  *(int *)(((char *)buf) + offset) = val != 0;
}

/// Generic buf_T OptInt field setter: writes OptInt value via byte offset.
void nvim_buf_set_optint_field(buf_T *buf, ptrdiff_t offset, OptInt val)
{
  *(OptInt *)(((char *)buf) + offset) = val;
}

/// Generic buf_T OptInt field getter: reads OptInt value via byte offset.
OptInt nvim_buf_get_optint_field(buf_T *buf, ptrdiff_t offset) { return *(OptInt *)(((char *)buf) + offset); }
/// Generic buf_T bool field getter: reads bool field via byte offset (returns 0 or 1).
int nvim_buf_get_bool_field(buf_T *buf, ptrdiff_t offset) { return (int)(*(bool *)(((char *)buf) + offset)); }
/// Sets buf->b_p_fenc = xstrdup(p_fenc).
void nvim_buf_set_b_p_fenc_dup(buf_T *buf) { buf->b_p_fenc = xstrdup(p_fenc); }

/// Sets buf->b_p_bh = empty_string_option.
void nvim_buf_set_b_p_bh_empty(buf_T *buf) { buf->b_p_bh = empty_string_option; }
/// Sets buf->b_p_bt = empty_string_option.
void nvim_buf_set_b_p_bt_empty(buf_T *buf) { buf->b_p_bt = empty_string_option; }

// Setters for global-local fields that default to "no local value":
// Simple empty-string ones now handled via nvim_buf_empty_string_field.
void nvim_buf_set_b_p_ac_minus1(buf_T *buf) { buf->b_p_ac = -1; }
void nvim_buf_set_b_p_ar_minus1(buf_T *buf) { buf->b_p_ar = -1; }
void nvim_buf_set_b_p_ul_no_local(buf_T *buf) { buf->b_p_ul = NO_LOCAL_UNDOLEVEL; }
// These also zero flag fields, so they cannot be replaced by the generic helper:
void nvim_buf_set_b_p_bkc_empty(buf_T *buf) { buf->b_p_bkc = empty_string_option; buf->b_bkc_flags = 0; }
void nvim_buf_set_b_p_tc_empty(buf_T *buf) { buf->b_p_tc = empty_string_option; buf->b_tc_flags = 0; }
void nvim_buf_set_b_p_cot_empty(buf_T *buf) { buf->b_p_cot = empty_string_option; buf->b_cot_flags = 0; }
/// Sets buf->b_s.b_syn_isk = empty_string_option (b_s substructure -- not in offset table).
void nvim_buf_set_b_s_syn_isk_empty(buf_T *buf) { buf->b_s.b_syn_isk = empty_string_option; }

// Scalar field setters kept for nopaste/nobin variants and special cases:
void nvim_buf_set_b_p_ai_nopaste(buf_T *buf, int v) { buf->b_p_ai_nopaste = v != 0; }
void nvim_buf_set_b_p_tw_nopaste(buf_T *buf, OptInt v) { buf->b_p_tw_nopaste = v; }
void nvim_buf_set_b_p_tw_nobin(buf_T *buf, OptInt v) { buf->b_p_tw_nobin = v; }
void nvim_buf_set_b_p_wm_nopaste(buf_T *buf, OptInt v) { buf->b_p_wm_nopaste = v; }
void nvim_buf_set_b_p_wm_nobin(buf_T *buf, OptInt v) { buf->b_p_wm_nobin = v; }
void nvim_buf_set_b_p_et_nobin(buf_T *buf, int v) { buf->b_p_et_nobin = v != 0; }
void nvim_buf_set_b_p_et_nopaste(buf_T *buf, int v) { buf->b_p_et_nopaste = v != 0; }
void nvim_buf_set_b_p_ml_nobin(buf_T *buf, int v) { buf->b_p_ml_nobin = v != 0; }
void nvim_buf_set_b_p_sts_nopaste(buf_T *buf, OptInt v) { buf->b_p_sts_nopaste = v; }
// Per-buffer nopaste/nobin field getters (for paste restore in Rust)
int nvim_buf_get_b_p_ai_nopaste(buf_T *buf) { return (int)buf->b_p_ai_nopaste; }
OptInt nvim_buf_get_b_p_tw_nopaste(buf_T *buf) { return buf->b_p_tw_nopaste; }
OptInt nvim_buf_get_b_p_wm_nopaste(buf_T *buf) { return buf->b_p_wm_nopaste; }
OptInt nvim_buf_get_b_p_sts_nopaste(buf_T *buf) { return buf->b_p_sts_nopaste; }
int nvim_buf_get_b_p_et_nopaste(buf_T *buf) { return (int)buf->b_p_et_nopaste; }
char *nvim_buf_get_b_p_vsts(buf_T *buf) { return buf->b_p_vsts; }
char *nvim_buf_get_b_p_vsts_nopaste(buf_T *buf) { return buf->b_p_vsts_nopaste; }
void nvim_buf_set_b_p_vsts_raw(buf_T *buf, char *val) { buf->b_p_vsts = val; }
int *volatile *nvim_buf_get_b_p_vsts_array_ptr(buf_T *buf) { return (int *volatile *)&buf->b_p_vsts_array; }
// nvim_buf_set_b_p_iminsert / nvim_buf_set_b_p_imsearch: already in buffer.c (int param)
void nvim_buf_set_b_p_ma(buf_T *buf, int v) { buf->b_p_ma = v != 0; }

// String field setters using xstrdup that cannot use the generic helper:
// (b_s substructure fields are not in the opt_field_offsets table)
void nvim_buf_set_b_p_vsts_nopaste_dup(buf_T *buf, const char *s) { buf->b_p_vsts_nopaste = s ? xstrdup(s) : NULL; }
void nvim_buf_set_b_s_spc_dup(buf_T *buf, const char *s) { buf->b_s.b_p_spc = xstrdup(s); }
void nvim_buf_set_b_s_spf_dup(buf_T *buf, const char *s) { buf->b_s.b_p_spf = xstrdup(s); }
void nvim_buf_set_b_s_spl_dup(buf_T *buf, const char *s) { buf->b_s.b_p_spl = xstrdup(s); }
void nvim_buf_set_b_s_spo_dup(buf_T *buf, const char *s) { buf->b_s.b_p_spo = xstrdup(s); }

/// Copy global script_ctx for a buf-opt index to the buffer's script_ctx array.
/// Implements COPY_OPT_SCTX(buf, bv).
void nvim_buf_copy_opt_sctx(buf_T *buf, int bv)
{
  if (buf && bv >= 0 && (size_t)bv < ARRAY_SIZE(buf->b_p_script_ctx)) {
    buf->b_p_script_ctx[bv] = options[buf_opt_idx[bv]].script_ctx;
  }
}

/// Set b_s.b_p_spo_flags from global spo_flags.
void nvim_buf_set_b_s_spo_flags_from_global(buf_T *buf) { buf->b_s.b_p_spo_flags = spo_flags; }

// =============================================================================
// Phase 11 (pass 11) accessors: set_init_1, set_init_expand_env
// =============================================================================


// Compile-time boundary validation for kBufOpt enum (Rust K_BUF_OPT_* constants).
// Checks first value, last value, and count; specific index alignment is validated at
// runtime by the offset table in varp.rs via buf_field_offsets().
_Static_assert((int)kBufOptAutocomplete == 0, "K_BUF_OPT_AUTOCOMPLETE mismatch");
_Static_assert((int)kBufOptWrapmargin == 90, "K_BUF_OPT_WRAPMARGIN mismatch");
_Static_assert(kBufOptCount == 91, "K_BUF_OPT_COUNT mismatch");

/// Calls bind_textdomain_codeset(PROJECT_NAME, p_enc) if HAVE_WORKING_LIBINTL.
/// No-op otherwise.
void nvim_call_bind_textdomain_codeset(void)
{
#ifdef HAVE_WORKING_LIBINTL
  (void)bind_textdomain_codeset(PROJECT_NAME, p_enc);
#endif
}



// =============================================================================
// Phase 12 Pass 2: didset_options / didset_options2 sub-function wrappers
// =============================================================================

// didset_string_options is now implemented in Rust; this call goes directly to Rust.
// (The Rust implementation is registered via #[export_name = "didset_string_options"])
extern void didset_string_options(void);  // defined in Rust optionstr crate
/// xfree(curbuf->b_p_vsts_array) + tabstop_set(curbuf->b_p_vsts, &curbuf->b_p_vsts_array).
void nvim_call_curbuf_tabstop_set_vsts(void)
{
  xfree(curbuf->b_p_vsts_array);
  tabstop_set(curbuf->b_p_vsts, &curbuf->b_p_vsts_array);
}
/// xfree(curbuf->b_p_vts_array) + tabstop_set(curbuf->b_p_vts, &curbuf->b_p_vts_array).
void nvim_call_curbuf_tabstop_set_vts(void)
{
  xfree(curbuf->b_p_vts_array);
  tabstop_set(curbuf->b_p_vts, &curbuf->b_p_vts_array);
}

// =============================================================================
// Phase 12 lifecycle accessors (set_option_default, set_options_default,
// option_expand, free_all_options)
// =============================================================================


/// free_operatorfunc_option() wrapper (EXITFREE only).
#if defined(EXITFREE)
void nvim_call_free_operatorfunc_option(void) { free_operatorfunc_option(); }
#else
void nvim_call_free_operatorfunc_option(void) {}
#endif




// =============================================================================
// optexpand_T field accessors for Rust expand.rs
// =============================================================================

char *nvim_oe_get_opt_value(const optexpand_T *args) { return args->oe_opt_value; }
const char *nvim_oe_get_set_arg(const optexpand_T *args) { return args->oe_set_arg; }
bool nvim_oe_get_append(const optexpand_T *args) { return args->oe_append; }
bool nvim_oe_get_include_orig_val(const optexpand_T *args) { return args->oe_include_orig_val; }
regmatch_T *nvim_oe_get_regmatch(const optexpand_T *args) { return args->oe_regmatch; }
expand_T *nvim_oe_get_xp(const optexpand_T *args) { return args->oe_xp; }
char *nvim_oe_get_varp(const optexpand_T *args) { return args->oe_varp; }
int nvim_oe_get_idx(const optexpand_T *args) { return (int)args->oe_idx; }

// Option value array accessors for expand_set_str_generic
const char **nvim_option_get_values(const vimoption_T *opt) { return (const char **)opt->values; }
size_t nvim_option_get_values_len(const vimoption_T *opt) { return opt->values_len; }


// Window p_lcs accessor
const char *nvim_win_get_p_lcs(const win_T *win) { return win ? win->w_p_lcs : NULL; }

// =============================================================================
// Phase 4 (optionstr): check_str_opt infrastructure for Rust migration
// =============================================================================


