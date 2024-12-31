use bevy::reflect::Tuple;

// Helper trait to convert a tuple type to a tuple of vectors
pub trait ToVecTuple<T: Tuple> {
    type VecTuple: Tuple;
}

// You can implement this for specific tuple sizes
// impl<
//     T1: bevy::reflect::Reflect
//         + bevy::reflect::GetTypeRegistration
//         + bevy::reflect::Typed
//         + bevy::reflect::FromReflect,
// > ToVecTuple<(T1,)> for ()
// {
//     type VecTuple = (Vec<T1>,);
// }
macro_rules! impl_to_vec_tuple {
    ($($t:ident),+) => {
        impl<$($t),+> ToVecTuple<($($t,)+)> for ()
        where
            $(
                $t: bevy::reflect::Reflect
                    + bevy::reflect::GetTypeRegistration
                    + bevy::reflect::Typed
                    + bevy::reflect::FromReflect,
            )+
        {
            type VecTuple = ($(Vec<$t>,)+);
        }
    }
}

impl_to_vec_tuple!(T1);
impl_to_vec_tuple!(T1, T2);
impl_to_vec_tuple!(T1, T2, T3);
impl_to_vec_tuple!(T1, T2, T3, T4);
impl_to_vec_tuple!(T1, T2, T3, T4, T5);
impl_to_vec_tuple!(T1, T2, T3, T4, T5, T6);
impl_to_vec_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_to_vec_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_to_vec_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_to_vec_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_to_vec_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_to_vec_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
