#include "nvim/map_defs.h"
#include "nvim/memory.h"

#ifndef KEY_NAME
// Don't error out. it is nice to type-check the file in isolation, in clangd or otherwise
# define KEY_NAME(x) x##int
#endif

#define SET_TYPE KEY_NAME(Set_)
#define KEY_TYPE KEY_NAME()

// Rust extern declarations
extern uint32_t KEY_NAME(rs_mh_find_bucket_)(SET_TYPE *set, KEY_TYPE key, bool put);
extern uint32_t KEY_NAME(rs_mh_get_)(SET_TYPE *set, KEY_TYPE key);
extern void KEY_NAME(rs_mh_rehash_)(SET_TYPE *set);
extern uint32_t KEY_NAME(rs_mh_put_)(SET_TYPE *set, KEY_TYPE key, MHPutStatus *new);
extern uint32_t KEY_NAME(rs_mh_delete_)(SET_TYPE *set, KEY_TYPE *key);

uint32_t KEY_NAME(mh_find_bucket_)(SET_TYPE *set, KEY_TYPE key, bool put)
{
  return KEY_NAME(rs_mh_find_bucket_)(set, key, put);
}

uint32_t KEY_NAME(mh_get_)(SET_TYPE *set, KEY_TYPE key)
{
  return KEY_NAME(rs_mh_get_)(set, key);
}

void KEY_NAME(mh_rehash_)(SET_TYPE *set)
{
  KEY_NAME(rs_mh_rehash_)(set);
}

uint32_t KEY_NAME(mh_put_)(SET_TYPE *set, KEY_TYPE key, MHPutStatus *new)
{
  return KEY_NAME(rs_mh_put_)(set, key, new);
}

uint32_t KEY_NAME(mh_delete_)(SET_TYPE *set, KEY_TYPE *key)
{
  return KEY_NAME(rs_mh_delete_)(set, key);
}

#undef SET_TYPE
#undef KEY_TYPE
