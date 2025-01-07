use proc_macro::Span;
use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::ToTokens;
use shared::wgsl_components::{
    WgslArray, WgslConstAssignment, WgslFunction, WgslOutputArray, WgslShaderModuleUserPortion,
    WgslType,
};
use syn::{Item, ItemConst, ItemFn, ItemMod, ItemStruct, spanned::Spanned};
use syn::{ItemUse, UseTree, Visibility};

use super::constants::add_constants;
use super::use_statements::handle_use_statements;

pub fn parse_shader_module(module: &ItemMod) -> WgslShaderModuleUserPortion {
    let content = match &module.content {
        Some((_, items)) => items,
        None => abort!(module.ident.span(), "Shader module must have a body"),
    };
    let content_filtered = handle_use_statements(content, module);
    let mut result = WgslShaderModuleUserPortion {
        module_ident: module.ident.to_string(),
        module_visibility: module.vis.to_token_stream().to_string(),
        static_consts: Vec::new(),
        helper_types: Vec::new(),
        uniforms: Vec::new(),
        input_arrays: Vec::new(),
        output_arrays: Vec::new(),
        helper_functions: Vec::new(),
        main_function: None,
    };
    let mut filtered_module = module.clone();
    filtered_module.content = Some((
        Default::default(),
        content_filtered.into_iter().cloned().collect(),
    ));

    add_constants(&filtered_module, &mut result);
    // covert to wgsl syntax
    // validate no global_id assignment
    // validate main function exist and has right signature
    // extract all the components

    for item in content_filtered {
        match item {
            Item::Struct(structure) => {
                if has_attribute(&structure.attrs, "uniform") {
                    result.uniforms.push(parse_struct_as_wgsl_type(structure));
                } else if has_attribute(&structure.attrs, "vec_output") {
                    result
                        .output_arrays
                        .push(parse_struct_as_output_array(structure, &module));
                } else {
                    result
                        .helper_types
                        .push(parse_struct_as_wgsl_type(structure));
                }
            }
            Item::Type(type_alias) => {
                if has_attribute(&type_alias.attrs, "vec_input") {
                    result
                        .input_arrays
                        .push(parse_type_as_input_array(type_alias));
                } else {
                    result.helper_types.push(parse_type_as_alias(type_alias));
                }
            }
            Item::Fn(function) => {
                if function.sig.ident == "main" {
                    if result.main_function.is_some() {
                        abort!(function.sig.ident.span(), "Multiple main functions found");
                    }
                    result.main_function = Some(parse_main_function(function));
                } else {
                    result
                        .helper_functions
                        .push(parse_helper_function(function));
                }
            }
            _ => abort!(item.span(), "Unsupported item type in shader module"),
        }
    }

    if result.main_function.is_none() {
        abort!(module.span(), "No main function found in shader module");
    }

    result
}

fn has_attribute(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .map(|seg| seg.ident == name)
            .unwrap_or(false)
    })
}

fn parse_struct_as_wgsl_type(structure: &ItemStruct) -> WgslType {
    // Implementation to convert struct to WGSL type
    WgslType {
        name: structure.ident.to_string(),
        wgsl: struct_def_to_wgsl(structure),
    }
}

fn parse_struct_as_output_array(structure: &ItemStruct, module: &syn::ItemMod) -> WgslOutputArray {
    // Implementation to convert struct to output array
    let array = WgslArray {
        type_name: format!("{}_array", structure.ident),
        item_type: parse_struct_as_wgsl_type(structure),
        length: 0, //todo  This should be configurable
    };

    WgslOutputArray {
        arr: array,
        atomic_counter: needs_atomic_counter(structure, module),
    }
}
fn needs_atomic_counter(structure: &ItemStruct, module: &syn::ItemMod) -> bool {
    // Track if we find any push operations for this structure
    let mut found_push = false;
    let struct_name = &structure.ident;

    // Helper function to check if a method call is WgslOutput::push with our struct
    fn is_push_call_for_struct(call: &syn::ExprCall, struct_name: &syn::Ident) -> bool {
        // First check if this is a path expression (like WgslOutput::push)
        let func_path = match &*call.func {
            syn::Expr::Path(path) => path,
            _ => return false,
        };

        // Get the last segment (should be "push")
        let last_segment = match func_path.path.segments.last() {
            Some(seg) if seg.ident == "push" => seg,
            _ => return false,
        };

        // Check the type parameter of push<T>
        match &last_segment.arguments {
            syn::PathArguments::AngleBracketed(args) => match args.args.first() {
                Some(syn::GenericArgument::Type(syn::Type::Path(type_path))) => type_path
                    .path
                    .segments
                    .last()
                    .map(|seg| seg.ident == *struct_name)
                    .unwrap_or(false),
                _ => false,
            },
            _ => false,
        }
    }

    // Get the content of the module
    if let Some((_, items)) = &module.content {
        // Look through all functions in the module
        for item in items {
            if let syn::Item::Fn(function) = item {
                // Visit the function body to look for push calls
                visit_function_body(function, |expr| {
                    if let syn::Expr::Call(call) = expr {
                        if is_push_call_for_struct(&call, struct_name) {
                            found_push = true;
                        }
                    }
                });
            }
        }
    }

    found_push
}

fn visit_function_body<F>(function: &ItemFn, mut visitor: F)
where
    F: FnMut(&syn::Expr),
{
    fn visit_expr(expr: &syn::Expr, visitor: &mut impl FnMut(&syn::Expr)) {
        // Visit this expression
        visitor(expr);

        // Recursively visit sub-expressions
        match expr {
            syn::Expr::Array(e) => e.elems.iter().for_each(|e| visit_expr(e, visitor)),
            syn::Expr::Binary(e) => {
                visit_expr(&e.left, visitor);
                visit_expr(&e.right, visitor);
            }
            syn::Expr::Block(e) => e.block.stmts.iter().for_each(|stmt| match stmt {
                syn::Stmt::Local(local) => {
                    if let Some(local_init) = &local.init {
                        visit_expr(&local_init.expr, visitor);
                    }
                }
                syn::Stmt::Expr(e, _) => visit_expr(e, visitor),
                _ => {}
            }),
            syn::Expr::Call(e) => {
                visit_expr(&e.func, visitor);
                e.args.iter().for_each(|arg| visit_expr(arg, visitor));
            }
            syn::Expr::If(e) => {
                visit_expr(&e.cond, visitor);
                e.then_branch.stmts.iter().for_each(|stmt| match stmt {
                    syn::Stmt::Expr(e, _) => visit_expr(e, visitor),
                    _ => {}
                });
                if let Some((_, else_branch)) = &e.else_branch {
                    visit_expr(else_branch, visitor);
                }
            }
            syn::Expr::While(e) => {
                visit_expr(&e.cond, visitor);
                e.body.stmts.iter().for_each(|stmt| match stmt {
                    syn::Stmt::Expr(e, _) => visit_expr(e, visitor),
                    _ => {}
                });
            }
            syn::Expr::ForLoop(e) => {
                visit_expr(&e.expr, visitor);
                e.body.stmts.iter().for_each(|stmt| match stmt {
                    syn::Stmt::Expr(e, _) => visit_expr(e, visitor),
                    _ => {}
                });
            }
            _ => {}
        }
    }

    // Visit all expressions in the function body
    for stmt in &function.block.stmts {
        match stmt {
            syn::Stmt::Local(local) => {
                if let Some(local_init) = &local.init {
                    visit_expr(&local_init.expr, &mut visitor);
                }
            }
            syn::Stmt::Expr(e, _) => visit_expr(e, &mut visitor),
            _ => {}
        }
    }
}

fn parse_type_as_input_array(type_alias: &syn::ItemType) -> WgslArray {
    WgslArray {
        type_name: type_alias.ident.to_string(),
        item_type: WgslType {
            name: format_type(&type_alias.ty),
            wgsl: type_to_wgsl(&type_alias.ty),
        },
        length: 0, //todo This should be configurable
    }
}
fn parse_type_as_alias(type_alias: &syn::ItemType) -> WgslType {
    WgslType {
        name: type_alias.ident.to_string(),
        wgsl: type_to_wgsl(&type_alias.ty),
    }
}

fn parse_main_function(function: &ItemFn) -> WgslFunction {
    validate_main_function(function);
    WgslFunction {
        name: "main".to_string(),
        wgsl_definition: function_to_wgsl(function),
    }
}

fn parse_helper_function(function: &ItemFn) -> WgslFunction {
    WgslFunction {
        name: function.sig.ident.to_string(),
        wgsl_definition: function_to_wgsl(function),
    }
}

fn validate_main_function(function: &ItemFn) {
    // Check that main has exactly one parameter
    if function.sig.inputs.len() != 1 {
        abort!(
            function.sig.span(),
            "Main function must have exactly one parameter of type WgslGlobalId"
        );
    }

    // Validate the parameter type is WgslGlobalId
    if let syn::FnArg::Typed(pat_type) = &function.sig.inputs[0] {
        if let syn::Type::Path(type_path) = &*pat_type.ty {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident != "WgslGlobalId" {
                    abort!(
                        pat_type.ty.span(),
                        "Main function parameter must be of type WgslGlobalId"
                    );
                }
            }
        }
    }

    // Check return type (should be void/unit)
    if let syn::ReturnType::Type(_, _) = &function.sig.output {
        abort!(
            function.sig.span(),
            "Main function cannot have a return type"
        );
    }

    // Validate no direct assignments to global_id
    validate_no_global_id_assignment(&function.block);
}
