#![feature(proc_macro_quote)]
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Block, Data, DeriveInput, Expr, Fields, FnArg, GenericArgument, Ident, ItemFn, Pat,
    PathArguments, Stmt, Type, parse_macro_input,
};

extern crate proc_macro;

// Proc macro implementation for ComputeShader
#[proc_macro_attribute]
pub fn compute_shader(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;

    // Extract function parameters and their types
    let params: Vec<(Ident, Type)> = input
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    Some((pat_ident.ident.clone(), (*pat_type.ty).clone()))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // Generate WGSL code
    let mut wgsl = String::new();

    // Add struct definitions
    for (name, ty) in &params {
        if ty_implements_trait(ty, "ComputeInput") {
            wgsl.push_str(&format!(
                "
                @group(0) @binding({binding_num})
                var<storage, read> {name}: {wgsl_type};
            ",
                binding_num = get_binding_num(name),
                name = name,
                wgsl_type = get_wgsl_type(ty)
            ));
        }
    }

    // Transform the function body into WGSL
    let wgsl_body = transform_rust_to_wgsl(&input.block);

    // Generate the complete WGSL shader
    wgsl.push_str(&format!(
        "
        @compute @workgroup_size(64, 1, 1)
        fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
            {wgsl_body}
        }}
    "
    ));

    // Store the generated WGSL for runtime use
    quote! {
        const WGSL_CODE: &str = #wgsl;

        #input

        impl ComputeShader for #fn_name {
            fn get_wgsl_code() -> &'static str {
                WGSL_CODE
            }
        }
        impl WGSLShader for #fn_name {
            fn wgsl_code() -> String {
                String::from(#wgsl)
            }
        }
    }
    .into()
}

// Helper functions for the proc macro implementation
fn transform_rust_to_wgsl(block: &Block) -> String {
    let mut output = String::new();

    for stmt in &block.stmts {
        match stmt {
            Stmt::Local(local) => {
                // Handle variable declarations
                let pat_ident = if let Pat::Ident(ident) = &local.pat {
                    &ident.ident
                } else {
                    continue;
                };

                if let Some(local_init) = &local.init {
                    output.push_str(&format!(
                        "let {} = {};\n",
                        pat_ident,
                        transform_expr(&local_init.expr)
                    ));
                }
            }
            Stmt::Expr(expr, _) => {
                // Handle expressions
                output.push_str(&format!("{};\n", transform_expr(&expr)));
            }
            _ => continue,
        }
    }

    output
}

fn transform_expr(expr: &Expr) -> String {
    match expr {
        Expr::Binary(binary) => {
            let op_str = match binary.op {
                syn::BinOp::Add(_) => "+",
                syn::BinOp::Sub(_) => "-",
                syn::BinOp::Mul(_) => "*",
                syn::BinOp::Div(_) => "/",
                syn::BinOp::Rem(_) => "%",
                syn::BinOp::And(_) => "&&",
                syn::BinOp::Or(_) => "||",
                syn::BinOp::BitXor(_) => "^",
                syn::BinOp::BitAnd(_) => "&",
                syn::BinOp::BitOr(_) => "|",
                syn::BinOp::Shl(_) => "<<",
                syn::BinOp::Shr(_) => ">>",
                syn::BinOp::Eq(_) => "==",
                syn::BinOp::Lt(_) => "<",
                syn::BinOp::Le(_) => "<=",
                syn::BinOp::Ne(_) => "!=",
                syn::BinOp::Ge(_) => ">=",
                syn::BinOp::Gt(_) => ">",
                syn::BinOp::AddAssign(_) => "+=",
                syn::BinOp::SubAssign(_) => "-=",
                syn::BinOp::MulAssign(_) => "*=",
                syn::BinOp::DivAssign(_) => "/=",
                syn::BinOp::RemAssign(_) => "%=",
                syn::BinOp::BitXorAssign(_) => "^=",
                syn::BinOp::BitAndAssign(_) => "&=",
                syn::BinOp::BitOrAssign(_) => "|=",
                syn::BinOp::ShlAssign(_) => "<<=",
                syn::BinOp::ShrAssign(_) => ">>=",
                _ => "/* unsupported operator */",
            };
            format!(
                "{} {} {}",
                transform_expr(&binary.left),
                op_str,
                transform_expr(&binary.right)
            )
        }
        Expr::Call(call) => {
            let func = transform_expr(&call.func);
            let args: Vec<String> = call.args.iter().map(|arg| transform_expr(arg)).collect();

            // Special case for Vec::push
            if func.ends_with(".push") {
                return format!(
                    "let idx = atomicAdd(&counter, 1u);\nif (idx < {}_length) {{\n    {}[idx] = {};\n}}",
                    get_output_name(&func),
                    get_output_name(&func),
                    args.join(", ")
                );
            }

            format!("{}({})", func, args.join(", "))
        }
        Expr::Index(idx) => {
            format!(
                "{}[{}]",
                transform_expr(&idx.expr),
                transform_expr(&idx.index)
            )
        }
        Expr::Field(field) => {
            let member_str = match &field.member {
                syn::Member::Named(ident) => ident.to_string(),
                syn::Member::Unnamed(index) => index.index.to_string(),
            };
            format!("{}.{}", transform_expr(&field.base), member_str)
        }
        Expr::Lit(lit) => TokenStream::from(lit.to_token_stream()).to_string(),
        Expr::Path(path) => path
            .path
            .segments
            .last()
            .map(|seg| seg.ident.to_string())
            .unwrap_or_default(),
        _ => "/* unsupported expression */".to_string(),
    }
}

fn get_wgsl_type(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            let last_segment = type_path
                .path
                .segments
                .last()
                .expect("Type path should have at least one segment");

            match &last_segment.arguments {
                PathArguments::AngleBracketed(args) => {
                    // Handle generic types
                    let type_name = last_segment.ident.to_string();
                    let generic_args: Vec<String> = args
                        .args
                        .iter()
                        .filter_map(|arg| {
                            if let GenericArgument::Type(ty) = arg {
                                Some(get_wgsl_type(ty))
                            } else {
                                None
                            }
                        })
                        .collect();

                    match type_name.as_str() {
                        "Vec" => format!("array<{}>", generic_args[0]),
                        "Vec2" => format!("vec2<{}>", generic_args[0]),
                        "Vec3" => format!("vec3<{}>", generic_args[0]),
                        "Vec4" => format!("vec4<{}>", generic_args[0]),
                        _ => type_name,
                    }
                }
                PathArguments::None => {
                    // Handle non-generic types
                    match last_segment.ident.to_string().as_str() {
                        "f32" => "f32",
                        "u32" => "u32",
                        "i32" => "i32",
                        "bool" => "bool",
                        custom => custom, // Custom type that should implement WGSLType
                    }
                    .to_string()
                }
                _ => panic!("Unsupported type arguments"),
            }
        }
        Type::Array(array) => {
            format!(
                "array<{}, {}>",
                get_wgsl_type(&array.elem),
                TokenStream::from(array.len.to_token_stream()).to_string()
            )
        }
        Type::Reference(reference) => {
            // References are ignored in WGSL, just get the inner type
            get_wgsl_type(&reference.elem)
        }
        _ => panic!("Unsupported type in WGSL generation"),
    }
}

// Keep track of binding numbers
use std::sync::atomic::{AtomicU32, Ordering};
static BINDING_COUNTER: AtomicU32 = AtomicU32::new(0);

fn get_binding_num(name: &Ident) -> u32 {
    // We could make this more sophisticated by:
    // 1. Using a hash map to ensure consistent numbers across recompilations
    // 2. Grouping related bindings together
    // 3. Reserving certain binding numbers for specific purposes

    BINDING_COUNTER.fetch_add(1, Ordering::SeqCst)
}

// Helper function to reset binding counter between shader compilations
fn reset_binding_counter() {
    BINDING_COUNTER.store(0, Ordering::SeqCst);
}

fn ty_implements_trait(ty: &Type, trait_name: &str) -> bool {
    // In a real implementation, we would use rustc's trait solving capabilities
    // This is a simplified version that checks for our marker attributes
    if let Type::Path(type_path) = ty {
        let type_name = type_path
            .path
            .segments
            .last()
            .map(|seg| seg.ident.to_string())
            .unwrap_or_default();

        // Check if the type has our derive macros
        // In a real implementation, we would check the actual trait bounds
        match trait_name {
            "ComputeInput" => type_name.ends_with("Input") || has_attribute(ty, "ComputeInput"),
            "ComputeOutput" => type_name.ends_with("Output") || has_attribute(ty, "ComputeOutput"),
            _ => false,
        }
    } else {
        false
    }
}

fn has_attribute(ty: &Type, attr_name: &str) -> bool {
    // In a real implementation, this would check the actual attributes
    // of the type using the Rust compiler's APIs
    // For now, we just assume types following our naming convention
    // implement the appropriate traits
    if let Type::Path(type_path) = ty {
        let type_name = type_path
            .path
            .segments
            .last()
            .map(|seg| seg.ident.to_string())
            .unwrap_or_default();

        type_name.contains(attr_name)
    } else {
        false
    }
}

// Helper function to get output buffer name from method call
fn get_output_name(method: &str) -> &str {
    // This is a simplified version - in reality, we'd need to track
    // the actual variable names and their types
    if method.contains("results") {
        "results"
    } else {
        "output"
    }
}

#[proc_macro_derive(ComputeInput)]
pub fn compute_input_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Get the inner type of the struct
    let inner_type = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                // Get the type of the single field
                &fields.unnamed.first().unwrap().ty
            }
            Fields::Named(fields) if fields.named.len() == 1 => {
                // Get the type of the single named field
                &fields.named.first().unwrap().ty
            }
            _ => panic!("ComputeInput can only be derived for structs with a single field"),
        },
        _ => panic!("ComputeInput can only be derived for structs"),
    };

    // Generate the implementation
    let expanded = quote! {
        impl ComputeInput for #name {
            type Inner = #inner_type;

            fn as_slice(&self) -> &[Self::Inner] {
                std::slice::from_ref(&self.0)
            }

            fn from_slice(slice: &[Self::Inner]) -> Vec<Self> {
                slice.iter().map(|item| Self(*item)).collect()
            }
        }

        impl WGSLType for #name {
            const TYPE_NAME: &'static str = <#inner_type as WGSLType>::TYPE_NAME;
            const STORAGE_COMPATIBLE: bool = true;
        }
    };

    // Return the generated implementation
    TokenStream::from(expanded)
}

#[proc_macro_derive(ComputeOutput)]
pub fn compute_output_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl ComputeOutput for #name {
            fn as_slice(&self) -> &[Self] {
                std::slice::from_ref(self)
            }

            fn from_slice(slice: &[Self]) -> Vec<Self> {
                slice.to_vec()
            }
        }

        impl WGSLType for #name {
            const TYPE_NAME: &'static str = stringify!(#name);
            const STORAGE_COMPATIBLE: bool = true;
        }
    };

    TokenStream::from(expanded)
}
