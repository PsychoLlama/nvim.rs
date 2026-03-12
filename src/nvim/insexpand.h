#pragma once

#include "nvim/option_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep
#include "nvim/vim_defs.h"  // IWYU pragma: keep

// Forward declarations for Rust-implemented functions (exported under C names via #[export_name])
bool check_compl_option(bool dict_opt);
void ins_ctrl_x(void);
const char *did_set_completefunc(optset_T *args);
const char *did_set_omnifunc(optset_T *args);
const char *did_set_thesaurusfunc(optset_T *args);
bool set_ref_in_insexpand_funcs(int copyID);

#include "insexpand_shim.h.generated.h"

// Rust-implemented functions (rs_ prefix)
extern void rs_ins_compl_show_pum(void);
extern int rs_ins_compl_col_range_attr(int lnum, int col);
extern int rs_vim_is_ctrl_x_key(int c);
extern int rs_ins_compl_pum_key(int c);
extern void rs_ins_compl_addleader(int c);
extern int rs_ins_compl_stop(int c, int prev_mode, int retval);
extern int rs_ins_compl_cancel(void);
extern int rs_ins_compl_prep(int c);
extern void rs_ins_compl_delete(int new_leader);
extern void rs_ins_compl_insert(int move_cursor, int insert_prefix);
extern void rs_ins_compl_check_keys(int frequency, int in_compl_func);
extern int rs_ins_compl_bs(void);
extern void rs_ins_compl_fuzzy_sort(void);
extern void rs_sort_compl_match_list(int compare_type);
extern void rs_ins_compl_new_leader(void);
extern void rs_ins_compl_del_pum(void);
extern void rs_ins_compl_addfrommatch(void);

/// Array indexes used for cp_text[].
typedef enum {
  CPT_ABBR,   ///< "abbr"
  CPT_KIND,   ///< "kind"
  CPT_MENU,   ///< "menu"
  CPT_INFO,   ///< "info"
  CPT_COUNT,  ///< Number of entries
} cpitem_T;
