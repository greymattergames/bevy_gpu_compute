fn create_persistent_gpu_resources(mut commands: Commands, render_device: Res<RenderDevice>) {
    let wgsl_file = std::fs::read_to_string("src/gpu_collision_detection/collision.wgsl").unwrap();
    commands.insert_resource(WgslFile(wgsl_file));
    let counter_staging_buffer = render_device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Counter Staging Buffer"),
        size: std::mem::size_of::<WgslCounter>() as u64,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    commands.insert_resource(CounterStagingBuffer(counter_staging_buffer));
    // Create bind group layout once
    let bind_group_layouts = render_device.create_bind_group_layout(Some(task_label), &[
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]);

    let pipeline_layout = render_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(task_label),
        bind_group_layouts: &[&bind_group_layouts],
        push_constant_ranges: &[],
    });
    commands.insert_resource(PipelineLayoutResource(pipeline_layout));
    commands.insert_resource(BindGroupLayoutsResource(bind_group_layouts));
}
