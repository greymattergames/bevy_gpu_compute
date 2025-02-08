use syn::{ItemStruct, parse_quote, visit_mut::VisitMut};

pub fn make_types_pod(input: &mut syn::ItemMod) {
    let mut transformer = MakeTypesPodTransformer;
    transformer.visit_item_mod_mut(input);
}

/**
Add the following as attributes:
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
*/
struct MakeTypesPodTransformer;

impl VisitMut for MakeTypesPodTransformer {
    fn visit_item_struct_mut(&mut self, i: &mut ItemStruct) {
        syn::visit_mut::visit_item_struct_mut(self, i);
        i.attrs.push(parse_quote! {
            #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
        });
        i.attrs.push(parse_quote! {
        #[repr(C)]});
    }
}
