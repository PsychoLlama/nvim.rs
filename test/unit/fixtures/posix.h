#include <errno.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

// `struct termios` and `struct winsize`: tools/ffigen/deny.txt names them, so
// the generated chunk references the tags without defining them.
#include <sys/ioctl.h>
#include <termios.h>

enum {
  kPOSIXErrnoEINTR = EINTR,
  kPOSIXErrnoECHILD = ECHILD,
  kPOSIXWaitWUNTRACED = WUNTRACED,
};
