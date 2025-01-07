pub trait WgslScalar {}
impl WgslScalar for bool {}
impl WgslScalar for u32 {}
impl WgslScalar for i32 {}
impl WgslScalar for f32 {}
impl WgslScalar for f16 {}
pub type Vec2<T: WgslScalar> = [T; 2];
pub type Vec3<T: WgslScalar> = [T; 3];
pub type Vec4<T: WgslScalar> = [T; 4];
pub type Mat2x2<T: WgslScalar> = [Vec2<T>; 2];
pub type Mat3x3<T: WgslScalar> = [Vec3<T>; 3];
pub type Mat4x4<T: WgslScalar> = [Vec4<T>; 4];

pub struct WgslGlobalId {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

/// This actually refers to wgpu "Uniforms", but most people don't know what those are, so we call them "configs" instead.
pub struct WgslConfigInput {}
impl WgslConfigInput {
    pub fn get<T>() -> T {
        unimplemented!()
    }
}

/// These method are named "vec" because per this library API you input your data as variable-sized vectors. But keep in mind that on the actual GPU these are all fixed-length arrays.
pub struct WgslVecInput {}
impl WgslVecInput {
    pub fn vec_len<T>() -> u32 {
        unimplemented!()
    }
    pub fn vec_val<T>(index: u32) -> T {
        unimplemented!()
    }
}
/**
 * All outputs are arrays/vectors.
 * No "get" type methods are implemented, sinc GPU operations are massively parallel, and you should not be READING from your outputs since you will have no way of knowing if another thread has already touched a certain output or not handled it yet. //todo: (need to add a link to an article explaining this)
 */
pub struct WgslOutput {}
impl WgslOutput {
    /// Using "array_set" is generally more performant, use this only if you can't determine beforehand the number of outputs you will be producing.
    /// Using this will generate an atomic counter for the specific output. If your ACTUAL output length is always very close to your maximum output length for this specific output, consider using "array_set" and manually removing the trailing empty values instead.
    /// However if your ACTUAL output length could be significantly smaller than your set maximum output length, then using "vec_push" is more performant. It was created for those situations. For example: collision detection where the actual number of collisions is often far less than the theoretical maximum.
    pub fn push<T>(val: T) {
        unimplemented!()
    }
    pub fn set<T>(index: u32, val: T) {
        unimplemented!()
    }
    /// returns the user-input maximum number of elements that can be stored in the output for this specific type.
    pub fn len<T>() -> u32 {
        unimplemented!()
    }
}
