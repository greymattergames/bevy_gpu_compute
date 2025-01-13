macro_rules! impl_vector {
    ($name:ident, $type:ty, $($field:ident, $index:expr),+) => {
        #[derive(Debug, Clone,Copy)]
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

// Generate concrete types for all scalar types
define_vector_types!(bool, Bool);
define_vector_types!(u32, U32);
define_vector_types!(i32, I32);
define_vector_types!(f32, F32);
define_vector_types!(f16, F16);

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
