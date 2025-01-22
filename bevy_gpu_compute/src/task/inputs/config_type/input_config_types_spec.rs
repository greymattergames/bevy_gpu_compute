use bevy_gpu_compute_core::ConfigInputTypesSpec;

//todo implement configs
pub struct BlankInputConfigTypesSpec {}
impl ConfigInputTypesSpec for BlankInputConfigTypesSpec {
    type Input0 = ();
    type Input1 = ();
    type Input2 = ();
    type Input3 = ();
    type Input4 = ();
    type Input5 = ();
}
