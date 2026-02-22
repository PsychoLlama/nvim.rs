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
extern void rs_ex_change(exarg_T *eap);
extern void rs_ex_z(exarg_T *eap);

// Rust utility functions
extern int rs_check_secure(void);

#include "ex_cmds_shim.h.generated.h"
