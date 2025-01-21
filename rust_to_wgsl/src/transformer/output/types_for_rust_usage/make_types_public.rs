use syn::{ItemStruct, ItemType, Visibility, spanned::Spanned, token::Pub, visit_mut::VisitMut};

pub struct MakeTypesPublicTransformer;

impl VisitMut for MakeTypesPublicTransformer {
    fn visit_item_type_mut(&mut self, i: &mut ItemType) {
        syn::visit_mut::visit_item_type_mut(self, i);
        i.vis = Visibility::Public(Pub { span: i.span() });
    }
    fn visit_item_struct_mut(&mut self, i: &mut ItemStruct) {
        syn::visit_mut::visit_item_struct_mut(self, i);
        i.vis = Visibility::Public(Pub { span: i.span() });
    }
}
