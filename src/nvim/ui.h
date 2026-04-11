#pragma once

#include <stdint.h>  // IWYU pragma: keep

#include "nvim/api/private/defs.h"  // IWYU pragma: keep
#include "nvim/event/defs.h"
#include "nvim/grid_defs.h"  // IWYU pragma: keep
#include "nvim/highlight_defs.h"  // IWYU pragma: keep
#include "nvim/macros_defs.h"
#include "nvim/types_defs.h"  // IWYU pragma: keep
#include "nvim/ui_defs.h"  // IWYU pragma: keep

// uncrustify:off
#include "ui.h.generated.h"
#include "ui_events_call.h.generated.h"
EXTERN Array noargs INIT(= ARRAY_DICT_INIT);
// uncrustify:on

// Rust-exported replacements for C thin wrappers (via #[export_name])
#ifndef DLLEXPORT
#  ifdef MSWIN
#    define DLLEXPORT __declspec(dllexport)
#  else
#    define DLLEXPORT
#  endif
#endif
DLLEXPORT bool ui_has(UIExtension ext);
DLLEXPORT int ui_current_row(void);
DLLEXPORT int ui_current_col(void);

// Functions implemented in Rust (nvim-ui crate) and exported via #[export_name]:
bool ui_rgb_attached(void);
bool ui_gui_attached(void);
bool ui_override(void);
size_t ui_active(void);
void ui_refresh(void);
int ui_pum_get_height(void);
bool ui_pum_get_pos(double *pwidth, double *pheight, double *prow, double *pcol);
void ui_default_colors_set(void);
void ui_busy_start(void);
void ui_busy_stop(void);
void vim_beep(unsigned val);
void do_autocmd_uienter_all(void);
void ui_attach_impl(RemoteUI *ui, uint64_t chanid);
void ui_detach_impl(RemoteUI *ui, uint64_t chanid);
void ui_set_ext_option(RemoteUI *ui, UIExtension ext, bool active);
void ui_cursor_goto(int new_row, int new_col);
void ui_grid_cursor_goto(handle_T grid_handle, int new_row, int new_col);
void ui_check_cursor_grid(handle_T grid_handle);
void ui_mode_info_set(void);
void ui_flush(void);
void ui_check_mouse(void);
void ui_cursor_shape_no_check_conceal(void);
void ui_cursor_shape(void);

// vim.ui_attach() namespace of currently executed callback.
EXTERN uint32_t ui_event_ns_id INIT( = 0);
EXTERN MultiQueue *resize_events INIT( = NULL);
EXTERN bool ui_refresh_cmdheight INIT( = true);
