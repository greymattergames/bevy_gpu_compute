use crate::pipeline::{
    compilation_unit::CompilationUnit, compile_error::CompileError,
    phases::compiler_phase::CompilerPhase,
};
use quote::quote;

use super::{
    config_input_data_builder::create_config_input_data_builder,
    input_data_builder::create_input_data_builder,
    max_output_lengths_builder::create_max_output_lengths_builder,
    output_data_builder::create_output_data_builder,
};

pub struct TypesafeBufferBuildersGenerator;

impl CompilerPhase for TypesafeBufferBuildersGenerator {
    fn execute(&self, input: &mut CompilationUnit) -> Result<(), CompileError> {
        let config_input = create_config_input_data_builder(input.custom_types());
        let array_input = create_input_data_builder(input.custom_types());
        let array_output = create_output_data_builder(input.custom_types());
        let output_lengths = create_max_output_lengths_builder(input.custom_types());
        input.set_typesafe_buffer_builders(quote! {
            #config_input
            #array_input
            #array_output
            #output_lengths
        });
        Ok(())
    }
}
