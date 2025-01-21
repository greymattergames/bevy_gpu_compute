use super::custom_type::WgslType;

#[derive(Clone, Debug, PartialEq)]

pub struct WgslOutputArray {
    pub item_type: WgslType,
    pub atomic_counter_name: Option<String>,
}
