use crate::third_crate::wgsl_components::{
    WgslArray, WgslConstAssignment, WgslFunction, WgslOutputArray, WgslShaderModuleUserPortion,
    WgslType,
};
use proc_macro::Span;
use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Item, ItemConst, ItemFn, ItemMod, ItemStruct, spanned::Spanned};
use syn::{ItemUse, UseTree, Visibility};

pub fn parse_shader_module(module: &ItemMod) -> WgslShaderModuleUserPortion {
    let content = match &module.content {
        Some((_, items)) => items,
        None => abort!(module.ident.span(), "Shader module must have a body"),
    };

    let mut found_valid_use = false;
    for item in content.iter() {
        if let Item::Use(use_stmt) = item {
            if !is_valid_use_statement(use_stmt, "wgsl_in_rust_helpers") {
                abort!(
                    use_stmt.span(),
                    "You cannot export anything into a wgsl module, except for the single helper module this library provides."
                );
            }
            if found_valid_use {
                abort!(
                    use_stmt.span(),
                    "Only one helper module import is allowed in a wgsl module."
                );
            }
            found_valid_use = true;
        }
    }
    // Filter out use statements and proceed with remaining items
    let filtered_content: Vec<_> = content
        .iter()
        .filter(|item| !matches!(item, Item::Use(_)))
        .collect();

    let mut user_portion = WgslShaderModuleUserPortion {
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

    for item in filtered_content {
        match item {
            Item::Const(constant) => {
                user_portion
                    .static_consts
                    .push(parse_const_assignment(constant));
            }
            Item::Struct(structure) => {
                if has_attribute(&structure.attrs, "uniform") {
                    user_portion
                        .uniforms
                        .push(parse_struct_as_wgsl_type(structure));
                } else if has_attribute(&structure.attrs, "vec_output") {
                    user_portion
                        .output_arrays
                        .push(parse_struct_as_output_array(structure, &module));
                } else {
                    user_portion
                        .helper_types
                        .push(parse_struct_as_wgsl_type(structure));
                }
            }
            Item::Type(type_alias) => {
                if has_attribute(&type_alias.attrs, "vec_input") {
                    user_portion
                        .input_arrays
                        .push(parse_type_as_input_array(type_alias));
                } else {
                    user_portion
                        .helper_types
                        .push(parse_type_as_alias(type_alias));
                }
            }
            Item::Fn(function) => {
                if function.sig.ident == "main" {
                    if user_portion.main_function.is_some() {
                        abort!(function.sig.ident.span(), "Multiple main functions found");
                    }
                    user_portion.main_function = Some(parse_main_function(function));
                } else {
                    user_portion
                        .helper_functions
                        .push(parse_helper_function(function));
                }
            }
            _ => abort!(item.span(), "Unsupported item type in shader module"),
        }
    }

    if user_portion.main_function.is_none() {
        abort!(module.span(), "No main function found in shader module");
    }

    user_portion
}

fn is_valid_use_statement(use_stmt: &ItemUse, valid_path_segment: &str) -> bool {
    fn check_path_for_helper(path: &UseTree, valid_path_segment: &str) -> bool {
        match path {
            UseTree::Name(name) => name.ident == valid_path_segment,
            UseTree::Path(path) if path.ident == valid_path_segment => true,
            UseTree::Path(path) => check_path_for_helper(&*path.tree, valid_path_segment),
            _ => false,
        }
    }

    let mut current = &use_stmt.tree;
    loop {
        match current {
            syn::UseTree::Path(path) => {
                if check_path_for_helper(&*path.tree, valid_path_segment) {
                    return true;
                }
                current = &path.tree;
            }
            syn::UseTree::Glob(..) => {
                // Only return true if parent path contained wgsl_in_rust_helpers
                return match &use_stmt.tree {
                    syn::UseTree::Path(path) => {
                        check_path_for_helper(&path.tree, valid_path_segment)
                    }
                    _ => false,
                };
            }
            _ => return false,
        }
    }
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

fn parse_const_assignment(constant: &ItemConst) -> WgslConstAssignment {
    WgslConstAssignment {
        assigner_keyword: "const".to_string(),
        var_name: constant.ident.to_string(),
        var_type: WgslType {
            name: format_type(&constant.ty),
            wgsl: convert_type_to_wgsl(&constant.ty),
        },
        value: format_expr(&constant.expr),
    }
}

fn parse_struct_as_wgsl_type(structure: &ItemStruct) -> WgslType {
    // Implementation to convert struct to WGSL type
    WgslType {
        name: structure.ident.to_string(),
        wgsl: format_struct_as_wgsl(structure),
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
            wgsl: convert_type_to_wgsl(&type_alias.ty),
        },
        length: 0, //todo This should be configurable
    }
}
fn parse_type_as_alias(type_alias: &syn::ItemType) -> WgslType {
    WgslType {
        name: type_alias.ident.to_string(),
        wgsl: convert_type_to_wgsl(&type_alias.ty),
    }
}

fn parse_main_function(function: &ItemFn) -> WgslFunction {
    validate_main_function(function);
    WgslFunction {
        name: "main".to_string(),
        wgsl_definition: format_function_as_wgsl(function),
    }
}

fn parse_helper_function(function: &ItemFn) -> WgslFunction {
    WgslFunction {
        name: function.sig.ident.to_string(),
        wgsl_definition: format_function_as_wgsl(function),
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

// Helper functions for type conversion and formatting
fn format_type(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            let last_segment = type_path
                .path
                .segments
                .last()
                .expect("Type path should have at least one segment");
            last_segment.ident.to_string()
        }
        syn::Type::Array(array) => {
            format!(
                "[{}; {}]",
                format_type(&array.elem),
                format_array_len(&array.len)
            )
        }
        _ => abort!(ty.span(), "Unsupported type in format_type"),
    }
}

fn format_array_len(expr: &syn::Expr) -> String {
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Int(int) => int.to_string(),
            _ => abort!(lit.span(), "Array length must be an integer literal"),
        },
        _ => abort!(expr.span(), "Array length must be a constant expression"),
    }
}

fn convert_type_to_wgsl(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            let segment = type_path
                .path
                .segments
                .last()
                .expect("Type path should have at least one segment");
            match segment.ident.to_string().as_str() {
                "f32" => "f32".to_string(),
                "i32" => "i32".to_string(),
                "u32" => "u32".to_string(),
                "bool" => "bool".to_string(),
                "Vec2" => {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            format!("vec2<{}>", convert_type_to_wgsl(inner_type))
                        } else {
                            abort!(segment.span(), "Vec2 requires a type parameter")
                        }
                    } else {
                        abort!(segment.span(), "Vec2 requires a type parameter")
                    }
                }
                "Vec3" => {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            format!("vec3<{}>", convert_type_to_wgsl(inner_type))
                        } else {
                            abort!(segment.span(), "Vec3 requires a type parameter")
                        }
                    } else {
                        abort!(segment.span(), "Vec3 requires a type parameter")
                    }
                }
                "Vec4" => {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            format!("vec4<{}>", convert_type_to_wgsl(inner_type))
                        } else {
                            abort!(segment.span(), "Vec4 requires a type parameter")
                        }
                    } else {
                        abort!(segment.span(), "Vec4 requires a type parameter")
                    }
                }
                name => name.to_string(), // Pass through custom type names
            }
        }
        syn::Type::Array(array) => {
            format!(
                "array<{}, {}>",
                convert_type_to_wgsl(&array.elem),
                format_array_len(&array.len)
            )
        }
        _ => abort!(ty.span(), "Unsupported type in convert_type_to_wgsl"),
    }
}

/// does nothing for now, wgsl binary ops should be the same as rust
fn format_binary_op(op: &syn::BinOp) -> TokenStream {
    op.to_token_stream()
}
/// does nothing for now, wgsl unary ops should be the same as rust
fn format_unary_op(op: &syn::UnOp) -> TokenStream {
    op.to_token_stream()
}

fn format_struct_as_wgsl(structure: &ItemStruct) -> String {
    let struct_name = &structure.ident;
    let fields: Vec<String> = structure
        .fields
        .iter()
        .map(|field| {
            let name = &field
                .ident
                .as_ref()
                .expect("Unnamed struct fields are not supported in WGSL");
            let field_type = convert_type_to_wgsl(&field.ty);
            format!("    {}: {}", name, field_type)
        })
        .collect();

    return format!("struct {} {{\n{}\n}}", struct_name, fields.join(",\n"));
}

fn format_function_as_wgsl(function: &ItemFn) -> String {
    let fn_name = &function.sig.ident;

    // Format parameters
    let params: Vec<String> = function
        .sig
        .inputs
        .iter()
        .map(|param| match param {
            syn::FnArg::Typed(pat_type) => {
                let param_name = match &*pat_type.pat {
                    syn::Pat::Ident(ident) => &ident.ident,
                    _ => abort!(pat_type.span(), "Unsupported parameter pattern"),
                };
                let param_type = convert_type_to_wgsl(&pat_type.ty);
                format!("{}: {}", param_name, param_type)
            }
            _ => abort!(param.span(), "Unsupported parameter type"),
        })
        .collect();

    // Format return type
    let return_type = match &function.sig.output {
        syn::ReturnType::Default => String::new(),
        syn::ReturnType::Type(_, ty) => format!(" -> {}", convert_type_to_wgsl(ty)),
    };

    // Add special attributes for main function
    let attributes = if fn_name == "main" {
        "@compute @workgroup_size(8, 8, 1)\n" // Default workgroup size
    } else {
        ""
    };

    // Format function body (this is a simplified version - needs more work)
    let body = format_block(&function.block);

    format!(
        "{}fn {}({}{}) {{\n{}\n}}",
        attributes,
        fn_name,
        params.join(", "),
        return_type,
        body
    )
}

fn format_block(block: &syn::Block) -> String {
    // This is a basic implementation - will need to be expanded
    let mut lines = Vec::new();
    for stmt in &block.stmts {
        match stmt {
            syn::Stmt::Local(local) => {
                lines.push(format_local_stmt(local));
            }
            syn::Stmt::Expr(expr, semi) => {
                if semi.is_some() {
                    lines.push(format!("{};", format_expr(expr)));
                } else {
                    lines.push(format_expr(expr));
                }
            }
            _ => abort!(stmt.span(), "Unsupported statement type"),
        }
    }
    lines.join("\n")
}

fn format_local_stmt(local: &syn::Local) -> String {
    let var_name = match &local.pat {
        syn::Pat::Ident(ident) => &ident.ident,
        _ => abort!(local.span(), "Unsupported variable pattern"),
    };

    let type_annotation = match &local.pat {
        syn::Pat::Type(ty) => format!(": {}", convert_type_to_wgsl(&ty.ty)),
        _ => String::new(),
    };

    let initializer = if let Some(local_init) = &local.init {
        format!(" = {}", format_expr(local_init.expr.as_ref()))
    } else {
        String::new()
    };

    format!("let {}{}{};", var_name, type_annotation, initializer)
}

// expression types and what to do with them for wgsl in the "format expression" function:
// /// A slice literal expression: `[a, b, c, d]`.
// Array(ExprArray),
//* Not supported in wgsl */
// /// An assignment expression: `a = compute()`.
// Assign(ExprAssign),
//* supported unchanged from rust*/
// /// An async block: `async { ... }`.
// Async(ExprAsync),
//* Not supported in wgsl */
// /// An await expression: `fut.await`.
// Await(ExprAwait),
//* Not supported in wgsl */
// /// A binary operation: `a + b`, `a += b`.
// Binary(ExprBinary),
//* supported, unchanged from rust*/
// /// A blocked scope: `{ ... }`.
// Block(ExprBlock),
//* supported, unchanged from rust*/
// /// A `break`, with an optional label to break and an optional
// /// expression.
// Break(ExprBreak),
//*supported, unchanged from rust */
// /// A function call expression: `invoke(a, b)`.
// Call(ExprCall),
//*supported, unchanged from rust */
// /// A cast expression: `foo as f64`.
// Cast(ExprCast),
//*not supported in wgsl */
// /// A closure expression: `|a, b| a + b`.
// Closure(ExprClosure),
//*not supported in wgsl */
// /// A const block: `const { ... }`.
// Const(ExprConst),
//* not supported in wgsl */
// /// A `continue`, with an optional label.
// Continue(ExprContinue),
// *supported, unchanged from rust */
// /// Access of a named struct field (`obj.k`) or unnamed tuple struct
// /// field (`obj.0`).
// Field(ExprField),
//* supported, unchanged from rust */
// /// A for loop: `for pat in expr { ... }`.
// ForLoop(ExprForLoop),
//* in wgsl, but with javascript style syntax: for (var i = 0; i< 10; i++){} */
// /// An expression contained within invisible delimiters.
// ///
// /// This variant is important for faithfully representing the precedence
// /// of expressions and is related to `None`-delimited spans in a
// /// `TokenStream`.
// Group(ExprGroup),
//* supported, unchanged from rust */
// /// An `if` expression with an optional `else` block: `if expr { ... }
// /// else { ... }`.
// ///
// /// The `else` branch expression may only be an `If` or `Block`
// /// expression, not any of the other types of expression.
// If(ExprIf),
//* supported, unchanged from rust */
// /// A square bracketed indexing expression: `vector[2]`.
// Index(ExprIndex),
//* supported, unchanged from rust */
// /// The inferred value of a const generic argument, denoted `_`.
// Infer(ExprInfer),
//* not supported in wgsl, no generics allowed */
// /// A `let` guard: `let Some(x) = opt`.
// Let(ExprLet),
//* not supported in wgsl, Result/Option types don't exist */
// /// A literal in place of an expression: `1`, `"foo"`.
// Lit(ExprLit),
//* supported, unchanged from rust */
// /// Conditionless loop: `loop { ... }`.
// Loop(ExprLoop),
//* supported in wgsl, but with different syntax: `for (;;) {}` */
// /// A macro invocation expression: `format!("{}", q)`.
// Macro(ExprMacro),
//* not supported in wgsl */
// /// A `match` expression: `match n { Some(n) => {}, None => {} }`.
// Match(ExprMatch),
//* not supported in wgsl */
// /// A method call expression: `x.foo::<T>(a, b)`.
// MethodCall(ExprMethodCall),
//* methods don't exist in wgsl, write standalone functions instead */
// /// A parenthesized expression: `(a + b)`.
// Paren(ExprParen),
//* supported, unchanged from rust */
// /// A path like `std::mem::replace` possibly containing generic
// /// parameters and a qualified self-type.
// ///
// /// A plain identifier like `x` is a path of length 1.
// Path(ExprPath),
//* not supported unless the total path length is just 1 */
// /// A range expression: `1..2`, `1..`, `..2`, `1..=2`, `..=2`.
// Range(ExprRange),
//* not supported in wgsl */
// /// Address-of operation: `&raw const place` or `&raw mut place`.
// RawAddr(ExprRawAddr),
//*not supported in wgsl */
// /// A referencing operation: `&a` or `&mut a`.
// Reference(ExprReference),
/*support pointer types, but this is something for a future version: here is an example of pointers in wgsl:
```
fn my_function(
    /* 'ptr<function,i32,read_write>' is the type of a pointer value that references
       memory for keeping an 'i32' value, using memory locations in the 'function'
       address space.  Here 'i32' is the store type.
       The implied access mode is 'read_write'.
       See "Address Space" section for defaults. */
    ptr_int: ptr<function,i32>,

    // 'ptr<private,array<f32,50>,read_write>' is the type of a pointer value that
    // refers to memory for keeping an array of 50 elements of type 'f32', using
    // memory locations in the 'private' address space.
    // Here the store type is 'array<f32,50>'.
    // The implied access mode is 'read_write'.
    // See the "Address space section for defaults.
    ptr_array: ptr<private, array<f32, 50>>
  ) { }
  ```
*/

// /// An array literal constructed from one repeated element: `[0u8; N]`.
// Repeat(ExprRepeat),
//* not supported in wgsl */
// /// A `return`, with an optional value to be returned.
// Return(ExprReturn),
//* supported, unchanged from rust */
// /// A struct literal expression: `Point { x: 1, y: 1 }`.
// ///
// /// The `rest` provides the value of the remaining fields as in `S { a:
// /// 1, b: 1, ..rest }`.
// Struct(ExprStruct),
//* supported, different syntax. in wgsl it becomes `Point(1,1)` , but we must warn the user that the order that they list the fields in when constructing their struct MUST be the same order that they are listed in when defining the struct type*/
// /// A try-expression: `expr?`.
// Try(ExprTry),
//*not supported in wgsl */
// /// A try block: `try { ... }`.
// TryBlock(ExprTryBlock),
//*not supported in wgsl, no easy way to output errors from the GPU to the CPU */
// /// A tuple expression: `(a, b, c, d)`.
// Tuple(ExprTuple),
//* tuples unsupported in wgsl */
// /// A unary operation: `!x`, `*x`.
// Unary(ExprUnary),
//* have not tested, but appear to be the same as in rust */
// /// An unsafe block: `unsafe { ... }`.
// Unsafe(ExprUnsafe),
//* does not exist in wgsl  */
// /// Tokens in expression position not interpreted by Syn.
// Verbatim(TokenStream),
//* just translate directly for wgsl, but emit a warning to the user we couldn't interpret the stream */
// /// A while loop: `while expr { ... }`.
// While(ExprWhile),
//* supported , same as rust */
// /// A yield expression: `yield expr`.
// Yield(ExprYield),
//* not supported by wgsl */
fn format_expr(expr: &syn::Expr) -> String {
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
                format_expr(&assign.left),
                &assign.eq_token.to_token_stream().to_string(),
                format_expr(&assign.right)
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
                format_expr(&bin.left),
                format_binary_op(&bin.op),
                format_expr(&bin.right)
            )
        }
        syn::Expr::Block(block) => format_block(&block.block),
        syn::Expr::Break(break_expr) => {
            if let Some(expr) = &break_expr.expr {
                format!("break {}", format_expr(expr))
            } else {
                "break".to_string()
            }
        }
        syn::Expr::Call(call) => {
            let args = call
                .args
                .iter()
                .map(format_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", format_expr(&call.func), args)
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
                format_expr(&field.base),
                field.dot_token.to_token_stream().to_string(),
                field.member.to_token_stream().to_string(),
            )
        }
        syn::Expr::ForLoop(for_loop) => {
            // Convert to C-style for loop
            format!(
                "for ({}) {{ {} }}",
                format_expr(&for_loop.expr),
                format_block(&for_loop.body)
            )
        }
        syn::Expr::Group(group) => format_expr(&group.expr),
        syn::Expr::If(if_expr) => {
            let else_branch = if_expr
                .else_branch
                .as_ref()
                .map(|(_, expr)| format!(" else {{ {} }}", format_expr(expr)))
                .unwrap_or_default();

            format!(
                "if ({}) {{ {} }}{}",
                format_expr(&if_expr.cond),
                format_block(&if_expr.then_branch),
                else_branch
            )
        }
        syn::Expr::Index(index) => {
            format!(
                "{}[{}]",
                format_expr(&index.expr),
                format_expr(&index.index)
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
        syn::Expr::Paren(paren) => format!("({})", format_expr(&paren.expr)),
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
            format!("&{}", format_expr(&reference.expr))
        }
        syn::Expr::Return(ret) => {
            if let Some(expr) = &ret.expr {
                format!("return {}", format_expr(expr))
            } else {
                "return".to_string()
            }
        }
        syn::Expr::Struct(struct_expr) => {
            let fields = struct_expr
                .fields
                .iter()
                .map(|field| format_expr(&field.expr))
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
                format_unary_op(&unary.op),
                format_expr(&unary.expr)
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
                format_expr(&while_expr.cond),
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
