use bytemuck::Pod;

pub trait OutputVectorTypesSpec {
    type Output0: Pod + Send + Sync;
    type Output1: Pod + Send + Sync;
    type Output2: Pod + Send + Sync;
    type Output3: Pod + Send + Sync;
    type Output4: Pod + Send + Sync;
    type Output5: Pod + Send + Sync;
}

pub struct BlankOutputVectorTypesSpec {}
impl OutputVectorTypesSpec for BlankOutputVectorTypesSpec {
    type Output0 = ();
    type Output1 = ();
    type Output2 = ();
    type Output3 = ();
    type Output4 = ();
    type Output5 = ();
}
