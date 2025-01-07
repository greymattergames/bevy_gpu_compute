use syn::Ident;

pub enum WgslHelperCategory {
    VecInput,
    Output,
    _Invalid,
}
// from ident
impl WgslHelperCategory {
    pub fn from_ident(ident: Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "WgslVecInput" => Some(WgslHelperCategory::VecInput),
            "WgslOutput" => Some(WgslHelperCategory::Output),
            _ => None,
        }
    }
}
