use syn::visit_mut::VisitMut;
use syn::{Block, Expr, Stmt, parse_quote, visit_mut};

/**
 Rust implicit returns like
   ```rust
   fn foo() -> i32 {
       42
   }
   ```
   must be converted to explicit returns like
       ```rust
       fn foo() -> i32 {
           return 42;
       }
       ```
   for WGSL
* */
pub struct ImplicitToExplicitReturnTransformer;

impl VisitMut for ImplicitToExplicitReturnTransformer {
    // we can identify an implicit return by...
    // must be a Stmt::Expr
    // must NOT be a Stmt::Semi
    // must be the last statement in the block
    // for now we'll say it can be any non semi expression
    fn visit_block_mut(&mut self, block: &mut Block) {
        visit_mut::visit_block_mut(self, block);
        let last_idx = block.stmts.len().checked_sub(1);
        if let Some(idx) = last_idx {
            if let Stmt::Expr(expr, semi) = &block.stmts[idx] {
                if semi.is_none()
                    && !matches!(
                        expr,
                        Expr::If(_)
                            | Expr::Match(_)
                            | Expr::Loop(_)
                            | Expr::ForLoop(_)
                            | Expr::While(_)
                    )
                {
                    // is probably an implicit return
                    block.stmts[idx] = parse_quote!(return #expr;);
                }
            }
        }
    }
    fn visit_item_fn_mut(&mut self, item_fn: &mut syn::ItemFn) {
        visit_mut::visit_item_fn_mut(self, item_fn);
        self.visit_block_mut(&mut item_fn.block);
    }
}
