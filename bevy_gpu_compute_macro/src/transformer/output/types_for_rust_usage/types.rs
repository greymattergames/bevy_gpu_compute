use bevy_gpu_compute_core::wgsl::shader_module::user_defined_portion::WgslShaderModuleUserPortion;
use proc_macro_error::abort;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{Ident, parse2, visit_mut::VisitMut};

use crate::{
    state::ModuleTransformState,
    transformer::{
        custom_types::custom_type::CustomTypeKind,
        output::types_for_rust_usage::{
            make_types_public::MakeTypesPublicTransformer,
            max_output_lengths_builder::{self, create_max_output_lengths_builder},
        },
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
    let max_output_lengths_builder = create_max_output_lengths_builder(state);
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
            type ConfigInputTypes = _ConfigInputTypes;
            type InputArrayTypes = _InputArrayTypes;
            type OutputArrayTypes = _OutputArrayTypes;
        }

        #max_output_lengths_builder

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

         pub struct _ConfigInputTypes {}
    impl ConfigInputTypesSpec for _ConfigInputTypes {
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
