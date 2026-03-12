#pragma once

#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep

/// flags for do_ecmd()
enum {
  ECMD_HIDE       = 0x01,  ///< don't free the current buffer
  ECMD_SET_HELP   = 0x02,  ///< set b_help flag of (new) buffer before opening file
  ECMD_OLDBUF     = 0x04,  ///< use existing buffer if it exists
  ECMD_FORCEIT    = 0x08,  ///< ! used in Ex command
  ECMD_ADDBUF     = 0x10,  ///< don't edit, just add to buffer list
  ECMD_ALTBUF     = 0x20,  ///< like ECMD_ADDBUF and set the alternate file
  ECMD_NOWINENTER = 0x40,  ///< do not trigger BufWinEnter
};

/// for lnum argument in do_ecmd()
enum {
  ECMD_LASTL = 0,   ///< use last position in loaded file
  ECMD_LAST  = -1,  ///< use last position in all files
  ECMD_ONE   = 1,   ///< use first line
};

// Rust Ex command implementations (dispatch table targets)
extern void rs_do_ascii(exarg_T *eap);
extern void rs_ex_align(exarg_T *eap);
extern void rs_ex_append(exarg_T *eap);
extern void rs_ex_change(exarg_T *eap);
extern void rs_ex_echohl(exarg_T *eap);
extern void rs_ex_sort(exarg_T *eap);
extern void rs_ex_uniq(exarg_T *eap);
extern void rs_ex_z(exarg_T *eap);

// Rust utility functions
extern int rs_check_secure(void);
extern void rs_print_line(int lnum, int use_number, int list, int first);
extern void rs_print_line_no_prefix(int lnum, int use_number, int list);
extern int rs_do_ecmd(int fnum, char *ffname, char *sfname, exarg_T *eap,
                      int newlnum, int flags, win_T *oldwin);

// Forward declarations for Rust-implemented functions (exported under C names via #[export_name])
bool do_sub_msg(bool count_only);
bool prepare_tagpreview(bool undo_sync);
void ex_substitute(exarg_T *eap);
int ex_substitute_preview(exarg_T *eap, int cmdpreview_ns, handle_T cmdpreview_bufnr);
void ex_file(exarg_T *eap);
void ex_update(exarg_T *eap);
void ex_write(exarg_T *eap);
void ex_wnext(exarg_T *eap);
void do_wqall(exarg_T *eap);
void ex_global(exarg_T *eap);
void ex_oldfiles(exarg_T *eap);
void do_shell(char *cmd, int flags);
void global_exe(char *cmd);
void free_old_sub(void);
#if defined(EXITFREE)
void free_prev_shellcmd(void);
#endif

#include "ex_cmds_shim.h.generated.h"
