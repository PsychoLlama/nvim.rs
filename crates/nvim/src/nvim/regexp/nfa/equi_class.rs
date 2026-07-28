//! `[[=a=]]`: the equivalence class each base character expands to,
//! emitted straight into the postfix form.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn nfa_emit_equi_class(mut c: ::core::ffi::c_int) {
    match c {
        65 | A_grave | A_acute | A_circumflex | A_virguilla | A_diaeresis | A_ring | 256 | 258
        | 260 | 461 | 478 | 480 | 506 | 512 | 514 | 550 | 570 | 7680 | 7840 | 7842 | 7844
        | 7846 | 7848 | 7850 | 7852 | 7854 | 7856 | 7858 | 7860 | 7862 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh132 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh132 = 'A' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh133 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh133 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh134 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh134 = 0xc0 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh135 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh135 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh136 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh136 = 0xc1 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh137 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh137 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh138 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh138 = 0xc2 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh139 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh139 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh140 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh140 = 0xc3 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh141 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh141 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh142 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh142 = 0xc4 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh143 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh143 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh144 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh144 = 0xc5 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh145 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh145 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh146 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh146 = 0x100 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh147 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh147 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh148 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh148 = 0x102 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh149 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh149 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh150 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh150 = 0x104 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh151 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh151 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh152 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh152 = 0x1cd as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh153 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh153 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh154 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh154 = 0x1de as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh155 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh155 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh156 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh156 = 0x1e0 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh157 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh157 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh158 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh158 = 0x1fa as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh159 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh159 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh160 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh160 = 0x200 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh161 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh161 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh162 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh162 = 0x202 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh163 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh163 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh164 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh164 = 0x226 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh165 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh165 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh166 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh166 = 0x23a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh167 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh167 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh168 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh168 = 0x1e00 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh169 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh169 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh170 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh170 = 0x1ea0 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh171 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh171 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh172 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh172 = 0x1ea2 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh173 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh173 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh174 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh174 = 0x1ea4 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh175 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh175 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh176 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh176 = 0x1ea6 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh177 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh177 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh178 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh178 = 0x1ea8 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh179 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh179 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh180 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh180 = 0x1eaa as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh181 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh181 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh182 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh182 = 0x1eac as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh183 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh183 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh184 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh184 = 0x1eae as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh185 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh185 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh186 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh186 = 0x1eb0 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh187 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh187 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh188 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh188 = 0x1eb2 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh189 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh189 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh190 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh190 = 0x1eb6 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh191 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh191 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh192 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh192 = 0x1eb4 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh193 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh193 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        66 | 385 | 579 | 7682 | 7684 | 7686 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh194 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh194 = 'B' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh195 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh195 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh196 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh196 = 0x181 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh197 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh197 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh198 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh198 = 0x243 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh199 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh199 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh200 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh200 = 0x1e02 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh201 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh201 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh202 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh202 = 0x1e04 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh203 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh203 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh204 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh204 = 0x1e06 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh205 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh205 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        67 | C_cedilla | 262 | 264 | 266 | 268 | 391 | 571 | 7688 | 42898 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh206 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh206 = 'C' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh207 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh207 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh208 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh208 = 0xc7 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh209 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh209 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh210 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh210 = 0x106 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh211 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh211 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh212 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh212 = 0x108 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh213 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh213 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh214 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh214 = 0x10a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh215 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh215 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh216 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh216 = 0x10c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh217 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh217 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh218 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh218 = 0x187 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh219 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh219 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh220 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh220 = 0x23b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh221 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh221 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh222 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh222 = 0x1e08 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh223 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh223 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh224 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh224 = 0xa792 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh225 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh225 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        68 | 270 | 272 | 394 | 7690 | 7692 | 7694 | 7696 | 7698 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh226 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh226 = 'D' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh227 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh227 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh228 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh228 = 0x10e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh229 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh229 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh230 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh230 = 0x110 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh231 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh231 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh232 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh232 = 0x18a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh233 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh233 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh234 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh234 = 0x1e0a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh235 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh235 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh236 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh236 = 0x1e0c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh237 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh237 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh238 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh238 = 0x1e0e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh239 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh239 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh240 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh240 = 0x1e10 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh241 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh241 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh242 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh242 = 0x1e12 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh243 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh243 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        69 | E_grave | E_acute | E_circumflex | E_diaeresis | 274 | 276 | 278 | 280 | 282 | 516
        | 518 | 552 | 582 | 7700 | 7702 | 7704 | 7706 | 7708 | 7864 | 7866 | 7868 | 7870 | 7872
        | 7874 | 7876 | 7878 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh244 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh244 = 'E' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh245 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh245 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh246 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh246 = 0xc8 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh247 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh247 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh248 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh248 = 0xc9 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh249 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh249 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh250 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh250 = 0xca as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh251 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh251 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh252 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh252 = 0xcb as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh253 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh253 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh254 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh254 = 0x112 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh255 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh255 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh256 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh256 = 0x114 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh257 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh257 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh258 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh258 = 0x116 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh259 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh259 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh260 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh260 = 0x118 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh261 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh261 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh262 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh262 = 0x11a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh263 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh263 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh264 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh264 = 0x204 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh265 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh265 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh266 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh266 = 0x206 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh267 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh267 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh268 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh268 = 0x228 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh269 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh269 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh270 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh270 = 0x246 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh271 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh271 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh272 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh272 = 0x1e14 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh273 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh273 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh274 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh274 = 0x1e16 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh275 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh275 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh276 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh276 = 0x1e18 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh277 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh277 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh278 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh278 = 0x1e1a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh279 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh279 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh280 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh280 = 0x1e1c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh281 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh281 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh282 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh282 = 0x1eb8 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh283 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh283 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh284 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh284 = 0x1eba as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh285 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh285 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh286 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh286 = 0x1ebc as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh287 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh287 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh288 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh288 = 0x1ebe as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh289 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh289 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh290 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh290 = 0x1ec0 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh291 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh291 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh292 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh292 = 0x1ec2 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh293 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh293 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh294 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh294 = 0x1ec4 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh295 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh295 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh296 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh296 = 0x1ec6 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh297 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh297 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        70 | 401 | 7710 | 42904 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh298 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh298 = 'F' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh299 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh299 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh300 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh300 = 0x191 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh301 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh301 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh302 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh302 = 0x1e1e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh303 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh303 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh304 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh304 = 0xa798 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh305 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh305 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        71 | 284 | 286 | 288 | 290 | 403 | 484 | 486 | 500 | 7712 | 42912 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh306 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh306 = 'G' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh307 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh307 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh308 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh308 = 0x11c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh309 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh309 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh310 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh310 = 0x11e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh311 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh311 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh312 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh312 = 0x120 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh313 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh313 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh314 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh314 = 0x122 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh315 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh315 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh316 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh316 = 0x193 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh317 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh317 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh318 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh318 = 0x1e4 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh319 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh319 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh320 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh320 = 0x1e6 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh321 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh321 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh322 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh322 = 0x1f4 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh323 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh323 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh324 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh324 = 0x1e20 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh325 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh325 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh326 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh326 = 0xa7a0 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh327 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh327 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        72 | 292 | 294 | 542 | 7714 | 7716 | 7718 | 7720 | 7722 | 11367 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh328 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh328 = 'H' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh329 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh329 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh330 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh330 = 0x124 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh331 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh331 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh332 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh332 = 0x126 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh333 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh333 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh334 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh334 = 0x21e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh335 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh335 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh336 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh336 = 0x1e22 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh337 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh337 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh338 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh338 = 0x1e24 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh339 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh339 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh340 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh340 = 0x1e26 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh341 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh341 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh342 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh342 = 0x1e28 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh343 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh343 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh344 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh344 = 0x1e2a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh345 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh345 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh346 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh346 = 0x2c67 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh347 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh347 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        73 | I_grave | I_acute | I_circumflex | I_diaeresis | 296 | 298 | 300 | 302 | 304 | 407
        | 463 | 520 | 522 | 7724 | 7726 | 7880 | 7882 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh348 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh348 = 'I' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh349 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh349 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh350 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh350 = 0xcc as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh351 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh351 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh352 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh352 = 0xcd as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh353 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh353 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh354 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh354 = 0xce as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh355 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh355 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh356 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh356 = 0xcf as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh357 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh357 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh358 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh358 = 0x128 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh359 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh359 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh360 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh360 = 0x12a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh361 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh361 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh362 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh362 = 0x12c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh363 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh363 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh364 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh364 = 0x12e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh365 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh365 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh366 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh366 = 0x130 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh367 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh367 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh368 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh368 = 0x197 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh369 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh369 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh370 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh370 = 0x1cf as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh371 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh371 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh372 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh372 = 0x208 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh373 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh373 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh374 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh374 = 0x20a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh375 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh375 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh376 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh376 = 0x1e2c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh377 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh377 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh378 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh378 = 0x1e2e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh379 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh379 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh380 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh380 = 0x1ec8 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh381 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh381 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh382 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh382 = 0x1eca as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh383 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh383 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        74 | 308 | 584 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh384 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh384 = 'J' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh385 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh385 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh386 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh386 = 0x134 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh387 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh387 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh388 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh388 = 0x248 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh389 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh389 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        75 | 310 | 408 | 488 | 7728 | 7730 | 7732 | 11369 | 42816 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh390 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh390 = 'K' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh391 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh391 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh392 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh392 = 0x136 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh393 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh393 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh394 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh394 = 0x198 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh395 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh395 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh396 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh396 = 0x1e8 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh397 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh397 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh398 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh398 = 0x1e30 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh399 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh399 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh400 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh400 = 0x1e32 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh401 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh401 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh402 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh402 = 0x1e34 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh403 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh403 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh404 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh404 = 0x2c69 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh405 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh405 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh406 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh406 = 0xa740 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh407 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh407 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        76 | 313 | 315 | 317 | 319 | 321 | 573 | 7734 | 7736 | 7738 | 7740 | 11360 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh408 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh408 = 'L' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh409 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh409 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh410 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh410 = 0x139 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh411 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh411 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh412 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh412 = 0x13b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh413 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh413 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh414 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh414 = 0x13d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh415 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh415 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh416 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh416 = 0x13f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh417 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh417 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh418 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh418 = 0x141 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh419 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh419 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh420 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh420 = 0x23d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh421 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh421 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh422 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh422 = 0x1e36 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh423 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh423 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh424 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh424 = 0x1e38 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh425 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh425 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh426 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh426 = 0x1e3a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh427 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh427 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh428 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh428 = 0x1e3c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh429 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh429 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh430 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh430 = 0x2c60 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh431 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh431 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        77 | 7742 | 7744 | 7746 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh432 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh432 = 'M' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh433 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh433 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh434 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh434 = 0x1e3e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh435 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh435 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh436 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh436 = 0x1e40 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh437 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh437 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh438 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh438 = 0x1e42 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh439 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh439 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        78 | N_virguilla | 323 | 325 | 327 | 504 | 7748 | 7750 | 7752 | 7754 | 42916 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh440 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh440 = 'N' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh441 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh441 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh442 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh442 = 0xd1 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh443 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh443 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh444 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh444 = 0x143 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh445 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh445 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh446 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh446 = 0x145 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh447 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh447 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh448 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh448 = 0x147 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh449 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh449 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh450 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh450 = 0x1f8 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh451 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh451 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh452 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh452 = 0x1e44 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh453 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh453 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh454 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh454 = 0x1e46 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh455 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh455 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh456 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh456 = 0x1e48 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh457 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh457 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh458 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh458 = 0x1e4a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh459 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh459 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh460 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh460 = 0xa7a4 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh461 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh461 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        79 | O_grave | O_acute | O_circumflex | O_virguilla | O_diaeresis | O_slash | 332 | 334
        | 336 | 415 | 416 | 465 | 490 | 492 | 510 | 524 | 526 | 554 | 556 | 558 | 560 | 7756
        | 7758 | 7760 | 7762 | 7884 | 7886 | 7888 | 7890 | 7892 | 7894 | 7896 | 7898 | 7900
        | 7902 | 7904 | 7906 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh462 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh462 = 'O' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh463 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh463 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh464 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh464 = 0xd2 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh465 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh465 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh466 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh466 = 0xd3 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh467 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh467 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh468 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh468 = 0xd4 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh469 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh469 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh470 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh470 = 0xd5 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh471 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh471 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh472 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh472 = 0xd6 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh473 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh473 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh474 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh474 = 0xd8 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh475 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh475 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh476 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh476 = 0x14c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh477 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh477 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh478 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh478 = 0x14e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh479 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh479 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh480 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh480 = 0x150 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh481 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh481 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh482 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh482 = 0x19f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh483 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh483 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh484 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh484 = 0x1a0 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh485 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh485 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh486 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh486 = 0x1d1 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh487 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh487 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh488 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh488 = 0x1ea as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh489 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh489 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh490 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh490 = 0x1ec as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh491 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh491 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh492 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh492 = 0x1fe as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh493 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh493 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh494 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh494 = 0x20c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh495 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh495 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh496 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh496 = 0x20e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh497 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh497 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh498 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh498 = 0x22a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh499 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh499 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh500 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh500 = 0x22c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh501 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh501 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh502 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh502 = 0x22e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh503 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh503 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh504 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh504 = 0x230 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh505 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh505 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh506 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh506 = 0x1e4c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh507 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh507 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh508 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh508 = 0x1e4e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh509 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh509 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh510 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh510 = 0x1e50 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh511 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh511 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh512 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh512 = 0x1e52 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh513 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh513 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh514 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh514 = 0x1ecc as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh515 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh515 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh516 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh516 = 0x1ece as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh517 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh517 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh518 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh518 = 0x1ed0 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh519 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh519 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh520 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh520 = 0x1ed2 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh521 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh521 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh522 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh522 = 0x1ed4 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh523 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh523 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh524 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh524 = 0x1ed6 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh525 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh525 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh526 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh526 = 0x1ed8 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh527 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh527 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh528 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh528 = 0x1eda as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh529 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh529 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh530 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh530 = 0x1edc as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh531 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh531 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh532 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh532 = 0x1ede as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh533 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh533 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh534 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh534 = 0x1ee0 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh535 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh535 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh536 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh536 = 0x1ee2 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh537 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh537 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        80 | 420 | 7764 | 7766 | 11363 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh538 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh538 = 'P' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh539 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh539 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh540 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh540 = 0x1a4 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh541 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh541 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh542 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh542 = 0x1e54 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh543 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh543 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh544 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh544 = 0x1e56 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh545 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh545 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh546 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh546 = 0x2c63 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh547 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh547 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        81 | 586 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh548 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh548 = 'Q' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh549 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh549 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh550 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh550 = 0x24a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh551 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh551 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        82 | 340 | 342 | 344 | 528 | 530 | 588 | 7768 | 7770 | 7772 | 7774 | 11364 | 42918 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh552 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh552 = 'R' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh553 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh553 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh554 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh554 = 0x154 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh555 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh555 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh556 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh556 = 0x156 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh557 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh557 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh558 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh558 = 0x158 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh559 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh559 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh560 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh560 = 0x210 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh561 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh561 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh562 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh562 = 0x212 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh563 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh563 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh564 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh564 = 0x24c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh565 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh565 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh566 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh566 = 0x1e58 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh567 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh567 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh568 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh568 = 0x1e5a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh569 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh569 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh570 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh570 = 0x1e5c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh571 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh571 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh572 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh572 = 0x1e5e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh573 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh573 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh574 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh574 = 0x2c64 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh575 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh575 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh576 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh576 = 0xa7a6 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh577 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh577 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        83 | 346 | 348 | 350 | 352 | 536 | 7776 | 7778 | 7780 | 7782 | 7784 | 11390 | 42920 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh578 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh578 = 'S' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh579 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh579 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh580 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh580 = 0x15a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh581 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh581 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh582 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh582 = 0x15c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh583 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh583 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh584 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh584 = 0x15e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh585 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh585 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh586 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh586 = 0x160 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh587 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh587 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh588 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh588 = 0x218 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh589 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh589 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh590 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh590 = 0x1e60 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh591 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh591 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh592 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh592 = 0x1e62 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh593 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh593 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh594 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh594 = 0x1e64 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh595 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh595 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh596 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh596 = 0x1e66 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh597 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh597 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh598 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh598 = 0x1e68 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh599 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh599 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh600 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh600 = 0x2c7e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh601 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh601 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh602 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh602 = 0xa7a8 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh603 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh603 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        84 | 354 | 356 | 358 | 428 | 430 | 538 | 574 | 7786 | 7788 | 7790 | 7792 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh604 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh604 = 'T' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh605 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh605 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh606 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh606 = 0x162 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh607 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh607 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh608 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh608 = 0x164 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh609 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh609 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh610 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh610 = 0x166 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh611 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh611 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh612 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh612 = 0x1ac as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh613 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh613 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh614 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh614 = 0x1ae as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh615 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh615 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh616 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh616 = 0x23e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh617 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh617 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh618 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh618 = 0x21a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh619 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh619 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh620 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh620 = 0x1e6a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh621 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh621 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh622 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh622 = 0x1e6c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh623 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh623 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh624 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh624 = 0x1e6e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh625 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh625 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh626 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh626 = 0x1e70 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh627 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh627 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        85 | U_grave | U_acute | U_diaeresis | U_circumflex | 360 | 362 | 364 | 366 | 368 | 370
        | 431 | 467 | 469 | 471 | 473 | 475 | 532 | 534 | 580 | 7794 | 7796 | 7798 | 7800
        | 7802 | 7908 | 7910 | 7912 | 7914 | 7916 | 7918 | 7920 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh628 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh628 = 'U' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh629 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh629 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh630 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh630 = 0xd9 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh631 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh631 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh632 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh632 = 0xda as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh633 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh633 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh634 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh634 = 0xdc as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh635 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh635 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh636 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh636 = 0xdb as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh637 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh637 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh638 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh638 = 0x168 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh639 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh639 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh640 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh640 = 0x16a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh641 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh641 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh642 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh642 = 0x16c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh643 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh643 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh644 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh644 = 0x16e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh645 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh645 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh646 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh646 = 0x170 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh647 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh647 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh648 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh648 = 0x172 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh649 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh649 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh650 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh650 = 0x1af as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh651 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh651 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh652 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh652 = 0x1d3 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh653 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh653 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh654 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh654 = 0x1d5 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh655 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh655 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh656 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh656 = 0x1d7 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh657 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh657 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh658 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh658 = 0x1d9 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh659 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh659 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh660 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh660 = 0x1db as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh661 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh661 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh662 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh662 = 0x214 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh663 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh663 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh664 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh664 = 0x216 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh665 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh665 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh666 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh666 = 0x244 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh667 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh667 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh668 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh668 = 0x1e72 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh669 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh669 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh670 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh670 = 0x1e74 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh671 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh671 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh672 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh672 = 0x1e76 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh673 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh673 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh674 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh674 = 0x1e78 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh675 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh675 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh676 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh676 = 0x1e7a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh677 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh677 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh678 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh678 = 0x1ee4 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh679 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh679 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh680 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh680 = 0x1ee6 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh681 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh681 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh682 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh682 = 0x1ee8 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh683 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh683 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh684 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh684 = 0x1eea as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh685 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh685 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh686 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh686 = 0x1eec as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh687 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh687 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh688 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh688 = 0x1eee as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh689 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh689 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh690 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh690 = 0x1ef0 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh691 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh691 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        86 | 434 | 7804 | 7806 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh692 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh692 = 'V' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh693 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh693 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh694 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh694 = 0x1b2 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh695 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh695 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh696 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh696 = 0x1e7c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh697 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh697 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh698 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh698 = 0x1e7e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh699 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh699 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        87 | 372 | 7808 | 7810 | 7812 | 7814 | 7816 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh700 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh700 = 'W' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh701 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh701 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh702 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh702 = 0x174 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh703 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh703 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh704 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh704 = 0x1e80 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh705 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh705 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh706 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh706 = 0x1e82 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh707 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh707 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh708 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh708 = 0x1e84 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh709 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh709 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh710 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh710 = 0x1e86 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh711 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh711 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh712 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh712 = 0x1e88 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh713 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh713 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        88 | 7818 | 7820 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh714 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh714 = 'X' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh715 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh715 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh716 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh716 = 0x1e8a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh717 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh717 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh718 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh718 = 0x1e8c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh719 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh719 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        89 | Y_acute | 374 | 376 | 435 | 562 | 590 | 7822 | 7922 | 7924 | 7926 | 7928 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh720 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh720 = 'Y' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh721 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh721 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh722 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh722 = 0xdd as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh723 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh723 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh724 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh724 = 0x176 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh725 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh725 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh726 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh726 = 0x178 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh727 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh727 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh728 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh728 = 0x1b3 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh729 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh729 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh730 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh730 = 0x232 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh731 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh731 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh732 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh732 = 0x24e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh733 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh733 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh734 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh734 = 0x1e8e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh735 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh735 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh736 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh736 = 0x1ef2 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh737 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh737 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh738 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh738 = 0x1ef4 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh739 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh739 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh740 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh740 = 0x1ef6 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh741 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh741 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh742 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh742 = 0x1ef8 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh743 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh743 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        90 | 377 | 379 | 381 | 437 | 7824 | 7826 | 7828 | 11371 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh744 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh744 = 'Z' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh745 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh745 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh746 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh746 = 0x179 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh747 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh747 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh748 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh748 = 0x17b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh749 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh749 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh750 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh750 = 0x17d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh751 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh751 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh752 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh752 = 0x1b5 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh753 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh753 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh754 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh754 = 0x1e90 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh755 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh755 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh756 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh756 = 0x1e92 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh757 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh757 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh758 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh758 = 0x1e94 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh759 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh759 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh760 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh760 = 0x2c6b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh761 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh761 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        97 | a_grave | a_acute | a_circumflex | a_virguilla | a_diaeresis | a_ring | 257 | 259
        | 261 | 462 | 479 | 481 | 507 | 513 | 515 | 551 | 7567 | 7681 | 7834 | 7841 | 7843
        | 7845 | 7847 | 7849 | 7851 | 7853 | 7855 | 7857 | 7859 | 7861 | 7863 | 11365 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh762 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh762 = 'a' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh763 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh763 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh764 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh764 = 0xe0 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh765 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh765 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh766 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh766 = 0xe1 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh767 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh767 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh768 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh768 = 0xe2 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh769 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh769 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh770 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh770 = 0xe3 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh771 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh771 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh772 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh772 = 0xe4 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh773 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh773 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh774 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh774 = 0xe5 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh775 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh775 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh776 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh776 = 0x101 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh777 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh777 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh778 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh778 = 0x103 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh779 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh779 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh780 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh780 = 0x105 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh781 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh781 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh782 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh782 = 0x1ce as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh783 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh783 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh784 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh784 = 0x1df as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh785 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh785 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh786 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh786 = 0x1e1 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh787 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh787 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh788 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh788 = 0x1fb as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh789 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh789 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh790 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh790 = 0x201 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh791 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh791 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh792 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh792 = 0x203 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh793 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh793 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh794 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh794 = 0x227 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh795 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh795 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh796 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh796 = 0x1d8f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh797 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh797 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh798 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh798 = 0x1e01 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh799 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh799 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh800 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh800 = 0x1e9a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh801 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh801 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh802 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh802 = 0x1ea1 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh803 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh803 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh804 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh804 = 0x1ea3 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh805 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh805 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh806 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh806 = 0x1ea5 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh807 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh807 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh808 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh808 = 0x1ea7 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh809 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh809 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh810 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh810 = 0x1ea9 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh811 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh811 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh812 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh812 = 0x1eab as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh813 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh813 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh814 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh814 = 0x1ead as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh815 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh815 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh816 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh816 = 0x1eaf as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh817 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh817 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh818 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh818 = 0x1eb1 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh819 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh819 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh820 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh820 = 0x1eb3 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh821 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh821 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh822 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh822 = 0x1eb5 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh823 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh823 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh824 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh824 = 0x1eb7 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh825 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh825 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh826 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh826 = 0x2c65 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh827 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh827 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        98 | 384 | 595 | 7532 | 7552 | 7683 | 7685 | 7687 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh828 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh828 = 'b' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh829 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh829 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh830 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh830 = 0x180 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh831 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh831 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh832 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh832 = 0x253 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh833 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh833 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh834 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh834 = 0x1d6c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh835 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh835 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh836 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh836 = 0x1d80 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh837 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh837 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh838 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh838 = 0x1e03 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh839 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh839 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh840 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh840 = 0x1e05 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh841 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh841 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh842 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh842 = 0x1e07 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh843 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh843 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        99 | c_cedilla | 263 | 265 | 267 | 269 | 392 | 572 | 7689 | 42899 | 42900 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh844 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh844 = 'c' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh845 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh845 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh846 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh846 = 0xe7 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh847 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh847 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh848 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh848 = 0x107 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh849 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh849 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh850 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh850 = 0x109 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh851 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh851 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh852 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh852 = 0x10b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh853 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh853 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh854 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh854 = 0x10d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh855 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh855 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh856 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh856 = 0x188 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh857 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh857 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh858 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh858 = 0x23c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh859 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh859 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh860 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh860 = 0x1e09 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh861 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh861 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh862 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh862 = 0xa793 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh863 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh863 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh864 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh864 = 0xa794 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh865 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh865 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        100 | 271 | 273 | 599 | 7533 | 7553 | 7569 | 7691 | 7693 | 7695 | 7697 | 7699 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh866 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh866 = 'd' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh867 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh867 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh868 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh868 = 0x10f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh869 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh869 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh870 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh870 = 0x111 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh871 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh871 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh872 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh872 = 0x257 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh873 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh873 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh874 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh874 = 0x1d6d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh875 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh875 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh876 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh876 = 0x1d81 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh877 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh877 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh878 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh878 = 0x1d91 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh879 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh879 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh880 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh880 = 0x1e0b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh881 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh881 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh882 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh882 = 0x1e0d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh883 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh883 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh884 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh884 = 0x1e0f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh885 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh885 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh886 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh886 = 0x1e11 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh887 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh887 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh888 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh888 = 0x1e13 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh889 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh889 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        101 | e_grave | e_acute | e_circumflex | e_diaeresis | 275 | 277 | 279 | 281 | 283
        | 517 | 519 | 553 | 583 | 7570 | 7701 | 7703 | 7705 | 7707 | 7709 | 7865 | 7867 | 7869
        | 7871 | 7873 | 7875 | 7877 | 7879 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh890 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh890 = 'e' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh891 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh891 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh892 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh892 = 0xe8 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh893 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh893 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh894 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh894 = 0xe9 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh895 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh895 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh896 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh896 = 0xea as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh897 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh897 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh898 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh898 = 0xeb as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh899 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh899 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh900 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh900 = 0x113 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh901 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh901 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh902 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh902 = 0x115 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh903 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh903 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh904 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh904 = 0x117 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh905 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh905 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh906 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh906 = 0x119 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh907 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh907 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh908 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh908 = 0x11b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh909 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh909 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh910 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh910 = 0x205 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh911 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh911 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh912 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh912 = 0x207 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh913 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh913 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh914 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh914 = 0x229 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh915 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh915 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh916 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh916 = 0x247 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh917 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh917 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh918 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh918 = 0x1d92 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh919 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh919 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh920 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh920 = 0x1e15 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh921 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh921 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh922 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh922 = 0x1e17 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh923 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh923 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh924 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh924 = 0x1e19 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh925 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh925 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh926 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh926 = 0x1e1b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh927 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh927 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh928 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh928 = 0x1e1d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh929 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh929 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh930 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh930 = 0x1eb9 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh931 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh931 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh932 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh932 = 0x1ebb as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh933 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh933 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh934 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh934 = 0x1ebd as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh935 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh935 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh936 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh936 = 0x1ebf as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh937 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh937 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh938 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh938 = 0x1ec1 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh939 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh939 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh940 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh940 = 0x1ec3 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh941 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh941 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh942 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh942 = 0x1ec5 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh943 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh943 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh944 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh944 = 0x1ec7 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh945 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh945 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        102 | 402 | 7534 | 7554 | 7711 | 42905 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh946 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh946 = 'f' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh947 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh947 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh948 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh948 = 0x192 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh949 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh949 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh950 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh950 = 0x1d6e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh951 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh951 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh952 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh952 = 0x1d82 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh953 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh953 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh954 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh954 = 0x1e1f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh955 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh955 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh956 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh956 = 0xa799 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh957 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh957 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        103 | 285 | 287 | 289 | 291 | 485 | 487 | 501 | 608 | 7555 | 7713 | 42913 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh958 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh958 = 'g' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh959 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh959 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh960 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh960 = 0x11d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh961 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh961 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh962 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh962 = 0x11f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh963 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh963 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh964 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh964 = 0x121 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh965 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh965 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh966 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh966 = 0x123 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh967 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh967 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh968 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh968 = 0x1e5 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh969 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh969 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh970 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh970 = 0x1e7 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh971 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh971 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh972 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh972 = 0x1f5 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh973 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh973 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh974 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh974 = 0x260 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh975 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh975 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh976 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh976 = 0x1d83 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh977 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh977 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh978 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh978 = 0x1e21 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh979 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh979 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh980 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh980 = 0xa7a1 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh981 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh981 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        104 | 293 | 295 | 543 | 7715 | 7717 | 7719 | 7721 | 7723 | 7830 | 11368 | 42901 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh982 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh982 = 'h' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh983 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh983 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh984 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh984 = 0x125 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh985 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh985 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh986 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh986 = 0x127 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh987 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh987 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh988 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh988 = 0x21f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh989 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh989 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh990 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh990 = 0x1e23 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh991 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh991 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh992 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh992 = 0x1e25 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh993 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh993 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh994 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh994 = 0x1e27 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh995 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh995 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh996 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh996 = 0x1e29 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh997 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh997 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh998 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh998 = 0x1e2b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh999 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh999 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1000 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1000 = 0x1e96 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1001 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1001 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1002 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1002 = 0x2c68 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1003 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1003 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1004 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1004 = 0xa795 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1005 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1005 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        105 | i_grave | i_acute | i_circumflex | i_diaeresis | 297 | 299 | 301 | 303 | 464
        | 521 | 523 | 616 | 7574 | 7725 | 7727 | 7881 | 7883 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1006 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1006 = 'i' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1007 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1007 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1008 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1008 = 0xec as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1009 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1009 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1010 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1010 = 0xed as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1011 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1011 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1012 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1012 = 0xee as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1013 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1013 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1014 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1014 = 0xef as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1015 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1015 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1016 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1016 = 0x129 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1017 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1017 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1018 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1018 = 0x12b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1019 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1019 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1020 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1020 = 0x12d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1021 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1021 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1022 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1022 = 0x12f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1023 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1023 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1024 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1024 = 0x1d0 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1025 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1025 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1026 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1026 = 0x209 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1027 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1027 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1028 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1028 = 0x20b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1029 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1029 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1030 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1030 = 0x268 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1031 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1031 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1032 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1032 = 0x1d96 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1033 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1033 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1034 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1034 = 0x1e2d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1035 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1035 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1036 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1036 = 0x1e2f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1037 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1037 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1038 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1038 = 0x1ec9 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1039 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1039 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1040 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1040 = 0x1ecb as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1041 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1041 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1042 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1042 = 0x1ecb as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1043 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1043 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        106 | 309 | 496 | 585 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1044 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1044 = 'j' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1045 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1045 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1046 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1046 = 0x135 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1047 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1047 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1048 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1048 = 0x1f0 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1049 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1049 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1050 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1050 = 0x249 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1051 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1051 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        107 | 311 | 409 | 489 | 7556 | 7729 | 7731 | 7733 | 11370 | 42817 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1052 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1052 = 'k' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1053 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1053 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1054 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1054 = 0x137 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1055 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1055 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1056 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1056 = 0x199 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1057 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1057 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1058 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1058 = 0x1e9 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1059 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1059 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1060 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1060 = 0x1d84 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1061 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1061 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1062 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1062 = 0x1e31 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1063 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1063 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1064 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1064 = 0x1e33 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1065 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1065 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1066 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1066 = 0x1e35 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1067 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1067 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1068 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1068 = 0x2c6a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1069 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1069 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1070 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1070 = 0xa741 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1071 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1071 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        108 | 314 | 316 | 318 | 320 | 322 | 410 | 7735 | 7737 | 7739 | 7741 | 11361 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1072 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1072 = 'l' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1073 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1073 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1074 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1074 = 0x13a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1075 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1075 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1076 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1076 = 0x13c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1077 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1077 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1078 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1078 = 0x13e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1079 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1079 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1080 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1080 = 0x140 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1081 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1081 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1082 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1082 = 0x142 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1083 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1083 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1084 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1084 = 0x19a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1085 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1085 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1086 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1086 = 0x1e37 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1087 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1087 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1088 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1088 = 0x1e39 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1089 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1089 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1090 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1090 = 0x1e3b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1091 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1091 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1092 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1092 = 0x1e3d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1093 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1093 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1094 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1094 = 0x2c61 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1095 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1095 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        109 | 7535 | 7743 | 7745 | 7747 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1096 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1096 = 'm' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1097 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1097 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1098 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1098 = 0x1d6f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1099 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1099 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1100 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1100 = 0x1e3f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1101 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1101 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1102 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1102 = 0x1e41 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1103 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1103 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1104 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1104 = 0x1e43 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1105 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1105 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        110 | n_virguilla | 324 | 326 | 328 | 329 | 505 | 7536 | 7559 | 7749 | 7751 | 7753
        | 7755 | 42917 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1106 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1106 = 'n' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1107 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1107 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1108 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1108 = 0xf1 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1109 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1109 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1110 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1110 = 0x144 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1111 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1111 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1112 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1112 = 0x146 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1113 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1113 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1114 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1114 = 0x148 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1115 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1115 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1116 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1116 = 0x149 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1117 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1117 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1118 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1118 = 0x1f9 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1119 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1119 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1120 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1120 = 0x1d70 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1121 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1121 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1122 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1122 = 0x1d87 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1123 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1123 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1124 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1124 = 0x1e45 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1125 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1125 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1126 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1126 = 0x1e47 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1127 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1127 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1128 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1128 = 0x1e49 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1129 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1129 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1130 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1130 = 0x1e4b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1131 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1131 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1132 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1132 = 0xa7a5 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1133 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1133 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        111 | o_grave | o_acute | o_circumflex | o_virguilla | o_diaeresis | o_slash | 333
        | 335 | 337 | 417 | 466 | 491 | 493 | 511 | 525 | 527 | 555 | 557 | 559 | 561 | 629
        | 7757 | 7759 | 7761 | 7763 | 7885 | 7887 | 7889 | 7891 | 7893 | 7895 | 7897 | 7899
        | 7901 | 7903 | 7905 | 7907 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1134 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1134 = 'o' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1135 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1135 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1136 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1136 = 0xf2 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1137 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1137 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1138 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1138 = 0xf3 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1139 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1139 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1140 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1140 = 0xf4 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1141 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1141 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1142 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1142 = 0xf5 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1143 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1143 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1144 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1144 = 0xf6 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1145 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1145 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1146 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1146 = 0xf8 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1147 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1147 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1148 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1148 = 0x14d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1149 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1149 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1150 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1150 = 0x14f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1151 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1151 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1152 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1152 = 0x151 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1153 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1153 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1154 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1154 = 0x1a1 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1155 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1155 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1156 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1156 = 0x1d2 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1157 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1157 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1158 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1158 = 0x1eb as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1159 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1159 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1160 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1160 = 0x1ed as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1161 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1161 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1162 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1162 = 0x1ff as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1163 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1163 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1164 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1164 = 0x20d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1165 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1165 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1166 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1166 = 0x20f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1167 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1167 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1168 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1168 = 0x22b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1169 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1169 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1170 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1170 = 0x22d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1171 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1171 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1172 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1172 = 0x22f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1173 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1173 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1174 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1174 = 0x231 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1175 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1175 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1176 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1176 = 0x275 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1177 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1177 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1178 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1178 = 0x1e4d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1179 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1179 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1180 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1180 = 0x1e4f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1181 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1181 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1182 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1182 = 0x1e51 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1183 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1183 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1184 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1184 = 0x1e53 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1185 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1185 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1186 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1186 = 0x1ecd as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1187 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1187 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1188 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1188 = 0x1ecf as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1189 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1189 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1190 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1190 = 0x1ed1 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1191 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1191 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1192 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1192 = 0x1ed3 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1193 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1193 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1194 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1194 = 0x1ed5 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1195 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1195 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1196 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1196 = 0x1ed7 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1197 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1197 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1198 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1198 = 0x1ed9 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1199 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1199 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1200 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1200 = 0x1edb as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1201 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1201 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1202 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1202 = 0x1edd as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1203 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1203 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1204 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1204 = 0x1edf as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1205 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1205 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1206 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1206 = 0x1ee1 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1207 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1207 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1208 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1208 = 0x1ee3 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1209 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1209 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        112 | 421 | 7537 | 7549 | 7560 | 7765 | 7767 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1210 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1210 = 'p' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1211 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1211 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1212 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1212 = 0x1a5 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1213 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1213 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1214 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1214 = 0x1d71 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1215 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1215 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1216 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1216 = 0x1d7d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1217 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1217 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1218 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1218 = 0x1d88 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1219 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1219 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1220 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1220 = 0x1e55 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1221 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1221 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1222 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1222 = 0x1e57 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1223 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1223 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        113 | 587 | 672 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1224 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1224 = 'q' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1225 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1225 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1226 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1226 = 0x24b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1227 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1227 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1228 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1228 = 0x2a0 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1229 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1229 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        114 | 341 | 343 | 345 | 529 | 531 | 589 | 637 | 7538 | 7539 | 7561 | 7769 | 7771 | 7773
        | 7775 | 42919 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1230 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1230 = 'r' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1231 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1231 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1232 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1232 = 0x155 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1233 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1233 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1234 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1234 = 0x157 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1235 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1235 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1236 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1236 = 0x159 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1237 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1237 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1238 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1238 = 0x211 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1239 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1239 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1240 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1240 = 0x213 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1241 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1241 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1242 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1242 = 0x24d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1243 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1243 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1244 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1244 = 0x27d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1245 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1245 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1246 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1246 = 0x1d72 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1247 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1247 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1248 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1248 = 0x1d73 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1249 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1249 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1250 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1250 = 0x1d89 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1251 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1251 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1252 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1252 = 0x1e59 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1253 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1253 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1254 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1254 = 0x1e5b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1255 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1255 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1256 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1256 = 0x1e5d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1257 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1257 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1258 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1258 = 0x1e5f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1259 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1259 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1260 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1260 = 0xa7a7 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1261 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1261 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        115 | 347 | 349 | 351 | 353 | 537 | 575 | 7540 | 7562 | 7777 | 7779 | 7781 | 7783
        | 7785 | 42921 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1262 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1262 = 's' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1263 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1263 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1264 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1264 = 0x15b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1265 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1265 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1266 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1266 = 0x15d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1267 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1267 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1268 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1268 = 0x15f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1269 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1269 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1270 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1270 = 0x161 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1271 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1271 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1272 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1272 = 0x219 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1273 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1273 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1274 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1274 = 0x23f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1275 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1275 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1276 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1276 = 0x1d74 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1277 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1277 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1278 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1278 = 0x1d8a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1279 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1279 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1280 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1280 = 0x1e61 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1281 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1281 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1282 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1282 = 0x1e63 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1283 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1283 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1284 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1284 = 0x1e65 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1285 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1285 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1286 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1286 = 0x1e67 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1287 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1287 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1288 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1288 = 0x1e69 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1289 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1289 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1290 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1290 = 0xa7a9 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1291 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1291 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        116 | 355 | 357 | 359 | 427 | 429 | 539 | 648 | 7541 | 7787 | 7789 | 7791 | 7793 | 7831
        | 11366 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1292 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1292 = 't' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1293 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1293 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1294 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1294 = 0x163 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1295 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1295 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1296 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1296 = 0x165 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1297 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1297 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1298 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1298 = 0x167 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1299 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1299 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1300 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1300 = 0x1ab as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1301 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1301 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1302 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1302 = 0x1ad as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1303 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1303 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1304 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1304 = 0x21b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1305 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1305 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1306 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1306 = 0x288 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1307 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1307 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1308 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1308 = 0x1d75 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1309 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1309 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1310 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1310 = 0x1e6b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1311 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1311 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1312 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1312 = 0x1e6d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1313 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1313 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1314 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1314 = 0x1e6f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1315 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1315 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1316 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1316 = 0x1e71 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1317 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1317 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1318 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1318 = 0x1e97 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1319 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1319 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1320 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1320 = 0x2c66 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1321 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1321 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        117 | u_grave | u_acute | u_circumflex | u_diaeresis | 361 | 363 | 365 | 367 | 369
        | 371 | 432 | 468 | 470 | 472 | 474 | 476 | 533 | 535 | 649 | 7550 | 7577 | 7795 | 7797
        | 7799 | 7801 | 7803 | 7909 | 7911 | 7913 | 7915 | 7917 | 7919 | 7921 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1322 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1322 = 'u' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1323 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1323 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1324 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1324 = 0xf9 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1325 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1325 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1326 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1326 = 0xfa as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1327 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1327 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1328 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1328 = 0xfb as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1329 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1329 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1330 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1330 = 0xfc as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1331 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1331 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1332 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1332 = 0x169 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1333 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1333 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1334 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1334 = 0x16b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1335 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1335 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1336 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1336 = 0x16d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1337 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1337 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1338 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1338 = 0x16f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1339 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1339 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1340 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1340 = 0x171 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1341 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1341 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1342 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1342 = 0x173 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1343 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1343 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1344 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1344 = 0x1d6 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1345 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1345 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1346 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1346 = 0x1d8 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1347 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1347 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1348 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1348 = 0x215 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1349 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1349 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1350 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1350 = 0x217 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1351 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1351 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1352 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1352 = 0x1b0 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1353 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1353 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1354 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1354 = 0x1d4 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1355 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1355 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1356 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1356 = 0x1da as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1357 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1357 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1358 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1358 = 0x1dc as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1359 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1359 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1360 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1360 = 0x289 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1361 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1361 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1362 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1362 = 0x1e73 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1363 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1363 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1364 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1364 = 0x1d7e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1365 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1365 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1366 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1366 = 0x1d99 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1367 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1367 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1368 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1368 = 0x1e75 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1369 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1369 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1370 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1370 = 0x1e77 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1371 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1371 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1372 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1372 = 0x1e79 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1373 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1373 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1374 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1374 = 0x1e7b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1375 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1375 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1376 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1376 = 0x1ee5 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1377 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1377 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1378 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1378 = 0x1ee7 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1379 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1379 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1380 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1380 = 0x1ee9 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1381 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1381 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1382 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1382 = 0x1eeb as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1383 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1383 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1384 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1384 = 0x1eed as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1385 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1385 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1386 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1386 = 0x1eef as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1387 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1387 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1388 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1388 = 0x1ef1 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1389 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1389 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        118 | 651 | 7564 | 7805 | 7807 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1390 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1390 = 'v' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1391 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1391 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1392 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1392 = 0x28b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1393 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1393 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1394 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1394 = 0x1d8c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1395 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1395 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1396 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1396 = 0x1e7d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1397 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1397 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1398 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1398 = 0x1e7f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1399 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1399 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        119 | 373 | 7809 | 7811 | 7813 | 7815 | 7817 | 7832 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1400 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1400 = 'w' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1401 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1401 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1402 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1402 = 0x175 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1403 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1403 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1404 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1404 = 0x1e81 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1405 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1405 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1406 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1406 = 0x1e83 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1407 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1407 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1408 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1408 = 0x1e85 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1409 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1409 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1410 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1410 = 0x1e87 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1411 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1411 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1412 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1412 = 0x1e89 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1413 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1413 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1414 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1414 = 0x1e98 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1415 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1415 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        120 | 7819 | 7821 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1416 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1416 = 'x' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1417 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1417 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1418 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1418 = 0x1e8b as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1419 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1419 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1420 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1420 = 0x1e8d as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1421 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1421 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        121 | y_acute | y_diaeresis | 375 | 436 | 563 | 591 | 7823 | 7833 | 7923 | 7925 | 7927
        | 7929 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1422 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1422 = 'y' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1423 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1423 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1424 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1424 = 0xfd as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1425 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1425 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1426 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1426 = 0xff as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1427 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1427 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1428 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1428 = 0x177 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1429 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1429 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1430 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1430 = 0x1b4 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1431 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1431 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1432 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1432 = 0x233 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1433 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1433 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1434 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1434 = 0x24f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1435 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1435 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1436 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1436 = 0x1e8f as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1437 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1437 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1438 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1438 = 0x1e99 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1439 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1439 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1440 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1440 = 0x1ef3 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1441 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1441 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1442 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1442 = 0x1ef5 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1443 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1443 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1444 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1444 = 0x1ef7 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1445 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1445 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1446 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1446 = 0x1ef9 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1447 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1447 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        122 | 378 | 380 | 382 | 438 | 7542 | 7566 | 7825 | 7827 | 7829 | 11372 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1448 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1448 = 'z' as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1449 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1449 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1450 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1450 = 0x17a as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1451 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1451 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1452 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1452 = 0x17c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1453 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1453 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1454 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1454 = 0x17e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1455 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1455 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1456 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1456 = 0x1b6 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1457 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1457 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1458 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1458 = 0x1d76 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1459 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1459 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1460 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1460 = 0x1d8e as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1461 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1461 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1462 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1462 = 0x1e91 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1463 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1463 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1464 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1464 = 0x1e93 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1465 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1465 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1466 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1466 = 0x1e95 as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1467 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1467 = NFA_CONCAT as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1468 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1468 = 0x2c6c as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh1469 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh1469 = NFA_CONCAT as ::core::ffi::c_int;
            return;
        }
        _ => {}
    }
    if post_ptr.get() >= post_end.get() {
        realloc_post_list();
    }
    let c2rust_fresh1470 = post_ptr.get();
    post_ptr.set((*post_ptr.ptr()).offset(1));
    *c2rust_fresh1470 = c;
    if post_ptr.get() >= post_end.get() {
        realloc_post_list();
    }
    let c2rust_fresh1471 = post_ptr.get();
    post_ptr.set((*post_ptr.ptr()).offset(1));
    *c2rust_fresh1471 = NFA_CONCAT as ::core::ffi::c_int;
}
