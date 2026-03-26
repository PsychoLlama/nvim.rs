//
// Log module
//
// How Linux printk() handles recursion, buffering, etc:
// https://lwn.net/Articles/780556/
//

#include <assert.h>
#include <errno.h>
#include <inttypes.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <uv.h>

#include "auto/config.h"
#include "nvim/ascii_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/globals.h"
#include "nvim/log.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/os/fs.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/stdpaths_defs.h"
#include "nvim/os/time.h"
#include "nvim/path.h"
#include "nvim/ui_client.h"

/// Cached location of the expanded log file path decided by log_path_init().
static char log_file_path[MAXPATHL + 1] = { 0 };

static bool did_log_init = false;
static uv_mutex_t mutex;

#include "log.c.generated.h"

// C Accessor Functions for Rust FFI

const char *nvim_log_get_file_path(void) { return log_file_path; }

/// Set the log file path.
void nvim_log_set_file_path(const char *path)
{
  if (path == NULL) {
    log_file_path[0] = NUL;
  } else {
    xstrlcpy(log_file_path, path, sizeof(log_file_path));
  }
}

/// Get the size of the log file path buffer.
size_t nvim_log_get_file_path_size(void) { return sizeof(log_file_path); }

/// Check if logging has been initialized.
bool nvim_log_is_initialized(void) { return did_log_init; }


/// Increment the log_skip counter in g_stats.
void nvim_log_increment_skip(void) { g_stats.log_skip++; }

/// Get ui_client_channel_id (non-zero means running as UI client).
uint64_t nvim_log_get_ui_client_channel_id(void) { return ui_client_channel_id; }

/// Get the servername (v:servername).
const char *nvim_log_get_servername(void) { return get_vim_var_str(VV_SEND_SERVER); }

/// Get the parent $NVIM environment variable.
const char *nvim_log_get_parent_nvim(void) { return os_getenv_noalloc(ENV_NVIM); }

/// Get the current PID.
int64_t nvim_log_get_pid(void) { return os_get_pid(); }

/// Get the tail of a path (filename part).
const char *nvim_log_path_tail(const char *path)
{
  if (path == NULL) {
    return "";
  }
  return path_tail(path);
}

/// Get local time structure.
/// Returns 0 on success, -1 on failure.
int nvim_log_get_localtime(struct tm *result)
{
  if (os_localtime(result) == NULL) {
    return -1;
  }
  return 0;
}

/// Get current time in milliseconds (for timestamps).
int nvim_log_get_millis(void)
{
  uv_timeval64_t curtime;
  if (uv_gettimeofday(&curtime) == 0) {
    return (int)(curtime.tv_usec / 1000);
  }
  return 0;
}

/// Expand environment variables in a string.
void nvim_log_expand_env(const char *src, char *dst, int dstlen) { expand_env((char *)src, dst, dstlen); }

/// Check if path is a directory.
bool nvim_log_is_dir(const char *path) { return os_isdir(path); }

/// Get XDG state home path (caller must free).
char *nvim_log_get_xdg_state_home(void) { return get_xdg_home(kXDGStateHome); }

/// Create directory recursively.
/// Returns 0 on success, error code on failure.
int nvim_log_mkdir_recurse(const char *path, char **failed_dir)
{
  return os_mkdir_recurse(path, 0700, failed_dir, NULL);
}

/// Get the user state subpath (e.g., ~/.local/state/nvim/log).
/// Caller must free the result.
char *nvim_log_get_state_subpath(const char *subpath) { return stdpaths_user_state_subpath(subpath, 0, true); }

/// Set environment variable.
void nvim_log_setenv(const char *name, const char *value) { os_setenv(name, value, true); }

/// Free a C string allocated by xmalloc/xstrdup.
void nvim_log_free(void *ptr) { xfree(ptr); }

/// Check if two strings are equal.
bool nvim_log_strequal(const char *s1, const char *s2) { return strequal(s1, s2); }

/// Copy a string with length limit.
size_t nvim_log_strlcpy(char *dst, const char *src, size_t dstsize) { return xstrlcpy(dst, src, dstsize); }


#ifdef HAVE_EXECINFO_BACKTRACE
# include <execinfo.h>
#endif

// Rust FFI declarations

extern void rs_log_path_init(void);
extern bool rs_do_log_to_file(FILE *log_file, int log_level, const char *context,
                              const char *func_name, int line_num, bool eol,
                              const char *message);

static bool log_try_create(char *fname)
{
  if (fname == NULL || fname[0] == NUL) {
    return false;
  }
  FILE *log_file = fopen(fname, "a");
  if (log_file == NULL) {
    return false;
  }
  fclose(log_file);
  return true;
}

void log_init(void)
{
  uv_mutex_init_recursive(&mutex);
  // AFTER init_homedir ("~", XDG) and set_init_1 (env vars). 22b52dd462e5 #11501
  rs_log_path_init();
  did_log_init = true;
}

void log_lock(void) { uv_mutex_lock(&mutex); }

void log_unlock(void) { uv_mutex_unlock(&mutex); }

/// Logs a message to $NVIM_LOG_FILE.
///
/// @param log_level  Log level (see log.h)
/// @param context    Description of a shared context or subsystem
/// @param func_name  Function name, or NULL
/// @param line_num   Source line number, or -1
/// @param eol        Append linefeed "\n"
/// @param fmt        printf-style format string
///
/// @return true if log was emitted normally, false if failed or recursive
bool logmsg(int log_level, const char *context, const char *func_name, int line_num, bool eol,
            const char *fmt, ...)
  FUNC_ATTR_PRINTF(6, 7)
{
  static bool recursive = false;
  static bool did_msg = false;  // Showed recursion message?
  if (!did_log_init) {
    g_stats.log_skip++;
    // set_init_1 may try logging before we are ready. 6f27f5ef91b3 #10183
    return false;
  }

#ifndef NVIM_LOG_DEBUG
  // This should rarely happen (callsites are compiled out), but to be sure.
  // TODO(bfredl): allow log levels to be configured at runtime
  if (log_level < LOGLVL_WRN) {
    return false;
  }
#endif

#ifdef EXITFREE
  // Logging after we've already started freeing all our memory will only cause
  // pain.  We need access to VV_PROGPATH, homedir, etc.
  if (entered_free_all_mem) {
    fprintf(stderr, "FATAL: error in free_all_mem\n %s %s %d: ", context, func_name, line_num);
    va_list args;
    va_start(args, fmt);
    vfprintf(stderr, fmt, args);
    va_end(args);
    if (eol) {
      fprintf(stderr, "\n");
    }
    abort();
  }
#endif

  log_lock();
  if (recursive) {
    if (!did_msg) {
      did_msg = true;
      msg_schedule_semsg("E5430: %s:%d: recursive log!", func_name ? func_name : context, line_num);
    }
    g_stats.log_skip++;
    log_unlock();
    return false;
  }
  recursive = true;
  bool ret = false;
  FILE *log_file = open_log_file();

  // Format the message first, then call Rust for output
  char msgbuf[4096];
  va_list args;
  va_start(args, fmt);
  vsnprintf(msgbuf, sizeof(msgbuf), fmt, args);
  va_end(args);

  ret = rs_do_log_to_file(log_file, log_level, context, func_name, line_num, eol, msgbuf);

  if (log_file != stderr && log_file != stdout) {
    fclose(log_file);
  }

  recursive = false;
  log_unlock();
  return ret;
}

void log_uv_handles(void *loop)
{
  uv_loop_t *l = loop;
  log_lock();
  FILE *log_file = open_log_file();

  uv_print_all_handles(l, log_file);

  if (log_file != stderr && log_file != stdout) {
    fclose(log_file);
  }

  log_unlock();
}

/// Open the log file for appending.
///
/// @return Log file, or stderr on failure
FILE *open_log_file(void)
{
  errno = 0;
  if (log_file_path[0]) {
    FILE *f = fopen(log_file_path, "a");
    if (f != NULL) {
      return f;
    }
  }

  // May happen if:
  //  - fopen() failed
  //  - LOG() is called before log_init()
  //  - Directory does not exist
  //  - File is not writable
  do_log_to_file(stderr, LOGLVL_ERR, NULL, __func__, __LINE__, true,
                 "failed to open $" ENV_LOGFILE " (%s): %s",
                 strerror(errno), log_file_path);
  return stderr;
}

#ifdef HAVE_EXECINFO_BACKTRACE
void log_callstack_to_file(FILE *log_file, const char *const func_name, const int line_num)
{
  void *trace[100];
  int trace_size = backtrace(trace, ARRAY_SIZE(trace));

  char exepath[MAXPATHL] = { 0 };
  size_t exepathlen = MAXPATHL;
  if (os_exepath(exepath, &exepathlen) != 0) {
    abort();
  }
  assert(24 + exepathlen < IOSIZE);  // Must fit in `cmdbuf` below.

  char cmdbuf[IOSIZE + (20 * ARRAY_SIZE(trace)) + MAXPATHL];
  snprintf(cmdbuf, sizeof(cmdbuf), "addr2line -e %s -f -p", exepath);
  for (int i = 1; i < trace_size; i++) {
    char buf[20];  // 64-bit pointer 0xNNNNNNNNNNNNNNNN with leading space.
    snprintf(buf, sizeof(buf), " %p", trace[i]);
    xstrlcat(cmdbuf, buf, sizeof(cmdbuf));
  }
  // Now we have a command string like:
  //    addr2line -e /path/to/exe -f -p 0x123 0x456 ...

  do_log_to_file(log_file, LOGLVL_DBG, NULL, func_name, line_num, true, "trace:");
  FILE *fp = popen(cmdbuf, "r");
  assert(fp);
  char linebuf[IOSIZE];
  while (fgets(linebuf, sizeof(linebuf) - 1, fp) != NULL) {
    fprintf(log_file, "  %s", linebuf);
  }
  pclose(fp);

  if (log_file != stderr && log_file != stdout) {
    fclose(log_file);
  }
}

void log_callstack(const char *const func_name, const int line_num)
{
  log_lock();
  FILE *log_file = open_log_file();
  log_callstack_to_file(log_file, func_name, line_num);
  log_unlock();
}
#endif

static bool do_log_to_file(FILE *log_file, int log_level, const char *context,
                           const char *func_name, int line_num, bool eol, const char *fmt, ...)
  FUNC_ATTR_PRINTF(7, 8)
{
  char msgbuf[4096];
  va_list args;
  va_start(args, fmt);
  vsnprintf(msgbuf, sizeof(msgbuf), fmt, args);
  va_end(args);

  return rs_do_log_to_file(log_file, log_level, context, func_name, line_num, eol, msgbuf);
}
