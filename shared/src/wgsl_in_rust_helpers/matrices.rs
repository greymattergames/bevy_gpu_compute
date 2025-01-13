use crate::wgsl_in_rust_helpers::vectors::*;
macro_rules! impl_matrix {
    ($name:ident, $vec_type:ty, $($field:ident, $index:expr),+) => {
        #[derive(Debug, Clone)]
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

// Generate concrete types for numeric types (matrices don't make sense for booleans)
define_matrix_types!(u32, U32);
define_matrix_types!(i32, I32);
define_matrix_types!(f32, F32);
define_matrix_types!(f16, F16);
define_matrix_types!(bool, Bool);

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
