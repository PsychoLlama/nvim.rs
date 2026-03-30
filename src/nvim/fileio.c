// fileio.c: read from and write to a file

#include <stdbool.h>

#include "nvim/ex_cmds_defs.h"
#include "nvim/fileio.h"
#include "nvim/pos_defs.h"
#include "nvim/types_defs.h"

// Rust implementations - declarations
extern bool rs_need_conversion(const char *fenc);
extern int rs_get_fio_flags(const char *name);
// next_fenc and readfile_charconvert are implemented in Rust (fileio crate).
extern char *next_fenc(char **pp, bool *alloced);
extern char *readfile_charconvert(char *fname, char *fenc, int *fdp);
// readfile() is implemented in Rust.
extern int rs_readfile(char *fname, char *sfname, linenr_T from, linenr_T lines_to_skip,
                       linenr_T lines_to_read, exarg_T *eap, int flags, int silent);

#include "fileio.c.generated.h"

/// Read lines from file "fname" into the buffer after line "from".
///
/// (caller must check that fname != NULL, unless READ_STDIN is used)
///
/// @param eap  can be NULL!
///
/// @return     FAIL for failure, NOTDONE for directory (failure), or OK
int readfile(char *fname, char *sfname, linenr_T from, linenr_T lines_to_skip,
             linenr_T lines_to_read, exarg_T *eap, int flags, bool silent)
{
  return rs_readfile(fname, sfname, from, lines_to_skip, lines_to_read, eap, flags, (int)silent);
}
