#define IN_OPTION_C
#include <string.h>
#include "auto/config.h"
#include "nvim/api/extmark.h"
#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/decoration_provider.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/vars.h"
#include "nvim/eval/window.h"
#include "nvim/fuzzy.h"
#include "nvim/globals.h"
#include "nvim/highlight.h"
#include "nvim/indent.h"
#include "nvim/log.h"
#include "nvim/mark.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/popupmenu.h"
#include "nvim/regexp.h"
#include "nvim/spell.h"
#include "nvim/strings.h"
#include "nvim/terminal.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/window.h"
_Static_assert(sizeof(vimoption_T) == 160,
               "sizeof(vimoption_T) changed - update Rust VIMOPTION_SIZE in option/src/accessors.rs");
extern bool rs_callback_from_typval(Callback *callback, const typval_T *arg);
extern int rs_option_is_global_local(int opt_idx);
extern int rs_option_get_type(int opt_idx);
extern void rs_option_value2string(OptIndex opt_idx, int opt_flags);
extern OptVal rs_optval_from_varp(OptIndex opt_idx, void *varp);
extern Object rs_optval_as_object(OptVal o);
extern OptVal rs_object_as_optval(Object o, bool *error);
extern void rs_set_option_varp(OptIndex opt_idx, void *varp, OptVal value, int free_oldval);
extern tabpage_T *rs_win_find_tabpage(win_T *win);
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
int nvim_verbose_check_and_open(void)
  { verbose_stop(); if (*p_vfile != NUL && verbose_open() == FAIL) return 0; return 1; }
const char *nvim_get_curbuf_sua(void) { return curbuf->b_p_sua; }
const char *nvim_win_get_p_wbr(win_T *win) { return win ? (const char *)win->w_p_wbr : NULL; }
int nvim_win_get_briopt_list(win_T *win) { return win ? win->w_briopt_list : 0; }
const char *nvim_win_get_p_briopt(win_T *win) { return win ? win->w_p_briopt : NULL; }
void nvim_win_set_briopt_shift(win_T *win, int val) { if (win) { win->w_briopt_shift = val; } }
void nvim_win_set_briopt_min(win_T *win, int val) { if (win) { win->w_briopt_min = val; } }
void nvim_win_set_briopt_sbr(win_T *win, int val) { if (win) { win->w_briopt_sbr = (val != 0); } }
void nvim_win_set_briopt_list(win_T *win, int val) { if (win) { win->w_briopt_list = val; } }
void nvim_win_set_briopt_vcol(win_T *win, int val) { if (win) { win->w_briopt_vcol = val; } }
const char *nvim_option_win_get_stc(win_T *win) { return win ? (const char *)win->w_p_stc : NULL; }
int nvim_buf_get_p_swf(buf_T *buf) { return buf ? buf->b_p_swf : 0; }
const char *nvim_compile_cap_prog_win(win_T *win) { return compile_cap_prog(win->w_s); }
void nvim_callback_for_all_tab_windows(void (*callback)(win_T *))
  { FOR_ALL_TAB_WINDOWS(tp, wp) { callback(wp); } }
void nvim_for_all_buffers(void (*callback)(buf_T *))
  { FOR_ALL_BUFFERS(bp) { callback(bp); } }
int nvim_buf_is_changed(buf_T *buf) { return buf ? bufIsChanged(buf) : 0; }
void nvim_apply_autocmds_buf_event(int event, buf_T *buf) { apply_autocmds((event_T)event, NULL, NULL, true, buf); }
int nvim_win_get_p_pvw(win_T *win) { return win ? win->w_p_pvw : 0; }
void nvim_for_all_windows_in_curtab(void (*callback)(win_T *, void *), void *ud)
  { FOR_ALL_WINDOWS_IN_TAB(wp, curtab) { callback(wp, ud); } }
int nvim_win_get_p_spell(win_T *win) { return win ? win->w_p_spell : 0; }
void nvim_callback_set_pum_grid_blending(int value) { pum_grid.blending = (value != 0); }
void nvim_callback_win_clamp_winbl(win_T *win) { if (win) { if (win->w_p_winbl > 100) win->w_p_winbl = 100; if (win->w_p_winbl < 0) win->w_p_winbl = 0; } }
const char *nvim_optset_get_varp_str(const void *args)
  { char **varp = (char **)((const optset_T *)args)->os_varp; return varp ? *varp : NULL; }
char *nvim_optset_get_errbuf(const void *args) { return ((const optset_T *)args)->os_errbuf; }
size_t nvim_optset_get_errbuflen(const void *args) { return ((const optset_T *)args)->os_errbuflen; }
int nvim_call_briopt_check_win(const char *val, win_T *win) { return briopt_check(val, win) == OK ? 1 : 0; }
const void *nvim_win_get_p_briopt_addr(win_T *win) { return win ? (const void *)&win->w_p_briopt : NULL; }
const void *nvim_optset_get_varp_ptr(const void *args) { return ((const optset_T *)args)->os_varp; }
const char *nvim_optset_get_newval_str(const void *args) { return ((const optset_T *)args)->os_newval.string.data; }
unsigned nvim_win_get_spo_flags(win_T *win) { return win->w_s->b_p_spo_flags; }
void nvim_win_set_spo_flags(win_T *win, unsigned val) { win->w_s->b_p_spo_flags = val; }
const char *nvim_win_get_p_winhl(win_T *win) { return win ? win->w_p_winhl : NULL; }
const void *nvim_win_get_p_winhl_addr(win_T *win) { return win ? (const void *)&win->w_p_winhl : NULL; }
int nvim_winhl_ns_prepare(win_T *wp)
  { if (wp->w_ns_hl_winhl == 0) { wp->w_ns_hl_winhl = (int)nvim_create_namespace(NULL_STRING); } else { get_decor_provider(wp->w_ns_hl_winhl, true)->hl_valid++; } return wp->w_ns_hl_winhl; }
void nvim_winhl_ns_hl_def(int ns_hl, int hl_id_link, int hl_id) { HlAttrs attrs = HLATTRS_INIT; attrs.rgb_ae_attr |= HL_GLOBAL; ns_hl_def(ns_hl, hl_id_link, attrs, hl_id, NULL); }
const char *nvim_win_get_p_culopt(win_T *wp) { return wp ? wp->w_p_culopt : NULL; }
#include "option_shim.c.generated.h"
#include "options.generated.h"
#include "options_map.generated.h"
void nvim_optset_restore_oldval_number(const void *args)
  { const optset_T *a = (const optset_T *)args; rs_set_option_varp(a->os_idx, a->os_varp, (OptVal){ .type = kOptValTypeNumber, .data = a->os_oldval }, 0); }
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
sctx_T nvim_get_option_script_ctx(OptIndex opt_idx) { return (opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) ? (sctx_T){ 0 } : options[opt_idx].script_ctx; }
sctx_T nvim_get_win_p_script_ctx(win_T *win, OptIndex opt_idx) { return (!win || opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) ? (sctx_T){ 0 } : win->w_p_script_ctx[opt_idx]; }
sctx_T nvim_get_buf_p_script_ctx(buf_T *buf, OptIndex opt_idx) { return (!buf || opt_idx < 0 || (size_t)opt_idx >= ARRAY_SIZE(options)) ? (sctx_T){ 0 } : buf->b_p_script_ctx[opt_idx]; }
void nvim_curbuf_set_b_p_tw(OptInt v) { curbuf->b_p_tw = v; }
int nvim_curbuf_get_b_p_ml(void) { return curbuf->b_p_ml; }
int nvim_curbuf_get_b_p_et(void) { return curbuf->b_p_et; }
const char *nvim_curbuf_get_b_p_ep(void) { return curbuf->b_p_ep; }
const char *nvim_curbuf_get_b_p_ffu(void) { return curbuf->b_p_ffu; }
unsigned nvim_win_get_ve_flags(win_T *wp) { return wp->w_ve_flags; }
OptInt *nvim_get_curbuf_b_p_iminsert_ptr(void) { return &curbuf->b_p_iminsert; }
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
  { return get_varp_scope(&options[opt_idx], opt_flags); }
OptVal nvim_option_get_def_val(OptIndex opt_idx) { return options[opt_idx].def_val; }
void nvim_option_ilog_rtp(void) { ILOG("startup runtimepath/packpath value: %s", p_rtp); }
int nvim_get_cmd_idx_setlocal(void) { return (int)CMD_setlocal; }
int nvim_get_cmd_idx_setglobal(void) { return (int)CMD_setglobal; }
const char *nvim_option_get_fullname(OptIndex opt_idx) { return options[opt_idx].fullname; }
void *nvim_option_get_p_kp_ptr(void) { return (void *)&p_kp; }
void *nvim_option_get_p_syn_ptr(void) { return (void *)&p_syn; }
void *nvim_option_get_p_ft_ptr(void) { return (void *)&p_ft; }
void *nvim_option_get_p_keymap_ptr(void) { return (void *)&p_keymap; }
void *nvim_option_get_p_sps_ptr(void) { return (void *)&p_sps; }
void *nvim_option_get_p_bdir_ptr(void) { return (void *)&p_bdir; }
void *nvim_option_get_p_dir_ptr(void) { return (void *)&p_dir; }
void *nvim_option_get_p_pp_ptr(void) { return (void *)&p_pp; }
void *nvim_option_get_p_rtp_ptr(void) { return (void *)&p_rtp; }
void *nvim_option_get_p_vdir_ptr(void) { return (void *)&p_vdir; }
void *nvim_option_get_p_path_ptr(void) { return (void *)&p_path; }
void *nvim_option_get_p_cdpath_ptr(void) { return (void *)&p_cdpath; }
void *nvim_option_get_p_tags_ptr(void) { return (void *)&p_tags; }
OptIndex nvim_find_option_len_hash(const char *name, size_t len)
  { int index = find_option_hash(name, len); return index >= 0 ? option_hash_elems[index].opt_idx : kOptInvalid; }
_Static_assert(sizeof(switchwin_T) == 24,
               "sizeof(switchwin_T) changed - update SWITCHWIN_SIZE in option/src/contextswitch.rs");
_Static_assert(sizeof(aco_save_T) == 64,
               "sizeof(aco_save_T) changed - update ACO_SAVE_SIZE in option/src/contextswitch.rs");
int nvim_option_switch_win_noblock(void *switchwin, void *win, void *tabpage)
  { return switch_win_noblock((switchwin_T *)switchwin, (win_T *)win, (tabpage_T *)tabpage, true); }
void nvim_option_restore_win_noblock(void *switchwin)
  { restore_win_noblock((switchwin_T *)switchwin, true); }
void nvim_option_aucmd_prepbuf(void *aco, void *buf)
  { aucmd_prepbuf((aco_save_T *)aco, (buf_T *)buf); }
void nvim_option_aucmd_restbuf(void *aco)
  { aucmd_restbuf((aco_save_T *)aco); }
OptVal nvim_option_get_option_value(OptIndex opt_idx, int opt_flags)
  { return get_option_value(opt_idx, opt_flags); }
const char *nvim_option_set_value_handle_tty(const char *name, OptIndex opt_idx, OptVal value, int opt_flags)
  { return set_option_value_handle_tty(name, opt_idx, value, opt_flags); }
extern void rs_ui_refresh_options(void);
void ui_refresh_options(void) { rs_ui_refresh_options(); }
void *get_varp_scope(vimoption_T *p, int opt_flags) { return get_varp_scope_from(p, opt_flags, curbuf, curwin); }
void *get_option_varp_scope_from(OptIndex opt_idx, int opt_flags, buf_T *buf, win_T *win)
  { return get_varp_scope_from(&(options[opt_idx]), opt_flags, buf, win); }
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
// Phase 1: os_restore_chartab accessor for did_set_isopt
void nvim_optset_set_restore_chartab(void *args, int val) { ((optset_T *)args)->os_restore_chartab = (val != 0); }
// buf field accessors for did_set_* callbacks
char *nvim_win_get_p_ve(win_T *wp) { return wp ? wp->w_p_ve : NULL; }
unsigned *nvim_win_get_ve_flags_ptr(win_T *wp) { return wp ? &wp->w_ve_flags : NULL; }
// signcolumn helpers (nvim_win_get_virtcol, nvim_win_get_minscwidth, nvim_win_set_nrwidth_line_count are in window/src/win_struct.rs)
char *nvim_win_get_p_scl(win_T *wp) { return wp ? wp->w_p_scl : NULL; }
// iskeyword: check if os_varp points to global p_isk
int nvim_optset_varp_is_global_isk(const void *args) { return ((const optset_T *)args)->os_varp == (void *)&p_isk ? 1 : 0; }
// Phase 2: helpers for complex did_set_* callbacks
// Notify all terminal buffers of theme change
void nvim_notify_all_terminals_theme(int dark) { FOR_ALL_BUFFERS(buf) { if (buf->terminal) { terminal_notify_theme(buf->terminal, dark); } } }
// optset oldval first char (for background comparison)
int nvim_optset_oldval_first_char(const void *args) { const char *s = ((const optset_T *)args)->os_oldval.string.data; return s ? (int)(unsigned char)s[0] : 0; }
// did_set_buftype helpers
void nvim_buf_buftype_prompt_init(buf_T *buf) { set_option_direct(kOptComments, STATIC_CSTR_AS_OPTVAL(""), OPT_LOCAL, SID_NONE); pos_T next_prompt = { .lnum = buf->b_ml.ml_line_count, .col = 1, .coladd = 0 }; RESET_FMARK(&buf->b_prompt_start, next_prompt, 0, ((fmarkv_T)INIT_FMARKV)); }
// nvim_win_get_status_height and nvim_win_set_redr_status are in window/src/win_struct.rs
// keymap helpers
int nvim_get_secure(void) { return secure; }
void nvim_set_secure(int val) { secure = val; }
// get/set ru_wid (for rulerformat)
void nvim_set_ru_wid(int val) { ru_wid = val; }
// encoding helpers
int nvim_optset_varp_is_p_fenc(const void *args) { char **gvarp = (char **)get_option_varp_scope_from(((const optset_T *)args)->os_idx, OPT_GLOBAL, (buf_T *)((const optset_T *)args)->os_buf, NULL); return gvarp == &p_fenc ? 1 : 0; }
int nvim_optset_varp_is_p_enc(const void *args) { return ((const optset_T *)args)->os_varp == (void *)&p_enc ? 1 : 0; }
char *nvim_enc_canonize(char *enc) { return enc_canonize(enc); }
// statusline default value helper
const char *nvim_optset_stl_get_default(const void *args) { return get_option_default(((const optset_T *)args)->os_idx, ((const optset_T *)args)->os_flags).data.string.data; }
int nvim_get_kOptStatusline(void) { return (int)kOptStatusline; }
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
    tv = eval_expr(optval, NULL);
    if (tv == NULL) {
      return FAIL;
    }
  } else {
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
void *nvim_tv_dict_alloc_for_winbuf(void) { return tv_dict_alloc(); }
void *nvim_get_varp_current(OptIndex opt_idx) { return get_varp_from(&options[opt_idx], curbuf, curwin); }
void nvim_dict_add_option_varp(void *dict, OptIndex opt_idx, void *varp)
  { typval_T opt_tv = optval_as_tv(rs_optval_from_varp(opt_idx, varp), true); tv_dict_add_tv((dict_T *)dict, options[opt_idx].fullname, strlen(options[opt_idx].fullname), &opt_tv); }
extern void *rs_get_winbuf_options(int bufopt);
dict_T *get_winbuf_options(const int bufopt)
  FUNC_ATTR_WARN_UNUSED_RESULT { return (dict_T *)rs_get_winbuf_options(bufopt); }
void nvim_apply_syntax_autocmd(buf_T *buf, bool force)
  { apply_autocmds(EVENT_SYNTAX, buf->b_p_syn, buf->b_fname, force, buf); }
const char *nvim_win_get_b_p_spl(win_T *win) { return (win && win->w_s) ? win->w_s->b_p_spl : NULL; }
void *nvim_get_varp_by_idx(OptIndex opt_idx) { return get_varp_from(&options[opt_idx], curbuf, curwin); }
int nvim_varp_is_curbuf_b_changed(const void *varp) { return (const int *)varp == &curbuf->b_changed ? 1 : 0; }
extern char *rs_escape_option_str_cmdline(const char *var);
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
void nvim_option_fuzmatch_set(void *fuzmatch, int idx, const char *str, int score)
  { fuzmatch_str_T *fm = (fuzmatch_str_T *)fuzmatch; fm[idx].idx = idx; fm[idx].str = xstrdup(str); fm[idx].score = score; }
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
void nvim_set_option_sctx_from_sid(OptIndex opt_idx, int opt_flags, int set_sid)
  { sctx_T script_ctx = set_sid == 0 ? current_sctx : (sctx_T){ .sc_sid = set_sid }; set_option_sctx(opt_idx, opt_flags, script_ctx); }
int nvim_option_has_did_set_cb(OptIndex opt_idx) { return options[opt_idx].opt_did_set_cb != NULL ? 1 : 0; }
vimoption_T *nvim_get_option_ptr_by_idx(OptIndex opt_idx) { return &options[opt_idx]; }
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
void nvim_ui_call_option_set(OptIndex opt_idx, OptVal saved_new_value)
  { ui_call_option_set(cstr_as_string(options[opt_idx].fullname), rs_optval_as_object(saved_new_value)); }
void nvim_buf_copy_opt_sctx(buf_T *buf, int bv)
{
  if (buf && bv >= 0 && (size_t)bv < ARRAY_SIZE(buf->b_p_script_ctx)) {
    buf->b_p_script_ctx[bv] = options[buf_opt_idx[bv]].script_ctx;
  }
}
_Static_assert((int)kBufOptAutocomplete == 0, "K_BUF_OPT_AUTOCOMPLETE mismatch");
_Static_assert((int)kBufOptWrapmargin == 90, "K_BUF_OPT_WRAPMARGIN mismatch");
_Static_assert(kBufOptCount == 91, "K_BUF_OPT_COUNT mismatch");
void nvim_call_bind_textdomain_codeset(void)
{
#ifdef HAVE_WORKING_LIBINTL
  (void)bind_textdomain_codeset(PROJECT_NAME, p_enc);
#endif
}
void nvim_call_curbuf_tabstop_set_vsts(void) { xfree(curbuf->b_p_vsts_array); tabstop_set(curbuf->b_p_vsts, &curbuf->b_p_vsts_array); }
void nvim_call_curbuf_tabstop_set_vts(void) { xfree(curbuf->b_p_vts_array); tabstop_set(curbuf->b_p_vts, &curbuf->b_p_vts_array); }
#if defined(EXITFREE)
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
  { if (handle == NULL) { return false; } return ((const regmatch_T *)handle)->regprog != NULL; }
bool nvim_regmatch_exec(void *handle, const char *name)
  { if (handle == NULL || name == NULL) { return false; } return vim_regexec((regmatch_T *)handle, (char *)name, 0); }
sctx_T *nvim_modeline_sctx_save_and_set(int lnum) {
  sctx_T *saved = xmalloc(sizeof(sctx_T)); *saved = current_sctx;
  current_sctx = (sctx_T){ .sc_sid = SID_MODELINE, .sc_lnum = lnum }; return saved;
}
void nvim_modeline_sctx_restore(sctx_T *saved) { current_sctx = *saved; xfree(saved); }

// Phase 3: chars option and signcolumn helpers
#include "nvim/api/private/helpers.h"
#include "nvim/api/win_config.h"
#include "nvim/charset.h"
#include "nvim/drawscreen.h"
#include "nvim/grid.h"
#include "nvim/optionstr.h"
const char *nvim_win_get_p_fcs(const win_T *win) { return win ? win->w_p_fcs : NULL; }
const void *nvim_win_get_p_lcs_addr(const win_T *win) { return win ? (const void *)&win->w_p_lcs : NULL; }
const void *nvim_win_get_p_fcs_addr(const win_T *win) { return win ? (const void *)&win->w_p_fcs : NULL; }
const void *nvim_get_p_lcs_addr(void) { return (const void *)&p_lcs; }
const void *nvim_get_p_fcs_addr(void) { return (const void *)&p_fcs; }
lcs_chars_T *nvim_win_get_lcs_chars_ptr(win_T *win) { return win ? &win->w_p_lcs_chars : NULL; }
void nvim_win_set_lcs_chars(win_T *win, const lcs_chars_T *val) { if (win && val) { win->w_p_lcs_chars = *val; } }
void nvim_win_set_fcs_chars(win_T *win, const fcs_chars_T *val) { if (win && val) { win->w_p_fcs_chars = *val; } }
bool nvim_parse_border_opt(char *border_opt) {
  WinConfig fconfig = WIN_CONFIG_INIT;
  Error err = ERROR_INIT;
  bool result = parse_winborder(&fconfig, border_opt, &err);
  api_clear_error(&err);
  return result;
}
void nvim_redraw_all_later_not_valid(void) { redraw_all_later(UPD_NOT_VALID); }
// Iterate all tab windows with empty local value, call set_chars_option
const char *nvim_for_all_tab_windows_set_chars(int what, char *errbuf, size_t errbuflen) {
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    char *opt = (what == kListchars) ? wp->w_p_lcs : wp->w_p_fcs;
    if (*opt == NUL) {
      const char *errmsg = set_chars_option(wp, opt, (CharsOption)what, true, errbuf, errbuflen);
      if (errmsg != NULL) {
        return errmsg;
      }
    }
  }
  return NULL;
}
// Check all tab windows, apply both lcs and fcs for each window
const char *nvim_for_all_tab_windows_check_impl(void) {
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    if (set_chars_option(wp, wp->w_p_lcs, kListchars, true, NULL, 0) != NULL) {
      return "E834: Conflicts with value of 'listchars'";
    }
    if (set_chars_option(wp, wp->w_p_fcs, kFillchars, true, NULL, 0) != NULL) {
      return "E835: Conflicts with value of 'fillchars'";
    }
  }
  return NULL;
}

// =============================================================================
// operatorfunc management (moved from ops.c Phase 1 migration)
// =============================================================================
#include "nvim/errors.h"

extern bool rs_set_ref_in_callback(Callback *callback, int copyID, ht_stack_T **ht_stack,
                                    list_stack_T **list_stack);

/// callback function for 'operatorfunc'
static Callback opfunc_cb;

/// Process the 'operatorfunc' option value.
const char *did_set_operatorfunc(optset_T *args FUNC_ATTR_UNUSED)
{
  if (option_set_callback_func(p_opfunc, &opfunc_cb) == FAIL) {
    return e_invarg;
  }
  return NULL;
}

#if defined(EXITFREE)
void free_operatorfunc_option(void) { callback_free(&opfunc_cb); }
#endif

/// Mark the global 'operatorfunc' callback with "copyID" so that it is not
/// garbage collected.
bool set_ref_in_opfunc(int copyID) { return rs_set_ref_in_callback(&opfunc_cb, copyID, NULL, NULL); }

/// Return a pointer to the opfunc_cb for Rust FFI use.
Callback *nvim_get_opfunc_cb(void) { return &opfunc_cb; }

/// Return true if p_opfunc is non-empty.
bool nvim_get_p_opfunc_nonempty(void) { return *p_opfunc != NUL; }

/// Set p_hls (hlsearch option).
void nvim_set_p_hls(bool val) { p_hls = val; }
