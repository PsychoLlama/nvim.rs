#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <uv.h>

#include "nvim/cmdexpand_defs.h"
#include "nvim/garray_defs.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/stdpaths_defs.h"
#include "nvim/types_defs.h"

extern char *default_vim_dir;
extern char *default_vimruntime_dir;
extern char *default_lib_dir;

// IWYU pragma: begin_exports
#include "os/env.h.generated.h"
#include "os/mem.h.generated.h"
#include "os/stdpaths.h.generated.h"
#include "os/users.h.generated.h"
// IWYU pragma: end_exports

// Rust-provided replacements for env.c functions (Phase 1 migration).
char *os_getenv(const char *name);
bool os_env_exists(const char *name, bool nonempty);
int os_setenv(const char *name, const char *value, int overwrite);
int os_unsetenv(const char *name);
int64_t os_get_pid(void);
void os_get_hostname(char *hostname, size_t size);
size_t os_get_fullenv_size(void);
void os_copy_fullenv(char **env, size_t env_size);
void os_free_fullenv(char **env);
char *os_getenvname_at_index(size_t index);
void os_hint_priority(void);

#define ENV_LOGFILE "NVIM_LOG_FILE"
#define ENV_NVIM "NVIM"
