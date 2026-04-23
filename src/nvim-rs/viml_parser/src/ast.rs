// VimL AST memory management — Rust replacement for viml_pexpr_free_ast().
//
// The C implementation uses a kvec_withinit_t stack.  Rust uses Vec<T>.
// The traversal order is identical: children first, then next, then free.

#![allow(clippy::items_after_statements)]

use std::ffi::c_void;

use crate::expr_types::{ExprAST, ExprASTNode, ExprASTNodeType};
#[cfg(debug_assertions)]
use crate::string_tables::NODE_MAXCHILDREN;

// ---------------------------------------------------------------------------
// Extern "C" memory functions.
// ---------------------------------------------------------------------------

extern "C" {
    fn xfree(ptr: *mut c_void);
}

// ---------------------------------------------------------------------------
// viml_pexpr_free_ast
// ---------------------------------------------------------------------------

/// Free memory occupied by AST.
///
/// Iteratively walks the tree, freeing string node data and then each node.
/// Matches the traversal order of the C implementation exactly.
///
/// # Safety
/// - `ast` must be a valid `ExprAST` whose nodes were allocated with `xmalloc`.
/// - After this call, all pointers into the AST are dangling.
#[unsafe(export_name = "viml_pexpr_free_ast")]
pub unsafe extern "C" fn viml_pexpr_free_ast(ast: ExprAST) {
    let mut stack: Vec<*mut *mut ExprASTNode> = Vec::with_capacity(16);
    // Push address of root so we can null it out when freed.
    let mut root = ast.root;
    stack.push(std::ptr::addr_of_mut!(root));

    while let Some(cur_node_p) = stack.last().copied() {
        let cur_node: *mut ExprASTNode = unsafe { *cur_node_p };

        #[cfg(debug_assertions)]
        {
            // Explicitly check for AST recursiveness (mirrors C's #ifndef NDEBUG).
            let all_but_last = stack.len().saturating_sub(1);
            for p in &stack[..all_but_last] {
                debug_assert_ne!(
                    unsafe { **p },
                    cur_node,
                    "AST is recursive: found the same node at two stack depths"
                );
            }
        }

        if cur_node.is_null() {
            debug_assert_eq!(stack.len(), 1, "NULL node found with non-empty stack");
            stack.pop();
        } else if !unsafe { (*cur_node).children }.is_null() {
            #[cfg(debug_assertions)]
            {
                let typ = unsafe { (*cur_node).typ } as usize;
                let maxchildren = NODE_MAXCHILDREN[typ];
                debug_assert!(maxchildren > 0);
                debug_assert!(maxchildren <= 2);
                let first_child: *mut ExprASTNode = unsafe { (*cur_node).children };
                if maxchildren == 1 {
                    debug_assert!(
                        unsafe { (*first_child).next }.is_null(),
                        "node with maxchildren=1 has >1 children"
                    );
                } else {
                    // maxchildren == 2
                    let second = unsafe { (*first_child).next };
                    debug_assert!(
                        second.is_null() || unsafe { (*second).next }.is_null(),
                        "node with maxchildren=2 has >2 children"
                    );
                }
            }
            // Push address of children pointer so we can NULL it after freeing.
            stack.push(unsafe { std::ptr::addr_of_mut!((*cur_node).children) });
        } else if !unsafe { (*cur_node).next }.is_null() {
            stack.push(unsafe { std::ptr::addr_of_mut!((*cur_node).next) });
        } else {
            // Leaf: free any owned data, then free the node itself.
            stack.pop();

            // Free string value if applicable.
            match unsafe { (*cur_node).typ } {
                ExprASTNodeType::DoubleQuotedString | ExprASTNodeType::SingleQuotedString => {
                    let str_value = unsafe { (*cur_node).data.str_.value };
                    if !str_value.is_null() {
                        unsafe { xfree(str_value.cast::<c_void>()) };
                    }
                }
                _ => {}
            }

            unsafe { xfree(cur_node.cast::<c_void>()) };
            unsafe { *cur_node_p = std::ptr::null_mut() };
        }
    }
}
