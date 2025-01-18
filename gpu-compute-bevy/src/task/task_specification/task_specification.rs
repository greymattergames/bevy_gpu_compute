use std::collections::HashMap;

use bevy::{log, prelude::{Commands, Component, Entity}, render::renderer::RenderDevice, state::commands};
use futures::never::Never;
use shared::{compose_wgsl_module::{compose_wgsl, WgslShaderModule}, misc_types::TypesSpec, wgsl_components::{WgslShaderModuleUserPortion, WORKGROUP_SIZE_X_VAR_NAME, WORKGROUP_SIZE_Y_VAR_NAME, WORKGROUP_SIZE_Z_VAR_NAME}};

use crate::task::{
    events::{
        GpuComputeTaskChangeEvent,
    }, inputs::input_vector_metadata_spec::{
        self, InputVectorMetadataDefinition, InputVectorsMetadataSpec,
    }, outputs::definitions::output_vector_metadata_spec::{OutputVectorMetadataDefinition, OutputVectorsMetadataSpec}, task_components::task_max_output_bytes::TaskMaxOutputBytes, task_specification::{
        gpu_workgroup_sizes::GpuWorkgroupSizes, gpu_workgroup_space::GpuWorkgroupSpace,
        iteration_space::IterationSpace,
    }, wgsl_code::WgslCode
};

use super::{derived_spec::ComputeTaskDerivedSpec, immutable_spec::ComputeTaskImmutableSpec, input_array_lengths::ComputeTaskInputArrayLengths, max_output_vector_lengths::MaxOutputLengths, mutable_spec::ComputeTaskMutableSpec};

/**
These all used to be separate components, but this limited the user api, for example the user could not update the iteration space and then retrieve the resulting correct GpuWorkgroupSpace/Sizes in the same frame, since these updates were handled in separate systems.
The size of this component should still be quite small, so the tradeoff of having a larger component for a better user api is worth it.
*/
#[derive(Component,Default)]
pub struct ComputeTaskSpecification {
    /// things that the user sets at task creation that never change afterwords
    immutable: ComputeTaskImmutableSpec,
    /// things the user can change after task creation
    mutate: ComputeTaskMutableSpec,
    /// things that change automatically if the user changes the task after creation
    derived: ComputeTaskDerivedSpec,
}

impl ComputeTaskSpecification {
    pub fn from_shader<ShaderModuleTypes: TypesSpec>(
        name: &str,
        mut commands: &mut Commands,
        entity: Entity,
        render_device: &RenderDevice, 
        wgsl_shader_module: WgslShaderModuleUserPortion,
        iteration_space: IterationSpace,
        max_output_vector_lengths: MaxOutputLengths,
    )->Self {
        let full_module = compose_wgsl(wgsl_shader_module);
        log::info!("wgsl: {}",full_module.wgsl_code);
        let mut input_definitions = [None; 6];
        full_module.user_portion
        .input_arrays.iter().enumerate().for_each(|(i,a)|{
            // get correct binding
            if let Some(binding) = full_module.library_portion.bindings.iter().find(|b| b.name == a.item_type.name.input_array()){
                
                if i < input_definitions.len() {
                    input_definitions[i] = Some(InputVectorMetadataDefinition { binding_number: binding.entry_num, name: &a.item_type.name });
                    //todo support variety of binding groups
                }else {
                    panic!("Too many input arrays in wgsl_shader_module, max is {}", input_definitions.len());
                }
            }else {
                panic!("Could not find binding for input array {}, something has gone wrong with the library", a.item_type.name.name());
            }
            
        });
        
        let input_metadata = InputVectorsMetadataSpec::from_input_vector_types_spec::<ShaderModuleTypes::InputArrayTypes>( 
            input_definitions,
        );
        let mut output_definitions = [const { None }; 6];
        full_module.user_portion
        .output_arrays.iter().enumerate().for_each(|(i,a)|{
            // get correct binding
            if let Some(binding) = full_module.library_portion.bindings.iter().find(|b| {
                b.name == a.item_type.name.output_array()
            }){
                
                if i < output_definitions.len() { 
                    output_definitions[i] = Some(OutputVectorMetadataDefinition { binding_number: binding.entry_num,
                        include_count: a.atomic_counter_name.is_some(),
                        count_binding_number: if a.atomic_counter_name.is_some() {Some(binding.entry_num + 1)}else {None},
                        name: &a.item_type.name,
                     });
                }else {
                    panic!("Too many output arrays in wgsl_shader_module, max is {}", input_definitions.len());
                }
            }else {
                panic!("Could not find binding for output array {}, something has gone wrong with the library", a.item_type.name.name());
            }
            
        });
        let output_metadata = OutputVectorsMetadataSpec::from_output_vector_types_spec::<ShaderModuleTypes::OutputArrayTypes>(
            output_definitions,
        );
        ComputeTaskSpecification::create_manually(
            &mut commands,
            entity,
            input_metadata,
            output_metadata,
            iteration_space,
            max_output_vector_lengths,
            WgslCode::from_string(
                name,
                    render_device,
                full_module.wgsl_code,"main".to_string()),
        )
    }

    /// ensure that you send relevant update events after calling this function
    pub fn create_manually(
        mut commands: &mut Commands,
        entity: Entity,
        input_vectors_metadata_spec: InputVectorsMetadataSpec,
        output_vectors_metadata_spec: OutputVectorsMetadataSpec,
        iteration_space: IterationSpace,
        max_output_array_lengths: MaxOutputLengths,
        wgsl_code: WgslCode,
    ) -> Self {
      
        let immutable = ComputeTaskImmutableSpec::new( output_vectors_metadata_spec, input_vectors_metadata_spec, wgsl_code );
        let mut derived = ComputeTaskDerivedSpec::new(
            GpuWorkgroupSpace::default(),
            TaskMaxOutputBytes::default(),
            GpuWorkgroupSizes::default(),
        );
        let mutable= ComputeTaskMutableSpec::new(iteration_space, ComputeTaskInputArrayLengths::default(), max_output_array_lengths,&mut derived, &immutable, &mut commands, entity);
        ComputeTaskSpecification {
            immutable,
            mutate: mutable,
            derived,
        }
    }
    // getters
    pub fn iteration_space(&self) -> &IterationSpace {
        &self.mutate.iteration_space()
    }
    pub fn wgsl_code(&self) -> &WgslCode {
        &self.immutable.wgsl_code()
    }
    pub fn gpu_workgroup_space(&self) -> &GpuWorkgroupSpace {
        &self.derived.gpu_workgroup_space()
    }
    pub fn task_max_output_bytes(&self) -> &TaskMaxOutputBytes {
        &self.derived.task_max_output_bytes()
    }
    pub fn output_array_lengths(&self) -> &MaxOutputLengths {
        &self.mutate.output_array_lengths()
    }
    pub fn output_vectors_metadata_spec(&self) -> &OutputVectorsMetadataSpec {
        &self.immutable.output_vectors_metadata_spec()
    }
    pub fn input_vectors_metadata_spec(&self) -> &InputVectorsMetadataSpec {
        &self.immutable.input_vectors_metadata_spec()
    }
    pub fn iter_space_and_out_lengths_version(&self) -> u64 {
        self.mutate.iter_space_and_out_lengths_version()
    }
    // setters
     /// one of each event type maximum is sent per call, so this is more efficient than updating each field individually
    /// If a parameter is None then the existing value is retained
    pub fn mutate(
        &mut self,
       mut commands: &mut Commands,
        entity: Entity,
        new_iteration_space: Option<IterationSpace>,
        new_max_output_array_lengths: Option<MaxOutputLengths>,
        new_input_array_lengths: Option<ComputeTaskInputArrayLengths>,
    ) {
        self.mutate.multiple(new_iteration_space, new_input_array_lengths, new_max_output_array_lengths, &self.immutable, &mut self.derived, &mut commands, entity);
    }
  
    pub fn get_pipeline_consts(&self) -> HashMap<String, f64>{
            let mut n: HashMap<String, f64> = HashMap::new();
            n.insert(
                WORKGROUP_SIZE_X_VAR_NAME.to_string(),
                self.derived.workgroup_sizes().x() as f64,
            );
            n.insert(
                WORKGROUP_SIZE_Y_VAR_NAME.to_string(),
                self.derived.workgroup_sizes().y() as f64,
            );
            n.insert(
                WORKGROUP_SIZE_Z_VAR_NAME.to_string(),
                self.derived.workgroup_sizes().z() as f64,
            );
            // input and output array lengths
            for (k, v) in &self.mutate.input_array_lengths().map {
                n.insert(k.clone(), *v);
            }
            for o in self.immutable.output_vectors_metadata_spec().get_all_metadata().iter(){
                if let Some(metadata) = o{
                    n.insert(metadata.name().output_array_length(), self.mutate.output_array_lengths().get_by_name(metadata.name()) as f64);
                }
            }
            n

    }
    
}
