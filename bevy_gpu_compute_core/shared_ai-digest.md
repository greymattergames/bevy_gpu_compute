# Cargo.toml

```toml
[package]
name = "bevy_gpu_compute_core"
version = "0.1.0"
edition = "2024"

[dependencies]
bytemuck = { version= "1.21.0", features=["derive"]}
# spl-pod = "0.5.0"
paste = "1.0.15"
# proc-macro2 = "1.0.92"
# quote = "1.0.38"
[features]
f16 = []  # Enable unstable f16 data type support (16 bit floating point numbers), which can improve gpu compute performance if used

[dev-dependencies]
pretty_assertions = "1.4.1"

```

# src\custom_type_name.rs

```rs
#[derive(Clone, Debug, PartialEq)]

pub struct CustomTypeName {
    name: String,
    upper: String,
    lower: String,
    input_array_length: String,
    input_array: String,
    output_array_length: String,
    output_array: String,
    counter: String,
    uniform: String,
}

impl CustomTypeName {
    pub fn new(name: &str) -> Self {
        let upper = name.to_uppercase();
        let lower = name.to_lowercase();
        Self {
            name: name.to_string(),
            upper: upper.clone(),
            lower: lower.clone(),
            input_array_length: format!("{}_INPUT_ARRAY_LENGTH", upper.clone()),
            input_array: format!("{}_input_array", lower.clone()),
            output_array_length: format!("{}_OUTPUT_ARRAY_LENGTH", upper),
            output_array: format!("{}_output_array", lower),
            counter: format!("{}_counter", lower),
            uniform: lower.clone(),
        }
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn upper(&self) -> &String {
        &self.upper
    }
    pub fn lower(&self) -> &String {
        &self.lower
    }
    pub fn input_array_length(&self) -> String {
        self.input_array_length.clone()
    }
    pub fn input_array(&self) -> String {
        self.input_array.clone()
    }

    pub fn output_array_length(&self) -> String {
        self.output_array_length.clone()
    }
    pub fn output_array(&self) -> String {
        self.output_array.clone()
    }

    pub fn counter(&self) -> String {
        self.counter.clone()
    }
    pub fn uniform(&self) -> String {
        self.uniform.clone()
    }
}

```

# src\lib.rs

```rs
#![feature(f16)]

pub mod custom_type_name;
pub mod misc_types;
pub mod wgsl_components;
pub mod wgsl_in_rust_helpers;
pub mod wgsl_shader_module;
pub mod wgsl_shader_module_lib_portion;
pub mod wgsl_wgpu_binding;

```

# src\misc_types.rs

```rs
use bytemuck::Pod;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]

pub struct _INTERNAL_UNUSED {}

pub trait TypesSpec {
    type InputConfigTypes: InputConfigTypesSpec;
    type InputArrayTypes: InputVectorTypesSpec;
    type OutputArrayTypes: OutputVectorTypesSpec;
}
#[derive(Debug)]

pub struct BlankInputVectorTypesSpec {}
impl InputVectorTypesSpec for BlankInputVectorTypesSpec {
    type Input0 = ();
    type Input1 = ();
    type Input2 = ();
    type Input3 = ();
    type Input4 = ();
    type Input5 = ();
}
#[derive(Debug)]

pub struct BlankInputConfigTypesSpec {}
impl InputConfigTypesSpec for BlankInputConfigTypesSpec {
    type Input0 = ();
    type Input1 = ();
    type Input2 = ();
    type Input3 = ();
    type Input4 = ();
    type Input5 = ();
}
#[derive(Debug)]

pub struct BlankOutputVectorTypesSpec {}
impl OutputVectorTypesSpec for BlankOutputVectorTypesSpec {
    type Output0 = ();
    type Output1 = ();
    type Output2 = ();
    type Output3 = ();
    type Output4 = ();
    type Output5 = ();
}
#[derive(Debug)]

pub struct BlankTypesSpec {}
impl TypesSpec for BlankTypesSpec {
    type InputConfigTypes = BlankInputConfigTypesSpec;
    type InputArrayTypes = BlankInputVectorTypesSpec;
    type OutputArrayTypes = BlankOutputVectorTypesSpec;
}

pub trait InputConfigTypesSpec {
    type Input0: Pod + Send + Sync + std::fmt::Debug;
    type Input1: Pod + Send + Sync + std::fmt::Debug;
    type Input2: Pod + Send + Sync + std::fmt::Debug;
    type Input3: Pod + Send + Sync + std::fmt::Debug;
    type Input4: Pod + Send + Sync + std::fmt::Debug;
    type Input5: Pod + Send + Sync + std::fmt::Debug;
}

pub trait InputVectorTypesSpec {
    type Input0: Pod + Send + Sync + std::fmt::Debug;
    type Input1: Pod + Send + Sync + std::fmt::Debug;
    type Input2: Pod + Send + Sync + std::fmt::Debug;
    type Input3: Pod + Send + Sync + std::fmt::Debug;
    type Input4: Pod + Send + Sync + std::fmt::Debug;
    type Input5: Pod + Send + Sync + std::fmt::Debug;
}

pub trait OutputVectorTypesSpec {
    type Output0: Pod + Send + Sync + std::fmt::Debug;
    type Output1: Pod + Send + Sync + std::fmt::Debug;
    type Output2: Pod + Send + Sync + std::fmt::Debug;
    type Output3: Pod + Send + Sync + std::fmt::Debug;
    type Output4: Pod + Send + Sync + std::fmt::Debug;
    type Output5: Pod + Send + Sync + std::fmt::Debug;
}

```

# src\wgsl_components.rs

```rs
use crate::custom_type_name::CustomTypeName;

/// includes just the parts the user has input, with any relevant metadata necessary for the library to complete the module

#[derive(Debug, Clone, PartialEq)]
pub struct WgslShaderModuleSectionCode {
    pub rust_code: String,
    pub wgsl_code: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WgslShaderModuleUserPortion {
    /// defined with the "const" keyword
    /// single line
    /// value remains static
    /// type must be wgsl type or created somewhere else in the module
    /// value could be a type instantiation, a scalar, or a function
    pub static_consts: Vec<WgslConstAssignment>,
    /// defined with either struct keyword, or a type alias
    /// These are not associated with any buffers and exist only on the GPU
    pub helper_types: Vec<WgslType>,
    /// identified with a #[config_input] attribute above them
    pub uniforms: Vec<WgslType>,
    /// identified with a #[vec_input] attribute above them
    pub input_arrays: Vec<WgslInputArray>,
    /// identified with a #[vec_output] attribute above them
    pub output_arrays: Vec<WgslOutputArray>,
    /// any function that appears besides the one called "main"
    pub helper_functions: Vec<WgslFunction>,
    /// the main function, identified by its name: "main"
    /// MUST contain a single parameter called "global_id" of type "WgslGlobalId"
    /// look for any attempt to ASSIGN to the value of "global_id.x", "global_id.y", or "global_id.z" or just "global_id" and throw an error
    pub main_function: Option<WgslFunction>,
}
impl WgslShaderModuleUserPortion {
    pub fn empty() -> Self {
        Self {
            static_consts: vec![],
            helper_types: vec![],
            uniforms: vec![],
            input_arrays: vec![],
            output_arrays: vec![],
            helper_functions: vec![],
            main_function: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WgslType {
    pub name: CustomTypeName,
    pub code: WgslShaderModuleSectionCode,
}

#[derive(Clone, Debug, PartialEq)]

pub struct WgslFunction {
    pub name: String,
    pub code: WgslShaderModuleSectionCode,
}

#[derive(Clone, Debug, PartialEq)]
/// assignments using let can happen within functions and we don't care about them, we don't need to change anything
pub struct WgslConstAssignment {
    pub code: WgslShaderModuleSectionCode,
}

impl WgslConstAssignment {
    pub fn new(name: &str, scalar_type: &str, value: &str) -> Self {
        Self {
            code: WgslShaderModuleSectionCode {
                rust_code: format!("const {}: {} = {};", name, scalar_type, value),
                wgsl_code: format!("override {}: {} = {};", name, scalar_type, value),
            },
        }
    }
    pub fn no_default(name: &str, scalar_type: &str) -> Self {
        Self {
            code: WgslShaderModuleSectionCode {
                rust_code: format!("const {}: {};", name, scalar_type),
                wgsl_code: format!("override {}: {};", name, scalar_type),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]

pub struct WgslInputArray {
    pub item_type: WgslType,
}

#[derive(Clone, Debug, PartialEq)]

pub struct WgslOutputArray {
    pub item_type: WgslType,
    pub atomic_counter_name: Option<String>,
}

pub enum WgpuShaderType {
    Compute,
    Vertex,
    Fragment,
}
impl ToString for WgpuShaderType {
    fn to_string(&self) -> String {
        match self {
            WgpuShaderType::Compute => "compute".to_string(),
            WgpuShaderType::Vertex => panic!("Vertex shaders not yet supported"),
            WgpuShaderType::Fragment => panic!("Fragment shaders not yet supported"),
        }
    }
}

pub const WORKGROUP_SIZE_X_VAR_NAME: &str = "_LIB_WORKGROUP_SIZE_X";
pub const WORKGROUP_SIZE_Y_VAR_NAME: &str = "_LIB_WORKGROUP_SIZE_Y";
pub const WORKGROUP_SIZE_Z_VAR_NAME: &str = "_LIB_WORKGROUP_SIZE_Z";
pub struct WgslWorkgroupDeclaration {
    pub shader_type: WgpuShaderType,
}
impl ToString for WgslWorkgroupDeclaration {
    fn to_string(&self) -> String {
        return format!(
            "@{} @workgroup_size({}, {}, {})\n",
            self.shader_type.to_string(),
            WORKGROUP_SIZE_X_VAR_NAME,
            WORKGROUP_SIZE_Y_VAR_NAME,
            WORKGROUP_SIZE_Z_VAR_NAME
        );
    }
}

```

# src\wgsl_in_rust_helpers\matrices.rs

```rs
use crate::wgsl_in_rust_helpers::vectors::*;
macro_rules! impl_matrix {
    ($name:ident, $vec_type:ty, $($field:ident, $index:expr),+) => {
        #[repr(C)]
        #[derive(Debug, Clone,Copy, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct $name {
            $(pub $field: $vec_type,)+
            _force_constructor: ()
        }

        impl $name {
            pub fn new($($field: $vec_type),+) -> Self {
                Self {
                    $($field,)+
                    _force_constructor: ()
                }
            }

            // Generate setters for row access (x,y,z,w)
            $(
                paste::paste! {
                    pub fn [<set_ $field>](&mut self, value: $vec_type) {
                        self.$field = value;
                    }
                }
            )+
        }

        impl std::ops::Index<i32> for $name {
            type Output = $vec_type;

            fn index(&self, index: i32) -> &Self::Output {
                match index {
                    $($index => &self.$field,)+
                    _ => panic!("Index out of bounds"),
                }
            }
        }

        impl std::ops::IndexMut<i32> for $name {
            fn index_mut(&mut self, index: i32) -> &mut Self::Output {
                match index {
                    $($index => &mut self.$field,)+
                    _ => panic!("Index out of bounds"),
                }
            }
        }
    }
}
macro_rules! impl_matrix_no_pod {
    ($name:ident, $vec_type:ty, $($field:ident, $index:expr),+) => {
        pub struct $name {
            $(pub $field: $vec_type,)+
            _force_constructor: ()
        }

        impl $name {
            pub fn new($($field: $vec_type),+) -> Self {
                Self {
                    $($field,)+
                    _force_constructor: ()
                }
            }

            // Generate setters for row access (x,y,z,w)
            $(
                paste::paste! {
                    pub fn [<set_ $field>](&mut self, value: $vec_type) {
                        self.$field = value;
                    }
                }
            )+
        }

        impl std::ops::Index<i32> for $name {
            type Output = $vec_type;

            fn index(&self, index: i32) -> &Self::Output {
                match index {
                    $($index => &self.$field,)+
                    _ => panic!("Index out of bounds"),
                }
            }
        }

        impl std::ops::IndexMut<i32> for $name {
            fn index_mut(&mut self, index: i32) -> &mut Self::Output {
                match index {
                    $($index => &mut self.$field,)+
                    _ => panic!("Index out of bounds"),
                }
            }
        }
    }
}

macro_rules! define_matrix_types {
    ($scalar_type:ty, $suffix:ident) => {
        paste::paste! {
            // Mat2xN types
            impl_matrix!([<Mat2x2 $suffix>], [<Vec2 $suffix>], x, 0, y, 1);
            impl_matrix!([<Mat2x3 $suffix>], [<Vec3 $suffix>], x, 0, y, 1);
            impl_matrix!([<Mat2x4 $suffix>], [<Vec4 $suffix>], x, 0, y, 1);

            // Mat3xN types
            impl_matrix!([<Mat3x2 $suffix>], [<Vec2 $suffix>], x, 0, y, 1, z, 2);
            impl_matrix!([<Mat3x3 $suffix>], [<Vec3 $suffix>], x, 0, y, 1, z, 2);
            impl_matrix!([<Mat3x4 $suffix>], [<Vec4 $suffix>], x, 0, y, 1, z, 2);

            // Mat4xN types
            impl_matrix!([<Mat4x2 $suffix>], [<Vec2 $suffix>], x, 0, y, 1, z, 2, w, 3);
            impl_matrix!([<Mat4x3 $suffix>], [<Vec3 $suffix>], x, 0, y, 1, z, 2, w, 3);
            impl_matrix!([<Mat4x4 $suffix>], [<Vec4 $suffix>], x, 0, y, 1, z, 2, w, 3);
        }
    };
}
macro_rules! define_matrix_types_no_pod {
    ($scalar_type:ty, $suffix:ident) => {
        paste::paste! {
            // Mat2xN types
            impl_matrix_no_pod!([<Mat2x2 $suffix>], [<Vec2 $suffix>], x, 0, y, 1);
            impl_matrix_no_pod!([<Mat2x3 $suffix>], [<Vec3 $suffix>], x, 0, y, 1);
            impl_matrix_no_pod!([<Mat2x4 $suffix>], [<Vec4 $suffix>], x, 0, y, 1);

            // Mat3xN types
            impl_matrix_no_pod!([<Mat3x2 $suffix>], [<Vec2 $suffix>], x, 0, y, 1, z, 2);
            impl_matrix_no_pod!([<Mat3x3 $suffix>], [<Vec3 $suffix>], x, 0, y, 1, z, 2);
            impl_matrix_no_pod!([<Mat3x4 $suffix>], [<Vec4 $suffix>], x, 0, y, 1, z, 2);

            // Mat4xN types
            impl_matrix_no_pod!([<Mat4x2 $suffix>], [<Vec2 $suffix>], x, 0, y, 1, z, 2, w, 3);
            impl_matrix_no_pod!([<Mat4x3 $suffix>], [<Vec3 $suffix>], x, 0, y, 1, z, 2, w, 3);
            impl_matrix_no_pod!([<Mat4x4 $suffix>], [<Vec4 $suffix>], x, 0, y, 1, z, 2, w, 3);
        }
    };
}

// Generate concrete types for numeric types (matrices don't make sense for booleans)
define_matrix_types!(u32, U32);
define_matrix_types!(i32, I32);
define_matrix_types!(f32, F32);
define_matrix_types!(f16, F16);
define_matrix_types_no_pod!(bool, Bool);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mat3x4_f32_creation_and_access() {
        let vec1 = Vec4F32::new(1.0, 2.0, 3.0, 4.0);
        let vec2 = Vec4F32::new(5.0, 6.0, 7.0, 8.0);
        let vec3 = Vec4F32::new(9.0, 10.0, 11.0, 12.0);

        let mut mat = Mat3x4F32::new(vec1, vec2, vec3);

        // Test direct field access
        assert_eq!(mat.x.x, 1.0);
        assert_eq!(mat.y.y, 6.0);
        assert_eq!(mat.z.z, 11.0);

        // Test index access
        assert_eq!(mat[0][0], 1.0);
        assert_eq!(mat[1][1], 6.0);
        assert_eq!(mat[2][2], 11.0);

        // Test setters
        let new_vec = Vec4F32::new(13.0, 14.0, 15.0, 16.0);
        mat.set_x(new_vec);

        assert_eq!(mat.x.w, 16.0);
        assert_eq!(mat[0][3], 16.0);
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_mat3x4_f32_invalid_index() {
        let vec1 = Vec4F32::new(1.0, 2.0, 3.0, 4.0);
        let vec2 = Vec4F32::new(5.0, 6.0, 7.0, 8.0);
        let vec3 = Vec4F32::new(9.0, 10.0, 11.0, 12.0);

        let mat = Mat3x4F32::new(vec1, vec2, vec3);
        let _value = mat[3]; // Should panic
    }
    #[test]
    fn test_mat4x2_bool_access_and_mutation() {
        let vec1 = Vec2Bool::new(true, false);
        let vec2 = Vec2Bool::new(false, true);
        let vec3 = Vec2Bool::new(true, true);
        let vec4 = Vec2Bool::new(false, false);

        let mut mat = Mat4x2Bool::new(vec1, vec2, vec3, vec4);

        // Test direct field access
        assert_eq!(mat.x.x, true);
        assert_eq!(mat.y.y, true);
        assert_eq!(mat.z.x, true);
        assert_eq!(mat.w.y, false);

        // Test index access
        assert_eq!(mat[0][0], true);
        assert_eq!(mat[1][1], true);
        assert_eq!(mat[2][0], true);
        assert_eq!(mat[3][1], false);

        // Test setters
        let new_vec = Vec2Bool::new(false, true);
        mat.set_x(new_vec);

        assert_eq!(mat.x.y, true);
        assert_eq!(mat[0][1], true);
    }
}

```

# src\wgsl_in_rust_helpers\mod.rs

```rs
pub mod matrices;
pub mod pod_f16;
pub mod vectors;
pub use matrices::*;
pub use pod_f16::*;
pub use vectors::*;

/// This is a representation of wgpu "GlobalId", but for ease of understanding we have renamed it
pub struct WgslIterationPosition {
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
    pub fn vec_val<T>(_index: u32) -> T {
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
    pub fn push<T>(_val: T) {
        unimplemented!()
    }
    pub fn set<T>(_index: u32, _val: T) {
        unimplemented!()
    }
    /// returns the user-input maximum number of elements that can be stored in the output for this specific type.
    pub fn len<T>() -> u32 {
        unimplemented!()
    }
}

```

# src\wgsl_in_rust_helpers\pod_f16.rs

```rs
use bytemuck::{Pod, Zeroable};

/// A 16-bit floating point number that implements Pod
/// Includes padding to ensure 32-bit alignment
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)] // Ensure consistent memory layout
pub struct PodF16 {
    pub value: f16,
}

// Implement Zeroable
unsafe impl Zeroable for PodF16 {}

// Implement Pod
unsafe impl Pod for PodF16 {}

// Implement conversions
impl PodF16 {
    pub fn new(value: f16) -> Self {
        Self { value }
    }

    pub fn get(&self) -> f16 {
        self.value
    }
}

impl From<f16> for PodF16 {
    fn from(value: f16) -> Self {
        Self::new(value)
    }
}

impl From<PodF16> for f16 {
    fn from(pod: PodF16) -> Self {
        pod.value
    }
}

impl From<f32> for PodF16 {
    fn from(value: f32) -> Self {
        Self::new(value as f16)
    }
}

impl From<PodF16> for f32 {
    fn from(pod: PodF16) -> Self {
        pod.value as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_and_alignment() {
        // Should be 32 bits (4 bytes) total
        assert_eq!(std::mem::size_of::<PodF16>(), 4);
        // Should have 32-bit alignment
        assert_eq!(std::mem::align_of::<PodF16>(), 4);
    }

    #[test]
    fn test_f16_pod() {
        let value = PodF16::from(3.14_f32);

        // Test casting to bytes
        let bytes: &[u8] = bytemuck::bytes_of(&value);
        assert_eq!(bytes.len(), 4); // Should be 4 bytes now

        // Test casting from bytes back to PodF16
        let restored: &PodF16 = bytemuck::from_bytes(bytes);
        assert_eq!(*restored, value);

        // Test zero initialization
        let zero = PodF16::zeroed();
        assert_eq!(zero, PodF16::from(0.0_f32));

        // Test casting slice
        let values = vec![
            PodF16::from(1.0_f32),
            PodF16::from(2.0_f32),
            PodF16::from(3.0_f32),
        ];
        let bytes: &[u8] = bytemuck::cast_slice(&values);
        let restored: &[PodF16] = bytemuck::cast_slice(bytes);
        assert_eq!(restored, values.as_slice());
    }

    #[test]
    fn test_conversions() {
        let original = 3.14_f32;
        let pod = PodF16::from(original);
        let roundtrip: f32 = pod.into();
        // Note: Some precision loss is expected due to f16
        assert!((original - roundtrip).abs() < 0.01);
    }
    #[test]
    fn can_use_with_bytemuck() {
        pub mod test_module {
            use crate::wgsl_in_rust_helpers::PodF16;
            /// user types
            #[repr(C)]
            #[allow(dead_code)]
            pub struct MyConfig {
                f16_val: PodF16,
            }
            const _: fn() = || {
                #[doc(hidden)]
                struct TypeWithoutPadding([u8; ::core::mem::size_of::<PodF16>()]);
                let _ = ::core::mem::transmute::<MyConfig, TypeWithoutPadding>;
            };

            const _: fn() = || {
                #[allow(clippy::missing_const_for_fn)]
                #[doc(hidden)]
                fn check() {
                    fn assert_impl<T: ::bytemuck::Pod>() {}
                    assert_impl::<PodF16>();
                }
            };
        }
    }
}

```

# src\wgsl_in_rust_helpers\vectors.rs

```rs
use super::PodF16;

macro_rules! impl_vector {
    ($name:ident, $type:ty, $($field:ident, $index:expr),+) => {
        #[repr(C)]
        #[derive(Debug, Clone,Copy, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct $name {
            $(pub $field: $type,)+
            _force_constructor: ()
        }

        impl $name {
            pub fn new($($field: $type),+) -> Self {
                Self {
                    $($field,)+
                    _force_constructor: ()
                }
            }

            // Generate setters for coordinate access (x,y,z,w)
            $(
                paste::paste! {
                    pub fn [<set_ $field>](&mut self, value: $type) {
                        self.$field = value;
                    }
                }
            )+
        }

        impl std::ops::Index<i32> for $name {
            type Output = $type;

            fn index(&self, index: i32) -> &Self::Output {
                match index {
                    $($index => &self.$field,)+
                    _ => panic!("Index out of bounds"),
                }
            }
        }

        impl std::ops::IndexMut<i32> for $name {
            fn index_mut(&mut self, index: i32) -> &mut Self::Output {
                match index {
                    $($index => &mut self.$field,)+
                    _ => panic!("Index out of bounds"),
                }
            }
        }
    }
}

macro_rules! impl_vector_no_pod {
    ($name:ident, $type:ty, $($field:ident, $index:expr),+) => {
        pub struct $name {
            $(pub $field: $type,)+
            _force_constructor: ()
        }

        impl $name {
            pub fn new($($field: $type),+) -> Self {
                Self {
                    $($field,)+
                    _force_constructor: ()
                }
            }

            // Generate setters for coordinate access (x,y,z,w)
            $(
                paste::paste! {
                    pub fn [<set_ $field>](&mut self, value: $type) {
                        self.$field = value;
                    }
                }
            )+
        }

        impl std::ops::Index<i32> for $name {
            type Output = $type;

            fn index(&self, index: i32) -> &Self::Output {
                match index {
                    $($index => &self.$field,)+
                    _ => panic!("Index out of bounds"),
                }
            }
        }

        impl std::ops::IndexMut<i32> for $name {
            fn index_mut(&mut self, index: i32) -> &mut Self::Output {
                match index {
                    $($index => &mut self.$field,)+
                    _ => panic!("Index out of bounds"),
                }
            }
        }
    }
}

macro_rules! define_vector_types {
    ($type:ty, $suffix:ident) => {
        paste::paste! {
            impl_vector!([<Vec2 $suffix>], $type, x, 0, y, 1);
            impl_vector!([<Vec3 $suffix>], $type, x, 0, y, 1, z, 2);
            impl_vector!([<Vec4 $suffix>], $type, x, 0, y, 1, z, 2, w, 3);


        }
    };
}
macro_rules! define_vector_types_no_pod {
    ($type:ty, $suffix:ident) => {
        paste::paste! {
            impl_vector_no_pod!([<Vec2 $suffix>], $type, x, 0, y, 1);
            impl_vector_no_pod!([<Vec3 $suffix>], $type, x, 0, y, 1, z, 2);
            impl_vector_no_pod!([<Vec4 $suffix>], $type, x, 0, y, 1, z, 2, w, 3);


        }
    };
}

// Generate concrete types for all scalar types
define_vector_types!(u32, U32);
define_vector_types!(i32, I32);
define_vector_types!(f32, F32);
define_vector_types!(PodF16, F16);
define_vector_types_no_pod!(bool, Bool);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec4_f32_creation_and_access() {
        let mut vec4 = Vec4F32::new(1.0, 2.0, 3.0, 4.0);
        // Test direct field access
        assert_eq!(vec4.x, 1.0);
        assert_eq!(vec4.y, 2.0);
        assert_eq!(vec4.z, 3.0);
        assert_eq!(vec4.w, 4.0);

        // Test index access
        assert_eq!(vec4[0], 1.0);
        assert_eq!(vec4[1], 2.0);
        assert_eq!(vec4[2], 3.0);
        assert_eq!(vec4[3], 4.0);

        // Test setters
        vec4.x = 5.0;
        vec4[2] = 7.0;

        assert_eq!(vec4.x, 5.0);
        assert_eq!(vec4.y, 2.0);
        assert_eq!(vec4.z, 7.0);
        assert_eq!(vec4.w, 4.0);
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_vec4_f32_invalid_index() {
        let vec4 = Vec4F32::new(1.0, 2.0, 3.0, 4.0);
        let _value = vec4[4]; // Should panic
    }
}

```

# src\wgsl_shader_module_lib_portion.rs

```rs
use super::{
    wgsl_components::{
        WgslConstAssignment, WgslFunction, WgslShaderModuleUserPortion, WgslType,
        WgslWorkgroupDeclaration,
    },
    wgsl_wgpu_binding::WgslWgpuBinding,
};
use crate::wgsl_components::{
    WORKGROUP_SIZE_X_VAR_NAME, WORKGROUP_SIZE_Y_VAR_NAME, WORKGROUP_SIZE_Z_VAR_NAME, WgpuShaderType,
};

pub struct WgslShaderModuleDerivedPortion {
    // generate these based on inputs and outputs
    pub pipeline_consts: Vec<WgslConstAssignment>,
    /// currently unused
    pub uniforms: Vec<WgslType>,
    /// currently unused
    pub helper_functions: Vec<WgslFunction>,
    /// static, generate automatically from the user portion
    pub bindings: Vec<WgslWgpuBinding>,
    /// static, workgroup sizes changed via pipeline consts
    pub workgroups_declaration: WgslWorkgroupDeclaration,
}

impl From<&WgslShaderModuleUserPortion> for WgslShaderModuleDerivedPortion {
    fn from(user_portion: &WgslShaderModuleUserPortion) -> Self {
        let mut pipeline_consts = vec![
            WgslConstAssignment::no_default(WORKGROUP_SIZE_X_VAR_NAME, "u32"),
            WgslConstAssignment::no_default(WORKGROUP_SIZE_Y_VAR_NAME, "u32"),
            WgslConstAssignment::no_default(WORKGROUP_SIZE_Z_VAR_NAME, "u32"),
        ];
        let mut binding_counter = 0;
        let mut bindings = Vec::new();
        user_portion.uniforms.iter().for_each(|u| {
            bindings.push(WgslWgpuBinding::uniform(
                0,
                binding_counter,
                u.name.uniform(),
                u.name.name(),
            ));
            binding_counter += 1;
        });
        user_portion.input_arrays.iter().for_each(|a| {
            pipeline_consts.push(WgslConstAssignment::no_default(
                &a.item_type.name.input_array_length(),
                "u32",
            ));
            bindings.push(WgslWgpuBinding::input_array(
                0,
                binding_counter,
                a.item_type.name.input_array(),
                format!("array < {} >", a.item_type.name.name(),),
            ));
            binding_counter += 1;
        });
        user_portion.output_arrays.iter().for_each(|a| {
            pipeline_consts.push(WgslConstAssignment::no_default(
                &a.item_type.name.output_array_length(),
                "u32",
            ));
            let output_array = WgslWgpuBinding::output_array(
                0,
                binding_counter,
                a.item_type.name.output_array(),
                format!("array < {} >", a.item_type.name.name(),),
            );
            bindings.push(output_array.clone());
            binding_counter += 1;
            if let Some(_) = &a.atomic_counter_name {
                bindings.push(WgslWgpuBinding::counter(binding_counter, &a, &output_array));
                binding_counter += 1;
            }
        });
        WgslShaderModuleDerivedPortion {
            pipeline_consts,
            uniforms: Vec::new(),
            helper_functions: Vec::new(),
            bindings: bindings,
            workgroups_declaration: WgslWorkgroupDeclaration {
                shader_type: WgpuShaderType::Compute,
            },
        }
    }
}

// impl PartialEq for WgslWgpuBinding {
//     fn eq(&self, other: &Self) -> bool {
//         // Implement comparison logic based on the fields of WgslWgpuBinding
//         // This is a placeholder implementation
//         self.binding_type == other.binding_type &&
//         self.group == other.group &&
//         self.binding == other.binding &&
//         self.name == other.name &&
//         self.resource_type == other.resource_type
//     }
// }

#[cfg(test)]

mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        custom_type_name::CustomTypeName,
        wgsl_components::{WgslInputArray, WgslOutputArray, WgslShaderModuleSectionCode},
        wgsl_shader_module::{IterSpaceDimmension, WgslShaderModule},
    };

    use super::*;

    #[test]
    fn test_wgsl_shader_module_library_portion_from_user_portion() {
        let user_portion = WgslShaderModuleUserPortion { static_consts: vec![WgslConstAssignment { code: WgslShaderModuleSectionCode { rust_code: "const example_module_const : u32 = 42;".to_string(), wgsl_code: "const example_module_const : u32 = 42;".to_string() } }], helper_types: vec![], uniforms: vec![WgslType { name: CustomTypeName::new("Uniforms"), code: WgslShaderModuleSectionCode { rust_code: "#[wgsl_config] struct Uniforms { time : f32, resolution : Vec2F32, }".to_string(), wgsl_code: "struct Uniforms { time : f32, resolution : vec2 < f32 > , }".to_string() } }], input_arrays: vec![WgslInputArray { item_type: WgslType { name: CustomTypeName::new("Position"), code: WgslShaderModuleSectionCode { rust_code: "#[wgsl_input_array] type Position = [f32; 2];".to_string(), wgsl_code: "alias Position  = array < f32, 2 > ;".to_string() } } }, WgslInputArray { item_type: WgslType { name: CustomTypeName::new("Radius") , code: WgslShaderModuleSectionCode { rust_code: "#[wgsl_input_array] type Radius = f32;".to_string(), wgsl_code: "alias Radius  = f32;".to_string() } }}], output_arrays: vec![WgslOutputArray { item_type: WgslType { name: CustomTypeName::new("CollisionResult"), code: WgslShaderModuleSectionCode { rust_code: "#[wgsl_output_vec] struct CollisionResult { entity1 : u32, entity2 : u32, }".to_string(), wgsl_code: "struct CollisionResult { entity1 : u32, entity2 : u32, }".to_string() } }, atomic_counter_name: Some("collisionresult_counter".to_string()) }], helper_functions: vec![WgslFunction { name: "calculate_distance_squared".to_string(), code: WgslShaderModuleSectionCode { rust_code: "fn calculate_distance_squared(p1 : [f32; 2], p2 : [f32; 2]) -> f32\n{\n    let dx = p1 [0] - p2 [0]; let dy = p1 [1] - p2 [1]; return dx * dx + dy *\n    dy;\n}".to_string(), wgsl_code: "fn calculate_distance_squared(p1 : array < f32, 2 > , p2 : array < f32, 2 >)\n-> f32\n{\n    let dx = p1 [0] - p2 [0]; let dy = p1 [1] - p2 [1]; return dx * dx + dy *\n    dy;\n}".to_string() } }], main_function: Some(WgslFunction { name: "main".to_owned(), code: WgslShaderModuleSectionCode { rust_code: "fn main(iter_pos : WgslIterationPosition)\n{\n    let current_entity = iter_pos.x; let other_entity = iter_pos.y; if\n    current_entity >= POSITION_INPUT_ARRAY_LENGTH || other_entity >=\n    POSITION_INPUT_ARRAY_LENGTH || current_entity == other_entity ||\n    current_entity >= other_entity { return; } let current_radius =\n    radius_input_array [current_entity]; let other_radius = radius_input_array\n    [other_entity]; if current_radius <= 0.0 || other_radius <= 0.0\n    { return; } let current_pos = position_input_array [current_entity]; let\n    other_pos = position_input_array [other_entity]; let dist_squared =\n    calculate_distance_squared(current_pos, other_pos); let radius_sum =\n    current_radius + other_radius; if dist_squared < radius_sum * radius_sum\n    {\n        {\n            let collisionresult_output_array_index =\n            atomicAdd(& collisionresult_counter, 1u); if\n            collisionresult_output_array_index <\n            COLLISIONRESULT_OUTPUT_ARRAY_LENGTH\n            {\n                collisionresult_output_array\n                [collisionresult_output_array_index] = CollisionResult\n                { entity1 : current_entity, entity2 : other_entity, };\n            }\n        };\n    }\n}".to_owned(), wgsl_code: "fn main(@builtin(global_invocation_id) iter_pos: vec3<u32>)\n{\n    let current_entity = iter_pos.x; let other_entity = iter_pos.y; if\n    current_entity >= POSITION_INPUT_ARRAY_LENGTH || other_entity >=\n    POSITION_INPUT_ARRAY_LENGTH || current_entity == other_entity ||\n    current_entity >= other_entity { return; } let current_radius =\n    radius_input_array [current_entity]; let other_radius = radius_input_array\n    [other_entity]; if current_radius <= 0.0 || other_radius <= 0.0\n    { return; } let current_pos = position_input_array [current_entity]; let\n    other_pos = position_input_array [other_entity]; let dist_squared =\n    calculate_distance_squared(current_pos, other_pos); let radius_sum =\n    current_radius + other_radius; if dist_squared < radius_sum * radius_sum\n    {\n        {\n            let collisionresult_output_array_index =\n            atomicAdd(& collisionresult_counter, 1u); if\n            collisionresult_output_array_index <\n            COLLISIONRESULT_OUTPUT_ARRAY_LENGTH\n            {\n                collisionresult_output_array\n                [collisionresult_output_array_index] = CollisionResult\n                { entity1 : current_entity, entity2 : other_entity, };\n            }\n        };\n    }\n}".to_owned() } }) };

        let expected_wgsl_code = "const example_module_const : u32 = 42;
override _LIB_WORKGROUP_SIZE_X: u32;
override _LIB_WORKGROUP_SIZE_Y: u32;
override _LIB_WORKGROUP_SIZE_Z: u32;
override POSITION_INPUT_ARRAY_LENGTH: u32;
override RADIUS_INPUT_ARRAY_LENGTH: u32;
override COLLISIONRESULT_OUTPUT_ARRAY_LENGTH: u32;
struct Uniforms { time : f32, resolution : vec2 < f32 > , }
alias Position  = array < f32, 2 > ;
alias Radius  = f32;
struct CollisionResult { entity1 : u32, entity2 : u32, }
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@group(0) @binding(1) var<storage, read> position_input_array: array < Position >;

@group(0) @binding(2) var<storage, read> radius_input_array: array < Radius >;

@group(0) @binding(3) var<storage, read_write> collisionresult_output_array: array < CollisionResult >;

@group(0) @binding(4) var<storage, read_write> collisionresult_counter: atomic<u32>;

fn calculate_distance_squared(p1 : array < f32, 2 > , p2 : array < f32, 2 >)
-> f32
{
    let dx = p1 [0] - p2 [0]; let dy = p1 [1] - p2 [1]; return dx * dx + dy *
    dy;
}
@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) iter_pos: vec3<u32>)
{
    let current_entity = iter_pos.x; let other_entity = iter_pos.y; if
    current_entity >= POSITION_INPUT_ARRAY_LENGTH || other_entity >=
    POSITION_INPUT_ARRAY_LENGTH || current_entity == other_entity ||
    current_entity >= other_entity { return; } let current_radius =
    radius_input_array [current_entity]; let other_radius = radius_input_array
    [other_entity]; if current_radius <= 0.0 || other_radius <= 0.0
    { return; } let current_pos = position_input_array [current_entity]; let
    other_pos = position_input_array [other_entity]; let dist_squared =
    calculate_distance_squared(current_pos, other_pos); let radius_sum =
    current_radius + other_radius; if dist_squared < radius_sum * radius_sum
    {
        {
            let collisionresult_output_array_index =
            atomicAdd(& collisionresult_counter, 1u); if
            collisionresult_output_array_index <
            COLLISIONRESULT_OUTPUT_ARRAY_LENGTH
            {
                collisionresult_output_array
                [collisionresult_output_array_index] = CollisionResult
                { entity1 : current_entity, entity2 : other_entity, };
            }
        };
    }
}
";
        let module = WgslShaderModule::new(user_portion);
        assert_eq!(
            module.wgsl_code(IterSpaceDimmension::OneD),
            expected_wgsl_code
        );
    }
}

```

# src\wgsl_shader_module.rs

```rs
use crate::wgsl_shader_module_lib_portion::WgslShaderModuleDerivedPortion;

use super::wgsl_components::WgslShaderModuleUserPortion;

#[derive(Debug, Clone, PartialEq, Hash, Copy)]
pub enum IterSpaceDimmension {
    OneD,
    TwoD,
    ThreeD,
}
impl IterSpaceDimmension {
    pub fn to_usize(&self) -> usize {
        match self {
            IterSpaceDimmension::OneD => 1,
            IterSpaceDimmension::TwoD => 2,
            IterSpaceDimmension::ThreeD => 3,
        }
    }
}

pub struct WgslShaderModule {
    pub user_portion: WgslShaderModuleUserPortion,
    pub library_portion: WgslShaderModuleDerivedPortion,
}
impl WgslShaderModule {
    pub fn new(module: WgslShaderModuleUserPortion) -> WgslShaderModule {
        let library_portion = WgslShaderModuleDerivedPortion::from(&module);
        WgslShaderModule {
            user_portion: module,
            library_portion: library_portion,
        }
    }
    pub fn wgsl_code(&self, iter_space_dimmensions: IterSpaceDimmension) -> String {
        let mut wgsl: String = String::new();
        // first add user static consts
        self.user_portion
            .static_consts
            .iter()
            .for_each(|c| wgsl.push_str_w_newline(&c.code.wgsl_code.clone()));
        // then add any miscelanious user helper types which are internal to the GPU only, not transfered to or from th CPU
        self.user_portion.helper_types.iter().for_each(|t| {
            wgsl.push_str_w_newline(&t.code.wgsl_code.clone());
        });
        // then add library pipeline consts
        // these include lengths of arrays, and workgroup sizes
        self.library_portion.pipeline_consts.iter().for_each(|c| {
            wgsl.push_str_w_newline(&c.code.wgsl_code.clone());
        });
        // then add user uniform definitions
        self.user_portion.uniforms.iter().for_each(|u| {
            wgsl.push_str_w_newline(&u.code.wgsl_code.clone());
        });
        // then add library uniform definitions
        self.library_portion.uniforms.iter().for_each(|u| {
            wgsl.push_str_w_newline(&u.code.wgsl_code.clone());
        });
        // then add user input array definitions
        self.user_portion.input_arrays.iter().for_each(|a| {
            wgsl.push_str_w_newline(&a.item_type.code.wgsl_code.clone());
        });
        // then add user output array definitions
        self.user_portion.output_arrays.iter().for_each(|a| {
            wgsl.push_str_w_newline(&a.item_type.code.wgsl_code.clone());
        });
        // now add wgpu bindings
        self.library_portion.bindings.iter().for_each(|b| {
            wgsl.push_str_w_newline(&b.to_string());
        });
        // now add user helper functions
        self.user_portion.helper_functions.iter().for_each(|f| {
            wgsl.push_str_w_newline(&f.code.wgsl_code.clone());
        });
        // now add library helper functions
        self.library_portion.helper_functions.iter().for_each(|f| {
            wgsl.push_str_w_newline(&f.code.wgsl_code.clone());
        });
        // now add the main function
        if iter_space_dimmensions == IterSpaceDimmension::OneD {
            wgsl.push_str_w_newline("@compute @workgroup_size(64, 1, 1)");
        } else if iter_space_dimmensions == IterSpaceDimmension::TwoD {
            wgsl.push_str_w_newline("@compute @workgroup_size(8, 8, 1)");
        } else {
            wgsl.push_str_w_newline("@compute @workgroup_size(4, 4, 4)");
        }
        wgsl.push_str_w_newline(
            &self
                .user_portion
                .main_function
                .as_ref()
                .unwrap()
                .code
                .wgsl_code
                .clone(),
        );
        wgsl
    }
}

// implement push_str_w_newline for String
trait PushStrWNewline {
    fn push_str_w_newline(&mut self, s: &str);
}
impl PushStrWNewline for String {
    fn push_str_w_newline(&mut self, s: &str) {
        self.push_str(s);
        self.push_str("\n");
    }
}

```

# src\wgsl_wgpu_binding.rs

```rs
use std::str::FromStr;

use crate::wgsl_components::WgslOutputArray;
#[derive(Clone, Debug)]
pub struct WgslWgpuBinding {
    pub group_num: u32,
    pub entry_num: u32,
    pub buffer_type: WgpuBufferType,
    pub access: WgpuBufferAccessMode,
    pub name: String,
    pub type_decl: String,
}
impl ToString for WgslWgpuBinding {
    fn to_string(&self) -> String {
        if self.buffer_type == WgpuBufferType::Uniform {
            return format!(
                "@group({}) @binding({}) var<{}> {}: {};\n",
                self.group_num,
                self.entry_num,
                self.buffer_type.to_string(),
                self.name,
                self.type_decl
            );
        }
        return format!(
            "@group({}) @binding({}) var<{}, {}> {}: {};\n",
            self.group_num,
            self.entry_num,
            self.buffer_type.to_string(),
            self.access.to_string(),
            self.name,
            self.type_decl
        );
    }
}

impl WgslWgpuBinding {
    pub fn uniform(group_num: u32, entry_num: u32, name: String, type_decl: &str) -> Self {
        WgslWgpuBinding {
            group_num,
            entry_num,
            buffer_type: WgpuBufferType::Uniform,
            access: WgpuBufferAccessMode::Read,
            name,
            type_decl: type_decl.to_string(),
        }
    }
    pub fn input_array(group_num: u32, entry_num: u32, name: String, type_decl: String) -> Self {
        WgslWgpuBinding {
            group_num,
            entry_num,
            buffer_type: WgpuBufferType::Storage,
            access: WgpuBufferAccessMode::Read,
            name,
            type_decl: type_decl,
        }
    }
    pub fn output_array(group_num: u32, entry_num: u32, name: String, type_decl: String) -> Self {
        WgslWgpuBinding {
            group_num,
            entry_num,
            buffer_type: WgpuBufferType::Storage,
            access: WgpuBufferAccessMode::ReadWrite,
            name,
            type_decl: type_decl,
        }
    }

    pub fn counter(
        entry_number: u32,
        out_array: &WgslOutputArray,
        out_array_binding: &WgslWgpuBinding,
    ) -> Self {
        assert!(
            out_array.atomic_counter_name.is_some(),
            "Atomic counter name must be present if you want to create a counter binding"
        );
        WgslWgpuBinding {
            group_num: out_array_binding.group_num,
            entry_num: entry_number,
            buffer_type: WgpuBufferType::Storage,
            access: WgpuBufferAccessMode::ReadWrite,
            name: out_array.atomic_counter_name.as_ref().unwrap().clone(),
            type_decl: "atomic<u32>".to_string(),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum WgpuBufferType {
    Storage,
    Uniform,
}
impl FromStr for WgpuBufferType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "storage" => Ok(WgpuBufferType::Storage),
            "uniform" => Ok(WgpuBufferType::Uniform),
            _ => Err("Invalid buffer access type".to_string()),
        }
    }
}
impl ToString for WgpuBufferType {
    fn to_string(&self) -> String {
        match self {
            WgpuBufferType::Storage => "storage".to_string(),
            WgpuBufferType::Uniform => "uniform".to_string(),
        }
    }
}
#[derive(Clone, Debug)]
pub enum WgpuBufferAccessMode {
    Read,
    ReadWrite,
}
impl FromStr for WgpuBufferAccessMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "read" => Ok(WgpuBufferAccessMode::Read),
            "read_write" => Ok(WgpuBufferAccessMode::ReadWrite),
            _ => Err("Invalid buffer access mode".to_string()),
        }
    }
}
impl ToString for WgpuBufferAccessMode {
    fn to_string(&self) -> String {
        match self {
            WgpuBufferAccessMode::Read => "read".to_string(),
            WgpuBufferAccessMode::ReadWrite => "read_write".to_string(),
        }
    }
}

```
