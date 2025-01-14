use std::collections::HashMap;

use proc_macro_error::abort;
use quote::{ToTokens, quote};
use syn::{Expr, parse_quote, spanned::Spanned, visit::Visit, visit_mut::VisitMut};

use crate::transformer::allowed_types::WGSL_NATIVE_TYPES;

pub struct ExprToWgslTransformer {
    // key is the rust syntax, value is the wgsl syntax
    pub replacements: HashMap<String, String>,
}

impl<'ast> Visit<'ast> for ExprToWgslTransformer {
    fn visit_expr(&mut self, expr: &syn::Expr) {
        // First visit nested expressions
        syn::visit::visit_expr(self, expr);
        if let Some(new_expr) = expr_to_wgsl(expr) {
            // Instead of direct replacement, use placeholder system
            self.replacements
                .insert(expr.to_token_stream().to_string(), new_expr);
        }
    }
}

/// if none then no mutation is needed
pub fn expr_to_wgsl(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Lit(lit) => None,
        syn::Expr::Array(array) => {
            abort!(array.span(), "Array literals are not supported in WGSL")
        }
        syn::Expr::Assign(assign) => None,
        syn::Expr::Async(async_expr) => {
            abort!(
                async_expr.span(),
                "Async expressions are not supported in WGSL"
            )
        }
        syn::Expr::Await(await_expr) => {
            abort!(
                await_expr.span(),
                "Await expressions are not supported in WGSL"
            )
        }
        syn::Expr::Binary(bin) => None,
        syn::Expr::Block(block) => None,
        syn::Expr::Break(break_expr) => None,
        syn::Expr::Call(call) => None,
        syn::Expr::Cast(cast) => {
            todo!("casts have this syntax in wgsl: `f32(x)`");
        }
        syn::Expr::Closure(closure) => {
            abort!(
                closure.span(),
                "Closure expressions are not supported in WGSL"
            )
        }
        syn::Expr::Const(const_expr) => {
            abort!(const_expr.span(), "Const blocks are not supported in WGSL")
        }
        syn::Expr::Continue(continue_expr) => None,
        syn::Expr::Field(field) => None,
        syn::Expr::ForLoop(for_loop) => {
            abort!(
                for_loop.span(),
                "For loops to wgsl syntax conversion not yet implemented"
            );
            //todo Convert to wgsl style for loop from rust
            // let s = format!(
            // "for ({init}; {cond}; {update}) {{ {body} }}",
            // init = &for_loop.init,
            // cond = &for_loop.cond,
            // update = &for_loop.update,
            // body = &for_loop.body
            // );
            // Some(parse_quote!(#s))
        }
        syn::Expr::Group(group) => None,
        syn::Expr::If(if_expr) => None,
        syn::Expr::Index(index) => None,
        syn::Expr::Infer(_) => {
            abort!(
                expr.span(),
                "Type inference expressions are not supported in WGSL"
            )
        }
        syn::Expr::Let(let_expr) => None,
        syn::Expr::Loop(loop_expr) => {
            abort!(
                loop_expr.span(),
                "Loop expression conversion to wgsl not yet implemented"
            )
            //todo format!("for (;;) {{ {} }}", format_block(&loop_expr.body))
        }
        syn::Expr::Macro(macro_expr) => {
            abort!(
                macro_expr.span(),
                "Macro invocations are not supported in WGSL"
            )
        }
        syn::Expr::Match(match_expr) => {
            abort!(
                match_expr.span(),
                "Match expressions are not supported in WGSL"
            )
        }
        syn::Expr::MethodCall(method_call) => {
            abort!(
                method_call.span(),
                "Method calls are not supported in WGSL, use standalone functions instead"
            )
        }
        syn::Expr::Paren(paren) => None,
        syn::Expr::Path(path) => {
            if path.path.segments.len() > 1 {
                if path.path.segments.len() == 2 {
                    let matched = WGSL_NATIVE_TYPES
                        .iter()
                        .find(|t| **t == path.path.segments[0].ident.to_string());
                    if let Some(m) = matched {
                        if path.path.segments.last().unwrap().ident.to_string() == "new" {
                            // will be handled at a later stage
                            return None;
                        }
                    }
                }

                abort!(
                    path.span(),
                    "Complex paths are not supported in WGSL, only simple identifiers are allowed"
                )
            }
            None
        }
        syn::Expr::Range(range) => {
            abort!(range.span(), "Range expressions are not supported in WGSL")
        }
        syn::Expr::Reference(reference) => {
            if reference.mutability.is_some() {
                abort!(
                    reference.span(),
                    "Mutable references are not supported in WGSL yet"
                )
            }
            None
            // format!("&{}", expr_to_wgsl(&reference.expr))
            // todo still some work to do around converting pointers correctly
        }
        syn::Expr::Return(ret) => None,
        // initialization field order must match the struct definition field order, because we are not able right now to reference the original struct definition to reorder the fields for wgsl
        syn::Expr::Struct(struct_expr) => {
            // Some(parse_quote!(Somethin2gnn ( x: 3 )))
            let fields = struct_expr
                .fields
                .iter()
                .map(|f| f.expr.to_token_stream().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let struct_type_name = if let Some(lp) = struct_expr.path.segments.last() {
                lp
            } else {
                abort!(struct_expr.span(), "Struct path is empty")
            };
            let s = format!("{}({})", &struct_type_name.ident.to_string(), fields);
            // Some(syn::Expr::Verbatim(quote!(#s)))
            Some(s)
        }
        syn::Expr::Try(try_expr) => {
            abort!(try_expr.span(), "Try expressions are not supported in WGSL")
        }
        syn::Expr::TryBlock(try_block) => {
            abort!(try_block.span(), "Try blocks are not supported in WGSL")
        }
        syn::Expr::Tuple(tuple) => {
            abort!(tuple.span(), "Tuple expressions are not supported in WGSL")
        }
        syn::Expr::Unary(unary) => None,
        syn::Expr::Unsafe(unsafe_expr) => {
            abort!(
                unsafe_expr.span(),
                "Unsafe blocks are not supported in WGSL"
            )
        }
        syn::Expr::Verbatim(tokens) => {
            //todo: Emit warning about uninterpreted tokens
            None
        }
        syn::Expr::While(while_expr) => None,
        syn::Expr::Yield(yield_expr) => {
            abort!(
                yield_expr.span(),
                "Yield expressions are not supported in WGSL"
            )
        }
        _ => {
            let message = format!(
                "Unsupported expression type in WGSL: {}",
                expr.to_token_stream().to_string()
            );
            abort!(expr.span(), message)
        }
    }
}
