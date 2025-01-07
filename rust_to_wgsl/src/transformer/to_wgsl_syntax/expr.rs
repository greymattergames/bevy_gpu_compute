use proc_macro_error::abort;
use quote::ToTokens;
use syn::spanned::Spanned;

fn expr_to_wgsl(expr: &syn::Expr) -> String {
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Int(n) => n.to_string(),
            syn::Lit::Float(n) => n.to_string(),
            syn::Lit::Bool(b) => b.value.to_string(),
            _ => abort!(
                lit.span(),
                "Unsupported literal type in constant expression"
            ),
        },
        syn::Expr::Array(array) => {
            abort!(array.span(), "Array literals are not supported in WGSL")
        }
        syn::Expr::Assign(assign) => {
            format!(
                "{} {} {}",
                expr_to_wgsl(&assign.left),
                &assign.eq_token.to_token_stream().to_string(),
                expr_to_wgsl(&assign.right)
            )
        }
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
        syn::Expr::Binary(bin) => {
            format!(
                "({} {} {})",
                expr_to_wgsl(&bin.left),
                binary_op_to_wgsl(&bin.op),
                expr_to_wgsl(&bin.right)
            )
        }
        syn::Expr::Block(block) => format_block(&block.block),
        syn::Expr::Break(break_expr) => {
            if let Some(expr) = &break_expr.expr {
                format!("break {}", expr_to_wgsl(expr))
            } else {
                "break".to_string()
            }
        }
        syn::Expr::Call(call) => {
            let args = call
                .args
                .iter()
                .map(expr_to_wgsl)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", expr_to_wgsl(&call.func), args)
        }
        syn::Expr::Cast(cast) => {
            abort!(cast.span(), "Cast expressions are not supported in WGSL")
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
        syn::Expr::Continue(continue_expr) => "continue".to_string(),
        syn::Expr::Field(field) => {
            format!(
                "{}{}{}",
                expr_to_wgsl(&field.base),
                field.dot_token.to_token_stream().to_string(),
                field.member.to_token_stream().to_string(),
            )
        }
        syn::Expr::ForLoop(for_loop) => {
            // Convert to C-style for loop
            format!(
                "for ({}) {{ {} }}",
                expr_to_wgsl(&for_loop.expr),
                format_block(&for_loop.body)
            )
        }
        syn::Expr::Group(group) => expr_to_wgsl(&group.expr),
        syn::Expr::If(if_expr) => {
            let else_branch = if_expr
                .else_branch
                .as_ref()
                .map(|(_, expr)| format!(" else {{ {} }}", expr_to_wgsl(expr)))
                .unwrap_or_default();

            format!(
                "if ({}) {{ {} }}{}",
                expr_to_wgsl(&if_expr.cond),
                format_block(&if_expr.then_branch),
                else_branch
            )
        }
        syn::Expr::Index(index) => {
            format!(
                "{}[{}]",
                expr_to_wgsl(&index.expr),
                expr_to_wgsl(&index.index)
            )
        }
        syn::Expr::Infer(_) => {
            abort!(
                expr.span(),
                "Type inference expressions are not supported in WGSL"
            )
        }
        syn::Expr::Let(let_expr) => {
            abort!(let_expr.span(), "Let expressions are not supported in WGSL")
        }
        syn::Expr::Loop(loop_expr) => {
            format!("for (;;) {{ {} }}", format_block(&loop_expr.body))
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
        syn::Expr::Paren(paren) => format!("({})", expr_to_wgsl(&paren.expr)),
        syn::Expr::Path(path) => {
            if path.path.segments.len() > 1 {
                abort!(
                    path.span(),
                    "Complex paths are not supported in WGSL, only simple identifiers are allowed"
                )
            }
            path.path
                .segments
                .first()
                .map(|seg| seg.ident.to_string())
                .unwrap_or_default()
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
            format!("&{}", expr_to_wgsl(&reference.expr))
        }
        syn::Expr::Return(ret) => {
            if let Some(expr) = &ret.expr {
                format!("return {}", expr_to_wgsl(expr))
            } else {
                "return".to_string()
            }
        }
        syn::Expr::Struct(struct_expr) => {
            let fields = struct_expr
                .fields
                .iter()
                .map(|field| expr_to_wgsl(&field.expr))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{}({})",
                &struct_expr.path.segments.last().unwrap().ident.to_string(),
                fields
            )
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
        syn::Expr::Unary(unary) => {
            format!(
                "({} {})",
                unary_op_to_wgsl(&unary.op),
                expr_to_wgsl(&unary.expr)
            )
        }
        syn::Expr::Unsafe(unsafe_expr) => {
            abort!(
                unsafe_expr.span(),
                "Unsafe blocks are not supported in WGSL"
            )
        }
        syn::Expr::Verbatim(tokens) => {
            // Emit warning about uninterpreted tokens
            tokens.to_string()
        }
        syn::Expr::While(while_expr) => {
            format!(
                "while ({}) {{ {} }}",
                expr_to_wgsl(&while_expr.cond),
                format_block(&while_expr.body)
            )
        }
        syn::Expr::Yield(yield_expr) => {
            abort!(
                yield_expr.span(),
                "Yield expressions are not supported in WGSL"
            )
        }
        _ => abort!(expr.span(), "Unsupported expression type in WGSL"),
    }
}
