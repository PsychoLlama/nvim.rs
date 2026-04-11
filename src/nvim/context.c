// Context: snapshot of the entire editor state as one big object/map
// Most logic has been migrated to Rust (src/nvim-rs/context/).
// This file only provides the kCtxAll and ctx_stack global definitions.

#include "nvim/context.h"

#include "context.c.generated.h"

int kCtxAll = (kCtxRegs | kCtxJumps | kCtxBufs | kCtxGVars | kCtxSFuncs
               | kCtxFuncs);

ContextVec ctx_stack = KV_INITIAL_VALUE;
