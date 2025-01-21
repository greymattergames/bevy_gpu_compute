use shared::misc_types::{BlankTypesSpec, OutputVectorTypesSpec, TypesSpec};

pub struct OutputData<T: TypesSpec> {
    output0: Option<Vec<<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output0>>,
    output1: Option<Vec<<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output1>>,
    output2: Option<Vec<<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output2>>,
    output3: Option<Vec<<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output3>>,
    output4: Option<Vec<<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output4>>,
    output5: Option<Vec<<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output5>>,

    _phantom: std::marker::PhantomData<T>,
}
impl<T: TypesSpec> std::fmt::Debug for OutputData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputData")
            .field("output0", &self.output0)
            .field("output1", &self.output1)
            .field("output2", &self.output2)
            .field("output3", &self.output3)
            .field("output4", &self.output4)
            .field("output5", &self.output5)
            .finish()
    }
}

impl Default for OutputData<BlankTypesSpec> {
    fn default() -> Self {
        OutputData {
            output0: None,
            output1: None,
            output2: None,
            output3: None,
            output4: None,
            output5: None,

            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: TypesSpec> OutputData<T> {
    pub fn empty() -> Self {
        OutputData {
            output0: None,
            output1: None,
            output2: None,
            output3: None,
            output4: None,
            output5: None,

            _phantom: std::marker::PhantomData,
        }
    }

    // Set outputs from raw bytes
    pub fn set_output0_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len()
            % std::mem::size_of::<
                <<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output0,
            >()
            != 0
        {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output0 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }

    pub fn set_output1_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len()
            % std::mem::size_of::<
                <<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output1,
            >()
            != 0
        {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output1 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }
    pub fn set_output2_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len()
            % std::mem::size_of::<
                <<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output2,
            >()
            != 0
        {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output2 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }
    pub fn set_output3_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len()
            % std::mem::size_of::<
                <<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output3,
            >()
            != 0
        {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output3 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }
    pub fn set_output4_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len()
            % std::mem::size_of::<
                <<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output4,
            >()
            != 0
        {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output4 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }
    pub fn set_output5_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len()
            % std::mem::size_of::<
                <<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output5,
            >()
            != 0
        {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output5 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }

    // Type-safe getters for processed results
    pub fn get_output0(
        &self,
    ) -> Option<&[<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output0]> {
        self.output0.as_deref()
    }

    pub fn get_output1(
        &self,
    ) -> Option<&[<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output1]> {
        self.output1.as_deref()
    }
    pub fn get_output2(
        &self,
    ) -> Option<&[<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output2]> {
        self.output2.as_deref()
    }
    pub fn get_output3(
        &self,
    ) -> Option<&[<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output3]> {
        self.output3.as_deref()
    }
    pub fn get_output4(
        &self,
    ) -> Option<&[<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output4]> {
        self.output4.as_deref()
    }
    pub fn get_output5(
        &self,
    ) -> Option<&[<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output5]> {
        self.output5.as_deref()
    }
}
