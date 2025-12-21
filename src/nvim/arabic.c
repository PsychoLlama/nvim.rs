/// @file arabic.c
///
/// Rust bridge for Arabic language functions.
/// All implementations are in src/nvim-rs/arabic/.

#include <stdbool.h>

#include "nvim/arabic.h"

#include "arabic.c.generated.h"

// Rust FFI declarations
extern bool rs_arabic_combine(int one, int two);
extern bool rs_arabic_maycombine(int two);
extern int rs_arabic_shape(int c, int *c1p, int prev_c, int prev_c1, int next_c);

/// Check whether we are dealing with a character that could be regarded as an
/// Arabic combining character, need to check the character before this.
bool arabic_maycombine(int two)
  FUNC_ATTR_PURE
{
  return rs_arabic_maycombine(two);
}

/// Check whether we are dealing with Arabic combining characters.
/// Returns false for negative values.
/// Note: these are NOT really composing characters!
///
/// @param one First character.
/// @param two Character just after "one".
bool arabic_combine(int one, int two)
  FUNC_ATTR_PURE
{
  return rs_arabic_combine(one, two);
}

/// Do Arabic shaping on character "c".  Returns the shaped character.
/// @param c       The character to shape
/// @param c1p     Pointer to the first composing char for "c" (in/out)
/// @param prev_c  The previous character (not shaped)
/// @param prev_c1 The first composing char for the previous char (not shaped)
/// @param next_c  The next character (not shaped)
int arabic_shape(int c, int *c1p, int prev_c, int prev_c1, int next_c)
{
  return rs_arabic_shape(c, c1p, prev_c, prev_c1, next_c);
}
