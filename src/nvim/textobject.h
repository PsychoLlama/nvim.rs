#pragma once

#include <stdbool.h>

#include "nvim/normal_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/vim_defs.h"  // IWYU pragma: keep

// Functions exported from Rust (via #[export_name])
int findsent(Direction dir, int count);
bool findpar(bool *pincl, int dir, int count, int what, bool both);
bool startPS(linenr_T lnum, int para, bool both);
int fwd_word(int count, bool bigword, bool eol);
int bck_word(int count, bool bigword, bool stop);
int end_word(int count, bool bigword, bool stop, bool empty);
int bckend_word(int count, bool bigword, bool eol);
int current_word(oparg_T *oap, int count, bool include, bool bigword);
int current_sent(oparg_T *oap, int count, bool include);
int current_block(oparg_T *oap, int count, bool include, int what, int other);
int current_par(oparg_T *oap, int count, bool include, int type);

#include "textobject.h.generated.h"
