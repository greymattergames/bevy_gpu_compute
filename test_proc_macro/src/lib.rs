use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::proc_macro_error;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, visit, visit_mut::VisitMut,
    AngleBracketedGenericArguments, Ident, PathArguments,
};

#[proc_macro_attribute]
#[proc_macro_error]
pub fn test2_proc_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut module = parse_macro_input!(item as syn::ItemMod);
    // transform_module(&mut module);
    assert!(false, "assert works in proc_macro");
    module.to_token_stream().into()
}

// fn transform_module(module: &mut syn::ItemMod) {
//     let mut mutator = Mutator {};
//     mutator.visit_item_mod_mut(module);
//     let mut mutator2 = Mutator2 {};
//     mutator2.visit_item_mod_mut(module);
// }

/// turn types inside Vec into non valid syntax
// struct Mutator {}
// impl VisitMut for Mutator {
//     fn visit_path_segment_mut(&mut self, seg: &mut syn::PathSegment) {
//         syn::visit_mut::visit_path_segment_mut(self, seg);
//         if seg.ident.to_string() == "i32" {
//             println!("changing i32 to i64");
//             seg.ident = parse_quote!(veveve<theuoh,theou>)
//         }
//     }
// }

/// convert Vec types to lowercase, preserving whatever is inside
// struct Mutator2 {}
// impl VisitMut for Mutator2 {}

#[cfg(test)]
mod tests {
    use syn::ItemMod;

    use super::*;

    // #[test]
    // fn visit_mut_can_insert_different_token_types_multiple_times() {
    //     // #[test2_proc_macro]
    //     let mut input: ItemMod = parse_quote! {mod test2_mod {
    //         struct TestStruct {
    //             pub x: Vec<Vec<i32>>,
    //             pub y: i32,
    //         }
    //     }};
    //     transform_module(&mut input);

    //     let result = input.to_token_stream().to_string();
    //     println!("{:#?}", result);
    //     // let result = add(2, 2);
    //     // assert_eq!(result, 4);
    // }
    #[test]
    fn assert_works_in_proc_macro() {
        #[test2_proc_macro]
        mod tt {
            fn main() {}
        }
    }
}
