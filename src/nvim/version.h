#pragma once

#include "nvim/api/private/defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/macros_defs.h"

// defined in version.c
extern char *Versions[];
extern char *longVersion;
#ifndef NDEBUG
extern char *version_cflags;
#endif

// Rust-implemented functions (exported from the version crate)
void version_msg(char *s);
void list_version(void);
void list_lua_version(void);
void ex_version(exarg_T *eap);
bool may_show_intro(void);
void intro_message(bool colon);
void ex_intro(exarg_T *eap);

#include "version.h.generated.h"
bool has_nvim_version(const char *version_str);
int min_vim_version(void);
int highest_patch(void);
bool has_vim_patch(int n, int major_minor_version);
