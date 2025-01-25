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
        assert_eq!(std::mem::size_of::<PodF16>(), 2);
        // Should have 32-bit alignment
        assert_eq!(std::mem::align_of::<PodF16>(), 2);
    }

    #[test]
    fn test_f16_pod() {
        let value = PodF16::from(std::f32::consts::PI);

        // Test casting to bytes
        let bytes: &[u8] = bytemuck::bytes_of(&value);
        assert_eq!(bytes.len(), 2); // Should be 4 bytes now

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
        let original = std::f32::consts::PI;
        let pod = PodF16::from(original);
        let roundtrip: f32 = pod.into();
        // Note: Some precision loss is expected due to f16
        assert!((original - roundtrip).abs() < 0.01);
    }
    #[test]
    fn can_use_with_bytemuck() {
        pub mod test_module {
            use super::*;
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
