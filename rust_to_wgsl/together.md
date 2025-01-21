# build.rs

```rs
fn main() {
    // Watch shader source files for changes
    println!("cargo:rerun-if-changed=examples/collision_shader.rs");

    // Optional: Pre-generate WGSL at build time
    // generate_shader_modules().unwrap();
}

```

# Cargo.toml

```toml
[package]
name = "rust_to_wgsl"
version = "0.1.0"
edition = "2024"

[lib]
proc-macro = true
[dependencies]
proc-macro-error = "1.0.4"
proc-macro2 = "1.0.92"
quote = "1.0.38"
syn = { version = "2.0.94", features = ["full"]   }

```

# examples\collision_shader\main.rs

```rs
#![feature(f16)]

mod compose_wgsl_module;
mod wgsl_components;
mod wgsl_in_rust_helpers;
mod wgsl_wgpu_binding;
use rust_to_wgsl::shader_module;
// This module will be transformed
// the user would not normally input the comments, those are just there for the developer, temporary
#[shader_module]
pub mod collision_shader {
    use crate::wgsl_in_rust_helpers::*;
    //* no other use or import statements allowed, since they break wgsl
    //* user Shader-Module-Constant
    const example_module_const: u128 = 42;
    //* library generated per-pipeline constants, will be inserted below
    //*  user generated buffer types
    // only one group of uniforms because this library is designed for simple compute shaders
    struct Uniforms {
        time: f32,
        resolution: Vec3<f32>,
    }
    //* user input vectors
    //todo this changes to alias
    type Position = [f32; 2];
    // todo, this changes to array<f32, 2>
    type Radius = f32;
    //* user output vectors
    struct CollisionResult {
        entity1: u32,
        entity2: u32,
    }
    //*  library generated buffer types, will go below
    //* bindings, all handled by the library, will go below
    //* USER GENERATED HELPER FUNCTIONS
    // Optimized distance calculation
    // todo the array defs convert
    fn calculate_distance_squared(p1: [f32; 2], p2: [f32; 2]) -> f32 {
        let dx = p1[0] - p2[0];
        let dy = p1[1] - p2[1];
        return dx * dx + dy * dy;
    }
    //* Library generated helper functions, will go below
    //todo this is automatically made available
    struct GlobalId {
        x: u32,
        y: u32,
        z: u32,
    }
    //* ENTRY POINT FUNCTION
    // @compute @workgroup_size(WORKGROUP_SIZE_X, WORKGROUP_SIZE_Y, WORKGROUP_SIZE_Z)
    fn main(
        //todo change name to main
        // @builtin(global_invocation_id) global_id: vec3<u32>,
        // todo this changes to the above
        global_id: WgslGlobalId,
    ) {
        //* USER GENERATED LOGIC
        let current_entity = global_id.x;
        let other_entity = global_id.y;
        // Early exit if invalid entity or zero radius
        if current_entity >= WgslVecInput::vec_len::<Position>()
            || other_entity >= WgslVecInput::vec_len::<Position>()
            || current_entity == other_entity
            || current_entity >= other_entity
        {
            return;
        }
        let current_radius = WgslVecInput::vec_val::<Radius>(current_entity);
        let other_radius = WgslVecInput::vec_val::<Radius>(other_entity);
        if current_radius <= 0.0 || other_radius <= 0.0 {
            return;
        }
        let current_pos = WgslVecInput::vec_val::<Position>(current_entity);
        let other_pos = WgslVecInput::vec_val::<Position>(other_entity);
        let dist_squared = calculate_distance_squared(current_pos, other_pos);
        let radius_sum = current_radius + other_radius;
        // Compare squared distances to avoid sqrt
        if dist_squared < radius_sum * radius_sum {
            WgslOutput::push::<CollisionResult>(CollisionResult {
                entity1: current_entity,
                entity2: other_entity,
            });
        }
    }
}

// Each input/output needs a binding number, as well as appropriate handling for pipeline consts
// will need to be able to change the library-generated pipeline const values (array lengths, etc.) in the code
//automatically determine which outputs need atomic counters based on if a push statement is used for them

fn main() {
    // User can test the Rust version directly
    // let shader = collision_shader::create_pipeline();
    // Can also get WGSL version
    // let wgsl = collision_shader::as_wgsl();
}

```

# examples\collision_shader\wgsl_components.rs

```rs
/**
 * ! logic for parsing the shader module
 * *first get all types declared in the module
 * combine these with all standard wgsl types to get a list of all type idents allowed
 * then whenever we see a type that isn't one of those, throw an error
 * *traverse through module scope tokens:
 * extracting them out into the objects shown below
 *
 */

/// includes just the parts the user has input, with any relevant metadata necessary for the library to complete the module
struct WgslShaderModuleUserPortion {
    /// defined with the "const" keyword
    /// single line
    /// value remains static
    /// type must be wgsl type or created somewhere else in the module
    /// value could be a type instantiation, a scalar, or a function
    static_consts: Vec<WgslConstAssignment>,
    /// identified with a #[config_input] attribute above them
    uniforms: Vec<WgslType>,
    /// identified with a #[vec_input] attribute above them
    input_arrays: Vec<WgslArray>,
    /// identified with a #[vec_output] attribute above them
    output_arrays: Vec<WgslOutputArray>,
    /// any function that appears besides the one called "main"
    helper_functions: Vec<WgslFunction>,
    /// the main function, identified by its name: "main"
    /// MUST contain a single parameter called "global_id" of type "WgslGlobalId"
    /// look for any attempt to ASSIGN to the value of "global_id.x", "global_id.y", or "global_id.z" or just "global_id" and throw an error
    main_function: WgslFunction,
}

pub struct WgslType {
    name: String,
    wgsl: String,
}
impl ToString for WgslType {
    fn to_string(&self) -> String {
        return format!("{}{}", self.wgsl.clone(), "\n");
    }
}
pub struct WgslFunction {
    name: String,
    wgsl_definition: String,
}
impl ToString for WgslFunction {
    fn to_string(&self) -> String {
        return format!("{}{}", self.wgsl_definition.clone(), "\n");
    }
}

/// assignments using let can happen within functions and we don't care about them, we don't need to change anything
pub struct WgslConstAssignment {
    assigner_keyword: String,
    var_name: String,
    var_type: WgslType,
    value: String,
}
impl ToString for WgslConstAssignment {
    fn to_string(&self) -> String {
        return format!(
            "{} {}: {} = {};\n",
            self.assigner_keyword, self.var_name, self.var_type.wgsl, self.value
        );
    }
}
pub struct WgslArray {
    type_name: String,
    pub item_type: WgslType,
    length: u32,
}
impl ToString for WgslArray {
    fn to_string(&self) -> String {
        return format!(
            "alias {} = array<{},{}>;\n",
            self.type_name, self.item_type.name, self.length
        );
    }
}
pub struct WgslOutputArray {
    pub arr: WgslArray,
    atomic_counter: bool,
}
impl ToString for WgslOutputArray {
    fn to_string(&self) -> String {
        let mut s = self.arr.to_string();
        if self.atomic_counter {
            s.push_str(&format!(
                "alias {}_counter : atomic<u32>;\n",
                self.arr.item_type.name
            ));
        }
        return s;
    }
}

pub enum WgpuShaderType {
    Compute,
    Vertex,
    Fragment,
}
impl ToString for WgpuShaderType {
    fn to_string(&self) -> String {
        match self {
            WgpuShaderType::Compute => "compute".to_string(),
            WgpuShaderType::Vertex => panic!("Vertex shaders not yet supported"),
            WgpuShaderType::Fragment => panic!("Fragment shaders not yet supported"),
        }
    }
}
pub struct WgslWorkgroupDeclaration {
    shader_type: WgpuShaderType,
    x: u32,
    y: u32,
    z: u32,
}
impl ToString for WgslWorkgroupDeclaration {
    fn to_string(&self) -> String {
        return format!(
            "@{} @workgroup_size({}, {}, {})\n",
            self.shader_type.to_string(),
            self.x,
            self.y,
            self.z
        );
    }
}

```

# examples\collision_shader\wgsl_in_rust_helpers.rs

```rs
pub trait WgslScalar {}
impl WgslScalar for bool {}
impl WgslScalar for u32 {}
impl WgslScalar for i32 {}
impl WgslScalar for f32 {}
impl WgslScalar for f16 {}
pub type Vec2<T: WgslScalar> = [T; 2];
pub type Vec3<T: WgslScalar> = [T; 3];
pub type Vec4<T: WgslScalar> = [T; 4];
pub type Mat2x2<T: WgslScalar> = [Vec2<T>; 2];
pub type Mat3x3<T: WgslScalar> = [Vec3<T>; 3];
pub type Mat4x4<T: WgslScalar> = [Vec4<T>; 4];

pub struct WgslGlobalId {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

/// This actually refers to wgpu "Uniforms", but most people don't know what those are, so we call them "configs" instead.
pub struct WgslConfigInput {}
impl WgslConfigInput {
    pub fn get<T>() -> T {
        unimplemented!()
    }
}

/// These method are named "vec" because per this library API you input your data as variable-sized vectors. But keep in mind that on the actual GPU these are all fixed-length arrays.
pub struct WgslVecInput {}
impl WgslVecInput {
    pub fn vec_len<T>() -> u32 {
        unimplemented!()
    }
    pub fn vec_val<T>(index: u32) -> T {
        unimplemented!()
    }
}
/**
 * All outputs are arrays/vectors.
 * No "get" type methods are implemented, sinc GPU operations are massively parallel, and you should not be READING from your outputs since you will have no way of knowing if another thread has already touched a certain output or not handled it yet. //todo: (need to add a link to an article explaining this)
 */
pub struct WgslOutput {}
impl WgslOutput {
    /// Using "array_set" is generally more performant, use this only if you can't determine beforehand the number of outputs you will be producing.
    /// Using this will generate an atomic counter for the specific output. If your ACTUAL output length is always very close to your maximum output length for this specific output, consider using "array_set" and manually removing the trailing empty values instead.
    /// However if your ACTUAL output length could be significantly smaller than your set maximum output length, then using "vec_push" is more performant. It was created for those situations. For example: collision detection where the actual number of collisions is often far less than the theoretical maximum.
    pub fn push<T>(val: T) {
        unimplemented!()
    }
    pub fn set<T>(index: u32, val: T) {
        unimplemented!()
    }
    /// returns the user-input maximum number of elements that can be stored in the output for this specific type.
    pub fn len<T>() -> u32 {
        unimplemented!()
    }
}

```

# src\lib.rs

```rs
use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro_error::{proc_macro_error, set_dummy};
use quote::quote;
use syn::{parse, parse_macro_input, token::Semi};
use transformer::type_transformer::apply_known_rust_to_wgsl_type_transformations;
mod runtime;
mod transformer;

/**
## *Please read this documentation carefully, especially if you are getting errors that you don't understand!*

*...because it's currently impossible with rust proc-macros to override some of the error messages, so a lot of them don't actually indicate correctly what your issue is. So this documentation is how you are going to solve them!*

Here are some pointers:
- No let statements allowed except within functions. If you want to define a variable use "const" instead.
- Every Input/Output you want to transfer between the CPU and GPU must have its type defined within the shader module. Here's how you do that:
    - Input Vec/Array/Matrices: Define the inner-type, and put `#[vec_input]` above the type definition. Example: If you want to work with an input equivalent to `Vec<{x:f32, y:f32}>` in your module, then write
    \`\`\`
    #[vec_input]
    pub struct MyStruct { x: f32, y: f32 }
    \`\`\`

    We wrap the inner type in an array for you automatically, so that you don't have to worry about data length or alignments.
    // todo


 */
#[proc_macro_attribute]
#[proc_macro_error]
pub fn shader_module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    set_dummy(item.clone().into());
    let mut module = parse_macro_input!(item as syn::ItemMod);
    // todo, convert the rust module (see example main file) to a WgslShaderModuleUserPortion object
    quote! (
        #module
    )
    .into()
}

```

# src\runtime\mod.rs

```rs

```

# src\runtime\wgsl_types.rs

```rs

```

# src\transformer\io_transform.rs

```rs

```

# src\transformer\mod.rs

```rs


```

# src\transformer\type_transformer.rs

```rs
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

```
