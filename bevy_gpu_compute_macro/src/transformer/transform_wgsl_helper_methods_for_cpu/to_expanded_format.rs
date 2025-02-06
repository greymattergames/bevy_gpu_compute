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

    OutputMaxLen,
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
                let var_name = method.t_def.name.input_array();
                quote! {
                    #var_name .len() as u32
                }
            }
            Some(ToExpandedFormatMethodKind::InputVal) => {
                let name = method.t_def.name.input_array();
                let index = if let Some(a1) = method.arg1 {
                    a1
                } else {
                    abort!(Span::call_site(), "arg1 is None for input value method")
                };
                quote! {
                    #name [ #index as usize ]
                }
            }
            Some(ToExpandedFormatMethodKind::OutputPush) => {
                let t_def = method.t_def;
                let arr = t_def.name.output_array();
                let value = if let Some(a1) = method.arg1 {
                    a1
                } else {
                    abort!(Span::call_site(), "arg1 is None for output push method")
                };
                quote! {#arr .push( #value )}
            }
            Some(ToExpandedFormatMethodKind::OutputMaxLen) => {
                // this may be a bit confusing, because the MaxLengths have no effect on the CPU side, but for now this I believe is the simplest way to handle things
                let var_name = method.t_def.name.output_array();
                quote! {
                    #var_name .len() as u32
                }
            }
            Some(ToExpandedFormatMethodKind::OutputLen) => {
                let var_name = method.t_def.name.output_array();
                quote! {
                    #var_name .len() as u32
                }
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
                    #arr [ #index as usize ] = #value
                }
            }
            None => panic!("method_expander_kind is None"),
        }
    }
}
