#define IN_OPTION_C
#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "auto/config.h"
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
#include "nvim/cmdexpand.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/decoration_defs.h"
#include "nvim/decoration_provider.h"
#include "nvim/drawscreen.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/eval/window.h"
#include "nvim/ex_session.h"
#include "nvim/fuzzy.h"
#include "nvim/globals.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/indent.h"
#include "nvim/keycodes.h"
#include "nvim/log.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/popupmenu.h"
#include "nvim/regexp.h"
#include "nvim/spell.h"
#include "nvim/strings.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/window.h"

_Static_assert(sizeof(vimoption_T) == 160,
               "sizeof(vimoption_T) changed - update Rust VIMOPTION_SIZE in option/src/accessors.rs");

extern bool rs_callback_from_typval(Callback *callback, const typval_T *arg);
extern int rs_option_is_global_local(int opt_idx);
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

extern OptVal rs_optval_from_varp(OptIndex opt_idx, void *varp);
extern Object rs_optval_as_object(OptVal o);
extern OptVal rs_object_as_optval(Object o, bool *error);
extern void rs_set_option_varp(OptIndex opt_idx, void *varp, OptVal value, int free_oldval);
extern tabpage_T *rs_win_find_tabpage(win_T *win);

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

int nvim_verbose_check_and_open(void) {
  verbose_stop();
  if (*p_vfile != NUL && verbose_open() == FAIL) {
    return 0;  // FAIL
  }
  return 1;  // OK
}

const char *nvim_get_curbuf_sua(void) { return curbuf->b_p_sua; }
int nvim_win_get_diff(win_T *win) { return win ? win->w_p_diff : 0; }
int nvim_option_win_get_view_height(win_T *win) { return win ? win->w_view_height : 0; }
const char *nvim_win_get_p_wbr(win_T *win) { return win ? (const char *)win->w_p_wbr : NULL; }
int nvim_win_get_briopt_list(win_T *win) { return win ? win->w_briopt_list : 0; }
const char *nvim_win_get_p_briopt(win_T *win) { return win ? win->w_p_briopt : NULL; }
void nvim_win_set_briopt_shift(win_T *win, int val) { if (win) { win->w_briopt_shift = val; } }
void nvim_win_set_briopt_min(win_T *win, int val) { if (win) { win->w_briopt_min = val; } }
void nvim_win_set_briopt_sbr(win_T *win, int val) { if (win) { win->w_briopt_sbr = (val != 0); } }
void nvim_win_set_briopt_list(win_T *win, int val) { if (win) { win->w_briopt_list = val; } }
void nvim_win_set_briopt_vcol(win_T *win, int val) { if (win) { win->w_briopt_vcol = val; } }
const char *nvim_option_win_get_stc(win_T *win) { return win ? (const char *)win->w_p_stc : NULL; }
void nvim_option_win_set_nrwidth(win_T *win, int value) { if (win) win->w_nrwidth_line_count = value; }
int nvim_option_win_get_sms(win_T *win) { return win ? win->w_p_sms : 0; }
void nvim_option_win_set_skipcol(win_T *win, int value) { if (win) win->w_skipcol = value; }
int nvim_buf_get_p_swf(buf_T *buf) { return buf ? buf->b_p_swf : 0; }
int nvim_buf_get_p_udf(buf_T *buf) { return buf ? buf->b_p_udf : 0; }
void nvim_option_buf_set_modified_was_set(buf_T *buf, int val) { if (buf) buf->b_modified_was_set = val; }
int nvim_option_buf_get_b_p_ro(buf_T *buf) { return buf ? buf->b_p_ro : 0; }
void nvim_option_buf_set_b_did_warn(buf_T *buf, int val) { if (buf) buf->b_did_warn = val != 0; }
void *nvim_option_buf_get_terminal_ptr(buf_T *buf) { return buf ? buf->terminal : NULL; }
int nvim_option_buf_get_b_p_bin(buf_T *buf) { return buf ? buf->b_p_bin : 0; }
void nvim_buf_set_b_p_ul(buf_T *buf, OptInt val) { buf->b_p_ul = val; }
const char *nvim_compile_cap_prog_win(win_T *win) { return compile_cap_prog(win->w_s); }
bool parse_border_opt(char *border_opt);  // defined in optionstr.c
void nvim_callback_for_all_tab_windows(void (*callback)(win_T *))
  { FOR_ALL_TAB_WINDOWS(tp, wp) { callback(wp); } }
void nvim_for_all_buffers(void (*callback)(buf_T *))
  { FOR_ALL_BUFFERS(bp) { callback(bp); } }
int nvim_buf_is_changed(buf_T *buf) { return buf ? bufIsChanged(buf) : 0; }
int nvim_buf_has_memfile(buf_T *buf) { return buf && buf->b_ml.ml_mfp != NULL; }
int nvim_buf_get_p_bl(buf_T *buf) { return buf ? buf->b_p_bl : 0; }
void nvim_apply_autocmds_buf_event(int event, buf_T *buf) { apply_autocmds((event_T)event, NULL, NULL, true, buf); }
int nvim_win_get_p_pvw(win_T *win) { return win ? win->w_p_pvw : 0; }
void nvim_win_set_p_pvw(win_T *win, int val) { if (win) win->w_p_pvw = val != 0; }
void nvim_for_all_windows_in_curtab(void (*callback)(win_T *, void *), void *ud)
  { FOR_ALL_WINDOWS_IN_TAB(wp, curtab) { callback(wp, ud); } }
int nvim_win_get_p_spell(win_T *win) { return win ? win->w_p_spell : 0; }
void *nvim_buf_get_b_p_sw_addr(buf_T *buf) { return buf ? (void *)&buf->b_p_sw : NULL; }
OptInt nvim_buf_get_b_p_sw(buf_T *buf) { return buf ? buf->b_p_sw : 0; }
void nvim_callback_set_pum_grid_blending(int value) { pum_grid.blending = (value != 0); }
void nvim_callback_win_clamp_winbl(win_T *win) { if (win) { if (win->w_p_winbl > 100) win->w_p_winbl = 100; if (win->w_p_winbl < 0) win->w_p_winbl = 0; } }
void nvim_callback_win_set_hl_needs_update(win_T *win, int value) {
  if (win) win->w_hl_needs_update = (value != 0);
}
void nvim_callback_win_set_scbind_pos(win_T *win, int value) {
  if (win) win->w_scbind_pos = value;
}
// Dereference varp as char** to get the current string value
const char *nvim_optset_get_varp_str(const void *args)
{
  char **varp = (char **)((const optset_T *)args)->os_varp;
  return varp ? *varp : NULL;
}
char *nvim_optset_get_errbuf(const void *args) { return ((const optset_T *)args)->os_errbuf; }
size_t nvim_optset_get_errbuflen(const void *args) { return ((const optset_T *)args)->os_errbuflen; }
int nvim_call_briopt_check_win(const char *val, win_T *win) { return briopt_check(val, win) == OK ? 1 : 0; }
const void *nvim_win_get_p_briopt_addr(win_T *win) { return win ? (const void *)&win->w_p_briopt : NULL; }
const void *nvim_optset_get_varp_ptr(const void *args) { return ((const optset_T *)args)->os_varp; }
const char *nvim_optset_get_newval_str(const void *args) { return ((const optset_T *)args)->os_newval.string.data; }
unsigned nvim_buf_get_bkc_flags(buf_T *buf) { return buf->b_bkc_flags; }
void nvim_buf_set_bkc_flags(buf_T *buf, unsigned val) { buf->b_bkc_flags = val; }
const char *nvim_buf_get_p_bkc(buf_T *buf) { return buf->b_p_bkc; }
unsigned nvim_win_get_spo_flags(win_T *win) { return win->w_s->b_p_spo_flags; }
void nvim_win_set_spo_flags(win_T *win, unsigned val) { win->w_s->b_p_spo_flags = val; }
int nvim_win_get_ns_hl_winhl(win_T *win) { return win->w_ns_hl_winhl; }
void nvim_win_set_ns_hl_winhl(win_T *win, int val) { win->w_ns_hl_winhl = val; }
void nvim_win_set_ns_hl(win_T *win, int val) { win->w_ns_hl = val; }
const char *nvim_win_get_p_winhl(win_T *win) { return win ? win->w_p_winhl : NULL; }
const void *nvim_win_get_p_winhl_addr(win_T *win) { return win ? (const void *)&win->w_p_winhl : NULL; }
int nvim_winhl_ns_prepare(win_T *wp) {
  if (wp->w_ns_hl_winhl == 0) { wp->w_ns_hl_winhl = (int)nvim_create_namespace(NULL_STRING); }
  else { get_decor_provider(wp->w_ns_hl_winhl, true)->hl_valid++; }
  return wp->w_ns_hl_winhl;
}
void nvim_winhl_ns_hl_def(int ns_hl, int hl_id_link, int hl_id) { HlAttrs attrs = HLATTRS_INIT; attrs.rgb_ae_attr |= HL_GLOBAL; ns_hl_def(ns_hl, hl_id_link, attrs, hl_id, NULL); }

const char *nvim_win_get_p_culopt(win_T *wp) { return wp ? wp->w_p_culopt : NULL; }
void nvim_win_set_p_culopt_flags(win_T *wp, uint8_t flags) { if (wp) wp->w_p_culopt_flags = flags; }

#include "option_shim.c.generated.h"
#include "options.generated.h"
#include "options_map.generated.h"

void nvim_optset_restore_oldval_number(const void *args) {
  const optset_T *a = (const optset_T *)args;
  rs_set_option_varp(a->os_idx, a->os_varp, (OptVal){ .type = kOptValTypeNumber, .data = a->os_oldval }, 0);
}
vimoption_T *nvim_get_options_array(void) { return options; }
uint32_t nvim_get_option_flags(OptIndex opt_idx) { return (opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) ? 0 : options[opt_idx].flags; }
void *nvim_get_option_var(OptIndex opt_idx) { return (opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) ? NULL : options[opt_idx].var; }
void *nvim_vimoption_get_var(vimoption_T *p) { return p->var; }
unsigned *nvim_vimoption_get_flags_var_ptr(vimoption_T *p) { return p->flags_var; }
OptIndex nvim_get_opt_idx_from_ptr(vimoption_T *p) { return (OptIndex)(p - options); }
int nvim_get_sizeof_winopt_T(void) { return (int)sizeof(winopt_T); }
int nvim_get_option_type(OptIndex opt_idx) { return (int)options[opt_idx].type; }
int nvim_get_option_scope_flags(OptIndex opt_idx) { return (int)options[opt_idx].scope_flags; }
int nvim_get_option_scope_idx(OptIndex opt_idx, int scope) { return (int)options[opt_idx].scope_idx[scope]; }
int nvim_get_option_immutable(OptIndex opt_idx) { return (int)options[opt_idx].immutable; }
const void *nvim_get_option_def_val_data_ptr(OptIndex opt_idx) { return &options[opt_idx].def_val.data; }
void *nvim_get_option_script_ctx_ptr(OptIndex opt_idx) { return &options[opt_idx].script_ctx; }
void nvim_set_option_def_val(OptIndex opt_idx, OptVal val) { options[opt_idx].def_val = val; }
int64_t nvim_get_cmdheight_def_number(void) { return options[kOptCmdheight].def_val.data.number; }

#ifdef UNIX
int nvim_is_root_user(void) { return getuid() == ROOT_UID ? 1 : 0; }
#else
int nvim_is_root_user(void) { return 0; }
#endif

sctx_T nvim_get_option_script_ctx(OptIndex opt_idx) { return (opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) ? (sctx_T){ 0 } : options[opt_idx].script_ctx; }

sctx_T nvim_get_win_p_script_ctx(win_T *win, OptIndex opt_idx) { return (!win || opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) ? (sctx_T){ 0 } : win->w_p_script_ctx[opt_idx]; }

sctx_T nvim_get_buf_p_script_ctx(buf_T *buf, OptIndex opt_idx) { return (!buf || opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) ? (sctx_T){ 0 } : buf->b_p_script_ctx[opt_idx]; }

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

const char *nvim_curbuf_get_b_p_ep(void) { return curbuf->b_p_ep; }
const char *nvim_curbuf_get_b_p_ffu(void) { return curbuf->b_p_ffu; }
const char *nvim_buf_get_p_flp(buf_T *buf) { return buf->b_p_flp; }
unsigned nvim_win_get_ve_flags(win_T *wp) { return wp->w_ve_flags; }
OptInt nvim_buf_get_b_p_iminsert(buf_T *buf) { return buf->b_p_iminsert; }
OptInt nvim_buf_get_b_p_imsearch(buf_T *buf) { return buf->b_p_imsearch; }
int nvim_curbuf_get_b_p_ma(void) { return curbuf->b_p_ma; }
void nvim_curbuf_set_b_p_ma(int v) { curbuf->b_p_ma = v != 0; }
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

void *nvim_get_varp_scope_by_idx(OptIndex opt_idx, int opt_flags)
{
  return get_varp_scope(&options[opt_idx], opt_flags);
}
OptVal nvim_option_get_def_val(OptIndex opt_idx) { return options[opt_idx].def_val; }

void nvim_option_ilog_rtp(void) { ILOG("startup runtimepath/packpath value: %s", p_rtp); }
int nvim_get_cmd_idx_setlocal(void) { return (int)CMD_setlocal; }
int nvim_get_cmd_idx_setglobal(void) { return (int)CMD_setglobal; }

// Get the option fullname for writing to session files
const char *nvim_option_get_fullname(OptIndex opt_idx) { return options[opt_idx].fullname; }

/// Get the address of p_kp (keywordprg global), as void*.
/// Used by Rust stropt_get_newval to detect keywordprg option.
void *nvim_option_get_p_kp_ptr(void) { return (void *)&p_kp; }

void check_redraw(uint32_t flags) { check_redraw_for(curbuf, curwin, flags); }

/// Direct hash-based option lookup for use by Rust (avoids circular delegation).
///
/// Called from rs_find_option_len / rs_find_option in index.rs.
OptIndex nvim_find_option_len_hash(const char *name, size_t len)
{
  int index = find_option_hash(name, len);
  return index >= 0 ? option_hash_elems[index].opt_idx : kOptInvalid;
}

// Context-switch helpers called by Rust (Phase 6).
// Rust uses MaybeUninit<[u8; N]> for switchwin_T and aco_save_T.
_Static_assert(sizeof(switchwin_T) == 24,
               "sizeof(switchwin_T) changed - update SWITCHWIN_SIZE in option/src/contextswitch.rs");
_Static_assert(sizeof(aco_save_T) == 64,
               "sizeof(aco_save_T) changed - update ACO_SAVE_SIZE in option/src/contextswitch.rs");
int nvim_option_switch_win_noblock(void *switchwin, void *win, void *tabpage) {
  return switch_win_noblock((switchwin_T *)switchwin, (win_T *)win, (tabpage_T *)tabpage, true);
}
void nvim_option_restore_win_noblock(void *switchwin) {
  restore_win_noblock((switchwin_T *)switchwin, true);
}
void nvim_option_aucmd_prepbuf(void *aco, void *buf) {
  aucmd_prepbuf((aco_save_T *)aco, (buf_T *)buf);
}
void nvim_option_aucmd_restbuf(void *aco) {
  aucmd_restbuf((aco_save_T *)aco);
}
OptVal nvim_option_get_option_value(OptIndex opt_idx, int opt_flags) {
  return get_option_value(opt_idx, opt_flags);
}
const char *nvim_option_set_value_handle_tty(const char *name, OptIndex opt_idx,
                                             OptVal value, int opt_flags) {
  return set_option_value_handle_tty(name, opt_idx, value, opt_flags);
}

extern void rs_ui_refresh_options(void);
void ui_refresh_options(void) { rs_ui_refresh_options(); }

/// Get pointer to option variable, depending on local or global scope.
///
/// @param  opt_flags  Option flags (can be OPT_LOCAL, OPT_GLOBAL or a combination).
void *get_varp_scope(vimoption_T *p, int opt_flags) { return get_varp_scope_from(p, opt_flags, curbuf, curwin); }

/// Get pointer to option variable at 'opt_idx', depending on local or global
/// scope.
void *get_option_varp_scope_from(OptIndex opt_idx, int opt_flags, buf_T *buf, win_T *win)
{
  return get_varp_scope_from(&(options[opt_idx]), opt_flags, buf, win);
}

static inline OptIndex get_opt_idx(vimoption_T *opt)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_PURE
{
  return (OptIndex)(opt - options);
}

static inline void *get_varp(vimoption_T *p) { return get_varp_from(p, curbuf, curwin); }

/// Copy options from one window to another.
/// Used when splitting a window.
void win_copy_options(win_T *wp_from, win_T *wp_to)
{
  copy_winopt(&wp_from->w_onebuf_opt, &wp_to->w_onebuf_opt);
  copy_winopt(&wp_from->w_allbuf_opt, &wp_to->w_allbuf_opt);
  didset_window_options(wp_to, true);
}

extern int expand_option_start_col;
extern bool expand_option_append;

int nvim_xp_get_context(expand_T *xp) { return xp->xp_context; }
void nvim_xp_set_context(expand_T *xp, int val) { xp->xp_context = val; }
char *nvim_xp_get_pattern(expand_T *xp) { return xp->xp_pattern; }
void nvim_xp_set_pattern(expand_T *xp, char *val) { xp->xp_pattern = val; }
void nvim_xp_set_prefix(expand_T *xp, int val) { xp->xp_prefix = (xp_prefix_T)val; }
char *nvim_xp_get_line(expand_T *xp) { return xp->xp_line; }
int nvim_xp_get_backslash(expand_T *xp) { return xp->xp_backslash; }
void nvim_xp_set_backslash(expand_T *xp, int val) { xp->xp_backslash = val; }
char *nvim_xp_get_buf(expand_T *xp) { return xp->xp_buf; }

int nvim_option_has_expand_cb(OptIndex opt_idx) { return (opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) ? 0 : (options[opt_idx].opt_expand_cb != NULL ? 1 : 0); }

// Returns an identity code for special option vars: 0=none, 1=syn, 2=ft, 3=keymap, 4=sps
int nvim_opt_var_identity(OptIndex opt_idx) {
  if (opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) return 0;
  void *v = options[opt_idx].var;
  if (v == &p_syn) return 1;
  if (v == &p_ft) return 2;
  if (v == &p_keymap) return 3;
  if (v == &p_sps) return 4;
  return 0;
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
        typval_T opt_tv = optval_as_tv(rs_optval_from_varp(opt_idx, varp), true);
        tv_dict_add_tv(d, opt->fullname, strlen(opt->fullname), &opt_tv);
      }
    }
  }

  return d;
}

/// Apply EVENT_SYNTAX autocmds for the given buffer.
/// @param force  whether to force the autocmd (value_changed || syn_recursive == 1)
void nvim_apply_syntax_autocmd(buf_T *buf, bool force)
{
  apply_autocmds(EVENT_SYNTAX, buf->b_p_syn, buf->b_fname, force, buf);
}

const char *nvim_win_get_b_p_spl(win_T *win) { return (win && win->w_s) ? win->w_s->b_p_spl : NULL; }

int nvim_buf_get_b_p_ff_first(const buf_T *buf) { return (buf && buf->b_p_ff) ? (unsigned char)(*buf->b_p_ff) : 0; }

void *nvim_get_varp_by_idx(OptIndex opt_idx) { return get_varp(&options[opt_idx]); }

/// Check if varp points to curbuf->b_changed.
/// Used by showoneopt to detect the 'modified' pseudo-boolean option.
int nvim_varp_is_curbuf_b_changed(const void *varp) { return (const int *)varp == &curbuf->b_changed ? 1 : 0; }

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

const char *nvim_option_get_shortname(OptIndex opt_idx) { return options[opt_idx].shortname; }

int nvim_regmatch_get_rm_ic(const void *regmatch) { return ((const regmatch_T *)regmatch)->rm_ic; }

void nvim_regmatch_set_rm_ic(void *regmatch, int val) { ((regmatch_T *)regmatch)->rm_ic = val; }

size_t nvim_option_get_fuzmatch_size(void) { return sizeof(fuzmatch_str_T); }

void nvim_option_fuzmatch_set(void *fuzmatch, int idx, const char *str, int score) {
  fuzmatch_str_T *fm = (fuzmatch_str_T *)fuzmatch;
  fm[idx].idx = idx; fm[idx].str = xstrdup(str); fm[idx].score = score;
}

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

int nvim_option_has_did_set_cb(OptIndex opt_idx) { return options[opt_idx].opt_did_set_cb != NULL ? 1 : 0; }

vimoption_T *nvim_get_option_ptr_by_idx(OptIndex opt_idx) { return &options[opt_idx]; }

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
  ui_call_option_set(cstr_as_string(options[opt_idx].fullname), rs_optval_as_object(saved_new_value));
}

/// Copy global script_ctx for a buf-opt index to the buffer's script_ctx array.
/// Implements COPY_OPT_SCTX(buf, bv).
void nvim_buf_copy_opt_sctx(buf_T *buf, int bv)
{
  if (buf && bv >= 0 && (size_t)bv < ARRAY_SIZE(buf->b_p_script_ctx)) {
    buf->b_p_script_ctx[bv] = options[buf_opt_idx[bv]].script_ctx;
  }
}

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

void nvim_call_curbuf_tabstop_set_vsts(void) { xfree(curbuf->b_p_vsts_array); tabstop_set(curbuf->b_p_vsts, &curbuf->b_p_vsts_array); }
void nvim_call_curbuf_tabstop_set_vts(void) { xfree(curbuf->b_p_vts_array); tabstop_set(curbuf->b_p_vts, &curbuf->b_p_vts_array); }

/// free_operatorfunc_option() wrapper (EXITFREE only).
#if defined(EXITFREE)
void nvim_call_free_operatorfunc_option(void) { free_operatorfunc_option(); }
#else
void nvim_call_free_operatorfunc_option(void) {}
#endif

char *nvim_oe_get_opt_value(const optexpand_T *args) { return args->oe_opt_value; }
const char *nvim_oe_get_set_arg(const optexpand_T *args) { return args->oe_set_arg; }
bool nvim_oe_get_append(const optexpand_T *args) { return args->oe_append; }
bool nvim_oe_get_include_orig_val(const optexpand_T *args) { return args->oe_include_orig_val; }
regmatch_T *nvim_oe_get_regmatch(const optexpand_T *args) { return args->oe_regmatch; }
expand_T *nvim_oe_get_xp(const optexpand_T *args) { return args->oe_xp; }
char *nvim_oe_get_varp(const optexpand_T *args) { return args->oe_varp; }
int nvim_oe_get_idx(const optexpand_T *args) { return (int)args->oe_idx; }

const char **nvim_option_get_values(const vimoption_T *opt) { return (const char **)opt->values; }
size_t nvim_option_get_values_len(const vimoption_T *opt) { return opt->values_len; }
const char *nvim_win_get_p_lcs(const win_T *win) { return win ? win->w_p_lcs : NULL; }
bool nvim_get_p_fic(void) { return p_fic; }

bool nvim_regmatch_has_regprog(const void *handle)
{
  if (handle == NULL) { return false; }
  return ((const regmatch_T *)handle)->regprog != NULL;
}

/// Execute vim_regexec on a regmatch handle against a string.
bool nvim_regmatch_exec(void *handle, const char *name)
{
  if (handle == NULL || name == NULL) { return false; }
  return vim_regexec((regmatch_T *)handle, (char *)name, 0);
}

/// Call do_set(s, flags).  Returns OK (0) or FAIL (-1).
int nvim_do_set(char *s, int flags) { return do_set(s, flags); }

sctx_T *nvim_modeline_sctx_save_and_set(int lnum) {
  sctx_T *saved = xmalloc(sizeof(sctx_T)); *saved = current_sctx;
  current_sctx = (sctx_T){ .sc_sid = SID_MODELINE, .sc_lnum = lnum }; return saved;
}
void nvim_modeline_sctx_restore(sctx_T *saved) { current_sctx = *saved; xfree(saved); }
