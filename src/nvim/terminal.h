#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "nvim/api/private/defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

// Forward declarations for vterm types (opaque pointers for Rust accessors)
typedef struct VTerm VTerm;
typedef struct VTermScreen VTermScreen;

typedef void (*terminal_write_cb)(const char *buffer, size_t size, void *data);
typedef void (*terminal_resize_cb)(uint16_t width, uint16_t height, void *data);
typedef void (*terminal_close_cb)(void *data);

typedef struct {
  void *data;  // PTY process channel
  uint16_t width, height;
  terminal_write_cb write_cb;
  terminal_resize_cb resize_cb;
  terminal_close_cb close_cb;
  bool force_crlf;
} TerminalOptions;

typedef void (*emit_termrequest_fn_t)(void **);

// Functions provided by Rust (nvim-terminal crate via #[export_name]).
// These symbols are defined in the Rust nvim-terminal crate.
Buffer terminal_buf(const Terminal *term);
bool terminal_running(const Terminal *term);

#include "terminal_shim.h.generated.h"
