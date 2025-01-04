use compute_lib::{ComputeInput, ComputeOutput, Vec2, Vec3, WGSLType, compute_shader};

// Implement for primitive types
impl WGSLType for f32 {
    const TYPE_NAME: &'static str = "f32";
    const STORAGE_COMPATIBLE: bool = true;
}
impl WGSLType for u32 {
    const TYPE_NAME: &'static str = "u32";
    const STORAGE_COMPATIBLE: bool = true;
}

impl<T: WGSLType> WGSLType for Vec2<T> {
    const TYPE_NAME: &'static str = "vec2<T>";
}

// Array types handled via const generics
impl<T: WGSLType, const N: usize> WGSLType for [T; N] {
    const TYPE_NAME: &'static str = "array<T, N>";
}

// Add this trait to allow access to the generated WGSL
pub trait WGSLShader {
    /// Get the generated WGSL code
    fn wgsl_code() -> String;

    /// Get the WGSL code with debug information
    fn debug_wgsl() -> String {
        format!(
            "// Generated WGSL for shader: {}\n\n{}",
            std::any::type_name::<Self>(),
            Self::wgsl_code()
        )
    }
}

// Example usage with the proc macro
#[derive(ComputeInput)]
struct Position(Vec2<f32>);

#[derive(ComputeInput)]
struct Radius(f32);

#[derive(ComputeOutput)]
struct CollisionResult {
    entity1: u32,
    entity2: u32,
}

#[compute_shader]
fn detect_collisions(positions: &[Position], radii: &[Radius], results: &mut Vec<CollisionResult>) {
    //todo  let idx = global_id().x as usize;
    //todo  let other_idx = global_id().y as usize;
    let idx = 0;
    let other_idx = 0;

    if idx >= positions.len() || other_idx >= positions.len() || idx >= other_idx {
        return;
    }

    let pos1 = positions[idx].0;
    let pos2 = positions[other_idx].0;
    let r1 = radii[idx].0;
    let r2 = radii[other_idx].0;

    let dx = pos1.0 - pos2.0;
    let dy = pos1.1 - pos2.1;
    let dist_sq = dx * dx + dy * dy;

    let sum_radii = r1 + r2;
    if dist_sq < sum_radii * sum_radii {
        results.push(CollisionResult {
            entity1: idx as u32,
            entity2: other_idx as u32,
        });
    }
}

fn main() {
    use compute_lib::WGSLShader;
    println!("{}", <detect_collisions as WGSLShader>::wgsl_code());
    println!("{}", <detect_collisions as WGSLShader>::debug_wgsl());
    // Print the generated WGSL code
    // println!("{}", detect_collisions::wgsl_code());

    // Or with debug info
    // println!("{}", detect_collisions::debug_wgsl());
}
