fn validate_no_global_id_assignment(block: &syn::Block) {
    for stmt in &block.stmts {
        match stmt {
            syn::Stmt::Local(local) => {
                if let Some(local_init) = &local.init {
                    check_expr_for_global_id_assignment(local_init.expr.as_ref());
                }
            }
            syn::Stmt::Expr(expr, _) => {
                check_expr_for_global_id_assignment(expr);
            }
            _ => {}
        }
    }
}

fn check_expr_for_global_id_assignment(expr: &syn::Expr) {
    match expr {
        syn::Expr::Assign(assign) => {
            // Check direct assignments to global_id
            if let syn::Expr::Path(path) = &*assign.left {
                if let Some(ident) = path.path.segments.last() {
                    if ident.ident == "global_id" {
                        abort!(assign.span(), "Cannot assign to global_id");
                    }
                }
            }

            // Check field assignments like global_id.x
            if let syn::Expr::Field(field) = &*assign.left {
                if let syn::Expr::Path(path) = &*field.base {
                    if let Some(ident) = path.path.segments.last() {
                        if ident.ident == "global_id" {
                            abort!(assign.span(), "Cannot assign to global_id components");
                        }
                    }
                }
            }

            // Recursively check the right side of the assignment
            check_expr_for_global_id_assignment(&assign.right);
        }
        syn::Expr::Binary(binary) => {
            // Check both sides of binary expressions
            check_expr_for_global_id_assignment(&binary.left);
            check_expr_for_global_id_assignment(&binary.right);
        }
        syn::Expr::Block(block) => {
            // Check expressions in blocks
            for stmt in &block.block.stmts {
                match stmt {
                    syn::Stmt::Local(local) => {
                        if let Some(local_init) = &local.init {
                            check_expr_for_global_id_assignment(&local_init.expr.as_ref());
                        }
                    }
                    syn::Stmt::Expr(e, _) => {
                        check_expr_for_global_id_assignment(e);
                    }
                    _ => {}
                }
            }
        }
        syn::Expr::Call(call) => {
            // Check function arguments
            for arg in &call.args {
                check_expr_for_global_id_assignment(arg);
            }
        }
        syn::Expr::If(if_expr) => {
            // Check condition and both branches
            check_expr_for_global_id_assignment(&if_expr.cond);
            if_expr.then_branch.stmts.iter().for_each(|stmt| {
                if let syn::Stmt::Expr(expr, _) = stmt {
                    check_expr_for_global_id_assignment(expr);
                }
            });
            if let Some((_, else_branch)) = &if_expr.else_branch {
                check_expr_for_global_id_assignment(else_branch);
            }
        }
        syn::Expr::While(while_expr) => {
            // Check while loop condition and body
            check_expr_for_global_id_assignment(&while_expr.cond);
            while_expr.body.stmts.iter().for_each(|stmt| {
                if let syn::Stmt::Expr(expr, _) = stmt {
                    check_expr_for_global_id_assignment(expr);
                }
            });
        }
        syn::Expr::ForLoop(for_loop) => {
            // Check for loop components
            check_expr_for_global_id_assignment(&for_loop.expr);
            for_loop.body.stmts.iter().for_each(|stmt| {
                if let syn::Stmt::Expr(expr, _) = stmt {
                    check_expr_for_global_id_assignment(expr);
                }
            });
        }
        syn::Expr::Index(index) => {
            // Check array indexing expressions
            check_expr_for_global_id_assignment(&index.expr);
            check_expr_for_global_id_assignment(&index.index);
        }
        _ => {}
    }
}
