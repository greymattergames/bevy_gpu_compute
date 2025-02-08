use crate::pipeline::{compilation_unit::CompilationUnit, compile_error::CompileError};

pub trait CompilerPhase {
    /// using mutation for performance reasons
    fn execute(&self, input: &mut CompilationUnit) -> Result<(), CompileError>;
}
