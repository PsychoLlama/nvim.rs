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
extern const char *rs_find_tty_option_end(const char *arg);
extern int rs_is_tty_option(const char *name);
extern const char *rs_skip_to_option_part(const char *p);
extern int rs_default_fileformat(void);
extern size_t rs_copy_option_part(char **pp, char *buf, size_t maxlen, const char *sep);
extern const char *rs_validate_num_option(int opt_idx, OptInt *newval, char *errbuf, size_t errbuflen);
extern const char *rs_find_dup_item(const char *origval, const char *newval, size_t newvallen, uint32_t flags);
extern char *rs_stropt_get_newval(int nextchar, int opt_idx, char **argp, void *varp,
                                  const char *origval, int *op_arg, uint32_t flags);
extern int rs_fill_culopt_flags(const char *val, win_T *wp);
extern void rs_set_options_bin(int oldval, int newval, int opt_flags);
extern void rs_set_fileformat(int eol_style, int opt_flags);
extern void rs_set_helplang_default(const char *lang);

// Rust parsing helpers and query functions (option pass 7 phase 1)
extern int rs_get_op(const char *arg);
extern int rs_get_option_prefix(char **argp);
extern int rs_shortmess(int x);
typedef struct { const char *end; int opt_idx; } FindOptionEndResult;
extern FindOptionEndResult rs_find_option_end(const char *arg);

// Rust init functions (option pass 7 phase 2)
extern void rs_set_init_2(int headless);
extern void rs_set_init_3(void);

// Rust query functions (from Rust query.rs, option pass 4 phase 1)
extern int rs_can_bs(int what);
extern const char *rs_get_equalprg(void);
extern const char *rs_get_findfunc(void);
extern unsigned rs_get_bkc_flags(buf_T *buf);
extern char *rs_get_flp_value(buf_T *buf);
extern unsigned rs_get_ve_flags(win_T *wp);
extern void rs_redraw_titles(void);
extern void rs_vimrc_found(const char *fname, const char *envname);
extern void rs_set_iminsert_global(buf_T *buf);
extern void rs_set_imsearch_global(buf_T *buf);
extern void rs_reset_modifiable(void);
extern OptVal rs_get_tty_option(const char *name);
extern bool rs_set_tty_option(const char *name, char *value);
extern int rs_string_to_key(char *arg);
extern void rs_check_redraw_for(buf_T *buf, win_T *win, uint32_t flags);
extern uint32_t *rs_insecure_flag(win_T *wp, OptIndex opt_idx, int opt_flags);
extern int rs_was_set_insecurely(win_T *wp, OptIndex opt_idx, int opt_flags);
extern void rs_set_option_sctx(OptIndex opt_idx, int opt_flags, sctx_T script_ctx);
extern int rs_optval_default(OptIndex opt_idx, void *varp);
extern int rs_wc_use_keyname(const void *varp, OptInt *wcp);
extern void rs_option_value2string(OptIndex opt_idx, int opt_flags);
extern int rs_put_set(FILE *fd, char *cmd, OptIndex opt_idx, void *varp);
extern int rs_makefoldset(FILE *fd);

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
extern bool rs_parse_winhl_opt(const char *winhl, win_T *wp);

// Phase 2: Medium-complexity string callbacks (from Rust string_simple.rs)
extern const char *rs_did_set_backupcopy(optset_T *args);
extern const char *rs_did_set_commentstring(optset_T *args);
extern const char *rs_did_set_comments(optset_T *args);
extern const char *rs_did_set_matchpairs(optset_T *args);
extern const char *rs_did_set_sessionoptions(optset_T *args);
extern const char *rs_did_set_spelloptions(optset_T *args);
extern const char *rs_did_set_diffanchors(optset_T *args);

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

// Rust varp dispatch functions (from Rust varp.rs)
extern void *rs_get_varp_from(vimoption_T *p, buf_T *buf, win_T *win);
extern void *rs_get_varp_scope_from(vimoption_T *p, int opt_flags, buf_T *buf, win_T *win);

// Rust set_context_in_set_cmd (from Rust setcmd.rs)
extern void rs_set_context_in_set_cmd(expand_T *xp, char *arg, int opt_flags);

// OptVal storage operations (from Rust storage.rs)
extern void rs_optval_free(OptVal o);
extern OptVal rs_optval_copy(OptVal o);
extern int rs_optval_equal(OptVal o1, OptVal o2);

// Rust FFI declarations (tag module)
extern void rs_free_tagfunc_option(void);
extern void rs_set_buflocal_tfu_callback(void *buf);

// Rust FFI declarations (window/layout module)
extern int rs_global_stl_height(void);
extern void rs_last_status(int morewin);
extern int rs_min_rows(tabpage_T *tp);
extern int rs_min_rows_for_all_tabpages(void);
extern int rs_tabline_height(void);
extern int rs_win_comp_pos(void);
extern int64_t rs_win_default_scroll(win_T *wp);
extern tabpage_T *rs_win_find_tabpage(win_T *win);
extern void rs_win_setheight(int height);
extern void rs_win_setwidth(int width);

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
const char *nvim_option_get_ffs(void) { return p_ffs; }
const char *nvim_option_get_cpo(void) { return p_cpo; }
const char *nvim_option_get_isk(void) { return p_isk; }
const char *nvim_option_get_isf(void) { return p_isf; }
const char *nvim_option_get_isp(void) { return p_isp; }
const char *nvim_option_get_isi(void) { return p_isi; }
const char *nvim_option_get_breakat(void) { return p_breakat; }
const char *nvim_option_get_sel(void) { return p_sel; }
const char *nvim_option_get_enc(void) { return p_enc; }
const char *nvim_option_get_ff(void) { return p_ff; }
const char *nvim_option_get_fo(void) { return p_fo; }
const char *nvim_option_get_mps(void) { return p_mps; }
const char *nvim_option_get_nf(void) { return p_nf; }
const char *nvim_option_get_ww(void) { return p_ww; }
const char *nvim_option_get_mouse(void) { return p_mouse; }
const char *nvim_option_get_shm(void) { return p_shm; }

// Boolean option accessors
int nvim_option_get_ai(void) { return p_ai; }
int nvim_option_get_et(void) { return p_et; }
int nvim_option_get_ic(void) { return p_ic; }
int nvim_option_get_scs(void) { return p_scs; }
int nvim_option_get_hls(void) { return p_hls; }
int nvim_option_get_is(void) { return p_is; }
int nvim_option_get_magic(void) { return p_magic; }
int nvim_option_get_fic(void) { return p_fic; }
int nvim_option_get_ml(void) { return p_ml; }
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
OptInt nvim_option_get_sw(void) { return p_sw; }
OptInt nvim_option_get_ts(void) { return p_ts; }
OptInt nvim_option_get_sts(void) { return p_sts; }
OptInt nvim_option_get_tw(void) { return p_tw; }
OptInt nvim_option_get_wm(void) { return p_wm; }
OptInt nvim_option_get_so(void) { return p_so; }
OptInt nvim_option_get_siso(void) { return p_siso; }
OptInt nvim_option_get_columns(void) { return p_columns; }
OptInt nvim_option_get_lines(void) { return p_lines; }
OptInt nvim_option_get_ch(void) { return p_ch; }
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
// nvim_option_get_hls (line ~302) is the canonical accessor; nvim_callback_get_p_hls removed
OptInt nvim_callback_get_p_titlelen(void) { return p_titlelen; }
int nvim_callback_get_no_hlsearch(void) { return no_hlsearch; }

// State setters for callbacks
void nvim_callback_set_need_maketitle(int value) { need_maketitle = value != 0; }

// Window diff accessor
int nvim_win_get_diff(win_T *win) { return win ? win->w_p_diff : 0; }

// Known option index accessors
OptIndex nvim_get_opt_idx_foldmethod(void) { return kOptFoldmethod; }
OptIndex nvim_get_opt_idx_wrap(void) { return kOptWrap; }

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
// nvim_option_get_ea (line ~312) is the canonical accessor; nvim_callback_get_p_ea removed
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
// nvim_win_get_ns_hl is already defined in window_shim.c
void nvim_win_set_ns_hl(win_T *win, int val) { win->w_ns_hl = val; }
// nvim_win_set_hl_needs_update is already defined in window_shim.c (takes bool)
// Return win->w_p_winhl (current winhighlight string)
const char *nvim_win_get_p_winhl(win_T *win) { return win ? win->w_p_winhl : NULL; }
// Return address of win->w_p_winhl for varp comparison
const void *nvim_win_get_p_winhl_addr(win_T *win) { return win ? (const void *)&win->w_p_winhl : NULL; }
// nvim_get_empty_string_option is already defined in window_shim.c (returns char*)
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
//   nvim_get_p_ru defined in drawscreen.c, nvim_get_p_ri defined in edit.c)
// nvim_option_get_sm / nvim_option_set_sm are the canonical accessors; nvim_get_p_sm/nvim_set_p_sm removed
int nvim_get_p_sta(void) { return p_sta; }
void nvim_set_p_sta(int val) { p_sta = val != 0; }
void nvim_set_p_ru(int val) { p_ru = val != 0; }
void nvim_set_p_ri(int val) { p_ri = val != 0; }

// fill_culopt_flags accessors
const char *nvim_win_get_p_culopt(win_T *wp) { return wp ? wp->w_p_culopt : NULL; }
void nvim_win_set_p_culopt_flags(win_T *wp, uint8_t flags) { if (wp) wp->w_p_culopt_flags = flags; }

// set_fileformat accessors: nvim_set_redraw_tabline/nvim_set_need_maketitle are defined in drawscreen.c

// set_options_bin global option accessors
OptInt nvim_get_p_tw(void) { return p_tw; }
void nvim_set_p_tw(OptInt v) { p_tw = v; }
OptInt nvim_get_p_wm(void) { return p_wm; }
void nvim_set_p_wm(OptInt v) { p_wm = v; }
// nvim_option_get_ml / nvim_option_set_ml and nvim_option_get_et / nvim_option_set_et
// are the canonical accessors; nvim_get_p_ml/nvim_set_p_ml/nvim_get_p_et/nvim_set_p_et removed
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

// Rust fold FFI declaration
extern void rs_foldUpdateAll(win_T *win);

// options[] is initialized in options.generated.h.
// The options with a NULL variable are 'hidden': a set command for them is
// ignored and they are not printed.

#include "options.generated.h"
#include "options_map.generated.h"

// After options[] is available:
int nvim_option_hlg_was_set(void) { return (options[kOptHelplang].flags & kOptFlagWasSet) != 0; }

// Xhistory callback wrappers (after includes for qf_resize_stack/ll_resize_stack)
void nvim_qf_resize_stack(int n) { qf_resize_stack(n); }
void nvim_ll_resize_stack(win_T *win, int n) { ll_resize_stack(win, n); }

// lines_or_columns callback: restore option varp to its old number value
void nvim_optset_restore_oldval_number(const void *args)
{
  const optset_T *a = (const optset_T *)args;
  OptVal oldval = (OptVal){ .type = kOptValTypeNumber, .data = a->os_oldval };
  set_option_varp(a->os_idx, a->os_varp, oldval, false);
}

// Paste callback consolidated buffer helpers (save-and-set / restore)
// Called via nvim_for_all_buffers from Rust.
void nvim_paste_buf_save_and_activate(buf_T *buf)
{
  buf->b_p_tw_nopaste = buf->b_p_tw;
  buf->b_p_wm_nopaste = buf->b_p_wm;
  buf->b_p_sts_nopaste = buf->b_p_sts;
  buf->b_p_ai_nopaste = buf->b_p_ai;
  buf->b_p_et_nopaste = buf->b_p_et;
  if (buf->b_p_vsts_nopaste) {
    xfree(buf->b_p_vsts_nopaste);
  }
  buf->b_p_vsts_nopaste = buf->b_p_vsts && buf->b_p_vsts != empty_string_option
                          ? xstrdup(buf->b_p_vsts) : NULL;
  buf->b_p_tw = 0;
  buf->b_p_wm = 0;
  buf->b_p_sts = 0;
  buf->b_p_ai = 0;
  buf->b_p_et = 0;
  if (buf->b_p_vsts) {
    free_string_option(buf->b_p_vsts);
  }
  buf->b_p_vsts = empty_string_option;
  XFREE_CLEAR(buf->b_p_vsts_array);
}

void nvim_paste_buf_activate_only(buf_T *buf)
{
  buf->b_p_tw = 0;
  buf->b_p_wm = 0;
  buf->b_p_sts = 0;
  buf->b_p_ai = 0;
  buf->b_p_et = 0;
  if (buf->b_p_vsts) {
    free_string_option(buf->b_p_vsts);
  }
  buf->b_p_vsts = empty_string_option;
  XFREE_CLEAR(buf->b_p_vsts_array);
}

void nvim_paste_buf_restore(buf_T *buf)
{
  buf->b_p_tw = buf->b_p_tw_nopaste;
  buf->b_p_wm = buf->b_p_wm_nopaste;
  buf->b_p_sts = buf->b_p_sts_nopaste;
  buf->b_p_ai = buf->b_p_ai_nopaste;
  buf->b_p_et = buf->b_p_et_nopaste;
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

void nvim_paste_global_save(void)
{
  p_ai_nopaste = p_ai;
  p_et_nopaste = p_et;
  p_sts_nopaste = p_sts;
  p_tw_nopaste = p_tw;
  p_wm_nopaste = p_wm;
  if (p_vsts_nopaste) {
    xfree(p_vsts_nopaste);
  }
  p_vsts_nopaste = p_vsts && p_vsts != empty_string_option ? xstrdup(p_vsts) : NULL;
}

void nvim_paste_global_activate(void)
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
  if (p_vsts) {
    free_string_option(p_vsts);
  }
  p_vsts = empty_string_option;
}

void nvim_paste_global_restore(int save_sm, int save_sta, int save_ru, int save_ri)
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
  if (p_vsts) {
    free_string_option(p_vsts);
  }
  p_vsts = p_vsts_nopaste ? xstrdup(p_vsts_nopaste) : empty_string_option;
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

// Get is_option_hidden as an int (for Rust FFI)
int nvim_opt_is_hidden(OptIndex opt_idx) { return (int)is_option_hidden(opt_idx); }

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
void nvim_showoptions(int all, int opt_flags);
void nvim_showoneopt(vimoption_T *opt, int opt_flags);
int nvim_validate_opt_idx(win_T *win, OptIndex opt_idx, int opt_flags, uint32_t flags,
                          int prefix, const char **errmsg);
OptVal nvim_get_option_newval(OptIndex opt_idx, int opt_flags, int prefix, char **argp,
                              char nextchar, int op, uint32_t flags, void *varp,
                              char *errbuf, size_t errbuflen, const char **errmsg);
const char *nvim_rs_set_option(OptIndex opt_idx, OptVal value, int opt_flags,
                               int set_sid, int direct, int value_replaced,
                               char *errbuf, size_t errbuflen);

static int p_bin_dep_opts[] = {
  kOptTextwidth, kOptWrapmargin, kOptModeline, kOptExpandtab, kOptInvalid
};

static int p_paste_dep_opts[] = {
  kOptAutoindent, kOptExpandtab, kOptRuler, kOptShowmatch, kOptSmarttab, kOptSofttabstop,
  kOptTextwidth, kOptWrapmargin, kOptRevins, kOptVarsofttabstop, kOptInvalid
};

// Paste callback: record sctx for all dependent options (after p_paste_dep_opts is defined)
void nvim_paste_didset_options_sctx(void)
{
  didset_options_sctx((OPT_LOCAL | OPT_GLOBAL), p_paste_dep_opts);
}

// bin callback: record sctx for all dependent options (after p_bin_dep_opts is defined)
void nvim_bin_didset_options_sctx(int opt_flags)
{
  didset_options_sctx(opt_flags, p_bin_dep_opts);
}

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
int nvim_curbuf_get_b_p_ml_nobin(void) { return curbuf->b_p_ml; }
void nvim_curbuf_set_b_p_ml_nobin(int v) { curbuf->b_p_ml_nobin = v != 0; }
int nvim_curbuf_get_b_p_et_nobin(void) { return curbuf->b_p_et; }
void nvim_curbuf_set_b_p_et_nobin(int v) { curbuf->b_p_et_nobin = v != 0; }
// nvim_curbuf_get_b_p_tw/wm are defined in ex_cmds_shim.c
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
const char *nvim_option_get_p_flp(void) { return p_flp; }
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
int nvim_option_get_p_ma(void) { return p_ma; }
void nvim_option_set_p_ma(int v) { p_ma = v != 0; }
int nvim_curbuf_get_b_p_ma(void) { return curbuf->b_p_ma; }
void nvim_curbuf_set_b_p_ma(int v) { curbuf->b_p_ma = v != 0; }
// change_option_default wrapper (for reset_modifiable)
void nvim_change_option_default_bool(OptIndex opt_idx, int value) { change_option_default(opt_idx, BOOLEAN_OPTVAL(value != 0)); }
// kOptModifiable index accessor (for reset_modifiable)
int nvim_get_opt_idx_modifiable(void) { return (int)kOptModifiable; }

// Phase 4 pass 2: TTY and key accessors
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
// Option index constants for insecure_flag (nvim_get_opt_idx_wrap already defined above)
int nvim_get_opt_idx_statusline(void) { return (int)kOptStatusline; }
int nvim_get_opt_idx_winbar(void) { return (int)kOptWinbar; }
int nvim_get_opt_idx_foldexpr(void) { return (int)kOptFoldexpr; }
int nvim_get_opt_idx_foldtext(void) { return (int)kOptFoldtext; }
int nvim_get_opt_idx_indentexpr(void) { return (int)kOptIndentexpr; }
int nvim_get_opt_idx_formatexpr(void) { return (int)kOptFormatexpr; }
int nvim_get_opt_idx_includeexpr(void) { return (int)kOptIncludeexpr; }

// set_option_sctx accessors (nvim_get_sourcing_lnum already defined in ex_docmd.c as int)
int64_t nvim_option_get_sourcing_lnum(void) { return (int64_t)SOURCING_LNUM; }
void nvim_call_nlua_set_sctx(sctx_T *sctx) { nlua_set_sctx(sctx); }
void nvim_curbuf_set_p_script_ctx(int idx, sctx_T sctx) { curbuf->b_p_script_ctx[idx] = sctx; }
void nvim_curwin_set_p_script_ctx(int idx, sctx_T sctx) { curwin->w_p_script_ctx[idx] = sctx; }
void nvim_curwin_set_allbuf_opt_script_ctx(int idx, sctx_T sctx) { curwin->w_allbuf_opt.wo_script_ctx[idx] = sctx; }
void nvim_option_set_script_ctx(OptIndex opt_idx, sctx_T sctx) { options[opt_idx].script_ctx = sctx; }

// Phase 4 (session.rs) accessors
OptVal nvim_optval_from_varp(OptIndex opt_idx, void *varp) { return optval_from_varp(opt_idx, varp); }
OptVal nvim_get_option_unset_value(OptIndex opt_idx) { return get_option_unset_value(opt_idx); }
int nvim_option_has_type(OptIndex opt_idx, int type) { return (int)option_has_type(opt_idx, (OptValType)type); }
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
// opt idx constants for makefoldset (nvim_get_opt_idx_foldmethod already defined above)
int nvim_get_opt_idx_foldmarker(void) { return (int)kOptFoldmarker; }
int nvim_get_opt_idx_foldignore(void) { return (int)kOptFoldignore; }
int nvim_get_opt_idx_foldlevel(void) { return (int)kOptFoldlevel; }
int nvim_get_opt_idx_foldminlines(void) { return (int)kOptFoldminlines; }
int nvim_get_opt_idx_foldnestmax(void) { return (int)kOptFoldnestmax; }
int nvim_get_opt_idx_foldenable(void) { return (int)kOptFoldenable; }
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
  set_string_default((OptIndex)opt_idx, val, allocated);
}
void nvim_call_change_option_default(int opt_idx, OptVal value)
{
  change_option_default((OptIndex)opt_idx, value);
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
void nvim_call_set_option_default(int opt_idx, int opt_flags) { set_option_default((OptIndex)opt_idx, opt_flags); }
void nvim_call_comp_col(void) { comp_col(); }
void nvim_call_parse_shape_opt(void) { parse_shape_opt(SHAPE_CURSOR); }
const char *nvim_call_invocation_path_tail(const char *p_sh, size_t *lenp) { return invocation_path_tail(p_sh, lenp); }
int nvim_call_path_fnamecmp(const char *a, const char *b) { return path_fnamecmp(a, b); }
int nvim_curbuf_is_empty(void) { return buf_is_empty(curbuf); }
void nvim_call_set_option_direct(int opt_idx, OptVal val, int opt_flags) { set_option_direct((OptIndex)opt_idx, val, opt_flags, SID_NONE); }

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
// Get the option count
int nvim_get_kopt_count(void) { return (int)kOptCount; }
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

void set_init_tablocal(void)
{
  // susy baka: cmdheight calls itself OPT_GLOBAL but is really tablocal!
  p_ch = options[kOptCmdheight].def_val.data.number;
}

extern void rs_set_init_default_shell(void);

/// Initialize the 'shell' option to a default value.
static void set_init_default_shell(void)
{
  rs_set_init_default_shell();
}

extern void rs_set_init_default_backupskip(void);

/// Set the default for 'backupskip' to include environment variables for
/// temp files.
static void set_init_default_backupskip(void)
{
  rs_set_init_default_backupskip();
}

extern void rs_set_init_default_cdpath(void);

/// Initialize the 'cdpath' option to a default value.
static void set_init_default_cdpath(void)
{
  rs_set_init_default_cdpath();
}

/// Expand environment variables and things like "~" for the defaults.
/// If option_expand() returns non-NULL the variable is expanded.  This can
/// only happen for non-indirect options.
/// Also set the default to the expanded value, so ":set" does not list
/// them.
static void set_init_expand_env(void)
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
      p = option_expand(opt_idx, NULL);
    }
    if (p != NULL) {
      set_option_varp(opt_idx, opt->var, CSTR_TO_OPTVAL(p), true);
      change_option_default(opt_idx, CSTR_TO_OPTVAL(p));
    }
  }
}

extern void rs_set_init_fenc_default(void);

/// Initialize the encoding used for "default" in 'fileencodings'.
static void set_init_fenc_default(void)
{
  rs_set_init_fenc_default();
}

/// Initialize the options, first part.
///
/// Called only once from main(), just after creating the first buffer.
/// If "clean_arg" is true, Nvim was started with --clean.
///
/// NOTE: ELOG() etc calls are not allowed here, as log location depends on
/// env var expansion which depends on expression evaluation and other
/// editor state initialized here. Do logging in set_init_2 or later.
void set_init_1(bool clean_arg)
{
  langmap_init();

  // Allocate the default option values.
  alloc_options_default();

  set_init_default_shell();
  set_init_default_backupskip();
  set_init_default_cdpath();

  char *backupdir = stdpaths_user_state_subpath("backup", 2, true);
  const size_t backupdir_len = strlen(backupdir);
  backupdir = xrealloc(backupdir, backupdir_len + 3);
  memmove(backupdir + 2, backupdir, backupdir_len + 1);
  memmove(backupdir, ".,", 2);
  set_string_default(kOptBackupdir, backupdir, true);
  set_string_default(kOptViewdir, stdpaths_user_state_subpath("view", 2, true),
                     true);
  set_string_default(kOptDirectory, stdpaths_user_state_subpath("swap", 2, true),
                     true);
  set_string_default(kOptUndodir, stdpaths_user_state_subpath("undo", 2, true),
                     true);
  // Set default for &runtimepath. All necessary expansions are performed in
  // this function.
  char *rtp = runtimepath_default(clean_arg);
  if (rtp) {
    set_string_default(kOptRuntimepath, rtp, true);
    // Make a copy of 'rtp' for 'packpath'
    set_string_default(kOptPackpath, rtp, false);
    rtp = NULL;  // ownership taken
  }

  // Set all the options (except the terminal options) to their default
  // value.  Also set the global value for local options.
  set_options_default(0);

  curbuf->b_p_initialized = true;
  curbuf->b_p_ac = -1;
  curbuf->b_p_ar = -1;          // no local 'autoread' value
  curbuf->b_p_ul = NO_LOCAL_UNDOLEVEL;
  check_buf_options(curbuf);
  check_win_options(curwin);
  check_options();

  // set 'laststatus'
  rs_last_status(0);

  // Must be before option_expand(), because that one needs vim_isIDc()
  didset_options();

  // Use the current chartab for the generic chartab. This is not in
  // didset_options() because it only depends on 'encoding'.
  init_spell_chartab();

  // Expand environment variables and things like "~" for the defaults.
  set_init_expand_env();

  save_file_ff(curbuf);         // Buffer is unchanged

  // Detect use of mlterm.
  // Mlterm is a terminal emulator akin to xterm that has some special
  // abilities (bidi namely).
  // NOTE: mlterm's author is being asked to 'set' a variable
  //       instead of an environment variable due to inheritance.
  if (os_env_exists("MLTERM", false)) {
    set_option_value_give_err(kOptTermbidi, BOOLEAN_OPTVAL(true), 0);
  }

  didset_options2();

  lang_init();
  set_init_fenc_default();

#ifdef HAVE_WORKING_LIBINTL
  // GNU gettext 0.10.37 supports this feature: set the codeset used for
  // translated messages independently from the current locale.
  (void)bind_textdomain_codeset(PROJECT_NAME, p_enc);
#endif

  // Set the default for 'helplang'.
  set_helplang_default(get_mess_lang());
}

/// Get default value for option, based on the option's type and scope.
///
/// @param  opt_idx    Option index in options[] table.
/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
///
/// @return Default value of option for the scope specified in opt_flags.
OptVal get_option_default(const OptIndex opt_idx, int opt_flags)
{
  vimoption_T *opt = &options[opt_idx];
  bool is_global_local_option = option_is_global_local(opt_idx);

#ifdef UNIX
  if (opt_idx == kOptModeline && getuid() == ROOT_UID) {
    // 'modeline' defaults to off for root.
    return BOOLEAN_OPTVAL(false);
  }
#endif

  if ((opt_flags & OPT_LOCAL) && is_global_local_option) {
    // Use unset local value instead of default value for local scope of global-local options.
    return get_option_unset_value(opt_idx);
  } else if (option_has_type(opt_idx, kOptValTypeString) && !(opt->flags & kOptFlagNoDefExp)) {
    // For string options, expand environment variables and ~ since the default value was already
    // expanded, only required when an environment variable was set later.
    char *s = option_expand(opt_idx, opt->def_val.data.string.data);
    return s == NULL ? opt->def_val : CSTR_AS_OPTVAL(s);
  } else {
    return opt->def_val;
  }
}

/// Allocate the default values for all options by copying them from the stack.
/// This ensures that we don't need to always check if the option default is allocated or not.
static void alloc_options_default(void)
{
  for (OptIndex opt_idx = 0; opt_idx < kOptCount; opt_idx++) {
    options[opt_idx].def_val = rs_optval_copy(options[opt_idx].def_val);
  }
}

/// Change the default value for an option.
///
/// @param  opt_idx  Option index in options[] table.
/// @param  value    New default value. Must be allocated.
static void change_option_default(const OptIndex opt_idx, OptVal value)
{
  rs_optval_free(options[opt_idx].def_val);
  options[opt_idx].def_val = value;
}

/// Set an option to its default value.
/// This does not take care of side effects!
///
/// @param  opt_idx    Option index in options[] table.
/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
static void set_option_default(const OptIndex opt_idx, int opt_flags)
{
  bool both = (opt_flags & (OPT_LOCAL | OPT_GLOBAL)) == 0;
  OptVal def_val = get_option_default(opt_idx, opt_flags);
  set_option_direct(opt_idx, def_val, opt_flags, current_sctx.sc_sid);

  if (opt_idx == kOptScroll) {
    win_comp_scroll(curwin);
  }

  // The default value is not insecure.
  uint32_t *flagsp = insecure_flag(curwin, opt_idx, opt_flags);
  *flagsp = *flagsp & ~(unsigned)kOptFlagInsecure;
  if (both) {
    flagsp = insecure_flag(curwin, opt_idx, OPT_LOCAL);
    *flagsp = *flagsp & ~(unsigned)kOptFlagInsecure;
  }
}

/// Set all options (except terminal options) to their default value.
///
/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
static void set_options_default(int opt_flags)
{
  for (OptIndex opt_idx = 0; opt_idx < kOptCount; opt_idx++) {
    if (!(options[opt_idx].flags & kOptFlagNoDefault)) {
      set_option_default(opt_idx, opt_flags);
    }
  }

  // The 'scroll' option must be computed for all windows.
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    win_comp_scroll(wp);
  }

  parse_cino(curbuf);
}

/// Set the Vi-default value of a string option.
/// Used for 'sh', 'backupskip' and 'term'.
///
/// @param  opt_idx    Option index in options[] table.
/// @param  val        The value of the option.
/// @param  allocated  If true, do not copy default as it was already allocated.
///
/// TODO(famiu): Remove this.
static void set_string_default(OptIndex opt_idx, char *val, bool allocated)
  FUNC_ATTR_NONNULL_ALL
{
  assert(opt_idx != kOptInvalid);
  change_option_default(opt_idx, CSTR_AS_OPTVAL(allocated ? val : xstrdup(val)));
}

#if defined(EXITFREE)
/// Free all options.
void free_all_options(void)
{
  for (OptIndex opt_idx = 0; opt_idx < kOptCount; opt_idx++) {
    bool hidden = is_option_hidden(opt_idx);

    if (option_is_global_only(opt_idx) || hidden) {
      // global option: free value and default value.
      // hidden option: free default value only.
      if (!hidden) {
        rs_optval_free(optval_from_varp(opt_idx, options[opt_idx].var));
      }
    } else if (!option_is_window_local(opt_idx)) {
      // buffer-local option: free global value.
      rs_optval_free(optval_from_varp(opt_idx, options[opt_idx].var));
    }
    rs_optval_free(options[opt_idx].def_val);
  }
  free_operatorfunc_option();
  rs_free_tagfunc_option();
  free_findfunc_option();
  XFREE_CLEAR(fenc_default);
  XFREE_CLEAR(p_term);
  XFREE_CLEAR(p_ttytype);
}
#endif

/// Initialize the options, part two: After getting Rows and Columns.
void set_init_2(bool headless)
{
  rs_set_init_2(headless ? 1 : 0);
}

/// Initialize the options, part three: After reading the .vimrc
void set_init_3(void)
{
  rs_set_init_3();
}

/// When 'helplang' is still at its default value, set it to "lang".
/// Only the first two characters of "lang" are used.
void set_helplang_default(const char *lang)
{
  rs_set_helplang_default(lang);
}

extern void rs_set_title_defaults(void);

/// 'title' and 'icon' only default to true if they have not been set or reset
/// in .vimrc and we can read the old value.
/// When 'title' and 'icon' have been reset in .vimrc, we won't even check if
/// they can be reset.  This reduces startup time when using X on a remote
/// machine.
void set_title_defaults(void)
{
  rs_set_title_defaults();
}

void ex_set(exarg_T *eap)
{
  int flags = 0;

  if (eap->cmdidx == CMD_setlocal) {
    flags = OPT_LOCAL;
  } else if (eap->cmdidx == CMD_setglobal) {
    flags = OPT_GLOBAL;
  }
  if (eap->forceit) {
    flags |= OPT_ONECOLUMN;
  }
  do_set(eap->arg, flags);
}

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

static set_op_T get_op(const char *arg)
{
  return (set_op_T)rs_get_op(arg);
}

static set_prefix_T get_option_prefix(char **argp)
{
  return (set_prefix_T)rs_get_option_prefix(argp);
}

static int validate_opt_idx(win_T *win, OptIndex opt_idx, int opt_flags, uint32_t flags,
                            set_prefix_T prefix, const char **errmsg)
{
  // Only bools can have a prefix of 'inv' or 'no'
  if (!option_has_type(opt_idx, kOptValTypeBoolean) && prefix != PREFIX_NONE) {
    *errmsg = e_invarg;
    return FAIL;
  }

  // Skip all options that are not window-local (used when showing
  // an already loaded buffer in a window).
  if ((opt_flags & OPT_WINONLY) && !option_is_window_local(opt_idx)) {
    return FAIL;
  }

  // Skip all options that are window-local (used for :vimgrep).
  if ((opt_flags & OPT_NOWIN) && option_is_window_local(opt_idx)) {
    return FAIL;
  }

  // Disallow changing some options from modelines.
  if (opt_flags & OPT_MODELINE) {
    if (flags & (kOptFlagSecure | kOptFlagNoML)) {
      *errmsg = e_not_allowed_in_modeline;
      return FAIL;
    }
    if ((flags & kOptFlagMLE) && !p_mle) {
      *errmsg = e_not_allowed_in_modeline_when_modelineexpr_is_off;
      return FAIL;
    }
    // In diff mode some options are overruled.  This avoids that
    // 'foldmethod' becomes "marker" instead of "diff" and that
    // "wrap" gets set.
    if (win->w_p_diff && (opt_idx == kOptFoldmethod || opt_idx == kOptWrap)) {
      return FAIL;
    }
  }

  // Disallow changing some options in the sandbox
  if (sandbox != 0 && (flags & kOptFlagSecure)) {
    *errmsg = e_sandbox;
    return FAIL;
  }

  return OK;
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

/// Get new option value from argp. Allocated OptVal must be freed by caller.
/// Can unset local value of an option when ":set {option}<" is used.
static OptVal get_option_newval(OptIndex opt_idx, int opt_flags, set_prefix_T prefix, char **argp,
                                int nextchar, set_op_T op, uint32_t flags, void *varp, char *errbuf,
                                const size_t errbuflen, const char **errmsg)
  FUNC_ATTR_WARN_UNUSED_RESULT
{
  assert(varp != NULL);

  vimoption_T *opt = &options[opt_idx];
  char *arg = *argp;
  // When setting the local value of a global option, the old value may be the global value.
  const bool oldval_is_global = option_is_global_local(opt_idx) && (opt_flags & OPT_LOCAL);
  OptVal oldval = optval_from_varp(opt_idx, oldval_is_global ? get_varp(opt) : varp);
  OptVal newval = NIL_OPTVAL;

  if (nextchar == '&') {
    // ":set opt&": Reset to default value.
    // NOTE: Use OPT_GLOBAL instead of opt_flags to ensure we don't use the unset local value for
    // global-local options when OPT_LOCAL is used.
    return rs_optval_copy(get_option_default(opt_idx, OPT_GLOBAL));
  } else if (nextchar == '<') {
    // ":set opt<": Reset to global value.
    // ":setlocal opt<": Copy global value to local value.
    if (option_is_global_local(opt_idx) && !(opt_flags & OPT_LOCAL)) {
      unset_option_local_value(opt_idx);
    }
    return get_option_value(opt_idx, OPT_GLOBAL);
  }

  switch (oldval.type) {
  case kOptValTypeNil:
    abort();
  case kOptValTypeBoolean: {
    TriState newval_bool;

    // ":set opt!": invert
    if (nextchar == '!') {
      switch (oldval.data.boolean) {
      case kNone:
        newval_bool = kNone;
        break;
      case kTrue:
        newval_bool = kFalse;
        break;
      case kFalse:
        newval_bool = kTrue;
        break;
      }
    } else {
      // ":set invopt": invert
      // ":set opt" or ":set noopt": set or reset
      if (prefix == PREFIX_INV) {
        newval_bool = *(int *)varp ^ 1;
      } else {
        newval_bool = prefix == PREFIX_NO ? 0 : 1;
      }
    }

    newval = BOOLEAN_OPTVAL(newval_bool);
    break;
  }
  case kOptValTypeNumber: {
    OptInt oldval_num = oldval.data.number;
    OptInt newval_num;

    // Different ways to set a number option:
    // <xx>         accept special key codes for 'wildchar' or 'wildcharm'
    // ^x           accept ctrl key codes for 'wildchar' or 'wildcharm'
    // c            accept any non-digit for 'wildchar' or 'wildcharm'
    // [-]0-9       set number
    // other        error
    arg++;
    if (((OptInt *)varp == &p_wc || (OptInt *)varp == &p_wcm)
        && (*arg == '<' || *arg == '^'
            || (*arg != NUL && (!arg[1] || ascii_iswhite(arg[1])) && !ascii_isdigit(*arg)))) {
      newval_num = string_to_key(arg);
      if (newval_num == 0) {
        *errmsg = e_invarg;
        return newval;
      }
    } else if (*arg == '-' || ascii_isdigit(*arg)) {
      int i;
      // Allow negative, octal and hex numbers.
      vim_str2nr(arg, NULL, &i, STR2NR_ALL, &newval_num, NULL, 0, true, NULL);
      if (i == 0 || (arg[i] != NUL && !ascii_iswhite(arg[i]))) {
        *errmsg = e_number_required_after_equal;
        return newval;
      }
    } else {
      *errmsg = e_number_required_after_equal;
      return newval;
    }

    if (op == OP_ADDING) {
      newval_num = oldval_num + newval_num;
    }
    if (op == OP_PREPENDING) {
      newval_num = oldval_num * newval_num;
    }
    if (op == OP_REMOVING) {
      newval_num = oldval_num - newval_num;
    }

    newval = NUMBER_OPTVAL(newval_num);
    break;
  }
  case kOptValTypeString: {
    const char *oldval_str = oldval.data.string.data;
    // Get the new value for the option
    const char *newval_str = stropt_get_newval(nextchar, opt_idx, argp, varp, oldval_str, &op,
                                               flags);
    newval = CSTR_AS_OPTVAL(newval_str);
    break;
  }
  }

  return newval;
}

static void do_one_set_option(int opt_flags, char **argp, bool *did_show, char *errbuf,
                              size_t errbuflen, const char **errmsg)
{
  // 1: nothing, 0: "no", 2: "inv" in front of name
  set_prefix_T prefix = get_option_prefix(argp);

  char *arg = *argp;

  // find end of name
  OptIndex opt_idx;
  const char *const option_end = find_option_end(arg, &opt_idx);

  if (opt_idx != kOptInvalid) {
    assert(option_end >= arg);
  } else if (rs_is_tty_option(arg)) {  // Silently ignore TTY options.
    return;
  } else {                          // Invalid option name, skip.
    *errmsg = e_unknown_option;
    return;
  }

  // Remember character after option name.
  uint8_t afterchar = (uint8_t)(*option_end);
  char *p = (char *)option_end;

  // Skip white space, allow ":set ai  ?".
  while (ascii_iswhite(*p)) {
    p++;
  }

  set_op_T op = get_op(p);
  if (op != OP_NONE) {
    p++;
  }

  uint8_t nextchar = (uint8_t)(*p);  // next non-white char after option name
  // flags for current option
  uint32_t flags = options[opt_idx].flags;
  // pointer to variable for current option
  void *varp = get_varp_scope(&(options[opt_idx]), opt_flags);

  if (validate_opt_idx(curwin, opt_idx, opt_flags, flags, prefix, errmsg) == FAIL) {
    return;
  }

  if (vim_strchr("?=:!&<", nextchar) != NULL) {
    *argp = p;

    if (nextchar == '&' && (*argp)[1] == 'v' && (*argp)[2] == 'i') {
      if ((*argp)[3] == 'm') {  // "opt&vim": set to Vim default
        *argp += 3;
      } else {  // "opt&vi": set to Vi default
        *argp += 2;
      }
    }
    if (vim_strchr("?!&<", nextchar) != NULL
        && (*argp)[1] != NUL && !ascii_iswhite((*argp)[1])) {
      *errmsg = e_trailing;
      return;
    }
  }

  // Allow '=' and ':' as MS-DOS command.com allows only one '=' character per "set" command line.
  if (nextchar == '?'
      || (prefix == PREFIX_NONE && vim_strchr("=:&<", nextchar) == NULL
          && !option_has_type(opt_idx, kOptValTypeBoolean))) {
    // print value
    if (*did_show) {
      msg_putchar('\n');                // cursor below last one
    } else {
      msg_ext_set_kind("list_cmd");
      gotocmdline(true);                // cursor at status line
      *did_show = true;                 // remember that we did a line
    }
    showoneopt(&options[opt_idx], opt_flags);

    if (p_verbose > 0) {
      // Mention where the option was last set.
      if (varp == options[opt_idx].var) {
        last_set_msg(options[opt_idx].script_ctx);
      } else if (option_has_scope(opt_idx, kOptScopeWin)) {
        last_set_msg(curwin->w_p_script_ctx[option_scope_idx(opt_idx, kOptScopeWin)]);
      } else if (option_has_scope(opt_idx, kOptScopeBuf)) {
        last_set_msg(curbuf->b_p_script_ctx[option_scope_idx(opt_idx, kOptScopeBuf)]);
      }
    }

    if (nextchar != '?' && nextchar != NUL && !ascii_iswhite(afterchar)) {
      *errmsg = e_trailing;
    }
    return;
  }

  if (option_has_type(opt_idx, kOptValTypeBoolean)) {
    if (vim_strchr("=:", nextchar) != NULL) {
      *errmsg = e_invarg;
      return;
    }

    if (vim_strchr("!&<", nextchar) == NULL && nextchar != NUL && !ascii_iswhite(afterchar)) {
      *errmsg = e_trailing;
      return;
    }
  } else {
    if (vim_strchr("=:&<", nextchar) == NULL) {
      *errmsg = e_invarg;
      return;
    }
  }

  OptVal newval = get_option_newval(opt_idx, opt_flags, prefix, argp, nextchar, op, flags, varp,
                                    errbuf, errbuflen, errmsg);

  if (newval.type == kOptValTypeNil || *errmsg != NULL) {
    return;
  }

  *errmsg = set_option(opt_idx, newval, opt_flags, 0, false, op == OP_NONE, errbuf, errbuflen);
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
int do_set(char *arg, int opt_flags)
{
  bool did_show = false;             // already showed one value

  if (*arg == NUL) {
    showoptions(false, opt_flags);
    did_show = true;
  } else {
    while (*arg != NUL) {         // loop to process all options
      if (strncmp(arg, "all", 3) == 0 && !ASCII_ISALPHA(arg[3])
          && !(opt_flags & OPT_MODELINE)) {
        // ":set all"  show all options.
        // ":set all&" set all options to their default value.
        arg += 3;
        if (*arg == '&') {
          arg++;
          // Only for :set command set global value of local options.
          set_options_default(opt_flags);
          didset_options();
          didset_options2();
          ui_refresh_options();
          redraw_all_later(UPD_CLEAR);
        } else {
          showoptions(true, opt_flags);
          did_show = true;
        }
      } else {
        char *startarg = arg;             // remember for error message
        const char *errmsg = NULL;
        char errbuf[ERR_BUFLEN];

        do_one_set_option(opt_flags, &arg, &did_show, errbuf, sizeof(errbuf), &errmsg);

        // Advance to next argument.
        // - skip until a blank found, taking care of backslashes
        // - skip blanks
        // - skip one "=val" argument (for hidden options ":set gfn =xx")
        for (int i = 0; i < 2; i++) {
          arg = skiptowhite_esc(arg);
          arg = skipwhite(arg);
          if (*arg != '=') {
            break;
          }
        }

        if (errmsg != NULL) {
          int i = vim_snprintf((char *)IObuff, IOSIZE, "%s", _(errmsg)) + 2;
          if (i + (arg - startarg) < IOSIZE) {
            // append the argument with the error
            xstrlcpy(IObuff + i - 2, ": ", (size_t)(IOSIZE - i + 2));
            assert(arg >= startarg);
            memmove(IObuff + i, startarg, (size_t)(arg - startarg));
            IObuff[i + (arg - startarg)] = NUL;
          }
          // make sure all characters are printable
          trans_characters(IObuff, IOSIZE);

          no_wait_return++;         // wait_return() done later
          emsg(IObuff);             // show error highlighted
          no_wait_return--;

          return FAIL;
        }
      }

      arg = skipwhite(arg);
    }
  }

  if (silent_mode && did_show) {
    // After displaying option values in silent mode.
    silent_mode = false;
    info_message = true;        // use stdout, not stderr
    msg_putchar('\n');
    silent_mode = true;
    info_message = false;       // use stdout, not stderr
  }

  return OK;
}

/// Convert a key name or string into a key value.
/// Used for 'cedit', 'wildchar' and 'wildcharm' options.
int string_to_key(char *arg)
{
  return rs_string_to_key(arg);
}

// When changing 'title', 'titlestring', 'icon' or 'iconstring', call
// maketitle() to create and display it.
/// set_options_bin -  called when 'bin' changes value.
///
/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
void set_options_bin(int oldval, int newval, int opt_flags)
{
  rs_set_options_bin(oldval, newval, opt_flags);
}

/// Expand environment variables for some string options.
/// These string options cannot be indirect!
/// If "val" is NULL expand the current value of the option.
/// Return pointer to NameBuff, or NULL when not expanded.
static char *option_expand(OptIndex opt_idx, const char *val)
{
  // if option doesn't need expansion nothing to do
  if (!(options[opt_idx].flags & kOptFlagExpand) || is_option_hidden(opt_idx)) {
    return NULL;
  }

  if (val == NULL) {
    val = *(char **)options[opt_idx].var;
  }

  // If val is longer than MAXPATHL no meaningful expansion can be done,
  // expand_env() would truncate the string.
  if (val == NULL || strlen(val) > MAXPATHL) {
    return NULL;
  }

  // Expanding this with NameBuff, expand_env() must not be passed IObuff.
  // Escape spaces when expanding 'tags' or 'path', they are used to separate
  // file names.
  // For 'spellsuggest' expand after "file:".
  char **var = (char **)options[opt_idx].var;
  bool esc = var == &p_tags || var == &p_path;
  expand_env_esc(val, NameBuff, MAXPATHL, esc, false,
                 (char **)options[opt_idx].var == &p_sps ? "file:" : NULL);
  if (strcmp(NameBuff, val) == 0) {   // they are the same
    return NULL;
  }

  return NameBuff;
}

/// Non-static wrapper for option_expand(), for Rust FFI.
/// Returns expanded string (in static NameBuff), or NULL if no expansion.
char *nvim_option_expand(OptIndex opt_idx, const char *val)
{
  return option_expand(opt_idx, val);
}

/// Get the address of p_kp (keywordprg global), as void*.
/// Used by Rust stropt_get_newval to detect keywordprg option.
void *nvim_option_get_p_kp_ptr(void)
{
  return (void *)&p_kp;
}

/// After setting various option values: recompute variables that depend on
/// option values.
static void didset_options(void)
{
  // initialize the table for 'iskeyword' et.al.
  init_chartab();

  didset_string_options();

  spell_check_msm();
  spell_check_sps();
  compile_cap_prog(curwin->w_s);
  did_set_spell_option();
  // set cedit_key
  did_set_cedit(NULL);
  // initialize the table for 'breakat'.
  did_set_breakat(NULL);
  didset_window_options(curwin, true);
}

// More side effects of setting options.
static void didset_options2(void)
{
  // Initialize the highlight_attr[] table.
  highlight_changed();

  // Parse default for 'fillchars'.
  set_chars_option(curwin, curwin->w_p_fcs, kFillchars, true, NULL, 0);

  // Parse default for 'listchars'.
  set_chars_option(curwin, curwin->w_p_lcs, kListchars, true, NULL, 0);

  // Parse default for 'wildmode'.
  check_opt_wim();
  xfree(curbuf->b_p_vsts_array);
  tabstop_set(curbuf->b_p_vsts, &curbuf->b_p_vsts_array);
  xfree(curbuf->b_p_vts_array);
  tabstop_set(curbuf->b_p_vts,  &curbuf->b_p_vts_array);
}

/// Check for string options that are NULL (normally only termcap options).
void check_options(void)
{
  for (OptIndex opt_idx = 0; opt_idx < kOptCount; opt_idx++) {
    if ((option_has_type(opt_idx, kOptValTypeString)) && options[opt_idx].var != NULL) {
      check_string_option((char **)get_varp(&(options[opt_idx])));
    }
  }
}

/// Check if option was set insecurely.
///
/// @param  wp         Window.
/// @param  opt_idx    Option index in options[] table.
/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
///
/// @return  True if option was set from a modeline or in secure mode, false if it wasn't.
int was_set_insecurely(win_T *const wp, OptIndex opt_idx, int opt_flags)
{
  return rs_was_set_insecurely(wp, opt_idx, opt_flags);
}

/// Get a pointer to the flags used for the kOptFlagInsecure flag of option
/// "opt_idx".  For some local options a local flags field is used.
/// NOTE: Caller must make sure that "wp" is set to the window from which
/// the option is used.
uint32_t *insecure_flag(win_T *const wp, OptIndex opt_idx, int opt_flags)
{
  return rs_insecure_flag(wp, opt_idx, opt_flags);
}

/// Redraw the window title and/or tab page text later.
void redraw_titles(void)
{
  rs_redraw_titles();
}

extern void rs_check_blending(win_T *wp);

void check_blending(win_T *wp)
{
  rs_check_blending(wp);
}

/// Handle setting `winhighlight' in window "wp"
///
/// @param winhl  when NULL: use "wp->w_p_winhl"
/// @param wp     when NULL: only parse "winhl"
///
/// @return  whether the option value is valid.

/// Get the script context of global option at index opt_idx.
sctx_T *get_option_sctx(OptIndex opt_idx)
{
  assert(opt_idx != kOptInvalid);
  return &options[opt_idx].script_ctx;
}

/// Set the script_ctx for an option, taking care of setting the buffer- or
/// window-local value.
void set_option_sctx(OptIndex opt_idx, int opt_flags, sctx_T script_ctx)
{
  rs_set_option_sctx(opt_idx, opt_flags, script_ctx);
}

/// Apply the OptionSet autocommand.
static void apply_optionset_autocmd(OptIndex opt_idx, int opt_flags, OptVal oldval, OptVal oldval_g,
                                    OptVal oldval_l, OptVal newval, const char *errmsg)
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




/// Process the new global 'undolevels' option value.
const char *did_set_global_undolevels(OptInt value, OptInt old_value)
{
  // sync undo before 'undolevels' changes
  // use the old value, otherwise u_sync() may not work properly
  p_ul = old_value;
  u_sync(true);
  p_ul = value;
  return NULL;
}

/// Process the new buffer local 'undolevels' option value.
const char *did_set_buflocal_undolevels(buf_T *buf, OptInt value, OptInt old_value)
{
  // use the old value, otherwise u_sync() may not work properly
  buf->b_p_ul = old_value;
  u_sync(true);
  buf->b_p_ul = value;
  return NULL;
}


extern void rs_do_syntax_autocmd(buf_T *buf, int value_changed);
extern void rs_do_spelllang_source(win_T *win);

// When 'syntax' is set, load the syntax of that name
static void do_syntax_autocmd(buf_T *buf, bool value_changed)
{
  rs_do_syntax_autocmd(buf, value_changed ? 1 : 0);
}

static void do_spelllang_source(win_T *win)
{
  rs_do_spelllang_source(win);
}

/// Called after an option changed: check if something needs to be redrawn.
void check_redraw_for(buf_T *buf, win_T *win, uint32_t flags)
{
  rs_check_redraw_for(buf, win, flags);
}

void check_redraw(uint32_t flags)
{
  check_redraw_for(curbuf, curwin, flags);
}

/// Get value of TTY option.
///
/// @param  name  Name of TTY option.
///
/// @return [allocated] TTY option value. Returns NIL_OPTVAL if option isn't a TTY option.
OptVal get_tty_option(const char *name)
{
  return rs_get_tty_option(name);
}

bool set_tty_option(const char *name, char *value)
{
  return rs_set_tty_option(name, value);
}

/// Find index for an option. Don't go beyond `len` length.
///
/// @param[in]  name  Option name.
/// @param      len   Option name length.
///
/// @return Option index or kOptInvalid if option was not found.
OptIndex find_option_len(const char *const name, size_t len)
  FUNC_ATTR_NONNULL_ALL
{
  int index = find_option_hash(name, len);
  return index >= 0 ? option_hash_elems[index].opt_idx : kOptInvalid;
}

/// Find index for an option.
///
/// @param[in]  name  Option name.
///
/// @return Option index or kOptInvalid if option was not found.
OptIndex find_option(const char *const name)
  FUNC_ATTR_NONNULL_ALL
{
  return find_option_len(name, strlen(name));
}



/// Get type of option.
static OptValType option_get_type(const OptIndex opt_idx)
{
  return options[opt_idx].type;
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
  // Special case: 'modified' is b_changed, but we also want to consider it set when 'ff' or 'fenc'
  // changed.
  if ((int *)varp == &curbuf->b_changed) {
    return BOOLEAN_OPTVAL(curbufIsChanged());
  }

  OptValType type = option_get_type(opt_idx);

  switch (type) {
  case kOptValTypeNil:
    return NIL_OPTVAL;
  case kOptValTypeBoolean:
    return BOOLEAN_OPTVAL(TRISTATE_FROM_INT(*(int *)varp));
  case kOptValTypeNumber:
    return NUMBER_OPTVAL(*(OptInt *)varp);
  case kOptValTypeString:
    return STRING_OPTVAL(cstr_as_string(*(char **)varp));
  }
  UNREACHABLE;
}

/// Set option var pointer value from OptVal.
///
/// @param       opt_idx      Option index in options[] table.
/// @param[out]  varp         Pointer to option variable.
/// @param[in]   value        New option value.
/// @param       free_oldval  Free old value.
static void set_option_varp(OptIndex opt_idx, void *varp, OptVal value, bool free_oldval)
  FUNC_ATTR_NONNULL_ARG(2)
{
  assert(option_has_type(opt_idx, value.type));

  if (free_oldval) {
    rs_optval_free(optval_from_varp(opt_idx, varp));
  }

  switch (value.type) {
  case kOptValTypeNil:
    abort();
  case kOptValTypeBoolean:
    *(int *)varp = value.data.boolean;
    return;
  case kOptValTypeNumber:
    *(OptInt *)varp = value.data.number;
    return;
  case kOptValTypeString:
    *(char **)varp = value.data.string.data;
    return;
  }
  UNREACHABLE;
}

/// Return C-string representation of OptVal. Caller must free the returned C-string.
static char *optval_to_cstr(OptVal o)
{
  switch (o.type) {
  case kOptValTypeNil:
    return xstrdup("");
  case kOptValTypeBoolean:
    return xstrdup(o.data.boolean ? "true" : "false");
  case kOptValTypeNumber: {
    char *buf = xmalloc(NUMBUFLEN);
    snprintf(buf, NUMBUFLEN, "%" PRId64, o.data.number);
    return buf;
  }
  case kOptValTypeString: {
    char *buf = xmalloc(o.data.string.size + 3);
    snprintf(buf, o.data.string.size + 3, "\"%s\"", o.data.string.data);
    return buf;
  }
  }
  UNREACHABLE;
}

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

/// Check if option is hidden.
///
/// @param  opt_idx  Option index in options[] table.
///
/// @return  True if option is hidden, false otherwise. Returns false if option name is invalid.
bool is_option_hidden(OptIndex opt_idx)
{
  // Hidden options are always immutable and point to their default value
  return opt_idx != kOptInvalid && options[opt_idx].immutable
         && options[opt_idx].var == &options[opt_idx].def_val.data;
}

/// Check if option supports a specific type.
bool option_has_type(OptIndex opt_idx, OptValType type)
{
  return opt_idx != kOptInvalid && options[opt_idx].type == type;
}

/// Check if option supports a specific scope.
bool option_has_scope(OptIndex opt_idx, OptScope scope)
{
  // Ensure that scope flags variable can hold all scopes.
  STATIC_ASSERT(kOptScopeSize <= sizeof(OptScopeFlags) * 8,
                "Option scope_flags cannot fit all option scopes");
  // Ensure that the scope is valid before accessing scope_flags.
  assert(scope >= kOptScopeGlobal && scope < kOptScopeSize);
  // Bitshift 1 by the value of scope to get the scope's corresponding flag, and check if it's set
  // in the scope_flags bit field.
  return get_option(opt_idx)->scope_flags & (1 << scope);
}

/// Check if option is global-local.
static inline bool option_is_global_local(OptIndex opt_idx)
{
  // Global-local options have at least two types, so their type flag cannot be a power of two.
  return opt_idx != kOptInvalid && !is_power_of_two(options[opt_idx].scope_flags);
}

/// Check if option only supports global scope.
static inline bool option_is_global_only(OptIndex opt_idx)
{
  // For an option to be global-only, it has to only have a single scope, which means the scope
  // flags must be a power of two, and it must have the global scope.
  return opt_idx != kOptInvalid && is_power_of_two(options[opt_idx].scope_flags)
         && option_has_scope(opt_idx, kOptScopeGlobal);
}

/// Check if option only supports window scope.
static inline bool option_is_window_local(OptIndex opt_idx)
{
  // For an option to be window-local it has to only have a single scope, which means the scope
  // flags must be a power of two, and it must have the window scope.
  return opt_idx != kOptInvalid && is_power_of_two(options[opt_idx].scope_flags)
         && option_has_scope(opt_idx, kOptScopeWin);
}

/// Get option index for scope.
ssize_t option_scope_idx(OptIndex opt_idx, OptScope scope)
{
  return options[opt_idx].scope_idx[scope];
}

// =============================================================================
// Non-static wrappers for Rust FFI
// =============================================================================

int nvim_option_is_global_local(OptIndex opt_idx) { return option_is_global_local(opt_idx); }
int nvim_option_is_global_only(OptIndex opt_idx) { return option_is_global_only(opt_idx); }
int nvim_option_is_window_local(OptIndex opt_idx) { return option_is_window_local(opt_idx); }

/// Get option flags.
///
/// @param  opt_idx  Option index in options[] table.
///
/// @return  Option flags. Returns 0 for invalid option name.
uint32_t get_option_flags(OptIndex opt_idx)
{
  return opt_idx == kOptInvalid ? 0 : options[opt_idx].flags;
}

/// Gets the value for an option.
///
/// @param  opt_idx    Option index in options[] table.
/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
///
/// @return [allocated] Option value. Returns NIL_OPTVAL for invalid option index.
OptVal get_option_value(OptIndex opt_idx, int opt_flags)
{
  if (opt_idx == kOptInvalid) {  // option not in the options[] table.
    return NIL_OPTVAL;
  }

  vimoption_T *opt = &options[opt_idx];
  void *varp = get_varp_scope(opt, opt_flags);

  return rs_optval_copy(optval_from_varp(opt_idx, varp));
}

/// Return information for option at 'opt_idx'
vimoption_T *get_option(OptIndex opt_idx)
{
  assert(opt_idx != kOptInvalid);
  return &options[opt_idx];
}

/// Get option value that represents an unset local value for an option.
/// TODO(famiu): Remove this once we have a dedicated OptVal type for unset local options.
///
/// @param      opt_idx  Option index in options[] table.
/// @param[in]  varp  Pointer to option variable.
///
/// @return Option value equal to the unset value for the option.
static OptVal get_option_unset_value(OptIndex opt_idx)
{
  assert(opt_idx != kOptInvalid);
  vimoption_T *opt = &options[opt_idx];

  // For global-local options, use the unset value of the local value.
  if (option_is_global_local(opt_idx)) {
    // String global-local options always use an empty string for the unset value.
    if (option_has_type(opt_idx, kOptValTypeString)) {
      return STATIC_CSTR_AS_OPTVAL("");
    }

    switch (opt_idx) {
    case kOptAutocomplete:
    case kOptAutoread:
      return BOOLEAN_OPTVAL(kNone);
    case kOptScrolloff:
    case kOptSidescrolloff:
      return NUMBER_OPTVAL(-1);
    case kOptUndolevels:
      return NUMBER_OPTVAL(NO_LOCAL_UNDOLEVEL);
    default:
      abort();
    }
  }

  // For options that aren't global-local, use the global value to represent an unset local value.
  return optval_from_varp(opt_idx, get_varp_scope(opt, OPT_GLOBAL));
}

/// Check if local value of global-local option is unset for current buffer / window.
/// Always returns false for options that aren't global-local.
///
/// TODO(famiu): Remove this once we have an OptVal type to indicate an unset local value.
static bool is_option_local_value_unset(OptIndex opt_idx)
{
  vimoption_T *opt = get_option(opt_idx);

  // Local value of option that isn't global-local is always considered set.
  if (!option_is_global_local(opt_idx)) {
    return false;
  }

  void *varp_local = get_varp_scope(opt, OPT_LOCAL);
  OptVal local_value = optval_from_varp(opt_idx, varp_local);
  OptVal unset_local_value = get_option_unset_value(opt_idx);

  return rs_optval_equal(local_value, unset_local_value) != 0;
}

/// Handle side-effects of setting an option.
///
/// @param       opt_idx         Index in options[] table. Must not be kOptInvalid.
/// @param[in]   varp            Option variable pointer, cannot be NULL.
/// @param       old_value       Old option value.
/// @param       opt_flags       Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
/// @param       set_sid         Script ID. Special values:
///                                0: Use current script ID.
///                                SID_NONE: Don't set script ID.
/// @param       direct          Don't process side-effects.
/// @param       value_replaced  Value was replaced completely.
/// @param[out]  errbuf          Buffer for error message.
/// @param       errbuflen       Length of error buffer.
///
/// @return  NULL on success, an untranslated error message on error.
static const char *did_set_option(OptIndex opt_idx, void *varp, OptVal old_value, OptVal new_value,
                                  int opt_flags, scid_T set_sid, const bool direct,
                                  const bool value_replaced, char *errbuf, size_t errbuflen)
{
  vimoption_T *opt = &options[opt_idx];
  const char *errmsg = NULL;
  bool restore_chartab = false;
  bool value_changed = false;
  bool value_checked = false;

  optset_T did_set_cb_args = {
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

  if (direct) {
    // Don't do any extra processing if setting directly.
  }
  // Disallow changing immutable options.
  else if (opt->immutable && rs_optval_equal(old_value, new_value) == 0) {
    errmsg = e_unsupportedoption;
  }
  // Disallow changing some options from secure mode.
  else if ((secure || sandbox != 0) && (opt->flags & kOptFlagSecure)) {
    errmsg = e_secure;
  }
  // Check for a "normal" directory or file name in some string options.
  else if (new_value.type == kOptValTypeString
           && check_illegal_path_names(*(char **)varp, opt->flags)) {
    errmsg = e_invarg;
  } else if (opt->opt_did_set_cb != NULL) {
    // Invoke the option specific callback function to validate and apply the new value.
    errmsg = opt->opt_did_set_cb(&did_set_cb_args);
    // The 'filetype' and 'syntax' option callback functions may change the os_value_changed field.
    value_changed = did_set_cb_args.os_value_changed;
    // The 'keymap', 'filetype' and 'syntax' option callback functions may change the
    // os_value_checked field.
    value_checked = did_set_cb_args.os_value_checked;
    // The 'isident', 'iskeyword', 'isprint' and 'isfname' options may change the character table.
    // On failure, this needs to be restored.
    restore_chartab = did_set_cb_args.os_restore_chartab;
  }

  // If option is hidden or if an error is detected, restore the previous value and don't do any
  // further processing.
  if (errmsg != NULL) {
    set_option_varp(opt_idx, varp, old_value, true);
    // When resetting some values, need to act on it.
    if (restore_chartab) {
      buf_init_chartab(curbuf, true);
    }

    return errmsg;
  }

  // Re-assign the new value as its value may get freed or modified by the option callback.
  new_value = optval_from_varp(opt_idx, varp);

  if (set_sid != SID_NONE) {
    sctx_T script_ctx = set_sid == 0 ? current_sctx : (sctx_T){ .sc_sid = set_sid };
    // Remember where the option was set.
    set_option_sctx(opt_idx, opt_flags, script_ctx);
  }

  rs_optval_free(old_value);

  const bool scope_both = (opt_flags & (OPT_LOCAL | OPT_GLOBAL)) == 0;

  if (scope_both) {
    if (option_is_global_local(opt_idx)) {
      // Global option with local value set to use global value.
      // Free the local value and clear it.
      void *varp_local = get_varp_scope(opt, OPT_LOCAL);
      OptVal local_unset_value = get_option_unset_value(opt_idx);
      set_option_varp(opt_idx, varp_local, rs_optval_copy(local_unset_value), true);
    } else {
      // May set global value for local option.
      void *varp_global = get_varp_scope(opt, OPT_GLOBAL);
      set_option_varp(opt_idx, varp_global, rs_optval_copy(new_value), true);
    }
  }

  // Don't do anything else if setting the option directly.
  if (direct) {
    return errmsg;
  }

  // Trigger the autocommand only after setting the flags.
  if (varp == &curbuf->b_p_syn) {
    do_syntax_autocmd(curbuf, value_changed);
  } else if (varp == &curbuf->b_p_ft) {
    // 'filetype' is set, trigger the FileType autocommand
    // Skip this when called from a modeline
    // Force autocmd when the filetype was changed
    if (!(opt_flags & OPT_MODELINE) || value_changed) {
      do_filetype_autocmd(curbuf, value_changed);
    }
  } else if (varp == &curwin->w_s->b_p_spl) {
    do_spelllang_source(curwin);
  }

  // In case 'ruler' or 'showcmd' or 'columns' or 'ls' changed.
  comp_col();

  if (varp == &p_mouse) {
    setmouse();  // in case 'mouse' changed
  } else if ((varp == &p_flp || varp == &(curbuf->b_p_flp)) && curwin->w_briopt_list) {
    // Changing Formatlistpattern when briopt includes the list setting:
    // redraw
    redraw_all_later(UPD_NOT_VALID);
  } else if (varp == &p_wbr || varp == &(curwin->w_p_wbr)) {
    // add / remove window bars for 'winbar'
    set_winbar(true);
  }

  if (curwin->w_curswant != MAXCOL
      && (opt->flags & (kOptFlagCurswant | kOptFlagRedrAll)) != 0
      && (opt->flags & kOptFlagHLOnly) == 0) {
    curwin->w_set_curswant = true;
  }

  check_redraw(opt->flags);

  if (errmsg == NULL) {
    opt->flags |= kOptFlagWasSet;

    uint32_t *flagsp = insecure_flag(curwin, opt_idx, opt_flags);
    uint32_t *flagsp_local = scope_both ? insecure_flag(curwin, opt_idx, OPT_LOCAL) : NULL;
    // When an option is set in the sandbox, from a modeline or in secure mode set the
    // kOptFlagInsecure flag.  Otherwise, if a new value is stored reset the flag.
    if (!value_checked && (secure || sandbox != 0 || (opt_flags & OPT_MODELINE))) {
      *flagsp |= kOptFlagInsecure;
      if (flagsp_local != NULL) {
        *flagsp_local |= kOptFlagInsecure;
      }
    } else if (value_replaced) {
      *flagsp &= ~(unsigned)kOptFlagInsecure;
      if (flagsp_local != NULL) {
        *flagsp_local &= ~(unsigned)kOptFlagInsecure;
      }
    }
  }

  return errmsg;
}

/// Validate the new value for an option.
///
/// @param  opt_idx         Index in options[] table. Must not be kOptInvalid.
/// @param  newval[in,out]  New option value. Might be modified.
static const char *validate_option_value(const OptIndex opt_idx, OptVal *newval, int opt_flags,
                                         char *errbuf, size_t errbuflen)
{
  const char *errmsg = NULL;
  vimoption_T *opt = &options[opt_idx];

  // Always allow unsetting local value of global-local option.
  if (option_is_global_local(opt_idx) && (opt_flags & OPT_LOCAL)
      && rs_optval_equal(*newval, get_option_unset_value(opt_idx)) != 0) {
    return NULL;
  }

  if (newval->type == kOptValTypeNil) {
    // Don't try to unset local value if scope is global.
    // TODO(famiu): Change this to forbid changing all non-local scopes when the API scope bug is
    // fixed.
    if (opt_flags == OPT_GLOBAL) {
      errmsg = _("Cannot unset global option value");
    } else {
      *newval = rs_optval_copy(get_option_unset_value(opt_idx));
    }
  } else if (!option_has_type(opt_idx, newval->type)) {
    char *rep = optval_to_cstr(*newval);
    const char *type_str = optval_type_get_name(opt->type);
    snprintf(errbuf, IOSIZE, _("Invalid value for option '%s': expected %s, got %s %s"),
             opt->fullname, type_str, optval_type_get_name(newval->type), rep);
    xfree(rep);
    errmsg = errbuf;
  } else if (newval->type == kOptValTypeNumber) {
    // Validate and bound check num option values.
    errmsg = rs_validate_num_option(opt_idx, &newval->data.number, errbuf, errbuflen);
  }

  return errmsg;
}

/// Set the value of an option using an OptVal.
///
/// @param       opt_idx         Index in options[] table. Must not be kOptInvalid.
/// @param       value           New option value. Might get freed.
/// @param       opt_flags       Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
/// @param       set_sid         Script ID. Special values:
///                                0: Use current script ID.
///                                SID_NONE: Don't set script ID.
/// @param       direct          Don't process side-effects.
/// @param       value_replaced  Value was replaced completely.
/// @param[out]  errbuf          Buffer for error message.
/// @param       errbuflen       Length of error buffer.
///
/// @return  NULL on success, an untranslated error message on error.
static const char *set_option(const OptIndex opt_idx, OptVal value, int opt_flags, scid_T set_sid,
                              const bool direct, const bool value_replaced, char *errbuf,
                              size_t errbuflen)
{
  assert(opt_idx != kOptInvalid);

  const char *errmsg = NULL;

  if (!direct) {
    errmsg = validate_option_value(opt_idx, &value, opt_flags, errbuf, errbuflen);

    if (errmsg != NULL) {
      rs_optval_free(value);
      return errmsg;
    }
  }

  vimoption_T *opt = &options[opt_idx];
  const bool scope_local = opt_flags & OPT_LOCAL;
  const bool scope_global = opt_flags & OPT_GLOBAL;
  const bool scope_both = !scope_local && !scope_global;
  // Whether local value of global-local option is unset.
  // NOTE: When this is true, it also implies that the option is global-local.
  const bool is_opt_local_unset = is_option_local_value_unset(opt_idx);

  // When using ":set opt=val" for a global option with a local value the local value will be reset,
  // use the global value in that case.
  void *varp
    = scope_both && option_is_global_local(opt_idx) ? opt->var : get_varp_scope(opt, opt_flags);
  void *varp_local = get_varp_scope(opt, OPT_LOCAL);
  void *varp_global = get_varp_scope(opt, OPT_GLOBAL);

  OptVal old_value = optval_from_varp(opt_idx, varp);
  OptVal old_global_value = optval_from_varp(opt_idx, varp_global);
  // If local value of global-local option is unset, use global value as local value.
  OptVal old_local_value = is_opt_local_unset
                           ? old_global_value
                           : optval_from_varp(opt_idx, varp_local);
  // Value that's actually being used.
  // For local scope of a global-local option, it's equal to the global value if the local value is
  // unset. In every other case, it is the same as old_value.
  // This value is used instead of old_value when triggering the OptionSet autocommand.
  OptVal used_old_value = (scope_local && is_opt_local_unset)
                          ? optval_from_varp(opt_idx, get_varp(opt))
                          : old_value;

  // Save the old values and the new value in case they get changed.
  OptVal saved_used_value = rs_optval_copy(used_old_value);
  OptVal saved_old_global_value = rs_optval_copy(old_global_value);
  OptVal saved_old_local_value = rs_optval_copy(old_local_value);
  // New value (and varp) may become invalid if the buffer is closed by autocommands.
  OptVal saved_new_value = rs_optval_copy(value);

  uint32_t *p = insecure_flag(curwin, opt_idx, opt_flags);
  const int secure_saved = secure;

  // When an option is set in the sandbox, from a modeline or in secure mode, then deal with side
  // effects in secure mode. Also when the value was set with the kOptFlagInsecure flag and is not
  // completely replaced.
  if ((opt_flags & OPT_MODELINE) || sandbox != 0 || (!value_replaced && (*p & kOptFlagInsecure))) {
    secure = 1;
  }

  // Set option through its variable pointer.
  set_option_varp(opt_idx, varp, value, false);
  // Process any side effects.
  errmsg = did_set_option(opt_idx, varp, old_value, value, opt_flags, set_sid, direct,
                          value_replaced, errbuf, errbuflen);

  secure = secure_saved;

  if (errmsg == NULL && !direct) {
    if (!starting) {
      apply_optionset_autocmd(opt_idx, opt_flags, saved_used_value, saved_old_global_value,
                              saved_old_local_value, saved_new_value, errmsg);
    }
    if (opt->flags & kOptFlagUIOption) {
      ui_call_option_set(cstr_as_string(opt->fullname), optval_as_object(saved_new_value));
    }
  }

  // Free copied values as they are not needed anymore
  rs_optval_free(saved_used_value);
  rs_optval_free(saved_old_local_value);
  rs_optval_free(saved_old_global_value);
  rs_optval_free(saved_new_value);

  return errmsg;
}

/// Set option value directly, without processing any side effects.
///
/// @param  opt_idx    Option index in options[] table.
/// @param  value      Option value.
/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
/// @param  set_sid    Script ID. Special values:
///                      0: Use current script ID.
///                      SID_NONE: Don't set script ID.
void set_option_direct(OptIndex opt_idx, OptVal value, int opt_flags, scid_T set_sid)
{
  static char errbuf[IOSIZE];

  if (is_option_hidden(opt_idx)) {
    return;
  }

  const char *errmsg = set_option(opt_idx, rs_optval_copy(value), opt_flags, set_sid, true, true,
                                  errbuf, sizeof(errbuf));
  assert(errmsg == NULL);
  (void)errmsg;  // ignore unused warning
}

/// Set option value directly for buffer / window, without processing any side effects.
///
/// @param      opt_idx    Option index in options[] table.
/// @param      value      Option value.
/// @param      opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
/// @param      set_sid    Script ID. Special values:
///                          0: Use current script ID.
///                          SID_NONE: Don't set script ID.
/// @param      scope      Option scope. See OptScope in option.h.
/// @param[in]  from       Target buffer/window.
void set_option_direct_for(OptIndex opt_idx, OptVal value, int opt_flags, scid_T set_sid,
                           OptScope scope, void *const from)
{
  buf_T *save_curbuf = curbuf;
  win_T *save_curwin = curwin;

  // Don't use switch_option_context(), as that calls aucmd_prepbuf(), which may have unintended
  // side-effects when setting an option directly. Just change the values of curbuf and curwin if
  // needed, no need to properly switch the window / buffer.
  switch (scope) {
  case kOptScopeGlobal:
    break;
  case kOptScopeWin:
    curwin = (win_T *)from;
    curbuf = curwin->w_buffer;
    break;
  case kOptScopeBuf:
    curbuf = (buf_T *)from;
    break;
  }

  set_option_direct(opt_idx, value, opt_flags, set_sid);

  curwin = save_curwin;
  curbuf = save_curbuf;
}

/// Set the value of an option.
///
/// @param      opt_idx    Index in options[] table. Must not be kOptInvalid.
/// @param[in]  value      Option value. If NIL_OPTVAL, the option value is cleared.
/// @param[in]  opt_flags  Flags: OPT_LOCAL, OPT_GLOBAL, or 0 (both).
///
/// @return  NULL on success, an untranslated error message on error.
const char *set_option_value(const OptIndex opt_idx, const OptVal value, int opt_flags)
{
  assert(opt_idx != kOptInvalid);

  static char errbuf[IOSIZE];
  uint32_t flags = options[opt_idx].flags;

  // Disallow changing some options in the sandbox
  if (sandbox > 0 && (flags & kOptFlagSecure)) {
    return _(e_sandbox);
  }

  return set_option(opt_idx, rs_optval_copy(value), opt_flags, 0, false, true, errbuf, sizeof(errbuf));
}

/// Unset the local value of a global-local option.
///
/// @param      opt_idx    Index in options[] table. Must not be kOptInvalid.
///
/// @return  NULL on success, an untranslated error message on error.
static inline const char *unset_option_local_value(const OptIndex opt_idx)
{
  assert(option_is_global_local(opt_idx));
  return set_option_value(opt_idx, get_option_unset_value(opt_idx), OPT_LOCAL);
}

/// Set the value of an option. Supports TTY options, unlike set_option_value().
///
/// @param      name       Option name. Used for error messages and for setting TTY options.
/// @param      opt_idx    Option indx in options[] table. If kOptInvalid, `name` is used to
///                        check if the option is a TTY option, and an error is shown if it's not.
///                        If the option is a TTY option, the function fails silently.
/// @param      value      Option value. If NIL_OPTVAL, the option value is cleared.
/// @param[in]  opt_flags  Flags: OPT_LOCAL, OPT_GLOBAL, or 0 (both).
///
/// @return  NULL on success, an untranslated error message on error.
const char *set_option_value_handle_tty(const char *name, OptIndex opt_idx, const OptVal value,
                                        int opt_flags)
  FUNC_ATTR_NONNULL_ARG(1)
{
  static char errbuf[IOSIZE];

  if (opt_idx == kOptInvalid) {
    if (rs_is_tty_option(name)) {
      return NULL;  // Fail silently; many old vimrcs set t_xx options.
    }

    snprintf(errbuf, sizeof(errbuf), _(e_unknown_option2), name);
    return errbuf;
  }

  return set_option_value(opt_idx, value, opt_flags);
}

/// Call set_option_value() and when an error is returned, report it.
///
/// @param  opt_idx    Option index in options[] table.
/// @param  value      Option value. If NIL_OPTVAL, the option value is cleared.
/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
void set_option_value_give_err(const OptIndex opt_idx, OptVal value, int opt_flags)
{
  const char *errmsg = set_option_value(opt_idx, value, opt_flags);

  if (errmsg != NULL) {
    emsg(_(errmsg));
  }
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

/// if 'all' == false: show changed options
/// if 'all' == true: show all normal options
///
/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
extern void rs_showoptions(int all, int opt_flags);
static void showoptions(bool all, int opt_flags)
{
  rs_showoptions(all ? 1 : 0, opt_flags);
}

/// Return true if option "p" has its default value.
static int optval_default(OptIndex opt_idx, void *varp)
{
  return rs_optval_default(opt_idx, varp);
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

/// showoneopt: show the value of one option
/// must not be called with a hidden option!
///
/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
extern void rs_showoneopt(int opt_idx, int opt_flags);
static void showoneopt(vimoption_T *opt, int opt_flags)
{
  rs_showoneopt((int)get_opt_idx(opt), opt_flags);
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
/// (fresh value = value used for a new buffer or window for a local option).
///
/// Return FAIL on error, OK otherwise.
extern int rs_makeset(FILE *fd, int opt_flags, int local_only);
int makeset(FILE *fd, int opt_flags, int local_only)
{
  return rs_makeset(fd, opt_flags, local_only);
}

/// Generate set commands for the local fold options only.  Used when
/// 'sessionoptions' or 'viewoptions' contains "folds" but not "options".
int makefoldset(FILE *fd)
{
  return rs_makefoldset(fd);
}

/// Print the ":set" command to set a single option to file.
///
/// @param  fd       File descriptor.
/// @param  cmd      Command name.
/// @param  opt_idx  Option index in options[] table.
/// @param  varp     Pointer to option variable.
///
/// @return FAIL on error, OK otherwise.
static int put_set(FILE *fd, char *cmd, OptIndex opt_idx, void *varp)
{
  return rs_put_set(fd, cmd, opt_idx, varp);
}

void *get_varp_scope_from(vimoption_T *p, int opt_flags, buf_T *buf, win_T *win)
{
  return rs_get_varp_scope_from(p, opt_flags, buf, win);
}

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

void *get_varp_from(vimoption_T *p, buf_T *buf, win_T *win)
{
  return rs_get_varp_from(p, buf, win);
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

/// Get the value of 'equalprg', either the buffer-local one or the global one.
char *get_equalprg(void)
{
  return (char *)rs_get_equalprg();
}

/// Get the value of 'findfunc', either the buffer-local one or the global one.
char *get_findfunc(void)
{
  return (char *)rs_get_findfunc();
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

extern void rs_copy_winopt(winopt_T *from, winopt_T *to);
extern void rs_clear_winopt(winopt_T *wop);
extern void rs_check_winopt(winopt_T *wop);
extern void rs_didset_window_options(win_T *wp, int valid_cursor);

/// Copy the options from one winopt_T to another.
/// Doesn't free the old option values in "to", use clear_winopt() for that.
/// The 'scroll' option is not copied, because it depends on the window height.
/// The 'previewwindow' option is reset, there can be only one preview window.
void copy_winopt(winopt_T *from, winopt_T *to)
{
  rs_copy_winopt(from, to);
}

/// Check string options in a window for a NULL value.
static void check_win_options(win_T *win)
{
  rs_check_winopt(&win->w_onebuf_opt);
  rs_check_winopt(&win->w_allbuf_opt);
}

/// Check for NULL pointers in a winopt_T and replace them with empty_string_option.
static void check_winopt(winopt_T *wop)
{
  rs_check_winopt(wop);
}

/// Free the allocated memory inside a winopt_T.
void clear_winopt(winopt_T *wop)
{
  rs_clear_winopt(wop);
}

void didset_window_options(win_T *wp, bool valid_cursor)
{
  rs_didset_window_options(wp, (int)valid_cursor);
}

#define COPY_OPT_SCTX(buf, bv) buf->b_p_script_ctx[bv] = options[buf_opt_idx[bv]].script_ctx

/// Copy global option values to local options for one buffer.
/// Used when creating a new buffer and sometimes when entering a buffer.
/// flags:
/// BCO_ENTER    We will enter the buffer "buf".
/// BCO_ALWAYS   Always copy the options, but only set b_p_initialized when
///      appropriate.
/// BCO_NOHELP   Don't copy the values to a help buffer.
void buf_copy_options(buf_T *buf, int flags)
{
  bool should_copy = true;
  char *save_p_isk = NULL;           // init for GCC
  bool did_isk = false;

  // Skip this when the option defaults have not been set yet.  Happens when
  // main() allocates the first buffer.
  if (p_cpo != NULL) {
    //
    // Always copy when entering and 'cpo' contains 'S'.
    // Don't copy when already initialized.
    // Don't copy when 'cpo' contains 's' and not entering.
    //    'S'      BCO_ENTER  initialized  's'  should_copy
    //    yes        yes          X         X      true
    //    yes        no          yes        X      false
    //    no          X          yes        X      false
    //     X         no          no        yes     false
    //     X         no          no        no      true
    //    no         yes         no         X      true
    ///
    if ((vim_strchr(p_cpo, CPO_BUFOPTGLOB) == NULL || !(flags & BCO_ENTER))
        && (buf->b_p_initialized
            || (!(flags & BCO_ENTER)
                && vim_strchr(p_cpo, CPO_BUFOPT) != NULL))) {
      should_copy = false;
    }

    if (should_copy || (flags & BCO_ALWAYS)) {
      CLEAR_FIELD(buf->b_p_script_ctx);
      // Don't copy the options specific to a help buffer when
      // BCO_NOHELP is given or the options were initialized already
      // (jumping back to a help file with CTRL-T or CTRL-O)
      bool dont_do_help = ((flags & BCO_NOHELP) && buf->b_help) || buf->b_p_initialized;
      if (dont_do_help) {               // don't free b_p_isk
        save_p_isk = buf->b_p_isk;
        buf->b_p_isk = NULL;
      }
      // Always free the allocated strings.  If not already initialized,
      // reset 'readonly' and copy 'fileformat'.
      if (!buf->b_p_initialized) {
        free_buf_options(buf, true);
        buf->b_p_ro = false;                    // don't copy readonly
        buf->b_p_fenc = xstrdup(p_fenc);
        switch (*p_ffs) {
        case 'm':
          buf->b_p_ff = xstrdup("mac");
          break;
        case 'd':
          buf->b_p_ff = xstrdup("dos");
          break;
        case 'u':
          buf->b_p_ff = xstrdup("unix");
          break;
        default:
          buf->b_p_ff = xstrdup(p_ff);
          break;
        }
        buf->b_p_bh = empty_string_option;
        buf->b_p_bt = empty_string_option;
      } else {
        free_buf_options(buf, false);
      }

      buf->b_p_ai = p_ai;
      COPY_OPT_SCTX(buf, kBufOptAutoindent);
      buf->b_p_ai_nopaste = p_ai_nopaste;
      buf->b_p_sw = p_sw;
      COPY_OPT_SCTX(buf, kBufOptShiftwidth);
      buf->b_p_scbk = p_scbk;
      COPY_OPT_SCTX(buf, kBufOptScrollback);
      buf->b_p_tw = p_tw;
      COPY_OPT_SCTX(buf, kBufOptTextwidth);
      buf->b_p_tw_nopaste = p_tw_nopaste;
      buf->b_p_tw_nobin = p_tw_nobin;
      buf->b_p_wm = p_wm;
      COPY_OPT_SCTX(buf, kBufOptWrapmargin);
      buf->b_p_wm_nopaste = p_wm_nopaste;
      buf->b_p_wm_nobin = p_wm_nobin;
      buf->b_p_bin = p_bin;
      COPY_OPT_SCTX(buf, kBufOptBinary);
      buf->b_p_bomb = p_bomb;
      COPY_OPT_SCTX(buf, kBufOptBomb);
      buf->b_p_et = p_et;
      COPY_OPT_SCTX(buf, kBufOptExpandtab);
      buf->b_p_fixeol = p_fixeol;
      COPY_OPT_SCTX(buf, kBufOptFixendofline);
      buf->b_p_et_nobin = p_et_nobin;
      buf->b_p_et_nopaste = p_et_nopaste;
      buf->b_p_ml = p_ml;
      COPY_OPT_SCTX(buf, kBufOptModeline);
      buf->b_p_ml_nobin = p_ml_nobin;
      buf->b_p_inf = p_inf;
      COPY_OPT_SCTX(buf, kBufOptInfercase);
      if (cmdmod.cmod_flags & CMOD_NOSWAPFILE) {
        buf->b_p_swf = false;
      } else {
        buf->b_p_swf = p_swf;
        COPY_OPT_SCTX(buf, kBufOptSwapfile);
      }
      buf->b_p_cpt = xstrdup(p_cpt);
      COPY_OPT_SCTX(buf, kBufOptComplete);
      set_buflocal_cpt_callbacks(buf);
#ifdef BACKSLASH_IN_FILENAME
      buf->b_p_csl = xstrdup(p_csl);
      COPY_OPT_SCTX(buf, kBufOptCompleteslash);
#endif
      buf->b_p_cfu = xstrdup(p_cfu);
      COPY_OPT_SCTX(buf, kBufOptCompletefunc);
      set_buflocal_cfu_callback(buf);
      buf->b_p_ofu = xstrdup(p_ofu);
      COPY_OPT_SCTX(buf, kBufOptOmnifunc);
      set_buflocal_ofu_callback(buf);
      buf->b_p_tfu = xstrdup(p_tfu);
      COPY_OPT_SCTX(buf, kBufOptTagfunc);
      rs_set_buflocal_tfu_callback(buf);
      buf->b_p_sts = p_sts;
      COPY_OPT_SCTX(buf, kBufOptSofttabstop);
      buf->b_p_sts_nopaste = p_sts_nopaste;
      buf->b_p_vsts = xstrdup(p_vsts);
      COPY_OPT_SCTX(buf, kBufOptVarsofttabstop);
      if (p_vsts && p_vsts != empty_string_option) {
        tabstop_set(p_vsts, &buf->b_p_vsts_array);
      } else {
        buf->b_p_vsts_array = NULL;
      }
      buf->b_p_vsts_nopaste = p_vsts_nopaste ? xstrdup(p_vsts_nopaste) : NULL;
      buf->b_p_com = xstrdup(p_com);
      COPY_OPT_SCTX(buf, kBufOptComments);
      buf->b_p_cms = xstrdup(p_cms);
      COPY_OPT_SCTX(buf, kBufOptCommentstring);
      buf->b_p_fo = xstrdup(p_fo);
      COPY_OPT_SCTX(buf, kBufOptFormatoptions);
      buf->b_p_flp = xstrdup(p_flp);
      COPY_OPT_SCTX(buf, kBufOptFormatlistpat);
      buf->b_p_nf = xstrdup(p_nf);
      COPY_OPT_SCTX(buf, kBufOptNrformats);
      buf->b_p_mps = xstrdup(p_mps);
      COPY_OPT_SCTX(buf, kBufOptMatchpairs);
      buf->b_p_si = p_si;
      COPY_OPT_SCTX(buf, kBufOptSmartindent);
      buf->b_p_channel = 0;
      buf->b_p_ci = p_ci;

      COPY_OPT_SCTX(buf, kBufOptCopyindent);
      buf->b_p_cin = p_cin;
      COPY_OPT_SCTX(buf, kBufOptCindent);
      buf->b_p_cink = xstrdup(p_cink);
      COPY_OPT_SCTX(buf, kBufOptCinkeys);
      buf->b_p_cino = xstrdup(p_cino);
      COPY_OPT_SCTX(buf, kBufOptCinoptions);
      buf->b_p_cinsd = xstrdup(p_cinsd);
      COPY_OPT_SCTX(buf, kBufOptCinscopedecls);
      buf->b_p_lop = xstrdup(p_lop);
      COPY_OPT_SCTX(buf, kBufOptLispoptions);

      // Don't copy 'filetype', it must be detected
      buf->b_p_ft = empty_string_option;
      buf->b_p_pi = p_pi;
      COPY_OPT_SCTX(buf, kBufOptPreserveindent);
      buf->b_p_cinw = xstrdup(p_cinw);
      COPY_OPT_SCTX(buf, kBufOptCinwords);
      buf->b_p_lisp = p_lisp;
      COPY_OPT_SCTX(buf, kBufOptLisp);
      // Don't copy 'syntax', it must be set
      buf->b_p_syn = empty_string_option;
      buf->b_p_smc = p_smc;
      COPY_OPT_SCTX(buf, kBufOptSynmaxcol);
      buf->b_s.b_syn_isk = empty_string_option;
      buf->b_s.b_p_spc = xstrdup(p_spc);
      COPY_OPT_SCTX(buf, kBufOptSpellcapcheck);
      compile_cap_prog(&buf->b_s);
      buf->b_s.b_p_spf = xstrdup(p_spf);
      COPY_OPT_SCTX(buf, kBufOptSpellfile);
      buf->b_s.b_p_spl = xstrdup(p_spl);
      COPY_OPT_SCTX(buf, kBufOptSpelllang);
      buf->b_s.b_p_spo = xstrdup(p_spo);
      COPY_OPT_SCTX(buf, kBufOptSpelloptions);
      buf->b_s.b_p_spo_flags = spo_flags;
      buf->b_p_inde = xstrdup(p_inde);
      COPY_OPT_SCTX(buf, kBufOptIndentexpr);
      buf->b_p_indk = xstrdup(p_indk);
      COPY_OPT_SCTX(buf, kBufOptIndentkeys);
      buf->b_p_fp = empty_string_option;
      buf->b_p_fex = xstrdup(p_fex);
      COPY_OPT_SCTX(buf, kBufOptFormatexpr);
      buf->b_p_sua = xstrdup(p_sua);
      COPY_OPT_SCTX(buf, kBufOptSuffixesadd);
      buf->b_p_keymap = xstrdup(p_keymap);
      COPY_OPT_SCTX(buf, kBufOptKeymap);
      buf->b_kmap_state |= KEYMAP_INIT;
      // This isn't really an option, but copying the langmap and IME
      // state from the current buffer is better than resetting it.
      buf->b_p_iminsert = p_iminsert;
      COPY_OPT_SCTX(buf, kBufOptIminsert);
      buf->b_p_imsearch = p_imsearch;
      COPY_OPT_SCTX(buf, kBufOptImsearch);

      // options that are normally global but also have a local value
      // are not copied, start using the global value
      buf->b_p_ac = -1;
      buf->b_p_ar = -1;
      buf->b_p_ul = NO_LOCAL_UNDOLEVEL;
      buf->b_p_bkc = empty_string_option;
      buf->b_bkc_flags = 0;
      buf->b_p_gefm = empty_string_option;
      buf->b_p_gp = empty_string_option;
      buf->b_p_mp = empty_string_option;
      buf->b_p_efm = empty_string_option;
      buf->b_p_ep = empty_string_option;
      buf->b_p_ffu = empty_string_option;
      buf->b_p_kp = empty_string_option;
      buf->b_p_path = empty_string_option;
      buf->b_p_tags = empty_string_option;
      buf->b_p_tc = empty_string_option;
      buf->b_tc_flags = 0;
      buf->b_p_def = empty_string_option;
      buf->b_p_inc = empty_string_option;
      buf->b_p_inex = xstrdup(p_inex);
      COPY_OPT_SCTX(buf, kBufOptIncludeexpr);
      buf->b_p_cot = empty_string_option;
      buf->b_cot_flags = 0;
      buf->b_p_dict = empty_string_option;
      buf->b_p_dia = empty_string_option;
      buf->b_p_tsr = empty_string_option;
      buf->b_p_tsrfu = empty_string_option;
      buf->b_p_qe = xstrdup(p_qe);
      COPY_OPT_SCTX(buf, kBufOptQuoteescape);
      buf->b_p_udf = p_udf;
      COPY_OPT_SCTX(buf, kBufOptUndofile);
      buf->b_p_lw = empty_string_option;
      buf->b_p_menc = empty_string_option;

      // Don't copy the options set by ex_help(), use the saved values,
      // when going from a help buffer to a non-help buffer.
      // Don't touch these at all when BCO_NOHELP is used and going from
      // or to a help buffer.
      if (dont_do_help) {
        buf->b_p_isk = save_p_isk;
        if (p_vts && *p_vts != NUL && !buf->b_p_vts_array) {
          tabstop_set(p_vts, &buf->b_p_vts_array);
        } else {
          buf->b_p_vts_array = NULL;
        }
      } else {
        buf->b_p_isk = xstrdup(p_isk);
        COPY_OPT_SCTX(buf, kBufOptIskeyword);
        did_isk = true;
        buf->b_p_ts = p_ts;
        COPY_OPT_SCTX(buf, kBufOptTabstop);
        buf->b_p_vts = xstrdup(p_vts);
        COPY_OPT_SCTX(buf, kBufOptVartabstop);
        if (p_vts && *p_vts != NUL && !buf->b_p_vts_array) {
          tabstop_set(p_vts, &buf->b_p_vts_array);
        } else {
          buf->b_p_vts_array = NULL;
        }
        buf->b_help = false;
        if (buf->b_p_bt[0] == 'h') {
          clear_string_option(&buf->b_p_bt);
        }
        buf->b_p_ma = p_ma;
        COPY_OPT_SCTX(buf, kBufOptModifiable);
      }
    }

    // When the options should be copied (ignoring BCO_ALWAYS), set the
    // flag that indicates that the options have been initialized.
    if (should_copy) {
      buf->b_p_initialized = true;
    }
  }

  check_buf_options(buf);           // make sure we don't have NULLs
  if (did_isk) {
    buf_init_chartab(buf, false);
  }
}

/// Reset the 'modifiable' option and its default value.
void reset_modifiable(void)
{
  rs_reset_modifiable();
}

/// Set the global value for 'iminsert' to the local value.
void set_iminsert_global(buf_T *buf)
{
  rs_set_iminsert_global(buf);
}

/// Set the global value for 'imsearch' to the local value.
void set_imsearch_global(buf_T *buf)
{
  rs_set_imsearch_global(buf);
}

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

// expand_option static variable get/set accessors
OptIndex nvim_get_expand_option_idx(void) { return expand_option_idx; }
void nvim_set_expand_option_idx(OptIndex val) { expand_option_idx = val; }
int nvim_get_expand_option_start_col(void) { return expand_option_start_col; }
void nvim_set_expand_option_start_col(int val) { expand_option_start_col = val; }
int nvim_get_expand_option_flags(void) { return expand_option_flags; }
void nvim_set_expand_option_flags(int val) { expand_option_flags = val; }
int nvim_get_expand_option_append(void) { return (int)expand_option_append; }
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

/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
void set_context_in_set_cmd(expand_T *xp, char *arg, int opt_flags)
{
  rs_set_context_in_set_cmd(xp, arg, opt_flags);
}

extern int rs_expand_option_settings(expand_T *xp, regmatch_T *regmatch, char *fuzzystr,
                                      int *numMatches, char ***matches, int can_fuzzy);
int ExpandSettings(expand_T *xp, regmatch_T *regmatch, char *fuzzystr, int *numMatches,
                   char ***matches, const bool can_fuzzy)
{
  return rs_expand_option_settings(xp, regmatch, fuzzystr, numMatches, matches, can_fuzzy ? 1 : 0);
}

/// Escape an option value that can be used on the command-line with :set.
/// Caller needs to free the returned string, unless NULL is returned.
extern char *rs_escape_option_str_cmdline(const char *var);
static char *escape_option_str_cmdline(char *var)
{
  return rs_escape_option_str_cmdline(var);
}

/// Non-static wrapper for escape_option_str_cmdline (for Rust FFI).
char *nvim_escape_option_str_cmdline(char *var)
{
  return escape_option_str_cmdline(var);
}

/// curbuf/curwin accessors for option expansion Rust code.
buf_T *nvim_opt_get_curbuf(void) { return curbuf; }
win_T *nvim_opt_get_curwin(void) { return curwin; }

/// Expansion handler for :set= when we just want to fill in with the existing value.
extern int rs_expand_old_setting(int *numMatches, char ***matches);
int ExpandOldSetting(int *numMatches, char ***matches)
{
  return rs_expand_old_setting(numMatches, matches);
}

/// Expansion handler for :set=/:set+= when the option has a custom expansion handler.
extern int rs_expand_string_setting(expand_T *xp, regmatch_T *regmatch, int *numMatches,
                                    char ***matches);
int ExpandStringSetting(expand_T *xp, regmatch_T *regmatch, int *numMatches, char ***matches)
{
  return rs_expand_string_setting(xp, regmatch, numMatches, matches);
}


/// Get the value for the numeric or string option///opp in a nice format into
/// NameBuff[].  Must not be called with a hidden option!
///
/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
///
/// TODO(famiu): Replace this with optval_to_cstr() if possible.
static void option_value2string(vimoption_T *opt, int opt_flags)
{
  rs_option_value2string((int)get_opt_idx(opt), opt_flags);
}

/// Return true if "varp" points to 'wildchar' or 'wildcharm' and it can be
/// printed as a keyname.
/// "*wcp" is set to the value of the option if it's 'wildchar' or 'wildcharm'.
static int wc_use_keyname(const void *varp, OptInt *wcp)
{
  return rs_wc_use_keyname(varp, wcp);
}

/// @returns true if "x" is present in 'shortmess' option, or
/// 'shortmess' contains 'a' and "x" is present in SHM_ALL_ABBREVIATIONS.
bool shortmess(int x)
{
  return rs_shortmess(x) != 0;
}

/// vimrc_found() - Called when a vimrc or "VIMINIT" has been found.
///
/// Set the values for options that didn't get set yet to the defaults.
/// When "fname" is not NULL, use it to set $"envname" when it wasn't set yet.
void vimrc_found(char *fname, char *envname)
{
  rs_vimrc_found(fname, envname);
}

/// Check whether global option has been set.
///
/// @param[in]  name  Option name.
///
/// @return True if option was set.
bool option_was_set(OptIndex opt_idx)
{
  assert(opt_idx != kOptInvalid);
  return options[opt_idx].flags & kOptFlagWasSet;
}

/// Reset the flag indicating option "name" was set.
///
/// @param[in]  name  Option name.
void reset_option_was_set(OptIndex opt_idx)
{
  assert(opt_idx != kOptInvalid);
  options[opt_idx].flags &= ~(unsigned)kOptFlagWasSet;
}

/// fill_culopt_flags() -- called when 'culopt' changes value
int fill_culopt_flags(char *val, win_T *wp)
{
  return rs_fill_culopt_flags(val, wp);
}



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

/// Check if backspacing over something is allowed.
/// @param  what  BS_INDENT, BS_EOL, BS_START, or BS_NOSTOP
bool can_bs(int what)
{
  return rs_can_bs(what) != 0;
}

/// Get the local or global value of 'backupcopy' flags.
///
/// @param buf The buffer.
unsigned get_bkc_flags(buf_T *buf)
{
  return rs_get_bkc_flags(buf);
}

/// Get the local or global value of 'formatlistpat'.
///
/// @param buf The buffer.
char *get_flp_value(buf_T *buf)
{
  return rs_get_flp_value(buf);
}

/// Get the local or global value of 'virtualedit' flags.
unsigned get_ve_flags(win_T *wp)
{
  return rs_get_ve_flags(wp);
}

extern int rs_get_fileformat_force(const buf_T *buf, const exarg_T *eap);

/// Like get_fileformat(), but override 'fileformat' with "p" for "++opt=val"
/// argument.
///
/// @param eap  can be NULL!
int get_fileformat_force(const buf_T *buf, const exarg_T *eap)
  FUNC_ATTR_NONNULL_ARG(1)
{
  return rs_get_fileformat_force(buf, eap);
}

/// Set the current end-of-line type to EOL_UNIX, EOL_MAC, or EOL_DOS.
///
/// Sets 'fileformat'.
///
/// @param eol_style End-of-line style.
/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
void set_fileformat(int eol_style, int opt_flags)
{
  rs_set_fileformat(eol_style, opt_flags);
}

/// Isolate one part of a string option separated by `sep_chars`.
///
/// @param[in,out]  option    advanced to the next part
/// @param[in,out]  buf       copy of the isolated part
/// @param[in]      maxlen    length of `buf`
/// @param[in]      sep_chars chars that separate the option parts
///
/// @return length of `*option`
size_t copy_option_part(char **option, char *buf, size_t maxlen, char *sep_chars)
{
  return rs_copy_option_part(option, buf, maxlen, sep_chars);
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

  PUT_C(dict, "type", CSTR_AS_OBJ(optval_type_get_name(option_get_type(get_opt_idx(opt)))));
  PUT_C(dict, "default", optval_as_object(opt->def_val));
  PUT_C(dict, "allows_duplicates", BOOLEAN_OBJ(!(opt->flags & kOptFlagNoDup)));

  return dict;
}

// =============================================================================
// Wrapper function implementations for Rust setcmd module
// =============================================================================

void nvim_set_options_default(int opt_flags) { set_options_default(opt_flags); }
void nvim_didset_options(void) { didset_options(); }
void nvim_didset_options2(void) { didset_options2(); }
void nvim_showoptions(int all, int opt_flags) { showoptions(all != 0, opt_flags); }
void nvim_showoneopt(vimoption_T *opt, int opt_flags) { showoneopt(opt, opt_flags); }

int nvim_validate_opt_idx(win_T *win, OptIndex opt_idx, int opt_flags, uint32_t flags,
                          int prefix, const char **errmsg)
{
  return validate_opt_idx(win, opt_idx, opt_flags, flags, (set_prefix_T)prefix, errmsg);
}

OptVal nvim_get_option_newval(OptIndex opt_idx, int opt_flags, int prefix, char **argp,
                              char nextchar, int op, uint32_t flags, void *varp,
                              char *errbuf, size_t errbuflen, const char **errmsg)
{
  return get_option_newval(opt_idx, opt_flags, (set_prefix_T)prefix, argp, nextchar,
                           (set_op_T)op, flags, varp, errbuf, errbuflen, errmsg);
}

const char *nvim_rs_set_option(OptIndex opt_idx, OptVal value, int opt_flags,
                               int set_sid, int direct, int value_replaced,
                               char *errbuf, size_t errbuflen)
{
  if (opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) {
    return N_("E518: Unknown option");
  }
  return set_option(opt_idx, value, opt_flags, set_sid, direct != 0, value_replaced != 0, errbuf,
                    errbuflen);
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

  option_value2string(&options[opt_idx], opt_flags);
  char *var = NameBuff;
  char *buf = escape_option_str_cmdline(var);
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



