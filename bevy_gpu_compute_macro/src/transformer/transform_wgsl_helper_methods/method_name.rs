use syn::Ident;

pub enum WgslHelperMethodName {
    VecLen,
    VecVal,
    Push,
    Len,
    Set,
    Get,
    _Invalid,
}
impl WgslHelperMethodName {
    pub fn from_ident(ident: Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "vec_len" => Some(WgslHelperMethodName::VecLen),
            "vec_val" => Some(WgslHelperMethodName::VecVal),
            "push" => Some(WgslHelperMethodName::Push),
            "len" => Some(WgslHelperMethodName::Len),
            "set" => Some(WgslHelperMethodName::Set),
            "get" => Some(WgslHelperMethodName::Get),
            _ => None,
        }
    }
}
