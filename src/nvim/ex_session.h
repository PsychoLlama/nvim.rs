#pragma once

#include <stdio.h>  // IWYU pragma: keep

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/garray_defs.h"  // IWYU pragma: keep

// Rust FFI: session writer (Phase 1)
extern int rs_put_eol(FILE *fd);
extern int rs_put_line(FILE *fd, const char *s);

// Rust FFI: window/frame predicates (Phase 2)
extern int rs_ses_do_win(win_T *wp);
extern bool rs_ses_do_frame(const frame_T *fr);
extern frame_T *rs_ses_skipframe(frame_T *fr);

// Rust FFI: filename helpers (Phase 3)
extern char *rs_ses_get_fname(buf_T *buf, const unsigned *flagp);
extern char *rs_ses_escape_fname(char *name, unsigned *flagp);
extern int rs_ses_put_fname(FILE *fd, char *name, unsigned *flagp);
extern int rs_ses_fname(FILE *fd, buf_T *buf, unsigned *flagp, bool add_eol);

// Rust FFI: layout writers (Phase 4)
extern int rs_put_view_curpos(FILE *fd, win_T *wp, char *spaces);
extern int rs_ses_winsizes(FILE *fd, bool restore_size, win_T *tab_firstwin);
extern int rs_ses_arglist(FILE *fd, char *cmd, garray_T *gap, bool fullname, unsigned *flagp);
extern int rs_ses_win_rec(FILE *fd, frame_T *fr);

#include "ex_session.h.generated.h"
