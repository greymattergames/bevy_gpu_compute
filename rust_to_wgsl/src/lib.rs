#![feature(allocator_api)]
use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro_error::{proc_macro_error, set_dummy};
use quote::{ToTokens, quote};
use state::ModuleTransformState;
use syn::{parse, parse_macro_input, token::Semi};
use transformer::{
    custom_types::get_all_custom_types::get_custom_types,
    module_parser::module_parser::parse_shader_module, remove_doc_comments::remove_doc_comments,
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
- When accessing special WGSL types like `Vec3`, `Mat3x4`, etc. you CANNOT use parenthesis when accessing the fields. For example:
### Valid:
```rust
# use shared::wgsl_in_rust_helpers::*;
let my_vec3 = Vec2I32::new(1,2);
// valid:
let x = my_vec3.x;
let x = my_vec3[0];
```
### Invalid:
```compile_fail
// invalid:
let x = my_vec3.x();
```
- "rgba" field access on vectors is not supported right now, use index or xyzw instead.
- Every Input/Output you want to transfer between the CPU and GPU must have its type defined within the shader module. Here's how you do that:
    - Input Vec/Array/Matrices: Define the inner-type, and put `#[vec_input]` above the type definition. Example: If you want to work with an input equivalent to `Vec<{x:f32, y:f32}>` in your module, then write
    ```ignore
    #[vec_input]
    pub struct MyStruct { x: f32, y: f32 }
    ```

    We wrap the inner type in an array for you automatically, so that you don't have to worry about data length or alignments.
    // todo
* You cannot use rust's struct-literal syntax for initializing structs from the helper types module. You must use the `new` method instead. For example:
    ## Wrong:
     ```compile_fail
         #use shared::wgsl_in_rust_helpers::*;
          const MY_CONST: Vec3Bool = Vec3Bool {
        x: true,
        y: false,
        z: true,
    };```


    ## Correct:
    ```
    #use shared::wgsl_in_rust_helpers::*;
    const MY_CONST: Vec3Bool = Vec3Bool::new(true, false, true);
    ```


 */

#[proc_macro_attribute]
#[proc_macro_error]
pub fn wgsl_shader_module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("Entered shader_module proc macro");
    set_dummy(item.clone().into());
    let content = item.to_string();
    let content_no_doc_comments: TokenStream = remove_doc_comments(&content).parse().unwrap();
    let mut module = parse_macro_input!(content_no_doc_comments as syn::ItemMod);
    let mut state = ModuleTransformState::empty(module, content);
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
