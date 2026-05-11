#include <assert.h>
#include <lauxlib.h>
#include <lua.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/eval/decode.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/typval_encode.h"
#include "nvim/eval/userfunc.h"
#include "nvim/gettext_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/lua/converter.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

/// Determine, which keys Lua table contains
typedef struct {
  size_t maxidx;  ///< Maximum positive integral value found.
  size_t string_keys_num;  ///< Number of string keys.
  bool has_string_with_nul;  ///< True if there is string key with NUL byte.
  ObjectType type;  ///< If has_type_key is true then attached value. Otherwise
                    ///< either kObjectTypeNil, kObjectTypeDict or
                    ///< kObjectTypeArray, depending on other properties.
  lua_Number val;  ///< If has_val_key and val_type == LUA_TNUMBER: value.
  bool has_type_key;  ///< True if type key is present.
} LuaTableProps;

#include "lua/converter.c.generated.h"

#define TYPE_IDX_VALUE true
#define VAL_IDX_VALUE false

#define LUA_PUSH_STATIC_STRING(lstate, s) \
  lua_pushlstring(lstate, s, sizeof(s) - 1)

// These helpers are still used by the macro forest (TYPVAL_ENCODE_CONV_EMPTY_DICT)
// and by nlua_init_types below. They are defined here so that C code calling
// them continues to compile while those callers remain in C. Once all callers
// are migrated to Rust, these can be removed.
static inline void nlua_push_type_idx(lua_State *lstate)
  FUNC_ATTR_NONNULL_ALL
{
  lua_pushboolean(lstate, TYPE_IDX_VALUE);
}

static inline void nlua_push_val_idx(lua_State *lstate)
  FUNC_ATTR_NONNULL_ALL
{
  lua_pushboolean(lstate, VAL_IDX_VALUE);
}

static inline void nlua_push_type(lua_State *lstate, ObjectType type)
  FUNC_ATTR_NONNULL_ALL
{
  lua_pushnumber(lstate, (lua_Number)type);
}

static inline void nlua_create_typed_table(lua_State *lstate, const size_t narr, const size_t nrec,
                                           const ObjectType type)
  FUNC_ATTR_NONNULL_ALL
{
  lua_createtable(lstate, (int)narr, (int)(1 + nrec));
  nlua_push_type_idx(lstate);
  nlua_push_type(lstate, type);
  lua_rawset(lstate, -3);
}

static LuaTableProps nlua_traverse_table(lua_State *const lstate)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT
{
  size_t tsize = 0;  // Total number of keys.
  int val_type = 0;  // If has_val_key: Lua type of the value.
  bool has_val_key = false;  // True if val key was found,
                             // @see nlua_push_val_idx().
  size_t other_keys_num = 0;  // Number of keys that are not string, integral
                              // or type keys.
  LuaTableProps ret;
  CLEAR_FIELD(ret);
  if (!lua_checkstack(lstate, lua_gettop(lstate) + 3)) {
    semsg(_("E1502: Lua failed to grow stack to %i"), lua_gettop(lstate) + 2);
    ret.type = kObjectTypeNil;
    return ret;
  }
  lua_pushnil(lstate);
  while (lua_next(lstate, -2)) {
    switch (lua_type(lstate, -2)) {
    case LUA_TSTRING: {
      size_t len;
      const char *s = lua_tolstring(lstate, -2, &len);
      if (memchr(s, NUL, len) != NULL) {
        ret.has_string_with_nul = true;
      }
      ret.string_keys_num++;
      break;
    }
    case LUA_TNUMBER: {
      const lua_Number n = lua_tonumber(lstate, -2);
      if (n > (lua_Number)SIZE_MAX || n <= 0
          || ((lua_Number)((size_t)n)) != n) {
        other_keys_num++;
      } else {
        const size_t idx = (size_t)n;
        if (idx > ret.maxidx) {
          ret.maxidx = idx;
        }
      }
      break;
    }
    case LUA_TBOOLEAN: {
      const bool b = lua_toboolean(lstate, -2);
      if (b == TYPE_IDX_VALUE) {
        if (lua_type(lstate, -1) == LUA_TNUMBER) {
          lua_Number n = lua_tonumber(lstate, -1);
          if (n == (lua_Number)kObjectTypeFloat
              || n == (lua_Number)kObjectTypeArray
              || n == (lua_Number)kObjectTypeDict) {
            ret.has_type_key = true;
            ret.type = (ObjectType)n;
          } else {
            other_keys_num++;
          }
        } else {
          other_keys_num++;
        }
      } else {
        has_val_key = true;
        val_type = lua_type(lstate, -1);
        if (val_type == LUA_TNUMBER) {
          ret.val = lua_tonumber(lstate, -1);
        }
      }
      break;
    }
    default:
      other_keys_num++;
      break;
    }
    tsize++;
    lua_pop(lstate, 1);
  }
  if (ret.has_type_key) {
    assert(tsize > 0);
    if (ret.type == kObjectTypeFloat
        && (!has_val_key || val_type != LUA_TNUMBER)) {
      ret.type = kObjectTypeNil;
    } else if (ret.type == kObjectTypeArray) {
      // Determine what is the last number in a *sequence* of keys.
      // This condition makes sure that Neovim will not crash when it gets table
      // {[vim.type_idx]=vim.types.array, [SIZE_MAX]=1}: without it maxidx will
      // be SIZE_MAX, with this condition it should be zero and [SIZE_MAX] key
      // should be ignored.
      if (ret.maxidx != 0
          && ret.maxidx != (tsize
                            - ret.has_type_key
                            - other_keys_num
                            - has_val_key
                            - ret.string_keys_num)) {
        for (ret.maxidx = 0;; ret.maxidx++) {
          lua_rawgeti(lstate, -1, (int)ret.maxidx + 1);
          if (lua_isnil(lstate, -1)) {
            lua_pop(lstate, 1);
            break;
          }
          lua_pop(lstate, 1);
        }
      }
    }
  } else {
    if (tsize == 0
        || (tsize <= ret.maxidx
            && other_keys_num == 0
            && ret.string_keys_num == 0)) {
      ret.type = kObjectTypeArray;
      if (tsize == 0 && lua_getmetatable(lstate, -1)) {
        nlua_pushref(lstate, nlua_global_refs->empty_dict_ref);
        if (lua_rawequal(lstate, -2, -1)) {
          ret.type = kObjectTypeDict;
        }
        lua_pop(lstate, 2);
      }
    } else if (ret.string_keys_num == tsize) {
      ret.type = kObjectTypeDict;
    } else {
      ret.type = kObjectTypeNil;
    }
  }
  return ret;
}

/// Helper structure for nlua_pop_typval
typedef struct {
  typval_T *tv;     ///< Location where conversion result is saved.
  size_t list_len;  ///< Maximum length when tv is a list.
  bool container;   ///< True if tv is a container.
  bool special;     ///< If true then tv is a _VAL part of special dict.
                    ///< that represents mapping.
  int idx;          ///< Container index (used to detect self-referencing structures).
} TVPopStackItem;

// nlua_pop_typval migrated to Rust (Phase 10).
// Implementation in src/nvim-rs/lua/src/api.rs (rs_nlua_pop_typval, exported as nlua_pop_typval).


