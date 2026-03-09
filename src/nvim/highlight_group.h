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
void restore_cterm_colors(void);
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

#include "highlight_group.h.generated.h"
