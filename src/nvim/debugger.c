/// @file debugger.c
///
/// Vim script debugger functions — thin wrappers delegating to Rust
/// (src/nvim-rs/debugger/src/vim_debugger.rs)
/// C accessor functions for struct/global access needed by Rust.

#include <inttypes.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/debugger.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_getln.h"
#include "nvim/fileio.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/getchar_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/regexp.h"
#include "nvim/runtime.h"
#include "nvim/runtime_defs.h"
#include "nvim/state_defs.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

/// The list of breakpoints: dbg_breakp.
/// This is a grow-array of structs.
struct debuggy {
  int dbg_nr;                   ///< breakpoint number
  int dbg_type;                 ///< DBG_FUNC or DBG_FILE or DBG_EXPR
  char *dbg_name;               ///< function, expression or file name
  regprog_T *dbg_prog;          ///< regexp program
  linenr_T dbg_lnum;            ///< line number in function or file
  int dbg_forceit;              ///< ! used
  typval_T *dbg_val;            ///< last result of watchexpression
  int dbg_level;                ///< stored nested level for expr
};

#include "debugger.c.generated.h"

static garray_T dbg_breakp = { 0, 0, sizeof(struct debuggy), 4, NULL };
#define DEBUGGY(gap, idx)       (((struct debuggy *)(gap)->ga_data)[idx])

// Profiling uses file and func names similar to breakpoints.
static garray_T prof_ga = { 0, 0, sizeof(struct debuggy), 4, NULL };
#define DBG_FUNC        1
#define DBG_FILE        2
#define DBG_EXPR        3

// =============================================================================
// Static assertions for constants used in Rust
// =============================================================================

_Static_assert(DBG_FUNC == 1, "DBG_FUNC mismatch");
_Static_assert(DBG_FILE == 2, "DBG_FILE mismatch");
_Static_assert(DBG_EXPR == 3, "DBG_EXPR mismatch");
_Static_assert(OK == 1, "OK mismatch");
_Static_assert(FAIL == 0, "FAIL mismatch");
_Static_assert(K_SPECIAL == 0x80, "K_SPECIAL mismatch");
_Static_assert(KS_EXTRA == 253, "KS_EXTRA mismatch");
_Static_assert(KE_SNR == 82, "KE_SNR mismatch");
_Static_assert(ESTACK_NONE == 0, "ESTACK_NONE mismatch");
_Static_assert(EXPAND_NOTHING == 0, "EXPAND_NOTHING mismatch");
_Static_assert(MODE_NORMAL == 0x01, "MODE_NORMAL mismatch");
_Static_assert(NUL == 0, "NUL mismatch");
_Static_assert(RE_MAGIC == 1, "RE_MAGIC mismatch");
_Static_assert(RE_STRING == 2, "RE_STRING mismatch");
_Static_assert(UPD_NOT_VALID == 40, "UPD_NOT_VALID mismatch");
_Static_assert(DOCMD_VERBOSE == 0x01, "DOCMD_VERBOSE mismatch");
_Static_assert(DOCMD_EXCRESET == 0x10, "DOCMD_EXCRESET mismatch");
_Static_assert(CMD_profile == 331, "CMD_profile mismatch");
_Static_assert(CMD_profdel == 332, "CMD_profdel mismatch");
_Static_assert(CMD_breakdel == 36, "CMD_breakdel mismatch");
_Static_assert(CMD_breakadd == 35, "CMD_breakadd mismatch");
_Static_assert(EXPR_IS == 9, "EXPR_IS mismatch");

// =============================================================================
// Rust function declarations
// =============================================================================

extern void rs_do_debug(char *cmd);
extern void rs_ex_debug(exarg_T *eap);
extern void rs_ex_debuggreedy(exarg_T *eap);
extern void rs_dbg_check_breakpoint(exarg_T *eap);
extern bool rs_dbg_check_skipped(exarg_T *eap);
extern void rs_ex_breakadd(exarg_T *eap);
extern void rs_ex_breakdel(exarg_T *eap);
extern void rs_ex_breaklist(exarg_T *eap);
extern linenr_T rs_dbg_find_breakpoint(bool file, char *fname, linenr_T after);
extern bool rs_has_profiling(bool file, char *fname, bool *fp);
extern void rs_dbg_breakpoint(char *name, linenr_T lnum);
extern void rs_update_has_expr_breakpoint(void);
extern int rs_typval_compare(typval_T *typ1, typval_T *typ2, int expr_type, int ic);

// =============================================================================
// C accessor functions for Rust to call back into
// =============================================================================

// --- garray_T gap handle getters ---

garray_T *nvim_dbg_get_breakp_gap(void)
{
  return &dbg_breakp;
}

garray_T *nvim_dbg_get_prof_gap(void)
{
  return &prof_ga;
}

// --- garray_T operations ---

int nvim_dbg_gap_len(garray_T *gap)
{
  return gap->ga_len;
}

void nvim_dbg_gap_set_len(garray_T *gap, int len)
{
  gap->ga_len = len;
}

void nvim_dbg_gap_grow(garray_T *gap, int n)
{
  ga_grow(gap, n);
}

void nvim_dbg_gap_clear(garray_T *gap)
{
  ga_clear(gap);
}

bool nvim_dbg_gap_is_empty(garray_T *gap)
{
  return GA_EMPTY(gap);
}

// --- struct debuggy per-field accessors (by index) ---

int nvim_dbg_get_nr(garray_T *gap, int idx)
{
  return DEBUGGY(gap, idx).dbg_nr;
}

void nvim_dbg_set_nr(garray_T *gap, int idx, int val)
{
  DEBUGGY(gap, idx).dbg_nr = val;
}

int nvim_dbg_get_type(garray_T *gap, int idx)
{
  return DEBUGGY(gap, idx).dbg_type;
}

void nvim_dbg_set_type(garray_T *gap, int idx, int val)
{
  DEBUGGY(gap, idx).dbg_type = val;
}

char *nvim_dbg_get_name(garray_T *gap, int idx)
{
  return DEBUGGY(gap, idx).dbg_name;
}

void nvim_dbg_set_name(garray_T *gap, int idx, char *val)
{
  DEBUGGY(gap, idx).dbg_name = val;
}

regprog_T *nvim_dbg_get_prog(garray_T *gap, int idx)
{
  return DEBUGGY(gap, idx).dbg_prog;
}

void nvim_dbg_set_prog(garray_T *gap, int idx, regprog_T *val)
{
  DEBUGGY(gap, idx).dbg_prog = val;
}

linenr_T nvim_dbg_get_lnum(garray_T *gap, int idx)
{
  return DEBUGGY(gap, idx).dbg_lnum;
}

void nvim_dbg_set_lnum(garray_T *gap, int idx, linenr_T val)
{
  DEBUGGY(gap, idx).dbg_lnum = val;
}

int nvim_dbg_get_forceit(garray_T *gap, int idx)
{
  return DEBUGGY(gap, idx).dbg_forceit;
}

void nvim_dbg_set_forceit(garray_T *gap, int idx, int val)
{
  DEBUGGY(gap, idx).dbg_forceit = val;
}

typval_T *nvim_dbg_get_val(garray_T *gap, int idx)
{
  return DEBUGGY(gap, idx).dbg_val;
}

void nvim_dbg_set_val(garray_T *gap, int idx, typval_T *val)
{
  DEBUGGY(gap, idx).dbg_val = val;
}

int nvim_dbg_get_level(garray_T *gap, int idx)
{
  return DEBUGGY(gap, idx).dbg_level;
}

void nvim_dbg_set_level(garray_T *gap, int idx, int val)
{
  DEBUGGY(gap, idx).dbg_level = val;
}

// --- gap entry removal (memmove helper) ---

void nvim_dbg_gap_remove_at(garray_T *gap, int idx)
{
  if (idx < gap->ga_len) {
    memmove(&DEBUGGY(gap, idx), &DEBUGGY(gap, idx + 1),
            (size_t)(gap->ga_len - idx) * sizeof(struct debuggy));
  }
}

int64_t nvim_dbg_get_sourcing_lnum(void)
{
  return (int64_t)SOURCING_LNUM;
}

// --- Buffer/window accessors ---

char *nvim_dbg_curbuf_ffname(void)
{
  return curbuf->b_ffname;
}

linenr_T nvim_dbg_curwin_cursor_lnum(void)
{
  return curwin->w_cursor.lnum;
}

// --- ExArg accessors ---

char *nvim_dbg_eap_get_arg(const exarg_T *eap)
{
  return eap->arg;
}

char *nvim_dbg_eap_get_cmd(const exarg_T *eap)
{
  return eap->cmd;
}

int nvim_dbg_eap_get_skip(const exarg_T *eap)
{
  return eap->skip;
}

void nvim_dbg_eap_set_skip(exarg_T *eap, int val)
{
  eap->skip = val;
}

int nvim_dbg_eap_get_forceit(const exarg_T *eap)
{
  return eap->forceit;
}

int nvim_dbg_eap_get_cmdidx(const exarg_T *eap)
{
  return (int)eap->cmdidx;
}

int nvim_dbg_eap_get_addr_count(const exarg_T *eap)
{
  return eap->addr_count;
}

linenr_T nvim_dbg_eap_get_line2(const exarg_T *eap)
{
  return eap->line2;
}

// =============================================================================
// Message wrappers (keep gettext in C)
// =============================================================================

void nvim_dbg_msg_entering_debug(void)
{
  msg(_("Entering Debug mode.  Type \"cont\" to continue."), 0);
}

void nvim_dbg_smsg_oldval(const char *val)
{
  smsg(0, _("Oldval = \"%s\""), val);
}

void nvim_dbg_smsg_newval(const char *val)
{
  smsg(0, _("Newval = \"%s\""), val);
}

void nvim_dbg_smsg_line_cmd(int64_t lnum, const char *cmd)
{
  smsg(0, _("line %" PRId64 ": %s"), lnum, cmd);
}

void nvim_dbg_smsg_cmd(const char *cmd)
{
  smsg(0, _("cmd: %s"), cmd);
}

void nvim_dbg_smsg_breakpoint(const char *prefix, const char *name, int64_t lnum)
{
  smsg(0, _("Breakpoint in \"%s%s\" line %" PRId64), prefix, name, lnum);
}

void nvim_dbg_smsg_frame_arrow(int num, const char *name)
{
  smsg(0, "->%d %s", num, name);
}

void nvim_dbg_smsg_frame(int num, const char *name)
{
  smsg(0, "  %d %s", num, name);
}

void nvim_dbg_msg_frame_zero(void)
{
  msg(_("frame is zero"), 0);
}

void nvim_dbg_smsg_frame_highest(int max)
{
  smsg(0, _("frame at highest level: %d"), max);
}

void nvim_dbg_smsg_bp_func(int nr, const char *name, int64_t lnum)
{
  smsg(0, _("%3d  %s %s  line %" PRId64), nr, "func", name, lnum);
}

void nvim_dbg_smsg_bp_file(int nr, const char *name, int64_t lnum)
{
  smsg(0, _("%3d  %s %s  line %" PRId64), nr, "file", name, lnum);
}

void nvim_dbg_smsg_bp_expr(int nr, const char *name)
{
  smsg(0, _("%3d  expr %s"), nr, name);
}

void nvim_dbg_msg_no_breakpoints(void)
{
  msg(_("No breakpoints defined"), 0);
}

void nvim_dbg_emsg_noname(void)
{
  emsg(_(e_noname));
}

void nvim_dbg_semsg_invarg(const char *arg)
{
  semsg(_(e_invarg2), arg);
}

void nvim_dbg_semsg_bp_not_found(const char *arg)
{
  semsg(_("E161: Breakpoint not found: %s"), arg);
}

void nvim_dbg_msg_str(const char *s)
{
  msg(s, 0);
}

// =============================================================================
// Eval wrappers
// =============================================================================

typval_T *nvim_dbg_eval_expr(const char *name)
{
  return eval_expr((char *)name, NULL);
}

int nvim_dbg_typval_compare(typval_T *tv1, typval_T *tv2, int ctype, bool ic)
{
  return rs_typval_compare(tv1, tv2, ctype, (int)ic);
}

int64_t nvim_dbg_typval_get_v_number(typval_T *tv)
{
  return (int64_t)tv->vval.v_number;
}

char *nvim_dbg_typval_tostring(typval_T *tv)
{
  return typval_tostring(tv, true);
}

void nvim_dbg_tv_free(typval_T *tv)
{
  tv_free(tv);
}

// =============================================================================
// Command line wrappers
// =============================================================================

char *nvim_dbg_getcmdline_prompt(void)
{
  return getcmdline_prompt('>', NULL, 0, EXPAND_NOTHING, NULL, CALLBACK_NONE,
                           false, NULL);
}

void nvim_dbg_do_cmdline(const char *cmd)
{
  do_cmdline((char *)cmd, getexline, NULL, DOCMD_VERBOSE|DOCMD_EXCRESET);
}

void nvim_dbg_do_cmdline_cmd(const char *cmd)
{
  do_cmdline_cmd((char *)cmd);
}

void nvim_dbg_msg_starthere(void)
{
  msg_starthere();
}

// =============================================================================
// Typeahead wrappers
// =============================================================================

/// Save typeahead state. Returns allocated handle (caller must free via restore).
void *nvim_dbg_save_typeahead(void)
{
  tasave_T *tp = xmalloc(sizeof(tasave_T));
  save_typeahead(tp);
  return tp;
}

/// Restore typeahead state and free the handle.
void nvim_dbg_restore_typeahead(void *handle)
{
  tasave_T *tp = (tasave_T *)handle;
  restore_typeahead(tp);
  xfree(tp);
}

// =============================================================================
// String/memory wrappers
// =============================================================================

char *nvim_dbg_xstrdup(const char *s)
{
  return xstrdup(s);
}

void nvim_dbg_xfree(void *p)
{
  xfree(p);
}

void *nvim_dbg_xmalloc(size_t size)
{
  return xmalloc(size);
}

char *nvim_dbg_skipwhite(const char *p)
{
  return skipwhite(p);
}

bool nvim_dbg_ascii_isdigit(int c)
{
  return ascii_isdigit(c);
}

int32_t nvim_dbg_getdigits_int32(char **pp)
{
  return getdigits_int32(pp, true, 0);
}

size_t nvim_dbg_strlen(const char *s)
{
  return strlen(s);
}

int nvim_dbg_strcmp(const char *a, const char *b)
{
  return strcmp(a, b);
}

int nvim_dbg_strncmp(const char *a, const char *b, size_t n)
{
  return strncmp(a, b, n);
}

char *nvim_dbg_strstr(const char *a, const char *b)
{
  return strstr(a, b);
}

// =============================================================================
// Path/file wrappers
// =============================================================================

char *nvim_dbg_expand_env_save(const char *p)
{
  return expand_env_save((char *)p);
}

char *nvim_dbg_fix_fname(const char *p)
{
  return fix_fname((char *)p);
}

void nvim_dbg_home_replace(const char *name, char *buf, int buflen)
{
  home_replace(NULL, name, buf, buflen, true);
}

char *nvim_dbg_file_pat_to_reg_pat(const char *pat)
{
  return file_pat_to_reg_pat((char *)pat, NULL, NULL, false);
}

// =============================================================================
// Regex wrappers
// =============================================================================

regprog_T *nvim_dbg_vim_regcomp(const char *pat, int flags)
{
  return vim_regcomp((char *)pat, flags);
}

void nvim_dbg_vim_regfree(regprog_T *prog)
{
  vim_regfree(prog);
}

bool nvim_dbg_vim_regexec_prog(regprog_T **prog_ptr, const char *name)
{
  return vim_regexec_prog(prog_ptr, false, (char *)name, 0);
}

// =============================================================================
// Screen/misc wrappers
// =============================================================================

void nvim_dbg_redraw_all_later(int typ)
{
  redraw_all_later(typ);
}

char *nvim_dbg_estack_sfile(int which)
{
  return estack_sfile((estack_arg_T)which);
}

bool nvim_dbg_ascii_isalpha(int c)
{
  return ASCII_ISALPHA(c);
}

// =============================================================================
// Thin wrappers delegating to Rust
// =============================================================================

void do_debug(char *cmd)
{
  rs_do_debug(cmd);
}

void ex_debug(exarg_T *eap)
{
  rs_ex_debug(eap);
}

void ex_debuggreedy(exarg_T *eap)
{
  rs_ex_debuggreedy(eap);
}

void dbg_check_breakpoint(exarg_T *eap)
{
  rs_dbg_check_breakpoint(eap);
}

bool dbg_check_skipped(exarg_T *eap)
{
  return rs_dbg_check_skipped(eap);
}

void ex_breakadd(exarg_T *eap)
{
  rs_ex_breakadd(eap);
}

void ex_breakdel(exarg_T *eap)
{
  rs_ex_breakdel(eap);
}

void ex_breaklist(exarg_T *eap)
{
  rs_ex_breaklist(eap);
}

linenr_T dbg_find_breakpoint(bool file, char *fname, linenr_T after)
{
  return rs_dbg_find_breakpoint(file, fname, after);
}

bool has_profiling(bool file, char *fname, bool *fp)
{
  return rs_has_profiling(file, fname, fp);
}

void dbg_breakpoint(char *name, linenr_T lnum)
{
  rs_dbg_breakpoint(name, lnum);
}
