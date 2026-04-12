#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#include "nvim/api/private/defs.h"  // IWYU pragma: keep (for Array)
#include "nvim/channel_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"
#include "nvim/event/defs.h"
#include "nvim/event/libuv_proc.h"
#include "nvim/macros_defs.h"
#include "nvim/map_defs.h"
#include "nvim/memory_defs.h"  // IWYU pragma: keep (for Arena)
#include "nvim/msgpack_rpc/channel_defs.h"
#include "nvim/os/pty_proc.h"
#include "nvim/types_defs.h"

struct Channel {
  uint64_t id;
  size_t refcount;
  MultiQueue *events;

  ChannelStreamType streamtype;
  union {
    Proc proc;
    LibuvProc uv;
    PtyProc pty;
    RStream socket;
    StdioPair stdio;
    StderrState err;
    InternalState internal;
  } stream;

  bool is_rpc;
  bool detach;  ///< Prevents self-exit on channel-close. Normally, Nvim self-exits if its primary
                ///< RPC channel is closed, unless detach=true. Note: currently, detach=false does
                ///< not FORCE self-exit.
  RpcState rpc;
  Terminal *term;

  CallbackReader on_data;
  CallbackReader on_stderr;
  Callback on_exit;
  int exit_status;  ///< Process exit-code (if the channel wraps a process).

  bool callback_busy;
  bool callback_scheduled;
};

#include "channel.h.generated.h"
#include "channel.h.inline.generated.h"

// Implemented in Rust (src/nvim-rs/channel/src/lib.rs)
Channel *channel_alloc(ChannelStreamType type) FUNC_ATTR_NONNULL_RET;
void channel_destroy_early(Channel *chan);
bool channel_close(uint64_t id, ChannelPart part, const char **error);
size_t channel_send(uint64_t id, char *data, size_t len, bool data_owned,
                    const char **error) FUNC_ATTR_NONNULL_ALL;
void channel_teardown(void);
bool channel_job_running(uint64_t id);
void channel_init(void);
void channel_incref(Channel *chan);
void channel_decref(Channel *chan);
void callback_reader_free(CallbackReader *reader);
void callback_reader_start(CallbackReader *reader, const char *type);
void close_cb(Stream *stream, void *data);
void free_channel_event(void **argv);
int int64_t_cmp(const void *pa, const void *pb);
void on_channel_event(void **argv);
void channel_reader_callbacks(Channel *chan, CallbackReader *reader);
void channel_callback_call(Channel *chan, CallbackReader *reader);
size_t on_channel_data(RStream *stream, const char *buf, size_t count, void *data, bool eof);
size_t on_job_stderr(RStream *stream, const char *buf, size_t count, void *data, bool eof);
void channel_proc_exit_cb(Proc *proc, int status, void *data);
void channel_destroy(Channel *chan);
void channel_info_changed(Channel *chan, bool new_chan);
void channel_create_event(Channel *chan, const char *ext_source);
void channel_from_connection(SocketWatcher *watcher);
Array channel_all_info(Arena *arena);
Dict channel_info(uint64_t id, Arena *arena);
void set_info_event(void **argv);
void channel_terminal_open(buf_T *buf, Channel *chan);
void term_delayed_free(void **argv);
void f_prompt_setcallback(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_prompt_setinterrupt(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_prompt_setprompt(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

static inline bool callback_reader_set(CallbackReader reader)
{
  return reader.cb.type != kCallbackNone || reader.self;
}

EXTERN PMap(uint64_t) channels INIT( = MAP_INIT);

EXTERN Callback on_print INIT( = CALLBACK_INIT);

/// @returns Channel with the id or NULL if not found
static inline Channel *find_channel(uint64_t id)
{
  return (Channel *)pmap_get(uint64_t)(&channels, id);
}

static inline Stream *channel_instream(Channel *chan)
  FUNC_ATTR_NONNULL_ALL
{
  switch (chan->streamtype) {
  case kChannelStreamProc:
    return &chan->stream.proc.in;

  case kChannelStreamSocket:
    return &chan->stream.socket.s;

  case kChannelStreamStdio:
    return &chan->stream.stdio.out;

  case kChannelStreamInternal:
  case kChannelStreamStderr:
    abort();
  }
  abort();
}

static inline RStream *channel_outstream(Channel *chan)
  FUNC_ATTR_NONNULL_ALL
{
  switch (chan->streamtype) {
  case kChannelStreamProc:
    return &chan->stream.proc.out;

  case kChannelStreamSocket:
    return &chan->stream.socket;

  case kChannelStreamStdio:
    return &chan->stream.stdio.in;

  case kChannelStreamInternal:
  case kChannelStreamStderr:
    abort();
  }
  abort();
}
