use proc_macro_error::abort;
use quote::ToTokens;
use syn::{Expr, ExprCall, LitFloat, parse_quote, parse2, spanned::Spanned, visit_mut::VisitMut};

use crate::pipeline::allowed_types::WGSL_NATIVE_TYPES;

pub struct ExprToWgslTransformer {}

impl VisitMut for ExprToWgslTransformer {
    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        // First visit nested expressions
        syn::visit_mut::visit_expr_mut(self, expr);
        if let Some(new_expr) = expr_to_wgsl(expr) {
            *expr = new_expr;
        }
    }
}

/// if none then no mutation is needed
pub fn expr_to_wgsl(expr: &syn::Expr) -> Option<Expr> {
    #[allow(unused_variables)]
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            // to handle things like 3.4_f32
            // if the suffix is u32 then write it as a type cast like u32(digits), same for f32, and i32 and f16
            syn::Lit::Float(l) => {
                // must either have no suffix, in which case we do nothing, or have f32 as the suffix
                let suffix = l.suffix();
                if suffix.is_empty() || suffix == "f" {
                    None
                } else if suffix == "f32" {
                    let value = l.base10_digits();
                    let value = LitFloat::new(value, l.span());
                    return Some(parse_quote!(f32(#value)));
                } else {
                    abort!(
                        l.span(),
                        "Unsupported float suffix in WGSL: ".to_owned() + suffix
                    );
                }
            }
            syn::Lit::Int(l) => {
                let suffix = l.suffix();
                if suffix.is_empty() || suffix == "u" || suffix == "i" {
                    None
                } else if suffix == "u32" {
                    let value = l.base10_digits();
                    let value = LitFloat::new(value, l.span());
                    return Some(parse_quote!(u32(#value)));
                } else if suffix == "i32" {
                    let value = l.base10_digits();
                    let value = LitFloat::new(value, l.span());
                    return Some(parse_quote!(i32(#value)));
                } else {
                    abort!(
                        l.span(),
                        "Unsupported integer suffix in WGSL: ".to_owned() + suffix
                    );
                }
            }
            _ => None,
        },
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
            let cast_type = cast.ty.clone();
            let cast_expr = cast.expr.clone();
            match *cast_type {
                syn::Type::Path(path) => {
                    let ident = path.path.segments.first().unwrap().ident.clone();
                    Some(parse_quote! (#ident(#cast_expr)))
                }
                _ => None,
            }
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
                        .find(|t| t == &&path.path.segments[0].ident.to_string());
                    if let Some(m) = matched {
                        if path.path.segments.last().unwrap().ident == "new" {
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
            let expr: syn::ExprCall = parse2::<ExprCall>(s.parse().unwrap()).unwrap();
            // Some(syn::Expr::Verbatim(quote!(#s)))
            Some(Expr::Call(expr))
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
                expr.to_token_stream()
            );
            abort!(expr.span(), message)
        }
    }
}
