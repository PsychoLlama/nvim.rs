#pragma once

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/option_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

// Opaque handles for quickfix state (for Rust FFI)
typedef void *QfStateHandle;
typedef void *EfmHandle;

/// flags for skip_vimgrep_pat()
enum {
  VGR_GLOBAL = 1,
  VGR_NOJUMP = 2,
  VGR_FUZZY  = 4,
};

// Rust quickfix implementations (called from external C files)
extern void rs_qf_view_result(bool split);
extern void rs_qf_jump_newwin(void *qi, int dir, int errornr, int forceit, bool newwin);

// Rust ex command implementations (called directly from command dispatch table)
extern void rs_ex_cc(void *eap);
extern void rs_ex_cnext(void *eap);
extern void rs_ex_cbelow(void *eap);
extern void rs_ex_cclose(void *eap);
extern void rs_ex_cbottom(void *eap);
extern void rs_ex_cwindow(void *eap);
extern void rs_ex_copen(void *eap);
extern void rs_ex_vimgrep(void *eap);
extern void rs_ex_helpgrep(void *eap);
extern void rs_qf_age(void *eap);
extern void rs_qf_history(void *eap);

#include "quickfix_shim.h.generated.h"
