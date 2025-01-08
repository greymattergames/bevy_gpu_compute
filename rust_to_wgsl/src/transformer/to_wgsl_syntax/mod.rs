use array::ArrayToWgslTransformer;
use expr::ExprToWgslTransformer;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Expr, File, Item, parse, parse2, visit_mut::VisitMut};
use r#type::TypeToWgslTransformer;
use type_def::TypeDefToWgslTransformer;

use crate::state::ModuleTransformState;

/**
 # Notes about conversions (all syntax not mentioned is either the same or not supported in wgsl)

- ForLoop(ExprForLoop):
  in wgsl, but with javascript style syntax: for (var i = 0; i< 10; i++){}

- Loop(ExprLoop):
  supported in wgsl, but with different syntax: `for (;;) {}`

- Reference(ExprReference):
  support pointer types, but this is something for a future version. Example of pointers in wgsl:
  ```
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
  ```

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
mod r#type;
mod type_def;

pub fn convert_to_wgsl(input: TokenStream, state: &ModuleTransformState) -> TokenStream {
    let mut file = parse::<File>(input.into()).unwrap();
    TypeToWgslTransformer {
        custom_types: &state.allowed_types.as_ref().unwrap().custom_types,
    }
    .visit_file_mut(&mut file);
    ArrayToWgslTransformer {}.visit_file_mut(&mut file);
    TypeDefToWgslTransformer {}.visit_file_mut(&mut file);
    ExprToWgslTransformer {}.visit_file_mut(&mut file);
    file.to_token_stream().into()
}
