/// @file sha256.c
///
/// FIPS-180-2 compliant SHA-256 implementation
/// GPL by Christophe Devine, applies to older version.
/// Modified for md5deep, in public domain.
/// Modified For Vim, Mohsin Ahmed,
/// (original link www.cs.albany.edu/~mosh no longer available)
/// Mohsin Ahmed states this work is distributed under the VIM License or GPL,
/// at your choice.
///
/// Vim specific notes:
/// sha256_self_test() is implicitly called once.

#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/memory.h"
#include "nvim/sha256.h"

#include "sha256.c.generated.h"

// sha256_start, sha256_update, sha256_finish are exported directly from Rust
// (src/nvim-rs/encoding/src/sha256.rs) via #[unsafe(export_name = "...")].

#define SHA_STEP 2

/// Gets the hex digest of the buffer.
///
/// @param buf
/// @param buf_len
/// @param salt
/// @param salt_len
///
/// @returns hex digest of "buf[buf_len]" in a static array.
///          if "salt" is not NULL also do "salt[salt_len]".
extern const char *rs_sha256_bytes(const uint8_t *buf, size_t buf_len, const uint8_t *salt,
                                   size_t salt_len);

const char *sha256_bytes(const uint8_t *restrict buf,  size_t buf_len, const uint8_t *restrict salt,
                         size_t salt_len)
{
  // Validate the Rust implementation once using FIPS test vectors
  sha256_self_test();
  return rs_sha256_bytes(buf, buf_len, salt, salt_len);
}

// These are the standard FIPS-180-2 test vectors
static char *sha_self_test_msg[] = {
  "abc",
  "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
  NULL
};

static char *sha_self_test_vector[] = {
  "ba7816bf8f01cfea414140de5dae2223" \
  "b00361a396177a9cb410ff61f20015ad",
  "248d6a61d20638b8e5c026930c3e6039" \
  "a33ce45964ff2167f6ecedd419db06c1",
  "cdc76e5c9914fb9281a1c7e284d73e67" \
  "f1809a48a497200e046d39ccc7112cd0"
};

/// Perform a test on the SHA256 algorithm.
///
/// @returns true if not failures generated.
bool sha256_self_test(void)
{
  char output[SHA256_BUFFER_SIZE + 1];  // buf size + NULL
  context_sha256_T ctx;
  uint8_t buf[1000];
  uint8_t sha256sum[SHA256_SUM_SIZE];
  const char *hexit;

  static bool sha256_self_tested = false;
  static bool failures = false;

  if (sha256_self_tested) {
    return failures == false;
  }
  sha256_self_tested = true;

  for (size_t i = 0; i < 3; i++) {
    if (i < 2) {
      hexit = sha256_bytes((uint8_t *)sha_self_test_msg[i],
                           strlen(sha_self_test_msg[i]),
                           NULL, 0);
      STRCPY(output, hexit);
    } else {
      sha256_start(&ctx);
      memset(buf, 'a', 1000);

      for (size_t j = 0; j < 1000; j++) {
        sha256_update(&ctx, buf, 1000);
      }
      sha256_finish(&ctx, sha256sum);

      for (size_t j = 0; j < SHA256_SUM_SIZE; j++) {
        snprintf(output + j * SHA_STEP, SHA_STEP + 1, "%02x", sha256sum[j]);
      }
    }

    if (memcmp(output, sha_self_test_vector[i], SHA256_BUFFER_SIZE) != 0) {
      failures = true;
      output[sizeof(output) - 1] = NUL;

      // printf("sha256_self_test %d failed %s\n", i, output);
    }
  }
  return failures == false;
}
