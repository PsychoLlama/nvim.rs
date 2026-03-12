#pragma once

#include <stdio.h>  // IWYU pragma: keep

#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep

int put_eol(FILE *fd);
int put_line(FILE *fd, const char *s);
void ex_loadview(exarg_T *eap);
void ex_mkrc(exarg_T *eap);
