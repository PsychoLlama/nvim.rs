/// message_ffi.c: Error message helpers for Rust FFI.
///
/// These thin wrappers emit specific Neovim error messages that are referenced
/// by Rust crates.  The underlying error constants (e_noprevre etc.) are
/// defined as extern variables in errors.h and cannot be referenced directly
/// from Rust without going through a C wrapper.

#include "nvim/errors.h"
#include "nvim/message.h"
#include "nvim/gettext_defs.h"

#include "message_ffi.c.generated.h"

/// Emit "E35: No previous regular expression" error.
void nvim_emsg_noprevre(void) { emsg(_(e_noprevre)); }

/// Emit "E29: No inserted text yet" error.
void nvim_emsg_noinstext(void) { emsg(_(e_noinstext)); }

/// Emit "E30: No previous command line" error.
void nvim_emsg_nolastcmd(void) { emsg(_(e_nolastcmd)); }
