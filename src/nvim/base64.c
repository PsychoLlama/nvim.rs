#include <stddef.h>
#include <stdint.h>

#include "nvim/base64.h"

#include "base64.c.generated.h"

extern char *rs_base64_encode(const uint8_t *src, size_t src_len);
extern char *rs_base64_decode(const uint8_t *src, size_t src_len, size_t *out_len);

/// Encode a string using Base64.
///
/// @param src String to encode
/// @param src_len Length of the string
/// @return Base64 encoded string
char *base64_encode(const char *src, size_t src_len)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_base64_encode((const uint8_t *)src, src_len);
}

/// Decode a Base64 encoded string.
///
/// The returned string is NOT null-terminated, because the decoded string may
/// contain embedded NULLs. Use the output parameter out_lenp to determine the
/// length of the returned string.
///
/// @param src Base64 encoded string
/// @param src_len Length of {src}
/// @param [out] out_lenp Returns the length of the decoded string
/// @return Decoded string
char *base64_decode(const char *src, size_t src_len, size_t *out_lenp)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_base64_decode((const uint8_t *)src, src_len, out_lenp);
}
