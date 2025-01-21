use bevy_gpu_compute_core::misc_types::InputConfigTypesSpec;

pub struct BlankInputConfigTypesSpec {}
impl InputConfigTypesSpec for BlankInputConfigTypesSpec {
    type Input0 = ();
    type Input1 = ();
    type Input2 = ();
    type Input3 = ();
    type Input4 = ();
    type Input5 = ();
}
