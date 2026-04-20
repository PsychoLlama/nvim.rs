// buffer_shim.c: C accessor wrappers for the Rust buffer crate (nvim-buffer).
//
// These thin wrappers provide a stable C ABI for Rust code to call into
// Neovim's C internals. Each function is called from one or more Rust
// modules in src/nvim-rs/buffer/.

#include <inttypes.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/private/helpers.h"
#include "nvim/arglist.h"
#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/extmark_defs.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/extmark.h"
#include "nvim/garray.h"
#include "nvim/hashtab.h"
#include "nvim/help.h"
#include "nvim/indent.h"
#include "nvim/indent_c.h"
#include "nvim/mapping.h"
#include "nvim/usercmd.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/memfile.h"
#include "nvim/memline.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/os/fs.h"
#include "nvim/os/os.h"
#include "nvim/os/time.h"
#include "nvim/path.h"
#include "nvim/regexp.h"
#include "nvim/spell.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/terminal.h"
#include "nvim/types_defs.h"
#include "nvim/syntax.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/channel.h"
#include "nvim/change.h"
#include "nvim/eval.h"
#include "nvim/eval/vars.h"
#include "nvim/buffer_updates.h"
#include "nvim/cursor.h"
#include "nvim/ex_docmd.h"
#include "nvim/memory.h"
#include "nvim/move.h"
#include "nvim/window.h"
#include "nvim/ex_eval.h"
#include "nvim/digraph.h"
#include "nvim/optionstr.h"

#include "buffer_shim.c.generated.h"

// Rust-exported fold/window helpers
extern void rs_cloneFoldGrowArray(garray_T *from, garray_T *to);
extern void rs_clearFolding(win_T *win);
extern void rs_foldUpdateAll(win_T *win);
// buffer.c non-static helpers
extern void free_buffer(buf_T *buf);
extern void free_buffer_stuff(buf_T *buf, int free_flags);
// Rust buffer-lifecycle helpers
extern int rs_do_buffer_ext(int action, int start, int dir, int count, int flags);
// Rust open_buffer (migrated from buffer.c)
extern int open_buffer(bool read_stdin, exarg_T *eap, int flags_arg);
// Rust read_buffer (migrated from buffer.c)
extern int rs_read_buffer(bool read_stdin, exarg_T *eap, int flags);
// getout declared in main.h.generated.h (included via buffer_shim.c.generated.h indirectly)
extern void getout(int exitval);

int nvim_buf_get_help(buf_T *buf) { return buf->b_help; }
int nvim_buf_get_terminal(buf_T *buf) { return buf->terminal != NULL; }
buf_T *nvim_get_lastbuf(void) { return lastbuf; }
buf_T *nvim_bufref_get_buf(bufref_T *bufref) { return bufref->br_buf; }
uint32_t nvim_buf_meta_total_sign_hl(buf_T *buf) { return buf ? buf_meta_total(buf, kMTMetaSignHL) : 0; }
uint32_t nvim_buf_meta_total_sign_text(buf_T *buf) { return buf ? buf_meta_total(buf, kMTMetaSignText) : 0; }
int nvim_bufref_get_fnum(bufref_T *bufref) { return bufref->br_fnum; }
int nvim_bufref_get_buf_free_count(bufref_T *bufref) { return bufref->br_buf_free_count; }
int nvim_buf_get_fnum(buf_T *buf) { return buf->b_fnum; }
const char *nvim_buf_get_b_fname(buf_T *buf) { return buf->b_fname; }
const char *nvim_buf_get_b_ffname(buf_T *buf) { return buf->b_ffname; }
int nvim_buf_get_b_p_ro(buf_T *buf) { return buf->b_p_ro; }
int nvim_get_cmdmod_cmod_flags(void) { return cmdmod.cmod_flags; }
uint64_t *nvim_buf_get_chartab(buf_T *buf) { return buf->b_chartab; }
int nvim_buf_get_nwindows(buf_T *buf) { return buf->b_nwindows; }
int nvim_buf_get_locked(buf_T *buf) { return buf->b_locked; }
const char *nvim_curbuf_get_ffname(void) { return curbuf->b_ffname; }
int nvim_curbuf_get_handle(void) { return curbuf->handle; }
int nvim_curbuf_has_ffname(void) { return curbuf->b_ffname != NULL ? 1 : 0; }
int nvim_curbuf_b_next_null(void) { return curbuf->b_next == NULL ? 1 : 0; }
const char *nvim_curbuf_get_path(void) { return curbuf->b_p_path; }
const char *nvim_curbuf_get_inex(void) { return curbuf->b_p_inex; }
char *nvim_get_namebuff(void) { return NameBuff; }
const char *nvim_curbuf_get_line_ptr(void) { return ml_get_buf(curbuf, curwin->w_cursor.lnum); }
void nvim_buf_set_name_body(buf_T *buf, char *name)
{
  if (buf->b_sfname != buf->b_ffname) {
    xfree(buf->b_sfname);
  }
  xfree(buf->b_ffname);
  buf->b_ffname = xstrdup(name);
  buf->b_sfname = NULL;
  fname_expand(buf, &buf->b_ffname, &buf->b_sfname);
  buf->b_fname = buf->b_sfname;
}
void nvim_check_arg_idx_if_curbuf(buf_T *buf)
{ if (curwin->w_buffer == buf) { check_arg_idx(curwin); } }
linenr_T nvim_buflist_findlnum(buf_T *buf) { return buflist_findfmark(buf)->mark.lnum; }
int nvim_get_argcount(void) { return ARGCOUNT; }
buf_T *nvim_get_cmdwin_buf(void) { return cmdwin_buf; }
int64_t nvim_buf_get_changedtick_direct(buf_T *buf) { return buf_get_changedtick(buf); }
int nvim_curwin_get_alt_fnum(void) { return curwin->w_alt_fnum; }
buf_T *nvim_handle_get_buffer(handle_T handle) { return handle_get_buffer(handle); }
int nvim_buf_terminal_running(buf_T *buf)
{ return (buf && buf->terminal && terminal_running(buf->terminal)) ? 1 : 0; }
int nvim_buf_channel_job_running(buf_T *buf)
{ return (buf && buf->terminal && channel_job_running((uint64_t)buf->b_p_channel)) ? 1 : 0; }
const char *nvim_curbuf_get_fname(void) { return curbuf->b_fname; }

/// Wrapper for try_getdigits: parses digits at s, sets *vers, returns bytes consumed.
/// Returns -1 on failure (no digits parsed).
int nvim_try_getdigits(const char *s, int64_t *vers)
{
  char *p = (char *)s;
  intmax_t v = 0;
  if (!try_getdigits(&p, &v)) {
    return -1;
  }
  *vers = (int64_t)v;
  return (int)(p - s);
}

void *nvim_blfp_regex_compile(const char *pat, int magic)
{ regmatch_T *rmp = xmalloc(sizeof(regmatch_T)); rmp->regprog = vim_regcomp((char *)pat, magic); return rmp; }

void *nvim_bufname_regex_compile(char *pat)
{
  regmatch_T *rmp = xmalloc(sizeof(regmatch_T));
  rmp->regprog = vim_regcomp(pat, RE_MAGIC);
  if (rmp->regprog == NULL) { xfree(rmp); return NULL; }
  return rmp;
}

int nvim_bufname_regex_valid(void *handle)
{ return handle != NULL && ((regmatch_T *)handle)->regprog != NULL ? 1 : 0; }
void nvim_bufname_regex_free(void *handle)
{ if (handle == NULL) { return; } vim_regfree(((regmatch_T *)handle)->regprog); xfree(handle); }
int nvim_curwin_get_p_diff(void) { return curwin->w_p_diff ? 1 : 0; }
int nvim_curbuf_ml_line_count(void) { return curbuf->b_ml.ml_line_count; }
bool nvim_get_curbuf_b_u_synced(void) { return curbuf->b_u_synced; }
bool nvim_curbuf_has_b_p_fex(void) { return *curbuf->b_p_fex != NUL; }

// buf_T option field offset table (indexed by OptIndex, -1 = unhandled)
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
void nvim_buf_clear_b_p_script_ctx(buf_T *buf) { CLEAR_FIELD(buf->b_p_script_ctx); }
char *nvim_buf_save_and_clear_b_p_isk(buf_T *buf)
{ char *saved = buf->b_p_isk; buf->b_p_isk = NULL; return saved; }
void nvim_buf_restore_b_p_isk(buf_T *buf, char *saved) { buf->b_p_isk = saved; }
void nvim_call_compile_cap_prog_buf(buf_T *buf) { compile_cap_prog(&buf->b_s); }
void nvim_call_tabstop_set_vsts(buf_T *buf, const char *str) { tabstop_set(str, &buf->b_p_vsts_array); }
void nvim_call_tabstop_set_vts(buf_T *buf, const char *str) { tabstop_set(str, &buf->b_p_vts_array); }
void nvim_buf_kmap_state_set_init(buf_T *buf) { buf->b_kmap_state |= KEYMAP_INIT; }
void nvim_buf_set_string_field(buf_T *buf, ptrdiff_t offset, const char *s)
{ *(char **)(((char *)buf) + offset) = xstrdup(s); }
void nvim_buf_empty_string_field(buf_T *buf, ptrdiff_t offset)
{ *(char **)(((char *)buf) + offset) = empty_string_option; }
void nvim_buf_set_bool_field(buf_T *buf, ptrdiff_t offset, int val) { *(int *)(((char *)buf) + offset) = val != 0; }
void nvim_buf_set_optint_field(buf_T *buf, ptrdiff_t offset, OptInt val) { *(OptInt *)(((char *)buf) + offset) = val; }
OptInt nvim_buf_get_optint_field(buf_T *buf, ptrdiff_t offset) { return *(OptInt *)(((char *)buf) + offset); }
int nvim_buf_get_bool_field(buf_T *buf, ptrdiff_t offset) { return (int)(*(bool *)(((char *)buf) + offset)); }
void nvim_buf_set_b_p_fenc_dup(buf_T *buf) { buf->b_p_fenc = xstrdup(p_fenc); }
void nvim_buf_set_b_s_syn_isk_empty(buf_T *buf) { buf->b_s.b_syn_isk = empty_string_option; }
void nvim_buf_set_b_p_vsts_nopaste_dup(buf_T *buf, const char *s) { buf->b_p_vsts_nopaste = s ? xstrdup(s) : NULL; }
void nvim_buf_set_b_s_spc_dup(buf_T *buf, const char *s) { buf->b_s.b_p_spc = xstrdup(s); }
void nvim_buf_set_b_s_spf_dup(buf_T *buf, const char *s) { buf->b_s.b_p_spf = xstrdup(s); }
void nvim_buf_set_b_s_spl_dup(buf_T *buf, const char *s) { buf->b_s.b_p_spl = xstrdup(s); }
void nvim_buf_set_b_s_spo_dup(buf_T *buf, const char *s) { buf->b_s.b_p_spo = xstrdup(s); }
void nvim_buf_set_b_s_spo_flags_from_global(buf_T *buf) { buf->b_s.b_p_spo_flags = spo_flags; }
MarkTree *nvim_buf_get_marktree(buf_T *buf) { return buf->b_marktree; }
bool nvim_buf_signcols_get_autom(buf_T *buf) { return buf->b_signcols.autom; }
void nvim_buf_signcols_set_autom(buf_T *buf, bool val) { buf->b_signcols.autom = val; }
int nvim_buf_signcols_get_max(buf_T *buf) { return buf->b_signcols.max; }
void nvim_buf_signcols_set_max(buf_T *buf, int val) { buf->b_signcols.max = val; }
int nvim_buf_signcols_get_last_max(buf_T *buf) { return buf->b_signcols.last_max; }
int nvim_buf_signcols_get_count(buf_T *buf, int idx) { return buf->b_signcols.count[idx]; }
void nvim_buf_signcols_clear(buf_T *buf) { buf->b_signcols.max = 0; CLEAR_FIELD(buf->b_signcols.count); }
size_t nvim_buf_wininfo_count(buf_T *buf) { return kv_size(buf->b_wininfo); }
WinInfo *nvim_buf_wininfo_get(buf_T *buf, size_t i) { return kv_A(buf->b_wininfo, i); }
win_T *nvim_wininfo_get_win(WinInfo *wip) { return wip->wi_win; }
bool nvim_wininfo_get_optset(WinInfo *wip) { return wip->wi_optset; }
bool nvim_wininfo_get_wo_diff(WinInfo *wip) { return wip->wi_opt.wo_diff; }
int nvim_wininfo_get_changelistidx(WinInfo *wip) { return wip->wi_changelistidx; }
fmark_T *nvim_wininfo_get_mark_ptr(WinInfo *wip) { return &wip->wi_mark; }
bool nvim_wininfo_win_in_curtab(WinInfo *wip)
{ FOR_ALL_WINDOWS_IN_TAB(wp, curtab) { if (wip->wi_win == wp) { return true; } } return false; }
WinInfo *nvim_buf_wininfo_find_and_detach(buf_T *buf, win_T *win, bool copy_options,
                                          linenr_T *lnum_inout)
{
  for (size_t i = 0; i < kv_size(buf->b_wininfo); i++) {
    WinInfo *wip = kv_A(buf->b_wininfo, i);
    if (wip->wi_win == win) {
      kv_shift(buf->b_wininfo, i, 1);
      if (copy_options && wip->wi_optset) {
        clear_winopt(&wip->wi_opt);
        deleteFoldRecurse(buf, &wip->wi_folds);
      }
      return wip;
    }
  }
  // Not found: allocate new entry
  WinInfo *wip = xcalloc(1, sizeof(WinInfo));
  wip->wi_win = win;
  if (*lnum_inout == 0) {
    *lnum_inout = 1;
  }
  return wip;
}

void nvim_buf_wininfo_prepend(buf_T *buf, WinInfo *wip)
{ kv_pushp(buf->b_wininfo); memmove(&kv_A(buf->b_wininfo, 1), &kv_A(buf->b_wininfo, 0), (kv_size(buf->b_wininfo) - 1) * sizeof(kv_A(buf->b_wininfo, 0))); kv_A(buf->b_wininfo, 0) = wip; }
void nvim_wininfo_set_mark(WinInfo *wip, linenr_T lnum, colnr_T col, win_T *win)
{ wip->wi_mark.mark.lnum = lnum; wip->wi_mark.mark.col = col;
  if (win != NULL) { wip->wi_mark.view = mark_view_make(win->w_topline, wip->wi_mark.mark); } }
void nvim_wininfo_copy_from_win(WinInfo *wip, win_T *win)
{ copy_winopt(&win->w_onebuf_opt, &wip->wi_opt); wip->wi_fold_manual = win->w_fold_manual;
  rs_cloneFoldGrowArray(&win->w_folds, &wip->wi_folds); wip->wi_optset = true; }

int nvim_get_winopts_apply(WinInfo *wip, buf_T *buf)
{
  if (wip == NULL) {
    copy_winopt(&curwin->w_allbuf_opt, &curwin->w_onebuf_opt);
    return 0;
  }
  if (wip->wi_win != curwin && wip->wi_win != NULL && wip->wi_win->w_buffer == buf) {
    win_T *wp = wip->wi_win;
    copy_winopt(&wp->w_onebuf_opt, &curwin->w_onebuf_opt);
    curwin->w_fold_manual = wp->w_fold_manual;
    curwin->w_foldinvalid = true;
    rs_cloneFoldGrowArray(&wp->w_folds, &curwin->w_folds);
    return 1;
  } else if (wip->wi_optset) {
    copy_winopt(&wip->wi_opt, &curwin->w_onebuf_opt);
    curwin->w_fold_manual = wip->wi_fold_manual;
    curwin->w_foldinvalid = true;
    rs_cloneFoldGrowArray(&wip->wi_folds, &curwin->w_folds);
    return 2;
  } else {
    copy_winopt(&curwin->w_allbuf_opt, &curwin->w_onebuf_opt);
    return 0;
  }
}
void nvim_clear_winopt_curwin(void) { clear_winopt(&curwin->w_onebuf_opt); }
void nvim_curwin_set_changelistidx(int val) { curwin->w_changelistidx = val; }
bool nvim_curwin_config_is_minimal(void) { return curwin->w_config.style == kWinStyleMinimal; }
void nvim_curwin_set_p_fdl(int val) { curwin->w_p_fdl = (OptInt)val; }
void nvim_wininfo_set_changelistidx(WinInfo *wip, int val) { wip->wi_changelistidx = val; }
void nvim_wininfo_set_win(WinInfo *wip, win_T *win) { wip->wi_win = win; }
void nvim_buf_wininfo_remove(buf_T *buf, size_t i) { kv_shift(buf->b_wininfo, i, 1); }
fmark_T *nvim_get_no_position_ptr(void)
{ static fmark_T no_position = { { 1, 0, 0 }, 0, 0, { 0 }, NULL }; return &no_position; }
void nvim_buf_changedtick_di_tv_copy(buf_T *buf, void *out)
{ memcpy(out, &buf->changedtick_di.di_tv, sizeof(typval_T)); }
void nvim_buf_changedtick_di_set_number(buf_T *buf, int64_t val)
{ buf->changedtick_di.di_tv.vval.v_number = (varnumber_T)val; }
void nvim_tv_dict_watcher_notify(dict_T *dict, const char *key, void *newtv, void *oldtv)
{ tv_dict_watcher_notify(dict, (char *)key, (typval_T *)newtv, (typval_T *)oldtv); }
const char *nvim_buf_changedtick_di_key(buf_T *buf) { return (const char *)buf->changedtick_di.di_key; }
void *nvim_buf_changedtick_di_tv_ptr(buf_T *buf) { return &buf->changedtick_di.di_tv; }
void nvim_buf_b_locked_inc(buf_T *buf) { buf->b_locked++; }
void nvim_buf_b_locked_dec(buf_T *buf) { buf->b_locked--; }
bool nvim_buf_is_in_any_window(buf_T *buf)
{ FOR_ALL_TAB_WINDOWS(tab, win) { if (win->w_buffer == buf) { return true; } } return false; }

void nvim_buf_remove_fnames(buf_T *buf)
{
  if (buf->b_sfname != buf->b_ffname) {
    XFREE_CLEAR(buf->b_sfname);
  } else {
    buf->b_sfname = NULL;
  }
  XFREE_CLEAR(buf->b_ffname);
  buf->b_fname = buf->b_sfname;
}

void nvim_buf_set_fnames(buf_T *buf, char *ffname, char *sfname)
{
  char *sfname_copy = xstrdup(sfname);
#ifdef CASE_INSENSITIVE_FILENAME
  path_fix_case(sfname_copy);
#endif
  if (buf->b_sfname != buf->b_ffname) {
    xfree(buf->b_sfname);
  }
  xfree(buf->b_ffname);
  buf->b_ffname = ffname;
  buf->b_sfname = sfname_copy;
  buf->b_fname = buf->b_sfname;
}

void nvim_set_buf_opts_scratch(void)
{ set_option_value_give_err(kOptBufhidden, STATIC_CSTR_AS_OPTVAL("hide"), OPT_LOCAL);
  set_option_value_give_err(kOptBuftype, STATIC_CSTR_AS_OPTVAL("nofile"), OPT_LOCAL);
  set_option_value_give_err(kOptSwapfile, BOOLEAN_OPTVAL(false), OPT_LOCAL);
  RESET_BINDING(curwin); }
int nvim_swb_has_newtab(void) { return (swb_flags & kOptSwbFlagNewtab) ? 1 : 0; }
void *nvim_buf_prep_exarg_alloc(buf_T *buf)
{ exarg_T *ea = xcalloc(1, sizeof(exarg_T)); prep_exarg(ea, buf); return ea; }
void nvim_exarg_free(void *ea_void)
{ exarg_T *ea = (exarg_T *)ea_void; xfree(ea->cmd); xfree(ea); }
void *nvim_buf_aucmd_prepbuf_alloc(buf_T *buf)
{ aco_save_T *aco = xcalloc(1, sizeof(aco_save_T)); aucmd_prepbuf(aco, buf); return aco; }
void nvim_buf_aucmd_restbuf_free(void *aco_void)
{ aco_save_T *aco = (aco_save_T *)aco_void; aucmd_restbuf(aco); xfree(aco); }
int nvim_readfile_for_buf(buf_T *buf, void *ea_void)
{ return readfile(buf->b_ffname, buf->b_fname, 0, 0, (linenr_T)MAXLNUM,
                  (exarg_T *)ea_void, READ_NEW | READ_DUMMY, false); }
void nvim_set_visual_reselect(int val) { VIsual_reselect = val != 0; }
int nvim_get_state_mode(void) { return State; }
int nvim_buf_terminal_check_size(buf_T *buf)
{ if (buf && buf->terminal) { terminal_check_size(buf->terminal); return 1; } return 0; }
void nvim_curbuf_dec_nwindows(void) { if (curbuf) { curbuf->b_nwindows--; } }
int nvim_curwin_buffer_is_null(void) { return curwin->w_buffer == NULL ? 1 : 0; }
OptInt nvim_curbuf_get_p_tw(void) { return curbuf->b_p_tw; }
int nvim_curwin_buffer_is_buf(buf_T *buf) { return curwin->w_buffer == buf ? 1 : 0; }
int nvim_buf_aucmd_open_buffer(buf_T *buf)
{ aco_save_T aco; aucmd_prepbuf(&aco, buf); int status = open_buffer(false, NULL, 0);
  aucmd_restbuf(&aco); return (status != FAIL) ? 1 : 0; }
int do_buffer_ext(int action, int start, int dir, int count, int flags)
{ return rs_do_buffer_ext(action, start, dir, count, flags); }
void nvim_buf_lock(buf_T *buf) { buf->b_locked++; buf->b_locked_split++; }
void nvim_buf_unlock(buf_T *buf) { buf->b_locked--; buf->b_locked_split--; }
void nvim_syntax_clear_buf(buf_T *buf) { syntax_clear(&buf->b_s); }
void nvim_buf_clearFolding_all_windows(buf_T *buf)
{ FOR_ALL_TAB_WINDOWS(tp, win) { if (win->w_buffer == buf) { rs_clearFolding(win); } } }
void nvim_buf_b_locked_split_inc(buf_T *buf) { buf->b_locked_split++; }
void nvim_buf_b_locked_split_dec(buf_T *buf) { buf->b_locked_split--; }
void nvim_emsg_auabort(void) { emsg(_(e_auabort)); }
void nvim_buflist_setfpos_win(buf_T *buf, win_T *win)
{ buflist_setfpos(buf, win,
                  win->w_cursor.lnum == 1 ? 0 : win->w_cursor.lnum,
                  win->w_cursor.col, true); }
void nvim_buf_b_nwindows_dec_safe(buf_T *buf) { if (buf->b_nwindows > 0) { buf->b_nwindows--; } }
void nvim_terminal_close_buf(buf_T *buf)
{ if (buf->terminal) { buf->b_locked++; terminal_close(&buf->terminal, -1); buf->b_locked--; } }
int nvim_get_entered_free_all_mem(void)
{
#if defined(EXITFREE)
  return entered_free_all_mem ? 1 : 0;
#else
  return 0;
#endif
}
void nvim_win_set_buffer_null(win_T *win) { win->w_buffer = NULL; }
void nvim_mark_forget_file_all_tabs(int fnum)
{ FOR_ALL_TAB_WINDOWS(tp, wp) { mark_forget_file(wp, fnum); } }
void nvim_buf_wipe_free(buf_T *buf)
{
  if (buf->b_sfname != buf->b_ffname) { XFREE_CLEAR(buf->b_sfname); } else { buf->b_sfname = NULL; }
  XFREE_CLEAR(buf->b_ffname);
  if (buf->b_prev == NULL) { firstbuf = buf->b_next; } else { buf->b_prev->b_next = buf->b_next; }
  if (buf->b_next == NULL) { lastbuf = buf->b_prev; } else { buf->b_next->b_prev = buf->b_prev; }
  free_buffer(buf);
}
void nvim_buf_free_stuff_del(buf_T *buf)
{ free_buffer_stuff(buf, kBffClearWinInfo | kBffInitChangedtick); }

// Phase 1: buf_store_file_info / prep_exarg / set_forced_fenc accessors

// FileInfo accessors
int64_t nvim_fileinfo_get_mtime(const FileInfo *fi) { return (int64_t)fi->stat.st_mtim.tv_sec; }
int64_t nvim_fileinfo_get_mtime_ns(const FileInfo *fi) { return (int64_t)fi->stat.st_mtim.tv_nsec; }
int32_t nvim_fileinfo_get_mode(const FileInfo *fi) { return (int32_t)fi->stat.st_mode; }

// set_option_direct wrapper for fileencoding
void nvim_set_fileencoding_local(const char *fenc) {
  set_option_direct(kOptFileencoding, CSTR_AS_OPTVAL((char *)fenc), OPT_LOCAL, 0);
}

// Phase 3: shorten_buf_fname / shorten_fnames accessors
void nvim_buf_mf_fullname(buf_T *buf) { mf_fullname(buf->b_ml.ml_mfp); }
void nvim_set_redraw_tabline(int val) { redraw_tabline = (bool)val; }
int nvim_bt_nofilename(buf_T *buf) { return bt_nofilename(buf) ? 1 : 0; }

// Phase 5: buf_check_timestamp accessors
int nvim_buf_get_b_orig_mode(const buf_T *buf) { return buf->b_orig_mode; }
int64_t nvim_get_p_ar(void) { return p_ar; }
void nvim_buf_copy_mtime_to_mtime_read(buf_T *buf) { buf->b_mtime_read = buf->b_mtime; buf->b_mtime_read_ns = buf->b_mtime_ns; }
// --- bt_normal wrapper ---
int nvim_bt_normal(const buf_T *buf) { return bt_normal(buf) ? 1 : 0; }
// --- buf_contents_changed wrapper ---
int nvim_buf_contents_changed(buf_T *buf) { return buf_contents_changed(buf) ? 1 : 0; }
// --- os_fileinfo combined accessor: fill file_info from fname ---
//     Returns 1 if the file was found, 0 otherwise.
//     Also extracts the metadata into output parameters.
int nvim_os_fileinfo(const char *fname, int64_t *mtime_sec, int64_t *mtime_ns,
                     uint64_t *size, int32_t *mode) {
  FileInfo fi;
  if (!os_fileinfo(fname, &fi)) {
    return 0;
  }
  if (mtime_sec) *mtime_sec = (int64_t)fi.stat.st_mtim.tv_sec;
  if (mtime_ns)  *mtime_ns  = (int64_t)fi.stat.st_mtim.tv_nsec;
  if (size)      *size      = os_fileinfo_size(&fi);
  if (mode)      *mode      = (int32_t)fi.stat.st_mode;
  return 1;
}
// --- os_isdir and os_path_exists (some may already exist in other shims) ---
int nvim_os_isdir2(const char *name) { return os_isdir(name) ? 1 : 0; }
// --- allbuf_lock setter ---
void nvim_set_allbuf_lock(int val) { allbuf_lock = val; }
// --- VV_ vim variable accessors ---
void nvim_set_vim_var_fcs_reason(const char *reason) { set_vim_var_string(VV_FCS_REASON, (char *)reason, -1); }
void nvim_set_vim_var_fcs_choice_empty(void) { set_vim_var_string(VV_FCS_CHOICE, "", -1); }
const char *nvim_get_vim_var_fcs_choice(void) { return get_vim_var_str(VV_FCS_CHOICE); }
void nvim_set_vim_var_warningmsg(const char *msg, int len) { set_vim_var_string(VV_WARNINGMSG, (char *)msg, len); }
// --- UI/message functions ---
int nvim_ui_has_messages(void) { return ui_has(kUIMessages) ? 1 : 0; }
void nvim_os_delay(int ms, int ignoreinput) { os_delay((uint64_t)ms, ignoreinput != 0); }
int nvim_get_emsg_silent(void) { return emsg_silent; }
int nvim_get_redraw_cmdline(void) { return (int)redraw_cmdline; }
// Note: nvim_set_redraw_cmdline is defined in src/nvim-rs/window/src/globals.rs
int nvim_get_State(void) { return State; }
int nvim_get_MODE_NORMAL_BUSY(void) { return MODE_NORMAL_BUSY; }
int nvim_get_MODE_CMDLINE(void) { return MODE_CMDLINE; }
void nvim_msg_puts_hl_e(const char *s) { msg_puts_hl(s, HLF_E, true); }
void nvim_msg_puts_hl_w(const char *s) { msg_puts_hl(s, HLF_W, true); }
// --- home_replace_save wrapper ---
char *nvim_home_replace_save(const buf_T *buf, const char *fname) { return home_replace_save(buf, (char *)fname); }
// --- do_dialog for file-changed warning ---
int nvim_do_dialog_file_changed(const char *tbuf) {
  return do_dialog(VIM_WARNING, _("Warning"), (char *)tbuf,
                   _("&OK\n&Load File\nLoad File &and Options"), 1, NULL, true);
}
// --- undo accessors for buf_reload (used in Phase 5/6) ---
void nvim_u_write_undo(const char *name, int forceit, buf_T *buf, uint8_t *hash) {
  u_write_undo((char *)name, (bool)forceit, buf, hash);
}
// buf_reload caller wrapper for use from Rust (Phase 5)
void nvim_buf_reload(buf_T *buf, int orig_mode, int reload_options) {
  buf_reload(buf, orig_mode, reload_options);
}

// Phase 6: buf_reload / move_lines accessors
int nvim_get_p_ur(void) { return (int)p_ur; }
int nvim_shortmess_fileinfo(void) { return shortmess(SHM_FILEINFO) ? 1 : 0; }
int nvim_buf_is_empty(buf_T *buf) { return buf_is_empty(buf) ? 1 : 0; }
void nvim_wipe_buffer(buf_T *buf) { wipe_buffer(buf, false); }
void nvim_buf_updates_unload(buf_T *buf) { buf_updates_unload(buf, true); }
void nvim_do_modelines(void) { do_modelines(0); }
int nvim_u_savecommon_reload_ok(buf_T *buf) {
  u_sync(false);
  return u_savecommon(buf, 0, buf->b_ml.ml_line_count + 1, 0, true);
}
void nvim_unchanged(buf_T *buf) { unchanged(buf, true, true); }
int nvim_ml_open_buf(buf_T *buf) {
  buf_T *saved = curbuf;
  curbuf = buf;
  int r = ml_open(curbuf);
  curbuf = saved;
  return r;
}
void nvim_curbuf_set_b_flags_or(int flags) { curbuf->b_flags |= flags; }
void nvim_curbuf_set_b_keep_filetype(int val) { curbuf->b_keep_filetype = (bool)val; }
void nvim_curbuf_set_b_mod_set(int val) { curbuf->b_mod_set = (bool)val; }
int nvim_curbuf_get_b_orig_mode(void) { return curbuf->b_orig_mode; }
// Note: nvim_curwin_get_topline is defined in window_shim.c
void nvim_curwin_set_topline_clamped(linenr_T topline) {
  linenr_T max_line = curbuf->b_ml.ml_line_count;
  curwin->w_topline = topline < max_line ? topline : max_line;
  if (curwin->w_topline < 1) curwin->w_topline = 1;
}
void nvim_curwin_get_cursor(linenr_T *lnum, colnr_T *col) {
  *lnum = curwin->w_cursor.lnum;
  *col = curwin->w_cursor.col;
}
void nvim_curwin_set_cursor(linenr_T lnum, colnr_T col) {
  curwin->w_cursor.lnum = lnum;
  curwin->w_cursor.col = col;
  curwin->w_cursor.coladd = 0;
}
void nvim_check_cursor_curwin(void) { check_cursor(curwin); }
// Note: nvim_update_topline_curwin is defined in eval_shim.c
void nvim_curwin_set_buffer2(buf_T *buf) { curwin->w_buffer = buf; }
// Note: nvim_get_curbuf_ptr is defined in window_shim.c
void nvim_set_curbuf_ptr(buf_T *buf) { curbuf = buf; }
// ml_delete wrapper for move_lines: takes buf as context
int nvim_ml_delete_in_buf(buf_T *buf, linenr_T lnum) {
  buf_T *saved = curbuf;
  curbuf = buf;
  int r = ml_delete(lnum);
  curbuf = saved;
  return r;
}
// ml_append wrapper: appends line to curbuf (caller sets curbuf)
int nvim_ml_append_curbuf(linenr_T lnum, const char *line) {
  return ml_append(lnum, (char *)line, 0, false);
}
// Note: nvim_buf_get_b_ml_line_count is defined in undo.c
void nvim_semsg_reload_fail(const char *fname) {
  semsg(_("E321: Could not reload \"%s\""), fname);
}
void nvim_semsg_prep_reload_fail(const char *fname) {
  semsg(_("E462: Could not prepare for reloading \"%s\""), fname);
}
int nvim_readfile_reload(buf_T *buf, exarg_T *ea, int flags, int silent) {
  return readfile(buf->b_ffname, buf->b_fname, 0, 0, (linenr_T)MAXLNUM, ea, flags, silent != 0);
}
int nvim_aborting(void) { return aborting() ? 1 : 0; }
// exarg_T allocation for buf_reload (zero-initialised, no prep_exarg)
exarg_T *nvim_exarg_alloc_clear(void) {
  exarg_T *ea = xmalloc(sizeof(exarg_T));
  CLEAR_FIELD(*ea);
  return ea;
}
// Note: nvim_exarg_free (frees ea->cmd + ea) is defined above at line ~475
// BLN_DUMMY constant (from buffer.h)
int nvim_BLN_DUMMY(void) { return BLN_DUMMY; }
// BF_CHECK_RO constant
int nvim_BF_CHECK_RO(void) { return BF_CHECK_RO; }

// =============================================================================
// open_buffer compound accessors (Phase N: migrate open_buffer to Rust)
// =============================================================================


/// Set mf_dirty to MF_DIRTY_YES_NOSYNC if memfile exists.
void nvim_curbuf_mf_set_nosync(void)
{
  if (curbuf->b_ml.ml_mfp != NULL) {
    curbuf->b_ml.ml_mfp->mf_dirty = MF_DIRTY_YES_NOSYNC;
  }
}

/// If mf_dirty == MF_DIRTY_YES_NOSYNC, upgrade to MF_DIRTY_YES.
void nvim_curbuf_mf_unset_nosync(void)
{
  if (curbuf->b_ml.ml_mfp != NULL
      && curbuf->b_ml.ml_mfp->mf_dirty == MF_DIRTY_YES_NOSYNC) {
    curbuf->b_ml.ml_mfp->mf_dirty = MF_DIRTY_YES;
  }
}

/// Initialize bufref, modified_was_set, and cursor validity for open_buffer.
/// Returns the old_curbuf bufref via the out pointer.
void nvim_open_buffer_setup_bufref(bufref_T *old_curbuf_out)
{
  set_bufref(old_curbuf_out, curbuf);
  curbuf->b_modified_was_set = false;
  curwin->w_valid = 0;
}

/// Read the file into curbuf for open_buffer.
/// Handles UNIX fifo detection and binary-mode save/restore.
/// @param eap  exarg pointer (may be NULL)
/// @param flags  read flags (e.g. READ_NOFILE)
/// @param silent  shortmess result
/// @param read_fifo_out  set to 1 if fifo/socket detected (UNIX only)
/// @return  readfile() result (OK or FAIL), or OK if no file
int nvim_open_buffer_read_file(void *eap, int flags, int silent, int *read_fifo_out)
{
  *read_fifo_out = 0;
  if (curbuf->b_ffname == NULL) {
    return 1;  // OK, no file to read
  }

#ifdef UNIX
  int save_bin = curbuf->b_p_bin;
  int perm = os_getperm(curbuf->b_ffname);
  bool is_fifo = perm >= 0 && (S_ISFIFO(perm) || S_ISSOCK(perm)
#ifdef OPEN_CHR_FILES
                               || (S_ISCHR(perm) && rs_is_dev_fd_file(curbuf->b_ffname))
#endif
                               );
  if (is_fifo) {
    *read_fifo_out = 1;
    curbuf->b_p_bin = true;
  }
#endif

  int retval = readfile(curbuf->b_ffname, curbuf->b_fname,
                        0, 0, (linenr_T)MAXLNUM, (exarg_T *)eap,
                        flags | READ_NEW | (*read_fifo_out ? READ_FIFO : 0), silent != 0);

#ifdef UNIX
  if (*read_fifo_out) {
    curbuf->b_p_bin = save_bin;
    if (retval == OK) {
      retval = rs_read_buffer(false, eap, flags);
    }
  }
#endif

  if (bt_help(curbuf)) {
    get_local_additions();
  }
  return retval;
}

/// Read stdin for open_buffer (binary pre-read then retry).
/// @param eap  exarg pointer
/// @param flags  read flags
/// @param silent  shortmess result
/// @return readfile()/rs_read_buffer() result
int nvim_open_buffer_read_stdin(void *eap, int flags, int silent)
{
  int save_bin = curbuf->b_p_bin;
  curbuf->b_p_bin = true;
  int retval = readfile(NULL, NULL, 0, 0, (linenr_T)MAXLNUM, NULL,
                        flags | (READ_NEW + READ_STDIN), silent != 0);
  curbuf->b_p_bin = save_bin;
  if (retval == OK) {
    retval = rs_read_buffer(true, eap, flags);
  }
  return retval;
}

/// Handle first-time load of curbuf: buf_init_chartab + parse_cino.
void nvim_curbuf_init_first_load(void)
{
  if (curbuf->b_flags & BF_NEVERLOADED) {
    buf_init_chartab(curbuf, false);
    parse_cino(curbuf);
  }
}

/// Decide changed/unchanged state and save file format for open_buffer.
/// @param retval  current readfile result (non-zero = OK)
/// @param read_stdin  was reading from stdin
/// @param read_fifo  was reading from fifo
void nvim_open_buffer_set_changed(int retval, int read_stdin, int read_fifo)
{
  int fail_val = 0;  // FAIL == 0
  if ((got_int && vim_strchr(p_cpo, CPO_INTMOD) != NULL)
      || curbuf->b_modified_was_set
      || (aborting() && vim_strchr(p_cpo, CPO_INTMOD) != NULL)) {
    changed(curbuf);
  } else if (retval != fail_val && !read_stdin && !read_fifo) {
    unchanged(curbuf, false, true);
  }
  save_file_ff(curbuf);
  curbuf->b_last_changedtick = buf_get_changedtick(curbuf);
  curbuf->b_last_changedtick_i = buf_get_changedtick(curbuf);
  curbuf->b_last_changedtick_pum = buf_get_changedtick(curbuf);
  if (aborting()) {
    curbuf->b_flags |= BF_READERR;
  }
}

/// Set topline/topfill to 1/0 if VALID_TOPLINE is not set.
void nvim_curwin_init_topline(void)
{
  if (!(curwin->w_valid & VALID_TOPLINE)) {
    curwin->w_topline = 1;
    curwin->w_topfill = 0;
  }
}

/// Fire EVENT_BUFENTER and update retval.
void nvim_open_buffer_bufenter(int *retval)
{
  apply_autocmds_retval(EVENT_BUFENTER, NULL, NULL, false, curbuf, retval);
}

/// Execute the post-BUFENTER section of open_buffer:
/// - validates old_curbuf is still alive with memfile
/// - calls aucmd_prepbuf, do_modelines, clears BF_CHECK_RO|BF_NEVERLOADED
/// - conditionally fires BUFWINENTER
/// - restores aucmd state
/// @param old_curbuf  bufref saved before readfile
/// @param flags  read flags (checked for READ_NOWINENTER)
/// @param retval  in/out: current retval, updated by BUFWINENTER autocmd
void nvim_open_buffer_post_autocmd(bufref_T *old_curbuf, int flags, int *retval)
{
  if (!bufref_valid(old_curbuf) || old_curbuf->br_buf->b_ml.ml_mfp == NULL) {
    return;
  }
  aco_save_T aco;
  aucmd_prepbuf(&aco, old_curbuf->br_buf);
  do_modelines(0);
  curbuf->b_flags &= ~(BF_CHECK_RO | BF_NEVERLOADED);
  if ((flags & READ_NOWINENTER) == 0) {
    apply_autocmds_retval(EVENT_BUFWINENTER, NULL, NULL, false, curbuf, retval);
  }
  aucmd_restbuf(&aco);
}

/// Call rs_foldUpdateAll on curwin (convenience wrapper for Rust).
void rs_foldUpdateAll_curwin(void) { rs_foldUpdateAll(curwin); }

// =============================================================================
// free_buffer cluster compound accessors (Phase N: migrate free_buffer to Rust)
// =============================================================================

// buf_init_changedtick and nvim_buf_init_changedtick_c migrated from buffer.c (Phase 4).
// These needed to stay in C to avoid complex Rust struct layout issues with
// ChangedtickDictItem / typval_T compound literal initializers.

/// Initialize b:changedtick and changedtick_val attribute.
/// (Migrated from static inline in buffer.c.)
static inline void buf_init_changedtick_shim(buf_T *const buf)
{
  buf->changedtick_di = (ChangedtickDictItem) {
    .di_flags = DI_FLAGS_RO|DI_FLAGS_FIX,  // Must not include DI_FLAGS_ALLOC.
    .di_tv = (typval_T) {
      .v_type = VAR_NUMBER,
      .v_lock = VAR_FIXED,
      .vval.v_number = buf_get_changedtick(buf),
    },
    .di_key = "changedtick",
  };
  tv_dict_add(buf->b_vars, (dictitem_T *)&buf->changedtick_di);
}

/// Non-static wrapper for buf_init_changedtick (called from buffer_shim.c compound shims).
void nvim_buf_init_changedtick_c(buf_T *buf)
{
  buf_init_changedtick_shim(buf);
}
// nvim_inc_buf_free_count is exported from Rust state.rs (Phase 1).
extern void nvim_inc_buf_free_count(void);
// free_buf_options is exported from Rust close.rs (Phase 2).
extern void free_buf_options(buf_T *buf, bool free_p_ff);
// buflist_new is exported from Rust close.rs (Phase 3).
extern buf_T *buflist_new(char *ffname_arg, char *sfname_arg, linenr_T lnum, int flags);
// top_file_num accessors exported from Rust state.rs (Phase 1).
extern int nvim_get_top_file_num(void);
extern int nvim_inc_top_file_num(void);
extern void nvim_reset_top_file_num(void);
extern void rs_aubuflocal_remove(int bufnr);

/// Free the b_wininfo list for a buffer.
void nvim_clear_wininfo_c(buf_T *buf)
{
  for (size_t i = 0; i < kv_size(buf->b_wininfo); i++) {
    free_wininfo(kv_A(buf->b_wininfo, i), buf);
  }
  kv_size(buf->b_wininfo) = 0;
}

/// Handle all C-side operations needed by free_buffer_stuff.
/// Corresponds to the body of free_buffer_stuff() in buffer.c.
/// @param buf       the buffer being freed
/// @param free_flags  BufFreeFlags (kBffClearWinInfo=1, kBffInitChangedtick=2)
void nvim_free_buffer_stuff_c_parts(buf_T *buf, int free_flags)
{
  if (free_flags & kBffClearWinInfo) {
    nvim_clear_wininfo_c(buf);
    free_buf_options(buf, true);
    ga_clear(&buf->b_s.b_langp);
  }
  {
    hashitem_T *const changedtick_hi = hash_find(&buf->b_vars->dv_hashtab, "changedtick");
    assert(changedtick_hi != NULL);
    hash_remove(&buf->b_vars->dv_hashtab, changedtick_hi);
  }
  vars_clear(&buf->b_vars->dv_hashtab);
  hash_init(&buf->b_vars->dv_hashtab);
  if (free_flags & kBffInitChangedtick) {
    nvim_buf_init_changedtick_c(buf);
  }
  uc_clear(&buf->b_ucmds);
  extmark_free_all(buf);
  map_clear_mode(buf, MAP_ALL_MODES, true, false);
  map_clear_mode(buf, MAP_ALL_MODES, true, true);
  XFREE_CLEAR(buf->b_start_fenc);
  buf_free_callbacks(buf);
}

/// Handle all C-side operations needed by free_buffer.
/// @param buf   the buffer being freed
void nvim_free_buffer_c_parts(buf_T *buf)
{
  pmap_del(int)(&buffer_handles, buf->b_fnum, NULL);
  nvim_inc_buf_free_count();
  // b:changedtick uses an item in buf_T.
  nvim_free_buffer_stuff_c_parts(buf, kBffClearWinInfo);
  if (buf->b_vars->dv_refcount > DO_NOT_FREE_CNT) {
    tv_dict_add(buf->b_vars,
                tv_dict_item_copy((dictitem_T *)(&buf->changedtick_di)));
  }
  unref_var_dict(buf->b_vars);
  rs_aubuflocal_remove(buf->b_fnum);
  xfree(buf->additional_data);
  xfree(buf->b_prompt_text);
  kv_destroy(buf->b_wininfo);
  callback_free(&buf->b_prompt_callback);
  callback_free(&buf->b_prompt_interrupt);
  clear_fmark(&buf->b_last_cursor, 0);
  clear_fmark(&buf->b_last_insert, 0);
  clear_fmark(&buf->b_last_change, 0);
  clear_fmark(&buf->b_prompt_start, 0);
  for (size_t i = 0; i < NMARKS; i++) {
    free_fmark(buf->b_namedm[i]);
  }
  for (int i = 0; i < buf->b_changelistlen; i++) {
    free_fmark(buf->b_changelist[i]);
  }
  if (autocmd_busy) {
    CLEAR_FIELD(buf->b_namedm);
    CLEAR_FIELD(buf->b_changelist);
    buf->b_next = au_pending_free_buf;
    au_pending_free_buf = buf;
  } else {
    xfree(buf);
  }
}

// nvim_buf_get_changedtick is defined in api/buffer.c (nvim API).

/// Set win->w_p_cul (cursorline option).
void nvim_win_set_w_p_cul(win_T *win, bool val) { win->w_p_cul = val; }

/// Set win->w_p_cuc (cursorcolumn option).
void nvim_win_set_w_p_cuc(win_T *win, bool val) { win->w_p_cuc = val; }

// =============================================================================
// free_buf_options shim (Phase 2: migrate free_buf_options to Rust)
// =============================================================================

// clear_cpt_callbacks is in insexpand_shim.c (no static header, use extern).
extern void clear_cpt_callbacks(Callback **callbacks, int count);

// =============================================================================
// buflist_new shim (Phase 3: migrate buflist_new to Rust)
// =============================================================================

/// Execute the body of buflist_new() in C.
/// Called from the Rust buflist_new() drop-in replacement.
///
/// This is the ONLY place where a new buffer structure is allocated.
/// (A spell file buffer is allocated in spell.c, but that's not a normal
/// buffer.)
buf_T *nvim_buflist_new_impl(char *ffname_arg, char *sfname_arg, linenr_T lnum, int flags)
{
  char *ffname = ffname_arg;
  char *sfname = sfname_arg;
  buf_T *buf;

  fname_expand(curbuf, &ffname, &sfname);       // will allocate ffname

  // If the file name already exists in the list, update the entry.

  // We can use inode numbers when the file exists.  Works better
  // for hard links.
  FileID file_id;
  bool file_id_valid = (sfname != NULL && os_fileid(sfname, &file_id));
  if (ffname != NULL && !(flags & (BLN_DUMMY | BLN_NEW))
      && (buf = buflist_findname_file_id(ffname, &file_id, file_id_valid)) != NULL) {
    xfree(ffname);
    if (lnum != 0) {
      buflist_setfpos(buf, (flags & BLN_NOCURWIN) ? NULL : curwin,
                      lnum, 0, false);
    }
    if ((flags & BLN_NOOPT) == 0) {
      // Copy the options now, if 'cpo' doesn't have 's' and not done already.
      buf_copy_options(buf, 0);
    }
    if ((flags & BLN_LISTED) && !buf->b_p_bl) {
      buf->b_p_bl = true;
      bufref_T bufref;
      set_bufref(&bufref, buf);
      if (!(flags & BLN_DUMMY)) {
        if (apply_autocmds(EVENT_BUFADD, NULL, NULL, false, buf)
            && !bufref_valid(&bufref)) {
          return NULL;
        }
      }
    }
    return buf;
  }

  buf = NULL;
  if ((flags & BLN_CURBUF) && curbuf_reusable()) {
    bufref_T bufref;

    assert(curbuf != NULL);
    buf = curbuf;
    set_bufref(&bufref, buf);
    // It's like this buffer is deleted.  Watch out for autocommands that
    // change curbuf!  If that happens, allocate a new buffer anyway.
    buf_freeall(buf, BFA_WIPE | BFA_DEL);
    if (aborting()) {           // autocmds may abort script processing
      xfree(ffname);
      return NULL;
    }
    if (!bufref_valid(&bufref)) {
      buf = NULL;  // buf was deleted; allocate a new buffer
    }
  }
  if (buf != curbuf || curbuf == NULL) {
    buf = xcalloc(1, sizeof(buf_T));
    // init b: variables
    buf->b_vars = tv_dict_alloc();
    init_var_dict(buf->b_vars, &buf->b_bufvar, VAR_SCOPE);
    nvim_buf_init_changedtick_c(buf);
  }

  if (ffname != NULL) {
    buf->b_ffname = ffname;
    buf->b_sfname = xstrdup(sfname);
  }

  clear_wininfo(buf);
  WinInfo *curwin_info = xcalloc(1, sizeof(WinInfo));
  kv_push(buf->b_wininfo, curwin_info);

  if (buf == curbuf) {
    free_buffer_stuff(buf, kBffInitChangedtick);  // delete local vars et al.

    // Init the options.
    buf->b_p_initialized = false;
    buf_copy_options(buf, BCO_ENTER);

    // need to reload lmaps and set b:keymap_name
    curbuf->b_kmap_state |= KEYMAP_INIT;
  } else {
    // put new buffer at the end of the buffer list
    buf->b_next = NULL;
    if (firstbuf == NULL) {             // buffer list is empty
      buf->b_prev = NULL;
      firstbuf = buf;
    } else {                            // append new buffer at end of list
      lastbuf->b_next = buf;
      buf->b_prev = lastbuf;
    }
    lastbuf = buf;

    buf->b_fnum = nvim_inc_top_file_num();
    pmap_put(int)(&buffer_handles, buf->b_fnum, buf);
    if (nvim_get_top_file_num() < 0) {  // wrap around (may cause duplicates)
      emsg(_("W14: Warning: List of file names overflow"));
      if (emsg_silent == 0 && !in_assert_fails && !ui_has(kUIMessages)) {
        ui_flush();
        os_delay(3001, true);  // make sure it is noticed
      }
      nvim_reset_top_file_num();
    }

    // Always copy the options from the current buffer.
    buf_copy_options(buf, BCO_ALWAYS);
  }

  curwin_info->wi_mark = (fmark_T)INIT_FMARK;
  curwin_info->wi_mark.mark.lnum = lnum;
  curwin_info->wi_win = curwin;

  hash_init(&buf->b_s.b_keywtab);
  hash_init(&buf->b_s.b_keywtab_ic);

  buf->b_fname = buf->b_sfname;
  if (!file_id_valid) {
    buf->file_id_valid = false;
  } else {
    buf->file_id_valid = true;
    buf->file_id = file_id;
  }
  buf->b_u_synced = true;
  buf->b_flags = BF_CHECK_RO | BF_NEVERLOADED;
  if (flags & BLN_DUMMY) {
    buf->b_flags |= BF_DUMMY;
  }
  buf_clear_file(buf);
  clrallmarks(buf, 0);                  // clear marks
  fmarks_check_names(buf);              // check file marks for this file
  buf->b_p_bl = (flags & BLN_LISTED) ? true : false;    // init 'buflisted'
  kv_destroy(buf->update_channels);
  kv_init(buf->update_channels);
  kv_destroy(buf->update_callbacks);
  kv_init(buf->update_callbacks);
  if (!(flags & BLN_DUMMY)) {
    // Tricky: these autocommands may change the buffer list.  They could also
    // split the window with re-using the one empty buffer. This may result in
    // unexpectedly losing the empty buffer.
    bufref_T bufref;
    set_bufref(&bufref, buf);
    if (apply_autocmds(EVENT_BUFNEW, NULL, NULL, false, buf)
        && !bufref_valid(&bufref)) {
      return NULL;
    }
    if ((flags & BLN_LISTED)
        && apply_autocmds(EVENT_BUFADD, NULL, NULL, false, buf)
        && !bufref_valid(&bufref)) {
      return NULL;
    }
    if (aborting()) {
      // Autocmds may abort script processing.
      return NULL;
    }
  }

  buf->b_prompt_callback.type = kCallbackNone;
  buf->b_prompt_interrupt.type = kCallbackNone;
  buf->b_prompt_text = NULL;
  clear_fmark(&buf->b_prompt_start, 0);

  return buf;
}

/// Execute the body of free_buf_options() in C.
/// Called from the Rust free_buf_options() drop-in replacement.
///
/// @param buf      Buffer whose option strings to free.
/// @param free_p_ff  Also free fileformat/buftype/fileencoding options.
void nvim_buf_do_free_options(buf_T *buf, bool free_p_ff)
{
  if (free_p_ff) {
    clear_string_option(&buf->b_p_fenc);
    clear_string_option(&buf->b_p_ff);
    clear_string_option(&buf->b_p_bh);
    clear_string_option(&buf->b_p_bt);
  }
  clear_string_option(&buf->b_p_def);
  clear_string_option(&buf->b_p_inc);
  clear_string_option(&buf->b_p_inex);
  clear_string_option(&buf->b_p_inde);
  clear_string_option(&buf->b_p_indk);
  clear_string_option(&buf->b_p_fp);
  clear_string_option(&buf->b_p_fex);
  clear_string_option(&buf->b_p_kp);
  clear_string_option(&buf->b_p_mps);
  clear_string_option(&buf->b_p_fo);
  clear_string_option(&buf->b_p_flp);
  clear_string_option(&buf->b_p_isk);
  clear_string_option(&buf->b_p_vsts);
  XFREE_CLEAR(buf->b_p_vsts_nopaste);
  XFREE_CLEAR(buf->b_p_vsts_array);
  clear_string_option(&buf->b_p_vts);
  XFREE_CLEAR(buf->b_p_vts_array);
  clear_string_option(&buf->b_p_keymap);
  keymap_ga_clear(&buf->b_kmap_ga);
  ga_clear(&buf->b_kmap_ga);
  clear_string_option(&buf->b_p_com);
  clear_string_option(&buf->b_p_cms);
  clear_string_option(&buf->b_p_nf);
  clear_string_option(&buf->b_p_syn);
  clear_string_option(&buf->b_s.b_syn_isk);
  clear_string_option(&buf->b_s.b_p_spc);
  clear_string_option(&buf->b_s.b_p_spf);
  vim_regfree(buf->b_s.b_cap_prog);
  buf->b_s.b_cap_prog = NULL;
  clear_string_option(&buf->b_s.b_p_spl);
  clear_string_option(&buf->b_s.b_p_spo);
  clear_string_option(&buf->b_p_sua);
  clear_string_option(&buf->b_p_ft);
  clear_string_option(&buf->b_p_cink);
  clear_string_option(&buf->b_p_cino);
  clear_string_option(&buf->b_p_lop);
  clear_string_option(&buf->b_p_cinsd);
  clear_string_option(&buf->b_p_cinw);
  clear_string_option(&buf->b_p_cot);
  clear_string_option(&buf->b_p_cpt);
  clear_string_option(&buf->b_p_cfu);
  callback_free(&buf->b_cfu_cb);
  clear_string_option(&buf->b_p_ofu);
  callback_free(&buf->b_ofu_cb);
  clear_string_option(&buf->b_p_tsrfu);
  callback_free(&buf->b_tsrfu_cb);
  clear_cpt_callbacks(&buf->b_p_cpt_cb, buf->b_p_cpt_count);
  buf->b_p_cpt_count = 0;
  clear_string_option(&buf->b_p_gefm);
  clear_string_option(&buf->b_p_gp);
  clear_string_option(&buf->b_p_mp);
  clear_string_option(&buf->b_p_efm);
  clear_string_option(&buf->b_p_ep);
  clear_string_option(&buf->b_p_path);
  clear_string_option(&buf->b_p_tags);
  clear_string_option(&buf->b_p_tc);
  clear_string_option(&buf->b_p_tfu);
  callback_free(&buf->b_tfu_cb);
  clear_string_option(&buf->b_p_ffu);
  callback_free(&buf->b_ffu_cb);
  clear_string_option(&buf->b_p_dict);
  clear_string_option(&buf->b_p_dia);
  clear_string_option(&buf->b_p_tsr);
  clear_string_option(&buf->b_p_qe);
  buf->b_p_ac = -1;
  buf->b_p_ar = -1;
  buf->b_p_ul = NO_LOCAL_UNDOLEVEL;
  clear_string_option(&buf->b_p_lw);
  clear_string_option(&buf->b_p_bkc);
  clear_string_option(&buf->b_p_menc);
}
