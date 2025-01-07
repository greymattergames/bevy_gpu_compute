use super::{
    wgsl_components::{
        WgslArray, WgslConstAssignment, WgslFunction, WgslOutputArray, WgslShaderModuleUserPortion,
        WgslType, WgslWorkgroupDeclaration,
    },
    wgsl_wgpu_binding::WgslWgpuBinding,
};

/**
 * ! logic for parsing the shader module
 * *first get all types declared in the module
 * combine these with all standard wgsl types to get a list of all type idents allowed
 * then whenever we see a type that isn't one of those, throw an error
 * *traverse through module scope tokens:
 * extracting them out into the objects shown below
 *
 */

struct WgslShaderModuleLibraryPortion {
    pipeline_consts: Vec<WgslConstAssignment>,
    uniforms: Vec<WgslType>,
    helper_functions: Vec<WgslFunction>,
    bindings: Vec<WgslWgpuBinding>,
    workgroups_declaration: WgslWorkgroupDeclaration,
}

/// for internal library use only, contains all the components necessary to generate the final shader module
struct WgslShaderModuleComponents {
    user: WgslShaderModuleUserPortion,
    lib: WgslShaderModuleLibraryPortion,
}

fn compose_wgsl(module: WgslShaderModuleComponents) {
    let mut wgsl: String = String::new();
    // first add user static consts
    module
        .user
        .static_consts
        .iter()
        .for_each(|c| wgsl.push_str(&c.to_string()));
    // then add any miscelanious user helper types which are internal to the GPU only, not transfered to or from th CPU
    module.user.helper_types.iter().for_each(|t| {
        wgsl.push_str(&t.to_string());
    });
    // then add library pipeline consts
    module.lib.pipeline_consts.iter().for_each(|c| {
        wgsl.push_str(&c.to_string());
    });
    // then add user uniform definitions
    module.user.uniforms.iter().for_each(|u| {
        wgsl.push_str(&u.to_string());
    });
    // then add library uniform definitions
    module.lib.uniforms.iter().for_each(|u| {
        wgsl.push_str(&u.to_string());
    });
    // then add user input array definitions
    module.user.input_arrays.iter().for_each(|a| {
        wgsl.push_str(&a.item_type.to_string());
        wgsl.push_str(&a.to_string());
    });
    // then add user output array definitions
    module.user.output_arrays.iter().for_each(|a| {
        wgsl.push_str(&a.arr.item_type.to_string());
        wgsl.push_str(&a.to_string());
    });
    // now add wgpu bindings
    module.lib.bindings.iter().for_each(|b| {
        wgsl.push_str(&b.to_string());
    });
    // now add user helper functions
    module.user.helper_functions.iter().for_each(|f| {
        wgsl.push_str(&f.to_string());
    });
    // now add library helper functions
    module.lib.helper_functions.iter().for_each(|f| {
        wgsl.push_str(&f.to_string());
    });
    // now add the main function
    wgsl.push_str(&module.lib.workgroups_declaration.to_string());
    wgsl.push_str(&module.user.main_function.unwrap().to_string());
}
