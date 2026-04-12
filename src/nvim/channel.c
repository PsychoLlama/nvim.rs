#include <assert.h>
#include <fcntl.h>
#include <inttypes.h>
#include <lauxlib.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/private/converter.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/channel.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/encode.h"
#include "nvim/eval/funcs.h"
#include "nvim/eval/typval.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/event/proc.h"
#include "nvim/event/rstream.h"
#include "nvim/event/socket.h"
#include "nvim/event/stream.h"
#include "nvim/event/wstream.h"
#include "nvim/ex_cmds.h"
#include "nvim/garray.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/log.h"
#include "nvim/lua/executor.h"
#include "nvim/main.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
#include "nvim/msgpack_rpc/channel.h"
#include "nvim/msgpack_rpc/server.h"
#include "nvim/os/fs.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/shell.h"
#include "nvim/terminal.h"
#include "nvim/types_defs.h"

#ifdef MSWIN
# include "nvim/os/fs.h"
# include "nvim/os/os_win_console.h"
# include "nvim/os/pty_conpty_win.h"
#endif

static bool did_stdio = false;

/// next free id for a job or rpc channel
/// 1 is reserved for stdio channel
/// 2 is reserved for stderr channel
static uint64_t next_chan_id = CHAN_STDERR + 1;

#include "channel.c.generated.h"

// Rust implementations in channel crate (src/nvim-rs/channel/src/lib.rs)
extern void free_channel_event(void **argv);
extern void close_cb(Stream *stream, void *data);
extern size_t on_channel_data(RStream *stream, const char *buf, size_t count, void *data, bool eof);
extern size_t on_job_stderr(RStream *stream, const char *buf, size_t count, void *data, bool eof);
extern void channel_proc_exit_cb(Proc *proc, int status, void *data);

// Rust implementation in nvim-event crate
extern bool rs_callback_from_typval(Callback *callback, const typval_T *arg);
extern int rs_stream_is_closed(Stream *stream);
extern int rs_proc_get_status(Proc *proc);
extern int rs_proc_get_type(Proc *proc);
extern void rs_proc_set_detach(Proc *proc, int detach);
extern void rs_proc_set_events(Proc *proc, MultiQueue *events);
extern void rs_proc_set_argv(Proc *proc, char **argv);
extern void rs_proc_set_exepath(Proc *proc, const char *exepath);
extern void rs_proc_set_cwd(Proc *proc, const char *cwd);
extern dict_T *rs_proc_get_env(Proc *proc);
extern void rs_proc_set_env(Proc *proc, dict_T *env);
extern int rs_proc_get_fwd_err(Proc *proc);
extern void rs_proc_set_fwd_err(Proc *proc, int fwd_err);
extern int rs_proc_get_overlapped(Proc *proc);
extern void rs_proc_set_overlapped(Proc *proc, int overlapped);
#define stream_is_closed(s) rs_stream_is_closed(s)
#define proc_get_status(p) rs_proc_get_status(p)
#define proc_get_type(p) rs_proc_get_type(p)
#define proc_set_detach(p, d) rs_proc_set_detach(p, d)
#define proc_set_events(p, e) rs_proc_set_events(p, e)
#define proc_set_argv(p, a) rs_proc_set_argv(p, a)
#define proc_set_exepath(p, e) rs_proc_set_exepath(p, e)
#define proc_set_cwd(p, c) rs_proc_set_cwd(p, c)
#define proc_get_env(p) rs_proc_get_env(p)
#define proc_set_env(p, e) rs_proc_set_env(p, e)
#define proc_get_fwd_err(p) rs_proc_get_fwd_err(p)
#define proc_set_fwd_err(p, f) rs_proc_set_fwd_err(p, f)
#define proc_get_overlapped(p) rs_proc_get_overlapped(p)
#define proc_set_overlapped(p, o) rs_proc_set_overlapped(p, o)
extern void rs_proc_set_cb(Proc *proc, void *cb);
#define proc_set_cb(p, c) rs_proc_set_cb(p, (void *)(c))
// Stream accessors for proc->in/out
extern size_t rs_stream_get_pending_reqs(Stream *stream);
#define stream_get_pending_reqs(s) rs_stream_get_pending_reqs(s)
// Loop accessors
extern MultiQueue *rs_loop_get_events(Loop *loop);
#define loop_get_events(l) rs_loop_get_events(l)

// channel_teardown implemented in Rust (src/nvim-rs/channel/src/lib.rs)

#ifdef EXITFREE
void channel_free_all_mem(void)
{
  Channel *chan;
  map_foreach_value(&channels, chan, {
    channel_destroy(chan);
  });
  map_destroy(uint64_t, &channels);

  callback_free(&on_print);
}
#endif

// channel_close implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_init implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_alloc implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// Accessors for next_chan_id (static in this file) used by Rust channel_alloc
uint64_t nvim_chan_get_next_chan_id(void) { return next_chan_id; }
void nvim_chan_set_next_chan_id(uint64_t v) { next_chan_id = v; }
void nvim_chan_map_put(uint64_t id, Channel *chan) { pmap_put(uint64_t)(&channels, id, chan); }
// Accessor for did_stdio (static in this file) used by Rust channel_from_stdio
bool nvim_chan_get_did_stdio(void) { return did_stdio; }
void nvim_chan_set_did_stdio(bool v) { did_stdio = v; }

// channel_create_event implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_incref, channel_decref, callback_reader_free, callback_reader_start
// implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_destroy (static helper), free_channel_event, close_cb
// implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_destroy_early implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_job_start implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_connect implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_from_connection implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_from_stdio implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_send implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// buffer_to_tv_list, on_channel_data, on_job_stderr, on_channel_output (static),
// schedule_channel_event (static), on_channel_event (static), channel_reader_callbacks,
// channel_proc_exit_cb (static), channel_callback_call (static)
// -- all implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_terminal_open, term_write, term_resize, term_delayed_free, term_close
// -- all implemented in Rust (src/nvim-rs/channel/src/lib.rs)
extern void term_delayed_free(void **argv);

// channel_info_changed implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// set_info_event implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_info implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// int64_t_cmp implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_all_info implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// f_prompt_setcallback, f_prompt_setinterrupt, f_prompt_setprompt
// -- all implemented in Rust (src/nvim-rs/channel/src/lib.rs)

