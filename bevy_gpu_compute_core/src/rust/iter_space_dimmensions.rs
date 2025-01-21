#[derive(Debug, Clone, PartialEq, Hash, Copy)]
pub enum IterSpaceDimmension {
    OneD,
    TwoD,
    ThreeD,
}
impl IterSpaceDimmension {
    pub fn to_usize(&self) -> usize {
        match self {
            IterSpaceDimmension::OneD => 1,
            IterSpaceDimmension::TwoD => 2,
            IterSpaceDimmension::ThreeD => 3,
        }
    }
}
