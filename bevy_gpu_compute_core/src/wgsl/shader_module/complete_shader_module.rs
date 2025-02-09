use crate::IterSpaceDimmension;
use crate::wgsl::shader_module::derived_portion::WgslShaderModuleDerivedPortion;
use crate::wgsl::shader_module::user_defined_portion::WgslShaderModuleUserPortion;
pub struct WgslShaderModule {
    pub user_portion: WgslShaderModuleUserPortion,
    pub library_portion: WgslShaderModuleDerivedPortion,
}
impl WgslShaderModule {
    pub fn new(module: WgslShaderModuleUserPortion) -> WgslShaderModule {
        let library_portion = WgslShaderModuleDerivedPortion::from(&module);
        WgslShaderModule {
            user_portion: module,
            library_portion,
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
        if let Some(main_function) = &self.user_portion.main_function {
            wgsl.push_str_w_newline(
                &main_function
                    .code
                    .wgsl_code
                    .clone(),
            );
        }
        wgsl
    }
}

// implement push_str_w_newline for String
trait _PushStrWNewline {
    fn push_str_w_newline(&mut self, s: &str);
}
impl _PushStrWNewline for String {
    fn push_str_w_newline(&mut self, s: &str) {
        self.push_str(s);
        self.push('\n');
    }
}
