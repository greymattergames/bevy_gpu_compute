use crate::{
    wgsl_components::{
        WORKGROUP_SIZE_X_VAR_NAME, WORKGROUP_SIZE_Y_VAR_NAME, WORKGROUP_SIZE_Z_VAR_NAME,
        WgpuShaderType, WgslShaderModuleComponent,
    },
    wgsl_shader_module_lib_portion::WgslShaderModuleLibraryPortion,
};

use super::{
    wgsl_components::{
        WgslConstAssignment, WgslFunction, WgslShaderModuleUserPortion, WgslType,
        WgslWorkgroupDeclaration,
    },
    wgsl_wgpu_binding::WgslWgpuBinding,
};

pub struct WgslShaderModule {
    pub user_portion: WgslShaderModuleUserPortion,
    pub library_portion: WgslShaderModuleLibraryPortion,
    pub wgsl_code: String,
}

// implement push_str_w_newline for String
trait PushStrWNewline {
    fn push_str_w_newline(&mut self, s: &str);
}
impl PushStrWNewline for String {
    fn push_str_w_newline(&mut self, s: &str) {
        self.push_str(s);
        self.push_str("\n");
    }
}

pub fn compose_wgsl(module: WgslShaderModuleUserPortion) -> WgslShaderModule {
    let lib_portion = WgslShaderModuleLibraryPortion::from(&module);
    let mut wgsl: String = String::new();
    // first add user static consts
    module
        .static_consts
        .iter()
        .for_each(|c| wgsl.push_str_w_newline(&c.code.wgsl_code.clone()));
    // then add any miscelanious user helper types which are internal to the GPU only, not transfered to or from th CPU
    module.helper_types.iter().for_each(|t| {
        wgsl.push_str_w_newline(&t.code.wgsl_code.clone());
    });
    // then add library pipeline consts
    // these include lengths of arrays, and workgroup sizes
    lib_portion.pipeline_consts.iter().for_each(|c| {
        wgsl.push_str_w_newline(&c.code.wgsl_code.clone());
    });
    // then add user uniform definitions
    module.uniforms.iter().for_each(|u| {
        wgsl.push_str_w_newline(&u.code.wgsl_code.clone());
    });
    // then add library uniform definitions
    lib_portion.uniforms.iter().for_each(|u| {
        wgsl.push_str_w_newline(&u.code.wgsl_code.clone());
    });
    // then add user input array definitions
    module.input_arrays.iter().for_each(|a| {
        wgsl.push_str_w_newline(&a.item_type.code.wgsl_code.clone());
    });
    // then add user output array definitions
    module.output_arrays.iter().for_each(|a| {
        wgsl.push_str_w_newline(&a.item_type.code.wgsl_code.clone());
    });
    // now add wgpu bindings
    lib_portion.bindings.iter().for_each(|b| {
        wgsl.push_str_w_newline(&b.to_string());
    });
    // now add user helper functions
    module.helper_functions.iter().for_each(|f| {
        wgsl.push_str_w_newline(&f.code.wgsl_code.clone());
    });
    // now add library helper functions
    lib_portion.helper_functions.iter().for_each(|f| {
        wgsl.push_str_w_newline(&f.code.wgsl_code.clone());
    });
    // now add the main function
    wgsl.push_str_w_newline(&lib_portion.workgroups_declaration.to_string());
    wgsl.push_str_w_newline(
        &module
            .main_function
            .as_ref()
            .unwrap()
            .code
            .wgsl_code
            .clone(),
    );
    WgslShaderModule {
        user_portion: module,
        library_portion: lib_portion,
        wgsl_code: wgsl,
    }
}
