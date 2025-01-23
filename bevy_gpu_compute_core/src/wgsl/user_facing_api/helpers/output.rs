/**
 * All outputs are arrays/vectors.
 * No "get" type methods are implemented, sinc GPU operations are massively parallel, and you should not be READING from your outputs since you will have no way of knowing if another thread has already touched a certain output or not handled it yet.  //todo: (need to add a link to an article explaining this)
 */
pub struct WgslOutput {}
impl WgslOutput {
    /// Using "array_set" is generally more performant, use this only if you can't determine beforehand the number of outputs you will be producing.
    /// Using this will generate an atomic counter for the specific output. If your ACTUAL output length is always very close to your maximum output length for this specific output, consider using "array_set" and manually removing the trailing empty values instead.
    /// However if your ACTUAL output length could be significantly smaller than your set maximum output length, then using "vec_push" is more performant. It was created for those situations. For example: collision detection where the actual number of collisions is often far less than the theoretical maximum.
    pub fn push<T>(_val: T) {
        unimplemented!()
    }
    pub fn set<T>(_index: u32, _val: T) {
        unimplemented!()
    }
    /// returns the user-input maximum number of elements that can be stored in the output for this specific type.
    pub fn max_len<T>() -> u32 {
        unimplemented!()
    }
    /// returns the number of times this vector has been pushed to. (Or, for the power-user, the value of the atomic counter)
    pub fn len<T>() -> u32 {
        unimplemented!()
    }
}
