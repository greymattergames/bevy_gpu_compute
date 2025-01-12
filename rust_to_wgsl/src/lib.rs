#![feature(allocator_api)]
use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro_error::{proc_macro_error, set_dummy};
use quote::{ToTokens, quote};
use state::ModuleTransformState;
use syn::{parse, parse_macro_input, token::Semi};
use transformer::{
    custom_types::get_all_custom_types::get_custom_types,
    module_parser::module_parser::parse_shader_module,
    tokenized_initializer_for_user_portion::convert_wgsl_shader_module_user_portion_into_tokenized_initializer_code,
    transform_wgsl_helper_methods::run::transform_wgsl_helper_methods,
};
mod state;
mod transformer;
/**
## *Please read this documentation carefully, especially if you are getting errors that you don't understand!*

*...because it's currently impossible with rust proc-macros to override some of the error messages, so a lot of them don't actually indicate correctly what your issue is. So this documentation is how you are going to solve them!*

Here are some pointers:
- No let statements allowed except within functions. If you want to define a variable use "const" instead.
- Every Input/Output you want to transfer between the CPU and GPU must have its type defined within the shader module. Here's how you do that:
    - Input Vec/Array/Matrices: Define the inner-type, and put `#[vec_input]` above the type definition. Example: If you want to work with an input equivalent to `Vec<{x:f32, y:f32}>` in your module, then write
    ```ignore
    #[vec_input]
    pub struct MyStruct { x: f32, y: f32 }
    ```

    We wrap the inner type in an array for you automatically, so that you don't have to worry about data length or alignments.
    // todo


 */
#[proc_macro_attribute]
#[proc_macro_error]
pub fn shader_module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // set_dummy(quote! { mod shader_module { const t: u32 = 0;} });
    let mut module = parse_macro_input!(item as syn::ItemMod);
    let mut state = ModuleTransformState::empty(module);
    get_custom_types(&mut state);
    transform_wgsl_helper_methods(&mut state);
    parse_shader_module(&mut state);
    let initialization =
        convert_wgsl_shader_module_user_portion_into_tokenized_initializer_code(&state);
    let r: TokenStream = quote! (
    #initialization
    )
    .into();
    return r;
    // let out_s = initialization.to_string();
    // quote!(struct S {};#out_s).into()
    // item
}
