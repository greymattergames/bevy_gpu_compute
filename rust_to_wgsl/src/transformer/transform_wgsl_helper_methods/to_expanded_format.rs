use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;

use super::helper_method::WgslHelperMethod;

pub enum ToExpandedFormatMethodKind {
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
            Some(ToExpandedFormatMethodKind::InputLen) => {
                method.t_def.name.input_array_length().to_token_stream()
            }
            Some(ToExpandedFormatMethodKind::InputVal) => {
                let name = method.t_def.name.input_array();
                let index = method.arg1.unwrap();
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
                let value = method.arg1.unwrap();
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
                let arr = method.t_def.name.output_array();
                let index = method.arg1.unwrap();
                let value = method.arg2.unwrap();
                quote! {
                    #arr [ #index ] = #value
                }
            }
            None => panic!("method_expander_kind is None"),
        }
    }
}
