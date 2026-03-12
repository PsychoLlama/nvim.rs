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
// IWYU pragma: end_exports

// Functions implemented in Rust (src/nvim-rs/os/src/users.rs)
int os_get_usernames(garray_T *users);
int os_get_username(char *s, size_t len);
int os_get_uname(uv_uid_t uid, char *s, size_t len);
char *os_get_userdir(const char *name);
void free_users(void);
char *get_users(expand_T *xp, int idx);
int match_user(char *name);

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

// Rust-provided replacements for env.c functions (Phase 2 migration).
const void *vim_env_iter(char delim, const char *val, const void *iter, const char **dir,
                         size_t *len);
const void *vim_env_iter_rev(char delim, const char *val, const void *iter, const char **dir,
                             size_t *len);
char *expand_env_save(char *src);
char *expand_env_save_opt(char *src, bool one);
size_t expand_env(char *src, char *dst, int dstlen);
char *get_env_name(expand_T *xp, int idx);
bool os_setenv_append_path(const char *fname);
bool os_shell_is_cmdexe(const char *sh);
void vim_unsetenv_ext(const char *var);
void vim_setenv_ext(const char *name, const char *val);
char *home_replace_save(buf_T *buf, const char *src);

#define ENV_LOGFILE "NVIM_LOG_FILE"
#define ENV_NVIM "NVIM"
