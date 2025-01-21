use syn::{Field, Visibility, visit_mut::VisitMut};

pub struct PubRemover {}

impl VisitMut for PubRemover {
    fn visit_field_mut(&mut self, i: &mut Field) {
        syn::visit_mut::visit_field_mut(self, i);
        i.vis = Visibility::Inherited;
    }
}
