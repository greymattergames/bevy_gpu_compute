fn function_to_wgsl(function: &ItemFn) -> String {
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
                let param_type = type_to_wgsl(&pat_type.ty);
                format!("{}: {}", param_name, param_type)
            }
            _ => abort!(param.span(), "Unsupported parameter type"),
        })
        .collect();

    // Format return type
    let return_type = match &function.sig.output {
        syn::ReturnType::Default => String::new(),
        syn::ReturnType::Type(_, ty) => format!(" -> {}", type_to_wgsl(ty)),
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
