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
