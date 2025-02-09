use crate::pipeline::compilation_unit::CompilationUnit;

pub trait CompilerPhase {
    /// using mutation for performance reasons
    /// Also not returning a result since we should try to use macro_error abort to give proper span info when possible
    fn execute(&self, input: &mut CompilationUnit);
}
