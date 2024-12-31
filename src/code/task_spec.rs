use bevy::prelude::Component;
use bytemuck::{Pod, Zeroable};
use paste::paste;
use wgpu::BufferView;

// Marker type for unused inputs/outputs
#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(transparent)]
pub struct Unused(u8);

// First, define a metadata struct to hold the properties you want to track
#[derive(Clone, Debug)]
pub struct TypeMetadata {
    pub name: &'static str,
    pub description: &'static str,
    pub byte_alignment: usize,
    // Add any other metadata fields you need
    pub is_optional: bool,
}
// First, let's define a trait that specifies the input and output types for a task
// Extend the InputSpec trait with metadata methods
pub trait InputSpec {
    type Input1: Pod;
    type Input2: Pod;
    type Input3: Pod;
    type Input4: Pod;
    type Input5: Pod;
    type Input6: Pod;

    // Add associated constants for metadata
    const INPUT1_METADATA: Option<TypeMetadata>;
    const INPUT2_METADATA: Option<TypeMetadata>;
    const INPUT3_METADATA: Option<TypeMetadata>;
    const INPUT4_METADATA: Option<TypeMetadata>;
    const INPUT5_METADATA: Option<TypeMetadata>;
    const INPUT6_METADATA: Option<TypeMetadata>;
}

// Similarly for OutputSpec
pub trait OutputSpec {
    type Output1: Pod;
    type Output2: Pod;
    type Output3: Pod;
    type Output4: Pod;
    type Output5: Pod;
    type Output6: Pod;

    const OUTPUT1_METADATA: Option<TypeMetadata>;
    const OUTPUT2_METADATA: Option<TypeMetadata>;
    const OUTPUT3_METADATA: Option<TypeMetadata>;
    const OUTPUT4_METADATA: Option<TypeMetadata>;
    const OUTPUT5_METADATA: Option<TypeMetadata>;
    const OUTPUT6_METADATA: Option<TypeMetadata>;
}

// Example task specification with Pod types
#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ImageParams {
    scale: f32,
    threshold: f32,
    other: f32,
}
#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct MyTuple([f32; 3]);
#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct MyBigList([f32; 100]);

// Example task specification
pub struct ImageProcessingTaskInput;

// Implement the TaskSpec trait for our example task
// Example implementation for ImageProcessingTaskInput
impl InputSpec for ImageProcessingTaskInput {
    type Input1 = MyTuple;
    type Input2 = ImageParams;
    type Input3 = MyBigList;
    type Input4 = u64;
    type Input5 = Unused;
    type Input6 = Unused;

    const INPUT1_METADATA: Option<TypeMetadata> = Some(TypeMetadata {
        name: "raw_pixels",
        description: "Raw RGB pixel data in normalized float format",
        byte_alignment: 4,
        is_optional: false,
    });

    const INPUT2_METADATA: Option<TypeMetadata> = Some(TypeMetadata {
        name: "processing_params",
        description: "Image processing parameters",
        byte_alignment: 4,
        is_optional: false,
    });

    const INPUT3_METADATA: Option<TypeMetadata> = Some(TypeMetadata {
        name: "auxiliary_data",
        description: "Additional processing parameters",
        byte_alignment: 4,
        is_optional: true,
    });

    const INPUT4_METADATA: Option<TypeMetadata> = Some(TypeMetadata {
        name: "flags",
        description: "Processing flags",
        byte_alignment: 8,
        is_optional: true,
    });
    const INPUT5_METADATA: Option<TypeMetadata> = None;
    const INPUT6_METADATA: Option<TypeMetadata> = None;
}

#[derive(Component)]
pub struct ImageProcessingTaskOutput;
impl OutputSpec for ImageProcessingTaskOutput {
    type Output1 = u8; // Processed pixel data
    type Output2 = f32; // Analysis results
    type Output3 = Unused;
    type Output4 = Unused;
    type Output5 = Unused;
    type Output6 = Unused;
    const OUTPUT1_METADATA: Option<TypeMetadata> = Some(TypeMetadata {
        name: "processed_pixels",
        description: "Processed pixel data",
        byte_alignment: 1,
        is_optional: false,
    });
    const OUTPUT2_METADATA: Option<TypeMetadata> = Some(TypeMetadata {
        name: "analysis_results",
        description: "Analysis results",
        byte_alignment: 4,
        is_optional: false,
    });
    const OUTPUT3_METADATA: Option<TypeMetadata> = None;
    const OUTPUT4_METADATA: Option<TypeMetadata> = None;
    const OUTPUT5_METADATA: Option<TypeMetadata> = None;
    const OUTPUT6_METADATA: Option<TypeMetadata> = None;
}
// Generic task container that holds the data
pub struct Input<T: InputSpec> {
    input1: Option<Vec<T::Input1>>,
    input2: Option<Vec<T::Input2>>,
    input3: Option<Vec<T::Input3>>,
    input4: Option<Vec<T::Input4>>,
    input5: Option<Vec<T::Input5>>,
    input6: Option<Vec<T::Input6>>,
    _phantom: std::marker::PhantomData<T>,
}
pub struct Output<T: OutputSpec> {
    output1: Option<Vec<T::Output1>>,
    output2: Option<Vec<T::Output2>>,
    output3: Option<Vec<T::Output3>>,
    output4: Option<Vec<T::Output4>>,
    output5: Option<Vec<T::Output5>>,
    output6: Option<Vec<T::Output6>>,

    _phantom: std::marker::PhantomData<T>,
}

// Implementation for the task container
impl<T: InputSpec> Input<T> {
    pub fn new() -> Self {
        Input {
            input1: None,
            input2: None,
            input3: None,
            input4: None,
            input5: None,
            input6: None,

            _phantom: std::marker::PhantomData,
        }
    }

    // Type-safe setters that take vectors of Pod types
    pub fn set_input1(&mut self, input: Vec<T::Input1>) {
        self.input1 = Some(input);
    }

    pub fn set_input2(&mut self, input: Vec<T::Input2>) {
        self.input2 = Some(input);
    }
    pub fn set_input3(&mut self, input: Vec<T::Input3>) {
        self.input3 = Some(input);
    }
    pub fn set_input4(&mut self, input: Vec<T::Input4>) {
        self.input4 = Some(input);
    }
    pub fn set_input5(&mut self, input: Vec<T::Input5>) {
        self.input5 = Some(input);
    }
    pub fn set_input6(&mut self, input: Vec<T::Input6>) {
        self.input6 = Some(input);
    }
    // Add methods to access metadata
    pub fn get_input1_metadata() -> Option<&'static TypeMetadata> {
        T::INPUT1_METADATA.as_ref()
    }

    pub fn get_input2_metadata() -> Option<&'static TypeMetadata> {
        T::INPUT2_METADATA.as_ref()
    }

    pub fn get_input3_metadata() -> Option<&'static TypeMetadata> {
        T::INPUT3_METADATA.as_ref()
    }

    pub fn get_input4_metadata() -> Option<&'static TypeMetadata> {
        T::INPUT4_METADATA.as_ref()
    }

    pub fn get_input5_metadata() -> Option<&'static TypeMetadata> {
        T::INPUT5_METADATA.as_ref()
    }

    pub fn get_input6_metadata() -> Option<&'static TypeMetadata> {
        T::INPUT6_METADATA.as_ref()
    }

    // Optional: Helper method to get all metadata
    pub fn get_all_metadata() -> [Option<&'static TypeMetadata>; 6] {
        [
            T::INPUT1_METADATA.as_ref(),
            T::INPUT2_METADATA.as_ref(),
            T::INPUT3_METADATA.as_ref(),
            T::INPUT4_METADATA.as_ref(),
            T::INPUT5_METADATA.as_ref(),
            T::INPUT6_METADATA.as_ref(),
        ]
    }
    // Get raw bytes from inputs
    pub fn input1_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input1 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }

    pub fn input2_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input2 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
        // self.input2.as_ref().map(|data| bytemuck::cast_slice(data))
    }
    pub fn input3_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input3 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }
    pub fn input4_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input4 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }
    pub fn input5_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input5 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }
    pub fn input6_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input6 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }
}
impl<T: OutputSpec> Output<T> {
    pub fn new() -> Self {
        Output {
            output1: None,
            output2: None,
            output3: None,
            output4: None,
            output5: None,
            output6: None,

            _phantom: std::marker::PhantomData,
        }
    }
    pub fn get_output1_metadata() -> Option<&'static TypeMetadata> {
        T::OUTPUT1_METADATA.as_ref()
    }
    pub fn get_output2_metadata() -> Option<&'static TypeMetadata> {
        T::OUTPUT2_METADATA.as_ref()
    }
    pub fn get_output3_metadata() -> Option<&'static TypeMetadata> {
        T::OUTPUT3_METADATA.as_ref()
    }
    pub fn get_output4_metadata() -> Option<&'static TypeMetadata> {
        T::OUTPUT4_METADATA.as_ref()
    }
    pub fn get_output5_metadata() -> Option<&'static TypeMetadata> {
        T::OUTPUT5_METADATA.as_ref()
    }
    pub fn get_output6_metadata() -> Option<&'static TypeMetadata> {
        T::OUTPUT6_METADATA.as_ref()
    }
    pub fn get_all_metadata() -> [Option<&'static TypeMetadata>; 6] {
        [
            T::OUTPUT1_METADATA.as_ref(),
            T::OUTPUT2_METADATA.as_ref(),
            T::OUTPUT3_METADATA.as_ref(),
            T::OUTPUT4_METADATA.as_ref(),
            T::OUTPUT5_METADATA.as_ref(),
            T::OUTPUT6_METADATA.as_ref(),
        ]
    }

    // Set outputs from raw bytes
    pub fn set_output1_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() % std::mem::size_of::<T::Output1>() != 0 {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output1 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }

    pub fn set_output2_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() % std::mem::size_of::<T::Output2>() != 0 {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output2 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }
    pub fn set_output3_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() % std::mem::size_of::<T::Output3>() != 0 {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output3 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }
    pub fn set_output4_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() % std::mem::size_of::<T::Output4>() != 0 {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output4 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }
    pub fn set_output5_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() % std::mem::size_of::<T::Output5>() != 0 {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output5 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }
    pub fn set_output6_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() % std::mem::size_of::<T::Output6>() != 0 {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output6 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }

    // Type-safe getters for processed results
    pub fn get_output1(&self) -> Option<&[T::Output1]> {
        self.output1.as_deref()
    }

    pub fn get_output2(&self) -> Option<&[T::Output2]> {
        self.output2.as_deref()
    }
    pub fn get_output3(&self) -> Option<&[T::Output3]> {
        self.output3.as_deref()
    }
    pub fn get_output4(&self) -> Option<&[T::Output4]> {
        self.output4.as_deref()
    }
    pub fn get_output5(&self) -> Option<&[T::Output5]> {
        self.output5.as_deref()
    }
    pub fn get_output6(&self) -> Option<&[T::Output6]> {
        self.output6.as_deref()
    }
}

// Example processor that works with raw bytes
pub struct Processor;

impl Processor {
    pub fn process<T: InputSpec, O: OutputSpec>(
        input: &mut Input<T>,
        output: &mut Output<O>,
    ) -> Result<(), String> {
        // Get input bytes
        let input1_bytes = input.input1_bytes().ok_or("Input1 not set")?.to_vec();
        let input2_bytes = input.input2_bytes().ok_or("Input2 not set")?.to_vec();

        // Process the raw bytes...
        // For this example, we'll just copy the inputs to outputs
        output.set_output1_from_bytes(&input1_bytes)?;
        output.set_output2_from_bytes(&input2_bytes)?;

        Ok(())
    }
}

// Example usage
fn main() {
    // Create a new image processing task
    let mut input = Input::<ImageProcessingTaskInput>::new();
    let mut output = Output::<ImageProcessingTaskOutput>::new();

    // Set inputs
    let pixels: Vec<MyTuple> = vec![MyTuple([0.0, 0.0, 0.0]), MyTuple([1.0, 1.0, 1.0])];
    let params: Vec<u64> = vec![100, 200, 300];
    let two: Vec<ImageParams> = vec![ImageParams {
        scale: 1.0,
        threshold: 0.5,
        other: 0.1,
    }];

    input.set_input1(pixels);
    input.set_input2(two);
    input.set_input4(params);

    // We can get raw bytes from inputs
    let raw_pixels = input.input1_bytes().unwrap();
    let raw_params = input.input2_bytes().unwrap();
    println!("Raw pixel bytes: {} bytes", raw_pixels.len());
    println!("Raw params bytes: {} bytes", raw_params.len());

    // Process the task
    Processor::process(&mut input, &mut output).unwrap();

    //
    // let data: BufferView<'_> = slice.get_mapped_range();

    // Get outputs as strongly-typed slices
    let processed_pixels = output.get_output1();
    let analysis_results: Option<&[f32]> = output.get_output2();
}

// test pod capabilities

//* NO NEED TO HAVE A TON OF DIFFERENT INPUT AND OUTPUT TYPES, DATA CAN SIMPLY BE SENT NESTED AS SHOWN BELOW */
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct TestPod([u32; 32]); // Using a fixed-size array instead of Vec

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct TestPod2([f32; 2]);

#[repr(C)]
#[derive(Clone)]
pub struct TestPodVec {
    pub positions: Vec<Vec<Vec<(TestPod, TestPod2)>>>,
}
//

fn t() {
    let test_pod = [TestPod([0; 32]), TestPod([1; 32])];
    let test_pod2 = [TestPod2([0.0; 2]), TestPod2([1.0; 2])];
    let test_pod_vec = TestPodVec {
        positions: vec![vec![vec![
            (test_pod[0], test_pod2[0]),
            (test_pod[1], test_pod2[1]),
        ]]],
    };
    // let v: &[u8] = bytemuck::cast_slice(test_pod_vec.positions);
}
