#pragma once

#include "nvim/ascii_defs.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/register_defs.h"

#include "register.h.generated.h"
#include "register.h.inline.generated.h"

extern int rs_is_literal_register(int regname);
extern int rs_op_reg_index(int regname);
extern int rs_is_append_register(int regname);
extern int rs_get_register_name(int num);

// Functions implemented in Rust (src/nvim-rs/register/src/lib.rs) with #[export_name].
// These replace the former C implementations that have been deleted.
extern int insert_reg(int regname, yankreg_T *reg, bool literally_arg);
extern bool get_spec_reg(int regname, char **argp, bool *allocated, bool errmsg);
extern void op_yank_reg(oparg_T *oap, bool message, yankreg_T *reg, bool append);
extern void do_autocmd_textyankpost(oparg_T *oap, yankreg_T *reg);
extern bool op_yank(oparg_T *oap, bool message);
extern bool cmdline_paste_reg(int regname, bool literally_arg, bool remcr);
extern void *get_reg_contents(int regname, int flags);
extern int do_execreg(int regname, int colon, int addcr, int silent);
extern int do_record(int c);
extern void ex_display(exarg_T *eap);
extern int rs_get_unname_register(void);
extern int get_unname_register(void);
extern yankreg_T *get_y_register(int reg);
extern yankreg_T *get_y_previous(void);
extern int get_expr_register(void);
extern void set_expr_line(char *new_line);
extern char *get_expr_line(void);
extern char *get_expr_line_src(void);
extern bool valid_yank_reg(int regname, bool writing);
extern int get_default_register_name(void);
extern const void *op_reg_iter(const void *const iter, const yankreg_T *const regs, char *const name, yankreg_T *const reg, bool *is_unnamed);
extern const void *op_global_reg_iter(const void *const iter, char *const name, yankreg_T *const reg, bool *is_unnamed);
extern size_t op_reg_amount(void);
extern bool op_reg_set(const char name, const yankreg_T reg, bool is_unnamed);
extern const yankreg_T *op_reg_get(const char name);
extern bool op_reg_set_previous(const char name);
extern void update_yankreg_width(yankreg_T *reg);
extern yankreg_T *get_yank_register(int regname, int mode);
extern bool yank_register_mline(int regname, yankreg_T **reg);
extern yankreg_T *copy_register(int name);
extern void shift_delete_registers(bool y_append);
extern void free_register(yankreg_T *reg);
extern MotionType get_reg_type(int regname, colnr_T *reg_width);
extern void format_reg_type(MotionType reg_type, colnr_T reg_width, char *buf, size_t buf_len);
extern void write_reg_contents(int name, const char *str, ssize_t len, int must_append);
extern void write_reg_contents_lst(int name, char **strings, bool must_append, MotionType yank_type, colnr_T block_len);
extern void write_reg_contents_ex(int name, const char *str, ssize_t len, bool must_append, MotionType yank_type, colnr_T block_len);
extern bool prepare_yankreg_from_object(yankreg_T *reg, String regtype, size_t lines);
extern void finish_yankreg_from_object(yankreg_T *reg, bool clipboard_adjust);
#if defined(EXITFREE)
extern void clear_registers(void);
#endif

/// @see get_yank_register
/// @return  true when register should be inserted literally
/// (selection or clipboard)
static inline bool is_literal_register(const int regname)
  FUNC_ATTR_CONST
{
  return rs_is_literal_register(regname) != 0;
}

/// Convert register name into register index
///
/// @param[in]  regname  Register name.
///
/// @return Index in y_regs array or -1 if register name was not recognized.
static inline int op_reg_index(const int regname)
  FUNC_ATTR_CONST
{
  return rs_op_reg_index(regname);
}

static inline bool is_append_register(int regname)
  FUNC_ATTR_CONST
{
  return rs_is_append_register(regname) != 0;
}

/// @return  the character name of the register with the given number
static inline int get_register_name(int num)
  FUNC_ATTR_CONST
{
  return rs_get_register_name(num);
}

/// Check whether register is empty
static inline bool reg_empty(const yankreg_T *const reg)
  FUNC_ATTR_PURE
{
  return (reg->y_array == NULL
          || reg->y_size == 0
          || (reg->y_size == 1
              && reg->y_type == kMTCharWise
              && reg->y_array[0].size == 0));
}
