extern crate proc_macro;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::DeriveInput;
use syn::ItemFn;
use syn::parse;
use syn::parse_macro_input;
use syn::parse_str;

#[proc_macro]
pub fn fn_macro_ast_viz_debug(input: TokenStream) -> TokenStream {
    fn_proc_macro_impl(input)
}

// #[proc_macro_derive(MyDerive)]
// pub fn my_derive_proc_macro(input: TokenStream) -> TokenStream {
//     let DeriveInput {
//         ident: struct_name_ident,
//         data,
//         generics,
//         ..
//     } = parse_macro_input!(input as DeriveInput); // Same as: syn::parse(input).unwrap();
//     // 1. Use syn to parse the input tokens into a syntax tree.
//     // 2. Generate new tokens based on the syntax tree. This is additive to the `enum` or
//     //    `struct` that is annotated (it doesn't replace them).
//     // 3. Return the generated tokens.
//     input
// }

// #[proc_macro_attribute]
// pub fn log_entry_and_exit(args: TokenStream, input: TokenStream) -> TokenStream {
//     // 1. Use syn to parse the args & input tokens into a syntax tree.
//     // 2. Generate new tokens based on the syntax tree. This will replace whatever `item` is
//     //    annotated w/ this attribute proc macro.
//     // 3. Return the generated tokens.
//     input
// }

/// https://docs.rs/syn/latest/syn/macro.parse_macro_input.html
fn fn_proc_macro_impl(_input: TokenStream) -> TokenStream {
    let output_token_stream_str = "fn foo() -> u32 { 42 }";
    let output = output_token_stream_str.parse().unwrap();

    let ast_item_fn: ItemFn = parse_str::<ItemFn>(output_token_stream_str).unwrap();
    viz_ast(ast_item_fn);

    output
}
fn viz_ast(ast: ItemFn) {
    // Simply dump the AST to the console.
    let ast_clone = ast.clone();
    // eprintln!("{} => {}", "Debug::ast", ast_clone);

    // Parse AST to dump some items to the console.
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = ast;

    eprintln!(
        "{} ast_item_fn < attrs.len:{}, vis:{}, sig:'{}' stmt: '{}' >",
        "=>",
        &attrs.len().to_string(),
        match vis {
            syn::Visibility::Public(_) => "public",
            // syn::Visibility::Crate(_) => "crate",
            syn::Visibility::Restricted(_) => "restricted",
            syn::Visibility::Inherited => "inherited",
        },
        &sig.ident.to_string(),
        &match block.stmts.first() {
            Some(stmt) => {
                let expr_str = TokenStream::from(stmt.to_token_stream())
                    .to_string()
                    .clone();
                expr_str
            }
            None => "empty".to_string(),
        },
    );
}
