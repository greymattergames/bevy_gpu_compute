use crate::pipeline::phases::custom_type_collector::collect::collect_custom_types;
use crate::pipeline::{
    compilation_metadata::CompilationMetadata, compilation_unit::CompilationUnit,
    compile_error::CompileError, phases::compiler_phase::CompilerPhase,
};

use super::validate_no_doc_comments::validate_no_doc_comments;
use super::validate_no_iter_pos_assignments::validate_no_iter_pos_assignments;
use super::validate_use_statements::validate_use_statements;

/// any sort of input validation that can be done on the original tree that doesn't require mutation
pub struct NonMutatingTreeValidation;

impl CompilerPhase for NonMutatingTreeValidation {
    fn execute(&self, input: &mut CompilationUnit) -> Result<(), CompileError> {
        validate_no_doc_comments(input.original_rust_module());
        validate_no_iter_pos_assignments(input.original_rust_module());
        validate_use_statements(input.original_rust_module());
        Ok(())
    }
}
