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

#include "version.h.generated.h"
bool has_nvim_version(const char *version_str);
int min_vim_version(void);
int highest_patch(void);
bool has_vim_patch(int n, int major_minor_version);
