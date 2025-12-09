#include <assert.h>
#include <locale.h>
#include <stdint.h>
#include <uv.h>

#include "nvim/eval/typval.h"
#include "nvim/event/defs.h"
#include "nvim/event/libuv_proc.h"
#include "nvim/event/loop.h"
#include "nvim/event/proc.h"
#include "nvim/log.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/types_defs.h"
#include "nvim/ui_client.h"

#include "event/libuv_proc.c.generated.h"

#ifdef USE_RUST_EVENT
// Rust implementation in nvim-event crate
extern int rs_rstream_is_closed(RStream *stream);
extern int rs_stream_is_closed(Stream *stream);
extern void rs_proc_set_status(Proc *proc, int status);
extern int rs_proc_get_detach(Proc *proc);
extern Loop *rs_proc_get_loop(Proc *proc);
extern void rs_proc_set_pid(Proc *proc, int pid);
extern char **rs_proc_get_argv(Proc *proc);
extern const char *rs_proc_get_cwd(Proc *proc);
extern dict_T *rs_proc_get_env(Proc *proc);
#define rstream_is_closed(s) rs_rstream_is_closed(s)
#define stream_is_closed(s) rs_stream_is_closed(s)
#define proc_set_status(p, s) rs_proc_set_status(p, s)
#define proc_get_detach(p) rs_proc_get_detach(p)
#define proc_get_loop(p) rs_proc_get_loop(p)
#define proc_set_pid(p, pid) rs_proc_set_pid(p, pid)
#define proc_get_argv(p) rs_proc_get_argv(p)
#define proc_get_cwd(p) rs_proc_get_cwd(p)
#define proc_get_env(p) rs_proc_get_env(p)
extern uint8_t rs_proc_get_exit_signal(Proc *proc);
#define proc_get_exit_signal(p) rs_proc_get_exit_signal(p)
extern int rs_proc_get_fwd_err(Proc *proc);
extern int rs_proc_get_overlapped(Proc *proc);
#define proc_get_fwd_err(p) rs_proc_get_fwd_err(p)
#define proc_get_overlapped(p) rs_proc_get_overlapped(p)
extern void rs_proc_call_internal_exit_cb(Proc *proc);
extern void rs_proc_call_internal_close_cb(Proc *proc);
#define proc_call_internal_exit_cb(p) rs_proc_call_internal_exit_cb(p)
#define proc_call_internal_close_cb(p) rs_proc_call_internal_close_cb(p)
#else
#define rstream_is_closed(s) ((s)->s.closed)
#define stream_is_closed(s) ((s)->closed)
#define proc_set_status(p, s) ((p)->status = (s))
#define proc_get_detach(p) ((p)->detach)
#define proc_get_loop(p) ((p)->loop)
#define proc_set_pid(p, pid) ((p)->pid = (pid))
#define proc_get_argv(p) ((p)->argv)
#define proc_get_cwd(p) ((p)->cwd)
#define proc_get_env(p) ((p)->env)
#define proc_get_exit_signal(p) ((p)->exit_signal)
#define proc_get_fwd_err(p) ((p)->fwd_err)
#define proc_get_overlapped(p) ((p)->overlapped)
#define proc_call_internal_exit_cb(p) do { if ((p)->internal_exit_cb) (p)->internal_exit_cb(p); } while (0)
#define proc_call_internal_close_cb(p) do { if ((p)->internal_close_cb) (p)->internal_close_cb(p); } while (0)
#endif

/// @returns zero on success, or negative error code
int libuv_proc_spawn(LibuvProc *uvproc)
  FUNC_ATTR_NONNULL_ALL
{
  Proc *proc = (Proc *)uvproc;
  uvproc->uvopts.file = proc_get_exepath(proc);
  uvproc->uvopts.args = proc_get_argv(proc);
  uvproc->uvopts.flags = UV_PROCESS_WINDOWS_HIDE;
#ifdef MSWIN
  // libuv collapses the argv to a CommandLineToArgvW()-style string. cmd.exe
  // expects a different syntax (must be prepared by the caller before now).
  if (os_shell_is_cmdexe(proc_get_argv(proc)[0])) {
    uvproc->uvopts.flags |= UV_PROCESS_WINDOWS_VERBATIM_ARGUMENTS;
  }
  if (proc_get_detach(proc)) {
    uvproc->uvopts.flags |= UV_PROCESS_DETACHED;
  }
#else
  // Always setsid() on unix-likes. #8107
  uvproc->uvopts.flags |= UV_PROCESS_DETACHED;
#endif
  uvproc->uvopts.exit_cb = exit_cb;
  uvproc->uvopts.cwd = proc_get_cwd(proc);

  uvproc->uvopts.stdio = uvproc->uvstdio;
  uvproc->uvopts.stdio_count = 3;
  uvproc->uvstdio[0].flags = UV_IGNORE;
  uvproc->uvstdio[1].flags = UV_IGNORE;
  uvproc->uvstdio[2].flags = UV_IGNORE;

  if (ui_client_forward_stdin) {
    assert(UI_CLIENT_STDIN_FD == 3);
    uvproc->uvopts.stdio_count = 4;
    uvproc->uvstdio[3].data.fd = 0;
    uvproc->uvstdio[3].flags = UV_INHERIT_FD;
  }
  uvproc->uv.data = proc;

  if (proc_get_env(proc)) {
    uvproc->uvopts.env = tv_dict_to_env(proc_get_env(proc));
  } else {
    uvproc->uvopts.env = NULL;
  }

  if (!stream_is_closed(&proc->in)) {
    uvproc->uvstdio[0].flags = UV_CREATE_PIPE | UV_READABLE_PIPE;
#ifdef MSWIN
    uvproc->uvstdio[0].flags |= proc_get_overlapped(proc) ? UV_OVERLAPPED_PIPE : 0;
#endif
    uvproc->uvstdio[0].data.stream = (uv_stream_t *)(&proc->in.uv.pipe);
  }

  if (!rstream_is_closed(&proc->out)) {
    uvproc->uvstdio[1].flags = UV_CREATE_PIPE | UV_WRITABLE_PIPE;
#ifdef MSWIN
    // pipe must be readable for IOCP to work on Windows.
    uvproc->uvstdio[1].flags |= proc_get_overlapped(proc)
                                ? (UV_READABLE_PIPE | UV_OVERLAPPED_PIPE) : 0;
#endif
    uvproc->uvstdio[1].data.stream = (uv_stream_t *)(&proc->out.s.uv.pipe);
  }

  if (!rstream_is_closed(&proc->err)) {
    uvproc->uvstdio[2].flags = UV_CREATE_PIPE | UV_WRITABLE_PIPE;
    uvproc->uvstdio[2].data.stream = (uv_stream_t *)(&proc->err.s.uv.pipe);
  } else if (proc_get_fwd_err(proc)) {
    uvproc->uvstdio[2].flags = UV_INHERIT_FD;
    uvproc->uvstdio[2].data.fd = STDERR_FILENO;
  }

  int status;
  if ((status = uv_spawn(&proc_get_loop(proc)->uv, &uvproc->uv, &uvproc->uvopts))) {
    ILOG("uv_spawn(%s) failed: %s", uvproc->uvopts.file, uv_strerror(status));
    if (uvproc->uvopts.env) {
      os_free_fullenv(uvproc->uvopts.env);
    }
    return status;
  }

  proc_set_pid(proc, uvproc->uv.pid);
  return status;
}

void libuv_proc_close(LibuvProc *uvproc)
  FUNC_ATTR_NONNULL_ARG(1)
{
  uv_close((uv_handle_t *)&uvproc->uv, close_cb);
}

static void close_cb(uv_handle_t *handle)
{
  Proc *proc = handle->data;
  proc_call_internal_close_cb(proc);
  LibuvProc *uvproc = (LibuvProc *)proc;
  if (uvproc->uvopts.env) {
    os_free_fullenv(uvproc->uvopts.env);
  }
}

static void exit_cb(uv_process_t *handle, int64_t status, int term_signal)
{
  Proc *proc = handle->data;
#if defined(MSWIN)
  // Use stored/expected signal.
  term_signal = proc_get_exit_signal(proc);
#endif
  proc_set_status(proc, term_signal ? 128 + term_signal : (int)status);
  proc_call_internal_exit_cb(proc);
}

LibuvProc libuv_proc_init(Loop *loop, void *data)
{
  LibuvProc rv = {
    .proc = proc_init(loop, kProcTypeUv, data)
  };
  return rv;
}
