use crate::src::nvim::global_cell::GlobalCell;
extern "C" {
    pub type unibi_term;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn unibi_destroy(_: *mut unibi_term);
    fn unibi_get_bool(_: *const unibi_term, _: unibi_boolean) -> ::core::ffi::c_int;
    fn unibi_get_num(_: *const unibi_term, _: unibi_numeric) -> ::core::ffi::c_int;
    fn unibi_get_str(_: *const unibi_term, _: unibi_string) -> *const ::core::ffi::c_char;
    fn unibi_from_term(_: *const ::core::ffi::c_char) -> *mut unibi_term;
    fn unibi_count_ext_bool(_: *const unibi_term) -> size_t;
    fn unibi_count_ext_str(_: *const unibi_term) -> size_t;
    fn unibi_get_ext_str(_: *const unibi_term, _: size_t) -> *const ::core::ffi::c_char;
    fn unibi_get_ext_bool_name(_: *const unibi_term, _: size_t) -> *const ::core::ffi::c_char;
    fn unibi_get_ext_str_name(_: *const unibi_term, _: size_t) -> *const ::core::ffi::c_char;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn arena_strdup(arena: *mut Arena, str: *const ::core::ffi::c_char)
        -> *mut ::core::ffi::c_char;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn kv_do_printf(
        str: *mut StringBuilder,
        fmt_0: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn kv_transstr(str: *mut StringBuilder, s: *const ::core::ffi::c_char, untab: bool) -> size_t;
}
pub type size_t = usize;
pub type ssize_t = isize;
pub type unibi_boolean = ::core::ffi::c_uint;
pub const unibi_boolean_end_: unibi_boolean = 45;
pub const unibi_return_does_clr_eol: unibi_boolean = 44;
pub const unibi_has_hardware_tabs: unibi_boolean = 43;
pub const unibi_linefeed_is_newline: unibi_boolean = 42;
pub const unibi_gnu_has_meta_key: unibi_boolean = 41;
pub const unibi_no_correctly_working_cr: unibi_boolean = 40;
pub const unibi_crt_no_scrolling: unibi_boolean = 39;
pub const unibi_backspaces_with_bs: unibi_boolean = 38;
pub const unibi_lpi_changes_res: unibi_boolean = 37;
pub const unibi_cpi_changes_res: unibi_boolean = 36;
pub const unibi_semi_auto_right_margin: unibi_boolean = 35;
pub const unibi_row_addr_glitch: unibi_boolean = 34;
pub const unibi_has_print_wheel: unibi_boolean = 33;
pub const unibi_cr_cancels_micro_mode: unibi_boolean = 32;
pub const unibi_col_addr_glitch: unibi_boolean = 31;
pub const unibi_hue_lightness_saturation: unibi_boolean = 30;
pub const unibi_back_color_erase: unibi_boolean = 29;
pub const unibi_can_change: unibi_boolean = 28;
pub const unibi_non_dest_scroll_region: unibi_boolean = 27;
pub const unibi_no_pad_char: unibi_boolean = 26;
pub const unibi_non_rev_rmcup: unibi_boolean = 25;
pub const unibi_hard_cursor: unibi_boolean = 24;
pub const unibi_prtr_silent: unibi_boolean = 23;
pub const unibi_needs_xon_xoff: unibi_boolean = 22;
pub const unibi_xon_xoff: unibi_boolean = 21;
pub const unibi_transparent_underline: unibi_boolean = 20;
pub const unibi_tilde_glitch: unibi_boolean = 19;
pub const unibi_dest_tabs_magic_smso: unibi_boolean = 18;
pub const unibi_status_line_esc_ok: unibi_boolean = 17;
pub const unibi_over_strike: unibi_boolean = 16;
pub const unibi_move_standout_mode: unibi_boolean = 15;
pub const unibi_move_insert_mode: unibi_boolean = 14;
pub const unibi_memory_below: unibi_boolean = 13;
pub const unibi_memory_above: unibi_boolean = 12;
pub const unibi_insert_null_glitch: unibi_boolean = 11;
pub const unibi_has_status_line: unibi_boolean = 10;
pub const unibi_has_meta_key: unibi_boolean = 9;
pub const unibi_hard_copy: unibi_boolean = 8;
pub const unibi_generic_type: unibi_boolean = 7;
pub const unibi_erase_overstrike: unibi_boolean = 6;
pub const unibi_eat_newline_glitch: unibi_boolean = 5;
pub const unibi_ceol_standout_glitch: unibi_boolean = 4;
pub const unibi_no_esc_ctlc: unibi_boolean = 3;
pub const unibi_auto_right_margin: unibi_boolean = 2;
pub const unibi_auto_left_margin: unibi_boolean = 1;
pub const unibi_boolean_begin_: unibi_boolean = 0;
pub type unibi_numeric = ::core::ffi::c_uint;
pub const unibi_numeric_end_: unibi_numeric = 85;
pub const unibi_number_of_function_keys: unibi_numeric = 84;
pub const unibi_horizontal_tab_delay: unibi_numeric = 83;
pub const unibi_backspace_delay: unibi_numeric = 82;
pub const unibi_new_line_delay: unibi_numeric = 81;
pub const unibi_carriage_return_delay: unibi_numeric = 80;
pub const unibi_magic_cookie_glitch_ul: unibi_numeric = 79;
pub const unibi_bit_image_type: unibi_numeric = 78;
pub const unibi_bit_image_entwining: unibi_numeric = 77;
pub const unibi_buttons: unibi_numeric = 76;
pub const unibi_wide_char_size: unibi_numeric = 75;
pub const unibi_print_rate: unibi_numeric = 74;
pub const unibi_output_res_vert_inch: unibi_numeric = 73;
pub const unibi_output_res_horz_inch: unibi_numeric = 72;
pub const unibi_output_res_line: unibi_numeric = 71;
pub const unibi_output_res_char: unibi_numeric = 70;
pub const unibi_number_of_pins: unibi_numeric = 69;
pub const unibi_micro_line_size: unibi_numeric = 68;
pub const unibi_micro_col_size: unibi_numeric = 67;
pub const unibi_max_micro_jump: unibi_numeric = 66;
pub const unibi_max_micro_address: unibi_numeric = 65;
pub const unibi_dot_horz_spacing: unibi_numeric = 64;
pub const unibi_dot_vert_spacing: unibi_numeric = 63;
pub const unibi_buffer_capacity: unibi_numeric = 62;
pub const unibi_no_color_video: unibi_numeric = 61;
pub const unibi_max_pairs: unibi_numeric = 60;
pub const unibi_max_colors: unibi_numeric = 59;
pub const unibi_maximum_windows: unibi_numeric = 58;
pub const unibi_max_attributes: unibi_numeric = 57;
pub const unibi_label_width: unibi_numeric = 56;
pub const unibi_label_height: unibi_numeric = 55;
pub const unibi_num_labels: unibi_numeric = 54;
pub const unibi_width_status_line: unibi_numeric = 53;
pub const unibi_virtual_terminal: unibi_numeric = 52;
pub const unibi_padding_baud_rate: unibi_numeric = 51;
pub const unibi_magic_cookie_glitch: unibi_numeric = 50;
pub const unibi_lines_of_memory: unibi_numeric = 49;
pub const unibi_lines: unibi_numeric = 48;
pub const unibi_init_tabs: unibi_numeric = 47;
pub const unibi_columns: unibi_numeric = 46;
pub const unibi_numeric_begin_: unibi_numeric = 45;
pub type unibi_string = ::core::ffi::c_uint;
pub const unibi_string_end_: unibi_string = 500;
pub const unibi_box_chars_1: unibi_string = 499;
pub const unibi_memory_unlock: unibi_string = 498;
pub const unibi_memory_lock: unibi_string = 497;
pub const unibi_acs_plus: unibi_string = 496;
pub const unibi_acs_vline: unibi_string = 495;
pub const unibi_acs_hline: unibi_string = 494;
pub const unibi_acs_ttee: unibi_string = 493;
pub const unibi_acs_btee: unibi_string = 492;
pub const unibi_acs_rtee: unibi_string = 491;
pub const unibi_acs_ltee: unibi_string = 490;
pub const unibi_acs_lrcorner: unibi_string = 489;
pub const unibi_acs_urcorner: unibi_string = 488;
pub const unibi_acs_llcorner: unibi_string = 487;
pub const unibi_acs_ulcorner: unibi_string = 486;
pub const unibi_arrow_key_map: unibi_string = 485;
pub const unibi_other_non_function_keys: unibi_string = 484;
pub const unibi_backspace_if_not_bs: unibi_string = 483;
pub const unibi_linefeed_if_not_lf: unibi_string = 482;
pub const unibi_termcap_reset: unibi_string = 481;
pub const unibi_termcap_init2: unibi_string = 480;
pub const unibi_set_pglen_inch: unibi_string = 479;
pub const unibi_set_a_attributes: unibi_string = 478;
pub const unibi_enter_vertical_hl_mode: unibi_string = 477;
pub const unibi_enter_top_hl_mode: unibi_string = 476;
pub const unibi_enter_right_hl_mode: unibi_string = 475;
pub const unibi_enter_low_hl_mode: unibi_string = 474;
pub const unibi_enter_left_hl_mode: unibi_string = 473;
pub const unibi_enter_horizontal_hl_mode: unibi_string = 472;
pub const unibi_alt_scancode_esc: unibi_string = 471;
pub const unibi_scancode_escape: unibi_string = 470;
pub const unibi_pc_term_options: unibi_string = 469;
pub const unibi_exit_scancode_mode: unibi_string = 468;
pub const unibi_enter_scancode_mode: unibi_string = 467;
pub const unibi_exit_pc_charset_mode: unibi_string = 466;
pub const unibi_enter_pc_charset_mode: unibi_string = 465;
pub const unibi_display_pc_char: unibi_string = 464;
pub const unibi_set_page_length: unibi_string = 463;
pub const unibi_set_color_band: unibi_string = 462;
pub const unibi_end_bit_image_region: unibi_string = 461;
pub const unibi_define_bit_image_region: unibi_string = 460;
pub const unibi_color_names: unibi_string = 459;
pub const unibi_bit_image_carriage_return: unibi_string = 458;
pub const unibi_bit_image_newline: unibi_string = 457;
pub const unibi_bit_image_repeat: unibi_string = 456;
pub const unibi_set_tb_margin: unibi_string = 455;
pub const unibi_set_lr_margin: unibi_string = 454;
pub const unibi_set3_des_seq: unibi_string = 453;
pub const unibi_set2_des_seq: unibi_string = 452;
pub const unibi_set1_des_seq: unibi_string = 451;
pub const unibi_set0_des_seq: unibi_string = 450;
pub const unibi_code_set_init: unibi_string = 449;
pub const unibi_device_type: unibi_string = 448;
pub const unibi_pkey_plab: unibi_string = 447;
pub const unibi_set_a_background: unibi_string = 446;
pub const unibi_set_a_foreground: unibi_string = 445;
pub const unibi_get_mouse: unibi_string = 444;
pub const unibi_req_mouse_pos: unibi_string = 443;
pub const unibi_mouse_info: unibi_string = 442;
pub const unibi_key_mouse: unibi_string = 441;
pub const unibi_char_set_names: unibi_string = 440;
pub const unibi_zero_motion: unibi_string = 439;
pub const unibi_these_cause_cr: unibi_string = 438;
pub const unibi_superscript_characters: unibi_string = 437;
pub const unibi_subscript_characters: unibi_string = 436;
pub const unibi_stop_char_set_def: unibi_string = 435;
pub const unibi_stop_bit_image: unibi_string = 434;
pub const unibi_start_char_set_def: unibi_string = 433;
pub const unibi_start_bit_image: unibi_string = 432;
pub const unibi_set_top_margin_parm: unibi_string = 431;
pub const unibi_set_top_margin: unibi_string = 430;
pub const unibi_set_right_margin_parm: unibi_string = 429;
pub const unibi_set_left_margin_parm: unibi_string = 428;
pub const unibi_set_bottom_margin_parm: unibi_string = 427;
pub const unibi_set_bottom_margin: unibi_string = 426;
pub const unibi_select_char_set: unibi_string = 425;
pub const unibi_parm_up_micro: unibi_string = 424;
pub const unibi_parm_right_micro: unibi_string = 423;
pub const unibi_parm_left_micro: unibi_string = 422;
pub const unibi_parm_down_micro: unibi_string = 421;
pub const unibi_order_of_pins: unibi_string = 420;
pub const unibi_micro_up: unibi_string = 419;
pub const unibi_micro_row_address: unibi_string = 418;
pub const unibi_micro_right: unibi_string = 417;
pub const unibi_micro_left: unibi_string = 416;
pub const unibi_micro_down: unibi_string = 415;
pub const unibi_micro_column_address: unibi_string = 414;
pub const unibi_exit_upward_mode: unibi_string = 413;
pub const unibi_exit_superscript_mode: unibi_string = 412;
pub const unibi_exit_subscript_mode: unibi_string = 411;
pub const unibi_exit_shadow_mode: unibi_string = 410;
pub const unibi_exit_micro_mode: unibi_string = 409;
pub const unibi_exit_leftward_mode: unibi_string = 408;
pub const unibi_exit_italics_mode: unibi_string = 407;
pub const unibi_exit_doublewide_mode: unibi_string = 406;
pub const unibi_enter_upward_mode: unibi_string = 405;
pub const unibi_enter_superscript_mode: unibi_string = 404;
pub const unibi_enter_subscript_mode: unibi_string = 403;
pub const unibi_enter_shadow_mode: unibi_string = 402;
pub const unibi_enter_normal_quality: unibi_string = 401;
pub const unibi_enter_near_letter_quality: unibi_string = 400;
pub const unibi_enter_micro_mode: unibi_string = 399;
pub const unibi_enter_leftward_mode: unibi_string = 398;
pub const unibi_enter_italics_mode: unibi_string = 397;
pub const unibi_enter_draft_quality: unibi_string = 396;
pub const unibi_enter_doublewide_mode: unibi_string = 395;
pub const unibi_define_char: unibi_string = 394;
pub const unibi_change_res_vert: unibi_string = 393;
pub const unibi_change_res_horz: unibi_string = 392;
pub const unibi_change_line_pitch: unibi_string = 391;
pub const unibi_change_char_pitch: unibi_string = 390;
pub const unibi_set_background: unibi_string = 389;
pub const unibi_set_foreground: unibi_string = 388;
pub const unibi_set_color_pair: unibi_string = 387;
pub const unibi_initialize_pair: unibi_string = 386;
pub const unibi_initialize_color: unibi_string = 385;
pub const unibi_orig_colors: unibi_string = 384;
pub const unibi_orig_pair: unibi_string = 383;
pub const unibi_user9: unibi_string = 382;
pub const unibi_user8: unibi_string = 381;
pub const unibi_user7: unibi_string = 380;
pub const unibi_user6: unibi_string = 379;
pub const unibi_user5: unibi_string = 378;
pub const unibi_user4: unibi_string = 377;
pub const unibi_user3: unibi_string = 376;
pub const unibi_user2: unibi_string = 375;
pub const unibi_user1: unibi_string = 374;
pub const unibi_user0: unibi_string = 373;
pub const unibi_wait_tone: unibi_string = 372;
pub const unibi_fixed_pause: unibi_string = 371;
pub const unibi_flash_hook: unibi_string = 370;
pub const unibi_pulse: unibi_string = 369;
pub const unibi_tone: unibi_string = 368;
pub const unibi_quick_dial: unibi_string = 367;
pub const unibi_dial_phone: unibi_string = 366;
pub const unibi_hangup: unibi_string = 365;
pub const unibi_goto_window: unibi_string = 364;
pub const unibi_create_window: unibi_string = 363;
pub const unibi_remove_clock: unibi_string = 362;
pub const unibi_display_clock: unibi_string = 361;
pub const unibi_set_clock: unibi_string = 360;
pub const unibi_label_format: unibi_string = 359;
pub const unibi_set_right_margin: unibi_string = 358;
pub const unibi_set_left_margin: unibi_string = 357;
pub const unibi_clear_margins: unibi_string = 356;
pub const unibi_clr_bol: unibi_string = 355;
pub const unibi_key_f63: unibi_string = 354;
pub const unibi_key_f62: unibi_string = 353;
pub const unibi_key_f61: unibi_string = 352;
pub const unibi_key_f60: unibi_string = 351;
pub const unibi_key_f59: unibi_string = 350;
pub const unibi_key_f58: unibi_string = 349;
pub const unibi_key_f57: unibi_string = 348;
pub const unibi_key_f56: unibi_string = 347;
pub const unibi_key_f55: unibi_string = 346;
pub const unibi_key_f54: unibi_string = 345;
pub const unibi_key_f53: unibi_string = 344;
pub const unibi_key_f52: unibi_string = 343;
pub const unibi_key_f51: unibi_string = 342;
pub const unibi_key_f50: unibi_string = 341;
pub const unibi_key_f49: unibi_string = 340;
pub const unibi_key_f48: unibi_string = 339;
pub const unibi_key_f47: unibi_string = 338;
pub const unibi_key_f46: unibi_string = 337;
pub const unibi_key_f45: unibi_string = 336;
pub const unibi_key_f44: unibi_string = 335;
pub const unibi_key_f43: unibi_string = 334;
pub const unibi_key_f42: unibi_string = 333;
pub const unibi_key_f41: unibi_string = 332;
pub const unibi_key_f40: unibi_string = 331;
pub const unibi_key_f39: unibi_string = 330;
pub const unibi_key_f38: unibi_string = 329;
pub const unibi_key_f37: unibi_string = 328;
pub const unibi_key_f36: unibi_string = 327;
pub const unibi_key_f35: unibi_string = 326;
pub const unibi_key_f34: unibi_string = 325;
pub const unibi_key_f33: unibi_string = 324;
pub const unibi_key_f32: unibi_string = 323;
pub const unibi_key_f31: unibi_string = 322;
pub const unibi_key_f30: unibi_string = 321;
pub const unibi_key_f29: unibi_string = 320;
pub const unibi_key_f28: unibi_string = 319;
pub const unibi_key_f27: unibi_string = 318;
pub const unibi_key_f26: unibi_string = 317;
pub const unibi_key_f25: unibi_string = 316;
pub const unibi_key_f24: unibi_string = 315;
pub const unibi_key_f23: unibi_string = 314;
pub const unibi_key_f22: unibi_string = 313;
pub const unibi_key_f21: unibi_string = 312;
pub const unibi_key_f20: unibi_string = 311;
pub const unibi_key_f19: unibi_string = 310;
pub const unibi_key_f18: unibi_string = 309;
pub const unibi_key_f17: unibi_string = 308;
pub const unibi_key_f16: unibi_string = 307;
pub const unibi_key_f15: unibi_string = 306;
pub const unibi_key_f14: unibi_string = 305;
pub const unibi_key_f13: unibi_string = 304;
pub const unibi_key_f12: unibi_string = 303;
pub const unibi_key_f11: unibi_string = 302;
pub const unibi_req_for_input: unibi_string = 301;
pub const unibi_key_sundo: unibi_string = 300;
pub const unibi_key_ssuspend: unibi_string = 299;
pub const unibi_key_ssave: unibi_string = 298;
pub const unibi_key_srsume: unibi_string = 297;
pub const unibi_key_sright: unibi_string = 296;
pub const unibi_key_sreplace: unibi_string = 295;
pub const unibi_key_sredo: unibi_string = 294;
pub const unibi_key_sprint: unibi_string = 293;
pub const unibi_key_sprevious: unibi_string = 292;
pub const unibi_key_soptions: unibi_string = 291;
pub const unibi_key_snext: unibi_string = 290;
pub const unibi_key_smove: unibi_string = 289;
pub const unibi_key_smessage: unibi_string = 288;
pub const unibi_key_sleft: unibi_string = 287;
pub const unibi_key_sic: unibi_string = 286;
pub const unibi_key_shome: unibi_string = 285;
pub const unibi_key_shelp: unibi_string = 284;
pub const unibi_key_sfind: unibi_string = 283;
pub const unibi_key_sexit: unibi_string = 282;
pub const unibi_key_seol: unibi_string = 281;
pub const unibi_key_send: unibi_string = 280;
pub const unibi_key_select: unibi_string = 279;
pub const unibi_key_sdl: unibi_string = 278;
pub const unibi_key_sdc: unibi_string = 277;
pub const unibi_key_screate: unibi_string = 276;
pub const unibi_key_scopy: unibi_string = 275;
pub const unibi_key_scommand: unibi_string = 274;
pub const unibi_key_scancel: unibi_string = 273;
pub const unibi_key_sbeg: unibi_string = 272;
pub const unibi_key_undo: unibi_string = 271;
pub const unibi_key_suspend: unibi_string = 270;
pub const unibi_key_save: unibi_string = 269;
pub const unibi_key_resume: unibi_string = 268;
pub const unibi_key_restart: unibi_string = 267;
pub const unibi_key_replace: unibi_string = 266;
pub const unibi_key_refresh: unibi_string = 265;
pub const unibi_key_reference: unibi_string = 264;
pub const unibi_key_redo: unibi_string = 263;
pub const unibi_key_print: unibi_string = 262;
pub const unibi_key_previous: unibi_string = 261;
pub const unibi_key_options: unibi_string = 260;
pub const unibi_key_open: unibi_string = 259;
pub const unibi_key_next: unibi_string = 258;
pub const unibi_key_move: unibi_string = 257;
pub const unibi_key_message: unibi_string = 256;
pub const unibi_key_mark: unibi_string = 255;
pub const unibi_key_help: unibi_string = 254;
pub const unibi_key_find: unibi_string = 253;
pub const unibi_key_exit: unibi_string = 252;
pub const unibi_key_enter: unibi_string = 251;
pub const unibi_key_end: unibi_string = 250;
pub const unibi_key_create: unibi_string = 249;
pub const unibi_key_copy: unibi_string = 248;
pub const unibi_key_command: unibi_string = 247;
pub const unibi_key_close: unibi_string = 246;
pub const unibi_key_cancel: unibi_string = 245;
pub const unibi_key_beg: unibi_string = 244;
pub const unibi_label_off: unibi_string = 243;
pub const unibi_label_on: unibi_string = 242;
pub const unibi_ena_acs: unibi_string = 241;
pub const unibi_xoff_character: unibi_string = 240;
pub const unibi_xon_character: unibi_string = 239;
pub const unibi_exit_am_mode: unibi_string = 238;
pub const unibi_enter_am_mode: unibi_string = 237;
pub const unibi_exit_xon_mode: unibi_string = 236;
pub const unibi_enter_xon_mode: unibi_string = 235;
pub const unibi_key_btab: unibi_string = 234;
pub const unibi_plab_norm: unibi_string = 233;
pub const unibi_acs_chars: unibi_string = 232;
pub const unibi_char_padding: unibi_string = 231;
pub const unibi_prtr_non: unibi_string = 230;
pub const unibi_key_c3: unibi_string = 229;
pub const unibi_key_c1: unibi_string = 228;
pub const unibi_key_b2: unibi_string = 227;
pub const unibi_key_a3: unibi_string = 226;
pub const unibi_key_a1: unibi_string = 225;
pub const unibi_init_prog: unibi_string = 224;
pub const unibi_up_half_line: unibi_string = 223;
pub const unibi_underline_char: unibi_string = 222;
pub const unibi_to_status_line: unibi_string = 221;
pub const unibi_tab: unibi_string = 220;
pub const unibi_set_window: unibi_string = 219;
pub const unibi_set_tab: unibi_string = 218;
pub const unibi_set_attributes: unibi_string = 217;
pub const unibi_scroll_reverse: unibi_string = 216;
pub const unibi_scroll_forward: unibi_string = 215;
pub const unibi_save_cursor: unibi_string = 214;
pub const unibi_row_address: unibi_string = 213;
pub const unibi_restore_cursor: unibi_string = 212;
pub const unibi_reset_file: unibi_string = 211;
pub const unibi_reset_3string: unibi_string = 210;
pub const unibi_reset_2string: unibi_string = 209;
pub const unibi_reset_1string: unibi_string = 208;
pub const unibi_repeat_char: unibi_string = 207;
pub const unibi_prtr_on: unibi_string = 206;
pub const unibi_prtr_off: unibi_string = 205;
pub const unibi_print_screen: unibi_string = 204;
pub const unibi_pkey_xmit: unibi_string = 203;
pub const unibi_pkey_local: unibi_string = 202;
pub const unibi_pkey_key: unibi_string = 201;
pub const unibi_parm_up_cursor: unibi_string = 200;
pub const unibi_parm_rindex: unibi_string = 199;
pub const unibi_parm_right_cursor: unibi_string = 198;
pub const unibi_parm_left_cursor: unibi_string = 197;
pub const unibi_parm_insert_line: unibi_string = 196;
pub const unibi_parm_index: unibi_string = 195;
pub const unibi_parm_ich: unibi_string = 194;
pub const unibi_parm_down_cursor: unibi_string = 193;
pub const unibi_parm_delete_line: unibi_string = 192;
pub const unibi_parm_dch: unibi_string = 191;
pub const unibi_pad_char: unibi_string = 190;
pub const unibi_newline: unibi_string = 189;
pub const unibi_meta_on: unibi_string = 188;
pub const unibi_meta_off: unibi_string = 187;
pub const unibi_lab_f9: unibi_string = 186;
pub const unibi_lab_f8: unibi_string = 185;
pub const unibi_lab_f7: unibi_string = 184;
pub const unibi_lab_f6: unibi_string = 183;
pub const unibi_lab_f5: unibi_string = 182;
pub const unibi_lab_f4: unibi_string = 181;
pub const unibi_lab_f3: unibi_string = 180;
pub const unibi_lab_f2: unibi_string = 179;
pub const unibi_lab_f10: unibi_string = 178;
pub const unibi_lab_f1: unibi_string = 177;
pub const unibi_lab_f0: unibi_string = 176;
pub const unibi_keypad_xmit: unibi_string = 175;
pub const unibi_keypad_local: unibi_string = 174;
pub const unibi_key_up: unibi_string = 173;
pub const unibi_key_stab: unibi_string = 172;
pub const unibi_key_sr: unibi_string = 171;
pub const unibi_key_sf: unibi_string = 170;
pub const unibi_key_right: unibi_string = 169;
pub const unibi_key_ppage: unibi_string = 168;
pub const unibi_key_npage: unibi_string = 167;
pub const unibi_key_ll: unibi_string = 166;
pub const unibi_key_left: unibi_string = 165;
pub const unibi_key_il: unibi_string = 164;
pub const unibi_key_ic: unibi_string = 163;
pub const unibi_key_home: unibi_string = 162;
pub const unibi_key_f9: unibi_string = 161;
pub const unibi_key_f8: unibi_string = 160;
pub const unibi_key_f7: unibi_string = 159;
pub const unibi_key_f6: unibi_string = 158;
pub const unibi_key_f5: unibi_string = 157;
pub const unibi_key_f4: unibi_string = 156;
pub const unibi_key_f3: unibi_string = 155;
pub const unibi_key_f2: unibi_string = 154;
pub const unibi_key_f10: unibi_string = 153;
pub const unibi_key_f1: unibi_string = 152;
pub const unibi_key_f0: unibi_string = 151;
pub const unibi_key_eos: unibi_string = 150;
pub const unibi_key_eol: unibi_string = 149;
pub const unibi_key_eic: unibi_string = 148;
pub const unibi_key_down: unibi_string = 147;
pub const unibi_key_dl: unibi_string = 146;
pub const unibi_key_dc: unibi_string = 145;
pub const unibi_key_ctab: unibi_string = 144;
pub const unibi_key_clear: unibi_string = 143;
pub const unibi_key_catab: unibi_string = 142;
pub const unibi_key_backspace: unibi_string = 141;
pub const unibi_insert_padding: unibi_string = 140;
pub const unibi_insert_line: unibi_string = 139;
pub const unibi_insert_character: unibi_string = 138;
pub const unibi_init_file: unibi_string = 137;
pub const unibi_init_3string: unibi_string = 136;
pub const unibi_init_2string: unibi_string = 135;
pub const unibi_init_1string: unibi_string = 134;
pub const unibi_from_status_line: unibi_string = 133;
pub const unibi_form_feed: unibi_string = 132;
pub const unibi_flash_screen: unibi_string = 131;
pub const unibi_exit_underline_mode: unibi_string = 130;
pub const unibi_exit_standout_mode: unibi_string = 129;
pub const unibi_exit_insert_mode: unibi_string = 128;
pub const unibi_exit_delete_mode: unibi_string = 127;
pub const unibi_exit_ca_mode: unibi_string = 126;
pub const unibi_exit_attribute_mode: unibi_string = 125;
pub const unibi_exit_alt_charset_mode: unibi_string = 124;
pub const unibi_erase_chars: unibi_string = 123;
pub const unibi_enter_underline_mode: unibi_string = 122;
pub const unibi_enter_standout_mode: unibi_string = 121;
pub const unibi_enter_reverse_mode: unibi_string = 120;
pub const unibi_enter_protected_mode: unibi_string = 119;
pub const unibi_enter_secure_mode: unibi_string = 118;
pub const unibi_enter_insert_mode: unibi_string = 117;
pub const unibi_enter_dim_mode: unibi_string = 116;
pub const unibi_enter_delete_mode: unibi_string = 115;
pub const unibi_enter_ca_mode: unibi_string = 114;
pub const unibi_enter_bold_mode: unibi_string = 113;
pub const unibi_enter_blink_mode: unibi_string = 112;
pub const unibi_enter_alt_charset_mode: unibi_string = 111;
pub const unibi_down_half_line: unibi_string = 110;
pub const unibi_dis_status_line: unibi_string = 109;
pub const unibi_delete_line: unibi_string = 108;
pub const unibi_delete_character: unibi_string = 107;
pub const unibi_cursor_visible: unibi_string = 106;
pub const unibi_cursor_up: unibi_string = 105;
pub const unibi_cursor_to_ll: unibi_string = 104;
pub const unibi_cursor_right: unibi_string = 103;
pub const unibi_cursor_normal: unibi_string = 102;
pub const unibi_cursor_mem_address: unibi_string = 101;
pub const unibi_cursor_left: unibi_string = 100;
pub const unibi_cursor_invisible: unibi_string = 99;
pub const unibi_cursor_home: unibi_string = 98;
pub const unibi_cursor_down: unibi_string = 97;
pub const unibi_cursor_address: unibi_string = 96;
pub const unibi_command_character: unibi_string = 95;
pub const unibi_column_address: unibi_string = 94;
pub const unibi_clr_eos: unibi_string = 93;
pub const unibi_clr_eol: unibi_string = 92;
pub const unibi_clear_screen: unibi_string = 91;
pub const unibi_clear_all_tabs: unibi_string = 90;
pub const unibi_change_scroll_region: unibi_string = 89;
pub const unibi_carriage_return: unibi_string = 88;
pub const unibi_bell: unibi_string = 87;
pub const unibi_back_tab: unibi_string = 86;
pub const unibi_string_begin_: unibi_string = 85;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Arena {
    pub cur_blk: *mut ::core::ffi::c_char,
    pub pos: size_t,
    pub size: size_t,
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2Rust_Unnamed = 8;
pub const _ISpunct: C2Rust_Unnamed = 4;
pub const _IScntrl: C2Rust_Unnamed = 2;
pub const _ISblank: C2Rust_Unnamed = 1;
pub const _ISgraph: C2Rust_Unnamed = 32768;
pub const _ISprint: C2Rust_Unnamed = 16384;
pub const _ISspace: C2Rust_Unnamed = 8192;
pub const _ISxdigit: C2Rust_Unnamed = 4096;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const _ISalpha: C2Rust_Unnamed = 1024;
pub const _ISlower: C2Rust_Unnamed = 512;
pub const _ISupper: C2Rust_Unnamed = 256;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StringBuilder {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
}
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const kTermCount: C2Rust_Unnamed_0 = 49;
pub const kTerm_set_underline_style: C2Rust_Unnamed_0 = 48;
pub const kTerm_reset_cursor_color: C2Rust_Unnamed_0 = 47;
pub const kTerm_set_cursor_color: C2Rust_Unnamed_0 = 46;
pub const kTerm_set_rgb_background: C2Rust_Unnamed_0 = 45;
pub const kTerm_set_rgb_foreground: C2Rust_Unnamed_0 = 44;
pub const kTerm_enter_strikethrough_mode: C2Rust_Unnamed_0 = 43;
pub const kTerm_set_cursor_style: C2Rust_Unnamed_0 = 42;
pub const kTerm_reset_cursor_style: C2Rust_Unnamed_0 = 41;
pub const kTerm_to_status_line: C2Rust_Unnamed_0 = 40;
pub const kTerm_set_lr_margin: C2Rust_Unnamed_0 = 39;
pub const kTerm_set_attributes: C2Rust_Unnamed_0 = 38;
pub const kTerm_set_a_foreground: C2Rust_Unnamed_0 = 37;
pub const kTerm_set_a_background: C2Rust_Unnamed_0 = 36;
pub const kTerm_parm_up_cursor: C2Rust_Unnamed_0 = 35;
pub const kTerm_parm_right_cursor: C2Rust_Unnamed_0 = 34;
pub const kTerm_parm_left_cursor: C2Rust_Unnamed_0 = 33;
pub const kTerm_parm_insert_line: C2Rust_Unnamed_0 = 32;
pub const kTerm_parm_down_cursor: C2Rust_Unnamed_0 = 31;
pub const kTerm_parm_delete_line: C2Rust_Unnamed_0 = 30;
pub const kTerm_keypad_xmit: C2Rust_Unnamed_0 = 29;
pub const kTerm_keypad_local: C2Rust_Unnamed_0 = 28;
pub const kTerm_insert_line: C2Rust_Unnamed_0 = 27;
pub const kTerm_from_status_line: C2Rust_Unnamed_0 = 26;
pub const kTerm_exit_ca_mode: C2Rust_Unnamed_0 = 25;
pub const kTerm_exit_attribute_mode: C2Rust_Unnamed_0 = 24;
pub const kTerm_erase_chars: C2Rust_Unnamed_0 = 23;
pub const kTerm_enter_underline_mode: C2Rust_Unnamed_0 = 22;
pub const kTerm_enter_standout_mode: C2Rust_Unnamed_0 = 21;
pub const kTerm_enter_secure_mode: C2Rust_Unnamed_0 = 20;
pub const kTerm_enter_reverse_mode: C2Rust_Unnamed_0 = 19;
pub const kTerm_enter_italics_mode: C2Rust_Unnamed_0 = 18;
pub const kTerm_enter_dim_mode: C2Rust_Unnamed_0 = 17;
pub const kTerm_enter_ca_mode: C2Rust_Unnamed_0 = 16;
pub const kTerm_enter_bold_mode: C2Rust_Unnamed_0 = 15;
pub const kTerm_enter_blink_mode: C2Rust_Unnamed_0 = 14;
pub const kTerm_delete_line: C2Rust_Unnamed_0 = 13;
pub const kTerm_cursor_right: C2Rust_Unnamed_0 = 12;
pub const kTerm_cursor_up: C2Rust_Unnamed_0 = 11;
pub const kTerm_cursor_normal: C2Rust_Unnamed_0 = 10;
pub const kTerm_cursor_home: C2Rust_Unnamed_0 = 9;
pub const kTerm_cursor_left: C2Rust_Unnamed_0 = 8;
pub const kTerm_cursor_invisible: C2Rust_Unnamed_0 = 7;
pub const kTerm_cursor_down: C2Rust_Unnamed_0 = 6;
pub const kTerm_cursor_address: C2Rust_Unnamed_0 = 5;
pub const kTerm_clr_eos: C2Rust_Unnamed_0 = 4;
pub const kTerm_clr_eol: C2Rust_Unnamed_0 = 3;
pub const kTerm_clear_screen: C2Rust_Unnamed_0 = 2;
pub const kTerm_change_scroll_region: C2Rust_Unnamed_0 = 1;
pub const kTerm_carriage_return: C2Rust_Unnamed_0 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TerminfoEntry {
    pub bce: bool,
    pub has_Tc_or_RGB: bool,
    pub Su: bool,
    pub max_colors: ::core::ffi::c_int,
    pub lines: ::core::ffi::c_int,
    pub columns: ::core::ffi::c_int,
    pub defs: [*const ::core::ffi::c_char; 49],
    pub keys: [[*const ::core::ffi::c_char; 2]; 16],
    pub f_keys: [*const ::core::ffi::c_char; 63],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TPVAR {
    pub num: ::core::ffi::c_long,
    pub string: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TPSTACK {
    pub nums: [::core::ffi::c_long; 20],
    pub strings: [*mut ::core::ffi::c_char; 20],
    pub offset: size_t,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: StringBuilder = StringBuilder {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn terminfo_is_term_family(
    mut term: *const ::core::ffi::c_char,
    mut family: *const ::core::ffi::c_char,
) -> bool {
    if term.is_null() {
        return false_0 != 0;
    }
    let mut tlen: size_t = strlen(term);
    let mut flen: size_t = strlen(family);
    return tlen >= flen
        && 0 as ::core::ffi::c_int
            == memcmp(
                term as *const ::core::ffi::c_void,
                family as *const ::core::ffi::c_void,
                flen,
            )
        && (NUL == *term.offset(flen as isize) as ::core::ffi::c_int
            || '-' as ::core::ffi::c_int == *term.offset(flen as isize) as ::core::ffi::c_int
            || '.' as ::core::ffi::c_int == *term.offset(flen as isize) as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn terminfo_is_bsd_console(mut _term: *const ::core::ffi::c_char) -> bool {
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn terminfo_from_builtin(
    mut term: *const ::core::ffi::c_char,
    mut termname: *mut *mut ::core::ffi::c_char,
) -> *const TerminfoEntry {
    if strequal(term, b"ghostty\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int
        != 0
        || strequal(
            term,
            b"xterm-ghostty\0".as_ptr() as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int
            != 0
    {
        *termname = b"ghostty\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return (ghostty_terminfo.ptr() as *const _);
    } else if terminfo_is_term_family(term, b"xterm\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"xterm\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return (xterm_256colour_terminfo.ptr() as *const _);
    } else if terminfo_is_term_family(term, b"screen\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"screen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return (screen_256colour_terminfo.ptr() as *const _);
    } else if terminfo_is_term_family(term, b"tmux\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"tmux\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return (tmux_256colour_terminfo.ptr() as *const _);
    } else if terminfo_is_term_family(term, b"rxvt\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"rxvt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return (rxvt_256colour_terminfo.ptr() as *const _);
    } else if terminfo_is_term_family(term, b"putty\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"putty\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return (putty_256colour_terminfo.ptr() as *const _);
    } else if terminfo_is_term_family(term, b"linux\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"linux\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return (linux_16colour_terminfo.ptr() as *const _);
    } else if terminfo_is_term_family(term, b"interix\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"interix\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return (interix_8colour_terminfo.ptr() as *const _);
    } else if terminfo_is_term_family(term, b"iterm\0".as_ptr() as *const ::core::ffi::c_char)
        as ::core::ffi::c_int
        != 0
        || terminfo_is_term_family(term, b"iterm2\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
        || terminfo_is_term_family(term, b"iTerm.app\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
        || terminfo_is_term_family(term, b"iTerm2.app\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
    {
        *termname = b"iterm\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return (iterm_256colour_terminfo.ptr() as *const _);
    } else if terminfo_is_term_family(term, b"st\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"st\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return (st_256colour_terminfo.ptr() as *const _);
    } else if terminfo_is_term_family(term, b"gnome\0".as_ptr() as *const ::core::ffi::c_char)
        as ::core::ffi::c_int
        != 0
        || terminfo_is_term_family(term, b"vte\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
    {
        *termname = b"vte\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return (vte_256colour_terminfo.ptr() as *const _);
    } else if terminfo_is_term_family(term, b"cygwin\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"cygwin\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return (cygwin_terminfo.ptr() as *const _);
    } else if terminfo_is_term_family(term, b"win32con\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname =
            b"win32con\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return (win32con_terminfo.ptr() as *const _);
    } else if terminfo_is_term_family(term, b"conemu\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"conemu\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return (conemu_terminfo.ptr() as *const _);
    } else if terminfo_is_term_family(term, b"vtpcon\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"vtpcon\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return (vtpcon_terminfo.ptr() as *const _);
    } else {
        *termname = b"ansi\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return (ansi_terminfo.ptr() as *const _);
    };
}
#[no_mangle]
pub unsafe extern "C" fn terminfo_from_database(
    mut ti: *mut TerminfoEntry,
    mut termname: *mut ::core::ffi::c_char,
    mut arena: *mut Arena,
) -> bool {
    let mut ut: *mut unibi_term = unibi_from_term(termname);
    if ut.is_null() {
        return false_0 != 0;
    }
    (*ti).bce = unibi_get_bool(ut, unibi_back_color_erase) != 0;
    (*ti).max_colors = unibi_get_num(ut, unibi_max_colors);
    (*ti).lines = unibi_get_num(ut, unibi_lines);
    (*ti).columns = unibi_get_num(ut, unibi_columns);
    (*ti).has_Tc_or_RGB = false_0 != 0;
    (*ti).Su = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < unibi_count_ext_bool(ut) {
        let mut n: *const ::core::ffi::c_char = unibi_get_ext_bool_name(ut, i);
        if !n.is_null()
            && (strcmp(n, b"Tc\0".as_ptr() as *const ::core::ffi::c_char) == 0
                || strcmp(n, b"RGB\0".as_ptr() as *const ::core::ffi::c_char) == 0)
        {
            (*ti).has_Tc_or_RGB = true_0 != 0;
        } else if !n.is_null() && strcmp(n, b"Su\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
            (*ti).Su = true_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    static uni_ids: GlobalCell<[unibi_string; 41]> = GlobalCell::new([
        unibi_carriage_return,
        unibi_change_scroll_region,
        unibi_clear_screen,
        unibi_clr_eol,
        unibi_clr_eos,
        unibi_cursor_address,
        unibi_cursor_down,
        unibi_cursor_invisible,
        unibi_cursor_left,
        unibi_cursor_home,
        unibi_cursor_normal,
        unibi_cursor_up,
        unibi_cursor_right,
        unibi_delete_line,
        unibi_enter_blink_mode,
        unibi_enter_bold_mode,
        unibi_enter_ca_mode,
        unibi_enter_dim_mode,
        unibi_enter_italics_mode,
        unibi_enter_reverse_mode,
        unibi_enter_secure_mode,
        unibi_enter_standout_mode,
        unibi_enter_underline_mode,
        unibi_erase_chars,
        unibi_exit_attribute_mode,
        unibi_exit_ca_mode,
        unibi_from_status_line,
        unibi_insert_line,
        unibi_keypad_local,
        unibi_keypad_xmit,
        unibi_parm_delete_line,
        unibi_parm_down_cursor,
        unibi_parm_insert_line,
        unibi_parm_left_cursor,
        unibi_parm_right_cursor,
        unibi_parm_up_cursor,
        unibi_set_a_background,
        unibi_set_a_foreground,
        unibi_set_attributes,
        unibi_set_lr_margin,
        unibi_to_status_line,
    ]);
    let mut i_0: size_t = 0 as size_t;
    while i_0
        < ::core::mem::size_of::<[unibi_string; 41]>()
            .wrapping_div(::core::mem::size_of::<unibi_string>())
            .wrapping_div(
                (::core::mem::size_of::<[unibi_string; 41]>()
                    .wrapping_rem(::core::mem::size_of::<unibi_string>())
                    == 0) as ::core::ffi::c_int as usize,
            )
    {
        let mut val: *const ::core::ffi::c_char = unibi_get_str(ut, (*uni_ids.ptr())[i_0 as usize]);
        (*ti).defs[i_0 as usize] = if !val.is_null() {
            arena_strdup(arena, val)
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        };
        i_0 = i_0.wrapping_add(1);
    }
    static uni_ext: GlobalCell<[*const ::core::ffi::c_char; 8]> = GlobalCell::new([
        b"Se\0".as_ptr() as *const ::core::ffi::c_char,
        b"Ss\0".as_ptr() as *const ::core::ffi::c_char,
        b"smxx\0".as_ptr() as *const ::core::ffi::c_char,
        b"setrgbf\0".as_ptr() as *const ::core::ffi::c_char,
        b"setrgbb\0".as_ptr() as *const ::core::ffi::c_char,
        b"Cs\0".as_ptr() as *const ::core::ffi::c_char,
        b"Cr\0".as_ptr() as *const ::core::ffi::c_char,
        b"Smulx\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    let mut max: size_t = unibi_count_ext_str(ut);
    let mut i_1: size_t = 0 as size_t;
    while i_1
        < ::core::mem::size_of::<[*const ::core::ffi::c_char; 8]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 8]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            )
    {
        let mut name: *const ::core::ffi::c_char = (*uni_ext.ptr())[i_1 as usize];
        let mut val_0: size_t = 0 as size_t;
        while val_0 < max {
            let mut n_0: *const ::core::ffi::c_char = unibi_get_ext_str_name(ut, val_0);
            if !n_0.is_null() && strequal(n_0, name) as ::core::ffi::c_int != 0 {
                let mut data: *const ::core::ffi::c_char = unibi_get_ext_str(ut, val_0);
                (*ti).defs[(kTerm_reset_cursor_style as ::core::ffi::c_int as size_t)
                    .wrapping_add(i_1) as usize] = if !data.is_null() {
                    arena_strdup(arena, data)
                } else {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                };
                break;
            } else {
                val_0 = val_0.wrapping_add(1);
            }
        }
        i_1 = i_1.wrapping_add(1);
    }
    static uni_keys: GlobalCell<[[unibi_string; 2]; 16]> = GlobalCell::new([
        [unibi_key_backspace, unibi_string_begin_],
        [unibi_key_beg, unibi_key_sbeg],
        [unibi_key_btab, unibi_string_begin_],
        [unibi_key_clear, unibi_string_begin_],
        [unibi_key_dc, unibi_key_sdc],
        [unibi_key_end, unibi_key_send],
        [unibi_key_find, unibi_key_sfind],
        [unibi_key_home, unibi_key_shome],
        [unibi_key_ic, unibi_key_sic],
        [unibi_key_npage, unibi_string_begin_],
        [unibi_key_ppage, unibi_string_begin_],
        [unibi_key_select, unibi_string_begin_],
        [unibi_key_suspend, unibi_key_ssuspend],
        [unibi_key_undo, unibi_key_sundo],
        [unibi_key_left, unibi_key_sleft],
        [unibi_key_right, unibi_key_sright],
    ]);
    let mut i_2: size_t = 0 as size_t;
    while i_2
        < ::core::mem::size_of::<[[unibi_string; 2]; 16]>()
            .wrapping_div(::core::mem::size_of::<[unibi_string; 2]>())
            .wrapping_div(
                (::core::mem::size_of::<[[unibi_string; 2]; 16]>()
                    .wrapping_rem(::core::mem::size_of::<[unibi_string; 2]>())
                    == 0) as ::core::ffi::c_int as usize,
            )
    {
        let mut val_1: *const ::core::ffi::c_char = unibi_get_str(
            ut,
            (*uni_keys.ptr())[i_2 as usize][0 as ::core::ffi::c_int as usize],
        );
        if !val_1.is_null() {
            (*ti).keys[i_2 as usize][0 as ::core::ffi::c_int as usize] = arena_strdup(arena, val_1);
            if (*uni_keys.ptr())[i_2 as usize][1 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_uint
                != unibi_string_begin_ as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut sval: *const ::core::ffi::c_char = unibi_get_str(
                    ut,
                    (*uni_keys.ptr())[i_2 as usize][1 as ::core::ffi::c_int as usize],
                );
                (*ti).keys[i_2 as usize][1 as ::core::ffi::c_int as usize] = if !sval.is_null() {
                    arena_strdup(arena, sval)
                } else {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                };
            }
        }
        i_2 = i_2.wrapping_add(1);
    }
    static uni_fkeys: GlobalCell<[unibi_string; 63]> = GlobalCell::new([
        unibi_key_f1,
        unibi_key_f2,
        unibi_key_f3,
        unibi_key_f4,
        unibi_key_f5,
        unibi_key_f6,
        unibi_key_f7,
        unibi_key_f8,
        unibi_key_f9,
        unibi_key_f10,
        unibi_key_f11,
        unibi_key_f12,
        unibi_key_f13,
        unibi_key_f14,
        unibi_key_f15,
        unibi_key_f16,
        unibi_key_f17,
        unibi_key_f18,
        unibi_key_f19,
        unibi_key_f20,
        unibi_key_f21,
        unibi_key_f22,
        unibi_key_f23,
        unibi_key_f24,
        unibi_key_f25,
        unibi_key_f26,
        unibi_key_f27,
        unibi_key_f28,
        unibi_key_f29,
        unibi_key_f30,
        unibi_key_f31,
        unibi_key_f32,
        unibi_key_f33,
        unibi_key_f34,
        unibi_key_f35,
        unibi_key_f36,
        unibi_key_f37,
        unibi_key_f38,
        unibi_key_f39,
        unibi_key_f40,
        unibi_key_f41,
        unibi_key_f42,
        unibi_key_f43,
        unibi_key_f44,
        unibi_key_f45,
        unibi_key_f46,
        unibi_key_f47,
        unibi_key_f48,
        unibi_key_f49,
        unibi_key_f50,
        unibi_key_f51,
        unibi_key_f52,
        unibi_key_f53,
        unibi_key_f54,
        unibi_key_f55,
        unibi_key_f56,
        unibi_key_f57,
        unibi_key_f58,
        unibi_key_f59,
        unibi_key_f60,
        unibi_key_f61,
        unibi_key_f62,
        unibi_key_f63,
    ]);
    let mut i_3: size_t = 0 as size_t;
    while i_3
        < ::core::mem::size_of::<[unibi_string; 63]>()
            .wrapping_div(::core::mem::size_of::<unibi_string>())
            .wrapping_div(
                (::core::mem::size_of::<[unibi_string; 63]>()
                    .wrapping_rem(::core::mem::size_of::<unibi_string>())
                    == 0) as ::core::ffi::c_int as usize,
            )
    {
        let mut val_2: *const ::core::ffi::c_char =
            unibi_get_str(ut, (*uni_fkeys.ptr())[i_3 as usize]);
        (*ti).f_keys[i_3 as usize] = if !val_2.is_null() {
            arena_strdup(arena, val_2)
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        };
        i_3 = i_3.wrapping_add(1);
    }
    unibi_destroy(ut);
    return true_0 != 0;
}
unsafe extern "C" fn fmt(mut val: bool) -> *const ::core::ffi::c_char {
    return if val as ::core::ffi::c_int != 0 {
        b"true\0".as_ptr() as *const ::core::ffi::c_char
    } else {
        b"false\0".as_ptr() as *const ::core::ffi::c_char
    };
}
#[no_mangle]
pub unsafe extern "C" fn terminfo_info_msg(
    mut ti: *const TerminfoEntry,
    mut termname: *const ::core::ffi::c_char,
    mut from_db: bool,
) -> String_0 {
    let mut data: StringBuilder = KV_INITIAL_VALUE;
    kv_do_printf(
        &raw mut data,
        b"&term: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        termname,
    );
    if from_db {
        kv_do_printf(
            &raw mut data,
            b"using terminfo database\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        kv_do_printf(
            &raw mut data,
            b"using builtin terminfo\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    kv_do_printf(
        &raw mut data,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    kv_do_printf(
        &raw mut data,
        b"Boolean capabilities:\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    kv_do_printf(
        &raw mut data,
        b"  back_color_erase: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        fmt((*ti).bce),
    );
    kv_do_printf(
        &raw mut data,
        b"  truecolor ('Tc' or 'RGB'): %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        fmt((*ti).has_Tc_or_RGB),
    );
    kv_do_printf(
        &raw mut data,
        b"  extended underline ('Su'): %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        fmt((*ti).Su),
    );
    kv_do_printf(
        &raw mut data,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    kv_do_printf(
        &raw mut data,
        b"Numeric capabilities: (-1 for unknown)\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    kv_do_printf(
        &raw mut data,
        b"  lines: %d\n\0".as_ptr() as *const ::core::ffi::c_char,
        (*ti).lines,
    );
    kv_do_printf(
        &raw mut data,
        b"  columns: %d\n\0".as_ptr() as *const ::core::ffi::c_char,
        (*ti).columns,
    );
    kv_do_printf(
        &raw mut data,
        b"  max_colors: %d\n\0".as_ptr() as *const ::core::ffi::c_char,
        (*ti).columns,
    );
    kv_do_printf(
        &raw mut data,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    kv_do_printf(
        &raw mut data,
        b"String capabilities:\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    static string_names: GlobalCell<[*const ::core::ffi::c_char; 49]> = GlobalCell::new([
        b"carriage_return\0".as_ptr() as *const ::core::ffi::c_char,
        b"change_scroll_region\0".as_ptr() as *const ::core::ffi::c_char,
        b"clear_screen\0".as_ptr() as *const ::core::ffi::c_char,
        b"clr_eol\0".as_ptr() as *const ::core::ffi::c_char,
        b"clr_eos\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor_address\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor_down\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor_invisible\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor_left\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor_home\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor_normal\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor_up\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor_right\0".as_ptr() as *const ::core::ffi::c_char,
        b"delete_line\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_blink_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_bold_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_ca_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_dim_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_italics_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_reverse_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_secure_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_standout_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_underline_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"erase_chars\0".as_ptr() as *const ::core::ffi::c_char,
        b"exit_attribute_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"exit_ca_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"from_status_line\0".as_ptr() as *const ::core::ffi::c_char,
        b"insert_line\0".as_ptr() as *const ::core::ffi::c_char,
        b"keypad_local\0".as_ptr() as *const ::core::ffi::c_char,
        b"keypad_xmit\0".as_ptr() as *const ::core::ffi::c_char,
        b"parm_delete_line\0".as_ptr() as *const ::core::ffi::c_char,
        b"parm_down_cursor\0".as_ptr() as *const ::core::ffi::c_char,
        b"parm_insert_line\0".as_ptr() as *const ::core::ffi::c_char,
        b"parm_left_cursor\0".as_ptr() as *const ::core::ffi::c_char,
        b"parm_right_cursor\0".as_ptr() as *const ::core::ffi::c_char,
        b"parm_up_cursor\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_a_background\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_a_foreground\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_attributes\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_lr_margin\0".as_ptr() as *const ::core::ffi::c_char,
        b"to_status_line\0".as_ptr() as *const ::core::ffi::c_char,
        b"reset_cursor_style (Se)\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_cursor_style (Ss)\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_strikethrough_mode (smxx)\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_rgb_foreground (setrgbf)\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_rgb_background (setrgbb)\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_cursor_color (Cs)\0".as_ptr() as *const ::core::ffi::c_char,
        b"reset_cursor_color (Cr)\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_underline_style (Smulx)\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<[*const ::core::ffi::c_char; 49]>()
        .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
        .wrapping_div(
            (::core::mem::size_of::<[*const ::core::ffi::c_char; 49]>()
                .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                == 0) as ::core::ffi::c_int as usize,
        )
    {
        let mut s: *const ::core::ffi::c_char = (*ti).defs[i as usize];
        if !s.is_null() {
            kv_do_printf(
                &raw mut data,
                b"  %-31s = \0".as_ptr() as *const ::core::ffi::c_char,
                (*string_names.ptr())[i as usize],
            );
            kv_transstr(&raw mut data, s, false_0 != 0);
            if data.size == data.capacity {
                data.capacity = if data.capacity != 0 {
                    data.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                data.items = xrealloc(
                    data.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(data.capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh0 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh0 as isize) = '\n' as ::core::ffi::c_char;
        }
        i = i.wrapping_add(1);
    }
    static key_names: GlobalCell<[*const ::core::ffi::c_char; 16]> = GlobalCell::new([
        b"backspace\0".as_ptr() as *const ::core::ffi::c_char,
        b"beg\0".as_ptr() as *const ::core::ffi::c_char,
        b"btab\0".as_ptr() as *const ::core::ffi::c_char,
        b"clear\0".as_ptr() as *const ::core::ffi::c_char,
        b"dc\0".as_ptr() as *const ::core::ffi::c_char,
        b"end\0".as_ptr() as *const ::core::ffi::c_char,
        b"find\0".as_ptr() as *const ::core::ffi::c_char,
        b"home\0".as_ptr() as *const ::core::ffi::c_char,
        b"ic\0".as_ptr() as *const ::core::ffi::c_char,
        b"npage\0".as_ptr() as *const ::core::ffi::c_char,
        b"ppage\0".as_ptr() as *const ::core::ffi::c_char,
        b"select\0".as_ptr() as *const ::core::ffi::c_char,
        b"suspend\0".as_ptr() as *const ::core::ffi::c_char,
        b"undo\0".as_ptr() as *const ::core::ffi::c_char,
        b"left\0".as_ptr() as *const ::core::ffi::c_char,
        b"right\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    let mut i_0: size_t = (0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t;
    while i_0
        < ::core::mem::size_of::<[*const ::core::ffi::c_char; 16]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 16]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            )
    {
        let mut s_0: *const ::core::ffi::c_char =
            (*ti).keys[i_0 as usize][0 as ::core::ffi::c_int as usize];
        if !s_0.is_null() {
            kv_do_printf(
                &raw mut data,
                b"  key_%-27s = \0".as_ptr() as *const ::core::ffi::c_char,
                (*key_names.ptr())[i_0 as usize],
            );
            kv_transstr(&raw mut data, s_0, false_0 != 0);
            let mut ss: *const ::core::ffi::c_char =
                (*ti).keys[i_0 as usize][1 as ::core::ffi::c_int as usize];
            if !ss.is_null() {
                kv_do_printf(
                    &raw mut data,
                    b", key_s%s = \0".as_ptr() as *const ::core::ffi::c_char,
                    (*key_names.ptr())[i_0 as usize],
                );
                kv_transstr(&raw mut data, ss, false_0 != 0);
            }
            if data.size == data.capacity {
                data.capacity = if data.capacity != 0 {
                    data.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                data.items = xrealloc(
                    data.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(data.capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh1 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh1 as isize) = '\n' as ::core::ffi::c_char;
        }
        i_0 = i_0.wrapping_add(1);
    }
    static fkey_names: GlobalCell<[*const ::core::ffi::c_char; 63]> = GlobalCell::new([
        b"f1\0".as_ptr() as *const ::core::ffi::c_char,
        b"f2\0".as_ptr() as *const ::core::ffi::c_char,
        b"f3\0".as_ptr() as *const ::core::ffi::c_char,
        b"f4\0".as_ptr() as *const ::core::ffi::c_char,
        b"f5\0".as_ptr() as *const ::core::ffi::c_char,
        b"f6\0".as_ptr() as *const ::core::ffi::c_char,
        b"f7\0".as_ptr() as *const ::core::ffi::c_char,
        b"f8\0".as_ptr() as *const ::core::ffi::c_char,
        b"f9\0".as_ptr() as *const ::core::ffi::c_char,
        b"f10\0".as_ptr() as *const ::core::ffi::c_char,
        b"f11\0".as_ptr() as *const ::core::ffi::c_char,
        b"f12\0".as_ptr() as *const ::core::ffi::c_char,
        b"f13\0".as_ptr() as *const ::core::ffi::c_char,
        b"f14\0".as_ptr() as *const ::core::ffi::c_char,
        b"f15\0".as_ptr() as *const ::core::ffi::c_char,
        b"f16\0".as_ptr() as *const ::core::ffi::c_char,
        b"f17\0".as_ptr() as *const ::core::ffi::c_char,
        b"f18\0".as_ptr() as *const ::core::ffi::c_char,
        b"f19\0".as_ptr() as *const ::core::ffi::c_char,
        b"f20\0".as_ptr() as *const ::core::ffi::c_char,
        b"f21\0".as_ptr() as *const ::core::ffi::c_char,
        b"f22\0".as_ptr() as *const ::core::ffi::c_char,
        b"f23\0".as_ptr() as *const ::core::ffi::c_char,
        b"f24\0".as_ptr() as *const ::core::ffi::c_char,
        b"f25\0".as_ptr() as *const ::core::ffi::c_char,
        b"f26\0".as_ptr() as *const ::core::ffi::c_char,
        b"f27\0".as_ptr() as *const ::core::ffi::c_char,
        b"f28\0".as_ptr() as *const ::core::ffi::c_char,
        b"f29\0".as_ptr() as *const ::core::ffi::c_char,
        b"f30\0".as_ptr() as *const ::core::ffi::c_char,
        b"f31\0".as_ptr() as *const ::core::ffi::c_char,
        b"f32\0".as_ptr() as *const ::core::ffi::c_char,
        b"f33\0".as_ptr() as *const ::core::ffi::c_char,
        b"f34\0".as_ptr() as *const ::core::ffi::c_char,
        b"f35\0".as_ptr() as *const ::core::ffi::c_char,
        b"f36\0".as_ptr() as *const ::core::ffi::c_char,
        b"f37\0".as_ptr() as *const ::core::ffi::c_char,
        b"f38\0".as_ptr() as *const ::core::ffi::c_char,
        b"f39\0".as_ptr() as *const ::core::ffi::c_char,
        b"f40\0".as_ptr() as *const ::core::ffi::c_char,
        b"f41\0".as_ptr() as *const ::core::ffi::c_char,
        b"f42\0".as_ptr() as *const ::core::ffi::c_char,
        b"f43\0".as_ptr() as *const ::core::ffi::c_char,
        b"f44\0".as_ptr() as *const ::core::ffi::c_char,
        b"f45\0".as_ptr() as *const ::core::ffi::c_char,
        b"f46\0".as_ptr() as *const ::core::ffi::c_char,
        b"f47\0".as_ptr() as *const ::core::ffi::c_char,
        b"f48\0".as_ptr() as *const ::core::ffi::c_char,
        b"f49\0".as_ptr() as *const ::core::ffi::c_char,
        b"f50\0".as_ptr() as *const ::core::ffi::c_char,
        b"f51\0".as_ptr() as *const ::core::ffi::c_char,
        b"f52\0".as_ptr() as *const ::core::ffi::c_char,
        b"f53\0".as_ptr() as *const ::core::ffi::c_char,
        b"f54\0".as_ptr() as *const ::core::ffi::c_char,
        b"f55\0".as_ptr() as *const ::core::ffi::c_char,
        b"f56\0".as_ptr() as *const ::core::ffi::c_char,
        b"f57\0".as_ptr() as *const ::core::ffi::c_char,
        b"f58\0".as_ptr() as *const ::core::ffi::c_char,
        b"f59\0".as_ptr() as *const ::core::ffi::c_char,
        b"f60\0".as_ptr() as *const ::core::ffi::c_char,
        b"f61\0".as_ptr() as *const ::core::ffi::c_char,
        b"f62\0".as_ptr() as *const ::core::ffi::c_char,
        b"f63\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    let mut i_1: size_t = (0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t;
    while i_1
        < ::core::mem::size_of::<[*const ::core::ffi::c_char; 63]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 63]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            )
    {
        let mut s_1: *const ::core::ffi::c_char = (*ti).f_keys[i_1 as usize];
        if !s_1.is_null() {
            kv_do_printf(
                &raw mut data,
                b"  key_%-27s = \0".as_ptr() as *const ::core::ffi::c_char,
                (*fkey_names.ptr())[i_1 as usize],
            );
            kv_transstr(&raw mut data, s_1, false_0 != 0);
            if data.size == data.capacity {
                data.capacity = if data.capacity != 0 {
                    data.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                data.items = xrealloc(
                    data.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(data.capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh2 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh2 as isize) = '\n' as ::core::ffi::c_char;
        }
        i_1 = i_1.wrapping_add(1);
    }
    if data.size == data.capacity {
        data.capacity = if data.capacity != 0 {
            data.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        data.items = xrealloc(
            data.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(data.capacity),
        ) as *mut ::core::ffi::c_char;
    } else {
    };
    let c2rust_fresh3 = data.size;
    data.size = data.size.wrapping_add(1);
    *data.items.offset(c2rust_fresh3 as isize) = '\0' as ::core::ffi::c_char;
    return String_0 {
        data: data.items,
        size: data.size.wrapping_sub(1 as size_t),
    };
}
unsafe extern "C" fn push(
    mut num: ::core::ffi::c_long,
    mut string: *mut ::core::ffi::c_char,
    mut stack: *mut TPSTACK,
) -> ::core::ffi::c_int {
    if (*stack).offset
        >= ::core::mem::size_of::<[::core::ffi::c_long; 20]>()
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_long>())
            .wrapping_div(
                (::core::mem::size_of::<[::core::ffi::c_long; 20]>()
                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_long>())
                    == 0) as ::core::ffi::c_int as usize,
            )
    {
        return -1 as ::core::ffi::c_int;
    }
    (*stack).nums[(*stack).offset as usize] = num;
    (*stack).strings[(*stack).offset as usize] = string;
    (*stack).offset = (*stack).offset.wrapping_add(1);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn pop(
    mut num: *mut ::core::ffi::c_long,
    mut string: *mut *mut ::core::ffi::c_char,
    mut stack: *mut TPSTACK,
) -> ::core::ffi::c_int {
    if (*stack).offset == 0 as size_t {
        if !num.is_null() {
            *num = 0 as ::core::ffi::c_long;
        }
        if !string.is_null() {
            *string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        return -1 as ::core::ffi::c_int;
    }
    (*stack).offset = (*stack).offset.wrapping_sub(1);
    if !num.is_null() {
        *num = (*stack).nums[(*stack).offset as usize];
    }
    if !string.is_null() {
        *string = (*stack).strings[(*stack).offset as usize];
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn ochar(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buf_end: *const ::core::ffi::c_char,
    mut c: ::core::ffi::c_int,
) -> bool {
    if c == 0 as ::core::ffi::c_int {
        c = 0o200 as ::core::ffi::c_int;
    }
    if buf_end.offset_from(*buf) < 2 as isize {
        return false;
    }
    let c2rust_fresh18 = *buf;
    *buf = (*buf).offset(1);
    *c2rust_fresh18 = c as ::core::ffi::c_char;
    return true;
}
unsafe extern "C" fn onum(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buf_end: *const ::core::ffi::c_char,
    mut fmt_0: *const ::core::ffi::c_char,
    mut num: ::core::ffi::c_int,
    mut len: size_t,
) -> bool {
    let LONG_STR_MAX: size_t = 21 as size_t;
    len = if len > LONG_STR_MAX {
        len
    } else {
        LONG_STR_MAX
    };
    if buf_end.offset_from(*buf) < len.wrapping_add(2 as size_t) as isize {
        return false;
    }
    let mut l: ::core::ffi::c_int = snprintf(*buf, len.wrapping_add(2 as size_t), fmt_0, num);
    if l == -1 as ::core::ffi::c_int {
        return false;
    }
    *buf = (*buf).offset(l as isize);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn terminfo_fmt(
    mut buf_start: *mut ::core::ffi::c_char,
    mut buf_end: *mut ::core::ffi::c_char,
    mut str: *const ::core::ffi::c_char,
    mut params: *mut TPVAR,
) -> size_t {
    let mut c: ::core::ffi::c_char = 0;
    let mut fmt_0: [::core::ffi::c_char; 64] = [0; 64];
    let mut fp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ostr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut val: ::core::ffi::c_long = 0;
    let mut val2: ::core::ffi::c_long = 0;
    let mut dnums: [::core::ffi::c_long; 26] = [0; 26];
    let mut snums: [::core::ffi::c_long; 26] = [0; 26];
    memset(
        &raw mut dnums as *mut ::core::ffi::c_long as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[::core::ffi::c_long; 26]>(),
    );
    memset(
        &raw mut snums as *mut ::core::ffi::c_long as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[::core::ffi::c_long; 26]>(),
    );
    let mut buf: *mut ::core::ffi::c_char = buf_start;
    let mut l: size_t = 0;
    let mut width: size_t = 0;
    let mut precision: size_t = 0;
    let mut olen: size_t = 0;
    let mut stack: TPSTACK = TPSTACK {
        nums: [0; 20],
        strings: [::core::ptr::null_mut::<::core::ffi::c_char>(); 20],
        offset: 0,
    };
    let mut done: ::core::ffi::c_uint = 0;
    let mut dot: ::core::ffi::c_uint = 0;
    let mut minus: ::core::ffi::c_uint = 0;
    memset(
        &raw mut stack as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<TPSTACK>(),
    );
    loop {
        let c2rust_fresh4 = str;
        str = str.offset(1);
        c = *c2rust_fresh4;
        if c as ::core::ffi::c_int == '\0' as ::core::ffi::c_int {
            break;
        }
        if c as ::core::ffi::c_int != '%' as ::core::ffi::c_int || {
            let c2rust_fresh5 = str;
            str = str.offset(1);
            c = *c2rust_fresh5;
            c as ::core::ffi::c_int == '%' as ::core::ffi::c_int
        } {
            if c as ::core::ffi::c_int == '\0' as ::core::ffi::c_int {
                break;
            }
            if !ochar(&raw mut buf, buf_end, c as ::core::ffi::c_int) {
                return false_0 as size_t;
            }
        } else {
            fp = &raw mut fmt_0 as *mut ::core::ffi::c_char;
            let c2rust_fresh6 = fp;
            fp = fp.offset(1);
            *c2rust_fresh6 = '%' as ::core::ffi::c_char;
            minus = 0 as ::core::ffi::c_uint;
            dot = minus;
            done = dot;
            precision = 0 as size_t;
            width = precision;
            val = 0 as ::core::ffi::c_long;
            while done == 0 as ::core::ffi::c_uint
                && (fp.offset_from(&raw mut fmt_0 as *mut ::core::ffi::c_char) as size_t)
                    < ::core::mem::size_of::<[::core::ffi::c_char; 64]>()
            {
                match c as ::core::ffi::c_int {
                    99 | 115 => {
                        let c2rust_fresh7 = fp;
                        fp = fp.offset(1);
                        *c2rust_fresh7 = c;
                        done = 1 as ::core::ffi::c_uint;
                    }
                    100 | 111 | 120 | 88 => {
                        let c2rust_fresh8 = fp;
                        fp = fp.offset(1);
                        *c2rust_fresh8 = 'l' as ::core::ffi::c_char;
                        let c2rust_fresh9 = fp;
                        fp = fp.offset(1);
                        *c2rust_fresh9 = c;
                        done = 1 as ::core::ffi::c_uint;
                    }
                    35 | 32 => {
                        let c2rust_fresh10 = fp;
                        fp = fp.offset(1);
                        *c2rust_fresh10 = c;
                    }
                    46 => {
                        let c2rust_fresh11 = fp;
                        fp = fp.offset(1);
                        *c2rust_fresh11 = c;
                        if dot == 0 as ::core::ffi::c_uint {
                            dot = 1 as ::core::ffi::c_uint;
                            width = val as size_t;
                        } else {
                            done = 2 as ::core::ffi::c_uint;
                        }
                        val = 0 as ::core::ffi::c_long;
                    }
                    58 => {
                        minus = 1 as ::core::ffi::c_uint;
                    }
                    45 => {
                        if minus != 0 {
                            let c2rust_fresh12 = fp;
                            fp = fp.offset(1);
                            *c2rust_fresh12 = c;
                        } else {
                            done = 1 as ::core::ffi::c_uint;
                        }
                    }
                    _ => {
                        if *(*__ctype_b_loc())
                            .offset(c as ::core::ffi::c_uchar as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort
                                as ::core::ffi::c_int
                            != 0
                        {
                            val = val * 10 as ::core::ffi::c_long
                                + (c as ::core::ffi::c_int - '0' as ::core::ffi::c_int)
                                    as ::core::ffi::c_long;
                            if val > 10000 as ::core::ffi::c_long {
                                done = 2 as ::core::ffi::c_uint;
                            } else {
                                let c2rust_fresh13 = fp;
                                fp = fp.offset(1);
                                *c2rust_fresh13 = c;
                            }
                        } else {
                            done = 1 as ::core::ffi::c_uint;
                        }
                    }
                }
                if done == 0 as ::core::ffi::c_uint {
                    let c2rust_fresh14 = str;
                    str = str.offset(1);
                    c = *c2rust_fresh14;
                }
            }
            if done == 2 as ::core::ffi::c_uint {
                fp = (&raw mut fmt_0 as *mut ::core::ffi::c_char)
                    .offset(1 as ::core::ffi::c_int as isize);
                *fp = *str;
                olen = 0 as size_t;
            } else {
                if dot == 0 as ::core::ffi::c_uint {
                    width = val as size_t;
                } else {
                    precision = val as size_t;
                }
                olen = if width > precision { width } else { precision };
            }
            let c2rust_fresh15 = fp;
            fp = fp.offset(1);
            *c2rust_fresh15 = '\0' as ::core::ffi::c_char;
            match c as ::core::ffi::c_int {
                99 => {
                    pop(
                        &raw mut val,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        &raw mut stack,
                    );
                    if !ochar(
                        &raw mut buf,
                        buf_end,
                        val as ::core::ffi::c_uchar as ::core::ffi::c_int,
                    ) {
                        return false_0 as size_t;
                    }
                }
                115 => {
                    pop(
                        ::core::ptr::null_mut::<::core::ffi::c_long>(),
                        &raw mut ostr,
                        &raw mut stack,
                    );
                    if !ostr.is_null() {
                        let mut r: ::core::ffi::c_int = 0;
                        l = strlen(ostr);
                        if l < olen {
                            l = olen;
                        }
                        if (buf_end.offset_from(buf) as size_t) < l.wrapping_add(1 as size_t) {
                            return false_0 as size_t;
                        }
                        r = snprintf(
                            buf,
                            l.wrapping_add(1 as size_t),
                            &raw mut fmt_0 as *mut ::core::ffi::c_char,
                            ostr,
                        );
                        if r != -1 as ::core::ffi::c_int {
                            buf = buf.offset(r as size_t as isize);
                        }
                    }
                }
                108 => {
                    pop(
                        ::core::ptr::null_mut::<::core::ffi::c_long>(),
                        &raw mut ostr,
                        &raw mut stack,
                    );
                    if ostr.is_null() {
                        l = 0 as size_t;
                    } else {
                        l = strlen(ostr);
                    }
                    push(
                        l as ::core::ffi::c_long,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        &raw mut stack,
                    );
                }
                100 | 111 | 120 | 88 => {
                    pop(
                        &raw mut val,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        &raw mut stack,
                    );
                    if onum(
                        &raw mut buf,
                        buf_end,
                        &raw mut fmt_0 as *mut ::core::ffi::c_char,
                        val as ::core::ffi::c_int,
                        olen,
                    ) as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        return 0 as size_t;
                    }
                }
                112 => {
                    if !((*str as ::core::ffi::c_int) < '1' as ::core::ffi::c_int
                        || *str as ::core::ffi::c_int > '9' as ::core::ffi::c_int)
                    {
                        let c2rust_fresh16 = str;
                        str = str.offset(1);
                        l = (*c2rust_fresh16 as ::core::ffi::c_int - '1' as ::core::ffi::c_int)
                            as size_t;
                        if push(
                            (*params.offset(l as isize)).num,
                            (*params.offset(l as isize)).string,
                            &raw mut stack,
                        ) != 0
                        {
                            return 0 as size_t;
                        }
                    }
                }
                80 => {
                    pop(
                        &raw mut val,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        &raw mut stack,
                    );
                    if *str as ::core::ffi::c_int >= 'a' as ::core::ffi::c_int
                        && *str as ::core::ffi::c_int <= 'z' as ::core::ffi::c_int
                    {
                        dnums[(*str as ::core::ffi::c_int - 'a' as ::core::ffi::c_int) as usize] =
                            val;
                    } else if *str as ::core::ffi::c_int >= 'A' as ::core::ffi::c_int
                        && *str as ::core::ffi::c_int <= 'Z' as ::core::ffi::c_int
                    {
                        snums[(*str as ::core::ffi::c_int - 'A' as ::core::ffi::c_int) as usize] =
                            val;
                    }
                }
                103 => {
                    if *str as ::core::ffi::c_int >= 'a' as ::core::ffi::c_int
                        && *str as ::core::ffi::c_int <= 'z' as ::core::ffi::c_int
                    {
                        if push(
                            dnums
                                [(*str as ::core::ffi::c_int - 'a' as ::core::ffi::c_int) as usize],
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            &raw mut stack,
                        ) != 0
                        {
                            return 0 as size_t;
                        }
                    } else if *str as ::core::ffi::c_int >= 'A' as ::core::ffi::c_int
                        && *str as ::core::ffi::c_int <= 'Z' as ::core::ffi::c_int
                    {
                        if push(
                            snums
                                [(*str as ::core::ffi::c_int - 'A' as ::core::ffi::c_int) as usize],
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            &raw mut stack,
                        ) != 0
                        {
                            return 0 as size_t;
                        }
                    }
                }
                105 => {
                    (*params.offset(0 as ::core::ffi::c_int as isize)).num += 1;
                    (*params.offset(1 as ::core::ffi::c_int as isize)).num += 1;
                }
                39 => {
                    let c2rust_fresh17 = str;
                    str = str.offset(1);
                    if push(
                        *c2rust_fresh17 as ::core::ffi::c_uchar as ::core::ffi::c_long,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        &raw mut stack,
                    ) != 0
                    {
                        return 0 as size_t;
                    }
                    while *str as ::core::ffi::c_int != '\0' as ::core::ffi::c_int
                        && *str as ::core::ffi::c_int != '\'' as ::core::ffi::c_int
                    {
                        str = str.offset(1);
                    }
                    if *str as ::core::ffi::c_int == '\'' as ::core::ffi::c_int {
                        str = str.offset(1);
                    }
                }
                123 => {
                    val = 0 as ::core::ffi::c_long;
                    while *(*__ctype_b_loc())
                        .offset(*str as ::core::ffi::c_uchar as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort
                            as ::core::ffi::c_int
                        != 0
                    {
                        val = val * 10 as ::core::ffi::c_long
                            + (*str as ::core::ffi::c_int - '0' as ::core::ffi::c_int)
                                as ::core::ffi::c_long;
                        str = str.offset(1);
                    }
                    if push(
                        val,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        &raw mut stack,
                    ) != 0
                    {
                        return 0 as size_t;
                    }
                    while *str as ::core::ffi::c_int != '\0' as ::core::ffi::c_int
                        && *str as ::core::ffi::c_int != '}' as ::core::ffi::c_int
                    {
                        str = str.offset(1);
                    }
                    if *str as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
                        str = str.offset(1);
                    }
                }
                43 | 45 | 42 | 47 | 109 | 65 | 79 | 38 | 124 | 94 | 61 | 60 | 62 => {
                    pop(
                        &raw mut val,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        &raw mut stack,
                    );
                    pop(
                        &raw mut val2,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        &raw mut stack,
                    );
                    match c as ::core::ffi::c_int {
                        43 => {
                            val = val + val2;
                        }
                        45 => {
                            val = val2 - val;
                        }
                        42 => {
                            val = val * val2;
                        }
                        47 => {
                            val = if val != 0 {
                                val2 / val
                            } else {
                                0 as ::core::ffi::c_long
                            };
                        }
                        109 => {
                            val = if val != 0 {
                                val2 % val
                            } else {
                                0 as ::core::ffi::c_long
                            };
                        }
                        65 => {
                            val = (val != 0 && val2 != 0) as ::core::ffi::c_int
                                as ::core::ffi::c_long;
                        }
                        79 => {
                            val = (val != 0 || val2 != 0) as ::core::ffi::c_int
                                as ::core::ffi::c_long;
                        }
                        38 => {
                            val = val & val2;
                        }
                        124 => {
                            val = val | val2;
                        }
                        94 => {
                            val = val ^ val2;
                        }
                        61 => {
                            val = (val == val2) as ::core::ffi::c_int as ::core::ffi::c_long;
                        }
                        60 => {
                            val = (val2 < val) as ::core::ffi::c_int as ::core::ffi::c_long;
                        }
                        62 => {
                            val = (val2 > val) as ::core::ffi::c_int as ::core::ffi::c_long;
                        }
                        _ => {}
                    }
                    if push(
                        val,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        &raw mut stack,
                    ) != 0
                    {
                        return 0 as size_t;
                    }
                }
                33 | 126 => {
                    pop(
                        &raw mut val,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        &raw mut stack,
                    );
                    match c as ::core::ffi::c_int {
                        33 => {
                            val = (val == 0) as ::core::ffi::c_int as ::core::ffi::c_long;
                        }
                        126 => {
                            val = !val;
                        }
                        _ => {}
                    }
                    if push(
                        val,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        &raw mut stack,
                    ) != 0
                    {
                        return 0 as size_t;
                    }
                }
                63 => {}
                116 => {
                    pop(
                        &raw mut val,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        &raw mut stack,
                    );
                    if val == 0 as ::core::ffi::c_long {
                        l = 0 as size_t;
                        while *str as ::core::ffi::c_int != '\0' as ::core::ffi::c_int {
                            if *str as ::core::ffi::c_int == '%' as ::core::ffi::c_int {
                                str = str.offset(1);
                                if *str as ::core::ffi::c_int == '?' as ::core::ffi::c_int {
                                    l = l.wrapping_add(1);
                                } else if *str as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                                    if l > 0 as size_t {
                                        l = l.wrapping_sub(1);
                                    } else {
                                        str = str.offset(1);
                                        break;
                                    }
                                } else if *str as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                                    && l == 0 as size_t
                                {
                                    str = str.offset(1);
                                    break;
                                }
                            }
                            str = str.offset(1);
                        }
                    }
                }
                101 => {
                    l = 0 as size_t;
                    while *str as ::core::ffi::c_int != '\0' as ::core::ffi::c_int {
                        if *str as ::core::ffi::c_int == '%' as ::core::ffi::c_int {
                            str = str.offset(1);
                            if *str as ::core::ffi::c_int == '?' as ::core::ffi::c_int {
                                l = l.wrapping_add(1);
                            } else if *str as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                                if l > 0 as size_t {
                                    l = l.wrapping_sub(1);
                                } else {
                                    str = str.offset(1);
                                    break;
                                }
                            }
                        }
                        str = str.offset(1);
                    }
                }
                59 | _ => {}
            }
        }
    }
    return buf.offset_from(buf_start) as size_t;
}
static ansi_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: false_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 8 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[H\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[B\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[8m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0;10m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0;10%?%p1%t;7%;%?%p2%t;4%;%?%p3%t;7%;%?%p4%t;5%;%?%p6%t;1%;%?%p7%t;8%;%?%p9%t;11%;m\0"
            .as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
    ],
    f_keys: [
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static ghostty_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[2J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?12l\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[8m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B(B\x1B[m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1l\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1h\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"%?%p9%t\x1B(0%e\x1B(B%;\x1B[0%?%p6%t;1%;%?%p2%t;4%;%?%p1%p3%|%t;7%;%?%p5%t;2%;%?%p7%t;8%;m\0"
            .as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?69h\x1B[%i%p1%d;%p2%ds\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B]2;\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2 q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%d q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[9m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[4:%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
    ],
    keys: [
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOF\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2F\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOH\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2H\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOD\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOC\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1BOP\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOQ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOR\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOS\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4R\0".as_ptr() as *const ::core::ffi::c_char,
    ],
});
static interix_8colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 8 as ::core::ffi::c_int,
    lines: 25 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[2J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[s\x1B[1b\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[0m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2b\x1B[u\r\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[U\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[T\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[S\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1BF^\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1BF$\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1BF1\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BF2\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BF3\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BF4\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BF5\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BF6\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BF7\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BF8\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BF9\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFE\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFF\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFG\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFI\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFJ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFK\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFN\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFO\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFP\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFQ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFR\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFS\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFT\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFU\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFV\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFW\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFY\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFZ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFa\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFb\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFc\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFd\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFe\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFf\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFg\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFh\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFi\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFj\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFk\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFn\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFo\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFp\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFq\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFs\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFt\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFu\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFv\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFw\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFx\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFy\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFz\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static iterm_256colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\x1B[22;0;0t\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[m\x0F\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\x1B[23;0;0t\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1l\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1h\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[0%?%p6%t;1%;%?%p2%t;4%;%?%p1%p3%|%t;7%;%?%p4%t;5%;%?%p5%t;2%;m%?%p9%t^N%e\x0F%;\0"
            .as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]2;\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[9m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[4:%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
    ],
    keys: [
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOF\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2F\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOH\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2H\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOD\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOC\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1BOP\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOQ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOR\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOS\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static linux_16colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 8 as ::core::ffi::c_int,
    lines: -1 as ::core::ffi::c_int,
    columns: -1 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\x1B[?1c\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25h\x1B[?0c\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[2m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[m\x0F\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0;10%?%p1%t;7%;%?%p2%t;4%;%?%p3%t;7%;%?%p4%t;5%;%?%p5%t;2%;%?%p6%t;1%;m%?%p9%t^N%e\x0F%;\0"
            .as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B^I\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"^Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
    ],
    f_keys: [
        b"\x1B[[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[B\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[D\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[E\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34~\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static putty_256colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: -1 as ::core::ffi::c_int,
    columns: -1 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[m\x0F\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1l\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1h\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[0%?%p1%p6%|%t;1%;%?%p2%t;4%;%?%p1%p3%|%t;7%;%?%p4%t;5%;m%?%p9%t^N%e\x0F%;\0".as_ptr()
            as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]0;\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[9m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"^Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOD\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOC\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
    ],
    f_keys: [
        b"\x1B[11~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[12~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[13~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[14~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34~\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static rxvt_256colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[2J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B7\x1B[?47h\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[m\x0F\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2J\x1B[?47l\x1B8\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[0%?%p6%t;1%;%?%p2%t;4%;%?%p1%p3%|%t;7%;%?%p4%t;5%;m%?%p9%t^N%e\x0F%;\0".as_ptr()
            as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3$\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[8~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[8$\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[7~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[7$\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2$\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[d\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[c\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1B[11~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[12~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[13~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[14~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23$\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24$\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[11^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[12^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[13^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[14^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23@\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24@\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static screen_256colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: false_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34h\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[m\x0F\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1l\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1h\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[0%?%p6%t;1%;%?%p1%t;3%;%?%p2%t;4%;%?%p3%t;7%;%?%p4%t;5%;%?%p5%t;2%;m%?%p9%t^N%e\x0F%;\0"
            .as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOD\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOC\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
    ],
    f_keys: [
        b"\x1BOP\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOQ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOR\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOS\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static st_256colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[2J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[8m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1l\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1h\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"%?%p9%t\x1B(0%e\x1B(B%;\x1B[0%?%p6%t;1%;%?%p2%t;4%;%?%p1%p3%|%t;7%;%?%p4%t;5%;%?%p5%t;2%;%?%p7%t;8%;m\0"
            .as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]0;\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2 q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%d q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[9m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]12;%p1%s\x07\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3;5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2F\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2H\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOD\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOC\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1BOP\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOQ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOR\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOS\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4R\0".as_ptr() as *const ::core::ffi::c_char,
    ],
});
static tmux_256colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: false_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34h\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[8m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[m\x0F\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1l\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1h\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[0%?%p6%t;1%;%?%p2%t;4%;%?%p1%p3%|%t;7%;%?%p4%t;5%;%?%p5%t;2%;%?%p7%t;8%;m%?%p9%t^N%e\x0F%;\0"
            .as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]0;\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2 q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%d q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[9m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]12;%p1%s\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B]112\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4:%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
    ],
    keys: [
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2F\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2H\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOD\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOC\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1BOP\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOQ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOR\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOS\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4R\0".as_ptr() as *const ::core::ffi::c_char,
    ],
});
static vte_256colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[2J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\x1B[22;0;0t\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[8m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0m\x0F\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\x1B[23;0;0t\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1l\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1h\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[0%?%p6%t;1%;%?%p2%t;4%;%?%p4%t;5%;%?%p5%t;2%;%?%p7%t;8%;%?%p1%p3%|%t;7%;m%?%p9%t^N%e\x0F%;\0"
            .as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[1 q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%d q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[9m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]12;%p1%s\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B]112\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4:%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
    ],
    keys: [
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOF\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2F\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOH\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2H\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOD\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOC\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1BOP\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOQ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOR\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOS\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4R\0".as_ptr() as *const ::core::ffi::c_char,
    ],
});
static xterm_256colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[2J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?12l\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\x1B[22;0;0t\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[8m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B(B\x1B[m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\x1B[23;0;0t\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1l\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1h\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"%?%p9%t\x1B(0%e\x1B(B%;\x1B[0%?%p6%t;1%;%?%p5%t;2%;%?%p2%t;4%;%?%p1%p3%|%t;7%;%?%p4%t;5%;%?%p7%t;8%;m\0"
            .as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?69h\x1B[%i%p1%d;%p2%ds\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[2 q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%d q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[9m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]12;%p1%s\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B]112\x07\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOE\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOF\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2F\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOH\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2H\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOD\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOC\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1BOP\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOQ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOR\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOS\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4R\0".as_ptr() as *const ::core::ffi::c_char,
    ],
});
static cygwin_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: false_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 8 as ::core::ffi::c_int,
    lines: -1 as ::core::ffi::c_int,
    columns: -1 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[H\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[B\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B7\x1B[?47h\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[8m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[0;10m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2J\x1B[?47l\x1B8\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0;10%?%p1%t;7%;%?%p2%t;4%;%?%p3%t;7%;%?%p6%t;1%;%?%p7%t;8%;%?%p9%t;11%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B];\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"^Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
    ],
    f_keys: [
        b"\x1B[[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[B\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[D\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[E\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34~\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static win32con_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: false_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 8 as ::core::ffi::c_int,
    lines: -1 as ::core::ffi::c_int,
    columns: -1 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[H\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[B\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B7\x1B[?47h\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[0m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2J\x1B[?47l\x1B8\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0%?%p1%t;7%;%?%p3%t;7%;%?%p6%t;1%;m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[0 q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%d q\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[4;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"^Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1B[[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[B\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[D\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[E\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23$\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24$\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[11^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[12^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[13^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[14^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31^\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[32^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23@\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24@\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static conemu_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[2J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[B\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[48;5;%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[38;5;%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0%?%p1%p3%|%t;7%;%?%p2%t;4%;%?%p6%t;1%;m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[2 q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%d q\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOE\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[4;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1B[[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[B\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[D\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[E\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23$\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24$\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[11^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[12^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[13^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[14^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23@\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24@\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static vtpcon_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[2J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[B\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?12l\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[0%?%p1%p3%|%t;7%;%?%p2%t;4%;%?%p6%t;1%;%?%p9%t;9%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]0;\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2 q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%d q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[9m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOE\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[4;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1B[[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[B\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[D\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[E\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23$\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24$\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[11^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[12^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[13^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[14^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23@\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24@\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
