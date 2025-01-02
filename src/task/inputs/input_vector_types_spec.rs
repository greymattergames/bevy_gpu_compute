use bytemuck::Pod;

pub trait InputVectorTypesSpec {
    type Input0: Pod + Send + Sync;
    type Input1: Pod + Send + Sync;
    type Input2: Pod + Send + Sync;
    type Input3: Pod + Send + Sync;
    type Input4: Pod + Send + Sync;
    type Input5: Pod + Send + Sync;
}

pub struct BlankInputVectorTypesSpec {}
impl InputVectorTypesSpec for BlankInputVectorTypesSpec {
    type Input0 = ();
    type Input1 = ();
    type Input2 = ();
    type Input3 = ();
    type Input4 = ();
    type Input5 = ();
}
