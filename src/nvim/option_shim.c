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
extern char *rs_stropt_get_newval(int nextchar, int opt_idx, char **argp, void *varp,
                                  const char *origval, int *op_arg, uint32_t flags);

// Rust metadata query functions (option pass 8 phase 1)
// is_option_hidden, option_has_type, option_has_scope now exported directly from Rust via #[export_name]
extern int rs_option_is_global_local(int opt_idx);
// New functions added in Phase 8 index.rs
extern int rs_option_get_type(int opt_idx);
extern int rs_option_scope_idx(int opt_idx, int scope);

// Rust default value management functions (option pass 8 phase 2)
extern void rs_alloc_options_default(void);
extern void rs_change_option_default(int opt_idx, OptVal value);
extern void rs_set_string_default_opt(int opt_idx, char *val, int allocated);
extern void rs_set_option_default(int opt_idx, int opt_flags);

typedef struct { const char *end; int opt_idx; } FindOptionEndResult;
extern FindOptionEndResult rs_find_option_end(const char *arg);

typedef struct { int result; const char *errmsg; } ValidateOptIdxResult;
extern ValidateOptIdxResult rs_validate_opt_idx(win_T *win, OptIndex opt_idx, int opt_flags,
                                                uint32_t flags, int prefix);

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
extern const char *rs_unset_option_local_value(int opt_idx);

// Rust FFI declarations (window/layout module)
// rs_free_tagfunc_option, rs_set_buflocal_tfu_callback: declared in other C files that use them
// rs_global_stl_height, rs_min_rows, rs_min_rows_for_all_tabpages, rs_tabline_height,
// rs_win_comp_pos, rs_win_default_scroll, rs_win_setheight, rs_win_setwidth: declared elsewhere
extern void rs_last_status(int morewin);
extern tabpage_T *rs_win_find_tabpage(win_T *win);

// =============================================================================
// Accessor functions for Rust code
// =============================================================================

// optset_T field accessors for Rust callbacks
void *nvim_optset_get_win(const void *args) { return (void *)((const optset_T *)args)->os_win; }
void *nvim_optset_get_buf(const void *args) { return (void *)((const optset_T *)args)->os_buf; }
int nvim_optset_get_oldval_boolean(const void *args) { return (int)((const optset_T *)args)->os_oldval.boolean; }
int64_t nvim_optset_get_oldval_number(const void *args) { return ((const optset_T *)args)->os_oldval.number; }
int64_t nvim_optset_get_newval_number(const void *args) { return ((const optset_T *)args)->os_newval.number; }
void *nvim_optset_get_varp(const void *args) { return ((const optset_T *)args)->os_varp; }
int nvim_optset_get_newval_boolean(const void *args) { return (int)((const optset_T *)args)->os_newval.boolean; }
int nvim_optset_get_flags(const void *args) { return ((const optset_T *)args)->os_flags; }

// String option accessors
const char *nvim_option_get_sh(void) { return p_sh; }
const char *nvim_option_get_cpo(void) { return p_cpo; }
const char *nvim_option_get_isf(void) { return p_isf; }
const char *nvim_option_get_isp(void) { return p_isp; }
const char *nvim_option_get_isi(void) { return p_isi; }
const char *nvim_option_get_breakat(void) { return p_breakat; }
const char *nvim_option_get_sel(void) { return p_sel; }
const char *nvim_option_get_enc(void) { return p_enc; }
const char *nvim_option_get_ww(void) { return p_ww; }
const char *nvim_option_get_mouse(void) { return p_mouse; }
const char *nvim_option_get_shm(void) { return p_shm; }

// Boolean option accessors
int nvim_option_get_ic(void) { return p_ic; }
int nvim_option_get_scs(void) { return p_scs; }
int nvim_option_get_hls(void) { return p_hls; }
int nvim_option_get_is(void) { return p_is; }
int nvim_option_get_magic(void) { return p_magic; }
int nvim_option_get_fic(void) { return p_fic; }
int nvim_option_get_mle(void) { return p_mle; }
int nvim_option_get_paste(void) { return p_paste; }
int nvim_option_get_ri(void) { return p_ri; }
int nvim_option_get_ws(void) { return p_ws; }
int nvim_option_get_gd(void) { return p_gd; }
int nvim_option_get_ea(void) { return p_ea; }
int nvim_option_get_hid(void) { return p_hid; }
int nvim_option_get_sm(void) { return p_sm; }
int nvim_option_get_lz(void) { return p_lz; }
int nvim_option_get_to(void) { return p_to; }

// Numeric option accessors
OptInt nvim_option_get_so(void) { return p_so; }
OptInt nvim_option_get_siso(void) { return p_siso; }
OptInt nvim_option_get_report(void) { return p_report; }
OptInt nvim_option_get_mat(void) { return p_mat; }
OptInt nvim_option_get_ut(void) { return p_ut; }
OptInt nvim_option_get_tm(void) { return p_tm; }
OptInt nvim_option_get_hi(void) { return p_hi; }
OptInt nvim_option_get_ls(void) { return p_ls; }
OptInt nvim_option_get_stal(void) { return p_stal; }
OptInt nvim_option_get_re(void) { return p_re; }

// Flag option accessors (unsigned)
unsigned nvim_option_get_cot_flags(void) { return cot_flags; }
unsigned nvim_option_get_fdo_flags(void) { return fdo_flags; }
unsigned nvim_option_get_dy_flags(void) { return dy_flags; }
unsigned nvim_option_get_cb_flags(void) { return cb_flags; }

// Special accessors
int nvim_option_get_magic_overruled(void) { return (int)magic_overruled; }
int nvim_option_get_secure(void) { return secure; }

const char *nvim_get_curbuf_sua(void) { return curbuf->b_p_sua; }

// =============================================================================
// Setter functions for Rust code
// =============================================================================

// Boolean option setters (direct assignment for simple options)
void nvim_option_set_ai(int value) { p_ai = value; }
void nvim_option_set_et(int value) { p_et = value; }
void nvim_option_set_ic(int value) { p_ic = value; }
void nvim_option_set_scs(int value) { p_scs = value; }
void nvim_option_set_hls(int value) { p_hls = value; }
void nvim_option_set_is(int value) { p_is = value; }
void nvim_option_set_magic(int value) { p_magic = value; }
void nvim_option_set_ml(int value) { p_ml = value; }
void nvim_option_set_paste(int value) { p_paste = value; }
void nvim_option_set_ri(int value) { p_ri = value; }
void nvim_option_set_ws(int value) { p_ws = value; }
void nvim_option_set_gd(int value) { p_gd = value; }
void nvim_option_set_ea(int value) { p_ea = value; }
void nvim_option_set_hid(int value) { p_hid = value; }
void nvim_option_set_sm(int value) { p_sm = value; }
void nvim_option_set_lz(int value) { p_lz = value; }
void nvim_option_set_to(int value) { p_to = value; }

// Numeric option setters (direct assignment for simple options)
void nvim_option_set_sw(OptInt value) { p_sw = value; }
void nvim_option_set_ts(OptInt value) { p_ts = value; }
void nvim_option_set_sts(OptInt value) { p_sts = value; }
void nvim_option_set_tw(OptInt value) { p_tw = value; }
void nvim_option_set_wm(OptInt value) { p_wm = value; }
void nvim_option_set_so(OptInt value) { p_so = value; }
void nvim_option_set_siso(OptInt value) { p_siso = value; }
void nvim_option_set_report(OptInt value) { p_report = value; }
void nvim_option_set_mat(OptInt value) { p_mat = value; }
void nvim_option_set_ut(OptInt value) { p_ut = value; }
void nvim_option_set_tm(OptInt value) { p_tm = value; }
void nvim_option_set_hi(OptInt value) { p_hi = value; }
void nvim_option_set_re(OptInt value) { p_re = value; }

// Special setter for magic_overruled (for Rust)
void nvim_option_set_magic_overruled(int value) { magic_overruled = (optmagic_T)value; }

// =============================================================================
// Callback accessor functions for Rust callbacks module
// =============================================================================

// State accessors for callbacks
int nvim_callback_get_starting(void) { return starting; }
OptInt nvim_callback_get_p_titlelen(void) { return p_titlelen; }
int nvim_callback_get_no_hlsearch(void) { return no_hlsearch; }

// State setters for callbacks
void nvim_callback_set_need_maketitle(int value) { need_maketitle = value != 0; }

// Window diff accessor
int nvim_win_get_diff(win_T *win) { return win ? win->w_p_diff : 0; }

// Full screen state (option module specific)
int nvim_option_get_full_screen(void) { return full_screen; }

// Screen dimensions
int nvim_option_get_rows(void) { return Rows; }

// Window accessor for view height (option module specific)
int nvim_option_win_get_view_height(win_T *win) { return win ? win->w_view_height : 0; }

// Global option value accessors for validation
OptInt nvim_option_get_p_wmh(void) { return p_wmh; }
OptInt nvim_option_get_p_wh(void) { return p_wh; }
OptInt nvim_option_get_p_wmw(void) { return p_wmw; }
OptInt nvim_option_get_p_wiw(void) { return p_wiw; }

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

// Display callback accessors
OptInt nvim_callback_get_p_ch(void) { return p_ch; }
void nvim_callback_set_p_ch(OptInt value) { p_ch = value; }
frame_T *nvim_callback_get_topframe(void) { return topframe; }
int nvim_callback_get_topframe_fr_height(void) { return topframe->fr_height; }
void nvim_callback_set_clear_cmdline(int value) { clear_cmdline = value != 0; }

// Window accessors for display callbacks
const char *nvim_option_win_get_stc(win_T *win) { return win ? (const char *)win->w_p_stc : NULL; }
void nvim_option_win_set_nrwidth(win_T *win, int value) { if (win) win->w_nrwidth_line_count = value; }
int nvim_option_win_get_sms(win_T *win) { return win ? win->w_p_sms : 0; }
void nvim_option_win_set_skipcol(win_T *win, int value) { if (win) win->w_skipcol = value; }

// Behavior callback accessors
OptInt nvim_callback_get_p_uc(void) { return p_uc; }
int nvim_callback_is_one_window(void) { return ONE_WINDOW; }
int nvim_callback_is_curbuf_help(void) { return curbuf->b_help; }
int nvim_callback_get_curwin_height(void) { return curwin->w_height; }
int nvim_callback_get_curwin_width(void) { return curwin->w_width; }
OptInt nvim_callback_get_p_hh(void) { return p_hh; }

// Buffer accessors for behavior callbacks
int nvim_buf_get_p_swf(buf_T *buf) { return buf ? buf->b_p_swf : 0; }
int nvim_buf_get_p_udf(buf_T *buf) { return buf ? buf->b_p_udf : 0; }
void nvim_option_buf_set_modified_was_set(buf_T *buf, int val) { if (buf) buf->b_modified_was_set = val; }
int nvim_option_buf_get_b_p_ro(buf_T *buf) { return buf ? buf->b_p_ro : 0; }
void nvim_option_buf_set_b_did_warn(buf_T *buf, int val) { if (buf) buf->b_did_warn = val != 0; }
void nvim_callback_set_readonlymode(int val) { readonlymode = val != 0; }
void *nvim_option_buf_get_terminal_ptr(buf_T *buf) { return buf ? buf->terminal : NULL; }
int nvim_option_buf_get_b_p_bin(buf_T *buf) { return buf ? buf->b_p_bin : 0; }
void *nvim_callback_get_p_ul_addr(void) { return (void *)&p_ul; }

// Phase 88: undolevels accessors
void nvim_set_p_ul(OptInt val) { p_ul = val; }
void nvim_buf_set_b_p_ul(buf_T *buf, OptInt val) { buf->b_p_ul = val; }

// Phase 91: langmap accessor
const char *nvim_get_p_langmap(void) { return p_langmap; }

// Phase 95: spell option accessors
int nvim_valid_spellfile(const char *val) { return valid_spellfile(val) ? 1 : 0; }
int nvim_valid_spelllang(const char *val) { return valid_spelllang(val) ? 1 : 0; }
const char *nvim_did_set_spell_option(void) { return did_set_spell_option(); }

// Colorcolumn check wrapper
void check_colorcolumn_win(win_T *win) { check_colorcolumn(NULL, win); }

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
int nvim_get_p_udf(void) { return p_udf; }

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
const char *nvim_parse_spelllang(win_T *win) { return parse_spelllang(win); }

// Shiftwidth/tabstop callback accessors
void nvim_parse_cino(buf_T *buf) { parse_cino(buf); }
void *nvim_buf_get_b_p_sw_addr(buf_T *buf) { return buf ? (void *)&buf->b_p_sw : NULL; }
OptInt nvim_buf_get_b_p_sw(buf_T *buf) { return buf ? buf->b_p_sw : 0; }

// Xhistory callback accessors
void *nvim_get_p_chi_addr(void) { return (void *)&p_chi; }

// Langnoremap/langremap toggle accessors
int nvim_callback_get_p_lnr(void) { return p_lnr; }
int nvim_callback_get_p_lrm(void) { return p_lrm; }
void nvim_callback_set_p_lnr(int value) { p_lnr = value; }
void nvim_callback_set_p_lrm(int value) { p_lrm = value; }

// Pumblend accessors
OptInt nvim_callback_get_p_pb(void) { return p_pb; }
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

// Error message accessor
const char *nvim_callback_get_e_invarg(void) { return e_invarg; }

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
// Wrapper for illegal_char (formats E539 error into errbuf)
const char *nvim_illegal_char(char *errbuf, size_t errbuflen, int c)
{
  return illegal_char(errbuf, errbuflen, c);
}
// Wrapper for did_set_str_generic (validates against option's allowed values)
const char *nvim_did_set_str_generic(void *args) { return did_set_str_generic(args); }
// Wrappers for side-effect functions
void nvim_call_init_chartab(void) { init_chartab(); }
void nvim_call_msg_grid_validate(void) { msg_grid_validate(); }
int nvim_call_check_opt_wim(void) { return check_opt_wim(); }
int nvim_call_briopt_check_win(const char *val, win_T *win)
{
  return briopt_check(val, win) == OK ? 1 : 0;
}
int nvim_get_cmdpreview(void) { return cmdpreview; }
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
// Return os_oldval.string.data from optset_T
const char *nvim_optset_get_oldval_str(const void *args)
{
  return ((const optset_T *)args)->os_oldval.string.data;
}
// Global bkc_flags accessors
unsigned nvim_get_bkc_flags(void) { return bkc_flags; }
void nvim_set_bkc_flags(unsigned val) { bkc_flags = val; }
// Buffer-local bkc accessors
unsigned nvim_buf_get_bkc_flags(buf_T *buf) { return buf->b_bkc_flags; }
void nvim_buf_set_bkc_flags(buf_T *buf, unsigned val) { buf->b_bkc_flags = val; }
const char *nvim_buf_get_p_bkc(buf_T *buf) { return buf->b_p_bkc; }
const char *nvim_get_p_bkc(void) { return p_bkc; }
// Global ssop_flags accessors
unsigned nvim_get_ssop_flags(void) { return ssop_flags; }
void nvim_set_ssop_flags(unsigned val) { ssop_flags = val; }
// Global spo_flags accessors
unsigned nvim_get_spo_flags(void) { return spo_flags; }
void nvim_set_spo_flags(unsigned val) { spo_flags = val; }
// Window-local spo_flags accessors
unsigned nvim_win_get_spo_flags(win_T *win) { return win->w_s->b_p_spo_flags; }
void nvim_win_set_spo_flags(win_T *win, unsigned val) { win->w_s->b_p_spo_flags = val; }
// Pointer accessors for option value arrays (return pointer to NULL-terminated array)
const char **nvim_get_opt_bkc_values(void) { return (const char **)opt_bkc_values; }
const char **nvim_get_opt_ssop_values(void) { return (const char **)opt_ssop_values; }
const char **nvim_get_opt_spo_values(void) { return (const char **)opt_spo_values; }

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
// syn_check_group wrapper (reuse existing signature: name + len)
int nvim_syn_check_group_for_winhl(const char *name, size_t len)
{
  return syn_check_group(name, len);
}
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

// IO buffer accessor
char *nvim_get_iobuff(void) { return IObuff; }

// no_wait_return state accessors
int nvim_get_no_wait_return(void) { return no_wait_return; }
void nvim_set_no_wait_return(int val) { no_wait_return = val; }

// silent_mode state accessor
void nvim_set_silent_mode(int val) { silent_mode = val != 0; }

// info_message state accessor
void nvim_set_info_message(int val) { info_message = val != 0; }

// Global option accessors for Rust callbacks
const char *nvim_get_p_path(void) { return p_path; }
const char *nvim_get_p_cdpath(void) { return p_cdpath; }
OptInt nvim_get_p_window(void) { return p_window; }
void nvim_set_p_window(OptInt val) { p_window = val; }
void nvim_set_p_arshape(int val) { p_arshape = val != 0; }
const char *nvim_get_p_enc(void) { return (const char *)p_enc; }
void nvim_set_p_deco(int val) { p_deco = val != 0; }

// Lines/columns callback accessors
OptInt nvim_get_p_lines(void) { return p_lines; }
OptInt nvim_get_p_columns(void) { return p_columns; }
int nvim_option_was_set_window(void) { return option_was_set(kOptWindow); }

// Paste callback accessors (nvim_get_p_paste defined in indent_c.c,
//   nvim_get_p_ru defined in drawscreen.c)
int nvim_get_p_sta(void) { return p_sta; }
void nvim_set_p_sta(int val) { p_sta = val != 0; }

// fill_culopt_flags accessors
const char *nvim_win_get_p_culopt(win_T *wp) { return wp ? wp->w_p_culopt : NULL; }
void nvim_win_set_p_culopt_flags(win_T *wp, uint8_t flags) { if (wp) wp->w_p_culopt_flags = flags; }

// set_options_bin global option accessors
OptInt nvim_get_p_tw(void) { return p_tw; }
OptInt nvim_get_p_wm(void) { return p_wm; }
void nvim_set_p_bin(int v) { p_bin = v != 0; }

// set_helplang_default accessors
void nvim_set_p_hlg_from_code(const char *code)
{
  free_string_option(p_hlg);
  // code is a 2-char null-terminated string (or empty)
  p_hlg = (code && code[0]) ? xstrdup(code) : xstrdup("");
}

static const char e_unknown_option[]
  = N_("E518: Unknown option");
static const char e_not_allowed_in_modeline[]
  = N_("E520: Not allowed in a modeline");
static const char e_not_allowed_in_modeline_when_modelineexpr_is_off[]
  = N_("E992: Not allowed in a modeline when 'modelineexpr' is off");
static const char e_number_required_after_equal[]
  = N_("E521: Number required after =");
static const char e_preview_window_already_exists[]
  = N_("E590: A preview window already exists");
const char *nvim_get_e_preview_window_exists(void) { return _(e_preview_window_already_exists); }
static char *p_term = NULL;
static char *p_ttytype = NULL;

// Saved values for when 'bin' is set.
static int p_et_nobin;
static int p_ml_nobin;
static OptInt p_tw_nobin;
static OptInt p_wm_nobin;

// Saved values for when 'paste' is set.
static int p_ai_nopaste;
static int p_et_nopaste;
static OptInt p_sts_nopaste;
static OptInt p_tw_nopaste;
static OptInt p_wm_nopaste;
static char *p_vsts_nopaste;

#define OPTION_COUNT ARRAY_SIZE(options)

/// :set boolean option prefix
typedef enum {
  PREFIX_NO = 0,  ///< "no" prefix
  PREFIX_NONE,    ///< no prefix
  PREFIX_INV,     ///< "inv" prefix
} set_prefix_T;

#include "option_shim.c.generated.h"

// options[] is initialized in options.generated.h.
// The options with a NULL variable are 'hidden': a set command for them is
// ignored and they are not printed.

#include "options.generated.h"
#include "options_map.generated.h"

// After options[] is available:
int nvim_option_hlg_was_set(void) { return (options[kOptHelplang].flags & kOptFlagWasSet) != 0; }

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
// Clears kOptFlagWasSet from options[opt_idx].flags (write accessor for Rust)
void nvim_option_clear_was_set_flag(OptIndex opt_idx) { options[opt_idx].flags &= ~(uint32_t)kOptFlagWasSet; }

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
// Returns the NO_LOCAL_UNDOLEVEL sentinel constant.
int64_t nvim_get_no_local_undolevel(void) { return NO_LOCAL_UNDOLEVEL; }
// Returns a static C string naming the OptValType: "nil", "boolean", "number", "string".
const char *nvim_optval_type_get_name(int type) { return optval_type_get_name((OptValType)type); }
// Allocates and returns a string representation of an OptVal (caller must xfree).
char *nvim_optval_to_cstr_alloc(OptVal o) { return rs_optval_to_cstr(o); }
// Returns translated "Cannot unset global option value" string pointer.
const char *nvim_errmsg_no_unset_global(void) { return _("Cannot unset global option value"); }


// Fill offset table for buf_T option fields indexed by OptIndex.
// Writes offsetof(buf_T, field) into out[idx] for each handled OptIndex.
// Unhandled indices receive -1 (sentinel). len must equal kOptCount.
void nvim_buf_opt_field_offsets(ptrdiff_t *out, int len)
{
  // Initialize all to -1 (unhandled)
  for (int i = 0; i < len; i++) {
    out[i] = -1;
  }
  // global-local string options
  out[kOptEqualprg]      = offsetof(buf_T, b_p_ep);
  out[kOptKeywordprg]    = offsetof(buf_T, b_p_kp);
  out[kOptPath]          = offsetof(buf_T, b_p_path);
  out[kOptTags]          = offsetof(buf_T, b_p_tags);
  out[kOptTagcase]       = offsetof(buf_T, b_p_tc);
  out[kOptBackupcopy]    = offsetof(buf_T, b_p_bkc);
  out[kOptDefine]        = offsetof(buf_T, b_p_def);
  out[kOptInclude]       = offsetof(buf_T, b_p_inc);
  out[kOptCompleteopt]   = offsetof(buf_T, b_p_cot);
  out[kOptDictionary]    = offsetof(buf_T, b_p_dict);
  out[kOptDiffanchors]   = offsetof(buf_T, b_p_dia);
  out[kOptThesaurus]     = offsetof(buf_T, b_p_tsr);
  out[kOptThesaurusfunc] = offsetof(buf_T, b_p_tsrfu);
  out[kOptFormatprg]     = offsetof(buf_T, b_p_fp);
  out[kOptFindfunc]      = offsetof(buf_T, b_p_ffu);
  out[kOptErrorformat]   = offsetof(buf_T, b_p_efm);
  out[kOptGrepformat]    = offsetof(buf_T, b_p_gefm);
  out[kOptGrepprg]       = offsetof(buf_T, b_p_gp);
  out[kOptMakeprg]       = offsetof(buf_T, b_p_mp);
  out[kOptLispwords]     = offsetof(buf_T, b_p_lw);
  out[kOptMakeencoding]  = offsetof(buf_T, b_p_menc);
  // global-local numeric options
  out[kOptAutocomplete]  = offsetof(buf_T, b_p_ac);
  out[kOptAutoread]      = offsetof(buf_T, b_p_ar);
  out[kOptUndolevels]    = offsetof(buf_T, b_p_ul);
  // buf-local options (non-global-local)
  out[kOptAutoindent]    = offsetof(buf_T, b_p_ai);
  out[kOptBinary]        = offsetof(buf_T, b_p_bin);
  out[kOptBomb]          = offsetof(buf_T, b_p_bomb);
  out[kOptBufhidden]     = offsetof(buf_T, b_p_bh);
  out[kOptBuftype]       = offsetof(buf_T, b_p_bt);
  out[kOptBuflisted]     = offsetof(buf_T, b_p_bl);
  out[kOptBusy]          = offsetof(buf_T, b_p_busy);
  out[kOptChannel]       = offsetof(buf_T, b_p_channel);
  out[kOptCopyindent]    = offsetof(buf_T, b_p_ci);
  out[kOptCindent]       = offsetof(buf_T, b_p_cin);
  out[kOptCinkeys]       = offsetof(buf_T, b_p_cink);
  out[kOptCinoptions]    = offsetof(buf_T, b_p_cino);
  out[kOptCinscopedecls] = offsetof(buf_T, b_p_cinsd);
  out[kOptCinwords]      = offsetof(buf_T, b_p_cinw);
  out[kOptComments]      = offsetof(buf_T, b_p_com);
  out[kOptCommentstring] = offsetof(buf_T, b_p_cms);
  out[kOptComplete]      = offsetof(buf_T, b_p_cpt);
#ifdef BACKSLASH_IN_FILENAME
  out[kOptCompleteslash] = offsetof(buf_T, b_p_csl);
#endif
  out[kOptCompletefunc]  = offsetof(buf_T, b_p_cfu);
  out[kOptOmnifunc]      = offsetof(buf_T, b_p_ofu);
  out[kOptEndoffile]     = offsetof(buf_T, b_p_eof);
  out[kOptEndofline]     = offsetof(buf_T, b_p_eol);
  out[kOptFixendofline]  = offsetof(buf_T, b_p_fixeol);
  out[kOptExpandtab]     = offsetof(buf_T, b_p_et);
  out[kOptFileencoding]  = offsetof(buf_T, b_p_fenc);
  out[kOptFileformat]    = offsetof(buf_T, b_p_ff);
  out[kOptFiletype]      = offsetof(buf_T, b_p_ft);
  out[kOptFormatoptions] = offsetof(buf_T, b_p_fo);
  out[kOptFormatlistpat] = offsetof(buf_T, b_p_flp);
  out[kOptIminsert]      = offsetof(buf_T, b_p_iminsert);
  out[kOptImsearch]      = offsetof(buf_T, b_p_imsearch);
  out[kOptInfercase]     = offsetof(buf_T, b_p_inf);
  out[kOptIskeyword]     = offsetof(buf_T, b_p_isk);
  out[kOptIncludeexpr]   = offsetof(buf_T, b_p_inex);
  out[kOptIndentexpr]    = offsetof(buf_T, b_p_inde);
  out[kOptIndentkeys]    = offsetof(buf_T, b_p_indk);
  out[kOptFormatexpr]    = offsetof(buf_T, b_p_fex);
  out[kOptLisp]          = offsetof(buf_T, b_p_lisp);
  out[kOptLispoptions]   = offsetof(buf_T, b_p_lop);
  out[kOptModeline]      = offsetof(buf_T, b_p_ml);
  out[kOptMatchpairs]    = offsetof(buf_T, b_p_mps);
  out[kOptModifiable]    = offsetof(buf_T, b_p_ma);
  out[kOptModified]      = offsetof(buf_T, b_changed);
  out[kOptNrformats]     = offsetof(buf_T, b_p_nf);
  out[kOptPreserveindent]= offsetof(buf_T, b_p_pi);
  out[kOptQuoteescape]   = offsetof(buf_T, b_p_qe);
  out[kOptReadonly]      = offsetof(buf_T, b_p_ro);
  out[kOptScrollback]    = offsetof(buf_T, b_p_scbk);
  out[kOptSmartindent]   = offsetof(buf_T, b_p_si);
  out[kOptSofttabstop]   = offsetof(buf_T, b_p_sts);
  out[kOptSuffixesadd]   = offsetof(buf_T, b_p_sua);
  out[kOptSwapfile]      = offsetof(buf_T, b_p_swf);
  out[kOptSynmaxcol]     = offsetof(buf_T, b_p_smc);
  out[kOptSyntax]        = offsetof(buf_T, b_p_syn);
  out[kOptShiftwidth]    = offsetof(buf_T, b_p_sw);
  out[kOptTagfunc]       = offsetof(buf_T, b_p_tfu);
  out[kOptTabstop]       = offsetof(buf_T, b_p_ts);
  out[kOptTextwidth]     = offsetof(buf_T, b_p_tw);
  out[kOptUndofile]      = offsetof(buf_T, b_p_udf);
  out[kOptWrapmargin]    = offsetof(buf_T, b_p_wm);
  out[kOptVarsofttabstop]= offsetof(buf_T, b_p_vsts);
  out[kOptVartabstop]    = offsetof(buf_T, b_p_vts);
  out[kOptKeymap]        = offsetof(buf_T, b_p_keymap);
}

// Get the address of a win_T option field by OptIndex.
// Returns NULL for unknown/unhandled indices.
void *nvim_win_get_opt_field_addr(win_T *win, OptIndex idx)
{
  if (!win) { return NULL; }
  switch (idx) {
  // global-local numeric win options
  case kOptSidescrolloff: return &win->w_p_siso;
  case kOptScrolloff: return &win->w_p_so;
  // global-local string win options
  case kOptShowbreak: return &win->w_p_sbr;
  case kOptStatusline: return &win->w_p_stl;
  case kOptWinbar: return &win->w_p_wbr;
  case kOptFillchars: return &win->w_p_fcs;
  case kOptListchars: return &win->w_p_lcs;
  case kOptVirtualedit: return &win->w_p_ve;
  // win-local only options
  case kOptArabic: return &win->w_p_arab;
  case kOptList: return &win->w_p_list;
  case kOptSpell: return &win->w_p_spell;
  case kOptCursorcolumn: return &win->w_p_cuc;
  case kOptCursorline: return &win->w_p_cul;
  case kOptCursorlineopt: return &win->w_p_culopt;
  case kOptColorcolumn: return &win->w_p_cc;
  case kOptDiff: return &win->w_p_diff;
  case kOptEventignorewin: return &win->w_p_eiw;
  case kOptFoldcolumn: return &win->w_p_fdc;
  case kOptFoldenable: return &win->w_p_fen;
  case kOptFoldignore: return &win->w_p_fdi;
  case kOptFoldlevel: return &win->w_p_fdl;
  case kOptFoldmethod: return &win->w_p_fdm;
  case kOptFoldminlines: return &win->w_p_fml;
  case kOptFoldnestmax: return &win->w_p_fdn;
  case kOptFoldexpr: return &win->w_p_fde;
  case kOptFoldtext: return &win->w_p_fdt;
  case kOptFoldmarker: return &win->w_p_fmr;
  case kOptNumber: return &win->w_p_nu;
  case kOptRelativenumber: return &win->w_p_rnu;
  case kOptNumberwidth: return &win->w_p_nuw;
  case kOptWinfixbuf: return &win->w_p_wfb;
  case kOptWinfixheight: return &win->w_p_wfh;
  case kOptWinfixwidth: return &win->w_p_wfw;
  case kOptPreviewwindow: return &win->w_p_pvw;
  case kOptLhistory: return &win->w_p_lhi;
  case kOptRightleft: return &win->w_p_rl;
  case kOptRightleftcmd: return &win->w_p_rlc;
  case kOptScroll: return &win->w_p_scr;
  case kOptSmoothscroll: return &win->w_p_sms;
  case kOptWrap: return &win->w_p_wrap;
  case kOptLinebreak: return &win->w_p_lbr;
  case kOptBreakindent: return &win->w_p_bri;
  case kOptBreakindentopt: return &win->w_p_briopt;
  case kOptScrollbind: return &win->w_p_scb;
  case kOptCursorbind: return &win->w_p_crb;
  case kOptConcealcursor: return &win->w_p_cocu;
  case kOptConceallevel: return &win->w_p_cole;
  // win->w_s (synblock) fields accessed via win
  case kOptSpellcapcheck: return &win->w_s->b_p_spc;
  case kOptSpellfile: return &win->w_s->b_p_spf;
  case kOptSpelllang: return &win->w_s->b_p_spl;
  case kOptSpelloptions: return &win->w_s->b_p_spo;
  case kOptSigncolumn: return &win->w_p_scl;
  case kOptWinhighlight: return &win->w_p_winhl;
  case kOptWinblend: return &win->w_p_winbl;
  case kOptStatuscolumn: return &win->w_p_stc;
  default: abort();
  }
}

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
void nvim_set_options_default(int opt_flags);
void nvim_didset_options(void);
void nvim_didset_options2(void);
int nvim_validate_opt_idx(win_T *win, OptIndex opt_idx, int opt_flags, uint32_t flags,
                          int prefix, const char **errmsg);

// set_fileformat helper: set the 'fileformat' option string and trigger redraws
void nvim_set_fileformat_option(const char *p, int opt_flags)
{
  if (p != NULL) {
    set_option_direct(kOptFileformat, CSTR_AS_OPTVAL(p), opt_flags, 0);
  }
  redraw_buf_status_later(curbuf);
  redraw_tabline = true;
  need_maketitle = true;
}

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
// set_options_bin helpers: nobin statics
OptInt nvim_get_p_tw_nobin(void) { return p_tw_nobin; }
void nvim_set_p_tw_nobin(OptInt v) { p_tw_nobin = v; }
OptInt nvim_get_p_wm_nobin(void) { return p_wm_nobin; }
void nvim_set_p_wm_nobin(OptInt v) { p_wm_nobin = v; }
int nvim_get_p_ml_nobin(void) { return p_ml_nobin; }
void nvim_set_p_ml_nobin(int v) { p_ml_nobin = v != 0; }
int nvim_get_p_et_nobin(void) { return p_et_nobin; }
void nvim_set_p_et_nobin(int v) { p_et_nobin = v != 0; }

// =============================================================================
// Phase 4 (option pass 4) accessors for query.rs migration
// =============================================================================

// p_bs accessor (for can_bs)
const char *nvim_option_get_p_bs(void) { return p_bs; }
// bt_prompt(curbuf) accessor (for can_bs)
int nvim_curbuf_is_prompt(void) { return bt_prompt(curbuf); }
// b_p_ep / p_ep accessors (for get_equalprg)
const char *nvim_curbuf_get_b_p_ep(void) { return curbuf->b_p_ep; }
const char *nvim_option_get_p_ep(void) { return p_ep; }
// b_p_ffu / p_ffu accessors (for get_findfunc)
const char *nvim_curbuf_get_b_p_ffu(void) { return curbuf->b_p_ffu; }
const char *nvim_option_get_p_ffu(void) { return p_ffu; }
// p_flp / b_p_flp accessors (for get_flp_value)
const char *nvim_buf_get_p_flp(buf_T *buf) { return buf->b_p_flp; }
// ve_flags accessors (for get_ve_flags)
unsigned nvim_get_ve_flags_global(void) { return ve_flags; }
unsigned nvim_win_get_ve_flags(win_T *wp) { return wp->w_ve_flags; }
// iminsert/imsearch global accessors (for set_iminsert/imsearch_global)
OptInt nvim_get_p_iminsert(void) { return p_iminsert; }
void nvim_set_p_iminsert(OptInt v) { p_iminsert = v; }
OptInt nvim_get_p_imsearch(void) { return p_imsearch; }
void nvim_set_p_imsearch(OptInt v) { p_imsearch = v; }
// buf b_p_iminsert/imsearch accessors (for set_iminsert/imsearch_global)
OptInt nvim_buf_get_b_p_iminsert(buf_T *buf) { return buf->b_p_iminsert; }
OptInt nvim_buf_get_b_p_imsearch(buf_T *buf) { return buf->b_p_imsearch; }
// p_ma / b_p_ma accessors (for reset_modifiable)
void nvim_option_set_p_ma(int v) { p_ma = v != 0; }
int nvim_curbuf_get_b_p_ma(void) { return curbuf->b_p_ma; }
void nvim_curbuf_set_b_p_ma(int v) { curbuf->b_p_ma = v != 0; }
// change_option_default wrapper (for reset_modifiable)
void nvim_change_option_default_bool(OptIndex opt_idx, int value) { rs_change_option_default(opt_idx, BOOLEAN_OPTVAL(value != 0)); }
// TTY and key accessors
int nvim_option_get_t_colors(void) { return t_colors; }
const char *nvim_option_get_p_term(void) { return p_term; }
const char *nvim_option_get_p_ttytype(void) { return p_ttytype; }
void nvim_option_set_p_term(char *val) { p_term = val; }
void nvim_option_set_p_ttytype(char *val) { p_ttytype = val; }
// FullName_save wrapper (for vimrc_found)
char *nvim_option_FullName_save(const char *fname, bool force) { return FullName_save(fname, force); }
// vim_getenv wrapper (for vimrc_found)
char *nvim_option_vim_getenv(const char *envname) { return vim_getenv(envname); }
// os_setenv wrapper (for vimrc_found)
int nvim_option_os_setenv(const char *name, const char *value, int overwrite) { return os_setenv(name, value, overwrite); }

// Phase 4 pass 3: check_redraw_for accessors
void nvim_call_status_redraw_all(void) { status_redraw_all(); }
void nvim_call_changed_window_setting(win_T *win) { changed_window_setting(win); }
void nvim_call_redraw_later(win_T *win, int type) { redraw_later(win, type); }
void nvim_call_redraw_buf_later(buf_T *buf, int type) { redraw_buf_later(buf, type); }
void nvim_call_redraw_all_later(int type) { redraw_all_later(type); }

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
// set_option_sctx accessors (nvim_get_sourcing_lnum already defined in ex_docmd.c as int)
int64_t nvim_option_get_sourcing_lnum(void) { return (int64_t)SOURCING_LNUM; }
void nvim_call_nlua_set_sctx(sctx_T *sctx) { nlua_set_sctx(sctx); }
void nvim_curbuf_set_p_script_ctx(int idx, sctx_T sctx) { curbuf->b_p_script_ctx[idx] = sctx; }
void nvim_curwin_set_p_script_ctx(int idx, sctx_T sctx) { curwin->w_p_script_ctx[idx] = sctx; }
void nvim_curwin_set_allbuf_opt_script_ctx(int idx, sctx_T sctx) { curwin->w_allbuf_opt.wo_script_ctx[idx] = sctx; }
void nvim_option_set_script_ctx(OptIndex opt_idx, sctx_T sctx) { options[opt_idx].script_ctx = sctx; }

// Phase 4 (session.rs) accessors
OptVal nvim_optval_from_varp(OptIndex opt_idx, void *varp) { return optval_from_varp(opt_idx, varp); }
void *nvim_get_varp_scope_by_idx(OptIndex opt_idx, int opt_flags)
{
  return get_varp_scope(&options[opt_idx], opt_flags);
}
// nvim_get_namebuff is defined in buffer.c
size_t nvim_get_namebuff_size(void) { return MAXPATHL; }
void nvim_option_home_replace(const char *src, char *dst, size_t dstlen)
{
  home_replace(NULL, src, dst, dstlen, false);
}
int nvim_call_put_escstr(FILE *fd, const char *str, int what) { return put_escstr(fd, str, what); }
int nvim_call_put_eol(FILE *fd) { return put_eol(fd); }
const void *nvim_option_get_p_wc_ptr(void) { return &p_wc; }
const void *nvim_option_get_p_wcm_ptr(void) { return &p_wcm; }
// curwin fold option varp pointers for rs_makefoldset
void *nvim_curwin_p_fdm_varp(void) { return &curwin->w_p_fdm; }
void *nvim_curwin_p_fde_varp(void) { return &curwin->w_p_fde; }
void *nvim_curwin_p_fmr_varp(void) { return &curwin->w_p_fmr; }
void *nvim_curwin_p_fdi_varp(void) { return &curwin->w_p_fdi; }
void *nvim_curwin_p_fdl_varp(void) { return &curwin->w_p_fdl; }
void *nvim_curwin_p_fml_varp(void) { return &curwin->w_p_fml; }
void *nvim_curwin_p_fdn_varp(void) { return &curwin->w_p_fdn; }
void *nvim_curwin_p_fen_varp(void) { return &curwin->w_p_fen; }
int nvim_call_fprintf(FILE *fd, const char *str) { return fputs(str, fd); }
int nvim_call_fprintf_int(FILE *fd, const char *fmt, int64_t val)
{
  return fprintf(fd, fmt, val);
}
void nvim_put_set_get_opt_name_flags(OptIndex opt_idx, const char **name, uint64_t *flags)
{
  *name = options[opt_idx].fullname;
  *flags = options[opt_idx].flags;
}
void *nvim_option_get_var_ptr(OptIndex opt_idx) { return options[opt_idx].var; }
OptVal nvim_option_get_def_val(OptIndex opt_idx) { return options[opt_idx].def_val; }

// Phase 1 init function accessors: expose static helpers to Rust
void nvim_call_set_string_default(int opt_idx, char *val, bool allocated)
{
  rs_set_string_default_opt((OptIndex)opt_idx, val, allocated);
}
void nvim_call_change_option_default(int opt_idx, OptVal value)
{
  rs_change_option_default((OptIndex)opt_idx, value);
}
char *nvim_call_enc_locale(void) { return enc_locale(); }
void nvim_set_fenc_default(char *val) { fenc_default = val; }
void nvim_set_p_title(int v) { p_title = v; }
void nvim_set_p_icon(int v) { p_icon = v; }
char *nvim_call_os_getenv(const char *name) { return os_getenv(name); }
char *nvim_call_vim_getenv(const char *name) { return vim_getenv(name); }
int nvim_option_was_set_idx(int opt_idx) { return option_was_set((OptIndex)opt_idx); }
// For set_init_default_backupskip: expose after_pathsep
int nvim_call_after_pathsep(const char *b, const char *p) { return after_pathsep(b, p); }
// For rs_find_dup_item call in backupskip (flags accessor already exists)

// Accessors for rs_set_init_2 and rs_set_init_3 (option pass 7 phase 2)
void nvim_option_ilog_rtp(void) { ILOG("startup runtimepath/packpath value: %s", p_rtp); }
void nvim_call_set_option_default(int opt_idx, int opt_flags) { rs_set_option_default(opt_idx, opt_flags); }
void nvim_call_comp_col(void) { comp_col(); }
void nvim_call_parse_shape_opt(void) { parse_shape_opt(SHAPE_CURSOR); }
const char *nvim_call_invocation_path_tail(const char *p_sh, size_t *lenp) { return invocation_path_tail(p_sh, lenp); }
int nvim_call_path_fnamecmp(const char *a, const char *b) { return path_fnamecmp(a, b); }
int nvim_curbuf_is_empty(void) { return buf_is_empty(curbuf); }
void nvim_call_set_option_direct(int opt_idx, OptVal val, int opt_flags) { set_option_direct((OptIndex)opt_idx, val, opt_flags, SID_NONE); }

// Accessors for rs_ex_set and rs_validate_opt_idx (option pass 7 phase 3)
int nvim_get_cmd_idx_setlocal(void) { return (int)CMD_setlocal; }
int nvim_get_cmd_idx_setglobal(void) { return (int)CMD_setglobal; }

// Phase 2 makeset accessors
int nvim_call_put_line(FILE *fd, const char *str) { return put_line(fd, str); }
// Write "if &optname != 'val'\n" to fd; returns OK or FAIL (< 0 = FAIL like fprintf)
int nvim_call_makeset_if_line(FILE *fd, const char *optname, const char *val)
{
  if (fprintf(fd, "if &%s != '%s'", optname, val) < 0) {
    return FAIL;
  }
  return put_eol(fd) < 0 ? FAIL : OK;
}
// Dereference a void* as char** to get the string value (for string option varp)
const char *nvim_get_varp_string_val(const void *varp) { return *(char *const *)varp; }
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
int nvim_winopt_string_field_count(void) { return 23; }

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

// Wrap copy_option_val (static) for use from Rust.
char *nvim_call_copy_option_val(const char *val) { return copy_option_val(val); }

// Wrap clear_string_option / check_string_option for Rust.
void nvim_call_clear_string_option(char **ptr) { clear_string_option(ptr); }
void nvim_call_check_string_option(char **ptr) { check_string_option(ptr); }

// Return pointer to win_T.w_onebuf_opt / w_allbuf_opt for Rust.
winopt_T *nvim_win_get_onebuf_opt(win_T *win) { return &win->w_onebuf_opt; }
winopt_T *nvim_win_get_allbuf_opt(win_T *win) { return &win->w_allbuf_opt; }

// Wrappers for didset_window_options internals.
void nvim_call_check_colorcolumn(win_T *wp) { check_colorcolumn(NULL, wp); }
void nvim_call_briopt_check(win_T *wp) { briopt_check(NULL, wp); }
void nvim_call_fill_culopt_flags(win_T *wp) { fill_culopt_flags(NULL, wp); }
void nvim_call_set_chars_option_fcs(win_T *wp)
{
  set_chars_option(wp, wp->w_p_fcs, kFillchars, true, NULL, 0);
}
void nvim_call_set_chars_option_lcs(win_T *wp)
{
  set_chars_option(wp, wp->w_p_lcs, kListchars, true, NULL, 0);
}
void nvim_call_check_blending(win_T *wp) { check_blending(wp); }
void nvim_call_set_winbar_win(win_T *wp, int valid_cursor)
{
  set_winbar_win(wp, false, (bool)valid_cursor);
}
void nvim_call_check_signcolumn(win_T *wp) { check_signcolumn(NULL, wp); }
// Update w_grid_alloc.blending based on current w_p_winbl value (different from window_shim's nvim_win_set_grid_blending which takes explicit bool).
void nvim_win_update_grid_blending(win_T *wp) { wp->w_grid_alloc.blending = wp->w_p_winbl > 0; }


extern void rs_set_init_fenc_default(void);




extern const char *rs_option_expand(int opt_idx, const char *val);
extern void rs_set_options_default(int opt_flags);





/// 'title' and 'icon' only default to true if they have not been set or reset
/// in .vimrc and we can read the old value.
/// When 'title' and 'icon' have been reset in .vimrc, we won't even check if
/// they can be reset.  This reduces startup time when using X on a remote
/// machine.


/// Get the string value specified for a ":set" command.  The following set options are supported:
///     set {opt}={val}
///     set {opt}:{val}
static char *stropt_get_newval(int nextchar, OptIndex opt_idx, char **argp, void *varp,
                               const char *origval, set_op_T *op_arg, uint32_t flags)
{
  int op = (int)(*op_arg);
  char *result = rs_stropt_get_newval(nextchar, opt_idx, argp, varp, origval, &op, flags);
  *op_arg = (set_op_T)op;
  return result;
}

static int validate_opt_idx(win_T *win, OptIndex opt_idx, int opt_flags, uint32_t flags,
                            set_prefix_T prefix, const char **errmsg)
{
  ValidateOptIdxResult r = rs_validate_opt_idx(win, opt_idx, opt_flags, flags, (int)prefix);
  if (r.errmsg != NULL) {
    *errmsg = r.errmsg;
  }
  return r.result;
}

/// Skip over the name of a TTY option or keycode option.
///
/// Skip over the name of an option.
///
/// @param[in]   arg       Start of option name.
/// @param[out]  opt_idxp  Set to option index in options[] table.
///
/// @return NULL when no option name found. Otherwise pointer to the char after the option name.
const char *find_option_end(const char *arg, OptIndex *opt_idxp)
{
  FindOptionEndResult r = rs_find_option_end(arg);
  *opt_idxp = (OptIndex)r.opt_idx;
  return r.end;
}

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

/// Expand environment variables for some string options.
/// These string options cannot be indirect!
/// Returns expanded string (in static NameBuff), or NULL if no expansion.
char *nvim_option_expand(OptIndex opt_idx, const char *val)
{
  return (char *)rs_option_expand((int)opt_idx, val);
}

/// Get the address of p_kp (keywordprg global), as void*.
/// Used by Rust stropt_get_newval to detect keywordprg option.
void *nvim_option_get_p_kp_ptr(void)
{
  return (void *)&p_kp;
}

extern void rs_didset_options(void);
extern void rs_didset_options2(void);


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


/// Get option index for scope.
ssize_t option_scope_idx(OptIndex opt_idx, OptScope scope)
{
  return rs_option_scope_idx(opt_idx, (int)scope);
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


/// Unset the local value of a global-local option.
///
/// @param      opt_idx    Index in options[] table. Must not be kOptInvalid.
///
/// @return  NULL on success, an untranslated error message on error.
static inline const char *unset_option_local_value(const OptIndex opt_idx)
{
  return rs_unset_option_local_value(opt_idx);
}


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

static char *copy_option_val(const char *val)
{
  if (val == empty_string_option) {
    return empty_string_option;  // no need to allocate memory
  }
  return xstrdup(val);
}

extern void rs_check_winopt(winopt_T *wop);




static OptIndex expand_option_idx = kOptInvalid;
static int expand_option_start_col = 0;
static char expand_option_name[5] = { 't', '_', NUL, NUL, NUL };
static int expand_option_flags = 0;
static bool expand_option_append = false;

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

// expand_option static variable get/set accessors
OptIndex nvim_get_expand_option_idx(void) { return expand_option_idx; }
void nvim_set_expand_option_idx(OptIndex val) { expand_option_idx = val; }
void nvim_set_expand_option_start_col(int val) { expand_option_start_col = val; }
int nvim_get_expand_option_flags(void) { return expand_option_flags; }
void nvim_set_expand_option_flags(int val) { expand_option_flags = val; }
void nvim_set_expand_option_append(int val) { expand_option_append = (bool)val; }
void nvim_get_expand_option_name(char out[5]) { memcpy(out, expand_option_name, 5); }
const char *nvim_get_expand_option_name_ptr(void) { return expand_option_name; }
void nvim_set_expand_option_name_chars(char c2, char c3)
{
  expand_option_name[2] = c2;
  expand_option_name[3] = c3;
}

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


/// Escape an option value that can be used on the command-line with :set.
/// Caller needs to free the returned string, unless NULL is returned.
extern char *rs_escape_option_str_cmdline(const char *var);
char *nvim_escape_option_str_cmdline(char *var)
{
  return rs_escape_option_str_cmdline(var);
}

/// curbuf/curwin accessors for option expansion Rust code.
buf_T *nvim_opt_get_curbuf(void) { return curbuf; }
win_T *nvim_opt_get_curwin(void) { return curwin; }
void nvim_opt_set_curbuf(buf_T *buf) { curbuf = buf; }
void nvim_opt_set_curwin(win_T *win) { curwin = win; }


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

static void didset_options_sctx(int opt_flags, int *buf)
{
  for (int i = 0;; i++) {
    if (buf[i] == kOptInvalid) {
      break;
    }

    set_option_sctx(buf[i], opt_flags, current_sctx);
  }
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

// =============================================================================
// Wrapper function implementations for Rust setcmd module
// =============================================================================

void nvim_set_options_default(int opt_flags) { rs_set_options_default(opt_flags); }
void nvim_didset_options(void) { rs_didset_options(); }
void nvim_didset_options2(void) { rs_didset_options2(); }

int nvim_validate_opt_idx(win_T *win, OptIndex opt_idx, int opt_flags, uint32_t flags,
                          int prefix, const char **errmsg)
{
  return validate_opt_idx(win, opt_idx, opt_flags, flags, (set_prefix_T)prefix, errmsg);
}

/// Parse a number from arg using vim_str2nr (STR2NR_ALL format).
/// Sets *len_out to the number of characters consumed.
/// Sets *num_out to the parsed value.
/// Wraps vim_str2nr for Rust FFI.
void nvim_call_vim_str2nr(const char *arg, int *len_out, int64_t *num_out)
{
  vim_str2nr(arg, NULL, len_out, STR2NR_ALL, num_out, NULL, 0, true, NULL);
}

/// Get the e_number_required_after_equal error string.
const char *nvim_get_e_number_required_after_equal(void)
{
  return e_number_required_after_equal;
}

OptInt nvim_get_p_ss(void) { return p_ss; }

// =============================================================================
// Phase 6 accessors: do_syntax_autocmd, do_spelllang_source, get_fileformat_force
// =============================================================================

/// Get buf->b_p_syn (the 'syntax' option value for this buffer).
const char *nvim_buf_get_b_p_syn(buf_T *buf)
{
  return buf ? buf->b_p_syn : NULL;
}

/// Set BF_SYN_SET flag in buf->b_flags.
void nvim_buf_set_b_flags_syn_set(buf_T *buf)
{
  if (buf) {
    buf->b_flags |= BF_SYN_SET;
  }
}

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

/// Get the escape_chars static string used by escape_option_str_cmdline.
const char *nvim_get_escape_chars(void)
{
  return escape_chars;
}

/// Wrap vim_strsave_escaped.
char *nvim_vim_strsave_escaped(const char *s, const char *esc)
{
  return vim_strsave_escaped(s, esc);
}

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

/// Call fuzzy_match_str with opaque pointer.
int nvim_option_fuzzy_match_str(const char *str, const char *pat)
{
  return fuzzy_match_str((char *)str, pat);
}

/// Call fuzzymatches_to_strmatches with opaque pointer.
void nvim_option_fuzzymatches_to_strmatches(void *fuzmatch, char ***matches, int count)
{
  fuzzymatches_to_strmatches((fuzmatch_str_T *)fuzmatch, matches, count, false);
}

/// Call cmdline_fuzzy_complete.
int nvim_option_cmdline_fuzzy_complete(const char *fuzzystr)
{
  return cmdline_fuzzy_complete(fuzzystr);
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

/// Address of curbuf->b_p_syn (for varp pointer comparison)
void *nvim_curbuf_b_p_syn_addr(void) { return &curbuf->b_p_syn; }
/// Address of curbuf->b_p_ft (for varp pointer comparison)
void *nvim_curbuf_b_p_ft_addr(void) { return &curbuf->b_p_ft; }
/// Address of curwin->w_s->b_p_spl (for varp pointer comparison)
void *nvim_curwin_b_p_spl_addr(void) { return &curwin->w_s->b_p_spl; }
/// Address of p_mouse (for varp pointer comparison)
void *nvim_get_p_mouse_addr(void) { return &p_mouse; }
/// Address of p_flp (for varp pointer comparison)
void *nvim_get_p_flp_addr(void) { return &p_flp; }
/// Address of curbuf->b_p_flp (for varp pointer comparison)
void *nvim_curbuf_b_p_flp_addr(void) { return &curbuf->b_p_flp; }
/// Address of p_wbr (for varp pointer comparison)
void *nvim_get_p_wbr_addr(void) { return &p_wbr; }
/// Address of curwin->w_p_wbr (for varp pointer comparison)
void *nvim_curwin_p_wbr_addr(void) { return &curwin->w_p_wbr; }

// nvim_curwin_get_w_curswant and nvim_curwin_set_w_set_curswant are defined in indent_ffi.c
/// Get curwin->w_briopt_list
int nvim_curwin_get_w_briopt_list(void) { return curwin->w_briopt_list ? 1 : 0; }

/// Get `secure` global variable
// nvim_get_secure, nvim_set_secure are defined in ex_docmd.c
// nvim_get_sandbox is defined in undo.c

/// Call buf_init_chartab(curbuf, true)
void nvim_call_buf_init_chartab(void) { buf_init_chartab(curbuf, true); }
/// Call setmouse()
void nvim_call_setmouse(void) { setmouse(); }
/// Call set_winbar(true)
void nvim_call_set_winbar(void) { set_winbar(true); }
/// Call do_filetype_autocmd(curbuf, value_changed)
void nvim_do_filetype_autocmd(int value_changed) { do_filetype_autocmd(curbuf, value_changed != 0); }

/// Set options[opt_idx].flags |= kOptFlagWasSet
void nvim_option_set_was_set_flag(OptIndex opt_idx) { options[opt_idx].flags |= kOptFlagWasSet; }

/// Error message strings for did_set_option
const char *nvim_get_e_unsupportedoption(void) { return e_unsupportedoption; }
const char *nvim_get_e_secure(void) { return e_secure; }
const char *nvim_get_e_unknown_option2(void) { return e_unknown_option2; }

/// Call emsg(_(msg)) -- translates and shows error message
void nvim_call_emsg_translated(const char *msg) { emsg(_(msg)); }

/// Call check_illegal_path_names(*(char**)varp, flags)
/// Returns 1 if illegal path names detected, 0 otherwise.
int nvim_check_illegal_path_names(void *varp, uint32_t flags)
{
  return check_illegal_path_names(*(char **)varp, flags) ? 1 : 0;
}

/// Get options[opt_idx].flags (already exists as nvim_option_get_flags_ptr, but need value)
uint32_t nvim_option_get_flags_val(OptIndex opt_idx) { return options[opt_idx].flags; }

/// Get current_sctx
sctx_T nvim_get_current_sctx(void) { return current_sctx; }

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


/// Returns cmdmod.cmod_flags (used by Rust to check CMOD_NOSWAPFILE).
int nvim_cmdmod_get_cmod_flags(void)
{
  return cmdmod.cmod_flags;
}

/// Returns cmdmod.cmod_flags & CMOD_NOSWAPFILE (1 if :noswapfile is active, 0 otherwise).
int nvim_get_cmod_noswapfile(void)
{
  return (cmdmod.cmod_flags & CMOD_NOSWAPFILE) != 0;
}

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

/// free_buf_options(buf, free_flags)
void nvim_call_free_buf_options(buf_T *buf, int free_flags) { free_buf_options(buf, free_flags != 0); }

/// check_buf_options(buf)
void nvim_call_check_buf_options(buf_T *buf) { check_buf_options(buf); }

/// buf_init_chartab(buf, false)
void nvim_call_buf_init_chartab_buf(buf_T *buf) { buf_init_chartab(buf, false); }

/// compile_cap_prog(&buf->b_s)
void nvim_call_compile_cap_prog_buf(buf_T *buf) { compile_cap_prog(&buf->b_s); }

/// tabstop_set(str, &buf->b_p_vsts_array)
void nvim_call_tabstop_set_vsts(buf_T *buf, const char *str) { tabstop_set(str, &buf->b_p_vsts_array); }
/// tabstop_set(str, &buf->b_p_vts_array)
void nvim_call_tabstop_set_vts(buf_T *buf, const char *str) { tabstop_set(str, &buf->b_p_vts_array); }

/// Returns buf->b_p_vts_array (for null check)
int nvim_buf_get_b_p_vts_array_is_null(buf_T *buf) { return buf->b_p_vts_array == NULL ? 1 : 0; }

/// set_buflocal_cpt_callbacks(buf)
void nvim_call_set_buflocal_cpt_callbacks(buf_T *buf) { set_buflocal_cpt_callbacks(buf); }
/// set_buflocal_cfu_callback(buf)
void nvim_call_set_buflocal_cfu_callback(buf_T *buf) { set_buflocal_cfu_callback(buf); }
/// set_buflocal_ofu_callback(buf)
void nvim_call_set_buflocal_ofu_callback(buf_T *buf) { set_buflocal_ofu_callback(buf); }

/// buf->b_kmap_state |= KEYMAP_INIT
void nvim_buf_kmap_state_set_init(buf_T *buf) { buf->b_kmap_state |= KEYMAP_INIT; }

/// Returns p_vsts (the global vartabstop value).
const char *nvim_get_p_vsts(void) { return p_vsts; }
/// Returns p_vts (the global vartabstop for normal tabstop).
const char *nvim_get_p_vts(void) { return p_vts; }
/// Returns p_vsts_nopaste.
const char *nvim_get_p_vsts_nopaste(void) { return p_vsts_nopaste; }

/// Returns p_fenc.
const char *nvim_get_p_fenc(void) { return p_fenc; }
/// Returns p_ff.
const char *nvim_get_p_ff(void) { return p_ff; }
/// Returns p_ffs (fileformats).
const char *nvim_get_p_ffs(void) { return p_ffs; }

// These individual global option getters are needed for the bulk copy:
const char *nvim_get_p_cpt(void) { return p_cpt; }
const char *nvim_get_p_cfu(void) { return p_cfu; }
const char *nvim_get_p_ofu(void) { return p_ofu; }
const char *nvim_get_p_tfu(void) { return p_tfu; }
const char *nvim_get_p_com(void) { return p_com; }
const char *nvim_get_p_cms(void) { return p_cms; }
const char *nvim_get_p_fo(void) { return p_fo; }
const char *nvim_get_p_flp(void) { return p_flp; }
const char *nvim_get_p_nf(void) { return p_nf; }
const char *nvim_get_p_mps(void) { return p_mps; }
const char *nvim_get_p_cink(void) { return p_cink; }
const char *nvim_get_p_cino(void) { return p_cino; }
const char *nvim_get_p_cinsd(void) { return p_cinsd; }
const char *nvim_get_p_lop(void) { return p_lop; }
const char *nvim_get_p_cinw(void) { return p_cinw; }
const char *nvim_get_p_inde(void) { return p_inde; }
const char *nvim_get_p_indk(void) { return p_indk; }
const char *nvim_get_p_fex(void) { return p_fex; }
const char *nvim_get_p_sua(void) { return p_sua; }
const char *nvim_get_p_keymap(void) { return p_keymap; }
const char *nvim_get_p_qe(void) { return p_qe; }
const char *nvim_get_p_inex(void) { return p_inex; }
const char *nvim_get_p_spc(void) { return p_spc; }
const char *nvim_get_p_spf(void) { return p_spf; }
const char *nvim_get_p_spl(void) { return p_spl; }
const char *nvim_get_p_spo(void) { return p_spo; }
const char *nvim_get_p_isk(void) { return p_isk; }
#ifdef BACKSLASH_IN_FILENAME
const char *nvim_get_p_csl(void) { return p_csl; }
#else
const char *nvim_get_p_csl(void) { return ""; }
#endif
int nvim_get_backslash_in_filename(void) {
#ifdef BACKSLASH_IN_FILENAME
  return 1;
#else
  return 0;
#endif
}

int nvim_get_p_ai(void) { return p_ai; }
bool nvim_get_p_bin(void) { return p_bin; }
bool nvim_get_p_bomb(void) { return p_bomb; }
bool nvim_get_p_ci(void) { return p_ci; }
bool nvim_get_p_cin(void) { return p_cin; }
int nvim_get_p_et(void) { return p_et; }
bool nvim_get_p_fixeol(void) { return p_fixeol; }
bool nvim_get_p_lisp(void) { return p_lisp; }
int nvim_get_p_ma(void) { return p_ma; }
int nvim_get_p_ml(void) { return p_ml; }
bool nvim_get_p_pi(void) { return p_pi; }
bool nvim_get_p_si(void) { return p_si; }
bool nvim_get_p_swf(void) { return p_swf; }
// nvim_get_p_udf: already defined as int at line 579

OptInt nvim_get_p_sw(void) { return p_sw; }
OptInt nvim_get_p_scbk(void) { return p_scbk; }
// nvim_get_p_tw / nvim_get_p_wm: already defined at lines 793/795
OptInt nvim_get_p_sts(void) { return p_sts; }
OptInt nvim_get_p_ts(void) { return p_ts; }
OptInt nvim_get_p_smc(void) { return p_smc; }

bool nvim_get_p_ai_nopaste(void) { return p_ai_nopaste; }
bool nvim_get_p_et_nopaste(void) { return p_et_nopaste; }
OptInt nvim_get_p_tw_nopaste(void) { return p_tw_nopaste; }
OptInt nvim_get_p_wm_nopaste(void) { return p_wm_nopaste; }
OptInt nvim_get_p_sts_nopaste(void) { return p_sts_nopaste; }

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

/// Sets buf->b_p_fenc = xstrdup(p_fenc).
void nvim_buf_set_b_p_fenc_dup(buf_T *buf) { buf->b_p_fenc = xstrdup(p_fenc); }

/// Sets buf->b_p_ff based on first char of p_ffs.
void nvim_buf_set_b_p_ff_from_ffs(buf_T *buf) {
  switch (*p_ffs) {
  case 'm': buf->b_p_ff = xstrdup("mac"); break;
  case 'd': buf->b_p_ff = xstrdup("dos"); break;
  case 'u': buf->b_p_ff = xstrdup("unix"); break;
  default:  buf->b_p_ff = xstrdup(p_ff); break;
  }
}

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


/// langmap_init() wrapper.
void nvim_call_langmap_init(void) { langmap_init(); }

/// save_file_ff(curbuf) wrapper.
void nvim_call_save_file_ff_curbuf(void) { save_file_ff(curbuf); }

/// os_env_exists(name, false) wrapper. Returns 1 if env exists, 0 otherwise.
int nvim_call_os_env_exists(const char *name) { return os_env_exists(name, false) ? 1 : 0; }

/// init_spell_chartab() wrapper.
void nvim_call_init_spell_chartab(void) { init_spell_chartab(); }

/// lang_init() wrapper.
void nvim_call_lang_init(void) { lang_init(); }

/// set_option_value_give_err(kOptTermbidi, BOOLEAN_OPTVAL(true), 0) wrapper.
void nvim_call_set_termbidi_true(void)
{
  set_option_value_give_err(kOptTermbidi, BOOLEAN_OPTVAL(true), 0);
}

/// alloc_options_default() wrapper.
void nvim_call_alloc_options_default(void) { rs_alloc_options_default(); }

/// check_win_options(curwin) wrapper.
void nvim_call_check_win_options(void)
{
  rs_check_winopt(&curwin->w_onebuf_opt);
  rs_check_winopt(&curwin->w_allbuf_opt);
}

/// set_helplang_default(get_mess_lang()) wrapper.
void nvim_call_set_helplang_default_from_mess_lang(void)
{
  set_helplang_default(get_mess_lang());
}

/// set_init_fenc_default() wrapper.
void nvim_call_set_init_fenc_default(void) { rs_set_init_fenc_default(); }

/// rs_last_status(0) -- already a Rust fn; call it here for the shim.
void nvim_call_rs_last_status_0(void) { rs_last_status(0); }

/// curbuf->b_p_initialized = true
void nvim_curbuf_set_b_p_initialized(void) { curbuf->b_p_initialized = true; }

/// curbuf->b_p_ac = -1
void nvim_curbuf_set_b_p_ac_minus1(void) { curbuf->b_p_ac = -1; }

/// curbuf->b_p_ar = -1
void nvim_curbuf_set_b_p_ar_minus1(void) { curbuf->b_p_ar = -1; }

/// curbuf->b_p_ul = NO_LOCAL_UNDOLEVEL
void nvim_curbuf_set_b_p_ul_no_local(void) { curbuf->b_p_ul = NO_LOCAL_UNDOLEVEL; }

/// check_buf_options(curbuf)
void nvim_call_check_buf_options_curbuf(void) { check_buf_options(curbuf); }

/// stdpaths_user_state_subpath(name, 2, true), returns allocated string.
char *nvim_call_stdpaths_user_state_subpath(const char *name) { return stdpaths_user_state_subpath(name, 2, true); }

/// runtimepath_default(clean_arg) wrapper.
char *nvim_call_runtimepath_default(int clean_arg) { return runtimepath_default(clean_arg != 0); }

/// set_string_default(opt_idx, val, allocated) with integer opt_idx and bool allocated.
void nvim_call_set_string_default_idx(int opt_idx, char *val, int allocated)
{
  rs_set_string_default_opt((OptIndex)opt_idx, val, allocated != 0);
}

/// set_init_expand_env() implementation called from Rust.
/// Iterates over all options and expands environment variables for defaults.
void nvim_call_set_init_expand_env(void)
{
  for (OptIndex opt_idx = 0; opt_idx < kOptCount; opt_idx++) {
    vimoption_T *opt = &options[opt_idx];
    if (opt->flags & kOptFlagNoDefExp) {
      continue;
    }
    char *p;
    if ((opt->flags & kOptFlagGettext) && opt->var != NULL) {
      p = _(*(char **)opt->var);
    } else {
      p = rs_option_expand((int)opt_idx, NULL);
    }
    if (p != NULL) {
      rs_set_option_varp(opt_idx, opt->var, CSTR_TO_OPTVAL(p), 1);
      rs_change_option_default(opt_idx, CSTR_TO_OPTVAL(p));
    }
  }
}

// Compile-time boundary validation for kBufOpt enum (Rust K_BUF_OPT_* constants).
// Checks first value, last value, and count; specific index alignment is validated at
// runtime by the offset table in varp.rs via buf_field_offsets().
_Static_assert((int)kBufOptAutocomplete == 0, "K_BUF_OPT_AUTOCOMPLETE mismatch");
_Static_assert((int)kBufOptWrapmargin == 90, "K_BUF_OPT_WRAPMARGIN mismatch");
_Static_assert(kBufOptCount == 91, "K_BUF_OPT_COUNT mismatch");

/// vim_strchr wrapper for Rust.
const char *nvim_call_vim_strchr(const char *s, int c) { return vim_strchr(s, c); }

/// Calls bind_textdomain_codeset(PROJECT_NAME, p_enc) if HAVE_WORKING_LIBINTL.
/// No-op otherwise.
void nvim_call_bind_textdomain_codeset(void)
{
#ifdef HAVE_WORKING_LIBINTL
  (void)bind_textdomain_codeset(PROJECT_NAME, p_enc);
#endif
}

/// If buf->b_p_bt[0] == 'h', clear it with clear_string_option.
void nvim_buf_clear_b_p_bt_if_help(buf_T *buf)
{
  if (buf && buf->b_p_bt && buf->b_p_bt[0] == 'h') {
    clear_string_option(&buf->b_p_bt);
  }
}

/// Check if CPO_UNDO is in p_cpo.
int nvim_option_p_cpo_has_undo(void)
{
  return vim_strchr(p_cpo, CPO_UNDO) != NULL ? 1 : 0;
}
/// Get p_rdt (redrawtime option, milliseconds).
int64_t nvim_option_get_p_rdt(void) { return (int64_t)p_rdt; }
/// Get p_cwh (cmdwinheight).
int nvim_option_get_p_cwh(void) { return (int)p_cwh; }
/// Get whether p_icm (inccommand) is non-empty.
int nvim_option_p_icm_notnul(void) { return *p_icm != NUL ? 1 : 0; }

// =============================================================================
// Phase 12 Pass 1: Paste helper compound accessors
// These replace the 6 nvim_paste_* functions called from Rust complex.rs.
// Each accessor handles a specific sub-operation that requires C pointer
// comparisons against empty_string_option or free_string_option.
// =============================================================================

/// Save buf paste scalar nopaste fields: copies tw/wm/sts/ai/et -> nopaste.
void nvim_buf_paste_save_scalars(buf_T *buf)
{
  buf->b_p_tw_nopaste = buf->b_p_tw;
  buf->b_p_wm_nopaste = buf->b_p_wm;
  buf->b_p_sts_nopaste = buf->b_p_sts;
  buf->b_p_ai_nopaste = buf->b_p_ai;
  buf->b_p_et_nopaste = buf->b_p_et;
}

/// Save buf->b_p_vsts into buf->b_p_vsts_nopaste (frees old nopaste first).
void nvim_buf_paste_save_vsts(buf_T *buf)
{
  if (buf->b_p_vsts_nopaste) {
    xfree(buf->b_p_vsts_nopaste);
  }
  buf->b_p_vsts_nopaste = buf->b_p_vsts && buf->b_p_vsts != empty_string_option
                          ? xstrdup(buf->b_p_vsts) : NULL;
}

/// Activate paste for buf scalars: zero tw/wm/sts/ai/et.
void nvim_buf_paste_activate_scalars(buf_T *buf)
{
  buf->b_p_tw = 0;
  buf->b_p_wm = 0;
  buf->b_p_sts = 0;
  buf->b_p_ai = 0;
  buf->b_p_et = 0;
}

/// Activate paste for buf->b_p_vsts: free and set to empty_string_option, clear array.
void nvim_buf_paste_activate_vsts(buf_T *buf)
{
  if (buf->b_p_vsts) {
    free_string_option(buf->b_p_vsts);
  }
  buf->b_p_vsts = empty_string_option;
  XFREE_CLEAR(buf->b_p_vsts_array);
}

/// Restore buf paste scalar fields: copies nopaste -> tw/wm/sts/ai/et.
void nvim_buf_paste_restore_scalars(buf_T *buf)
{
  buf->b_p_tw = buf->b_p_tw_nopaste;
  buf->b_p_wm = buf->b_p_wm_nopaste;
  buf->b_p_sts = buf->b_p_sts_nopaste;
  buf->b_p_ai = buf->b_p_ai_nopaste;
  buf->b_p_et = buf->b_p_et_nopaste;
}

/// Restore buf->b_p_vsts from nopaste: free current, dup from nopaste, run tabstop_set.
void nvim_buf_paste_restore_vsts(buf_T *buf)
{
  if (buf->b_p_vsts) {
    free_string_option(buf->b_p_vsts);
  }
  buf->b_p_vsts = buf->b_p_vsts_nopaste ? xstrdup(buf->b_p_vsts_nopaste) : empty_string_option;
  xfree(buf->b_p_vsts_array);
  if (buf->b_p_vsts && buf->b_p_vsts != empty_string_option) {
    tabstop_set(buf->b_p_vsts, &buf->b_p_vsts_array);
  } else {
    buf->b_p_vsts_array = NULL;
  }
}

/// Save global paste scalars: copies ai/et/sts/tw/wm to nopaste statics.
void nvim_paste_global_save_scalars(void)
{
  p_ai_nopaste = p_ai;
  p_et_nopaste = p_et;
  p_sts_nopaste = p_sts;
  p_tw_nopaste = p_tw;
  p_wm_nopaste = p_wm;
}

/// Save p_vsts to p_vsts_nopaste (frees old nopaste first).
void nvim_paste_global_save_vsts(void)
{
  if (p_vsts_nopaste) {
    xfree(p_vsts_nopaste);
  }
  p_vsts_nopaste = p_vsts && p_vsts != empty_string_option ? xstrdup(p_vsts) : NULL;
}

/// Activate global paste scalars: zero sm/sta/ru(+redraw)/ri/tw/wm/sts/ai/et.
void nvim_paste_global_activate_scalars(void)
{
  p_sm = 0;
  p_sta = 0;
  if (p_ru) {
    status_redraw_all();
  }
  p_ru = 0;
  p_ri = 0;
  p_tw = 0;
  p_wm = 0;
  p_sts = 0;
  p_ai = 0;
  p_et = 0;
}

/// Activate global p_vsts: free and set to empty_string_option.
void nvim_paste_global_activate_vsts(void)
{
  if (p_vsts) {
    free_string_option(p_vsts);
  }
  p_vsts = empty_string_option;
}

/// Restore global paste scalars: restores sm/sta/ru(+redraw)/ri from args,
/// and ai/et/sts/tw/wm from nopaste statics.
void nvim_paste_global_restore_scalars(int save_sm, int save_sta, int save_ru, int save_ri)
{
  p_sm = save_sm;
  p_sta = save_sta;
  if (p_ru != save_ru) {
    status_redraw_all();
  }
  p_ru = save_ru;
  p_ri = save_ri;
  p_ai = p_ai_nopaste;
  p_et = p_et_nopaste;
  p_sts = p_sts_nopaste;
  p_tw = p_tw_nopaste;
  p_wm = p_wm_nopaste;
}

/// Restore p_vsts from p_vsts_nopaste.
void nvim_paste_global_restore_vsts(void)
{
  if (p_vsts) {
    free_string_option(p_vsts);
  }
  p_vsts = p_vsts_nopaste ? xstrdup(p_vsts_nopaste) : empty_string_option;
}

/// Record sctx for all paste-dependent options.
void nvim_paste_didset_sctx_all(void)
{
  static int paste_dep[] = {
    kOptAutoindent, kOptExpandtab, kOptRuler, kOptShowmatch, kOptSmarttab, kOptSofttabstop,
    kOptTextwidth, kOptWrapmargin, kOptRevins, kOptVarsofttabstop, kOptInvalid
  };
  didset_options_sctx((OPT_LOCAL | OPT_GLOBAL), paste_dep);
}

// =============================================================================
// Phase 12 Pass 2: didset_options / didset_options2 sub-function wrappers
// =============================================================================

/// didset_string_options() wrapper.
void nvim_call_didset_string_options(void) { didset_string_options(); }
/// spell_check_msm() wrapper.
void nvim_call_spell_check_msm(void) { spell_check_msm(); }
/// spell_check_sps() wrapper.
void nvim_call_spell_check_sps(void) { spell_check_sps(); }
/// compile_cap_prog(curwin->w_s) wrapper.
void nvim_call_compile_cap_prog_curwin(void) { compile_cap_prog(curwin->w_s); }
/// did_set_spell_option() wrapper.
void nvim_call_did_set_spell_option(void) { did_set_spell_option(); }
/// did_set_cedit(NULL) wrapper.
void nvim_call_did_set_cedit(void) { did_set_cedit(NULL); }
/// did_set_breakat(NULL) wrapper.
void nvim_call_did_set_breakat(void) { did_set_breakat(NULL); }
/// didset_window_options(curwin, true) wrapper.
void nvim_call_didset_window_options_curwin(void) { didset_window_options(curwin, true); }
/// set_chars_option(curwin, curwin->w_p_fcs, kFillchars, true, NULL, 0)
void nvim_call_set_chars_option_fcs_curwin(void)
{
  set_chars_option(curwin, curwin->w_p_fcs, kFillchars, true, NULL, 0);
}
/// set_chars_option(curwin, curwin->w_p_lcs, kListchars, true, NULL, 0)
void nvim_call_set_chars_option_lcs_curwin(void)
{
  set_chars_option(curwin, curwin->w_p_lcs, kListchars, true, NULL, 0);
}
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
/// Record sctx for all bin-dependent options.
void nvim_bin_didset_sctx_all(int opt_flags)
{
  static int bin_dep[] = {
    kOptTextwidth, kOptWrapmargin, kOptModeline, kOptExpandtab, kOptInvalid
  };
  didset_options_sctx(opt_flags, bin_dep);
}

// =============================================================================
// Phase 12 lifecycle accessors (set_option_default, set_options_default,
// option_expand, free_all_options)
// =============================================================================

/// set_option_direct(opt_idx, val, opt_flags, current_sctx.sc_sid) wrapper.
void nvim_call_set_option_direct_with_sctx(int opt_idx, OptVal val, int opt_flags)
{
  set_option_direct((OptIndex)opt_idx, val, opt_flags, current_sctx.sc_sid);
}

/// win_comp_scroll(curwin) wrapper.
void nvim_call_win_comp_scroll_curwin(void) { win_comp_scroll(curwin); }

/// FOR_ALL_TAB_WINDOWS win_comp_scroll wrapper.
void nvim_call_comp_scroll_all_windows(void)
{
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    win_comp_scroll(wp);
  }
}

/// parse_cino(curbuf) wrapper.
void nvim_call_parse_cino_curbuf(void) { parse_cino(curbuf); }

/// free_operatorfunc_option() wrapper (EXITFREE only).
#if defined(EXITFREE)
void nvim_call_free_operatorfunc_option(void) { free_operatorfunc_option(); }
#else
void nvim_call_free_operatorfunc_option(void) {}
#endif

/// free_findfunc_option() wrapper.
void nvim_call_free_findfunc_option(void) { free_findfunc_option(); }

/// XFREE_CLEAR(fenc_default) wrapper.
void nvim_call_xfree_clear_fenc_default(void) { XFREE_CLEAR(fenc_default); }

/// XFREE_CLEAR(p_term) wrapper.
void nvim_call_xfree_clear_p_term(void) { XFREE_CLEAR(p_term); }

/// XFREE_CLEAR(p_ttytype) wrapper.
void nvim_call_xfree_clear_p_ttytype(void) { XFREE_CLEAR(p_ttytype); }

/// Return escape kind for option_expand:
///   0 = no escape needed
///   1 = escape spaces (p_tags or p_path)
///   2 = use "file:" prefix (p_sps)
int nvim_option_expand_escape_kind(int opt_idx)
{
  char **var = (char **)options[(OptIndex)opt_idx].var;
  if (var == &p_tags || var == &p_path) {
    return 1;
  }
  if (var == &p_sps) {
    return 2;
  }
  return 0;
}

/// expand_env_esc into NameBuff wrapper for option_expand.
/// esc_kind: 0=no escape, 1=escape, 2=use "file:" prefix.
/// Returns NameBuff if the expanded string differs from val, else NULL.
const char *nvim_call_expand_env_esc_option(const char *val, int esc_kind)
{
  const char *prefix = (esc_kind == 2) ? "file:" : NULL;
  bool esc = (esc_kind == 1);
  expand_env_esc(val, NameBuff, MAXPATHL, esc, false, (char *)prefix);
  if (strcmp(NameBuff, val) == 0) {
    return NULL;
  }
  return NameBuff;
}
