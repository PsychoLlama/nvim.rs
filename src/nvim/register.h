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
