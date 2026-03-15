#pragma once

#include <stddef.h>
#include <stdbool.h>

#include "nvim/api/keysets_defs.h"  // IWYU pragma: keep
#include "nvim/api/private/defs.h"  // IWYU pragma: keep
#include "nvim/cmdexpand_defs.h"  // IWYU pragma: keep
#include "nvim/highlight_defs.h"
#include "nvim/types_defs.h"  // IWYU pragma: keep

enum { MAX_HL_ID = 20000, };  ///< maximum value for a highlight ID.

typedef struct {
  char *name;
  RgbValue color;
} color_name_table_T;
extern color_name_table_T color_name_table[708];

// Functions implemented in Rust (nvim-highlight crate) via #[export_name]
int highlight_num_groups(void);

// Functions migrated to Rust (highlight_group crate) via #[export_name]
char *highlight_group_name(int id);
int highlight_link_id(int id);
int highlight_group_attr(int id);
bool highlight_group_cleared(int id);
int highlight_group_set(int id);
int highlight_group_parent(int id);
void init_highlight(bool both, bool reset);
bool hl_has_settings(int idx, bool check_link);
void highlight_clear(int idx);
void set_hl_attr(int idx);
void restore_cterm_colors(void);
void highlight_attr_set_all(void);
int load_colors(char *name);
void highlight_list_one(int id);
bool highlight_list_arg(int id, bool didh, int type, int iarg, const char *sarg, const char *name);
bool syn_list_header(bool did_header, int outlen, int id, bool force_newline);
const char *highlight_color(int id, const char *what, int modec);
const char *highlight_has_attr(int id, int flag, int modec);
int highlight_exists(const char *name);
int syn_name2id_len(const char *name, size_t len);
int syn_name2attr(const char *name);
char *syn_id2name(int id);
int syn_check_group(const char *name, size_t len);
int syn_id2attr(int hl_id);
int syn_ns_id2attr(int ns_id, int hl_id, bool *optional);
int syn_get_final_id(int hl_id);
bool syn_ns_get_final_id(int *ns_id, int *hl_idp);
RgbValue name_to_color(const char *name, int *idx);
const char *coloridx_to_name(int idx, int val, char hexbuf[8]);
int name_to_ctermcolor(const char *name);
void syn_init_cmdline_highlight(bool reset, bool init);

#include "highlight_group.h.generated.h"
