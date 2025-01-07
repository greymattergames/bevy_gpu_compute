use proc_macro_error::abort;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Item, ItemMod, ItemType, Type, parse2};

pub fn apply_known_rust_to_wgsl_type_transformations(module: &mut ItemMod) -> TokenStream {
    if let Some((brace, ref mut content)) = module.content {
        let mut new_items = Vec::new();
        for item in content.iter() {
            let new_item =
                convert_token_stream_to_item(convert_all_rust_types_in_item_to_wgsl_types(item));
            if let Ok(item) = new_item {
                new_items.push(item);
            }
        }
        module.content = Some((brace.clone(), new_items));
        quote! {
            #module
        }
        .into()
    } else {
        // If the module doesn't have a body, return unchanged
        quote! {
            #module
        }
        .into()
    }
}
pub fn convert_all_rust_types_in_item_to_wgsl_types(item: &syn::Item) -> TokenStream {
    match item {
        syn::Item::Type(type_alias) => {
            // Convert the type alias to a WGSL alias
            let ident = &type_alias.ident;
            let converted_type = convert_rust_type_to_wgsl(&type_alias.ty);
            quote! {
                alias #ident = #converted_type;
            }
        }
        syn::Item::Struct(item_struct) => {
            // Create a mutable copy of the struct to modify
            let mut modified_struct = item_struct.clone();

            // Convert types in struct fields
            for field in &mut modified_struct.fields {
                let new_type = convert_rust_type_to_wgsl(&field.ty);
                field.ty = parse2(new_type).unwrap_or(field.ty.clone());
            }

            quote! { #modified_struct }
        }
        syn::Item::Fn(item_fn) => {
            // Create a mutable copy of the function to modify
            let mut modified_fn = item_fn.clone();

            // Convert return type if it exists
            if let syn::ReturnType::Type(arrow, ty) = &modified_fn.sig.output {
                let new_return_type = convert_rust_type_to_wgsl(ty);
                modified_fn.sig.output = syn::ReturnType::Type(
                    arrow.clone(),
                    Box::new(parse2(new_return_type).unwrap_or(*ty.clone())),
                );
            }

            // Convert parameter types
            for input in &mut modified_fn.sig.inputs {
                if let syn::FnArg::Typed(pat_type) = input {
                    let new_type = convert_rust_type_to_wgsl(&pat_type.ty);
                    pat_type.ty = Box::new(parse2(new_type).unwrap_or(*pat_type.ty.clone()));
                }
            }

            quote! { #modified_fn }
        }
        // For any other item type, return it unchanged as a TokenStream
        other_item => quote! { #other_item },
    }
}

pub fn convert_item_to_token_stream(item: &syn::Item) -> TokenStream {
    quote!(#item)
}

pub fn convert_token_stream_to_item(input: TokenStream) -> Result<syn::Item, syn::Error> {
    parse2::<syn::Item>(input)
}

/// recursive
fn convert_rust_type_to_wgsl(ty: &Type) -> TokenStream {
    match ty {
        Type::Array(_) => rust_array_to_wgsl_array(ty),
        Type::Slice(_) => {
            abort!(
                Span::call_site(),
                "WGSL does not support slices or arrays with dynamic length."
            );
        }
        Type::Path(_) => rust_handle_path_type(ty),
        Type::Group(g) => convert_rust_type_to_wgsl(g.elem.as_ref()),
        Type::Tuple(_) => {
            abort!(
                Span::call_site(),
                "WGSL does not support Tuple types, use arrays instead."
            );
        }
        _ => {
            abort!(Span::call_site(), "Unsupported type");
        }
    }
}

fn rust_handle_path_type(ty: &Type) -> TokenStream {
    if let Type::Path(type_path) = ty {
        let last_segment = type_path
            .path
            .segments
            .last()
            .expect("Type path should have at least one segment");
        return rust_handle_path_segment(last_segment);
    } else {
        abort!(
            Span::call_site(),
            "rust_handle_type_path was given a non-Type::Path type"
        );
    }
}

fn rust_handle_path_segment(segment: &syn::PathSegment) -> TokenStream {
    match segment.ident.to_string().as_str() {
        "f32" => quote!(f32),
        "f64" | "f8" | "u8" | "u16" | "u64" | "u128" | "i8" | "i16" | "i64" | "i128" | "usize" => {
            abort!(
                Span::call_site(),
                "WGSL only supports numeric types f32, f16, i32, and u32.",
            );
        }
        "i32" => quote!(i32),
        "u32" => quote!(u32),
        "bool" => quote!(bool),
        "Vec2" | "Vec3" | "Vec4" => {
            // Handle generic parameters for vector types
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                if let Some(arg) = args.args.first() {
                    if let syn::GenericArgument::Type(type_arg) = arg {
                        let inner_type = convert_rust_type_to_wgsl(type_arg);
                        let vec_type = match segment.ident.to_string().as_str() {
                            "Vec2" => quote!(vec2),
                            "Vec3" => quote!(vec3),
                            "Vec4" => quote!(vec4),
                            _ => unreachable!(),
                        };
                        return quote!(#vec_type<#inner_type>);
                    }
                }
            }
            abort!(
                Span::call_site(),
                "Vector types must specify their element type",
            );
        }
        "Mat2" => quote!(mat2x2),
        "Mat3" => quote!(mat3x3),
        "Mat4" => quote!(mat4x4),
        "Vec" => {
            abort!(Span::call_site(), "WGSL does not support Vec types.");
        }
        other => {
            // do not validate, since if we do then the user cannot pass in custom types which may be valid in the final wgsl module
            quote!(#other)
        }
    }
}

fn rust_array_to_wgsl_array(ty: &Type) -> TokenStream {
    match ty {
        Type::Array(array) => {
            let ty = convert_rust_type_to_wgsl(&array.elem);
            let len = &array.len;
            quote!(array<#ty, #len>)
        }
        _ => {
            abort!(Span::call_site(), "Expected array type");
        }
    }
}
