#pragma once

#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/regexp_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

// Second argument for vim_regcomp().
#define RE_MAGIC        1       ///< 'magic' option
#define RE_STRING       2       ///< match in string instead of buffer text
#define RE_STRICT       4       ///< don't allow [abc] without ]
#define RE_AUTO         8       ///< automatic engine selection
#define RE_NOBREAK      16      ///< don't use breakcheck functions

// values for reg_do_extmatch
#define REX_SET        1       ///< to allow \z\(...\),
#define REX_USE        2       ///< to allow \z\1 et al.
#define REX_ALL       (REX_SET | REX_USE)

// Functions directly exported from Rust (no C wrapper).
extern int re_multiline(const regprog_T *prog);
extern reg_extmatch_T *ref_extmatch(reg_extmatch_T *em);
extern void unref_extmatch(reg_extmatch_T *em);
extern char *regtilde(char *source, int magic, bool preview);
extern char *reg_submatch(int no);
extern list_T *reg_submatch_list(int no);
extern char *skip_regexp_err(char *startp, int delim, int magic);
extern void vim_regfree(regprog_T *prog);
extern void free_regexp_stuff(void);

// Forward declarations for Rust-implemented functions (exported under C names via #[export_name])
int vim_regcomp_had_eol(void);
regprog_T *vim_regcomp(const char *expr_arg, int re_flags);
int vim_regsub(regmatch_T *rmp, char *source, typval_T *expr, char *dest, int destlen, int flags);
int vim_regexec_multi(regmmatch_T *rmp, win_T *win, buf_T *buf, linenr_T lnum, colnr_T col,
                      proftime_T *tm, int *timed_out);
bool vim_regexec_prog(regprog_T **prog, bool ignore_case, const char *line, colnr_T col);
bool vim_regexec(regmatch_T *rmp, const char *line, colnr_T col);
bool vim_regexec_nl(regmatch_T *rmp, const char *line, colnr_T col);
char *skip_regexp_ex(char *startp, int dirc, int magic, char **newp, int *dropped, magic_T *magic_val);
char *skip_regexp(char *startp, int delim, int magic);
int vim_regsub_multi(regmmatch_T *rmp, linenr_T lnum, char *source, char *dest, int destlen, int flags);

#include "regexp_shim.h.generated.h"
