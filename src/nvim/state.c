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

void state_enter(VimState *s)
  FUNC_ATTR_NONNULL_ALL
{
  while (true) {
    int check_result = s->check ? s->check(s) : 1;

    if (!check_result) {
      break;     // Terminate this state.
    } else if (check_result == -1) {
      continue;  // check() again.
    }
    // Execute this state.

    int key;

getkey:
    // Apply mappings first by calling vpeekc() directly.
    // - If vpeekc() returns non-NUL, there is a character already available for processing, so
    //   don't block for events. vgetc() may still block, in case of an incomplete UTF-8 sequence.
    // - If vpeekc() returns NUL, vgetc() will block, and there are three cases:
    //   - There is no input available.
    //   - All of available input maps to an empty string.
    //   - There is an incomplete mapping.
    //   A blocking wait for a character should only be done in the third case, which is the only
    //   case of the three where typebuf.tb_len > 0 after vpeekc() returns NUL.
    if (vpeekc() != NUL || typebuf.tb_len > 0) {
      key = safe_vgetc();
    } else if (!multiqueue_empty(loop_get_events(&main_loop))) {
      // No input available and processing events may take time, flush now.
      ui_flush();
      // Event was made available after the last multiqueue_process_events call
      key = K_EVENT;
    } else {
      // Ensure the screen is fully updated before blocking for input. Because of the duality of
      // redraw_later, this can't be done in command-line or when waiting for "Press ENTER".
      // In many of those cases the redraw is expected AFTER the key press, while normally it should
      // update the screen immediately.
      if (must_redraw != 0 && !need_wait_return && (State & MODE_CMDLINE) == 0) {
        update_screen();
        setcursor();  // put cursor back where it belongs
      }
      // Flush screen updates before blocking.
      ui_flush();
      // Call `input_get` directly to block for events or user input without consuming anything from
      // `os/input.c:input_buffer` or calling the mapping engine.
      input_get(NULL, 0, -1, typebuf.tb_change_cnt, loop_get_events(&main_loop));
      // If an event was put into the queue, we send K_EVENT directly.
      if (!input_available() && !multiqueue_empty(loop_get_events(&main_loop))) {
        key = K_EVENT;
      } else {
        goto getkey;
      }
    }

    if (key == K_EVENT) {
      // An event handler may use the value of reg_executing.
      // Clear it if it should be cleared when getting the next character.
      check_end_reg_executing(true);
      may_sync_undo();
    }

#ifdef NVIM_LOG_DEBUG
    char *keyname = key == K_EVENT ? "K_EVENT" : get_special_key_name(key, mod_mask);
    DLOG("input: %s", keyname);
#endif

    int execute_result = s->execute(s, key);
    if (!execute_result) {
      break;
    } else if (execute_result == -1) {
      goto getkey;
    }
  }
}

/// process events on main_loop, but interrupt if input is available
///
/// This should be used to handle K_EVENT in states accepting input
/// otherwise bursts of events can block break checking indefinitely.
void state_handle_k_event(void)
{
  while (true) {
    Event event = multiqueue_get(loop_get_events(&main_loop));
    if (event.handler) {
      event.handler(event.argv);
    }

    if (multiqueue_empty(loop_get_events(&main_loop))) {
      // don't breakcheck before return, caller should return to main-loop
      // and handle input already.
      return;
    }

    // TODO(bfredl): as an further micro-optimization, we could check whether
    // event.handler already checked input.
    os_breakcheck();
    if (input_available() || got_int) {
      return;
    }
  }
}

// get_mode and may_trigger_modechanged migrated to Rust (state crate, Phase 2).

// was_safe, is_safe_now, may_trigger_safestate, state_no_longer_safe
// migrated to Rust (state crate, Phase 1).

