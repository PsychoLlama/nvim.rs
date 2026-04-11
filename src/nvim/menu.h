#pragma once

#include "nvim/cmdexpand_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/menu_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

#include "menu.h.generated.h"

// Declarations for Rust-exported menu functions (via #[export_name]).
// These were previously generated from C wrappers but are now implemented directly in Rust.
bool menu_is_menubar(const char *name);
bool menu_is_popup(const char *name);
bool menu_is_toolbar(const char *name);
bool menu_is_separator(char *name);
int get_menu_mode_flag(void);
void show_popupmenu(void);
void ex_menu(exarg_T *eap);
void ex_emenu(exarg_T *eap);
vimmenu_T *menu_find(const char *path_name);
void ex_menutranslate(exarg_T *eap);
char *set_context_in_menu_cmd(expand_T *xp, const char *cmd, char *arg, bool forceit);
char *get_menu_name(expand_T *xp, int idx);
char *get_menu_names(expand_T *xp, int idx);
bool menu_get(char *path_name, int modes, list_T *list);
void f_menu_info(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
