#pragma once

#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep

// Rust-exported functions (from nvim-rs/help)
void ex_help(exarg_T *eap);
void ex_helpclose(exarg_T *eap);
char *check_help_lang(char *arg);
int help_heuristic(char *matched_string, int offset, bool wrong_case);
int help_compare(const void *s1, const void *s2);
int find_help_tags(const char *arg, int *num_matches, char ***matches, bool keep_lang);
void cleanup_help_tags(int num_file, char **file);
void prepare_help_buffer(void);
void get_local_additions(void);
void ex_exusage(exarg_T *eap);
void ex_viusage(exarg_T *eap);
void helptags_one(char *dir, const char *ext, const char *tagfname, bool add_help_tags, bool ignore_writeerr);
void do_helptags(char *dirname, bool add_help_tags, bool ignore_writeerr);
void ex_helptags(exarg_T *eap);

#include "help.h.generated.h"
