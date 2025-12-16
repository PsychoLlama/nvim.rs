/// @file hashtab.c
///
/// Handling of a hashtable with Vim-specific properties.
///
/// Each item in a hashtable has a NUL terminated string key. A key can appear
/// only once in the table.
///
/// A hash number is computed from the key for quick lookup. When the hashes
/// of two different keys point to the same entry an algorithm is used to
/// iterate over other entries in the table until the right one is found.
/// To make the iteration work removed keys are different from entries where a
/// key was never present.
///
/// The mechanism has been partly based on how Python Dictionaries are
/// implemented. The algorithm is from Knuth Vol. 3, Sec. 6.4.
///
/// The hashtable grows to accommodate more entries when needed. At least 1/3
/// of the entries is empty to keep the lookup efficient (at the cost of extra
/// memory).

#include <stddef.h>
#include <string.h>

#include "nvim/gettext_defs.h"
#include "nvim/hashtab.h"
#include "nvim/message.h"
#include "nvim/vim_defs.h"

extern hash_T rs_hash_hash(const char *key);
extern hash_T rs_hash_hash_len(const char *key, size_t len);

// Rust hashtab implementations
extern void rs_hash_init(hashtab_T *ht);
extern void rs_hash_clear(hashtab_T *ht);
extern void rs_hash_clear_all(hashtab_T *ht, unsigned off);
extern hashitem_T *rs_hash_lookup(const hashtab_T *ht, const char *key, size_t key_len, hash_T hash);
extern hashitem_T *rs_hash_find(const hashtab_T *ht, const char *key);
extern hashitem_T *rs_hash_find_len(const hashtab_T *ht, const char *key, size_t len);
extern void rs_hash_add_item(hashtab_T *ht, hashitem_T *hi, char *key, hash_T hash);
extern void rs_hash_remove(hashtab_T *ht, hashitem_T *hi);
extern void rs_hash_lock(hashtab_T *ht);
extern void rs_hash_unlock(hashtab_T *ht);
extern const char *rs_hash_key_removed(void);

#include "hashtab.c.generated.h"

// Global marker for removed items - used by both C and Rust
char hash_removed;

/// Initialize an empty hash table.
void hash_init(hashtab_T *ht)
{
  rs_hash_init(ht);
}

/// Free the array of a hash table without freeing contained values.
///
/// If "ht" is not freed (after calling this) then you should call hash_init()
/// right next!
void hash_clear(hashtab_T *ht)
{
  rs_hash_clear(ht);
}

/// Free the array of a hash table and all contained values.
///
/// @param off the offset from start of value to start of key (@see hashitem_T).
void hash_clear_all(hashtab_T *ht, unsigned off)
{
  rs_hash_clear_all(ht, off);
}

/// Find item for given "key" in hashtable "ht".
///
/// @param key The key of the looked-for item. Must not be NULL.
///
/// @return Pointer to the hash item corresponding to the given key.
///         If not found, then return pointer to the empty item that would be
///         used for that key.
///         WARNING: Returned pointer becomes invalid as soon as the hash table
///                  is changed in any way.
hashitem_T *hash_find(const hashtab_T *const ht, const char *const key)
{
  return rs_hash_find(ht, key);
}

/// Like hash_find, but key is not NUL-terminated
///
/// @param[in]  ht  Hashtab to look in.
/// @param[in]  key  Key of the looked-for item. Must not be NULL.
/// @param[in]  len  Key length.
///
/// @return Pointer to the hash item corresponding to the given key.
///         If not found, then return pointer to the empty item that would be
///         used for that key.
///
///         @warning Returned pointer becomes invalid as soon as the hash table
///                  is changed in any way.
hashitem_T *hash_find_len(const hashtab_T *const ht, const char *const key, const size_t len)
{
  return rs_hash_find_len(ht, key, len);
}

/// Like hash_find(), but caller computes "hash".
///
/// @param[in]  key  The key of the looked-for item. Must not be NULL.
/// @param[in]  key_len  Key length.
/// @param[in]  hash  The precomputed hash for the key.
///
/// @return Pointer to the hashitem corresponding to the given key.
///         If not found, then return pointer to the empty item that would be
///         used for that key.
///         WARNING: Returned pointer becomes invalid as soon as the hash table
///                  is changed in any way.
hashitem_T *hash_lookup(const hashtab_T *const ht, const char *const key, const size_t key_len,
                        const hash_T hash)
{
  return rs_hash_lookup(ht, key, key_len, hash);
}

/// Print the efficiency of hashtable lookups.
///
/// Useful when trying different hash algorithms.
/// Called when exiting.
void hash_debug_results(void)
{
#ifdef HT_DEBUG
  fprintf(stderr, "\r\n\r\n\r\n\r\n");
  fprintf(stderr, "Number of hashtable lookups: %" PRId64 "\r\n",
          (int64_t)hash_count_lookup);
  fprintf(stderr, "Number of perturb loops: %" PRId64 "\r\n",
          (int64_t)hash_count_perturb);
  fprintf(stderr, "Percentage of perturb loops: %" PRId64 "%%\r\n",
          (int64_t)(hash_count_perturb * 100 / hash_count_lookup));
#endif
}

/// Add (empty) item for key `key` to hashtable `ht`.
///
/// @param key Pointer to the key for the new item. The key has to be contained
///            in the new item (@see hashitem_T). Must not be NULL.
///
/// @return OK   if success.
///         FAIL if key already present
int hash_add(hashtab_T *ht, char *key)
{
  hash_T hash = hash_hash(key);
  hashitem_T *hi = hash_lookup(ht, key, strlen(key), hash);
  if (!HASHITEM_EMPTY(hi)) {
    siemsg(_("E685: Internal error: hash_add(): duplicate key \"%s\""), key);
    return FAIL;
  }
  hash_add_item(ht, hi, key, hash);
  return OK;
}

/// Add item "hi" for key "key" to hashtable "ht".
///
/// @param hi   The hash item to be used. Must have been obtained through
///             hash_lookup() and point to an empty item.
/// @param key  Pointer to the key for the new item. The key has to be contained
///             in the new item (@see hashitem_T). Must not be NULL.
/// @param hash The precomputed hash value for the key.
void hash_add_item(hashtab_T *ht, hashitem_T *hi, char *key, hash_T hash)
{
  rs_hash_add_item(ht, hi, key, hash);
}

/// Remove item "hi" from hashtable "ht".
///
/// Caller must take care of freeing the item itself.
///
/// @param hi The hash item to be removed.
///           It must have been obtained with hash_lookup().
void hash_remove(hashtab_T *ht, hashitem_T *hi)
{
  rs_hash_remove(ht, hi);
}

/// Lock hashtable (prevent changes in ht_array).
///
/// Don't use this when items are to be added!
/// Must call hash_unlock() later.
void hash_lock(hashtab_T *ht)
{
  rs_hash_lock(ht);
}

/// Unlock hashtable (allow changes in ht_array again).
///
/// Table will be resized (shrunk) when necessary.
/// This must balance a call to hash_lock().
void hash_unlock(hashtab_T *ht)
{
  rs_hash_unlock(ht);
}

#define HASH_CYCLE_BODY(hash, p) \
  hash = hash * 101 + *p++

/// Get the hash number for a key.
///
/// If you think you know a better hash function: Compile with HT_DEBUG set and
/// run a script that uses hashtables a lot. Vim will then print statistics
/// when exiting. Try that with the current hash algorithm and yours. The
/// lower the percentage the better.
hash_T hash_hash(const char *key)
{
  return rs_hash_hash(key);
}

/// Get the hash number for a key that is not a NUL-terminated string
///
/// @warning Function does not check whether key contains NUL. But you will not
///          be able to get hash entry in this case.
///
/// @param[in]  key  Key.
/// @param[in]  len  Key length.
///
/// @return Key hash.
hash_T hash_hash_len(const char *key, const size_t len)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_hash_hash_len(key, len);
}

#undef HASH_CYCLE_BODY

/// Function to get HI_KEY_REMOVED value
///
/// Used for testing because luajit ffi does not allow getting addresses of
/// globals.
const char *_hash_key_removed(void)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_hash_key_removed();
}
