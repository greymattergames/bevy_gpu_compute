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
