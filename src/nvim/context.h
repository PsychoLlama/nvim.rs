#pragma once

#include <stddef.h>  // IWYU pragma: keep

#include "klib/kvec.h"
#include "nvim/api/private/defs.h"
#include "nvim/memory_defs.h"

typedef struct {
  String regs;     ///< Registers.
  String jumps;    ///< Jumplist.
  String bufs;     ///< Buffer list.
  String gvars;    ///< Global variables.
  Array funcs;              ///< Functions.
} Context;
typedef kvec_t(Context) ContextVec;

extern ContextVec ctx_stack;

#define CONTEXT_INIT (Context) { \
  .regs = STRING_INIT, \
  .jumps = STRING_INIT, \
  .bufs = STRING_INIT, \
  .gvars = STRING_INIT, \
  .funcs = ARRAY_DICT_INIT, \
}

typedef enum {
  kCtxRegs = 1,       ///< Registers
  kCtxJumps = 2,      ///< Jumplist
  kCtxBufs = 4,       ///< Buffer list
  kCtxGVars = 8,      ///< Global variables
  kCtxSFuncs = 16,    ///< Script functions
  kCtxFuncs = 32,     ///< Functions
} ContextTypeFlags;

extern int kCtxAll;

// Functions implemented in Rust (export_name matches these symbols)
void ctx_free_all(void);
size_t ctx_size(void);
Context *ctx_get(size_t index);
void ctx_free(Context *ctx);
void ctx_save(Context *ctx, int flags);
bool ctx_restore(Context *ctx, int flags);
Dict ctx_to_dict(Context *ctx, Arena *arena);
int ctx_from_dict(Dict dict, Context *ctx, Error *err);

// Prototypes for C accessor functions (generated from context.c)
#include "context.h.generated.h"
