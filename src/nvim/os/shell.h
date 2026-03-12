#pragma once

#include <stddef.h>  // IWYU pragma: keep

#include "nvim/event/defs.h"  // IWYU pragma: keep (Stream)

// Flags for os_call_shell() second argument
typedef enum {
  kShellOptFilter = 1,     ///< filtering text
  kShellOptExpand = 2,     ///< expanding wildcards
  kShellOptDoOut = 4,      ///< redirecting output
  kShellOptSilent = 8,     ///< don't print error returned by command
  kShellOptRead = 16,      ///< read lines and insert into buffer
  kShellOptWrite = 32,     ///< write lines from buffer
  kShellOptHideMess = 64,  ///< previously a global variable from os_unix.c
} ShellOpts;

// Rust-provided replacements for shell.c functions (Phase 3 migration).
char **shell_build_argv(const char *cmd, const char *extra_args);
void shell_free_argv(char **argv);
char *shell_argv_to_str(char **argv);
void shell_write_cb(Stream *stream, void *data, int status);

#include "os/shell.h.generated.h"
