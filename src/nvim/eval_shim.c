// eval.c: Expression evaluation.

#include <assert.h>
#include <string.h>

#include "nvim/api/private/converter.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/autocmd.h"
#include "nvim/errors.h"
#include "nvim/buffer.h"
#include "nvim/channel.h"
#include "nvim/cursor.h"
#include "nvim/eval.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/event/proc.h"
#include "nvim/event/time.h"
#include "nvim/garray.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/insexpand.h"
#include "nvim/lua/executor.h"
#include "nvim/mark.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/os/fs.h"
#include "nvim/main.h"
#include "nvim/profile.h"
#include "nvim/runtime.h"
#include "nvim/strings.h"
#include "nvim/undo.h"
#include "nvim/window.h"

extern bool rs_set_ref_in_item(typval_T *tv, int copyID, ht_stack_T **ht_stack, list_stack_T **list_stack);
extern bool rs_set_ref_in_callback(Callback *callback, int copyID, ht_stack_T **ht_stack, list_stack_T **list_stack);
extern MultiQueue *rs_loop_get_events(Loop *loop);
extern bool rs_set_ref_in_callback_reader(CallbackReader *reader, int copyID, ht_stack_T **ht_stack, list_stack_T **list_stack);
extern void rs_timer_close_cb(TimeWatcher *tw, void *data);

bool nvim_eval_ht_foreach_di_tv(hashtab_T *ht, int copyID, ht_stack_T **ht_stack, list_stack_T **list_stack)
{ bool abort = false; HASHTAB_ITER(ht, hi, { abort = abort || rs_set_ref_in_item(&TV_DICT_HI2DI(hi)->di_tv, copyID, ht_stack, list_stack); }); return abort; }

bool nvim_eval_list_foreach_tv(list_T *l, int copyID, ht_stack_T **ht_stack, list_stack_T **list_stack)
{ bool abort = false; TV_LIST_ITER(l, li, { if (abort) { break; } abort = rs_set_ref_in_item(TV_LIST_ITEM_TV(li), copyID, ht_stack, list_stack); }); return abort; }

void nvim_eval_dict_foreach_watcher_callback(dict_T *dd, int copyID, ht_stack_T **ht_stack, list_stack_T **list_stack)
{ QUEUE *w = NULL; DictWatcher *watcher = NULL; QUEUE_FOREACH(w, &dd->watchers, { watcher = tv_dict_watcher_node_data(w); rs_set_ref_in_callback(&watcher->callback, copyID, ht_stack, list_stack); }) }

int nvim_eval_buf_ml_valid(const buf_T *buf) { return buf != NULL && buf->b_ml.ml_mfp != NULL; }
int nvim_eval_buf_line_count(const buf_T *buf) { return buf->b_ml.ml_line_count; }

static char *saved_eval_p_cpo;
void nvim_eval_save_set_cpo(void) { saved_eval_p_cpo = p_cpo; p_cpo = empty_string_option; }
void nvim_eval_restore_cpo(void) { p_cpo = saved_eval_p_cpo; }

void nvim_do_string_sub_restore_cpo_complex(char *save_cpo)
{ if (*p_cpo == NUL) { set_option_value_give_err(kOptCpoptions, CSTR_AS_OPTVAL(save_cpo), 0); } free_string_option(save_cpo); }

bool *eval_lavars_used = NULL;

#include "eval_shim.c.generated.h"

static uint64_t last_timer_id = 1;
static PMap(uint64_t) timers = MAP_INIT;

dict_T *get_v_event(save_v_event_T *sve)
{
  dict_T *v_event = get_vim_var_dict(VV_EVENT);

  if (v_event->dv_hashtab.ht_used > 0) {
    // recursive use of v:event, save, make empty and restore later
    sve->sve_did_save = true;
    sve->sve_hashtab = v_event->dv_hashtab;
    hash_init(&v_event->dv_hashtab);
  } else {
    sve->sve_did_save = false;
  }
  return v_event;
}

void restore_v_event(dict_T *v_event, save_v_event_T *sve)
{
  tv_dict_free_contents(v_event);
  if (sve->sve_did_save) {
    v_event->dv_hashtab = sve->sve_hashtab;
  } else {
    hash_init(&v_event->dv_hashtab);
  }
}

#if defined(EXITFREE)
void eval_clear(void)
{
  evalvars_clear();
  free_scriptnames();  // must come after evalvars_clear().
# ifdef HAVE_WORKING_LIBINTL
  free_locales();
# endif

  // autoloaded script names
  free_autoload_scriptnames();

  // unreferenced lists and dicts
  garbage_collect(false);

  // functions not garbage collected
  free_all_functions();
}

#endif

char *nvim_partial_get_pt_func_uf_name(partial_T *pt) { return pt->pt_func != NULL ? pt->pt_func->uf_name : NULL; }

bool nvim_gc_mark_buffers(int copyID, bool abort)
{ FOR_ALL_BUFFERS(buf) { abort = abort || rs_set_ref_in_item(&buf->b_bufvar.di_tv, copyID, NULL, NULL); abort = abort || rs_set_ref_in_callback(&buf->b_prompt_callback, copyID, NULL, NULL); abort = abort || rs_set_ref_in_callback(&buf->b_prompt_interrupt, copyID, NULL, NULL); abort = abort || rs_set_ref_in_callback(&buf->b_cfu_cb, copyID, NULL, NULL); abort = abort || rs_set_ref_in_callback(&buf->b_ofu_cb, copyID, NULL, NULL); abort = abort || rs_set_ref_in_callback(&buf->b_tsrfu_cb, copyID, NULL, NULL); abort = abort || rs_set_ref_in_callback(&buf->b_tfu_cb, copyID, NULL, NULL); abort = abort || rs_set_ref_in_callback(&buf->b_ffu_cb, copyID, NULL, NULL); if (buf->b_p_cpt_cb != NULL) { abort = abort || set_ref_in_cpt_callbacks(buf->b_p_cpt_cb, buf->b_p_cpt_count, copyID); } } return abort; }

bool nvim_gc_mark_tab_windows(int copyID, bool abort)
{ FOR_ALL_TAB_WINDOWS(tp, wp) { abort = abort || rs_set_ref_in_item(&wp->w_winvar.di_tv, copyID, NULL, NULL); } for (int i = 0; i < AUCMD_WIN_COUNT; i++) { if (aucmd_win[i].auc_win != NULL) { abort = abort || rs_set_ref_in_item(&aucmd_win[i].auc_win->w_winvar.di_tv, copyID, NULL, NULL); } } return abort; }

bool nvim_gc_mark_tabs(int copyID, bool abort)
{ FOR_ALL_TABS(tp) { abort = abort || rs_set_ref_in_item(&tp->tp_winvar.di_tv, copyID, NULL, NULL); } return abort; }

bool nvim_gc_mark_channels(int copyID, bool abort)
{ Channel *data; map_foreach_value(&channels, data, { abort = abort || rs_set_ref_in_callback_reader(&data->on_data, copyID, NULL, NULL); abort = abort || rs_set_ref_in_callback_reader(&data->on_stderr, copyID, NULL, NULL); abort = abort || rs_set_ref_in_callback(&data->on_exit, copyID, NULL, NULL); }) return abort; }

bool nvim_gc_mark_timers(int copyID, bool abort)
{ timer_T *timer; map_foreach_value(&timers, timer, { abort = abort || rs_set_ref_in_callback(&timer->callback, copyID, NULL, NULL); }) return abort; }

void nvim_gc_verb_msg_abort(void)
{
  if (p_verbose > 0) {
    verb_msg(_("Not enough memory to set references, garbage collection aborted!"));
  }
}

bool nvim_callback_call_lua(LuaRef luaref)
{
  Array args = ARRAY_DICT_INIT;
  Object rv = nlua_call_ref(luaref, NULL, args, kRetNilBool, NULL, NULL);
  return LUARET_TRUTHY(rv);
}

partial_T *nvim_get_vlua_partial(void) { return get_vim_var_partial(VV_LUA); }
int nvim_blob_len(const blob_T *b) { return tv_blob_len(b); }
int nvim_blob_get(const blob_T *b, int idx) { return (int)tv_blob_get(b, idx); }

void nvim_blob_ga_clear_and_free(blob_T *b) { if (b != NULL) { ga_clear(&b->bv_ga); xfree(b); } }

void nvim_blob_set_ret(typval_T *tv, blob_T *b) { tv_blob_set_ret(tv, b); }
typval_T *nvim_di_get_tv(dictitem_T *di) { return &di->di_tv; }
evalarg_T *nvim_get_evalarg_evaluate_ptr(void) { return &EVALARG_EVALUATE; }
VarLockStatus nvim_blob_get_bv_lock(const blob_T *blob) { return blob->bv_lock; }

bool nvim_lval_check_tv_lock(const lval_T *lp, const char *name)
{ VarLockStatus lock = lp->ll_newkey == NULL ? lp->ll_tv->v_lock : lp->ll_tv->vval.v_dict->dv_lock; return value_check_lock(lock, name, TV_CSTRING); }

const char *nvim_di_get_key(const dictitem_T *di) { return di->di_key; }
bool nvim_di_check_ro(const dictitem_T *di, const char *name) { return var_check_ro(di->di_flags, name, TV_CSTRING); }
bool nvim_di_check_lock(const dictitem_T *di, const char *name) { return tv_check_lock(&di->di_tv, name, TV_CSTRING); }
bool nvim_tv_dict_is_watched(const dict_T *d) { return tv_dict_is_watched(d); }


void nvim_eval_tv_list_append_owned_tv_ptr(list_T *l, typval_T *tv)
{
  tv->v_lock = VAR_UNLOCKED; tv_list_append_owned_tv(l, *tv);
}

void nvim_eval_tv_list_set_ret(typval_T *rettv, list_T *l) { tv_list_set_ret(rettv, l); }
void nvim_eval_di_set_tv_from_typval(dictitem_T *di, typval_T *tv)
  { di->di_tv = *tv; di->di_tv.v_lock = VAR_UNLOCKED; }
void nvim_eval_tv_dict_set_ret(typval_T *rettv, dict_T *d) { tv_dict_set_ret(rettv, d); }
bool nvim_lval_dict_is_v_or_a_scope(const lval_T *lp) { return lp->ll_dict == get_vimvar_dict() || &lp->ll_dict->dv_hashtab == get_funccal_args_ht(); }
bool nvim_lval_di_check_ro_lock(const lval_T *lp, const char *name, size_t name_len) { return var_check_ro(lp->ll_di->di_flags, name, name_len) || var_check_lock(lp->ll_di->di_flags, name, name_len); }
void nvim_lval_set_tv_to_li_tv(lval_T *lp) { lp->ll_tv = TV_LIST_ITEM_TV(lp->ll_li); }
void nvim_lval_tv_list_alloc_ret(lval_T *lp) { tv_list_alloc_ret(lp->ll_tv, kListLenUnknown); }
void nvim_lval_tv_blob_alloc_ret(lval_T *lp) { tv_blob_alloc_ret(lp->ll_tv); }
listitem_T *nvim_tv_list_check_range_index_one(lval_T *lp, bool quiet) { return tv_list_check_range_index_one(lp->ll_list, &lp->ll_n1, quiet); }
int nvim_tv_list_check_range_index_two(lval_T *lp, bool quiet) { return tv_list_check_range_index_two(lp->ll_list, &lp->ll_n1, lp->ll_li, &lp->ll_n2, quiet); }
bool nvim_partial_get_pt_auto(const partial_T *pt) { return pt->pt_auto; }

bool nvim_lval_dict_scope_check(lval_T *lp, char *key, int len, const typval_T *rettv)
{
  char prevval;
  if (len != -1) {
    prevval = key[len];
    key[len] = NUL;
  } else {
    prevval = 0;
  }
  bool wrong = ((lp->ll_dict->dv_scope == VAR_DEF_SCOPE
                 && tv_is_func(*rettv)
                 && var_wrong_func_name(key, lp->ll_di == NULL))
                || !valid_varname(key));
  if (len != -1) {
    key[len] = prevval;
  }
  return wrong;
}

void nvim_msg_echomsg(const char *str, int hl_id) { msg(str, hl_id); }
int nvim_eap_get_skip_local(const exarg_T *eap) { return eap->skip; }
char *nvim_eap_get_arg_local(const exarg_T *eap) { return eap->arg; }

void nvim_read_cursor_visual_state(NvimCursorVisualState *out)
{ out->cursor_lnum = curwin->w_cursor.lnum; out->cursor_col = curwin->w_cursor.col; out->cursor_coladd = curwin->w_cursor.coladd; out->topline = curwin->w_topline; out->botline = curwin->w_botline; out->visual_active = VIsual_active; out->visual_lnum = VIsual.lnum; out->visual_col = VIsual.col; out->visual_coladd = VIsual.coladd; out->curbuf_fnum = curbuf->b_fnum; }

int nvim_curbuf_fnum(void) { return curbuf->b_fnum; }

bool nvim_mark_get_wrapper(int mname, int32_t *lnum_out, int *col_out, int *coladd_out, int *fnum_out)
{
  const fmark_T *const fm = mark_get(curbuf, curwin, NULL, kMarkAll, mname);
  if (fm == NULL || fm->mark.lnum <= 0) { return false; }
  *lnum_out = fm->mark.lnum; *col_out = fm->mark.col;
  *coladd_out = fm->mark.coladd; *fnum_out = fm->fnum;
  return true;
}

void nvim_update_topline_curwin(void) { update_topline(curwin); }
void nvim_check_cursor_moved_curwin(void) { check_cursor_moved(curwin); }

bool nvim_tv_list_item_is_dollar(list_T *l, int idx)
{
  listitem_T *li = tv_list_find(l, idx);
  return li != NULL && TV_LIST_ITEM_TV(li)->v_type == VAR_STRING
         && TV_LIST_ITEM_TV(li)->vval.v_string != NULL
         && strcmp(TV_LIST_ITEM_TV(li)->vval.v_string, "$") == 0;
}

int nvim_tv_list_len(const list_T *l) { return tv_list_len(l); }
int nvim_mb_charlen_ml(int32_t lnum) { return mb_charlen(ml_get(lnum)); }
int nvim_get_cursor_line_charlen(void) { return mb_charlen(get_cursor_line_ptr()); }


const char *nvim_find_option_var_end(const char **arg, int *opt_idxp, int *opt_flagsp)
{
  OptIndex opt_idx = kOptInvalid; int opt_flags = 0;
  const char *end = find_option_var_end(arg, &opt_idx, &opt_flags);
  *opt_idxp = (int)opt_idx; *opt_flagsp = opt_flags;
  return end;
}

void nvim_get_option_value_as_tv(int opt_idx, int opt_flags, typval_T *rettv)
{ OptVal value = get_option_value((OptIndex)opt_idx, opt_flags); assert(value.type != kOptValTypeNil); *rettv = optval_as_tv(value, true); }

void nvim_get_tty_option_as_tv(const char *name, typval_T *rettv)
{ OptVal value = get_tty_option(name); assert(value.type != kOptValTypeNil); *rettv = optval_as_tv(value, true); }

int nvim_vimconv_get_type(const vimconv_T *conv) { return conv == NULL ? CONV_NONE : (int)conv->vc_type; }
char *nvim_string_convert(const vimconv_T *conv, const char *str) { return string_convert((vimconv_T *)conv, (char *)str, NULL); }
int nvim_tv_list_copyid(const list_T *list) { return tv_list_copyid(list); }
list_T *nvim_tv_list_latest_copy(const list_T *list) { return tv_list_latest_copy(list); }
void nvim_tv_list_ref(list_T *list) { tv_list_ref(list); }
dict_T *nvim_dict_get_copydict(const dict_T *dict) { return dict->dv_copydict; }
listitem_T *nvim_list_first_item(const list_T *l) { return tv_list_first(l); }
const char *nvim_list_item_get_string(listitem_T *item) { return tv_get_string(TV_LIST_ITEM_TV(item)); }

char *nvim_eap_get_cmdline_tofree(exarg_T *eap) { return eap->cmdline_tofree; }
void nvim_eap_set_cmdline_tofree(exarg_T *eap, char *val) { eap->cmdline_tofree = val; }
char *nvim_eap_get_cmdlinep_deref(const exarg_T *eap) { return *eap->cmdlinep; }
void nvim_eap_set_cmdlinep_deref(exarg_T *eap, char *val) { *eap->cmdlinep = val; }
LineGetter nvim_eap_get_getline(const exarg_T *eap) { return eap->ea_getline; }
void *nvim_eap_get_cookie(const exarg_T *eap) { return eap->cookie; }

typval_T *nvim_alloc_typval(void) { return xmalloc(sizeof(typval_T)); }
void nvim_set_var_wrapper(const char *name, size_t name_len, typval_T *tv) { set_var(name, name_len, tv, false); }
void nvim_set_vim_var_argv_list(list_T *list) { set_vim_var_list(VV_ARGV, list); }
const char *nvim_sourcing_name_get(void) { return SOURCING_NAME; }
linenr_T nvim_sourcing_lnum_get(void) { return SOURCING_LNUM; }

void nvim_tv_list_set_lock(list_T *l, int lock) { tv_list_set_lock(l, (VarLockStatus)lock); }
void nvim_tv_list_last_fix_lock(list_T *l) { TV_LIST_ITEM_TV(tv_list_last(l))->v_lock = VAR_FIXED; }

void nvim_read_prompt_state(NvimPromptState *out)
{ out->curbuf = curbuf; out->ml_line_count = (int32_t)curbuf->b_ml.ml_line_count; out->prompt_start_lnum = (int32_t)curbuf->b_prompt_start.mark.lnum; out->prompt_callback = &curbuf->b_prompt_callback; out->prompt_interrupt = &curbuf->b_prompt_interrupt; }

void nvim_write_prompt_start_lnum(int32_t lnum) { curbuf->b_prompt_start.mark.lnum = (linenr_T)lnum; }
linenr_T nvim_buf_get_prompt_start_lnum(buf_T *buf) { return buf->b_prompt_start.mark.lnum; }
void nvim_appended_lines_mark(linenr_T lnum, int count) { appended_lines_mark(lnum, count); }
void nvim_curbuf_u_clearallandblockfree(void) { u_clearallandblockfree(curbuf); }

void nvim_read_fold_eval_state(win_T *wp, NvimFoldEvalState *out)
{ out->insecure_foldexpr = was_set_insecurely(wp, kOptFoldexpr, OPT_LOCAL); out->insecure_foldtext = was_set_insecurely(wp, kOptFoldtext, OPT_LOCAL); out->foldexpr = skipwhite(wp->w_p_fde); out->foldtext = wp->w_p_fdt; }

sctx_T *nvim_fold_sctx_save_and_set(win_T *wp)
{ sctx_T *saved = xmalloc(sizeof(sctx_T)); *saved = current_sctx; current_sctx = wp->w_p_script_ctx[kWinOptFoldexpr]; return saved; }

void nvim_restore_current_sctx(sctx_T *saved) { current_sctx = *saved; xfree(saved); }

void nvim_foldtext_make_obj(typval_T *tv, int tv_type, Object *out)
{
  if (tv == NULL) {
    *out = STRING_OBJ(NULL_STRING);
  } else if (tv_type == VAR_LIST) {
    *out = vim_to_object(tv, NULL, false);
  } else {
    *out = STRING_OBJ(cstr_to_string(tv_get_string(tv)));
  }
}

void nvim_set_shell_error(int status) { set_vim_var_nr(VV_SHELL_ERROR, (varnumber_T)status); }
void nvim_smsg_system_cmd(const char *cmdstr) { smsg(0, _("Executing command: \"%s\""), cmdstr); }
bool nvim_do_profiling_active(void) { return do_profiling == PROF_YES; }
bool nvim_eval_os_can_exe(const char *name, char **abspath) { return os_can_exe(name, abspath, true); }
int nvim_eval_variable(const char *name, int len, typval_T *rettv, bool verbose,
                       bool import_script) { return eval_variable(name, len, rettv, NULL, verbose, import_script); }


void *nvim_save_provider_caller_scope(void)
{
  struct caller_scope *saved = xmalloc(sizeof(struct caller_scope));
  *saved = provider_caller_scope;
  provider_caller_scope = (struct caller_scope){
    .script_ctx = current_sctx,
    .es_entry = ((estack_T *)exestack.ga_data)[exestack.ga_len - 1],
    .autocmd_fname = autocmd_fname,
    .autocmd_match = autocmd_match,
    .autocmd_fname_full = autocmd_fname_full,
    .autocmd_bufnr = autocmd_bufnr,
    .funccalp = (void *)get_current_funccal()
  };
  return saved;
}

void nvim_restore_provider_caller_scope(void *saved) { provider_caller_scope = *(struct caller_scope *)saved; xfree(saved); }
timer_T *nvim_timer_alloc(void) { return xcalloc(1, sizeof(timer_T)); }
void nvim_timer_free(timer_T *timer) { xfree(timer); }

void nvim_timer_read_fields(const timer_T *timer, NvimTimerFields *out)
{ out->timer_id = timer->timer_id; out->repeat_count = timer->repeat_count; out->refcount = timer->refcount; out->emsg_count = timer->emsg_count; out->timeout = timer->timeout; out->stopped = timer->stopped; out->paused = timer->paused; }
void nvim_timer_write_fields(timer_T *timer, const NvimTimerFields *fields)
{ timer->timer_id = fields->timer_id; timer->repeat_count = fields->repeat_count; timer->refcount = fields->refcount; timer->emsg_count = fields->emsg_count; timer->timeout = fields->timeout; timer->stopped = fields->stopped; timer->paused = fields->paused; }

Callback *nvim_timer_get_callback_ptr(timer_T *timer) { return &timer->callback; }
void nvim_timer_set_callback(timer_T *timer, const Callback *cb) { timer->callback = *cb; }
void nvim_timer_tw_init(timer_T *timer) { time_watcher_init(&main_loop, &timer->tw, timer); }
void nvim_timer_tw_start(timer_T *timer, uint64_t timeout, uint64_t repeat) { time_watcher_start(&timer->tw, timer_due_cb, timeout, repeat); }
void nvim_timer_tw_stop(timer_T *timer) { time_watcher_stop(&timer->tw); }
void nvim_timer_tw_close(timer_T *timer) { time_watcher_close(&timer->tw, rs_timer_close_cb); }
void nvim_timer_tw_set_events_child(timer_T *timer) { timer->tw.events = multiqueue_new_child(rs_loop_get_events(&main_loop)); }
void nvim_timer_tw_set_blockable(timer_T *timer, int blockable) { timer->tw.blockable = blockable != 0; }
void nvim_timer_tw_free_events(timer_T *timer) { multiqueue_free(timer->tw.events); }

timer_T *nvim_timers_get(int64_t id) { return pmap_get(uint64_t)(&timers, (uint64_t)id); }
void nvim_timers_put(timer_T *timer) { pmap_put(uint64_t)(&timers, (uint64_t)timer->timer_id, timer); }
void nvim_timers_del(int64_t id) { pmap_del(uint64_t)(&timers, (uint64_t)id, NULL); }
size_t nvim_timers_size(void) { return map_size(&timers); }
uint64_t nvim_timers_next_id(void) { return last_timer_id++; }

void nvim_timers_foreach(void (*cb)(timer_T *, void *), void *userdata)
{ timer_T *timer; map_foreach_value(&timers, timer, { cb(timer, userdata); }) }

int nvim_get_pressedreturn(void) { extern int get_pressedreturn(void); return get_pressedreturn() ? 1 : 0; }
void nvim_set_pressedreturn(int val) { extern void set_pressedreturn(bool); set_pressedreturn(val != 0); }

int nvim_channel_is_valid_job(Channel *chan) { return chan != NULL && chan->streamtype == kChannelStreamProc && !proc_is_stopped(&chan->stream.proc); }

int nvim_channel_is_not_proc(Channel *chan) { return (chan != NULL && chan->streamtype != kChannelStreamProc) ? 1 : 0; }
Channel *nvim_find_channel(uint64_t id) { return find_channel(id); }
void nvim_channels_del(uint64_t id) { pmap_del(uint64_t)(&channels, id, NULL); }

char *nvim_docmd_fmt_exception_not_caught(const char *value)
{ vim_snprintf(IObuff, IOSIZE, _("E605: Exception not caught: %s"), value); return xstrdup(IObuff); }

void nvim_msg_multiline_cstr(const char *s, int hl_id, bool check_int, bool hist, bool *need_clear) { msg_multiline(cstr_as_string(s), hl_id, check_int, hist, need_clear); }

// Phase 6 (channel): stream sub-member accessors for channel_close / channel_send in Rust

#include <stdio.h>
#include "nvim/event/rstream.h"
#include "nvim/event/stream.h"
#include "nvim/os/pty_proc.h"
#include "nvim/msgpack_rpc/channel.h"

// stderr accessor for Rust channel_close (stderr is a macro in C)
void *nvim_get_stderr(void) { return (void *)stderr; }

// rstream_may_close on socket stream
void nvim_chan_socket_rstream_may_close(Channel *chan) { rstream_may_close(&chan->stream.socket); }
// stream_may_close / rstream_may_close on proc sub-streams
void nvim_chan_proc_stream_may_close_in(Channel *chan) { stream_may_close(&chan->stream.proc.in); }
void nvim_chan_proc_rstream_may_close_out(Channel *chan) { rstream_may_close(&chan->stream.proc.out); }
void nvim_chan_proc_rstream_may_close_err(Channel *chan) { rstream_may_close(&chan->stream.proc.err); }
// pty_proc_close_master on pty stream
void nvim_chan_pty_close_master(Channel *chan) { pty_proc_close_master(&chan->stream.pty); }
// stream accessors for stdio
void nvim_chan_stdio_rstream_may_close_in(Channel *chan) { rstream_may_close(&chan->stream.stdio.in); }
void nvim_chan_stdio_stream_may_close_out(Channel *chan) { stream_may_close(&chan->stream.stdio.out); }
// stderr stream err.closed field
int nvim_chan_get_err_closed(Channel *chan) { return chan->stream.err.closed ? 1 : 0; }
void nvim_chan_set_err_closed(Channel *chan, int v) { chan->stream.err.closed = v != 0; }
// internal stream field accessors
int nvim_chan_get_internal_cb(Channel *chan) { return chan->stream.internal.cb; }
void nvim_chan_set_internal_cb(Channel *chan, int v) { chan->stream.internal.cb = (LuaRef)v; }
int nvim_chan_get_internal_closed(Channel *chan) { return chan->stream.internal.closed ? 1 : 0; }
void nvim_chan_set_internal_closed(Channel *chan, int v) { chan->stream.internal.closed = v != 0; }
// proc type accessor (kProcTypePty detection)
int nvim_chan_get_proc_type(Channel *chan) { return (int)chan->stream.proc.type; }
// rpc_close wrapper (rpc_close is declared in nvim/msgpack_rpc/channel.h, included via channel.h)
void nvim_chan_rpc_close(Channel *chan) { rpc_close(chan); }
// api_free_luaref wrapper
void nvim_chan_api_free_luaref(int ref) { api_free_luaref((LuaRef)ref); }

// channel_send error strings (translated)
#include "nvim/gettext_defs.h"
const char *nvim_chan_send_err_invchan(void) { return _(e_invchan); }
const char *nvim_chan_send_err_closed_stream(void) { return _("Can't send data to closed stream"); }
const char *nvim_chan_send_err_raw_rpc(void) { return _("Can't send raw data to rpc channel"); }

// channel_send helpers: instream pointer and os_write wrapper
Stream *nvim_chan_instream(Channel *chan)
{
  switch (chan->streamtype) {
  case kChannelStreamProc:   return &chan->stream.proc.in;
  case kChannelStreamSocket: return &chan->stream.socket.s;
  case kChannelStreamStdio:  return &chan->stream.stdio.out;
  default: abort();
  }
}

// channel_teardown helper: iterate channels, close all
void nvim_chan_foreach_close_all(void)
{
  Channel *chan;
  map_foreach_value(&channels, chan, {
    channel_close(chan->id, kChannelPartAll, NULL);
  });
}

// channel_free_all_mem helper: destroy all channels, clear map, free on_print
#ifdef EXITFREE
void nvim_chan_foreach_destroy_all(void)
{
  Channel *chan;
  map_foreach_value(&channels, chan, {
    channel_destroy(chan);
  });
  map_destroy(uint64_t, &channels);
  callback_free(&on_print);
}
#endif

// Phase 6f: tv_dict_remove accessors (Rust FFI helpers)
bool nvim_di_check_fixed_translate(const dictitem_T *di, const char *name) { return var_check_fixed(di->di_flags, name, TV_TRANSLATE); }
bool nvim_di_check_ro_translate(const dictitem_T *di, const char *name) { return var_check_ro(di->di_flags, name, TV_TRANSLATE); }
void nvim_dictitem_move_tv_to_rettv(typval_T *rettv, dictitem_T *di) { *rettv = di->di_tv; di->di_tv = TV_INITIAL_VALUE; }
void nvim_semsg_dictkey(const char *key) { semsg(_(e_dictkey), key); }
void nvim_semsg_toomanyarg(const char *fname) { semsg(_(e_toomanyarg), fname); }

// Phase 7 (channel migration): accessors for channel_create_event, channel_from_connection, channel_all_info

#include "nvim/api/private/helpers.h"
#include "nvim/event/socket.h"
#include "nvim/event/wstream.h"
#include "nvim/event/rstream.h"

/// Accept a socket connection from watcher into chan->stream.socket, then
/// set close_cb/data and init streams. Called from Rust channel_from_connection.
void nvim_socket_watcher_accept_and_init(SocketWatcher *watcher, Channel *chan)
{
  socket_watcher_accept(watcher, &chan->stream.socket);
  chan->stream.socket.s.internal_close_cb = close_cb;
  chan->stream.socket.s.internal_data = chan;
  wstream_init(&chan->stream.socket.s, 0);
  rstream_init(&chan->stream.socket);
}

/// Wrap rpc_start for Rust (rpc_start is declared in msgpack_rpc/channel.h).
void nvim_chan_rpc_start(Channel *chan) { rpc_start(chan); }

/// Return the number of entries in the channels map.
size_t nvim_chan_map_size(void) { return map_size(&channels); }

/// Fill `out[0..cap]` with channel IDs (as uint64_t). Returns the count stored.
size_t nvim_chan_map_collect_ids(uint64_t *out, size_t cap)
{
  size_t n = 0;
  uint64_t id;
  map_foreach_key(&channels, id, {
    if (n < cap) {
      out[n++] = id;
    }
  });
  return n;
}

/// Return an arena-allocated Array of the given max_size.
Array nvim_arena_array(Arena *arena, size_t max_size)
{
  return arena_array(arena, max_size);
}

/// Call channel_info(id, arena) and wrap result in DICT_OBJ.
/// Returns an Object of type Dict containing the channel info.
Object nvim_chan_info_as_object(uint64_t id, Arena *arena)
{
  Dict info = channel_info(id, arena);
  return DICT_OBJ(info);
}

// Phase 7b (channel migration): accessors for channel_info, set_info_event

#include "nvim/os/pty_proc.h"
#include "nvim/terminal.h"
#include "nvim/api/private/converter.h"
#include "nvim/autocmd.h"

/// Return chan->stream.proc.argv (char **).
char **nvim_chan_proc_get_argv(Channel *chan) { return chan->stream.proc.argv; }

/// Return pty_proc_tty_name(&chan->stream.pty) (const char *).
const char *nvim_chan_pty_tty_name(Channel *chan) { return pty_proc_tty_name(&chan->stream.pty); }

/// Return terminal_buf(chan->term) as int (Buffer handle).
int nvim_terminal_buf_id(Channel *chan) { return (int)terminal_buf(chan->term); }

/// Return chan->rpc.info (Dict, by value).
Dict nvim_chan_get_rpc_info(Channel *chan) { return chan->rpc.info; }

/// arena_dict(arena, max_size) wrapper.
Dict nvim_arena_dict(Arena *arena, size_t max_size) { return arena_dict(arena, max_size); }

/// Build an arena-allocated CSTR_TO_ARENA_STR (copies s into arena, returns String).
/// Caller wraps in STRING_OBJ.
String nvim_arena_cstr_to_str(Arena *arena, const char *s)
{
  return CSTR_TO_ARENA_STR(arena, s);
}

/// Convert a Dict to v:event["info"] (typval dict) and mark readonly.
/// Calls object_to_vim(DICT_OBJ(info), ...) + tv_dict_add_dict + tv_dict_set_keys_readonly.
void nvim_chan_set_v_event_info(dict_T *v_event, Dict *info_dict)
{
  typval_T retval;
  object_to_vim(DICT_OBJ(*info_dict), &retval, NULL);
  assert(retval.v_type == VAR_DICT);
  tv_dict_add_dict(v_event, S_LEN("info"), retval.vval.v_dict);
  tv_dict_set_keys_readonly(v_event);
}

/// apply_autocmds(event, NULL, NULL, true, curbuf) wrapper (force=true variant for channel events).
void nvim_chan_apply_autocmds_event(int event)
{
  apply_autocmds((event_T)event, NULL, NULL, true, curbuf);
}

/// arena_finish + arena_mem_free wrapper (called from Rust set_info_event).
void nvim_chan_arena_finish_and_free(Arena *arena) { arena_mem_free(arena_finish(arena)); }

// Phase 7c (channel migration): accessors for channel_connect, channel_from_stdio

#include <fcntl.h>
#include "nvim/msgpack_rpc/server.h"
#include "nvim/os/os_defs.h"
#ifdef MSWIN
# include "nvim/os/fs.h"
# include "nvim/os/pty_conpty_win.h"
#endif

extern size_t on_channel_data(RStream *stream, const char *buf, size_t count, void *data, bool eof);

/// socket_connect wrapper: connect chan->stream.socket. Returns 0 on success, nonzero on error.
bool nvim_chan_socket_connect(Channel *chan, bool tcp, const char *address, int timeout,
                              const char **error)
{
  return socket_connect(&main_loop, &chan->stream.socket, tcp, address, timeout, error);
}

/// Set close_cb and internal_data on chan->stream.socket.s.
void nvim_chan_set_socket_close_cb_and_data(Channel *chan)
{
  chan->stream.socket.s.internal_close_cb = close_cb;
  chan->stream.socket.s.internal_data = chan;
}

/// wstream_init for the socket stream (write side).
void nvim_chan_socket_wstream_init(Channel *chan)
{
  wstream_init(&chan->stream.socket.s, 0);
}

/// rstream_init for the socket stream (read side).
void nvim_chan_socket_rstream_init(Channel *chan)
{
  rstream_init(&chan->stream.socket);
}

/// rstream_start on socket with on_channel_data callback.
void nvim_chan_socket_rstream_start_data(Channel *chan)
{
  rstream_start(&chan->stream.socket, on_channel_data, chan);
}

/// rstream_init_fd for chan->stream.stdio.in (stdin fd).
void nvim_chan_stdio_rstream_init_fd(Channel *chan, int fd)
{
  rstream_init_fd(&main_loop, &chan->stream.stdio.in, fd);
}

/// wstream_init_fd for chan->stream.stdio.out (stdout fd).
void nvim_chan_stdio_wstream_init_fd(Channel *chan, int fd)
{
  wstream_init_fd(&main_loop, &chan->stream.stdio.out, fd, 0);
}

/// rstream_start on stdio.in with on_channel_data callback.
void nvim_chan_stdio_rstream_start_data(Channel *chan)
{
  rstream_start(&chan->stream.stdio.in, on_channel_data, chan);
}

/// Translated error strings for channel_from_stdio.
const char *nvim_chan_from_stdio_err_headless(void) { return _("can only be opened in headless mode"); }
const char *nvim_chan_from_stdio_err_already_open(void) { return _("channel was already open"); }

/// Platform-specific fd duplication for channel_from_stdio.
/// Returns stdin_dup_fd in *p_stdin and stdout_dup_fd in *p_stdout.
void nvim_chan_from_stdio_dup_fds(int *p_stdin, int *p_stdout)
{
  *p_stdin = STDIN_FILENO;
  *p_stdout = STDOUT_FILENO;
#ifdef MSWIN
  if (embedded_mode && os_has_conpty_working()) {
    *p_stdin = os_dup(STDIN_FILENO);
    os_replace_stdin_to_conin();
    *p_stdout = os_dup(STDOUT_FILENO);
    os_replace_stdout_and_stderr_to_conout();
  }
#else
  if (embedded_mode) {
    *p_stdin = fcntl(STDIN_FILENO, F_DUPFD_CLOEXEC, STDERR_FILENO + 1);
    *p_stdout = fcntl(STDOUT_FILENO, F_DUPFD_CLOEXEC, STDERR_FILENO + 1);
    dup2(STDERR_FILENO, STDOUT_FILENO);
    dup2(STDERR_FILENO, STDIN_FILENO);
  }
#endif
}

// Phase 8 (channel migration): accessors for channel_job_start

#include "nvim/os/pty_proc.h"
#include "nvim/event/libuv_proc.h"
#include "nvim/os/shell.h"
#include "nvim/errors.h"
#include "nvim/os/fs.h"

extern size_t on_job_stderr(RStream *stream, const char *buf, size_t count, void *data, bool eof);
extern void channel_proc_exit_cb(Proc *proc, int status, void *data);
extern void rs_proc_set_cb(Proc *proc, void *cb);

/// pty_proc_init wrapper: initialise chan->stream.pty and return Proc * for the chan.
void nvim_chan_pty_proc_init(Channel *chan)
{
  chan->stream.pty = pty_proc_init(&main_loop, chan);
}

/// libuv_proc_init wrapper: initialise chan->stream.uv and return Proc * for the chan.
void nvim_chan_libuv_proc_init(Channel *chan)
{
  chan->stream.uv = libuv_proc_init(&main_loop, chan);
}

/// Set chan->stream.pty.width.
void nvim_chan_set_pty_width(Channel *chan, uint16_t w)
{
  chan->stream.pty.width = w;
}

/// Set chan->stream.pty.height.
void nvim_chan_set_pty_height(Channel *chan, uint16_t h)
{
  chan->stream.pty.height = h;
}

/// Return &chan->stream.proc (pointer to the Proc union member).
Proc *nvim_chan_get_proc_ptr(Channel *chan)
{
  return &chan->stream.proc;
}

/// proc_spawn wrapper.
int nvim_proc_spawn(Proc *proc, bool has_in, bool has_out, bool has_err)
{
  return proc_spawn(proc, has_in, has_out, has_err);
}

/// Set the proc exit callback to channel_proc_exit_cb.
void nvim_chan_proc_set_exit_cb(Channel *chan)
{
  rs_proc_set_cb(&chan->stream.proc, (void *)channel_proc_exit_cb);
}

/// wstream_init for proc->in (stdin pipe).
void nvim_chan_proc_wstream_init_in(Channel *chan)
{
  wstream_init(&chan->stream.proc.in, 0);
}

/// rstream_init for proc->out (stdout).
void nvim_chan_proc_rstream_init_out(Channel *chan)
{
  rstream_init(&chan->stream.proc.out);
}

/// rstream_start on proc->out with on_channel_data callback.
void nvim_chan_proc_rstream_start_data(Channel *chan)
{
  rstream_start(&chan->stream.proc.out, on_channel_data, chan);
}

/// rstream_init for proc->err (stderr).
void nvim_chan_proc_rstream_init_err(Channel *chan)
{
  rstream_init(&chan->stream.proc.err);
}

/// rstream_start on proc->err with on_job_stderr callback.
void nvim_chan_proc_rstream_start_stderr(Channel *chan)
{
  rstream_start(&chan->stream.proc.err, on_job_stderr, chan);
}

/// semsg(_(e_invarg2), msg) for PTY detach error.
void nvim_chan_job_semsg_invarg2(const char *msg)
{
  semsg(_(e_invarg2), msg);
}

/// semsg(_(e_jobspawn), os_strerror(status), cmd) for spawn failure.
void nvim_chan_job_semsg_spawn_error(int status, const char *cmd)
{
  semsg(_(e_jobspawn), os_strerror(status), cmd);
}
