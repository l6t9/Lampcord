#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_errors;

use rustc_ast::{Block, ItemKind, StmtKind, UseTreeKind};
use rustc_errors::{Diag, DiagDecorator};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext as _};

dylint_linting::declare_early_lint! {
    /// ### What it does
    ///
    /// Flags `use` declarations written inside a block (a function body, closure or nested
    /// block). Glob imports (`use Enum::*;`) are allowed, since pulling a handful of enum
    /// variants into one function is the usual reason for a local import.
    ///
    /// ### Why is this bad?
    ///
    /// Imports scattered through function bodies hide a file's dependencies, repeat the same
    /// alias per function, and make it easy for two functions to bind one name to different
    /// paths. Module-level imports keep them in one place.
    ///
    /// ### Example
    ///
    /// ```rust
    /// fn count() -> usize {
    ///     use std::collections::HashMap;
    ///     HashMap::<u8, u8>::new().len()
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// fn count() -> usize {
    ///     HashMap::<u8, u8>::new().len()
    /// }
    /// ```
    pub FUNCTION_LOCAL_USE,
    Deny,
    "`use` declaration inside a function body"
}

impl EarlyLintPass for FunctionLocalUse {
    fn check_block(&mut self, cx: &EarlyContext<'_>, block: &Block) {
        for stmt in &block.stmts {
            let StmtKind::Item(item) = &stmt.kind else {
                continue;
            };
            let ItemKind::Use(tree) = &item.kind else {
                continue;
            };
            if item.span.from_expansion() || matches!(tree.kind, UseTreeKind::Glob(_)) {
                continue;
            }
            cx.opt_span_lint(
                FUNCTION_LOCAL_USE,
                Some(item.span),
                DiagDecorator(|diag: &mut Diag<'_, ()>| {
                    diag.primary_message("`use` declaration inside a function body");
                    diag.help("move the import to the top of the module");
                }),
            );
        }
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
}
