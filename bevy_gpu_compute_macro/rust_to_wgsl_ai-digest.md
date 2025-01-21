# .aidigestignore

```
target/*
target
compose_wgsl_module
wgsl_wgpu_binding
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
syn = { version = "2.0.94", features = ["full","visit-mut","visit"]   }
shared = {path = "../shared"}
regex = "1.11.1"
compiletest_rs = "0.11.2"
trybuild = "1.0.101"
bytemuck = {version = "1.21.0", features=["derive"]}

[dev-dependencies]
pretty_assertions = "1.4.1"

```

# src\lib.rs

```rs
#![feature(allocator_api)]

use proc_macro::TokenStream;
use proc_macro_error::{proc_macro_error, set_dummy};
use state::ModuleTransformState;
use syn::parse_macro_input;
use transformer::{
    custom_types::get_all_custom_types::get_custom_types,
    module_parser::module_parser::parse_shader_module, output::produce_expanded_output,
    remove_doc_comments::remove_doc_comments,
    transform_wgsl_helper_methods::run::transform_wgsl_helper_methods,
};
mod state;
mod transformer;
/**
## *Please read this documentation carefully, especially if you are getting errors that you don't understand!*

*...because it's currently impossible with rust proc-macros to override some of the error messages, so a lot of them don't actually indicate correctly what your issue is. So this documentation is how you are going to solve them!*

Here are some pointers:
- No let statements allowed except within functions. If you want to define a variable use "const" instead.
- If you see an error like `cannot transmute between types of different sizes...`, it is probably a memory padding/alignment issue. The f16 (aka PodF16) is a common culprit. If you have structs with multiple fields of different types, try adding various amounts of padding to the structs to see if that fixes the issue.
- When accessing special WGSL types like `Vec3`, `Mat3x4`, etc. you CANNOT use parenthesis when accessing the fields. For example:
### Valid:
\`\`\`rust
# use shared::wgsl_in_rust_helpers::*;
let my_vec3 = Vec2I32::new(1,2);
// valid:
let x = my_vec3.x;
let x = my_vec3[0];
\`\`\`
### Invalid:
\`\`\`compile_fail
// invalid:
let x = my_vec3.x();
\`\`\`
- "rgba" field access on vectors is not supported right now, use index or xyzw instead.
- Every Input/Output you want to transfer between the CPU and GPU must have its type defined within the shader module. Here's how you do that:
    - Input Vec/Array/Matrices: Define the inner-type, and put `#[vec_input]` above the type definition. Example: If you want to work with an input equivalent to `Vec<{x:f32, y:f32}>` in your module, then write
    \`\`\`ignore
    #[vec_input]
    pub struct MyStruct { x: f32, y: f32 }
    \`\`\`

    We wrap the inner type in an array for you automatically, so that you don't have to worry about data length or alignments.
    // todo
* You cannot use rust's struct-literal syntax for initializing structs from the helper types module. You must use the `new` method instead. For example:
    ## Wrong:
     \`\`\`compile_fail
         #use shared::wgsl_in_rust_helpers::*;
          const MY_CONST: Vec3Bool = Vec3Bool {
        x: true,
        y: false,
        z: true,
    };\`\`\`


    ## Correct:
    \`\`\`
    #use shared::wgsl_in_rust_helpers::*;
    const MY_CONST: Vec3Bool = Vec3Bool::new(true, false, true);
    \`\`\`
* If you see the error `the trait bound `bool: Pod` is not satisfied...` make sure you are not trying to use a `bool` in any input data or output data. The `bool` type CAN be used but only ON the GPU, it cannot be passed between the CPU and GPU.
 */
#[proc_macro_attribute]
#[proc_macro_error]
pub fn wgsl_shader_module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("Entered shader_module proc macro");
    set_dummy(item.clone().into());
    let content = item.to_string();
    let content_no_doc_comments: TokenStream = remove_doc_comments(&content).parse().unwrap();
    let module = parse_macro_input!(content_no_doc_comments as syn::ItemMod);
    let mut state = ModuleTransformState::empty(module, content);
    get_custom_types(&mut state);
    transform_wgsl_helper_methods(&mut state);
    parse_shader_module(&mut state);
    let output = produce_expanded_output(&state);
    output.into()

    // let out_s = initialization.to_string();
    // quote!(struct S {};#out_s).into()
    // output the original rust as well, to allow for correct syntax/ compile checking on it
    // quote!({}).into()
}

/// used to help this library figure out what to do with user-defined types
#[proc_macro_attribute]
#[proc_macro_error]
pub fn wgsl_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    return item;
}
/// used to help this library figure out what to do with user-defined types
#[proc_macro_attribute]
#[proc_macro_error]
pub fn wgsl_input_array(_attr: TokenStream, item: TokenStream) -> TokenStream {
    return item;
}
/// used to help this library figure out what to do with user-defined types
#[proc_macro_attribute]
#[proc_macro_error]
pub fn wgsl_output_vec(_attr: TokenStream, item: TokenStream) -> TokenStream {
    return item;
}
/// used to help this library figure out what to do with user-defined types
#[proc_macro_attribute]
#[proc_macro_error]
pub fn wgsl_output_array(_attr: TokenStream, item: TokenStream) -> TokenStream {
    return item;
}

```

# src\state.rs

```rs
use shared::wgsl_components::WgslShaderModuleUserPortion;
use syn::ItemMod;

use crate::transformer::custom_types::custom_type::CustomType;

pub struct ModuleTransformState {
    _original_content: String,
    pub rust_module: ItemMod,
    pub custom_types: Option<Vec<CustomType>>,
    pub module_visibility: Option<String>,
    pub module_ident: Option<String>,
    pub result: WgslShaderModuleUserPortion,
}
impl ModuleTransformState {
    pub fn empty(rust_module: ItemMod, content: String) -> Self {
        Self {
            _original_content: content,
            rust_module,
            custom_types: None,
            module_visibility: None,
            module_ident: None,
            result: WgslShaderModuleUserPortion::empty(),
        }
    }
    pub fn get_original_content(&self) -> String {
        self._original_content.clone()
    }
}

```

# src\transformer\allowed_types.rs

```rs
pub const WGSL_NATIVE_TYPES: [&str; 65] = [
    "Vec2I32",
    "Vec2U32",
    "Vec2F32",
    "Vec2F16",
    "Vec2Bool",
    "Vec3I32",
    "Vec3U32",
    "Vec3F32",
    "Vec3F16",
    "Vec3Bool",
    "Vec4I32",
    "Vec4U32",
    "Vec4F32",
    "Vec4F16",
    "Vec4Bool",
    "Mat2x2I32",
    "Mat2x2U32",
    "Mat2x2F32",
    "Mat2x2F16",
    "Mat2x2Bool",
    "Mat2x3U32",
    "Mat2x3I32",
    "Mat2x3F32",
    "Mat2x3F16",
    "Mat2x3Bool",
    "Mat2x4I32",
    "Mat2x4U32",
    "Mat2x4F32",
    "Mat2x4F16",
    "Mat2x4Bool",
    "Mat3x2I32",
    "Mat3x2U32",
    "Mat3x2F32",
    "Mat3x2F16",
    "Mat3x2Bool",
    "Mat3x3I32",
    "Mat3x3U32",
    "Mat3x3F32",
    "Mat3x3F16",
    "Mat3x3Bool",
    "Mat3x4I32",
    "Mat3x4U32",
    "Mat3x4F32",
    "Mat3x4F16",
    "Mat3x4Bool",
    "Mat4x2I32",
    "Mat4x2U32",
    "Mat4x2F32",
    "Mat4x2F16",
    "Mat4x2Bool",
    "Mat4x3I32",
    "Mat4x3U32",
    "Mat4x3F32",
    "Mat4x3F16",
    "Mat4x3Bool",
    "Mat4x4I32",
    "Mat4x4U32",
    "Mat4x4F32",
    "Mat4x4F16",
    "Mat4x4Bool",
    "f32",
    "f16",
    "i32",
    "u32",
    "bool",
];
#[allow(dead_code)]
const LIB_HELPER_TYPES: [&str; 5] = [
    "WgslScalar",
    "WgslIterationPosition",
    "WgslConfigInput",
    "WgslVecInput",
    "WgslOutput",
];

```

# src\transformer\custom_types\custom_type_idents.rs

```rs
use proc_macro2::Span;
use quote::format_ident;
use shared::custom_type_name::CustomTypeName;
use syn::Ident;

#[derive(Clone, Debug)]

pub struct CustomTypeIdents {
    pub name: Ident,
    pub upper: Ident,
    pub lower: Ident,
}
impl CustomTypeIdents {
    pub fn new(name: &Ident) -> Self {
        let upper = Ident::new(&name.to_string().to_uppercase(), Span::call_site());
        let lower = Ident::new(&name.to_string().to_lowercase(), Span::call_site());
        Self {
            name: name.clone(),
            upper,
            lower,
        }
    }
    pub fn eq(&self, other: &Ident) -> bool {
        self.name.to_string() == *other.to_string()
    }
    pub fn uniform(&self) -> &Ident {
        &self.lower
    }
    pub fn input_array_length(&self) -> Ident {
        format_ident!("{}_INPUT_ARRAY_LENGTH", self.upper)
    }
    pub fn input_array(&self) -> Ident {
        format_ident!("{}_input_array", self.lower)
    }

    pub fn output_array_length(&self) -> Ident {
        format_ident!("{}_OUTPUT_ARRAY_LENGTH", self.upper)
    }
    pub fn output_array(&self) -> Ident {
        format_ident!("{}_output_array", self.lower)
    }

    pub fn counter(&self) -> Ident {
        format_ident!("{}_counter", self.lower)
    }
    pub fn index(&self) -> Ident {
        format_ident!("{}_output_array_index", self.lower)
    }
}

impl Into<CustomTypeName> for CustomTypeIdents {
    fn into(self) -> CustomTypeName {
        CustomTypeName::new(&self.name.to_string())
    }
}

```

# src\transformer\custom_types\custom_type.rs

```rs
use std::alloc::Global;

use proc_macro2::TokenStream;
use shared::wgsl_components::{WgslShaderModuleSectionCode, WgslType};
use syn::{Attribute, Ident};

use crate::{state::ModuleTransformState, transformer::to_wgsl_syntax::convert_file_to_wgsl};

use super::custom_type_idents::CustomTypeIdents;

#[derive(PartialEq, Clone, Debug)]
pub enum CustomTypeKind {
    GpuOnlyHelperType,
    Uniform,
    InputArray,
    OutputArray,
    OutputVec,
    ArrayLengthVariable,
}

impl From<&Vec<Attribute, Global>> for CustomTypeKind {
    fn from(attrs: &Vec<Attribute, Global>) -> Self {
        for attr in attrs {
            if attr.path().is_ident("wgsl_config") {
                return CustomTypeKind::Uniform;
            } else if attr.path().is_ident("wgsl_input_array") {
                return CustomTypeKind::InputArray;
            } else if attr.path().is_ident("wgsl_output_array") {
                return CustomTypeKind::OutputArray;
            } else if attr.path().is_ident("wgsl_output_vec") {
                return CustomTypeKind::OutputVec;
            }
        }
        CustomTypeKind::GpuOnlyHelperType
    }
}
#[derive(Clone, Debug)]
pub struct CustomType {
    pub name: CustomTypeIdents,
    pub kind: CustomTypeKind,
    pub rust_code: TokenStream,
}
impl CustomType {
    pub fn new(name: &Ident, kind: CustomTypeKind, type_def_code: TokenStream) -> Self {
        Self {
            name: CustomTypeIdents::new(name),
            kind,
            rust_code: type_def_code,
        }
    }
    pub fn into_wgsl_type(self, state: &ModuleTransformState) -> WgslType {
        WgslType {
            name: self.name.into(),
            code: WgslShaderModuleSectionCode {
                rust_code: self.rust_code.to_string(),
                wgsl_code: convert_file_to_wgsl(self.rust_code, &state, "custom_type".to_string()),
            },
        }
    }
}

```

# src\transformer\custom_types\get_all_custom_types.rs

```rs
// find all user declared types, and make a list of them

// ItemStruct.ident or  ItemType.ident

use quote::ToTokens;
use syn::visit::Visit;

use crate::state::ModuleTransformState;

use super::custom_type::{CustomType, CustomTypeKind};

struct CustomTypesLister {
    custom_types: Vec<CustomType>,
}

impl<'ast> Visit<'ast> for CustomTypesLister {
    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        syn::visit::visit_item_struct(self, i);

        self.custom_types.push(CustomType::new(
            &i.ident,
            CustomTypeKind::from(&i.attrs),
            i.to_token_stream(),
        ));
    }

    fn visit_item_type(&mut self, i: &'ast syn::ItemType) {
        syn::visit::visit_item_type(self, i);
        self.custom_types.push(CustomType::new(
            &i.ident,
            CustomTypeKind::from(&i.attrs),
            i.to_token_stream(),
        ));
    }
}

impl CustomTypesLister {
    pub fn new() -> Self {
        CustomTypesLister {
            custom_types: vec![],
        }
    }
}

pub fn get_custom_types(state: &mut ModuleTransformState) {
    let mut types_lister = CustomTypesLister::new();
    types_lister.visit_item_mod(&state.rust_module);
    // println!("allowed types {:?}", types_lister.allowed_types);
    state.custom_types = Some(types_lister.custom_types);
}

```

# src\transformer\custom_types\mod.rs

```rs
pub mod custom_type;
pub mod custom_type_idents;
pub mod get_all_custom_types;

```

# src\transformer\mod.rs

```rs
pub mod allowed_types;
pub mod custom_types;
pub mod module_parser;
pub mod output;
pub mod remove_doc_comments;
pub mod to_wgsl_syntax;
pub mod transform_wgsl_helper_methods;

```

# src\transformer\module_parser\constants.rs

```rs
use quote::ToTokens;
use shared::wgsl_components::{WgslConstAssignment, WgslShaderModuleSectionCode};
use syn::{ItemConst, visit::Visit};

use crate::{state::ModuleTransformState, transformer::to_wgsl_syntax::convert_file_to_wgsl};

pub fn find_constants(state: &mut ModuleTransformState) {
    let rust_module = state.rust_module.clone();
    let mut extractor = ConstantsExtractor::new(state);
    extractor.visit_item_mod(&rust_module);
    state.rust_module = rust_module;
}

struct ConstantsExtractor<'a> {
    state: &'a mut ModuleTransformState,
}

impl<'ast> Visit<'ast> for ConstantsExtractor<'ast> {
    fn visit_item_const(&mut self, c: &'ast syn::ItemConst) {
        syn::visit::visit_item_const(self, c);
        self.state
            .result
            .static_consts
            .push(parse_const_assignment(c, self.state));
    }
}

impl<'ast> ConstantsExtractor<'ast> {
    pub fn new(state: &'ast mut ModuleTransformState) -> Self {
        ConstantsExtractor { state }
    }
}

fn parse_const_assignment(
    constant: &ItemConst,
    state: &ModuleTransformState,
) -> WgslConstAssignment {
    WgslConstAssignment {
        code: WgslShaderModuleSectionCode {
            rust_code: constant.to_token_stream().to_string(),
            wgsl_code: convert_file_to_wgsl(constant.to_token_stream(), &state, "const".to_string()),
        },
    }
}

```

# src\transformer\module_parser\divide_custom_types.rs

```rs
use proc_macro_error::abort;
use shared::wgsl_components::{WgslInputArray, WgslOutputArray};

use crate::{
    state::ModuleTransformState,
    transformer::custom_types::custom_type::{CustomType, CustomTypeKind},
};
use quote::quote;

pub fn divide_custom_types_by_category(state: &mut ModuleTransformState) {
    let custom_types = if let Some(ct) = state.custom_types.clone() {
        ct
    } else {
        abort!(
            state.rust_module.ident.span(),
            "Allowed types must be set before dividing custom types"
        );
    };
    for custom_type in custom_types.iter() {
        match custom_type.kind {
            CustomTypeKind::GpuOnlyHelperType => state
                .result
                .helper_types
                .push(custom_type.clone().into_wgsl_type(&state)),
            CustomTypeKind::InputArray => {
                state.custom_types.as_mut().unwrap().push(CustomType::new(
                    &custom_type.name.input_array_length(),
                    CustomTypeKind::ArrayLengthVariable,
                    quote!(),
                ));
                println!("HERE");
                state.result.input_arrays.push(WgslInputArray {
                    item_type: custom_type.clone().into_wgsl_type(&state),
                });
            }
            CustomTypeKind::OutputArray => {
                state.custom_types.as_mut().unwrap().push(CustomType::new(
                    &custom_type.name.output_array_length(),
                    CustomTypeKind::ArrayLengthVariable,
                    quote!(),
                ));
                state.result.output_arrays.push(WgslOutputArray {
                    item_type: custom_type.clone().into_wgsl_type(&state),
                    atomic_counter_name: None,
                });
            }
            CustomTypeKind::OutputVec => {
                state.custom_types.as_mut().unwrap().push(CustomType::new(
                    &custom_type.name.output_array_length(),
                    CustomTypeKind::ArrayLengthVariable,
                    quote!(),
                ));
                state.result.output_arrays.push(WgslOutputArray {
                    item_type: custom_type.clone().into_wgsl_type(state),
                    atomic_counter_name: Some(custom_type.name.counter().to_string()),
                });
            }
            CustomTypeKind::Uniform => state
                .result
                .uniforms
                .push(custom_type.clone().into_wgsl_type(state)),
            CustomTypeKind::ArrayLengthVariable => {
                // do nothing
            }
        }
    }
}

```

# src\transformer\module_parser\helper_functions.rs

```rs
use quote::ToTokens;
use shared::wgsl_components::{WgslFunction, WgslShaderModuleSectionCode};
use syn::{ItemFn, visit::Visit};

use crate::{state::ModuleTransformState, transformer::to_wgsl_syntax::convert_file_to_wgsl};

pub fn find_helper_functions(mut state: &mut ModuleTransformState) {
    let module = state.rust_module.clone();
    let mut extractor = HelperFunctionsExtractor::new(&mut state);
    extractor.visit_item_mod(&module);
    state.rust_module = module;
}

struct HelperFunctionsExtractor<'a> {
    state: &'a mut ModuleTransformState,
}

impl<'ast> Visit<'ast> for HelperFunctionsExtractor<'ast> {
    fn visit_item_fn(&mut self, c: &'ast syn::ItemFn) {
        syn::visit::visit_item_fn(self, c);
        if c.sig.ident.to_string() == "main" {
            return;
        }
        // ident from string

        self.state
            .result
            .helper_functions
            .push(parse_fn(c, self.state));
    }
}

impl<'ast> HelperFunctionsExtractor<'ast> {
    pub fn new(state: &'ast mut ModuleTransformState) -> Self {
        HelperFunctionsExtractor { state }
    }
}

fn parse_fn(func: &ItemFn, state: &ModuleTransformState) -> WgslFunction {
    WgslFunction {
        code: WgslShaderModuleSectionCode {
            rust_code: func.to_token_stream().to_string(),
            wgsl_code: convert_file_to_wgsl(func.to_token_stream(), state, "helper fn".to_string()),
        },
        name: func.sig.ident.to_string(),
    }
}

```

# src\transformer\module_parser\main_function.rs

```rs
use crate::{state::ModuleTransformState, transformer::to_wgsl_syntax::convert_file_to_wgsl};
use proc_macro::Span;
use proc_macro_error::abort;
use quote::ToTokens;
use shared::wgsl_components::{WgslFunction, WgslShaderModuleSectionCode};
use syn::{ItemFn, spanned::Spanned, visit::Visit};

pub fn find_main_function(mut state: &mut ModuleTransformState) {
    let module = state.rust_module.clone();
    let mut extractor = MainFunctionsExtractor::new(&mut state);
    extractor.visit_item_mod(&module);
    let main_func = if let Some(mf) = &state.result.main_function {
        mf
    } else {
        abort!(state.rust_module.ident.span(), "No main function found");
    };
    let r_code = main_func.code.rust_code.clone();
    validate_main_function(r_code);
    state.rust_module = module;
}

struct MainFunctionsExtractor<'a> {
    count: usize,
    state: &'a mut ModuleTransformState,
}

impl<'ast> Visit<'ast> for MainFunctionsExtractor<'ast> {
    fn visit_item_fn(&mut self, c: &'ast syn::ItemFn) {
        syn::visit::visit_item_fn(self, c);
        let name = c.sig.ident.to_string();
        if name != "main" {
            return;
        }
        self.count += 1;
        if self.count > 1 {
            abort!(c.sig.ident.span(), "Only one main function is allowed");
        }
        self.state.result.main_function = Some(parse_main_fn(c, self.state));
    }
}

impl<'ast> MainFunctionsExtractor<'ast> {
    pub fn new(state: &'ast mut ModuleTransformState) -> Self {
        MainFunctionsExtractor { count: 0, state }
    }
}

fn parse_main_fn(func: &ItemFn, state: &ModuleTransformState) -> WgslFunction {
    let func_clone = func.clone();
    // alter the main function argument
    WgslFunction {
        code: WgslShaderModuleSectionCode {
            rust_code: func_clone.to_token_stream().to_string(),
            wgsl_code: alter_global_id_argument(convert_file_to_wgsl(
                func_clone.to_token_stream(),
                state,
                "main".to_string(),
            )),
        },
        name: func_clone.sig.ident.to_string(),
    }
}
/// we have to alter the main function argument to match the wgsl spec by string replace instead of ast manipulation because the new argument is not a valid rust syntax
fn alter_global_id_argument(func_string: String) -> String {
    let match_patterns = [
        "iter_pos: WgslIterationPosition",
        "iter_pos : WgslIterationPosition",
        "iter_pos:WgslIterationPosition",
        "iter_pos:  WgslIterationPosition",
    ];
    let replace_pattern = "@builtin(global_invocation_id) iter_pos: vec3<u32>";
    let mut new_func = func_string.clone();
    let mut found = false;
    for pattern in match_patterns.iter() {
        if new_func.find(pattern).is_some() {
            found = true;
            new_func = new_func.replace(pattern, replace_pattern);
        }
    }
    if !found {
        let error_message = format!(
            "Failed to find main function argument, we are looking for a string that exactly matches 'iter_pos: WgslIterationPosition', found {}",
            new_func
        );
        abort!(Span::call_site(), error_message);
    }
    new_func
}

fn validate_main_function(function_string: String) {
    let function = if let Ok(f) = syn::parse_str::<ItemFn>(&function_string) {
        f
    } else {
        let message = format!("Failed to parse main function: {}", function_string);
        abort!(Span::call_site(), message);
    };
    // Check that main has exactly one parameter
    if function.sig.inputs.len() != 1 {
        abort!(
            function.sig.span(),
            "Main function must have exactly one parameter of type WgslIterationPosition"
        );
    }
    // Validate the parameter type is WgslIterationPosition called "global_id"
    if let syn::FnArg::Typed(pat_type) = &function.sig.inputs[0] {
        match &*pat_type.pat {
            syn::Pat::Ident(_) => {}
            _ => abort!(
                pat_type.pat.span(),
                "Main function parameter must be called 'iter_pos'"
            ),
        }
        if let syn::Type::Path(type_path) = &*pat_type.ty {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident != "WgslIterationPosition" {
                    abort!(
                        pat_type.ty.span(),
                        "Main function parameter must be of type WgslIterationPosition"
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
}

```

# src\transformer\module_parser\mod.rs

```rs
pub mod constants;
pub mod divide_custom_types;
pub mod helper_functions;
pub mod main_function;
pub mod module_parser;
pub mod use_statements;
pub mod validate_no_global_id_assignments;

```

# src\transformer\module_parser\module_parser.rs

```rs
use proc_macro_error::abort;
use quote::ToTokens;

use crate::state::ModuleTransformState;

use super::constants::find_constants;
use super::divide_custom_types::divide_custom_types_by_category;
use super::helper_functions::find_helper_functions;
use super::main_function::find_main_function;
use super::use_statements::handle_use_statements;
use super::validate_no_global_id_assignments::check_module_for_global_id_assignment;

pub fn parse_shader_module(mut state: &mut ModuleTransformState) {
    if state.rust_module.content.is_none() {
        abort!(
            state.rust_module.ident.span(),
            "Shader module must have a body"
        );
    }
    find_main_function(&mut state);
    handle_use_statements(&mut state);
    state.module_ident = Some(state.rust_module.ident.to_string());
    state.module_visibility = Some(state.rust_module.vis.to_token_stream().to_string());
    check_module_for_global_id_assignment(&mut state);
    find_constants(&mut state);
    divide_custom_types_by_category(&mut state);
    find_helper_functions(&mut state);
}

```

# src\transformer\module_parser\use_statements.rs

```rs
use proc_macro_error::abort;
use quote::{ToTokens, quote};
use syn::{Item, ItemUse, spanned::Spanned, visit::Visit, visit_mut::VisitMut};

use crate::state::ModuleTransformState;

const VALID_USE_STATEMENT_PATHS: [&str; 2] = ["wgsl_in_rust_helpers", "rust_to_wgsl"];

pub fn handle_use_statements(state: &mut ModuleTransformState) {
    let mut handler = UseStatementHandler {};
    handler.visit_item_mod_mut(&mut state.rust_module);
}

struct UseStatementHandler {}

impl VisitMut for UseStatementHandler {
    fn visit_item_mut(&mut self, i: &mut Item) {
        syn::visit_mut::visit_item_mut(self, i);
        match i {
            Item::Use(use_stmt) => {
                validate_use_statement(&use_stmt);
                // remove the use statement
                *i = Item::Verbatim(quote! {})
            }
            _ => {}
        }
    }
}

fn validate_use_statement(use_stmt: &ItemUse) {
    let mut single_handler = SingleUseStatementHandler { found: false };
    single_handler.visit_item_use(use_stmt);
    if !single_handler.found {
        let message = format!(
            "Invalid use statement: {:?}. You are only allowed to import from one of these crates: {}",
            use_stmt.to_token_stream().to_string(),
            VALID_USE_STATEMENT_PATHS.join(", ")
        );
        abort!(use_stmt.span(), message);
    }
}

struct SingleUseStatementHandler {
    found: bool,
}

impl<'ast> Visit<'ast> for SingleUseStatementHandler {
    fn visit_use_path(&mut self, i: &syn::UsePath) {
        syn::visit::visit_use_path(self, i);
        if VALID_USE_STATEMENT_PATHS.contains(&i.ident.to_string().as_str()) {
            self.found = true;
        }
    }
    fn visit_use_name(&mut self, i: &syn::UseName) {
        syn::visit::visit_use_name(self, i);
        if VALID_USE_STATEMENT_PATHS.contains(&i.ident.to_string().as_str()) {
            self.found = true;
        }
    }
}

```

# src\transformer\module_parser\validate_no_global_id_assignments.rs

```rs
use proc_macro_error::abort;
use syn::{ExprAssign, spanned::Spanned, visit::Visit};

use crate::state::ModuleTransformState;

pub fn check_module_for_global_id_assignment(state: &mut ModuleTransformState) {
    let mut checker = GlobalIdAssignmentChecker {};
    checker.visit_item_mod(&state.rust_module);
}

struct GlobalIdAssignmentChecker {}
impl<'ast> Visit<'ast> for GlobalIdAssignmentChecker {
    fn visit_expr_assign(&mut self, c: &'ast syn::ExprAssign) {
        syn::visit::visit_expr_assign(self, c);
        check_for_global_id_assignment(c);
    }
}

fn check_for_global_id_assignment(assign: &ExprAssign) {
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
}

```

# src\transformer\output\expanded_module.rs

```rs
use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    state::ModuleTransformState,
    transformer::output::{
        shader_module_object::generate_shader_module_object,
        types_for_rust_usage::types::define_types_for_use_in_rust,
    },
};
pub fn generate_expanded_module(state: &ModuleTransformState) -> TokenStream {
    let module_ident: TokenStream = if let Some(c) = &state.module_ident {
        c.parse().unwrap()
    } else {
        abort!(
            state.rust_module.ident.span(),
            "No module ident found in transform state"
        );
    };
    let module_visibility: TokenStream = if let Some(c) = &state.module_visibility {
        c.parse().unwrap()
    } else {
        abort!(
            state.rust_module.ident.span(),
            "No module visibility found in transform state"
        );
    };
    let types = define_types_for_use_in_rust(state);
    let object = generate_shader_module_object(state);
    quote!(
        #module_visibility mod #module_ident {
            use shared::wgsl_components::*; //todo, make this less brittle
            use shared::custom_type_name::*;
            use shared::misc_types::*;
            use shared::wgsl_in_rust_helpers::*;
            // use shared::wgsl_in_rust_helpers::pod_bool::PodBool;

            #types

            #object
        }
    )
}

```

# src\transformer\output\mod.rs

```rs
use crate::state::ModuleTransformState;
use expanded_module::generate_expanded_module;
use proc_macro2::TokenStream;
use quote::quote;
use unaltered_module::generate_unaltered_module;

mod expanded_module;
mod per_component_expansion;
mod shader_module_object;
mod types_for_rust_usage;
mod unaltered_module;
pub fn produce_expanded_output(state: &ModuleTransformState) -> TokenStream {
    let unaltered_module = generate_unaltered_module(state);
    let expanded_module = generate_expanded_module(state);
    quote!(
        #unaltered_module

        #expanded_module
    )
}

```

# src\transformer\output\per_component_expansion.rs

```rs
use proc_macro2::TokenStream;
use quote::quote;
use shared::{
    custom_type_name::CustomTypeName,
    wgsl_components::{
        WgslConstAssignment, WgslFunction, WgslInputArray, WgslOutputArray,
        WgslShaderModuleSectionCode, WgslType,
    },
};

pub struct ToStructInitializer {}

impl ToStructInitializer {
    pub fn wgsl_shader_module_component(c: WgslShaderModuleSectionCode) -> TokenStream {
        let r = c.rust_code;
        let w = c.wgsl_code;
        quote!(
            WgslShaderModuleSectionCode {
                rust_code: (#r).to_string(),
                wgsl_code: (#w).to_string(),
            }
        )
    }

    pub fn wgsl_type(c: WgslType) -> TokenStream {
        let n = ToStructInitializer::custom_type_name(c.name);
        let c = ToStructInitializer::wgsl_shader_module_component(c.code);
        quote!(
            WgslType {
                name: #n,
                code: #c,
            }
        )
        .into()
    }

    pub fn custom_type_name(c: CustomTypeName) -> TokenStream {
        let n = c.name();
        quote!(
            CustomTypeName::new(#n)
        )
    }

    pub fn wgsl_function(c: WgslFunction) -> TokenStream {
        let n = c.name;
        let c = ToStructInitializer::wgsl_shader_module_component(c.code);
        quote!(
            WgslFunction {
                name: (#n).to_string(),
                code: #c
            }
        )
    }

    pub fn wgsl_const_assignment(c: WgslConstAssignment) -> TokenStream {
        let c = ToStructInitializer::wgsl_shader_module_component(c.code);
        quote!(
            WgslConstAssignment {
                code: #c,
            }
        )
    }

    pub fn wgsl_input_array(c: WgslInputArray) -> TokenStream {
        let i = ToStructInitializer::wgsl_type(c.item_type);
        quote!(
            WgslInputArray {
                item_type: #i,
            }
        )
    }

    pub fn wgsl_output_array(c: WgslOutputArray) -> TokenStream {
        let i = ToStructInitializer::wgsl_type(c.item_type);
        let ac: TokenStream = c
            .atomic_counter_name
            .as_ref()
            .map_or("None".to_string(), |counter| {
                format!("Some(\"{}\".to_string())", counter)
            })
            .to_string()
            .parse()
            .unwrap();
        quote!(
            WgslOutputArray {
                item_type: #i,
                atomic_counter_name: #ac

            }
        )
    }
}

```

# src\transformer\output\remove_attributes.rs

```rs
/// whenever `#[` is detected, remove everything up to the next `]` using regex, also remove the hash+square brackets
pub fn remove_attributes(file: String) -> String {
    let re = regex::Regex::new(r"#\[[^\]]*\]").unwrap();
    re.replace_all(&file, "").to_string()
}

```

# src\transformer\output\shader_module_object.rs

```rs
use crate::{
    state::ModuleTransformState, transformer::output::per_component_expansion::ToStructInitializer,
};
use proc_macro2::TokenStream;
use quote::quote;
use shared::wgsl_components::WgslShaderModuleUserPortion;

pub fn generate_shader_module_object(state: &ModuleTransformState) -> TokenStream {
    let obj: WgslShaderModuleUserPortion = state.result.clone();

    let static_consts: TokenStream = obj
        .static_consts
        .into_iter()
        .map(|const_assignment| {
            let ts = ToStructInitializer::wgsl_const_assignment(const_assignment);
            quote!(#ts,)
        })
        .collect();

    let helper_types: TokenStream = obj
        .helper_types
        .into_iter()
        .map(|type_def| {
            let ts = ToStructInitializer::wgsl_type(type_def);
            quote!(#ts,)
        })
        .collect();

    let uniforms2: TokenStream = obj
        .uniforms
        .into_iter()
        .map(|uniform| {
            let ts = ToStructInitializer::wgsl_type(uniform);
            quote!(#ts,)
        })
        .collect();

    let input_arrays: TokenStream = obj
        .input_arrays
        .into_iter()
        .map(|array| {
            let ts = ToStructInitializer::wgsl_input_array(array);
            quote!(#ts,)
        })
        .collect();

    let output_arrays: TokenStream = obj
        .output_arrays
        .into_iter()
        .map(|output_array| {
            let ts = ToStructInitializer::wgsl_output_array(output_array);
            quote!(#ts,)
        })
        .collect();

    let helper_functions: TokenStream = obj
        .helper_functions
        .into_iter()
        .map(|func| {
            let ts = ToStructInitializer::wgsl_function(func);
            quote!(#ts,)
        })
        .collect();

    let main_function: TokenStream = obj.main_function.map_or(quote!(None), |func| {
        let ts = ToStructInitializer::wgsl_function(func);
        quote!(Some(#ts))
    });

    quote!(
        pub fn parsed() -> WgslShaderModuleUserPortion {
            WgslShaderModuleUserPortion {
                static_consts: [
                    #static_consts
                    ]
                .into(),
                helper_types: [
                    #helper_types
                    ]
                .into(),
                uniforms: Vec::from([
                   #uniforms2
                    ]),
                input_arrays: [
                    #input_arrays
                    ]
                .into(),
                output_arrays: [
                    #output_arrays
                    ]
                .into(),
                helper_functions: [
                    #helper_functions
                    ]
                .into(),
                // main_function: None,
                main_function: #main_function,
            }
        }
    )
}

#[cfg(test)]

mod test {
    use proc_macro_error::abort;
    use proc_macro2::{Span, TokenStream};

    #[test]
    pub fn test_parse_str() {
        let uniforms_str2 = "";
        let _uniforms2: TokenStream = if let Ok(ts) = uniforms_str2.parse() {
            ts
        } else {
            abort!(
                Span::call_site(),
                "Failed to parse uniforms into TokenStream"
            );
        };
    }
}

```

# src\transformer\output\types_for_rust_usage\make_types_pod.rs

```rs
use syn::{ItemStruct, parse_quote, visit_mut::VisitMut};

pub struct MakeTypesPodTransformer;

impl VisitMut for MakeTypesPodTransformer {
    /**
    Add the following as attributes:
    #[repr(C)]
    #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
    */
    fn visit_item_struct_mut(&mut self, i: &mut ItemStruct) {
        syn::visit_mut::visit_item_struct_mut(self, i);
        i.attrs.push(parse_quote! {
            #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
        });
        i.attrs.push(parse_quote! {
        #[repr(C)]});
    }
}

```

# src\transformer\output\types_for_rust_usage\make_types_public.rs

```rs
use syn::{ItemStruct, ItemType, Visibility, spanned::Spanned, token::Pub, visit_mut::VisitMut};

pub struct MakeTypesPublicTransformer;

impl VisitMut for MakeTypesPublicTransformer {
    fn visit_item_type_mut(&mut self, i: &mut ItemType) {
        syn::visit_mut::visit_item_type_mut(self, i);
        i.vis = Visibility::Public(Pub { span: i.span() });
    }
    fn visit_item_struct_mut(&mut self, i: &mut ItemStruct) {
        syn::visit_mut::visit_item_struct_mut(self, i);
        i.vis = Visibility::Public(Pub { span: i.span() });
    }
}

```

# src\transformer\output\types_for_rust_usage\mod.rs

```rs
pub mod make_types_pod;
pub mod make_types_public;
pub mod remove_internal_attributes;
pub mod types;

```

# src\transformer\output\types_for_rust_usage\remove_internal_attributes.rs

```rs
/// whenever `#[` is detected, remove everything up to the next `]` using regex, also remove the hash+square brackets
/// remove these specific strings: `#[wgsl_config]`, `#[wgsl_input_array]`, `#[wgsl_output_array]` and `#[wgsl_output_vec]`, but allow any number of whitespaces or newlines between the square brackets and the attribute name
pub fn remove_internal_attributes(file: String) -> String {
    let re = regex::Regex::new(r"#\[\s*wgsl_config\s*\]|\s*#\[\s*wgsl_input_array\s*\]|\s*#\[\s*wgsl_output_array\s*\]|\s*#\[\s*wgsl_output_vec\s*\]").unwrap();
    re.replace_all(&file, "").to_string()
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_remove_internal_attributes() {
        let input = r#"
        #[wgsl_config]
        #[wgsl_input_array]
        #[wgsl_output_array]
        #[wgsl_output_vec]
        #[valid]
        #[wgsl_config]#[wgsl_input_array]#[wgsl_output_array]#[wgsl_output_vec]
        "#;
        let expected = r#"

        #[valid]

        "#;
        let result = remove_internal_attributes(input.to_string());
        assert_eq!(result, expected);
    }
}

```

# src\transformer\output\types_for_rust_usage\types.rs

```rs
use proc_macro_error::abort;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use shared::wgsl_components::WgslShaderModuleUserPortion;
use syn::{Ident, parse2, visit_mut::VisitMut};

use crate::{
    state::ModuleTransformState,
    transformer::{
        custom_types::custom_type::CustomTypeKind,
        output::types_for_rust_usage::make_types_public::MakeTypesPublicTransformer,
    },
};

use super::{
    make_types_pod::MakeTypesPodTransformer, remove_internal_attributes::remove_internal_attributes,
};

pub fn define_types_for_use_in_rust(state: &ModuleTransformState) -> TokenStream {
    let user_types = user_defined_types(state);
    let uniforms: TokenStream = uniform_types(state);
    let input_arrays = input_array_types(state);
    let output_arrays = output_array_types(state);
    quote!(
        /// user types
    #user_types
        /// uniforms
    #uniforms
        /// input arrays
    #input_arrays
        /// output types
    #output_arrays
        /// public facing types for use by library



        pub struct Types;
        impl TypesSpec for Types {
            type InputConfigTypes = _InputConfigTypes;
            type InputArrayTypes = _InputArrayTypes;
            type OutputArrayTypes = _OutputArrayTypes;
        }

    )
}

pub fn user_defined_types(state: &ModuleTransformState) -> TokenStream {
    let mut publicifier = MakeTypesPublicTransformer {};
    let mut podifier = MakeTypesPodTransformer {};
    let custom_types = remove_internal_attributes(
        state
            .custom_types
            .as_ref()
            .unwrap()
            .iter()
            .map(|c| {
                // get item
                if c.kind == CustomTypeKind::ArrayLengthVariable {
                    return "".to_string();
                }
                let s = c.rust_code.clone();
                let mut item = parse2::<syn::Item>(s);
                if let Err(e) = item {
                    let message = format!(
                        "Error parsing custom type: {:?}, with custom type: {:?}",
                        e, c
                    );
                    abort!(Span::call_site(), message);
                }
                // make public
                publicifier.visit_item_mut(&mut item.as_mut().unwrap());
                podifier.visit_item_mut(&mut item.as_mut().unwrap());
                // stringify
                let string: String = item.unwrap().to_token_stream().to_string();
                string
            })
            .collect::<Vec<String>>()
            .join("\n"),
    );
    custom_types.parse().unwrap()
}

pub fn uniform_types(state: &ModuleTransformState) -> TokenStream {
    let obj: WgslShaderModuleUserPortion = state.result.clone();
    let uniforms = obj.uniforms;
    assert!(
        uniforms.len() <= 6,
        "Only a max of 6 input configs are supported"
    );
    let uniforms_as_idents: Vec<Ident> = uniforms
        .iter()
        .map(|array| Ident::new(&array.name.name(), Span::call_site()))
        .collect();
    let unused = Ident::new("_INTERNAL_UNUSED", Span::call_site());
    let t1: &Ident = uniforms_as_idents.get(0).unwrap_or(&unused);
    let t2: &Ident = uniforms_as_idents.get(1).unwrap_or(&unused);
    let t3: &Ident = uniforms_as_idents.get(2).unwrap_or(&unused);
    let t4: &Ident = uniforms_as_idents.get(3).unwrap_or(&unused);
    let t5: &Ident = uniforms_as_idents.get(4).unwrap_or(&unused);
    let t6: &Ident = uniforms_as_idents.get(5).unwrap_or(&unused);

    quote!(

         pub struct _InputConfigTypes {}
    impl InputConfigTypesSpec for _InputConfigTypes {
        type Input0 = #t1;
        type Input1 = #t2;
        type Input2 = #t3;
        type Input3 = #t4;
        type Input4 = #t5;
        type Input5 = #t6;
    })
}

pub fn input_array_types(state: &ModuleTransformState) -> TokenStream {
    let obj: WgslShaderModuleUserPortion = state.result.clone();
    let input_arrays = obj.input_arrays;
    assert!(
        input_arrays.len() <= 6,
        "Only a max of 6 input arrays are supported"
    );
    let input_arrays_as_idents: Vec<Ident> = input_arrays
        .iter()
        .map(|array| Ident::new(&array.item_type.name.name(), Span::call_site()))
        .collect();
    let unused = Ident::new("_INTERNAL_UNUSED", Span::call_site());
    let t1: &Ident = input_arrays_as_idents.get(0).unwrap_or(&unused);
    let t2: &Ident = input_arrays_as_idents.get(1).unwrap_or(&unused);
    let t3: &Ident = input_arrays_as_idents.get(2).unwrap_or(&unused);
    let t4: &Ident = input_arrays_as_idents.get(3).unwrap_or(&unused);
    let t5: &Ident = input_arrays_as_idents.get(4).unwrap_or(&unused);
    let t6: &Ident = input_arrays_as_idents.get(5).unwrap_or(&unused);

    quote!(

     pub struct _InputArrayTypes {}
    impl InputVectorTypesSpec for _InputArrayTypes {
        type Input0 = #t1;
        type Input1 = #t2;
        type Input2 = #t3;
        type Input3 = #t4;
        type Input4 = #t5;
        type Input5 = #t6;
    })
}

pub fn output_array_types(state: &ModuleTransformState) -> TokenStream {
    let obj: WgslShaderModuleUserPortion = state.result.clone();
    let output_arrays = obj.output_arrays;
    assert!(
        output_arrays.len() <= 6,
        "Only a max of 6 output arrays are supported"
    );
    let output_arrays_as_idents: Vec<Ident> = output_arrays
        .iter()
        .map(|array| Ident::new(&array.item_type.name.name(), Span::call_site()))
        .collect();
    let unused = Ident::new("_INTERNAL_UNUSED", Span::call_site());
    let t1: &Ident = output_arrays_as_idents.get(0).unwrap_or(&unused);
    let t2: &Ident = output_arrays_as_idents.get(1).unwrap_or(&unused);
    let t3: &Ident = output_arrays_as_idents.get(2).unwrap_or(&unused);
    let t4: &Ident = output_arrays_as_idents.get(3).unwrap_or(&unused);
    let t5: &Ident = output_arrays_as_idents.get(4).unwrap_or(&unused);
    let t6: &Ident = output_arrays_as_idents.get(5).unwrap_or(&unused);

    quote!(

        pub struct _OutputArrayTypes {}
        impl OutputVectorTypesSpec for _OutputArrayTypes {
            type Output0 = #t1;
            type Output1 = #t2;
            type Output2 = #t3;
            type Output3 = #t4;
            type Output4 = #t5;
            type Output5 = #t6;
        }
    )
}

```

# src\transformer\output\unaltered_module.rs

```rs
use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::quote;

use crate::state::ModuleTransformState;
pub fn generate_unaltered_module(state: &ModuleTransformState) -> TokenStream {
    let module_ident: TokenStream = if let Some(c) = &state.module_ident {
        format!("{}_for_syntax_check", c).parse().unwrap()
    } else {
        abort!(
            state.rust_module.ident.span(),
            "No module ident found in transform state"
        );
    };
    let module_visibility: TokenStream = if let Some(c) = &state.module_visibility {
        c.parse().unwrap()
    } else {
        abort!(
            state.rust_module.ident.span(),
            "No module visibility found in transform state"
        );
    };
    let content: TokenStream = state.get_original_content().parse().unwrap();
    quote!(
    #module_visibility mod #module_ident {
        #content
    })
}

```

# src\transformer\remove_doc_comments.rs

```rs
use regex::Regex;

/// Remove doc comments from rust source code
///
/// # Arguments
/// * `source` - The source code to process as a string
///
/// # Returns
/// * The processed source code with comments removed
pub fn remove_doc_comments(source: &str) -> String {
    let mut content = String::from(source);
    content = remove_doc_block_comments(&content);
    content = remove_doc_singleline_comments(&content);
    content
}

/// Remove documentation block comments (/** */ and /*! */) from source code
fn remove_doc_block_comments(source: &str) -> String {
    let doc_re = Regex::new(r"/\*[\*!].*?\*/").unwrap();
    doc_re.replace_all(source, "").to_string()
}

/// Remove documentation single line comments (/// and //!) from source code
fn remove_doc_singleline_comments(source: &str) -> String {
    let doc_re = Regex::new(r"(?m)///.*$|//!.*$").unwrap();
    doc_re.replace_all(source, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_doc_block_comments() {
        let input = "/** This is doc comment */\nfn test() {}\n/*! Module doc */";
        let expected = "\nfn test() {}\n";
        assert_eq!(remove_doc_block_comments(input), expected);
    }

    #[test]
    fn test_remove_doc_singleline_comments() {
        let input = "/// Doc comment\n//! Module doc\nfn test() {}";
        let expected = "\n\nfn test() {}";
        assert_eq!(remove_doc_singleline_comments(input), expected);
    }
    #[test]
    fn remove_both() {
        let input = "//! Module doc\n/** This is doc comment */\nfn test() {}\n/*! Module doc */";
        let expected = "\n\nfn test() {}\n";
        assert_eq!(remove_doc_comments(input), expected);
    }
}

```

# src\transformer\to_wgsl_syntax\array.rs

```rs
use proc_macro_error::abort;
use syn::{parse_quote, spanned::Spanned, visit_mut::VisitMut};

pub struct ArrayToWgslTransformer {}

impl VisitMut for ArrayToWgslTransformer {
    fn visit_item_type_mut(&mut self, t: &mut syn::ItemType) {
        syn::visit_mut::visit_item_type_mut(self, t);
        match *t.ty.clone() {
            syn::Type::Array(arr) => {
                let type_path = array_to_wgsl(&arr);
                *t.ty = syn::Type::Path(type_path);
            }
            _ => (),
        }
    }
    fn visit_pat_type_mut(&mut self, t: &mut syn::PatType) {
        syn::visit_mut::visit_pat_type_mut(self, t);
        match *t.ty.clone() {
            syn::Type::Array(arr) => {
                let type_path = array_to_wgsl(&arr);
                *t.ty = syn::Type::Path(type_path);
            }
            _ => (),
        }
    }
}

pub fn array_to_wgsl(arr: &syn::TypeArray) -> syn::TypePath {
    let ident = match *arr.elem.clone() {
        syn::Type::Path(p) => {
            if let Some(f) = p.path.segments.first() {
                f.ident.clone()
            } else {
                abort!(arr.elem.span(), "Array element type is not a path")
            }
        }
        _ => abort!(arr.elem.span(), "Array element type is not a path"),
    };
    let len = arr.len.clone();

    return parse_quote!(array<#ident,#len>);
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;
    use syn::{TypeArray, parse_quote};

    #[test]
    fn test_array_to_wgsl() {
        let input: TypeArray = parse_quote! { [f32; 4] };
        let output = array_to_wgsl(&input);
        assert_eq!(output.to_token_stream().to_string(), "array < f32 , 4 >");
    }
}

```

# src\transformer\to_wgsl_syntax\expr.rs

```rs
use proc_macro_error::abort;
use quote::ToTokens;
use syn::{Expr, ExprCall, parse2, spanned::Spanned, visit_mut::VisitMut};

use crate::transformer::allowed_types::WGSL_NATIVE_TYPES;

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
                expr.to_token_stream().to_string()
            );
            abort!(expr.span(), message)
        }
    }
}

```

# src\transformer\to_wgsl_syntax\mod.rs

```rs
use std::collections::HashMap;

use array::ArrayToWgslTransformer;
use expr::ExprToWgslTransformer;
use proc_macro_error::abort;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use remove_attributes::remove_attributes;
use remove_pub_from_struct_def::PubRemover;
use syn::{File, parse, visit::Visit, visit_mut::VisitMut};
use r#type::TypeToWgslTransformer;
use type_def::TypeDefToWgslTransformer;
use wgsl_builtin_constructors::convert_wgsl_builtin_constructors;

use crate::state::ModuleTransformState;

/**
 # Notes about conversions (all syntax not mentioned is either the same or not supported in wgsl)

- ForLoop(ExprForLoop):
  in wgsl, but with javascript style syntax: for (var i = 0; i< 10; i++){}

- Loop(ExprLoop):
  supported in wgsl, but with different syntax: `for (;;) {}`

- Reference(ExprReference):
  support pointer types, but this is something for a future version. Example of pointers in wgsl:
  \`\`\`ignore
  fn my_function(
      /* 'ptr<function,i32,read_write>' is the type of a pointer value that references
         memory for keeping an 'i32' value, using memory locations in the 'function'
         address space.  Here 'i32' is the store type.
         The implied access mode is 'read_write'.
         See "Address Space" section for defaults. */
      ptr_int: ptr<function,i32>,

      /* 'ptr<private,array<f32,50>,read_write>' is the type of a pointer value that
       refers to memory for keeping an array of 50 elements of type 'f32', using
       memory locations in the 'private' address space.
       Here the store type is 'array<f32,50>'.
       The implied access mode is 'read_write'.
       See the "Address space section for defaults.
      */
      ptr_array: ptr<private, array<f32, 50>>
    ) { }
  \`\`\`

- Struct(ExprStruct):
  supported, different syntax. in wgsl it becomes `Point(1,1)`, but we must warn the user that
  the order that they list the fields in when constructing their struct MUST be the same order
  that they are listed in when defining the struct type

- Array(ExprArray):
    supported, but with different syntax. in wgsl it becomes `array<f32, 3>`

- Types:
    - f32, f16, i32, u32, bool, vec2, vec3, vec4, mat2x2, mat3x3, mat4x4
  */
mod array;
mod expr;
pub mod remove_attributes;
mod remove_pub_from_struct_def;
mod r#type;
mod type_def;
mod wgsl_builtin_constructors;
/// called_from is for debug messages
pub fn convert_file_to_wgsl(
    input: TokenStream,
    state: &ModuleTransformState,
    called_from: String,
) -> String {
    let unprocessed_string = input.to_string();
    let input_no_attributes = remove_attributes(unprocessed_string);
    let processed_stream: TokenStream = input_no_attributes.parse().unwrap();
    let debug_str = processed_stream.clone().to_string();

    let mut file = if let Ok(f) = parse::<File>(processed_stream.into()) {
        f
    } else {
        let message = format!(
            "Failed to parse file, in convert_to_wgsl, from {}. File:{}",
            called_from, debug_str
        );
        abort!(Span::call_site(), message);
    };

    let custom_types = if let Some(ct) = &state.custom_types {
        ct
    } else {
        abort!(
            state.rust_module.ident.span(),
            "Allowed types must be set before converting to wgsl"
        );
    };
    PubRemover {}.visit_file_mut(&mut file);
    TypeToWgslTransformer {
        custom_types: &custom_types,
    }
    .visit_file_mut(&mut file);
    ArrayToWgslTransformer {}.visit_file_mut(&mut file);
    ExprToWgslTransformer {}.visit_file_mut(&mut file);
    // expressions and type defs have to be transformed differently because they may change the token structure, so we have to transition to strings
    let mut string_version = file.to_token_stream().to_string();
    // transform expressions
    // let mut expression_transformer = ExprToWgslTransformer {
    // replacements: HashMap::new(),
    // };
    // expression_transformer.visit_file(&file);
    // println!("Going to replace...");
    // expression_transformer
    // .replacements
    // .iter()
    // .for_each(|(k, v)| {
    // println!("Replacing: {} with: {}", k, v);
    // string_version = string_version.replace(k, v);
    // });
    // transform type defs (should not conflict with other replacements)
    let mut type_def_transformer = TypeDefToWgslTransformer {
        replacements: HashMap::new(),
    };
    type_def_transformer.visit_file(&file);
    type_def_transformer.replacements.iter().for_each(|(k, v)| {
        string_version = string_version.replace(k, v);
    });
    println!("Final string version: {}", string_version);
    // transform vec and matrix constructors
    string_version = convert_wgsl_builtin_constructors(string_version);
    string_version
}

```

# src\transformer\to_wgsl_syntax\remove_attributes.rs

```rs
/// whenever `#[` is detected, remove everything up to the next `]` using regex, also remove the hash+square brackets
pub fn remove_attributes(file: String) -> String {
    let re = regex::Regex::new(r"#\[[^\]]*\]").unwrap();
    re.replace_all(&file, "").to_string()
}

```

# src\transformer\to_wgsl_syntax\remove_pub_from_struct_def.rs

```rs
use syn::{Field, Visibility, visit_mut::VisitMut};

pub struct PubRemover {}

impl VisitMut for PubRemover {
    fn visit_field_mut(&mut self, i: &mut Field) {
        syn::visit_mut::visit_field_mut(self, i);
        i.vis = Visibility::Inherited;
    }
}

```

# src\transformer\to_wgsl_syntax\type_def.rs

```rs
use std::collections::HashMap;

use proc_macro_error::abort;
use quote::ToTokens;
use syn::{ItemType, visit::Visit};

pub struct TypeDefToWgslTransformer {
    pub replacements: HashMap<String, String>,
}

impl<'ast> Visit<'ast> for TypeDefToWgslTransformer {
    fn visit_item_type(&mut self, t: &syn::ItemType) {
        syn::visit::visit_item_type(self, t);
        // Instead of direct replacement, use placeholder system
        let new_type_def = type_def_to_wgsl(t);
        let existing = t.to_token_stream().to_string();
        // extract everything to the left of the first =
        let expr = existing.split('=').collect::<Vec<&str>>()[0]
            .trim()
            .to_string();
        self.replacements.insert(expr, new_type_def);
    }
}

const UNALLOWED_TYPES_FOR_RENAMING: [&str; 12] = [
    "vec2", "vec3", "vec4", "mat2x2", "mat2x3", "mat2x4", "mat3x2", "mat3x3", "mat3x4", "mat4x2",
    "mat4x3", "mat4x4",
];
fn type_def_to_wgsl(type_def: &ItemType) -> String {
    // ensure that the type is not a custom type
    match *type_def.ty.clone() {
        syn::Type::Path(p) => {
            if let Some(f) = p.path.segments.first() {
                let mtch = UNALLOWED_TYPES_FOR_RENAMING
                    .iter()
                    .find(|t| **t == f.ident.to_string());
                if mtch.is_some() {
                    abort!(
                        f.ident.span(),
                        "Renaming/aliasing helper types like Vec3F32, Mat2x2Bool, etc. is not supported. For example don't do `type MyType = Vec3U32;`. Instead put it in a struct field like `struct MyType = { v: Vec3U32 }`"
                    );
                }
            }
        }
        _ => (),
    }
    let s = format!("alias {} ", type_def.ident.to_string(),);
    s
}

```

# src\transformer\to_wgsl_syntax\type.rs

```rs
use proc_macro_error::abort;
use syn::{PathSegment, parse_quote, visit_mut::VisitMut};

use crate::transformer::custom_types::custom_type::CustomType;

pub struct TypeToWgslTransformer<'a> {
    pub custom_types: &'a Vec<CustomType>,
}
impl<'a> VisitMut for TypeToWgslTransformer<'a> {
    fn visit_type_path_mut(&mut self, mut t: &mut syn::TypePath) {
        syn::visit_mut::visit_type_path_mut(self, t);
        path_type_to_wgsl(&mut t, &self.custom_types);
    }
}

pub fn path_type_to_wgsl<'a>(type_path: &mut syn::TypePath, custom_types: &Vec<CustomType>) {
    let path = &mut type_path.path;
    let segments = &mut path.segments;
    for segment in segments.iter_mut() {
        let new_segment = convert_path_segment(segment.clone(), custom_types);
        *segment = new_segment;
    }
}
fn convert_path_segment(segment: PathSegment, custom_types: &Vec<CustomType>) -> PathSegment {
    let ident = &segment.ident;
    let custom_t = custom_types.iter().find(|t| t.name.eq(&ident));
    if let Some(_) = custom_t {
        segment.clone()
    } else {
        match ident.to_string().as_str() {
            "atomic" => segment.clone(),
            "array" => segment.clone(),
            "f32" => segment.clone(),
            "i32" => segment.clone(),
            "u32" => segment.clone(),
            "PodF16" => parse_quote!(f16),
            "f16" => {
                abort!(
                    ident.span(),
                    "Standard rust f16s are not \"Pods\", use `PodF16` instead of `f16`. This is because we use `bytemuck` for creating and reading GPU buffers."
                )
            }
            "bool" => segment.clone(),
            "vec3" => segment.clone(),
            "vec2" => segment.clone(),
            "vec4" => segment.clone(),
            "mat2x2" => segment.clone(),
            "mat3x3" => segment.clone(),
            "mat4x4" => segment.clone(),
            "mat2x3" => segment.clone(),
            "mat2x4" => segment.clone(),
            "mat3x2" => segment.clone(),
            "mat3x4" => segment.clone(),
            "mat4x2" => segment.clone(),
            "mat4x3" => segment.clone(),
            "WgslIterationPosition" => segment.clone(),
            "Vec2I32" => parse_quote!(vec2<i32>),
            "Vec2U32" => parse_quote!(vec2<u32>),
            "Vec2F32" => parse_quote!(vec2<f32>),
            "Vec2F16" => parse_quote!(vec2<f16>),
            "Vec2Bool" => parse_quote!(vec2<bool>),
            "Vec3I32" => parse_quote!(vec3<i32>),
            "Vec3U32" => parse_quote!(vec3<u32>),
            "Vec3F32" => parse_quote!(vec3<f32>),
            "Vec3F16" => parse_quote!(vec3<f16>),
            "Vec3Bool" => parse_quote!(vec3<bool>),
            "Vec4I32" => parse_quote!(vec4<i32>),
            "Vec4U32" => parse_quote!(vec4<u32>),
            "Vec4F32" => parse_quote!(vec4<f32>),
            "Vec4F16" => parse_quote!(vec4<f16>),
            "Vec4Bool" => parse_quote!(vec4<bool>),
            "Mat2x2I32" => parse_quote!(mat2x2<i32>),
            "Mat2x2U32" => parse_quote!(mat2x2<u32>),
            "Mat2x2F32" => parse_quote!(mat2x2<f32>),
            "Mat2x2F16" => parse_quote!(mat2x2<f16>),
            "Mat2x2Bool" => parse_quote!(mat2x2<bool>),
            "Mat2x3I32" => parse_quote!(mat2x3<i32>),
            "Mat2x3U32" => parse_quote!(mat2x3<u32>),
            "Mat2x3F32" => parse_quote!(mat2x3<f32>),
            "Mat2x3F16" => parse_quote!(mat2x3<f16>),
            "Mat2x3Bool" => parse_quote!(mat2x3<bool>),
            "Mat2x4I32" => parse_quote!(mat2x4<i32>),
            "Mat2x4U32" => parse_quote!(mat2x4<u32>),
            "Mat2x4F32" => parse_quote!(mat2x4<f32>),
            "Mat2x4F16" => parse_quote!(mat2x4<f16>),
            "Mat2x4Bool" => parse_quote!(mat2x4<bool>),
            "Mat3x2I32" => parse_quote!(mat3x2<i32>),
            "Mat3x2U32" => parse_quote!(mat3x2<u32>),
            "Mat3x2F32" => parse_quote!(mat3x2<f32>),
            "Mat3x2F16" => parse_quote!(mat3x2<f16>),
            "Mat3x2Bool" => parse_quote!(mat3x2<bool>),
            "Mat3x3I32" => parse_quote!(mat3x3<i32>),
            "Mat3x3U32" => parse_quote!(mat3x3<u32>),
            "Mat3x3F32" => parse_quote!(mat3x3<f32>),
            "Mat3x3F16" => parse_quote!(mat3x3<f16>),
            "Mat3x3Bool" => parse_quote!(mat3x3<bool>),
            "Mat3x4I32" => parse_quote!(mat3x4<i32>),
            "Mat3x4U32" => parse_quote!(mat3x4<u32>),
            "Mat3x4F32" => parse_quote!(mat3x4<f32>),
            "Mat3x4F16" => parse_quote!(mat3x4<f16>),
            "Mat3x4Bool" => parse_quote!(mat3x4<bool>),
            "Mat4x2I32" => parse_quote!(mat4x2<i32>),
            "Mat4x2U32" => parse_quote!(mat4x2<u32>),
            "Mat4x2F32" => parse_quote!(mat4x2<f32>),
            "Mat4x2F16" => parse_quote!(mat4x2<f16>),
            "Mat4x2Bool" => parse_quote!(mat4x2<bool>),
            "Mat4x3I32" => parse_quote!(mat4x3<i32>),
            "Mat4x3U32" => parse_quote!(mat4x3<u32>),
            "Mat4x3F32" => parse_quote!(mat4x3<f32>),
            "Mat4x3F16" => parse_quote!(mat4x3<f16>),
            "Mat4x3Bool" => parse_quote!(mat4x3<bool>),
            "Mat4x4I32" => parse_quote!(mat4x4<i32>),
            "Mat4x4U32" => parse_quote!(mat4x4<u32>),
            "Mat4x4F32" => parse_quote!(mat4x4<f32>),
            "Mat4x4F16" => parse_quote!(mat4x4<f16>),
            "Mat4x4Bool" => parse_quote!(mat4x4<bool>),

            _ => {
                let message = format!("Unsupported type in type_to_wgsl: {}", ident.to_string());
                abort!(ident.span(), message)
            }
        }
    }
}

```

# src\transformer\to_wgsl_syntax\wgsl_builtin_constructors.rs

```rs
use regex::Regex;

/**  for converting vec and mat types
*Vectors are declared with the form vecN<T> , where N is the number of elements in the vector, and T is the element type.
* Matrices are declared with the form matCxR<f32> , where C is the number of columns in the matrix, R is the number of rows in the matrix.
* vectors are accesed like arrays ... my_vec = vec3(1.0, 2.0, 3.0); my_vec[0] = 1.0;
* matrices are accesed wierdly: my_mat = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0)); my_mat[1,0] = 3.0;

## creation:
 Vec3I32::new(1, 2, 3) => vec3<i32>(1, 2, 3)
 Mat4x2Bool::new(Vec2Bool::new(true, false), Vec2Bool::new(false, true), Vec2Bool::new(true, true), Vec2Bool::new(false, false)) => mat4x2<bool>(vec2<bool>(true, false), vec2<bool>(false, true), vec2<bool>(true, true), vec2<bool>(false, false))

## creation if conversion already happened:
 vec3<i32>::new(1, 2, 3) => vec3<i32>(1, 2, 3)
    mat4x2<bool>::new(vec2<bool>::new(true, false), vec2<bool>::new(false, true), vec2<bool>::new(true, true), vec2<bool>::new(false, false)) => mat4x2<bool>::new(vec2<bool>(true, false), vec2<bool>::new(false, true), vec2<bool>::new(true, true), vec2<bool>::new(false, false))

we have these different type conversions:
 "Vec2I32" => parse_quote!(vec2<i32>),
            "Vec2U32" => parse_quote!(vec3<u32>),
            "Vec2F32" => parse_quote!(vec3<f32>),
            "Vec2F16" => parse_quote!(vec3<f16>),
            "Vec2Bool" => parse_quote!(vec3<bool>),
            "Vec3I32" => parse_quote!(vec3<i32>),
            "Vec3U32" => parse_quote!(vec3<u32>),
            "Vec3F32" => parse_quote!(vec3<f32>),
            "Vec3F16" => parse_quote!(vec3<f16>),
            "Vec3Bool" => parse_quote!(vec3<bool>),
            "Vec4I32" => parse_quote!(vec4<i32>),
            "Vec4U32" => parse_quote!(vec4<u32>),
            "Vec4F32" => parse_quote!(vec4<f32>),
            "Vec4F16" => parse_quote!(vec4<f16>),
            "Vec4Bool" => parse_quote!(vec4<bool>),
            "Mat2x2I32" => parse_quote!(mat2x2<i32>),
            "Mat2x2U32" => parse_quote!(mat2x2<u32>),
            "Mat2x2F32" => parse_quote!(mat2x2<f32>),
            "Mat2x2F16" => parse_quote!(mat2x2<f16>),
            "Mat2x2Bool" => parse_quote!(mat2x2<bool>),
            "Mat2x3I32" => parse_quote!(mat3x3<i32>),
            "Mat2x3U32" => parse_quote!(mat3x3<u32>),
            "Mat2x3F32" => parse_quote!(mat3x3<f32>),
            "Mat2x3F16" => parse_quote!(mat3x3<f16>),
            "Mat2x3Bool" => parse_quote!(mat3x3<bool>),
            "Mat2x4I32" => parse_quote!(mat2x4<i32>),
            "Mat2x4U32" => parse_quote!(mat2x4<u32>),
            "Mat2x4F32" => parse_quote!(mat2x4<f32>),
            "Mat2x4F16" => parse_quote!(mat2x4<f16>),
            "Mat2x4Bool" => parse_quote!(mat2x4<bool>),
            "Mat3x2I32" => parse_quote!(mat3x2<i32>),
            "Mat3x2U32" => parse_quote!(mat3x2<u32>),
            "Mat3x2F32" => parse_quote!(mat3x2<f32>),
            "Mat3x2F16" => parse_quote!(mat3x2<f16>),
            "Mat3x2Bool" => parse_quote!(mat3x2<bool>),
            "Mat3x3I32" => parse_quote!(mat3x3<i32>),
            "Mat3x3U32" => parse_quote!(mat3x3<u32>),
            "Mat3x3F32" => parse_quote!(mat3x3<f32>),
            "Mat3x3F16" => parse_quote!(mat3x3<f16>),
            "Mat3x3Bool" => parse_quote!(mat3x3<bool>),
            "Mat3x4I32" => parse_quote!(mat3x4<i32>),
            "Mat3x4U32" => parse_quote!(mat3x4<u32>),
            "Mat3x4F32" => parse_quote!(mat3x4<f32>),
            "Mat3x4F16" => parse_quote!(mat3x4<f16>),
            "Mat3x4Bool" => parse_quote!(mat3x4<bool>),
            "Mat4x2I32" => parse_quote!(mat4x2<i32>),
            "Mat4x2U32" => parse_quote!(mat4x2<u32>),
            "Mat4x2F32" => parse_quote!(mat4x2<f32>),
            "Mat4x2F16" => parse_quote!(mat4x2<f16>),
            "Mat4x2Bool" => parse_quote!(mat4x2<bool>),
            "Mat4x3I32" => parse_quote!(mat4x3<i32>),
            "Mat4x3U32" => parse_quote!(mat4x3<u32>),
            "Mat4x3F32" => parse_quote!(mat4x3<f32>),
            "Mat4x3F16" => parse_quote!(mat4x3<f16>),
            "Mat4x3Bool" => parse_quote!(mat4x3<bool>),
            "Mat4x4I32" => parse_quote!(mat4x4<i32>),
            "Mat4x4U32" => parse_quote!(mat4x4<u32>),
            "Mat4x4F32" => parse_quote!(mat4x4<f32>),
            "Mat4x4F16" => parse_quote!(mat4x4<f16>),
            "Mat4x4Bool" => parse_quote!(mat4x4<bool>),


# access:
NO CHANGE TO ACCESS


*/
pub fn convert_wgsl_builtin_constructors(wgsl_code: String) -> String {
    process_string_recursively(&wgsl_code)
}

fn convert_single_constructor(input: &str) -> Option<(String, bool)> {
    // Modified regex to handle whitespace around ::
    let re = Regex::new(r"(?:(Vec|Mat)([234])(?:x([234]))?((?:I32|U32|F32|F16|Bool))|(?:vec|mat)([234])(?:x([234]))?\s*<\s*((?:i32|u32|f32|f16|bool))\s*>)\s*::\s*new").unwrap();

    if let Some(caps) = re.captures(input) {
        // Rest of the function remains the same
        if let (Some(type_kind), Some(first_dim), second_dim, Some(type_str)) =
            (caps.get(1), caps.get(2), caps.get(3), caps.get(4))
        {
            let type_str = match type_str.as_str() {
                "I32" => "i32",
                "U32" => "u32",
                "F32" => "f32",
                "F16" => "f16",
                "Bool" => "bool",
                _ => panic!("Unsupported type"),
            };

            let prefix = match type_kind.as_str() {
                "Vec" => format!("vec{}<{}>", first_dim.as_str(), type_str),
                "Mat" => format!(
                    "mat{}x{}<{}>",
                    first_dim.as_str(),
                    second_dim.map_or(first_dim.as_str(), |m| m.as_str()),
                    type_str
                ),
                _ => unreachable!(),
            };
            Some((prefix, true))
        }
        // Handle WGSL format (vec3<f32>)
        else if let (Some(first_dim), second_dim, Some(type_str)) =
            (caps.get(5), caps.get(6), caps.get(7))
        {
            let prefix = if second_dim.is_some() {
                format!(
                    "mat{}x{}<{}>",
                    first_dim.as_str(),
                    second_dim.unwrap().as_str(),
                    type_str.as_str()
                )
            } else {
                format!("vec{}<{}>", first_dim.as_str(), type_str.as_str())
            };
            Some((prefix, true))
        } else {
            None
        }
    } else {
        None
    }
}
fn process_string_recursively(input: &str) -> String {
    let mut result = String::new();
    let mut current_constructor = String::new();
    let mut current_args = String::new();
    let mut paren_depth = 0;
    let mut pos = 0;
    let chars: Vec<char> = input.chars().collect();

    while pos < chars.len() {
        let c = chars[pos];
        match c {
            '(' => {
                paren_depth += 1;
                if paren_depth == 1 {
                    // Process any constructor before the opening parenthesis
                    if !current_constructor.is_empty() {
                        if let Some((prefix, _)) =
                            convert_single_constructor(&current_constructor.trim())
                        {
                            result.push_str(&prefix);
                        } else {
                            result.push_str(&current_constructor);
                        }
                        current_constructor.clear();
                    }
                    result.push(c);
                } else {
                    current_args.push(c);
                }
            }
            ')' => {
                if paren_depth == 1 {
                    // Process accumulated arguments
                    if !current_args.is_empty() {
                        let processed = process_string_recursively(&current_args);
                        result.push_str(&processed);
                        current_args.clear();
                    }
                    result.push(c);
                    paren_depth -= 1;
                } else {
                    current_args.push(c);
                    paren_depth -= 1;
                }
            }
            ' ' | '\t' | '\n' => {
                if paren_depth == 0 {
                    // Instead of processing immediately, add the whitespace to the constructor
                    current_constructor.push(c);
                } else {
                    current_args.push(c);
                }
            }
            ',' => {
                if paren_depth == 0 {
                    if !current_constructor.is_empty() {
                        if let Some((prefix, _)) =
                            convert_single_constructor(&current_constructor.trim())
                        {
                            result.push_str(&prefix);
                        } else {
                            result.push_str(&current_constructor);
                        }
                        current_constructor.clear();
                    }
                    result.push(c);
                } else {
                    current_args.push(c);
                }
            }
            _ => {
                if paren_depth == 0 {
                    current_constructor.push(c);
                } else {
                    current_args.push(c);
                }
            }
        }
        pos += 1;
    }

    // Handle any remaining content
    if !current_constructor.is_empty() {
        if let Some((prefix, _)) = convert_single_constructor(&current_constructor.trim()) {
            result.push_str(&prefix);
        } else {
            result.push_str(&current_constructor);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_conversion() {
        let input = "Vec3F32::new(1.0, 2.0, 3.0)".to_string();
        let expected = "vec3<f32>(1.0, 2.0, 3.0)";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);
    }

    #[test]
    fn test_vector_conversion_bool() {
        let input = "Vec3Bool::new(false, true, false)".to_string();
        let expected = "vec3<bool>(false, true, false)";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);
    }

    #[test]
    fn test_matrix_conversion() {
        let input = "Mat2x2F32::new(Vec2F32::new(1.0, 2.0), Vec2F32::new(3.0, 4.0))".to_string();
        let expected = "mat2x2<f32>(vec2<f32>(1.0, 2.0),vec2<f32>(3.0, 4.0))";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);
    }
    #[test]
    fn test_matrix_conversion2() {
        let input = "Mat3x4Bool::new(Vec4Bool::new(Vec2Bool::new(true, false)))".to_string();
        let expected = "mat3x4<bool>(vec4<bool>(vec2<bool>(true, false)))";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);
    }
    #[test]
    fn test_already_converted_types() {
        let input = "vec3<f32>::new(1.0, 2.0, 3.0)".to_string();
        let expected = "vec3<f32>(1.0, 2.0, 3.0)";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);

        let input = "mat2x2<bool>::new(vec2<bool>::new(true, false), vec2<bool>::new(false, true))"
            .to_string();
        let expected = "mat2x2<bool>(vec2<bool>(true, false),vec2<bool>(false, true))";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);
    }
    #[test]
    fn test_triple_nested_conversion() {
        let input = "Mat3x4Bool::new(Vec4Bool::new(Vec2Bool::new(true, false), Vec2Bool::new(false, true), Vec2Bool::new(true, true), Vec2Bool::new(false, false)), Vec4Bool::new(Vec2Bool::new(true, false), Vec2Bool::new(false, true), Vec2Bool::new(true, true), Vec2Bool::new(false, false)), Vec4Bool::new(Vec2Bool::new(true, false), Vec2Bool::new(false, true), Vec2Bool::new(true, true), Vec2Bool::new(false, false)))"
            .to_string();
        let expected = "mat3x4<bool>(vec4<bool>(vec2<bool>(true, false),vec2<bool>(false, true),vec2<bool>(true, true),vec2<bool>(false, false)),vec4<bool>(vec2<bool>(true, false),vec2<bool>(false, true),vec2<bool>(true, true),vec2<bool>(false, false)),vec4<bool>(vec2<bool>(true, false),vec2<bool>(false, true),vec2<bool>(true, true),vec2<bool>(false, false)))";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);
    }

    #[test]
    fn test_nested_vector_conversion() {
        let input = "Mat2x2F32::new(Vec2F32::new(1.0, Vec2F32::new(2.0)), Vec2F32::new(3.0, 4.0))"
            .to_string();
        let expected = "mat2x2<f32>(vec2<f32>(1.0,vec2<f32>(2.0)),vec2<f32>(3.0, 4.0))";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);
    }
    #[test]
    fn test_partial_nested() {
        let input =
            "Mat2x2F32::new(Vec2F32::new(1.0, Vec2F32::new(2.0)), vec2<f32>(3.0, 4.0))".to_string();
        let expected = "mat2x2<f32>(vec2<f32>(1.0,vec2<f32>(2.0)), vec2<f32>(3.0, 4.0))";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);
    }

    #[test]
    fn test_complex_string_conversion() {
        let partially_converted_input = "fn main(){
        struct MyStruct { x: Vec3F32, y: Mat2x4Bool };
        struct MyOuterStruct { z: MyStruct };
        let obj = MyOuterStruct ( MyStruct ( Vec3F32::new(1.0, 2.0, 3.0), mat2x4<bool>::new(Vec2Bool::new(true, false), Vec2Bool::new(false, true), vec2<bool>::new(true, true), Vec2Bool::new(false, false)) ) ); }"
        .to_string();
        let expected_output = "fn main(){
        struct MyStruct { x: Vec3F32, y: Mat2x4Bool };
        struct MyOuterStruct { z: MyStruct };
        let obj = MyOuterStruct ( MyStruct (vec3<f32>(1.0, 2.0, 3.0),mat2x4<bool>(vec2<bool>(true, false),vec2<bool>(false, true),vec2<bool>(true, true),vec2<bool>(false, false)) ) ); }".to_string();
        let output = convert_wgsl_builtin_constructors(partially_converted_input);
        assert_eq!(output, expected_output);
    }
    #[test]
    fn test_whole_module() {
        let input = "fn main(@builtin(global_invocation_id) global_id:vec3<u32>) {
                let obj = TStruct (
                    1.0,
                    Vec3F32 :: new(2.0, 3.0, 4.0)
                ); return;
                }";
        let expected = "fn main(@builtin(global_invocation_id) global_id:vec3<u32>) {
                let obj = TStruct (
                    1.0,vec3<f32>(2.0, 3.0, 4.0)
                ); return;
                }";
        assert_eq!(
            convert_wgsl_builtin_constructors(input.to_string()),
            expected
        );
    }
}

```

# src\transformer\transform_wgsl_helper_methods\category.rs

```rs
use syn::Ident;

pub enum WgslHelperCategory {
    VecInput,
    Output,
    ConfigInput,
    _Invalid,
}
// from ident
impl WgslHelperCategory {
    pub fn from_ident(ident: Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "WgslVecInput" => Some(WgslHelperCategory::VecInput),
            "WgslOutput" => Some(WgslHelperCategory::Output),
            "WgslConfigInput" => Some(WgslHelperCategory::ConfigInput),
            _ => None,
        }
    }
}

```

# src\transformer\transform_wgsl_helper_methods\helper_method.rs

```rs
use syn::Expr;

use crate::transformer::custom_types::custom_type::CustomType;

use super::{
    category::WgslHelperCategory, method_name::WgslHelperMethodName,
    to_expanded_format::ToExpandedFormatMethodKind,
};

pub struct WgslHelperMethod<'a> {
    pub category: WgslHelperCategory,
    pub method: WgslHelperMethodName,
    pub t_def: &'a CustomType,
    pub arg1: Option<&'a Expr>,
    pub arg2: Option<&'a Expr>,
    pub method_expander_kind: Option<ToExpandedFormatMethodKind>,
}

```

# src\transformer\transform_wgsl_helper_methods\matcher.rs

```rs
use crate::transformer::{
    custom_types::custom_type::CustomTypeKind,
    transform_wgsl_helper_methods::to_expanded_format::ToExpandedFormatMethodKind,
};

use super::{
    category::WgslHelperCategory, helper_method::WgslHelperMethod,
    method_name::WgslHelperMethodName,
};

pub struct WgslHelperMethodMatcher {}
impl WgslHelperMethodMatcher {
    pub fn choose_expand_format(method: &mut WgslHelperMethod) {
        match (&method.category, &method.method) {
            (WgslHelperCategory::ConfigInput, WgslHelperMethodName::Get) => {
                assert!(
                    method.t_def.kind == CustomTypeKind::Uniform,
                    "Expected {} to be an input config type, since WgslConfigInput::get is called, instead found it was of type {:?}. Put #[wgsl_config] above your type declaration to fix this. A given type cannot be used for multiple purposes, for example a type T cannot be both a config and a input array.",
                    method.t_def.name.name,
                    method.t_def.kind
                );
                method.method_expander_kind = Some(ToExpandedFormatMethodKind::ConfigGet);
            }
            (WgslHelperCategory::VecInput, WgslHelperMethodName::VecLen) => {
                assert!(
                    method.t_def.kind == CustomTypeKind::InputArray,
                    "Expected {} to be an input array type, since WgslVecInput::vec_len is called, instead found it was of type {:?}. Put #[wgsl_input_array] above your type declaration to fix this. A given type cannot be used for multiple purposes, for example a type T cannot be both an input array and an output array.",
                    method.t_def.name.name,
                    method.t_def.kind
                );
                method.method_expander_kind = Some(ToExpandedFormatMethodKind::InputLen);
            }
            (WgslHelperCategory::VecInput, WgslHelperMethodName::VecVal) => {
                assert!(
                    method.arg1.is_some(),
                    "Expected an argument for input vec value getter"
                );
                assert!(
                    method.t_def.kind == CustomTypeKind::InputArray,
                    "Expected {} to be an input array type, since WgslVecInput::vec_val is called, instead found it was of type {:?}. Put #[wgsl_input_array] above your type declaration to fix this. A given type cannot be used for multiple purposes, for example a type T cannot be both an input array and an output array.",
                    method.t_def.name.name,
                    method.t_def.kind
                );
                method.method_expander_kind = Some(ToExpandedFormatMethodKind::InputVal);
            }
            (WgslHelperCategory::Output, WgslHelperMethodName::Push) => {
                assert!(
                    method.arg1.is_some(),
                    "Expected an argument for output push"
                );
                assert!(
                    method.t_def.kind == CustomTypeKind::OutputVec,
                    "Expected {} to be an output vec type, since WgslOutput::push is called, instead found it was of type {:?}. Put #[wgsl_output_vec] above your type declaration to fix this. A given type cannot be used for multiple purposes, for example a type T cannot be both an input array and an output array, an output cannot be both a vec and an array.",
                    method.t_def.name.name,
                    method.t_def.kind
                );
                method.method_expander_kind = Some(ToExpandedFormatMethodKind::OutputPush);
            }
            (WgslHelperCategory::Output, WgslHelperMethodName::Len) => {
                assert!(
                    method.t_def.kind == CustomTypeKind::OutputArray
                        || method.t_def.kind == CustomTypeKind::OutputVec,
                    "Expected {} to be an output array or vec type, since WgslOutput::len is called, instead found it was of type {:?}. Put #[wgsl_output_array] or #[wgsl_output_vec] above your type declaration to fix this. A given type cannot be used for multiple purposes, for example a type T cannot be both an input array and an output array.",
                    method.t_def.name.name,
                    method.t_def.kind
                );
                method.method_expander_kind = Some(ToExpandedFormatMethodKind::OutputLen);
            }
            (WgslHelperCategory::Output, WgslHelperMethodName::Set) => {
                assert!(
                    method.t_def.kind == CustomTypeKind::OutputArray,
                    "Expected {} to be an output array type, since WgslOutput::set is called, instead found it was of type {:?}. Put #[wgsl_output_array] above your type declaration to fix this. A given type cannot be used for multiple purposes, for example a type T cannot be both an input array and an output array.",
                    method.t_def.name.name,
                    method.t_def.kind
                );
                assert!(
                    method.arg1.is_some() && method.arg2.is_some(),
                    "Expected two arguments for output set"
                );
                method.method_expander_kind = Some(ToExpandedFormatMethodKind::OutputSet);
            }
            _ => {
                method.method_expander_kind = None;
            }
        }
    }
}

```

# src\transformer\transform_wgsl_helper_methods\method_name.rs

```rs
use syn::Ident;

pub enum WgslHelperMethodName {
    VecLen,
    VecVal,
    Push,
    Len,
    Set,
    Get,
    _Invalid,
}
impl WgslHelperMethodName {
    pub fn from_ident(ident: Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "vec_len" => Some(WgslHelperMethodName::VecLen),
            "vec_val" => Some(WgslHelperMethodName::VecVal),
            "push" => Some(WgslHelperMethodName::Push),
            "len" => Some(WgslHelperMethodName::Len),
            "set" => Some(WgslHelperMethodName::Set),
            "get" => Some(WgslHelperMethodName::Get),
            _ => None,
        }
    }
}

```

# src\transformer\transform_wgsl_helper_methods\mod.rs

```rs
pub mod category;
pub mod helper_method;
pub mod matcher;
pub mod method_name;
pub mod run;
pub mod test;
pub mod to_expanded_format;

```

# src\transformer\transform_wgsl_helper_methods\run.rs

```rs
use proc_macro_error::abort;
use proc_macro2::TokenStream;

use syn::{
    Expr, ExprCall, GenericArgument, PathArguments, Type, parse_quote,
    visit_mut::{self, VisitMut},
};

use crate::{
    state::ModuleTransformState,
    transformer::{
        custom_types::custom_type::CustomType,
        transform_wgsl_helper_methods::{
            helper_method::WgslHelperMethod, to_expanded_format::ToExpandedFormat,
        },
    },
};

use super::{
    category::WgslHelperCategory, matcher::WgslHelperMethodMatcher,
    method_name::WgslHelperMethodName,
};

fn get_special_function_category(call: &ExprCall) -> Option<WgslHelperCategory> {
    if let Expr::Path(path) = &*call.func {
        if let Some(first_seg) = path.path.segments.first() {
            return WgslHelperCategory::from_ident(first_seg.ident.clone());
        }
    }
    None
}
fn get_special_function_method(call: &ExprCall) -> Option<WgslHelperMethodName> {
    if let Expr::Path(path) = &*call.func {
        if let Some(last_seg) = path.path.segments.last() {
            return WgslHelperMethodName::from_ident(last_seg.ident.clone());
        }
    }
    None
}
fn get_special_function_generic_type<'a>(
    call: &'a ExprCall,
    custom_types: &'a Vec<CustomType>,
) -> Option<&'a CustomType> {
    if let Expr::Path(path) = &*call.func {
        if let Some(last_seg) = path.path.segments.last() {
            if let PathArguments::AngleBracketed(args) = &last_seg.arguments {
                if let Some(GenericArgument::Type(Type::Path(type_path))) = args.args.first() {
                    if let Some(last_seg) = type_path.path.segments.last() {
                        return custom_types.iter().find(|t| t.name.eq(&last_seg.ident));
                    }
                }
            }
        }
    }
    None
}

fn replace(call: ExprCall, custom_types: &Vec<CustomType>) -> Option<TokenStream> {
    let category = get_special_function_category(&call);
    let method = get_special_function_method(&call);
    let type_name = get_special_function_generic_type(&call, custom_types);
    if let Some(cat) = category {
        if let Some(met) = method {
            if let Some(ty) = type_name {
                let mut method = WgslHelperMethod {
                    category: cat,
                    method: met,
                    t_def: ty,
                    arg1: call.args.first(),
                    arg2: call.args.get(1),
                    method_expander_kind: None,
                };
                WgslHelperMethodMatcher::choose_expand_format(&mut method);
                if let Some(_) = &method.method_expander_kind {
                    let t = ToExpandedFormat::run(&method);
                    return Some(t);
                }
            }
        }
    }
    None
}

struct HelperFunctionConverter {
    custom_types: Vec<CustomType>,
}

impl VisitMut for HelperFunctionConverter {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        visit_mut::visit_expr_mut(self, expr);
        if let Expr::Call(call) = expr {
            let replacement = replace(call.clone(), &self.custom_types);
            if let Some(r) = replacement {
                *expr = parse_quote!(#r);
            }
        }
    }
}
impl HelperFunctionConverter {
    pub fn new(custom_types: &Vec<CustomType>) -> Self {
        Self {
            custom_types: custom_types.clone(),
        }
    }
}

/// Rust's normal type checking will ensure that these helper functions are using correctly defined types
pub fn transform_wgsl_helper_methods(state: &mut ModuleTransformState) {
    assert!(
        state.custom_types.is_some(),
        "Allowed types must be defined"
    );
    let custom_types = if let Some(ct) = &state.custom_types {
        ct
    } else {
        abort!(
            state.rust_module.ident.span(),
            "Allowed types must be set before transforming helper functions"
        );
    };
    let mut converter = HelperFunctionConverter::new(&custom_types);
    converter.visit_item_mod_mut(&mut state.rust_module);
}

```

# src\transformer\transform_wgsl_helper_methods\test.rs

```rs
#[cfg(test)]
mod tests {
    use crate::{
        state::ModuleTransformState,
        transformer::{
            custom_types::custom_type::{CustomType, CustomTypeKind},
            transform_wgsl_helper_methods::run::transform_wgsl_helper_methods,
        },
    };

    use proc_macro2::TokenStream;
    use quote::{ToTokens, format_ident};
    use syn::{ItemMod, parse_quote};

    #[test]
    fn test_vec_len() {
        let input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    let x = WgslVecInput::vec_len::<Position>();
                }
            }
        };
        let expected_output =
            "mod test { fn example () { let x = POSITION_INPUT_ARRAY_LENGTH ; } }";
        let mut state = ModuleTransformState::empty(input, "".to_string());
        let custom_types = vec![CustomType::new(
            &format_ident!("Position"),
            CustomTypeKind::InputArray,
            TokenStream::new(),
        )];
        state.custom_types = Some(custom_types);
        transform_wgsl_helper_methods(&mut state);
        let result = state.rust_module.to_token_stream().to_string();
        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_vec_val() {
        let input: ItemMod = parse_quote! {
            mod test {

                fn example() {
                    let x = WgslVecInput::vec_val::<Radius>(5);
                }
            }
        };
        let expected_output = "mod test { fn example () { let x = radius_input_array [5] ; } }";

        let mut state = ModuleTransformState::empty(input, "".to_string());
        let custom_types = vec![CustomType::new(
            &format_ident!("Radius"),
            CustomTypeKind::InputArray,
            TokenStream::new(),
        )];
        state.custom_types = Some(custom_types);
        transform_wgsl_helper_methods(&mut state);
        let result = state.rust_module.to_token_stream().to_string();
        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_push() {
        let input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    WgslOutput::push::<CollisionResult>(value);
                }
            }
        };

        let expected_output = "mod test { fn example () { { let collisionresult_output_array_index = atomicAdd (& collisionresult_counter , 1u) ; if collisionresult_output_array_index < COLLISIONRESULT_OUTPUT_ARRAY_LENGTH { collisionresult_output_array [collisionresult_output_array_index] = value ; } } ; } }";
        let mut state = ModuleTransformState::empty(input, "".to_string());
        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputVec,
            TokenStream::new(),
        )];
        state.custom_types = Some(custom_types);
        transform_wgsl_helper_methods(&mut state);
        let result = state.rust_module.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_output_len() {
        let input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    let x = WgslOutput::len::<CollisionResult>();
                }
            }
        };
        let expected_output =
            "mod test { fn example () { let x = COLLISIONRESULT_OUTPUT_ARRAY_LENGTH ; } }";

        let mut state = ModuleTransformState::empty(input, "".to_string());
        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputVec,
            TokenStream::new(),
        )];
        state.custom_types = Some(custom_types);
        transform_wgsl_helper_methods(&mut state);
        let result = state.rust_module.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_output_set() {
        let input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    WgslOutput::set::<CollisionResult>(idx, val);
                }
            }
        };
        let expected_output =
            "mod test { fn example () { collisionresult_output_array [idx] = val ; } }";

        let mut state = ModuleTransformState::empty(input, "".to_string());
        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputArray,
            TokenStream::new(),
        )];
        state.custom_types = Some(custom_types);
        transform_wgsl_helper_methods(&mut state);
        let result = state.rust_module.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }
    #[test]
    fn test_config_get() {
        let input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    let t = WgslConfigInput::get::<Position>();
                }
            }
        };
        let expected_output = "mod test { fn example () { let t = position ; } }";

        let mut state = ModuleTransformState::empty(input, "".to_string());
        let custom_types = vec![CustomType::new(
            &format_ident!("Position"),
            CustomTypeKind::Uniform,
            TokenStream::new(),
        )];
        state.custom_types = Some(custom_types);
        transform_wgsl_helper_methods(&mut state);
        let result = state.rust_module.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }
}

```

# src\transformer\transform_wgsl_helper_methods\to_expanded_format.rs

```rs
use proc_macro_error::abort;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;

use super::helper_method::WgslHelperMethod;

pub enum ToExpandedFormatMethodKind {
    ConfigGet,
    InputLen,
    InputVal,
    OutputPush,
    OutputLen,
    OutputSet,
}

pub struct ToExpandedFormat {}
impl ToExpandedFormat {
    pub fn run(method: &WgslHelperMethod) -> TokenStream {
        match method.method_expander_kind {
            Some(ToExpandedFormatMethodKind::ConfigGet) => {
                let name = method.t_def.name.uniform();
                quote! {
                    #name
                }
            }
            Some(ToExpandedFormatMethodKind::InputLen) => {
                method.t_def.name.input_array_length().to_token_stream()
            }
            Some(ToExpandedFormatMethodKind::InputVal) => {
                let name = method.t_def.name.input_array();
                let index = if let Some(a1) = method.arg1 {
                    a1
                } else {
                    abort!(Span::call_site(), "arg1 is None for input value method")
                };
                quote! {
                    #name [ #index ]
                }
            }
            Some(ToExpandedFormatMethodKind::OutputPush) => {
                let t_def = method.t_def;
                let counter = t_def.name.counter();
                let arr = t_def.name.output_array();
                let len = t_def.name.output_array_length();
                let index = t_def.name.index();
                let value = if let Some(a1) = method.arg1 {
                    a1
                } else {
                    abort!(Span::call_site(), "arg1 is None for output push method")
                };
                quote! {
                    {
                    let #index = atomicAdd( & #counter, 1u);
                    if #index < #len {
                      #arr [ #index ] = #value;
                    }
                    }
                }
            }
            Some(ToExpandedFormatMethodKind::OutputLen) => {
                let len = method.t_def.name.output_array_length();
                len.to_token_stream()
            }
            Some(ToExpandedFormatMethodKind::OutputSet) => {
                let arr = method.t_def.name.output_array().to_token_stream();
                let index = if let Some(a1) = method.arg1 {
                    a1
                } else {
                    abort!(Span::call_site(), "arg1 is None for output set method")
                };
                let value = if let Some(a2) = method.arg2 {
                    a2
                } else {
                    abort!(Span::call_site(), "arg2 is None for output set method")
                };
                quote! {
                    #arr [ #index ] = #value
                }
            }
            None => panic!("method_expander_kind is None"),
        }
    }
}

```

# tests\compile_fail.rs

```rs
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}

```

# tests\components.rs

```rs
#![feature(f16)]
use pretty_assertions::assert_eq;
use rust_to_wgsl::wgsl_shader_module;
use shared::{
    custom_type_name::CustomTypeName,
    misc_types::TypesSpec,
    wgsl_components::{
        WgslConstAssignment, WgslFunction, WgslInputArray, WgslOutputArray,
        WgslShaderModuleSectionCode, WgslShaderModuleUserPortion, WgslType,
    },
};

#[test]
fn test_simple_struct() {
    #[wgsl_shader_module]
    pub mod test_module {
        use shared::wgsl_in_rust_helpers::WgslIterationPosition;

        struct TStruct {
            x: f32,
            y: f32,
        }
        fn main(iter_pos: WgslIterationPosition) {
            return;
        }
    }

    let t2 = test_module::parsed();
    assert!(t2.output_arrays.len() == 0);
    assert!(t2.input_arrays.len() == 0);
    assert!(t2.uniforms.len() == 0);
    assert!(t2.helper_functions.len() == 0);
    assert!(t2.main_function.is_some());
    assert!(t2.helper_types.len() == 1);
    assert_eq!(
        t2.main_function.unwrap().code.wgsl_code,
        "fn main(@builtin(global_invocation_id) iter_pos: vec3<u32>) { return; }"
    );
}

#[test]
fn test_struct_creation() {
    #[wgsl_shader_module]
    pub mod test_module {
        use shared::wgsl_in_rust_helpers::WgslIterationPosition;

        struct TStruct {
            x: f32,
            y: f32,
        }
        fn main(iter_pos: WgslIterationPosition) {
            let obj = TStruct { x: 1.0, y: 2.0 };
            return;
        }
    }

    let t2 = test_module::parsed();
    assert!(t2.output_arrays.len() == 0);
    assert!(t2.input_arrays.len() == 0);
    assert!(t2.uniforms.len() == 0);
    assert!(t2.helper_functions.len() == 0);
    assert!(t2.main_function.is_some());
    assert!(t2.helper_types.len() == 1);
    assert_eq!(
        t2.main_function.unwrap().code.wgsl_code,
        "fn main(@builtin(global_invocation_id) iter_pos: vec3<u32>)\n{ let obj = TStruct(1.0, 2.0); return; }"
    );
}

#[test]
fn test_struct_creation_with_nested_transforms() {
    #[wgsl_shader_module]
    pub mod test_module {
        use shared::wgsl_in_rust_helpers::*;

        struct TStruct {
            x: f32,
            y: Vec3F32,
        }
        fn main(iter_pos: WgslIterationPosition) {
            let obj = TStruct {
                x: 1.0,
                y: Vec3F32::new(2.0, 3.0, 4.0),
            };
            return;
        }
    }

    let t2 = test_module::parsed();
    assert!(t2.output_arrays.len() == 0);
    assert!(t2.input_arrays.len() == 0);
    assert!(t2.uniforms.len() == 0);
    assert!(t2.helper_functions.len() == 0);
    assert!(t2.main_function.is_some());
    assert!(t2.static_consts.len() == 0);

    assert!(t2.helper_types.len() == 1);
    assert_eq!(
        t2.main_function.unwrap().code.wgsl_code,
        "fn main(@builtin(global_invocation_id) iter_pos: vec3<u32>)\n{ let obj = TStruct(1.0,vec3<f32>(2.0, 3.0, 4.0)); return; }"
    );
}
#[test]
fn test_type_alias() {
    #[wgsl_shader_module]
    pub mod test_module {
        use shared::wgsl_in_rust_helpers::*;
        type MyType = i32;
        fn main(iter_pos: WgslIterationPosition) {}
    }

    let t2 = test_module::parsed();
    assert!(t2.output_arrays.len() == 0);
    assert!(t2.input_arrays.len() == 0);
    assert!(t2.uniforms.len() == 0);
    assert!(t2.helper_functions.len() == 0);
    assert!(t2.main_function.is_some());
    assert!(t2.helper_types.len() == 1);
    assert_eq!(
        t2.helper_types.first().unwrap().code.wgsl_code,
        "alias MyType  = i32;"
    );
}
#[test]
fn test_consts() {
    #[wgsl_shader_module]
    pub mod test_module {
        use shared::wgsl_in_rust_helpers::{WgslIterationPosition, *};
        const MY_CONST: i32 = 3;
        fn main(iter_pos: WgslIterationPosition) {}
    }

    let t2 = test_module::parsed();
    assert!(t2.output_arrays.len() == 0);
    assert!(t2.input_arrays.len() == 0);
    assert!(t2.uniforms.len() == 0);
    assert!(t2.helper_functions.len() == 0);
    assert!(t2.main_function.is_some());
    assert!(t2.static_consts.len() == 1);
    assert!(t2.helper_types.len() == 0);
    assert_eq!(
        t2.static_consts.first().unwrap().code.wgsl_code,
        "const MY_CONST : i32 = 3;"
    );
}
#[test]
fn test_uniforms() {
    #[wgsl_shader_module]
    pub mod test_module {
        use rust_to_wgsl::wgsl_config;
        use shared::wgsl_in_rust_helpers::*;
        #[wgsl_config]
        struct Uniforms {
            time: f32,
            resolution: Vec2F32,
        }
        fn main(iter_pos: WgslIterationPosition) {
            let time = WgslConfigInput::get::<Uniforms>().time;
        }
    }
    let t2 = test_module::parsed();
    assert!(t2.output_arrays.len() == 0);
    assert!(t2.input_arrays.len() == 0);
    assert!(t2.uniforms.len() == 1);
    assert!(t2.helper_functions.len() == 0);
    assert!(t2.main_function.is_some());
    assert!(t2.static_consts.len() == 0);
    assert!(t2.helper_types.len() == 0);
    assert_eq!(
        t2.uniforms.first().unwrap().code.wgsl_code,
        "struct Uniforms { time : f32, resolution : vec2 < f32 > , }"
    );
}

#[test]
fn test_output_arrays() {
    #[wgsl_shader_module]
    pub mod test_module {
        use rust_to_wgsl::wgsl_output_array;
        use shared::wgsl_in_rust_helpers::*;
        #[wgsl_output_array]
        struct CollisionResult {
            entity1: u32,
            entity2: u32,
        }
        fn main(iter_pos: WgslIterationPosition) {}
    }
    let t2 = test_module::parsed();
    assert!(t2.output_arrays.len() == 1);
    assert!(t2.input_arrays.len() == 0);
    assert!(t2.uniforms.len() == 0);
    assert!(t2.helper_functions.len() == 0);
    assert!(t2.main_function.is_some());
    assert!(t2.static_consts.len() == 0);
    assert!(t2.helper_types.len() == 0);
    assert_eq!(
        t2.output_arrays.first().unwrap().item_type.code.wgsl_code,
        "struct CollisionResult { entity1 : u32, entity2 : u32, }"
    );

    assert!(
        t2.output_arrays
            .first()
            .unwrap()
            .atomic_counter_name
            .is_none()
    );
}

#[test]
fn test_helper_functions() {
    #[wgsl_shader_module]
    pub mod test_module {
        use shared::wgsl_in_rust_helpers::*;
        fn calculate_distance_squared(p1: [f32; 2], p2: [f32; 2]) -> f32 {
            let dx = p1[0] - p2[0];
            let dy = p1[1] - p2[1];
            return dx * dx + dy * dy;
        }
        fn main(iter_pos: WgslIterationPosition) {}
    }
    let t2 = test_module::parsed();
    assert!(t2.output_arrays.len() == 0);
    assert!(t2.input_arrays.len() == 0);
    assert!(t2.uniforms.len() == 0);
    assert!(t2.helper_functions.len() == 1);
    assert!(t2.main_function.is_some());
    assert!(t2.static_consts.len() == 0);
    assert!(t2.helper_types.len() == 0);
    assert_eq!(
        t2.helper_functions.first().unwrap().code.wgsl_code,
        "fn calculate_distance_squared(p1 : array < f32, 2 > , p2 : array < f32, 2 >)\n-> f32\n{\n    let dx = p1 [0] - p2 [0]; let dy = p1 [1] - p2 [1]; return dx * dx + dy *\n    dy;\n}"
    );
}

#[test]

fn t() {}

#[test]
// expect a panic
#[should_panic(expected = "not implemented")]
fn can_extract_types() {
    #[wgsl_shader_module]
    pub mod test_module {
        use rust_to_wgsl::wgsl_config;
        use shared::wgsl_in_rust_helpers::*;
        /// some doc comment, should be removed
        #[wgsl_config]
        struct MyConfig {
            value: PodF16,
        }
        fn main(iter_pos: WgslIterationPosition) {}
    }
    fn fun<T: TypesSpec>() -> T::InputConfigTypes {
        unimplemented!();
    }
    let _t = fun::<test_module::Types>();
}

#[test]
fn test_simple_type_transforms() {
    #[wgsl_shader_module]
    pub mod test_module {
        use shared::wgsl_in_rust_helpers::{WgslIterationPosition, *};
        struct TStruct {
            x: f32,
            y: Vec3F32,
        }
        fn main(iter_pos: WgslIterationPosition) {}
    }

    let t2 = test_module::parsed();
    assert!(t2.output_arrays.len() == 0);
    assert!(t2.input_arrays.len() == 0);
    assert!(t2.uniforms.len() == 0);
    assert!(t2.helper_functions.len() == 0);
    assert!(t2.main_function.is_some());
    assert!(t2.helper_types.len() == 1);
    assert_eq!(
        t2.helper_types.first().unwrap().code.wgsl_code,
        "struct TStruct { x : f32, y : vec3 < f32 > , }"
    );
}

#[test]
fn test_doc_comments() {
    #[wgsl_shader_module]
    pub mod test_module {
        use rust_to_wgsl::wgsl_config;
        use shared::wgsl_in_rust_helpers::*;
        /// some doc comment, should be removed
        #[wgsl_config]
        struct MyConfig {
            f16_val: PodF16,
        }
        fn main(iter_pos: WgslIterationPosition) {}
    }
    let t2 = test_module::parsed();
    assert!(t2.output_arrays.len() == 0);
    assert!(t2.input_arrays.len() == 0);
    assert!(t2.uniforms.len() == 1);
    assert!(t2.helper_functions.len() == 0);
    assert!(t2.main_function.is_some());
    assert!(t2.static_consts.len() == 0);
    assert!(t2.helper_types.len() == 0);
    let _t = WgslShaderModuleSectionCode {
        rust_code: ("#[wgsl_config] struct MyConfig { value : PodBool, }").to_string(),
        wgsl_code: ("struct MyConfig { value : bool, }").to_string(),
    };
}
#[test]
fn test_input_arrays() {
    #[wgsl_shader_module]
    pub mod test_module {
        use rust_to_wgsl::wgsl_input_array;
        use shared::wgsl_in_rust_helpers::*;
        #[wgsl_input_array]
        type Position = [f32; 2];
        fn main(iter_pos: WgslIterationPosition) {}
    }

    let t2 = test_module::parsed();
    assert!(t2.output_arrays.len() == 0);
    assert!(t2.input_arrays.len() == 1);
    assert!(t2.uniforms.len() == 0);
    // type Position = array<f32, 2>;
    assert!(t2.helper_functions.len() == 0);
    assert!(t2.main_function.is_some());
    assert!(t2.static_consts.len() == 0);
    assert!(t2.helper_types.len() == 0);

    assert_eq!(
        t2.input_arrays.first().unwrap().item_type.code.wgsl_code,
        "alias Position  = array < f32, 2 > ;"
    )
}

#[test]
fn test_output_vec() {
    #[wgsl_shader_module]
    pub mod test_module {
        use rust_to_wgsl::wgsl_output_vec;
        use shared::wgsl_in_rust_helpers::*;
        #[wgsl_output_vec]
        struct CollisionResult {
            entity1: u32,
            entity2: u32,
        }
        fn main(iter_pos: WgslIterationPosition) {}
    }
    let t2 = test_module::parsed();
    assert!(t2.output_arrays.len() == 1);
    assert!(t2.input_arrays.len() == 0);
    assert!(t2.uniforms.len() == 0);
    assert!(t2.helper_functions.len() == 0);
    assert!(t2.main_function.is_some());
    assert!(t2.static_consts.len() == 0);
    assert!(t2.helper_types.len() == 0);
    assert_eq!(
        t2.output_arrays.first().unwrap().item_type.code.wgsl_code,
        "struct CollisionResult { entity1 : u32, entity2 : u32, }"
    );

    assert!(
        t2.output_arrays
            .first()
            .unwrap()
            .atomic_counter_name
            .is_some()
    );
    assert_eq!(
        t2.output_arrays
            .first()
            .unwrap()
            .atomic_counter_name
            .as_ref()
            .unwrap(),
        &"collisionresult_counter".to_string()
    )
}

#[test]
fn test_entire_collision_shader() {
    #[wgsl_shader_module]
    pub mod collision_shader {

        use rust_to_wgsl::*;
        use shared::wgsl_in_rust_helpers::*;
        const example_module_const: u32 = 42;
        #[wgsl_config]
        struct Uniforms {
            time: f32,
            resolution: Vec2F32,
        }
        #[wgsl_input_array]
        type Position = [f32; 2];
        #[wgsl_input_array]
        type Radius = f32;
        //* user output vectors
        #[wgsl_output_vec]
        struct CollisionResult {
            entity1: u32,
            entity2: u32,
        }
        fn calculate_distance_squared(p1: [f32; 2], p2: [f32; 2]) -> f32 {
            let dx = p1[0] - p2[0];
            let dy = p1[1] - p2[1];
            return dx * dx + dy * dy;
        }
        fn main(iter_pos: WgslIterationPosition) {
            //* USER GENERATED LOGIC
            let current_entity = iter_pos.x;
            let other_entity = iter_pos.y;
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
    let t2 = collision_shader::parsed();

    let user_portion = WgslShaderModuleUserPortion { static_consts: vec![WgslConstAssignment { code: WgslShaderModuleSectionCode { rust_code: "const example_module_const : u32 = 42;".to_string(), wgsl_code: "const example_module_const : u32 = 42;".to_string() } }], helper_types: vec![], uniforms: vec![WgslType { name: CustomTypeName::new("Uniforms"), code: WgslShaderModuleSectionCode { rust_code: "#[wgsl_config] struct Uniforms { time : f32, resolution : Vec2F32, }".to_string(), wgsl_code: "struct Uniforms { time : f32, resolution : vec2 < f32 > , }".to_string() } }], input_arrays: vec![WgslInputArray { item_type: WgslType { name: CustomTypeName::new("Position"), code: WgslShaderModuleSectionCode { rust_code: "#[wgsl_input_array] type Position = [f32; 2];".to_string(), wgsl_code: "alias Position  = array < f32, 2 > ;".to_string() } } }, WgslInputArray { item_type: WgslType { name: CustomTypeName::new("Radius") , code: WgslShaderModuleSectionCode { rust_code: "#[wgsl_input_array] type Radius = f32;".to_string(), wgsl_code: "alias Radius  = f32;".to_string() } } }], output_arrays: vec![WgslOutputArray { item_type: WgslType { name: CustomTypeName::new("CollisionResult"), code: WgslShaderModuleSectionCode { rust_code: "#[wgsl_output_vec] struct CollisionResult { entity1 : u32, entity2 : u32, }".to_string(), wgsl_code: "struct CollisionResult { entity1 : u32, entity2 : u32, }".to_string() } }, atomic_counter_name: Some("collisionresult_counter".to_string()) }], helper_functions: vec![WgslFunction { name: "calculate_distance_squared".to_string(), code: WgslShaderModuleSectionCode { rust_code: "fn calculate_distance_squared(p1 : [f32; 2], p2 : [f32; 2]) -> f32\n{\n    let dx = p1 [0] - p2 [0]; let dy = p1 [1] - p2 [1]; return dx * dx + dy *\n    dy;\n}".to_string(), wgsl_code: "fn calculate_distance_squared(p1 : array < f32, 2 > , p2 : array < f32, 2 >)\n-> f32\n{\n    let dx = p1 [0] - p2 [0]; let dy = p1 [1] - p2 [1]; return dx * dx + dy *\n    dy;\n}".to_string() } }], main_function: Some(WgslFunction { name: "main".to_owned(), code: WgslShaderModuleSectionCode { rust_code: "fn main(iter_pos : WgslIterationPosition)\n{\n    let current_entity = iter_pos.x; let other_entity = iter_pos.y; if\n    current_entity >= POSITION_INPUT_ARRAY_LENGTH || other_entity >=\n    POSITION_INPUT_ARRAY_LENGTH || current_entity == other_entity ||\n    current_entity >= other_entity { return; } let current_radius =\n    radius_input_array [current_entity]; let other_radius = radius_input_array\n    [other_entity]; if current_radius <= 0.0 || other_radius <= 0.0\n    { return; } let current_pos = position_input_array [current_entity]; let\n    other_pos = position_input_array [other_entity]; let dist_squared =\n    calculate_distance_squared(current_pos, other_pos); let radius_sum =\n    current_radius + other_radius; if dist_squared < radius_sum * radius_sum\n    {\n        {\n            let collisionresult_output_array_index =\n            atomicAdd(& collisionresult_counter, 1u); if\n            collisionresult_output_array_index <\n            COLLISIONRESULT_OUTPUT_ARRAY_LENGTH\n            {\n                collisionresult_output_array\n                [collisionresult_output_array_index] = CollisionResult\n                { entity1 : current_entity, entity2 : other_entity, };\n            }\n        };\n    }\n}".to_owned(), wgsl_code: "fn main(@builtin(global_invocation_id) iter_pos: vec3<u32>)\n{\n    let current_entity = iter_pos.x; let other_entity = iter_pos.y; if\n    current_entity >= POSITION_INPUT_ARRAY_LENGTH || other_entity >=\n    POSITION_INPUT_ARRAY_LENGTH || current_entity == other_entity ||\n    current_entity >= other_entity { return; } let current_radius =\n    radius_input_array [current_entity]; let other_radius = radius_input_array\n    [other_entity]; if current_radius <= 0.0 || other_radius <= 0.0\n    { return; } let current_pos = position_input_array [current_entity]; let\n    other_pos = position_input_array [other_entity]; let dist_squared =\n    calculate_distance_squared(current_pos, other_pos); let radius_sum =\n    current_radius + other_radius; if dist_squared < radius_sum * radius_sum\n    {\n        {\n            let collisionresult_output_array_index =\n            atomicAdd(& collisionresult_counter, 1u); if\n            collisionresult_output_array_index <\n            COLLISIONRESULT_OUTPUT_ARRAY_LENGTH\n            {\n                collisionresult_output_array\n                [collisionresult_output_array_index] =
                CollisionResult(current_entity, other_entity);\n            }\n        };\n    }\n}".to_owned() } }) };
    assert_eq!(t2, user_portion);
}

```

# tests\ui\alias_to_helper_type.rs

```rs
use rust_to_wgsl::shader_module;

#[shader_module]
mod my_mod {
    use shared::wgsl_in_rust_helpers::*;
    type MyAlias = Vec3F32;
    fn main(global_id: WgslGlobalId) {}
}

fn main() {}

```

# tests\ui\alias_to_helper_type.stderr

```stderr
error: Renaming helper types like Vec3F32, Mat2x2Bool, etc. is not supported
 --> tests/ui/alias_to_helper_type.rs:3:1
  |
3 | #[shader_module]
  | ^^^^^^^^^^^^^^^^
  |
  = note: this error originates in the attribute macro `shader_module` (in Nightly builds, run with -Z macro-backtrace for more info)

warning: unused variable: `global_id`
 --> tests/ui/alias_to_helper_type.rs:7:13
  |
7 |     fn main(global_id: WgslGlobalId) {}
  |             ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_global_id`
  |
  = note: `#[warn(unused_variables)]` on by default

```

# tests\ui\incorrect_vec_construction.rs

```rs
use shared::wgsl_in_rust_helpers::*;
const MY_CONST: Vec3Bool = Vec3Bool {
    x: true,
    y: false,
    z: true,
};

```

# tests\ui\incorrect_vec_construction.stderr

```stderr
error[E0601]: `main` function not found in crate `$CRATE`
 --> tests/ui/incorrect_vec_construction.rs:6:3
  |
6 | };
  |   ^ consider adding a `main` function to `$DIR/tests/ui/incorrect_vec_construction.rs`

error: cannot construct `shared::wgsl_in_rust_helpers::Vec3Bool` with struct literal syntax due to private fields
 --> tests/ui/incorrect_vec_construction.rs:2:28
  |
2 | const MY_CONST: Vec3Bool = Vec3Bool {
  |                            ^^^^^^^^
  |
  = note: ...and other private field `_force_constructor` that was not provided
help: you might have meant to use the `new` associated function
  |
2 | const MY_CONST: Vec3Bool = Vec3Bool::new(_, _, _);
  |                                    ~~~~~~~~~~~~~~

```

# tests\ui\incorrect_vec_constructor_in_macro.rs

```rs
use rust_to_wgsl::shader_module;

#[shader_module]
mod my_mod {
    use shared::wgsl_in_rust_helpers::*;
    const MY_CONST: Vec3Bool = Vec3Bool {
        x: true,
        y: false,
        z: true,
    };
    fn main(global_id: WgslGlobalId) {}
}

fn main() {}

```

# tests\ui\incorrect_vec_constructor_in_macro.stderr

```stderr
error: cannot construct `shared::wgsl_in_rust_helpers::Vec3Bool` with struct literal syntax due to private fields
 --> tests/ui/incorrect_vec_constructor_in_macro.rs:3:1
  |
3 | #[shader_module]
  | ^^^^^^^^^^^^^^^^
  |
  = note: ...and other private field `_force_constructor` that was not provided
  = note: this error originates in the attribute macro `shader_module` (in Nightly builds, run with -Z macro-backtrace for more info)
help: you might have meant to use the `new` associated function
  |
3 | #[shader_module]::new(_, _, _)
  |                 ++++++++++++++

```

# wip\.gitignore

```
*

```
