#pragma once

#include <stdbool.h>

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

/// flags for check_changed()
enum {
  CCGD_AW      = 1,   ///< do autowrite if buffer was changed
  CCGD_MULTWIN = 2,   ///< check also when several wins for the buf
  CCGD_FORCEIT = 4,   ///< ! used
  CCGD_ALLBUF  = 8,   ///< may write all buffers
  CCGD_EXCMD   = 16,  ///< may suggest using !
};

// Functions implemented in Rust (src/nvim-rs/ex_cmds2/) and exported directly
// via #[export_name]. Declared here since they no longer appear in ex_cmds2.c.
int autowrite(buf_T *buf, bool forceit);
void autowrite_all(void);
bool check_changed(buf_T *buf, int flags);
void dialog_changed(buf_T *buf, bool checkall);
bool dialog_close_terminal(buf_T *buf);
bool can_abandon(buf_T *buf, bool forceit);
bool check_changed_any(bool hidden, bool unload);
int check_fname(void);
int buf_write_all(buf_T *buf, bool forceit);
void ex_listdo(exarg_T *eap);
void ex_compiler(exarg_T *eap);
void ex_checktime(exarg_T *eap);
void ex_drop(exarg_T *eap);
void ex_ruby(exarg_T *eap);
void ex_rubyfile(exarg_T *eap);
void ex_rubydo(exarg_T *eap);
void ex_python3(exarg_T *eap);
void ex_py3file(exarg_T *eap);
void ex_pydo3(exarg_T *eap);
void ex_perl(exarg_T *eap);
void ex_perlfile(exarg_T *eap);
void ex_perldo(exarg_T *eap);

#include "ex_cmds2.h.generated.h"
