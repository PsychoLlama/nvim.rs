#include <stdbool.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/drawscreen.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/event/defs.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/ex_getln.h"
#include "nvim/getchar.h"
#include "nvim/globals.h"
#include "nvim/insexpand.h"
#include "nvim/keycodes.h"
#include "nvim/log.h"
#include "nvim/macros_defs.h"
#include "nvim/main.h"
#include "nvim/memory.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/state.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"

#include "state.c.generated.h"

// Rust implementation in nvim-event crate
extern int rs_multiqueue_empty(MultiQueue *mq);
extern MultiQueue *rs_loop_get_events(Loop *loop);
#define multiqueue_empty(mq) rs_multiqueue_empty(mq)
#define loop_get_events(l) rs_loop_get_events(l)

// Rust implementations
extern int rs_ctrl_x_mode_not_defined_yet(void);
extern int rs_ins_compl_active(void);

// state_enter and state_handle_k_event migrated to Rust (state crate, Phase 3).

// get_mode and may_trigger_modechanged migrated to Rust (state crate, Phase 2).

// was_safe, is_safe_now, may_trigger_safestate, state_no_longer_safe
// migrated to Rust (state crate, Phase 1).

