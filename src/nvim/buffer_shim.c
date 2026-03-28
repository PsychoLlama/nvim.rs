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
#include "nvim/indent.h"
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

#include "buffer_shim.c.generated.h"

// Rust-exported fold/window helpers
extern void rs_cloneFoldGrowArray(garray_T *from, garray_T *to);
extern void rs_clearFolding(win_T *win);
// buffer.c non-static helpers
extern void free_buffer(buf_T *buf);
extern void free_buffer_stuff(buf_T *buf, int free_flags);
// Rust buffer-lifecycle helpers
extern int rs_do_buffer_ext(int action, int start, int dir, int count, int flags);

int nvim_buf_get_handle(buf_T *buf) { return buf ? buf->b_fnum : 0; }
char nvim_buf_get_buftype(buf_T *buf) { return buf->b_p_bt[0]; }
char nvim_buf_get_buftype_2(buf_T *buf) { return buf->b_p_bt[2]; }
int nvim_buf_get_help(buf_T *buf) { return buf->b_help; }
int nvim_buf_get_terminal(buf_T *buf) { return buf->terminal != NULL; }
char nvim_buf_get_fileformat(buf_T *buf) { return buf->b_p_ff[0]; }
int nvim_buf_get_bin(buf_T *buf) { return buf->b_p_bin; }
buf_T *nvim_get_lastbuf(void) { return lastbuf; }
buf_T *nvim_buf_get_prev(buf_T *buf) { return buf->b_prev; }
buf_T *nvim_bufref_get_buf(bufref_T *bufref) { return bufref->br_buf; }
uint32_t nvim_buf_meta_total_sign_hl(buf_T *buf) { return buf ? buf_meta_total(buf, kMTMetaSignHL) : 0; }
uint32_t nvim_buf_meta_total_sign_text(buf_T *buf) { return buf ? buf_meta_total(buf, kMTMetaSignText) : 0; }
int nvim_bufref_get_fnum(bufref_T *bufref) { return bufref->br_fnum; }
int nvim_bufref_get_buf_free_count(bufref_T *bufref) { return bufref->br_buf_free_count; }
int nvim_buf_get_fnum(buf_T *buf) { return buf->b_fnum; }
char nvim_buf_get_bufhidden(buf_T *buf) { return buf->b_p_bh[0]; }
const char *nvim_buf_get_b_fname(buf_T *buf) { return buf->b_fname; }
const char *nvim_buf_get_b_ffname(buf_T *buf) { return buf->b_ffname; }
const char *nvim_buf_get_b_sfname(buf_T *buf) { return buf->b_sfname; }
const char *nvim_buf_get_b_p_efm(buf_T *buf) { return buf->b_p_efm; }
int nvim_buf_get_b_p_ro(buf_T *buf) { return buf->b_p_ro; }
const char *nvim_buf_get_b_p_ft(buf_T *buf) { return buf->b_p_ft; }
int nvim_buf_get_b_p_ma(buf_T *buf) { return buf->b_p_ma; }
void nvim_buf_set_b_p_ml(buf_T *buf, int val) { if (buf) { buf->b_p_ml = val != 0; } }
void nvim_buf_set_b_p_iminsert(buf_T *buf, int val) { if (buf) { buf->b_p_iminsert = val; } }
void nvim_buf_set_b_p_imsearch(buf_T *buf, int val) { if (buf) { buf->b_p_imsearch = val; } }
int nvim_get_cmdmod_cmod_flags(void) { return cmdmod.cmod_flags; }
uint64_t *nvim_buf_get_chartab(buf_T *buf) { return buf->b_chartab; }
OptInt nvim_buf_get_p_ts(buf_T *buf) { return buf->b_p_ts; }
int *nvim_buf_get_p_vts_array(buf_T *buf) { return buf->b_p_vts_array; }
OptInt nvim_buf_get_p_sw(buf_T *buf) { return buf->b_p_sw; }
int nvim_buf_get_nwindows(buf_T *buf) { return buf->b_nwindows; }
int nvim_buf_get_locked(buf_T *buf) { return buf->b_locked; }
int nvim_buf_get_locked_split(buf_T *buf) { return buf->b_locked_split; }
int nvim_buf_get_flags(buf_T *buf) { return buf->b_flags; }
int nvim_buf_get_changed(buf_T *buf) { return buf->b_changed; }
int nvim_buf_get_b_p_bl(buf_T *buf) { return buf->b_p_bl; }
const char *nvim_curbuf_get_ffname(void) { return curbuf->b_ffname; }
const char *nvim_curbuf_get_path(void) { return curbuf->b_p_path; }
const char *nvim_curbuf_get_inex(void) { return curbuf->b_p_inex; }
char *nvim_get_namebuff(void) { return NameBuff; }
OptInt nvim_buf_get_p_sts(buf_T *buf) { return buf ? buf->b_p_sts : 0; }
const char *nvim_curbuf_get_line_ptr(void) { return ml_get_buf(curbuf, curwin->w_cursor.lnum); }
int nvim_buf_get_ml_mfp_null(buf_T *buf) { return buf->b_ml.ml_mfp == NULL; }
int nvim_buf_file_id_valid(buf_T *buf) { return buf->file_id_valid; }
void nvim_buf_get_file_id(buf_T *buf, void *out) { *(FileID *)out = buf->file_id; }
void nvim_buf_set_file_id_data(buf_T *buf, const void *file_id, bool valid)
{ if (valid) { buf->file_id = *(const FileID *)file_id; } buf->file_id_valid = valid; }

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
void nvim_buf_set_ml_line_count(buf_T *buf, linenr_T val) { buf->b_ml.ml_line_count = val; }
void nvim_buf_set_ml_mfp_null(buf_T *buf) { buf->b_ml.ml_mfp = NULL; }
int64_t nvim_buf_get_changedtick_direct(buf_T *buf) { return buf_get_changedtick(buf); }
void nvim_buf_set_p_eof(buf_T *buf, int val) { buf->b_p_eof = val; }
void nvim_buf_set_start_eof(buf_T *buf, int val) { buf->b_start_eof = val; }
void nvim_buf_set_p_eol(buf_T *buf, int val) { buf->b_p_eol = val; }
void nvim_buf_set_start_eol(buf_T *buf, int val) { buf->b_start_eol = val; }
void nvim_buf_set_p_bomb(buf_T *buf, int val) { buf->b_p_bomb = val; }
void nvim_buf_set_start_bomb(buf_T *buf, int val) { buf->b_start_bomb = val; }
int nvim_curwin_get_alt_fnum(void) { return curwin->w_alt_fnum; }
buf_T *nvim_handle_get_buffer(handle_T handle) { return handle_get_buffer(handle); }
void nvim_buf_set_b_p_bl(buf_T *buf, int val) { buf->b_p_bl = val; }
int64_t nvim_buf_get_last_used(buf_T *buf) { return buf ? (int64_t)buf->b_last_used : 0; }
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
int nvim_buf_get_b_p_initialized(buf_T *buf) { return buf->b_p_initialized ? 1 : 0; }
void nvim_buf_set_b_p_initialized(buf_T *buf, int val) { buf->b_p_initialized = val != 0; }
void nvim_buf_set_b_help(buf_T *buf, int val) { buf->b_help = val != 0; }
void nvim_buf_clear_b_p_script_ctx(buf_T *buf) { CLEAR_FIELD(buf->b_p_script_ctx); }
int nvim_buf_get_b_p_bt_is_help(buf_T *buf) { return (buf->b_p_bt && buf->b_p_bt[0] == 'h') ? 1 : 0; }
char *nvim_buf_save_and_clear_b_p_isk(buf_T *buf)
{ char *saved = buf->b_p_isk; buf->b_p_isk = NULL; return saved; }
void nvim_buf_restore_b_p_isk(buf_T *buf, char *saved) { buf->b_p_isk = saved; }
void nvim_buf_clear_b_p_ro(buf_T *buf) { buf->b_p_ro = false; }
void nvim_call_compile_cap_prog_buf(buf_T *buf) { compile_cap_prog(&buf->b_s); }
void nvim_call_tabstop_set_vsts(buf_T *buf, const char *str) { tabstop_set(str, &buf->b_p_vsts_array); }
void nvim_call_tabstop_set_vts(buf_T *buf, const char *str) { tabstop_set(str, &buf->b_p_vts_array); }
int nvim_buf_get_b_p_vts_array_is_null(buf_T *buf) { return buf->b_p_vts_array == NULL ? 1 : 0; }
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
void nvim_buf_set_b_p_bh_empty(buf_T *buf) { buf->b_p_bh = empty_string_option; }
void nvim_buf_set_b_p_bt_empty(buf_T *buf) { buf->b_p_bt = empty_string_option; }
void nvim_buf_set_b_p_ac_minus1(buf_T *buf) { buf->b_p_ac = -1; }
void nvim_buf_set_b_p_ar_minus1(buf_T *buf) { buf->b_p_ar = -1; }
void nvim_buf_set_b_p_ul_no_local(buf_T *buf) { buf->b_p_ul = NO_LOCAL_UNDOLEVEL; }
void nvim_buf_set_b_p_bkc_empty(buf_T *buf) { buf->b_p_bkc = empty_string_option; buf->b_bkc_flags = 0; }
void nvim_buf_set_b_p_tc_empty(buf_T *buf) { buf->b_p_tc = empty_string_option; buf->b_tc_flags = 0; }
void nvim_buf_set_b_p_cot_empty(buf_T *buf) { buf->b_p_cot = empty_string_option; buf->b_cot_flags = 0; }
void nvim_buf_set_b_s_syn_isk_empty(buf_T *buf) { buf->b_s.b_syn_isk = empty_string_option; }
void nvim_buf_set_b_p_ai_nopaste(buf_T *buf, int v) { buf->b_p_ai_nopaste = v != 0; }
void nvim_buf_set_b_p_tw_nopaste(buf_T *buf, OptInt v) { buf->b_p_tw_nopaste = v; }
void nvim_buf_set_b_p_tw_nobin(buf_T *buf, OptInt v) { buf->b_p_tw_nobin = v; }
void nvim_buf_set_b_p_wm_nopaste(buf_T *buf, OptInt v) { buf->b_p_wm_nopaste = v; }
void nvim_buf_set_b_p_wm_nobin(buf_T *buf, OptInt v) { buf->b_p_wm_nobin = v; }
void nvim_buf_set_b_p_et_nobin(buf_T *buf, int v) { buf->b_p_et_nobin = v != 0; }
void nvim_buf_set_b_p_et_nopaste(buf_T *buf, int v) { buf->b_p_et_nopaste = v != 0; }
void nvim_buf_set_b_p_ml_nobin(buf_T *buf, int v) { buf->b_p_ml_nobin = v != 0; }
void nvim_buf_set_b_p_sts_nopaste(buf_T *buf, OptInt v) { buf->b_p_sts_nopaste = v; }
int nvim_buf_get_b_p_ai_nopaste(buf_T *buf) { return (int)buf->b_p_ai_nopaste; }
OptInt nvim_buf_get_b_p_tw_nopaste(buf_T *buf) { return buf->b_p_tw_nopaste; }
OptInt nvim_buf_get_b_p_wm_nopaste(buf_T *buf) { return buf->b_p_wm_nopaste; }
OptInt nvim_buf_get_b_p_sts_nopaste(buf_T *buf) { return buf->b_p_sts_nopaste; }
int nvim_buf_get_b_p_et_nopaste(buf_T *buf) { return (int)buf->b_p_et_nopaste; }
char *nvim_buf_get_b_p_vsts(buf_T *buf) { return buf->b_p_vsts; }
char *nvim_buf_get_b_p_vsts_nopaste(buf_T *buf) { return buf->b_p_vsts_nopaste; }
void nvim_buf_set_b_p_vsts_raw(buf_T *buf, char *val) { buf->b_p_vsts = val; }
int *volatile *nvim_buf_get_b_p_vsts_array_ptr(buf_T *buf) { return (int *volatile *)&buf->b_p_vsts_array; }
void nvim_buf_set_b_p_ma(buf_T *buf, int v) { buf->b_p_ma = v != 0; }
void nvim_buf_set_b_p_vsts_nopaste_dup(buf_T *buf, const char *s) { buf->b_p_vsts_nopaste = s ? xstrdup(s) : NULL; }
void nvim_buf_set_b_s_spc_dup(buf_T *buf, const char *s) { buf->b_s.b_p_spc = xstrdup(s); }
void nvim_buf_set_b_s_spf_dup(buf_T *buf, const char *s) { buf->b_s.b_p_spf = xstrdup(s); }
void nvim_buf_set_b_s_spl_dup(buf_T *buf, const char *s) { buf->b_s.b_p_spl = xstrdup(s); }
void nvim_buf_set_b_s_spo_dup(buf_T *buf, const char *s) { buf->b_s.b_p_spo = xstrdup(s); }
void nvim_buf_set_b_s_spo_flags_from_global(buf_T *buf) { buf->b_s.b_p_spo_flags = spo_flags; }
MarkTree *nvim_buf_get_marktree(buf_T *buf) { return buf->b_marktree; }
bcount_t nvim_buf_get_deleted_bytes2(buf_T *buf) { return buf->deleted_bytes2; }
void nvim_buf_set_deleted_bytes2(buf_T *buf, bcount_t val) { buf->deleted_bytes2 = val; }
int nvim_buf_get_prev_line_count(buf_T *buf) { return buf->b_prev_line_count; }
void nvim_buf_set_prev_line_count(buf_T *buf, int val) { buf->b_prev_line_count = val; }
bool nvim_buf_signcols_get_autom(buf_T *buf) { return buf->b_signcols.autom; }
void nvim_buf_signcols_clear(buf_T *buf) { buf->b_signcols.max = 0; CLEAR_FIELD(buf->b_signcols.count); }
size_t nvim_buf_wininfo_count(buf_T *buf) { return kv_size(buf->b_wininfo); }
WinInfo *nvim_buf_wininfo_get(buf_T *buf, size_t i) { return kv_A(buf->b_wininfo, i); }
win_T *nvim_wininfo_get_win(WinInfo *wip) { return wip->wi_win; }
bool nvim_wininfo_get_optset(WinInfo *wip) { return wip->wi_optset; }
bool nvim_wininfo_get_wo_diff(WinInfo *wip) { return wip->wi_opt.wo_diff; }
int nvim_wininfo_get_changelistidx(WinInfo *wip) { return wip->wi_changelistidx; }
fmark_T *nvim_wininfo_get_mark_ptr(WinInfo *wip) { return &wip->wi_mark; }
bool nvim_wininfo_get_fold_manual(WinInfo *wip) { return wip->wi_fold_manual; }
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
void nvim_wininfo_set_optset(WinInfo *wip, bool val) { wip->wi_optset = val; }
void nvim_wininfo_set_fold_manual(WinInfo *wip, bool val) { wip->wi_fold_manual = val; }
void nvim_wininfo_set_win(WinInfo *wip, win_T *win) { wip->wi_win = win; }
void nvim_buf_wininfo_remove(buf_T *buf, size_t i) { kv_shift(buf->b_wininfo, i, 1); }
fmark_T *nvim_get_no_position_ptr(void)
{ static fmark_T no_position = { { 1, 0, 0 }, 0, 0, { 0 }, NULL }; return &no_position; }
void nvim_buf_changedtick_di_tv_copy(buf_T *buf, void *out)
{ memcpy(out, &buf->changedtick_di.di_tv, sizeof(typval_T)); }
void nvim_buf_changedtick_di_set_number(buf_T *buf, int64_t val)
{ buf->changedtick_di.di_tv.vval.v_number = (varnumber_T)val; }
dict_T *nvim_buf_get_b_vars(buf_T *buf) { return buf->b_vars; }
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
void nvim_buf_set_nwindows(buf_T *buf, int val) { buf->b_nwindows = val; }
void nvim_buf_flags_and(buf_T *buf, int mask) { buf->b_flags &= mask; }
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
void nvim_buf_set_flags(buf_T *buf, int flags) { buf->b_flags = flags; }
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
int nvim_buf_b_ffname_is_null(buf_T *buf) { return buf->b_ffname == NULL ? 1 : 0; }

// Phase 1: buf_store_file_info / prep_exarg / set_forced_fenc accessors

// FileInfo accessors
int64_t nvim_fileinfo_get_mtime(const FileInfo *fi) { return (int64_t)fi->stat.st_mtim.tv_sec; }
int64_t nvim_fileinfo_get_mtime_ns(const FileInfo *fi) { return (int64_t)fi->stat.st_mtim.tv_nsec; }
uint64_t nvim_fileinfo_get_size(const FileInfo *fi) { return os_fileinfo_size(fi); }
int32_t nvim_fileinfo_get_mode(const FileInfo *fi) { return (int32_t)fi->stat.st_mode; }

// Buffer option field accessors for prep_exarg
// (nvim_buf_get_b_p_fenc, nvim_buf_get_b_p_bin already in change_ffi.c)
// (nvim_buf_set_b_orig_mode already in memline_shim.c)
int nvim_buf_get_b_bad_char(const buf_T *buf) { return buf->b_bad_char; }
int nvim_buf_get_b_p_ff_char(const buf_T *buf) { return (unsigned char)buf->b_p_ff[0]; }

// set_option_direct wrapper for fileencoding
void nvim_set_fileencoding_local(const char *fenc) {
  set_option_direct(kOptFileencoding, CSTR_AS_OPTVAL((char *)fenc), OPT_LOCAL, 0);
}

// Phase 3: shorten_buf_fname / shorten_fnames accessors
void nvim_buf_set_b_sfname(buf_T *buf, char *val) { buf->b_sfname = val; }
void nvim_buf_set_b_fname(buf_T *buf, char *val) { buf->b_fname = val; }
void nvim_buf_mf_fullname(buf_T *buf) { mf_fullname(buf->b_ml.ml_mfp); }
void nvim_set_redraw_tabline(int val) { redraw_tabline = (bool)val; }
int nvim_bt_nofilename(buf_T *buf) { return bt_nofilename(buf) ? 1 : 0; }

// Phase 5: buf_check_timestamp accessors
// --- buf field getters ---
int64_t nvim_buf_get_b_mtime(const buf_T *buf) { return buf->b_mtime; }
int64_t nvim_buf_get_b_mtime_ns(const buf_T *buf) { return buf->b_mtime_ns; }
uint64_t nvim_buf_get_b_orig_size(const buf_T *buf) { return buf->b_orig_size; }
int nvim_buf_get_b_orig_mode(const buf_T *buf) { return buf->b_orig_mode; }
int nvim_buf_get_b_saving(const buf_T *buf) { return (int)buf->b_saving; }
int nvim_buf_get_b_p_ar(const buf_T *buf) { return (int)buf->b_p_ar; }
int nvim_buf_get_b_p_udf(const buf_T *buf) { return (int)buf->b_p_udf; }
int64_t nvim_get_p_ar(void) { return p_ar; }
// --- b_mtime/b_orig manipulation for buf_check_timestamp ---
void nvim_buf_set_b_mtime_minus1(buf_T *buf) { buf->b_mtime = -1; buf->b_orig_size = 0; buf->b_orig_mode = 0; }
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
void nvim_msg_end_wrap(void) { msg_end(); }
// --- home_replace_save wrapper ---
char *nvim_home_replace_save(const buf_T *buf, const char *fname) { return home_replace_save(buf, (char *)fname); }
// --- do_dialog for file-changed warning ---
int nvim_do_dialog_file_changed(const char *tbuf) {
  return do_dialog(VIM_WARNING, _("Warning"), (char *)tbuf,
                   _("&OK\n&Load File\nLoad File &and Options"), 1, NULL, true);
}
// --- undo accessors for buf_reload (used in Phase 5/6) ---
void nvim_u_compute_hash(buf_T *buf, uint8_t *hash) { u_compute_hash(buf, hash); }
void nvim_u_write_undo(const char *name, int forceit, buf_T *buf, uint8_t *hash) {
  u_write_undo((char *)name, (bool)forceit, buf, hash);
}
// buf_reload caller wrapper for use from Rust (Phase 5)
extern void buf_reload(buf_T *buf, int orig_mode, bool reload_options);
void nvim_buf_reload(buf_T *buf, int orig_mode, int reload_options) {
  buf_reload(buf, orig_mode, reload_options != 0);
}

// Phase 6: buf_reload / move_lines accessors
int nvim_get_p_ur(void) { return (int)p_ur; }
int nvim_shortmess_fileinfo(void) { return shortmess(SHM_FILEINFO) ? 1 : 0; }
int nvim_buf_is_empty(buf_T *buf) { return buf_is_empty(buf) ? 1 : 0; }
void nvim_wipe_buffer(buf_T *buf) { wipe_buffer(buf, false); }
void nvim_buf_updates_unload(buf_T *buf) { buf_updates_unload(buf, true); }
void nvim_do_modelines(void) { do_modelines(0); }
void nvim_u_savecommon_reload(buf_T *buf) {
  u_sync(false);
  u_savecommon(buf, 0, buf->b_ml.ml_line_count + 1, 0, true);
}
int nvim_u_savecommon_reload_ok(buf_T *buf) {
  u_sync(false);
  return u_savecommon(buf, 0, buf->b_ml.ml_line_count + 1, 0, true);
}
void nvim_u_clearallandblockfree(buf_T *buf) { u_clearallandblockfree(buf); }
void nvim_u_unchanged(buf_T *buf) { u_unchanged(buf); }
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
void nvim_buf_set_b_p_ro_or(buf_T *buf, int val) { buf->b_p_ro |= (bool)val; }
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
void nvim_set_buf_curwin_buffer(buf_T *buf) { curbuf = buf; curwin->w_buffer = buf; }
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
// READ_NEW and READ_KEEP_UNDO constants
int nvim_READ_NEW(void) { return READ_NEW; }
int nvim_READ_KEEP_UNDO(void) { return READ_KEEP_UNDO; }
