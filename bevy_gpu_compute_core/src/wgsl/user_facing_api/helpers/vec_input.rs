/// These method are named "vec" because per this library API you input your data as variable-sized vectors. But keep in mind that on the actual GPU these are all fixed-length arrays.
pub struct WgslVecInput {}
impl WgslVecInput {
    pub fn vec_len<T>() -> u32 {
        unimplemented!()
    }
    pub fn vec_val<T>(_index: u32) -> T {
        unimplemented!()
    }
}
