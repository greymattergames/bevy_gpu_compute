use bevy::prelude::{Commands, Component, Entity};
use futures::never::Never;
use shared::{compose_wgsl_module::{compose_wgsl, WgslShaderModule}, misc_types::TypesSpec, wgsl_components::WgslShaderModuleUserPortion};

use crate::task::{
    events::{
        GpuComputeTaskChangeEvent, IterationSpaceOrMaxOutVecLengthChangedEvent,
        WgslCodeChangedEvent,
    },
    inputs::input_vector_metadata_spec::{
        self, InputVectorMetadataDefinition, InputVectorsMetadataSpec,
    },
    outputs::definitions::output_vector_metadata_spec::{OutputVectorMetadataDefinition, OutputVectorsMetadataSpec},
    task_components::task_max_output_bytes::TaskMaxOutputBytes,
    task_specification::{
        gpu_workgroup_sizes::GpuWorkgroupSizes, gpu_workgroup_space::GpuWorkgroupSpace,
        iteration_space::IterationSpace,
    },
    wgsl_code::WgslCode,
};

use super::max_output_vector_lengths::MaxOutputVectorLengths;

/**
These all used to be separate components, but this limited the user api, for example the user could not update the iteration space and then retrieve the resulting correct GpuWorkgroupSpace/Sizes in the same frame, since these updates were handled in separate systems.
The size of this component should still be quite small, so the tradeoff of having a larger component for a better user api is worth it.
*/
#[derive(Component)]
pub struct TaskUserSpecification {
    iteration_space: IterationSpace,
    max_output_vector_lengths: MaxOutputVectorLengths,
    wgsl_code: WgslCode,
    gpu_workgroup_space: GpuWorkgroupSpace,
    gpu_workgroup_sizes: GpuWorkgroupSizes,
    task_max_output_bytes: TaskMaxOutputBytes,
    output_vectors_metadata_spec: OutputVectorsMetadataSpec,
    input_vectors_metadata_spec: InputVectorsMetadataSpec,
}
impl Default for TaskUserSpecification {
    fn default() -> Self {
        TaskUserSpecification {
            iteration_space: IterationSpace::default(),
            max_output_vector_lengths: MaxOutputVectorLengths::default(),
            wgsl_code: WgslCode::default(),
            gpu_workgroup_space: GpuWorkgroupSpace::default(),
            gpu_workgroup_sizes: GpuWorkgroupSizes::default(),
            task_max_output_bytes: TaskMaxOutputBytes::default(),
            output_vectors_metadata_spec: OutputVectorsMetadataSpec::default(),
            input_vectors_metadata_spec: InputVectorsMetadataSpec::default(),
        }
    }
}

impl TaskUserSpecification {
    pub fn create_automatically<ShaderModuleTypes: TypesSpec>(
        wgsl_shader_module: WgslShaderModuleUserPortion,
        iteration_space: IterationSpace,
        max_output_vector_lengths: MaxOutputVectorLengths,
    )->Self {
        let full_module = compose_wgsl(wgsl_shader_module);
        let mut input_definitions = [None; 6];
        full_module.user_portion
        .input_arrays.iter().enumerate().for_each(|(i,a)|{
            // get correct binding
            if let Some(binding) = full_module.library_portion.bindings.iter().find(|b| b.type_name == a.item_type.name.name){
                
                if i < input_definitions.len() {
                    input_definitions[i] = Some(InputVectorMetadataDefinition { binding_number: binding.entry_num });
                    //todo support variety of binding groups
                }else {
                    panic!("Too many input arrays in wgsl_shader_module, max is {}", input_definitions.len());
                }
            }else {
                panic!("Could not find binding for input array {}, something has gone wrong with the library", a.item_type.name.name);
            }
            
        });
        
        let input_metadata = InputVectorsMetadataSpec::from_input_vector_types_spec::<ShaderModuleTypes::InputArrayTypes>( 
            input_definitions,
        );
        let mut output_definitions = [const { None }; 6];
        full_module.user_portion
        .output_arrays.iter().enumerate().for_each(|(i,a)|{
            // get correct binding
            if let Some(binding) = full_module.library_portion.bindings.iter().find(|b| b.type_name == a.item_type.name.name){
                
                if i < output_definitions.len() {
                    output_definitions[i] = Some(OutputVectorMetadataDefinition { binding_number: binding.entry_num,
                        include_count: a.atomic_counter_name.is_some(),
                        count_binding_number: if a.atomic_counter_name.is_some() {Some(binding.entry_num + 1)}else {None},
                     });
                }else {
                    panic!("Too many output arrays in wgsl_shader_module, max is {}", input_definitions.len());
                }
            }else {
                panic!("Could not find binding for output array {}, something has gone wrong with the library", a.item_type.name.name);
            }
            
        });
        let output_metadata = OutputVectorsMetadataSpec::from_output_vector_types_spec::<ShaderModuleTypes::OutputArrayTypes>(
            output_definitions,
        );
        TaskUserSpecification::create_manually(
            input_metadata,
            output_metadata,
            iteration_space,
            max_output_vector_lengths,
            WgslCode::new(full_module.wgsl_code,"main".to_string()),
        )
    }

    /// ensure that you send relevant update events after calling this function
    pub fn create_manually(
        input_vector_metadata_spec: InputVectorsMetadataSpec,
        output_vector_metadata_spec: OutputVectorsMetadataSpec,
        iteration_space: IterationSpace,
        max_output_vector_lengths: MaxOutputVectorLengths,
        wgsl_code: WgslCode,
    ) -> Self {
        let mut task = TaskUserSpecification::default();
        task.input_vectors_metadata_spec = input_vector_metadata_spec;
        task.output_vectors_metadata_spec = output_vector_metadata_spec;
        task.iteration_space = iteration_space;
        task.max_output_vector_lengths = max_output_vector_lengths;
        task.wgsl_code = wgsl_code;
        task.update_on_iter_space_or_max_output_lengths_change();

        task
    }
    // getters
    pub fn iteration_space(&self) -> &IterationSpace {
        &self.iteration_space
    }
    pub fn max_output_vector_lengths(&self) -> &MaxOutputVectorLengths {
        &self.max_output_vector_lengths
    }
    pub fn wgsl_code(&self) -> &WgslCode {
        &self.wgsl_code
    }
    pub fn gpu_workgroup_space(&self) -> &GpuWorkgroupSpace {
        &self.gpu_workgroup_space
    }
    pub fn gpu_workgroup_sizes(&self) -> &GpuWorkgroupSizes {
        &self.gpu_workgroup_sizes
    }
    pub fn task_max_output_bytes(&self) -> &TaskMaxOutputBytes {
        &self.task_max_output_bytes
    }
    pub fn output_vectors_metadata_spec(&self) -> &OutputVectorsMetadataSpec {
        &self.output_vectors_metadata_spec
    }
    pub fn input_vectors_metadata_spec(&self) -> &InputVectorsMetadataSpec {
        &self.input_vectors_metadata_spec
    }
    // setters
    pub fn set_iteration_space(
        &mut self,
        commands: &mut Commands,
        entity: Entity,
        new_iteration_space: IterationSpace,
    ) {
        self.set_iteration_space_no_event(new_iteration_space);
        commands.send_event(IterationSpaceOrMaxOutVecLengthChangedEvent::new(entity));
    }
    pub fn set_max_output_vector_lengths(
        &mut self,
        commands: &mut Commands,
        entity: Entity,
        new_max_output_vector_lengths: MaxOutputVectorLengths,
    ) {
        self.set_max_output_vector_lengths_no_event(new_max_output_vector_lengths);
        commands.send_event(IterationSpaceOrMaxOutVecLengthChangedEvent::new(entity));
    }
    pub fn set_wgsl_code(
        &mut self,
        commands: &mut Commands,
        entity: Entity,
        new_wgsl_code: WgslCode,
    ) {
        self.set_wgsl_code_no_event(new_wgsl_code);
        commands.send_event(WgslCodeChangedEvent::new(entity));
    }

    pub fn set_iteration_space_no_event(&mut self, new_iteration_space: IterationSpace) {
        self.iteration_space = new_iteration_space;
        self.update_on_iter_space_or_max_output_lengths_change();
    }
    pub fn set_max_output_vector_lengths_no_event(
        &mut self,
        new_max_output_vector_lengths: MaxOutputVectorLengths,
    ) {
        self.max_output_vector_lengths = new_max_output_vector_lengths;
        self.update_on_iter_space_or_max_output_lengths_change();
    }
    pub fn set_wgsl_code_no_event(&mut self, new_wgsl_code: WgslCode) {
        self.wgsl_code = new_wgsl_code;
    }

    fn update_on_iter_space_or_max_output_lengths_change(&mut self) {
        if self.iteration_space.x() > 1
            || self.iteration_space.y() > 1
            || self.iteration_space.z() > 1
        {
            // update task max output bytes
            self.task_max_output_bytes = TaskMaxOutputBytes::from_max_lengths_and_spec(
                &self.max_output_vector_lengths,
                &self.output_vectors_metadata_spec,
            );
            // update workgroup sizes
            if self.iteration_space.num_dimmensions() != self.gpu_workgroup_sizes.num_dimmensions()
            {
                self.gpu_workgroup_sizes =
                    GpuWorkgroupSizes::from_iter_space(&self.iteration_space);
            }
            // update num workgroups required
            self.gpu_workgroup_space.x = (self.iteration_space.x() as f32
                / self.gpu_workgroup_sizes.x() as f32)
                .ceil() as u32;
            self.gpu_workgroup_space.y = (self.iteration_space.y() as f32
                / self.gpu_workgroup_sizes.y() as f32)
                .ceil() as u32;
            self.gpu_workgroup_space.z = (self.iteration_space.z() as f32
                / self.gpu_workgroup_sizes.z() as f32)
                .ceil() as u32;
        }
    }
}
